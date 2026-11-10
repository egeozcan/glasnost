//! Claim tests of S-0208 revision 58 and S-0209 revision 60 (Q-2750
//! answered), S-0208 claim 6, *Destinations given during a unit's update*: a
//! walker standing with no walking target whose stored path a scatter drops
//! has the extra locomotion, in which it searches for the scatter's
//! destination (scenario W17 part l); a scatter whose cell differs from its
//! destination ends a running retry delay and restarts the blockage delay
//! (W17 part m, S-0209 claim 10). Synthetic data: invented type names and
//! values on the flat scene of `common`, our own corridor of raised cells.
//! No game data. Ledger: `docs/m8-claims/intake114.md`.

mod common;

use common::{FOOT, Scene, cell, open_orders, order_move, run_until, unit_type};
use glasnost_map::CellXY;
use glasnost_rules::sim::Decimal;
use glasnost_sim::rules::Kind;
use glasnost_sim::scatter::{self, ScatterRequest};
use glasnost_sim::tick;
use glasnost_sim::world::{Layer, ObjectId, Pos, Target, World};

/// A fraidycat soldier TESTCAT (speed field 4) whose *move* has the scatter
/// property, with the player-scatter rule on, so that it accepts a request
/// that is not forced while its locomotor is moving (S-0208 claim 11, third
/// case). `retry`: the path-retry rate (S-0212 F3).
fn scene(retry: Option<&str>) -> Scene {
    let mut s = Scene::new();
    let mut cat = unit_type(FOOT, 4, 0);
    cat.fraidycat = Some(true);
    s.infantry("TESTCAT", cat);
    let props = glasnost_rules::sim::BehaviourProps {
        scatter: Some(true),
        ..Default::default()
    };
    s.ruleset.sim.behaviours.insert("move".into(), props);
    s.ruleset.sim.general.player_scatter = Some(true);
    if let Some(r) = retry {
        s.ruleset.sim.general.path_delay = Some(Decimal(r.into()));
    }
    s
}

const UNFORCED: ScatterRequest = ScatterRequest {
    threat: None,
    forced: false,
    anywhere: false,
};

fn pos(w: &World, id: ObjectId) -> Pos {
    w.obj(id).unwrap().pos
}

fn walk_target(w: &World, id: ObjectId) -> Option<Pos> {
    w.obj(id)
        .and_then(|o| o.walk.as_ref())
        .and_then(|s| s.target)
}

fn standing_with_path(w: &World, id: ObjectId) -> bool {
    let o = w.obj(id).unwrap();
    o.walk.as_ref().is_some_and(|s| s.target.is_none()) && !o.path.steps.is_empty()
}

// spec: S-0208 claim 6 (r58, Q-2750: a walker standing with no walking target whose stored path the scatter drops, one in move after the arrival tick of a standing position in the middle of its walk among them, searches in the extra locomotion for the scatter's destination and chooses its walking target there, so it moves in the tick of the scatter; the control at rest with no destination after a walk has ended moves in that tick as well); claim 11 (third case: fraidycat, scatter property, permission); scenario W17 part l
#[test]
fn w17_l_a_dropped_path_gives_the_extra_locomotion() {
    for control in [false, true] {
        let mut s = scene(None);
        s.place_infantry(0, "TESTCAT", cell(0, 0), 0);
        let mut w = s.build();
        open_orders(&mut w);
        let id = common::objects_of(&w, Kind::Infantry)[0];
        if control {
            order_move(&mut w, id, cell(1, 0));
            let n = run_until(&mut w, 400, |w| {
                w.obj(id).unwrap().dest.is_none() && walk_target(w, id).is_none()
            });
            assert!(n < 400, "the walk ends");
        } else {
            order_move(&mut w, id, cell(6, 0));
            let n = run_until(&mut w, 400, |w| {
                pos(w, id).cell() != cell(0, 0) && standing_with_path(w, id)
            });
            assert!(n < 400, "it stands between steps with its stored path");
        }
        let before = pos(&w, id);
        assert!(scatter::request(&mut w, id, UNFORCED), "control {control}");
        let o = w.obj(id).unwrap();
        let Some(Target::Cell(d)) = o.dest else {
            panic!("control {control}: the scatter's destination {:?}", o.dest)
        };
        assert!(
            o.path.steps.is_empty() || control,
            "the stored path is dropped"
        );
        assert_eq!(
            o.path.for_goal,
            Some(d),
            "control {control}: searched at once"
        );
        assert!(
            walk_target(&w, id).is_some(),
            "control {control}: chosen at once"
        );
        assert_eq!(pos(&w, id), before, "chosen without moving");
        tick::step(&mut w);
        assert_ne!(
            pos(&w, id),
            before,
            "control {control}: it moves in the tick"
        );
    }
}

/// A corridor along y = 0 from x = -3 to x = 3, walled by raised cells
/// with one gap at (-3, 1) to the open ground outside, with TESTCAT at
/// x = -2 and a path-retry delay of 90 ticks. Every route from inside to
/// the corridor's east end runs along the corridor; the gap lets the
/// walker reach the open ground south of it.
fn corridor() -> (World, ObjectId) {
    let mut s = scene(Some("0.1"));
    for x in -4..=4 {
        s.set_cell(cell(x, -1), 0, 4);
        if x != -3 {
            s.set_cell(cell(x, 1), 0, 4);
        }
    }
    s.set_cell(cell(-4, 0), 0, 4);
    s.set_cell(cell(4, 0), 0, 4);
    s.place_infantry(0, "TESTCAT", cell(-2, 0), 0);
    let mut w = s.build();
    open_orders(&mut w);
    let id = common::objects_of(&w, Kind::Infantry)[0];
    (w, id)
}

/// Runs until the soldier stands in `c` with no walking target, past its
/// arrival tick.
fn stand_in(w: &mut World, id: ObjectId, c: CellXY) {
    let n = run_until(w, 400, |w| {
        let o = w.obj(id).unwrap();
        o.pos.cell() == c
            && o.walk
                .as_ref()
                .is_some_and(|s| s.target.is_none() && s.arrived.is_some())
    });
    assert!(n < 400, "never stood in {c:?}");
}

// spec: S-0208 claim 6 (r58: a scatter whose cell differs from the walker's destination ends any retry delay that was running, S-0212 claim 21, so the search in the extra locomotion is not held back; the scatter restarts the blockage delay, S-0209 claim 10); S-0209 claim 10 (a moving blocker: wait, a re-path on the first tick of the wait starts the retry delay); scenario W17 part m
#[test]
fn w17_m_a_scatter_ends_the_retry_delay_of_a_blocked_walker() {
    let (mut w, id) = corridor();
    order_move(&mut w, id, cell(3, 0));
    stand_in(&mut w, id, cell(0, 0));
    w.cells.claim_vehicle(cell(1, 0), Layer::Ground, true);
    tick::step(&mut w);
    let first = w.tick - 1;
    let o = w.obj(id).unwrap();
    assert_eq!(
        o.path.blocked_since,
        Some(first),
        "it waits behind the blocker"
    );
    assert_eq!(o.path.retry_at, first + 90, "the re-path started the delay");
    common::run(&mut w, 20);
    let before = pos(&w, id);
    assert_eq!(before.cell(), cell(0, 0));
    assert_eq!(walk_target(&w, id), None, "still waiting");
    assert_eq!(w.obj(id).unwrap().path.blocked_since, Some(first));
    assert!(
        w.obj(id).unwrap().path.retry_at > w.tick,
        "the retry delay runs"
    );
    assert!(scatter::request(&mut w, id, UNFORCED));
    let o = w.obj(id).unwrap();
    let Some(Target::Cell(d)) = o.dest else {
        panic!("the scatter's destination {:?}", o.dest)
    };
    // The scatter search (S-0212 claim 25, no reach region) gives a cell on
    // the open ground south of the corridor in this set-up, reached through
    // the gap, a route away from the blocker.
    assert_eq!(d.y, cell(0, 2).y, "south of the corridor: {d:?}");
    assert!(o.path.retry_at <= w.tick, "the retry delay has ended");
    assert_eq!(o.path.blocked_since, None, "the blockage delay restarts");
    assert_eq!(o.path.for_goal, Some(d), "searched in the extra locomotion");
    let t = walk_target(&w, id).expect("a walking target chosen at once");
    assert_eq!(t.cell(), cell(-1, 0), "west, towards the gap");
    tick::step(&mut w);
    assert_ne!(pos(&w, id), before, "it moves in the tick of the scatter");
}

// spec: S-0209 claim 10 (r76, Q-2822: an accepted scatter whose cell is already the walker's destination starts the blockage delay afresh too, S-0208 claim 6)
// spec: S-0212 claim 21 (r98, Q-2896: that destination given again by a scatter ends the walker's running retry delay too, so the search in the extra locomotion is not held back, as S-0212's note on S-0208 claim 6 reads it)
#[test]
fn s0209_r76_a_scatter_to_the_walkers_own_destination_restarts_the_blockage_delay() {
    let (mut w, id) = corridor();
    order_move(&mut w, id, cell(3, 0));
    stand_in(&mut w, id, cell(0, 0));
    w.cells.claim_vehicle(cell(1, 0), Layer::Ground, true);
    tick::step(&mut w);
    let first = w.tick - 1;
    common::run(&mut w, 20);
    // The cell this scatter gives, read on a copy of the game.
    let mut copy = w.clone();
    assert!(scatter::request(&mut copy, id, UNFORCED));
    let d = copy.obj(id).unwrap().dest.expect("a scatter destination");
    // The walker waits with that cell as its destination, under which the
    // wait was met and its retry delay started.
    let ob = w.obj_mut(id).unwrap();
    ob.dest = Some(d);
    ob.path.blocked_for = Some(d);
    ob.path.retry_for = Some(d);
    let retry = ob.path.retry_at;
    assert_eq!(ob.path.blocked_since, Some(first));
    assert!(retry > w.tick, "the retry delay runs");
    let before = pos(&w, id);
    assert!(scatter::request(&mut w, id, UNFORCED));
    let o = w.obj(id).unwrap();
    assert_eq!(o.dest, Some(d), "the scatter gives that destination again");
    assert_eq!(o.path.blocked_since, None, "the blockage delay restarts");
    assert!(o.path.retry_at <= w.tick, "the retry delay has ended");
    let Target::Cell(dc) = d else {
        panic!("the scatter's destination {d:?}")
    };
    assert_eq!(o.path.for_goal, Some(dc), "searched in the extra locomotion");
    assert!(walk_target(&w, id).is_some(), "a walking target chosen at once");
    tick::step(&mut w);
    assert_ne!(pos(&w, id), before, "it moves in the tick of the scatter");
}
