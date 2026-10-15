//! Claim tests of S-0516 claims 13 and 22 as revised for Q-0795 (revision
//! 8): which movement and relocation supplies a cell-arrival update (sensor
//! disc relocation and the adjacent sensor test) outside the walking,
//! driving, ship and hovering locomotors. Synthetic rules only (see
//! `cloak_common`).

mod cloak_common;

use cloak_common::*;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::Locomotor;
use glasnost_sim::cloak::CloakState;
use glasnost_sim::world::{ObjectId, World};

/// The x offset of the centre of house `h`'s sensor disc on row 0 (the
/// disc is symmetric).
fn disc_x(w: &World, h: glasnost_sim::world::HouseId) -> i32 {
    let o = at(0, 0);
    let xs: Vec<i32> = w
        .cloak
        .sensors
        .iter()
        .filter(|((hh, c), n)| *hh == h.0 && c.y == o.y && **n > 0)
        .map(|((_, c), _)| c.x - o.x)
        .collect();
    (xs.iter().min().unwrap() + xs.iter().max().unwrap()) / 2
}

/// The cloak ruleset with sensor types: JUMPER (teleporting), JUMPERC (the
/// same, with the cloak flag), STATIC (a type with no movement in the
/// engine: a jumpjet), SEEKGHOST (a driving sensor with the cloak flag).
fn rules() -> Ruleset {
    let mut r = ruleset();
    let seeker = r.sim.vehicles["SEEKER"].clone();
    let mut jumper = seeker.clone();
    jumper.locomotor = Some(Locomotor::Teleport);
    let mut jumperc = jumper.clone();
    jumperc.combat.cloakable = Some(true);
    jumperc.combat.cloaking_speed = Some(SPEED);
    let mut statik = seeker.clone();
    statik.locomotor = Some(Locomotor::Jumpjet);
    let mut seekghost = seeker;
    seekghost.combat.cloakable = Some(true);
    seekghost.combat.cloaking_speed = Some(SPEED);
    for (n, u) in [
        ("JUMPER", jumper),
        ("JUMPERC", jumperc),
        ("STATIC", statik),
        ("SEEKGHOST", seekghost),
    ] {
        r.sim.vehicles.insert(n.into(), u);
        r.registries.get_mut("vehicles").unwrap().push(n.into());
    }
    r
}

/// Runs until the object's cell changes (at most 40 ticks); returns the
/// disc's x offset before that tick and in it.
fn until_cell_changes(w: &mut World, id: ObjectId) -> (i32, i32) {
    let c0 = w.obj(id).unwrap().pos.cell();
    for _ in 0..40 {
        let before = disc_x(w, ALPHA);
        run(w, 1);
        if w.obj(id).unwrap().pos.cell() != c0 {
            return (before, disc_x(w, ALPHA));
        }
        assert_eq!(disc_x(w, ALPHA), before, "the disc stays until the arrival");
    }
    panic!("the object never changed cell");
}

// spec: S-0516 claims 13 and 22 (Q-0795: a teleporting mover's cell-arrival update comes at the destination in the jump tick, not at the cloak update after it: the disc is at the landing cell in the tick the position changes; nothing for the cells between)
#[test]
fn claim_22_teleport_disc_moves_in_the_jump_tick() {
    let r = rules();
    let mut w = game(&r);
    let j = spawn(&mut w, "JUMPER", ALPHA, 0, 0);
    assert_eq!(disc_x(&w, ALPHA), 0);
    move_to(&mut w, j, at(6, 0));
    let (before, after) = until_cell_changes(&mut w, j);
    assert_eq!(before, 0);
    assert_eq!(after, 6, "relocated in the jump tick itself");
    assert_eq!(
        count_at(&w, ALPHA, 3, 0),
        0,
        "no cell between became a centre"
    );
}

// spec: S-0516 claims 13 and 22 (a fully cloaked teleporting mover takes the adjacent sensor test in the jump tick: an enemy sensor in a cell next to the landing cell reveals it then)
#[test]
fn claim_13_teleport_arrival_reveals_a_cloaked_jumper() {
    let r = rules();
    let mut w = game(&r);
    let j = spawn_cloaked(&mut w, "JUMPERC", ALPHA, 0, 0);
    spawn(&mut w, "SEEKER", BETA, 7, 0);
    move_to(&mut w, j, at(6, 0));
    let c0 = w.obj(j).unwrap().pos.cell();
    for _ in 0..40 {
        run(&mut w, 1);
        if w.obj(j).unwrap().pos.cell() != c0 {
            break;
        }
        assert_eq!(state(&w, j), CloakState::Cloaked, "before the jump");
    }
    assert_ne!(w.obj(j).unwrap().pos.cell(), c0, "it jumped");
    assert_ne!(
        state(&w, j),
        CloakState::Cloaked,
        "revealed in the jump tick"
    );
}

// spec: S-0516 claim 22 (Q-0795: movement outside the table's rows, here a type with a locomotor the engine does not move (a jumpjet's touchdown is S-0502's seam), supplies no cell-arrival update from a changed centre cell: the disc stays where entering play left it)
#[test]
fn claim_22_unbuilt_locomotors_supply_no_update_from_the_cloak_turn() {
    let r = rules();
    let mut w = game(&r);
    let s = spawn(&mut w, "STATIC", ALPHA, 0, 0);
    assert_eq!(disc_x(&w, ALPHA), 0);
    let p = w.obj(s).unwrap().pos;
    w.obj_mut(s).unwrap().pos.x = p.x + 2 * 256;
    run(&mut w, 3);
    assert_ne!(w.obj(s).unwrap().pos.cell(), at(0, 0));
    assert_eq!(disc_x(&w, ALPHA), 0, "no relocation at the cloak update");
}

// spec: S-0516 claim 22 (Entering play: the disc is added when the object enters play, which is not itself a cell-arrival update and supplies no adjacency reveal)
#[test]
fn claim_22_entering_play_adds_the_disc_without_an_adjacency_test() {
    let r = rules();
    let mut w = game(&r);
    spawn(&mut w, "SEEKER", BETA, 1, 0);
    let g = spawn_cloaked(&mut w, "SEEKGHOST", ALPHA, 0, 0);
    assert_eq!(count_at(&w, ALPHA, 0, 0), 1, "the disc is added at once");
    run(&mut w, 4);
    assert_eq!(
        state(&w, g),
        CloakState::Cloaked,
        "placement revealed nothing"
    );
}

// spec: S-0516 claim 22 (Drops: a completed fall landing is a cell-arrival update, also in the same cell, after the height becomes 0 and falling ends; descent moves nothing: a cloaked faller next to an enemy sensor stays cloaked in the air and is revealed by the landing)
#[test]
fn claim_22_fall_landing_is_a_cell_arrival_update() {
    let r = rules();
    let mut w = game(&r);
    spawn(&mut w, "SEEKER", BETA, 1, 0);
    let g = spawn_cloaked(&mut w, "SEEKGHOST", ALPHA, 0, 0);
    w.obj_mut(g).unwrap().pos.z += 600;
    glasnost_sim::fall::start(&mut w, g, false);
    let mut landed = false;
    for _ in 0..80 {
        run(&mut w, 1);
        let falling = w.obj(g).unwrap().fall.falling;
        if falling {
            assert_eq!(state(&w, g), CloakState::Cloaked, "no reveal in descent");
            assert_eq!(disc_x(&w, ALPHA), 0);
        } else {
            landed = true;
            break;
        }
    }
    assert!(landed, "the fall ended");
    assert_ne!(state(&w, g), CloakState::Cloaked, "revealed at the landing");
}
