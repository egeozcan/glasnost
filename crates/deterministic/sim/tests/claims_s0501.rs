//! Claim tests for S-0501 (aircraft: flight, attack runs, pads) on
//! synthetic data: invented aircraft, pad, weapon and carrier types, none
//! of them stock values.

mod econ_common;

use econ_common::{SIZE, building, dec, factory, game, issue, ready, set_credits, t, wheel};
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{Locomotor, ProjectileType, UnitType, WeaponType};
use glasnost_sim::air;
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::targeting::{self, ClickOn, Modifiers};
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, Target, World};

const ALPHA: HouseId = HouseId(0);

fn owners() -> Option<Vec<String>> {
    Some(vec!["Alphaland".into(), "Betaland".into()])
}

fn push(r: &mut Ruleset, reg: &str, n: &str) {
    r.registries.entry(reg.into()).or_default().push(n.into());
}

/// A flying aircraft type with weapon `weapon`, `ammo` rounds, docking at
/// APAD.
fn flyer(weapon: Option<&str>, ammo: i32) -> UnitType {
    let mut u = UnitType {
        strength: Some(200),
        speed: Some(30),
        rot: Some(12),
        locomotor: Some(Locomotor::Fly),
        tech_level: Some(1),
        owner: owners(),
        cost: Some(1000),
        primary: weapon.map(Into::into),
        ..Default::default()
    };
    u.combat.ammo = Some(ammo);
    u.econ.landable = Some(true);
    u.econ.dock = Some(vec!["APAD".into(), "PADB".into()]);
    u
}

/// The aircraft ruleset: the combat ruleset with
/// - the pad APAD (an aircraft factory, helipad and reload pad, one
///   docking place) and the plain pad PADB (helipad, reload pad, no
///   factory);
/// - JET (a non-strafing attacker, weapon Agm, 2 rounds), JET2 (the same
///   with the burst-2 weapon Agm2, 3 rounds), STRAFE (a strafer, weapon
///   Chain, 3 rounds), FIGHT (a fighter, weapon Agm, 2 rounds), SCOUT
///   (unarmed), SPY (unarmed and not selectable, so visiting);
/// - HORNET (a spawned slot aircraft, weapon Agm, 1 round) and the carrier
///   vehicle CARRIER (two HORNET slots, spawner weapon Launch);
/// - the target vehicle VICTIM (strength 5000).
fn rules() -> Ruleset {
    let mut r = econ_common::ruleset();
    let c = testkit::combat_ruleset();
    r.sim.weapons.extend(c.sim.weapons.clone());
    r.sim.projectiles.extend(c.sim.projectiles.clone());
    r.sim.warheads.extend(c.sim.warheads.clone());
    let mut pad = factory("aircraft");
    pad.helipad = Some(true);
    pad.econ.unit_reload = Some(true);
    pad.econ.number_of_docks = Some(1);
    push(&mut r, "buildings", "APAD");
    r.sim.buildings.insert("APAD".into(), pad);
    let mut padb = econ_common::plain(500);
    padb.foundation = Some([2, 2]);
    padb.helipad = Some(true);
    padb.econ.unit_reload = Some(true);
    padb.econ.number_of_docks = Some(1);
    push(&mut r, "buildings", "PADB");
    r.sim.buildings.insert("PADB".into(), padb);
    let s = &mut r.sim;
    s.weapons.insert(
        "Agm".into(),
        WeaponType {
            damage: Some(40),
            rof: Some(30),
            range: dec("6"),
            speed: Some(60),
            projectile: Some("Rocket".into()),
            warhead: Some("HE".into()),
            ..Default::default()
        },
    );
    s.weapons.insert(
        "Agm2".into(),
        WeaponType {
            damage: Some(40),
            rof: Some(30),
            range: dec("6"),
            speed: Some(60),
            burst: Some(2),
            projectile: Some("Rocket".into()),
            warhead: Some("HE".into()),
            ..Default::default()
        },
    );
    s.weapons.insert(
        "Chain".into(),
        WeaponType {
            damage: Some(10),
            rof: Some(6),
            range: dec("5"),
            speed: Some(100),
            projectile: Some("Straight".into()),
            warhead: Some("SA".into()),
            ..Default::default()
        },
    );
    s.projectiles
        .insert("Straight".into(), ProjectileType::default());
    for (n, u) in [
        ("JET", flyer(Some("Agm"), 2)),
        ("STRAFE", flyer(Some("Chain"), 3)),
        ("FIGHT", {
            let mut u = flyer(Some("Agm"), 2);
            u.econ.fighter = Some(true);
            u
        }),
        ("SCOUT", flyer(None, 0)),
        ("SPY", {
            let mut u = flyer(None, 0);
            u.selectable = Some(false);
            u
        }),
        ("JET2", flyer(Some("Agm2"), 3)),
        ("HORNET", {
            let mut u = flyer(Some("Agm"), 1);
            u.spawned = Some(true);
            u.econ.dock = None;
            u
        }),
    ] {
        push(&mut r, "aircraft", n);
        r.sim.aircraft.insert(n.into(), u);
    }
    let mut victim = wheel(1000);
    victim.strength = Some(5000);
    push(&mut r, "vehicles", "VICTIM");
    r.sim.vehicles.insert("VICTIM".into(), victim);
    let mut carrier = wheel(1000);
    carrier.strength = Some(2000);
    carrier.primary = Some("Launch".into());
    carrier.combat.spawns = Some("HORNET".into());
    carrier.combat.spawns_number = Some(2);
    carrier.combat.spawn_regen_rate = Some(50);
    carrier.combat.spawn_reload_rate = Some(20);
    carrier.combat.primary_fire_flh = Some([0, 0, 40]);
    push(&mut r, "vehicles", "CARRIER");
    r.sim.vehicles.insert("CARRIER".into(), carrier);
    r.sim.weapons.insert(
        "Launch".into(),
        WeaponType {
            damage: Some(1),
            rof: Some(20),
            range: dec("12"),
            projectile: Some("Bullet".into()),
            warhead: Some("HE".into()),
            spawner: Some(true),
            ..Default::default()
        },
    );
    // A short service cadence: P = ⌊0.01 × 900⌋, 8 ticks with the decimal read toward zero.
    r.sim.general.economy.reload_rate = dec("0.01");
    r
}

fn put(w: &mut World, kind: Kind, name: &str, house: u8, dx: i32, dy: i32) -> ObjectId {
    testkit::spawn(w, kind, name, house, testkit::mid(SIZE, dx, dy), 0)
}

fn open(w: &mut World) {
    w.tick = w.tick.max(w.orders_blocked_through + 1);
}

fn click(w: &mut World, actor: ObjectId, on: ClickOn) {
    open(w);
    let r = targeting::click(w, ALPHA, &[actor], on, Modifiers::default());
    assert!(!r.orders.is_empty(), "the click gives an order");
    for o in r.orders {
        assert!(orders::issue(w, ALPHA, o));
    }
}

fn step(w: &mut World) {
    glasnost_sim::tick::step(w);
}

fn run_until(w: &mut World, max: i32, mut done: impl FnMut(&World) -> bool) -> bool {
    for _ in 0..max {
        if done(w) {
            return true;
        }
        step(w);
    }
    done(w)
}

fn st(w: &World, id: ObjectId) -> air::AirState {
    w.obj(id).unwrap().air.clone().unwrap_or_default()
}

fn describe(w: &World, id: ObjectId) -> String {
    let o = w.obj(id).unwrap();
    format!(
        "t={} pos={:?} h={} beh={:?}/{:?} dest={:?} target={:?} ammo={} contact={:?} air={:?}",
        w.tick,
        o.pos,
        air::height(w, id),
        o.beh.current,
        o.beh.queued,
        o.dest,
        o.target,
        o.combat.ammo,
        o.contact,
        o.air
    )
}

/// Builds `name` at a fresh APAD of house 0; returns (world, pad, aircraft).
fn built(name: &str) -> (World, ObjectId, ObjectId) {
    built_with(&rules(), name)
}

fn built_with(r: &Ruleset, name: &str) -> (World, ObjectId, ObjectId) {
    let mut w = game(r);
    let k = building(&mut w, "APAD", 0, 0, 0);
    ready(&mut w);
    set_credits(&mut w, 0, 100_000);
    let what = t(&w, Kind::Aircraft, name);
    assert!(issue(&mut w, 0, Order::Build { what }));
    let ti = what.index;
    let found = run_until(&mut w, 3000, |w| {
        w.objects
            .values()
            .any(|o| o.kind == Kind::Aircraft && o.type_index == ti && o.on_map)
    });
    assert!(found, "the aircraft is built");
    let a = w
        .objects
        .values()
        .find(|o| o.kind == Kind::Aircraft && o.type_index == ti && o.on_map)
        .unwrap()
        .id;
    (w, k, a)
}

// spec: S-0501 claim 1, S-0404 claims 13 and 15 (end to end: an aircraft built at a pad stands docked on the ground at its pad position, in contact with the pad)
#[test]
fn built_aircraft_stands_docked() {
    let (mut w, k, a) = built("JET");
    let (p, _) = glasnost_sim::econ::release::pad_position(&w, k, a);
    assert_eq!(w.obj(a).unwrap().pos, p, "{}", describe(&w, a));
    assert_eq!(w.obj(a).unwrap().contact, Some(k));
    assert!(air::on_ground(&w, a));
    assert!(st(&w, a).ground);
    testkit::run(&mut w, 30);
    assert_eq!(w.obj(a).unwrap().pos, p, "it stays: {}", describe(&w, a));
}

// spec: S-0501 claims 8, 12 to 15, 17, 18, 21 to 26 and table T1 (end to end: a docked aircraft ordered by a click to attack an enemy vehicle takes off, makes its runs, damages the target, returns to its pad, lands and rearms)
#[test]
fn attack_take_off_run_return_rearm() {
    let mut r = rules();
    // The re-approach flag on, so that each run ends (claim 14).
    r.sim.general.economy.curley_shuffle = Some(true);
    let (mut w, k, a) = built_with(&r, "JET");
    let v = put(&mut w, Kind::Vehicle, "VICTIM", 1, 10, 0);
    let full = w.obj(v).unwrap().health;
    click(&mut w, a, ClickOn::Object(v));
    let off = run_until(&mut w, 200, |w| air::height(w, a) > 0);
    assert!(off, "takes off: {}", describe(&w, a));
    let hit = run_until(&mut w, 2000, |w| w.obj(v).unwrap().health < full);
    assert!(hit, "damages the target: {}", describe(&w, a));
    let empty = run_until(&mut w, 3000, |w| w.obj(a).unwrap().combat.ammo == 0);
    assert!(empty, "spends its rounds: {}", describe(&w, a));
    let back = run_until(&mut w, 3000, |w| {
        air::on_ground(w, a) && w.obj(a).unwrap().contact == Some(k)
    });
    assert!(back, "lands on its pad: {}", describe(&w, a));
    let (p, _) = glasnost_sim::econ::release::pad_position(&w, k, a);
    assert_eq!(w.obj(a).unwrap().pos, p);
    let full_ammo = run_until(&mut w, 500, |w| w.obj(a).unwrap().combat.ammo == 2);
    assert!(full_ammo, "rearms: {}", describe(&w, a));
}

// spec: S-0501 claims 4 to 8 and 33 (end to end: a move order flies the aircraft to the cell, where it lands); S-0417 claim 11 (the exit tether of a type that need not return to a dock ends when it leaves the ground)
#[test]
fn move_order_flies_to_the_cell() {
    let (mut w, _k, a) = built("JET");
    let tethered = |w: &World| {
        w.obj(a)
            .unwrap()
            .econ
            .exit
            .as_ref()
            .is_some_and(|e| e.tethered)
    };
    assert!(tethered(&w), "tethered while docked");
    let goal = testkit::mid(SIZE, 8, 6);
    click(&mut w, a, ClickOn::Cell(goal));
    let flew = run_until(&mut w, 200, |w| air::height(w, a) > 100);
    assert!(flew, "takes off: {}", describe(&w, a));
    assert!(!tethered(&w), "the tether ends at take-off");
    let there = run_until(&mut w, 2000, |w| {
        w.obj(a).unwrap().pos.cell() == goal && air::on_ground(w, a)
    });
    assert!(there, "lands in the cell: {}", describe(&w, a));
}

// spec: S-0501 claims 35 and 36 and table T4, S-0506 claims 6 to 9 (end to end: a carrier with slot aircraft ordered to attack launches them; they attack, damage the target and come back)
#[test]
fn carrier_launches_attacks_and_recovers() {
    let r = rules();
    let mut w = game(&r);
    ready(&mut w);
    let c = put(&mut w, Kind::Vehicle, "CARRIER", 0, 0, 0);
    let v = put(&mut w, Kind::Vehicle, "VICTIM", 1, 8, 0);
    let full = w.obj(v).unwrap().health;
    testkit::run(&mut w, 2);
    click(&mut w, c, ClickOn::Object(v));
    let hornets = |w: &World| -> Vec<ObjectId> {
        w.objects
            .values()
            .filter(|o| o.kind == Kind::Aircraft && o.on_map)
            .map(|o| o.id)
            .collect()
    };
    let launched = run_until(&mut w, 400, |w| !hornets(w).is_empty());
    assert!(launched, "launches");
    let flying = run_until(&mut w, 400, |w| {
        hornets(w).iter().any(|&h| air::height(w, h) > 100)
    });
    assert!(flying, "a slot aircraft flies");
    let hit = run_until(&mut w, 3000, |w| w.obj(v).unwrap().health < full);
    let hs = hornets(&w);
    assert!(
        hit,
        "damages the target: {:?}",
        hs.iter().map(|&h| describe(&w, h)).collect::<Vec<_>>()
    );
    let home = run_until(&mut w, 4000, |w| hornets(w).is_empty());
    assert!(
        home,
        "the aircraft come back inside: {:?}",
        hornets(&w)
            .iter()
            .map(|&h| describe(&w, h))
            .collect::<Vec<_>>()
    );
    let slots = w.obj(c).unwrap().spawn.clone().unwrap().slots;
    assert!(
        slots.iter().all(|s| s.object.is_some()),
        "both slots hold their aircraft again"
    );
}

// ---------------------------------------------------------------------------
// Helpers for the claim tests

fn layer_mark(w: &World, id: ObjectId) -> u32 {
    let c = w.obj(id).unwrap().pos.cell();
    w.cells
        .occ(c, glasnost_sim::world::Layer::Ground)
        .map_or(0, |r| u32::from(r.others))
}

fn force(w: &mut World, id: ObjectId, b: Behaviour) {
    let t = w.tick;
    w.obj_mut(id).unwrap().beh.force(b, t, false);
}

/// Puts `name` of house 0 on the ground at (dx, dy) and flies it up to
/// its cruise height toward a far cell; returns it in flight.
fn lifted(w: &mut World, name: &str, dx: i32, dy: i32) -> ObjectId {
    let a = put(w, Kind::Aircraft, name, 0, dx, dy);
    let far = testkit::mid(SIZE, dx - 12, dy - 12);
    open(w);
    assert!(orders::issue(
        w,
        ALPHA,
        Order::Behaviour {
            actor: a,
            behaviour: Behaviour::Move,
            attack_move: false,
            target: None,
            dest: Some(Target::Cell(far)),
        }
    ));
    assert!(
        run_until(w, 300, |w| air::height(w, a) >= 500),
        "lifted: {}",
        describe(w, a)
    );
    a
}

fn hit(amount: i32) -> glasnost_sim::combat::damage::Hit {
    glasnost_sim::combat::damage::Hit {
        amount,
        warhead: None,
        distance: 0,
        attacker: None,
        house: None,
        ignore_defences: true,
        no_escape: false,
    }
}

fn nd(s: &str) -> glasnost_num::F64 {
    glasnost_sim::rules::nearest_double(s)
}

// ---------------------------------------------------------------------------
// Formulas

// spec: S-0501 F1 (climb by min(FLₕ − h, c), c = 10 with passengers, 20 otherwise)
#[test]
fn f1_climb() {
    assert_eq!(air::climb(0, 500, false), 20);
    assert_eq!(air::climb(0, 500, true), 10);
    assert_eq!(air::climb(490, 500, false), 500);
}

// spec: S-0501 F2 (δ = min(50, max(20, ⌊(h − FLₕ) / 20⌋)); a dead aircraft keeps at least 1 lepton; then at least 0)
#[test]
fn f2_descent() {
    assert_eq!(air::descend(500, 0, false), 475);
    assert_eq!(air::descend(2000, 0, false), 1950);
    assert_eq!(air::descend(30, 0, false), 10);
    assert_eq!(air::descend(10, 0, false), 0);
    assert_eq!(air::descend(10, 0, true), 1);
    assert_eq!(air::descend(700, 500, false), 680);
}

// spec: S-0501 F4 (τ₀ = min(1, d / S); the creep case below 0.1 and 86 leptons; one adjustment of at most 0.1)
#[test]
fn f4_slowing() {
    let w = game(&rules());
    let ar = w.rules().air.clone();
    let m = w.rules().mode;
    let one = glasnost_num::F64::from_i64(1, m);
    let zero = glasnost_num::F64::ZERO;
    assert_eq!(air::slowing(one, 1000, 500, None, &ar, m), (one, one));
    let (tau, f) = air::slowing(one, 250, 500, None, &ar, m);
    assert_eq!(tau, nd("0.5"));
    assert_eq!(f, one.sub(nd("0.1"), m));
    // The creep case: τ = 0, f halved, then one step toward 0.
    let (tau, f) = air::slowing(nd("0.3"), 40, 500, None, &ar, m);
    assert_eq!(tau, zero);
    let half = nd("0.3").mul(nd("0.5"), m);
    assert_eq!(f, zero.max(half.sub(nd("0.1"), m)));
    // A stopped aircraft creeps: fₚ = 0.05, adjusted toward 0.
    assert_eq!(air::slowing(zero, 40, 500, None, &ar, m).1, zero);
    // No slowing with a slowdown distance of 0.
    assert_eq!(air::slowing(one, 10, 0, None, &ar, m).0, one);
}

// spec: S-0501 F5 (k grows by 1 for a dead aircraft and by 3 for a living one; h₁ = h − k)
#[test]
fn f5_crash_step() {
    assert_eq!(air::crash_step(0, 300, true), (1, 299));
    assert_eq!(air::crash_step(0, 300, false), (3, 297));
    assert_eq!(air::crash_step(4, 300, true), (5, 295));
}

// spec: S-0501 F6 (the strafing wait ⌊(R + 1024) / Vₜ⌋; Q-0635: Vₜ at least 1)
#[test]
fn f6_strafe_wait() {
    assert_eq!(air::strafe_wait(1280, 30), 76);
    assert_eq!(air::strafe_wait(1280, 0), 2304);
}

// spec: S-0501 F9 and quirks.pad_distance_overflow (on: the signed 32-bit square wraps; off: exact)
#[test]
fn f9_pad_distance_both_ways() {
    assert_eq!(air::pad_distance(300, 400, true), 250_000);
    assert_eq!(air::pad_distance(50_000, 0, true), -1_794_967_296);
    assert_eq!(air::pad_distance(50_000, 0, false), 2_500_000_000);
}

// spec: S-0501 F10 and Import mapping (P = ⌊r × 900⌋; ReloadRate absent 0.05; FlightLevel absent 500)
#[test]
fn f10_service_cadence() {
    let w = game(&rules());
    // The decimal 0.01 is read toward zero (S-0215 F1), so the product falls just below 9.
    assert_eq!(w.rules().air.service, 8);
    let w = game(&econ_common::ruleset());
    assert_eq!(w.rules().air.service, 45);
    assert_eq!(w.rules().air.flight_level, 500);
}

// spec: S-0501 claim 25 (with neither contact nor passengers: the pose-direction integer × 8192, modulo 65536)
#[test]
fn claim_25_landing_facing() {
    let mut r = rules();
    r.sim.general.economy.pose_dir = Some(11);
    let mut w = game(&r);
    let a = put(&mut w, Kind::Aircraft, "JET", 0, 0, 0);
    assert_eq!(air::landing_facing(&w, a), 3 * 8192);
}

// ---------------------------------------------------------------------------
// Flight

// spec: S-0501 claim 1 and Terms (a non-visiting aircraft put inside the usable area stands on the ground with fraction 0 and the object-present mark; a visiting one is placed at cruise height with fraction 1 and no mark)
#[test]
fn claim_1_placement() {
    let mut w = game(&rules());
    let a = put(&mut w, Kind::Aircraft, "JET", 0, 0, 0);
    assert_eq!(air::height(&w, a), 0);
    assert!(air::on_ground(&w, a));
    assert_eq!(st(&w, a).cur, glasnost_num::F64::ZERO);
    assert_eq!(layer_mark(&w, a), 1);
    assert!(!st(&w, a).visiting);
    // A type that is not selectable is marked visiting by its placement, after the height is chosen.
    let s = put(&mut w, Kind::Aircraft, "SPY", 0, 4, 0);
    assert_eq!(air::height(&w, s), 0);
    assert!(st(&w, s).visiting, "not selectable: visiting");
    // An aircraft marked visiting before placement is placed at cruise height.
    let ti = w.rules().index_of(Kind::Aircraft, "SPY").unwrap();
    let x = glasnost_sim::objects::create(&mut w, Kind::Aircraft, ti, Some(ALPHA));
    w.obj_mut(x).unwrap().air = Some(air::AirState {
        visiting: true,
        ..Default::default()
    });
    let p = glasnost_sim::world::Pos::centre(testkit::mid(SIZE, 0, 5), 0);
    glasnost_sim::objects::place(&mut w, x, p, 0);
    assert_eq!(air::height(&w, x), 500);
    assert_eq!(st(&w, x).cur, nd("1"));
    assert_eq!(layer_mark(&w, x), 0);
}

// spec: S-0501 claims 2 and 8 (take-off: the held height becomes the cruise height and the aircraft climbs at most 20 leptons a tick until it reaches it); S-0209 claim 1 (the mark is cleared when it leaves the ground)
#[test]
fn claim_2_climb_rate() {
    let (mut w, _k, a) = built("JET");
    let goal = testkit::mid(SIZE, 9, 9);
    click(&mut w, a, ClickOn::Cell(goal));
    let mut last = air::height(&w, a);
    let mut top = false;
    for _ in 0..200 {
        step(&mut w);
        let h = air::height(&w, a);
        assert!(h - last <= 20, "climbs at most 20: {h} after {last}");
        last = h;
        if h == 500 {
            top = true;
            break;
        }
    }
    assert!(top, "reaches cruise height: {}", describe(&w, a));
    assert!(!st(&w, a).ground);
    assert_eq!(st(&w, a).mark, None);
}

// spec: S-0501 claim 7 and F7 (a touchdown cell holding another object is refused: the aircraft rises by 10 leptons and takes off again; it never lands there)
#[test]
fn claim_7_refused_touchdown() {
    let mut w = game(&rules());
    ready(&mut w);
    let goal = testkit::mid(SIZE, 6, 0);
    testkit::spawn(&mut w, Kind::Vehicle, "VICTIM", 0, goal, 0);
    let a = put(&mut w, Kind::Aircraft, "SCOUT", 0, -2, 0);
    click(&mut w, a, ClickOn::Cell(goal));
    let mut rose = false;
    let mut last = air::height(&w, a);
    for _ in 0..1500 {
        step(&mut w);
        let h = air::height(&w, a);
        if h == 10 && last < 20 && st(&w, a).taking_off {
            rose = true;
        }
        last = h;
        assert!(
            !(st(&w, a).ground && w.obj(a).unwrap().pos.cell() == goal),
            "never lands on the occupied cell: {}",
            describe(&w, a)
        );
    }
    assert!(
        rose,
        "a refused touchdown rises 10 leptons: {}",
        describe(&w, a)
    );
}

// spec: S-0501 claim 9, S-0312 claim 16 (a shot-down aircraft stays in play, takes the three tumble draws and falls every tick; at impact it leaves play)
#[test]
fn claim_9_shot_down_crash() {
    let mut w = game(&rules());
    ready(&mut w);
    let a = lifted(&mut w, "JET", 0, 0);
    let taken = w.rng.taken();
    glasnost_sim::combat::damage::apply(&mut w, a, hit(10_000));
    assert!(w.obj(a).is_some(), "still in play");
    assert!(air::crashing(&w, a));
    assert!(w.rng.taken() >= taken + 3, "the three tumble draws");
    let mut last = air::height(&w, a);
    let gone = run_until(&mut w, 200, |w| {
        if w.obj(a).is_none() {
            return true;
        }
        let h = air::height(w, a);
        assert!(h <= last, "falls");
        last = h;
        false
    });
    assert!(gone, "leaves play at impact");
}

// spec: S-0501 claim 9 and S-0312 claim 16 (a crash started without a kill: the counter grows by 3; at impact it is destroyed by damage equal to its health)
#[test]
fn claim_9_living_crash() {
    let mut w = game(&rules());
    ready(&mut w);
    let a = lifted(&mut w, "SCOUT", 0, 0);
    assert!(air::start_crash(&mut w, a, true));
    step(&mut w);
    assert_eq!(st(&w, a).crash, Some(3));
    assert!(w.obj(a).unwrap().health > 0, "alive while it falls");
    assert!(run_until(&mut w, 200, |w| w
        .obj(a)
        .is_none_or(|o| o.health <= 0)));
}

// spec: S-0501 claims 10 and 34 (a visiting aircraft in retreat flies to a map-edge cell and leaves the map outside the usable area; Q-0634)
#[test]
fn claim_34_retreat_leaves_the_map() {
    // A map whose usable area leaves a band of legal cells around it.
    let mut m = testkit::empty_map(SIZE);
    let fp = glasnost_map::Footprint::new(SIZE, SIZE, [2, 2, SIZE - 4, SIZE - 4]);
    let mut cells = glasnost_map::CellGrid::new(fp);
    let all: Vec<_> = cells.iter().map(|(c, _)| c).collect();
    for c in all {
        if let (Some(cell), Some(old)) = (cells.get_mut(c), m.cells.get(c)) {
            *cell = old.clone();
        }
    }
    m.footprint = fp;
    m.cells = cells;
    let rules = std::sync::Arc::new(glasnost_sim::rules::SimRules::new(
        &rules(),
        "synthetic-test",
    ));
    let mut w = glasnost_sim::load::new_game(rules, &m, testkit::catalogue(), &testkit::setup(7))
        .expect("loads");
    ready(&mut w);
    let s = put(&mut w, Kind::Aircraft, "SPY", 0, 4, 0);
    force(&mut w, s, Behaviour::Retreat);
    let gone = run_until(&mut w, 3000, |w| w.obj(s).is_none());
    assert!(gone, "removed at the edge: {}", describe(&w, s));
}

// ---------------------------------------------------------------------------
// Attack runs

/// A world with `name` (house 0) on the ground and VICTIM (house 1).
fn attacker(r: &Ruleset, name: &str) -> (World, ObjectId, ObjectId) {
    let mut w = game(r);
    ready(&mut w);
    let v = put(&mut w, Kind::Vehicle, "VICTIM", 1, 8, 0);
    let a = put(&mut w, Kind::Aircraft, name, 0, -4, 0);
    (w, a, v)
}

// spec: S-0501 claim 15 (one round per run in which it fired, whatever the burst; firing itself does not change the count)
#[test]
fn claim_15_one_round_per_run() {
    let mut r = rules();
    r.sim.general.economy.curley_shuffle = Some(true);
    let (mut w, a, v) = attacker(&r, "JET2");
    let full = w.obj(v).unwrap().health;
    click(&mut w, a, ClickOn::Object(v));
    let mut seen = 3;
    let mut fired_at_3 = false;
    for _ in 0..3000 {
        step(&mut w);
        let ammo = w.obj(a).unwrap().combat.ammo;
        assert!(ammo == seen || ammo == seen - 1, "one round at a time");
        if ammo == 3 && w.obj(v).unwrap().health < full {
            fired_at_3 = true;
        }
        seen = ammo;
        if ammo == 1 {
            break;
        }
    }
    assert!(fired_at_3, "the target is hit before any round is spent");
    assert_eq!(seen, 1, "{}", describe(&w, a));
}

// spec: S-0501 claim 14 and quirks.aircraft_endless_refire (re-approach flag off: on, the run never spends its round; off, each burst-and-refire cycle ends a run)
#[test]
fn claim_14_endless_refire_both_ways() {
    for endless in [true, false] {
        let mut r = rules();
        r.globals.quirks.aircraft_endless_refire = endless;
        let (mut w, a, v) = attacker(&r, "JET");
        let full = w.obj(v).unwrap().health;
        click(&mut w, a, ClickOn::Object(v));
        testkit::run(&mut w, 1500);
        let ammo = w.obj(a).unwrap().combat.ammo;
        assert!(w.obj(v).unwrap().health < full);
        if endless {
            assert_eq!(ammo, 2, "never spent: {}", describe(&w, a));
            let h = w.obj(v).unwrap().health;
            testkit::run(&mut w, 300);
            assert!(w.obj(v).unwrap().health < h, "keeps firing");
        } else {
            assert!(ammo < 2, "runs end: {}", describe(&w, a));
        }
    }
}

// spec: S-0501 claim 19 (an aircraft that is not a fighter answers not facing beyond 2048 units; a fighter never does)
#[test]
fn claim_19_fighter_never_not_facing() {
    use glasnost_sim::combat::check::{Answer, check};
    let mut w = game(&rules());
    ready(&mut w);
    let v = put(&mut w, Kind::Vehicle, "VICTIM", 1, 0, 4);
    let j = put(&mut w, Kind::Aircraft, "JET", 0, 0, 0);
    let f = put(&mut w, Kind::Aircraft, "FIGHT", 0, 1, 0);
    let tg = Some(Target::Object(v));
    // Both face 0; the target lies the other way.
    assert_eq!(check(&w, j, tg, 0, false), Answer::NotFacing);
    assert_ne!(check(&w, f, tg, 0, false), Answer::NotFacing);
}

// spec: S-0501 claim 32 (before processing, a target whose in-the-air answer is yes is cleared)
#[test]
fn claim_32_in_air_target_cleared() {
    let mut w = game(&rules());
    ready(&mut w);
    let s = lifted(&mut w, "SCOUT", 0, 0);
    let j = put(&mut w, Kind::Aircraft, "JET", 0, 4, 4);
    assert!(glasnost_sim::combat::weapon::in_air(&w, s));
    assert!(!glasnost_sim::combat::weapon::in_air(&w, j));
    w.obj_mut(j).unwrap().target = Some(Target::Object(s));
    step(&mut w);
    assert_eq!(w.obj(j).unwrap().target, None);
}

// spec: S-0501 claim 31 (not ready with the run lock or in sticky)
#[test]
fn claim_31_readiness() {
    let mut w = game(&rules());
    let a = put(&mut w, Kind::Aircraft, "JET", 0, 0, 0);
    assert!(air::ready(&w, a));
    w.obj_mut(a).unwrap().air.as_mut().unwrap().run_lock = true;
    assert!(!air::ready(&w, a));
    w.obj_mut(a).unwrap().air.as_mut().unwrap().run_lock = false;
    force(&mut w, a, Behaviour::Sticky);
    assert!(!air::ready(&w, a));
}

// spec: S-0501 claim 37 and quirks.aircraft_smoke_scenario_draw (a damaged aircraft above the ground takes one U(0, 99) draw a tick on the scenario stream; off, none)
#[test]
fn claim_37_smoke_draw_both_ways() {
    let mut taken = Vec::new();
    for on in [true, false] {
        let mut r = rules();
        r.globals.quirks.aircraft_smoke_scenario_draw = on;
        let mut w = game(&r);
        ready(&mut w);
        let a = lifted(&mut w, "SCOUT", 0, 0);
        w.obj_mut(a).unwrap().health = 20;
        let before = w.rng.taken();
        step(&mut w);
        taken.push(w.rng.taken() - before);
    }
    assert_eq!(taken[1], 0, "off: no draw");
    assert!(
        taken[0] >= 1,
        "on: the chance draw (with any rejected raw values)"
    );
}

// ---------------------------------------------------------------------------
// Pads

// spec: S-0501 claims 21 to 23 (the nearest accepting pad; a primary pad replaces a nearer earlier one; an enemy, switched-off or full pad does not accept)
#[test]
fn claims_21_to_23_pad_choice() {
    let mut w = game(&rules());
    ready(&mut w);
    let near = building(&mut w, "PADB", 0, 2, 0);
    let far = building(&mut w, "PADB", 0, -8, 0);
    let enemy = building(&mut w, "PADB", 1, 0, 4);
    let a = put(&mut w, Kind::Aircraft, "JET", 0, 4, 0);
    assert_eq!(air::find_pad(&w, a), Some(near));
    assert!(!air::accepts(&w, enemy, a));
    w.obj_mut(far).unwrap().econ.building.primary = true;
    assert_eq!(air::find_pad(&w, a), Some(far), "primary replaces");
    w.obj_mut(far).unwrap().econ.building.primary = false;
    econ_common::switch(&mut w, near, false);
    assert!(!air::accepts(&w, near, a));
    assert_eq!(air::find_pad(&w, a), Some(far));
    econ_common::switch(&mut w, near, true);
    // A full pad: another aircraft holds its only place.
    let b = put(&mut w, Kind::Aircraft, "JET", 0, 5, 0);
    assert!(air::request_contact(&mut w, b, near));
    assert!(!air::accepts(&w, near, a));
    assert!(air::accepts(&w, near, b), "its own place is free for it");
}

/// A JET docked on its new APAD with `ammo` rounds, in sleep, with the
/// pad servicing.
fn serviced(r: &Ruleset, ammo: i32) -> (World, ObjectId, ObjectId) {
    let (mut w, k, a) = built_with(r, "JET");
    w.obj_mut(a).unwrap().combat.ammo = ammo;
    force(&mut w, a, Behaviour::Sleep);
    let t = w.tick;
    w.obj_mut(k).unwrap().air = Some(air::AirState {
        service: Some(t),
        ..Default::default()
    });
    (w, k, a)
}

// spec: S-0501 claims 26 and 28 and quirks.aircraft_half_reload_with_target (on: with a target it is reloaded only up to ⌊A / 2⌋; off: to full)
#[test]
fn claim_28_half_reload_both_ways() {
    for half in [true, false] {
        let mut r = rules();
        r.globals.quirks.aircraft_half_reload_with_target = half;
        let (mut w, _k, a) = serviced(&r, 0);
        let v = put(&mut w, Kind::Vehicle, "VICTIM", 1, 9, 9);
        w.obj_mut(a).unwrap().target = Some(Target::Object(v));
        testkit::run(&mut w, 40);
        let ammo = w.obj(a).unwrap().combat.ammo;
        assert_eq!(
            ammo,
            if half { 1 } else { 2 },
            "half {half}: {}",
            describe(&w, a)
        );
    }
}

// spec: S-0501 claim 26 and F10 (one round per service step every P ticks; at full ammunition and health it is released into guard)
#[test]
fn claim_26_service_and_release() {
    let (mut w, k, a) = serviced(&rules(), 0);
    testkit::run(&mut w, 1);
    assert_eq!(w.obj(a).unwrap().combat.ammo, 1);
    testkit::run(&mut w, 7);
    assert_eq!(w.obj(a).unwrap().combat.ammo, 1, "every 8 ticks");
    testkit::run(&mut w, 1);
    assert_eq!(w.obj(a).unwrap().combat.ammo, 2);
    testkit::run(&mut w, 9);
    assert_eq!(w.obj(a).unwrap().beh.current, Some(Behaviour::Guard));
    assert_eq!(w.obj(a).unwrap().contact, Some(k), "it stays on its pad");
}

// spec: S-0501 claim 27 (a docked aircraft at full ammunition gets repair steps that cost money until full health, then is released)
#[test]
fn claim_27_repair_on_pad() {
    let (mut w, _k, a) = serviced(&rules(), 2);
    set_credits(&mut w, 0, 100_000);
    w.obj_mut(a).unwrap().health = 100;
    let credits = w.houses[0].credits;
    let healed = run_until(&mut w, 3000, |w| w.obj(a).unwrap().health == 200);
    assert!(healed, "{}", describe(&w, a));
    assert!(w.houses[0].credits < credits, "repair costs money");
    testkit::run(&mut w, 10);
    assert_eq!(w.obj(a).unwrap().beh.current, Some(Behaviour::Guard));
}

// spec: S-0407 claim 16, S-0501 claim 30 and quirks.dock_bound_aircraft_ignores_limit (on: a pad-bound type's queue test ignores its own build limit; off: both limits apply)
#[test]
fn dock_bound_limit_both_ways() {
    for ignores in [true, false] {
        let mut r = econ_common::ruleset();
        r.globals.quirks.dock_bound_aircraft_ignores_limit = ignores;
        let b = r.sim.buildings.get_mut("AIRPAD").unwrap();
        b.helipad = Some(true);
        b.econ.number_of_docks = Some(3);
        r.registries
            .get_mut("aircraft")
            .unwrap()
            .push("PADA".into());
        let mut p = r.sim.aircraft["PLANE"].clone();
        p.econ.airport_bound = Some(true);
        p.econ.build_limit = Some(1);
        r.sim.aircraft.insert("PADA".into(), p);
        r.sim.general.economy.pad_aircraft = Some(vec!["PADA".into()]);
        let mut w = game(&r);
        building(&mut w, "AIRPAD", 0, 0, 0);
        ready(&mut w);
        set_credits(&mut w, 0, 100_000);
        let what = t(&w, Kind::Aircraft, "PADA");
        assert!(issue(&mut w, 0, Order::Build { what }));
        step(&mut w);
        issue(&mut w, 0, Order::Build { what });
        step(&mut w);
        let queued = econ_common::line(&w, 0, what).unwrap().queue.len();
        assert_eq!(queued, usize::from(ignores), "ignores {ignores}");
    }
}

// spec: S-0501 claims 29 and 33 (an unarmed aircraft in guard at cruise height with no destination is given its own cell and lands there)
#[test]
fn claim_29_unarmed_guard_lands() {
    let mut w = game(&rules());
    ready(&mut w);
    let a = lifted(&mut w, "SCOUT", 0, 0);
    w.obj_mut(a).unwrap().dest = None;
    force(&mut w, a, Behaviour::Guard);
    let landed = run_until(&mut w, 1500, |w| air::on_ground(w, a));
    assert!(landed, "{}", describe(&w, a));
}

// spec: S-0501 State and ordering (all aircraft state is saved: a save and load in mid-run gives the same state hash, and the two worlds stay equal)
#[test]
fn save_load_mid_run() {
    let mut r = rules();
    r.sim.general.economy.curley_shuffle = Some(true);
    let (mut w, a, v) = attacker(&r, "JET");
    click(&mut w, a, ClickOn::Object(v));
    assert!(run_until(&mut w, 400, |w| air::height(w, a) > 200));
    let bytes = glasnost_sim::save::save(&w);
    let mut w2 =
        glasnost_sim::save::load(&bytes, w.rules_arc(), testkit::catalogue()).expect("loads");
    assert_eq!(
        glasnost_sim::save::state_hash(&w),
        glasnost_sim::save::state_hash(&w2)
    );
    for _ in 0..600 {
        step(&mut w);
        step(&mut w2);
        assert_eq!(
            glasnost_sim::save::state_hash(&w),
            glasnost_sim::save::state_hash(&w2),
            "t = {}",
            w.tick
        );
    }
}
