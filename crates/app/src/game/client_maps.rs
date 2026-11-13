//! The skirmish setup's map picker in the client (design: map picker
//! (lead's request 2026-11-13)): the facts it filters on, the chosen map's
//! preview, the picker's keys, wheel and touch, and its search and filters
//! kept in the settings.
//!
//! A preview is decoded only when its map is chosen, in [`Client::update`]
//! and never while a frame is drawn, at most one per update; the last
//! [`KEPT`] decoded previews are kept, and the shown one is uploaded once,
//! as image [`PREVIEW_IMAGE`]. The map list itself holds each preview
//! packed (`install::MapInfo`), so hundreds of maps hold no pictures.

use std::collections::VecDeque;

use glasnost_ui::draw::DrawList;
use glasnost_ui::geom::{Point, Rect};
use glasnost_ui::mappick::{self, MapFacts, PreviewShow, Shown};
use glasnost_ui::setup::{Listed, MapEntry, SetupHit};
use glasnost_ui::shell::Screen;
use glasnost_ui::widget::Focus;

use super::{Client, Key, busy_veil};

/// The image id of the chosen map's preview (the game's battlefield and
/// radar are 1 and 2).
pub const PREVIEW_IMAGE: u32 = 3;

/// Decoded previews kept.
// design: map picker (lead's request 2026-11-13)
const KEPT: usize = 32;

/// The longest preview side uploaded: a larger preview is shown at every
/// n-th pixel, n the smallest that fits.
// design: map picker (lead's request 2026-11-13)
const SIDE_MOST: u32 = 1024;

/// Wheel notches move the picker's list this many rows.
// design: map picker (lead's request 2026-11-13)
const WHEEL_ROWS: i32 = 3;

/// A decoded preview, ready to upload.
pub(super) struct Decoded {
    pub(super) w: u32,
    pub(super) h: u32,
    pub(super) rgba: Vec<u8>,
}

/// The decoded previews kept, and the one uploaded.
#[derive(Default)]
pub(super) struct Previews {
    /// `(map index, preview)`, the most recently used last.
    kept: VecDeque<(usize, Decoded)>,
    /// The map whose preview is uploaded as [`PREVIEW_IMAGE`].
    uploaded: Option<usize>,
}

impl Previews {
    fn get(&self, at: usize) -> Option<&Decoded> {
        self.kept.iter().find(|(i, _)| *i == at).map(|(_, d)| d)
    }

    fn insert(&mut self, at: usize, d: Decoded) {
        self.kept.retain(|(i, _)| *i != at);
        while self.kept.len() >= KEPT {
            self.kept.pop_front();
        }
        self.kept.push_back((at, d));
    }

    /// The device holds no preview (a new one, or none yet).
    pub(super) fn forget_upload(&mut self) {
        self.uploaded = None;
    }

    /// How many previews are kept decoded.
    #[cfg(test)]
    pub(super) fn kept(&self) -> usize {
        self.kept.len()
    }
}

/// A `width` × `height` RGB preview as RGBA, at every n-th pixel when a
/// side is over [`SIDE_MOST`].
pub(super) fn preview_rgba(width: u32, height: u32, rgb: &[u8]) -> Decoded {
    let step = width.max(height).div_ceil(SIDE_MOST).max(1);
    let (w, h) = (width.div_ceil(step), height.div_ceil(step));
    let mut rgba = Vec::with_capacity(w as usize * h as usize * 4);
    for y in 0..h {
        for x in 0..w {
            let i = ((y * step) as usize * width as usize + (x * step) as usize) * 3;
            let px = rgb.get(i..i + 3).unwrap_or(&[0, 0, 0]);
            rgba.extend_from_slice(&[px[0], px[1], px[2], 255]);
        }
    }
    Decoded { w, h, rgba }
}

impl Client {
    /// The offered maps' entries.
    fn map_entries(&self) -> Vec<MapEntry> {
        self.install.as_ref().map_or_else(Vec::new, |i| {
            i.maps.iter().map(|(_, m)| m.clone()).collect()
        })
    }

    /// The picker's facts of the offered maps, by index (none known for an
    /// install read without them).
    pub(super) fn map_facts(&self) -> Vec<MapFacts> {
        self.install.as_ref().map_or_else(Vec::new, |i| {
            i.map_info.iter().map(|m| m.facts.clone()).collect()
        })
    }

    /// Applies the picker's selection rule to the setup (after the
    /// skirmish screen opens with a kept search and filters).
    pub(super) fn refilter_maps(&mut self) {
        let (maps, facts) = (self.map_entries(), self.map_facts());
        self.shell.refilter(&maps, &facts);
    }

    /// Keeps the picker's search and filters in the settings when they
    /// changed (only the store is written: nothing else follows them).
    pub(super) fn keep_map_query(&mut self) {
        if self.settings.maps == self.shell.picker.query {
            return;
        }
        self.settings.maps = self.shell.picker.query.clone();
        if let Some(store) = self.store.as_ref()
            && let Err(e) = store.write(&self.settings.to_text())
        {
            log::warn!("settings not kept: {e}");
        }
    }

    /// A setup-screen control pressed (a click or the keyboard).
    pub(super) fn setup_press(&mut self, hit: SetupHit) {
        let Some(i) = self.install.as_ref() else {
            return;
        };
        let rules = i.setup.clone();
        let (maps, facts) = (self.map_entries(), self.map_facts());
        let was_open = self.shell.picker.open;
        let e = self.shell.setup_with(hit, &rules, &maps, &facts);
        if self.shell.picker.open != was_open {
            // Another set of controls: the keyboard focus starts again.
            self.setup_focus = Focus::new();
        }
        self.keep_map_query();
        self.event(e);
    }

    /// Text typed into the picker's search.
    pub(super) fn search_typed(&mut self, text: &str) {
        let (maps, facts) = (self.map_entries(), self.map_facts());
        self.shell.type_search(text, &maps, &facts);
        self.keep_map_query();
    }

    /// The picker's own keys while it is open: Backspace in the search,
    /// Escape closes it, and with no control focused Enter closes it and
    /// Up and Down choose the previous or next map of the list. Returns
    /// whether the key was taken.
    pub(super) fn picker_key(&mut self, k: Key) -> bool {
        if self.shell.screen != Screen::Setup || !self.shell.picker.open || self.busy {
            return false;
        }
        let free = !self.setup_focus.shown;
        match k {
            Key::Backspace if self.shell.picker.typing => {
                let (maps, facts) = (self.map_entries(), self.map_facts());
                self.shell.search_backspace(&maps, &facts);
                self.keep_map_query();
            }
            Key::Escape => self.setup_press(SetupHit::MapsDone),
            Key::Enter if free => self.setup_press(SetupHit::MapsDone),
            Key::Up if free => self.setup_press(SetupHit::Map(-1)),
            Key::Down if free => self.setup_press(SetupHit::Map(1)),
            _ => return false,
        }
        true
    }

    /// After a focus step on the picker: typing goes to the search while
    /// it has the focus.
    pub(super) fn picker_focus_moved(&mut self) {
        if self.shell.picker.open {
            self.shell.picker.typing = matches!(
                self.setup_focus.target(&self.setup_targets),
                Some(SetupHit::SearchField)
            );
        }
    }

    /// Whether the open picker takes the wheel and touch drags.
    pub(super) fn picker_scrolls(&self) -> bool {
        self.shell.screen == Screen::Setup && self.shell.picker.open
    }

    /// A wheel step `delta` (S-0411 claim 18's sign: positive, the content
    /// moves down) on the open picker scrolls its list.
    pub(super) fn picker_wheel(&mut self, delta: i32) {
        self.setup_press(SetupHit::ScrollMaps(-delta.signum() * WHEEL_ROWS));
    }

    /// `rows` rows of a swipe or a drag (positive, the content moves down)
    /// on the open picker scroll its list with the finger.
    pub(super) fn picker_rows(&mut self, rows: i32) {
        if rows != 0 {
            self.setup_press(SetupHit::ScrollMaps(-rows));
        }
    }

    /// A list row's height in the client's pixels.
    pub(super) fn picker_pitch(&self) -> i32 {
        self.ui_scale()
            .rect(Rect::new(0, 0, 1, mappick::ROW_H))
            .h
            .max(1)
    }

    /// Decodes the chosen map's preview when it is not kept yet (at most
    /// one per update, outside the frame).
    pub(super) fn decode_preview(&mut self) {
        if self.shell.screen != Screen::Setup {
            return;
        }
        let (Some(s), Some(i)) = (self.shell.setup.as_ref(), self.install.as_ref()) else {
            return;
        };
        let at = s.map;
        if self.previews.get(at).is_some() {
            return;
        }
        let Some(src) = i.map_info.get(at).and_then(|m| m.preview.as_ref()) else {
            return;
        };
        // spec: S-0009 claim 24 and S-0012 claim 8 (the preview's pixels, read row by row from its expanded block; a short stream leaves the rest black)
        let p = src.decode();
        let d = preview_rgba(p.width, p.height, &p.rgb);
        self.previews.insert(at, d);
    }

    /// The chosen map's preview for this frame, uploaded when the image
    /// holds another.
    fn preview_shown(&mut self, changed: &mut Vec<(u32, u32, u32, Vec<u8>)>) -> PreviewShow {
        let (Some(s), Some(i)) = (self.shell.setup.as_ref(), self.install.as_ref()) else {
            return PreviewShow::None;
        };
        let at = s.map;
        if !i.map_info.get(at).is_some_and(|m| m.preview.is_some()) {
            return PreviewShow::None;
        }
        let Some(d) = self.previews.get(at) else {
            return PreviewShow::Pending;
        };
        let size = (d.w as i32, d.h as i32);
        if self.previews.uploaded != Some(at) {
            changed.push((PREVIEW_IMAGE, d.w, d.h, d.rgba.clone()));
            self.previews.uploaded = Some(at);
        }
        self.painter.set_image_size(PREVIEW_IMAGE, size);
        PreviewShow::Image {
            id: PREVIEW_IMAGE,
            size,
        }
    }

    /// The skirmish screen, or the picker over it.
    pub(super) fn setup_frame(
        &mut self,
        size: (i32, i32),
        hover: Option<Point>,
        changed: &mut Vec<(u32, u32, u32, Vec<u8>)>,
    ) -> DrawList {
        let preview = self.preview_shown(changed);
        let (maps, facts) = (self.map_entries(), self.map_facts());
        let (Some(setup), Some(i)) = (self.shell.setup.as_ref(), self.install.as_ref()) else {
            return DrawList::new();
        };
        let marks = i.map_info.get(setup.map).and_then(|m| m.marks.as_ref());
        let shown = Shown { preview, marks };
        let list = self.shell.maps_shown(&maps, &facts);
        let (mut d, t) = if self.shell.picker.open {
            self.shell
                .picker
                .view(size, &maps, &facts, setup.map, &list, &shown, hover)
        } else {
            let rules = self.shown_setup_rules(&i.setup);
            let listed = Listed {
                any: !list.is_empty(),
                shown,
            };
            setup.view_listed(size, &rules, &maps, hover, &listed)
        };
        self.setup_focus.mark(&mut d, &t);
        if self.busy {
            busy_veil(&mut d, size, &self.status);
        }
        self.setup_targets = t;
        d
    }
}
