//! Claim tests for psychic weapons (S-0610) on synthetic data: invented
//! types, weapons and warheads, every number our own. The psychic reveal's
//! superweapon rules come from `tests/sw_common` with one more type.

mod sw_common;

use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{BuildingType, Decimal, UnitType, WarheadType, WeaponType};
use glasnost_rules::sim_econ::SuperweaponType;
use glasnost_sim::combat::area::{self, Detonation};
use glasnost_sim::combat::check::{Answer, check};
use glasnost_sim::combat::damage::{self, Hit, HitResult};
use glasnost_sim::combat::data::WarheadId;
use glasnost_sim::econ::shroud;
use glasnost_sim::psychic;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::superweapons::SwEvent;
use glasnost_sim::world::{HouseId, ObjectId, Pos, Target, World};
use glasnost_sim::{save, testkit};

const SIZE: i32 = 24;
const ALPHA: u8 = 0;
const BETA: u8 = 1;
/// The neutral house of `testkit::game_with_non_players`.
const NEUTRAL: u8 = 2;

fn dec(s: &str) -> Decimal {
    Decimal(s.into())
}

fn every(v: &str) -> Option<Vec<Decimal>> {
    Some(vec![dec(v); 11])
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
    f(&mut t);
    register(r, "vehicles", name);
    r.sim.vehicles.insert(name.into(), t);
}

fn infantry(r: &mut Ruleset, name: &str, f: impl FnOnce(&mut UnitType)) {
    let mut t = r.sim.infantry["WALKER"].clone();
    t.strength = Some(1000);
    t.primary = None;
    f(&mut t);
    register(r, "infantry", name);
    r.sim.infantry.insert(name.into(), t);
}

fn building(r: &mut Ruleset, name: &str, f: impl FnOnce(&mut BuildingType)) {
    let mut t = BuildingType {
        strength: Some(1000),
        foundation: Some([1, 1]),
        ..Default::default()
    };
    f(&mut t);
    register(r, "buildings", name);
    r.sim.buildings.insert(name.into(), t);
}

/// The combat ruleset with:
/// - warheads MAD (berserk-causing, versus 0.5), PSI (psychic damage,
///   versus 1, spread 2 cells), WIDE (berserk-causing, versus 0.5, spread
///   3 cells);
/// - single-target weapons MadRay (MAD) and PsiRay (PSI), damage 100,
///   range 10;
/// - vehicles BOX (plain), STOIC (psionics-immune), DRONE (psionics-immune,
///   berserk-friendly), PAL (berserk-friendly only), RAYCAR (MadRay),
///   PSICAR (PsiRay), TANK (Gun);
/// - infantry GRUNT (plain), CALM (psychic-weapon-immune);
/// - buildings HUT (both immunities omitted), HUTON (both explicitly on),
///   HUTOFF (both explicitly off).
fn rules() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    r.sim.warheads.insert(
        "MAD".into(),
        WarheadType {
            verses: every("0.5"),
            psychedelic: Some(true),
            ..Default::default()
        },
    );
    r.sim.warheads.insert(
        "PSI".into(),
        WarheadType {
            verses: every("1"),
            psychic_damage: Some(true),
            cell_spread: Some(dec("2")),
            percent_at_max: Some(dec("1")),
            ..Default::default()
        },
    );
    r.sim.warheads.insert(
        "WIDE".into(),
        WarheadType {
            verses: every("0.5"),
            psychedelic: Some(true),
            cell_spread: Some(dec("3")),
            percent_at_max: Some(dec("0.25")),
            ..Default::default()
        },
    );
    for (name, wh) in [("MadRay", "MAD"), ("PsiRay", "PSI")] {
        r.sim.weapons.insert(
            name.into(),
            WeaponType {
                damage: Some(100),
                rof: Some(30),
                range: Some(dec("10")),
                speed: Some(100),
                projectile: Some("Bullet".into()),
                warhead: Some(wh.into()),
                ..Default::default()
            },
        );
    }
    vehicle(&mut r, "BOX", |_| {});
    vehicle(&mut r, "STOIC", |t| {
        t.combat.immune_to_psionics = Some(true)
    });
    vehicle(&mut r, "DRONE", |t| {
        t.combat.immune_to_psionics = Some(true);
        t.combat.berserk_friendly = Some(true);
    });
    vehicle(&mut r, "PAL", |t| t.combat.berserk_friendly = Some(true));
    vehicle(&mut r, "RAYCAR", |t| t.primary = Some("MadRay".into()));
    vehicle(&mut r, "PSICAR", |t| t.primary = Some("PsiRay".into()));
    vehicle(&mut r, "TANK", |t| t.primary = Some("Gun".into()));
    infantry(&mut r, "GRUNT", |_| {});
    infantry(&mut r, "CALM", |t| {
        t.combat.immune_to_psionic_weapons = Some(true);
    });
    building(&mut r, "HUT", |_| {});
    building(&mut r, "HUTON", |b| {
        b.combat.immune_to_psionics = Some(true);
        b.combat.immune_to_psionic_weapons = Some(true);
    });
    building(&mut r, "HUTOFF", |b| {
        b.combat.immune_to_psionics = Some(false);
        b.combat.immune_to_psionic_weapons = Some(false);
    });
    r
}

fn at(dx: i32, dy: i32) -> CellXY {
    testkit::mid(SIZE, dx, dy)
}

fn put(w: &mut World, kind: Kind, name: &str, house: u8, dx: i32, dy: i32) -> ObjectId {
    testkit::spawn(w, kind, name, house, at(dx, dy), 0)
}

fn wh(w: &World, name: &str) -> WarheadId {
    w.rules()
        .combat
        .warheads
        .iter()
        .position(|h| h.name == name)
        .unwrap() as u32
}

/// A direct berserk hit of `amount` from house `house` (MAD, distance 0).
fn mad(w: &World, amount: i32, house: Option<u8>) -> Hit {
    Hit {
        amount,
        warhead: Some(wh(w, "MAD")),
        distance: 0,
        attacker: None,
        house: house.map(HouseId),
        ignore_defences: false,
        no_escape: false,
    }
}

fn health(w: &World, id: ObjectId) -> i32 {
    w.obj(id).map_or(0, |o| o.health)
}

fn left(w: &World, id: ObjectId) -> Option<i32> {
    w.obj(id).unwrap().combat.berserk
}

fn aim(w: &mut World, a: ObjectId, t: Target) -> Answer {
    w.obj_mut(a).unwrap().target = Some(t);
    check(w, a, Some(t), 0, false)
}

// ---------------------------------------------------------------------------
// Claim 1: the two immunities.

// spec: S-0610 claim 1 (both immunities absent: yes for building types, no for infantry, vehicle and aircraft types; an explicit value wins for either kind; scenario P8)
#[test]
fn claim_1_defaults_by_kind() {
    let w = testkit::game_with(&rules(), SIZE, 1);
    let r = w.rules();
    let b = |n: &str| &r.buildings[r.index_of(Kind::Building, n).unwrap() as usize].combat;
    let v = |n: &str| {
        &r.unit(Kind::Vehicle, r.index_of(Kind::Vehicle, n).unwrap())
            .unwrap()
            .combat
    };
    assert!(b("HUT").immune_psionics && b("HUT").immune_psionic_weapons);
    assert!(b("HUTON").immune_psionics && b("HUTON").immune_psionic_weapons);
    assert!(!b("HUTOFF").immune_psionics && !b("HUTOFF").immune_psionic_weapons);
    assert!(!v("BOX").immune_psionics && !v("BOX").immune_psionic_weapons);
    assert!(v("STOIC").immune_psionics && !v("STOIC").immune_psionic_weapons);
}

// spec: S-0610 claim 1 and scenario P8 c (a psychic-damage weapon may fire at every building variant and damages only the explicit-off one; psychic-weapon immunity adds no firing refusal)
#[test]
fn claim_1_psychic_damage_weapon_fires_at_immune_buildings() {
    let mut w = testkit::game_with(&rules(), SIZE, 1);
    let car = put(&mut w, Kind::Vehicle, "PSICAR", ALPHA, -2, 0);
    for (name, hurt) in [("HUT", false), ("HUTON", false), ("HUTOFF", true)] {
        let hut = put(&mut w, Kind::Building, name, BETA, 2, 3);
        assert_ne!(
            aim(&mut w, car, Target::Object(hut)),
            Answer::Illegal,
            "{name}"
        );
        let h = Hit {
            amount: 100,
            warhead: Some(wh(&w, "PSI")),
            distance: 0,
            attacker: Some(car),
            house: Some(HouseId(ALPHA)),
            ignore_defences: false,
            no_escape: false,
        };
        let before = health(&w, hut);
        damage::apply(&mut w, hut, h);
        assert_eq!(health(&w, hut) < before, hurt, "{name}");
        glasnost_sim::objects::remove(&mut w, hut);
    }
}

// ---------------------------------------------------------------------------
// Claim 2: psychic pulses are ordinary area damage.

// spec: S-0610 claim 2 and scenario P1 (a psychic pulse spreads as ordinary area damage: the vehicle and the plain infantry are damaged, the building and the psychic-weapon-immune infantry are not)
#[test]
fn claim_2_pulse_reaches_everyone_and_immunity_ignores_it() {
    let mut w = testkit::game_with(&rules(), SIZE, 1);
    let hut = put(&mut w, Kind::Building, "HUT", BETA, 1, 0);
    let boxv = put(&mut w, Kind::Vehicle, "BOX", BETA, 0, 1);
    let grunt = put(&mut w, Kind::Infantry, "GRUNT", BETA, -1, 0);
    let calm = put(&mut w, Kind::Infantry, "CALM", BETA, 0, -1);
    let before: Vec<i32> = [hut, boxv, grunt, calm]
        .iter()
        .map(|&i| health(&w, i))
        .collect();
    let point = Pos::centre(at(0, 0), 0);
    let psi = wh(&w, "PSI");
    area::detonate(
        &mut w,
        Detonation {
            point,
            damage: 100,
            warhead: Some(psi),
            source: None,
            house: Some(HouseId(ALPHA)),
            resources: true,
        },
    );
    let after: Vec<i32> = [hut, boxv, grunt, calm]
        .iter()
        .map(|&i| health(&w, i))
        .collect();
    assert_eq!(after[0], before[0], "the building ignores the hit");
    assert!(after[1] < before[1], "the vehicle is damaged");
    assert!(after[2] < before[2], "the plain infantry is damaged");
    assert_eq!(after[3], before[3], "the immune infantry ignores the hit");
}

// ---------------------------------------------------------------------------
// Claim 3: firing refusals for berserk weapons.

// spec: S-0610 claim 3 and scenario P8 c (a berserk weapon is illegal against a psionics-immune target: omitted and explicit-on buildings and an immune vehicle; it may fire at the explicit-off building and a plain vehicle; a cell target is not refused)
#[test]
fn claim_3_berserk_weapon_refusals() {
    let mut w = testkit::game_with(&rules(), SIZE, 1);
    let car = put(&mut w, Kind::Vehicle, "RAYCAR", ALPHA, -2, 0);
    let hut = put(&mut w, Kind::Building, "HUT", BETA, 2, 3);
    let huton = put(&mut w, Kind::Building, "HUTON", BETA, 2, -3);
    let hutoff = put(&mut w, Kind::Building, "HUTOFF", BETA, 4, 3);
    let stoic = put(&mut w, Kind::Vehicle, "STOIC", BETA, 2, 0);
    let boxv = put(&mut w, Kind::Vehicle, "BOX", BETA, 2, 1);
    assert_eq!(aim(&mut w, car, Target::Object(hut)), Answer::Illegal);
    assert_eq!(aim(&mut w, car, Target::Object(huton)), Answer::Illegal);
    assert_eq!(aim(&mut w, car, Target::Object(stoic)), Answer::Illegal);
    assert_ne!(aim(&mut w, car, Target::Object(hutoff)), Answer::Illegal);
    assert_ne!(aim(&mut w, car, Target::Object(boxv)), Answer::Illegal);
    assert_ne!(aim(&mut w, car, Target::Cell(at(2, 0))), Answer::Illegal);
    // The explicit-off building still cannot go berserk (S-0302 claim 10's building exception).
    let h = mad(&w, 100, Some(ALPHA));
    assert_eq!(damage::apply(&mut w, hutoff, h), HitResult::Unaffected);
    assert_eq!(left(&w, hutoff), None);
}

// ---------------------------------------------------------------------------
// Claims 4 and 5: entry, countdown and expiry.

/// A Beta TANK attacking a durable Alpha BOX.
fn attacker(w: &mut World, house: u8) -> (ObjectId, ObjectId) {
    let tank = put(w, Kind::Vehicle, "TANK", house, 0, 0);
    let foe = put(w, Kind::Vehicle, "BOX", ALPHA, 3, 0);
    let o = w.obj_mut(tank).unwrap();
    o.target = Some(Target::Object(foe));
    o.beh.current = Some(Behaviour::Attack);
    o.beh.queued = Some(Behaviour::Move);
    (tank, foe)
}

// spec: S-0610 claim 4 and scenario P2 (entry: the remaining time becomes T, the target is dropped, hunt is queued replacing what was queued, the current behaviour stays; a re-hit only sets the new T, not the sum, with no target clear or behaviour change)
#[test]
fn claim_4_entry_and_re_hit() {
    let mut w = testkit::game_with(&rules(), SIZE, 1);
    let (tank, foe) = attacker(&mut w, BETA);
    let h = mad(&w, 100, Some(ALPHA));
    assert_eq!(damage::apply(&mut w, tank, h), HitResult::Unchanged);
    let o = w.obj(tank).unwrap();
    assert_eq!(o.combat.berserk, Some(50));
    assert_eq!(o.target, None);
    assert_eq!(o.beh.queued, Some(Behaviour::Hunt));
    assert_eq!(o.beh.current, Some(Behaviour::Attack));
    // A re-hit while berserk: the new T replaces the remaining time; target and behaviour stay.
    let o = w.obj_mut(tank).unwrap();
    o.target = Some(Target::Object(foe));
    o.beh.queued = Some(Behaviour::Move);
    let h = mad(&w, 40, Some(ALPHA));
    damage::apply(&mut w, tank, h);
    let o = w.obj(tank).unwrap();
    assert_eq!(o.combat.berserk, Some(20), "the new T, not 50 + 20");
    assert_eq!(o.target, Some(Target::Object(foe)));
    assert_eq!(o.beh.queued, Some(Behaviour::Move));
}

// spec: S-0610 claim 5 and F2 (expiry on the T-th countdown update after the hit: the state ends, the target is dropped and guard is queued for a human-controlled owner, an AI house included)
#[test]
fn claim_5_countdown_and_expiry_for_a_player_house() {
    let mut w = testkit::game_with(&rules(), SIZE, 1);
    let (tank, foe) = attacker(&mut w, BETA);
    assert!(psychic::enter(&mut w, tank, 3));
    let o = w.obj_mut(tank).unwrap();
    o.target = Some(Target::Object(foe));
    psychic::countdown_turn(&mut w, tank);
    psychic::countdown_turn(&mut w, tank);
    assert_eq!(left(&w, tank), Some(1));
    assert!(psychic::berserk(&w, tank));
    psychic::countdown_turn(&mut w, tank);
    let o = w.obj(tank).unwrap();
    assert_eq!(o.combat.berserk, None, "ends on the third update");
    assert_eq!(o.target, None);
    assert_eq!(o.beh.queued, Some(Behaviour::Guard));
}

// spec: S-0610 claim 5 (a unit of the neutral house queues hunt at expiry)
#[test]
fn claim_5_expiry_for_the_neutral_house_queues_hunt() {
    let mut w = testkit::game_with_non_players(&rules(), SIZE, 1);
    let (tank, _) = attacker(&mut w, NEUTRAL);
    assert!(psychic::enter(&mut w, tank, 1));
    w.obj_mut(tank).unwrap().beh.queued = None;
    psychic::countdown_turn(&mut w, tank);
    let o = w.obj(tank).unwrap();
    assert_eq!(o.combat.berserk, None);
    assert_eq!(o.beh.queued, Some(Behaviour::Hunt));
}

// spec: S-0610 claim 5 (the countdown runs in the unit's own updates: a berserk unit with T = 4 is no longer berserk after four ticks, and takes orders again; S-0214 claim 15)
#[test]
fn claim_5_countdown_runs_in_the_turn() {
    let mut w = testkit::game_with(&rules(), SIZE, 1);
    let tank = put(&mut w, Kind::Vehicle, "BOX", BETA, 0, 0);
    assert!(psychic::enter(&mut w, tank, 4));
    let mods = glasnost_sim::targeting::Modifiers::default();
    let click = |w: &World| {
        glasnost_sim::targeting::click(
            w,
            HouseId(BETA),
            &[tank],
            glasnost_sim::targeting::ClickOn::Cell(at(-3, 0)),
            mods,
        )
        .orders
    };
    assert!(click(&w).is_empty(), "a berserk unit takes no orders");
    testkit::run(&mut w, 3);
    assert!(psychic::berserk(&w, tank));
    testkit::run(&mut w, 1);
    assert!(!psychic::berserk(&w, tank));
    assert!(!click(&w).is_empty(), "orders are accepted again");
}

// spec: S-0610 Edge cases (quirks.berserk_zero_duration on: a hit with T ≤ 0 drops the target and queues hunt and the state ends at the next countdown update with guard queued; off: no effect)
#[test]
fn zero_duration_quirk_both_positions() {
    for on in [true, false] {
        let mut r = rules();
        r.globals.quirks.berserk_zero_duration = on;
        let mut w = testkit::game_with(&r, SIZE, 1);
        let (tank, foe) = attacker(&mut w, BETA);
        assert_eq!(psychic::enter(&mut w, tank, 0), on);
        let o = w.obj(tank).unwrap();
        if on {
            assert_eq!(o.combat.berserk, Some(0));
            assert_eq!(o.target, None);
            assert_eq!(o.beh.queued, Some(Behaviour::Hunt));
            psychic::countdown_turn(&mut w, tank);
            let o = w.obj(tank).unwrap();
            assert_eq!(o.combat.berserk, None);
            assert_eq!(o.beh.queued, Some(Behaviour::Guard));
        } else {
            assert_eq!(o.combat.berserk, None);
            assert_eq!(o.target, Some(Target::Object(foe)));
            assert_eq!(o.beh.queued, Some(Behaviour::Move));
        }
    }
}

// spec: S-0610 Edge cases (quirks.berserk_zero_duration off: a T ≤ 0 hit leaves a unit that is already berserk unchanged)
#[test]
fn zero_duration_off_does_not_reset_a_berserk_unit() {
    let mut r = rules();
    r.globals.quirks.berserk_zero_duration = false;
    let mut w = testkit::game_with(&r, SIZE, 1);
    let tank = put(&mut w, Kind::Vehicle, "BOX", BETA, 0, 0);
    assert!(psychic::enter(&mut w, tank, 9));
    assert!(!psychic::enter(&mut w, tank, -2));
    assert_eq!(left(&w, tank), Some(9));
}

// ---------------------------------------------------------------------------
// Claim 6: while berserk.

// spec: S-0610 claim 6 (a berserk firer gets illegal against a berserk-friendly type; not berserk it may fire)
#[test]
fn claim_6_berserk_friendly_refusal() {
    let mut w = testkit::game_with(&rules(), SIZE, 1);
    let tank = put(&mut w, Kind::Vehicle, "TANK", ALPHA, -2, 0);
    let pal = put(&mut w, Kind::Vehicle, "PAL", BETA, 2, 0);
    assert_ne!(aim(&mut w, tank, Target::Object(pal)), Answer::Illegal);
    psychic::enter(&mut w, tank, 1000);
    assert_eq!(aim(&mut w, tank, Target::Object(pal)), Answer::Illegal);
}

// spec: S-0610 claim 6 and scenario P4 a (a berserk tank of house A beside a berserk-friendly drone and an infantry of house A attacks the infantry, never the drone)
#[test]
fn claim_6_berserk_unit_attacks_its_own_infantry_not_the_drone() {
    let mut w = testkit::game_with(&rules(), SIZE, 1);
    let tank = put(&mut w, Kind::Vehicle, "TANK", ALPHA, 0, 0);
    let drone = put(&mut w, Kind::Vehicle, "DRONE", ALPHA, 2, 0);
    let grunt = put(&mut w, Kind::Infantry, "GRUNT", ALPHA, 0, 3);
    testkit::run(&mut w, 2);
    psychic::enter(&mut w, tank, 10_000);
    let mut hit_grunt = false;
    for _ in 0..200 {
        testkit::run(&mut w, 1);
        let t = w.obj(tank).unwrap().target;
        assert_ne!(t, Some(Target::Object(drone)), "never the drone");
        hit_grunt |= t == Some(Target::Object(grunt));
    }
    assert!(hit_grunt, "it attacks its own infantry");
}

// ---------------------------------------------------------------------------
// M-0011 (CAOS against BFRT watched per unit): the parts of the record's
// chain that the approved specs give. How a berserk crusher comes to hold a
// berserk-friendly drone as its target is Q-2005.

/// The rules with MITE: a crushable, berserk-friendly, psionics-immune
/// vehicle with no weapon (an invented stand-in for the record's drones).
fn mite_rules() -> Ruleset {
    let mut r = rules();
    vehicle(&mut r, "MITE", |t| {
        t.crushable = Some(true);
        t.combat.immune_to_psionics = Some(true);
        t.combat.berserk_friendly = Some(true);
    });
    r
}

// spec: S-0302 claim 10 and S-0610 claim 4 (a further berserk hit sets the remaining time to its own T, longer or shorter; target and behaviour stay)
// measure: M-0011 (Gas on the fortresses: each further hit sets the remaining time to its own amount, one shortening it and two lengthening it)
#[test]
fn m0011_a_further_berserk_hit_sets_the_time_anew_longer_or_shorter() {
    let mut w = testkit::game_with(&rules(), SIZE, 1);
    let tank = put(&mut w, Kind::Vehicle, "TANK", BETA, 0, 0);
    let h = mad(&w, 100, Some(ALPHA));
    damage::apply(&mut w, tank, h);
    let first = left(&w, tank).expect("berserk");
    testkit::run(&mut w, 3);
    assert_eq!(left(&w, tank), Some(first - 3), "one countdown step a tick");
    // Longer: the new T replaces the remaining time, it is not added.
    let h = mad(&w, 100, Some(ALPHA));
    damage::apply(&mut w, tank, h);
    assert_eq!(left(&w, tank), Some(first));
    // Shorter.
    let h = mad(&w, 20, Some(ALPHA));
    damage::apply(&mut w, tank, h);
    assert!(left(&w, tank).is_some_and(|t| t < first - 3));
    let short = left(&w, tank).unwrap();
    // Longer again.
    let h = mad(&w, 60, Some(ALPHA));
    damage::apply(&mut w, tank, h);
    assert!(left(&w, tank).is_some_and(|t| t > short));
    // A berserk hit takes no health.
    assert_eq!(health(&w, tank), 1000);
}

// spec: S-0610 claims 6 and 8 and S-0309 table C row "firing check" (a berserk unit's search never takes a berserk-friendly object, here the only enemy beside it)
// measure: M-0011 (What the berserk fortresses do: their fire goes at each other)
#[test]
fn m0011_a_berserk_crusher_never_searches_out_a_berserk_friendly_enemy() {
    let mut w = testkit::game_with(&mite_rules(), SIZE, 1);
    w.houses[usize::from(ALPHA)].human = false;
    let crusher = put(&mut w, Kind::Vehicle, "CRUSHER", ALPHA, 0, 0);
    let mite = put(&mut w, Kind::Vehicle, "MITE", BETA, 2, 0);
    psychic::enter(&mut w, crusher, 10_000);
    w.obj_mut(crusher).unwrap().beh.current = Some(Behaviour::Hunt);
    for _ in 0..200 {
        testkit::run(&mut w, 1);
        assert_ne!(
            w.obj(crusher).unwrap().target,
            Some(Target::Object(mite)),
            "never the berserk-friendly enemy"
        );
    }
    assert!(w.obj(mite).is_some(), "not crushed");
}

// spec: S-0210 claim 11 (the attack-target variant of a computer-controlled owner: a crushable target less than the crush range away and no move destination; nothing in it asks the firing check or the berserk state)
// spec: S-0210 claims 3 and 4 (crushed at the cell centre: removed on the same tick, no damage, the crush sound)
// spec: S-0610 claims 6 and 8 (the berserk crusher never fires at the berserk-friendly target it holds)
// spec: S-0309 claim 27 (a crusher that drives over the object it holds drops it in its movement and makes no shot in that tick's firing step)
// spec: S-0309 claim 29 (the illegal held target is dropped at a check tick in hunt too, so the hold starts just after one; the drop changes no destination, claim 29 The drop)
// measure: M-0011 (How the drones leave play: no damage event, while a grounded fortress that had the drone as its target stood on its position)
#[test]
fn m0011_a_berserk_crusher_holding_a_berserk_friendly_target_drives_over_it() {
    use glasnost_sim::combat::effects::{Effect, Sound};
    let mut w = testkit::game_with(&mite_rules(), SIZE, 1);
    // A test-only computer-controlled player house, as the arena's.
    w.houses[usize::from(ALPHA)].human = false;
    // The held mite is illegal for the berserk crusher (S-0610 claim 6),
    // so its first check tick drops it in *hunt* too (S-0309 claim 29
    // r94, Behaviours). The pair is placed just after a check tick, so
    // that the crusher's first processing, in its first turn, takes the
    // held mite as its destination (the crushing approach) before that
    // drop; the drop leaves the destination, and the crusher drives over
    // the mite.
    while w.tick % 16 != 1 {
        testkit::run(&mut w, 1);
    }
    let crusher = put(&mut w, Kind::Vehicle, "CRUSHER", ALPHA, 0, 0);
    let mite = put(&mut w, Kind::Vehicle, "MITE", BETA, 1, 0);
    psychic::enter(&mut w, crusher, 10_000);
    let o = w.obj_mut(crusher).unwrap();
    o.beh.current = Some(Behaviour::Hunt);
    o.beh.queued = None;
    o.target = Some(Target::Object(mite));
    let mut shots = 0;
    let mut crushed = false;
    for _ in 0..200 {
        let before = health(&w, mite);
        testkit::run(&mut w, 1);
        let gone = w.obj(mite).is_none();
        for e in std::mem::take(&mut w.effects) {
            match e {
                // The crush tick's shot counts too: the crusher drops the
                // mite inside the crush and fires nothing in that tick's
                // firing step (S-0309 claim 27, Q-2006).
                Effect::Shot { firer, .. } if firer == crusher => shots += 1,
                Effect::Sound {
                    sound: Sound::Crush,
                    ..
                } => crushed = true,
                _ => {}
            }
        }
        if gone {
            assert_eq!(before, 1000, "no damage before it leaves play");
            break;
        }
    }
    assert!(w.obj(mite).is_none(), "crushed");
    assert!(crushed, "the crush sound");
    assert_eq!(shots, 0, "never fired at");
    assert!(psychic::berserk(&w, crusher), "still berserk");
}

// ---------------------------------------------------------------------------
// Claim 8: area berserk weapons.

// spec: S-0610 claim 8 and scenario P5 (every berserk hit of an area pulse takes T at distance 0: equal tanks at different distances get the same initial T; the psionics-immune, berserk-friendly drone cannot go berserk from it)
#[test]
fn claim_8_area_pulse_gives_equal_durations() {
    let mut w = testkit::game_with(&rules(), SIZE, 1);
    let drone = put(&mut w, Kind::Vehicle, "DRONE", ALPHA, 0, 0);
    let near = put(&mut w, Kind::Vehicle, "BOX", BETA, 1, 0);
    let far = put(&mut w, Kind::Vehicle, "BOX", BETA, 0, 2);
    let enemy_drone = put(&mut w, Kind::Vehicle, "DRONE", BETA, -1, 0);
    let wide = wh(&w, "WIDE");
    let point = w.obj(drone).unwrap().pos;
    area::detonate(
        &mut w,
        Detonation {
            point,
            damage: 100,
            warhead: Some(wide),
            source: Some(drone),
            house: Some(HouseId(ALPHA)),
            resources: true,
        },
    );
    assert_eq!(left(&w, near), Some(50));
    assert_eq!(left(&w, far), Some(50));
    assert_eq!(left(&w, drone), None);
    assert_eq!(left(&w, enemy_drone), None);
}

// ---------------------------------------------------------------------------
// Claims 9 to 12: the psychic sensor.

/// Rules with a sensor building EYE of radius 4 that needs power, a power
/// plant PLANT and a drain HOG.
fn sensor_rules() -> Ruleset {
    let mut r = rules();
    building(&mut r, "EYE", |b| {
        b.econ.psychic_detection_radius = Some(4);
        b.econ.has_radial_indicator = Some(true);
    });
    building(&mut r, "BLIND", |b| {
        b.econ.has_radial_indicator = Some(true);
    });
    r
}

fn sensor_world() -> (World, ObjectId) {
    let mut w = testkit::game_with(&sensor_rules(), SIZE, 1);
    let eye = put(&mut w, Kind::Building, "EYE", ALPHA, 0, 0);
    (w, eye)
}

// spec: S-0610 claim 9 and scenario P6 (a placed sensor is its owner's; captured it moves to the new owner at once; switched off it is not in operation and does not count)
#[test]
fn claim_9_membership_capture_and_operation() {
    let (mut w, eye) = sensor_world();
    assert_eq!(psychic::sensors(&w, HouseId(ALPHA)), vec![eye]);
    assert!(psychic::sensors(&w, HouseId(BETA)).is_empty());
    glasnost_sim::structures::ownership::change(
        &mut w,
        eye,
        Some(HouseId(BETA)),
        glasnost_sim::structures::ownership::Cause::Trigger,
    );
    assert!(psychic::sensors(&w, HouseId(ALPHA)).is_empty());
    assert_eq!(psychic::sensors(&w, HouseId(BETA)), vec![eye]);
    w.obj_mut(eye).unwrap().econ.building.switched_off = true;
    assert!(psychic::sensors(&w, HouseId(BETA)).is_empty());
}

// spec: S-0610 claim 10 and scenario P6 (lines for an enemy moving into reach and an enemy attacking an object in reach; none for a move outside reach, for an allied unit or with the sensor out of operation)
#[test]
fn claim_10_detection_conditions() {
    let (mut w, eye) = sensor_world();
    let mover = put(&mut w, Kind::Vehicle, "BOX", BETA, -8, 0);
    w.obj_mut(mover).unwrap().dest = Some(Target::Cell(at(2, 0)));
    let striker = put(&mut w, Kind::Vehicle, "TANK", BETA, 8, 0);
    let mine = put(&mut w, Kind::Vehicle, "BOX", ALPHA, 1, 1);
    w.obj_mut(striker).unwrap().target = Some(Target::Object(mine));
    let away = put(&mut w, Kind::Vehicle, "BOX", BETA, 0, 2);
    w.obj_mut(away).unwrap().dest = Some(Target::Cell(at(9, 9)));
    let lines = psychic::intent_lines(&w, HouseId(ALPHA));
    let units: Vec<ObjectId> = lines.iter().map(|l| l.unit).collect();
    assert_eq!(
        units,
        vec![mover, striker],
        "the unit's own position plays no part"
    );
    assert_eq!(lines[0].owner, Some(HouseId(BETA)));
    // Alpha allies Beta: no lines (the viewer's side of the alliance decides).
    sw_common::ally(&mut w, HouseId(ALPHA), HouseId(BETA));
    assert!(psychic::intent_lines(&w, HouseId(ALPHA)).is_empty());
    w.houses[usize::from(ALPHA)]
        .allies
        .retain(|&h| h != HouseId(BETA));
    // Beta allying Alpha does not suppress Alpha's lines.
    sw_common::ally(&mut w, HouseId(BETA), HouseId(ALPHA));
    assert_eq!(psychic::intent_lines(&w, HouseId(ALPHA)).len(), 2);
    w.obj_mut(eye).unwrap().econ.building.switched_off = true;
    assert!(psychic::intent_lines(&w, HouseId(ALPHA)).is_empty());
}

// spec: S-0610 claim 10 and scenario P6 e (one line per unit and a target takes precedence: a destination in reach makes the unit eligible while its line points to a target outside reach)
#[test]
fn claim_10_target_precedence() {
    let (mut w, _) = sensor_world();
    let tank = put(&mut w, Kind::Vehicle, "TANK", BETA, -8, 0);
    let far = put(&mut w, Kind::Vehicle, "BOX", ALPHA, -8, 8);
    let o = w.obj_mut(tank).unwrap();
    o.target = Some(Target::Object(far));
    o.dest = Some(Target::Cell(at(1, 0)));
    let lines = psychic::intent_lines(&w, HouseId(ALPHA));
    assert_eq!(lines.len(), 1);
    assert!(lines[0].to_target);
    assert_eq!(lines[0].to, w.reference_point(far));
}

// spec: S-0610 F1 and claim 10 (within reach at exactly 256 × R leptons; height counts; with the exact root (quirks.table_root_distance off) one lepton more is out of reach; the table root's coarser steps are S-0204 claim 5's, so it is checked with a margin)
#[test]
fn f1_reach_boundary() {
    for table in [false, true] {
        let mut r = sensor_rules();
        r.globals.quirks.table_root_distance = table;
        let mut w = testkit::game_with(&r, SIZE, 1);
        let eye = put(&mut w, Kind::Building, "EYE", ALPHA, 0, 0);
        let c = w.reference_point(eye);
        let r = 4 * 256;
        let out = if table { 64 } else { 1 };
        assert!(
            psychic::within_reach(&w, eye, Pos::new(c.x + r, c.y, c.z)),
            "{table}"
        );
        assert!(
            !psychic::within_reach(&w, eye, Pos::new(c.x + r + out, c.y, c.z)),
            "{table}"
        );
        // Height counts: 1024 east and 300 up is beyond reach; 900 east and 300 up is within.
        assert!(
            !psychic::within_reach(&w, eye, Pos::new(c.x + r, c.y, c.z + 300)),
            "{table}"
        );
        assert!(
            psychic::within_reach(&w, eye, Pos::new(c.x + 900, c.y, c.z + 300)),
            "{table}"
        );
    }
}

// spec: S-0610 claim 10 (a building target is measured to its reference point: a large building whose centre is outside reach does not make the unit eligible although its foundation edge is inside)
#[test]
fn f1_building_size_does_not_extend_reach() {
    let mut r = sensor_rules();
    building(&mut r, "BIG", |b| b.foundation = Some([3, 3]));
    let mut w = testkit::game_with(&r, SIZE, 1);
    let _eye = put(&mut w, Kind::Building, "EYE", ALPHA, 0, 0);
    // BIG's top-left cell is 4 east: its edge is at 4 cells, its reference point 5 cells east.
    let big = put(&mut w, Kind::Building, "BIG", ALPHA, 4, -1);
    let tank = put(&mut w, Kind::Vehicle, "TANK", BETA, 9, 0);
    w.obj_mut(tank).unwrap().target = Some(Target::Object(big));
    assert!(psychic::intent_lines(&w, HouseId(ALPHA)).is_empty());
}

// spec: S-0610 Edge cases (quirks.psychic_intent_ignores_visibility on: a line for a unit on a cell the viewer does not see; off: none)
#[test]
fn visibility_quirk_both_positions() {
    for on in [true, false] {
        let mut r = sensor_rules();
        r.globals.quirks.psychic_intent_ignores_visibility = on;
        let mut w = testkit::game_with(&r, SIZE, 1);
        let _eye = put(&mut w, Kind::Building, "EYE", ALPHA, 0, 0);
        let hidden = put(&mut w, Kind::Vehicle, "BOX", BETA, -10, -10);
        w.obj_mut(hidden).unwrap().dest = Some(Target::Cell(at(1, 1)));
        assert!(!shroud::seen(&w, HouseId(ALPHA), at(-10, -10)));
        assert_eq!(
            psychic::intent_lines(&w, HouseId(ALPHA)).len(),
            usize::from(on),
            "{on}"
        );
    }
}

// spec: S-0610 claim 12 (a sensor whose type shows a ring shows its psychic-detection radius; a type with radius 0 gives no psychic ring)
#[test]
fn claim_12_range_ring() {
    let (mut w, eye) = sensor_world();
    let blind = put(&mut w, Kind::Building, "BLIND", ALPHA, 4, 4);
    assert_eq!(psychic::range_ring(&w, eye), Some(4));
    assert_eq!(psychic::range_ring(&w, blind), None);
}

// ---------------------------------------------------------------------------
// Claims 13 to 15: the psychic reveal.

const REVEAL: u32 = 12;

fn reveal_rules(radius: Option<i32>) -> Ruleset {
    let mut r = sw_common::ruleset();
    r.sim.superweapons.insert(
        "REVEAL".into(),
        SuperweaponType {
            kind: Some("PsychicReveal".into()),
            action: Some("PsychicReveal".into()),
            recharge_time: Some(dec("0.1")),
            ..Default::default()
        },
    );
    register(&mut r, "superweapons", "REVEAL");
    building(&mut r, "MINDB", |b| {
        b.econ.super_weapon = Some("REVEAL".into())
    });
    r.sim.general.superweapons.psychic_reveal_radius = radius;
    r.sim.general.superweapons.psychic_reveal_activate_sound = Some("Whisper".into());
    r
}

fn launched(radius: Option<i32>, target: CellXY, block: Option<CellXY>) -> World {
    use sw_common::*;
    let mut w = game(&reveal_rules(radius));
    if let Some(b) = block {
        w.cells.grid.get_mut(b).unwrap().level = 8;
    }
    spawn(&mut w, "MINDB", ALPHA, -8, -8);
    ready_tick(&mut w, ALPHA, REVEAL, 200);
    w.effects.clear();
    fire_now(&mut w, ALPHA, REVEAL, target);
    w
}

// spec: S-0610 claims 13 and 14 (the launch spends the record and in the same tick reveals the default radius 3 around the target cell for the launching house only; the activation sound at the target's centre; no launch announcement and no radar event)
#[test]
fn claims_13_14_launch_and_area() {
    let t = sw_common::at(6, 6);
    let w = launched(None, t, None);
    assert!(!sw_common::rec(&w, sw_common::ALPHA, REVEAL).ready);
    let c = |dx, dy| CellXY::new(t.x + dx, t.y + dy);
    assert!(shroud::explored(&w, sw_common::ALPHA, c(3, 0)));
    assert!(shroud::seen(&w, sw_common::ALPHA, c(0, 3)));
    assert!(!shroud::explored(&w, sw_common::ALPHA, c(5, 0)));
    assert!(!shroud::explored(&w, sw_common::BETA, c(0, 0)));
    let ev = sw_common::events(&w);
    assert!(ev.contains(&(SwEvent::RevealSound, Some(sw_common::ALPHA), Some(REVEAL))));
    assert!(
        !ev.iter()
            .any(|e| matches!(e.0, SwEvent::Launch | SwEvent::Radar))
    );
    let pos = w.effects.iter().find_map(|e| match e {
        glasnost_sim::combat::effects::Effect::Superweapon {
            event: SwEvent::RevealSound,
            pos,
            ..
        } => *pos,
        _ => None,
    });
    assert_eq!(pos.map(|p| p.cell()), Some(t));
}

// spec: S-0610 claim 14 (high ground hides nothing: a cell an ordinary area reveal leaves out behind high ground is revealed; the radius comes from the rules)
#[test]
fn claim_14_no_high_ground_test() {
    let t = sw_common::at(4, 4);
    let c = |dx, dy| CellXY::new(t.x + dx, t.y + dy);
    // As in the S-0409 claim 7 test: T = centre + (3, 0) is blocked by centre + (4, 2).
    let mut plain = sw_common::game(&reveal_rules(Some(3)));
    plain.cells.grid.get_mut(c(4, 2)).unwrap().level = 8;
    shroud::area_reveal(&mut plain, sw_common::ALPHA, Pos::centre(t, 0), 3);
    assert!(
        !shroud::explored(&plain, sw_common::ALPHA, c(3, 0)),
        "an ordinary reveal is blocked"
    );
    let w = launched(Some(3), t, Some(c(4, 2)));
    assert!(shroud::explored(&w, sw_common::ALPHA, c(3, 0)));
    let wide = launched(Some(5), t, None);
    assert!(shroud::explored(&wide, sw_common::ALPHA, c(5, 0)));
}

// spec: S-0610 claim 15 (no duration: no lasting sight is added, so the revealed cells keep no sight count)
#[test]
fn claim_15_no_lasting_sight() {
    let t = sw_common::at(6, 6);
    let w = launched(None, t, None);
    assert_eq!(shroud::counts(&w, sw_common::ALPHA, t), Some((0, 0)));
}

// ---------------------------------------------------------------------------
// State and ordering: save and load.

// spec: S-0610 State and ordering (the berserk state and its remaining time are simulation state: the state hash survives a save and load round trip and differs from a world without it)
#[test]
fn berserk_state_survives_save_and_load() {
    let mut w = testkit::game_with(&rules(), SIZE, 1);
    let tank = put(&mut w, Kind::Vehicle, "BOX", BETA, 0, 0);
    let plain = save::state_hash(&w);
    psychic::enter(&mut w, tank, 7);
    let h = save::state_hash(&w);
    assert_ne!(h, plain);
    let l = save::load(&save::save(&w), w.rules_arc(), testkit::catalogue()).unwrap();
    assert_eq!(save::state_hash(&l), h);
    assert_eq!(l.obj(tank).unwrap().combat.berserk, Some(7));
}

// spec: S-0610 claim 14 (a radius of 0 reveals nothing; the reveal is shared with an ally of the launching house under S-0409 claim 2, ally reveal on by default)
#[test]
fn claim_14_zero_radius_and_allies() {
    use sw_common::*;
    let t = at(6, 6);
    let w = launched(Some(0), t, None);
    assert!(!shroud::explored(&w, ALPHA, t));
    let mut w = game(&reveal_rules(None));
    ally(&mut w, ALPHA, BETA);
    spawn(&mut w, "MINDB", ALPHA, -8, -8);
    ready_tick(&mut w, ALPHA, REVEAL, 200);
    fire_now(&mut w, ALPHA, REVEAL, t);
    assert!(shroud::explored(&w, ALPHA, t));
    assert!(shroud::explored(&w, BETA, t));
}

// spec: S-0610 claim 16 (the berserk-allowed flag is read but has no effect: a berserk hit takes effect and counts down the same with the flag on and off)
#[test]
fn claim_16_berserk_allowed_has_no_effect() {
    for allowed in [Some(true), Some(false), None] {
        let mut r = rules();
        r.sim.general.superweapons.berzerk_allowed = allowed;
        let mut w = testkit::game_with(&r, SIZE, 1);
        let tank = put(&mut w, Kind::Vehicle, "BOX", BETA, 0, 0);
        let h = mad(&w, 100, Some(ALPHA));
        assert_eq!(damage::apply(&mut w, tank, h), HitResult::Unchanged);
        assert_eq!(left(&w, tank), Some(50), "{allowed:?}");
        psychic::countdown_turn(&mut w, tank);
        assert_eq!(left(&w, tank), Some(49), "{allowed:?}");
    }
}

// spec: S-0610 claim 7 (a berserk vehicle or infantry has the tint at the berserk-colour position, 0 when absent; not berserk, no tint)
#[test]
fn claim_7_tint_query() {
    for (color, want) in [(None, 0), (Some(5), 5)] {
        let mut r = rules();
        r.sim.general.superweapons.berserk_color = color;
        let mut w = testkit::game_with(&r, SIZE, 1);
        let v = put(&mut w, Kind::Vehicle, "BOX", BETA, 0, 0);
        let i = put(&mut w, Kind::Infantry, "GRUNT", BETA, 2, 0);
        assert_eq!(psychic::tint(&w, v), None);
        psychic::enter(&mut w, v, 9);
        psychic::enter(&mut w, i, 9);
        assert_eq!(psychic::tint(&w, v), Some(want));
        assert_eq!(psychic::tint(&w, i), Some(want));
    }
}

// spec: S-0610 claim 14 (the grid centre includes S-0409 claim 5's height shift: on a raised target cell the reveal covers the cells of an area reveal at the cell's surface point, not those of the unshifted cell)
#[test]
fn claim_14_centre_has_the_height_shift() {
    let t = sw_common::at(6, 6);
    let explored = |w: &World| -> Vec<CellXY> {
        w.cells
            .grid
            .iter()
            .map(|(c, _)| c)
            .filter(|&c| shroud::explored(w, sw_common::ALPHA, c))
            .collect()
    };
    let raise = |w: &mut World| w.cells.grid.get_mut(t).unwrap().level = 8;
    let mut r = reveal_rules(None);
    r.sim.general.superweapons.psychic_reveal_radius = Some(3);
    let mut shifted = sw_common::game(&r);
    raise(&mut shifted);
    let p = glasnost_sim::superweapons::cell_point(&shifted, t, false);
    assert!(p.z > 0, "the raised cell has a surface height");
    let before = explored(&shifted);
    shroud::area_reveal(&mut shifted, sw_common::ALPHA, p, 3);
    let mut flat = sw_common::game(&r);
    raise(&mut flat);
    shroud::area_reveal(&mut flat, sw_common::ALPHA, Pos::centre(t, 0), 3);
    let mut w = sw_common::game(&r);
    raise(&mut w);
    assert_eq!(explored(&w), before);
    sw_common::spawn(&mut w, "MINDB", sw_common::ALPHA, -8, -8);
    sw_common::ready_tick(&mut w, sw_common::ALPHA, REVEAL, 200);
    let base = explored(&w);
    sw_common::fire_now(&mut w, sw_common::ALPHA, REVEAL, t);
    let got: Vec<CellXY> = explored(&w)
        .into_iter()
        .filter(|c| !base.contains(c))
        .collect();
    let want: Vec<CellXY> = explored(&shifted)
        .into_iter()
        .filter(|c| !before.contains(c) && !base.contains(c))
        .collect();
    let unshifted: Vec<CellXY> = explored(&flat)
        .into_iter()
        .filter(|c| !before.contains(c) && !base.contains(c))
        .collect();
    assert_eq!(got, want);
    assert_ne!(want, unshifted, "the shift moves the area");
}

// spec: S-0610 scenario P3 (zero duration: any target in an inert game takes a berserk hit with T = 0, S-0302 claim 6 and Formulas; with quirks.berserk_zero_duration on it is still dropped to hunt and the state ends at the next countdown update); the control on a map without the flag gives T = 50
#[test]
fn p3_berserk_hit_in_an_inert_game_has_zero_duration() {
    for inert in [false, true] {
        let mut w = testkit::game_with(&rules(), SIZE, 1);
        w.map_flags.inert = inert;
        let (tank, _) = attacker(&mut w, BETA);
        let h = mad(&w, 100, Some(ALPHA));
        assert_eq!(damage::apply(&mut w, tank, h), HitResult::Unchanged);
        let o = w.obj(tank).unwrap();
        assert_eq!(o.combat.berserk, Some(if inert { 0 } else { 50 }));
        assert_eq!(o.target, None, "dropped either way");
        assert_eq!(o.beh.queued, Some(Behaviour::Hunt));
        if inert {
            psychic::countdown_turn(&mut w, tank);
            assert_eq!(w.obj(tank).unwrap().combat.berserk, None);
        }
    }
}

/// `psychic::intent_lines` as before M8 C4 part 4: each unit's line asked
/// with [`psychic::intent_line`], which finds the sensors again for each.
fn intent_lines_one_by_one(w: &World, viewer: HouseId) -> Vec<psychic::IntentLine> {
    if psychic::sensors(w, viewer).is_empty() {
        return Vec::new();
    }
    w.objects
        .keys()
        .filter_map(|&id| psychic::intent_line(w, viewer, id))
        .collect()
}

// spec: S-0610 claims 9 to 11 and Edge cases (the lines with the sensors found once are the lines asked unit by unit: two sensors, one switched off and on, targets and destinations in and out of reach, own, allied and hidden units, both settings of quirks.psychic_intent_ignores_visibility, for both houses)
#[test]
fn intent_lines_with_the_sensors_found_once() {
    for on in [true, false] {
        let mut r = sensor_rules();
        r.globals.quirks.psychic_intent_ignores_visibility = on;
        building(&mut r, "WATCH", |b| b.econ.sight = Some(4));
        let mut w = testkit::game_with(&r, SIZE, 1);
        let eye = put(&mut w, Kind::Building, "EYE", ALPHA, 0, 0);
        put(&mut w, Kind::Building, "EYE", BETA, 9, 9);
        // Each house sees one corner of the fight, so with the quirk off some
        // units are seen and some are not.
        put(&mut w, Kind::Building, "WATCH", ALPHA, -7, 2);
        put(&mut w, Kind::Building, "WATCH", BETA, 7, 7);
        let mut units = Vec::new();
        for (i, (house, dx, dy)) in [
            (BETA, -8, 0),
            (BETA, 8, 0),
            (BETA, 0, 2),
            (BETA, -10, -10),
            (ALPHA, 1, 1),
            (ALPHA, 7, 9),
            (ALPHA, -6, 5),
        ]
        .into_iter()
        .enumerate()
        {
            let name = if i % 2 == 0 { "BOX" } else { "TANK" };
            units.push(put(&mut w, Kind::Vehicle, name, house, dx, dy));
        }
        testkit::run(&mut w, 2);
        let dests = [
            at(2, 0),
            at(9, 9),
            at(8, 8),
            at(1, 1),
            at(-9, 0),
            at(0, 1),
            at(9, 8),
        ];
        for (k, (&u, &d)) in units.iter().zip(&dests).enumerate() {
            let o = w.obj_mut(u).unwrap();
            o.dest = Some(Target::Cell(d));
            if k % 3 == 1 {
                o.target = Some(Target::Object(units[(k + 3) % units.len()]));
            }
        }
        let mut seen = 0;
        for round in 0..3 {
            match round {
                1 => w.obj_mut(eye).unwrap().econ.building.switched_off = true,
                2 => {
                    w.obj_mut(eye).unwrap().econ.building.switched_off = false;
                    sw_common::ally(&mut w, HouseId(BETA), HouseId(ALPHA));
                }
                _ => {}
            }
            for h in [ALPHA, BETA] {
                let lines = psychic::intent_lines(&w, HouseId(h));
                assert_eq!(
                    lines,
                    intent_lines_one_by_one(&w, HouseId(h)),
                    "quirk {on}, round {round}, house {h}"
                );
                seen += lines.len();
            }
        }
        assert!(seen > 0, "quirk {on}: some lines are drawn");
    }
}
