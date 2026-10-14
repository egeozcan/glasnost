//! A GPU-independent list of 2D drawing primitives. Screens and the HUD
//! describe themselves as a [`DrawList`]; the renderer's 2D batch turns the
//! list into draws.

use crate::font;
use crate::geom::Rect;

/// An 8-bit RGBA colour.
pub type Rgba = [u8; 4];

/// A primitive, drawn in list order (later ones on top).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Prim {
    /// A filled rectangle; alpha below 255 blends over what is under it.
    Fill {
        /// The rectangle.
        rect: Rect,
        /// Its colour.
        rgba: Rgba,
    },
    /// A one-pixel outline just inside `rect`.
    Frame {
        /// The rectangle.
        rect: Rect,
        /// Its colour.
        rgba: Rgba,
    },
    /// A line of text in the interface font, top-left at `x, y`.
    Text {
        /// Left column.
        x: i32,
        /// Top row of the glyph box.
        y: i32,
        /// The text (one line).
        text: String,
        /// Ink colour.
        rgba: Rgba,
        /// Pixels outside are not drawn.
        clip: Rect,
    },
    /// A palette-indexed sprite frame, top-left at `x, y`.
    Sprite {
        /// The sprite, as the client registered it.
        id: u32,
        /// The frame.
        frame: u32,
        /// Left column.
        x: i32,
        /// Top row.
        y: i32,
        /// Pixels outside are not drawn.
        clip: Rect,
    },
    /// An RGBA image (such as the rendered map or the radar) stretched to
    /// `rect`.
    Image {
        /// The image, as the client registered it.
        id: u32,
        /// Where it goes.
        rect: Rect,
    },
    /// A line between two pixels (both included).
    Line {
        /// Start.
        from: (i32, i32),
        /// End.
        to: (i32, i32),
        /// Its colour.
        rgba: Rgba,
    },
}

/// A list of primitives.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DrawList {
    /// The primitives in drawing order.
    pub prims: Vec<Prim>,
}

impl DrawList {
    /// An empty list.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a filled rectangle.
    pub fn fill(&mut self, rect: Rect, rgba: Rgba) {
        if !rect.is_empty() {
            self.prims.push(Prim::Fill { rect, rgba });
        }
    }

    /// Adds a one-pixel outline.
    pub fn frame(&mut self, rect: Rect, rgba: Rgba) {
        if !rect.is_empty() {
            self.prims.push(Prim::Frame { rect, rgba });
        }
    }

    /// Adds text clipped to `clip`.
    pub fn text(&mut self, x: i32, y: i32, text: impl Into<String>, rgba: Rgba, clip: Rect) {
        self.prims.push(Prim::Text {
            x,
            y,
            text: text.into(),
            rgba,
            clip,
        });
    }

    /// Adds text centred in `rect` (cut with ".." when too wide).
    pub fn text_centred(&mut self, rect: Rect, text: &str, rgba: Rgba) {
        let t = font::fit(text, rect.w - 2);
        let x = rect.x + (rect.w - font::text_width(&t)) / 2;
        let y = rect.y + (rect.h - 7) / 2;
        self.text(x, y, t, rgba, rect);
    }

    /// Adds text left-aligned in `rect` with a small margin, vertically
    /// centred.
    pub fn text_left(&mut self, rect: Rect, text: &str, rgba: Rgba) {
        let t = font::fit(text, rect.w - 4);
        let y = rect.y + (rect.h - 7) / 2;
        self.text(rect.x + 3, y, t, rgba, rect);
    }

    /// Adds a sprite frame.
    pub fn sprite(&mut self, id: u32, frame: u32, x: i32, y: i32, clip: Rect) {
        self.prims.push(Prim::Sprite {
            id,
            frame,
            x,
            y,
            clip,
        });
    }

    /// Adds an RGBA image.
    pub fn image(&mut self, id: u32, rect: Rect) {
        if !rect.is_empty() {
            self.prims.push(Prim::Image { id, rect });
        }
    }

    /// Adds a line.
    pub fn line(&mut self, from: (i32, i32), to: (i32, i32), rgba: Rgba) {
        self.prims.push(Prim::Line { from, to, rgba });
    }

    /// Appends another list on top of this one.
    pub fn extend(&mut self, other: DrawList) {
        self.prims.extend(other.prims);
    }

    /// Every text string in the list, in order (for tests).
    pub fn texts(&self) -> Vec<&str> {
        self.prims
            .iter()
            .filter_map(|p| match p {
                Prim::Text { text, .. } => Some(text.as_str()),
                _ => None,
            })
            .collect()
    }
}

/// The interface colour scheme: graphite panels with amber accents (our own
/// design).
pub mod colour {
    use super::Rgba;

    /// Screen background.
    pub const BACKDROP: Rgba = [18, 20, 24, 255];
    /// Panel background.
    pub const PANEL: Rgba = [34, 37, 43, 255];
    /// Raised panel or idle button.
    pub const RAISED: Rgba = [52, 56, 64, 255];
    /// Hovered or active button.
    pub const HOT: Rgba = [78, 70, 44, 255];
    /// Panel and button edges.
    pub const EDGE: Rgba = [96, 102, 112, 255];
    /// Accent (titles, selection).
    pub const ACCENT: Rgba = [232, 176, 64, 255];
    /// Body text.
    pub const TEXT: Rgba = [214, 218, 224, 255];
    /// Dim text.
    pub const DIM: Rgba = [130, 136, 146, 255];
    /// Good (power surplus, legal placement, victory).
    pub const GOOD: Rgba = [96, 200, 112, 255];
    /// Bad (power shortage, illegal placement, defeat).
    pub const BAD: Rgba = [220, 80, 72, 255];
    /// Darkening veil over unavailable cameos.
    pub const VEIL: Rgba = [0, 0, 0, 150];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_rectangles_are_dropped_and_texts_listed() {
        let mut d = DrawList::new();
        d.fill(Rect::new(0, 0, 0, 5), colour::PANEL);
        d.text_centred(Rect::new(0, 0, 100, 20), "Hello", colour::TEXT);
        assert_eq!(d.prims.len(), 1);
        assert_eq!(d.texts(), vec!["Hello"]);
        match &d.prims[0] {
            Prim::Text { x, .. } => assert_eq!(*x, (100 - font::text_width("Hello")) / 2),
            p => panic!("{p:?}"),
        }
    }
}
