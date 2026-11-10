//! Captions for announcements (M8 step C3): each announcement the sound
//! side starts (S-0701 claims 26 and 27) is shown as a line of text for a
//! few seconds, so that a player who cannot hear it still reads it. The
//! text is S-0701 claim 26's (answering Q-1735): the entry's text key in the
//! player's own announcement data when present with a nonempty value, else
//! the entry's name. The timing and the look are our own interface choices,
//! which claim 26 lets stand (4 s, at most 3 at once, newest last).

use glasnost_ui::draw::{DrawList, colour};
use glasnost_ui::font;
use glasnost_ui::geom::Rect;

/// How long a caption stays, host milliseconds.
// spec: S-0701 claim 26 (each caption shown for 4 seconds: our interface choice, which the claim lets stand)
pub const CAPTION_MS: i64 = 4000;

/// The most captions shown at once (newest at the bottom).
// spec: S-0701 claim 26 (at most 3 at once, newest last: our interface choice, which the claim lets stand)
pub const CAPTION_LINES: usize = 3;

/// The captions on screen.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Captions {
    /// Each caption's text and the host time it goes, oldest first.
    shown: Vec<(String, i64)>,
}

impl Captions {
    /// No captions.
    pub fn new() -> Self {
        Self::default()
    }

    /// A caption for an announcement that started at host time `now_ms`.
    pub fn push(&mut self, text: &str, now_ms: i64) {
        self.shown.push((text.to_owned(), now_ms + CAPTION_MS));
        if self.shown.len() > CAPTION_LINES {
            self.shown.remove(0);
        }
    }

    /// Drops the captions whose time is over at host time `now_ms`.
    pub fn update(&mut self, now_ms: i64) {
        self.shown.retain(|(_, until)| now_ms < *until);
    }

    /// Removes every caption.
    pub fn clear(&mut self) {
        self.shown.clear();
    }

    /// The texts shown, oldest first.
    pub fn texts(&self) -> Vec<&str> {
        self.shown.iter().map(|(t, _)| t.as_str()).collect()
    }

    /// Draws the captions centred low over the battlefield `map` (logical
    /// pixels), above the chat's place at its bottom left.
    pub fn draw(&self, d: &mut DrawList, map: Rect) {
        let n = self.shown.len() as i32;
        for (i, (text, _)) in self.shown.iter().enumerate() {
            let t = font::fit(text, map.w - 40);
            let w = font::text_width(&t) + 16;
            let y = map.bottom() - 230 - (n - i as i32) * 18;
            let r = Rect::new(map.x + (map.w - w) / 2, y, w, 16);
            d.fill(r, [0, 0, 0, 190]);
            // design: readable text and display scale (lead's request 2026-11-10)
            d.text_centred_outlined(r, &t, colour::TEXT);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_caption_stays_for_its_time_and_the_newest_three_show() {
        let mut c = Captions::new();
        c.push("ALPHA", 1000);
        c.update(1000 + CAPTION_MS - 1);
        assert_eq!(c.texts(), ["ALPHA"]);
        c.update(1000 + CAPTION_MS);
        assert!(c.texts().is_empty());
        for (i, t) in ["ONE", "TWO", "THREE", "FOUR"].iter().enumerate() {
            c.push(t, 2000 + i as i64);
        }
        assert_eq!(c.texts(), ["TWO", "THREE", "FOUR"]);
        let mut d = DrawList::new();
        c.draw(&mut d, Rect::new(0, 0, 640, 480));
        assert_eq!(d.texts(), ["TWO", "THREE", "FOUR"]);
        c.update(2001 + CAPTION_MS);
        assert_eq!(c.texts(), ["THREE", "FOUR"]);
    }
}
