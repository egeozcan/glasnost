//! Presentation events of combat for the drawing: what was created where.
//! They are not game state: the simulation never reads them, and they are
//! neither saved nor hashed.

use crate::world::{ObjectId, Pos};

/// One presentation event.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Effect {
    /// An animation was created (its creation draws were taken in the
    /// simulation, S-0202 claim 10).
    Anim {
        /// Animation type name.
        name: String,
        /// Where.
        pos: Pos,
    },
    /// A detonation happened (S-0303).
    Detonation {
        /// Where.
        pos: Pos,
        /// Spread radius R, leptons.
        radius: i32,
        /// Damage amount D.
        damage: i32,
    },
    /// A shot left a firer (S-0301).
    Shot {
        /// The firer.
        firer: ObjectId,
        /// The fire point.
        from: Pos,
    },
    /// An object was killed (S-0312).
    Killed {
        /// The object.
        id: ObjectId,
        /// Where.
        pos: Pos,
    },
}

/// Creates an animation: its creation draws (S-0202 claim 10) and a
/// presentation event. Animations are not placed in the update order: where
/// a shot's or a death's animations stand in it is not established (S-0203
/// claim 2, `unmeasured`).
pub fn anim(w: &mut crate::world::World, name: &str, pos: Pos) {
    crate::load::anim_creation_draws(w, name);
    w.effects.push(Effect::Anim {
        name: name.to_owned(),
        pos,
    });
}
