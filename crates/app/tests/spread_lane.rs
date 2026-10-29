//! Local lane for M8 A3 intake 42, S-0401 r17 and M-0013 (never in CI):
//! with `GLASNOST_GAME_DIR` set, starts a skirmish through the windowless
//! client and watches resource spreading next to trees, buildings, walls and
//! bridges. Each new resource cell is sorted by what its cell holds when it
//! appears; under claim 11 a cell holding a terrain object never receives
//! resource, and under claim 10 no building, wall or bridge cell does. It
//! also counts the spreading cells whose only germinable neighbours hold
//! terrain objects (two or more of them: such a cell keeps its key-0 entry
//! and spends attempts there with `quirks.spread_step_lost_on_terrain_object`
//! on). Screens go to `local-lane/m8/a3-42/` (git-ignored;
//! `GLASNOST_LANE_OUT` moves it). It prints counts and booleans only.
//!
//! Run with `cargo test -p glasnost-app --test spread_lane -- --ignored
//! --nocapture`. `GLASNOST_UI_MAP=n` picks the n-th offered map (default
//! 0), `GLASNOST_UI_TICKS` the watch length (default 3000),
//! `GLASNOST_LANE_SPREAD=s` patches every tiberium's spread field to `s`
//! ticks in the lane's own copy of the rules (default 30, so that a short
//! scene shows spreading; 0 keeps the install's), and
//! `GLASNOST_LANE_SWITCH=off` sets the switch off in that copy.

#![cfg(not(target_arch = "wasm32"))]

use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;
use std::sync::Arc;

use glasnost_app::game::client::{Client, run_job};
use glasnost_app::game::play::Button;
use glasnost_app::mapview::png;
use glasnost_map::{CellXY, NO_OVERLAY};
use glasnost_sim::econ::resource;
use glasnost_sim::rules::Kind;
use glasnost_sim::targeting::Modifiers;
use glasnost_sim::world::{HouseId, World};
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
    c.pointer_up(Button::Left, p, Modifiers::default());
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

fn save(c: &mut Client, dir: &std::path::Path, name: &str) {
    let rgba = c.frame_rgba();
    let (w, h) = c.size;
    std::fs::write(
        dir.join(format!("{name}.png")),
        png::png(w as u32, h as u32, &rgba),
    )
    .unwrap();
    println!("wrote {name}.png");
}

fn look_at(c: &mut Client, cell: CellXY) {
    let s = c.session.as_mut().unwrap();
    let (x, y) = glasnost_render::projection::cell_centre(cell.x, cell.y);
    s.set_centre(glasnost_render::projection::project_lifted(x, y, 0));
}

fn world(c: &Client) -> &World {
    c.session.as_ref().unwrap().play.world()
}

fn neighbours(c: CellXY) -> impl Iterator<Item = CellXY> {
    glasnost_sim::regions::STEPS
        .iter()
        .map(move |&(dx, dy)| CellXY::new(c.x + dx, c.y + dy))
}

/// What a cell holds, for sorting: the first that applies.
fn holds(w: &World, c: CellXY) -> &'static str {
    let Some(i) = w.cells.idx(c) else {
        return "off the map";
    };
    if w.cells.deck[i] || w.cells.bridge_body[i] {
        return "bridge";
    }
    let list = w.cells.list(c, glasnost_sim::world::Layer::Ground);
    let objects: Vec<_> = list.iter().filter_map(|&id| w.obj(id)).collect();
    let kinds: Vec<Kind> = objects.iter().map(|o| o.kind).collect();
    if kinds.contains(&Kind::Terrain) {
        return "terrain object";
    }
    // Claim 10 looks at the first building only; a dead one, or one of an
    // invisible type, does not prevent new resource.
    if let Some(b) = objects.iter().find(|o| o.kind == Kind::Building) {
        let t = &w.rules().buildings[b.type_index as usize].raw;
        let invisible = t.econ.invisible.unwrap_or(false) || t.invisible_in_game.unwrap_or(false);
        return if invisible {
            "invisible building"
        } else if b.health <= 0 {
            "dead building"
        } else {
            "building"
        };
    }
    if w.cells.cell(c).is_some_and(|x| x.overlay != NO_OVERLAY)
        && resource::resource_at(w, c).is_none()
    {
        return "other overlay (a wall)";
    }
    if kinds.contains(&Kind::Infantry) || kinds.contains(&Kind::Vehicle) {
        return "a unit";
    }
    "free"
}

/// The objects of a cell's ground layer (kind, health, on the map, and for
/// a building whether its type is invisible), for a gain that should not
/// happen.
fn detail(w: &World, c: CellXY) -> String {
    let objects: Vec<String> = w
        .cells
        .list(c, glasnost_sim::world::Layer::Ground)
        .iter()
        .filter_map(|&id| w.obj(id))
        .map(|o| {
            let invisible = (o.kind == Kind::Building).then(|| {
                let b = &w.rules().buildings[o.type_index as usize].raw;
                b.econ.invisible.unwrap_or(false) || b.invisible_in_game.unwrap_or(false)
            });
            format!(
                "{:?} health {} on map {} invisible {invisible:?}",
                o.kind, o.health, o.on_map
            )
        })
        .collect();
    format!("{objects:?}, germinable now {}", resource::germinable(w, c))
}

/// Every resource cell of the map.
fn resource_cells(w: &World) -> BTreeSet<CellXY> {
    w.cells
        .grid
        .iter()
        .map(|(c, _)| c)
        .filter(|&c| resource::resource_at(w, c).is_some())
        .collect()
}

/// The resource cells next to a cell of each kind, and the cells whose
/// germinable neighbours (two or more) all hold terrain objects.
fn neighbourhood(w: &World, cells: &BTreeSet<CellXY>) -> (BTreeMap<&'static str, usize>, usize) {
    let mut next: BTreeMap<&'static str, usize> = BTreeMap::new();
    let mut tree_only = 0;
    for &c in cells {
        let mut seen = BTreeSet::new();
        let (mut g, mut g_tree) = (0, 0);
        for n in neighbours(c) {
            let k = holds(w, n);
            if k != "free" && k != "off the map" {
                seen.insert(k);
            }
            if resource::germinable(w, n) {
                g += 1;
                if k == "terrain object" {
                    g_tree += 1;
                }
            }
        }
        for k in seen {
            *next.entry(k).or_default() += 1;
        }
        if g >= 2 && g == g_tree {
            tree_only += 1;
        }
    }
    (next, tree_only)
}

#[test]
#[ignore = "local lane: needs GLASNOST_GAME_DIR"]
fn ore_spreads_next_to_trees_buildings_walls_and_bridges() {
    let Some(dir) = std::env::var_os("GLASNOST_GAME_DIR") else {
        eprintln!("GLASNOST_GAME_DIR not set: skipped");
        return;
    };
    let out = png::lane_out("m8/a3-42");
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
    c.next_seed = 4242;
    click(&mut c, r);
    run_jobs(&mut c);
    assert_eq!(
        c.shell.screen,
        Screen::Game,
        "the game starts: {}",
        c.status
    );
    let spread = env_num("GLASNOST_LANE_SPREAD", 30) as i32;
    let off = std::env::var("GLASNOST_LANE_SWITCH").is_ok_and(|v| v == "off");
    {
        let w = &mut c.session.as_mut().unwrap().play.game.world;
        let mut rules = (*w.rules_arc()).clone();
        if spread > 0 {
            for t in &mut rules.econ.resource.tiberiums {
                t.spread = spread;
            }
        }
        if off {
            rules.quirks.spread_step_lost_on_terrain_object = false;
        }
        w.rules = Some(Arc::new(rules));
        // Every player house allied with every other: the game plays on.
        let players: Vec<HouseId> = (0..w.houses.len())
            .filter(|&h| w.houses[h].start.is_some())
            .map(|h| HouseId(h as u8))
            .collect();
        for &h in &players {
            for &o in &players {
                if !w.houses[usize::from(h.0)].allies.contains(&o) {
                    w.houses[usize::from(h.0)].allies.push(o);
                }
            }
        }
    }
    println!(
        "spread field patched {}, switch {}",
        spread > 0,
        if off { "off" } else { "on" }
    );
    let mut clock = 0i64;
    let start = resource_cells(world(&c));
    let (next, tree_only) = neighbourhood(world(&c), &start);
    println!(
        "start: resource cells {}, next to {next:?}, spreading cells whose germinable neighbours are all terrain objects {tree_only}",
        start.len()
    );
    // Screens: a resource cell next to a tree, one next to a building, one
    // next to a wall and one next to a bridge, where the map has them.
    let mut spots: Vec<(&'static str, CellXY)> = Vec::new();
    for want in [
        "terrain object",
        "building",
        "other overlay (a wall)",
        "bridge",
    ] {
        if let Some(&s) = start
            .iter()
            .find(|&&s| neighbours(s).any(|n| holds(world(&c), n) == want))
        {
            spots.push((want, s));
        }
    }
    let shoot = |c: &mut Client, when: &str, spots: &[(&'static str, CellXY)]| {
        for (k, s) in spots {
            look_at(c, *s);
            let _ = c.frame();
            let name = format!("a3-42-{when}-{}", k.split(' ').next().unwrap_or("x"));
            save(c, &out, &name);
        }
    };
    shoot(&mut c, "start", &spots);
    let watch = env_num("GLASNOST_UI_TICKS", 3000);
    let mut known = start.clone();
    let mut gains: BTreeMap<&'static str, usize> = BTreeMap::new();
    let mut lost = 0usize;
    for done in 1..=watch {
        ticks(&mut c, &mut clock, 1);
        if done % 10 != 0 && done != watch {
            continue;
        }
        let w = world(&c);
        let now = resource_cells(w);
        for &n in now.difference(&known) {
            let k = holds(w, n);
            *gains.entry(k).or_default() += 1;
            if !["free", "a unit", "invisible building", "dead building"].contains(&k) {
                println!(
                    "tick +{done}: a gain in a cell holding {k}: {}",
                    detail(w, n)
                );
            }
        }
        lost += known.difference(&now).count();
        known = now;
        if done % 1000 == 0 {
            let (next, tree_only) = neighbourhood(w, &known);
            println!(
                "tick +{done}: resource cells {}, gained by kind {gains:?}, emptied {lost}, next to {next:?}, all-terrain-object spreaders {tree_only}",
                known.len()
            );
        }
        if c.shell.screen != Screen::Game {
            println!("tick +{done}: the game left the game screen");
            break;
        }
    }
    shoot(&mut c, "end", &spots);
    let w = world(&c);
    let bad: usize = [
        "terrain object",
        "building",
        "bridge",
        "other overlay (a wall)",
    ]
    .iter()
    .map(|k| gains.get(k).copied().unwrap_or(0))
    .sum();
    let trees_with_ore = known
        .iter()
        .filter(|&&k| holds(w, k) == "terrain object")
        .inspect(|&&k| {
            println!(
                "resource in a cell holding a terrain object: at the start {}, {}",
                start.contains(&k),
                detail(w, k)
            )
        })
        .count();
    println!(
        "end: gained {gains:?}; in a cell holding a terrain object, building, bridge or other overlay: {bad}; resource cells holding a terrain object now {trees_with_ore}"
    );
    assert_eq!(bad, 0, "no resource appears in those cells");
}
