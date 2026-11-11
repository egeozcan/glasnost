//! Turning decoded assets into pictures for the viewer.
//!
//! Everything here is presentation for a development tool: palette
//! resolution happens on the CPU, and the voxel view is a simple preview
//! (orthographic, one light-free colour per voxel, limbs placed by their
//! bounds). The faithful voxel renderer waits for its spec.

use glasnost_fmt::image::{IndexedImage, Palette};
use glasnost_fmt::vxl::Vxl;
use glasnost_render::palette::resolve_rgba;

/// An RGBA picture.
pub struct Rgba {
    /// Width.
    pub width: usize,
    /// Height.
    pub height: usize,
    /// `width × height × 4` bytes.
    pub pixels: Vec<u8>,
}

/// Gap between copies in a strip of remap rows.
const STRIP_GAP: usize = 6;

/// Draws `image` once per palette row, side by side (index 0 transparent
/// when `transparent_zero`), wrapped onto further lines so that the
/// picture is at most `max_side` pixels wide where one copy fits.
///
/// design: render (#109): a strip wider than the device's texture limit
/// would fail to upload, so the copies wrap.
pub fn strip(
    image: &IndexedImage,
    rows: &[Palette],
    transparent_zero: bool,
    max_side: usize,
) -> Rgba {
    let (w, h) = (image.width() as usize, image.height() as usize);
    let n = rows.len().max(1);
    // Copies per line: as many as fit in `max_side`, at least one.
    let per_line = (max_side.saturating_add(STRIP_GAP) / (w + STRIP_GAP)).clamp(1, n);
    let lines = n.div_ceil(per_line);
    let width = w * per_line + STRIP_GAP * (per_line - 1);
    let height = h * lines + STRIP_GAP * (lines - 1);
    let mut pixels = vec![0u8; width * height * 4];
    for (k, row) in rows.iter().enumerate() {
        let rgba = resolve_rgba(image, row, transparent_zero);
        let x0 = (k % per_line) * (w + STRIP_GAP);
        let y0 = (k / per_line) * (h + STRIP_GAP);
        for y in 0..h {
            let src = &rgba[y * w * 4..(y + 1) * w * 4];
            let dst = ((y0 + y) * width + x0) * 4;
            pixels[dst..dst + w * 4].copy_from_slice(src);
        }
    }
    Rgba {
        width,
        height,
        pixels,
    }
}

/// Whether picture `p` fits a texture of at most `max_side` pixels a side,
/// else the preview's error.
///
/// design: render (#109): a picture over the device's texture limit is
/// not uploaded (the upload would fail); the preview says so instead.
pub fn fits(p: &Rgba, max_side: usize) -> Result<(), String> {
    if p.width > max_side || p.height > max_side {
        Err(format!(
            "the picture is {} × {} pixels, more than this device's texture limit of {max_side}",
            p.width, p.height
        ))
    } else {
        Ok(())
    }
}

/// An RGB picture with alpha: copies a decoded colour image.
pub fn rgb(width: u32, height: u32, rgb: &[u8]) -> Rgba {
    let pixels = rgb
        .as_chunks::<3>()
        .0
        .iter()
        .flat_map(|c| [c[0], c[1], c[2], 255])
        .collect();
    Rgba {
        width: width as usize,
        height: height as usize,
        pixels,
    }
}

/// A simple orthographic preview of a voxel model, rotated `angle` degrees
/// about the vertical axis and seen from above at a fixed tilt, as an
/// indexed image of `size × size` (index 0 where no voxel is).
pub fn voxel_preview(model: &Vxl, angle: i32, size: u32) -> IndexedImage {
    let size_px = size as usize;
    let mut pixels = vec![0u8; size_px * size_px];
    let mut depth = vec![f32::NEG_INFINITY; size_px * size_px];
    // Gather voxel centres in model space: each limb's grid is spread over
    // its stated bounds.
    let mut points: Vec<([f32; 3], u8)> = Vec::new();
    for limb in &model.limbs {
        let [sx, sy, sz] = limb.size.map(|s| f32::from(s.max(1)));
        let min = limb.min.map(|v| v.get());
        let max = limb.max.map(|v| v.get());
        let span = [
            sanitize(max[0] - min[0], sx),
            sanitize(max[1] - min[1], sy),
            sanitize(max[2] - min[2], sz),
        ];
        let min = min.map(|v| if v.is_finite() { v } else { 0.0 });
        for v in &limb.voxels {
            let p = [
                min[0] + (f32::from(v.x) + 0.5) * span[0] / sx,
                min[1] + (f32::from(v.y) + 0.5) * span[1] / sy,
                min[2] + (f32::from(v.z) + 0.5) * span[2] / sz,
            ];
            points.push((p, v.color));
        }
    }
    if points.is_empty() {
        return IndexedImage::new(size, size, pixels).unwrap_or_else(|_| IndexedImage::empty());
    }
    let (sin, cos) = (angle as f32).to_radians().sin_cos();
    // Tilt: screen y mixes depth and height.
    let project = |p: [f32; 3]| {
        let rx = p[0] * cos - p[1] * sin;
        let ry = p[0] * sin + p[1] * cos;
        (rx, ry * 0.5 - p[2] * 0.87, ry + p[2])
    };
    let projected: Vec<(f32, f32, f32, u8)> = points
        .iter()
        .map(|&(p, c)| {
            let (x, y, d) = project(p);
            (x, y, d, c)
        })
        .collect();
    let (mut x0, mut x1, mut y0, mut y1) = (f32::MAX, f32::MIN, f32::MAX, f32::MIN);
    for &(x, y, _, _) in &projected {
        x0 = x0.min(x);
        x1 = x1.max(x);
        y0 = y0.min(y);
        y1 = y1.max(y);
    }
    let extent = (x1 - x0).max(y1 - y0).max(1e-3);
    let scale = (size as f32 - 8.0) / extent;
    let dot = ((scale * 0.9).ceil() as usize).clamp(1, 8);
    for &(x, y, d, c) in &projected {
        let px = ((x - x0) * scale + 4.0) as usize;
        let py = ((y - y0) * scale + 4.0) as usize;
        for dy in 0..dot {
            for dx in 0..dot {
                let (qx, qy) = (px + dx, py + dy);
                if qx < size_px && qy < size_px {
                    let i = qy * size_px + qx;
                    if d > depth[i] {
                        depth[i] = d;
                        pixels[i] = c;
                    }
                }
            }
        }
    }
    IndexedImage::new(size, size, pixels).unwrap_or_else(|_| IndexedImage::empty())
}

/// A usable extent: the stated one if finite and positive, else the grid size.
fn sanitize(extent: f32, cells: f32) -> f32 {
    if extent.is_finite() && extent > 0.0 {
        extent
    } else {
        cells
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_places_copies_side_by_side() {
        let img = IndexedImage::new(2, 1, vec![0, 1]).unwrap();
        let mut a = Palette::default();
        a.colors[1] = glasnost_fmt::image::Rgb8::new(9, 9, 9);
        let s = strip(&img, &[a.clone(), a], true, usize::MAX);
        assert_eq!(s.width, 2 * 2 + STRIP_GAP);
        assert_eq!(&s.pixels[4..8], &[9, 9, 9, 255]);
        assert_eq!(s.pixels[3], 0);
    }

    /// design: render (#109): eight copies of a 64-wide image under a 256 texture limit wrap
    /// onto lines no wider than the limit, every copy kept.
    #[test]
    fn strip_wraps_at_the_texture_limit() {
        let img = IndexedImage::new(64, 2, vec![1; 128]).unwrap();
        let mut a = Palette::default();
        a.colors[1] = glasnost_fmt::image::Rgb8::new(9, 9, 9);
        let rows = vec![a; 8];
        // Unwrapped: 8 · 64 + 7 · 6 = 554 > 256.
        assert!(fits(&strip(&img, &rows, true, usize::MAX), 256).is_err());
        let s = strip(&img, &rows, true, 256);
        // (256 + 6) / (64 + 6) = 3 copies a line, 3 lines.
        assert_eq!(
            (s.width, s.height),
            (3 * 64 + 2 * STRIP_GAP, 3 * 2 + 2 * STRIP_GAP)
        );
        assert!(fits(&s, 256).is_ok());
        let opaque = s.pixels.chunks(4).filter(|p| p[3] == 255).count();
        assert_eq!(opaque, 8 * 64 * 2);
        // One copy wider than the limit cannot fit: the preview says so.
        let wide = IndexedImage::new(300, 1, vec![1; 300]).unwrap();
        let s = strip(&wide, &[Palette::default()], true, 256);
        assert!(fits(&s, 256).is_err());
    }

    #[test]
    fn voxel_preview_draws_something() {
        let model = glasnost_fmt::vxl::synthetic_model();
        let img = voxel_preview(&model, 30, 96);
        assert!(img.pixels().iter().any(|&p| p != 0));
        assert_eq!(img.width(), 96);
    }
}
