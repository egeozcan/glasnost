//! Announcements: the announcement data (S-0701 claim 26) and the
//! one-at-a-time queue with its gap (claim 27).

use std::collections::{BTreeMap, VecDeque};

use glasnost_ini::Document;
use glasnost_ini::value::{self, COMMON_TEXT_LIMIT};

/// The gap: the next announcement needs more than this many ms since the
/// previous one started (S-0701 Constants).
pub const GAP_MS: i64 = 500; // spec: S-0701 claim 27 and Constants (more than 500 ms from the start of the previous one)
/// Characters of a sample file name kept (S-0701 claim 26).
pub const SAMPLE_NAME: usize = 8; // spec: S-0701 claim 26 (cut to 8 characters)

/// An announcement kind (S-0701 Terms).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LineKind {
    /// `STANDARD`, the default.
    #[default]
    Standard,
    /// `QUEUE`.
    Queued,
    /// `INTERRUPT`.
    Interrupt,
    /// `QUEUED_INTERRUPT`.
    QueuedInterrupt,
}

/// An announcement priority, 0 to 3 (S-0701 Terms).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum LinePriority {
    /// `LOW` (0).
    Low,
    /// `NORMAL` (1), the default.
    #[default]
    Normal,
    /// `IMPORTANT` (2).
    Important,
    /// `CRITICAL` (3).
    Critical,
}

/// One announcement (S-0701 claim 26).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Line {
    /// Its name as listed.
    pub name: String,
    /// Its kind.
    pub kind: LineKind,
    /// Its priority.
    pub priority: LinePriority,
    /// Sample per voice set: Allied, Soviet, Yuri.
    pub samples: [String; 3],
    /// The volume key, a fraction (read; used only with
    /// `quirks.announcement_volume_ignored` off).
    pub volume: Option<f64>,
    /// The text key's value, read as the caption text itself; `None` when
    /// the key is absent or its value empty (S-0701 claim 26).
    pub text: Option<String>,
}

impl Line {
    /// Our caption for this announcement (S-0701 claim 26): the text key
    /// when present with a nonempty value, otherwise the entry's name; the
    /// same for every voice set, with the whole value as read.
    pub fn caption(&self) -> &str {
        // spec: S-0701 claim 26 (the caption is the text key when present with a nonempty value, otherwise the entry's name; the same for every voice set)
        self.text.as_deref().unwrap_or(&self.name)
    }
}

/// An announcement, by position.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct LineId(pub u32);

/// The announcement data.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SpeechCatalog {
    lines: Vec<Line>,
    by_name: BTreeMap<String, LineId>,
}

fn text(b: &[u8]) -> String {
    b.iter().map(|&c| char::from(c)).collect()
}

impl SpeechCatalog {
    /// Reads the announcement data file (S-0701 claim 26).
    pub fn from_ini(doc: &Document) -> Self {
        let mut out = Self::default();
        let Some(list) = doc.section(b"DialogList") else {
            return out;
        };
        // spec: S-0701 claim 26 (a list section of announcement names; duplicates re-read the same entry)
        for (_, e) in list.list_positions() {
            let name = value::text(&e.value, COMMON_TEXT_LIMIT);
            if name.is_empty() {
                continue;
            }
            let key = text(name).to_ascii_uppercase();
            let id = *out.by_name.entry(key).or_insert_with(|| {
                out.lines.push(Line {
                    name: text(name),
                    ..Line::default()
                });
                LineId(out.lines.len() as u32 - 1)
            });
            let Some(s) = doc.section(name) else { continue };
            let line = &mut out.lines[id.0 as usize];
            // spec: S-0701 claim 26 and Import mapping (kind words in any case; an empty or unknown word keeps the value in force)
            if let Some(v) = s.value(b"Type") {
                match value::named(
                    v,
                    v.len(),
                    &["STANDARD", "QUEUE", "INTERRUPT", "QUEUED_INTERRUPT"],
                ) {
                    Some(0) => line.kind = LineKind::Standard,
                    Some(1) => line.kind = LineKind::Queued,
                    Some(2) => line.kind = LineKind::Interrupt,
                    Some(3) => line.kind = LineKind::QueuedInterrupt,
                    _ => {}
                }
            }
            if let Some(v) = s.value(b"Priority") {
                match value::named(v, v.len(), &["LOW", "NORMAL", "IMPORTANT", "CRITICAL"]) {
                    Some(0) => line.priority = LinePriority::Low,
                    Some(1) => line.priority = LinePriority::Normal,
                    Some(2) => line.priority = LinePriority::Important,
                    Some(3) => line.priority = LinePriority::Critical,
                    _ => {}
                }
            }
            // spec: S-0701 claim 26 (one sample per voice set, cut to 8 characters; the extension is added, see file_name)
            for (i, k) in [&b"Allied"[..], b"Russian", b"Yuri"].iter().enumerate() {
                if let Some(v) = s.value(k) {
                    let t = value::text(v, SAMPLE_NAME);
                    line.samples[i] = text(t);
                }
            }
            // spec: S-0701 claim 26 and Import mapping (the key spelled exactly `Text`, S-0001 claim 13; its value is the caption text itself, no length limit beyond S-0001 claim 3's line limit; an empty value counts as absent, S-0001 claim 11)
            if let Some(v) = s.value(b"Text").filter(|v| !v.is_empty()) {
                line.text = Some(text(v));
            }
            if let Some(v) = s.value(b"Volume")
                && let value::Decimal::Value { value, .. } =
                    value::decimal(v, value::DecimalField::Double)
            {
                line.volume = Some(match value.format {
                    glasnost_ini::binfloat::Format::Single => {
                        f64::from(f32::from_bits(value.bits as u32))
                    }
                    glasnost_ini::binfloat::Format::Double => f64::from_bits(value.bits),
                });
            }
        }
        out
    }

    /// Adds an announcement (tests and tools).
    pub fn insert(&mut self, line: Line) -> LineId {
        let id = LineId(self.lines.len() as u32);
        self.by_name.insert(line.name.to_ascii_uppercase(), id);
        self.lines.push(line);
        id
    }

    /// An announcement by name, any case.
    pub fn resolve(&self, name: &str) -> Option<LineId> {
        self.by_name.get(&name.to_ascii_uppercase()).copied()
    }

    /// An announcement.
    pub fn line(&self, id: LineId) -> &Line {
        &self.lines[id.0 as usize]
    }

    /// Every announcement, in list order.
    pub fn lines(&self) -> &[Line] {
        &self.lines
    }

    /// The files of one voice set's samples ([`file_name`]), each once, in
    /// list order. Empty samples are left out.
    pub fn files(&self, set: usize) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        for l in &self.lines {
            let s = &l.samples[set.min(2)];
            if s.is_empty() {
                continue;
            }
            let f = file_name(s);
            if !out.iter().any(|o| o.eq_ignore_ascii_case(&f)) {
                out.push(f);
            }
        }
        out
    }
}

/// The file an announcement sample is read from: its eight-character name
/// plus `.wav` (S-0701 claim 34 and Import mapping).
pub fn file_name(sample: &str) -> String {
    // spec: S-0701 claim 34 (the eight-character name plus the extension `.wav`)
    format!("{sample}.wav")
}

/// The voice set of a side index (S-0701 claim 26): the first side's set,
/// the second side's, and the third for any other side.
pub fn voice_set(side: usize) -> usize {
    // spec: S-0701 claim 26 (first side: Allied; second: Soviet; any other: Yuri)
    side.min(2)
}

/// The announcement queue (S-0701 claim 27).
#[derive(Clone, Debug, Default)]
pub struct Announcer {
    /// The announcement started last and when (audio clock, ms).
    current: Option<(LineId, i64)>,
    queued_interrupt: VecDeque<LineId>,
    critical: VecDeque<LineId>,
    standard: Option<LineId>,
    queued: [VecDeque<LineId>; 4],
    /// Muted (during a movie or the teleport screen effect).
    pub muted: bool,
}

/// What a request did to the playing announcement.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Requested {
    /// Nothing to do on the voice.
    Nothing,
    /// The playing announcement must stop (an interrupt).
    StopCurrent,
}

impl Announcer {
    fn waiting(&self, id: LineId) -> bool {
        self.queued_interrupt.contains(&id)
            || self.critical.contains(&id)
            || self.standard == Some(id)
            || self.queued.iter().any(|q| q.contains(&id))
    }

    fn flush(&mut self) {
        self.queued_interrupt.clear();
        self.critical.clear();
        self.standard = None;
        for q in &mut self.queued {
            q.clear();
        }
    }

    /// A request (claim 27). `playing` is whether the announcement voice
    /// is playing now.
    pub fn request(&mut self, cat: &SpeechCatalog, id: LineId, playing: bool) -> Requested {
        // spec: S-0701 claim 27 (ignored while muted, and when it is the announcement playing at that moment)
        if self.muted || (playing && self.current.is_some_and(|(c, _)| c == id)) {
            return Requested::Nothing;
        }
        let line = cat.line(id);
        let mut out = Requested::Nothing;
        if line.kind == LineKind::Interrupt && playing {
            // spec: S-0701 claim 27 (an interrupt stops the current announcement and empties every waiting list, only when there is a current one)
            self.flush();
            out = Requested::StopCurrent;
        }
        // spec: S-0701 claim 27 (an announcement already waiting with the same kind is not added again)
        if self.waiting(id) {
            return out;
        }
        match (line.kind, line.priority) {
            // spec: S-0701 claim 27 (queued interrupt requests wait in one list)
            (LineKind::QueuedInterrupt, _) => self.queued_interrupt.push_back(id),
            // spec: S-0701 claim 27 (queued requests wait in one list per priority)
            (LineKind::Queued, p) => self.queued[p as usize].push_back(id),
            // spec: S-0701 claim 27 (critical requests that are not queued wait in a second list)
            (_, LinePriority::Critical) => self.critical.push_back(id),
            // spec: S-0701 claim 27 (the single standard place: only if the first two lists are empty and it is empty or holds a strictly lower priority; otherwise dropped)
            (_, p) => {
                if self.queued_interrupt.is_empty()
                    && self.critical.is_empty()
                    && self.standard.is_none_or(|s| cat.line(s).priority < p)
                {
                    self.standard = Some(id);
                }
            }
        }
        out
    }

    /// The announcement to start now, if any (claim 27): the previous one has
    /// finished (`playing` false) and more than 500 ms have passed since it
    /// started.
    pub fn next(&mut self, now_ms: i64, playing: bool) -> Option<LineId> {
        if playing || self.current.is_some_and(|(_, t)| now_ms - t <= GAP_MS) {
            return None;
        }
        // spec: S-0701 claim 27 (queued-interrupt list, critical list, standard place, then queued lists from critical down to low; taking from the first two discards the standard place)
        let id = if let Some(id) = self.queued_interrupt.pop_front() {
            self.standard = None;
            id
        } else if let Some(id) = self.critical.pop_front() {
            self.standard = None;
            id
        } else if let Some(id) = self.standard.take() {
            id
        } else {
            self.queued.iter_mut().rev().find_map(VecDeque::pop_front)?
        };
        self.current = Some((id, now_ms));
        Some(id)
    }

    /// Forgets everything (a new game).
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glasnost_ini::Quirks;

    fn cat() -> SpeechCatalog {
        let t = "[DialogList]\n0=Std\n1=Que\n2=Int\n3=QInt\n4=Crit\n5=Low\n6=Std\n\n\
                 [Std]\nType=bogus\nAllied=AlliedLongName\nRussian=sov\nYuri=yur\nVolume=0.5\n\n\
                 [Que]\nType=queue\nPriority=low\n\n[Int]\nType=INTERRUPT\n\n\
                 [QInt]\nType=QUEUED_INTERRUPT\n\n[Crit]\nPriority=CRITICAL\n\n[Low]\nPriority=LOW\n";
        SpeechCatalog::from_ini(&Document::parse(
            t.as_bytes(),
            Quirks::default(),
            &mut Vec::new(),
        ))
    }

    // spec: S-0701 claim 26 (kinds and priorities in any case; unknown words keep the default; samples cut to 8; voice set by side)
    #[test]
    fn claim_26_announcement_data() {
        let c = cat();
        let s = c.line(c.resolve("std").unwrap());
        assert_eq!(
            (s.kind, s.priority),
            (LineKind::Standard, LinePriority::Normal)
        );
        assert_eq!(s.samples, ["AlliedLo", "sov", "yur"]);
        let q = c.line(c.resolve("Que").unwrap());
        assert_eq!((q.kind, q.priority), (LineKind::Queued, LinePriority::Low));
        assert_eq!(
            c.line(c.resolve("QInt").unwrap()).kind,
            LineKind::QueuedInterrupt
        );
        assert_eq!(
            c.resolve("std"),
            Some(LineId(0)),
            "a duplicate is one entry"
        );
        assert_eq!(
            [voice_set(0), voice_set(1), voice_set(2), voice_set(5)],
            [0, 1, 2, 2]
        );
    }

    // spec: S-0701 claim 26 and Import mapping (the caption: the `Text` key's value as read, spelled exactly so; an empty value or a differently spelled key counts as absent and the entry's name is the caption; the same for every voice set)
    #[test]
    fn claim_26_caption_from_the_text_key() {
        let long = "w".repeat(300);
        let t = format!(
            "[DialogList]\n0=Lit\n1=Empty\n2=None\n3=Lower\n4=Long\n\n\
             [Lit]\nText=Some made-up words, with a comma\nAllied=a\nRussian=b\n\n\
             [Empty]\nText=\n\n[None]\nType=QUEUE\n\n[Lower]\ntext=lower case key\n\n\
             [Long]\nText={long}\n"
        );
        let c = SpeechCatalog::from_ini(&Document::parse(
            t.as_bytes(),
            Quirks::default(),
            &mut Vec::new(),
        ));
        let cap = |n| c.line(c.resolve(n).unwrap()).caption().to_owned();
        assert_eq!(cap("Lit"), "Some made-up words, with a comma");
        assert_eq!(c.line(c.resolve("Empty").unwrap()).text, None);
        assert_eq!(cap("Empty"), "Empty", "an empty value counts as absent");
        assert_eq!(cap("None"), "None", "absent: the entry's name");
        assert_eq!(cap("Lower"), "Lower", "the key is spelled exactly `Text`");
        assert_eq!(cap("Long"), long, "no length limit of its own");
    }

    // spec: S-0701 claim 27 (one at a time; more than 500 ms from the previous start; the standard place; lists in order)
    #[test]
    fn claim_27_queue_and_gap() {
        let c = cat();
        let id = |n| c.resolve(n).unwrap();
        let mut a = Announcer::default();
        a.request(&c, id("Std"), false);
        assert_eq!(a.next(0, false), Some(id("Std")));
        // Playing: the same one is ignored; a standard one takes the empty place.
        a.request(&c, id("Std"), true);
        a.request(&c, id("Low"), true);
        // A second standard one of the same priority does not replace it.
        a.request(&c, id("Low"), true);
        a.request(&c, id("Que"), true);
        assert_eq!(a.next(100, true), None, "still playing");
        assert_eq!(a.next(500, false), None, "500 ms is not more than 500 ms");
        assert_eq!(a.next(501, false), Some(id("Low")));
        assert_eq!(a.next(1100, false), Some(id("Que")));
        assert_eq!(a.next(1700, false), None);
        // Critical and queued-interrupt lists come first and discard the standard place.
        a.request(&c, id("Std"), false);
        a.request(&c, id("Crit"), false);
        a.request(&c, id("QInt"), false);
        a.request(&c, id("Std"), false);
        assert_eq!(a.next(2300, false), Some(id("QInt")));
        assert_eq!(a.next(2900, false), Some(id("Crit")));
        assert_eq!(
            a.next(3500, false),
            None,
            "the standard place was discarded"
        );
        // A standard request while the critical list holds one is dropped.
        a.request(&c, id("Crit"), false);
        a.request(&c, id("Low"), false);
        assert_eq!(a.next(4100, false), Some(id("Crit")));
        assert_eq!(a.next(4700, false), None);
    }

    // spec: S-0701 claim 27 (an interrupt stops the current one and empties the lists only when one plays)
    #[test]
    fn claim_27_interrupt() {
        let c = cat();
        let id = |n| c.resolve(n).unwrap();
        let mut a = Announcer::default();
        a.request(&c, id("Que"), false);
        assert_eq!(a.request(&c, id("Int"), false), Requested::Nothing);
        assert_eq!(a.next(0, false), Some(id("Int")));
        a.request(&c, id("Que"), true);
        assert_eq!(
            a.request(&c, id("Int"), true),
            Requested::Nothing,
            "the one playing is ignored"
        );
        a.request(&c, id("Crit"), true);
        assert_eq!(a.next(600, false), Some(id("Crit")));
        assert_eq!(a.request(&c, id("Int"), true), Requested::StopCurrent);
        assert_eq!(
            a.next(1200, false),
            Some(id("Int")),
            "the queued lines were flushed"
        );
        assert_eq!(a.next(1800, false), None);
        a.muted = true;
        a.request(&c, id("Std"), false);
        assert_eq!(a.next(2400, false), None, "muted requests are ignored");
    }
}
