//! S-0607 revision 50 (Q-3119, #285): a lifter that rides an open-topped
//! transport is in play, so its seizing hit is tested as any other and its
//! wave lasts while it rides; a firer that has left play since the shot,
//! even if it entered play again, or has started to cloak since the shot,
//! seizes nothing; a lifter that leaves play while it grips ends the grip
//! and stays the dropper of the unit it drops. Synthetic data only.

#[path = "lift_common/mod.rs"]
mod lift_common;

use glasnost_sim::combat::fire;
use glasnost_sim::rules::Kind;
use glasnost_sim::world::{HouseId, ObjectId, Target, World};
use glasnost_sim::{cloak, lift, objects, transport};
use lift_common::*;

/// The lift ruleset with an open-topped transport (OPEN) and an infantry
/// type carrying the magnetic-beam weapon (MAGTROOP, the walking lifter).
fn rules_world() -> World {
    world(|r| {
        vehicle(r, "OPEN", |t| {
            t.strength = Some(1000);
            t.passengers = Some(4);
            t.size_limit = Some(d("2"));
            t.open_topped = Some(true);
        });
        infantry(r, "MAGTROOP", |t| t.primary = Some("MagBeam".into()));
    })
}

/// A walking lifter of house 0 aboard OPEN at the centre, and VICT of
/// house 1 five cells away.
fn riding() -> (World, ObjectId, ObjectId, ObjectId) {
    let mut w = rules_world();
    let t = spawn(&mut w, Kind::Vehicle, "OPEN", 0, at(0, 0));
    let m = spawn(&mut w, Kind::Infantry, "MAGTROOP", 0, at(0, 1));
    let v = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(5, 0));
    transport::board(&mut w, m, t);
    assert_eq!(transport::riding(&w, m), Some(t));
    (w, t, m, v)
}

/// Fires `firer`'s beam at `target` and runs, calling `between` once after
/// the shot, until the shot has detonated.
fn shot_then(w: &mut World, firer: ObjectId, target: ObjectId, between: impl FnOnce(&mut World)) {
    attack(w, firer, target);
    assert!(fire::shot(w, firer, 0, Target::Object(target)));
    assert!(!w.projectiles.is_empty(), "the shot flies");
    between(w);
    assert!(
        run_until(w, 60, |w| w.projectiles.is_empty()),
        "the beam did not detonate"
    );
}

// spec: S-0607 claim 2 (control: an on-map firer's seizing hit seizes an eligible target)
#[test]
fn an_on_map_walking_lifter_seizes() {
    let mut w = rules_world();
    let m = spawn(&mut w, Kind::Infantry, "MAGTROOP", 0, at(0, 0));
    let v = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(5, 0));
    beam(&mut w, m, v);
    assert!(lift::seized(&w, v));
    assert_eq!(lift::lifter_of(&w, v), Some(m));
}

// spec: S-0607 claim 2 (revision 50: a firer that was already a passenger of an open-topped transport when it fired is in play while it rides, S-0507 claim 14, so riding does not void its hit)
#[test]
fn an_open_topped_passenger_seizes() {
    let (mut w, _, m, v) = riding();
    beam(&mut w, m, v);
    assert!(lift::seized(&w, v), "the passenger's hit seizes");
    assert_eq!(lift::lifter_of(&w, v), Some(m));
    assert!(lift::grips(&w, m, v));
}

// spec: S-0607 claim 21 (revision 50: a firer that is already a passenger of an open-topped transport when it fires creates a wave as any other firer does, and only the ending conditions end it)
#[test]
fn a_riding_firers_wave_lasts_while_it_rides() {
    let (mut w, _, m, v) = riding();
    beam(&mut w, m, v);
    let wave = lift::wave_of(&w, m).expect("the riding firer's shot makes a wave");
    for _ in 0..5 {
        run(&mut w, 1);
        attack(&mut w, m, v);
        assert!(
            w.waves.get(&wave).is_some_and(|x| !x.ended),
            "the wave has not ended at tick {}",
            w.tick
        );
    }
}

// spec: S-0607 claims 7 and 12 (revision 50: riding and the transport's movement are not events of claim 12's table; the grip lasts while the transport moves and the unit keeps flying to the movement point fixed at the seize)
#[test]
fn the_grip_lasts_while_the_transport_moves() {
    let (mut w, t, m, v) = riding();
    beam(&mut w, m, v);
    assert!(lift::grips(&w, m, v));
    let start = w.obj(t).unwrap().pos;
    glasnost_sim::econ::assign_destination(&mut w, t, Some(Target::Cell(at(0, 6))));
    for _ in 0..60 {
        run(&mut w, 1);
        assert!(lift::grips(&w, m, v), "the grip lasts at tick {}", w.tick);
        if w.obj(t).unwrap().pos != start {
            break;
        }
    }
    assert_ne!(w.obj(t).unwrap().pos, start, "the transport moved");
}

// spec: S-0607 claim 2 (the firer has left play since the shot, even if it entered play again since, as a unit that boards an open-topped transport does, S-0203 claim 3)
#[test]
fn a_firer_that_boards_after_the_shot_seizes_nothing() {
    let mut w = rules_world();
    let t = spawn(&mut w, Kind::Vehicle, "OPEN", 0, at(0, 1));
    let m = spawn(&mut w, Kind::Infantry, "MAGTROOP", 0, at(0, 0));
    let v = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(5, 0));
    shot_then(&mut w, m, v, |w| transport::board(w, m, t));
    assert!(!lift::seized(&w, v), "boarded after the shot: no seize");
}

// spec: S-0607 claim 2 (the firer has left play since the shot, even if it entered play again since)
#[test]
fn a_firer_that_went_inside_and_came_out_seizes_nothing() {
    let mut w = rules_world();
    let m = spawn(&mut w, Kind::Infantry, "MAGTROOP", 0, at(0, 0));
    let v = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(5, 0));
    shot_then(&mut w, m, v, |w| {
        let pos = w.obj(m).unwrap().pos;
        objects::take_inside(w, m);
        objects::place(w, m, pos, Default::default());
        assert!(w.obj(m).unwrap().on_map, "back in play before the hit");
    });
    assert!(!lift::seized(&w, v), "left play since the shot: no seize");
}

// spec: S-0607 claim 2 (the firer has started to cloak since the shot)
#[test]
fn a_firer_that_started_to_cloak_seizes_nothing() {
    let mut w = rules_world();
    let m = spawn(&mut w, Kind::Infantry, "MAGTROOP", 0, at(0, 0));
    let v = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(5, 0));
    shot_then(&mut w, m, v, |w| {
        cloak::machine::grant_own_flag(w, m);
        let tick = w.tick;
        let o = w.obj_mut(m).unwrap();
        o.target = None;
        o.combat.rearm_until = tick;
        o.cloak.delay_until = tick;
        // spec: S-0516 claims 4 and 42 (a ready object with its own cloak flag starts cloaking in its ordinary cloak update)
        cloak::machine::turn(w, m);
        assert_eq!(
            w.obj(m).unwrap().cloak.state,
            cloak::CloakState::Cloaking,
            "the firer starts to cloak"
        );
        assert!(!w.projectiles.is_empty(), "before the hit");
    });
    assert!(!lift::seized(&w, v), "cloaking since the shot: no seize");
}

/// LIFTER of house 0 gripping VICT of house 1 high over the ground, with
/// OPEN of house 0 beside the lifter.
fn gripping_high() -> (World, ObjectId, ObjectId, ObjectId) {
    let mut w = rules_world();
    let t = spawn(&mut w, Kind::Vehicle, "OPEN", 0, at(0, 1));
    let l = spawn(&mut w, Kind::Vehicle, "LIFTER", 0, at(0, 0));
    let v = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(6, 0));
    beam(&mut w, l, v);
    assert!(lift::grips(&w, l, v));
    assert!(
        run_until(&mut w, 400, |w| height(w, v) > 300),
        "the victim rises"
    );
    (w, t, l, v)
}

// spec: S-0607 claims 12 and 14 (revision 50: a lifter that leaves play while it grips without being destroyed or removed, by going inside a transport, ends the grip at that moment and its target is cleared; the unit is dropped and keeps the lifter as its dropper)
#[test]
fn a_lifter_that_boards_while_gripping_drops_and_stays_the_dropper() {
    let (mut w, t, l, v) = gripping_high();
    transport::board(&mut w, l, t);
    assert!(!lift::grips(&w, l, v), "the grip ends at the boarding");
    assert_eq!(lift::lifter_of(&w, v), None);
    assert!(lift::dropped(&w, v), "dropped, being high");
    let o = w.obj(v).unwrap();
    assert_eq!(o.lift.dropper, Some(l), "the lifter stays the dropper");
    assert_eq!(o.lift.dropper_house, Some(HouseId(0)));
    assert_eq!(w.obj(l).unwrap().target, None, "the lifter's target is cleared");
    // spec: S-0607 claim 14 (the exception's dropper: the first sentence applies once it is removed while out of play)
    objects::remove(&mut w, l);
    let o = w.obj(v).unwrap();
    assert_eq!(o.lift.dropper, None, "removed: no dropper any more");
    assert_eq!(
        o.lift.dropper_house,
        Some(HouseId(0)),
        "the dropper house stays"
    );
}

// spec: S-0607 claim 14 (a dropper that leaves play before its dropped unit lands: the unit no longer has a dropper but keeps its dropper house)
#[test]
fn a_dropper_that_boards_before_the_landing_is_no_dropper() {
    let (mut w, t, l, v) = gripping_high();
    glasnost_sim::econ::assign_destination(&mut w, l, Some(Target::Cell(at(0, -1))));
    assert!(lift::dropped(&w, v), "a destination ends the grip with a drop");
    assert_eq!(w.obj(v).unwrap().lift.dropper, Some(l));
    transport::board(&mut w, l, t);
    let o = w.obj(v).unwrap();
    assert_eq!(o.lift.dropper, None, "the dropper has left play");
    assert_eq!(
        o.lift.dropper_house,
        Some(HouseId(0)),
        "the dropper house stays"
    );
}
