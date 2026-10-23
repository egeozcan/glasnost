//! Tests of the music database import, the next-track rule and the music
//! player (S-0703), on synthetic databases and synthetic tones. Each test
//! names the claim or scenario part (MU1 to MU7) whose engine fixture it is.

use super::*;
use crate::catalog::{SoundCatalog, SoundEntry};
use crate::output::NullOutput;
use crate::{Audio, Where};
use glasnost_fmt::pcm::Pcm;
use glasnost_rules::quirks::Quirks;
use glasnost_sim::testkit;

const RATE: u32 = 8000;
/// The audio pass cadence the player tests use (any interval above 33 ms).
const PASS: i64 = 40;

fn doc(text: &str) -> Document {
    Document::parse(text.as_bytes(), Default::default(), &mut Vec::new())
}

fn db(text: &str) -> MusicDatabase {
    MusicDatabase::from_ini(&doc(text))
}

fn ids(d: &MusicDatabase) -> Vec<&str> {
    d.entries.iter().map(|t| t.id.as_str()).collect()
}

/// Entries T0 … T(n−1), each section setting only its sound file name.
fn plain(n: usize) -> String {
    let mut s = String::from("[Themes]\n");
    for k in 0..n {
        s += &format!("{k}=T{k}\n");
    }
    for k in 0..n {
        s += &format!("[T{k}]\nSound=tone{k}\n");
    }
    s
}

/// A synthetic tone of `frames` frames at 8000 Hz as a PCM WAVE file.
fn tone(frames: usize, seed: u64) -> Vec<u8> {
    Pcm::synthetic_tone(RATE, 1, frames, 40, 8000, seed).to_wav()
}

fn on() -> MusicQuirks {
    MusicQuirks::default()
}

fn off() -> MusicQuirks {
    MusicQuirks {
        fallback_entry_zero: false,
    }
}

const LIST: MusicSettings = MusicSettings {
    shuffle: false,
    repeat: false,
};
const SHUFFLE: MusicSettings = MusicSettings {
    shuffle: true,
    repeat: false,
};

fn pick(
    d: &MusicDatabase,
    a: &[bool],
    e: Option<usize>,
    s: MusicSettings,
    q: MusicQuirks,
    rng: &mut Rng,
) -> Option<usize> {
    next_track(d, a, e, s, q, rng, true)
}

// ---------------------------------------------------------------- import

// spec: S-0703 claim 1 and Import mapping (MU1 part a: the defaults of Normal, Repeat and Side; list order)
#[test]
fn mu1a_defaults_and_order() {
    let d = db(&plain(5));
    assert_eq!(ids(&d), ["T0", "T1", "T2", "T3", "T4"]);
    for (k, t) in d.entries.iter().enumerate() {
        assert_eq!(t.file.as_deref(), Some(format!("tone{k}.wav").as_str()));
        assert!(t.normal);
        assert!(!t.repeat);
        assert_eq!(t.side, None);
    }
}

// spec: S-0703 claim 1 and Import mapping (MU1 part c: an identifier listed again, in the same or another case, is one entry at its first listing; a repeated key gives its winning occurrence)
#[test]
fn mu1c_repeats() {
    let base = plain(5);
    let d = db(&base.replacen("[T0]", "5=T1\n[T0]", 1));
    assert_eq!(ids(&d), ["T0", "T1", "T2", "T3", "T4"]);
    let d = db(&base.replacen("[T0]", "5=t1\n[T0]", 1));
    assert_eq!(ids(&d), ["T0", "T1", "T2", "T3", "T4"]);
    // Third run: T4's line under T1's key. The winning occurrence's value
    // stands at both positions, and the repeat keeps the first.
    let d = db(&base.replacen("4=T4", "1=T4", 1));
    assert_eq!(d.len(), 4);
    assert!(
        ids(&d)
            .iter()
            .filter(|i| **i == "T1" || **i == "T4")
            .count()
            == 1
    );
}

// spec: S-0703 claim 1a (MU1 part e: leading $ and # in any order and number are removed; the same characters inside the name are kept)
#[test]
fn mu1e_removed_prefix() {
    for prefix in ["$#", "#", "$$", "##$", "#$#$"] {
        assert_eq!(
            sound_file(format!("{prefix}tone4").as_bytes()).as_deref(),
            Some("tone4.wav")
        );
    }
    assert_eq!(sound_file(b"to$#ne4").as_deref(), Some("to$#ne4.wav"));
}

// spec: S-0703 Import mapping (MU1 part f: the identifier is cut to 31 bytes and its section is found by the cut text)
#[test]
fn mu1f_identifier_limit() {
    let long = "A".repeat(32);
    let cut = &long[..31];
    let d = db(&format!("[Themes]\n0={long}\n[{long}]\nSound=x\n"));
    assert_eq!(ids(&d), [cut]);
    assert_eq!(
        d.entries[0].file, None,
        "a section named by all 32 bytes is not found"
    );
    let d = db(&format!("[Themes]\n0={long}\n[{cut}]\nSound=x\n"));
    assert_eq!(d.entries[0].file.as_deref(), Some("x.wav"));
}

// spec: S-0703 claim 1a and Import mapping (MU1 part g: no section of its own, an empty sound file name, or one made only of $ and #: never present)
#[test]
fn mu1g_no_file() {
    let d = db("[Themes]\n0=T5\n");
    assert_eq!(d.entries[0], TrackEntry::defaults("T5".into()));
    assert_eq!(d.entries[0].file, None);
    let d = db("[Themes]\n0=T5\n[T5]\nSound=\nNormal=yes\n");
    assert_eq!(d.entries[0].file, None);
    let d = db("[Themes]\n0=T5\n[T5]\nSound=$#$\n");
    assert_eq!(d.entries[0].file, None);
}

// spec: S-0703 Import mapping (MU1 part i: the sound file name is cut to 255 bytes and its trailing whitespace removed)
#[test]
fn mu1i_sound_limit() {
    let name = "tone4";
    let v256 = format!("{name}{}X", " ".repeat(256 - name.len() - 1));
    assert_eq!(v256.len(), 256);
    assert_eq!(sound_file(v256.as_bytes()).as_deref(), Some("tone4.wav"));
    let v255 = &v256[..254];
    let v255 = format!("{v255}X");
    assert_eq!(v255.len(), 255);
    assert_ne!(sound_file(v255.as_bytes()).as_deref(), Some("tone4.wav"));
}

// spec: S-0703 Import mapping (MU1 part j: the side restriction is cut to 127 bytes and its trailing whitespace removed; MU1 part b third run: empty counts as absent)
#[test]
fn mu1j_side_limit_and_empty() {
    let y = "Alpha";
    let long = format!("{y}{}X", " ".repeat(128 - y.len() - 1));
    let short = format!("{}X", &long[..126]);
    let d = db(&format!(
        "[Themes]\n0=T1\n1=T4\n2=T5\n[T1]\nSide={y}\n[T4]\nSide={long}\n[T5]\nSide={short}\n"
    ));
    assert_eq!(d.entries[0].side.as_deref(), Some(y));
    assert_eq!(d.entries[1].side.as_deref(), Some(y));
    assert_ne!(d.entries[2].side.as_deref(), Some(y));
    let d = db("[Themes]\n0=T1\n[T1]\nSide=\n");
    assert_eq!(d.entries[0].side, None);
}

// spec: S-0703 Import mapping (MU6 part a, second to fourth runs: an empty value adds no entry; none and <none> are ordinary entries)
#[test]
fn mu6a_empty_none_values() {
    let base = plain(5);
    assert_eq!(db(&base.replacen("[T0]", "5=\n[T0]", 1)).len(), 5);
    for v in ["none", "<none>"] {
        let d = db(&base.replacen("[T0]", &format!("5={v}\n[T0]"), 1));
        assert_eq!(d.len(), 6);
        assert_eq!(d.entries[5].id, v);
        assert_eq!(d.entries[5].file, None);
    }
}

// spec: S-0703 Import mapping (Normal and Repeat booleans of S-0003; a malformed value keeps the default)
#[test]
fn flags_are_booleans() {
    let d = db("[Themes]\n0=A\n1=B\n[A]\nNormal=no\nRepeat=yes\n[B]\nNormal=maybe\nRepeat=?\n");
    assert!(!d.entries[0].normal && d.entries[0].repeat);
    assert!(d.entries[1].normal && !d.entries[1].repeat);
}

// spec: S-0703 Import mapping (the settings file's [Audio] IsScoreShuffle and IsScoreRepeat)
#[test]
fn settings_from_the_settings_file() {
    let s = MusicSettings::from_ini(Some(&doc("[Audio]\nIsScoreShuffle=yes\nIsScoreRepeat=1\n")));
    assert_eq!(
        s,
        MusicSettings {
            shuffle: true,
            repeat: true
        }
    );
    // Q-1831 (interim: off when the file or the key is missing, or the value malformed)
    assert_eq!(MusicSettings::from_ini(None), MusicSettings::default());
    let s = MusicSettings::from_ini(Some(&doc("[Audio]\nIsScoreShuffle=what\n")));
    assert!(!s.shuffle && !s.repeat);
}

// ---------------------------------------------------------- availability

fn sides() -> Vec<String> {
    ["Alpha", "Beta", "Gamma"].map(String::from).to_vec()
}

// spec: S-0703 claim 2 (MU1 part a: a missing file and a clear normal flag make an entry unavailable)
#[test]
fn mu1a_availability() {
    let d = db(&plain(5).replacen("[T3]\nSound=tone3", "[T3]\nSound=tone3\nNormal=no", 1));
    let present = [true, true, false, true, true];
    let a = d.availability(&present, &sides(), Some(0));
    assert_eq!(a, [true, true, false, false, true]);
}

// spec: S-0703 claim 2 and Edge cases (MU1 part b's engine fixture: a restriction matches the local player's side by name ignoring case, for a side whose index is not 0; a name of no side warns and never plays)
#[test]
fn mu1b_side_restriction() {
    let d = db(&plain(5).replacen("[T1]\nSound=tone1", "[T1]\nSound=tone1\nSide=beta", 1));
    let present = [true, true, false, true, true];
    assert!(d.availability(&present, &sides(), Some(1))[1]);
    assert!(!d.availability(&present, &sides(), Some(0))[1]);
    assert!(d.side_warnings(&sides()).is_empty());
    // Second run: a restriction naming no side of the rules data.
    let d = db(&plain(5).replacen("[T1]\nSound=tone1", "[T1]\nSound=tone1\nSide=Delta", 1));
    let w = d.side_warnings(&sides());
    assert_eq!(w.len(), 1);
    assert!(w[0].contains("T1") && w[0].contains("Delta"), "{}", w[0]);
    for side in 0..3 {
        assert!(!d.availability(&present, &sides(), Some(side))[1]);
    }
    // A side that only a game layer adds counts at that game's start.
    let mut more = sides();
    more.push("delta".into());
    assert!(d.side_warnings(&more).is_empty());
    assert!(d.availability(&present, &more, Some(3))[1]);
}

// ------------------------------------------------------------ next track

// spec: S-0703 claims 3 and 5 (MU1 part a: list order passes over unavailable entries at the pick and wraps; MU6 part e: no ranged draw)
#[test]
fn mu1a_list_order_cycle() {
    let d = db(&plain(5));
    let a = [true, true, false, false, true];
    let mut rng = Rng::new(9);
    let mut e = None;
    let mut heard = Vec::new();
    for _ in 0..12 {
        e = pick(&d, &a, e, LIST, on(), &mut rng);
        heard.push(e.unwrap());
    }
    assert_eq!(heard, [0, 1, 4, 0, 1, 4, 0, 1, 4, 0, 1, 4]);
    assert_eq!(rng.taken(), 0, "list order makes no draw");
}

// spec: S-0703 claims 5 and 7 (MU4 part a: list order with no available entry; switch on starts entry 0 again and again, switch off no track and no draw)
#[test]
fn mu4a_list_order_without_available_entry() {
    let d = db(&plain(3));
    let a = [false, false, false];
    let mut rng = Rng::new(1);
    for e in [None, Some(0), Some(2)] {
        assert_eq!(pick(&d, &a, e, LIST, on(), &mut rng), Some(0));
        assert_eq!(pick(&d, &a, e, LIST, off(), &mut rng), None);
    }
    assert_eq!(rng.taken(), 0);
}

// spec: S-0703 claim 5 (MU4 part c: the ended track is the last candidate; entry 0 is never the fallback while it is available)
#[test]
fn mu4c_ended_track_last_candidate() {
    let d = db(&plain(3));
    let a = [false, true, false];
    let mut rng = Rng::new(1);
    for q in [on(), off()] {
        assert_eq!(pick(&d, &a, Some(1), LIST, q, &mut rng), Some(1));
        assert_eq!(pick(&d, &a, None, LIST, q, &mut rng), Some(1));
    }
}

/// The draws a shuffle pick makes, predicted from a copy of the stream.
fn predict(rng: &Rng, n: usize, e: Option<usize>, a: &[bool]) -> (usize, u64) {
    let mut r = rng.clone();
    let start = r.taken();
    loop {
        let x = r.ranged(0, n as i32 - 1, true) as usize;
        if Some(x) != e && a[x] {
            return (x, r.taken() - start);
        }
    }
}

// spec: S-0703 claim 6 (MU6 part a: ranged draws R(0, n − 1) over all entries, the rejected ones included, up to the first that differs from the ended track and names an available track; with n = 6 for none and <none>)
#[test]
fn mu6a_accepted_draws() {
    for n in [5usize, 6] {
        let mut a = vec![true, true, true, false, false];
        a.resize(n, false);
        let d = MusicDatabase {
            entries: (0..n)
                .map(|k| TrackEntry::defaults(format!("T{k}")))
                .collect(),
        };
        let mut rng = Rng::new(17);
        let mut e = None;
        for _ in 0..20 {
            let (want, raws) = predict(&rng, n, e, &a);
            let before = rng.taken();
            let got = pick(&d, &a, e, SHUFFLE, on(), &mut rng);
            assert_eq!(got, Some(want));
            assert_eq!(rng.taken() - before, raws);
            e = got;
        }
    }
}

// spec: S-0703 claim 6 (MU2 part a: no immediate repeat; with exactly two available tracks the picks alternate)
#[test]
fn mu2a_no_immediate_repeat() {
    let d = db(&plain(3));
    let mut rng = Rng::new(3);
    let mut e = None;
    let mut seen = [false; 3];
    for _ in 0..60 {
        let next = pick(&d, &[true; 3], e, SHUFFLE, on(), &mut rng);
        assert_ne!(next, e);
        seen[next.unwrap()] = true;
        e = next;
    }
    assert_eq!(seen, [true; 3]);
    let a = [true, true, false];
    for _ in 0..30 {
        let next = pick(&d, &a, e, SHUFFLE, on(), &mut rng).unwrap();
        if let Some(prev) = e {
            assert_eq!(next, 1 - prev.min(1));
        }
        e = Some(next);
    }
}

// spec: S-0703 claim 6 (MU2 part b: a uniform choice among the other available tracks)
#[test]
fn mu2b_uniform_choice() {
    let d = db(&plain(4));
    let mut rng = Rng::new(77);
    let mut counts = [[0u32; 4]; 4];
    let mut e = Some(0);
    for _ in 0..1200 {
        let next = pick(&d, &[true; 4], e, SHUFFLE, on(), &mut rng).unwrap();
        counts[e.unwrap()][next] += 1;
        e = Some(next);
    }
    for (from, row) in counts.iter().enumerate() {
        let total: u32 = row.iter().sum();
        assert!(total >= 60, "{from}: {total}");
        for (to, &c) in row.iter().enumerate() {
            if to == from {
                assert_eq!(c, 0);
            } else {
                assert!(
                    c * 8 >= total && c * 8 <= total * 5,
                    "{from}->{to}: {c} of {total}"
                );
            }
        }
    }
}

// spec: S-0703 claims 6 and 7 (MU6 part b: a pick that accepts none makes exactly 1000 ranged draws, then entry 0; switch off: no draw and the ended track again)
#[test]
fn mu6b_fallback_count() {
    let d = db(&plain(4));
    let a = [false, true, false, false];
    let mut rng = Rng::new(5);
    let before = rng.taken();
    assert_eq!(pick(&d, &a, Some(1), SHUFFLE, on(), &mut rng), Some(0));
    // R(0, 3) takes exactly one raw value per draw (S-0202 claim 4).
    assert_eq!(rng.taken() - before, 1000);
    // After entry 0, the draws up to the first that gives T1.
    let (want, raws) = predict(&rng, 4, Some(0), &a);
    let before = rng.taken();
    assert_eq!(pick(&d, &a, Some(0), SHUFFLE, on(), &mut rng), Some(want));
    assert_eq!(want, 1);
    assert_eq!(rng.taken() - before, raws);
    // Switch off.
    let before = rng.taken();
    assert_eq!(pick(&d, &a, Some(1), SHUFFLE, off(), &mut rng), Some(1));
    assert_eq!(rng.taken(), before);
    // Switch off, no available track: no track, no draw.
    assert_eq!(
        pick(&d, &[false; 4], Some(1), SHUFFLE, off(), &mut rng),
        None
    );
    assert_eq!(rng.taken(), before);
}

// spec: S-0703 claim 6 (a single entry: R(0, 0) takes no raw value; the 1000th draw is never accepted)
#[test]
fn single_entry_shuffle() {
    let d = db(&plain(1));
    let mut rng = Rng::new(5);
    assert_eq!(pick(&d, &[true], None, SHUFFLE, on(), &mut rng), Some(0));
    assert_eq!(pick(&d, &[true], Some(0), SHUFFLE, on(), &mut rng), Some(0));
    assert_eq!(
        pick(&d, &[true], Some(0), SHUFFLE, off(), &mut rng),
        Some(0)
    );
    assert_eq!(rng.taken(), 0);
}

// spec: S-0703 claim 4 (MU6 part d: repeat by the entry's flag or by the setting picks the ended track, whatever the shuffle setting and its availability, with no ranged draw)
#[test]
fn mu6d_repeat_makes_no_draw() {
    let d = db(&plain(5).replacen("[T1]\nSound=tone1", "[T1]\nSound=tone1\nRepeat=yes", 1));
    let a = [true, true, true, false, false];
    let mut rng = Rng::new(8);
    for s in [LIST, SHUFFLE] {
        for q in [on(), off()] {
            assert_eq!(pick(&d, &a, Some(1), s, q, &mut rng), Some(1));
            // A track that is not available repeats too.
            assert_eq!(pick(&d, &[false; 5], Some(1), s, q, &mut rng), Some(1));
            let rep = MusicSettings { repeat: true, ..s };
            assert_eq!(pick(&d, &a, Some(2), rep, q, &mut rng), Some(2));
        }
    }
    assert_eq!(rng.taken(), 0);
    // With no ended track, repeat does not apply.
    let rep = MusicSettings {
        repeat: true,
        shuffle: false,
    };
    assert_eq!(pick(&d, &a, None, rep, on(), &mut rng), Some(0));
}

// spec: S-0703 claim 9 (MU5's engine fixture: an empty list gives no pick and no ranged draw with either switch and every setting)
#[test]
fn mu5_empty_list() {
    let d = db("[Themes]\n[Other]\nA=1\n");
    assert!(d.is_empty());
    let mut rng = Rng::new(2);
    for shuffle in [false, true] {
        for repeat in [false, true] {
            for q in [on(), off()] {
                let s = MusicSettings { shuffle, repeat };
                for e in [None, Some(0)] {
                    assert_eq!(pick(&d, &[], e, s, q, &mut rng), None);
                }
                let mut m = Music::new(d.clone(), vec![], s, Box::new(MemoryTracks::new()));
                m.set_quirks(q);
                m.start_game(&sides(), Some(0));
                for p in 0..50 {
                    assert!(m.update(p * PASS, &mut rng, true, RATE).is_none());
                }
            }
        }
    }
    assert_eq!(rng.taken(), 0);
}

// ---------------------------------------------------------------- player

/// A player over `text` with tone files for the entries in `files` (entry,
/// frames).
fn player(text: &str, files: &[(usize, usize)], s: MusicSettings, q: MusicQuirks) -> Music {
    let d = db(text);
    let mut t = MemoryTracks::new();
    for &(k, frames) in files {
        t.insert(k, tone(frames, k as u64));
    }
    let present = t.present(d.len());
    let mut m = Music::new(d, present, s, Box::new(t));
    m.set_quirks(q);
    m.start_game(&sides(), Some(0));
    m
}

/// Runs `passes` music updates at the pass cadence from `from`; returns
/// the update log.
fn run(m: &mut Music, rng: &mut Rng, from: i64, passes: i64) -> Vec<Update> {
    for p in 0..passes {
        m.update(from + p * PASS, rng, true, RATE);
    }
    m.take_log()
}

fn started(log: &[Update]) -> Vec<usize> {
    log.iter()
        .filter_map(|u| match u {
            Update::Started(k) => Some(*k),
            _ => None,
        })
        .collect()
}

// spec: S-0703 claim 10 (MU7: a track completes at the first pass with C ≥ C0 + T, T = max(1, ⌈100000 × N / (f × 100)⌉); the next track starts in that pass)
#[test]
fn mu7_completion_pass() {
    assert_eq!(logical_ms(800, 8000), 100);
    assert_eq!(logical_ms(801, 8000), 101);
    assert_eq!(logical_ms(0, 8000), 1);
    assert_eq!(logical_ms(1, 22050), 1);
    assert_eq!(logical_ms(22051, 22050), 1001);
    // T0: 801 frames, T = 101 ms; passes every 40 ms from 0.
    let mut m = player(&plain(2), &[(0, 801), (1, 800)], LIST, on());
    let mut rng = Rng::new(1);
    m.update(0, &mut rng, true, RATE);
    assert_eq!(m.take_log(), [Update::Started(0)]);
    for c in [40, 80] {
        assert!(m.update(c, &mut rng, true, RATE).is_none());
    }
    // 120 ≥ 0 + 101: T1 starts at 120 with T = 100 and completes at the first pass with C ≥ 220.
    assert!(m.update(120, &mut rng, true, RATE).is_some());
    assert_eq!(m.take_log(), [Update::Started(1)]);
    for c in [160, 200] {
        assert!(m.update(c, &mut rng, true, RATE).is_none());
    }
    assert!(m.update(240, &mut rng, true, RATE).is_some());
    assert_eq!(m.take_log(), [Update::Started(0)]);
}

// spec: S-0703 claims 8 and 10 (MU7 second run and MU1 part h: a file unreadable after startup is still picked; nothing plays; the next pick comes at the following pass; it stays the ended track)
#[test]
fn mu7_failed_start_picks_at_the_following_pass() {
    let mut m = player(&plain(5), &[(0, 800), (1, 800), (4, 800)], LIST, on());
    // T2's file was absent at startup, T1's goes after it.
    let mut rng = Rng::new(1);
    m.update(0, &mut rng, true, RATE);
    assert_eq!(m.take_log(), [Update::Started(0)]);
    // Replace the source by one without T1.
    let mut t = MemoryTracks::new();
    t.insert(0, tone(800, 0));
    t.insert(4, tone(800, 4));
    m.source = Box::new(t);
    // T0 ends at 120: T1 is picked (present at startup) and fails.
    assert!(matches!(m.update(120, &mut rng, true, RATE), Some(None)));
    assert_eq!(m.take_log(), [Update::Failed(1)]);
    assert_eq!(m.ended(), Some(1));
    assert_eq!(m.playing(), None);
    // The following pass picks T4 (list order from the failed T1: T2 and T3 are not available).
    assert!(m.update(160, &mut rng, true, RATE).is_some());
    assert_eq!(m.take_log(), [Update::Started(4)]);
}

// spec: S-0703 claims 4 and 8 (MU3 part d's engine analog: a repeat-flagged track whose file went after startup is started again at each update and fails each time; with its file back it plays)
#[test]
fn mu3d_failed_repeat_track_retries() {
    let text = plain(3).replacen("[T1]\nSound=tone1", "[T1]\nSound=tone1\nRepeat=yes", 1);
    let mut m = player(&text, &[(0, 800), (1, 800), (2, 800)], LIST, on());
    let mut rng = Rng::new(1);
    let log = run(&mut m, &mut rng, 0, 4);
    assert_eq!(started(&log), [0, 1]);
    let mut t = MemoryTracks::new();
    t.insert(0, tone(800, 0));
    t.insert(2, tone(800, 2));
    m.source = Box::new(t);
    let log = run(&mut m, &mut rng, 160, 10);
    assert!(log.iter().all(|u| *u == Update::Failed(1)), "{log:?}");
    assert!(log.len() >= 8);
    let mut t = MemoryTracks::new();
    t.insert(1, tone(800, 1));
    m.source = Box::new(t);
    let log = run(&mut m, &mut rng, 560, 10);
    assert_eq!(started(&log)[0], 1);
    assert!(started(&log).iter().all(|&k| k == 1));
}

// spec: S-0703 claims 6, 7 and 4 (MU4 part b's engine fixture: with the switch off, T1 over and over; with it on, T1 and T0 alternate, and with T0's file missing T1 follows the failed start)
#[test]
fn mu4b_shuffle_fallback() {
    let text = plain(3)
        .replacen("[T0]\nSound=tone0", "[T0]\nSound=tone0\nNormal=no", 1)
        .replacen("[T2]\nSound=tone2", "[T2]\nSound=tone2\nNormal=no", 1);
    let mut rng = Rng::new(4);
    let mut m = player(&text, &[(0, 800), (1, 800), (2, 800)], SHUFFLE, off());
    let log = run(&mut m, &mut rng, 0, 40);
    assert!(started(&log).iter().all(|&k| k == 1), "{log:?}");
    let mut m = player(&text, &[(0, 800), (1, 800), (2, 800)], SHUFFLE, on());
    let heard = started(&run(&mut m, &mut rng, 0, 40));
    assert!(heard.windows(2).all(|w| w[0] != w[1]), "{heard:?}");
    assert!(heard.iter().all(|&k| k < 2));
    // T0's file missing: T1, then the fallback's failed start of T0, then T1.
    let mut m = player(&text, &[(1, 800), (2, 800)], SHUFFLE, on());
    let log = run(&mut m, &mut rng, 0, 40);
    let firsts: Vec<Update> = log.iter().copied().take(4).collect();
    assert_eq!(
        firsts,
        [
            Update::Started(1),
            Update::Failed(0),
            Update::Started(1),
            Update::Failed(0)
        ]
    );
}

// spec: S-0703 claims 4 and 7 (MU3 part c's engine fixture: with the switch off, T1 over and over; with it on, T0 follows T1 and, its repeat flag set, keeps playing although it is not available)
#[test]
fn mu3c_repeat_after_fallback() {
    let text = plain(3)
        .replacen(
            "[T0]\nSound=tone0",
            "[T0]\nSound=tone0\nNormal=no\nRepeat=yes",
            1,
        )
        .replacen("[T2]\nSound=tone2", "[T2]\nSound=tone2\nNormal=no", 1);
    let mut rng = Rng::new(6);
    let files = [(0, 800), (1, 800), (2, 800)];
    let mut m = player(&text, &files, SHUFFLE, off());
    assert!(
        started(&run(&mut m, &mut rng, 0, 30))
            .iter()
            .all(|&k| k == 1)
    );
    let mut m = player(&text, &files, SHUFFLE, on());
    let heard = started(&run(&mut m, &mut rng, 0, 30));
    assert_eq!(heard[..2], [1, 0]);
    assert!(heard[1..].iter().all(|&k| k == 0));
}

// spec: S-0703 claim 7 (switch off, no available entry: no track starts, the pick is made again at each update, silence throughout)
#[test]
fn switch_off_no_track_repicks() {
    let mut m = player(&plain(3), &[], LIST, off());
    let mut rng = Rng::new(1);
    let log = run(&mut m, &mut rng, 0, 10);
    assert_eq!(log, [Update::NoTrack; 10]);
    assert_eq!(rng.taken(), 0);
}

// spec: S-0703 Terms and State and ordering (before music has started, no update picks)
#[test]
fn no_pick_before_music_starts() {
    let d = db(&plain(3));
    let mut t = MemoryTracks::new();
    t.insert(0, tone(800, 0));
    let present = t.present(3);
    let mut m = Music::new(d, present, SHUFFLE, Box::new(t));
    let mut rng = Rng::new(1);
    for p in 0..10 {
        assert!(m.update(p * PASS, &mut rng, true, RATE).is_none());
    }
    assert_eq!(rng.taken(), 0);
    m.start_game(&sides(), Some(0));
    m.stop();
    assert!(m.update(0, &mut rng, true, RATE).is_none());
}

/// A source whose reads finish only when told (the web's file reads).
struct Slow {
    files: MemoryTracks,
    pending: Option<usize>,
    ready: std::rc::Rc<std::cell::Cell<bool>>,
}

impl TrackSource for Slow {
    fn open(&mut self, k: usize) -> Load {
        self.pending = Some(k);
        self.ready.set(false);
        Load::Pending
    }
    fn poll(&mut self) -> Load {
        match self.pending {
            Some(k) if self.ready.get() => {
                self.pending = None;
                self.files.open(k)
            }
            _ => Load::Pending,
        }
    }
}

// spec: S-0703 claim 10 (the logical duration counts from the pass at which the track started, also when its file is read later)
#[test]
fn pending_read_counts_from_the_start_pass() {
    let mut files = MemoryTracks::new();
    files.insert(0, tone(800, 0));
    let ready = std::rc::Rc::new(std::cell::Cell::new(false));
    let slow = Slow {
        files,
        pending: None,
        ready: ready.clone(),
    };
    let mut m = Music::new(db(&plain(2)), vec![true, true], LIST, Box::new(slow));
    m.start_game(&sides(), Some(0));
    let mut rng = Rng::new(1);
    // T0 is picked at 0 and read later.
    assert!(matches!(m.update(0, &mut rng, true, RATE), Some(None)));
    assert_eq!(m.take_log(), [Update::Started(0)]);
    assert_eq!(m.playing(), Some(0));
    // Still reading: no completion and no voice.
    assert!(m.update(200, &mut rng, true, RATE).is_none());
    assert!(m.poll(RATE).is_none());
    // The read finishes at 220: T0 plays; T = 100 counts from C0 = 0.
    ready.set(true);
    assert!(m.poll(RATE).is_some());
    assert!(m.update(240, &mut rng, true, RATE).is_some());
    // T1's file is missing from the source although present at startup:
    // the finished read fails, and the next pass picks again (T0).
    assert_eq!(m.take_log(), [Update::Started(1)]);
    ready.set(true);
    assert!(m.poll(RATE).is_none());
    assert_eq!(m.take_log(), [Update::Failed(1)]);
    assert!(m.update(280, &mut rng, true, RATE).is_some());
    assert_eq!(m.take_log(), [Update::Started(0)]);
}

// ------------------------------------------------------------ with Audio

fn audio_with_music(w: &glasnost_sim::world::World, m: Music) -> Audio {
    let mut a = Audio::new(Box::new(NullOutput::new(RATE, "test")));
    a.set_quirks(Quirks::default());
    a.set_sides(sides());
    a.set_music(Some(m));
    a.start(w, Some(glasnost_sim::world::HouseId(0)));
    a
}

fn render(a: &mut Audio, frames: usize) -> Vec<i16> {
    let mut b = vec![0i16; frames * 2];
    a.mixer_mut().render(&mut b);
    b
}

// spec: S-0703 claim 10 (MU7: the shuffle pick's draws follow that pass's per-instance sound-choice draws on the same unsynchronised stream)
#[test]
fn mu7_music_draws_follow_the_pass_sound_choices() {
    let w = testkit::combat_game(3);
    let files = [(0, 800), (1, 800), (2, 800)];
    for seed in 1..40u32 {
        // The stream after the pass's sound choices alone.
        let mut sound_only = Audio::new(Box::new(NullOutput::new(RATE, "test")));
        let mut cat = SoundCatalog::new();
        let mut e = SoundEntry::new("Wobble", &["Wobble"]);
        e.fshift = (-5, 7);
        cat.insert(e);
        let mut bank = crate::bank::SoundBank::new();
        bank.add_pcm(
            "Wobble",
            Pcm {
                sample_rate: RATE,
                channels: 1,
                samples: vec![1000; 4],
            },
        );
        sound_only.set_catalog(cat.clone());
        sound_only.set_bank(bank.clone());
        sound_only.start(&w, None);
        sound_only.play_named(Some("Wobble"), Where::Screen);
        let mut u = Rng::new(seed);
        sound_only.service(0, &mut u);
        let after_sound = u.clone();
        assert!(after_sound.taken() > 0);
        // Both: the music pick's draws come from the stream after the sound's.
        let mut a = audio_with_music(&w, player(&plain(3), &files, SHUFFLE, on()));
        a.set_catalog(cat);
        a.set_bank(bank);
        a.play_named(Some("Wobble"), Where::Screen);
        let mut u = Rng::new(seed);
        a.service(0, &mut u);
        let got = a.music().unwrap().playing();
        let mut expect_rng = after_sound.clone();
        let d = db(&plain(3));
        let want = pick(&d, &[true; 3], None, SHUFFLE, on(), &mut expect_rng);
        assert_eq!(got, want);
        assert_eq!(u, expect_rng);
        // Some seed tells the two orders apart.
        let mut before = Rng::new(seed);
        let other = pick(&d, &[true; 3], None, SHUFFLE, on(), &mut before);
        if other != want {
            return;
        }
    }
    panic!("no seed told the orders apart");
}

// spec: S-0703 claim 3 and Edge cases (MU6 part c's engine fixture: at zero music volume the pick is made as at any volume; switch on: at T1's end 1000 draws and T0 started; switch off: no draw and T1 again; nothing is heard)
#[test]
fn mu6c_zero_volume() {
    let text = plain(4)
        .replacen("[T0]\nSound=tone0", "[T0]\nSound=tone0\nNormal=no", 1)
        .replacen("[T2]\nSound=tone2", "[T2]\nSound=tone2\nNormal=no", 1)
        .replacen("[T3]\nSound=tone3", "[T3]\nSound=tone3\nNormal=no", 1);
    let files = [(0, 800), (1, 800), (2, 800), (3, 800)];
    let w = testkit::combat_game(3);
    for (q, want, draws) in [(on(), 0usize, 1000u64), (off(), 1, 0)] {
        let mut loud = None;
        for volume in [crate::catalog::FULL, 0] {
            let mut a = audio_with_music(&w, player(&text, &files, SHUFFLE, q));
            a.mixer_mut().set_music_volume(volume);
            let mut u = Rng::new(2);
            a.service(0, &mut u);
            assert_eq!(a.music().unwrap().playing(), Some(1));
            let mut sound = Vec::new();
            sound.extend(render(&mut a, 400));
            let before = u.taken();
            // T1 (100 ms) ends at the pass at 120.
            for c in [40, 80, 120] {
                a.service(c, &mut u);
            }
            assert_eq!(u.taken() - before, draws);
            assert_eq!(a.music().unwrap().playing(), Some(want));
            sound.extend(render(&mut a, 400));
            if volume == 0 {
                assert!(
                    sound.iter().all(|&s| s == 0),
                    "nothing is heard at zero volume"
                );
                assert_eq!(loud, Some(u.taken()), "the volume adds or removes no draw");
            } else {
                assert!(sound.iter().any(|&s| s != 0));
                loud = Some(u.taken());
            }
        }
    }
}

// spec: S-0703 claim 4 (MU4 part b, second run, engine: a failed start leaves music started; the available track plays again)
#[test]
fn failed_start_leaves_music_started() {
    let text = plain(3)
        .replacen("[T0]\nSound=tone0", "[T0]\nSound=tone0\nNormal=no", 1)
        .replacen("[T2]\nSound=tone2", "[T2]\nSound=tone2\nNormal=no", 1);
    let w = testkit::combat_game(3);
    let mut a = audio_with_music(&w, player(&text, &[(1, 800), (2, 800)], SHUFFLE, on()));
    let mut u = Rng::new(9);
    for p in 0..30 {
        a.service(p * PASS, &mut u);
    }
    let log = a.music_mut().unwrap().take_log();
    assert!(started(&log).len() >= 5, "{log:?}");
    assert!(started(&log).iter().all(|&k| k == 1));
}

// spec: S-0701 claim 29, S-0703 Summary (music is presentation: the state hash is the same with and without music, under shuffle)
#[test]
fn state_hash_with_music() {
    let run = |music: bool| {
        let mut w = testkit::combat_game(21);
        let rep = testkit::combat_replay(21, 300);
        let mut a = music.then(|| {
            let files = [(0, 300), (1, 500), (2, 700)];
            audio_with_music(&w, player(&plain(3), &files, SHUFFLE, on()))
        });
        let mut u = Rng::new(21);
        let mut picks = 0;
        for t in 0..300 {
            rep.issue_due(&mut w);
            glasnost_sim::tick::step(&mut w);
            let fx = std::mem::take(&mut w.effects);
            if let Some(a) = a.as_mut() {
                a.tick(&w, &fx, &mut u);
                a.service(i64::from(t) * 67, &mut u);
                render(a, 64);
                picks += started(&a.music_mut().unwrap().take_log()).len();
            }
        }
        if music {
            assert!(picks >= 10, "{picks}");
        }
        glasnost_sim::save::state_hash(&w)
    };
    assert_eq!(run(true), run(false));
}

// The music voice streams the decoded track: chunked IMA ADPCM gives the
// same sound as the decoded whole.
#[test]
fn voice_streams_ima_tracks() {
    let pcm = Pcm::synthetic_tone(RATE, 2, 9000, 40, 9000, 3);
    let (file, recon) = glasnost_fmt::wav::ima_wav(&pcm, 512);
    let info = WavInfo::parse(&file).unwrap();
    let mut v = MusicVoice::new(info, Arc::from(file), RATE);
    let mut acc = vec![0i64; 9000 * 2];
    v.mix(&mut acc, 1 << 16);
    let got: Vec<i16> = acc.iter().map(|&s| s as i16).collect();
    assert_eq!(got, recon);
    assert!(v.finished());
}
