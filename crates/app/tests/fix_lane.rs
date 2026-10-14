//! Local lane for the second visual-fix list (never in CI): with
//! `GLASNOST_GAME_DIR` set, starts a skirmish through the windowless client
//! and plays the scenes of the lead's review through the client's own
//! input, writing each screen drawn on the CPU to `local-lane/visual-fix-2/`
//! (git-ignored). Prints counts, cells and timings only.
//!
//! Run with `cargo test -p glasnost-app --test fix_lane -- --ignored
//! --nocapture`. `GLASNOST_UI_MAP=n` picks the n-th offered map (default 0),
//! `GLASNOST_FACTORY=n` the n-th ground vehicle factory type when none has a
//! door sprite (default 1), `GLASNOST_WIDE=n` the wide window's width.

#![cfg(not(target_arch = "wasm32"))]

use std::rc::Rc;

use glasnost_app::game::client::{Client, Key, run_job};
use glasnost_app::game::play::Button;
use glasnost_app::mapview::png;
use glasnost_map::CellXY;
use glasnost_sim::behaviour::Door;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::targeting::Modifiers;
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

fn click_at(c: &mut Client, b: Button, p: Point) {
    // The hit targets come from the last drawn frame.
    let _ = c.frame();
    c.pointer_moved(p);
    c.pointer_down(b, p);
    c.pointer_up(b, p, Modifiers::default());
}

fn click(c: &mut Client, r: Rect) {
    click_at(c, Button::Left, Point::new(r.x + r.w / 2, r.y + r.h / 2));
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
    }

    /// Writes 600-pixel-wide crops at the left and right ends of the drawn
    /// (not black) part of the battlefield, `name-l.png` and `name-r.png`.
    fn edges(&self, c: &mut Client, name: &str) {
        let rgba = c.frame_rgba();
        let (w, h) = c.size;
        let m = c.session.as_ref().unwrap().map_rect();
        let lit = |x: i32| {
            (0..h).step_by(4).any(|y| {
                let i = ((y * w + x) * 4) as usize;
                rgba[i] | rgba[i + 1] | rgba[i + 2] != 0
            })
        };
        let cols: Vec<i32> = (m.x..m.x + m.w).filter(|&x| lit(x)).collect();
        let (Some(&l), Some(&r)) = (cols.first(), cols.last()) else {
            return;
        };
        println!("{name}: drawn columns {l} … {r} of {}", m.w);
        for (tag, x0) in [("l", (l - 100).max(0)), ("r", (r - 500).max(0))] {
            let cw = 600.min(w - x0);
            let mut out = Vec::with_capacity((cw * h * 4) as usize);
            for y in 0..h {
                let i = ((y * w + x0) * 4) as usize;
                out.extend_from_slice(&rgba[i..i + (cw * 4) as usize]);
            }
            let path = self.dir.join(format!("{name}-{tag}.png"));
            std::fs::write(&path, png::png(cw as u32, h as u32, &out)).unwrap();
        }
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
    let out =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../local-lane/visual-fix-2");
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

/// Reveals the whole map to the local house, as its defeat would
/// (S-0409 claim 15), so that the edges are drawn.
fn reveal(c: &mut Client) {
    let s = c.session.as_mut().unwrap();
    let local = s.play.local;
    glasnost_sim::econ::shroud::defeat_reveal(&mut s.play.game.world, local);
}

/// Centres the view on `cell`.
fn look_at(c: &mut Client, cell: CellXY) {
    let s = c.session.as_mut().unwrap();
    let (x, y) = glasnost_render::projection::cell_centre(cell.x, cell.y);
    s.set_centre(glasnost_render::projection::project_lifted(x, y, 0));
}

/// A battlefield point whose cell is `cell` (and, with `obj`, whose
/// object is `obj`).
fn point_of(c: &mut Client, cell: Option<CellXY>, obj: Option<ObjectId>) -> Option<Point> {
    let s = c.session.as_mut().unwrap();
    let m = s.map_rect();
    for y in (m.y..m.y + m.h).step_by(3) {
        for x in (m.x..m.x + m.w).step_by(3) {
            let p = Point::new(x, y);
            let (at, id) = s.under(p);
            if cell.is_none_or(|c| c == at) && obj.is_none_or(|o| id == Some(o)) {
                return Some(p);
            }
        }
    }
    None
}

/// The mean screen point of the pixels that hit object `obj`.
fn centre_of(c: &mut Client, obj: ObjectId) -> Option<Point> {
    let s = c.session.as_mut().unwrap();
    let m = s.map_rect();
    let (mut sx, mut sy, mut n) = (0i64, 0i64, 0i64);
    for y in (m.y..m.y + m.h).step_by(3) {
        for x in (m.x..m.x + m.w).step_by(3) {
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
    let local = s.play.local;
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

/// Item 6: a rally point set with the client's left click from a selected
/// war factory, and a produced vehicle going there.
#[test]
#[ignore = "local lane: needs GLASNOST_GAME_DIR"]
fn rally_point_from_a_factory() {
    let Some((mut c, shots)) = start((1280, 800)) else {
        eprintln!("GLASNOST_GAME_DIR not set: skipped");
        return;
    };
    let mut clock = 0i64;
    let base = deploy(&mut c, &mut clock);
    let s = c.session.as_mut().unwrap();
    let local = s.play.local;
    // The first ground vehicle factory type, placed beside the base.
    let w = s.play.world();
    let names: Vec<String> = w
        .rules()
        .buildings
        .iter()
        .filter(|b| b.raw.weapons_factory == Some(true) && b.raw.naval != Some(true))
        .map(|b| b.name.clone())
        .collect();
    println!("{} ground vehicle factory types", names.len());
    // The factory types' door sprites (S-0108 claim 23): use one that has one.
    let import = &s.loaded.import;
    let has_door: Vec<bool> = names
        .iter()
        .map(|n| {
            let ti = w.rules().index_of(Kind::Building, n).unwrap() as usize;
            let roles = import
                .arts
                .get(&(glasnost_convert::kinds::Kind::Buildings, ti))
                .map(|a| a.building_sprites.keys().copied().collect::<Vec<_>>())
                .unwrap_or_default();
            println!("factory type {n}: sprite roles {roles:?}");
            roles.contains(&"door")
        })
        .collect();
    let pick = has_door
        .iter()
        .position(|&d| d)
        .unwrap_or(env_num("GLASNOST_FACTORY", 1).min(names.len() - 1));
    let at = CellXY::new(base.x + 5, base.y - 1);
    let fact = glasnost_sim::testkit::spawn(
        &mut s.play.game.world,
        Kind::Building,
        &names[pick],
        local.0,
        at,
        0,
    );
    s.play.game.world.houses[usize::from(local.0)].credits = 100_000;
    ticks(&mut c, &mut clock, 2);
    look_at(&mut c, CellXY::new(at.x + 3, at.y + 3));
    ticks(&mut c, &mut clock, 1);
    let on_fact = point_of(&mut c, None, Some(fact)).expect("the factory is on screen");
    let fact_centre = centre_of(&mut c, fact).unwrap_or(on_fact);
    {
        // The factory type's door art as imported (S-0108 claims 22 to 24).
        let s = c.session.as_ref().unwrap();
        let ti = s
            .play
            .world()
            .obj(fact)
            .map(|o| o.type_index as usize)
            .unwrap();
        let import = &s.loaded.import;
        if let Some(a) = import
            .arts
            .get(&(glasnost_convert::kinds::Kind::Buildings, ti))
        {
            let doc = glasnost_ini::Document::parse(
                &import
                    .art_file
                    .as_ref()
                    .map(|f| f.bytes.clone())
                    .unwrap_or_default(),
                Default::default(),
                &mut Vec::new(),
            );
            let key = |k: &[u8]| {
                doc.value(&a.image, k)
                    .map(|v| String::from_utf8_lossy(v).into_owned())
            };
            println!(
                "factory image {}: sprite roles {:?}; DoorStages {:?}, DamagedDoor {:?}",
                String::from_utf8_lossy(&a.image),
                a.building_sprites.keys().collect::<Vec<_>>(),
                key(b"DoorStages"),
                key(b"DamagedDoor"),
            );
        }
    }
    click_at(&mut c, Button::Left, on_fact);
    let s = c.session.as_ref().unwrap();
    println!("selection after the factory click: {:?}", s.play.selection);
    let rally = CellXY::new(at.x + 6, at.y + 6);
    let on_rally = point_of(&mut c, Some(rally), None).expect("the rally cell is on screen");
    c.pointer_moved(on_rally);
    click_at(&mut c, Button::Left, on_rally);
    ticks(&mut c, &mut clock, 2);
    let s = c.session.as_ref().unwrap();
    println!(
        "selection {:?}; guard spot {:?}",
        s.play.selection,
        s.play.world().obj(fact).and_then(|o| o.guard_spot)
    );
    shots.save(&mut c, "rally-set");
    shots.zoom(&mut c, "door-before", fact_centre);
    let s = c.session.as_mut().unwrap();
    s.play.game.world.houses[usize::from(local.0)].credits = 100_000;
    let vehicle = s.play.world().houses[usize::from(local.0)]
        .econ
        .tech
        .icons
        .iter()
        .flatten()
        .find(|i| i.t.kind == Kind::Vehicle && i.cost < 1500)
        .map(|i| i.t);
    println!("vehicle icon {vehicle:?}");
    let Some(what) = vehicle else { return };
    let n = s
        .play
        .issue(vec![glasnost_sim::orders::Order::Build { what }]);
    println!("build orders taken: {n}");
    let before: Vec<ObjectId> = own_vehicles(&c);
    let mut seen = None;
    let mut doors: Vec<&str> = Vec::new();
    let mut going = false;
    for _ in 0..3000 {
        ticks(&mut c, &mut clock, 1);
        let s = c.session.as_ref().unwrap();
        let w = s.play.world();
        if seen.is_none() {
            seen = own_vehicles(&c).into_iter().find(|id| !before.contains(id));
            if let Some(id) = seen {
                println!("vehicle {id:?} made at tick {}", w.tick);
            }
        }
        // Item 3: the factory's door through the unload (S-0108 claim 23).
        let door = w
            .obj(fact)
            .and_then(|o| (o.beh.current == Some(Behaviour::Unload)).then_some(o.beh.gate.door));
        let name = match door {
            Some(Door::Opening { .. }) => Some("door-opening"),
            Some(Door::Open) => Some("door-open"),
            Some(Door::Closing { .. }) => Some("door-closing"),
            Some(Door::Closed) => Some("door-closed-unloading"),
            None => None,
        };
        let tick = w.tick;
        if let Some(name) = name
            && !doors.contains(&name)
        {
            doors.push(name);
            println!("{name} at tick {tick}");
            shots.save(&mut c, name);
            shots.zoom(&mut c, name, fact_centre);
        }
        let s = c.session.as_ref().unwrap();
        let w = s.play.world();
        if let Some(o) = seen.and_then(|id| w.obj(id))
            && o.on_map
        {
            if o.pos.cell() == rally {
                println!("reached the rally cell at tick {}", w.tick);
                break;
            }
            let (dx, dy) = (o.pos.cell().x - rally.x, o.pos.cell().y - rally.y);
            if !going && door.is_none() && dx.abs() + dy.abs() <= 4 {
                going = true;
                shots.save(&mut c, "rally-going");
            }
        }
    }
    let s = c.session.as_ref().unwrap();
    if let Some(o) = seen.and_then(|id| s.play.world().obj(id)) {
        println!(
            "vehicle at {:?}, guard spot {:?}, behaviour {:?}, dest {:?}",
            o.pos.cell(),
            o.guard_spot,
            o.beh.current,
            o.dest
        );
    }
    shots.save(&mut c, "rally-arrived");
}

fn own_vehicles(c: &Client) -> Vec<ObjectId> {
    let s = c.session.as_ref().unwrap();
    let local = s.play.local;
    s.play
        .world()
        .objects
        .values()
        .filter(|o| o.owner == Some(local) && o.kind == Kind::Vehicle)
        .map(|o| o.id)
        .collect()
}

/// Item 8: on a window wider than the map's local area the view stands
/// still on that axis while scrolling (S-0101 claim 8, r10's centring),
/// at both map edges; and the map's right edge at an ordinary width.
#[test]
#[ignore = "local lane: needs GLASNOST_GAME_DIR"]
fn wide_view_and_the_right_edge() {
    let wide = env_num("GLASNOST_WIDE", 6000) as i32;
    let Some((mut c, shots)) = start((wide, 1080)) else {
        eprintln!("GLASNOST_GAME_DIR not set: skipped");
        return;
    };
    let mut clock = 0i64;
    reveal(&mut c);
    ticks(&mut c, &mut clock, 5);
    let s = c.session.as_mut().unwrap();
    let m = s.map_rect();
    println!("view {}x{}", m.w, m.h);
    for (name, x) in [("wide-left", -1_000_000), ("wide-right", 1_000_000)] {
        let s = c.session.as_mut().unwrap();
        let y = s.view().origin.1 + m.h / 2;
        s.set_centre((x, y));
        let before = s.view().origin;
        let mut seen = vec![before];
        for d in [32, 32, -32, -64, 32, 32, 32] {
            s.scroll((d, 0));
            seen.push(s.view().origin);
        }
        println!("{name}: view origins while scrolling {seen:?}");
        shots.save(&mut c, name);
        shots.edges(&mut c, name);
    }
    c.resize((1280, 800));
    let s = c.session.as_mut().unwrap();
    let y = s.view().origin.1 + s.map_rect().h / 2;
    s.set_centre((1_000_000, y));
    shots.save(&mut c, "right-edge");
}

/// High bridges of the map whose both ends are banks at deck level: the
/// (bank, bank) pairs along each deck run (S-0209 claim 15's crossing).
fn bridges(c: &Client) -> Vec<(CellXY, CellXY)> {
    use glasnost_sim::cells::Axis;
    let w = c.session.as_ref().unwrap().play.world();
    let cells = &w.cells;
    let level = |c: CellXY| cells.cell(c).map(|c| i32::from(c.level));
    let deck = |c: CellXY| cells.idx(c).is_some_and(|i| cells.deck[i]);
    let mut out = Vec::new();
    for y in 0..512 {
        for x in 0..512 {
            let start = CellXY::new(x, y);
            let Some(i) = cells.idx(start) else { continue };
            if !cells.deck[i] {
                continue;
            }
            let (dx, dy) = match cells.bridge_axis[i] {
                Axis::X => (1, 0),
                Axis::Y => (0, 1),
                Axis::None => continue,
            };
            // The first cell of each run only.
            if deck(CellXY::new(x - dx, y - dy)) {
                continue;
            }
            let mut end = start;
            while deck(CellXY::new(end.x + dx, end.y + dy)) {
                end = CellXY::new(end.x + dx, end.y + dy);
            }
            let (a, b) = (
                CellXY::new(x - dx, y - dy),
                CellXY::new(end.x + dx, end.y + dy),
            );
            let ground = level(start).unwrap_or(0);
            if level(a) == Some(ground + 4) && level(b) == Some(ground + 4) && end != start {
                out.push((a, b));
            }
        }
    }
    out
}

/// Item 1: infantry walking across a high bridge on its deck, ordered with
/// the client's left click, and one ordered onto the deck itself.
#[test]
#[ignore = "local lane: needs GLASNOST_GAME_DIR"]
fn infantry_on_a_bridge_deck() {
    use glasnost_sim::world::Layer;
    let Some((mut c, shots)) = start((1280, 800)) else {
        eprintln!("GLASNOST_GAME_DIR not set: skipped");
        return;
    };
    let mut clock = 0i64;
    reveal(&mut c);
    ticks(&mut c, &mut clock, 2);
    let found = bridges(&c);
    println!("{} high bridges with banks at deck level", found.len());
    let Some(&(a, b)) = found.first() else {
        println!("no such bridge on this map: pick another with GLASNOST_UI_MAP");
        return;
    };
    println!("bridge from {a:?} to {b:?}");
    let s = c.session.as_mut().unwrap();
    let local = s.play.local;
    let w = s.play.world();
    let walker = w
        .rules()
        .infantry
        .iter()
        .find(|u| u.locomotor == glasnost_sim::rules::Locomotor::Walk && !u.jumpjet && u.speed > 0)
        .map(|u| u.name.clone())
        .expect("a walking infantry type");
    let mut men = Vec::new();
    for k in 0..2 {
        men.push(glasnost_sim::testkit::spawn(
            &mut s.play.game.world,
            Kind::Infantry,
            &walker,
            local.0,
            a,
            16384 * k,
        ));
    }
    ticks(&mut c, &mut clock, 2);
    let mid = CellXY::new((a.x + b.x) / 2, (a.y + b.y) / 2);
    look_at(&mut c, mid);
    ticks(&mut c, &mut clock, 1);
    // The first across to the far bank, the second onto the deck's middle.
    for (man, to) in [(men[0], b), (men[1], mid)] {
        let s = c.session.as_mut().unwrap();
        s.play.select_box(&[man], false);
        let p = point_of(&mut c, Some(to), None).expect("the cell is on screen");
        click_at(&mut c, Button::Left, p);
    }
    let mut shot = false;
    let mut last = Vec::new();
    for _ in 0..1500 {
        ticks(&mut c, &mut clock, 1);
        let s = c.session.as_ref().unwrap();
        let w = s.play.world();
        let now: Vec<_> = men
            .iter()
            .filter_map(|&m| w.obj(m))
            .map(|o| {
                (
                    o.pos.cell(),
                    o.layer,
                    o.pos.z - w.surface_at(o.pos.x, o.pos.y),
                )
            })
            .collect();
        if now.iter().map(|n| n.1).collect::<Vec<_>>() != last {
            println!("tick {}: {now:?}", w.tick);
            last = now.iter().map(|n| n.1).collect();
        }
        let first = w.obj(men[0]).unwrap();
        if !shot && first.layer == Layer::Deck && first.pos.cell() == mid {
            shot = true;
            shots.save(&mut c, "bridge-deck");
        }
        let s = c.session.as_ref().unwrap();
        let w = s.play.world();
        let done = men.iter().all(|&m| {
            w.obj(m).is_some_and(|o| {
                o.dest.is_none() && o.walk.as_ref().is_some_and(|s| s.target.is_none())
            })
        });
        if shot && done {
            break;
        }
    }
    let s = c.session.as_ref().unwrap();
    let w = s.play.world();
    for &m in &men {
        let o = w.obj(m).unwrap();
        println!(
            "{m:?} ends at {:?} on {:?}, {} above the ground, destination {:?}",
            o.pos.cell(),
            o.layer,
            o.pos.z - w.surface_at(o.pos.x, o.pos.y),
            o.dest
        );
    }
    shots.save(&mut c, "bridge-deck-standing");
}
