//! Name tables: the name hash, the hash-order rule and the name-table model
//! of layered loads (S-0002).
//!
//! The original identifies names by a 32-bit hash and finds a name by a
//! bisection over the table put in the order of its own sort. When a name
//! occurs more than once, the occurrence that bisection reaches wins. A
//! layered load changes tables after lookups have begun, and each table
//! then also has a working order, an ordered flag, a remembered occurrence
//! and a peak count (S-0002 State and ordering, *Name tables during and
//! after a layered load*). This module reproduces the winner exactly; it
//! sorts only tables in which some name repeats, as S-0002 Formulas allow.

use std::collections::{BTreeMap, BTreeSet};

use glasnost_fmt::mix::crc32;

use crate::Quirks;

// spec: S-0002 Constants (hash group size: names are padded to a multiple of 4 bytes)
const GROUP: usize = 4;
// spec: S-0002 Constants (selection-step limit)
const SELECTION_LIMIT: usize = 8;
// spec: S-0002 Constants (forgetting peaks: 0, then 10, then each value twice the one before)
const FIRST_NONZERO_FORGETTING_PEAK: usize = 10;

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

/// Whether `count` occurrences is a forgetting peak (S-0002 Constants).
fn forgetting_peak(count: usize) -> bool {
    // spec: S-0002 Constants (forgetting peaks 0, 10, 20, 40, 80, ...)
    count == 0
        || (count.is_multiple_of(FIRST_NONZERO_FORGETTING_PEAK)
            && (count / FIRST_NONZERO_FORGETTING_PEAK).is_power_of_two())
}

/// What a table compares: the name hash, and with hash identity off the
/// name as well (S-0002 Edge cases, second row of the switch table: pairs
/// order by hash, then by name as unsigned bytes, a prefix first).
type Key = (i32, Option<Vec<u8>>);

/// The reference sort of S-0002 Formulas over `x`, comparing `key` only.
/// Iterative, so hostile input cannot exhaust the stack.
fn reference_sort<'k, T, K: Ord + Clone + 'k>(x: &mut [T], key: impl Fn(&T) -> &'k K) {
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
                    if key(&x[i]) > key(&x[j]) {
                        j = i;
                    }
                }
                x.swap(j, top);
            }
            continue;
        }
        // spec: S-0002 Formulas (partition for s > 8)
        x.swap(lo + s / 2, lo);
        let v = key(&x[lo]).clone();
        let (mut u, mut d) = (lo, hi + 1);
        loop {
            let u_next = (u + 1..=hi).find(|&i| *key(&x[i]) > v).unwrap_or(hi + 1);
            let d_next = (lo + 1..d).rev().find(|&i| *key(&x[i]) < v).unwrap_or(lo);
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
/// key equals `t`.
fn bisect<'k, T, K: Ord + 'k>(x: &[T], key: impl Fn(&T) -> &'k K, t: &K) -> Option<usize> {
    let (mut b, mut m) = (0usize, x.len());
    while m > 0 {
        // spec: S-0002 Formulas (bisection: probe q = b + floor(m / 2))
        let q = b + m / 2;
        let kq = key(&x[q]);
        if kq == t {
            return Some(q);
        }
        if kq < t {
            b = q + 1;
            m = m - m / 2 - 1;
        } else {
            m /= 2;
        }
    }
    None
}

/// One name table: the keys of a section, or the section names of a
/// document.
///
/// Occurrences are numbered by their position in table order (entry order,
/// or file order of the sections). A layered load can discard occurrences;
/// their numbers stay valid until [`Table::compact`] renumbers the rest.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Table {
    /// `quirks.ini_duplicate_hash_order` (S-0002 claims 1 and 2).
    order: bool,
    /// `quirks.ini_hash_identity` (S-0002 claim 3).
    identity: bool,
    /// Key of each occurrence, by number (discarded ones included).
    keys: Vec<Key>,
    /// Numbers of the occurrences still in the table, by key, each list in
    /// table order.
    by_key: BTreeMap<Key, Vec<u32>>,
    /// Whether some key has more than one occurrence. A layered load never
    /// adds a repeat (it replaces the occurrence a lookup finds), so only a
    /// fresh load sets this.
    repeats: bool,
    /// The working order (S-0002 Terms).
    working: Vec<u32>,
    /// Whether the working order is in reference-sort order (S-0002 Terms).
    ordered: bool,
    /// The remembered occurrence (S-0002 Terms).
    remembered: Option<u32>,
    /// The peak count (S-0002 Terms).
    peak: usize,
    /// For every repeated key, the occurrence a lookup returns in the state
    /// the table is in, when the remembered occurrence does not answer.
    winners: BTreeMap<Key, u32>,
}

impl Table {
    /// The table a fresh load of `names` (in table order) leaves: file
    /// order as its working order, not ordered, nothing remembered, and
    /// its number of occurrences as its peak count (S-0002 State and
    /// ordering).
    pub fn new<'a>(names: impl IntoIterator<Item = &'a [u8]>, quirks: &Quirks) -> Self {
        let mut t = Self {
            order: quirks.ini_duplicate_hash_order,
            identity: quirks.ini_hash_identity,
            keys: Vec::new(),
            by_key: BTreeMap::new(),
            repeats: false,
            working: Vec::new(),
            ordered: false,
            remembered: None,
            peak: 0,
            winners: BTreeMap::new(),
        };
        for name in names {
            t.push(name);
        }
        // spec: S-0002 State and ordering (a fresh load: peak count = number of occurrences)
        t.peak = t.working.len();
        t.settle();
        t
    }

    fn key(&self, name: &[u8]) -> Key {
        // spec: S-0002 claim 3 (names are identified by hash alone); Edge cases (identity off: pairs)
        (name_hash(name), (!self.identity).then(|| name.to_vec()))
    }

    /// Appends an occurrence at the end of table order and of the working
    /// order, and returns its number.
    fn push(&mut self, name: &[u8]) -> u32 {
        let id = u32::try_from(self.keys.len()).unwrap_or(u32::MAX);
        let key = self.key(name);
        let same = self.by_key.entry(key.clone()).or_default();
        same.push(id);
        self.repeats |= same.len() > 1;
        self.keys.push(key);
        self.working.push(id);
        id
    }

    /// Whether two occurrences in the table share a name hash, whatever
    /// their names.
    pub fn has_repeated_hash(&self) -> bool {
        let mut previous = None;
        for (k, ids) in &self.by_key {
            if ids.len() > 1 || previous == Some(k.0) {
                return true;
            }
            previous = Some(k.0);
        }
        false
    }

    /// The occurrence a lookup of `name` returns, without changing the
    /// table, under the two quirk switches (S-0002 Edge cases, the table of
    /// how they combine).
    ///
    /// After a layered load this is the result of the first lookup made
    /// after it. The model lets a lookup's result depend on the lookups
    /// before it (S-0002 claim 11), but the original reads the layered
    /// document by single lookups whose values do not depend on their
    /// order, and claim 11 has this engine give the same values.
    pub fn lookup(&self, name: &[u8]) -> Option<u32> {
        let k = self.key(name);
        let same = self.by_key.get(&k)?;
        if !self.order {
            // spec: S-0002 Formulas (switch off: the last occurrence in the table's order wins)
            return same.last().copied();
        }
        // spec: S-0002 State and ordering (the remembered occurrence answers a lookup of its key)
        if let Some(r) = self.remembered
            && self.keys[r as usize] == k
        {
            return Some(r);
        }
        // spec: S-0002 State and ordering (a name that does not repeat: its one occurrence)
        if let [only] = same.as_slice() {
            return Some(*only);
        }
        self.winners.get(&k).copied()
    }

    /// A lookup that a line of a layered load makes (S-0002 claim 11),
    /// changing the table as the name-table model says.
    pub(crate) fn lookup_mut(&mut self, name: &[u8]) -> Option<u32> {
        // spec: S-0002 State and ordering (a table with no occurrences: nothing changes)
        if self.working.is_empty() {
            return None;
        }
        let k = self.key(name);
        if !self.order {
            // spec: S-0002 Formulas (switch off: the last occurrence in the table's order)
            return self.by_key.get(&k).and_then(|v| v.last().copied());
        }
        if !self.repeats {
            // spec: S-0002 State and ordering (two consequences: in a table without a repeated
            // name, every lookup returns the one occurrence of its name, whatever the model's
            // state). The table never gains a repeat, so its state is never observed.
            return self.by_key.get(&k).and_then(|v| v.first().copied());
        }
        // spec: S-0002 State and ordering (row 1: the remembered occurrence; nothing changes)
        if let Some(r) = self.remembered
            && self.keys[r as usize] == k
        {
            return Some(r);
        }
        let was_ordered = self.ordered;
        if !was_ordered {
            // spec: S-0002 State and ordering (rows 4 and 5: the working order becomes Sort(x))
            let keys = &self.keys;
            reference_sort(&mut self.working, |&id| &keys[id as usize]);
            self.ordered = true;
        }
        let keys = &self.keys;
        match bisect(&self.working, |&id| &keys[id as usize], &k) {
            Some(q) => {
                // spec: S-0002 State and ordering (rows 2 and 4: the found occurrence is remembered)
                let id = self.working[q];
                self.remembered = Some(id);
                Some(id)
            }
            None => {
                if !was_ordered {
                    // spec: S-0002 State and ordering (row 5: "nothing is remembered"). Read as
                    // forgetting the remembered occurrence, unlike row 3; request Q-0183.
                    self.remembered = None;
                }
                None
            }
        }
    }

    /// Adds an occurrence of `name` (S-0002 State and ordering, rows 6 and
    /// 7) and returns its number.
    pub(crate) fn add(&mut self, name: &[u8]) -> u32 {
        let count = self.working.len();
        // spec: S-0002 State and ordering (row 6: an addition at a peak count that is a forgetting peak forgets)
        if count == self.peak && forgetting_peak(count) {
            self.remembered = None;
        }
        let id = self.push(name);
        // spec: S-0002 State and ordering (rows 6 and 7: at the end of the working order, not ordered)
        self.ordered = false;
        self.peak = self.peak.max(count + 1);
        id
    }

    /// Discards occurrence `id`, which the line's lookup returned (S-0002
    /// State and ordering, row 8). The caller then adds the new entry.
    pub(crate) fn replace(&mut self, id: u32) {
        // spec: S-0002 State and ordering (row 8: leaves the working order, which stays ordered if it was)
        self.working.retain(|&x| x != id);
        if let Some(key) = self.keys.get(id as usize)
            && let Some(same) = self.by_key.get_mut(key)
        {
            same.retain(|&x| x != id);
            if same.is_empty() {
                self.by_key.remove(key);
            }
        }
        // spec: S-0002 State and ordering (row 8: the table forgets its remembered occurrence)
        self.remembered = None;
    }

    /// Forgets the remembered occurrence: the state every table starts a
    /// layered load in (S-0002 claim 12).
    pub(crate) fn forget(&mut self) {
        // spec: S-0002 claim 12 (every table starts the layered load with nothing remembered)
        self.remembered = None;
    }

    /// An occurrence still in the table whose name differs from `name` but
    /// shares its name hash. `name_at` gives the bytes of an occurrence.
    pub(crate) fn other_name_with_hash<'n>(
        &self,
        name: &[u8],
        name_at: impl Fn(u32) -> &'n [u8],
    ) -> Option<u32> {
        let h = name_hash(name);
        self.by_key
            .range((h, None)..)
            .take_while(|(k, _)| k.0 == h)
            .flat_map(|(_, ids)| ids.iter().copied())
            .find(|&id| name_at(id) != name)
    }

    /// Renumbers the occurrences after those in `discarded` left the table,
    /// keeping table order, and settles the table.
    pub(crate) fn compact(&mut self, discarded: &BTreeSet<u32>) {
        if !discarded.is_empty() {
            let mut renumber = vec![None; self.keys.len()];
            let mut keys = Vec::with_capacity(self.keys.len() - discarded.len());
            for (old, key) in std::mem::take(&mut self.keys).into_iter().enumerate() {
                let old = u32::try_from(old).unwrap_or(u32::MAX);
                if !discarded.contains(&old) {
                    renumber[old as usize] = u32::try_from(keys.len()).ok();
                    keys.push(key);
                }
            }
            let map = |id: u32| renumber.get(id as usize).copied().flatten();
            self.keys = keys;
            self.working = self.working.iter().filter_map(|&id| map(id)).collect();
            self.remembered = self.remembered.and_then(map);
            for ids in self.by_key.values_mut() {
                *ids = ids.iter().filter_map(|&id| map(id)).collect();
            }
        }
        self.settle();
    }

    /// Recomputes the winners that [`Table::lookup`] gives for repeated
    /// keys: the bisection over the working order, sorted first if the
    /// table is not ordered (S-0002 Formulas, State and ordering).
    pub(crate) fn settle(&mut self) {
        self.winners.clear();
        if !self.order || !self.repeats {
            return;
        }
        let keys = &self.keys;
        let mut view = self.working.clone();
        if !self.ordered {
            reference_sort(&mut view, |&id| &keys[id as usize]);
        }
        for (k, same) in &self.by_key {
            if same.len() > 1
                && let Some(q) = bisect(&view, |&id| &keys[id as usize], k)
            {
                self.winners.insert(k.clone(), view[q]);
            }
        }
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

    fn table(names: &[&str], q: &Quirks) -> Table {
        Table::new(names.iter().map(|n| n.as_bytes()), q)
    }

    fn look(names: &[&str], q: &Quirks, name: &str) -> Option<u32> {
        table(names, q).lookup(name.as_bytes())
    }

    /// A name made of `prefix` and a number whose hash satisfies `ok`.
    fn name_where(prefix: &str, ok: impl Fn(i32) -> bool) -> String {
        (0..10_000)
            .map(|i| format!("{prefix}{i}"))
            .find(|n| ok(name_hash(n.as_bytes())))
            .unwrap()
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
        // on/on: the hash-order rule over the hashes, Filler0's the
        // smaller. Selection over (C0, Z1, C2, F3): top 3 takes C0 (the
        // lowest of three equal hashes), giving (F3, Z1, C2, C0); top 2
        // takes Z1, giving (F3, C2, Z1, C0); top 1 leaves it. The bisection
        // probes position 2: occurrence 1, for either name.
        assert_eq!(look(&names, &q(true, true), "Cost"), Some(1));
        assert_eq!(look(&names, &q(true, true), "ZdxeLZgd"), Some(1));
        // on/off: pairs (h, name). Filler0 < (h, "Cost") < (h, "ZdxeLZgd").
        // Selection over (C0, Z1, C2, F3): top 3 swaps Z1 to the top,
        // giving (C0, F3, C2, Z1); top 2 takes C0 (lowest of the equal
        // pair), giving (C2, F3, C0, Z1); top 1 gives (F3, C2, C0, Z1). The
        // bisection probes position 2: occurrence 0.
        assert_eq!(look(&names, &q(true, false), "Cost"), Some(0));
        assert_eq!(look(&names, &q(true, false), "ZdxeLZgd"), Some(1));
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
        let keys: Vec<i32> = (0..500)
            .map(|_| {
                state = state.wrapping_mul(1_103_515_245).wrapping_add(12345);
                (state >> 20) as i32 % 37 - 18
            })
            .collect();
        let mut x: Vec<usize> = (0..500).collect();
        reference_sort(&mut x, |&i| &keys[i]);
        assert!(x.windows(2).all(|w| keys[w[0]] <= keys[w[1]]));
        let mut ids = x.clone();
        ids.sort_unstable();
        assert_eq!(ids, (0..500).collect::<Vec<_>>());
    }

    /// spec: S-0002 Constants (forgetting peaks)
    #[test]
    fn forgetting_peaks() {
        let peaks: Vec<usize> = (0..200).filter(|&c| forgetting_peak(c)).collect();
        assert_eq!(peaks, [0, 10, 20, 40, 80, 160]);
    }

    /// spec: S-0002 State and ordering (name tables during and after a
    /// layered load: lookups, additions and replacements)
    #[test]
    fn model_transitions() {
        let q = Quirks::default();
        // A fresh load: file order, not ordered, nothing remembered, peak 3.
        let mut t = table(&["Cost", "Cost", "Filler0"], &q);
        assert_eq!((t.ordered, t.remembered, t.peak), (false, None, 3));
        // Row 4: sorts, finds fixture 2's winner and remembers it.
        assert_eq!(t.lookup_mut(b"Cost"), Some(1));
        assert_eq!((t.ordered, t.remembered), (true, Some(1)));
        assert_eq!(t.working, [2, 1, 0]);
        // Row 3: ordered, not found: the remembered occurrence stays.
        assert_eq!(t.lookup_mut(b"Nope"), None);
        assert_eq!(t.remembered, Some(1));
        // Row 7: an addition at 3 occurrences (not a forgetting peak) keeps
        // it. Re-sorting with a name smaller than Filler0 would now return
        // occurrence 0, but the remembered one answers.
        let small = name_where("S", |h| h < name_hash(b"Filler0"));
        t.add(small.as_bytes());
        assert_eq!((t.ordered, t.remembered, t.peak), (false, Some(1), 4));
        t.settle();
        assert_eq!(t.lookup(b"Cost"), Some(1));
        // Working order (Filler0, Cost#1, Cost#0, small) sorts to (small,
        // Filler0, Cost#0, Cost#1), whose bisection probes Cost#0.
        let mut forgot = t.clone();
        forgot.forget();
        assert_eq!(forgot.lookup(b"Cost"), Some(0));
        // Row 5 (Q-0183): not ordered, not found: nothing is remembered.
        assert_eq!(t.lookup_mut(b"Nope"), None);
        assert_eq!((t.ordered, t.remembered), (true, None));
        // Row 8: a replacement forgets and keeps the table ordered.
        assert_eq!(t.lookup_mut(b"Cost"), Some(0));
        assert_eq!(t.remembered, Some(0));
        t.replace(0);
        assert_eq!((t.ordered, t.remembered), (true, None));
        assert!(!t.working.contains(&0));
        // The peak count does not fall.
        assert_eq!(t.peak, 4);
    }

    /// spec: S-0002 State and ordering (row 6: an addition at a peak count
    /// that is a forgetting peak forgets; one below it does not)
    #[test]
    fn addition_at_a_forgetting_peak() {
        let q = Quirks::default();
        let mut names = vec!["Cost".to_owned(), "Cost".to_owned()];
        names.extend((0..7).map(|i| format!("F{i}")));
        let refs: Vec<&str> = names.iter().map(String::as_str).collect();
        let mut t = table(&refs, &q);
        assert_eq!(t.peak, 9);
        assert!(t.lookup_mut(b"Cost").is_some());
        t.add(b"G0");
        assert!(t.remembered.is_some());
        assert_eq!(t.peak, 10);
        assert!(t.lookup_mut(b"Cost").is_some());
        t.add(b"G1");
        assert_eq!((t.remembered, t.peak), (None, 11));
        // An addition below the peak count does not forget, and one made
        // when the table again holds exactly its peak count does.
        let mut t = table(&refs, &q);
        t.add(b"G0");
        t.replace(2);
        assert_eq!((t.working.len(), t.peak), (9, 10));
        assert!(t.lookup_mut(b"Cost").is_some());
        t.add(b"G1");
        assert!(t.remembered.is_some());
        assert_eq!((t.working.len(), t.peak), (10, 10));
        t.add(b"G2");
        assert_eq!((t.remembered, t.peak), (None, 11));
    }

    /// spec: S-0002 State and ordering (a table with no occurrences)
    #[test]
    fn empty_table() {
        let mut t = table(&[], &Quirks::default());
        assert_eq!(t.lookup_mut(b"Cost"), None);
        assert_eq!((t.ordered, t.peak), (false, 0));
        // An addition to an empty table: 0 is a forgetting peak.
        t.add(b"Cost");
        assert_eq!(t.lookup(b"Cost"), Some(0));
    }

    /// spec: S-0002 Edge cases (second row: the remembered occurrence
    /// answers only a lookup of its own name)
    #[test]
    fn identity_off_remembers_by_name() {
        let q = Quirks {
            ini_hash_identity: false,
            ..Quirks::default()
        };
        let mut t = table(&["Cost", "Cost", "ZdxeLZgd"], &q);
        // Pairs (C0, C1, Z2) sort to (C1, C0, Z2); the bisection probes 1.
        assert_eq!(t.lookup_mut(b"Cost"), Some(0));
        assert_eq!(t.lookup_mut(b"ZdxeLZgd"), Some(2));
        assert_eq!(t.remembered, Some(2));
        t.settle();
        assert_eq!(t.lookup(b"Cost"), Some(0));
        let names: [&[u8]; 3] = [b"Cost", b"Cost", b"ZdxeLZgd"];
        assert_eq!(
            t.other_name_with_hash(b"Cost", |i| names[i as usize]),
            Some(2)
        );
    }
}
