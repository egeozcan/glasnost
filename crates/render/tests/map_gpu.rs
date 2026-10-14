//! The GPU map path draws a synthetic scene exactly as the CPU reference
//! compositor does (S-0102 claims 9 and 10, F1 to F3). Needs a GPU adapter
//! (software is fine); skipped without one unless `GLASNOST_REQUIRE_GPU=1`.

#![cfg(not(target_arch = "wasm32"))]

use glasnost_fmt::image::IndexedImage;
use glasnost_render::compose::{ComposeQuirks, DEPTH_REFERENCE, DepthTest, Paint, Profile};
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
            // S-0010 claim 22: every third tile paints index 0 inside its diamond.
            paint: if k % 3 == 0 {
                Paint::Diamond(0)
            } else {
                Paint::Row(0)
            },
            group: Group::Tiles,
            provisional: false,
            object: None,
            stack: None,
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
            stack: None,
        });
    }
    // S-0102 claim 25: a smudge neither reads nor writes depth.
    s.items.push(Item {
        image: flat,
        x: 50,
        y: 22,
        test: DepthTest::Ignore,
        depth: ItemDepth::Profile {
            profile: Profile::Flat,
            bias: 0,
            companion: None,
        },
        paint: Paint::Row(2),
        group: Group::Smudges,
        provisional: false,
        object: None,
        stack: None,
    });
    s
}

#[test]
fn gpu_matches_cpu_reference() {
    let Some((device, queue)) = device() else {
        return;
    };
    let s = scene();
    let mut r = MapRenderer::new(&device, &queue, &s, glasnost_render::map_gpu::COLOR_FORMAT)
        .expect("scene uploads");
    for (origin, size) in [
        ((0, 0), (128, 72)),
        ((-7, 13), (100, 40)),
        ((5, -3), (64, 64)),
    ] {
        let view = View {
            origin,
            size,
            reference: DEPTH_REFERENCE,
        };
        let filter = Filter::default();
        let cpu = s
            .compose(&view, &filter, &ComposeQuirks::default())
            .to_rgba();
        let (color, depth) = targets(&device, size, glasnost_render::map_gpu::COLOR_FORMAT);
        let mut encoder = device.create_command_encoder(&Default::default());
        r.render(
            &device,
            &queue,
            &mut encoder,
            &color,
            &depth.create_view(&Default::default()),
            &s,
            &view,
            &filter,
            &ComposeQuirks::default(),
        );
        // The viewer shows an sRGB-typed copy of the target: the copy must
        // keep the bytes (the GL backends included).
        let display = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("display"),
            size: wgpu::Extent3d {
                width: size.0,
                height: size.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        encoder.copy_texture_to_texture(
            color.as_image_copy(),
            display.as_image_copy(),
            wgpu::Extent3d {
                width: size.0,
                height: size.1,
                depth_or_array_layers: 1,
            },
        );
        queue.submit([encoder.finish()]);
        let gpu = read_texture(&device, &queue, &color, size.0, size.1);
        let copied = read_texture(&device, &queue, &display, size.0, size.1);
        assert_eq!(gpu, copied, "the sRGB-typed copy changed bytes");
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

/// Images and palette rows added after the upload reach the GPU through
/// `sync`: the result still matches the CPU reference.
#[test]
fn sync_uploads_added_images_and_rows() {
    let Some((device, queue)) = device() else {
        return;
    };
    let full = scene();
    let mut head = Scene::default();
    head.add_row(full.rows[0]);
    head.images.extend(full.images[..2].iter().cloned());
    head.items.extend(
        full.items
            .iter()
            .filter(|i| i.image < 2 && i.paint == Paint::Row(0))
            .copied(),
    );
    let mut r = MapRenderer::new(
        &device,
        &queue,
        &head,
        glasnost_render::map_gpu::COLOR_FORMAT,
    )
    .expect("scene uploads");
    r.sync(&device, &queue, &full).expect("added images fit");
    let size = (128, 72);
    let view = View {
        origin: (0, 0),
        size,
        reference: DEPTH_REFERENCE,
    };
    let filter = Filter::default();
    let cpu = full
        .compose(&view, &filter, &ComposeQuirks::default())
        .to_rgba();
    let (color, depth) = targets(&device, size, glasnost_render::map_gpu::COLOR_FORMAT);
    let mut encoder = device.create_command_encoder(&Default::default());
    r.render(
        &device,
        &queue,
        &mut encoder,
        &color,
        &depth.create_view(&Default::default()),
        &full,
        &view,
        &filter,
        &ComposeQuirks::default(),
    );
    queue.submit([encoder.finish()]);
    let gpu = read_texture(&device, &queue, &color, size.0, size.1);
    let differing = cpu
        .chunks(4)
        .zip(gpu.chunks(4))
        .filter(|(a, b)| a[..3] != b[..3])
        .count();
    assert_eq!(differing, 0);
}

/// spec: S-0102 claim 27 (cell-pass shadows store their depth; quirks.cell_pass_shadows_write_depth)
///
/// Two writer shadows at the same depth over a flat item: with the switch
/// on the overlap is darkened once, with it off twice, on the GPU as in the
/// CPU reference (the GPU halves expanded 8-bit channels, so channels are
/// compared within a small tolerance).
#[test]
fn overlapping_writer_shadows() {
    let Some((device, queue)) = device() else {
        return;
    };
    let mut s = Scene::default();
    s.add_row(std::array::from_fn(|_| 0xFFFF));
    let ground = s.add_image(IndexedImage::new(16, 4, vec![1; 64]).unwrap());
    let shade = s.add_image(IndexedImage::new(8, 4, vec![1; 32]).unwrap());
    let item = |image, x, bias, test, paint| Item {
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
    };
    s.items
        .push(item(ground, 0, 0, DepthTest::Writer, Paint::Row(0)));
    s.items
        .push(item(shade, 0, -4, DepthTest::Writer, Paint::Shadow));
    s.items
        .push(item(shade, 4, -4, DepthTest::Writer, Paint::Shadow));
    let mut r = MapRenderer::new(&device, &queue, &s, glasnost_render::map_gpu::COLOR_FORMAT)
        .expect("scene uploads");
    let size = (16, 4);
    let view = View {
        origin: (0, 0),
        size,
        reference: DEPTH_REFERENCE,
    };
    for on in [true, false] {
        let q = ComposeQuirks {
            cell_pass_shadows_write_depth: on,
        };
        let cpu = s.compose(&view, &Filter::default(), &q).to_rgba();
        let (color, depth) = targets(&device, size, glasnost_render::map_gpu::COLOR_FORMAT);
        let mut encoder = device.create_command_encoder(&Default::default());
        r.render(
            &device,
            &queue,
            &mut encoder,
            &color,
            &depth.create_view(&Default::default()),
            &s,
            &view,
            &Filter::default(),
            &q,
        );
        queue.submit([encoder.finish()]);
        let gpu = read_texture(&device, &queue, &color, size.0, size.1);
        // The overlap (x = 4 … 7) of row 0: once darkened with the switch on, twice off.
        let overlap = 4 * 4;
        let single = 4; // pixel 1 of row 0, inside one shadow only
        let (o, one) = (cpu[overlap], cpu[single]);
        if on {
            assert_eq!(o, one, "darkened once where the shadows overlap");
        } else {
            assert!(o < one, "darkened twice where the shadows overlap");
        }
        for (a, b) in cpu.chunks(4).zip(gpu.chunks(4)) {
            for k in 0..3 {
                assert!(a[k].abs_diff(b[k]) <= 8, "switch {on}: cpu {a:?} gpu {b:?}");
            }
        }
    }
}

/// spec: S-0102 claim 32, F8 (translucent drawings blend each packed channel
/// with the colour already drawn, write no depth, and stack)
///
/// Overlapping translucent readers of all three codes over the synthetic
/// scene, one partly hidden behind a writer, draw the same on the GPU as on
/// the CPU, exactly.
#[test]
fn gpu_translucency_matches_cpu() {
    use glasnost_render::compose::{Shade, Translucency};
    let Some((device, queue)) = device() else {
        return;
    };
    let mut s = scene();
    let tall = s.items[12].image;
    for (k, (code, x, y, bias)) in [
        (1, 25, 15, 40),
        (2, 35, 20, 40),
        (3, 45, 10, 40),
        (2, 15, 5, -60),
    ]
    .into_iter()
    .enumerate()
    {
        s.items.push(Item {
            image: tall,
            x,
            y,
            test: DepthTest::Reader,
            depth: ItemDepth::Profile {
                profile: Profile::Upright,
                bias,
                companion: None,
            },
            paint: Paint::Translucent(
                Shade::new(k % 3, 1, 1000),
                Translucency::from_code(code).unwrap(),
            ),
            group: Group::Buildings,
            provisional: false,
            object: None,
            stack: None,
        });
    }
    let mut r = MapRenderer::new(&device, &queue, &s, glasnost_render::map_gpu::COLOR_FORMAT)
        .expect("scene uploads");
    for (origin, size) in [((0, 0), (128, 72)), ((10, 4), (60, 50))] {
        let view = View {
            origin,
            size,
            reference: DEPTH_REFERENCE,
        };
        let filter = Filter::default();
        let q = ComposeQuirks::default();
        let cpu = s.compose(&view, &filter, &q).to_rgba();
        assert_ne!(
            cpu,
            scene().compose(&view, &filter, &q).to_rgba(),
            "the translucent drawings show"
        );
        let (color, depth) = targets(&device, size, glasnost_render::map_gpu::COLOR_FORMAT);
        let mut encoder = device.create_command_encoder(&Default::default());
        r.render(
            &device,
            &queue,
            &mut encoder,
            &color,
            &depth.create_view(&Default::default()),
            &s,
            &view,
            &filter,
            &q,
        );
        queue.submit([encoder.finish()]);
        let gpu = read_texture(&device, &queue, &color, size.0, size.1);
        let differing = cpu
            .chunks(4)
            .zip(gpu.chunks(4))
            .filter(|(a, b)| a[..3] != b[..3])
            .count();
        assert_eq!(differing, 0, "view at {origin:?}");
    }
}

/// spec: S-0102 claim 31, F2, F5 (a companion depth image: every row takes the anchor value minus
/// the signed sample; pixels outside its stored rectangle are not drawn and write no depth)
///
/// Writers and readers of all three profiles with companion images that
/// cover them partly, over the synthetic scene, draw the same on the GPU as
/// on the CPU, exactly, and a second companion added later reaches the GPU
/// through `sync`.
#[test]
fn gpu_companion_matches_cpu() {
    use glasnost_render::compose::SignedImage;
    let Some((device, queue)) = device() else {
        return;
    };
    let mut s = scene();
    let tall = s.items[12].image;
    let bytes: Vec<u8> = (0..16 * 40)
        .map(|i| [0, 64, 65, 66, 1, 200][i % 6])
        .collect();
    s.signed
        .push(SignedImage::from_companion_bytes(16, 40, &bytes));
    let companion_item = |profile, test, x, y, bias, signed: u32, cx, cy| Item {
        image: tall,
        x,
        y,
        test,
        depth: ItemDepth::Profile {
            profile,
            bias,
            companion: Some((signed, cx, cy)),
        },
        paint: Paint::Row(1),
        group: Group::Buildings,
        provisional: false,
        object: None,
        stack: None,
    };
    s.items.push(companion_item(
        Profile::Upright,
        DepthTest::Writer,
        24,
        12,
        -10,
        0,
        28,
        20,
    ));
    s.items.push(companion_item(
        Profile::Flat,
        DepthTest::Reader,
        34,
        18,
        -40,
        0,
        30,
        8,
    ));
    s.items.push(companion_item(
        Profile::Slope,
        DepthTest::Writer,
        60,
        2,
        -5,
        0,
        64,
        -10,
    ));
    let mut r = MapRenderer::new(&device, &queue, &s, glasnost_render::map_gpu::COLOR_FORMAT)
        .expect("scene uploads");
    let check = |s: &Scene, r: &mut MapRenderer| {
        for (origin, size) in [((0, 0), (128, 72)), ((20, 6), (50, 40))] {
            let view = View {
                origin,
                size,
                reference: DEPTH_REFERENCE,
            };
            let filter = Filter::default();
            let q = ComposeQuirks::default();
            let cpu = s.compose(&view, &filter, &q).to_rgba();
            assert_ne!(
                cpu,
                scene().compose(&view, &filter, &q).to_rgba(),
                "the companion drawings show"
            );
            let (color, depth) = targets(&device, size, glasnost_render::map_gpu::COLOR_FORMAT);
            let mut encoder = device.create_command_encoder(&Default::default());
            r.render(
                &device,
                &queue,
                &mut encoder,
                &color,
                &depth.create_view(&Default::default()),
                s,
                &view,
                &filter,
                &q,
            );
            queue.submit([encoder.finish()]);
            let gpu = read_texture(&device, &queue, &color, size.0, size.1);
            let differing = cpu
                .chunks(4)
                .zip(gpu.chunks(4))
                .filter(|(a, b)| a[..3] != b[..3])
                .count();
            assert_eq!(differing, 0, "view at {origin:?}");
        }
    };
    check(&s, &mut r);
    s.signed
        .push(SignedImage::from_companion_bytes(8, 8, &[70; 64]));
    s.items.push(companion_item(
        Profile::Upright,
        DepthTest::Reader,
        40,
        25,
        -60,
        1,
        42,
        40,
    ));
    r.sync(&device, &queue, &s)
        .expect("the added companion fits");
    check(&s, &mut r);
}

/// spec: S-0104 claim 15, S-0110 claim 5 (each pixel of a shaded drawing takes the row its
/// light-alpha value gives)
///
/// The same scene with every item shaded through the three-row table and a
/// light-alpha gradient over the view draws the same on the GPU as on the
/// CPU, and twice: a second view of the same size reuses the texture.
#[test]
fn gpu_light_alpha_matches_cpu() {
    use glasnost_render::compose::Shade;
    let Some((device, queue)) = device() else {
        return;
    };
    let mut s = scene();
    for (k, item) in s.items.iter_mut().enumerate() {
        let shade = Shade::new(0, 3, [400, 1000, 2000][k % 3]);
        item.paint = match item.paint {
            Paint::Diamond(_) => Paint::LitDiamond(shade),
            Paint::Row(_) => Paint::Lit(shade),
            p => p,
        };
    }
    let mut r = MapRenderer::new(&device, &queue, &s, glasnost_render::map_gpu::COLOR_FORMAT)
        .expect("scene uploads");
    let size = (128u32, 72u32);
    for (origin, step) in [((0, 0), 1usize), ((-7, 13), 3)] {
        let view = View {
            origin,
            size,
            reference: DEPTH_REFERENCE,
        };
        let alpha: Vec<u8> = (0..(size.0 * size.1) as usize)
            .map(|i| ((i * step) % 256) as u8)
            .collect();
        let filter = Filter::default();
        let q = ComposeQuirks::default();
        let cpu = s
            .compose_lit(&view, &filter, &q, Some(alpha.clone()))
            .to_rgba();
        let (color, depth) = targets(&device, size, glasnost_render::map_gpu::COLOR_FORMAT);
        let mut encoder = device.create_command_encoder(&Default::default());
        r.render_lit(
            &device,
            &queue,
            &mut encoder,
            &color,
            &depth.create_view(&Default::default()),
            &s,
            &view,
            &filter,
            &q,
            Some(&alpha),
        );
        queue.submit([encoder.finish()]);
        let gpu = read_texture(&device, &queue, &color, size.0, size.1);
        let differing = cpu
            .chunks(4)
            .zip(gpu.chunks(4))
            .filter(|(a, b)| a[..3] != b[..3])
            .count();
        assert_eq!(differing, 0, "view at {origin:?}");
    }
}
