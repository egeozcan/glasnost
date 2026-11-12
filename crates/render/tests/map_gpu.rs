//! The GPU map path draws a synthetic scene exactly as the CPU reference
//! compositor does (S-0102 claims 9 and 10, F1 to F3). Needs a GPU adapter
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

/// Images rewritten in place after the upload (`Scene::replace_image`)
/// reach the GPU through `sync`, into their own slots: the result still
/// matches the CPU reference, and the image count does not grow.
#[test]
fn sync_rewrites_replaced_images() {
    let Some((device, queue)) = device() else {
        return;
    };
    let mut s = scene();
    let mut r = MapRenderer::new(&device, &queue, &s, glasnost_render::map_gpu::COLOR_FORMAT)
        .expect("scene uploads");
    let n = s.images.len();
    for id in [0u32, 2] {
        let (w, h) = (
            s.images[id as usize].width(),
            s.images[id as usize].height(),
        );
        assert!(s.replace_image(id, image(w, h, 77 + id as u8)));
    }
    assert!(
        !s.replace_image(1, image(1, 1, 3)),
        "a size change is refused"
    );
    assert_eq!(s.images.len(), n);
    r.sync(&device, &queue, &s).expect("rewrites fit");
    let size = (128, 72);
    let view = View {
        origin: (0, 0),
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
    queue.submit([encoder.finish()]);
    let gpu = read_texture(&device, &queue, &color, size.0, size.1);
    let differing = cpu
        .chunks(4)
        .zip(gpu.chunks(4))
        .filter(|(a, b)| a[..3] != b[..3])
        .count();
    assert_eq!(differing, 0);
}

/// How many pixels of `r`'s drawing of `s` at a 128 × 72 view differ from
/// the CPU reference.
fn differing_from_cpu(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    r: &mut MapRenderer,
    s: &Scene,
) -> usize {
    let size = (128, 72);
    let view = View {
        origin: (0, 0),
        size,
        reference: DEPTH_REFERENCE,
    };
    let filter = Filter::default();
    let cpu = s
        .compose(&view, &filter, &ComposeQuirks::default())
        .to_rgba();
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
        &filter,
        &ComposeQuirks::default(),
    );
    queue.submit([encoder.finish()]);
    let gpu = read_texture(device, queue, &color, size.0, size.1);
    cpu.chunks(4)
        .zip(gpu.chunks(4))
        .filter(|(a, b)| a[..3] != b[..3])
        .count()
}

/// An item drawing image `image` flat at (`x`, `y`) with no depth test.
fn flat_item(image: u32, x: i32, y: i32) -> Item {
    Item {
        image,
        x,
        y,
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
    }
}

/// design: render (#105): images added while the atlas is full make it
/// grow: `sync` succeeds with no new renderer, and the images placed before
/// and after the growth are drawn as the CPU reference draws them.
#[test]
fn a_full_atlas_grows_in_sync() {
    let Some((device, queue)) = device() else {
        return;
    };
    let mut s = scene();
    let mut r = MapRenderer::with_side(
        &device,
        &queue,
        &s,
        glasnost_render::map_gpu::COLOR_FORMAT,
        64,
    )
    .expect("scene uploads");
    let layers = r.atlas_use().layers;
    let mut added = Vec::new();
    for k in 0..48u32 {
        let id = s.add_image(image(30, 20, k as u8));
        added.push(id);
        if k % 8 == 0 {
            r.sync(&device, &queue, &s)
                .expect("a full atlas grows instead of failing");
        }
    }
    r.sync(&device, &queue, &s)
        .expect("a full atlas grows instead of failing");
    assert!(
        r.atlas_use().layers > layers,
        "the atlas grew from {layers} layers"
    );
    // The first and the last images added, and one between.
    for (n, &id) in [added[0], added[23], added[47]].iter().enumerate() {
        s.items.push(flat_item(id, 10 + 40 * n as i32, 45));
    }
    assert_eq!(differing_from_cpu(&device, &queue, &mut r, &s), 0);
}

/// design: render (#104, #105): an image rewritten at another size
/// (`Scene::resize_image`) moves to a new place, and a released one (0 × 0)
/// frees its place for later images; the result matches the CPU reference.
#[test]
fn resized_and_released_images_move_and_free_their_places() {
    let Some((device, queue)) = device() else {
        return;
    };
    let mut s = scene();
    let spare = s.add_image(image(40, 40, 9));
    let mut r = MapRenderer::with_side(
        &device,
        &queue,
        &s,
        glasnost_render::map_gpu::COLOR_FORMAT,
        64,
    )
    .expect("scene uploads");
    let before = r.atlas_use();
    // The tall and flat images (2 and 3) change size; the spare one goes.
    assert!(s.resize_image(2, image(25, 40, 61)));
    assert!(s.resize_image(3, image(40, 14, 62)));
    assert!(s.resize_image(spare, IndexedImage::new(0, 0, Vec::new()).unwrap()));
    r.sync(&device, &queue, &s).expect("the rewrites fit");
    assert_eq!(differing_from_cpu(&device, &queue, &mut r, &s), 0);
    // The spare image's place is free again: as many images of its size fit
    // with no growth.
    let id = s.add_image(image(40, 40, 10));
    s.items.push(flat_item(id, 70, 20));
    r.sync(&device, &queue, &s).expect("the new image fits");
    assert_eq!(r.atlas_use().layers, before.layers, "no growth");
    assert_eq!(differing_from_cpu(&device, &queue, &mut r, &s), 0);
}

/// design: render (#105): images added later are packed by height class,
/// not in arrival order: tall and short images arriving in turn do not
/// leave every shelf as tall as the tall ones.
#[test]
fn later_images_are_packed_by_height_class() {
    let Some((device, queue)) = device() else {
        return;
    };
    let mut s = Scene::default();
    s.add_row([0; 256]);
    s.add_image(image(1, 1, 1));
    let mut r = MapRenderer::with_side(
        &device,
        &queue,
        &s,
        glasnost_render::map_gpu::COLOR_FORMAT,
        256,
    )
    .expect("scene uploads");
    for k in 0..64u8 {
        s.add_image(image(8, 64, k));
        s.add_image(image(8, 8, k));
    }
    r.sync(&device, &queue, &s).expect("the images fit");
    // 64 tall images fill two 64-row shelves, 64 short ones (and the first
    // image) two or three 8-row shelves.
    let used = r.atlas_use();
    assert!(
        used.committed_px <= (128 + 24) * 256,
        "{} atlas pixels in shelves",
        used.committed_px
    );
}

/// An invented palette row, different for each `k`.
fn row(k: u16) -> [u16; 256] {
    std::array::from_fn(|i| (i as u16).wrapping_mul(31).wrapping_add(k.wrapping_mul(613)))
}

/// design: render (#107): a palette row appended to the scene is written
/// into the row texture on its own, so one row costs one row's bytes, not
/// the whole texture's; rows appended past the texture's room still draw
/// as the CPU reference draws them.
#[test]
fn appended_rows_write_only_themselves() {
    let Some((device, queue)) = device() else {
        return;
    };
    let mut s = scene();
    for k in 0..40 {
        s.add_row(row(k));
    }
    let mut r = MapRenderer::new(&device, &queue, &s, glasnost_render::map_gpu::COLOR_FORMAT)
        .expect("scene uploads");
    r.take_stats();
    let one = s.add_row(row(100));
    r.sync(&device, &queue, &s).expect("the row fits");
    let stats = r.take_stats();
    assert!(
        stats.upload_bytes <= 256 * 4,
        "one appended row uploaded {} bytes",
        stats.upload_bytes
    );
    let mut item = flat_item(3, 10, 40);
    item.paint = Paint::Row(one);
    s.items.push(item);
    // Many more rows, synced now and then: the texture makes room for them.
    for k in 0..3000 {
        s.add_row(row(200 + k));
        if k % 500 == 0 {
            r.sync(&device, &queue, &s).expect("the rows fit");
        }
    }
    r.sync(&device, &queue, &s).expect("the rows fit");
    for (n, at) in [(1, s.rows.len() / 2), (2, s.rows.len() - 1)] {
        let mut item = flat_item(3, 10 + 40 * n, 40);
        item.paint = Paint::Row(at);
        s.items.push(item);
    }
    assert_eq!(differing_from_cpu(&device, &queue, &mut r, &s), 0);
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
        // The GPU halves rounded down (#79): the same packed pixel. The low display bits keep
        // the halved expansion (255, 127, 63), the CPU re-expands (255, 123, 57).
        for (a, b) in cpu.chunks(4).zip(gpu.chunks(4)) {
            assert_eq!(
                (a[0] >> 3, a[1] >> 2, a[2] >> 3),
                (b[0] >> 3, b[1] >> 2, b[2] >> 3),
                "switch {on}: cpu {a:?} gpu {b:?}"
            );
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

/// spec: S-0616 claim 28 (a pixel particle: one opaque pixel of its own
/// colour, skipped at light-alpha 0, scaled by a / 128 below 127; a flat
/// reader with bias −50 − L(z), never writing depth)
///
/// Single-pixel items over the whole view, some hidden by the writers,
/// draw the same on the GPU as on the CPU under a light-alpha gradient.
#[test]
fn gpu_pixel_particles_match_cpu() {
    use glasnost_fmt::image::Rgb8;
    let Some((device, queue)) = device() else {
        return;
    };
    let mut s = scene();
    let dot = s.add_image(IndexedImage::new(1, 1, vec![1]).unwrap());
    for k in 0..400i32 {
        s.items.push(Item {
            image: dot,
            x: (k * 37) % 128,
            y: (k * 11) % 72,
            test: DepthTest::Reader,
            depth: ItemDepth::Profile {
                profile: Profile::Flat,
                bias: -50 - (k % 7) * 5,
                companion: None,
            },
            paint: Paint::Pixel(Rgb8 {
                r: (k * 13 % 256) as u8,
                g: (k * 29 % 256) as u8,
                b: (255 - k % 256) as u8,
            }),
            group: Group::Effects,
            provisional: false,
            object: None,
            stack: None,
        });
    }
    let mut r = MapRenderer::new(&device, &queue, &s, glasnost_render::map_gpu::COLOR_FORMAT)
        .expect("scene uploads");
    let size = (128u32, 72u32);
    let view = View {
        origin: (0, 0),
        size,
        reference: DEPTH_REFERENCE,
    };
    let alpha: Vec<u8> = (0..(size.0 * size.1) as usize)
        .map(|i| ((i * 3) % 256) as u8)
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
    assert_eq!(differing, 0);
}

/// spec: S-0102 claim 34 and F9 (an opaque, depth-passing pixel becomes S ∨ K after shade
/// selection; transparent and depth-rejected pixels stay untouched; a translucent drawing keeps
/// its blend and takes no mask, Open questions)
///
/// The scene with every lit item carrying a protection colour mask, some
/// oversized (bits into another channel) and one the all-ones triple, draws
/// the same on the GPU as on the CPU.
#[test]
fn gpu_protection_mask_matches_cpu() {
    use glasnost_render::compose::{PixelLayout, Shade, Translucency, protection_mask};
    let Some((device, queue)) = device() else {
        return;
    };
    let masks = [
        protection_mask([7, 19, 3], PixelLayout::Rgb565),
        protection_mask([255, 255, 255], PixelLayout::Rgb565),
        protection_mask([0, 0, 40], PixelLayout::Rgb565),
        0,
    ];
    let mut s = scene();
    for (k, item) in s.items.iter_mut().enumerate() {
        let shade = Shade::new(0, 3, [400, 1000, 2000][k % 3]).with_mask(masks[k % 4]);
        item.paint = match item.paint {
            Paint::Diamond(_) => Paint::LitDiamond(shade),
            Paint::Row(_) => Paint::Lit(shade),
            p => p,
        };
    }
    let tall = s.items[12].image;
    s.items.push(Item {
        image: tall,
        x: 35,
        y: 20,
        test: DepthTest::Reader,
        depth: ItemDepth::Profile {
            profile: Profile::Upright,
            bias: 40,
            companion: None,
        },
        paint: Paint::Translucent(Shade::new(1, 1, 1000), Translucency::Half),
        group: Group::Buildings,
        provisional: false,
        object: None,
        stack: None,
    });
    // spec: S-0513 claim 29b (a translucent drawing of a protected object takes no mask: a shade
    // carrying one blends as its unmasked source on both paths)
    s.items.push(Item {
        image: tall,
        x: 60,
        y: 20,
        test: DepthTest::Reader,
        depth: ItemDepth::Profile {
            profile: Profile::Upright,
            bias: 40,
            companion: None,
        },
        paint: Paint::Translucent(
            Shade::new(0, 3, 2000).with_mask(masks[1]),
            Translucency::Quarter,
        ),
        group: Group::Buildings,
        provisional: false,
        object: None,
        stack: None,
    });
    let mut r = MapRenderer::new(&device, &queue, &s, glasnost_render::map_gpu::COLOR_FORMAT)
        .expect("scene uploads");
    let size = (128u32, 72u32);
    let view = View {
        origin: (0, 0),
        size,
        reference: DEPTH_REFERENCE,
    };
    let alpha: Vec<u8> = (0..(size.0 * size.1) as usize)
        .map(|i| (i % 256) as u8)
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
    assert_eq!(differing, 0);
}

/// spec: S-0518 claims 20 to 23, F7, F8 (opaque, additive and blended beam pixels: strict depth
/// test without depth write, overlapping samples composed in order, the pair's first light-alpha);
/// S-0514 claim 25, F8 (beam flash pixels); S-0607 claim 25, F5 (the wave reads the picture from
/// before it, 3-pixel view-edge margin)
///
/// Spot groups of every kind, some overlapping, some failing depth, one
/// between items and the rest after them, draw the same on the GPU as on the
/// CPU, exactly.
#[test]
fn gpu_spots_match_cpu() {
    use glasnost_render::spots::{Spot, SpotGroup, SpotKind, easing};
    let Some((device, queue)) = device() else {
        return;
    };
    let mut s = scene();
    let mut wave = Vec::new();
    for y in 0..40 {
        for x in 0..50 {
            wave.push(Spot {
                x: x + 2,
                y: y + 1,
                bias: (x % 7) * 40 - 120,
                kind: SpotKind::Wave {
                    shift: [(1, 0), (-2, 0), (0, 1), (0, -2)][((x + y) % 4) as usize],
                    k: 110 + 8 * ((x * y) % 10) as u16,
                },
            });
        }
    }
    s.spots.push(SpotGroup {
        at: 6,
        group: Group::Effects,
        provisional: false,
        spots: wave,
    });
    let mut beams = Vec::new();
    for k in 0..90 {
        let (x, y) = (k + 3, 10 + k / 3);
        let bias = [-200, -2, 30, 400][(k % 4) as usize];
        let rgb = [
            (k * 37 % 256) as u8,
            (k * 91 % 256) as u8,
            (k * 13 % 256) as u8,
        ];
        let w = (k as u32 * 29) % 257;
        for kind in [
            SpotKind::Additive { rgb },
            SpotKind::Additive { rgb: [200, 40, 0] },
            SpotKind::Blended {
                rgb,
                line: easing(w),
                screen: easing(256 - w),
                alpha_from: (0, -1),
            },
            SpotKind::Opaque {
                colour: (k as u16).wrapping_mul(2971),
            },
            SpotKind::Flash { rgb },
        ]
        .into_iter()
        .skip((k % 3) as usize)
        {
            beams.push(Spot { x, y, bias, kind });
            beams.push(Spot {
                x,
                y: y + 1,
                bias,
                kind,
            });
        }
    }
    s.spots.push(SpotGroup {
        at: usize::MAX,
        group: Group::Effects,
        provisional: false,
        spots: beams,
    });
    let mut r = MapRenderer::new(&device, &queue, &s, glasnost_render::map_gpu::COLOR_FORMAT)
        .expect("scene uploads");
    let size = (128u32, 72u32);
    for (origin, step) in [((0, 0), 1usize), ((5, 3), 7)] {
        let view = View {
            origin,
            size,
            reference: DEPTH_REFERENCE,
        };
        let alpha: Vec<u8> = (0..(size.0 * size.1) as usize)
            .map(|i| [127, 0, 64, 200, 127][(i * step) % 5])
            .collect();
        let filter = Filter::default();
        let q = ComposeQuirks::default();
        let cpu = s
            .compose_lit(&view, &filter, &q, Some(alpha.clone()))
            .to_rgba();
        let plain = scene()
            .compose_lit(&view, &filter, &q, Some(alpha.clone()))
            .to_rgba();
        assert!(
            cpu.chunks(4)
                .zip(plain.chunks(4))
                .filter(|(a, b)| a != b)
                .count()
                > 300,
            "the spots show"
        );
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
