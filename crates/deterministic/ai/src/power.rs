//! Power management of the own AI (PLAN §4.7, economy layer): what the
//! house's power bar shows a person, read without the blackouts, and the
//! power plant a person builds before a building that would leave the
//! house short.
//!
//! The M7b AI round 2 lane found three AI houses that judged their power by
//! the house total of S-0405 claim 4 alone:
//!
//! - a force-shield launch blacks out the launching house (S-0513 claim 30,
//!   S-0405 claim 6), and one drained power plant cuts the whole house's
//!   output to 0 (S-0405 claim 5); in both the total output reads 0
//!   whatever plants stand, and the house ordered a power plant every think
//!   (one Allied house stood with 47 plants built and no factory);
//! - a superweapon building or a defence was started with the power bar
//!   just full, so its drain left the house short until the next plant was
//!   built (the giving building and the house's other superweapons stood
//!   suspended, S-0512 claims 4 and 6, for 20 % to 99 % of their grant).
//!
//! A person reads the cause of a blackout off the bar and the base, does not
//! answer it with power plants, and builds a plant before a building whose
//! drain the plants cannot carry.

use glasnost_sim::econ::{TypeRef, power, production};
use glasnost_sim::orders::Order;
use glasnost_sim::rules::Kind;
use glasnost_sim::world::{HouseId, ObjectId, World};

/// The output the house's buildings give without a blackout: the sum of
/// each counted building's output with its health as it is (S-0405 claims 1
/// and 3, F1). A timed or drain blackout sets the house total to 0 (claim
/// 5) but not this sum.
pub fn plant_output(w: &World, house: HouseId) -> i32 {
    w.objects
        .values()
        // spec: S-0405 claim 3 (the counted buildings: every building of the house on the map)
        .filter(|o| o.kind == Kind::Building && o.owner == Some(house) && o.on_map)
        .map(|o| power::building_output(w, o.id))
        .fold(0i32, i32::saturating_add)
}

/// The base drain of building type `t` (S-0405 claim 2, F2, without
/// upgrades); 0 for a unit.
pub fn type_drain(w: &World, t: TypeRef) -> i32 {
    if t.kind != Kind::Building {
        return 0;
    }
    w.rules()
        .econ
        .power
        .buildings
        .get(t.index as usize)
        .map_or(0, |b| b.drain)
}

/// The base output of building type `t` (S-0405 claim 1, F1, at full
/// health); 0 for a unit.
pub fn type_output(w: &World, t: TypeRef) -> i32 {
    if t.kind != Kind::Building {
        return 0;
    }
    w.rules()
        .econ
        .power
        .buildings
        .get(t.index as usize)
        .map_or(0, |b| b.output)
}

/// The drain the house will have once the buildings on its lines are
/// placed: its drain (S-0405 claim 4) plus the drain of every building item
/// on its lines, finished or not, less the output those items will add.
pub fn coming_drain(w: &World, house: HouseId) -> i32 {
    let p = &w.houses[usize::from(house.0)].econ.power;
    let lines: i32 = w
        .econ
        .lines
        .lines
        .iter()
        .filter(|l| l.house == house)
        .filter_map(|l| l.item.as_ref())
        .filter(|i| i.t.kind == Kind::Building)
        .map(|i| type_drain(w, i.t) - type_output(w, i.t))
        .sum();
    p.drain + lines
}

/// Whether the house's plants leave it short of power, read as a person
/// reads the bar without a blackout: the plants' output ([`plant_output`])
/// is at most the drain to come ([`coming_drain`]).
pub fn short(w: &World, house: HouseId) -> bool {
    plant_output(w, house) <= coming_drain(w, house)
}

/// Whether building `t` now would leave the house short of power: its
/// drain (less its own output) added to the drain to come is at least the
/// plants' output.
pub fn short_after(w: &World, house: HouseId, t: TypeRef) -> bool {
    let d = type_drain(w, t) - type_output(w, t);
    d > 0 && plant_output(w, house) <= coming_drain(w, house) + d
}

/// Power before a building (M7b AI round 2): for every build order for a
/// building of the house's own choosing whose drain would leave the house
/// short of power ([`short_after`]), the house builds `plant`, the power
/// plant the plan would build, first, as a person does. A building on the
/// plant's own line (the two cannot run together) is replaced by the
/// plant; one on another line (a defence) keeps its order, and the plant is
/// ordered beside it once. Two orders for the plant become one. An order
/// for `before_power` keeps its place: a house that lost its refinery
/// rebuilds it first, short of power or not (M8 step B8, the brain's
/// `Script::refinery_lost`). Returns how many orders it changed or added.
pub fn plant_first(
    w: &World,
    house: HouseId,
    plant: Option<TypeRef>,
    before_power: Option<TypeRef>,
    out: &mut Vec<Order>,
) -> usize {
    let Some(plant) = plant else { return 0 };
    // A build order for a type already on a line resumes or queues it; its
    // drain is already in `coming_drain`.
    let on_line = |t: TypeRef| {
        w.econ
            .lines
            .lines
            .iter()
            .any(|l| l.house == house && l.item.as_ref().is_some_and(|i| i.t == t))
    };
    let plant_line = production::line_kind(w, plant);
    let mut planted = on_line(plant) || out.contains(&Order::Build { what: plant });
    let mut n = 0;
    let mut beside = false;
    for o in out.iter_mut() {
        if let Order::Build { what } = o
            && what.kind == Kind::Building
            && *what != plant
            && Some(*what) != before_power
            && !on_line(*what)
            && short_after(w, house, *what)
        {
            if production::line_kind(w, *what) == plant_line {
                *what = plant;
                n += 1;
                planted = true;
            } else if !planted {
                beside = true;
                planted = true;
            }
        }
    }
    if beside {
        out.push(Order::Build { what: plant });
        n += 1;
    }
    // One plant at a time on its line.
    let mut first = true;
    out.retain(|o| {
        if *o != (Order::Build { what: plant }) {
            return true;
        }
        std::mem::replace(&mut first, false)
    });
    n
}

/// The house's buildings that cause a drain blackout (S-0405 claim 5: a
/// counted building with output above 0 that is being drained), each with
/// its drainer (S-0608 claim 4).
pub fn drained_plants(w: &World, house: HouseId) -> Vec<(ObjectId, ObjectId)> {
    w.drain
        .links
        .iter()
        .filter(|&(&b, _)| {
            w.obj(b)
                .is_some_and(|o| o.owner == Some(house) && o.on_map && o.kind == Kind::Building)
                && power::building_output(w, b) > 0
        })
        .map(|(&b, &d)| (b, d))
        .collect()
}
