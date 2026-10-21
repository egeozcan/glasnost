//! The main menu (our own layout).

use crate::draw::{DrawList, colour};
use crate::geom::{Point, Rect};
use crate::widget::{self, Targets};

/// A main-menu target.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuHit {
    /// Open the skirmish setup.
    Skirmish,
    /// Open the multiplayer screen (host or join a room on a relay).
    Multiplayer,
    /// Choose the game folder again.
    Install,
    /// Leave the program.
    Quit,
}

/// The line under the title.
pub const DISCLAIMER: &str =
    "An independent engine. Not affiliated with or endorsed by any publisher.";

/// Draws the main menu. `status` is a line about the mounted install (or a
/// problem); `can_quit` hides Quit where leaving is not ours (the web).
pub fn view(
    size: (i32, i32),
    status: &str,
    ready: bool,
    can_quit: bool,
    hover: Option<Point>,
) -> (DrawList, Targets<MenuHit>) {
    let (w, h) = size;
    let mut d = DrawList::new();
    let mut t = Targets::new();
    d.fill(Rect::new(0, 0, w, h), colour::BACKDROP);
    // A band of diagonal hatching behind the title: part of our own look.
    let band = Rect::new(0, h / 6, w, 44);
    d.fill(band, colour::PANEL);
    for x in (-(band.h)..w).step_by(12) {
        d.line(
            (x, band.bottom() - 1),
            (x + band.h - 1, band.y),
            colour::RAISED,
        );
    }
    d.fill(Rect::new(0, band.bottom(), w, 1), colour::ACCENT);
    d.text_centred(
        Rect::new(0, band.y, w, band.h),
        "G L A S N O S T",
        colour::ACCENT,
    );
    d.text_centred(
        Rect::new(0, band.bottom() + 6, w, 12),
        DISCLAIMER,
        colour::DIM,
    );

    let bw = 180;
    let x = (w - bw) / 2;
    let mut y = h / 2 - 40;
    let hot = |r: Rect| hover.is_some_and(|p| r.contains(p));
    let mut buttons = vec![
        (MenuHit::Skirmish, "Skirmish", ready),
        (MenuHit::Multiplayer, "Multiplayer", ready),
        (MenuHit::Install, "Game folder", true),
    ];
    if can_quit {
        buttons.push((MenuHit::Quit, "Quit", true));
    }
    for (hit, label, enabled) in buttons {
        let r = Rect::new(x, y, bw, 24);
        widget::button(&mut d, r, label, hot(r), enabled);
        if enabled {
            t.push(r, hit);
        }
        y += 32;
    }
    d.text_centred(
        Rect::new(0, h - 30, w, 14),
        status,
        if ready { colour::TEXT } else { colour::BAD },
    );
    (d, t)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buttons_hit_and_skirmish_waits_for_an_install() {
        let (d, t) = view((640, 480), "ready", true, true, None);
        assert!(d.texts().contains(&DISCLAIMER));
        for want in [
            MenuHit::Skirmish,
            MenuHit::Multiplayer,
            MenuHit::Install,
            MenuHit::Quit,
        ] {
            let r = t.rect_of(&want).unwrap();
            assert_eq!(t.hit(r.centre()), Some(want));
        }
        let (_, t) = view((640, 480), "no install", false, false, None);
        assert!(t.rect_of(&MenuHit::Skirmish).is_none());
        assert!(t.rect_of(&MenuHit::Multiplayer).is_none());
        assert!(t.rect_of(&MenuHit::Quit).is_none());
        assert!(t.rect_of(&MenuHit::Install).is_some());
    }
}
