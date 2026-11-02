//! Local lane for the M6a units (never in CI): with `GLASNOST_GAME_DIR`
//! set, starts a skirmish through the windowless client, puts the new
//! kinds of unit of the install's rules on the map by their flags (a
//! vehicle whose spawner weapon launches a rocket type, S-0506; a unit
//! with a latching weapon and a vehicle it can latch onto, S-0614; an
//! armed aircraft and a pad, S-0501; ships on water, S-0504), plays each
//! scene and writes the screens to `local-lane/m6a/` (git-ignored). Prints
//! counts, cells and ticks only, never type names.
//!
//! Run with `cargo test --release -p glasnost-app --test m6a_lane --
//! --ignored --nocapture`. `GLASNOST_UI_MAP=n` picks the n-th offered map
//! (default 0); the ship scene needs a map with open water.

#![cfg(not(target_arch = "wasm32"))]

use std::rc::Rc;

use glasnost_app::game::client::{Client, run_job};
use glasnost_app::mapview::png;
use glasnost_map::CellXY;
use glasnost_map::terrain::land;
use glasnost_sim::rocket::Phase;
use glasnost_sim::rules::{Kind, Locomotor};
use glasnost_sim::targeting::{self, ClickOn, Modifiers};
use glasnost_sim::world::{HouseId, HouseRole, ObjectId, World};
use glasnost_ui::geom::Rect;
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
    /// Starts a skirmish from the menu.
    fn start() -> Option<Lane> {
        let dir = std::env::var_os("GLASNOST_GAME_DIR")?;
        let out = png::lane_out("m6a");
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
        let local = s.play.me();
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
        self.c.session.as_ref().unwrap().play.me()
    }

    /// Centres the view on `cell`.
    fn look_at(&mut self, cell: CellXY) {
        let s = self.c.session.as_mut().unwrap();
        let (x, y) = glasnost_render::projection::cell_centre(cell.x, cell.y);
        s.set_centre(glasnost_render::projection::project_lifted(x, y, 0));
    }

    /// Centres the view on a position's lifted projection: a missile high in the air is drawn
    /// far up the screen from its cell (S-0101 F1 and F2).
    fn look_at_pos(&mut self, x: i32, y: i32, z: i32) {
        let s = self.c.session.as_mut().unwrap();
        s.set_centre(glasnost_render::projection::project_lifted(x, y, z));
    }

    fn save(&mut self, name: &str) {
        let rgba = self.c.frame_rgba();
        let (w, h) = self.c.size;
        std::fs::write(
            self.dir.join(format!("{name}.png")),
            png::png(w as u32, h as u32, &rgba),
        )
        .unwrap();
        println!(
            "wrote {}/{name}.png (tick {})",
            self.dir.display(),
            self.world().tick
        );
        // GLASNOST_TRACE="x,y;x,y": the pixel trace of the saved shot's screen points.
        // GLASNOST_BLANK="idx,idx": a second shot, "<name>-blank", without those scene items' images.
        if let Ok(b) = std::env::var("GLASNOST_BLANK") {
            let s = self.c.session.as_mut().unwrap();
            let mut saved = Vec::new();
            for i in b.split(',').filter_map(|v| v.trim().parse::<usize>().ok()) {
                let Some(it) = s.scene.items.get(i).copied() else {
                    continue;
                };
                let img = &s.scene.images[it.image as usize];
                let blank = glasnost_fmt::image::IndexedImage::new(
                    img.width(),
                    img.height(),
                    vec![0; (img.width() * img.height()) as usize],
                )
                .unwrap();
                saved.push((
                    it.image as usize,
                    std::mem::replace(&mut s.scene.images[it.image as usize], blank),
                ));
            }
            if !saved.is_empty() {
                let rgba = self.c.frame_rgba();
                let (w, h) = self.c.size;
                std::fs::write(
                    self.dir.join(format!("{name}-blank.png")),
                    png::png(w as u32, h as u32, &rgba),
                )
                .unwrap();
                let s = self.c.session.as_mut().unwrap();
                for (i, img) in saved {
                    s.scene.images[i] = img;
                }
            }
        }
        // GLASNOST_TRACE_BOX="x0,y0,x1,y1,step": the distinct items over a grid of screen points.
        if let Ok(b) = std::env::var("GLASNOST_TRACE_BOX") {
            let n: Vec<i32> = b.split(',').filter_map(|v| v.trim().parse().ok()).collect();
            if let [x0, y0, x1, y1, step] = n[..] {
                let mut seen = std::collections::BTreeSet::new();
                for x in (x0..x1).step_by(step.max(1) as usize) {
                    for y in (y0..y1).step_by(step.max(1) as usize) {
                        let s = self.c.session.as_mut().unwrap();
                        seen.extend(s.trace_at(glasnost_ui::geom::Point::new(x, y)));
                    }
                }
                println!("trace box {name}: {} distinct items", seen.len());
                for l in seen {
                    println!("  {l}");
                }
            }
        }
        if let Ok(points) = std::env::var("GLASNOST_TRACE") {
            for p in points.split(';') {
                let Some((x, y)) = p.split_once(',') else {
                    continue;
                };
                let (Ok(x), Ok(y)) = (x.trim().parse::<i32>(), y.trim().parse::<i32>()) else {
                    continue;
                };
                let lines = self
                    .c
                    .session
                    .as_mut()
                    .unwrap()
                    .trace_at(glasnost_ui::geom::Point::new(x, y));
                println!("trace {name} at ({x}, {y}): {} items", lines.len());
                for l in lines {
                    println!("  {l}");
                }
            }
        }
    }

    fn spawn(&mut self, kind: Kind, name: &str, house: HouseId, at: CellXY) -> ObjectId {
        glasnost_sim::testkit::spawn(self.world_mut(), kind, name, house.0, at, 0)
    }

    /// Orders `actor` of `house` by a click on `target`, as a person would.
    fn attack(&mut self, house: HouseId, actor: ObjectId, target: ClickOn) -> usize {
        let w = self.world();
        let r = targeting::click(w, house, &[actor], target, Modifiers::default());
        let mut n = 0;
        for o in r.orders {
            n += usize::from(glasnost_sim::orders::issue(self.world_mut(), house, o));
        }
        n
    }

    /// The first player house that is not the local one.
    fn enemy(&self) -> HouseId {
        let w = self.world();
        let local = self.local();
        (0..w.houses.len())
            .map(|i| HouseId(i as u8))
            .find(|&h| {
                h != local
                    && w.houses[usize::from(h.0)].role == HouseRole::Player
                    && !w.allied(Some(local), Some(h))
            })
            .expect("an enemy house")
    }

    /// The cell of an enemy object on the map.
    fn enemy_cell(&self) -> CellXY {
        let w = self.world();
        let e = self.enemy();
        w.objects
            .values()
            .find(|o| o.on_map && o.owner == Some(e))
            .map(|o| o.pos.cell())
            .expect("an enemy object")
    }

    /// The nearest cell to `near` whose 5 × 5 neighbourhood is legal land
    /// that is not water, so that units can stand there.
    fn open_land(&self, near: CellXY) -> CellXY {
        self.open(near, |l| l != land::WATER && l != land::ROCK)
    }

    fn open(&self, near: CellXY, ok: impl Fn(u8) -> bool) -> CellXY {
        let w = self.world();
        let good = |c: CellXY| {
            (-2..=2).all(|dx| {
                (-2..=2).all(|dy| {
                    let k = CellXY::new(c.x + dx, c.y + dy);
                    w.cells.usable(k) && w.cells.cell(k).is_some_and(|x| ok(x.land))
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
            .unwrap_or(near)
    }
}

/// The name of the first type of `kind` matching `f`.
fn find_type(
    w: &World,
    kind: Kind,
    f: impl Fn(u32, &glasnost_sim::rules::UnitRules) -> bool,
) -> Option<String> {
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

#[test]
#[ignore = "local lane: needs GLASNOST_GAME_DIR"]
fn m6a_scenes() {
    let Some(mut lane) = Lane::start() else {
        eprintln!("GLASNOST_GAME_DIR not set: skipped");
        return;
    };
    let local = lane.local();
    let enemy = lane.enemy();
    let target = lane.enemy_cell();
    println!("enemy object at cell {target:?}");

    // S-0506: a vehicle whose spawner launches a rocket type.
    let w = lane.world();
    let v3 = find_type(w, Kind::Vehicle, |i, _| {
        w.rules()
            .missiles
            .spawners
            .get(&(Kind::Vehicle, i))
            .is_some_and(|s| w.rules().missiles.set_of(s.spawned).is_some())
    });
    println!("rocket launcher type found: {}", v3.is_some());
    if let Some(name) = v3 {
        let at = lane.open_land(CellXY::new(target.x - 14, target.y));
        let id = lane.spawn(Kind::Vehicle, &name, local, at);
        let e = lane.world().obj_at_cell_owner(target, enemy);
        let issued = e.map_or(0, |e| lane.attack(local, id, ClickOn::Object(e)));
        println!("launcher at {at:?}, {issued} orders");
        // The rocket is in flight once it has left its pause, tilt or raise.
        let flight = |w: &World| {
            w.objects
                .values()
                .find(|o| {
                    o.rocket.as_ref().is_some_and(|r| {
                        matches!(
                            r.phase,
                            Some(Phase::Ascent | Phase::Cruise | Phase::Descent)
                        )
                    })
                })
                .map(|o| (o.pos.cell(), o.pos.x, o.pos.y, o.pos.z))
        };
        let mut seen = 0;
        for _ in 0..2000 {
            lane.ticks(1);
            if let Some((c, x, y, z)) = flight(lane.world()) {
                seen += 1;
                println!("rocket in flight at {c:?}, height {z}");
                lane.look_at_pos(x, y, z);
                let name = if seen == 1 {
                    "v3-rocket-in-flight".to_string()
                } else {
                    format!("v3-rocket-in-flight-{seen}")
                };
                lane.save(&name);
                if seen == 3 {
                    break;
                }
                lane.ticks(30);
            }
        }
        println!("rocket seen in flight: {seen}");
        // Whether the rocket type's data asks for no shadow (S-0105 claim 29), without its name.
        if let Some(o) = lane.world().objects.values().find(|o| o.rocket.is_some()) {
            let ty = o.type_index as usize;
            let s = lane.c.session.as_ref().unwrap();
            let keys = s
                .loaded
                .import
                .outcome
                .registries
                .get(&glasnost_convert::kinds::Kind::Aircraft)
                .and_then(|r| r.get(ty))
                .map(|e| &e.keys);
            let val = |k: &[u8]| {
                keys.and_then(|m| m.get(k))
                    .and_then(|v| v.value.as_deref())
                    .map(|v| String::from_utf8_lossy(v).into_owned())
            };
            println!(
                "rocket type: no-shadow key {:?}, shadow-index key {:?}",
                val(b"NoShadow"),
                val(b"ShadowIndex")
            );
        }
        // The later scenes share this world: take the launcher and its missiles out, so that no
        // missile flies (lifted high on the screen) through the parasite, aircraft and ship shots.
        let missiles: Vec<ObjectId> = lane
            .world()
            .objects
            .values()
            .filter(|o| o.rocket.is_some())
            .map(|o| o.id)
            .collect();
        for m in missiles {
            glasnost_sim::objects::remove(lane.world_mut(), m);
        }
        glasnost_sim::objects::remove(lane.world_mut(), id);
        println!(
            "missiles left: {}",
            lane.world()
                .objects
                .values()
                .filter(|o| o.rocket.is_some())
                .count()
        );
    }

    // S-0614: a unit with a latching weapon and an enemy vehicle it can latch onto.
    let w = lane.world();
    let latches = |u: &glasnost_sim::rules::UnitRules| {
        u.combat
            .slots
            .iter()
            .flatten()
            .any(|&wp| glasnost_sim::parasite::latching(w, wp))
    };
    let drone = find_type(w, Kind::Vehicle, |_, u| latches(u))
        .map(|n| (Kind::Vehicle, n))
        .or_else(|| find_type(w, Kind::Infantry, |_, u| latches(u)).map(|n| (Kind::Infantry, n)));
    let host = find_type(w, Kind::Vehicle, |_, u| {
        u.vehicle.parasiteable
            && !u.combat.organic
            && u.locomotor == Locomotor::Drive
            && !latches(u)
    });
    println!(
        "parasite type found: {}, host type found: {}",
        drone.is_some(),
        host.is_some()
    );
    if let (Some((dk, dn)), Some(hn)) = (drone, host) {
        let at = lane.open_land(CellXY::new(target.x - 8, target.y + 8));
        let h = lane.spawn(Kind::Vehicle, &hn, enemy, CellXY::new(at.x + 2, at.y));
        let d = lane.spawn(dk, &dn, local, CellXY::new(at.x - 2, at.y));
        let issued = lane.attack(local, d, ClickOn::Object(h));
        println!("parasite at {at:?}, {issued} orders");
        let mut latched = false;
        for _ in 0..600 {
            lane.ticks(1);
            if glasnost_sim::parasite::host_of(lane.world(), d).is_some() {
                latched = true;
                break;
            }
        }
        println!("latched: {latched}");
        if latched {
            lane.ticks(20);
            if let Some(c) = lane.world().obj(h).map(|o| o.pos.cell()) {
                lane.look_at(c);
                lane.ticks(1);
                lane.save("drone-inside-vehicle");
            }
        }
    }

    // S-0501: an armed aircraft and a pad of the local house.
    let w = lane.world();
    let r = w.rules();
    let spawned: Vec<u32> = r.missiles.spawners.values().map(|s| s.spawned).collect();
    let plane = find_type(w, Kind::Aircraft, |i, u| {
        u.locomotor == Locomotor::Fly
            && u.combat.slots[0].is_some()
            && r.missiles.set_of(i).is_none()
            && !spawned.contains(&i)
    });
    let pad = r
        .buildings
        .iter()
        .find(|b| b.building.helipad)
        .map(|b| b.name.clone());
    println!(
        "aircraft type found: {}, pad type found: {}",
        plane.is_some(),
        pad.is_some()
    );
    if let (Some(pn), Some(bn)) = (plane, pad) {
        let at = lane.open_land(CellXY::new(target.x - 20, target.y - 6));
        let k = lane.spawn(Kind::Building, &bn, local, at);
        let a = lane.spawn(Kind::Aircraft, &pn, local, CellXY::new(at.x + 3, at.y + 3));
        lane.ticks(2);
        lane.look_at(at);
        lane.ticks(1);
        lane.save("aircraft-and-pad");
        let e = lane.world().obj_at_cell_owner(target, enemy);
        let issued = e.map_or(0, |e| lane.attack(local, a, ClickOn::Object(e)));
        println!("pad at {at:?}, {issued} orders");
        let mut fired = false;
        let mut home = false;
        for _ in 0..3000 {
            lane.ticks(1);
            let w = lane.world();
            let Some(o) = w.obj(a) else { break };
            let shooting = w.projectiles.values().any(|p| p.attacker == Some(a));
            if !fired && shooting {
                fired = true;
                let c = o.pos.cell();
                lane.look_at(c);
                lane.ticks(1);
                lane.save("aircraft-attacking");
                continue;
            }
            if fired
                && glasnost_sim::air::on_ground(w, a)
                && w.obj(a).is_some_and(|o| o.contact == Some(k))
            {
                home = true;
                lane.look_at(at);
                lane.ticks(1);
                lane.save("aircraft-on-pad");
                break;
            }
        }
        println!("aircraft fired: {fired}, landed on its pad again: {home}");
    }

    // The building types named by GLASNOST_RADAR_TYPE (type names given on the command line,
    // comma-separated, so that no stock name is kept in the source): one shot each, "yuri-radar"
    // for a single name, else "building-<i>". The lead check of the Yuri radar shot.
    // GLASNOST_RADAR_TYPE=radar: every building type with the radar flag
    // (S-0407 claim 6), found in the data, so that no stock name is needed; each shot is
    // "radar-side<s>-<i>" with the type's base-planning side.
    let radar_names: Option<String> = std::env::var("GLASNOST_RADAR_TYPE").ok().map(|n| {
        if n != "radar" {
            return n;
        }
        let r = lane.world().rules();
        let names: Vec<String> = r
            .econ
            .tech
            .buildings
            .iter()
            .zip(&r.buildings)
            .filter(|(t, _)| t.radar)
            .map(|(t, b)| format!("{}:{}", b.name, t.side))
            .collect();
        println!("building types with the radar flag: {names:?}");
        names.join(",")
    });
    if let Some(names) = radar_names {
        let list: Vec<&str> = names.split(',').collect();
        for (i, entry) in list.iter().enumerate() {
            // "name:side" from the radar-flag search: the base-planning side goes into the
            // shot name (S-0407 claim 6), so the pictures can be told apart without stock names.
            let (name, side) = entry.split_once(':').unwrap_or((entry, ""));
            let at = lane.open_land(CellXY::new(target.x - 20, target.y + 12));
            let id = lane.spawn(Kind::Building, name, local, at);
            lane.ticks(2);
            lane.look_at(at);
            lane.ticks(1);
            let shot = if list.len() == 1 {
                "yuri-radar".to_string()
            } else if side.is_empty() {
                format!("building-{i}")
            } else {
                format!("radar-side{side}-{i}")
            };
            lane.save(&shot);
            glasnost_sim::objects::remove(lane.world_mut(), id);
        }
    }

    // S-0504: ships on open water, moving, then one sinking.
    let w = lane.world();
    let ship = find_type(w, Kind::Vehicle, |_, u| {
        u.locomotor == Locomotor::Ship && u.combat.naval && !u.combat.underwater
    });
    let sea = lane.open(target, |l| l == land::WATER);
    let water = lane
        .world()
        .cells
        .cell(sea)
        .is_some_and(|c| c.land == land::WATER);
    println!("ship type found: {}, open water: {water}", ship.is_some());
    if let (Some(sn), true) = (ship, water) {
        // Three water cells four apart, so that no ship hides another.
        let mut cells: Vec<CellXY> = Vec::new();
        for (dx, dy) in [
            (0, 0),
            (4, 0),
            (0, 4),
            (4, 4),
            (8, 0),
            (0, 8),
            (-4, 0),
            (0, -4),
        ] {
            let c = CellXY::new(sea.x + dx, sea.y + dy);
            let wet = lane
                .world()
                .cells
                .cell(c)
                .is_some_and(|k| k.land == land::WATER);
            if wet && cells.len() < 3 {
                cells.push(c);
            }
        }
        let ids: Vec<ObjectId> = cells
            .iter()
            .map(|&at| lane.spawn(Kind::Vehicle, &sn, local, at))
            .collect();
        let dest = lane.open(CellXY::new(sea.x + 12, sea.y + 12), |l| l == land::WATER);
        // The first ship stays where it is, alone, for the sinking pictures.
        for &id in &ids[1..] {
            lane.attack(local, id, ClickOn::Cell(dest));
        }
        lane.ticks(40);
        if let Some(c) = ids
            .get(1)
            .and_then(|&id| lane.world().obj(id))
            .map(|o| o.pos.cell())
        {
            lane.look_at(c);
            lane.ticks(1);
            lane.save("ships-moving");
        }
        // The sinking ship at rest, so that only its height changes.
        let mut last = None;
        let mut still = 0;
        for _ in 0..900 {
            lane.ticks(1);
            let now = lane
                .world()
                .obj(ids[0])
                .map(|o| (o.pos, o.body.value(lane.world().tick)));
            still = if now == last { still + 1 } else { 0 };
            last = now;
            if still >= 10 {
                break;
            }
        }
        println!("first ship at rest before the hit: {}", still >= 10);
        if let Some(c) = lane.world().obj(ids[0]).map(|o| o.pos.cell()) {
            lane.look_at(c);
            lane.ticks(1);
            lane.save("ship-before-sinking");
        }
        let hit = glasnost_sim::combat::damage::Hit {
            amount: 100_000,
            warhead: None,
            distance: 0,
            attacker: None,
            house: None,
            ignore_defences: true,
            no_escape: true,
        };
        let _ = glasnost_sim::combat::damage::apply(lane.world_mut(), ids[0], hit);
        let sinking = glasnost_sim::naval::sinking(lane.world(), ids[0]);
        println!("ship set sinking by a killing hit: {sinking}");
        // A sinking vessel goes down 5 leptons a tick and leaves play below
        // −400 (S-0504 claim 4): pictures early, half-way and after it left,
        // from one camera place, with the other ships as the level reference.
        {
            for (wait, name) in [
                (20, "ship-sinking"),
                (40, "ship-sinking-deep"),
                (30, "ship-sunk"),
            ] {
                lane.ticks(wait);
                let w = lane.world();
                let z = w.obj(ids[0]).map(|_| w.height_above_surface(ids[0]));
                println!("{name}: sinking ship height above surface {z:?}");
                lane.save(name);
            }
        }
    }
}

trait AtCell {
    fn obj_at_cell_owner(&self, c: CellXY, h: HouseId) -> Option<ObjectId>;
}

impl AtCell for World {
    /// An object of `h` at cell `c`, or else the nearest one of `h`.
    fn obj_at_cell_owner(&self, c: CellXY, h: HouseId) -> Option<ObjectId> {
        self.objects
            .values()
            .filter(|o| o.on_map && o.owner == Some(h))
            .min_by_key(|o| {
                let k = o.pos.cell();
                let (dx, dy) = (i64::from(k.x - c.x), i64::from(k.y - c.y));
                dx * dx + dy * dy
            })
            .map(|o| o.id)
    }
}
