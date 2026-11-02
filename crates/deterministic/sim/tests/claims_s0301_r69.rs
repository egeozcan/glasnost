//! Claim tests of S-0301 revision 69 (claim 9: a materialising firer and
//! the infantry answers within rank 4), S-0607 revision 38 (claim 11: an
//! infantry lifter's answers ahead of the gripped-unit *rearming*), S-0304
//! revision 6 (claim 2's drop occasions) and S-0310 revision 8 (claim 11's
//! keep row and the infantry healer's reply), M8 step A3 intake 79.
//! Synthetic data only: every type, weapon, warhead and number is invented.
//!
//! A drop in tick *k* shows after `tick::step` has run that tick, when the
//! tick number reads *k* + 1. The ledger is `docs/m8-claims/intake79.md`.

use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{
    ArmourClass, Decimal, Locomotor, ProjectileType, UnitType, WarheadType, WeaponType,
};
use glasnost_sim::combat::check::{self, Answer};
use glasnost_sim::combat::effects::Effect;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::teleport::TeleportState;
use glasnost_sim::testkit;
use glasnost_sim::world::{ObjectId, Target, World};

const SIZE: i32 = 24;
const ALPHA: u8 = 0;
const BETA: u8 = 1;
/// The check period of S-0309 claim 29, written out for the expected ticks.
const PERIOD: i32 = 16;
/// The armour class *light*'s index in the versus list.
const LIGHT: usize = 3;

fn dec(s: &str) -> Decimal {
    Decimal(s.into())
}

/// Eleven versus values: `v` for class `class`, 1 for the others.
fn only(class: usize, v: &str) -> Option<Vec<Decimal>> {
    let mut l = vec![dec("1"); 11];
    l[class] = dec(v);
    Some(l)
}

fn register(r: &mut Ruleset, reg: &str, name: &str) {
    let l = r.registries.entry(reg.into()).or_default();
    if !l.iter().any(|n| n == name) {
        l.push(name.into());
    }
}

fn vehicle(r: &mut Ruleset, name: &str, f: impl FnOnce(&mut UnitType)) {
    let mut t = r.sim.vehicles["BUGGY"].clone();
    t.strength = Some(1000);
    t.primary = None;
    t.secondary = None;
    t.combat.armor = None;
    f(&mut t);
    register(r, "vehicles", name);
    r.sim.vehicles.insert(name.into(), t);
}

fn infantry(r: &mut Ruleset, name: &str, f: impl FnOnce(&mut UnitType)) {
    let mut t = r.sim.infantry["WALKER"].clone();
    t.strength = Some(1000);
    t.primary = None;
    t.secondary = None;
    f(&mut t);
    register(r, "infantry", name);
    r.sim.infantry.insert(name.into(), t);
}

fn warhead(r: &mut Ruleset, name: &str, f: impl FnOnce(&mut WarheadType)) {
    let mut h = WarheadType {
        verses: only(LIGHT, "1"),
        ..Default::default()
    };
    f(&mut h);
    r.sim.warheads.insert(name.into(), h);
}

/// A weapon of range 5 cells firing every 30 ticks.
fn weapon(r: &mut Ruleset, name: &str, warhead_name: &str, damage: i32, projectile: &str) {
    r.sim.weapons.insert(
        name.into(),
        WeaponType {
            damage: Some(damage),
            rof: Some(30),
            range: Some(dec("5")),
            speed: Some(100),
            projectile: Some(projectile.into()),
            warhead: Some(warhead_name.into()),
            ..Default::default()
        },
    );
}

/// The rules of this file:
/// - warhead `H` (0 against *light*, S-0304 V1's test warhead) with weapon
///   `W`, vehicle `DEF` (primary `W`) and vehicle `TARGET` (class *light*,
///   unarmed);
/// - warhead `PLAIN` (1 everywhere) with weapon `Plain` (the testkit's
///   invisible `Bullet`, which is not anti-air); weapon `AirOnly` (an
///   anti-air projectile that is not anti-ground); warhead `HEAL0` (0
///   against *light*) with the healing weapon `Mend0`; warhead `BUG` (a
///   parasite warhead) with weapon `Latch`;
/// - vehicles `GUN` (primary `Plain`), `SKY` (primary `AirOnly`), `BARE`
///   (no weapon), `LEECH` (primary `Latch`), `PLAINV` (class *heavy*,
///   unarmed), `TELE` (the teleporting locomotor, primary `AirOnly`);
/// - infantry `MEDIC` (primary `Mend0`), `HURT` (class *light*), `GRUNT`
///   (unarmed, class *heavy*).
fn rules() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    warhead(&mut r, "H", |h| h.verses = only(LIGHT, "0"));
    weapon(&mut r, "W", "H", 20, "Bullet");
    vehicle(&mut r, "DEF", |t| t.primary = Some("W".into()));
    vehicle(&mut r, "TARGET", |t| {
        t.combat.armor = Some(ArmourClass::Light)
    });
    warhead(&mut r, "PLAIN", |_| {});
    warhead(&mut r, "HEAL0", |h| h.verses = only(LIGHT, "0"));
    warhead(&mut r, "BUG", |h| h.parasite = Some(true));
    weapon(&mut r, "Plain", "PLAIN", 20, "Bullet");
    weapon(&mut r, "Mend0", "HEAL0", -20, "Bullet");
    weapon(&mut r, "Latch", "BUG", 20, "Bullet");
    r.sim.projectiles.insert(
        "Sky".into(),
        ProjectileType {
            inviso: Some(true),
            aa: Some(true),
            ag: Some(false),
            ..Default::default()
        },
    );
    weapon(&mut r, "AirOnly", "PLAIN", 20, "Sky");
    vehicle(&mut r, "GUN", |t| t.primary = Some("Plain".into()));
    vehicle(&mut r, "SKY", |t| t.primary = Some("AirOnly".into()));
    vehicle(&mut r, "BARE", |_| {});
    vehicle(&mut r, "LEECH", |t| t.primary = Some("Latch".into()));
    vehicle(&mut r, "PLAINV", |t| {
        t.combat.armor = Some(ArmourClass::Heavy)
    });
    vehicle(&mut r, "TELE", |t| {
        t.primary = Some("AirOnly".into());
        t.locomotor = Some(Locomotor::Teleport);
        t.harvester = Some(false);
    });
    infantry(&mut r, "MEDIC", |t| t.primary = Some("Mend0".into()));
    infantry(&mut r, "HURT", |t| {
        t.combat.armor = Some(ArmourClass::Light)
    });
    infantry(&mut r, "GRUNT", |t| {
        t.combat.armor = Some(ArmourClass::Heavy)
    });
    r
}

fn at(dx: i32, dy: i32) -> CellXY {
    testkit::mid(SIZE, dx, dy)
}

fn game() -> World {
    testkit::game_with(&rules(), SIZE, 1)
}

fn put(w: &mut World, kind: Kind, name: &str, house: u8, dx: i32, dy: i32) -> ObjectId {
    testkit::spawn(w, kind, name, house, at(dx, dy), 0)
}

fn obj(x: ObjectId) -> Option<Target> {
    Some(Target::Object(x))
}

fn ask(w: &World, id: ObjectId, t: Option<Target>, slot: usize) -> Answer {
    check::check(w, id, t, slot, false)
}

fn target(w: &World, id: ObjectId) -> Option<Target> {
    w.obj(id).and_then(|o| o.target)
}

fn step(w: &mut World) {
    glasnost_sim::tick::step(w);
}

/// The first check tick after tick `n`.
fn next_check(n: i32) -> i32 {
    (n / PERIOD + 1) * PERIOD
}

/// Makes `id` materialise (phase in, S-0207 claim 20 f) for `d` ticks
/// from now.
fn materialise(w: &mut World, id: ObjectId, d: i64) {
    let t = w.tick;
    w.obj_mut(id).unwrap().teleport = Some(TeleportState {
        phasing: Some((t, d)),
        ..Default::default()
    });
    assert!(glasnost_sim::teleport::phasing_in(w, id));
}

/// Lets `lifter` grip `unit` (S-0607 Terms), set directly.
fn grip(w: &mut World, lifter: ObjectId, unit: ObjectId) {
    w.obj_mut(lifter).unwrap().lift.gripped = Some(unit);
    w.obj_mut(unit).unwrap().lift.lifter = Some(lifter);
    assert!(glasnost_sim::lift::grips(w, lifter, unit));
}

/// Gives `id` target `x` and behaviour `b` directly, its targeting timer
/// 200 ticks away and its behaviour processing not due for a long time.
fn hold(w: &mut World, id: ObjectId, x: Target, b: Behaviour) {
    let t = w.tick;
    let o = w.obj_mut(id).unwrap();
    o.beh.force(b, t, false);
    o.target = Some(x);
    o.targeting_timer = t + 200;
    o.beh.processed(t, 1000);
}

/// Runs whole ticks, at most `limit`, until `id` no longer holds `x`;
/// returns the tick in which it dropped it.
fn drop_tick(w: &mut World, id: ObjectId, x: Target, limit: i32) -> Option<i32> {
    for _ in 0..limit {
        step(w);
        if target(w, id) != Some(x) {
            return Some(w.tick - 1);
        }
    }
    None
}

fn shots_by(w: &World, id: ObjectId) -> usize {
    w.effects
        .iter()
        .filter(|e| matches!(e, Effect::Shot { firer, .. } if *firer == id))
        .count()
}

// ---------------------------------------------------------------------------
// S-0301 claim 9, *A materialising firer within rank 4*.
// ---------------------------------------------------------------------------

// spec: S-0301 claim 9 (A materialising firer within rank 4: rearming for every target and weapon; it outranks every other rank-4 answer, the projectile refusals of S-0306 claim 23, the hidden-target impossible of S-0516 claim 24, the illegal for a parasite warhead against a protected object of S-0513 claim 27 and the empty slot's impossible among them)
#[test]
fn a_materialising_firer_gets_rearming_ahead_of_the_rank_4_refusals() {
    let mut w = game();
    let gun = put(&mut w, Kind::Vehicle, "GUN", ALPHA, -2, 0);
    let sky = put(&mut w, Kind::Vehicle, "SKY", ALPHA, -2, 2);
    let bare = put(&mut w, Kind::Vehicle, "BARE", ALPHA, -2, -2);
    let leech = put(&mut w, Kind::Vehicle, "LEECH", ALPHA, -2, 4);
    let up = put(&mut w, Kind::Vehicle, "PLAINV", BETA, 2, 0);
    let hid = put(&mut w, Kind::Vehicle, "PLAINV", BETA, 2, 2);
    let safe = put(&mut w, Kind::Vehicle, "PLAINV", BETA, 2, 4);
    let cell = Some(Target::Cell(at(2, -2)));
    // An airborne target and a projectile that is not anti-air (S-0306 claim 23).
    w.obj_mut(up).unwrap().pos.z += 600;
    // A target hidden from the firer's house (S-0516 claim 24): a cloaked vehicle, no sensor.
    w.obj_mut(hid).unwrap().cloak.state = glasnost_sim::cloak::CloakState::Cloaked;
    // A protected object (S-0513 claim 27).
    let tick = w.tick;
    w.obj_mut(safe).unwrap().sw.protection = Some(glasnost_sim::superweapons::Protection {
        start: tick,
        duration: 1000,
        kind: glasnost_sim::superweapons::ProtectKind::Curtain,
        tint: Default::default(),
    });
    let cases = [
        (gun, obj(up), 0, Answer::Illegal, "airborne target"),
        (sky, cell, 0, Answer::Illegal, "a cell and no anti-ground"),
        (gun, obj(hid), 0, Answer::Impossible, "hidden target"),
        (
            leech,
            obj(safe),
            0,
            Answer::Illegal,
            "parasite at a protected object",
        ),
        (
            bare,
            obj(up),
            0,
            Answer::Impossible,
            "no weapon in the slot",
        ),
        (gun, obj(up), 1, Answer::Impossible, "no weapon in slot 1"),
    ];
    for (f, t, slot, control, what) in cases {
        assert_eq!(ask(&w, f, t, slot), control, "control: {what}");
    }
    for f in [gun, sky, bare, leech] {
        materialise(&mut w, f, 100);
    }
    for (f, t, slot, _, what) in cases {
        assert_eq!(
            ask(&w, f, t, slot),
            Answer::Rearming,
            "materialising: {what}"
        );
    }
}

// spec: S-0214 claim 6 (e) and S-0301 claim 9, interim under Q-2356: the attack rule's check (S-0214 claim 6 (e), "other reasons for not firing at this moment ... do not count") passes over the materialising rearming of S-0301 claim 9, as it passes over ranks 5 to 8, so the target tests decide there.
#[test]
fn q2356_the_attack_rules_check_passes_over_the_materialising_rearming() {
    let mut w = game();
    let gun = put(&mut w, Kind::Vehicle, "GUN", ALPHA, -2, 0);
    let sky = put(&mut w, Kind::Vehicle, "SKY", ALPHA, -2, 2);
    let up = put(&mut w, Kind::Vehicle, "PLAINV", BETA, 2, 0);
    let low = put(&mut w, Kind::Vehicle, "PLAINV", BETA, 2, 2);
    w.obj_mut(up).unwrap().pos.z += 600;
    let cell = Some(Target::Cell(at(2, -2)));
    for f in [gun, sky] {
        materialise(&mut w, f, 100);
    }
    assert_eq!(check::check_lasting(&w, gun, obj(up), 0), Answer::Illegal);
    assert_eq!(check::check_lasting(&w, sky, cell, 0), Answer::Illegal);
    assert_ne!(check::check_lasting(&w, gun, obj(low), 0), Answer::Illegal);
    // The ordinary check gives the materialising rearming.
    assert_eq!(ask(&w, gun, obj(up), 0), Answer::Rearming);
    assert_eq!(ask(&w, gun, obj(low), 0), Answer::Rearming);
}

// spec: S-0301 claim 9 (A materialising firer within rank 4: for infantry it yields to the answers an infantry firer is asked first, the infantry healer's illegal of S-0310 claim 11, which outranks every answer of the general part, the materialising rearming among them) and S-0310 claim 11 (an infantry healer: illegal when its target is absent, not infantry or at full health)
#[test]
fn a_materialising_infantry_healer_gets_its_illegal_first() {
    let mut w = game();
    let medic = put(&mut w, Kind::Infantry, "MEDIC", ALPHA, -1, 0);
    let full = put(&mut w, Kind::Infantry, "GRUNT", ALPHA, 1, 0);
    let car = put(&mut w, Kind::Vehicle, "PLAINV", ALPHA, 1, 2);
    let hurt = put(&mut w, Kind::Infantry, "HURT", ALPHA, 1, -2);
    w.obj_mut(car).unwrap().health = 500;
    w.obj_mut(hurt).unwrap().health = 500;
    materialise(&mut w, medic, 100);
    assert_eq!(ask(&w, medic, obj(full), 0), Answer::Illegal, "full health");
    assert_eq!(ask(&w, medic, obj(car), 0), Answer::Illegal, "not infantry");
    assert_eq!(ask(&w, medic, None, 0), Answer::Illegal, "no target");
    // An infantry target below full health passes the healer's test; the
    // versus value of 0 (rank 9) is then outranked by the materialising rearming.
    assert_eq!(ask(&w, medic, obj(hurt), 0), Answer::Rearming);
    w.obj_mut(medic).unwrap().teleport = None;
    assert_eq!(
        ask(&w, medic, obj(hurt), 0),
        Answer::Illegal,
        "control: versus 0"
    );
}

// spec: S-0301 claim 9 (Grip refusals within rank 4 and A materialising firer within rank 4: a firer that meets the healer's illegal, the materialising rearming and the gripped-unit rearming gets the healer's illegal first, then the materialising rearming, then the gripped-unit rearming) and S-0607 claim 11 (Against its gripped unit: for an infantry lifter the answers an infantry firer is asked first precede the gripped-unit rearming)
#[test]
fn an_infantry_healer_lifter_gets_the_healers_illegal_ahead_of_its_grip() {
    let mut w = game();
    let medic = put(&mut w, Kind::Infantry, "MEDIC", ALPHA, -1, 0);
    let car = put(&mut w, Kind::Vehicle, "PLAINV", BETA, 1, 0);
    let hurt = put(&mut w, Kind::Infantry, "HURT", BETA, 1, 2);
    w.obj_mut(car).unwrap().health = 500;
    w.obj_mut(hurt).unwrap().health = 500;
    // A seized vehicle is not infantry: the healer's illegal, grip or not.
    grip(&mut w, medic, car);
    assert_eq!(
        ask(&w, medic, obj(car), 0),
        Answer::Illegal,
        "gripped vehicle"
    );
    materialise(&mut w, medic, 100);
    assert_eq!(
        ask(&w, medic, obj(car), 0),
        Answer::Illegal,
        "gripped vehicle, materialising"
    );
    // A gripped infantry target below full health passes the healer's test:
    // the materialising rearming, and without it the gripped-unit rearming,
    // which also outranks the rank-9 zero versus value.
    grip(&mut w, medic, hurt);
    assert_eq!(
        ask(&w, medic, obj(hurt), 0),
        Answer::Rearming,
        "materialising"
    );
    w.obj_mut(medic).unwrap().teleport = None;
    assert_eq!(ask(&w, medic, obj(hurt), 0), Answer::Rearming, "gripped");
}

// spec: S-0607 claim 11 (Against its gripped unit: an infantry lifter that gets one of the answers asked first drops the gripped unit as its target, at S-0309 claim 29's check or through its firing step's reply; the grip stays, as claim 12 has no row for a target change) and S-0310 claim 11 (the infantry healer's reply in its firing step drops a target that is not infantry)
#[test]
fn an_infantry_healer_lifter_drops_its_gripped_vehicle_and_keeps_the_grip() {
    let mut w = game();
    let medic = put(&mut w, Kind::Infantry, "MEDIC", ALPHA, -1, 0);
    let car = put(&mut w, Kind::Vehicle, "PLAINV", BETA, 1, 0);
    w.obj_mut(car).unwrap().health = 500;
    while w.tick.rem_euclid(PERIOD) != 3 {
        step(&mut w);
    }
    let n = w.tick;
    grip(&mut w, medic, car);
    hold(&mut w, medic, Target::Object(car), Behaviour::Guard);
    assert_eq!(drop_tick(&mut w, medic, Target::Object(car), 20), Some(n));
    assert!(
        glasnost_sim::lift::grips(&w, medic, car),
        "the grip stays after the target is dropped"
    );
}

// spec: S-0301 claim 9 (A materialising firer within rank 4: a materialising firer gets rearming against a target that a rank-4 projectile refusal makes illegal, so the periodic check keeps it) and S-0309 claim 29 and scenario A6 part p run p10a (a materialising holder keeps the refused target at every check tick in which it still phases in and drops it at the first check tick after its phase-in ends); the refusal here is S-0306 claim 23's cell row
#[test]
fn p10a_a_materialising_holder_keeps_its_refused_target_until_the_phase_in_ends() {
    for phasing in [false, true] {
        let mut w = game();
        let tele = put(&mut w, Kind::Vehicle, "TELE", ALPHA, -1, 0);
        while w.tick.rem_euclid(PERIOD) != 6 {
            step(&mut w);
        }
        let n = w.tick;
        let c = Target::Cell(at(2, 0));
        hold(&mut w, tele, c, Behaviour::Attack);
        if phasing {
            // The phase-in ends in the movement update of tick n + 40, after
            // the check of that tick's turn; check ticks n + 10 and n + 26 fall in it.
            materialise(&mut w, tele, 40);
            assert_eq!(ask(&w, tele, Some(c), 0), Answer::Rearming);
        } else {
            assert_eq!(ask(&w, tele, Some(c), 0), Answer::Illegal);
        }
        w.effects.clear();
        let dropped = drop_tick(&mut w, tele, c, 80);
        if phasing {
            assert_eq!(
                dropped,
                Some(n + 42),
                "the first check tick after the phase-in"
            );
            assert!(!glasnost_sim::teleport::phasing_in(&w, tele));
        } else {
            assert_eq!(
                dropped,
                Some(next_check(n)),
                "control: the first check tick"
            );
        }
        assert_eq!(shots_by(&w, tele), 0, "no shot at the refused cell");
    }
}

// ---------------------------------------------------------------------------
// S-0304 claim 2: the drop occasions at their edges.
// ---------------------------------------------------------------------------

// spec: S-0304 claim 2 (revision 6: while the countdown runs the answer is rearming, not illegal; a vehicle drops the target at the first tick whose tick number is a multiple of 16 at whose turn the countdown has run out, the tick in which it runs out included) and scenario V1 run 4b; the illegal answer here is claim 1's versus value of exactly 0, a rank-9 reason
#[test]
fn v1_4b_a_countdown_that_runs_out_in_a_check_tick_drops_in_that_tick() {
    for late in [0, 1] {
        let mut w = game();
        let d = put(&mut w, Kind::Vehicle, "DEF", ALPHA, -1, 0);
        let x = put(&mut w, Kind::Vehicle, "TARGET", BETA, 2, 0);
        while w.tick.rem_euclid(PERIOD) != 2 {
            step(&mut w);
        }
        let n = w.tick;
        hold(&mut w, d, Target::Object(x), Behaviour::Attack);
        // The countdown runs while the tick number is below rearm_until.
        w.obj_mut(d).unwrap().combat.rearm_until = next_check(n) + late;
        assert_eq!(ask(&w, d, obj(x), 0), Answer::Rearming);
        w.effects.clear();
        let dropped = drop_tick(&mut w, d, Target::Object(x), 40);
        let expected = if late == 0 {
            next_check(n)
        } else {
            next_check(n) + PERIOD
        };
        assert_eq!(dropped, Some(expected), "late {late}");
        assert_eq!(shots_by(&w, d), 0, "late {late}: never fires");
    }
}

// spec: S-0304 claim 2 (revision 6: the exception is an infantry healer whose own illegal holds, its target absent, not infantry or at full health: that answer outranks the countdown's rearming, so it gets illegal while its countdown runs; a healer can drop the target earlier, at its firing step) and S-0310 claim 11 (the infantry healer's illegal and its firing-step reply)
#[test]
fn an_infantry_healer_drops_a_full_health_target_in_its_firing_step_while_its_countdown_runs() {
    let mut w = game();
    let h = put(&mut w, Kind::Infantry, "MEDIC", ALPHA, -1, 0);
    let y = put(&mut w, Kind::Infantry, "GRUNT", ALPHA, 1, 0);
    while w.tick.rem_euclid(PERIOD) != 2 {
        step(&mut w);
    }
    let n = w.tick;
    hold(&mut w, h, Target::Object(y), Behaviour::Guard);
    w.obj_mut(h).unwrap().combat.rearm_until = n + 200;
    assert_eq!(check::check(&w, h, obj(y), 0, true), Answer::Illegal);
    assert_eq!(
        drop_tick(&mut w, h, Target::Object(y), 10),
        Some(n),
        "in its first turn"
    );
}

// spec: S-0304 claim 2 (revision 6: S-0310 claim 11's reply table keeps a target of the kind the healer heals that is below full health and still gives illegal, by a versus value of exactly 0, so the periodic check drops it; while the countdown runs the answer is rearming, so the check keeps it to the first check tick at whose turn the countdown has run out) and S-0310 claim 11
#[test]
fn an_infantry_healers_kept_hurt_target_waits_for_the_countdown() {
    let mut w = game();
    let h = put(&mut w, Kind::Infantry, "MEDIC", ALPHA, -1, 0);
    let y = put(&mut w, Kind::Infantry, "HURT", ALPHA, 1, 0);
    w.obj_mut(y).unwrap().health = 500;
    while w.tick.rem_euclid(PERIOD) != 2 {
        step(&mut w);
    }
    let n = w.tick;
    hold(&mut w, h, Target::Object(y), Behaviour::Guard);
    // Check ticks n + 14 and n + 30 fall in the countdown; n + 46 is the first after it.
    w.obj_mut(h).unwrap().combat.rearm_until = n + 40;
    w.effects.clear();
    assert_eq!(drop_tick(&mut w, h, Target::Object(y), 80), Some(n + 46));
    assert_eq!(shots_by(&w, h), 0);
}
