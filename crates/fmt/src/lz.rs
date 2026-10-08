//! Helpers shared by the LZ-family codecs ([`crate::lzo`], [`crate::lcw`]):
//! bounded output appends, back-reference copies and a small hash-chain
//! match finder for the encoders.
//!
//! The decoders call only the `push_*`/`copy_*` functions, which check every
//! source position and the output cap, so a hostile stream can neither read
//! outside the output written so far nor grow it past the caller's limit.

use crate::error::{Error, Result};
use crate::synth::SynthRng;

const LIMIT: Error = Error::TooLarge("decompressed output exceeds the limit");

/// Checks that `len` more bytes fit under `cap`.
fn reserve(out: &mut Vec<u8>, len: usize, cap: usize) -> Result<()> {
    let end = out.len().checked_add(len).ok_or(LIMIT)?;
    if end > cap {
        return Err(LIMIT);
    }
    out.reserve(len);
    Ok(())
}

/// Appends literal bytes.
pub(crate) fn push_literals(out: &mut Vec<u8>, lits: &[u8], cap: usize) -> Result<()> {
    reserve(out, lits.len(), cap)?;
    out.extend_from_slice(lits);
    Ok(())
}

/// Appends `len` copies of `byte`.
pub(crate) fn push_fill(out: &mut Vec<u8>, byte: u8, len: usize, cap: usize) -> Result<()> {
    reserve(out, len, cap)?;
    out.resize(out.len() + len, byte);
    Ok(())
}

/// Appends `len` bytes copied from output position `src`, byte by byte, so
/// the source may overlap the bytes being written (a short repeating
/// pattern). `src` must lie before the current output position.
pub(crate) fn copy_from(out: &mut Vec<u8>, src: usize, len: usize, cap: usize) -> Result<()> {
    if src >= out.len() {
        return Err(Error::Invalid(
            "back-reference at or past the current output position",
        ));
    }
    reserve(out, len, cap)?;
    match src.checked_add(len) {
        Some(end) if end <= out.len() => out.extend_from_within(src..end),
        _ => {
            // Overlapping: each byte read was either present before the copy
            // or written earlier in this loop, so `i < out.len()` holds.
            for i in src..src + len {
                let b = out[i];
                out.push(b);
            }
        }
    }
    Ok(())
}

/// Appends `len` bytes copied from `dist` bytes back (`dist >= 1`).
pub(crate) fn copy_back(out: &mut Vec<u8>, dist: usize, len: usize, cap: usize) -> Result<()> {
    if dist == 0 || dist > out.len() {
        return Err(Error::Invalid(
            "back-reference before the start of the output",
        ));
    }
    copy_from(out, out.len() - dist, len, cap)
}

const HASH_BITS: u32 = 14;

fn hash3(data: &[u8], pos: usize) -> Option<usize> {
    let b = data.get(pos..pos.checked_add(3)?)?;
    let v = u32::from(b[0]) | u32::from(b[1]) << 8 | u32::from(b[2]) << 16;
    Some((v.wrapping_mul(0x9E37_79B1) >> (32 - HASH_BITS)) as usize)
}

/// Hash chains over 3-byte prefixes, for greedy encoders. Deterministic:
/// candidates are visited newest first, ties keep the nearest.
pub(crate) struct MatchFinder {
    /// Newest position + 1 per hash bucket (0 = empty).
    head: Vec<usize>,
    /// Previous position + 1 in the same bucket, per position.
    prev: Vec<usize>,
    depth: usize,
}

impl MatchFinder {
    /// A finder for an input of `len` bytes that walks at most `depth`
    /// candidates per lookup.
    pub(crate) fn new(len: usize, depth: usize) -> Self {
        Self {
            head: vec![0; 1 << HASH_BITS],
            prev: vec![0; len],
            depth,
        }
    }

    /// Records `pos` as a match candidate for later positions.
    pub(crate) fn insert(&mut self, data: &[u8], pos: usize) {
        if let Some(h) = hash3(data, pos) {
            self.prev[pos] = self.head[h];
            self.head[h] = pos + 1;
        }
    }

    /// The longest earlier match for `data[pos..]` as `(distance, length)`,
    /// with distance at most `max_dist`, length at most `max_len` and at
    /// least 3, among candidates whose distance passes `ok`.
    pub(crate) fn find(
        &self,
        data: &[u8],
        pos: usize,
        max_dist: usize,
        max_len: usize,
        ok: impl Fn(usize) -> bool,
    ) -> Option<(usize, usize)> {
        let h = hash3(data, pos)?;
        let limit = max_len.min(data.len() - pos);
        let mut best: Option<(usize, usize)> = None;
        let mut cand = self.head[h];
        let mut steps = 0;
        while cand != 0 && steps < self.depth {
            let c = cand - 1;
            let dist = pos - c;
            if dist > max_dist {
                break;
            }
            if ok(dist) {
                let len = data[c..]
                    .iter()
                    .zip(&data[pos..])
                    .take(limit)
                    .take_while(|(a, b)| a == b)
                    .count();
                if len >= 3 && best.is_none_or(|(_, l)| len > l) {
                    best = Some((dist, len));
                }
            }
            cand = self.prev[c];
            steps += 1;
        }
        best
    }
}

/// Invented test content for the codecs: a deterministic mix of random
/// literals, single-byte runs, ramps and repeats of earlier content at
/// distances from 1 up to tens of kilobytes, with lengths beyond 255.
pub(crate) fn sample_data(seed: u64, len: usize) -> Vec<u8> {
    let mut rng = SynthRng::new(seed);
    let mut out = Vec::with_capacity(len);
    while out.len() < len {
        let r = rng.next_u64();
        let n = 3 + (r >> 8) as usize % 300;
        match r % 5 {
            0 => {
                let mut lit = vec![0u8; 1 + (r >> 20) as usize % 40];
                rng.fill(&mut lit);
                out.extend_from_slice(&lit);
            }
            1 => out.resize(out.len() + n, (r >> 32) as u8),
            2 => {
                let step = (r >> 40) as u8 | 1;
                let mut v = (r >> 48) as u8;
                for _ in 0..n {
                    out.push(v);
                    v = v.wrapping_add(step);
                }
            }
            _ if out.is_empty() => out.push((r >> 32) as u8),
            _ => {
                // Favour short distances but reach every LZO distance class.
                let span = match (r >> 32) % 4 {
                    0 => 16,
                    1 => 2048,
                    2 => 16384,
                    _ => 50000,
                };
                let dist = 1 + (r >> 36) as usize % span.min(out.len());
                let start = out.len() - dist;
                for i in 0..n {
                    out.push(out[start + i]);
                }
            }
        }
    }
    out.truncate(len);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn copies_check_bounds_and_cap() {
        let mut out = b"ab".to_vec();
        copy_back(&mut out, 2, 5, 100).unwrap();
        assert_eq!(out, b"abababa");
        assert!(copy_back(&mut out, 8, 1, 100).is_err());
        assert!(copy_back(&mut out, 0, 1, 100).is_err());
        assert!(copy_from(&mut out, 7, 1, 100).is_err());
        assert_eq!(copy_back(&mut out, 1, 4, 10), Err(LIMIT));
        assert_eq!(push_fill(&mut out, 0, usize::MAX, usize::MAX), Err(LIMIT));
        assert_eq!(out, b"abababa");
    }

    #[test]
    fn finder_prefers_longest() {
        let data = b"abcdXabcdeYabcdeZ";
        let mut mf = MatchFinder::new(data.len(), 8);
        for p in 0..11 {
            mf.insert(data, p);
        }
        assert_eq!(mf.find(data, 11, 100, 100, |_| true), Some((6, 5)));
        assert_eq!(mf.find(data, 11, 100, 4, |_| true), Some((6, 4)));
        assert_eq!(mf.find(data, 11, 100, 100, |d| d > 6), Some((11, 4)));
    }

    #[test]
    fn sample_data_is_deterministic() {
        assert_eq!(sample_data(3, 5000), sample_data(3, 5000));
        assert_ne!(sample_data(3, 5000), sample_data(4, 5000));
        assert_eq!(sample_data(3, 777).len(), 777);
    }
}
