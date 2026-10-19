//! The own AI and drainers (S-0608, PLAN §4.7): an idle drainer is sent by
//! the attack click at a seen enemy building it could drain, one that pays
//! credits first, then a power plant, never at an undrainable building and
//! never two at one building; a draining drainer takes no order; the house
//! builds drainers once it has a war factory. The tests use the invented
//! types of the synthetic M7b skirmish (DISC, Alpha's drainer; Beta's
//! drainable POWR and PROC). Alpha's start discs are taken off the map
//! first, so that only the discs a test places count.

use glasnost_ai::brain::Brain;
use glasnost_ai::fixture;
use glasnost_ai::fixture_m7b as kit;
use glasnost_ai::game::Game;
use glasnost_map::CellXY;
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::world::{HouseId, ObjectId, Target, World};
use glasnost_sim::{drain, objects, testkit, tick};

fn game() -> Game {
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
    assert_eq!(discs.len(), kit::DISCS, "Alpha starts with its discs");
    for d in discs {
        objects::remove(&mut g.world, d);
    }
    while g.world.tick <= g.world.orders_blocked_through || g.world.tick % 15 != 0 {
        g.step();
    }
    g
}

fn home(w: &World) -> CellXY {
    w.objects
        .values()
        .find(|o| o.owner == Some(HouseId(0)) && o.on_map)
        .map(|o| o.pos.cell())
        .unwrap()
}

fn spawn(w: &mut World, kind: Kind, name: &str, house: u8, dx: i32, dy: i32) -> ObjectId {
    let h = home(w);
    testkit::spawn(w, kind, name, house, CellXY::new(h.x + dx, h.y + dy), 0)
}

fn brain(w: &World) -> Brain {
    Brain::new(w, &kit::map(), HouseId(0))
}

fn settle(w: &mut World) {
    testkit::run(w, 1);
    while w.tick % 15 != 0 {
        testkit::run(w, 1);
    }
}

/// The object unit `who`'s last attack order aims at.
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

/// Plays house 0's brain alone for `ticks` ticks, issuing its orders.
fn play(w: &mut World, br: &mut Brain, ticks: i32) {
    for _ in 0..ticks {
        if w.tick % 15 == 0 {
            for o in br.think(w) {
                orders::issue(w, HouseId(0), o);
            }
        }
        tick::step(w);
    }
}

#[test]
fn a_drainer_goes_for_a_paying_building_first() {
    let g = game();
    let mut w = g.world.clone();
    let d = spawn(&mut w, Kind::Vehicle, "DISC", 0, 0, 4);
    // A power plant nearer than the refinery: the refinery pays, so it
    // comes first.
    let _powr = spawn(&mut w, Kind::Building, "POWR", 1, 3, 4);
    let proc_ = spawn(&mut w, Kind::Building, "PROC", 1, 6, 6);
    settle(&mut w);
    let mut br = brain(&w);
    let out = br.think(&w);
    assert_eq!(attacked(&out, d), Some(proc_));
    assert_eq!(br.special_uses().drain_orders, 1);
}

#[test]
fn a_drainer_takes_a_power_plant_and_never_an_undrainable_building() {
    let g = game();
    let mut w = g.world.clone();
    let d = spawn(&mut w, Kind::Vehicle, "DISC", 0, 0, 4);
    // A nearer enemy war factory, not of a drainable type (S-0608 claim 3 c).
    let _fact = spawn(&mut w, Kind::Building, "FACT", 1, 3, 3);
    let powr = spawn(&mut w, Kind::Building, "POWR", 1, 7, 4);
    settle(&mut w);
    let mut br = brain(&w);
    let out = br.think(&w);
    assert_eq!(attacked(&out, d), Some(powr));
}

#[test]
fn two_drainers_do_not_go_for_one_building() {
    let g = game();
    let mut w = g.world.clone();
    let a = spawn(&mut w, Kind::Vehicle, "DISC", 0, 0, 4);
    let b = spawn(&mut w, Kind::Vehicle, "DISC", 0, 1, 4);
    let proc_ = spawn(&mut w, Kind::Building, "PROC", 1, 6, 6);
    settle(&mut w);
    let mut br = brain(&w);
    let out = br.think(&w);
    let sent: Vec<_> = [a, b]
        .into_iter()
        .filter(|&d| attacked(&out, d) == Some(proc_))
        .collect();
    assert_eq!(sent.len(), 1, "one drainer per building");
    assert_eq!(br.special_uses().drain_orders, 1);
}

#[test]
fn a_drainer_drains_and_then_takes_no_order() {
    let g = game();
    let mut w = g.world.clone();
    let d = spawn(&mut w, Kind::Vehicle, "DISC", 0, 0, 4);
    let proc_ = spawn(&mut w, Kind::Building, "PROC", 1, 6, 6);
    settle(&mut w);
    let mut br = brain(&w);
    let mut linked = false;
    for _ in 0..40 {
        play(&mut w, &mut br, 15);
        if drain::partner(&w, d) == Some(proc_) {
            linked = true;
            break;
        }
    }
    assert!(linked, "the disc linked with the refinery (S-0608 claim 2)");
    // spec: S-0608 claim 12 (a move off the foundation ends the link): the brain gives a draining drainer no order
    for _ in 0..10 {
        let out = br.think(&w);
        assert!(
            out.iter().all(|o| o.actor() != Some(d)),
            "no order to a draining drainer"
        );
        settle(&mut w);
    }
    assert_eq!(drain::partner(&w, d), Some(proc_), "the link holds");
}

#[test]
fn builds_a_drainer_with_a_war_factory() {
    let g = game();
    let mut w = g.world.clone();
    spawn(&mut w, Kind::Building, "FACT", 0, 6, -6);
    // The plan's harvesting: Alpha's slave miners.
    spawn(&mut w, Kind::Vehicle, "SMV", 0, -3, -4);
    spawn(&mut w, Kind::Vehicle, "SMV", 0, -3, -6);
    settle(&mut w);
    let mut br = brain(&w);
    let disc = w.rules().index_of(Kind::Vehicle, "DISC").unwrap();
    let mut built = false;
    for _ in 0..20 {
        let out = br.think(&w);
        built |= out.iter().any(
            |o| matches!(o, Order::Build { what } if what.kind == Kind::Vehicle && what.index == disc),
        );
        if built {
            break;
        }
        settle(&mut w);
    }
    assert!(built);
    assert!(br.special_uses().drainer_builds >= 1);
}

#[test]
fn without_a_drainer_type_nothing_changes() {
    // The M7a skirmish has no draining weapon: no drain order, no build.
    let mut g = Game::new(
        glasnost_ai::fixture_m7a::rules(),
        &glasnost_ai::fixture_m7a::map(),
        glasnost_ai::fixture_m7a::MAP_NAME,
        fixture::catalogue(),
        glasnost_ai::fixture_m7a::setup(5),
    )
    .expect("the M7a skirmish loads");
    for _ in 0..600 {
        g.step();
    }
    let u = g.special_uses();
    assert!(
        u.iter()
            .all(|(_, x)| x.drain_orders == 0 && x.drainer_builds == 0)
    );
}

#[test]
fn a_drainer_that_only_keeps_its_target_is_sent_again() {
    // A disc that keeps a refinery as its target without attacking it (no
    // attack behaviour, so it does not go there) is not on its way: the
    // brain clicks it onto the refinery again, and no later order of
    // another part replaces that click.
    let g = game();
    let mut w = g.world.clone();
    let d = spawn(&mut w, Kind::Vehicle, "DISC", 0, 0, 4);
    let proc_ = spawn(&mut w, Kind::Building, "PROC", 1, 6, 6);
    settle(&mut w);
    w.obj_mut(d).unwrap().target = Some(Target::Object(proc_));
    assert_ne!(w.obj(d).unwrap().beh.current, Some(Behaviour::Attack));
    let mut br = brain(&w);
    let out = br.think(&w);
    assert_eq!(attacked(&out, d), Some(proc_));
    let last = out.iter().rev().find(|o| o.actor() == Some(d));
    assert!(
        matches!(
            last,
            Some(Order::Behaviour {
                behaviour: Behaviour::Attack,
                ..
            })
        ),
        "the drain click is the disc's last order"
    );
}
