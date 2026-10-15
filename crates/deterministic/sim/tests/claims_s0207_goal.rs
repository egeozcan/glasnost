//! Claim tests for S-0207 claim 26 (removing the order-level goal at rest,
//! Q-0797) and the claims of S-0215 that it composes with (claims 22 and
//! 29), on the synthetic scenario. The building-goal rows with a depot are
//! in `claims_s0314_depot.rs`.

use glasnost_map::CellXY;
use glasnost_sim::drive;
use glasnost_sim::orders::{self, Order};
use glasnost_sim::process;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit;
use glasnost_sim::tick;
use glasnost_sim::world::{HouseId, ObjectId, Target, World};

const ALPHA: HouseId = HouseId(0);
const SIZE: i32 = 40;

fn game() -> (World, ObjectId) {
    let r = testkit::ruleset();
    let mut w = testkit::game_with(&r, SIZE, 3);
    let v = testkit::spawn(
        &mut w,
        Kind::Vehicle,
        "CART",
        0,
        testkit::mid(SIZE, 0, 0),
        0,
    );
    while w.tick < 4 {
        tick::step(&mut w);
    }
    (w, v)
}

fn mv(actor: ObjectId, c: CellXY) -> Order {
    Order::Behaviour {
        actor,
        behaviour: Behaviour::Move,
        attack_move: false,
        target: None,
        dest: Some(Target::Cell(c)),
    }
}

// spec: S-0207 claim 26 (same-cell order from rest: the first locomotion update without an active track removes the cell destination, in that update's tick, with no search and no wait for due behaviour processing; assignment supplied the locomotor point, which goes with it)
#[test]
fn claim_26_same_cell_order_is_cleaned_up_by_the_first_update() {
    let (mut w, v) = game();
    let here = w.obj(v).unwrap().pos.cell();
    assert!(orders::issue(&mut w, ALPHA, mv(v, here)));
    orders::apply_pending(&mut w);
    assert_eq!(w.obj(v).unwrap().dest, Some(Target::Cell(here)));
    assert!(drive::goal_present(&w, v), "assignment supplies the point");
    let taken = w.rng.taken();
    drive::tick(&mut w, v);
    let o = w.obj(v).unwrap();
    assert_eq!(o.dest, None, "removed by the first update");
    assert!(!drive::goal_present(&w, v), "the point goes with it");
    assert!(o.drive.as_ref().unwrap().track.is_none());
    assert_eq!(w.rng.taken(), taken, "no search draw");
}

// spec: S-0207 claim 26 (final track: the qualifying completion of claim 25 clears the locomotor point in the completion tick, not the order-level destination; the next tick's update, which begins without a track, removes the cell destination)
#[test]
fn claim_26_final_track_leaves_the_cell_goal_for_the_next_update() {
    let (mut w, v) = game();
    let here = w.obj(v).unwrap().pos.cell();
    let dest = CellXY::new(here.x, here.y - 1);
    assert!(orders::issue(&mut w, ALPHA, mv(v, dest)));
    let mut completed = None;
    let mut saw_track = false;
    for _ in 0..200 {
        tick::step(&mut w);
        let o = w.obj(v).unwrap();
        let on_track = o.drive.as_ref().unwrap().track.is_some();
        saw_track |= on_track;
        if saw_track && !on_track {
            completed = Some(w.tick);
            break;
        }
    }
    assert!(completed.is_some(), "the track completes");
    let o = w.obj(v).unwrap();
    assert_eq!(o.pos.cell(), dest, "standing in the goal cell");
    assert_eq!(
        o.dest,
        Some(Target::Cell(dest)),
        "completion keeps the goal"
    );
    assert!(!drive::goal_present(&w, v), "completion cleared the point");
    // The next tick's locomotion update has no active track and removes it.
    tick::step(&mut w);
    assert_eq!(w.obj(v).unwrap().dest, None, "removed at the next update");
}

// spec: S-0207 claim 26 (with approach destinations the current cell goal is abandoned and the vehicle falls back to rest under S-0215 claim 13, which supplies the replacement goal)
#[test]
fn claim_26_approach_destinations_fall_back() {
    let (mut w, v) = game();
    let here = w.obj(v).unwrap().pos.cell();
    let far = CellXY::new(here.x, here.y - 6);
    let o = w.obj_mut(v).unwrap();
    o.dest = Some(Target::Cell(here));
    o.queued_dests = vec![Target::Cell(far)];
    drive::tick(&mut w, v);
    let o = w.obj(v).unwrap();
    assert_ne!(o.dest, Some(Target::Cell(here)), "the old goal is gone");
    assert_eq!(
        o.dest,
        Some(Target::Cell(far)),
        "the fallback supplies the next"
    );
    assert!(o.queued_dests.is_empty());
}

// spec: S-0215 claim 22 (Q-0797: move processing removes no destination for cell equality and a due processing before cleanup sees it and chooses the standard delay; S-0207 claim 26's cleanup does not touch the timer, and the next due processing with no destination, not moving and nothing queued falls back and chooses 1 tick)
#[test]
fn s0215_claim_22_cleanup_is_the_locomotions_and_leaves_the_timer() {
    let (mut w, v) = game();
    let here = w.obj(v).unwrap().pos.cell();
    let t = w.tick;
    {
        let o = w.obj_mut(v).unwrap();
        o.dest = Some(Target::Cell(here));
        o.beh.force(Behaviour::Move, t, true);
    }
    process::turn(&mut w, v);
    let o = w.obj(v).unwrap();
    assert_eq!(
        o.dest,
        Some(Target::Cell(here)),
        "processing keeps the goal"
    );
    let (at, d) = o.beh.last.unwrap();
    assert_eq!(at, t);
    assert!((14..=16).contains(&d), "the standard delay, got {d}");
    // The locomotion cleanup leaves the processing timer as it was.
    let timer = w.obj(v).unwrap().beh.last;
    drive::tick(&mut w, v);
    assert_eq!(w.obj(v).unwrap().dest, None, "cleanup removed the goal");
    assert_eq!(w.obj(v).unwrap().beh.last, timer, "the timer is untouched");
    // The next due processing finds no destination and falls back with 1.
    w.tick += d;
    process::turn(&mut w, v);
    let b = &w.obj(v).unwrap().beh;
    assert_eq!(b.last, Some((w.tick, 1)));
    assert!(
        b.queued.is_some(),
        "a fallback behaviour is queued (S-0215 claim 13)"
    );
}

/// The mover `v` with the named unit `n` (of `kind`, `name`) moved onto the
/// mover's own point, the mover's destination naming it.
fn naming(kind: Kind, name: &str) -> (World, ObjectId, ObjectId) {
    let (mut w, v) = game();
    let here = w.obj(v).unwrap().pos.cell();
    let n = testkit::spawn(&mut w, kind, name, 0, CellXY::new(here.x, here.y - 3), 0);
    let at = w.obj(v).unwrap().pos;
    w.obj_mut(n).unwrap().pos = at;
    w.obj_mut(v).unwrap().dest = Some(Target::Object(n));
    // Not at rest in guard: the cell-equality rows are tested here, not the guard row.
    w.obj_mut(v).unwrap().beh.current = Some(Behaviour::Move);
    (w, v, n)
}

// spec: S-0207 claim 26 (Q-0830, second and third rows: a destination naming a vehicle or an infantry unit, stationary or moving, is not a cell destination: the first locomotion update at rest in the supplied point's cell removes neither the destination nor the point, as for a building)
#[test]
fn claim_26_a_destination_naming_a_unit_is_kept_by_cell_equality() {
    for (kind, name) in [(Kind::Vehicle, "BUGGY"), (Kind::Infantry, "WALKER")] {
        let (mut w, v, n) = naming(kind, name);
        assert_eq!(
            w.obj(n).unwrap().pos.cell(),
            w.obj(v).unwrap().pos.cell(),
            "the supplied point is in the mover's cell"
        );
        for step in 0..3 {
            // A named unit that moves within the cell (no replacement instruction) makes no difference.
            if step == 2 {
                w.obj_mut(n).unwrap().pos.x += 20;
            }
            drive::tick(&mut w, v);
            assert_eq!(
                w.obj(v).unwrap().dest,
                Some(Target::Object(n)),
                "{name}, update {step}"
            );
            assert!(drive::goal_present(&w, v), "{name}: the point stays");
        }
    }
}

// spec: S-0207 claim 26 (Q-0830, guard row: at rest in guard, the retained locomotor point equal to the vehicle's world point in x, y and z removes the destination, also one naming a unit; the comparison is with the retained point, not the named unit's present position, and a point differing in the same cell does not qualify)
#[test]
fn claim_26_guard_row_for_a_destination_naming_a_unit_compares_the_retained_point() {
    // (the retained point's offset from the mover, the named unit's offset from the mover) -> removed?
    for (mover_dx, named_dx, removed) in [(0, 0, true), (0, 90, true), (30, 0, false)] {
        let (mut w, v, n) = naming(Kind::Vehicle, "BUGGY");
        let p = w.obj(v).unwrap().pos;
        {
            let o = w.obj_mut(v).unwrap();
            o.beh.current = Some(Behaviour::Guard);
            let d = o.drive.as_mut().unwrap();
            d.for_dest = Some(Target::Object(n));
            d.goal = Some(glasnost_sim::world::Pos::new(p.x - mover_dx, p.y, p.z));
        }
        w.obj_mut(n).unwrap().pos.x += named_dx;
        drive::tick(&mut w, v);
        let kept = w.obj(v).unwrap().dest == Some(Target::Object(n));
        assert_eq!(
            kept, !removed,
            "mover {mover_dx}, named {named_dx}: removed = {removed}"
        );
    }
}

// spec: S-0215 claim 22 (a destination naming a vehicle, infantry unit or aircraft is retained under cell equality: a due move processing sees it and chooses the standard delay, and the locomotion cleanup does not remove it or touch the timer)
#[test]
fn s0215_claim_22_a_unit_goal_is_kept_through_processing_and_cleanup() {
    for (kind, name) in [(Kind::Vehicle, "BUGGY"), (Kind::Infantry, "WALKER")] {
        let (mut w, v, n) = naming(kind, name);
        let t = w.tick;
        w.obj_mut(v).unwrap().beh.force(Behaviour::Move, t, true);
        process::turn(&mut w, v);
        let (at, d) = w.obj(v).unwrap().beh.last.unwrap();
        assert_eq!(at, t);
        assert!(
            (14..=16).contains(&d),
            "{name}: the standard delay, got {d}"
        );
        let timer = w.obj(v).unwrap().beh.last;
        drive::tick(&mut w, v);
        assert_eq!(w.obj(v).unwrap().dest, Some(Target::Object(n)), "{name}");
        assert_eq!(w.obj(v).unwrap().beh.last, timer, "{name}: timer untouched");
    }
}

// spec: S-0215 claim 22 (the exact-point guard exception of S-0207 claim 26 removes a goal naming a unit and invokes the approach-destination fallback; it compares the retained point, not the named unit's current position)
#[test]
fn s0215_claim_22_guard_exception_with_a_unit_goal_falls_back_to_the_approach_destination() {
    let (mut w, v, n) = naming(Kind::Vehicle, "BUGGY");
    let here = w.obj(v).unwrap().pos;
    let far = CellXY::new(here.cell().x, here.cell().y - 6);
    {
        let o = w.obj_mut(v).unwrap();
        o.beh.current = Some(Behaviour::Guard);
        o.queued_dests = vec![Target::Cell(far)];
        let d = o.drive.as_mut().unwrap();
        d.for_dest = Some(Target::Object(n));
        d.goal = Some(here);
    }
    // The named unit is elsewhere in the cell: the retained point alone decides.
    w.obj_mut(n).unwrap().pos.x += 90;
    drive::tick(&mut w, v);
    let o = w.obj(v).unwrap();
    assert_ne!(
        o.dest,
        Some(Target::Object(n)),
        "the unit goal is abandoned"
    );
    assert_eq!(
        o.dest,
        Some(Target::Cell(far)),
        "the approach fallback supplies the next"
    );
    assert!(o.queued_dests.is_empty());
}
