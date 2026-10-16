//! Tech buildings (S-0515): the acquisition bonus, the income countdown with
//! its cycles and payments, and the effects of an ownership change on them.
//! The manned state itself is `econ::power`'s (claim 2, set by
//! `power::owner_changed`); the other services of a tech building are the
//! general rules of the fields its type sets (claim 1).

use crate::econ::power;
use crate::rules::Kind;
use crate::world::{HouseId, ObjectId, World};

/// The type's acquisition bonus *B*, income amount *a* and income delay *D*
/// (Import mapping; an absent key reads 0).
fn cash_fields(w: &World, id: ObjectId) -> Option<(i32, i32, i32)> {
    let o = w.obj(id)?;
    if o.kind != Kind::Building {
        return None;
    }
    let b = w.rules().buildings.get(o.type_index as usize)?;
    // spec: S-0515 Constants (absent-key values of B, a and D are 0)
    Some((
        b.raw.produce_cash_startup.unwrap_or(0),
        b.raw.produce_cash_amount.unwrap_or(0),
        b.raw.produce_cash_delay.unwrap_or(0),
    ))
}

/// Whether a building's type is cash-producing (Terms: a nonzero acquisition
/// bonus or a positive income delay).
pub fn cash_producing(w: &World, id: ObjectId) -> bool {
    // spec: S-0515 Terms (cash-producing type: a nonzero acquisition bonus or a positive income delay)
    cash_fields(w, id).is_some_and(|(b, _, d)| b != 0 || d > 0)
}

/// Restarts the building's income countdown in the present tick (claim 4).
fn restart(w: &mut World, id: ObjectId) {
    let tick = w.tick;
    if let Some(o) = w.obj_mut(id) {
        o.econ.building.income_restart = Some(tick);
    }
}

/// The remaining time of the countdown at the present tick (Terms: the
/// length less the ticks since the restart, never below 0); a countdown
/// created and not yet restarted has run out and reads 0.
pub fn remaining(w: &World, id: ObjectId) -> i32 {
    let Some((_, _, d)) = cash_fields(w, id) else {
        return 0;
    };
    let Some(r) = w.obj(id).and_then(|o| o.econ.building.income_restart) else {
        return 0;
    };
    d.wrapping_sub(w.tick.wrapping_sub(r)).max(0)
}

/// A building is put on the map: the first time restarts its countdown
/// (claim 4 (a)); a countdown created and not yet restarted has run out.
pub fn entered_map(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id) else { return };
    if o.kind != Kind::Building || o.econ.building.income_restart.is_some() {
        return;
    }
    restart(w, id);
}

/// A building changed owner (claims 3, 4 b and 7): the acquisition bonus to
/// the new owner when the old one is a passive house, then the countdown
/// restarts.
pub fn owner_changed(w: &mut World, id: ObjectId, old: Option<HouseId>) {
    let Some(o) = w.obj(id) else { return };
    let new = o.owner;
    // spec: S-0515 Terms (an event that gives the building to the house that already owns it is not an ownership change)
    if old == new {
        return;
    }
    if let Some((bonus, _, _)) = cash_fields(w, id)
        && bonus != 0
        && let Some(n) = new
        && old.is_some_and(|h| crate::econ::setup::passive(w, h))
        && let Some(h) = w.houses.get_mut(usize::from(n.0))
    {
        // spec: S-0515 claim 3 and F3 (the new owner's balance changes by B at once, signed 32-bit wrapping, no floor, no factor of any kind)
        h.credits = h.credits.wrapping_add(bonus);
    }
    // spec: S-0515 claim 4 (b) (every ownership change restarts the countdown with D: the new owner does not inherit the old phase)
    restart(w, id);
}

/// The building's own turn (claims 5 and 6): an income cycle when the
/// countdown's remaining time is exactly 1, restarting it and paying the
/// current owner when the conditions hold.
pub fn turn(w: &mut World, id: ObjectId) {
    let Some((_, amount, delay)) = cash_fields(w, id) else {
        return;
    };
    let Some(o) = w.obj(id) else { return };
    let (owner, on_map) = (o.owner, o.on_map);
    let Some(started) = o.econ.building.income_restart else {
        return;
    };
    // spec: S-0515 claims 4 and 5 (a countdown restarted with D of 0 or less has run out and has no cycle)
    if !on_map || delay <= 0 {
        return;
    }
    let q = &w.rules().quirks;
    // spec: S-0515 F1 and quirks.cash_cycle_one_short (the cycle comes when one tick is left: D − 1 ticks after the restart; off: when the full D ticks have passed)
    let due = started
        .wrapping_add(delay)
        .wrapping_sub(i32::from(q.cash_cycle_one_short));
    let lapse = q.cash_cycle_lapse;
    let now = w.tick;
    // spec: S-0515 claim 5 (before the due tick nothing; after it a countdown that has run out has no cycle, unless quirks.cash_cycle_lapse is off: then the first turn at or after the due tick has the cycle)
    if now < due || (now > due && lapse) {
        return;
    }
    // spec: S-0515 claim 6 (a building being erased has no income cycle in its turns; the countdown keeps running and runs out)
    if power::being_erased(w, id) {
        return;
    }
    // spec: S-0515 claim 5 (the cycle restarts the countdown with D whether or not a payment is made)
    restart(w, id);
    // spec: S-0515 claim 5 (the payment goes to the current owner only if it is not a passive house and the building is in operation)
    let Some(h) = owner else { return };
    if crate::econ::setup::passive(w, h) || !power::in_operation(w, id) {
        return;
    }
    pay(w, h, amount);
}

/// F2: the payment of `a` credits to a house, signed 32-bit wrapping.
fn pay(w: &mut World, h: HouseId, a: i32) {
    let Some(house) = w.houses.get_mut(usize::from(h.0)) else {
        return;
    };
    let y = house.credits;
    house.credits = if a > 0 {
        // spec: S-0515 F2 (a > 0: Y' = wrap(Y + a))
        y.wrapping_add(a)
    } else {
        // spec: S-0515 F2 (a ≤ 0: q = wrap(−a); no stored resources, Y' = wrap(Y − q) when Y ≥ q, otherwise 0)
        let q = a.wrapping_neg();
        if y >= q { y.wrapping_sub(q) } else { 0 }
    };
}
