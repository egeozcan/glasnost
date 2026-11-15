//! Claim tests for S-0215 r139 claims 23 and 24, sim issues S15 (#234,
//! Q-3101 answered): the hover-attack cell. A hover-attack type at no
//! height above the surface takes a nearby cell (S-0212 claim 25) as its
//! destination in *attack*, with or without a target, and in *guard* when
//! it had a target as the processing began. State set by hand on synthetic
//! data (the testkit's combat ruleset with an invented type); every name and
//! number is invented. Ledger: `docs/m8-claims/sim-issues.md`, *Batch S15*.

use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_sim::process;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit;
use glasnost_sim::world::{ObjectId, Target, World};

const SIZE: i32 = 20;

/// HOVA: the armed CRUSHER with the hover-attack flag.
fn rules() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    let mut t = r.sim.vehicles["CRUSHER"].clone();
    t.hover_attack = Some(true);
    r.registries
        .entry("vehicles".into())
        .or_default()
        .push("HOVA".into());
    r.sim.vehicles.insert("HOVA".into(), t);
    r
}

fn game() -> World {
    let mut w = testkit::game_with(&rules(), SIZE, 3);
    testkit::run(&mut w, 4);
    w
}

fn put(w: &mut World, name: &str, house: u8, dx: i32, dy: i32) -> ObjectId {
    testkit::spawn(w, Kind::Vehicle, name, house, testkit::mid(SIZE, dx, dy), 0)
}

/// Runs a processing of `b` for `v` now, with `target` and `dest`, nothing
/// queued: the behaviour part of its turn after the per-tick start.
fn process(
    w: &mut World,
    v: ObjectId,
    b: Behaviour,
    target: Option<ObjectId>,
    dest: Option<Target>,
) {
    let o = w.obj_mut(v).unwrap();
    o.beh.current = Some(b);
    o.beh.queued = None;
    o.beh.expired = true;
    o.target = target.map(Target::Object);
    o.dest = dest;
    let t = w.tick;
    process::tail(w, v);
    assert_eq!(
        w.obj(v).unwrap().beh.last.map(|l| l.0),
        Some(t),
        "set-up: the processing ran"
    );
}

/// The destination cell, when it is a cell other than the actor's own,
/// next to it (the first ring of the nearby search).
fn nearby_dest(w: &World, v: ObjectId) -> Option<CellXY> {
    let o = w.obj(v).unwrap();
    let own = o.pos.cell();
    match o.dest {
        Some(Target::Cell(c))
            if c != own && (c.x - own.x).abs() <= 1 && (c.y - own.y).abs() <= 1 =>
        {
            Some(c)
        }
        _ => None,
    }
}

// spec: S-0215 r139 claim 23 (the hover-attack cell: a hover-attack type at height 0 above the surface takes the nearby cell of S-0212 claim 25, centred on its own cell, as its destination, with a target; the actor's own cell does not qualify while its occupancy record holds the actor)
#[test]
fn issue_234_attack_with_a_target_takes_a_nearby_cell() {
    let mut w = game();
    let enemy = put(&mut w, "BUGGY", 1, 3, 0);
    let v = put(&mut w, "HOVA", 0, 0, 0);
    assert_eq!(w.height_above_surface(v), 0, "set-up: on the ground");
    process(&mut w, v, Behaviour::Attack, Some(enemy), None);
    assert!(
        nearby_dest(&w, v).is_some(),
        "a nearby cell is its destination: {:?}",
        w.obj(v).unwrap().dest
    );
    assert_eq!(w.obj(v).unwrap().target, Some(Target::Object(enemy)));
    // Control: the same type without the flag takes no cell.
    let plain = put(&mut w, "CRUSHER", 0, 0, 4);
    process(&mut w, plain, Behaviour::Attack, Some(enemy), None);
    assert_eq!(
        w.obj(plain).unwrap().dest,
        None,
        "no hover-attack flag: no cell"
    );
}

// spec: S-0215 r139 claim 23 (the rule applies with or without a target; it is taken before the fall-back, which does not remove it by itself)
#[test]
fn issue_234_attack_without_a_target_takes_a_nearby_cell() {
    let mut w = game();
    let v = put(&mut w, "HOVA", 0, 0, 0);
    process(&mut w, v, Behaviour::Attack, None, None);
    assert!(
        nearby_dest(&w, v).is_some(),
        "a nearby cell is its destination: {:?}",
        w.obj(v).unwrap().dest
    );
}

// spec: S-0215 r139 claim 23 (the rule applies also while the actor holds a destination, which the result then replaces)
#[test]
fn issue_234_the_cell_replaces_a_held_destination() {
    let mut w = game();
    let enemy = put(&mut w, "BUGGY", 1, 3, 0);
    let v = put(&mut w, "HOVA", 0, 0, 0);
    let far = Target::Cell(testkit::mid(SIZE, -6, -6));
    process(&mut w, v, Behaviour::Attack, Some(enemy), Some(far));
    assert!(
        nearby_dest(&w, v).is_some(),
        "the held destination is replaced: {:?}",
        w.obj(v).unwrap().dest
    );
}

// spec: S-0215 r139 claim 23 (only at a height above the surface of 0)
#[test]
fn issue_234_no_cell_above_the_surface() {
    let mut w = game();
    let enemy = put(&mut w, "BUGGY", 1, 3, 0);
    let v = put(&mut w, "HOVA", 0, 0, 0);
    w.obj_mut(v).unwrap().pos.z += 40;
    assert_eq!(w.height_above_surface(v), 40, "set-up: above the surface");
    process(&mut w, v, Behaviour::Attack, Some(enemy), None);
    assert_eq!(nearby_dest(&w, v), None, "no hover-attack cell");
}

// spec: S-0215 r139 claim 24 (Other actors, target present when the processing began: a hover-attack type at height 0 takes the hover-attack cell of claim 23 by the same search and assignment)
#[test]
fn issue_234_guard_with_a_target_takes_a_nearby_cell() {
    let mut w = game();
    let enemy = put(&mut w, "BUGGY", 1, 3, 0);
    let v = put(&mut w, "HOVA", 0, 0, 0);
    process(&mut w, v, Behaviour::Guard, Some(enemy), None);
    assert!(
        nearby_dest(&w, v).is_some(),
        "a nearby cell is its destination: {:?}",
        w.obj(v).unwrap().dest
    );
}

// spec: S-0215 r139 claim 24 (Other actors, no target when the processing began: ordinary guard processing, no hover-attack cell)
#[test]
fn issue_234_guard_without_a_target_takes_no_cell() {
    let mut w = game();
    let v = put(&mut w, "HOVA", 0, 0, 0);
    process(&mut w, v, Behaviour::Guard, None, None);
    assert_eq!(w.obj(v).unwrap().dest, None, "no hover-attack cell");
}

// spec: S-0215 r139 claim 23 (N is the processing tick; the assignment is the one an order makes, and a vehicle's unchanged destination is rejected, S-0207 claim 27); design (an unchanged destination keeps its stored route)
#[test]
fn issue_234_an_unchanged_cell_keeps_the_route() {
    let mut w = game();
    let enemy = put(&mut w, "BUGGY", 1, 3, 0);
    let v = put(&mut w, "HOVA", 0, 0, 0);
    process(&mut w, v, Behaviour::Attack, Some(enemy), None);
    let first = nearby_dest(&w, v).expect("a nearby cell");
    w.obj_mut(v).unwrap().path.steps = vec![2, 2];
    process(
        &mut w,
        v,
        Behaviour::Attack,
        Some(enemy),
        Some(Target::Cell(first)),
    );
    assert_eq!(
        nearby_dest(&w, v),
        Some(first),
        "the same cell in the same tick"
    );
    assert_eq!(
        w.obj(v).unwrap().path.steps,
        vec![2, 2],
        "the route is kept"
    );
}
