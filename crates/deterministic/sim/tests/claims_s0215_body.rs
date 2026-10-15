//! Claim tests for S-0215 claims 1 and 11 (revision 19): a building placed
//! at game set-up gets the idle body and *guard* queued, and the body range
//! of a building without a turret sets its ready mark (scenario B33), on
//! synthetic data.

mod common;

use common::{Scene, cell};
use glasnost_map::Structure;
use glasnost_map::objects::Placed;
use glasnost_rules::sim::BuildingType;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::world::{ObjectId, World};

/// A map with one unarmed building of type HUT, idle range `idle`
/// (`AnimIdle` start, count, delay), with or without the turret flag.
fn hut(idle: [i32; 3], turret: bool) -> (World, ObjectId) {
    let mut s = Scene::new();
    s.register("buildings", "HUT");
    let mut b = BuildingType {
        strength: Some(300),
        foundation: Some([1, 1]),
        turret: Some(turret),
        ..Default::default()
    };
    b.econ.anim_idle = Some(idle);
    s.ruleset.sim.buildings.insert("HUT".into(), b);
    s.map.objects.structures.push(Structure {
        placed: Placed {
            key: b"k".to_vec(),
            owner: b"Alpha".to_vec(),
            type_name: b"HUT".to_vec(),
            health: 256,
            cell: cell(0, 0),
            facing: 0,
            raw: Vec::new(),
        },
    });
    let w = s.build();
    let id = w
        .objects
        .values()
        .find(|o| o.kind == Kind::Building)
        .expect("the hut")
        .id;
    (w, id)
}

/// One tick; returns the hut's ready mark and current behaviour after it.
fn step(w: &mut World, id: ObjectId) -> (bool, Option<Behaviour>, i32) {
    glasnost_sim::tick::step(w);
    let o = w.obj(id).unwrap();
    (
        o.beh.ready_mark,
        o.beh.current,
        o.econ.idle_body.unwrap().stage,
    )
}

// spec: S-0215 claim 1 (at game set-up: the idle body and guard queued, not a construction body or construction queued; the grand opening sets the ready mark)
#[test]
fn claim_1_set_up_building_has_guard_queued() {
    let (w, id) = hut([0, 3, 4], false);
    let o = w.obj(id).unwrap();
    assert_eq!(
        (o.beh.current, o.beh.queued),
        (None, Some(Behaviour::Guard))
    );
    assert!(!o.econ.construction.building_up);
    let body = o.econ.idle_body.expect("the idle body");
    // spec: S-0108 claim 6 (an immediate change at set-up: stage a, raw d for art not marked normalised)
    assert_eq!((body.stage, body.delay), (0, 4));
    // spec: S-0215 claim 1 (the grand opening later in the placement sets the ready mark)
    assert!(o.beh.ready_mark);
}

// spec: S-0215 claim 1 (set-up guard becomes current at the first per-tick start, which clears the mark) and claim 11 (a building without a turret: landing on the last stage sets the ready mark; the wrap alone does not); S-0108 claim 6 (a wrap restarts at a with N(d, s) for idle)
#[test]
fn claim_11_last_stage_sets_the_mark_wrap_does_not() {
    let (mut w, id) = hut([0, 3, 4], false);
    let speed = w.game_speed;
    let (mark, cur, k) = step(&mut w, id);
    assert_eq!(
        (mark, cur, k),
        (false, Some(Behaviour::Guard), 0),
        "first turn"
    );
    // Until the stage lands on 2, the last of 0 … 2, no mark.
    loop {
        let (mark, cur, k) = step(&mut w, id);
        assert_eq!(cur, Some(Behaviour::Guard));
        if k == 2 {
            assert!(mark, "landing on the last stage");
            break;
        }
        assert!(!mark, "stage {k}");
    }
    // Clear the mark; the wrap to stage 0 does not set it again.
    w.obj_mut(id).unwrap().beh.ready_mark = false;
    loop {
        let (mark, _, k) = step(&mut w, id);
        if k == 0 {
            assert!(!mark, "the wrap does not set the mark");
            break;
        }
        assert!(!mark);
    }
    let body = w.obj(id).unwrap().econ.idle_body.unwrap();
    assert_eq!(body.delay, glasnost_sim::infantry::normalised(4, speed));
    loop {
        let (mark, _, k) = step(&mut w, id);
        if k == 2 {
            assert!(mark, "landing on the last stage again");
            break;
        }
        assert!(!mark);
    }
}

// spec: S-0215 claim 11 (a zero-delay range: every non-advancing turn sets the ready mark)
#[test]
fn claim_11_zero_delay_range_sets_the_mark_every_turn() {
    let (mut w, id) = hut([0, 3, 0], false);
    let (mark, cur, k) = step(&mut w, id);
    // The first turn's mark is taken by the per-tick start of the queued guard.
    assert_eq!((mark, cur, k), (false, Some(Behaviour::Guard), 0));
    for _ in 0..5 {
        let (mark, _, k) = step(&mut w, id);
        assert_eq!((mark, k), (true, 0));
        w.obj_mut(id).unwrap().beh.ready_mark = false;
    }
}

fn placed(owner: &str, ty: &str, c: glasnost_map::CellXY) -> Placed {
    Placed {
        key: b"k".to_vec(),
        owner: owner.as_bytes().to_vec(),
        type_name: ty.as_bytes().to_vec(),
        health: 256,
        cell: c,
        facing: 0,
        raw: Vec::new(),
    }
}

// spec: S-0215 claim 1 (a map-placed armed turreted building ends set-up with the idle body, guard queued, no current behaviour and the ready mark set by its grand opening; guard is current from its first per-tick start)
#[test]
fn claim_1_set_up_turreted_defence_starts_guard_on_its_first_turn() {
    let mut s = Scene::new();
    s.ruleset = glasnost_sim::testkit::combat_ruleset();
    s.map.objects.structures.push(Structure {
        placed: placed("Alpha", "TOWER", cell(0, 0)),
    });
    s.map.objects.units.push(glasnost_map::Unit {
        placed: placed("Beta", "CART", cell(3, 0)),
    });
    let mut w = s.build();
    let tower = w
        .objects
        .values()
        .find(|o| o.kind == Kind::Building)
        .unwrap()
        .id;
    let o = w.obj(tower).unwrap();
    assert!(o.econ.idle_body.is_some());
    assert_eq!(
        (o.beh.queued, o.beh.current, o.beh.ready_mark),
        (Some(Behaviour::Guard), None, true)
    );
    glasnost_sim::tick::step(&mut w);
    let o = w.obj(tower).unwrap();
    assert_eq!(
        (o.beh.queued, o.beh.current),
        (None, Some(Behaviour::Guard))
    );
}

// spec: S-0215 claim 11 (a turreted building's idle body range sets no ready mark) and claim 1 (its set-up guard is current from the first per-tick start, which clears the grand opening's mark)
#[test]
fn claim_11_turreted_idle_range_sets_no_mark() {
    let (mut w, id) = hut([0, 3, 0], true);
    assert!(w.obj(id).unwrap().beh.ready_mark);
    let (mark, cur, _) = step(&mut w, id);
    assert_eq!((mark, cur), (false, Some(Behaviour::Guard)));
    for _ in 0..20 {
        let (mark, _, _) = step(&mut w, id);
        assert!(!mark);
    }
}
