//! The display scale in the client (design: readable text and display
//! scale (lead's request 2026-11-10)): the view a window gives, the
//! settings row, and host positions converted to view pixels at 2x.

use super::*;
use glasnost_ui::settings::{Row, SettingsHit};

/// A 1280 × 800 logical window on a display with a scale factor of 2.
fn two_times() -> Client {
    let mut c = Client::new();
    c.set_window((2560, 1600), 2.0);
    c
}

/// The host position, in physical pixels, of the middle of a target laid
/// out in the interface's logical pixels, at display factor `f`: the
/// middle of the view rectangle, at the last physical pixel of its square.
fn host_middle(c: &Client, r: Rect) -> (f64, f64) {
    let r = c.ui_scale().rect(r);
    let f = c.display().factor;
    let p = Point::new(r.x + r.w / 2, r.y + r.h / 2);
    (
        f64::from(p.x * f + f - 1) + 0.5,
        f64::from(p.y * f + f - 1) + 0.5,
    )
}

/// A host click, converted to view pixels as the window converts it.
fn host_click(c: &mut Client, at: (f64, f64)) {
    let p = c.display().point(at.0, at.1);
    c.pointer_moved(p);
    c.pointer_down(Button::Left, p);
    c.pointer_up(Button::Left, p, Modifiers::default());
}

#[test]
fn a_two_times_window_gives_a_view_of_its_logical_size() {
    let c = two_times();
    assert_eq!(c.size, (1280, 800));
    assert_eq!(c.display().factor, 2);
    assert_eq!(
        c.ui_size(),
        (1280, 800),
        "the interface at 100 % of the view"
    );
    // A move to a 1x monitor (a new scale factor, then the new size).
    let mut c = c;
    c.set_window((2560, 1600), 1.0);
    assert_eq!((c.size, c.display().factor), ((2560, 1600), 1));
    c.set_window((1280, 800), 1.0);
    assert_eq!(c.size, (1280, 800));
    // A client sized only through `resize` draws one view pixel per screen pixel.
    c.resize((800, 600));
    assert_eq!(c.display(), Display::one((800, 600)));
}

#[test]
fn a_mouse_click_at_two_times_lands_on_its_button() {
    let mut c = two_times();
    let _ = c.frame();
    let r = c.menu_targets.rect_of(&MenuHit::Settings).unwrap();
    let at = host_middle(&c, r);
    host_click(&mut c, at);
    assert_eq!(c.shell.screen, Screen::Settings);
}

#[test]
fn a_tap_at_two_times_lands_on_its_button() {
    let mut c = two_times();
    let _ = c.frame();
    let r = c.menu_targets.rect_of(&MenuHit::Settings).unwrap();
    let at = host_middle(&c, r);
    let p = c.display().point(at.0, at.1);
    let mods = Modifiers::default();
    c.touch(touch::Phase::Start, 1, p, 1000, mods);
    c.touch(touch::Phase::End, 1, p, 1050, mods);
    assert_eq!(c.shell.screen, Screen::Settings);
}

/// The settings row steps Auto, 1x, 2x, 3x; the view follows at once and
/// the row names the factor in use.
#[test]
fn the_settings_row_changes_the_factor_at_once() {
    let mut c = two_times();
    let _ = c.frame();
    let r = c.menu_targets.rect_of(&MenuHit::Settings).unwrap();
    let at = host_middle(&c, r);
    host_click(&mut c, at);
    let d = c.frame();
    assert!(d.0.texts().contains(&"Display scale"));
    assert!(d.0.texts().contains(&"Interface size"));
    assert!(d.0.texts().contains(&"Auto (2x)"));
    let step = |c: &mut Client| {
        let r = c
            .settings_targets
            .rect_of(&SettingsHit::Step(Row::Display, 1))
            .unwrap();
        let at = host_middle(c, r);
        host_click(c, at);
        let d = c.frame();
        d.0.texts()
            .iter()
            .map(|s| (*s).to_owned())
            .collect::<Vec<_>>()
    };
    let texts = step(&mut c);
    assert_eq!(c.settings.display_scale, 1);
    assert_eq!((c.size, c.display().factor), ((2560, 1600), 1));
    assert!(texts.contains(&"1x".to_owned()));
    let texts = step(&mut c);
    assert_eq!((c.size, c.display().factor), ((1280, 800), 2));
    assert!(texts.contains(&"2x".to_owned()));
    // 3x would leave 853 x 533 view pixels: it fits.
    let texts = step(&mut c);
    assert_eq!((c.size, c.display().factor), ((853, 533), 3));
    assert!(texts.contains(&"3x".to_owned()));
    // On a smaller window 3x does not fit, and the row says so.
    c.set_window((1600, 1200), 2.0);
    let d = c.frame();
    assert_eq!(c.display().factor, 2);
    assert!(d.0.texts().contains(&"3x (2x fits)"));
}
