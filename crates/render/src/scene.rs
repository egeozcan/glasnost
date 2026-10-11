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
}

impl Scene {
    /// Adds an image and returns its index.
    pub fn add_image(&mut self, image: IndexedImage) -> u32 {
        self.images.push(image);
        (self.images.len() - 1) as u32
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

    /// Composites the scene for `view` on the CPU.
    pub fn compose(&self, view: &View, filter: &Filter, quirks: &ComposeQuirks) -> Frame {
        let (ox, oy) = view.origin;
        let (w, h) = view.size;
        let mut frame = Frame::new(w, h, oy, view.reference);
        for item in &self.items {
            if !filter.shows(item) {
                continue;
            }
            let (l, t, r, b) = self.bounds(item);
            if r <= ox || b <= oy || l >= ox + w as i32 || t >= oy + h as i32 {
                continue;
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
            frame.draw(&d, &self.rows, quirks);
        }
        frame
    }

    /// The topmost item under world pixel `(x, y)` whose image pixel there is
    /// not transparent, among items with an object, searching from the last
    /// drawn.
    pub fn object_at(&self, x: i32, y: i32, filter: &Filter) -> Option<u32> {
        self.items.iter().rev().find_map(|item| {
            let object = item.object?;
            if !filter.shows(item) || item.paint == Paint::Shadow {
                return None;
            }
            let (l, t, r, b) = self.bounds(item);
            if x < l || y < t || x >= r || y >= b {
                return None;
            }
            let img = &self.images[item.image as usize];
            let v = img.get((x - l) as u32, (y - t) as u32)?;
            (v != 0).then_some(object)
        })
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
        });
        let c = Frame::reset_reference(0);
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
        assert_eq!(s.object_at(11, 21, &Filter::default()), Some(7));
        assert_eq!(s.object_at(12, 21, &Filter::default()), None);
    }
}
