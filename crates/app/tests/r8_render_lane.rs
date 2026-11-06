//! Local lane for the M8 A4 round 8 rendering and UI notes (never in CI):
//! with `GLASNOST_GAME_DIR` set, starts a skirmish through the windowless
//! client and plays notes 45 and 46 (a squid's life on a ship, both ways
//! round), note 47 (a hovering vehicle on land and on water), note 49 (a
//! veteran infantry unit going into a building) and note 54 (the pointer
//! resting on a unit). Screens go to `<out>/<tag>/` (git-ignored), `<out>`
//! being `GLASNOST_LANE_OUT` (default `local-lane/m8/a4-r8/render`) and the
//! tag `GLASNOST_R8_TAG` (default `after`). Types are chosen by their
//! fields, never by name; it prints counts, cells, ticks and booleans only.
//! `GLASNOST_UI_MAP=n` picks the n-th offered map (default 0); the water
//! scenes need open water and are skipped without it.
//!
//! Run with `cargo test --release -p glasnost-app --test r8_render_lane --
//! --ignored --nocapture --test-threads 1`.

#![cfg(not(target_arch = "wasm32"))]

use std::rc::Rc;

use glasnost_app::game::client::{Client, run_job};
use glasnost_app::game::play::Button;
use glasnost_app::mapview::png;
use glasnost_map::CellXY;
use glasnost_map::terrain::land;
use glasnost_num::{F64, Rounding};
use glasnost_render::scene::Group;
use glasnost_sim::rules::{Kind, Locomotor};
use glasnost_sim::targeting::{self, ClickOn, Modifiers};
use glasnost_sim::world::{HouseId, HouseRole, ObjectId, World};
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
    /// Starts a skirmish from the menu on the `GLASNOST_UI_MAP`-th map, the
    /// whole map revealed to the local player.
    fn start() -> Option<Lane> {
        let dir = std::env::var_os("GLASNOST_GAME_DIR")?;
        let tag = std::env::var("GLASNOST_R8_TAG").unwrap_or_else(|_| "after".into());
        let out = png::lane_out("m8/a4-r8/render").join(tag);
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
        println!("map {map}");
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

    fn look_at(&mut self, cell: CellXY) {
        let s = self.c.session.as_mut().unwrap();
        let (x, y) = glasnost_render::projection::cell_centre(cell.x, cell.y);
        s.set_centre(glasnost_render::projection::project_lifted(x, y, 0));
    }

    /// The mean screen point of the pixels that hit object `obj`.
    fn centre_of(&mut self, obj: ObjectId) -> Option<Point> {
        let _ = self.c.frame();
        let s = self.c.session.as_mut().unwrap();
        let m = s.map_rect();
        let (mut sx, mut sy, mut n) = (0i64, 0i64, 0i64);
        for y in (m.y..m.y + m.h).step_by(2) {
            for x in (m.x..m.x + m.w).step_by(2) {
                if s.under(Point::new(x, y)).1 == Some(obj) {
                    (sx, sy, n) = (sx + i64::from(x), sy + i64::from(y), n + 1);
                }
            }
        }
        (n > 0).then(|| Point::new((sx / n) as i32, (sy / n) as i32))
    }

    /// The screen point of the centre of `cell`, after a frame.
    fn screen_of(&mut self, cell: CellXY) -> Option<Point> {
        let _ = self.c.frame();
        let s = self.c.session.as_mut().unwrap();
        let m = s.map_rect();
        for y in (m.y..m.y + m.h).step_by(2) {
            for x in (m.x..m.x + m.w).step_by(2) {
                if s.under(Point::new(x, y)).0 == cell {
                    return Some(Point::new(x, y + 8));
                }
            }
        }
        None
    }

    fn save(&mut self, name: &str) {
        let rgba = self.c.frame_rgba();
        let (w, h) = self.c.size;
        std::fs::write(
            self.dir.join(format!("{name}.png")),
            png::png(w as u32, h as u32, &rgba),
        )
        .unwrap();
        println!("wrote {name}.png (tick {})", self.world().tick);
    }

    /// Writes `name-zoom.png`: the `cw` × `ch` pixels around `at`, enlarged
    /// `k` times.
    fn zoom(&mut self, name: &str, at: Point, (cw, ch, k): (i32, i32, i32)) {
        let rgba = self.c.frame_rgba();
        let (w, h) = self.c.size;
        let (x0, y0) = (
            (at.x - cw / 2).clamp(0, w - cw),
            (at.y - ch / 2).clamp(0, h - ch),
        );
        let mut out = Vec::with_capacity((cw * ch * k * k * 4) as usize);
        for y in 0..ch * k {
            for x in 0..cw * k {
                let i = (((y0 + y / k) * w + x0 + x / k) * 4) as usize;
                out.extend_from_slice(&rgba[i..i + 4]);
            }
        }
        std::fs::write(
            self.dir.join(format!("{name}-zoom.png")),
            png::png((cw * k) as u32, (ch * k) as u32, &out),
        )
        .unwrap();
        println!("wrote {name}-zoom.png");
    }

    /// The scene items of object `id`, by group.
    fn items(&self, id: ObjectId) -> Vec<(Group, i32, i32)> {
        let code = glasnost_app::mapview::live::sim_code(id);
        self.c
            .session
            .as_ref()
            .unwrap()
            .scene
            .items
            .iter()
            .filter(|it| it.object == Some(code))
            .map(|it| (it.group, it.x, it.y))
            .collect()
    }

    fn count(&self, id: ObjectId, g: Group) -> usize {
        self.items(id).iter().filter(|i| i.0 == g).count()
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

    /// The nearest cell to `near` whose 7 × 7 neighbourhood passes `ok`.
    fn open(&self, near: CellXY, ok: impl Fn(u8) -> bool) -> Option<CellXY> {
        let w = self.world();
        let good = |c: CellXY| {
            (-3..=3).all(|dx| {
                (-3..=3).all(|dy| {
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
    }

    fn home(&self) -> CellXY {
        let w = self.world();
        let local = self.local();
        w.objects
            .values()
            .find(|o| o.on_map && o.owner == Some(local))
            .map(|o| o.pos.cell())
            .expect("a local object")
    }

    fn water_cells(&self) -> usize {
        self.world()
            .cells
            .grid
            .iter()
            .filter(|(_, c)| c.land == land::WATER)
            .count()
    }
}

/// The name of the first vehicle type matching `f`.
fn vehicle_type(w: &World, f: impl Fn(&glasnost_sim::rules::UnitRules) -> bool) -> Option<String> {
    w.rules()
        .vehicles
        .iter()
        .find(|u| f(u))
        .map(|u| u.name.clone())
}

fn elite() -> glasnost_num::F32 {
    let rz = Rounding::TowardZero;
    F64::from_i64(5, rz)
        .div(F64::from_i64(2, rz), rz)
        .to_f32(rz)
}

/// Notes 45 and 46: a squid (a naval, organic vehicle type) of `squid_house`
/// sent at a ship of the other house, followed tick by tick through its
/// approach, latch, grab, pushes and strikes until the ship sinks or the
/// squid comes out, with zoomed screens of the ship.
fn squid_scene(lane: &mut Lane, ours: bool, tag: &str) {
    let w = lane.world();
    let Some(squid_t) = vehicle_type(w, |u| u.combat.naval && u.combat.organic) else {
        println!("no squid type");
        return;
    };
    let Some(ship_t) = vehicle_type(w, |u| {
        u.locomotor == Locomotor::Ship
            && u.combat.naval
            && !u.combat.underwater
            && !u.combat.organic
    }) else {
        println!("no ship type");
        return;
    };
    let home = lane.home();
    let Some(sea) = lane.open(home, |l| l == land::WATER) else {
        println!("no open water");
        return;
    };
    let (local, enemy) = (lane.local(), lane.enemy());
    let (sq_house, ship_house) = if ours { (local, enemy) } else { (enemy, local) };
    let ship = lane.spawn(Kind::Vehicle, &ship_t, ship_house, sea);
    let squid = lane.spawn(
        Kind::Vehicle,
        &squid_t,
        sq_house,
        CellXY::new(sea.x + 3, sea.y + 2),
    );
    println!("{tag}: sea {sea:?}, squid ours {ours}");
    lane.ticks(40);
    lane.look_at(sea);
    if let Some(p) = lane.screen_of(sea) {
        lane.zoom(&format!("{tag}-00-before"), p, (200, 160, 3));
    }
    lane.save(&format!("{tag}-00-before"));
    let n = lane.order(sq_house, squid, ClickOn::Object(ship));
    println!("{tag}: attack orders taken {n}");
    let mut last = String::new();
    let mut changed_at = 0;
    let full = lane.world().obj(ship).map_or(0, |o| o.health);
    let (mut seen, mut hits) = (full, 0);
    for t in 0..2800 {
        lane.ticks(1);
        let w = lane.world();
        let Some(sh) = w.obj(ship) else {
            println!("{tag} +{t}: ship gone after {hits} hits");
            break;
        };
        if sh.health < seen {
            hits += 1;
            seen = sh.health;
        }
        let health = format!("damaged {}, hits {hits}", sh.health < full);
        let sinking = glasnost_sim::naval::sinking(w, ship);
        let (on_map, cloak, host) = match w.obj(squid) {
            Some(o) => (
                o.on_map,
                format!("{:?}", glasnost_sim::cloak::state(w, squid)),
                glasnost_sim::parasite::host_of(w, squid).is_some(),
            ),
            None => (false, "gone".into(), false),
        };
        let anims = w.anims.anims.len();
        let _ = lane.c.frame();
        let sq_items = lane.items(squid).len();
        let ship_items = lane.items(ship).len();
        let state = format!(
            "squid on map {on_map}, cloak {cloak}, attached {host}, squid items {sq_items}; ship {health}, sinking {sinking}, ship items {ship_items}"
        );
        let anim_note = format!(", animations {anims}");
        if state != last {
            println!("{tag} +{t}: {state}{anim_note}");
            // Health alone changing is not a phase change worth frames.
            let phase = |s: &str| s.split("; ship").next().unwrap_or("").to_string();
            if phase(&state) != phase(&last) || sinking {
                changed_at = t;
            }
            last = state;
        }
        if t % 6 == 0 && (t < 600 || t - changed_at < 120) {
            let at = lane.world().obj(ship).map(|o| o.pos.cell()).unwrap_or(sea);
            lane.look_at(at);
            if let Some(p) = lane.screen_of(at) {
                lane.zoom(&format!("{tag}-t{t:04}"), p, (200, 160, 3));
            }
        }
        if t == 120 || t == 240 {
            lane.save(&format!("{tag}-t{t:04}"));
        }
    }
    // Let the squid come out and move on, then watch it a little.
    for t in 0..120 {
        lane.ticks(1);
        if t % 10 == 0
            && let Some(o) = lane.world().obj(squid)
            && o.on_map
        {
            let c = o.pos.cell();
            lane.look_at(c);
            if let Some(p) = lane.screen_of(c) {
                lane.zoom(&format!("{tag}-after-{t:03}"), p, (200, 160, 3));
            }
        }
    }
}

#[test]
#[ignore = "local lane: needs GLASNOST_GAME_DIR"]
fn notes_45_46_a_squid_on_a_ship() {
    let Some(mut lane) = Lane::start() else {
        eprintln!("GLASNOST_GAME_DIR not set: skipped");
        return;
    };
    println!("water cells {}", lane.water_cells());
    squid_scene(&mut lane, true, "n45-ours");
    squid_scene(&mut lane, false, "n45-theirs");
}

/// Note 47: a vehicle of the first hovering type, on open land and on open
/// water: its shadow items against its body items, standing and moving.
#[test]
#[ignore = "local lane: needs GLASNOST_GAME_DIR"]
fn note_47_a_hovering_vehicles_shadow() {
    let Some(mut lane) = Lane::start() else {
        eprintln!("GLASNOST_GAME_DIR not set: skipped");
        return;
    };
    let Some(hover_t) = vehicle_type(lane.world(), |u| u.locomotor == Locomotor::Hover) else {
        println!("no hovering type");
        return;
    };
    let Some(driver_t) = vehicle_type(lane.world(), |u| {
        u.locomotor == Locomotor::Drive && !u.combat.naval
    }) else {
        println!("no driving type");
        return;
    };
    let home = lane.home();
    let local = lane.local();
    let ground = lane
        .open(CellXY::new(home.x + 6, home.y + 6), |l| {
            l != land::WATER && l != land::ROCK
        })
        .expect("open land");
    let mut places = vec![("land", ground)];
    if let Some(sea) = lane.open(home, |l| l == land::WATER) {
        places.push(("water", sea));
    }
    for (name, at) in places {
        let h = lane.spawn(Kind::Vehicle, &hover_t, local, at);
        let d = (name == "land")
            .then(|| lane.spawn(Kind::Vehicle, &driver_t, local, CellXY::new(at.x + 2, at.y)));
        lane.ticks(60);
        lane.look_at(at);
        let _ = lane.c.frame();
        let w = lane.world();
        let o = w.obj(h).unwrap();
        let above = o.pos.z > w.surface_at(o.pos.x, o.pos.y);
        // Rows from the body item's draw point down to the shadow item's
        // (kept to the lane: only the comparison is printed).
        let drop = |items: &[(Group, i32, i32)]| {
            let row = |g| items.iter().find(|i| i.0 == g).map(|i| i.2);
            Some(row(Group::Shadows)? - row(Group::Vehicles)?)
        };
        let hover_drop = drop(&lane.items(h));
        println!(
            "{name}: hovering above its surface {above}; hover shadows {}, bodies {}",
            lane.count(h, Group::Shadows),
            lane.count(h, Group::Vehicles),
        );
        if let Some(d) = d {
            let driver_drop = drop(&lane.items(d));
            println!(
                "  driver shadows {}; the hover's shadow item sits lower under its body than the driver's {}",
                lane.count(d, Group::Shadows),
                hover_drop.zip(driver_drop).is_some_and(|(a, b)| a > b)
            );
        }
        if let Some(p) = lane.screen_of(at) {
            lane.zoom(&format!("n47-{name}"), p, (160, 120, 4));
        }
        lane.save(&format!("n47-{name}"));
        let dest = CellXY::new(at.x - 2, at.y + 2);
        lane.order(local, h, ClickOn::Cell(dest));
        for t in 0..30 {
            lane.ticks(1);
            if t == 12 {
                lane.look_at(at);
                if let Some(p) = lane.centre_of(h) {
                    println!(
                        "{name} moving: shadows {}, bodies {}",
                        lane.count(h, Group::Shadows),
                        lane.count(h, Group::Vehicles)
                    );
                    lane.zoom(&format!("n47-{name}-moving"), p, (160, 120, 4));
                }
            }
        }
    }
}

/// Note 49: an elite infantry unit of the first occupier type sent into a
/// building of a civilian garrisonable type: the status items of the unit
/// and of the building before and after it goes in, plain, hovered and
/// selected.
#[test]
#[ignore = "local lane: needs GLASNOST_GAME_DIR"]
fn note_49_a_veteran_goes_into_a_building() {
    let Some(mut lane) = Lane::start() else {
        eprintln!("GLASNOST_GAME_DIR not set: skipped");
        return;
    };
    let w = lane.world();
    let r = w.rules_arc();
    let Some(inf_t) = r
        .infantry
        .iter()
        .find(|u| u.raw.occupier == Some(true))
        .map(|u| u.name.clone())
    else {
        println!("no occupier type");
        return;
    };
    let Some(bt) = r
        .buildings
        .iter()
        .find(|b| {
            b.building.can_be_occupied
                && b.building.max_occupants >= 2
                && b.raw.tech_level == Some(-1)
                && b.foundation.0 <= 3
                && b.foundation.1 <= 3
        })
        .map(|b| b.name.clone())
    else {
        println!("no civilian garrisonable type");
        return;
    };
    let civ = glasnost_sim::structures::garrison::civilian_house(w);
    println!("civilian house found {}", civ.is_some());
    let home = lane.home();
    let local = lane.local();
    let at = lane
        .open(CellXY::new(home.x + 7, home.y + 7), |l| {
            l != land::WATER && l != land::ROCK
        })
        .expect("open land");
    let b = lane.spawn(Kind::Building, &bt, civ.unwrap_or(local), at);
    let units: Vec<ObjectId> = (0..2)
        .map(|k| {
            let u = lane.spawn(
                Kind::Infantry,
                &inf_t,
                local,
                CellXY::new(at.x + 4 + k, at.y + 4),
            );
            lane.world_mut().obj_mut(u).unwrap().combat.experience = elite();
            u
        })
        .collect();
    lane.ticks(10);
    lane.look_at(at);
    let _ = lane.c.frame();
    for &u in &units {
        println!(
            "outside: unit status items {}",
            lane.count(u, Group::Status)
        );
    }
    println!(
        "outside: building status items {}",
        lane.count(b, Group::Status)
    );
    if let Some(p) = lane.screen_of(at) {
        lane.zoom("n49-0-outside", p, (220, 180, 3));
    }
    for &u in &units {
        let n = lane.order(local, u, ClickOn::Object(b));
        println!("enter orders taken {n}");
    }
    println!("building cell {at:?}");
    for t in 0..400 {
        lane.ticks(1);
        let w = lane.world();
        if t % 50 == 0 {
            for &u in &units {
                if let Some(o) = w.obj(u) {
                    println!(
                        "  +{t}: unit cell {:?}, on map {}, destination {}, contact {}",
                        o.pos.cell(),
                        o.on_map,
                        o.dest.is_some(),
                        o.contact.is_some()
                    );
                }
            }
        }
        if units.iter().all(|&u| w.obj(u).is_some_and(|o| !o.on_map)) {
            break;
        }
    }
    let w = lane.world();
    println!(
        "inside: {:?}; building occupants {}",
        units
            .iter()
            .map(|&u| w.obj(u).map(|o| (o.on_map, o.structure.inside.is_some())))
            .collect::<Vec<_>>(),
        glasnost_sim::structures::garrison::count(w, b)
    );
    let _ = lane.c.frame();
    for &u in &units {
        println!(
            "inside: unit items {}, status items {}",
            lane.items(u).len(),
            lane.count(u, Group::Status)
        );
    }
    println!(
        "inside: building status items {}, building has experience {}",
        lane.count(b, Group::Status),
        lane.world().obj(b).unwrap().combat.experience != glasnost_num::F32::ZERO
    );
    if let Some(p) = lane.screen_of(at) {
        lane.zoom("n49-1-inside", p, (220, 180, 3));
    }
    lane.save("n49-1-inside");
    // Hovered and selected.
    if let Some(p) = lane.centre_of(b) {
        lane.c.pointer_moved(p);
        let _ = lane.c.frame();
        println!(
            "hovered: building status items {}",
            lane.count(b, Group::Status)
        );
        lane.zoom("n49-2-hovered", p, (220, 180, 3));
    }
    lane.c
        .session
        .as_mut()
        .unwrap()
        .play
        .select_box(&[b], false);
    let _ = lane.c.frame();
    println!(
        "selected: building status items {}",
        lane.count(b, Group::Status)
    );
    if let Some(p) = lane.centre_of(b) {
        lane.zoom("n49-3-selected", p, (220, 180, 3));
    }
    // An enemy walks up: the occupants fire, and whoever gains experience shows it.
    let enemy = lane.enemy();
    let foes: Vec<ObjectId> = (0..3)
        .map(|k| {
            lane.spawn(
                Kind::Infantry,
                &inf_t,
                enemy,
                CellXY::new(at.x - 4, at.y + k),
            )
        })
        .collect();
    lane.c.session.as_mut().unwrap().play.select_box(&[], false);
    lane.ticks(300);
    let w = lane.world();
    println!(
        "after the fight: foes alive {}, building has experience {}",
        foes.iter().filter(|&&f| w.obj(f).is_some()).count(),
        w.obj(b).unwrap().combat.experience != glasnost_num::F32::ZERO
    );
    let _ = lane.c.frame();
    println!(
        "after the fight: building status items {}",
        lane.count(b, Group::Status)
    );
    if let Some(p) = lane.screen_of(at) {
        lane.zoom("n49-4-after-fight", p, (220, 180, 3));
    }
}

/// Note 49, the cloak side: an elite unit of the first cloaking naval type
/// on open water, once the local house's and once the enemy's, after it has
/// had time to cloak: its status items (S-0111 claim 18: no insignia while
/// the cloak hides it from the viewer; the owner sees it).
#[test]
#[ignore = "local lane: needs GLASNOST_GAME_DIR"]
fn note_49_an_elite_unit_under_its_cloak() {
    let Some(mut lane) = Lane::start() else {
        eprintln!("GLASNOST_GAME_DIR not set: skipped");
        return;
    };
    let Some(t) = vehicle_type(lane.world(), |u| u.combat.naval && u.combat.organic) else {
        println!("no cloaking naval type");
        return;
    };
    let home = lane.home();
    let Some(sea) = lane.open(home, |l| l == land::WATER) else {
        println!("no open water");
        return;
    };
    // The enemy's first, alone, so that nothing of ours makes it surface.
    for (who, k) in [("theirs", 0), ("ours", 3)] {
        let house = if who == "ours" {
            lane.local()
        } else {
            lane.enemy()
        };
        let u = lane.spawn(Kind::Vehicle, &t, house, CellXY::new(sea.x + k, sea.y));
        lane.world_mut().obj_mut(u).unwrap().combat.experience = elite();
        lane.ticks(60);
        lane.look_at(sea);
        let _ = lane.c.frame();
        println!(
            "{who}: cloak {:?}, body items {}, status items {}",
            glasnost_sim::cloak::state(lane.world(), u),
            lane.count(u, Group::Vehicles),
            lane.count(u, Group::Status)
        );
        if let Some(p) = lane.screen_of(CellXY::new(sea.x + k, sea.y)) {
            lane.zoom(&format!("n49-cloaked-{who}"), p, (160, 120, 4));
        }
    }
}

/// Note 54: the pointer rests on a unit for a while; screens at the start
/// and after two seconds of the host clock.
#[test]
#[ignore = "local lane: needs GLASNOST_GAME_DIR"]
fn note_54_the_pointer_rests_on_a_unit() {
    let Some(mut lane) = Lane::start() else {
        eprintln!("GLASNOST_GAME_DIR not set: skipped");
        return;
    };
    let home = lane.home();
    let local = lane.local();
    let Some(t) = vehicle_type(lane.world(), |u| {
        u.locomotor == Locomotor::Drive && !u.combat.naval && u.combat.slots[0].is_some()
    }) else {
        println!("no armed driving type");
        return;
    };
    let at = lane
        .open(CellXY::new(home.x + 5, home.y + 5), |l| {
            l != land::WATER && l != land::ROCK
        })
        .expect("open land");
    let v = lane.spawn(Kind::Vehicle, &t, local, at);
    lane.ticks(5);
    lane.look_at(at);
    let Some(p) = lane.centre_of(v) else {
        println!("the unit is not on screen");
        return;
    };
    lane.c.pointer_moved(p);
    let _ = lane.c.frame();
    lane.zoom("n54-0-at-once", p, (240, 140, 3));
    // S-0111 claims 20 to 26: the name tip's box and lines, once it shows.
    let tag = |l: &Lane| {
        l.c.session
            .as_ref()
            .and_then(|s| s.name_tip_shown())
            .map(|t| (t.rect, t.lines.len(), t.ink))
    };
    println!("at once: name tip {:?}", tag(&lane));
    // Two seconds of the host clock in steps of 100 ms (the wait is 1000 ms, S-0111 claim 20).
    for k in 1..=20 {
        lane.clock += 100;
        lane.c.update(lane.clock);
        let _ = lane.c.frame();
        if k % 2 == 0 || k == 9 {
            println!("+{} ms: name tip {:?}", k * 100, tag(&lane));
        }
    }
    lane.zoom("n54-1-two-seconds", p, (240, 140, 3));
    // The pointer moves: the tip goes (S-0111 claim 20).
    lane.c.pointer_moved(Point::new(p.x + 200, p.y + 120));
    let _ = lane.c.frame();
    println!("pointer moved off: name tip {:?}", tag(&lane));
    lane.save("n54-1-two-seconds");
}

/// Intake 109 (S-0015): the install's `ra2md.csf` reads at the start of a
/// game. Counts only: no label or text is printed or compared.
#[test]
#[ignore = "local lane: needs GLASNOST_GAME_DIR"]
fn i109_the_install_string_table_reads() {
    let Some(lane) = Lane::start() else {
        eprintln!("GLASNOST_GAME_DIR not set: skipped");
        return;
    };
    let s = lane.c.session.as_ref().unwrap();
    let t = s.strings.as_ref().expect("the string table is read");
    // The reading's warnings went to the log when the program started
    // (they carry labels, so they are never printed here).
    println!("string table: {} entries", t.len());
    assert!(!t.is_empty());
}

/// Intake 109 (S-0015, S-0111 r40): the name tip of a unit and of a
/// building shows the display name read from the install's string table.
/// Screens go under the tag (`GLASNOST_R8_TAG`: `before` on main, `after`
/// here). It prints the tip's size and line count and whether its text is
/// the placeholder, never the text.
#[test]
#[ignore = "local lane: needs GLASNOST_GAME_DIR"]
fn i109_the_name_tip_shows_the_display_name() {
    let Some(mut lane) = Lane::start() else {
        eprintln!("GLASNOST_GAME_DIR not set: skipped");
        return;
    };
    println!(
        "string table read: {}",
        lane.c.session.as_ref().unwrap().strings.is_some()
    );
    let home = lane.home();
    let local = lane.local();
    let r = lane.world().rules_arc();
    let Some(vt) = vehicle_type(lane.world(), |u| {
        u.locomotor == Locomotor::Drive && !u.combat.naval && u.combat.slots[0].is_some()
    }) else {
        println!("no armed driving type");
        return;
    };
    let Some(bt) = r
        .buildings
        .iter()
        .find(|b| b.raw.tech_level.is_some_and(|t| t >= 0) && b.foundation.0 <= 3)
        .map(|b| b.name.clone())
    else {
        println!("no building type");
        return;
    };
    for (name, kind, t, dx) in [
        ("unit", Kind::Vehicle, vt, 5),
        ("building", Kind::Building, bt, 9),
    ] {
        let at = lane
            .open(CellXY::new(home.x + dx, home.y + dx), |l| {
                l != land::WATER && l != land::ROCK
            })
            .expect("open land");
        let o = lane.spawn(kind, &t, local, at);
        lane.ticks(8);
        lane.look_at(at);
        let Some(p) = lane.centre_of(o) else {
            println!("{name}: not on screen");
            continue;
        };
        lane.c.pointer_moved(p);
        let _ = lane.c.frame();
        for _ in 0..12 {
            lane.clock += 100;
            lane.c.update(lane.clock);
            let _ = lane.c.frame();
        }
        let s = lane.c.session.as_ref().unwrap();
        let tip = s.name_tip_shown().map(|t| {
            let text = t.lines.concat();
            let placeholder = text.starts_with(&"MISSING:'".encode_utf16().collect::<Vec<_>>());
            (t.rect, t.lines.len(), text.len(), placeholder)
        });
        println!("{name}: name tip (box, lines, units, placeholder) {tip:?}");
        lane.zoom(&format!("i109-{name}"), p, (300, 140, 3));
        lane.c.pointer_moved(Point::new(p.x + 300, p.y + 100));
        let _ = lane.c.frame();
    }
}
