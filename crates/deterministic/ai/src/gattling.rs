//! The own AI and gattling weapons (S-0609; PLAN §4.7).
//!
//! A gattling unit or building gets stronger the longer it keeps firing:
//! every tick it can fire, waits between shots or turns to its target
//! spins it up, and every tick it moves, has no target or cannot attack
//! spins it down (S-0609 claims 2, 3, 5 and 7). A person who knows this:
//!
//! - **keeps a firing gattling on its target.** A gattling unit whose
//!   target it is firing at (its firing check gives one of the spin-up
//!   answers of claim 5) is not given a new attack target by any part of
//!   the brain: another target could make it drive (*moving*, a spin-down)
//!   and its stage would fall. [`engaged`] reads this; the brain drops such
//!   attack orders (`brain::Brain::think`). Orders a person still gives it
//!   (repair, ferry, a wave once it is idle again) are left alone, and a
//!   new order never resets the spin by itself (claim 11);
//! - **builds gattling defences.** A house that can build an armed
//!   building type with the gattling flag builds [`DEFENCES_WANTED`] of
//!   them in its base, the cheapest first, once it keeps its harvesters and
//!   holds [`RESERVE`] times the price ([`defence_type`]; the brain places
//!   it by its ordinary placement). A building spins up while it fires and
//!   stays in place, which suits a base defence;
//! - **builds gattling units** like any other army type: the production
//!   round-robin of `scripted` already builds every army type the house can
//!   build, so no change is needed there, and the waves use them like any
//!   army unit.
//!
//! It reads only the house's own objects and the rules a person may read,
//! and draws no random value. The numbers are tuning of our own opponent,
//! not game behaviour.

use glasnost_sim::combat::check::{self, Answer};
use glasnost_sim::combat::weapon;
use glasnost_sim::econ::{TypeRef, production, tech};
use glasnost_sim::gattling;
use glasnost_sim::rules::Kind;
use glasnost_sim::world::{HouseId, ObjectId, Target, World};

use crate::scripted::{Script, all_types};

/// How many gattling defence buildings a house wants.
pub const DEFENCES_WANTED: usize = 2; // nocite: tuning of our own opponent

/// The credits kept in hand, as a multiple of the building's price, before
/// a gattling defence is ordered.
const RESERVE: i32 = 2; // nocite: tuning of our own opponent

/// Whether unit `id` is a gattling unit firing at an enemy object: its type
/// has the gattling flag, it targets an object of a house it is not allied
/// to, and the firing check for the weapon it would use, range included,
/// gives an answer that spins it up (S-0609 claim 5).
pub fn engaged(w: &World, id: ObjectId) -> bool {
    if !gattling::flagged(w, id) {
        return false;
    }
    let Some(o) = w.obj(id) else { return false };
    let Some(t @ Target::Object(e)) = o.target else {
        return false;
    };
    let enemy = w
        .obj(e)
        .is_some_and(|x| x.owner.is_some() && o.owner.is_some() && !w.allied(o.owner, x.owner));
    if !enemy {
        return false;
    }
    let (slot, wpn) = weapon::would_use(w, id, Some(t));
    wpn.is_some()
        && matches!(
            check::check(w, id, Some(t), slot, true),
            Answer::CanFire | Answer::Rearming | Answer::NotFacing | Answer::StillTurning
        )
}

/// Whether building type `t` is a gattling defence: it has the gattling
/// flag and a weapon in slot 0.
pub fn defence(w: &World, t: TypeRef) -> bool {
    t.kind == Kind::Building
        && w.rules()
            .buildings
            .get(t.index as usize)
            .is_some_and(|b| b.combat.gattling && b.combat.slots[0].is_some())
}

/// Whether type `t` has the gattling flag (S-0609 Terms).
pub fn gattling_type(w: &World, t: TypeRef) -> bool {
    let r = w.rules();
    let tc = match t.kind {
        Kind::Building => r.buildings.get(t.index as usize).map(|b| &b.combat),
        Kind::Vehicle => r.vehicles.get(t.index as usize).map(|u| &u.combat),
        Kind::Aircraft => r.aircraft.get(t.index as usize).map(|u| &u.combat),
        _ => r.infantry.get(t.index as usize).map(|u| &u.combat),
    };
    tc.is_some_and(|c| c.gattling)
}

/// How many gattling defence buildings the house has or has queued.
fn defences(w: &World, house: HouseId) -> usize {
    let on_map = w
        .objects
        .values()
        .filter(|o| o.owner == Some(house) && defence(w, tech::type_of(o)))
        .count();
    let queued: usize = all_types(w)
        .into_iter()
        .filter(|&t| defence(w, t))
        .map(|t| {
            production::line_of(w, house, t).map_or(0, |l| {
                usize::from(l.item.as_ref().is_some_and(|i| i.t == t))
                    + l.queue.iter().filter(|&&q| q == t).count()
            })
        })
        .sum();
    on_map + queued
}

/// The cheapest gattling defence type the house can build and place now,
/// while it has fewer than [`DEFENCES_WANTED`] and can pay [`RESERVE`]
/// times the price.
pub(crate) fn defence_type(script: &Script, w: &World) -> Option<TypeRef> {
    let house = script.house;
    if defences(w, house) >= DEFENCES_WANTED {
        return None;
    }
    let credits = glasnost_sim::seams::available_credits(w, house);
    all_types(w)
        .into_iter()
        .filter(|&t| defence(w, t))
        .filter(|&t| script.startable(w, t))
        .filter(|&t| !script.busy(w, t))
        .filter(|&t| credits >= production::cost_field(w, t).saturating_mul(RESERVE))
        .filter(|&t| script.place_cell(w, t).is_some())
        .min_by_key(|&t| (production::cost_field(w, t), t.index))
}
