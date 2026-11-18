//! #434 (design: own AI): an AI house's unit hit by an air unit it cannot
//! hit gets away, as a person moves it: toward the nearest unit or defence
//! of the house that can hit the attacker, else back toward home when it
//! is out in the field, else out of the attacker's reach; and a unit of the
//! house nearby that can hit the attacker is sent at it. The simulation
//! leaves such a unit standing (S-0215 claims 18 and 19 with S-0306 claim
//! 23: its gun cannot fire at the air, so it does not retaliate, and an AI
//! house is human-controlled in our engine, so it scatters only with the
//! player-scatter setting). Beta's GUNR is hit by Alpha's hovering DISC;
//! FLAKT has an anti-air gun. Each test compares the run in which the disc
//! shoots with the same run without the shot, so that only the orders the
//! shot brings count (the base gives idle units moves of its own).
//! Synthetic data only.

mod air_common;

use air_common::*;
use glasnost_map::CellXY;
use glasnost_sim::orders::Order;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::world::{ObjectId, Target, World};

/// The disc's gun's reach in whole cells, rounded up (S-0307: the fixture
/// gun's range is 5.5 cells).
const REACH: i64 = 6; // nocite: the fixture gun's range, rounded up

/// Where the unit stands out in the field, from Beta's home: more than 30
/// cells away (the brain's recall distance), on the usable area of the
/// synthetic map (S-0009 claim 4) and away from Alpha's start.
const FIELD: (i32, i32) = (-22, 22); // nocite: a synthetic scene

/// Where FLAKT stands, from Beta's home: some fourteen cells from
/// [`FIELD`], and more than the script's ten engage cells from the disc,
/// so that only the shot sends it.
const FLAK: (i32, i32) = (-33, 13); // nocite: a synthetic scene

/// Where the unit stands near home, from Beta's home: within 30 cells.
const NEAR_HOME: (i32, i32) = (4, -16); // nocite: a synthetic scene

/// The disc hovers two cells east of the unit it shoots.
const OVER: i32 = 2; // nocite: a synthetic scene

/// A scene: the world, Beta's unit the disc shoots, the disc, and Beta's
/// FLAKT if there is one.
struct Scene {
    w: World,
    x: ObjectId,
    d: ObjectId,
    flak: Option<ObjectId>,
}

/// The field scene: Beta's GUNR out in the field under the disc, and with
/// `flak` Beta's FLAKT near by.
fn field(flak: bool) -> Scene {
    let mut w = world();
    let x = spawn(&mut w, "GUNR", Kind::Vehicle, 1, FIELD.0, FIELD.1);
    let flak = flak.then(|| spawn(&mut w, "FLAKT", Kind::Vehicle, 1, FLAK.0, FLAK.1));
    let d = disc(&mut w, FIELD.0 + OVER, FIELD.1);
    Scene { w, x, d, flak }
}

/// The home scene: Beta's GUNR near home under the disc.
fn near_home() -> Scene {
    let mut w = world();
    let x = spawn(&mut w, "GUNR", Kind::Vehicle, 1, NEAR_HOME.0, NEAR_HOME.1);
    let d = disc(&mut w, NEAR_HOME.0 + OVER, NEAR_HOME.1);
    Scene {
        w,
        x,
        d,
        flak: None,
    }
}

/// The unit the disc shoots.
fn victim(s: &Scene) -> ObjectId {
    s.x
}

/// Beta's FLAKT.
fn flak_of(s: &Scene) -> ObjectId {
    s.flak.expect("FLAKT is in the scene")
}

/// The move cells of a log.
fn moves(log: &Log) -> Vec<CellXY> {
    log.iter()
        .flat_map(|(_, o)| o.iter().filter_map(move_cell))
        .collect()
}

/// Beta's orders to `of` over `ticks` ticks of `scene`, with the disc's
/// shot at the unit first when `shot`.
fn log_of(scene: Scene, of: fn(&Scene) -> ObjectId, shot: bool, ticks: i32) -> Log {
    let id = of(&scene);
    let Scene { mut w, x, d, .. } = scene;
    let mut br = brain(&w);
    if shot {
        shoot(&mut w, d, x);
    }
    run(&mut w, &mut br, id, ticks, |_| {})
}

/// The moves Beta gives the unit of `scene` for the shot: those of the
/// shot run that the run without it does not give, with the shot run's
/// log.
fn moves_for_shot(scene: impl Fn() -> Scene, ticks: i32) -> (Vec<CellXY>, Log) {
    let quiet = moves(&log_of(scene(), victim, false, ticks));
    let log = log_of(scene(), victim, true, ticks);
    let fresh = moves(&log)
        .into_iter()
        .filter(|c| !quiet.contains(c))
        .collect();
    (fresh, log)
}

/// Whether `log` holds an attack order on `d`.
fn attacks(log: &Log, d: ObjectId) -> bool {
    log.iter().flat_map(|(_, o)| o.iter()).any(|o| {
        matches!(o, Order::Behaviour {
            behaviour: Behaviour::Attack,
            target: Some(Target::Object(t)),
            ..
        } if *t == d)
    })
}

// design: own AI (#434); S-0306 claim 23 (the ground gun cannot fire at the airborne disc): out in the field, a unit hit by an air unit it cannot hit goes to the nearest unit of the house that can hit it.
#[test]
fn a_ground_unit_hit_from_the_air_goes_to_anti_air_cover() {
    let s = field(true);
    let at = s.w.obj(flak_of(&s)).unwrap().pos.cell();
    let (sent, log) = moves_for_shot(|| field(true), 60);
    assert!(
        sent.iter().any(|&c| dist2(c, at) <= 9),
        "sent to {sent:?}, cover at {at:?}: {log:?}"
    );
}

// design: own AI (#434): the house's unit that can hit the attacker, out of the reach of the house's other orders, is sent at it.
#[test]
fn an_anti_air_unit_nearby_is_sent_at_the_attacker() {
    let d = field(true).d;
    let quiet = log_of(field(true), flak_of, false, 60);
    assert!(!attacks(&quiet, d), "only the shot sends it: {quiet:?}");
    let log = log_of(field(true), flak_of, true, 60);
    assert!(attacks(&log, d), "{log:?}");
}

// design: own AI (#434): out in the field with no cover, it goes back toward home.
#[test]
fn a_ground_unit_hit_from_the_air_in_the_field_goes_home() {
    let s = field(false);
    let h = home(&s.w);
    let xc = s.w.obj(s.x).unwrap().pos.cell();
    assert!(dist2(xc, h) > 30 * 30, "the unit stands out in the field"); // nocite: the brain's recall distance
    let (sent, log) = moves_for_shot(|| field(false), 60);
    assert!(
        sent.iter().any(|&c| dist2(c, h) <= 36),
        "sent to {sent:?}, home {h:?}: {log:?}"
    );
}

// design: own AI (#434); S-0307 (the attacker's weapon range): at home with no cover, it moves out of the attacker's reach.
#[test]
fn a_ground_unit_hit_from_the_air_at_home_moves_out_of_reach() {
    let s = near_home();
    let dc = s.w.obj(s.d).unwrap().pos.cell();
    let (sent, log) = moves_for_shot(near_home, 60);
    assert!(
        sent.iter().any(|&c| dist2(c, dc) > REACH * REACH),
        "sent to {sent:?}, the disc at {dc:?}: {log:?}"
    );
}

// design: own AI (#434): while it gets away and the attacker keeps firing whenever it is in reach, the house does not order the unit at every think.
#[test]
fn a_unit_getting_away_is_not_ordered_at_every_think() {
    let quiet = moves(&log_of(near_home(), victim, false, 180));
    let Scene { mut w, x, d, .. } = near_home();
    let mut br = brain(&w);
    let mut n = 0;
    let log = run(&mut w, &mut br, x, 180, |w| {
        if n % 40 == 0 && in_reach(w, d, x) {
            shoot(w, d, x);
        }
        n += 1;
    });
    let fresh = moves(&log).into_iter().any(|c| !quiet.contains(&c));
    assert!(fresh, "it is ordered away: {log:?}");
    let ordered = log.iter().filter(|(_, o)| !o.is_empty()).count();
    assert!(ordered <= 2, "{ordered} of {} thinks: {log:?}", log.len());
}

// design: own AI (#434): a unit of another house (a person's) is never ordered by the AI house's brain.
#[test]
fn a_unit_of_another_house_is_never_ordered() {
    let mut w = world();
    let x = spawn(&mut w, "GUNR", Kind::Vehicle, 0, NEAR_HOME.0, NEAR_HOME.1);
    let d = spawn(
        &mut w,
        "DISC",
        Kind::Vehicle,
        1,
        NEAR_HOME.0 + OVER,
        NEAR_HOME.1,
    );
    let (t, stale) = (w.tick, w.rules().quirks.stale_suspended_revert);
    w.obj_mut(d).unwrap().beh.force(Behaviour::Sleep, t, stale);
    glasnost_sim::jump::start_in_flight(&mut w, d, glasnost_sim::jump::Phase::Hovering, HOVER);
    glasnost_sim::testkit::run(&mut w, 2);
    let mut br = brain(&w);
    shoot(&mut w, d, x);
    let log = run(&mut w, &mut br, x, 60, |_| {});
    assert!(log.iter().all(|(_, o)| o.is_empty()), "{log:?}");
}
