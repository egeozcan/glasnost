//! The own AI of M5b (PLAN §4.7) on the synthetic skirmish: it deploys,
//! builds an economy, produces, attacks, defends its base, rebuilds what it
//! lost, plays the game to a winner (S-0414, play to the last side), and
//! decides only on what its own house sees.
//!
//! The quick tests run in debug builds; the whole games are ignored there
//! (`cargo test --release -p glasnost-ai -- --include-ignored`).

use std::collections::BTreeMap;

use glasnost_ai::brain::Brain;
use glasnost_ai::{fixture, game::Game};
use glasnost_sim::econ::{shroud, slaves, tech, victory};
use glasnost_sim::objects;
use glasnost_sim::orders::Order;
use glasnost_sim::rules::Kind;
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, Target, World};

/// Ticks a whole synthetic game may take at most. With aircraft in the
/// army (M6a) a game on the fixture map runs longer: seed 5 ends near
/// 58 000 ticks.
const LIMIT: i32 = 70_000;

fn start(seed: u32) -> Game {
    Game::new(
        fixture::rules(),
        &fixture::map(),
        "synthetic-skirmish",
        fixture::catalogue(),
        fixture::setup(seed),
    )
    .expect("the synthetic skirmish loads")
}

fn name(w: &World, id: ObjectId) -> Option<String> {
    let o = w.obj(id)?;
    let r = w.rules();
    let t = tech::type_of(o);
    match t.kind {
        Kind::Building => r.buildings.get(t.index as usize).map(|b| b.name.clone()),
        Kind::Vehicle => r.vehicles.get(t.index as usize).map(|u| u.name.clone()),
        Kind::Infantry => r.infantry.get(t.index as usize).map(|u| u.name.clone()),
        _ => None,
    }
}

/// The house's live objects on the map named `n`.
fn of(w: &World, h: u8, n: &str) -> Vec<ObjectId> {
    w.objects
        .values()
        .filter(|o| o.owner == Some(HouseId(h)) && o.on_map && o.health > 0)
        .map(|o| o.id)
        .filter(|&id| name(w, id).as_deref() == Some(n))
        .collect()
}

/// Steps until `done` holds or `limit` ticks have run; returns whether it
/// held.
fn step_until(g: &mut Game, limit: i32, mut done: impl FnMut(&World) -> bool) -> bool {
    // The final tick leaves the tick number at itself (S-0201 claim 1): stop once the game is over.
    while g.world.tick < limit && g.outcome().is_none() {
        if done(&g.world) {
            return true;
        }
        g.step();
    }
    done(&g.world)
}

// The build-up: each AI house deploys its MCV and builds power, a refinery
// and a barracks by orders.
#[test]
fn deploys_and_builds_up() {
    let mut g = start(5);
    let built = step_until(&mut g, 3000, |w| {
        (0..2).all(|h| {
            ["YARD", "POWR", "PROC", "PILL"]
                .iter()
                .all(|n| !of(w, h, n).is_empty())
        })
    });
    assert!(built, "both houses built up by tick {}", g.world.tick);
    for h in 0..2 {
        assert!(
            of(&g.world, h, "MCV").is_empty(),
            "house {h} deployed its MCV"
        );
    }
}

// A lost power plant is rebuilt.
#[test]
fn rebuilds_a_lost_power_plant() {
    let mut g = start(5);
    assert!(step_until(&mut g, 1500, |w| !of(w, 0, "POWR").is_empty()));
    for id in of(&g.world, 0, "POWR") {
        objects::remove(&mut g.world, id);
    }
    let lost = g.world.tick;
    assert!(
        step_until(&mut g, lost + 1500, |w| !of(w, 0, "POWR").is_empty()),
        "the power plant lost at tick {lost} was rebuilt"
    );
}

/// The game when both houses have their first gun vehicle.
fn armed() -> Game {
    let mut g = start(5);
    assert!(step_until(&mut g, 9000, |w| (0..2)
        .all(|h| !of(w, h, "GUNR").is_empty())));
    g
}

// An enemy seen near the base draws the house's army to it in the next
// tick (the reactions run every tick).
#[test]
fn defends_the_base() {
    let mut g = armed();
    let yard = of(&g.world, 0, "YARD")[0];
    let at = g.world.obj(yard).unwrap().pos.cell();
    // On the far side of the yard from the army, out of its reach.
    let gun = g.world.obj(of(&g.world, 0, "GUNR")[0]).unwrap().pos.cell();
    let far = glasnost_map::CellXY::new(
        at.x + 1 + 5 * (at.x - gun.x).signum(),
        at.y + 1 + 5 * (at.y - gun.y).signum(),
    );
    let intruder = testkit::spawn(&mut g.world, Kind::Vehicle, "GUNR", 1, far, 0);
    // The intruder is seen once the next tick's sight update ran.
    g.step();
    assert!(shroud::seen(
        &g.world,
        HouseId(0),
        g.world.obj(intruder).unwrap().pos.cell()
    ));
    // The house's own reactions aside, the brain sends its army: with
    // every army unit's own target dropped, it orders them at the intruder.
    let army = of(&g.world, 0, "GUNR");
    let mut w = g.world.clone();
    for &id in &army {
        w.obj_mut(id).unwrap().target = None;
    }
    let map = fixture::map();
    let orders = Brain::new(&w, &map, HouseId(0)).think(&w);
    for &id in &army {
        assert!(
            orders.iter().any(|o| matches!(o,
                Order::Behaviour { actor, target: Some(Target::Object(t)), .. }
                    if *t == intruder && *actor == id)),
            "army unit {id:?} is sent at the intruder: {orders:?}"
        );
    }
}

// A hovercraft on the sea is army like any armed ground unit: an enemy
// seen near the base draws it, and it crosses the water onto the land to
// reach it, by orders only (S-0503 claims 1 and 4; the reach region of its
// amphibious zone spans sea and land).
#[test]
fn a_hover_crosses_the_sea_to_a_land_goal() {
    let map = fixture::map_naval();
    let mut g = Game::new(
        fixture::rules(),
        &map,
        "synthetic-naval",
        fixture::catalogue(),
        fixture::setup(5),
    )
    .expect("the naval skirmish loads");
    assert!(step_until(&mut g, 3000, |w| !of(w, 0, "YARD").is_empty()));
    let yard = g.world.obj(of(&g.world, 0, "YARD")[0]).unwrap().pos.cell();
    let sea = fixture::at(0, fixture::SEA_DY - 4).y;
    let water = |c: glasnost_map::CellXY| {
        map.cells
            .get(c)
            .is_some_and(|x| x.tile == fixture::WATER_TILE)
    };
    let start = glasnost_map::CellXY::new(yard.x, sea);
    assert!(water(start));
    let hover = testkit::spawn(&mut g.world, Kind::Vehicle, "HOVR", 0, start, 32768);
    // An unarmed enemy on land beside the yard, on the side away from the sea.
    let goal = glasnost_map::CellXY::new(yard.x + 1, yard.y + 5);
    assert!(!water(goal));
    let intruder = testkit::spawn(&mut g.world, Kind::Vehicle, "MINR", 1, goal, 0);
    // The amphibious zone's reach region spans the sea and the land, so
    // `Script::reachable` keeps the goal as ordered.
    let zone = glasnost_rules::sim::MovementZone::Amphibious;
    let mut probe = g.world.clone();
    glasnost_sim::regions::ensure(&mut probe);
    let from = glasnost_sim::regions::reach_in(&probe, start, zone);
    assert!(matches!(from, glasnost_sim::regions::Reach::Region(_)));
    assert_eq!(from, glasnost_sim::regions::reach_in(&probe, goal, zone));
    g.step();
    // With its own target dropped, the brain orders the hover at the intruder.
    let mut w = g.world.clone();
    w.obj_mut(hover).unwrap().target = None;
    let orders = Brain::new(&w, &map, HouseId(0)).think(&w);
    assert!(
        orders.iter().any(|o| matches!(o,
            Order::Behaviour { actor, target: Some(Target::Object(t)), .. }
                if *t == intruder && *actor == hover)),
        "the hover is sent at the intruder: {orders:?}"
    );
    let limit = g.world.tick + 3000;
    let ashore = step_until(&mut g, limit, |w| {
        w.obj(hover).is_some_and(|o| {
            let c = o.pos.cell();
            !water(c) && (c.x - goal.x).abs().max((c.y - goal.y).abs()) <= 6
        }) || w.obj(intruder).is_none_or(|o| o.health == 0)
    });
    assert!(
        ashore,
        "the hover crossed onto the land by tick {}",
        g.world.tick
    );
    let c = g.world.obj(hover).expect("the hover lives").pos.cell();
    assert!(!water(c), "it stands on land: {c:?}");
}

// The AI sees only its own house's shroud: two worlds that differ only by
// an enemy unit on a cell the house has not explored give the same orders.
#[test]
fn decides_only_on_what_it_sees() {
    let mut g = armed();
    let map = fixture::map();
    let hidden = (0..=2 * fixture::SIZE)
        .flat_map(|y| (0..=2 * fixture::SIZE).map(move |x| glasnost_map::CellXY::new(x, y)))
        .find(|&c| {
            g.world.cells.usable(c)
                && !shroud::explored(&g.world, HouseId(0), c)
                && g.world.objects.values().all(|o| o.pos.cell() != c)
        })
        .expect("a cell house 0 has not explored");
    let base = Brain::new(&g.world, &map, HouseId(0));
    let mut other = g.world.clone();
    testkit::spawn(&mut other, Kind::Vehicle, "GUNR", 1, hidden, 0);
    assert!(!shroud::explored(&other, HouseId(0), hidden));
    // Over a whole think period, from the same brain state.
    for _ in 0..glasnost_ai::scripted::THINK_PERIOD {
        let (mut a, mut b) = (base.clone(), base.clone());
        assert_eq!(
            format!("{:?}", a.think(&g.world)),
            format!("{:?}", b.think(&other)),
            "same orders at tick {}",
            g.world.tick
        );
        g.step();
        other = {
            let mut o = g.world.clone();
            testkit::spawn(&mut o, Kind::Vehicle, "GUNR", 1, hidden, 0);
            o
        };
    }
}

/// The superweapon launches seen so far in `w`'s effects: (house, type).
fn launches(w: &World) -> Vec<(Option<HouseId>, Option<u32>)> {
    w.effects
        .iter()
        .filter_map(|e| match e {
            glasnost_sim::combat::effects::Effect::Superweapon {
                event: glasnost_sim::superweapons::SwEvent::Launch,
                house,
                sw,
                ..
            } => Some((*house, *sw)),
            _ => None,
        })
        .collect()
}

// A ready superweapon is fired, through the fire order, at the seen enemy
// cell with the most seen enemies around it (S-0512 claim 8, S-0513
// claims 7 and 8: the missile kind's silo launch).
#[test]
fn fires_a_ready_superweapon_at_seen_enemies() {
    let mut g = start(5);
    assert!(step_until(&mut g, 3000, |w| !of(w, 0, "POWR").is_empty()));
    let yard = g.world.obj(of(&g.world, 0, "YARD")[0]).unwrap().pos.cell();
    let free = |w: &World, c: glasnost_map::CellXY| {
        w.cells.usable(c) && w.objects.values().all(|o| o.pos.cell() != c)
    };
    let silo_at = (2..12)
        .map(|d| glasnost_map::CellXY::new(yard.x, yard.y + d))
        .find(|&c| {
            (0..2).all(|dx| {
                (0..2).all(|dy| free(&g.world, glasnost_map::CellXY::new(c.x + dx, c.y + dy)))
            })
        })
        .expect("a free cell for the silo");
    testkit::spawn(&mut g.world, Kind::Building, "SILO", 0, silo_at, 0);
    // A row of three enemy power plants clear of the house, on cells a
    // scout of the house explored.
    let clear = |w: &World, c: glasnost_map::CellXY| {
        (0..6).all(|i| (0..2).all(|j| free(w, glasnost_map::CellXY::new(c.x + i, c.y + j))))
            && w.objects
                .values()
                .filter(|o| o.owner == Some(HouseId(0)))
                .all(|o| {
                    let p = o.pos.cell();
                    (p.x - c.x - 2).pow(2) + (p.y - c.y).pow(2) > 100
                })
    };
    let spot = (0..=2 * fixture::SIZE)
        .flat_map(|y| (0..=2 * fixture::SIZE).map(move |x| glasnost_map::CellXY::new(x, y)))
        .filter(|&c| clear(&g.world, c))
        .min_by_key(|&c| ((c.x - yard.x).pow(2) + (c.y - yard.y).pow(2), c.y, c.x))
        .expect("a spot clear of the house");
    let scout_at = glasnost_map::CellXY::new(spot.x + 2, spot.y + 1);
    let scout = testkit::spawn(&mut g.world, Kind::Infantry, "WALKER", 0, scout_at, 0);
    g.step();
    objects::remove(&mut g.world, scout);
    assert!(shroud::explored(&g.world, HouseId(0), spot));
    for i in 0..3 {
        let at = glasnost_map::CellXY::new(spot.x + 2 * i, spot.y);
        testkit::spawn(&mut g.world, Kind::Building, "POWR", 1, at, 0);
    }
    g.world.effects.clear();
    let limit = g.world.tick + 2000;
    let fired = step_until(&mut g, limit, |w| !launches(w).is_empty());
    assert!(
        fired,
        "the ready superweapon was fired by tick {}",
        g.world.tick
    );
    assert_eq!(launches(&g.world)[0].0, Some(HouseId(0)));
    let target = g.world.sw.houses[0]
        .nuke_target
        .expect("the strike's target");
    assert!(
        (target.x - spot.x - 2).abs() <= 3 && (target.y - spot.y).abs() <= 3,
        "aimed at the cluster: {target:?} near {spot:?}"
    );
}

// In a whole game both houses build a superweapon building, and a house
// fires its superweapon when charged.
#[test]
#[ignore = "long in debug builds; run in release (`cargo test --release -p glasnost-ai -- --include-ignored`)"]
fn builds_and_fires_a_superweapon() {
    let mut g = start(5);
    let mut fired: Vec<HouseId> = Vec::new();
    let mut built = [false; 2];
    while g.world.tick < LIMIT && g.outcome().is_none() && fired.len() < 2 {
        g.step();
        for h in 0..2 {
            built[h as usize] |= !of(&g.world, h, "SILO").is_empty();
        }
        for (h, _) in launches(&g.world) {
            if let Some(h) = h
                && !fired.contains(&h)
            {
                fired.push(h);
            }
        }
        g.world.effects.clear();
    }
    assert!(
        built.iter().all(|&b| b),
        "both houses built a silo: {built:?}"
    );
    assert!(
        !fired.is_empty(),
        "a house fired its superweapon by tick {}",
        g.world.tick
    );
}

// A lost harvester is rebuilt.
#[test]
#[ignore = "long in debug builds; run in release (`cargo test --release -p glasnost-ai -- --include-ignored`)"]
fn rebuilds_lost_harvesters() {
    let mut g = start(5);
    assert!(step_until(&mut g, 9000, |w| !of(w, 0, "MINR").is_empty()));
    for id in of(&g.world, 0, "MINR") {
        objects::remove(&mut g.world, id);
    }
    let lost = g.world.tick;
    assert!(
        step_until(&mut g, lost + 4000, |w| !of(w, 0, "MINR").is_empty()),
        "the harvesters lost at tick {lost} were rebuilt"
    );
}

// A lost construction yard is rebuilt: the house builds a unit that
// deploys into one and deploys it.
#[test]
#[ignore = "long in debug builds; run in release (`cargo test --release -p glasnost-ai -- --include-ignored`)"]
fn rebuilds_a_lost_yard() {
    let mut g = start(5);
    assert!(step_until(&mut g, 9000, |w| !of(w, 0, "FACT").is_empty()));
    for id in of(&g.world, 0, "YARD") {
        objects::remove(&mut g.world, id);
    }
    let lost = g.world.tick;
    let mut built = false;
    assert!(
        step_until(&mut g, lost + 6000, |w| {
            built |= !of(w, 0, "MCV").is_empty();
            !of(w, 0, "YARD").is_empty()
        }),
        "the yard lost at tick {lost} was rebuilt (a deploying unit built: {built})"
    );
    assert!(built, "by a unit built for it");
}

// AI against AI is played to a winner, and every house went through the
// whole loop on the way: deploy, economy, production, attack.
#[test]
#[ignore = "long in debug builds; run in release (`cargo test --release -p glasnost-ai -- --include-ignored`)"]
fn plays_to_a_winner() {
    for seed in [5, 6] {
        let mut g = start(seed);
        let mut most: [BTreeMap<String, usize>; 2] = Default::default();
        let mut paid = [false; 2];
        let mut last = [i32::MAX; 2];
        while g.world.tick < LIMIT {
            g.step();
            for h in 0..2u8 {
                let w = &g.world;
                let mut now: BTreeMap<String, usize> = BTreeMap::new();
                for o in w
                    .objects
                    .values()
                    .filter(|o| o.owner == Some(HouseId(h)) && o.on_map)
                {
                    if let Some(n) = name(w, o.id) {
                        *now.entry(n).or_default() += 1;
                    }
                }
                for (k, v) in now {
                    let e = most[usize::from(h)].entry(k).or_default();
                    *e = (*e).max(v);
                }
                let c = w.houses[usize::from(h)].credits;
                paid[usize::from(h)] |= c > last[usize::from(h)];
                last[usize::from(h)] = c;
            }
            if g.outcome().is_some() {
                break;
            }
        }
        let out = g.outcome().expect("the game ended");
        let winners: Vec<_> = out
            .results
            .iter()
            .filter(|r| r.1 == victory::Verdict::Win)
            .collect();
        assert_eq!(winners.len(), 1, "seed {seed}: one winner: {out:?}");
        for h in 0..2 {
            for n in ["YARD", "POWR", "PROC", "PILL", "FACT", "MINR"] {
                assert!(
                    most[h].get(n).copied().unwrap_or(0) >= 1,
                    "seed {seed} house {h} had a {n}: {:?}",
                    most[h]
                );
            }
            let army = most[h].get("GUNR").copied().unwrap_or(0)
                + most[h].get("WALKER").copied().unwrap_or(0);
            assert!(
                army >= 3,
                "seed {seed} house {h} produced an army: {:?}",
                most[h]
            );
            assert!(paid[h], "seed {seed} house {h} was paid for ore");
        }
        // spec: S-0201 claim 1 (the final tick leaves the tick number at itself; a replay of an ended game stops after it)
        let replayed = Game::run_replay(
            g.replay(),
            fixture::rules(),
            &fixture::map(),
            fixture::catalogue(),
            |_| {},
        )
        .expect("replays");
        assert_eq!(
            replayed.state_hash(),
            g.state_hash(),
            "seed {seed}: the replay of the ended game reproduces its final state"
        );
    }
}

// A house without income and without the credits for anything it could
// build sells a sellable building for its refund.
#[test]
fn a_broke_house_sells_a_building() {
    // The power plant has a build-up sprite, so it is sellable (S-0408 claim 3).
    let mut r = fixture::ruleset();
    let p = r
        .sim
        .buildings
        .get_mut("POWR")
        .expect("fixture power plant");
    p.econ.buildup = Some(true);
    p.econ.buildup_frames = Some(8);
    let rules = std::sync::Arc::new(glasnost_sim::rules::SimRules::new(&r, "synthetic-skirmish"));
    let new = || {
        Game::new(
            rules.clone(),
            &fixture::map(),
            "synthetic-skirmish",
            fixture::catalogue(),
            fixture::setup(5),
        )
        .expect("the synthetic skirmish loads")
    };
    let mut g = new();
    assert!(step_until(&mut g, 3000, |w| ["YARD", "POWR", "PROC"]
        .iter()
        .all(|n| !of(w, 0, n).is_empty())));
    // No income from now on: the harvesters go, and the credits too.
    for id in of(&g.world, 0, "MINR") {
        objects::remove(&mut g.world, id);
    }
    g.world.houses[0].credits = 0;
    let from = g.world.tick;
    let powr = of(&g.world, 0, "POWR").len();
    let sold = step_until(&mut g, from + 4000, |w| of(w, 0, "POWR").len() < powr);
    assert!(
        sold,
        "house 0 sold its power plant by tick {}",
        g.world.tick
    );
    // A house with income does not sell.
    let mut g = new();
    let sold = step_until(&mut g, 6000, |w| {
        w.objects.values().any(|o| o.econ.sale.is_some())
    });
    assert!(!sold, "no sale while the economy runs");
}

// A unit type refused for a reason other than a missing building (here
// its build limit) does not send the tech climb to build more factories of
// its kind: a factory the house already has clears nothing.
#[test]
fn a_refused_unit_builds_no_extra_factory() {
    let mut r = fixture::ruleset();
    // spec: S-0407 F2 (a build limit of 0 refuses the type whatever has been built)
    r.sim
        .vehicles
        .get_mut("GUNR")
        .expect("fixture gun")
        .econ
        .build_limit = Some(0);
    let rules = std::sync::Arc::new(glasnost_sim::rules::SimRules::new(&r, "synthetic-skirmish"));
    let mut g = Game::new(
        rules,
        &fixture::map(),
        "synthetic-skirmish",
        fixture::catalogue(),
        fixture::setup(5),
    )
    .expect("the synthetic skirmish loads");
    let mut most = 0;
    step_until(&mut g, 15_000, |w| {
        most = most.max(of(w, 0, "FACT").len());
        false
    });
    assert_eq!(most, 1, "house 0 kept one vehicle factory");
}

// spec: S-0314 claim 24 (our AI sends a vehicle to a depot only by an ordinary enter order): a damaged idle army vehicle of a house with an accepting repair depot is given an enter order on it; a healthy one is not.
#[test]
fn sends_damaged_vehicles_to_a_depot() {
    let mut r = fixture::ruleset();
    let mut depot = r.sim.buildings["FACT"].clone();
    depot.factory = None;
    depot.unit_repair = Some(true);
    r.sim.buildings.insert("DEPO".into(), depot);
    r.registries
        .get_mut("buildings")
        .unwrap()
        .push("DEPO".into());
    let rules = std::sync::Arc::new(glasnost_sim::rules::SimRules::new(&r, "depot"));
    let map = fixture::map();
    let mut w = glasnost_sim::load::new_game(rules, &map, fixture::catalogue(), &fixture::setup(5))
        .expect("loads");
    let at = |dx: i32, dy: i32| fixture::at(dx, dy);
    let d = testkit::spawn(&mut w, Kind::Building, "DEPO", 0, at(-10, 4), 0);
    let hurt = testkit::spawn(&mut w, Kind::Vehicle, "GUNR", 0, at(-12, 0), 0);
    let fine = testkit::spawn(&mut w, Kind::Vehicle, "GUNR", 0, at(-12, 2), 0);
    w.obj_mut(hurt).unwrap().health = 100;
    assert_eq!(w.tick % glasnost_ai::scripted::THINK_PERIOD, 0);
    let orders = Brain::new(&w, &map, HouseId(0)).think(&w);
    let enter = |id: ObjectId| {
        orders.iter().any(|o| {
            matches!(o, Order::Behaviour { actor, behaviour: glasnost_sim::rules::Behaviour::Enter, dest: Some(Target::Object(t)), .. }
                if *actor == id && *t == d)
        })
    };
    assert!(enter(hurt), "{orders:?}");
    assert!(!enter(fine));
}

/// The skirmish ruleset with slave miners for harvesters: SLAVE (a walker
/// with a rifle, storage 2, harvest rate 3), SMV (a slave-miner vehicle
/// with three slaves, built at the war factory, deploying into SMB) and SMB
/// (its 3 × 3 deployed form, not on the sidebar); MINR is no longer offered.
/// Invented values.
fn slave_rules() -> std::sync::Arc<glasnost_sim::rules::SimRules> {
    use glasnost_rules::sim::{BuildingType, LocomotionClass, Locomotor, UnitType};
    let mut r = fixture::ruleset();
    let group = |e: &mut glasnost_rules::sim_econ::EconFields| {
        e.enslaves = Some("SLAVE".into());
        e.slaves_number = Some(3);
        e.slave_regen_rate = Some(50);
        e.slave_reload_rate = Some(40);
    };
    r.sim
        .vehicles
        .get_mut("MINR")
        .expect("fixture harvester")
        .owner = Some(vec!["Nowhere".into()]);
    let mut slave = UnitType {
        strength: Some(100),
        speed: Some(6),
        locomotion_class: Some(LocomotionClass::Foot),
        locomotor: Some(Locomotor::Walk),
        storage: Some(2),
        sight: Some(4),
        primary: Some("Rifle".into()),
        ..Default::default()
    };
    slave.econ.harvest_rate = Some(3);
    r.sim.infantry.insert("SLAVE".into(), slave);
    r.registries
        .entry("infantry".into())
        .or_default()
        .push("SLAVE".into());
    let mut smv = UnitType {
        strength: Some(1000),
        speed: Some(4),
        rot: Some(6),
        locomotion_class: Some(LocomotionClass::Wheel),
        locomotor: Some(Locomotor::Drive),
        tech_level: Some(1),
        owner: Some(vec!["Alphaland".into(), "Betaland".into()]),
        cost: Some(1500),
        sight: Some(6),
        deploys_into: Some("SMB".into()),
        ..Default::default()
    };
    smv.econ.resource_gatherer = Some(true);
    smv.econ.resource_destination = Some(true);
    group(&mut smv.econ);
    r.sim.vehicles.insert("SMV".into(), smv);
    r.registries
        .entry("vehicles".into())
        .or_default()
        .push("SMV".into());
    let mut smb = BuildingType {
        strength: Some(1000),
        foundation: Some([3, 3]),
        undeploys_into: Some("SMV".into()),
        ..Default::default()
    };
    smb.econ.sight = Some(6);
    smb.econ.buildup = Some(true);
    smb.econ.buildup_frames = Some(10);
    group(&mut smb.econ);
    r.sim.buildings.insert("SMB".into(), smb);
    r.registries
        .entry("buildings".into())
        .or_default()
        .push("SMB".into());
    std::sync::Arc::new(glasnost_sim::rules::SimRules::new(&r, "synthetic-skirmish"))
}

// A side whose only gatherer is a slave miner builds one, sends it to a
// field with a harvest order, and earns income from its slaves; the house
// does not count as broke while they deliver; a lost one is rebuilt.
#[test]
#[ignore = "long in debug builds; run in release (`cargo test --release -p glasnost-ai -- --include-ignored`)"]
fn a_slave_miner_side_earns_income() {
    let mut g = Game::new(
        slave_rules(),
        &fixture::map(),
        "synthetic-skirmish",
        fixture::catalogue(),
        fixture::setup(5),
    )
    .expect("the synthetic skirmish loads");
    let deployed = step_until(&mut g, 12_000, |w| {
        (0..2).all(|h| !of(w, h, "SMB").is_empty())
    });
    assert!(
        deployed,
        "both houses deployed a slave miner by tick {}",
        g.world.tick
    );
    let from = g.world.tick;
    let mut last = [i32::MAX; 2];
    let mut paid = [false; 2];
    let mut slaves_out = [false; 2];
    step_until(&mut g, from + 3000, |w| {
        for h in 0..2u8 {
            let i = usize::from(h);
            let c = w.houses[i].credits;
            paid[i] |= c > last[i];
            last[i] = c;
            slaves_out[i] |= !of(w, h, "SLAVE").is_empty();
            // Slaves work for their master: the plan never sends them in a wave.
            assert!(
                of(w, h, "SLAVE")
                    .iter()
                    .all(|&s| w.obj(s).is_some_and(|o| o.attack_move.is_none())),
                "house {h} sent a slave with the army"
            );
        }
        paid.iter().all(|&p| p)
    });
    for h in 0..2 {
        assert!(slaves_out[h], "house {h}'s slaves went out");
        assert!(
            paid[h],
            "house {h} was paid by its slaves by tick {}",
            g.world.tick
        );
        assert!(
            of(&g.world, h as u8, "MINR").is_empty(),
            "house {h} has no harvester"
        );
    }
    assert!(
        !g.world.objects.values().any(|o| o.econ.sale.is_some()),
        "no sale while the slaves deliver"
    );
    // A lost slave miner is rebuilt and deployed again.
    for id in of(&g.world, 0, "SMB") {
        objects::remove(&mut g.world, id);
    }
    let lost = g.world.tick;
    assert!(
        step_until(&mut g, lost + 6000, |w| !of(w, 0, "SMB").is_empty()),
        "the slave miner lost at tick {lost} was rebuilt and deployed"
    );
    // Through the waves that follow, no slave with a master goes with the
    // army (those of the removed miner have none and may).
    let end = g.world.tick + 8000;
    step_until(&mut g, end, |w| {
        for h in 0..2u8 {
            assert!(
                of(w, h, "SLAVE").iter().all(|&s| !slaves::is_slave(w, s)
                    || w.obj(s).is_some_and(|o| o.attack_move.is_none())),
                "house {h} sent a slave with the army at tick {}",
                w.tick
            );
        }
        false
    });
}

/// `n` free land cells around `c`, nearest first.
fn free_cells(w: &World, c: glasnost_map::CellXY, n: usize) -> Vec<glasnost_map::CellXY> {
    let mut v: Vec<glasnost_map::CellXY> = (-6..=6)
        .flat_map(|dy| (-6..=6).map(move |dx| glasnost_map::CellXY::new(c.x + dx, c.y + dy)))
        .filter(|&k| {
            w.cells.usable(k)
                && w.cells
                    .cell(k)
                    .is_some_and(|x| x.land != glasnost_map::terrain::land::WATER)
                && w.cells
                    .occ(k, glasnost_sim::world::Layer::Ground)
                    .is_some_and(|o| o.empty())
        })
        .collect();
    v.sort_by_key(|k| ((k.x - c.x).pow(2) + (k.y - c.y).pow(2), k.y, k.x));
    v.truncate(n);
    v
}

// The ferry (S-0507, PLAN §4.7): on the strait map no rifleman walks to
// the enemy; the AI sends idle riflemen aboard its hovering transport with
// a click on it, carries them over the water and lets them out on the far
// shore with a click on the transport, by orders only.
#[test]
fn ferries_infantry_across_a_strait() {
    let map = fixture::map_strait();
    let mut g = Game::new(
        fixture::rules(),
        &map,
        "synthetic-strait",
        fixture::catalogue(),
        fixture::setup(5),
    )
    .expect("the strait skirmish loads");
    assert!(step_until(&mut g, 3000, |w| !of(w, 0, "YARD").is_empty()));
    let yard = g.world.obj(of(&g.world, 0, "YARD")[0]).unwrap().pos.cell();
    let mid = fixture::at(0, 0).x;
    let side = (yard.x - mid).signum();
    let cells = free_cells(&g.world, glasnost_map::CellXY::new(yard.x, yard.y + 5), 4);
    assert_eq!(cells.len(), 4, "room beside the yard");
    let lift = testkit::spawn(&mut g.world, Kind::Vehicle, "LIFT", 0, cells[0], 0);
    let riders: Vec<ObjectId> = cells[1..]
        .iter()
        .map(|&c| testkit::spawn(&mut g.world, Kind::Infantry, "WALKER", 0, c, 0))
        .collect();
    let mut boarded = false;
    let mut riding: Vec<ObjectId> = Vec::new();
    let mut across = None;
    let limit = g.world.tick + 6000;
    step_until(&mut g, limit, |w| {
        let now: Vec<ObjectId> = riders
            .iter()
            .copied()
            .filter(|&p| glasnost_sim::transport::riding(w, p) == Some(lift))
            .collect();
        boarded |= !now.is_empty();
        for &p in &riding {
            if !now.contains(&p)
                && let Some(o) = w.obj(p).filter(|o| o.on_map && o.health > 0)
            {
                let c = o.pos.cell();
                if (c.x - mid) * side < -fixture::STRAIT_HALF {
                    across = Some(c);
                }
            }
        }
        riding = now;
        across.is_some()
    });
    assert!(boarded, "the riflemen boarded the ferry");
    let c = across.expect("a rifleman was let out across the strait");
    assert_ne!(
        g.world.cells.cell(c).map(|x| x.land),
        Some(glasnost_map::terrain::land::WATER),
        "on land"
    );
}

/// The skirmish rules with a paradrop (test data of our own): the
/// superweapon DROP (paradrop kind, not power-dependent) given by the
/// building AIRB, which no house can build, the plane PDPLANE and the
/// paratrooper PARA (an armed rifleman no house can build), two per plane
/// for every side group.
fn paradrop_rules() -> std::sync::Arc<glasnost_sim::rules::SimRules> {
    use glasnost_rules::sim::{BuildingType, Decimal, Locomotor, UnitType};
    use glasnost_rules::sim_econ::SuperweaponType;
    let mut r = fixture::ruleset();
    let reg = |r: &mut glasnost_rules::schema::Ruleset, k: &str, n: &str| {
        r.registries.entry(k.into()).or_default().push(n.into());
    };
    r.sim.superweapons.insert(
        "DROP".into(),
        SuperweaponType {
            kind: Some("ParachuteDrop".into()),
            action: Some("ParaDrop".into()),
            recharge_time: Some(Decimal("0.1".into())),
            is_powered: Some(false),
            ..Default::default()
        },
    );
    reg(&mut r, "superweapons", "DROP");
    let mut airb = BuildingType {
        strength: Some(500),
        foundation: Some([1, 1]),
        ..Default::default()
    };
    airb.econ.super_weapon = Some("DROP".into());
    r.sim.buildings.insert("AIRB".into(), airb);
    reg(&mut r, "buildings", "AIRB");
    r.sim.aircraft.insert(
        "PDPLANE".into(),
        UnitType {
            strength: Some(100),
            speed: Some(30),
            rot: Some(12),
            locomotor: Some(Locomotor::Fly),
            ..Default::default()
        },
    );
    reg(&mut r, "aircraft", "PDPLANE");
    let mut para = r.sim.infantry["WALKER"].clone();
    para.owner = None;
    para.tech_level = None;
    r.sim.infantry.insert("PARA".into(), para);
    reg(&mut r, "infantry", "PARA");
    let g = &mut r.sim.general.superweapons;
    for (inf, num) in [
        (&mut g.ally_para_drop_inf, &mut g.ally_para_drop_num),
        (&mut g.sov_para_drop_inf, &mut g.sov_para_drop_num),
        (&mut g.yuri_para_drop_inf, &mut g.yuri_para_drop_num),
        (&mut g.amer_para_drop_inf, &mut g.amer_para_drop_num),
    ] {
        *inf = Some(vec!["PARA".into()]);
        *num = Some(vec![2]);
    }
    std::sync::Arc::new(glasnost_sim::rules::SimRules::new(&r, "synthetic-paradrop"))
}

// A ready paradrop is fired near home (S-0512 claim 8, S-0513 claim 42);
// the paratroopers land (S-0522) and join the army: a brain whose plan
// names another type as its army still sends them at a nearby enemy.
#[test]
fn fires_a_paradrop_whose_troops_join_the_army() {
    let mut g = Game::new(
        paradrop_rules(),
        &fixture::map(),
        "synthetic-paradrop",
        fixture::catalogue(),
        fixture::setup(5),
    )
    .expect("the synthetic skirmish loads");
    assert!(step_until(&mut g, 3000, |w| !of(w, 0, "YARD").is_empty()));
    let yard = g.world.obj(of(&g.world, 0, "YARD")[0]).unwrap().pos.cell();
    let free = |w: &World, c: glasnost_map::CellXY| {
        w.cells.usable(c) && w.objects.values().all(|o| o.pos.cell() != c)
    };
    let airb_at = (2..12)
        .map(|d| glasnost_map::CellXY::new(yard.x - d, yard.y))
        .find(|&c| free(&g.world, c))
        .expect("a free cell for the paradrop building");
    testkit::spawn(&mut g.world, Kind::Building, "AIRB", 0, airb_at, 0);
    let plane = |w: &World| {
        w.objects
            .values()
            .find(|o| {
                o.kind == Kind::Aircraft
                    && o.owner == Some(HouseId(0))
                    && w.rules().aircraft[o.type_index as usize].name == "PDPLANE"
            })
            .map(|o| o.id)
    };
    let limit = g.world.tick + 1000;
    assert!(
        step_until(&mut g, limit, |w| plane(w).is_some()),
        "the brain fired its paradrop"
    );
    let landed = |w: &World| {
        of(w, 0, "PARA")
            .into_iter()
            .filter(|&id| !w.obj(id).unwrap().fall.falling)
            .collect::<Vec<_>>()
    };
    let limit = g.world.tick + 3000;
    assert!(
        step_until(&mut g, limit, |w| landed(w).len() == 2),
        "both paratroopers landed"
    );
    let para = landed(&g.world)[0];
    let at = g.world.obj(para).unwrap().pos.cell();
    assert!(
        (at.x - yard.x).abs() <= 12 && (at.y - yard.y).abs() <= 12,
        "near home: {at:?}, yard {yard:?}"
    );
    // An enemy within reach, then a brain whose plan's army names only GUNR.
    let mut w = g.world.clone();
    let near = (2..6)
        .map(|d| glasnost_map::CellXY::new(at.x + d, at.y))
        .find(|&c| free(&w, c))
        .expect("a free cell for the enemy");
    let enemy = testkit::spawn(&mut w, Kind::Infantry, "WALKER", 1, near, 0);
    glasnost_sim::tick::step(&mut w);
    let mut plan = glasnost_ai::brain::default_plan();
    plan.names.army = vec!["GUNR".into()];
    let mut brain = Brain::with_plan(&w, &fixture::map(), HouseId(0), plan);
    let orders = brain.think(&w);
    assert!(
        orders.iter().any(|o| matches!(
            o,
            Order::Behaviour { actor, target: Some(Target::Object(e)), .. }
                if landed(&w).contains(actor) && *e == enemy
        )),
        "a paratrooper is sent at the enemy: {orders:?}"
    );
}
