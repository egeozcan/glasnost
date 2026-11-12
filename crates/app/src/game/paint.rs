//! Turns the interface's draw lists (`glasnost_ui::draw`) into quads of
//! the 2D batch (`glasnost_render::batch2d`): our bitmap font's glyphs and
//! the registered sprite frames live in one index atlas, RGBA images (the
//! battlefield, the radar) are drawn whole. The same quads are drawn on
//! the GPU by the client and on the CPU by [`Painter::reference`] for
//! tests and local snapshots.

use std::collections::{BTreeMap, BTreeSet};

use glasnost_fmt::image::IndexedImage;
use glasnost_render::batch2d::{self, Atlas, CpuImage, Quad};
use glasnost_ui::draw::{DrawList, Prim, Rgba};
use glasnost_ui::font;
use glasnost_ui::geom::Rect;

/// A registered sprite frame: its atlas region and palette row.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SpriteFrame {
    region: [i32; 4],
    row: u32,
}

/// The atlas, palettes and images the quads refer to.
pub struct Painter {
    /// Glyphs and sprite frames (palette indices).
    pub atlas: Atlas,
    /// Palette rows (256 RGBA entries each).
    pub palettes: Vec<[u8; 1024]>,
    /// Whether the atlas or palettes changed since the GPU copy.
    pub dirty: bool,
    glyphs: BTreeMap<char, [i32; 4]>,
    sprites: BTreeMap<(u32, u32), SpriteFrame>,
    /// Sprite frames the full atlas did not take (design: render (#103)).
    failed: BTreeSet<(u32, u32)>,
    images: BTreeMap<u32, (i32, i32)>,
}

impl Default for Painter {
    fn default() -> Self {
        Self::new()
    }
}

fn r4(r: Rect) -> [i32; 4] {
    [r.x, r.y, r.w, r.h]
}

impl Painter {
    /// A painter with the interface font in its atlas and one grey palette
    /// row.
    pub fn new() -> Self {
        Self::with_side(batch2d::ATLAS_SIZE)
    }

    /// A painter like [`Painter::new`] whose atlas is `side` × `side`: the
    /// GPU atlas's side (design: render (#103)).
    pub fn with_side(side: u32) -> Self {
        let mut atlas = Atlas::new(side);
        let mut glyphs = BTreeMap::new();
        for g in font::glyphs() {
            if g.width == 0 {
                continue;
            }
            let px: Vec<u8> = g.mask.iter().map(|&b| u8::from(b)).collect();
            if let Some(r) = atlas.add(g.width as u32, font::GLYPH_ROWS as u32, &px) {
                glyphs.insert(g.ch, r);
            }
        }
        let mut grey = [0u8; 1024];
        for i in 0..256 {
            grey[i * 4..i * 4 + 4].copy_from_slice(&[i as u8, i as u8, i as u8, 255]);
        }
        Self {
            atlas,
            palettes: vec![grey],
            dirty: true,
            glyphs,
            sprites: BTreeMap::new(),
            failed: BTreeSet::new(),
            images: BTreeMap::new(),
        }
    }

    /// Adds a palette row and returns its number.
    pub fn add_palette(&mut self, rgba: [u8; 1024]) -> u32 {
        self.palettes.push(rgba);
        self.dirty = true;
        (self.palettes.len() - 1) as u32
    }

    /// Registers frame `frame` of sprite `id`, drawn through palette `row`.
    /// Returns false when the atlas is full; the frame is then remembered
    /// as tried ([`Painter::tried_sprite`]) and the first such frame is
    /// logged (design: render (#103)).
    pub fn add_sprite(&mut self, id: u32, frame: u32, image: &IndexedImage, row: u32) -> bool {
        let Some(region) = self
            .atlas
            .add(image.width(), image.height(), image.pixels())
        else {
            if self.failed.is_empty() {
                log::warn!(
                    "the sprite atlas of {0} × {0} is full: a {1} × {2} frame and any later ones are not drawn",
                    self.atlas.size(),
                    image.width(),
                    image.height()
                );
            }
            self.failed.insert((id, frame));
            return false;
        };
        self.sprites
            .insert((id, frame), SpriteFrame { region, row });
        self.dirty = true;
        true
    }

    /// Whether frame `frame` of sprite `id` is registered.
    pub fn has_sprite(&self, id: u32, frame: u32) -> bool {
        self.sprites.contains_key(&(id, frame))
    }

    /// Whether frame `frame` of sprite `id` was registered or failed to be:
    /// a frame the full atlas did not take is not decoded and offered again
    /// each frame (design: render (#103)).
    pub fn tried_sprite(&self, id: u32, frame: u32) -> bool {
        self.has_sprite(id, frame) || self.failed.contains(&(id, frame))
    }

    /// Records the size of image `id` (its pixels are the client's).
    pub fn set_image_size(&mut self, id: u32, size: (i32, i32)) {
        self.images.insert(id, size);
    }

    /// The quads of `list`, in order.
    pub fn quads(&self, list: &DrawList) -> Vec<Quad> {
        let mut out = Vec::new();
        for p in &list.prims {
            match p {
                Prim::Fill { rect, rgba } => {
                    if !rect.is_empty() {
                        out.push(Quad::solid(r4(*rect), *rgba));
                    }
                }
                Prim::Frame { rect, rgba } => {
                    let r = *rect;
                    if r.is_empty() {
                        continue;
                    }
                    for e in [
                        Rect::new(r.x, r.y, r.w, 1),
                        Rect::new(r.x, r.bottom() - 1, r.w, 1),
                        Rect::new(r.x, r.y + 1, 1, r.h - 2),
                        Rect::new(r.right() - 1, r.y + 1, 1, r.h - 2),
                    ] {
                        if !e.is_empty() {
                            out.push(Quad::solid(r4(e), *rgba));
                        }
                    }
                }
                Prim::Text {
                    x,
                    y,
                    text,
                    rgba,
                    outline,
                    clip,
                } => {
                    // The outline first, at the eight neighbouring offsets
                    // of the whole line, so that no outline covers ink.
                    // design: readable text and display scale (lead's request 2026-11-10)
                    if let Some(o) = outline {
                        for (dx, dy) in [
                            (-1, -1),
                            (0, -1),
                            (1, -1),
                            (-1, 0),
                            (1, 0),
                            (-1, 1),
                            (0, 1),
                            (1, 1),
                        ] {
                            self.glyph_quads(&mut out, (x + dx, y + dy), text, *o, *clip);
                        }
                    }
                    self.glyph_quads(&mut out, (*x, *y), text, *rgba, *clip);
                }
                Prim::Sprite {
                    id,
                    frame,
                    x,
                    y,
                    clip,
                } => {
                    if let Some(s) = self.sprites.get(&(*id, *frame)) {
                        out.extend(batch2d::clip(
                            Quad::sprite(*x, *y, s.region, s.row),
                            r4(*clip),
                        ));
                    }
                }
                Prim::Image { id, rect } => {
                    if let Some(&size) = self.images.get(id) {
                        out.push(Quad::image(*id, size, r4(*rect)));
                    }
                }
                Prim::Line { from, to, rgba } => out.extend(batch2d::line(*from, *to, *rgba)),
                Prim::Layer { scale, list } => {
                    // Clipped in logical pixels, then each destination
                    // scaled; the sources stay, so the 2D batch's nearest
                    // sampling enlarges glyphs, sprites and images alike.
                    for mut q in self.quads(list) {
                        let r = scale.rect(Rect::new(q.dst[0], q.dst[1], q.dst[2], q.dst[3]));
                        if !r.is_empty() {
                            q.dst = r4(r);
                            out.push(q);
                        }
                    }
                }
            }
        }
        out
    }

    /// The glyph quads of one line of `text` at `at`, in `rgba`, clipped.
    fn glyph_quads(&self, out: &mut Vec<Quad>, at: (i32, i32), text: &str, rgba: Rgba, clip: Rect) {
        let mut cx = at.0;
        for c in text.chars() {
            if let Some(region) = self.glyphs.get(&c).or_else(|| self.glyphs.get(&'?'))
                && c != ' '
            {
                out.extend(batch2d::clip(
                    Quad::glyph(cx, at.1, *region, rgba),
                    r4(clip),
                ));
            }
            cx += font::advance(c);
        }
    }

    /// The CPU drawing of `quads` over black, for tests and snapshots.
    pub fn reference(
        &self,
        quads: &[Quad],
        images: &BTreeMap<u32, CpuImage>,
        size: (i32, i32),
    ) -> Vec<u8> {
        batch2d::reference(quads, &self.atlas, &self.palettes, images, size.0, size.1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glasnost_ui::draw::colour;

    #[test]
    fn text_frames_and_images_become_quads() {
        let mut p = Painter::new();
        let mut d = DrawList::new();
        d.fill(Rect::new(0, 0, 40, 20), colour::PANEL);
        d.frame(Rect::new(0, 0, 40, 20), colour::EDGE);
        d.text(2, 2, "Hi", colour::TEXT, Rect::new(0, 0, 40, 20));
        d.image(9, Rect::new(10, 10, 8, 8));
        let q = p.quads(&d);
        // One fill, four edges, two glyphs; the unregistered image is left out.
        assert_eq!(q.len(), 7);
        p.set_image_size(9, (2, 2));
        let q = p.quads(&d);
        assert_eq!(q.len(), 8);
        let mut images = BTreeMap::new();
        images.insert(
            9,
            CpuImage {
                w: 2,
                h: 2,
                rgba: vec![255; 16],
            },
        );
        let px = p.reference(&q, &images, (40, 20));
        // The image's corner is white; the panel's interior is the panel colour.
        assert_eq!(
            &px[(12 * 40 + 12) * 4..(12 * 40 + 12) * 4 + 3],
            &[255, 255, 255]
        );
        assert_eq!(
            &px[(18 * 40 + 30) * 4..(18 * 40 + 30) * 4 + 3],
            &colour::PANEL[..3]
        );
    }

    /// Outlined text (design: readable text and display scale, lead's
    /// request 2026-11-10): the ink stays where it was and every pixel
    /// next to it that is not ink is the outline colour.
    #[test]
    fn outlined_text_has_a_dark_ring_around_its_ink() {
        let p = Painter::new();
        let clip = Rect::new(0, 0, 20, 14);
        let mut plain = DrawList::new();
        plain.fill(clip, [200, 200, 200, 255]);
        plain.text(3, 2, "I", colour::TEXT, clip);
        let mut ringed = DrawList::new();
        ringed.fill(clip, [200, 200, 200, 255]);
        ringed.text_outlined(3, 2, "I", colour::TEXT, clip);
        let q = p.quads(&ringed);
        // One fill, eight outline glyphs, one ink glyph.
        assert_eq!(q.len(), 10);
        let a = p.reference(&p.quads(&plain), &BTreeMap::new(), (20, 14));
        let b = p.reference(&q, &BTreeMap::new(), (20, 14));
        let ink = |px: &[u8], x: i32, y: i32| {
            let at = ((y * 20 + x) * 4) as usize;
            (0..20).contains(&x) && (0..14).contains(&y) && px[at..at + 3] == colour::TEXT[..3]
        };
        let (mut inked, mut ringed_px) = (0, 0);
        for y in 0..14 {
            for x in 0..20 {
                let at = ((y * 20 + x) * 4) as usize;
                assert_eq!(ink(&a, x, y), ink(&b, x, y), "the ink at {x},{y}");
                inked += i32::from(ink(&a, x, y));
                let near = (-1..=1).any(|dy| (-1..=1).any(|dx| ink(&a, x + dx, y + dy)));
                if near && !ink(&a, x, y) {
                    assert_eq!(&b[at..at + 3], &colour::OUTLINE[..3], "the ring at {x},{y}");
                    ringed_px += 1;
                }
            }
        }
        assert!(
            inked > 0 && ringed_px > inked,
            "{inked} ink, {ringed_px} ring"
        );
    }

    #[test]
    fn a_scaled_layer_enlarges_its_quads_after_clipping() {
        use glasnost_ui::geom::Scale;
        let p = Painter::new();
        let mut inner = DrawList::new();
        inner.fill(Rect::new(10, 4, 5, 3), colour::PANEL);
        inner.text(0, 0, "Hi", colour::TEXT, Rect::new(0, 0, 3, 9));
        let plain = p.quads(&inner);
        let s = Scale::fit(200, (1280, 960));
        let mut d = DrawList::new();
        d.scaled(s, inner);
        let q = p.quads(&d);
        assert_eq!(q.len(), plain.len());
        assert_eq!(q[0].dst, [20, 8, 10, 6]);
        // The glyph clipped to 3 logical columns covers 6 screen columns and
        // keeps its 3 source columns.
        assert_eq!(q[1].dst[2], 6);
        assert_eq!(q[1].src[2], 3);
        assert_eq!(q[1].src, plain[1].src);
    }

    /// design: render (#103): a sprite frame the atlas cannot take is
    /// remembered as tried, so callers do not decode it again each frame;
    /// the atlas has the side asked for and still holds the font.
    #[test]
    fn a_full_atlas_is_tried_once() {
        let mut p = Painter::with_side(256);
        assert_eq!(p.atlas.size(), 256);
        let mut d = DrawList::new();
        d.text(0, 0, "Hi", colour::TEXT, Rect::new(0, 0, 40, 20));
        assert_eq!(p.quads(&d).len(), 2, "the font is in the atlas");
        let big = IndexedImage::new(300, 4, vec![1; 1200]).unwrap();
        assert!(!p.add_sprite(7, 0, &big, 0));
        assert!(!p.has_sprite(7, 0));
        assert!(p.tried_sprite(7, 0));
        assert!(!p.tried_sprite(7, 1));
        let small = IndexedImage::new(4, 4, vec![1; 16]).unwrap();
        assert!(p.add_sprite(7, 1, &small, 0));
        assert!(p.has_sprite(7, 1) && p.tried_sprite(7, 1));
    }

    #[test]
    fn sprites_are_clipped_and_use_their_row() {
        let mut p = Painter::new();
        let mut pal = [0u8; 1024];
        pal[4..8].copy_from_slice(&[10, 20, 30, 255]);
        let row = p.add_palette(pal);
        let img = IndexedImage::new(4, 4, vec![1; 16]).unwrap();
        assert!(p.add_sprite(3, 0, &img, row));
        let mut d = DrawList::new();
        d.sprite(3, 0, 0, 0, Rect::new(0, 0, 2, 4));
        let q = p.quads(&d);
        assert_eq!(q.len(), 1);
        assert_eq!(q[0].dst, [0, 0, 2, 4]);
        let px = p.reference(&q, &BTreeMap::new(), (4, 4));
        assert_eq!(&px[0..3], &[10, 20, 30]);
        assert_eq!(
            &px[3 * 4..3 * 4 + 3],
            &[0, 0, 0],
            "clipped column stays black"
        );
    }
}
