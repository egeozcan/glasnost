//! Music through the windowless client (S-0703 claims 11 to 13) on
//! synthetic data: an invented music database and synthetic tones, the
//! invented rules and map of `glasnost_ai::fixture`. The client's screens
//! drive the player's transitions; the audio output is silent.

use std::rc::Rc;

use super::*;
use crate::mapview::load::Loaded;
use glasnost_ai::fixture;
use glasnost_audio::music::{Phase, Update};
use glasnost_audio::music_files::InstallMusic;
use glasnost_convert::find::Listing;
use glasnost_fmt::mix::MixEntry;
use glasnost_fmt::pcm::Pcm;
use glasnost_sim::econ::victory::{Outcome, Verdict};
use glasnost_ui::hud::PauseHit;
use glasnost_ui::setup::{MapEntry, SetupRules};
use glasnost_vfs::ArchiveId;
use glasnost_vfs::source::MemSource;

/// Entries T0 and T1 (normal), then the loading, score and menu tracks
/// (normal flag clear), each with its own synthetic tone.
const DATABASE: &str = "[Themes]\n0=T0\n1=T1\n2=LOADING\n3=SCORE\n4=INTRO\n\
    [T0]\nSound=tone0\n[T1]\nSound=tone1\n\
    [LOADING]\nSound=loading\nNormal=no\n[SCORE]\nSound=score\nNormal=no\n\
    [INTRO]\nSound=intro\nNormal=no\n";
const T0: usize = 0;
const LOADING: usize = 2;
const SCORE: usize = 3;
const INTRO: usize = 4;

fn picked(path: &str, bytes: Vec<u8>) -> PickedFile {
    PickedFile {
        path: path.into(),
        source: Rc::new(MemSource(bytes)),
    }
}

/// The music of a synthetic install: tones of 2 s at 8000 Hz.
fn music() -> InstallMusic {
    let tone = |seed| Pcm::synthetic_tone(8000, 1, 16000, 40, 8000, seed).to_wav();
    let mut vfs = Vfs::new(Default::default(), glasnost_vfs::MountOptions::default());
    let mut opts = *vfs.game_options();
    opts.mod_layer = true;
    pollster::block_on(vfs.mount_install(vec![
        picked("thememd.ini", DATABASE.as_bytes().to_vec()),
        picked("tone0.wav", tone(0)),
        picked("tone1.wav", tone(1)),
        picked("loading.wav", tone(2)),
        picked("score.wav", tone(3)),
        picked("intro.wav", tone(4)),
    ]));
    pollster::block_on(vfs.mount_startup(opts));
    pollster::block_on(InstallMusic::read(&vfs)).expect("the music database")
}

fn install() -> Install {
    let file = FileRef {
        archive: ArchiveId(0),
        entry: MixEntry {
            id: 1,
            offset: 0,
            size: 0,
        },
    };
    Install {
        setup: SetupRules {
            money: 5000,
            money_range: (1000, 9000, 500),
            units: 3,
            units_range: (1, 9),
            bases: true,
            flags: [false, true, true, true, true],
            game_speed: 3,
            countries: vec!["Alphaland".into(), "Betaland".into()],
            colours: vec![[200, 0, 0, 255]; 8],
        },
        maps: vec![(
            Listing {
                file,
                archive: "synthetic".into(),
                name: "synthetic".into(),
                theater: "x".into(),
                multiplayer: true,
            },
            MapEntry {
                label: "synthetic (2)".into(),
                starts: 2,
            },
        )],
        map_hashes: vec!["map-synthetic".into()],
        rules_hash: fixture::rules().hash.clone(),
        names: Default::default(),
        sounds: None,
        speech: None,
        bank: Default::default(),
        cursor_palette: Default::default(),
        music: Some(music()),
        summary: "synthetic".into(),
    }
}

fn client() -> Client {
    let mut c = Client::new();
    c.cpu_map = false;
    c.resize((1024, 768));
    c.done(Done::Prepared(Vfs::default(), Ok(Box::new(install()))));
    let _ = c.frame();
    c
}

fn phase(c: &Client) -> Phase {
    c.audio
        .as_ref()
        .and_then(|a| a.music())
        .map_or(Phase::Off, |m| m.phase())
}

fn playing(c: &Client) -> Option<usize> {
    c.audio.as_ref().and_then(|a| a.music()?.playing())
}

fn log(c: &mut Client) -> Vec<Update> {
    c.audio
        .as_mut()
        .and_then(|a| a.music_mut())
        .map(|m| m.take_log())
        .unwrap_or_default()
}

/// Starts a skirmish from the menus: the start job is handed out (the game
/// loads), then it is run on the synthetic data.
fn start(c: &mut Client, theme: Option<&[u8]>) {
    c.shell.screen = Screen::Menu;
    c.menu_press(MenuHit::Skirmish);
    c.setup_press(SetupHit::Start_);
    assert_eq!(phase(c), Phase::Loading, "the game loads");
    assert_eq!(playing(c), Some(LOADING));
    let Some(Job::Start {
        vfs,
        lobby,
        offered,
        seed,
        local,
        ..
    }) = c.job.take()
    else {
        panic!("a start job");
    };
    let gs = ui_setup::game_setup(&lobby, &fixture::rules(), &offered, seed, seed)
        .expect("the setup makes a game setup");
    let s = Loaded::bare(fixture::map(), fixture::rules(), fixture::catalogue(), &gs)
        .map(|mut l| {
            l.theme = theme.map(<[u8]>::to_vec);
            l
        })
        .and_then(|l| Session::bare(l, "map-synthetic".into(), local));
    c.done(Done::Started(vfs, Box::new(s), SoundBank::new()));
    assert_eq!(c.shell.screen, Screen::Game);
    let _ = c.frame();
}

/// Presses a control of the open options menu.
fn press_pause(c: &mut Client, hit: PauseHit) {
    let _ = c.frame();
    let r = c
        .session
        .as_ref()
        .expect("a game")
        .pause_targets()
        .rect_of(&hit)
        .unwrap_or_else(|| panic!("{hit:?} on the options menu"));
    let r = c.ui_scale().rect(r);
    let p = Point::new(r.x + r.w / 2, r.y + r.h / 2);
    c.pointer_down(Button::Left, p);
    c.pointer_up(Button::Left, p, Modifiers::default());
}

fn open_menu(c: &mut Client) {
    c.key(Key::Escape, true);
    c.key(Key::Escape, false);
    assert!(c.session.as_ref().expect("a game").paused);
}

// spec: S-0703 claim 11 (MU8 and MU11 part g, through the client: the loading track plays while the game loads and stops when it starts; the first start is made at the game's first audio pass, with no ended track)
#[test]
fn the_loading_track_and_the_first_start() {
    let mut c = client();
    assert_eq!(phase(&c), Phase::Off, "no music at program start");
    start(&mut c, None);
    assert_eq!(phase(&c), Phase::Armed);
    assert_eq!(playing(&c), None, "the loading track stopped");
    c.update(1_000);
    assert_eq!(phase(&c), Phase::Started);
    assert_eq!(log(&mut c), [Update::Started(T0)]);
}

// spec: S-0703 claim 11 (the map's [Basic] Theme names the first track, through the client)
#[test]
fn the_map_theme_starts_first() {
    let mut c = client();
    start(&mut c, Some(b"intro"));
    c.update(1_000);
    assert_eq!(playing(&c), Some(INTRO));
}

fn stand(c: &Client) -> Stand {
    c.audio.as_ref().unwrap().stand()
}

// spec: S-0703 claim 13 and S-0701 claim 30 (MU11 part f, our engine, through the client: the music plays on while the options menu is open, and a track that completes there on music time is followed by its pick, heard in the menu; our settings screen opened from the game counts as the options menu; sound stands still in both)
#[test]
fn the_options_menu_and_the_settings_screen_play_the_music_on() {
    let mut c = client();
    start(&mut c, None);
    // T0 (2 s) starts at music time 1000.
    c.update(1_000);
    assert_eq!(log(&mut c), [Update::Started(T0)]);
    open_menu(&mut c);
    for t in (1_040..3_000).step_by(40) {
        c.update(t);
        assert_eq!(stand(&c), Stand::Menu);
        let a = c.audio.as_ref().unwrap();
        assert!(a.sounds_held() && !a.music_held());
    }
    assert!(log(&mut c).is_empty(), "T0 still plays");
    // A music-only pass at 3000: T0 has completed, T1 is picked in the menu.
    c.update(3_000);
    assert_eq!(log(&mut c), [Update::Started(1)]);
    assert_eq!(playing(&c), Some(1));
    press_pause(&mut c, PauseHit::Settings);
    assert_eq!(c.shell.screen, Screen::Settings);
    for t in (3_040..5_000).step_by(40) {
        c.update(t);
        assert_eq!(
            stand(&c),
            Stand::Menu,
            "the settings screen is the options menu"
        );
    }
    assert!(log(&mut c).is_empty());
    c.update(5_000);
    assert_eq!(
        log(&mut c),
        [Update::Started(T0)],
        "a pick on the settings screen"
    );
    // Back to the game, its options menu still open, then play.
    c.settings_from_game = false;
    c.shell.screen = Screen::Game;
    press_pause(&mut c, PauseHit::Resume);
    c.update(5_040);
    assert_eq!(stand(&c), Stand::Play);
    assert!(!c.audio.as_ref().unwrap().sounds_held());
    // T0 started at 5000 goes on counting on music time: it ends at 7000.
    for t in (5_080..7_000).step_by(40) {
        c.update(t);
    }
    assert!(log(&mut c).is_empty());
    for t in (7_000..7_200).step_by(40) {
        c.update(t);
    }
    assert_eq!(started_of(&log(&mut c)), [1]);
}

// spec: S-0701 claim 30 (through the client: lost focus in a skirmish is a full stand-still for music, with the options menu closed or open; the track is held, no pick is made, and the unfocused stretch adds nothing to music time)
#[test]
fn lost_focus_holds_the_music() {
    let mut c = client();
    start(&mut c, None);
    // T0 (2 s) starts at music time 1000: it ends at music time 3000.
    c.update(1_000);
    log(&mut c);
    c.focus(false);
    for t in (1_040..20_000).step_by(40) {
        c.update(t);
        assert_eq!(stand(&c), Stand::Unfocused);
        assert!(c.audio.as_ref().unwrap().music_held());
    }
    assert!(log(&mut c).is_empty(), "no pick without focus");
    // Focus returns: 19 s left out, so T0 ends at host time 22000.
    c.focus(true);
    for t in (20_000..22_000).step_by(40) {
        c.update(t);
        assert!(!c.audio.as_ref().unwrap().music_held());
    }
    assert!(log(&mut c).is_empty());
    c.update(22_000);
    assert_eq!(log(&mut c), [Update::Started(1)]);
    // The options menu open, then focus lost and found again: T1, started
    // at music time 3000, ends at 5000, after 2 s more with focus.
    open_menu(&mut c);
    c.update(22_040);
    c.focus(false);
    for t in (22_080..40_000).step_by(40) {
        c.update(t);
        assert_eq!(stand(&c), Stand::Unfocused, "menu open, focus lost");
        assert!(c.audio.as_ref().unwrap().music_held());
    }
    c.focus(true);
    // Music time 3040 again at 40000: T1 ends at host time 41960.
    for t in (40_000..41_960).step_by(40) {
        c.update(t);
        assert_eq!(stand(&c), Stand::Menu);
    }
    assert!(log(&mut c).is_empty());
    c.update(41_960);
    assert_eq!(log(&mut c), [Update::Started(T0)]);
}

fn started_of(log: &[Update]) -> Vec<usize> {
    log.iter()
        .filter_map(|u| match u {
            Update::Started(k) => Some(*k),
            _ => None,
        })
        .collect()
}

// spec: S-0703 claim 13 (MU11 part e, our engine, through the client: leaving through the options menu stops the track, no music update follows, and the main menu starts the menu track; a surrender leaves the music as in play)
#[test]
fn surrender_keeps_the_music_and_leaving_starts_the_menu_track() {
    let mut c = client();
    start(&mut c, None);
    c.update(1_000);
    open_menu(&mut c);
    press_pause(&mut c, PauseHit::Surrender);
    c.update(1_040);
    assert_eq!(phase(&c), Phase::Started, "the music as in play");
    assert_eq!(playing(&c), Some(T0));
    // A second game, left through its options menu.
    start(&mut c, None);
    c.update(1_000);
    log(&mut c);
    open_menu(&mut c);
    press_pause(&mut c, PauseHit::Leave);
    assert_eq!(c.shell.screen, Screen::Menu);
    assert_eq!(phase(&c), Phase::Menu);
    assert_eq!(playing(&c), Some(INTRO));
    assert!(!c.audio.as_ref().unwrap().music_held());
    for t in (1_040..10_000).step_by(40) {
        c.update(t);
    }
    assert_eq!(log(&mut c), [Update::Started(INTRO)], "no pick");
}

// spec: S-0703 claim 12 (MU11 parts a, g and i, our engine, through the client: after a win or a loss the end screen opens when the game is over, stops the track and starts the score track, with no music update after; leaving it starts the menu track)
#[test]
fn the_end_screen_plays_the_score_track() {
    end_screen_after(Verdict::Win);
    end_screen_after(Verdict::Loss);
}

fn end_screen_after(verdict: Verdict) {
    let mut c = client();
    start(&mut c, None);
    c.update(1_000);
    log(&mut c);
    // The game is over (no tick runs without focus, so the outcome set
    // here stands for this cycle).
    c.focus(false);
    let s = c.session.as_mut().expect("a game");
    let tick = s.play.world().tick;
    let local = s.play.me();
    s.play.outcome = Some(Outcome {
        ended: tick,
        over: tick,
        results: vec![(local, verdict)],
    });
    c.update(1_040);
    assert!(matches!(c.shell.screen, Screen::End(_)));
    assert_eq!(phase(&c), Phase::Score);
    assert_eq!(playing(&c), Some(SCORE));
    for t in (1_080..10_000).step_by(40) {
        c.update(t);
    }
    assert_eq!(log(&mut c), [Update::Started(SCORE)], "no pick");
    c.end_press();
    assert_eq!(c.shell.screen, Screen::Menu);
    assert_eq!(playing(&c), Some(INTRO));
    // A game started from the menus stops the menu track at once.
    start(&mut c, None);
    assert_eq!(phase(&c), Phase::Armed);
}

// spec: S-0703 claim 11 (a game that does not start: the loading track stops with the return to the setup)
#[test]
fn a_failed_start_stops_the_loading_track() {
    let mut c = client();
    c.menu_press(MenuHit::Skirmish);
    c.setup_press(SetupHit::Start_);
    assert_eq!(playing(&c), Some(LOADING));
    let Some(Job::Start { vfs, .. }) = c.job.take() else {
        panic!("a start job");
    };
    c.done(Done::Started(
        vfs,
        Box::new(Err("no map".into())),
        SoundBank::new(),
    ));
    assert_eq!(c.shell.screen, Screen::Setup);
    c.update(1_000);
    assert_eq!(playing(&c), None);
    assert_eq!(phase(&c), Phase::Off);
}
