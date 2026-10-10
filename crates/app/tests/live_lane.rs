//! Local lane for the live map scene (never in CI): with `GLASNOST_GAME_DIR`
//! set, loads a few real maps with their sandbox game, selects some units of
//! the controlled house through the input rules, orders them to move, runs
//! the simulation and writes views before and after to
//! `local-lane/screens/` (git-ignored). Prints counts and timings only.
//! Without the variable it does nothing.
//!
//! `GLASNOST_LIVE_MAPS=n` (default 3) is the number of maps with units to
//! run, `GLASNOST_LIVE_TICKS=n` (default 240) the ticks after the order, and
//! `GLASNOST_LIVE_FILTER=text` keeps only maps whose name contains `text`,
//! `GLASNOST_LIVE_INFANTRY=1` only maps with infantry, and
//! `GLASNOST_LIVE_TURRETS=1` only maps with turreted vehicles (whose owner's
//! turreted vehicles are then ordered). A 640 × 400 close-up of the first
//! ordered unit is written during the move.

#![cfg(not(target_arch = "wasm32"))]

use std::rc::Rc;
use std::time::Instant;

use glasnost_app::mapview::{find, load, png};
use glasnost_render::compose::{ComposeQuirks, Frame};
use glasnost_render::projection::{ProjectionQuirks, limit_centre, project_lifted, view_origin};
use glasnost_render::scene::{Filter, View};
use glasnost_sim::targeting::{ClickOn, Modifiers};
use glasnost_sim::world::ObjectId;
use glasnost_vfs::source::{FileSource, Unopened};
use glasnost_vfs::{ByteSource, GameOptions, MountOptions, PickedFile, Vfs};

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

/// A 1280 × 720 view centred on world pixel `centre`, limited to the map.
fn view_at(map: &glasnost_map::Map, centre: (i32, i32)) -> View {
    let size = (1280, 720);
    let c = limit_centre(
        centre,
        size,
        map.footprint.width,
        map.footprint.local,
        &ProjectionQuirks::default(),
    );
    let origin = view_origin(c, size);
    View {
        origin,
        size: (size.0 as u32, size.1 as u32),
        reference: Frame::reset_reference(origin.1),
    }
}

/// The world pixel at the middle of the objects' positions.
fn middle(w: &glasnost_sim::world::World, ids: &[ObjectId]) -> (i32, i32) {
    let ps: Vec<_> = ids
        .iter()
        .filter_map(|&id| w.obj(id))
        .map(|o| o.pos)
        .collect();
    let n = ps.len().max(1) as i64;
    let x = ps.iter().map(|p| i64::from(p.x)).sum::<i64>() / n;
    let y = ps.iter().map(|p| i64::from(p.y)).sum::<i64>() / n;
    let z = ps.iter().map(|p| i64::from(p.z)).sum::<i64>() / n;
    project_lifted(x as i32, y as i32, z as i32)
}

#[test]
fn live_maps_move_and_draw() {
    let Some(dir) = std::env::var_os("GLASNOST_GAME_DIR") else {
        eprintln!("GLASNOST_GAME_DIR not set: skipped");
        return;
    };
    let want = env_num("GLASNOST_LIVE_MAPS", 3);
    let ticks = env_num("GLASNOST_LIVE_TICKS", 240) as i32;
    let need_infantry = std::env::var_os("GLASNOST_LIVE_INFANTRY").is_some();
    let need_turrets = std::env::var_os("GLASNOST_LIVE_TURRETS").is_some();
    let filter = std::env::var("GLASNOST_LIVE_FILTER")
        .unwrap_or_default()
        .to_ascii_lowercase();
    let out_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../local-lane/screens");
    std::fs::create_dir_all(&out_dir).unwrap();
    pollster::block_on(async {
        let mut vfs = Vfs::default();
        vfs.mount_install(files(std::path::Path::new(&dir))).await;
        vfs.mount_startup(GameOptions::default()).await;
        let maps = find::find_maps(&vfs).await;
        let mut done = 0usize;
        let mut moved_total = 0usize;
        for (n, m) in maps.iter().enumerate() {
            if done >= want {
                break;
            }
            if !filter.is_empty() && !m.name.to_ascii_lowercase().contains(&filter) {
                continue;
            }
            let bytes = vfs.read(&m.file).await.unwrap();
            let t0 = Instant::now();
            let mut loaded = load::load(
                &mut vfs,
                bytes,
                m.archive.clone(),
                &Default::default(),
                true,
            )
            .await;
            let load_ms = t0.elapsed().as_millis();
            let Some(live) = loaded.live.as_ref() else {
                println!(
                    "map {n} ({}): no game: {:?}",
                    m.name,
                    loaded.problems.last()
                );
                continue;
            };
            // The house with the most infantry (else the viewer's default),
            // and up to 8 of its units near its first one.
            let mut infantry: std::collections::BTreeMap<_, usize> = Default::default();
            for o in live.world.objects.values() {
                if o.kind == glasnost_sim::rules::Kind::Infantry && o.on_map {
                    *infantry.entry(o.owner).or_default() += 1;
                }
            }
            let turreted = |o: &glasnost_sim::world::Object| {
                o.kind == glasnost_sim::rules::Kind::Vehicle
                    && live
                        .rules
                        .unit(o.kind, o.type_index)
                        .is_some_and(|u| u.turret)
            };
            let mut turrets: std::collections::BTreeMap<_, usize> = Default::default();
            for o in live.world.objects.values() {
                if turreted(o) && o.on_map {
                    *turrets.entry(o.owner).or_default() += 1;
                }
            }
            let by = if need_turrets { &turrets } else { &infantry };
            let house = by
                .iter()
                .max_by_key(|(_, n)| **n)
                .and_then(|(h, _)| *h)
                .unwrap_or_else(|| live.default_house());
            let mobiles: Vec<_> = live
                .world
                .objects
                .values()
                .filter(|o| o.kind.is_mobile() && o.owner == Some(house) && o.on_map)
                .filter(|o| !need_turrets || turreted(o))
                .collect();
            let near = |a: glasnost_sim::world::Pos, b: glasnost_sim::world::Pos| {
                (a.x - b.x).abs() <= 8 * 256 && (a.y - b.y).abs() <= 8 * 256
            };
            let own: Vec<ObjectId> = mobiles
                .iter()
                .filter(|o| near(o.pos, mobiles[0].pos))
                .map(|o| o.id)
                .take(8)
                .collect();
            if own.is_empty()
                || (need_infantry && infantry.is_empty())
                || (need_turrets && turrets.is_empty())
            {
                continue;
            }
            done += 1;
            let objects = live.world.objects.len();
            let items = loaded.built.scene.items.len();
            // Past the order block (S-0201 claim 14).
            while loaded.live.as_ref().unwrap().world.tick < 4 {
                loaded.tick();
            }
            let live = loaded.live.as_mut().unwrap();
            // Select through the input rules: a click, then force-select clicks.
            let mut selection = Vec::new();
            for (i, &id) in own.iter().enumerate() {
                let mods = Modifiers {
                    force_select: i > 0,
                    ..Default::default()
                };
                live.click(house, &mut selection, ClickOn::Object(id), mods);
            }
            let first = live.world.obj(own[0]).unwrap().pos.cell();
            let target = glasnost_map::CellXY::new(first.x + 6, first.y + 4);
            let report = live.click(
                house,
                &mut selection,
                ClickOn::Cell(target),
                Modifiers::default(),
            );
            let starts: Vec<_> = own
                .iter()
                .map(|&id| live.world.obj(id).unwrap().pos)
                .collect();
            let before_view = view_at(&loaded.map, middle(&live.world, &own));
            let before = loaded.built.scene.compose(
                &before_view,
                &Filter::default(),
                &ComposeQuirks::default(),
            );
            let (mut step_us, mut redraw_us) = (0u128, 0u128);
            let mut during = None;
            for t in 0..ticks {
                if t == 24 {
                    let live = loaded.live.as_ref().unwrap();
                    let p = live.world.obj(own[0]).map(|o| o.pos).unwrap_or_default();
                    let c = limit_centre(
                        project_lifted(p.x, p.y, p.z),
                        (640, 400),
                        loaded.map.footprint.width,
                        loaded.map.footprint.local,
                        &ProjectionQuirks::default(),
                    );
                    let origin = view_origin(c, (640, 400));
                    let v = View {
                        origin,
                        size: (640, 400),
                        reference: Frame::reset_reference(origin.1),
                    };
                    let f = loaded.built.scene.compose(
                        &v,
                        &Filter::default(),
                        &ComposeQuirks::default(),
                    );
                    std::fs::write(
                        out_dir.join(format!("live-{n}-closeup.png")),
                        png::png(640, 400, &f.to_rgba()),
                    )
                    .unwrap();
                    let v = view_at(&loaded.map, middle(&live.world, &own));
                    during = Some((
                        v,
                        loaded.built.scene.compose(
                            &v,
                            &Filter::default(),
                            &ComposeQuirks::default(),
                        ),
                    ));
                }
                let t = Instant::now();
                loaded.live.as_mut().unwrap().step();
                step_us += t.elapsed().as_micros();
                let t = Instant::now();
                let mut scene = std::mem::take(&mut loaded.built.scene);
                let (inputs, live) = loaded.split();
                live.unwrap().redraw(&inputs, &mut scene);
                loaded.built.scene = scene;
                redraw_us += t.elapsed().as_micros();
            }
            let live = loaded.live.as_ref().unwrap();
            let moved = own
                .iter()
                .zip(&starts)
                .filter(|(id, p)| live.world.obj(**id).is_some_and(|o| o.pos != **p))
                .count();
            moved_total += moved;
            let t = Instant::now();
            let after_same = loaded.built.scene.compose(
                &before_view,
                &Filter::default(),
                &ComposeQuirks::default(),
            );
            let compose_ms = t.elapsed().as_millis();
            let after_view = view_at(&loaded.map, middle(&live.world, &own));
            let after = loaded.built.scene.compose(
                &after_view,
                &Filter::default(),
                &ComposeQuirks::default(),
            );
            let size = before_view.size;
            if let Some((v, f)) = &during {
                std::fs::write(
                    out_dir.join(format!("live-{n}-during.png")),
                    png::png(v.size.0, v.size.1, &f.to_rgba()),
                )
                .unwrap();
            }
            for (name, f) in [
                ("before", &before),
                ("after", &after_same),
                ("after-follow", &after),
            ] {
                std::fs::write(
                    out_dir.join(format!("live-{n}-{name}.png")),
                    png::png(size.0, size.1, &f.to_rgba()),
                )
                .unwrap();
            }
            println!(
                "map {n} ({}, {}): {objects} objects, {items} items, load {load_ms} ms; \
                 house {}: {} selected, {} orders issued, {} refused; \
                 after {ticks} ticks {moved} of {} moved; \
                 per tick: step {} µs, redraw {} µs; CPU compose {compose_ms} ms; \
                 {} problems, {} game notes, {} drawing notes, scene images {}",
                m.name,
                m.theater,
                live.world.houses[usize::from(house.0)].name,
                selection.len(),
                report.issued,
                report.refused,
                own.len(),
                step_us / ticks.max(1) as u128,
                redraw_us / ticks.max(1) as u128,
                loaded.problems.len(),
                live.world.notes.len(),
                live.notes.len(),
                loaded.built.scene.images.len(),
            );
            for &id in &own {
                let o = live.world.obj(id).unwrap();
                println!(
                    "  object {} {}: cell ({}, {}), behaviour {:?}, destination {:?}",
                    id.0,
                    glasnost_app::mapview::live::type_name(&live.rules, o.kind, o.type_index),
                    o.pos.cell().x,
                    o.pos.cell().y,
                    o.beh.current,
                    o.dest
                );
            }
            for p in &loaded.problems {
                println!("  problem: {p}");
            }
            for (k, v) in &loaded.built.dropped {
                println!("  dropped: {k}: {v}");
            }
        }
        println!("wrote screenshots to {}", out_dir.display());
        assert!(done == 0 || moved_total > 0, "no unit moved");
    });
}
