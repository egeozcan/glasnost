//! Name tables: the name hash and the hash-order rule (S-0002).
//!
//! The original identifies names by a 32-bit hash and finds a name by a
//! bisection over the table put in the order of its own sort. When a name
//! occurs more than once, the occurrence that bisection reaches wins. This
//! module reproduces the winner exactly; it sorts only tables in which some
//! hash repeats, as S-0002 Formulas allow.

use std::collections::BTreeMap;

use glasnost_fmt::mix::crc32;

use crate::Quirks;

// spec: S-0002 Constants (hash group size: names are padded to a multiple of 4 bytes)
const GROUP: usize = 4;
// spec: S-0002 Constants (selection-step limit)
const SELECTION_LIMIT: usize = 8;

/// The name hash of S-0002 Formulas, read as a signed 32-bit integer.
///
/// The standard reflected CRC-32 over the name padded to a multiple of four
/// bytes: one byte holding `len mod 4`, then copies of the first byte of the
/// last incomplete group. Bytes are hashed as written (no case folding).
pub fn name_hash(name: &[u8]) -> i32 {
    let r = name.len() % GROUP;
    // spec: S-0002 Formulas (name hash, padded copy)
    let crc = if r == 0 {
        crc32(name)
    } else {
        let mut padded = Vec::with_capacity(name.len() + GROUP - r);
        padded.extend_from_slice(name);
        // `r` is 1..=3.
        padded.push(r as u8);
        let fill = name[name.len() - r];
        padded.extend(std::iter::repeat_n(fill, GROUP - 1 - r));
        crc32(&padded)
    };
    // spec: S-0002 Formulas (compared as signed two's complement)
    crc as i32
}

/// The reference sort of S-0002 Formulas over `x`, comparing `h` only.
/// Iterative, so hostile input cannot exhaust the stack.
fn reference_sort<T: Copy>(x: &mut [T], h: impl Fn(&T) -> i32) {
    if x.len() < 2 {
        return;
    }
    let mut work = vec![(0usize, x.len() - 1)];
    while let Some((lo, hi)) = work.pop() {
        let s = hi - lo + 1;
        if s <= 1 {
            continue;
        }
        // spec: S-0002 Formulas (selection from the top for 2 <= s <= 8)
        if s <= SELECTION_LIMIT {
            for top in (lo + 1..=hi).rev() {
                let mut j = lo;
                for i in lo + 1..=top {
                    if h(&x[i]) > h(&x[j]) {
                        j = i;
                    }
                }
                x.swap(j, top);
            }
            continue;
        }
        // spec: S-0002 Formulas (partition for s > 8)
        x.swap(lo + s / 2, lo);
        let v = h(&x[lo]);
        let (mut u, mut d) = (lo, hi + 1);
        loop {
            let u_next = (u + 1..=hi).find(|&i| h(&x[i]) > v).unwrap_or(hi + 1);
            let d_next = (lo + 1..d).rev().find(|&i| h(&x[i]) < v).unwrap_or(lo);
            (u, d) = (u_next, d_next);
            if d >= u {
                x.swap(u, d);
            } else {
                break;
            }
        }
        x.swap(lo, d);
        if d > lo + 1 {
            work.push((lo, d - 1));
        }
        if u < hi {
            work.push((u, hi));
        }
    }
}

/// The bisection *F* of S-0002 Formulas: the first probed position whose
/// hash equals `t`.
fn bisect<T>(x: &[T], h: impl Fn(&T) -> i32, t: i32) -> Option<usize> {
    let (mut b, mut m) = (0usize, x.len());
    while m > 0 {
        // spec: S-0002 Formulas (bisection: probe q = b + floor(m / 2))
        let q = b + m / 2;
        let hq = h(&x[q]);
        if hq == t {
            return Some(q);
        }
        if hq < t {
            b = q + 1;
            m = m - m / 2 - 1;
        } else {
            m /= 2;
        }
    }
    None
}

/// The winning occurrence among `occurrences` (positions in table order,
/// with their hashes) for a lookup of hash `t` (S-0002 Formulas).
fn hash_order_winner(occurrences: &[(i32, u32)], t: i32) -> Option<u32> {
    let mut x = occurrences.to_vec();
    reference_sort(&mut x, |e| e.0);
    bisect(&x, |e| e.0, t).map(|q| x[q].1)
}

/// One name table: the keys of a section, or the section names of a
/// document, in table order.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Table {
    /// Hash of each occurrence, in table order.
    hashes: Vec<i32>,
    /// Occurrence positions by hash, ascending.
    by_hash: BTreeMap<i32, Vec<u32>>,
    /// Winner of the hash-order rule for every hash that occurs more than
    /// once.
    winners: BTreeMap<i32, u32>,
}

impl Table {
    /// Builds the table for `names` in table order.
    pub fn new<'a>(names: impl IntoIterator<Item = &'a [u8]>) -> Self {
        let hashes: Vec<i32> = names.into_iter().map(name_hash).collect();
        let mut by_hash: BTreeMap<i32, Vec<u32>> = BTreeMap::new();
        for (i, &h) in hashes.iter().enumerate() {
            by_hash
                .entry(h)
                .or_default()
                .push(u32::try_from(i).unwrap_or(u32::MAX));
        }
        let mut winners = BTreeMap::new();
        if by_hash.values().any(|v| v.len() > 1) {
            let all: Vec<(i32, u32)> = hashes
                .iter()
                .enumerate()
                .map(|(i, &h)| (h, u32::try_from(i).unwrap_or(u32::MAX)))
                .collect();
            let mut x = all;
            reference_sort(&mut x, |e| e.0);
            for (&h, occ) in &by_hash {
                if occ.len() > 1
                    && let Some(q) = bisect(&x, |e| e.0, h)
                {
                    winners.insert(h, x[q].1);
                }
            }
        }
        Self {
            hashes,
            by_hash,
            winners,
        }
    }

    /// Whether some hash occurs more than once.
    pub fn has_repeated_hash(&self) -> bool {
        !self.winners.is_empty()
    }

    /// Positions sharing each repeated hash, by hash.
    pub fn repeated(&self) -> impl Iterator<Item = (i32, &[u32])> {
        self.by_hash
            .iter()
            .filter(|(_, v)| v.len() > 1)
            .map(|(&h, v)| (h, v.as_slice()))
    }

    /// The occurrence a lookup of `name` returns, as a position in table
    /// order, under the two quirk switches (S-0002 Edge cases, the table of
    /// how they combine). `name_at` gives the bytes of an occurrence.
    pub fn lookup<'n>(
        &self,
        name: &[u8],
        quirks: &Quirks,
        name_at: impl Fn(u32) -> &'n [u8],
    ) -> Option<u32> {
        let h = name_hash(name);
        let same_hash = self.by_hash.get(&h)?;
        if quirks.ini_hash_identity {
            // spec: S-0002 claim 3 (names are identified by hash alone)
            if !quirks.ini_duplicate_hash_order {
                // spec: S-0002 Formulas (switch off: the last occurrence wins)
                return same_hash.last().copied();
            }
            return match self.winners.get(&h) {
                Some(&w) => Some(w),
                None => same_hash.first().copied(),
            };
        }
        // Hash identity off: only same-name occurrences count.
        let same_name: Vec<u32> = same_hash
            .iter()
            .copied()
            .filter(|&i| name_at(i) == name)
            .collect();
        let last = *same_name.last()?;
        if !quirks.ini_duplicate_hash_order {
            return Some(last);
        }
        if same_name.len() == same_hash.len() {
            // No other name shares the hash: the full table's winner.
            return Some(self.winners.get(&h).copied().unwrap_or(last));
        }
        // spec: S-0002 Edge cases (hash order on, identity off: remove
        // same-hash occurrences of different names, then apply the rule)
        let kept: Vec<(i32, u32)> = self
            .hashes
            .iter()
            .enumerate()
            .map(|(i, &hh)| (hh, u32::try_from(i).unwrap_or(u32::MAX)))
            .filter(|&(hh, i)| hh != h || name_at(i) == name)
            .collect();
        hash_order_winner(&kept, h)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// spec: S-0002 Formulas (CRC check value and the stated hashes)
    #[test]
    fn hash_vectors() {
        assert_eq!(crc32(b"123456789"), 3_421_780_262);
        assert_eq!(name_hash(b"Cost"), -1_206_633_534);
        assert_eq!(name_hash(b"ZdxeLZgd"), -1_206_633_534);
        assert_eq!(name_hash(b"Filler0"), -1_429_695_458);
        assert_eq!(name_hash(b""), 0);
        // "A" is hashed as "A", 1, "A", "A".
        assert_eq!(name_hash(b"A"), crc32(b"A\x01AA") as i32);
    }

    fn table(names: &[&str]) -> Table {
        Table::new(names.iter().map(|n| n.as_bytes()))
    }

    fn look(names: &[&str], q: &Quirks, name: &str) -> Option<u32> {
        table(names).lookup(name.as_bytes(), q, |i| names[i as usize].as_bytes())
    }

    /// spec: S-0002 Formulas (worked examples, fixtures 1 and 2; Import mapping rows D-c, D-d)
    #[test]
    fn fixtures_one_and_two() {
        let q = Quirks::default();
        assert_eq!(look(&["Cost", "Cost"], &q, "Cost"), Some(0));
        assert_eq!(look(&["Cost", "Cost", "Filler0"], &q, "Cost"), Some(1));
    }

    /// spec: S-0002 claim 3, Import mapping row D-e (fixture 3)
    #[test]
    fn fixture_three_collision() {
        let names = ["ZdxeLZgd", "Cost"];
        let on = Quirks::default();
        assert_eq!(look(&names, &on, "Cost"), Some(0));
        let off = Quirks {
            ini_hash_identity: false,
            ..Quirks::default()
        };
        assert_eq!(look(&names, &off, "Cost"), Some(1));
    }

    /// spec: S-0002 Edge cases (the four combinations of the two switches)
    #[test]
    fn switch_matrix() {
        // Two Cost occurrences, a colliding name, and a smaller filler.
        let names = ["Cost", "ZdxeLZgd", "Cost", "Filler0"];
        let q = |order, identity| Quirks {
            ini_duplicate_hash_order: order,
            ini_hash_identity: identity,
            ..Quirks::default()
        };
        // on/on: hash order over all three same-hash occurrences.
        let all = [
            (name_hash(b"Cost"), 0),
            (name_hash(b"ZdxeLZgd"), 1),
            (name_hash(b"Cost"), 2),
            (name_hash(b"Filler0"), 3),
        ];
        assert_eq!(
            look(&names, &q(true, true), "Cost"),
            hash_order_winner(&all, name_hash(b"Cost"))
        );
        // on/off: the colliding name removed first.
        let kept = [all[0], all[2], all[3]];
        assert_eq!(
            look(&names, &q(true, false), "Cost"),
            hash_order_winner(&kept, name_hash(b"Cost"))
        );
        // off/on: the last same-hash occurrence.
        assert_eq!(look(&names, &q(false, true), "Cost"), Some(2));
        assert_eq!(look(&names, &q(false, true), "ZdxeLZgd"), Some(2));
        // off/off: the last same-name occurrence.
        assert_eq!(look(&names, &q(false, false), "ZdxeLZgd"), Some(1));
        assert_eq!(look(&names, &q(false, false), "Cost"), Some(2));
        // Not found in every row.
        for (o, i) in [(true, true), (true, false), (false, true), (false, false)] {
            assert_eq!(look(&names, &q(o, i), "Nope"), None);
        }
    }

    /// The sort's result is a sorted permutation, also for large tables
    /// that take the partition step.
    #[test]
    fn reference_sort_sorts() {
        let mut state = 12345u32;
        let mut x: Vec<(i32, u32)> = (0..500)
            .map(|i| {
                state = state.wrapping_mul(1_103_515_245).wrapping_add(12345);
                ((state >> 20) as i32 % 37 - 18, i)
            })
            .collect();
        reference_sort(&mut x, |e| e.0);
        assert!(x.windows(2).all(|w| w[0].0 <= w[1].0));
        let mut ids: Vec<u32> = x.iter().map(|e| e.1).collect();
        ids.sort_unstable();
        assert_eq!(ids, (0..500).collect::<Vec<_>>());
    }
}
