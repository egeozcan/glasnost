//! Renders the palette-indexed quad offscreen and compares every pixel with
//! the CPU reference. Needs a GPU adapter (a software one such as lavapipe or
//! llvmpipe is fine). Without one the test is skipped, unless
//! `GLASNOST_REQUIRE_GPU=1` is set (CI sets it), in which case it fails.

#![cfg(not(target_arch = "wasm32"))]

use glasnost_fmt::synth::{checkerboard, ramp_palette};
use glasnost_render::gpu::{self, BackendChoice};
use glasnost_render::offscreen::{OFFSCREEN_FORMAT, render_rgba};
use glasnost_render::palette_quad::{PaletteQuad, QuadParams, cycled_index};

fn gpu_required() -> bool {
    std::env::var("GLASNOST_REQUIRE_GPU").is_ok_and(|v| v == "1")
}

fn device() -> Option<(wgpu::Device, wgpu::Queue, String)> {
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
        let info = adapter.get_info();
        let label = format!("{} on {}", gpu::backend_label(info.backend), info.name);
        let (device, queue) = gpu::request_device(&adapter).await.expect("device opens");
        Some((device, queue, label))
    })
}

#[test]
fn quad_pixels_match_the_palette_lookup() {
    let Some((device, queue, label)) = device() else {
        return;
    };
    eprintln!("rendering on {label}");

    // Width 64 keeps rows at 256 bytes; height is deliberately different.
    let (width, height) = (64, 40);
    let image = checkerboard(width, height, 8);
    let palettes = [ramp_palette(0), ramp_palette(5)];
    let quad = PaletteQuad::new(&device, &queue, OFFSCREEN_FORMAT, &image, &palettes)
        .expect("quad builds within core limits");

    for (row, cycle) in [(0u32, 0u32), (1, 3)] {
        quad.set_params(
            &queue,
            &QuadParams {
                scale: [1.0, 1.0],
                palette_row: row,
                cycle,
            },
        );
        let pixels = render_rgba(&device, &queue, width, height, |pass| quad.draw(pass));

        for y in 0..height {
            for x in 0..width {
                let index = cycled_index(image.get(x, y).unwrap(), cycle);
                let c = palettes[row as usize].colors[usize::from(index)];
                let at = ((y * width + x) * 4) as usize;
                assert_eq!(
                    &pixels[at..at + 4],
                    &[c.r, c.g, c.b, 255],
                    "pixel ({x},{y}) with palette row {row}, cycle {cycle}"
                );
            }
        }
    }
}
