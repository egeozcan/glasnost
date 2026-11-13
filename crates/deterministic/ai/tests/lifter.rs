//! The own AI and magnetic lifters (S-0607, PLAN §4.7): a lifter is sent
//! by the attack click at the dearest enemy vehicle it can seize near home
//! (never at infantry); a holding lifter is let go over an enemy building,
//! which kills the held one, and takes no other order meanwhile; the house
//! builds lifters once it has a war factory. The tests use the
//! invented types of the synthetic M7a skirmish (MAGN, the lifter both
//! sides may build).

use glasnost_ai::brain::Brain;
use glasnost_ai::fixture;
use glasnost_ai::fixture_m7a as kit;
use glasnost_ai::game::Game;
use glasnost_map::CellXY;
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::world::{HouseId, ObjectId, Target, World};
use glasnost_sim::{lift, testkit, tick};

fn game() -> Game {
    let mut g = Game::new(
        kit::rules(),
        &kit::map(),
        kit::MAP_NAME,
        fixture::catalogue(),
        kit::setup(5),
    )
    .expect("the M7a skirmish loads");
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

#[test]
fn a_lifter_goes_for_the_dearest_enemy_vehicle_never_infantry() {
    let g = game();
    let mut w = g.world.clone();
    let l = spawn(&mut w, Kind::Vehicle, "MAGN", 0, 0, 4);
    spawn(&mut w, Kind::Infantry, "MADMAN", 1, 2, 4);
    let cheap = spawn(&mut w, Kind::Vehicle, "TRUK", 1, 4, 4);
    let dear = spawn(&mut w, Kind::Vehicle, "GUNR", 1, 5, 6);
    settle(&mut w);
    let mut br = brain(&w);
    let out = br.think(&w);
    let price = |id: ObjectId| {
        let o = w.obj(id).unwrap();
        glasnost_sim::econ::production::cost_field(&w, glasnost_sim::econ::tech::type_of(o))
    };
    let want = if price(dear) >= price(cheap) {
        dear
    } else {
        cheap
    };
    assert_eq!(attacked(&out, l), Some(want));
    assert_eq!(br.special_uses().lift_orders, 1);
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
fn a_holding_lifter_lets_go_over_an_enemy_building() {
    let g = game();
    let mut w = g.world.clone();
    let l = spawn(&mut w, Kind::Vehicle, "MAGN", 0, 0, 4);
    let v = spawn(&mut w, Kind::Vehicle, "GUNR", 1, 7, 4);
    // An enemy building under the pull's path, three cells from the lifter.
    let under = spawn(&mut w, Kind::Building, "POWR", 1, 3, 3);
    settle(&mut w);
    let mut br = brain(&w);
    let mut held = false;
    for _ in 0..40 {
        play(&mut w, &mut br, 15);
        held |= lift::grips(&w, l, v);
        if w.obj(v).is_none() {
            break;
        }
    }
    assert!(held, "the lifter seized the vehicle");
    assert!(w.obj(v).is_none(), "the held vehicle fell to its death");
    assert!(br.special_uses().lift_drops >= 1, "let go by a move order");
    assert!(
        w.obj(l).is_some_and(|o| o.health == 400),
        "the lifter is unhurt"
    );
    let _ = under;
}

// design: own AI (#24) and S-0607 claims 13 and 16 (a vehicle let go in the air drops where it is, and the infantry under it are removed): a held vehicle near its lifter is not let go over the house's own infantry.
#[test]
fn a_held_vehicle_is_not_let_go_over_own_infantry() {
    let g = game();
    let mut w = g.world.clone();
    let l = spawn(&mut w, Kind::Vehicle, "MAGN", 0, 0, 4);
    let v = spawn(&mut w, Kind::Vehicle, "GUNR", 1, 7, 4);
    settle(&mut w);
    let mut br = brain(&w);
    for o in br.think(&w) {
        if o.actor() == Some(l) {
            orders::issue(&mut w, HouseId(0), o);
        }
    }
    let close = |w: &World| {
        let (Some(lo), Some(vo)) = (w.obj(l), w.obj(v)) else {
            return false;
        };
        let (dx, dy) = (
            i64::from(vo.pos.x - lo.pos.x),
            i64::from(vo.pos.y - lo.pos.y),
        );
        dx * dx + dy * dy <= glasnost_ai::lifter::LET_GO * glasnost_ai::lifter::LET_GO
    };
    // The sim alone until a think finds the held vehicle near the lifter.
    let mut found = false;
    for _ in 0..1500 {
        tick::step(&mut w);
        if br.thinks_at(w.tick) && lift::grips(&w, l, v) && close(&w) {
            found = true;
            break;
        }
    }
    assert!(found, "the lifter pulled the vehicle near");
    let under = w.obj(v).unwrap().pos.cell();
    let own = testkit::spawn(&mut w, Kind::Infantry, "WALKER", 0, under, 0);
    assert_eq!(w.obj(own).map(|o| o.pos.cell()), Some(under));
    let at = w.obj(l).unwrap().pos.cell();
    let out = br.think(&w);
    assert!(
        !out.iter().any(|o| matches!(o,
            Order::Behaviour { actor, behaviour: Behaviour::Move, dest: Some(Target::Cell(c)), .. }
                if *actor == l && *c == at)),
        "let go over own infantry: {out:?}"
    );
}

// design: own AI (#13): a lifter is no wave unit: an idle lifter at home is not sent with an attack wave, and does not count toward the wave's size (it drove to the enemy base with every wave, its attack-move passed the lifter filter).
#[test]
fn a_lifter_does_not_go_with_a_wave() {
    let g = game();
    let mut w = g.world.clone();
    // Only the units this test places: house 0's starting units go (its
    // first object, which `home` reads, stays).
    let first = w
        .objects
        .values()
        .find(|o| o.owner == Some(HouseId(0)) && o.on_map)
        .map(|o| o.id);
    let units: Vec<ObjectId> = w
        .objects
        .values()
        .filter(|o| {
            o.owner == Some(HouseId(0))
                && o.on_map
                && o.kind != Kind::Building
                && Some(o.id) != first
        })
        .map(|o| o.id)
        .collect();
    for id in units {
        glasnost_sim::objects::remove(&mut w, id);
    }
    let guards: Vec<ObjectId> = [(1, 1), (2, 1)]
        .into_iter()
        .map(|(dx, dy)| spawn(&mut w, Kind::Vehicle, "GUNR", 0, dx, dy))
        .collect();
    let l = spawn(&mut w, Kind::Vehicle, "MAGN", 0, 0, 6);
    let tank = spawn(&mut w, Kind::Vehicle, "GUNR", 0, 0, 8);
    settle(&mut w);
    let period = glasnost_ai::scripted::THINK_PERIOD;
    let opening = glasnost_ai::scripted::OPENING;
    w.tick = (w.tick.max(opening) + period - 1) / period * period;
    let mut plan = glasnost_ai::brain::default_plan();
    plan.wave = 2;
    let mut br = Brain::with_plan(&w, &kit::map(), HouseId(0), plan);
    let out = br.think(&w);
    let sent = |who: ObjectId| {
        out.iter().any(|o| {
            matches!(o, Order::Behaviour {
                actor,
                attack_move: true,
                ..
            } if *actor == who)
        })
    };
    assert!(!sent(l), "the lifter goes with the wave: {out:?}");
    assert!(!sent(tank), "the lifter made up the wave of two: {out:?}");
    assert!(!guards.iter().any(|&g| sent(g)), "the guard stays");
}

#[test]
fn builds_a_lifter_with_a_war_factory() {
    let g = game();
    let mut w = g.world.clone();
    spawn(&mut w, Kind::Building, "FACT", 0, 6, -6);
    // The plan's two harvesters, kept before any lifter is ordered.
    spawn(&mut w, Kind::Vehicle, "MINR", 0, -3, -4);
    spawn(&mut w, Kind::Vehicle, "MINR", 0, -3, -6);
    settle(&mut w);
    let mut br = brain(&w);
    let magn = w.rules().index_of(Kind::Vehicle, "MAGN").unwrap();
    let mut built = false;
    for _ in 0..20 {
        let out = br.think(&w);
        built |= out.iter().any(
            |o| matches!(o, Order::Build { what } if what.kind == Kind::Vehicle && what.index == magn),
        );
        if built {
            break;
        }
        settle(&mut w);
    }
    assert!(built);
    assert!(br.special_uses().lifter_builds >= 1);
}
