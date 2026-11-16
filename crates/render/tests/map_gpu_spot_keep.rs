//! Spot instances kept between frames on the GPU map path (design: render
//! (#117)): while a frame's spot groups are those of the last frame, their
//! instances are not built or uploaded again, whatever the view; any change
//! to a spot, a group or the filter draws as the CPU reference does (S-0518
//! claims 20 to 23, F7 and F8; S-0514 claim 25; S-0607 claim 25). Needs a
//! GPU adapter (software is fine); skipped without one unless
//! `GLASNOST_REQUIRE_GPU=1`.

#![cfg(not(target_arch = "wasm32"))]

use glasnost_fmt::image::IndexedImage;
use glasnost_render::compose::{ComposeQuirks, DEPTH_REFERENCE, DepthTest, Paint, Profile};
use glasnost_render::gpu;
use glasnost_render::map_gpu::{COLOR_FORMAT, MapRenderer, MapStats, targets};
use glasnost_render::offscreen::read_texture;
use glasnost_render::scene::{Filter, Group, Item, ItemDepth, Scene, View};
use glasnost_render::spots::{Spot, SpotGroup, SpotKind, easing};

fn device() -> Option<(wgpu::Device, wgpu::Queue)> {
    let required = std::env::var("GLASNOST_REQUIRE_GPU").is_ok_and(|v| v == "1");
    match gpu::shared_device() {
        Ok(g) => Some((g.device, g.queue)),
        Err(gpu::SharedGpuError::NoAdapter(e)) if !required => {
            eprintln!("skipping: {e} (set GLASNOST_REQUIRE_GPU=1 to make this fatal)");
            None
        }
        Err(gpu::SharedGpuError::NoAdapter(e)) => panic!("GLASNOST_REQUIRE_GPU=1 but {e}"),
        Err(e) => panic!("device opens: {e}"),
    }
}

/// The view's size; the ground is twice as wide and high.
const SIZE: (u32, u32) = (64, 32);

/// A ground of varied colours (a depth writer) under every view used here.
fn ground() -> Scene {
    let mut s = Scene::default();
    let row = s.add_row(std::array::from_fn(|i| {
        (i as u16).wrapping_mul(2971).wrapping_add(0x4321)
    }));
    let (w, h) = (SIZE.0 * 2, SIZE.1 * 2);
    let px = (0..h)
        .flat_map(|y| (0..w).map(move |x| ((x * 7 + y * 13) % 255 + 1) as u8))
        .collect();
    let image = s.add_image(IndexedImage::new(w, h, px).unwrap());
    s.items.push(Item {
        image,
        x: 0,
        y: 0,
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
    s
}

/// A thick beam along a slope: additive and blended pixels, each drawn
/// twice, with flashes over them, reaching past the first view's right
/// and lower edges.
fn beam(seed: u8) -> SpotGroup {
    let mut spots = Vec::new();
    for k in 0..90i32 {
        for w in 0..3 {
            for at in [k, k - 1] {
                let rgb = [(at * 37 % 256) as u8, seed, (w * 70 + 20) as u8];
                let kind = if (at + w) % 2 == 0 {
                    SpotKind::Additive { rgb }
                } else {
                    let e = (at.rem_euclid(257) as u32 * 29) % 257;
                    SpotKind::Blended {
                        rgb,
                        line: easing(e),
                        screen: easing(256 - e),
                        alpha_from: (0, -1),
                    }
                };
                spots.push(Spot {
                    x: at + 2,
                    y: 4 + at / 3 + w,
                    bias: [-200, -2, 30][(at.rem_euclid(3)) as usize],
                    kind,
                });
            }
        }
        spots.push(Spot {
            x: k + 2,
            y: 3 + k / 3,
            bias: -50,
            kind: SpotKind::Flash {
                rgb: [200, seed, 40],
            },
        });
    }
    SpotGroup {
        at: usize::MAX,
        group: Group::Effects,
        provisional: false,
        spots,
    }
}

/// A magnetic wave's band between the items (S-0607 claim 25: not drawn
/// within 3 pixels of the view's edges).
fn wave() -> SpotGroup {
    let mut spots = Vec::new();
    for y in 0..30 {
        for x in 0..40 {
            spots.push(Spot {
                x: x + 20,
                y: y + 1,
                bias: -100,
                kind: SpotKind::Wave {
                    shift: [(1, 0), (-2, 0), (0, 1), (0, -2)][((x + y) % 4) as usize],
                    k: 110 + 8 * ((x * y) % 10) as u16,
                },
            });
        }
    }
    SpotGroup {
        at: 1,
        group: Group::Effects,
        provisional: false,
        spots,
    }
}

/// Draws `s` with `r` in the view at `origin` and returns the pixels, how
/// many differ from the CPU reference, and the renderer's counts.
fn draw(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    r: &mut MapRenderer,
    s: &Scene,
    origin: (i32, i32),
    filter: &Filter,
) -> (Vec<u8>, usize, MapStats) {
    let view = View {
        origin,
        size: SIZE,
        reference: DEPTH_REFERENCE,
    };
    let q = ComposeQuirks::default();
    let cpu = s.compose_lit(&view, filter, &q, None).to_rgba();
    let (color, depth) = targets(device, SIZE, COLOR_FORMAT);
    let mut encoder = device.create_command_encoder(&Default::default());
    let _ = r.take_stats();
    r.render_lit(
        device,
        queue,
        &mut encoder,
        &color,
        &depth.create_view(&Default::default()),
        s,
        &view,
        filter,
        &q,
        None,
    );
    queue.submit([encoder.finish()]);
    let stats = r.take_stats();
    let gpu = read_texture(device, queue, &color, SIZE.0, SIZE.1);
    let differing = cpu
        .chunks(4)
        .zip(gpu.chunks(4))
        .filter(|(a, b)| a[..3] != b[..3])
        .count();
    (gpu, differing, stats)
}

/// design: render (#117): the same spot groups drawn in views that move take no upload after
/// the first frame, and every frame is the CPU reference's picture.
#[test]
fn the_spots_stay_uploaded_while_only_the_view_moves() {
    let Some((device, queue)) = device() else {
        return;
    };
    let mut s = ground();
    s.spots.push(beam(90));
    let mut r = MapRenderer::new(&device, &queue, &s, COLOR_FORMAT).expect("scene uploads");
    let filter = Filter::default();
    let (_, differing, _) = draw(&device, &queue, &mut r, &s, (0, 0), &filter);
    assert_eq!(differing, 0, "the first view: as the CPU draws it");
    for origin in [(9, 5), (30, 20), (-20, -6), (0, 0)] {
        let (_, differing, stats) = draw(&device, &queue, &mut r, &s, origin, &filter);
        assert_eq!(differing, 0, "view at {origin:?}: as the CPU draws it");
        assert_eq!(
            (stats.uploads, stats.upload_bytes),
            (0, 0),
            "view at {origin:?}: nothing uploaded again"
        );
    }
}

/// design: render (#117): a frame whose spots differ from the last frame's in one colour, one
/// place, one group more or less, or whose filter hides or shows them, draws them as the CPU
/// reference does; each change shows in the picture.
#[test]
fn every_change_of_the_spots_draws_anew() {
    let Some((device, queue)) = device() else {
        return;
    };
    let mut s = ground();
    s.spots.push(beam(90));
    let mut r = MapRenderer::new(&device, &queue, &s, COLOR_FORMAT).expect("scene uploads");
    let shown = Filter::default();
    let hidden = Filter {
        groups: vec![Group::Tiles],
        provisional: true,
    };
    let origin = (3, 2);
    let (mut last, differing, _) = draw(&device, &queue, &mut r, &s, origin, &shown);
    assert_eq!(differing, 0, "first frame");
    let mut step = |s: &Scene, filter: &Filter, what: &str| {
        let (px, differing, _) = draw(&device, &queue, &mut r, s, origin, filter);
        assert_eq!(differing, 0, "{what}: as the CPU draws it");
        let changed = px
            .chunks(4)
            .zip(last.chunks(4))
            .filter(|(a, b)| a != b)
            .count();
        assert!(changed > 0, "{what}: the change shows");
        last = px;
    };
    // One spot's colour, at the same count and places.
    // (The flash of step k is spot 7k + 6; no later spot of the beam
    // covers it.)
    s.spots[0].spots[7 * 10 + 6].kind = SpotKind::Flash { rgb: [10, 250, 10] };
    step(&s, &shown, "one colour");
    // One spot moved, to a pixel of the ground alone.
    s.spots[0].spots[7 * 30 + 6].y -= 1;
    step(&s, &shown, "one place");
    // A group more, between the items.
    s.spots.insert(0, wave());
    s.spots[0].at = 1;
    step(&s, &shown, "a wave added");
    // Hidden, then shown again.
    step(&s, &hidden, "hidden");
    step(&s, &shown, "shown");
    // A group less.
    s.spots.remove(1);
    step(&s, &shown, "the beam removed");
    // The same groups in a new list.
    let again = s.spots.clone();
    s.spots = again;
    let (px, differing, stats) = draw(&device, &queue, &mut r, &s, origin, &shown);
    assert_eq!(differing, 0, "the same groups again");
    assert!(px == last, "the same picture");
    assert_eq!(stats.upload_bytes, 0, "the same groups: nothing uploaded");
}
