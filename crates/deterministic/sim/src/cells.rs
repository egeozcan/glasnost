//! Per-cell state of play: terrain, passability category, the occupancy
//! record of each cell layer and the objects each layer lists (S-0209).

use glasnost_map::{Cell, CellGrid, CellXY, Footprint};
use serde::{Deserialize, Serialize};

use crate::world::{HouseId, Layer, ObjectId};

/// Standing positions (S-0208 table P).
pub mod position {
    /// Centre.
    pub const CENTRE: u8 = 0; // spec: S-0208 table P (0 centre)
    /// Top.
    pub const TOP: u8 = 1; // spec: S-0208 table P (1 top)
    /// Right.
    pub const RIGHT: u8 = 2; // spec: S-0208 table P (2 right)
    /// Left.
    pub const LEFT: u8 = 3; // spec: S-0208 table P (3 left)
    /// Bottom.
    pub const BOTTOM: u8 = 4; // spec: S-0208 table P (4 bottom)
    /// The three side positions (S-0208 Terms).
    pub const SIDES: [u8; 3] = [RIGHT, LEFT, BOTTOM]; // spec: S-0208 Terms (side positions)
}

/// The bridge axis of a cell (S-0212 Terms).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Axis {
    /// No axis.
    #[default]
    None,
    /// Along cell x.
    X,
    /// Along cell y.
    Y,
}

/// A bridge link (S-0212 Terms and claim 3): a bridge span of S-0010 claim
/// 25 with its two end cells.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BridgeLink {
    /// The first end: the span's start cell.
    pub first: CellXY,
    /// The second end: the span's far end.
    pub second: CellXY,
    /// Whether the link's line runs along cell x (a span running east);
    /// otherwise along cell y (a span running south).
    pub along_x: bool,
    /// Whether the link is enabled (S-0212 claim 3).
    pub enabled: bool,
}

impl BridgeLink {
    /// Whether `c` lies within `k` cells of the link (S-0212 Terms): between
    /// the two end cells along the line, both included, and at most `k`
    /// cells to the side of it.
    pub fn within(&self, c: CellXY, k: i32) -> bool {
        let (a, b, side, along) = if self.along_x {
            (self.first.x, self.second.x, c.y - self.first.y, c.x)
        } else {
            (self.first.y, self.second.y, c.x - self.first.x, c.y)
        };
        (a.min(b)..=a.max(b)).contains(&along) && side.abs() <= k
    }

    /// The signed distance by which `c` lies to the side of the first end
    /// across the line.
    pub fn side(&self, c: CellXY) -> i32 {
        if self.along_x {
            c.y - self.first.y
        } else {
            c.x - self.first.x
        }
    }
}

/// What marked a standing position.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Marker {
    /// An infantry unit and its house.
    Infantry(ObjectId, HouseId),
    /// A terrain object (S-0209 claim 1).
    Terrain(ObjectId),
    /// An infantry-making animation and its owner, if any (S-0611 claim
    /// 8).
    Anim(ObjectId, Option<HouseId>),
}

/// A mark with the order in which it was made.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Mark {
    /// Who marked.
    pub by: Marker,
    /// Marking order (for `quirks.occupancy_house_stale` off).
    pub seq: u32,
}

/// The occupancy record of one cell layer (S-0209 Terms).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Occupancy {
    /// The five standing positions.
    pub marks: [Option<Mark>; 5], // spec: S-0209 claim 1 (one mark per standing position; five positions, S-0208 table P)
    /// Vehicles that set the vehicle-present flag (S-0209 claim 1).
    pub vehicles: u8,
    /// Buildings that set the building-present flag.
    pub buildings: u8,
    /// Objects that set the object-present flag.
    pub others: u8,
    /// The infantry house.
    pub house: Option<HouseId>,
}

impl Occupancy {
    /// Whether a standing position is taken.
    pub fn taken(&self, p: u8) -> bool {
        self.marks[usize::from(p)].is_some()
    }

    /// Whether the three side positions are all taken (S-0209 claim 2).
    pub fn full(&self) -> bool {
        position::SIDES.iter().all(|&p| self.taken(p))
    }

    /// Whether the record is empty as a whole (S-0212 claim 25).
    pub fn empty(&self) -> bool {
        self.marks.iter().all(Option::is_none)
            && self.vehicles == 0
            && self.buildings == 0
            && self.others == 0
    }

    /// The vehicle-present flag.
    pub fn vehicle_present(&self) -> bool {
        self.vehicles > 0
    }
}

/// The switches the occupancy record reads.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OccupancyQuirks {
    /// `quirks.occupancy_house_side_positions_only` (S-0209 Edge cases).
    pub side_positions_only: bool,
    /// `quirks.occupancy_house_stale` (S-0209 Edge cases).
    pub stale: bool,
}

/// Per-cell state of play.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cells {
    /// Terrain and overlay of every legal cell.
    pub grid: CellGrid,
    /// Passability category (S-0211 claim 11) by dense index.
    pub category: Vec<u8>,
    /// Occupancy records (ground, deck) by dense index.
    pub occ: Vec<[Occupancy; 2]>, // spec: S-0209 Terms (two cell layers: ground and bridge deck)
    /// Listed objects (ground, deck) by dense index, in listing order
    /// (S-0209 claim 13).
    pub lists: Vec<[Vec<ObjectId>; 2]>, // spec: S-0209 Terms (two cell layers)
    /// Whether a cell is a high-bridge cell, with a bridge deck (S-0211
    /// claim 15).
    pub deck: Vec<bool>,
    /// The deck-entry mark of each cell (S-0212 Terms).
    pub deck_entry: Vec<bool>,
    /// The bridge axis of each cell (S-0212 Terms).
    pub bridge_axis: Vec<Axis>,
    /// The bridge-body mark of each cell (S-0010 Terms, claim 25).
    pub bridge_body: Vec<bool>,
    /// Whether a high-bridge overlay has marked the cell as its own
    /// section cell (S-0211 claim 15 (a); S-0511 Terms, *responsible
    /// section*). Historical: never cleared.
    pub section_self: Vec<bool>,
    /// The section cell of the last high-bridge overlay that marked the
    /// cell as one of its covered cells (S-0211 claim 15 (a)). Historical:
    /// a later covering mark replaces it, nothing clears it.
    pub covered_by: Vec<Option<CellXY>>,
    /// The bridge links, in creation order (S-0212 claim 3).
    pub links: Vec<BridgeLink>,
    /// The cells of ramp tiles showing their alternate (damaged) look
    /// (S-0511 claim 4); drawing only.
    #[serde(default)]
    pub alternate: std::collections::BTreeSet<CellXY>,
    /// The height levels the cells of collapsed piers had before their
    /// collapse (S-0511 claim 6), for `quirks.pier_repair_partial_height`
    /// off (claim 19).
    #[serde(default)]
    pub pier_levels: std::collections::BTreeMap<CellXY, i8>,
    /// Marking counter.
    pub mark_seq: u32,
    /// The owner of each cell's wall (S-0210 claim 14); `None`: no owner.
    pub wall_owner: Vec<Option<HouseId>>,
    /// Whether a cell has started its tile animation since the map load's
    /// second recalculation pass (S-0010 claim 19: at most one until the
    /// next map load).
    pub anim_started: Vec<bool>,
    /// While a map loads, before its second recalculation pass: whether a
    /// cell has been recalculated yet (S-0202 table, a tile animation starts
    /// at the first recalculation of its cell). Not state: `None` after.
    #[serde(skip)]
    pub anim_first: Option<Vec<bool>>,
}

impl Cells {
    /// Whether the per-cell tables hold together: the grid is consistent
    /// and every table indexed by dense index has one entry per cell. A
    /// world decoded from bytes may not (#157). Every field is named, so a
    /// new table must be placed here.
    pub fn is_consistent(&self) -> bool {
        let Cells {
            grid,
            category,
            occ,
            lists,
            deck,
            deck_entry,
            bridge_axis,
            bridge_body,
            section_self,
            covered_by,
            links: _,
            alternate: _,
            pier_levels: _,
            mark_seq: _,
            wall_owner,
            anim_started,
            anim_first: _,
        } = self;
        let n = grid.len();
        grid.is_consistent()
            && [
                category.len(),
                occ.len(),
                lists.len(),
                deck.len(),
                deck_entry.len(),
                bridge_axis.len(),
                bridge_body.len(),
                section_self.len(),
                covered_by.len(),
                wall_owner.len(),
                anim_started.len(),
            ]
            .iter()
            .all(|&l| l == n)
    }
}

/// The high-bridge overlay indices and their marking directions (S-0211
/// claim 15): indices 24 and 237 mark along step direction 0 (map north),
/// 25 and 238 along step direction 6 (map west).
// spec: S-0211 claim 15, Constants (high-bridge overlay indices and marking directions)
const HIGH_BRIDGE_OVERLAYS: [(u8, (i32, i32)); 4] =
    [(24, (0, -1)), (237, (0, -1)), (25, (-1, 0)), (238, (-1, 0))];

fn li(l: Layer) -> usize {
    match l {
        Layer::Ground => 0,
        Layer::Deck => 1,
    }
}

/// The house of the infantry unit that marked most recently among those
/// still holding a position in the record, or none (S-0209 Edge cases).
fn most_recent_house(o: &Occupancy) -> Option<HouseId> {
    o.marks
        .iter()
        .flatten()
        .filter_map(|m| match m.by {
            Marker::Infantry(_, h) | Marker::Anim(_, Some(h)) => Some((m.seq, h)),
            Marker::Terrain(_) | Marker::Anim(_, None) => None,
        })
        .max()
        .map(|(_, h)| h)
}

/// The house-clearing rule after an infantry mark is cleared (S-0209
/// claim 1 and Edge cases).
fn settle_house(o: &mut Occupancy, q: OccupancyQuirks) {
    // spec: S-0209 claim 1 (the house goes to none only when no side position stays taken)
    let none_left = if q.side_positions_only {
        !position::SIDES.iter().any(|&p| o.taken(p))
    } else {
        // spec: S-0209 Edge cases (switch off: cleared only when no standing position is taken)
        o.marks.iter().all(Option::is_none)
    };
    if none_left {
        o.house = None;
    } else if !q.stale {
        // spec: S-0209 Edge cases (`quirks.occupancy_house_stale` off: the most recent infantry still holding a position)
        o.house = most_recent_house(o);
    }
}

impl Cells {
    /// Cells for a grid.
    pub fn new(grid: CellGrid) -> Self {
        let n = grid.len();
        Self {
            grid,
            category: vec![0; n],
            occ: vec![[Occupancy::default(); 2]; n], // spec: S-0209 Terms (two cell layers)
            lists: vec![[Vec::new(), Vec::new()]; n],
            deck: vec![false; n],
            deck_entry: vec![false; n],
            bridge_axis: vec![Axis::None; n],
            bridge_body: vec![false; n],
            section_self: vec![false; n],
            covered_by: vec![None; n],
            links: Vec::new(),
            alternate: Default::default(),
            pier_levels: Default::default(),
            mark_seq: 0,
            wall_owner: vec![None; n],
            anim_started: vec![false; n],
            anim_first: None,
        }
    }

    /// Whether `c` has the elevation-deck mark (S-0307 Terms): it holds a
    /// high-bridge overlay itself; the neighbours that the overlay's
    /// bridge marking reaches do not get it.
    pub fn elevation_deck(&self, c: CellXY) -> bool {
        // spec: S-0307 Terms (elevation-deck mark: an intact high-bridge overlay sets it on its own cell only)
        self.cell(c)
            .is_some_and(|cell| HIGH_BRIDGE_OVERLAYS.iter().any(|h| h.0 == cell.overlay))
    }

    /// An overlay of `overlay` placed in `c` marks its high-bridge cells
    /// (S-0211 claim 15): for a high-bridge overlay index, the cell one step
    /// against its marking direction d, its own cell, and the cells one and
    /// two steps along d. Marking looks at nothing in the marked cells. The
    /// same four cells get the overlay's bridge axis, and the first three the
    /// deck-entry mark, which the fourth loses (S-0212 Terms); called in
    /// placement order, the overlay placed last decides.
    pub fn mark_high_bridge(&mut self, c: CellXY, overlay: u8) {
        let Some(&(_, (dx, dy))) = HIGH_BRIDGE_OVERLAYS.iter().find(|h| h.0 == overlay) else {
            return;
        };
        // spec: S-0212 Terms (bridge axis: along cell x for marking direction 0, along cell y for direction 6)
        let axis = if dx == 0 { Axis::X } else { Axis::Y };
        // spec: S-0211 claim 15 (four cells: one step against d, its own, one and two steps along d)
        for k in [-1, 0, 1, 2] {
            if let Some(i) = self.idx(CellXY::new(c.x + k * dx, c.y + k * dy)) {
                self.deck[i] = true;
                self.bridge_axis[i] = axis;
                // spec: S-0212 Terms (deck-entry mark set one step against d, on its own cell and one step along d; cleared two steps along d)
                self.deck_entry[i] = k < 2;
                if k == 0 {
                    // spec: S-0211 claim 15 (a) (a cell marked as a section's own cell selects itself, even after another section covers it)
                    self.section_self[i] = true;
                } else {
                    // spec: S-0211 claim 15 (a) (otherwise the last section that marked it as a covered cell; a later covering mark replaces it)
                    self.covered_by[i] = Some(c);
                }
            }
        }
    }

    /// The responsible section cell of `c` (S-0211 claim 15 (a); S-0511
    /// Terms and claim 2): `c` itself when it was marked as a section cell,
    /// else the last section that marked it as a covered cell.
    pub fn responsible(&self, c: CellXY) -> Option<CellXY> {
        let i = self.idx(c)?;
        // spec: S-0511 claim 2 (the cell itself when it retains the section-cell marking, else the last section to mark it as a covered cell)
        if self.section_self[i] {
            Some(c)
        } else {
            self.covered_by[i]
        }
    }

    /// A lost section's four cells lose their high-bridge marking and gain
    /// the bridge-body mark (S-0010 claim 25, *In play*; S-0511 claim 5);
    /// their bridge axis stays.
    pub fn unmark_high_bridge(&mut self, cells: &[CellXY]) {
        for &c in cells {
            if let Some(i) = self.idx(c) {
                // spec: S-0010 claim 25 (In play: the four cells receive the bridge-body mark as their high-bridge marking is cleared)
                self.deck[i] = false;
                self.deck_entry[i] = false;
                self.bridge_body[i] = true;
            }
        }
    }

    /// A placed section's four cells lose the bridge-body mark (S-0010
    /// claim 25, *In play*; S-0511 claim 19); [`Self::mark_high_bridge`]
    /// marks them.
    pub fn clear_body(&mut self, cells: &[CellXY]) {
        for &c in cells {
            if let Some(i) = self.idx(c) {
                // spec: S-0010 claim 25 (In play: repair clears the bridge-body mark on those four cells)
                self.bridge_body[i] = false;
            }
        }
    }

    /// Whether `c` has the bridge-body mark (S-0010 claim 25).
    pub fn body(&self, c: CellXY) -> bool {
        self.idx(c).is_some_and(|i| self.bridge_body[i])
    }

    /// Whether `c` is a high-bridge cell (S-0211 claim 15), a bridge cell
    /// of S-0212.
    pub fn is_bridge(&self, c: CellXY) -> bool {
        self.idx(c).is_some_and(|i| self.deck[i])
    }

    /// Whether `c` is a bridge cell with the deck-entry mark (S-0212 Terms).
    pub fn deck_entry_at(&self, c: CellXY) -> bool {
        self.idx(c)
            .is_some_and(|i| self.deck[i] && self.deck_entry[i])
    }

    /// The bridge axis of `c` (S-0212 Terms).
    pub fn axis(&self, c: CellXY) -> Axis {
        self.idx(c).map_or(Axis::None, |i| self.bridge_axis[i])
    }

    /// The map's footprint.
    pub fn footprint(&self) -> &Footprint {
        self.grid.footprint()
    }

    /// The farthest square ring around `c` that can hold a legal cell:
    /// every legal cell lies in the coordinate square 0 … side − 1 (S-0009
    /// claim 2), so a ring of a larger radius holds none and a search that
    /// skips cells that are not legal finds nothing there (#401, #386).
    pub fn ring_limit(&self, c: CellXY) -> i32 {
        let hi = i64::from(self.footprint().coordinate_side()) - 1;
        let (x, y) = (i64::from(c.x), i64::from(c.y));
        let far = x.abs().max((hi - x).abs()).max(y.abs()).max((hi - y).abs());
        i32::try_from(far).unwrap_or(i32::MAX)
    }

    /// The dense index of a legal cell.
    pub fn idx(&self, c: CellXY) -> Option<usize> {
        self.grid.index(c)
    }

    /// A cell's terrain.
    pub fn cell(&self, c: CellXY) -> Option<&Cell> {
        self.grid.get(c)
    }

    /// Whether a legal cell is inside the usable area (S-0211 Terms; S-0009
    /// claim 4 with the cell's height and slope).
    pub fn usable(&self, c: CellXY) -> bool {
        match self.grid.get(c) {
            Some(cell) => self
                .footprint()
                .is_usable(c, i32::from(cell.level), cell.slope != 0),
            None => false,
        }
    }

    /// A cell layer's occupancy record.
    pub fn occ(&self, c: CellXY, l: Layer) -> Option<&Occupancy> {
        self.idx(c).map(|i| &self.occ[i][li(l)])
    }

    /// A cell layer's occupancy record, for writing.
    pub fn occ_mut(&mut self, c: CellXY, l: Layer) -> Option<&mut Occupancy> {
        let i = self.idx(c)?;
        Some(&mut self.occ[i][li(l)])
    }

    /// The objects a cell layer lists, in listing order.
    pub fn list(&self, c: CellXY, l: Layer) -> &[ObjectId] {
        match self.idx(c) {
            Some(i) => &self.lists[i][li(l)],
            None => &[],
        }
    }

    /// An object enters a cell layer (S-0209 claim 13): a non-building goes
    /// to the front, a building to the end.
    pub fn enter(&mut self, c: CellXY, l: Layer, id: ObjectId, building: bool) {
        if let Some(i) = self.idx(c) {
            let list = &mut self.lists[i][li(l)];
            if list.contains(&id) {
                return;
            }
            // spec: S-0209 claim 13 (non-buildings most recent first, then buildings in placement order)
            if building {
                list.push(id);
            } else {
                list.insert(0, id);
            }
        }
    }

    /// Puts an object back into a cell layer's list at index `at`, where
    /// [`Self::leave`] took it from.
    pub fn restore(&mut self, c: CellXY, l: Layer, id: ObjectId, at: usize) {
        if let Some(i) = self.idx(c) {
            let list = &mut self.lists[i][li(l)];
            if !list.contains(&id) {
                list.insert(at.min(list.len()), id);
            }
        }
    }

    /// An object leaves both cell layers of a cell (an aircraft's listing
    /// moves between them, S-0209 claim 13).
    pub fn leave_both(&mut self, c: CellXY, id: ObjectId) {
        self.leave(c, Layer::Ground, id);
        self.leave(c, Layer::Deck, id);
    }

    /// An object leaves a cell layer.
    pub fn leave(&mut self, c: CellXY, l: Layer, id: ObjectId) {
        if let Some(i) = self.idx(c) {
            self.lists[i][li(l)].retain(|o| *o != id);
        }
    }

    /// An infantry unit marks a standing position, setting the infantry
    /// house (S-0209 claim 1).
    pub fn mark_infantry(&mut self, c: CellXY, l: Layer, p: u8, id: ObjectId, house: HouseId) {
        self.mark_seq += 1;
        let seq = self.mark_seq;
        if let Some(o) = self.occ_mut(c, l) {
            o.marks[usize::from(p)] = Some(Mark {
                by: Marker::Infantry(id, house),
                seq,
            });
            // spec: S-0209 claim 1 (the most recent mark's house wins)
            o.house = Some(house);
        }
    }

    /// An infantry-making animation marks standing position `p` taken for
    /// its owner `house` in one of its updates (S-0611 claim 8). With
    /// `quirks.infantry_animation_occupancy` (`seize`) on, the mark replaces
    /// whatever holds the position and the cell's infantry house becomes the
    /// owner. Off: it marks only a position that is free or already its own,
    /// and sets the infantry house only while no other infantry mark is in
    /// the record.
    pub fn mark_anim(
        &mut self,
        c: CellXY,
        l: Layer,
        p: u8,
        id: ObjectId,
        house: Option<HouseId>,
        seize: bool,
    ) {
        let Some(o) = self.occ(c, l) else { return };
        let own = |m: &Option<Mark>| matches!(m, Some(Mark { by: Marker::Anim(who, _), .. }) if *who == id);
        let free_or_own = o.marks[usize::from(p)].is_none() || own(&o.marks[usize::from(p)]);
        let others = o.marks.iter().any(|m| {
            matches!(
                m,
                Some(Mark {
                    by: Marker::Infantry(..),
                    ..
                })
            )
        });
        if !seize && !free_or_own {
            // spec: S-0611 Edge cases (quirks.infantry_animation_occupancy off: it holds the position only while no other unit holds it)
            return;
        }
        self.mark_seq += 1;
        let seq = self.mark_seq;
        if let Some(o) = self.occ_mut(c, l) {
            // spec: S-0611 claim 8 (in each of its updates its position is marked taken in the occupancy record of its surface)
            o.marks[usize::from(p)] = Some(Mark {
                by: Marker::Anim(id, house),
                seq,
            });
            if seize || !others {
                // spec: S-0611 claim 8 (the cell's infantry house becomes the animation's owner); Edge cases (off: never while other infantry stands in the cell)
                o.house = house;
            }
        }
    }

    /// An infantry unit's mark is cleared (S-0209 claim 1 and Edge cases).
    pub fn clear_infantry(&mut self, c: CellXY, l: Layer, id: ObjectId, q: OccupancyQuirks) {
        let Some(o) = self.occ_mut(c, l) else {
            return;
        };
        let mut cleared = false;
        for m in o.marks.iter_mut() {
            if matches!(m, Some(Mark { by: Marker::Infantry(who, _) | Marker::Anim(who, _), .. }) if *who == id)
            {
                *m = None;
                cleared = true;
            }
        }
        if cleared {
            settle_house(o, q);
        }
    }

    /// Standing position `p` of a cell layer is cleared whoever marked it,
    /// as an infantry unit clears its entry by its actual position (S-0208
    /// claim 19, `quirks.teleport_infantry_ground_reservation`); the
    /// house-clearing rule of an infantry mark applies (S-0209 claim 1).
    pub fn clear_position(&mut self, c: CellXY, l: Layer, p: u8, q: OccupancyQuirks) {
        let Some(o) = self.occ_mut(c, l) else {
            return;
        };
        if let Some(m) = o.marks.get_mut(usize::from(p)) {
            *m = None;
        }
        settle_house(o, q);
    }

    /// A terrain object marks the side positions of its pattern (S-0209
    /// claim 1, Terms *side-position pattern*).
    pub fn mark_terrain(&mut self, c: CellXY, id: ObjectId, pattern: i32) {
        self.mark_seq += 1;
        let seq = self.mark_seq;
        if let Some(o) = self.occ_mut(c, Layer::Ground) {
            for (bit, p) in [
                (1, position::RIGHT),
                (2, position::LEFT),   // spec: S-0209 Terms (pattern bit 2: left)
                (4, position::BOTTOM), // spec: S-0209 Terms (pattern bit 4: bottom)
            ] {
                // spec: S-0209 Terms (side-position pattern bits 1, 2 and 4)
                // spec: S-0209 Terms (right ⇔ ⌊p/1⌋ mod 2 = 1, left ⇔ ⌊p/2⌋ mod 2 = 1, bottom ⇔ ⌊p/4⌋ mod 2 = 1)
                if pattern.div_euclid(bit).rem_euclid(2) == 1 {
                    o.marks[usize::from(p)] = Some(Mark {
                        by: Marker::Terrain(id),
                        seq,
                    });
                }
            }
        }
    }

    /// A terrain object's marks are cleared. The infantry house is left as
    /// it is, even when the last side position is freed (S-0209 claim 1);
    /// with `quirks.occupancy_house_stale` off it is recomputed (S-0209
    /// Edge cases).
    pub fn clear_terrain(&mut self, c: CellXY, id: ObjectId, q: OccupancyQuirks) {
        if let Some(o) = self.occ_mut(c, Layer::Ground) {
            for m in o.marks.iter_mut() {
                if matches!(m, Some(Mark { by: Marker::Terrain(who), .. }) if *who == id) {
                    *m = None;
                }
            }
            if !q.stale {
                // spec: S-0209 Edge cases (switch off: clearing a terrain object's marks recomputes the house regardless of the remaining side positions)
                o.house = most_recent_house(o);
            }
        }
    }

    /// Adds or removes a vehicle-present claim (S-0209 claim 1).
    pub fn claim_vehicle(&mut self, c: CellXY, l: Layer, add: bool) {
        if let Some(o) = self.occ_mut(c, l) {
            o.vehicles = if add {
                o.vehicles.saturating_add(1)
            } else {
                o.vehicles.saturating_sub(1)
            };
        }
    }

    /// Adds or removes a building-present claim.
    pub fn claim_building(&mut self, c: CellXY, add: bool) {
        if let Some(o) = self.occ_mut(c, Layer::Ground) {
            o.buildings = if add {
                o.buildings.saturating_add(1)
            } else {
                o.buildings.saturating_sub(1)
            };
        }
    }

    /// The passability category of a cell (S-0211 claim 11).
    pub fn category(&self, c: CellXY) -> Option<u8> {
        self.idx(c).map(|i| self.category[i])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cells() -> Cells {
        Cells::new(CellGrid::new(Footprint::new(8, 8, [0, 0, 8, 8])))
    }

    const Q: OccupancyQuirks = OccupancyQuirks {
        side_positions_only: true,
        stale: true,
    };

    // spec: S-0009 claims 1 and 2 (every legal cell lies within the ring limit of any centre, #401)
    #[test]
    fn every_legal_cell_lies_within_the_ring_limit() {
        for (w, h) in [(8, 8), (5, 3), (2, 9), (1, 1), (0, 4)] {
            let c = Cells::new(CellGrid::new(Footprint::new(w, h, [0, 0, w, h])));
            let side = i32::from(c.footprint().coordinate_side());
            let span = -3..side + 3;
            let legal: Vec<CellXY> = span
                .clone()
                .flat_map(|y| span.clone().map(move |x| CellXY::new(x, y)))
                .filter(|&p| c.cell(p).is_some())
                .collect();
            for cy in span.clone() {
                for cx in span.clone() {
                    let limit = c.ring_limit(CellXY::new(cx, cy));
                    let far = legal
                        .iter()
                        .map(|p| (p.x - cx).abs().max((p.y - cy).abs()))
                        .max();
                    assert!(far.is_none_or(|f| f <= limit), "{w}×{h} at ({cx}, {cy})");
                }
            }
        }
        // The far corner of the coordinate square bounds a centre in it.
        let c = cells();
        assert_eq!(c.ring_limit(CellXY::new(8, 8)), 8);
        assert_eq!(c.ring_limit(CellXY::new(2, 12)), 13);
    }

    // spec: S-0209 claim 13 (listing order: non-buildings most recent first, buildings in placement order)
    #[test]
    fn listing_order() {
        let mut c = cells();
        let x = CellXY::new(8, 8);
        c.enter(x, Layer::Ground, ObjectId(1), true);
        c.enter(x, Layer::Ground, ObjectId(2), false);
        c.enter(x, Layer::Ground, ObjectId(3), true);
        c.enter(x, Layer::Ground, ObjectId(4), false);
        assert_eq!(
            c.list(x, Layer::Ground),
            &[ObjectId(4), ObjectId(2), ObjectId(1), ObjectId(3)]
        );
    }

    // spec: S-0209 claim 1 (the infantry house clears only when no side position stays taken; it can be forgotten)
    #[test]
    fn infantry_house() {
        let mut c = cells();
        let x = CellXY::new(8, 8);
        c.mark_infantry(x, Layer::Ground, position::CENTRE, ObjectId(1), HouseId(0));
        c.mark_infantry(x, Layer::Ground, position::RIGHT, ObjectId(2), HouseId(1));
        assert_eq!(c.occ(x, Layer::Ground).unwrap().house, Some(HouseId(1)));
        c.clear_infantry(x, Layer::Ground, ObjectId(2), Q);
        // The centre unit stays, but the house is forgotten.
        assert_eq!(c.occ(x, Layer::Ground).unwrap().house, None);
        // With the switch off, it clears only when no position is taken.
        let mut d = cells();
        d.mark_infantry(x, Layer::Ground, position::CENTRE, ObjectId(1), HouseId(0));
        d.mark_infantry(x, Layer::Ground, position::RIGHT, ObjectId(2), HouseId(1));
        d.clear_infantry(
            x,
            Layer::Ground,
            ObjectId(2),
            OccupancyQuirks {
                side_positions_only: false,
                stale: true,
            },
        );
        assert_eq!(d.occ(x, Layer::Ground).unwrap().house, Some(HouseId(1)));
    }

    // spec: S-0209 claim 1 (clearing a terrain object's marks leaves the house unchanged, even when it frees the last side position) and Edge cases (`quirks.occupancy_house_stale` off: it recomputes the house regardless of the remaining side positions)
    #[test]
    fn terrain_clearing_and_the_house() {
        let x = CellXY::new(8, 8);
        for stale in [true, false] {
            let q = OccupancyQuirks {
                side_positions_only: true,
                stale,
            };
            let mut c = cells();
            c.mark_terrain(x, ObjectId(7), 1);
            c.mark_infantry(x, Layer::Ground, position::LEFT, ObjectId(1), HouseId(0));
            c.clear_infantry(x, Layer::Ground, ObjectId(1), q);
            // The terrain mark keeps the house in force (stale on).
            let after_infantry = c.occ(x, Layer::Ground).unwrap().house;
            assert_eq!(after_infantry, stale.then_some(HouseId(0)));
            c.mark_infantry(x, Layer::Ground, position::CENTRE, ObjectId(2), HouseId(1));
            c.clear_terrain(x, ObjectId(7), q);
            let rec = c.occ(x, Layer::Ground).unwrap();
            assert!(!rec.taken(position::RIGHT));
            // Stale on: unchanged although no side position is taken; off:
            // the most recent infantry still holding a position.
            assert_eq!(rec.house, Some(HouseId(1)), "stale {stale}");
            c.clear_infantry(x, Layer::Ground, ObjectId(2), q);
            c.mark_terrain(x, ObjectId(8), 1);
            c.mark_infantry(x, Layer::Ground, position::LEFT, ObjectId(3), HouseId(0));
            c.clear_infantry(x, Layer::Ground, ObjectId(3), q);
            c.clear_terrain(x, ObjectId(8), q);
            let house = c.occ(x, Layer::Ground).unwrap().house;
            assert_eq!(house, stale.then_some(HouseId(0)), "stale {stale}");
        }
    }

    // spec: S-0208 claim 19 (clearing by actual position clears the position whoever marked it, with the infantry house-clearing rule)
    #[test]
    fn clearing_a_position_whoever_marked_it() {
        let x = CellXY::new(8, 8);
        let mut c = cells();
        c.mark_infantry(x, Layer::Deck, position::RIGHT, ObjectId(1), HouseId(0));
        c.clear_position(x, Layer::Deck, position::RIGHT, Q);
        let rec = c.occ(x, Layer::Deck).unwrap();
        assert!(!rec.taken(position::RIGHT));
        assert_eq!(rec.house, None);
    }

    // spec: S-0209 Edge cases (the house outlasts its infantry; `quirks.occupancy_house_stale` off)
    #[test]
    fn stale_house() {
        let x = CellXY::new(8, 8);
        let mut c = cells();
        c.mark_infantry(x, Layer::Ground, position::RIGHT, ObjectId(1), HouseId(0));
        c.mark_infantry(x, Layer::Ground, position::LEFT, ObjectId(2), HouseId(1));
        c.clear_infantry(x, Layer::Ground, ObjectId(2), Q);
        assert_eq!(c.occ(x, Layer::Ground).unwrap().house, Some(HouseId(1)));
        let mut d = cells();
        d.mark_infantry(x, Layer::Ground, position::RIGHT, ObjectId(1), HouseId(0));
        d.mark_infantry(x, Layer::Ground, position::LEFT, ObjectId(2), HouseId(1));
        d.clear_infantry(
            x,
            Layer::Ground,
            ObjectId(2),
            OccupancyQuirks {
                side_positions_only: true,
                stale: false,
            },
        );
        assert_eq!(d.occ(x, Layer::Ground).unwrap().house, Some(HouseId(0)));
    }

    // spec: S-0209 Terms (side-position pattern bits; only p mod 8 counts, negatives included)
    #[test]
    fn terrain_patterns() {
        let x = CellXY::new(8, 8);
        for (p, want) in [
            (7, [true, true, true]),
            (4, [false, false, true]),
            (-1, [true, true, true]),
            (-5, [true, true, false]),
        ] {
            let mut c = cells();
            c.mark_terrain(x, ObjectId(9), p);
            let o = c.occ(x, Layer::Ground).unwrap();
            assert_eq!(
                [
                    o.taken(position::RIGHT),
                    o.taken(position::LEFT),
                    o.taken(position::BOTTOM)
                ],
                want,
                "pattern {p}"
            );
            assert_eq!(o.house, None);
        }
    }
}
