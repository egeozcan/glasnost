//! Local lane for the M6b features (never in CI): with `GLASNOST_GAME_DIR`
//! set, every scene of [`SCENES`] starts its own skirmish through the
//! windowless client, puts the objects it needs on the map by their rules
//! flags (never by type name), plays the scene and writes its screens to
//! `local-lane/m6b/` (git-ignored). Prints counts, cells, ticks and
//! per-mille health only, never type names or rules values.
//!
//! Scenes now: a hover crossing water (S-0503), an own unit passing an own
//! gate (S-0215 claim 35, S-0108 claim 4), a teleporting unit's jump
//! (S-0207 claims 20 to 22, S-0208 claims 19 and 20), a damaged ship
//! repaired at its naval yard (S-0314 claims 16 to 24, S-0504 claim 14) and
//! a vehicle repaired at a repair depot (S-0314). A new scene is one more
//! function and one more row of [`SCENES`].
//!
//! Run with `cargo test --release -p glasnost-app --test m6b_lane --
//! --ignored --nocapture`. `GLASNOST_LANE_SCENE=a,b` plays only the scenes
//! with those keys; `GLASNOST_UI_MAP=n` picks the n-th offered map (default
//! 0; the hover and yard scenes need a map with open water).

#![cfg(not(target_arch = "wasm32"))]

use std::rc::Rc;

use glasnost_app::game::client::{Client, run_job};
use glasnost_app::mapview::png;
use glasnost_map::CellXY;
use glasnost_map::terrain::land;
use glasnost_sim::behaviour::Door;
use glasnost_sim::orders::Order;
use glasnost_sim::rules::{Behaviour, Kind, Locomotor, UnitRules};
use glasnost_sim::targeting::{self, ClickOn, Modifiers};
use glasnost_sim::world::{HouseId, Layer, ObjectId, World};
use glasnost_ui::geom::Rect;
use glasnost_ui::menu::MenuHit;
use glasnost_ui::setup::SetupHit;
use glasnost_ui::shell::Screen;
use glasnost_vfs::source::{FileSource, Unopened};
use glasnost_vfs::{ByteSource, MountOptions, PickedFile};

/// A scene: its key for `GLASNOST_LANE_SCENE` and the function that plays
/// it on a freshly started skirmish.
type Scene = (&'static str, fn(&mut Lane));

/// The scenes, in the order they are played.
const SCENES: &[Scene] = &[
    ("hover", hover_crossing_water),
    ("gate", gate_opening),
    ("teleport", unit_teleporting),
    ("yard", ship_repaired_at_yard),
    ("depot", vehicle_repaired_at_depot),
];

#[test]
#[ignore = "local lane: needs GLASNOST_GAME_DIR"]
fn m6b_scenes() {
    if std::env::var_os("GLASNOST_GAME_DIR").is_none() {
        eprintln!("GLASNOST_GAME_DIR not set: skipped");
        return;
    }
    let only: Vec<String> = std::env::var("GLASNOST_LANE_SCENE")
        .map(|v| v.split(',').map(|s| s.trim().to_string()).collect())
        .unwrap_or_default();
    for (key, play) in SCENES {
        if !only.is_empty() && !only.iter().any(|o| o == key) {
            continue;
        }
        println!("== scene {key}");
        let mut lane = Lane::start().expect("the install is set");
        play(&mut lane);
    }
}

// ---------------------------------------------------------------------------
// Scenes

/// S-0503: a hover-locomotor vehicle driven from the shore out across open
/// water, shot while it is moving over water cells away from the shore.
fn hover_crossing_water(lane: &mut Lane) {
    let w = lane.world();
    let hover = find_type(w, Kind::Vehicle, |_, u| u.locomotor == Locomotor::Hover);
    println!("hover type found: {}", hover.is_some());
    let Some(name) = hover else { return };
    let home = lane.home();
    let Some(sea) = lane.area(home, (1, 1), 2, |l| l == land::WATER) else {
        println!("no open water");
        return;
    };
    // The nearest standing place on land to that water, and a point across it.
    let shore = lane.open_land(sea);
    let (dx, dy) = (sea.x - shore.x, sea.y - shore.y);
    let far = CellXY::new(sea.x + dx.signum() * 8, sea.y + dy.signum() * 8);
    let dest = lane
        .area(far, (1, 1), 1, |l| l == land::WATER)
        .unwrap_or(sea);
    let local = lane.local();
    let id = lane.spawn(Kind::Vehicle, &name, local, shore);
    let issued = lane.order(local, id, ClickOn::Cell(dest));
    println!("hover from {shore:?} to {dest:?}, {issued} orders");
    let mut shot = false;
    for _ in 0..1500 {
        lane.ticks(1);
        let w = lane.world();
        let Some(o) = w.obj(id) else { break };
        let c = o.pos.cell();
        let on_water = w.cells.cell(c).is_some_and(|x| x.land == land::WATER);
        let from_shore = (c.x - shore.x).abs().max((c.y - shore.y).abs());
        if on_water && from_shore >= 3 && glasnost_sim::hover::moving_now(w, id) {
            let above = glasnost_sim::hover::hover_height(w, id) > 0;
            println!(
                "hover over water at {c:?}, {from_shore} cells out, above the surface: {above}"
            );
            lane.look_at(c);
            lane.ticks(1);
            lane.save("hover-crossing-water");
            shot = true;
            break;
        }
    }
    println!("hover seen moving over water: {shot}");
}

/// S-0215 claim 35, S-0108 claim 4: an own vehicle ordered straight through
/// an own gate, shot while the door is half open. The door frame of a
/// moving gate is S-0215 claim 35 d1's.
fn gate_opening(lane: &mut Lane) {
    let w = lane.world();
    let gate = w
        .rules()
        .buildings
        .iter()
        .find(|b| b.gate)
        .map(|b| (b.name.clone(), b.foundation));
    let unit = find_type(w, Kind::Vehicle, |_, u| plain_vehicle(u));
    println!(
        "gate type found: {}, vehicle type found: {}",
        gate.is_some(),
        unit.is_some()
    );
    let (Some((gn, (fw, fh))), Some(un)) = (gate, unit) else {
        return;
    };
    let home = lane.home();
    let Some(at) = lane.area(home, (fw, fh), 4, |l| l != land::WATER && l != land::ROCK) else {
        println!("no open land for the gate");
        return;
    };
    let local = lane.local();
    let g = lane.spawn(Kind::Building, &gn, local, at);
    // Across the gate's thin axis, through its middle cell.
    let (from, to) = if fw >= fh {
        let x = at.x + fw / 2;
        (CellXY::new(x, at.y - 4), CellXY::new(x, at.y + fh + 3))
    } else {
        let y = at.y + fh / 2;
        (CellXY::new(at.x - 4, y), CellXY::new(at.x + fw + 3, y))
    };
    let v = lane.spawn(Kind::Vehicle, &un, local, from);
    lane.ticks(2);
    let issued = lane.order(local, v, ClickOn::Cell(to));
    println!("gate at {at:?} foundation {fw}x{fh}, vehicle {from:?} to {to:?}, {issued} orders");
    let cells: Vec<CellXY> = (0..fw)
        .flat_map(|dx| (0..fh).map(move |dy| CellXY::new(at.x + dx, at.y + dy)))
        .collect();
    let (mut shot, mut inside, mut opened) = (false, false, None);
    for _ in 0..1500 {
        lane.ticks(1);
        let w = lane.world();
        let Some(o) = w.obj(v) else { break };
        inside |= cells.contains(&o.pos.cell());
        let t = w.tick;
        if let Some(Door::Opening { since, ticks }) = w.obj(g).map(|x| x.beh.gate.door) {
            opened.get_or_insert(since);
            if !shot && t - since >= ticks / 2 {
                println!(
                    "door opening since tick {since} over {ticks} ticks; vehicle at {:?}",
                    o.pos.cell()
                );
                lane.look_at(at);
                lane.ticks(1);
                lane.save("gate-opening");
                shot = true;
            }
        }
        if shot && inside {
            break;
        }
    }
    let end = lane.world().obj(v).map(|o| o.pos.cell());
    println!(
        "door began opening at tick {opened:?}; shot: {shot}; vehicle stood in the gate: {inside}; vehicle now at {end:?}"
    );
}

/// S-0207 claims 20 to 22, S-0208 claims 19 and 20: a teleporting unit
/// ordered a few cells away; shot just after its jump (the warp animations
/// at the old point and the landing point) and again after it has phased in.
fn unit_teleporting(lane: &mut Lane) {
    let w = lane.world();
    // An explicit teleporting locomotor: a type with none defaults to it (S-0207 claim 14).
    let real = |u: &UnitRules| u.raw.locomotor.is_some() && u.locomotor == Locomotor::Teleport;
    let unit = find_type(w, Kind::Vehicle, |_, u| real(u) && !u.harvester)
        .map(|n| (Kind::Vehicle, n))
        .or_else(|| find_type(w, Kind::Infantry, |_, u| real(u)).map(|n| (Kind::Infantry, n)));
    println!(
        "teleporting type found: {}",
        unit.as_ref()
            .map(|(k, _)| format!("{k:?}"))
            .unwrap_or("none".into())
    );
    let Some((kind, name)) = unit else { return };
    let home = lane.home();
    let Some(from) = lane.area(CellXY::new(home.x + 6, home.y + 6), (1, 1), 2, |l| {
        l != land::WATER && l != land::ROCK
    }) else {
        return;
    };
    let Some(to) = lane.area(CellXY::new(from.x + 6, from.y - 3), (1, 1), 1, |l| {
        l != land::WATER && l != land::ROCK
    }) else {
        return;
    };
    let local = lane.local();
    let id = lane.spawn(kind, &name, local, from);
    lane.ticks(2);
    println!(
        "teleporter: {}",
        glasnost_sim::teleport::is_teleporter(lane.world(), id)
    );
    let issued = lane.order(local, id, ClickOn::Cell(to));
    println!("teleport from {from:?} to {to:?}, {issued} orders");
    let mut last = lane.world().obj(id).map(|o| o.pos.cell());
    let mut jumped = None;
    for _ in 0..600 {
        lane.ticks(1);
        let w = lane.world();
        let now = w.obj(id).map(|o| o.pos.cell());
        if let (Some(a), Some(b)) = (last, now)
            && (a.x - b.x).abs().max((a.y - b.y).abs()) > 1
        {
            jumped = Some((w.tick, a, b));
            break;
        }
        last = now;
    }
    println!("jump (tick, from, to): {jumped:?}");
    let Some((_, a, b)) = jumped else { return };
    lane.look_at(CellXY::new((a.x + b.x) / 2, (a.y + b.y) / 2));
    lane.ticks(1);
    let phasing = glasnost_sim::teleport::phasing_in(lane.world(), id);
    println!("phasing in after the jump: {phasing}");
    lane.save("unit-teleporting");
    // The warp animations a few frames on.
    lane.ticks(5);
    lane.save("unit-teleporting-2");
    for _ in 0..600 {
        lane.ticks(1);
        if !glasnost_sim::teleport::phasing_in(lane.world(), id) {
            break;
        }
    }
    lane.look_at(b);
    lane.ticks(1);
    lane.save("unit-teleported");
}

/// S-0314 claims 16 to 24 and S-0504 claim 14: a damaged ship clicked onto
/// an own naval yard with the repair flag docks and is repaired.
fn ship_repaired_at_yard(lane: &mut Lane) {
    let w = lane.world();
    let yard = w
        .rules()
        .buildings
        .iter()
        .find(|b| b.naval && b.factory.is_some() && b.building.unit_repair)
        .map(|b| (b.name.clone(), b.foundation));
    let ship = find_type(w, Kind::Vehicle, |_, u| {
        u.locomotor == Locomotor::Ship && u.combat.naval && !u.combat.underwater
    });
    println!(
        "repairing yard type found: {}, ship type found: {}",
        yard.is_some(),
        ship.is_some()
    );
    let (Some((yn, (fw, fh))), Some(sn)) = (yard, ship) else {
        return;
    };
    let home = lane.home();
    let water = |l| l == land::WATER;
    let Some(at) = lane
        .area(home, (fw, fh), 1, water)
        .or_else(|| lane.area(home, (fw, fh), 0, water))
    else {
        println!("no open water for the yard");
        return;
    };
    let local = lane.local();
    let k = lane.spawn(Kind::Building, &yn, local, at);
    let Some(sea) = lane.area(CellXY::new(at.x - 5, at.y + 1), (1, 1), 1, water) else {
        return;
    };
    let s = lane.spawn(Kind::Vehicle, &sn, local, sea);
    println!("yard at {at:?} foundation {fw}x{fh}, ship at {sea:?}");
    repair_scene(lane, k, s, at, "ship-repaired-at-yard");
}

/// S-0314 claims 6 and 16 to 24: a damaged vehicle clicked onto an own
/// repair depot docks and is repaired.
fn vehicle_repaired_at_depot(lane: &mut Lane) {
    let w = lane.world();
    let depot = w
        .rules()
        .buildings
        .iter()
        .find(|b| b.building.unit_repair && !b.naval && b.factory.is_none())
        .map(|b| (b.name.clone(), b.foundation));
    let unit = find_type(w, Kind::Vehicle, |_, u| plain_vehicle(u));
    println!(
        "depot type found: {}, vehicle type found: {}",
        depot.is_some(),
        unit.is_some()
    );
    let (Some((dn, (fw, fh))), Some(un)) = (depot, unit) else {
        return;
    };
    let home = lane.home();
    let Some(at) = lane.area(home, (fw, fh), 3, |l| l != land::WATER && l != land::ROCK) else {
        println!("no open land for the depot");
        return;
    };
    let local = lane.local();
    let k = lane.spawn(Kind::Building, &dn, local, at);
    let Some(from) = lane.area(CellXY::new(at.x + fw + 5, at.y), (1, 1), 1, |l| {
        l != land::WATER && l != land::ROCK
    }) else {
        return;
    };
    let v = lane.spawn(Kind::Vehicle, &un, local, from);
    println!("depot at {at:?} foundation {fw}x{fh}, vehicle at {from:?}");
    repair_scene(lane, k, v, at, "vehicle-repaired-at-depot");
}

/// The shared repair scene: `unit` is damaged to a quarter, clicked onto
/// `depot`, and run until it is docked and repaired; shot once it has been
/// docked and repaired for a while. Prints health in per-mille of the type's
/// strength with the ticks.
fn repair_scene(lane: &mut Lane, depot: ObjectId, unit: ObjectId, at: CellXY, shot: &str) {
    let local = lane.local();
    lane.world_mut().houses[usize::from(local.0)].credits += 10_000;
    let full = lane.strength(unit);
    lane.world_mut().obj_mut(unit).unwrap().health = full / 4;
    lane.ticks(2);
    let orders = targeting::click(
        lane.world(),
        local,
        &[unit],
        ClickOn::Object(depot),
        Modifiers::default(),
    )
    .orders;
    let enter = orders.iter().any(|o| {
        matches!(
            o,
            Order::Behaviour {
                behaviour: Behaviour::Enter,
                ..
            }
        )
    });
    let mut issued = 0;
    for o in orders {
        issued += usize::from(glasnost_sim::orders::issue(lane.world_mut(), local, o));
    }
    let t0 = lane.world().tick;
    println!(
        "click onto the depot: enter order {enter}, {issued} issued at tick {t0}; health {} per mille",
        lane.permille(unit)
    );
    let mut docked = None;
    let mut first_rise = None;
    let mut saved = false;
    let mut start = 0;
    for _ in 0..6000 {
        lane.ticks(1);
        let w = lane.world();
        let Some(o) = w.obj(unit) else { break };
        let t = w.tick;
        let tethered = glasnost_sim::depot::tethered(w, unit);
        if docked.is_none() && (t - t0) % 250 == 0 {
            println!(
                "  tick {t}: cell {:?}, behaviour {:?}, depot link {:?}",
                o.pos.cell(),
                o.beh.current,
                glasnost_sim::depot::link(w, unit).map(|l| l.tethered)
            );
        }
        if tethered && docked.is_none() {
            docked = Some(t);
            start = lane.permille(unit);
            println!(
                "docked at tick {t} (cell {:?}); health {start} per mille",
                o.pos.cell()
            );
        }
        let pm = lane.permille(unit);
        if docked.is_some() && pm > start && first_rise.is_none() {
            first_rise = Some(t);
            println!("first repair step at tick {t}; health {pm} per mille");
        }
        if !saved && first_rise.is_some() && pm >= (start + 1000) / 2 {
            lane.look_at(at);
            lane.ticks(1);
            lane.save(shot);
            saved = true;
        }
        if pm >= 1000 || (docked.is_some() && !tethered) {
            break;
        }
    }
    let w = lane.world();
    println!(
        "after tick {}: health {} per mille, still docked: {}; docked at {docked:?}, first repair step at {first_rise:?}",
        w.tick,
        lane.permille(unit),
        glasnost_sim::depot::tethered(w, unit)
    );
    if !saved {
        lane.look_at(at);
        lane.ticks(1);
        lane.save(shot);
    }
}

// ---------------------------------------------------------------------------
// The lane

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
    use glasnost_app::game::play::Button;
    use glasnost_ui::geom::Point;
    let p = Point::new(r.x + r.w / 2, r.y + r.h / 2);
    let _ = c.frame();
    c.pointer_moved(p);
    c.pointer_down(Button::Left, p);
    c.pointer_up(Button::Left, p, Modifiers::default());
}

struct Lane {
    c: Client,
    clock: i64,
    dir: std::path::PathBuf,
}

impl Lane {
    /// Starts a skirmish from the menu, the map revealed to the local house.
    fn start() -> Option<Lane> {
        let dir = std::env::var_os("GLASNOST_GAME_DIR")?;
        let out = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../local-lane/m6b");
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
            let _ = c.frame();
            let r = c.setup_targets.rect_of(&SetupHit::Map(1)).unwrap();
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
        let mut lane = Lane {
            c,
            clock: 0,
            dir: out,
        };
        let s = lane.c.session.as_mut().unwrap();
        let local = s.play.local;
        glasnost_sim::econ::shroud::defeat_reveal(&mut s.play.game.world, local);
        lane.ticks(5);
        Some(lane)
    }

    fn ticks(&mut self, n: usize) {
        for _ in 0..n {
            self.clock += 1000;
            self.c.update(self.clock);
            if self.c.shell.screen != Screen::Game {
                break;
            }
        }
    }

    fn world(&self) -> &World {
        self.c.session.as_ref().unwrap().play.world()
    }

    fn world_mut(&mut self) -> &mut World {
        &mut self.c.session.as_mut().unwrap().play.game.world
    }

    fn local(&self) -> HouseId {
        self.c.session.as_ref().unwrap().play.local
    }

    /// The cell of the local house's first object on the map (its start).
    fn home(&self) -> CellXY {
        let w = self.world();
        let local = self.local();
        w.objects
            .values()
            .find(|o| o.on_map && o.owner == Some(local))
            .map(|o| o.pos.cell())
            .expect("a local object")
    }

    /// Centres the view on `cell`.
    fn look_at(&mut self, cell: CellXY) {
        let s = self.c.session.as_mut().unwrap();
        let (x, y) = glasnost_render::projection::cell_centre(cell.x, cell.y);
        s.set_centre(glasnost_render::projection::project_lifted(x, y, 0));
    }

    /// The live view's animation counts: created, with art, without a
    /// sprite, drawn in the last frame.
    fn counts(&self) -> (u64, u64, u64, usize) {
        let s = self.c.session.as_ref().unwrap();
        let k = &s.loaded.live.as_ref().expect("a live game").counts;
        (
            k.anims,
            k.anims_with_art,
            k.anims_without_sprite,
            k.anims_drawn,
        )
    }

    fn save(&mut self, name: &str) {
        let rgba = self.c.frame_rgba();
        let (w, h) = self.c.size;
        std::fs::write(
            self.dir.join(format!("{name}.png")),
            png::png(w as u32, h as u32, &rgba),
        )
        .unwrap();
        let (made, art, none, drawn) = self.counts();
        println!(
            "wrote local-lane/m6b/{name}.png (tick {}): animations created {made}, \
             with art {art}, without sprite {none}, drawn now {drawn}",
            self.world().tick
        );
    }

    fn spawn(&mut self, kind: Kind, name: &str, house: HouseId, at: CellXY) -> ObjectId {
        glasnost_sim::testkit::spawn(self.world_mut(), kind, name, house.0, at, 0)
    }

    /// Orders `actor` of `house` by a click on `target`, as a person would.
    fn order(&mut self, house: HouseId, actor: ObjectId, target: ClickOn) -> usize {
        let w = self.world();
        let r = targeting::click(w, house, &[actor], target, Modifiers::default());
        let mut n = 0;
        for o in r.orders {
            n += usize::from(glasnost_sim::orders::issue(self.world_mut(), house, o));
        }
        n
    }

    /// The strength of `id`'s type.
    fn strength(&self, id: ObjectId) -> i32 {
        let w = self.world();
        w.obj(id)
            .and_then(|o| w.rules().unit(o.kind, o.type_index))
            .map_or(1, |u| u.strength.max(1))
    }

    /// `id`'s health in per-mille of its type's strength.
    fn permille(&self, id: ObjectId) -> i32 {
        let h = self.world().obj(id).map_or(0, |o| o.health);
        (i64::from(h) * 1000 / i64::from(self.strength(id))) as i32
    }

    /// The nearest cell to `near` whose 5 × 5 neighbourhood is legal land
    /// that is not water, so that units can stand there.
    fn open_land(&self, near: CellXY) -> CellXY {
        self.area(near, (1, 1), 2, |l| l != land::WATER && l != land::ROCK)
            .unwrap_or(near)
    }

    /// The nearest top-left cell to `near` of a `size` area that, with a
    /// `margin` of cells around it, is usable, of land types `ok` and free of
    /// ground objects.
    fn area(
        &self,
        near: CellXY,
        size: (i32, i32),
        margin: i32,
        ok: impl Fn(u8) -> bool,
    ) -> Option<CellXY> {
        let w = self.world();
        let good = |c: CellXY| {
            (-margin..size.0 + margin).all(|dx| {
                (-margin..size.1 + margin).all(|dy| {
                    let k = CellXY::new(c.x + dx, c.y + dy);
                    w.cells.usable(k)
                        && w.cells.cell(k).is_some_and(|x| ok(x.land))
                        && w.cells.list(k, Layer::Ground).is_empty()
                })
            })
        };
        w.cells
            .grid
            .iter()
            .map(|(k, _)| k)
            .filter(|&k| good(k))
            .min_by_key(|k| {
                let (dx, dy) = (i64::from(k.x - near.x), i64::from(k.y - near.y));
                (dx * dx + dy * dy, k.y, k.x)
            })
    }
}

/// An armed driving vehicle that is not a harvester.
fn plain_vehicle(u: &UnitRules) -> bool {
    u.locomotor == Locomotor::Drive
        && !u.harvester
        && u.combat.slots[0].is_some()
        && !u.combat.naval
}

/// The name of the first type of `kind` matching `f`.
fn find_type(w: &World, kind: Kind, f: impl Fn(u32, &UnitRules) -> bool) -> Option<String> {
    let r = w.rules();
    let list = match kind {
        Kind::Infantry => &r.infantry,
        Kind::Vehicle => &r.vehicles,
        _ => &r.aircraft,
    };
    list.iter()
        .enumerate()
        .find(|(i, u)| f(*i as u32, u))
        .map(|(_, u)| u.name.clone())
}
