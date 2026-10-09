//! Finding the maps of a mounted install: archive entries whose text holds
//! `[Map]` and `[IsoMapPack5]` (archives keep only name hashes, so maps are
//! found by content).

use glasnost_fmt::sniff::{Kind, SNIFF_LEN, sniff};
use glasnost_ini::Document;
use glasnost_vfs::{ArchiveId, FileRef, Vfs};

/// Largest text entry looked at.
const MAX_MAP: u32 = 16 << 20;

/// One map found in the install.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Listing {
    /// Where it is.
    pub file: FileRef,
    /// The archive holding it.
    pub archive: String,
    /// Its map-list name (S-0009 Import mapping, `[Basic] Name`).
    pub name: String,
    /// Its theater value as written.
    pub theater: String,
    /// Whether start waypoints 0 and 1 are defined (a multiplayer map).
    pub multiplayer: bool,
}

/// Every map in `vfs`, in archive and entry order. Nested archives are not
/// themselves read as maps.
pub async fn find_maps(vfs: &Vfs) -> Vec<Listing> {
    let nested: std::collections::BTreeSet<(ArchiveId, u32)> =
        vfs.archives().iter().filter_map(|a| a.parent).collect();
    let mut out = Vec::new();
    for (i, archive) in vfs.archives().iter().enumerate() {
        let aid = ArchiveId(i as u32);
        for entry in archive.index.entries.iter().copied() {
            if nested.contains(&(aid, entry.id)) || entry.size > MAX_MAP {
                continue;
            }
            let file = FileRef {
                archive: aid,
                entry,
            };
            let Ok(prefix) = vfs.read_prefix(&file, SNIFF_LEN as u64).await else {
                continue;
            };
            if sniff(&prefix, u64::from(entry.size)) != Kind::Text {
                continue;
            }
            let Ok(bytes) = vfs.read(&file).await else {
                continue;
            };
            if !bytes
                .windows(13)
                .any(|w| w.eq_ignore_ascii_case(b"[IsoMapPack5]"))
            {
                continue;
            }
            let doc = Document::parse(&bytes, Default::default(), &mut Vec::new());
            if !glasnost_convert::map::is_map(&doc) {
                continue;
            }
            let text = |s: &[u8], k: &[u8]| {
                doc.value(s, k)
                    .map(|v| String::from_utf8_lossy(v).into_owned())
                    .unwrap_or_default()
            };
            let wp = |k: &[u8]| doc.value(b"Waypoints", k).is_some();
            out.push(Listing {
                file,
                archive: archive.name.clone(),
                name: text(b"Basic", b"Name"),
                theater: text(b"Map", b"Theater"),
                multiplayer: wp(b"0") && wp(b"1"),
            });
        }
    }
    out
}
