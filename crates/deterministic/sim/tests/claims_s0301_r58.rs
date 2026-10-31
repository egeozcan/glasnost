//! Claim tests for S-0301 revision 58 (intake 55a): a building's refill
//! (claim 10, backed by M-0004), the unchecked secondary attempt at 0
//! rounds, the hospital and armory exceptions (Import mapping), and the
//! matched fire emitter's refire (claim 5, scenario F4).
//!
//! Every test runs the real tick pipeline (`testkit::run`) on synthetic
//! data: invented weapons and an invented building defence, no stock
//! values. The building's ammunition is read at the end of every tick, as
//! M-0004 and scenario F9 read it.

use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{BuildingType, CombatFields, Decimal, WeaponType};
use glasnost_sim::combat::check::{Answer, check};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit;
use glasnost_sim::world::{ObjectId, Target, World};

/// Map size of every test here.
const N: i32 = 24;
/// Facing east (S-0204).
const EAST: u16 = 16384;
/// Facing west.
const WEST: u16 = 49152;

fn d(s: &str) -> Decimal {
    Decimal(s.into())
}

/// An invented single-shot weapon: rate `rof`, range 10 cells, the
/// invisible projectile Bullet, warhead SA.
fn zap(rof: i32) -> WeaponType {
    WeaponType {
        damage: Some(1),
        rof: Some(rof),
        range: Some(d("10")),
        speed: Some(100),
        projectile: Some("Bullet".into()),
        warhead: Some("SA".into()),
        ..Default::default()
    }
}

/// The combat ruleset with an Alpha building TOWERZ (no turret, durable)
/// armed with ZAP (rate 30, burst `burst`), maximum ammunition `max`,
/// initial ammunition `initial` (None: the field absent) and a reload
/// field far longer than any test here (claim 11: it plays no part). The
/// target BUGGY is durable and unarmed, so it stays put and never fires.
fn tower(max: i32, initial: Option<i32>, burst: i32) -> Ruleset {
    let mut r = testkit::combat_ruleset();
    r.sim.weapons.insert(
        "ZAP".into(),
        WeaponType {
            burst: Some(burst),
            ..zap(30)
        },
    );
    let buggy = r.sim.vehicles.get_mut("BUGGY").unwrap();
    buggy.strength = Some(30000);
    buggy.primary = None;
    buggy.secondary = None;
    r.registries
        .get_mut("buildings")
        .unwrap()
        .push("TOWERZ".into());
    r.sim.buildings.insert(
        "TOWERZ".into(),
        BuildingType {
            strength: Some(30000),
            foundation: Some([1, 1]),
            primary: Some("ZAP".into()),
            combat: CombatFields {
                ammo: Some(max),
                initial_ammo: initial,
                reload: Some(100_000),
                empty_reload: Some(100_000),
                ..Default::default()
            },
            ..Default::default()
        },
    );
    r
}

/// The tower two cells west of the middle and the BUGGY two cells east of
/// it, set as the tower's target.
fn duel(r: &Ruleset) -> (World, ObjectId, ObjectId) {
    let mut w = testkit::game_with(r, N, 7);
    let t = testkit::spawn(
        &mut w,
        Kind::Building,
        "TOWERZ",
        0,
        testkit::mid(N, -2, 0),
        EAST,
    );
    let b = testkit::spawn(
        &mut w,
        Kind::Vehicle,
        "BUGGY",
        1,
        testkit::mid(N, 2, 0),
        WEST,
    );
    w.obj_mut(t).unwrap().target = Some(Target::Object(b));
    (w, t, b)
}

/// What one tick shows of the building, read at the end of that tick.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Tick {
    /// The tick's number.
    tick: i32,
    /// Ammunition at the end of the tick.
    ammo: i32,
    /// Whether the building made a shot in the tick.
    shot: bool,
    /// The rearm countdown at the end of the tick, from the tick itself.
    countdown: i32,
    /// Whether it still holds the BUGGY as its target.
    target: bool,
}

/// Runs `n` ticks of the real pipeline and reads the building at the end of
/// each.
fn record(w: &mut World, t: ObjectId, b: ObjectId, n: i32) -> Vec<Tick> {
    let mut out = Vec::new();
    for _ in 0..n {
        let tick = w.tick;
        let before = w.obj(t).unwrap().last_fired;
        testkit::run(w, 1);
        let o = w.obj(t).unwrap();
        out.push(Tick {
            tick,
            ammo: o.combat.ammo,
            shot: o.last_fired != before,
            countdown: o.combat.rearm_until.saturating_sub(tick),
            target: o.target == Some(Target::Object(b)),
        });
    }
    out
}

/// The shot ticks of a record.
fn shots(rec: &[Tick]) -> Vec<&Tick> {
    rec.iter().filter(|x| x.shot).collect()
}

// spec: S-0301 claim 10 (refill) and M-0004 (pillbox, burst-2 weapon, 3 rounds: countdowns 1, 1, then one draw from 3 to 5; the next group 1, 1, then base + u; ammunition at the end of each tick of a group 3, 2, 1, 3, never 0; it keeps its target and never stops for lack of ammunition, with a long reload field)
#[test]
fn claim_10_refill_three_rounds_burst_two() {
    let (mut w, t, b) = duel(&tower(3, Some(3), 2));
    let rec = record(&mut w, t, b, 260);
    assert!(
        rec.iter().all(|x| x.ammo != 0),
        "never 0 at the end of a tick"
    );
    assert!(rec.iter().all(|x| x.target), "keeps its target throughout");
    let s = shots(&rec);
    assert!(s.len() >= 9, "keeps firing group after group: {}", s.len());
    // Groups of three, back to back within a group.
    for g in s.chunks(3).take(3) {
        assert_eq!(g[1].tick, g[0].tick + 1);
        assert_eq!(g[2].tick, g[1].tick + 1);
    }
    // Countdowns: group 1 gives 1, 1, R(3, 5); group 2 gives 1, 1, base + u.
    let c: Vec<i32> = s.iter().map(|x| x.countdown).collect();
    assert_eq!(&c[0..2], &[1, 1]);
    assert!(
        (3..=5).contains(&c[2]),
        "a new burst's in-burst draw: {}",
        c[2]
    );
    assert_eq!(&c[3..5], &[1, 1]);
    assert!((30..=32).contains(&c[5]), "the full delay: {}", c[5]);
    // Ammunition at the end of each tick of the second group: 3 the tick
    // before, then 2, 1, 3.
    let i = rec.iter().position(|x| x.tick == s[3].tick).unwrap();
    let a: Vec<i32> = rec[i - 1..i + 3].iter().map(|x| x.ammo).collect();
    assert_eq!(a, [3, 2, 1, 3]);
    // The next group starts once the countdown has run (the building's
    // processing while rearming comes every 2 ticks, S-0215 claim 32).
    let due = s[5].tick + c[5];
    assert!(
        (due..=due + 1).contains(&s[6].tick),
        "{} {}",
        due,
        s[6].tick
    );
}

// spec: S-0301 claim 10 (refill) and M-0004 (pillbox, burst-2 weapon, 4 rounds: countdowns 1, 1, 1 with no draw, then base + u; ammunition 4, 3, 2, 1, 4)
#[test]
fn claim_10_refill_four_rounds_burst_two() {
    let (mut w, t, b) = duel(&tower(4, Some(4), 2));
    let rec = record(&mut w, t, b, 200);
    assert!(rec.iter().all(|x| x.ammo != 0 && x.target));
    let s = shots(&rec);
    assert!(s.len() >= 8);
    for g in s.chunks(4).take(2) {
        let c: Vec<i32> = g.iter().map(|x| x.countdown).collect();
        assert_eq!(&c[0..3], &[1, 1, 1]);
        assert!((30..=32).contains(&c[3]), "{c:?}");
    }
    let i = rec.iter().position(|x| x.tick == s[4].tick).unwrap();
    let a: Vec<i32> = rec[i - 1..i + 4].iter().map(|x| x.ammo).collect();
    assert_eq!(a, [4, 3, 2, 1, 4]);
}

// spec: S-0301 claim 10 and scenario F9 (single-shot weapon, 3 rounds, long reload field: shots in t, t + 1, t + 2, 3 rounds again at the end of t + 2, the group repeats after each full delay; never 0 at the end of a tick, the target kept)
#[test]
fn claim_10_scenario_f9_building_groups() {
    let (mut w, t, b) = duel(&tower(3, Some(3), 1));
    let rec = record(&mut w, t, b, 140);
    assert!(rec.iter().all(|x| x.ammo != 0 && x.target));
    let s = shots(&rec);
    assert!(s.len() >= 9, "{}", s.len());
    for g in s.chunks(3).take(3) {
        assert_eq!([g[1].tick - g[0].tick, g[2].tick - g[1].tick], [1, 1]);
        assert_eq!([g[0].countdown, g[1].countdown], [1, 1]);
        assert!((30..=32).contains(&g[2].countdown));
        assert_eq!(
            g[2].ammo, 3,
            "full again at the end of the third shot's tick"
        );
    }
    // Never out of ammunition: every check at the end of a tick answers
    // can fire or rearming.
    let a = check(&w, t, w.obj(t).unwrap().target, 0, true);
    assert!(matches!(a, Answer::CanFire | Answer::Rearming), "{a:?}");
}

// spec: S-0301 claim 10 (a building that starts with 0 rounds and A > 0 is filled at the end of its first tick, and from its first attempt after that fires as one that started full)
#[test]
fn claim_10_starts_empty_filled_after_first_tick() {
    let (mut w, t, b) = duel(&tower(4, Some(0), 2));
    assert_eq!(w.obj(t).unwrap().combat.ammo, 0, "0 when created");
    let rec = record(&mut w, t, b, 80);
    assert_eq!(rec[0].ammo, 4, "full at the end of its first tick");
    assert!(rec.iter().all(|x| x.ammo != 0 && x.target));
    let s = shots(&rec);
    let c: Vec<i32> = s.iter().take(4).map(|x| x.countdown).collect();
    assert_eq!(&c[0..3], &[1, 1, 1], "a first group of four");
    assert!((30..=32).contains(&c[3]));
}

// spec: S-0301 claim 10 (an initial count above A is spent first, and every group after it holds A rounds; quirks.initial_ammo_above_capacity default original)
#[test]
fn claim_10_initial_above_maximum_spent_first() {
    let (mut w, t, b) = duel(&tower(2, Some(5), 1));
    assert_eq!(w.obj(t).unwrap().combat.ammo, 5);
    let rec = record(&mut w, t, b, 120);
    let s = shots(&rec);
    let c: Vec<i32> = s.iter().take(7).map(|x| x.countdown).collect();
    assert_eq!(&c[0..4], &[1, 1, 1, 1], "five rounds first");
    assert!((30..=32).contains(&c[4]));
    assert_eq!(c[5], 1, "then groups of A = 2");
    assert!((30..=32).contains(&c[6]));
    assert_eq!(s[4].ammo, 2, "refilled to A, not to the initial count");
}

// spec: S-0301 claim 10 and Edge cases (A = −1 with an initial count of 0: the refill gives −1, so the building then has unlimited ammunition; no switch)
#[test]
fn claim_10_unlimited_maximum_with_zero_initial() {
    let (mut w, t, b) = duel(&tower(-1, Some(0), 1));
    assert_eq!(w.obj(t).unwrap().combat.ammo, 0);
    let rec = record(&mut w, t, b, 80);
    assert_eq!(rec[0].ammo, -1, "unlimited after its first tick");
    let s = shots(&rec);
    assert!(s.len() >= 2);
    // Unlimited ammunition is not affected by the 1-tick rule.
    assert!(
        s.iter()
            .all(|x| (30..=32).contains(&x.countdown) && x.ammo == -1)
    );
}

// spec: S-0301 claim 10 (with A = 0 the refill gives 0: the building stays out of ammunition and makes no shot)
#[test]
fn claim_10_zero_maximum_stays_empty() {
    let (mut w, t, b) = duel(&tower(0, None, 1));
    assert_eq!(w.obj(t).unwrap().combat.ammo, 0);
    let rec = record(&mut w, t, b, 30);
    assert!(rec.iter().all(|x| x.ammo == 0 && !x.shot));
}

// spec: S-0301 claim 10 and Import mapping (no refill for a type with the hospital flag or the armory flag; absent no)
#[test]
fn claim_10_hospital_and_armory_are_not_refilled() {
    for flag in ["hospital", "armory", "none"] {
        let mut r = tower(3, Some(3), 1);
        let bt = r.sim.buildings.get_mut("TOWERZ").unwrap();
        match flag {
            "hospital" => bt.hospital = Some(true),
            "armory" => bt.armory = Some(true),
            _ => {}
        }
        let (mut w, t, b) = duel(&r);
        let rec = record(&mut w, t, b, 100);
        let n = shots(&rec).len();
        if flag == "none" {
            assert!(n > 3, "refilled: {n}");
        } else {
            assert_eq!(n, 3, "{flag}: three shots and never again");
            assert_eq!(rec.last().unwrap().ammo, 0, "{flag}");
        }
    }
}

// ---------------------------------------------------------------------------
// Scenario F9's paired-refill part: the refill comes after both attempts of
// a paired firing (claim 9's passing upgrade gate, S-0215 claim 32).

/// [`tower`] unarmed, with its maximum ammunition 2 and initial `initial`,
/// and the upgrade U1 supplying ZAP in both slots (a passing gate: both
/// slots the same type), rate 30.
fn paired(initial: i32) -> (World, ObjectId, ObjectId) {
    let mut r = tower(2, Some(initial), 1);
    // A slow visible projectile, so that both shots of the tick are still in
    // flight at its end (an invisible one detonates at once).
    r.sim.projectiles.insert(
        "Slug".into(),
        glasnost_rules::sim::ProjectileType::default(),
    );
    let zap = r.sim.weapons.get_mut("ZAP").unwrap();
    zap.speed = Some(5);
    zap.projectile = Some("Slug".into());
    let host = r.sim.buildings.get_mut("TOWERZ").unwrap();
    host.primary = None;
    host.econ.upgrades = Some(3);
    r.registries.get_mut("buildings").unwrap().push("U1".into());
    r.sim.buildings.insert(
        "U1".into(),
        BuildingType {
            strength: Some(100),
            foundation: Some([1, 1]),
            primary: Some("ZAP".into()),
            secondary: Some("ZAP".into()),
            ..Default::default()
        },
    );
    let (mut w, t, b) = duel(&r);
    let u = glasnost_sim::objects::type_index(w.rules(), Kind::Building, b"U1").unwrap();
    glasnost_sim::econ::placement::add_upgrade(&mut w, t, u);
    let tick = w.tick;
    w.obj_mut(t)
        .unwrap()
        .beh
        .force(Behaviour::Attack, tick, true);
    (w, t, b)
}

/// The building's paired attempts in its first processing, read on a copy
/// of the world: the projectiles launched, the ammunition and the
/// countdown just after both attempts, before the end of the tick.
fn first_processing(w: &World, t: ObjectId) -> (usize, i32, i32) {
    let mut c = w.clone();
    let tick = c.tick;
    glasnost_sim::combat::behave::building_process(&mut c, t);
    let o = c.obj(t).unwrap();
    (
        c.projectiles.len(),
        o.combat.ammo,
        o.combat.rearm_until - tick,
    )
}

// spec: S-0301 claims 9 and 10, scenario F9 paired-refill part variant a (initial 2: the primary shot takes the 1-tick delay and leaves 1; the secondary, made with the last round, takes claim 2's normal delay and leaves 0; the building holds 2 at the end of that tick)
#[test]
fn claim_10_paired_refill_variant_a() {
    let (mut w, t, b) = paired(2);
    let (shots, ammo, countdown) = first_processing(&w, t);
    assert_eq!(
        (shots, ammo),
        (2, 0),
        "both attempts made, each with a round"
    );
    let rec = record(&mut w, t, b, 1);
    assert!(rec[0].shot);
    assert_eq!(rec[0].ammo, 2, "refilled after both attempts, in that tick");
    assert_eq!(rec[0].countdown, countdown);
    assert!(
        (30..=32).contains(&countdown),
        "the secondary takes the normal delay, not the 1-tick one: {countdown}"
    );
}

// spec: S-0301 claims 9 and 10, scenario F9 paired-refill part variant b (initial 1: the primary empties the magazine; the secondary attempt asks no firing check, is still made at 0 rounds, uses no round and takes the full delay; the building holds 2 at the end of that tick)
#[test]
fn claim_10_paired_refill_variant_b_unchecked_secondary_at_zero() {
    let (mut w, t, b) = paired(1);
    let (shots, ammo, countdown) = first_processing(&w, t);
    assert_eq!(shots, 2, "the unchecked secondary still fires at 0 rounds");
    assert_eq!(ammo, 0, "it uses no round");
    assert!((30..=32).contains(&countdown), "{countdown}");
    let rec = record(&mut w, t, b, 1);
    assert_eq!(
        rec[0].ammo, 2,
        "not 1 (a refill between the attempts), not 0"
    );
    assert_eq!(rec[0].countdown, countdown);
}
