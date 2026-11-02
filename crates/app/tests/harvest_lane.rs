//! Local lane for the M8 A4 round 5 play notes on harvesting (never in CI):
//! with `GLASNOST_GAME_DIR` set, starts a skirmish through the windowless
//! client and watches every house's miners (notes 23 and 24) and the map's
//! ore mines (note 30). Miners of each kind are also placed by the local
//! base, queued to harvest as a factory releases them (S-0417 claim 8), one
//! set for the local house and one for a computer house. Screens go to
//! `local-lane/m8/a3-36/` (git-ignored; `GLASNOST_LANE_OUT` moves it);
//! `far_slave_miners_of_a_person_and_a_computer` (Q-1990) writes to
//! `local-lane/m8/a3-41/`.
//! Types are chosen by their fields, never by name; it prints counts,
//! states and booleans only.
//!
//! Run with `cargo test -p glasnost-app --test harvest_lane -- --ignored
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
use glasnost_sim::econ::{harvest, resource, slaves};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::targeting::Modifiers;
use glasnost_sim::world::{HouseId, ObjectId, World};
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

/// The miner kind of a vehicle type: slave miner, chrono miner or plain.
fn miner_kind(w: &World, ti: u32) -> Option<&'static str> {
    let u = w.rules().econ.harvest.vehicles.get(ti as usize)?;
    if !u.harvester && !u.slave_miner {
        return None;
    }
    Some(if u.slave_miner {
        "slave miner"
    } else if u.teleporter {
        "chrono miner"
    } else {
        "harvester"
    })
}

/// One line per house that owns objects: credits, its miners by
/// behaviour and phase, its slave groups and slaves.
fn report(w: &World, tag: &str) {
    for (h, house) in w.houses.iter().enumerate() {
        let id = HouseId(h as u8);
        let own: Vec<_> = w
            .objects
            .values()
            .filter(|o| o.on_map && o.health > 0 && o.owner == Some(id))
            .collect();
        if own.is_empty() {
            continue;
        }
        let mut miners: BTreeMap<String, usize> = BTreeMap::new();
        let mut groups: BTreeMap<String, usize> = BTreeMap::new();
        let mut slave_phases: BTreeMap<String, usize> = BTreeMap::new();
        for o in &own {
            if o.kind == Kind::Vehicle
                && let Some(k) = miner_kind(w, o.type_index)
            {
                let phase = o.econ.harvest.as_ref().map(|s| s.phase);
                *miners
                    .entry(format!("{k} {:?}/{phase:?}", o.beh.current))
                    .or_default() += 1;
            }
            if let Some(g) = slaves::group_of(w, o.id) {
                let out = g.members.iter().filter(|m| m.id.is_some() && !m.inside);
                *groups
                    .entry(format!("{:?} {:?} out {}", o.kind, g.state, out.count()))
                    .or_default() += 1;
            }
            if let Some(ss) = &o.econ.slave {
                *slave_phases
                    .entry(format!("{:?}/{:?}", o.beh.current, ss.phase))
                    .or_default() += 1;
            }
        }
        println!(
            "{tag} house {h} human {}: credits {}; miners {miners:?}; groups {groups:?}; slaves {slave_phases:?}",
            house.human, house.credits
        );
    }
}

/// The ore mines on the map (S-0401 claim 15): count, those animating now,
/// the resource cells within 2 cells of any mine, and how many of the cells
/// there without resource at the first call (`bare`) hold resource now.
fn mines(
    w: &World,
    tag: &str,
    bare: &mut Option<std::collections::BTreeSet<CellXY>>,
) -> Vec<CellXY> {
    let r = w.rules();
    let cells: Vec<CellXY> = w
        .objects
        .values()
        .filter(|o| o.kind == Kind::Terrain && o.on_map)
        .filter(|o| {
            r.terrain
                .get(o.type_index as usize)
                .is_some_and(|t| t.spawns_resources)
        })
        .map(|o| o.pos.cell())
        .collect();
    let frames: Vec<i32> = w
        .objects
        .values()
        .filter(|o| o.kind == Kind::Terrain && o.on_map)
        .filter_map(|o| r.terrain.get(o.type_index as usize))
        .filter(|t| t.spawns_resources)
        .map(|t| t.image_frames)
        .collect();
    let mut near = 0;
    let mut bales = 0i32;
    let mut seen = std::collections::BTreeSet::new();
    let mut empty = std::collections::BTreeSet::new();
    for c in &cells {
        for dy in -2..=2 {
            for dx in -2..=2 {
                let x = CellXY::new(c.x + dx, c.y + dy);
                if !seen.insert(x) {
                    continue;
                }
                match resource::resource_at(w, x) {
                    Some((_, d)) => {
                        near += 1;
                        bales += i32::from(d) + 1;
                    }
                    None => {
                        empty.insert(x);
                    }
                }
            }
        }
    }
    let bare = bare.get_or_insert(empty);
    let grown = bare
        .iter()
        .filter(|&&x| resource::resource_at(w, x).is_some())
        .count();
    let mut kinds: BTreeMap<i32, usize> = BTreeMap::new();
    for f in frames {
        *kinds.entry(f).or_default() += 1;
    }
    println!(
        "{tag} ore mines {} (by frame count {kinds:?}), animating {}, resource cells within 2 {near}, stages {bales}, cells bare at the start that hold resource now {grown} of {}",
        cells.len(),
        w.econ.resources.mines.len(),
        bare.len()
    );
    cells
}

/// Places a unit of vehicle type `ti` for `house`, queued to harvest as a
/// factory release does, on the first free cell around `at` with no
/// tiberium land within the slave-miner short scan range (S-0402 claim 14).
fn place_far(c: &mut Client, ti: u32, house: HouseId, at: CellXY) -> Option<ObjectId> {
    let s = c.session.as_mut().unwrap();
    let w = &mut s.play.game.world;
    let name = w.rules().vehicles[ti as usize].name.clone();
    let range = w.rules().econ.harvest.slave_short_scan;
    let ore_near = |w: &World, c: CellXY| {
        (-range..=range).any(|dy| {
            (-range..=range).any(|dx| {
                w.cells
                    .cell(CellXY::new(c.x + dx, c.y + dy))
                    .is_some_and(|x| x.land == glasnost_map::terrain::land::TIBERIUM)
            })
        })
    };
    for r in 2i32..40 {
        for dy in -r..=r {
            for dx in -r..=r {
                if dx.abs() != r && dy.abs() != r {
                    continue;
                }
                let cell = CellXY::new(at.x + dx, at.y + dy);
                if ore_near(w, cell) {
                    continue;
                }
                if let Some(id) = glasnost_sim::testkit::spawn_in_play(
                    w,
                    Kind::Vehicle,
                    &name,
                    house.0,
                    cell,
                    0,
                    Some(Behaviour::Harvest),
                ) {
                    return Some(id);
                }
            }
        }
    }
    None
}

/// Places a unit of vehicle type `ti` for `house` on the first free cell
/// around `at`, queued to harvest as a factory release does.
fn place_miner(c: &mut Client, ti: u32, house: HouseId, at: CellXY) -> Option<ObjectId> {
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
                    Some(Behaviour::Harvest),
                ) {
                    return Some(id);
                }
            }
        }
    }
    None
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

/// Places a complete refinery that the plain harvester docks at for
/// `house`, on the first cell near `home` where its own placement preview
/// allows it.
fn place_refinery(
    c: &mut Client,
    house: HouseId,
    home: CellXY,
    kinds: &BTreeMap<&'static str, u32>,
) -> Option<ObjectId> {
    let s = c.session.as_mut().unwrap();
    let w = &mut s.play.game.world;
    let ti = *kinds.get("harvester")?;
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
                    // The foundation's top-left cell.
                    let tl = CellXY::new(
                        foot.iter().map(|f| f.x).min()?,
                        foot.iter().map(|f| f.y).min()?,
                    );
                    let id = glasnost_sim::testkit::spawn(w, Kind::Building, &name, house.0, tl, 0);
                    return Some(id);
                }
            }
        }
    }
    None
}

/// Places a building type with a slave group through the placement of a
/// done item (S-0406 claim 1) near `home`; none when the house could not
/// make one.
fn place_slave_building(
    c: &mut Client,
    house: HouseId,
    home: CellXY,
) -> Option<(ObjectId, CellXY)> {
    let s = c.session.as_mut().unwrap();
    let w = &mut s.play.game.world;
    let types: Vec<u32> = (0..w.rules().econ.harvest.buildings.len() as u32)
        .filter(|&i| w.rules().econ.harvest.buildings[i as usize].group.is_some())
        .collect();
    println!("slave building types {}", types.len());
    for index in types {
        let t = glasnost_sim::econ::TypeRef {
            kind: Kind::Building,
            index,
        };
        for r in 3i32..14 {
            for dy in -r..=r {
                for dx in -r..=r {
                    if dx.abs() != r && dy.abs() != r {
                        continue;
                    }
                    let cell = CellXY::new(home.x + dx, home.y + dy);
                    if glasnost_sim::econ::placement::place_building(w, house, t, cell) {
                        let id = w
                            .objects
                            .values()
                            .filter(|o| o.kind == Kind::Building && o.type_index == index)
                            .map(|o| o.id)
                            .max()?;
                        return Some((id, cell));
                    }
                }
            }
        }
    }
    None
}

fn describe(w: &World, id: ObjectId, start: CellXY) -> String {
    let Some(o) = w.obj(id) else {
        return "gone".into();
    };
    let c = o.pos.cell();
    let moved = (c.x - start.x).abs().max((c.y - start.y).abs());
    let phase = o.econ.harvest.as_ref().map(|s| s.phase);
    let load: i32 = o.econ.harvest.as_ref().map_or(0, |s| s.load.iter().sum());
    let group = slaves::group_of(w, id).map(|g| {
        let out = g
            .members
            .iter()
            .filter(|m| m.id.is_some() && !m.inside)
            .count();
        format!("{:?} out {out}", g.state)
    });
    // The nearest refinery of the owner: distance to its docking cell and
    // who is docked there.
    let dock = w
        .objects
        .values()
        .filter(|b| b.kind == Kind::Building && b.owner == o.owner && b.on_map)
        .filter_map(|b| harvest::docking_cell(w, b.id).map(|d| (b.id, d)))
        .min_by_key(|(_, d)| (d.x - c.x).abs().max((d.y - c.y).abs()));
    let dock = dock.map(|(b, d)| {
        let docked = harvest::docked(w, b);
        let on: Vec<String> = w
            .cells
            .list(d, glasnost_sim::world::Layer::Ground)
            .iter()
            .filter_map(|&x| w.obj(x))
            .map(|x| {
                format!(
                    "{:?} me {} own {} {:?} dest {}",
                    x.kind,
                    x.id == id,
                    x.owner == o.owner,
                    x.beh.current,
                    x.dest.is_some()
                )
            })
            .collect();
        format!(
            "dock at {} docked {} me {} contact {} on dock {on:?}",
            (d.x - c.x).abs().max((d.y - c.y).abs()),
            docked.len(),
            docked.contains(&id),
            o.contact == Some(b)
        )
    });
    let dest = match o.dest {
        Some(glasnost_sim::world::Target::Cell(d)) => {
            format!("cell at {}", (d.x - c.x).abs().max((d.y - c.y).abs()))
        }
        Some(glasnost_sim::world::Target::Object(x)) => {
            format!("object {:?}", w.obj(x).map(|x| x.kind))
        }
        None => "none".into(),
    };
    format!(
        "{:?} {:?}/{phase:?} queued {:?} dest {dest} load {load} moved {moved} moving {} group {group:?} {dock:?}",
        o.kind,
        o.beh.current,
        o.beh.queued,
        !o.path.steps.is_empty()
    )
}

/// Starts a skirmish on the `GLASNOST_UI_MAP`-th map through the client,
/// deploys the local house's base unit and runs 200 ticks. Returns the
/// client, its clock and the local house.
/// Starts a skirmish on the chosen map through the setup screen. With
/// `allied`, the local slot and the computer slot are both put on team 1
/// in the lobby, so the game starts with them allied (S-0412 claim 8), as
/// a person sets up a game to watch the economy without either house
/// losing and ending it. (Writing the houses' alliance lists directly is
/// not what any game setup does: the sim keeps them sorted and rebuilds
/// what an alliance changes, S-0412 claim 8, S-0414 claim 12, and a two-
/// player game cannot ally in play, S-0414 claim 12.)
fn start(dir: &std::ffi::OsStr, allied: bool) -> (Client, i64, HouseId) {
    let mut c = Client::new();
    c.resize((1280, 800));
    c.mount(files(std::path::Path::new(dir)));
    run_jobs(&mut c);
    let maps = c.install.as_ref().expect("the install reads").maps.len();
    let map = env_num("GLASNOST_UI_MAP", 0).min(maps - 1);
    let _ = c.frame();
    let r = c.menu_targets.rect_of(&MenuHit::Skirmish).unwrap();
    click(&mut c, r);
    for _ in 0..map {
        let _ = c.frame();
        let r = c.setup_targets.rect_of(&SetupHit::Map(1)).unwrap();
        click(&mut c, r);
    }
    for _ in 0..env_num("GLASNOST_UI_COUNTRY", 0) {
        let _ = c.frame();
        let r = c.setup_targets.rect_of(&SetupHit::Country(0, 1)).unwrap();
        click(&mut c, r);
    }
    if allied {
        for slot in 0..2 {
            let _ = c.frame();
            // spec: S-0412 claim 15 (no team, then teams 1 … 4: one step gives team 1)
            let r = c.setup_targets.rect_of(&SetupHit::Team(slot, 1)).unwrap();
            click(&mut c, r);
        }
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
    println!(
        "map {map}, local house {} country {} human {}",
        local.0,
        w.houses[usize::from(local.0)].country,
        w.houses[usize::from(local.0)].human
    );
    let mcv = w
        .objects
        .values()
        .find(|o| o.owner == Some(local) && o.kind == Kind::Vehicle && o.on_map)
        .map(|o| (o.id, o.pos.cell()))
        .expect("a base unit");
    s.play.select_box(&[mcv.0], false);
    c.key(Key::Char('d'), true);
    c.key(Key::Char('d'), false);
    ticks(&mut c, &mut clock, 200);
    (c, clock, local)
}

/// Whether every two player houses are allied both ways in the sim's
/// view (S-0412 claim 8: lobby teams).
fn all_allied(w: &World) -> bool {
    let players: Vec<HouseId> = (0..w.houses.len())
        .filter(|&h| w.houses[h].start.is_some())
        .map(|h| HouseId(h as u8))
        .collect();
    players
        .iter()
        .all(|&a| players.iter().all(|&b| w.allied(Some(a), Some(b))))
}

/// The other player house with the most objects: the computer's.
fn computer_of(w: &World, local: HouseId) -> Option<HouseId> {
    (0..w.houses.len())
        .filter(|&h| h != usize::from(local.0) && w.houses[h].start.is_some())
        .max_by_key(|&h| {
            w.objects
                .values()
                .filter(|o| o.owner == Some(HouseId(h as u8)))
                .count()
        })
        .map(|h| HouseId(h as u8))
}

/// Q-1990 (S-0402 r80 claim 14): a slave miner of the local house (a
/// person's) and one of the computer's (an AI house) placed with no
/// order far from any field, off tiberium land and with no tiberium land
/// within the short scan. The person's stays put; the computer's brain
/// sends its own with a harvest order on a field cell it has explored,
/// and it travels, deploys and its slaves work. Prints, per miner, the
/// first tick it had a destination, moved, deployed and had a slave out;
/// screens of each at the end go to `local-lane/m8/a3-41/`. The two houses
/// start allied by lobby team (S-0412 claim 8), and nothing of either
/// targets the other's (M8 step B9).
#[test]
#[ignore = "local lane: needs GLASNOST_GAME_DIR"]
fn far_slave_miners_of_a_person_and_a_computer() {
    let Some(dir) = std::env::var_os("GLASNOST_GAME_DIR") else {
        eprintln!("GLASNOST_GAME_DIR not set: skipped");
        return;
    };
    let out = png::lane_out("m8/a3-41");
    std::fs::create_dir_all(&out).unwrap();
    let (mut c, mut clock, local) = start(&dir, true);
    let w = world(&c);
    assert!(all_allied(w), "the lobby teams ally the players");
    let computer = computer_of(w, local).expect("a computer house");
    println!(
        "computer house {} human {} (an AI house)",
        computer.0,
        w.houses[usize::from(computer.0)].human
    );
    let ti = (0..w.rules().vehicles.len() as u32)
        .find(|&ti| miner_kind(w, ti) == Some("slave miner"))
        .expect("a slave miner type");
    let mut watched = Vec::new();
    for (who, house) in [("person", local), ("computer", computer)] {
        let home = home_of(world(&c), house).expect("a base");
        let id = place_far(&mut c, ti, house, home).expect("placed");
        let w = world(&c);
        let cell = w.obj(id).unwrap().pos.cell();
        // The nearest tiberium land, in cells.
        let ore = (-150i32..=150)
            .flat_map(|dy| (-150i32..=150).map(move |dx| CellXY::new(cell.x + dx, cell.y + dy)))
            .filter(|&o| {
                w.cells
                    .cell(o)
                    .is_some_and(|x| x.land == glasnost_map::terrain::land::TIBERIUM)
            })
            .map(|o| (o.x - cell.x).abs().max((o.y - cell.y).abs()))
            .min();
        println!(
            "{who}: miner placed at tick {}, nearest tiberium land {ore:?} cells, short scan {} cells",
            w.tick,
            w.rules().econ.harvest.slave_short_scan
        );
        // (who, house, the vehicle, its cell, its master now, its last
        // cell, first ticks of [destination, moved, deployed, slave out]).
        watched.push((who, house, id, cell, id, cell, [None::<i32>; 4]));
    }
    let watch = env_num("GLASNOST_UI_TICKS", 4500);
    let from = world(&c).tick;
    let mut hp: BTreeMap<ObjectId, i32> = BTreeMap::new();
    let mut first_hit: BTreeMap<ObjectId, i32> = BTreeMap::new();
    let mut allied_targets = 0usize;
    for done in 1..=watch {
        ticks(&mut c, &mut clock, 1);
        let w = world(&c);
        let at_ally = w.objects.values().any(|x| {
            let Some(glasnost_sim::world::Target::Object(t)) = x.target else {
                return false;
            };
            let to = w.obj(t).and_then(|t| t.owner);
            x.owner != to
                && x.owner
                    .is_some_and(|h| w.houses[usize::from(h.0)].start.is_some())
                && w.allied(x.owner, to)
        });
        if at_ally {
            allied_targets += 1;
        }
        for (_, house, id, cell, master, last, firsts) in &mut watched {
            let t = w.tick - from;
            match w.obj(*id) {
                Some(o) => {
                    if o.dest.is_some() {
                        firsts[0].get_or_insert(t);
                    }
                    if o.pos.cell() != *cell {
                        firsts[1].get_or_insert(t);
                    }
                    *last = o.pos.cell();
                    // Damage taken, and who has it as their target.
                    if hp
                        .insert(*id, o.health)
                        .is_some_and(|before| o.health < before)
                    {
                        first_hit.entry(*id).or_insert(t);
                        let by: Vec<_> = w
                            .objects
                            .values()
                            .filter(|x| x.target == Some(glasnost_sim::world::Target::Object(*id)))
                            .map(|x| (x.kind, x.owner.map(|h| h.0)))
                            .collect();
                        println!(
                            "tick +{t}: watched vehicle of house {} hit, health {}, targeted by {by:?}",
                            house.0, o.health
                        );
                    }
                }
                None => {
                    if firsts[2].is_none() {
                        // Who stood near its last cell when it left the map.
                        let near: Vec<String> = w
                            .objects
                            .values()
                            .filter(|x| {
                                let p = x.pos.cell();
                                x.on_map && (p.x - last.x).abs().max((p.y - last.y).abs()) <= 6
                            })
                            .map(|x| {
                                format!(
                                    "{:?} house {:?} own {} {:?}",
                                    x.kind,
                                    x.owner.map(|h| h.0),
                                    x.owner == Some(*house),
                                    x.beh.current
                                )
                            })
                            .collect();
                        println!("tick +{t}: the watched vehicle left the map; near it {near:?}");
                    }
                    firsts[1].get_or_insert(t);
                    firsts[2].get_or_insert(t);
                    // The deployed form: the owner's slave master building
                    // nearest the vehicle's last cell (S-0415 claim 10).
                    if *master == *id
                        && let Some(b) = w
                            .objects
                            .values()
                            .filter(|b| {
                                b.on_map
                                    && b.kind == Kind::Building
                                    && b.owner == Some(*house)
                                    && slaves::is_master(w, b.id)
                            })
                            .min_by_key(|b| {
                                let p = b.pos.cell();
                                (p.x - last.x).abs().max((p.y - last.y).abs())
                            })
                    {
                        *master = b.id;
                    }
                }
            }
            let out = w.objects.values().any(|s| {
                s.on_map && slaves::slave_of(w, s.id).is_some_and(|st| st.master == Some(*master))
            });
            if out {
                firsts[3].get_or_insert(t);
            }
        }
        if done % 1500 == 0 {
            for (who, _, id, start, master, _, firsts) in &watched {
                println!(
                    "tick +{done} {who}: {} | first dest/moved/deployed/slave out {firsts:?}",
                    describe(w, if w.obj(*id).is_some() { *id } else { *master }, *start)
                );
            }
        }
    }
    let credits: Vec<i32> = [local, computer]
        .iter()
        .map(|h| world(&c).houses[usize::from(h.0)].credits)
        .collect();
    println!("credits person/computer {credits:?}");
    for (who, _, id, _, master, last, firsts) in &watched {
        let at = world(&c).obj(*master).map_or(*last, |o| o.pos.cell());
        look_at(&mut c, at);
        let _ = c.frame();
        save(&mut c, &out, &format!("a3-41-{who}-far-slave-miner"));
        if *who == "person" {
            // It never gets a destination, sends no slave out and stays
            // put; the computer is its ally (lobby teams), so nothing of
            // the computer's goes for it (M8 step B9: with the alliance
            // lists written by hand, unsorted, the sim saw no alliance and
            // a computer vehicle hit it at +1557).
            let hit = first_hit.get(id).copied();
            println!("person's miner first hit {hit:?}");
            assert_eq!(firsts[0], None, "no destination");
            assert_eq!(firsts[3], None, "no slave out");
            assert_eq!(hit, None, "an ally's miner is never hit");
            assert_eq!(firsts[1], None, "the person's miner stays put: {firsts:?}");
        }
    }
    // spec: S-0309 claim 18 (an object of an allied house is not acquired); S-0412 claim 8 (lobby teams ally)
    println!("ticks where a player house's object targeted an ally's: {allied_targets}");
    assert_eq!(allied_targets, 0, "no object targets an ally's");
}

#[test]
#[ignore = "local lane: needs GLASNOST_GAME_DIR"]
fn harvest_notes_on_the_real_data() {
    let Some(dir) = std::env::var_os("GLASNOST_GAME_DIR") else {
        eprintln!("GLASNOST_GAME_DIR not set: skipped");
        return;
    };
    let out = png::lane_out("m8/a3-36");
    std::fs::create_dir_all(&out).unwrap();
    // Every player house allied with every other by lobby teams
    // (GLASNOST_HARVEST_PEACE=0 keeps the war): the economy plays on
    // without the local house losing and ending the game.
    let peace = env_num("GLASNOST_HARVEST_PEACE", 1) != 0;
    let (mut c, mut clock, local) = start(&dir, peace);
    assert!(
        !peace || all_allied(world(&c)),
        "the lobby teams ally the players"
    );
    // Note 30: the resource within 2 cells of every mine is taken away, and
    // so is that of a 5 × 5 control patch of the field cell farthest from
    // any mine; what comes back near the mines and in the patch is counted.
    let control = {
        let w = &mut c.session.as_mut().unwrap().play.game.world;
        let r = w.rules_arc();
        let mine_at: Vec<CellXY> = w
            .objects
            .values()
            .filter(|o| o.kind == Kind::Terrain && o.on_map)
            .filter(|o| {
                r.terrain
                    .get(o.type_index as usize)
                    .is_some_and(|t| t.spawns_resources)
            })
            .map(|o| o.pos.cell())
            .collect();
        let far = |x: CellXY| {
            mine_at
                .iter()
                .map(|m| (m.x - x.x).abs().max((m.y - x.y).abs()))
                .min()
                .unwrap_or(i32::MAX)
        };
        let mut best: Option<(i32, CellXY)> = None;
        let all: Vec<CellXY> = w.cells.grid.iter().map(|(c, _)| c).collect();
        for cell in all {
            if resource::resource_at(w, cell).is_some() && best.is_none_or(|(d, _)| far(cell) > d) {
                best = Some((far(cell), cell));
            }
        }
        for m in &mine_at {
            for dy in -2..=2 {
                for dx in -2..=2 {
                    resource::take_all(w, CellXY::new(m.x + dx, m.y + dy));
                }
            }
        }
        best.map(|(_, at)| {
            let patch: Vec<CellXY> = (-2..=2)
                .flat_map(|dy| (-2..=2).map(move |dx| CellXY::new(at.x + dx, at.y + dy)))
                .collect();
            for &p in &patch {
                resource::take_all(w, p);
            }
            patch
        })
    };
    let mut bare = None;
    let mine_cells = mines(world(&c), "start", &mut bare);

    // Notes 23 and 24: a refinery and one miner of each kind for the local
    // house and for the other player house (the computer's), each by its
    // own base.
    let w = world(&c);
    let computer = computer_of(w, local);
    let mut kinds: BTreeMap<&'static str, u32> = BTreeMap::new();
    for ti in 0..w.rules().vehicles.len() as u32 {
        if let Some(k) = miner_kind(w, ti) {
            kinds.entry(k).or_insert(ti);
        }
    }
    println!(
        "miner kinds {:?}, computer house {computer:?}",
        kinds.keys()
    );
    let mut watched: Vec<(String, ObjectId, CellXY)> = Vec::new();
    // GLASNOST_HARVEST_PLACE=0 places nothing: only the houses' own miners.
    let place = env_num("GLASNOST_HARVEST_PLACE", 1) != 0;
    for (who, house) in [("local", Some(local)), ("computer", computer)] {
        let Some(house) = house else { continue };
        if !place {
            continue;
        }
        let Some(home) = home_of(world(&c), house) else {
            println!("{who}: no base");
            continue;
        };
        let refinery = place_refinery(&mut c, house, home, &kinds);
        println!("{who}: refinery placed {}", refinery.is_some());
        for (k, &ti) in &kinds {
            if let Some(id) = place_miner(&mut c, ti, house, CellXY::new(home.x + 3, home.y + 3)) {
                let cell = world(&c).obj(id).unwrap().pos.cell();
                watched.push((format!("{who} {k}"), id, cell));
            }
        }
        // A slave-miner building placed as a done item is (S-0406), when
        // the house's yard could make one.
        if let Some((id, cell)) = place_slave_building(&mut c, house, home) {
            watched.push((format!("{who} placed slave building"), id, cell));
        }
    }
    // Notes 23 and 24 away from ore: a slave miner of each player house
    // where its short scan finds no field (GLASNOST_HARVEST_FAR=0: none).
    if env_num("GLASNOST_HARVEST_FAR", 1) != 0
        && let Some(&ti) = kinds.get("slave miner")
    {
        for (who, house) in [("local", Some(local)), ("computer", computer)] {
            let Some(house) = house else { continue };
            let Some(home) = home_of(world(&c), house) else {
                continue;
            };
            if let Some(id) = place_far(&mut c, ti, house, home) {
                let cell = world(&c).obj(id).unwrap().pos.cell();
                watched.push((format!("{who} far slave miner"), id, cell));
            }
        }
        // The same far from ore, ordered to harvest the nearest tiberium
        // cell as a player's click does (S-0402 claim 22).
        if let Some(home) = home_of(world(&c), local)
            && let Some(id) = place_far(&mut c, ti, local, CellXY::new(home.x - 4, home.y - 4))
        {
            let cell = world(&c).obj(id).unwrap().pos.cell();
            let w = world(&c);
            let ore = (-150i32..=150)
                .flat_map(|dy| (-150i32..=150).map(move |dx| CellXY::new(cell.x + dx, cell.y + dy)))
                .filter(|&o| {
                    w.cells
                        .cell(o)
                        .is_some_and(|x| x.land == glasnost_map::terrain::land::TIBERIUM)
                })
                .min_by_key(|o| (o.x - cell.x).abs().max((o.y - cell.y).abs()));
            if let Some(ore) = ore {
                let n = c.session.as_mut().unwrap().play.issue(vec![
                    glasnost_sim::orders::Order::Behaviour {
                        actor: id,
                        behaviour: Behaviour::Harvest,
                        attack_move: false,
                        target: None,
                        dest: Some(glasnost_sim::world::Target::Cell(ore)),
                    },
                ]);
                println!(
                    "ordered far slave miner: ore at {}, orders taken {n}",
                    (ore.x - cell.x).abs().max((ore.y - cell.y).abs())
                );
                watched.push(("local ordered far slave miner".into(), id, cell));
            }
        }
    }
    // Note 23 through the factory: a slave miner built by the local house
    // and released by its own vehicle factory.
    if let Some(&ti) = kinds.get("slave miner")
        && let Some(home) = home_of(world(&c), local)
    {
        let t = glasnost_sim::econ::TypeRef {
            kind: Kind::Vehicle,
            index: ti,
        };
        let s = c.session.as_mut().unwrap();
        let w = &mut s.play.game.world;
        w.houses[usize::from(local.0)].credits += 20_000;
        let factories: Vec<u32> = (0..w.rules().buildings.len() as u32)
            .filter(|&i| {
                w.rules().buildings[i as usize].factory
                    == Some(glasnost_sim::rules::FactoryKind::Vehicle)
            })
            .collect();
        // The prerequisites: the strongest power plant the yard could make,
        // then vehicle factories until the slave miner is buildable; each
        // complete, on a cell its placement preview allows.
        let power = (0..w.rules().buildings.len() as u32)
            .filter(|&i| w.rules().buildings[i as usize].raw.econ.power.unwrap_or(0) > 0)
            .max_by_key(|&i| w.rules().buildings[i as usize].raw.econ.power.unwrap_or(0));
        let mut placed = false;
        for index in power.into_iter().chain(factories) {
            if glasnost_sim::econ::tech::buildable(w, local, t) {
                break;
            }
            let ft = glasnost_sim::econ::TypeRef {
                kind: Kind::Building,
                index,
            };
            let name = w.rules().buildings[index as usize].name.clone();
            'r: for r in 4i32..16 {
                for dy in -r..=r {
                    for dx in -r..=r {
                        if dx.abs() != r && dy.abs() != r {
                            continue;
                        }
                        let cell = CellXY::new(home.x + dx, home.y + dy);
                        let (foot, ok) =
                            glasnost_app::mapview::sidebar::preview(w, local, ft, cell);
                        if ok {
                            let tl = CellXY::new(
                                foot.iter().map(|f| f.x).min().unwrap(),
                                foot.iter().map(|f| f.y).min().unwrap(),
                            );
                            glasnost_sim::testkit::spawn(w, Kind::Building, &name, local.0, tl, 0);
                            placed = true;
                            break 'r;
                        }
                    }
                }
            }
        }
        ticks(&mut c, &mut clock, 300);
        let s = c.session.as_mut().unwrap();
        let w = s.play.world();
        println!(
            "factory slave miner: factory placed {placed}, answer {:?}, buildable {}",
            glasnost_sim::econ::tech::answer(w, local, t),
            glasnost_sim::econ::tech::buildable(w, local, t)
        );
        let n = s
            .play
            .issue(vec![glasnost_sim::orders::Order::Build { what: t }]);
        println!("factory slave miner: build orders taken {n}");
    }
    let watch = env_num("GLASNOST_UI_TICKS", 4500);
    let mut done = 0;
    while done < watch {
        ticks(&mut c, &mut clock, 300);
        done += 300;
        let w = world(&c);
        // A newly released local slave miner joins the watch.
        let new: Vec<(String, ObjectId, CellXY)> = w
            .objects
            .values()
            .filter(|o| {
                o.on_map
                    && o.owner == Some(local)
                    && o.kind == Kind::Vehicle
                    && miner_kind(w, o.type_index) == Some("slave miner")
                    && !watched.iter().any(|(_, id, _)| *id == o.id)
            })
            .map(|o| ("local factory slave miner".to_owned(), o.id, o.pos.cell()))
            .collect();
        watched.extend(new);
        for (name, id, start) in &watched {
            println!("tick +{done} {name}: {}", describe(w, *id, *start));
        }
        if done == 1800 {
            for (name, id, _) in &watched {
                if let Some(cell) = world(&c).obj(*id).map(|o| o.pos.cell()) {
                    look_at(&mut c, cell);
                    let _ = c.frame();
                    save(
                        &mut c,
                        &out,
                        &format!("a3-36-1800-{}", name.replace(' ', "-")),
                    );
                }
            }
        }
        let w = world(&c);
        if done % 1500 == 0 {
            let s = c.session.as_ref().unwrap();
            println!(
                "tick +{done}: world tick {}, screen game {}, failed {:?}, outcome {}",
                w.tick,
                c.shell.screen == Screen::Game,
                s.play.failed,
                s.play.outcome.is_some()
            );
            report(w, &format!("tick +{done}"));
            mines(w, &format!("tick +{done}"), &mut bare);
            if let Some(patch) = &control {
                let back = patch
                    .iter()
                    .filter(|&&p| resource::resource_at(w, p).is_some())
                    .count();
                println!("tick +{done} control patch cells holding resource again {back} of 25");
            }
            // Every player house's own miners, one by one.
            for o in w.objects.values().filter(|o| {
                o.on_map
                    && o.kind == Kind::Vehicle
                    && miner_kind(w, o.type_index).is_some()
                    && !watched.iter().any(|(_, id, _)| *id == o.id)
            }) {
                let h = o.owner.map_or(-1, |h| i32::from(h.0));
                let here = o.pos.cell();
                let cell = w.cells.cell(here);
                let contact = o.contact.and_then(|x| w.obj(x)).map(|x| {
                    let f = x.pos.cell();
                    (x.kind, here.x - f.x, here.y - f.y)
                });
                println!(
                    "tick +{done} house {h} miner {:?} {} | last {:?} now {} land {:?} resource {:?} contact {contact:?} ready {} expired {} remembered {:?}",
                    o.id,
                    describe(w, o.id, o.pos.cell()),
                    o.beh.last,
                    w.tick,
                    cell.map(|c| c.land),
                    resource::resource_at(w, here),
                    glasnost_sim::process::ready(w, o.id),
                    o.beh.expired,
                    o.econ
                        .harvest
                        .as_ref()
                        .and_then(|s| s.remembered)
                        .map(|r| (r.x - here.x, r.y - here.y)),
                );
            }
        }
    }
    for (name, id, _) in &watched {
        if let Some(cell) = world(&c).obj(*id).map(|o| o.pos.cell()) {
            look_at(&mut c, cell);
            let _ = c.frame();
            save(&mut c, &out, &format!("a3-36-{}", name.replace(' ', "-")));
        }
    }
    if let Some(&m) = mine_cells.first() {
        look_at(&mut c, m);
        let _ = c.frame();
        save(&mut c, &out, "a3-36-ore-mine");
    }
}
