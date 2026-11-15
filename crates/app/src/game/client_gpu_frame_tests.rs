//! The battlefield drawn on the GPU through the window's path (`GpuDraw`)
//! on synthetic data (invented rules and map, `glasnost_ai::fixture`, and
//! an invented image): a frame whose scene, view, filter and light did not
//! change keeps the battlefield it drew last, and each kind of change draws
//! it anew, as a drawer made for that frame alone does (design: render
//! (#112)). Needs a GPU adapter (software is fine); skipped without one
//! unless `GLASNOST_REQUIRE_GPU=1`.

use super::access_tests::{client, own, session, session_mut, start_by_keyboard};
use super::*;
use crate::game::window::GpuDraw;
use glasnost_fmt::image::IndexedImage;
use glasnost_render::compose::{DepthTest, Paint, Profile};
use glasnost_render::offscreen::{OFFSCREEN_FORMAT, render_rgba};
use glasnost_render::scene::{Group, Item, ItemDepth};
use glasnost_render::{gpu, wgpu};
use glasnost_sim::rules::Kind;

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

/// One frame through `draw`: the picture (RGBA, the client's size) and the
/// drawer's counts for it.
fn frame(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    draw: &mut GpuDraw,
    c: &mut Client,
) -> (Vec<u8>, crate::game::DrawStats) {
    let size = (c.size.0 as u32, c.size.1 as u32);
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("test frame"),
    });
    draw.prepare(device, queue, c, size, &mut encoder);
    queue.submit([encoder.finish()]);
    let rgba = render_rgba(device, queue, size.0, size.1, |pass| draw.draw(pass));
    assert!(!c.cpu_map, "the GPU battlefield did not fail");
    (rgba, draw.take_stats())
}

/// The same frame through a drawer made for it alone.
fn fresh(device: &wgpu::Device, queue: &wgpu::Queue, c: &mut Client) -> Vec<u8> {
    let mut draw = GpuDraw::new(device, OFFSCREEN_FORMAT);
    c.painter.dirty = true;
    frame(device, queue, &mut draw, c).0
}

/// The battlefield's pixels of a picture of the client's size.
fn battlefield(c: &Client, rgba: &[u8]) -> Vec<u8> {
    let m = session(c).map_rect();
    let w = c.size.0;
    let mut out = Vec::new();
    for y in m.y..m.y + m.h {
        let at = ((y * w + m.x) * 4) as usize;
        out.extend_from_slice(&rgba[at..at + (m.w * 4) as usize]);
    }
    out
}

/// An invented image of `w` × `h` with indices from `seed`.
fn image(w: u32, h: u32, seed: u8) -> IndexedImage {
    let px = (0..h)
        .flat_map(|y| (0..w).map(move |x| (u32::from(seed) + x * 3 + y * 5) as u8 | 1))
        .collect();
    IndexedImage::new(w, h, px).unwrap()
}

/// An invented image drawn over the view's top left part with every pixel
/// painted, on an invented palette row; returns the image's index.
fn invented_item(s: &mut Session, at: (i32, i32), seed: u8) -> u32 {
    let o = s.view().origin;
    let scene = s.scene_mut();
    let row = scene.add_row(std::array::from_fn(|i| {
        (i as u16)
            .wrapping_mul(97)
            .wrapping_add(u16::from(seed) * 1000)
    }));
    let id = scene.add_image(image(48, 40, seed));
    scene.items.push(Item {
        image: id,
        x: o.0 + at.0,
        y: o.1 + at.1,
        test: DepthTest::Ignore,
        depth: ItemDepth::Profile {
            profile: Profile::Flat,
            bias: 0,
            companion: None,
        },
        paint: Paint::Row(row),
        group: Group::Effects,
        provisional: false,
        object: None,
        stack: None,
    });
    id
}

/// design: render (#112): an unchanged frame records no battlefield and shows the one drawn
/// last; a rewritten image, an added item, a moved view, a new selection (the marks' redraw
/// between ticks), a tick and a resized window each draw it anew, as a fresh drawer does, and
/// the frame after is kept again.
#[test]
fn an_unchanged_frame_keeps_the_battlefield_and_a_change_draws_it_anew() {
    use crate::mapview::combat::{BRACKET_SPRITE, OVERLAY_PALETTE, PIP_SPRITE};
    use glasnost_fmt::shp::{ShpWriter, WriteFrame};
    let Some((device, queue)) = device() else {
        return;
    };
    let mut c = client((800, 600), 100);
    start_by_keyboard(&mut c);
    let unit = own(&c, Kind::Vehicle)[0];
    let s = session_mut(&mut c);
    // A selected unit's bracket and pips (S-0111 claim 3), as in the play notes' tests.
    let sprite = |frames: usize, (w, h): (u32, u32)| {
        ShpWriter {
            width: w as u16,
            height: h as u16,
            frames: (0..frames)
                .map(|_| WriteFrame {
                    x: 0,
                    y: 0,
                    image: IndexedImage::new(w, h, vec![1; (w * h) as usize]).unwrap(),
                    compression: 1,
                })
                .collect(),
        }
        .write()
    };
    s.loaded.assets.insert(PIP_SPRITE, sprite(21, (4, 4)));
    s.loaded.assets.insert(BRACKET_SPRITE, sprite(2, (36, 10)));
    s.loaded.assets.insert(OVERLAY_PALETTE, vec![0; 768]);
    let first = invented_item(s, (60, 60), 1);
    let mut kept = GpuDraw::new(&device, OFFSCREEN_FORMAT);
    // The first frames settle what a game's start leaves to the first frame.
    for _ in 0..2 {
        frame(&device, &queue, &mut kept, &mut c);
    }
    let (mut last, stats) = frame(&device, &queue, &mut kept, &mut c);
    assert_eq!(
        stats.map.passes, 0,
        "an unchanged frame records no battlefield"
    );
    let want = fresh(&device, &queue, &mut c);
    assert_eq!(
        battlefield(&c, &last),
        battlefield(&c, &want),
        "an unchanged frame shows the battlefield drawn last"
    );
    type Change = (&'static str, bool, Box<dyn Fn(&mut Client)>);
    let changes: Vec<Change> = vec![
        (
            "an image rewritten",
            true,
            Box::new(move |c| {
                let s = session_mut(c);
                assert!(s.scene_mut().replace_image(first, image(48, 40, 77)));
            }),
        ),
        (
            "an item added",
            true,
            Box::new(|c| {
                invented_item(session_mut(c), (200, 120), 2);
            }),
        ),
        (
            "the view moved",
            true,
            Box::new(|c| {
                // The centre is limited to the map (S-0101 claim 8): a step that it allows.
                let s = session_mut(c);
                let (x, y) = s.centre();
                let moved = [
                    (24, 12),
                    (-24, -12),
                    (24, -12),
                    (-24, 12),
                    (24, 0),
                    (-24, 0),
                ]
                .into_iter()
                .any(|(dx, dy)| {
                    s.set_centre((x + dx, y + dy));
                    s.centre() != (x, y)
                });
                assert!(moved, "the view moves");
            }),
        ),
        (
            "a unit selected",
            false,
            Box::new(move |c| session_mut(c).play.selection = vec![unit]),
        ),
        ("a tick", false, Box::new(|c| session_mut(c).step())),
        (
            "the window resized",
            false,
            Box::new(|c| c.resize((720, 560))),
        ),
    ];
    for (what, shows, change) in changes {
        change(&mut c);
        let (got, stats) = frame(&device, &queue, &mut kept, &mut c);
        assert!(
            stats.map.passes > 0,
            "{what}: the battlefield is drawn anew"
        );
        let want = fresh(&device, &queue, &mut c);
        assert_eq!(
            battlefield(&c, &got),
            battlefield(&c, &want),
            "{what}: as a fresh drawer draws it"
        );
        if shows {
            assert_ne!(got, last, "{what}: the picture changed");
        }
        let (again, stats) = frame(&device, &queue, &mut kept, &mut c);
        assert_eq!(stats.map.passes, 0, "{what}: the frame after is kept");
        assert_eq!(again, got, "{what}: and shows the same");
        last = got;
    }
}
