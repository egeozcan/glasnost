//! A map scene: every image of the battlefield placed in world pixels, in
//! draw order, ready to be composited for any view position.
//!
//! The scene is built once per map (by the app's map viewer). Each frame,
//! [`Scene::compose`] turns it into [`compose::Drawing`]s for the current
//! view origin and depth reference and composites them on the CPU; the GPU
//! path (`map_gpu`) draws the same items with the same depth rules.
//!
//! World pixels and rows are those of S-0101 F1; a drawing's depth is tied to
//! world rows through the depth reference *C* (S-0102 F1), so items are
//! stored with world rows and converted to view rows per frame.

use glasnost_fmt::image::IndexedImage;

use crate::compose::{
    self, ComposeQuirks, DepthSource, DepthTest, Drawing, Frame, Paint, Profile, Row565,
    SignedImage,
};
use crate::spots::{self, SpotGroup};

/// What an item shows, for layer toggles and object info.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Group {
    /// Terrain tiles.
    Tiles,
    /// Overlays.
    Overlays,
    /// Smudges.
    Smudges,
    /// Terrain objects (trees and similar).
    TerrainObjects,
    /// Building bodies.
    Buildings,
    /// Building bibs.
    Bibs,
    /// Vehicles.
    Vehicles,
    /// Infantry.
    Infantry,
    /// Aircraft.
    Aircraft,
    /// Shadows of any kind.
    Shadows,
    /// Combat effects: projectiles, shots, detonations and animations
    /// (the app's placeholders).
    Effects,
    /// Status extras: health bars, selection brackets, insignia (S-0111).
    Status,
}

impl Group {
    /// Every group, in a fixed order.
    pub const ALL: [Group; 12] = [
        Group::Tiles,
        Group::Overlays,
        Group::Smudges,
        Group::TerrainObjects,
        Group::Buildings,
        Group::Bibs,
        Group::Vehicles,
        Group::Infantry,
        Group::Aircraft,
        Group::Shadows,
        Group::Effects,
        Group::Status,
    ];

    /// A short label.
    pub fn label(self) -> &'static str {
        match self {
            Group::Tiles => "tiles",
            Group::Overlays => "overlays",
            Group::Smudges => "smudges",
            Group::TerrainObjects => "terrain objects",
            Group::Buildings => "buildings",
            Group::Bibs => "bibs",
            Group::Vehicles => "vehicles",
            Group::Infantry => "infantry",
            Group::Aircraft => "aircraft",
            Group::Shadows => "shadows",
            Group::Effects => "combat effects",
            Group::Status => "status extras",
        }
    }
}

/// An item's depth, in world rows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ItemDepth {
    /// A terrain tile (S-0102 F3): tile depth image index and the world row
    /// of the tile's ground reference top row.
    Tile {
        /// Index into [`Scene::images`] of the tile depth data.
        depth: Option<u32>,
        /// World row *t*.
        reference_row: i32,
    },
    /// A profile drawing (S-0102 F2).
    Profile {
        /// Profile.
        profile: Profile,
        /// Depth bias.
        bias: i32,
        /// Companion depth image: index into [`Scene::signed`] and its
        /// top-left world pixel.
        companion: Option<(u32, i32, i32)>,
    },
}

/// One placed image.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Item {
    /// Index into [`Scene::images`].
    pub image: u32,
    /// World pixel of the image's top-left corner.
    pub x: i32,
    /// World pixel of the image's top-left corner.
    pub y: i32,
    /// Depth test.
    pub test: DepthTest,
    /// Depth.
    pub depth: ItemDepth,
    /// Paint: a row of [`Scene::rows`] or a shadow.
    pub paint: Paint,
    /// Group, for toggles.
    pub group: Group,
    /// Whether the item rests on a provisional choice (a spec request).
    pub provisional: bool,
    /// The object this item belongs to (an index of the caller's object
    /// table), for hover info.
    pub object: Option<u32>,
    /// A tiled animation's stack: the item is its lowest copy, and the
    /// view decides how many copies above it are drawn (S-0109 claim 27).
    pub stack: Option<Stack>,
}

/// The vertical stack of copies of a tiled animation (S-0109 claim 27):
/// copy *j* is the item moved up by *j h* rows with its depth bias reduced
/// by *j h* (S-0102 claim 13).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Stack {
    /// The spacing *h* > 0: the stored rectangle height of image frame 0.
    pub spacing: i32,
    /// The world row *pᵧ* − ⌊*h*/2⌋ of the lowest copy's unshifted centre.
    pub base_row: i32,
}

impl Stack {
    /// The number of copies *C* in a view whose top row is world row `top`
    /// (S-0109 claim 27).
    pub fn copies(&self, top: i32) -> i32 {
        let above = self.base_row - top;
        if above < 0 {
            // spec: S-0109 claim 27 (C = 1 when pᵧ − ⌊h/2⌋ < 0)
            1
        } else {
            // spec: S-0109 claim 27 (C = ⌊(pᵧ − ⌊h/2⌋) / h⌋ + 2; both operands are non-negative here)
            above / self.spacing + 2
        }
    }
}

impl Item {
    /// The copies of this item drawn in a view whose top row is world row
    /// `top`, lowest first (S-0109 claim 27): the item alone unless it is a
    /// stack.
    pub fn copies(&self, top: i32) -> impl Iterator<Item = Item> + '_ {
        let (n, h) = self.stack.map_or((1, 0), |s| (s.copies(top), s.spacing));
        (0..n).map(move |j| {
            let mut c = *self;
            // spec: S-0109 claim 27 (copy j is centred j·h rows above the lowest)
            c.y -= j * h;
            if let ItemDepth::Profile { bias, .. } = &mut c.depth {
                // spec: S-0102 claim 13 (tiled copy j: the bias reduced by exactly j·h)
                *bias -= j * h;
            }
            c
        })
    }
}

/// A view of the scene: origin in world pixels, size, depth reference.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct View {
    /// World pixel at view pixel (0, 0) (S-0101: the view origin *O*).
    pub origin: (i32, i32),
    /// View size in pixels.
    pub size: (u32, u32),
    /// Depth reference *C* (S-0102 F1).
    pub reference: i32,
}

/// Which items to draw.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Filter {
    /// Groups shown.
    pub groups: Vec<Group>,
    /// Whether provisional items are drawn.
    pub provisional: bool,
}

impl Default for Filter {
    fn default() -> Self {
        Self {
            groups: Group::ALL.to_vec(),
            provisional: true,
        }
    }
}

impl Filter {
    /// Whether `item` passes the filter.
    pub fn shows(&self, item: &Item) -> bool {
        (self.provisional || !item.provisional) && self.groups.contains(&item.group)
    }

    /// Whether the spot group `g` passes the filter.
    pub fn shows_spots(&self, g: &SpotGroup) -> bool {
        (self.provisional || !g.provisional) && self.groups.contains(&g.group)
    }
}

/// A scene.
#[derive(Clone, Debug, Default)]
pub struct Scene {
    /// Images (palette indices, 0 transparent; tile depth data too).
    pub images: Vec<IndexedImage>,
    /// Signed companion depth images.
    pub signed: Vec<SignedImage>,
    /// Palette rows.
    pub rows: Vec<Row565>,
    /// Items in draw order: the cell pass, then the object pass.
    pub items: Vec<Item>,
    /// Number of leading items that belong to the cell pass.
    pub cell_pass_len: usize,
    /// Single-pixel drawings placed between items ([`spots`]), sorted by
    /// [`SpotGroup::at`]; groups at the same place are drawn in list order.
    pub spots: Vec<SpotGroup>,
    /// The images rewritten in place by [`Scene::replace_image`]: each
    /// index with the value of [`Scene::rewrites`] at its last rewrite, so
    /// that an uploader can find those rewritten since it last looked.
    pub rewritten: std::collections::BTreeMap<u32, u64>,
    /// The number of in-place rewrites so far.
    pub rewrites: u64,
}

impl Scene {
    /// Adds an image and returns its index.
    pub fn add_image(&mut self, image: IndexedImage) -> u32 {
        self.images.push(image);
        (self.images.len() - 1) as u32
    }

    /// Replaces image `id` in place with `image` of the same size, and
    /// records the rewrite. Returns false, changing nothing, when there is
    /// no such image or the sizes differ.
    pub fn replace_image(&mut self, id: u32, image: IndexedImage) -> bool {
        let Some(old) = self.images.get_mut(id as usize) else {
            return false;
        };
        if (old.width(), old.height()) != (image.width(), image.height()) {
            return false;
        }
        *old = image;
        self.rewrites += 1;
        self.rewritten.insert(id, self.rewrites);
        true
    }

    /// Replaces image `id` in place with `image` of any size, and records
    /// the rewrite; an uploader moves it when its size changed (design:
    /// render (#104, #105)). A 0 × 0 `image` releases the old one's pixels.
    /// Returns false, changing nothing, when there is no such image.
    pub fn resize_image(&mut self, id: u32, image: IndexedImage) -> bool {
        let Some(old) = self.images.get_mut(id as usize) else {
            return false;
        };
        *old = image;
        self.rewrites += 1;
        self.rewritten.insert(id, self.rewrites);
        true
    }

    /// The images the scene holds (those with pixels) and their pixels:
    /// the benchmark's gauges (#105).
    pub fn held(&self) -> (usize, u64) {
        self.images
            .iter()
            .map(|i| u64::from(i.width()) * u64::from(i.height()))
            .filter(|&px| px > 0)
            .fold((0, 0), |(n, sum), px| (n + 1, sum + px))
    }

    /// Adds a palette row and returns its index.
    pub fn add_row(&mut self, row: Row565) -> usize {
        self.rows.push(row);
        self.rows.len() - 1
    }

    /// The world rectangle (left, top, right, bottom; right and bottom
    /// exclusive) of `item`.
    pub fn bounds(&self, item: &Item) -> (i32, i32, i32, i32) {
        let img = &self.images[item.image as usize];
        (
            item.x,
            item.y,
            item.x + img.width() as i32,
            item.y + img.height() as i32,
        )
    }

    /// Whether `item`'s rectangle contains world pixel `(x, y)`; the cheap
    /// tests on the item's corner come before the image's size is read.
    fn covers(&self, item: &Item, x: i32, y: i32) -> bool {
        if x < item.x || y < item.y {
            return false;
        }
        let (_, _, r, b) = self.bounds(item);
        x < r && y < b
    }

    /// Whether `item`'s image pixel at world pixel `(x, y)` is there and not
    /// transparent.
    fn opaque_at(&self, item: &Item, x: i32, y: i32) -> bool {
        self.covers(item, x, y)
            && self.images[item.image as usize]
                .get((x - item.x) as u32, (y - item.y) as u32)
                .is_some_and(|v| v != 0)
    }

    /// Composites the scene for `view` on the CPU.
    pub fn compose(&self, view: &View, filter: &Filter, quirks: &ComposeQuirks) -> Frame {
        self.compose_lit(view, filter, quirks, None)
    }

    /// Composites the scene for `view` on the CPU with the light-alpha
    /// values `alpha` of the view's pixels (S-0110 claims 5 and 6; `None`:
    /// 127 everywhere).
    pub fn compose_lit(
        &self,
        view: &View,
        filter: &Filter,
        quirks: &ComposeQuirks,
        alpha: Option<Vec<u8>>,
    ) -> Frame {
        let (ox, oy) = view.origin;
        let (w, h) = view.size;
        let mut frame = Frame::new(w, h, oy, view.reference);
        if let Some(a) = alpha {
            frame.set_light_alpha(a);
        }
        let mut spots = self.spots.iter().peekable();
        for (n, item) in self.items.iter().enumerate() {
            while let Some(g) = spots.next_if(|g| g.at <= n) {
                if filter.shows_spots(g) {
                    spots::apply(&mut frame, g, (ox, oy));
                }
            }
            if !filter.shows(item) {
                continue;
            }
            for item in item.copies(oy) {
                let _ = self.draw_item(&mut frame, &item, (ox, oy), quirks);
            }
        }
        for g in spots.filter(|g| filter.shows_spots(g)) {
            spots::apply(&mut frame, g, (ox, oy));
        }
        frame
    }

    /// Draws one item (one copy of a stack) into `frame` for a view with
    /// origin `(ox, oy)`, unless its bounds miss the view, and returns how
    /// many of its pixels were painted.
    fn draw_item(
        &self,
        frame: &mut Frame,
        item: &Item,
        (ox, oy): (i32, i32),
        quirks: &ComposeQuirks,
    ) -> usize {
        let (w, h) = (frame.width as i32, frame.height as i32);
        let (l, t, r, b) = self.bounds(item);
        if r <= ox || b <= oy || l >= ox + w || t >= oy + h {
            return 0;
        }
        let depth = match item.depth {
            ItemDepth::Tile {
                depth,
                reference_row,
            } => DepthSource::Tile {
                tile_depth: depth.map(|i| &self.images[i as usize]),
                reference_row: reference_row - oy,
            },
            ItemDepth::Profile {
                profile,
                bias,
                companion,
            } => DepthSource::Profile {
                profile,
                bias,
                companion: companion
                    .map(|(i, cx, cy)| (&self.signed[i as usize], cx - ox, cy - oy)),
            },
        };
        let d = Drawing {
            image: &self.images[item.image as usize],
            x: item.x - ox,
            y: item.y - oy,
            test: item.test,
            depth,
            paint: item.paint,
        };
        frame.draw(&d, &self.rows, quirks)
    }

    /// Every item (index into [`Scene::items`], draw order) whose image has a
    /// non-transparent pixel at world pixel `(x, y)`, shadows included: the
    /// debug trace of what covers a screen point.
    pub fn items_at(&self, x: i32, y: i32, top: i32) -> Vec<usize> {
        let covers = |item: &Item| {
            item.copies(top).any(|c| {
                let (l, t, r, b) = self.bounds(&c);
                if x < l || y < t || x >= r || y >= b {
                    return false;
                }
                self.images[c.image as usize]
                    .get((x - l) as u32, (y - t) as u32)
                    .is_some_and(|v| v != 0)
            })
        };
        (0..self.items.len())
            .filter(|&i| covers(&self.items[i]))
            .collect()
    }

    /// The object shown at world pixel `(x, y)` of `view`: the compositor is
    /// replayed for that one pixel, every item that `filter` shows in draw
    /// order (the copies of a stack for the view's top row, S-0109 claim 27),
    /// and the last item with an object whose pixel there passed every test
    /// of the drawing wins (S-0102 claims 9, 10, 25 and 31: a pixel that
    /// failed the depth test or lies outside its companion depth image was
    /// not drawn).
    ///
    /// design: render (#68, #69): shadows and the status extras (health bars,
    /// insignias, pips; S-0111 claim 20, a drawing on the view is not a place
    /// of its own) never supply the object, and an item without an object
    /// (an effect) drawn over an object does not hide it. Spots never write
    /// depth (S-0518 claim 20), so they are not replayed.
    pub fn object_at(
        &self,
        x: i32,
        y: i32,
        view: &View,
        filter: &Filter,
        quirks: &ComposeQuirks,
    ) -> Option<u32> {
        let top = view.origin.1;
        let candidate = |item: &Item| {
            item.object
                .filter(|_| item.paint != Paint::Shadow && item.group != Group::Status)
        };
        // design: render (#68): only an item that can supply the object and has an opaque pixel
        // at the point can win, so the replay ends at the last such item, and there is none at
        // all when no such item exists (over open ground the pick costs one cheap pass that
        // skips the items without an object, as before #68).
        let last = self.items.iter().rposition(|item| {
            candidate(item).is_some()
                && filter.shows(item)
                && item.copies(top).any(|c| self.opaque_at(&c, x, y))
        })?;
        let mut frame = Frame::new(1, 1, y, view.reference);
        let mut found = None;
        for item in &self.items[..=last] {
            // Only the items with a copy over the point can paint it.
            if !item.copies(top).any(|c| self.covers(&c, x, y)) || !filter.shows(item) {
                continue;
            }
            let candidate = candidate(item);
            for copy in item.copies(top) {
                let painted = self.draw_item(&mut frame, &copy, (x, y), quirks);
                if painted > 0 && candidate.is_some() {
                    found = candidate;
                }
            }
        }
        found
    }
}

/// Re-export for builders.
pub use compose::Profile as DepthProfile;

#[cfg(test)]
mod tests {
    use super::*;

    /// spec: S-0102 F1 (depth tied to world rows: scrolling keeps a world row's depth)
    #[test]
    fn scrolling_keeps_world_depth() {
        let mut s = Scene::default();
        let img = s.add_image(IndexedImage::new(2, 2, vec![1; 4]).unwrap());
        let row = s.add_row([0xFFFF; 256]);
        s.items.push(Item {
            image: img,
            x: 10,
            y: 20,
            test: DepthTest::Writer,
            depth: ItemDepth::Profile {
                profile: Profile::Flat,
                bias: 0,
                companion: None,
            },
            paint: Paint::Row(row),
            group: Group::Overlays,
            provisional: false,
            object: Some(7),
            stack: None,
        });
        let c = compose::DEPTH_REFERENCE;
        let a = s.compose(
            &View {
                origin: (0, 0),
                size: (32, 32),
                reference: c,
            },
            &Filter::default(),
            &ComposeQuirks::default(),
        );
        let b = s.compose(
            &View {
                origin: (5, 7),
                size: (32, 32),
                reference: c,
            },
            &Filter::default(),
            &ComposeQuirks::default(),
        );
        assert_eq!(
            a.depth[(20 * 32 + 10) as usize],
            b.depth[(13 * 32 + 5) as usize]
        );
        assert_eq!(pick(&s, 11, 21, 0), Some(7));
        assert_eq!(pick(&s, 12, 21, 0), None);
    }

    /// The debug pixel trace lists every item with a non-transparent pixel at a point, in draw
    /// order, shadows and objectless items included, where `object_at` names only the topmost
    /// object (a presentation diagnostic: it cites no behaviour).
    #[test]
    fn items_at_lists_every_covering_item() {
        let mut s = Scene::default();
        let solid = s.add_image(IndexedImage::new(2, 2, vec![1; 4]).unwrap());
        let hole = s.add_image(IndexedImage::new(2, 2, vec![0, 1, 1, 1]).unwrap());
        let row = s.add_row([0xFFFF; 256]);
        let item = |image, x, paint, object| Item {
            image,
            x,
            y: 0,
            test: DepthTest::Writer,
            depth: ItemDepth::Profile {
                profile: Profile::Flat,
                bias: 0,
                companion: None,
            },
            paint,
            group: Group::Overlays,
            provisional: false,
            object,
            stack: None,
        };
        s.items.push(item(solid, 0, Paint::Row(row), Some(1)));
        s.items.push(item(solid, 1, Paint::Shadow, Some(2)));
        s.items.push(item(hole, 0, Paint::Row(row), None));
        // (0, 0): the solid body and nothing else (the hole's pixel there is transparent).
        assert_eq!(s.items_at(0, 0, 0), vec![0]);
        // (1, 0): the body, the shadow and the objectless item; the topmost object ignores shadows.
        assert_eq!(s.items_at(1, 0, 0), vec![0, 1, 2]);
        assert_eq!(pick(&s, 1, 0, 0), Some(1));
        // Outside every image: nothing.
        assert!(s.items_at(5, 0, 0).is_empty());
    }

    /// A one-pixel item at world pixel (0, 0) with depth bias `bias` (smaller is nearer).
    fn pick_item(image: u32, paint: Paint, test: DepthTest, bias: i32, object: u32) -> Item {
        Item {
            image,
            x: 0,
            y: 0,
            test,
            depth: ItemDepth::Profile {
                profile: Profile::Flat,
                bias,
                companion: None,
            },
            paint,
            group: Group::Vehicles,
            provisional: false,
            object: Some(object),
            stack: None,
        }
    }

    /// The colour the CPU compositor shows at world pixel (0, 0).
    fn shown_at_origin(s: &Scene) -> u16 {
        s.compose(
            &View {
                origin: (0, 0),
                size: (1, 1),
                reference: compose::DEPTH_REFERENCE,
            },
            &Filter::default(),
            &ComposeQuirks::default(),
        )
        .color[0]
    }

    /// The object picked at world pixel `(x, y)` in a view whose top row is world row `top`.
    fn pick(s: &Scene, x: i32, y: i32, top: i32) -> Option<u32> {
        let view = View {
            origin: (0, top),
            size: (64, 64),
            reference: compose::DEPTH_REFERENCE,
        };
        s.object_at(x, y, &view, &Filter::default(), &ComposeQuirks::default())
    }

    /// #68: the pointer picks the object drawn at the pixel. spec: S-0102 claims 9 and 10, F2
    /// (a reader is drawn only where its depth is below the stored depth, so a farther reader
    /// after a nearer writer is not drawn there); design: render (#68) (the pick follows that
    /// drawing)
    #[test]
    fn object_at_follows_the_depth_test() {
        let mut s = Scene::default();
        let img = s.add_image(IndexedImage::new(1, 1, vec![1]).unwrap());
        let near = s.add_row([0x1111; 256]);
        let far = s.add_row([0x2222; 256]);
        s.items
            .push(pick_item(img, Paint::Row(near), DepthTest::Writer, 0, 1));
        s.items
            .push(pick_item(img, Paint::Row(far), DepthTest::Reader, 10, 2));
        assert_eq!(
            shown_at_origin(&s),
            0x1111,
            "the reader fails the depth test"
        );
        assert_eq!(pick(&s, 0, 0, 0), Some(1));
        // A nearer reader is drawn, and picked.
        s.items[1] = pick_item(img, Paint::Row(far), DepthTest::Reader, -10, 2);
        assert_eq!(shown_at_origin(&s), 0x2222);
        assert_eq!(pick(&s, 0, 0, 0), Some(2));
    }

    /// #68: a body pixel its companion depth image does not cover is not drawn, so it is not
    /// picked. spec: S-0102 claim 31, F5 (not drawn and no depth written); design: render (#68)
    #[test]
    fn object_at_skips_pixels_outside_the_companion() {
        let mut s = Scene::default();
        let img = s.add_image(IndexedImage::new(1, 1, vec![1]).unwrap());
        let row = s.add_row([0x1111; 256]);
        let other = s.add_row([0x2222; 256]);
        s.signed.push(SignedImage {
            width: 1,
            height: 1,
            values: vec![0],
        });
        s.items
            .push(pick_item(img, Paint::Row(row), DepthTest::Writer, 0, 1));
        let mut clipped = pick_item(img, Paint::Row(other), DepthTest::Writer, -10, 2);
        // The companion lies one pixel to the right of the body's only pixel.
        clipped.depth = ItemDepth::Profile {
            profile: Profile::Flat,
            bias: -10,
            companion: Some((0, 1, 0)),
        };
        s.items.push(clipped);
        assert_eq!(shown_at_origin(&s), 0x1111);
        assert_eq!(pick(&s, 0, 0, 0), Some(1));
    }

    /// #69: health bars, brackets, pips and group numbers are drawings on the view, not a place
    /// of their own: the pick finds the body under them. spec: S-0111 Terms (pointer object: the
    /// object the pointer is on) and claim 20 (a drawing on the view is not a place of its own)
    #[test]
    fn object_at_passes_over_status_extras() {
        let mut s = Scene::default();
        let img = s.add_image(IndexedImage::new(1, 1, vec![1]).unwrap());
        let row = s.add_row([0x1111; 256]);
        s.items
            .push(pick_item(img, Paint::Row(row), DepthTest::Reader, 0, 1));
        let mut bar = pick_item(img, Paint::Row(row), DepthTest::Ignore, 0, 2);
        bar.group = Group::Status;
        s.items.push(bar);
        assert_eq!(pick(&s, 0, 0, 0), Some(1));
        // A status extra alone over empty ground names no object.
        s.items.remove(0);
        assert_eq!(pick(&s, 0, 0, 0), None);
    }

    /// spec: S-0109 claim 27 (C = 1 when pᵧ − ⌊h/2⌋ < 0, else ⌊(pᵧ − ⌊h/2⌋) / h⌋ + 2, with pᵧ
    /// from the view's top row: the highest copy's unshifted centre lies strictly above that row;
    /// copies lowest first, h rows apart); S-0102 claim 13 and D40 (copy j's bias reduced by
    /// exactly j·h)
    #[test]
    fn stack_copies_follow_the_view_top() {
        let s = Stack {
            spacing: 10,
            base_row: 250,
        };
        assert_eq!(s.copies(0), 27);
        assert_eq!(s.copies(240), 3);
        assert_eq!(s.copies(241), 2);
        assert_eq!(s.copies(250), 2);
        assert_eq!(s.copies(251), 1);
        assert_eq!(s.copies(400), 1);
        let s11 = Stack {
            spacing: 11,
            base_row: 250,
        };
        assert_eq!(s11.copies(0), 24);
        for st in [s, s11] {
            for top in -40..300 {
                let c = st.copies(top);
                let centre = |j: i32| st.base_row - j * st.spacing;
                assert!(centre(c - 1) < top, "highest copy above the top row");
                if c > 1 {
                    assert!(centre(c - 2) >= top, "no copy too many");
                }
            }
        }
        let item = Item {
            image: 0,
            x: 3,
            y: 100,
            test: DepthTest::Reader,
            depth: ItemDepth::Profile {
                profile: Profile::Upright,
                bias: -50,
                companion: None,
            },
            paint: Paint::Shadow,
            group: Group::Buildings,
            provisional: false,
            object: None,
            stack: Some(s),
        };
        let copies: Vec<(i32, i32, i32)> = item
            .copies(240)
            .map(|c| match c.depth {
                ItemDepth::Profile { bias, .. } => (c.x, c.y, bias),
                ItemDepth::Tile { .. } => unreachable!(),
            })
            .collect();
        assert_eq!(copies, [(3, 100, -50), (3, 90, -60), (3, 80, -70)]);
        let single = Item {
            stack: None,
            ..item
        };
        assert_eq!(single.copies(-1000).count(), 1);
        // D40's marker: h = 12 gives copy 1 the bias −50 − 12 and copy 2 −50 − 24, not the
        // original's excessive −50 − 18 and −50 − 36 (S-0102 claim 13, a drawing-only departure).
        let d40 = Item {
            stack: Some(Stack {
                spacing: 12,
                base_row: 300,
            }),
            ..item
        };
        let biases: Vec<i32> = d40
            .copies(0)
            .take(3)
            .map(|c| match c.depth {
                ItemDepth::Profile { bias, .. } => bias,
                ItemDepth::Tile { .. } => unreachable!(),
            })
            .collect();
        assert_eq!(biases, [-50, -62, -74]);
    }

    /// spec: S-0109 claim 27 (the composite draws as many copies as the view's top row allows)
    #[test]
    fn compose_draws_the_stack_copies() {
        let mut s = Scene::default();
        let img = s.add_image(IndexedImage::new(1, 1, vec![1]).unwrap());
        let row = s.add_row([0xFFFF; 256]);
        s.items.push(Item {
            image: img,
            x: 0,
            y: 20,
            test: DepthTest::Ignore,
            depth: ItemDepth::Profile {
                profile: Profile::Upright,
                bias: 0,
                companion: None,
            },
            paint: Paint::Row(row),
            group: Group::Buildings,
            provisional: false,
            object: Some(1),
            stack: Some(Stack {
                spacing: 5,
                base_row: 20,
            }),
        });
        let painted = |top: i32| {
            let f = s.compose(
                &View {
                    origin: (0, top),
                    size: (1, 40),
                    reference: compose::DEPTH_REFERENCE,
                },
                &Filter::default(),
                &ComposeQuirks::default(),
            );
            (0..40)
                .filter(|&r| f.color[r as usize] == 0xFFFF)
                .map(|r| r + top)
                .collect::<Vec<i32>>()
        };
        // Top row 0: C = 20/5 + 2 = 6; the sixth copy (row −5) lies outside the view.
        assert_eq!(painted(0), [0, 5, 10, 15, 20]);
        assert_eq!(painted(-10), [-10, -5, 0, 5, 10, 15, 20]);
        // Top row 21: C = 1.
        assert_eq!(painted(21), Vec::<i32>::new());
        assert_eq!(painted(16), [20]);
        assert_eq!(pick(&s, 0, 10, 0), Some(1));
        assert_eq!(pick(&s, 0, 10, 16), None);
    }
}
