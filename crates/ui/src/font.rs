//! Glasnost's own bitmap font: a square-cornered, stencil-like face drawn
//! for this project (our own work, GPL-3.0-or-later like the rest of the
//! repository). It is the interface font while no font reader for the
//! player's files exists.
//!
//! Each glyph is a mask of rows; capitals and digits are 7 rows tall, small
//! letters 5 rows above the baseline, with 2 rows of descender below it.
//! Glyphs are proportional: a glyph's advance is its mask width plus one
//! column of spacing.

/// Rows in a glyph mask (cap height 7 plus 2 descender rows).
pub const GLYPH_ROWS: i32 = 9;
/// Distance between baselines of successive lines.
pub const LINE_HEIGHT: i32 = 11;
/// Advance of a space.
pub const SPACE_ADVANCE: i32 = 3;
/// Columns between glyphs.
pub const SPACING: i32 = 1;

/// The masks, rows separated by `/`; `#` is ink. A missing row is empty.
///
/// The round capitals (C, G, O, Q, S) have cut corners and B a short
/// middle bar, so that they differ from the square digits 0, 5, 6, 8 and 9
/// at a glance; their widths are the old ones, so no layout moved.
// design: readable text and display scale (lead's request 2026-11-10)
const GLYPHS: &[(char, &str)] = &[
    ('A', "#####/#...#/#...#/#####/#...#/#...#/#...#"),
    ('B', "####./#...#/#...#/####./#...#/#...#/####."),
    ('C', ".####/#..../#..../#..../#..../#..../.####"),
    ('D', "####./#...#/#...#/#...#/#...#/#...#/####."),
    ('E', "#####/#..../#..../####./#..../#..../#####"),
    ('F', "#####/#..../#..../####./#..../#..../#...."),
    ('G', ".####/#..../#..../#.###/#...#/#...#/.###."),
    ('H', "#...#/#...#/#...#/#####/#...#/#...#/#...#"),
    ('I', "###/.#./.#./.#./.#./.#./###"),
    ('J', "..###/...#./...#./...#./...#./#..#./####."),
    ('K', "#...#/#..#./#.#../###../#.#../#..#./#...#"),
    ('L', "#..../#..../#..../#..../#..../#..../#####"),
    ('M', "#...#/##.##/#.#.#/#.#.#/#...#/#...#/#...#"),
    ('N', "#...#/##..#/#.#.#/#.#.#/#..##/#...#/#...#"),
    ('O', ".###./#...#/#...#/#...#/#...#/#...#/.###."),
    ('P', "#####/#...#/#...#/#####/#..../#..../#...."),
    ('Q', ".###./#...#/#...#/#...#/#.#.#/#..#./.##.#"),
    ('R', "#####/#...#/#...#/#####/#.#../#..#./#...#"),
    ('S', ".####/#..../#..../.###./....#/....#/####."),
    ('T', "#####/..#../..#../..#../..#../..#../..#.."),
    ('U', "#...#/#...#/#...#/#...#/#...#/#...#/#####"),
    ('V', "#...#/#...#/#...#/#...#/.#.#./.#.#./..#.."),
    ('W', "#...#/#...#/#...#/#.#.#/#.#.#/##.##/#...#"),
    ('X', "#...#/#...#/.#.#./..#../.#.#./#...#/#...#"),
    ('Y', "#...#/#...#/.#.#./..#../..#../..#../..#.."),
    ('Z', "#####/....#/...#./..#../.#.../#..../#####"),
    ('0', "#####/#...#/#..##/#.#.#/##..#/#...#/#####"),
    ('1', "..#../.##../..#../..#../..#../..#../.###."),
    ('2', "#####/....#/....#/#####/#..../#..../#####"),
    ('3', "#####/....#/....#/.####/....#/....#/#####"),
    ('4', "#...#/#...#/#...#/#####/....#/....#/....#"),
    ('5', "#####/#..../#..../#####/....#/....#/#####"),
    ('6', "#####/#..../#..../#####/#...#/#...#/#####"),
    ('7', "#####/....#/...#./..#../..#../..#../..#.."),
    ('8', "#####/#...#/#...#/#####/#...#/#...#/#####"),
    ('9', "#####/#...#/#...#/#####/....#/....#/#####"),
    ('a', "//####./....#/#####/#...#/#####"),
    ('b', "#..../#..../#####/#...#/#...#/#...#/#####"),
    ('c', "//#####/#..../#..../#..../#####"),
    ('d', "....#/....#/#####/#...#/#...#/#...#/#####"),
    ('e', "//#####/#...#/#####/#..../#####"),
    ('f', ".###/.#../####/.#../.#../.#../.#.."),
    ('g', "//#####/#...#/#...#/#...#/#####/....#/#####"),
    ('h', "#..../#..../#####/#...#/#...#/#...#/#...#"),
    ('i', "#//#/#/#/#/#"),
    ('j', "..#//..#/..#/..#/..#/..#/..#/###"),
    ('k', "#.../#.../#..#/#.#./##../#.#./#..#"),
    ('l', "#./#./#./#./#./#./##"),
    ('m', "//#####/#.#.#/#.#.#/#.#.#/#.#.#"),
    ('n', "//####./#...#/#...#/#...#/#...#"),
    ('o', "//#####/#...#/#...#/#...#/#####"),
    ('p', "//#####/#...#/#...#/#...#/#####/#..../#...."),
    ('q', "//#####/#...#/#...#/#...#/#####/....#/....#"),
    ('r', "//#.##/##../#.../#.../#..."),
    ('s', "//#####/#..../#####/....#/#####"),
    ('t', ".#../.#../####/.#../.#../.#../.###"),
    ('u', "//#...#/#...#/#...#/#...#/#####"),
    ('v', "//#...#/#...#/.#.#./.#.#./..#.."),
    ('w', "//#...#/#.#.#/#.#.#/#.#.#/#####"),
    ('x', "//#...#/.#.#./..#../.#.#./#...#"),
    ('y', "//#...#/#...#/#...#/#...#/#####/....#/#####"),
    ('z', "//#####/...#./..#../.#.../#####"),
    ('!', "#/#/#/#/#//#"),
    ('"', "#.#/#.#"),
    ('#', "/.#.#./#####/.#.#./#####/.#.#."),
    ('$', "..#../#####/#.#../#####/..#.#/#####/..#.."),
    ('%', "##..#/##.#./...#./..#../.#.../.#.##/#..##"),
    ('&', ".##../#..#./.##../.#..#/#.##./#..#./.##.#"),
    ('\'', "#/#"),
    ('(', ".#/#./#./#./#./#./.#"),
    (')', "#./.#/.#/.#/.#/.#/#."),
    ('*', "/#.#.#/.###./#####/.###./#.#.#"),
    ('+', "/..#../..#../#####/..#../..#.."),
    (',', "/////.#/.#/#."),
    ('-', "///###"),
    ('.', "//////#"),
    ('/', "....#/...#./...#./..#../.#.../.#.../#...."),
    (':', "//#///#"),
    (';', "//.#///.#/.#/#."),
    ('<', "...#/..#./.#../#.../.#../..#./...#"),
    ('=', "//####//####"),
    ('>', "#.../.#../..#./...#/..#./.#../#..."),
    ('?', "####/...#/...#/.###/.#..//.#.."),
    ('@', "#####/#...#/#.###/#.#.#/#.###/#..../#####"),
    ('[', "##/#./#./#./#./#./##"),
    ('\\', "#..../.#.../.#.../..#../...#./...#./....#"),
    (']', "##/.#/.#/.#/.#/.#/##"),
    ('^', "..#../.#.#./#...#"),
    ('_', "///////#####"),
    ('`', "#./.#"),
    ('{', "..#/.#./.#./#../.#./.#./..#"),
    ('|', "#/#/#/#/#/#/#/#/#"),
    ('}', "#../.#./.#./..#/.#./.#./#.."),
    ('~', "///.##.#/#.##."),
];

/// A glyph: its width and its mask, row by row (`true` is ink).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Glyph {
    /// The character.
    pub ch: char,
    /// Width of the mask in pixels.
    pub width: i32,
    /// `GLYPH_ROWS` × `width` cells, row by row.
    pub mask: Vec<bool>,
}

impl Glyph {
    /// The advance to the next glyph.
    pub fn advance(&self) -> i32 {
        self.width + SPACING
    }
}

fn parse(ch: char, s: &str) -> Glyph {
    let rows: Vec<&str> = s.split('/').collect();
    let width = rows.iter().map(|r| r.len()).max().unwrap_or(0) as i32;
    let mut mask = vec![false; (width * GLYPH_ROWS) as usize];
    for (y, r) in rows.iter().enumerate().take(GLYPH_ROWS as usize) {
        for (x, c) in r.bytes().enumerate() {
            if c == b'#' {
                mask[y * width as usize + x] = true;
            }
        }
    }
    Glyph { ch, width, mask }
}

/// Every glyph of the font, in table order.
pub fn glyphs() -> Vec<Glyph> {
    GLYPHS.iter().map(|&(c, s)| parse(c, s)).collect()
}

/// The glyph of `c`, or the glyph of `?` for a character the font lacks;
/// `None` for a space.
pub fn glyph(c: char) -> Option<Glyph> {
    if c == ' ' {
        return None;
    }
    let s = GLYPHS
        .iter()
        .find(|(g, _)| *g == c)
        .or_else(|| GLYPHS.iter().find(|(g, _)| *g == '?'))?;
    Some(parse(s.0, s.1))
}

/// The advance of one character.
pub fn advance(c: char) -> i32 {
    if c == ' ' {
        return SPACE_ADVANCE;
    }
    let s = GLYPHS
        .iter()
        .find(|(g, _)| *g == c)
        .or_else(|| GLYPHS.iter().find(|(g, _)| *g == '?'));
    s.map_or(SPACE_ADVANCE, |(_, m)| {
        m.split('/').map(str::len).max().unwrap_or(0) as i32 + SPACING
    })
}

/// The width of a line of text (no trailing spacing).
pub fn text_width(s: &str) -> i32 {
    let w: i32 = s.chars().map(advance).sum();
    if w > 0 { w - SPACING } else { 0 }
}

/// Cuts `s` to fit in `width` pixels, ending with ".." when cut.
pub fn fit(s: &str, width: i32) -> String {
    if text_width(s) <= width {
        return s.to_owned();
    }
    let mut out = String::new();
    for c in s.chars() {
        let mut t = out.clone();
        t.push(c);
        t.push_str("..");
        if text_width(&t) > width {
            break;
        }
        out.push(c);
    }
    out.push_str("..");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_printable_ascii_character_has_a_glyph() {
        let gs = glyphs();
        for c in '!'..='~' {
            assert!(gs.iter().any(|g| g.ch == c), "{c:?} has no glyph");
        }
        for g in &gs {
            assert!(g.width > 0 && g.width <= 6, "{:?} width {}", g.ch, g.width);
            assert!(g.mask.iter().any(|&b| b), "{:?} is empty", g.ch);
            assert_eq!(g.mask.len() as i32, g.width * GLYPH_ROWS);
        }
    }

    #[test]
    fn widths_and_fitting() {
        assert_eq!(text_width(""), 0);
        assert_eq!(text_width("A"), 5);
        assert_eq!(text_width("AB"), 11);
        assert_eq!(text_width("A B"), 5 + 1 + 3 + 5);
        assert_eq!(text_width("il"), 1 + 1 + 2);
        let f = fit("ABCDEFGHIJ", 30);
        assert!(f.ends_with(".."));
        assert!(text_width(&f) <= 30);
        assert_eq!(fit("AB", 30), "AB");
        // An unknown character draws as '?'.
        assert_eq!(glyph('é').map(|g| g.ch), Some('?'));
        assert_eq!(advance('é'), advance('?'));
    }

    /// No two characters look the same, and the reshaped capitals kept
    /// their widths (design: readable text and display scale, lead's
    /// request 2026-11-10).
    #[test]
    fn every_glyph_is_distinct_and_the_round_capitals_kept_their_width() {
        let gs = glyphs();
        for (i, a) in gs.iter().enumerate() {
            for b in &gs[i + 1..] {
                assert!(
                    a.width != b.width || a.mask != b.mask,
                    "{:?} and {:?} look the same",
                    a.ch,
                    b.ch
                );
            }
        }
        for c in ['B', 'C', 'G', 'O', 'Q', 'S'] {
            assert_eq!(glyph(c).unwrap().width, 5, "{c:?}");
        }
        // The letters that looked like digits now differ from them in at
        // least two cells.
        for (l, d) in [('S', '5'), ('O', '0'), ('B', '8'), ('G', '6'), ('O', 'D')] {
            let (l, d) = (glyph(l).unwrap(), glyph(d).unwrap());
            let differ = l.mask.iter().zip(&d.mask).filter(|(a, b)| a != b).count();
            assert!(
                differ >= 2,
                "{:?} and {:?} differ in {differ} cells",
                l.ch,
                d.ch
            );
        }
    }

    #[test]
    fn descenders_use_the_rows_below_the_baseline() {
        let g = glyph('g').unwrap();
        let row = |y: i32| (0..g.width).any(|x| g.mask[(y * g.width + x) as usize]);
        assert!(row(7) && row(8));
        let a = glyph('A').unwrap();
        let row_a = |y: i32| (0..a.width).any(|x| a.mask[(y * a.width + x) as usize]);
        assert!(!row_a(7) && !row_a(8));
    }
}
