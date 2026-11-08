//! The sidebar's tips (S-0411 claims 20 to 23): the places of the sidebar
//! that show one, their texts (with the values written into an entry's
//! marks) and their boxes' places. Their timing, layout
//! and look are the name tip's ([`super::name_tip`]); the session gives
//! the display names, prices and totals. Nothing here reaches the
//! simulation.

use glasnost_ui::geom::{Point, Rect};
use glasnost_ui::hud::Layout;

/// How far a build icon's tip overlaps the icon's corner, pixels each way.
pub const OVERLAP: i32 = 5; // spec: S-0411 Constants (icon-tip overlap 5) and claim 21 (l₀ = x_s + w_s − 5 or x_s − w + 5; m₀ = y_s + h_s − 5 or y_s − h + 5)
/// What a sidebar tip's wrap width takes from the tip area's width, pixels.
pub const WRAP_MARGIN: i32 = 8; // spec: S-0411 Constants (sidebar-tip wrap margin 8) and claim 21 (wrap width A_w − 8)

/// The money-format entry: a unit's or building's icon tip.
pub const MONEY_FORMAT: &str = "TXT_MONEY_FORMAT_2"; // spec: S-0411 Import mapping (TXT_MONEY_FORMAT_2: the money-format entry)
/// The power entry: the power meter's tip.
pub const POWER_DRAIN: &str = "TXT_POWER_DRAIN"; // spec: S-0411 Import mapping (TXT_POWER_DRAIN: the power entry)
/// The repair-mode label.
pub const REPAIR_MODE: &str = "TXT_REPAIR_MODE"; // spec: S-0411 Import mapping (TXT_REPAIR_MODE: the repair-mode label)
/// The sell-mode label.
pub const SELL_MODE: &str = "TXT_SELL_MODE"; // spec: S-0411 Import mapping (TXT_SELL_MODE: the sell-mode label)
/// The four tab labels, in claim 1's tab order.
pub const TAB_LABELS: [&str; 4] = ["Tip:Tab1", "Tip:Tab2", "Tip:Tab3", "Tip:Tab4"]; // spec: S-0411 Import mapping (Tip:Tab1 to Tip:Tab4: the first to fourth tab labels, in claim 1's tab order)
/// The scroll-up and scroll-down labels.
pub const SCROLL_LABELS: [&str; 2] = ["Tip:ScrollUp", "Tip:ScrollDown"]; // spec: S-0411 Import mapping (Tip:ScrollUp, Tip:ScrollDown)
/// The disabled label.
pub const DISABLED: &str = "Tip:Disabled"; // spec: S-0411 Import mapping (Tip:Disabled: the disabled label)
/// The options label.
pub const OPTIONS: &str = "Tip:OptionsButton"; // spec: S-0411 Import mapping (Tip:OptionsButton: the options label)

/// The stand-in of the money-format entry while no string table is read.
// design: S-0015 claim 1 (only a game built without an install has no table): our stand-in, a text point and a number point
pub const MONEY_STAND_IN: &str = "%s %d";
/// The stand-in of the power entry while no string table is read.
// design: S-0015 claim 1 (only a game built without an install has no table): our stand-in, two number points
pub const POWER_STAND_IN: &str = "%d %d";
/// Our power-mode button's tip.
// design: S-0411 claim 22 (our engine's power-mode button has a tip only if its design gives one): ours gives its name, as our earlier hover name did
pub const POWER_MODE_TIP: &str = "Power on/off";

const PERCENT: u16 = b'%' as u16;
const SPACE: u16 = b' ' as u16;
const LF: u16 = 10;

/// A sidebar control of claim 22's table, and our power-mode button.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Control {
    /// The n-th tab button, in claim 1's order.
    Tab(usize),
    /// The scroll-up (0) or scroll-down (1) button.
    Scroll(usize),
    /// The repair button.
    Repair,
    /// The sell button.
    Sell,
    /// The power meter.
    PowerMeter,
    /// The options button.
    Options,
    /// Our power-mode button.
    PowerMode,
}

/// What lies under the pointer on the sidebar, for its tips.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Place {
    /// The n-th shown slot of the strip (S-0411 Terms), filled or empty.
    Slot(usize),
    /// A control with a tip.
    Control(Control),
    /// Any other part of the sidebar: no tip (claim 22).
    Nothing,
}

/// The place of logical pixel `p` on the sidebar of layout `l`; `None` off
/// the sidebar.
pub fn place_at(l: &Layout, p: Point) -> Option<Place> {
    if !l.bar.contains(p) {
        return None;
    }
    if let Some(k) = l.slot_at(p) {
        return Some(Place::Slot(k));
    }
    let [sell, repair, power_mode] = l.modes;
    let controls = l
        .tabs
        .iter()
        .enumerate()
        .map(|(i, r)| (*r, Control::Tab(i)))
        .chain(
            l.scroll
                .iter()
                .enumerate()
                .map(|(k, r)| (*r, Control::Scroll(k))),
        )
        .chain([
            (sell, Control::Sell),
            (repair, Control::Repair),
            (power_mode, Control::PowerMode),
            (l.power, Control::PowerMeter),
            (l.options, Control::Options),
        ]);
    for (r, c) in controls {
        if r.contains(p) {
            return Some(Place::Control(c));
        }
    }
    // spec: S-0411 claim 22 (no other part of the sidebar shows a tip: the radar panel, the credits counter, the shell art, a gap between controls)
    Some(Place::Nothing)
}

/// The most text units [`fill`] writes. A tip shows only the first
/// [`super::name_tip::TEXT_LIMIT`] units of its text (`name_tip::cut`),
/// and neither [`spaces_to_lines`] nor anything else before the cut
/// changes a text's length, so the units after them are not written: a
/// large width or precision then costs nothing.
// spec: S-0411 Edge cases (Icon-tip text of 66 text units or more, and a power-meter tip text too long for the original, an entry with a long text or a large width: shown as S-0111's name tip shows a long text) and S-0111 Edge cases (our engine shows the first 256 text units)
const FILL_LIMIT: usize = super::name_tip::TEXT_LIMIT;

/// A mark's width or precision.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Count {
    /// A run of decimal digits (saturating).
    Number(usize),
    /// An asterisk.
    Star,
}

/// One mark of an entry's text (S-0411 claim 23, *Marks*): its flags,
/// width, precision and size marks, read up to the character that ends it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Mark {
    minus: bool,
    plus: bool,
    space: bool,
    hash: bool,
    zero: bool,
    width: Option<Count>,
    precision: Option<Count>,
    /// The size mark h is among its size marks.
    h: bool,
    /// The size mark I64 is among its size marks.
    i64: bool,
}

impl Mark {
    fn star(&self) -> bool {
        self.width == Some(Count::Star) || self.precision == Some(Count::Star)
    }

    fn width(&self) -> usize {
        match self.width {
            Some(Count::Number(n)) => n,
            _ => 0,
        }
    }

    fn precision(&self) -> Option<usize> {
        match self.precision {
            Some(Count::Number(n)) => Some(n),
            _ => None,
        }
    }
}

/// An entry's filled text, written no further than [`FILL_LIMIT`] units.
struct Out(Vec<u16>);

impl Out {
    fn push(&mut self, c: u16) {
        if self.0.len() < FILL_LIMIT {
            self.0.push(c);
        }
    }

    fn push_n(&mut self, c: u16, n: usize) {
        let n = n.min(FILL_LIMIT.saturating_sub(self.0.len()));
        self.0.extend(std::iter::repeat_n(c, n));
    }

    fn extend(&mut self, s: &[u16]) {
        let n = s.len().min(FILL_LIMIT.saturating_sub(self.0.len()));
        self.0.extend_from_slice(&s[..n]);
    }

    /// `body` (`len` units, written by `write`) padded to `width` units:
    /// spaces after it with the minus flag, else `fill` before it.
    fn padded(&mut self, m: &Mark, len: usize, fill: u16, write: impl FnOnce(&mut Self)) {
        let pad = m.width().saturating_sub(len);
        if m.minus {
            write(self);
            self.push_n(SPACE, pad);
        } else {
            self.push_n(fill, pad);
            write(self);
        }
    }
}

const PLUS: u16 = b'+' as u16;
const MINUS: u16 = b'-' as u16;
const HASH: u16 = b'#' as u16;
const ZERO: u16 = b'0' as u16;
const STAR: u16 = b'*' as u16;
const DOT: u16 = b'.' as u16;

fn digit(c: u16) -> Option<usize> {
    (u16::from(b'0')..=u16::from(b'9'))
        .contains(&c)
        .then(|| usize::from(c - u16::from(b'0')))
}

/// A run of decimal digits from `j`, saturating; the index after it.
fn digits(entry: &[u16], mut j: usize) -> (usize, usize) {
    let mut n = 0usize;
    while let Some(d) = entry.get(j).copied().and_then(digit) {
        n = n.saturating_mul(10).saturating_add(d);
        j += 1;
    }
    (n, j)
}

/// Reads a mark whose percent sign stands at `i`: the mark and the index
/// of the character that ends it (`entry.len()` at the end of the text).
fn read_mark(entry: &[u16], i: usize) -> (Mark, usize) {
    let mut m = Mark::default();
    let mut j = i + 1;
    let at = |j: usize| entry.get(j).copied();
    // spec: S-0411 claim 23 (Marks: flags, any of the space, plus, minus, hash and zero characters, in any order and any number of times)
    while let Some(c) = at(j) {
        match c {
            SPACE => m.space = true,
            PLUS => m.plus = true,
            MINUS => m.minus = true,
            HASH => m.hash = true,
            ZERO => m.zero = true,
            _ => break,
        }
        j += 1;
    }
    // spec: S-0411 claim 23 (Marks: a width, a run of decimal digits whose first is not 0, or an asterisk)
    match at(j) {
        Some(STAR) => {
            m.width = Some(Count::Star);
            j += 1;
        }
        Some(c) if digit(c).is_some_and(|d| d != 0) => {
            let (n, k) = digits(entry, j);
            m.width = Some(Count::Number(n));
            j = k;
        }
        _ => {}
    }
    // spec: S-0411 claim 23 (Marks: a precision, a full stop followed by a run of decimal digits, or by an asterisk, or by neither, which counts as 0)
    if at(j) == Some(DOT) {
        j += 1;
        if at(j) == Some(STAR) {
            m.precision = Some(Count::Star);
            j += 1;
        } else {
            let (n, k) = digits(entry, j);
            m.precision = Some(Count::Number(n));
            j = k;
        }
    }
    // spec: S-0411 claim 23 (Marks: size marks, any number of the letters h, l, w, F, L and N and of the letter I followed by the digits 6 and 4)
    loop {
        match at(j).and_then(|c| u8::try_from(c).ok()) {
            Some(b'h') => {
                m.h = true;
                j += 1;
            }
            Some(b'l' | b'w' | b'F' | b'L' | b'N') => j += 1,
            Some(b'I') if at(j + 1) == Some(u16::from(b'6')) && at(j + 2) == Some(u16::from(b'4')) => {
                m.i64 = true;
                j += 3;
            }
            _ => break,
        }
    }
    (m, j)
}

/// What the character that ends a mark makes of it (S-0411 claim 23).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum End {
    /// A text point.
    Text,
    /// A number point of this letter.
    Number(u8),
    /// One percent sign, no value.
    Percent,
    /// The whole mark as it stands, no value.
    AsItStands,
    /// The mark dropped, the ending character shown in its place.
    Dropped,
}

fn end_of(m: &Mark, c: u16) -> End {
    let Ok(b) = u8::try_from(c) else {
        // spec: S-0411 claim 23 (Marks: any other character, here one outside basic Latin, drops the mark and is shown in its place)
        return End::Dropped;
    };
    match b {
        // spec: S-0411 claim 23 (Marks: s after size marks that include I64 or h is shown as it stands, our engine's rule; Edge cases for the original)
        b's' if m.i64 || m.h => End::AsItStands,
        // spec: S-0411 claim 23 (Marks: the letter s makes the mark a text point)
        b's' => End::Text,
        // spec: S-0411 claim 23 (Marks: a number point's letter after size marks that include I64 is shown as it stands, our engine's rule)
        b'd' | b'i' | b'u' | b'o' | b'x' | b'X' | b'p' if m.i64 => End::AsItStands,
        // spec: S-0411 claim 23 (Marks: one of the letters d, i, u, o, x, X and p makes it a number point)
        b'd' | b'i' | b'u' | b'o' | b'x' | b'X' | b'p' => End::Number(b),
        // spec: S-0411 claim 23 (Marks: a percent sign shows one percent sign, and the mark takes no value)
        b'%' => End::Percent,
        // spec: S-0411 claim 23 (Marks: the letters c, C, B, e, E, f, g, G, n, S and Z: our engine shows the whole mark as it stands, no value) and Edge cases (Marks with a defined result that our engine shows as they stand; A tip entry's point given a value of another kind: departures, no switch)
        b'c' | b'C' | b'B' | b'e' | b'E' | b'f' | b'g' | b'G' | b'n' | b'S' | b'Z' => {
            End::AsItStands
        }
        // spec: S-0411 claim 23 (Marks: any other character, a letter the list does not name, the letter I not followed by 6 and 4, a digit, flag or full stop out of order, or another sign, drops the mark and is shown in its place)
        _ => End::Dropped,
    }
}

/// Writes text point `m` with value `v` (S-0411 claim 23, *Forms* and
/// *Flags and width*).
fn text_point(out: &mut Out, m: &Mark, v: &[u16]) {
    // spec: S-0411 claim 23 (Forms: with a precision a text point writes at most that many of its text units)
    let v = &v[..m.precision().map_or(v.len(), |p| p.min(v.len()))];
    // spec: S-0411 claim 23 (Flags and width: spaces before, or after with the minus flag; the zero flag pads a text point with zeros too)
    let fill = if m.zero { ZERO } else { SPACE };
    out.padded(m, v.len(), fill, |o| o.extend(v));
}

/// Writes number point `m` of letter `letter` with value `v` (S-0411
/// claim 23, *Forms* and *Flags and width*).
fn number_point(out: &mut Out, m: &Mark, letter: u8, v: i32) {
    let signed = matches!(letter, b'd' | b'i');
    // spec: S-0411 claim 23 (Forms: with h a number point uses only the value's lowest 16 bits, signed from −32768 to 32767 for d and i, a whole number from 0 to 65535 for the others; u, o, x and X take a negative value v as v + 2³²; l, w, F, L and N change nothing)
    let (negative, magnitude): (bool, u64) = match (signed, m.h) {
        (true, true) => {
            let n = v as i16;
            (n < 0, u64::from(n.unsigned_abs()))
        }
        (true, false) => (v < 0, u64::from(v.unsigned_abs())),
        (false, true) => (false, u64::from(v as u16)),
        (false, false) => (false, u64::from(v as u32)),
    };
    // spec: S-0411 claim 23 (Forms: d, i and u in decimal; o in octal; x in hexadecimal with lowercase letters, X with capitals; p as X)
    let text = match letter {
        b'o' => format!("{magnitude:o}"),
        b'x' => format!("{magnitude:x}"),
        b'X' | b'p' => format!("{magnitude:X}"),
        _ => magnitude.to_string(),
    };
    // spec: S-0411 claim 23 (Forms: p always with a precision of 8)
    let precision = if letter == b'p' { Some(8) } else { m.precision() };
    // spec: S-0411 claim 23 (Forms: the precision is the least number of digits, zeros added before them; without one, 1; with a precision of 0 the value 0 writes no digits)
    let shown = if precision == Some(0) && magnitude == 0 {
        0
    } else {
        text.len()
    };
    let least = precision.unwrap_or(1);
    let mut lead = least.saturating_sub(shown);
    // spec: S-0411 claim 23 (Flags and width: for o the hash flag adds a leading 0 when the digits do not start with one)
    if letter == b'o' && m.hash && lead == 0 && (shown == 0 || !text.starts_with('0')) {
        lead = 1;
    }
    // spec: S-0411 claim 23 (Flags and width: for d and i the plus flag writes a plus sign before a value that is not negative, the space flag without the plus flag a space; a minus sign when negative)
    let sign = match (signed, negative) {
        (true, true) => Some(MINUS),
        (true, false) if m.plus => Some(PLUS),
        (true, false) if m.space => Some(SPACE),
        _ => None,
    };
    // spec: S-0411 claim 23 (Flags and width: the hash flag writes 0x before the digits of x, 0X before those of X and p, when the value is not 0)
    let prefix: &[u8] = match letter {
        b'x' if m.hash && magnitude != 0 => b"0x",
        b'X' | b'p' if m.hash && magnitude != 0 => b"0X",
        _ => b"",
    };
    let head = usize::from(sign.is_some()) + prefix.len();
    let len = head.saturating_add(lead).saturating_add(shown);
    let body = |o: &mut Out| {
        if let Some(s) = sign {
            o.push(s);
        }
        for &c in prefix {
            o.push(u16::from(c));
        }
        o.push_n(ZERO, lead);
        for c in text.bytes().take(shown) {
            o.push(u16::from(c));
        }
    };
    // spec: S-0411 claim 23 (Flags and width: the width counts sign and prefix; with the zero flag and without the minus flag zeros between any sign or prefix and the digits, except that a number point with a precision gets spaces; p has a precision, Forms)
    if m.zero && !m.minus && precision.is_none() {
        let pad = m.width().saturating_sub(len);
        if let Some(s) = sign {
            out.push(s);
        }
        for &c in prefix {
            out.push(u16::from(c));
        }
        out.push_n(ZERO, pad.saturating_add(lead));
        for c in text.bytes().take(shown) {
            out.push(u16::from(c));
        }
    } else {
        out.padded(m, len, SPACE, body);
    }
}

/// An entry's text with its values written in (S-0411 claim 23): each
/// text point takes the next of `texts` and each number point the next of
/// `numbers`, in the order the points stand; a point with no value of its
/// kind left writes nothing; values no point takes are not written; the
/// rest of the text is shown as it stands, except at its marks.
pub fn fill(entry: &[u16], texts: &[&[u16]], numbers: &[i32]) -> Vec<u16> {
    let mut out = Out(Vec::with_capacity(entry.len().min(FILL_LIMIT)));
    let (mut ts, mut ns) = (texts.iter(), numbers.iter());
    let mut i = 0;
    while i < entry.len() {
        let c = entry[i];
        if c != PERCENT {
            // spec: S-0411 claim 23 (each entry's text is shown as it stands, except at its marks)
            out.push(c);
            i += 1;
            continue;
        }
        let (m, j) = read_mark(entry, i);
        let Some(&end) = entry.get(j) else {
            if m.star() {
                // spec: S-0411 claim 23 (a mark with an asterisk as its width or precision, also at the end of the text, is shown as it stands by our engine and takes no value)
                out.extend(&entry[i..]);
            }
            // spec: S-0411 claim 23 (Marks: the end of the text drops a mark that has not ended, so a percent sign at the very end is not shown)
            break;
        };
        let mark = &entry[i..=j];
        i = j + 1;
        if m.star() {
            // spec: S-0411 claim 23 (a mark with an asterisk as its width or its precision, whatever character ends it, is shown as it stands by our engine and takes no value) and Edge cases (Marks with a defined result that our engine shows as they stand)
            out.extend(mark);
            continue;
        }
        match end_of(&m, end) {
            End::Text => {
                // spec: S-0411 claim 23 (Which value each point takes: each text point the entry's next text value; with none left it writes nothing, its width and flags adding nothing)
                if let Some(v) = ts.next() {
                    text_point(&mut out, &m, v);
                }
            }
            End::Number(letter) => {
                // spec: S-0411 claim 23 (Which value each point takes: each number point the next number value; with none left it writes nothing, its width and flags adding nothing)
                if let Some(&v) = ns.next() {
                    number_point(&mut out, &m, letter, v);
                }
            }
            End::Percent => out.push(PERCENT),
            End::AsItStands => out.extend(mark),
            End::Dropped => out.push(end),
        }
    }
    out.0
}

/// `text` with every space a line feed.
pub fn spaces_to_lines(text: &[u16]) -> Vec<u16> {
    // spec: S-0411 claim 20 (Text: every space of the whole text, from the display name and from the entry alike, becomes a line feed)
    text.iter()
        .map(|&c| if c == SPACE { LF } else { c })
        .collect()
}

/// An entry looked up in the string table: its text (or our stand-in
/// without a table), or the placeholder of a missing entry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Looked {
    /// The entry's text, or our stand-in.
    Text(Vec<u16>),
    /// The entry is missing: its placeholder.
    Missing(Vec<u16>),
}

/// The text of a unit's or building's icon tip: the money-format entry
/// `entry` filled with the display name `name` and the price `price`, every
/// space then a line feed.
pub fn icon_text(entry: Looked, name: &[u16], price: i32) -> Vec<u16> {
    match entry {
        // spec: S-0411 claim 20 (Text: the money-format entry with its text point filled by the display name and its number point by the price P for the local player's house; then every space a line feed)
        // spec: S-0411 claim 23 (the money-format entry has a text value, the display name, and then a number value, the price; every space a line feed after the values are written, those a width adds included)
        Looked::Text(e) => spaces_to_lines(&fill(&e, &[name], &[price])),
        // spec: S-0411 Import mapping (TXT_MONEY_FORMAT_2: a missing entry shows S-0004 claim 17's placeholder alone, with no name and no price)
        Looked::Missing(p) => p,
    }
}

/// The text of the power meter's tip: the power entry `entry` filled with
/// output `o` and drain `d`, spaces kept.
pub fn power_text(entry: Looked, (o, d): (i32, i32)) -> Vec<u16> {
    match entry {
        // spec: S-0411 claim 22 (power meter: the power entry's text with its first number point filled by the house output O and its second by its drain D; the spaces kept)
        // spec: S-0411 claim 23 (the power entry has two number values, the output O and then the drain D; the power meter's tip keeps its spaces)
        Looked::Text(e) => fill(&e, &[], &[o, d]),
        // spec: S-0411 Import mapping (TXT_POWER_DRAIN: missing entry: the placeholder alone)
        Looked::Missing(p) => p,
    }
}

/// A tab's or scroll button's label `label`, followed while `disabled` by
/// a line feed and the disabled label `off` in parentheses.
pub fn with_disabled(mut label: Vec<u16>, disabled: bool, off: &[u16]) -> Vec<u16> {
    if disabled {
        // spec: S-0411 claim 22 (followed by a line feed, an opening parenthesis, the disabled label and a closing parenthesis)
        label.push(LF);
        label.push(u16::from(b'('));
        label.extend_from_slice(off);
        label.push(u16::from(b')'));
    }
    label
}

/// The top-left corner of a `w` × `h` box of a build icon's tip for the
/// slot `slot` (its icon's cell) in the tip area `area`, in screen pixels.
pub fn slot_place(slot: Rect, (w, h): (i32, i32), area: Rect) -> (i32, i32) {
    let (xs, ys, ws, hs) = (slot.x, slot.y, slot.w, slot.h);
    // spec: S-0411 claim 21 (l₀ = x_s + w_s − 5 when x_s < A_x + ⌊A_w / 2⌋, else x_s − w + 5; l₁ = max(l₀, A_x); l = A_x + A_w − w when l₁ + w > A_x + A_w, else l₁)
    let l0 = if xs < area.x + area.w.div_euclid(2) {
        xs + ws - OVERLAP
    } else {
        xs - w + OVERLAP
    };
    let l1 = l0.max(area.x);
    let l = if l1 + w > area.x + area.w {
        area.x + area.w - w
    } else {
        l1
    };
    // spec: S-0411 claim 21 (m₀ = y_s + h_s − 5 when y_s < A_y + ⌊A_h / 2⌋, else y_s − h + 5; m = max(m₀, A_y); no correction at the area's bottom edge)
    let m0 = if ys < area.y + area.h.div_euclid(2) {
        ys + hs - OVERLAP
    } else {
        ys - h + OVERLAP
    };
    (l, m0.max(area.y))
}

/// The wrap width of a sidebar tip in tip area `area`.
pub fn wrap_width(area: Rect) -> i32 {
    // spec: S-0411 claim 21 (its wrap width is A_w − 8)
    area.w - WRAP_MARGIN
}

#[cfg(test)]
mod tests {
    use super::*;

    fn u(s: &str) -> Vec<u16> {
        s.encode_utf16().collect()
    }

    /// spec: S-0411 Terms (Insertion point: %s a text point, %d a number point, filled in the
    /// order they stand; decimal, a leading minus sign, no grouping), claim 20 (Text), claim 22
    /// (power meter) and claim 23 (Which value each point takes)
    #[test]
    fn terms_insertion_points() {
        assert_eq!(
            fill(&u("<%s|%d>"), &[&u("Tank X")], &[1500]),
            u("<Tank X|1500>")
        );
        assert_eq!(fill(&u("%d/%d"), &[], &[-25, 1234567]), u("-25/1234567"));
        assert_eq!(fill(&u("a%sb%dc"), &[], &[]), u("abc"));
        assert_eq!(fill(&u("%d%%x%"), &[], &[7]), u("7%x"));
        assert_eq!(fill(&u("%d %s"), &[&u("n")], &[1]), u("1 n"));
    }

    /// spec: S-0411 claim 23 and scenario S16 part j, its eight texts with invented names and
    /// values (Backing: measurable): (1) a text point alone; (2) a text of two words; (3)
    /// "%s|%%|%q|%d|%"; (4) "%.3s|%+06d"; (5) a number point alone; (6) "%#x|%#o"; (7)
    /// "%X|%u"; (8) "%5i|%-5d|"
    #[test]
    fn claim_23_s16_part_j_texts() {
        let name = u("Probe Tank");
        let icon = |e: &str, p: i32| icon_text(Looked::Text(u(e)), &name, p);
        let power = |e: &str, od: (i32, i32)| power_text(Looked::Text(u(e)), od);
        assert_eq!(icon("%s", 1234), u("Probe\nTank"), "(1) the name, no price");
        assert_eq!(icon("Two words", 1234), u("Two\nwords"), "(2)");
        assert_eq!(icon("%s|%%|%q|%d|%", 777), u("Probe\nTank|%|q|777|"), "(3)");
        assert_eq!(icon("%.3s|%+06d", 42), u("Pro|+00042"), "(4)");
        assert_eq!(icon("%.3s|%+06d", 9999), u("Pro|+09999"), "(4)");
        assert_eq!(power("%d", (300, -20)), u("300"), "(5)");
        assert_eq!(power("%#x|%#o", (255, 8)), u("0xff|010"), "(6)");
        assert_eq!(power("%#x|%#o", (0, 0)), u("0|0"), "(6) O = 0: no 0x");
        assert_eq!(power("%#x|%#o", (-1, 7)), u("0xffffffff|07"), "(6)");
        assert_eq!(power("%X|%u", (255, -1)), u("FF|4294967295"), "(7)");
        assert_eq!(power("%5i|%-5d|", (42, -7)), u("   42|-7   |"), "(8)");
    }

    /// spec: S-0411 claim 23 (Marks: flags, width, precision and size marks in this order; the
    /// ending character decides; a percent sign shows one; the listed letters, s or a number
    /// letter after I64 and s after h show the whole mark as it stands with no value; any other
    /// character drops the mark and is shown in its place; the end of the text drops an unended
    /// mark; an asterisk as width or precision shows the mark as it stands) and Edge cases (Marks
    /// with a defined result that our engine shows as they stand)
    #[test]
    fn claim_23_marks() {
        let f = |e: &str| fill(&u(e), &[&u("nm")], &[7]);
        // Shown as they stand, taking no value: the number point after them takes 7.
        for m in [
            "%c", "%5C", "%B", "%e", "%E", "%-f", "%g", "%G", "%n", "%S", "%Z", "%I64d", "%I64s",
            "%lI64x", "%I64hu", "%hs", "%lhs", "%hls", "%*d", "%.*d", "%-*.*s", "%*q",
        ] {
            assert_eq!(f(&format!("{m}%d")), u(&format!("{m}7")), "{m}");
        }
        // An asterisk mark also at the end of the text.
        assert_eq!(f("a%*"), u("a%*"));
        assert_eq!(f("a%5.*"), u("a%5.*"));
        // s after size marks without h, as with none: a text point.
        assert_eq!(f("%ls|%wFLNs"), u("nm|"));
        // Dropped: the ending character shown in the mark's place.
        assert_eq!(f("%q%d"), u("q7"));
        assert_eq!(f("%I6d"), u("I6d"), "I not followed by 6 and 4");
        assert_eq!(f("%Ix"), u("Ix"));
        assert_eq!(f("%.3-d"), u("-d"), "a flag after the precision");
        assert_eq!(f("%5 d"), u(" d"), "a flag after the width");
        assert_eq!(f("%h5d"), u("5d"), "a digit after a size mark");
        assert_eq!(f("%.3.d"), u(".d"), "a second full stop");
        assert_eq!(f("%5*d"), u("*d"), "an asterisk after a width of digits");
        assert_eq!(f("%\u{e9}x"), u("\u{e9}x"), "a character outside basic Latin");
        // The end of the text drops an unended mark.
        assert_eq!(f("ab%"), u("ab"));
        assert_eq!(f("ab%-05.3hl"), u("ab"));
        // Flags in any order and number; a width's first digit is not 0 (a 0 is a flag).
        assert_eq!(f("%0-0-3d|"), u("7  |"));
        assert_eq!(f("%+ +d"), u("+7"));
        assert_eq!(f("%%%d%%"), u("%7%"));
    }

    /// spec: S-0411 claim 23 (Which value each point takes: each point the next value of its own
    /// kind in the order the points stand; a point with none left writes nothing, its width and
    /// flags adding nothing; a value no point takes is not written) and Edge cases (A tip entry's
    /// point given a value of another kind: departure, no switch)
    #[test]
    fn claim_23_values() {
        let name = u("Big Tank");
        let icon = |e: &str| icon_text(Looked::Text(u(e)), &name, 900);
        let power = |e: &str| power_text(Looked::Text(u(e)), (300, -20));
        assert_eq!(icon("%d"), u("900"), "the price and no name");
        assert_eq!(icon("%d %s"), u("900\nBig\nTank"), "price first, then the name");
        assert_eq!(icon("%s%s|%d%d"), u("Big\nTank|900"), "extra points: nothing");
        assert_eq!(
            icon("%s|%-9s|%09d|%09d|"),
            u("Big\nTank||000000900||"),
            "no value: no width either"
        );
        assert_eq!(power("%d|%d|%d|%s|%5d|%-5s"), u("300|-20||||"));
        assert_eq!(power("%s"), u(""));
        assert_eq!(power("x"), u("x"), "values left over are not written");
    }

    /// spec: S-0411 claim 23 (Forms: a text precision is at most that many text units; d and i
    /// decimal with a minus sign; u, o, x and X take v + 2³² for a negative v; p as X with a
    /// precision of 8; a precision is the least number of digits, 1 without one, and 0 writes
    /// no digits for the value 0; h the lowest 16 bits, signed for d and i; l, w, F, L and N
    /// change nothing)
    #[test]
    fn claim_23_forms() {
        let t = |e: &str, v: &str| fill(&u(e), &[&u(v)], &[]);
        let n = |e: &str, v: i32| fill(&u(e), &[], &[v]);
        assert_eq!(t("%.0s|", "abc"), u("|"));
        assert_eq!(t("%.2s|", "abc"), u("ab|"));
        assert_eq!(t("%.9s|", "abc"), u("abc|"));
        assert_eq!(t("%.s|", "abc"), u("|"), "a full stop alone counts as 0");
        assert_eq!(n("%i", i32::MIN), u("-2147483648"));
        assert_eq!(n("%u", -2), u("4294967294"));
        assert_eq!(n("%o", -1), u("37777777777"));
        assert_eq!(n("%x|", -1), u("ffffffff|"));
        assert_eq!(n("%X", 0xBEEF), u("BEEF"));
        assert_eq!(n("%p", 255), u("000000FF"));
        assert_eq!(n("%p", -1), u("FFFFFFFF"));
        assert_eq!(n("%.5d", -42), u("-00042"));
        assert_eq!(n("%.5x", 255), u("000ff"));
        assert_eq!(n("%.0d|", 0), u("|"));
        assert_eq!(n("%.d|", 0), u("|"));
        assert_eq!(n("%.0x|", 0), u("|"));
        assert_eq!(n("%.0d", 3), u("3"));
        assert_eq!(n("%d", 0), u("0"));
        assert_eq!(n("%hd", 65535), u("-1"));
        assert_eq!(n("%hi", 0x0001_8000), u("-32768"));
        assert_eq!(n("%hu", -1), u("65535"));
        assert_eq!(n("%hx", 0x0012_3456), u("3456"));
        assert_eq!(n("%lhX", 0x0012_ABCD), u("ABCD"));
        assert_eq!(n("%ld|%wFLNu", -3), u("-3|"));
        assert_eq!(fill(&u("%ld|%wFLNu"), &[], &[-3, -3]), u("-3|4294967293"));
    }

    /// spec: S-0411 claim 23 (Flags and width: plus and space for d and i; hash for o, x, X and
    /// p; the width counts sign and prefix; spaces before, after with minus; zeros with the zero
    /// flag between sign or prefix and the digits, spaces for a number point with a precision;
    /// the zero flag pads a text point; claim 20 turns a width's spaces into line feeds for the
    /// money-format tip, the power tip keeps them)
    #[test]
    fn claim_23_flags_and_width() {
        let n = |e: &str, v: i32| fill(&u(e), &[], &[v]);
        let t = |e: &str, v: &str| fill(&u(e), &[&u(v)], &[]);
        assert_eq!(n("%+d", 5), u("+5"));
        assert_eq!(n("%+d", 0), u("+0"));
        assert_eq!(n("%+d", -5), u("-5"));
        assert_eq!(n("% d", 5), u(" 5"));
        assert_eq!(n("% +d", 5), u("+5"), "plus wins over space");
        assert_eq!(n("%+u", 5), u("5"), "no sign for u");
        assert_eq!(n("% x", 5), u("5"));
        assert_eq!(n("%#o", 8), u("010"));
        assert_eq!(n("%#o", 0), u("0"));
        assert_eq!(n("%#.3o", 8), u("010"), "the digits already start with 0");
        assert_eq!(n("%#.0o", 0), u("0"));
        assert_eq!(n("%#x", 0), u("0"));
        assert_eq!(n("%#X", 171), u("0XAB"));
        assert_eq!(n("%#p", 171), u("0X000000AB"));
        assert_eq!(n("%#p", 0), u("00000000"));
        assert_eq!(n("%#u", 9), u("9"));
        assert_eq!(n("%6d|", -42), u("   -42|"));
        assert_eq!(n("%-6d|", -42), u("-42   |"));
        assert_eq!(n("%06d", -42), u("-00042"));
        assert_eq!(n("%+06d", 42), u("+00042"));
        assert_eq!(n("% 06d", 42), u(" 00042"));
        assert_eq!(n("%#08x", 255), u("0x0000ff"));
        assert_eq!(n("%#8x", 255), u("    0xff"));
        assert_eq!(n("%-#8X|", 255), u("0XFF    |"));
        assert_eq!(n("%-06d|", 42), u("42    |"), "minus wins over zero");
        assert_eq!(n("%08.3d", 7), u("     007"), "a precision: spaces");
        assert_eq!(n("%012p", 255), u("    000000FF"), "p has a precision");
        assert_eq!(n("%2d", 12345), u("12345"), "no cut to the width");
        assert_eq!(t("%05s", "ab"), u("000ab"));
        assert_eq!(t("%-05s|", "ab"), u("ab   |"));
        assert_eq!(t("%05.1s", "ab"), u("0000a"), "a text precision keeps the zeros");
        assert_eq!(t("%+ #3s", "ab"), u(" ab"), "other flags add nothing to a text");
        // The money-format tip: a width's spaces become line feeds; the power tip keeps them.
        assert_eq!(
            icon_text(Looked::Text(u("%-4s|%5d")), &u("ab"), 12),
            u("ab\n\n|\n\n\n12")
        );
        assert_eq!(
            power_text(Looked::Text(u("%4d|%-4d|")), (1, 2)),
            u("   1|2   |")
        );
    }

    /// spec: S-0411 Edge cases (Icon-tip text of 66 text units or more, and a power-meter text
    /// with a large width: shown as S-0111's name tip shows a long text) and S-0111 Edge cases
    /// (the first 256 text units)
    #[test]
    fn claim_23_a_large_width_is_cut_as_a_long_text() {
        let n = |e: &str, v: i32| fill(&u(e), &[], &[v]);
        let long = n("%300d", 1);
        assert_eq!(long.len(), super::super::name_tip::TEXT_LIMIT);
        assert!(long.iter().all(|&c| c == SPACE));
        let left = n("%-300d", 1);
        assert_eq!(left.len(), super::super::name_tip::TEXT_LIMIT);
        assert_eq!(left[0], u16::from(b'1'));
        let digits = n("%.999999999999999999999999d", 1);
        assert_eq!(digits.len(), super::super::name_tip::TEXT_LIMIT);
        assert!(digits.iter().all(|&c| c == ZERO));
        assert_eq!(
            fill(&u("%99999999999999999999999s"), &[&u("a")], &[]).len(),
            super::super::name_tip::TEXT_LIMIT
        );
    }

    /// spec: S-0411 claim 20 (Text: every space, from the display name and from the entry alike, a
    /// line feed; a negative price written with its minus sign) and Import mapping (a missing
    /// money-format entry: the placeholder alone)
    #[test]
    fn claim_20_icon_text() {
        assert_eq!(
            icon_text(Looked::Text(u("%s costs %d")), &u("Big Tank"), 900),
            u("Big\nTank\ncosts\n900")
        );
        assert_eq!(
            icon_text(Looked::Text(u("<%s|%d>")), &u("a b"), -3),
            u("<a\nb|-3>")
        );
        assert_eq!(
            icon_text(Looked::Missing(u("MISSING:'X Y'")), &u("a"), 1),
            u("MISSING:'X Y'")
        );
        assert_eq!(
            power_text(Looked::Text(u("Power %d of %d")), (300, -20)),
            u("Power 300 of -20"),
            "claim 22: spaces kept"
        );
        assert_eq!(power_text(Looked::Missing(u("M")), (1, 2)), u("M"));
    }

    /// spec: S-0411 claim 22 (a disabled tab or scroll button: its label, a line feed and the
    /// disabled label in parentheses)
    #[test]
    fn claim_22_disabled_line() {
        assert_eq!(with_disabled(u("Tab"), false, &u("off")), u("Tab"));
        assert_eq!(
            with_disabled(u("Tab a"), true, &u("off x")),
            u("Tab a\n(off x)")
        );
    }

    /// spec: S-0411 claim 21 (the box by the slot: toward the middle of the tip area, overlapping
    /// the icon's corner by 5 each way; l₁ = max(l₀, A_x); the right-edge correction; m =
    /// max(m₀, A_y); no correction at the bottom edge; ⌊ ⌋ rounding down)
    #[test]
    fn claim_21_slot_place() {
        let area = Rect::new(100, 200, 168, 401);
        // A_x + ⌊A_w / 2⌋ = 184, A_y + ⌊A_h / 2⌋ = 400.
        let upper_left = Rect::new(114, 210, 68, 52);
        assert_eq!(slot_place(upper_left, (40, 30), area), (177, 257));
        let lower_right = Rect::new(186, 500, 68, 52);
        assert_eq!(slot_place(lower_right, (40, 30), area), (151, 475));
        // xs = 184 is not left of the middle: placed leftward.
        assert_eq!(
            slot_place(Rect::new(184, 399, 68, 52), (20, 10), area).0,
            169
        );
        assert_eq!(
            slot_place(Rect::new(184, 400, 68, 52), (20, 10), area).1,
            395
        );
        // Left: no box starts left of the area.
        assert_eq!(slot_place(lower_right, (150, 30), area).0, 100);
        // Right: a box passing the area's right edge ends on it.
        assert_eq!(slot_place(upper_left, (100, 30), area).0, 168);
        // Top: m = max(m₀, A_y); bottom: no correction.
        assert_eq!(
            slot_place(Rect::new(186, 410, 68, 52), (20, 300), area).1,
            200
        );
        assert_eq!(
            slot_place(Rect::new(114, 390, 68, 52), (20, 300), area).1,
            437
        );
        assert_eq!(wrap_width(area), 160);
    }

    /// spec: S-0411 claim 22 (the controls of the table, and nothing elsewhere on the sidebar:
    /// the radar panel, the credits counter, a gap) and Terms (Slot: every shown place, filled
    /// or empty); design: our slots tile the strip, gaps included
    #[test]
    fn claim_22_places() {
        let l = Layout::new((800, 600));
        let mid = |r: Rect| Point::new(r.x + r.w / 2, r.y + r.h / 2);
        assert_eq!(place_at(&l, mid(l.map)), None);
        assert_eq!(place_at(&l, mid(l.radar)), Some(Place::Nothing));
        assert_eq!(place_at(&l, mid(l.credits)), Some(Place::Nothing));
        assert_eq!(
            place_at(&l, mid(l.power)),
            Some(Place::Control(Control::PowerMeter))
        );
        assert_eq!(
            place_at(&l, mid(l.options)),
            Some(Place::Control(Control::Options))
        );
        assert_eq!(
            place_at(&l, mid(l.modes[0])),
            Some(Place::Control(Control::Sell))
        );
        assert_eq!(
            place_at(&l, mid(l.modes[1])),
            Some(Place::Control(Control::Repair))
        );
        assert_eq!(
            place_at(&l, mid(l.modes[2])),
            Some(Place::Control(Control::PowerMode))
        );
        for i in 0..4 {
            assert_eq!(
                place_at(&l, mid(l.tabs[i])),
                Some(Place::Control(Control::Tab(i)))
            );
        }
        assert_eq!(
            place_at(&l, mid(l.scroll[1])),
            Some(Place::Control(Control::Scroll(1)))
        );
        for k in 0..l.rows() * 2 {
            assert_eq!(place_at(&l, mid(l.cameo(k))), Some(Place::Slot(k)));
        }
        // The gap right of the first icon and below it: still its slot.
        let c = l.cameo(0);
        assert_eq!(
            place_at(&l, Point::new(c.right() + 1, c.y)),
            Some(Place::Slot(0))
        );
        assert_eq!(
            place_at(&l, Point::new(c.x, c.bottom() + 1)),
            Some(Place::Slot(0))
        );
        // A gap between the tab buttons: nothing.
        let t = l.tabs[0];
        assert_eq!(
            place_at(&l, Point::new(t.right() + 1, t.y)),
            Some(Place::Nothing)
        );
        // The tip area: below the radar's frame to the bottom, the whole width.
        let a = l.tip_area();
        assert_eq!((a.x, a.w, a.bottom()), (l.bar.x, l.bar.w, l.bar.bottom()));
        assert_eq!(a.y, l.radar.bottom() + 1);
    }
}
