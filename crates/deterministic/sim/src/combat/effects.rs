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
        /// The stage delay the creation's ranged draw gave (S-0109 claim 3),
        /// when the simulation took one; `None` when the type's range holds
        /// one value or the rules do not list the type. Only the drawing
        /// reads it: the simulation keeps no playback clock.
        delay: Option<i32>,
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
        /// The weapon fired (its report sounds, S-0701 claim 21).
        weapon: crate::combat::data::WeaponId,
        /// The fire point.
        from: Pos,
    },
    /// A hit's damage-state result (S-0302 claim 12), for the sound
    /// reactions of S-0701 claims 20 and 28. Not reported for a hit that
    /// claim 8 stops before its health stage, nor for a berserk hit (S-0302
    /// claim 10: it has no reactions).
    Hit {
        /// The object hit.
        id: ObjectId,
        /// Its type (the object may be gone when the effect is read).
        of: crate::econ::TypeRef,
        /// Its owner.
        owner: Option<crate::world::HouseId>,
        /// Where it was.
        pos: Pos,
        /// The result.
        result: crate::combat::damage::HitResult,
        /// The attacker, if any.
        attacker: Option<ObjectId>,
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
    /// A presentation-only trigger action (S-0011 claim 32): text message
    /// (11), play sound (19), play speech (21), radar event at a waypoint
    /// (55) or sound at a waypoint (99). How it is shown or heard is the
    /// presentation's.
    Trigger {
        /// The action code.
        code: i32,
        /// The sound or speech name (action parameter kinds 6 and 7).
        name: String,
        /// The text label (kind 4).
        text: String,
        /// The number slot (the radar event kind for 55).
        number: i32,
        /// The waypoint cell's centre, for 55 and 99.
        pos: Option<Pos>,
    },
    /// A crusher crushed an overlay (S-0210 claim 8): the overlay type's
    /// crush sound plays at the crusher's position.
    OverlayCrushed {
        /// The overlay type, by registration index.
        overlay: u8,
        /// The crusher's position.
        pos: Pos,
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
    /// A produced unit left its factory (S-0701 claims 23 and 28): `of` is
    /// the unit's type, `pos` the factory's position.
    UnitReady,
    /// A queue request refused as it is applied, by the queue limit or the
    /// build limit (S-0404 claim 3, S-0701 claim 24).
    QueueRefused,
    /// A rocket missile's take-off sound at the end of its tilt or raise
    /// (S-0506 claim 15): `of` is the missile's type, `pos` its position.
    TakeOff,
    /// A depot finished repairing a vehicle (S-0314 claim 6, S-0701 claim
    /// 28): the repair-complete announcement and the *unit repaired* radar
    /// event, for the depot's owner at the depot's position.
    UnitRepaired,
    /// The low-funds announcement of a depot that cannot pay (S-0314 claim
    /// 6), for the depot's owner.
    LowFunds,
    /// A crushed object's crush sound (S-0210 claim 4): `of` is the crushed
    /// object's type, `pos` the crusher's position.
    Crush,
    /// A teleporting object's own jump left its old point (S-0207 claim 20
    /// e): `of` is its type, `pos` the old point.
    ChronoOut,
    /// A teleporting object's own jump arrived at its landing point (S-0207
    /// claim 20 e): `of` is its type, `pos` the landing point.
    ChronoIn,
    /// The bridge-repaired announcement (S-0511 claim 17), for the
    /// engineer's owner at the engineer's position.
    BridgeRepaired,
    /// The game-wide bridge repair sound (S-0511 claim 17), at the hut's
    /// position, for every player.
    RepairBridge,
}

/// Creates an animation with no loop delay: its creation draws and its
/// start event's draws, as for one created during loading (S-0202 claims
/// 10 and 15), and a presentation event. Animations are not placed in the update order: where
/// a shot's or a death's animations stand in it is not established (S-0203
/// claim 2, `unmeasured`).
pub fn anim(w: &mut crate::world::World, name: &str, pos: Pos) {
    let delay = crate::load::anim_created_at(w, name, Some((pos.x, pos.y, pos.z)));
    w.effects.push(Effect::Anim {
        name: name.to_owned(),
        pos,
        delay,
    });
}
