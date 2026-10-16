//! Claim tests of S-0513 claim 50's placement acceptance (Q-0871 answered): a passed landing check
//! is not a reservation, so the placement one update later refuses an illegal cell, a newly
//! blocking object, a newly present protected object, a member that does not fit on the selected
//! layer and a terrain change that makes the landing unenterable; a refusal never deletes the
//! member, and the next update repeats the full claim 37 check. Synthetic rules
//! (`tests/sw_common`); SW24 part (d) engine-only checks, not oracle expectations.

mod sw_common;

use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_sim::superweapons::warp::Stage;
use glasnost_sim::superweapons::{ProtectKind, Protection, Tint};
use glasnost_sim::testkit;
use glasnost_sim::world::{ObjectId, World};
use sw_common::*;

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

fn refused(w: &World, id: ObjectId) -> bool {
    w.obj(id)
        .and_then(|o| o.sw.warp.as_ref())
        .is_some_and(|m| m.refused)
}

/// Launches a warp of the block around (0, 0) to (5, 5) and runs to the update in which the
/// check of `m`'s landing has passed (stage `Away`); the placement is the next update.
fn to_away(w: &mut World, m: ObjectId) {
    spawn(w, "CHRONOB", ALPHA, -9, -9);
    ready_tick(w, ALPHA, SOURCE, 300);
    fire_now(w, ALPHA, SOURCE, at(0, 0));
    fire(w, ALPHA, WARP, at(5, 5));
    for _ in 0..200 {
        step(w);
        if stage(w, m) == Some(Stage::Away) {
            return;
        }
    }
    panic!("the check never passed");
}

fn landing_cell(w: &World, id: ObjectId) -> CellXY {
    w.obj(id)
        .and_then(|o| o.sw.warp.as_ref())
        .map(|m| m.landing.cell())
        .unwrap()
}

fn protect(w: &mut World, id: ObjectId) {
    let t = w.tick;
    w.obj_mut(id).unwrap().sw.protection = Some(Protection {
        start: t,
        duration: 10_000,
        kind: ProtectKind::Curtain,
        tint: Tint::default(),
    });
}

#[test]
fn claim50_a_newly_blocking_building_refuses_the_placement_and_the_member_survives() {
    // spec: S-0513 claim 50 (placement refuses a newly blocking building; the failure alone never deletes the member, which stays alive and off the map in the landing retry; the next update repeats the full check, and a passed check schedules the placement for the following update) and claim 51's diagnostic
    let mut w = game(&rules());
    let m = spawn(&mut w, "CART", ALPHA, 0, 0);
    to_away(&mut w, m);
    assert_eq!(landing_cell(&w, m), at(5, 5));
    let b = spawn(&mut w, "AUXB", BETA, 5, 5);
    step(&mut w);
    assert_eq!(
        stage(&w, m),
        Some(Stage::Blocked),
        "back in the landing retry"
    );
    assert!(refused(&w, m));
    let o = w.obj(m).unwrap();
    assert!(!o.on_map && o.health > 0 && o.owner == Some(ALPHA));
    assert!(
        w.obj(b).unwrap().health > 0,
        "the failed placement hits nothing"
    );
    // The full check runs again: the building blocks it and the redirect search moves the landing.
    step(&mut w);
    let new = landing_cell(&w, m);
    assert_ne!(new, at(5, 5), "redirected away from the building");
    for _ in 0..200 {
        if stage(&w, m).is_none() {
            break;
        }
        step(&mut w);
    }
    let o = w.obj(m).expect("alive");
    assert!(o.on_map && o.health > 0);
    assert_eq!(o.pos.cell(), new);
}

#[test]
fn claim50_a_newly_present_protected_object_refuses_the_placement_and_the_next_check_kills() {
    // spec: S-0513 claim 50 (placement refuses a newly present protected object, hitting nobody) and claim 37 (the next update's full check: a protected object kills the member)
    let mut w = game(&rules());
    let m = spawn(&mut w, "CART", ALPHA, 0, 0);
    to_away(&mut w, m);
    let p = spawn(&mut w, "CART", BETA, 5, 5);
    protect(&mut w, p);
    let hp = w.obj(p).unwrap().health;
    step(&mut w);
    assert_eq!(stage(&w, m), Some(Stage::Blocked));
    assert!(refused(&w, m));
    assert!(
        w.obj(m).unwrap().health > 0,
        "no hit from the failed placement"
    );
    step(&mut w);
    assert!(
        w.obj(m).is_none_or(|o| o.health <= 0),
        "the full check of the next update kills the member"
    );
    assert_eq!(
        w.obj(p).unwrap().health,
        hp,
        "the protected object is unharmed"
    );
}

#[test]
fn claim50_a_vehicle_that_arrived_first_is_not_displaced_by_the_placement() {
    // spec: S-0513 claim 50 (insufficient capacity on the selected layer refuses the placement; current occupants are not killed, displaced or ignored by the attempt; incompatible vehicles refuse sharing) and claim 37 (the next full check kills an unprotected vehicle at the landing cell)
    let mut w = game(&rules());
    let m = spawn(&mut w, "CART", ALPHA, 0, 0);
    to_away(&mut w, m);
    let v = spawn(&mut w, "CART", BETA, 5, 5);
    let h = w.obj(v).unwrap().health;
    step(&mut w);
    assert_eq!(stage(&w, m), Some(Stage::Blocked));
    assert!(refused(&w, m));
    assert_eq!(w.obj(v).unwrap().health, h, "the occupant was not hit");
    assert!(
        w.cells
            .list(at(5, 5), glasnost_sim::world::Layer::Ground)
            .contains(&v)
    );
    assert!(!w.obj(m).unwrap().on_map);
    // The full check of the next update kills the occupant and passes; the placement follows.
    step(&mut w);
    assert_eq!(stage(&w, m), Some(Stage::Away));
    assert!(w.obj(v).is_none_or(|o| o.health <= 0));
    step(&mut w);
    assert_eq!(stage(&w, m), Some(Stage::Placed));
    assert!(w.obj(m).unwrap().on_map);
}

#[test]
fn claim50_two_members_competing_for_one_cell_never_overlap() {
    // spec: S-0513 claim 50 (multiple placements in one tick see the occupancy left by earlier placements in the turn order of S-0203 claim 3; no over-capacity overlap is allowed)
    let mut w = game(&rules());
    let a = spawn(&mut w, "CART", ALPHA, 0, 0);
    let b = spawn(&mut w, "CART", ALPHA, 1, 0);
    to_away(&mut w, a);
    // Both passed the check in the same update; give them the same landing point.
    assert_eq!(stage(&w, b), Some(Stage::Away));
    let land = w.obj(a).unwrap().sw.warp.clone().unwrap().landing;
    {
        let mut mb = w.obj(b).unwrap().sw.warp.clone().unwrap();
        mb.landing = land;
        w.obj_mut(b).unwrap().sw.warp = Some(mb);
    }
    step(&mut w);
    let placed = [a, b]
        .iter()
        .filter(|&&x| stage(&w, x) == Some(Stage::Placed))
        .count();
    let waiting = [a, b]
        .iter()
        .filter(|&&x| stage(&w, x) == Some(Stage::Blocked) && refused(&w, x))
        .count();
    assert_eq!(
        (placed, waiting),
        (1, 1),
        "one is placed, the other refused"
    );
    let listed = w
        .cells
        .list(land.cell(), glasnost_sim::world::Layer::Ground)
        .iter()
        .filter(|&&x| x == a || x == b)
        .count();
    assert_eq!(listed, 1, "no overlap");
    assert!(w.obj(a).unwrap().health > 0 && w.obj(b).unwrap().health > 0);
}

#[test]
fn claim50_an_infantry_unit_shares_only_a_free_standing_position() {
    // spec: S-0513 claim 50 and S-0209 claim 4 (infantry may share a cell only when their landing position and the cell's capacity permit it; no over-capacity overlap; a refusal with no victim hit and no draw)
    for same_position in [false, true] {
        let mut w = game(&rules());
        let m = spawn(&mut w, "JUMPER", ALPHA, 0, 0);
        to_away(&mut w, m);
        let o = spawn(&mut w, "WALKER", BETA, 5, 5);
        {
            let ob = w.obj_mut(o).unwrap();
            // Same centre class (not the exact point, which would have been killed by the check),
            // or the far left side position.
            if same_position {
                ob.pos.x += 20;
            } else {
                // The left standing position of the cell (S-0208 table P).
                ob.pos.x = ob.pos.x - ob.pos.x.rem_euclid(256) + 64;
                ob.pos.y = ob.pos.y - ob.pos.y.rem_euclid(256) + 192;
            }
        }
        // Re-mark the walker's standing position at its new point.
        let q = glasnost_sim::objects::occ_quirks(&w);
        w.cells
            .clear_infantry(at(5, 5), glasnost_sim::world::Layer::Ground, o, q);
        let p = w.obj(o).unwrap().pos;
        let class =
            glasnost_sim::walk::position_class(p.x.rem_euclid(256), p.y.rem_euclid(256), true);
        w.cells
            .mark_infantry(at(5, 5), glasnost_sim::world::Layer::Ground, class, o, BETA);
        let h = w.obj(o).unwrap().health;
        step(&mut w);
        let st = stage(&w, m);
        if same_position {
            assert_eq!(st, Some(Stage::Blocked), "the position is taken");
            assert!(refused(&w, m));
            // The check passes again (no exact-point kill) and the placement stays refused while the
            // position is taken: the member remains alive and off the map, with no timeout.
            run(&mut w, 20);
            assert!(w.obj(m).unwrap().health > 0 && !w.obj(m).unwrap().on_map);
            assert_eq!(w.obj(o).unwrap().health, h);
            // Once the other unit leaves the position, the member is placed.
            let q = glasnost_sim::objects::occ_quirks(&w);
            w.cells
                .clear_infantry(at(5, 5), glasnost_sim::world::Layer::Ground, o, q);
            run(&mut w, 5);
            assert!(w.obj(m).unwrap().on_map, "placed once the position is free");
        } else {
            assert_eq!(
                st,
                Some(Stage::Placed),
                "a different standing position is shared"
            );
            assert!(w.obj(m).unwrap().on_map);
            assert_eq!(w.obj(o).unwrap().health, h);
        }
    }
}

/// Makes the cell water (a tile of a one-cell water set), as `claims_s0513_contract` does for the
/// whole map.
fn wet(w: &mut World, c: CellXY) {
    let mut cat = (*testkit::catalogue()).clone();
    let first = cat.tiles.len() as i32;
    let mut water = cat.tiles[0].clone();
    if let Some(Some(cell)) = water.cells.first_mut() {
        cell.terrain_code = 9;
    }
    cat.tiles.push(water);
    cat.allow_tiberium.push(false);
    w.catalogue = Some(std::sync::Arc::new(cat));
    w.cells.grid.get_mut(c).unwrap().tile = first;
    glasnost_sim::load::recalc_cells(w, &[c]);
}

#[test]
fn claim50_a_terrain_change_since_the_check_refuses_once_and_unchanged_fatal_terrain_does_not() {
    // spec: S-0513 claim 50 (a terrain change since the passed check that makes the landing no longer enterable refuses the placement; unchanged landing terrain that already implied an explicit arrival death or sinking under claim 38 retains that fate: the safety validation is not a new refusal of every inherently fatal destination)
    // Changed after the check: one refusal, then the full check passes again and claim 38's fate applies.
    let mut w = game(&rules());
    let m = spawn(&mut w, "CART", ALPHA, 0, 0);
    to_away(&mut w, m);
    wet(&mut w, at(5, 5));
    step(&mut w);
    assert_eq!(
        stage(&w, m),
        Some(Stage::Blocked),
        "the change refused the placement"
    );
    assert!(refused(&w, m));
    assert!(w.obj(m).unwrap().health > 0 && !w.obj(m).unwrap().on_map);
    for _ in 0..4 {
        step(&mut w);
    }
    let sank = w
        .obj(m)
        .is_none_or(|o| o.health <= 0 || glasnost_sim::naval::sinking(&w, m));
    assert!(
        sank,
        "then the vessel-less vehicle meets claim 38 on the water"
    );
    // Control: water from the start. The check passes with the fatal terrain, the placement is not
    // refused, and the explicit fate follows.
    let mut w = game(&rules());
    let m = spawn(&mut w, "CART", ALPHA, 0, 0);
    wet(&mut w, at(5, 5));
    to_away(&mut w, m);
    step(&mut w);
    assert_eq!(
        stage(&w, m),
        Some(Stage::Placed),
        "unchanged fatal terrain is not a refusal"
    );
    assert!(!refused(&w, m));
    for _ in 0..3 {
        step(&mut w);
    }
    let sank = w
        .obj(m)
        .is_none_or(|o| o.health <= 0 || glasnost_sim::naval::sinking(&w, m));
    assert!(sank, "claim 38's fate kept");
}
