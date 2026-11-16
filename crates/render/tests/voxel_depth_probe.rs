//! A timing probe for the voxel rasteriser's depth mode (design: render
//! (#144)): one synthetic solid block drawn with and without depth, with a
//! checksum of the images so two builds can be compared. Run in release:
//! `cargo test --release -p glasnost-render --test voxel_depth_probe -- --ignored --nocapture`.

use std::time::Instant;

use glasnost_render::voxel::{Mat34, Section, VoxelModel, VoxelQuirks, draw_model, encode_column};

/// A solid `n`³ block, every voxel of colour 64 and normal 0.
fn block(n: u8) -> VoxelModel {
    let col: Vec<(u8, u8, u8)> = (0..n).map(|z| (z, 64, 0)).collect();
    let cells = usize::from(n) * usize::from(n);
    let section = Section {
        size: [n; 3],
        min: [0.0; 3],
        max: [f32::from(n); 3],
        scale: 1.0,
        normal_mode: 2,
        columns: vec![Some(encode_column(&col, n)); cells],
    };
    VoxelModel::from_sections(vec![section], None, &VoxelQuirks::default())
}

#[test]
#[ignore = "timing probe; run by hand in release"]
fn voxel_depth_probe() {
    let model = block(40);
    // design: render (#144): a turned view, so voxels overlap on screen
    let draw = Mat34([
        [0.7, -0.7, 0.0, 0.0],
        [0.4, 0.4, -0.8, 0.0],
        [0.6, 0.6, 0.5, 0.0],
    ]);
    let quirks = VoxelQuirks::default();
    let rounds = 200;
    for depth in [false, true] {
        let mut sum = 0u64;
        let start = Instant::now();
        for _ in 0..rounds {
            let img = draw_model(&model, 0, &draw, None, depth, &quirks).expect("drawn");
            sum = sum.wrapping_add(
                img.pixels
                    .iter()
                    .enumerate()
                    .map(|(i, &p)| (i as u64 + 1) * u64::from(p))
                    .sum::<u64>(),
            );
        }
        let ms = start.elapsed().as_secs_f64() * 1000.0 / f64::from(rounds);
        println!("depth={depth} ms_per_draw={ms:.4} checksum={sum}");
    }
}
