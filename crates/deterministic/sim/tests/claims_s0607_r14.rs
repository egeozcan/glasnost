//! S-0607 revision 14 (M8 step A3 intake 45; answers Q-2035): claim 20's
//! restoration test at the end of each of the unit's turns, whatever its
//! behaviour and house, not while it is being erased; the hovering set-down
//! unit's retry at the end of each turn, not at a fall-back to rest.
//! Synthetic data only (the lift fixture of `lift_common`); no stock value.

#[path = "lift_common/mod.rs"]
mod lift_common;

use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_sim::combat::effects::Effect;
use glasnost_sim::jump::{self, Phase};
use glasnost_sim::lift;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::world::{ObjectId, Target, World};
use lift_common::*;

/// One tick of a watched unit, read after the tick.
#[derive(Clone, Copy, Debug)]
struct Seen {
    tick: i32,
    seized: bool,
    dropped: bool,
    height: i32,
    moving: bool,
    target: bool,
    hunt: bool,
    shots: usize,
}

fn look(w: &World, v: ObjectId, shots: usize) -> Seen {
    let o = w.obj(v).unwrap();
    Seen {
        tick: w.tick,
        seized: lift::seized(w, v),
        dropped: lift::dropped(w, v),
        height: height(w, v),
        moving: jump::moving(w, v),
        target: o.target.is_some(),
        hunt: o.beh.current == Some(Behaviour::Hunt),
        shots,
    }
}

/// Steps `n` ticks, recording `v` after each (its shots in that tick).
fn watch(w: &mut World, v: ObjectId, n: i32) -> Vec<Seen> {
    let mut seen = Vec::new();
    for _ in 0..n {
        glasnost_sim::tick::step(w);
        let shots = std::mem::take(&mut w.effects)
            .iter()
            .filter(|e| matches!(e, Effect::Shot { firer, .. } if *firer == v))
            .count();
        seen.push(look(w, v, shots));
    }
    seen
}

/// The index of the touchdown tick: the first tick after which the unit is
/// no longer dropped, at height 0 and its jumpjet movement not moving.
fn touchdown(seen: &[Seen]) -> usize {
    seen.iter()
        .position(|s| !s.dropped && s.height == 0 && !s.moving)
        .expect("no touchdown")
}

/// A lifter of house 0 seizes VICT of house 1 (`computer`: a computer-
/// controlled house), pulls it over cell (3, 0) and is ordered on, which
/// drops it there over clear land (claims 12 and 13); the lifter then
/// leaves play, so that it cannot seize the unit again (L11 part e keeps
/// it away). Returns the world and the unit.
fn dropped_over_land(computer: bool, tweak: impl FnOnce(&mut Ruleset)) -> (World, ObjectId) {
    let mut w = world(tweak);
    w.houses[1].human = !computer;
    let l = spawn(&mut w, Kind::Vehicle, "LIFTER", 0, at(0, 0));
    let v = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(6, 0));
    beam(&mut w, l, v);
    assert!(lift::seized(&w, v));
    assert!(
        run_until(&mut w, 400, |w| w
            .obj(v)
            .is_some_and(|o| o.pos.cell() == at(3, 0))
            && height(w, v) > 300),
        "the victim did not pass over the cell"
    );
    glasnost_sim::econ::assign_destination(&mut w, l, Some(Target::Cell(at(0, 1))));
    assert!(lift::dropped(&w, v), "claim 13: dropped");
    glasnost_sim::objects::remove(&mut w, l);
    (w, v)
}

fn armed_victim(r: &mut Ruleset) {
    r.sim.vehicles.get_mut("VICT").unwrap().primary = Some("Gun".into());
}

// spec: S-0607 claim 20 (revision 14, when restoration is tested: at the end of each of the unit's turns, after its movement; at touchdown on a clear cell, claim 17's not-blocked row, the unit is restored at the end of that same turn; scenario L11 parts a and c) and claim 9 (revision 14, the first order after touchdown): a human-owned unit in guard, dropped over clear land, is seized through its fall and restored in its touchdown tick with no order; one move order in the next tick moves it with its own locomotor
#[test]
fn claim_20_restored_at_the_end_of_the_touchdown_turn() {
    let (mut w, v) = dropped_over_land(false, |_| {});
    let seen = watch(&mut w, v, 400);
    let t = touchdown(&seen);
    assert!(
        seen[..t].iter().all(|s| s.seized),
        "seized through the fall"
    );
    assert!(
        !seen[t].seized,
        "restored in the touchdown tick: {:?}",
        seen[t]
    );
    assert!(seen[t..].iter().all(|s| !s.seized), "and stays restored");
    assert_eq!(health(&w, v), 100, "claim 17: unhurt");
    assert!(
        w.obj(v).unwrap().jump.is_none(),
        "its own locomotor is back"
    );

    // L11 part a: the first move order after touchdown moves it.
    let (mut w, v) = dropped_over_land(false, |_| {});
    assert!(run_until(&mut w, 400, |w| !lift::dropped(w, v)
        && height(w, v) == 0
        && !jump::moving(w, v)));
    assert!(!lift::seized(&w, v), "restored at touchdown");
    let here = w.obj(v).unwrap().pos.cell();
    let goal = CellXY::new(here.x, here.y + 3);
    assert!(order_move(&mut w, v, goal));
    assert!(
        run_until(&mut w, 300, |w| w.obj(v).unwrap().pos.cell() == goal),
        "one order moves it"
    );
    assert!(!lift::seized(&w, v));
}

// spec: S-0607 claim 20 (revision 14, behaviour and house make no difference: a set-down unit in guard and one in hunt are restored at the same tick, for every house; scenario L11 part e, the control) and claim 6 (revision 14: the hunt queued at the seize is still current when the unit is restored)
#[test]
fn claim_20_hunter_and_guard_restored_at_the_same_tick() {
    let (mut w, v) = dropped_over_land(true, |_| {});
    let hunter = watch(&mut w, v, 400);
    let (mut w2, v2) = dropped_over_land(false, |_| {});
    let guard = watch(&mut w2, v2, 400);
    let (th, tg) = (touchdown(&hunter), touchdown(&guard));
    assert_eq!(hunter[th].tick, guard[tg].tick, "the same touchdown tick");
    let restored = |s: &[Seen]| s.iter().position(|s| !s.seized).unwrap();
    assert_eq!(restored(&hunter), th, "the hunter: in its touchdown tick");
    assert_eq!(restored(&guard), tg, "the guard: in its touchdown tick");
    assert!(hunter[th].hunt, "claim 6: hunt is still current");
    assert!(!guard[tg].hunt);
}

// spec: S-0607 claim 20 (revision 14, before restoration the unit's searches find no target, its firing check answering illegal, claim 10; after restoration it keeps hunt and is an ordinary hunter from its next hunt processing, S-0215 claim 27: it takes a target and fires) and the ordering consequence (the first search that can find one is its next hunt processing after the touchdown turn); scenario L11 part e runs 1 and 2: a computer house's armed unit dropped near an enemy takes no target and fires no shot through the grip, the fall and the touchdown tick, then takes one with hunt current and fires
#[test]
fn claim_20_a_restored_hunter_takes_a_target_and_fires() {
    let (mut w, v) = dropped_over_land(true, armed_victim);
    let enemy = spawn(&mut w, Kind::Vehicle, "HEAVY", 0, at(3, 2));
    let seen = watch(&mut w, v, 600);
    let t = touchdown(&seen);
    assert!(
        seen[..=t].iter().all(|s| !s.target && s.shots == 0),
        "no target and no shot through the fall and the touchdown tick"
    );
    let first_target = seen
        .iter()
        .position(|s| s.target)
        .expect("a restored hunter takes a target");
    assert!(first_target > t);
    assert!(seen[first_target].hunt, "hunt is current");
    let first_shot = seen
        .iter()
        .position(|s| s.shots > 0)
        .expect("a restored hunter fires");
    assert!(seen[first_shot].hunt, "hunt is current at the first shot");
    assert!(seen[t..].iter().all(|s| !s.seized));
    assert!(
        w.obj(enemy).is_none_or(|o| o.health < 100),
        "it hit the enemy"
    );
}

// spec: S-0607 claim 20 (revision 14, after restoration the unit can be seized again, claim 3) and claim 3 (a seize succeeds only on a unit that is not already seized); scenario L11 part e run 1b: a second lifter that fires at the unit k = 1, 5 and 30 ticks after its touchdown seizes it every time
#[test]
fn claim_20_a_restored_unit_can_be_seized_again() {
    for k in [1, 5, 30] {
        let (mut w, v) = dropped_over_land(true, |_| {});
        assert!(run_until(&mut w, 400, |w| !lift::dropped(w, v)
            && height(w, v) == 0
            && !jump::moving(w, v)));
        run(&mut w, k);
        assert!(!lift::seized(&w, v), "k = {k}: restored");
        let second = spawn(&mut w, Kind::Vehicle, "LIFTER", 0, at(-2, 0));
        beam(&mut w, second, v);
        assert!(lift::seized(&w, v), "k = {k}: seized again");
        assert!(lift::grips(&w, second, v), "k = {k}: by the second lifter");
    }
}

/// Turns every cell water except the lifter's (0, 0), so that the stop of
/// a wheeled unit finds no cell (S-0502 claim 17).
fn flood(w: &mut World) {
    let mut cells = Vec::new();
    for dx in -SIZE..SIZE {
        for dy in -SIZE..SIZE {
            let c = at(dx, dy);
            if c != at(0, 0) && w.cells.grid.get(c).is_some() {
                cells.push(c);
            }
        }
    }
    water(w, &cells);
}

// spec: S-0607 claim 20 (revision 14, units being erased: a seized vehicle is not restored while it is being erased, S-0215 claim 2, even when it is set down and not moving, neither at the end of a turn nor at a fall-back to rest; one that survives is restored at the end of its first turn after the erasure ends; scenario L11 part f) and claim 13 (a grip ended by its lifter with no cell for the stop: set down where it is, no movement point, not moving)
#[test]
fn claim_20_not_restored_while_being_erased() {
    let mut w = world(|_| {});
    let l = spawn(&mut w, Kind::Vehicle, "LIFTER", 0, at(0, 0));
    let v = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(6, 0));
    // An unarmed eraser stand-in far away: its lock makes no progress (S-0514 claim 7, D = 0).
    let eraser = spawn(&mut w, Kind::Vehicle, "VICT", 0, at(-15, -15));
    beam(&mut w, l, v);
    flood(&mut w);
    glasnost_sim::econ::assign_destination(&mut w, l, Some(Target::Cell(at(0, 1))));
    assert!(w.obj(v).unwrap().lift.set_down, "set down by its lifter");
    assert!(!jump::moving(&w, v), "not moving");
    assert_eq!(height(&w, v), 0);
    glasnost_sim::objects::remove(&mut w, l);
    w.temporal.locks.insert(
        v,
        glasnost_sim::temporal::Lock {
            leader: eraser,
            joiners: Vec::new(),
            amount: 1_000_000,
            was_off: false,
        },
    );
    run(&mut w, 30);
    assert!(lift::seized(&w, v), "not restored at the end of its turns");
    glasnost_sim::process::fall_back(&mut w, v, false);
    assert!(lift::seized(&w, v), "nor at a fall-back to rest");
    w.temporal.locks.remove(&v);
    run(&mut w, 1);
    assert!(
        !lift::seized(&w, v),
        "restored at the end of its first turn after the erasure"
    );
}

fn quirk_off(r: &mut Ruleset) {
    r.globals.quirks.jumpjet_no_cell_self_destruct = false;
}

/// The land tile of cell (0, 0), taken before it is flooded.
fn land_tile(w: &World) -> i32 {
    w.cells.grid.get(at(0, 0)).unwrap().tile
}

/// Makes `c` land again, with `tile`.
fn dry(w: &mut World, c: CellXY, tile: i32) {
    w.cells.grid.get_mut(c).unwrap().tile = tile;
    glasnost_sim::load::recalc_cells(w, &[c]);
}

// spec: S-0607 claim 20 (revision 14, the hovering set-down unit, design: restoration is not tested for it, neither at the end of a turn nor at a fall-back to rest; at the end of each of its turns it repeats the stop's search instead; a cell found, it flies there, lands and is restored at the end of the turn in which it lands) and claim 9 (revision 14: no cell found, it searches again at the next test, so it lands once a cell becomes free); S-0607 Changes for other specs (S-0502 Edge cases and J27 c: the retry at the end of each turn, not at a fall-back to rest); S-0502 Edge cases and J27 (revision 21: the retry at the end of each of its turns, S-0203, not at a fall-back to rest)
#[test]
fn claim_20_hovering_set_down_retries_at_the_end_of_each_turn() {
    let mut w = world(quirk_off);
    let l = spawn(&mut w, Kind::Vehicle, "LIFTER", 0, at(0, 0));
    let v = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(6, 0));
    let tile = land_tile(&w);
    beam(&mut w, l, v);
    flood(&mut w);
    assert!(run_until(&mut w, 400, |w| height(w, v) > 300));
    glasnost_sim::econ::assign_destination(&mut w, v, None);
    run(&mut w, 1);
    assert!(!lift::grips(&w, l, v), "the grip ends");
    glasnost_sim::objects::remove(&mut w, l);
    // The lifter's cell is the last land: flooded too, no cell is left.
    water(&mut w, &[at(0, 0)]);
    assert_eq!(jump::phase(&w, v), Some(Phase::Hovering), "it hovers");
    assert!(!jump::moving(&w, v));
    run(&mut w, 10);
    assert!(lift::seized(&w, v), "not restored while it hovers");
    assert!(!jump::moving(&w, v), "the retries find nothing");
    let c = w.obj(v).unwrap().pos.cell();
    dry(&mut w, c, tile);
    // A fall-back to rest neither retries nor restores.
    glasnost_sim::process::fall_back(&mut w, v, false);
    assert!(!jump::moving(&w, v), "no retry at a fall-back to rest");
    assert!(lift::seized(&w, v));
    // The end of its next turn retries, with no order given.
    run(&mut w, 1);
    assert!(jump::moving(&w, v), "the end-of-turn retry found the cell");
    assert!(lift::seized(&w, v), "not restored by the retry");
    let seen = watch(&mut w, v, 400);
    let t = touchdown(&seen);
    assert!(seen[..t].iter().all(|s| s.seized));
    assert!(!seen[t].seized, "restored in its landing tick");
}
