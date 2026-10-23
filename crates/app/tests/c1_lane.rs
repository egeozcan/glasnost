//! Local lane for music (M8 step C1; never in CI): with `GLASNOST_GAME_DIR`
//! set, reads the install through the windowless client and starts the
//! same skirmish twice, once with music and once without, for
//! `GLASNOST_C1_TICKS` updates (default 700) a second of logical time
//! apart. The music run plays under list order (the settings forced off
//! for the order check, whatever the install's settings file says) and
//! checks that the tracks start in list order over the available entries
//! and that each starts at the first pass at or after the last one's
//! logical end (S-0703 claims 5 and 10); the two runs must end with the
//! same state hash (music is presentation). Writes `music.txt` into
//! `local-lane/m8/c1/` (git-ignored; `GLASNOST_LANE_OUT` redirects) with
//! counts and timings only, never a track's name.
//!
//! Run with `cargo test -p glasnost-app --test c1_lane -- --ignored
//! --nocapture`.

#![cfg(not(target_arch = "wasm32"))]

use std::fmt::Write as _;
use std::rc::Rc;
use std::time::Instant;

use glasnost_audio::music_files::SETTINGS_FILE;
use glasnost_vfs::source::{FileSource, Unopened};
use glasnost_vfs::{ByteSource, MountOptions, PickedFile};

use glasnost_app::game::client::{Client, run_job};
use glasnost_app::mapview::png;
use glasnost_audio::music::{MusicSettings, Update, logical_ms};
use glasnost_fmt::wav::WavInfo;
use glasnost_sim::targeting::Modifiers;
use glasnost_ui::geom::{Point, Rect};
use glasnost_ui::menu::MenuHit;
use glasnost_ui::setup::SetupHit;
use glasnost_ui::shell::Screen;

/// The install's top-level files as the native client picks them: the
/// archives and the settings file opened, other loose files never read.
fn files(dir: &std::path::Path) -> Vec<PickedFile> {
    let ext = MountOptions::default().archive_extensions;
    let mut out = Vec::new();
    for e in std::fs::read_dir(dir).unwrap().flatten() {
        let Ok(meta) = e.metadata() else { continue };
        if !meta.is_file() {
            continue;
        }
        let name = e.file_name().to_string_lossy().into_owned();
        let x = name
            .rsplit_once('.')
            .map(|(_, x)| x.to_ascii_lowercase())
            .unwrap_or_default();
        let open = ext.contains(&x) || name.eq_ignore_ascii_case(SETTINGS_FILE);
        let source: Rc<dyn ByteSource> = if open {
            match FileSource::open(&e.path()) {
                Ok(f) => Rc::new(f),
                Err(_) => continue,
            }
        } else {
            Rc::new(Unopened(meta.len()))
        };
        out.push(PickedFile { path: name, source });
    }
    out
}

fn run_jobs(c: &mut Client) {
    while let Some(job) = c.job.take() {
        let done = pollster::block_on(run_job(job));
        c.done(done);
    }
}

fn click(c: &mut Client, r: Rect) {
    let r = c.ui_scale().rect(r);
    let p = Point::new(r.x + r.w / 2, r.y + r.h / 2);
    c.pointer_moved(p);
    c.pointer_down(glasnost_app::game::play::Button::Left, p);
    c.pointer_up(
        glasnost_app::game::play::Button::Left,
        p,
        Modifiers::default(),
    );
    let _ = c.frame();
}

struct Run {
    hash: String,
    starts: Vec<(i64, usize)>,
    failed: usize,
    report: String,
}

fn game(dir: &std::path::Path, music: bool, n: usize) -> Run {
    let mut report = String::new();
    let mut c = Client::new();
    c.resize((1024, 768));
    let t = Instant::now();
    c.mount(files(dir));
    run_jobs(&mut c);
    let read_ms = t.elapsed().as_millis();
    let install = c.install.as_mut().expect("the install reads");
    if music {
        let m = install
            .music
            .as_mut()
            .expect("the install has a music database");
        let present = m.files.present();
        let _ = writeln!(
            report,
            "music database: {} entries, {} with their file present; install read in {read_ms} ms",
            m.db.len(),
            present.iter().filter(|p| **p).count()
        );
        let _ = writeln!(
            report,
            "settings file: shuffle {}, repeat {} (forced off for this run)",
            m.settings.shuffle, m.settings.repeat
        );
        m.settings = MusicSettings::default();
    } else {
        install.music = None;
    }
    let _ = c.frame();
    let r = c.menu_targets.rect_of(&MenuHit::Skirmish).unwrap();
    click(&mut c, r);
    let r = c.setup_targets.rect_of(&SetupHit::Start_).unwrap();
    c.next_seed = 4242;
    click(&mut c, r);
    run_jobs(&mut c);
    assert_eq!(
        c.shell.screen,
        Screen::Game,
        "the game starts: {}",
        c.status
    );
    let mut starts = Vec::new();
    let mut failed = 0;
    let mut slowest = 0u128;
    let mut slowest_start = 0u128;
    let mut clock = 0i64;
    for _ in 0..n {
        clock += 1000;
        let t = Instant::now();
        c.update(clock);
        let us = t.elapsed().as_micros();
        if c.shell.screen != Screen::Game {
            break;
        }
        let mut started = false;
        if let Some(m) = c.audio.as_mut().and_then(|a| a.music_mut()) {
            for u in m.take_log() {
                match u {
                    Update::Started(k) => {
                        starts.push((clock, k));
                        started = true;
                    }
                    Update::Failed(_) => failed += 1,
                    _ => {}
                }
            }
        }
        if started {
            slowest_start = slowest_start.max(us);
        } else {
            slowest = slowest.max(us);
        }
    }
    let s = c.session.as_ref().expect("the session");
    let hash = glasnost_sim::save::state_hash(&s.play.game.world);
    let _ = writeln!(
        report,
        "{}: {n} updates, slowest without a track start {slowest} us, slowest with one {slowest_start} us, state hash {hash}",
        if music { "with music" } else { "without music" }
    );
    if music {
        let install = c.install.as_ref().unwrap();
        let m = install.music.as_ref().unwrap();
        let available = c
            .audio
            .as_ref()
            .and_then(|a| a.music())
            .map(|p| p.available().to_vec())
            .unwrap_or_default();
        let _ = writeln!(
            report,
            "available tracks: {}; tracks started: {}; failed starts: {failed}",
            available.iter().filter(|a| **a).count(),
            starts.len()
        );
        // List order (claim 5): each start is the next available entry
        // after the last, wrapping.
        let want: Vec<usize> = (0..available.len())
            .cycle()
            .filter(|&k| available[k])
            .take(starts.len())
            .collect();
        let got: Vec<usize> = starts.iter().map(|&(_, k)| k).collect();
        assert_eq!(got, want, "the tracks start in list order");
        // Completion (claim 10): the next track starts at the first pass
        // with C ≥ C0 + T; passes here are 1000 ms apart.
        for w in starts.windows(2) {
            let (c0, k) = w[0];
            let loc = m.files.location(k).unwrap();
            let bytes =
                pollster::block_on(loc.source.read_at(loc.offset, loc.len as usize)).unwrap();
            let info = WavInfo::parse(&bytes).unwrap();
            let t = logical_ms(info.frames(bytes.len()), info.sample_rate);
            let gap = w[1].0 - c0;
            let _ = writeln!(
                report,
                "track at position {}: logical {t} ms, next start after {gap} ms",
                got.iter().position(|&x| x == k).unwrap_or(0)
            );
            assert!(
                gap >= t && gap < t + 1000,
                "completion pass: T {t}, gap {gap}"
            );
        }
    }
    Run {
        hash,
        starts,
        failed,
        report,
    }
}

#[test]
#[ignore = "local lane: needs GLASNOST_GAME_DIR"]
fn music_plays_in_list_order_and_leaves_the_hash() {
    let Some(dir) = std::env::var_os("GLASNOST_GAME_DIR") else {
        eprintln!("GLASNOST_GAME_DIR not set: skipped");
        return;
    };
    let dir = std::path::PathBuf::from(dir);
    let out = png::lane_out("m8/c1");
    std::fs::create_dir_all(&out).unwrap();
    let n = std::env::var("GLASNOST_C1_TICKS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(700);
    let with = game(&dir, true, n);
    let without = game(&dir, false, n);
    let mut text = with.report.clone();
    text += &without.report;
    let same = with.hash == without.hash;
    let _ = writeln!(text, "state hash with and without music equal: {same}");
    std::fs::write(out.join("music.txt"), &text).unwrap();
    print!("{text}");
    assert!(with.starts.len() >= 2, "at least two tracks started");
    assert_eq!(with.failed, 0);
    assert!(without.starts.is_empty());
    assert!(same, "music leaves the state hash");
}
