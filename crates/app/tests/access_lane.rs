//! Local lane for accessibility (M8 step C3; never in CI): with
//! `GLASNOST_GAME_DIR` set, starts the same skirmish twice through the
//! windowless client, once with the house colours of the colour slots at
//! 100 % and once with our colour-blind colours at 200 %, and plays a
//! building's production and placement with the keyboard only in the
//! second. Writes into `local-lane/m8/c3/` (git-ignored; `GLASNOST_LANE_OUT`
//! redirects): `colours-off.png`, `colours-on-200.png`,
//! `keyboard-aim-200.png` (the keyboard cell with captions, when an
//! announcement started) and `keyboard-placed-200.png`. Prints counts and
//! the captions only.
//!
//! Run with `cargo test -p glasnost-app --test access_lane -- --ignored
//! --nocapture`. `GLASNOST_UI_MAP=n` picks the n-th offered map (default
//! 0), `GLASNOST_UI_TICKS=n` the ticks before the colour pictures (default
//! 300).

#![cfg(not(target_arch = "wasm32"))]

use std::rc::Rc;

use glasnost_app::game::client::{Client, Key, run_job};
use glasnost_app::game::play::{Button, Tool};
use glasnost_app::mapview::png;
use glasnost_map::CellXY;
use glasnost_sim::rules::Kind;
use glasnost_sim::targeting::Modifiers;
use glasnost_ui::geom::{Point, Rect};
use glasnost_ui::menu::MenuHit;
use glasnost_ui::setup::SetupHit;
use glasnost_ui::shell::Screen;
use glasnost_vfs::source::{FileSource, Unopened};
use glasnost_vfs::{ByteSource, MountOptions, PickedFile};

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
        let source: Rc<dyn ByteSource> = if ext.contains(&x) {
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

fn env_num(name: &str, default: usize) -> usize {
    std::env::var(name)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
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
    c.pointer_down(Button::Left, p);
    c.pointer_up(Button::Left, p, Modifiers::default());
    let _ = c.frame();
}

fn key(c: &mut Client, k: Key) {
    c.key(k, true);
    c.key(k, false);
    let _ = c.frame();
}

fn save(dir: &std::path::Path, c: &mut Client, name: &str) {
    let rgba = c.frame_rgba();
    let (w, h) = c.size;
    std::fs::write(
        dir.join(format!("{name}.png")),
        png::png(w as u32, h as u32, &rgba),
    )
    .unwrap();
    println!("wrote {name}.png");
}

fn ticks(c: &mut Client, clock: &mut i64, n: usize) {
    for _ in 0..n {
        *clock += 1000;
        c.update(*clock);
        if c.shell.screen != Screen::Game {
            break;
        }
    }
}

/// The arrows that walk the keyboard cell from `from` to `to` (the
/// session's steps written out again: our own walk for the lane).
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
    let mut seen = std::collections::BTreeMap::new();
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

/// Mounts the install and starts the skirmish of map `map` at `scale` with
/// colour-blind colours `blind`; deploys the base unit and plays `n` ticks.
fn game(dir: &std::path::Path, scale: u16, blind: bool, map: usize, n: usize) -> (Client, i64) {
    let mut c = Client::new();
    c.settings.ui_scale = scale;
    c.settings.colour_blind = blind;
    c.resize((1280, 960));
    c.mount(files(dir));
    run_jobs(&mut c);
    assert!(c.install.is_some(), "the install reads: {}", c.status);
    let _ = c.frame();
    let r = c.menu_targets.rect_of(&MenuHit::Skirmish).unwrap();
    click(&mut c, r);
    for _ in 0..map {
        let r = c.setup_targets.rect_of(&SetupHit::Map(1)).unwrap();
        click(&mut c, r);
    }
    // The second colour slot for the person (vermillion among ours), so
    // the two pictures compare one slot's colours.
    for _ in 0..2 {
        let r = c.setup_targets.rect_of(&SetupHit::Colour(0, 1)).unwrap();
        click(&mut c, r);
    }
    let r = c.setup_targets.rect_of(&SetupHit::Start_).unwrap();
    c.next_seed = 12345;
    click(&mut c, r);
    run_jobs(&mut c);
    assert_eq!(
        c.shell.screen,
        Screen::Game,
        "the game starts: {}",
        c.status
    );
    let mut clock = 0i64;
    ticks(&mut c, &mut clock, 5);
    {
        let s = c.session.as_mut().unwrap();
        let local = s.play.me();
        let mcv = s
            .play
            .world()
            .objects
            .values()
            .find(|o| o.owner == Some(local) && o.kind == Kind::Vehicle && o.on_map)
            .map(|o| o.id);
        if let Some(id) = mcv {
            s.play.select_box(&[id], false);
        }
    }
    key(&mut c, Key::Char('d'));
    ticks(&mut c, &mut clock, n);
    (c, clock)
}

#[test]
#[ignore = "local lane: needs GLASNOST_GAME_DIR"]
fn colours_and_keyboard_play() {
    let Some(dir) = std::env::var_os("GLASNOST_GAME_DIR") else {
        eprintln!("GLASNOST_GAME_DIR not set: skipped");
        return;
    };
    let dir = std::path::PathBuf::from(dir);
    let out = png::lane_out("m8/c3");
    std::fs::create_dir_all(&out).unwrap();
    let map = env_num("GLASNOST_UI_MAP", 0);
    let n = env_num("GLASNOST_UI_TICKS", 300);

    let (mut plain, _) = game(&dir, 100, false, map, n);
    save(&out, &mut plain, "colours-off");
    drop(plain);

    let (mut c, mut clock) = game(&dir, 200, true, map, n);
    save(&out, &mut c, "colours-on-200");

    // The first building icon, by keyboard: Page Down shows the tab with
    // icons, Space starts its first icon and, once ready, takes it up.
    key(&mut c, Key::PageDown);
    key(&mut c, Key::Space);
    let s = c.session.as_ref().unwrap();
    let local = s.play.me();
    let Some(t) = s.play.world().houses[usize::from(local.0)].econ.tech.icons[0]
        .first()
        .map(|i| i.t)
    else {
        println!("no building icon: keyboard pictures not written");
        return;
    };
    let mut ready = false;
    for _ in 0..6000 {
        let s = c.session.as_ref().unwrap();
        ready = glasnost_sim::econ::sidebar::icon_state(s.play.world(), local, t)
            .is_some_and(|st| st.ready);
        if ready {
            break;
        }
        clock += 1000;
        c.update(clock);
    }
    if !ready {
        println!("the building did not become ready: keyboard pictures not written");
        return;
    }
    let _ = c.frame();
    key(&mut c, Key::Space);
    let s = c.session.as_mut().unwrap();
    assert!(matches!(s.play.tool, Tool::Place(_)), "placement mode");
    let from = s.aim_cell().expect("the keyboard cell aims");
    // From two cells away on, so that the picture shows a keyboard walk.
    let spot = (2..=12)
        .flat_map(|r| {
            (-r..=r)
                .flat_map(move |dy| (-r..=r).map(move |dx| CellXY::new(from.x + dx, from.y + dy)))
        })
        .filter(|c| (c.x - from.x).abs().max((c.y - from.y).abs()) >= 2)
        .find(|&cell| s.play.preview(cell).is_some_and(|(_, ok)| ok));
    let Some(spot) = spot else {
        println!("no legal cell near the view's centre: placement not shown");
        return;
    };
    c.modifiers(Modifiers {
        force_fire: true,
        ..Modifiers::default()
    });
    let path = arrow_path(from, spot);
    for k in &path {
        key(&mut c, *k);
    }
    c.modifiers(Modifiers::default());
    println!(
        "keyboard cell walked {} steps; captions: {:?}",
        path.len(),
        c.captions.texts()
    );
    save(&out, &mut c, "keyboard-aim-200");
    key(&mut c, Key::Enter);
    let orders = c.session.as_ref().unwrap().play.game.replay().orders.len();
    ticks(&mut c, &mut clock, 60);
    println!(
        "placed: tool {:?}, {orders} orders; captions: {:?}",
        c.session.as_ref().unwrap().play.tool,
        c.captions.texts()
    );
    save(&out, &mut c, "keyboard-placed-200");
}
