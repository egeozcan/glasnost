//! The cell pass's cull on the GPU map path (design: render (#113)): a
//! frame visits only the cell-pass items near the view, and what it keeps
//! of them between frames follows every change that moves them: a rebuilt
//! cell pass, a resized image, another filter, a cell pass of another
//! length. Each frame draws as the CPU reference does, exactly (S-0101
//! F4b; S-0102 claims 9 and 10). Needs a GPU adapter (software is fine);
//! skipped without one unless `GLASNOST_REQUIRE_GPU=1`.

#![cfg(not(target_arch = "wasm32"))]

use glasnost_fmt::image::IndexedImage;
use glasnost_render::compose::{ComposeQuirks, DEPTH_REFERENCE, DepthTest, Paint, Profile};
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

const SIZE: (u32, u32) = (128, 72);

/// An invented image with transparent pixels on a diagonal grid.
fn image(w: u32, h: u32, seed: u32) -> IndexedImage {
    let px = (0..h)
        .flat_map(|y| {
            (0..w).map(move |x| {
                if (x + y + seed) % 9 == 0 {
                    0
                } else {
                    ((seed * 17 + x * 3 + y * 5) % 255 + 1) as u8
                }
            })
        })
        .collect();
    IndexedImage::new(w, h, px).unwrap()
}

fn item(image: u32, x: i32, y: i32, test: DepthTest, bias: i32, group: Group) -> Item {
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
        paint: Paint::Row(usize::from(group == Group::Smudges)),
        group,
        provisional: false,
        object: None,
        stack: None,
    }
}

/// A cell pass of 20 × 20 overlapping ground pieces and smudges over a
/// world of about 600 × 300 pixels, then a few objects.
fn scene() -> Scene {
    let mut s = Scene::default();
    for k in 0..2u16 {
        s.add_row(std::array::from_fn(|i| {
            (i as u16).wrapping_mul(97).wrapping_add(k * 5000)
        }));
    }
    let piece = s.add_image(image(34, 18, 1));
    let smudge = s.add_image(image(12, 9, 2));
    for cy in 0..20 {
        for cx in 0..20 {
            let (x, y) = (cx * 30 - 10, cy * 15 - 8);
            s.items
                .push(item(piece, x, y, DepthTest::Writer, -(cx + cy), Group::Tiles));
            if (cx + cy) % 3 == 0 {
                s.items
                    .push(item(smudge, x + 9, y + 4, DepthTest::Ignore, 0, Group::Smudges));
            }
        }
    }
    s.cell_pass_len = s.items.len();
    let body = s.add_image(image(16, 30, 3));
    for k in 0..6 {
        s.items.push(item(
            body,
            k * 37,
            k * 11,
            DepthTest::Reader,
            -60,
            Group::Vehicles,
        ));
    }
    s
}

/// Draws `s` through `r` at a view at `origin` with `filter`; returns the
/// pixels that differ from the CPU reference and the items visited.
fn draw(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    r: &mut MapRenderer,
    s: &Scene,
    origin: (i32, i32),
    filter: &Filter,
) -> (usize, usize) {
    let view = View {
        origin,
        size: SIZE,
        reference: DEPTH_REFERENCE,
    };
    let q = ComposeQuirks::default();
    let cpu = s.compose(&view, filter, &q).to_rgba();
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
        filter,
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

/// design: render (#113): a frame visits the cell-pass items near the view only, and the cull
/// kept between frames follows a moved view, a rebuilt cell pass (S-0101 F4b: a moved item is
/// drawn where it now is), a resized image, another filter and a shorter cell pass: every frame
/// as the CPU draws it.
#[test]
fn the_cell_pass_cull_follows_every_change() {
    let Some((device, queue)) = device() else {
        return;
    };
    let mut s = scene();
    let mut r = MapRenderer::new(&device, &queue, &s, COLOR_FORMAT).expect("scene uploads");
    let all = Filter::default();
    let cell = s.cell_pass_len;
    let mut check = |s: &Scene, origin, filter: &Filter, what: &str| {
        let (differing, visited) = draw(&device, &queue, &mut r, s, origin, filter);
        assert_eq!(differing, 0, "{what}: as the CPU draws it");
        visited
    };
    let visited = check(&s, (0, 0), &all, "the first frame");
    assert!(
        visited < cell / 4,
        "the first frame visits {visited} items of a cell pass of {cell}"
    );
    check(&s, (230, 110), &all, "the view moved");
    check(&s, (-40, -20), &all, "the view at the corner");
    // The cell pass rebuilt with its pieces moved far: the view now shows
    // pieces that were far from it.
    for it in &mut s.items[..cell] {
        it.x -= 300;
        it.y -= 150;
    }
    s.cell_pass_rev += 1;
    check(&s, (-40, -20), &all, "a rebuilt cell pass");
    // A cell-pass image resized: smudges whose old rectangles were far to
    // the left of the view now reach into it.
    assert!(s.resize_image(1, image(200, 120, 9)));
    check(&s, (-40, -20), &all, "a resized image");
    // Another filter: the tiles hidden, then the smudges.
    let no_tiles = Filter {
        groups: Group::ALL.into_iter().filter(|&g| g != Group::Tiles).collect(),
        provisional: true,
    };
    check(&s, (-40, -20), &no_tiles, "the tiles hidden");
    let no_smudges = Filter {
        groups: Group::ALL.into_iter().filter(|&g| g != Group::Smudges).collect(),
        provisional: true,
    };
    check(&s, (-40, -20), &no_smudges, "the smudges hidden");
    // A shorter cell pass, its last items now the object pass's.
    s.cell_pass_len = cell / 2;
    s.cell_pass_rev += 1;
    check(&s, (100, 60), &all, "a shorter cell pass");
}
