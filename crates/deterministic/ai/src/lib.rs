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
pub mod antiair;
pub mod brain;
pub mod bunker;
pub mod chrono;
pub mod crates;
pub mod deployers;
pub mod dominator;
pub mod drain;
pub mod fixture;
pub mod fixture_m6c;
pub mod fixture_m7a;
pub mod fixture_m7b;
pub mod game;
pub mod gattling;
pub mod grinder;
pub mod gunner;
pub mod jumpjets;
pub mod lifter;
pub mod linked;
pub mod memory;
pub mod mind;
pub mod mutator;
pub mod power;
pub mod psychic;
pub mod radiation;
pub mod scripted;
pub mod skirmish;
pub mod special;
pub mod tech_buildings;
pub mod terror;

/// Whether house `house` sees object `id` where it stands, as a person in
/// its seat does: on the map, and either the house's own or an ally's, or
/// on a cell the house has explored (fog of war is off in a skirmish,
/// S-0409 claim 19) and not hidden from it by a cloak it does not sense
/// (S-0516 Terms and claim 28: not drawn for it). The special abilities ask
/// this before they aim at a building (#36), the superweapon and ferry
/// cell choosers before they call a cell free (#38).
pub fn in_view(
    w: &glasnost_sim::world::World,
    house: glasnost_sim::world::HouseId,
    id: glasnost_sim::world::ObjectId,
) -> bool {
    let Some(o) = w.obj(id) else { return false };
    let me = &w.houses[usize::from(house.0)];
    // design: own AI (#36): the house's own and its allies' objects are known to it; the cloak test is asked of the others
    let friend = o.owner.is_some_and(|h| h == house || me.allied_with(h));
    // design: own AI (#38): the house's own and its allies' objects are seen wherever they stand
    o.on_map
        && (friend
            || (explored_part(w, house, o)
                // spec: S-0516 Terms and claim 28 (a cloaked object the house does not sense is hidden from it)
                && !glasnost_sim::cloak::hidden_from(w, id, Some(house))))
}

/// Whether the house has explored a cell of the object: any cell of a
/// building's foundation (S-0406 claim 2), a unit's own cell. A person sees
/// a large building from whichever side the house has explored (#37).
fn explored_part(
    w: &glasnost_sim::world::World,
    house: glasnost_sim::world::HouseId,
    o: &glasnost_sim::world::Object,
) -> bool {
    use glasnost_sim::econ::shroud;
    // design: own AI (#37): any cell the object stands on; its own cell first, the foundation only when that one is unexplored
    shroud::explored(w, house, o.pos.cell())
        || (o.kind == glasnost_sim::rules::Kind::Building
            && cells_of(w, o)
                .into_iter()
                .any(|c| shroud::explored(w, house, c)))
}

/// The cells object `o` stands on: a building's foundation from its
/// top-left cell (its position, S-0204 claim 4), any other object's own
/// cell.
pub(crate) fn cells_of(
    w: &glasnost_sim::world::World,
    o: &glasnost_sim::world::Object,
) -> Vec<glasnost_map::CellXY> {
    let at = o.pos.cell();
    if o.kind != glasnost_sim::rules::Kind::Building {
        return vec![at];
    }
    // design: own AI (#30): every foundation cell, the top-left one included
    let mut cells = glasnost_sim::econ::placement::foundation_cells(w, o.type_index, at);
    if !cells.contains(&at) {
        cells.push(at);
    }
    cells
}

/// Whether an enemy unit appears to house `house` as its disguise rather
/// than as itself: it appears disguised to the house (S-0516 claim 35) and
/// the house is shown the disguise now (claim 37: not during a running fake
/// blink, not under the house's detector coverage). A person takes it for
/// what it shows (#37).
pub fn shown_as_disguise(
    w: &glasnost_sim::world::World,
    house: glasnost_sim::world::HouseId,
    id: glasnost_sim::world::ObjectId,
) -> bool {
    // spec: S-0516 claims 35 and 37 (appears disguised to the house, and the disguise is what it is shown)
    glasnost_sim::cloak::appears_disguised(w, id, Some(house))
        && glasnost_sim::cloak::view::shown_disguise(w, id, Some(house)).is_some()
}

/// Whether a build order for `t` would be applied now: the type is
/// buildable (S-0411 Terms) and a switched-on factory building of the house
/// could start it. A person sees a switched-off factory (a building locked
/// by a temporal beam, say) and does not click a cameo the order would be
/// refused for, so the M7a build choices ask this rather than
/// buildability alone. The order must also pass the sidebar's legality
/// when issued (S-0411 claim 4): offered, not at limit and not refused by
/// claim 16's test (a full pad total for a dock-bound aircraft), and not on
/// a busy building line, so a special builder never gives an order the
/// sim refuses at every think (#426).
pub fn startable(
    w: &glasnost_sim::world::World,
    house: glasnost_sim::world::HouseId,
    t: glasnost_sim::econ::TypeRef,
) -> bool {
    glasnost_sim::econ::tech::buildable(w, house, t) && could_start(w, house, t)
}

/// [`startable`] without its buildability test: a switched-on factory
/// building of the house could start `t` and the order passes the
/// sidebar's legality ([`scripted::Script::startable`] asks the kept
/// buildability first; design: own AI, #60).
pub(crate) fn could_start(
    w: &glasnost_sim::world::World,
    house: glasnost_sim::world::HouseId,
    t: glasnost_sim::econ::TypeRef,
) -> bool {
    use glasnost_sim::econ::tech;
    // spec: S-0404 claim 3 (a build order is applied only with a switched-on factory that could make the type)
    w.objects.values().any(|b| {
            b.kind == glasnost_sim::rules::Kind::Building
                && b.owner == Some(house)
                && !b.econ.building.switched_off
                && tech::could_start(w, b, t)
        })
        // spec: S-0411 claim 4 and S-0407 claims 13 and 16 (the legality a build order meets when issued; design: own AI, #426, asks it before ordering)
        && glasnost_sim::econ::sidebar::legal(
            w,
            house,
            &glasnost_sim::orders::Order::Build { what: t },
        )
}
