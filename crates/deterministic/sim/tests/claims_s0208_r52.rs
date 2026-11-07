//! Claim tests of S-0208 revision 52 (Q-2701 and Q-2702 answered), claim 6:
//! *A stored path kept under enter* (a walker given a new destination keeps
//! its stored path when its current or queued behaviour is *enter* and it is
//! in contact with nothing at that moment; `quirks.enter_keeps_stored_path`
//! off drops it), *Destinations given during a unit's update* (a walker
//! standing with no walking target that keeps its path at a scatter has the
//! extra locomotion too and follows that path) and *Requests sent from an
//! extra locomotion* (our engine's departure: no unit is given an extra
//! locomotion while a locomotion of its own is under way). Built on scenario
//! W17 parts (f) to (i), on synthetic data: invented type names and values on
//! the flat scene of `common`, our own water cells. No game data.

mod common;

use common::{FOOT, Scene, WHEEL, cell, open_orders, order_move, unit_type};
use glasnost_map::CellXY;
use glasnost_map::terrain::land;
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::scatter::{self, ScatterRequest};
use glasnost_sim::tick;
use glasnost_sim::world::{HouseId, ObjectId, Pos, Target, World};

/// The soldier type TESTMAN (speed field 4: walking speed 10) and the
/// wheeled transport VAN for four.
fn scene(keep: bool) -> Scene {
    let mut s = Scene::new();
    s.infantry("TESTMAN", unit_type(FOOT, 4, 0));
    let mut van = unit_type(WHEEL, 6, 8);
    van.passengers = Some(4);
    s.vehicle("VAN", van);
    s.ruleset.globals.quirks.enter_keeps_stored_path = keep;
    s
}

fn pos(w: &World, id: ObjectId) -> Pos {
    w.obj(id).unwrap().pos
}

fn walk_target(w: &World, id: ObjectId) -> Option<Pos> {
    w.obj(id)
        .and_then(|o| o.walk.as_ref())
        .and_then(|s| s.target)
}

fn steps(w: &World, id: ObjectId) -> Vec<u8> {
    w.obj(id).unwrap().path.steps.clone()
}

/// A soldier at standing position 0 of the middle cell and an empty VAN at
/// `van`, orders open.
fn with_van(keep: bool, van: CellXY) -> (World, ObjectId, ObjectId) {
    let mut s = scene(keep);
    s.place_infantry(0, "TESTMAN", cell(0, 0), 0);
    s.place_unit(0, "VAN", van, 0);
    let mut w = s.build();
    open_orders(&mut w);
    let man = common::objects_of(&w, Kind::Infantry)[0];
    let v = common::objects_of(&w, Kind::Vehicle)[0];
    (w, man, v)
}

/// An enter order on `v`, issued for the actor's owner.
fn order_enter(w: &mut World, actor: ObjectId, v: ObjectId) {
    let order = Order::Behaviour {
        actor,
        behaviour: Behaviour::Enter,
        attack_move: false,
        target: None,
        dest: Some(Target::Object(v)),
    };
    assert!(orders::issue(w, HouseId(0), order), "enter order refused");
}

/// Runs until the soldier is between its first and second standing
/// positions (a walking target in the cell after its start); returns that
/// walking target.
fn until_mid_step(w: &mut World, id: ObjectId, next: CellXY) -> Pos {
    for _ in 0..200 {
        tick::step(w);
        if walk_target(w, id).is_some_and(|t| t.cell() == next)
            && pos(w, id).cell() == cell(0, 0)
            && w.obj(id).unwrap().walk.as_ref().unwrap().arrived != Some(w.tick - 1)
        {
            break;
        }
    }
    walk_target(w, id).expect("a step under way")
}

/// The cells the soldier stands in at the end of each of the next `n`
/// ticks.
fn cells(w: &mut World, id: ObjectId, n: usize) -> Vec<CellXY> {
    let mut out = Vec::new();
    for _ in 0..n {
        tick::step(w);
        out.push(pos(w, id).cell());
    }
    out
}

// spec: S-0208 claim 6 (r52, A stored path kept under enter: an enter order given to a walker under move keeps its path, enter being queued when the destination is assigned; it finishes the step under way and goes on along its old line to the end of its stored path, then turns towards what it enters); scenario W17 part f
#[test]
fn w17_f_enter_order_during_a_step_keeps_the_old_line() {
    let (mut w, id, v) = with_van(true, cell(-2, 0));
    order_move(&mut w, id, cell(4, 0));
    until_mid_step(&mut w, id, cell(1, 0));
    let before = steps(&w, id);
    assert!(!before.is_empty(), "a stored path ahead");
    order_enter(&mut w, id, v);
    orders::apply_pending(&mut w);
    let o = w.obj(id).unwrap();
    assert_eq!(o.path.steps, before, "the stored path is kept");
    assert_eq!(o.dest, Some(Target::Object(v)));
    assert!(
        matches!(o.beh.seen(), Some(Behaviour::Enter)) || o.beh.queued == Some(Behaviour::Enter)
    );
    let c = cells(&mut w, id, 400);
    let end = c
        .iter()
        .position(|&x| x == cell(4, 0))
        .expect("reaches the end of its line");
    for k in 1..end {
        assert!(
            c[k].x >= c[k - 1].x,
            "no step towards the VAN before the end of the line: {c:?}"
        );
    }
    assert!(
        c[end..].iter().any(|x| x.x < cell(4, 0).x),
        "then it walks back towards the VAN"
    );
}

// spec: S-0208 Edge cases (quirks.enter_keeps_stored_path off: every new destination drops the stored path, and the walker searches for the new destination after the step under way, claim 6); scenario W17 part f with the switch off
#[test]
fn w17_f_switch_off_drops_the_path() {
    let (mut w, id, v) = with_van(false, cell(-2, 0));
    order_move(&mut w, id, cell(4, 0));
    let t = until_mid_step(&mut w, id, cell(1, 0));
    order_enter(&mut w, id, v);
    orders::apply_pending(&mut w);
    assert!(steps(&w, id).is_empty(), "the stored path is dropped");
    let c = cells(&mut w, id, 200);
    assert!(
        !c.contains(&cell(2, 0)),
        "it never walks on beyond the step under way ({t:?}): {c:?}"
    );
}

// spec: S-0208 claim 6 (r52: any behaviour order given to a walker under enter keeps its path, enter being still current at the assignment; its first step after the step under way is along its line towards what it was entering); scenario W17 part g
#[test]
fn w17_g_move_order_to_a_walker_under_enter_keeps_its_path() {
    for keep in [true, false] {
        let (mut w, id, v) = with_van(keep, cell(8, 0));
        order_enter(&mut w, id, v);
        orders::apply_pending(&mut w);
        until_mid_step(&mut w, id, cell(1, 0));
        assert_eq!(w.obj(id).unwrap().beh.current, Some(Behaviour::Enter));
        let before = steps(&w, id);
        order_move(&mut w, id, cell(0, 3));
        orders::apply_pending(&mut w);
        // The first standing position after the step under way.
        let under_way = walk_target(&w, id);
        let mut first = None;
        for _ in 0..80 {
            tick::step(&mut w);
            if let Some(b) = walk_target(&w, id)
                && Some(b) != under_way
            {
                first = Some(b.cell());
                break;
            }
        }
        let first = first.expect("a next walking target");
        if keep {
            assert!(!before.is_empty());
            assert_eq!(first, cell(2, 0), "on along its line towards the VAN");
        } else {
            assert_ne!(first, cell(2, 0), "towards the new cell");
            assert!(first.y > cell(0, 0).y, "towards the new cell: {first:?}");
        }
    }
}

// spec: S-0208 claim 6 (r52: whether it has a walking target plays no part, so a walker standing between steps with a stored path keeps it too); scenario W17 part h
#[test]
fn w17_h_enter_order_between_steps_keeps_the_path() {
    for keep in [true, false] {
        let (mut w, id, v) = with_van(keep, cell(-2, 0));
        order_move(&mut w, id, cell(4, 0));
        // Until its first arrival tick is over: standing, no walking target, a stored path.
        for _ in 0..200 {
            tick::step(&mut w);
            let o = w.obj(id).unwrap();
            if o.pos.cell() == cell(1, 0) && walk_target(&w, id).is_none() {
                break;
            }
        }
        let before = steps(&w, id);
        assert!(!before.is_empty());
        order_enter(&mut w, id, v);
        orders::apply_pending(&mut w);
        assert_eq!(
            steps(&w, id).is_empty(),
            !keep,
            "kept exactly with the switch on"
        );
    }
}

// spec: S-0208 claim 6 (r52: the condition is read at the moment the destination is given: current or queued behaviour enter, and in contact with nothing; a tether is a contact, glossary contact)
#[test]
fn the_keep_condition() {
    let (mut w, id, v) = with_van(true, cell(-2, 0));
    let set = |w: &mut World, cur: Option<Behaviour>, q: Option<Behaviour>, c: Option<ObjectId>| {
        let o = w.obj_mut(id).unwrap();
        o.beh.current = cur;
        o.beh.queued = q;
        o.contact = c;
    };
    set(&mut w, Some(Behaviour::Enter), None, None);
    assert!(
        glasnost_sim::walk::keeps_stored_path(&w, id),
        "enter current"
    );
    set(&mut w, Some(Behaviour::Move), Some(Behaviour::Enter), None);
    assert!(
        glasnost_sim::walk::keeps_stored_path(&w, id),
        "enter queued"
    );
    set(&mut w, Some(Behaviour::Move), None, None);
    assert!(!glasnost_sim::walk::keeps_stored_path(&w, id), "no enter");
    set(&mut w, Some(Behaviour::Enter), None, Some(v));
    assert!(!glasnost_sim::walk::keeps_stored_path(&w, id), "in contact");
    let (mut w, id, _) = with_van(false, cell(-2, 0));
    let o = w.obj_mut(id).unwrap();
    o.beh.current = Some(Behaviour::Enter);
    assert!(!glasnost_sim::walk::keeps_stored_path(&w, id), "switch off");
}

// spec: S-0208 claim 6 (r52: a walker held by a tether link keeps its link at the order, S-0213 table T3 (b), so it drops its path, unless the link is with a building of the unloading kind, which the order breaks off too); S-0213 table T3 (b), (d) and (f) in that order. The tether is a factory release's (S-0417 claim 10), set by hand: no setup here times an order within one
#[test]
fn seams6_a_tethered_walker_drops_its_path_unless_the_link_is_an_unloading_kind() {
    for unloading in [false, true] {
        let mut s = scene(true);
        let mut hut = glasnost_rules::sim::BuildingType {
            strength: Some(300),
            foundation: Some([1, 1]),
            ..Default::default()
        };
        hut.econ.dock_unload = unloading.then_some(true);
        s.building("HUT", hut);
        s.place_infantry(0, "TESTMAN", cell(0, 0), 0);
        s.place_unit(0, "VAN", cell(8, 0), 0);
        s.place_structure(0, "HUT", cell(-3, 3));
        let mut w = s.build();
        open_orders(&mut w);
        let id = common::objects_of(&w, Kind::Infantry)[0];
        let v = common::objects_of(&w, Kind::Vehicle)[0];
        let k = common::objects_of(&w, Kind::Building)[0];
        order_enter(&mut w, id, v);
        orders::apply_pending(&mut w);
        until_mid_step(&mut w, id, cell(1, 0));
        assert_eq!(w.obj(id).unwrap().beh.current, Some(Behaviour::Enter));
        assert!(!steps(&w, id).is_empty(), "a stored path");
        let o = w.obj_mut(id).unwrap();
        o.econ.exit = Some(glasnost_sim::econ::release::Exit {
            factory: k,
            tethered: true,
            place: 0,
        });
        o.contact = Some(k);
        assert!(glasnost_sim::econ::tethered(&w, id));
        order_move(&mut w, id, cell(0, 3));
        orders::apply_pending(&mut w);
        let o = w.obj(id).unwrap();
        assert_eq!(o.dest, Some(Target::Cell(cell(0, 3))));
        if unloading {
            assert_eq!(o.contact, None, "the order breaks off the unloading link");
            assert!(!o.path.steps.is_empty(), "so it keeps its path");
        } else {
            assert_eq!(o.contact, Some(k), "the tether link is kept");
            assert!(o.path.steps.is_empty(), "so it drops its path");
        }
    }
}

/// A fraidycat soldier whose *enter* has the scatter property, so it
/// accepts a request that is not forced while it walks into the VAN
/// (S-0208 claim 11, third case); stopped at its first standing position,
/// with no walking target and a stored path.
fn cat_under_enter(keep: bool) -> (World, ObjectId, ObjectId) {
    let mut s = scene(keep);
    let mut cat = unit_type(FOOT, 4, 0);
    cat.fraidycat = Some(true);
    s.infantry("TESTCAT", cat);
    let props = glasnost_rules::sim::BehaviourProps {
        scatter: Some(true),
        ..Default::default()
    };
    s.ruleset.sim.behaviours.insert("enter".into(), props);
    s.ruleset.sim.general.player_scatter = Some(true);
    s.place_infantry(0, "TESTCAT", cell(0, 0), 0);
    s.place_unit(0, "VAN", cell(8, 0), 0);
    let mut w = s.build();
    open_orders(&mut w);
    let id = common::objects_of(&w, Kind::Infantry)[0];
    let v = common::objects_of(&w, Kind::Vehicle)[0];
    order_enter(&mut w, id, v);
    orders::apply_pending(&mut w);
    for _ in 0..200 {
        tick::step(&mut w);
        if pos(&w, id).cell() == cell(1, 0) && walk_target(&w, id).is_none() {
            break;
        }
    }
    assert!(!steps(&w, id).is_empty(), "a stored path");
    assert_eq!(w.obj(id).unwrap().beh.current, Some(Behaviour::Enter));
    (w, id, v)
}

const UNFORCED: ScatterRequest = ScatterRequest {
    threat: None,
    forced: false,
    anywhere: false,
};

// spec: S-0208 claim 6 (r52: a walker standing with no walking target that keeps a stored path at a scatter whose destination comes from the nearby-cell search has the extra locomotion, and in it chooses its walking target in the path's next cell, S-0209 claim 6, without a search); claims 12 and 13 (the scatter's destination is still given)
#[test]
fn a_kept_path_is_followed_in_the_extra_locomotion() {
    let (mut w, id, _) = cat_under_enter(true);
    let before = steps(&w, id);
    assert!(scatter::request(&mut w, id, UNFORCED));
    assert!(
        matches!(w.obj(id).unwrap().dest, Some(Target::Cell(_))),
        "the scatter's destination"
    );
    let t = walk_target(&w, id).expect("a walking target chosen at once");
    assert_eq!(
        t.cell(),
        cell(2, 0),
        "in the stored path's next cell, towards the VAN"
    );
    assert_eq!(
        steps(&w, id).len() + 1,
        before.len(),
        "the path is followed, not replaced"
    );
}

// spec: S-0208 claim 6 (r52: a destination given by a rule that is not an order breaks off no contact, so a walker under enter in contact with something drops its path at a scatter); Edge cases (quirks.enter_keeps_stored_path off: dropped too); design: Q-2750 interim (the walker held a stored path at the scatter and does not keep it: no extra locomotion, so no walking target until its next locomotion)
#[test]
fn a_scatter_drops_the_path_in_contact_or_with_the_switch_off() {
    for (keep, contact) in [(true, true), (false, false)] {
        let (mut w, id, v) = cat_under_enter(keep);
        if contact {
            w.obj_mut(id).unwrap().contact = Some(v);
        }
        assert!(scatter::request(&mut w, id, UNFORCED));
        let o = w.obj(id).unwrap();
        assert!(
            matches!(o.dest, Some(Target::Cell(_))),
            "the scatter's destination"
        );
        assert!(o.path.steps.is_empty(), "the stored path is dropped");
        assert_eq!(
            walk_target(&w, id),
            None,
            "no extra locomotion (Q-2750 interim)"
        );
        // Its next locomotion searches for the scatter's destination.
        let Some(Target::Cell(d)) = o.dest else {
            unreachable!()
        };
        tick::step(&mut w);
        assert_eq!(w.obj(id).unwrap().path.for_goal, Some(d));
    }
}

// spec: S-0208 claim 6 (r52: a walker follows a stored path only towards a destination, so a destination that becomes nothing leaves it no path to follow, whatever it keeps)
#[test]
fn no_destination_no_path_to_follow() {
    let (mut w, id, _) = with_van(true, cell(-2, 0));
    order_move(&mut w, id, cell(4, 0));
    for _ in 0..200 {
        tick::step(&mut w);
        if pos(&w, id).cell() == cell(1, 0) && walk_target(&w, id).is_none() {
            break;
        }
    }
    assert!(!steps(&w, id).is_empty());
    w.obj_mut(id).unwrap().dest = None;
    let here = pos(&w, id);
    let c = cells(&mut w, id, 40);
    assert!(
        c.iter().all(|&x| x == here.cell()),
        "no further step: {c:?}"
    );
    assert!(steps(&w, id).is_empty(), "no path left");
}

/// Infantry A on a side position of cell a, three allied soldiers on the
/// side positions of its east neighbour b, every other neighbour of a water
/// (scenario W17 part i). The close-enough distance is an invented quarter
/// cell, so that A's scatter destination is at least that far away (the
/// runs the scenario keeps).
fn crowd_at_b() -> (World, ObjectId, Vec<ObjectId>) {
    let mut s = scene(true);
    s.ruleset.sim.general.close_enough = Some(glasnost_rules::sim::Decimal("0.25".into()));
    s.place_infantry(0, "TESTMAN", cell(0, 0), 2);
    for sub in 2..=4 {
        s.place_infantry(0, "TESTMAN", cell(1, 0), sub);
    }
    let mut w = s.build();
    for dy in -1..=1 {
        for dx in -1..=1 {
            if (dx, dy) != (0, 0) && (dx, dy) != (1, 0) {
                let n = cell(dx, dy);
                w.cells.grid.get_mut(n).unwrap().land = land::WATER;
                glasnost_sim::regions::cell_changed(&mut w, n);
            }
        }
    }
    open_orders(&mut w);
    let m = common::objects_of(&w, Kind::Infantry);
    let a = *m
        .iter()
        .find(|&&x| pos(&w, x).cell() == cell(0, 0))
        .unwrap();
    let b: Vec<ObjectId> = m.into_iter().filter(|&x| x != a).collect();
    (w, a, b)
}

// spec: S-0208 claim 6 (r52, Requests sent from an extra locomotion: the extra locomotion is a whole locomotion, with S-0209's asks; its request to make room, S-0209 claim 8, reaches the occupants of b at once, so they hold destinations at the end of the scatter's tick); scenario W17 part i and W9 part f
#[test]
fn w17_i_requests_from_the_extra_locomotion_fall_in_the_scatter_tick() {
    let (mut w, a, b) = crowd_at_b();
    assert!(orders::issue(
        &mut w,
        HouseId(0),
        Order::Scatter { actor: a }
    ));
    // The order takes effect at the order boundary, the start of tick T.
    orders::apply_pending(&mut w);
    let Some(Target::Cell(d)) = w.obj(a).unwrap().dest else {
        panic!("A's scatter destination")
    };
    assert_ne!(d, cell(0, 0));
    // In our engine the request comes from A's extra locomotion, already
    // at the order (the run of the original does not tell which).
    for &x in &b {
        assert!(
            w.obj(x).unwrap().dest.is_some(),
            "the request reached b at once"
        );
    }
    tick::step(&mut w);
    for x in b {
        assert!(
            w.obj(x).unwrap().dest.is_some(),
            "each occupant of b holds a destination at the end of tick T"
        );
    }
}

// spec: S-0208 claim 6 (r52, In our engine: no unit is given an extra locomotion while a locomotion of its own, that of its update or an extra one, is under way; the request is otherwise accepted, with its draw and destination)
#[test]
fn no_extra_locomotion_while_one_of_its_own_is_under_way() {
    let mut s = scene(true);
    s.place_infantry(0, "TESTMAN", cell(0, 0), 0);
    let mut w = s.build();
    open_orders(&mut w);
    let id = common::objects_of(&w, Kind::Infantry)[0];
    let forced = ScatterRequest {
        threat: None,
        forced: true,
        anywhere: false,
    };
    w.locomotion_under_way.push(id);
    let before = w.rng.taken();
    assert!(scatter::request(&mut w, id, forced));
    assert_eq!(w.rng.taken() - before, 1, "the scatter's draw only");
    assert!(w.obj(id).unwrap().dest.is_some(), "the request is accepted");
    assert_eq!(walk_target(&w, id), None, "no extra locomotion");
    w.locomotion_under_way.clear();
    // Between turns no locomotion is under way.
    tick::step(&mut w);
    assert!(w.locomotion_under_way.is_empty());
}
