//! Local lane for render issues batch R8 (never in CI): with
//! `GLASNOST_GAME_DIR` set, each scene starts its own skirmish through the
//! windowless client and writes its CPU screens to
//! `local-lane/render-issues/r8/<tag>/` (git-ignored; the tag is
//! `GLASNOST_R8_TAG`, default `after`; `GLASNOST_LANE_OUT` moves it). Types
//! are chosen by their fields, never by name; it prints counts, cells,
//! ticks, booleans and positions relative to the ground only.
//!
//! Scenes: `yard` (#429: a harvester behind the local construction yard),
//! `bunker` (#421: a vehicle clicked into a tank bunker), `infantry`
//! (#428: infantry of two houses fighting, every infantry pair on one
//! point and every pair drawn on one point), `gas` (#424: a gas warhead's
//! cloud made on a high bridge's deck, its particles' height against the
//! deck's). `GLASNOST_LANE_SCENE=a,b` plays only those scenes.
//!
//! Run with `cargo test --release -p glasnost-app --test r8_lane --
//! --ignored --nocapture --test-threads 1`.

#![cfg(not(target_arch = "wasm32"))]

use std::rc::Rc;

use glasnost_app::game::client::{Client, Key, run_job};
use glasnost_app::game::play::Button;
use glasnost_app::mapview::live::sim_code;
use glasnost_app::mapview::png;
use glasnost_map::CellXY;
use glasnost_map::terrain::land;
use glasnost_render::compose::Paint;
use glasnost_render::scene::Group;
use glasnost_sim::rules::Kind;
use glasnost_sim::targeting::{self, ClickOn, Modifiers};
use glasnost_sim::world::{HouseId, Layer, ObjectId, Pos, World};
use glasnost_ui::geom::{Point, Rect};
use glasnost_ui::menu::MenuHit;
use glasnost_ui::setup::SetupHit;
use glasnost_ui::shell::Screen;
use glasnost_vfs::source::{FileSource, Unopened};
use glasnost_vfs::{ByteSource, MountOptions, PickedFile};

type Scene = (&'static str, fn());

const SCENES: &[Scene] = &[
    ("yard", yard),
    ("bunker", bunker),
    ("infantry", infantry),
    ("gas", gas),
];

#[test]
#[ignore = "local lane: needs GLASNOST_GAME_DIR"]
fn r8_scenes() {
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
        play();
    }
}

// ---------------------------------------------------------------------------
// Scenes

/// #429: the local base unit deploys; a harvester of the local house is
/// placed on the cell behind the middle of the yard's back edge. The yard's
/// items drawn after the harvester's first item, and the screen.
fn yard() {
    // The local country stepped 0, 2, … 8 times in the lobby, for yards of
    // both sides.
    for steps in [0, 2, 4, 6, 8] {
        println!("-- local country stepped {steps} times");
        yard_with(steps);
    }
}

/// [`yard`] with the local country stepped `steps` times.
fn yard_with(steps: i32) {
    let Some(mut lane) = Lane::start_with(0, false, steps) else {
        return;
    };
    let local = lane.local();
    lane.deploy();
    let w = lane.world();
    let Some((yard, at, size)) = w
        .objects
        .values()
        .find(|o| o.kind == Kind::Building && o.on_map && o.owner == Some(local))
        .map(|o| {
            (
                o.id,
                o.pos.cell(),
                w.rules().buildings[o.type_index as usize].foundation,
            )
        })
    else {
        println!("no yard");
        return;
    };
    let r = w.rules();
    let harvester = (0..r.vehicles.len())
        .find(|&i| {
            r.unit(Kind::Vehicle, i as u32)
                .is_some_and(|u| u.harvester && u.strength > 0)
        })
        .map(|i| r.vehicles[i].name.clone());
    println!(
        "yard foundation {size:?}, harvester type found: {}",
        harvester.is_some()
    );
    let Some(hname) = harvester else { return };
    let h = lane.spawn(
        Kind::Vehicle,
        &hname,
        local,
        CellXY::new(at.x + size.0 / 2, at.y - 1),
    );
    let h2 = lane.spawn(
        Kind::Vehicle,
        &hname,
        local,
        CellXY::new(at.x + size.0 - 1, at.y - 1),
    );
    lane.ticks(3);
    lane.look_at(CellXY::new(at.x + 1, at.y + 1));
    lane.ticks(2);
    println!(
        "yard type index {}, running slot animations of the yard: {}",
        lane.world().objects[&yard].type_index,
        lane.slot_list(yard).len()
    );
    lane.order_report("yard, harvester 1", yard, h);
    lane.order_report("yard, harvester 2", yard, h2);
    let name = format!("yard-c{steps}");
    lane.save(&name);
    lane.zoom(&name, yard, (200, 160, 4));
}

/// #421: a tank bunker of the local house and a vehicle it accepts; the
/// enter click sends it in; once bunkered and the walls settled, the
/// bunker's items drawn after the vehicle's first item, and the screen.
fn bunker() {
    use glasnost_sim::structures::bunker;
    let Some(mut lane) = Lane::start(0, false) else {
        return;
    };
    let w = lane.world();
    let ty = w
        .rules()
        .buildings
        .iter()
        .find(|b| b.raw.bunker.unwrap_or(false))
        .map(|b| (b.name.clone(), b.foundation));
    let r = w.rules();
    let veh = (0..r.vehicles.len())
        .find(|&i| {
            r.unit(Kind::Vehicle, i as u32).is_some_and(|u| {
                u.raw.bunkerable.unwrap_or(true)
                    && u.turret
                    && u.combat.slots[0].is_some()
                    && u.class != Some(glasnost_rules::sim::LocomotionClass::Hover)
                    && !u.harvester
                    && u.strength > 0
            })
        })
        .map(|i| r.vehicles[i].name.clone());
    println!(
        "bunker type found: {}, accepted vehicle type found: {}",
        ty.is_some(),
        veh.is_some()
    );
    let (Some((bname, size)), Some(vname)) = (ty, veh) else {
        return;
    };
    let local = lane.local();
    let home = lane.home();
    let Some(at) = lane.area(CellXY::new(home.x + 8, home.y + 8), size, 5, |l| {
        l != land::WATER && l != land::ROCK
    }) else {
        println!("no room for the bunker");
        return;
    };
    let b = lane.spawn(Kind::Building, &bname, local, at);
    lane.ticks(4);
    let v = lane.spawn(
        Kind::Vehicle,
        &vname,
        local,
        CellXY::new(at.x + size.0 + 3, at.y + 1),
    );
    lane.ticks(2);
    let n = lane.order(local, v, ClickOn::Object(b));
    println!("click on the bunker: {n} orders");
    let mut inside = false;
    for t in 0..1500 {
        lane.ticks(1);
        if bunker::bunkered_in(lane.world(), v) == Some(b) {
            println!("bunkered after {t} ticks");
            inside = true;
            break;
        }
    }
    if !inside {
        println!("not bunkered");
        return;
    }
    let (mut last, mut still) = (lane.slot_list(b), 0);
    for t in 0..900 {
        lane.ticks(1);
        let now = lane.slot_list(b);
        still = if now == last { still + 1 } else { 0 };
        last = now;
        if still >= 60 {
            println!("wall slots settled after {t} more ticks");
            break;
        }
    }
    lane.look_at(CellXY::new(at.x + 1, at.y + 1));
    lane.ticks(2);
    lane.order_report("bunker", b, v);
    lane.save("bunker");
    lane.zoom("bunker", b, (160, 120, 5));
}

/// #428: groups of infantry of the local house and of an enemy house on
/// neighbouring open cells, one unit to a cell, fight. Every tick: the
/// infantry pairs on one point of the game state (position X and Y equal,
/// both on the map); every 10 ticks: the pairs at different points whose
/// bodies are drawn at one screen point.
fn infantry() {
    let Some(mut lane) = Lane::start(0, false) else {
        return;
    };
    let w = lane.world();
    let r = w.rules();
    let soldier = (0..r.infantry.len())
        .find(|&i| {
            r.unit(Kind::Infantry, i as u32)
                .is_some_and(|u| u.combat.slots[0].is_some() && u.strength > 0 && !u.engineer)
        })
        .map(|i| r.infantry[i].name.clone());
    let Some(enemy) = lane.enemy() else {
        println!("no enemy house");
        return;
    };
    let Some(sname) = soldier else {
        println!("no armed infantry type");
        return;
    };
    let local = lane.local();
    let home = lane.home();
    let Some(at) = lane.area(CellXY::new(home.x + 10, home.y + 10), (6, 4), 2, |l| {
        l != land::WATER && l != land::ROCK
    }) else {
        println!("no room");
        return;
    };
    let mut ids = Vec::new();
    for dx in 0..3 {
        for dy in 0..4 {
            ids.push(lane.spawn(
                Kind::Infantry,
                &sname,
                local,
                CellXY::new(at.x + dx, at.y + dy),
            ));
            ids.push(lane.spawn(
                Kind::Infantry,
                &sname,
                enemy,
                CellXY::new(at.x + 3 + dx, at.y + dy),
            ));
        }
    }
    lane.look_at(CellXY::new(at.x + 3, at.y + 2));
    let mut same = 0;
    let mut drawn_same = 0;
    let mut shown = 0;
    for t in 0..1500 {
        lane.ticks(1);
        let w = lane.world();
        let inf: Vec<_> = w
            .objects
            .values()
            .filter(|o| o.kind == Kind::Infantry && o.on_map)
            .collect();
        for (i, a) in inf.iter().enumerate() {
            for b in &inf[i + 1..] {
                if a.pos.x == b.pos.x && a.pos.y == b.pos.y && a.pos.z == b.pos.z {
                    same += 1;
                    if shown < 8 {
                        shown += 1;
                        let c = a.pos.cell();
                        println!(
                            "tick {}: two infantry on one point, cell {c:?}, offset in cell ({}, {}), \
                             one house: {}, behaviours {:?} / {:?}, layers {:?} / {:?}",
                            w.tick,
                            a.pos.x - 256 * c.x,
                            a.pos.y - 256 * c.y,
                            a.owner == b.owner,
                            a.beh.current,
                            b.beh.current,
                            a.layer,
                            b.layer,
                        );
                    }
                }
            }
        }
        if t % 10 == 0 {
            let drawn: Vec<(Pos, Option<(i32, i32)>)> =
                inf.iter().map(|o| (o.pos, lane.body_point(o.id))).collect();
            for (i, (pa, da)) in drawn.iter().enumerate() {
                for (pb, db) in &drawn[i + 1..] {
                    if pa != pb && da.is_some() && da == db {
                        drawn_same += 1;
                    }
                }
            }
        }
        if t == 200 {
            lane.save("infantry-200");
        }
    }
    let left = lane
        .world()
        .objects
        .values()
        .filter(|o| o.kind == Kind::Infantry && o.on_map)
        .count();
    println!(
        "infantry placed {}, left {left}; pair-ticks on one point {same}; \
         pairs at different points drawn at one point (every 10 ticks) {drawn_same}",
        ids.len()
    );
}

/// #424: the first warhead whose particle system is a gas system makes its
/// system at the centre of a high-bridge cell at the deck's height
/// (terrain + 416, S-0102 claim 15; S-0613 claim 4: the first particle at
/// the detonation point). Every 5 ticks: the particles' height above the
/// terrain and against the deck.
fn gas() {
    let Some(mut lane) = Lane::start_where(|w| w.cells.deck.iter().any(|d| *d), true) else {
        println!("no map with a high bridge");
        return;
    };
    let w = lane.world();
    let rules = w.rules_arc();
    let name = rules.combat.warheads.iter().find_map(|h| {
        let n = h.particle.clone()?;
        let ti = rules.gas.system(&n)?;
        (rules.gas.systems[ti as usize].kind == Some(glasnost_sim::gas::Behaviour::Gas))
            .then_some(n)
    });
    let Some(name) = name else {
        println!("no gas warhead");
        return;
    };
    // A deck cell with deck cells on all four sides, so the cloud starts in
    // the middle of a span.
    let deck = |c: CellXY| w.cells.is_bridge(c);
    let Some(cell) = w.cells.grid.iter().map(|(k, _)| k).find(|&k| {
        deck(k)
            && deck(CellXY::new(k.x + 1, k.y))
            && deck(CellXY::new(k.x - 1, k.y))
            && deck(CellXY::new(k.x, k.y + 1))
            && deck(CellXY::new(k.x, k.y - 1))
    }) else {
        println!("no deck cell inside a span");
        return;
    };
    let centre = Pos::centre(cell, 0);
    let ground = w.surface_at(centre.x, centre.y);
    let p = Pos::new(centre.x, centre.y, ground + 416);
    let local = lane.local();
    let before: Vec<ObjectId> = w.gas.systems.keys().copied().collect();
    glasnost_sim::gas::warhead_system(lane.world_mut(), &name, p, Some(local));
    let sys = lane
        .world()
        .gas
        .systems
        .keys()
        .copied()
        .find(|k| !before.contains(k));
    let Some(sys) = sys else {
        println!("no system made");
        return;
    };
    println!("gas system made on a deck cell {cell:?} at deck height");
    lane.look_at(cell);
    for t in 0..=120 {
        if t % 5 == 0 {
            let w = lane.world();
            let Some(s) = w.gas.systems.get(&sys) else {
                println!("tick +{t}: system gone");
                break;
            };
            let parts: Vec<(i32, i32)> = s
                .particles
                .iter()
                .filter(|q| !q.dead)
                .map(|q| {
                    let g = w.surface_at(q.pos.x, q.pos.y);
                    let on_deck = w.cells.is_bridge(q.pos.cell());
                    (
                        q.pos.z - g,
                        if on_deck {
                            q.pos.z - (g + 416)
                        } else {
                            i32::MIN
                        },
                    )
                })
                .collect();
            let above: Vec<i32> = parts.iter().map(|p| p.0).collect();
            let deck_rel: Vec<i32> = parts
                .iter()
                .map(|p| p.1)
                .filter(|&d| d != i32::MIN)
                .collect();
            println!(
                "tick +{t}: particles {}, above the terrain min {:?} max {:?}; over deck cells {}, \
                 against the deck min {:?} max {:?}",
                parts.len(),
                above.iter().min(),
                above.iter().max(),
                deck_rel.len(),
                deck_rel.iter().min(),
                deck_rel.iter().max(),
            );
            if t == 30 {
                lane.save("gas-30");
            }
        }
        lane.ticks(1);
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
    /// The number of maps the install offers.
    fn map_count() -> usize {
        let Some(dir) = std::env::var_os("GLASNOST_GAME_DIR") else {
            return 0;
        };
        let mut c = Client::new();
        c.mount(files(std::path::Path::new(&dir)));
        run_jobs(&mut c);
        c.install.as_ref().map_or(0, |i| i.maps.len())
    }

    /// Starts a skirmish on the first offered map whose world satisfies
    /// `fits`, chosen by content, never by name.
    fn start_where(fits: impl Fn(&World) -> bool, reveal: bool) -> Option<Lane> {
        for i in 0..Lane::map_count() {
            let lane = Lane::start(i, reveal)?;
            if fits(lane.world()) {
                println!("map index {i} fits");
                return Some(lane);
            }
        }
        None
    }

    /// Starts a skirmish on the `map`-th offered map; with `reveal`, the
    /// map revealed to the local house.
    fn start(map: usize, reveal: bool) -> Option<Lane> {
        Lane::start_with(map, reveal, 0)
    }

    /// [`Lane::start`] with the local slot's country stepped `steps` times.
    fn start_with(map: usize, reveal: bool, steps: i32) -> Option<Lane> {
        let dir = std::env::var_os("GLASNOST_GAME_DIR")?;
        let tag = std::env::var("GLASNOST_R8_TAG").unwrap_or_else(|_| "after".into());
        let out = png::lane_out(&format!("render-issues/r8/{tag}"));
        std::fs::create_dir_all(&out).unwrap();
        let mut c = Client::new();
        c.resize((1280, 800));
        c.mount(files(std::path::Path::new(&dir)));
        run_jobs(&mut c);
        let maps = c.install.as_ref().expect("the install reads").maps.len();
        let map = map.min(maps - 1);
        let _ = c.frame();
        let r = c.menu_targets.rect_of(&MenuHit::Skirmish).unwrap();
        click(&mut c, r);
        for _ in 0..map {
            let _ = c.frame();
            let r = c.setup_targets.rect_of(&SetupHit::Map(1)).unwrap();
            click(&mut c, r);
        }
        for _ in 0..steps {
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
        let mut lane = Lane {
            c,
            clock: 0,
            dir: out,
        };
        if reveal {
            let s = lane.c.session.as_mut().unwrap();
            let local = s.play.me();
            glasnost_sim::econ::shroud::defeat_reveal(&mut s.play.game.world, local);
        }
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

    /// The local base unit, deployed where it stands.
    fn deploy(&mut self) {
        self.ticks(5);
        let local = self.local();
        let mcv = self
            .world()
            .objects
            .values()
            .find(|o| o.owner == Some(local) && o.kind == Kind::Vehicle && o.on_map)
            .map(|o| o.id)
            .expect("a base unit");
        let s = self.c.session.as_mut().unwrap();
        s.play.select_box(&[mcv], false);
        self.c.key(Key::Char('d'), true);
        self.c.key(Key::Char('d'), false);
        self.ticks(200);
    }

    /// Centres the view on `cell`.
    fn look_at(&mut self, cell: CellXY) {
        let s = self.c.session.as_mut().unwrap();
        let (x, y) = glasnost_render::projection::cell_centre(cell.x, cell.y);
        s.set_centre(glasnost_render::projection::project_lifted(x, y, 0));
    }

    /// A building's running slot animations (S-0108) with their frames.
    fn slot_list(&self, id: ObjectId) -> Vec<String> {
        let s = self.c.session.as_ref().unwrap();
        s.loaded.live.as_ref().expect("a live game").slot_names(id)
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

    /// A player house that is not allied with the local house.
    fn enemy(&self) -> Option<HouseId> {
        let w = self.world();
        let local = self.local();
        (0..w.houses.len()).map(|i| HouseId(i as u8)).find(|&h| {
            h != local
                && !w.allied(Some(local), Some(h))
                && w.houses[usize::from(h.0)].role == glasnost_sim::world::HouseRole::Player
        })
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

    /// The indices in the drawn scene of the items of `id`.
    fn items_of(&self, id: ObjectId) -> Vec<usize> {
        let code = sim_code(id);
        self.c
            .session
            .as_ref()
            .unwrap()
            .scene
            .items
            .iter()
            .enumerate()
            .filter(|(_, it)| it.object == Some(code))
            .map(|(n, _)| n)
            .collect()
    }

    /// The screen point of the first body item of `id` (not a shadow).
    fn body_point(&self, id: ObjectId) -> Option<(i32, i32)> {
        let code = sim_code(id);
        self.c
            .session
            .as_ref()
            .unwrap()
            .scene
            .items
            .iter()
            .find(|it| {
                it.object == Some(code) && it.group != Group::Shadows && it.paint != Paint::Shadow
            })
            .map(|it| (it.x, it.y))
    }

    /// Prints how the items of building `b` stand against those of the
    /// unit `u` in the drawing order.
    fn order_report(&self, name: &str, b: ObjectId, u: ObjectId) {
        let bi = self.items_of(b);
        let ui = self.items_of(u);
        let first = ui.first().copied().unwrap_or(usize::MAX);
        let last = ui.last().copied().unwrap_or(0);
        println!(
            "{name}: building items {}, unit items {}; building items after the unit's first {}, \
             after its last {}",
            bi.len(),
            ui.len(),
            bi.iter().filter(|&&i| i > first).count(),
            bi.iter().filter(|&&i| i > last).count(),
        );
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

    /// Writes `name-zoom.png`: the `cw` × `ch` pixels around the mean
    /// screen point of `obj`'s pixels, enlarged `k` times.
    fn zoom(&mut self, name: &str, obj: ObjectId, (cw, ch, k): (i32, i32, i32)) {
        let _ = self.c.frame();
        let at = {
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
            if n == 0 {
                println!("{name}: the object is not under any screen point");
                return;
            }
            Point::new((sx / n) as i32, (sy / n) as i32)
        };
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
}
