//! The image slots of the live game's debris pieces (S-0317 claims 22, 24
//! and 27).
//!
//! A piece's model turns smoothly and its light turns with it, so each
//! piece needs its own body and shadow image, drawn again whenever its pose
//! changes (S-0317 claims 22, 24 and 27). The scene keeps every image it is
//! given, so instead of adding an image per pose each piece holds one image
//! slot for its body and one for its shadow, rewritten in place
//! ([`Scene::replace_image`]), and a piece that leaves play gives its slots
//! back for later pieces. A slot is larger than the drawing it first holds,
//! so that the turned model usually still fits; a drawing that does not fit
//! moves to a larger slot. The images held are bounded by the pieces in
//! play at once. (Our design: S-0317 fixes what is drawn, not how it is
//! kept.)

use std::collections::{BTreeMap, BTreeSet};

use glasnost_fmt::image::IndexedImage;
use glasnost_render::scene::Scene;
use glasnost_render::voxel::PartImage;

/// A scene image held as a slot, with its size.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Slot {
    image: u32,
    size: (u32, u32),
}

/// One image a piece holds: its slot, and where its current drawing is
/// placed (the scene image and the offset from the draw point); `None`
/// when the current pose draws nothing.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Held {
    slot: Option<Slot>,
    /// The current drawing: scene image and offset from the draw point.
    pub placed: Option<(u32, [i32; 2])>,
}

/// The images of one piece for the pose `key`.
#[derive(Clone, Debug, PartialEq)]
pub struct PieceImages<K> {
    /// The pose they were drawn for.
    pub key: K,
    /// The body.
    pub body: Held,
    /// The shadow.
    pub shadow: Held,
}

/// The pieces' image slots, by the piece's number.
#[derive(Clone, Debug)]
pub struct PiecePool<K> {
    pieces: BTreeMap<u32, PieceImages<K>>,
    free: Vec<Slot>,
    seen: BTreeSet<u32>,
}

impl<K> Default for PiecePool<K> {
    fn default() -> Self {
        Self {
            pieces: BTreeMap::new(),
            free: Vec::new(),
            seen: BTreeSet::new(),
        }
    }
}

impl<K: PartialEq> PiecePool<K> {
    /// The images piece `id` holds for the pose `key`, `None` when it holds
    /// none or holds them for another pose. Marks the piece as seen.
    pub fn current(&mut self, id: u32, key: &K) -> Option<&PieceImages<K>> {
        self.seen.insert(id);
        self.pieces.get(&id).filter(|p| &p.key == key)
    }

    /// Stores the drawings of piece `id` for the pose `key` in its slots,
    /// or in other slots when they do not fit. Marks the piece as seen.
    pub fn store(
        &mut self,
        scene: &mut Scene,
        id: u32,
        key: K,
        body: Option<&PartImage>,
        shadow: Option<&PartImage>,
    ) -> &PieceImages<K> {
        self.seen.insert(id);
        let (b, s) = self
            .pieces
            .remove(&id)
            .map_or_else(Default::default, |p| (p.body, p.shadow));
        let body = self.put(scene, b, body);
        let shadow = self.put(scene, s, shadow);
        self.pieces
            .entry(id)
            .or_insert(PieceImages { key, body, shadow })
    }

    /// Frees the slots of every piece not seen since the last call.
    pub fn release_unseen(&mut self) {
        let seen = std::mem::take(&mut self.seen);
        let gone: Vec<u32> = self
            .pieces
            .keys()
            .filter(|id| !seen.contains(id))
            .copied()
            .collect();
        for id in gone {
            if let Some(p) = self.pieces.remove(&id) {
                self.free.extend(p.body.slot);
                self.free.extend(p.shadow.slot);
            }
        }
    }

    /// The number of images the pool holds, in use or free.
    pub fn images(&self) -> usize {
        let held = self
            .pieces
            .values()
            .map(|p| usize::from(p.body.slot.is_some()) + usize::from(p.shadow.slot.is_some()))
            .sum::<usize>();
        held + self.free.len()
    }

    fn put(&mut self, scene: &mut Scene, held: Held, img: Option<&PartImage>) -> Held {
        let Some(img) = img.filter(|i| i.width > 0 && i.height > 0) else {
            return Held {
                slot: held.slot,
                placed: None,
            };
        };
        let need = (img.width, img.height);
        let fits = |s: &Slot| s.size.0 >= need.0 && s.size.1 >= need.1;
        let slot = match held.slot {
            Some(s) if fits(&s) => Some(s),
            other => {
                self.free.extend(other);
                self.take(need)
            }
        };
        let slot = match slot {
            Some(s) => {
                let replaced = padded(img, s.size).is_some_and(|i| scene.replace_image(s.image, i));
                if !replaced {
                    return Held {
                        slot: Some(s),
                        placed: None,
                    };
                }
                s
            }
            None => {
                let size = room(need);
                let Some(i) = padded(img, size) else {
                    return Held::default();
                };
                Slot {
                    image: scene.add_image(i),
                    size,
                }
            }
        };
        let offset = [img.offset[0], img.offset[1]].map(|o| o as i32);
        Held {
            slot: Some(slot),
            placed: Some((slot.image, offset)),
        }
    }

    /// The smallest free slot that `need` fits in, taken from the free
    /// list.
    fn take(&mut self, need: (u32, u32)) -> Option<Slot> {
        let at = self
            .free
            .iter()
            .enumerate()
            .filter(|(_, s)| s.size.0 >= need.0 && s.size.1 >= need.1)
            .min_by_key(|(_, s)| u64::from(s.size.0) * u64::from(s.size.1))
            .map(|(i, _)| i)?;
        Some(self.free.swap_remove(at))
    }
}

/// The size of a new slot for a drawing of size `need`: a square half as
/// large again as its larger side, so that the turned model usually fits.
fn room(need: (u32, u32)) -> (u32, u32) {
    let side = need.0.max(need.1);
    let side = side + side / 2 + 2;
    (side, side)
}

/// `img` in the top-left corner of a transparent image of `size`.
fn padded(img: &PartImage, size: (u32, u32)) -> Option<IndexedImage> {
    let (w, h) = (img.width as usize, img.height as usize);
    let sw = size.0 as usize;
    let mut px = vec![0u8; sw * size.1 as usize];
    for (y, row) in img.pixels.chunks_exact(w.max(1)).take(h).enumerate() {
        px[y * sw..y * sw + w].copy_from_slice(row);
    }
    IndexedImage::new(size.0, size.1, px).ok()
}
