//! Spies and infiltration (S-0510). Placeholder until the spec is built:
//! a spy is refused like any other entrant.

use crate::world::{ObjectId, World};

/// A spy type (S-0510 Terms, `Agent`).
pub fn is_spy(w: &World, id: ObjectId) -> bool {
    w.obj(id)
        .filter(|o| o.kind == crate::rules::Kind::Infantry)
        .and_then(|o| w.rules().unit(o.kind, o.type_index))
        // spec: S-0510 Import mapping (Agent absent: no)
        .is_some_and(|u| u.raw.agent.unwrap_or(false))
}

/// A spy's entry after the building's entry raise.
pub fn infiltrate(w: &mut World, spy: ObjectId, b: ObjectId) -> bool {
    crate::structures::entry::refuse(w, spy, b);
    false
}
