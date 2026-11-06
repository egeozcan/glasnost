//! Accessibility (M8 step C3) through the windowless client on synthetic
//! data (invented rules and map, `glasnost_ai::fixture`): the interface
//! scale's hit-testing, keyboard-only play of every screen, of the sidebar
//! and of a building's placement, captions, colour-blind swatches, and the
//! state hash under all of these settings. Our own client behaviour; the
//! specs cited are the ones the keyboard routes through unchanged.

use super::*;
use crate::mapview::load::Loaded;
use glasnost_ai::fixture;
use glasnost_convert::find::Listing;
use glasnost_fmt::mix::MixEntry;
use glasnost_map::CellXY;
use glasnost_sim::orders::Order;
use glasnost_sim::rules::Kind;
use glasnost_sim::world::ObjectId;
use glasnost_ui::draw::Prim;
use glasnost_ui::end::{End, EndRow};
use glasnost_ui::hud::{HudHit, PauseHit};
use glasnost_ui::netmenu::NetField;
use glasnost_ui::settings::{Row, SettingsHit};
use glasnost_ui::setup::{MapEntry, SetupRules};
use glasnost_vfs::ArchiveId;

use super::super::play::Tool;

/// The interface scales the settings offer.
const SCALES: [u16; 5] = [100, 125, 150, 200, 300];

/// The window sizes tried: a large one every scale fits, and one that
/// lowers the largest.
const SIZES: [(i32, i32); 2] = [(1280, 960), (1024, 768)];

pub(super) fn install() -> Install {
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
        music: None,
        summary: "synthetic".into(),
    }
}

/// A client with the synthetic install, at `size` and interface `scale`.
pub(super) fn client(size: (i32, i32), scale: u16) -> Client {
    let mut c = Client::new();
    c.cpu_map = false;
    c.settings.ui_scale = scale;
    c.resize(size);
    c.done(Done::Prepared(Vfs::default(), Ok(Box::new(install()))));
    let _ = c.frame();
    c
}

/// Clicks the screen pixel `p`.
pub(super) fn click_at(c: &mut Client, p: Point) {
    c.pointer_down(Button::Left, p);
    c.pointer_up(Button::Left, p, Modifiers::default());
}

/// Clicks the middle of a target laid out in logical pixels.
pub(super) fn click(c: &mut Client, r: Rect) {
    let r = c.ui_scale().rect(r);
    click_at(c, Point::new(r.x + r.w / 2, r.y + r.h / 2));
}

/// Presses and releases a key, then draws a frame (the targets follow).
pub(super) fn key(c: &mut Client, k: Key) {
    c.key(k, true);
    c.key(k, false);
    let _ = c.frame();
}

/// Every pixel at the corners of each target's scaled rectangle maps back
/// into the target, and the pixels just outside it do not.
fn check_targets<T: Clone + std::fmt::Debug>(scale: Scale, t: &Targets<T>) {
    for (r, hit) in &t.list {
        let s = scale.rect(*r);
        assert!(s.w > 0 && s.h > 0, "{hit:?} has pixels");
        let inside = [
            Point::new(s.x, s.y),
            Point::new(s.right() - 1, s.y),
            Point::new(s.x, s.bottom() - 1),
            Point::new(s.right() - 1, s.bottom() - 1),
        ];
        for p in inside {
            assert!(r.contains(scale.to_ui(p)), "{hit:?} at {p:?}");
        }
        let outside = [
            Point::new(s.x - 1, s.y),
            Point::new(s.right(), s.y),
            Point::new(s.x, s.y - 1),
            Point::new(s.x, s.bottom()),
        ];
        for p in outside {
            assert!(!r.contains(scale.to_ui(p)), "{hit:?} past {p:?}");
        }
    }
}

/// Runs the start job a client hands out on the synthetic data.
pub(super) fn run_start(c: &mut Client) {
    run_start_with(c, fixture::rules());
}

/// Runs the start job a client hands out on the synthetic map with `rules`.
pub(super) fn run_start_with(c: &mut Client, rules: std::sync::Arc<glasnost_sim::rules::SimRules>) {
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
    let gs = ui_setup::game_setup(&lobby, &rules, &offered, seed, seed)
        .expect("the setup makes a game setup");
    let s = Loaded::bare(fixture::map(), rules, fixture::catalogue(), &gs)
        .and_then(|l| Session::bare(l, "map-synthetic".into(), local));
    c.done(Done::Started(vfs, Box::new(s), SoundBank::new()));
    assert_eq!(c.shell.screen, Screen::Game);
    let _ = c.frame();
}

/// Starts a skirmish from the menu with the keyboard only.
pub(super) fn start_by_keyboard(c: &mut Client) {
    start_by_keyboard_with(c, fixture::rules());
}

/// Starts a skirmish from the menu with the keyboard only, on `rules`.
pub(super) fn start_by_keyboard_with(
    c: &mut Client,
    rules: std::sync::Arc<glasnost_sim::rules::SimRules>,
) {
    // Enter focuses the first control (Skirmish), the second press opens it.
    key(c, Key::Enter);
    assert_eq!(
        c.menu_focus.target(&c.menu_targets),
        Some(MenuHit::Skirmish)
    );
    key(c, Key::Enter);
    assert_eq!(c.shell.screen, Screen::Setup);
    // Shift+Tab from nothing goes to the last control, Start.
    c.modifiers(Modifiers {
        force_select: true,
        ..Modifiers::default()
    });
    key(c, Key::Tab);
    c.modifiers(Modifiers::default());
    assert_eq!(
        c.setup_focus.target(&c.setup_targets),
        Some(SetupHit::Start_)
    );
    c.next_seed = 7;
    key(c, Key::Space);
    run_start_with(c, rules);
}

pub(super) fn session(c: &Client) -> &Session {
    c.session.as_ref().expect("a game")
}

pub(super) fn session_mut(c: &mut Client) -> &mut Session {
    c.session.as_mut().expect("a game")
}

pub(super) fn own(c: &Client, kind: Kind) -> Vec<ObjectId> {
    let p = &session(c).play;
    p.world()
        .objects
        .values()
        .filter(|o| o.owner == Some(p.me()) && o.kind == kind && o.on_map)
        .map(|o| o.id)
        .collect()
}

pub(super) fn steps(c: &mut Client, n: i32, done: impl Fn(&Client) -> bool) {
    for _ in 0..n {
        if done(c) {
            return;
        }
        session_mut(c).step();
    }
    assert!(done(c), "not reached in {n} ticks");
}

/// Moves the base unit two cells and deploys it (a selecting click, a
/// moving click and D, as a player does), until the first building icon
/// shows.
pub(super) fn deploy_base(c: &mut Client) {
    let mcv = *own(c, Kind::Vehicle).first().expect("a base unit");
    let cell = session(c).play.world().obj(mcv).unwrap().pos.cell();
    session_mut(c)
        .play
        .click_map(Button::Left, Some(mcv), cell, Modifiers::default());
    steps(c, 10, |c| session(c).play.world().tick >= 4);
    let to = CellXY::new(cell.x, cell.y + 2);
    session_mut(c)
        .play
        .click_map(Button::Left, None, to, Modifiers::default());
    steps(c, 400, |c| {
        session(c)
            .play
            .world()
            .obj(mcv)
            .is_some_and(|o| o.pos.cell() == to)
    });
    key(c, Key::Char('d'));
    steps(c, 400, |c| !own(c, Kind::Building).is_empty());
    steps(c, 50, |c| {
        let p = &session(c).play;
        !p.world().houses[usize::from(p.me().0)].econ.tech.icons[0].is_empty()
    });
    let _ = c.frame();
}

/// The screens and the game's interface hit what they draw at every scale:
/// the corners of every target's scaled rectangle map back into it, clicks
/// at scaled positions press the controls, and the battlefield keeps its
/// own pixels left of the scaled sidebar.
#[test]
fn targets_hit_at_every_scale() {
    for size in SIZES {
        for pct in SCALES {
            let mut c = client(size, pct);
            let scale = c.ui_scale();
            assert!(scale.ui.0 >= 640 && scale.ui.1 >= 480, "{size:?} {pct}");
            check_targets(scale, &c.menu_targets);
            // The settings screen.
            let r = c.menu_targets.rect_of(&MenuHit::Settings).unwrap();
            click(&mut c, r);
            assert_eq!(c.shell.screen, Screen::Settings);
            let _ = c.frame();
            check_targets(scale, &c.settings_targets);
            let r = c
                .settings_targets
                .rect_of(&SettingsHit::Step(Row::Sound, -1))
                .unwrap();
            click(&mut c, r);
            assert_eq!(c.settings.sound, 90, "{size:?} {pct}");
            let r = c.settings_targets.rect_of(&SettingsHit::Back).unwrap();
            click(&mut c, r);
            assert_eq!(c.shell.screen, Screen::Menu);
            let _ = c.frame();
            // The setup screen.
            let r = c.menu_targets.rect_of(&MenuHit::Skirmish).unwrap();
            click(&mut c, r);
            assert_eq!(c.shell.screen, Screen::Setup);
            let _ = c.frame();
            check_targets(scale, &c.setup_targets);
            let before = c.shell.setup.as_ref().unwrap().slots[0].colour;
            let r = c.setup_targets.rect_of(&SetupHit::Colour(0, 1)).unwrap();
            click(&mut c, r);
            let after = c.shell.setup.as_ref().unwrap().slots[0].colour;
            assert_ne!(before, after, "the colour stepped at {size:?} {pct}");
            let _ = c.frame();
            let r = c.setup_targets.rect_of(&SetupHit::Start_).unwrap();
            click(&mut c, r);
            run_start(&mut c);
            // The game: the battlefield image fills the screen left of the
            // scaled sidebar; the sidebar's targets hit.
            let (list, _) = c.frame();
            let s = session(&c);
            let m = s.map_rect();
            let layout = glasnost_ui::hud::Layout::new(scale.ui);
            assert_eq!(m, Rect::new(0, 0, scale.x(layout.map.right()), size.1));
            assert!(matches!(
                list.prims.first(),
                Some(Prim::Image { id: MAP_IMAGE, rect }) if *rect == m
            ));
            if !scale.is_one() {
                assert!(list.prims.iter().any(|p| matches!(p, Prim::Layer { .. })));
            }
            check_targets(scale, s.hud_targets());
            let r = s.hud_targets().rect_of(&HudHit::Options).unwrap();
            click(&mut c, r);
            assert!(session(&c).paused, "the options control at {size:?} {pct}");
            let _ = c.frame();
            check_targets(scale, session(&c).pause_targets());
            let r = session(&c)
                .pause_targets()
                .rect_of(&PauseHit::Resume)
                .unwrap();
            click(&mut c, r);
            assert!(!session(&c).paused, "resumed at {size:?} {pct}");
        }
    }
}

/// The largest scale is lowered on a small window, and the settings screen
/// says so.
#[test]
fn a_scale_too_large_is_lowered_and_named() {
    let mut c = client((1024, 768), 300);
    assert_eq!(c.ui_scale().percent, 160);
    let r = c.menu_targets.rect_of(&MenuHit::Settings).unwrap();
    click(&mut c, r);
    let (list, _) = c.frame();
    assert!(
        list.texts().contains(&"300% (160% fits)"),
        "{:?}",
        list.texts()
    );
}

/// Every control of the menu, the settings, the setup, the multiplayer and
/// the end screens is reached by Tab, pressed by Enter or Space as a click
/// would, and Escape goes back.
#[test]
fn keyboard_reaches_every_control_of_the_screens() {
    let mut c = client((1024, 768), 100);
    // The main menu: Tab visits every control once, then wraps.
    let stops = c.menu_targets.stops.len();
    assert_eq!(stops, 5);
    let mut seen = Vec::new();
    for _ in 0..stops {
        key(&mut c, Key::Tab);
        seen.push(c.menu_focus.target(&c.menu_targets).unwrap());
    }
    seen.dedup();
    assert_eq!(seen.len(), stops);
    key(&mut c, Key::Tab);
    assert_eq!(c.menu_focus.target(&c.menu_targets), Some(seen[0]));
    // The focus mark shows after a key and goes with the pointer.
    assert!(c.menu_focus.rect(&c.menu_targets).is_some());
    c.pointer_down(Button::Left, Point::new(0, 0));
    assert!(c.menu_focus.rect(&c.menu_targets).is_none());
    c.pointer_up(Button::Left, Point::new(0, 0), Modifiers::default());

    // The settings screen: 8 values (the name tips' since M8 A4 round 8b),
    // 10 bindings, Reset and Back.
    while c.menu_focus.target(&c.menu_targets) != Some(MenuHit::Settings) {
        key(&mut c, Key::Down);
    }
    key(&mut c, Key::Enter);
    assert_eq!(c.shell.screen, Screen::Settings);
    let _ = c.frame();
    assert_eq!(c.settings_targets.stops.len(), 20);
    key(&mut c, Key::Tab);
    assert_eq!(
        c.settings_focus.target(&c.settings_targets),
        Some(SettingsHit::Step(Row::Sound, 1))
    );
    key(&mut c, Key::Left);
    assert_eq!(c.settings.sound, 90, "Left steps the focused value down");
    key(&mut c, Key::Right);
    assert_eq!(c.settings.sound, 100);
    // Down to the captions, colours and name tips rows; Enter toggles them.
    for _ in 0..5 {
        key(&mut c, Key::Down);
    }
    key(&mut c, Key::Enter);
    assert!(!c.settings.captions);
    key(&mut c, Key::Down);
    key(&mut c, Key::Space);
    assert!(c.settings.colour_blind);
    key(&mut c, Key::Down);
    key(&mut c, Key::Enter);
    assert!(!c.settings.tool_tips);
    // A binding: Enter waits for a key, the letter rebinds.
    key(&mut c, Key::Tab);
    key(&mut c, Key::Enter);
    assert_eq!(c.rebinding, Some(0));
    key(&mut c, Key::Char('q'));
    assert_eq!(c.settings.bindings.key(Action::Stop), 'q');
    key(&mut c, Key::Escape);
    assert_eq!(c.shell.screen, Screen::Menu);

    // The multiplayer screen: a field takes the typing while focused.
    while c.menu_focus.target(&c.menu_targets) != Some(MenuHit::Multiplayer) {
        key(&mut c, Key::Tab);
    }
    key(&mut c, Key::Space);
    assert_eq!(c.shell.screen, Screen::Multiplayer);
    let _ = c.frame();
    key(&mut c, Key::Tab);
    assert_eq!(c.net_field(), Some(NetField::Name));
    c.netform.name.clear();
    c.typed("Ada");
    // Space types into the field; it presses nothing.
    c.key(Key::Space, true);
    c.typed(" ");
    c.key(Key::Space, false);
    c.typed("L");
    assert_eq!(c.netform.name, "Ada L");
    // Enter moves to the next field, as before; Tab leaves the fields.
    key(&mut c, Key::Enter);
    assert_eq!(c.net_field(), Some(NetField::Relay));
    assert_eq!(
        c.net_focus.target(&c.net_targets),
        Some(NetHit::Field(NetField::Relay))
    );
    let stops = c.net_targets.stops.len();
    for _ in 0..stops {
        key(&mut c, Key::Tab);
        if c.net_focus.target(&c.net_targets) == Some(NetHit::Back) {
            break;
        }
    }
    assert_eq!(c.net_field(), None, "no field types on the Back control");
    // Escape with no field typing goes back.
    key(&mut c, Key::Escape);
    assert_eq!(c.shell.screen, Screen::Menu);

    // The setup screen: every stop reached; Escape goes back.
    key(&mut c, Key::Up);
    while c.menu_focus.target(&c.menu_targets) != Some(MenuHit::Skirmish) {
        key(&mut c, Key::Up);
    }
    key(&mut c, Key::Enter);
    assert_eq!(c.shell.screen, Screen::Setup);
    let _ = c.frame();
    let stops = c.setup_targets.stops.len();
    let mut seen = Vec::new();
    for _ in 0..stops {
        key(&mut c, Key::Tab);
        seen.push(c.setup_focus.target(&c.setup_targets).unwrap());
    }
    let distinct: std::collections::BTreeSet<String> =
        seen.iter().map(|h| format!("{h:?}")).collect();
    assert_eq!(distinct.len(), stops, "every setup control once");
    // Left and Right step a focused choice both ways.
    while !matches!(
        c.setup_focus.target(&c.setup_targets),
        Some(SetupHit::Colour(0, _))
    ) {
        key(&mut c, Key::Tab);
    }
    let colour = |c: &Client| c.shell.setup.as_ref().unwrap().slots[0].colour;
    let before = colour(&c);
    key(&mut c, Key::Right);
    assert_ne!(colour(&c), before);
    key(&mut c, Key::Left);
    assert_eq!(colour(&c), before);
    key(&mut c, Key::Escape);
    assert_eq!(c.shell.screen, Screen::Menu);

    // The end screen: Enter twice (focus, press) goes to the menu.
    c.shell.game_over(End {
        ticks: 10,
        units: 1,
        rows: vec![EndRow {
            name: "Person".into(),
            local: true,
            won: true,
            built: 0,
            kills: 0,
            losses: 0,
            score: 0,
        }],
    });
    let _ = c.frame();
    key(&mut c, Key::Enter);
    assert!(matches!(c.shell.screen, Screen::End(_)));
    key(&mut c, Key::Enter);
    assert_eq!(c.shell.screen, Screen::Menu);
}

/// The open options menu is played with the keyboard: arrows and Tab move,
/// Enter and Space press, and its controls do what their clicks do; the
/// arrows still scroll as they do with the menu open.
/// spec: S-0201 claim 8 (no ticks while the options menu is open), S-0213 claim 23 (Escape opens it; holding an arrow scrolls and its release ends its contribution)
#[test]
fn the_options_menu_is_played_by_keyboard() {
    let mut c = client((1024, 768), 150);
    start_by_keyboard(&mut c);
    key(&mut c, Key::Escape);
    assert!(session(&c).paused);
    // The first key focuses the first control; Tab visits every control.
    key(&mut c, Key::Down);
    let stops = session(&c).pause_targets().stops.len();
    let mut seen = Vec::new();
    for _ in 0..stops {
        seen.push(session(&c).pause_focus().expect("a focused control"));
        key(&mut c, Key::Tab);
    }
    for hit in [PauseHit::Resume, PauseHit::Settings, PauseHit::Leave] {
        assert!(seen.contains(&hit), "{hit:?} in {seen:?}");
    }
    // The speed choice: Left and Right step it as its halves' clicks do.
    while !matches!(session(&c).pause_focus(), Some(PauseHit::Speed(_))) {
        key(&mut c, Key::Tab);
    }
    // spec: S-0201 claims 8 and 17 (the control starts at the index in force; the change is sent when the menu closes)
    let speed = session(&c).speed;
    assert_eq!(session(&c).menu_speed(), speed);
    key(&mut c, Key::Left);
    assert_ne!(session(&c).menu_speed(), speed);
    assert_eq!(session(&c).speed, speed, "not before the menu closes");
    key(&mut c, Key::Right);
    assert_eq!(session(&c).menu_speed(), speed);
    // An arrow's press on the open menu is still the arrow's (S-0213
    // claim 23); its release ends it.
    c.key(Key::Up, true);
    assert_eq!(c.held[2], session(&c).arrow_event_accepted());
    c.key(Key::Up, false);
    assert!(!c.held[2]);
    let _ = c.frame();
    // Back to Resume and press it.
    while session(&c).pause_focus() != Some(PauseHit::Resume) {
        key(&mut c, Key::Tab);
    }
    key(&mut c, Key::Enter);
    assert!(!session(&c).paused);
    // Settings from the options menu, then Leave.
    key(&mut c, Key::Escape);
    while session(&c).pause_focus() != Some(PauseHit::Settings) {
        key(&mut c, Key::Tab);
    }
    key(&mut c, Key::Space);
    assert_eq!(c.shell.screen, Screen::Settings);
    key(&mut c, Key::Escape);
    assert_eq!(c.shell.screen, Screen::Game);
    while session(&c).pause_focus() != Some(PauseHit::Leave) {
        key(&mut c, Key::Tab);
    }
    key(&mut c, Key::Enter);
    assert_eq!(c.shell.screen, Screen::Menu);
    assert!(c.session.is_none());
}

/// The arrows that walk the keyboard cell from `from` to `to` (a search
/// over the session's steps, written out again here: our own walk for the
/// tests).
fn arrow_path(from: CellXY, to: CellXY) -> Vec<Key> {
    let step = |c: CellXY, k: Key| {
        let even = (c.x + c.y).rem_euclid(2) == 0;
        let (x, y) = match (k, even) {
            (Key::Right, true) | (Key::Down, false) => (1, 0),
            (Key::Right, false) | (Key::Up, false) => (0, -1),
            (Key::Left, true) | (Key::Down, true) => (0, 1),
            _ => (-1, 0),
        };
        CellXY::new(c.x + x, c.y + y)
    };
    let mut seen = BTreeMap::new();
    seen.insert((from.x, from.y), None);
    let mut queue = std::collections::VecDeque::from([from]);
    while let Some(c) = queue.pop_front() {
        if c == to {
            break;
        }
        for k in [Key::Right, Key::Left, Key::Down, Key::Up] {
            let n = step(c, k);
            if (n.x - from.x).abs() + (n.y - from.y).abs() > 64 || seen.contains_key(&(n.x, n.y)) {
                continue;
            }
            seen.insert((n.x, n.y), Some((c, k)));
            queue.push_back(n);
        }
    }
    let mut path = Vec::new();
    let mut c = to;
    while let Some(Some((prev, k))) = seen.get(&(c.x, c.y)) {
        path.push(*k);
        c = *prev;
    }
    path.reverse();
    path
}

/// A whole build is played from the keyboard: the sidebar's tab and icon,
/// production, placement mode, the keyboard cell moved with Ctrl and the
/// arrows, and Enter to place. The orders are the ones the same clicks
/// make: the replay and the state hash match a game played by the pointer.
/// spec: S-0411 claims 10 and 18 (a left click on an icon starts it or, ready, enters placement mode), S-0214 claim 28 (a battlefield right click ends placement mode)
#[test]
fn a_building_is_built_and_placed_by_keyboard() {
    // Two games from the same setup: one by keyboard, one by pointer.
    let mut kb = client((1024, 768), 125);
    start_by_keyboard(&mut kb);
    let mut ptr = client((1024, 768), 125);
    start_by_keyboard(&mut ptr);
    for c in [&mut kb, &mut ptr] {
        deploy_base(c);
    }
    // Keyboard: Page Down shows the tab with icons and focuses its first
    // icon; Space starts it. Pointer: the tab and the icon clicked.
    key(&mut kb, Key::PageDown);
    assert_eq!(session(&kb).sidebar_focus(), Some(HudHit::Cameo(0)));
    key(&mut kb, Key::Space);
    let tab = session(&kb).play.shown_tab();
    let r = session(&ptr)
        .hud_targets()
        .rect_of(&HudHit::Tab(tab))
        .unwrap();
    click(&mut ptr, r);
    let _ = ptr.frame();
    let r = session(&ptr)
        .hud_targets()
        .rect_of(&HudHit::Cameo(0))
        .unwrap();
    click(&mut ptr, r);
    let orders = |c: &Client| session(c).play.game.replay().orders.clone();
    assert_eq!(orders(&kb), orders(&ptr), "the same production order");
    assert!(!orders(&kb).is_empty());
    let local = session(&kb).play.me();
    let t = session(&kb).play.world().houses[usize::from(local.0)]
        .econ
        .tech
        .icons[0][0]
        .t;
    let ready = move |c: &Client| {
        glasnost_sim::econ::sidebar::icon_state(session(c).play.world(), local, t)
            .is_some_and(|s| s.ready)
    };
    steps(&mut kb, 4000, ready);
    steps(&mut ptr, 4000, ready);
    let _ = kb.frame();
    let _ = ptr.frame();
    // Space on the ready icon enters placement mode with the keyboard cell
    // at the battlefield's centre.
    key(&mut kb, Key::Space);
    assert_eq!(session(&kb).play.tool, Tool::Place(t));
    assert!(session(&kb).aiming());
    let r = session(&ptr)
        .hud_targets()
        .rect_of(&HudHit::Cameo(0))
        .unwrap();
    click(&mut ptr, r);
    assert_eq!(session(&ptr).play.tool, Tool::Place(t));
    // A cell the preview shows green near the base.
    let yard = session(&kb)
        .play
        .world()
        .obj(own(&kb, Kind::Building)[0])
        .unwrap()
        .pos
        .cell();
    let spot = (-8..=8)
        .flat_map(|dy| (-8..=8).map(move |dx| CellXY::new(yard.x + dx, yard.y + dy)))
        .find(|&c| session(&kb).play.preview(c).is_some_and(|(_, ok)| ok))
        .expect("a legal cell near the base");
    // Ctrl and the arrows walk the keyboard cell there; the view follows.
    kb.modifiers(Modifiers {
        force_fire: true,
        ..Modifiers::default()
    });
    let at = session(&kb).aim_cell().expect("aiming");
    for k in arrow_path(at, spot) {
        let held = kb.held;
        key(&mut kb, k);
        assert_eq!(kb.held, held, "Ctrl and an arrow do not scroll");
    }
    assert_eq!(session(&kb).aim_cell(), Some(spot));
    kb.modifiers(Modifiers::default());
    let before = orders(&kb).len();
    key(&mut kb, Key::Enter);
    assert_eq!(session(&kb).play.tool, Tool::Select, "placed");
    assert_eq!(orders(&kb).len(), before + 1);
    assert_eq!(session(&kb).aim_cell(), None);
    session_mut(&mut ptr)
        .play
        .click_map(Button::Left, None, spot, Modifiers::default());
    assert_eq!(orders(&kb), orders(&ptr), "the same placement order");
    for _ in 0..30 {
        session_mut(&mut kb).step();
        session_mut(&mut ptr).step();
    }
    assert_eq!(
        session(&kb).play.game.state_hash(),
        session(&ptr).play.game.state_hash()
    );
}

/// Backspace on a focused icon is its right click, and ends keyboard
/// aiming as a battlefield right click does.
/// spec: S-0214 claim 28 (a battlefield right click ends placement mode with no order)
#[test]
fn backspace_is_the_right_click_of_the_keyboard() {
    let mut c = client((1024, 768), 100);
    start_by_keyboard(&mut c);
    deploy_base(&mut c);
    key(&mut c, Key::PageDown);
    key(&mut c, Key::Space);
    let orders = |c: &Client| session(c).play.game.replay().orders.len();
    let started = orders(&c);
    assert!(started > 0, "Space started the icon");
    // The build begins with the next tick.
    for _ in 0..3 {
        session_mut(&mut c).step();
    }
    let _ = c.frame();
    // Backspace: the icon's right click (a hold or cancel order).
    key(&mut c, Key::Backspace);
    assert_eq!(orders(&c), started + 1);
}

/// Captions show each announcement the sound side starts, for their time,
/// and none when the setting is off. The caption text itself (the entry's
/// text key, else its name) is tested in `glasnost_audio`
/// (`claim_26_caption_from_the_text_key`).
// spec: S-0701 claim 26 (the caption setting; 4 s, at most 3 at once, newest last stand as our interface choices)
#[test]
fn captions_follow_the_started_announcements() {
    let mut c = client((1024, 768), 100);
    c.captions.push("ALPHA", 0);
    assert_eq!(c.captions.texts(), ["ALPHA"]);
    c.settings.captions = false;
    c.settings_changed();
    assert!(c.captions.texts().is_empty(), "off clears them");
    // In a game the captions are drawn over the battlefield.
    c.settings.captions = true;
    start_by_keyboard(&mut c);
    c.captions.push("BETA", c.clock);
    let (list, _) = c.frame();
    assert!(list.texts().contains(&"BETA"));
}

/// The colour-blind setting changes the setup's swatches and the colours a
/// game starts with; it is kept, and off by default.
#[test]
fn colour_blind_colours_show_in_the_setup_and_start_the_game() {
    let mut c = client((1024, 768), 100);
    assert!(!Settings::default().colour_blind);
    let r = c.menu_targets.rect_of(&MenuHit::Skirmish).unwrap();
    click(&mut c, r);
    let _ = c.frame();
    // A chosen colour (not random) shows its swatch.
    let r = c.setup_targets.rect_of(&SetupHit::Colour(0, 1)).unwrap();
    click(&mut c, r);
    assert!(c.shell.setup.as_ref().unwrap().slots[0].colour.is_some());
    let fills = |c: &mut Client| {
        let (list, _) = c.frame();
        list.prims
            .iter()
            .filter_map(|p| match p {
                Prim::Fill { rgba, .. } => Some(*rgba),
                _ => None,
            })
            .collect::<Vec<_>>()
    };
    let ours = install::colour_blind_swatches();
    assert!(!fills(&mut c).iter().any(|f| ours.contains(f)));
    c.settings.colour_blind = true;
    let shown = fills(&mut c);
    assert!(
        shown.iter().any(|f| ours.contains(f)),
        "a chosen colour shows our swatch"
    );
    let r = c.setup_targets.rect_of(&SetupHit::Start_).unwrap();
    click(&mut c, r);
    match c.job.as_ref() {
        Some(Job::Start { colour_blind, .. }) => assert!(*colour_blind),
        _ => panic!("a start job"),
    }
}

/// Settings never reach the simulation: a game played through the client
/// at the largest scale, with colour-blind colours, captions off and the
/// keyboard's own keys pressed, has the same state hash at every tick as
/// one under the defaults.
#[test]
fn state_hash_is_the_same_under_every_accessibility_setting() {
    let play = |extreme: bool| {
        let mut c = client((1024, 768), if extreme { 300 } else { 100 });
        if extreme {
            c.settings.colour_blind = true;
            c.settings.captions = false;
            c.settings_changed();
        }
        start_by_keyboard(&mut c);
        let mut hashes = Vec::new();
        for i in 0..120 {
            if extreme && i % 10 == 0 {
                // Keys that move only the interface's focus.
                key(&mut c, Key::Tab);
                key(&mut c, Key::PageDown);
            }
            session_mut(&mut c).step();
            let _ = c.frame();
            hashes.push(session(&c).play.game.state_hash());
        }
        hashes
    };
    assert_eq!(play(false), play(true));
}

/// The reports the pathing-report test store keeps (M8 A4).
static PATH_REPORTS: std::sync::Mutex<Vec<Vec<u8>>> = std::sync::Mutex::new(Vec::new());

fn keep_path_report(name: &str, bytes: &[u8]) -> Result<String, String> {
    PATH_REPORTS.lock().unwrap().push(bytes.to_vec());
    Ok(format!("the test store as {name}"))
}

/// The mark and save keys of the pathing report (M8 A4) in a game: M marks
/// the present tick, P saves a report with the mark and the selection
/// through the client's store. Our own diagnostics; no spec applies.
#[test]
fn the_mark_and_report_keys_save_a_pathing_report() {
    use super::super::capture::Bundle;
    let mut c = client((1024, 768), 100);
    c.save_path_report = Some(keep_path_report);
    start_by_keyboard(&mut c);
    let mcv = *own(&c, Kind::Vehicle).first().expect("a base unit");
    session_mut(&mut c).play.selection = vec![mcv];
    // nocite: our own diagnostics (a test length)
    for _ in 0..40 {
        session_mut(&mut c).step();
    }
    let marked = session(&c).play.game.world.tick;
    key(&mut c, Key::Char('m'));
    for _ in 0..10 {
        session_mut(&mut c).step();
    }
    key(&mut c, Key::Char('p'));
    let saved = PATH_REPORTS.lock().unwrap().clone();
    let b = Bundle::from_bytes(saved.last().expect("a report was saved")).unwrap();
    assert_eq!(b.marks, vec![marked]);
    assert_eq!(b.look.selection, vec![mcv.0]);
    assert_eq!(b.capture_hash, session(&c).play.game.state_hash());
}

/// Presses a control of the open options menu.
fn press_pause(c: &mut Client, hit: PauseHit) {
    let _ = c.frame();
    let r = session(c)
        .pause_targets()
        .rect_of(&hit)
        .unwrap_or_else(|| panic!("{hit:?} on the options menu"));
    click(c, r);
}

/// The local house's pending orders.
fn pending(c: &Client) -> Vec<Order> {
    let p = &session(c).play;
    p.world().houses[usize::from(p.me().0)].pending.clone()
}

/// A skirmish's speed control: closing the menu with the index in force
/// sends nothing; closing it with another index sends the speed order,
/// which the next tick carries out, and the paced index holds from the
/// next cycle.
/// spec: S-0201 claim 8 (the control's step; the new paced period from the next cycle) and claim 17 (sent when the menu closes; setting the index already in force sends nothing; the order sets the game-speed index when carried out)
#[test]
fn a_skirmish_speed_change_is_sent_when_the_menu_closes() {
    let mut c = client((1024, 768), 100);
    start_by_keyboard(&mut c);
    let speed = session(&c).speed;
    let game = session(&c).play.world().game_speed;
    key(&mut c, Key::Escape);
    assert!(session(&c).paused);
    press_pause(&mut c, PauseHit::Speed(1));
    press_pause(&mut c, PauseHit::Speed(-1));
    press_pause(&mut c, PauseHit::Resume);
    assert!(!session(&c).paused);
    assert!(pending(&c).is_empty(), "the index in force: nothing sent");
    key(&mut c, Key::Escape);
    let step = if speed == 0 { -1 } else { 1 };
    press_pause(&mut c, PauseHit::Speed(step));
    let want = session(&c).menu_speed();
    assert_ne!(want, speed);
    assert_eq!(session(&c).speed, speed, "not before the menu closes");
    press_pause(&mut c, PauseHit::Resume);
    assert_eq!(session(&c).speed, want);
    assert_eq!(pending(&c), vec![Order::GameSpeed { speed: want }]);
    assert_eq!(session(&c).play.world().game_speed, game);
    session_mut(&mut c).step();
    assert_eq!(session(&c).play.world().game_speed, want);
    assert_eq!(session(&c).speed, want);
}

/// In a network game the menu's speed change goes to the lockstep outbox
/// and the index in force changes only when the order is carried out; the
/// menu stops no tick, says no pause and offers no settings screen.
/// spec: S-0201 claim 15 (the menu stops no tick in a network game) and claim 17 (the change is sent at the menu's close as an order of the player; the sender's own index changes only when the order is carried out)
#[test]
fn a_network_speed_change_goes_through_the_lockstep_orders() {
    let mut c = client((1024, 768), 100);
    start_by_keyboard(&mut c);
    {
        let s = session_mut(&mut c);
        s.network = true;
        s.play.outbox = Some(Vec::new());
    }
    let speed = session(&c).play.world().game_speed;
    key(&mut c, Key::Escape);
    assert!(session(&c).paused);
    let _ = c.frame();
    assert!(
        session(&c)
            .pause_targets()
            .rect_of(&PauseHit::Settings)
            .is_none()
    );
    let tick = session(&c).play.world().tick;
    session_mut(&mut c).step();
    assert_eq!(session(&c).play.world().tick, tick + 1, "ticks go on");
    let step = if speed == 0 { -1 } else { 1 };
    press_pause(&mut c, PauseHit::Speed(step));
    let want = session(&c).menu_speed();
    press_pause(&mut c, PauseHit::Resume);
    let out = session(&c).play.outbox.clone().unwrap();
    assert_eq!(out, vec![Order::GameSpeed { speed: want }]);
    assert!(pending(&c).is_empty());
    assert_eq!(session(&c).play.world().game_speed, speed);
    // Carried out (here by hand, as the lockstep session would at t+W):
    // the paced index follows after the tick.
    session_mut(&mut c).play.game.world.game_speed = want;
    session_mut(&mut c).step();
    assert_eq!(session(&c).speed, want);
}

/// A skirmish's menu does not open once the game has ended. After our
/// engine's surrender it opens again, offering only the leave; after the
/// defeat it shows the speed control, whose change is sent when the menu
/// closes and carried out by the next tick.
/// spec: S-0201 claim 15 (refused from the end of the game on, S-0414 claim 6; the refusal after resigning is a network game's) and claim 17 (in a skirmish watched on after the human player's defeat the control is shown, and a change takes effect as in claim 8, by the project lead's ruling of 2026-10-24)
#[test]
fn the_menu_is_refused_after_the_end_and_offers_the_speed_after_the_defeat() {
    let mut c = client((1024, 768), 100);
    start_by_keyboard(&mut c);
    assert_eq!(
        session(&c).leave_choices(),
        vec![PauseHit::Surrender, PauseHit::Leave]
    );
    key(&mut c, Key::Escape);
    press_pause(&mut c, PauseHit::Surrender);
    assert!(!session(&c).paused);
    // design: our engine's skirmish menu opens again after the surrender, with the leave only
    assert!(session_mut(&mut c).open_options(), "a skirmish reopens");
    assert_eq!(session(&c).leave_choices(), vec![PauseHit::Leave]);
    let _ = c.frame();
    assert!(
        session(&c)
            .pause_targets()
            .rect_of(&PauseHit::Surrender)
            .is_none()
    );
    press_pause(&mut c, PauseHit::Resume);
    session_mut(&mut c).step();
    let local = usize::from(session(&c).play.me().0);
    session_mut(&mut c).play.game.world.houses[local]
        .econ
        .result
        .defeated = true;
    assert!(session_mut(&mut c).open_options(), "defeated: it opens");
    let speed = session(&c).speed;
    let step = if speed == 0 { -1 } else { 1 };
    press_pause(&mut c, PauseHit::Speed(step));
    let want = session(&c).menu_speed();
    assert_ne!(want, speed);
    press_pause(&mut c, PauseHit::Resume);
    assert_eq!(session(&c).speed, want);
    assert_eq!(pending(&c), vec![Order::GameSpeed { speed: want }]);
    session_mut(&mut c).step();
    assert_eq!(session(&c).play.world().game_speed, want);
    let tick = session(&c).play.world().tick;
    session_mut(&mut c).play.game.world.econ.end.ended = Some(tick);
    assert!(!session_mut(&mut c).open_options(), "the game has ended");
    key(&mut c, Key::Escape);
    assert!(!session(&c).paused);
}

/// A network game's leave choices: two that surrender while at least two
/// human houses, our own among them, are undefeated, otherwise the leave
/// alone; after the surrender that keeps the player watching the menu is
/// refused until the house is defeated, and a defeated house's menu shows
/// no speed control.
/// spec: S-0201 claim 15 (what leaving offers in a network game; after resigning through the menu it does not open again until the house is defeated; a defeated player watching can open it) and claim 17 (no speed control once the player's own house is defeated, in a network game)
#[test]
fn a_network_menu_offers_the_surrenders_while_two_humans_stand() {
    let mut c = client((1024, 768), 100);
    start_by_keyboard(&mut c);
    let local = usize::from(session(&c).play.me().0);
    let other = usize::from(local == 0);
    {
        let s = session_mut(&mut c);
        s.network = true;
        s.play.outbox = Some(Vec::new());
    }
    assert_eq!(
        session(&c).leave_choices(),
        vec![PauseHit::Leave],
        "one human house"
    );
    session_mut(&mut c).play.game.world.houses[other]
        .econ
        .setup
        .person_played = true;
    assert_eq!(
        session(&c).leave_choices(),
        vec![PauseHit::Surrender, PauseHit::SurrenderLeave]
    );
    session_mut(&mut c).play.game.world.houses[other]
        .econ
        .result
        .defeated = true;
    assert_eq!(
        session(&c).leave_choices(),
        vec![PauseHit::Leave],
        "the other human house defeated"
    );
    session_mut(&mut c).play.game.world.houses[other]
        .econ
        .result
        .defeated = false;
    key(&mut c, Key::Escape);
    let _ = c.frame();
    let t = session(&c).pause_targets();
    assert!(t.rect_of(&PauseHit::Leave).is_none());
    assert!(t.rect_of(&PauseHit::SurrenderLeave).is_some());
    press_pause(&mut c, PauseHit::Surrender);
    assert!(!session(&c).paused);
    assert!(!session(&c).leave_after_surrender);
    assert_eq!(
        session(&c).play.outbox.clone().unwrap(),
        vec![Order::Surrender]
    );
    assert!(!session_mut(&mut c).open_options(), "resigned");
    session_mut(&mut c).play.game.world.houses[local]
        .econ
        .result
        .defeated = true;
    assert!(session_mut(&mut c).open_options(), "defeated: it opens");
    assert_eq!(
        session(&c).leave_choices(),
        vec![PauseHit::Leave],
        "the opener's house defeated"
    );
    let _ = c.frame();
    let t = session(&c).pause_targets();
    assert!(t.rect_of(&PauseHit::Speed(1)).is_none());
    assert!(t.rect_of(&PauseHit::Speed(-1)).is_none());
    assert!(t.rect_of(&PauseHit::Surrender).is_none());
    assert!(t.rect_of(&PauseHit::Leave).is_some());
}

/// A skirmish's active clock counts time without the input focus, in
/// which no tick runs, and not the time the options menu is open.
/// spec: S-0414 claim 23 (revision 16: in a skirmish the clock pauses while the options menu is open; time without the input focus counts) and S-0201 claim 9 (no cycle runs without focus)
#[test]
fn a_skirmish_clock_counts_lost_focus_but_not_the_menu() {
    let mut c = client((1024, 768), 100);
    start_by_keyboard(&mut c);
    let units = |c: &Client| session(c).play.clock_units;
    c.update(1_000);
    c.focus(false);
    let (tick, before) = (session(&c).play.world().tick, units(&c));
    c.update(1_160);
    c.update(1_320);
    assert_eq!(session(&c).play.world().tick, tick, "no tick without focus");
    assert_eq!(units(&c) - before, 20, "lost focus counts");
    c.focus(true);
    key(&mut c, Key::Escape);
    assert!(session(&c).paused);
    let before = units(&c);
    c.update(1_480);
    c.update(1_640);
    assert_eq!(units(&c), before, "the open menu does not count");
}
