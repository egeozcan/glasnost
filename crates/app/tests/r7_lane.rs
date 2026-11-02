//! Local lane for the M8 A4 round 7 notes (never in CI): with
//! `GLASNOST_GAME_DIR` set, starts a skirmish through the windowless client
//! and plays note 40 (an aircraft shot down in flight), note 41 (the
//! sidebar's mode and tab buttons), note 42 (a factory's rally point) and
//! note 43 (the power meter at full power and short of power). Each screen
//! drawn on the CPU goes to `local-lane/m8/a4-r7/<tag>/` (git-ignored; the
//! tag is `GLASNOST_R7_TAG`, default `after`; `GLASNOST_LANE_OUT` moves
//! it). Types are chosen by their fields, never by name (the damage smoke
//! by the engine's fixed name, S-0501 Import mapping); it prints counts,
//! cells, ticks and booleans only. `intake_67_a_damaged_and_a_crashing_aircraft`
//! (M8 A3 intake 67) writes to `local-lane/m8/intake67/<tag>/` instead.
//!
//! Run with `cargo test --release -p glasnost-app --test r7_lane --
//! --ignored --nocapture --test-threads 1`.

#![cfg(not(target_arch = "wasm32"))]

use std::rc::Rc;

use glasnost_app::game::client::{Client, Key, run_job};
use glasnost_app::game::play::Button;
use glasnost_app::mapview::png;
use glasnost_map::CellXY;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::world::{ObjectId, Target, World};
use glasnost_ui::geom::{Point, Rect};
use glasnost_ui::hud::HudHit;
use glasnost_ui::menu::MenuHit;
use glasnost_ui::modes::Mode;
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

fn run_jobs(c: &mut Client) {
    while let Some(job) = c.job.take() {
        let done = pollster::block_on(run_job(job));
        c.done(done);
    }
}

fn click_at(c: &mut Client, p: Point) {
    let _ = c.frame();
    c.pointer_moved(p);
    c.pointer_down(Button::Left, p);
    c.pointer_up(Button::Left, p, Default::default());
}

fn click(c: &mut Client, r: Rect) {
    click_at(c, Point::new(r.x + r.w / 2, r.y + r.h / 2));
}

struct Shots {
    dir: std::path::PathBuf,
}

impl Shots {
    fn save(&self, c: &mut Client, name: &str) {
        let rgba = c.frame_rgba();
        let (w, h) = c.size;
        let path = self.dir.join(format!("{name}.png"));
        std::fs::write(&path, png::png(w as u32, h as u32, &rgba)).unwrap();
        println!("wrote {name}.png");
    }

    /// Writes `name-zoom.png`: the `cw` × `ch` pixels around `at`,
    /// enlarged `k` times.
    fn zoom(&self, c: &mut Client, name: &str, at: Point, (cw, ch, k): (i32, i32, i32)) {
        let rgba = c.frame_rgba();
        let (w, h) = c.size;
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
        let path = self.dir.join(format!("{name}-zoom.png"));
        std::fs::write(&path, png::png((cw * k) as u32, (ch * k) as u32, &out)).unwrap();
        println!("wrote {name}-zoom.png");
    }
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

/// Starts a skirmish on map 0 from the menu on a window of `size`, every
/// player house allied (lobby team 1) so the game plays on undisturbed.
fn start(size: (i32, i32)) -> Option<(Client, Shots)> {
    start_in(size, "m8/a4-r7")
}

/// [`start`] with its screens written under `local-lane/<sub>/<tag>/`.
fn start_in(size: (i32, i32), sub: &str) -> Option<(Client, Shots)> {
    let dir = std::env::var_os("GLASNOST_GAME_DIR")?;
    let tag = std::env::var("GLASNOST_R7_TAG").unwrap_or_else(|_| "after".into());
    let out = png::lane_out(&format!("{sub}/{tag}"));
    std::fs::create_dir_all(&out).unwrap();
    let mut c = Client::new();
    c.resize(size);
    c.mount(files(std::path::Path::new(&dir)));
    run_jobs(&mut c);
    let _ = c.frame();
    let r = c.menu_targets.rect_of(&MenuHit::Skirmish).unwrap();
    click(&mut c, r);
    for slot in 0..2 {
        let _ = c.frame();
        // spec: S-0412 claim 15 (no team, then teams 1 … 4: one step gives team 1)
        let r = c.setup_targets.rect_of(&SetupHit::Team(slot, 1)).unwrap();
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
    Some((c, Shots { dir: out }))
}

/// Centres the view on `cell`.
fn look_at(c: &mut Client, cell: CellXY) {
    let s = c.session.as_mut().unwrap();
    let (x, y) = glasnost_render::projection::cell_centre(cell.x, cell.y);
    s.set_centre(glasnost_render::projection::project_lifted(x, y, 0));
}

/// The screen point of the centre of `cell`, after a frame.
fn screen_of(c: &mut Client, cell: CellXY) -> Option<Point> {
    let _ = c.frame();
    let s = c.session.as_mut().unwrap();
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

/// The mean screen point of the pixels that hit object `obj`.
fn centre_of(c: &mut Client, obj: ObjectId) -> Option<Point> {
    let _ = c.frame();
    let s = c.session.as_mut().unwrap();
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

/// The local base unit, deployed where it stands; returns its cell.
fn deploy(c: &mut Client, clock: &mut i64) -> CellXY {
    ticks(c, clock, 5);
    let s = c.session.as_mut().unwrap();
    let local = s.play.me();
    let mcv = s
        .play
        .world()
        .objects
        .values()
        .find(|o| o.owner == Some(local) && o.kind == Kind::Vehicle && o.on_map)
        .map(|o| (o.id, o.pos.cell()))
        .expect("a base unit");
    s.play.select_box(&[mcv.0], false);
    c.key(Key::Char('d'), true);
    c.key(Key::Char('d'), false);
    ticks(c, clock, 200);
    mcv.1
}

fn world(c: &Client) -> &World {
    c.session.as_ref().unwrap().play.world()
}

fn world_mut(c: &mut Client) -> &mut World {
    &mut c.session.as_mut().unwrap().play.game.world
}

/// The body images the scene draws for `id` (its items other than shadows).
fn body_images(c: &Client, id: ObjectId) -> Vec<u32> {
    let code = glasnost_app::mapview::live::sim_code(id);
    c.session
        .as_ref()
        .unwrap()
        .scene
        .items
        .iter()
        .filter(|it| it.object == Some(code) && it.group != glasnost_render::scene::Group::Shadows)
        .map(|it| it.image)
        .collect()
}

/// Note 40: an aircraft of the first flying type with a voxel body is sent
/// across the map, shot down once it flies at height, and followed tick by
/// tick until it hits the ground: its height, destination, position and
/// the body images drawn.
#[test]
#[ignore = "local lane: needs GLASNOST_GAME_DIR"]
fn note_40_an_aircraft_shot_down() {
    let Some((mut c, shots)) = start((1280, 800)) else {
        eprintln!("GLASNOST_GAME_DIR not set: skipped");
        return;
    };
    let mut clock = 0;
    let home = deploy(&mut c, &mut clock);
    let local = c.session.as_ref().unwrap().play.me();
    let w = world(&c);
    let r = w.rules_arc();
    let picked = (0..r.aircraft.len()).find(|&i| {
        let u = &r.aircraft[i];
        let voxel = c
            .session
            .as_ref()
            .unwrap()
            .loaded
            .import
            .arts
            .get(&(glasnost_convert::kinds::Kind::Aircraft, i))
            .is_some_and(|a| a.voxels.contains_key("body"));
        voxel && u.type_locomotor() == glasnost_rules::sim::Locomotor::Fly
    });
    let Some(ti) = picked else {
        println!("no flying aircraft type with a voxel body");
        return;
    };
    println!("aircraft type {ti}");
    let at = CellXY::new(home.x + 6, home.y + 6);
    let a = {
        let w = world_mut(&mut c);
        let name = w.rules().aircraft[ti].name.clone();
        glasnost_sim::testkit::spawn(w, Kind::Aircraft, &name, local.0, at, 0)
    };
    let dest = CellXY::new(home.x - 10, home.y + 6);
    let n = c
        .session
        .as_mut()
        .unwrap()
        .play
        .issue(vec![glasnost_sim::orders::Order::Behaviour {
            actor: a,
            behaviour: Behaviour::Move,
            attack_move: false,
            target: None,
            dest: Some(Target::Cell(dest)),
        }]);
    println!("move orders taken {n}");
    let mut up = 0;
    let x0 = world(&c).obj(a).unwrap().pos.x;
    for t in 0..600 {
        ticks(&mut c, &mut clock, 1);
        let w = world(&c);
        let moved = (w.obj(a).unwrap().pos.x - x0).abs() >= 3 * 256;
        if glasnost_sim::air::height(w, a) >= 400 && moved {
            up = t;
            break;
        }
    }
    let h = glasnost_sim::air::height(world(&c), a);
    println!("airborne after {up} ticks at height {h}");
    let cell = world(&c).obj(a).unwrap().pos.cell();
    look_at(&mut c, cell);
    if let Some(p) = centre_of(&mut c, a) {
        shots.save(&mut c, "n40-0-flying");
        shots.zoom(&mut c, "n40-0-flying", p, (120, 120, 4));
    }
    let killed_at = world(&c).tick;
    let res = glasnost_sim::combat::damage::apply(
        world_mut(&mut c),
        a,
        glasnost_sim::combat::damage::Hit {
            amount: 1_000_000,
            warhead: None,
            distance: 0,
            attacker: None,
            house: None,
            ignore_defences: true,
            no_escape: false,
        },
    );
    println!(
        "killed at tick {killed_at}: {res:?}; crashing {}",
        glasnost_sim::air::crashing(world(&c), a)
    );
    let mut images = std::collections::BTreeSet::new();
    let mut changes = 0;
    let mut last: Option<Vec<u32>> = None;
    for t in 0..200 {
        ticks(&mut c, &mut clock, 1);
        let Some(o) = world(&c).obj(a) else {
            println!("tick +{t}: left play");
            break;
        };
        let (cell, pos, dest) = (o.pos.cell(), o.pos, o.dest);
        let h = glasnost_sim::air::height(world(&c), a);
        let fall = o.air.as_ref().and_then(|x| x.crash);
        println!(
            "tick +{t}: height {h}, fall counter {fall:?}, x {} y {}, destination {dest:?}",
            pos.x, pos.y
        );
        look_at(&mut c, cell);
        let Some(p) = centre_of(&mut c, a) else {
            continue;
        };
        let drawn = body_images(&c, a);
        images.extend(drawn.iter().copied());
        if last.as_ref().is_some_and(|l| *l != drawn) {
            changes += 1;
        }
        last = Some(drawn);
        if t % 3 == 0 {
            shots.zoom(&mut c, &format!("n40-1-falling-t{t:02}"), p, (120, 120, 4));
        }
        if t == 6 {
            shots.save(&mut c, "n40-1-falling");
        }
    }
    println!(
        "falling: body image changes {changes}, distinct body images {}",
        images.len()
    );
}

/// The damage-smoke animations in play (S-0501 claim 37, the engine's
/// fixed type name): their ids.
fn smoke_ids(c: &Client) -> Vec<ObjectId> {
    let w = world(c);
    let r = w.rules_arc();
    w.anims
        .anims
        .iter()
        .filter(|(_, a)| {
            a.anim
                .and_then(|i| r.animations.get(i as usize))
                .is_some_and(|t| t.name.eq_ignore_ascii_case(glasnost_sim::air::DAMAGE_SMOKE))
        })
        .map(|(id, _)| *id)
        .collect()
}

/// M8 A3 intake 67 (note 40 again, S-0501 r65 and S-0113 r17): an aircraft
/// of the note 40 type, damaged below the smoke threshold in flight until
/// it has shown smoke, then shot down and followed until it leaves play.
/// Screens go to `local-lane/m8/intake67/<tag>/`.
#[test]
#[ignore = "local lane: needs GLASNOST_GAME_DIR"]
fn intake_67_a_damaged_and_a_crashing_aircraft() {
    let Some((mut c, shots)) = start_in((1280, 800), "m8/intake67") else {
        eprintln!("GLASNOST_GAME_DIR not set: skipped");
        return;
    };
    let mut clock = 0;
    let home = deploy(&mut c, &mut clock);
    let local = c.session.as_ref().unwrap().play.me();
    let r = world(&c).rules_arc();
    let picked = (0..r.aircraft.len()).find(|&i| {
        let voxel = c
            .session
            .as_ref()
            .unwrap()
            .loaded
            .import
            .arts
            .get(&(glasnost_convert::kinds::Kind::Aircraft, i))
            .is_some_and(|a| a.voxels.contains_key("body"));
        voxel && r.aircraft[i].type_locomotor() == glasnost_rules::sim::Locomotor::Fly
    });
    let Some(ti) = picked else {
        println!("no flying aircraft type with a voxel body");
        return;
    };
    let smoke_type = r
        .animations
        .iter()
        .any(|t| t.name.eq_ignore_ascii_case(glasnost_sim::air::DAMAGE_SMOKE));
    println!("aircraft type {ti}; the damage-smoke type is registered: {smoke_type}");
    let at = CellXY::new(home.x + 8, home.y + 8);
    let a = {
        let w = world_mut(&mut c);
        let name = w.rules().aircraft[ti].name.clone();
        glasnost_sim::testkit::spawn(w, Kind::Aircraft, &name, local.0, at, 0)
    };
    let dest = CellXY::new(home.x - 14, home.y + 8);
    c.session
        .as_mut()
        .unwrap()
        .play
        .issue(vec![glasnost_sim::orders::Order::Behaviour {
            actor: a,
            behaviour: Behaviour::Move,
            attack_move: false,
            target: None,
            dest: Some(Target::Cell(dest)),
        }]);
    for _ in 0..600 {
        ticks(&mut c, &mut clock, 1);
        if glasnost_sim::air::height(world(&c), a) >= 400 {
            break;
        }
    }
    // Below the condition-red ratio: a third of its strength.
    {
        let w = world_mut(&mut c);
        let o = w.obj_mut(a).unwrap();
        o.health = (o.health / 3).max(1);
    }
    let mut alive_smokes = 0;
    let mut seen: std::collections::BTreeSet<ObjectId> = Default::default();
    for t in 0..400 {
        ticks(&mut c, &mut clock, 1);
        for id in smoke_ids(&c) {
            if seen.insert(id) {
                alive_smokes += 1;
            }
        }
        if alive_smokes >= 3 || t == 399 {
            println!("living: {alive_smokes} smoke animations after {t} ticks");
            break;
        }
    }
    let cell = world(&c).obj(a).unwrap().pos.cell();
    look_at(&mut c, cell);
    if let Some(p) = centre_of(&mut c, a) {
        shots.save(&mut c, "i67-0-damaged");
        shots.zoom(&mut c, "i67-0-damaged", p, (200, 160, 3));
    }
    let killed_at = world(&c).tick;
    glasnost_sim::combat::damage::apply(
        world_mut(&mut c),
        a,
        glasnost_sim::combat::damage::Hit {
            amount: 1_000_000,
            warhead: None,
            distance: 0,
            attacker: None,
            house: None,
            ignore_defences: true,
            no_escape: false,
        },
    );
    println!(
        "killed at tick {killed_at}; crashing {}",
        glasnost_sim::air::crashing(world(&c), a)
    );
    let mut dead_smokes = 0;
    let mut images = std::collections::BTreeSet::new();
    for t in 0..200 {
        ticks(&mut c, &mut clock, 1);
        for id in smoke_ids(&c) {
            if seen.insert(id) {
                dead_smokes += 1;
            }
        }
        let Some(o) = world(&c).obj(a) else {
            println!("tick +{t}: left play");
            break;
        };
        let cell = o.pos.cell();
        let h = glasnost_sim::air::height(world(&c), a);
        look_at(&mut c, cell);
        let Some(p) = centre_of(&mut c, a) else {
            continue;
        };
        images.extend(body_images(&c, a));
        if t % 3 == 0 {
            println!("tick +{t}: height {h}, smoke animations so far {dead_smokes}");
            shots.zoom(&mut c, &format!("i67-1-crashing-t{t:02}"), p, (200, 160, 3));
        }
        if t == 9 {
            shots.save(&mut c, "i67-1-crashing");
        }
    }
    println!(
        "falling: {dead_smokes} smoke animations, {} distinct body images",
        images.len()
    );
}

/// The first building type, by index, that `pick` accepts.
fn building_type(
    w: &World,
    pick: impl Fn(&glasnost_sim::rules::BuildingRules) -> bool,
) -> Option<usize> {
    let r = w.rules();
    (0..r.buildings.len()).find(|&i| pick(&r.buildings[i]))
}

/// Notes 41, 42 and 43: the sidebar's buttons, a vehicle factory's rally
/// point, and the power meter at full power and short of power.
#[test]
#[ignore = "local lane: needs GLASNOST_GAME_DIR"]
fn notes_41_to_43_the_sidebar_and_the_rally_point() {
    let Some((mut c, shots)) = start((1280, 800)) else {
        eprintln!("GLASNOST_GAME_DIR not set: skipped");
        return;
    };
    let mut clock = 0;
    let home = deploy(&mut c, &mut clock);
    let local = c.session.as_ref().unwrap().play.me();

    // Note 41: the mode and tab buttons, zoomed.
    let _ = c.frame();
    let sell = c
        .session
        .as_ref()
        .unwrap()
        .hud_targets()
        .rect_of(&HudHit::Mode(Mode::Sell));
    if let Some(r) = sell {
        shots.save(&mut c, "n41-sidebar");
        shots.zoom(
            &mut c,
            "n41-buttons",
            Point::new(r.x + 70, r.y + 20),
            (150, 50, 5),
        );
        // Hover the repair button and the third tab.
        if let Some(r) = c
            .session
            .as_ref()
            .unwrap()
            .hud_targets()
            .rect_of(&HudHit::Mode(Mode::Repair))
        {
            c.pointer_moved(Point::new(r.x + r.w / 2, r.y + r.h / 2));
            let _ = c.frame();
            shots.zoom(
                &mut c,
                "n41-buttons-hover",
                Point::new(r.x + 20, r.y + 20),
                (220, 60, 4),
            );
        }
    }

    // Note 42: a vehicle factory of ours; a click on a cell sets its rally
    // point.
    let w = world(&c);
    let factory = building_type(w, |b| {
        b.factory == Some(glasnost_sim::rules::FactoryKind::Vehicle) && !b.naval
    });
    let Some(fi) = factory else {
        println!("no vehicle factory type");
        return;
    };
    let fcell = CellXY::new(home.x + 5, home.y - 1);
    let f = {
        let w = world_mut(&mut c);
        let name = w.rules().buildings[fi].name.clone();
        glasnost_sim::testkit::spawn(w, Kind::Building, &name, local.0, fcell, 0)
    };
    ticks(&mut c, &mut clock, 5);
    let rally = CellXY::new(home.x + 1, home.y + 6);
    look_at(&mut c, CellXY::new(home.x + 3, home.y + 3));
    // A building is selected by a click on it (a band box takes units only).
    if let Some(p) = centre_of(&mut c, f) {
        click_at(&mut c, p);
    }
    let before = world(&c).obj(f).and_then(|o| o.guard_spot);
    let selected = c.session.as_ref().unwrap().play.selection.clone();
    println!(
        "factory type {fi} at {fcell:?}; selected {}",
        selected.contains(&f)
    );
    if let Some(p) = screen_of(&mut c, rally) {
        click_at(&mut c, p);
    }
    ticks(&mut c, &mut clock, 3);
    let after = world(&c).obj(f).and_then(|o| o.guard_spot);
    println!("rally point before {before:?}, after {after:?} (clicked {rally:?})");
    let p = screen_of(&mut c, CellXY::new(home.x + 3, home.y + 3)).unwrap_or(Point::new(500, 400));
    // Move the pointer off the battlefield so the cursor does not cover it.
    c.pointer_moved(Point::new(1270, 790));
    shots.save(&mut c, "n42-rally");
    shots.zoom(&mut c, "n42-rally", p, (400, 300, 2));

    // Note 43: the power meter at full power ...
    let meter = |c: &mut Client, shots: &Shots, name: &str| {
        let s = c.session.as_ref().unwrap();
        let w = s.play.world();
        let (o, d) = glasnost_sim::econ::power::totals(w, local);
        let parts = glasnost_ui::hud::power_meter(w, local);
        println!("{name}: output {o}, drain {d}, meter parts {parts:?}");
        let r = glasnost_ui::hud::Layout::new(c.size).power;
        shots.zoom(
            c,
            name,
            Point::new(r.x + r.w / 2, r.y + r.h / 2),
            (170, 60, 4),
        );
        c.pointer_moved(Point::new(r.x + r.w / 2, r.y + r.h / 2));
        let _ = c.frame();
        shots.zoom(
            c,
            &format!("{name}-hover"),
            Point::new(r.x - 40, r.y + r.h / 2),
            (320, 90, 3),
        );
        c.pointer_moved(Point::new(1270, 790));
    };
    meter(&mut c, &shots, "n43-power-0");
    // ... with a power plant: output above drain, the lead's case ...
    let w = world(&c);
    let plant = building_type(w, |b| {
        b.raw.econ.power.unwrap_or(0) > 0 && b.factory.is_none() && b.foundation == (2, 2)
    });
    if let Some(pi) = plant {
        let cell = CellXY::new(home.x - 4, home.y + 3);
        let w = world_mut(&mut c);
        let name = w.rules().buildings[pi].name.clone();
        glasnost_sim::testkit::spawn(w, Kind::Building, &name, local.0, cell, 0);
        ticks(&mut c, &mut clock, 60);
        meter(&mut c, &shots, "n43-power-full");
    } else {
        println!("no 2 x 2 power plant type");
    }
    // ... then short of power: consumers of the largest drain placed in a
    // row until the drain is above the output.
    let w = world(&c);
    let consumer = building_type(w, |b| {
        b.raw.econ.power.unwrap_or(0) < 0 && b.factory.is_none() && b.foundation == (2, 2)
    });
    if let Some(ci) = consumer {
        for k in 0..12 {
            let (o, d) = glasnost_sim::econ::power::totals(world(&c), local);
            if d > o + 150 {
                break;
            }
            let cell = CellXY::new(home.x - 8 + 3 * (k % 4), home.y - 8 - 3 * (k / 4));
            let w = world_mut(&mut c);
            let name = w.rules().buildings[ci].name.clone();
            glasnost_sim::testkit::spawn(w, Kind::Building, &name, local.0, cell, 0);
            ticks(&mut c, &mut clock, 20);
        }
        ticks(&mut c, &mut clock, 60);
        meter(&mut c, &shots, "n43-power-1");
    } else {
        println!("no 2 x 2 consumer type");
    }
}
