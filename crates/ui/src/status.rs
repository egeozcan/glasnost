//! Splitting status text into the lines of the battlefield's shared status
//! display (S-0213 claim 25, Line splitting and font).
//!
//! The splitter works on the text's 16-bit units and asks a glyph-width
//! provider for each character's width. The original measures with the
//! imported game bitmap font; our engine has no reader for that file yet
//! (Q-0501), so the client passes [`our_glyph_width`], our own font's
//! widths, in the meantime.

use crate::font;

/// The most text characters (16-bit units) a status line holds.
// spec: S-0213 claim 25 (a line contains at most 111 text characters, counted in the string table's 16-bit text units)
pub const MAX_LINE_UNITS: usize = 111;
/// The tab-stop spacing from the line origin, in pixels.
// spec: S-0213 claim 25 (a tab advances to the next multiple of 64 pixels from the line origin)
pub const TAB_STOP: i32 = 64;
/// The extra advance after every character's glyph width, in pixels.
// spec: S-0213 claim 25 (one extra pixel of advance per character, including the last)
pub const EXTRA_ADVANCE: i32 = 1;
/// The width budget's margin at game start: the view width minus this.
// spec: S-0213 claim 25 (the battlefield view's width minus 22 pixels after game start)
pub const START_MARGIN: i32 = 22;
/// The width budget's margin after an in-game change of the view's
/// dimensions.
// spec: S-0213 claim 25 (after an in-game change of the battlefield view's dimensions, the new width minus 16 pixels)
pub const RESIZED_MARGIN: i32 = 16;

const TAB: u16 = 9;
const LF: u16 = 10;
const CR: u16 = 13;
// spec: S-0213 claim 25 (a word boundary is a character whose value is at most 32: an ordinary space or a tab, not punctuation)
const SEPARATOR_MAX: u16 = 32;

/// The lines a status text makes.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Split {
    /// The stored lines, in order. A line ended by an explicit break keeps
    /// that character.
    pub lines: Vec<String>,
    /// Whether a remainder was left out because its first character could
    /// not fit the budget (S-0213 Edge cases, *Unfittable status text*).
    pub unfittable: bool,
}

/// The pen position after character `c` drawn at `x`: a tab goes to the
/// next tab stop, a carriage return or line feed does not move it, any
/// other character advances its glyph width plus one pixel.
pub fn advance(x: i32, c: u16, glyph_width: &impl Fn(u16) -> i32) -> i32 {
    match c {
        // spec: S-0213 claim 25 (a tab advances to the next multiple of 64 pixels from the line origin)
        TAB => (x.div_euclid(TAB_STOP) + 1) * TAB_STOP,
        // Interim (Q-0501, question 5): an explicit break has no advance.
        CR | LF => x,
        // spec: S-0213 claim 25 (the glyph's width and one extra pixel of advance, integer pixels)
        _ => x + glyph_width(c) + EXTRA_ADVANCE,
    }
}

/// Splits `text` into status lines within `budget` pixels (S-0213 claim
/// 25): explicit breaks, the 111-character cap, word-boundary breaks at a
/// width overflow, and continuations without their leading control
/// characters.
pub fn split(text: &str, budget: i32, glyph_width: impl Fn(u16) -> i32) -> Split {
    let units: Vec<u16> = text.encode_utf16().collect();
    let mut out = Split::default();
    // spec: S-0213 claim 25 (an empty imported message adds one blank stored line)
    if units.is_empty() {
        out.lines.push(String::new());
        return out;
    }
    let mut i = 0;
    let mut first = true;
    loop {
        if !first {
            // spec: S-0213 claim 25 (at the start of a continuation, characters 1 to 31 are skipped; ordinary spaces are kept)
            while i < units.len() && (1..SEPARATOR_MAX).contains(&units[i]) {
                i += 1;
            }
        }
        if i >= units.len() {
            break;
        }
        let end = line_end(&units, i, budget, &glyph_width);
        if end == i {
            // spec: S-0213 Edge cases (Unfittable status text: our engine omits the remainder and reports a warning)
            out.unfittable = true;
            break;
        }
        out.lines.push(String::from_utf16_lossy(&units[i..end]));
        i = end;
        first = false;
    }
    out
}

/// The end of the line that starts at `start`.
fn line_end(units: &[u16], start: usize, budget: i32, glyph_width: &impl Fn(u16) -> i32) -> usize {
    let mut x = 0;
    let mut j = start;
    // spec: S-0213 claim 25 (reaching 111 characters makes a break there without searching back for a word boundary)
    while j < units.len() && j - start < MAX_LINE_UNITS {
        let c = units[j];
        // spec: S-0213 claim 25 (an explicit carriage return or line feed ends a line after that character)
        if c == CR || c == LF {
            return j + 1;
        }
        let next = advance(x, c, glyph_width);
        // spec: S-0213 claim 25 (the width limit is inclusive: reaching it exactly fits)
        if next <= budget {
            x = next;
            j += 1;
            continue;
        }
        // spec: S-0213 claim 25 (an overflowing separator ends the line just before it)
        if c <= SEPARATOR_MAX {
            return j;
        }
        // spec: S-0213 claim 25 (inside a word, break after the last preceding character of value at most 32 in the fitting prefix; without one, the word's fitting prefix is used)
        return units[start..j]
            .iter()
            .rposition(|&u| u <= SEPARATOR_MAX)
            .map_or(j, |k| start + k + 1);
    }
    j
}

/// A glyph's width in our own font: the mask width, a space 2 pixels, `?`
/// for a character the font lacks. Interim provider until the game font can
/// be read (Q-0501).
pub fn our_glyph_width(c: u16) -> i32 {
    let ch = char::from_u32(u32::from(c)).unwrap_or('?');
    font::advance(ch) - font::SPACING
}

/// The drawn pieces of a stored line: runs of printable text with their
/// pen offsets from the line origin. Tabs move the pen to the next stop;
/// other control characters (an ending break) draw nothing.
pub fn runs(line: &str, glyph_width: impl Fn(u16) -> i32) -> Vec<(i32, String)> {
    let mut out: Vec<(i32, String)> = Vec::new();
    let mut x = 0;
    let mut run: Option<(i32, Vec<u16>)> = None;
    for c in line.encode_utf16() {
        if c < SEPARATOR_MAX {
            if let Some((rx, r)) = run.take() {
                out.push((rx, String::from_utf16_lossy(&r)));
            }
        } else {
            run.get_or_insert_with(|| (x, Vec::new())).1.push(c);
        }
        x = advance(x, c, &glyph_width);
    }
    if let Some((rx, r)) = run {
        out.push((rx, String::from_utf16_lossy(&r)));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Synthetic widths: every glyph 4 pixels wide (advance 5), a space 2
    /// (advance 3), `W` 9 (advance 10).
    fn w(c: u16) -> i32 {
        match c {
            32 => 2,
            87 => 9,
            _ => 4,
        }
    }

    fn lines(text: &str, budget: i32) -> Vec<String> {
        let s = split(text, budget, w);
        assert!(!s.unfittable, "{text:?}");
        s.lines
    }

    // spec: S-0213 claim 25 (glyph width plus one pixel per character, including the last; the width limit is inclusive)
    #[test]
    fn claim_25_advance_and_inclusive_limit() {
        // "abcd" is 4 × 5 = 20 pixels, the last character's extra pixel included.
        assert_eq!(lines("abcd", 20), ["abcd"]);
        assert_eq!(lines("abcde", 20), ["abcd", "e"]);
        assert_eq!(lines("abcd", 19), ["abc", "d"]);
        assert_eq!(advance(0, 87, &w), 10);
    }

    // spec: S-0213 claim 25 (inside a word, break after the last preceding character of value at most 32; punctuation is no boundary; a word alone splits inside itself)
    #[test]
    fn claim_25_word_boundary() {
        // "ab cd" = 5 + 5 + 3 + 5 + 5 = 23; at 20 the "d" overflows.
        assert_eq!(lines("ab cd", 20), ["ab ", "cd"]);
        // A full stop is not a boundary: the word splits at the fitting prefix.
        assert_eq!(lines("ab.cd", 20), ["ab.c", "d"]);
        assert_eq!(lines("abcdefgh", 20), ["abcd", "efgh"]);
        // A tab is a boundary too.
        assert_eq!(lines("a\tbc", 66), ["a\t", "bc"]);
    }

    // spec: S-0213 claim 25 (an overflowing separator ends the line just before it; continuation spaces are kept)
    #[test]
    fn claim_25_overflowing_separator() {
        // "abcd" fills 20 exactly; the space overflows and starts the next line.
        assert_eq!(lines("abcd  ef", 20), ["abcd", "  ef"]);
    }

    // spec: S-0213 claim 25 (a tab advances to the next multiple of 64 from the line origin)
    #[test]
    fn claim_25_tab_stops() {
        assert_eq!(advance(0, 9, &w), 64);
        assert_eq!(advance(63, 9, &w), 64);
        assert_eq!(advance(64, 9, &w), 128);
        // "a" (5) + tab (to 64) + "b" (69) fits 69 exactly, not 68.
        assert_eq!(lines("a\tb", 69), ["a\tb"]);
        assert_eq!(lines("a\tb", 68), ["a\t", "b"]);
    }

    // spec: S-0213 claim 25 (at most 111 characters, breaking there without searching back; counts are 16-bit units)
    #[test]
    fn claim_25_character_cap() {
        let narrow = |_: u16| 0;
        let text = "a ".repeat(56); // 112 units
        let s = split(&text, 10_000, narrow);
        assert_eq!(s.lines.len(), 2);
        assert_eq!(s.lines[0].encode_utf16().count(), 111);
        assert_eq!(s.lines[1], " ");
        // A character outside the basic plane takes two units.
        let wide = format!("{}\u{1F600}", "a".repeat(110));
        let s = split(&wide, 10_000, narrow);
        assert_eq!(s.lines[0].encode_utf16().count(), 111);
    }

    // spec: S-0213 claim 25 (an explicit CR or LF ends a line after it; continuation characters 1 to 31 are skipped, so CR LF is one break and repeated breaks add no empty lines; spaces are kept)
    #[test]
    fn claim_25_explicit_breaks() {
        assert_eq!(lines("ab\r\ncd", 100), ["ab\r", "cd"]);
        assert_eq!(lines("ab\n\n\ncd", 100), ["ab\n", "cd"]);
        assert_eq!(lines("ab\n  cd", 100), ["ab\n", "  cd"]);
        assert_eq!(lines("ab\n", 100), ["ab\n"]);
        assert_eq!(lines("\nab", 100), ["\n", "ab"]);
    }

    // spec: S-0213 claim 25 (an empty message adds one blank stored line)
    #[test]
    fn claim_25_empty_message() {
        assert_eq!(lines("", 100), [""]);
    }

    // spec: S-0213 Edge cases (Unfittable status text: the remainder is omitted and reported)
    #[test]
    fn unfittable_remainder_is_omitted() {
        let s = split("aW", 6, w);
        assert_eq!(s.lines, ["a"]);
        assert!(s.unfittable);
        let s = split("W", 6, w);
        assert!(s.lines.is_empty());
        assert!(s.unfittable);
    }

    // spec: S-0213 claim 25 (changed glyph widths change the breaks accordingly)
    #[test]
    fn claim_25_widths_come_from_the_provider() {
        assert_eq!(lines("abcd", 20), ["abcd"]);
        let s = split("abcd", 20, |_| 5);
        assert_eq!(s.lines, ["abc", "d"]);
    }

    #[test]
    fn runs_follow_tabs_and_skip_breaks() {
        assert_eq!(
            runs("ab\tc\r", w),
            [(0, "ab".to_owned()), (64, "c".to_owned())]
        );
    }
}
