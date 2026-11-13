//! Drawing helpers for the interface's widgets: panels, buttons and choice
//! boxes (our own look). Hit-testing is done by the screens, which keep the
//! rectangle of every target they draw.

use crate::draw::{DrawList, colour};
use crate::font;
use crate::geom::{Point, Rect};

/// Widgets laid out by a screen: a rectangle per target, topmost last, and
/// the keyboard's focus stops in focus order.
#[derive(Clone, Debug)]
pub struct Targets<T> {
    /// The rectangles and their targets.
    pub list: Vec<(Rect, T)>,
    /// The focus stops, in the order the keyboard moves through them.
    pub stops: Vec<Stop<T>>,
}

/// A keyboard focus stop: one widget.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Stop<T> {
    /// The widget's rectangle (where the focus mark goes).
    pub rect: Rect,
    /// What activating it does.
    pub main: T,
    /// A choice box's previous and next steps (Left and Right).
    pub sides: Option<(T, T)>,
}

impl<T: Clone> Targets<T> {
    /// No targets.
    pub fn new() -> Self {
        Self {
            list: Vec::new(),
            stops: Vec::new(),
        }
    }

    /// Adds a target that is also a focus stop.
    pub fn push(&mut self, rect: Rect, t: T) {
        self.list.push((rect, t.clone()));
        self.stops.push(Stop {
            rect,
            main: t,
            sides: None,
        });
    }

    /// Adds a target the pointer reaches but the keyboard focus skips.
    pub fn push_hit(&mut self, rect: Rect, t: T) {
        self.list.push((rect, t));
    }

    /// Adds a choice box's two halves ([`choice`]) as one focus stop:
    /// activating it steps to the next value, Left and Right step either
    /// way.
    pub fn push_choice(&mut self, prev_rect: Rect, prev: T, next_rect: Rect, next: T) {
        self.list.push((prev_rect, prev.clone()));
        self.list.push((next_rect, next.clone()));
        let x = prev_rect.x.min(next_rect.x);
        let y = prev_rect.y.min(next_rect.y);
        let r = prev_rect.right().max(next_rect.right());
        let b = prev_rect.bottom().max(next_rect.bottom());
        self.stops.push(Stop {
            rect: Rect::new(x, y, r - x, b - y),
            main: next.clone(),
            sides: Some((prev, next)),
        });
    }

    /// The topmost target under `p`.
    pub fn hit(&self, p: Point) -> Option<T> {
        self.list
            .iter()
            .rev()
            .find(|(r, _)| r.contains(p))
            .map(|(_, t)| t.clone())
    }

    /// The rectangle of the first target equal to `t`.
    pub fn rect_of(&self, t: &T) -> Option<Rect>
    where
        T: PartialEq,
    {
        self.list.iter().find(|(_, x)| x == t).map(|(r, _)| *r)
    }
}

impl<T: Clone> Default for Targets<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// A keyboard navigation step (the client's keys choose them).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Nav {
    /// The next stop.
    Next,
    /// The previous stop.
    Prev,
    /// A choice's previous value; elsewhere the previous stop.
    Left,
    /// A choice's next value; elsewhere the next stop.
    Right,
    /// Activate the focused stop.
    Activate,
}

/// Where the keyboard focus is on a screen: the focused stop's target,
/// kept across frames so that it stays on its widget while the screen
/// changes around it. The mark shows only after a keyboard step.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Focus<T> {
    /// The focused stop's main target.
    pub at: Option<T>,
    /// Whether the mark is drawn (after a key; a pointer press hides it).
    pub shown: bool,
}

impl<T> Default for Focus<T> {
    fn default() -> Self {
        Self {
            at: None,
            shown: false,
        }
    }
}

impl<T: Clone + PartialEq> Focus<T> {
    /// No focus.
    pub fn new() -> Self {
        Self::default()
    }

    /// The focused stop's place in `t`, when it is still there.
    fn index(&self, t: &Targets<T>) -> Option<usize> {
        let at = self.at.as_ref()?;
        t.stops.iter().position(|s| &s.main == at)
    }

    /// Takes a step on the screen whose targets are `t`. Returns the target
    /// to act on, when the step activates one: [`Nav::Activate`] on the
    /// focused stop, or Left and Right on a choice. The first step with
    /// nothing focused only focuses a stop.
    pub fn nav(&mut self, t: &Targets<T>, n: Nav) -> Option<T> {
        let len = t.stops.len();
        if len == 0 {
            self.at = None;
            return None;
        }
        self.shown = true;
        let i = self.index(t);
        let step = |d: usize| match i {
            Some(i) => (i + d) % len,
            None if d == 1 => 0,
            None => len - 1,
        };
        let to = match (n, i) {
            (Nav::Activate, Some(i)) => return Some(t.stops[i].main.clone()),
            (Nav::Left | Nav::Right, Some(i)) if t.stops[i].sides.is_some() => {
                let (a, b) = t.stops[i].sides.clone().expect("checked");
                return Some(if n == Nav::Left { a } else { b });
            }
            (Nav::Next | Nav::Right | Nav::Activate, _) => step(1),
            (Nav::Prev | Nav::Left, _) => step(len - 1),
        };
        self.at = Some(t.stops[to].main.clone());
        None
    }

    /// Focuses the stop whose main target is `x`, if `t` has one.
    pub fn set(&mut self, t: &Targets<T>, x: &T) -> bool {
        if t.stops.iter().any(|s| &s.main == x) {
            self.at = Some(x.clone());
            self.shown = true;
            true
        } else {
            false
        }
    }

    /// The focused stop's main target, while it is on the screen.
    pub fn target(&self, t: &Targets<T>) -> Option<T> {
        self.index(t).map(|i| t.stops[i].main.clone())
    }

    /// The rectangle the mark goes around, while shown.
    pub fn rect(&self, t: &Targets<T>) -> Option<Rect> {
        self.index(t)
            .filter(|_| self.shown)
            .map(|i| t.stops[i].rect)
    }

    /// Draws the focus mark: a two-pixel outline just outside the focused
    /// widget, in the focus colour (our own look, unlike the amber hover).
    pub fn mark(&self, d: &mut DrawList, t: &Targets<T>) {
        if let Some(r) = self.rect(t) {
            d.frame(r.inset(-2), colour::FOCUS);
            d.frame(r.inset(-3), colour::FOCUS);
        }
    }
}

/// A panel with an optional title strip.
pub fn panel(d: &mut DrawList, r: Rect, title: Option<&str>) {
    d.fill(r, colour::PANEL);
    d.frame(r, colour::EDGE);
    if let Some(t) = title {
        let strip = Rect::new(r.x + 1, r.y + 1, r.w - 2, 15);
        d.fill(strip, colour::RAISED);
        d.fill(Rect::new(r.x + 1, r.y + 16, r.w - 2, 1), colour::ACCENT);
        d.text_left(strip, t, colour::ACCENT);
    }
}

/// A push button. `hot` when hovered or active; disabled buttons are dim.
pub fn button(d: &mut DrawList, r: Rect, label: &str, hot: bool, enabled: bool) {
    d.fill(
        r,
        if hot && enabled {
            colour::HOT
        } else {
            colour::RAISED
        },
    );
    d.frame(
        r,
        if hot && enabled {
            colour::ACCENT
        } else {
            colour::EDGE
        },
    );
    // A notch in the top-left corner: part of our own look.
    d.fill(Rect::new(r.x, r.y, 3, 1), colour::ACCENT);
    d.fill(Rect::new(r.x, r.y, 1, 3), colour::ACCENT);
    d.text_centred(r, label, if enabled { colour::TEXT } else { colour::DIM });
}

/// A choice box: a label on the left and the current value between arrow
/// marks on the right. Returns the rectangles of the "previous" and "next"
/// halves of the value box.
pub fn choice(d: &mut DrawList, r: Rect, label: &str, value: &str, enabled: bool) -> (Rect, Rect) {
    // Without a label (a table cell) the value takes the whole width.
    // design: readable text and display scale (lead's request 2026-11-10)
    let split = if label.is_empty() { 0 } else { r.w * 2 / 5 };
    let label_r = Rect::new(r.x, r.y, split, r.h);
    let value_r = Rect::new(r.x + split, r.y, r.w - split, r.h);
    d.text_left(label_r, label, colour::DIM);
    d.fill(value_r, colour::RAISED);
    d.frame(value_r, colour::EDGE);
    let ink = if enabled { colour::TEXT } else { colour::DIM };
    d.text_left(value_r, "<", colour::ACCENT);
    let arrow = Rect::new(value_r.right() - 9, value_r.y, 9, value_r.h);
    d.text_left(arrow, ">", colour::ACCENT);
    d.text_centred(
        Rect::new(value_r.x + 8, value_r.y, value_r.w - 16, value_r.h),
        value,
        ink,
    );
    let half = value_r.w / 2;
    (
        Rect::new(value_r.x, value_r.y, half, value_r.h),
        Rect::new(value_r.x + half, value_r.y, value_r.w - half, value_r.h),
    )
}

/// A one-line text field: a label on the left, the text in a box on the
/// right, with a caret when `focused`. Returns the box. Text too wide for
/// the box shows its end.
pub fn text_field(
    d: &mut DrawList,
    r: Rect,
    label: &str,
    value: &str,
    focused: bool,
    hot: bool,
) -> Rect {
    let split = r.w * 2 / 5;
    let label_r = Rect::new(r.x, r.y, split, r.h);
    let box_r = Rect::new(r.x + split, r.y, r.w - split, r.h);
    d.text_left(label_r, label, colour::DIM);
    d.fill(box_r, if focused { colour::HOT } else { colour::RAISED });
    d.frame(
        box_r,
        if focused || hot {
            colour::ACCENT
        } else {
            colour::EDGE
        },
    );
    let shown = tail_fit(value, box_r.w - 12, focused);
    d.text_left(
        Rect::new(box_r.x + 3, box_r.y, box_r.w - 6, box_r.h),
        &shown,
        colour::TEXT,
    );
    box_r
}

/// A one-line text field without a label: the text in the box `r`, or
/// `placeholder` dimmed while the text is empty and the field not focused,
/// with a caret when `focused`. Returns the box (design: map picker
/// (lead's request 2026-11-13)).
pub fn search_field(
    d: &mut DrawList,
    r: Rect,
    value: &str,
    placeholder: &str,
    focused: bool,
    hot: bool,
) -> Rect {
    d.fill(r, if focused { colour::HOT } else { colour::RAISED });
    d.frame(
        r,
        if focused || hot {
            colour::ACCENT
        } else {
            colour::EDGE
        },
    );
    let inner = Rect::new(r.x + 3, r.y, r.w - 6, r.h);
    if value.is_empty() && !focused {
        d.text_left(inner, &font::fit(placeholder, r.w - 12), colour::DIM);
    } else {
        d.text_left(inner, &tail_fit(value, r.w - 12, focused), colour::TEXT);
    }
    r
}

/// The end of `s` that fits in `width` pixels, with a caret when `caret`.
pub fn tail_fit(s: &str, width: i32, caret: bool) -> String {
    let caret = if caret { "_" } else { "" };
    let chars: Vec<char> = s.chars().collect();
    for start in 0..=chars.len() {
        let t: String = chars[start..].iter().collect::<String>() + caret;
        if font::text_width(&t) <= width {
            return t;
        }
    }
    caret.to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn topmost_target_wins() {
        let mut t = Targets::new();
        t.push(Rect::new(0, 0, 100, 100), 1);
        t.push(Rect::new(10, 10, 10, 10), 2);
        assert_eq!(t.hit(Point::new(15, 15)), Some(2));
        assert_eq!(t.hit(Point::new(50, 50)), Some(1));
        assert_eq!(t.hit(Point::new(150, 50)), None);
        assert_eq!(t.rect_of(&2), Some(Rect::new(10, 10, 10, 10)));
    }

    #[test]
    fn the_focus_moves_through_the_stops_and_activates_them() {
        let mut t = Targets::new();
        t.push_hit(Rect::new(0, 0, 500, 500), 0);
        t.push(Rect::new(10, 10, 10, 10), 1);
        t.push_choice(Rect::new(10, 30, 5, 10), 2, Rect::new(15, 30, 5, 10), 3);
        t.push(Rect::new(10, 50, 10, 10), 4);
        // The pointer-only target is no stop; the choice is one stop.
        assert_eq!(t.stops.len(), 3);
        assert_eq!(t.stops[1].rect, Rect::new(10, 30, 10, 10));
        let mut f = Focus::new();
        assert_eq!(f.rect(&t), None);
        // The first activation only focuses.
        assert_eq!(f.nav(&t, Nav::Activate), None);
        assert_eq!(f.at, Some(1));
        assert_eq!(f.nav(&t, Nav::Activate), Some(1));
        assert_eq!(f.nav(&t, Nav::Next), None);
        assert_eq!(f.rect(&t), Some(Rect::new(10, 30, 10, 10)));
        assert_eq!(f.nav(&t, Nav::Left), Some(2));
        assert_eq!(f.nav(&t, Nav::Right), Some(3));
        assert_eq!(f.nav(&t, Nav::Activate), Some(3));
        // Left and Right elsewhere move, and the order wraps.
        f.nav(&t, Nav::Next);
        assert_eq!(f.at, Some(4));
        f.nav(&t, Nav::Right);
        assert_eq!(f.at, Some(1));
        f.nav(&t, Nav::Prev);
        assert_eq!(f.at, Some(4));
        // A focused target that left the screen starts the order again.
        let mut fewer = Targets::new();
        fewer.push(Rect::new(0, 0, 4, 4), 7);
        assert_eq!(f.target(&fewer), None);
        f.nav(&fewer, Nav::Next);
        assert_eq!(f.target(&fewer), Some(7));
        let mut d = DrawList::new();
        f.mark(&mut d, &fewer);
        assert_eq!(d.prims.len(), 2);
    }

    #[test]
    fn choice_halves_cover_the_value_box() {
        let mut d = DrawList::new();
        let (a, b) = choice(&mut d, Rect::new(0, 0, 200, 20), "Credits", "10000", true);
        assert_eq!(a.right(), b.x);
        assert_eq!(b.right(), 200);
        assert_eq!(a.x, 80);
        assert!(d.texts().contains(&"10000"));
    }

    /// A table cell's choice box has no label and uses its whole width, so
    /// a short column shows its value whole (design: readable text and
    /// display scale, lead's request 2026-11-10).
    #[test]
    fn a_choice_without_a_label_uses_its_whole_width() {
        let mut d = DrawList::new();
        let (a, b) = choice(&mut d, Rect::new(10, 0, 70, 16), "", "Closed", true);
        assert_eq!((a.x, b.right()), (10, 80));
        assert!(d.texts().contains(&"Closed"), "{:?}", d.texts());
        for value in ["Random", "Team 4", "Start 8", "Slot 8"] {
            let mut d = DrawList::new();
            choice(&mut d, Rect::new(0, 0, 70, 16), "", value, true);
            assert!(d.texts().contains(&value), "{value}: {:?}", d.texts());
        }
    }

    #[test]
    fn a_text_field_shows_the_end_of_long_text_and_a_caret() {
        let mut d = DrawList::new();
        let b = text_field(&mut d, Rect::new(0, 0, 200, 16), "Name", "abc", true, false);
        assert_eq!(b, Rect::new(80, 0, 120, 16));
        assert!(d.texts().contains(&"abc_"));
        let long = "x".repeat(200);
        let t = tail_fit(&long, 50, false);
        assert!(font::text_width(&t) <= 50 && t.len() > 3);
        assert!(long.ends_with(&t));
    }
}
