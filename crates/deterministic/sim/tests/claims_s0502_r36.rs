//! Claim tests of S-0502 r36 claim 18 (the answers to Q-3281 and Q-3282):
//! the update that enters *crashing* runs only the preceding state's
//! physics and then crashing's decisions (scenario J12's balloon-hover runs
//! (a) near the cell centre, (a) in a speed band, and (e)); a killed unit
//! whose nearby-cell search finds no cell keeps its flight state, movement
//! point and commitment under both settings of
//! `quirks.jumpjet_no_cell_self_destruct`. Synthetic rules (`jump_common`,
//! plus BALCRASH, the balloon-hover vehicle with the crash flag); every name
//! and number is invented.

mod jump_common;

use glasnost_map::CellXY;
use glasnost_sim::combat::damage::{self, Hit};
use glasnost_sim::jump::{self, JumpState, Phase};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit;
use glasnost_sim::world::{Layer, ObjectId, Pos, Target, World};
use jump_common::*;

fn j(w: &World, id: ObjectId) -> &JumpState {
    w.obj(id).unwrap().jump.as_ref().unwrap()
}

fn kill(w: &mut World, id: ObjectId) {
    let health = w.obj(id).unwrap().health;
    damage::apply(
        w,
        id,
        Hit {
            amount: health * 10,
            warhead: None,
            distance: 0,
            attacker: None,
            house: None,
            ignore_defences: true,
            no_escape: false,
        },
    );
}

fn quirk(w: &mut World, f: impl FnOnce(&mut glasnost_rules::quirks::Quirks)) {
    let mut rules = (*w.rules_arc()).clone();
    f(&mut rules.quirks);
    w.rules = Some(std::sync::Arc::new(rules));
}

fn block_every_cell(w: &mut World) {
    let cells: Vec<CellXY> = w.cells.grid.iter().map(|(c, _)| c).collect();
    for c in cells {
        w.cells.claim_vehicle(c, Layer::Ground, true);
    }
}

/// A game with BALCRASH: BALLOON (balloon-hover, mobile fire, speed field
/// 14, jumpjet turn rate 4, height 500) with the crash flag.
fn balloon_game() -> World {
    let mut r = ruleset();
    let mut t = r.sim.vehicles["BALLOON"].clone();
    t.crashable = Some(true);
    r.sim.vehicles.insert("BALCRASH".into(), t);
    r.registries
        .get_mut("vehicles")
        .unwrap()
        .push("BALCRASH".into());
    let mut w = testkit::game_with(&r, SIZE, 7);
    testkit::run(&mut w, 4);
    w
}

/// A BALCRASH at (0, −4) ordered to (20, −4) and run until it cruises at
/// its full speed 14, 400 leptons or more above the ground.
fn cruising(w: &mut World) -> ObjectId {
    let u = put(w, "BALCRASH", Kind::Vehicle, 0, 0, -4);
    let t = w.tick;
    w.obj_mut(u).unwrap().beh.force(Behaviour::Move, t, true);
    glasnost_sim::econ::assign_destination(w, u, Some(Target::Cell(at(20, -4))));
    assert!(
        until(w, 400, |w| phase(w, u) == Phase::Cruising
            && j(w, u).vc.to_i32_common() == 14
            && j(w, u).vm.to_i32_common() == 14
            && height(w, u) >= 400),
        "it takes off and cruises at full speed"
    );
    u
}

/// The truncated horizontal distance between two points (S-0502 Terms, ρ).
fn rho(a: Pos, b: Pos) -> i64 {
    let (dx, dy) = (i64::from(a.x - b.x), i64::from(a.y - b.y));
    let sq = dx * dx + dy * dy;
    let mut r = 0i64;
    while (r + 1) * (r + 1) <= sq {
        r += 1;
    }
    r
}

/// The centre of the cell a point is in.
fn cell_centre(p: Pos) -> Pos {
    Pos::centre(p.cell(), 0)
}

/// One tick of a fall: position, current and maximum speed, flight state.
struct Tick {
    pos: Pos,
    vc: i32,
    vm: i32,
    phase: Phase,
}

/// Traces `u`, already hit, from the hit to its crash end.
fn trace(w: &mut World, u: ObjectId) -> Vec<Tick> {
    let mut log = Vec::new();
    for k in 0..200 {
        if k > 0 {
            step(w);
        }
        let Some(o) = w.obj(u) else { break };
        let s = o.jump.as_ref().unwrap();
        log.push(Tick {
            pos: o.pos,
            vc: s.vc.to_i32_common(),
            vm: s.vm.to_i32_common(),
            phase: s.phase,
        });
    }
    assert!(w.obj(u).is_none(), "it reaches its crash end");
    log
}

/// The checks of J12 (a) and its two further runs: the maximum speed keeps
/// its value at the hit until the crash end, and no crashing tick shows both
/// speeds at 0.
fn expect_path_kept(log: &[Tick], vm_at_hit: i32, run: &str) {
    assert!(log.len() > 2, "{run}: a fall of more than one update");
    for (k, t) in log.iter().enumerate() {
        assert_eq!(
            t.vm, vm_at_hit,
            "{run}: the maximum speed keeps its value at the hit (tick {k} of the fall)"
        );
        if t.phase == Phase::Crashing {
            assert!(
                t.vc != 0 || t.vm != 0,
                "{run}: no crashing tick with both speeds 0 (tick {k} of the fall)"
            );
        }
    }
    assert_eq!(
        log[1].phase,
        Phase::Crashing,
        "{run}: the first crash update enters crashing"
    );
}

// spec: S-0502 claim 18 (r36, Q-3281, *Start* and *Path of the fall*: in the update that enters crashing only the preceding state's physics runs, then crashing's decisions, so for a unit killed while cruising neither claim 11's arrival test nor F6's bands act on the movement point the destination clear has just moved near it; the maximum speed keeps the value it had before the hit until the crash end, also through the first crash update); scenario J12 balloon-hover subcase (a)
#[test]
fn j12_a_r36_the_maximum_speed_is_kept_through_the_first_crash_update() {
    let mut w = balloon_game();
    let u = cruising(&mut w);
    let vm = j(&w, u).vm.to_i32_common();
    assert_eq!(vm, 14, "void unless at its full speed at the hit");
    kill(&mut w, u);
    assert!(jump::crashing(&w, u));
    let log = trace(&mut w, u);
    expect_path_kept(&log, vm, "(a)");
}

// spec: S-0502 claim 18 (r36, Q-3281: the first crash update makes no decision of cruising, so claim 11's arrival test, ρ below the arrival radius of 20 leptons, never acts on the movement point the destination clear has just moved near it) and scenario J12 balloon-hover subcase (a), its first further run: killed under 20 leptons from the centre of the cell it is in
#[test]
fn j12_a_r36_killed_near_the_cell_centre() {
    let mut w = balloon_game();
    let u = cruising(&mut w);
    assert!(
        until(&mut w, 200, |w| rho(pos(w, u), cell_centre(pos(w, u))) < 20
            && phase(w, u) == Phase::Cruising),
        "it passes within 20 leptons of a cell centre"
    );
    let vm = j(&w, u).vm.to_i32_common();
    assert_eq!(vm, 14, "void unless at its full speed at the hit");
    kill(&mut w, u);
    assert!(jump::crashing(&w, u));
    let log = trace(&mut w, u);
    expect_path_kept(&log, vm, "(a) near the centre");
}

// spec: S-0502 claim 18 (r36, Q-3281: the first crash update makes no decision of cruising, so F6's bands never act on the movement point the destination clear has just moved near it) and F6 (the band ρ < (50 × S) div T = (50 × 14) div 4 = 175 gives a maximum speed below S); scenario J12 balloon-hover subcase (a), its second further run: killed at a distance from the new point of 20 leptons or more and below 175
#[test]
fn j12_a_r36_killed_in_a_speed_band() {
    let mut w = balloon_game();
    let u = cruising(&mut w);
    // Each tick, a probe of the same state is killed; the first whose
    // rerouted point lies in the band from the unit is the run.
    let mut run = None;
    for _ in 0..200 {
        let mut probe = w.clone();
        kill(&mut probe, u);
        if let Some(p) = j(&probe, u).dest {
            let r = rho(pos(&probe, u), p);
            // A margin of the 14-lepton step inside [20, 175), so that the first crash update's ρ is in the band too.
            if (40..150).contains(&r) {
                run = Some(probe);
                break;
            }
        }
        step(&mut w);
    }
    let mut w = run.expect("a hit with the new point in the band");
    let vm = 14;
    assert!(jump::crashing(&w, u));
    let log = trace(&mut w, u);
    expect_path_kept(&log, vm, "(a) in the band");
}

// spec: S-0502 claim 18 (r36, Q-3281: the first crash update makes no decision of ascending, so claim 7's maximum speed S, set when h exceeds one half of the target height for a balloon-hover vehicle, is not set there; the maximum speed stays 0 until the crash end) and claim 7; scenario J12 balloon-hover subcase (e)
#[test]
fn j12_e_r36_killed_ascending_before_its_maximum_speed() {
    let mut w = balloon_game();
    let u = put(&mut w, "BALCRASH", Kind::Vehicle, 0, 0, -4);
    let t = w.tick;
    w.obj_mut(u).unwrap().beh.force(Behaviour::Move, t, true);
    glasnost_sim::econ::assign_destination(&mut w, u, Some(Target::Cell(at(20, -4))));
    // Run until the next update would first take h above half the target
    // height (a probe of the same state without the kill).
    let mut found = false;
    for _ in 0..400 {
        let mut probe = w.clone();
        step(&mut probe);
        if phase(&w, u) == Phase::Ascending && height(&probe, u) * 2 > j(&probe, u).ht {
            found = true;
            break;
        }
        step(&mut w);
    }
    assert!(found, "its h reaches half the target height");
    let s = j(&w, u);
    assert_eq!(s.phase, Phase::Ascending, "void unless ascending");
    assert!(s.moving, "void unless committed");
    assert_eq!(s.vm.to_i32_common(), 0, "void unless the maximum speed is 0");
    assert!(height(&w, u) * 2 <= s.ht, "void unless h is at most half");
    assert!(height(&w, u) > 0, "above the surface");
    kill(&mut w, u);
    assert!(jump::crashing(&w, u));
    let log = trace(&mut w, u);
    assert!(log.len() > 2);
    assert!(
        log.iter().all(|t| t.vm == 0),
        "the maximum speed stays 0 until the crash end"
    );
    assert!(
        log.iter()
            .all(|t| (t.pos.x, t.pos.y) == (log[0].pos.x, log[0].pos.y)),
        "no sideways step"
    );
}

// spec: S-0502 claim 18 (r36, Q-3282, *Destination clear at crash start*: claim 17's no-cell rule, and the outcome of the off setting of quirks.jumpjet_no_cell_self_destruct that replaces it, act only on a unit with health, so a killed unit whose search finds no cell keeps its flight state, movement point and commitment under both settings, and its first crash update's physics is that of the state it was killed in) and Edge cases (r36, *No landing slot*)
#[test]
fn claim18_r36_a_killed_unit_with_no_cell_keeps_its_flight() {
    for on in [true, false] {
        let mut w = balloon_game();
        quirk(&mut w, |q| q.jumpjet_no_cell_self_destruct = on);
        let u = cruising(&mut w);
        block_every_cell(&mut w);
        let point = j(&w, u).dest;
        assert!(point.is_some() && jump::moving(&w, u), "committed");
        let before = pos(&w, u);
        kill(&mut w, u);
        assert!(jump::crashing(&w, u), "switch {on}: it starts to crash");
        assert_eq!(
            phase(&w, u),
            Phase::Cruising,
            "switch {on}: the flight state is kept"
        );
        assert_eq!(j(&w, u).dest, point, "switch {on}: the movement point is kept");
        assert!(jump::moving(&w, u), "switch {on}: the commitment is kept");
        assert_eq!(
            w.obj(u).unwrap().dest,
            None,
            "switch {on}: the order-level destination is empty"
        );
        step(&mut w);
        assert_eq!(phase(&w, u), Phase::Crashing, "switch {on}");
        let after = pos(&w, u);
        assert_ne!(
            (after.x, after.y),
            (before.x, before.y),
            "switch {on}: the first crash update steps on as cruising's physics does"
        );
    }
}

// spec: S-0502 Edge cases (r36, *No landing slot*: the switch-off outcome acts on a unit that still has health) and claim 18 (a crash started without a kill keeps the unit's health)
#[test]
fn claim18_r36_a_crash_without_a_kill_still_takes_the_switch_off_outcome() {
    let mut w = balloon_game();
    quirk(&mut w, |q| q.jumpjet_no_cell_self_destruct = false);
    let u = cruising(&mut w);
    block_every_cell(&mut w);
    assert!(jump::start_crash(&mut w, u, None), "a crash started");
    assert!(w.obj(u).unwrap().health > 0, "with health");
    assert_eq!(phase(&w, u), Phase::Hovering, "it hovers again");
    assert_eq!(j(&w, u).dest, None, "no movement point");
    assert!(!jump::moving(&w, u), "no commitment");
}
