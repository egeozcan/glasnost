//! M-0015 (TELE against BFRT, watched per unit; M8 step A3 intake 40):
//! synthetic tests of the magnetic-lift rules the record's facts rest on.
//! Synthetic data only (the lift fixture of `lift_common`); no stock value.

#[path = "lift_common/mod.rs"]
mod lift_common;

use glasnost_sim::combat::effects::Effect;
use glasnost_sim::lift;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::world::{ObjectId, Target, World};
use lift_common::*;

/// The victim type VICT armed with the fixture's gun. The tests make its
/// house 1 computer-controlled, as the arena's test houses are (glossary).
fn armed_victim(r: &mut glasnost_rules::schema::Ruleset) {
    r.sim.vehicles.get_mut("VICT").unwrap().primary = Some("Gun".into());
}

/// Steps `n` ticks and returns the shots `firer` made in them.
fn shots_by(w: &mut World, firer: ObjectId, n: i32) -> usize {
    let mut shots = 0;
    for _ in 0..n {
        glasnost_sim::tick::step(w);
        shots += std::mem::take(&mut w.effects)
            .iter()
            .filter(|e| matches!(e, Effect::Shot { firer: f, .. } if *f == firer))
            .count();
    }
    shots
}

// spec: S-0607 claims 6, 10, 17 and 20, S-0215 claim 27, S-0309 table C (firing check row); measure: M-0015 (Q-2035): a seized unit of a computer-controlled house, dropped over clear land and set down unhurt, has *hunt* current (claim 6); its hunt search finds nothing because its firing check answers illegal for every target (claim 10), so hunt never falls back to rest (claim 27) and claim 20 never restores it: it stays seized, takes no target and fires no shot, with an enemy in range. Control: the same unit unseized, in hunt, fires at that enemy
#[test]
fn m0015_a_set_down_computer_unit_in_hunt_stays_seized_and_never_fires() {
    let mut w = world(armed_victim);
    w.houses[1].human = false;
    let l = spawn(&mut w, Kind::Vehicle, "LIFTER", 0, at(0, 0));
    let v = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(6, 0));
    let enemy = spawn(&mut w, Kind::Vehicle, "HEAVY", 0, at(3, 2));
    beam(&mut w, l, v);
    assert!(lift::seized(&w, v));
    assert!(
        run_until(&mut w, 400, |w| w
            .obj(v)
            .is_some_and(|o| o.pos.cell() == at(3, 0))
            && height(w, v) > 300),
        "the victim did not pass over the cell"
    );
    // The lifter accepts a destination: the grip ends with a drop (claims 12 and 13).
    glasnost_sim::econ::assign_destination(&mut w, l, Some(Target::Cell(at(0, 1))));
    assert!(lift::dropped(&w, v));
    assert!(
        run_until(&mut w, 400, |w| !lift::dropped(w, v)
            && height(w, v) == 0
            && !glasnost_sim::jump::moving(w, v)),
        "the victim did not touch down"
    );
    assert_eq!(health(&w, v), 100, "claim 17: unhurt on clear land");
    assert!(lift::seized(&w, v), "claim 17: set down, still seized");
    let shots = shots_by(&mut w, v, 600);
    let o = w.obj(v).unwrap();
    assert_eq!(o.beh.current, Some(Behaviour::Hunt), "claim 6: hunt");
    assert!(lift::seized(&w, v), "claim 20: never restored in hunt");
    assert!(o.target.is_none(), "claim 10: the search takes no target");
    assert_eq!(shots, 0, "claim 10: no shot");
    assert!(w.obj(enemy).is_some() && w.obj(l).is_some());

    // Control: the same unit, never seized, hunting from the same cell.
    let mut w = world(armed_victim);
    w.houses[1].human = false;
    let v = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(3, 0));
    spawn(&mut w, Kind::Vehicle, "HEAVY", 0, at(3, 2));
    let t = w.tick;
    let stale = w.rules().quirks.stale_suspended_revert;
    w.obj_mut(v).unwrap().beh.force(Behaviour::Hunt, t, stale);
    assert!(
        shots_by(&mut w, v, 600) > 0,
        "control: an unseized hunter fires"
    );
}

// spec: S-0607 claims 16 to 18 and quirks.lift_impact_double_credit (on); measure: M-0015 (the landing crush): a unit dropped onto a cell holding a grounded vehicle of its own house crashes; the first impact detonation (crushing warhead, its dropper as attacker, ordinary area damage) kills it and the vehicle below; the falling unit is credited twice, the other once: 3 kills to the dropper's house for 2 units
#[test]
fn m0015_landing_on_a_grounded_vehicle_of_its_own_house_kills_both_three_credits() {
    let mut w = world(|r| {
        // F2 with the type's strength and a multiplier of 1: the falling
        // damage equals VICT's strength.
        r.sim.general.falling_damage_multiplier = Some(d("1.0"));
        r.sim.general.current_strength_damage = Some(false);
        r.globals.quirks.lift_impact_double_credit = true;
    });
    let l = spawn(&mut w, Kind::Vehicle, "LIFTER", 0, at(0, 0));
    let v = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(6, 0));
    beam(&mut w, l, v);
    assert!(lift::seized(&w, v));
    // The grounded vehicle of the falling unit's own house on the cell.
    let below = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(3, 0));
    assert!(
        run_until(&mut w, 400, |w| w
            .obj(v)
            .is_some_and(|o| o.pos.cell() == at(3, 0))
            && height(w, v) > 300),
        "the victim did not pass over the cell"
    );
    w.effects.clear();
    let kills_before = w.houses[0].combat.kills;
    glasnost_sim::econ::assign_destination(&mut w, l, Some(Target::Cell(at(0, 1))));
    let mut hits = Vec::new();
    for _ in 0..400 {
        if w.obj(v).is_none() {
            break;
        }
        glasnost_sim::tick::step(&mut w);
        for e in std::mem::take(&mut w.effects) {
            if let Effect::Hit { id, attacker, .. } = e {
                hits.push((id, attacker));
            }
        }
    }
    assert!(w.obj(v).is_none(), "the falling unit leaves play");
    assert!(
        w.obj(below).is_none_or(|o| o.health <= 0),
        "the vehicle below is killed"
    );
    run(&mut w, 5);
    assert!(w.obj(below).is_none(), "the vehicle below leaves play");
    assert!(
        hits.iter().any(|&(id, a)| id == below && a == Some(l)),
        "the vehicle below is hit with the dropper as attacker"
    );
    assert!(
        hits.iter().all(|&(_, a)| a == Some(l)),
        "every damage event names the dropper"
    );
    assert_eq!(
        w.houses[0].combat.kills - kills_before,
        3,
        "3 credits for 2 units"
    );
    assert_eq!(w.houses[1].combat.losses, 3);
}
