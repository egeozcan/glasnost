//! Spot groups laid out on the GPU map path when a view first meets them
//! (design: render (#117)): a group beyond the view is not laid out or
//! uploaded until a view meets it, and then draws as the CPU reference does
//! with the spots it has then (S-0518 claims 20 to 23, F7 and F8). Needs a
//! GPU adapter (software is fine); skipped without one unless
//! `GLASNOST_REQUIRE_GPU=1`.

#![cfg(not(target_arch = "wasm32"))]

use glasnost_fmt::image::IndexedImage;
use glasnost_render::compose::{ComposeQuirks, DEPTH_REFERENCE, DepthTest, Paint, Profile};
use glasnost_render::gpu;
use glasnost_render::map_gpu::{COLOR_FORMAT, MapRenderer, MapStats, targets};
use glasnost_render::offscreen::read_texture;
use glasnost_render::scene::{Filter, Group, Item, ItemDepth, Scene, View};
use glasnost_render::spots::{Spot, SpotGroup, SpotKind};

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

/// Additive pixels along row `y`, each drawn twice.
fn readers(y: i32, xs: std::ops::Range<i32>, seed: u8) -> SpotGroup {
    let mut spots = Vec::new();
    for x in xs {
        for n in 0..2u8 {
            spots.push(Spot {
                x,
                y,
                bias: -40,
                kind: SpotKind::Additive {
                    rgb: [(x * 37) as u8, seed, n * 90 + 20],
                },
            });
        }
    }
    SpotGroup {
        at: usize::MAX,
        group: Group::Effects,
        provisional: false,
        spots,
    }
}

fn draw(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    r: &mut MapRenderer,
    s: &Scene,
    origin: (i32, i32),
) -> (usize, MapStats) {
    let view = View {
        origin,
        size: SIZE,
        reference: DEPTH_REFERENCE,
    };
    let filter = Filter::default();
    let q = ComposeQuirks::default();
    let cpu = s.compose_lit(&view, &filter, &q, None).to_rgba();
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
        &filter,
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
    (differing, stats)
}

/// design: render (#117): a group beyond the first view, changed while beyond it, draws as the
/// CPU reference does once a view meets it, and only its own instances are uploaded then; the
/// group the first view met is not uploaded again.
#[test]
fn a_group_beyond_the_view_is_laid_out_when_a_view_meets_it() {
    let Some((device, queue)) = device() else {
        return;
    };
    let mut s = ground();
    s.spots.push(readers(10, 4..40, 1));
    s.spots.push(readers(30, 90..120, 2));
    let mut r = MapRenderer::new(&device, &queue, &s, COLOR_FORMAT).expect("scene uploads");
    let (differing, first) = draw(&device, &queue, &mut r, &s, (0, 0));
    assert_eq!(differing, 0, "the first view: as the CPU draws it");
    assert!(first.upload_bytes > 0, "the first frame uploads");
    // The far group changes while no view meets it.
    s.spots[1] = readers(30, 92..124, 3);
    let (differing, stats) = draw(&device, &queue, &mut r, &s, (0, 0));
    assert_eq!(differing, 0, "the same view: as the CPU draws it");
    assert_eq!(stats.upload_bytes, 0, "nothing in the view changed");
    // View x 30 to 94, y 5 to 37: both groups' pixels.
    let (differing, stats) = draw(&device, &queue, &mut r, &s, (30, 5));
    assert_eq!(
        differing, 0,
        "a view meeting both groups: as the CPU draws it"
    );
    assert_eq!(
        stats.upload_bytes,
        64 * 24,
        "the far group's instances alone: 32 pixels drawn twice, 24 bytes a spot"
    );
    let (differing, stats) = draw(&device, &queue, &mut r, &s, (0, 0));
    assert_eq!(differing, 0, "back to the first view");
    assert_eq!(stats.upload_bytes, 0, "both groups held");
}
