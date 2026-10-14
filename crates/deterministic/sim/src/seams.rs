//! Seams where the behaviour and combat rules (S-02xx, S-03xx) call into
//! topics that milestone M5a implements: the harvest cycle and slave
//! miners (S-0402), production payment and credits (S-0404), power
//! (S-0405), selling and building refunds (S-0408), and defeat (S-0414).
//!
//! Each function states the rule it stands for and answers through the
//! topic that owns it.

use crate::world::{HouseId, ObjectId, World};

/// Whether the wake-up rule of S-0402 claim 14 holds for a slave master
/// (S-0215 claim 24, vehicle table row 1; claim 26, row 1).
pub fn slave_wake_rule_holds(w: &mut World, master: ObjectId) -> bool {
    // spec: S-0402 claim 14 (the wake-up rule, crate::econ::harvest::wake_rule)
    crate::econ::harvest::slave_wake_rule_holds(w, master)
}

/// Whether a slave master's slave group is at rest (S-0402 claim 14), for
/// S-0215 claim 26's "wakes its slaves anyway".
pub fn slave_group_at_rest(w: &World, master: ObjectId) -> bool {
    // spec: S-0402 claim 14 (a slave group at rest)
    crate::econ::slaves::at_rest(w, master)
}

/// A slave master wakes its slaves (S-0402 claims 14 and 22; S-0215 claims
/// 24 and 26): its group starts field seeking.
pub fn wake_slaves(w: &mut World, master: ObjectId) {
    // spec: S-0402 claim 22 (waking starts field seeking, not the immediate emergence of the hidden slaves)
    crate::econ::slaves::wake(w, master);
}

/// Whether a unit is a harvesting slave (S-0215 Terms; S-0402 claim 14),
/// for S-0215 claim 14 row 8 and claim 15 row 11: a slave with a master.
pub fn is_harvesting_slave(w: &World, id: ObjectId) -> bool {
    // spec: S-0402 claim 14 (harvesting slaves: the slaves of a slave group)
    crate::econ::slaves::is_slave(w, id)
}

/// Whether a slave master (S-0215 claim 15 row 11): a slave miner, or an
/// object that has a slave group.
pub fn is_slave_master(w: &World, id: ObjectId) -> bool {
    // spec: S-0402 claim 14 and Terms (a slave miner: gatherer, destination and an enslaves field; claim 19: the group)
    crate::econ::harvest::is_slave_miner(w, id) || crate::econ::slaves::is_master(w, id)
}

/// The guard rules of S-0402 that queue *harvest* again for a harvester in
/// *guard* (claim 13 for an owner that is not human-controlled, claim 17
/// for a chrono miner), for S-0215 claim 24's vehicle table row 2. Returns
/// whether such a rule applied (it queues *harvest* itself); the processing
/// then ends with 1 tick. None applies before M5a.
pub fn harvester_guard_rule(w: &mut World, id: ObjectId) -> bool {
    // spec: S-0402 claims 13 and 17 (the guard rules that queue harvest again)
    crate::econ::harvest::guard_rule(w, id)
}

/// Whether a house is short of power (S-0215 Terms), for the SAM power row
/// of S-0215 claim 32: S-0405's low power, as the house last worked out its
/// power (S-0405 claims 4 and 7).
pub fn short_of_power(w: &World, house: HouseId) -> bool {
    // spec: S-0215 Terms (short of power: output below drain) and S-0405 claim 4 (low power: D > 0 and O < D)
    crate::econ::power::low_power(w, house)
}

/// The power row of a SAM building's *aiming* stage (S-0215 claim 32):
/// its type has the needs-power flag, drains power (S-0405's base drain
/// above 0, from a negative power value) and its owner is short of power.
pub fn sam_held_by_low_power(w: &World, building: ObjectId) -> bool {
    let Some(o) = w.obj(building) else {
        return false;
    };
    let Some(t) = w.rules().econ.power.buildings.get(o.type_index as usize) else {
        return false;
    };
    // spec: S-0215 claim 32 and S-0405 Terms (needs-power flag, base drain above 0, owner short of power)
    t.needs_power && t.drain > 0 && o.owner.is_some_and(|h| short_of_power(w, h))
}

/// A house's available credits (glossary; S-0404 claim 4), from the
/// production topic.
pub fn available_credits(w: &World, house: HouseId) -> i32 {
    crate::econ::production::available_credits(w, house)
}

/// A house pays `amount` credits (S-0314 claims 3 and 8; S-0404 claim 4),
/// through the production topic.
pub fn pay(w: &mut World, house: HouseId, amount: i32) {
    crate::econ::production::pay(w, house, amount);
}

/// A building type's adjusted cost (S-0408 claim 2, F2), for the step
/// cost of S-0314 claim 5: `crate::econ::sell::adjusted_cost`.
pub fn building_adjusted_cost(w: &World, building_type: u32) -> Option<i32> {
    // spec: S-0408 claim 2 (the same adjusted cost is the type's cost in S-0314's step cost)
    Some(crate::econ::sell::adjusted_cost(w, building_type))
}

/// A building's refund for its owner (S-0408 claim 1, F1), the *K* of
/// S-0312 F5. `None` for a building without an owner.
pub fn building_refund(w: &World, building: ObjectId) -> Option<i32> {
    let o = w.obj(building)?;
    let house = o.owner?;
    // spec: S-0408 claim 1 and F1 (the refund for the building's owner)
    Some(crate::econ::sell::refund(
        w,
        house,
        crate::econ::TypeRef {
            kind: o.kind,
            index: o.type_index,
        },
    ))
}

/// Whether a house's game is over, the state S-0414's defeat puts it in
/// (S-0312 claim 18's survivor condition).
pub fn game_over(w: &World, house: HouseId) -> bool {
    // spec: S-0414 claim 5 (the defeated state)
    crate::econ::victory::defeated(w, house)
}

/// The centre that a neutral or special house's choice supplies for the
/// destination of a unit that gave up after a failed search (S-0212 claim
/// 23a): the house-selected centre of S-0215 claim 38.
pub fn give_up_centre(w: &mut World, id: ObjectId) -> Option<glasnost_map::CellXY> {
    let h = w.obj(id)?.owner?;
    // spec: S-0215 claim 38 and F11 (the central choice, with its ranged and raw draws, for infantry and vehicles whatever their weapons)
    crate::base::selected_centre(w, h)
}
