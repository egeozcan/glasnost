//! Music (S-0703): the music database's track list, which entries can play
//! in a skirmish, the next-track rule (repeat, list order, shuffle and the
//! fallback), and the player that runs the rule at the end of every audio
//! pass and streams the picked track to the mixer.
//!
//! Music is presentation. It reads nothing of the game but the local
//! player's side at the start of a game, and its shuffle draws take the
//! client's unsynchronised stream that [`crate::Audio`] lends the mixer
//! (S-0703 claim 6, S-0202 claim 11), never the scenario stream. The
//! music volume (our own setting) never adds or removes a draw (S-0703
//! claim 3, Edge cases).
//!
//! Track files are found once, when the install is read (claim 8), and
//! read through a [`TrackSource`] one track at a time when a pick starts
//! it. Every interim rule is listed in docs/development.md, "Music (M8 step
//! C1 part 1)".

use std::sync::Arc;

use glasnost_fmt::wav::WavInfo;
use glasnost_ini::Document;
use glasnost_ini::value;
use glasnost_sim::rng::Rng;

use crate::catalog::FULL;

/// Ranged draws a shuffle pick makes before it gives up (S-0703
/// Constants).
pub const SHUFFLE_DRAWS: u32 = 1000; // spec: S-0703 claim 6 and Constants (1000 ranged draws)
/// The entry the fallback starts (S-0703 Constants).
pub const FALLBACK_ENTRY: usize = 0; // spec: S-0703 claim 7 and Constants (entry 0)
/// Text limit of a track identifier in the track list.
const ID_LIMIT: usize = 31; // spec: S-0703 Import mapping ([Themes] values: text limit 31 bytes)
/// Text limit of a sound file name.
const SOUND_LIMIT: usize = 255; // spec: S-0703 Import mapping (Sound: text limit 255 bytes)
/// Text limit of a side restriction.
const SIDE_LIMIT: usize = 127; // spec: S-0703 Import mapping (Side: text limit 127 bytes)
/// The extension added to a sound file name.
const EXTENSION: &str = ".wav"; // spec: S-0703 claim 1a (the extension .wav is added)
/// Text limit of a map's theme name.
const THEME_LIMIT: usize = 127; // spec: S-0703 Import mapping ([Basic] Theme: text limit 127 bytes)
/// The loading track's identifier (S-0703 Terms).
pub const LOADING_TRACK: &str = "LOADING"; // spec: S-0703 Import mapping (LOADING, the loading track)
/// The score track's identifier (S-0703 Terms).
pub const SCORE_TRACK: &str = "SCORE"; // spec: S-0703 Import mapping (SCORE, the score track)
/// The menu track's identifier (S-0703 Terms).
pub const MENU_TRACK: &str = "INTRO"; // spec: S-0703 Import mapping (INTRO, the menu track)

/// One track entry of the music database (S-0703 Terms, Import mapping).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TrackEntry {
    /// The identifier, as cut (its own section's name).
    pub id: String,
    /// The file looked for (claim 1a), or `None` when the sound file name
    /// is empty once its leading `$` and `#` are removed: such an entry
    /// never has its file present.
    pub file: Option<String>,
    /// The *normal* flag (claim 2).
    pub normal: bool,
    /// The *repeat* flag (claim 4).
    pub repeat: bool,
    /// The side restriction (claim 2); `None` is "any side".
    pub side: Option<String>,
}

impl TrackEntry {
    /// An entry with every default (no section of its own).
    fn defaults(id: String) -> Self {
        Self {
            id,
            file: None,
            // spec: S-0703 Import mapping (Normal absent: true)
            normal: true,
            // spec: S-0703 Import mapping (Repeat absent: false)
            repeat: false,
            // spec: S-0703 Import mapping (Side absent: any side)
            side: None,
        }
    }
}

/// The track list (S-0703 claim 1), in list order.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MusicDatabase {
    /// Entry *k* at position *k*.
    pub entries: Vec<TrackEntry>,
}

fn text(bytes: &[u8]) -> String {
    bytes.iter().map(|&b| char::from(b)).collect()
}

/// The file a sound file name looks for (claim 1a): the name cut to 255
/// bytes, its leading `$` and `#` removed and `.wav` added; `None` when
/// nothing is left.
pub fn sound_file(stored: &[u8]) -> Option<String> {
    // spec: S-0703 Import mapping (Sound: cut as S-0003 claim 1 gives, then leading $ and # dropped)
    let cut = value::text(stored, SOUND_LIMIT);
    // spec: S-0703 claim 1a (any leading $ and # characters removed; the same characters elsewhere kept)
    let start = cut
        .iter()
        .position(|&b| b != b'$' && b != b'#')
        .unwrap_or(cut.len());
    let name = &cut[start..];
    // spec: S-0703 claim 1a (an empty name, or one made only of $ and #, never has its file present)
    if name.is_empty() {
        return None;
    }
    // spec: S-0703 claim 1a (the extension is added whatever the name ends with, also to a name already ending in .wav)
    Some(format!("{}{EXTENSION}", text(name)))
}

impl MusicDatabase {
    /// Reads the music database (S-0703 claim 1 and Import mapping).
    pub fn from_ini(doc: &Document) -> Self {
        let mut out = Self::default();
        let Some(list) = doc.section(b"Themes") else {
            // spec: S-0703 Import mapping ([Themes] absent: an empty list, claim 9)
            return out;
        };
        // spec: S-0703 Import mapping (every value of [Themes] in order; a repeated key gives its winning occurrence at each position, S-0002 claim 4)
        for (_, e) in list.list_positions() {
            // spec: S-0703 Import mapping (text limit 31 bytes, S-0003 claim 1)
            let id = value::text(&e.value, ID_LIMIT);
            // spec: S-0703 Import mapping (a key whose value is empty adds no entry; none and <none> are ordinary identifiers)
            if id.is_empty() {
                continue;
            }
            // spec: S-0703 claim 1 (an identifier listed twice is one entry, at its first listing; compared ignoring case in our engine)
            if out
                .entries
                .iter()
                .any(|t| t.id.as_bytes().eq_ignore_ascii_case(id))
            {
                continue;
            }
            let mut t = TrackEntry::defaults(text(id));
            // spec: S-0703 Import mapping (the entry's own section, found by the cut identifier, S-0002)
            if let Some(s) = doc.section(id) {
                if let Some(v) = s.value(b"Sound") {
                    t.file = sound_file(v);
                }
                // spec: S-0703 Import mapping (Normal, Repeat: booleans of S-0003; a malformed value keeps the default)
                if let Some(b) = s.value(b"Normal").and_then(value::boolean) {
                    t.normal = b;
                }
                if let Some(b) = s.value(b"Repeat").and_then(value::boolean) {
                    t.repeat = b;
                }
                if let Some(v) = s.value(b"Side") {
                    // spec: S-0703 Import mapping (Side: text limit 127 bytes; empty counts as absent)
                    let side = value::text(v, SIDE_LIMIT);
                    t.side = (!side.is_empty()).then(|| text(side));
                }
            }
            out.entries.push(t);
        }
        out
    }

    /// The number of entries, *n*.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the list is empty (claim 9).
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// The entry whose identifier is `id`, compared ignoring case as claim
    /// 1 compares identifiers.
    pub fn find(&self, id: &[u8]) -> Option<usize> {
        // spec: S-0703 Import mapping (the fixed names and the map theme are compared with the entry identifiers ignoring case, as claim 1 compares them)
        self.entries
            .iter()
            .position(|t| t.id.as_bytes().eq_ignore_ascii_case(id))
    }

    /// The loading track (S-0703 Terms, claim 11), if the list has one.
    pub fn loading(&self) -> Option<usize> {
        self.find(LOADING_TRACK.as_bytes())
    }

    /// The score track (S-0703 Terms, claim 12), if the list has one.
    pub fn score(&self) -> Option<usize> {
        self.find(SCORE_TRACK.as_bytes())
    }

    /// The menu track (S-0703 Terms, claim 12), if the list has one.
    pub fn menu(&self) -> Option<usize> {
        self.find(MENU_TRACK.as_bytes())
    }

    /// The map theme (S-0703 Terms, claim 11): the entry a map's stored
    /// `[Basic]` `Theme` value names, if any.
    pub fn theme(&self, stored: Option<&[u8]>) -> Option<usize> {
        // spec: S-0703 Import mapping ([Basic] Theme: text limit 127 bytes; an empty value counts as absent; a value that matches no entry gives no map theme)
        let name = value::text(stored?, THEME_LIMIT);
        if name.is_empty() {
            return None;
        }
        self.find(name)
    }

    /// The importer warnings for side restrictions that name no side of
    /// `sides` (S-0703 Edge cases), naming the entry and the value.
    pub fn side_warnings(&self, sides: &[String]) -> Vec<String> {
        self.entries
            .iter()
            .filter_map(|t| {
                let s = t.side.as_ref()?;
                (!names_a_side(s, sides)).then(|| {
                    format!(
                        "music track {}: side restriction {s:?} names no side of the game; the track never plays",
                        t.id
                    )
                })
            })
            .collect()
    }

    /// Which entries are available tracks in a game (S-0703 claim 2):
    /// `present` is claim 8's file presence by entry, `sides` the game's
    /// sides (S-0004 claim 1) and `local_side` the index of the local
    /// player's country's side.
    pub fn availability(
        &self,
        present: &[bool],
        sides: &[String],
        local_side: Option<usize>,
    ) -> Vec<bool> {
        let local = local_side.and_then(|i| sides.get(i));
        self.entries
            .iter()
            .enumerate()
            .map(|(k, t)| {
                // spec: S-0703 claim 2 (file present, normal flag set, side restriction any or matching; the scenario number plays no part)
                let side_ok = match &t.side {
                    None => true,
                    // spec: S-0703 Edge cases (a restriction naming no side of the game: never available)
                    Some(s) if !names_a_side(s, sides) => false,
                    // spec: S-0703 claim 2 and Edge cases (our engine: matched to the local player's side by name, ignoring case; no switch)
                    Some(s) => local.is_some_and(|l| l.eq_ignore_ascii_case(s)),
                };
                present.get(k).copied().unwrap_or(false) && t.normal && side_ok
            })
            .collect()
    }
}

/// Whether a side restriction names one of `sides` (ignoring case).
fn names_a_side(s: &str, sides: &[String]) -> bool {
    sides.iter().any(|n| n.eq_ignore_ascii_case(s))
}

/// The shuffle, repeat and in-game music settings (S-0703 Terms, Import
/// mapping).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MusicSettings {
    /// The shuffle setting (claims 5 and 6).
    pub shuffle: bool,
    /// The repeat setting (claim 4).
    pub repeat: bool,
    /// The in-game music setting (claim 11).
    pub in_game: bool,
}

impl Default for MusicSettings {
    /// The settings when the settings file does not give them.
    fn default() -> Self {
        Self {
            // spec: S-0703 Import mapping ([Audio] IsScoreShuffle absent: off)
            shuffle: false,
            // spec: S-0703 Import mapping ([Audio] IsScoreRepeat absent: off)
            repeat: false,
            // spec: S-0703 Import mapping ([Audio] InGameMusic absent: on)
            in_game: true,
        }
    }
}

impl MusicSettings {
    /// Reads `[Audio]` `IsScoreShuffle`, `IsScoreRepeat` and `InGameMusic`
    /// from the settings file, when there is one.
    pub fn from_ini(doc: Option<&Document>) -> Self {
        let d = Self::default();
        // spec: S-0703 Import mapping ([Audio] keys: booleans of S-0003 claim 2; a value whose first byte is none of the six recognised keeps the default; a missing settings file means every default, S-0005 claim 5)
        let get = |k: &[u8], default: bool| {
            doc.and_then(|d| d.value(b"Audio", k))
                .and_then(value::boolean)
                .unwrap_or(default)
        };
        Self {
            shuffle: get(b"IsScoreShuffle", d.shuffle),
            repeat: get(b"IsScoreRepeat", d.repeat),
            in_game: get(b"InGameMusic", d.in_game),
        }
    }
}

/// The music quirk switches.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MusicQuirks {
    /// `quirks.music_fallback_entry_zero` (S-0703 claim 7).
    pub fallback_entry_zero: bool,
}

impl Default for MusicQuirks {
    fn default() -> Self {
        Self {
            // spec: S-0703 Edge cases (quirks.music_fallback_entry_zero: default on, the original's behaviour)
            fallback_entry_zero: true,
        }
    }
}

/// The next track (S-0703 claims 4 to 7, 9 and Formulas): `available` by
/// entry, `ended` the ended track. Shuffle draws take `rng` (the
/// unsynchronised stream); `half_span` is `quirks.ranged_draw_half_span`
/// (S-0202 claim 5). `None` is no track.
pub fn next_track(
    db: &MusicDatabase,
    available: &[bool],
    ended: Option<usize>,
    settings: MusicSettings,
    quirks: MusicQuirks,
    rng: &mut Rng,
    half_span: bool,
) -> Option<usize> {
    let n = db.len();
    // spec: S-0703 claim 9 (n = 0: no pick and no ranged draw, with either switch or setting)
    if n == 0 {
        return None;
    }
    let avail = |k: usize| available.get(k).copied().unwrap_or(false);
    // spec: S-0703 claim 4 (with an ended track whose repeat flag is set, or the repeat setting on: the ended track again, no draw, available or not)
    if let Some(e) = ended
        && (db.entries.get(e).is_some_and(|t| t.repeat) || settings.repeat)
    {
        return Some(e);
    }
    let fallback = || {
        // spec: S-0703 claim 7 (switch on: entry 0, whether or not available; off: no track)
        quirks.fallback_entry_zero.then_some(FALLBACK_ENTRY)
    };
    if settings.shuffle {
        // spec: S-0703 claim 6 (an accepted draw differs from the ended track and names an available track)
        let accept = |x: usize| Some(x) != ended && avail(x);
        let hi = i32::try_from(n - 1).unwrap_or(i32::MAX);
        if quirks.fallback_entry_zero {
            // spec: S-0703 claim 6 and Formulas (switch on: at most 1000 ranged draws R(0, n − 1); the 1000th is made but never accepted)
            for i in 1..=SHUFFLE_DRAWS {
                let x = rng.ranged(0, hi, half_span) as usize;
                if i < SHUFFLE_DRAWS && accept(x) {
                    return Some(x);
                }
            }
            return fallback();
        }
        // spec: S-0703 claim 7 and Formulas (switch off: draws go on until one is accepted when A holds an entry other than e; otherwise no draw: e when e is available, else no track)
        if (0..n).any(accept) {
            loop {
                let x = rng.ranged(0, hi, half_span) as usize;
                if accept(x) {
                    return Some(x);
                }
            }
        }
        return ended.filter(|&e| avail(e));
    }
    // spec: S-0703 claim 5 and Formulas (list order: from e + 1, or 0 with no ended track, n steps modulo n ending at e; no draw)
    let s = ended.map_or(0, |e| e + 1);
    (0..n)
        .map(|i| (s + i) % n)
        .find(|&j| avail(j))
        .or_else(fallback)
}

/// The result of opening a track's file.
pub enum Load {
    /// The file's bytes.
    Ready(Arc<[u8]>),
    /// The read is under way: [`TrackSource::poll`] gives it later.
    Pending,
    /// The file could not be opened or read.
    Failed,
}

/// Where the player reads track files from.
pub trait TrackSource {
    /// Starts reading entry `k`'s file, dropping any read under way.
    fn open(&mut self, k: usize) -> Load;
    /// The read under way, if it has finished ([`Load::Pending`] while it
    /// has not, or when there is none).
    fn poll(&mut self) -> Load;
}

/// Track files held in memory, by entry (tests, tools).
#[derive(Clone, Debug, Default)]
pub struct MemoryTracks {
    files: std::collections::BTreeMap<usize, Arc<[u8]>>,
    /// How many opens have been asked for (tests).
    pub opens: usize,
}

impl MemoryTracks {
    /// No files.
    pub fn new() -> Self {
        Self::default()
    }

    /// Gives entry `k` these file bytes.
    pub fn insert(&mut self, k: usize, bytes: Vec<u8>) {
        self.files.insert(k, Arc::from(bytes));
    }

    /// Removes entry `k`'s file (a file removed after startup, claim 8).
    pub fn remove(&mut self, k: usize) {
        self.files.remove(&k);
    }

    /// Which entries have a file, for `n` entries (claim 8's presence).
    pub fn present(&self, n: usize) -> Vec<bool> {
        (0..n).map(|k| self.files.contains_key(&k)).collect()
    }
}

impl TrackSource for MemoryTracks {
    fn open(&mut self, k: usize) -> Load {
        self.opens += 1;
        self.files
            .get(&k)
            .cloned()
            .map_or(Load::Failed, Load::Ready)
    }

    fn poll(&mut self) -> Load {
        Load::Pending
    }
}

/// The logical duration of a track (S-0703 claim 10): S-0701 claim 30's
/// *T* = max(1, ⌈100000 × *N* / (*f* × *p*)⌉) ms with *p* = 100.
pub fn logical_ms(frames: u64, rate: u32) -> i64 {
    // spec: S-0703 claim 10 (S-0701 claim 30's completion rule with pitch percentage p = 100)
    const P: u128 = 100;
    let num = 100_000u128 * u128::from(frames);
    let den = u128::from(rate.max(1)) * P;
    // spec: S-0701 claim 30 (exact rational arithmetic rounded upward once; at least 1 ms)
    i64::try_from(num.div_ceil(den)).unwrap_or(i64::MAX).max(1)
}

/// A track being streamed to the output: decoded a chunk at a time and
/// resampled to the output rate by linear interpolation, as the mixer's
/// other voices are.
#[derive(Clone, Debug)]
pub struct MusicVoice {
    info: WavInfo,
    file: Arc<[u8]>,
    chunks: usize,
    next_chunk: usize,
    /// Decoded samples from frame `base` on.
    buf: Vec<i16>,
    base: u64,
    /// Position in track frames, 32.32.
    pos: u64,
    /// Track frames per output frame, 32.32.
    step: u64,
}

impl MusicVoice {
    /// A voice playing `file` (with its header `info`) at `output_rate`.
    pub fn new(info: WavInfo, file: Arc<[u8]>, output_rate: u32) -> Self {
        let step = ((u64::from(info.sample_rate) << 32) / u64::from(output_rate.max(1))).max(1);
        Self {
            chunks: info.chunks(file.len()),
            info,
            file,
            next_chunk: 0,
            buf: Vec::new(),
            base: 0,
            pos: 0,
            step,
        }
    }

    fn channels(&self) -> usize {
        usize::from(self.info.channels.max(1))
    }

    fn end(&self) -> u64 {
        self.base + (self.buf.len() / self.channels()) as u64
    }

    /// Decodes until frame `i` is held (or the file ends); drops frames
    /// before `keep`.
    fn fill(&mut self, i: u64, keep: u64) {
        while i >= self.end() && self.next_chunk < self.chunks {
            if keep > self.base {
                let base = keep.min(self.end());
                let drop = ((base - self.base) as usize) * self.channels();
                self.buf.drain(..drop);
                self.base = base;
            }
            let k = self.next_chunk;
            self.next_chunk += 1;
            if self
                .info
                .decode_chunk(&self.file, k, &mut self.buf)
                .is_err()
            {
                // A block that does not decode ends the track's sound.
                self.chunks = self.next_chunk;
            }
        }
    }

    fn sample(&self, f: u64, c: usize) -> i64 {
        let ch = self.channels();
        let at = (f - self.base) as usize * ch + c.min(ch - 1);
        i64::from(self.buf.get(at).copied().unwrap_or(0))
    }

    /// Whether the decoded audio has ended.
    pub fn finished(&self) -> bool {
        self.next_chunk >= self.chunks && (self.pos >> 32) >= self.end()
    }

    /// Mixes the next `acc.len() / 2` stereo frames into `acc` at gain `g`
    /// (16.16), advancing the voice.
    pub fn mix(&mut self, acc: &mut [i64], g: i64) {
        let frames = acc.len() / 2;
        for f in 0..frames {
            let i = self.pos >> 32;
            self.fill(i + 1, i);
            if i >= self.end() {
                break;
            }
            let j = (i + 1).min(self.end() - 1);
            let frac = (self.pos & 0xFFFF_FFFF) as i64;
            let lerp = |c: usize| {
                let a = self.sample(i, c);
                let b = self.sample(j, c);
                a + (((b - a) * frac) >> 32)
            };
            let (l, r) = if self.channels() == 1 {
                let s = lerp(0);
                (s, s)
            } else {
                (lerp(0), lerp(1))
            };
            acc[2 * f] += (l * g) >> 16;
            acc[2 * f + 1] += (r * g) >> 16;
            self.pos += self.step;
        }
    }
}

/// The header of a playable sound file (S-0703 claim 8a) from its first
/// bytes `head`, for a file of `file_len` bytes: a RIFF WAVE file whose
/// format chunk comes before its data chunk, whose format code is one our
/// decoder plays, and which holds sample data after its header. `None`
/// when the file does not parse as a playable sound (its start fails).
pub fn playable(head: &[u8], file_len: usize) -> Option<WavInfo> {
    // spec: S-0703 claim 8a (RIFF form WAVE, format chunk before the data chunk, format code 1 or 17, sample data after the header; a file that lacks any of these does not parse)
    WavInfo::parse(head).ok().filter(|i| i.frames(file_len) > 0)
}

/// What the player is doing with the track it last started.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum State {
    /// No track plays: a failed start, no track picked, a track that
    /// finished, or nothing started yet.
    Idle,
    /// Entry `entry` started at pass time `c0`; `t` is its logical
    /// duration once its file has been read.
    Track {
        entry: usize,
        c0: i64,
        t: Option<i64>,
    },
}

/// Where the program is, for music (S-0703 claims 10 to 13).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    /// No music: no game yet, or the in-game music setting off in a game.
    Off,
    /// A game is loading: the loading track, if any, plays; no pick.
    Loading,
    /// The game has loaded: the next music update makes the first start.
    Armed,
    /// Music started (claim 10): music updates pick.
    Started,
    /// The end screen: the score track, timed by the output, no pick.
    Score,
    /// The main menu: the menu track, if any, no pick.
    Menu,
}

/// What a music update or a direct start did (tests, the lane).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Update {
    /// Nothing (music not started, an empty list, or a track still
    /// playing).
    Nothing,
    /// A pick or a direct start started entry `k`, which plays or is
    /// being read.
    Started(usize),
    /// A start of entry `k` failed (claims 8 and 8a).
    Failed(usize),
    /// A pick found no track (switch off).
    NoTrack,
}

/// The music player of the program (S-0703 claims 3 to 13).
pub struct Music {
    db: MusicDatabase,
    present: Vec<bool>,
    available: Vec<bool>,
    settings: MusicSettings,
    quirks: MusicQuirks,
    source: Box<dyn TrackSource>,
    warnings: Vec<String>,
    phase: Phase,
    theme: Option<usize>,
    ended: Option<usize>,
    state: State,
    log: Vec<Update>,
}

impl Music {
    /// A player for `db`, whose files are present by `present` (claim 8:
    /// found when the install was read) and read from `source`.
    pub fn new(
        db: MusicDatabase,
        present: Vec<bool>,
        settings: MusicSettings,
        source: Box<dyn TrackSource>,
    ) -> Self {
        Self {
            available: vec![false; db.len()],
            db,
            present,
            settings,
            quirks: MusicQuirks::default(),
            source,
            warnings: Vec::new(),
            phase: Phase::Off,
            theme: None,
            ended: None,
            state: State::Idle,
            log: Vec::new(),
        }
    }

    /// Sets the quirk switches.
    pub fn set_quirks(&mut self, q: MusicQuirks) {
        self.quirks = q;
    }

    /// The settings.
    pub fn settings(&self) -> MusicSettings {
        self.settings
    }

    /// The track list.
    pub fn database(&self) -> &MusicDatabase {
        &self.db
    }

    /// The source of track files (tests).
    pub fn source_mut(&mut self) -> &mut dyn TrackSource {
        self.source.as_mut()
    }

    /// Where the program is, for music.
    pub fn phase(&self) -> Phase {
        self.phase
    }

    /// A game starts to load (S-0703 claim 11): whatever plays is stopped
    /// (the caller silences the output), the ended track is forgotten, and
    /// the loading track, if any, is started directly. Returns its voice.
    pub fn loading_track(&mut self, output_rate: u32) -> Option<MusicVoice> {
        // spec: S-0703 claim 11 and State and ordering (loading starts: the ended track is forgotten; the loading track is started directly, not by a pick; music is not yet started)
        self.phase = Phase::Loading;
        self.ended = None;
        let k = self.db.loading();
        self.direct(k, 0, output_rate)
    }

    /// Starts music for a game whose sides are `sides` (S-0004 claim 1),
    /// whose local player's country is on side `local_side` and whose map
    /// gives `theme` as its stored `[Basic]` `Theme` value: decides
    /// availability (claim 2), gives the side warnings (Edge cases) and
    /// arms the first start (claim 11). Any loading track has been stopped.
    pub fn start_game(
        &mut self,
        sides: &[String],
        local_side: Option<usize>,
        theme: Option<&[u8]>,
    ) {
        // spec: S-0703 Edge cases (the side check at each skirmish start, against that game's sides)
        self.warnings = self.db.side_warnings(sides);
        self.available = self.db.availability(&self.present, sides, local_side);
        // spec: S-0703 claim 11 (every skirmish starts with no ended track, whatever its loading track did; our engine stops the loading track when the game starts)
        self.ended = None;
        self.state = State::Idle;
        self.log.clear();
        self.theme = self.db.theme(theme);
        // spec: S-0703 claim 11 (the in-game music setting, as read at startup, off: no track is picked or started during the game)
        self.phase = if self.settings.in_game {
            Phase::Armed
        } else {
            Phase::Off
        };
    }

    /// The game is over (S-0703 claim 12): our end screen opens, the
    /// playing track stops at once (the caller silences the output) and
    /// the score track, if any, is started directly, even with the in-game
    /// music setting off. No music update follows. Returns its voice.
    pub fn game_over(&mut self, output_rate: u32) -> Option<MusicVoice> {
        // spec: S-0703 claim 12 (our engine: the end screen opens when the game is over; the playing track stops at once; the score track is started directly, from its start, not by a pick; no music update after that cycle)
        self.phase = Phase::Score;
        let k = self.db.score();
        self.direct(k, 0, output_rate)
    }

    /// The main menu opens from the end screen or from a game left through
    /// its options menu (S-0703 claims 12 and 13): whatever plays stops at
    /// once (the caller silences the output) and the menu track, if any,
    /// starts. Returns its voice.
    pub fn main_menu(&mut self, output_rate: u32) -> Option<MusicVoice> {
        // spec: S-0703 claim 12 (leaving the score screen for the main menu stops the score track at once; the menu track, if any, starts)
        // spec: S-0703 claim 13 (our engine: leaving through the options menu stops the track at once, with no music update or pick; the main menu then starts the menu track)
        self.phase = Phase::Menu;
        let k = self.db.menu();
        self.direct(k, 0, output_rate)
    }

    /// Stops the music, with no track to follow (the program leaves a
    /// screen with music for one that has none of its own).
    pub fn stop(&mut self) {
        self.phase = Phase::Off;
        self.state = State::Idle;
    }

    /// The side warnings of the game.
    pub fn warnings(&self) -> &[String] {
        &self.warnings
    }

    /// The available tracks of the game.
    pub fn available(&self) -> &[bool] {
        &self.available
    }

    /// The ended track.
    pub fn ended(&self) -> Option<usize> {
        self.ended
    }

    /// The entry playing (or being read), if any.
    pub fn playing(&self) -> Option<usize> {
        match self.state {
            State::Track { entry, .. } => Some(entry),
            State::Idle => None,
        }
    }

    /// The updates and starts since the last call, oldest first.
    pub fn take_log(&mut self) -> Vec<Update> {
        std::mem::take(&mut self.log)
    }

    /// A direct start of entry `k` (claims 11 and 12), not by a pick: it
    /// plays whether or not it is available, unless it fails, and it makes
    /// no ranged draw. `None` for `k` silences the music.
    fn direct(&mut self, k: Option<usize>, c: i64, output_rate: u32) -> Option<MusicVoice> {
        let Some(k) = k else {
            self.state = State::Idle;
            return None;
        };
        self.start(k, c, output_rate)
    }

    /// Starts entry `k` at pass time `c`: opens its file.
    fn start(&mut self, k: usize, c: i64, output_rate: u32) -> Option<MusicVoice> {
        self.state = State::Track {
            entry: k,
            c0: c,
            t: None,
        };
        let load = self.source.open(k);
        let pending = matches!(load, Load::Pending);
        let voice = self.loaded(load, output_rate);
        if pending || voice.is_some() {
            self.log.push(Update::Started(k));
        }
        voice
    }

    /// A finished read: the track plays, or its start fails.
    fn loaded(&mut self, load: Load, output_rate: u32) -> Option<MusicVoice> {
        let State::Track { entry, c0, t: None } = self.state else {
            return None;
        };
        let parsed = match load {
            Load::Pending => return None,
            Load::Failed => None,
            Load::Ready(bytes) => playable(&bytes, bytes.len()).map(|i| (i, bytes)),
        };
        match parsed {
            Some((info, bytes)) => {
                // spec: S-0703 claim 10 (the logical duration counts from the pass at which the track started, and is that of the sample data it actually plays)
                let t = logical_ms(info.frames(bytes.len()), info.sample_rate);
                self.state = State::Track {
                    entry,
                    c0,
                    t: Some(t),
                };
                Some(MusicVoice::new(info, bytes, output_rate))
            }
            None => {
                // spec: S-0703 claim 8 and State and ordering (a file that cannot be opened: silence; the next update picks again; the entry stays the ended track)
                // spec: S-0703 claim 8a (a present file that does not parse as a playable sound: a failed start, with the same result as a missing file)
                self.state = State::Idle;
                self.log.push(Update::Failed(entry));
                None
            }
        }
    }

    /// Takes a read that finished since the last pass (call every frame).
    /// Returns the voice to play, if one is ready.
    pub fn poll(&mut self, output_rate: u32) -> Option<MusicVoice> {
        if !matches!(self.state, State::Track { t: None, .. }) {
            return None;
        }
        let load = self.source.poll();
        self.loaded(load, output_rate)
    }

    /// The music update at the end of an audio pass at logical time `c`
    /// (S-0703 claim 10), drawing from `rng`. Returns the voice to start
    /// (`Some(None)` silences the music, `None` leaves it as it is).
    pub fn update(
        &mut self,
        c: i64,
        rng: &mut Rng,
        half_span: bool,
        output_rate: u32,
    ) -> Option<Option<MusicVoice>> {
        // spec: S-0703 claim 9 (an empty list: no pick, no draw)
        if self.db.is_empty() {
            return None;
        }
        match self.phase {
            Phase::Armed => {
                // spec: S-0703 claim 10 (music started begins with claim 11's first start after loading, even when that pick starts nothing)
                self.phase = Phase::Started;
                if let Some(k) = self.theme {
                    // spec: S-0703 claim 11 (the map theme's entry is started first, directly, whether or not it is available, with no ranged draw; it becomes the ended track)
                    // spec: S-0703 claim 11 (our engine: the first start is made at the music update of the game's first audio pass)
                    self.ended = Some(k);
                    return Some(self.start(k, c, output_rate));
                }
                // spec: S-0703 claim 11 (otherwise the first track is chosen by an ordinary pick with no ended track)
            }
            Phase::Started => match self.state {
                // spec: S-0703 claim 10 (a track finishes at the first pass with C ≥ C0 + T)
                State::Track { c0, t: Some(t), .. } if c >= c0.saturating_add(t) => {}
                // A read under way, or a track still playing: nothing.
                State::Track { .. } => return None,
                // spec: S-0703 State and ordering (music started, no track playing: a pick)
                State::Idle => {}
            },
            // spec: S-0703 Terms and State and ordering (no pick before music has started; none with the in-game music setting off, claim 11; none at the end screen or in the menus, claims 12 and 13)
            Phase::Off | Phase::Loading | Phase::Score | Phase::Menu => return None,
        }
        // spec: S-0703 claim 3 and Edge cases (the pick at every music volume: the volume never adds or removes a draw)
        let next = next_track(
            &self.db,
            &self.available,
            self.ended,
            self.settings,
            self.quirks,
            rng,
            half_span,
        );
        let Some(k) = next else {
            // spec: S-0703 claim 7 (switch off, nothing picked: silence; the pick is made again at each update)
            self.state = State::Idle;
            self.log.push(Update::NoTrack);
            return Some(None);
        };
        // spec: S-0703 claim 4 (a started entry becomes the ended track whether or not its file opens; music stays started)
        self.ended = Some(k);
        // spec: S-0703 claim 10 (the track picked starts in the same pass; its logical duration counts from it)
        Some(self.start(k, c, output_rate))
    }

    /// Off the audio passes (the end screen, the menus, a loading game):
    /// the track started directly there is timed by the output, and
    /// `has_voice` says whether the output still plays it. Returns the
    /// voice to start (`Some(None)` silences, `None` leaves it as it is).
    pub fn screen_update(
        &mut self,
        has_voice: bool,
        output_rate: u32,
    ) -> Option<Option<MusicVoice>> {
        let State::Track {
            entry, t: Some(_), ..
        } = self.state
        else {
            // Nothing plays, or a read is under way.
            return None;
        };
        if has_voice {
            return None;
        }
        // spec: S-0703 claim 12 (our engine: on the end screen the score track has finished when its whole length has been played out; under the repeat setting or with its own repeat flag it starts again from its start at once, with no ranged draw; otherwise silence, no pick)
        let repeat = self.settings.repeat || self.db.entries.get(entry).is_some_and(|t| t.repeat);
        if self.phase == Phase::Score && repeat {
            return Some(self.start(entry, 0, output_rate));
        }
        // nocite: our own choice for the menu and loading tracks, which play once (how menu music goes on is open, Q-1700 question 7)
        self.state = State::Idle;
        Some(None)
    }
}

/// The music volume of our own settings, 0 … 16384, as a 16.16 gain.
pub fn music_gain(option: i32) -> i64 {
    // nocite: our own UI (the music volume scales the track linearly; S-0703 leaves volume scaling open, Q-1700 question 5)
    (1i64 << 16) * i64::from(option.clamp(0, FULL)) / i64::from(FULL)
}

#[cfg(test)]
#[path = "music_tests.rs"]
mod tests;
