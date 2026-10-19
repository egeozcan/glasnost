//! Presentation events of combat for the drawing: what was created where.
//! They are not game state: the simulation never reads them, and they are
//! neither saved nor hashed.

use crate::world::{HouseId, ObjectId, Pos};

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
        /// The animation plays in reverse (S-0517 claim 6: the deploying
        /// presentation of an undeploy); only the drawing reads it.
        reverse: bool,
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
        /// The firer's creation-derived report value *r* (S-0701 claim 21,
        /// S-0202 claim 10), read when it fires.
        report: u16,
        /// The firer's type has the gattling flag: it plays no per-shot
        /// report (S-0609 claim 13).
        gattling: bool,
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
    /// A gas particle hit an object (S-0613 claim 13); the hit itself is
    /// ordinary damage, reported as for any other.
    GasHit {
        /// The object hit.
        id: ObjectId,
        /// The particle's position.
        pos: Pos,
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
    /// A sound of mind control (S-0601 claims 5, 9 and 16). `houses` are
    /// the controller's house and the target's owner before the takeover,
    /// for the take-over sound only the local player of one of them hears.
    Mind {
        /// Which sound.
        sound: crate::mind::MindSound,
        /// The released object's type, whose own release sound is preferred.
        of: Option<crate::econ::TypeRef>,
        /// The houses that hear a take-over sound.
        // spec: S-0601 claim 5 (two houses: the controller's and the target's before)
        houses: [Option<crate::world::HouseId>; 2],
        /// Where.
        pos: Option<Pos>,
    },
    /// A recycling building consumed an entrant (S-0603 claim 7): the
    /// entrant's death response (its owner's player only) and death sound
    /// at `pos`, chosen as S-0701 claim 20's *now dead* case, then the
    /// rules-wide enter-grinder sound at `pos` when `grinding` (the
    /// destination has the grinding flag, claim 13). Not a death: no
    /// announcement, kill or loss goes with it (claim 11).
    Consumed {
        /// The entrant's type.
        of: crate::econ::TypeRef,
        /// The entrant's owner at consumption.
        owner: Option<crate::world::HouseId>,
        /// The entrant's position (its centre).
        pos: Pos,
        /// Whether the building has the grinding flag.
        grinding: bool,
    },
    /// A building's special animation slots changed (S-0606 Terms, walls
    /// effects; S-0108's *special-1* to *special-4*): the slots named in
    /// `slots` start (`start`) or stop, in their damaged variant when
    /// `damaged`. Only the drawing reads it.
    SpecialAnims {
        /// The building.
        building: ObjectId,
        /// The slots, 1 to 4.
        slots: [u8; 2], // spec: S-0606 claim 12 and table S1 (the walls' slots start and stop in pairs)
        /// Started (else stopped).
        start: bool,
        /// The damaged variant.
        damaged: bool,
    },
    /// A vehicle became bunkered (S-0606 claim 7): it leaves any control
    /// group it belonged to and is removed from the selection.
    Bunkered {
        /// The vehicle.
        vehicle: ObjectId,
        /// Its bunker.
        bunker: ObjectId,
    },
    /// A unit was seized by magnetic lift (S-0607 claim 5): it is removed
    /// from the selection.
    Seized {
        /// The seized unit.
        unit: ObjectId,
    },
    /// A vehicle was disabled by the loss of its control centre (S-0405
    /// claims 13 and 20): it is removed from the selection.
    Deactivated {
        /// The disabled vehicle.
        unit: ObjectId,
    },
    /// A unit dropped by magnetic lift is deselected by the stop on
    /// dropping (S-0607 claim 13): it is removed from the selection.
    Dropped {
        /// The dropped unit.
        unit: ObjectId,
    },
    /// A gattling spin update or a gattling-flagged object leaving the map
    /// (S-0609 claim 13), for the spin sound the audio crate keeps.
    Spin {
        /// The object.
        id: ObjectId,
        /// What happened.
        event: crate::gattling::SpinEvent,
        /// Where it was.
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
    /// A cloak event (S-0516 claim 26): selections are pruned on it.
    Cloak {
        /// The object.
        id: ObjectId,
        /// What happened.
        event: crate::cloak::machine::CloakEvent,
    },
    /// A building changed owner (S-0509 claim 12): for the capture and loss
    /// announcements (S-0701 claim 39), the *captured* radar event and the
    /// local player's building placement (claim 15). `announce` is whether
    /// the cause requests an announcement (an engineer capture or a
    /// vehicle-like takeover do; a trigger and the occupation rules do not).
    Captured {
        /// The building.
        building: ObjectId,
        /// Its type.
        of: crate::econ::TypeRef,
        /// The old owner.
        old: Option<crate::world::HouseId>,
        /// The new owner.
        new: Option<crate::world::HouseId>,
        /// Where (the building's position).
        pos: Pos,
        /// Whether the cause requests an announcement.
        announce: bool,
    },
    /// A spy infiltrated a building (S-0510 claim 11): the announcement
    /// lines for the infiltrating and the victim house, and the *infiltrated*
    /// radar event, when the two houses differ.
    Infiltrated {
        /// The building.
        building: ObjectId,
        /// The infiltrating house (the spy's owner).
        infiltrator: Option<crate::world::HouseId>,
        /// The victim house (the building's owner).
        victim: Option<crate::world::HouseId>,
        /// The effect rows that applied, in table order (one row unless
        /// `quirks.infiltration_single_effect` is off).
        rows: Vec<InfiltrationRow>,
        /// Whether the victim has an active spy satellite (row E1 then
        /// makes no radar lines).
        satellite: bool,
        /// Where (the building's top-left cell's position).
        pos: Pos,
    },
    /// A prism support beam is drawn (S-0518 claim 6): from the supporter's
    /// fire point to the master's at recruitment, for `duration` ticks, in
    /// the house's laser colour; drawing only.
    SupportBeam {
        /// The supporter's fire point.
        from: Pos,
        /// The master's fire point at recruitment.
        to: Pos,
        /// How long it lasts, ticks.
        duration: i32,
        /// The supporter's house.
        house: Option<crate::world::HouseId>,
    },
    /// A disc laser event (S-0615 claims 12 to 16): its shot, its
    /// charge-up sound, its hit with the report sound, or its cancellation.
    Disc {
        /// The firer.
        firer: ObjectId,
        /// The weapon fired (its report sounds, S-0701 claim 21).
        weapon: crate::combat::data::WeaponId,
        /// What happened.
        event: DiscEvent,
        /// Where: the firer at the shot and cancellation, the ring centre at
        /// the charge-up, the target point at the hit.
        pos: Pos,
    },
    /// A beam of laser light is drawn (S-0615 Terms, *beam*): drawing only.
    Beam {
        /// One end.
        from: Pos,
        /// The other end.
        to: Pos,
        /// How long it lasts, ticks.
        duration: i32,
        /// It fades to no intensity (the disc laser's final beam); otherwise
        /// to half intensity (its arc beams).
        fade_out: bool,
        /// The inner, outer and spread colours.
        colours: [[i32; 3]; 3], // spec: S-0615 Import mapping (three integer colour channels, S-0003)
        /// With the house-colour flag, the house whose laser colour the
        /// drawing uses for the inner and outer colours
        /// (`crate::disc::house_colours`).
        house: Option<crate::world::HouseId>,
    },
    /// A drain link formed, ended or paid (S-0608 claims 5, 7 and 14).
    /// Presentation and counting only: a payout has no message and no
    /// sound (claim 7).
    Drain {
        /// The drainer.
        drainer: ObjectId,
        /// The drained building.
        building: ObjectId,
        /// What happened.
        event: crate::drain::DrainEvent,
    },
    /// A burning walker was made (S-0615 claim 1, death kind 4), or ended.
    Walker {
        /// The walker.
        id: ObjectId,
        /// Its animation type's name (its start sound, S-0701 claim 22).
        name: String,
        /// Where.
        pos: Pos,
        /// It was made (otherwise it was removed).
        made: bool,
    },
    /// An infantry-making animation (S-0611 claims 8 to 12) was made,
    /// switched type, changed owner, had a placement refused, or ended.
    /// The simulation plays it; the drawing shows `name` at `frame`.
    Maker {
        /// The animation.
        id: ObjectId,
        /// Its current type's name.
        name: String,
        /// Where.
        pos: Pos,
        /// Its owner (its colour).
        owner: Option<HouseId>,
        /// The image frame it shows.
        frame: i32,
        /// What happened.
        event: crate::mutation::MakerEvent,
    },
    /// A superweapon presentation event (S-0512 claim 5, S-0513 claim 5).
    Superweapon {
        /// What happened.
        event: crate::superweapons::SwEvent,
        /// The house it concerns (the launching or record house), if any.
        house: Option<crate::world::HouseId>,
        /// The superweapon type, by list position, if any.
        sw: Option<u32>,
        /// Where, if positional.
        pos: Option<Pos>,
    },
}

/// What a disc laser did (S-0615 claims 12 to 16).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DiscEvent {
    /// The shot made it (no report sound, claim 12).
    Shot,
    /// Its charge-up sound, when the rules data names one (claim 15).
    ChargeUp {
        /// The sound's name.
        sound: Option<String>,
    },
    /// Its final step: the detonation and the report sound (claim 16).
    Hit {
        /// The firer's creation-derived report value *r* (S-0701 claim 21).
        report: u16,
    },
    /// It was cancelled with no damage (claim 14).
    Cancelled,
}

/// The effect rows of S-0510 claim 3 that an infiltration applied.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InfiltrationRow {
    /// E1: the whole-map re-cover of a radar building's owner.
    Radar,
    /// E2: the timed blackout of a power plant's owner.
    Power,
    /// E3: the stolen-tech flag of a tech-lab building.
    Tech,
    /// E4: the reset of a superweapon building's record.
    Superweapon,
    /// E5: the money theft.
    Cash,
    /// E6: the infiltrated-vehicle-factory flag.
    VehicleFactory,
    /// E7: the infiltrated-infantry-factory flag.
    InfantryFactory,
}

/// The sounds of [`Effect::Sound`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sound {
    /// A type's deploy sound (S-0415 claims 6 and 9, S-0517 claim 12).
    Deploy,
    /// A vehicle type's deactivation sound, played at its position when it
    /// becomes disabled (S-0405 claim 20, `DeactivateSound`).
    Deactivate,
    /// A vehicle type's activation sound, played at its position when it
    /// stops being disabled (S-0405 claim 20, `ActivateSound`).
    Activate,
    /// A vehicle type's undeploy sound (S-0517 claim 12).
    Undeploy,
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
    /// The game-wide cloak sound (S-0516 claim 15), at the object.
    Cloak,
    /// The game-wide building-repaired sound (S-0509 claim 5), at the
    /// repaired building's centre.
    BuildingRepaired,
    /// An entry made a building's occupant count 1 (S-0508 claim 6): the
    /// garrisoned announcement and sound for the entering unit's house, the
    /// sound at the entering unit's last position; `of` is the building's type.
    Garrisoned,
    /// A civilian garrison building returned to the civilian house
    /// (S-0508 claim 8): the abandoned sound (not positioned, not
    /// throttled), the *garrison abandoned* radar event at the building's
    /// top-left cell and, when the event is created, the announcement, for
    /// the owner that lost the building (`house`).
    Abandoned,
    /// An infantry unit went into an infantry absorber (S-0605 claim 7):
    /// the rules-wide entry sound at the entrant's position; `of` is the
    /// entrant's type, `house` its owner.
    EnterBioReactor,
    /// A passenger came out of an infantry absorber (S-0605 claim 13): the
    /// rules-wide exit sound at the building; `of` is the building's type.
    LeaveBioReactor,
    /// A vehicle became bunkered (S-0606 claim 7): the rules-wide walls-up
    /// sound at the bunker's position; `of` is the bunker's type.
    BunkerWallsUp,
    /// The walls-down effects of a bunker (S-0606 claims 12 and 14): the
    /// rules-wide walls-down sound at the bunker's position; `of` is the
    /// bunker's type.
    BunkerWallsDown,
}

/// Creates an animation with no loop delay: its creation draws and its
/// start event's draws, as for one created during loading (S-0202 claims
/// 10 and 15), and a presentation event. Animations are not placed in the update order: where
/// a shot's or a death's animations stand in it is not established (S-0203
/// claim 2, `unmeasured`).
pub fn anim(w: &mut crate::world::World, name: &str, pos: Pos) {
    anim_reversible(w, name, pos, false);
}

/// [`anim`], playing in reverse when `reverse` (S-0517 claim 6: the deploying
/// presentation of an undeploy).
pub fn anim_reversible(w: &mut crate::world::World, name: &str, pos: Pos, reverse: bool) {
    let index = w
        .rules()
        .animations
        .iter()
        .position(|a| a.name.eq_ignore_ascii_case(name))
        .and_then(|i| u32::try_from(i).ok());
    if !reverse
        && let Some(i) = index
        && crate::mutation::makes_infantry(w, i)
    {
        let layer = if w.high_bridge(pos.cell()) && pos.z > w.surface_at(pos.x, pos.y) {
            crate::world::Layer::Deck
        } else {
            crate::world::Layer::Ground
        };
        // spec: S-0611 claim 9 (the rules for every infantry-making animation: it is played by the simulation and creates its unit at its end; no owner)
        crate::mutation::start(w, i, pos, layer, None, false);
        return;
    }
    let delay = crate::load::anim_created_at(w, name, Some((pos.x, pos.y, pos.z)));
    w.effects.push(Effect::Anim {
        name: name.to_owned(),
        pos,
        delay,
        reverse,
    });
}
