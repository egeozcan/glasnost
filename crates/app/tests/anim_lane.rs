//! Local lane for animation playback (S-0109, never in CI): with
//! `GLASNOST_GAME_DIR` set, starts a skirmish through the windowless
//! client, re-shoots the V3-rocket and ship-sinking scenes of the M6a lane
//! with the animations played from their art, and writes the screens to
//! `local-lane/m6b/` (git-ignored). Prints counts, cells and ticks only,
//! never type names.
//!
//! Run with `cargo test --release -p glasnost-app --test anim_lane --
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
        let out = png::lane_out("m6b");
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
            "wrote {}/{name}.png (tick {}): animations created {made}, \
             with art {art}, without sprite {none}, drawn now {drawn}",
            self.dir.display(),
            self.world().tick
        );
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
        // GLASNOST_TRACE_ANIMS=1: the animations playing in this shot, by name and world pixel.
        if std::env::var_os("GLASNOST_TRACE_ANIMS").is_some() {
            let s = self.c.session.as_ref().unwrap();
            let live = s.loaded.live.as_ref().expect("a live game");
            for (n, (x, y, z)) in live.playing_anims() {
                let (wx, wy) = glasnost_render::projection::project_lifted(x, y, z);
                println!(
                    "  anim {n} at world pixel ({wx}, {wy}), position ({x}, {y}, {z}), surface {}",
                    live.world.surface_at(x, y)
                );
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

    /// An object of `h` at cell `c`, or else the nearest one of `h`.
    fn obj_near(&self, c: CellXY, h: HouseId) -> Option<ObjectId> {
        self.world()
            .objects
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
fn anim_scenes() {
    let Some(mut lane) = Lane::start() else {
        eprintln!("GLASNOST_GAME_DIR not set: skipped");
        return;
    };
    let local = lane.local();
    let enemy = lane.enemy();
    let target = lane.enemy_cell();
    println!("enemy object at cell {target:?}");

    // The V3 scene (S-0506): a vehicle whose spawner launches a rocket type.
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
        let e = lane.obj_near(target, enemy);
        let issued = e.map_or(0, |e| lane.attack(local, id, ClickOn::Object(e)));
        println!("launcher at {at:?}, {issued} orders");
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
                .map(|o| (o.pos.cell(), o.pos.z))
        };
        let mut seen = 0;
        let mut last = None;
        for _ in 0..2000 {
            lane.ticks(1);
            match flight(lane.world()) {
                Some((c, z)) => {
                    last = Some(c);
                    seen += 1;
                    // The M6a lane's second shot, 31 ticks after the first sighting.
                    if seen == 32 {
                        println!("rocket in flight at {c:?}, height {z}");
                        lane.look_at(c);
                        lane.ticks(1);
                        lane.save("v3-rocket-anims");
                    }
                }
                None if seen > 0 => {
                    let w = lane.world();
                    let gone: Vec<_> = w
                        .objects
                        .values()
                        .filter(|o| o.rocket.is_some())
                        .map(|o| (o.pos, o.rocket.as_ref().map(|r| r.phase)))
                        .collect();
                    println!("rocket down near {last:?}; rocket objects left: {gone:?}");
                    if let Some(c) = last {
                        let (x, y) = glasnost_render::projection::cell_centre(c.x, c.y);
                        let centre = glasnost_render::projection::project_lifted(x, y, 0);
                        println!("camera centre world pixel {centre:?}");
                    }
                    // The rocket has come down: its detonation's animations play.
                    if let Some(c) = last {
                        lane.look_at(c);
                    }
                    // The pictures 1, 4 and 12 ticks after the detonation.
                    for (wait, name) in [
                        (1, "v3-rocket-impact-anims-1"),
                        (3, "v3-rocket-impact-anims"),
                        (8, "v3-rocket-impact-anims-12"),
                    ] {
                        lane.ticks(wait);
                        lane.save(name);
                    }
                    break;
                }
                None => {}
            }
        }
        println!("rocket ticks in flight: {seen}");
    }

    // The ship scene (S-0504): ships on open water, one sinking.
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
        let ids: Vec<ObjectId> = (0..3)
            .map(|i| lane.spawn(Kind::Vehicle, &sn, local, CellXY::new(sea.x - 1 + i, sea.y)))
            .collect();
        let dest = lane.open(CellXY::new(sea.x + 12, sea.y + 12), |l| l == land::WATER);
        for &id in &ids {
            lane.attack(local, id, ClickOn::Cell(dest));
        }
        lane.ticks(40);
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
        lane.ticks(30);
        if let Some(c) = lane.world().obj(ids[0]).map(|o| o.pos.cell()) {
            lane.look_at(c);
            lane.ticks(1);
            lane.save("ship-sinking-anims");
        }
    }
    let (made, art, none, _) = lane.counts();
    println!("animations created {made}, with art {art}, without sprite {none}");
}
