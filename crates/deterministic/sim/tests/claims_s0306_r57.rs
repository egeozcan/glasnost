//! Claim tests of S-0306 revision 57, claim 17 (Q-2212): a projectile
//! whose target object is a superweapon warp member that has departed, on
//! synthetic rules (`tests/sw_common`). Every name and number is invented.
//!
//! Scenario P11 run f is played through whole ticks with a real warp. The
//! other tests set apart the member's update (`superweapons::warp::turn`)
//! from the projectile's (`combat::projectile::turn`), as S-0513's SW12
//! part i does, so that the member stays off the map for as long as a test
//! needs. A member put straight into its last leaving update (`Expired`)
//! departs through the real departure, landing check and placement.

mod sw_common;

use glasnost_map::CellXY;
use glasnost_num::{F64, Rounding};
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{Decimal, ProjectileType, WeaponType};
use glasnost_sim::combat::effects::Effect;
use glasnost_sim::combat::projectile::{self, launch};
use glasnost_sim::superweapons::warp::{self, Member, Stage};
use glasnost_sim::superweapons::{ProtectKind, protect};
use glasnost_sim::world::{Layer, ObjectId, Pos, Target, World};
use sw_common::*;

const DECK: i32 = 416; // spec: S-0307 Constants (bridge deck height 416 leptons above the terrain)

fn n(v: i64) -> F64 {
    F64::from_i64(v, Rounding::TowardZero)
}

/// The warp rules of intake 60's tests, a slow steering test projectile
/// whose arm time outlasts its flight (P11 run f: neither arrival row
/// applies), a straight one (Liner) and an anti-air straight one (Flak).
fn rules() -> Ruleset {
    let mut r = ruleset();
    {
        let g = &mut r.sim.general.superweapons;
        g.chrono_placement = Some("MARK".into());
        g.chrono_blast = Some("BLASTS".into());
        g.chrono_blast_dest = Some("BLASTD".into());
        g.chrono_delay = Some(7);
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
    r.sim.general.missile_rot_var = Some(Decimal("0".into()));
    r.sim.projectiles.insert(
        "Homer".into(),
        ProjectileType {
            rot: Some(40),
            acceleration: Some(3),
            arm: Some(100_000),
            ..Default::default()
        },
    );
    r.sim.projectiles.insert(
        "Line".into(),
        ProjectileType {
            ..Default::default()
        },
    );
    r.sim.projectiles.insert(
        "Sky".into(),
        ProjectileType {
            aa: Some(true),
            ..Default::default()
        },
    );
    for (name, projectile, speed) in [
        ("Seeker", "Homer", 30),
        ("Liner", "Line", 30),
        ("Flak", "Sky", 40),
    ] {
        r.sim.weapons.insert(
            name.into(),
            WeaponType {
                damage: Some(40),
                rof: Some(5000),
                range: Some(Decimal("20".into())),
                speed: Some(speed),
                projectile: Some(projectile.into()),
                warhead: Some("HE".into()),
                ..Default::default()
            },
        );
    }
    r
}

fn weapon(w: &World, name: &str) -> u32 {
    w.rules().combat.weapon_index[name]
}

fn stage(w: &World, id: ObjectId) -> Option<Stage> {
    w.obj(id).and_then(|o| o.sw.warp.as_ref()).map(|m| m.stage)
}

fn pos(w: &World, id: ObjectId) -> Pos {
    w.obj(id).unwrap().pos
}

fn health(w: &World, id: ObjectId) -> i32 {
    w.obj(id).map_or(0, |o| o.health)
}

fn cell_pos(c: CellXY, z: i32) -> Pos {
    Pos::centre(c, z)
}

fn d3(a: Pos, b: Pos) -> i64 {
    let (x, y, z) = (
        i64::from(a.x - b.x),
        i64::from(a.y - b.y),
        i64::from(a.z - b.z),
    );
    (x * x + y * y + z * z).isqrt()
}

/// A game with ALPHA's teleport source ready and an unarmed ALPHA firer
/// well outside the warp block.
fn ready(r: &Ruleset) -> (World, ObjectId) {
    let mut w = game(r);
    spawn(&mut w, "CHRONOB", ALPHA, -9, -9);
    ready_tick(&mut w, ALPHA, SOURCE, 300);
    let f = spawn(&mut w, "BUGGY", ALPHA, -7, 2);
    (w, f)
}

/// Launches a projectile of `weapon` from `firer` at `target`, then puts it
/// at `p` with velocity `v` (as `claims_s0306`'s placements do).
fn shoot(
    w: &mut World,
    firer: ObjectId,
    weapon_name: &str,
    target: Target,
    p: Pos,
    v: [i64; 3],
) -> ObjectId {
    let wp = weapon(w, weapon_name);
    let fp = pos(w, firer);
    let id = launch(w, firer, wp, target, Pos::new(fp.x, fp.y, fp.z + 100), 40).expect("a launch");
    let x = w.projectiles.get_mut(&id).unwrap();
    x.pos = p;
    x.vel = v.map(n);
    x.prev_cell = p.cell();
    x.omega = 1_000_000;
    id
}

/// What one projectile turn did.
#[derive(Debug, PartialEq)]
enum Out {
    Flying(Pos),
    Detonated(Pos),
    Removed,
}

fn step_projectile(w: &mut World, id: ObjectId) -> Out {
    w.effects.clear();
    projectile::turn(w, id);
    w.tick += 1;
    if let Some(p) = w.projectiles.get(&id) {
        return Out::Flying(p.pos);
    }
    for e in &w.effects {
        if let Effect::Detonation { pos, .. } = e {
            return Out::Detonated(*pos);
        }
    }
    Out::Removed
}

fn target_of(w: &World, id: ObjectId) -> Option<Target> {
    w.projectiles.get(&id).and_then(|p| p.target)
}

/// Gives `m` a warp in its last leaving update (S-0513 T2: the wait has
/// expired, it departs in its next update) with landing cell `to`.
fn expire(w: &mut World, m: ObjectId, to: CellXY) {
    let z = w.surface_at(to.x * 256 + 128, to.y * 256 + 128);
    w.obj_mut(m).unwrap().sw.warp = Some(Member {
        stage: Stage::Expired,
        house: ALPHA,
        landing: cell_pos(to, z),
        redirected: false,
        fell_back: false,
        refused: false,
        entry_ok: false,
    });
}

/// Marks `c` as a high-bridge cell (as `claims_s0208_teleport`'s bridge
/// world does).
fn set_deck(w: &mut World, c: CellXY) {
    let i = w.cells.idx(c).unwrap();
    w.cells.deck[i] = true;
    w.cells.deck_entry[i] = true;
    w.cells.bridge_axis[i] = glasnost_sim::cells::Axis::Y;
}

/// Puts vehicle `m` on the deck of its own cell, which becomes a
/// high-bridge cell.
fn onto_deck(w: &mut World, m: ObjectId) {
    let p = pos(w, m);
    set_deck(w, p.cell());
    let z = w.surface_at(p.x, p.y) + DECK;
    let o = w.obj_mut(m).unwrap();
    o.layer = Layer::Deck;
    o.pos = Pos::new(p.x, p.y, z);
}

/// An object made temporarily invulnerable on cell `c`, so that a landing
/// check there kills the member (S-0513 claim 37).
fn guard(w: &mut World, dx: i32, dy: i32) -> ObjectId {
    let g = spawn(w, "BUGGY", BETA, dx, dy);
    protect::start(w, g, ProtectKind::Curtain, 500);
    g
}

// --- P11 run f ---------------------------------------------------------------

#[test]
fn claim17_p11_run_f_the_projectile_keeps_the_member_and_hits_it_at_its_landing_point() {
    // spec: S-0306 claim 17 (a projectile whose target object is a departed warp member keeps it; a steering projectile steers towards the member's old position and, from its first turn after the member appears, turns towards the landing point; once it has appeared it is found in its landing cell like any object and its phasing in does not stop a hit) and scenario P11 run f
    let (mut w, f) = ready(&rules());
    let v = spawn(&mut w, "BUGGY", BETA, 0, 0);
    let old = pos(&w, v);
    fire_now(&mut w, ALPHA, SOURCE, at(0, 0));
    fire(&mut w, ALPHA, WARP, at(5, 5));
    let tl = w.tick;
    step(&mut w);
    while w.tick <= tl + 50 {
        step(&mut w);
    }
    // The shot is in flight before departure (t_L + 62).
    let fp = pos(&w, f);
    let id = shoot(
        &mut w,
        f,
        "Seeker",
        Target::Object(v),
        Pos::new(fp.x, fp.y, fp.z + 100),
        [8, -4, 0],
    );
    let start_health = health(&w, v);
    let landing = cell_pos(at(5, 5), 0);
    let mut departed_at = None;
    let mut appeared_at = None;
    let mut last_old = i64::MAX;
    let mut last_landing = i64::MAX;
    let mut nearer_landing = 0;
    let mut detonated = false;
    for _ in 0..600 {
        step(&mut w);
        let t = w.tick - 1;
        let s = stage(&w, v);
        if departed_at.is_none() && matches!(s, Some(Stage::Blocked | Stage::Away)) {
            departed_at = Some(t);
            assert!(!w.obj(v).unwrap().on_map, "off the map until it appears");
        }
        if departed_at.is_some() && appeared_at.is_none() && w.obj(v).is_some_and(|o| o.on_map) {
            appeared_at = Some(t);
            assert_eq!(
                pos(&w, v).cell(),
                at(5, 5),
                "it appears at its landing point"
            );
        }
        let Some(p) = w.projectiles.get(&id) else {
            detonated = true;
            break;
        };
        assert_eq!(
            p.target,
            Some(Target::Object(v)),
            "tick {t}: it keeps the member"
        );
        if appeared_at.is_none() {
            let d = d3(p.pos, old);
            assert!(
                d <= last_old,
                "tick {t}: towards the old position, {d} after {last_old}"
            );
            last_old = d;
        } else {
            let d = d3(p.pos, landing);
            if d < last_landing {
                nearer_landing += 1;
            }
            last_landing = d;
        }
    }
    let departed_at = departed_at.expect("the member departed");
    let appeared_at = appeared_at.expect("the member appeared");
    assert_eq!(
        departed_at,
        tl + 62,
        "departure in t_L + 62 (S-0513 claim 39)"
    );
    assert_eq!(
        appeared_at,
        tl + 63,
        "a free landing cell: it appears in its next update"
    );
    assert!(detonated, "the projectile detonated");
    assert!(nearer_landing > 10, "it turned towards the landing point");
    assert!(
        health(&w, v) < start_health,
        "the member loses health at its landing point: {} of {start_health}",
        health(&w, v)
    );
    assert_eq!(w.projectiles.len(), 0, "no second projectile");
    let _ = landing;
}

// --- While the member is off the map -------------------------------------------

/// A world with member `m` (a BETA vehicle at (0, 0)) departed: off the map,
/// its landing check passed for (5, 5), waiting to be placed.
fn departed() -> (World, ObjectId, ObjectId) {
    let (mut w, f) = ready(&rules());
    let m = spawn(&mut w, "BUGGY", BETA, 0, 0);
    expire(&mut w, m, at(5, 5));
    assert!(warp::turn(&mut w, m));
    assert_eq!(stage(&w, m), Some(Stage::Away));
    assert!(!w.obj(m).unwrap().on_map);
    (w, f, m)
}

#[test]
fn claim17_a_steering_projectile_detonates_at_the_old_position_and_area_damage_misses_the_member() {
    // spec: S-0306 claim 17 (every rule that reads the target's position reads the position the member had at departure until it appears: the steering-distance row of T1 and claim 20 place a detonation there; while the member is off the map it is in no cell, so neither the proximity row of T1 nor the detonation's area damage, S-0303, finds it)
    let (mut w, f, m) = departed();
    let old = pos(&w, m);
    // Another vehicle now stands in the cell the member left.
    let other = spawn(&mut w, "BUGGY", BETA, 0, 0);
    let (hm, ho) = (health(&w, m), health(&w, other));
    let id = shoot(
        &mut w,
        f,
        "Seeker",
        Target::Object(m),
        Pos::new(old.x - 600, old.y, old.z + 100),
        [20, 0, 0],
    );
    let mut out = Out::Removed;
    for _ in 0..200 {
        out = step_projectile(&mut w, id);
        match out {
            Out::Flying(_) => assert_eq!(target_of(&w, id), Some(Target::Object(m))),
            _ => break,
        }
    }
    let Out::Detonated(at_) = out else {
        panic!("no detonation: {out:?}")
    };
    assert_eq!(at_, old, "at the member's old position (its target point)");
    assert_eq!(stage(&w, m), Some(Stage::Away), "still off the map");
    assert_eq!(
        health(&w, m),
        hm,
        "the area damage does not reach the member"
    );
    assert!(health(&w, other) < ho, "it does reach what stands there");
}

#[test]
fn claim17_a_straight_projectile_keeps_its_stored_target_point() {
    // spec: S-0306 claim 17 (a straight or arcing projectile retains its stored target point K, claim 9, as for any target, and its flight and detonation remain governed by claim 10 and T1)
    let (mut w, _) = ready(&rules());
    let f = spawn(&mut w, "BUGGY", ALPHA, -3, 0);
    let m = spawn(&mut w, "BUGGY", BETA, 0, 0);
    expire(&mut w, m, at(5, 5));
    let old = pos(&w, m);
    let id = shoot(
        &mut w,
        f,
        "Liner",
        Target::Object(m),
        Pos::new(old.x - 1500, old.y, 1500),
        [10, 0, 0],
    );
    let k = w.projectiles[&id].stored;
    assert!(warp::turn(&mut w, m), "departure");
    for _ in 0..3 {
        assert!(matches!(step_projectile(&mut w, id), Out::Flying(_)));
        assert_eq!(target_of(&w, id), Some(Target::Object(m)));
        assert_eq!(w.projectiles[&id].stored, k);
    }
    assert!(warp::turn(&mut w, m), "placement");
    assert_eq!(stage(&w, m), Some(Stage::Placed));
    assert!(matches!(step_projectile(&mut w, id), Out::Flying(_)));
    assert_eq!(target_of(&w, id), Some(Target::Object(m)));
    assert_eq!(
        w.projectiles[&id].stored, k,
        "K is not moved to the landing point"
    );
}

#[test]
fn claim17_a_hit_while_the_member_phases_in_after_it_appears() {
    // spec: S-0306 claim 17 (once the member has appeared it is found in its landing cell like any object, and its phasing in does not stop a hit: no row of S-0302 claim 8 is for it)
    let (mut w, f, m) = departed();
    assert!(warp::turn(&mut w, m), "placement");
    assert_eq!(stage(&w, m), Some(Stage::Placed));
    assert!(warp::phasing(&w, m), "phasing in");
    let p = pos(&w, m);
    let h = health(&w, m);
    let id = shoot(
        &mut w,
        f,
        "Seeker",
        Target::Object(m),
        Pos::new(p.x - 400, p.y, p.z + 100),
        [20, 0, 0],
    );
    let mut out = Out::Removed;
    for _ in 0..200 {
        out = step_projectile(&mut w, id);
        if !matches!(out, Out::Flying(_)) {
            break;
        }
    }
    assert!(matches!(out, Out::Detonated(_)), "{out:?}");
    assert!(warp::phasing(&w, m), "still phasing in");
    assert!(health(&w, m) < h, "hit while phasing in");
}

// --- The loss at a landing-check kill ----------------------------------------------

#[test]
fn claim17_a_member_killed_by_its_landing_check_leaves_its_departure_cell() {
    // spec: S-0306 claim 17 (a member that its landing check kills, S-0513 claim 37, leaves the game before it appears, so the projectile's target becomes the cell the member stood in at departure; a member on the ground is not airborne)
    // At its departure update.
    let (mut w, f) = ready(&rules());
    let m = spawn(&mut w, "BUGGY", BETA, 0, 0);
    guard(&mut w, 5, 5);
    expire(&mut w, m, at(5, 5));
    let old = pos(&w, m);
    let id = shoot(
        &mut w,
        f,
        "Seeker",
        Target::Object(m),
        Pos::new(old.x - 1500, old.y, 300),
        [10, 0, 0],
    );
    warp::turn(&mut w, m);
    assert!(
        w.obj(m).is_none_or(|o| o.health <= 0),
        "killed by its landing check"
    );
    assert_eq!(target_of(&w, id), Some(Target::Cell(old.cell())));
    // In a later landing check, after the projectile has kept the member
    // through its own turns while it was off the map.
    let (mut w, f, m) = departed();
    let old = pos(&w, m);
    let id = shoot(
        &mut w,
        f,
        "Seeker",
        Target::Object(m),
        Pos::new(old.x - 1500, old.y, 300),
        [10, 0, 0],
    );
    assert!(matches!(step_projectile(&mut w, id), Out::Flying(_)));
    assert_eq!(target_of(&w, id), Some(Target::Object(m)));
    guard(&mut w, -5, 5);
    // The landing retry (S-0513 T2: a failed check is made again in each later update) towards the guarded cell.
    if let Some(mm) = w.obj_mut(m).unwrap().sw.warp.as_mut() {
        mm.stage = Stage::Blocked;
        mm.landing = cell_pos(at(-5, 5), 0);
    }
    warp::turn(&mut w, m);
    assert!(
        w.obj(m).is_none_or(|o| o.health <= 0),
        "killed by its landing check"
    );
    assert_eq!(target_of(&w, id), Some(Target::Cell(old.cell())));
}

#[test]
fn claim17_a_departed_member_removed_from_the_game_leaves_its_departure_cell() {
    // spec: S-0306 claim 17 (when the target object leaves the game the projectile's target becomes the cell the target stood in if it was not airborne; for a departed member, the cell it stood in at departure)
    let (mut w, f, m) = departed();
    let old = pos(&w, m);
    let id = shoot(
        &mut w,
        f,
        "Seeker",
        Target::Object(m),
        Pos::new(old.x - 1500, old.y, 300),
        [10, 0, 0],
    );
    remove(&mut w, m);
    assert_eq!(target_of(&w, id), Some(Target::Cell(old.cell())));
}

/// A member on the deck of a high bridge at (0, 0), killed by its landing
/// check at departure; returns the projectile's target afterwards and the
/// world, the projectile and the departure cell.
fn deck_member_killed(quirk: bool) -> (Option<Target>, World, ObjectId, CellXY) {
    let mut r = rules();
    r.globals.quirks.warp_deck_departure_airborne = quirk;
    let (mut w, f) = ready(&r);
    let m = spawn(&mut w, "BUGGY", BETA, 0, 0);
    onto_deck(&mut w, m);
    guard(&mut w, 5, 5);
    expire(&mut w, m, at(5, 5));
    let old = pos(&w, m);
    let id = shoot(
        &mut w,
        f,
        "Seeker",
        Target::Object(m),
        Pos::new(old.x - 1500, old.y, old.z + 200),
        [10, 0, 0],
    );
    warp::turn(&mut w, m);
    assert!(
        w.obj(m).is_none_or(|o| o.health <= 0),
        "killed by its landing check"
    );
    (target_of(&w, id), w, id, old.cell())
}

#[test]
fn quirk_warp_deck_departure_airborne_on_a_deck_member_killed_leaves_no_target() {
    // spec: S-0306 claim 17 and Edge cases (quirks.warp_deck_departure_airborne on: a member that departed from a high-bridge deck counts as airborne from departure until it appears; at a landing-check kill the projectile loses its target by claim 17's airborne classification, and a steering one left without target climbs, claim 15)
    let (t, mut w, id, _) = deck_member_killed(true);
    assert_eq!(t, None, "airborne: nothing");
    let start = w.projectiles[&id].pos.z;
    let mut last = start;
    let safety = w.rules().combat.general.missile_safety_altitude;
    for _ in 0..200 {
        match step_projectile(&mut w, id) {
            Out::Flying(p) => {
                assert!(p.z >= last, "it climbs");
                last = p.z;
            }
            Out::Detonated(p) => {
                // spec: S-0306 claim 15 and T1 (a steering projectile without target climbs and detonates at the missile safety altitude)
                assert!(p.z > start, "above where it lost its target");
                assert!(p.z - w.surface_at(p.x, p.y) >= safety);
                return;
            }
            Out::Removed => panic!("removed"),
        }
    }
    panic!("no detonation");
}

#[test]
fn quirk_warp_deck_departure_airborne_off_a_deck_member_killed_leaves_its_deck_cell() {
    // spec: S-0306 Edge cases (quirks.warp_deck_departure_airborne off: until it appears the member counts as standing on the deck it departed from, so it is not airborne and its loss gives the deck cell it stood in)
    let (t, _, _, cell) = deck_member_killed(false);
    assert_eq!(t, Some(Target::Cell(cell)));
}

// --- The anti-air row on a member counted as airborne -------------------------------

/// A member that departed from the deck at (0, 0), landing at `to` (a deck
/// cell when `deck_landing`), and an anti-air shot 50 leptons short of its
/// kept position and 80 above it, its stored point far beyond: returns the
/// shot's first turn.
fn flak_at_deck_member(quirk: bool, deck_landing: bool) -> Out {
    let mut r = rules();
    r.globals.quirks.warp_deck_departure_airborne = quirk;
    let (mut w, f) = ready(&r);
    let m = spawn(&mut w, "BUGGY", BETA, 0, 0);
    onto_deck(&mut w, m);
    let to = at(5, 5);
    if deck_landing {
        set_deck(&mut w, to);
    }
    expire(&mut w, m, to);
    assert!(warp::turn(&mut w, m));
    assert_eq!(stage(&w, m), Some(Stage::Away), "its landing check passed");
    let old = pos(&w, m);
    let id = shoot(
        &mut w,
        f,
        "Flak",
        Target::Object(m),
        Pos::new(old.x - 60, old.y, old.z + 80),
        [10, 0, 0],
    );
    // Its stored point far beyond the member, so that only the anti-air
    // row can detonate it near the member (S-0306 T1).
    w.projectiles.get_mut(&id).unwrap().stored = Pos::new(old.x + 3000, old.y, old.z + 80);
    step_projectile(&mut w, id)
}

#[test]
fn quirk_warp_deck_departure_airborne_on_the_anti_air_row_finds_the_member() {
    // spec: S-0306 claim 17 and T1 (anti-air with an airborne target: d₃ from P′ to the target's reference point < 128; a member that departed from a high-bridge deck counts as airborne until it appears; quirks.warp_deck_departure_airborne on)
    assert!(matches!(
        flak_at_deck_member(true, false),
        Out::Detonated(_)
    ));
}

#[test]
fn quirk_warp_deck_departure_airborne_off_the_anti_air_row_does_not() {
    // spec: S-0306 Edge cases (quirks.warp_deck_departure_airborne off: the member counts as standing on the deck it departed from, not airborne)
    assert!(matches!(flak_at_deck_member(false, false), Out::Flying(_)));
}

#[test]
fn claim17_a_passed_landing_check_for_a_deck_cell_stands_the_member_on_a_deck() {
    // spec: S-0306 claim 17 (a member between departure and appearance counts as standing on a deck once its landing check has passed for a landing cell on a deck: not airborne)
    assert!(matches!(flak_at_deck_member(true, true), Out::Flying(_)));
}

#[test]
fn claim17_a_member_departed_from_the_ground_is_not_airborne() {
    // spec: S-0306 claim 17 (counted by its height above the surface beneath its kept position: a member that departed from the ground is not airborne, so the anti-air row does not apply)
    let (mut w, f, m) = departed();
    let old = pos(&w, m);
    let id = shoot(
        &mut w,
        f,
        "Flak",
        Target::Object(m),
        Pos::new(old.x - 60, old.y, old.z + 80),
        [10, 0, 0],
    );
    // Its stored point far beyond the member, so that only the anti-air
    // row can detonate it near the member (S-0306 T1).
    w.projectiles.get_mut(&id).unwrap().stored = Pos::new(old.x + 3000, old.y, old.z + 80);
    assert!(matches!(step_projectile(&mut w, id), Out::Flying(_)));
}
