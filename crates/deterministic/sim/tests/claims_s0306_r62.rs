//! Claim tests of S-0306 revision 62, claim 17 (Q-2231, Q-2232): the
//! special hits of a projectile whose target object is a superweapon warp
//! member that has departed or is phasing in, the deck count after a
//! refused placement, and the ground render layer of a warp member, on
//! synthetic rules (`tests/sw_common`). Every name and number is invented.
//! Revision 65's ground render layer after the arrival hold (M8 step A3
//! intake 84) is the last section.
//!
//! As in `claims_s0306_r57`, the member's update (`superweapons::warp::turn`)
//! is set apart from the projectile's (`combat::projectile::turn`), as
//! S-0513's SW12 part i does, so that the member stays off the map or
//! phasing in for as long as a test needs. A member put straight into its
//! last leaving update (`Expired`) departs through the real departure,
//! landing check and placement.

mod sw_common;

use glasnost_map::CellXY;
use glasnost_num::{F64, Rounding};
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{Decimal, ProjectileType, UnitType, WarheadType, WeaponType};
use glasnost_sim::bomb::{self, Bomb};
use glasnost_sim::cloak::disguise::{Disguise, DisguiseAs};
use glasnost_sim::combat::effects::Effect;
use glasnost_sim::combat::projectile::{self, launch, target_airborne};
use glasnost_sim::rules::Kind;
use glasnost_sim::superweapons::warp::{self, Member, Stage};
use glasnost_sim::world::{Layer, ObjectId, Pos, Target, World};
use glasnost_sim::{lift, temporal, testkit};
use sw_common::*;

const DECK: i32 = 416; // spec: S-0307 Constants (bridge deck height 416 leptons above the terrain)

fn n(v: i64) -> F64 {
    F64::from_i64(v, Rounding::TowardZero)
}

fn d(s: &str) -> Decimal {
    Decimal(s.into())
}

/// The special-hit parts of P11 runs g and h, in the order of S-0514
/// claim 20's table that they appear in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Part {
    /// a: takeover.
    Take,
    /// b: bomb planting.
    Plant,
    /// c: electric charging.
    Zap,
    /// d: temporal erasure.
    Erase,
    /// e: air-strike designation.
    Call,
    /// f: bomb disarming.
    Defuse,
    /// g: disguise-making.
    Copy,
    /// h (run h): locomotor seizing.
    Grab,
}

const PARTS: [Part; 7] = [
    Part::Take,
    Part::Plant,
    Part::Zap,
    Part::Erase,
    Part::Call,
    Part::Defuse,
    Part::Copy,
];

impl Part {
    /// The firer type and its weapon.
    fn firer(self) -> (&'static str, &'static str) {
        match self {
            Part::Take => ("CTRL", "TakeRay"),
            Part::Plant => ("BOMBER", "PlantRay"),
            Part::Zap => ("ZAPPER", "ZapRay"),
            Part::Erase => ("TIMER", "EraseRay"),
            Part::Call => ("DESIG", "CallRay"),
            Part::Defuse => ("ENG", "DefuseRay"),
            Part::Copy => ("MIRAGE", "CopyRay"),
            Part::Grab => ("GRABBER", "GrabRay"),
        }
    }
}

fn register(r: &mut Ruleset, reg: &str, name: &str) {
    let l = r.registries.entry(reg.into()).or_default();
    if !l.iter().any(|x| x == name) {
        l.push(name.into());
    }
}

/// The warp rules of intake 60's tests; a steering test projectile with a
/// short arm time (Dart) carrying one special-hit warhead per part, its
/// firers, the strike aircraft JET, a straight ranged test projectile
/// (Pass) and an anti-air straight one (Flak).
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
    for nm in ["MARK", "BLASTS", "BLASTD", "WARPA"] {
        r.sim
            .animations
            .insert(nm.into(), glasnost_rules::sim::AnimationType::default());
        register(&mut r, "animations", nm);
    }
    r.sim.general.missile_rot_var = Some(d("0"));
    // No gravity, so that a straight shot flies level (S-0306 claim 10:
    // V_z falls by g in each turn), as run j's shot does.
    r.sim.general.gravity = Some(0);
    r.sim.projectiles.insert(
        "Dart".into(),
        ProjectileType {
            rot: Some(40),
            acceleration: Some(3),
            arm: Some(1),
            ..Default::default()
        },
    );
    r.sim.projectiles.insert(
        "Pass".into(),
        ProjectileType {
            ranged: Some(true),
            arm: Some(1),
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
    let wh = |f: &dyn Fn(&mut WarheadType)| {
        let mut h = WarheadType {
            verses: Some(vec![d("1"); 11]),
            ..Default::default()
        };
        f(&mut h);
        h
    };
    let heads: [(&str, WarheadType); 8] = [
        ("TAKE", wh(&|h| h.mind_control = Some(true))),
        ("PLANT", wh(&|h| h.ivan_bomb = Some(true))),
        ("ZAP", wh(&|h| h.electric_assault = Some(true))),
        ("ERASE", wh(&|h| h.temporal = Some(true))),
        ("CALL", wh(&|h| h.airstrike = Some(true))),
        ("DEFUSE", wh(&|h| h.bomb_disarm = Some(true))),
        ("COPY", wh(&|h| h.makes_disguise = Some(true))),
        (
            "GRAB",
            wh(&|h| {
                h.is_locomotor = Some(true);
                h.locomotor = Some("jumpjet".into());
            }),
        ),
    ];
    for (nm, h) in heads {
        r.sim.warheads.insert(nm.into(), h);
    }
    let weapon = |projectile: &str, warhead: &str, damage: i32, speed: i32| WeaponType {
        damage: Some(damage),
        rof: Some(5000),
        range: Some(d("20")),
        speed: Some(speed),
        projectile: Some(projectile.into()),
        warhead: Some(warhead.into()),
        ..Default::default()
    };
    for (nm, h) in [
        ("TakeRay", "TAKE"),
        ("PlantRay", "PLANT"),
        ("ZapRay", "ZAP"),
        ("EraseRay", "ERASE"),
        ("CallRay", "CALL"),
        ("DefuseRay", "DEFUSE"),
        ("CopyRay", "COPY"),
        ("GrabRay", "GRAB"),
    ] {
        r.sim.weapons.insert(nm.into(), weapon("Dart", h, 20, 30));
    }
    r.sim
        .weapons
        .insert("Passer".into(), weapon("Pass", "HE", 40, 150));
    r.sim
        .weapons
        .insert("Flak".into(), weapon("Sky", "HE", 40, 40));
    let vehicle = |r: &mut Ruleset, nm: &str, primary: &str| {
        let mut t = r.sim.vehicles["BUGGY"].clone();
        t.strength = Some(100_000);
        t.primary = Some(primary.into());
        register(r, "vehicles", nm);
        r.sim.vehicles.insert(nm.into(), t);
    };
    let infantry = |r: &mut Ruleset, nm: &str, primary: &str| {
        let mut t = r.sim.infantry["WALKER"].clone();
        t.strength = Some(100_000);
        t.primary = Some(primary.into());
        register(r, "infantry", nm);
        r.sim.infantry.insert(nm.into(), t);
    };
    vehicle(&mut r, "CTRL", "TakeRay");
    infantry(&mut r, "BOMBER", "PlantRay");
    infantry(&mut r, "ZAPPER", "ZapRay");
    vehicle(&mut r, "TIMER", "EraseRay");
    vehicle(&mut r, "DESIG", "CallRay");
    infantry(&mut r, "ENG", "DefuseRay");
    vehicle(&mut r, "MIRAGE", "CopyRay");
    vehicle(&mut r, "GRABBER", "GrabRay");
    {
        let t = r.sim.vehicles.get_mut("DESIG").unwrap();
        t.combat.airstrike_team = Some(2);
        t.combat.airstrike_team_type = Some("JET".into());
        t.combat.airstrike_recharge_time = Some(100);
    }
    r.sim.aircraft.insert(
        "JET".into(),
        UnitType {
            strength: Some(200),
            speed: Some(30),
            rot: Some(12),
            locomotor: Some(glasnost_rules::sim::Locomotor::Fly),
            primary: Some("Flak".into()),
            ..Default::default()
        },
    );
    register(&mut r, "aircraft", "JET");
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

/// A game under `r` with ALPHA's teleport source ready.
fn ready(r: &Ruleset) -> World {
    let mut w = game(r);
    spawn(&mut w, "CHRONOB", ALPHA, -9, -9);
    ready_tick(&mut w, ALPHA, SOURCE, 300);
    w
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

/// Runs the projectile until it leaves play; gives its last turn.
fn fly(w: &mut World, id: ObjectId) -> Out {
    for _ in 0..300 {
        let out = step_projectile(w, id);
        if !matches!(out, Out::Flying(_)) {
            return out;
        }
    }
    panic!("the projectile is still in flight");
}

/// Runs the projectile until it leaves play; gives its position at the
/// start of its last turn. A special-hit detonation does no area damage
/// (S-0514 claim 20), so it leaves no detonation effect to read.
fn fly_last(w: &mut World, id: ObjectId) -> Pos {
    let mut last = w.projectiles[&id].pos;
    for _ in 0..300 {
        match step_projectile(w, id) {
            Out::Flying(p) => last = p,
            _ => return last,
        }
    }
    panic!("the projectile is still in flight");
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

// --- P11 runs g and h: the special hits --------------------------------------------

/// When the hit falls in the member's warp.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Timing {
    /// Control: no warp, V stands at its landing cell.
    NoWarp,
    /// Run g: departed, off the map (its landing check passed, awaiting
    /// placement).
    Away,
    /// Run h: appeared at its landing point, before its arrival effects.
    Placed,
    /// Run h: in its arrival hold.
    Hold,
}

/// A world with V, a BETA vehicle, at `timing`: departed from (0, 0) towards
/// (5, 5), or standing at (5, 5) with no warp.
fn member_at(timing: Timing) -> (World, ObjectId) {
    let mut w = ready(&rules());
    if timing == Timing::NoWarp {
        let v = spawn(&mut w, "BUGGY", BETA, 5, 5);
        return (w, v);
    }
    let v = spawn(&mut w, "BUGGY", BETA, 0, 0);
    expire(&mut w, v, at(5, 5));
    assert!(warp::turn(&mut w, v), "departure");
    assert_eq!(stage(&w, v), Some(Stage::Away));
    if matches!(timing, Timing::Placed | Timing::Hold) {
        assert!(warp::turn(&mut w, v), "placement");
        assert_eq!(stage(&w, v), Some(Stage::Placed));
    }
    if timing == Timing::Hold {
        assert!(warp::turn(&mut w, v), "arrival effects");
        assert!(matches!(stage(&w, v), Some(Stage::Hold(_))));
    }
    (w, v)
}

/// What a part's hit left.
struct Hit {
    w: World,
    v: ObjectId,
    firer: ObjectId,
    /// The projectile's position at the start of its last turn.
    at: Pos,
    /// V's position when the shot was made (its old position while it is
    /// off the map).
    kept: Pos,
    /// V's health before the shot.
    health: i32,
}

/// Makes `part`'s firer, prepares V as the part asks (part f: a bomb planted
/// by another unit; part g: a disguise as another vehicle type), and fires
/// one steering shot at V from 400 leptons west of its kept position.
fn hit(part: Part, timing: Timing) -> Hit {
    let (mut w, v) = member_at(timing);
    let (ty, wp) = part.firer();
    let firer = spawn(&mut w, ty, ALPHA, -4, 3);
    match part {
        Part::Defuse => {
            let other = spawn(&mut w, "BUGGY", ALPHA, -6, -6);
            let tick = w.tick;
            w.obj_mut(v).unwrap().combat.bomb = Some(Bomb {
                planter: Some(other),
                house: Some(ALPHA),
                planted: tick,
            });
        }
        Part::Copy => {
            let k = w.rules().index_of(Kind::Vehicle, "CART").unwrap();
            w.obj_mut(v).unwrap().cloak.disguise = Some(Disguise {
                what: Some(DisguiseAs::Vehicle(k)),
                house: Some(BETA),
                since: 0,
            });
        }
        _ => {}
    }
    let kept = pos(&w, v);
    let health = health(&w, v);
    let id = shoot(
        &mut w,
        firer,
        wp,
        Target::Object(v),
        Pos::new(kept.x - 400, kept.y, kept.z + 100),
        [20, 0, 0],
    );
    let at = fly_last(&mut w, id);
    Hit {
        w,
        v,
        firer,
        at,
        kept,
        health,
    }
}

/// Whether `part`'s effect happened on V.
fn effect(part: Part, h: &Hit) -> bool {
    let (w, v, f) = (&h.w, h.v, h.firer);
    let o = w.obj(v).unwrap();
    match part {
        Part::Take => o.owner == Some(ALPHA),
        Part::Plant => bomb::carries(w, v),
        // Electric charging acts on buildings only: nothing to see.
        Part::Zap => false,
        Part::Erase => temporal::erased(w, v) && temporal::holds(w, f, v),
        Part::Call => w
            .strike
            .strikes
            .get(&f)
            .is_some_and(|s| s.target == Some(v)),
        Part::Defuse => !bomb::carries(w, v),
        Part::Copy => {
            w.obj(f).unwrap().cloak.disguise.and_then(|d| d.what)
                == Some(DisguiseAs::Vehicle(
                    w.rules().index_of(Kind::Vehicle, "CART").unwrap(),
                ))
        }
        Part::Grab => lift::seized(w, v),
    }
}

#[test]
fn claim17_p11_controls_each_part_acts_on_a_member_with_no_warp() {
    // spec: S-0306 claim 17 and scenario P11 runs g and h (the control run of each part without the warp: the detonation on V gives each part's effect; electric charging acts on buildings only, so part c changes nothing; the special row replaces ordinary damage, S-0514 claim 20)
    for part in PARTS {
        let h = hit(part, Timing::NoWarp);
        assert_eq!(effect(part, &h), part != Part::Zap, "{part:?}");
        assert_eq!(health(&h.w, h.v), h.health, "{part:?}: no damage");
    }
    let h = hit(Part::Grab, Timing::NoWarp);
    assert!(effect(Part::Grab, &h), "a vehicle with no warp is seized");
}

#[test]
fn claim17_p11_run_g_special_hits_act_on_a_member_off_the_map() {
    // spec: S-0306 claim 17 (takeover, bomb planting, bomb disarming, air-strike designation, disguise-making and the erasure lock act on the projectile's target object, so a departed member off the map receives them by each row's own tests; electric charging acts on buildings only, so it does nothing; in every case the selected row still replaces ordinary damage) and scenario P11 run g
    // spec: S-0601 r21 claim 5, S-0519 r18 claims 2, 7 and 16 and S-0516 r13 claim 31 (a departed superweapon warp member that is the target, off the map, is an ordinary target for the takeover, the bomb, the disarm hit, the designation and the disguise copy, by each claim's own tests; S-0306 scenario P11 run g)
    for part in PARTS {
        let h = hit(part, Timing::Away);
        assert!(d3(h.at, h.kept) < 256, "{part:?}: at V's old position");
        assert_eq!(
            stage(&h.w, h.v),
            Some(Stage::Away),
            "{part:?}: still off the map"
        );
        assert!(!h.w.obj(h.v).unwrap().on_map);
        assert_eq!(effect(part, &h), part != Part::Zap, "{part:?}");
        assert_eq!(
            health(&h.w, h.v),
            h.health,
            "{part:?}: V's health is unchanged"
        );
    }
}

#[test]
fn claim17_p11_run_h_special_hits_act_on_a_member_phasing_in() {
    // spec: S-0306 claim 17 (a member phasing in, appeared at its landing point and until its arrival hold ends, receives the special hits by each row's own tests; electric charging does nothing; the selected row replaces ordinary damage) and scenario P11 run h
    // spec: S-0601 r21 claim 5, S-0519 r18 claims 2, 7 and 16 and S-0516 r13 claim 31 (a departed superweapon warp member that is the target, phasing in after it appears, is an ordinary target for the takeover, the bomb, the disarm hit, the designation and the disguise copy, by each claim's own tests; S-0306 scenario P11 run h)
    for timing in [Timing::Placed, Timing::Hold] {
        for part in PARTS {
            let h = hit(part, timing);
            assert!(warp::phasing(&h.w, h.v), "{part:?} {timing:?}: phasing in");
            assert!(h.w.obj(h.v).unwrap().on_map);
            assert_eq!(h.kept.cell(), at(5, 5), "at its landing point");
            assert!(d3(h.at, h.kept) < 256, "{part:?} {timing:?}: on V");
            assert_eq!(effect(part, &h), part != Part::Zap, "{part:?} {timing:?}");
            assert_eq!(
                health(&h.w, h.v),
                h.health,
                "{part:?} {timing:?}: V's health is unchanged"
            );
        }
    }
}

/// The member's stage after each of `n` further updates, one tick apart.
fn stages(w: &mut World, v: ObjectId, n: usize) -> Vec<(Option<Stage>, bool)> {
    let mut out = Vec::new();
    for _ in 0..n {
        warp::turn(w, v);
        out.push((stage(w, v), w.obj(v).is_some_and(|o| o.on_map)));
        w.tick += 1;
    }
    out
}

#[test]
fn claim17_p11_run_h_part_h_no_seizing_from_departure_until_the_hold_ends() {
    // spec: S-0306 claim 17 and Questions for the project lead (engine design, no switch: a locomotor-seizing hit does nothing to a warp member from its departure until its arrival hold ends; the row still replaces ordinary damage; V appears and has its arrival effects on run g's ticks) and scenario P11 run h part h, in both timings
    for timing in [Timing::Away, Timing::Placed, Timing::Hold] {
        let h = hit(Part::Grab, timing);
        assert!(!lift::seized(&h.w, h.v), "{timing:?}: not seized");
        assert_eq!(health(&h.w, h.v), h.health, "{timing:?}: no damage");
        // The same member without the shot, the same number of ticks on.
        let (mut c, cv) = member_at(timing);
        c.tick = h.w.tick;
        let mut w = h.w;
        assert_eq!(
            stages(&mut w, h.v, 12),
            stages(&mut c, cv, 12),
            "{timing:?}: appearance and arrival effects on the same ticks"
        );
    }
}

#[test]
fn claim17_a_seizing_hit_after_the_hold_ends_seizes() {
    // spec: S-0306 claim 17 (the rule of no seizing lasts until the arrival hold ends) and S-0607 claims 1 and 3 (afterwards the member is an ordinary vehicle on the map)
    let (mut w, v) = member_at(Timing::Hold);
    for _ in 0..20 {
        if stage(&w, v).is_none() {
            break;
        }
        warp::turn(&mut w, v);
        w.tick += 1;
    }
    assert_eq!(stage(&w, v), None, "the hold has ended");
    let f = spawn(&mut w, "GRABBER", ALPHA, -4, 3);
    let kept = pos(&w, v);
    let id = shoot(
        &mut w,
        f,
        "GrabRay",
        Target::Object(v),
        Pos::new(kept.x - 400, kept.y, kept.z + 100),
        [20, 0, 0],
    );
    fly_last(&mut w, id);
    assert!(lift::seized(&w, v));
}

#[test]
fn claim17_an_erasure_lock_on_a_member_off_the_map_makes_progress_in_every_turn() {
    // spec: S-0306 claim 17 (the erasure lock acts on a departed member) and S-0513 claim 39a (A lock after the departure: a leader's lock, the member being erased; The erasure goes on: an erasure update, S-0514 claim 7, in each member turn at every stage, before the warp step, an intrinsic exception to claim 51's whole-member-turn contract, S-0203 claim 4); what the lock does to the warp in the original is an Open question of S-0306, scenario P11 run g part d
    let mut h = hit(Part::Erase, Timing::Away);
    let amount = |w: &World, v| w.temporal.locks.get(&v).map(|l| l.amount);
    let start = amount(&h.w, h.v).expect("a lock");
    assert!(glasnost_sim::temporal::erased(&h.w, h.v));
    let (mut appeared, mut held) = (None, None);
    let mut last = start;
    for i in 0..40 {
        step(&mut h.w);
        let s = stage(&h.w, h.v);
        if appeared.is_none() && h.w.obj(h.v).is_some_and(|o| o.on_map) {
            appeared = Some(i);
        }
        if held.is_none() && matches!(s, Some(Stage::Hold(_))) {
            held = Some(i);
        }
        if let Some(a) = amount(&h.w, h.v) {
            assert!(a < last, "tick {i}: an erasure update in every turn");
            last = a;
        }
    }
    assert_eq!(
        appeared,
        Some(0),
        "it appears in its next update, as with no lock"
    );
    assert_eq!(held, Some(1), "its arrival effects in the update after");
    assert!(
        amount(&h.w, h.v).is_none_or(|a| a < start),
        "the erasure goes on through the warp"
    );
}

// --- The ground render layer of a warp member ---------------------------------------

/// An aircraft of BETA high above (2, 2) and a warp member state when
/// `member` (its warp launched, not yet departed).
fn raised(member: bool) -> (World, ObjectId) {
    let mut w = ready(&rules());
    let a = testkit::spawn(&mut w, Kind::Aircraft, "JET", BETA.0, at(2, 2), 0);
    let p = pos(&w, a);
    w.obj_mut(a).unwrap().pos = Pos::new(p.x, p.y, p.z + 600);
    if member {
        expire(&mut w, a, at(5, 5));
        w.obj_mut(a).unwrap().sw.warp.as_mut().unwrap().stage = Stage::Leaving(w.tick);
    }
    (w, a)
}

#[test]
fn claim17_a_warp_member_is_in_the_ground_layer_for_claim_20c() {
    // spec: S-0306 claim 17 (a warp member is in the ground render layer from its warp's launch until at least the end of its arrival hold, whatever its height) and claim 20 (c) (threshold 42 for a target that is not airborne or is airborne in the ground render layer, 128 for one airborne in another layer)
    let mut seen = Vec::new();
    for member in [false, true] {
        let (mut w, a) = raised(member);
        assert!(projectile::target_airborne(&w, a), "airborne");
        let f = spawn(&mut w, "BUGGY", ALPHA, -6, 2);
        let rp = w.reference_point(a);
        // Out of its cell's list, so that T1's proximity row does not find
        // it before the passed-by row, as for a member off the map.
        w.cells.leave_both(rp.cell(), a);
        // A straight ranged shot, its stored point the target's reference
        // point, passes by it: last before it 70 short, then 80 past it.
        let id = shoot(
            &mut w,
            f,
            "Passer",
            Target::Object(a),
            Pos::new(rp.x - 370, rp.y, rp.z),
            [150, 0, 0],
        );
        w.projectiles.get_mut(&id).unwrap().stored = rp;
        let out = fly(&mut w, id);
        let Out::Detonated(p) = out else {
            panic!("{out:?}")
        };
        seen.push((p, rp));
    }
    let (p, rp) = seen[0];
    assert_eq!(p, rp, "no warp: threshold 128, moved onto the target point");
    let (p, rp) = seen[1];
    assert_ne!(p, rp, "a warp member: threshold 42, its own position");
    assert!((64..128).contains(&d3(p, rp)), "{}", d3(p, rp));
}

#[test]
fn claim17_a_warp_member_is_in_the_ground_layer_for_the_erasure_test() {
    // spec: S-0306 claim 17 (a warp member is in the ground render layer from its warp's launch until at least the end of its arrival hold) and S-0514 claim 1 (an object target must be in the ground render layer, otherwise the erasing effect does nothing)
    for member in [false, true] {
        let (mut w, a) = raised(member);
        let f = spawn(&mut w, "TIMER", ALPHA, -2, 2);
        temporal::deliver(&mut w, Some(f), Some(Target::Object(a)));
        assert_eq!(temporal::erased(&w, a), member, "member {member}");
    }
}

// --- P11 run j: the threshold for a member that departed from a deck --------------

#[test]
fn claim17_p11_run_j_a_deck_departed_member_uses_threshold_42() {
    // spec: S-0306 claim 17 (a member that departed from a high-bridge deck counts as airborne, quirks.warp_deck_departure_airborne on, the default, but is in the ground render layer, so claim 20 (c) uses 42) and scenario P11 run j
    let r = rules();
    assert!(r.globals.quirks.warp_deck_departure_airborne);
    let mut w = ready(&r);
    let v = spawn(&mut w, "BUGGY", BETA, 0, 0);
    onto_deck(&mut w, v);
    expire(&mut w, v, at(5, 5));
    assert!(warp::turn(&mut w, v));
    assert_eq!(stage(&w, v), Some(Stage::Away), "off the map");
    assert!(target_airborne(&w, v), "counted as airborne");
    let f = spawn(&mut w, "BUGGY", ALPHA, -6, 2);
    let rp = w.reference_point(v);
    let id = shoot(
        &mut w,
        f,
        "Passer",
        Target::Object(v),
        Pos::new(rp.x - 370, rp.y, rp.z),
        [150, 0, 0],
    );
    w.projectiles.get_mut(&id).unwrap().stored = rp;
    let out = fly(&mut w, id);
    let Out::Detonated(p) = out else {
        panic!("{out:?}")
    };
    assert_eq!(stage(&w, v), Some(Stage::Away), "still off the map");
    assert_eq!(p, Pos::new(rp.x + 80, rp.y, rp.z), "its own position");
    assert!((64..128).contains(&d3(p, rp)));
}

// --- P11 run i: the deck count after a refused placement ---------------------------

/// V on the deck at (0, 0), departed with its check passed for the deck
/// cell (5, 5), and a vehicle then standing on that deck, so that the
/// placement is refused (S-0513 claim 50). Gives the world, V and the
/// blocker, with V back in its landing retry.
fn refused(quirk: bool) -> (World, ObjectId, ObjectId) {
    let mut r = rules();
    r.globals.quirks.warp_deck_departure_airborne = quirk;
    let mut w = ready(&r);
    let v = spawn(&mut w, "BUGGY", BETA, 0, 0);
    onto_deck(&mut w, v);
    set_deck(&mut w, at(5, 5));
    expire(&mut w, v, at(5, 5));
    assert!(warp::turn(&mut w, v));
    assert_eq!(
        stage(&w, v),
        Some(Stage::Away),
        "the check passed for a deck cell"
    );
    assert!(
        !target_airborne(&w, v),
        "on a deck while the pass awaits placement"
    );
    // A vehicle newly standing on that deck refuses the placement (S-0513
    // claim 50: no vehicle shares a cell's layer with another).
    let b = spawn(&mut w, "BUGGY", BETA, 5, 5);
    onto_deck(&mut w, b);
    w.cells.leave_both(at(5, 5), b);
    w.cells.enter(at(5, 5), Layer::Deck, b, false);
    assert!(warp::turn(&mut w, v));
    let m = w.obj(v).unwrap().sw.warp.clone().unwrap();
    assert_eq!(m.stage, Stage::Blocked, "refused");
    assert!(m.refused);
    assert!(!w.obj(v).unwrap().on_map);
    (w, v, b)
}

/// Whether an anti-air shot 60 leptons short of V's kept position and 80
/// above it, its stored point far beyond, detonates in its first turn (the
/// anti-air row of T1 finds only an airborne target).
fn aa_finds(w: &World, v: ObjectId) -> bool {
    let mut w = w.clone();
    let f = spawn(&mut w, "BUGGY", ALPHA, -6, 2);
    let old = pos(&w, v);
    let id = shoot(
        &mut w,
        f,
        "Flak",
        Target::Object(v),
        Pos::new(old.x - 60, old.y, old.z + 80),
        [10, 0, 0],
    );
    w.projectiles.get_mut(&id).unwrap().stored = Pos::new(old.x + 3000, old.y, old.z + 80);
    matches!(step_projectile(&mut w, id), Out::Detonated(_))
}

#[test]
fn claim17_p11_run_i_a_refused_placement_ends_the_deck_count() {
    // spec: S-0306 claim 17 (engine design, no switch: a departed member counts as standing on a deck only while a check that passed for a deck landing cell awaits its placement; after a refused placement it counts by its height until a later deck pass; Q-2232) and scenario P11 run i, the refusal
    let (w, v, _) = refused(true);
    assert!(target_airborne(&w, v), "airborne from the refusal");
    assert!(aa_finds(&w, v), "the anti-air row treats it as airborne");
}

#[test]
fn claim17_p11_run_i_parts_a_b_a_failed_check_and_its_redirect_count_by_height() {
    // spec: S-0306 claim 17 (after a refused placement it counts by its height after a failed check and a redirect, until a later check passes for a deck cell) and scenario P11 run i parts a and b; in our engine a failed check always redirects (S-0513 claim 37 and T2), so parts a and b are one update
    let (mut w, v, b) = refused(true);
    remove(&mut w, b);
    // Force the failure: a building now stands on the deck of the landing
    // cell (S-0513 claim 37: an unprotected building blocks the landing).
    let k = spawn(&mut w, "AUXB", BETA, 5, 5);
    w.cells.leave_both(at(5, 5), k);
    w.cells.enter(at(5, 5), Layer::Deck, k, true);
    assert!(warp::turn(&mut w, v));
    let m = w.obj(v).unwrap().sw.warp.clone().unwrap();
    assert_eq!(m.stage, Stage::Blocked, "the check failed");
    assert!(m.redirected, "and redirected V");
    // Until a later check passes, it counts by its height, wherever the
    // redirect put its landing point.
    assert!(target_airborne(&w, v));
    assert!(aa_finds(&w, v));
}

#[test]
fn claim17_p11_run_i_part_c_a_pass_for_a_cell_not_on_a_deck_counts_by_height() {
    // spec: S-0306 claim 17 (each passed check sets the count from its own landing cell: a pass for a cell not on a deck leaves it counted by its height) and scenario P11 run i part c
    let (mut w, v, b) = refused(true);
    remove(&mut w, b);
    w.obj_mut(v).unwrap().sw.warp.as_mut().unwrap().landing = cell_pos(at(6, 2), 0);
    assert!(warp::turn(&mut w, v));
    assert_eq!(stage(&w, v), Some(Stage::Away), "passed");
    assert!(target_airborne(&w, v));
    assert!(aa_finds(&w, v));
}

#[test]
fn claim17_p11_run_i_part_d_a_later_deck_pass_stands_it_on_a_deck_again() {
    // spec: S-0306 claim 17 (a later check that passes for a deck landing cell stands the member on a deck again, until that check's placement) and scenario P11 run i part d
    let (mut w, v, b) = refused(true);
    remove(&mut w, b);
    assert!(warp::turn(&mut w, v));
    assert_eq!(stage(&w, v), Some(Stage::Away), "passed for the deck cell");
    assert!(!target_airborne(&w, v), "on a deck from the forced check");
    assert!(!aa_finds(&w, v));
    assert!(warp::turn(&mut w, v), "placement");
    assert_eq!(stage(&w, v), Some(Stage::Placed));
    assert_eq!(w.obj(v).unwrap().layer, Layer::Deck, "placed on the deck");
}

#[test]
fn quirk_warp_deck_departure_airborne_off_stands_on_its_departure_deck_through_a_refusal() {
    // spec: S-0306 claim 17 and Edge cases (quirks.warp_deck_departure_airborne off: the member stands on the deck it departed from until it appears, through any refusal)
    let (mut w, v, b) = refused(false);
    assert!(!target_airborne(&w, v), "after the refusal");
    assert!(!aa_finds(&w, v));
    remove(&mut w, b);
    w.obj_mut(v).unwrap().sw.warp.as_mut().unwrap().landing = Pos::new(128, 128, 0);
    assert!(warp::turn(&mut w, v));
    assert!(
        !target_airborne(&w, v),
        "after a failed check and its redirect"
    );
    assert!(!aa_finds(&w, v));
}

// --- Revision 65: the ground render layer after the arrival hold -------------------

/// The ruleset with LANDER, a twin of the strike aircraft JET whose type can
/// land (JET's cannot).
fn rules_r65() -> Ruleset {
    let mut r = rules();
    let mut t = r.sim.aircraft["JET"].clone();
    t.econ.landable = Some(true);
    r.sim.aircraft.insert("LANDER".into(), t);
    register(&mut r, "aircraft", "LANDER");
    r
}

/// An aircraft of type `name` of BETA standing at (2, 2), a warp member in
/// its arrival hold, whose update then ends the hold (S-0513 claim 40,
/// with no held delay: the tick after the arrival effects).
fn after_hold(name: &str) -> (World, ObjectId) {
    let mut w = ready(&rules_r65());
    let a = testkit::spawn(&mut w, Kind::Aircraft, name, BETA.0, at(2, 2), 0);
    assert_eq!(w.height_above_surface(a), 0, "{name}: on the ground");
    expire(&mut w, a, at(2, 2));
    let t = w.tick;
    w.obj_mut(a).unwrap().sw.warp.as_mut().unwrap().stage = Stage::Hold(t - 1);
    assert!(warp::in_ground_layer(&w, a), "{name}: in the hold");
    assert!(!warp::turn(&mut w, a));
    assert!(!warp::is_member(&w, a), "{name}: the hold has ended");
    (w, a)
}

/// Raises the aircraft 600 leptons, as its own movement would in flight.
fn up(w: &mut World, a: ObjectId) {
    let p = pos(w, a);
    w.obj_mut(a).unwrap().pos = Pos::new(p.x, p.y, p.z + 600);
}

/// Whether a straight ranged shot passing by the airborne target, last 70
/// short of its reference point and then 80 past it, detonates at its own
/// position (claim 20 (c)'s threshold 42) rather than being moved onto the
/// target point (threshold 128), as in the revision 62 test above.
fn threshold_42(w: &World, a: ObjectId) -> bool {
    let mut w = w.clone();
    assert!(projectile::target_airborne(&w, a), "airborne");
    let f = spawn(&mut w, "BUGGY", ALPHA, -6, 2);
    let rp = w.reference_point(a);
    w.cells.leave_both(rp.cell(), a);
    let id = shoot(
        &mut w,
        f,
        "Passer",
        Target::Object(a),
        Pos::new(rp.x - 370, rp.y, rp.z),
        [150, 0, 0],
    );
    w.projectiles.get_mut(&id).unwrap().stored = rp;
    let Out::Detonated(p) = fly(&mut w, id) else {
        panic!("no detonation")
    };
    p != rp
}

/// Whether a temporal detonation from TIMER on the target locks it
/// (S-0514 claim 1: only an object of the ground render layer).
fn erasable(w: &World, a: ObjectId) -> bool {
    let mut w = w.clone();
    let f = spawn(&mut w, "TIMER", ALPHA, -2, 2);
    temporal::deliver(&mut w, Some(f), Some(Target::Object(a)));
    temporal::erased(&w, a)
}

#[test]
fn r65_claim17_an_aircraft_member_whose_type_cannot_land_keeps_the_ground_layer_in_flight() {
    // spec: S-0306 r65 claim 17 (after its arrival hold a member leaves the ground render layer only when its own movement raises it, except an aircraft whose type cannot land, which keeps the ground layer while it flies, until a crash), claim 20 (c) (threshold 42 for a target airborne in the ground render layer) and S-0514 claim 1 (an object target of the ground render layer)
    let (mut w, a) = after_hold("JET");
    up(&mut w, a);
    assert!(!temporal::ground_layer(&w, a), "its own rows: in the air");
    assert!(warp::in_ground_layer(&w, a), "kept in the ground layer");
    assert!(threshold_42(&w, a), "claim 20 (c): threshold 42");
    assert!(erasable(&w, a), "S-0514 claim 1: erasable");
}

#[test]
fn r65_claim17_an_aircraft_member_whose_type_can_land_leaves_the_ground_layer_when_raised() {
    // spec: S-0306 r65 claim 17 (after its hold the member stands at no height above its surface, so its own rows give the ground layer; it leaves that layer when its own movement raises it) and claim 20 (c) (threshold 128 for a target airborne in another render layer)
    let (mut w, a) = after_hold("LANDER");
    assert!(temporal::ground_layer(&w, a), "at no height: ground layer");
    assert!(
        !warp::in_ground_layer(&w, a),
        "no warp layer after the hold"
    );
    up(&mut w, a);
    assert!(!warp::in_ground_layer(&w, a));
    assert!(!temporal::ground_layer(&w, a), "raised: another layer");
    assert!(!threshold_42(&w, a), "claim 20 (c): threshold 128");
    assert!(!erasable(&w, a), "S-0514 claim 1: not erasable in the air");
}

#[test]
fn r65_claim17_a_crash_ends_the_kept_ground_layer() {
    // spec: S-0306 r65 claim 17 (an aircraft member whose type cannot land keeps the ground layer while it flies, until a crash) and S-0102 claim 4 (every tick of an aircraft's crash fall re-files)
    let (mut w, a) = after_hold("JET");
    up(&mut w, a);
    assert!(warp::in_ground_layer(&w, a));
    assert!(glasnost_sim::air::start_crash(&mut w, a, true));
    step(&mut w);
    assert!(w.obj(a).is_some(), "still falling");
    assert!(!warp::in_ground_layer(&w, a), "re-filed by its crash fall");
}

#[test]
fn r65_claim17_a_ground_member_is_in_the_ground_layer_by_its_own_rows_after_the_hold() {
    // spec: S-0306 r65 claim 17 (from the launch until its own movement applies again after its hold a member is in the ground render layer; a member that does not move between bridge decks then stands at no height above its surface, so its own rows give it)
    let mut w = ready(&rules_r65());
    let v = spawn(&mut w, "BUGGY", BETA, 2, 2);
    expire(&mut w, v, at(2, 2));
    let t = w.tick;
    w.obj_mut(v).unwrap().sw.warp.as_mut().unwrap().stage = Stage::Hold(t - 1);
    assert!(!warp::turn(&mut w, v));
    assert!(
        !warp::in_ground_layer(&w, v),
        "no warp layer after the hold"
    );
    assert!(temporal::ground_layer(&w, v), "its own rows");
    assert!(erasable(&w, v));
}
