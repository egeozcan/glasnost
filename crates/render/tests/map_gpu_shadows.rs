//! Bodies and their shadows on the GPU map path (design: render (#114)):
//! a run of bodies and shadows with the same depth test is one draw, and
//! each pixel is what drawing the items one by one gives: a body pixel its
//! row colour, a shadow pixel the 8-bit channels under it halved, rounded
//! down (S-0102 claim 26, F7; the GPU's halving of #79). Needs a GPU
//! adapter (software is fine); skipped without one unless
//! `GLASNOST_REQUIRE_GPU=1`.

#![cfg(not(target_arch = "wasm32"))]

use glasnost_fmt::image::IndexedImage;
use glasnost_render::compose::{
    ComposeQuirks, DEPTH_REFERENCE, DepthTest, Paint, Profile, expand565,
};
use glasnost_render::gpu;
use glasnost_render::map_gpu::{COLOR_FORMAT, MapRenderer, targets};
use glasnost_render::offscreen::read_texture;
use glasnost_render::scene::{Filter, Group, Item, ItemDepth, Scene, View};

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

const SIZE: (u32, u32) = (64, 32);

/// An invented image with transparent pixels on a diagonal grid.
fn image(w: u32, h: u32, seed: u32) -> IndexedImage {
    let px = (0..h)
        .flat_map(|y| {
            (0..w).map(move |x| {
                if (x + 2 * y + seed) % 7 == 0 {
                    0
                } else {
                    ((seed * 31 + x * 3 + y * 5) % 255 + 1) as u8
                }
            })
        })
        .collect();
    IndexedImage::new(w, h, px).unwrap()
}

/// The scene: a ground writer over the whole view, then for each depth test
/// in turn ten bodies each followed by its shadow, overlapping one another.
/// Every item is nearer than every item before it, so every pixel of every
/// item passes its depth test and the picture is the items drawn in order.
fn scene() -> Scene {
    let mut s = Scene::default();
    let row = s.add_row(std::array::from_fn(|i| {
        (i as u16).wrapping_mul(2971).wrapping_add(0x1234)
    }));
    let ground = s.add_image(
        IndexedImage::new(SIZE.0, SIZE.1, vec![200; (SIZE.0 * SIZE.1) as usize]).unwrap(),
    );
    let mut bias = 0;
    let mut item = |image, x, y, test, paint| {
        bias -= 10;
        Item {
            image,
            x,
            y,
            test,
            depth: ItemDepth::Profile {
                profile: Profile::Flat,
                bias,
                companion: None,
            },
            paint,
            group: Group::Vehicles,
            provisional: false,
            object: None,
            stack: None,
        }
    };
    let ground = item(ground, 0, 0, DepthTest::Writer, Paint::Row(row));
    let mut items = vec![ground];
    let mut seed = 0;
    for test in [DepthTest::Reader, DepthTest::Writer, DepthTest::Ignore] {
        for k in 0..10 {
            seed += 1;
            let (x, y) = (k * 5 - 4, (k % 3) * 7 - 3);
            let body = s.add_image(image(14, 12, seed));
            let shade = s.add_image(image(12, 8, seed + 50));
            items.push(item(body, x, y, test, Paint::Row(row)));
            items.push(item(shade, x + 3, y + 6, test, Paint::Shadow));
        }
    }
    s.items = items;
    s
}

/// The items of `s` drawn one by one into 8-bit channels.
fn one_by_one(s: &Scene) -> Vec<[u8; 3]> {
    let (w, h) = (SIZE.0 as i32, SIZE.1 as i32);
    let mut px = vec![[0u8; 3]; (w * h) as usize];
    for it in &s.items {
        let img = &s.images[it.image as usize];
        for y in 0..img.height() as i32 {
            for x in 0..img.width() as i32 {
                let (vx, vy) = (it.x + x, it.y + y);
                if vx < 0 || vy < 0 || vx >= w || vy >= h {
                    continue;
                }
                let index = img.get(x as u32, y as u32).unwrap_or(0);
                if index == 0 {
                    continue;
                }
                let p = &mut px[(vy * w + vx) as usize];
                *p = match it.paint {
                    Paint::Row(r) => {
                        let (r, g, b) = expand565(s.rows[r][usize::from(index)]);
                        [r, g, b]
                    }
                    Paint::Shadow => [p[0] / 2, p[1] / 2, p[2] / 2],
                    _ => unreachable!("rows and shadows only"),
                };
            }
        }
    }
    px
}

/// design: render (#114): bodies and shadows that alternate within each depth test draw in one
/// call per depth test (S-0102 claim 27's switch on, its default; off, a writer's shadow is a
/// reader), pixel for pixel as drawn one by one, with the switch on and off.
#[test]
fn bodies_and_shadows_share_a_draw_and_draw_as_one_by_one() {
    let Some((device, queue)) = device() else {
        return;
    };
    let s = scene();
    let want = one_by_one(&s);
    let mut r = MapRenderer::new(&device, &queue, &s, COLOR_FORMAT).expect("scene uploads");
    let view = View {
        origin: (0, 0),
        size: SIZE,
        reference: DEPTH_REFERENCE,
    };
    for on in [false, true] {
        let q = ComposeQuirks {
            cell_pass_shadows_write_depth: on,
        };
        let (color, depth) = targets(&device, SIZE, COLOR_FORMAT);
        let mut encoder = device.create_command_encoder(&Default::default());
        let _ = r.take_stats();
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
        let stats = r.take_stats();
        let gpu = read_texture(&device, &queue, &color, SIZE.0, SIZE.1);
        let differing: Vec<usize> = gpu
            .chunks(4)
            .zip(&want)
            .enumerate()
            .filter(|(_, (g, w))| g[..3] != w[..])
            .map(|(i, _)| i)
            .collect();
        assert!(
            differing.is_empty(),
            "switch {on}: {} pixels differ, the first at {:?}",
            differing.len(),
            differing.first()
        );
        // The ground, then one run for each depth test; with the switch off a writer's shadow
        // stores no depth, so it is a reader and the writers' run breaks at each of the ten.
        let draws = if on { 4 } else { 1 + 1 + 20 + 1 };
        assert_eq!(stats.draws, draws, "switch {on}: draw calls");
    }
}
