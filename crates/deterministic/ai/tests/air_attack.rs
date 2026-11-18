//! #434 (design: own AI): an AI house's unit hit by an air unit it cannot
//! hit gets away, as a person moves it: toward the nearest unit or defence
//! of the house that can hit the attacker, else back toward home when it
//! is out in the field, else out of the attacker's reach; and a unit of the
//! house nearby that can hit the attacker is sent at it. The simulation
//! leaves such a unit standing (S-0215 claims 18 and 19 with S-0306 claim
//! 23: its gun cannot fire at the air, so it does not retaliate, and an AI
//! house is human-controlled in our engine, so it scatters only with the
//! player-scatter setting). Beta's GUNR is hit by Alpha's hovering DISC;
//! FLAKT has an anti-air gun. Synthetic data only.

mod air_common;

use air_common::*;
use glasnost_sim::orders::Order;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::world::Target;

/// The disc's gun's reach in whole cells, rounded up (S-0307: the fixture
/// gun's range is 5.5 cells).
const REACH: i64 = 6; // nocite: the fixture gun's range, rounded up

// design: own AI (#434); S-0306 claim 23 (the ground gun cannot fire at the airborne disc): out in the field, a unit hit by an air unit it cannot hit goes to the nearest unit of the house that can hit it.
#[test]
fn a_ground_unit_hit_from_the_air_goes_to_anti_air_cover() {
    let mut w = world();
    let x = spawn(&mut w, "GUNR", Kind::Vehicle, 1, -20, 30);
    let flak = spawn(&mut w, "FLAKT", Kind::Vehicle, 1, -30, 22);
    let d = disc(&mut w, -18, 30);
    let at = w.obj(flak).unwrap().pos.cell();
    let mut br = brain(&w);
    shoot(&mut w, d, x);
    let log = run(&mut w, &mut br, x, 60, |_| {});
    let sent = log
        .iter()
        .flat_map(|(_, o)| o.iter().filter_map(move_cell))
        .collect::<Vec<_>>();
    assert!(
        sent.iter().any(|&c| dist2(c, at) <= 9),
        "sent to {sent:?}, cover at {at:?}: {log:?}"
    );
}

// design: own AI (#434): the house's unit that can hit the attacker, out of the reach of the house's other orders, is sent at it.
#[test]
fn an_anti_air_unit_nearby_is_sent_at_the_attacker() {
    let mut w = world();
    let x = spawn(&mut w, "GUNR", Kind::Vehicle, 1, -20, 30);
    let flak = spawn(&mut w, "FLAKT", Kind::Vehicle, 1, -30, 22);
    let d = disc(&mut w, -18, 30);
    let mut br = brain(&w);
    shoot(&mut w, d, x);
    let log = run(&mut w, &mut br, flak, 60, |_| {});
    let attacks = log.iter().flat_map(|(_, o)| o.iter()).any(|o| {
        matches!(o, Order::Behaviour {
            behaviour: Behaviour::Attack,
            target: Some(Target::Object(t)),
            ..
        } if *t == d)
    });
    assert!(attacks, "{log:?}");
}

// design: own AI (#434): out in the field with no cover, it goes back toward home.
#[test]
fn a_ground_unit_hit_from_the_air_in_the_field_goes_home() {
    let mut w = world();
    let x = spawn(&mut w, "GUNR", Kind::Vehicle, 1, -20, 30);
    let d = disc(&mut w, -18, 30);
    let h = home(&w);
    let mut br = brain(&w);
    shoot(&mut w, d, x);
    let log = run(&mut w, &mut br, x, 60, |_| {});
    let sent = log
        .iter()
        .flat_map(|(_, o)| o.iter().filter_map(move_cell))
        .collect::<Vec<_>>();
    assert!(
        sent.iter().any(|&c| dist2(c, h) <= 36),
        "sent to {sent:?}, home {h:?}: {log:?}"
    );
}

// design: own AI (#434); S-0307 (the attacker's weapon range): at home with no cover, it moves out of the attacker's reach.
#[test]
fn a_ground_unit_hit_from_the_air_at_home_moves_out_of_reach() {
    let mut w = world();
    let x = spawn(&mut w, "GUNR", Kind::Vehicle, 1, 6, 20);
    let d = disc(&mut w, 8, 20);
    let mut br = brain(&w);
    shoot(&mut w, d, x);
    let log = run(&mut w, &mut br, x, 60, |_| {});
    let dc = w.obj(d).unwrap().pos.cell();
    let sent = log
        .iter()
        .flat_map(|(_, o)| o.iter().filter_map(move_cell))
        .collect::<Vec<_>>();
    assert!(
        sent.iter().any(|&c| dist2(c, dc) > REACH * REACH),
        "sent to {sent:?}, the disc at {dc:?}: {log:?}"
    );
}

// design: own AI (#434): while it gets away and the attacker keeps firing whenever it is in reach, the house does not order the unit at every think.
#[test]
fn a_unit_getting_away_is_not_ordered_at_every_think() {
    let mut w = world();
    let x = spawn(&mut w, "GUNR", Kind::Vehicle, 1, 6, 20);
    let d = disc(&mut w, 8, 20);
    let mut br = brain(&w);
    let mut n = 0;
    let log = run(&mut w, &mut br, x, 180, |w| {
        if n % 40 == 0 && in_reach(w, d, x) {
            shoot(w, d, x);
        }
        n += 1;
    });
    let ordered = log.iter().filter(|(_, o)| !o.is_empty()).count();
    assert!(ordered >= 1, "it is ordered away: {log:?}");
    assert!(ordered <= 2, "{ordered} of {} thinks: {log:?}", log.len());
}

// design: own AI (#434): a unit of another house (a person's) is never ordered by the AI house's brain.
#[test]
fn a_unit_of_another_house_is_never_ordered() {
    let mut w = world();
    let x = spawn(&mut w, "GUNR", Kind::Vehicle, 0, 6, 20);
    let d = spawn(&mut w, "DISC", Kind::Vehicle, 1, 8, 20);
    let (t, stale) = (w.tick, w.rules().quirks.stale_suspended_revert);
    w.obj_mut(d).unwrap().beh.force(Behaviour::Sleep, t, stale);
    glasnost_sim::jump::start_in_flight(&mut w, d, glasnost_sim::jump::Phase::Hovering, HOVER);
    glasnost_sim::testkit::run(&mut w, 2);
    let mut br = brain(&w);
    shoot(&mut w, d, x);
    let log = run(&mut w, &mut br, x, 60, |_| {});
    assert!(log.iter().all(|(_, o)| o.is_empty()), "{log:?}");
}
