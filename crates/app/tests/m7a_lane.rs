//! Local lane for the M7a features (never in CI): with `GLASNOST_GAME_DIR`
//! set, every scene of [`SCENES`] starts its own skirmish through the
//! windowless client, puts the objects it needs on the map by their rules
//! flags (never by type name), plays the scene and writes its screens to
//! `local-lane/m7a/` (git-ignored; `GLASNOST_LANE_OUT` overrides the
//! folder). Prints counts, cells and ticks only, never type names or rules
//! values.
//!
//! Scenes now (S-0610, psychic weapons): a unit with a berserk weapon turns
//! an enemy vehicle berserk among its fellows; a psychic sensor shows the
//! hostile-intent line of an enemy heading into its reach; a psychic reveal
//! uncovers an area of a shrouded map. S-0601 (mind control): a
//! controller takes an enemy vehicle over (the link line and the ring),
//! then a second, and its death releases both; an unlimited controller
//! taken past its overload counts pulses.
//!
//! Run with `cargo test --release -p glasnost-app --test m7a_lane --
//! --ignored --nocapture`. `GLASNOST_LANE_SCENE=a,b` plays only the scenes
//! with those keys; `GLASNOST_UI_MAP=n` picks the n-th offered map.

#![cfg(not(target_arch = "wasm32"))]
#![allow(dead_code)]

use std::rc::Rc;

use glasnost_app::game::client::{Client, run_job};
use glasnost_app::mapview::png;
use glasnost_map::CellXY;
use glasnost_map::terrain::land;
use glasnost_sim::rules::Kind;
use glasnost_sim::targeting::{self, ClickOn, Modifiers};
use glasnost_sim::world::{HouseId, Layer, ObjectId, Target, World};
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
    ("berserk", berserk_spreads),
    ("sensor", sensor_shows_intent),
    ("reveal", psychic_reveal),
    ("mind", mind_control),
    ("overload", mind_overload),
];

#[test]
#[ignore = "local lane: needs GLASNOST_GAME_DIR"]
fn m7a_scenes() {
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

/// An armed vehicle type that is neither psionics-immune nor a harvester,
/// for the enemy group.
fn plain_vehicle(w: &World) -> Option<String> {
    let r = w.rules();
    (0..r.vehicles.len())
        .find(|&i| {
            r.unit(Kind::Vehicle, i as u32).is_some_and(|u| {
                u.combat.slots[0].is_some()
                    && !u.harvester
                    && u.strength > 0
                    && !u.combat.immune_psionics
                    && u.passengers == 0
            })
        })
        .map(|i| r.vehicles[i].name.clone())
}

/// S-0610 claims 3 to 6: a unit whose first weapon has a berserk-causing
/// warhead is ordered (by the click a person gives) at one of three enemy
/// vehicles standing together; the hit vehicle goes berserk, turns on its
/// fellows and ends berserk when its time runs out.
fn berserk_spreads(lane: &mut Lane) {
    let w = lane.world();
    let r = w.rules();
    let shooter = (0..r.infantry.len())
        .map(|i| (Kind::Infantry, i as u32))
        .chain((0..r.vehicles.len()).map(|i| (Kind::Vehicle, i as u32)))
        .find(|&(k, i)| {
            r.unit(k, i).is_some_and(|u| {
                u.combat.slots[0]
                    .and_then(|p| r.combat.weapon(p).warhead)
                    .is_some_and(|h| r.combat.warhead(h).psychedelic)
            })
        })
        .map(|(k, i)| (k, r.unit(k, i).map(|u| u.name.clone()).unwrap_or_default()));
    let victim = plain_vehicle(w);
    println!(
        "berserk-weapon type found: {}, enemy vehicle type found: {}",
        shooter.is_some(),
        victim.is_some()
    );
    let (Some((kind, sname)), Some(vname)) = (shooter, victim) else {
        return;
    };
    let local = lane.local();
    let Some(enemy) = lane.enemy() else { return };
    let home = lane.home();
    let spot = lane.open_land(CellXY::new(home.x + 8, home.y + 8));
    let s = lane.spawn(kind, &sname, local, CellXY::new(spot.x - 3, spot.y));
    let vs: Vec<ObjectId> = [(1, 0), (2, 0), (1, 1)]
        .iter()
        .map(|&(dx, dy)| {
            lane.spawn(
                Kind::Vehicle,
                &vname,
                enemy,
                CellXY::new(spot.x + dx, spot.y + dy),
            )
        })
        .collect();
    lane.look_at(spot);
    lane.ticks(4);
    lane.save("berserk-before");
    let sent = lane.order(local, s, ClickOn::Object(vs[0]));
    println!("click on the target: {sent} orders");
    let mut went = None;
    let mut shot = false;
    for t in 0..3000 {
        lane.ticks(1);
        let w = lane.world();
        let mad: Vec<ObjectId> = vs
            .iter()
            .copied()
            .filter(|&v| glasnost_sim::psychic::berserk(w, v))
            .collect();
        if went.is_none() && !mad.is_empty() {
            went = Some(t);
            println!("first enemy berserk at tick {t}; berserk now {}", mad.len());
            // A person stops the shooter so the berserk vehicle can be watched.
            let _ = lane.order(local, s, ClickOn::Cell(CellXY::new(spot.x - 6, spot.y)));
        }
        if let Some(g) = went
            && !shot
            && t >= g + 20
        {
            shot = true;
            let w = lane.world();
            let targets = vs
                .iter()
                .filter(|&&v| {
                    w.obj(v).is_some_and(|o| {
                        matches!(o.target, Some(Target::Object(x)) if w.obj(x).is_some_and(|xo| xo.owner == Some(enemy)))
                    })
                })
                .count();
            println!("20 ticks later: enemy vehicles aiming at their own side {targets}");
            lane.save("berserk-turned");
        }
        if went.is_some()
            && vs
                .iter()
                .all(|&v| !glasnost_sim::psychic::berserk(lane.world(), v))
        {
            println!("berserk over at tick {t}");
            break;
        }
    }
    lane.ticks(3);
    lane.save("berserk-after");
}

/// S-0610 claims 9 to 12: a psychic sensor (a building type with a
/// psychic-detection radius) is placed for the local house; an enemy
/// vehicle far outside its reach is sent to a cell inside it; the
/// hostile-intent line is drawn with its end squares.
fn sensor_shows_intent(lane: &mut Lane) {
    let w = lane.world();
    let r = w.rules();
    let sensor = (0..r.buildings.len())
        .filter(|&b| {
            r.buildings[b]
                .raw
                .econ
                .psychic_detection_radius
                .unwrap_or(0)
                > 0
        })
        .min_by_key(|&b| (r.buildings[b].foundation.0 * r.buildings[b].foundation.1, b))
        .map(|b| (r.buildings[b].name.clone(), r.buildings[b].foundation));
    let mover = plain_vehicle(w);
    println!(
        "sensor type found: {}, enemy vehicle type found: {}",
        sensor.is_some(),
        mover.is_some()
    );
    let (Some((bname, size)), Some(vname)) = (sensor, mover) else {
        return;
    };
    let local = lane.local();
    let Some(enemy) = lane.enemy() else { return };
    let home = lane.home();
    let Some(site) = lane.area(CellXY::new(home.x + 6, home.y + 6), size, 2, |l| {
        l != land::WATER && l != land::ROCK
    }) else {
        println!("no open ground");
        return;
    };
    let b = lane.spawn(Kind::Building, &bname, local, site);
    // Power for the sensor: the local house's strongest power plant type, beside it.
    let plant = {
        let r = lane.world().rules();
        (0..r.buildings.len())
            .filter(|&p| r.buildings[p].raw.econ.power.unwrap_or(0) > 0)
            .max_by_key(|&p| {
                (
                    r.buildings[p].raw.econ.power.unwrap_or(0),
                    std::cmp::Reverse(p),
                )
            })
            .map(|p| (r.buildings[p].name.clone(), r.buildings[p].foundation))
    };
    if let Some((pname, psize)) = plant
        && let Some(at) = lane.area(CellXY::new(site.x - 4, site.y), psize, 1, |l| {
            l != land::WATER && l != land::ROCK
        })
    {
        lane.spawn(Kind::Building, &pname, local, at);
    }
    lane.ticks(2);
    let radius = glasnost_sim::psychic::sensor_radius(lane.world(), b);
    println!(
        "sensor placed at {site:?}; in operation {}; shows a ring {}",
        glasnost_sim::econ::power::in_operation(lane.world(), b),
        glasnost_sim::psychic::range_ring(lane.world(), b).is_some()
    );
    let reach = radius.unwrap_or(0);
    let far = lane.open_land(CellXY::new(site.x + reach + 6, site.y + 2));
    let v = lane.spawn(Kind::Vehicle, &vname, enemy, far);
    let goal = lane.open_land(CellXY::new(site.x + 1, site.y + size.1 + 1));
    let w = lane.world();
    let r = targeting::click(w, enemy, &[v], ClickOn::Cell(goal), Modifiers::default());
    let mut n = 0;
    for o in r.orders {
        n += usize::from(glasnost_sim::orders::issue(lane.world_mut(), enemy, o));
    }
    println!("enemy move order: {n}");
    lane.look_at(CellXY::new((site.x + far.x) / 2, (site.y + far.y) / 2));
    lane.ticks(3);
    let lines = glasnost_sim::psychic::intent_lines(lane.world(), local).len();
    println!(
        "intent lines for the local house: {lines}; the enemy vehicle has a destination {}",
        lane.world().obj(v).is_some_and(|o| o.dest.is_some())
    );
    lane.save("sensor-intent");
    lane.ticks(40);
    let lines = glasnost_sim::psychic::intent_lines(lane.world(), local).len();
    println!("40 ticks later: intent lines {lines}");
    lane.save("sensor-intent-later");
}

/// S-0610 claims 13 to 15: the local house's map is shrouded again; the
/// psychic reveal's effect (with the install's radius) is made at a cell
/// near an enemy start; the area is explored with no lasting sight.
fn psychic_reveal(lane: &mut Lane) {
    let local = lane.local();
    let w = lane.world();
    let sw = w
        .rules()
        .superweapons
        .types
        .iter()
        .position(|t| t.kind == Some(glasnost_sim::superweapons::SwKind::PsychicReveal));
    println!("psychic reveal type found: {}", sw.is_some());
    let Some(i) = sw else { return };
    let target = lane
        .enemy()
        .and_then(|e| {
            let w = lane.world();
            w.objects
                .values()
                .find(|o| o.on_map && o.owner == Some(e))
                .map(|o| o.pos.cell())
        })
        .unwrap_or_else(|| {
            let h = lane.home();
            CellXY::new(h.x + 12, h.y + 12)
        });
    glasnost_sim::econ::shroud::crate_darkness(lane.world_mut(), local);
    lane.ticks(2);
    lane.look_at(target);
    lane.save("reveal-before");
    glasnost_sim::psychic::reveal(lane.world_mut(), local, i as u32, target);
    let explored = |w: &World| {
        w.cells
            .grid
            .iter()
            .filter(|(c, _)| glasnost_sim::econ::shroud::explored(w, local, *c))
            .count()
    };
    println!(
        "explored cells after the reveal: {}",
        explored(lane.world())
    );
    lane.ticks(2);
    lane.save("reveal-after");
    lane.ticks(60);
    println!("explored cells 60 ticks later: {}", explored(lane.world()));
}

/// The first unit type (infantry, then vehicles) whose primary weapon has a
/// takeover warhead and whose unlimited-control flag is `unlimited`.
fn controller_type(w: &World, unlimited: bool) -> Option<(Kind, String)> {
    let r = w.rules();
    (0..r.infantry.len())
        .map(|i| (Kind::Infantry, i as u32))
        .chain((0..r.vehicles.len()).map(|i| (Kind::Vehicle, i as u32)))
        .find(|&(k, i)| {
            r.unit(k, i).is_some_and(|u| {
                u.combat.slots[0].is_some_and(|p| {
                    let wr = r.combat.weapon(p);
                    wr.infinite_mind_control == unlimited
                        && wr.warhead.is_some_and(|h| r.combat.warhead(h).mind_control)
                })
            })
        })
        .map(|(k, i)| (k, r.unit(k, i).map(|u| u.name.clone()).unwrap_or_default()))
}

/// S-0601 claims 1 to 7, 9, 17 and 18: a controller of the local house is
/// ordered (by the click a person gives) at an enemy vehicle; the vehicle
/// changes sides and shows its ring; a second enemy vehicle is put beside
/// it and taken (a capacity-1 controller lets the first go, claim 7); with
/// the controller selected the link line is drawn; the controller is
/// removed and its victim goes back to its owner.
fn mind_control(lane: &mut Lane) {
    let w = lane.world();
    let ctrl = controller_type(w, false);
    let victim = plain_vehicle(w);
    println!(
        "controller type found: {}, enemy vehicle type found: {}",
        ctrl.is_some(),
        victim.is_some()
    );
    let (Some((kind, cname)), Some(vname)) = (ctrl, victim) else {
        return;
    };
    let local = lane.local();
    let Some(enemy) = lane.enemy() else { return };
    let home = lane.home();
    let spot = lane.open_land(CellXY::new(home.x + 8, home.y + 8));
    let c = lane.spawn(kind, &cname, local, CellXY::new(spot.x - 3, spot.y));
    // One enemy vehicle at a time, so the controller lives to take the next.
    let mut vs: Vec<ObjectId> = vec![lane.spawn(
        Kind::Vehicle,
        &vname,
        enemy,
        CellXY::new(spot.x + 1, spot.y),
    )];
    let rec = glasnost_sim::mind::controller(lane.world(), c).map(|r| (r.capacity, r.unlimited));
    println!("controller record (capacity, unlimited): {rec:?}");
    lane.look_at(spot);
    lane.ticks(4);
    lane.save("mind-before");
    let mut taken = Vec::new();
    for n in 0..2 {
        if n == 1 {
            let v = lane.spawn(
                Kind::Vehicle,
                &vname,
                enemy,
                CellXY::new(spot.x + 1, spot.y + 2),
            );
            vs.push(v);
        }
        let v = vs[n];
        let sent = lane.order(local, c, ClickOn::Object(v));
        println!("click on enemy vehicle {n}: {sent} orders");
        for t in 0..600 {
            lane.ticks(1);
            if glasnost_sim::mind::controlled_by(lane.world(), v) == Some(c) {
                println!(
                    "vehicle {n} taken after {t} ticks; owner is the local house {}",
                    lane.world().obj(v).is_some_and(|o| o.owner == Some(local))
                );
                taken.push(v);
                break;
            }
        }
        lane.ticks(3);
        if let Some(at) = lane.world().obj(v).map(|o| o.pos.cell()) {
            lane.look_at(at);
            lane.ticks(1);
        }
        if n == 0 {
            lane.save("mind-taken");
        } else {
            lane.save("mind-second");
        }
    }
    println!(
        "controller alive {} (health per mille {})",
        lane.world().obj(c).is_some(),
        lane.permille(c)
    );
    for (n, &v) in vs.iter().enumerate() {
        let w = lane.world();
        println!(
            "vehicle {n}: exists {}, local {}, controlled by the controller {}",
            w.obj(v).is_some(),
            w.obj(v).is_some_and(|o| o.owner == Some(local)),
            glasnost_sim::mind::controlled_by(w, v) == Some(c)
        );
    }
    let lines = glasnost_sim::mind::link_lines(lane.world()).len();
    let rings = vs
        .iter()
        .filter(|&&v| glasnost_sim::mind::ring(lane.world(), v).is_some())
        .count();
    println!(
        "link lines {lines}; rings {rings}; taken in turn {}",
        taken.len()
    );
    lane.c.session.as_mut().unwrap().play.selection = vec![c];
    if let Some(at) = lane.world().obj(c).map(|o| o.pos.cell()) {
        lane.look_at(at);
    }
    lane.ticks(1);
    lane.save("mind-link-selected");
    glasnost_sim::objects::remove(lane.world_mut(), c);
    lane.ticks(3);
    let back = taken
        .iter()
        .filter(|&&v| lane.world().obj(v).is_some_and(|o| o.owner == Some(enemy)))
        .count();
    println!(
        "controller removed: {back} of {} back with their owner",
        taken.len()
    );
    lane.save("mind-released");
}

/// S-0601 claim 16: an unlimited controller is given (by the takeover
/// itself) more victims than its overload counts allow; the pulses are
/// counted and its health followed.
fn mind_overload(lane: &mut Lane) {
    let w = lane.world();
    let ctrl = controller_type(w, true);
    let victim = plain_vehicle(w);
    println!(
        "unlimited controller type found: {}, enemy vehicle type found: {}",
        ctrl.is_some(),
        victim.is_some()
    );
    let (Some((kind, cname)), Some(vname)) = (ctrl, victim) else {
        return;
    };
    let local = lane.local();
    let Some(enemy) = lane.enemy() else { return };
    let home = lane.home();
    let spot = lane.open_land(CellXY::new(home.x + 8, home.y + 8));
    let c = lane.spawn(kind, &cname, local, CellXY::new(spot.x - 4, spot.y));
    let counts = lane
        .world()
        .rules()
        .combat
        .general
        .mind
        .overload_count
        .clone();
    let n = counts.iter().copied().max().unwrap_or(0).max(0) as usize + 2;
    let n = n.min(12);
    let mut held = 0;
    for i in 0..n {
        let at = lane.open_land(CellXY::new(
            spot.x + (i % 4) as i32 * 2,
            spot.y + (i / 4) as i32 * 2,
        ));
        let v = lane.spawn(Kind::Vehicle, &vname, enemy, at);
        held += usize::from(glasnost_sim::mind::take(lane.world_mut(), c, v));
    }
    println!(
        "victims taken directly: {held} (overload counts list length {})",
        counts.len()
    );
    lane.c.session.as_mut().unwrap().play.selection = vec![c];
    lane.look_at(spot);
    let start = lane.permille(c);
    lane.ticks(2);
    lane.save("mind-overload");
    let mut low = start;
    for _ in 0..300 {
        lane.ticks(1);
        if lane.world().obj(c).is_none() {
            println!(
                "the controller died of overload at tick {}",
                lane.world().tick
            );
            break;
        }
        low = low.min(lane.permille(c));
    }
    println!("controller health per mille: start {start}, lowest {low}");
    lane.save("mind-overload-later");
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
        Lane::start_on(env_num("GLASNOST_UI_MAP", 0))
    }

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
    /// `fits`, chosen by content, never by name. Prints the map's index.
    fn start_where(fits: impl Fn(&World) -> bool) -> Option<Lane> {
        for i in 0..Lane::map_count() {
            let lane = Lane::start_on(i)?;
            if fits(lane.world()) {
                println!("map index {i} fits");
                return Some(lane);
            }
        }
        None
    }

    /// Starts a skirmish on the `map`-th offered map.
    fn start_on(map: usize) -> Option<Lane> {
        let dir = std::env::var_os("GLASNOST_GAME_DIR")?;
        let out = std::env::var_os("GLASNOST_LANE_OUT")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| {
                std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../local-lane/m7a")
            });
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
            "wrote {name}.png (tick {}): animations created {made}, \
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
