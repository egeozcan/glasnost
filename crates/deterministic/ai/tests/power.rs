//! The own AI's power management (S-0405, PLAN §4.7; M7b AI round 2): it
//! reads its power off its plants, not off a house total that a blackout
//! sets to 0 (S-0405 claims 5 and 6), builds a power plant before a
//! building whose drain its plants cannot carry, and sends its army at a
//! drainer on one of its power plants. The tests use the invented types of
//! the synthetic skirmishes (fixture: POWR gives 100, LAB drains 60, PILL
//! 20, FACT 30, RADR 40; fixture_m7b: Alpha's drainer DISC).

use glasnost_ai::brain::Brain;
use glasnost_ai::fixture;
use glasnost_ai::fixture_m7b as kit;
use glasnost_ai::game::Game;
use glasnost_ai::power;
use glasnost_map::CellXY;
use glasnost_sim::econ::power as sim_power;
use glasnost_sim::econ::{TypeRef, shroud};
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::world::{HouseId, ObjectId, Pos, Target, World};
use glasnost_sim::{drain, objects, testkit, tick};

fn fixture_game() -> Game {
    Game::new(
        fixture::rules(),
        &fixture::map(),
        "synthetic",
        fixture::catalogue(),
        fixture::setup(5),
    )
    .expect("the synthetic skirmish loads")
}

fn building(w: &World, name: &str) -> TypeRef {
    TypeRef {
        kind: Kind::Building,
        index: w
            .rules()
            .index_of(Kind::Building, name)
            .expect("the type exists"),
    }
}

/// A free cell far from both starts, for buildings of a test's own.
fn spot(dx: i32, dy: i32) -> CellXY {
    fixture::at(dx, dy)
}

/// Steps until the next house power update has run.
fn settle(w: &mut World) {
    testkit::run(w, 1);
    while w.tick % 15 != 0 {
        testkit::run(w, 1);
    }
}

// spec: S-0405 claims 1, 2 and 4 (output and drain by building type)
// A build order for a building whose drain the plants cannot carry becomes
// one for the power plant; one the plants carry, a power plant's own and a
// unit's stay as they are.
#[test]
fn builds_a_power_plant_before_a_building_it_cannot_carry() {
    let mut w = fixture_game().world;
    for (name, at) in [
        ("POWR", spot(-6, -12)),
        ("FACT", spot(-2, -12)),
        ("RADR", spot(3, -12)),
    ] {
        testkit::spawn(&mut w, Kind::Building, name, 0, at, 0);
    }
    settle(&mut w);
    let h = HouseId(0);
    assert_eq!(power::plant_output(&w, h), 100);
    assert_eq!(power::coming_drain(&w, h), 70);
    assert!(!power::short(&w, h));
    let (powr, lab, pill) = (
        building(&w, "POWR"),
        building(&w, "LAB"),
        building(&w, "PILL"),
    );
    let gunr = TypeRef {
        kind: Kind::Vehicle,
        index: w.rules().index_of(Kind::Vehicle, "GUNR").unwrap(),
    };
    let mut out = vec![
        Order::Build { what: lab },
        Order::Build { what: pill },
        Order::Build { what: powr },
        Order::Build { what: gunr },
    ];
    assert_eq!(power::plant_first(&w, h, Some(powr), &mut out), 1);
    assert_eq!(
        out,
        vec![
            Order::Build { what: powr },
            Order::Build { what: pill },
            Order::Build { what: gunr },
        ],
        "only the lab (drain 60 on 30 spare) waits for a plant, and one plant is ordered"
    );
}

// spec: S-0405 claim 6 (a timed blackout sets the house output to 0)
// A timed blackout (a force-shield launch, S-0513 claim 30) leaves the
// house's plants standing: the AI does not read it as short power, and its
// build orders are those it gives without the blackout.
#[test]
fn a_blackout_is_not_short_power() {
    let mut w = fixture_game().world;
    for (name, at) in [("POWR", spot(-6, -12)), ("FACT", spot(-2, -12))] {
        testkit::spawn(&mut w, Kind::Building, name, 0, at, 0);
    }
    settle(&mut w);
    let mut dark = w.clone();
    sim_power::start_blackout(&mut dark, HouseId(0), 600);
    settle(&mut dark);
    assert!(sim_power::blackout_running(&dark, HouseId(0)));
    assert_eq!(
        dark.houses[0].econ.power.output, 0,
        "the house total reads 0"
    );
    assert!(
        !power::short(&dark, HouseId(0)),
        "the plants still carry the base"
    );
    let builds = |w: &World| -> Vec<Order> {
        let mut b = Brain::with_plan(
            w,
            &fixture::map(),
            HouseId(0),
            glasnost_ai::brain::default_plan(),
        );
        b.think(w)
            .into_iter()
            .filter(|o| matches!(o, Order::Build { .. }))
            .collect()
    };
    let mut lit = w.clone();
    while lit.tick < dark.tick {
        testkit::run(&mut lit, 1);
    }
    assert_eq!(
        builds(&dark),
        builds(&lit),
        "the same build orders in the dark"
    );
}

fn kit_game() -> Game {
    let mut g = Game::new(
        kit::rules(),
        &kit::map(),
        kit::MAP_NAME,
        fixture::catalogue(),
        kit::setup(5),
    )
    .expect("the M7b skirmish loads");
    let disc = g.world.rules().index_of(Kind::Vehicle, "DISC").unwrap();
    let discs: Vec<ObjectId> = g
        .world
        .objects
        .values()
        .filter(|o| o.kind == Kind::Vehicle && o.type_index == disc)
        .map(|o| o.id)
        .collect();
    for d in discs {
        objects::remove(&mut g.world, d);
    }
    while g.world.tick <= g.world.orders_blocked_through || g.world.tick % 15 != 0 {
        g.step();
    }
    g
}

fn home(w: &World, h: u8) -> CellXY {
    w.objects
        .values()
        .find(|o| o.owner == Some(HouseId(h)) && o.on_map)
        .map(|o| o.pos.cell())
        .unwrap()
}

fn attacked(orders: &[Order], who: ObjectId) -> Option<ObjectId> {
    orders.iter().rev().find_map(|o| match o {
        Order::Behaviour {
            actor,
            behaviour: Behaviour::Attack,
            target: Some(Target::Object(x)),
            ..
        } if *actor == who => Some(*x),
        _ => None,
    })
}

// spec: S-0405 claim 5 (one drained building with output above 0 blacks out the house)
// A drained power plant blacks out the house, but its plants still carry
// its base, so the AI is not short of power and builds no plant for it; it
// knows the plant and its drainer (the drainer is a raider its army goes
// for, when a unit can hit it).
#[test]
fn a_drained_plant_is_known_with_its_drainer() {
    let g = kit_game();
    let mut w = g.world.clone();
    let b = home(&w, 1);
    let at = |dx: i32, dy: i32| CellXY::new(b.x + dx, b.y + dy);
    let plant = testkit::spawn(&mut w, Kind::Building, "POWR", 1, at(4, -8), 0);
    let disc = testkit::spawn(&mut w, Kind::Vehicle, "DISC", 0, at(4, -12), 0);
    let gun = testkit::spawn(&mut w, Kind::Vehicle, "GUNR", 1, at(8, -14), 0);
    // Alpha's brain sends the disc; play until the link forms.
    let mut alpha = Brain::new(&w, &kit::map(), HouseId(0));
    let mut linked = false;
    for _ in 0..60 * 15 {
        if w.tick % 15 == 0 {
            for o in alpha.think(&w) {
                if o.actor() == Some(disc) {
                    orders::issue(&mut w, HouseId(0), o);
                }
            }
        }
        tick::step(&mut w);
        if drain::partner(&w, disc) == Some(plant) {
            linked = true;
            break;
        }
    }
    assert!(linked, "the disc drains the plant (S-0608 claim 2)");
    settle(&mut w);
    shroud::area_reveal(&mut w, HouseId(1), Pos::centre(at(4, -12), 0), 8);
    assert!(w.houses[1].econ.power.drain_blackout, "a drain blackout");
    assert_eq!(power::drained_plants(&w, HouseId(1)), vec![(plant, disc)]);
    assert_eq!(w.houses[1].econ.power.output, 0, "the house total reads 0");
    assert!(
        power::plant_output(&w, HouseId(1)) >= 100,
        "the plants still stand"
    );
    // The fixture's gun cannot hit the disc in the air, so the brain gives
    // it no order at it (a person's cursor shows no attack).
    if let Some(o) = w.obj_mut(gun) {
        o.target = None;
    }
    let mut beta = Brain::new(&w, &kit::map(), HouseId(1));
    let out = beta.think(&w);
    assert_eq!(
        attacked(&out, gun),
        None,
        "no order the gun cannot carry out"
    );
    assert_eq!(beta.special_uses().economy_defence, 0);
}

// spec: S-0610 claim 6 (a berserk unit takes no orders)
// A unit of the house that an enemy shoots at its harvester goes for that
// enemy, though it stands away from home (M7b AI round 2: undefended
// slaves and harvesters).
#[test]
fn defends_a_harvester_under_fire() {
    let mut w = fixture_game().world;
    // Between the starts, away from house 0's start.
    let at = |dx: i32, dy: i32| fixture::at(dx - 8, dy + 10);
    let minr = testkit::spawn(&mut w, Kind::Vehicle, "MINR", 0, at(0, 0), 0);
    let gun = testkit::spawn(&mut w, Kind::Vehicle, "GUNR", 0, at(4, 2), 0);
    let raider = testkit::spawn(&mut w, Kind::Infantry, "WALKER", 1, at(2, 4), 0);
    // The house sees the scene (its units' sight, without waiting for their turns).
    shroud::area_reveal(&mut w, HouseId(0), Pos::centre(at(2, 2), 0), 8);
    settle(&mut w);
    // The gun is not yet shooting at the raider on its own.
    if let Some(o) = w.obj_mut(gun) {
        o.target = None;
    }
    let mut calm = w.clone();
    if let Some(o) = calm.obj_mut(raider) {
        o.target = None;
    }
    if let Some(o) = w.obj_mut(raider) {
        o.target = Some(Target::Object(minr));
    }
    let think = |w: &World| {
        let mut b = Brain::with_plan(
            w,
            &fixture::map(),
            HouseId(0),
            glasnost_ai::brain::default_plan(),
        );
        let out = b.think(w);
        (attacked(&out, gun), b.special_uses().economy_defence)
    };
    assert_eq!(
        think(&w),
        (Some(raider), 1),
        "the gun defends the harvester"
    );
    assert_eq!(think(&calm).1, 0, "no raid, no economy defence");
    // Sent once, the gun is not sent at the raider again by the defence
    // from where it stands (the count of the defence's orders stays).
    let mut b = Brain::with_plan(
        &w,
        &fixture::map(),
        HouseId(0),
        glasnost_ai::brain::default_plan(),
    );
    let _ = b.think(&w);
    let _ = b.think(&w);
    assert_eq!(b.special_uses().economy_defence, 1);
    // Once it has moved and lost its target (another order) while the
    // raider still shoots at the harvester, it is sent again at once.
    let mut moved = w.clone();
    for o in b.think(&moved) {
        orders::issue(&mut moved, HouseId(0), o);
    }
    orders::issue(
        &mut moved,
        HouseId(0),
        Order::Behaviour {
            actor: gun,
            behaviour: Behaviour::Move,
            attack_move: false,
            target: None,
            dest: Some(Target::Cell(at(8, 0))),
        },
    );
    let from = moved.obj(gun).unwrap().pos.cell();
    for _ in 0..300 {
        tick::step(&mut moved);
        if moved.obj(gun).unwrap().pos.cell() != from && glasnost_ai::scripted::idle(&moved, gun) {
            break;
        }
    }
    assert_ne!(moved.obj(gun).unwrap().pos.cell(), from, "the gun moved");
    for (id, t) in [(gun, None), (raider, Some(Target::Object(minr)))] {
        if let Some(o) = moved.obj_mut(id) {
            o.target = t;
        }
    }
    shroud::area_reveal(&mut moved, HouseId(0), Pos::centre(at(2, 2), 0), 12);
    let mut berserk = moved.clone();
    let mut again = b.clone();
    let out = again.think(&moved);
    assert_eq!(attacked(&out, gun), Some(raider), "sent again at once");
    assert_eq!(again.special_uses().economy_defence, 2);
    // A berserk unit takes no orders (S-0610 claim 6): none is given or
    // counted.
    if let Some(o) = berserk.obj_mut(gun) {
        o.combat.berserk = Some(300);
    }
    let out = b.think(&berserk);
    assert_eq!(attacked(&out, gun), None, "no order to a berserk unit");
    assert_eq!(b.special_uses().economy_defence, 1);
}
