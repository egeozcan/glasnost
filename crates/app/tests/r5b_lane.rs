//! Local lane for the M8 A4 round 5 play notes, part 2 (never in CI): with
//! `GLASNOST_GAME_DIR` set, starts a skirmish through the windowless client
//! and plays notes 26 (the superweapon buildings' slot animations) and 28 (a
//! building under repair). Note 32 (the deploy key pressed for falling
//! paratroopers) needs no picture: the specs give what the game does, and
//! `claims_s0522`'s claim-17 test plays it. Each screen drawn on the CPU goes to
//! `local-lane/m8/a4-r5/part2/` (git-ignored; `GLASNOST_LANE_OUT` moves
//! it). Types are chosen by their fields, never by name; it prints counts
//! and booleans only.
//!
//! Run with `cargo test --release -p glasnost-app --test r5b_lane --
//! --ignored --nocapture`. `GLASNOST_UI_MAP=n` picks the n-th offered map
//! (default 0).

#![cfg(not(target_arch = "wasm32"))]

use std::rc::Rc;

use glasnost_app::game::client::{Client, Key, run_job};
use glasnost_app::game::play::Button;
use glasnost_app::mapview::png;
use glasnost_map::CellXY;
use glasnost_sim::rules::Kind;
use glasnost_sim::world::ObjectId;
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
    c.pointer_up(Button::Left, p, Default::default());
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

    /// Writes `name-zoom.png`: the 300 × 200 pixels around `at`, enlarged
    /// twice.
    fn zoom(&self, c: &mut Client, name: &str, at: Point) {
        let rgba = c.frame_rgba();
        let (w, h) = c.size;
        let (cw, ch, k) = (300, 200, 2);
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

/// Starts a skirmish from the menu on a window of `size`.
fn start(size: (i32, i32)) -> Option<(Client, Shots)> {
    let dir = std::env::var_os("GLASNOST_GAME_DIR")?;
    let out = png::lane_out("m8/a4-r5/part2");
    std::fs::create_dir_all(&out).unwrap();
    let mut c = Client::new();
    c.resize(size);
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
    Some((c, Shots { dir: out }))
}

/// Centres the view on `cell`.
fn look_at(c: &mut Client, cell: CellXY) {
    let s = c.session.as_mut().unwrap();
    let (x, y) = glasnost_render::projection::cell_centre(cell.x, cell.y);
    s.set_centre(glasnost_render::projection::project_lifted(x, y, 0));
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

/// The first cell, row by row in a square of `r` cells round `around`,
/// where a building of type `t` passes the place order's cell test with a
/// free ring of one cell round it (trees and other objects out of the way).
fn free_spot(
    w: &glasnost_sim::world::World,
    house: glasnost_sim::world::HouseId,
    t: u32,
    around: CellXY,
    r: i32,
) -> Option<CellXY> {
    use glasnost_sim::econ::placement;
    let (fw, fh) = placement::place_type(w, t).size;
    let test = placement::Test::order(house);
    for dy in -r..=r {
        for dx in -r..=r {
            let at = CellXY::new(around.x + dx, around.y + dy);
            let ring = (0..fh + 2).all(|y| {
                (0..fw + 2).all(|x| {
                    let c = CellXY::new(at.x - 1 + x, at.y - 1 + y);
                    w.cells.footprint().is_legal(c)
                        && w.objects.values().all(|o| !o.on_map || o.pos.cell() != c)
                })
            });
            if ring && placement::cell_test(w, t, at, &test) {
                return Some(at);
            }
        }
    }
    None
}

/// The slot base keys of S-0108 (Import mapping), in its Terms' order.
const SLOT_KEYS: [&[u8]; 18] = [
    b"IdleAnim",
    b"ActiveAnim",
    b"ActiveAnimTwo",
    b"ActiveAnimThree",
    b"ActiveAnimFour",
    b"PreProductionAnim",
    b"ProductionAnim",
    b"TurretAnim",
    b"SpecialAnim",
    b"SpecialAnimTwo",
    b"SpecialAnimThree",
    b"SpecialAnimFour",
    b"SuperAnim",
    b"SuperAnimTwo",
    b"SuperAnimThree",
    b"SuperAnimFour",
    b"LowPower",
    b"SuperLowPower",
];

/// The art document of the lane's import.
fn art_doc(c: &Client) -> glasnost_ini::Document {
    let s = c.session.as_ref().unwrap();
    let f = s.loaded.import.art_file.as_ref().expect("an art file");
    glasnost_ini::Document::parse(&f.bytes, Default::default(), &mut Vec::new())
}

/// Note 26: every superweapon building type, its art slots (which have a
/// name, and which name an animation type), placed and run; the running
/// slot count per placed building and a picture of each.
#[test]
#[ignore = "local lane: needs GLASNOST_GAME_DIR"]
fn note_26_superweapon_buildings() {
    use glasnost_app::mapview::{build, live, slots};
    let Some((mut c, shots)) = start((1280, 800)) else {
        eprintln!("GLASNOST_GAME_DIR not set: skipped");
        return;
    };
    let mut clock = 0i64;
    let base = deploy(&mut c, &mut clock);
    let doc = art_doc(&c);
    let s = c.session.as_mut().unwrap();
    let local = s.play.me();
    let r = s.play.world().rules().clone();
    let sw_types: Vec<(usize, u32)> = (0..r.buildings.len())
        .filter_map(|i| Some((i, r.superweapons.building_sw.get(i)?[0]?)))
        .collect();
    println!("note 26: superweapon building types {}", sw_types.len());
    let import = &s.loaded.import;
    let mut placed = Vec::new();
    for (k, &(i, sw)) in sw_types.iter().enumerate() {
        let kind = r.superweapons.types.get(sw as usize).and_then(|t| t.kind);
        let name = live::name_bytes(&r.buildings[i].name);
        let Some(idx) = build::find_type(import, glasnost_convert::kinds::Kind::Buildings, &name)
        else {
            println!("note 26: type {k}: not in the import");
            continue;
        };
        let image = import
            .arts
            .get(&(glasnost_convert::kinds::Kind::Buildings, idx))
            .map(|a| a.image.clone())
            .unwrap_or_default();
        let mut row = Vec::new();
        for base_key in SLOT_KEYS {
            let d = slots::read_slot(&doc, &image, base_key);
            if d.normal.is_empty() {
                continue;
            }
            let found =
                build::find_type(import, glasnost_convert::kinds::Kind::Animations, &d.normal)
                    .is_some();
            row.push(format!(
                "{}{}",
                String::from_utf8_lossy(base_key),
                if found { "" } else { "(no type)" }
            ));
        }
        let bib = doc.value(&image, b"BibShape").is_some();
        // S-0108 claim 18 runs only when ChargedAnimTime is at most 990: whether the type sets
        // the key, and whether it is then at most 990 (a boolean, never the value).
        let charged = live::charged_anim_time(import, &r.buildings[i].name, r.mode);
        let set = charged.is_some();
        let on = charged.is_some_and(|t| {
            t.le(glasnost_num::F64::from_i64(
                990,
                glasnost_num::Rounding::TowardZero,
            ))
        });
        println!(
            "note 26: type {k}: kind {kind:?}, slots named [{}], bib {bib}, ChargedAnimTime set {set}, at most 990 {on}",
            row.join(", ")
        );
        placed.push((k, i, kind));
    }
    let wm = &mut s.play.game.world;
    wm.houses[usize::from(local.0)].credits = 100_000;
    // Power for everything.
    let power = (0..r.buildings.len())
        .filter(|&i| {
            r.superweapons
                .building_sw
                .get(i)
                .is_none_or(|s| s[0].is_none())
                && r.buildings[i].raw.econ.power.unwrap_or(0) > 0
        })
        .max_by_key(|&i| r.buildings[i].raw.econ.power.unwrap_or(0));
    if let Some(p) = power {
        for _ in 0..6 {
            if let Some(at) = free_spot(wm, local, p as u32, base, 30) {
                glasnost_sim::testkit::spawn(
                    wm,
                    Kind::Building,
                    &r.buildings[p].name,
                    local.0,
                    at,
                    0,
                );
            }
        }
    }
    let mut ids = Vec::new();
    for &(k, i, kind) in &placed {
        let Some(at) = free_spot(wm, local, i as u32, base, 30) else {
            println!("note 26: type {k}: no free place");
            continue;
        };
        let id =
            glasnost_sim::testkit::spawn(wm, Kind::Building, &r.buildings[i].name, local.0, at, 0);
        ids.push((k, id, at, kind));
    }
    ticks(&mut c, &mut clock, 300);
    for &(k, id, at, kind) in &ids {
        let s = c.session.as_ref().unwrap();
        let live = s.loaded.live.as_ref().expect("a live game");
        let names = live.slot_names(id);
        let slots: Vec<&str> = names
            .iter()
            .map(|n| n.split('=').next().unwrap_or(""))
            .collect();
        let w = s.play.world();
        let sw = w.rules().superweapons.building_sw[sw_types[k].0][0];
        let granted = sw
            .and_then(|i| glasnost_sim::superweapons::record(w, local, i))
            .is_some_and(|rec| rec.granted);
        println!(
            "note 26: type {k} ({kind:?}) placed: running slots {slots:?}, record granted {granted}"
        );
        look_at(&mut c, at);
        let chrono = matches!(kind, Some(glasnost_sim::superweapons::SwKind::Chrono));
        let tag = if chrono {
            "r5-26-chrono".to_string()
        } else {
            format!("r5-26-sw{k}")
        };
        if let Some(p) = centre_of(&mut c, id) {
            shots.zoom(&mut c, &tag, p);
        } else {
            shots.save(&mut c, &tag);
        }
    }
    // The chronosphere charged (S-0512 claim 14's set charge, 100 percent): claim 18 opens it when
    // its type's ChargedAnimTime is at most 990.
    let chrono = ids
        .iter()
        .find(|(.., kind)| matches!(kind, Some(glasnost_sim::superweapons::SwKind::Chrono)))
        .copied();
    if let Some((k, id, at, _)) = chrono {
        {
            let s = c.session.as_mut().unwrap();
            let sw = r.superweapons.building_sw[sw_types[k].0][0].unwrap();
            glasnost_sim::superweapons::charge::set_charge(&mut s.play.game.world, local, sw, 100);
        }
        ticks(&mut c, &mut clock, 300);
        let s = c.session.as_ref().unwrap();
        let live = s.loaded.live.as_ref().expect("a live game");
        let names = live.slot_names(id);
        let slots: Vec<&str> = names
            .iter()
            .map(|n| n.split('=').next().unwrap_or(""))
            .collect();
        println!("note 26: chrono charged: running slots {slots:?}");
        look_at(&mut c, at);
        if let Some(p) = centre_of(&mut c, id) {
            shots.zoom(&mut c, "r5-26-chrono-charged", p);
        }
    }
}

/// Note 28: a damaged power plant of the local house switched to repair;
/// the wrench over it.
#[test]
#[ignore = "local lane: needs GLASNOST_GAME_DIR"]
fn note_28_repair_wrench() {
    use glasnost_sim::orders::Order;
    let Some((mut c, shots)) = start((1280, 800)) else {
        eprintln!("GLASNOST_GAME_DIR not set: skipped");
        return;
    };
    let mut clock = 0i64;
    let base = deploy(&mut c, &mut clock);
    let s = c.session.as_mut().unwrap();
    let local = s.play.me();
    let r = s.play.world().rules().clone();
    let wm = &mut s.play.game.world;
    wm.houses[usize::from(local.0)].credits = 100_000;
    let plant = (0..r.buildings.len())
        .filter(|&i| {
            r.superweapons
                .building_sw
                .get(i)
                .is_none_or(|s| s[0].is_none())
                && r.buildings[i].raw.econ.power.unwrap_or(0) > 0
        })
        .max_by_key(|&i| r.buildings[i].raw.econ.power.unwrap_or(0))
        .expect("a power type");
    let at = free_spot(wm, local, plant as u32, base, 30).expect("a free place");
    let id =
        glasnost_sim::testkit::spawn(wm, Kind::Building, &r.buildings[plant].name, local.0, at, 0);
    let strength = r.buildings[plant].strength;
    wm.obj_mut(id).unwrap().health = strength / 3;
    ticks(&mut c, &mut clock, 20);
    let s = c.session.as_mut().unwrap();
    let n = s.play.issue(vec![Order::Repair { actor: id }]);
    println!("note 28: repair orders taken {n}");
    ticks(&mut c, &mut clock, 30);
    let s = c.session.as_ref().unwrap();
    let repairing = s
        .play
        .world()
        .obj(id)
        .is_some_and(|o| o.econ.building.repairing);
    println!("note 28: repairing {repairing}");
    look_at(&mut c, at);
    if let Some(p) = centre_of(&mut c, id) {
        shots.zoom(&mut c, "r5-28-wrench", p);
    } else {
        shots.save(&mut c, "r5-28-wrench");
    }
    // Two more pictures a few ticks apart: the wrench's image frame changes.
    for k in 0..2 {
        ticks(&mut c, &mut clock, 2);
        if let Some(p) = centre_of(&mut c, id) {
            shots.zoom(&mut c, &format!("r5-28-wrench-{k}"), p);
        }
    }
}
