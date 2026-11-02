//! Local lane for the M8 A4 round 6 rendering notes (never in CI): with
//! `GLASNOST_GAME_DIR` set, starts a skirmish through the windowless client
//! and plays note 38 (ore near the ore mines, grown in play) and note 39
//! (the walking vehicles' walk frames while they move). Each screen drawn
//! on the CPU goes to `local-lane/m8/a4-r6/render/` (git-ignored;
//! `GLASNOST_LANE_OUT` moves it). Types are chosen by their fields, never
//! by name; it prints counts, cells, ticks and booleans only.
//!
//! Run with `cargo test --release -p glasnost-app --test r6_render_lane --
//! --ignored --nocapture`. `GLASNOST_UI_MAP=n` picks the n-th offered map
//! (default 0).

#![cfg(not(target_arch = "wasm32"))]

use std::collections::BTreeMap;
use std::rc::Rc;

use glasnost_app::game::client::{Client, Key, run_job};
use glasnost_app::game::play::Button;
use glasnost_app::mapview::png;
use glasnost_map::CellXY;
use glasnost_map::terrain::land;
use glasnost_sim::econ::{minimap, resource};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::world::{ObjectId, Target, World};
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

/// Starts a skirmish from the menu on a window of `size`, every player
/// house allied (lobby team 1) so the game plays on undisturbed.
fn start(size: (i32, i32)) -> Option<(Client, Shots)> {
    let dir = std::env::var_os("GLASNOST_GAME_DIR")?;
    let out = png::lane_out("m8/a4-r6/render");
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

/// The screen point of the centre of `cell` (level 0), after a frame.
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

/// The ore mines on the map (S-0401 claim 15), by cell.
fn mine_cells(w: &World) -> Vec<CellXY> {
    let r = w.rules();
    w.objects
        .values()
        .filter(|o| o.kind == Kind::Terrain && o.on_map)
        .filter(|o| {
            r.terrain
                .get(o.type_index as usize)
                .is_some_and(|t| t.spawns_resources)
        })
        .map(|o| o.pos.cell())
        .collect()
}

/// The resource cells within `r` of `at`: stage histogram, and how many
/// hold a terrain object.
fn ore_round(w: &World, at: CellXY, r: i32) -> (BTreeMap<u8, usize>, usize) {
    let mut stages = BTreeMap::new();
    let mut n = 0;
    for dy in -r..=r {
        for dx in -r..=r {
            let c = CellXY::new(at.x + dx, at.y + dy);
            if let Some((_, d)) = resource::resource_at(w, c) {
                *stages.entry(d).or_default() += 1;
                n += 1;
            }
        }
    }
    (stages, n)
}

/// How many screen pixels the overlay of `cell` shows: the frame drawn
/// with the cell's overlay and without it (one tick each), compared in
/// a box round the cell.
fn shown_pixels(c: &mut Client, clock: &mut i64, cell: CellXY) -> usize {
    let Some(s) = screen_of(c, cell) else {
        return 0;
    };
    let with = c.frame_rgba();
    let kept = {
        let w = &mut c.session.as_mut().unwrap().play.game.world;
        let x = w.cells.grid.get_mut(cell).unwrap();
        let kept = (x.overlay, x.overlay_data);
        x.overlay = glasnost_map::NO_OVERLAY;
        x.overlay_data = 0;
        kept
    };
    ticks(c, clock, 1);
    let without = c.frame_rgba();
    {
        let w = &mut c.session.as_mut().unwrap().play.game.world;
        let x = w.cells.grid.get_mut(cell).unwrap();
        (x.overlay, x.overlay_data) = kept;
    }
    ticks(c, clock, 1);
    let (fw, fh) = c.size;
    let mut n = 0;
    for y in s.y - 60..s.y + 20 {
        for x in s.x - 40..s.x + 40 {
            if x < 0 || y < 0 || x >= fw || y >= fh {
                continue;
            }
            let k = ((y * fw + x) * 4) as usize;
            if with[k..k + 3] != without[k..k + 3] {
                n += 1;
            }
        }
    }
    n
}

/// Note 38: the ore round an ore mine. The resource within 3 cells of the
/// first mine near the local base is taken away, then the mine and the
/// field spread grow it back in play; pictures before, at the start and
/// as it grows, and of a 3-row strip of cells each set to one stage in
/// play (stages 0 to 11), beside the map's own field.
#[test]
#[ignore = "local lane: needs GLASNOST_GAME_DIR"]
fn note_38_ore_near_mines() {
    let Some((mut c, shots)) = start((1280, 800)) else {
        eprintln!("GLASNOST_GAME_DIR not set: skipped");
        return;
    };
    let mut clock = 0;
    let home = deploy(&mut c, &mut clock);
    let w = world(&c);
    let mines = mine_cells(w);
    let Some(&mine) = mines
        .iter()
        .min_by_key(|m| (m.x - home.x).abs().max((m.y - home.y).abs()))
    else {
        println!("no ore mine on the map");
        return;
    };
    println!(
        "ore mines {}, the nearest at {:?} (base at {:?}); tiberium runs {:?}",
        mines.len(),
        mine,
        home,
        w.rules()
            .econ
            .resource
            .tiberiums
            .iter()
            .map(|t| t.run)
            .collect::<Vec<_>>()
    );
    let (st, n) = ore_round(w, mine, 4);
    println!("start: resource cells within 4 of the mine {n}, stages {st:?}");
    look_at(&mut c, mine);
    let p = screen_of(&mut c, mine).unwrap_or(Point::new(640, 400));
    shots.save(&mut c, "n38-0-start");
    shots.zoom(&mut c, "n38-0-start", p, (300, 200, 3));

    // The map's own resource cells somewhere in sight, by stage.
    let w = world(&c);
    let mut all: BTreeMap<u8, usize> = BTreeMap::new();
    for (cell, _) in w.cells.grid.iter() {
        if let Some((_, d)) = resource::resource_at(w, cell) {
            *all.entry(d).or_default() += 1;
        }
    }
    println!("map resource cells by stage {all:?}");
    let mut stored: BTreeMap<u8, usize> = BTreeMap::new();
    let mut residues: BTreeMap<i32, usize> = BTreeMap::new();
    for (cell, x) in w.cells.grid.iter() {
        if let Some((0, _)) = resource::resource_at(w, cell) {
            *stored.entry(x.overlay).or_default() += 1;
            *residues.entry((cell.x * cell.y) % 12).or_default() += 1;
        }
    }
    println!("first tiberium's cells: stored overlay types {stored:?}; (x·y) mod 12 {residues:?}");

    // The first tiberium's run of bodies: each body sprite's frame count
    // and its frames' opaque pixel counts.
    {
        let s = c.session.as_ref().unwrap();
        // The overlay types play can put in a cell besides resource (walls,
        // crates): how many have their sprite read.
        let loaded = |idx: usize| {
            s.loaded
                .import
                .arts
                .get(&(glasnost_convert::kinds::Kind::Overlays, idx))
                .and_then(|a| a.sprite.as_deref())
                .is_some_and(|n| s.loaded.assets.get(n).is_some())
        };
        let ov = &w.rules().overlays;
        for (what, pick) in [
            (
                "wall",
                &(|o: &glasnost_sim::rules::OverlayRules| o.wall) as &dyn Fn(_) -> bool,
            ),
            ("crate", &|o: &glasnost_sim::rules::OverlayRules| o.crate_),
            ("tiberium", &|o: &glasnost_sim::rules::OverlayRules| {
                o.tiberium
            }),
        ] {
            let all: Vec<usize> = (0..ov.len()).filter(|&i| pick(&ov[i])).collect();
            println!(
                "{what} overlay types {}, sprite read {}",
                all.len(),
                all.iter().filter(|&&i| loaded(i)).count()
            );
        }
        let run = w.rules().econ.resource.tiberiums[0].run;
        if let Some([p, f, _]) = run {
            for idx in p..p + f {
                let sprite = s
                    .loaded
                    .import
                    .arts
                    .get(&(glasnost_convert::kinds::Kind::Overlays, idx as usize))
                    .and_then(|a| a.sprite.clone());
                let bytes = sprite.as_deref().and_then(|n| s.loaded.assets.get(n));
                let Some(shp) = bytes.and_then(|b| glasnost_fmt::shp::Shp::parse(b).ok()) else {
                    println!(
                        "  body {idx}: no sprite (art entry {}, sprite named {}, file read {})",
                        s.loaded
                            .import
                            .arts
                            .contains_key(&(glasnost_convert::kinds::Kind::Overlays, idx as usize)),
                        sprite.is_some(),
                        bytes.is_some()
                    );
                    continue;
                };
                let counts: Vec<usize> = (0..shp.frames.len())
                    .map(|i| {
                        shp.decode_frame(i)
                            .map_or(0, |im| im.pixels().iter().filter(|&&x| x != 0).count())
                    })
                    .collect();
                println!(
                    "  body {idx}: frames {} opaque {counts:?}",
                    shp.frames.len()
                );
            }
        }
    }

    // The same cells drawn by the live overlay path: every resource cell
    // within 8 of the mine loses its overlay for a tick and gets the very
    // same overlay and data back, so it is drawn again from the game's
    // cells instead of the cell pass.
    {
        let w = &mut c.session.as_mut().unwrap().play.game.world;
        let mut kept = Vec::new();
        for dy in -8..=8 {
            for dx in -8..=8 {
                let cc = CellXY::new(mine.x + dx, mine.y + dy);
                if resource::resource_at(w, cc).is_some()
                    && let Some(cell) = w.cells.grid.get_mut(cc)
                {
                    kept.push((cc, cell.overlay, cell.overlay_data));
                    cell.overlay = glasnost_map::NO_OVERLAY;
                    cell.overlay_data = 0;
                }
            }
        }
        println!("redrawn resource cells {}", kept.len());
        ticks(&mut c, &mut clock, 1);
        let w = &mut c.session.as_mut().unwrap().play.game.world;
        for (cc, o, d) in kept {
            if let Some(cell) = w.cells.grid.get_mut(cc) {
                cell.overlay = o;
                cell.overlay_data = d;
            }
        }
        ticks(&mut c, &mut clock, 1);
        look_at(&mut c, mine);
        shots.zoom(&mut c, "n38-0-redrawn", p, (300, 200, 3));
    }

    // Clear 3 cells round the mine.
    {
        let w = &mut c.session.as_mut().unwrap().play.game.world;
        for dy in -3..=3 {
            for dx in -3..=3 {
                resource::take_all(w, CellXY::new(mine.x + dx, mine.y + dy));
            }
        }
    }
    ticks(&mut c, &mut clock, 2);
    shots.zoom(&mut c, "n38-1-cleared", p, (300, 200, 3));

    // Watch it grow back.
    let watch = env_num("GLASNOST_UI_TICKS", 6000);
    let mut done = 0;
    while done < watch {
        ticks(&mut c, &mut clock, 1500);
        done += 1500;
        let (st, n) = ore_round(world(&c), mine, 3);
        println!("tick +{done}: resource cells within 3 of the mine {n}, stages {st:?}");
        look_at(&mut c, mine);
        shots.zoom(&mut c, &format!("n38-2-grown-{done}"), p, (300, 200, 3));
    }
    // Each grown cell: its overlay, stage, slope, whether a terrain object
    // stands there, and how many pixels its overlay shows on screen.
    look_at(&mut c, mine);
    let mut grown = Vec::new();
    {
        let w = world(&c);
        for dy in -3..=3 {
            for dx in -3..=3 {
                let cc = CellXY::new(mine.x + dx, mine.y + dy);
                if let (Some(cell), Some((t, d))) = (w.cells.cell(cc), resource::resource_at(w, cc))
                {
                    let obj = w
                        .objects
                        .values()
                        .any(|o| o.on_map && o.kind == Kind::Terrain && o.pos.cell() == cc);
                    grown.push((dx, dy, cc, cell.overlay, d, t, cell.slope, obj));
                }
            }
        }
    }
    for (dx, dy, cc, o, d, t, slope, obj) in grown {
        let shown = shown_pixels(&mut c, &mut clock, cc);
        println!(
            "  cell ({dx}, {dy}): overlay {o} stage {d} tiberium {t} slope {slope} terrain object {obj} pixels shown {shown}"
        );
    }

    // A strip of cells at each stage, set in play (live overlay path):
    // 12 clear flat cells in a screen row ((x + i, y − i)) near the mine,
    // none of them or their neighbours under the shroud; each cell's shown
    // pixels are counted.
    let local = c.session.as_ref().unwrap().play.me();
    let strip = {
        let w = world(&c);
        let open = |cc: CellXY| {
            w.cells.cell(cc).is_some_and(|x| {
                x.slope == 0
                    && x.overlay == glasnost_map::NO_OVERLAY
                    && [land::CLEAR, land::ROUGH].contains(&x.land)
            }) && w.objects.values().all(|o| !o.on_map || o.pos.cell() != cc)
                && (-1..=1).all(|ny| {
                    (-1..=1).all(|nx| {
                        !minimap::cell_shrouded(w, local, CellXY::new(cc.x + nx, cc.y + ny))
                    })
                })
        };
        let mut found = None;
        'f: for r in 2i32..24 {
            for dy in -r..=r {
                for dx in -r..=r {
                    let at = CellXY::new(mine.x + dx, mine.y + dy);
                    if (0..12).all(|i| open(CellXY::new(at.x + i, at.y - i))) {
                        found = Some(at);
                        break 'f;
                    }
                }
            }
        }
        found
    };
    println!("stage strip at {strip:?}");
    if let Some(at) = strip {
        let mid = CellXY::new(at.x + 6, at.y - 6);
        look_at(&mut c, mid);
        let p = screen_of(&mut c, mid).unwrap_or(Point::new(640, 400));
        shots.zoom(&mut c, "n38-3-strip-before", p, (400, 200, 3));
        {
            let w = &mut c.session.as_mut().unwrap().play.game.world;
            for i in 0..12u8 {
                let cc = CellXY::new(at.x + i32::from(i), at.y - i32::from(i));
                resource::add_resource_cell(w, cc, 0, i);
            }
        }
        ticks(&mut c, &mut clock, 1);
        look_at(&mut c, mid);
        for i in 0..12 {
            let cc = CellXY::new(at.x + i, at.y - i);
            let (o, d) = {
                let w = world(&c);
                (
                    w.cells.cell(cc).map(|x| x.overlay),
                    resource::resource_at(w, cc).map(|x| x.1),
                )
            };
            let shown = shown_pixels(&mut c, &mut clock, cc);
            println!("  strip cell {i}: overlay {o:?} stage {d:?} pixels shown {shown}");
        }
        // The live overlay items (after the cell pass): how many view pixels
        // they change as composed, and as composed without the depth test.
        {
            use glasnost_render::compose::{ComposeQuirks, DepthTest};
            use glasnost_render::scene::Group;
            let s = c.session.as_ref().unwrap();
            let view = s.view();
            let compose = |sc: &glasnost_render::scene::Scene| {
                sc.compose_lit(
                    &view,
                    s.filter(),
                    &ComposeQuirks::default(),
                    Some(s.light_alpha()),
                )
                .to_rgba()
            };
            let full = s.scene.clone();
            let is_live = |n: usize, it: &glasnost_render::scene::Item| {
                n >= full.cell_pass_len && it.group == Group::Overlays
            };
            let mut none = full.clone();
            none.items = full
                .items
                .iter()
                .enumerate()
                .filter(|(n, it)| !is_live(*n, it))
                .map(|(_, it)| *it)
                .collect();
            let mut free = full.clone();
            for (n, it) in free.items.iter_mut().enumerate() {
                if n >= full.cell_pass_len && it.group == Group::Overlays {
                    it.test = DepthTest::Ignore;
                }
            }
            println!("view origin {:?} size {:?}", view.origin, view.size);
            for (n, it) in full.items.iter().enumerate() {
                if is_live(n, it) {
                    let img = &full.images[it.image as usize];
                    let opaque = img.pixels().iter().filter(|&&p| p != 0).count();
                    println!(
                        "  live item image {} at ({}, {}) size {}x{} opaque {opaque}",
                        it.image,
                        it.x,
                        it.y,
                        img.width(),
                        img.height()
                    );
                }
            }
            let (a, b, f) = (compose(&full), compose(&none), compose(&free));
            let diff =
                |x: &[u8], y: &[u8]| x.chunks(4).zip(y.chunks(4)).filter(|(p, q)| p != q).count();
            println!(
                "live overlay items {}: pixels they change {}, without the depth test {}",
                full.items
                    .iter()
                    .enumerate()
                    .filter(|(n, it)| is_live(*n, it))
                    .count(),
                diff(&a, &b),
                diff(&f, &b)
            );
        }
        shots.save(&mut c, "n38-3-strip");
        shots.zoom(&mut c, "n38-3-strip", p, (400, 200, 3));
    }
}

/// Note 39: every vehicle type with a walking frame count or a body motion
/// file of more than one frame, placed and ordered to drive 8 cells; the
/// picture is compared from tick to tick while it drives.
#[test]
#[ignore = "local lane: needs GLASNOST_GAME_DIR"]
fn note_39_walking_vehicles() {
    let Some((mut c, shots)) = start((1280, 800)) else {
        eprintln!("GLASNOST_GAME_DIR not set: skipped");
        return;
    };
    let mut clock = 0;
    let home = deploy(&mut c, &mut clock);
    let local = c.session.as_ref().unwrap().play.me();
    // Candidates: vehicle types whose rules name the parasite weapon's
    // walking locomotor or any type whose art is a voxel body with a
    // motion file of several frames.
    let w = world(&c);
    let r = w.rules_arc();
    let mut picked: Vec<u32> = Vec::new();
    for (i, v) in r.vehicles.iter().enumerate() {
        let loco = v.raw.locomotor;
        let parasite = [v.raw.primary.as_ref(), v.raw.secondary.as_ref()]
            .into_iter()
            .flatten()
            .any(|wname| {
                r.weapons
                    .get(wname)
                    .and_then(|x| x.warhead.as_ref())
                    .and_then(|h| r.warheads.get(h))
                    .is_some_and(|h| h.parasite == Some(true))
            });
        if parasite || loco == Some(glasnost_rules::sim::Locomotor::Mech) {
            println!(
                "vehicle type {i}: parasite weapon {parasite}, locomotor {loco:?}, sim locomotor {:?}",
                v.locomotor
            );
            picked.push(i as u32);
        }
    }
    {
        let s = c.session.as_ref().unwrap();
        for &i in &picked {
            let art = s
                .loaded
                .import
                .arts
                .get(&(glasnost_convert::kinds::Kind::Vehicles, i as usize));
            println!(
                "  vehicle type {i}: voxel body {}, sprite {}",
                art.is_some_and(|a| a.voxels.contains_key("body")),
                art.is_some_and(|a| a.sprite.is_some())
            );
        }
    }
    for (k, &ti) in picked.iter().enumerate().take(4) {
        let at = CellXY::new(home.x + 4, home.y + 4 + 2 * k as i32);
        let id = {
            let w = &mut c.session.as_mut().unwrap().play.game.world;
            let name = w.rules().vehicles[ti as usize].name.clone();
            glasnost_sim::testkit::spawn(w, Kind::Vehicle, &name, local.0, at, 0)
        };
        let dest = CellXY::new(at.x + 8, at.y);
        let n =
            c.session
                .as_mut()
                .unwrap()
                .play
                .issue(vec![glasnost_sim::orders::Order::Behaviour {
                    actor: id,
                    behaviour: Behaviour::Move,
                    attack_move: false,
                    target: None,
                    dest: Some(Target::Cell(dest)),
                }]);
        println!("type {ti}: move orders taken {n}");
        // The body images the scene draws for the vehicle, tick by tick
        // (its items other than shadows), and the ticks it moves.
        let code = glasnost_app::mapview::live::sim_code(id);
        let mut images = std::collections::BTreeSet::new();
        let mut changes = 0;
        let mut last: Option<Vec<u32>> = None;
        let mut moved = 0;
        for t in 0..40 {
            let Some(o) = world(&c).obj(id) else { break };
            let (px, py) = (o.pos.x, o.pos.y);
            ticks(&mut c, &mut clock, 1);
            let Some(o) = world(&c).obj(id) else { break };
            let cell = o.pos.cell();
            look_at(&mut c, cell);
            let Some(p) = centre_of(&mut c, id) else {
                continue;
            };
            let drawn: Vec<u32> = c
                .session
                .as_ref()
                .unwrap()
                .scene
                .items
                .iter()
                .filter(|it| {
                    it.object == Some(code) && it.group != glasnost_render::scene::Group::Shadows
                })
                .map(|it| it.image)
                .collect();
            images.extend(drawn.iter().copied());
            if last.as_ref().is_some_and(|l| *l != drawn) {
                changes += 1;
            }
            last = Some(drawn);
            if t % 4 == 0 {
                shots.zoom(&mut c, &format!("n39-{ti}-t{t:02}"), p, (80, 80, 4));
            }
            let o = world(&c).obj(id).unwrap();
            if (o.pos.x, o.pos.y) != (px, py) {
                moved += 1;
            }
        }
        let o = world(&c).obj(id);
        println!(
            "type {ti}: ticks moving {moved} of 40, body image changes {changes}, distinct body images {}, cells driven {:?}",
            images.len(),
            o.map(|o| (o.pos.cell().x - at.x, o.pos.cell().y - at.y))
        );
    }
}
