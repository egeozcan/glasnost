//! The marks a spot group's layout leaves on the view's pixels on the GPU
//! map path (design: render (#115, #116)): they hold for that group only, so
//! a later group, a later frame, a view of another size and a group after
//! many others lay out as if drawn alone, with as many draws, passes and
//! copies, and the picture is the CPU reference's exactly (S-0518 claim 20,
//! F7). Needs a GPU adapter (software is fine); skipped without one unless
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

/// The ground's size, and the larger of the two view sizes; the other has
/// the same width, so a pixel keeps its place in a row-major mark buffer.
const SIZE: (u32, u32) = (96, 48);
const SMALLER: (u32, u32) = (96, 40);

/// A ground of varied colours over the whole view (a depth writer).
fn ground() -> Scene {
    let mut s = Scene::default();
    let row = s.add_row(std::array::from_fn(|i| {
        (i as u16).wrapping_mul(2971).wrapping_add(0x4321)
    }));
    let px = (0..SIZE.1)
        .flat_map(|y| (0..SIZE.0).map(move |x| ((x * 7 + y * 13) % 255 + 1) as u8))
        .collect();
    let image = s.add_image(IndexedImage::new(SIZE.0, SIZE.1, px).unwrap());
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

/// A group of additive pixels along row `y`, each drawn once.
fn readers(y: i32, xs: std::ops::Range<i32>, seed: i32) -> SpotGroup {
    SpotGroup {
        at: usize::MAX,
        group: Group::Effects,
        provisional: false,
        spots: xs
            .map(|x| Spot {
                x,
                y,
                bias: -40,
                kind: SpotKind::Additive {
                    rgb: [(x * 37 + seed) as u8, (x * 11) as u8, (seed * 50) as u8],
                },
            })
            .collect(),
    }
}

/// Draws `s` with `r` in a view of `size` and returns the pixels that
/// differ from the CPU reference and the renderer's counts.
fn draw(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    r: &mut MapRenderer,
    s: &Scene,
    size: (u32, u32),
) -> (usize, MapStats) {
    let view = View {
        origin: (0, 0),
        size,
        reference: DEPTH_REFERENCE,
    };
    let filter = Filter::default();
    let q = ComposeQuirks::default();
    let cpu = s.compose_lit(&view, &filter, &q, None).to_rgba();
    let (color, depth) = targets(device, size, COLOR_FORMAT);
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
    let gpu = read_texture(device, queue, &color, size.0, size.1);
    let differing = cpu
        .chunks(4)
        .zip(gpu.chunks(4))
        .filter(|(a, b)| a[..3] != b[..3])
        .count();
    (differing, stats)
}

/// design: render (#115, #116): two groups of additive pixels that share some pixels, drawn on
/// one renderer at one size, at another, and at the first again: each group is one layer, one
/// draw and one copy every time, as when the renderer is new.
#[test]
fn a_groups_marks_hold_for_that_group_only() {
    let Some((device, queue)) = device() else {
        return;
    };
    let mut s = ground();
    s.spots.push(readers(10, 10..40, 1));
    s.spots.push(readers(10, 20..60, 2));
    let mut r = MapRenderer::new(&device, &queue, &s, COLOR_FORMAT).expect("scene uploads");
    for (n, size) in [SIZE, SMALLER, SMALLER, SIZE].into_iter().enumerate() {
        let (differing, stats) = draw(&device, &queue, &mut r, &s, size);
        assert_eq!(differing, 0, "frame {n}: as the CPU draws them");
        assert_eq!(
            stats.draws, 3,
            "frame {n}: the ground's draw and a draw a group"
        );
        assert_eq!(stats.copies, 2, "frame {n}: a copy a group");
        assert_eq!(stats.passes, 3, "frame {n}: passes");
    }
}

/// design: render (#115, #116): 300 groups on one pixel, more than the marks have generations;
/// the first and every group from the 256th on also take the pixel beside it, which the groups
/// between leave alone: each group is one layer, one draw and one copy.
#[test]
fn marks_hold_past_their_generations() {
    let Some((device, queue)) = device() else {
        return;
    };
    let mut s = ground();
    for n in 0..300 {
        let xs = if n == 0 || n >= 255 { 5..7 } else { 5..6 };
        s.spots.push(readers(5, xs, n));
    }
    let mut r = MapRenderer::new(&device, &queue, &s, COLOR_FORMAT).expect("scene uploads");
    let (differing, stats) = draw(&device, &queue, &mut r, &s, SIZE);
    assert_eq!(differing, 0, "as the CPU draws them");
    assert_eq!(stats.draws, 301, "the ground's draw and a draw a group");
    assert_eq!(stats.copies, 300, "a copy a group");
    assert_eq!(stats.passes, 301, "passes");
}
