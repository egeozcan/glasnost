//! Crews and survivors (S-0312 claims 13, 18, 19 (g), 21, 24 and 25, F4
//! and F5; S-0408 claim 7): the crew and survivor type of F4, the survivor
//! count of F5 that a destroyed and a sold building share, the survivor
//! and smudge rounds of a destroyed building and the survivor attempts of a
//! sold one.

use glasnost_map::CellXY;

use crate::combat::damage;
use crate::rules::{Behaviour, Kind};
use crate::world::{HouseId, Layer, ObjectId, Pos, Target, World};

/// What the survivor rules read of a building, taken while it is in play
/// so that claim 21's second round, after it has left play, reads the same.
#[derive(Clone, Copy, Debug)]
pub struct Source {
    /// Its owner.
    pub owner: Option<HouseId>,
    /// Its top-left foundation cell (the cell of its position).
    pub top_left: CellXY,
    /// Its foundation, width × height in cells.
    pub foundation: (i32, i32),
    /// Its type has the crewed flag.
    pub crewed: bool,
    /// It has no crew (F5): its killing damage ignored defences.
    pub no_crew: bool,
    /// It has ever changed owner (the captured mark, S-0314 Terms).
    pub captured: bool,
    /// Its recorded charge planter (Terms).
    pub planter: Option<ObjectId>,
    /// Its type produces buildings.
    pub factory: bool,
    /// It is armed in F4's sense (Terms).
    pub armed: bool,
}

/// The survivor source of building `id`, while it is in play.
pub fn source(w: &World, id: ObjectId) -> Option<Source> {
    let o = w.obj(id)?;
    if o.kind != Kind::Building {
        return None;
    }
    let r = w.rules();
    let t = r.buildings.get(o.type_index as usize)?;
    Some(Source {
        owner: o.owner,
        top_left: o.pos.cell(),
        foundation: o.foundation,
        crewed: t.building.crewed,
        no_crew: o.combat.no_crew,
        // spec: S-0312 claim 18 and F5 (has ever changed owner: S-0314's captured mark, set by every ownership change and never cleared)
        captured: o.combat.captured,
        planter: o.combat.charge_planter,
        // spec: S-0312 claim 18 (the type produces buildings: a factory of buildings, S-0403 Terms)
        factory: t.factory == Some(crate::rules::FactoryKind::Building),
        armed: armed(w, id),
    })
}

/// Whether an object is armed in F4's sense (S-0312 Terms): it has the
/// primary weapon of claim 6's second choice, or it is a building that
/// infantry can occupy and fire from.
pub fn armed(w: &World, id: ObjectId) -> bool {
    // spec: S-0312 Terms (armed: the primary weapon of claim 6's second choice, the current primary weapon)
    if crate::combat::weapon::current_weapon(w, id).is_some() {
        return true;
    }
    // spec: S-0312 Terms and S-0301 Terms (a building that infantry can occupy, whose type lets its occupants fire from it: CanBeOccupied and CanOccupyFire)
    w.obj(id).is_some_and(|o| {
        o.kind == Kind::Building
            && w.rules()
                .buildings
                .get(o.type_index as usize)
                .is_some_and(|t| t.building.can_be_occupied && t.building.can_occupy_fire)
    })
}

/// The side number of a house's country, `None` when its country belongs
/// to no side (S-0004 claim 9).
fn side(w: &World, h: Option<HouseId>) -> Option<i32> {
    let house = w.houses.get(usize::from(h?.0))?;
    w.rules()
        .econ
        .tech
        .country_side
        .get(house.country as usize)
        .copied()
        .flatten()
}

/// F4: the crew or survivor type of an object of a crewed type owned by
/// `owner`, armed or not, with its percent draw when it takes one. `None`
/// when the game-wide type it gives is missing from the data (S-0312 Edge
/// cases, *Departure: missing survivor type*).
pub fn crew_type(w: &mut World, owner: Option<HouseId>, armed: bool) -> Option<u32> {
    let rules = w.rules_arc();
    let g = &rules.combat.general;
    match side(w, owner) {
        // spec: S-0312 F4 (a country that belongs to no side: the game-wide technician, no draw)
        None => g.technician,
        // spec: S-0312 F4 (side 0, 1 or 2, unarmed: that side's crew type, no draw)
        Some(s @ 0..=2) if !armed => g.side_crew[s as usize],
        // spec: S-0312 F4 (sides 0, 1 and 2, armed)
        Some(s @ 0..=2) => {
            // spec: S-0312 F4 (side 0, 1 or 2, armed: one percent draw, below 15 the technician, otherwise the side crew type)
            if w.rng.percent() < 15 {
                g.technician
            } else {
                g.side_crew[s as usize]
            }
        }
        Some(_) => {
            if armed {
                // spec: S-0312 F4 (another side number, armed: one percent draw whose result changes nothing)
                let _ = w.rng.percent();
            }
            // spec: S-0312 F4 (another side number: the technician)
            g.technician
        }
    }
}

/// F5: the survivor count C of a building, with `refund` its type's
/// refund K for its owner as it stands now (S-0408 claim 1). Shared by a
/// destruction (S-0312 claim 18) and a sale (S-0408 claim 7).
pub fn count(w: &World, src: &Source, refund: Option<i32>) -> i32 {
    // spec: S-0312 F5 (C = 0 when the building has no crew or its type lacks the crewed flag)
    if src.no_crew || !src.crewed {
        return 0;
    }
    // spec: S-0312 F5 (C = 0 when the owner's side is not 0, 1 or 2)
    let s = match side(w, src.owner) {
        Some(s @ 0..=2) => s as usize,
        _ => return 0,
    };
    let Some(k) = refund else { return 0 };
    let mut q = i64::from(w.rules().combat.general.survivor_divisor[s]);
    if src.captured {
        // spec: S-0312 F5 (q doubled when the building has ever changed owner)
        q *= 2;
    }
    // spec: S-0312 F5 (C = 0 when q = 0: no division)
    if q == 0 {
        return 0;
    }
    // spec: S-0312 F5 (C = min(max(trunc(K / q), 1), 5), integer division truncated toward zero)
    (i64::from(k) / q).clamp(1, 5) as i32
}

/// The survivor type of claim 18's table *Type* (and S-0408 claim 7's
/// *Type choice*, the same rows), with its draws.
fn survivor_type(w: &mut World, src: &Source) -> Option<u32> {
    if !src.captured {
        // spec: S-0312 claim 18 and S-0408 claim 7 (a building that has never changed owner takes one percent draw p, whatever its type)
        let p = w.rng.percent();
        // spec: S-0312 Constants and claim 18 (a type that produces buildings: an engineer when p < 25)
        if src.factory && p < 25 {
            return w.rules().combat.general.engineer;
        }
    }
    // spec: S-0312 claim 18 (otherwise F4's type, with F4's percent draw when it takes one)
    crew_type(w, src.owner, src.armed)
}

/// Puts a created crew or survivor `id` down at `pos` in `layer` (S-0312
/// claims 13 and 18): facing map north, health R(5, `top`) and the scatter
/// request of claim 12.
// spec: S-0312 claims 13 and 18 (lowest health 5)
pub(crate) fn put_infantry(w: &mut World, id: ObjectId, pos: Pos, layer: Layer, top: i32) {
    let half = w.rules().quirks.ranged_draw_half_span;
    if let Some(o) = w.obj_mut(id) {
        o.layer = layer;
    }
    // spec: S-0312 claim 13 (it faces map north)
    crate::objects::place(w, id, pos, 0);
    // spec: S-0312 claims 13 and 18 (health a ranged draw over 5 to its top, only for one placed)
    let h = w.rng.ranged(5, top, half);
    if let Some(ob) = w.obj_mut(id) {
        ob.health = h;
        ob.combat.expected_health = h;
    }
    // spec: S-0312 claims 12, 13 and 18 (a scatter request with no threat point, forced, not move-anywhere)
    crate::scatter::request(
        w,
        id,
        crate::scatter::ScatterRequest {
            threat: None,
            forced: true,
            anywhere: false,
        },
    );
}

/// A destruction survivor of type `ty` for `owner` in foundation cell `c`
/// (S-0312 claim 18): S-0202 claim 10's creation draw whether or not it is
/// then placed, then its standing position as for an absorbed infantry
/// passenger (claim 24): the centre class, S-0208 claim 3's table on the
/// ground record with its R(0, 3) unless the first row refuses the cell.
/// The chosen position is kept: no entry test. `None` when the table gives
/// no position: the unit never enters play and takes no health draw.
fn place_survivor(
    w: &mut World,
    ty: u32,
    owner: Option<HouseId>,
    c: CellXY,
    top: i32,
) -> Option<ObjectId> {
    let id = crate::objects::create(w, Kind::Infantry, ty, owner);
    // spec: S-0312 claims 18 and 24 (the desired point 36 leptons from the cell centre along y: the centre class; S-0208 claim 3 on the ground record, R(0, 3) unless the first row refuses; the building-present flag plays no part)
    let Some(k) =
        crate::walk::choose_position(w, c, Layer::Ground, crate::cells::position::CENTRE, false)
    else {
        // spec: S-0312 claim 18 (no position: it never enters play and takes no health draw; the creation draw is already taken)
        w.objects.remove(&id);
        return None;
    };
    let (u, v) = crate::walk::standing_point(k);
    let (x, y) = (256 * c.x + u, 256 * c.y + v); // spec: S-0208 table P (a standing position within the cell)
    let z = w.surface_at(x, y);
    // spec: S-0312 claim 18 (the chosen position is kept: no second choice, no further draw and no entry test)
    put_infantry(w, id, Pos::new(x, y, z), Layer::Ground, top);
    Some(id)
}

/// Whether a house is human-controlled in claim 25's sense: a player
/// house, an AI house included (no owner counts as one).
fn player(w: &World, owner: Option<HouseId>) -> bool {
    owner.is_none_or(|h| w.houses.get(usize::from(h.0)).is_some_and(|x| x.human))
}

/// A destruction survivor's behaviour (claims 18 and 25): it attacks the
/// building's charge planter when the planter is in the game and the owner
/// is not allied with it; otherwise *move* for a player house, *hunt* for
/// the neutral or special house.
fn survivor_orders(w: &mut World, id: ObjectId, src: &Source) {
    let stale = w.rules().quirks.stale_suspended_revert; // spec: S-0215 Edge cases (quirks.stale_suspended_revert)
    let tick = w.tick;
    // spec: S-0312 claim 18 (the charge planter, while it is in the game, and the owner not allied with it; the attacker of the killing hit plays no part)
    let planter = src
        .planter
        .filter(|&p| w.obj(p).is_some())
        .filter(|&p| !w.allied(src.owner, w.obj(p).and_then(|x| x.owner)));
    let human = player(w, src.owner);
    let Some(ob) = w.obj_mut(id) else { return };
    if let Some(p) = planter {
        // spec: S-0312 claim 18 (the survivor is ordered to attack the planter: its target, attack current)
        crate::spawn::set_own_target(ob, Some(Target::Object(p)));
        ob.beh.force(Behaviour::Attack, tick, stale);
        return;
    }
    // spec: S-0312 claim 25 (a survivor without an attack order: move for a player house, human or AI; hunt for the neutral or special house)
    let b = if human {
        Behaviour::Move
    } else {
        Behaviour::Hunt
    };
    ob.beh.force(b, tick, stale);
}

/// Whether a foundation cell is clear for movement for claim 19 (g)'s
/// smudge step: three tests, none of them a passability category.
fn clear_for_movement(w: &World, c: CellXY) -> bool {
    let Some(cell) = w.cells.cell(c) else {
        return false;
    };
    let r = w.rules();
    // spec: S-0312 claim 19 (g) and S-0211 claim 15 (a high-bridge cell reads the bridge-deck record and makes no test 3; every other cell the ground record)
    let deck = w.cells.idx(c).is_some_and(|i| w.cells.deck[i]);
    let layer = if deck { Layer::Deck } else { Layer::Ground };
    // spec: S-0312 claim 19 (g) test 1 (the object-present flag of that record fails it; infantry positions, terrain marks and the vehicle-present and building-present flags are ignored)
    if w.cells.occ(c, layer).is_some_and(|o| o.others > 0) {
        return false;
    }
    // spec: S-0312 claim 19 (g) test 2 (an overlay type with the wall flag fails it)
    if cell.overlay != glasnost_map::NO_OVERLAY
        && r.overlays
            .get(usize::from(cell.overlay))
            .is_some_and(|o| o.wall)
    {
        return false;
    }
    // spec: S-0312 claim 19 (g) test 3 (the tracked class's multiplier M(ℓ, track) of the cell's land type exactly 0 fails it, not merely below S-0211's 0.01; not made on a high-bridge cell)
    if !deck
        && r.multiplier(cell.land, glasnost_rules::sim::LocomotionClass::Track)
            .to_f64()
            .eq_num(glasnost_num::F64::ZERO)
    {
        return false;
    }
    true
}

/// One survivor round of a destroyed building (S-0312 claims 18 and 19
/// (g)): with C of F5 above 0, each foundation cell in S-0302's order from
/// cell `skip` on (the cells used by absorbed passengers come first, claim
/// 24) takes the survivor chance, the survivor and the smudge step, in
/// that order. Returns the number of survivors released.
pub fn destruction_round(w: &mut World, src: &Source, refund: Option<i32>, skip: usize) -> i32 {
    let c = count(w, src, refund);
    // spec: S-0312 claim 18 (C = 0: nothing in this claim happens, and no smudge step of claim 19 (g))
    if c == 0 {
        return 0;
    }
    let half = w.rules().quirks.ranged_draw_half_span;
    // spec: S-0312 claim 18 and Constants (g = 2, or 1 with a charge planter, plus 6 when the building has ever changed owner)
    let g = if src.planter.is_some() { 1 } else { 2 } + if src.captured { 6 } else { 0 };
    let mut released = 0;
    // spec: S-0312 claims 18 and 24 (the foundation cells in S-0302's claim 13 order, starting after the cells absorbed passengers used; beyond the last cell, none)
    let cells = damage::foundation_order(src.top_left, src.foundation);
    for &cell in cells.iter().skip(skip) {
        // spec: S-0312 claim 18 (the chance: fewer than C released and the owner's game not over, S-0414's defeated state)
        let over = src.owner.is_some_and(|h| crate::seams::game_over(w, h));
        if released < c && !over {
            // spec: S-0312 claim 18 (one ranged draw R(0, g); a result of 1 releases one survivor)
            let roll = w.rng.ranged(0, g, half);
            if roll == 1
                // spec: S-0312 claim 18 and Edge cases (the type draws; a missing type: no survivor, no creation or health draw)
                && let Some(ty) = survivor_type(w, src)
            {
                let top = w.rules().unit(Kind::Infantry, ty).map_or(0, |u| u.strength);
                // spec: S-0312 claim 18 (the creation draw whether or not it can be placed; health R(5, S); one not placed does not count against C)
                if let Some(id) = place_survivor(w, ty, src.owner, cell, top) {
                    survivor_orders(w, id, src);
                    released += 1;
                }
            }
        }
        // spec: S-0312 claim 19 (g) (every cell in the order has the smudge step when it is clear for movement, whether or not survivors remain)
        if clear_for_movement(w, cell) {
            // spec: S-0312 claim 19 (g) and Constants (one percent draw: below 50 a scorch mark, otherwise a crater)
            let _scorch = w.rng.percent() < 50;
            // spec: S-0312 claim 19 (g) (then the raw value of the nearby point N(cell centre, 128, b))
            let raw = w.rng.raw();
            let _at = crate::combat::death::nearby_point(w, Pos::centre(cell, 0), 128, raw, false);
            // spec: S-0312 claim 26 (the plain form: with no fitting smudge type, n = 0, no draw and no smudge)
            w.note("S-0312 claim 26: smudge types, the smudges of cells and the morphable flag of tilesets are not modelled, so no smudge type fits: n = 0, no draw and no smudge");
        }
    }
    released
}

/// The survivors of a sold building (S-0408 claim 7), after its absorbed
/// passengers and garrison: exactly C attempts, C of S-0312 F5 with the
/// refund as it stands now, no chance draw. Returns the number placed.
pub fn sold(w: &mut World, id: ObjectId) -> i32 {
    let Some(src) = source(w, id) else { return 0 };
    let refund = crate::seams::building_refund(w, id);
    // spec: S-0408 claim 7 and S-0312 F5 (the same count as a destruction; a building with no crew releases none)
    let c = count(w, &src, refund);
    let half = w.rules().quirks.ranged_draw_half_span;
    // spec: S-0408 claim 7 (the building's n foundation cells in the cell order of S-0302 claim 13)
    let cells = damage::foundation_order(src.top_left, src.foundation);
    let n = cells.len() as i32;
    let mut engineer_chosen = false;
    let mut placed = 0;
    for _ in 0..c {
        let mut ty = None;
        // spec: S-0408 Constants and Edge cases (at most 100 type choices for one survivor: a choice that always gives a second engineer releases no survivor, its draws taken)
        for _ in 0..100 {
            // spec: S-0408 claim 7 (the type choice, with its draws, first in every attempt)
            let t = survivor_type(w, &src);
            // spec: S-0408 Terms (an engineer type: an infantry type with the engineer flag)
            let engineer = t.is_some_and(|t| {
                w.rules()
                    .unit(Kind::Infantry, t)
                    .is_some_and(|u| u.engineer)
            });
            // spec: S-0408 claim 7 (the whole choice again while it gives an engineer and this sale has already chosen one)
            if engineer && engineer_chosen {
                continue;
            }
            // spec: S-0408 claim 7 (the first engineer chosen counts even if its attempt is lost)
            engineer_chosen |= engineer;
            ty = Some(t);
            break;
        }
        // spec: S-0312 Edge cases (a missing survivor type: no survivor, no creation draw), applied to a sale's attempt
        let Some(Some(t)) = ty else { continue };
        let owner = src.owner;
        // spec: S-0408 claim 7 (the creation draw once the type is chosen)
        let uid = crate::objects::create(w, Kind::Infantry, t, owner);
        // spec: S-0408 claim 7 (R(0, n − 1): one of the foundation cells)
        let k = w.rng.ranged(0, n - 1, half);
        let Some(&cell) = cells.get(k as usize) else {
            w.objects.remove(&uid);
            continue;
        };
        // spec: S-0408 claim 7 and S-0208 claim 3 (the desired point 36 leptons from the cell centre, class centre: R(0, 3) unless the cell's record blocks every position, then the first free position of that row; no search of neighbouring cells)
        let Some(p) = crate::walk::choose_position(
            w,
            cell,
            Layer::Ground,
            crate::cells::position::CENTRE,
            false,
        ) else {
            // spec: S-0408 claim 7 (no position free: no survivor, the attempt lost, its draws taken)
            w.objects.remove(&uid);
            continue;
        };
        let (u, v) = crate::walk::standing_point(p);
        let (x, y) = (256 * cell.x + u, 256 * cell.y + v); // spec: S-0208 table P (a standing position within the cell)
        let z = w.surface_at(x, y);
        // spec: S-0408 claim 7 (facing 0, full strength: no health draw)
        crate::objects::place(w, uid, Pos::new(x, y, z), 0);
        // spec: S-0408 claim 7 (a forced scatter request)
        crate::scatter::request(
            w,
            uid,
            crate::scatter::ScatterRequest {
                threat: None,
                forced: true,
                anywhere: false,
            },
        );
        let stale = w.rules().quirks.stale_suspended_revert; // spec: S-0215 Edge cases (quirks.stale_suspended_revert)
        let tick = w.tick;
        if let Some(ob) = w.obj_mut(uid) {
            // spec: S-0408 claim 7 (on the move behaviour, whoever owns it)
            ob.beh.force(Behaviour::Move, tick, stale);
        }
        placed += 1;
    }
    placed
}
