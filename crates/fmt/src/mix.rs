//! MIX archives: the container format of the user's install.
//!
//! A MIX file is an index followed by a body. Index entries hold a 32-bit ID
//! (a hash of the file name; the names themselves are not stored), an offset
//! into the body and a size. Two layouts exist:
//!
//! - **classic**: `u16 count, u32 body size, entries`;
//! - **flagged**: `u32 flags`, then the classic header. Flag
//!   [`FLAG_ENCRYPTED`] means the header after the flags is an 80-byte RSA
//!   key source followed by the Blowfish-encrypted header, padded to whole
//!   8-byte blocks; [`FLAG_CHECKSUM`] means a 20-byte digest follows the body.
//!
//! The index reader is incremental ([`MixIndex::parse`]): given a prefix of
//! the archive it either returns the index or [`Error::Truncated`] with the
//! prefix length it needs, so callers can read archives lazily (web
//! `File.slice`, nested archives inside other archives).
//!
//! Layout details follow XCC's `mix_file.cpp` and OpenRA's `MixFile.cs` and
//! `PackageEntry.cs` (both GPL-3.0-or-later; XCC Copyright (C) 2000 Olaf van
//! der Spek, OpenRA Copyright (c) The OpenRA Developers and Contributors).

use blowfish::cipher::{Block, BlockCipherDecrypt, BlockCipherEncrypt, KeyInit};
use blowfish::{Blowfish, BlowfishLE};

use crate::bytes::{PutLe, Reader, len_u16, len_u32, slice64, wide};
use crate::error::{Error, Result};
use crate::mix_key::{BLOWFISH_KEY_LEN, KEY_SOURCE_LEN, RsaKeyPair, RsaPublicKey};

/// Flag: a 20-byte digest follows the body.
pub const FLAG_CHECKSUM: u32 = 0x0001_0000;
/// Flag: the header is encrypted.
pub const FLAG_ENCRYPTED: u32 = 0x0002_0000;
/// Length of one index entry.
pub const ENTRY_LEN: usize = 12;
/// Length of the digest that [`FLAG_CHECKSUM`] announces.
pub const CHECKSUM_LEN: u64 = 20;
/// Blowfish block length.
const BLOCK: usize = 8;

/// Word order used when running Blowfish over the encrypted header.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum HeaderCipher {
    /// Each 8-byte block is two big-endian words: standard Blowfish. XCC and
    /// OpenRA read little-endian words and byte-swap them before the
    /// cipher, which amounts to this.
    #[default]
    BigEndianWords,
    /// Each 8-byte block is two little-endian words (`BlowfishLE`).
    LittleEndianWords,
}

impl HeaderCipher {
    fn apply(self, key: &[u8; BLOWFISH_KEY_LEN], data: &mut [u8], encrypt: bool) -> Result<()> {
        let bad_key = |_| Error::Invalid("Blowfish key");
        match self {
            Self::BigEndianWords => {
                let c: Blowfish = Blowfish::new_from_slice(key).map_err(bad_key)?;
                run_blocks(data, |b| {
                    if encrypt {
                        c.encrypt_block(b)
                    } else {
                        c.decrypt_block(b)
                    }
                });
            }
            Self::LittleEndianWords => {
                let c = BlowfishLE::new_from_slice(key).map_err(bad_key)?;
                run_blocks(data, |b| {
                    if encrypt {
                        c.encrypt_block(b)
                    } else {
                        c.decrypt_block(b)
                    }
                });
            }
        }
        Ok(())
    }
}

fn run_blocks(data: &mut [u8], mut f: impl FnMut(&mut Block<Blowfish>)) {
    for chunk in data.as_chunks_mut::<BLOCK>().0 {
        let mut block = Block::<Blowfish>::from(*chunk);
        f(&mut block);
        chunk.copy_from_slice(&block);
    }
}

/// How the index is stored.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MixLayout {
    /// No flags word.
    Classic,
    /// A flags word precedes the header.
    Flagged {
        /// The header is encrypted.
        encrypted: bool,
        /// A digest follows the body.
        checksum: bool,
    },
}

/// One index entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MixEntry {
    /// Name hash (see [`id_hash`]).
    pub id: u32,
    /// Offset from the start of the body.
    pub offset: u32,
    /// Size in bytes.
    pub size: u32,
}

/// A parsed MIX index.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MixIndex {
    /// Storage layout.
    pub layout: MixLayout,
    /// Offset of the body from the start of the archive (the header length).
    pub body_offset: u64,
    /// Body size stated by the header.
    pub body_size: u32,
    /// Entries in file order.
    pub entries: Vec<MixEntry>,
    /// Indices into `entries`, sorted by ID then file order.
    by_id: Vec<u32>,
}

/// Keys and cipher settings for reading encrypted headers.
#[derive(Clone, Debug)]
pub struct MixKeys {
    /// Key that unwraps the key source.
    pub rsa: RsaPublicKey,
    /// Blowfish word order.
    pub cipher: HeaderCipher,
}

impl Default for MixKeys {
    /// The Westwood public key with standard Blowfish.
    fn default() -> Self {
        Self {
            rsa: RsaPublicKey::westwood(),
            cipher: HeaderCipher::default(),
        }
    }
}

/// Rounds `n` up to whole Blowfish blocks.
fn round_to_block(n: usize) -> usize {
    n.div_ceil(BLOCK) * BLOCK
}

impl MixIndex {
    /// Reads the index from a prefix of an archive.
    ///
    /// Returns [`Error::Truncated`] with the prefix length needed when
    /// `prefix` is too short; any longer prefix (up to the whole archive)
    /// gives the same result.
    pub fn parse(prefix: &[u8], keys: &MixKeys) -> Result<Self> {
        let mut r = Reader::new(prefix);
        let first = r.u16()?;
        if first != 0 {
            let count = usize::from(first);
            let body_size = r.u32()?;
            let entries = read_entries(&mut r, count)?;
            return Ok(Self::new(
                MixLayout::Classic,
                wide(r.pos()),
                body_size,
                entries,
            ));
        }
        let mut r = Reader::new(prefix);
        let flags = r.u32()?;
        if flags & !(FLAG_CHECKSUM | FLAG_ENCRYPTED) != 0 {
            return Err(Error::Invalid("unknown MIX flags"));
        }
        let layout = MixLayout::Flagged {
            encrypted: flags & FLAG_ENCRYPTED != 0,
            checksum: flags & FLAG_CHECKSUM != 0,
        };
        if flags & FLAG_ENCRYPTED == 0 {
            let count = usize::from(r.u16()?);
            let body_size = r.u32()?;
            let entries = read_entries(&mut r, count)?;
            return Ok(Self::new(layout, wide(r.pos()), body_size, entries));
        }

        let source: [u8; KEY_SOURCE_LEN] = r.array()?;
        let key = keys.rsa.unwrap_key(&source)?;
        let enc_start = r.pos();
        let mut first_block: [u8; BLOCK] = r.array()?;
        keys.cipher.apply(&key, &mut first_block, false)?;
        let count = usize::from(u16::from_le_bytes([first_block[0], first_block[1]]));
        let enc_len = round_to_block(6 + count * ENTRY_LEN);
        let mut header = Reader::at(prefix, enc_start).bytes(enc_len)?.to_vec();
        keys.cipher.apply(&key, &mut header, false)?;
        let mut h = Reader::new(&header);
        h.skip(2)?;
        let body_size = h.u32()?;
        let entries = read_entries(&mut h, count)?;
        Ok(Self::new(
            layout,
            wide(enc_start + enc_len),
            body_size,
            entries,
        ))
    }

    fn new(layout: MixLayout, body_offset: u64, body_size: u32, entries: Vec<MixEntry>) -> Self {
        let mut by_id: Vec<u32> = (0..len_u32(entries.len())).collect();
        by_id.sort_by_key(|&i| (entries[i as usize].id, i));
        Self {
            layout,
            body_offset,
            body_size,
            entries,
            by_id,
        }
    }

    /// The first entry (in file order) with this ID.
    pub fn find(&self, id: u32) -> Option<&MixEntry> {
        let pos = self
            .by_id
            .partition_point(|&i| self.entries[i as usize].id < id);
        let &i = self.by_id.get(pos)?;
        let e = &self.entries[i as usize];
        (e.id == id).then_some(e)
    }

    /// Whether some ID occurs more than once.
    pub fn has_duplicate_ids(&self) -> bool {
        self.by_id
            .windows(2)
            .any(|w| self.entries[w[0] as usize].id == self.entries[w[1] as usize].id)
    }

    /// Archive length implied by the header: header, body and digest.
    pub fn implied_len(&self) -> u64 {
        let digest = match self.layout {
            MixLayout::Flagged { checksum: true, .. } => CHECKSUM_LEN,
            _ => 0,
        };
        self.body_offset + u64::from(self.body_size) + digest
    }

    /// Byte range of `entry` within the archive, checked against the body.
    pub fn range(&self, entry: &MixEntry) -> Result<std::ops::Range<u64>> {
        let end = u64::from(entry.offset) + u64::from(entry.size);
        if end > u64::from(self.body_size) {
            return Err(Error::Invalid("MIX entry extends past the body"));
        }
        let start = self.body_offset + u64::from(entry.offset);
        Ok(start..start + u64::from(entry.size))
    }

    /// Strict consistency check against the archive's real length: the
    /// implied length matches exactly and every entry lies in the body.
    /// Used to recognise nested archives by content.
    pub fn check_consistent(&self, archive_len: u64) -> Result<()> {
        if self.implied_len() != archive_len {
            return Err(Error::Invalid(
                "MIX header does not match the archive length",
            ));
        }
        for e in &self.entries {
            self.range(e)?;
        }
        Ok(())
    }
}

fn read_entries(r: &mut Reader<'_>, count: usize) -> Result<Vec<MixEntry>> {
    // Check the whole index is present before allocating for it.
    let need = r.pos() + count * ENTRY_LEN;
    if r.remaining() < count * ENTRY_LEN {
        return Err(Error::Truncated { needed: wide(need) });
    }
    let mut entries = Vec::with_capacity(count);
    for _ in 0..count {
        entries.push(MixEntry {
            id: r.u32()?,
            offset: r.u32()?,
            size: r.u32()?,
        });
    }
    Ok(entries)
}

/// A whole archive held in memory.
#[derive(Clone, Debug)]
pub struct MixArchive<'a> {
    /// The index.
    pub index: MixIndex,
    data: &'a [u8],
}

impl<'a> MixArchive<'a> {
    /// Parses the index of `data`.
    pub fn new(data: &'a [u8], keys: &MixKeys) -> Result<Self> {
        Ok(Self {
            index: MixIndex::parse(data, keys)?,
            data,
        })
    }

    /// Contents of `entry`.
    pub fn read(&self, entry: &MixEntry) -> Result<&'a [u8]> {
        let r = self.index.range(entry)?;
        slice64(self.data, r.start, r.end - r.start)
    }

    /// Contents of the file with this ID, if present.
    pub fn get(&self, id: u32) -> Option<Result<&'a [u8]>> {
        self.index.find(id).map(|e| self.read(e))
    }
}

/// ID of a file name in TS/RA2-era archives: CRC-32 of the upper-cased name,
/// padded to a multiple of four bytes. The padding is the remainder
/// `len % 4` as a byte, then copies of the first byte of the last partial
/// group. (Port of `PackageEntry.HashFilename`, CRC32 case, OpenRA.)
pub fn id_hash(name: &str) -> u32 {
    let mut buf: Vec<u8> = name.bytes().map(|b| b.to_ascii_uppercase()).collect();
    let len = buf.len();
    let rem = len % 4;
    if rem != 0 {
        let fill = buf[len - rem];
        // `rem` is 1..=3.
        buf.push(rem as u8);
        buf.extend(std::iter::repeat_n(fill, 3 - rem));
    }
    crc32(&buf)
}

/// ID of a file name in classic (earlier) archives: a rotate-and-add over
/// the upper-cased name as little-endian words, zero padded.
pub fn id_hash_classic(name: &str) -> u32 {
    let mut buf: Vec<u8> = name.bytes().map(|b| b.to_ascii_uppercase()).collect();
    buf.resize(buf.len().div_ceil(4) * 4, 0);
    buf.as_chunks::<4>().0.iter().fold(0u32, |acc, w| {
        acc.rotate_left(1).wrapping_add(u32::from_le_bytes(*w))
    })
}

/// Standard CRC-32 (reflected polynomial 0xEDB88320, initial and final
/// value all ones).
pub fn crc32(data: &[u8]) -> u32 {
    const fn table() -> [u32; 256] {
        let mut t = [0u32; 256];
        let mut i = 0;
        while i < 256 {
            let mut c = i as u32;
            let mut k = 0;
            while k < 8 {
                c = if c & 1 != 0 {
                    0xEDB8_8320 ^ (c >> 1)
                } else {
                    c >> 1
                };
                k += 1;
            }
            t[i] = c;
            i += 1;
        }
        t
    }
    const TABLE: [u32; 256] = table();
    !data.iter().fold(!0u32, |crc, &b| {
        TABLE[((crc ^ u32::from(b)) & 0xFF) as usize] ^ (crc >> 8)
    })
}

/// How [`MixWriter`] lays out the index.
#[derive(Clone, Debug)]
pub enum WriteLayout {
    /// Classic layout.
    Classic,
    /// Flags word, plain header.
    Plain,
    /// Flags word, encrypted header.
    Encrypted {
        /// Blowfish key for the header.
        key: [u8; BLOWFISH_KEY_LEN],
        /// Key pair that wraps the key (normally [`RsaKeyPair::synthetic`]).
        rsa: RsaKeyPair,
        /// Blowfish word order.
        cipher: HeaderCipher,
    },
}

/// Writes synthetic MIX archives.
#[derive(Clone, Debug)]
pub struct MixWriter {
    /// Index layout.
    pub layout: WriteLayout,
    /// Files as (ID, contents), stored in this order in the body; the index
    /// lists them sorted by ID.
    pub files: Vec<(u32, Vec<u8>)>,
}

impl MixWriter {
    /// Serialises the archive.
    ///
    /// # Panics
    /// If there are more than 65535 files or the body exceeds 4 GiB, or the
    /// RSA key cannot wrap keys (a writer-side programming error).
    #[allow(clippy::expect_used)]
    pub fn write(&self) -> Vec<u8> {
        let mut entries = Vec::with_capacity(self.files.len());
        let mut body = Vec::new();
        for (id, data) in &self.files {
            entries.push(MixEntry {
                id: *id,
                offset: len_u32(body.len()),
                size: len_u32(data.len()),
            });
            body.extend_from_slice(data);
        }
        entries.sort();
        let mut header = Vec::new();
        header.put_u16(len_u16(entries.len()));
        header.put_u32(len_u32(body.len()));
        for e in &entries {
            header.put_u32(e.id);
            header.put_u32(e.offset);
            header.put_u32(e.size);
        }
        let mut out = Vec::new();
        match &self.layout {
            WriteLayout::Classic => {
                assert!(!entries.is_empty(), "a classic MIX needs at least one file");
                out.extend_from_slice(&header);
            }
            WriteLayout::Plain => {
                out.put_u32(0);
                out.extend_from_slice(&header);
            }
            WriteLayout::Encrypted { key, rsa, cipher } => {
                out.put_u32(FLAG_ENCRYPTED);
                out.extend_from_slice(&rsa.wrap_key(key).expect("synthetic key wraps"));
                header.resize(round_to_block(header.len()), 0);
                cipher
                    .apply(key, &mut header, true)
                    .expect("56-byte key is valid");
                out.extend_from_slice(&header);
            }
        }
        out.extend_from_slice(&body);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::synth::SynthRng;

    fn files(seed: u64, n: usize) -> Vec<(u32, Vec<u8>)> {
        let mut rng = SynthRng::new(seed);
        (0..n)
            .map(|i| {
                let mut data = vec![0u8; (rng.next_u64() % 300) as usize];
                rng.fill(&mut data);
                (id_hash(&format!("synthetic{i}.bin")), data)
            })
            .collect()
    }

    fn synthetic_keys(cipher: HeaderCipher) -> MixKeys {
        MixKeys {
            rsa: RsaKeyPair::synthetic().public().clone(),
            cipher,
        }
    }

    fn check(layout: WriteLayout, keys: &MixKeys) {
        let files = files(3, 17);
        let bytes = MixWriter {
            layout,
            files: files.clone(),
        }
        .write();
        let archive = MixArchive::new(&bytes, keys).unwrap();
        archive.index.check_consistent(wide(bytes.len())).unwrap();
        assert_eq!(archive.index.entries.len(), files.len());
        for (id, data) in &files {
            assert_eq!(archive.get(*id).unwrap().unwrap(), data.as_slice());
        }
        // Every shorter prefix asks for more, and the stated need suffices.
        let header_len = archive.index.body_offset as usize;
        for cut in 0..header_len {
            match MixIndex::parse(&bytes[..cut], keys) {
                Err(Error::Truncated { needed }) => {
                    assert!(needed as usize > cut, "cut {cut} needs {needed}")
                }
                other => panic!("prefix {cut}: {other:?}"),
            }
        }
        assert_eq!(
            MixIndex::parse(&bytes[..header_len], keys).unwrap(),
            archive.index
        );
    }

    #[test]
    fn classic_round_trip() {
        check(WriteLayout::Classic, &MixKeys::default());
    }

    #[test]
    fn plain_round_trip() {
        check(WriteLayout::Plain, &MixKeys::default());
    }

    #[test]
    fn encrypted_round_trip_both_word_orders() {
        for cipher in [
            HeaderCipher::BigEndianWords,
            HeaderCipher::LittleEndianWords,
        ] {
            let mut key = [0u8; BLOWFISH_KEY_LEN];
            SynthRng::new(11).fill(&mut key);
            check(
                WriteLayout::Encrypted {
                    key,
                    rsa: RsaKeyPair::synthetic(),
                    cipher,
                },
                &synthetic_keys(cipher),
            );
        }
    }

    #[test]
    fn word_orders_differ() {
        // A round trip cannot tell the two orders apart; this shows they
        // really are different ciphers, so the choice matters for real files.
        let key = [7u8; BLOWFISH_KEY_LEN];
        let mut a = *b"synthblk";
        let mut b = a;
        HeaderCipher::BigEndianWords
            .apply(&key, &mut a, true)
            .unwrap();
        HeaderCipher::LittleEndianWords
            .apply(&key, &mut b, true)
            .unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn blowfish_big_endian_matches_published_vector() {
        // Eric Young's Blowfish test vectors (ECB): key and plaintext all
        // zero bytes give 4EF997456198DD78. Expanded to 56 key bytes by
        // repetition, which is equivalent for the key schedule.
        let key = [0u8; BLOWFISH_KEY_LEN];
        let mut block = [0u8; 8];
        HeaderCipher::BigEndianWords
            .apply(&key, &mut block, true)
            .unwrap();
        assert_eq!(block, [0x4E, 0xF9, 0x97, 0x45, 0x61, 0x98, 0xDD, 0x78]);
    }

    #[test]
    fn crc32_reference() {
        assert_eq!(crc32(b"123456789"), 0xCBF4_3926);
    }

    #[test]
    fn id_hash_padding_rule() {
        // Padding follows the rule in the doc comment, checked by building
        // the padded buffer by hand.
        assert_eq!(id_hash("abcd"), crc32(b"ABCD"));
        assert_eq!(id_hash("abcde"), crc32(b"ABCDE\x01EE"));
        assert_eq!(id_hash("abcdef"), crc32(b"ABCDEF\x02E"));
        assert_eq!(id_hash("abcdefg"), crc32(b"ABCDEFG\x03"));
        assert_eq!(id_hash("Some.File"), id_hash("SOME.FILE"));
        assert_eq!(id_hash(""), crc32(b""));
    }

    #[test]
    fn classic_hash_rotates_and_adds() {
        let w = u32::from_le_bytes(*b"ABCD");
        let v = u32::from_le_bytes([b'E', 0, 0, 0]);
        assert_eq!(id_hash_classic("abcde"), w.rotate_left(1).wrapping_add(v));
    }

    #[test]
    fn rejects_unknown_flags_and_bad_ranges() {
        let flags = [0, 0, 0, 0x80];
        assert_eq!(
            MixIndex::parse(&flags, &MixKeys::default()),
            Err(Error::Invalid("unknown MIX flags"))
        );
        let bytes = MixWriter {
            layout: WriteLayout::Plain,
            files: files(1, 2),
        }
        .write();
        let index = MixIndex::parse(&bytes, &MixKeys::default()).unwrap();
        assert!(index.check_consistent(wide(bytes.len()) + 1).is_err());
        let past = MixEntry {
            id: 0,
            offset: index.body_size,
            size: 1,
        };
        assert!(index.range(&past).is_err());
    }
}
