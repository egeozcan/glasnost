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
