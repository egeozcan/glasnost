//! The move destination on a high-bridge cell (S-0207 Terms, claim 10 and
//! F5, claim 15 and F7; sim issue #183): braking and the give-up test
//! measure to the deck's height there. Synthetic data: column x = 0 from
//! y = 1 down to y = −6; with a bridge, land at level 4 at y = 1, 0, −5,
//! −6 and a one-cell-wide high-bridge deck over level-0 ground at
//! y = −1 … −4; without, flat ground at level 0 (the controls).

mod common;

use common::*;
use glasnost_map::CellXY;
use glasnost_num::F64;
use glasnost_rules::sim::UnitType;
use glasnost_sim::rules::Kind;
use glasnost_sim::tick;
use glasnost_sim::world::{Layer, ObjectId, World};

const N: u8 = 0;
/// F0.3: single-precision 0.3 widened to double.
// spec: S-0207 Constants (braking floor 0.300000011920928955078125)
const F03: F64 = F64(0x3fd3_3333_4000_0000);

fn car(accel: bool) -> UnitType {
    let mut t = unit_type(WHEEL, 30, 5);
    t.accelerates = Some(accel);
    t
}

fn scene(bridge: bool) -> Scene {
    let mut s = Scene::new();
    // spec: S-0211 claim 5 (terrain code 15: rock)
    let rock = code_tile(15);
    for y in -7..=-1 {
        for x in [-1, 1] {
            s.set_cell(cell(x, y), rock, 0);
        }
    }
    if bridge {
        for y in [0, 1, -5, -6] {
            s.set_cell(cell(0, y), 0, 4);
        }
    } else {
        s.set_cell(cell(0, -5), rock, 0);
    }
    s
}

fn lay_deck(w: &mut World) {
    for y in -4..=-1 {
        let i = w.cells.idx(cell(0, y)).unwrap();
        w.cells.deck[i] = true;
        w.cells.deck_entry[i] = true;
        w.cells.bridge_axis[i] = glasnost_sim::cells::Axis::Y;
    }
    w.regions.dirty = true;
}

/// Orders `id` to `to` along `steps` and runs until it rests there.
fn drive_to(w: &mut World, id: ObjectId, to: CellXY, steps: &[u8]) {
    order_move(w, id, to);
    tick::step(w);
    w.obj_mut(id).expect("v").path.steps = steps.to_vec();
    for _ in 0..600 {
        tick::step(w);
        let o = w.obj(id).expect("v");
        if o.pos.cell() == to && o.dest.is_none() && o.drive.as_ref().unwrap().track.is_none() {
            return;
        }
    }
    let o = w.obj(id).expect("v");
    panic!("did not arrive: at {:?}, dest {:?}", o.pos, o.dest);
}

/// The braking signature of claim 10 case 2 on every tick that starts on
/// the last track less than 250 leptons (slowdown distance 300) from the
/// destination cell's centre: the fraction falls, or sits at the floor.
/// Returns (ticks checked, ticks with the signature, z seen).
fn braking(bridge: bool) -> (usize, usize, i32) {
    let mut s = scene(bridge);
    let mut t = car(true);
    t.slowdown_distance = Some(300);
    s.vehicle("TESTCAR", t);
    s.place_unit(0, "TESTCAR", cell(0, 0), 0);
    let mut w = s.build();
    if bridge {
        lay_deck(&mut w);
    }
    let id = objects_of(&w, Kind::Vehicle)[0];
    open_orders(&mut w);
    let dest = cell(0, -4);
    order_move(&mut w, id, dest);
    tick::step(&mut w);
    w.obj_mut(id).expect("v").path.steps = vec![N; 4];
    let (cx, cy) = (256 * dest.x + 128, 256 * dest.y + 128);
    let (mut checked, mut braked, mut z) = (0, 0, 0);
    for _ in 0..600 {
        let before = w.obj(id).expect("v").clone();
        tick::step(&mut w);
        let after = w.obj(id).expect("v");
        let (dx, dy) = (i64::from(cx - before.pos.x), i64::from(cy - before.pos.y));
        let near = dx * dx + dy * dy < 250 * 250;
        let (db, da) = (
            before.drive.as_ref().unwrap(),
            after.drive.as_ref().unwrap(),
        );
        if near && db.track.is_some() && da.track.is_some() {
            checked += 1;
            z = before.pos.z;
            if da.rho.lt(db.rho) || da.rho == F03 {
                braked += 1;
            }
        }
        if after.pos.cell() == dest && da.track.is_none() {
            break;
        }
    }
    (checked, braked, z)
}

// spec: S-0207 claim 10 case 2 and F5 (braking when dₘ < Dₛ)
#[test]
fn braking_on_flat_ground_control() {
    let (checked, braked, z) = braking(false);
    assert!(checked > 0);
    assert_eq!(z, 0);
    assert_eq!(
        braked, checked,
        "every tick within the slowdown distance brakes"
    );
}

// spec: S-0207 claim 10 case 2 and F5, Terms (Move destination: on a high-bridge cell its height is the deck's height)
#[test]
fn braking_on_a_bridge_deck() {
    let (checked, braked, z) = braking(true);
    assert!(checked > 0);
    assert_eq!(z, 416, "the vehicle is on the deck (claim 5)");
    assert_eq!(
        braked, checked,
        "every tick within the slowdown distance brakes"
    );
}

/// A friend rests at (0, −3), the mover at (0, −2) is ordered to (0, −4).
/// Returns (the mover gave up, the friend got a destination, mover z).
fn friend(bridge: bool) -> (bool, bool, i32) {
    let mut s = scene(bridge);
    s.vehicle("MOVER", car(false)).vehicle("IDLER", car(false));
    s.place_unit(0, "IDLER", cell(0, 0), 0);
    s.place_unit(0, "MOVER", cell(0, 1), 0);
    let mut w = s.build();
    if bridge {
        lay_deck(&mut w);
    }
    let b = object_in(&w, cell(0, 0)).expect("idler");
    let m = object_in(&w, cell(0, 1)).expect("mover");
    open_orders(&mut w);
    drive_to(&mut w, b, cell(0, -3), &[N; 3]);
    drive_to(&mut w, m, cell(0, -2), &[N; 3]);
    let layer = if bridge { Layer::Deck } else { Layer::Ground };
    assert_eq!(w.obj(b).unwrap().layer, layer);
    assert_eq!(w.obj(m).unwrap().layer, layer);
    order_move(&mut w, m, cell(0, -4));
    let mz = w.obj(m).unwrap().pos.z;
    let b0 = w.obj(b).map(|o| (o.dest, o.target));
    for _ in 0..60 {
        tick::step(&mut w);
        let gave_up = w.obj(m).expect("m").dest.is_none();
        let asked = w.obj(b).map(|o| (o.dest, o.target)) != b0;
        if gave_up || asked {
            return (gave_up, asked, mz);
        }
    }
    panic!("no reaction (mover z {mz})");
}

// spec: S-0207 claim 15 table T4 and F7 (within the close-enough distance the repeated check gives up the move; no request)
#[test]
fn give_up_on_flat_ground_control() {
    let (gave_up, asked, mz) = friend(false);
    assert_eq!(mz, 0);
    assert!(gave_up && !asked, "the move is given up, no request");
}

// spec: S-0207 claim 15 table T4 and F7, Terms (Move destination: on a high-bridge cell its height is the deck's height)
#[test]
fn give_up_on_a_bridge_deck() {
    let (gave_up, asked, mz) = friend(true);
    assert_eq!(mz, 416, "the mover is on the deck (claim 5)");
    assert!(
        gave_up && !asked,
        "the move is given up, no request (gave up {gave_up}, request sent {asked})"
    );
}
