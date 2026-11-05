//! The own AI and tech buildings (S-0515; PLAN §4.7): which buildings held by
//! a passive house an engineer is worth sending to. The capture itself is the
//! engineer's (S-0509, `special`); what this module adds is the person's
//! taste: a building that pays (an acquisition bonus, periodic income) before
//! one that only serves, and the wish for an engineer when one is in sight.
//!
//! Like the rest of the brain it reads only what a person in the house's
//! seat sees: the type's fields (the sidebar and the game's help show what a
//! building does), the owner of a building on a cell the house has explored.

use glasnost_sim::econ::{setup, shroud};
use glasnost_sim::rules::Kind;
use glasnost_sim::structures::tech;
use glasnost_sim::world::{HouseId, ObjectId, World};

/// A tech building (S-0515 Terms: the needs-engineer flag) held by a passive
/// house, which an engineer may take over.
pub fn passive_tech(w: &World, b: ObjectId) -> bool {
    let Some(o) = w.obj(b) else { return false };
    o.kind == Kind::Building
        && o.on_map
        && o.health > 0
        && o.owner.is_some_and(|h| setup::passive(w, h))
        && w.rules()
            .buildings
            .get(o.type_index as usize)
            .is_some_and(|t| {
                t.raw.needs_engineer.unwrap_or(false) && t.raw.capturable.unwrap_or(false)
            })
}

/// How much a person wants to take the building, lower first: 0 a tech
/// building that pays (S-0515 claims 3 and 5: a cash-producing type), 1 any
/// other tech building (its services), 2 anything else.
pub fn rank(w: &World, b: ObjectId) -> u8 {
    if !passive_tech(w, b) {
        return 2; // nocite: tuning of our own opponent
    }
    if tech::cash_producing(w, b) {
        0
    } else {
        1 // nocite: tuning of our own opponent
    }
}

/// The tech buildings that pay (rank 0), held by a passive house, on cells
/// the house has explored, in object order.
pub fn paying_in_sight(w: &World, house: HouseId) -> Vec<ObjectId> {
    w.objects
        .values()
        .filter(|o| rank(w, o.id) == 0 && shroud::explored(w, house, o.pos.cell()))
        .map(|o| o.id)
        .collect()
}

/// Whether the house sees a tech building held by a passive house on a cell
/// it has explored.
pub fn in_sight(w: &World, house: HouseId) -> bool {
    w.objects
        .values()
        .any(|o| passive_tech(w, o.id) && shroud::explored(w, house, o.pos.cell()))
}
