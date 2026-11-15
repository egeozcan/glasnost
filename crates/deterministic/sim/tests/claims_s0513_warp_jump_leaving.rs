//! A jumpjet warp member's departure ends its flight state as any going
//! off the map does: no tracked flying object, no landing reservation and
//! no hover slot left behind (#360; S-0502 claims 4, 15 and 29, S-0513
//! claims 34 and 39). On the synthetic superweapon rules of `sw_common`
//! (JJ a CART with jumpjet movement), no game data.

mod sw_common;

use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_sim::combat::flying;
use glasnost_sim::jump::Phase;
use glasnost_sim::superweapons::warp::Stage;
use glasnost_sim::world::{Layer, ObjectId, Target, World};
use sw_common::*;

/// The synthetic rules with JJ, a CART with jumpjet movement and the given
/// cruise height.
fn rules_jj(height: i32) -> Ruleset {
    let mut r = ruleset();
    r.sim.general.superweapons.chrono_delay = Some(7);
    let mut t = r.sim.vehicles["CART"].clone();
    t.locomotor = Some(glasnost_rules::sim::Locomotor::Jumpjet);
    t.jumpjet_height = Some(height);
    r.sim.vehicles.insert("JJ".into(), t);
    r.registries
        .entry("vehicles".into())
        .or_default()
        .push("JJ".into());
    r
}

fn stage(w: &World, id: ObjectId) -> Option<Stage> {
    w.obj(id).and_then(|o| o.sw.warp.as_ref()).map(|m| m.stage)
}

fn move_order(w: &mut World, j: ObjectId, c: CellXY) {
    let ok = glasnost_sim::orders::issue(
        w,
        BETA,
        glasnost_sim::orders::Order::Behaviour {
            actor: j,
            behaviour: glasnost_sim::rules::Behaviour::Move,
            attack_move: false,
            target: None,
            dest: Some(Target::Cell(c)),
        },
    );
    assert!(ok, "the move order is taken");
}

/// Launches the warp from `src` to `dst` in the first tick whose rehearsal
/// (the objects' turns of that tick, which come before its orders)
/// satisfies `f`. Returns t_L.
fn warp_when(w: &mut World, src: CellXY, dst: CellXY, f: impl Fn(&World) -> bool) -> i32 {
    if !rec(w, ALPHA, SOURCE).ready {
        ready_tick(w, ALPHA, SOURCE, 300);
    }
    fire_now(w, ALPHA, SOURCE, src);
    until(w, 400, |w| {
        let mut p = w.clone();
        step(&mut p);
        f(&p)
    });
    fire(w, ALPHA, WARP, dst);
    let tl = w.tick;
    step(w);
    tl
}

fn vehicles(w: &World, c: CellXY) -> u8 {
    w.cells.occ(c, Layer::Ground).map_or(0, |r| r.vehicles)
}

// spec: S-0502 claim 29 (End: going off the map for any reason) and S-0513 claims 34 and 39 (SW12 part b8's unit; a departed member is off the map until it appears)
#[test]
fn issue_360_a_departed_jumpjet_member_is_no_tracked_flying_object() {
    let mut w = game(&rules_jj(1000));
    spawn(&mut w, "CHRONOB", ALPHA, -9, -9);
    ready_tick(&mut w, ALPHA, SOURCE, 300);
    let j = spawn(&mut w, "JJ", BETA, 0, 0);
    move_order(&mut w, j, at(-4, 0));
    let tl = warp_when(&mut w, at(0, 0), at(5, 5), |p| {
        let h = p.height_above_surface(j);
        h > 0 && h < 208 && p.obj(j).unwrap().jump.as_ref().unwrap().phase == Phase::Ascending
    });
    assert_eq!(stage(&w, j), Some(Stage::Pending), "set-up: it joins");
    assert!(flying::tracked(&w, j), "set-up: registered at its take-off");
    let mut seen = Vec::new();
    while w.tick <= tl + 63 {
        step(&mut w);
        let o = w.obj(j).unwrap();
        if matches!(stage(&w, j), Some(Stage::Away | Stage::Placed)) {
            seen.push((
                stage(&w, j).unwrap(),
                o.on_map,
                w.height_above_surface(j),
                o.jump.as_ref().unwrap().phase,
                flying::tracked(&w, j),
            ));
        }
    }
    assert_eq!(
        seen,
        vec![
            (Stage::Away, false, 5, Phase::Ascending, false),
            (Stage::Placed, true, 0, Phase::Grounded, false),
        ],
        "(stage, on the map, height, jumpjet state, tracked flying object)"
    );
}

// spec: S-0502 claims 4 and 15 (the landing reservation, the vehicle-present mark it sets; released when the jumpjet leaves the map) and S-0513 claims 34 and 39
#[test]
fn issue_360_a_departed_descending_jumpjet_member_keeps_no_landing_reservation() {
    let mut w = game(&rules_jj(300));
    spawn(&mut w, "CHRONOB", ALPHA, -9, -9);
    ready_tick(&mut w, ALPHA, SOURCE, 300);
    let j = spawn(&mut w, "JJ", BETA, -3, 0);
    move_order(&mut w, j, at(0, 0));
    let tl = warp_when(&mut w, at(0, 0), at(5, 5), |p| {
        let o = p.obj(j).unwrap();
        let jj = o.jump.as_ref().unwrap();
        let h = p.height_above_surface(j);
        jj.phase == Phase::Descending
            && jj.reservation.is_some()
            && h > 0
            && h < 208
            && p.cells.list(o.pos.cell(), o.layer).contains(&j)
    });
    assert_eq!(stage(&w, j), Some(Stage::Pending), "set-up: it joins");
    let old = w.obj(j).unwrap().pos.cell();
    let reservation = |w: &World| w.obj(j).unwrap().jump.as_ref().unwrap().reservation;
    assert_eq!(reservation(&w).map(|m| m.cell), Some(old), "set-up");
    while w.tick <= tl + 62 {
        step(&mut w);
    }
    assert!(!w.obj(j).unwrap().on_map, "set-up: departed, off the map");
    assert_eq!(
        (reservation(&w).map(|m| m.cell), vehicles(&w, old)),
        (None, 0),
        "(the reservation of the member off the map, the vehicle-present marks left in its old cell)"
    );
}
