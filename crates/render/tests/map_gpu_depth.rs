//! The GPU map path's depth roles against the CPU reference compositor for
//! drawings that skip the depth test (#70, #71). Needs a GPU adapter
//! (software is fine); skipped without one unless `GLASNOST_REQUIRE_GPU=1`.

#![cfg(not(target_arch = "wasm32"))]

use glasnost_fmt::image::IndexedImage;
use glasnost_render::compose::{ComposeQuirks, DEPTH_REFERENCE, DepthTest, Paint, Profile};
use glasnost_render::gpu;
use glasnost_render::map_gpu::{MapRenderer, targets};
use glasnost_render::offscreen::read_texture;
use glasnost_render::scene::{Filter, Group, Item, ItemDepth, Scene, View};

fn gpu_required() -> bool {
    std::env::var("GLASNOST_REQUIRE_GPU").is_ok_and(|v| v == "1")
}

fn device() -> Option<(wgpu::Device, wgpu::Queue)> {
    match gpu::shared_device() {
        Ok(g) => Some((g.device, g.queue)),
        Err(gpu::SharedGpuError::NoAdapter(e)) if !gpu_required() => {
            eprintln!("skipping: {e} (set GLASNOST_REQUIRE_GPU=1 to make this fatal)");
            None
        }
        Err(gpu::SharedGpuError::NoAdapter(e)) => panic!("GLASNOST_REQUIRE_GPU=1 but {e}"),
        Err(e) => panic!("device opens: {e}"),
    }
}

/// Draws `s` at the origin with a view of `size` on the CPU reference and
/// on the GPU; returns both RGBA pictures.
fn cpu_and_gpu(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    s: &Scene,
    size: (u32, u32),
) -> (Vec<u8>, Vec<u8>) {
    let view = View {
        origin: (0, 0),
        size,
        reference: DEPTH_REFERENCE,
    };
    let q = ComposeQuirks::default();
    let cpu = s.compose(&view, &Filter::default(), &q).to_rgba();
    let mut r = MapRenderer::new(device, queue, s, glasnost_render::map_gpu::COLOR_FORMAT)
        .expect("scene uploads");
    let (color, depth) = targets(device, size, glasnost_render::map_gpu::COLOR_FORMAT);
    let mut encoder = device.create_command_encoder(&Default::default());
    r.render(
        device,
        queue,
        &mut encoder,
        &color,
        &depth.create_view(&Default::default()),
        s,
        &view,
        &Filter::default(),
        &q,
    );
    queue.submit([encoder.finish()]);
    let gpu = read_texture(device, queue, &color, size.0, size.1);
    (cpu, gpu)
}

/// A flat item of `image` at (`x`, 0) with depth bias `bias`.
fn flat_item(image: u32, x: i32, bias: i32, test: DepthTest, paint: Paint) -> Item {
    Item {
        image,
        x,
        y: 0,
        test,
        depth: ItemDepth::Profile {
            profile: Profile::Flat,
            bias,
            companion: None,
        },
        paint,
        group: Group::Buildings,
        provisional: false,
        object: None,
        stack: None,
    }
}

/// #70. spec: S-0102 claim 9 (a tile without depth data does not touch the depth buffer: it is
/// painted whatever the stored depth and stores none)
///
/// A nearer writer, then a tile without depth data over it and the open ground beside it, then
/// a flat reader over both: the tile covers the writer, and the reader is tested against the
/// writer's depth and the empty buffer, not against the tile's, on the GPU as on the CPU.
#[test]
fn a_tile_without_depth_data_ignores_the_depth_buffer() {
    let Some((device, queue)) = device() else {
        return;
    };
    let mut s = Scene::default();
    for r in 0..3u16 {
        s.add_row(std::array::from_fn(|i| {
            (i as u16).wrapping_mul(31).wrapping_add(r * 2000)
        }));
    }
    let writer = s.add_image(IndexedImage::new(16, 16, vec![1; 256]).unwrap());
    let tile = s.add_image(IndexedImage::new(32, 16, vec![2; 512]).unwrap());
    let reader = s.add_image(IndexedImage::new(32, 16, vec![3; 512]).unwrap());
    s.items
        .push(flat_item(writer, 0, -100, DepthTest::Writer, Paint::Row(0)));
    s.items.push(Item {
        image: tile,
        x: 0,
        y: 0,
        test: DepthTest::Tile,
        depth: ItemDepth::Tile {
            depth: None,
            reference_row: 0,
        },
        paint: Paint::Row(1),
        group: Group::Tiles,
        provisional: false,
        object: None,
        stack: None,
    });
    s.items
        .push(flat_item(reader, 0, 0, DepthTest::Reader, Paint::Row(2)));
    let (cpu, gpu) = cpu_and_gpu(&device, &queue, &s, (32, 16));
    let differing = cpu
        .chunks(4)
        .zip(gpu.chunks(4))
        .filter(|(a, b)| a[..3] != b[..3])
        .count();
    assert_eq!(differing, 0, "of {} pixels", 32 * 16);
}

/// #71. spec: S-0102 claim 25 (a drawing with no depth test neither reads nor writes depth),
/// S-0111 claim 16 (the self-heal blink pip: a shadow drawing with no depth test)
///
/// A shadow that ignores depth over a nearer writer darkens it, on the GPU as on the CPU (the
/// GPU halves expanded 8-bit channels, so channels are compared within a small tolerance).
#[test]
fn a_shadow_with_no_depth_test_darkens_over_a_nearer_writer() {
    let Some((device, queue)) = device() else {
        return;
    };
    let mut s = Scene::default();
    s.add_row(std::array::from_fn(|_| 0xFFFF));
    let ground = s.add_image(IndexedImage::new(16, 4, vec![1; 64]).unwrap());
    let shade = s.add_image(IndexedImage::new(8, 4, vec![1; 32]).unwrap());
    s.items
        .push(flat_item(ground, 0, -100, DepthTest::Writer, Paint::Row(0)));
    s.items
        .push(flat_item(shade, 4, 0, DepthTest::Ignore, Paint::Shadow));
    let (cpu, gpu) = cpu_and_gpu(&device, &queue, &s, (16, 4));
    assert!(cpu[4 * 4] < cpu[0], "the CPU darkens under the shadow");
    for (a, b) in cpu.chunks(4).zip(gpu.chunks(4)) {
        for k in 0..3 {
            assert!(a[k].abs_diff(b[k]) <= 8, "cpu {a:?} gpu {b:?}");
        }
    }
}
