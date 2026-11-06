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
        /// The depth adjustment its creator gives in place of its type's
        /// (S-0102 claim 13): −15 for a rocket detonation's explosion
        /// (S-0506 claim 19); `None` keeps the type's.
        z_adjust: Option<i32>,
        /// The animation in play the simulation plays (`crate::anims`),
        /// whose stage the drawing shows (S-0109 claim 28); `None` for one
        /// a specialised model plays, or a type the rules do not list.
        id: Option<ObjectId>,
    },
    /// A shot animation was created (S-0301 claims 20 to 22; its creation
    /// draws were taken in the firer's turn, S-0109 claim 29).
    ShotAnim {
        /// Animation type name.
        name: String,
        /// Where it was created: the shot's launch point.
        pos: Pos,
        /// The stage delay the creation's ranged draw gave (S-0109 claim 3),
        /// as for [`Effect::Anim`].
        delay: Option<i32>,
        /// The firer it is attached to and its displacement from the
        /// firer's position at creation, height included (S-0109 claim
        /// 22): it follows the firer in the ground layer and disappears
        /// with it. `None` for a building's, which stays where it was
        /// created in its type's layer.
        attached: Option<(ObjectId, Pos)>,
        /// The depth adjustment that replaces its type's (S-0301 F8), for a
        /// building's.
        z_adjust: Option<i32>,
        /// The animation in play the simulation plays, as for
        /// [`Effect::Anim`].
        id: Option<ObjectId>,
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
    /// A detonation of a warhead with the rocking flag rocks bodies (S-0303
    /// claim 20): its rocking strength exceeds 0.3 and it was not
    /// nullified. The drawing keeps the rocking; the game reads none of it.
    Rock {
        /// The damage amount *D*, at least 31; the strength is min(*D* ×
        /// 0.01, 4.0).
        damage: i32,
        /// The objects in the detonation's cell layer of the 7 × 7 cells
        /// around the centre cell, row by row, each with the point it is
        /// pushed away from: the source object's position for the centre
        /// cell when there is one, else the detonation point.
        pushed: Vec<(ObjectId, Pos)>,
    },
    /// A drone parasite's delivery rocks its host, which is not infantry
    /// (S-0614 claim 9, F2): `side` is the delivery's draw, 0 or 1.
    DroneRock {
        /// The host.
        host: ObjectId,
        /// The draw R(0, 1) that chose the side.
        side: i32,
    },
    /// A damaging overload pulse sets its surviving controller's sideways
    /// rocking rate (S-0601 claim 16, F4): `level` is the pulse's level
    /// *i* (at least 1) and `draw` its rocking draw *r* in [0, 100].
    OverloadRock {
        /// The controller.
        controller: ObjectId,
        /// The pulse's level *i*.
        level: i32,
        /// The draw *r* that chose the sign.
        draw: i32,
    },
    /// An object that is not infantry started to crash (S-0312 claim 16):
    /// its three tumbling draws, which set its rocking rates (S-0113 claim
    /// 16 and F5). Presentation only: the view keeps the angles (a seam of
    /// S-0113 claim 19, as for the other rocking reports).
    CrashTumble {
        /// The crashing object.
        id: ObjectId,
        /// The first draw, a unit-interval value *u*₁.
        u1: glasnost_num::F64,
        /// The second draw *b*, 0 or 1 (the roll's sense).
        sense: i32,
        /// The third draw, a unit-interval value *u*₃.
        u3: glasnost_num::F64,
    },
    /// A grappling parasite's push starts (S-0614 claim 10): `way` is the
    /// push's draw, 0 or 1.
    GrapplePush {
        /// The host.
        host: ObjectId,
        /// The draw R(0, 1) that chose the way.
        way: i32,
    },
    /// A shot that reached the choice of a shot animation started its
    /// vehicle's target line (S-0301 claim 24 a): the vehicle's type has the
    /// target-laser flag. Presentation only: the drawing keeps the line's
    /// timer for the local player who owned the vehicle at the shot.
    TargetLine {
        /// The vehicle.
        firer: ObjectId,
        /// Its owner at the shot (claim 24 a: the owner test is made then).
        house: Option<crate::world::HouseId>,
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
        /// The shot's target (read by the arena's M-0021 recorder, which
        /// judges each shot against its target).
        target: crate::world::Target,
    },
    /// A hit's damage-state result (S-0302 claim 12), for the sound
    /// reactions of S-0701 claims 20 and 28. Not reported for a hit that
    /// claim 8 stops before its health stage, nor for a berserk hit (S-0302
    /// r43 claim 10, Q-2626: it plays no damage sound, and its harvester
    /// announcement is `Sound::HarvesterAttacked`).
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
        /// The hit's warhead, if any (read by the arena's M-0021 recorder,
        /// which tells a shot's hits from its fragments' by warhead).
        warhead: Option<crate::combat::data::WarheadId>,
        /// The object's health before the hit and after it.
        health: (i32, i32),
    },
    /// A defeat or surrender sweep's hit on a leftover (S-0203 claim 13
    /// (c)): no attacker, no health taken, no destruction registered. No
    /// presentation reads it (no spec gives it a sound); the arena's M-0021
    /// recorder does.
    LeftoverHit {
        /// The leftover.
        id: ObjectId,
        /// Its owner.
        owner: Option<crate::world::HouseId>,
        /// Its last position.
        pos: Pos,
        /// The hit's warhead.
        warhead: Option<crate::combat::data::WarheadId>,
        /// Its health, before the hit and after it (the same).
        health: i32,
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
    /// A repair depot's *idle* or *active-1* slot starts or stops (S-0108
    /// claim 20), in its damaged variant when `damaged` (never the occupied
    /// one). Only the drawing reads it.
    DepotSlot {
        /// The depot.
        building: ObjectId,
        /// *active-1* (else *idle*).
        active1: bool,
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
    /// A deploy or a pack-up put its new object on the map in place of the
    /// old one (S-0415 claims 6 and 10): the new object takes over the old
    /// one's control group and selection, which are the client's. Reported
    /// only on success; a refused deploy and a failed pack-up (claim 11)
    /// report nothing. Presentation only, like every effect.
    Converted {
        /// The deploying vehicle, or the packing building.
        old: ObjectId,
        /// The building put on the map, or the vehicle that appeared.
        new: ObjectId,
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
        /// Whether the change comes after the turns of the building's slot
        /// animations in its tick (S-0108 claim 26; Q-2186).
        after_slots: bool,
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
    /// A shot that is not a disc laser's was fired, with the points the
    /// drawing's line effects need: a laser weapon's beam (S-0518 claims 18
    /// and 20) and a radiation-beam weapon's beam flash (S-0514 claims 23
    /// and 24). The drawing decides from the weapon's flags whether either
    /// is drawn.
    Line {
        /// The firer.
        firer: ObjectId,
        /// The weapon fired.
        weapon: crate::combat::data::WeaponId,
        /// The fire point of the slot that fired.
        from: Pos,
        /// The firer's fire point for its primary weapon slot.
        primary: Pos,
        /// The target's target point, or the target cell's centre at its
        /// height.
        to: Pos,
        /// A building firer's position (S-0518 claim 20, building source).
        building: Option<Pos>,
        /// For a shot of a prism master, its support count *N* at the shot
        /// (S-0518 claim 18).
        prism: Option<i32>,
    },
    /// A shrapnel fragment of a laser fragment weapon was launched while
    /// the projectile still had its attacker (S-0318 claim 8): a beam from
    /// the burst point to the fragment's target with the fragment weapon's
    /// laser look, thickness 1 and no building-source adjustment.
    FragmentLine {
        /// The projectile's attacker, whose house gives a house-colour
        /// beam its inner colour.
        attacker: ObjectId,
        /// The fragment weapon.
        weapon: crate::combat::data::WeaponId,
        /// The burst point.
        from: Pos,
        /// The fragment target's target point, or the target cell's centre
        /// at its height.
        to: Pos,
    },
    /// An electric-bolt shot made its bolt (S-0316 claims 2 to 4): the
    /// drawing makes the bolt, with its own draws from the unsynchronised
    /// stream (claim 13). Its spark system is made in play
    /// (`crate::emitters::electric_bolt_sparks`).
    Bolt {
        /// The firer.
        firer: ObjectId,
        /// The weapon fired (its alternate-colour flag, claim 10).
        weapon: crate::combat::data::WeaponId,
        /// The weapon slot that fired (claims 3 and 4).
        slot: usize,
        /// The shot's burst counter *k* (claim 3: the side of the shot's own
        /// projectile, also in an attached bolt's later frames).
        burst: i32,
        /// The bolt start: the fire point of the slot that fired.
        from: Pos,
        /// The bolt end, fixed (Terms: S-0518 claim 20's target point).
        to: Pos,
        /// A building firer's position (F4: S-0518 claim 20's
        /// building-source adjustment).
        building: Option<Pos>,
        /// The firer was a vehicle on the map at the shot (claim 4: only
        /// such a bolt can be attached).
        vehicle_on_map: bool,
    },
    /// A magnetic wave was made (S-0607 claims 21 and 25), with the shot's
    /// firing and impact points it was made from.
    Wave {
        /// The wave (a key of `World::waves`).
        id: ObjectId,
        /// The shot's firing point.
        from: Pos,
        /// The shot's impact point.
        to: Pos,
    },
    /// A railgun system emitted its line (S-0616 claims 17 and 31): the
    /// view draws the type's laser beam, when the type has one. Drawing
    /// only.
    RailLine {
        /// The system type, by registration index.
        system_type: u32,
        /// The system's position.
        from: Pos,
        /// The system's target point.
        to: Pos,
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
    /// A psychic domination event (S-0612): started, swept, an object
    /// taken over, the final phase, over, a blast ring removed.
    Domination {
        /// What happened.
        event: crate::domination::DomEvent,
        /// The dominating house, if any.
        house: Option<HouseId>,
        /// Where (the target cell's centre; a taken object's position).
        pos: Pos,
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
    /// A moment of S-0102 claim 4 at which the drawing evaluates an
    /// object's render layer, in the order the moments happened. `check`
    /// `None`: a re-file (the object is put at the end of the list of the
    /// layer it evaluates to after the tick). `Some((before, after))`: a
    /// check, the heights just before and just after the object's own step;
    /// the object is re-filed only when the layers they give differ.
    LayerMoment {
        /// The object.
        object: ObjectId,
        /// The heights of a check.
        check: Option<(LayerHeights, LayerHeights)>,
    },
    /// A debris piece's start sound (S-0317 claim 14): played at the
    /// creation point when the piece is made; it then moves with the piece
    /// (`World::pieces`) in each of its turns.
    PieceSound {
        /// The piece.
        id: ObjectId,
        /// The sound's name.
        sound: String,
        /// The creation point.
        pos: Pos,
    },
    /// A route order of a house was applied (S-0213 claim 16b): the
    /// messages and the route-point sound for the house's own player.
    Route {
        /// The house whose order it was.
        house: HouseId,
        /// What it did.
        note: crate::routes::RouteNote,
    },
}

/// The two heights an object's render layer is evaluated from (S-0102
/// *State and ordering*): its height above the surface *r* and its
/// bridge-adjusted height *r*′.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LayerHeights {
    /// *r*, leptons.
    pub r: i32,
    /// *r*′, leptons.
    pub adjusted: i32,
}

/// A re-file moment of S-0102 claim 4 for `object`.
pub fn layer_refile(w: &mut crate::world::World, object: ObjectId) {
    // spec: S-0501 claim 48 (a) and (c) (an aircraft that a factory release left above its pad stays filed in the ground render layer until a re-file: a landing or a crash, S-0102 claim 4)
    if let Some(a) = w.obj_mut(object).and_then(|o| o.air.as_mut()) {
        a.ground_filed = false;
        // spec: S-0306 r72 claim 17 (an aircraft whose type cannot land keeps the ground list while it flies, until a crash, S-0102 claim 4: its next re-file)
        a.kept_filed = false;
    }
    w.effects.push(Effect::LayerMoment {
        object,
        check: None,
    });
}

/// A check moment of S-0102 claim 4 for `object`: `before` its step, and
/// its heights now.
pub fn layer_check(w: &mut crate::world::World, object: ObjectId, before: LayerHeights) {
    if w.obj(object).is_none() {
        return;
    }
    let after = crate::jump::layer_heights(w, object);
    w.effects.push(Effect::LayerMoment {
        object,
        check: Some((before, after)),
    });
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
    /// A berserk hit on a unit (S-0302 r43 claim 10): the announcement that a
    /// harvester is under attack (S-0701 claim 28), made when `of` is a
    /// harvester type of the local player (`house`), at `pos`. No other
    /// sound, announcement or radar event (r43, Q-2626).
    HarvesterAttacked,
}

/// Creates an animation with no loop delay: its creation draws and its
/// start event's draws, as for one created during loading (S-0202 claims
/// 10 and 15), then it is appended to the update order and plays there
/// (S-0109 claim 29, `crate::anims`); and a presentation event.
pub fn anim(w: &mut crate::world::World, name: &str, pos: Pos) {
    anim_reversible(w, name, pos, false);
}

/// [`anim`], playing in reverse when `reverse` (S-0517 claim 6: the deploying
/// presentation of an undeploy).
pub fn anim_reversible(w: &mut crate::world::World, name: &str, pos: Pos, reverse: bool) {
    anim_with(w, name, pos, reverse, None);
}

/// [`anim`] with the depth adjustment its creator gives in place of its
/// type's (S-0102 claim 13).
pub fn anim_adjusted(w: &mut crate::world::World, name: &str, pos: Pos, z_adjust: i32) {
    anim_with(w, name, pos, false, Some(z_adjust));
}

fn anim_with(
    w: &mut crate::world::World,
    name: &str,
    pos: Pos,
    reverse: bool,
    z_adjust: Option<i32>,
) {
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
    anim_spawned(
        w,
        name,
        pos,
        reverse,
        z_adjust,
        crate::anims::Spawn::default(),
    );
}

/// A projectile's trailer animation at `pos` (S-0109 claim 29): its
/// creator gives it an initial loop delay of 1 update, repeat factor 1 and
/// no reverse request, so its start event comes at the end of that delay.
pub fn trailer(w: &mut crate::world::World, name: &str, pos: Pos) {
    let spawn = crate::anims::Spawn {
        creation: crate::anim_clock::Creation {
            loop_delay: TRAILER_LOOP_DELAY,
            repeat: 1,
            reverse: false,
        },
        ..Default::default()
    };
    anim_spawned(w, name, pos, false, None, spawn);
}

/// The initial loop delay a projectile gives its trailer, in updates.
const TRAILER_LOOP_DELAY: i32 = 1; // spec: S-0109 claim 29 (a projectile trailer: an initial loop delay of 1 update)

/// Creates an ordinary animation in play with `spawn` (S-0109 claim 29)
/// and its presentation event.
fn anim_spawned(
    w: &mut crate::world::World,
    name: &str,
    pos: Pos,
    reverse: bool,
    z_adjust: Option<i32>,
    mut spawn: crate::anims::Spawn,
) {
    // spec: S-0109 claim 29 and Terms (a reverse request: the deploying presentation of an undeploy, S-0517 claim 7)
    spawn.creation.reverse |= reverse;
    let made = crate::anims::spawn(w, name, Some(pos), spawn);
    w.effects.push(Effect::Anim {
        name: name.to_owned(),
        pos,
        delay: made.delay,
        reverse,
        z_adjust,
        id: made.id,
    });
}

/// Whether a shot of weapon `wr` that launched a projectile makes a bolt
/// (S-0316 claim 2): the electric-bolt flag without the laser flag, which
/// wins over it. The caller checks that a projectile was put on the map
/// (claims 1 and 2: a firing that launches none, and a void shot, make no
/// bolt).
pub fn makes_bolt(wr: &crate::combat::data::WeaponRules) -> bool {
    // spec: S-0316 claim 2 (the laser flag wins over the electric-bolt flag: a laser and no bolt, no bolt sparks)
    wr.electric_bolt && !wr.laser
}

/// An electric-bolt shot's bolt and its spark system (S-0316 claims 2 to
/// 4 and 6), made in the tick of the shot, after the shot's report (claim
/// 13, State and ordering): reports [`Effect::Bolt`] from the fire point
/// `from` of `slot` with the shot's burst counter `burst` to the bolt end,
/// then makes one system of the game-wide spark type at the bolt end.
pub fn bolt(
    w: &mut crate::world::World,
    firer: ObjectId,
    weapon: crate::combat::data::WeaponId,
    slot: usize,
    burst: i32,
    from: Pos,
    target: crate::world::Target,
) {
    let Some(o) = w.obj(firer) else {
        return;
    };
    // spec: S-0316 F4 (a building firer's bolt takes S-0518 claim 20's building-source adjustment)
    let building = (o.kind == crate::rules::Kind::Building).then_some(o.pos);
    // spec: S-0316 claim 4 (only a vehicle on the map can have its bolt attached; a carried vehicle is off the map)
    let vehicle_on_map = o.kind == crate::rules::Kind::Vehicle && o.on_map;
    // spec: S-0316 Terms and claim 3 (the bolt end, fixed at the shot: the shot target's target point by S-0518 claim 20, a cell's centre at its height)
    let to = crate::combat::range::target_point(w, target);
    w.effects.push(Effect::Bolt {
        firer,
        weapon,
        slot,
        burst,
        from,
        to,
        building,
        vehicle_on_map,
    });
    // spec: S-0316 claim 6 and S-0616 claim 1 (one system of the game-wide spark type at the bolt end, made with the bolt, after the bolt's own draw, whatever the shot's outcome)
    crate::emitters::electric_bolt_sparks(w, to);
}

/// Reports a fired shot's line-effect points ([`Effect::Line`]).
pub fn line(
    w: &mut crate::world::World,
    firer: ObjectId,
    weapon: crate::combat::data::WeaponId,
    from: Pos,
    target: crate::world::Target,
) {
    let Some(o) = w.obj(firer) else {
        return;
    };
    let building = (o.kind == crate::rules::Kind::Building).then_some(o.pos);
    // spec: S-0518 claim 18 (a prism master's shot draws its beam by its support count N)
    let prism = crate::linked::prism(w, firer).then_some(o.combat.support);
    // spec: S-0514 claim 24 (S is the firer's fire point for its primary weapon slot with no extra displacement)
    let primary = crate::combat::position::fire_point(w, firer, 0, 0);
    // spec: S-0518 claim 20 and S-0514 claim 24 (the target point of an object, else the cell's centre at its height)
    let to = crate::combat::range::target_point(w, target);
    w.effects.push(Effect::Line {
        firer,
        weapon,
        from,
        primary,
        to,
        building,
        prism,
    });
}
