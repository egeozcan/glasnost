//! Drawing helpers for the interface's widgets: panels, buttons and choice
//! boxes (our own look). Hit-testing is done by the screens, which keep the
//! rectangle of every target they draw.

use crate::draw::{DrawList, colour};
use crate::font;
use crate::geom::{Point, Rect};

/// Widgets laid out by a screen: a rectangle per target, topmost last.
#[derive(Clone, Debug)]
pub struct Targets<T> {
    /// The rectangles and their targets.
    pub list: Vec<(Rect, T)>,
}

impl<T: Clone> Targets<T> {
    /// No targets.
    pub fn new() -> Self {
        Self { list: Vec::new() }
    }

    /// Adds a target.
    pub fn push(&mut self, rect: Rect, t: T) {
        self.list.push((rect, t));
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
    let split = r.w * 2 / 5;
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
    fn choice_halves_cover_the_value_box() {
        let mut d = DrawList::new();
        let (a, b) = choice(&mut d, Rect::new(0, 0, 200, 20), "Credits", "10000", true);
        assert_eq!(a.right(), b.x);
        assert_eq!(b.right(), 200);
        assert_eq!(a.x, 80);
        assert!(d.texts().contains(&"10000"));
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
