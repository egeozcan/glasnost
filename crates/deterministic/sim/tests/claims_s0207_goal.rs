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
