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
    /// A sound or announcement of the structures topics (S-0406, S-0408,
    /// S-0415). Which sound it is, from the rules data, and whether the
    /// local player hears it (`house`) are the presentation's.
    Sound {
        /// Which sound.
        sound: Sound,
        /// The type whose sound it is (a building or vehicle type).
        of: Option<crate::econ::TypeRef>,
        /// The house it concerns (for sounds only its own player hears).
        house: Option<crate::world::HouseId>,
        /// Where, for a sound played at a position.
        pos: Option<Pos>,
    },
}

/// The sounds of [`Effect::Sound`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sound {
    /// A type's deploy sound (S-0415 claims 6 and 9).
    Deploy,
    /// A building type's pack-up sound, when it has one (S-0415 claim 9).
    Packup,
    /// The sell sound (S-0408 claims 6 and 12).
    Sell,
    /// The refusal announcement of a deploy (S-0415 claim 4).
    CannotDeploy,
    /// The unit-sold announcement (S-0408 claim 11).
    UnitSold,
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
