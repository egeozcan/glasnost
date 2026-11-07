//! Claim tests of S-0208 revision 46 (Q-2657 answered), claim 6:
//! *Destinations given during a unit's update* (a destination given before
//! the update's locomotion is searched for in that update, one given after it
//! in the next tick; a standing walker with no walking target and no stored
//! path that accepts a scatter whose destination comes from the nearby-cell
//! search has one more locomotion at once, in which it chooses its walking
//! target without moving) and *A new destination while a step is under way*
//! (the walker keeps its walking target, finishes the step and drops its
//! stored path). Built on scenario W17 parts (a) and (c) to (e) and on the
//! draws of W9, on synthetic data: invented type names and values on the
//! flat scene of `common`, our own water cells. No game data.

mod common;

use common::{FOOT, Scene, cell, open_orders, order_move, unit_type};
use glasnost_map::CellXY;
use glasnost_map::terrain::land;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::scatter::{self, ScatterRequest};
use glasnost_sim::tick;
use glasnost_sim::world::{ObjectId, Pos, Target, World};

/// A forced request with no threat point: its destination comes from the
/// nearby-cell search (S-0208 claim 13).
const FORCED: ScatterRequest = ScatterRequest {
    threat: None,
    forced: true,
    anywhere: false,
};

/// The soldier type TESTMAN (speed field 4: walking speed 10).
fn scene() -> Scene {
    let mut s = Scene::new();
    s.infantry("TESTMAN", unit_type(FOOT, 4, 0));
    s
}

fn men(w: &World) -> Vec<ObjectId> {
    common::objects_of(w, Kind::Infantry)
}

fn pos(w: &World, id: ObjectId) -> Pos {
    w.obj(id).unwrap().pos
}

fn walk_target(w: &World, id: ObjectId) -> Option<Pos> {
    w.obj(id)
        .and_then(|o| o.walk.as_ref())
        .and_then(|s| s.target)
}

/// One soldier at standing position `sub` of the middle cell, orders open.
fn alone(sub: i32) -> (World, ObjectId) {
    let mut s = scene();
    s.place_infantry(0, "TESTMAN", cell(0, 0), sub);
    let mut w = s.build();
    open_orders(&mut w);
    let id = men(&w)[0];
    (w, id)
}

/// Makes `c` an island: its eight neighbours become water.
fn island(w: &mut World, c: CellXY) {
    for dy in -1..=1 {
        for dx in -1..=1 {
            if (dx, dy) != (0, 0) {
                let n = CellXY::new(c.x + dx, c.y + dy);
                w.cells.grid.get_mut(n).unwrap().land = land::WATER;
                glasnost_sim::regions::cell_changed(w, n);
            }
        }
    }
}

/// The ticks, counted from the next one (0), at whose end the position
/// differs from the end of the tick before, over `n` ticks.
fn changes(w: &mut World, id: ObjectId, n: usize) -> Vec<usize> {
    let mut last = pos(w, id);
    let mut out = Vec::new();
    for i in 0..n {
        tick::step(w);
        let p = pos(w, id);
        if p != last {
            out.push(i);
        }
        last = p;
    }
    out
}

// spec: S-0208 claim 6 (r46, Destinations given during a unit's update: an order taking effect at the start of tick T is searched for in T's locomotion, the asking tick, and the first step is in T + 1); scenario W17 part a
#[test]
fn w17_a_order_at_the_start_of_a_tick_steps_in_the_next() {
    let (mut w, id) = alone(0);
    order_move(&mut w, id, cell(3, 0));
    // Orders take effect after the turn pass (S-0203 claim 7): applied now,
    // as at the end of the tick just run, it is in effect at the next start.
    glasnost_sim::orders::apply_pending(&mut w);
    let c = changes(&mut w, id, 4);
    assert_eq!(c.first(), Some(&1), "{c:?}");
}

// spec: S-0208 claim 6 (r46: a standing walker with no walking target and no stored path that accepts a scatter whose destination comes from the nearby-cell search has one more locomotion at once, in which it searches and chooses its walking target without moving; that choice takes claim 3's draw when its desired point has class centre, after the scatter's own draw)
#[test]
fn extra_locomotion_chooses_the_walking_target_at_once() {
    let (mut w, id) = alone(0);
    let here = pos(&w, id);
    let before = w.rng.taken();
    assert!(scatter::request(&mut w, id, FORCED));
    let draws = w.rng.taken() - before;
    let o = w.obj(id).unwrap();
    assert!(matches!(o.dest, Some(Target::Cell(_))), "a destination");
    let t = walk_target(&w, id).expect("the walking target is chosen at once");
    assert_eq!(pos(&w, id), here, "chosen without moving");
    let Some(Target::Cell(d)) = o.dest else {
        unreachable!()
    };
    assert_eq!(t.cell(), d, "the first cell of the walk is the neighbour");
    // spec: S-0208 claim 6 (r46: one draw for the scatter, claims 12 and 13, then claim 3's for the centre class)
    assert_eq!(draws, 2, "the scatter's draw and the position choice's");
}

// spec: S-0208 claim 6 (r46: a scatter that comes before the walker's own locomotion in the tick: that locomotion finds the walking target already chosen and takes the first step, so the walker moves on the tick of the scatter)
#[test]
fn extra_locomotion_before_the_own_locomotion_steps_that_tick() {
    let (mut w, id) = alone(0);
    assert!(scatter::request(&mut w, id, FORCED));
    let c = changes(&mut w, id, 3);
    assert_eq!(c.first(), Some(&0), "{c:?}");
}

// spec: S-0208 claim 6 (r46: a scatter whose destination is chosen among the eight neighbours, by claim 12, has no extra locomotion: no walking target until the next locomotion, which is the asking tick; the first step the tick after)
#[test]
fn neighbour_choice_has_no_extra_locomotion() {
    let (mut w, id) = alone(0);
    let p = pos(&w, id);
    let req = ScatterRequest {
        threat: Some(Pos::new(p.x - 600, p.y, p.z)),
        forced: true,
        anywhere: false,
    };
    let before = w.rng.taken();
    assert!(scatter::request(&mut w, id, req));
    assert_eq!(w.rng.taken() - before, 1, "the scatter's draw only");
    assert!(w.obj(id).unwrap().dest.is_some());
    assert_eq!(walk_target(&w, id), None, "no walking target yet");
    let c = changes(&mut w, id, 3);
    assert_eq!(c.first(), Some(&1), "{c:?}");
}

// spec: S-0208 claim 6 (r52: what decides is which choice gives the destination; a walker that already has a walking target is not given this extra locomotion in our engine, Open questions)
#[test]
fn a_walker_with_a_walking_target_gets_no_extra_locomotion() {
    // A fraidycat in move with the scatter property accepts a request that
    // is not forced while it walks (S-0208 claim 11, third case).
    let mut s = scene();
    let mut cat = unit_type(FOOT, 4, 0);
    cat.fraidycat = Some(true);
    s.infantry("TESTCAT", cat);
    let props = glasnost_rules::sim::BehaviourProps {
        scatter: Some(true),
        ..Default::default()
    };
    s.ruleset.sim.behaviours.insert("move".into(), props);
    s.ruleset.sim.general.player_scatter = Some(true);
    s.place_infantry(0, "TESTCAT", cell(0, 0), 0);
    let mut w = s.build();
    open_orders(&mut w);
    let id = men(&w)[0];
    order_move(&mut w, id, cell(6, 0));
    // Until it is in the middle of a step.
    for _ in 0..4 {
        tick::step(&mut w);
    }
    let t = walk_target(&w, id).expect("a step under way");
    let req = ScatterRequest {
        threat: None,
        forced: false,
        anywhere: false,
    };
    let before = w.rng.taken();
    assert!(scatter::request(&mut w, id, req));
    assert_eq!(w.rng.taken() - before, 1, "the scatter's draw only");
    assert_eq!(walk_target(&w, id), Some(t), "the walking target is kept");
    assert!(
        w.obj(id).unwrap().path.steps.is_empty(),
        "the stored path dropped"
    );
}

/// A soldier ordered 6 cells due map east; runs until it is between its
/// first and second standing positions (a walking target in the second
/// cell), then gives it a move order to `to` at the start of the next tick.
/// Returns the world, the soldier and the walking target it held.
fn mid_step(to: impl FnOnce(&mut World) -> CellXY) -> (World, ObjectId, Pos) {
    let (mut w, id) = alone(0);
    order_move(&mut w, id, cell(6, 0));
    for _ in 0..200 {
        tick::step(&mut w);
        if walk_target(&w, id).is_some_and(|t| t.cell() == cell(1, 0))
            && pos(&w, id).cell() == cell(0, 0)
            && w.obj(id).unwrap().walk.as_ref().unwrap().arrived != Some(w.tick - 1)
        {
            break;
        }
    }
    let t = walk_target(&w, id).expect("a step under way");
    let c = to(&mut w);
    order_move(&mut w, id, c);
    // In effect at the next tick's start (S-0203 claim 7).
    glasnost_sim::orders::apply_pending(&mut w);
    (w, id, t)
}

// spec: S-0208 claim 6 (r46, A new destination while a step is under way: the walker keeps its walking target, finishes the step and arrives; the new destination is first searched for in the tick after the arrival tick, where the gate failure clears it, S-0212 claim 23; it moves no further); scenario W17 part c
// spec: S-0215 claim 22 (r132: move current, so the end-of-update check of that update queues the rest behaviour)
#[test]
fn w17_c_order_into_another_reach_region_during_a_step() {
    let pen = cell(4, 4);
    let (mut w, id, t) = mid_step(|w| {
        island(w, pen);
        pen
    });
    let mut arrived = None;
    for i in 0..60 {
        tick::step(&mut w);
        let o = w.obj(id).unwrap();
        if arrived.is_none() && walk_target(&w, id).is_none() {
            arrived = Some(i);
            assert_eq!((o.pos.x, o.pos.y), (t.x, t.y), "arrives on the kept target");
            assert_eq!(
                o.dest,
                Some(Target::Cell(pen)),
                "the destination still reads the pen"
            );
        } else if arrived == Some(i - 1) {
            assert_eq!(o.dest, None, "cleared in the tick after the arrival tick");
            assert_eq!(o.beh.current, Some(Behaviour::Move));
            assert_eq!(o.beh.queued, Some(Behaviour::Guard));
        }
        if arrived.is_some() {
            assert_eq!((o.pos.x, o.pos.y), (t.x, t.y), "moves no further");
        }
    }
    assert!(arrived.is_some());
}

// spec: S-0208 claim 6 (r46: the second order to a cell beyond the cell it is stepping to: the step under way is finished and the walk continues there, with one asking tick after the arrival tick); scenario W17 part d
#[test]
fn w17_d_order_further_along_the_line_during_a_step() {
    let (mut w, id, t) = mid_step(|_| cell(3, 0));
    let mut ticks = Vec::new();
    for _ in 0..200 {
        let before = pos(&w, id);
        let had = walk_target(&w, id);
        tick::step(&mut w);
        ticks.push((before, pos(&w, id), had, walk_target(&w, id)));
        if w.obj(id).unwrap().dest.is_none() {
            break;
        }
    }
    let k = ticks
        .iter()
        .position(|&(_, _, had, now)| had.is_some() && now.is_none())
        .expect("the arrival");
    assert_eq!(
        (ticks[k].1.x, ticks[k].1.y),
        (t.x, t.y),
        "the step is finished"
    );
    // The tick after the arrival tick is the asking tick: no move, a target.
    assert_eq!(
        ticks[k + 1].0,
        ticks[k + 1].1,
        "the asking tick does not move"
    );
    assert!(ticks[k + 1].3.is_some(), "a target chosen there");
    assert_ne!(ticks[k + 2].0, ticks[k + 2].1, "the first step after");
    assert_eq!(
        pos(&w, id).cell(),
        cell(3, 0),
        "the walk ends at the new cell"
    );
}

// spec: S-0208 claim 6 (r46: the second order to the free cell it is stepping into: it arrives there, and claim 16 ends the walk on that arrival tick; no destination after it, no further step); scenario W17 part e
#[test]
fn w17_e_order_to_the_cell_being_stepped_into() {
    let (mut w, id, t) = mid_step(|_| cell(1, 0));
    let mut arrival = None;
    for i in 0..60 {
        tick::step(&mut w);
        if arrival.is_none() && walk_target(&w, id).is_none() {
            arrival = Some(i);
            let o = w.obj(id).unwrap();
            assert_eq!(
                o.dest, None,
                "no destination at the end of the arrival tick"
            );
        }
        if arrival.is_some() {
            let p = pos(&w, id);
            assert_eq!((p.x, p.y), (t.x, t.y), "no further step");
        }
    }
    assert!(arrival.is_some());
}

// spec: S-0208 claim 6 (r46: the extra locomotion of a released walker in the factory request's tick, S-0417 claim 17, comes after the update's locomotion, so the first step falls in the next tick) is held by `claims_s0417_r79.rs`; here the extra locomotion is given only to a walker on foot with the walk locomotor: a teleporting unit has its own jump instead (S-0417 claim 17, During the walk-off)
#[test]
fn extra_locomotion_only_for_the_walk_locomotor() {
    let mut s = scene();
    let mut blink = unit_type(FOOT, 4, 0);
    blink.locomotor = Some(glasnost_rules::sim::Locomotor::Teleport);
    s.infantry("BLINKER", blink);
    s.place_infantry(0, "BLINKER", cell(0, 0), 0);
    let mut w = s.build();
    open_orders(&mut w);
    let id = men(&w)[0];
    // The same request with any extra locomotion held off (the r52
    // departure's guard, a locomotion of its own under way): the teleporting
    // unit's request takes the same draws and ends in the same state, so it
    // had no extra locomotion.
    let mut held = w.clone();
    held.locomotion_under_way.push(id);
    assert!(scatter::request(&mut w, id, FORCED));
    assert!(scatter::request(&mut held, id, FORCED));
    held.locomotion_under_way.clear();
    assert_eq!(walk_target(&w, id), None);
    assert_eq!(w.rng.taken(), held.rng.taken(), "no draw of its own");
    assert_eq!(
        glasnost_sim::save::state_hash(&w),
        glasnost_sim::save::state_hash(&held)
    );
}

/// Soldiers filling standing positions 2 to 4 of every cell of a block of
/// `n` × `n` cells around the middle one; the middle cell's are the first.
fn crowd(n: i32) -> (World, Vec<ObjectId>) {
    let mut s = scene();
    let h = n / 2;
    for sub in 2..=4 {
        s.place_infantry(0, "TESTMAN", cell(0, 0), sub);
    }
    for dy in -h..=h {
        for dx in -h..=h {
            if (dx, dy) != (0, 0) {
                for sub in 2..=4 {
                    s.place_infantry(0, "TESTMAN", cell(dx, dy), sub);
                }
            }
        }
    }
    let mut w = s.build();
    open_orders(&mut w);
    let m = men(&w);
    (w, m)
}

// spec: S-0208 claim 6 (r52, Requests sent from an extra locomotion, In our engine: a departure, no switch; no unit is given an extra locomotion while a locomotion of its own is under way, so the chain of extra locomotions in a crowd ends; every other unit that accepts such a request gets its extra locomotion)
#[test]
fn q2702_extra_locomotions_in_a_crowd_end() {
    let (mut w, m) = crowd(5);
    let mut v = w.clone();
    assert!(scatter::request(&mut w, m[0], FORCED));
    assert!(scatter::request(&mut v, m[0], FORCED));
    assert_eq!(
        glasnost_sim::save::state_hash(&w),
        glasnost_sim::save::state_hash(&v),
        "deterministic"
    );
    assert!(w.locomotion_under_way.is_empty(), "the chain is unwound");
    for _ in 0..200 {
        tick::step(&mut w);
    }
}

// spec: S-0208 claim 6 (r52, Requests sent from an extra locomotion, In our engine: no extra locomotion for a unit already in its own further up the chain)
#[test]
fn q2702_no_extra_locomotion_inside_its_own() {
    let (mut w, id) = alone(0);
    w.locomotion_under_way.push(id);
    assert!(scatter::request(&mut w, id, FORCED));
    assert_eq!(walk_target(&w, id), None, "no second extra locomotion");
    w.locomotion_under_way.clear();
}
