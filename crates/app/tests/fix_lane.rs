//! Local lane for the second visual-fix list (never in CI): with
//! `GLASNOST_GAME_DIR` set, starts a skirmish through the windowless client
//! and plays the scenes of the lead's review through the client's own
//! input, writing each screen drawn on the CPU to `local-lane/visual-fix-2/`
//! (git-ignored). Prints counts, cells and timings only.
//!
//! Run with `cargo test -p glasnost-app --test fix_lane -- --ignored
//! --nocapture`. `GLASNOST_UI_MAP=n` picks the n-th offered map (default 0).

#![cfg(not(target_arch = "wasm32"))]

use std::rc::Rc;

use glasnost_app::game::client::{Client, Key, run_job};
use glasnost_app::game::play::Button;
use glasnost_app::mapview::png;
use glasnost_map::CellXY;
use glasnost_sim::rules::Kind;
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
    let at = CellXY::new(base.x + 5, base.y - 1);
    let fact = glasnost_sim::testkit::spawn(
        &mut s.play.game.world,
        Kind::Building,
        &names[0],
        local.0,
        at,
        0,
    );
    s.play.game.world.houses[usize::from(local.0)].credits = 100_000;
    ticks(&mut c, &mut clock, 2);
    look_at(&mut c, CellXY::new(at.x + 3, at.y + 3));
    ticks(&mut c, &mut clock, 1);
    let on_fact = point_of(&mut c, None, Some(fact)).expect("the factory is on screen");
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
        if let Some(o) = seen.and_then(|id| w.obj(id))
            && o.on_map
            && o.pos.cell() == rally
        {
            println!("reached the rally cell at tick {}", w.tick);
            break;
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
    shots.save(&mut c, "rally");
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
