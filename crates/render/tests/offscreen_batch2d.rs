//! Renders the 2D batch offscreen and compares every pixel with the CPU
//! reference. Needs a GPU adapter; skipped without one unless
//! `GLASNOST_REQUIRE_GPU=1` is set.

#![cfg(not(target_arch = "wasm32"))]

use std::collections::BTreeMap;

use glasnost_render::batch2d::{self, Atlas, Batch2d, CpuImage, Quad};
use glasnost_render::gpu::{self, BackendChoice};
use glasnost_render::offscreen::{OFFSCREEN_FORMAT, render_rgba};

fn gpu_required() -> bool {
    std::env::var("GLASNOST_REQUIRE_GPU").is_ok_and(|v| v == "1")
}

fn device() -> Option<(wgpu::Device, wgpu::Queue)> {
    pollster::block_on(async {
        let instance = gpu::create_instance(BackendChoice::Auto, None).await;
        let adapter = match gpu::request_adapter(&instance, None).await {
            Ok(a) => a,
            Err(e) if !gpu_required() => {
                eprintln!("skipping: {e} (set GLASNOST_REQUIRE_GPU=1 to make this fatal)");
                return None;
            }
            Err(e) => panic!("GLASNOST_REQUIRE_GPU=1 but {e}"),
        };
        Some(gpu::request_device(&adapter).await.expect("device opens"))
    })
}

fn palette(seed: u8) -> [u8; 1024] {
    let mut p = [0u8; 1024];
    for i in 0..256usize {
        p[i * 4] = (i as u8).wrapping_mul(3).wrapping_add(seed);
        p[i * 4 + 1] = (i as u8) ^ seed;
        p[i * 4 + 2] = 255 - i as u8;
        p[i * 4 + 3] = 255;
    }
    p
}

#[test]
fn batch_pixels_match_the_cpu_reference() {
    let Some((device, queue)) = device() else {
        return;
    };
    let (w, h) = (64u32, 48u32);
    let mut atlas = Atlas::default();
    let sprite: Vec<u8> = (0..12 * 10)
        .map(|i| if i % 7 == 0 { 0 } else { (i % 40 + 16) as u8 })
        .collect();
    let sr = atlas.add(12, 10, &sprite).unwrap();
    let glyph: Vec<u8> = (0..5 * 7).map(|i| u8::from(i % 3 == 0)).collect();
    let gr = atlas.add(5, 7, &glyph).unwrap();
    let rows = [palette(0), palette(77)];
    let img = CpuImage {
        w: 4,
        h: 3,
        rgba: (0..48)
            .map(|i| (i * 5) as u8 | 3)
            .collect::<Vec<u8>>()
            .chunks(4)
            .flat_map(|c| [c[0], c[1], c[2], 255])
            .collect(),
    };
    let mut images = BTreeMap::new();
    images.insert(7, img.clone());

    let mut quads = vec![
        Quad::solid([0, 0, 64, 48], [30, 40, 50, 255]),
        Quad::image(7, (4, 3), [40, 30, 8, 6]),
        Quad::sprite(3, 4, sr, 0),
        Quad::sprite(20, 4, sr, 1),
        Quad::glyph(40, 5, gr, [250, 200, 10, 255]),
        Quad::solid([10, 10, 30, 20], [0, 0, 255, 128]),
    ];
    quads.extend(batch2d::clip(Quad::sprite(50, 20, sr, 1), [0, 0, 56, 26]));
    quads.extend(batch2d::line((0, 47), (63, 30), [255, 255, 255, 255]));

    let mut batch = Batch2d::new(&device, OFFSCREEN_FORMAT);
    batch.upload_atlas(&queue, &mut atlas);
    batch.set_palettes(&device, &queue, &rows);
    batch.set_image(&device, &queue, 7, 4, 3, &img.rgba);
    batch.prepare(&device, &queue, (w, h), &quads);
    let got = render_rgba(&device, &queue, w, h, |pass| batch.draw(pass));
    let want = batch2d::reference(&quads, &atlas, &rows, &images, w as i32, h as i32);
    for (i, (g, e)) in got.chunks(4).zip(want.chunks(4)).enumerate() {
        for k in 0..3 {
            let d = (i32::from(g[k]) - i32::from(e[k])).abs();
            assert!(
                d <= 1,
                "pixel ({}, {}) got {g:?} want {e:?}",
                i as u32 % w,
                i as u32 / w
            );
        }
    }
}

/// An image drawn from a texture the caller owns (the client's map target)
/// matches the same image uploaded through `set_image`.
#[test]
fn image_views_draw_like_uploaded_images() {
    let Some((device, queue)) = device() else {
        return;
    };
    let (w, h) = (16u32, 12u32);
    let rgba: Vec<u8> = (0..(w * h) as usize)
        .flat_map(|i| [(i * 7) as u8, (i * 3) as u8, 255 - i as u8, 255])
        .collect();
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("owned"),
        size: wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    queue.write_texture(
        texture.as_image_copy(),
        &rgba,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(w * 4),
            rows_per_image: Some(h),
        },
        wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
    );
    let quads = [Quad::image(1, (w as i32, h as i32), [0, 0, w as i32, h as i32])];
    let mut atlas = Atlas::default();
    let mut batch = Batch2d::new(&device, OFFSCREEN_FORMAT);
    batch.upload_atlas(&queue, &mut atlas);
    batch.set_palettes(&device, &queue, &[palette(0)]);
    batch.set_image_view(&device, 1, &texture.create_view(&Default::default()), (w, h));
    batch.prepare(&device, &queue, (w, h), &quads);
    let got = render_rgba(&device, &queue, w, h, |pass| batch.draw(pass));
    for (i, (g, e)) in got.chunks(4).zip(rgba.chunks(4)).enumerate() {
        for k in 0..3 {
            assert!(
                (i32::from(g[k]) - i32::from(e[k])).abs() <= 1,
                "pixel {i} got {g:?} want {e:?}"
            );
        }
    }
}
