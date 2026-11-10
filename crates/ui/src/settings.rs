//! The settings screen (our own layout): volumes, scroll speed, display
//! scale, captions, the house colour choice, the tooltip option, touchpad
//! scrolling and the key bindings. It only
//! draws the values it is given and reports what was clicked; what they
//! mean and where they are stored is the client's (`glasnost-app`,
//! `settings`).

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
    /// Captions for announcements, on or off.
    Captions,
    /// House colours for colour-blind players, on or off.
    Colours,
    /// The tooltip option: name tips over the battlefield, on or off.
    ToolTips,
    /// Touchpad scrolling, on or off (our own addition).
    Touchpad,
}

/// All rows in the order drawn.
pub const ROWS: [Row; 9] = [
    Row::Sound,
    Row::Voices,
    Row::Music,
    Row::Scroll,
    Row::Scale,
    Row::Captions,
    Row::Colours,
    Row::ToolTips,
    Row::Touchpad,
];

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
    pub values: [String; 9],
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
        Row::Captions => "Captions",
        Row::Colours => "Colour-blind houses",
        Row::ToolTips => "Name tips",
        Row::Touchpad => "Touchpad scrolling",
    }
}

/// Draws the screen and lays out its targets: the values on the left, the
/// keys on the right, Reset and Back below.
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
    let pw = 620;
    let cw = 290;
    let left_h = 24 * ROWS.len() as i32;
    // design: the fixed keys sit in two columns under both columns, so the
    // panel stays within a 640x480 logical screen with 9 bindings.
    let keys_h = 18 + 22 * page.bindings.len() as i32;
    let fixed_lines = (page.fixed.len() as i32 + 1) / 2;
    let content = left_h.max(keys_h) + 18 + 14 * fixed_lines;
    let ph = 26 + content + 98;
    let r = Rect::new((w - pw) / 2, ((h - ph) / 2).max(4), pw, ph);
    widget::panel(&mut d, r, Some("SETTINGS"));
    let x0 = r.x + 10;
    let x1 = r.x + 20 + cw;
    let mut y = r.y + 26;
    for (row, value) in ROWS.iter().zip(&page.values) {
        let (a, b) = widget::choice(&mut d, Rect::new(x0, y, cw, 18), label(*row), value, true);
        t.push_choice(
            a,
            SettingsHit::Step(*row, -1),
            b,
            SettingsHit::Step(*row, 1),
        );
        y += 24;
    }
    let mut y = r.y + 26;
    d.text_left(Rect::new(x1, y, cw, 14), "Keys", colour::ACCENT);
    y += 18;
    for (i, (action, key)) in page.bindings.iter().enumerate() {
        let waiting = page.rebinding == Some(i);
        let br = Rect::new(x1, y, cw, 18);
        let text = if waiting {
            format!("{action}: press a key")
        } else {
            format!("{action}: {key}")
        };
        widget::button(&mut d, br, &text, waiting || hot(br), true);
        t.push(br, SettingsHit::Rebind(i));
        y += 22;
    }
    let y = r.y + 26 + left_h.max(keys_h);
    d.text_left(Rect::new(x0, y + 2, cw, 14), "Fixed keys", colour::ACCENT);
    let y = y + 18;
    for (i, (action, key)) in page.fixed.iter().enumerate() {
        let (col, line) = (i as i32 / fixed_lines, i as i32 % fixed_lines);
        d.text_left(
            Rect::new(
                if col == 0 { x0 } else { x1 } + 4,
                y + 14 * line,
                cw - 4,
                12,
            ),
            &format!("{action}: {key}"),
            colour::DIM,
        );
    }
    let y = r.y + 26 + content + 12;
    let reset = Rect::new(x0, y, 160, 20);
    widget::button(&mut d, reset, "Reset to defaults", hot(reset), true);
    t.push(reset, SettingsHit::Reset);
    let back = Rect::new(r.right() - 170, y, 160, 20);
    widget::button(&mut d, back, "Back", hot(back), true);
    t.push(back, SettingsHit::Back);
    d.text_centred(Rect::new(r.x, y + 30, pw, 14), page.note, colour::TEXT);
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
                "On".into(),
                "Off".into(),
                "On".into(),
                "On".into(),
            ],
            bindings: vec![("Stop".into(), "S".into()), ("Guard".into(), "G".into())],
            rebinding: None,
            fixed: vec![("Options".into(), "Escape".into())],
            note: "",
        }
    }

    #[test]
    fn every_target_is_reachable_at_a_small_window() {
        let mut p = page();
        p.bindings = (0..5).map(|i| (format!("A{i}"), "K".into())).collect();
        p.fixed = (0..12).map(|i| (format!("F{i}"), "K".into())).collect();
        let (d, t) = view((640, 480), &p, None);
        assert!(d.texts().contains(&"SETTINGS"));
        for hit in [
            SettingsHit::Step(Row::Music, 1),
            SettingsHit::Step(Row::Scale, -1),
            SettingsHit::Step(Row::Colours, 1),
            SettingsHit::Step(Row::ToolTips, -1),
            SettingsHit::Step(Row::Touchpad, 1),
            SettingsHit::Rebind(4),
            SettingsHit::Reset,
            SettingsHit::Back,
        ] {
            let r = t.rect_of(&hit).unwrap();
            assert_eq!(t.hit(r.centre()), Some(hit));
            assert!(r.y >= 0 && r.bottom() <= 480, "{hit:?} {r:?}");
        }
        for size in [(640, 480), (800, 600)] {
            let mut p = page();
            p.bindings = (0..9).map(|i| (format!("A{i}"), "K".into())).collect();
            p.fixed = (0..14).map(|i| (format!("F{i}"), "K".into())).collect();
            let (_, t) = view(size, &p, None);
            for (r, _) in &t.list {
                assert!(
                    r.x >= 0 && r.y >= 0 && r.right() <= size.0 && r.bottom() <= size.1,
                    "{size:?} {r:?}"
                );
            }
        }
        // Every value row is one keyboard stop, then the keys, Reset and Back.
        assert_eq!(t.stops.len(), ROWS.len() + 5 + 2);
        assert!(d.texts().contains(&"F11: K"));
    }

    /// Every row's label fits beside its value box.
    #[test]
    fn every_label_fits_its_row() {
        for row in ROWS {
            let w = crate::font::text_width(label(row));
            assert!(w <= 290 * 2 / 5, "{row:?} is {w} pixels wide");
        }
    }
}
