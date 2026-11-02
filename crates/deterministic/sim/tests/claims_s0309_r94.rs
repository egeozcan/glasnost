//! Claim tests of S-0309 revision 94 (M8 step A3 intake 74): claim 29's
//! answered place in the turn, behaviours and owners (Q-2261 to Q-2263),
//! the new claim 25 part k (the allied held-target drop of a holder of a
//! computer-controlled owner), and S-0204 revision 10's switch over S-0214
//! F1's representative distance. Synthetic data only: every type, weapon,
//! warhead and number is invented.
//!
//! A drop in tick *k* shows after `tick::step` has run that tick, when the
//! tick number reads *k* + 1. The ledger is `docs/m8-claims/intake74.md`.

use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{ArmourClass, Decimal, UnitType, WarheadType, WeaponType};
use glasnost_sim::combat::held;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::targeting::{self, ClickOn};
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, Pos, Target, World};

const SIZE: i32 = 24;
const ALPHA: u8 = 0;
const BETA: u8 = 1;
/// The neutral and special houses of `testkit::game_with_non_players`.
const NEUTRAL: u8 = 2;
const SPECIAL: u8 = 3;
/// The check period of claim 29, written out for the expected ticks.
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

fn building(r: &mut Ruleset, name: &str, f: impl FnOnce(&mut glasnost_rules::sim::BuildingType)) {
    let mut b = r.sim.buildings["TOWER"].clone();
    b.primary = None;
    b.secondary = None;
    f(&mut b);
    register(r, "buildings", name);
    r.sim.buildings.insert(name.into(), b);
}

fn warhead(r: &mut Ruleset, name: &str, verses: Option<Vec<Decimal>>, electric: bool) {
    r.sim.warheads.insert(
        name.into(),
        WarheadType {
            verses,
            electric_assault: electric.then_some(true),
            ..Default::default()
        },
    );
}

/// A direct weapon of range 5 cells firing every 30 ticks.
fn weapon(r: &mut Ruleset, name: &str, warhead_name: &str) {
    r.sim.weapons.insert(
        name.into(),
        WeaponType {
            damage: Some(20),
            rof: Some(30),
            range: Some(dec("5")),
            speed: Some(100),
            projectile: Some("Bullet".into()),
            warhead: Some(warhead_name.into()),
            ..Default::default()
        },
    );
}

/// The rules of this file:
/// - warhead `H`, 0 against class *light* and 1 elsewhere, with weapon `W`;
///   warhead `PLAIN`, 1 everywhere, with weapon `Plain`; warhead `ELEC`, 1
///   everywhere and electric-charge, with weapon `Zap`;
/// - vehicles `DEF` (primary `W`), `TARGET` (class *light*), `PLAINV`
///   (class *heavy*), `ZAPPER` (primary `Plain`, secondary `Zap`), `OPEN`
///   (an open-topped transport of 2 passengers);
/// - infantry `GI` (primary `Plain`), `ENG` (`GI` of an engineer type),
///   `OCC` (`GI` of an occupier type);
/// - buildings `PILL` (garrisonable, 2 occupants), `COILB` (overpowerable)
///   and `HUT` (neither), all unarmed.
fn rules() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    warhead(&mut r, "H", only(LIGHT, "0"), false);
    warhead(&mut r, "PLAIN", only(LIGHT, "1"), false);
    warhead(&mut r, "ELEC", only(LIGHT, "1"), true);
    weapon(&mut r, "W", "H");
    weapon(&mut r, "Plain", "PLAIN");
    weapon(&mut r, "Zap", "ELEC");
    vehicle(&mut r, "DEF", |t| t.primary = Some("W".into()));
    vehicle(&mut r, "TARGET", |t| {
        t.combat.armor = Some(ArmourClass::Light)
    });
    vehicle(&mut r, "PLAINV", |t| {
        t.combat.armor = Some(ArmourClass::Heavy)
    });
    vehicle(&mut r, "ZAPPER", |t| {
        t.primary = Some("Plain".into());
        t.secondary = Some("Zap".into());
    });
    vehicle(&mut r, "OPEN", |t| {
        t.passengers = Some(2);
        t.open_topped = Some(true);
    });
    infantry(&mut r, "GI", |t| t.primary = Some("Plain".into()));
    infantry(&mut r, "ENG", |t| {
        t.primary = Some("Plain".into());
        t.engineer = Some(true);
    });
    infantry(&mut r, "OCC", |t| {
        t.primary = Some("Plain".into());
        t.occupier = Some(true);
    });
    building(&mut r, "PILL", |b| {
        b.can_be_occupied = Some(true);
        b.max_number_occupants = Some(2);
    });
    building(&mut r, "COILB", |b| b.overpowerable = Some(true));
    building(&mut r, "HUT", |_| {});
    r
}

fn at(dx: i32, dy: i32) -> CellXY {
    testkit::mid(SIZE, dx, dy)
}

fn put(w: &mut World, kind: Kind, name: &str, house: u8, dx: i32, dy: i32) -> ObjectId {
    testkit::spawn(w, kind, name, house, at(dx, dy), 0)
}

fn target(w: &World, id: ObjectId) -> Option<Target> {
    w.obj(id).and_then(|o| o.target)
}

fn step(w: &mut World) {
    glasnost_sim::tick::step(w);
}

/// A game of this file's rules with the neutral and special houses.
fn game() -> World {
    testkit::game_with_non_players(&rules(), SIZE, 1)
}

/// Runs until the tick about to run leaves `rem` when divided by 16.
fn run_to(w: &mut World, rem: i32) {
    while w.tick.rem_euclid(PERIOD) != rem {
        step(w);
    }
}

/// Gives `id` target `x` and behaviour `b` directly (as an order or a
/// search would), its targeting timer 200 ticks away and its behaviour
/// processing not due for a long time, so that the turn's held-target work
/// shows alone.
fn hold(w: &mut World, id: ObjectId, x: ObjectId, b: Behaviour) {
    let t = w.tick;
    let o = w.obj_mut(id).unwrap();
    o.beh.force(b, t, false);
    o.target = Some(Target::Object(x));
    o.combat.opportunity = false;
    o.targeting_timer = t + 200;
    o.beh.processed(t, 1000);
}

/// Whether holder `id`, given `x` with behaviour `b` in a tick that is not
/// a check tick, still holds it at the end of that tick (its first turn
/// after it was given it, run p9b's tick *n* + 1).
fn kept_off_phase(w: &mut World, id: ObjectId, x: ObjectId, b: Behaviour) -> bool {
    run_to(w, 5);
    hold(w, id, x, b);
    assert!(!held::check_tick(w.tick), "not a check tick");
    step(w);
    target(w, id) == Some(Target::Object(x))
}

// ---------------------------------------------------------------------------
// Claim 25 part k: the allied drop.
// ---------------------------------------------------------------------------

// spec: S-0309 claim 25 part k (a holder of a computer-controlled owner, in our engine the neutral and special houses, whose target is allied with it, its own house included, drops the target in each of its turns, in a tick of any number) and scenario A6 part p run p9b (the drop by a vehicle holder of a target of its own house in its first turn after its pick, tick n + 1 not a check tick)
#[test]
fn p9b_a_computer_owned_holder_drops_a_target_of_its_own_house_off_the_check_phase() {
    for house in [NEUTRAL, SPECIAL] {
        let mut w = game();
        let d = put(&mut w, Kind::Vehicle, "DEF", house, -1, 0);
        let z = put(&mut w, Kind::Vehicle, "PLAINV", house, 2, 0);
        assert!(
            !kept_off_phase(&mut w, d, z, Behaviour::AreaGuard),
            "house {house}"
        );
        // spec: S-0309 claim 25 part k (the drop has claim 29's consequences: no behaviour change by itself)
        assert_eq!(w.obj(d).unwrap().beh.current, Some(Behaviour::AreaGuard));
    }
}

// spec: S-0309 claim 25 part k (a holder of a human-controlled owner has no such drop: in our engine the holders of every player house keep an allied target) and scenario A6 part p run p9b's control
#[test]
fn p9b_control_a_player_holder_keeps_a_target_of_its_own_house() {
    let mut w = game();
    let d = put(&mut w, Kind::Vehicle, "DEF", ALPHA, -1, 0);
    let z = put(&mut w, Kind::Vehicle, "PLAINV", ALPHA, 2, 0);
    assert!(kept_off_phase(&mut w, d, z, Behaviour::AreaGuard));
}

// spec: S-0309 Terms (Test-only exception: a marked player house takes the computer-owner rules) and claim 25 part k
#[test]
fn a_marked_test_house_takes_the_allied_drop() {
    let mut w = game();
    w.houses[usize::from(ALPHA)].human = false;
    let d = put(&mut w, Kind::Vehicle, "DEF", ALPHA, -1, 0);
    let z = put(&mut w, Kind::Vehicle, "PLAINV", ALPHA, 2, 0);
    assert!(!kept_off_phase(&mut w, d, z, Behaviour::Guard));
}

// spec: S-0309 claim 25 part k (only a target allied with the holder: an enemy target that the firing check allows is kept off the check phase, as before)
#[test]
fn a_computer_owned_holder_keeps_an_enemy_target() {
    let mut w = game();
    let d = put(&mut w, Kind::Vehicle, "DEF", NEUTRAL, -1, 0);
    let x = put(&mut w, Kind::Vehicle, "PLAINV", BETA, 2, 0);
    assert!(!w.allied(Some(HouseId(NEUTRAL)), Some(HouseId(BETA))));
    assert!(kept_off_phase(&mut w, d, x, Behaviour::Guard));
}

// spec: S-0309 claim 25 part k (the drop is made in every behaviour, capture and sabotage included, which make no claim 29 check)
#[test]
fn the_allied_drop_is_made_in_every_behaviour() {
    for b in [
        Behaviour::Attack,
        Behaviour::Guard,
        Behaviour::Hunt,
        Behaviour::Move,
        Behaviour::Capture,
        Behaviour::Sabotage,
    ] {
        let mut w = game();
        let d = put(&mut w, Kind::Vehicle, "DEF", NEUTRAL, -1, 0);
        let z = put(&mut w, Kind::Vehicle, "PLAINV", NEUTRAL, 2, 0);
        assert!(!kept_off_phase(&mut w, d, z, b), "{b:?}");
    }
}

// spec: S-0309 claim 25 part k (the drop has claim 29's consequences: no timer draw) and claim 29 (The drop: no targeting-timer draw; State and ordering: the drops take no draw)
#[test]
fn the_allied_drop_takes_no_draw() {
    let mut w = game();
    let d = put(&mut w, Kind::Vehicle, "DEF", NEUTRAL, -1, 0);
    let z = put(&mut w, Kind::Vehicle, "PLAINV", NEUTRAL, 2, 0);
    run_to(&mut w, 5);
    hold(&mut w, d, z, Behaviour::Guard);
    let mut control = w.clone();
    control.obj_mut(d).unwrap().target = None;
    let timer = w.obj(d).unwrap().targeting_timer;
    step(&mut w);
    step(&mut control);
    assert_eq!(target(&w, d), None, "dropped");
    assert_eq!(w.obj(d).unwrap().targeting_timer, timer, "no reset");
    assert_eq!(w.rng, control.rng, "no draw");
}

// spec: S-0309 claim 25 part k (kept when the holder is berserk, S-0610)
#[test]
fn a_berserk_holder_keeps_an_allied_target() {
    for berserk in [true, false] {
        let mut w = game();
        let d = put(&mut w, Kind::Vehicle, "DEF", NEUTRAL, -1, 0);
        let z = put(&mut w, Kind::Vehicle, "PLAINV", NEUTRAL, 2, 0);
        if berserk {
            w.obj_mut(d).unwrap().combat.berserk = Some(500);
        }
        assert_eq!(
            kept_off_phase(&mut w, d, z, Behaviour::Guard),
            berserk,
            "berserk {berserk}"
        );
    }
}

// spec: S-0309 claim 25 part k (kept when the holder is an infantry unit of an engineer type)
#[test]
fn an_engineer_keeps_an_allied_target() {
    for (name, kept) in [("ENG", true), ("GI", false)] {
        let mut w = game();
        let d = put(&mut w, Kind::Infantry, name, NEUTRAL, -1, 0);
        let z = put(&mut w, Kind::Vehicle, "PLAINV", NEUTRAL, 2, 0);
        assert_eq!(
            kept_off_phase(&mut w, d, z, Behaviour::Guard),
            kept,
            "{name}"
        );
    }
}

// spec: S-0309 claim 25 part k (kept when the holder is an infantry unit, the target a building and the holder can occupy it: the structures specs' test, S-0508 claim 1)
#[test]
fn an_occupier_keeps_a_building_it_can_occupy() {
    for (unit, b, kept) in [
        ("OCC", "PILL", true),
        ("GI", "PILL", false),
        ("OCC", "HUT", false),
    ] {
        let mut w = game();
        let d = put(&mut w, Kind::Infantry, unit, NEUTRAL, -1, 0);
        let p = put(&mut w, Kind::Building, b, NEUTRAL, 3, 3);
        assert_eq!(
            glasnost_sim::structures::garrison::may_enter(&w, d, p),
            kept,
            "{unit} {b}: the occupation test"
        );
        assert_eq!(
            kept_off_phase(&mut w, d, p, Behaviour::Guard),
            kept,
            "{unit} {b}"
        );
    }
}

// spec: S-0309 claim 25 part k (kept when the holder's second weapon is an electric-charge weapon, S-0518, and the target a building of an overpowerable type)
#[test]
fn an_electric_charger_keeps_an_overpowerable_building() {
    for (unit, b, kept) in [
        ("ZAPPER", "COILB", true),
        ("ZAPPER", "HUT", false),
        ("DEF", "COILB", false),
    ] {
        let mut w = game();
        let d = put(&mut w, Kind::Vehicle, unit, NEUTRAL, -1, 0);
        let p = put(&mut w, Kind::Building, b, NEUTRAL, 3, 3);
        assert_eq!(
            kept_off_phase(&mut w, d, p, Behaviour::Guard),
            kept,
            "{unit} {b}"
        );
    }
}

// spec: S-0309 claim 25 part k (for a passenger of an open-topped transport that is under mind control, both the owner test and the alliance are those of the house that controls the transport)
#[test]
fn a_passenger_of_a_controlled_open_topped_transport_reads_the_controlling_house() {
    for controlled in [false, true] {
        let mut w = game();
        let open = put(&mut w, Kind::Vehicle, "OPEN", NEUTRAL, 0, 0);
        let p = put(&mut w, Kind::Infantry, "GI", NEUTRAL, 0, 1);
        glasnost_sim::transport::board(&mut w, p, open);
        assert!(glasnost_sim::transport::open_topped_passenger(&w, p));
        let z = put(&mut w, Kind::Vehicle, "PLAINV", ALPHA, 3, 0);
        if controlled {
            // The transport permanently controlled by Alpha, a human-controlled house of our engine.
            assert!(glasnost_sim::mind::dominate(&mut w, open, HouseId(ALPHA)));
            assert_eq!(w.obj(p).unwrap().owner, Some(HouseId(NEUTRAL)));
        }
        // Not controlled: the neutral passenger's own house decides, and Alpha's vehicle is no ally of it.
        // Controlled: Alpha decides, a human-controlled house, so no drop.
        assert!(
            kept_off_phase(&mut w, p, z, Behaviour::Guard),
            "an enemy or a human-controlled decider: kept ({controlled})"
        );
        let mut w = game();
        let open = put(&mut w, Kind::Vehicle, "OPEN", ALPHA, 0, 0);
        let p = put(&mut w, Kind::Infantry, "GI", ALPHA, 0, 1);
        glasnost_sim::transport::board(&mut w, p, open);
        let z = put(&mut w, Kind::Vehicle, "PLAINV", NEUTRAL, 3, 0);
        if controlled {
            // The transport permanently controlled by the neutral house, a computer-controlled one.
            assert!(glasnost_sim::mind::dominate(&mut w, open, HouseId(NEUTRAL)));
            assert_eq!(w.obj(p).unwrap().owner, Some(HouseId(ALPHA)));
        }
        // Controlled: the neutral house decides both the owner test and the alliance, so the
        // Alpha passenger drops the neutral vehicle; not controlled: its own human house keeps it.
        assert_eq!(
            kept_off_phase(&mut w, p, z, Behaviour::Guard),
            !controlled,
            "the controlling house decides ({controlled})"
        );
    }
}

// ---------------------------------------------------------------------------
// Claim 29, revision 94: place in the turn.
// ---------------------------------------------------------------------------

// spec: S-0309 claim 29 (Place in the turn, revision 94, Q-2261 answered: after the turn's first per-tick start, S-0215 claim 11, so the behaviour it makes current decides: a holder in capture with attack queued that the start makes current makes the check) and claim 29 (Behaviours: capture makes none)
#[test]
fn the_behaviour_made_current_by_the_first_per_tick_start_decides() {
    for queued in [None, Some(Behaviour::Attack)] {
        let mut w = game();
        let d = put(&mut w, Kind::Vehicle, "DEF", ALPHA, -1, 0);
        let x = put(&mut w, Kind::Vehicle, "TARGET", BETA, 2, 0);
        // The tick about to run is a check tick; the attack is queued just before it.
        run_to(&mut w, 0);
        hold(&mut w, d, x, Behaviour::Capture);
        w.obj_mut(d).unwrap().beh.queued = queued;
        step(&mut w);
        assert!(held::check_tick(w.tick - 1));
        let o = w.obj(d).unwrap();
        if queued.is_some() {
            // The attack's own processing then has no target and falls
            // back (S-0215), so only the drop is asserted.
            assert_eq!(o.target, None, "the attack made current made the check");
        } else {
            assert_eq!(o.target, Some(Target::Object(x)), "capture makes no check");
        }
    }
}

// spec: S-0309 claim 29 (Owners and targets, revision 94, Q-2263 answered: every holder of every house, the special house included) and claim 29 (Behaviours, Q-2262 answered: area guard makes the check), scenario A6 part p run p9a
#[test]
fn p9a_a_special_holder_in_area_guard_drops_an_illegal_target_at_the_check_tick() {
    let mut w = game();
    let d = put(&mut w, Kind::Vehicle, "DEF", SPECIAL, -1, 0);
    let x = put(&mut w, Kind::Vehicle, "TARGET", ALPHA, 2, 0);
    run_to(&mut w, 3);
    hold(&mut w, d, x, Behaviour::AreaGuard);
    let n = w.tick;
    for _ in 0..13 {
        step(&mut w);
    }
    assert_eq!(
        target(&w, d),
        Some(Target::Object(x)),
        "held to the check tick"
    );
    step(&mut w);
    assert_eq!(w.tick - 1, n + 13);
    assert!(held::check_tick(w.tick - 1));
    assert_eq!(target(&w, d), None, "dropped at the check tick");
}

// ---------------------------------------------------------------------------
// S-0204 revision 10: the switch over S-0214 F1.
// ---------------------------------------------------------------------------

// spec: S-0214 F1 (d = ⌊r(n)⌋, the table square root) and S-0204 claim 5 revision 10 and Edge cases (quirks.table_root_distance covers S-0214's representative distance; off: ⌊√n⌋)
#[test]
fn the_representative_distance_follows_the_table_root_switch() {
    let mut differs = false;
    for table in [true, false] {
        let mut r = rules();
        r.globals.quirks.table_root_distance = table;
        let mut w = testkit::game_with(&r, SIZE, 1);
        let a = put(&mut w, Kind::Vehicle, "DEF", ALPHA, 0, 0);
        let p = w.reference_point(a);
        // Cells far beyond the map, where the table root falls short (S-0204 claim 5: from m = 16393 on).
        for k in (130..400).step_by(7) {
            let c = CellXY::new(SIZE + k, SIZE + k / 3);
            let q = Pos::centre(c, 0);
            let (dx, dy, dz) = (
                i64::from(p.x - q.x),
                i64::from(p.y - q.y),
                i64::from(p.z - q.z),
            );
            let n = (dx * dx + dy * dy + dz * dz) as u64;
            let d = targeting::rep_distance(&w, a, ClickOn::Cell(c));
            let exact = glasnost_num::geom::root_floor(n, false, w.rules().mode);
            let tabled = glasnost_num::geom::root_floor(n, true, w.rules().mode);
            assert_eq!(
                d,
                if table { tabled } else { exact },
                "k {k}, table {table}"
            );
            differs |= tabled != exact;
        }
    }
    assert!(differs, "some distance tells the two settings apart");
}
