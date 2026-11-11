//! The client's screens without a window or an install: the menu, the
//! setup and the start job, on a synthetic install.

use super::*;
use glasnost_convert::find::Listing;
use glasnost_fmt::mix::MixEntry;
use glasnost_ui::settings::{Row, SettingsHit};
use glasnost_ui::setup::{MapEntry, SetupRules};
use glasnost_vfs::ArchiveId;

/// Clicks the middle of a target laid out in the interface's logical
/// pixels: the middle of its scaled rectangle on the screen.
fn click(c: &mut Client, r: Rect) {
    let r = c.ui_scale().rect(r);
    let p = Point::new(r.x + r.w / 2, r.y + r.h / 2);
    c.pointer_down(Button::Left, p);
    c.pointer_up(Button::Left, p, Modifiers::default());
}

fn synthetic_install() -> Install {
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
            colours: vec![[200, 0, 0, 255], [0, 0, 200, 255]],
        },
        maps: vec![(
            Listing {
                file,
                archive: "synthetic".into(),
                name: "Two".into(),
                theater: "x".into(),
                multiplayer: true,
            },
            MapEntry {
                label: "Two (2)".into(),
                starts: 2,
            },
        )],
        map_hashes: vec!["map-synthetic-two".into()],
        rules_hash: "rules-synthetic".into(),
        names: Default::default(),
        sounds: None,
        speech: None,
        bank: Default::default(),
        cursor_palette: Default::default(),
        strings: None,
        music: None,
        summary: "synthetic".into(),
    }
}

/// The menu offers the install picker before an install is read, and the
/// setup's Start hands out a start job with the lobby's choices.
/// spec: S-0412 claim 1 (the setup screen's values from the rules)
#[test]
fn menu_setup_and_start_hand_out_jobs() {
    let mut c = Client::new();
    let _ = c.frame_rgba();
    let install = c.menu_targets.rect_of(&MenuHit::Install).unwrap();
    assert!(
        c.menu_targets.rect_of(&MenuHit::Skirmish).is_none(),
        "no skirmish without an install"
    );
    click(&mut c, install);
    assert!(c.pick_install);

    c.done(Done::Prepared(
        Vfs::default(),
        Ok(Box::new(synthetic_install())),
    ));
    assert!(c.status.contains("synthetic"));
    let _ = c.frame_rgba();
    let skirmish = c.menu_targets.rect_of(&MenuHit::Skirmish).unwrap();
    click(&mut c, skirmish);
    assert_eq!(c.shell.screen, Screen::Setup);
    let _ = c.frame_rgba();
    let go = c.setup_targets.rect_of(&SetupHit::Start_).unwrap();
    c.next_seed = 42;
    click(&mut c, go);
    match c.job.take() {
        Some(Job::Start {
            lobby,
            seed,
            offered,
            ..
        }) => {
            assert_eq!(seed, 42);
            assert_eq!(lobby.map, 0);
            assert_eq!(offered.len(), 2);
            // Our own fixes layer is on in a new setup (M8 step C7).
            assert_eq!(lobby.layers, glasnost_rules::layers::Layers::PLAY_FIXES);
        }
        _ => panic!("Start hands out a start job"),
    }
    assert!(c.vfs.is_none(), "the job holds the file system");
    let px = c.frame_rgba();
    assert_eq!(px.len(), (c.size.0 * c.size.1 * 4) as usize);
    // The map does not load: back to the setup, the file system kept.
    c.done(Done::Started(
        Vfs::default(),
        Box::new(Err("no map".into())),
        glasnost_audio::bank::SoundBank::new(),
    ));
    assert_eq!(c.shell.screen, Screen::Setup);
    assert!(c.vfs.is_some());
    assert_eq!(c.status, "no map");
}

/// The setup's *Pathing and other fixes* box (M8 step C7) turns our own
/// layer off for the game the start job loads.
#[test]
fn the_fixes_box_turns_the_layer_off() {
    let mut c = Client::new();
    c.done(Done::Prepared(
        Vfs::default(),
        Ok(Box::new(synthetic_install())),
    ));
    let _ = c.frame_rgba();
    let r = c.menu_targets.rect_of(&MenuHit::Skirmish).unwrap();
    click(&mut c, r);
    let _ = c.frame_rgba();
    let r = c.setup_targets.rect_of(&SetupHit::Fixes).unwrap();
    click(&mut c, r);
    let _ = c.frame_rgba();
    let r = c.setup_targets.rect_of(&SetupHit::Start_).unwrap();
    click(&mut c, r);
    match c.job.take() {
        Some(Job::Start { lobby, .. }) => {
            assert_eq!(lobby.layers, glasnost_rules::layers::Layers::REFERENCE);
        }
        _ => panic!("Start hands out a start job"),
    }
}

/// spec: S-0213 claim 22 (fresh-profile bindings: S stop, X scatter, G guard, D deploy, C cheer) and claim 23 (Escape opens the options menu; the arrows are view keys, not commands)
#[test]
fn claims_22_23_default_keys() {
    use glasnost_sim::targeting::KeyCommand;
    assert_eq!(
        bound(Key::Char('s')),
        Some(Bound::Command(KeyCommand::Stop))
    );
    assert_eq!(
        bound(Key::Char('x')),
        Some(Bound::Command(KeyCommand::Scatter))
    );
    assert_eq!(
        bound(Key::Char('g')),
        Some(Bound::Command(KeyCommand::Guard))
    );
    assert_eq!(
        bound(Key::Char('d')),
        Some(Bound::Command(KeyCommand::Deploy))
    );
    assert_eq!(bound(Key::Char('c')), Some(Bound::Cheer));
    assert_eq!(bound(Key::Escape), Some(Bound::Options));
    for k in [Key::Left, Key::Right, Key::Up, Key::Down] {
        assert_eq!(bound(k), None);
    }
}

/// A settings store the test can look into.
#[derive(Clone, Default)]
struct Shared(std::rc::Rc<std::cell::RefCell<Option<String>>>);

impl Store for Shared {
    fn read(&self) -> Option<String> {
        self.0.borrow().clone()
    }

    fn write(&self, text: &str) -> Result<(), String> {
        *self.0.borrow_mut() = Some(text.into());
        Ok(())
    }
}

fn open_settings(c: &mut Client) {
    let _ = c.frame_rgba();
    let r = c.menu_targets.rect_of(&MenuHit::Settings).unwrap();
    click(c, r);
    assert_eq!(c.shell.screen, Screen::Settings);
    let _ = c.frame_rgba();
}

/// Settings open from the main menu without an install, change, persist
/// and load again in a new client.
#[test]
fn settings_screen_changes_persist_to_the_store() {
    let store = Shared::default();
    let mut c = Client::new();
    c.attach_store(Box::new(store.clone()));
    open_settings(&mut c);
    let r = c
        .settings_targets
        .rect_of(&SettingsHit::Step(Row::Sound, -1))
        .unwrap();
    click(&mut c, r);
    click(&mut c, r);
    assert_eq!(c.settings.sound, 80);
    let r = c
        .settings_targets
        .rect_of(&SettingsHit::Step(Row::Scroll, 1))
        .unwrap();
    click(&mut c, r);
    assert_eq!(c.settings.scroll_index, 2);
    let r = c
        .settings_targets
        .rect_of(&SettingsHit::Step(Row::Scale, 1))
        .unwrap();
    click(&mut c, r);
    let _ = c.frame_rgba();
    let kept = store.read().unwrap();
    assert!(kept.contains("sound=80") && kept.contains("scroll=2") && kept.contains("scale=125"));

    // Back to the menu.
    let r = c.settings_targets.rect_of(&SettingsHit::Back).unwrap();
    click(&mut c, r);
    assert_eq!(c.shell.screen, Screen::Menu);

    let mut again = Client::new();
    again.attach_store(Box::new(store.clone()));
    assert_eq!(again.settings, c.settings);

    // Reset restores the defaults and keeps them.
    open_settings(&mut c);
    let r = c.settings_targets.rect_of(&SettingsHit::Reset).unwrap();
    click(&mut c, r);
    assert_eq!(c.settings, Settings::default());
    assert!(store.read().unwrap().contains("sound=100"));
}

/// An output that plays (a stand-in for a device).
struct Playing;

impl glasnost_audio::output::Output for Playing {
    fn rate(&self) -> u32 {
        48_000
    }
    fn pump(&mut self, mixer: &mut glasnost_audio::mixer::Mixer) {
        mixer.clear();
    }
    fn state(&self) -> glasnost_audio::output::OutputState {
        glasnost_audio::output::OutputState::Running
    }
}

/// The settings screen says when nothing is heard, and why (design: native
/// audio): before a game, what is known without a device; then the
/// output's own reason; nothing while an output plays.
#[test]
fn the_settings_screen_says_when_there_is_no_sound() {
    use glasnost_audio::output::{NullOutput, known_silent};
    let mut c = Client::new();
    assert_eq!(c.no_sound().as_deref(), known_silent());
    open_settings(&mut c);
    c.audio = Some(Audio::new(Box::new(NullOutput::new(
        48_000,
        "no device here",
    ))));
    assert_eq!(c.no_sound().as_deref(), Some("no device here"));
    let _ = c.frame_rgba();
    c.audio = Some(Audio::new(Box::new(Playing)));
    assert_eq!(c.no_sound(), None);
    let _ = c.frame_rgba();
}

/// Rebinding refuses a key another action has, accepts a free one, and the
/// game then takes the new key.
#[test]
fn rebinding_checks_conflicts_and_changes_what_a_key_does() {
    let mut c = Client::new();
    open_settings(&mut c);
    let r = c.settings_targets.rect_of(&SettingsHit::Rebind(0)).unwrap();
    click(&mut c, r);
    assert_eq!(c.rebinding, Some(0));
    c.key(Key::Char('x'), true);
    assert_eq!(c.rebinding, Some(0), "refused, still waiting");
    assert!(c.settings_note.contains("scatter"), "{}", c.settings_note);
    assert_eq!(c.settings.bindings.key(Action::Stop), 's');
    c.key(Key::Char('q'), true);
    assert_eq!(c.rebinding, None);
    assert_eq!(c.settings.bindings.key(Action::Stop), 'q');
    assert_eq!(
        bound_with(&c.settings.bindings, Key::Char('q')),
        Some(Bound::Command(KeyCommand::Stop))
    );
    assert_eq!(bound_with(&c.settings.bindings, Key::Char('s')), None);
    assert_eq!(
        bound_with(&c.settings.bindings, Key::Escape),
        Some(Bound::Options)
    );
    // Escape cancels a rebinding, then leaves the screen.
    let _ = c.frame_rgba();
    let r = c.settings_targets.rect_of(&SettingsHit::Rebind(1)).unwrap();
    click(&mut c, r);
    c.key(Key::Escape, true);
    assert_eq!(c.rebinding, None);
    assert_eq!(c.shell.screen, Screen::Settings);
    c.key(Key::Escape, true);
    assert_eq!(c.shell.screen, Screen::Menu);
}

/// The relay address last tried is kept and comes back.
#[test]
fn the_last_relay_is_remembered() {
    let store = Shared::default();
    let mut c = Client::new();
    c.attach_store(Box::new(store.clone()));
    assert_eq!(c.netform.relay, DEFAULT_RELAY);
    c.netform.relay = " ws://relay.example:1234/ ".into();
    c.remember_relay();
    let mut again = Client::new();
    again.attach_store(Box::new(store.clone()));
    assert_eq!(again.netform.relay, "ws://relay.example:1234/");
    // Reset does not forget the relay.
    open_settings(&mut again);
    let r = again.settings_targets.rect_of(&SettingsHit::Reset).unwrap();
    click(&mut again, r);
    assert_eq!(again.settings.relay, "ws://relay.example:1234/");
}

/// A store that refuses writes leaves the screen working and says so.
#[test]
fn a_refusing_store_gives_a_note_not_a_failure() {
    struct Refuses;
    impl Store for Refuses {
        fn read(&self) -> Option<String> {
            None
        }
        fn write(&self, _: &str) -> Result<(), String> {
            Err("blocked".into())
        }
    }
    let mut c = Client::new();
    c.attach_store(Box::new(Refuses));
    assert_eq!(c.settings, Settings::default());
    open_settings(&mut c);
    let r = c
        .settings_targets
        .rect_of(&SettingsHit::Step(Row::Music, -1))
        .unwrap();
    click(&mut c, r);
    assert_eq!(c.settings.music, 90);
    assert!(c.settings_note.contains("blocked"));
    let _ = c.frame_rgba();
}

/// Settings never reach the simulation: the same game played under two very
/// different settings has the same state hash at every tick.
#[test]
fn state_hash_is_the_same_under_any_settings() {
    use glasnost_ai::fixture;
    use glasnost_ai::game::Game;
    use glasnost_sim::econ::setup::{self as econ, Player};
    let play = |extreme: bool| {
        let mut c = Client::new();
        if extreme {
            let mut b = Bindings::default();
            b.rebind(Action::Stop, 'q').unwrap();
            c.settings = Settings {
                sound: 0,
                voices: 0,
                music: 0,
                scroll_index: 0,
                ui_scale: 300,
                // design: readable text and display scale (lead's request 2026-11-10): no setting reaches the hash
                display_scale: 3,
                captions: false,
                colour_blind: true,
                bindings: b,
                relay: "ws://elsewhere/".into(),
                rejoin: None,
                direct: false,
                ice: "stun:elsewhere:3478".into(),
                tool_tips: false,
                // design: touch and touchpad scrolling (lead's request 2026-11-10): no setting reaches the hash
                touchpad_scroll: false,
            };
            c.settings_changed();
        }
        let rules = fixture::rules();
        let players = [
            Player {
                name: "Person".into(),
                country: "Alphaland".into(),
                ..Default::default()
            },
            Player {
                name: "Machine".into(),
                country: "Betaland".into(),
                ai: true,
                ..Default::default()
            },
        ];
        let s = econ::skirmish(&rules, &players, econ::Settings::default(), 0, 7);
        let mut g = Game::new(rules, &fixture::map(), "synthetic", fixture::catalogue(), s)
            .expect("the synthetic skirmish loads");
        let mut hashes = Vec::new();
        for _ in 0..120 {
            g.step();
            hashes.push(g.state_hash());
        }
        hashes
    };
    assert_eq!(play(false), play(true));
}

/// The edge-scroll cursor's frames (S-0213 table E: the allowed and the
/// blocked variant of the eight directions).
const EDGE_FRAMES: std::ops::RangeInclusive<u16> = 2..=17; // spec: S-0213 table E (frames 2 to 9 allowed, 10 to 17 blocked)

/// A started game past the order block of its first ticks, with a
/// synthetic cursor image of every battlefield frame, and the client
/// focused.
fn cursor_game() -> Client {
    let mut c = super::access_tests::client((800, 600), 100);
    super::access_tests::start_by_keyboard(&mut c);
    c.focused = true;
    let s = c.session.as_mut().expect("a game");
    // Frames up to the last of S-0214 table CS (503).
    s.set_test_cursor(504, (8, 8));
    while s.play.game.world.tick <= s.play.game.world.orders_blocked_through {
        s.step();
    }
    c
}

/// One unfrozen cycle's scrolling at real time `t`, then the frame's cursor.
fn cursor_cycle(c: &mut Client, t: i64) -> Option<u16> {
    let s = c.session.as_mut().expect("a game");
    s.clock_ms = t;
    s.scroll_cycle(t as u64, [false; 4], true);
    s.update_cursor(&mut c.painter);
    s.cursor_frame()
}

/// #86. spec: S-0101 claim 12 (edge scrolling only without a mouse button held), S-0213 claim 24
/// (the edge-scroll cursor shows during edge scrolling)
///
/// A button pressed on the battlefield and held while the pointer moves onto the screen's edge
/// neither scrolls the view nor shows the edge-scroll cursor; released there, both follow.
#[test]
fn a_held_button_shows_no_edge_cursor() {
    let mut c = cursor_game();
    let s = c.session.as_ref().expect("a game");
    let m = s.map_rect();
    let inside = Point::new(m.x + m.w / 2, m.y + m.h / 2);
    let edge = Point::new(m.x + m.w / 2, 0);
    let centre = s.view().origin;
    c.pointer_down(Button::Left, inside);
    c.pointer_moved(edge);
    let held = cursor_cycle(&mut c, 20_000);
    assert_eq!(
        c.session.as_ref().unwrap().view().origin,
        centre,
        "a held button: no edge scroll"
    );
    assert!(
        !held.is_some_and(|f| EDGE_FRAMES.contains(&f)),
        "a held button: no edge-scroll cursor, got {held:?}"
    );
    c.pointer_up(Button::Left, edge, Modifiers::default());
    let released = cursor_cycle(&mut c, 20_016);
    assert!(
        released.is_some_and(|f| EDGE_FRAMES.contains(&f)),
        "released at the edge: the edge-scroll cursor, got {released:?}"
    );
}

/// #83. spec: S-0101 F8 (the cursor probe reads each cell's height level) and F1 (a cell's
/// level lifts it by the terrain lift per level)
///
/// With every cell of the world raised by 4 levels in play, the cell under a point is the one
/// the load-time map gives 4 terrain lifts lower on the screen.
#[test]
fn the_cursor_cell_follows_the_world_levels() {
    let mut c = super::access_tests::client((800, 600), 100);
    super::access_tests::start_by_keyboard(&mut c);
    let s = c.session.as_mut().expect("a game");
    let m = s.map_rect();
    let lift = glasnost_render::projection::TERRAIN_LIFT * 4;
    let p = Point::new(m.x + m.w / 2, m.y + m.h / 2 - lift);
    let lower = s.under(Point::new(p.x, p.y + lift)).0;
    let grid = &mut s.play.game.world.cells.grid;
    let all: Vec<_> = grid.iter().map(|(xy, _)| xy).collect();
    for xy in all {
        grid.get_mut(xy).expect("a legal cell").level += 4;
    }
    assert_eq!(s.under(p).0, lower);
}

/// Animated cursors of at least four frames (S-0214 table C).
const ANIMATED: [std::ops::RangeInclusive<u16>; 3] = [
    18..=30,   // spec: S-0214 table C (select, toggle select: 18–30)
    31..=40,   // spec: S-0214 table C (move frames 31–40)
    110..=118, // spec: S-0214 table C (deploy, area attack: 110–118)
];

/// A point on the battlefield, away from the screen's edges, where an
/// animated cursor shows, and that cursor's first frame.
fn animated_point(c: &mut Client, t: i64) -> (Point, u16) {
    let m = c.session.as_ref().expect("a game").map_rect();
    for dy in (40..m.h - 40).step_by(4) {
        for dx in (40..m.w - 40).step_by(4) {
            let p = Point::new(m.x + dx, m.y + dy);
            c.pointer_moved(p);
            let s = c.session.as_mut().expect("a game");
            s.clock_ms = t;
            s.update_cursor(&mut c.painter);
            if let Some(r) = s
                .cursor_frame()
                .and_then(|f| ANIMATED.iter().find(|r| r.contains(&f)))
            {
                return (p, *r.start());
            }
        }
    }
    let s = c.session.as_ref().unwrap();
    panic!(
        "no point shows an animated cursor; selection {:?}, map {m:?}",
        s.play.selection
    );
}

/// The frame's cursor at real time `t`, without a scroll cycle.
fn cursor_at(c: &mut Client, t: i64) -> Option<u16> {
    let s = c.session.as_mut().expect("a game");
    s.clock_ms = t;
    s.update_cursor(&mut c.painter);
    s.cursor_frame()
}

/// #85. spec: S-0214 claim 22 (changing cursor kind starts at its first frame and restarts the
/// interval; the clock advances a frame every 4 units of 16 milliseconds)
///
/// An animated action cursor that gave way to the system pointer (over the sidebar) or to the
/// edge-scroll cursor starts again at its first frame when the pointer comes back.
#[test]
fn a_cursor_shown_again_starts_at_its_first_frame() {
    let mut c = cursor_game();
    // An own vehicle selected: the ground offers a move.
    let unit = super::access_tests::own(&c, glasnost_sim::rules::Kind::Vehicle)[0];
    c.session.as_mut().unwrap().play.selection = vec![unit];
    let (p, first) = animated_point(&mut c, 30_000);
    // Restart at a known time: away from the map, then back.
    c.pointer_left();
    let _ = cursor_at(&mut c, 30_000);
    c.pointer_moved(p);
    assert_eq!(cursor_at(&mut c, 40_000), Some(first));
    // Three advances, 64 ms apart.
    let advance = |c: &mut Client, t: i64| {
        for k in 1..=3 {
            assert_eq!(cursor_at(c, t + 64 * k), Some(first + k as u16));
        }
    };
    advance(&mut c, 40_000);
    // Over the sidebar the system pointer shows; back on the map, the
    // select cursor starts again.
    let m = c.session.as_ref().unwrap().map_rect();
    c.pointer_moved(Point::new(m.x + m.w + 20, m.y + m.h / 2));
    assert_eq!(cursor_at(&mut c, 40_208), None, "the system pointer");
    c.pointer_moved(p);
    assert_eq!(
        cursor_at(&mut c, 40_272),
        Some(first),
        "after the system pointer"
    );
    // The edge-scroll cursor, then back to the same place.
    advance(&mut c, 40_272);
    let centre = c.session.as_ref().unwrap().centre();
    c.pointer_moved(Point::new(m.x + m.w / 2, 0));
    let edge = cursor_cycle(&mut c, 40_480);
    assert!(edge.is_some_and(|f| EDGE_FRAMES.contains(&f)), "{edge:?}");
    c.pointer_moved(p);
    // While the edge momentum runs out the retained direction's cursor
    // shows; the view is put back before each frame's cursor, so that `p`
    // keeps its object.
    let mut t = 40_496;
    let back = loop {
        let s = c.session.as_mut().unwrap();
        s.clock_ms = t;
        s.scroll_cycle(t as u64, [false; 4], true);
        s.set_centre(centre);
        s.update_cursor(&mut c.painter);
        let f = s.cursor_frame();
        if !f.is_some_and(|f| EDGE_FRAMES.contains(&f)) {
            break f;
        }
        t += 16;
        assert!(t < 50_000, "the edge momentum runs out");
    };
    assert_eq!(back, Some(first), "after the edge-scroll cursor");
}
