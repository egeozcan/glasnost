//! The name tip over the battlefield (S-0111 claims 20 to 26): its
//! real-time wait and lifetime, what hides it, its layout in the game font,
//! its placement and its look. What text it shows is the session's
//! (`Session::name_tip_text`); nothing here reaches the simulation.

use glasnost_ui::draw::DrawList;
use glasnost_ui::geom::Rect;

use super::paint::Painter;
use super::status_font::{self, StatusFont};

/// How long the pointer rests before a tip shows, ms of real time.
pub const WAIT_MS: i64 = 1000; // spec: S-0111 claim 20 (the tip shows once the pointer has rested on the battlefield view for 1000 ms of real time)
/// How long a tip stays once shown, ms of real time.
pub const LIFETIME_MS: i64 = 10_000; // spec: S-0111 claim 20 (a tip stays for 10000 ms of real time after it shows, then hides)
/// The most text units a tip shows.
pub const TEXT_LIMIT: usize = 256; // spec: S-0111 Constants (name-tip text limit 256) and Edge cases (departure, no switch: the first 256 text units)
/// The box's distance below the pointer, pixels.
pub const BELOW: i32 = 16; // spec: S-0111 claim 24 (m = y + 16)
/// The box's width beyond the widest line, pixels.
pub const EXTRA_W: i32 = 4; // spec: S-0111 claim 24 (w = w_t + 4)
/// The box's height beyond the lines, pixels.
pub const EXTRA_H: i32 = 3; // spec: S-0111 claim 24 (h = h_t + 3)
/// The tab stop of a tip line, pixels.
pub const TAB_STOP: i32 = 64; // spec: S-0111 claim 24 (a tab moves the line's width up to the next multiple of 64)
/// The extra pixel each text unit adds to a line's width.
pub const EXTRA_ADVANCE: i32 = 1; // spec: S-0111 claim 24 (each text unit's glyph width plus 1)
/// The tip colour when the local player's side index is 0.
pub const INK_SIDE_0: [u8; 3] = [164, 210, 255]; // spec: S-0111 claim 25 (RGB (164, 210, 255) for side index 0)
/// The tip colour for any other side index.
pub const INK_OTHER: [u8; 3] = [255, 255, 0]; // spec: S-0111 claim 25 (RGB (255, 255, 0) for any other side index)
/// Where the text sits inside the box, pixels from its top left.
// design: Q-2687 (S-0111 claim 25 leaves the text's inset inside the frame open: 2 and 2, inside the 1-pixel frame with one free pixel, which the margins of 4 and 3 leave room for)
pub const INSET: (i32, i32) = (2, 2);
/// The painter's sprite id of the tip glyphs in side index 0's colour; the
/// next id holds them in the other colour.
// nocite: our own sprite ids, after the status glyphs'
pub const SPRITE: u32 = 951;

const TAB: u16 = 9;
const LF: u16 = 10;
const CR: u16 = 13;

/// A tip that shows: its lines, its box and when it goes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Shown {
    /// The text's lines, without their breaks.
    pub lines: Vec<Vec<u16>>,
    /// The box in screen pixels.
    pub rect: Rect,
    /// The real time it hides at, ms.
    pub until: i64,
    /// The tip colour.
    pub ink: [u8; 3],
}

/// The tip's state: the rest the pointer is in and the tip that shows.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NameTip {
    /// The real time the pointer came to rest, while the wait runs.
    rest: Option<i64>,
    /// The tip that shows.
    pub shown: Option<Shown>,
}

impl NameTip {
    /// The pointer moved, by any distance: a showing tip hides and the
    /// wait starts again.
    pub fn moved(&mut self, now: i64) {
        // spec: S-0111 claim 20 (every pointer movement, of any distance and whether or not the pointer object changes, hides a tip that is showing and starts the wait again) and claim 26
        self.shown = None;
        self.rest = Some(now);
    }

    /// A left, right or middle button went down or up: the tip hides and
    /// the wait stops until the pointer moves.
    pub fn button(&mut self) {
        // spec: S-0111 claim 20 (pressing or releasing the left, right or middle mouse button hides the tip and stops the wait; no tip until the pointer has moved and rested again) and claim 26
        self.shown = None;
        self.rest = None;
    }

    /// The tooltip option is off: no tip shows, and one that shows hides.
    pub fn off(&mut self) {
        // spec: S-0111 claim 20 (with the option off no name tip ever shows) and claim 26 (turning it off hides a tip)
        self.shown = None;
        self.rest = None;
    }

    /// Whether the wait has ended at `now` without a choice made yet.
    pub fn due(&self, now: i64) -> bool {
        // spec: S-0111 claim 20 (the wait: 1000 ms of real time at rest)
        self.rest.is_some_and(|s| now - s >= WAIT_MS)
    }

    /// One frame at real time `now`: a tip whose lifetime is over hides;
    /// when the wait ends, the rest's one choice is made: no tip while
    /// `blocked`, else `choose` gives the tip, if any.
    pub fn update(&mut self, now: i64, blocked: bool, choose: impl FnOnce() -> Option<Shown>) {
        if self.shown.as_ref().is_some_and(|s| now >= s.until) {
            // spec: S-0111 claim 20 (Lifetime: it hides after 10000 ms and does not show again until the pointer moves and rests again)
            self.shown = None;
        }
        if !self.due(now) {
            return;
        }
        // spec: S-0111 claim 20 (One choice per rest: the text is chosen once, when the wait ends; nothing then shows until the pointer moves and rests again)
        self.rest = None;
        if blocked {
            // spec: S-0111 claim 20 (Blocked: no tip shows when the wait ends while the cursor is hidden, the input is locked or the view is moving)
            return;
        }
        self.shown = choose();
    }
}

/// The text a tip shows: the first [`TEXT_LIMIT`] text units.
pub fn cut(units: &[u16]) -> &[u16] {
    // spec: S-0111 Edge cases (name-tip text of 256 text units or more, departure, no switch: our engine shows the first 256 text units)
    &units[..units.len().min(TEXT_LIMIT)]
}

/// A line's width: each text unit's glyph width plus 1, a tab moving the
/// width up to the next multiple of 64.
pub fn line_width(line: &[u16], glyph_width: &impl Fn(u16) -> i32) -> i32 {
    line.iter().fold(0, |x, &c| advance(x, c, glyph_width))
}

fn advance(x: i32, c: u16, glyph_width: &impl Fn(u16) -> i32) -> i32 {
    if c == TAB {
        // spec: S-0111 claim 24 (a tab moves the line's width up to the next multiple of 64)
        (x.div_euclid(TAB_STOP) + 1) * TAB_STOP
    } else {
        // spec: S-0111 claim 24 (the sum over its text units of each glyph width plus 1)
        x + glyph_width(c) + EXTRA_ADVANCE
    }
}

/// The lines of `units`: a carriage return, a line feed or a carriage
/// return followed by a line feed starts a new line, and a line wider than
/// `view_w` is wrapped.
pub fn lines(units: &[u16], glyph_width: &impl Fn(u16) -> i32, view_w: i32) -> Vec<Vec<u16>> {
    let mut out = vec![Vec::new()];
    let mut i = 0;
    while i < units.len() {
        let c = units[i];
        if c == CR || c == LF {
            // spec: S-0111 claim 24 (a carriage return or a line feed starts a new line, a carriage return followed by a line feed counting as one)
            if c == CR && units.get(i + 1) == Some(&LF) {
                i += 1;
            }
            out.push(Vec::new());
        } else {
            let line = out.last_mut().expect("one line at least");
            // design: Q-2687 (S-0111 claim 24 wraps lines to the view's width but leaves how open: a line breaks before the text unit that would take it past the width, keeping one unit at least)
            if !line.is_empty() && advance(line_width(line, glyph_width), c, glyph_width) > view_w {
                out.push(vec![c]);
            } else {
                line.push(c);
            }
        }
        i += 1;
    }
    out
}

/// The box's size for `lines` with font line pitch `pitch`.
pub fn size(lines: &[Vec<u16>], glyph_width: &impl Fn(u16) -> i32, pitch: i32) -> (i32, i32) {
    let wt = lines
        .iter()
        .map(|l| line_width(l, glyph_width))
        .max()
        .unwrap_or(0);
    // spec: S-0111 claim 24 (h_t is the number of lines times the font's line pitch; w = w_t + 4, h = h_t + 3)
    (wt + EXTRA_W, lines.len() as i32 * pitch + EXTRA_H)
}

/// The box's top-left corner for a `w` × `h` box under the pointer at
/// `(x, y)` in a view whose right and bottom edges (exclusive) are `x1`
/// and `y1`.
pub fn place((x, y): (i32, i32), (w, h): (i32, i32), (x1, y1): (i32, i32)) -> (i32, i32) {
    // spec: S-0111 claim 24 (l = x, or X1 − w when x + w > X1; m = y + 16, or y − h when y + 16 + h > Y1; no correction at the left or top edge)
    let l = if x + w > x1 { x1 - w } else { x };
    let m = if y + BELOW + h > y1 { y - h } else { y + BELOW };
    (l, m)
}

/// The tip colour for the local player's side index.
pub fn ink(side: u32) -> [u8; 3] {
    // spec: S-0111 claim 25 (the tip colour: side index 0 or any other)
    if side == 0 { INK_SIDE_0 } else { INK_OTHER }
}

/// The painter's sprite id of the tip glyphs in tip colour `ink`: the
/// game font's glyphs once per tip colour.
pub fn sprite(ink: [u8; 3]) -> u32 {
    if ink == INK_SIDE_0 {
        SPRITE
    } else {
        SPRITE + 1
    }
}

/// Registers with `painter` the glyphs tip `s` draws in its colour; `rows`
/// keeps the palette row of each tip colour, made on first use.
pub fn register(painter: &mut Painter, font: &StatusFont, s: &Shown, rows: &mut [Option<u32>; 2]) {
    let k = usize::from(s.ink != INK_SIDE_0);
    let [r, g, b] = s.ink;
    let row = *rows[k].get_or_insert_with(|| status_font::ink_row(painter, [r, g, b, 255]));
    let units = s.lines.iter().flat_map(|l| l.iter().copied());
    status_font::register_glyphs(painter, font, units, sprite(s.ink), row);
}

/// Draws tip `s` in the battlefield view `view`: its box (`draw_box`),
/// then its lines in the game font, inset by [`INSET`] and one line pitch
/// apart, each glyph's first min(*H*, *P*) rows painted, inside the frame.
pub fn draw(d: &mut DrawList, font: &StatusFont, s: &Shown, view: Rect) {
    draw_box(d, s, view);
    let r = s.rect;
    // spec: S-0111 claim 25 (the text sits inside the frame)
    let inside = view.intersect(&Rect::new(r.x + 1, r.y + 1, r.w - 2, r.h - 2));
    // spec: S-0111 claim 24 (h_t is the number of lines times the line pitch P_f: a line per pitch)
    let pitch = font.line_pitch() as i32;
    // spec: S-0213 claim 25 (the game font's rows 0 ≤ r < min(H, P) of a line are painted), as the status text the tip shares its font with
    let rows = font.painted_rows().min(r.h.max(0) as u64) as i32;
    let sprite = sprite(s.ink);
    for (i, line) in s.lines.iter().enumerate() {
        let y = r.y + INSET.1 + i as i32 * pitch;
        let clip = inside.intersect(&Rect::new(r.x, y, r.w, rows));
        if clip.is_empty() {
            continue;
        }
        // spec: S-0111 claim 24 (each text unit's glyph width plus 1, a tab to the next multiple of 64): the glyphs at the widths the box is laid out with
        let width = |c: u16| font.width(c);
        let mut dx = 0;
        for &c in line {
            if c != TAB {
                let g = font.glyph(c);
                d.sprite(sprite, status_font::frame(g), r.x + INSET.0 + dx, y, clip);
            }
            dx = advance(dx, c, &width);
        }
    }
}

/// A tip for `units` under the pointer at `p` in the battlefield view
/// `view`, shown at `now`: `None` for an empty text.
pub fn tip(
    units: &[u16],
    p: (i32, i32),
    view: Rect,
    font: (&impl Fn(u16) -> i32, i32),
    ink: [u8; 3],
    now: i64,
) -> Option<Shown> {
    let units = cut(units);
    if units.is_empty() {
        // spec: S-0111 claim 20 (if there is no text, or it is empty, nothing shows)
        return None;
    }
    let (glyph_width, pitch) = font;
    let lines = lines(units, glyph_width, view.w);
    let (w, h) = size(&lines, glyph_width, pitch);
    // spec: S-0111 claim 24 (the box is placed once, from the pointer's position when the tip shows; it does not move while it shows)
    let (l, m) = place(p, (w, h), (view.right(), view.bottom()));
    Some(Shown {
        lines,
        rect: Rect::new(l, m, w, h),
        // spec: S-0111 claim 20 (Lifetime: 10000 ms after it shows)
        until: now + LIFETIME_MS,
        ink,
    })
}

/// Draws the box of `s`: opaque black, framed by a 1-pixel line in the tip
/// colour, clipped to `view`. The text is the caller's (the game font's
/// sprites in the tip colour).
pub fn draw_box(d: &mut DrawList, s: &Shown, view: Rect) {
    let r = s.rect;
    let [cr, cg, cb] = s.ink;
    // spec: S-0111 claim 25 (filled with opaque black and framed by a line one pixel wide along its four edges in the tip colour; drawn as it is, with no shade table)
    let clip = |x: Rect| view.intersect(&x);
    d.fill(clip(r), [0, 0, 0, 255]);
    let ink = [cr, cg, cb, 255];
    for e in [
        Rect::new(r.x, r.y, r.w, 1),
        Rect::new(r.x, r.bottom() - 1, r.w, 1),
        Rect::new(r.x, r.y, 1, r.h),
        Rect::new(r.right() - 1, r.y, 1, r.h),
    ] {
        d.fill(clip(e), ink);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn u(s: &str) -> Vec<u16> {
        s.encode_utf16().collect()
    }

    /// Every glyph 4 pixels wide, `W` 9.
    fn gw(c: u16) -> i32 {
        if c == u16::from(b'W') { 9 } else { 4 }
    }

    fn shown(now: i64) -> Option<Shown> {
        tip(
            &u("ab"),
            (10, 10),
            Rect::new(0, 0, 640, 400),
            (&gw, 12),
            INK_SIDE_0,
            now,
        )
    }

    /// spec: S-0111 claim 20 (the 1000 ms wait, restarted by every movement; one choice per
    /// rest; the 10000 ms lifetime; real time: the caller's clock, whatever the game does)
    #[test]
    fn claim_20_wait_lifetime_and_one_choice() {
        let mut t = NameTip::default();
        t.update(5000, false, || panic!("no rest yet"));
        t.moved(1000);
        t.update(1999, false, || panic!("the wait runs"));
        assert!(t.shown.is_none());
        t.moved(1500);
        t.update(2400, false, || panic!("the wait started again"));
        let mut asked = 0;
        t.update(2500, false, || {
            asked += 1;
            shown(2500)
        });
        assert_eq!(asked, 1);
        assert_eq!(t.shown.as_ref().unwrap().until, 12_500);
        t.update(6000, false, || panic!("one choice per rest"));
        assert!(t.shown.is_some());
        t.update(12_499, false, || panic!("one choice per rest"));
        assert!(t.shown.is_some());
        t.update(12_500, false, || panic!("no second tip in the same rest"));
        assert!(t.shown.is_none(), "the lifetime ended");
        // An empty choice shows nothing, and nothing until the pointer moves.
        t.moved(20_000);
        t.update(21_000, false, || None);
        t.update(22_000, false, || panic!("one choice per rest"));
        assert!(t.shown.is_none());
    }

    /// spec: S-0111 claims 20 and 26 (a press or release hides the tip and stops the wait until
    /// the pointer moves; a movement hides it; turning the option off hides it; while blocked when
    /// the wait ends, nothing shows)
    #[test]
    fn claims_20_and_26_what_hides_it() {
        let mut t = NameTip::default();
        t.moved(0);
        t.update(1000, false, || shown(1000));
        assert!(t.shown.is_some());
        t.button();
        assert!(t.shown.is_none());
        t.update(5000, false, || panic!("the wait stopped"));
        t.moved(5000);
        t.button();
        t.update(7000, false, || panic!("the wait stopped"));
        t.moved(7000);
        t.update(8000, false, || shown(8000));
        t.moved(8100);
        assert!(t.shown.is_none(), "a movement hides it");
        t.update(9100, true, || panic!("blocked: no choice"));
        assert!(t.shown.is_none());
        t.update(9200, false, || panic!("the rest's choice is spent"));
        t.moved(10_000);
        t.update(11_000, false, || shown(11_000));
        t.off();
        assert!(t.shown.is_none());
        t.update(13_000, false, || panic!("off: no wait"));
    }

    /// spec: S-0111 claim 24 (width: glyph widths plus 1 per unit, a tab to the next multiple of 64;
    /// CR, LF and CRLF each start one new line; w = w_t + 4, h = lines × P_f + 3) and Q-2687 (the
    /// wrap before the unit that would pass the view's width)
    #[test]
    fn claim_24_layout() {
        assert_eq!(line_width(&u("ab"), &gw), 10);
        assert_eq!(line_width(&u("W"), &gw), 10);
        assert_eq!(line_width(&u("a\tb"), &gw), 69);
        assert_eq!(line_width(&u("\t"), &gw), 64);
        assert_eq!(
            line_width(&u("aaaaaaaaaaaaa\t"), &gw),
            128,
            "65 goes up to 128"
        );
        let l = lines(&u("ab\rc\nd\r\ne"), &gw, 640);
        assert_eq!(l, vec![u("ab"), u("c"), u("d"), u("e")]);
        assert_eq!(lines(&u("a\n\nb"), &gw, 640).len(), 3);
        assert_eq!(size(&l, &gw, 12), (10 + 4, 4 * 12 + 3));
        assert_eq!(lines(&u("aaaa"), &gw, 12), vec![u("aa"), u("aa")]);
        assert_eq!(lines(&u("W"), &gw, 3), vec![u("W")], "one unit at least");
    }

    /// spec: S-0111 claim 24 (16 below the pointer at its column; the right-edge shift; the flip
    /// above the pointer near the bottom; no left or top correction)
    #[test]
    fn claim_24_placement() {
        let edges = (640, 400);
        assert_eq!(place((100, 100), (50, 20), edges), (100, 116));
        assert_eq!(place((600, 100), (50, 20), edges), (590, 116));
        assert_eq!(
            place((590, 100), (50, 20), edges),
            (590, 116),
            "x + w = X1 fits"
        );
        assert_eq!(
            place((100, 364), (50, 20), edges),
            (100, 380),
            "y + 16 + h = Y1 fits"
        );
        assert_eq!(
            place((100, 365), (50, 20), edges),
            (100, 345),
            "last row on y − 1"
        );
        assert_eq!(
            place((10, 5), (700, 500), edges),
            (-60, -495),
            "no left or top correction"
        );
    }

    /// spec: S-0111 claim 25 (black fill, a 1-pixel frame, frame and text in the tip colour by
    /// side index) and Edge cases (the first 256 text units)
    #[test]
    fn claim_25_look_and_the_text_limit() {
        assert_eq!(ink(0), [164, 210, 255]);
        assert_eq!(ink(1), [255, 255, 0]);
        assert_eq!(ink(2), [255, 255, 0]);
        let long = vec![u16::from(b'a'); 300];
        assert_eq!(cut(&long).len(), 256);
        assert_eq!(cut(&long[..255]).len(), 255);
        assert!(
            tip(
                &[],
                (0, 0),
                Rect::new(0, 0, 640, 400),
                (&gw, 12),
                INK_OTHER,
                0
            )
            .is_none()
        );
        let s = shown(0).unwrap();
        let mut d = DrawList::new();
        draw_box(&mut d, &s, Rect::new(0, 0, 640, 400));
        let fills: Vec<(Rect, [u8; 4])> = d
            .prims
            .iter()
            .filter_map(|p| match p {
                glasnost_ui::draw::Prim::Fill { rect, rgba } => Some((*rect, *rgba)),
                _ => None,
            })
            .collect();
        assert_eq!(fills[0], (s.rect, [0, 0, 0, 255]));
        assert_eq!(fills.len(), 5);
        assert!(fills[1..].iter().all(|(_, c)| *c == [164, 210, 255, 255]));
    }

    /// spec: S-0111 claims 24 and 25 (drawn in the game font with the layout's widths: the text in
    /// the tip colour inside the frame, over the black fill, inset by our Q-2687 margin; one line
    /// pitch per line) and S-0014 claim 5 (one bits paint the foreground)
    #[test]
    fn claims_24_and_25_the_drawn_tip() {
        let font = StatusFont::new(glasnost_fmt::fnt::synthetic_font()).unwrap();
        let view = Rect::new(0, 0, 120, 80);
        let width = |c: u16| font.width(c);
        let pitch = font.line_pitch() as i32;
        let s = tip(&u("A\nA"), (10, 10), view, (&width, pitch), INK_OTHER, 0).unwrap();
        assert_eq!(s.rect, Rect::new(10, 26, 5 + 4, 2 * pitch + 3));
        let mut painter = Painter::new();
        let mut rows = [None; 2];
        register(&mut painter, &font, &s, &mut rows);
        assert!(rows[1].is_some() && rows[0].is_none());
        let mut d = DrawList::new();
        draw(&mut d, &font, &s, view);
        let quads = painter.quads(&d);
        let size = (120, 80);
        let px = painter.reference(&quads, &Default::default(), size);
        let at = |x: i32, y: i32| {
            let i = ((y * size.0 + x) * 4) as usize;
            [px[i], px[i + 1], px[i + 2]]
        };
        let r = s.rect;
        // The frame and the fill.
        assert_eq!(at(r.x, r.y), INK_OTHER);
        assert_eq!(at(r.right() - 1, r.bottom() - 1), INK_OTHER);
        assert_eq!(at(r.x + 1, r.bottom() - 2), [0, 0, 0]);
        // "A" row 0 is ".##.": its set pixels at the inset, on both lines.
        let (x, y) = (r.x + INSET.0, r.y + INSET.1);
        assert_eq!(at(x, y), [0, 0, 0]);
        assert_eq!(at(x + 1, y), INK_OTHER);
        assert_eq!(at(x + 1, y + pitch), INK_OTHER);
        // Nothing outside the box.
        let outside = (0..size.1)
            .flat_map(|y| (0..size.0).map(move |x| (x, y)))
            .filter(|&(x, y)| !r.contains(glasnost_ui::geom::Point::new(x, y)))
            .any(|(x, y)| at(x, y) != [0, 0, 0]);
        assert!(!outside);
    }
}
