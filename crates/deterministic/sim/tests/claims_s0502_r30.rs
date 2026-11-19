//! Claim tests of S-0502 r30 claim 18 (Q-3255, #446): the stop at a
//! jumpjet's crash start (the same stop as S-0607 claim 13's stop on
//! dropping: two empty destination assignments with a direct stop between
//! them, the target cleared and contact broken off) and the path of the
//! fall (scenario J12's balloon-hover subcases, J31 part (a)), with S-0312
//! r93 claim 16. Synthetic rules (`jump_common`, plus BALCRASH, the
//! balloon-hover vehicle with the crash flag); every name and number is
//! invented.

mod jump_common;

use glasnost_sim::combat::damage::{self, Hit};
use glasnost_sim::jump::{self, JumpState, Phase};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit;
use glasnost_sim::world::{ObjectId, Pos, Target, World};
use jump_common::*;

fn j(w: &World, id: ObjectId) -> &JumpState {
    w.obj(id).unwrap().jump.as_ref().unwrap()
}

fn kill(w: &mut World, id: ObjectId) {
    let health = w.obj(id).unwrap().health;
    damage::apply(
        w,
        id,
        Hit {
            amount: health * 10,
            warhead: None,
            distance: 0,
            attacker: None,
            house: None,
            ignore_defences: true,
            no_escape: false,
        },
    );
}

/// A game with BALCRASH: BALLOON (balloon-hover, mobile fire) with the
/// crash flag.
fn balloon_game() -> World {
    let mut r = ruleset();
    let mut t = r.sim.vehicles["BALLOON"].clone();
    t.crashable = Some(true);
    r.sim.vehicles.insert("BALCRASH".into(), t);
    r.registries
        .get_mut("vehicles")
        .unwrap()
        .push("BALCRASH".into());
    let mut w = testkit::game_with(&r, SIZE, 7);
    testkit::run(&mut w, 4);
    w
}

/// A BALCRASH at (0, −4) given `dest` and run until it cruises at its full
/// speed 14, 400 leptons or more above the ground.
fn cruising(w: &mut World, dest: Target) -> ObjectId {
    let u = put(w, "BALCRASH", Kind::Vehicle, 0, 0, -4);
    let t = w.tick;
    w.obj_mut(u).unwrap().beh.force(Behaviour::Move, t, true);
    glasnost_sim::econ::assign_destination(w, u, Some(dest));
    assert!(
        until(w, 400, |w| phase(w, u) == Phase::Cruising
            && j(w, u).vc.to_i32_common() == 14
            && height(w, u) >= 400),
        "it takes off and cruises at full speed"
    );
    u
}

// spec: S-0502 claim 18 (r30, Q-3255, scenario J12 balloon-hover subcase (c): a balloon-hover vehicle that holds a retained destination and a target at the hit rejects the crash start's first empty assignment under claim 28, with that claim's effect on its behaviour, attack current; its target is then cleared, the direct stop runs claim 17 whatever the first assignment did and reroutes it to the centre of the cell the nearby search around its current cell finds, its own or a neighbour, and the second assignment, made with no target, cannot be rejected: its order-level destination is empty) and S-0312 claim 16 (r93)
#[test]
fn j12_c_a_retained_destination_ends_empty_and_the_point_is_rerouted() {
    let mut w = balloon_game();
    let x = put(&mut w, "CART", Kind::Vehicle, 1, 18, -4);
    let u = cruising(&mut w, Target::Object(x));
    w.obj_mut(u).unwrap().target = Some(Target::Object(x));
    let o = w.obj(u).unwrap();
    assert_eq!(
        (o.dest, o.target),
        (Some(Target::Object(x)), Some(Target::Object(x))),
        "void unless it holds the destination and the target at the hit (claim 28 row 1)"
    );
    let here = pos(&w, u).cell();
    kill(&mut w, u);
    assert!(jump::crashing(&w, u), "it crashes");
    let o = w.obj(u).unwrap();
    assert_eq!(o.dest, None, "the order-level destination is empty");
    assert_eq!(o.target, None, "its target is cleared");
    assert_eq!(
        o.beh.seen(),
        Some(Behaviour::Attack),
        "the first assignment's rejection made attack current"
    );
    let p = j(&w, u).dest.expect("the commitment survives");
    let (dx, dy) = (p.cell().x - here.x, p.cell().y - here.y);
    assert!(
        dx.abs() <= 1 && dy.abs() <= 1,
        "the point moved to its own cell's centre or a neighbour's: {dx}, {dy}"
    );
    assert_eq!(
        (p.x, p.y),
        (Pos::centre(p.cell(), 0).x, Pos::centre(p.cell(), 0).y),
        "a vehicle's point is the cell's centre"
    );
}

/// A JJCRASH (a crashable jumpjet infantry type) at (0, −4) cruising at
/// 500 towards (10, 0); `shared`: another infantry unit with no order
/// stands in its cell. Returns the unit.
fn infantry_cruising(w: &mut World, shared: bool) -> ObjectId {
    let i = put(w, "JJCRASH", Kind::Infantry, 0, 0, -4);
    if shared {
        put(w, "JJ", Kind::Infantry, 0, 0, -4);
    }
    go(w, i, at(10, 0));
    jump::start_in_flight(w, i, Phase::Cruising, 500);
    i
}

// spec: S-0502 claim 18 (r30, Q-3255, scenario J31 part (a): a jumpjet-type infantry unit with a movement commitment has no equality suppression, so the first assignment, the direct stop and the second assignment all reroute it; each reroute adjusts its standing position afresh in the cell the nearby search finds, the same cell each time, so it takes three of its ranged draws over 0 to 3 in an eligible cell and no tumbling draw; the point lies in another cell when the search rejects the current one, as for another unit's mark there) and S-0312 claim 16 (r93: the three draws for infantry with a commitment)
#[test]
fn j31_a_infantry_takes_three_reroute_draws_at_the_crash_start() {
    for shared in [false, true] {
        let mut w = game();
        let i = infantry_cruising(&mut w, shared);
        let here = pos(&w, i).cell();
        let before = w.rng.taken();
        kill(&mut w, i);
        assert!(jump::crashing(&w, i), "shared {shared}: it crashes");
        assert_eq!(
            w.rng.taken(),
            before + 3,
            "shared {shared}: three reroute draws"
        );
        assert_eq!(w.obj(i).unwrap().dest, None, "shared {shared}");
        let p = j(&w, i).dest.expect("the commitment survives");
        assert_ne!(p.cell(), at(10, 0), "shared {shared}: rerouted");
        let (dx, dy) = (p.cell().x - here.x, p.cell().y - here.y);
        assert!(dx.abs() <= 1 && dy.abs() <= 1, "shared {shared}: nearby");
        if shared {
            assert_ne!(p.cell(), here, "another unit's mark rejects its own cell");
        }
    }
}

// spec: S-0502 claim 18 (r30: the crash start's stop is S-0607 claim 13's stop on dropping: it breaks off any contact, on both sides when the other is not a building)
#[test]
fn claim18_r30_the_crash_start_breaks_off_contact() {
    let mut w = game();
    let i = infantry_cruising(&mut w, false);
    let other = put(&mut w, "JJ", Kind::Infantry, 0, 3, -4);
    w.obj_mut(i).unwrap().contact = Some(other);
    w.obj_mut(other).unwrap().contact = Some(i);
    kill(&mut w, i);
    assert!(jump::crashing(&w, i));
    assert_eq!(w.obj(i).unwrap().contact, None, "its side");
    assert_eq!(w.obj(other).unwrap().contact, None, "the other side");
}

/// One crashing tick: position, current and maximum speed, the order-level
/// destination, movement heading and body facing.
struct Tick {
    pos: Pos,
    vc: i32,
    vm: i32,
    order: Option<Target>,
    heading: u16,
    body: u16,
}

/// Kills `u` and traces it from the hit to its crash end.
fn fall(w: &mut World, u: ObjectId) -> Vec<Tick> {
    kill(w, u);
    let mut log = Vec::new();
    for k in 0..200 {
        if k > 0 {
            step(w);
        }
        let Some(o) = w.obj(u) else { break };
        let s = o.jump.as_ref().unwrap();
        log.push(Tick {
            pos: o.pos,
            vc: s.vc.to_i32_common(),
            vm: s.vm.to_i32_common(),
            order: o.dest,
            heading: s.heading.value(w.tick),
            body: o.body.value(w.tick),
        });
    }
    assert!(w.obj(u).is_none(), "it reaches its crash end");
    log
}

// spec: S-0502 claim 18 (r30, Q-3255, *Path of the fall*, as built: every crashing jumpjet falls this way, balloon-hover type or not; J12's balloon-hover subcases: the order-level destination is empty from the hit on; the body facing equals the heading after every crashing tick, claim 10; (a) one killed while cruising steps along its spinning heading, and a fall shorter than a full turn leaves an open arc, not a crash end back at the kill position; (b) one killed hovering with no order keeps current and maximum speed 0 and falls straight down) and S-0312 claim 16 (r93)
#[test]
fn j12_r30_the_path_of_the_fall_as_built() {
    // (a): cruising at full speed towards a cell 20 cells away.
    let mut w = balloon_game();
    let u = cruising(&mut w, Target::Cell(at(20, -4)));
    let log = fall(&mut w, u);
    assert!(log.iter().all(|t| t.order.is_none()), "(a) no destination");
    assert!(
        log[1..].windows(2).all(|p| p[1].body == p[0].heading),
        "(a) the body facing follows the heading the step used"
    );
    let (first, last) = (&log[0], log.last().unwrap());
    assert_ne!(
        (first.pos.x, first.pos.y),
        (last.pos.x, last.pos.y),
        "(a) an open arc"
    );
    assert!(
        log.windows(2)
            .all(|p| p[1].vc > 0 || (p[1].pos.x, p[1].pos.y) == (p[0].pos.x, p[0].pos.y)),
        "(a) a sideways step only at a current speed above 0"
    );
    // (b): hovering with no order.
    let mut w = balloon_game();
    let u = put(&mut w, "BALCRASH", Kind::Vehicle, 0, 0, -4);
    jump::start_in_flight(&mut w, u, Phase::Hovering, 500);
    let log = fall(&mut w, u);
    assert!(log.iter().all(|t| t.order.is_none()), "(b) no destination");
    assert!(
        log.iter().all(|t| t.vc == 0 && t.vm == 0),
        "(b) speeds 0 throughout"
    );
    assert!(
        log.iter()
            .all(|t| (t.pos.x, t.pos.y) == (log[0].pos.x, log[0].pos.y)),
        "(b) straight down"
    );
    assert!(
        log[1..].windows(2).all(|p| p[1].body == p[0].heading),
        "(b) the body facing follows the heading the step used"
    );
    assert_ne!(log[0].heading, log.last().unwrap().heading, "(b) it spins");
}

// spec: S-0502 claim 18 (r30: a unit without a movement commitment: the stop changes neither its movement point nor its flight state; its order-level destination ends empty) and claim 17 (stopping a jumpjet that is not moving does nothing)
#[test]
fn claim18_r30_without_a_commitment_the_stop_changes_no_point() {
    let mut w = balloon_game();
    let u = put(&mut w, "BALCRASH", Kind::Vehicle, 0, 0, -4);
    jump::start_in_flight(&mut w, u, Phase::Hovering, 500);
    assert!(!jump::moving(&w, u), "no commitment");
    let point = j(&w, u).dest;
    kill(&mut w, u);
    assert!(jump::crashing(&w, u));
    assert_eq!(j(&w, u).dest, point, "the point is unchanged");
    assert_eq!(
        phase(&w, u),
        Phase::Hovering,
        "the flight state is unchanged"
    );
    assert_eq!(w.obj(u).unwrap().dest, None);
}
