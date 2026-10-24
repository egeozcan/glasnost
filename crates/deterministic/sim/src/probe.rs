//! Phase marks for the tools' tick benchmark (M8 step C4 part 2): our own
//! tooling, no game behaviour.
//!
//! [`crate::tick::step_with`] tells a [`Probe`] each time the tick moves on
//! to another part of S-0203's pipeline (and, inside the turn pass, to
//! another object's turn and to the parts of that turn). A mark carries no
//! numbers and returns nothing: the simulation never reads anything back,
//! so a probe cannot change the state or its hash. The clock lives in the
//! caller (the benchmark in `glasnost-app`), never here.
//!
//! [`crate::tick::step`] runs with the unit probe `()`, whose
//! [`Probe::on`] is `false`: every mark and every classification behind it
//! drops out of the compiled tick.

use crate::world::{ObjectId, World};

/// A part of the tick, as the probe is told of it. The time from one mark
/// to the next belongs to the earlier mark's part.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Phase {
    /// Our own AI houses think (`glasnost_ai::Game::step`), by house number.
    Think(u8),
    /// The AI's orders are issued (`orders::issue`).
    Issue,
    /// Row 1: the trigger poll.
    Triggers,
    /// Row 9: resource growth and spread.
    Resources,
    /// Row 17: the kamikaze re-order cycle.
    Kamikaze,
    /// Row 12: disc lasers.
    Discs,
    /// The ambient fade and flash phase.
    Lighting,
    /// The domination update (row 3).
    Domination,
    /// The weather storm (row 3).
    Storm,
    /// Row 13: radiation maintenance.
    Radiation,
    /// Shroud regrowth.
    ShroudRegrowth,
    /// The re-cover sweep.
    ShroudSweep,
    /// The turn pass's bookkeeping between turns (`next_turn`).
    TurnPass,
    /// One object's turn, from its start, by the kind of object.
    Turn(TurnClass),
    /// Inside a living unit's or building's turn: from the behaviour start
    /// through the behaviour processing.
    Behaviour,
    /// Inside a living unit's or building's turn: from the behaviour
    /// processing's end to the group update (sparks, the radiation hit,
    /// bombs, spawners, the cloak; a building's remaining turn).
    BeforeMove,
    /// Inside a living unit's turn: from the group update through the
    /// locomotor's movement.
    Movement,
    /// Inside a living unit's turn: from the movement's end through its
    /// firing steps and the rest of the turn.
    AfterMove,
    /// The designator sweep after each turn.
    StrikeSweep,
    /// Row 15: blast rings.
    Rings,
    /// Row 16: crates.
    Crates,
    /// Row 6: the production lines.
    Production,
    /// Row 7: the houses.
    Houses,
    /// Row 8: the orders.
    Orders,
    /// The end of the tick (one-time tags, the tick number).
    EndTick,
    /// Derived data after the tick (the regions).
    Regions,
    /// A finer part, named by the code that marks it (our AI's parts,
    /// `ai:…`).
    Part(&'static str),
    /// The step is over.
    Done,
}

/// The kinds of turn in the update order, as `tick::object_turn` tells
/// them apart.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum TurnClass {
    /// A projectile.
    Projectile,
    /// An ordinary animation.
    Anim,
    /// A particle system.
    GasSystem,
    /// A particle system's animation.
    GasAnim,
    /// A burning walker.
    Walker,
    /// An infantry-making animation.
    Maker,
    /// A domination animation.
    DominationAnim,
    /// A repair depot's slot animation.
    DepotAnim,
    /// A debris piece.
    Piece,
    /// A magnetic wave.
    Wave,
    /// A killed object still in play.
    Dead,
    /// A building.
    Building,
    /// A vehicle.
    Vehicle,
    /// An infantry unit.
    Infantry,
    /// An aircraft.
    Aircraft,
    /// A terrain object.
    Terrain,
    /// An object number no longer in play.
    Other,
}

/// What the tick tells about its parts.
pub trait Probe {
    /// Whether marks are wanted. The unit probe says no, so the tick skips
    /// the marks and their classification.
    fn on(&self) -> bool {
        true
    }
    /// The tick moves on to `phase`.
    fn mark(&mut self, phase: Phase);
}

/// The unit probe: no marks.
impl Probe for () {
    #[inline(always)]
    fn on(&self) -> bool {
        false
    }
    #[inline(always)]
    fn mark(&mut self, _: Phase) {}
}

/// The kind of turn object `id` takes, in the same order of tests as
/// `tick::object_turn` (only for marks).
pub fn classify(w: &World, id: ObjectId) -> TurnClass {
    use crate::rules::Kind;
    if w.projectiles.contains_key(&id) {
        return TurnClass::Projectile;
    }
    if w.anims.anims.contains_key(&id) {
        return TurnClass::Anim;
    }
    if w.gas.systems.contains_key(&id) {
        return TurnClass::GasSystem;
    }
    if w.gas.anims.contains_key(&id) {
        return TurnClass::GasAnim;
    }
    if w.walkers.contains_key(&id) {
        return TurnClass::Walker;
    }
    if w.mutation.makers.contains_key(&id) {
        return TurnClass::Maker;
    }
    if w.domination.anims.contains_key(&id) {
        return TurnClass::DominationAnim;
    }
    if w.depot_anims.anims.contains_key(&id) {
        return TurnClass::DepotAnim;
    }
    if w.pieces.contains_key(&id) {
        return TurnClass::Piece;
    }
    if w.waves.contains_key(&id) {
        return TurnClass::Wave;
    }
    let Some(o) = w.obj(id) else {
        return TurnClass::Other;
    };
    if crate::combat::damage::dead(o) {
        return TurnClass::Dead;
    }
    match o.kind {
        Kind::Building => TurnClass::Building,
        Kind::Vehicle => TurnClass::Vehicle,
        Kind::Infantry => TurnClass::Infantry,
        Kind::Aircraft => TurnClass::Aircraft,
        Kind::Terrain => TurnClass::Terrain,
    }
}
