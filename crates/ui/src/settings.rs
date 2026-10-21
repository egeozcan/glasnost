//! The settings screen (our own layout): volumes, scroll speed, display
//! scale and the key bindings. It only draws the values it is given and
//! reports what was clicked; what they mean and where they are stored is
//! the client's (`glasnost-app`, `settings`).

use crate::draw::{DrawList, colour};
use crate::geom::{Point, Rect};
use crate::widget::{self, Targets};

/// A stepped value row.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Row {
    /// Sound-effects volume.
    Sound,
    /// Announcement (voice) volume.
    Voices,
    /// Music volume.
    Music,
    /// Scroll speed.
    Scroll,
    /// Display scale of the interface.
    Scale,
}

/// All rows in the order drawn.
pub const ROWS: [Row; 5] = [Row::Sound, Row::Voices, Row::Music, Row::Scroll, Row::Scale];

/// A target of the screen.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingsHit {
    /// A step of a value row: -1 lower, +1 higher.
    Step(Row, i32),
    /// Start rebinding the binding at this index.
    Rebind(usize),
    /// Restore every setting to its default.
    Reset,
    /// Leave the screen.
    Back,
}

/// What the screen shows.
pub struct Page<'a> {
    /// The value text of each row, in [`ROWS`] order.
    pub values: [String; 5],
    /// Each rebindable action and the key it has now.
    pub bindings: Vec<(String, String)>,
    /// The binding waiting for a key press.
    pub rebinding: Option<usize>,
    /// Keys that cannot be rebound: label and key.
    pub fixed: Vec<(String, String)>,
    /// A line about the last change (a refusal or a storage problem).
    pub note: &'a str,
}

fn label(r: Row) -> &'static str {
    match r {
        Row::Sound => "Sound effects",
        Row::Voices => "Voices",
        Row::Music => "Music",
        Row::Scroll => "Scroll speed",
        Row::Scale => "Display scale",
    }
}

/// Draws the screen and lays out its targets.
pub fn view(
    size: (i32, i32),
    page: &Page<'_>,
    hover: Option<Point>,
) -> (DrawList, Targets<SettingsHit>) {
    let (w, h) = size;
    let mut d = DrawList::new();
    let mut t = Targets::new();
    d.fill(Rect::new(0, 0, w, h), colour::BACKDROP);
    let hot = |r: Rect| hover.is_some_and(|p| r.contains(p));
    let pw = 360;
    let ph = 40 + 24 * 5 + 22 + 22 * page.bindings.len() as i32 + 14 * page.fixed.len() as i32 + 50;
    let r = Rect::new((w - pw) / 2, ((h - ph) / 2).max(4), pw, ph);
    widget::panel(&mut d, r, Some("SETTINGS"));
    let mut y = r.y + 26;
    for (row, value) in ROWS.iter().zip(&page.values) {
        let (a, b) = widget::choice(
            &mut d,
            Rect::new(r.x + 10, y, pw - 20, 18),
            label(*row),
            value,
            true,
        );
        t.push(a, SettingsHit::Step(*row, -1));
        t.push(b, SettingsHit::Step(*row, 1));
        y += 24;
    }
    y += 4;
    d.text_left(Rect::new(r.x + 10, y, pw - 20, 14), "Keys", colour::ACCENT);
    y += 18;
    for (i, (action, key)) in page.bindings.iter().enumerate() {
        let waiting = page.rebinding == Some(i);
        let br = Rect::new(r.x + 10, y, pw - 20, 18);
        let text = if waiting {
            format!("{action}: press a key")
        } else {
            format!("{action}: {key}")
        };
        widget::button(&mut d, br, &text, waiting || hot(br), true);
        t.push(br, SettingsHit::Rebind(i));
        y += 22;
    }
    for (action, key) in &page.fixed {
        d.text_left(
            Rect::new(r.x + 14, y, pw - 28, 12),
            &format!("{action}: {key} (fixed)"),
            colour::DIM,
        );
        y += 14;
    }
    y += 6;
    let reset = Rect::new(r.x + 10, y, 160, 20);
    widget::button(&mut d, reset, "Reset to defaults", hot(reset), true);
    t.push(reset, SettingsHit::Reset);
    let back = Rect::new(r.right() - 170, y, 160, 20);
    widget::button(&mut d, back, "Back", hot(back), true);
    t.push(back, SettingsHit::Back);
    d.text_centred(Rect::new(r.x, y + 26, pw, 14), page.note, colour::TEXT);
    (d, t)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn page() -> Page<'static> {
        Page {
            values: [
                "100".into(),
                "100".into(),
                "100".into(),
                "4 of 7".into(),
                "100%".into(),
            ],
            bindings: vec![("Stop".into(), "S".into()), ("Guard".into(), "G".into())],
            rebinding: None,
            fixed: vec![("Options".into(), "Escape".into())],
            note: "",
        }
    }

    #[test]
    fn every_target_is_reachable_at_a_small_window() {
        let (d, t) = view((640, 480), &page(), None);
        assert!(d.texts().contains(&"SETTINGS"));
        for hit in [
            SettingsHit::Step(Row::Music, 1),
            SettingsHit::Step(Row::Scale, -1),
            SettingsHit::Rebind(1),
            SettingsHit::Reset,
            SettingsHit::Back,
        ] {
            let r = t.rect_of(&hit).unwrap();
            assert_eq!(t.hit(r.centre()), Some(hit));
            assert!(r.y >= 0 && r.bottom() <= 480, "{hit:?} {r:?}");
        }
    }
}
