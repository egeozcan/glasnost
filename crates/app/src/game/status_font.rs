//! The game bitmap font as status text uses it (S-0213 claim 25 and Import
//! mapping; layout S-0014): each 16-bit text unit's glyph, or the generated
//! fallback for a missing unit, and their widths.

use glasnost_fmt::fnt::{Font, Mask};
use glasnost_fmt::image::IndexedImage;
use glasnost_ui::draw::{DrawList, colour};
use glasnost_ui::geom::Rect;
use glasnost_ui::status;
use glasnost_vfs::Vfs;

use super::paint::Painter;
use super::play::Message;

/// The font file's name, resolved through the ordinary file precedence.
// spec: S-0005 claim 18a (status text uses game.fnt, resolved by claims 1 to 8), S-0213 Import mapping (GAME.FNT)
pub const FILE: &str = "GAME.FNT";

/// Sprite id of the status glyphs in the painter: frame *j* is glyph *j*,
/// [`FALLBACK_FRAME`] the generated fallback.
// nocite: our own sprite id, between the cursor's and the icons'
pub const SPRITE: u32 = 950;
/// The status glyphs' frame of the generated fallback.
// nocite: our own frame number, past every 16-bit glyph number
pub const FALLBACK_FRAME: u32 = u32::MAX;
/// Distance between status-line origins, pixels.
// spec: S-0213 claim 25 (a fixed 19 pixels between line origins, independent of the imported font's line pitch and bitmap height)
pub const LINE_STEP: i32 = 19;
/// The first status line's left offset from the battlefield's left edge.
// spec: S-0213 claim 25 (the first line begins 3 pixels right of the battlefield's left edge, at its top edge)
pub const LEFT: i32 = 3;

/// What a text unit draws.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Glyph {
    /// The mapped glyph with this zero-based number.
    Mapped(usize),
    /// The generated fallback (the complement of the mapped capital X).
    Fallback,
}

/// A loaded game font with its fallback.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatusFont {
    font: Font,
    fallback: Mask,
}

impl StatusFont {
    /// Wraps `font`; a font without a mapped capital X has no fallback and
    /// is rejected as a whole, as a missing font (S-0014 claim 6, S-0213
    /// claim 25).
    pub fn new(font: Font) -> Result<Self, String> {
        // spec: S-0014 claim 6 (an absent mapping for capital X rejects the entire font), S-0213 Import mapping
        let fallback = font
            .fallback()
            .ok_or_else(|| "it maps no capital X for the fallback glyph".to_owned())?;
        Ok(Self { font, fallback })
    }

    /// Parses and wraps a font file (S-0014); a file outside the valid
    /// scope is rejected whole (claim 6).
    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        Self::new(Font::parse(bytes).map_err(|e| e.to_string())?)
    }

    /// The font as parsed, for its other use: the control-group number
    /// (S-0111 claim 17).
    pub fn font(&self) -> &Font {
        &self.font
    }

    /// The common bitmap height in pixels (S-0014 claim 1).
    pub fn height(&self) -> usize {
        self.font.height
    }

    /// The line pitch in pixels (S-0014 claim 1).
    pub fn line_pitch(&self) -> u32 {
        self.font.line_pitch
    }

    /// How many glyph rows, from row 0, a status line may paint:
    /// min(*H*, *P*) for bitmap height *H* and line pitch *P* (S-0213
    /// claim 25, Vertical metrics). Exact integers; zero pitch paints none.
    pub fn painted_rows(&self) -> u64 {
        // spec: S-0213 claim 25 (a glyph row r is eligible exactly when 0 ≤ r < min(H, P); our engine excludes the original's extra row at P, with no quirk switch)
        (self.font.height as u64).min(u64::from(self.font.line_pitch))
    }

    /// What text unit `c` draws: its mapped glyph, or the fallback when
    /// its selector is zero (S-0213 claim 25, S-0014 claim 4).
    pub fn glyph(&self, c: u16) -> Glyph {
        // spec: S-0213 claim 25 (each unsigned 16-bit unit selects the mapping; zero means missing, v selects glyph v − 1; a missing glyph uses the generated fallback)
        self.font
            .glyph_number(c)
            .map_or(Glyph::Fallback, Glyph::Mapped)
    }

    /// Text unit `c`'s glyph width in pixels: the mapped glyph's stored
    /// width, or the capital X's for a missing unit (S-0213 claim 25). The
    /// advance adds one pixel (`glasnost_ui::status::advance`).
    pub fn width(&self, c: u16) -> i32 {
        match self.glyph(c) {
            // spec: S-0213 claim 25 (individual glyph widths; a present zero-width glyph is not missing)
            Glyph::Mapped(j) => self.font.width(j).map_or(0, i32::from),
            // spec: S-0213 claim 25 (the fallback's width is the capital X's stored width)
            Glyph::Fallback => self.fallback.width as i32,
        }
    }

    /// The mask `g` draws (S-0014 claims 3 and 4).
    pub fn mask(&self, g: Glyph) -> Option<Mask> {
        match g {
            Glyph::Mapped(j) => self.font.mask(j),
            Glyph::Fallback => Some(self.fallback.clone()),
        }
    }
}

/// The status-glyph sprite frame of `g`.
pub fn frame(g: Glyph) -> u32 {
    match g {
        Glyph::Mapped(j) => j as u32,
        Glyph::Fallback => FALLBACK_FRAME,
    }
}

/// Registers with `painter` the glyphs `messages` draw, each once per
/// painter: one-bit masks through a palette row (made on first use, kept in
/// `row`) whose index 0 is transparent and index 1 the foreground (S-0014
/// claims 3 to 5). A zero-width glyph has no pixels and nothing to
/// register.
pub fn register(
    painter: &mut Painter,
    font: &StatusFont,
    messages: &[Message],
    row: &mut Option<u32>,
) {
    // Our own messages use our text colour: no route-refusal notice is shown yet.
    let row = *row.get_or_insert_with(|| ink_row(painter, colour::TEXT));
    let units = messages.iter().flat_map(|m| m.units.iter().copied());
    register_glyphs(painter, font, units, SPRITE, row);
}

/// A palette row for one-bit glyph masks in `ink`: index 0 transparent,
/// index 1 the foreground (S-0014 claim 5).
pub fn ink_row(painter: &mut Painter, ink: [u8; 4]) -> u32 {
    let mut pal = [0u8; 1024];
    // spec: S-0014 claim 5 (a one bit paints the line's foreground; a zero bit leaves the destination unchanged; there is no font palette)
    pal[4..8].copy_from_slice(&ink);
    painter.add_palette(pal)
}

/// Registers with `painter`, under sprite id `sprite` through palette row
/// `row`, the glyphs the text units `units` draw, each once.
pub fn register_glyphs(
    painter: &mut Painter,
    font: &StatusFont,
    units: impl IntoIterator<Item = u16>,
    sprite: u32,
    row: u32,
) {
    for c in units {
        let g = font.glyph(c);
        if painter.has_sprite(sprite, frame(g)) {
            continue;
        }
        // spec: S-0014 claim 3 (a zero-width glyph paints no pixels in our engine)
        let Some(mask) = font.mask(g).filter(|m| m.width > 0 && m.height > 0) else {
            continue;
        };
        let px = mask.bits.iter().map(|&b| u8::from(b)).collect();
        if let Ok(img) = IndexedImage::new(mask.width as u32, mask.height as u32, px) {
            painter.add_sprite(sprite, frame(g), &img, row);
        }
    }
}

/// Draws the status lines `messages` at the battlefield `m`'s top left,
/// oldest above newest, each glyph's top row at its line origin and only
/// its first min(*H*, *P*) rows painted (S-0213 claim 25, Vertical
/// metrics), clipped to `m`.
pub fn draw(d: &mut DrawList, font: &StatusFont, messages: &[Message], m: Rect) {
    // spec: S-0213 claim 25 (the pitch is each stored line's clipping height: rows 0 ≤ r < min(H, P), within the display surface; it reserves no background and clears no band)
    let rows = font.painted_rows().min(m.h.max(0) as u64) as i32;
    if rows == 0 {
        // spec: S-0213 claim 25 (a zero pitch paints no rows)
        return;
    }
    // spec: S-0213 claim 25 (left-aligned at the battlefield's top left, oldest above newest; the original's first line 3 pixels in at the top edge, a fixed 19 pixels between line origins)
    let mut y = m.y;
    for msg in messages {
        // spec: S-0213 claim 25 (stored row 0 at the line origin, row r at y + r; no baseline adjustment from H or P)
        let clip = m.intersect(&Rect::new(m.x, y, m.w, rows));
        // spec: S-0213 claim 25 (drawn with the measuring font's advances and 64-pixel tab stops; breaks draw nothing)
        for (dx, c) in status::glyphs(&msg.units, |c| font.width(c)) {
            d.sprite(SPRITE, frame(font.glyph(c)), m.x + LEFT + dx, y, clip);
        }
        y += LINE_STEP;
    }
}

/// Reads the game font for status text. The error is the importer
/// diagnostic, naming the file and the reason, for a missing font or one
/// rejected as a whole; status text is then omitted, with no partial use
/// and no substitute font (S-0213 Import mapping, S-0014 claim 6).
pub async fn read(vfs: &Vfs) -> Result<StatusFont, String> {
    // spec: S-0213 Import mapping (no original status text without a loaded font; our engine reports a missing-asset diagnostic and omits the text)
    let omitted =
        "status text and the control-group number are omitted (S-0213 claim 25, S-0111 claim 17)";
    let Some(r) = vfs.resolve(FILE) else {
        return Err(format!("{FILE}: the game font is missing; {omitted}"));
    };
    let bytes = vfs
        .read_resolved(&r)
        .await
        .map_err(|_| format!("{FILE}: the game font cannot be read; {omitted}"))?;
    // spec: S-0014 claim 6 (a resolved font outside the valid scope is treated exactly as a missing font)
    StatusFont::parse(&bytes)
        .map_err(|e| format!("{FILE}: the game font is rejected (S-0014 claim 6): {e}; {omitted}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use glasnost_fmt::fnt::synthetic_font;
    use glasnost_ui::geom::Point;

    /// spec: S-0213 claim 25 (zero selector → fallback with X's width; aliases; full 16-bit units; a present zero-width glyph keeps width 0 and is not missing)
    #[test]
    fn claim_25_widths_and_glyphs() {
        let f = StatusFont::new(synthetic_font()).unwrap();
        assert_eq!(f.glyph(65), f.glyph(97));
        assert_eq!(f.width(65), 4);
        assert_eq!(f.glyph(300), Glyph::Mapped(2));
        assert_eq!(f.width(300), 10);
        assert_eq!(f.width(44), 2, "the low byte maps elsewhere");
        assert_eq!(f.glyph(66), Glyph::Fallback);
        assert_eq!(f.width(66), 5, "the capital X's width");
        assert_eq!(f.glyph(1), Glyph::Mapped(5));
        assert_eq!(f.width(1), 0);
        assert!(f.mask(Glyph::Mapped(5)).unwrap().bits.is_empty());
        assert_eq!(f.mask(Glyph::Fallback).unwrap().width, 5);
        assert_eq!(f.height(), 5);
    }

    /// spec: S-0213 claim 25 and S-0014 claims 3 to 5 (glyphs drawn at their advances from 3 pixels in, lines 19 apart; one bits paint the foreground, zero bits leave the destination; a missing unit draws the complemented X; a zero-width glyph paints nothing; breaks draw nothing)
    #[test]
    fn claim_25_drawn_status_lines() {
        let font = StatusFont::new(synthetic_font()).unwrap();
        let line = |t: &str| Message {
            units: t.encode_utf16().collect(),
            until: 0,
        };
        // "A", a zero-width glyph (unit 1), a missing unit, then a break.
        let messages = [line("A\u{1}\u{e9}\n"), line("X")];
        let mut painter = Painter::new();
        let mut row = None;
        register(&mut painter, &font, &messages, &mut row);
        assert!(painter.has_sprite(SPRITE, 1));
        assert!(painter.has_sprite(SPRITE, FALLBACK_FRAME));
        assert!(
            !painter.has_sprite(SPRITE, 5),
            "zero width: nothing to draw"
        );
        let m = Rect::new(10, 20, 60, 50);
        let mut d = DrawList::new();
        draw(&mut d, &font, &messages, m);
        let quads = painter.quads(&d);
        let size = (80, 80);
        let px = painter.reference(&quads, &Default::default(), size);
        let at = |x: i32, y: i32| {
            let i = ((y * size.0 + x) * 4) as usize;
            [px[i], px[i + 1], px[i + 2]]
        };
        let ink = [colour::TEXT[0], colour::TEXT[1], colour::TEXT[2]];
        // "A" at (13, 20): row 0 is ".##.".
        assert_eq!(at(13, 20), [0, 0, 0], "a zero bit leaves the background");
        assert_eq!(at(14, 20), ink);
        assert_eq!(at(15, 20), ink);
        // The zero-width glyph advances 1 pixel: A (5) + 1 = 6; the
        // fallback starts at 13 + 6 = 19 and is X's complement:
        // row 0 ".###.".
        assert_eq!(at(19, 20), [0, 0, 0]);
        assert_eq!(at(20, 20), ink);
        assert_eq!(at(23, 20), [0, 0, 0]);
        assert_eq!(at(18, 20), [0, 0, 0], "nothing from the zero-width glyph");
        // The second line's X at (13, 39): row 0 "#...#".
        assert_eq!(at(13, 39), ink);
        assert_eq!(at(14, 39), [0, 0, 0]);
        assert_eq!(at(17, 39), ink);
        // Every drawn pixel lies inside the battlefield.
        let outside = (0..size.1)
            .flat_map(|y| (0..size.0).map(move |x| (x, y)))
            .filter(|&(x, y)| !m.contains(Point::new(x, y)))
            .any(|(x, y)| at(x, y) != [0, 0, 0]);
        assert!(!outside);
    }

    /// spec: S-0014 claim 6 and S-0213 Import mapping (a font without a mapped capital X, or an
    /// incomplete one, is rejected as a whole, even when the text to draw needs no fallback)
    #[test]
    fn claim_6_a_font_without_x_is_not_used() {
        let mut font = synthetic_font();
        font.map_unit(88, None);
        assert!(StatusFont::new(font.clone()).is_err());
        assert!(StatusFont::parse(&font.write()).is_err());
        assert!(StatusFont::parse(b"fonT").is_err());
    }

    /// A font of height `h` and pitch `p`, 16 pixels per row, whose `A`
    /// (also the capital X) is 3 pixels wide with a set pixel in column
    /// `r mod 3` of every row `r`, so each row is told apart by its column.
    fn tall_font(h: usize, p: u32, blank_top: usize) -> StatusFont {
        let mut f = Font::empty(2, h, p);
        let rows: Vec<String> = (0..h)
            .map(|r| {
                if r < blank_top {
                    String::new()
                } else {
                    (0..3).map(|x| if x == r % 3 { '#' } else { '.' }).collect()
                }
            })
            .collect();
        let rows: Vec<&str> = rows.iter().map(String::as_str).collect();
        let j = f.push_glyph(3, &rows);
        f.map_unit(65, Some(j));
        f.map_unit(88, Some(j));
        StatusFont::new(f).unwrap()
    }

    /// The painted rows of the column-marked glyphs drawn for `messages`
    /// with `font` in a 40 × 120 battlefield at (0, 0): for each screen row,
    /// the painted column offsets from the glyph's left edge.
    fn painted(font: &StatusFont, messages: &[Message]) -> Vec<Vec<i32>> {
        let mut painter = Painter::new();
        let mut row = None;
        register(&mut painter, font, messages, &mut row);
        let m = Rect::new(0, 0, 40, 120);
        let mut d = DrawList::new();
        draw(&mut d, font, messages, m);
        let quads = painter.quads(&d);
        let size = (40, 120);
        let px = painter.reference(&quads, &Default::default(), size);
        (0..size.1)
            .map(|y| {
                (0..3)
                    .filter(|&x| px[(((y * size.0) + LEFT + x) * 4) as usize] != 0)
                    .collect()
            })
            .collect()
    }

    fn line(t: &str) -> Message {
        Message {
            units: t.encode_utf16().collect(),
            until: 0,
        }
    }

    /// spec: S-0213 claim 25, Vertical metrics, and Scenario O21 d (row 0 at the line origin;
    /// origins a fixed 19 pixels apart whatever H and P; P clips each line to rows r < min(H, P),
    /// excluding the original's extra row at P; a zero pitch paints nothing)
    #[test]
    fn claim_25_pitch_clips_rows_without_moving_lines() {
        let h = 12;
        for p in [0u32, 5, 12, 30] {
            let font = tall_font(h, p, 0);
            let rows = painted(&font, &[line("A"), line("A")]);
            let shown = (h as u32).min(p) as usize;
            for origin in [0usize, 19] {
                for r in 0..19 {
                    let expect: Vec<i32> = if r < shown {
                        vec![(r % 3) as i32]
                    } else {
                        vec![]
                    };
                    assert_eq!(
                        rows[origin + r],
                        expect,
                        "P = {p}, origin {origin}, row {r}"
                    );
                }
            }
            assert!(rows[38..].iter().all(Vec::is_empty), "P = {p}");
        }
    }

    /// spec: S-0213 claim 25, Vertical metrics (blank leading rows stay blank at their positions:
    /// no centring or baseline shift)
    #[test]
    fn claim_25_blank_leading_rows_are_not_shifted() {
        let font = tall_font(8, 8, 3);
        let rows = painted(&font, &[line("A")]);
        for (r, row) in rows.iter().enumerate().take(8) {
            if r < 3 {
                assert!(row.is_empty(), "row {r}");
            } else {
                assert_eq!(*row, [(r % 3) as i32], "row {r}");
            }
        }
    }

    /// spec: S-0213 claim 25, Vertical metrics, and Scenario O21 d (with H and P above 19 a tall
    /// mask overlaps the next line's band, which still starts 19 pixels down; clear bits stay
    /// transparent and no band is cleared)
    #[test]
    fn claim_25_tall_masks_overlap_without_respacing() {
        let font = tall_font(30, 25, 0);
        let rows = painted(&font, &[line("A"), line("A")]);
        // Rows 19 to 24 hold the first line's rows 19 to 24 and the second
        // line's rows 0 to 5: both marks paint, neither clears the other.
        for (y, row) in rows.iter().enumerate().take(25).skip(19) {
            let mut expect = vec![(y % 3) as i32, ((y - 19) % 3) as i32];
            expect.sort();
            assert_eq!(*row, expect, "row {y}");
        }
        // The second line paints its rows 0 to 24 and nothing below them.
        for (y, row) in rows.iter().enumerate().take(44).skip(25) {
            assert_eq!(*row, [((y - 19) % 3) as i32], "row {y}");
        }
        assert!(rows[44..].iter().all(Vec::is_empty));
    }
}
