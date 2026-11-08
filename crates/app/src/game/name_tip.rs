//! The name tip over the battlefield (S-0111 claims 20 to 26) and the
//! sidebar's tips that share its timing, layout and look (S-0411 claims 20
//! to 22): the real-time wait and lifetime, the build-icon slots' tips
//! shown at once, what hides a tip, its layout in the game font, its
//! placement and its look. What text a tip shows is the session's
//! (`Session::name_tip_text`, [`super::sidebar_tip`]); nothing here reaches
//! the simulation.

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
pub const EXTRA_W: i32 = 8; // spec: S-0111 claim 24 (r40: w = w_t + 8) and Constants (margins 8 and 4)
/// The box's height beyond the lines, pixels.
pub const EXTRA_H: i32 = 4; // spec: S-0111 claim 24 (r40: h = h_t + 4)
/// What the wrap width takes from the view's width, pixels.
pub const WRAP_ALLOWANCE: i32 = 8; // spec: S-0111 claim 24 (r40: lines wrap to the wrap width W − 8) and Constants (wrap allowance 8)
/// The tab stop of a tip line, pixels.
pub const TAB_STOP: i32 = 64; // spec: S-0111 claim 24 (a tab moves the line's width up to the next multiple of 64)
/// The extra pixel each text unit adds to a line's width.
pub const EXTRA_ADVANCE: i32 = 1; // spec: S-0111 claim 24 (each text unit's glyph width plus 1)
/// The tip colour when the local player's side index is 0.
pub const INK_SIDE_0: [u8; 3] = [164, 210, 255]; // spec: S-0111 claim 25 (RGB (164, 210, 255) for side index 0)
/// The tip colour for any other side index.
pub const INK_OTHER: [u8; 3] = [255, 255, 0]; // spec: S-0111 claim 25 (RGB (255, 255, 0) for any other side index)
/// Where the text sits inside the box, pixels from its top left.
pub const INSET: (i32, i32) = (4, 2); // spec: S-0111 claim 25 (r40: the text's origin is 4 pixels right of the box's left edge and 2 below its top edge) and Constants (text origin 4, 2)
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
    /// The text the tip was chosen for (cut to the text limit), to tell a
    /// text chosen again that differs.
    pub text: Vec<u16>,
    /// The text's lines, without their breaks.
    pub lines: Vec<Vec<u16>>,
    /// The box in screen pixels.
    pub rect: Rect,
    /// The real time it hides at, ms.
    pub until: i64,
    /// The tip colour.
    pub ink: [u8; 3],
    /// The screen rectangle the tip is cut at: the battlefield view for a
    /// name tip, the sidebar for a sidebar tip.
    pub clip: Rect,
}

/// The tip's state: the rest the pointer is in and the tip that shows.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NameTip {
    /// The real time the pointer came to rest, while the wait runs.
    rest: Option<i64>,
    /// The tip that shows.
    pub shown: Option<Shown>,
    /// Whether a drawn frame has passed with the pointer on a build-icon
    /// slot since its last movement that ended off the slots.
    armed: bool,
    /// The real time of a movement over the slots whose tip shows at once,
    /// until the next drawn frame shows it.
    at_once: Option<i64>,
}

impl NameTip {
    /// The pointer moved, by any distance, ending on a build-icon slot of
    /// the sidebar when `on_slot`. Over the slots, once a drawn frame has
    /// passed with the pointer on one, the movement shows the slot's tip at
    /// once (at the next drawn frame); any other movement hides a showing
    /// tip and starts the wait again.
    pub fn moved(&mut self, now: i64, on_slot: bool) {
        self.shown = None;
        if on_slot && self.armed {
            // spec: S-0411 claim 20 (Over the slots, no wait: once one drawn frame has passed with the pointer on a slot, every later movement that ends on a slot shows the tip of that slot's icon at once; one that ends on an empty slot hides it) and S-0111 claim 20 (the one exception over the build-icon slots)
            self.rest = None;
            self.at_once = Some(now);
            return;
        }
        // spec: S-0111 claim 20 (every pointer movement, of any distance and whether or not the pointer object changes, hides a tip that is showing and starts the wait again) and claim 26; S-0411 claim 20 (a movement that brings the pointer onto the slots from anywhere else, or that ends off them, is an ordinary one)
        self.rest = Some(now);
        self.at_once = None;
        if !on_slot {
            self.armed = false;
        }
    }

    /// A left, right or middle button went down or up: the tip hides and
    /// the wait stops until the pointer moves.
    pub fn button(&mut self) {
        // spec: S-0111 claim 20 (pressing or releasing the left, right or middle mouse button hides the tip and stops the wait; no tip until the pointer has moved and rested again) and claim 26; S-0411 claim 20 (Buttons and blocks: over the slots the next movement shows it again at once)
        self.shown = None;
        self.rest = None;
        self.at_once = None;
    }

    /// The tooltip option is off, or the options menu is open: no tip
    /// shows, and one that shows hides.
    pub fn off(&mut self) {
        // spec: S-0111 claim 20 (with the option off no name tip ever shows; the options menu hides a tip and stops the wait) and claim 26 (turning it off hides a tip); S-0411 claim 20 (with the option off no sidebar tip ever shows; the options menu hides a sidebar tip and stops its wait as for the name tip)
        self.shown = None;
        self.rest = None;
        self.at_once = None;
    }

    /// Whether the wait has ended at `now` without a choice made yet.
    pub fn due(&self, now: i64) -> bool {
        // spec: S-0111 claim 20 (the wait: 1000 ms of real time at rest)
        self.rest.is_some_and(|s| now - s >= WAIT_MS)
    }

    /// One drawn frame at real time `now`, the pointer on a build-icon slot
    /// when `on_slot`. A tip whose lifetime is over hides. A tip that shows
    /// has its text chosen again: `choose(Some(tip))` gives the tip for the
    /// place of the pointer's last movement now; none, or `blocked`, hides
    /// it, a different text replaces it with the lifetime unchanged, the same
    /// text leaves it. A movement over the slots since the last frame shows
    /// its tip now, its lifetime from that movement. When the wait ends, the
    /// rest's one choice is made. In both, no tip while `blocked`, else
    /// `choose(None)` gives the tip, if any, with its lifetime from now.
    pub fn update(
        &mut self,
        now: i64,
        on_slot: bool,
        blocked: bool,
        choose: impl FnOnce(Option<&Shown>) -> Option<Shown>,
    ) {
        // spec: S-0411 claim 20 (Over the slots, no wait: once one drawn frame has passed with the pointer on a slot, every later movement that ends on a slot shows its tip at once)
        self.armed = on_slot;
        if let Some(t) = self.at_once.take() {
            self.shown = None;
            if blocked {
                // spec: S-0411 claim 20 (Buttons and blocks: the hidden mouse cursor, so an order block, blocks sidebar tips; the options menu hides them)
                return;
            }
            // spec: S-0411 claim 20 (Lifetime: the lifetime starts at the movement for a tip that a movement shows at once; every movement over the slots that shows a tip, the same tip included, starts it again)
            self.shown = choose(None).map(|mut s| {
                s.until = t + LIFETIME_MS;
                s
            });
            if self.shown.as_ref().is_some_and(|s| now >= s.until) {
                self.shown = None;
            }
            return;
        }
        if self.shown.as_ref().is_some_and(|s| now >= s.until) {
            // spec: S-0111 claim 20 (Lifetime: it hides after 10000 ms and does not show again until the pointer moves and rests again); S-0411 claim 20 (Lifetime: over the slots the next movement shows it again at once)
            self.shown = None;
        }
        if let Some(old) = self.shown.take() {
            // spec: S-0111 claim 26 (while it shows, its text is chosen again in every drawn frame; no text, an empty text or a blocked tip hides it and nothing shows until the pointer moves and rests again)
            if blocked {
                return;
            }
            match choose(Some(&old)) {
                None => {}
                // spec: S-0111 claim 26 (the same text: the tip stays as it is)
                Some(new) if new.text == old.text => self.shown = Some(old),
                // spec: S-0111 claims 20 and 26 (another text replaces the shown one, laid out and placed again; the lifetime does not start again)
                Some(mut new) => {
                    new.until = old.until;
                    self.shown = Some(new);
                }
            }
            return;
        }
        if !self.due(now) {
            return;
        }
        // spec: S-0111 claim 20 (One choice per rest: the text is chosen once, when the wait ends; nothing then shows until the pointer moves and rests again, even if the block ends)
        self.rest = None;
        if blocked {
            // spec: S-0111 claim 20 (Blocked: the wait that ends while the cursor is hidden, the input is locked, the view is moving, a left or right button pressed over the view is held, shows nothing)
            return;
        }
        self.shown = choose(None);
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

const SPACE: u16 = 0x20;

/// The lines of `units`: a carriage return, a line feed or a carriage
/// return followed by a line feed starts a new line, and a line wider than
/// the wrap width `wrap_w` (the view's width less [`WRAP_ALLOWANCE`]) is
/// wrapped at its last space.
pub fn lines(units: &[u16], glyph_width: &impl Fn(u16) -> i32, wrap_w: i32) -> Vec<Vec<u16>> {
    let mut out = vec![Vec::new()];
    // The last break point of the current line: a space with a unit before it.
    let mut space: Option<usize> = None;
    let mut i = 0;
    while i < units.len() {
        let c = units[i];
        if c == CR || c == LF {
            // spec: S-0111 claim 24 (a carriage return or a line feed starts a new line, a carriage return followed by a line feed counting as one; a line break forgets every earlier space)
            if c == CR && units.get(i + 1) == Some(&LF) {
                i += 1;
            }
            out.push(Vec::new());
            space = None;
        } else {
            let line = out.last_mut().expect("one line at least");
            // spec: S-0111 claim 24 (Wrapping: a unit that would make its line wider than the wrap width, and is not first on its line, breaks it)
            if !line.is_empty() && advance(line_width(line, glyph_width), c, glyph_width) > wrap_w {
                let mut next = Vec::new();
                if let Some(k) = space.take() {
                    // spec: S-0111 claim 24 (Wrapping: at the last break point; the space is dropped and the next line starts with the unit after it)
                    next = line.split_off(k + 1);
                    line.pop();
                }
                next.push(c);
                // spec: S-0111 claim 24 (Wrapping: a space is a break point when at least one unit comes before it on its line, also one that starts the new line after moved units)
                space = (c == SPACE && next.len() > 1).then(|| next.len() - 1);
                out.push(next);
            } else {
                // spec: S-0111 claim 24 (Wrapping: a space is a break point only when at least one unit comes before it on its line)
                if c == SPACE && !line.is_empty() {
                    space = Some(line.len());
                }
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
    // spec: S-0111 claim 24 (h_t is the number of lines times the font's line pitch; w = w_t + 8, h = h_t + 4)
    (wt + EXTRA_W, lines.len() as i32 * pitch + EXTRA_H)
}

/// The box's top-left corner for a `w` × `h` box under the pointer at
/// `(x, y)` in a view whose right and bottom edges (exclusive) are `x1`
/// and `y1`.
pub fn place((x, y): (i32, i32), (w, h): (i32, i32), (x1, y1): (i32, i32)) -> (i32, i32) {
    // spec: S-0111 claim 24 (r40: l = x, or X1 − w when x + w > X1; m = y + 16, or y − h + 1 when y + 16 + h > Y1, the last row on the pointer's row; no correction at the left or top edge)
    let l = if x + w > x1 { x1 - w } else { x };
    let m = if y + BELOW + h > y1 {
        y - h + 1
    } else {
        y + BELOW
    };
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

/// Draws tip `s`, cut at its clip rectangle: its box (`draw_box`), then its
/// lines in the game font, inset by [`INSET`] and one line pitch apart,
/// each glyph's first min(*H*, *P*) rows painted, inside the frame.
pub fn draw(d: &mut DrawList, font: &StatusFont, s: &Shown) {
    draw_box(d, s);
    let r = s.rect;
    // spec: S-0111 claim 25 (the text sits inside the frame); S-0411 claim 21 (laid out and drawn as S-0111 claims 24 and 25 give)
    let inside = s
        .clip
        .intersect(&Rect::new(r.x + 1, r.y + 1, r.w - 2, r.h - 2));
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
    // spec: S-0111 claim 24 (r40: lines wrap to the wrap width W − 8, W the view's width)
    let wrap_w = view.w - WRAP_ALLOWANCE;
    // spec: S-0111 claim 24 (the box is placed once, from the pointer's position when the tip shows; it does not move while it shows)
    let at = |size| place(p, size, (view.right(), view.bottom()));
    make(units, wrap_w, font, ink, now, view, at)
}

/// A tip for `units` (cut to the text limit) laid out in the game font
/// (`font`: glyph widths and line pitch) with wrap width `wrap_w`, its box
/// placed by `at` from its size, cut at the screen rectangle `clip`, shown
/// at `now`: `None` for an empty text.
pub fn make(
    units: &[u16],
    wrap_w: i32,
    font: (&impl Fn(u16) -> i32, i32),
    ink: [u8; 3],
    now: i64,
    clip: Rect,
    at: impl FnOnce((i32, i32)) -> (i32, i32),
) -> Option<Shown> {
    let units = cut(units);
    if units.is_empty() {
        // spec: S-0111 claim 20 (if there is no text, or it is empty, nothing shows); S-0411 claim 20 (an empty slot has no text, and no tip shows there)
        return None;
    }
    let (glyph_width, pitch) = font;
    let lines = lines(units, glyph_width, wrap_w);
    let (w, h) = size(&lines, glyph_width, pitch);
    let (l, m) = at((w, h));
    Some(Shown {
        text: units.to_vec(),
        lines,
        rect: Rect::new(l, m, w, h),
        // spec: S-0111 claim 20 (Lifetime: 10000 ms after it shows); S-0411 claim 20 (Lifetime: at the end of the wait for a tip that shows after a rest)
        until: now + LIFETIME_MS,
        ink,
        clip,
    })
}

/// Draws the box of `s`: opaque black, framed by a 1-pixel line in the tip
/// colour, cut at the tip's clip rectangle. The text is the caller's (the
/// game font's sprites in the tip colour).
pub fn draw_box(d: &mut DrawList, s: &Shown) {
    let r = s.rect;
    let [cr, cg, cb] = s.ink;
    // spec: S-0111 claim 25 (filled with opaque black and framed by a line one pixel wide along its four edges in the tip colour; drawn as it is, with no shade table); S-0411 claim 21 (drawn over the sidebar and cut at its edges: it never covers the battlefield view)
    let clip = |x: Rect| s.clip.intersect(&x);
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

    /// A movement that ends off the build-icon slots.
    fn off_slots(t: &mut NameTip, now: i64) {
        t.moved(now, false);
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
        t.update(5000, false, false, |_| panic!("no rest yet"));
        off_slots(&mut t, 1000);
        t.update(1999, false, false, |_| panic!("the wait runs"));
        assert!(t.shown.is_none());
        off_slots(&mut t, 1500);
        t.update(2400, false, false, |_| panic!("the wait started again"));
        let mut asked = 0;
        t.update(2500, false, false, |_| {
            asked += 1;
            shown(2500)
        });
        assert_eq!(asked, 1);
        assert_eq!(t.shown.as_ref().unwrap().until, 12_500);
        // While it shows, the text is chosen again (claim 26): the same text stays.
        t.update(6000, false, false, |old| {
            assert!(old.is_some(), "chosen again while it shows");
            shown(6000)
        });
        assert!(t.shown.is_some());
        t.update(12_499, false, false, |_| shown(12_499));
        assert_eq!(t.shown.as_ref().unwrap().until, 12_500);
        t.update(12_500, false, false, |_| {
            panic!("no second tip in the same rest")
        });
        assert!(t.shown.is_none(), "the lifetime ended");
        // An empty choice shows nothing, and nothing until the pointer moves.
        off_slots(&mut t, 20_000);
        t.update(21_000, false, false, |_| None);
        t.update(22_000, false, false, |_| panic!("one choice per rest"));
        assert!(t.shown.is_none());
    }

    /// spec: S-0111 claims 20 and 26 (a press or release hides the tip and stops the wait until
    /// the pointer moves; a movement hides it; turning the option off hides it; while blocked when
    /// the wait ends, nothing shows)
    #[test]
    fn claims_20_and_26_what_hides_it() {
        let mut t = NameTip::default();
        off_slots(&mut t, 0);
        t.update(1000, false, false, |_| shown(1000));
        assert!(t.shown.is_some());
        t.button();
        assert!(t.shown.is_none());
        t.update(5000, false, false, |_| panic!("the wait stopped"));
        off_slots(&mut t, 5000);
        t.button();
        t.update(7000, false, false, |_| panic!("the wait stopped"));
        off_slots(&mut t, 7000);
        t.update(8000, false, false, |_| shown(8000));
        off_slots(&mut t, 8100);
        assert!(t.shown.is_none(), "a movement hides it");
        t.update(9100, false, true, |_| panic!("blocked: no choice"));
        assert!(t.shown.is_none());
        t.update(9200, false, false, |_| panic!("the rest's choice is spent"));
        off_slots(&mut t, 10_000);
        t.update(11_000, false, false, |_| shown(11_000));
        t.off();
        assert!(t.shown.is_none());
        t.update(13_000, false, false, |_| panic!("off: no wait"));
    }

    /// spec: S-0111 claim 26 (a text chosen again while it shows: no text or blocked hides it with
    /// nothing until the pointer moves and rests again; another text replaces it, the lifetime
    /// unchanged; the same text leaves it) and claim 20 (the tip is blocked: a wait that ends
    /// blocked shows nothing, and the block ending brings nothing back)
    #[test]
    fn claim_26_chosen_again() {
        let other = |now: i64| {
            tip(
                &u("abc"),
                (10, 10),
                Rect::new(0, 0, 640, 400),
                (&gw, 12),
                INK_SIDE_0,
                now,
            )
        };
        let mut t = NameTip::default();
        off_slots(&mut t, 0);
        t.update(1000, false, false, |_| shown(1000));
        let first = t.shown.clone().unwrap();
        // Another text: replaced, laid out again, the lifetime not started again.
        t.update(3000, false, false, |_| other(3000));
        let s = t.shown.as_ref().unwrap();
        assert_eq!(s.lines, vec![u("abc")]);
        assert_eq!(s.until, first.until, "the lifetime does not start again");
        assert_ne!(s.rect, first.rect);
        // The same text: as it is.
        let before = t.shown.clone();
        t.update(4000, false, false, |_| other(4000));
        assert_eq!(t.shown, before);
        // No text hides it, and nothing shows again until a movement and a rest.
        t.update(5000, false, false, |_| None);
        assert!(t.shown.is_none());
        t.update(6000, false, false, |_| panic!("one choice per rest"));
        t.update(20_000, false, false, |_| panic!("one choice per rest"));
        // Blocked while it shows: hides.
        off_slots(&mut t, 30_000);
        t.update(31_000, false, false, |_| shown(31_000));
        t.update(31_100, false, true, |_| panic!("blocked: no choice"));
        assert!(t.shown.is_none());
        t.update(31_200, false, false, |_| {
            panic!("the block ending brings nothing back")
        });
        assert!(t.shown.is_none());
        // A wait that ends blocked uses up the rest's choice.
        off_slots(&mut t, 40_000);
        t.update(41_000, false, true, |_| panic!("blocked"));
        t.update(42_000, false, false, |_| panic!("the choice is spent"));
        assert!(t.shown.is_none());
    }

    /// spec: S-0111 claim 24 (width: glyph widths plus 1 per unit, a tab to the next multiple of 64;
    /// CR, LF and CRLF each start one new line; w = w_t + 8, h = lines × P_f + 4; Wrapping: at the
    /// last space before the unit that passes the wrap width, the space dropped, a space first on
    /// its line no break point, a line break forgetting earlier spaces, a first unit always kept)
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
        assert_eq!(size(&l, &gw, 12), (10 + 8, 4 * 12 + 4));
        assert_eq!(lines(&u("aaaa"), &gw, 12), vec![u("aa"), u("aa")]);
        assert_eq!(lines(&u("W"), &gw, 3), vec![u("W")], "one unit at least");
        // Three equal words, 5 pixels a unit: "aaa bbb " is 40 wide, and the third word's first
        // unit passes 44. The break is at the last space, which is dropped.
        let three = u("aaa bbb ccc");
        assert_eq!(
            lines(&three, &gw, 44),
            vec![u("aaa bbb"), u("ccc")],
            "the last space is the break, dropped"
        );
        assert_eq!(lines(&three, &gw, 24), vec![u("aaa"), u("bbb"), u("ccc")]);
        // The overflowing unit is a space after moved units: a break point of the new line.
        assert_eq!(
            lines(&u("aaa bbb ccccc"), &gw, 39),
            vec![u("aaa"), u("bbb"), u("ccccc")],
            "bbb is moved down, then the space after it breaks"
        );
        // A space first on its line is no break point: the line keeps it.
        assert_eq!(
            lines(&u(" aaaaa"), &gw, 14),
            vec![u(" a"), u("aa"), u("aa")]
        );
        // A line break forgets every earlier space.
        assert_eq!(
            lines(&u("a b\naaaaa"), &gw, 24),
            vec![u("a b"), u("aaaa"), u("a")]
        );
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
            (100, 346),
            "last row on the pointer's row y (r40): 365 + 20 − 1 = y"
        );
        assert_eq!(
            place((10, 5), (700, 500), edges),
            (-60, -494),
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
        draw_box(&mut d, &s);
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
    /// the tip colour inside the frame, over the black fill, from the origin 4 right and 2 below
    /// the box's corner; one line pitch per line) and S-0014 claim 5 (one bits paint the
    /// foreground)
    #[test]
    fn claims_24_and_25_the_drawn_tip() {
        let font = StatusFont::new(glasnost_fmt::fnt::synthetic_font()).unwrap();
        let view = Rect::new(0, 0, 120, 80);
        let width = |c: u16| font.width(c);
        let pitch = font.line_pitch() as i32;
        let s = tip(&u("A\nA"), (10, 10), view, (&width, pitch), INK_OTHER, 0).unwrap();
        assert_eq!(s.rect, Rect::new(10, 26, 5 + 8, 2 * pitch + 4));
        let mut painter = Painter::new();
        let mut rows = [None; 2];
        register(&mut painter, &font, &s, &mut rows);
        assert!(rows[1].is_some() && rows[0].is_none());
        let mut d = DrawList::new();
        draw(&mut d, &font, &s);
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

    /// spec: S-0411 claim 20 (Over the slots, no wait: the movement onto the slots is an ordinary
    /// one; once one drawn frame has passed with the pointer on a slot, every later movement that
    /// ends on a slot shows its tip at once; Lifetime: from the movement, started again by every
    /// such movement, the same tip included; a movement off the slots is an ordinary one) and
    /// S-0111 claim 20 (the one exception over the build-icon slots)
    #[test]
    fn s0411_claim_20_over_the_slots_no_wait() {
        let mut t = NameTip::default();
        // The movement onto the slots: the ordinary wait.
        t.moved(1000, true);
        t.update(1016, true, false, |_| panic!("the wait runs"));
        // A drawn frame has passed on a slot: the next movement shows at once.
        t.moved(1020, true);
        assert!(t.shown.is_none(), "shown at the next drawn frame");
        let mut asked = 0;
        t.update(1032, true, false, |old| {
            assert!(old.is_none(), "a new tip, not a text chosen again");
            asked += 1;
            shown(1032)
        });
        assert_eq!(asked, 1);
        assert_eq!(
            t.shown.as_ref().unwrap().until,
            1020 + 10_000,
            "from the movement"
        );
        // The same tip again, by a movement: the lifetime starts again.
        t.moved(5000, true);
        t.update(5016, true, false, |_| shown(5016));
        assert_eq!(t.shown.as_ref().unwrap().until, 15_000);
        // A text chosen again does not start it again.
        t.update(9000, true, false, |_| shown(9000));
        assert_eq!(t.shown.as_ref().unwrap().until, 15_000);
        // The lifetime ends; the next movement there shows it again at once.
        t.update(15_000, true, false, |_| shown(15_000));
        assert!(t.shown.is_none());
        t.update(16_000, true, false, |_| panic!("no wait runs"));
        t.moved(16_100, true);
        t.update(16_116, true, false, |_| shown(16_116));
        assert_eq!(t.shown.as_ref().unwrap().until, 26_100);
        // A movement onto an empty slot: no text, hidden; still on the slots.
        t.moved(17_000, true);
        t.update(17_016, true, false, |_| None);
        assert!(t.shown.is_none());
        t.update(19_000, true, false, |_| {
            panic!("no wait over an empty slot")
        });
        t.moved(19_100, true);
        t.update(19_116, true, false, |_| shown(19_116));
        assert!(t.shown.is_some());
        // A movement off the slots is an ordinary one, and so is the next one back on.
        t.moved(20_000, false);
        assert!(t.shown.is_none());
        t.update(20_016, false, false, |_| panic!("the wait runs"));
        t.moved(20_100, true);
        t.update(20_116, true, false, |_| panic!("the wait runs again"));
        t.update(21_100, true, false, |_| shown(21_100));
        assert_eq!(
            t.shown.as_ref().unwrap().until,
            31_100,
            "from the end of the wait"
        );
    }

    /// spec: S-0411 claim 20 (no drawn frame between the movement onto the slots and the next
    /// one: that one is ordinary too; the one drawn frame that must pass first)
    #[test]
    fn s0411_claim_20_one_drawn_frame_first() {
        let mut t = NameTip::default();
        t.moved(0, true);
        t.moved(5, true);
        t.update(16, true, false, |_| panic!("the wait runs"));
        t.update(1004, true, false, |_| panic!("the wait started at 5"));
        t.update(1005, true, false, |_| shown(1005));
        assert!(t.shown.is_some());
    }

    /// spec: S-0411 claim 20 (Buttons and blocks: a press or release hides the tip, and over the
    /// slots the next movement shows it again at once, also while a button is held; the hidden
    /// cursor, so an order block, blocks sidebar tips; the options menu hides it and stops the
    /// wait)
    #[test]
    fn s0411_claim_20_buttons_and_blocks() {
        let mut t = NameTip::default();
        t.moved(0, true);
        t.update(16, true, false, |_| panic!("the wait runs"));
        t.moved(20, true);
        t.update(32, true, false, |_| shown(32));
        // A press hides it; the next movement over the slots shows it again at once.
        t.button();
        assert!(t.shown.is_none());
        t.update(48, true, false, |_| {
            panic!("hidden until the pointer moves")
        });
        t.moved(60, true);
        t.update(64, true, false, |_| shown(64));
        assert!(t.shown.is_some(), "a held button does not stop it");
        // Blocked (the hidden cursor): the movement shows nothing.
        t.moved(100, true);
        t.update(116, true, true, |_| panic!("blocked: no choice"));
        assert!(t.shown.is_none());
        t.update(132, true, false, |_| {
            panic!("nothing until the pointer moves")
        });
        t.moved(140, true);
        t.update(148, true, false, |_| shown(148));
        assert!(t.shown.is_some());
        // Blocked while it shows: hidden.
        t.update(160, true, true, |_| panic!("blocked: no choice"));
        assert!(t.shown.is_none());
        // The options menu (or the option off): hidden, no movement pending.
        t.moved(200, true);
        t.off();
        t.update(216, true, false, |_| panic!("off: nothing pending"));
        assert!(t.shown.is_none());
    }

    /// spec: S-0411 claim 21 (laid out and drawn as S-0111 claims 24 and 25 give; drawn over the
    /// sidebar and cut at its edges: it never covers the battlefield view)
    #[test]
    fn s0411_claim_21_cut_at_the_sidebar() {
        let side = Rect::new(100, 0, 60, 80);
        let s = make(&u("aaaa"), 52, (&gw, 12), INK_SIDE_0, 0, side, |_| (90, 10)).unwrap();
        assert_eq!(s.rect, Rect::new(90, 10, 20 + 8, 12 + 4));
        let mut d = DrawList::new();
        draw_box(&mut d, &s);
        let fills: Vec<Rect> = d
            .prims
            .iter()
            .filter_map(|p| match p {
                glasnost_ui::draw::Prim::Fill { rect, .. } => Some(*rect),
                _ => None,
            })
            .collect();
        assert!(
            fills.iter().all(|r| r.is_empty() || r.x >= 100),
            "{fills:?}"
        );
        assert_eq!(fills[0], Rect::new(100, 10, 18, 16));
    }
}
