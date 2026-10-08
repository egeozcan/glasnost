//! Entry points that drive every parser and decoder over arbitrary bytes,
//! shared by the cargo-fuzz targets (`fuzz/`) and the stable hostile-input
//! smoke test (`tests/hostile.rs`).
//!
//! [`run`] ignores every error: the only failure is a panic, an overflow
//! trap, a hang or unbounded memory, which the fuzzer and the test catch.

use crate::mix::{MixArchive, MixKeys};
use crate::mix_key::RsaKeyPair;

/// Bound on decompressed output while fuzzing.
const MAX_OUT: usize = 1 << 20;

/// What to feed the bytes to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Target {
    /// MIX index (Westwood and synthetic keys) and every entry read.
    Mix,
    /// SHP-TS headers and every frame.
    Shp,
    /// TMP-TS headers, every tile and the template render.
    Tmp,
    /// VXL.
    Vxl,
    /// HVA.
    Hva,
    /// PAL.
    Pal,
    /// PCX.
    Pcx,
    /// CSF.
    Csf,
    /// IDX, with the same bytes as its BAG.
    Idx,
    /// AUD.
    Aud,
    /// LZO1X decompression.
    Lzo,
    /// LCW decompression, both position modes.
    Lcw,
    /// Map-pack base64, block framing and tile records.
    MapPack,
    /// Content sniffing.
    Sniff,
}

/// Every target, in a fixed order.
pub const ALL: &[Target] = &[
    Target::Mix,
    Target::Shp,
    Target::Tmp,
    Target::Vxl,
    Target::Hva,
    Target::Pal,
    Target::Pcx,
    Target::Csf,
    Target::Idx,
    Target::Aud,
    Target::Lzo,
    Target::Lcw,
    Target::MapPack,
    Target::Sniff,
];

/// Runs `target` over `data`, discarding results.
pub fn run(target: Target, data: &[u8]) {
    match target {
        Target::Mix => {
            let synthetic = MixKeys {
                rsa: RsaKeyPair::synthetic().public().clone(),
                ..MixKeys::default()
            };
            for keys in [MixKeys::default(), synthetic] {
                if let Ok(a) = MixArchive::new(data, &keys) {
                    let _ = a.index.check_consistent(data.len() as u64);
                    let _ = a.index.has_duplicate_ids();
                    for e in &a.index.entries {
                        let _ = a.read(e);
                    }
                }
            }
            let _ = crate::mix::may_be_mix(data, data.len() as u64);
        }
        Target::Shp => {
            if let Ok(s) = crate::shp::Shp::parse(data) {
                for i in 0..s.frames.len() {
                    let _ = s.decode_frame_stats(i);
                    let _ = s.decode_on_canvas(i);
                }
            }
        }
        Target::Tmp => {
            if let Ok(t) = crate::tmp::Tmp::parse(data) {
                for i in 0..t.tiles.len() {
                    let _ = t.decode_tile(i);
                }
                let _ = t.render();
            }
        }
        Target::Vxl => {
            if let Ok(v) = crate::vxl::Vxl::parse(data) {
                let _ = crate::vxl::VxlWriter { model: &v }.write();
            }
        }
        Target::Hva => {
            if let Ok(h) = crate::hva::Hva::parse(data) {
                let _ = h.matrix(0, 0);
                let _ = h.write();
            }
        }
        Target::Pal => {
            if let Ok(p) = crate::pal::Palette6::parse(data) {
                let _ = p.to_palette(crate::pal::Expansion::Replicate);
            }
        }
        Target::Pcx => {
            let _ = crate::pcx::Pcx::parse(data);
        }
        Target::Csf => {
            if let Ok(c) = crate::csf::Csf::parse(data) {
                for l in &c.labels {
                    for s in &l.strings {
                        let _ = s.text();
                    }
                }
            }
        }
        Target::Idx => {
            if let Ok(idx) = crate::idx::Idx::parse(data) {
                for e in &idx.entries {
                    if let Ok(pcm) = e.decode(data) {
                        let _ = pcm.to_wav();
                    }
                }
            }
        }
        Target::Aud => {
            if let Ok(a) = crate::aud::Aud::parse(data) {
                let _ = a.decode();
            }
        }
        Target::Lzo => {
            let _ = crate::lzo::decompress(data, MAX_OUT);
            let _ = crate::lzo::decompress_prefix(data, MAX_OUT);
        }
        Target::Lcw => {
            for positions in [
                crate::lcw::Positions::Absolute,
                crate::lcw::Positions::Relative,
            ] {
                let _ = crate::lcw::decompress_with(data, positions, MAX_OUT);
            }
        }
        Target::MapPack => {
            use crate::mappack::{PackCodec, decode_base64, iso_tiles, unpack};
            for codec in [PackCodec::Lzo, PackCodec::Lcw] {
                if let Ok(raw) = unpack(data, codec, MAX_OUT) {
                    let _ = iso_tiles(&raw);
                }
            }
            let text = String::from_utf8_lossy(data);
            if let Ok(bytes) = decode_base64(text.lines()) {
                let _ = unpack(&bytes, PackCodec::Lzo, MAX_OUT);
            }
            let _ = iso_tiles(data);
        }
        Target::Sniff => {
            let _ = crate::sniff::sniff(data, data.len() as u64);
            let _ = crate::sniff::sniff(data, u64::MAX);
        }
    }
}
