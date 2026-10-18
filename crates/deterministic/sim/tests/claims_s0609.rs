//! Claim tests for gattling spin-up (S-0609) on synthetic data: invented
//! types and weapons, every number our own. The test vehicle follows the
//! spec's measurement setup (stage count 3, spin-up amount 7, spin-down
//! amount 1, normal thresholds 10, 30 and 60), which are the spec's own
//! invented numbers, not game data. The ledger is
//! `docs/m7-claims/S-0609.md`.

mod temporal_common;

use glasnost_num::{F64, Rounding};
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{BuildingType, Decimal, UnitType, WeaponType};
use glasnost_sim::combat::check::Answer;
use glasnost_sim::combat::data::WeaponId;
use glasnost_sim::combat::effects::Effect;
use glasnost_sim::combat::weapon;
use glasnost_sim::gattling::{self, AttackRow, Spin, SpinEvent, SpinType};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::world::{HouseId, ObjectId, Target, World};
use glasnost_sim::{save, testkit, transport};

const SIZE: i32 = 30;
const ALPHA: u8 = 0;
const BETA: u8 = 1;
/// The spec's invented thresholds (G1).
const NORMAL: [i32; 3] = [10, 30, 60];
/// The spec's invented elite thresholds (G7 a).
const ELITE: [i32; 3] = [5, 15, 30];
/// The facing towards the east (S-0204's facing points).
const EAST: u16 = 16384;

fn dec(s: &str) -> Decimal {
    Decimal(s.into())
}

fn register(r: &mut Ruleset, reg: &str, name: &str) {
    let l = r.registries.entry(reg.into()).or_default();
    if !l.iter().any(|n| n == name) {
        l.push(name.into());
    }
}

fn gun(rof: i32) -> WeaponType {
    WeaponType {
        damage: Some(1),
        rof: Some(rof),
        range: Some(dec("10")),
        speed: Some(100),
        projectile: Some("Bullet".into()),
        warhead: Some("SA".into()),
        ..Default::default()
    }
}

/// The six slot names: W0 … W5, and the elite ones X0 … X5.
fn slots(prefix: &str) -> Vec<String> {
    (0..6).map(|i| format!("{prefix}{i}")).collect()
}

/// Gives a type's combat fields the gattling flag, six slots and the
/// spec's spin settings (u = 7, d = 1, thresholds 10, 30, 60).
fn spin_fields(c: &mut glasnost_rules::sim::CombatFields) {
    c.weapon_slots = Some(slots("W"));
    c.is_gattling = Some(true);
    c.weapon_stages = Some(3);
    c.rate_up = Some(7);
    c.rate_down = Some(1);
    c.stage_thresholds = Some(NORMAL.to_vec());
}

/// The rules:
/// - weapons W0 … W5 and X0 … X5 (damage 1, rate 1, range 10);
/// - GAT, a turreted gattling vehicle with turret count 1, six slots and
///   the spin settings above; GATX, the same with the elite thresholds
///   5, 15, 30 and elite slots X0 … X5; GATE, GAT with slot 0 empty;
/// - TGT, a stationary target vehicle of strength 30000;
/// - GTOWER, an unturreted gattling building with the same slots and
///   settings; GSAM, the same with the SAM flag; GHUT, the same unarmed;
/// - GGRUNT, gattling infantry; GPLANE, a gattling aircraft;
/// - BUS, a transport for 2.
fn rules() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    for p in ["W", "X"] {
        for n in slots(p) {
            r.sim.weapons.insert(n, gun(1));
        }
    }
    let base = |r: &Ruleset| {
        let mut t = r.sim.vehicles["BUGGY"].clone();
        t.strength = Some(30000);
        t.primary = None;
        t
    };
    let mut gat = base(&r);
    gat.turret_count = Some(1);
    gat.weapon_count = Some(6);
    spin_fields(&mut gat.combat);
    let mut gatx = gat.clone();
    gatx.combat.elite_stage_thresholds = Some(ELITE.to_vec());
    gatx.combat.elite_weapon_slots = Some(slots("X"));
    let mut gate = gat.clone();
    let mut s0 = slots("W");
    s0[0].clear();
    gate.combat.weapon_slots = Some(s0);
    let tgt = base(&r);
    let mut bus = base(&r);
    bus.passengers = Some(2);
    bus.size_limit = Some(dec("2"));
    for (n, t) in [
        ("GAT", gat),
        ("GATX", gatx),
        ("GATE", gate),
        ("TGT", tgt),
        ("BUS", bus),
    ] {
        register(&mut r, "vehicles", n);
        r.sim.vehicles.insert(n.into(), t);
    }
    let tower = |sam: bool, armed: bool| {
        let mut b = BuildingType {
            strength: Some(30000),
            foundation: Some([1, 1]),
            turret_count: Some(1),
            sam: Some(sam),
            ..Default::default()
        };
        spin_fields(&mut b.combat);
        if !armed {
            b.combat.weapon_slots = None;
        }
        b
    };
    for (n, t) in [
        ("GTOWER", tower(false, true)),
        ("GSAM", tower(true, true)),
        ("GHUT", tower(false, false)),
    ] {
        register(&mut r, "buildings", n);
        r.sim.buildings.insert(n.into(), t);
    }
    let mut grunt = r.sim.infantry["WALKER"].clone();
    grunt.strength = Some(30000);
    grunt.turret_count = Some(1);
    grunt.weapon_count = Some(6);
    spin_fields(&mut grunt.combat);
    register(&mut r, "infantry", "GGRUNT");
    r.sim.infantry.insert("GGRUNT".into(), grunt);
    let mut plane = UnitType {
        strength: Some(30000),
        turret_count: Some(1),
        weapon_count: Some(6),
        ..Default::default()
    };
    spin_fields(&mut plane.combat);
    register(&mut r, "aircraft", "GPLANE");
    r.sim.aircraft.insert("GPLANE".into(), plane);
    r
}

fn game(r: &Ruleset) -> World {
    testkit::game_with(r, SIZE, 3)
}

fn put(w: &mut World, kind: Kind, ty: &str, house: u8, dx: i32, dy: i32) -> ObjectId {
    testkit::spawn(w, kind, ty, house, testkit::mid(SIZE, dx, dy), 0)
}

/// Puts `a` in *attack* on `t`.
fn attack(w: &mut World, a: ObjectId, t: ObjectId) {
    let tick = w.tick;
    let stale = w.rules().quirks.stale_suspended_revert;
    let o = w.obj_mut(a).unwrap();
    o.target = Some(Target::Object(t));
    o.beh.force(Behaviour::Attack, tick, stale);
}

/// A world with an Alpha `ty` in the middle, facing east, attacking a
/// Beta TGT three cells east.
fn duel(r: &Ruleset, kind: Kind, ty: &str) -> (World, ObjectId, ObjectId) {
    let mut w = game(r);
    let a = testkit::spawn(&mut w, kind, ty, ALPHA, testkit::mid(SIZE, 0, 0), EAST);
    let t = put(&mut w, Kind::Vehicle, "TGT", BETA, 3, 0);
    attack(&mut w, a, t);
    (w, a, t)
}

fn set(w: &mut World, id: ObjectId, value: i32, stage: i32) {
    let o = w.obj_mut(id).unwrap();
    o.combat.spin = value;
    o.combat.gattling_stage = stage;
}

fn vs(w: &World, id: ObjectId) -> (i32, i32) {
    let s = gattling::spin(w, id);
    (s.value, s.stage)
}

fn experience(w: &mut World, id: ObjectId, e: i64) {
    let x = F64::from_i64(e, Rounding::TowardZero).to_f32(Rounding::TowardZero);
    w.obj_mut(id).unwrap().combat.experience = x;
}

fn weapon_named(w: &World, name: &str) -> WeaponId {
    let r = w.rules();
    (0..r.combat.weapons.len() as u32)
        .find(|&i| r.combat.weapon(i).name == name)
        .unwrap_or_else(|| panic!("no weapon {name}"))
}

/// Runs one tick with the effects list emptied first, so that the effects
/// read afterwards are that tick's (the list collects until drained).
fn tick(w: &mut World) {
    w.effects.clear();
    testkit::run(w, 1);
}

/// The spin effects of `id` in the effects of the last tick or call.
fn spin_events(w: &World, id: ObjectId) -> Vec<SpinEvent> {
    w.effects
        .iter()
        .filter_map(|e| match e {
            Effect::Spin { id: x, event, .. } if *x == id => Some(*event),
            _ => None,
        })
        .collect()
}

/// The weapons `id` fired, by the effects of the last tick.
fn shots(w: &World, id: ObjectId) -> Vec<WeaponId> {
    w.effects
        .iter()
        .filter_map(|e| match e {
            Effect::Shot { firer, weapon, .. } if *firer == id => Some(*weapon),
            _ => None,
        })
        .collect()
}

fn st(normal: [i32; 3], up: i32, down: i32) -> (SpinType, [i32; 6]) {
    let mut thr = [0; 6];
    thr[..3].copy_from_slice(&normal);
    (
        SpinType {
            stages: 3,
            up,
            down,
            normal: thr,
            elite: [0; 6],
        },
        thr,
    )
}

// ---------------------------------------------------------------------------
// Formulas F1, F2 and F3.

// spec: S-0609 F1 and F3 (u = 7, thresholds 10, 30, 60: the stage reaches 1 at update 3 and 2 at update 6, and the value stops at v_max = 63)
#[test]
fn f1_g1_rises_at_updates_three_and_six() {
    let (t, thr) = st(NORMAL, 7, 1);
    let (mut v, mut s) = (0, 0);
    let mut rises = Vec::new();
    for j in 1..=12 {
        let (v2, s2, raised) = gattling::f1(&t, &thr, v, s, 1);
        assert_eq!(raised, s2 == s + 1);
        if raised {
            rises.push(j);
        }
        (v, s) = (v2, s2);
    }
    assert_eq!(rises, vec![3, 6]);
    assert_eq!((v, s), (63, 2), "no clamp: the value overshoots the cap 60");
}

// spec: S-0609 claim 2 (the rise test reads the value before the update: a stage rises one update after the value reached its threshold)
#[test]
fn f1_rise_reads_the_value_before_the_update() {
    let (t, thr) = st(NORMAL, 7, 1);
    // 7 → 14 crosses 10, but the stage tests 7.
    assert_eq!(gattling::f1(&t, &thr, 7, 0, 1), (14, 0, false));
    assert_eq!(gattling::f1(&t, &thr, 14, 0, 1), (21, 1, true));
    // At most one stage per update, whatever the value.
    assert_eq!(gattling::f1(&t, &thr, 59, 0, 1), (66, 1, true));
    // The multiplier scales the amount.
    assert_eq!(gattling::f1(&t, &thr, 0, 0, 3), (21, 0, false));
    // At the cap the value is unchanged, the stage still rises.
    assert_eq!(gattling::f1(&t, &thr, 60, 1, 1), (60, 2, true));
    assert_eq!(gattling::f1(&t, &thr, 63, 2, 1), (63, 2, false));
}

// spec: S-0609 claim 3, F2 and G2 (from v = 63 at stage 2 the stage falls after 34 spin-downs and again after 54)
#[test]
fn f2_g2_falls_after_34_and_54() {
    let (t, thr) = st(NORMAL, 7, 1);
    let (mut v, mut s) = (63, 2);
    let mut falls = Vec::new();
    for k in 1..=70 {
        let (v2, s2) = gattling::f2(&t, &thr, v, s, 1, true);
        if s2 < s {
            assert_eq!(s2, s - 1, "at most one stage per update");
            falls.push(k);
        }
        (v, s) = (v2, s2);
    }
    assert_eq!(falls, vec![34, 54]);
    assert_eq!((v, s), (0, 0), "never below 0");
}

// spec: S-0609 claim 3 (a stage falls by at most one per update even when the value drops several bands, and falls in the update that takes the value below T_s)
#[test]
fn f2_one_stage_per_update() {
    let (t, thr) = st(NORMAL, 7, 1);
    assert_eq!(gattling::f2(&t, &thr, 63, 2, 60, true), (3, 1));
    assert_eq!(gattling::f2(&t, &thr, 3, 1, 1, true), (2, 0));
    assert_eq!(gattling::f2(&t, &thr, 30, 2, 1, true), (29, 1));
}

// spec: S-0609 claim 3, Edge cases and G6 (quirks.gattling_zero_spin_down_empties on: d = 0 empties the value at the first spin-down, then one stage per spin-down; off: nothing changes)
#[test]
fn quirk_zero_spin_down_empties_both_positions() {
    let (t, thr) = st(NORMAL, 7, 0);
    assert_eq!(gattling::f2(&t, &thr, 63, 2, 1, true), (0, 1));
    assert_eq!(gattling::f2(&t, &thr, 0, 1, 1, true), (0, 0));
    assert_eq!(gattling::f2(&t, &thr, 63, 2, 1, false), (63, 2));
    // Off: the stage falls only when the value is already below T_s.
    assert_eq!(gattling::f2(&t, &thr, 20, 2, 1, false), (20, 1));
    // A multiplier of 0 (a building with c = 0) is an amount of 0 too.
    let (t1, thr1) = st(NORMAL, 7, 1);
    assert_eq!(gattling::f2(&t1, &thr1, 40, 2, 0, true), (0, 1));
    assert_eq!(gattling::f2(&t1, &thr1, 40, 2, 0, false), (40, 2));
    // Through the world, by the switch in the rules.
    for (on, want) in [(true, (0, 1)), (false, (63, 2))] {
        let mut r = rules();
        r.sim.vehicles.get_mut("GAT").unwrap().combat.rate_down = Some(0);
        r.globals.quirks.gattling_zero_spin_down_empties = on;
        let mut w = game(&r);
        let a = put(&mut w, Kind::Vehicle, "GAT", ALPHA, 0, 0);
        set(&mut w, a, 63, 2);
        gattling::spin_down(&mut w, a, 1, true);
        assert_eq!(vs(&w, a), want, "switch {on}");
    }
}

// spec: S-0609 Edge cases (thresholds out of order are applied as written)
#[test]
fn thresholds_out_of_order_apply_as_written() {
    let (t, thr) = st([30, 10, 60], 7, 1);
    // From 14: the rise to stage 1 needs 30; at stage 1 the rise to 2 needs 10.
    assert_eq!(gattling::f1(&t, &thr, 14, 0, 1).1, 0);
    assert_eq!(gattling::f1(&t, &thr, 14, 1, 1).1, 2);
    // At stage 2 with value 21 < T_2 = 10? no: 20 ≥ 10, so no fall; at stage 1, 20 < 30 falls.
    assert_eq!(gattling::f2(&t, &thr, 21, 1, 1, true), (20, 0));
    assert_eq!(gattling::f2(&t, &thr, 21, 2, 1, true), (20, 2));
}

// spec: S-0609 Import mapping and Edge cases (a stage count above 6 is used as 6; stage count 0 or 1: the thresholds stay 0, the value never grows; negative settings are used as 0)
#[test]
fn spin_type_from_fields() {
    let mut c = glasnost_rules::sim::CombatFields::default();
    assert_eq!(SpinType::from_fields(&c), SpinType::default());
    c.weapon_stages = Some(9);
    c.rate_up = Some(-3);
    c.rate_down = Some(2);
    c.stage_thresholds = Some(vec![1, 2, 3, 4, 5, 6, 7, 8, 9]);
    c.elite_stage_thresholds = Some(vec![-4, 5]);
    let s = SpinType::from_fields(&c);
    assert_eq!(s.stages, 6);
    assert_eq!(s.up, 0);
    assert_eq!(s.down, 2);
    assert_eq!(s.normal, [1, 2, 3, 4, 5, 6]);
    assert_eq!(s.elite, [0, 5, 0, 0, 0, 0]);
    c.weapon_stages = Some(1);
    let s = SpinType::from_fields(&c);
    assert_eq!((s.stages, s.normal, s.elite), (1, [0; 6], [0; 6]));
    // Stage count 1: the cap is 0, so the value never grows and the stage stays 0.
    assert_eq!(gattling::f1(&s, &s.normal, 0, 0, 1), (0, 0, false));
    c.weapon_stages = Some(-2);
    assert_eq!(SpinType::from_fields(&c).stages, 0);
}

// spec: S-0609 Edge cases (revision 5, answering Q-1151: a stage count lowered below 2 by a later layer, after an earlier layer populated the thresholds: the final count governs, the thresholds in force are all 0 even though they are held as imported, the spin cap is 0, the value never grows and the stage stays 0) and Import mapping
#[test]
fn layered_reduction_below_2_uses_no_thresholds() {
    // As the importer writes such a type: the held rows of the earlier layer, the final count 1.
    let c = glasnost_rules::sim::CombatFields {
        is_gattling: Some(true),
        weapon_stages: Some(1),
        rate_up: Some(7),
        rate_down: Some(1),
        stage_thresholds: Some(vec![10, 30, 60, 0, 0, 0]),
        elite_stage_thresholds: Some(vec![5, 15, 30, 0, 0, 0]),
        ..Default::default()
    };
    let s = SpinType::from_fields(&c);
    assert_eq!((s.stages, s.normal, s.elite), (1, [0; 6], [0; 6]));
    let (mut v, mut stage) = (0, 0);
    for _ in 0..20 {
        let (v2, s2, raised) = gattling::f1(&s, &s.normal, v, stage, 1);
        assert!(!raised);
        (v, stage) = (v2, s2);
    }
    assert_eq!(
        (v, stage),
        (0, 0),
        "the value never grows and the stage stays 0"
    );
    // The same rows with a final count of 3 are used as imported.
    let s3 = SpinType::from_fields(&glasnost_rules::sim::CombatFields {
        weapon_stages: Some(3),
        ..c
    });
    assert_eq!(s3.normal, [10, 30, 60, 0, 0, 0]);
}

// spec: S-0609 Edge cases and Constants (engine rule, answering Q-1220: every spin product and sum and the elapsed-tick count saturate at 2³¹ − 1, nothing wraps; a saturated value is left unchanged by F1)
#[test]
fn saturation_at_the_32_bit_bound() {
    let (t, thr) = st([10, 30, i32::MAX], i32::MAX, i32::MAX);
    // u × m and v + u × m saturate.
    assert_eq!(gattling::f1(&t, &thr, 5, 0, 3).0, i32::MAX);
    // A saturated value is at the cap: F1 leaves it.
    assert_eq!(gattling::f1(&t, &thr, i32::MAX, 2, 3).0, i32::MAX);
    // d × m saturates, and the value falls to 0, not below.
    assert_eq!(gattling::f2(&t, &thr, 100, 0, 5, true).0, 0);
    // The elapsed-tick count stays at the bound.
    let r = rules();
    let mut w = game(&r);
    let a = put(&mut w, Kind::Vehicle, "GAT", ALPHA, 0, 0);
    w.obj_mut(a).unwrap().beh.elapsed = i32::MAX - 1;
    gattling::elapse(&mut w, a);
    gattling::elapse(&mut w, a);
    assert_eq!(w.obj(a).unwrap().beh.elapsed, i32::MAX);
}

// ---------------------------------------------------------------------------
// Claim 1 and 4: state, thresholds in force.

// spec: S-0609 claim 1 (created with value 0 and stage 0; a change of owner or of rank leaves both unchanged)
#[test]
fn claim_1_initial_values_and_persistence() {
    let r = rules();
    let mut w = game(&r);
    let a = put(&mut w, Kind::Vehicle, "GAT", ALPHA, 0, 0);
    assert_eq!(gattling::spin(&w, a), Spin::default());
    assert_eq!(w.obj(a).unwrap().combat.anim_frame, 0);
    set(&mut w, a, 40, 2);
    glasnost_sim::mind::set_owner(&mut w, a, Some(HouseId(BETA)));
    assert_eq!(vs(&w, a), (40, 2), "owner change");
    experience(&mut w, a, 2);
    assert_eq!(vs(&w, a), (40, 2), "rank change");
}

// spec: S-0609 claim 4 and G7 a (elite thresholds 5, 15, 30: rises at updates 2 and 4)
#[test]
fn claim_4_g7a_elite_thresholds() {
    let r = rules();
    let mut w = game(&r);
    let a = put(&mut w, Kind::Vehicle, "GATX", ALPHA, 0, 0);
    experience(&mut w, a, 2);
    let mut rises = Vec::new();
    for j in 1..=8 {
        let before = vs(&w, a).1;
        gattling::spin_up(&mut w, a, 1);
        if vs(&w, a).1 > before {
            rises.push(j);
        }
    }
    assert_eq!(rises, vec![2, 4]);
    // v_max: the first multiple of 7 at least 30.
    assert_eq!(vs(&w, a), (35, 2));
}

// spec: S-0609 claim 4, Edge cases and G7 b (an elite object without elite thresholds: cap 0, a rise at every spin-up, never a fall)
#[test]
fn claim_4_g7b_elite_without_elite_thresholds() {
    let r = rules();
    let mut w = game(&r);
    let a = put(&mut w, Kind::Vehicle, "GAT", ALPHA, 0, 0);
    experience(&mut w, a, 2);
    gattling::spin_up(&mut w, a, 1);
    assert_eq!(vs(&w, a), (0, 1));
    gattling::spin_up(&mut w, a, 1);
    assert_eq!(vs(&w, a), (0, 2));
    for _ in 0..10 {
        gattling::spin_down(&mut w, a, 1, true);
    }
    assert_eq!(vs(&w, a), (0, 2));
}

// spec: S-0609 claim 4 and G7 c (becoming elite keeps value and stage; later updates use the elite row and move the stage by at most one)
#[test]
fn claim_4_g7c_rank_change_part_way() {
    let r = rules();
    let mut w = game(&r);
    let a = put(&mut w, Kind::Vehicle, "GATX", ALPHA, 0, 0);
    for _ in 0..3 {
        gattling::spin_up(&mut w, a, 1);
    }
    assert_eq!(vs(&w, a), (21, 1));
    experience(&mut w, a, 2);
    assert_eq!(vs(&w, a), (21, 1), "the rank change itself");
    // Elite cap 30: 21 < 30 grows to 28; 21 ≥ E_2 = 15 raises to 2.
    gattling::spin_up(&mut w, a, 1);
    assert_eq!(vs(&w, a), (28, 2));
    gattling::spin_up(&mut w, a, 1);
    gattling::spin_up(&mut w, a, 1);
    assert_eq!(vs(&w, a), (35, 2), "the elite cap stops growth");
}

// ---------------------------------------------------------------------------
// Claim 5: vehicles.

// spec: S-0609 claim 5, G1 and F3 (one spin-up per tick of the firing step: stage 1 after update 3, stage 2 after update 6, value 63; each shot uses slot 2s of the stage before that tick's update)
#[test]
fn claim_5_g1_through_ticks() {
    let r = rules();
    let (mut w, a, _) = duel(&r, Kind::Vehicle, "GAT");
    let expect = [
        (7, 0),
        (14, 0),
        (21, 1),
        (28, 1),
        (35, 1),
        (42, 2),
        (49, 2),
        (56, 2),
        (63, 2),
        (63, 2),
        (63, 2),
    ];
    let mut fired = Vec::new();
    for (k, want) in expect.iter().enumerate() {
        let before = vs(&w, a).1;
        tick(&mut w);
        assert_eq!(vs(&w, a), *want, "tick {}", k + 1);
        for wpn in shots(&w, a) {
            let slot = (before * 2) as usize;
            assert_eq!(
                Some(wpn),
                weapon::slot_weapon(&w, a, slot),
                "tick {}",
                k + 1
            );
            fired.push(wpn);
        }
    }
    assert!(fired.contains(&weapon_named(&w, "W0")), "a stage-0 shot");
    assert!(fired.contains(&weapon_named(&w, "W4")), "a stage-2 shot");
}

/// Every firing-check answer and its row of claim 5's table.
const ANSWERS: [(Answer, bool); 12] = [
    (Answer::CanFire, true),
    (Answer::Rearming, true),
    (Answer::NotFacing, true),
    (Answer::StillTurning, true),
    (Answer::OutOfAmmo, false),
    (Answer::Illegal, false),
    (Answer::Impossible, false),
    (Answer::Moving, false),
    (Answer::OutOfRange, false),
    (Answer::Cloaked, false),
    (Answer::Busy, false),
    (Answer::MustDeploy, false),
];

// spec: S-0609 claim 5 (can fire, rearming, not facing, still turning: spin-up; any other answer, no target or an empty slot 0: spin-down; the deploy-to-fire exit: no update)
#[test]
fn claim_5_answer_table() {
    let r = rules();
    let mut w = game(&r);
    let a = put(&mut w, Kind::Vehicle, "GAT", ALPHA, 0, 0);
    for (answer, up) in ANSWERS {
        set(&mut w, a, 20, 1);
        gattling::vehicle_step(&mut w, a, Some(answer), false);
        let want = if up { (27, 1) } else { (19, 1) };
        assert_eq!(vs(&w, a), want, "{answer:?}");
    }
    set(&mut w, a, 20, 1);
    gattling::vehicle_step(&mut w, a, None, false);
    assert_eq!(vs(&w, a), (19, 1), "no target or empty slot 0");
    for (answer, _) in ANSWERS {
        set(&mut w, a, 20, 1);
        gattling::vehicle_step(&mut w, a, Some(answer), true);
        assert_eq!(vs(&w, a), (20, 1), "unload exit, {answer:?}");
    }
}

// spec: S-0609 claim 5 (a firing step with no target spins down by 1)
#[test]
fn claim_5_no_target_spins_down_each_tick() {
    let r = rules();
    let mut w = game(&r);
    let a = put(&mut w, Kind::Vehicle, "GAT", ALPHA, 0, 0);
    set(&mut w, a, 20, 1);
    testkit::run(&mut w, 5);
    assert_eq!(vs(&w, a), (15, 1));
}

// spec: S-0609 claim 5 (a target but an empty slot 0: spin-down)
#[test]
fn claim_5_empty_slot_zero_spins_down() {
    let r = rules();
    let (mut w, a, _) = duel(&r, Kind::Vehicle, "GATE");
    set(&mut w, a, 20, 1);
    testkit::run(&mut w, 3);
    assert_eq!(vs(&w, a), (17, 1));
}

// spec: S-0609 claim 5 and claim 12 (the rate of spin is one update per firing step: berserk and veteran rank change neither the amount nor the pace)
#[test]
fn claim_5_and_12_no_modifiers() {
    let r = rules();
    let mut seen = Vec::new();
    for case in 0..3 {
        let (mut w, a, _) = duel(&r, Kind::Vehicle, "GAT");
        match case {
            1 => experience(&mut w, a, 1),
            2 => w.obj_mut(a).unwrap().combat.berserk = Some(1000),
            _ => {}
        }
        let mut row = Vec::new();
        for _ in 0..9 {
            testkit::run(&mut w, 1);
            row.push(vs(&w, a));
        }
        seen.push(row);
    }
    assert_eq!(seen[0], seen[1], "veteran");
    assert_eq!(seen[0], seen[2], "berserk");
}

// ---------------------------------------------------------------------------
// Claim 6 and 11: freezes and resets.

// spec: S-0609 claim 11 and claim 13 (boarding resets value, stage and elapsed count; leaving the map ends the sound); claim 6 (a passenger takes no firing step: its spin is frozen)
#[test]
fn claim_11_boarding_resets_and_passengers_freeze() {
    let r = rules();
    let mut w = game(&r);
    let a = put(&mut w, Kind::Vehicle, "GAT", ALPHA, 0, 0);
    let bus = put(&mut w, Kind::Vehicle, "BUS", ALPHA, 1, 0);
    set(&mut w, a, 63, 2);
    w.obj_mut(a).unwrap().beh.elapsed = 9;
    w.effects.clear();
    transport::board(&mut w, a, bus);
    assert_eq!(gattling::spin(&w, a), Spin::default());
    assert_eq!(spin_events(&w, a), vec![SpinEvent::Left]);
    // Inside, nothing changes the spin (set by hand to see it).
    set(&mut w, a, 30, 1);
    testkit::run(&mut w, 5);
    assert_eq!(vs(&w, a), (30, 1));
}

/// The temporal fixture with a gattling target vehicle GVICT that attacks
/// a VICT of its own: stage count 3, u = 7, d = 1, thresholds 100, 200,
/// 300 (so the value grows by 7 every firing step).
fn temporal_world() -> (World, ObjectId, ObjectId) {
    use temporal_common as tc;
    let mut r = tc::ruleset();
    r.sim.weapons.insert("W0".into(), gun(1));
    tc::vehicle(&mut r, "GVICT", |t| {
        t.primary = Some("W0".into());
        t.combat.is_gattling = Some(true);
        t.combat.weapon_stages = Some(3);
        t.combat.rate_up = Some(7);
        t.combat.rate_down = Some(1);
        t.combat.stage_thresholds = Some(vec![100, 200, 300]);
    });
    let mut w = tc::game(&r);
    let g = tc::spawn(&mut w, Kind::Vehicle, "GVICT", 1, tc::at(0, 0));
    let prey = tc::spawn(&mut w, Kind::Vehicle, "VICT", 0, tc::at(2, 0));
    attack(&mut w, g, prey);
    (w, g, prey)
}

// spec: S-0609 claim 11 and S-0514 claim 4 (each successful temporal lock, a joining one included, gives exactly one spin-down with multiplier 1); claim 6 (no firing step, no spin change, while being erased)
#[test]
fn claim_11_temporal_locks_spin_down_once_each() {
    use temporal_common as tc;
    let (mut w, g, _) = temporal_world();
    let e1 = tc::attacker(&mut w, "ERASER", 0, tc::at(-3, 0));
    let e2 = tc::attacker(&mut w, "ERASER2", 0, tc::at(-3, 2));
    set(&mut w, g, 150, 1);
    let mut downs = 0;
    let t0 = w.tick;
    assert!(glasnost_sim::combat::fire::shot(
        &mut w,
        e1,
        0,
        Target::Object(g)
    ));
    while tc::amount(&w, g).is_none() {
        tick(&mut w);
        downs += spin_events(&w, g)
            .iter()
            .filter(|e| **e == SpinEvent::Down)
            .count();
        assert!(w.tick < t0 + 40, "no lock");
    }
    // Only spin-ups of 7 apart from the lock's one spin-down of 1.
    let v1 = vs(&w, g).0;
    assert_eq!(downs, 1, "one spin-down at the lock");
    assert_eq!((v1 - 150 + 1) % 7, 0, "value {v1}");
    // Frozen between locks.
    for _ in 0..3 {
        tick(&mut w);
        assert_eq!(vs(&w, g).0, v1);
        assert!(spin_events(&w, g).is_empty(), "{:?}", spin_events(&w, g));
    }
    // The joiner's lock: one more spin-down.
    let t1 = w.tick;
    let held = |w: &World| glasnost_sim::temporal::holds(w, e2, g);
    assert!(glasnost_sim::combat::fire::shot(
        &mut w,
        e2,
        0,
        Target::Object(g)
    ));
    let mut downs = 0;
    while !held(&w) {
        tick(&mut w);
        downs += spin_events(&w, g).len();
        assert!(w.tick < t1 + 40, "no join");
    }
    assert_eq!(downs, 1);
    assert_eq!(vs(&w, g).0, v1 - 1);
}

// ---------------------------------------------------------------------------
// Claims 7 to 9: buildings.

// spec: S-0609 claim 7 and F4 (a building in attack spins up only at its processings, with multiplier c, the elapsed ticks since the last update: steps of u or 2u; c returns to 0)
#[test]
fn claim_7_building_cadence() {
    let r = rules();
    let (mut w, b, _) = duel(&r, Kind::Building, "GTOWER");
    let mut steps = Vec::new();
    let mut prev = vs(&w, b);
    for n in 1..=40 {
        testkit::run(&mut w, 1);
        let s = gattling::spin(&w, b);
        if s.value < 60 || prev.0 < 60 {
            // Below the cap nothing is lost: every elapsed tick is counted once.
            if prev.0 < 60 {
                assert_eq!(s.value + 7 * s.elapsed, 7 * n, "tick {n}");
            }
        }
        if s.value != prev.0 {
            assert_eq!(s.elapsed, 0, "an update returns c to 0");
            steps.push(s.value - prev.0);
        } else if s.stage != prev.1 {
            panic!("a stage change without a processing at tick {n}");
        }
        prev = (s.value, s.stage);
    }
    assert!(steps.iter().all(|d| *d == 7 || *d == 14), "{steps:?}");
    assert!(steps.contains(&7), "c = 1 after can fire: {steps:?}");
    assert!(steps.contains(&14), "c = 2 after rearming: {steps:?}");
    assert_eq!(prev.1, 2, "top stage");
    assert!(prev.0 >= 60);
}

// spec: S-0609 claim 7 (the table: can fire, not facing, rearming spin up with m = c; out of ammunition, illegal, impossible, out of range, cloaked, busy spin down with m = c; still turning, moving, must deploy pause; every row returns c to 0)
#[test]
fn claim_7_attack_rows() {
    let r = rules();
    let mut w = game(&r);
    let b = put(&mut w, Kind::Building, "GTOWER", ALPHA, 0, 0);
    let rows = [
        (Answer::CanFire, AttackRow::Up),
        (Answer::NotFacing, AttackRow::Up),
        (Answer::Rearming, AttackRow::Up),
        (Answer::StillTurning, AttackRow::Pause),
        (Answer::Moving, AttackRow::Pause),
        (Answer::MustDeploy, AttackRow::Pause),
        (Answer::OutOfAmmo, AttackRow::Down),
        (Answer::Illegal, AttackRow::Down),
        (Answer::Impossible, AttackRow::Down),
        (Answer::OutOfRange, AttackRow::Down),
        (Answer::Cloaked, AttackRow::Down),
        (Answer::Busy, AttackRow::Down),
    ];
    for (answer, row) in rows {
        assert_eq!(gattling::attack_row(answer), row, "{answer:?}");
        set(&mut w, b, 20, 1);
        w.obj_mut(b).unwrap().beh.elapsed = 3;
        gattling::building_attack(&mut w, b, answer, 3);
        let want = match row {
            AttackRow::Up => (41, 1),
            AttackRow::Down => (17, 1),
            AttackRow::Pause => (20, 1),
        };
        assert_eq!(vs(&w, b), want, "{answer:?}");
        assert_eq!(gattling::spin(&w, b).elapsed, 0, "{answer:?}");
    }
}

// spec: S-0609 claim 7 (a building with the SAM flag makes no spin update in attack)
#[test]
fn claim_7_sam_building_makes_no_update() {
    let r = rules();
    for (ty, changes) in [("GSAM", false), ("GTOWER", true)] {
        let (mut w, b, _) = duel(&r, Kind::Building, ty);
        set(&mut w, b, 20, 1);
        w.obj_mut(b).unwrap().beh.elapsed = 2;
        glasnost_sim::combat::behave::building_process(&mut w, b);
        assert_eq!(vs(&w, b) != (20, 1), changes, "{ty}");
    }
}

// spec: S-0609 claim 8 (guard, sticky and area guard processings start with a spin-down by c, armed or unarmed, then c returns to 0)
#[test]
fn claim_8_guard_spins_down_by_c() {
    let r = rules();
    for ty in ["GTOWER", "GHUT"] {
        let mut w = game(&r);
        let b = put(&mut w, Kind::Building, ty, ALPHA, 0, 0);
        set(&mut w, b, 20, 1);
        w.obj_mut(b).unwrap().beh.elapsed = 4;
        gattling::building_guard(&mut w, b);
        assert_eq!(
            gattling::spin(&w, b),
            Spin {
                value: 16,
                stage: 1,
                elapsed: 0
            }
        );
        assert_eq!(spin_events(&w, b), vec![SpinEvent::Down]);
        // Through ticks: no shot for a while, so no passive spin-down yet;
        // every elapsed tick is counted once by the guard processings, so
        // value − c falls by exactly one per tick.
        let mut w = game(&r);
        let b = put(&mut w, Kind::Building, ty, ALPHA, 0, 0);
        set(&mut w, b, 100, 0);
        w.obj_mut(b).unwrap().last_fired = w.tick;
        testkit::run(&mut w, 20);
        let s = gattling::spin(&w, b);
        assert!(s.value < 100, "{ty}: a guard processing took place");
        assert_eq!(s.value - s.elapsed, 100 - 20, "{ty}");
    }
}

// spec: S-0609 claim 9 (outside attack, more than G + 5 ticks after the last shot: F2 with m = 1 each tick, after the processing, no sound change)
#[test]
fn claim_9_passive_spin_down_boundary() {
    let r = rules();
    let mut w = game(&r);
    let b = put(&mut w, Kind::Building, "GHUT", ALPHA, 0, 0);
    let g = w.rules().general.guard_area_targeting_delay;
    set(&mut w, b, 20, 1);
    w.obj_mut(b).unwrap().last_fired = w.tick - (g + 5);
    w.effects.clear();
    gattling::building_turn(&mut w, b);
    assert_eq!(vs(&w, b), (20, 1), "G + 5: not yet");
    w.obj_mut(b).unwrap().last_fired = w.tick - (g + 6);
    gattling::building_turn(&mut w, b);
    assert_eq!(vs(&w, b), (19, 1), "G + 6");
    assert!(spin_events(&w, b).is_empty(), "no sound change");
    // Not in attack only.
    let t = put(&mut w, Kind::Vehicle, "TGT", BETA, 3, 0);
    attack(&mut w, b, t);
    gattling::building_turn(&mut w, b);
    assert_eq!(vs(&w, b), (19, 1), "in attack");
}

// spec: S-0609 claim 9 (through ticks: the guard processings' spin-downs by c plus one passive spin-down in each tick more than G + 5 ticks after the last shot)
#[test]
fn claim_9_passive_through_ticks() {
    let r = rules();
    let mut w = game(&r);
    let b = put(&mut w, Kind::Building, "GHUT", ALPHA, 0, 0);
    let g = w.rules().general.guard_area_targeting_delay;
    set(&mut w, b, 1000, 0);
    let t0 = w.tick;
    w.obj_mut(b).unwrap().last_fired = t0;
    let n = g + 30;
    testkit::run(&mut w, n);
    let passive = (n - (g + 6)).max(0);
    let s = gattling::spin(&w, b);
    assert_eq!(s.value - s.elapsed, 1000 - n - passive);
}

// spec: S-0609 F5 and claim 9 (T1 = 10, T2 = 30, d = 1, s = 2, v = 31: a guard processing with c = 25 gives v = 6, s = 1; the passive spin-down then gives v = 5, s = 0)
#[test]
fn claim_9_f5_two_spin_downs_in_one_tick() {
    let r = rules();
    let mut w = game(&r);
    let b = put(&mut w, Kind::Building, "GHUT", ALPHA, 0, 0);
    set(&mut w, b, 31, 2);
    w.obj_mut(b).unwrap().beh.elapsed = 25;
    w.obj_mut(b).unwrap().last_fired = w.tick - 1000;
    gattling::building_guard(&mut w, b);
    assert_eq!(vs(&w, b), (6, 1));
    gattling::building_turn(&mut w, b);
    assert_eq!(vs(&w, b), (5, 0));
}

// ---------------------------------------------------------------------------
// Claim 10: infantry and aircraft.

// spec: S-0609 claim 10 (no rule raises the spin of infantry or aircraft)
#[test]
fn claim_10_infantry_and_aircraft_do_not_spin() {
    let r = rules();
    for (kind, ty) in [(Kind::Infantry, "GGRUNT"), (Kind::Aircraft, "GPLANE")] {
        let (mut w, a, _) = duel(&r, kind, ty);
        testkit::run(&mut w, 30);
        assert_eq!(vs(&w, a), (0, 0), "{ty}");
        gattling::vehicle_step(&mut w, a, Some(Answer::CanFire), false);
        gattling::building_turn(&mut w, a);
        assert_eq!(vs(&w, a), (0, 0), "{ty}");
    }
}

// ---------------------------------------------------------------------------
// Claim 13: the spin sound requests.

// spec: S-0609 claim 13 (a spin-up reports whether it raised the stage and the weapon in slot 2s′ after elite replacement; a spin-down reports Down; the passive spin-down and the reset report nothing)
#[test]
fn claim_13_spin_effects() {
    let r = rules();
    let mut w = game(&r);
    let a = put(&mut w, Kind::Vehicle, "GATX", ALPHA, 0, 0);
    set(&mut w, a, 14, 0);
    w.effects.clear();
    gattling::spin_up(&mut w, a, 1);
    let w2 = weapon_named(&w, "W2");
    assert_eq!(
        spin_events(&w, a),
        vec![SpinEvent::Up {
            raised: true,
            weapon: Some(w2)
        }]
    );
    w.effects.clear();
    gattling::spin_up(&mut w, a, 1);
    assert_eq!(
        spin_events(&w, a),
        vec![SpinEvent::Up {
            raised: false,
            weapon: Some(w2)
        }]
    );
    experience(&mut w, a, 2);
    w.effects.clear();
    gattling::spin_up(&mut w, a, 1);
    let x4 = weapon_named(&w, "X4");
    assert_eq!(
        spin_events(&w, a),
        vec![SpinEvent::Up {
            raised: true,
            weapon: Some(x4)
        }],
        "elite replacement, stage 2"
    );
    w.effects.clear();
    gattling::spin_down(&mut w, a, 1, true);
    assert_eq!(spin_events(&w, a), vec![SpinEvent::Down]);
    w.effects.clear();
    gattling::spin_down(&mut w, a, 1, false);
    gattling::reset(&mut w, a);
    assert!(spin_events(&w, a).is_empty());
    // An empty slot 2s′: no weapon, so no report list.
    let e = put(&mut w, Kind::Vehicle, "GATE", ALPHA, 2, 2);
    w.effects.clear();
    gattling::spin_up(&mut w, e, 1);
    assert_eq!(
        spin_events(&w, e),
        vec![SpinEvent::Up {
            raised: false,
            weapon: None
        }]
    );
}

// spec: S-0609 claim 13 (a gattling-flagged firer's shot is marked so that it plays no per-shot report)
#[test]
fn claim_13_shots_are_marked() {
    let r = rules();
    let (mut w, a, _) = duel(&r, Kind::Vehicle, "GAT");
    let mut marked = 0;
    for _ in 0..10 {
        tick(&mut w);
        for e in &w.effects {
            if let Effect::Shot {
                firer, gattling, ..
            } = e
                && *firer == a
            {
                assert!(*gattling);
                marked += 1;
            }
        }
    }
    assert!(marked > 0);
}

// ---------------------------------------------------------------------------
// Claim 14: the animation counter.

// spec: S-0609 claim 14 (a vehicle advances once per completed firing step with value at least 1; the unload exit makes no advance; a building advances once per tick with positive value after its processing, also outside attack)
#[test]
fn claim_14_animation_counter() {
    let r = rules();
    let mut w = game(&r);
    let a = put(&mut w, Kind::Vehicle, "GAT", ALPHA, 0, 0);
    let frame = |w: &World, id| w.obj(id).unwrap().combat.anim_frame;
    gattling::vehicle_step(&mut w, a, Some(Answer::CanFire), false);
    assert_eq!(frame(&w, a), 1);
    gattling::vehicle_step(&mut w, a, Some(Answer::CanFire), true);
    assert_eq!(frame(&w, a), 1, "unload exit");
    set(&mut w, a, 1, 0);
    gattling::vehicle_step(&mut w, a, None, false);
    assert_eq!(frame(&w, a), 1, "value 0 after the update");
    let b = put(&mut w, Kind::Building, "GHUT", ALPHA, 3, 3);
    set(&mut w, b, 50, 0);
    w.obj_mut(b).unwrap().last_fired = w.tick - 1000;
    gattling::building_turn(&mut w, b);
    assert_eq!(frame(&w, b), 1, "one advance, passive decay adds none");
    set(&mut w, b, 0, 0);
    gattling::building_turn(&mut w, b);
    assert_eq!(frame(&w, b), 1);
}

// ---------------------------------------------------------------------------
// Save and load.

// spec: S-0609 claim 1 (spin value and stage are part of the object's state) and Terms (the elapsed-tick count): saved, loaded and hashed
#[test]
fn save_round_trip() {
    let r = rules();
    let (mut w, a, _) = duel(&r, Kind::Vehicle, "GAT");
    let b = put(&mut w, Kind::Building, "GHUT", ALPHA, -3, -3);
    let plain = save::state_hash(&w);
    testkit::run(&mut w, 4);
    set(&mut w, b, 9, 0);
    w.obj_mut(b).unwrap().beh.elapsed = 3;
    let h = save::state_hash(&w);
    assert_ne!(h, plain);
    let l = save::load(&save::save(&w), w.rules_arc(), testkit::catalogue()).unwrap();
    assert_eq!(save::state_hash(&l), h);
    assert_eq!(gattling::spin(&l, a), gattling::spin(&w, a));
    assert_eq!(gattling::spin(&l, b), gattling::spin(&w, b));
    assert_eq!(
        l.obj(a).unwrap().combat.anim_frame,
        w.obj(a).unwrap().combat.anim_frame
    );
    // The state hash sees each part.
    let mut m = l;
    m.obj_mut(b).unwrap().beh.elapsed = 4;
    assert_ne!(save::state_hash(&m), h);
}
