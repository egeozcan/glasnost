//! Local lane for M8 A3 intake 39, S-0207 r82 (never in CI): with
//! `GLASNOST_GAME_DIR` set, starts a skirmish through the windowless client,
//! sends driving vehicles of the local house across the map and watches
//! every chrono miner (a harvester with the teleporter flag) tick by tick:
//! how often it jumps, how many jumps end on a docking cell, and how far it
//! drives. Under claim 28 a flagged miner drives everywhere and teleports
//! only onto a refinery's docking cell. Screens go to `local-lane/m8/a3-39/`
//! (git-ignored; `GLASNOST_LANE_OUT` moves it). Types are chosen by their
//! fields, never by name; it prints counts, states and booleans only.
//!
//! Run with `cargo test -p glasnost-app --test drive_lane -- --ignored
//! --nocapture`. `GLASNOST_UI_MAP=n` picks the n-th offered map (default
//! 0), `GLASNOST_UI_COUNTRY=k` steps the local slot's country k times, and
//! `GLASNOST_UI_TICKS` sets the watch length (default 4500).

#![cfg(not(target_arch = "wasm32"))]

use std::collections::BTreeMap;
use std::rc::Rc;

use glasnost_app::game::client::{Client, Key, run_job};
use glasnost_app::game::play::Button;
use glasnost_app::mapview::png;
use glasnost_map::CellXY;
use glasnost_rules::sim::Locomotor;
use glasnost_sim::econ::{harvest, resource};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::targeting::Modifiers;
use glasnost_sim::world::{HouseId, ObjectId, Target, World};
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
    let _ = c.frame();
    let p = Point::new(r.x + r.w / 2, r.y + r.h / 2);
    c.pointer_moved(p);
    c.pointer_down(Button::Left, p);
    c.pointer_up(Button::Left, p, Modifiers::default());
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

fn save(c: &mut Client, dir: &std::path::Path, name: &str) {
    let rgba = c.frame_rgba();
    let (w, h) = c.size;
    std::fs::write(
        dir.join(format!("{name}.png")),
        png::png(w as u32, h as u32, &rgba),
    )
    .unwrap();
    println!("wrote {name}.png");
}

fn look_at(c: &mut Client, cell: CellXY) {
    let s = c.session.as_mut().unwrap();
    let (x, y) = glasnost_render::projection::cell_centre(cell.x, cell.y);
    s.set_centre(glasnost_render::projection::project_lifted(x, y, 0));
}

fn world(c: &Client) -> &World {
    c.session.as_ref().unwrap().play.world()
}

fn far(a: CellXY, b: CellXY) -> i32 {
    (a.x - b.x).abs().max((a.y - b.y).abs())
}

/// The miner kind of a vehicle type: chrono miner (flagged), harvester, or
/// none for any other type (slave miners included).
fn miner_kind(w: &World, ti: u32) -> Option<&'static str> {
    let u = w.rules().econ.harvest.vehicles.get(ti as usize)?;
    if !u.harvester || u.slave_miner {
        return None;
    }
    Some(if u.teleporter {
        "chrono miner"
    } else {
        "harvester"
    })
}

/// A house's construction yard cell, or its base unit's.
fn home_of(w: &World, house: HouseId) -> Option<CellXY> {
    let r = w.rules();
    let own = || {
        w.objects
            .values()
            .filter(move |o| o.on_map && o.health > 0 && o.owner == Some(house))
    };
    own()
        .find(|o| {
            o.kind == Kind::Building
                && r.buildings
                    .get(o.type_index as usize)
                    .is_some_and(|b| b.raw.construction_yard == Some(true))
        })
        .or_else(|| own().find(|o| o.kind == Kind::Vehicle))
        .map(|o| o.pos.cell())
}

/// Places a complete refinery the miner type `ti` docks at for `house`, on
/// the first cell near `home` its own placement preview allows.
fn place_refinery(c: &mut Client, house: HouseId, home: CellXY, ti: u32) -> Option<ObjectId> {
    let s = c.session.as_mut().unwrap();
    let w = &mut s.play.game.world;
    let dock = *w.rules().econ.harvest.vehicles[ti as usize].docks.first()?;
    let t = glasnost_sim::econ::TypeRef {
        kind: Kind::Building,
        index: dock,
    };
    let name = w.rules().buildings[dock as usize].name.clone();
    for r in 3i32..14 {
        for dy in -r..=r {
            for dx in -r..=r {
                if dx.abs() != r && dy.abs() != r {
                    continue;
                }
                let cell = CellXY::new(home.x + dx, home.y + dy);
                let (foot, ok) = glasnost_app::mapview::sidebar::preview(w, house, t, cell);
                if ok {
                    let tl = CellXY::new(
                        foot.iter().map(|f| f.x).min()?,
                        foot.iter().map(|f| f.y).min()?,
                    );
                    return Some(glasnost_sim::testkit::spawn(
                        w,
                        Kind::Building,
                        &name,
                        house.0,
                        tl,
                        0,
                    ));
                }
            }
        }
    }
    None
}

/// Places a unit of vehicle type `ti` for `house` on the first free cell
/// around `at` with no resource, with `queued` as its queued behaviour.
fn place(
    c: &mut Client,
    ti: u32,
    house: HouseId,
    at: CellXY,
    queued: Option<Behaviour>,
) -> Option<ObjectId> {
    let s = c.session.as_mut().unwrap();
    let w = &mut s.play.game.world;
    let name = w.rules().vehicles[ti as usize].name.clone();
    for r in 2i32..12 {
        for dy in -r..=r {
            for dx in -r..=r {
                if dx.abs() != r && dy.abs() != r {
                    continue;
                }
                let cell = CellXY::new(at.x + dx, at.y + dy);
                if resource::resource_at(w, cell).is_some() {
                    continue;
                }
                if let Some(id) = glasnost_sim::testkit::spawn_in_play(
                    w,
                    Kind::Vehicle,
                    &name,
                    house.0,
                    cell,
                    0,
                    queued,
                ) {
                    return Some(id);
                }
            }
        }
    }
    None
}

/// What one watched chrono miner did.
#[derive(Default)]
struct Tally {
    jumps: u32,
    to_dock: u32,
    elsewhere: u32,
    driven: u32,
    teleporting_ticks: u32,
}

/// The docking cells of every refinery on the map.
fn docks(w: &World) -> Vec<CellXY> {
    w.objects
        .values()
        .filter(|b| b.kind == Kind::Building && b.on_map)
        .filter_map(|b| harvest::docking_cell(w, b.id))
        .collect()
}

#[test]
#[ignore = "local lane: needs GLASNOST_GAME_DIR"]
fn vehicles_drive_and_the_chrono_miner_jumps_only_to_dock() {
    let Some(dir) = std::env::var_os("GLASNOST_GAME_DIR") else {
        eprintln!("GLASNOST_GAME_DIR not set: skipped");
        return;
    };
    let out = png::lane_out("m8/a3-39");
    std::fs::create_dir_all(&out).unwrap();
    let mut c = Client::new();
    c.resize((1280, 800));
    c.mount(files(std::path::Path::new(&dir)));
    run_jobs(&mut c);
    let maps = c.install.as_ref().expect("the install reads").maps.len();
    let map = env_num("GLASNOST_UI_MAP", 0).min(maps - 1);
    let _ = c.frame();
    let r = c.menu_targets.rect_of(&MenuHit::Skirmish).unwrap();
    click(&mut c, r);
    for _ in 0..map {
        let r = c.setup_targets.rect_of(&SetupHit::Map(1)).unwrap();
        click(&mut c, r);
    }
    for _ in 0..env_num("GLASNOST_UI_COUNTRY", 0) {
        let _ = c.frame();
        let r = c.setup_targets.rect_of(&SetupHit::Country(0, 1)).unwrap();
        click(&mut c, r);
    }
    let _ = c.frame();
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
    let s = c.session.as_mut().unwrap();
    let local = s.play.me();
    let w = s.play.world();
    let mcv = w
        .objects
        .values()
        .find(|o| o.owner == Some(local) && o.kind == Kind::Vehicle && o.on_map)
        .map(|o| o.id)
        .expect("a base unit");
    s.play.select_box(&[mcv], false);
    c.key(Key::Char('d'), true);
    c.key(Key::Char('d'), false);
    ticks(&mut c, &mut clock, 200);
    // Every player house allied with every other: the game plays on.
    {
        let w = &mut c.session.as_mut().unwrap().play.game.world;
        let players: Vec<HouseId> = (0..w.houses.len())
            .filter(|&h| w.houses[h].start.is_some())
            .map(|h| HouseId(h as u8))
            .collect();
        for &h in &players {
            for &o in &players {
                if !w.houses[usize::from(h.0)].allies.contains(&o) {
                    w.houses[usize::from(h.0)].allies.push(o);
                }
            }
        }
    }
    let home = home_of(world(&c), local).expect("a base");
    let w = world(&c);
    let mut kinds: BTreeMap<&'static str, u32> = BTreeMap::new();
    for ti in 0..w.rules().vehicles.len() as u32 {
        if let Some(k) = miner_kind(w, ti) {
            kinds.entry(k).or_insert(ti);
        }
    }
    // Up to four driving types that are no miner and no teleporter.
    let drivers: Vec<u32> = (0..w.rules().vehicles.len() as u32)
        .filter(|&ti| {
            let v = &w.rules().vehicles[ti as usize];
            let h = w.rules().econ.harvest.vehicles.get(ti as usize);
            v.type_locomotor() == Locomotor::Drive
                && h.is_none_or(|h| !h.harvester && !h.slave_miner && !h.teleporter)
        })
        .take(4)
        .collect();
    println!(
        "miner kinds {:?}, driving types {}",
        kinds.keys(),
        drivers.len()
    );
    let mut placed_refinery = false;
    if let Some(&ti) = kinds.get("chrono miner") {
        placed_refinery = place_refinery(&mut c, local, home, ti).is_some();
    }
    println!("refinery placed {placed_refinery}");
    let mut chrono: Vec<(String, ObjectId)> = Vec::new();
    let mut watched: Vec<(String, ObjectId, CellXY)> = Vec::new();
    for (k, &ti) in &kinds {
        if let Some(id) = place(
            &mut c,
            ti,
            local,
            CellXY::new(home.x + 3, home.y + 3),
            Some(Behaviour::Harvest),
        ) {
            let cell = world(&c).obj(id).unwrap().pos.cell();
            watched.push((format!("placed {k}"), id, cell));
            if *k == "chrono miner" {
                chrono.push(("placed chrono miner".into(), id));
            }
        }
    }
    // The driving vehicles go about 20 cells off, towards the map inside (the start is near the
    // usable area's north-west edge).
    let dirs = [(20, 0), (0, 20), (15, 15), (20, 10)];
    for (i, &ti) in drivers.iter().enumerate() {
        let Some(id) = place(&mut c, ti, local, CellXY::new(home.x - 3, home.y - 3), None) else {
            continue;
        };
        let at = world(&c).obj(id).unwrap().pos.cell();
        let (dx, dy) = dirs[i % dirs.len()];
        let goal = CellXY::new((at.x + dx).max(1), (at.y + dy).max(1));
        let n =
            c.session
                .as_mut()
                .unwrap()
                .play
                .issue(vec![glasnost_sim::orders::Order::Behaviour {
                    actor: id,
                    behaviour: Behaviour::Move,
                    attack_move: false,
                    target: None,
                    dest: Some(Target::Cell(goal)),
                }]);
        let w = world(&c);
        let usable = w.cells.usable(goal);
        // Whether the driver's movement zone joins its cell with the goal (S-0212 claim 4);
        // read only from up-to-date regions.
        let reach = match w.rules().unit(Kind::Vehicle, ti).map(|u| u.zone) {
            Some(z) if !w.regions.dirty && !w.regions.region.is_empty() => {
                let a = glasnost_sim::regions::reach_in(w, at, z);
                let b = glasnost_sim::regions::reach_in(w, goal, z);
                format!("start {a:?}, goal {b:?}")
            }
            _ => "not read".to_string(),
        };
        println!(
            "driver {i}: from {at:?} to {goal:?} (usable {usable}; reach {reach}), move order taken {n}"
        );
        watched.push((format!("driver {i}"), id, at));
    }
    // Every house's own chrono miners join the tally as they appear.
    let mut tally: BTreeMap<ObjectId, (String, Tally, CellXY)> = BTreeMap::new();
    let watch = env_num("GLASNOST_UI_TICKS", 4500);
    let credits0 = world(&c).houses[usize::from(local.0)].credits;
    for done in 1..=watch {
        ticks(&mut c, &mut clock, 1);
        let w = world(&c);
        let dock_cells = docks(w);
        for o in w.objects.values().filter(|o| {
            o.on_map
                && o.kind == Kind::Vehicle
                && miner_kind(w, o.type_index) == Some("chrono miner")
        }) {
            let name = chrono
                .iter()
                .find(|(_, id)| *id == o.id)
                .map(|(n, _)| n.clone())
                .unwrap_or_else(|| {
                    format!(
                        "house {} chrono miner",
                        o.owner.map_or(-1, |h| i32::from(h.0))
                    )
                });
            let here = o.pos.cell();
            let e = tally
                .entry(o.id)
                .or_insert_with(|| (name, Tally::default(), here));
            let step = far(here, e.2);
            if step > 2 {
                e.1.jumps += 1;
                if dock_cells.contains(&here) {
                    e.1.to_dock += 1;
                } else {
                    e.1.elsewhere += 1;
                }
            } else {
                e.1.driven += step as u32;
            }
            if glasnost_sim::teleport::is_teleporter(w, o.id) {
                e.1.teleporting_ticks += 1;
            }
            e.2 = here;
        }
        if done % 300 == 0 {
            for (name, id, start) in &watched {
                let Some(o) = w.obj(*id) else {
                    println!("tick +{done} {name}: gone");
                    continue;
                };
                let phase = o.econ.harvest.as_ref().map(|s| s.phase);
                println!(
                    "tick +{done} {name}: {:?}/{phase:?} moved {} moving {} dest {} teleporting {}",
                    o.beh.current,
                    far(o.pos.cell(), *start),
                    !o.path.steps.is_empty(),
                    o.dest.is_some(),
                    glasnost_sim::teleport::is_teleporter(w, *id)
                );
            }
        }
        if done == 600 || done == 1800 {
            let shots: Vec<(String, CellXY)> = watched
                .iter()
                .filter_map(|(n, id, _)| world(&c).obj(*id).map(|o| (n.clone(), o.pos.cell())))
                .collect();
            for (name, cell) in shots {
                look_at(&mut c, cell);
                let _ = c.frame();
                save(
                    &mut c,
                    &out,
                    &format!("a3-39-{done}-{}", name.replace(' ', "-")),
                );
            }
        }
        if c.shell.screen != Screen::Game {
            println!("tick +{done}: the game left the game screen");
            break;
        }
    }
    let w = world(&c);
    println!(
        "local credits gained {}",
        w.houses[usize::from(local.0)].credits - credits0
    );
    for (name, t, _) in tally.values() {
        println!(
            "{name}: jumps {} (to a docking cell {}, elsewhere {}), cells driven {}, ticks with teleporting current {}",
            t.jumps, t.to_dock, t.elsewhere, t.driven, t.teleporting_ticks
        );
    }
}
