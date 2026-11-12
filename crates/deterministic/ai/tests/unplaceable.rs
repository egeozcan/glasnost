//! The own AI and buildings it has no cell for (#40): a done building the
//! base has no room for does not stop the rest of the base, and a base
//! role whose every type has no cell does not hold up the roles after it.
//! A person whose finished building fits nowhere cancels it (S-0404 claim
//! 6) and builds something else.

use std::sync::Arc;

use glasnost_ai::fixture;
use glasnost_ai::game::Game;
use glasnost_sim::econ::TypeRef;
use glasnost_sim::orders::Order;
use glasnost_sim::rules::{Kind, SimRules};
use glasnost_sim::world::{HouseId, World};

/// A foundation taller than the synthetic map: no cell takes it.
const TOO_TALL: [i32; 2] = [2, fixture::SIZE * 2 + 1];

/// The skirmish rules with a building `TALL` no cell takes and a 1 × 1
/// defence `DEFX`; with `pill_too_tall` the infantry factory PILL takes no
/// cell either.
fn rules(pill_too_tall: bool) -> Arc<SimRules> {
    let mut r = fixture::ruleset();
    let mut tall = fixture::building(2, 2, 100, 0);
    tall.foundation = Some(TOO_TALL);
    r.sim.buildings.insert("TALL".into(), tall);
    let mut defx = fixture::building(1, 1, 100, 0);
    defx.defense = Some(true);
    r.sim.buildings.insert("DEFX".into(), defx);
    for n in ["TALL", "DEFX"] {
        r.registries
            .entry("buildings".into())
            .or_default()
            .push(n.into());
    }
    if pill_too_tall {
        r.sim
            .buildings
            .get_mut("PILL")
            .expect("PILL")
            .foundation = Some(TOO_TALL);
    }
    Arc::new(SimRules::new(&r, "synthetic-skirmish"))
}

fn game(pill_too_tall: bool) -> Game {
    Game::new(
        rules(pill_too_tall),
        &fixture::map(),
        "synthetic",
        fixture::catalogue(),
        fixture::setup(5),
    )
    .expect("the skirmish loads")
}

fn building(w: &World, name: &str) -> TypeRef {
    TypeRef {
        kind: Kind::Building,
        index: w.rules().index_of(Kind::Building, name).expect(name),
    }
}

/// House 0's buildings on the map of type `name`.
fn standing(w: &World, name: &str) -> usize {
    let t = building(w, name);
    w.objects
        .values()
        .filter(|o| {
            o.owner == Some(HouseId(0)) && o.on_map && o.kind == Kind::Building && o.type_index == t.index
        })
        .count()
}

/// House 0's buildings on the map.
fn buildings(w: &World) -> usize {
    w.objects
        .values()
        .filter(|o| o.owner == Some(HouseId(0)) && o.on_map && o.kind == Kind::Building)
        .count()
}

/// Whether house 0 has a done `t` on a line.
fn done(w: &World, t: TypeRef) -> bool {
    w.econ.lines.lines.iter().any(|l| {
        l.house == HouseId(0)
            && l.item
                .as_ref()
                .is_some_and(|i| i.t == t && i.done_tick.is_some())
    })
}

/// Whether house 0 has `t` on a line, as its item or queued.
fn lined(w: &World, t: TypeRef) -> bool {
    w.econ.lines.lines.iter().any(|l| {
        l.house == HouseId(0)
            && (l.item.as_ref().is_some_and(|i| i.t == t) || l.queue.contains(&t))
    })
}

/// The skirmish once house 0's yard and power plant stand, rich, with a
/// done `TALL` on its building line (ordered as a person would, by a click
/// on the sidebar).
fn tall_done() -> Game {
    let mut g = game(false);
    while g.world.tick < 3000 && standing(&g.world, "POWR") == 0 {
        g.step();
    }
    assert!(standing(&g.world, "POWR") > 0, "the base stands");
    g.world.houses[0].credits = 100_000;
    // spec: S-0404 claim 6 (the cancel-all order empties the line of the type)
    let lined: Vec<TypeRef> = g
        .world
        .econ
        .lines
        .lines
        .iter()
        .filter(|l| l.house == HouseId(0))
        .filter_map(|l| l.item.as_ref().map(|i| i.t))
        .filter(|t| t.kind == Kind::Building)
        .collect();
    for t in lined {
        g.issue(HouseId(0), Order::CancelAll { what: t });
    }
    let tall = building(&g.world, "TALL");
    g.issue(HouseId(0), Order::Build { what: tall });
    let end = g.world.tick + 3000;
    while g.world.tick < end && !done(&g.world, tall) {
        g.step();
    }
    assert!(done(&g.world, tall), "TALL is done");
    g
}

// design: own AI (#40) and S-0404 claim 6 (cancel and refund): a done building no cell takes does not stop the base for good: the house cancels it after a while and places further buildings.
#[test]
fn a_done_building_with_no_cell_gives_way() {
    let mut g = tall_done();
    let tall = building(&g.world, "TALL");
    let before = buildings(&g.world);
    for _ in 0..3000 {
        g.step();
    }
    assert!(!lined(&g.world, tall), "TALL still waits on its line");
    assert!(
        buildings(&g.world) > before,
        "no building placed: {} then {}",
        before,
        buildings(&g.world)
    );
}

// design: own AI (#40): a done defence is placed while a done building on the building line has no cell (only the first done building of the house was looked at).
#[test]
fn a_done_defence_is_placed_beside_a_stuck_building() {
    let mut g = tall_done();
    let defx = building(&g.world, "DEFX");
    g.issue(HouseId(0), Order::Build { what: defx });
    let end = g.world.tick + 600;
    while g.world.tick < end && standing(&g.world, "DEFX") == 0 {
        g.step();
    }
    assert!(standing(&g.world, "DEFX") > 0, "the defence is not placed");
}

// design: own AI (#40): an infantry factory no cell takes does not hold up the war factory (the base role whose types have no cell was wanted for ever, and nothing else was built).
#[test]
fn an_infantry_factory_with_no_cell_does_not_hold_up_the_war_factory() {
    let mut g = game(true);
    while g.world.tick < 9000 && standing(&g.world, "FACT") == 0 {
        g.step();
    }
    assert_eq!(standing(&g.world, "PILL"), 0, "PILL took a cell");
    assert!(standing(&g.world, "FACT") > 0, "no war factory by tick 9000");
}
