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
    let player = w
        .houses
        .get(usize::from(house.0))
        .is_some_and(|h| h.role == HouseRole::Player);
    if !player {
        // spec: S-0408 F1 and claim 1 (the neutral and special houses receive P)
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
fn credit(w: &mut World, house: HouseId, amount: i32) {
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
        && !matches!(
            o.beh.current,
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
    if o.beh.current == Some(Behaviour::Selling) {
        // spec: S-0408 claim 4 (a second sell order changes nothing)
        return;
    }
    // spec: S-0213 claim 12 (a building whose type undeploys into a vehicle packs up under its conditions)
    let packs = crate::econ::deploy::packs_up(w, id);
    let tick = w.tick;
    if let Some(ob) = w.obj_mut(id) {
        // spec: S-0408 claim 4 (the building's behaviour becomes selling)
        ob.beh.force(Behaviour::Selling, tick, stale);
        ob.econ.sale = Some(Sale {
            stage: Stage::Ordered,
            packs,
        });
    }
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
    w.note("S-0408 claim 11: the passengers of a sold vehicle are an open question; none are dealt with");
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
    if !o.econ.building.repairing || o.health <= 0 {
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

// ---------------------------------------------------------------------------
// The course of a sale (claims 5 to 10, 16).

/// The selling behaviour's processing delay (S-0408 claim 16).
// spec: S-0408 Constants (selling processing delay: 1 tick, no draw)
pub const SELLING_DELAY: i32 = 1;

/// The ticks from the start of a build-up (or build-down) to its end
/// (S-0108 claim 5, F1): (*n* − 1) × *d_b*, never below 0.
pub fn buildup_ticks(w: &World, t: u32) -> i32 {
    let r = w.rules();
    let mode = r.mode;
    let Some(b) = r.buildings.get(t as usize) else {
        return 0;
    };
    if !b.raw.econ.buildup.unwrap_or(false) {
        // spec: S-0108 claim 5 (no build-up sprite: range a = 0, c = 1, d = 0, finished at once)
        return 0;
    }
    // spec: S-0108 claim 5 (n = ⌊N_b / 2⌋; unknown frame counts read as n = 0)
    let n = b.raw.econ.buildup_frames.unwrap_or(0).div_euclid(2);
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
    // spec: S-0108 claim 5 (finished (n − 1) × d_b ticks after the state began)
    (n - 1).saturating_mul(db).max(0)
}

/// Whether any unit is docked with a building (S-0408 Terms). Seam: the
/// services that make such links (docking at refineries, repair depots,
/// pads) are other topics'; none is linked yet.
fn any_docked(w: &World, id: ObjectId) -> bool {
    let _ = (w, id);
    false
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
            if let Some(&u) = o.econ.building.upgrades.last() {
                sell_upgrade(w, id, u);
                return SELLING_DELAY;
            }
            // spec: S-0408 claim 6 (first processing: docked units told to leave; a bunkered vehicle let out; burning animations end)
            w.note("S-0408 claim 6: docked units, bunkered vehicles and burning animations are other topics' (seams)");
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
            // spec: S-0408 claim 6 (a later processing while any unit remains docked: the sale waits)
            if any_docked(w, id) {
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
                // spec: S-0408 claim 6 (the sell sound for the owner's player)
                w.effects.push(Effect::Sound {
                    sound: Sound::Sell,
                    of: Some(t),
                    house: o.owner,
                    pos: None,
                });
            }
            // spec: S-0415 claim 9 (the pack-up sound at the start of the build-down, for the owner's player, when the type has one; quirks.packup_sound_on_sale for a sold building)
            if sale.packs || w.rules().quirks.packup_sound_on_sale {
                w.effects.push(Effect::Sound {
                    sound: Sound::Packup,
                    of: Some(t),
                    house: o.owner,
                    pos: None,
                });
            }
            // spec: S-0408 claim 6 (build-down begins)
            set_stage(w, id, sale.packs, Stage::BuildDown(tick));
        }
        Stage::BuildDown(since) => {
            // spec: S-0408 claim 6 (the first processing after the build-down has finished; S-0108 claim 5, request Q-0130)
            if tick.wrapping_sub(since) < buildup_ticks(w, o.type_index) {
                return SELLING_DELAY;
            }
            finish(w, id, sale.packs);
        }
    }
    SELLING_DELAY
}

fn set_stage(w: &mut World, id: ObjectId, packs: bool, stage: Stage) {
    if let Some(ob) = w.obj_mut(id) {
        ob.econ.sale = Some(Sale { stage, packs });
    }
}

/// The releases of a sale (S-0408 claims 7 to 9): absorbed passengers,
/// garrison and sale survivors, in that order.
fn release_occupants(w: &mut World, id: ObjectId) {
    let _ = id;
    // Absorbed passengers (claim 8) and garrisons (claim 9) need the transport and garrison topics; none are held yet.
    // spec: S-0408 claim 7 (C attempts, C of S-0312 F5; an owner without a side gives C = 0, as for a destroyed building)
    w.note("S-0408 claims 7 to 9: sale survivors need sides (S-0312 F5 gives C = 0 without one); passengers and garrisons are other topics'");
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
    crate::objects::remove(w, id);
}

/// Sells the top upgrade of building `id` (S-0408 claim 5).
fn sell_upgrade(w: &mut World, id: ObjectId, u: u32) {
    let stale = w.rules().quirks.stale_suspended_revert; // spec: S-0215 Edge cases (quirks.stale_suspended_revert)
    let Some(house) = w.obj(id).and_then(|o| o.owner) else {
        return;
    };
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
    let tick = w.tick;
    if let Some(ob) = w.obj_mut(id) {
        // spec: S-0408 claim 5 (the most recently added upgrade is removed)
        ob.econ.building.upgrades.pop();
        // The upgrade count after a sale is not stated (request Q-0131): it drops by 1, not below 0.
        ob.econ.building.upgrade_count = (ob.econ.building.upgrade_count - 1).max(0);
        ob.econ.sale = None;
        // spec: S-0408 claim 5 (the building goes to guard, otherwise unchanged)
        ob.beh.force(Behaviour::Guard, tick, stale);
    }
    // spec: S-0408 claim 5 (the house works out its power again, S-0405 claim 7)
    crate::econ::power::make_due(w, house);
    crate::econ::radar::mark_due(w, house);
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
