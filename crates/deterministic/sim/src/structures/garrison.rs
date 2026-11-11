//! Garrisons (S-0508): infantry occupy a building whose type can be
//! occupied; civilian garrison buildings follow their occupants' houses; the
//! occupants leave by an eviction (claims 10 to 13), are cleared by an
//! assault (claim 14), removed with an erased building (claim 15) or lost
//! to their own house's defeat (claim 19).
//!
//! What the occupants do while inside (the weapon, delay, damage, range and
//! muzzle of S-0308 claim 13, S-0301 claim 7, S-0302 claim 2, S-0307 claim 4
//! and S-0313 claim 6) is read from here by the combat code.

use glasnost_map::CellXY;
use glasnost_num::geom::Facing;

use crate::combat::damage::Hit;
use crate::combat::effects::{self, Effect, Sound};
use crate::econ::TypeRef;
use crate::occupancy::{self, Outcome};
use crate::rules::{Behaviour, BuildingRules, Kind};
use crate::scatter::{self, ScatterRequest};
use crate::world::{HouseId, Layer, ObjectId, Pos, World};

/// The tech level of a civilian garrison building.
// spec: S-0508 Constants (tech level of a civilian garrison building −1)
const CIVILIAN_TECH_LEVEL: i32 = -1;

/// The facing of a put-out occupant.
// spec: S-0508 Constants (facing of a put-out occupant 0)
const PUT_OUT_FACING: Facing = 0;

/// The reference level of the exit-cell check.
// spec: S-0508 Constants (reference level of the exit-cell check −1)
const EXIT_REFERENCE_LEVEL: i32 = -1;

/// The options of an eviction (S-0508 Terms, claim 13).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Evict {
    /// The released occupants are given *hunt* (claim 11).
    pub hunting: bool,
    /// With no qualifying exit cell, the inside fallback cell is the exit
    /// (claim 12).
    pub inside_fallback: bool,
}

impl Evict {
    /// The deploy order, forced evacuation, destruction and the map's
    /// eviction action: no hunting, no inside fallback.
    // spec: S-0508 claim 13 (rows 1 to 3 and 5: no hunting, no inside fallback)
    pub const PLAIN: Evict = Evict {
        hunting: false,
        inside_fallback: false,
    };
    /// A sale: no hunting, the inside fallback.
    // spec: S-0508 claim 13 (the building is sold: no hunting, inside fallback)
    pub const SALE: Evict = Evict {
        hunting: false,
        inside_fallback: true,
    };
    /// The house-wide hunt order: hunting, no inside fallback.
    // spec: S-0508 claim 13 (a house-wide order that sends all its units hunting: hunting, no inside fallback)
    pub const HUNT: Evict = Evict {
        hunting: true,
        inside_fallback: false,
    };
}

// ---------------------------------------------------------------------------
// Queries

fn brules(w: &World, b: ObjectId) -> Option<&BuildingRules> {
    let o = w.obj(b)?;
    if o.kind != Kind::Building {
        return None;
    }
    w.rules().buildings.get(o.type_index as usize)
}

/// A garrisonable building: its type's occupiable flag is set (S-0508
/// Terms).
pub fn garrisonable(w: &World, b: ObjectId) -> bool {
    // spec: S-0508 Import mapping (CanBeOccupied absent: no)
    brules(w, b).is_some_and(|t| t.raw.can_be_occupied.unwrap_or(false))
}

/// The occupant capacity of a building's type (S-0508 Terms).
pub fn capacity(w: &World, b: ObjectId) -> i32 {
    // spec: S-0508 Constants (absent occupant capacity 0)
    brules(w, b).map_or(0, |t| t.building.max_occupants)
}

/// The occupants of a building in order, the first first (S-0508 Terms).
/// A place whose occupant dies inside is removed in the same update
/// (claim 23), so every identifier names an occupant.
pub fn occupants(w: &World, b: ObjectId) -> Vec<ObjectId> {
    w.obj(b)
        .map_or_else(Vec::new, |o| o.structure.occupants.clone())
}

/// The occupant count (claim 23: the count after any death removal).
pub fn count(w: &World, b: ObjectId) -> i32 {
    w.obj(b).map_or(0, |o| {
        i32::try_from(o.structure.occupants.len()).unwrap_or(i32::MAX)
    })
}

/// A garrisoned building (glossary): a building of a garrisonable type with
/// at least one occupant.
pub fn garrisoned(w: &World, b: ObjectId) -> bool {
    garrisonable(w, b) && count(w, b) > 0
}

/// A garrisoned building whose type lets the occupants fire from inside
/// (S-0308 claim 13): the occupants supply its weapons.
pub fn fires(w: &World, b: ObjectId) -> bool {
    garrisoned(w, b) && brules(w, b).is_some_and(|t| t.building.can_occupy_fire)
}

/// Whether an infantry unit's type is an occupier type (S-0508 Terms).
pub fn occupier(w: &World, id: ObjectId) -> bool {
    // spec: S-0508 Import mapping (Occupier absent: no)
    unit_flag(w, id, |u| u.raw.occupier.unwrap_or(false))
}

/// Whether an infantry unit's type is an assaulter type (S-0508 Terms).
pub fn assaulter(w: &World, id: ObjectId) -> bool {
    // spec: S-0508 Import mapping (Assaulter absent: no)
    unit_flag(w, id, |u| u.raw.assaulter.unwrap_or(false))
}

fn unit_flag(w: &World, id: ObjectId, f: impl Fn(&crate::rules::UnitRules) -> bool) -> bool {
    w.obj(id)
        .filter(|o| o.kind == Kind::Infantry)
        .and_then(|o| w.rules().unit(o.kind, o.type_index))
        .is_some_and(f)
}

/// A civilian garrison building: garrisonable with a tech level of exactly
/// −1 (S-0508 Terms).
pub fn civilian_building(w: &World, b: ObjectId) -> bool {
    // spec: S-0508 Import mapping (TechLevel absent: 255, S-0407 Import mapping)
    garrisonable(w, b)
        && brules(w, b).is_some_and(|t| t.raw.tech_level.unwrap_or(255) == CIVILIAN_TECH_LEVEL)
}

/// The civilian house: the first house, in house order, whose country
/// belongs to the civilian side (S-0508 Terms, S-0414 claim 12).
pub fn civilian_house(w: &World) -> Option<HouseId> {
    (0..w.houses.len())
        .map(|i| HouseId(i as u8))
        .find(|&h| crate::econ::setup::civilian(w, h))
}

/// The occupation test of claim 1 (the "can garrison" test of S-0214 row
/// I8), also the test of an arrival (claim 2).
pub fn may_enter(w: &World, unit: ObjectId, b: ObjectId) -> bool {
    crate::targeting::may_garrison(w, unit, b)
}

/// Whether infantry unit `unit` "may occupy" building `b` in the sense of
/// S-0215 claims 23 and 27 (S-0508 claim 1, r19, Q-2927), which is also the
/// test of an arrival (claim 2): claim 1's test for a unit of an occupier
/// type (a type with both marks included, claim 14), claim 14's assault
/// eligibility for a unit of an assaulter type that is not an occupier type,
/// and never for a type with neither mark.
pub fn may_occupy(w: &World, unit: ObjectId, b: ObjectId) -> bool {
    if occupier(w, unit) {
        // spec: S-0508 claim 1 (r19, Q-2927: for a unit of an occupier type, this claim's test) and claim 14 (a type with both marks behaves as an occupier type only)
        may_enter(w, unit, b)
    } else if assaulter(w, unit) {
        // spec: S-0508 claim 1 (r19, Q-2927: for an assaulter type that is not an occupier type, claim 14's assault eligibility)
        assault_eligible(w, unit, b)
    } else {
        // spec: S-0508 claim 1 (r19, Q-2927: never met by a type with neither mark)
        false
    }
}

/// The complete assault eligibility test of claim 14 for an infantry unit of
/// an assaulter type that is not an occupier type.
pub fn assault_eligible(w: &World, unit: ObjectId, b: ObjectId) -> bool {
    let (Some(u), Some(o)) = (w.obj(unit), w.obj(b)) else {
        return false;
    };
    // spec: S-0508 claim 14 (the building's type is garrisonable)
    if !garrisonable(w, b) {
        return false;
    }
    // spec: S-0508 claim 14 (neither being built up nor being sold)
    if matches!(
        o.beh.seen(),
        Some(Behaviour::Construction | Behaviour::Selling)
    ) || o.econ.sale.is_some()
    {
        return false;
    }
    // spec: S-0508 claim 14 (the building's centre lies in the usable area)
    if !w.cells.usable(w.reference_point(b).cell()) {
        return false;
    }
    // spec: S-0508 claim 14 (not being erased)
    if crate::econ::power::being_erased(w, b) {
        return false;
    }
    // spec: S-0508 claim 14 (at least one occupant)
    if count(w, b) == 0 {
        return false;
    }
    // spec: S-0508 claim 14 (the building's owner does not count the unit's house as allied: a one-way test from the building owner's side, own-house alliance included)
    !w.allied(o.owner, u.owner)
}

// ---------------------------------------------------------------------------
// Entering (claims 2 to 6)

/// An arrival of an occupier or assaulter type at its destination building
/// (claim 2): the entry attempt. Returns whether the unit left the map.
pub fn attempt(w: &mut World, unit: ObjectId, b: ObjectId) -> bool {
    // spec: S-0508 claim 2 (an arrival of an occupier type or an assaulter type) and Terms and claim 14 (a type with both marks behaves as an occupier type only: claim 1's test, else claim 14's assault eligibility)
    let occ = occupier(w, unit);
    if !may_occupy(w, unit, b) {
        // spec: S-0508 claim 4 (a failed attempt: no raise, destination cleared, scatter from the building's centre; r19: the target is not changed)
        crate::structures::entry::refuse(w, unit, b);
        return false;
    }
    // spec: S-0508 claim 3 (a passing attempt: one entry raise on the building's tag with the entrant as causing object, the entrant still outside and the occupants not yet cleared)
    crate::mp_triggers::entry_raise(w, b, unit);
    // The raise's actions may have removed either object or taken the unit off the map.
    let on_map = w.obj(unit).is_some_and(|o| o.on_map);
    if !on_map || w.obj(b).is_none() {
        return !on_map;
    }
    if occ {
        go_inside(w, unit, b);
        true
    } else {
        assault(w, unit, b);
        false
    }
}

/// Going inside (claim 5): the unit leaves the map at once and is added at
/// the end of the building's occupant list.
fn go_inside(w: &mut World, unit: ObjectId, b: ObjectId) {
    let Some(uo) = w.obj(unit).cloned() else {
        return;
    };
    let first = count(w, b) == 0;
    // spec: S-0508 claim 5 (it leaves the map in the same tick; it keeps its owner, health and experience)
    crate::objects::take_inside(w, unit);
    if let Some(ob) = w.obj_mut(unit) {
        ob.contact = None;
        ob.structure.inside = Some(b);
    }
    let of = w.obj(b).map(|o| TypeRef {
        kind: o.kind,
        index: o.type_index,
    });
    if let Some(bb) = w.obj_mut(b) {
        // spec: S-0508 claim 5 (added at the end of the building's occupant list; the shared index is kept, claim 18)
        bb.structure.occupants.push(unit);
    }
    // spec: S-0212 claim 16a (an infantry unit entering a building as an occupant: the building's amount is figured again)
    crate::threat::refigure(w, b);
    if first {
        // spec: S-0508 claim 6 (the garrisoned announcement and sound when the entry makes the count 1 and the entering unit's owner is the local player; the sound at the entering unit's last position)
        w.effects.push(Effect::Sound {
            sound: Sound::Garrisoned,
            of,
            house: uo.owner,
            pos: Some(uo.pos),
        });
    }
}

// ---------------------------------------------------------------------------
// The building's own turn (claims 7 to 9)

/// A civilian garrison building's own turn: forced evacuation when badly
/// damaged (claim 9), the owner following the first occupant (claim 7) and
/// the return of an abandoned building (claim 8).
pub fn turn(w: &mut World, b: ObjectId) {
    if !civilian_building(w, b) {
        return;
    }
    // spec: S-0508 claim 9 (badly damaged, as in claim 1: the occupants are evicted without hunting and without inside fallback)
    if count(w, b) > 0 && crate::targeting::badly_damaged(w, b) {
        evict(w, b, Evict::PLAIN);
    }
    let Some(o) = w.obj(b) else { return };
    let owner = o.owner;
    let civilian = civilian_house(w);
    // spec: S-0508 claim 7 (occupied and belonging to the civilian house: the owner becomes the first occupant's house)
    if count(w, b) > 0 && civilian.is_some() && owner == civilian {
        let first = occupants(w, b)
            .first()
            .and_then(|&x| w.obj(x))
            .and_then(|x| x.owner);
        if let Some(h) = first
            && Some(h) != owner
        {
            crate::structures::ownership::change(
                w,
                b,
                Some(h),
                crate::structures::ownership::Cause::Occupation,
            );
        }
        return;
    }
    // spec: S-0508 claim 8 (no occupants and not belonging to the civilian house: the owner becomes the civilian house) and claim 24 (engine design rule: with no civilian house claims 8 and 9 change no owner, at every turn, with no abandoned sound, radar event or announcement; claim 9's evacuation above still applied)
    if count(w, b) == 0
        && let Some(c) = civilian
        && owner != Some(c)
    {
        let of = TypeRef {
            kind: Kind::Building,
            index: o.type_index,
        };
        let at = o.pos;
        crate::structures::ownership::change(
            w,
            b,
            Some(c),
            crate::structures::ownership::Cause::Occupation,
        );
        // spec: S-0508 claim 8 (the abandoned feedback belongs to the owner losing the building: the sound unthrottled, the radar event at the top-left cell, the announcement only when that event is created)
        w.effects.push(Effect::Sound {
            sound: Sound::Abandoned,
            of: Some(of),
            house: owner,
            pos: Some(at),
        });
    }
}

// ---------------------------------------------------------------------------
// Eviction (claims 10 to 13)

/// The exit ring of a foundation in rank order (claim 10): the cells of
/// priority 1 to 4, then, with `quirks.garrison_exit_skips_corner` off, the
/// corner (x₀ − 1, y₀ − 1) last.
pub fn exit_ring(top_left: CellXY, foundation: (i32, i32), skips_corner: bool) -> Vec<CellXY> {
    let (x0, y0) = (top_left.x, top_left.y);
    let (fw, fh) = foundation;
    let mut out = Vec::new();
    // spec: S-0508 claim 10 (priority 1: x = x₀ + W, both corners included, larger y first)
    for y in (y0 - 1..=y0 + fh).rev() {
        out.push(CellXY::new(x0 + fw, y));
    }
    // spec: S-0508 claim 10 (priority 2: y = y₀ + H, x from x₀ − 1 to x₀ + W − 1, larger x first)
    for x in (x0 - 1..x0 + fw).rev() {
        out.push(CellXY::new(x, y0 + fh));
    }
    // spec: S-0508 claim 10 (priority 3: y = y₀ − 1, x from x₀ to x₀ + W − 1, smaller x first)
    for x in x0..x0 + fw {
        out.push(CellXY::new(x, y0 - 1));
    }
    // spec: S-0508 claim 10 (priority 4: x = x₀ − 1, y from y₀ to y₀ + H − 1, smaller y first)
    for y in y0..y0 + fh {
        out.push(CellXY::new(x0 - 1, y));
    }
    if !skips_corner {
        // spec: S-0508 Edge cases (quirks.garrison_exit_skips_corner off: the corner is tried last)
        out.push(CellXY::new(x0 - 1, y0 - 1));
    }
    out
}

/// The inside fallback cell of a foundation (claim 12): the foundation
/// rectangle's cell with the largest x and y.
pub fn inside_cell(top_left: CellXY, foundation: (i32, i32)) -> CellXY {
    // spec: S-0508 Formulas (inside fallback cell = (x₀ + W − 1, y₀ + H − 1))
    CellXY::new(top_left.x + foundation.0 - 1, top_left.y + foundation.1 - 1)
}

/// The terrain level at or below which the exit check is *clear* without
/// the later tests.
// spec: S-0508 Constants (highest terrain level receiving the exit-check bypass −6)
const EXIT_BYPASS_LEVEL: i32 = -6;

/// Whether a ring cell qualifies for the first occupant (claim 10): the
/// infantry entry check of S-0208 claim 19 answers *clear*, with the initial
/// reference level −1 and no connecting-step height restriction. There is no
/// independent legal-cell or usable-area gate: a candidate with no map cell
/// is read as the stand-in blank cell (S-0009 claim 44) and meets the same
/// check.
fn qualifies(w: &World, first: ObjectId, c: CellXY) -> bool {
    // spec: S-0508 claim 10 (a candidate with no map cell reads as the stand-in cell: level 0, flat, clear land, no overlay, no objects, S-0009 claim 44)
    let cell = w.cells.cell(c);
    let level = cell.map_or(0, |x| i32::from(x.level));
    // spec: S-0508 claim 10 and Edge cases (the reference-level bypass of S-0208 claim 19: terrain level −6 or lower is clear before the usable-area and occupancy tests; quirks.garrison_negative_level_exit)
    if w.rules().quirks.garrison_negative_level_exit && level <= EXIT_BYPASS_LEVEL {
        return true;
    }
    // spec: S-0508 claim 10 (an ordinary occupant entered from inside the usable area and keeps that status inside the building: S-0209 claim 4's rejection of a cell outside the usable area applies, whether or not the cell is on the map)
    if inside_usable_area(w, first) && !w.cells.usable(c) {
        return false;
    }
    if cell.is_none() {
        // spec: S-0508 claim 10 (a stand-in cell has no overlay and no objects: the entry check there is clear for an occupant exempt from the outside-area rejection)
        return true;
    }
    // spec: S-0508 claim 10 (S-0209's infantry entry outcome at the initial reference level −1)
    occupancy::infantry_outcome(w, first, c, EXIT_REFERENCE_LEVEL) == Outcome::Clear
}

/// Whether the occupant was inside the usable area when it went inside: its
/// retained position is in a usable cell (claim 10).
fn inside_usable_area(w: &World, id: ObjectId) -> bool {
    w.obj(id).is_some_and(|o| w.cells.usable(o.pos.cell()))
}

/// The exit cell of an eviction (claim 10): the qualifying ring cell of the
/// highest rank, judged for the first occupant alone.
pub fn exit_cell(w: &World, b: ObjectId) -> Option<CellXY> {
    let o = w.obj(b)?;
    // spec: S-0508 claim 10 (only the first occupant's view of a cell matters)
    let first = occupants(w, b).first().copied()?;
    let skips = w.rules().quirks.garrison_exit_skips_corner;
    exit_ring(o.pos.cell(), o.foundation, skips)
        .into_iter()
        .find(|&c| qualifies(w, first, c))
}

/// Puts the occupants of a building out (claims 10 to 13). Returns whether
/// the eviction completed (occupants put out, or removed with no exit).
pub fn evict(w: &mut World, b: ObjectId, opts: Evict) -> bool {
    let Some(o) = w.obj(b).cloned() else {
        return false;
    };
    if o.structure.occupants.is_empty() {
        return false;
    }
    let cell = exit_cell(w, b);
    let exit = match cell {
        Some(c) => Some(c),
        // spec: S-0508 claim 12 (no cell qualifies: with the inside fallback, its cell is the exit)
        None if opts.inside_fallback => Some(inside_cell(o.pos.cell(), o.foundation)),
        None => None,
    };
    let Some(exit) = exit else {
        // spec: S-0508 claim 12 and Edge cases (quirks.garrison_no_exit_removes: every occupant is removed, not destroyed; off: the eviction does nothing and the occupants stay inside)
        if !w.rules().quirks.garrison_no_exit_removes {
            return false;
        }
        remove_all(w, b);
        return true;
    };
    let list = occupants(w, b);
    // spec: S-0508 claim 11 (the occupants are put on the map in that one cell, from the last occupant to the first, within the same tick)
    for &id in list.iter().rev() {
        let Some(ob) = w.obj(id) else { continue };
        if ob.on_map {
            // spec: S-0508 claim 11 and 22 (an occupant that cannot be placed, here one already marked on the map, is removed, not destroyed)
            crate::objects::remove(w, id);
            continue;
        }
        put_out(w, b, id, exit, opts);
    }
    finish_action(w, b);
    true
}

/// Puts one occupant on the map at the exit cell's centre (claim 11).
fn put_out(w: &mut World, b: ObjectId, id: ObjectId, c: CellXY, opts: Evict) {
    // spec: S-0508 claim 11 (the cell's centre on its terrain surface, standing-position occupancy ignored, no random draw) and claim 22 (a living occupant off the map is placed: no second entry test, no terrain, capacity or usable-area refusal)
    let (x, y) = (256 * c.x + 128, 256 * c.y + 128); // spec: S-0204 claim 3 (the centre of a cell)
    let pos = Pos::new(x, y, w.surface_at(x, y));
    if let Some(ob) = w.obj_mut(id) {
        ob.structure.inside = None;
        ob.layer = Layer::Ground;
    }
    // spec: S-0508 claim 11 (facing 0)
    crate::objects::place(w, id, pos, PUT_OUT_FACING);
    if let Some(ob) = w.obj_mut(id) {
        // spec: S-0508 claim 11 (its target is cleared)
        crate::spawn::clear_own_target(ob);
        ob.combat.burst = 0; // spec: S-0301 claim 4 (a cleared target resets the burst counter)
    }
    // spec: S-0508 claim 11 (a scatter request whose threat point is the building's centre, forced and move-anywhere, before the next occupant is placed)
    let threat = w.reference_point(b);
    scatter::request(
        w,
        id,
        ScatterRequest {
            threat: Some(threat),
            forced: true,
            anywhere: true,
        },
    );
    if opts.hunting
        && let Some(ob) = w.obj_mut(id)
    {
        // spec: S-0508 claim 11 (a hunting eviction: the occupant is given hunt as its next behaviour; our engine has no teams to leave)
        ob.beh.queue(Some(Behaviour::Hunt));
    }
}

/// The occupants are removed, not destroyed (claims 12, 14 and 15): no
/// death, no kill, no survivors.
fn remove_all(w: &mut World, b: ObjectId) {
    let list = occupants(w, b);
    for &id in list.iter().rev() {
        if w.obj(id).is_some() {
            crate::objects::remove(w, id);
        }
    }
    finish_action(w, b);
}

/// The end of a completed whole-building action: the building has no
/// occupants and the shared index is 0 (claim 18).
fn finish_action(w: &mut World, b: ObjectId) {
    if let Some(bb) = w.obj_mut(b) {
        bb.structure.occupants.clear();
        // spec: S-0508 claim 18 (after a completed eviction or clearing action the shared index is 0)
        bb.structure.index = 0;
    }
    // spec: S-0212 claim 16a (a building's occupants leave: its amount is figured again)
    crate::threat::refigure(w, b);
}

/// *Unload* of a garrisonable building (S-0213 claim 10, S-0508 claim 13
/// row 1): the occupants are evicted without hunting and without inside
/// fallback, and the building falls back to rest. Returns the delay.
pub fn process_unload(w: &mut World, b: ObjectId) -> i32 {
    evict(w, b, Evict::PLAIN);
    if let Some(ob) = w.obj_mut(b) {
        // spec: S-0215 claim 6 (an unload that has finished falls back to rest: guard)
        ob.beh.queue(Some(Behaviour::Guard));
    }
    crate::process::standard_delay(w, b)
}

/// The occupants of a sold building come out (S-0408 claim 9, S-0508 claim
/// 13): no hunting, with the inside fallback.
pub fn sold(w: &mut World, b: ObjectId) {
    evict(w, b, Evict::SALE);
}

/// A destroyed building's occupants are evicted (S-0312 claim 17, S-0508
/// claim 13): no hunting, no inside fallback.
pub fn destroyed(w: &mut World, b: ObjectId) {
    evict(w, b, Evict::PLAIN);
}

/// The map's eviction action (S-0011 action 111): the first eligible
/// building attached to the trigger evicts without hunting or inside
/// fallback. `eligible` lists the attached buildings in order.
pub fn trigger_evict(w: &mut World, eligible: &[ObjectId]) {
    // spec: S-0508 Import mapping (action 111 selects the first eligible building attached to the firing trigger and evicts its occupants without hunting or inside fallback)
    if let Some(&b) = eligible.iter().find(|&&b| garrisoned(w, b)) {
        evict(w, b, Evict::PLAIN);
    }
}

/// The house-wide hunt order's garrison part (S-0011 action 6, S-0508 claim
/// 13): every garrisoned building of the house evicts, hunting, without
/// inside fallback.
pub fn hunt_order(w: &mut World, house: HouseId) {
    let mine: Vec<ObjectId> = w
        .objects
        .values()
        .filter(|o| o.kind == Kind::Building && o.owner == Some(house))
        .map(|o| o.id)
        .collect();
    for b in mine {
        // spec: S-0508 claim 13 (a house-wide order that sends all its units hunting: hunting eviction, no inside fallback)
        if garrisoned(w, b) {
            evict(w, b, Evict::HUNT);
        }
    }
}

// ---------------------------------------------------------------------------
// Clearing and loss (claims 14, 15, 19)

/// A passing assault (claim 14): every occupant is removed with the
/// assaulter weapon's occupant animation, the surviving assaulter is
/// recorded as destroyed by each, and the assaulter has its destination
/// cleared and scatters.
fn assault(w: &mut World, unit: ObjectId, b: ObjectId) {
    let anim = occupant_animation(w, unit);
    let reversed = w.rules().quirks.assaulter_reversed_kill_credit;
    let list = occupants(w, b);
    let base = w.obj(b).map(|o| (o.pos, o.foundation));
    // spec: S-0508 claim 14 (every occupant is removed during the attempt from the last list position to position 0; each one's animation and credit precede its removal and those of the preceding position; the animation at the muzzle point i of the list position, not the firing index)
    for (i, &id) in list.iter().enumerate().rev() {
        if w.obj(id).is_none() {
            continue;
        }
        if let Some(name) = anim.as_deref()
            && let Some((pos, f)) = base
        {
            // spec: S-0508 claim 14 (supported point numbers 0 through 9; point i for list position i) and claim 25 (engine design rule, no quirk switch: position i >= 10 uses point i mod 10, the divisor always 10, not the configured-point count, the occupant count or the firing index)
            let at = muzzle_position(w, b, i % ASSAULT_POINTS, pos, f);
            effects::anim(w, name, at);
        }
        let hit = Hit {
            amount: 0,
            warhead: None,
            distance: 0,
            attacker: None,
            house: None,
            ignore_defences: true,
            no_escape: true,
        };
        if reversed {
            // spec: S-0508 claim 14 (reversed kill credit: the surviving assaulter is recorded as destroyed by the occupant; its destruction notifications are raised and its house's loss bookkeeping runs; the occupant receives the kill credit and experience, S-0311 claims 7, 8 and 10)
            let h = Hit {
                attacker: Some(id),
                ..hit
            };
            crate::combat::damage::credit(w, unit, &h);
            crate::combat::death::destruction_raises(w, unit, &h);
        } else {
            // spec: S-0508 Edge cases (quirks.assaulter_reversed_kill_credit off: each removed occupant is the victim, with destruction notifications and loss bookkeeping, and the assaulter gets ordinary kill credit and experience)
            let h = Hit {
                attacker: Some(unit),
                ..hit
            };
            crate::combat::damage::credit(w, id, &h);
            crate::combat::death::destruction_raises(w, id, &h);
        }
        // spec: S-0508 claim 14 (the occupants themselves are removed without their own destruction notifications)
        if w.obj(id).is_some() {
            crate::objects::remove(w, id);
        }
    }
    finish_action(w, b);
    // spec: S-0508 claim 14 (the assaulter does not go inside: it has its destination cleared and scatters as in claim 4)
    crate::structures::entry::refuse(w, unit, b);
}

/// The assaulter weapon's occupant animation (claim 14, Import mapping
/// `OccupantAnim`).
fn occupant_animation(w: &World, unit: ObjectId) -> Option<String> {
    let wp = crate::combat::weapon::current_weapon(w, unit)?;
    w.rules().combat.weapon(wp).occupant_anim.clone()
}

/// The number of muzzle points an assault supports.
// spec: S-0508 Constants (supported assault muzzle points 10, numbered 0 through 9; also the engine-only remainder divisor of claim 25)
const ASSAULT_POINTS: usize = 10;

/// The position of garrison muzzle point `k` of building `b` for an
/// assault animation (S-0313 Terms: the *k*-th pixel shift of the type's
/// list of garrison muzzle points).
fn muzzle_position(w: &World, b: ObjectId, k: usize, pos: Pos, f: (i32, i32)) -> Pos {
    let _ = f;
    let (dx, dy) = assault_shift(w, b, k);
    let c = pos.cell();
    // spec: S-0313 Terms (a building's base point: the top corner of its foundation at its world height)
    Pos::new(256 * c.x + dx, 256 * c.y + dy, pos.z)
}

/// The pixel shift of garrison muzzle point `k` for an assault animation,
/// in leptons (S-0508 claim 14): no wrapping by the number of configured
/// points, a point the art does not give keeps its default (0, 0).
pub fn assault_shift(w: &World, b: ObjectId, k: usize) -> (i32, i32) {
    let Some(t) = brules(w, b) else {
        return (0, 0);
    };
    // spec: S-0508 claim 14 and Import mapping (missing points retain their imported default (0, 0); no wrap to point 0)
    let Some(&[px, py]) = t.building.garrison_muzzles.get(k) else {
        return (0, 0);
    };
    point_shift(w, px, py)
}

/// The pixel shift of garrison muzzle point `k` in leptons for a firing
/// shot (S-0313 claim 6, F3): a point that the art does not give is (0, 0).
pub fn muzzle_shift(w: &World, b: ObjectId, k: usize) -> (i32, i32) {
    let Some(t) = brules(w, b) else {
        return (0, 0);
    };
    let pts = &t.building.garrison_muzzles;
    if pts.is_empty() {
        return (0, 0);
    }
    // spec: S-0313 Edge cases (more muzzle points than ten: our engine uses every point up to the maximum occupant count); the index wraps over the points read
    let [px, py] = pts[k % pts.len()];
    point_shift(w, px, py)
}

fn point_shift(w: &World, px: i32, py: i32) -> (i32, i32) {
    // spec: S-0313 claim 6 (a point given as (65535, 65535) is (0, 0))
    if px == MUZZLE_ABSENT && py == MUZZLE_ABSENT {
        return (0, 0);
    }
    crate::combat::position::pixel_to_leptons(w, px, py)
}

/// The absent garrison muzzle point.
// spec: S-0313 claim 6 (a muzzle point given as (65535, 65535) is (0, 0))
const MUZZLE_ABSENT: i32 = 65535;

/// Erasure completes on a building (claim 15): its occupants are removed
/// with it, without death and without being put out. Called by the erasure
/// topic (`temporal::complete`, S-0514) when the erasure of a building
/// completes.
pub fn erased(w: &mut World, b: ObjectId) {
    // spec: S-0508 claim 15 (a completed erasure removes the occupants: no death, none put out)
    remove_all(w, b);
}

/// A building leaves play while it still holds occupants (S-0508 claim 15
/// as a last resort): they are removed with it.
pub fn removing(w: &mut World, b: ObjectId) {
    // spec: S-0508 claim 23 (engine design rule: an occupant that dies or is removed inside has its place removed in the same update; the others keep their order and are numbered from 0; the index is kept when below the new count, else 0, no modulo)
    if let Some(home) = w.obj(b).and_then(|o| o.structure.inside) {
        if let Some(h) = w.obj_mut(home) {
            h.structure.occupants.retain(|&x| x != b);
            let n = u32::try_from(h.structure.occupants.len()).unwrap_or(u32::MAX);
            if h.structure.index >= n {
                h.structure.index = 0;
            }
        }
        // spec: S-0212 claim 16a (an occupant killed or removed inside: the building's amount is figured again)
        crate::threat::refigure(w, home);
        return;
    }
    if w.obj(b).is_none_or(|o| o.structure.occupants.is_empty()) {
        return;
    }
    remove_all(w, b);
}

// ---------------------------------------------------------------------------
// Firing from inside (S-0308 claim 13, S-0313 claim 6, S-0311 claim 8)

/// The index of the occupant whose turn it is (S-0308 claim 13): the shared
/// index, which names a place in the list (S-0508 claim 23 keeps it below
/// the count). `None` when the building has no occupant.
pub fn current_index(w: &World, b: ObjectId) -> Option<usize> {
    let o = w.obj(b)?;
    let n = o.structure.occupants.len();
    if n == 0 {
        return None;
    }
    // spec: S-0508 claim 23 (a range check, not a reduction modulo the count: an index at or above the count is 0)
    let k = o.structure.index as usize;
    Some(if k < n { k } else { 0 })
}

/// The current occupant (S-0308 claim 13): the occupant at the shared
/// index.
pub fn current_occupant(w: &World, b: ObjectId) -> Option<ObjectId> {
    let o = w.obj(b)?;
    current_index(w, b).map(|i| o.structure.occupants[i])
}

/// A shot of the building was launched (S-0313 claim 6, S-0308 claim 13):
/// the index becomes (*k* + 1) mod *n*, unless the shot was a disc laser's.
pub fn shot_launched(w: &mut World, b: ObjectId, disc_laser: bool) {
    if disc_laser {
        // spec: S-0313 claim 6 (a disc laser shot does not advance the index)
        return;
    }
    let Some(k) = current_index(w, b) else { return };
    let n = count(w, b).max(1) as usize;
    if let Some(bb) = w.obj_mut(b) {
        // spec: S-0313 claim 6 (after every shot the index becomes (k + 1) mod n, n the occupant count then)
        bb.structure.index = ((k + 1) % n) as u32;
    }
}

/// The occupant that receives the experience of a destruction credited to a
/// garrisoned building (S-0311 claim 8): the occupant the shared index
/// selects at destruction time (`quirks.occupant_experience_next_in_turn`;
/// off: the occupant that made the last shot), whatever its trainable flag
/// (`quirks.occupant_experience_untrained`; off: a trainable one only). The
/// occupants are the remaining ones at the repaired index (S-0508 claim 23).
pub fn experience_recipient(w: &World, b: ObjectId) -> Option<ObjectId> {
    if !fires(w, b) {
        return None;
    }
    let q = w.rules().quirks;
    let o = w.obj(b)?;
    let n = o.structure.occupants.len();
    let k = current_index(w, b)?;
    // spec: S-0311 claim 8 (the recipient is the occupant selected by the shared index: the next in turn after an ordinary launch)
    let i = if q.occupant_experience_next_in_turn {
        k
    } else {
        // spec: S-0311 claim 8 (switch off: the shooter, the occupant before the selected one)
        (k + n - 1) % n
    };
    let id = *o.structure.occupants.get(i)?;
    let occ = w.obj(id)?;
    let trainable = w.rules().unit(occ.kind, occ.type_index).is_some_and(|_| {
        crate::combat::veterancy::type_combat(w, id).is_some_and(|tc| tc.trainable)
    });
    // spec: S-0311 claim 8 (quirks.occupant_experience_untrained: regardless of the occupant's trainability)
    (q.occupant_experience_untrained || trainable).then_some(id)
}
