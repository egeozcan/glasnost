//! Claim tests for turn rates and turning (S-0206), on synthetic data.
//!
//! Turret control (where a turret points, the firing check's facing error,
//! spinning turrets), the hovering and jumpjet locomotors and the track-class
//! body-turn stop are not implemented in the simulation yet, so their claims
//! have no tests here.

mod common;

use common::{FOOT, Scene, WHEEL, cell, objects_of, open_orders, order_move, unit_type};
use glasnost_num::geom::delta;
use glasnost_rules::sim::BuildingType;
use glasnost_sim::rules::Kind;
use glasnost_sim::tick;
use glasnost_sim::world::{ObjectId, World};

/// A scene with one vehicle of type TESTCAR (wheel class, invented speed
/// 40, turn-rate field `rot`) at the map centre with map facing value `v`.
fn car_scene(rot: i32, v: i32) -> Scene {
    let mut s = Scene::new();
    s.vehicle("TESTCAR", unit_type(WHEEL, 40, rot));
    s.place_unit(0, "TESTCAR", cell(0, 0), v);
    s
}

fn only(w: &World, kind: Kind) -> ObjectId {
    objects_of(w, kind)[0]
}

/// One tick of a drive log: tick number, body facing, drive turn (t0, Θ).
type LogEntry = (i32, u16, Option<(i32, i32)>);

/// Orders the vehicle 6 cells map east and runs until it first leaves its
/// start position; returns the tick number of that tick and a log of
/// (tick number, body facing, drive turn) for every tick run.
fn drive_east(mut w: World) -> (i32, Vec<LogEntry>) {
    open_orders(&mut w);
    let id = only(&w, Kind::Vehicle);
    let start = w.obj(id).unwrap().pos;
    order_move(&mut w, id, cell(6, 0));
    let mut log = Vec::new();
    for _ in 0..400 {
        tick::step(&mut w);
        let t = w.tick - 1;
        let o = w.obj(id).unwrap();
        log.push((t, o.body.value(t), o.drive.as_ref().and_then(|d| d.turn)));
        if o.pos != start {
            return (t, log);
        }
    }
    panic!("the vehicle never moved");
}

// spec: S-0206 claim 7 (vehicle and building turn rates come from their own turn-rate field)
// spec: S-0206 claim 15 (infantry turn at 127 whatever their field says)
#[test]
fn turn_rate_by_object_kind() {
    let mut s = Scene::new();
    s.vehicle("TESTCAR", unit_type(WHEEL, 40, 9));
    s.infantry("TESTMAN", unit_type(FOOT, 4, 3));
    s.register("buildings", "TESTPOST");
    s.ruleset.sim.buildings.insert(
        "TESTPOST".into(),
        BuildingType {
            strength: Some(500),
            turret: Some(true),
            rot: Some(11),
            ..Default::default()
        },
    );
    let r = s.sim_rules();
    assert_eq!(r.vehicles[0].turn_rate, 9);
    assert_eq!(r.infantry[0].turn_rate, 127);
    assert_eq!(r.buildings[0].turn_rate, 11);
}

// spec: S-0206 claim 7 (a vehicle body turns at the type's turn rate)
// spec: S-0206 claim 12 (a turn in place: no advance while turning, the track begins in tick t0 + Θ)
#[test]
fn turn_in_place_before_a_step_takes_theta_ticks() {
    // Map facing value 0 (facing 0) to map east (16384) at ω = 1: Θ = 64.
    let (moved_at, log) = drive_east(car_scene(1, 0).build());
    let (t0, dur) = log
        .iter()
        .find_map(|e| e.2)
        .expect("a turn in place was given");
    assert_eq!(dur, 64);
    // The body follows F1 at ω = 1 during the turn and reaches 16384 at t0 + Θ.
    for &(t, f, _) in &log {
        if t >= t0 && t < t0 + dur {
            assert_ne!(f, 16384, "tick {t}");
            assert_eq!(f, (16384 - 256 * (t0 + dur - t)) as u16);
        }
    }
    let at_end = log.iter().find(|e| e.0 == t0 + dur).unwrap();
    assert_eq!(at_end.1, 16384);
    // It makes progress in the tick its body reaches the step's facing.
    assert_eq!(moved_at, t0 + dur);
}

// spec: S-0206 claim 12 (a turn in place delays the step by max(Θ, 1) ticks against a vehicle that needs no turn)
#[test]
fn turn_in_place_delay_against_a_vehicle_facing_the_step() {
    // Facing map east already (map facing value 64 = facing 16384).
    let (reference, ref_log) = drive_east(car_scene(1, 64).build());
    assert!(ref_log.iter().all(|e| e.2.is_none()));
    // ω = 1, from facing 0: Θ = 64.
    let (slow, _) = drive_east(car_scene(1, 0).build());
    assert_eq!(slow - reference, 64);
    // ω = 127, from facing 8192 (map value 32): |Δ| = 8192 < 256 × 127, Θ = 0.
    let (fast, log) = drive_east(car_scene(127, 32).build());
    let (t0, dur) = log.iter().find_map(|e| e.2).unwrap();
    assert_eq!(dur, 0);
    // The body has the new facing in tick t0, but the track begins in t0 + 1.
    assert_eq!(log.iter().find(|e| e.0 == t0).unwrap().1, 16384);
    assert_eq!(fast, t0 + 1);
    assert_eq!(fast - reference, 1);
    // ω = 0 (field 0) also completes at once and delays by 1.
    let (zero, _) = drive_east(car_scene(0, 0).build());
    assert_eq!(zero - reference, 1);
}

// spec: S-0206 claim 12 (while driving, the body facing is set at once to the track's facing, not turned)
#[test]
fn body_facing_is_set_at_once_while_driving() {
    let mut w = car_scene(1, 64).build();
    open_orders(&mut w);
    let id = only(&w, Kind::Vehicle);
    // A destination off the 8 directions makes the path change direction.
    order_move(&mut w, id, cell(6, 4));
    let mut prev: Option<u16> = None;
    let mut big_change = false;
    for _ in 0..300 {
        tick::step(&mut w);
        let t = w.tick - 1;
        let o = w.obj(id).unwrap();
        let on_track = o.drive.as_ref().is_some_and(|d| d.track.is_some());
        if on_track {
            // Set at once: never turning while on a track.
            assert!(!o.body.turning(t), "tick {t}");
            let f = o.body.value(t);
            if let Some(p) = prev
                && delta(p, f).abs() > 256
            {
                // ω = 1 would allow at most 256 facing units per tick.
                big_change = true;
            }
            prev = Some(f);
        } else {
            prev = None;
        }
        if o.pos.cell() == cell(6, 4) && o.dest.is_none() {
            break;
        }
    }
    assert!(big_change, "the track's facing changed faster than ω = 1");
}

// spec: S-0206 claim 4 (a turn takes the same number of ticks at every game speed)
#[test]
fn turning_counts_ticks_not_game_speed() {
    let mut logs = Vec::new();
    for speed in [0u8, 6] {
        let mut s = car_scene(2, 0);
        s.setup.game_speed = speed;
        logs.push(drive_east(s.build()));
    }
    assert_eq!(logs[0], logs[1]);
    let (t0, dur) = logs[0].1.iter().find_map(|e| e.2).unwrap();
    // 16384 / (256 × 2) = 32 ticks.
    assert_eq!(dur, 32);
    assert_eq!(logs[0].0, t0 + 32);
}

// spec: S-0206 claim 4 (no progress while no ticks run)
#[test]
fn turning_makes_no_progress_without_ticks() {
    let mut w = car_scene(1, 0).build();
    open_orders(&mut w);
    let id = only(&w, Kind::Vehicle);
    order_move(&mut w, id, cell(6, 0));
    let mut n = 0;
    while !w.obj(id).unwrap().body.turning(w.tick) {
        tick::step(&mut w);
        n += 1;
        assert!(n < 50, "no turn began");
    }
    let t = w.tick;
    let before = w.obj(id).unwrap().body.value(t);
    let snapshot = w.obj(id).unwrap().body;
    // Without a tick the tick number and the facing stay put.
    assert_eq!(w.tick, t);
    assert_eq!(w.obj(id).unwrap().body, snapshot);
    assert_eq!(w.obj(id).unwrap().body.value(w.tick), before);
    // One tick later the facing has moved by one step.
    tick::step(&mut w);
    assert_ne!(w.obj(id).unwrap().body.value(w.tick), before);
}

// spec: S-0206 claim 15 (an infantry body turn shorter than 32512 completes at once; a larger one takes 1 tick)
#[test]
fn infantry_turns_at_127() {
    // Fields 1 and 0 would turn slowly or at once; infantry ignore them.
    for field in [1, 0] {
        let mut s = Scene::new();
        s.infantry("TESTMAN", unit_type(FOOT, 4, field));
        s.place_infantry(0, "TESTMAN", cell(0, 0), 1);
        let mut w = s.build();
        open_orders(&mut w);
        let id = only(&w, Kind::Infantry);
        let (mut saw_once, mut saw_one_tick) = (false, false);
        // Walk map east, then reverse map west: the reversal is about a
        // half turn.
        for dest in [cell(4, 0), cell(-4, 0)] {
            order_move(&mut w, id, dest);
            let mut prev = w.obj(id).unwrap().body;
            for _ in 0..150 {
                let t = w.tick;
                tick::step(&mut w);
                let b = w.obj(id).unwrap().body;
                if b.target != prev.target {
                    let d = delta(prev.value(t), b.target).abs();
                    let want = if d >= 32512 { 1 } else { 0 };
                    assert_eq!(b.duration, want, "turn of {d}, field {field}");
                    saw_once |= want == 0;
                    saw_one_tick |= want == 1;
                }
                prev = b;
            }
        }
        assert!(saw_once && saw_one_tick, "field {field}");
    }
}
