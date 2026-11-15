//! The interface atlas uploaded by the rows that changed (design: render
//! (#123)): an image added after an upload sends the rows it lies on, not
//! the whole atlas, and every draw is the CPU reference's picture. Needs a
//! GPU adapter; skipped without one unless `GLASNOST_REQUIRE_GPU=1`.

#![cfg(not(target_arch = "wasm32"))]

use std::collections::BTreeMap;

use glasnost_render::batch2d::{self, Atlas, Batch2d, Quad};
use glasnost_render::gpu;
use glasnost_render::offscreen::{OFFSCREEN_FORMAT, render_rgba};

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

fn palette() -> [u8; 1024] {
    let mut p = [0u8; 1024];
    for i in 0..256usize {
        p[i * 4] = (i as u8).wrapping_mul(3);
        p[i * 4 + 1] = (i as u8) ^ 0x5A;
        p[i * 4 + 2] = 255 - i as u8;
        p[i * 4 + 3] = 255;
    }
    p
}

/// A `w` × `h` image of varied indices, none transparent.
fn image(w: u32, h: u32, seed: u32) -> Vec<u8> {
    (0..w * h).map(|i| ((i * 7 + seed) % 200 + 20) as u8).collect()
}

/// Draws `quads` and asserts the picture is the CPU reference's.
fn check(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    batch: &mut Batch2d,
    atlas: &Atlas,
    quads: &[Quad],
    what: &str,
) {
    let (w, h) = (64u32, 48u32);
    let rows = [palette()];
    batch.prepare(device, queue, (w, h), quads);
    let got = render_rgba(device, queue, w, h, |pass| batch.draw(pass));
    let want = batch2d::reference(quads, atlas, &rows, &BTreeMap::new(), w as i32, h as i32);
    for (i, (g, e)) in got.chunks(4).zip(want.chunks(4)).enumerate() {
        for k in 0..3 {
            let d = (i32::from(g[k]) - i32::from(e[k])).abs();
            assert!(
                d <= 1,
                "{what}: pixel ({}, {}) got {g:?} want {e:?}",
                i as u32 % w,
                i as u32 / w
            );
        }
    }
}

/// design: render (#123): the first upload sends the whole atlas; a sprite added on the same
/// shelf sends that shelf's rows, one on a new shelf its own rows, and after `touch` the whole
/// atlas goes again; each new image draws.
#[test]
fn an_added_image_uploads_its_rows_alone() {
    let Some((device, queue)) = device() else {
        return;
    };
    let mut atlas = Atlas::default();
    let side = u64::from(atlas.size());
    let a = atlas.add(5, 7, &image(5, 7, 1)).unwrap();
    let mut batch = Batch2d::new(&device, OFFSCREEN_FORMAT);
    batch.set_palettes(&device, &queue, &[palette()]);
    let _ = batch.take_stats();
    batch.upload_atlas(&queue, &mut atlas);
    assert_eq!(batch.take_stats().upload_bytes, side * side, "the first upload");
    let mut quads = vec![Quad::sprite(2, 2, a, 0)];
    check(&device, &queue, &mut batch, &atlas, &quads, "the first image");

    // On the first shelf, rows 0 to 10.
    let b = atlas.add(12, 10, &image(12, 10, 2)).unwrap();
    assert_eq!((b[1], b[3]), (0, 10));
    let _ = batch.take_stats();
    batch.upload_atlas(&queue, &mut atlas);
    assert_eq!(batch.take_stats().upload_bytes, 10 * side, "the first shelf's rows");
    quads.push(Quad::sprite(20, 4, b, 0));
    check(&device, &queue, &mut batch, &atlas, &quads, "a sprite on the shelf");

    // Too wide for the first shelf: a new shelf from row 11, 5 rows high.
    let wide = atlas.size() - 4;
    let c = atlas.add(wide, 5, &image(wide, 5, 3)).unwrap();
    assert_eq!((c[1], c[3]), (11, 5));
    let _ = batch.take_stats();
    batch.upload_atlas(&queue, &mut atlas);
    assert_eq!(batch.take_stats().upload_bytes, 5 * side, "the new shelf's rows");
    quads.push(Quad::sprite(1, 30, [c[0], c[1], 60, 5], 0));
    check(&device, &queue, &mut batch, &atlas, &quads, "a sprite on a new shelf");

    // Nothing added: nothing sent.
    let _ = batch.take_stats();
    batch.upload_atlas(&queue, &mut atlas);
    assert_eq!(batch.take_stats().upload_bytes, 0, "nothing changed");

    // Two images on two shelves: the rows from the first's to the second's
    // (the first fills the second shelf to its end).
    let d = atlas.add(3, 3, &image(3, 3, 4)).unwrap();
    let e = atlas.add(wide, 4, &image(wide, 4, 5)).unwrap();
    assert_eq!((d[1], e[1], e[3]), (11, 17, 4));
    let _ = batch.take_stats();
    batch.upload_atlas(&queue, &mut atlas);
    assert_eq!(batch.take_stats().upload_bytes, 10 * side, "rows 11 to 21");
    quads.push(Quad::sprite(40, 20, d, 0));
    quads.push(Quad::sprite(1, 40, [e[0], e[1], 60, 4], 0));
    check(&device, &queue, &mut batch, &atlas, &quads, "two shelves at once");

    // A new device holds nothing: the whole atlas again.
    atlas.touch();
    let _ = batch.take_stats();
    batch.upload_atlas(&queue, &mut atlas);
    assert_eq!(batch.take_stats().upload_bytes, side * side, "after touch");
    check(&device, &queue, &mut batch, &atlas, &quads, "after touch");
}
