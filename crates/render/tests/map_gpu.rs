//! The GPU map path draws a synthetic scene exactly as the CPU reference
//! compositor does (S-0102 claims 9 and 10, F1 to F3). Needs a GPU adapter
//! (software is fine); skipped without one unless `GLASNOST_REQUIRE_GPU=1`.

#![cfg(not(target_arch = "wasm32"))]

use glasnost_fmt::image::IndexedImage;
use glasnost_render::compose::{ComposeQuirks, DepthTest, Frame, Paint, Profile};
use glasnost_render::gpu::{self, BackendChoice};
use glasnost_render::map_gpu::{MapRenderer, targets};
use glasnost_render::offscreen::read_texture;
use glasnost_render::scene::{Filter, Group, Item, ItemDepth, Scene, View};

fn gpu_required() -> bool {
    std::env::var("GLASNOST_REQUIRE_GPU").is_ok_and(|v| v == "1")
}

fn device() -> Option<(wgpu::Device, wgpu::Queue)> {
    pollster::block_on(async {
        let instance = gpu::create_instance(BackendChoice::Auto, None).await;
        let adapter = match gpu::request_adapter(&instance, None).await {
            Ok(a) => a,
            Err(e) if !gpu_required() => {
                eprintln!("skipping: {e}");
                return None;
            }
            Err(e) => panic!("GLASNOST_REQUIRE_GPU=1 but {e}"),
        };
        Some(gpu::request_device(&adapter).await.expect("device opens"))
    })
}

/// An invented diamond-ish image with a gradient of indices.
fn image(w: u32, h: u32, seed: u8) -> IndexedImage {
    let mut px = Vec::new();
    for y in 0..h {
        for x in 0..w {
            let edge = (x + y) % 11 == 0;
            px.push(if edge {
                0
            } else {
                (seed as u32 + x * 3 + y * 5) as u8 | 1
            });
        }
    }
    IndexedImage::new(w, h, px).unwrap()
}

fn depth_image(w: u32, h: u32) -> IndexedImage {
    let px = (0..h)
        .flat_map(|y| (0..w).map(move |x| ((h - y) + x % 3) as u8))
        .collect();
    IndexedImage::new(w, h, px).unwrap()
}

fn scene() -> Scene {
    let mut s = Scene::default();
    let mut rows = Vec::new();
    for r in 0..3u16 {
        rows.push(std::array::from_fn(|i| {
            (i as u16).wrapping_mul(97).wrapping_add(r * 1000)
        }));
    }
    for r in rows {
        s.add_row(r);
    }
    let td = s.add_image(depth_image(60, 30));
    let tile = s.add_image(image(60, 30, 3));
    for k in 0..12 {
        let (x, y) = ((k % 4) * 30, (k / 4) * 15 + (k % 2) * 7);
        s.items.push(Item {
            image: tile,
            x,
            y,
            test: DepthTest::Tile,
            depth: ItemDepth::Tile {
                depth: Some(td),
                reference_row: y + (k % 3),
            },
            paint: Paint::Row(0),
            group: Group::Tiles,
            provisional: false,
            object: None,
        });
    }
    let tall = s.add_image(image(20, 50, 40));
    let flat = s.add_image(image(30, 12, 90));
    for (k, (test, profile, bias, x, y)) in [
        (DepthTest::Writer, Profile::Upright, -17, 20, 10),
        (DepthTest::Writer, Profile::Flat, -2, 45, 30),
        (DepthTest::Writer, Profile::Slope, -3, 70, 5),
        (DepthTest::Reader, Profile::Upright, -3, 30, 20),
        (DepthTest::Reader, Profile::Flat, -30, 60, 40),
        (DepthTest::Reader, Profile::Upright, 4, 90, 30),
    ]
    .into_iter()
    .enumerate()
    {
        s.items.push(Item {
            image: if profile == Profile::Flat { flat } else { tall },
            x,
            y,
            test,
            depth: ItemDepth::Profile {
                profile,
                bias,
                companion: None,
            },
            paint: Paint::Row(1 + k % 2),
            group: Group::Buildings,
            provisional: false,
            object: None,
        });
    }
    s
}

#[test]
fn gpu_matches_cpu_reference() {
    let Some((device, queue)) = device() else {
        return;
    };
    let s = scene();
    let mut r = MapRenderer::new(&device, &queue, &s).expect("scene uploads");
    for (origin, size) in [
        ((0, 0), (128, 72)),
        ((-7, 13), (100, 40)),
        ((5, -3), (64, 64)),
    ] {
        let view = View {
            origin,
            size,
            reference: Frame::reset_reference(origin.1),
        };
        let filter = Filter::default();
        let cpu = s
            .compose(&view, &filter, &ComposeQuirks::default())
            .to_rgba();
        let (color, depth) = targets(&device, size);
        let mut encoder = device.create_command_encoder(&Default::default());
        r.render(
            &device,
            &queue,
            &mut encoder,
            &color.create_view(&Default::default()),
            &depth.create_view(&Default::default()),
            &s,
            &view,
            &filter,
            true,
        );
        queue.submit([encoder.finish()]);
        let gpu = read_texture(&device, &queue, &color, size.0, size.1);
        let differing = cpu
            .chunks(4)
            .zip(gpu.chunks(4))
            .filter(|(a, b)| a[..3] != b[..3])
            .count();
        if differing != 0 {
            let w = size.0 as usize;
            let flipped = (0..size.1 as usize)
                .flat_map(|y| {
                    let row = &gpu[(size.1 as usize - 1 - y) * w * 4..][..w * 4];
                    row.to_vec()
                })
                .collect::<Vec<u8>>();
            let flip_diff = cpu
                .chunks(4)
                .zip(flipped.chunks(4))
                .filter(|(a, b)| a[..3] != b[..3])
                .count();
            let first = cpu
                .chunks(4)
                .zip(gpu.chunks(4))
                .position(|(a, b)| a[..3] != b[..3])
                .unwrap();
            eprintln!(
                "flipped diff {flip_diff}; first at {} cpu {:?} gpu {:?}",
                first,
                &cpu[first * 4..first * 4 + 4],
                &gpu[first * 4..first * 4 + 4]
            );
        }
        assert_eq!(differing, 0, "view at {origin:?} size {size:?}");
    }
}
