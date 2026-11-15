//! Spot groups whose pixels repeat on the GPU map path (design: render
//! (#115, #116)): spots that do not read the target (beam flashes, bolts,
//! opaque beam cores) take no copy of it and start no pass, and spots that
//! read it (additive and blended beam pixels) take one copy for each depth
//! of repetition, not one at every repeated pixel; the picture is the CPU
//! reference's exactly (S-0518 claims 20 and 23, F7 and F8; S-0514 claim 25,
//! F8). Needs a GPU adapter (software is fine); skipped without one unless
//! `GLASNOST_REQUIRE_GPU=1`.

#![cfg(not(target_arch = "wasm32"))]

use glasnost_fmt::image::IndexedImage;
use glasnost_render::compose::{ComposeQuirks, DEPTH_REFERENCE, DepthTest, Paint, Profile};
use glasnost_render::gpu;
use glasnost_render::map_gpu::{COLOR_FORMAT, MapRenderer, MapStats, targets};
use glasnost_render::offscreen::read_texture;
use glasnost_render::scene::{Filter, Group, Item, ItemDepth, Scene, View};
use glasnost_render::spots::{Spot, SpotGroup, SpotKind, easing};

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

const SIZE: (u32, u32) = (96, 48);

/// A ground of varied colours over the whole view (a depth writer), so
/// that what a spot reads shows in its pixel.
fn ground() -> Scene {
    let mut s = Scene::default();
    let row = s.add_row(std::array::from_fn(|i| {
        (i as u16).wrapping_mul(2971).wrapping_add(0x4321)
    }));
    let px = (0..SIZE.1)
        .flat_map(|y| (0..SIZE.0).map(move |x| ((x * 7 + y * 13) % 255 + 1) as u8))
        .collect();
    let image = s.add_image(IndexedImage::new(SIZE.0, SIZE.1, px).unwrap());
    s.items.push(Item {
        image,
        x: 0,
        y: 0,
        test: DepthTest::Writer,
        depth: ItemDepth::Profile {
            profile: Profile::Flat,
            bias: 0,
            companion: None,
        },
        paint: Paint::Row(row),
        group: Group::Tiles,
        provisional: false,
        object: None,
        stack: None,
    });
    s
}

/// A line of spots along row `y` in which each step draws pixel `x` and
/// then pixel `x − 1` again (as a thick beam's neighbouring samples do), so
/// every pixel but the ends is drawn twice and the repeats interleave.
fn interleaved(y: i32, kinds: impl Fn(i32, usize) -> SpotKind) -> Vec<Spot> {
    let mut out = Vec::new();
    for x in 2..90 {
        for (n, at) in [x, x - 1].into_iter().enumerate() {
            out.push(Spot {
                x: at,
                y,
                bias: [-200, -2, -40][(at % 3) as usize],
                kind: kinds(at, n),
            });
        }
    }
    out
}

/// The light-alpha values of the view: varied, with zeros.
fn alpha() -> Vec<u8> {
    (0..(SIZE.0 * SIZE.1) as usize)
        .map(|i| [127, 0, 64, 200, 127, 90][i % 6])
        .collect()
}

/// Draws `s` on the GPU and returns the pixels that differ from the CPU
/// reference and the renderer's counts.
fn draw(device: &wgpu::Device, queue: &wgpu::Queue, s: &Scene) -> (usize, MapStats) {
    let view = View {
        origin: (0, 0),
        size: SIZE,
        reference: DEPTH_REFERENCE,
    };
    let filter = Filter::default();
    let q = ComposeQuirks::default();
    let cpu = s
        .compose_lit(&view, &filter, &q, Some(alpha()))
        .to_rgba();
    let plain = ground()
        .compose_lit(&view, &filter, &q, Some(alpha()))
        .to_rgba();
    assert!(
        cpu.chunks(4).zip(plain.chunks(4)).filter(|(a, b)| a != b).count() > 100,
        "the spots show"
    );
    let mut r = MapRenderer::new(device, queue, s, COLOR_FORMAT).expect("scene uploads");
    let (color, depth) = targets(device, SIZE, COLOR_FORMAT);
    let mut encoder = device.create_command_encoder(&Default::default());
    let _ = r.take_stats();
    r.render_lit(
        device,
        queue,
        &mut encoder,
        &color,
        &depth.create_view(&Default::default()),
        s,
        &view,
        &filter,
        &q,
        Some(&alpha()),
    );
    queue.submit([encoder.finish()]);
    let stats = r.take_stats();
    let gpu = read_texture(device, queue, &color, SIZE.0, SIZE.1);
    let differing = cpu
        .chunks(4)
        .zip(gpu.chunks(4))
        .filter(|(a, b)| a[..3] != b[..3])
        .count();
    (differing, stats)
}

fn group(spots: Vec<Spot>) -> SpotGroup {
    SpotGroup {
        at: usize::MAX,
        group: Group::Effects,
        provisional: false,
        spots,
    }
}

/// design: render (#115): flashes and opaque pixels that repeat, interleaved, and a line of
/// opaque pixels over a line of flashes, take no copy of the target and start no pass: one
/// draw after the ground, exactly as the CPU draws them.
#[test]
fn repeated_flashes_take_no_copy() {
    let Some((device, queue)) = device() else {
        return;
    };
    let mut s = ground();
    let flash = |at: i32, n: usize| SpotKind::Flash {
        rgb: [(at * 37 % 256) as u8, (n * 90 + 40) as u8, (at * 11 % 256) as u8],
    };
    let mut spots = interleaved(10, flash);
    spots.extend(interleaved(10, |at, n| SpotKind::Opaque {
        colour: (at as u16).wrapping_mul(2971) ^ (n as u16 * 0x0F0F),
    }));
    spots.extend(interleaved(30, flash));
    s.spots.push(group(spots));
    let (differing, stats) = draw(&device, &queue, &s);
    assert_eq!(differing, 0, "as the CPU draws them");
    assert_eq!(stats.copies, 0, "backdrop copies");
    assert_eq!(stats.passes, 1, "passes");
    assert_eq!(stats.draws, 2, "the ground's draw and the spots'");
}

/// design: render (#116): additive and blended pixels drawn twice each, interleaved, with
/// opaque pixels and flashes between them, take one copy of the target for each depth of
/// repetition (two here), exactly as the CPU draws them.
#[test]
fn repeated_beam_pixels_take_a_copy_per_depth() {
    let Some((device, queue)) = device() else {
        return;
    };
    let mut s = ground();
    let reader = |at: i32, n: usize| {
        let rgb = [(at * 37 % 256) as u8, (at * 91 % 256) as u8, (n * 120 + 30) as u8];
        if at % 2 == 0 {
            SpotKind::Additive { rgb }
        } else {
            let w = (at as u32 * 29) % 257;
            SpotKind::Blended {
                rgb,
                line: easing(w),
                screen: easing(256 - w),
                alpha_from: (0, -1),
            }
        }
    };
    let mut spots = interleaved(20, reader);
    // Pixels that do not read the target, on another row and at the readers' own pixels after
    // them: they add no depth of repetition.
    spots.extend(interleaved(24, |at, n| SpotKind::Flash {
        rgb: [(at * 5 % 256) as u8, (n * 70) as u8, 200],
    }));
    spots.extend((2..40).map(|x| Spot {
        x,
        y: 20,
        bias: -300,
        kind: SpotKind::Opaque {
            colour: (x as u16).wrapping_mul(977),
        },
    }));
    s.spots.push(group(spots));
    let (differing, stats) = draw(&device, &queue, &s);
    assert_eq!(differing, 0, "as the CPU draws them");
    assert_eq!(stats.copies, 2, "backdrop copies");
    assert_eq!(stats.passes, 3, "passes");
}
