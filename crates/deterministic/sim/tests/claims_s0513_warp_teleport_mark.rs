//! A warp leaves no standing-position mark of a teleporting infantry unit
//! at a landing point it never jumped to (#363; S-0208 claim 19, S-0513
//! claim 51). On the synthetic superweapon rules of `sw_common` plus an
//! invented infantry type with the teleporting locomotor, no game data.

mod sw_common;

use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{LocomotionClass, Locomotor, UnitType};
use glasnost_sim::cells::Marker;
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::Behaviour;
use glasnost_sim::world::{Layer, ObjectId, Target, World};
use sw_common::*;

fn rules() -> Ruleset {
    let mut r = ruleset();
    r.sim.general.superweapons.chrono_delay = Some(7);
    // An infantry type with the teleporting locomotor, not killed by a warp.
    r.sim.infantry.insert(
        "BLINK".into(),
        UnitType {
            strength: Some(100),
            speed: Some(4),
            locomotion_class: Some(LocomotionClass::Foot),
            locomotor: Some(Locomotor::Teleport),
            teleporter: Some(true),
            ..Default::default()
        },
    );
    r.registries
        .entry("infantry".into())
        .or_default()
        .push("BLINK".into());
    r
}

/// Every standing-position mark held in the name of infantry unit `id`.
fn marks_of(w: &World, id: ObjectId) -> Vec<(CellXY, Layer)> {
    let mut v = Vec::new();
    let cells: Vec<CellXY> = w.cells.grid.iter().map(|(c, _)| c).collect();
    for c in cells {
        for l in [Layer::Ground, Layer::Deck] {
            if let Some(rec) = w.cells.occ(c, l)
                && rec
                    .marks
                    .iter()
                    .flatten()
                    .any(|m| matches!(m.by, Marker::Infantry(x, _) if x == id))
            {
                v.push((c, l));
            }
        }
    }
    v
}

fn move_order(w: &mut World, id: ObjectId, c: CellXY) {
    let owner = w.obj(id).unwrap().owner.unwrap();
    assert!(orders::issue(
        w,
        owner,
        Order::Behaviour {
            actor: id,
            behaviour: Behaviour::Move,
            attack_move: false,
            target: None,
            dest: Some(Target::Cell(c)),
        }
    ));
}

// spec: S-0208 claim 19 (an infantry unit reserves its chosen standing position; Q-0235: a landing point not jumped to releases its reservation when it is replaced) and S-0513 claim 51 (launch clears the current driving step; none resumes when the warp ends)
#[test]
fn issue_363_a_join_after_a_fixed_landing_point_leaves_no_mark_there() {
    let mut w = game(&rules());
    spawn(&mut w, "CHRONOB", BETA, -9, -9);
    ready_tick(&mut w, BETA, SOURCE, 300);
    let b = spawn(&mut w, "BLINK", ALPHA, 0, 0);
    let l = at(0, -6);
    fire_now(&mut w, BETA, SOURCE, at(0, 0));
    // One order step: house 0's move order first, then house 1's warp.
    move_order(&mut w, b, l);
    fire(&mut w, BETA, WARP, at(5, 5));
    step(&mut w);
    assert!(
        w.obj(b).unwrap().sw.warp.is_some(),
        "set-up: the unit joined the warp"
    );
    run(&mut w, 200);
    let o = w.obj(b).unwrap();
    assert!(o.sw.warp.is_none(), "set-up: the warp is over");
    assert_eq!(
        o.pos.cell(),
        at(5, 5),
        "set-up: it landed at the warp destination"
    );
    assert!(
        !marks_of(&w, b).iter().any(|&(c, _)| c == l),
        "the standing position reserved at the never-used landing cell is still marked: {:?}",
        marks_of(&w, b)
    );
}

// spec: S-0208 claim 19 (an infantry unit reserves its chosen standing position) and S-0513 claims 39 and 51 (a departed member is off the map until it appears; none of the old movement resumes when the warp ends)
#[test]
fn issue_363_a_move_order_to_a_pending_member_leaves_no_mark_there() {
    let mut w = game(&rules());
    spawn(&mut w, "CHRONOB", ALPHA, -9, -9);
    ready_tick(&mut w, ALPHA, SOURCE, 300);
    let b = spawn(&mut w, "BLINK", ALPHA, 0, 0);
    let l = at(0, -6);
    fire_now(&mut w, ALPHA, SOURCE, at(0, 0));
    fire(&mut w, ALPHA, WARP, at(5, 5));
    step(&mut w);
    assert!(
        w.obj(b).unwrap().sw.warp.is_some(),
        "set-up: the unit joined"
    );
    run(&mut w, 5);
    move_order(&mut w, b, l);
    run(&mut w, 201);
    assert!(
        !marks_of(&w, b).iter().any(|&(c, _)| c == l),
        "the standing position reserved at the never-used landing cell is still marked: {:?}",
        marks_of(&w, b)
    );
}
