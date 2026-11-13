//! The own AI expands toward a far resource field with a second
//! construction yard (M7b AI round 3), on synthetic data.
//!
//! - Deploying a yard vehicle gives a construction yard wherever the
//!   deploy's cell test passes, with no adjacency to the base (S-0415
//!   claim 3, S-0406 claim 4).
//! - A rich house with its economy kept builds one yard vehicle beyond its
//!   yard when an explored field lies beyond the reach of every yard on its
//!   half of the map, drives it to that field and deploys it there; a
//!   poor house, or one with no far field, builds none (tuning of our own
//!   opponent).
//! - M8 step B1: a house with its economy whole and no superweapon
//!   building short of money buys the yard vehicle step by step like any
//!   item, without twice its cost in hand (S-0404 claim 4: no money is
//!   needed to start an item); and while the yard vehicle waits on an
//!   unmet prerequisite, the house builds what clears it (S-0407 claim 8).
//! - M8 step B5: that house must also earn the yard vehicle's cost per
//!   income window and have no superweapon building left to climb to (a
//!   person climbs to his superweapons first, then expands).
//! - M8 step B8: a house whose explored ore within reach of its refineries
//!   has run dry builds a refinery beside the field nearest home that it
//!   has scouted, when one fits there (tuning of our own opponent).

use glasnost_ai::brain::Brain;
use glasnost_ai::{fixture, game::Game};
use glasnost_map::{CellXY, Map};
use glasnost_sim::orders::Order;
use std::sync::Arc;

use glasnost_sim::rules::{Behaviour, Kind, SimRules};
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, Pos, Target, World};

const H: HouseId = HouseId(0);

/// The far field: 24 cells north-east of start 0, on its half of the
/// map.
fn far() -> CellXY {
    fixture::at(-8, -20)
}

/// [`fixture::map`] with a field of 5 × 5 ore cells around [`far`].
fn far_field_map() -> Map {
    let mut m = fixture::map();
    let f = far();
    for ox in -2..=2 {
        for oy in -2..=2 {
            if let Some(cell) = m.cells.get_mut(CellXY::new(f.x + ox, f.y + oy)) {
                cell.overlay = ((ox + oy + 4) % 12) as u8;
                cell.overlay_data = 6;
            }
        }
    }
    m
}

fn name(w: &World, id: ObjectId) -> Option<String> {
    let o = w.obj(id)?;
    let r = w.rules();
    match o.kind {
        Kind::Building => r
            .buildings
            .get(o.type_index as usize)
            .map(|b| b.name.clone()),
        Kind::Vehicle => r
            .vehicles
            .get(o.type_index as usize)
            .map(|u| u.name.clone()),
        _ => None,
    }
}

/// House 0's live objects on the map named `n`.
fn of(w: &World, n: &str) -> Vec<ObjectId> {
    w.objects
        .values()
        .filter(|o| o.owner == Some(H) && o.on_map && o.health > 0)
        .map(|o| o.id)
        .filter(|&id| name(w, id).as_deref() == Some(n))
        .collect()
}

fn dist2(a: CellXY, b: CellXY) -> i64 {
    let (dx, dy) = (i64::from(a.x - b.x), i64::from(a.y - b.y));
    dx * dx + dy * dy
}

/// Whether no object stands on cell `c`.
fn free(w: &World, c: CellXY) -> bool {
    w.objects.values().all(|o| !o.on_map || o.pos.cell() != c)
}

/// A skirmish on `map` once house 0's weapons factory and two harvesters
/// (the plan's) stand.
fn built_up(map: &Map) -> Game {
    built_up_with(map, fixture::rules())
}

/// [`built_up`] under `rules`.
fn built_up_with(map: &Map, rules: Arc<SimRules>) -> Game {
    let mut g = Game::new(
        rules,
        map,
        "synthetic-expansion",
        fixture::catalogue(),
        fixture::setup(5),
    )
    .expect("the skirmish loads");
    while g.world.tick < 9000 && (of(&g.world, "FACT").is_empty() || of(&g.world, "MINR").len() < 2)
    {
        g.step();
    }
    assert!(
        !of(&g.world, "FACT").is_empty() && of(&g.world, "MINR").len() >= 2,
        "a weapons factory and two harvesters by tick 9000"
    );
    g
}

/// House 0 scouts the far field.
fn scout(w: &mut World) {
    glasnost_sim::econ::shroud::area_reveal(w, H, Pos::centre(far(), 0), 6);
}

/// House 0's orders at the next think tick, with the expansion builds and
/// moves counted.
fn think(w: &mut World, map: &Map) -> (Vec<Order>, u32, u32) {
    testkit::run(w, 1);
    while w.tick % glasnost_ai::scripted::THINK_PERIOD != 0 {
        testkit::run(w, 1);
    }
    let mut brain = Brain::with_plan(w, map, H, fixture::plan());
    let orders = brain.think(w);
    let u = brain.special_uses();
    (orders, u.expansion_builds, u.expansion_moves)
}

fn builds_mcv(w: &World, orders: &[Order]) -> bool {
    let mcv = w.rules().index_of(Kind::Vehicle, "MCV").unwrap();
    orders.iter().any(
        |o| matches!(o, Order::Build { what } if what.kind == Kind::Vehicle && what.index == mcv),
    )
}

// S-0415 claim 3 and S-0406 claim 4 (a deployed yard needs no adjacency) and S-0403 claim 6 (a second building factory): a rich house that has scouted a far field builds a yard vehicle.
#[test]
fn a_rich_house_with_a_far_field_builds_a_yard_vehicle() {
    let map = far_field_map();
    let g = built_up(&map);
    let mut w = g.world.clone();
    scout(&mut w);
    w.houses[0].credits = 100_000;
    assert!(w.cells.usable(far()) && glasnost_sim::econ::shroud::explored(&w, H, far()));
    let (orders, builds, _) = think(&mut w, &map);
    assert!(builds_mcv(&w, &orders), "{orders:?}");
    assert_eq!(builds, 1);
    // Without the far field, none. (On the far-field map the harvesters
    // explore it themselves once the refinery beside the home field has
    // mined that field down, M8 step B8.)
    let plain = fixture::map();
    let mut w = built_up(&plain).world;
    w.houses[0].credits = 100_000;
    let (orders, builds, _) = think(&mut w, &plain);
    assert!(!builds_mcv(&w, &orders), "{orders:?}");
    assert_eq!(builds, 0);
}

// Tuning of our own opponent (M8 step B8): a house whose explored ore within reach of its refinery has run dry builds a second refinery beside the far field it has scouted, its foundation within reach of the field; while the home field is rich it builds none there.
#[test]
#[ignore = "long in debug builds; run in release (`cargo test --release -p glasnost-ai -- --include-ignored`)"]
fn builds_a_refinery_by_the_far_field_once_the_home_field_runs_dry() {
    let map = far_field_map();
    let (mut rich, mut dry) = (built_up(&map), built_up(&map));
    scout(&mut rich.world);
    scout(&mut dry.world);
    // The reach the house counts harvesters in (14 cells).
    let r = 14;
    let by_far = |w: &World| {
        of(w, "PROC")
            .into_iter()
            .filter_map(|id| w.obj(id))
            .any(|o| dist2(o.pos.cell(), far()) <= r * r)
    };
    assert!(!by_far(&rich.world));
    let start = rich.world.tick;
    while rich.world.tick < start + 3000 {
        rich.world.houses[0].credits = 3000;
        rich.step();
    }
    assert!(
        !by_far(&rich.world),
        "no refinery by the far field while the home field is rich"
    );
    // Everything but the far field is mined out.
    let ore: Vec<CellXY> = dry
        .world
        .cells
        .grid
        .iter()
        .map(|(c, _)| c)
        .filter(|&c| {
            glasnost_sim::econ::resource::resource_at(&dry.world, c).is_some()
                && dist2(c, far()) > 8
        })
        .collect();
    for c in ore {
        glasnost_sim::crush::remove_overlay(&mut dry.world, c);
    }
    while dry.world.tick < start + 3000 && !by_far(&dry.world) {
        dry.world.houses[0].credits = 3000;
        dry.step();
    }
    assert!(
        by_far(&dry.world),
        "a refinery by the far field by tick {}",
        dry.world.tick
    );
}

// Tuning of our own opponent (M8 step B9): a house whose home field has run dry wants no refinery by the far field while no cell of its build spiral takes one, since it would never order it and the rest of its plan would wait; the want is read afresh, so it comes back when a cell opens.
#[test]
#[ignore = "long in debug builds; run in release (`cargo test --release -p glasnost-ai -- --include-ignored`)"]
fn wants_no_far_field_refinery_it_could_not_order() {
    let map = far_field_map();
    let mut g = built_up(&map);
    scout(&mut g.world);
    let w = &mut g.world;
    let ore: Vec<CellXY> = w
        .cells
        .grid
        .iter()
        .map(|(c, _)| c)
        .filter(|&c| {
            glasnost_sim::econ::resource::resource_at(w, c).is_some() && dist2(c, far()) > 8
        })
        .collect();
    for c in ore {
        glasnost_sim::crush::remove_overlay(w, c);
    }
    let brain = Brain::with_plan(w, &map, H, fixture::plan());
    assert!(
        brain.wants_field_refinery(w),
        "the far field's refinery is wanted"
    );
    // A power plant beyond the build spiral (12 rings around the yard)
    // toward the far field, so that a refinery may stand beside it there,
    // and a soldier on every free cell of the spiral, none beyond it.
    let home = w.obj(of(w, "YARD")[0]).expect("the yard").pos.cell();
    let plant = CellXY::new(home.x + 9, home.y - 14);
    testkit::spawn(w, Kind::Building, "POWR", 0, plant, 0);
    for dy in -12..=12 {
        for dx in -12..=12 {
            let c = CellXY::new(home.x + dx, home.y + dy);
            if free(w, c) && w.cells.usable(c) {
                testkit::spawn(w, Kind::Infantry, "WALKER", 0, c, 0);
            }
        }
    }
    let brain = Brain::with_plan(w, &map, H, fixture::plan());
    assert!(
        !brain.wants_field_refinery(w),
        "no cell of the build spiral takes it"
    );
}

/// The yard vehicle type.
fn mcv(w: &World) -> glasnost_sim::econ::TypeRef {
    glasnost_sim::econ::TypeRef {
        kind: Kind::Vehicle,
        index: w.rules().index_of(Kind::Vehicle, "MCV").unwrap(),
    }
}

/// A building type by name.
fn building(w: &World, n: &str) -> glasnost_sim::econ::TypeRef {
    glasnost_sim::econ::TypeRef {
        kind: Kind::Building,
        index: w.rules().index_of(Kind::Building, n).unwrap(),
    }
}

/// A cell south of house 0's yard where building `b` may be placed.
fn room_for(w: &World, b: glasnost_sim::econ::TypeRef) -> CellXY {
    let yard = w.obj(of(w, "YARD")[0]).unwrap().pos.cell();
    (4..24)
        .flat_map(|d| (-6..=6).map(move |dx| CellXY::new(yard.x + dx, yard.y + d)))
        .find(|&c| {
            glasnost_sim::econ::placement::cell_test(
                w,
                b.index,
                c,
                &glasnost_sim::econ::placement::Test::order(H),
            )
        })
        .expect("room for the building")
}

/// Spawns `n` for house 0 on a free cell south of its yard.
fn stand(w: &mut World, n: &str) {
    let t = building(w, n);
    let at = room_for(w, t);
    testkit::spawn(w, Kind::Building, n, 0, at, 0);
}

/// A brain for house 0 that has watched its income over two whole income
/// windows, its credits rising by `per_window` in each: the brain thinks
/// at the start of each window (orders dropped) while the simulation does
/// not run, so the credits are its only income and nothing pays.
fn earning(w: &mut World, map: &Map, per_window: i32) -> Brain {
    let window = glasnost_ai::scripted::INCOME_WINDOW;
    let mut brain = Brain::with_plan(w, map, H, fixture::plan());
    w.tick = (w.tick / window + 1) * window;
    brain.think(w);
    for _ in 0..2 {
        w.tick += window;
        w.houses[0].credits += per_window;
        brain.think(w);
    }
    // The next think tick, inside the window.
    w.tick += glasnost_ai::scripted::THINK_PERIOD;
    brain
}

// Tuning of our own opponent (M8 steps B1 and B5) and S-0404 claim 4 (no money is needed to start an item): a house with less than twice a yard vehicle's cost in hand still builds one when it keeps its harvesters, earns the yard vehicle's cost per income window and has a building of every superweapon kind; earning one credit less, it does not.
#[test]
fn a_house_earning_a_yard_vehicle_per_window_buys_one_step_by_step() {
    let map = far_field_map();
    let g = built_up(&map);
    let mut w = g.world.clone();
    scout(&mut w);
    stand(&mut w, "LAB");
    stand(&mut w, "SILO");
    testkit::run(&mut w, 60);
    let cost = glasnost_sim::econ::production::cost_field(&w, mcv(&w));
    for (per_window, buys) in [(cost, true), (cost - 1, false)] {
        let mut w = w.clone();
        let mut brain = earning(&mut w, &map, per_window);
        w.houses[0].credits = 2 * cost - 1;
        let probe = brain.expansion_probe(&w);
        assert_eq!(probe.income, Some(i64::from(per_window)), "{probe:?}");
        assert!(!probe.superweapon_wanted, "{probe:?}");
        let before = brain.special_uses().expansion_builds;
        let orders = brain.think(&w);
        assert_eq!(builds_mcv(&w, &orders), buys, "{per_window}: {orders:?}");
        assert_eq!(
            brain.special_uses().expansion_builds - before,
            u32::from(buys)
        );
        if !buys {
            assert!(probe.blocked_by().contains(&"not rich"), "{probe:?}");
        }
    }
}

// Tuning of our own opponent (M8 step B5): a house earning well with a superweapon building still to climb to (here the building its superweapon building waits for) buys no yard vehicle with less than twice its cost to spare; the house goes for the superweapon first.
#[test]
fn a_house_with_a_superweapon_to_climb_to_buys_no_yard_vehicle() {
    let map = far_field_map();
    let g = built_up(&map);
    let mut w = g.world.clone();
    scout(&mut w);
    let cost = glasnost_sim::econ::production::cost_field(&w, mcv(&w));
    let mut brain = earning(&mut w, &map, 10 * cost);
    w.houses[0].credits = 2 * cost - 1;
    let probe = brain.expansion_probe(&w);
    assert!(probe.superweapon_wanted, "{probe:?}");
    assert!(!probe.superweapon_building, "{probe:?}");
    assert!(probe.blocked_by().contains(&"not rich"), "{probe:?}");
    let before = brain.special_uses().expansion_builds;
    let orders = brain.think(&w);
    assert!(!builds_mcv(&w, &orders), "{orders:?}");
    assert_eq!(brain.special_uses().expansion_builds, before);
    // Twice the cost to spare is enough.
    w.houses[0].credits = 2 * cost;
    let orders = brain.think(&w);
    assert!(builds_mcv(&w, &orders), "{orders:?}");
}

// Tuning of our own opponent (M8 step B1) and S-0404 claim 4 (a step waits for money it lacks): a poor house saving for a superweapon building that is short of money builds no yard vehicle.
#[test]
fn a_poor_house_saving_for_a_superweapon_builds_no_yard_vehicle() {
    let map = far_field_map();
    let g = built_up(&map);
    let mut w = g.world.clone();
    scout(&mut w);
    let lab = building(&w, "LAB");
    let at = room_for(&w, lab);
    testkit::spawn(&mut w, Kind::Building, "LAB", 0, at, 0);
    // The sidebar offers SILO once the house has been updated with LAB.
    testkit::run(&mut w, 60);
    w.houses[0].credits = 1;
    let silo = building(&w, "SILO");
    assert!(glasnost_sim::orders::issue(
        &mut w,
        H,
        Order::Build { what: silo }
    ));
    let (orders, builds, _) = think(&mut w, &map);
    assert!(!builds_mcv(&w, &orders), "{orders:?}");
    assert_eq!(builds, 0);
    let probe = Brain::with_plan(&w, &map, H, fixture::plan()).expansion_probe(&w);
    assert!(probe.saving, "{probe:?}");
    assert!(probe.blocked_by().contains(&"not rich"), "{probe:?}");
}

/// The fixture's rules with DEPOT, a plain building that no army type
/// needs, as the yard vehicle's prerequisite.
fn depot_rules() -> Arc<SimRules> {
    let mut r = fixture::ruleset();
    r.sim
        .buildings
        .insert("DEPOT".into(), fixture::building(2, 2, 500, -10));
    r.registries
        .entry("buildings".into())
        .or_default()
        .push("DEPOT".into());
    r.sim.vehicles.get_mut("MCV").unwrap().econ.prerequisite = Some(vec!["DEPOT".into()]);
    Arc::new(SimRules::new(&r, "synthetic-expansion-depot"))
}

// S-0407 claim 8 (a prerequisite the house lacks keeps a type off its sidebar) and tuning of our own opponent (M8 step B1): a house that would expand while its yard vehicle waits on a prerequisite builds that prerequisite, places it, and then builds the yard vehicle.
#[test]
fn builds_what_the_yard_vehicle_waits_for() {
    let map = far_field_map();
    let mut g = built_up_with(&map, depot_rules());
    scout(&mut g.world);
    let mcv = mcv(&g.world);
    assert!(!glasnost_sim::econ::tech::buildable(&g.world, H, mcv));
    let start = g.world.tick;
    let uses = |g: &Game| g.special_uses()[0].1;
    while g.world.tick < start + 6000 && uses(&g).expansion_builds == 0 {
        g.world.houses[0].credits = 100_000;
        g.step();
    }
    assert!(uses(&g).expansion_unblocks >= 1, "{:?}", uses(&g));
    assert_eq!(of(&g.world, "DEPOT").len(), 1, "DEPOT placed");
    assert!(
        uses(&g).expansion_builds >= 1,
        "a yard vehicle ordered by tick {}",
        g.world.tick
    );
}

// S-0415 claims 1 to 3 (the deploy order and its cell test, no adjacency): a spare yard vehicle is driven to the far field and deployed there; no second one is built.
#[test]
#[ignore = "long in debug builds; run in release (`cargo test --release -p glasnost-ai -- --include-ignored`)"]
fn drives_a_yard_vehicle_to_the_far_field_and_deploys_it() {
    let map = far_field_map();
    let mut g = built_up(&map);
    scout(&mut g.world);
    g.world.houses[0].credits = 100_000;
    let yard = g.world.obj(of(&g.world, "YARD")[0]).unwrap().pos.cell();
    let spare = (2..12)
        .map(|d| CellXY::new(yard.x + d, yard.y - 4))
        .find(|&c| g.world.cells.usable(c) && free(&g.world, c))
        .expect("a free cell by the yard");
    let mcv = testkit::spawn(&mut g.world, Kind::Vehicle, "MCV", 0, spare, 0);
    let mut w = g.world.clone();
    let (orders, builds, moves) = think(&mut w, &map);
    assert_eq!(builds, 0, "the spare counts toward the second yard");
    assert_eq!(moves, 1);
    let dest = orders.iter().find_map(|o| match o {
        Order::Behaviour {
            actor,
            behaviour: Behaviour::Move,
            dest: Some(Target::Cell(c)),
            ..
        } if *actor == mcv => Some(*c),
        _ => None,
    });
    let dest = dest.expect("a move order for the spare");
    // The spiral of cells a person looks at around the field (12 rings).
    let reach = 12;
    assert!(
        dist2(dest, far()) <= 2 * reach * reach,
        "{dest:?} near the field {:?}",
        far()
    );
    let start = g.world.tick;
    while g.world.tick < start + 6000 && of(&g.world, "YARD").len() < 2 {
        g.step();
    }
    let yards: Vec<CellXY> = of(&g.world, "YARD")
        .into_iter()
        .filter_map(|id| g.world.obj(id))
        .map(|o| o.pos.cell())
        .collect();
    assert_eq!(
        yards.len(),
        2,
        "the spare deployed by tick {}",
        g.world.tick
    );
    // The reach the house counts harvesters in (14 cells).
    let r = 14;
    assert!(
        yards.iter().any(|&y| dist2(y, far()) <= r * r),
        "a yard by the far field: {yards:?}"
    );
    assert!(of(&g.world, "MCV").is_empty());
}

/// [`fixture::map`] with one ore cell at [`far`] in a lake of water out
/// to 14 cells from it (the script's field reach), so that no yard fits
/// within that reach of the field, while the outer rings of the spiral
/// around it hold land.
fn moat_map() -> Map {
    let mut m = fixture::map();
    let f = far();
    for ox in -14i32..=14 {
        for oy in -14i32..=14 {
            if ox * ox + oy * oy > 196 {
                continue;
            }
            let Some(cell) = m.cells.get_mut(CellXY::new(f.x + ox, f.y + oy)) else {
                continue;
            };
            if ox == 0 && oy == 0 {
                cell.overlay = 0;
                cell.overlay_data = 6;
            } else {
                cell.tile = fixture::WATER_TILE;
                cell.overlay = glasnost_map::NO_OVERLAY;
            }
        }
    }
    m
}

// design: own AI (#41): the cells looked at around the far field are those within the reach the house counts the field's harvesters in (14 cells, `scripted::FIELD_REACH`), so a yard vehicle sent there deploys on arrival; with none there it deploys where it stands, or near it (#5). Before, the spiral's first fit could be a cell of an outer ring beyond that reach, where the arriving vehicle did not deploy and was sent on.
#[test]
fn the_cell_by_the_far_field_is_within_its_reach() {
    let map = moat_map();
    let mut g = built_up(&map);
    scout(&mut g.world);
    g.world.houses[0].credits = 100_000;
    let yard = g.world.obj(of(&g.world, "YARD")[0]).unwrap().pos.cell();
    let spare = (2..12)
        .map(|d| CellXY::new(yard.x + d, yard.y - 4))
        .find(|&c| g.world.cells.usable(c) && free(&g.world, c))
        .expect("a free cell by the yard");
    let mcv = testkit::spawn(&mut g.world, Kind::Vehicle, "MCV", 0, spare, 0);
    let mut w = g.world.clone();
    let (orders, _, _) = think(&mut w, &map);
    let dest = orders.iter().find_map(|o| match o {
        Order::Behaviour {
            actor,
            behaviour: Behaviour::Move,
            dest: Some(Target::Cell(c)),
            ..
        } if *actor == mcv => Some(*c),
        _ => None,
    });
    // The script's field reach (`scripted::FIELD_REACH`, crate-private).
    let reach = 14; // nocite: tuning of our own opponent
    // The spiral's rings (`scripted::SPIRAL_RINGS`, crate-private): with no
    // cell by the field (the moat), the unit is sent near where it stands.
    let rings = 12; // nocite: tuning of our own opponent
    let by_the_unit = |c: CellXY| (c.x - spare.x).abs().max((c.y - spare.y).abs()) <= rings;
    match dest {
        Some(dest) => assert!(
            dist2(dest, far()) <= reach * reach || by_the_unit(dest),
            "{dest:?} (squared distance {}) beyond the reach of the field {:?}, and not by the unit at {spare:?}",
            dist2(dest, far()),
            far()
        ),
        None => assert!(
            orders
                .iter()
                .any(|o| matches!(o, Order::Deploy { actor } if *actor == mcv)),
            "{orders:?}"
        ),
    }
}

// Tuning of our own opponent and PLAN §4.7 (M8 step B10): a house whose home field has run dry and that cannot pay for both a power plant and the refinery by the far field orders the refinery, though its drain would leave the house short of power; with the money for both it builds the plant first, as before.
#[test]
#[ignore = "long in debug builds; run in release (`cargo test --release -p glasnost-ai -- --include-ignored`)"]
fn a_broke_house_orders_the_far_field_refinery_before_power() {
    let map = far_field_map();
    let mut g = built_up(&map);
    scout(&mut g.world);
    let w = &mut g.world;
    let ore: Vec<CellXY> = w
        .cells
        .grid
        .iter()
        .map(|(c, _)| c)
        .filter(|&c| {
            glasnost_sim::econ::resource::resource_at(w, c).is_some() && dist2(c, far()) > 8
        })
        .collect();
    for c in ore {
        glasnost_sim::crush::remove_overlay(w, c);
    }
    w.econ.lines.lines.retain(|l| l.house != H);
    let proc_ = building(w, "PROC");
    let powr = building(w, "POWR");
    let drain = glasnost_ai::power::type_drain(w, proc_);
    // Power to spare, but less than the refinery drains: barracks (each
    // drains) bring the margin down, a plant brings it up.
    loop {
        let margin =
            glasnost_ai::power::plant_output(w, H) - glasnost_ai::power::coming_drain(w, H);
        if margin <= 0 {
            stand(w, "POWR");
        } else if margin > drain {
            stand(w, "PILL");
        } else {
            break;
        }
        // The house works its power out in its update (S-0405 claim 7).
        testkit::run(w, 1);
    }
    let wants = |w: &World| Brain::with_plan(w, &map, H, fixture::plan()).wants_field_refinery(w);
    assert!(wants(w), "the far field's refinery is wanted");
    assert!(glasnost_ai::power::short_after(w, H, proc_));
    let price = |w: &World, t| glasnost_sim::econ::production::price(w, H, t);
    let both = price(w, proc_) + price(w, powr);
    let mut broke = w.clone();
    broke.houses[0].credits = both - 1;
    let (orders, _, _) = think(&mut broke, &map);
    assert!(orders.contains(&Order::Build { what: proc_ }), "{orders:?}");
    assert!(!orders.contains(&Order::Build { what: powr }), "{orders:?}");
    w.houses[0].credits = both;
    let (orders, _, _) = think(w, &map);
    assert!(orders.contains(&Order::Build { what: powr }), "{orders:?}");
    assert!(
        !orders.contains(&Order::Build { what: proc_ }),
        "{orders:?}"
    );
}
