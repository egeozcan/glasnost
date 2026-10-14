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

// spec: S-0215 claim 1 (at game set-up: the idle body and guard queued, not a construction body or construction queued)
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
    assert!(!o.beh.ready_mark);
}

// spec: S-0215 claim 11 (a building without a turret: landing on the last stage sets the ready mark, so the queued guard starts; the wrap alone does not set it); S-0108 claim 6 (a wrap restarts at a with N(d, s) for idle)
#[test]
fn claim_11_last_stage_sets_the_mark_wrap_does_not() {
    let (mut w, id) = hut([0, 3, 4], false);
    let speed = w.game_speed;
    let mut stage = 0;
    // Until the stage lands on 2, the last of 0 … 2, guard stays queued.
    loop {
        let (mark, cur, k) = step(&mut w, id);
        if k == 2 {
            // The mark set by the body, cleared by the successful per-tick start.
            assert_eq!(cur, Some(Behaviour::Guard));
            assert!(!mark);
            break;
        }
        assert_eq!((mark, cur), (false, None), "stage {k}");
        stage = k;
    }
    assert_eq!(stage, 1);
    // Guard is current and nothing is queued: a mark set now stays set.
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

// spec: S-0215 claims 1, 11, 32 and 33 (a map-placed armed turreted building starts with guard queued and, through its set-up ready mark, interim Q-0241, takes guard and fires on an enemy in range)
#[test]
fn set_up_turreted_defence_fires() {
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
    let cart = w
        .objects
        .values()
        .find(|o| o.kind == Kind::Vehicle)
        .unwrap()
        .id;
    assert_eq!(w.obj(tower).unwrap().beh.queued, Some(Behaviour::Guard));
    let full = w.obj(cart).unwrap().health;
    for _ in 0..300 {
        glasnost_sim::tick::step(&mut w);
        if w.obj(cart).is_none_or(|o| o.health < full) {
            assert!(w.obj(tower).unwrap().beh.current.is_some());
            return;
        }
    }
    panic!("the tower never hit the cart");
}

// spec: S-0215 claim 11 (a turreted building's idle body range sets no ready mark); interim Q-0241 (its set-up ready mark, so its queued guard becomes current)
#[test]
fn claim_11_turreted_idle_range_sets_no_mark() {
    let (mut w, id) = hut([0, 3, 0], true);
    assert!(w.obj(id).unwrap().beh.ready_mark, "Q-0241 interim");
    let (mark, cur, _) = step(&mut w, id);
    assert_eq!((mark, cur), (false, Some(Behaviour::Guard)));
    for _ in 0..20 {
        let (mark, _, _) = step(&mut w, id);
        assert!(!mark);
    }
}
