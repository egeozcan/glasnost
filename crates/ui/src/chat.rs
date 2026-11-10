//! Chat in a network room and in a network game (our own layout): the
//! last lines, newest at the bottom, and the line being typed. Chat is the
//! client's and the relay's; it never reaches a simulation.

use crate::draw::{DrawList, colour};
use crate::font;
use crate::geom::Rect;
use crate::widget;

/// One line of the chat log.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Line {
    /// Who said it (empty for a notice of the client's own).
    pub who: String,
    /// What was said.
    pub text: String,
    /// A notice (someone left, a refusal) rather than a person's line.
    pub notice: bool,
}

impl Line {
    /// The line as shown.
    pub fn shown(&self) -> String {
        if self.who.is_empty() {
            self.text.clone()
        } else {
            format!("{}: {}", self.who, self.text)
        }
    }
}

/// Rows of the chat box (our own layout).
// nocite: our own layout
const ROW: i32 = 12;

/// A chat box in `r`: the latest lines that fit above an input field.
/// Returns the input field, the target that focuses it.
pub fn view(d: &mut DrawList, r: Rect, lines: &[Line], input: &str, focused: bool) -> Rect {
    widget::panel(d, r, Some("CHAT"));
    let input_r = Rect::new(r.x + 4, r.bottom() - 20, r.w - 8, 16);
    let field = widget::text_field(d, input_r, "Say", input, focused, false);
    let top = r.y + 19;
    let rows = ((input_r.y - 2 - top) / ROW).max(0) as usize;
    let first = lines.len().saturating_sub(rows);
    for (k, l) in lines[first..].iter().enumerate() {
        let y = top + k as i32 * ROW;
        let text = font::fit(&l.shown(), r.w - 12);
        d.text_left(
            Rect::new(r.x + 6, y, r.w - 12, ROW),
            &text,
            if l.notice {
                colour::ACCENT
            } else {
                colour::TEXT
            },
        );
    }
    field
}

/// Chat over the battlefield in `r`: the given lines, newest at the
/// bottom, on a dark band each, and the line being typed when `input` is
/// some.
pub fn overlay(d: &mut DrawList, r: Rect, lines: &[Line], input: Option<&str>) {
    let mut y = r.bottom() - ROW;
    if let Some(text) = input {
        let shown = widget::tail_fit(&format!("Say: {text}"), r.w - 8, true);
        let w = font::text_width(&shown) + 8;
        d.fill(Rect::new(r.x, y - 1, w, ROW), colour::VEIL);
        // design: readable text and display scale (lead's request 2026-11-10)
        d.text_left_outlined(Rect::new(r.x + 4, y, w, ROW - 1), &shown, colour::ACCENT);
        y -= ROW;
    }
    for l in lines.iter().rev() {
        if y < r.y {
            break;
        }
        let shown = font::fit(&l.shown(), r.w - 8);
        let w = font::text_width(&shown) + 8;
        d.fill(Rect::new(r.x, y - 1, w, ROW), colour::VEIL);
        d.text_left_outlined(
            Rect::new(r.x + 4, y, w, ROW - 1),
            &shown,
            if l.notice {
                colour::ACCENT
            } else {
                colour::TEXT
            },
        );
        y -= ROW;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(who: &str, text: &str) -> Line {
        Line {
            who: who.into(),
            text: text.into(),
            notice: who.is_empty(),
        }
    }

    #[test]
    fn the_box_shows_the_latest_lines_and_the_input() {
        let lines: Vec<Line> = (0..30).map(|i| line("P", &format!("line {i}"))).collect();
        let mut d = DrawList::new();
        let f = view(&mut d, Rect::new(0, 0, 300, 100), &lines, "hi", true);
        let texts = d.texts();
        assert!(texts.contains(&"P: line 29"));
        assert!(!texts.contains(&"P: line 0"));
        assert!(texts.contains(&"hi_"));
        assert!(f.contains(crate::geom::Point::new(f.x + 1, f.y + 1)));
        let mut d = DrawList::new();
        overlay(
            &mut d,
            Rect::new(0, 0, 300, 100),
            &[line("", "Ann left")],
            Some("gg"),
        );
        let texts = d.texts();
        assert!(texts.contains(&"Ann left"));
        assert!(texts.contains(&"Say: gg_"));
    }
}
