//! Keys for encrypted MIX headers.
//!
//! An encrypted MIX header starts with an 80-byte *key source*. Raising each
//! 40-byte little-endian block of it to the public exponent modulo the
//! published Westwood modulus yields 39 bytes of plain key material; the
//! first 56 bytes of that are the Blowfish key for the rest of the header.
//!
//! Derived from OpenRA's `BlowfishKeyProvider.cs`
//! (OpenRA.Mods.Cnc/FileFormats), Copyright (c) The OpenRA Developers and
//! Contributors, GPL-3.0-or-later: the public key string, its encoding and
//! the block framing come from there. The big-number arithmetic is replaced
//! by `num-bigint`.
//!
//! Writers cannot produce key sources that the Westwood key accepts (that
//! would need its private half, which we do not have), so synthetic
//! encrypted fixtures use [`RsaKeyPair::synthetic`], a key pair of our own
//! with the same modulus size, and are parsed with its public half.

use base64::Engine;
use num_bigint::BigUint;

use crate::error::{Error, Result};

/// Length of the RSA-encrypted key source at the start of an encrypted header.
pub const KEY_SOURCE_LEN: usize = 80;
/// Length of the Blowfish key recovered from a key source.
pub const BLOWFISH_KEY_LEN: usize = 56;

/// The published Westwood public key as distributed with OpenRA and XCC:
/// base64 of a DER-style INTEGER (tag 2, length, big-endian modulus).
const WESTWOOD_PUBLIC_KEY_B64: &str = "AihRvNoIbTn85FZRYNZRcT+i6KpU+maCsEqr3Q5q+LDB5tH7Tz2qQ38V";

/// Public exponent of the Westwood key (and of our synthetic key).
const PUBLIC_EXPONENT: u32 = 65537;

/// An RSA public key used to unwrap MIX key sources.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RsaPublicKey {
    modulus: BigUint,
    exponent: BigUint,
}

impl RsaPublicKey {
    /// The published Westwood key that the user's archives are made with.
    pub fn westwood() -> Self {
        // The constant is valid, so neither step can fail; fall back to an
        // empty modulus (which every key source then fails) rather than panic.
        let modulus = base64::engine::general_purpose::STANDARD
            .decode(WESTWOOD_PUBLIC_KEY_B64)
            .ok()
            .and_then(|der| parse_der_integer(&der))
            .unwrap_or_default();
        Self {
            modulus,
            exponent: BigUint::from(PUBLIC_EXPONENT),
        }
    }

    /// Bit length of the modulus.
    pub fn bits(&self) -> u64 {
        self.modulus.bits()
    }

    /// Plain bytes produced per key-source block: one less than the block
    /// length, so every block value stays below the modulus.
    fn plain_block_len(&self) -> Result<usize> {
        // OpenRA: len = bits - 1; a = (len - 1) / 8.
        let a = self.bits().saturating_sub(2) / 8;
        match usize::try_from(a) {
            Ok(a) if a > 0 => Ok(a),
            _ => Err(Error::Unsupported(
                "RSA modulus too small for MIX key sources",
            )),
        }
    }

    /// Recovers the Blowfish key from an 80-byte key source.
    pub fn unwrap_key(&self, source: &[u8; KEY_SOURCE_LEN]) -> Result<[u8; BLOWFISH_KEY_LEN]> {
        let a = self.plain_block_len()?;
        let block = a + 1;
        // Enough whole blocks to yield at least 56 plain bytes.
        let used = (55 / a + 1) * block;
        if used > KEY_SOURCE_LEN {
            return Err(Error::Unsupported(
                "RSA modulus too small for MIX key sources",
            ));
        }
        let mut plain = Vec::with_capacity(used);
        for chunk in source[..used].chunks_exact(block) {
            let c = BigUint::from_bytes_le(chunk);
            let m = c.modpow(&self.exponent, &self.modulus).to_bytes_le();
            let mut out = vec![0u8; a];
            let n = m.len().min(a);
            out[..n].copy_from_slice(&m[..n]);
            plain.extend_from_slice(&out);
        }
        let mut key = [0u8; BLOWFISH_KEY_LEN];
        key.copy_from_slice(&plain[..BLOWFISH_KEY_LEN]);
        Ok(key)
    }
}

/// Parses `[2, len, big-endian bytes...]` (short or long length form).
fn parse_der_integer(der: &[u8]) -> Option<BigUint> {
    let (&tag, rest) = der.split_first()?;
    if tag != 2 {
        return None;
    }
    let (&len_byte, rest) = rest.split_first()?;
    let (len, rest) = if len_byte & 0x80 != 0 {
        let n = usize::from(len_byte & 0x7F);
        let (len_bytes, rest) = rest.split_at_checked(n)?;
        let len = len_bytes.iter().try_fold(0usize, |acc, &b| {
            acc.checked_mul(256)?.checked_add(usize::from(b))
        })?;
        (len, rest)
    } else {
        (usize::from(len_byte), rest)
    };
    Some(BigUint::from_bytes_be(rest.get(..len)?))
}

/// An RSA key pair of our own, for writing synthetic encrypted archives.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RsaKeyPair {
    public: RsaPublicKey,
    private_exponent: BigUint,
}

/// Modulus of [`RsaKeyPair::synthetic`]: the product of two primes generated
/// for this project (seeded Miller-Rabin search), sized like the Westwood
/// modulus so the key-source framing is identical.
const SYNTHETIC_MODULUS_HEX: &str =
    "54b531fef9773c6105ab17498c73fb5a1fd0d20b6f4180b2803e5dc7507390f5b4f5e770d6b1a6f5";
/// Private exponent matching [`SYNTHETIC_MODULUS_HEX`] and exponent 65537.
const SYNTHETIC_PRIVATE_HEX: &str =
    "2059b7075eb4c7ed71e1be9a0b36fc46962205257afca2550e5cf0f46cfb9542f7c2397054673741";

impl RsaKeyPair {
    /// The project's synthetic key pair. Not secret and not Westwood's.
    pub fn synthetic() -> Self {
        let hex = |s: &str| BigUint::parse_bytes(s.as_bytes(), 16).unwrap_or_default();
        Self {
            public: RsaPublicKey {
                modulus: hex(SYNTHETIC_MODULUS_HEX),
                exponent: BigUint::from(PUBLIC_EXPONENT),
            },
            private_exponent: hex(SYNTHETIC_PRIVATE_HEX),
        }
    }

    /// The public half, for parsing what [`Self::wrap_key`] produced.
    pub fn public(&self) -> &RsaPublicKey {
        &self.public
    }

    /// Builds a key source that [`RsaPublicKey::unwrap_key`] turns back into
    /// `key`. Unused plain bytes after the key are zero.
    pub fn wrap_key(&self, key: &[u8; BLOWFISH_KEY_LEN]) -> Result<[u8; KEY_SOURCE_LEN]> {
        let a = self.public.plain_block_len()?;
        let block = a + 1;
        let blocks = 55 / a + 1;
        if blocks * block > KEY_SOURCE_LEN {
            return Err(Error::Unsupported(
                "RSA modulus too small for MIX key sources",
            ));
        }
        let mut plain = vec![0u8; blocks * a];
        plain[..BLOWFISH_KEY_LEN].copy_from_slice(key);
        let mut source = [0u8; KEY_SOURCE_LEN];
        for (i, chunk) in plain.chunks_exact(a).enumerate() {
            let m = BigUint::from_bytes_le(chunk);
            let c = m
                .modpow(&self.private_exponent, &self.public.modulus)
                .to_bytes_le();
            let dst = &mut source[i * block..(i + 1) * block];
            let n = c.len().min(block);
            dst[..n].copy_from_slice(&c[..n]);
        }
        Ok(source)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::synth::SynthRng;

    #[test]
    fn westwood_key_has_the_published_shape() {
        let k = RsaPublicKey::westwood();
        // 40-byte modulus with a 319-bit value, as decoded from the published string.
        assert_eq!(k.bits(), 319);
        assert_eq!(k.plain_block_len(), Ok(39));
    }

    #[test]
    fn synthetic_key_matches_westwood_framing() {
        let pair = RsaKeyPair::synthetic();
        assert_eq!(pair.public().bits(), RsaPublicKey::westwood().bits());
        assert_ne!(pair.public(), &RsaPublicKey::westwood());
    }

    #[test]
    fn wrap_then_unwrap_returns_the_key() {
        let pair = RsaKeyPair::synthetic();
        for seed in 0..8 {
            let mut key = [0u8; BLOWFISH_KEY_LEN];
            SynthRng::new(seed).fill(&mut key);
            let source = pair.wrap_key(&key).unwrap();
            assert_eq!(pair.public().unwrap_key(&source), Ok(key));
        }
    }

    #[test]
    fn der_lengths() {
        assert_eq!(parse_der_integer(&[2, 1, 5]), Some(BigUint::from(5u32)));
        assert_eq!(
            parse_der_integer(&[2, 0x81, 1, 7]),
            Some(BigUint::from(7u32))
        );
        assert_eq!(parse_der_integer(&[3, 1, 5]), None);
        assert_eq!(parse_der_integer(&[2, 4, 5]), None);
    }
}
