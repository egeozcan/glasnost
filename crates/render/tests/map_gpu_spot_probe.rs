//! A timing probe of beam pixels on the GPU map path (design: render
//! (#117)), our own tooling: thirty thick beams over a 1920 × 1080 view,
//! most of them beyond it, drawn on frames whose view moves and spots stay
//! (a pan), frames whose spots change and view stays (a tick), and frames
//! where neither changes. It prints the mean recording time, the mean time
//! with the GPU's work, and the renderer's counts of each kind of frame.
//! It asserts nothing; ignored, run it with
//! `cargo test --release -p glasnost-render --test map_gpu_spot_probe -- --ignored --nocapture`.

#![cfg(not(target_arch = "wasm32"))]

use std::time::Instant;

use glasnost_fmt::image::IndexedImage;
use glasnost_render::compose::{ComposeQuirks, DEPTH_REFERENCE, DepthTest, Paint, Profile};
use glasnost_render::gpu;
use glasnost_render::map_gpu::{COLOR_FORMAT, MapRenderer, MapStats, targets};
use glasnost_render::scene::{Filter, Group, Item, ItemDepth, Scene, View};
use glasnost_render::spots::{Spot, SpotGroup, SpotKind, easing};

const SIZE: (u32, u32) = (1920, 1080);
/// The ground's tile side, and the tiles on each axis (the world is three
/// views wide and high, the view in the middle).
const TILE: i32 = 240;
const TILES: (i32, i32) = (24, 14);
const BEAMS: i32 = 30;
const FRAMES: usize = 100;

/// A ground of tiles over the world (depth writers).
fn ground() -> Scene {
    let mut s = Scene::default();
    let row = s.add_row(std::array::from_fn(|i| {
        (i as u16).wrapping_mul(2971).wrapping_add(0x4321)
    }));
    let px: Vec<u8> = (0..TILE * TILE)
        .map(|i| ((i * 7 + i / TILE * 13) % 255 + 1) as u8)
        .collect();
    let image = s.add_image(IndexedImage::new(TILE as u32, TILE as u32, px).unwrap());
    for ty in 0..TILES.1 {
        for tx in 0..TILES.0 {
            s.items.push(Item {
                image,
                x: tx * TILE - SIZE.0 as i32,
                y: ty * TILE - SIZE.1 as i32,
                test: DepthTest::Writer,
                depth: ItemDepth::Profile {
                    profile: Profile::Flat,
                    bias: 0,
                    companion: None,
                },
                paint: Paint::Row(row),
                group: Group::Tiles,
                provisional: false,
                object: None,
                stack: None,
            });
        }
    }
    s
}

/// Beam `b`: 300 steps of 7 pixels across, each pixel drawn twice, as
/// neighbouring samples of a thick beam are; additive and blended pixels,
/// coloured by `seed`.
fn beam(b: i32, seed: i32) -> SpotGroup {
    let x0 = (b * 997) % (3 * SIZE.0 as i32) - SIZE.0 as i32;
    let y0 = (b * 541) % (3 * SIZE.1 as i32) - SIZE.1 as i32;
    let mut spots = Vec::with_capacity(4200);
    for k in 0..300 {
        for w in -3..=3 {
            for at in [k, k - 1] {
                let (x, y) = (x0 + at, y0 + at / 2 + w);
                let rgb = [
                    ((at + seed) * 37 % 256) as u8,
                    ((w + 3) * 30) as u8,
                    (b * 8) as u8,
                ];
                let kind = if (at + w) % 2 == 0 {
                    SpotKind::Additive { rgb }
                } else {
                    let e = ((at as u32) * 29 + seed as u32) % 257;
                    SpotKind::Blended {
                        rgb,
                        line: easing(e),
                        screen: easing(256 - e),
                        alpha_from: (0, -1),
                    }
                };
                spots.push(Spot {
                    x,
                    y,
                    bias: -100,
                    kind,
                });
            }
        }
    }
    SpotGroup {
        at: usize::MAX,
        group: Group::Effects,
        provisional: false,
        spots,
    }
}

fn beams(s: &mut Scene, seed: i32) {
    s.spots = (0..BEAMS).map(|b| beam(b, seed)).collect();
}

#[test]
#[ignore = "a timing probe: run in release with --ignored --nocapture"]
fn probe_beam_pixels() {
    let g = match gpu::shared_device() {
        Ok(g) => g,
        Err(e) => {
            eprintln!("skipping: {e}");
            return;
        }
    };
    let (device, queue) = (g.device, g.queue);
    let mut s = ground();
    beams(&mut s, 0);
    let spots: usize = s.spots.iter().map(|g| g.spots.len()).sum();
    let mut r = MapRenderer::new(&device, &queue, &s, COLOR_FORMAT).expect("scene uploads");
    let (color, depth) = targets(&device, SIZE, COLOR_FORMAT);
    let depth = depth.create_view(&Default::default());
    let filter = Filter::default();
    let q = ComposeQuirks::default();
    println!("probe: {BEAMS} beams, {spots} spots, view {}x{}", SIZE.0, SIZE.1);
    for kind in ["pan", "tick", "still"] {
        let mut recording = 0.0;
        let mut total = 0.0;
        let mut sum = MapStats::default();
        for f in 0..FRAMES + 5 {
            let origin = match kind {
                "pan" => (((f as i32) * 4) % 400, ((f as i32) * 2) % 200),
                _ => (0, 0),
            };
            if kind == "tick" {
                beams(&mut s, f as i32 + 1);
            }
            let view = View {
                origin,
                size: SIZE,
                reference: DEPTH_REFERENCE,
            };
            let _ = r.take_stats();
            let start = Instant::now();
            let mut encoder = device.create_command_encoder(&Default::default());
            r.render_lit(
                &device, &queue, &mut encoder, &color, &depth, &s, &view, &filter, &q, None,
            );
            let recorded = start.elapsed().as_secs_f64() * 1000.0;
            queue.submit([encoder.finish()]);
            device
                .poll(wgpu::PollType::wait_indefinitely())
                .expect("the GPU finishes");
            let done = start.elapsed().as_secs_f64() * 1000.0;
            let st = r.take_stats();
            if f < 5 {
                continue;
            }
            recording += recorded;
            total += done;
            sum.instances += st.instances;
            sum.draws += st.draws;
            sum.passes += st.passes;
            sum.copies += st.copies;
            sum.uploads += st.uploads;
            sum.upload_bytes += st.upload_bytes;
        }
        let n = FRAMES as f64;
        println!(
            "probe {kind}: recording {:.3} ms, recording+GPU {:.3} ms; per frame: instances {:.0}, draws {:.1}, passes {:.1}, copies {:.1}, uploads {:.1}, upload bytes {:.0}",
            recording / n,
            total / n,
            sum.instances as f64 / n,
            sum.draws as f64 / n,
            sum.passes as f64 / n,
            sum.copies as f64 / n,
            sum.uploads as f64 / n,
            sum.upload_bytes as f64 / n,
        );
    }
}
