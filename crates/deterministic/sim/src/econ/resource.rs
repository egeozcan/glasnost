//! Ore and gems (S-0401): resource cells, bales and value, growth and
//! spread, ore mines.
//!
//! Each tiberium keeps a growth list and a spread list ([`List`]) with the
//! timers of its updates ([`TiberiumState`]). Row 9 of S-0203 runs every
//! growth update of the tick before any spread update ([`part`]); ore mines
//! make resource in their own turns ([`mine_turn`], called from
//! [`super::events::turn`]); harvesting and the other removals take bales
//! through [`take_bales`]. The game start rebuilds the lists
//! ([`game_start`], called from `crate::load::new_game`).
//!
//! Seams: the chain reaction of claim 20 and the crater and fence removals
//! of claim 16 (b), (c) and (e) need the animation start and middle events
//! and fence posts, which are other topics'; they call [`take_bales`] or
//! [`take_all`] once those exist. The waypoint thinning of claim 16 (f) is
//! [`thin_around_waypoint`], for the trigger topic. The carry-over of the
//! timers between the games of a session (claim 19) is [`carry_over`], for
//! the session layer.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use glasnost_map::CellXY;
use glasnost_map::NO_OVERLAY;
use glasnost_num::{F64, Rounding};
use glasnost_rules::schema::Ruleset;
use serde::{Deserialize, Serialize};

use crate::rules::{Kind, nearest_double};
use crate::world::{Layer, ObjectId, World};

/// The top stage (S-0401 Constants).
// spec: S-0401 Constants (top stage 11)
pub const TOP_STAGE: u8 = 11;

/// The stage of a new resource cell from a spread step or an ore mine.
// spec: S-0401 Constants (stage of a new resource cell from a spread step or an ore mine: 3)
pub const NEW_STAGE: u8 = 3;

/// One tiberium's fields (S-0401 Import mapping).
#[derive(Clone, Debug, Default)]
pub struct Tiberium {
    /// Section name.
    pub name: String,
    /// Value per bale *V*.
    pub value: i32,
    /// Growth field *G*.
    pub growth: i32,
    /// Growth fraction *g*, a double.
    pub growth_fraction: F64,
    /// Spread field *S*.
    pub spread: i32,
    /// Spread fraction *s*, a double.
    pub spread_fraction: F64,
    /// The image run (*P*, *F*, *S*) of S-0112; `None`: never given an image
    /// number (S-0112 claim 12).
    pub run: Option<[i32; 3]>, // spec: S-0112 claim 10 (the run P, F, S)
}

/// The resource rules: the tiberiums in registration order.
#[derive(Clone, Debug, Default)]
pub struct Rules {
    /// The tiberiums in registration order (S-0004 claim 10).
    pub tiberiums: Vec<Tiberium>,
}

impl Rules {
    /// The resource rules of a resolved ruleset.
    pub fn new(r: &Ruleset, mode: Rounding) -> Self {
        let reg = r.registries.get("tiberiums").cloned().unwrap_or_default();
        let tenth = nearest_double("0.1");
        let fraction = |d: &Option<glasnost_rules::sim::Decimal>| {
            // spec: S-0401 Import mapping (GrowthPercentage, SpreadPercentage: a single widened to double; absent: the double nearest 0.1)
            d.as_ref().map_or(tenth, |d| d.single_value(mode))
        };
        let tiberiums = reg
            .iter()
            .map(|n| {
                let t = r.sim.tiberiums.get(n).cloned().unwrap_or_default();
                Tiberium {
                    name: n.clone(),
                    // spec: S-0401 Import mapping (Value, Growth, Spread absent: 0)
                    value: t.value.unwrap_or(0),
                    growth: t.growth.unwrap_or(0),
                    growth_fraction: fraction(&t.growth_percentage),
                    spread: t.spread.unwrap_or(0),
                    spread_fraction: fraction(&t.spread_percentage),
                    // spec: S-0112 claims 10 and 12 (the run the importer worked out; none when no image number was given)
                    run: t.image_run,
                }
            })
            .collect();
        Self { tiberiums }
    }
}

/// An entry of a growth or spread list: a cell and its key (S-0401 Terms).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entry {
    /// The cell.
    pub cell: CellXY,
    /// The key, already rounded to single precision (F2), held as an integer.
    pub key: i64,
}

/// A growth or spread list (S-0401 Terms). Entries stand in the order they
/// are taken: smallest key first, equal keys in the order they were added
/// (claim 14, a departure).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct List {
    /// The entries in taking order.
    pub entries: VecDeque<Entry>,
    /// The marked cells (spread lists only).
    pub marks: BTreeSet<CellXY>,
    /// The entry count *E* since the last rebuilding.
    pub count: i64,
}

impl List {
    /// Adds an entry behind every entry whose key is at most its key
    /// (claim 14: equal keys in the order added), counting it in *E*.
    fn add(&mut self, cell: CellXY, key: i64) {
        let at = self.entries.partition_point(|e| e.key <= key);
        self.entries.insert(at, Entry { cell, key });
        self.count += 1;
    }

    /// Takes the first entry (S-0401 Terms, *taking an entry*).
    fn take(&mut self) -> Option<Entry> {
        self.entries.pop_front()
    }

    #[cfg(any())]
    fn clear(&mut self) {
        self.entries.clear();
        self.marks.clear();
        self.count = 0;
    }
}

/// One tiberium's lists and timers (S-0401 claims 6 to 9).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TiberiumState {
    /// The growth list.
    pub growth: List,
    /// The spread list.
    pub spread: List,
    /// The tick of the last growth update; `None` before the first.
    pub growth_last: Option<i32>,
    /// The tick of the last spread update; `None` before the first.
    pub spread_last: Option<i32>,
}

/// Resource timers and the growth and spread lists of every tiberium
/// (S-0401 claims 6 to 9 and 13), the map growth switch (claim 4) and the
/// ore mines' animations (claim 15).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Resources {
    /// Per tiberium, in registration order.
    pub tiberiums: Vec<TiberiumState>,
    /// The map growth switch is off (claim 4); on by default.
    pub growth_off: bool,
    /// The ore mines that are animating, with the tick their animation
    /// started (claim 15).
    pub mines: BTreeMap<ObjectId, i32>,
}

fn mode(w: &World) -> Rounding {
    w.rules().mode
}

/// The fraction threshold, the double nearest 10⁻⁵ (S-0401 Constants).
fn threshold() -> F64 {
    // spec: S-0401 Constants (fraction threshold: the double nearest 0.00001, the pattern of consts::THREAT_COEFFICIENT_MIN)
    crate::consts::THREAT_COEFFICIENT_MIN
}

/// A fraction stops an update when it is at most 10⁻⁵ (S-0401 Terms).
fn stops_update(f: F64) -> bool {
    f.le(threshold())
}

/// A fraction passes an eligibility test when it is at least 10⁻⁵ (S-0401
/// Terms).
fn eligible(f: F64) -> bool {
    f.ge(threshold())
}

/// |*r*| mod *k* with S-0202's magnitude rule (S-0401 F2): for *r* = −2³¹
/// under `quirks.random_magnitude_overflow` the remainder is negative.
fn magnitude_mod(w: &World, r: i32, k: i64) -> i64 {
    let mag = if r == i32::MIN && w.rules().quirks.random_magnitude_overflow {
        // spec: S-0202 Edge cases (quirks.random_magnitude_overflow: |−2147483648| stays negative)
        i64::from(r)
    } else {
        i64::from(r).abs()
    };
    // spec: S-0401 F2 (the remainder of the division truncated toward zero)
    mag % k
}

/// A key *t* + *x* stored in single precision (S-0401 F2).
fn single_key(w: &World, v: i64) -> i64 {
    let m = mode(w);
    // spec: S-0401 F2 (the key is stored in single precision; the conversion follows S-0205)
    F64::from_i64(v, m)
        .to_f32(m)
        .to_f64()
        .trunc_i128()
        .map_or(v, |x| x as i64)
}

/// The key of a new entry, one raw draw (S-0401 F2): *t* + (|*r*| mod 50).
fn offer_key(w: &mut World) -> i64 {
    let r = w.rng.raw() as i32;
    // spec: S-0401 F2 (key = t + (|r| mod 50))
    let k = i64::from(w.tick) + magnitude_mod(w, r, 50);
    single_key(w, k)
}

/// The key of a growth re-entry, one raw draw (S-0401 F2): *t* + |*r* mod 50|.
fn reentry_key(w: &mut World) -> i64 {
    let r = w.rng.raw() as i32;
    // spec: S-0401 F2 (the growth re-entry: t + |r mod 50|)
    let k = i64::from(w.tick) + (i64::from(r) % 50).abs();
    single_key(w, k)
}

/// The grid size *C* = 2 × *W* × (*H* + 4) (S-0401 Terms).
fn grid_size(w: &World) -> i64 {
    let fp = w.cells.footprint();
    // spec: S-0401 Terms (grid size C = 2 × W × (H + 4))
    2 * i64::from(fp.width) * (i64::from(fp.height) + 4)
}

/// The tiberium of an overlay type (S-0401 Terms; S-0112 Terms): the first
/// tiberium whose flat or sloped run contains the overlay's list position,
/// else the first tiberium. `None` for an overlay that is not a resource
/// overlay, or when there are no tiberiums.
pub fn tiberium_of_overlay(w: &World, overlay: u8) -> Option<usize> {
    if overlay == NO_OVERLAY {
        return None;
    }
    let r = w.rules();
    if !r
        .overlays
        .get(usize::from(overlay))
        .is_some_and(|o| o.tiberium)
    {
        return None;
    }
    let tibs = &r.econ.resource.tiberiums;
    if tibs.is_empty() {
        return None;
    }
    let pos = i32::from(overlay);
    // spec: S-0401 Terms (the first tiberium whose flat or sloped image run contains the position, else the first)
    Some(
        tibs.iter()
            .position(|t| t.run.is_some_and(|[p, f, s]| pos >= p && pos < p + f + s))
            .unwrap_or(0),
    )
}

/// The tiberium and stage of a resource cell (S-0401 claim 1).
pub fn resource_at(w: &World, c: CellXY) -> Option<(usize, u8)> {
    let cell = w.cells.cell(c)?;
    tiberium_of_overlay(w, cell.overlay).map(|t| (t, cell.overlay_data))
}

/// The worth of a cell, *V* × (*d* + 1) for a resource cell, 0 otherwise
/// (S-0401 claim 2, F1).
pub fn worth(w: &World, c: CellXY) -> i64 {
    match resource_at(w, c) {
        Some((t, d)) => {
            let v = w.rules().econ.resource.tiberiums[t].value;
            // spec: S-0401 F1 (worth = V × (d + 1))
            i64::from(v) * (i64::from(d) + 1)
        }
        None => 0,
    }
}

/// Makes sure every tiberium has its state.
fn ensure(w: &mut World) {
    let n = w.rules().econ.resource.tiberiums.len();
    if w.econ.resources.tiberiums.len() < n {
        w.econ
            .resources
            .tiberiums
            .resize(n, TiberiumState::default());
    }
}

fn neighbour(c: CellXY, d: u8) -> CellXY {
    // spec: S-0212 Terms (step directions 0 to 7)
    let (dx, dy) = crate::regions::STEPS[usize::from(d % 8)];
    CellXY::new(c.x + dx, c.y + dy)
}

/// The conditions of claim 8 for a cell, judged with its own tiberium and
/// not counting the mark. `only_terrain`: the game-start rebuilding sees
/// only terrain objects in cells (claim 13).
fn can_spread(w: &World, c: CellXY, only_terrain: bool) -> bool {
    let Some((t, d)) = resource_at(w, c) else {
        return false;
    };
    let Some(cell) = w.cells.cell(c) else {
        return false;
    };
    let r = w.rules();
    // spec: S-0401 claim 17 (the stage must be above ⌊i/2⌋; quirks.spread_stage_from_index off: above ⌊11/2⌋ = 5)
    let floor = if r.quirks.spread_stage_from_index {
        t / 2
    } else {
        usize::from(TOP_STAGE / 2)
    };
    // spec: S-0401 claim 4 (the spreads setting is always on in a skirmish)
    let objects = w.cells.list(c, Layer::Ground);
    let empty = if only_terrain {
        // spec: S-0401 claim 13 (the game-start rebuilding sees only terrain objects in cells)
        !objects
            .iter()
            .any(|&x| w.obj(x).is_some_and(|o| o.kind == Kind::Terrain))
    } else {
        objects.is_empty()
    };
    // spec: S-0401 claim 8 (a resource cell above stage ⌊i/2⌋, flat, spread fraction at least 10⁻⁵, no object in the ground layer)
    usize::from(d) > floor
        && cell.slope == 0
        && eligible(r.econ.resource.tiberiums[t].spread_fraction)
        && empty
}

/// Whether new resource may appear in a cell (S-0401 claim 10), while a
/// game is running.
pub fn germinable(w: &World, c: CellXY) -> bool {
    let Some(cell) = w.cells.cell(c) else {
        return false;
    };
    let Some(i) = w.cells.idx(c) else {
        return false;
    };
    let r = w.rules();
    // spec: S-0401 claim 10 (in the usable area, by the height-aware form)
    if !w.cells.usable(c) {
        return false;
    }
    // spec: S-0401 claim 10 (not part of a bridge: a high bridge's cell or any other bridge part)
    if w.cells.deck[i] || w.cells.bridge_body[i] {
        return false;
    }
    let list = w.cells.list(c, Layer::Ground);
    // spec: S-0401 claim 10 (only the first building of the ground layer is looked at; a living one prevents it unless invisible or invisible in game)
    if let Some(b) = list
        .iter()
        .filter_map(|&x| w.obj(x))
        .find(|o| o.kind == Kind::Building)
    {
        let bt = &r.buildings[b.type_index as usize].raw;
        let invisible = bt.econ.invisible.unwrap_or(false) || bt.invisible_in_game.unwrap_or(false);
        if b.health > 0 && !invisible {
            return false;
        }
    }
    // spec: S-0401 claim 10 (the first terrain object, if any, is not an ore mine)
    if let Some(t) = list
        .iter()
        .filter_map(|&x| w.obj(x))
        .find(|o| o.kind == Kind::Terrain)
        && r.terrain
            .get(t.type_index as usize)
            .is_some_and(|tt| tt.spawns_resources)
    {
        return false;
    }
    // spec: S-0401 claims 10 and 11 (quirks.spread_step_lost_on_terrain_object off: a cell holding a terrain object is not germinable; on, it is, and receives nothing)
    if !r.quirks.spread_step_lost_on_terrain_object && holds_terrain_object(w, c) {
        return false;
    }
    // spec: S-0401 claim 10 (buildable land, no overlay, flat)
    if !r
        .land
        .get(usize::from(cell.land))
        .is_some_and(|l| l.buildable)
        || cell.overlay != NO_OVERLAY
        || cell.slope != 0
    {
        return false;
    }
    let cat = w.catalogue_arc();
    // spec: S-0401 claim 10 and S-0010 claim 16 (the tileset's allow-tiberium flag, or a tile number outside the theater's tiles)
    match usize::try_from(cell.tile) {
        Ok(n) if n < cat.tiles.len() => cat.allow_tiberium.get(n).copied().unwrap_or(false),
        _ => true,
    }
}

/// Whether a terrain object (a tree, an ore mine) is listed in the cell's
/// ground layer (S-0401 claim 11).
fn holds_terrain_object(w: &World, c: CellXY) -> bool {
    w.cells
        .list(c, Layer::Ground)
        .iter()
        .any(|&x| w.obj(x).is_some_and(|o| o.kind == Kind::Terrain))
}

/// Offers a cell to tiberium `t`'s growth list (S-0401 claim 7).
pub(crate) fn offer_growth(w: &mut World, t: usize, c: CellXY) {
    let Some(d) = w.cells.cell(c).map(|x| x.overlay_data) else {
        return;
    };
    // spec: S-0401 claim 7 (an offer of a cell at stage 11 or above does nothing and takes no draw)
    if d >= TOP_STAGE {
        return;
    }
    ensure(w);
    // spec: S-0401 F7 (an offer that will add an entry rebuilds the growth list first when E > C − 10)
    if w.econ.resources.tiberiums[t].growth.count > grid_size(w) - 10 {
        rebuild_growth(w, t);
    }
    let key = offer_key(w);
    w.econ.resources.tiberiums[t].growth.add(c, key);
}

/// Offers a cell to tiberium `t`'s spread list (S-0401 claim 8).
fn offer_spread(w: &mut World, t: usize, c: CellXY) {
    ensure(w);
    // spec: S-0401 claim 8 (a cell already marked in that list is not added)
    if w.econ.resources.tiberiums[t].spread.marks.contains(&c) || !can_spread(w, c, false) {
        return;
    }
    // spec: S-0401 F7 (an offer that will add an entry rebuilds the spread list first when E ≥ C − 20)
    if w.econ.resources.tiberiums[t].spread.count >= grid_size(w) - 20 {
        rebuild_spread(w, t, false);
    }
    let key = offer_key(w);
    let l = &mut w.econ.resources.tiberiums[t].spread;
    l.add(c, key);
    l.marks.insert(c);
}

/// Rebuilds tiberium `t`'s growth list (S-0401 claim 13): every flat cell
/// of that tiberium below stage 11, with key 0, in the visiting order, when
/// the map growth switch is on and the growth fraction is at least 10⁻⁵.
fn rebuild_growth(w: &mut World, t: usize) {
    ensure(w);
    let order = w
        .cells
        .grid
        .visiting_order(w.rules().quirks.narrow_map_partial_pass);
    let on = !w.econ.resources.growth_off
        && eligible(w.rules().econ.resource.tiberiums[t].growth_fraction);
    let mut list = List::default();
    if on {
        for c in order {
            if let Some((tt, d)) = resource_at(w, c)
                && tt == t
                && d < TOP_STAGE
                && w.cells.cell(c).is_some_and(|x| x.slope == 0)
            {
                // spec: S-0401 claim 13 (key 0, in the visiting order)
                list.add(c, 0);
            }
        }
    }
    // spec: S-0401 claim 13 (E equals the number of entries the rebuilding made)
    w.econ.resources.tiberiums[t].growth = list;
}

/// Rebuilds tiberium `t`'s spread list (S-0401 claim 13): every cell of
/// that tiberium for which claim 8's conditions hold, marked, key 0, in the
/// visiting order.
fn rebuild_spread(w: &mut World, t: usize, only_terrain: bool) {
    ensure(w);
    let order = w
        .cells
        .grid
        .visiting_order(w.rules().quirks.narrow_map_partial_pass);
    let mut list = List::default();
    for c in order {
        if resource_at(w, c).is_some_and(|(tt, _)| tt == t) && can_spread(w, c, only_terrain) {
            // spec: S-0401 claim 13 (exactly those cells are marked; key 0, in the visiting order)
            list.add(c, 0);
            list.marks.insert(c);
        }
    }
    w.econ.resources.tiberiums[t].spread = list;
}

/// The cell's land type is worked out again after a change of its overlay
/// (S-0211 claim 5) and the regions learn of it.
fn cell_changed(w: &mut World, c: CellXY) {
    // spec: S-0401 claim 5 (its land type is worked out again, S-0211 claim 5)
    crate::load::recalc_cells(w, &[c]);
    crate::regions::cell_changed(w, c);
}

/// Empties a resource cell (S-0401 claim 5).
fn empty_cell(w: &mut World, c: CellXY) {
    let Some((t, _)) = resource_at(w, c) else {
        return;
    };
    if let Some(cell) = w.cells.grid.get_mut(c) {
        // spec: S-0401 claim 5 (the overlay is removed and the stage set to 0)
        cell.overlay = NO_OVERLAY;
        cell.overlay_data = 0;
    }
    cell_changed(w, c);
    ensure(w);
    // spec: S-0401 claim 5 (unmarked in the spread list of every tiberium)
    for s in &mut w.econ.resources.tiberiums {
        s.spread.marks.remove(&c);
    }
    let by_emptied = w.rules().quirks.spread_offer_by_emptied_type;
    // spec: S-0401 claim 5 (the eight neighbours, in step-direction order 0 to 7)
    for d in 0..8 {
        let n = neighbour(c, d);
        if w.cells.cell(n).is_none() {
            continue;
        }
        let list = if by_emptied {
            // spec: S-0401 claim 5 (offered to the emptied cell's tiberium's spread list; quirks.spread_offer_by_emptied_type)
            Some(t)
        } else {
            resource_at(w, n).map(|(own, _)| own)
        };
        if let Some(l) = list {
            offer_spread(w, l, n);
        }
    }
}

/// Takes `n` bales from a cell (S-0401 claim 3) and returns the bales
/// delivered.
pub fn take_bales(w: &mut World, c: CellXY, n: i32) -> i32 {
    // spec: S-0401 claim 3 (taking 0 or fewer bales, or from a cell without resource, changes nothing)
    if n <= 0 {
        return 0;
    }
    let Some((t, d)) = resource_at(w, c) else {
        return 0;
    };
    let d32 = i32::from(d);
    if n <= d32 {
        // spec: S-0401 claim 3 (n ≤ d: the stage falls to d − n, all n delivered)
        if let Some(cell) = w.cells.grid.get_mut(c) {
            cell.overlay_data = (d32 - n) as u8;
        }
        // spec: S-0401 claim 18 (no growth entry after a take; quirks.full_cell_no_regrowth off: a cell that was at stage 11 is offered once its stage has fallen)
        if !w.rules().quirks.full_cell_no_regrowth && d == TOP_STAGE {
            offer_growth(w, t, c);
        }
        return n;
    }
    empty_cell(w, c);
    // spec: S-0401 claim 3 (n ≥ d + 1: emptied and only d delivered; quirks.last_bale_lost off: all d + 1)
    if w.rules().quirks.last_bale_lost {
        d32
    } else {
        d32 + 1
    }
}

/// Takes every bale of a cell (S-0401 claim 16 (b), the chain reaction's
/// emptying of claim 20), returning the bales delivered.
pub fn take_all(w: &mut World, c: CellXY) -> i32 {
    match resource_at(w, c) {
        Some((_, d)) => take_bales(w, c, i32::from(d) + 1),
        None => 0,
    }
}

/// The trigger action that thins resource around a waypoint (S-0401 claim
/// 16 (f)): one bale, once for every displacement with |dx|, |dy| ≤ 5 and
/// dx² + dy² ≤ 35.
pub fn thin_around_waypoint(w: &mut World, wp: CellXY) {
    let only_wp = w.rules().quirks.thinning_hits_waypoint_only;
    // spec: S-0401 claim 16 (f) (displacements with |dx|, |dy| ≤ 5 and dx² + dy² ≤ 35)
    for dy in -5..=5 {
        for dx in -5..=5 {
            if dx * dx + dy * dy > 35 {
                continue;
            }
            // spec: S-0401 claim 16 (f) (every take hits the waypoint's cell; quirks.thinning_hits_waypoint_only off: the displaced cell)
            let c = if only_wp {
                wp
            } else {
                CellXY::new(wp.x + dx, wp.y + dy)
            };
            take_bales(w, c, 1);
        }
    }
}

/// A germinable cell becomes a resource cell of tiberium `t` at stage `a`
/// (S-0401 claim 11), for the producers of other specs (S-0317 claim 12).
pub fn add_resource_cell(w: &mut World, c: CellXY, t: usize, a: u8) {
    new_resource_cell(w, c, t, a);
}

/// A germinable cell becomes a resource cell of tiberium `t` at stage `a`
/// (S-0401 claim 11); one holding a terrain object takes the draws and the
/// growth-list offer and receives nothing.
fn new_resource_cell(w: &mut World, c: CellXY, t: usize, a: u8) {
    let Some(tib) = w.rules().econ.resource.tiberiums.get(t).cloned() else {
        return;
    };
    // spec: S-0401 claim 11 (no new cell at stage 12 or more, nor of a tiberium never given an image number)
    if a >= 12 {
        return;
    }
    let Some([p, f, _]) = tib.run else { return };
    let Some(slope) = w.cells.cell(c).map(|x| x.slope) else {
        return;
    };
    let half = w.rules().quirks.ranged_draw_half_span;
    let position = if slope == 0 {
        // spec: S-0401 claim 11 (flat: P + U(0, 11))
        p + w.rng.ranged(0, 11, half)
    } else {
        // spec: S-0401 claim 11 (slope σ: P + F + 2(σ − 1) + U(0, 1))
        p + f + 2 * (i32::from(slope) - 1) + w.rng.ranged(0, 1, half)
    };
    let Ok(o) = u8::try_from(position) else {
        return;
    };
    if holds_terrain_object(w, c) {
        // spec: S-0401 claim 11 (a cell holding a terrain object gets no overlay: the overlay draw and the growth-list offer as for any new cell, and it holds no resource afterwards)
        // Our design: its stage is not set either (it holds no resource; the Open questions' kept overlay data value), so the cell is unchanged.
        offer_growth(w, t, c);
        return;
    }
    if let Some(cell) = w.cells.grid.get_mut(c) {
        cell.overlay = o;
    }
    // spec: S-0401 claim 11 (offered to the growth list before the stage is set; Open questions: the offer sees the old value)
    offer_growth(w, t, c);
    if let Some(cell) = w.cells.grid.get_mut(c) {
        cell.overlay_data = a;
    }
    cell_changed(w, c);
}

/// One stage increase of tiberium `t` at a cell by a crate (S-0520 claim
/// 24): a germinable cell becomes a new resource cell at stage 1, a cell
/// already holding `t` gains one stage when claim 7's growth-step conditions
/// hold (and is offered to the spread list, with no new growth entry); a
/// run of at most one picture changes nothing, and the stage stays below the
/// picture count.
pub fn crate_stage(w: &mut World, c: CellXY, t: usize) {
    let Some(tib) = w.rules().econ.resource.tiberiums.get(t).cloned() else {
        return;
    };
    let Some([_, flat, _]) = tib.run else { return };
    // spec: S-0520 claim 24 (an image run with at most one picture: the increase changes nothing)
    if flat <= 1 {
        return;
    }
    if germinable(w, c) {
        // spec: S-0520 claim 24 (a germinable cell becomes a new resource cell of t at stage 1, with S-0401 claim 11's draws)
        new_resource_cell(w, c, t, 1);
        return;
    }
    let Some((tt, d)) = resource_at(w, c) else {
        return;
    };
    let flat_cell = w.cells.cell(c).is_some_and(|x| x.slope == 0);
    let grows = !w.econ.resources.growth_off && eligible(tib.growth_fraction);
    // spec: S-0520 claim 24 (an existing cell of t gains one stage only when S-0401 claim 7's growth-step conditions hold; the stage stays below the run's picture count)
    if tt == t && grows && flat_cell && d < TOP_STAGE && i32::from(d) + 1 < flat {
        if let Some(cell) = w.cells.grid.get_mut(c) {
            cell.overlay_data = d + 1;
        }
        offer_spread(w, t, c);
    }
}

/// A spread step from `from` with tiberium `t` (S-0401 claims 9 and 15):
/// one ranged draw U(0, 7), then the first germinable neighbour from that
/// direction is chosen and becomes a new resource cell at stage 3, or
/// receives nothing when it holds a terrain object (claim 11).
fn spread_step(w: &mut World, from: CellXY, t: usize) {
    let half = w.rules().quirks.ranged_draw_half_span;
    // spec: S-0401 claim 9 (one ranged draw U(0, 7) gives the starting step direction)
    let j = w.rng.ranged(0, 7, half);
    // spec: S-0401 claim 9 (the neighbours in step directions j, j + 1, …, j + 7, modulo 8)
    for k in 0..8 {
        let n = neighbour(from, ((j + k) % 8) as u8); // spec: S-0401 claim 9 (modulo 8)
        if germinable(w, n) {
            // spec: S-0401 claim 9 (the first germinable neighbour is chosen; claim 11 says what it receives)
            new_resource_cell(w, n, t, NEW_STAGE);
            return;
        }
    }
}

fn count_germinable(w: &World, c: CellXY) -> u8 {
    (0..8).filter(|&d| germinable(w, neighbour(c, d))).count() as u8 // spec: S-0401 claim 9 (c counts the eight neighbours)
}

/// The count *m* of F5 and F6: trunc(RZ₅₃(*n* × *f*)) clamped to
/// `lo` … `hi`, then (|*r*| mod *k*) + 1, one raw draw.
fn count(w: &mut World, n: usize, f: F64, lo: i64, hi: i64) -> i64 {
    let r = w.rng.raw() as i32;
    let m = mode(w);
    // spec: S-0401 F5 and F6 (k = trunc(RZ₅₃(n × fraction)), clamped)
    let k = F64::from_i64(n as i64, m)
        .mul(f, m)
        .trunc_i128()
        .unwrap_or(0)
        .clamp(i128::from(lo), i128::from(hi)) as i64;
    // spec: S-0401 F5 and F6 (m = (|r| mod k) + 1)
    magnitude_mod(w, r, k) + 1
}

/// A growth update of tiberium `t` (S-0401 claim 7).
fn growth_update(w: &mut World, t: usize) {
    let tib = w.rules().econ.resource.tiberiums[t].clone();
    let n = w.econ.resources.tiberiums[t].growth.entries.len();
    // spec: S-0401 claim 6 (nothing when the list is empty or the growth fraction is at most 10⁻⁵)
    if n == 0 || stops_update(tib.growth_fraction) {
        return;
    }
    // spec: S-0401 F5 (k clamped to 5 … 50)
    let m = count(w, n, tib.growth_fraction, 5, 50);
    // spec: S-0401 F7 (a growth update rebuilds after its draw for m when n > C − 2m)
    if n as i64 > grid_size(w) - 2 * m {
        rebuild_growth(w, t);
    }
    if m < 1 {
        // spec: S-0401 claim 7 (m below 1: the first entry is removed without a result)
        w.econ.resources.tiberiums[t].growth.take();
        return;
    }
    let grows = !w.econ.resources.growth_off && tib.run.is_some() && eligible(tib.growth_fraction);
    for _ in 0..m {
        let Some(e) = w.econ.resources.tiberiums[t].growth.take() else {
            break;
        };
        let c = e.cell;
        // spec: S-0401 claim 7 table (no resource, or another tiberium: dropped)
        let Some((tt, d)) = resource_at(w, c) else {
            continue;
        };
        if tt != t || d >= TOP_STAGE {
            // spec: S-0401 claim 7 table (stage 11 or above: no change, no new entry)
            continue;
        }
        let flat = w.cells.cell(c).is_some_and(|x| x.slope == 0);
        // spec: S-0401 claim 7 (a growth step: switch on, flat, below 11, an image number, fraction at least 10⁻⁵)
        if grows && flat {
            if let Some(cell) = w.cells.grid.get_mut(c) {
                // spec: S-0401 claim 7 (a growth step raises the stage by 1)
                cell.overlay_data = d + 1;
            }
            // spec: S-0401 claim 7 table (stage becomes d + 1; offered to T's spread list; then a new growth entry below 11)
            offer_spread(w, t, c);
            if d + 1 < TOP_STAGE {
                let key = reentry_key(w);
                // spec: S-0401 F7 (re-entries add 1 to E without any test)
                w.econ.resources.tiberiums[t].growth.add(c, key);
            }
        } else {
            // spec: S-0401 claim 7 table (no growth step: a new growth entry, then offered to T's spread list)
            let key = reentry_key(w);
            w.econ.resources.tiberiums[t].growth.add(c, key);
            offer_spread(w, t, c);
        }
    }
}

/// A spread update of tiberium `t` (S-0401 claim 9).
fn spread_update(w: &mut World, t: usize) {
    let tib = w.rules().econ.resource.tiberiums[t].clone();
    let n = w.econ.resources.tiberiums[t].spread.entries.len();
    // spec: S-0401 claim 6 (nothing when the list is empty or the spread fraction is at most 10⁻⁵)
    if n == 0 || stops_update(tib.spread_fraction) {
        return;
    }
    // spec: S-0401 F6 (k clamped to 5 … 25)
    let m = count(w, n, tib.spread_fraction, 5, 25);
    // spec: S-0401 F7 (a spread update rebuilds after its draw for m when n > C − 20)
    if n as i64 > grid_size(w) - 20 {
        rebuild_spread(w, t, false);
    }
    if m < 1 {
        // spec: S-0401 claim 9 (m below 1: the first entry is removed without a result; its cell keeps its mark)
        w.econ.resources.tiberiums[t].spread.take();
        return;
    }
    let q = w.rules().quirks;
    let mut attempts = 0;
    while attempts < m {
        let Some(e) = w.econ.resources.tiberiums[t].spread.take() else {
            break;
        };
        let c = e.cell;
        // spec: S-0401 claim 9 (c, the germinable neighbours when the entry is taken, before its spread step)
        let g = count_germinable(w, c);
        let ok = can_spread(w, c, false);
        if g == 0 || (!q.spread_attempts_count_failures && !ok) {
            // spec: S-0401 claim 9 (c = 0: unmarked, dropped, no attempt; quirks.spread_attempts_count_failures off: a failing cell likewise)
            w.econ.resources.tiberiums[t].spread.marks.remove(&c);
            continue;
        }
        // spec: S-0401 claim 9 (c ≥ 1: one attempt)
        attempts += 1;
        if ok {
            let own = resource_at(w, c).map_or(t, |(own, _)| own);
            // spec: S-0401 claim 9 (the spread step makes resource of the spreading cell's own tiberium)
            spread_step(w, c, own);
        }
        let l = &mut w.econ.resources.tiberiums[t].spread;
        // spec: S-0401 claim 9 (c ≥ 2: added again, marked, with key 0)
        if g >= 2 {
            l.add(c, 0);
            l.marks.insert(c);
        } else if !q.spread_mark_kept_on_last_neighbour {
            // spec: S-0401 claim 9 (c = 1: not added again; quirks.spread_mark_kept_on_last_neighbour off: unmarked)
            l.marks.remove(&c);
        }
    }
}

/// The growth period *P*g = trunc(RZ₅₃(*G* × 0.3)) of a tiberium (S-0401
/// F3), the grows setting being on in a skirmish.
pub fn growth_period(w: &World, t: usize) -> i32 {
    let m = mode(w);
    let g = w.rules().econ.resource.tiberiums[t].growth;
    // spec: S-0401 F3 and claim 4 (k = the double nearest 0.3 while the grows setting is on, always in a skirmish)
    F64::from_i64(i64::from(g), m)
        .mul(crate::consts::THREE_TENTHS, m)
        .to_i32_common()
}

/// The spread period *P*s = *S* (S-0401 F4).
pub fn spread_period(w: &World, t: usize) -> i32 {
    // spec: S-0401 F4 (Ps = S)
    w.rules().econ.resource.tiberiums[t].spread
}

/// Whether a timer has run out (S-0401 claim 6).
fn due(now: i32, last: Option<i32>, period: i32) -> bool {
    // spec: S-0401 claim 6 (a period of 0 or less runs the update in every tick)
    if period <= 0 {
        return true;
    }
    match last {
        // spec: S-0401 claim 19 (the first game of a session: the first updates run in the first tick)
        None => true,
        // spec: S-0401 claim 6 (the tick number minus the tick of the last update is at least the period)
        Some(l) => i64::from(now) - i64::from(l) >= i64::from(period),
    }
}

/// Row 9 of S-0203: growth and spread of resources, growth first.
pub fn part(w: &mut World) {
    // spec: S-0401 claim 6 (while the map growth switch is off no update runs and the timers do not restart)
    if w.econ.resources.growth_off {
        return;
    }
    ensure(w);
    let now = w.tick;
    let n = w.rules().econ.resource.tiberiums.len();
    // spec: S-0401 claim 6 (all growth updates, in registration order, then all spread updates)
    for t in 0..n {
        if due(
            now,
            w.econ.resources.tiberiums[t].growth_last,
            growth_period(w, t),
        ) {
            growth_update(w, t);
            // spec: S-0401 claim 6 (the timer restarts from that tick)
            w.econ.resources.tiberiums[t].growth_last = Some(now);
        }
    }
    for t in 0..n {
        if due(
            now,
            w.econ.resources.tiberiums[t].spread_last,
            spread_period(w, t),
        ) {
            spread_update(w, t);
            w.econ.resources.tiberiums[t].spread_last = Some(now);
        }
    }
}

/// The game-start rebuilding of every list (S-0401 claim 13) and the map
/// growth switch (claim 4). Judges cells seeing only terrain objects.
pub fn game_start(w: &mut World, map_growth: bool) {
    // spec: S-0401 claim 4 (the map growth switch: the map's own growth key, on unless the map turns it off)
    w.econ.resources.growth_off = !map_growth;
    ensure(w);
    let n = w.rules().econ.resource.tiberiums.len();
    for t in 0..n {
        // spec: S-0401 claim 13 (at every game start each tiberium's growth list and spread list are rebuilt)
        rebuild_growth(w, t);
        rebuild_spread(w, t, true);
    }
}

/// Sets the map growth switch during play (S-0401 claim 4; the trigger
/// action that changes it is the trigger specs').
pub fn set_map_growth(w: &mut World, on: bool) {
    w.econ.resources.growth_off = !on;
}

/// The timers carried from the previous game of a session into a new one
/// (S-0401 claim 19): with `quirks.resource_timers_carry_over` on each
/// tiberium keeps the tick of its last updates; off, they restart.
pub fn carry_over(w: &mut World, previous: &Resources) {
    if !w.rules().quirks.resource_timers_carry_over {
        // spec: S-0401 claim 19 (quirks.resource_timers_carry_over off: the timers restart at each game start)
        return;
    }
    ensure(w);
    // spec: S-0401 claim 19 (the timers belong to the tiberiums, which carry over between the games of a session)
    for (s, p) in w
        .econ
        .resources
        .tiberiums
        .iter_mut()
        .zip(&previous.tiberiums)
    {
        s.growth_last = p.growth_last;
        s.spread_last = p.spread_last;
    }
}

/// An ore mine's turn (S-0401 claim 15).
pub fn mine_turn(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id) else { return };
    if o.kind != Kind::Terrain || !o.on_map {
        return;
    }
    let cell = o.pos.cell();
    let Some(tt) = w.rules().terrain.get(o.type_index as usize).cloned() else {
        return;
    };
    // spec: S-0401 claim 15 (a mine whose type lacks the animated flag never draws and never makes resource)
    if !tt.spawns_resources || !tt.animated {
        return;
    }
    let now = w.tick;
    let half = tt.image_frames / 2; // spec: S-0401 claim 15 (⌊N/2⌋)
    match w.econ.resources.mines.get(&id).copied() {
        None => {
            let r = w.rng.raw() as i32;
            let m = mode(w);
            // spec: S-0401 F8 ((|r| mod 1000000) × the double nearest 10⁻⁶, RZ₅₃, compared with p)
            let x =
                F64::from_i64(magnitude_mod(w, r, 1_000_000), m).mul(crate::consts::MILLIONTH, m);
            // spec: S-0401 claim 15 (with R = 0 a started animation never advances and the mine stays idle)
            if x.lt(tt.animation_probability) && tt.animation_rate != 0 {
                w.econ.resources.mines.insert(id, now);
            }
        }
        Some(t0) => {
            // spec: S-0401 claim 15 (fewer than 2 image frames: the animation never stops)
            if half < 1 {
                return;
            }
            // spec: S-0401 claim 15 (the turn in tick t₀ + R × ⌊N/2⌋ stops the animation and makes a spread step)
            if i64::from(now) - i64::from(t0) < i64::from(tt.animation_rate) * i64::from(half) {
                return;
            }
            w.econ.resources.mines.remove(&id);
            let n = w.rules().econ.resource.tiberiums.len();
            // spec: S-0401 claim 15 (the tiberium of the mine's cell, or the first in registration order)
            let t = resource_at(w, cell).map_or(0, |(t, _)| t);
            if n == 0 {
                // No tiberium: the draw is taken all the same and nothing appears.
                let half_q = w.rules().quirks.ranged_draw_half_span;
                let _ = w.rng.ranged(0, 7, half_q); // spec: S-0401 claim 15 (the U(0, 7) draw is taken even when nothing appears)
                return;
            }
            // spec: S-0401 claim 15 (none of claim 8's conditions is checked; the U(0, 7) draw is taken even when no neighbour is germinable)
            spread_step(w, cell, t);
        }
    }
}

/// An object left play: an ore mine forgets its animation.
pub fn removed(w: &mut World, id: ObjectId) {
    w.econ.resources.mines.remove(&id);
}
