//! Drain (S-0608): a unit with a draining weapon that stands over an enemy
//! building attaches itself to it. While attached the building is *being
//! drained*, and a building of a resource-destination type pays credits to
//! the drainer's house at a fixed cadence.
//!
//! - The drain attempt (claim 1, [`attempt`]) replaces an ordinary shot of a
//!   draining weapon in `combat::fire::shot`.
//! - The link forms only over the target's foundation (claim 2), one
//!   drainer to one building (claims 3 and 4, the firing check in
//!   `combat::check` and S-0308 table P row 8 in `combat::weapon`); its
//!   forming effects (claim 5) are in [`attempt`].
//! - Payouts (claims 7 to 10) come in the drained building's own turn
//!   ([`building_turn`]).
//! - The link ends by the alliance test and the position check in the
//!   drainer's turn (claims 11 and 12, [`drainer_turn`]), by a killing hit
//!   ([`killed`]) or removal ([`removing`]) of either party (claim 13), and
//!   ending restores what claim 14 lists ([`end`]). Taking a party off the
//!   map for a while without removing it does not touch its links (Open
//!   question 3; request Q-1443).
//!
//! State is world-level ([`DrainWorld`]): every drained building with its
//! drainer, and each drainer's newest partner. The two differ only for a
//! first link stranded by `quirks.drain_relink_strands_previous` (Edge
//! cases). It is saved and hashed.
//!
//! The consumers of the state read [`being_drained`] and [`draining`]:
//! S-0405 (the drain blackout, the building's own firing answer), S-0408
//! claim 3 (no sale), S-0518 claim 2 (no support), S-0215 claim 37 (d) (no
//! ordinary approach), S-0308 table P row 8 and S-0309 claim 25 (h).
//!
//! The drain animation (claim 5) takes its creation draws here
//! (`load::anim_created_at`); its playback over the drainer is the view's,
//! which draws it on every drainer that has a partner, so it is removed when
//! the link ends (claim 14) and an old animation never stays on a relinked
//! drainer (Edge cases: a display defect our engine fixes in both positions
//! of the switch).
//!
//! The position of the drainer's tests and of the payout within the turn is
//! request Q-1440 (interim: the drainer's tests after the warp-only return,
//! before the overload update; the payout after the building's erasure
//! return, before its own power and economy turn).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::combat::effects::Effect;
use crate::rules::Kind;
use crate::world::{ObjectId, Target, World};

/// The position check period in ticks (claim 12, F3).
pub const CHECK_PERIOD: i32 = 16; // spec: S-0608 Constants (position check period 16 ticks)

/// The position check phase (claim 12, F3).
pub const CHECK_PHASE: i32 = 2; // spec: S-0608 Constants (position check phase 2: ticks 2, 18, 34, …)

/// The drain links of the world.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DrainWorld {
    /// Every building being drained, with its drainer (claim 4: at most one
    /// drainer per building).
    pub links: BTreeMap<ObjectId, ObjectId>,
    /// Each draining object's partner: the building of its newest link.
    pub partner: BTreeMap<ObjectId, ObjectId>,
}

/// What happened to a drain link (presentation and counting only).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DrainEvent {
    /// The link formed (claim 5). `anim` is the drain animation's type,
    /// when the rules name a listed one, and `delay` the stage delay its
    /// creation draw gave (S-0109 claim 3).
    Formed {
        /// The drain animation's type name.
        anim: Option<String>,
        /// The creation draw's stage delay, if one was taken.
        delay: Option<i32>,
    },
    /// The link ended (claim 14).
    Ended,
    /// A payout (claim 7): `amount` moved from `payer` to `recipient`.
    Paid {
        /// The amount *t* of F2.
        amount: i32,
        /// The building's house.
        payer: crate::world::HouseId,
        /// The drainer's house.
        recipient: crate::world::HouseId,
    },
}

/// Whether `id` is draining: the drainer of a link (S-0608 Terms). A
/// drainer whose only link is a stranded first link (Edge cases) has no
/// partner and is not draining.
pub fn draining(w: &World, id: ObjectId) -> bool {
    w.drain.partner.contains_key(&id)
}

/// The building `id` drains (its newest link), if any.
pub fn partner(w: &World, id: ObjectId) -> Option<ObjectId> {
    w.drain.partner.get(&id).copied()
}

/// Whether building `id` is being drained (S-0405 Terms; S-0608 Terms).
pub fn being_drained(w: &World, id: ObjectId) -> bool {
    w.drain.links.contains_key(&id)
}

/// The drainer of building `id`, if it is being drained.
pub fn drainer_of(w: &World, id: ObjectId) -> Option<ObjectId> {
    w.drain.links.get(&id).copied()
}

/// Whether object `id` is of a drainable type (S-0608 Terms).
pub fn drainable(w: &World, id: ObjectId) -> bool {
    // spec: S-0608 Import mapping (Drainable, boolean, absent no)
    crate::combat::veterancy::type_combat(w, id).is_some_and(|c| c.drainable)
}

/// Whether building `id` is of a resource-destination type (S-0608 Terms).
pub fn resource_destination(w: &World, id: ObjectId) -> bool {
    let Some(o) = w.obj(id) else { return false };
    if o.kind != Kind::Building {
        return false;
    }
    w.rules()
        .buildings
        .get(o.type_index as usize)
        // spec: S-0608 Import mapping (ResourceDestination, boolean, absent no)
        .is_some_and(|b| b.raw.econ.resource_destination.unwrap_or(false))
}

/// Whether the cell containing `drainer`'s position (its horizontal
/// position; height does not matter) is a cell of building `b`'s
/// foundation (claims 2 and 12).
fn over(w: &World, drainer: ObjectId, b: ObjectId) -> bool {
    let (Some(d), Some(bo)) = (w.obj(drainer), w.obj(b)) else {
        return false;
    };
    if bo.kind != Kind::Building {
        // spec: S-0608 claim 2 (only buildings can be drained: an infantry unit, vehicle or aircraft never gets a link)
        return false;
    }
    // spec: S-0608 claim 2 (the cell that contains the drainer's position is a cell of the target building's foundation; any cell of it will do)
    crate::econ::placement::foundation_cells(w, bo.type_index, bo.pos.cell())
        .contains(&d.pos.cell())
}

/// A drain attempt by `firer` at `target` (claims 1, 2 and 5): what a shot
/// with a draining weapon does instead of an ordinary shot. No projectile,
/// no shot damage, no warhead effect, no ordinary-shot draws; the rearm
/// countdown does not start, the burst does not advance and no ammunition is
/// used (the caller returns before all of them).
pub fn attempt(w: &mut World, firer: ObjectId, target: Target) {
    // spec: S-0608 claim 1 (against a target that is not an object, nothing at all happens and the firer keeps its target)
    let Target::Object(x) = target else {
        return;
    };
    // spec: S-0608 claim 1 (against an object whose type is not drainable, nothing happens and the firer keeps its target)
    if w.obj(x).is_none() || !drainable(w, x) {
        return;
    }
    if let Some(ob) = w.obj_mut(firer) {
        // spec: S-0608 claim 1 (against a drainable object the target is cleared whether or not a link forms, whatever the fire-once flag says; the destination is not touched)
        crate::spawn::clear_own_target(ob);
        ob.combat.burst = 0; // spec: S-0301 claim 4 (a cleared target resets the burst counter)
    }
    // spec: S-0608 claim 2 (otherwise the attempt forms no link and changes nothing but the firer's target)
    if !over(w, firer, x) {
        return;
    }
    // spec: S-0608 State and ordering (a link forms only when the target is not being drained; claim 3 b refuses the attempt earlier)
    if being_drained(w, x) {
        return;
    }
    if let Some(old) = partner(w, firer) {
        if w.rules().quirks.drain_relink_strands_previous {
            // spec: S-0608 Edge cases (quirks.drain_relink_strands_previous on: the first link is not ended; it stays drained, and the drainer's partner becomes the second building)
            w.drain.partner.remove(&firer);
        } else {
            // spec: S-0608 Edge cases (quirks.drain_relink_strands_previous off: forming a new link first ends the old one with all of claim 14's effects)
            end(w, old);
        }
    }
    form(w, firer, x);
}

/// Forms the link of `drainer` to building `b` (claim 5).
fn form(w: &mut World, drainer: ObjectId, b: ObjectId) {
    w.drain.links.insert(b, drainer);
    w.drain.partner.insert(drainer, b);
    // spec: S-0608 claim 5 (the building is being drained from that moment, and its house is due to work out its power, S-0405 claim 7)
    if let Some(h) = w.obj(b).and_then(|o| o.owner) {
        crate::econ::power::make_due(w, h);
    }
    // spec: S-0608 claim 5 and S-0601 claim 10 (a controller building releases all its victims at once, only when a link actually forms)
    crate::mind::drain_link_formed(w, b);
    // spec: S-0608 claim 5 and Import mapping (the drain animation starts attached to the drainer; with no drain animation in the data nothing is shown; S-0109 claim 3 owns its creation draws)
    let rules = w.rules_arc();
    let name = rules
        .combat
        .general
        .drain
        .anim
        .clone()
        .filter(|n| rules.animation(n).is_some());
    let at = w.obj(drainer).map(|o| (o.pos.x, o.pos.y, o.pos.z));
    let delay = name
        .as_deref()
        .and_then(|n| crate::load::anim_created_at(w, n, at));
    w.effects.push(Effect::Drain {
        drainer,
        building: b,
        event: DrainEvent::Formed { anim: name, delay },
    });
}

/// Ends the link of building `b`, for any cause (claim 14).
pub fn end(w: &mut World, b: ObjectId) {
    let Some(drainer) = w.drain.links.remove(&b) else {
        return;
    };
    // spec: S-0608 claim 14 (the drainer is no longer draining: table P row 8 can choose its draining weapon again and claim 3 a no longer refuses the building; the drain animation is removed)
    if partner(w, drainer) == Some(b) {
        w.drain.partner.remove(&drainer);
    }
    // spec: S-0608 claim 14 (the building is no longer being drained, and its house is due to work out its power, S-0405 claim 7)
    if let Some(h) = w.obj(b).and_then(|o| o.owner) {
        crate::econ::power::make_due(w, h);
    }
    w.effects.push(Effect::Drain {
        drainer,
        building: b,
        event: DrainEvent::Ended,
    });
}

/// The drainer's tests in its own turn (claims 11 and 12).
pub fn drainer_turn(w: &mut World, id: ObjectId) {
    let Some(b) = partner(w, id) else { return };
    let Some(bo) = w.obj(b) else {
        // A building that left play without its removal reaching this
        // module: the link cannot outlive it (claim 13).
        end(w, b);
        return;
    };
    let building_house = bo.owner;
    let drainer_house = w.obj(id).and_then(|o| o.owner);
    // spec: S-0608 claim 11 (in every turn of the drainer the link ends if the building's house allies the drainer's house, the same house included; read in that direction only)
    if w.allied(building_house, drainer_house) {
        end(w, b);
        return;
    }
    let vehicle = w.obj(id).is_some_and(|o| o.kind == Kind::Vehicle);
    // spec: S-0608 claim 12 and Edge cases (quirks.drain_position_check_vehicles_only on: only a drainer that is a vehicle performs the check; off: every drainer)
    let checks = vehicle || !w.rules().quirks.drain_position_check_vehicles_only;
    // spec: S-0608 F3 (a check in the turn of tick n with n mod 16 = 2, exact integer arithmetic)
    let due = w.tick.rem_euclid(CHECK_PERIOD) == CHECK_PHASE;
    // spec: S-0608 claim 12 (the link ends if the building whose foundation covers the drainer's cell is not the one it drains, or there is none)
    if checks && due && !over(w, id, b) {
        end(w, b);
    }
}

/// Whether the tick number `n` is a payout tick for period `p` (F1):
/// *n* mod |*P*| = 0, the absolute value taken mathematically. A period of
/// exactly 0 never pays (the spec's proposed departure, pending the lead's
/// choice: Questions for the project lead; the importer warns).
pub fn payout_tick(n: i32, p: i32) -> bool {
    if p == 0 {
        return false;
    }
    // spec: S-0608 F1 (n mod |P| = 0, exact integer arithmetic; |P| mathematical, also for the minimum signed integer)
    i64::from(n).rem_euclid(i64::from(p).abs()) == 0
}

/// The payout amount *t* of F2 for payout amount `a` and the payer's
/// available credits `c`.
pub fn payout_amount(a: i32, c: i32) -> i32 {
    // spec: S-0608 F2 (t = min(A, C))
    a.min(c)
}

/// A drained building's payout in its own turn (claims 7 to 10).
pub fn building_turn(w: &mut World, id: ObjectId) {
    let Some(drainer) = drainer_of(w, id) else {
        return;
    };
    // spec: S-0608 claim 7 (a building that is not of a resource-destination type pays nothing, even when it is drainable)
    if !resource_destination(w, id) {
        return;
    }
    let g = w.rules().combat.general.drain.clone();
    // spec: S-0608 claim 8 and F1 (the payout ticks are fixed by the tick number alone)
    if !payout_tick(w.tick, g.period) {
        return;
    }
    // spec: S-0608 claim 10 (a drained building that is being erased, or that is materialising after a chronoshift, makes no payout in that tick; it keeps its link)
    if crate::econ::power::being_erased(w, id) || crate::teleport::phasing_in(w, id) {
        return;
    }
    // spec: S-0608 claim 7 (the payer is the building's house at that moment and the recipient the drainer's house at that moment)
    let (Some(payer), Some(recipient)) = (
        w.obj(id).and_then(|o| o.owner),
        w.obj(drainer).and_then(|o| o.owner),
    ) else {
        return;
    };
    // spec: S-0608 claim 9 and F2 (t = min(A, C), C the payer's available credits, S-0404 claim 4; no payout is refused, postponed or made up later)
    let c = crate::seams::available_credits(w, payer);
    let t = payout_amount(g.amount, c);
    // spec: S-0608 F2 (the payer's credit balance falls by t and the recipient's rises by t, in the same payout; signed 32-bit wrapping, S-0205 claim 1)
    if let Some(h) = w.houses.get_mut(usize::from(payer.0)) {
        h.credits = h.credits.wrapping_sub(t);
    }
    if let Some(h) = w.houses.get_mut(usize::from(recipient.0)) {
        h.credits = h.credits.wrapping_add(t);
    }
    // spec: S-0608 claim 7 (nothing else happens: no message, no sound, no score, no harvest)
    w.effects.push(Effect::Drain {
        drainer,
        building: id,
        event: DrainEvent::Paid {
            amount: t,
            payer,
            recipient,
        },
    });
}

/// Object `id` receives its killing hit (claim 13), before the death
/// effects that follow it.
pub fn killed(w: &mut World, id: ObjectId) {
    // spec: S-0608 claim 13 and Edge cases (the drainer's killing hit ends its newest link at once; a stranded first link ends only when the drainer is removed)
    if let Some(b) = partner(w, id) {
        end(w, b);
    }
    building_gone(w, id);
}

/// Object `id` is removed from the game (claim 13).
pub fn removing(w: &mut World, id: ObjectId) {
    // spec: S-0608 claim 13 and Edge cases (a removed drainer's links all end, a stranded first link included)
    let mine: Vec<ObjectId> = w
        .drain
        .links
        .iter()
        .filter(|&(_, &d)| d == id)
        .map(|(&b, _)| b)
        .collect();
    for b in mine {
        end(w, b);
    }
    building_gone(w, id);
}

/// The drained building `id` receives its killing hit or is removed: its
/// link ends at once, and when that link was a stranded first link the
/// drainer's newer link ends with it (claim 13, Edge cases).
fn building_gone(w: &mut World, id: ObjectId) {
    let Some(d) = drainer_of(w, id) else { return };
    let stranded = partner(w, d) != Some(id);
    // spec: S-0608 claim 13 (the link that involves a building that receives its killing hit or is removed ends at once)
    end(w, id);
    if stranded && let Some(newer) = partner(w, d) {
        // spec: S-0608 claim 13 and Edge cases (destroying or removing the stranded first building also tears down the drainer's newer link)
        end(w, newer);
    }
}
