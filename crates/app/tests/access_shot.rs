//! Pictures of the accessibility settings (M8 step C3) on the CPU path,
//! with synthetic data only (invented rules and map,
//! `glasnost_ai::fixture`; no install). Writes PNG files into the folder
//! named by `GLASNOST_SHOT_DIR` when it is set; otherwise it only checks
//! that the screens draw:
//!
//! - `menu-100.png`, `menu-200.png`: the main menu at 1280 × 960 at 100 %
//!   and 200 %, the keyboard focus on Skirmish;
//! - `settings-150.png`: the settings screen at 150 %, the focus on a value;
//! - `setup-colours-off.png`, `setup-colours-on.png`: the setup screen with
//!   the colour slots' own swatches and with our colour-blind ones;
//! - `game-200.png`: a game at 200 %, the sidebar's focus on an icon, the
//!   keyboard cell aiming a placement and a caption over the battlefield
//!   (the battlefield is black: the synthetic map has no art);
//! - `options-200.png`: the options menu at 200 %, the focus on Resume.

#![cfg(not(target_arch = "wasm32"))]

use glasnost_ai::fixture;
use glasnost_app::game::client::{Client, Done, Job, Key};
use glasnost_app::game::install::Install;
use glasnost_app::game::play::{Button, Tool};
use glasnost_app::game::session::Session;
use glasnost_app::mapview::load::Loaded;
use glasnost_app::mapview::png;
use glasnost_audio::bank::SoundBank;
use glasnost_convert::find::Listing;
use glasnost_fmt::mix::MixEntry;
use glasnost_sim::rules::Kind;
use glasnost_sim::targeting::Modifiers;
use glasnost_ui::geom::Point;
use glasnost_ui::hud::HudHit;
use glasnost_ui::menu::MenuHit;
use glasnost_ui::setup::{self as ui_setup, MapEntry, SetupHit, SetupRules};
use glasnost_ui::shell::Screen;
use glasnost_vfs::{ArchiveId, FileRef, Vfs};

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
            // Invented swatches, one per slot.
            colours: (0..8u8)
                .map(|i| [40 + 25 * i, 200 - 20 * i, 120, 255])
                .collect(),
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

fn client(scale: u16) -> Client {
    let mut c = Client::new();
    c.settings.ui_scale = scale;
    c.resize((1280, 960));
    c.done(Done::Prepared(Vfs::default(), Ok(Box::new(install()))));
    let _ = c.frame();
    c
}

fn key(c: &mut Client, k: Key) {
    c.key(k, true);
    c.key(k, false);
    let _ = c.frame();
}

fn shoot(c: &mut Client, name: &str) {
    let rgba = c.frame_rgba();
    assert_eq!(rgba.len(), (c.size.0 * c.size.1 * 4) as usize);
    if let Ok(dir) = std::env::var("GLASNOST_SHOT_DIR") {
        std::fs::create_dir_all(&dir).unwrap();
        let path = std::path::Path::new(&dir).join(name);
        std::fs::write(path, png::png(c.size.0 as u32, c.size.1 as u32, &rgba)).unwrap();
    }
}

fn click(c: &mut Client, r: glasnost_ui::geom::Rect) {
    let r = c.ui_scale().rect(r);
    let p = Point::new(r.x + r.w / 2, r.y + r.h / 2);
    c.pointer_down(Button::Left, p);
    c.pointer_up(Button::Left, p, Modifiers::default());
    let _ = c.frame();
}

fn start(c: &mut Client) {
    let r = c.menu_targets.rect_of(&MenuHit::Skirmish).unwrap();
    click(c, r);
    let r = c.setup_targets.rect_of(&SetupHit::Start_).unwrap();
    click(c, r);
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
    let gs = ui_setup::game_setup(&lobby, &fixture::rules(), &offered, seed, seed).unwrap();
    let s = Loaded::bare(fixture::map(), fixture::rules(), fixture::catalogue(), &gs)
        .and_then(|l| Session::bare(l, "map-synthetic".into(), local));
    c.done(Done::Started(vfs, Box::new(s), SoundBank::new()));
    assert_eq!(c.shell.screen, Screen::Game);
}

#[test]
fn the_menu_and_settings_draw_at_their_scales() {
    for pct in [100, 200] {
        let mut c = client(pct);
        key(&mut c, Key::Tab);
        assert_eq!(
            c.menu_focus.target(&c.menu_targets),
            Some(MenuHit::Skirmish)
        );
        shoot(&mut c, &format!("menu-{pct}.png"));
    }
    let mut c = client(150);
    let r = c.menu_targets.rect_of(&MenuHit::Settings).unwrap();
    click(&mut c, r);
    for _ in 0..5 {
        key(&mut c, Key::Down);
    }
    shoot(&mut c, "settings-150.png");
}

#[test]
fn the_setup_shows_both_swatch_sets() {
    let mut c = client(100);
    let r = c.menu_targets.rect_of(&MenuHit::Skirmish).unwrap();
    click(&mut c, r);
    for i in 0..2 {
        for _ in 0..=i {
            let r = c.setup_targets.rect_of(&SetupHit::Colour(i, 1)).unwrap();
            click(&mut c, r);
        }
    }
    shoot(&mut c, "setup-colours-off.png");
    c.settings.colour_blind = true;
    shoot(&mut c, "setup-colours-on.png");
}

#[test]
fn the_game_draws_at_twice_the_size_with_the_keyboard_marks() {
    let mut c = client(200);
    start(&mut c);
    // Deploy the base unit and build the first icon (synthetic rules).
    let local = c.session.as_ref().unwrap().play.me();
    let own = |c: &Client, kind: Kind| {
        let p = &c.session.as_ref().unwrap().play;
        p.world()
            .objects
            .values()
            .filter(|o| o.owner == Some(local) && o.kind == kind && o.on_map)
            .map(|o| o.id)
            .collect::<Vec<_>>()
    };
    let step = |c: &mut Client, n: i32, done: &dyn Fn(&Client) -> bool| {
        for _ in 0..n {
            if done(c) {
                return;
            }
            c.session.as_mut().unwrap().step();
        }
        assert!(done(c));
    };
    let mcv = own(&c, Kind::Vehicle)[0];
    let cell = c
        .session
        .as_ref()
        .unwrap()
        .play
        .world()
        .obj(mcv)
        .unwrap()
        .pos
        .cell();
    let s = c.session.as_mut().unwrap();
    s.play
        .click_map(Button::Left, Some(mcv), cell, Modifiers::default());
    step(&mut c, 10, &|c| {
        c.session.as_ref().unwrap().play.world().tick >= 4
    });
    let to = glasnost_map::CellXY::new(cell.x, cell.y + 2);
    c.session
        .as_mut()
        .unwrap()
        .play
        .click_map(Button::Left, None, to, Modifiers::default());
    step(&mut c, 400, &|c| {
        c.session
            .as_ref()
            .unwrap()
            .play
            .world()
            .obj(mcv)
            .is_some_and(|o| o.pos.cell() == to)
    });
    key(&mut c, Key::Char('d'));
    step(&mut c, 400, &|c| !own(c, Kind::Building).is_empty());
    step(&mut c, 50, &|c| {
        let p = &c.session.as_ref().unwrap().play;
        !p.world().houses[usize::from(local.0)].econ.tech.icons[0].is_empty()
    });
    let _ = c.frame();
    key(&mut c, Key::PageDown);
    key(&mut c, Key::Space);
    let t = c.session.as_ref().unwrap().play.world().houses[usize::from(local.0)]
        .econ
        .tech
        .icons[0][0]
        .t;
    step(&mut c, 4000, &|c| {
        glasnost_sim::econ::sidebar::icon_state(c.session.as_ref().unwrap().play.world(), local, t)
            .is_some_and(|s| s.ready)
    });
    let _ = c.frame();
    key(&mut c, Key::Space);
    assert_eq!(c.session.as_ref().unwrap().play.tool, Tool::Place(t));
    c.modifiers(Modifiers {
        force_fire: true,
        ..Modifiers::default()
    });
    key(&mut c, Key::Right);
    key(&mut c, Key::Right);
    c.modifiers(Modifiers::default());
    assert_eq!(
        c.session.as_ref().unwrap().sidebar_focus(),
        Some(HudHit::Cameo(0))
    );
    c.captions.push("invented announcement one", 0);
    shoot(&mut c, "game-200.png");
    // The options menu with its focus.
    key(&mut c, Key::Escape);
    key(&mut c, Key::Tab);
    shoot(&mut c, "options-200.png");
}
