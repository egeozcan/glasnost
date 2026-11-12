//! The own AI's home (#12): the base the house defends, gathers at and
//! builds around stays where its construction yard stood. It does not jump
//! into the enemy base when the house captures an enemy yard, nor to the
//! house's oldest unit when its own yard is lost.

use glasnost_ai::brain::Brain;
use glasnost_ai::{fixture, game::Game};
use glasnost_map::CellXY;
use glasnost_sim::econ::{TypeRef, shroud};
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, Pos, Target, World};

fn named(w: &World, h: u8, n: &str) -> Vec<ObjectId> {
    let Some(i) = w.rules().index_of(Kind::Building, n) else {
        return Vec::new();
    };
    w.objects
        .values()
        .filter(|o| {
            o.owner == Some(HouseId(h)) && o.on_map && o.kind == Kind::Building && o.type_index == i
        })
        .map(|o| o.id)
        .collect()
}

fn d2(a: CellXY, b: CellXY) -> i64 {
    i64::from(a.x - b.x).pow(2) + i64::from(a.y - b.y).pow(2)
}

/// The synthetic skirmish once both yards and house 0's power plant stand.
fn base_up() -> World {
    let mut g = Game::new(
        fixture::rules(),
        &fixture::map(),
        "synthetic-skirmish",
        fixture::catalogue(),
        fixture::setup(5),
    )
    .expect("the skirmish loads");
    while g.world.tick < 4000
        && (named(&g.world, 0, "YARD").is_empty()
            || named(&g.world, 1, "YARD").is_empty()
            || named(&g.world, 0, "POWR").is_empty())
    {
        g.step();
    }
    assert!(!named(&g.world, 0, "POWR").is_empty(), "the base stands");
    g.world.clone()
}

/// Runs `w` to the next tick `brain` thinks at.
fn to_think(w: &mut World, brain: &Brain) {
    testkit::run(w, 1);
    while !brain.thinks_at(w.tick) {
        testkit::run(w, 1);
    }
}

// design: own AI (#12): a captured enemy yard with a lower object number than the house's own does not move home into the enemy base: the house's next power plant goes up by its own yard.
#[test]
fn a_captured_yard_does_not_move_home() {
    let mut w = base_up();
    let own = named(&w, 0, "YARD")[0];
    let at = w.obj(own).unwrap().pos.cell();
    // The house's yard rebuilt on its cell, so that the enemy's yard has
    // the lower object number.
    glasnost_sim::objects::remove(&mut w, own);
    let own = testkit::spawn(&mut w, Kind::Building, "YARD", 0, at, 0);
    let theirs = named(&w, 1, "YARD")[0];
    assert!(theirs < own);
    let there = w.obj(theirs).unwrap().pos.cell();
    testkit::run(&mut w, 1);
    let mut brain = Brain::with_plan(&w, &fixture::map(), HouseId(0), fixture::plan());
    to_think(&mut w, &brain);
    brain.think(&w);
    glasnost_sim::mind::set_owner(&mut w, theirs, Some(HouseId(0)));
    shroud::area_reveal(&mut w, HouseId(0), Pos::centre(there, 0), 8);
    w.houses[0].credits = 100_000;
    // spec: S-0404 claim 6 (the cancel-all order empties the line of the type)
    let lined: Vec<TypeRef> = w
        .econ
        .lines
        .lines
        .iter()
        .filter(|l| l.house == HouseId(0))
        .filter_map(|l| l.item.as_ref().map(|i| i.t))
        .filter(|t| t.kind == Kind::Building)
        .collect();
    for t in lined {
        orders::issue(&mut w, HouseId(0), Order::CancelAll { what: t });
    }
    let powr = TypeRef {
        kind: Kind::Building,
        index: w.rules().index_of(Kind::Building, "POWR").expect("POWR"),
    };
    orders::issue(&mut w, HouseId(0), Order::Build { what: powr });
    let end = w.tick + 3000;
    let mut placed = None;
    while w.tick < end && placed.is_none() {
        for o in brain.think(&w) {
            if let Order::Place {
                what,
                cell: Some(c),
            } = &o
                && *what == powr
            {
                placed = Some(*c);
            }
            orders::issue(&mut w, HouseId(0), o);
        }
        testkit::run(&mut w, 1);
    }
    let c = placed.expect("the power plant is placed");
    assert!(
        d2(c, at) < d2(c, there),
        "placed at {c:?}, by the captured yard at {there:?} (own yard at {at:?})"
    );
}

// design: own AI (#12): with its yard lost, the house's home is not its oldest unit far away: a defender at the base still answers an intruder there.
#[test]
fn a_lost_yard_does_not_move_home_to_a_far_unit() {
    let mut w = base_up();
    let yard = named(&w, 0, "YARD")[0];
    let at = w.obj(yard).unwrap().pos.cell();
    let mut brain = Brain::with_plan(&w, &fixture::map(), HouseId(0), fixture::plan());
    to_think(&mut w, &brain);
    brain.think(&w);
    // A tank by the enemy base, then the house's base gone but for a power
    // plant (its yard lost): the tank is the house's oldest object.
    let far = testkit::spawn(
        &mut w,
        Kind::Vehicle,
        "GUNR",
        0,
        fixture::at(fixture::START_DX - 8, 6),
        0,
    );
    let old: Vec<ObjectId> = w
        .objects
        .values()
        .filter(|o| o.owner == Some(HouseId(0)) && o.on_map && o.id != far)
        .map(|o| o.id)
        .collect();
    for id in old {
        glasnost_sim::objects::remove(&mut w, id);
    }
    let plant = CellXY::new(at.x + 2, at.y);
    testkit::spawn(&mut w, Kind::Building, "POWR", 0, plant, 0);
    let defender = testkit::spawn(
        &mut w,
        Kind::Vehicle,
        "GUNR",
        0,
        CellXY::new(plant.x - 2, plant.y),
        0,
    );
    let raid = CellXY::new(plant.x + 11, plant.y);
    let intruder = testkit::spawn(&mut w, Kind::Vehicle, "GUNR", 1, raid, 0);
    shroud::area_reveal(&mut w, HouseId(0), Pos::centre(raid, 0), 3);
    assert!(d2(at, w.obj(far).unwrap().pos.cell()) > 30 * 30);
    to_think(&mut w, &brain);
    shroud::area_reveal(&mut w, HouseId(0), Pos::centre(raid, 0), 3);
    let orders = brain.think(&w);
    let answers = orders.iter().any(|o| {
        matches!(o, Order::Behaviour {
            actor,
            behaviour: Behaviour::Attack,
            target: Some(Target::Object(e)),
            ..
        } if *actor == defender && *e == intruder)
    });
    assert!(
        answers,
        "the defender does not answer the intruder: {orders:?}"
    );
}
