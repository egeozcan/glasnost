//! A tiled animation's stack on the GPU map path (design: render (#127)): a
//! frame visits only the copies of a stack that meet the view's rows, and
//! none of a stack beside the view, however many copies the view's top row
//! gives it. Each frame draws as the CPU reference does, exactly (S-0109
//! claim 27; S-0102 claim 13; S-0101 F4b). Needs a GPU adapter (software is
//! fine); skipped without one unless `GLASNOST_REQUIRE_GPU=1`.

#![cfg(not(target_arch = "wasm32"))]

use glasnost_fmt::image::IndexedImage;
use glasnost_render::compose::{ComposeQuirks, DEPTH_REFERENCE, DepthTest, Paint, Profile};
use glasnost_render::gpu;
use glasnost_render::map_gpu::{COLOR_FORMAT, MapRenderer, targets};
use glasnost_render::offscreen::read_texture;
use glasnost_render::scene::{Filter, Group, Item, ItemDepth, Scene, Stack, View};

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

const SIZE: (u32, u32) = (128, 72);

/// An invented image with transparent pixels on a diagonal grid.
fn image(w: u32, h: u32, seed: u32) -> IndexedImage {
    let px = (0..h)
        .flat_map(|y| {
            (0..w).map(move |x| {
                if (x + y + seed).is_multiple_of(7) {
                    0
                } else {
                    ((seed * 23 + x * 5 + y * 3) % 255 + 1) as u8
                }
            })
        })
        .collect();
    IndexedImage::new(w, h, px).unwrap()
}

fn item(image: u32, (x, y): (i32, i32), bias: i32, stack: Option<Stack>) -> Item {
    Item {
        image,
        x,
        y,
        test: DepthTest::Writer,
        depth: ItemDepth::Profile {
            profile: Profile::Flat,
            bias,
            companion: None,
        },
        paint: Paint::Row(0),
        group: Group::Effects,
        provisional: false,
        object: None,
        stack,
    }
}

fn stack(spacing: i32, base_row: i32) -> Option<Stack> {
    Some(Stack { spacing, base_row })
}

/// A stack whose lowest copy is 6 000 rows below the view's top, the same
/// stack beside the view, a stack whose copies overlap one another, a stack
/// above the view, and a plain item.
fn scene() -> Scene {
    let mut s = Scene::default();
    s.add_row(std::array::from_fn(|i| (i as u16).wrapping_mul(131)));
    let tall = s.add_image(image(20, 16, 1));
    let thin = s.add_image(image(12, 20, 2));
    let plain = s.add_image(image(30, 10, 3));
    s.items.push(item(plain, (40, 30), -5, None));
    s.items.push(item(tall, (10, 6000), 0, stack(8, 6000)));
    s.items.push(item(tall, (1000, 6000), 0, stack(8, 6000)));
    s.items.push(item(thin, (60, 50), 10, stack(5, 50)));
    s.items.push(item(thin, (20, -100), 0, stack(5, -100)));
    s
}

/// Draws `s` through `r` at a view at `origin`; returns the pixels that
/// differ from the CPU reference and the items visited.
fn draw(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    r: &mut MapRenderer,
    s: &Scene,
    origin: (i32, i32),
) -> (usize, usize) {
    let view = View {
        origin,
        size: SIZE,
        reference: DEPTH_REFERENCE,
    };
    let filter = Filter::default();
    let q = ComposeQuirks::default();
    let cpu = s.compose(&view, &filter, &q).to_rgba();
    r.sync(device, queue, s).expect("the scene syncs");
    let (color, depth) = targets(device, SIZE, COLOR_FORMAT);
    let mut encoder = device.create_command_encoder(&Default::default());
    let _ = r.take_stats();
    r.render(
        device,
        queue,
        &mut encoder,
        &color,
        &depth.create_view(&Default::default()),
        s,
        &view,
        &filter,
        &q,
    );
    queue.submit([encoder.finish()]);
    let visited = r.take_stats().items;
    let gpu = read_texture(device, queue, &color, SIZE.0, SIZE.1);
    let differing = cpu
        .chunks(4)
        .zip(gpu.chunks(4))
        .filter(|(a, b)| a[..3] != b[..3])
        .count();
    (differing, visited)
}

/// design: render (#127): the copies a frame visits are those that meet the view's rows (a
/// stack beside the view: none), not every copy the view's top row gives the stack (S-0109
/// claim 27: 752 copies for the lowest copy 6 000 rows below the top); every frame as the CPU
/// draws it (S-0101 F4b: a copy whose bounds miss the view draws no pixel).
#[test]
fn a_frame_visits_only_the_stack_copies_that_meet_the_view() {
    let Some((device, queue)) = device() else {
        return;
    };
    let s = scene();
    let mut r = MapRenderer::new(&device, &queue, &s, COLOR_FORMAT).expect("scene uploads");
    let views = [
        (0, 0),
        (-30, 5900),
        (5, 3000),
        (0, -200),
        (50, 20),
        (950, 5950),
    ];
    for origin in views {
        let (differing, visited) = draw(&device, &queue, &mut r, &s, origin);
        assert_eq!(differing, 0, "view at {origin:?}: as the CPU draws it");
        assert!(
            visited <= 40,
            "view at {origin:?}: {visited} items visited for 5 items"
        );
    }
}
