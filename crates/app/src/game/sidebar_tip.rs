//! The sidebar's tips (S-0411 claims 20 to 22): the places of the sidebar
//! that show one, their texts and their boxes' places. Their timing, layout
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
const S: u16 = b's' as u16;
const D: u16 = b'd' as u16;
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

/// An entry's text with its insertion points filled: each text point
/// (`%s`) takes the next of `texts` and each number point (`%d`) the next
/// of `numbers`, in the order they stand, a number in decimal with a minus
/// sign when negative and no digit grouping.
///
/// design: Q-2831 (an entry whose insertion points differ from Import
/// mapping's, S-0411 Open question 11) interim: a point with no value left
/// writes nothing, values left over are not written, and a percent sign
/// followed by anything else is kept as it stands.
pub fn fill(entry: &[u16], texts: &[&[u16]], numbers: &[i64]) -> Vec<u16> {
    let mut out = Vec::with_capacity(entry.len());
    let (mut ts, mut ns) = (texts.iter(), numbers.iter());
    let mut i = 0;
    while i < entry.len() {
        let c = entry[i];
        match (c, entry.get(i + 1)) {
            // spec: S-0411 Terms (Insertion point: a percent sign followed by s is a text point, filled in the order they stand)
            (PERCENT, Some(&S)) => {
                out.extend_from_slice(ts.next().copied().unwrap_or_default());
                i += 2;
            }
            // spec: S-0411 Terms (Insertion point: a percent sign followed by d is a number point; decimal, a leading minus sign when negative, no digit grouping)
            (PERCENT, Some(&D)) => {
                if let Some(n) = ns.next() {
                    out.extend(n.to_string().encode_utf16());
                }
                i += 2;
            }
            _ => {
                out.push(c);
                i += 1;
            }
        }
    }
    out
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
        Looked::Text(e) => spaces_to_lines(&fill(&e, &[name], &[i64::from(price)])),
        // spec: S-0411 Import mapping (TXT_MONEY_FORMAT_2: a missing entry shows S-0004 claim 17's placeholder alone, with no name and no price)
        Looked::Missing(p) => p,
    }
}

/// The text of the power meter's tip: the power entry `entry` filled with
/// output `o` and drain `d`, spaces kept.
pub fn power_text(entry: Looked, (o, d): (i32, i32)) -> Vec<u16> {
    match entry {
        // spec: S-0411 claim 22 (power meter: the power entry's text with its first number point filled by the house output O and its second by its drain D; the spaces kept)
        Looked::Text(e) => fill(&e, &[], &[i64::from(o), i64::from(d)]),
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
    /// order they stand; decimal, a leading minus sign, no grouping) and claim 20 (Text) and
    /// claim 22 (power meter); design: Q-2831 interim (other counts and kinds)
    #[test]
    fn terms_insertion_points() {
        assert_eq!(
            fill(&u("<%s|%d>"), &[&u("Tank X")], &[1500]),
            u("<Tank X|1500>")
        );
        assert_eq!(fill(&u("%d/%d"), &[], &[-25, 1234567]), u("-25/1234567"));
        // Q-2831 interim: a point with no value writes nothing; other percent signs stay.
        assert_eq!(fill(&u("a%sb%dc"), &[], &[]), u("abc"));
        assert_eq!(fill(&u("%d%%x%"), &[], &[7]), u("7%%x%"));
        assert_eq!(fill(&u("%d %s"), &[&u("n")], &[1]), u("1 n"));
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
