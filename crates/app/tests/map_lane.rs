//! Local lane for the map viewer (never in CI): with `GLASNOST_GAME_DIR`
//! set, finds every map of the install, builds each one's scene, times a
//! full redraw and writes screenshots to `local-lane/` (git-ignored). It
//! prints counts and timings only. Without the variable it does nothing.
//!
//! `GLASNOST_MAP_LIMIT=n` builds only the first n maps; `GLASNOST_MAP_SHOTS`
//! (default 1) is the number of screenshots per theater.

#![cfg(not(target_arch = "wasm32"))]

use std::collections::BTreeMap;
use std::rc::Rc;
use std::time::Instant;

use glasnost_app::mapview::{build, find, load, png};
use glasnost_render::compose::{ComposeQuirks, Frame};
use glasnost_render::projection::{
    ScrollQuirks, cell_centre, limit_centre, project_lifted, view_origin,
};
use glasnost_render::scene::{Filter, View};
use glasnost_render::wgpu;
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

/// A GPU device, when an adapter exists (any backend `WGPU_BACKEND` picks).
fn gpu() -> Option<(wgpu::Device, wgpu::Queue, String)> {
    use glasnost_render::gpu::{self, BackendChoice};
    pollster::block_on(async {
        let instance = gpu::create_instance(BackendChoice::Auto, None).await;
        let adapter = gpu::request_adapter(&instance, None).await.ok()?;
        let info = adapter.get_info();
        let label = format!("{} on {}", gpu::backend_label(info.backend), info.name);
        let (d, q) = gpu::request_device(&adapter).await.ok()?;
        Some((d, q, label))
    })
}

/// Milliseconds per frame of `frames` GPU redraws of a 1280 × 720 view
/// scrolling 4 pixels per frame from `origin`, waiting for the GPU at the end.
fn gpu_scroll_ms(
    gpu: &(wgpu::Device, wgpu::Queue, String),
    scene: &glasnost_render::scene::Scene,
    origin: (i32, i32),
    frames: u32,
) -> f64 {
    use glasnost_render::map_gpu::{COLOR_FORMAT, MapRenderer, targets};
    let (device, queue, _) = gpu;
    let mut r = MapRenderer::new(device, queue, scene, COLOR_FORMAT).unwrap();
    let size = (1280, 720);
    let (color, depth) = targets(device, size, COLOR_FORMAT);
    let (cv, dv) = (
        color.create_view(&Default::default()),
        depth.create_view(&Default::default()),
    );
    let reference = Frame::reset_reference(origin.1);
    let t = Instant::now();
    for i in 0..frames {
        let view = View {
            origin: (origin.0 + 4 * i as i32, origin.1 + 2 * i as i32),
            size,
            reference,
        };
        let mut enc = device.create_command_encoder(&Default::default());
        r.render(
            device,
            queue,
            &mut enc,
            &cv,
            &dv,
            scene,
            &view,
            &Filter::default(),
            true,
        );
        queue.submit([enc.finish()]);
    }
    device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    t.elapsed().as_secs_f64() * 1000.0 / f64::from(frames)
}

#[test]
fn map_lane() {
    let Some(dir) = std::env::var_os("GLASNOST_GAME_DIR") else {
        return;
    };
    let limit: usize = std::env::var("GLASNOST_MAP_LIMIT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(usize::MAX);
    let shots_per_theater: usize = std::env::var("GLASNOST_MAP_SHOTS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(1);
    let out_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../local-lane");
    std::fs::create_dir_all(&out_dir).unwrap();
    pollster::block_on(async {
        let mut vfs = Vfs::default();
        vfs.mount_install(files(std::path::Path::new(&dir))).await;
        vfs.mount_startup(GameOptions::default()).await;
        let maps = find::find_maps(&vfs).await;
        let multiplayer = maps.iter().filter(|m| m.multiplayer).count();
        println!(
            "maps found: {} ({multiplayer} with start waypoints 0 and 1)",
            maps.len()
        );
        let mut ok = 0usize;
        let mut panics = 0usize;
        let mut shots: BTreeMap<String, usize> = BTreeMap::new();
        let mut load_ms = Vec::new();
        let mut redraw_ms = Vec::new();
        let mut items = 0usize;
        let mut notes: BTreeMap<String, usize> = BTreeMap::new();
        let gpu = gpu();
        let mut gpu_ms: Vec<f64> = Vec::new();
        let mut dropped: BTreeMap<String, usize> = BTreeMap::new();
        for (n, m) in maps.iter().enumerate().take(limit) {
            let bytes = vfs.read(&m.file).await.unwrap();
            let t0 = Instant::now();
            let options = build::Options {
                skirmish: std::env::var_os("GLASNOST_MAP_ALL_OWNERS").is_none(),
                ..build::Options::default()
            };
            let loaded = {
                let fut = load::load(&mut vfs, bytes, m.archive.clone(), &options);
                // A panic inside the load is counted, not propagated.
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| pollster::block_on(fut)))
            };
            let Ok(loaded) = loaded else {
                panics += 1;
                continue;
            };
            load_ms.push(t0.elapsed().as_millis());
            ok += 1;
            items += loaded.built.scene.items.len();
            for k in &loaded.built.notes {
                *notes
                    .entry(k.split(':').next().unwrap_or("").to_owned())
                    .or_default() += 1;
            }
            for (k, v) in &loaded.built.dropped {
                *dropped.entry(k.clone()).or_default() += v;
            }
            // A full redraw of a 1280 × 720 view at the first start waypoint (or home).
            let map = &loaded.map;
            let wp = map
                .waypoints
                .get(&0)
                .or_else(|| map.waypoints.get(&map.home_waypoint));
            let centre = wp.map_or((0, 0), |&(x, y)| {
                let (lx, ly) = cell_centre(x.into(), y.into());
                project_lifted(lx, ly, 0)
            });
            let size = (1280, 720);
            let centre = limit_centre(
                centre,
                size,
                map.footprint.width,
                map.footprint.local,
                &ScrollQuirks::default(),
            );
            let origin = view_origin(centre, size);
            let view = View {
                origin,
                size: (size.0 as u32, size.1 as u32),
                reference: Frame::reset_reference(origin.1),
            };
            let t1 = Instant::now();
            let frame =
                loaded
                    .built
                    .scene
                    .compose(&view, &Filter::default(), &ComposeQuirks::default());
            redraw_ms.push(t1.elapsed().as_millis());
            if let Some(g) = &gpu
                && gpu_ms.len() < 10
            {
                gpu_ms.push(gpu_scroll_ms(g, &loaded.built.scene, origin, 120));
            }
            let theater = m.theater.to_ascii_uppercase();
            let count = shots.entry(theater.clone()).or_default();
            if *count < shots_per_theater && m.multiplayer {
                *count += 1;
                let path = out_dir.join(format!("map-{theater}-{n}.png"));
                std::fs::write(&path, png::png(view.size.0, view.size.1, &frame.to_rgba()))
                    .unwrap();
                let (w, h, mm) = &loaded.minimap;
                std::fs::write(
                    out_dir.join(format!("map-{theater}-{n}-minimap.png")),
                    png::png(*w, *h, mm),
                )
                .unwrap();
                println!(
                    "wrote {} ({} items, {} problems)",
                    path.display(),
                    loaded.built.scene.items.len(),
                    loaded.problems.len()
                );
                // A second shot centred on the first vehicle or infantry unit drawn.
                let objects = &loaded.built.objects;
                if let Some(o) = objects
                    .iter()
                    .find(|o| o.kind == "vehicle")
                    .or_else(|| objects.iter().find(|o| o.kind == "infantry"))
                {
                    let (lx, ly) = cell_centre(o.cell.x, o.cell.y);
                    let c = limit_centre(
                        project_lifted(lx, ly, 0),
                        (640, 400),
                        map.footprint.width,
                        map.footprint.local,
                        &ScrollQuirks::default(),
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
                        out_dir.join(format!("map-{theater}-{n}-objects.png")),
                        png::png(640, 400, &f.to_rgba()),
                    )
                    .unwrap();
                }
                for p in &loaded.problems {
                    println!("  problem: {p}");
                }
            }
        }
        load_ms.sort_unstable();
        redraw_ms.sort_unstable();
        let median = |v: &Vec<u128>| v.get(v.len() / 2).copied().unwrap_or(0);
        let max = |v: &Vec<u128>| v.last().copied().unwrap_or(0);
        println!("maps loaded and built: {ok}, panics: {panics}");
        println!("scene items: {items} in total");
        println!(
            "load+build ms: median {}, max {}",
            median(&load_ms),
            max(&load_ms)
        );
        println!(
            "CPU full redraw (1280x720) ms: median {}, max {}",
            median(&redraw_ms),
            max(&redraw_ms)
        );
        if let Some(g) = &gpu {
            gpu_ms.sort_by(f64::total_cmp);
            println!(
                "GPU scrolling redraw (1280x720, {}): median {:.2} ms/frame, max {:.2} over {} maps",
                g.2,
                gpu_ms.get(gpu_ms.len() / 2).copied().unwrap_or(0.0),
                gpu_ms.last().copied().unwrap_or(0.0),
                gpu_ms.len()
            );
        }
        for (k, v) in &notes {
            println!("  note in {v} maps: {k}");
        }
        for (k, v) in &dropped {
            println!("  dropped objects: {k}: {v}");
        }
        assert_eq!(panics, 0);
    });
}
