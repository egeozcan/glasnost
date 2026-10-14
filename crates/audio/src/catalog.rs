//! The sound data: named sound entries and their settings (S-0701 claims 1
//! to 4, Formulas F1), read from the sound rules file of S-0005 with the
//! INI reader of S-0001 to S-0003.
//!
//! Entry names are matched without regard to case; keys are matched by
//! exact case (S-0001 claim 13), so a key spelled with other capitals is
//! ignored (S-0701 claim 2, Import mapping).

use std::collections::BTreeMap;
use std::sync::Arc;

use glasnost_ini::binfloat::{Binary, Format};
use glasnost_ini::value::{self, COMMON_TEXT_LIMIT, Decimal, DecimalField};
use glasnost_ini::{Document, Section};

/// Most samples an entry holds (S-0701 Constants).
pub const MAX_SAMPLES: usize = 32; // spec: S-0701 claim 2 and Constants (at most 32 samples)
/// Longest entry name (S-0701 Constants).
pub const MAX_NAME: usize = 31; // spec: S-0701 claim 1 and Constants (a longer name is cut to 31)
/// Full sound level.
pub const FULL: i32 = 16384; // spec: S-0701 Terms (sound level 0 … 16384)

// spec: S-0701 Constants (defaults when absent from both the entry and the defaults section)
const DEFAULT_VOLUME: i64 = 80;
const DEFAULT_MIN_VOLUME: i64 = 20;
const DEFAULT_LIMIT: i32 = 5;
const DEFAULT_RANGE: i32 = 10;

/// A sound priority (S-0701 Terms), lowest first.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    /// `LOWEST` (0).
    Lowest,
    /// `LOW` (1).
    Low,
    /// `NORMAL` (2), the default (S-0701 Constants).
    #[default]
    Normal,
    /// `HIGH` (3).
    High,
    /// `CRITICAL` (4).
    Critical,
}

impl Priority {
    /// The priority a value names (S-0701 Import mapping: the whole value,
    /// any case); anything else is `NORMAL`.
    pub fn parse(stored: &[u8]) -> Self {
        // spec: S-0701 claim 2 and Import mapping (the five words in any case; anything else NORMAL)
        match value::named(
            stored,
            stored.len(),
            &["LOWEST", "LOW", "NORMAL", "HIGH", "CRITICAL"],
        ) {
            Some(0) => Self::Lowest,
            Some(1) => Self::Low,
            Some(3) => Self::High,
            Some(4) => Self::Critical,
            _ => Self::Normal,
        }
    }
}

/// The control words of an entry (S-0701 claim 2).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Control {
    /// `ALL`: every body sample (claim 14).
    pub all: bool,
    /// `LOOP`: the body repeats (claim 14).
    pub looped: bool,
    /// `RANDOM`: a random body choice or order (claim 14).
    pub random: bool,
    /// `PREDELAY`: a pre-delay draw (claim 13).
    pub predelay: bool,
    /// `INTERRUPT`: a new request ranks above equally loud playing ones
    /// (claim 8).
    pub interrupt: bool,
    /// `ATTACK`: the attack partition (claim 2).
    pub attack: bool,
    /// `DECAY`: the decay partition (claim 2).
    pub decay: bool,
    /// `AMBIENT`: a pre-delay of at least 33 ms (claim 13).
    pub ambient: bool,
}

/// The placement words that act (S-0701 claims 2 and 6).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Placement {
    /// `GLOBAL`: at least the minimum level.
    pub global: bool,
    /// `LOCAL` (excludes `SCREEN`; the last given wins): distance from
    /// the view centre.
    pub local: bool,
    /// `SHROUD`: silent on cells the local player has not explored.
    pub shroud: bool,
}

/// One sound entry (S-0701 claim 2).
#[derive(Clone, Debug, PartialEq)]
pub struct SoundEntry {
    /// Its name, cut to 31 characters.
    pub name: String,
    /// Sample names, in order, at most 32.
    pub samples: Vec<String>,
    /// The entry level *e* of F1, 0 … 16384.
    pub level: i32,
    /// Volume shift, percent, 0 … 100.
    pub vshift: i32,
    /// Minimum level, a fraction 0 … 1 in single precision (F1).
    pub min_level: f32,
    /// Sound priority.
    pub priority: Priority,
    /// Attack count, after the control words (claim 2).
    pub attack: i32,
    /// Decay count, after the control words (claim 2).
    pub decay: i32,
    /// Control words.
    pub control: Control,
    /// Placement words.
    pub placement: Placement,
    /// Most instances at once; 0 is no limit (claim 8).
    pub limit: i32,
    /// Loop count; 0 is forever (claim 14).
    pub loop_count: i32,
    /// Range, cells (F2).
    pub range: i32,
    /// Delay *d*₁ *d*₂, ms (claims 13 and 14).
    pub delay: (i32, i32),
    /// Pitch shift *f*₁ *f*₂, percent (claim 13).
    pub fshift: (i32, i32),
}

/// An entry of a [`SoundCatalog`], by position.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EntryId(pub u32);

/// The game-wide defaults section's values (S-0701 claim 2).
#[derive(Clone, Copy, Debug)]
struct Defaults {
    level: i32,
    min_level: f32,
    priority: Priority,
    limit: i32,
    range: i32,
    control: Control,
    placement: Placement,
}

/// The sound entries of a game.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SoundCatalog {
    entries: Vec<Arc<SoundEntry>>,
    by_name: BTreeMap<String, EntryId>,
}

/// An exactly scaled clamp of F1: `v × c` in double precision rounded toward
/// zero, clamped to 0 … 1, where `c` is the single-precision approximation
/// to 0.01 widened exactly.
fn scaled(v: Binary) -> f64 {
    // spec: S-0701 F1 (c = 0.00999999977648258209228515625 = 10737418 × 2⁻³⁰)
    let c = Binary::round_toward_zero(Format::Single, false, 10_737_418, -30);
    // spec: S-0701 F1 (the product in double precision under S-0205's rounding, without a single-precision rounding)
    let p = v.mul_rz(&c, Format::Double);
    let x = as_f64(p);
    // spec: S-0701 F1 (clamp to 0 … 1; NaN gives 0)
    if x.is_nan() { 0.0 } else { x.clamp(0.0, 1.0) }
}

/// A binary value as a host double (exact: every single widens exactly).
fn as_f64(b: Binary) -> f64 {
    match b.format {
        Format::Single => f64::from(f32::from_bits(b.bits as u32)),
        Format::Double => f64::from_bits(b.bits),
    }
}

/// The entry level of F1 for a volume percentage as read.
pub fn entry_level(v: Binary, full_volume_16383: bool) -> i32 {
    // spec: S-0701 F1 and Edge cases (quirks.full_volume_16383 off: 100 or more gives exactly 16384; below 100 the formula is unchanged)
    if !full_volume_16383 && as_f64(v) >= 100.0 {
        return FULL;
    }
    // spec: S-0701 F1 (e = ⌊clamp(v × 0.01, 0, 1) × 16384⌋, truncated toward zero)
    (scaled(v) * f64::from(FULL)) as i32
}

/// The minimum level of F1 for a percentage as read: the same scaled clamp,
/// narrowed to single precision toward zero.
pub fn min_level(v: Binary) -> f32 {
    let x = scaled(v);
    // spec: S-0701 F1 (narrowed to single precision as a fraction; S-0205 rounding toward zero)
    let b = Binary {
        format: Format::Double,
        bits: x.to_bits(),
    }
    .convert_rz(Format::Single);
    f32::from_bits(b.bits as u32)
}

/// A decimal value through S-0003's floating-point import; `None` keeps the
/// value in force (S-0003 claim 8).
fn decimal(stored: &[u8]) -> Option<Binary> {
    match value::decimal(stored, DecimalField::Double) {
        Decimal::Value { value, .. } => Some(value),
        Decimal::NoNumber => None,
    }
}

fn percent(p: i64) -> Binary {
    Binary::from_i64(Format::Single, p)
}

/// Words separated by spaces, tabs or newlines (S-0701 claim 2).
fn words(stored: &[u8]) -> impl Iterator<Item = &[u8]> {
    // spec: S-0701 claim 2 (separated by spaces, tabs or newlines, not commas)
    stored
        .split(|&b| b == b' ' || b == b'\t' || b == b'\n' || b == b'\r')
        .filter(|w| !w.is_empty())
}

fn text(bytes: &[u8]) -> String {
    bytes.iter().map(|&b| char::from(b)).collect()
}

/// The control words of a value, or `None` when it has no token (the set in
/// force stays).
fn control(stored: &[u8]) -> Option<Control> {
    let mut c = Control::default();
    let mut any = false;
    for w in words(stored) {
        any = true;
        // spec: S-0701 claim 2 and Import mapping (the eight control words; unknown words are ignored)
        match value::named(
            w,
            w.len(),
            &[
                "ALL",
                "LOOP",
                "RANDOM",
                "PREDELAY",
                "INTERRUPT",
                "ATTACK",
                "DECAY",
                "AMBIENT",
            ],
        ) {
            Some(0) => c.all = true,
            Some(1) => c.looped = true,
            Some(2) => c.random = true,
            Some(3) => c.predelay = true,
            Some(4) => c.interrupt = true,
            Some(5) => c.attack = true,
            Some(6) => c.decay = true,
            Some(7) => c.ambient = true,
            _ => {}
        }
    }
    // spec: S-0701 claim 2 (at least one nonempty token, even an unknown one: exactly its own words)
    any.then_some(c)
}

/// The placement words of a value, starting from *screen*, or `None` when
/// it has no token.
fn placement(stored: &[u8]) -> Option<Placement> {
    let mut p = Placement::default();
    let mut any = false;
    for w in words(stored) {
        any = true;
        // spec: S-0701 claim 2 and Import mapping (GLOBAL, LOCAL and SHROUD act; the last of SCREEN and LOCAL, and of UNSHROUD and SHROUD, applies; the other words have no established effect)
        match value::named(
            w,
            w.len(),
            &["GLOBAL", "LOCAL", "SHROUD", "SCREEN", "UNSHROUD"],
        ) {
            Some(0) => p.global = true,
            Some(1) => p.local = true,
            Some(2) => p.shroud = true,
            Some(3) => p.local = false,
            Some(4) => p.shroud = false,
            _ => {}
        }
    }
    any.then_some(p)
}

/// One or two integers *a* [*b*], *b* = *a* when only one is given.
///
/// The separator between the two is not stated by S-0701; interim (Q-0334):
/// spaces, tabs or commas.
fn pair(stored: &[u8], in_force: (i32, i32)) -> (i32, i32) {
    // interim: Q-0334 (the separator of the two numbers)
    let mut it = stored
        .split(|&b| b == b' ' || b == b'\t' || b == b',')
        .filter(|w| !w.is_empty());
    let Some(a) = it.next().and_then(value::integer) else {
        return in_force;
    };
    // spec: S-0701 claim 2 (b = a when only one is given)
    let b = it.next().and_then(value::integer).unwrap_or(a);
    (a, b)
}

impl SoundCatalog {
    /// A catalog with no entries.
    pub fn new() -> Self {
        Self::default()
    }

    /// Reads the sound data file (S-0701 claims 1 to 3).
    /// `default_priority_ignored` is `quirks.default_sound_priority_ignored`,
    /// `full_volume_16383` is `quirks.full_volume_16383`.
    pub fn from_ini(
        doc: &Document,
        default_priority_ignored: bool,
        full_volume_16383: bool,
    ) -> Self {
        let d = doc.section(b"Defaults");
        let get = |k: &[u8]| d.and_then(|s| s.value(k));
        let level = get(b"Volume")
            .and_then(decimal)
            // spec: S-0701 Constants (default volume 80 %)
            .unwrap_or_else(|| percent(DEFAULT_VOLUME));
        let min = get(b"MinVolume")
            .and_then(decimal)
            // spec: S-0701 Constants (default minimum level 20 %)
            .unwrap_or_else(|| percent(DEFAULT_MIN_VOLUME));
        let defaults = Defaults {
            level: entry_level(level, full_volume_16383),
            min_level: min_level(min),
            // spec: S-0701 claim 3 (quirks.default_sound_priority_ignored: the defaults section's priority is never applied)
            priority: if default_priority_ignored {
                Priority::Normal
            } else {
                get(b"Priority").map_or(Priority::Normal, Priority::parse)
            },
            // spec: S-0701 Constants (default limit 5, range 10 cells)
            limit: get(b"Limit")
                .and_then(value::integer)
                .unwrap_or(DEFAULT_LIMIT),
            range: get(b"Range")
                .and_then(value::integer)
                .unwrap_or(DEFAULT_RANGE),
            // spec: S-0701 Import mapping (default control: none; default placement: SCREEN)
            control: get(b"Control").and_then(control).unwrap_or_default(),
            placement: get(b"Type").and_then(placement).unwrap_or_default(),
        };
        let mut out = Self::default();
        let Some(list) = doc.section(b"SoundList") else {
            return out;
        };
        // spec: S-0701 claim 1 (every value of the list section, in order; S-0002 claim 4 list positions)
        for (_, e) in list.list_positions() {
            let full = value::text(&e.value, COMMON_TEXT_LIMIT);
            if full.is_empty() {
                continue;
            }
            // spec: S-0701 claim 1 (a longer name is cut to 31)
            let name = text(&full[..full.len().min(MAX_NAME)]);
            let key = name.to_ascii_uppercase();
            let id = match out.by_name.get(&key) {
                Some(&id) => id,
                None => {
                    let id = EntryId(out.entries.len() as u32);
                    out.entries.push(Arc::new(SoundEntry {
                        name,
                        samples: Vec::new(),
                        level: defaults.level,
                        vshift: 0,
                        min_level: defaults.min_level,
                        priority: defaults.priority,
                        attack: 0,
                        decay: 0,
                        control: defaults.control,
                        placement: defaults.placement,
                        limit: defaults.limit,
                        loop_count: 0,
                        range: defaults.range,
                        delay: (0, 0),
                        fshift: (0, 0),
                    }));
                    out.by_name.insert(key, id);
                    id
                }
            };
            // spec: S-0701 claim 1 (a listed name with no section: no samples, silent; a second listing reads the section again over the same entry)
            if let Some(s) = doc.section(full) {
                read_entry(
                    Arc::make_mut(&mut out.entries[id.0 as usize]),
                    s,
                    full_volume_16383,
                );
            }
        }
        out
    }

    /// Adds or replaces an entry (tests and tools).
    pub fn insert(&mut self, entry: SoundEntry) -> EntryId {
        let key = entry.name.to_ascii_uppercase();
        if let Some(&id) = self.by_name.get(&key) {
            self.entries[id.0 as usize] = Arc::new(entry);
            return id;
        }
        let id = EntryId(self.entries.len() as u32);
        self.entries.push(Arc::new(entry));
        self.by_name.insert(key, id);
        id
    }

    /// The entry a sound name refers to, without regard to case; an unknown
    /// name resolves to no entry (S-0701 claim 1).
    pub fn resolve(&self, name: &str) -> Option<EntryId> {
        // spec: S-0701 claim 1 (matched without regard to case; unknown: no entry)
        self.by_name.get(&name.to_ascii_uppercase()).copied()
    }

    /// An entry.
    pub fn entry(&self, id: EntryId) -> &SoundEntry {
        &self.entries[id.0 as usize]
    }

    /// An entry, shared.
    pub fn entry_arc(&self, id: EntryId) -> Arc<SoundEntry> {
        self.entries[id.0 as usize].clone()
    }

    /// Number of entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether there are no entries.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl SoundEntry {
    /// An entry with the Constants' defaults and the given samples (tests
    /// and tools).
    pub fn new(name: &str, samples: &[&str]) -> Self {
        Self {
            name: name.to_owned(),
            samples: samples.iter().map(|s| (*s).to_owned()).collect(),
            // spec: S-0701 Constants and F1 (default volume 80 %, below 100, so quirks.full_volume_16383 does not change it)
            level: entry_level(percent(DEFAULT_VOLUME), true),
            vshift: 0,
            min_level: min_level(percent(DEFAULT_MIN_VOLUME)),
            priority: Priority::Normal,
            attack: 0,
            decay: 0,
            control: Control::default(),
            placement: Placement::default(),
            limit: DEFAULT_LIMIT,
            loop_count: 0,
            range: DEFAULT_RANGE,
            delay: (0, 0),
            fshift: (0, 0),
        }
    }
}

/// Reads an entry's section over the entry (S-0701 claim 2).
fn read_entry(e: &mut SoundEntry, s: &Section, full_volume_16383: bool) {
    // spec: S-0701 claim 2 (keys matched by exact case, S-0001 claim 13)
    let get = |k: &[u8]| s.value(k);
    if let Some(v) = get(b"Sounds") {
        for w in words(v) {
            // spec: S-0701 claim 2 (every leading `$` and `#` is dropped)
            let start = w
                .iter()
                .position(|&b| b != b'$' && b != b'#')
                .unwrap_or(w.len());
            // spec: S-0701 claim 1 (appending again up to the 32-sample cap); claim 2 (names past the 32nd are ignored)
            if e.samples.len() < MAX_SAMPLES {
                e.samples.push(text(&w[start..]));
            }
        }
    }
    if let Some(v) = get(b"Volume").and_then(decimal) {
        e.level = entry_level(v, full_volume_16383);
    }
    if let Some(v) = get(b"VShift").and_then(value::integer) {
        // spec: S-0701 claim 2 (volume shift clamped to 0 … 100)
        e.vshift = v.clamp(0, 100);
    }
    if let Some(v) = get(b"MinVolume").and_then(decimal) {
        e.min_level = min_level(v);
    }
    if let Some(v) = get(b"Priority") {
        e.priority = Priority::parse(v);
    }
    let attack = get(b"Attack").and_then(value::integer).unwrap_or(0);
    let decay = get(b"Decay").and_then(value::integer).unwrap_or(0);
    if let Some(c) = get(b"Control").and_then(control) {
        e.control = c;
    }
    // spec: S-0701 claim 2 (without the word the count becomes 0; with it and a count of 0 it becomes 1)
    let count = |word: bool, n: i32| {
        if !word {
            0
        } else if n == 0 {
            1
        } else {
            n
        }
    };
    e.attack = count(e.control.attack, attack);
    e.decay = count(e.control.decay, decay);
    if let Some(p) = get(b"Type").and_then(placement) {
        e.placement = p;
    }
    if let Some(v) = get(b"Limit").and_then(value::integer) {
        e.limit = v;
    }
    if let Some(v) = get(b"Loop").and_then(value::integer) {
        e.loop_count = v;
    }
    if let Some(v) = get(b"Range").and_then(value::integer) {
        e.range = v;
    }
    if let Some(v) = get(b"Delay") {
        e.delay = pair(v, e.delay);
    }
    if let Some(v) = get(b"FShift") {
        e.fshift = pair(v, e.fshift);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glasnost_ini::Quirks;

    fn doc(text: &str) -> Document {
        Document::parse(text.as_bytes(), Quirks::default(), &mut Vec::new())
    }

    fn cat(text: &str) -> SoundCatalog {
        SoundCatalog::from_ini(&doc(text), true, true)
    }

    // spec: S-0701 claim 1 (a name listed twice appends its samples; a name without a section is silent; any case resolves; unknown resolves to nothing; names cut to 31)
    #[test]
    fn claim_1_entries_come_from_the_list() {
        let long = "ABCDEFGHIJKLMNOPQRSTUVWXYZ01234_one";
        let long2 = "ABCDEFGHIJKLMNOPQRSTUVWXYZ01234_two";
        let c = cat(&format!(
            "[SoundList]\n0=Pair\n1=Pair\n2=Bare\n3={long}\n4={long2}\n\n\
             [Pair]\nSounds=a b\nControl=ALL\n\n\
             [{long}]\nSounds=x\n\n[{long2}]\nSounds=y\n"
        ));
        let p = c.entry(c.resolve("pAIR").unwrap());
        assert_eq!(p.samples, ["a", "b", "a", "b"]);
        assert!(c.entry(c.resolve("bare").unwrap()).samples.is_empty());
        assert!(c.resolve("Nothing").is_none());
        let t = c.entry(c.resolve(&long[..31]).unwrap());
        assert_eq!(t.name.len(), 31);
        assert_eq!(t.samples, ["x", "y"]);
        // spec: S-0701 claim 1 and scenario A1 (g) (a reference is compared in full: 32 or more characters never match a cut name)
        assert!(c.resolve(long).is_none());
    }

    // spec: S-0701 claim 2 (fields, forms, clamps and defaults; exact key case)
    #[test]
    fn claim_2_fields_and_defaults() {
        let c = cat("[SoundList]\n0=A\n1=B\n2=C\n3=D\n\n\
             [Defaults]\nVolume=50\nLimit=3\nRange=7\nControl=LOOP\nType=GLOBAL\n\n\
             [A]\nSounds=$#one\t##$two three\n\n\
             [B]\nSounds=a,b\nVolume=100\nVShift=150\nMinVolume=-5\nPriority=high\nAttack=0\nDecay=2\n\
             Control=ATTACK RANDOM,ALL\nType=LOCAL SCREEN SHROUD\nLimit=0\nLoop=3\nRange=4\nDelay=40\nFShift=-5 7\n\n\
             [C]\nvolume=10\nControl=BOGUS\nType=SCREEN LOCAL\nPriority=urgent\n\n\
             [D]\nControl=ATTACK DECAY\nAttack=2\n");
        let a = c.entry(c.resolve("A").unwrap());
        assert_eq!(a.samples, ["one", "two", "three"]);
        // Defaults section values.
        assert_eq!((a.level, a.limit, a.range), (8191, 3, 7));
        assert!(a.control.looped && a.placement.global);
        assert_eq!(a.priority, Priority::Normal);
        let b = c.entry(c.resolve("B").unwrap());
        assert_eq!(b.samples, ["a,b"], "commas do not separate samples");
        assert_eq!(b.level, 16383, "quirks.full_volume_16383");
        assert_eq!(b.vshift, 100);
        assert_eq!(b.min_level, 0.0);
        assert_eq!(b.priority, Priority::High);
        // `RANDOM,ALL` is one unknown word; the entry's own set replaces the default.
        assert_eq!(
            b.control,
            Control {
                attack: true,
                ..Control::default()
            }
        );
        assert_eq!(
            (b.attack, b.decay),
            (1, 0),
            "word with count 0 gives 1; no word gives 0"
        );
        assert_eq!(
            b.placement,
            Placement {
                shroud: true,
                ..Placement::default()
            },
            "SCREEN after LOCAL wins; own words replace the default set"
        );
        assert_eq!((b.limit, b.loop_count, b.range), (0, 3, 4));
        assert_eq!((b.delay, b.fshift), ((40, 40), (-5, 7)));
        let cc = c.entry(c.resolve("C").unwrap());
        assert_eq!(cc.level, 8191, "a key in other capitals is ignored");
        assert_eq!(
            cc.control,
            Control::default(),
            "an unknown token clears the default set"
        );
        assert!(cc.placement.local);
        assert_eq!(cc.priority, Priority::Normal);
        let d = c.entry(c.resolve("D").unwrap());
        assert_eq!((d.attack, d.decay), (2, 1));
    }

    // spec: S-0701 claim 2 and Constants (no defaults section: 80 %, 20 %, limit 5, range 10, no control, screen)
    #[test]
    fn claim_2_constants_without_a_defaults_section() {
        let c = cat("[SoundList]\n0=A\n\n[A]\nSounds=s\n");
        let a = c.entry(c.resolve("a").unwrap());
        // ⌊0.8 × 0.0099999998 × 100 … ⌋: 80 × c = 0.79999998…
        assert_eq!(a.level, 13107);
        assert_eq!(a.min_level, min_level(percent(20)));
        assert!(a.min_level < 0.2);
        assert_eq!((a.limit, a.range), (5, 10));
        assert_eq!(a.control, Control::default());
        assert_eq!(a.placement, Placement::default());
        assert_eq!(a, &SoundEntry::new("A", &["s"]));
    }

    // spec: S-0701 claim 3 (the defaults section's priority is ignored; off: it applies)
    #[test]
    fn claim_3_default_priority_is_ignored() {
        let text = "[SoundList]\n0=A\n\n[Defaults]\nPriority=CRITICAL\n\n[A]\nSounds=s\n";
        let on = SoundCatalog::from_ini(&doc(text), true, true);
        assert_eq!(on.entry(EntryId(0)).priority, Priority::Normal);
        let off = SoundCatalog::from_ini(&doc(text), false, true);
        assert_eq!(off.entry(EntryId(0)).priority, Priority::Critical);
    }

    // spec: S-0701 F1 (v = 100 gives 16383 with the quirk, 16384 without; v ≤ 0 gives 0; above 100 clamps)
    #[test]
    fn f1_entry_level() {
        assert_eq!(entry_level(percent(100), true), 16383);
        assert_eq!(entry_level(percent(100), false), 16384);
        assert_eq!(entry_level(percent(0), true), 0);
        assert_eq!(entry_level(percent(-20), true), 0);
        assert_eq!(entry_level(percent(250), true), 16384);
        assert_eq!(entry_level(percent(50), true), 8191);
        // spec: S-0701 F1 (quirk off: 100 or more is exactly 16384; below 100 the formula is unchanged)
        assert_eq!(entry_level(percent(250), false), 16384);
        assert_eq!(entry_level(percent(50), false), 8191);
        assert_eq!(entry_level(percent(0), false), 0);
    }

    // spec: S-0701 claim 2 and scenario B2 (e) (the last of SCREEN and LOCAL, and of UNSHROUD and SHROUD, applies)
    #[test]
    fn placement_words_last_one_applies() {
        let c = cat("[SoundList]\n0=A\n1=B\n2=C\n3=D\n\n\
             [A]\nType=local screen\n\n[B]\nType=screen local\n\n\
             [C]\nType=shroud unshroud\n\n[D]\nType=unshroud shroud\n");
        let p = |n: &str| c.entry(c.resolve(n).unwrap()).placement;
        assert!(!p("A").local);
        assert!(p("B").local);
        assert!(!p("C").shroud);
        assert!(p("D").shroud);
    }
}
