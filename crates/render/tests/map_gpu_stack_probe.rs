//! A timing probe for tiled animations on the GPU map path (design: render
//! (#127)): how long a frame's instance walk takes with stacks whose lowest
//! copy is far below the view's top. Synthetic data; ignored, run by hand in
//! release with `--ignored --nocapture`. Needs a GPU adapter.

#![cfg(not(target_arch = "wasm32"))]

use glasnost_fmt::image::IndexedImage;
use glasnost_render::compose::{ComposeQuirks, DEPTH_REFERENCE, DepthTest, Paint, Profile};
use glasnost_render::gpu;
use glasnost_render::map_gpu::{COLOR_FORMAT, MapRenderer, targets};
use glasnost_render::scene::{Filter, Group, Item, ItemDepth, Scene, Stack, View};
use std::time::Instant;

/// design: render (#127): 20 stacks of 8-row spacing whose lowest copies are 6 000 rows below
/// the view's top (S-0109 claim 27: 752 copies each), 10 beside the view; the mean time of a
/// frame's `render` call over 40 frames.
#[test]
#[ignore = "timing probe; run by hand in release"]
fn stack_probe() {
    let g = gpu::shared_device().expect("a GPU adapter");
    let (device, queue) = (g.device, g.queue);
    let mut s = Scene::default();
    s.add_row(std::array::from_fn(|i| (i as u16).wrapping_mul(131)));
    let px = (0..20 * 16).map(|k| (k % 200 + 1) as u8).collect();
    let image = s.add_image(IndexedImage::new(20, 16, px).unwrap());
    for k in 0..20 {
        let x = if k < 10 { k * 60 } else { 5000 + k * 60 };
        s.items.push(Item {
            image,
            x,
            y: 6000,
            test: DepthTest::Writer,
            depth: ItemDepth::Profile {
                profile: Profile::Flat,
                bias: 0,
                companion: None,
            },
            paint: Paint::Row(0),
            group: Group::Effects,
            provisional: false,
            object: None,
            stack: Some(Stack {
                spacing: 8,
                base_row: 6000,
            }),
        });
    }
    let size = (640, 360);
    let view = View {
        origin: (0, 0),
        size,
        reference: DEPTH_REFERENCE,
    };
    let filter = Filter::default();
    let q = ComposeQuirks::default();
    let mut r = MapRenderer::new(&device, &queue, &s, COLOR_FORMAT).expect("scene uploads");
    let (color, depth) = targets(&device, size, COLOR_FORMAT);
    let depth = depth.create_view(&Default::default());
    let frames = 40;
    let mut spent = std::time::Duration::ZERO;
    let mut items = 0;
    for frame in 0..frames + 5 {
        r.sync(&device, &queue, &s).expect("the scene syncs");
        let mut encoder = device.create_command_encoder(&Default::default());
        let _ = r.take_stats();
        let start = Instant::now();
        r.render(
            &device,
            &queue,
            &mut encoder,
            &color,
            &depth,
            &s,
            &view,
            &filter,
            &q,
        );
        let took = start.elapsed();
        items = r.take_stats().items;
        queue.submit([encoder.finish()]);
        if frame >= 5 {
            spent += took;
        }
    }
    let mean = spent.as_secs_f64() * 1000.0 / f64::from(frames);
    println!("stack probe: 20 stacks, {items} items visited a frame; render {mean:.3} ms a frame");
}
