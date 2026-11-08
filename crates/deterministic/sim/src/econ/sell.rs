//! Selling and building repair (S-0408; repair itself is S-0314's): the
//! refund, which buildings can be sold, the course of a sale, selling an
//! upgrade, a vehicle or a wall, and the repair switch. A building that
//! packs up goes through the same course (S-0415 claims 9 to 11, in
//! `crate::econ::deploy`).

use glasnost_map::{CellXY, NO_OVERLAY};
use glasnost_num::F64;
use serde::{Deserialize, Serialize};

use super::TypeRef;
use crate::combat::effects::{Effect, Sound};
use crate::rules::{Behaviour, Kind};
use crate::world::{HouseId, HouseRole, Layer, ObjectId, World};

/// A sale in progress (S-0408 claim 6).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sale {
    /// Where the sale stands.
    pub stage: Stage,
    /// The building packs up instead of being sold (S-0213 claim 12,
    /// S-0415 claim 9), as judged when the order was applied.
    pub packs: bool,
}

/// The states of a sale (S-0408 claim 6).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Stage {
    /// Ordered; the selling behaviour has not been processed yet.
    #[default]
    Ordered,
    /// The first processing is done: docked units told to leave.
    Begun,
    /// Build-down running since the given tick (S-0108 claim 5).
    BuildDown(i32),
}

// ---------------------------------------------------------------------------
// The refund (claims 1 and 2).

/// The refund override *v* of a type (S-0408 Constants: absent 0).
fn refund_override(w: &World, t: TypeRef) -> i32 {
    let r = w.rules();
    let v = match t.kind {
        Kind::Building => r
            .buildings
            .get(t.index as usize)
            .and_then(|b| b.raw.econ.soylent),
        k => r.unit(k, t.index).and_then(|u| u.raw.econ.soylent),
    };
    // spec: S-0408 Constants (refund override absent-key default 0)
    v.unwrap_or(0)
}

/// The refund a house receives for an object of type `t` (S-0408 claim 1,
/// F1).
pub fn refund(w: &World, house: HouseId, t: TypeRef) -> i32 {
    refund_at(w, house, t, false)
}

/// The refund of S-0408 claim 1 with the refund percentage taken as 1: the
/// amount *P*, or the refund-override form (S-0402 claim 18's free unit).
pub fn refund_whole(w: &World, house: HouseId, t: TypeRef) -> i32 {
    refund_at(w, house, t, true)
}

fn refund_at(w: &World, house: HouseId, t: TypeRef, whole: bool) -> i32 {
    let mode = w.rules().mode;
    // spec: S-0408 claim 1 (a and b of S-0404 claim 2 as they stand now)
    let (a, b) = crate::econ::production::cost_factors(w, house, t);
    let v = refund_override(w, t);
    if v != 0 {
        // spec: S-0408 F1 (v ≠ 0: trunc(RZ₅₃(v × a)))
        return F64::from_i64(i64::from(v), mode)
            .mul(a, mode)
            .to_i32_common();
    }
    let c = match t.kind {
        Kind::Building => adjusted_cost(w, t.index),
        _ => crate::econ::production::cost_field(w, t),
    };
    // spec: S-0408 F1 (P = trunc(RZ₅₃(RZ₅₃(C × b) × a)))
    let p = F64::from_i64(i64::from(c), mode)
        .mul(b, mode)
        .mul(a, mode)
        .to_i32_common();
    // spec: S-0408 claim 1 (a computer-controlled house receives P in full: the neutral and special houses, and a test-only computer-controlled player house, glossary; every other player house, AI houses included, the refund)
    let player = w
        .houses
        .get(usize::from(house.0))
        .is_some_and(|h| h.role == HouseRole::Player && h.human);
    if !player || whole {
        // spec: S-0408 F1 and claim 1 (the neutral and special houses receive P) and S-0402 claim 18 (the percentage taken as 1)
        return p;
    }
    // spec: S-0408 Terms and F1 (r narrowed to single precision when used)
    let r = w
        .rules()
        .econ
        .placement
        .refund_percent
        .to_f32(mode)
        .to_f64();
    // spec: S-0408 F1 (refund = trunc(RZ₅₃(P × r)) for a player house)
    F64::from_i64(i64::from(p), mode)
        .mul(r, mode)
        .to_i32_common()
}

/// Pays `amount` credits to a house (a negative refund lowers its
/// credits, S-0408 Edge cases).
pub(crate) fn credit(w: &mut World, house: HouseId, amount: i32) {
    if let Some(h) = w.houses.get_mut(usize::from(house.0)) {
        h.credits = h.credits.wrapping_add(amount);
    }
}

// ---------------------------------------------------------------------------
// Sellable and repairable (claims 3 and 14; input's tests).

/// Whether a building's type has a build-up sprite (S-0108 claim 1;
/// S-0408 claims 3 and 4).
pub fn has_buildup(w: &World, t: u32) -> bool {
    w.rules()
        .buildings
        .get(t as usize)
        .is_some_and(|b| b.raw.econ.buildup.unwrap_or(false))
}

/// The firestorm of a house (S-0211 Terms). No event turns it on (S-0211
/// claim 11), so it is never active.
fn firestorm_active(w: &World, house: HouseId) -> bool {
    let _ = (w, house);
    false
}

/// Whether a building is sellable (S-0408 claim 3).
pub fn sellable(w: &World, id: ObjectId) -> bool {
    let Some(o) = w.obj(id) else { return false };
    if o.kind != Kind::Building {
        return false;
    }
    let b = &w.rules().buildings[o.type_index as usize];
    // spec: S-0408 claim 3 (not being drained; the unsellable flag absent-key default no)
    if crate::econ::power::being_drained(w, id) || b.raw.econ.unsellable.unwrap_or(false) {
        return false;
    }
    // spec: S-0408 claim 3 (a) (a build-up sprite, construction completed, neither construction nor selling)
    let a = has_buildup(w, o.type_index)
        && !o.econ.construction.building_up
        // spec: S-0406 claim 17 (the queued construction is seen while none is current)
        && !matches!(
            crate::econ::placement::seen_behaviour(o),
            Some(Behaviour::Construction | Behaviour::Selling)
        );
    // spec: S-0408 claim 3 (b) (the firestorm-wall flag and the house's firestorm not active)
    let fw =
        b.raw.firestorm_wall.unwrap_or(false) && !o.owner.is_some_and(|h| firestorm_active(w, h));
    a || fw
}

/// Whether a building is a vehicle-like building (S-0214 Terms): its type
/// undeploys into a vehicle and has a 1 × 1 foundation.
pub fn vehicle_like(w: &World, id: ObjectId) -> bool {
    w.obj(id).is_some_and(|o| {
        o.kind == Kind::Building
            && crate::econ::deploy::undeploys_into(w, o.type_index).is_some()
            && w.rules().buildings[o.type_index as usize].foundation == (1, 1)
    })
}

/// Whether a building is repairable (S-0408 claim 14).
pub fn repairable(w: &World, id: ObjectId) -> bool {
    let Some(o) = w.obj(id) else { return false };
    if o.kind != Kind::Building {
        return false;
    }
    let b = &w.rules().buildings[o.type_index as usize];
    let s = crate::combat::damage::strength(w, id);
    // spec: S-0408 claim 14 and Constants (health in (0, S); repairable and click-repairable, absent yes; not vehicle-like)
    o.health > 0
        && o.health < s
        && b.raw.repairable.unwrap_or(true)
        && b.raw.econ.click_repairable.unwrap_or(true)
        && !vehicle_like(w, id)
}

// ---------------------------------------------------------------------------
// Orders.

/// Applies a sell order (S-0213 claim 12, S-0408 claims 4 and 11).
pub fn sell(w: &mut World, house: HouseId, actor: ObjectId) {
    // spec: S-0213 claim 12 (the actor must be live and owned by the issuing house)
    let Some(o) = w.obj(actor) else { return };
    if o.owner != Some(house) {
        return;
    }
    match o.kind {
        Kind::Building => sell_building(w, actor),
        Kind::Vehicle | Kind::Aircraft => sell_unit(w, house, actor),
        _ => {}
    }
}

/// An automatic packing-up requested by a deployed slave master (S-0402
/// claim 22): the building goes the way of a sell order that makes it pack
/// up (S-0415 claims 9 and 10). The caller has checked that it packs up.
pub(crate) fn start_pack_up(w: &mut World, id: ObjectId) {
    sell_building(w, id);
}

fn sell_building(w: &mut World, id: ObjectId) {
    let stale = w.rules().quirks.stale_suspended_revert; // spec: S-0215 Edge cases (quirks.stale_suspended_revert)
    let Some(o) = w.obj(id) else { return };
    let t = o.type_index;
    if !has_buildup(w, t) {
        if w.rules().buildings[t as usize]
            .raw
            .firestorm_wall
            .unwrap_or(false)
        {
            // spec: S-0408 claim 4 (no build-up sprite, firestorm-wall flag: removed at once, no refund, no survivors)
            crate::objects::remove(w, id);
        }
        // spec: S-0408 claim 4 (no build-up sprite otherwise: nothing happens)
        return;
    }
    if o.beh.seen() == Some(Behaviour::Selling) {
        // spec: S-0408 claim 4 (a second sell order changes nothing)
        return;
    }
    // spec: S-0213 claim 12 (a building whose type undeploys into a vehicle packs up under its conditions)
    let packs = crate::econ::deploy::packs_up(w, id);
    let tick = w.tick;
    // spec: S-0108 claims 4 and 6 (a sale runs the construction range: the idle body is left)
    crate::econ::body::leave_idle(w, id);
    if let Some(ob) = w.obj_mut(id) {
        // spec: S-0408 claim 4 (the building's behaviour becomes selling)
        ob.beh.force(Behaviour::Selling, tick, stale);
        ob.econ.sale = Some(Sale {
            stage: Stage::Ordered,
            packs,
        });
    }
    // spec: S-0314 claim 22 (a depot that starts selling gives every vehicle in contact a run-away notice)
    crate::depot::sale_started(w, id);
    // spec: S-0501 claim 24 (r110, Q-2437: a pad that starts selling sends every aircraft in contact with it the run-away notice of S-0314 claim 22)
    crate::air::sale_started(w, id);
}

/// A sell order on a vehicle or aircraft (S-0408 claim 11).
fn sell_unit(w: &mut World, house: HouseId, id: ObjectId) {
    let Some(o) = w.obj(id) else { return };
    let c = o.pos.cell();
    let pos = o.pos;
    let t = TypeRef {
        kind: o.kind,
        index: o.type_index,
    };
    // spec: S-0213 claim 12 (sold only if a building occupies its cell)
    let on_building = w
        .cells
        .list(c, Layer::Ground)
        .iter()
        .any(|&x| w.obj(x).is_some_and(|b| b.kind == Kind::Building));
    if !on_building {
        return;
    }
    let amount = refund(w, house, t);
    // spec: S-0408 claim 11 (removed at once, no animation, no survivors; its house receives the refund)
    crate::objects::remove(w, id);
    credit(w, house, amount);
    // spec: S-0408 claim 11 and S-0507 claim 16 (a sold transport's remaining passengers die without attacker credit, in objects::remove)
    w.effects.push(Effect::Sound {
        sound: Sound::UnitSold,
        of: Some(t),
        house: Some(house),
        pos: Some(pos),
    });
}

/// Applies a sell-wall order (S-0213 claim 12, S-0408 claim 12).
pub fn sell_wall(w: &mut World, house: HouseId, cell: CellXY) {
    wall_sale(w, house, cell, false);
}

/// A wall sale by `house` (S-0408 claim 12): removes the wall in `cell`
/// when it has a player house as its owner and its builder exists without
/// the unsellable flag; the owner need not be `house`. `silent`: the
/// removal under a placed building (S-0406 claim 9), which plays no sound.
/// Returns whether the wall was removed.
pub fn wall_sale(w: &mut World, house: HouseId, cell: CellXY, silent: bool) -> bool {
    let Some(i) = w.cells.idx(cell) else {
        return false;
    };
    let Some(ov) = w.cells.cell(cell).map(|c| c.overlay) else {
        return false;
    };
    // spec: S-0408 claim 12 (the cell holds a wall overlay)
    if ov == NO_OVERLAY || !crate::walls::is_wall(w, ov) {
        return false;
    }
    // spec: S-0408 claim 12 (the wall has an owner, a player house in our engine)
    let Some(owner) = w.cells.wall_owner[i] else {
        return false;
    };
    if !crate::econ::placement::player_house(w, owner) {
        return false;
    }
    // spec: S-0408 claim 12 (the builder of the wall overlay exists and does not have the unsellable flag)
    let Some(builder) = crate::econ::placement::wall_builder(w, ov) else {
        return false;
    };
    if w.rules().buildings[builder as usize]
        .raw
        .econ
        .unsellable
        .unwrap_or(false)
    {
        return false;
    }
    if let Some(c) = w.cells.grid.get_mut(cell) {
        c.overlay = NO_OVERLAY;
        c.overlay_data = 0;
    }
    // spec: S-0408 claim 12 (the wall leaves its cell with no owner)
    w.cells.wall_owner[i] = None;
    // spec: S-0112 claim 7 (a sold wall: c and its four edge neighbours)
    crate::walls::recompute(w, &crate::walls::with_neighbours(cell));
    // spec: S-0408 claim 12 (passability and the pathfinding regions are worked out again)
    crate::load::recalc_cells(w, &[cell]);
    crate::regions::cell_changed(w, cell);
    if !w.rules().quirks.wall_sale_no_refund {
        // spec: S-0408 Edge cases (quirks.wall_sale_no_refund off: the refund of the wall overlay's builder)
        let amount = refund(
            w,
            house,
            TypeRef {
                kind: Kind::Building,
                index: builder,
            },
        );
        credit(w, house, amount);
    }
    if !silent {
        // spec: S-0408 claim 12 (the sell sound for the issuing house's player; none for a silent wall sale)
        w.effects.push(Effect::Sound {
            sound: Sound::Sell,
            of: None,
            house: Some(house),
            pos: None,
        });
    }
    true
}

/// Applies a repair order (S-0213 claim 13): switches the actor's repair
/// on or off; it needs only a live actor.
pub fn repair(w: &mut World, actor: ObjectId) {
    if let Some(o) = w.obj_mut(actor) {
        // spec: S-0213 claim 13 (repair switched on if off and off if on)
        o.econ.building.repairing = !o.econ.building.repairing;
    }
}

/// A building's repair in its turn (S-0408 claim 13, S-0314 claim 8): a
/// step at each tick that is a multiple of the building repair period
/// *P*<sub>b</sub> while its repair is on.
pub fn building_turn(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id) else { return };
    if o.health <= 0 {
        return;
    }
    // spec: S-0314 claim 12 (every ordinary living building turn that reaches repair work considers automatic action, before claim 8's step)
    automatic(w, id);
    let Some(o) = w.obj(id) else { return };
    if !o.econ.building.repairing {
        return;
    }
    let pb = w.rules().combat.general.repair_period;
    // spec: S-0314 claim 8 (one step at each tick whose number is a multiple of P_b)
    if !crate::combat::heal::multiple(w.tick, pb) {
        return;
    }
    // spec: S-0314 claim 8 (the step's outcome; repair turns off when it says so)
    if let Some(on) = crate::combat::heal::building_repair_step(w, id)
        && !on
        && let Some(ob) = w.obj_mut(id)
    {
        ob.econ.building.repairing = false;
    }
}

/// Automatic building repair and sale (S-0314 claims 12 to 15).
fn automatic(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id) else { return };
    let Some(owner) = o.owner else { return };
    let Some(h) = w.houses.get(usize::from(owner.0)) else {
        return;
    };
    let g = &w.rules().combat.general;
    let (threshold, reserve, sell_back) = (g.iq_repair_sell, g.credit_reserve, g.iq_sell_back);
    // spec: S-0314 claim 8 (the gate: the house skill level is at least the repair-and-sell threshold)
    if h.skill < threshold {
        return;
    }
    // spec: S-0314 claim 12 (the behaviour is neither construction nor selling)
    if matches!(
        o.beh.seen(),
        Some(Behaviour::Construction | Behaviour::Selling)
    ) {
        return;
    }
    // spec: S-0314 claim 12 (repairable under S-0408 claim 14: a full-health building is ineligible)
    if !repairable(w, id) {
        return;
    }
    let (human, held) = (h.human, h.combat.repair_hold);
    let (repairing, captured, flagged, hostile) = (
        o.econ.building.repairing,
        o.combat.captured,
        o.combat.computer_repair,
        o.combat.hostile_hit,
    );
    // spec: S-0314 claim 12 (credits at least the reserve: repair only; below it: sale only; signed, equality is repair's)
    if crate::seams::available_credits(w, owner) >= reserve {
        // spec: S-0314 claim 13 (repair already on neither sets the hold nor draws; the hold blocks a start)
        if repairing || held {
            return;
        }
        // spec: S-0314 claim 13 (a human-controlled owner, an AI house included, or the captured mark or the computer-repair flag)
        if !(human || captured || flagged) {
            return;
        }
        if let Some(ob) = w.obj_mut(id) {
            // spec: S-0314 claim 13 (the start takes no payment and changes no health; claim 8's next step does)
            ob.econ.building.repairing = true;
        }
        if let Some(hh) = w.houses.get_mut(usize::from(owner.0)) {
            // spec: S-0314 claim 13 (the start sets the house's hold)
            hh.combat.repair_hold = true;
        }
        if !human {
            // spec: S-0314 claim 15 and F4 (a neutral or special house's delay, or a test-only computer-controlled player house's, glossary: R(0, 0) for d = 0, no raw value; the hold is released in this tick's house update)
            let half = w.rules().quirks.ranged_draw_half_span;
            let _ = w.rng.ranged(0, 0, half);
        }
        return;
    }
    // spec: S-0314 claim 14 (below the reserve: only a building with the hostile-hit mark)
    if !hostile {
        return;
    }
    // spec: S-0314 Terms (the sale skill stays 0 in a skirmish)
    let sale_skill: i32 = 0;
    // spec: S-0314 claim 14 (sale skill at least the sale threshold, both read as unsigned 32-bit values)
    if (sale_skill as u32) < (sell_back as u32) {
        return;
    }
    let half = w.rules().quirks.ranged_draw_half_span;
    // spec: S-0314 claim 14 and Constants (one ranged draw R(0, 50) on each eligible turn, taken even when a later test fails)
    let k = w.rng.ranged(0, 50, half);
    let tech = crate::econ::tech::house_tech_level(w, owner);
    // spec: S-0314 claim 14 (success: the result strictly below the owner's tech level, read as unsigned)
    if (k as u32) >= (tech as u32) {
        return;
    }
    let Some(o) = w.obj(id) else { return };
    let r = w.rules();
    let mode = r.mode;
    // spec: S-0314 claim 14 (no tag)
    let tagged = w.triggers.objects.contains_key(&id);
    // spec: S-0314 claim 14 (its type is not a factory for buildings)
    let factory =
        r.buildings[o.type_index as usize].factory == Some(crate::rules::FactoryKind::Building);
    let s = crate::combat::damage::strength(w, id);
    // spec: S-0314 claim 14 (the floating-point health ratio strictly below the red-health threshold)
    let ratio =
        F64::from_i64(i64::from(o.health), mode).div(F64::from_i64(i64::from(s), mode), mode);
    let red = ratio.lt(r.general.condition_red);
    // spec: S-0314 claim 14 (a pending delayed kill prevents the sale)
    let dying = o.combat.delay_kill_until.is_some();
    if !tagged && !factory && red && !dying {
        // spec: S-0314 claim 14 (the effect of S-0408 claim 4; the input sellability test is not a condition)
        sell_building(w, id);
    }
}

// ---------------------------------------------------------------------------
// The course of a sale (claims 5 to 10, 16).

/// The selling behaviour's processing delay (S-0408 claim 16).
// spec: S-0408 Constants (selling processing delay: 1 tick, no draw)
pub const SELLING_DELAY: i32 = 1;

/// The fewest stages of a range whose report falls on its last stage
/// (S-0406 claims 15 and 18, S-0408 claim 17).
// spec: S-0406 F6 and S-0408 F4 (n ≥ 2 stages)
const MULTI_STAGE: i32 = 2;

/// A building type's construction range (S-0108 claim 5): its stage count
/// *n* and its stage delay *d_b* in ticks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Range {
    /// The stage count *n*.
    pub n: i32,
    /// The stage delay *d_b*, in ticks.
    pub delay: i32,
}

impl Range {
    /// The ticks from the start of the range to the building turn whose
    /// completion report a construction sees (S-0406 claims 15, 17 and 18,
    /// F6 and F7); none when the range never reports completion.
    pub fn report_after(self, never: bool) -> Option<i32> {
        if self.delay <= 0 {
            // spec: S-0406 claim 15 and F6 (a zero-delay range reports completion at the next building turn: H = C + 1)
            Some(1)
        } else if self.n >= MULTI_STAGE {
            // spec: S-0406 claim 15, F6 and F7 (the report on the turn the stage lands on the last stage, (n − 1) × d_b)
            Some((self.n - 1).saturating_mul(self.delay))
        } else if never {
            // spec: S-0406 claim 18 (quirks.single_stage_buildup_never_completes: a positive-delay range of one stage or fewer never reports completion)
            None
        } else {
            // spec: S-0406 Edge cases (switch off: the report on the turn the stage first wraps, d_b ticks after the range starts)
            Some(self.delay)
        }
    }
}

/// The construction range of building type `t` (S-0108 claim 5, F1).
pub fn buildup_range(w: &World, t: u32) -> Range {
    let r = w.rules();
    let mode = r.mode;
    // spec: S-0108 claim 5 (no build-up sprite: range a = 0, c = 1, d = 0)
    let none = Range { n: 1, delay: 0 };
    let Some(b) = r.buildings.get(t as usize) else {
        return none;
    };
    if !b.raw.econ.buildup.unwrap_or(false) {
        return none;
    }
    let n = if b.raw.gate.unwrap_or(false) {
        // spec: S-0108 claim 5 and Import mapping (a gate: n = G + 1, GateStages absent 9; S-0406 claim 15)
        b.raw.econ.gate_stages.unwrap_or(9).saturating_add(1)
    } else {
        // spec: S-0108 claim 5 (n = ⌊N_b / 2⌋; unknown frame counts read as n = 0)
        b.raw.econ.buildup_frames.unwrap_or(0).div_euclid(2)
    };
    let db = if n <= 0 {
        // spec: S-0108 claim 5 (n ≤ 0: d_b = 1)
        1
    } else {
        // spec: S-0108 claim 5 and F1 (d_b = trunc(B × 900 / n))
        r.econ
            .placement
            .buildup_time
            .mul(F64::from_i64(900, mode), mode)
            .div(F64::from_i64(i64::from(n), mode), mode)
            .to_i32_common()
    };
    Range { n, delay: db }
}

/// The ticks from the start of a build-up (or build-down) to its last
/// stage (S-0108 claim 5, F1): (*n* − 1) × *d_b*, never below 0.
pub fn buildup_ticks(w: &World, t: u32) -> i32 {
    let r = buildup_range(w, t);
    // spec: S-0108 claim 5 (the last stage (n − 1) × d_b ticks after the state began)
    (r.n - 1).saturating_mul(r.delay).max(0)
}

/// Whether any unit is docked with a building (S-0408 Terms): a harvester
/// holding a dock slot and contact (S-0402). Seam: the other services that
/// make such links (repair depots, pads) are other topics'.
fn any_docked(w: &World, id: ObjectId) -> bool {
    !crate::econ::harvest::docked(w, id).is_empty()
}

/// The selling behaviour's processing (S-0408 claims 5, 6 and 16; S-0415
/// claim 9). Returns the delay.
pub fn process_selling(w: &mut World, id: ObjectId) -> i32 {
    let Some(o) = w.obj(id).cloned() else {
        return SELLING_DELAY;
    };
    // spec: S-0408 claim 6 (a building destroyed during its sale: the sale stops at the killing hit; no later selling processing completes it)
    if o.health <= 0 {
        return SELLING_DELAY;
    }
    // spec: S-0314 claim 22 (at its next processing the selling depot releases every contact)
    crate::depot::selling_processing(w, id);
    let tick = w.tick;
    let sale = o.econ.sale.clone().unwrap_or(Sale {
        stage: Stage::Ordered,
        packs: crate::econ::deploy::packs_up(w, id),
    });
    let t = TypeRef {
        kind: Kind::Building,
        index: o.type_index,
    };
    match sale.stage {
        Stage::Ordered => {
            // spec: S-0408 claim 5 (a positive upgrade count sells the upgrade at that count's position; a count of 0 sells the building)
            let c = o.econ.building.upgrade_count;
            if c > 0 {
                // spec: S-0408 Edge cases (a count above 3 names no position: the empty-position departure)
                let u = usize::try_from(c - 1)
                    .ok()
                    .and_then(|i| o.econ.building.upgrades.get(i).copied().flatten());
                sell_upgrade(w, id, u);
                return SELLING_DELAY;
            }
            // spec: S-0408 claim 6 and S-0402 claim 28 (a slave-master building's first processing: the conditional resource-directed site rule)
            crate::econ::slaves::first_sale_processing(w, id);
            // spec: S-0408 claim 6 (first processing: docked units told to leave; a bunkered vehicle let out; burning animations end) and S-0402 claim 11 (a refinery's harvesters)
            crate::econ::harvest::refinery_sold(w, id);
            // spec: S-0408 claim 6 and S-0606 claim 13 (a bunkered vehicle is let out on the release track, without walls-down effects or a new behaviour or destination)
            crate::structures::bunker::let_out(w, id);
            w.note("S-0408 claim 6: other docked services and burning animations are other topics' (seams)");
            if sale.packs {
                // spec: S-0415 claim 9 (at the sale's first processing the type's deploy sound at the building's position)
                w.effects.push(Effect::Sound {
                    sound: Sound::Deploy,
                    of: Some(t),
                    house: None,
                    pos: Some(o.pos),
                });
            }
            set_stage(w, id, sale.packs, Stage::Begun);
        }
        Stage::Begun => {
            // spec: S-0402 claim 11 and Changes for other specs (the second processing breaks off every refinery contact; a harvester tethered then still holds back the releases and the build-down)
            let held = crate::econ::harvest::sale_breaks_contact(w, id);
            // spec: S-0408 claim 6 (a later processing while any unit remains docked: the sale waits)
            if held || any_docked(w, id) {
                return SELLING_DELAY;
            }
            let quirk = w.rules().quirks.packup_release_by_guard_spot;
            let packing_type = crate::econ::deploy::undeploys_into(w, o.type_index).is_some();
            let release = if quirk {
                // spec: S-0415 claim 9 (quirks.packup_release_by_guard_spot: a packing building with a guard spot releases nothing)
                !(packing_type && o.guard_spot.is_some())
            } else {
                // spec: S-0415 Edge cases (switch off: the release is skipped exactly when the building packs up)
                !sale.packs
            };
            if release {
                release_occupants(w, id);
            }
            if !sale.packs {
                // spec: S-0408 claim 6 (the sell sound for the owner's player); S-0701 claim 23 (positional at the building)
                w.effects.push(Effect::Sound {
                    sound: Sound::Sell,
                    of: Some(t),
                    house: o.owner,
                    pos: Some(o.pos),
                });
            }
            // spec: S-0415 claim 9 (the pack-up sound at the start of the build-down, for the owner's player, when the type has one; quirks.packup_sound_on_sale for a sold building); S-0701 claim 23 (positional)
            if sale.packs || w.rules().quirks.packup_sound_on_sale {
                w.effects.push(Effect::Sound {
                    sound: Sound::Packup,
                    of: Some(t),
                    house: o.owner,
                    pos: Some(o.pos),
                });
            }
            // spec: S-0309 claim 31 (an idle-body request of an owner change before the build-down starts took effect or is cancelled by that start, an immediate change, S-0108 claim 6: either way the build-down runs its whole range)
            crate::econ::body::leave_idle(w, id);
            // spec: S-0408 claim 6 (build-down begins)
            set_stage(w, id, sale.packs, Stage::BuildDown(tick));
        }
        Stage::BuildDown(_) if o.econ.idle_body.is_some() => {
            // spec: S-0309 claim 31 (with an owner change during the build-down the idle range replaced it: the sale ends at the first selling processing that finds the ready mark set by that range, S-0408 claim 17; an idle range of one stage with a positive delay never sets it, so the sale stalls)
            if o.econ.idle_body.is_some_and(|b| b.sale_report) {
                finish(w, id, sale.packs);
            }
        }
        Stage::BuildDown(since) => {
            // spec: S-0408 claims 6 and 17 (the processing that sees the build-down's completion report pays and removes)
            // spec: S-0408 claim 17 (quirks.single_stage_buildup_never_completes: a range that never reports keeps the sale in build-down, with no refund or removal)
            let Some(e) = builddown_ticks(w, id) else {
                return SELLING_DELAY;
            };
            if tick.wrapping_sub(since) < e {
                return SELLING_DELAY;
            }
            finish(w, id, sale.packs);
        }
    }
    SELLING_DELAY
}

/// The ticks from the start *D* of a building's build-down to its
/// completion report (S-0408 claim 17): E − D; none when the range never
/// reports completion.
fn builddown_ticks(w: &World, id: ObjectId) -> Option<i32> {
    let o = w.obj(id)?;
    let r = buildup_range(w, o.type_index);
    if r.delay <= 0 {
        // spec: S-0408 claim 17 (a zero-delay build-down, one-stage ones included, reports at the next eligible building turn: E = D + 1)
        return Some(1);
    }
    if r.n < MULTI_STAGE {
        // spec: S-0408 claim 17 and Edge cases (quirks.single_stage_buildup_never_completes: a positive-delay range of 1 stage or fewer never reports, the stage-23 early report included; off: the report on the first wrap, E = D + d_b)
        return r.report_after(w.rules().quirks.single_stage_buildup_never_completes);
    }
    let mut stages = r.n - 1;
    if crate::econ::deploy::undeploys_into(w, o.type_index).is_some() && o.guard_spot.is_none() {
        // spec: S-0408 claim 17 and Constants (a type that names a vehicle to pack into, without a guard spot, also reports at stage 23; even when sold as an ordinary building)
        stages = stages.min(EARLY_REPORT_STAGE);
    }
    // spec: S-0408 claim 17 and F4 (E = D + (n − 1) × d_b, on the tick the last stage is reached)
    Some(stages.saturating_mul(r.delay))
}

/// The build-down stage at which a packing-capable type without a guard
/// spot reports completion early (S-0408 claim 17).
// spec: S-0408 Constants (early completion stage 23)
const EARLY_REPORT_STAGE: i32 = 23;

fn set_stage(w: &mut World, id: ObjectId, packs: bool, stage: Stage) {
    if let Some(ob) = w.obj_mut(id) {
        ob.econ.sale = Some(Sale { stage, packs });
    }
}

/// The releases of a sale (S-0408 claims 7 to 9): absorbed passengers,
/// garrison and sale survivors, in that order.
fn release_occupants(w: &mut World, id: ObjectId) {
    // spec: S-0408 claim 8 and S-0605 claim 14 (absorbed passengers first, before the garrison)
    crate::structures::absorb::sold(w, id);
    // spec: S-0408 claim 9 and S-0508 claim 13 (a sold garrisoned building puts out its occupants after the absorbed passengers and before the sale survivors: no hunting, with the inside fallback)
    crate::structures::garrison::sold(w, id);
    // spec: S-0408 claims 7 and 9 (the sale survivors last: C attempts, C of S-0312 F5)
    crate::combat::survivors::sold(w, id);
    w.note("S-0408 claims 7 to 9: the release step put out the absorbed passengers (S-0605), the garrison (S-0508) and the sale survivors (S-0312 F5's count)");
}

/// The end of a sale (S-0408 claim 6; S-0415 claims 9 to 11).
fn finish(w: &mut World, id: ObjectId, packs: bool) {
    let Some(o) = w.obj(id).cloned() else { return };
    let Some(house) = o.owner else {
        crate::objects::remove(w, id);
        return;
    };
    // spec: S-0415 claim 9 (S-0213's conditions are judged again when the build-down finishes)
    if packs && crate::econ::deploy::packs_up(w, id) {
        crate::econ::deploy::pack_up(w, id);
        return;
    }
    let amount = refund(
        w,
        house,
        TypeRef {
            kind: Kind::Building,
            index: o.type_index,
        },
    );
    // spec: S-0408 claim 6 (the refund, then stored resources, and the building is removed)
    credit(w, house, amount);
    // spec: S-0408 claim 10 (stored resources: no skirmish path fills a building's storage; nothing stored)
    // spec: S-0408 claim 10a (a true sale's slave cleanup, as a consequence of removing the master; its exact tick against the refund is open)
    crate::econ::slaves::master_sold(w, id);
    crate::objects::remove(w, id);
}

/// Sells the upgrade at the upgrade count's position of building `id`
/// (S-0408 claim 5); `u` is the type recorded there, none for an empty
/// position (the departure of S-0408 Edge cases).
fn sell_upgrade(w: &mut World, id: ObjectId, u: Option<u32>) {
    let stale = w.rules().quirks.stale_suspended_revert; // spec: S-0215 Edge cases (quirks.stale_suspended_revert)
    let Some(house) = w.obj(id).and_then(|o| o.owner) else {
        return;
    };
    if let Some(u) = u {
        let amount = refund(
            w,
            house,
            TypeRef {
                kind: Kind::Building,
                index: u,
            },
        );
        // spec: S-0408 claim 5 (the refund for the upgrade's type)
        credit(w, house, amount);
        // spec: S-0408 claim 5 (selling an upgrade takes one creation draw, S-0202 claim 10)
        let _ = w.rng.raw();
    } else {
        // spec: S-0408 Edge cases (an empty position: no refund, no creation draw; the count drops by 1 and the building goes to guard)
        w.note("S-0408 Edge cases: an upgrade sale found its position empty (our engine's departure: no refund)");
    }
    let turret = u.is_some_and(|u| {
        w.rules()
            .buildings
            .get(u as usize)
            .is_some_and(|b| b.turret)
    });
    let tick = w.tick;
    if let Some(ob) = w.obj_mut(id) {
        let b = &mut ob.econ.building;
        if let Some(slot) = usize::try_from(b.upgrade_count - 1)
            .ok()
            .and_then(|i| b.upgrades.get_mut(i))
        {
            // spec: S-0408 claim 5 (only the sold position is cleared)
            *slot = None;
        }
        b.upgrade_count = if turret {
            // spec: S-0408 claim 5 (a turret upgrade resets the count to 0, whatever its level or the target's limit)
            0
        } else {
            // spec: S-0408 claim 5 (otherwise c − 1 exactly, not recomputed, not clamped to the type's limit)
            b.upgrade_count - 1
        };
        ob.econ.sale = None;
        // spec: S-0408 claim 5 (the building goes to guard; its health is unchanged)
        ob.beh.force(Behaviour::Guard, tick, stale);
    }
    // spec: S-0408 claim 5 (the house works out its power again, S-0405 claim 7)
    crate::econ::power::make_due(w, house);
    crate::econ::radar::mark_due(w, house);
    if let Some(u) = u {
        // spec: S-0512 claim 2 (d) (a building losing an upgrade: the availability check at once)
        crate::superweapons::charge::upgrade_lost(w, id, u);
    }
}

/// A building type's adjusted cost *C*′ (S-0408 claim 2, F2), which the
/// price of S-0404 claim 2 uses. Owned by S-0408's topic.
pub fn adjusted_cost(w: &World, b: u32) -> i32 {
    use crate::econ::production as p;
    let cost = |t: Option<crate::econ::TypeRef>| t.map_or(0, |t| p::cost_field(w, t));
    // spec: S-0408 F2 (d = trunc((C₀ + C₁) / 2) when the pad deduction applies; a missing entry counts 0)
    let d = if p::pad_term_applies(w, b) {
        (cost(p::pad(w, 0)) + cost(p::pad(w, 1))) / 2
    } else {
        0
    };
    let k = cost(Some(crate::econ::TypeRef {
        kind: crate::rules::Kind::Building,
        index: b,
    }));
    match p::free_unit(w, b) {
        // spec: S-0408 F2 (C = max(0, K − d − F) with a free unit, K − d without one)
        Some(f) => (k - d - cost(Some(f))).max(0),
        None => k - d,
    }
}
