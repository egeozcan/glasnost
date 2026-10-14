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
//! ordered unit is written during the move. The move target is an open
//! cell near 6 cells right and 4 down of the first unit (`open_target`).
//! `GLASNOST_LIVE_INDEX=i,j,…`
//! runs only the maps at those indices of the found list, and
//! `GLASNOST_LIVE_OUT` names a folder under `local-lane/` instead of
//! `screens`.

#![cfg(not(target_arch = "wasm32"))]

use std::rc::Rc;
use std::time::Instant;

use glasnost_app::mapview::{find, load, png};
use glasnost_render::compose::{ComposeQuirks, DEPTH_REFERENCE};
use glasnost_render::projection::{limit_centre, project_lifted, view_origin};
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

/// The output folder under `local-lane/` (`GLASNOST_LIVE_OUT`, default
/// `screens`).
fn out_dir() -> std::path::PathBuf {
    let sub = std::env::var("GLASNOST_LIVE_OUT").unwrap_or_else(|_| "screens".into());
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../local-lane")
        .join(sub);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// Whether map index `n` is wanted (`GLASNOST_LIVE_INDEX`, every map when
/// unset).
fn index_wanted(n: usize) -> bool {
    std::env::var("GLASNOST_LIVE_INDEX").map_or(true, |v| {
        v.split(',')
            .filter_map(|x| x.trim().parse::<usize>().ok())
            .any(|i| i == n)
    })
}

/// A move target near 6 cells right and 4 down of `from` (the lane's own
/// choice): the first cell, ring by ring, that is usable, has the
/// passability category of `from`, and whose 3 × 3 block and the cells
/// that are drawn in front of it hold no object, so the moved unit is
/// neither stopped short nor hidden behind trees or buildings.
fn open_target(w: &glasnost_sim::world::World, from: glasnost_map::CellXY) -> glasnost_map::CellXY {
    use glasnost_map::CellXY;
    use glasnost_sim::world::Layer;
    let base = CellXY::new(from.x + 6, from.y + 4);
    let category = w.cells.category(from);
    let open = |c: CellXY| {
        (-1..=2).all(|dx| {
            (-1..=2).all(|dy| {
                let n = CellXY::new(c.x + dx, c.y + dy);
                w.cells.usable(n)
                    && w.cells.category(n) == category
                    && w.cells.list(n, Layer::Ground).is_empty()
            })
        })
    };
    for r in 0..8i32 {
        for dy in -r..=r {
            for dx in -r..=r {
                if dx.abs().max(dy.abs()) != r {
                    continue;
                }
                let c = CellXY::new(base.x + dx, base.y + dy);
                if open(c) {
                    return c;
                }
            }
        }
    }
    base
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
    let c = limit_centre(centre, size, map.footprint.width, map.footprint.local);
    let origin = view_origin(c, size);
    View {
        origin,
        size: (size.0 as u32, size.1 as u32),
        reference: DEPTH_REFERENCE,
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
    let out_dir = out_dir();
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
            if !index_wanted(n)
                || (!filter.is_empty() && !m.name.to_ascii_lowercase().contains(&filter))
            {
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
                Default::default(),
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
            let target = open_target(&live.world, first);
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
                    );
                    let origin = view_origin(c, (640, 400));
                    let v = View {
                        origin,
                        size: (640, 400),
                        reference: DEPTH_REFERENCE,
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

/// Combat run: on each map, the closest pair of an armed unit and an enemy
/// unit of two houses that are not allied; up to 8 armed units of
/// the first house near it are selected through the input rules and ordered
/// to attack the enemy with a right click. The game runs
/// `GLASNOST_COMBAT_TICKS` ticks (default 360) and views are written before
/// the order, at the first visible projectile, 8 ticks later and at the end,
/// as `local-lane/screens/combat-<map index>-<tick>.png`, with the attackers
/// selected and the enemy hovered (their S-0111 health bars). Prints counts
/// and timings only. `GLASNOST_COMBAT_MAPS=n` (default 2) maps;
/// `GLASNOST_LIVE_FILTER` as above.
#[test]
fn live_maps_fight_and_draw() {
    use glasnost_sim::world::{Object, Pos};

    let Some(dir) = std::env::var_os("GLASNOST_GAME_DIR") else {
        eprintln!("GLASNOST_GAME_DIR not set: skipped");
        return;
    };
    let want = env_num("GLASNOST_COMBAT_MAPS", 2);
    let ticks = env_num("GLASNOST_COMBAT_TICKS", 360) as i32;
    let filter = std::env::var("GLASNOST_LIVE_FILTER")
        .unwrap_or_default()
        .to_ascii_lowercase();
    let out_dir = out_dir();
    pollster::block_on(async {
        let mut vfs = Vfs::default();
        vfs.mount_install(files(std::path::Path::new(&dir))).await;
        vfs.mount_startup(GameOptions::default()).await;
        let maps = find::find_maps(&vfs).await;
        let mut done = 0usize;
        let mut shots_total = 0u64;
        let mut armed_runs = 0usize;
        let mut written = Vec::new();
        for (n, m) in maps.iter().enumerate() {
            if done >= want {
                break;
            }
            if !index_wanted(n)
                || (!filter.is_empty() && !m.name.to_ascii_lowercase().contains(&filter))
            {
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
                Default::default(),
            )
            .await;
            let load_ms = t0.elapsed().as_millis();
            let Some(live) = loaded.live.as_ref() else {
                continue;
            };
            let w = &live.world;
            // Any owner will do: the sandbox lets the viewer control every
            // house.
            let player = |o: &Object| o.on_map && o.kind.is_mobile() && o.owner.is_some();
            // When the import gives no weapons at all (the importer does not
            // yet create weapons by reference, S-0004 claim 13), any unit
            // will do: the run then still shows the status extras.
            let weapons = live.rules.combat.weapons.len();
            let armed = |o: &Object| {
                weapons == 0
                    || live
                        .rules
                        .unit(o.kind, o.type_index)
                        .is_some_and(|u| u.combat.slots.iter().any(Option::is_some))
            };
            let dist = |a: Pos, b: Pos| i64::from(a.x - b.x).pow(2) + i64::from(a.y - b.y).pow(2);
            let mut best: Option<(i64, ObjectId, ObjectId)> = None;
            for a in w.objects.values().filter(|o| player(o) && armed(o)) {
                for b in w.objects.values().filter(|o| player(o)) {
                    if b.owner == a.owner || w.allied(a.owner, b.owner) {
                        continue;
                    }
                    let d = dist(a.pos, b.pos);
                    if best.is_none_or(|(bd, ..)| d < bd) {
                        best = Some((d, a.id, b.id));
                    }
                }
            }
            let Some((_, first, enemy)) = best else {
                continue;
            };
            let a0 = w.obj(first).unwrap();
            let house = a0.owner.unwrap();
            let attackers: Vec<ObjectId> = w
                .objects
                .values()
                .filter(|o| {
                    player(o)
                        && armed(o)
                        && o.owner == Some(house)
                        && dist(o.pos, a0.pos) <= (6 * 256i64).pow(2)
                })
                .map(|o| o.id)
                .take(8)
                .collect();
            done += 1;
            while loaded.live.as_ref().unwrap().world.tick < 4 {
                loaded.tick();
            }
            let live = loaded.live.as_mut().unwrap();
            let mut selection = Vec::new();
            for (i, &id) in attackers.iter().enumerate() {
                let mods = Modifiers {
                    force_select: i > 0,
                    ..Default::default()
                };
                live.click(house, &mut selection, ClickOn::Object(id), mods);
            }
            let report = live.click(
                house,
                &mut selection,
                ClickOn::Object(enemy),
                Modifiers::default(),
            );
            let mut focus: Vec<ObjectId> = attackers.clone();
            focus.push(enemy);
            let redraw = |loaded: &mut load::Loaded, selection: &mut Vec<ObjectId>| {
                let mut scene = std::mem::take(&mut loaded.built.scene);
                let (inputs, live) = loaded.split();
                let live = live.unwrap();
                live.prune(selection);
                let hovered = live.world.obj(enemy).map(|o| o.id);
                live.set_marks(selection, hovered);
                live.redraw(&inputs, &mut scene);
                loaded.built.scene = scene;
            };
            let shoot = |loaded: &load::Loaded, written: &mut Vec<String>| {
                let live = loaded.live.as_ref().unwrap();
                let v = view_at(&loaded.map, middle(&live.world, &focus));
                let f =
                    loaded
                        .built
                        .scene
                        .compose(&v, &Filter::default(), &ComposeQuirks::default());
                let name = format!("combat-{n}-{}.png", live.world.tick);
                std::fs::write(
                    out_dir.join(&name),
                    png::png(v.size.0, v.size.1, &f.to_rgba()),
                )
                .unwrap();
                written.push(name);
            };
            redraw(&mut loaded, &mut selection);
            shoot(&loaded, &mut written);
            let (mut step_us, mut redraw_us) = (0u128, 0u128);
            let mut seen_at: Option<i32> = None;
            for t in 0..ticks {
                let t0 = Instant::now();
                loaded.live.as_mut().unwrap().step();
                step_us += t0.elapsed().as_micros();
                let t0 = Instant::now();
                redraw(&mut loaded, &mut selection);
                redraw_us += t0.elapsed().as_micros();
                let live = loaded.live.as_ref().unwrap();
                let visible = live.world.projectiles.values().any(|p| {
                    live.rules
                        .combat
                        .projectile_of(p.weapon)
                        .is_none_or(|k| !k.inviso)
                });
                if seen_at.is_none() && (visible || live.counts.detonations > 0) {
                    seen_at = Some(t);
                    shoot(&loaded, &mut written);
                }
                if seen_at.is_some_and(|s| t == s + 8) {
                    shoot(&loaded, &mut written);
                }
            }
            shoot(&loaded, &mut written);
            let live = loaded.live.as_ref().unwrap();
            let c = live.counts;
            shots_total += c.shots;
            armed_runs += usize::from(weapons > 0);
            let kills: i32 = live.world.houses.iter().map(|h| h.combat.kills).sum();
            let losses: i32 = live.world.houses.iter().map(|h| h.combat.losses).sum();
            println!(
                "combat map {n} ({}): load {load_ms} ms; {weapons} weapons imported; {} attackers, {} selected, \
                 {} orders issued, {} refused; enemy {}; after {ticks} ticks: {} shots, \
                 {} detonations, {} animations, {} killed (houses: {kills} kills, {losses} losses), \
                 at most {} projectiles in flight, first seen after {:?} ticks; \
                 per tick: step {} µs, redraw {} µs; scene images {}",
                m.name,
                attackers.len(),
                selection.len(),
                report.issued,
                report.refused,
                if live.world.obj(enemy).is_some() {
                    "alive"
                } else {
                    "destroyed"
                },
                c.shots,
                c.detonations,
                c.anims,
                c.kills,
                c.most_in_flight,
                seen_at,
                step_us / ticks.max(1) as u128,
                redraw_us / ticks.max(1) as u128,
                loaded.built.scene.images.len(),
            );
        }
        for name in &written {
            println!("wrote {}", out_dir.join(name).display());
        }
        assert!(armed_runs == 0 || shots_total > 0, "no shot was fired");
    });
}
