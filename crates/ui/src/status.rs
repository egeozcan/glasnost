//! Splitting status text into the lines of the battlefield's shared status
//! display (S-0213 claim 25, Line splitting and font).
//!
//! The splitter works on the text's 16-bit units and asks a glyph-width
//! provider for each character's width: the client passes the imported game
//! bitmap font's widths (S-0014; a missing unit takes the generated
//! fallback's width). Lines stay 16-bit units, so a unit is never replaced
//! or merged with its neighbour before it selects its glyph.

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

// spec: S-0213 claim 25 (the zero text unit ends the text rather than selecting a glyph)
const END: u16 = 0;
const TAB: u16 = 9;
const LF: u16 = 10;
const CR: u16 = 13;
// spec: S-0213 claim 25 (a word boundary is a character whose value is at most 32: an ordinary space or a tab, not punctuation)
const SEPARATOR_MAX: u16 = 32;

/// The lines a status text makes.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Split {
    /// The stored lines as 16-bit text units, in order. A line ended by an
    /// explicit break keeps that character.
    pub lines: Vec<Vec<u16>>,
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
        // spec: S-0213 claim 25 (an explicit carriage return or line feed has no horizontal advance; Q-0501 question 5)
        CR | LF => x,
        // spec: S-0213 claim 25 (the glyph's width and one extra pixel of advance, integer pixels)
        _ => x + glyph_width(c) + EXTRA_ADVANCE,
    }
}

/// Splits the 16-bit text `text` into status lines within `budget` pixels
/// (S-0213 claim 25): the text ends at its first zero unit; explicit
/// breaks, the 111-character cap, word-boundary breaks at a width
/// overflow, and continuations without their leading control characters.
pub fn split(text: &[u16], budget: i32, glyph_width: impl Fn(u16) -> i32) -> Split {
    // spec: S-0213 claim 25 (the zero text unit ends the text)
    let units = &text[..text.iter().position(|&u| u == END).unwrap_or(text.len())];
    let mut out = Split::default();
    // spec: S-0213 claim 25 (an empty imported message adds one blank stored line)
    if units.is_empty() {
        out.lines.push(Vec::new());
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
        let end = line_end(units, i, budget, &glyph_width);
        if end == i {
            // spec: S-0213 Edge cases (Unfittable status text: our engine omits the remainder and reports a warning)
            out.unfittable = true;
            break;
        }
        out.lines.push(units[i..end].to_vec());
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

/// The drawn characters of a stored line with their pen offsets from the
/// line origin (S-0213 claim 25): tabs move the pen to the next stop,
/// carriage returns and line feeds draw nothing and do not move it, every
/// other unit draws its glyph (or the fallback) and advances. The text ends
/// at a zero unit.
pub fn glyphs(line: &[u16], glyph_width: impl Fn(u16) -> i32) -> Vec<(i32, u16)> {
    let mut out = Vec::new();
    let mut x = 0;
    for &c in line {
        match c {
            // spec: S-0213 claim 25 (the zero text unit ends the text)
            END => break,
            // spec: S-0213 claim 25 (tabs and explicit breaks use their special rules rather than mapped glyph metrics; breaks draw no glyph)
            TAB | CR | LF => {}
            _ => out.push((x, c)),
        }
        x = advance(x, c, &glyph_width);
    }
    out
}

/// The pen position after the whole of `line` (S-0213 claim 25).
pub fn width(line: &[u16], glyph_width: impl Fn(u16) -> i32) -> i32 {
    line.iter()
        .take_while(|&&c| c != END)
        .fold(0, |x, &c| advance(x, c, &glyph_width))
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

    fn u(text: &str) -> Vec<u16> {
        text.encode_utf16().collect()
    }

    fn lines(text: &str, budget: i32) -> Vec<String> {
        let s = split(&u(text), budget, w);
        assert!(!s.unfittable, "{text:?}");
        s.lines
            .iter()
            .map(|l| String::from_utf16(l).unwrap())
            .collect()
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
        let s = split(&u(&text), 10_000, narrow);
        assert_eq!(s.lines.len(), 2);
        assert_eq!(s.lines[0].len(), 111);
        assert_eq!(s.lines[1], u(" "));
        // A character outside the basic plane takes two units; the cut
        // keeps each unit as it is (no replacement character).
        let wide = u(&format!("{}\u{1F600}", "a".repeat(110)));
        let s = split(&wide, 10_000, narrow);
        assert_eq!(s.lines[0].len(), 111);
        assert_eq!(s.lines[0][110], wide[110]);
        assert_eq!(s.lines[1], [wide[111]]);
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
        let s = split(&u("aW"), 6, w);
        assert_eq!(s.lines, [u("a")]);
        assert!(s.unfittable);
        let s = split(&u("W"), 6, w);
        assert!(s.lines.is_empty());
        assert!(s.unfittable);
    }

    // spec: S-0213 claim 25 (changed glyph widths change the breaks accordingly)
    #[test]
    fn claim_25_widths_come_from_the_provider() {
        assert_eq!(lines("abcd", 20), ["abcd"]);
        let s = split(&u("abcd"), 20, |_| 5);
        assert_eq!(s.lines, [u("abc"), u("d")]);
    }

    // spec: S-0213 claim 25 (the zero text unit ends the text)
    #[test]
    fn claim_25_zero_unit_ends_the_text() {
        assert_eq!(lines("ab\0cd", 100), ["ab"]);
        assert_eq!(lines("\0cd", 100), [""]);
        assert_eq!(glyphs(&u("a\0b"), w), [(0, 97)]);
        assert_eq!(width(&u("a\0b"), w), 5);
    }

    // spec: S-0213 claim 25 (tabs go to the next stop and breaks draw nothing without advance; other units, control characters included, draw their glyph)
    #[test]
    fn claim_25_drawn_glyphs() {
        assert_eq!(
            glyphs(&u("ab\tc\u{1}d\r"), w),
            [(0, 97), (5, 98), (64, 99), (69, 1), (74, 100)]
        );
        assert_eq!(width(&u("ab\tc\r"), w), 69);
        assert_eq!(glyphs(&u("a\r\nb"), w), [(0, 97), (5, 98)]);
    }
}
