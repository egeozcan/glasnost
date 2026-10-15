//! Claim tests of S-0513 claims 50 and 51 (the engine-design contracts of
//! the exceptional landing and the warp turn; Q-0850 and Q-0851 answered),
//! on synthetic rules (`tests/sw_common`). SW23 part f and SW24 part c are
//! engine-only contract tests, not oracle expectations.

mod sw_common;

use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_sim::superweapons::warp::Stage;
use glasnost_sim::world::{ObjectId, Pos, Target, World};
use glasnost_sim::testkit;
use sw_common::*;

/// The rules' teleport delay in these tests.
const DELAY: i32 = 7;

fn rules() -> Ruleset {
    let mut r = ruleset();
    {
        let g = &mut r.sim.general.superweapons;
        g.chrono_placement = Some("MARK".into());
        g.chrono_blast = Some("BLASTS".into());
        g.chrono_blast_dest = Some("BLASTD".into());
        g.chrono_delay = Some(DELAY);
    }
    r.sim.general.warp_out = Some("WARPA".into());
    for n in ["MARK", "BLASTS", "BLASTD", "WARPA"] {
        r.sim
            .animations
            .insert(n.into(), glasnost_rules::sim::AnimationType::default());
        r.registries
            .entry("animations".into())
            .or_default()
            .push(n.into());
    }
    r
}

fn stage(w: &World, id: ObjectId) -> Option<Stage> {
    w.obj(id).and_then(|o| o.sw.warp.as_ref()).map(|m| m.stage)
}

fn chrono(w: &mut World) {
    spawn(w, "CHRONOB", ALPHA, -9, -9);
    ready_tick(w, ALPHA, SOURCE, 300);
}

/// Picks `src`, orders the warp to `dst` and runs the tick that applies it; returns t_L.
fn warp(w: &mut World, src: CellXY, dst: CellXY) -> i32 {
    if !rec(w, ALPHA, SOURCE).ready {
        ready_tick(w, ALPHA, SOURCE, 300);
    }
    fire_now(w, ALPHA, SOURCE, src);
    fire(w, ALPHA, WARP, dst);
    let tl = w.tick;
    step(w);
    tl
}

fn run_through(w: &mut World, t: i32) {
    while w.tick <= t {
        step(w);
    }
}

/// Every cell of the map becomes water: no landing cell qualifies for the search.
fn flood(w: &mut World) {
    let mut cat = (*testkit::catalogue()).clone();
    let first = cat.tiles.len() as i32;
    let mut water = cat.tiles[0].clone();
    if let Some(Some(c)) = water.cells.first_mut() {
        c.terrain_code = 9; // the catalogue's water terrain code (as in claims_s0614)
    }
    cat.tiles.push(water);
    cat.allow_tiberium.push(false);
    w.catalogue = Some(std::sync::Arc::new(cat));
    let all: Vec<CellXY> = w.cells.grid.iter().map(|(c, _)| c).collect();
    for &c in &all {
        w.cells.grid.get_mut(c).unwrap().tile = first;
    }
    glasnost_sim::load::recalc_cells(w, &all);
}

/// A legal destination cell whose neighbours at (1, 0) and (0, 1) are off the map: the
/// destination cells that the source block's cells 1 and 7 map to do not exist (F1).
fn corner_destination(w: &World) -> CellXY {
    w.cells
        .grid
        .iter()
        .map(|(c, _)| c)
        .find(|&c| {
            w.cells.cell(CellXY::new(c.x + 1, c.y)).is_none()
                && w.cells.cell(CellXY::new(c.x, c.y + 1)).is_none()
        })
        .expect("the map has a corner")
}

/// The warp diagnostics the world holds (claim 50 reports the first of each kind).
fn warp_notes(w: &World) -> usize {
    w.notes
        .iter()
        .filter(|n| n.starts_with("warp: object"))
        .count()
}

// --- Claim 50: the exceptional landing contract -------------------------------------------

#[test]
fn claim50_an_off_map_destination_falls_back_to_cell_zero_with_the_claim_36_point() {
    // spec: S-0513 claim 50 (both exceptional cases use fallback cell coordinates (0, 0); the initial fallback point is that cell's centre for a vehicle, and infantry and aircraft retain their position within the source cell as in claim 36; the height is 0 leptons with no bridge lift while (0, 0) is not a legal cell, S-0009 claim 1 and claim 44)
    let mut w = game(&rules());
    chrono(&mut w);
    let corner = corner_destination(&w);
    let cart = spawn(&mut w, "CART", BETA, 1, 0); // block cell 1: its destination cell is off the map
    let jumper = spawn(&mut w, "JUMPER", BETA, 0, 1); // block cell 7
    {
        let o = w.obj_mut(jumper).unwrap();
        o.pos.x += 37;
        o.pos.y += 61;
    }
    let within = (
        w.obj(jumper).unwrap().pos.x % 256,
        w.obj(jumper).unwrap().pos.y % 256,
    );
    warp(&mut w, at(0, 0), corner);
    assert!(
        w.cells.cell(CellXY::new(0, 0)).is_none(),
        "(0, 0) is never a legal cell"
    );
    let v = w.obj(cart).unwrap().sw.warp.clone().unwrap();
    assert_eq!((v.landing.x, v.landing.y, v.landing.z), (128, 128, 0));
    let j = w.obj(jumper).unwrap().sw.warp.clone().unwrap();
    assert_eq!(
        (j.landing.x, j.landing.y, j.landing.z),
        (within.0, within.1, 0)
    );
    assert!(v.fell_back && j.fell_back);
    assert_eq!(
        warp_notes(&w),
        2,
        "each member's first fallback is reported once"
    );
}

#[test]
fn claim50_a_failed_search_falls_back_keeping_the_position_within_the_previous_landing_cell() {
    // spec: S-0513 claim 50 (after a failed nearby search the fallback retains the member's position within its previous landing cell, as claim 37 does for a successful redirect; a vehicle's is the cell centre); claims 49 and 40 (the retry sets the held delay)
    let mut w = game(&rules());
    chrono(&mut w);
    let jumper = spawn(&mut w, "JUMPER", ALPHA, 0, 0);
    {
        let o = w.obj_mut(jumper).unwrap();
        o.pos.x += 37;
        o.pos.y += 61;
    }
    let cart = spawn(&mut w, "CART", ALPHA, 1, 0);
    spawn(&mut w, "TOWER", BETA, 5, 5);
    spawn(&mut w, "TOWER", BETA, 6, 5);
    let tl = warp(&mut w, at(0, 0), at(5, 5));
    let before = w.obj(jumper).unwrap().sw.warp.clone().unwrap().landing;
    run_through(&mut w, tl + 61);
    flood(&mut w);
    run_through(&mut w, tl + 62);
    let m = w.obj(jumper).unwrap().sw.warp.clone().unwrap();
    assert_eq!(m.stage, Stage::Blocked);
    assert_eq!(m.landing.cell(), CellXY::new(0, 0));
    assert_eq!((m.landing.x, m.landing.y), (before.x % 256, before.y % 256));
    assert_eq!(m.landing.z, 0, "no height and no lift outside the map");
    let c = w.obj(cart).unwrap().sw.warp.clone().unwrap();
    assert_eq!((c.landing.x, c.landing.y), (128, 128));
    assert_eq!(w.obj(jumper).unwrap().sw.held_delay, DELAY);
}

#[test]
fn claim50_a_member_with_no_legal_landing_stays_alive_in_the_retry_without_a_cap() {
    // spec: S-0513 claim 50 (a point outside the map is blocked and uses the same redirect search; a failed search replaces the point with the fallback again; retry updates set the held delay; placement failure never deletes a member: it stays alive and tracked by its house in the landing-retry stage with no retry cap and no timeout death; the first fallback is reported once per member and warp, without game-rule effects)
    let mut w = game(&rules());
    chrono(&mut w);
    let corner = corner_destination(&w);
    let a = spawn(&mut w, "CART", ALPHA, 1, 0);
    flood(&mut w);
    let tl = warp(&mut w, at(0, 0), corner);
    run_through(&mut w, tl + 400);
    let o = w.obj(a).expect("the member is not deleted");
    assert!(o.health > 0, "no timeout death");
    assert_eq!(o.owner, Some(ALPHA), "its owner is retained");
    assert!(!o.on_map, "it is not placed outside the map");
    let m = o.sw.warp.clone().unwrap();
    assert_eq!(m.stage, Stage::Blocked);
    assert_eq!(m.landing.cell(), CellXY::new(0, 0));
    assert_eq!(o.sw.held_delay, DELAY);
    assert_eq!(warp_notes(&w), 1, "one fallback report, no repeats");
}

#[test]
fn claim50_a_legal_redirect_after_the_fallback_places_and_arrives_on_separate_updates() {
    // spec: S-0513 claim 50 (a passed check schedules placement on the next update, not placement and arrival effects together; successful placement schedules the arrival effects on the next update; the redirect search from the fallback may find a legal cell)
    let mut w = game(&rules());
    chrono(&mut w);
    let corner = corner_destination(&w);
    let a = spawn(&mut w, "CART", ALPHA, 1, 0);
    warp(&mut w, at(0, 0), corner);
    let mut seen: Vec<(i32, Stage)> = Vec::new();
    while let Some(s) = stage(&w, a) {
        seen.push((w.tick - 1, s));
        step(&mut w);
        assert!(seen.len() < 300, "the member lands");
    }
    let at_stage = |f: fn(&Stage) -> bool| seen.iter().find(|(_, s)| f(s)).map(|p| p.0);
    let away = at_stage(|s| matches!(s, Stage::Away)).expect("check passed");
    let placed = at_stage(|s| matches!(s, Stage::Placed)).expect("placed");
    let hold = at_stage(|s| matches!(s, Stage::Hold(_))).expect("effects done");
    assert_eq!(placed, away + 1, "placement on the update after the check");
    assert_eq!(hold, placed + 1, "arrival effects on the update after placement");
    let o = w.obj(a).unwrap();
    assert!(o.on_map && o.health > 0);
    assert!(w.cells.cell(o.pos.cell()).is_some());
}

#[test]
fn claim50_a_refused_placement_keeps_the_member_in_the_retry_and_tries_again() {
    // spec: S-0513 claim 50 (placement failure by itself never deletes a member: it leaves it alive in the landing-retry stage with its landing point and owner retained; retry resumes on the next update; the first failure per member and warp is reported once)
    let mut w = game(&rules());
    chrono(&mut w);
    let a = spawn(&mut w, "CART", ALPHA, 0, 0);
    let tl = warp(&mut w, at(0, 0), at(5, 5));
    run_through(&mut w, tl + 62); // departed, the check passed
    assert_eq!(stage(&w, a), Some(Stage::Away));
    // Force the refusal: the landing point is outside the map at the placement update.
    let mut m = w.obj(a).unwrap().sw.warp.clone().unwrap();
    m.landing = Pos::new(128, 128, 0);
    w.obj_mut(a).unwrap().sw.warp = Some(m);
    step(&mut w);
    let m = w.obj(a).unwrap().sw.warp.clone().unwrap();
    assert_eq!(m.stage, Stage::Blocked, "back in the landing retry");
    assert!(m.refused);
    assert_eq!(m.landing, Pos::new(128, 128, 0), "its landing point is retained");
    let o = w.obj(a).unwrap();
    assert!(!o.on_map && o.health > 0);
    assert_eq!(o.owner, Some(ALPHA));
    assert_eq!(warp_notes(&w), 1);
    run_through(&mut w, tl + 200);
    let o = w.obj(a).expect("the retries delete nothing");
    assert!(o.health > 0 && o.on_map, "the retry found a legal cell and placed it");
    assert_eq!(stage(&w, a), None);
    assert_eq!(warp_notes(&w), 1, "the refusal is reported once");
}

// --- Claim 51: the warp turn contract --------------------------------------------------------

/// CART self-heals every tick (a period of one tick) and a redirected hold lasts DELAY ticks.
fn contract_rules() -> Ruleset {
    let mut r = rules();
    r.sim.vehicles.get_mut("CART").unwrap().combat.self_healing = Some(true);
    r.sim.general.repair_rate = Some(glasnost_rules::sim::Decimal("0.001".into()));
    r.globals.quirks.warp_hold_persists = true;
    r
}

#[test]
fn claim51_no_self_healing_before_the_hold_and_healing_in_it() {
    // spec: S-0513 claim 51 and S-0314 claims 9 and 10 (no ordinary self-healing from the first pending update through arrival effects, before or after the warp work; permitted in the arrival hold after the hold-end test, whether still holding or expiring)
    let mut w = game(&contract_rules());
    chrono(&mut w);
    let a = spawn(&mut w, "CART", ALPHA, 0, 0);
    w.obj_mut(a).unwrap().health = 100;
    w.obj_mut(a).unwrap().sw.held_delay = DELAY;
    warp(&mut w, at(0, 0), at(5, 5));
    let (mut hold_ticks, mut hold_gain) = (0, 0);
    for _ in 0..120 {
        let s = stage(&w, a);
        let h0 = w.obj(a).unwrap().health;
        step(&mut w);
        let gain = w.obj(a).unwrap().health - h0;
        match s {
            None => break,
            Some(Stage::Hold(_)) => {
                hold_ticks += 1;
                hold_gain += gain;
            }
            Some(st) => assert_eq!(gain, 0, "no healing in {st:?}"),
        }
    }
    assert!(hold_ticks >= 2, "the hold lasts several turns");
    assert_eq!(hold_gain, hold_ticks, "one point per hold turn, the expiry turn included");
    let h = w.obj(a).unwrap().health;
    step(&mut w);
    assert_eq!(w.obj(a).unwrap().health, h + 1, "healing goes on after the warp");
}

#[test]
fn claim51_no_movement_before_the_hold() {
    // spec: S-0513 claim 51 and S-0203 claim 4 (no ordinary movement from the first pending update through arrival effects, before or after the warp work; in the arrival hold, ordinary movement follows the hold-end test under its own eligibility)
    let mut r = rules();
    r.globals.quirks.warp_hold_persists = true;
    let mut w = game(&r);
    chrono(&mut w);
    let a = spawn(&mut w, "CART", ALPHA, 0, 0);
    w.obj_mut(a).unwrap().sw.held_delay = DELAY;
    warp(&mut w, at(0, 0), at(5, 5));
    let mut moved_in_hold = false;
    for _ in 0..120 {
        let s = stage(&w, a);
        // The destination is offered on the expiry turn: movement may only follow once phasing in ends.
        if let Some(Stage::Hold(ta)) = s
            && w.tick - ta >= DELAY
        {
            w.obj_mut(a).unwrap().dest = Some(Target::Cell(at(5, -5)));
        }
        if matches!(s, Some(st) if !matches!(st, Stage::Hold(_))) {
            w.obj_mut(a).unwrap().dest = Some(Target::Cell(at(5, -5)));
        }
        let p0 = w.obj(a).unwrap().pos;
        step(&mut w);
        match s {
            None => break,
            Some(Stage::Hold(_)) => {
                // Phasing in still refuses movement itself (S-0207); the expiry turn is the first on which the ordinary movement is eligible.
                let o = w.obj(a).unwrap();
                if stage(&w, a).is_none() {
                    moved_in_hold |=
                        o.pos != p0 || o.drive.as_ref().is_some_and(|d| d.track.is_some());
                }
            }
            Some(st) => {
                let o = w.obj(a).unwrap();
                assert!(
                    o.drive.as_ref().is_none_or(|d| d.track.is_none()),
                    "no movement in {st:?}"
                );
            }
        }
    }
    assert!(moved_in_hold, "the expiry turn permits ordinary movement");
}

#[test]
fn claim51_no_behaviour_firing_or_turret_before_the_hold() {
    // spec: S-0513 claim 51 and S-0215 claim 2, S-0310 (no ordinary behaviour processing, firing or turret turning from the first pending update through arrival effects; in the arrival hold each is under its own eligibility after the hold-end test)
    let mut r = rules();
    r.globals.quirks.warp_hold_persists = true;
    let mut w = game(&r);
    chrono(&mut w);
    let a = spawn(&mut w, "CART", ALPHA, 0, 0);
    let enemy = spawn(&mut w, "WALKER", BETA, 8, 5); // within the gun's range of the landing cell (5, 5)
    w.obj_mut(a).unwrap().sw.held_delay = DELAY;
    let e0 = w.obj(enemy).unwrap().health;
    let turret0 = w.obj(a).unwrap().turret.value(w.tick);
    warp(&mut w, at(0, 0), at(5, 5));
    let mut processed_in_hold = false;
    for _ in 0..120 {
        let s = stage(&w, a);
        step(&mut w);
        match s {
            None => break,
            Some(Stage::Hold(_)) => {
                processed_in_hold |= w.obj(a).is_some_and(|o| o.target.is_some());
            }
            Some(st) => {
                let o = w.obj(a).unwrap();
                assert_eq!(w.obj(enemy).unwrap().health, e0, "no shot in {st:?}");
                assert!(o.target.is_none(), "no behaviour processing in {st:?}");
                assert_eq!(o.turret.value(w.tick), turret0, "no turret turning in {st:?}");
            }
        }
    }
    assert!(processed_in_hold, "the hold permits ordinary behaviour processing");
    // Firing follows under its own eligibility once phasing in ends.
    let mut shot = false;
    for _ in 0..80 {
        step(&mut w);
        shot |= w.obj(enemy).is_none_or(|e| e.health < e0);
    }
    assert!(shot, "ordinary firing resumes");
}

#[test]
fn claim51_launch_clears_the_path_the_queue_and_the_driving_step_at_once() {
    // spec: S-0513 claim 51 (launch immediately clears the old path, queued destinations and current driving step; none resumes when the warp ends; the target and current destination are dropped by claim 48's leaving-stage rule, not earlier)
    let mut w = game(&rules());
    chrono(&mut w);
    let a = spawn(&mut w, "CART", ALPHA, 0, 0);
    let j = spawn(&mut w, "JUMPER", ALPHA, 1, 0);
    // A route, a queue and a driving step, set directly: the synthetic scene's vehicles do not start tracks from a bare order.
    let here = w.obj(a).unwrap().pos;
    {
        let o = w.obj_mut(a).unwrap();
        o.dest = Some(Target::Cell(at(5, -5)));
        o.path.steps = vec![2, 2, 2];
        let d = o.drive.as_mut().unwrap();
        d.goal = Some(here);
        d.between_steps = false;
        d.turning_in_place = true;
    }
    {
        let o = w.obj_mut(j).unwrap();
        o.dest = Some(Target::Cell(at(5, -5)));
        o.path.steps = vec![2, 2];
        o.walk.as_mut().unwrap().fraction = true;
    }
    for id in [a, j] {
        w.obj_mut(id)
            .unwrap()
            .queued_dests
            .push(Target::Cell(at(5, -6)));
    }
    let src = here.cell();
    warp(&mut w, src, at(5, 5));
    for id in [a, j] {
        let o = w.obj(id).unwrap();
        if o.sw.warp.is_none() {
            continue; // outside the 3 x 3 block of the launch
        }
        assert!(o.queued_dests.is_empty(), "queued destinations cleared at launch");
        assert!(o.path.steps.is_empty());
        assert!(
            o.drive
                .as_ref()
                .is_none_or(|d| d.track.is_none() && d.goal.is_none() && !d.turning_in_place),
            "driving step cleared"
        );
        assert!(
            o.walk.as_ref().is_none_or(|k| k.target.is_none() && !k.fraction),
            "walking step cleared"
        );
        assert!(
            o.dest.is_some(),
            "the destination goes in the first update (claim 48)"
        );
    }
    assert!(w.obj(a).unwrap().sw.warp.is_some(), "the vehicle joined");
    run(&mut w, 100);
    for id in [a, j] {
        let o = w.obj(id).unwrap();
        assert!(
            o.queued_dests.is_empty() && o.path.steps.is_empty(),
            "nothing resumes after the warp"
        );
    }
}

#[test]
fn claim51_a_member_receives_one_warp_update_per_tick_even_off_the_map() {
    // spec: S-0513 claim 51 (one warp update in each tick while pending, leaving, awaiting landing, placed awaiting effects or holding, even while off-map) and claim 39's tick labels
    let mut w = game(&rules());
    chrono(&mut w);
    let a = spawn(&mut w, "CART", ALPHA, 0, 0);
    let tl = warp(&mut w, at(0, 0), at(5, 5));
    let mut path = vec![(tl, stage(&w, a).unwrap())];
    loop {
        step(&mut w);
        match stage(&w, a) {
            Some(s) => path.push((w.tick - 1, s)),
            None => break,
        }
        assert!(path.len() < 300);
    }
    // Each tick advances at most one stage: a stage lasts its own updates and none is skipped.
    let names: Vec<&str> = path
        .iter()
        .map(|p| match p.1 {
            Stage::Pending => "pending",
            Stage::Leaving(_) => "leaving",
            Stage::Expired => "expired",
            Stage::Blocked => "blocked",
            Stage::Away => "away",
            Stage::Placed => "placed",
            Stage::Hold(_) => "hold",
        })
        .collect();
    let i = names.iter().position(|n| *n == "expired").unwrap();
    assert_eq!(names[i + 1], "away", "{names:?}"); // the departure update also passes the check
    assert_eq!(names[i + 2], "placed");
    assert_eq!(names[i + 3], "hold");
    assert_eq!(path[i + 1].0, path[i].0 + 1);
    assert_eq!(path[i + 2].0, path[i].0 + 2);
    assert_eq!(path[i + 3].0, path[i].0 + 3);
    assert_eq!(path[1].0, tl + 1, "claim 39: leaving in t_L + 1");
}
