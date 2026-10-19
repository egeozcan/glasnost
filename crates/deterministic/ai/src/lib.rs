//! Our own computer opponent. It issues only Orders a human player could
//! issue and sees only its own player's shroud.
//!
//! **Milestone M5** brings a minimal opponent; later milestones deepen it.
//! It runs inside the simulation tick and uses the simulation RNG, so it is a
//! simulation crate: the determinism lints apply.
//!
//! - [`scripted`]: a fixed build plan played through orders (M5a's scripted
//!   skirmish).
//! - [`skirmish`]: the headless skirmish runner that records a replay.
//! - [`game`]: a game with its AI houses (world and AI state stepped, saved
//!   and hashed together); [`brain`]: one AI house's state and thinking.
//! - [`special`]: garrisoning, engineers capturing and spies infiltrating
//!   (M6c); [`tech_buildings`]: which tech buildings an engineer is worth
//!   sending to (S-0515).
//! - [`deployers`]: deployable units deployed near the enemy and packed up
//!   again (S-0517).
//! - [`jumpjets`]: jumpjet units sent over obstacles to reach the enemy (S-0502).
//! - [`memory`]: the orders a unit did not take, so that the brain tries
//!   another goal instead of the same click every think (PLAN §4.7).
//! - [`grinder`]: the recycling building built and fed with controlled
//!   enemy units, badly damaged units and surplus infantry (S-0603).
//! - [`bunker`]: tank bunkers built near the base, filled with suitable
//!   vehicles and emptied by a deploy click to repair, retreat or attack
//!   (S-0606).
//! - [`mutator`]: the genetic mutator fired at a dense group of seen enemy
//!   infantry with none of the house's own or allied infantry in its reach
//!   (S-0611).
//! - [`fixture`]: the synthetic skirmish (invented rules and map).

pub mod absorber;
pub mod brain;
pub mod bunker;
pub mod chrono;
pub mod crates;
pub mod deployers;
pub mod drain;
pub mod fixture;
pub mod fixture_m6c;
pub mod fixture_m7a;
pub mod fixture_m7b;
pub mod game;
pub mod gattling;
pub mod grinder;
pub mod jumpjets;
pub mod lifter;
pub mod linked;
pub mod memory;
pub mod mind;
pub mod mutator;
pub mod psychic;
pub mod radiation;
pub mod scripted;
pub mod skirmish;
pub mod special;
pub mod tech_buildings;
pub mod terror;

/// Whether a build order for `t` would be applied now: the type is
/// buildable (S-0411 Terms) and a switched-on factory building of the house
/// could start it. A person sees a switched-off factory (a building locked
/// by a temporal beam, say) and does not click a cameo the order would be
/// refused for, so the M7a build choices ask this rather than
/// buildability alone.
pub fn startable(
    w: &glasnost_sim::world::World,
    house: glasnost_sim::world::HouseId,
    t: glasnost_sim::econ::TypeRef,
) -> bool {
    use glasnost_sim::econ::tech;
    // spec: S-0404 claim 3 (a build order is applied only with a switched-on factory that could make the type)
    tech::buildable(w, house, t)
        && w.objects.values().any(|b| {
            b.kind == glasnost_sim::rules::Kind::Building
                && b.owner == Some(house)
                && !b.econ.building.switched_off
                && tech::could_start(w, b, t)
        })
}
