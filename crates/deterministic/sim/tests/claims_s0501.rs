//! Claim tests for S-0501 (aircraft: flight, attack runs, pads) on
//! synthetic data: invented aircraft, pad, weapon and carrier types, none
//! of them stock values.

mod common;
mod econ_common;

use econ_common::{SIZE, building, dec, factory, game, issue, ready, set_credits, t, wheel};
use glasnost_num::geom::{Facing, direction, root_floor};
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{LandRow, LandType, Locomotor, ProjectileType, UnitType, WeaponType};
use glasnost_sim::air;
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::targeting::{self, ClickOn, Modifiers};
use glasnost_sim::testkit;
use glasnost_sim::world::Pos;
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
    built_in(game(r), name)
}

/// [`built_with`] in a given game.
fn built_in(mut w: World, name: &str) -> (World, ObjectId, ObjectId) {
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

// spec: S-0501 claims 8, 12 to 15, 17, 18, 21 to 26 and table T1 (end to end: a docked aircraft ordered by a click to attack an enemy vehicle takes off, makes its runs, damages the target, returns to its pad, lands and rearms); the close approach of claim 42
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
    // spec: S-0501 claim 24 (enter's docking steps move it horizontally to the docking position before touchdown, at the landing height; after touchdown it reports entered: sleep)
    let back = run_until(&mut w, 3000, |w| {
        air::on_ground(w, a)
            && w.obj(a).unwrap().contact == Some(k)
            && w.obj(a).unwrap().beh.current == Some(Behaviour::Sleep)
    });
    assert!(back, "lands on its pad: {}", describe(&w, a));
    let (p, _) = glasnost_sim::econ::release::pad_position(&w, k, a);
    assert_eq!(w.obj(a).unwrap().pos, p);
    let full_ammo = run_until(&mut w, 500, |w| w.obj(a).unwrap().combat.ammo == 2);
    assert!(full_ammo, "rearms: {}", describe(&w, a));
}

// spec: S-0501 claim 47 (a) (leaving a pad is a take-off: tracked from it) and (b) (tracked until the accepted touchdown on the pad, through the landing; an aircraft at rest on its pad, built there or docked again, is never tracked)
#[test]
fn claim_47_tracked_from_the_pad_to_the_pad() {
    use glasnost_sim::combat::flying;
    let mut r = rules();
    r.sim.general.economy.curley_shuffle = Some(true);
    let (mut w, k, a) = built_with(&r, "JET");
    for _ in 0..10 {
        step(&mut w);
        assert!(
            !flying::tracked(&w, a),
            "built, docked: {}",
            describe(&w, a)
        );
    }
    let v = put(&mut w, Kind::Vehicle, "VICTIM", 1, 10, 0);
    click(&mut w, a, ClickOn::Object(v));
    assert!(run_until(&mut w, 200, |w| !st(w, a).ground));
    assert!(
        flying::tracked(&w, a),
        "from leaving the pad: {}",
        describe(&w, a)
    );
    assert!(run_until(&mut w, 3000, |w| w.obj(a).unwrap().combat.ammo == 0));
    let mut landing = 0;
    let down = run_until(&mut w, 3000, |w| {
        let s = st(w, a);
        if s.ground {
            return true;
        }
        assert!(flying::tracked(w, a), "in flight: {}", describe(w, a));
        if s.landing {
            landing += 1;
        }
        false
    });
    assert!(down, "{}", describe(&w, a));
    assert!(landing > 1, "a landing of several updates");
    assert_eq!(w.obj(a).unwrap().contact, Some(k), "on its pad");
    assert!(
        !flying::tracked(&w, a),
        "touchdown on the pad: {}",
        describe(&w, a)
    );
    for _ in 0..30 {
        step(&mut w);
        assert!(!flying::tracked(&w, a), "docked: {}", describe(&w, a));
    }
}

// spec: S-0501 claim 24 and S-0524 claim 14 (ordinary pad landing, docking and departure add no entered-by raise on the pad's object tag and no destroyed-by-anything raise on the aircraft's own tag; C16)
#[test]
fn claim_24_pad_docking_raises_no_tag() {
    let mut r = rules();
    r.sim.general.economy.curley_shuffle = Some(true);
    let w = testkit::game_with_triggers(&r, SIZE, 7, testkit::entry_triggers());
    let (mut w, k, a) = built_in(w, "JET");
    glasnost_sim::triggers::object_tag(&mut w, k, Some(b"ENT"));
    glasnost_sim::triggers::object_tag(&mut w, a, Some(b"DIE"));
    let v = put(&mut w, Kind::Vehicle, "VICTIM", 1, 10, 0);
    let f0 = w.triggers.fired;
    click(&mut w, a, ClickOn::Object(v));
    assert!(
        run_until(&mut w, 200, |w| air::height(w, a) > 0),
        "takes off"
    );
    assert!(
        run_until(&mut w, 3000, |w| w.obj(a).unwrap().combat.ammo == 0),
        "spends its rounds"
    );
    assert!(
        run_until(&mut w, 3000, |w| air::on_ground(w, a)
            && w.obj(a).unwrap().contact == Some(k)
            && w.obj(a).unwrap().beh.current == Some(Behaviour::Sleep)),
        "docked: {}",
        describe(&w, a)
    );
    testkit::run(&mut w, 100);
    assert_eq!(w.triggers.fired, f0, "no raise reached a firing");
    assert!(!w.triggers.variables[0] && !w.triggers.variables[1]);
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

// spec: S-0501 claims 35 and 36 and table T4, S-0506 claims 6 to 9 (end to end: a carrier with slot aircraft ordered to attack launches them; they attack, damage the target and come back); the close approach of claim 42
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

// spec: S-0501 claim 47 (a) (a slot aircraft launched on the ground inside the usable area, claim 35, is first tracked when its take-off starts) and (b) (going inside its carrier ends the tracking)
#[test]
fn claim_47_a_launched_slot_aircraft_is_tracked_from_its_take_off() {
    use glasnost_sim::combat::flying;
    let mut w = game(&rules());
    ready(&mut w);
    let c = put(&mut w, Kind::Vehicle, "CARRIER", 0, 0, 0);
    let v = put(&mut w, Kind::Vehicle, "VICTIM", 1, 8, 0);
    testkit::run(&mut w, 2);
    click(&mut w, c, ClickOn::Object(v));
    let mut seen = std::collections::BTreeSet::new();
    let mut flew = 0;
    for _ in 0..6000 {
        step(&mut w);
        let on: Vec<ObjectId> = w
            .objects
            .values()
            .filter(|o| o.kind == Kind::Aircraft && o.on_map)
            .map(|o| o.id)
            .collect();
        for &h in &on {
            seen.insert(h);
            let ground = st(&w, h).ground;
            assert_eq!(flying::tracked(&w, h), !ground, "{}", describe(&w, h));
            if !ground {
                flew += 1;
            }
        }
        if on.is_empty() && !seen.is_empty() {
            break;
        }
    }
    assert!(seen.len() == 2 && flew > 100, "both launched and flew");
    for &h in &seen {
        assert!(!flying::tracked(&w, h), "inside its carrier");
    }
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
    assert_eq!(air::slowing(one, 1000, 500, &ar, m), (one, one));
    let (tau, f) = air::slowing(one, 250, 500, &ar, m);
    assert_eq!(tau, nd("0.5"));
    assert_eq!(f, one.sub(nd("0.1"), m));
    // The creep case: τ = 0, f halved, then one step toward 0.
    let (tau, f) = air::slowing(nd("0.3"), 40, 500, &ar, m);
    assert_eq!(tau, zero);
    let half = nd("0.3").mul(nd("0.5"), m);
    assert_eq!(f, zero.max(half.sub(nd("0.1"), m)));
    // A stopped aircraft creeps: fₚ = 0.05, adjusted toward 0.
    assert_eq!(air::slowing(zero, 40, 500, &ar, m).1, zero);
}

// spec: S-0501 claim 5 and F4 (S = 0: the ordinary target is 1 at every distance, d = 0 included, without halving; the distance clamp and the 0.1 adjustment still apply, so at d = 0 the fraction is 0.1; a negative S takes the quotient: below 86 leptons halving and τ = 0, at 86 or more τ = 0.1)
#[test]
fn claim_5_signed_slowdown_distance() {
    let w = game(&rules());
    let ar = w.rules().air.clone();
    let m = w.rules().mode;
    let one = glasnost_num::F64::from_i64(1, m);
    let zero = glasnost_num::F64::ZERO;
    // S = 0, positive distance: target 1 and min(1, f + 0.1), no halving.
    assert_eq!(
        air::slowing(nd("0.3"), 10, 0, &ar, m),
        (one, nd("0.3").add(nd("0.1"), m))
    );
    assert_eq!(air::slowing(one, 40, 0, &ar, m), (one, one));
    // S = 0, distance 0: the clamp gives 0, the adjustment raises it to 0.1.
    assert_eq!(air::slowing(one, 0, 0, &ar, m), (one, nd("0.1")));
    // S = −500 below 86 leptons: τ = 0 and the fraction halved, then lowered by 0.1.
    let (tau, f) = air::slowing(nd("0.6"), 40, -500, &ar, m);
    assert_eq!(tau, zero);
    assert_eq!(f, nd("0.6").mul(nd("0.5"), m).sub(nd("0.1"), m));
    // S = −500 at 86 leptons and beyond: τ = 0.1.
    assert_eq!(air::slowing(nd("0.6"), 86, -500, &ar, m).0, nd("0.1"));
    assert_eq!(air::slowing(nd("0.6"), 2000, -500, &ar, m).0, nd("0.1"));
}

// spec: S-0501 F5 (k grows by 1 for a dead aircraft and by 3 for a living one; h₁ = h − k)
#[test]
fn f5_crash_step() {
    assert_eq!(air::crash_step(0, 300, true), (1, 299));
    assert_eq!(air::crash_step(0, 300, false), (3, 297));
    assert_eq!(air::crash_step(4, 300, true), (5, 295));
}

// spec: S-0501 F6 (the strafing wait ⌊(R + 1024) / Vₜ⌋ with the converted speed; our engine's departure: a converted speed of 0 gives 0)
#[test]
fn f6_strafe_wait() {
    assert_eq!(air::strafe_wait(1280, 30), 76);
    assert_eq!(air::strafe_wait(1280, 0), 0);
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

// spec: S-0501 claim 7 and F7 (a touchdown cell holding another object is refused: the aircraft rises by 10 leptons and takes off again; it never lands there). The blocker arrives after the movement point is accepted, so claim 40's substitution saw a free cell.
#[test]
fn claim_7_refused_touchdown() {
    let mut w = game(&rules());
    ready(&mut w);
    let goal = testkit::mid(SIZE, 6, 0);
    let a = put(&mut w, Kind::Aircraft, "SCOUT", 0, -2, 0);
    click(&mut w, a, ClickOn::Cell(goal));
    assert!(run_until(&mut w, 200, |w| air::height(w, a) > 100));
    assert_eq!(w.obj(a).unwrap().dest, Some(Target::Cell(goal)), "retained");
    testkit::spawn(&mut w, Kind::Vehicle, "VICTIM", 0, goal, 0);
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

// spec: S-0501 claims 10 and 34 (a visiting aircraft in retreat flies to a cell on its house's arrival edge and leaves the map outside the usable area)
/// A game of `r` on a map whose usable area leaves a band of legal cells
/// around it, so that a position can leave the usable area on the map.
fn banded_game(r: &Ruleset) -> World {
    let mut m = testkit::empty_map(SIZE);
    let fp = glasnost_map::Footprint::new(SIZE, SIZE, [2, 2, SIZE - 4, SIZE - 4]);
    let mut cells = glasnost_map::CellGrid::new(fp);
    let all: Vec<_> = cells.iter().map(|(c, _)| c).collect();
    for c in all {
        if let (Some(cell), Some(old)) = (cells.get_mut(c), m.cells.get(c)) {
            *cell = *old;
        }
    }
    m.footprint = fp;
    m.cells = cells;
    let rules = std::sync::Arc::new(glasnost_sim::rules::SimRules::new(r, "synthetic-test"));
    glasnost_sim::load::new_game(rules, &m, testkit::catalogue(), &testkit::setup(7))
        .expect("loads")
}

#[test]
fn claim_34_retreat_leaves_the_map() {
    let mut w = banded_game(&rules());
    ready(&mut w);
    let s = put(&mut w, Kind::Aircraft, "SPY", 0, 4, 0);
    force(&mut w, s, Behaviour::Retreat);
    let gone = run_until(&mut w, 3000, |w| w.obj(s).is_none());
    assert!(gone, "removed at the edge: {}", describe(&w, s));
}

// spec: S-0501 claim 32 (an approaching aircraft whose destination is lost falls back to rest) and claim 40 (a slot aircraft's move toward its carrier is retained: F7 and the reservation test accept a spawned lander over a spawner); the carrier is removed, the aircraft drops it and leaves move
#[test]
fn claim_32_lost_destination_falls_back() {
    let mut w = game(&rules());
    ready(&mut w);
    let k = put(&mut w, Kind::Vehicle, "CARRIER", 0, 10, 0);
    let a = lifted(&mut w, "HORNET", -4, 0);
    assert!(orders::issue(
        &mut w,
        ALPHA,
        Order::Behaviour {
            actor: a,
            behaviour: Behaviour::Move,
            attack_move: false,
            target: None,
            dest: Some(Target::Object(k)),
        }
    ));
    testkit::run(&mut w, 5);
    assert_eq!(w.obj(a).unwrap().dest, Some(Target::Object(k)));
    glasnost_sim::objects::remove(&mut w, k);
    let back = run_until(&mut w, 200, |w| {
        w.obj(a).is_none_or(|o| {
            o.dest != Some(Target::Object(k)) && o.beh.current != Some(Behaviour::Move)
        })
    });
    assert!(back, "falls back to rest: {}", describe(&w, a));
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

// spec: S-0501 claim 15 (one round per run in which it fired, whatever the burst; firing itself does not change the count); the close approach of claim 42
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

// spec: S-0501 claim 14 and quirks.aircraft_endless_refire; the close approach of claim 42 (re-approach flag off: on, the run never spends its round; off, each burst-and-refire cycle ends a run)
#[test]
fn claim_14_endless_refire_both_ways() {
    for endless in [true, false] {
        let mut r = rules();
        r.globals.quirks.aircraft_endless_refire = endless;
        let (mut w, a, v) = attacker(&r, "JET");
        let full = w.obj(v).unwrap().health;
        click(&mut w, a, ClickOn::Object(v));
        run_until(&mut w, 1500, |_| false);
        let ammo = w.obj(a).unwrap().combat.ammo;
        assert!(w.obj(v).unwrap().health < full);
        if endless {
            assert_eq!(ammo, 2, "never spent: {}", describe(&w, a));
            let h = w.obj(v).unwrap().health;
            run_until(&mut w, 300, |_| false);
            assert!(w.obj(v).unwrap().health < h, "keeps firing");
        } else {
            assert!(ammo < 2, "runs end: {}", describe(&w, a));
        }
    }
}

// spec: S-0501 claim 19 and T1, S-0204 claims 4 and 8, S-0310 F1 (a non-strafer attacking a 2 × 2 building turns toward its reference point, the point the facing check measures to, so its run reaches a shot); the close approach of claim 42
#[test]
fn attack_on_a_large_building_fires() {
    let mut r = rules();
    r.sim.general.economy.curley_shuffle = Some(true);
    let mut w = game(&r);
    ready(&mut w);
    let b = building(&mut w, "PADB", 1, 8, 0);
    let a = put(&mut w, Kind::Aircraft, "JET", 0, -4, 0);
    let full = w.obj(b).unwrap().health;
    click(&mut w, a, ClickOn::Object(b));
    let hit = run_until(&mut w, 2000, |w| w.obj(b).unwrap().health < full);
    assert!(hit, "damages the building: {}", describe(&w, a));
    let spent = run_until(&mut w, 3000, |w| w.obj(a).unwrap().combat.ammo == 0);
    assert!(spent, "spends its rounds: {}", describe(&w, a));
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

// spec: S-0501 Edge cases (a pad with no docking-place key, or with the key set to 0, has one contact place: an aircraft built there docks in it)
#[test]
fn pad_without_docking_places_has_one_place() {
    for docks in [None, Some(0)] {
        let mut r = rules();
        r.sim
            .buildings
            .get_mut("APAD")
            .unwrap()
            .econ
            .number_of_docks = docks;
        let (w, k, a) = built_with(&r, "JET");
        assert_eq!(
            glasnost_sim::econ::release::contact_places(&w, k),
            1,
            "{docks:?}"
        );
        assert_eq!(w.obj(a).unwrap().contact, Some(k), "{docks:?}");
        assert!(air::on_ground(&w, a), "{docks:?}");
    }
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

// ---------------------------------------------------------------------------
// Revision 7 claims

// ---------------------------------------------------------------------------
// Revision 9: the close approach and guidance facings (claim 42, AC24)

/// The truncated table root of the planar distance (S-0204 claim 5).
fn guide_dist(w: &World, a: Pos, b: Pos) -> i32 {
    let q = w.rules().quirks;
    let (dx, dy) = (i64::from(a.x - b.x), i64::from(a.y - b.y));
    root_floor(
        (dx * dx + dy * dy) as u64,
        q.table_root_distance,
        w.rules().mode,
    ) as i32
}

/// The bearing of S-0204 claim 8 from `from` to `to`.
fn bearing(w: &World, from: Pos, to: Pos) -> Facing {
    direction(
        i64::from(to.x - from.x),
        i64::from(to.y - from.y),
        w.rules().quirks.table_direction,
    )
}

/// The flight point claim 42's guidance steers by at the end of this
/// tick's movement, when guidance runs: a living aircraft in flight,
/// neither landing nor taking off, without the run lock; its destination,
/// or after that was cleared in *attack* with a target, the point of its
/// unfinished flight.
fn guided_point(w: &World, a: ObjectId) -> Option<Pos> {
    let o = w.obj(a)?;
    let s = st(w, a);
    if s.landing || s.taking_off || s.ground || s.run_lock || air::height(w, a) <= 0 {
        return None;
    }
    let attack =
        o.beh.current == Some(Behaviour::Attack) || o.beh.queued == Some(Behaviour::Attack);
    let d = match o.dest {
        Some(d) => d,
        None if attack && o.target.is_some() && s.unfinished => s.flying_to?,
        None => return None,
    };
    Some(air::flight_point(w, a, d))
}

/// What one close approach of AC24 shows.
#[derive(Default, Debug)]
struct Approach {
    /// The first tick, after flying, with fraction 0: its post-step
    /// distance to the destination point, whether the destination is
    /// still set, and the position.
    zero: Option<(i32, bool, Pos)>,
    /// The destination and flight-point target at the first tick in *firing*.
    firing: Option<(Option<Target>, Option<Target>)>,
    /// The position when the target is first damaged.
    shot: Option<Pos>,
    /// Ticks in which guidance aimed at the target (below 256 leptons).
    near_aims: i32,
    /// The current fraction is positive at the end.
    positive_at_end: bool,
    /// At the end: the position, the distance to the flight point, the
    /// body facing, the current fraction, and for how many ticks the
    /// position, fraction and body facing have not changed.
    end: Option<(Pos, i32, Facing, glasnost_num::F64, i32)>,
    /// The ticks spent in *approaching* with a positive current fraction
    /// whose F3 step length is 0 leptons, while at least 16 leptons from
    /// the flight point.
    zero_steps: i32,
    /// The first tick counted in `zero_steps`, and the first tick in *firing*.
    first_zero_step: Option<i32>,
    firing_tick: Option<i32>,
    /// The flight point at the end.
    point: Option<Pos>,
    /// In each tick ending in *approaching* whose fraction gives the
    /// next step 1 lepton: the body facing and the offset (x, y) from
    /// the flight point.
    unit: Vec<(Facing, i32, i32)>,
}

/// Where an AC24 run starts relative to its flight point.
#[derive(Clone, Copy, Debug)]
enum Start {
    /// The attacker's placement of [`attacker`], 12 cells west of the target.
    Placed,
    /// `cells` cells from the flight point toward the map direction
    /// (dx, dy), and `side` leptons off the point's row or column (north
    /// or west negative), measured from the logged flight point (Q-0653).
    Axis {
        dx: i32,
        dy: i32,
        cells: i32,
        side: i32,
    },
}

/// Moves the airborne `a` to `p` (x and y only), its body facing set to
/// `body`.
fn relocate(w: &mut World, a: ObjectId, p: Pos, body: Facing) {
    let from = w.obj(a).unwrap().pos;
    let layer = glasnost_sim::world::Layer::Ground;
    w.cells.leave(from.cell(), layer, a);
    let o = w.obj_mut(a).unwrap();
    o.pos.x = p.x;
    o.pos.y = p.y;
    o.body = glasnost_sim::turning::Turning::at(body);
    w.cells.enter(p.cell(), layer, a, false);
}

/// The world's rounding mode.
fn mode() -> glasnost_num::Rounding {
    game(&rules()).rules().mode
}

/// AC24: JET with raw speed `speed` (converted ⌊256 × speed / 100⌋, S-0207
/// F1) and slowdown distance `s` attacks VICTIM from `start`, with
/// `quirks` applied to the switches; checks claim 42's guidance requests
/// in every tick before the first shot.
fn close_approach_with(
    speed: i32,
    s: i32,
    start: Start,
    quirks: impl FnOnce(&mut glasnost_rules::quirks::Quirks),
) -> Approach {
    let mut r = rules();
    r.sim.general.economy.curley_shuffle = Some(true);
    quirks(&mut r.globals.quirks);
    let j = r.sim.aircraft.get_mut("JET").unwrap();
    j.speed = Some(speed);
    j.slowdown_distance = Some(s);
    let (mut w, a, v) = attacker(&r, "JET");
    let full = w.obj(v).unwrap().health;
    click(&mut w, a, ClickOn::Object(v));
    if let Start::Axis {
        dx,
        dy,
        cells,
        side,
    } = start
    {
        // In cruise flight, after take-off.
        assert!(run_until(&mut w, 200, |w| {
            let s = st(w, a);
            s.stage == air::Stage::Approaching
                && w.obj(a).unwrap().dest.is_some()
                && !s.taking_off
                && air::height(w, a) == s.held
        }));
        let p = air::flight_point(&w, a, w.obj(a).unwrap().dest.unwrap());
        // spec: S-0501 AC24 (b) (the start along one axis, the side distance across it); S-0204 claim 2 (256 leptons per cell)
        let (sx, sy) = if dx != 0 { (0, side) } else { (side, 0) };
        let at = Pos::new(p.x + dx * cells * 256 + sx, p.y + dy * cells * 256 + sy, 0);
        let body = bearing(&w, at, p);
        relocate(&mut w, a, at, body);
    }
    let zero = glasnost_num::F64::ZERO;
    let mode = w.rules().mode;
    let v_conv = speed * 256 / 100; // spec: S-0207 F1 (⌊256 × speed / 100⌋)
    let mut out = Approach::default();
    let mut flown = false;
    let mut prev_body = w.obj(a).unwrap().body;
    let mut last: Option<(Pos, Facing, glasnost_num::F64)> = None;
    let mut still = 0;
    for _ in 0..2000 {
        step(&mut w);
        let o = w.obj(a).unwrap().clone();
        let s = st(&w, a);
        if w.obj(v).unwrap().health < full {
            out.shot = Some(o.pos);
            break;
        }
        // spec: S-0501 claim 42 (guidance after the movement tick: the body toward the flight point; the aiming facing toward the target below 256 leptons with a target and ammunition, otherwise toward the flight point)
        if let Some(p) = guided_point(&w, a) {
            let b = bearing(&w, o.pos, p);
            assert_eq!(
                o.body.target,
                b,
                "body toward the spot: {}",
                describe(&w, a)
            );
            let aim = if guide_dist(&w, o.pos, p) < 256 {
                out.near_aims += 1;
                assert!(o.combat.ammo > 0);
                let tp = glasnost_sim::combat::range::target_point(&w, o.target.unwrap());
                bearing(&w, o.pos, tp)
            } else {
                b
            };
            assert_eq!(o.turret.target, aim, "aiming facing: {}", describe(&w, a));
            // spec: S-0501 claim 42 (in firing the body request toward the target is replaced in the same update and leaves no trace)
            if s.stage == air::Stage::Firing && prev_body.target == b {
                assert_eq!(o.body, prev_body, "no trace: {}", describe(&w, a));
            }
        }
        prev_body = o.body;
        if s.cur.gt(zero) {
            flown = true;
        }
        let point = o.dest.or(s.flying_to).map(|d| air::flight_point(&w, a, d));
        if flown && out.zero.is_none() && s.cur.eq_num(zero) {
            let p = point.expect("a flight point");
            out.zero = Some((guide_dist(&w, o.pos, p), o.dest.is_some(), o.pos));
        }
        if s.stage == air::Stage::Firing && out.firing.is_none() {
            out.firing = Some((o.dest, s.flying_to));
            out.firing_tick = Some(w.tick);
        }
        let body = o.body.value(w.tick);
        if let Some(p) = point {
            let d = guide_dist(&w, o.pos, p);
            // spec: S-0501 F3 (s = ⌊V × f⌋) and claim 42 (at least 16 leptons from the point)
            if s.stage == air::Stage::Approaching
                && s.cur.gt(zero)
                && air::step_length(v_conv, s.cur, mode) == 0
                && d >= 16
            {
                out.zero_steps += 1;
                out.first_zero_step.get_or_insert(w.tick);
            }
            if s.stage == air::Stage::Approaching
                && air::step_length(v_conv, s.cur, mode) == 1
                && d < 1000
            {
                out.unit.push((body, o.pos.x - p.x, o.pos.y - p.y));
            }
            out.point = Some(p);
            let now = (o.pos, body, s.cur);
            still = if last.is_some_and(|l| l.0 == now.0 && l.1 == now.1 && l.2.eq_num(now.2)) {
                still + 1
            } else {
                0
            };
            last = Some(now);
            out.end = Some((o.pos, d, body, s.cur, still));
        }
        out.positive_at_end = s.cur.gt(zero);
    }
    out
}

fn close_approach(speed: i32, s: i32, start: Start) -> Approach {
    close_approach_with(speed, s, start, |_| {})
}

/// AC24 (b)'s starts by name: 20 cells from the flight point along one
/// axis and 11 leptons off its row or column (Q-0653).
fn ac24_b(name: &str) -> Start {
    let (dx, dy, side) = match name {
        "b1" => (-1, 0, -11), // from the west, north of the row
        "b2" => (-1, 0, 11),  // from the west, south of the row
        "b3" => (0, -1, -11), // from the north, west of the column
        "b4" => (0, -1, 11),  // from the north, east of the column
        "b5n" => (1, 0, -11), // from the east, north of the row
        "b5s" => (1, 0, 11),  // from the east, south of the row
        "b6w" => (0, 1, -11), // from the south, west of the column
        "b6e" => (0, 1, 11),  // from the south, east of the column
        _ => unreachable!(),
    };
    Start::Axis {
        dx,
        dy,
        cells: 20,
        side,
    }
}

// spec: S-0501 claims 5 and 42, F4 and T1 (AC24 variant a, converted speed 35 and S = 500: the ordinary attacker slows to fraction 0 at a post-step distance below min(86, S / 10) and above 16 leptons, moves to firing with its destination kept, and fires from where it hovers; the body never turns to the target, the aiming facing turns to it only below 256 leptons)
#[test]
fn claim_42_close_approach_hovers_short_and_fires() {
    let x = close_approach(14, 500, Start::Placed);
    let (d, kept, at) = x
        .zero
        .unwrap_or_else(|| panic!("reaches fraction 0: {x:?}"));
    assert!(d > 16 && d < 50, "{x:?}");
    assert!(kept, "the destination is kept: {x:?}");
    let (dest, _) = x.firing.unwrap_or_else(|| panic!("firing: {x:?}"));
    assert!(
        dest.is_some(),
        "firing with the destination still set: {x:?}"
    );
    assert_eq!(x.shot, Some(at), "the shot leaves from the hover: {x:?}");
    assert!(x.near_aims > 0, "{x:?}");
}

// spec: S-0501 claims 5 and 42 (the approach target replacement, 0 below 128 leptons, 0.5 below 512, 0.75 below 768, is part of guidance, which runs only for an aircraft above the terrain at the end of that movement update, whatever the aircraft: at height 0 its target fraction is claim 5's ordinary one, Q-1941) and claim 6 (holding-height approach: a target with ammunition left sets the mark, and below 768 leptons the held height is the destination's height minus the terrain height there; a point accepted without a target keeps the cell's surface height, so that held height is 0). An attacker that holds height 0 on such an approach, inside 768 leptons, keeps the ordinary fraction
#[test]
fn q1941_ordinary_flight_at_height_0_has_no_approach_replacement() {
    let mut r = rules();
    r.sim.general.economy.curley_shuffle = Some(true);
    // Slow, so that it reaches height 0 while still in the 0.5 band.
    let j = r.sim.aircraft.get_mut("JET").unwrap();
    j.speed = Some(5);
    j.slowdown_distance = Some(1000);
    let (mut w, a, v) = attacker(&r, "JET");
    // A move order: its point is accepted with no target, so not raised.
    click(&mut w, a, ClickOn::Cell(testkit::mid(SIZE, 8, -4)));
    assert!(run_until(&mut w, 300, |w| {
        let s = st(w, a);
        !s.taking_off && s.held > 0 && air::height(w, a) == s.held
    }));
    assert!(!st(&w, a).raised, "the move order's point is not raised");
    let mut bands = [0; 3];
    let s_jet = r.sim.aircraft["JET"].slowdown_distance.unwrap();
    for _ in 0..1500 {
        // A target with ammunition left, kept on every tick, gives the
        // holding-height mark (claim 6).
        w.obj_mut(a).unwrap().target = Some(Target::Object(v));
        step(&mut w);
        let s = st(&w, a);
        let o = w.obj(a).unwrap();
        let Some(d) = o.dest else { continue };
        if s.released
            || s.landing
            || s.taking_off
            || s.ground
            || s.run_lock
            || air::height(&w, a) > 0
        {
            continue;
        }
        assert!(s.holding, "it approaches with the holding-height mark");
        let dd = guide_dist(&w, o.pos, air::flight_point(&w, a, d));
        let ar = &w.rules().air;
        // spec: S-0501 claim 5 (the bands 128, 512 and 768 leptons)
        let (band, replacement) = if dd < 128 {
            (0, glasnost_num::F64::ZERO)
        } else if dd < 512 {
            (1, ar.half)
        } else if dd < 768 {
            (2, ar.three_quarters)
        } else {
            continue;
        };
        // spec: S-0501 claim 5 and F4 (the ordinary calculation: τ from the post-step distance and S)
        let (ordinary, _) = air::slowing(s.cur, dd, s_jet, ar, w.rules().mode);
        assert_eq!(
            s.tgt, ordinary,
            "the ordinary fraction at height 0, {dd} leptons"
        );
        if !ordinary.eq_num(replacement) {
            bands[band] += 1;
        }
    }
    assert!(
        bands.iter().sum::<i32>() > 0,
        "it flies at height 0 inside 768 leptons where the replacement differs: {bands:?}"
    );
}

// spec: S-0501 claims 4, 5 and 42, F3, F4 and T1 (AC24 variants b2, b5 and b6, converted speed 12 and S = 1000, Q-0653: fraction 0 at a post-step distance below min(86, S / 10) and above 16 leptons, then firing with the destination kept, and the shot from the hover)
#[test]
fn claim_42_slow_close_approach_hovers_short_and_fires() {
    for name in ["b2", "b5n", "b5s", "b6w", "b6e"] {
        let x = close_approach(5, 1000, ac24_b(name));
        let (d, kept, at) = x
            .zero
            .unwrap_or_else(|| panic!("{name} reaches fraction 0: {x:?}"));
        assert!(d > 16 && d < 86, "{name}: {x:?}");
        assert!(kept, "{name}: {x:?}");
        assert!(x.firing.is_some_and(|(d, _)| d.is_some()), "{name}: {x:?}");
        assert_eq!(x.shot, Some(at), "{name}: {x:?}");
        if name == "b2" {
            // spec: S-0501 AC24 (b2: the drift north stops about 2 leptons south of the row when the body faces exactly east, 16383, and the 1-lepton steps then go east)
            let p = x.point.unwrap();
            assert!((1..=3).contains(&(at.y - p.y)), "{x:?}");
            let tail: Vec<_> = x.unit.iter().rev().take(30).collect();
            assert_eq!(tail.len(), 30, "{x:?}");
            for w in tail.windows(2) {
                assert_eq!(w[0].0, 16383, "{x:?}");
                assert_eq!(w[0].2, at.y - p.y, "{x:?}");
                assert_eq!(w[0].1, w[1].1 + 1, "one lepton east per tick: {x:?}");
            }
        }
    }
}

// spec: S-0501 claim 5 (converted speed of at least 20 takes no 1-lepton step outside the halving zone, so AC24 variant a flown from the b1, b3 and b4 starts reaches fraction 0 and fires)
#[test]
fn claim_5_fast_aircraft_do_not_stall() {
    for name in ["b1", "b3", "b4"] {
        let x = close_approach(14, 500, ac24_b(name));
        let (d, kept, at) = x
            .zero
            .unwrap_or_else(|| panic!("{name} reaches fraction 0: {x:?}"));
        // spec: S-0501 F4 (d / S in double precision with S-0205 claim 5's rounding: 50 / 500 rounds below 0.1, so d = 50 is in the halving zone)
        assert!(d > 16 && d <= 50, "{name}: {x:?}");
        assert!(kept, "{name}: {x:?}");
        assert_eq!(x.shot, Some(at), "{name}: {x:?}");
    }
}

// spec: S-0501 claims 4, 5 and 42, F3 and Edge cases (AC24 variants b1, b3 and b4, converted speed 12 and S = 1000, Q-0653; quirks.aircraft_unit_step_stall on: once the step falls to 1 lepton, about 166 leptons short, at a body facing in 16391 to 32766 the position stops changing with the fraction positive and steady; it never reaches firing and fires no shot within 2000 ticks)
#[test]
fn claim_5_unit_step_hover_between_east_and_south() {
    let mode = mode();
    for name in ["b1", "b3", "b4"] {
        let x = close_approach(5, 1000, ac24_b(name));
        assert_eq!(x.shot, None, "{name}: {x:?}");
        assert!(x.firing.is_none(), "{name}: {x:?}");
        assert!(x.zero.is_none(), "{name}: {x:?}");
        let (_, d, body, f, still) = x.end.unwrap();
        assert!(still > 1000, "{name} hovers: {x:?}");
        assert!((16391..=32766).contains(&body), "{name}: {x:?}");
        // spec: S-0501 AC24 (b1, b3 and b4: post-step distance about 166, the first d with ⌊V × d / S⌋ = 1)
        assert!((160..=170).contains(&d), "{name}: {x:?}");
        assert_eq!(air::step_length(12, f, mode), 1, "{name}: {x:?}");
        let (at, p) = (x.end.unwrap().0, x.point.unwrap());
        match name {
            // spec: S-0501 AC24 (b1: the facing about 16892; the side distance stays 11 leptons, the southward component of each step truncating away)
            "b1" => {
                assert!((16700..=17100).contains(&body), "{x:?}");
                assert_eq!(at.y - p.y, -11, "{x:?}");
            }
            // spec: S-0501 AC24 (b3: under quirks.chop_rounding each step at body facing 32767 also moves 1 lepton west; from 5120 leptons north the hover facing is about 29778 and the side distance about 47 leptons west of the column)
            "b3" => {
                assert!((29500..=30100).contains(&body), "{x:?}");
                assert!((-50..=-44).contains(&(at.x - p.x)), "{x:?}");
            }
            // spec: S-0501 AC24 (b4: 32766, the westward drift ended 1 lepton east of the column)
            _ => {
                assert_eq!(body, 32766, "{x:?}");
                assert_eq!(at.x - p.x, 1, "{x:?}");
            }
        }
    }
}

// spec: S-0501 AC24 (b3, converted speed 12 and S = 1000: the hover at about 166 leptons with the facing about 32076 and the side distance 11), here with quirks.chop_rounding off, the engine-replay result: no sideways lepton at 32767 to 32772
#[test]
fn claim_5_unit_step_hover_from_the_north_without_chop() {
    let x = close_approach_with(5, 1000, ac24_b("b3"), |q| q.chop_rounding = false);
    assert_eq!(x.shot, None, "{x:?}");
    let (at, d, body, _, still) = x.end.unwrap();
    let p = x.point.unwrap();
    assert!(still > 1000, "{x:?}");
    assert!((160..=170).contains(&d), "{x:?}");
    assert!((31800..=32400).contains(&body), "{x:?}");
    assert_eq!(at.x - p.x, -11, "{x:?}");
}

/// A *move* of JET (raw speed 5, converted 12; S = 1000) from a pad to a
/// cell, flown in cruise from 20 cells west of the destination point and
/// 11 leptons north of its row; whether it lands within 2000 ticks, and
/// its fraction at the end.
fn unit_step_move(stall: bool) -> (bool, glasnost_num::F64, String) {
    let mut r = rules();
    r.globals.quirks.aircraft_unit_step_stall = stall;
    let j = r.sim.aircraft.get_mut("JET").unwrap();
    j.speed = Some(5);
    j.slowdown_distance = Some(1000);
    let (mut w, _k, a) = built_with(&r, "JET");
    let goal = testkit::mid(SIZE, 8, 6);
    click(&mut w, a, ClickOn::Cell(goal));
    assert!(run_until(&mut w, 300, |w| {
        let s = st(w, a);
        !s.taking_off
            && !s.ground
            && air::height(w, a) == s.held
            && w.obj(a).unwrap().dest.is_some()
    }));
    let p = air::flight_point(&w, a, w.obj(a).unwrap().dest.unwrap());
    let at = Pos::new(p.x - 20 * 256, p.y - 11, 0); // spec: S-0204 claim 2 (256 leptons per cell)
    let body = bearing(&w, at, p);
    relocate(&mut w, a, at, body);
    let landed = run_until(&mut w, 2000, |w| air::on_ground(w, a));
    (landed, st(&w, a).cur, describe(&w, a))
}

// spec: S-0501 claims 5 and 6, Edge cases (quirks.aircraft_unit_step_stall on: the 1-lepton hover between east and south also stops a move short of landing, with a positive fraction; off: it lands)
#[test]
fn claim_6_unit_step_hover_stops_a_move() {
    let (landed, f, d) = unit_step_move(true);
    assert!(!landed, "{d}");
    assert!(f.gt(glasnost_num::F64::ZERO), "{d}");
    let (landed, _, d) = unit_step_move(false);
    assert!(landed, "{d}");
}

// spec: S-0501 F3 and Edge cases (quirks.aircraft_unit_step_stall off: the stalled 1-lepton step moves 1 lepton along its larger component, so AC24 b1, b3 and b4 reach fraction 0 in the halving zone and fire)
#[test]
fn claim_5_unit_step_stall_off_advances_and_fires() {
    for name in ["b1", "b3", "b4"] {
        let x = close_approach_with(5, 1000, ac24_b(name), |q| {
            q.aircraft_unit_step_stall = false;
        });
        let (d, kept, at) = x
            .zero
            .unwrap_or_else(|| panic!("{name} reaches fraction 0: {x:?}"));
        assert!(d > 16 && d < 86, "{name}: {x:?}");
        assert!(kept, "{name}: {x:?}");
        assert_eq!(x.shot, Some(at), "{name}: {x:?}");
    }
}

/// The change of one 1-lepton step from (5000, 5000) at facing `f`.
fn unit_step(f: Facing, stall: bool, mode: glasnost_num::Rounding) -> (i32, i32) {
    let one = glasnost_num::F64::from_i64(1, mode);
    let (x, y) = air::step(5000, 5000, f, 1, one, stall, mode);
    (x - 5000, y - 5000)
}

// spec: S-0501 F3 (with s = 1, by body facing, quirks.chop_rounding on: 0 to 16379 (0, −1); 16380 to 16386 (+1, 0); 16387 to 16390 (0, −1); 16391 to 32766 (0, 0), quirks.aircraft_unit_step_stall; 32767 to 32772 (−1, +1); 32773 to 49147 (−1, 0); 49148 to 65531 (−1, −1); 65532 to 65535 (0, −1); off, at positions of at least 1024 leptons, the rows for 32767 to 32772 and 49148 to 49155 become (0, +1) and (−1, 0); a step of two or more leptons always changes a coordinate); Edge cases (off: the stalled facings move 1 lepton along the larger component, x on equal magnitudes; every other step unchanged). Revision 11 answers Q-0654
#[test]
fn f3_unit_step_by_facing() {
    for chop in [false, true] {
        let mut r = rules();
        r.globals.quirks.chop_rounding = chop;
        let mode = game(&r).rules().mode;
        let one = glasnost_num::F64::from_i64(1, mode);
        for f in 0..=u16::MAX {
            let got = unit_step(f, true, mode);
            let want = match f {
                0..=16379 => (0, -1),
                16380..=16386 => (1, 0),
                16387..=16390 => (0, -1),
                16391..=32766 => (0, 0),
                32767..=32772 if chop => (-1, 1),
                32767..=32772 => (0, 1),
                32773..=49147 => (-1, 0),
                49148..=49155 if chop => (-1, -1),
                49148..=49155 => (-1, 0),
                49156..=65531 => (-1, -1),
                65532..=65535 => (0, -1),
            };
            assert_eq!(got, want, "facing {f}, chop {chop}");
            let off = unit_step(f, false, mode);
            if want == (0, 0) {
                assert!(off == (1, 0) || off == (0, 1), "facing {f}: {off:?}");
            } else {
                assert_eq!(off, got, "facing {f}");
            }
            let s2 = air::step(5000, 5000, f, 2, one, true, mode);
            assert_ne!(s2, (5000, 5000), "facing {f}");
        }
        // spec: S-0501 F3 (off: x on equal magnitudes)
        let mut ties = 0;
        for f in 16391..=32766u16 {
            let (dx, dy) = glasnost_sim::walk::step_components(f, 1, true, mode);
            ties += i32::from(dx.abs().eq_num(dy.abs()));
            let want = if dx.abs().ge(dy.abs()) {
                (1, 0)
            } else {
                (0, 1)
            };
            assert_eq!(unit_step(f, false, mode), want, "facing {f}");
        }
        assert!(ties > 0, "the table has facings with equal magnitudes");
        // Off: x near east, y near south (the larger component).
        assert_eq!(unit_step(16500, false, mode), (1, 0));
        assert_eq!(unit_step(32600, false, mode), (0, 1));
    }
}

// spec: S-0501 claims 12 and 42 and T1 (AC24 variant c, converted speed 35 and S = 100: within 16 leptons with a positive fraction the destination is cleared, the flight point remains, and it fires)
#[test]
fn claim_42_short_slowdown_clears_the_destination_keeps_the_point() {
    let x = close_approach(14, 100, Start::Placed);
    let (dest, point) = x.firing.unwrap_or_else(|| panic!("firing: {x:?}"));
    assert_eq!(dest, None, "destination cleared: {x:?}");
    assert!(point.is_some(), "the flight point remains: {x:?}");
    assert!(x.shot.is_some(), "fires: {x:?}");
}

// spec: S-0501 claim 42 and Edge cases (AC24 variant d, converted speed 7 and S = 500, flown from the west south of the row, as b2, so that its 1-lepton steps advance; quirks.aircraft_zero_step_attack_stall on: it hovers with a positive fraction and F3 step length 0 at least 16 leptons from the point, and fires no shot within 2000 ticks)
#[test]
fn claim_42_slow_attacker_never_fires() {
    let x = close_approach(3, 500, ac24_b("b2"));
    assert_eq!(x.shot, None, "{x:?}");
    assert!(x.firing.is_none(), "{x:?}");
    assert!(x.positive_at_end, "{x:?}");
    assert!(x.zero_steps > 1000, "{x:?}");
}

// spec: S-0501 claim 42, T1 and Edge cases (quirks.aircraft_zero_step_attack_stall off: AC24 variant d, flown as b2, moves to firing at the first approach processing with step length 0, keeping its destination, and fires; F3 and F4 are unchanged, so it never reaches fraction 0)
#[test]
fn claim_42_zero_step_stall_off_fires() {
    let x = close_approach_with(3, 500, ac24_b("b2"), |q| {
        q.aircraft_zero_step_attack_stall = false;
    });
    let (dest, _) = x.firing.unwrap_or_else(|| panic!("firing: {x:?}"));
    assert!(dest.is_some(), "the destination kept: {x:?}");
    assert!(x.zero.is_none(), "{x:?}");
    assert!(x.shot.is_some(), "fires: {x:?}");
    let first = x
        .first_zero_step
        .unwrap_or_else(|| panic!("a zero step: {x:?}"));
    assert_eq!(x.firing_tick, Some(first + 1), "the next processing: {x:?}");
    assert_eq!(x.zero_steps, 1, "{x:?}");
}

// spec: S-0501 claim 42 and Edge cases (quirks.aircraft_zero_step_attack_stall off alone: a 1-lepton step that changes neither coordinate is not a length-0 step, so AC24 b1 remains approaching and fires no shot)
#[test]
fn claim_42_zero_step_switch_leaves_the_unit_step_hover() {
    let x = close_approach_with(5, 1000, ac24_b("b1"), |q| {
        q.aircraft_zero_step_attack_stall = false;
    });
    assert_eq!(x.shot, None, "{x:?}");
    assert!(x.firing.is_none(), "{x:?}");
    assert!(x.end.unwrap().4 > 1000, "hovers: {x:?}");
}

// spec: S-0501 claim 39 (the firing spot: a cell on the largest circle of radius ρ − 256 around the target, the last or the preceding record by distance to the attacker, here the cells of facings 49152 and 45056)
#[test]
fn claim_39_firing_spot_on_the_circle() {
    let mut r = rules();
    r.sim.general.economy.curley_shuffle = Some(true);
    let (mut w, a, v) = attacker(&r, "JET");
    click(&mut w, a, ClickOn::Object(v));
    assert!(run_until(&mut w, 50, |w| st(w, a).stage == air::Stage::Approaching));
    let Some(Target::Cell(c)) = w.obj(a).unwrap().dest else {
        panic!("a cell destination: {}", describe(&w, a));
    };
    let tp = w.obj(v).unwrap().pos;
    let mode = w.rules().mode;
    // spec: S-0501 claim 39 (ρ − 256: the 6-cell range in leptons less 256; F3's trigonometry and truncation, then division by 256)
    let cell_at = |f: u16| {
        let (x, y) = glasnost_sim::walk::step_point(tp.x, tp.y, f, 6 * 256 - 256, true, mode);
        glasnost_map::CellXY::new(x / 256, y / 256)
    };
    assert!(
        c == cell_at(49152) || c == cell_at(45056),
        "{c:?} is neither the last nor the preceding record"
    );
}

// spec: S-0501 claim 39 (radii strictly above 256 only: a range of 2 cells leaves none, so no destination and the run is done)
#[test]
fn claim_39_short_range_has_no_spot() {
    let mut r = rules();
    r.sim.weapons.insert(
        "Short".into(),
        WeaponType {
            damage: Some(40),
            rof: Some(30),
            range: dec("2"),
            speed: Some(60),
            projectile: Some("Rocket".into()),
            warhead: Some("HE".into()),
            ..Default::default()
        },
    );
    push(&mut r, "aircraft", "JETS");
    r.sim
        .aircraft
        .insert("JETS".into(), flyer(Some("Short"), 2));
    let (mut w, a, v) = attacker(&r, "JETS");
    click(&mut w, a, ClickOn::Object(v));
    let mut approached = false;
    for _ in 0..60 {
        step(&mut w);
        approached |= st(&w, a).stage == air::Stage::Approaching;
    }
    assert!(!approached, "{}", describe(&w, a));
    assert_eq!(w.obj(a).unwrap().dest, None);
}

// spec: S-0501 claim 40 (a move onto a cell F7 refuses, here one holding a vehicle, is replaced: radius 0 gives only that cell, radius 1 a neighbour)
#[test]
fn claim_40_move_onto_an_occupied_object_substitutes_a_cell() {
    let mut w = game(&rules());
    ready(&mut w);
    let v = put(&mut w, Kind::Vehicle, "VICTIM", 0, 6, 0);
    let a = put(&mut w, Kind::Aircraft, "SCOUT", 0, 0, 0);
    open(&mut w);
    assert!(orders::issue(
        &mut w,
        ALPHA,
        Order::Behaviour {
            actor: a,
            behaviour: Behaviour::Move,
            attack_move: false,
            target: None,
            dest: Some(Target::Object(v)),
        }
    ));
    assert!(run_until(&mut w, 20, |w| w.obj(a).unwrap().beh.current
        == Some(Behaviour::Move)));
    step(&mut w);
    let vc = w.obj(v).unwrap().pos.cell();
    let Some(Target::Cell(c)) = w.obj(a).unwrap().dest else {
        panic!("a substituted cell: {}", describe(&w, a));
    };
    assert_ne!(c, vc);
    assert_eq!((c.x - vc.x).abs().max((c.y - vc.y).abs()), 1, "{c:?}");
}

// spec: S-0501 claim 40 (stopped: no unfinished flight; a non-landable aircraft hovering at its destination with fraction 0 is not stopped, a landed one is, and landing completion clears the destination)
#[test]
fn claim_40_stopped_needs_no_unfinished_flight() {
    let mut r = rules();
    let mut hover = flyer(None, 0);
    hover.econ.landable = Some(false);
    push(&mut r, "aircraft", "HOVER");
    r.sim.aircraft.insert("HOVER".into(), hover);
    let mut w = game(&r);
    ready(&mut w);
    let h = put(&mut w, Kind::Aircraft, "HOVER", 0, 0, 0);
    let s = put(&mut w, Kind::Aircraft, "SCOUT", 0, 0, 3);
    // spec: S-0501 claim 40 (a freshly placed aircraft has no unfinished flight)
    assert!(air::stopped(&w, h) && air::stopped(&w, s));
    click(&mut w, h, ClickOn::Cell(testkit::mid(SIZE, 5, 0)));
    click(&mut w, s, ClickOn::Cell(testkit::mid(SIZE, 5, 3)));
    let hovering = run_until(&mut w, 600, |w| {
        st(w, h).cur == glasnost_num::F64::ZERO && air::height(w, h) > 0 && !st(w, h).taking_off
    });
    assert!(hovering, "{}", describe(&w, h));
    testkit::run(&mut w, 30);
    assert!(
        st(&w, h).unfinished && !air::stopped(&w, h),
        "{}",
        describe(&w, h)
    );
    let landed = run_until(&mut w, 600, |w| st(w, s).ground);
    assert!(landed, "{}", describe(&w, s));
    assert!(air::stopped(&w, s));
    assert_eq!(w.obj(s).unwrap().dest, None);
}

/// Whether the effects hold a re-file moment (S-0102 claim 4) of `id`.
fn refiled(w: &World, id: ObjectId) -> bool {
    w.effects.iter().any(|e| {
        matches!(
            e,
            glasnost_sim::combat::effects::Effect::LayerMoment { object, check: None } if *object == id
        )
    })
}

// spec: S-0102 claim 4 (a landable aircraft is re-filed at its take-off and at every landing update, not while it cruises; a non-landable aircraft is never re-filed by take-off or landing)
#[test]
fn s0102_claim_4_aircraft_layer_moments() {
    let mut r = rules();
    let mut hover = flyer(None, 0);
    hover.econ.landable = Some(false);
    push(&mut r, "aircraft", "HOVER");
    r.sim.aircraft.insert("HOVER".into(), hover);
    let mut w = game(&r);
    ready(&mut w);
    let h = put(&mut w, Kind::Aircraft, "HOVER", 0, 0, 0);
    let s = put(&mut w, Kind::Aircraft, "SCOUT", 0, 0, 3);
    w.effects.clear();
    click(&mut w, h, ClickOn::Cell(testkit::mid(SIZE, 5, 0)));
    click(&mut w, s, ClickOn::Cell(testkit::mid(SIZE, 9, 3)));
    let (mut take_off, mut cruise_quiet, mut landings) = (false, false, 0);
    for _ in 0..1500 {
        let before = st(&w, s);
        step(&mut w);
        let after = st(&w, s);
        assert!(!refiled(&w, h), "non-landable: {}", describe(&w, h));
        let moment = refiled(&w, s);
        if before.ground && !after.ground {
            assert!(moment, "the take-off: {}", describe(&w, s));
            take_off = true;
        }
        if before.landing || after.landing {
            assert!(moment, "a landing update: {}", describe(&w, s));
            landings += 1;
        } else if take_off && !before.taking_off && !after.taking_off && !after.ground {
            assert!(!moment, "cruising: {}", describe(&w, s));
            cruise_quiet = true;
        }
        w.effects.clear();
        if take_off && after.ground {
            break;
        }
    }
    assert!(
        take_off && cruise_quiet && landings > 1,
        "{}",
        describe(&w, s)
    );
}

// spec: S-0501 claim 4 (a living descending aircraft with a destination moves 5 more leptons toward its destination cell's centre; with fraction 0 that is its whole displacement)
#[test]
fn claim_4_descent_nudge() {
    let mut w = game(&rules());
    ready(&mut w);
    let a = lifted(&mut w, "SCOUT", 0, 0);
    let goal = testkit::mid(SIZE, -2, -2);
    click(&mut w, a, ClickOn::Cell(goal));
    let mut seen = false;
    for _ in 0..1500 {
        let s = st(&w, a);
        let (p0, h0) = (w.obj(a).unwrap().pos, air::height(&w, a));
        step(&mut w);
        if s.landing && s.cur == glasnost_num::F64::ZERO && h0 > 0 && air::height(&w, a) < h0 {
            let p1 = w.obj(a).unwrap().pos;
            let m = (i64::from(p1.x - p0.x).pow(2) + i64::from(p1.y - p0.y).pow(2)).isqrt();
            // spec: S-0501 Constants (descent horizontal nudge 5 leptons; table trigonometry and truncation, so at most 5 and at least 4 along each step)
            assert!((4..=5).contains(&m), "moved {m} leptons");
            seen = true;
        }
        if st(&w, a).ground {
            break;
        }
    }
    assert!(seen, "a descending tick at fraction 0: {}", describe(&w, a));
}

// spec: S-0501 claim 6 (a nonempty movement point accepted with a target and ammunition left is raised to terrain plus cruise height; without a target it keeps the surface height)
#[test]
fn claim_6_armed_flight_point_height() {
    let mut w = game(&rules());
    ready(&mut w);
    let v = put(&mut w, Kind::Vehicle, "VICTIM", 1, 8, 0);
    let a = put(&mut w, Kind::Aircraft, "JET", 0, 0, 0);
    let c = testkit::mid(SIZE, 4, 4);
    let centre = glasnost_sim::world::Pos::centre(c, 0);
    let ground = w.surface_at(centre.x, centre.y);
    assert_eq!(air::flight_point(&w, a, Target::Cell(c)).z, ground);
    w.obj_mut(a).unwrap().target = Some(Target::Object(v));
    // spec: S-0501 F10 and Import mapping (FlightLevel absent: cruise height 500)
    assert_eq!(air::flight_point(&w, a, Target::Cell(c)).z, ground + 500);
    w.obj_mut(a).unwrap().combat.ammo = 0;
    assert_eq!(air::flight_point(&w, a, Target::Cell(c)).z, ground);
}

// spec: S-0501 claim 7 (the ground layer exactly when the height above the surface is at most 0; any positive height is in flight)
#[test]
fn claim_7_ground_boundary() {
    let mut w = game(&rules());
    ready(&mut w);
    let a = put(&mut w, Kind::Aircraft, "SCOUT", 0, 0, 0);
    let p = w.obj(a).unwrap().pos;
    let ground = w.surface_at(p.x, p.y);
    assert!(air::on_ground(&w, a));
    w.obj_mut(a).unwrap().pos.z = ground + 1;
    assert!(!air::on_ground(&w, a));
    w.obj_mut(a).unwrap().pos.z = ground;
    assert!(air::on_ground(&w, a));
}

// spec: S-0501 claim 8 (at or below trunc(H / 2) no new heading request: the body keeps its facing; above it the body is requested to face the destination)
#[test]
fn claim_8_no_heading_request_below_half_height() {
    let mut w = game(&rules());
    ready(&mut w);
    let a = put(&mut w, Kind::Aircraft, "SCOUT", 0, 0, 0);
    let f0 = w.obj(a).unwrap().body.value(w.tick);
    click(&mut w, a, ClickOn::Cell(testkit::mid(SIZE, 0, 8)));
    let mut turned = false;
    for _ in 0..80 {
        step(&mut w);
        let h = air::height(&w, a);
        let f = w.obj(a).unwrap().body.value(w.tick);
        if !st(&w, a).taking_off {
            continue;
        }
        // spec: S-0501 claim 8 (H = 500, trunc(H / 2) = 250)
        if h <= 250 {
            assert_eq!(f, f0, "h = {h}");
        } else if f != f0 {
            turned = true;
        }
    }
    assert!(turned, "{}", describe(&w, a));
}

// spec: S-0501 claim 24 (enter's docking steps leave it at the docking position at touchdown, here with no displacement; after touchdown it reports entered: sleep, not on the touchdown tick, within the 3-tick arrival and 1-tick docking stages)
#[test]
fn claim_24_docking_follows_touchdown() {
    let mut r = rules();
    r.sim.general.economy.curley_shuffle = Some(true);
    let (mut w, k, a) = built_with(&r, "JET");
    let v = put(&mut w, Kind::Vehicle, "VICTIM", 1, 10, 0);
    click(&mut w, a, ClickOn::Object(v));
    assert!(run_until(&mut w, 3000, |w| w.obj(a).unwrap().combat.ammo == 0));
    let down = run_until(&mut w, 3000, |w| st(w, a).ground);
    assert!(down, "{}", describe(&w, a));
    assert_eq!(w.obj(a).unwrap().beh.current, Some(Behaviour::Enter));
    assert_eq!(w.obj(a).unwrap().contact, Some(k));
    let t0 = w.tick;
    let (p, _) = glasnost_sim::econ::release::pad_position(&w, k, a);
    // Docked: at the docking position, enter over (sleep, or the guard that a same-tick start of the queued behaviour makes current before the pad's service step queues sleep again).
    let docked = run_until(&mut w, 10, |w| {
        let o = w.obj(a).unwrap();
        o.pos == p && o.beh.current != Some(Behaviour::Enter)
    });
    assert!(docked, "{}", describe(&w, a));
    assert!((1..=6).contains(&(w.tick - t0)), "{} ticks", w.tick - t0);
    let slept = run_until(&mut w, 2, |w| {
        w.obj(a).unwrap().beh.current == Some(Behaviour::Sleep)
    });
    assert!(slept, "{}", describe(&w, a));
}

// spec: S-0501 claim 36 and S-0506 claims 12 and 13 (a carrier's destruction releases its flying slot aircraft into a crash: health 0, unfinished flight and speed fraction kept, ordinary movement goes on)
#[test]
fn claim_36_release_crash_keeps_flight() {
    let r = rules();
    let mut w = game(&r);
    ready(&mut w);
    let c = put(&mut w, Kind::Vehicle, "CARRIER", 0, 0, 0);
    let v = put(&mut w, Kind::Vehicle, "VICTIM", 1, 8, 0);
    testkit::run(&mut w, 2);
    click(&mut w, c, ClickOn::Object(v));
    let flying = |w: &World| {
        w.objects
            .values()
            .find(|o| {
                o.kind == Kind::Aircraft
                    && o.on_map
                    && air::height(w, o.id) > 200
                    && o.air
                        .as_ref()
                        .is_some_and(|s| s.cur != glasnost_num::F64::ZERO)
            })
            .map(|o| o.id)
    };
    assert!(run_until(&mut w, 600, |w| flying(w).is_some()));
    let h = flying(&w).unwrap();
    let before = st(&w, h);
    let p0 = w.obj(h).unwrap().pos;
    w.obj_mut(c).unwrap().health = 0;
    glasnost_sim::combat::death::kill(
        &mut w,
        c,
        &glasnost_sim::combat::damage::Hit {
            amount: 0,
            warhead: None,
            distance: 0,
            attacker: None,
            house: None,
            ignore_defences: false,
            no_escape: false,
        },
    );
    let o = w.obj(h).expect("still falling");
    assert_eq!(o.health, 0);
    let s = st(&w, h);
    assert!(s.crash.is_some(), "crashing: {}", describe(&w, h));
    assert!(s.unfinished);
    assert_eq!(s.cur, before.cur);
    step(&mut w);
    let p1 = w.obj(h).unwrap().pos;
    assert_ne!((p1.x, p1.y), (p0.x, p0.y), "it keeps moving");
}

// spec: S-0501 claims 9 and 36, S-0506 claims 12 and 13 and S-0312 claim 16 (the lead's note 16 of M8 step A4, round 4: a carrier destroyed by a hit releases its flying slot aircraft into a crash at health 0; each falls as a dead aircraft, its fall counter growing by 1 a tick, and at impact leaves play as one shot down)
#[test]
fn claim_36_released_aircraft_crash_and_leave_play() {
    let r = rules();
    let mut w = game(&r);
    ready(&mut w);
    let c = put(&mut w, Kind::Vehicle, "CARRIER", 0, 0, 0);
    let v = put(&mut w, Kind::Vehicle, "VICTIM", 1, 8, 0);
    testkit::run(&mut w, 2);
    click(&mut w, c, ClickOn::Object(v));
    let flying = |w: &World| -> Vec<ObjectId> {
        w.objects
            .values()
            .filter(|o| o.kind == Kind::Aircraft && o.on_map && air::height(w, o.id) > 0)
            .map(|o| o.id)
            .collect()
    };
    assert!(run_until(&mut w, 600, |w| !flying(w).is_empty()));
    let planes = flying(&w);
    let strength = w.obj(c).unwrap().health;
    // A killing hit on the carrier, through the damage path.
    glasnost_sim::combat::damage::apply(
        &mut w,
        c,
        glasnost_sim::combat::damage::Hit {
            amount: strength,
            warhead: None,
            distance: 0,
            attacker: None,
            house: None,
            ignore_defences: true,
            no_escape: true,
        },
    );
    assert!(
        w.obj(c).is_none_or(|o| o.health <= 0),
        "the carrier is killed"
    );
    for &p in &planes {
        let o = w.obj(p).expect("still in play at the release");
        assert_eq!(o.health, 0, "{}", describe(&w, p));
        assert!(glasnost_sim::combat::damage::dead(o), "a dead aircraft");
        assert!(st(&w, p).crash.is_some(), "crashing: {}", describe(&w, p));
    }
    let mut last: Vec<Option<i32>> = planes.iter().map(|&p| st(&w, p).crash).collect();
    for _ in 0..600 {
        step(&mut w);
        for (i, &p) in planes.iter().enumerate() {
            let Some(o) = w.obj(p) else { continue };
            let k = o.air.as_ref().and_then(|a| a.crash);
            if let (Some(a), Some(b)) = (last[i], k) {
                // spec: S-0501 claim 9 (the fall counter grows by 1 a tick for a dead aircraft, by 3 for a living one)
                assert_eq!(b, a + 1, "the dead fall counter: {}", describe(&w, p));
            }
            last[i] = k;
        }
        if planes.iter().all(|&p| w.obj(p).is_none()) {
            break;
        }
    }
    assert!(
        planes.iter().all(|&p| w.obj(p).is_none()),
        "every released aircraft has left play"
    );
}

/// The cell of local point (a, b) by S-0522 F2, for the test map's local
/// area (0, 0, 30, 30) and width 30.
fn local_cell(a: i32, b: i32) -> glasnost_map::CellXY {
    // spec: S-0522 F2 (x = ⌊(Y + 1) / 2⌋ + X, y = W + ⌊Y / 2⌋ − X, floor division)
    glasnost_map::CellXY::new((b + 1).div_euclid(2) + a, SIZE + b.div_euclid(2) - a)
}

// spec: S-0522 claim 4 and S-0501 claim 34 (edge cells lie outside the usable area); S-0522 Edge cases (quirks.paradrop_west_edge_start_row: on, the west rows start at rem(r − 1, 2Lh); off, at r mod 2Lh)
#[test]
fn claim_34_edge_cells_and_the_west_start_row() {
    let w0 = game(&rules());
    assert_eq!(w0.cells.footprint().local, [0, 0, 30, 30]);
    let mut w = w0.clone();
    for e in 0..4 {
        let c = air::edge_cell_on(&mut w, e);
        assert!(!w.cells.usable(c), "edge {e}: {c:?}");
    }
    for on in [true, false] {
        let mut r = rules();
        r.globals.quirks.paradrop_west_edge_start_row = on;
        let mut w = game(&r);
        let half = w.rules().quirks.ranged_draw_half_span;
        // spec: S-0522 claim 4 (west: r = R(0, 2Lh))
        let rr = w.rng.clone().ranged(0, 60, half);
        let first = (0..60)
            .map(|i| {
                if on {
                    (rr - 1 + i) % 60
                } else {
                    (rr + i).rem_euclid(60)
                }
            })
            .find(|&b| !w.cells.usable(local_cell(0, b)))
            .unwrap();
        assert_eq!(
            air::edge_cell_on(&mut w, 3),
            local_cell(0, first),
            "quirk {on}, r = {rr}"
        );
    }
}

// spec: S-0522 claim 5 (marking a building factory primary computes the house's edge again: the edge whose F3 midpoint is nearest the cell of its centre)
#[test]
fn claim_34_primary_building_factory_sets_the_edge() {
    let mut r = rules();
    push(&mut r, "buildings", "BFAC");
    r.sim.buildings.insert("BFAC".into(), factory("building"));
    let mut w = game(&r);
    ready(&mut w);
    assert_eq!(w.houses[0].computed_edge, 0);
    // Centre cells (44, 16) beside the east midpoint (45, 15) and (15, 44) beside the west midpoint (15, 45).
    let east = building(&mut w, "BFAC", 0, 13, -15);
    let west = building(&mut w, "BFAC", 0, -16, 13);
    for (b, edge) in [(east, 1), (west, 3)] {
        open(&mut w);
        assert!(orders::issue(&mut w, ALPHA, Order::Primary { actor: b }));
        step(&mut w);
        assert_eq!(w.houses[0].computed_edge, edge);
    }
}

// spec: S-0501 claim 41 (no dock types and no other own-house object: the aircraft's own cell, without another search)
#[test]
fn claim_41_own_cell_without_anchors() {
    let mut w = game(&rules());
    ready(&mut w);
    let a = put(&mut w, Kind::Aircraft, "HORNET", 0, 2, 2);
    assert_eq!(air::landing_spot(&mut w, a), Some(testkit::mid(SIZE, 2, 2)));
}

fn cheb(a: glasnost_map::CellXY, b: glasnost_map::CellXY) -> i32 {
    (a.x - b.x).abs().max((a.y - b.y).abs())
}

// spec: S-0501 claim 41 (the fallback prefers the nearest other own-house object; equal integer distances favour the object created earliest, whatever its kind)
#[test]
fn claim_41_fallback_anchor_preference() {
    // Unequal: the nearer anchor wins even when created later.
    let mut w = game(&rules());
    ready(&mut w);
    let far = put(&mut w, Kind::Vehicle, "VICTIM", 0, 7, 0);
    let near = put(&mut w, Kind::Vehicle, "VICTIM", 0, -3, 0);
    let a = put(&mut w, Kind::Aircraft, "HORNET", 0, 0, 0);
    let s = air::landing_spot(&mut w, a).unwrap();
    assert!(cheb(s, w.obj(near).unwrap().pos.cell()) <= 1, "{s:?}");
    let _ = far;
    // Equal: the earlier-created of two anchors at the same distance, in either placement.
    for flip in [false, true] {
        let mut w = game(&rules());
        ready(&mut w);
        let (d1, d2) = if flip { (-4, 4) } else { (4, -4) };
        let first = put(&mut w, Kind::Vehicle, "VICTIM", 0, d1, 0);
        let _second = put(&mut w, Kind::Infantry, "GRUNT", 0, d2, 0);
        let a = put(&mut w, Kind::Aircraft, "HORNET", 0, 0, 0);
        let s = air::landing_spot(&mut w, a).unwrap();
        assert!(
            cheb(s, w.obj(first).unwrap().pos.cell()) <= 1,
            "flip {flip}: {s:?}"
        );
    }
}

// spec: S-0501 claim 41 and S-0212 claim 25a (near a house building far enough away the result is the top-left of a 3 × 3 footprint, all nine cells free, returned unchanged)
#[test]
fn claim_41_building_search_footprint() {
    let mut w = game(&rules());
    let k = building(&mut w, "APAD", 0, 10, 0);
    ready(&mut w);
    let a = put(&mut w, Kind::Aircraft, "SCOUT", 0, -6, 0);
    let s = air::landing_spot(&mut w, a).unwrap();
    for i in 0..3 {
        for j in 0..3 {
            let c = glasnost_map::CellXY::new(s.x + i, s.y + j);
            assert!(
                w.cells
                    .occ(c, glasnost_sim::world::Layer::Ground)
                    .is_some_and(|o| o.empty()),
                "{c:?} of {s:?}"
            );
        }
    }
    assert!(cheb(s, w.obj(k).unwrap().pos.cell()) <= 4, "{s:?}");
}

/// The terrain code of a water tile in the scene catalogue (S-0211 Terms).
const WATER_CODE: u8 = 9;

/// AC25's patch: SCOUT on a one-cell island of clear ground ringed by
/// water (no foot passage), the island cell also held by an enemy VICTIM,
/// and an APAD (SCOUT's first dock type) of its house two cells away, so
/// the weighted building distance is below 256 leptons. With `ring` off
/// the island is ordinary open ground.
fn no_own_cell(quirk: bool, ring: bool) -> (World, ObjectId, ObjectId) {
    let mut r = rules();
    r.globals.quirks.landing_own_cell_no_result = quirk;
    let zero = LandRow {
        foot: dec("0"),
        track: dec("0"),
        wheel: dec("0"),
        hover: dec("0"),
        float: dec("1"),
        amphibious: dec("1"),
        float_beach: dec("1"),
        buildable: None,
    };
    r.sim.land_types.insert(LandType::Water, zero);
    let mut s = common::Scene::new();
    s.ruleset = r;
    if ring {
        for dx in -1..=1 {
            for dy in -1..=1 {
                if (dx, dy) != (0, 0) {
                    s.set_cell(common::cell(dx, dy), common::code_tile(WATER_CODE), 0);
                }
            }
        }
    }
    let mut w = s.build();
    let pad = testkit::spawn(&mut w, Kind::Building, "APAD", 0, common::cell(2, 0), 0);
    let a = testkit::spawn(&mut w, Kind::Aircraft, "SCOUT", 0, common::cell(0, 0), 0);
    testkit::spawn(&mut w, Kind::Vehicle, "VICTIM", 1, common::cell(0, 0), 0);
    (w, a, pad)
}

// spec: S-0501 claim 41 and F8 (Q-0650: a building chosen below 256 leptons and no result from the own-cell search: the landing destination is the placeholder at map cell (0, 0), where no map has a cell; not the spot near the building, not the other-object fallback, not the own cell; quirks.landing_own_cell_no_result on)
#[test]
fn claim_41_own_cell_no_result_gives_the_placeholder() {
    let (mut w, a, _) = no_own_cell(true, true);
    let origin = glasnost_map::CellXY::new(0, 0);
    assert!(
        w.cells.cell(origin).is_none(),
        "no map has a cell at (0, 0)"
    );
    assert_eq!(air::landing_spot(&mut w, a), Some(origin));
    assert!(air::is_placeholder(&w, Some(Target::Cell(origin))));
    // Control: on open ground the own-cell search finds a free cell near it.
    let (mut w, a, _) = no_own_cell(true, false);
    let s = air::landing_spot(&mut w, a).unwrap();
    assert_ne!(s, origin);
    assert!(cheb(s, common::cell(0, 0)) <= 2, "{s:?}");
}

// spec: S-0501 Edge cases and claim 41 (quirks.landing_own_cell_no_result off: no successful building choice, so the other-object fallback runs and here gives a free cell near the chosen building, counted like any other own-house object)
#[test]
fn claim_41_own_cell_no_result_quirk_off_falls_back() {
    let (mut w, a, pad) = no_own_cell(false, true);
    let s = air::landing_spot(&mut w, a).unwrap();
    assert_ne!(s, glasnost_map::CellXY::new(0, 0));
    assert_ne!(s, common::cell(0, 0), "not the own cell");
    assert!(w.cells.cell(s).is_some());
    assert!(cheb(s, w.obj(pad).unwrap().pos.cell()) <= 3, "{s:?}");
}

// spec: S-0501 claim 41 (Q-0650: an aircraft whose target is the placeholder discards its destination and target at the end of its update and falls back to rest again, claim 18)
#[test]
fn claim_41_placeholder_discarded_at_end_of_update() {
    let mut w = game(&rules());
    ready(&mut w);
    let a = lifted(&mut w, "SCOUT", 0, 0);
    let origin = glasnost_map::CellXY::new(0, 0);
    assert!(w.cells.cell(origin).is_none());
    w.obj_mut(a).unwrap().target = Some(Target::Cell(origin));
    step(&mut w);
    let o = w.obj(a).unwrap();
    assert_eq!(o.target, None, "{}", describe(&w, a));
    assert_ne!(o.dest, Some(Target::Cell(origin)), "{}", describe(&w, a));
    let far = testkit::mid(SIZE, -12, -12);
    assert_ne!(
        o.dest,
        Some(Target::Cell(far)),
        "the move destination discarded too: {}",
        describe(&w, a)
    );
    // spec: S-0501 T2 (in flight otherwise: the landing spot; move)
    assert!(
        o.beh.queued == Some(Behaviour::Move) || o.beh.current == Some(Behaviour::Move),
        "fell back to rest: {}",
        describe(&w, a)
    );
}

// ---------------------------------------------------------------------------
// Claim 43: the aircraft deck mark (Q-0841 answered), with S-0209 claim 13's listing height and
// S-0105 claim 40's use of the mark. Scenario AC27 on a synthetic scene.

/// Makes the cell at (dx, dy) and its neighbour towards y - 1 high-bridge cells (axis along x).
fn bridge(w: &mut World, dx: i32, dy: i32) -> glasnost_map::CellXY {
    let c = testkit::mid(SIZE, dx, dy);
    let n = glasnost_map::CellXY::new(c.x, c.y - 1);
    for k in [c, n] {
        let i = w.cells.idx(k).unwrap();
        w.cells.deck[i] = true;
        w.cells.deck_entry[i] = true;
        w.cells.bridge_axis[i] = glasnost_sim::cells::Axis::X;
    }
    c
}

fn listed(w: &World, id: ObjectId) -> (bool, bool) {
    let c = w.obj(id).unwrap().pos.cell();
    (
        w.cells
            .list(c, glasnost_sim::world::Layer::Ground)
            .contains(&id),
        w.cells
            .list(c, glasnost_sim::world::Layer::Deck)
            .contains(&id),
    )
}

fn above_terrain(w: &World, id: ObjectId) -> i32 {
    let p = w.obj(id).unwrap().pos;
    p.z - w.surface_at(p.x, p.y)
}

// spec: S-0501 claim 43 (an unmarked aircraft that flies across a high bridge without settling onto it keeps the mark clear, even above deck height; horizontal flight alone leaves the mark unchanged) and Terms, height (an unmarked aircraft at least 416 leptons above terrain on a high-bridge cell also uses the deck as its reference); S-0209 claim 13 (a positive listing height excludes it from both cell-layer lists)
#[test]
fn ac27_flying_across_a_bridge_never_marks() {
    let mut w = game(&rules());
    let bridge_cell = bridge(&mut w, 0, 0);
    let a = put(&mut w, Kind::Aircraft, "SCOUT", 0, -6, 0);
    let goal = testkit::mid(SIZE, 10, 0);
    click(&mut w, a, ClickOn::Cell(goal));
    let mut over = 0;
    let mut deck_reference = false;
    for _ in 0..400 {
        step(&mut w);
        let o = w.obj(a).unwrap();
        assert!(
            !air::deck_marked(&w, a),
            "no mark in flight: {}",
            describe(&w, a)
        );
        if above_terrain(&w, a) > 0 {
            assert_eq!(
                listed(&w, a),
                (false, false),
                "a positive listing height lists nowhere"
            );
        }
        if o.pos.cell() == bridge_cell {
            over += 1;
            // At cruise height (500 >= 416) the deck is the temporary reference: h is 416 lower.
            deck_reference |=
                above_terrain(&w, a) >= 416 && air::height(&w, a) == above_terrain(&w, a) - 416;
        }
        if air::on_ground(&w, a) && o.pos.cell() == goal {
            break;
        }
    }
    assert!(over > 0, "it crossed the bridge cell");
    assert!(
        deck_reference,
        "the deck was the temporary altitude reference above the bridge"
    );
    assert!(!air::deck_marked(&w, a));
}

// spec: S-0501 claim 43 (ordinary descent that uses the high bridge as its reference and whose step reaches or passes height 0 above that deck sets the mark, with the resulting height 0 above the deck; the mark is not inferred from the render layer or the occupancy flags) and claim 7 (height 0 above the deck: the ground render layer); S-0209 claim 13 (listing height = world height − terrain surface height − deck allowance; at 0 or below it is listed on deck when deck-marked, never on ground); S-0105 claim 40 (the mark is the shadow's b)
#[test]
fn ac27_landing_on_a_bridge_sets_the_mark_and_lists_on_deck() {
    let mut w = game(&rules());
    let goal = bridge(&mut w, 5, 0);
    let a = put(&mut w, Kind::Aircraft, "SCOUT", 0, -3, 0);
    click(&mut w, a, ClickOn::Cell(goal));
    let mut marked_at = None;
    let mut flew = false;
    for _ in 0..600 {
        step(&mut w);
        flew |= !st(&w, a).ground;
        if marked_at.is_none() && air::deck_marked(&w, a) {
            marked_at = Some(w.tick);
            assert_eq!(air::height(&w, a), 0, "set with height 0 above the deck");
            assert_eq!(above_terrain(&w, a), 416);
        }
        if flew && st(&w, a).ground {
            break;
        }
    }
    assert!(
        marked_at.is_some(),
        "the descent set the mark: {}",
        describe(&w, a)
    );
    assert!(st(&w, a).ground, "settled: {}", describe(&w, a));
    assert_eq!(w.obj(a).unwrap().pos.cell(), goal);
    assert_eq!(above_terrain(&w, a), 416, "at deck height");
    assert_eq!(air::listing_height(&w, a), 0);
    assert!(
        air::on_ground(&w, a),
        "height 0 and the ground render layer"
    );
    assert_eq!(
        w.obj(a).unwrap().layer,
        glasnost_sim::world::Layer::Ground,
        "no separate bridge render layer"
    );
    assert_eq!(listed(&w, a), (false, true), "deck list only");
    assert!(air::deck_marked(&w, a));
    // The shadow's b (S-0105 Terms) is read from the same mark: a = z - H - 416 = 0.
    assert_eq!(glasnost_sim::world::Layer::Ground, w.obj(a).unwrap().layer);
}

// spec: S-0501 claim 43 (an ordinary altitude adjustment that climbs toward the held height clears the mark in that movement tick, the first climbing tick after a deck take-off included; ordinary cruise over the bridge is not marked just because the aircraft once landed there); S-0209 claim 13 (taking off removes the listing when the listing height becomes positive)
#[test]
fn ac27_taking_off_from_the_deck_clears_the_mark_in_the_first_climbing_tick() {
    let mut w = game(&rules());
    let goal = bridge(&mut w, 5, 0);
    let a = put(&mut w, Kind::Aircraft, "SCOUT", 0, -3, 0);
    click(&mut w, a, ClickOn::Cell(goal));
    assert!(run_until(&mut w, 800, |w| st(w, a).ground && air::deck_marked(w, a)));
    assert_eq!(listed(&w, a), (false, true));
    let far = testkit::mid(SIZE, -9, 3);
    click(&mut w, a, ClickOn::Cell(far));
    let mut cleared = false;
    for _ in 0..40 {
        step(&mut w);
        if !air::deck_marked(&w, a) {
            cleared = true;
            // Cleared in the climbing tick: the aircraft is above deck height, listed nowhere.
            assert!(!st(&w, a).ground);
            assert!(above_terrain(&w, a) > 416, "{}", describe(&w, a));
            assert_eq!(
                listed(&w, a),
                (false, false),
                "no listing once the listing height is positive"
            );
            break;
        }
        assert_eq!(
            air::height(&w, a),
            0,
            "the mark holds while it has not climbed"
        );
    }
    assert!(
        cleared,
        "the first climbing tick cleared the mark: {}",
        describe(&w, a)
    );
}

// spec: S-0501 claim 43 (a touchdown refused under claim 7 clears the mark as take-off restarts) and claim 7 (a refused cell: it rises by 10 leptons and starts to take off again)
#[test]
fn ac27_a_refused_touchdown_clears_the_mark() {
    let mut w = game(&rules());
    let goal = bridge(&mut w, 5, 0);
    let a = put(&mut w, Kind::Aircraft, "SCOUT", 0, -3, 0);
    click(&mut w, a, ClickOn::Cell(goal));
    assert!(run_until(&mut w, 200, |w| air::height(w, a) > 100));
    // A blocker arrives on the ground layer of the landing cell after the movement point is accepted (claim 7).
    testkit::spawn(&mut w, Kind::Vehicle, "VICTIM", 0, goal, 0);
    let mut refused = false;
    for _ in 0..800 {
        step(&mut w);
        let s = st(&w, a);
        if s.taking_off && !s.ground && above_terrain(&w, a) == 426 {
            refused = true;
            assert!(
                !air::deck_marked(&w, a),
                "the refused touchdown cleared the mark"
            );
            break;
        }
        assert!(
            !(s.ground && w.obj(a).unwrap().pos.cell() == goal),
            "never lands on the occupied cell"
        );
    }
    assert!(
        refused,
        "the touchdown on the deck was refused: {}",
        describe(&w, a)
    );
}

// spec: S-0501 claim 43 (horizontal flight or hovering alone, without one of these altitude or landing transitions, leaves the mark unchanged: a previously marked aircraft does not lose it merely because it is above the deck or crosses a cell boundary; a positive-height aircraft may retain a mark when it is not climbing, so the render layer alone cannot recover it); S-0209 claim 13 (a positive listing height lists nowhere, the mark notwithstanding)
#[test]
fn ac27_a_marked_aircraft_at_the_held_height_keeps_its_mark_across_cells() {
    let mut r = rules();
    // A cruise height that equals the deck-relative height the marked aircraft is set at.
    r.sim.aircraft.get_mut("SCOUT").unwrap().econ.flight_level = Some(100);
    let mut w = game(&r);
    let a = put(&mut w, Kind::Aircraft, "SCOUT", 0, -3, 0);
    // Marked, 100 leptons above the deck: neither climbing nor descending towards its cruise height 100.
    {
        let p = w.obj(a).unwrap().pos;
        let surface = w.surface_at(p.x, p.y);
        let o = w.obj_mut(a).unwrap();
        o.pos.z = surface + 416 + 100;
        let s = o.air.as_mut().unwrap();
        s.deck = true;
        s.ground = false;
        s.held = 100;
        s.cur = nd("1");
    }
    glasnost_sim::air::sync_listing(&mut w, a);
    assert_eq!(air::height(&w, a), 100);
    assert_eq!(listed(&w, a), (false, false), "positive listing height");
    let start = w.obj(a).unwrap().pos.cell();
    click(&mut w, a, ClickOn::Cell(testkit::mid(SIZE, 9, 0)));
    for _ in 0..40 {
        step(&mut w);
    }
    assert_ne!(
        w.obj(a).unwrap().pos.cell(),
        start,
        "it crossed cell boundaries"
    );
    assert!(
        air::deck_marked(&w, a),
        "the mark is unchanged by horizontal flight: {}",
        describe(&w, a)
    );
    assert_eq!(above_terrain(&w, a), 516);
    assert_eq!(listed(&w, a), (false, false));
}

// spec: S-0209 claim 13 (an aircraft using ordinary flight is listed in the ground layer exactly while its listing height is 0 or below and its deck mark is clear; a positive listing height excludes it from both lists; taking off removes the listing and landing adds it again) and S-0501 claim 7 (the ground layer exactly at a height of 0 or below)
#[test]
fn s0209_claim_13_ordinary_flight_is_listed_only_at_height_zero() {
    let mut w = game(&rules());
    let a = put(&mut w, Kind::Aircraft, "SCOUT", 0, -3, 0);
    assert_eq!(listed(&w, a), (true, false), "landed: listed on ground");
    click(&mut w, a, ClickOn::Cell(testkit::mid(SIZE, 6, 0)));
    let (mut flew, mut landed) = (false, false);
    for _ in 0..800 {
        step(&mut w);
        let on = above_terrain(&w, a) <= 0;
        assert_eq!(listed(&w, a), (on, false), "{}", describe(&w, a));
        flew |= !on;
        if flew && on && st(&w, a).ground {
            landed = true;
            break;
        }
    }
    assert!(
        flew && landed,
        "took off and landed again: {}",
        describe(&w, a)
    );
}

// ---------------------------------------------------------------------------
// S-0209 revision 37, claim 1 (other objects) and claim 5's bridge-deck paragraph, with S-0501 claim 44
// (Q-0872 answered): the object-present flag of an ordinary aircraft at rest, and the vehicle entry
// consequences. Scenario AC27, occupancy and vehicle variants.

/// The object-present flags of a cell: (ground, deck).
fn flags(w: &World, c: glasnost_map::CellXY) -> (u8, u8) {
    use glasnost_sim::world::Layer;
    (
        w.cells.occ(c, Layer::Ground).unwrap().others,
        w.cells.occ(c, Layer::Deck).unwrap().others,
    )
}

// spec: S-0209 claim 1 (other objects: rest on terrain marks ground) and S-0501 claim 44 (ordinary rest on terrain marks ground; the deck record is untouched)
#[test]
fn ac27_rest_on_terrain_marks_the_ground_record_only() {
    let mut w = game(&rules());
    let a = put(&mut w, Kind::Aircraft, "SCOUT", 0, -3, 0);
    let c = w.obj(a).unwrap().pos.cell();
    assert_eq!(flags(&w, c), (1, 0));
    // Landing on terrain after a flight marks ground as well.
    let goal = testkit::mid(SIZE, 6, 0);
    click(&mut w, a, ClickOn::Cell(goal));
    assert!(run_until(&mut w, 800, |w| st(w, a).ground
        && w.obj(a).unwrap().pos.cell() == goal));
    assert_eq!(flags(&w, goal), (1, 0));
    assert_eq!(flags(&w, c), (0, 0), "the take-off cleared the first cell");
}

// spec: S-0209 claim 1 (other objects: rest on a high-bridge deck marks only deck, not both records; the deck is used when a high bridge is present and the actual world height is at least the terrain surface height plus 416 leptons) and S-0501 claim 44 (the flag is set in the deck record only; the aircraft is listed on deck)
// spec: S-0209 claim 1 and S-0501 claim 44 (normal deck take-off clears the deck flag in the first climbing movement tick, without transferring it to ground; accepting the take-off destination alone does not release it; by the end of that tick the aircraft is absent from both occupant lists)
#[test]
fn ac27_deck_rest_marks_deck_only_and_take_off_clears_it_in_the_first_climbing_tick() {
    let mut w = game(&rules());
    let goal = bridge(&mut w, 5, 0);
    let a = put(&mut w, Kind::Aircraft, "SCOUT", 0, -3, 0);
    click(&mut w, a, ClickOn::Cell(goal));
    assert!(run_until(&mut w, 800, |w| st(w, a).ground && air::deck_marked(w, a)));
    assert_eq!(flags(&w, goal), (0, 1), "deck only, not ground");
    assert_eq!(listed(&w, a), (false, true));
    // A few idle ticks change nothing.
    testkit::run(&mut w, 5);
    assert_eq!(flags(&w, goal), (0, 1));
    let far = testkit::mid(SIZE, -9, 3);
    click(&mut w, a, ClickOn::Cell(far));
    assert_eq!(
        flags(&w, goal),
        (0, 1),
        "the accepted destination alone releases nothing"
    );
    let mut climbed = false;
    for _ in 0..40 {
        step(&mut w);
        if above_terrain(&w, a) > 416 {
            climbed = true;
            assert_eq!(
                flags(&w, goal),
                (0, 0),
                "cleared in the first climbing tick, not moved to ground"
            );
            assert_eq!(listed(&w, a), (false, false));
            break;
        }
        assert_eq!(
            flags(&w, goal),
            (0, 1),
            "held until it climbs: {}",
            describe(&w, a)
        );
    }
    assert!(climbed, "{}", describe(&w, a));
}

// spec: S-0209 claim 5 (the bridge-deck paragraph: the object-present flag alone gives no vehicle entry refusal on either layer; an ordinary allied aircraft at rest on deck without a destination is listed only on deck; a vehicle approaching below gets clear, one approaching on deck gets stationary friendly from the listed aircraft under row VP20; after take-off removes the listing, deck entry is clear) and S-0501 claim 44 (vehicles below and on deck)
#[test]
fn ac27_a_resting_deck_aircraft_refuses_only_the_deck_vehicle_through_its_listing() {
    use glasnost_sim::occupancy::{self, Outcome};
    let mut w = game(&rules());
    let goal = bridge(&mut w, 5, 0);
    let a = put(&mut w, Kind::Aircraft, "SCOUT", 0, -3, 0);
    let v = put(&mut w, Kind::Vehicle, "VICTIM", 0, -8, 5);
    click(&mut w, a, ClickOn::Cell(goal));
    assert!(run_until(&mut w, 800, |w| st(w, a).ground && air::deck_marked(w, a)));
    assert_eq!(flags(&w, goal), (0, 1));
    let level = i32::from(w.cells.cell(goal).unwrap().level);
    assert_eq!(
        occupancy::vehicle_outcome(&w, v, goal, level),
        Outcome::Clear,
        "below the bridge"
    );
    assert_eq!(
        occupancy::vehicle_outcome(&w, v, goal, level + 4),
        Outcome::StationaryFriendly,
        "on deck, from the listed aircraft (row VP20)"
    );
    // After take-off the listing and the flag are gone: deck entry is clear.
    click(&mut w, a, ClickOn::Cell(testkit::mid(SIZE, -9, 3)));
    assert!(run_until(&mut w, 40, |w| above_terrain(w, a) > 416));
    assert_eq!(
        occupancy::vehicle_outcome(&w, v, goal, level + 4),
        Outcome::Clear
    );
}

// ---------------------------------------------------------------------------
// Drain attempts in attack runs (claims 12, 15 and 16; S-0608 claim 16)

/// [`rules`] with the draining weapons Suck (burst 2, the Rocket
/// projectile) and SuckS (burst 2, the strafing projectile Straight), the
/// aircraft DJET, DSTRAFE and DFIGHT (a fighter) with 3 rounds each, and the
/// drainable 2 × 2 building DRB.
fn drain_rules() -> Ruleset {
    let mut r = rules();
    r.sim.general.economy.curley_shuffle = Some(true);
    for (n, proj) in [("Suck", "Rocket"), ("SuckS", "Straight")] {
        r.sim.weapons.insert(
            n.into(),
            WeaponType {
                damage: Some(1),
                rof: Some(30),
                range: dec("6"),
                speed: Some(60),
                burst: Some(2),
                projectile: Some(proj.into()),
                warhead: Some("HE".into()),
                drain_weapon: Some(true),
                ..Default::default()
            },
        );
    }
    for (n, u) in [
        ("DJET", flyer(Some("Suck"), 3)),
        ("DSTRAFE", flyer(Some("SuckS"), 3)),
        ("DFIGHT", {
            let mut u = flyer(Some("Suck"), 3);
            u.econ.fighter = Some(true);
            u
        }),
    ] {
        push(&mut r, "aircraft", n);
        r.sim.aircraft.insert(n.into(), u);
    }
    let mut b = econ_common::plain(500);
    b.foundation = Some([2, 2]);
    b.combat.drainable = Some(true);
    push(&mut r, "buildings", "DRB");
    r.sim.buildings.insert("DRB".into(), b);
    r
}

// spec: S-0501 claim 12 (Drain continuation: an attempt that clears the object target of an aircraft in play cancels the remaining burst attempts and the pass's scatter and enters done with a 1-tick delay; it neither restores the target nor undoes a formed link) and claim 15 (the admitted initial pass makes its round pending, spent once at the done boundary); S-0608 claim 16
#[test]
fn claim_12_drain_departure() {
    let r = drain_rules();
    for name in ["DJET", "DSTRAFE", "DFIGHT"] {
        let mut w = game(&r);
        ready(&mut w);
        let b = building(&mut w, "DRB", 1, 8, 0);
        let a = put(&mut w, Kind::Aircraft, name, 0, -4, 0);
        assert_eq!(air::strafer(&w, a), name == "DSTRAFE");
        click(&mut w, a, ClickOn::Object(b));
        assert!(
            run_until(&mut w, 200, |w| w.obj(a).unwrap().target.is_some()),
            "{name}: the order applies"
        );
        let cleared = run_until(&mut w, 3000, |w| w.obj(a).unwrap().target.is_none());
        assert!(cleared, "{name}: the attempt clears the target");
        let s = st(&w, a);
        assert_eq!(s.stage, air::Stage::Done, "{name}: {}", describe(&w, a));
        assert!(s.pending, "{name}: the round is pending");
        assert_eq!(
            w.obj(a).unwrap().combat.ammo,
            3,
            "{name}: nothing spent yet"
        );
        assert!(w.projectiles.is_empty(), "{name}: no ordinary shot");
        let link = glasnost_sim::drain::partner(&w, a);
        assert!(link.is_none() || link == Some(b));
        step(&mut w);
        assert_eq!(
            w.obj(a).unwrap().combat.ammo,
            2,
            "{name}: one round, at done"
        );
        assert!(!st(&w, a).pending);
        for _ in 0..5 {
            step(&mut w);
            assert_eq!(w.obj(a).unwrap().target, None, "{name}: not restored");
        }
        assert_eq!(
            glasnost_sim::drain::partner(&w, a),
            link,
            "{name}: the link stays"
        );
        assert_eq!(w.obj(a).unwrap().combat.ammo, 2, "{name}: no second round");
    }
}

// ---------------------------------------------------------------------------
// An attacker that loses its target during the approach (the lead's play
// note 5 of M8 step A4)

/// The aircraft ruleset with JET1, JET with the one dock type APAD, and
/// `edit` applied to it.
fn one_dock(edit: impl FnOnce(&mut UnitType)) -> Ruleset {
    let mut r = rules();
    let mut u = flyer(Some("Agm"), 2);
    u.econ.dock = Some(vec!["APAD".into()]);
    edit(&mut u);
    push(&mut r, "aircraft", "JET1");
    r.sim.aircraft.insert("JET1".into(), u);
    r
}

/// How the attacker's target is lost.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Loss {
    /// Destroyed by a hit from nobody, as by another unit's fire.
    Kill,
    /// Removed from the game without a kill.
    Remove,
    /// Taken off the map without ending it (inside a transport or a
    /// building).
    Inside,
}

/// Orders aircraft `a` to attack a fresh VICTIM 12 cells east of the map
/// middle. Once the aircraft is 4 cells further east, before any shot,
/// `before` runs and the victim is lost as `loss` says.
fn lose_target_on_approach(
    w: &mut World,
    a: ObjectId,
    loss: Loss,
    before: impl FnOnce(&mut World),
) -> ObjectId {
    let v = put(w, Kind::Vehicle, "VICTIM", 1, 12, 0);
    let full = w.obj(v).unwrap().health;
    let x0 = w.obj(a).unwrap().pos.x;
    click(w, a, ClickOn::Object(v));
    let out = run_until(w, 600, |w| w.obj(a).unwrap().pos.x >= x0 + 4 * 256);
    assert!(out, "on its approach: {}", describe(w, a));
    assert_eq!(w.obj(a).unwrap().target, Some(Target::Object(v)));
    assert_eq!(w.obj(v).unwrap().health, full, "no shot yet");
    assert!(w.projectiles.is_empty(), "no shot in flight");
    before(w);
    match loss {
        Loss::Kill => {
            glasnost_sim::combat::damage::apply(w, v, hit(1_000_000));
            assert!(
                run_until(w, 200, |w| w.obj(v).is_none()),
                "the victim is gone"
            );
        }
        Loss::Remove => glasnost_sim::objects::remove(w, v),
        Loss::Inside => glasnost_sim::objects::take_inside(w, v),
    }
    v
}

// spec: S-0501 F10 and Import mapping (FlightLevel absent: cruise height 500)
const CRUISE: i32 = 500;

/// After the loss: the aircraft keeps its cruise height and full
/// ammunition and does not land until its pad `k` grants contact and it
/// enters; it then lands on the pad at its docking position, contact held.
fn flies_home_and_docks(w: &mut World, k: ObjectId, a: ObjectId) {
    let mut entered = false;
    for _ in 0..3000 {
        step(w);
        let o = w.obj(a).expect("alive");
        assert_eq!(o.combat.ammo, 2, "ammunition full: {}", describe(w, a));
        let s = st(w, a);
        if o.contact == Some(k) && o.beh.current == Some(Behaviour::Enter) {
            entered = true;
        }
        if !entered {
            // spec: S-0501 claim 6 (in ordinary flight the held height is the cruise height; no landing trigger before the pad grants contact)
            assert!(!s.landing, "no landing before contact: {}", describe(w, a));
            assert!(!air::on_ground(w, a), "in flight: {}", describe(w, a));
            assert_eq!(s.held, CRUISE, "held height: {}", describe(w, a));
        }
        if air::on_ground(w, a) && o.beh.current != Some(Behaviour::Enter) {
            break;
        }
    }
    assert!(
        entered,
        "the pad grants contact and it enters: {}",
        describe(w, a)
    );
    let o = w.obj(a).unwrap();
    assert_eq!(o.contact, Some(k), "contact held: {}", describe(w, a));
    // spec: S-0501 claim 26 (the pad's service step releases an aircraft with no target, full ammunition and full health: it falls back to rest with its guard forced; it may have docked in sleep first, claim 24)
    assert!(
        matches!(o.beh.current, Some(Behaviour::Sleep | Behaviour::Guard)),
        "docked: {}",
        describe(w, a)
    );
    let (p, _) = glasnost_sim::econ::release::pad_position(w, k, a);
    assert_eq!(o.pos, p, "at its docking position");
}

// spec: S-0501 claims 6, 17, 18, 21 to 24, 32 and table T2 (a landable armed aircraft with one dock type whose target is destroyed during its approach: the run ends, it falls back to rest, the pad grants contact, it enters the pad at its held cruise height with its ammunition full and docks; the held flight point keeps claim 6's replacement made at acceptance, so it does not start to land at the old firing spot)
#[test]
fn lost_target_on_approach_flies_home_and_docks() {
    let r = one_dock(|_| {});
    let (mut w, k, a) = built_with(&r, "JET1");
    lose_target_on_approach(&mut w, a, Loss::Kill, |_| {});
    flies_home_and_docks(&mut w, k, a);
}

// spec: S-0501 claim 32 (Target loss: a target removed from the game is lost at that moment, its targeters' target cleared in the turn of the event) and claims 17 and 18 (it then flies home as for a kill)
#[test]
fn removed_target_on_approach_flies_home_and_docks() {
    let r = one_dock(|_| {});
    let (mut w, k, a) = built_with(&r, "JET1");
    lose_target_on_approach(&mut w, a, Loss::Remove, |_| {});
    assert_eq!(w.obj(a).unwrap().target, None, "lost at that moment");
    flies_home_and_docks(&mut w, k, a);
}

// spec: S-0501 claim 32 (Target loss: a target taken off the map, inside a transport or building, is lost as a removed one is, Q-1755 answered) and claims 17 and 18 (the run ends and it flies home with its ammunition full)
#[test]
fn target_taken_inside_is_lost_and_it_flies_home() {
    let r = one_dock(|_| {});
    let (mut w, k, a) = built_with(&r, "JET1");
    let v = lose_target_on_approach(&mut w, a, Loss::Inside, |_| {});
    assert!(w.obj(v).is_some(), "the victim still exists");
    assert_eq!(w.obj(a).unwrap().target, None, "lost at that moment");
    flies_home_and_docks(&mut w, k, a);
}

// spec: S-0501 claim 45's visiting case and table T2 (in flight, armed, visiting, ammunition left after its target is lost, outside a team: hunt, keeping claim 17's edge destination while its hunt finds no target; with neither flight flag it is removed, without dying, when its position leaves the usable area, having entered it since it appeared, claim 10). The type is not landable, so its placement's type test marks it visiting; its placement fall-back came first, so it rests in guard on the ground and takes the attack order (claim 1 and Terms, Q-1838).
#[test]
fn lost_target_visiting_hunts_toward_the_edge_and_is_removed_at_the_border() {
    let r = one_dock(|u| u.econ.landable = Some(false));
    let mut w = banded_game(&r);
    ready(&mut w);
    let a = put(&mut w, Kind::Aircraft, "JET1", 0, 0, 0);
    assert!(st(&w, a).visiting, "marked by the type test");
    assert!(air::on_ground(&w, a), "placed on the ground");
    let losses = glasnost_sim::econ::stats::of(&w, ALPHA).losses;
    lose_target_on_approach(&mut w, a, Loss::Remove, |_| {});
    let hunts = run_until(&mut w, 100, |w| {
        w.obj(a).unwrap().beh.current == Some(Behaviour::Hunt)
    });
    assert!(hunts, "hunt: {}", describe(&w, a));
    let e = dest_cell(&w, a).expect("a destination");
    assert!(outside(&w, e), "claim 17's edge cell: {e:?}");
    let mut last = None;
    for _ in 0..3000 {
        let Some(o) = w.obj(a) else { break };
        assert!(o.health > 0, "alive: {}", describe(&w, a));
        assert_eq!(
            dest_cell(&w, a),
            Some(e),
            "the edge cell kept: {}",
            describe(&w, a)
        );
        assert!(!air::on_ground(&w, a), "never lands: {}", describe(&w, a));
        last = Some(o.pos.cell());
        step(&mut w);
    }
    assert!(w.obj(a).is_none(), "removed on the way");
    let at = last.unwrap();
    assert!(
        w.cells.usable(at),
        "removed as it leaves the usable area: {at:?}"
    );
    // spec: S-0501 claim 10 (removed without dying: no loss counted)
    assert_eq!(glasnost_sim::econ::stats::of(&w, ALPHA).losses, losses);
}

// spec: S-0501 claims 22 and 23 (a pad whose every contact place is held grants no contact) and table T2 (in flight, armed, no pad, not pad-bound: the landing spot of F8 as destination and move); claim 6 (without a target the new point is not raised: it lands there)
#[test]
fn lost_target_every_place_reserved_lands_at_a_landing_spot() {
    let r = one_dock(|_| {});
    let (mut w, k, a) = built_with(&r, "JET1");
    lose_target_on_approach(&mut w, a, Loss::Kill, |w| {
        assert_eq!(
            w.obj(a).unwrap().contact,
            None,
            "the take-off ended the reservation"
        );
        let other = put(w, Kind::Aircraft, "SCOUT", 0, -3, 3);
        assert!(
            air::request_contact(w, other, k),
            "another aircraft holds the place"
        );
    });
    let moves = run_until(&mut w, 100, |w| {
        w.obj(a).unwrap().beh.current == Some(Behaviour::Move)
    });
    assert!(moves, "move: {}", describe(&w, a));
    let Some(Target::Cell(spot)) = w.obj(a).unwrap().dest else {
        panic!("a landing spot: {}", describe(&w, a));
    };
    let landed = run_until(&mut w, 600, |w| air::on_ground(w, a));
    assert!(landed, "lands: {}", describe(&w, a));
    let o = w.obj(a).unwrap();
    assert_eq!(o.pos.cell(), spot, "at the landing spot");
    assert_eq!(o.contact, None);
    assert_eq!(o.combat.ammo, 2);
}

// spec: S-0501 claim 40 (the selection runs at the first processing of every ordinary move, also one started inside another behaviour's processing: here claim 17's run end, whose fall-back, table T2's no-pad row, gives the F8 landing spot and starts move in the attack's processing; a landing spot that F7 refuses by then, here one holding a vehicle, is replaced at that first processing, one tick later) and claim 6 (the replacement point, given without a target, is not raised). The target is lost on the approach while every place of the pad is held.
#[test]
fn claim_40_a_run_end_move_takes_the_selection() {
    let r = one_dock(|_| {});
    let (mut w, k, a) = built_with(&r, "JET1");
    lose_target_on_approach(&mut w, a, Loss::Kill, |w| {
        let other = put(w, Kind::Aircraft, "SCOUT", 0, -3, 3);
        assert!(
            air::request_contact(w, other, k),
            "another aircraft holds the place"
        );
    });
    let moves = run_until(&mut w, 300, |w| {
        w.obj(a).unwrap().beh.current == Some(Behaviour::Move)
    });
    assert!(moves, "move: {}", describe(&w, a));
    let Some(Target::Cell(spot)) = w.obj(a).unwrap().dest else {
        panic!("a landing spot: {}", describe(&w, a));
    };
    let blocker = testkit::spawn(&mut w, Kind::Vehicle, "VICTIM", 0, spot, 0);
    let bc = w.obj(blocker).unwrap().pos.cell();
    assert_eq!(bc, spot, "the vehicle stands on the landing spot");
    step(&mut w);
    let Some(Target::Cell(c)) = w.obj(a).unwrap().dest else {
        panic!("a substituted cell: {}", describe(&w, a));
    };
    assert_ne!(
        c,
        spot,
        "replaced at the first processing: {}",
        describe(&w, a)
    );
    assert_eq!(
        st(&w, a).flying_to,
        Some(Target::Cell(c)),
        "given to the flight"
    );
    assert!(!st(&w, a).raised, "given without a target, not raised");
}

// spec: S-0501 claim 6 (the replacement is made when the movement point is accepted with a target and ammunition; the held point keeps it after the target goes; a destination that is not the held one reads the state now)
#[test]
fn claim_6_held_point_height_fixed_at_acceptance() {
    let mut w = game(&rules());
    ready(&mut w);
    let v = put(&mut w, Kind::Vehicle, "VICTIM", 1, 8, 0);
    let a = put(&mut w, Kind::Aircraft, "JET", 0, 0, 0);
    let c = testkit::mid(SIZE, 4, 4);
    let centre = glasnost_sim::world::Pos::centre(c, 0);
    let ground = w.surface_at(centre.x, centre.y);
    w.obj_mut(a).unwrap().target = Some(Target::Object(v));
    assert!(orders::issue(
        &mut w,
        ALPHA,
        Order::Behaviour {
            actor: a,
            behaviour: Behaviour::Move,
            attack_move: false,
            target: Some(Target::Object(v)),
            dest: Some(Target::Cell(c)),
        }
    ));
    let held = run_until(&mut w, 50, |w| st(w, a).flying_to == Some(Target::Cell(c)));
    assert!(held, "the point is accepted: {}", describe(&w, a));
    assert_eq!(air::flight_point(&w, a, Target::Cell(c)).z, ground + CRUISE);
    w.obj_mut(a).unwrap().target = None;
    assert_eq!(
        air::flight_point(&w, a, Target::Cell(c)).z,
        ground + CRUISE,
        "the held point keeps its height"
    );
    let other = testkit::mid(SIZE, -4, 4);
    let o = glasnost_sim::world::Pos::centre(other, 0);
    assert_eq!(
        air::flight_point(&w, a, Target::Cell(other)).z,
        w.surface_at(o.x, o.y),
        "another destination: no target now"
    );
}

// ---------------------------------------------------------------------------
// After the run (S-0501 r40: claims 17, 32, 34, 37, 40, 45 and 46)

/// Whether `c` lies outside the usable area (S-0522 claim 4's edge cells).
fn outside(w: &World, c: glasnost_map::CellXY) -> bool {
    !w.cells.usable(c)
}

/// The aircraft's cell destination.
fn dest_cell(w: &World, a: ObjectId) -> Option<glasnost_map::CellXY> {
    match w.obj(a)?.dest {
        Some(Target::Cell(c)) => Some(c),
        _ => None,
    }
}

/// Runs until the aircraft's destination is a cell outside the usable area
/// and returns that cell.
fn edge_destination(w: &mut World, a: ObjectId) -> glasnost_map::CellXY {
    let found = run_until(w, 200, |w| dest_cell(w, a).is_some_and(|c| outside(w, c)));
    assert!(found, "an edge destination: {}", describe(w, a));
    dest_cell(w, a).unwrap()
}

/// After claim 45's replacement: the aircraft flies to the usable cell
/// `c`, lands there and rests in guard, alive, with `ammo` rounds.
fn lands_at_and_rests(w: &mut World, a: ObjectId, c: glasnost_map::CellXY, ammo: i32) {
    let landed = run_until(w, 3000, |w| air::on_ground(w, a));
    assert!(landed, "lands: {}", describe(w, a));
    assert_eq!(w.obj(a).unwrap().pos.cell(), c, "at the replacement cell");
    for _ in 0..300 {
        step(w);
        let o = w.obj(a).expect("not removed");
        assert!(o.health > 0, "no crash: {}", describe(w, a));
        assert!(air::on_ground(w, a), "rests: {}", describe(w, a));
        assert_eq!(o.combat.ammo, ammo);
    }
    // spec: S-0501 T2 (on the ground: guard, for a human-controlled house)
    assert_eq!(w.obj(a).unwrap().beh.current, Some(Behaviour::Guard));
}

/// Claim 45's replacement: the destination `e` outside the usable area is
/// replaced by a usable cell near it at the first processing of the
/// *move* that *guard* queues; returns the replacement.
fn replaced_near(w: &mut World, a: ObjectId, e: glasnost_map::CellXY) -> glasnost_map::CellXY {
    let moved = run_until(w, 200, |w| dest_cell(w, a) != Some(e));
    assert!(moved, "the destination is replaced: {}", describe(w, a));
    assert_eq!(w.obj(a).unwrap().beh.current, Some(Behaviour::Move));
    let c = dest_cell(w, a).expect("a cell");
    assert!(w.cells.usable(c), "a usable cell: {c:?}");
    assert!(
        cheb(c, e) <= 6,
        "a few cells from the edge cell: {c:?} {e:?}"
    );
    c
}

// spec: S-0501 claims 17, 40 and 45 and table T2 (an armed, landable aircraft with no dock types of a human-controlled house whose target is lost before it fires: the run end gives it a cell on its house's computed edge outside the usable area; guard keeps it; the move that guard queues replaces it at its first processing with a usable cell near it; the aircraft lands there and rests in guard, not removed and not crashed, its ammunition unspent; Q-1756 answered)
#[test]
fn lost_target_no_dock_types_heads_for_the_edge_and_lands_near_it() {
    let r = one_dock(|u| u.econ.dock = None);
    let mut w = game(&r);
    ready(&mut w);
    let a = put(&mut w, Kind::Aircraft, "JET1", 0, 0, 0);
    lose_target_on_approach(&mut w, a, Loss::Remove, |_| {});
    let e = edge_destination(&mut w, a);
    let guard = run_until(&mut w, 100, |w| {
        w.obj(a).unwrap().beh.current == Some(Behaviour::Guard)
    });
    assert!(guard, "guard: {}", describe(&w, a));
    let c = replaced_near(&mut w, a, e);
    lands_at_and_rests(&mut w, a, c, 2);
}

// spec: S-0501 claims 17, 40 and 45 (an armed aircraft with no dock types whose ammunition is spent: the edge destination, its replacement near the edge, a landing there and rest in guard; it neither lands at nor holds above its last firing spot). The re-approach flag is on so that each run ends (claim 14).
#[test]
fn spent_no_dock_types_heads_for_the_edge_and_lands_near_it() {
    let mut r = one_dock(|u| u.econ.dock = None);
    r.sim.general.economy.curley_shuffle = Some(true);
    let mut w = game(&r);
    ready(&mut w);
    let v = put(&mut w, Kind::Vehicle, "VICTIM", 1, 1, 8);
    let a = put(&mut w, Kind::Aircraft, "JET1", 0, 0, -4);
    click(&mut w, a, ClickOn::Object(v));
    assert!(run_until(&mut w, 3000, |w| w.obj(a).unwrap().combat.ammo == 0));
    let e = edge_destination(&mut w, a);
    let c = replaced_near(&mut w, a, e);
    lands_at_and_rests(&mut w, a, c, 0);
}

// spec: S-0501 claims 32 and 37 (Target loss: an aircraft whose target object is killed, taken off the map or removed takes, when its targeting timer has more than 10 integer ticks left, one U(4, 8) draw that restarts the timer, before the target is cleared and in the turn of the event; with at most 10 ticks left, no draw)
#[test]
fn claim_37_target_loss_draw_both_ways() {
    let r = one_dock(|_| {});
    let (mut w, _k, a) = built_with(&r, "JET1");
    let v = put(&mut w, Kind::Vehicle, "VICTIM", 1, 12, 0);
    w.obj_mut(a).unwrap().target = Some(Target::Object(v));
    let half = w.rules().quirks.ranged_draw_half_span;
    // The draw comes first in the event: the raw values and the value one
    // U(4, 8) draw takes from the stream as it stands.
    let mut e = w.rng.clone();
    // spec: S-0501 claim 37 (Target loss: U(4, 8))
    let drawn = e.ranged(4, 8, half);
    let raws = e.taken() - w.rng.taken();
    for loss in [Loss::Kill, Loss::Inside, Loss::Remove] {
        let taken = |left: i32| {
            let mut c = w.clone();
            let tick = c.tick;
            c.obj_mut(a).unwrap().targeting_timer = tick + left;
            let before = c.rng.taken();
            match loss {
                Loss::Kill => {
                    glasnost_sim::combat::damage::apply(&mut c, v, hit(1_000_000));
                }
                Loss::Inside => glasnost_sim::objects::take_inside(&mut c, v),
                Loss::Remove => glasnost_sim::objects::remove(&mut c, v),
            }
            let o = c.obj(a).unwrap();
            assert_eq!(o.target, None, "{loss:?}: lost at that moment");
            (c.rng.taken() - before, o.targeting_timer - tick)
        };
        let (base, kept) = taken(10);
        assert_eq!(kept, 10, "{loss:?}: 10 ticks left, no restart");
        assert_eq!(taken(5), (base, 5), "{loss:?}: 5 ticks left, no draw");
        for left in [11, 50] {
            let (n, timer) = taken(left);
            assert_eq!(
                timer, drawn,
                "{loss:?}: {left} ticks left, restarted to the draw"
            );
            if loss != Loss::Kill {
                assert_eq!(n, base + raws, "{loss:?}: {left} ticks left, one draw");
            } else {
                assert!(n > base, "{loss:?}: {left} ticks left, the draw taken");
            }
        }
    }
}

// spec: S-0501 claim 32 and T1 (the approaching row with no target: done after 1 tick, and the processing of done then ends the run 1 tick later)
#[test]
fn claim_32_lost_target_done_after_one_tick() {
    let r = one_dock(|_| {});
    let (mut w, _k, a) = built_with(&r, "JET1");
    lose_target_on_approach(&mut w, a, Loss::Remove, |_| {});
    let done = run_until(&mut w, 100, |w| st(w, a).stage == air::Stage::Done);
    assert!(done, "done: {}", describe(&w, a));
    let last = w.obj(a).unwrap().beh.last;
    let (at, d) = last.expect("processed");
    assert_eq!(d, 1, "done after 1 tick");
    let next = run_until(&mut w, 10, |w| w.obj(a).unwrap().beh.last != last);
    assert!(next, "processed again: {}", describe(&w, a));
    let (then, _) = w.obj(a).unwrap().beh.last.unwrap();
    assert_eq!(then, at + 1, "the next processing comes 1 tick later");
    assert_ne!(st(&w, a).stage, air::Stage::Done, "it ends the run");
}

// spec: S-0501 claims 17 and 37 and scenario AC29 part a (an aircraft whose pad grants contact: the run end takes the edge-cell draws of S-0522 claim 4, then the fall-back gives it the pad as destination and enter, in the same processing, so it never heads for the edge)
#[test]
fn ac29a_run_end_takes_the_edge_draws_then_the_pad() {
    let r = one_dock(|_| {});
    let (mut w, k, a) = built_with(&r, "JET1");
    lose_target_on_approach(&mut w, a, Loss::Remove, |_| {});
    let done = run_until(&mut w, 100, |w| st(w, a).stage == air::Stage::Done);
    assert!(done, "done: {}", describe(&w, a));
    let last = w.obj(a).unwrap().beh.last;
    let mut ended = false;
    for _ in 0..10 {
        let before = w.rng.taken();
        step(&mut w);
        let o = w.obj(a).unwrap();
        assert!(
            !dest_cell(&w, a).is_some_and(|c| outside(&w, c)),
            "never an edge destination between turns: {}",
            describe(&w, a)
        );
        if o.beh.last != last {
            assert!(w.rng.taken() > before, "the run end's draws");
            assert_eq!(
                o.dest,
                Some(Target::Object(k)),
                "the pad: {}",
                describe(&w, a)
            );
            assert!(
                o.beh.current == Some(Behaviour::Enter) || o.beh.queued == Some(Behaviour::Enter),
                "enter: {}",
                describe(&w, a)
            );
            ended = true;
            break;
        }
    }
    assert!(ended, "the run ends: {}", describe(&w, a));
}

/// JET1 made pad-bound by the test rules, with `docks` as its dock types;
/// APAD has a build-up sprite of 10 frames, so that it can be sold and its
/// build-down completes (S-0408 claims 4 and 17, S-0108 claim 5).
fn pad_bound(docks: Option<Vec<String>>) -> Ruleset {
    let mut r = one_dock(|u| {
        u.econ.dock = docks;
        u.econ.airport_bound = Some(true);
    });
    let pad = r.sim.buildings.get_mut("APAD").unwrap();
    pad.econ.buildup = Some(true);
    pad.econ.buildup_frames = Some(10);
    r
}

/// Asserts that aircraft `a` has started claim 46's fall-back crash: health
/// 0 and a running fall counter, in flight. Returns its house's losses then.
fn crashed(w: &World, a: ObjectId) {
    let o = w.obj(a).expect("still falling");
    assert_eq!(
        o.health,
        0,
        "health 0 at the crash start: {}",
        describe(w, a)
    );
    assert!(st(w, a).crash.is_some(), "falling: {}", describe(w, a));
    assert!(air::height(w, a) > 0, "in flight");
}

/// Runs the crash to impact: the fall counter grows by 1 per tick and the
/// aircraft leaves play at impact; its own house counts one loss in all
/// and no other house a kill.
fn falls_to_impact(w: &mut World, a: ObjectId, losses_before: i32) {
    let stats = glasnost_sim::econ::stats::of;
    let kills: Vec<i32> = (0..2).map(|h| stats(w, HouseId(h)).kills).collect();
    let mut last = st(w, a).crash.unwrap();
    for _ in 0..600 {
        step(w);
        let Some(o) = w.obj(a) else { break };
        let c = st(w, a).crash.expect("still crashing");
        // spec: S-0501 claim 9 (a dead aircraft's fall counter grows by 1 per tick)
        assert_eq!(c, last + 1, "fall counter: {}", describe(w, a));
        assert_eq!(o.health, 0);
        last = c;
    }
    assert!(w.obj(a).is_none(), "it leaves play at impact");
    // spec: S-0501 claim 46 and S-0414 claim 21 (credited to nobody: one loss of its own house in all, no house's kills change)
    assert_eq!(stats(w, ALPHA).losses, losses_before + 1, "one loss");
    let after: Vec<i32> = (0..2).map(|h| stats(w, HouseId(h)).kills).collect();
    assert_eq!(after, kills, "no kill credited");
}

// spec: S-0501 claims 45 and 46 and T2's after-the-row entry (a pad-bound type with no dock types whose ammunition is spent crashes at the run end's fall-back: health 0 at once, a dead fall, one loss of its own house, no kill)
#[test]
fn pad_bound_no_dock_types_spent_crashes_at_the_run_end() {
    let mut r = pad_bound(None);
    r.sim.general.economy.curley_shuffle = Some(true);
    let mut w = game(&r);
    ready(&mut w);
    let v = put(&mut w, Kind::Vehicle, "VICTIM", 1, 1, 8);
    let a = put(&mut w, Kind::Aircraft, "JET1", 0, 0, -4);
    click(&mut w, a, ClickOn::Object(v));
    let losses = glasnost_sim::econ::stats::of(&w, ALPHA).losses;
    assert!(run_until(&mut w, 3000, |w| w.obj(a).unwrap().combat.ammo == 0));
    let crash = run_until(&mut w, 200, |w| w.obj(a).unwrap().health == 0);
    assert!(crash, "a crash: {}", describe(&w, a));
    crashed(&w, a);
    falls_to_impact(&mut w, a, losses);
}

// spec: S-0501 claim 45 (a pad-bound type with no dock types and ammunition left keeps claim 17's edge cell in guard without a move, since claim 33 queues move only for one that is not pad-bound; it does not land, by claim 6's pad-bound rule, and stays above the cell) and claim 6
#[test]
fn pad_bound_no_dock_types_with_ammunition_holds_over_the_edge_cell() {
    let r = pad_bound(None);
    let mut w = game(&r);
    ready(&mut w);
    let a = put(&mut w, Kind::Aircraft, "JET1", 0, 0, 0);
    lose_target_on_approach(&mut w, a, Loss::Remove, |_| {});
    let e = edge_destination(&mut w, a);
    for _ in 0..1500 {
        step(&mut w);
        let o = w.obj(a).expect("not removed");
        assert_eq!(
            dest_cell(&w, a),
            Some(e),
            "the edge cell kept: {}",
            describe(&w, a)
        );
        assert_ne!(o.beh.current, Some(Behaviour::Move), "{}", describe(&w, a));
        assert_ne!(o.beh.queued, Some(Behaviour::Move));
        assert!(o.health > 0, "no crash with ammunition left");
        assert!(!air::on_ground(&w, a), "never lands: {}", describe(&w, a));
        assert_eq!(o.combat.ammo, 2);
    }
    let o = w.obj(a).unwrap();
    assert_eq!(o.pos.cell(), e, "above the edge cell: {}", describe(&w, a));
}

// spec: S-0501 claim 46 and T2 (in flight, armed, no pad, pad-bound: a pad-bound aircraft with dock types whose house has no building of them crashes at its run end's fall-back, with its ammunition spent or with ammunition left after its target is lost)
#[test]
fn pad_bound_without_a_pad_crashes_at_the_run_end_both_ways() {
    for spent in [true, false] {
        let mut r = pad_bound(Some(vec!["APAD".into()]));
        r.sim.general.economy.curley_shuffle = Some(true);
        let mut w = game(&r);
        ready(&mut w);
        let a = put(&mut w, Kind::Aircraft, "JET1", 0, 0, -4);
        let losses = glasnost_sim::econ::stats::of(&w, ALPHA).losses;
        if spent {
            let v = put(&mut w, Kind::Vehicle, "VICTIM", 1, 1, 8);
            click(&mut w, a, ClickOn::Object(v));
            assert!(run_until(&mut w, 3000, |w| w.obj(a).unwrap().combat.ammo == 0));
        } else {
            lose_target_on_approach(&mut w, a, Loss::Remove, |_| {});
        }
        let crash = run_until(&mut w, 200, |w| w.obj(a).unwrap().health == 0);
        assert!(crash, "spent {spent}: a crash: {}", describe(&w, a));
        crashed(&w, a);
        falls_to_impact(&mut w, a, losses);
    }
}

// spec: S-0501 claim 46 (r110: the pad it flies to is sold during its approach: the sale reaches the aircraft in contact through the run-away notice, claim 24, which replaces the pad as destination with a landing spot and gives guard, so the fall-back that decides comes outside enter; no other pad grants contact, and a pad-bound aircraft in flight crashes then)
#[test]
fn pad_bound_pad_sold_during_the_approach_crashes() {
    let mut r = pad_bound(Some(vec!["APAD".into()]));
    r.sim.general.economy.curley_shuffle = Some(true);
    let (mut w, k, a) = built_with(&r, "JET1");
    let v = put(&mut w, Kind::Vehicle, "VICTIM", 1, 12, 0);
    click(&mut w, a, ClickOn::Object(v));
    assert!(run_until(&mut w, 3000, |w| w.obj(a).unwrap().combat.ammo == 0));
    let home = run_until(&mut w, 200, |w| {
        w.obj(a).unwrap().dest == Some(Target::Object(k))
    });
    assert!(home, "flies home: {}", describe(&w, a));
    let losses = glasnost_sim::econ::stats::of(&w, ALPHA).losses;
    open(&mut w);
    assert!(orders::issue(&mut w, ALPHA, Order::Sell { actor: k }));
    let crash = run_until(&mut w, 200, |w| w.obj(a).unwrap().health == 0);
    assert!(crash, "a crash: {}", describe(&w, a));
    assert_ne!(
        w.obj(a).unwrap().dest,
        Some(Target::Object(k)),
        "the pad is no longer its destination"
    );
    crashed(&w, a);
    falls_to_impact(&mut w, a, losses);
}

// spec: S-0501 claim 46 (r110: an aircraft standing on its pad when the pad is sold still holds its contact, so the run-away notice reaches it, claim 24: it holds no contact and has a cell as destination, not the pad, at the end of the sale's first tick; in guard the move the notice queues stays queued, not guard; the landing spot makes it take off, claim 8, and the fall-back that decides comes in flight, at cruise height or over that cell, where a pad-bound aircraft does not land, so with no other pad it crashes in flight with its ammunition full) and claim 26 (guard current once released); AC29 part f (4)
#[test]
fn ac29f4_pad_bound_standing_on_its_pad_when_sold_takes_off_and_crashes() {
    let mut r = pad_bound(Some(vec!["APAD".into()]));
    r.sim.general.economy.curley_shuffle = Some(true);
    let (mut w, k, a) = built_with(&r, "JET1");
    assert!(air::on_ground(&w, a), "standing on its pad");
    assert!(run_until(&mut w, 200, |w| w.obj(a).unwrap().beh.current
        == Some(Behaviour::Guard)
        && w.obj(a).unwrap().beh.queued.is_none()));
    assert_eq!(w.obj(a).unwrap().combat.ammo, 2, "full ammunition");
    let losses = glasnost_sim::econ::stats::of(&w, ALPHA).losses;
    open(&mut w);
    assert!(orders::issue(&mut w, ALPHA, Order::Sell { actor: k }));
    let selling = run_until(&mut w, 20, |w| {
        w.obj(k).unwrap().beh.current == Some(Behaviour::Selling)
    });
    assert!(selling, "the sale");
    let o = w.obj(a).unwrap();
    assert_eq!(o.contact, None, "{}", describe(&w, a));
    assert!(
        matches!(o.dest, Some(Target::Cell(_))),
        "a landing spot: {}",
        describe(&w, a)
    );
    assert!(
        o.beh.current == Some(Behaviour::Move) || o.beh.queued == Some(Behaviour::Move),
        "move queued or current: {}",
        describe(&w, a)
    );
    assert_ne!(o.beh.queued, Some(Behaviour::Guard), "not guard queued");
    let h = air::height(&w, a);
    assert!(run_until(&mut w, 20, |w| air::height(w, a) > h), "it rises");
    let crash = run_until(&mut w, 3000, |w| w.obj(a).unwrap().health == 0);
    assert!(crash, "a crash: {}", describe(&w, a));
    crashed(&w, a);
    pad_removed_then_falls(&mut w, k, a, losses);
}

/// Runs until the sold pad `k` leaves play, then runs aircraft `a`'s crash
/// to impact. The pad comes before the aircraft in the update order, so its
/// removal in its own turn costs the aircraft that tick's turn (S-0203 claim
/// 10, `quirks.update_order_removal_skip`), and the fall counter does not
/// grow in that tick.
fn pad_removed_then_falls(w: &mut World, k: ObjectId, a: ObjectId, losses: i32) {
    assert!(run_until(w, 600, |w| w.obj(k).is_none()), "sold");
    assert!(w.obj(a).is_some(), "still falling when the pad is removed");
    crashed(w, a);
    falls_to_impact(w, a, losses);
}

// spec: S-0501 claim 46 (r110: an aircraft standing on its pad after its run with no ammunition, in sleep when the pad is sold, takes the notice there: guard queued with an immediate start, so guard is queued or current; no contact, a cell as destination; it takes off and crashes in flight) and claim 24; AC29 part f (6)
#[test]
fn ac29f6_pad_bound_asleep_on_its_pad_with_no_ammunition_when_sold_crashes() {
    let mut r = pad_bound(Some(vec!["APAD".into()]));
    r.sim.general.economy.curley_shuffle = Some(true);
    let (mut w, k, a) = built_with(&r, "JET1");
    let v = put(&mut w, Kind::Vehicle, "VICTIM", 1, 12, 0);
    click(&mut w, a, ClickOn::Object(v));
    assert!(run_until(&mut w, 3000, |w| w.obj(a).unwrap().combat.ammo == 0));
    let asleep = run_until(&mut w, 3000, |w| {
        w.obj(a).unwrap().beh.current == Some(Behaviour::Sleep)
    });
    assert!(asleep, "sleep on its pad: {}", describe(&w, a));
    assert_eq!(w.obj(a).unwrap().combat.ammo, 0);
    let losses = glasnost_sim::econ::stats::of(&w, ALPHA).losses;
    open(&mut w);
    assert!(orders::issue(&mut w, ALPHA, Order::Sell { actor: k }));
    let selling = run_until(&mut w, 20, |w| {
        w.obj(k).unwrap().beh.current == Some(Behaviour::Selling)
    });
    assert!(selling, "the sale");
    let o = w.obj(a).unwrap();
    assert_eq!(o.contact, None, "{}", describe(&w, a));
    assert!(
        matches!(o.dest, Some(Target::Cell(_))),
        "a landing spot: {}",
        describe(&w, a)
    );
    assert!(
        o.beh.current == Some(Behaviour::Guard) || o.beh.queued == Some(Behaviour::Guard),
        "guard queued or current: {}",
        describe(&w, a)
    );
    let crash = run_until(&mut w, 3000, |w| w.obj(a).unwrap().health == 0);
    assert!(crash, "a crash: {}", describe(&w, a));
    crashed(&w, a);
    pad_removed_then_falls(&mut w, k, a, losses);
}

/// The bomb rules of claim 46's planted-bomb case: a bomb warhead whose
/// spread reaches every object near the crash, a delay longer than the
/// record, `quirk` for `quirks.crash_drops_planted_bomb`.
fn bomb_rules(quirk: bool) -> Ruleset {
    let mut r = pad_bound(Some(vec!["APAD".into()]));
    r.sim.general.economy.curley_shuffle = Some(true);
    r.sim.warheads.insert(
        "BigBoom".into(),
        glasnost_rules::sim::WarheadType {
            cell_spread: dec("12"),
            percent_at_max: dec("1"),
            verses: Some(vec![glasnost_rules::sim::Decimal("1".into()); 11]),
            ..Default::default()
        },
    );
    let g = &mut r.sim.general;
    g.ivan_warhead = Some("BigBoom".into());
    g.ivan_damage = Some(77);
    g.ivan_timed_delay = Some(100_000);
    r.globals.quirks.crash_drops_planted_bomb = quirk;
    r
}

/// A pad-bound aircraft with no pad that carries a planted bomb, and an own
/// bystander near its target; runs until claim 46's crash has started.
/// Returns (world, aircraft, bystander, the bystander's health).
fn bomb_crash(quirk: bool) -> (World, ObjectId, ObjectId, i32) {
    let mut w = game(&bomb_rules(quirk));
    ready(&mut w);
    let a = put(&mut w, Kind::Aircraft, "JET1", 0, 0, -4);
    let v = put(&mut w, Kind::Vehicle, "VICTIM", 1, 1, 8);
    let by = put(&mut w, Kind::Vehicle, "VICTIM", 0, -2, 6);
    let t = w.tick;
    w.obj_mut(a).unwrap().combat.bomb = Some(glasnost_sim::bomb::Bomb {
        planter: None,
        house: Some(ALPHA),
        planted: t,
    });
    click(&mut w, a, ClickOn::Object(v));
    assert!(run_until(&mut w, 3000, |w| w.obj(a).unwrap().health == 0));
    crashed(&w, a);
    let h = w.obj(by).unwrap().health;
    (w, a, by, h)
}

// spec: S-0501 claim 46 and Edge cases (quirks.crash_drops_planted_bomb on: a destruction without a hit does not set off a planted bomb; it stays attached through the fall and is removed at impact with no detonation and no damage); S-0312 claim 7 (revision 44: a carrier destroyed without a hit has no shared death effects, so its bomb does not detonate at the destruction)
#[test]
fn claim_46_crash_drops_a_planted_bomb() {
    let (mut w, a, by, h) = bomb_crash(true);
    assert!(
        w.obj(a).unwrap().combat.bomb.is_some(),
        "attached at the crash start"
    );
    while w.obj(a).is_some() {
        assert!(
            w.obj(a).unwrap().combat.bomb.is_some(),
            "attached in the fall"
        );
        step(&mut w);
    }
    assert_eq!(w.obj(by).unwrap().health, h, "no bomb damage at impact");
}

// spec: S-0501 Edge cases (quirks.crash_drops_planted_bomb off: the bomb goes off at impact by S-0519 claim 5, before the death weapon)
#[test]
fn claim_46_crash_sets_off_a_planted_bomb_with_the_quirk_off() {
    let (mut w, a, by, h) = bomb_crash(false);
    assert!(
        w.obj(a).unwrap().combat.bomb.is_some(),
        "attached at the crash start"
    );
    assert!(run_until(&mut w, 600, |w| w.obj(a).is_none()));
    assert!(w.obj(by).unwrap().health < h, "bomb damage at impact");
}

// spec: S-0501 claim 46 and S-0312 claim 7 (revision 44: the detonate order during the fall sets the bomb off by S-0519 claim 5 at the aircraft's position then) and S-0519 claims 5 and 7 (the falling carrier's health is already 0, so the one detonation cannot kill it again: no second death, loss or credit, and the bomb is gone, so impact sets off nothing)
#[test]
fn claim_46_detonate_order_during_the_fall() {
    let stats = glasnost_sim::econ::stats::of;
    let (mut w, a, by, h) = bomb_crash(true);
    step(&mut w);
    assert!(w.obj(a).is_some(), "still falling");
    let losses: Vec<i32> = (0..2).map(|i| stats(&w, HouseId(i)).losses).collect();
    let kills: Vec<i32> = (0..2).map(|i| stats(&w, HouseId(i)).kills).collect();
    glasnost_sim::bomb::detonate_order(&mut w, a);
    assert!(w.obj(a).unwrap().combat.bomb.is_none(), "gone off");
    let hb = w.obj(by).expect("the bystander survives").health;
    assert!(hb < h, "bomb damage in the order's tick");
    crashed(&w, a);
    let now: Vec<i32> = (0..2).map(|i| stats(&w, HouseId(i)).losses).collect();
    assert_eq!(now, losses, "no second loss");
    let now: Vec<i32> = (0..2).map(|i| stats(&w, HouseId(i)).kills).collect();
    assert_eq!(now, kills, "no credit");
    assert!(run_until(&mut w, 600, |w| w.obj(a).is_none()));
    assert_eq!(w.obj(by).unwrap().health, hb, "no second detonation");
    let now: Vec<i32> = (0..2).map(|i| stats(&w, HouseId(i)).losses).collect();
    assert_eq!(now, losses, "still one loss in all");
}

// spec: S-0501 claim 46 and S-0519 claims 4, 5 and 7 (a bomb whose timer comes due in the falling carrier's impact tick goes off in that turn, before the impact: one detonation, not the silent removal of quirks.crash_drops_planted_bomb)
#[test]
fn claim_46_bomb_due_in_the_impact_tick_goes_off_before_the_impact() {
    // A first run finds the tick number of the impact tick.
    let (mut w, a, _, _) = bomb_crash(true);
    let mut last = w.tick;
    while w.obj(a).is_some() {
        last = w.tick;
        step(&mut w);
    }
    // The same crash with the bomb due then.
    let (mut w, a, by, h) = bomb_crash(true);
    let delay = w.rules().combat.general.ivan.delay;
    // spec: S-0519 F1 (the bomb goes off in the first tick n with n > t + D)
    let planted = last.wrapping_sub(1).wrapping_sub(delay);
    w.obj_mut(a).unwrap().combat.bomb.as_mut().unwrap().planted = planted;
    while w.tick < last {
        step(&mut w);
        let o = w.obj(a).expect("still falling");
        assert!(o.combat.bomb.is_some(), "not due before the impact tick");
        assert_eq!(w.obj(by).unwrap().health, h, "no damage before");
    }
    step(&mut w);
    assert!(w.obj(a).is_none(), "the impact came in this tick");
    assert!(
        w.obj(by).unwrap().health < h,
        "the bomb went off in the impact tick"
    );
}

// spec: S-0501 claim 34 (an aircraft that is not a strike aircraft, in retreat, ignores a request to queue any other behaviour entirely, with no start, and its current behaviour goes on)
#[test]
fn claim_34_retreat_ignores_a_move_order() {
    let mut w = game(&rules());
    ready(&mut w);
    let s = put(&mut w, Kind::Aircraft, "JET", 0, 4, 0);
    force(&mut w, s, Behaviour::Retreat);
    step(&mut w);
    assert_eq!(w.obj(s).unwrap().beh.current, Some(Behaviour::Retreat));
    click(&mut w, s, ClickOn::Cell(testkit::mid(SIZE, -6, -6)));
    let o = w.obj(s).unwrap();
    assert_eq!(o.beh.queued, None, "the move is not queued");
    assert_eq!(o.beh.current, Some(Behaviour::Retreat));
    step(&mut w);
    assert_eq!(w.obj(s).unwrap().beh.current, Some(Behaviour::Retreat));
}

// spec: S-0501 claims 18 and 45 and T2 (a computer-controlled house's armed aircraft with no dock types, outside a team, keeps the edge destination in area guard; at exactly its flight height each area-guard processing is the fall-back to rest with a delay of 1 tick, and it queues no move). The house is made computer-controlled by S-0309's test-only mark.
#[test]
fn claim_45_computer_area_guard_at_flight_height() {
    let r = one_dock(|u| u.econ.dock = None);
    let mut w = game(&r);
    ready(&mut w);
    w.houses[0].human = false;
    let a = put(&mut w, Kind::Aircraft, "JET1", 0, 0, 0);
    lose_target_on_approach(&mut w, a, Loss::Remove, |_| {});
    let e = edge_destination(&mut w, a);
    let guard = run_until(&mut w, 100, |w| {
        w.obj(a).unwrap().beh.current == Some(Behaviour::AreaGuard)
    });
    assert!(guard, "area guard: {}", describe(&w, a));
    let mut seen = 0;
    for _ in 0..200 {
        let before = w.obj(a).unwrap().beh.last;
        let at_height = air::height(&w, a) == CRUISE;
        step(&mut w);
        let o = w.obj(a).unwrap();
        assert_eq!(
            dest_cell(&w, a),
            Some(e),
            "the edge cell kept: {}",
            describe(&w, a)
        );
        assert_eq!(
            o.beh.current,
            Some(Behaviour::AreaGuard),
            "{}",
            describe(&w, a)
        );
        assert_ne!(o.beh.queued, Some(Behaviour::Move));
        if at_height && o.beh.last != before {
            assert_eq!(
                o.beh.last.unwrap().1,
                1,
                "a delay of 1 tick at flight height"
            );
            seen += 1;
        }
    }
    assert!(seen > 10, "processed at flight height: {seen}");
}

// ---------------------------------------------------------------------------
// Area guard away from the cruise height (revision 41, Q-1837)

/// Lifts JET to its cruise height, forces *area guard*, and raises it
/// `above` leptons over its cruise height before every tick until its
/// next *area guard* processing has run; returns the delay that processing
/// chose. The height is set by hand, standing in for AC29 part j's cliff.
fn area_guard_processing_at(w: &mut World, a: ObjectId, above: i32) -> i32 {
    force(w, a, Behaviour::AreaGuard);
    let before = w.obj(a).unwrap().beh.last;
    for _ in 0..400 {
        let o = w.obj(a).unwrap();
        let surface = w.surface_at(o.pos.x, o.pos.y);
        w.obj_mut(a).unwrap().pos.z = surface + CRUISE + above;
        assert_eq!(air::height(w, a), CRUISE + above);
        step(w);
        let o = w.obj(a).unwrap();
        if o.beh.last != before {
            return o.beh.last.unwrap().1;
        }
    }
    panic!("no area guard processing: {}", describe(w, a));
}

// spec: S-0501 claim 45 (each area-guard processing compares the aircraft's height, taken at its start, with its cruise height; above it, with no target held and ammunition left, the processing has S-0215 claim 26's rows and the delay of claim 26 with k = 2, and area guard is kept; Q-1837, AC29 part j)
// spec: S-0215 claim 26 and F0 (D = k × D_r + U(1, 5), k = 2 for aircraft)
#[test]
fn claim_45_above_the_cruise_height_takes_claim_26_with_k_2() {
    let mut w = game(&rules());
    ready(&mut w);
    let a = lifted(&mut w, "JET", 0, 0);
    let dr = w.rules().behaviour(Behaviour::AreaGuard).rate_delay;
    assert!(dr > 0, "a rate delay, so that 1 tick is not claim 26's");
    for above in [1, 40, 300] {
        let d = area_guard_processing_at(&mut w, a, above);
        assert!(
            (2 * dr + 1..=2 * dr + 5).contains(&d),
            "claim 26's delay with k = 2 at {above} above: {d}, D_r {dr}"
        );
        let o = w.obj(a).unwrap();
        assert_eq!(
            o.beh.current,
            Some(Behaviour::AreaGuard),
            "{}",
            describe(&w, a)
        );
        assert_eq!(o.target, None);
        assert_eq!(o.beh.queued, None, "no attack queued, no fall-back");
    }
}

// spec: S-0501 claim 45 (above the cruise height, as below it, an aircraft holding a target has attack queued after the processing and its next processing comes 1 tick later; Q-1837, AC29 part j)
#[test]
fn claim_45_above_the_cruise_height_with_a_target_queues_attack() {
    let mut w = game(&rules());
    ready(&mut w);
    let a = lifted(&mut w, "JET", 0, 0);
    let v = put(&mut w, Kind::Vehicle, "VICTIM", 1, -10, 10);
    force(&mut w, a, Behaviour::AreaGuard);
    glasnost_sim::spawn::set_own_target(w.obj_mut(a).unwrap(), Some(Target::Object(v)));
    let d = area_guard_processing_at(&mut w, a, 200);
    assert_eq!(d, 1, "1 tick with a held target");
    // spec: S-0215 claim 11 (the late start of that update starts the queued attack)
    let o = w.obj(a).unwrap();
    assert_eq!(
        o.beh.current,
        Some(Behaviour::Attack),
        "{}",
        describe(&w, a)
    );
    assert_eq!(
        o.beh.start_tick,
        o.beh.last.unwrap().0,
        "started in that update"
    );
    assert_eq!(o.target, Some(Target::Object(v)));
}

// spec: S-0501 claim 45 (only equality with the cruise height gives the fall-back to rest with a delay of 1 tick; a human house's aircraft so falls back to guard, T2, Q-1837)
#[test]
fn claim_45_at_exactly_the_cruise_height_falls_back() {
    let mut w = game(&rules());
    ready(&mut w);
    let a = lifted(&mut w, "JET", 0, 0);
    force(&mut w, a, Behaviour::AreaGuard);
    let before = w.obj(a).unwrap().beh.last;
    let mut done = false;
    for _ in 0..400 {
        let o = w.obj(a).unwrap();
        let surface = w.surface_at(o.pos.x, o.pos.y);
        w.obj_mut(a).unwrap().pos.z = surface + CRUISE;
        step(&mut w);
        let o = w.obj(a).unwrap();
        if o.beh.last != before {
            assert_eq!(o.beh.last.unwrap().1, 1, "{}", describe(&w, a));
            assert_ne!(o.beh.current, Some(Behaviour::AreaGuard), "fell back");
            done = true;
            break;
        }
    }
    assert!(done, "processed: {}", describe(&w, a));
}

// ---------------------------------------------------------------------------
// The placement fall-back before the type test (revision 41, Q-1838)

/// Creates and places JET1 for house 0 at the map middle through placement
/// alone (no guard forced afterwards); returns it and the draws its
/// placement took.
fn placed_jet1(w: &mut World) -> (ObjectId, u64) {
    let ti = w.rules().index_of(Kind::Aircraft, "JET1").unwrap();
    let x = glasnost_sim::objects::create(w, Kind::Aircraft, ti, Some(ALPHA));
    let p = Pos::centre(testkit::mid(SIZE, 0, 0), 0);
    let z = w.surface_at(p.x, p.y);
    let draws = w.rng.taken();
    glasnost_sim::objects::place(w, x, Pos::new(p.x, p.y, z), 0);
    (x, w.rng.taken() - draws)
}

/// JET1: armed and not landable, so visiting by the type test; with the
/// one dock type APAD when `dock`, else with no dock types.
fn not_landable(dock: bool) -> Ruleset {
    one_dock(|u| {
        u.econ.landable = Some(false);
        if !dock {
            u.econ.dock = None;
        }
    })
}

// spec: S-0501 claim 1 and Terms (a type that only the type test marks visiting, with no mark before placement, is placed on the ground inside the usable area; its placement fall-back, made before the type test, takes T2's ground row for an aircraft that is not visiting: guard, not retreat, no edge draws; it makes no further fall-back while it rests; orders reach it; Q-1838, AC29 part g (4))
#[test]
fn claim_1_a_type_marked_by_the_type_test_rests_in_guard_and_takes_orders() {
    let mut w = game(&not_landable(true));
    ready(&mut w);
    let (a, draws) = placed_jet1(&mut w);
    assert_eq!(draws, 0, "no edge draws at placement");
    assert_eq!(air::height(&w, a), 0, "on the ground");
    assert!(air::on_ground(&w, a));
    assert!(st(&w, a).visiting, "the type test marks it");
    let o = w.obj(a).unwrap();
    assert_eq!(o.beh.current, Some(Behaviour::Guard), "{}", describe(&w, a));
    assert_eq!((o.target, o.dest), (None, None));
    let p = o.pos;
    for _ in 0..300 {
        step(&mut w);
        let o = w.obj(a).unwrap();
        assert_eq!(o.beh.current, Some(Behaviour::Guard), "{}", describe(&w, a));
        assert_eq!(o.pos, p, "rests: {}", describe(&w, a));
    }
    // spec: S-0501 claim 34 (this aircraft is not in retreat, so the order is not ignored: it takes the target and has attack queued in the tick the order is carried out, which then starts)
    let v = put(&mut w, Kind::Vehicle, "VICTIM", 1, 12, 0);
    click(&mut w, a, ClickOn::Object(v));
    step(&mut w);
    let o = w.obj(a).unwrap();
    assert_eq!(o.target, Some(Target::Object(v)), "{}", describe(&w, a));
    assert_eq!(o.beh.queued, Some(Behaviour::Attack), "{}", describe(&w, a));
    let attacks = run_until(&mut w, 100, |w| {
        w.obj(a).unwrap().beh.current == Some(Behaviour::Attack)
    });
    assert!(attacks, "attack starts: {}", describe(&w, a));
    assert!(
        run_until(&mut w, 100, |w| air::height(w, a) > 0),
        "takes off: {}",
        describe(&w, a)
    );
}

// spec: S-0501 claim 1 and Terms (the placement fall-back of an armed aircraft of a computer-controlled house outside a team gives area guard, not retreat; Q-1838). The house is made computer-controlled by S-0309's test-only mark.
#[test]
fn claim_1_a_computer_house_type_marked_by_the_type_test_rests_in_area_guard() {
    let mut w = game(&not_landable(false));
    ready(&mut w);
    w.houses[0].human = false;
    let (a, draws) = placed_jet1(&mut w);
    assert_eq!(draws, 0, "no edge draws at placement");
    assert_eq!(air::height(&w, a), 0);
    assert!(st(&w, a).visiting);
    assert_eq!(
        w.obj(a).unwrap().beh.current,
        Some(Behaviour::AreaGuard),
        "{}",
        describe(&w, a)
    );
}

// spec: S-0501 claim 1 and Terms (a later fall-back to rest on the ground takes T2's visiting rows: retreat, Q-1838)
#[test]
fn claim_1_a_later_fall_back_on_the_ground_takes_the_visiting_row() {
    let mut w = game(&not_landable(true));
    ready(&mut w);
    let (a, _) = placed_jet1(&mut w);
    testkit::run(&mut w, 5);
    assert_eq!(w.obj(a).unwrap().beh.current, Some(Behaviour::Guard));
    glasnost_sim::process::fall_back(&mut w, a, false);
    assert_eq!(
        w.obj(a).unwrap().beh.current,
        Some(Behaviour::Retreat),
        "{}",
        describe(&w, a)
    );
}

/// [`pad_bound_no_dock_types_spent_crashes_at_the_run_end`]'s run to the
/// tick of its fall-back crash; returns the raw values taken in that tick
/// and the stream after it. With `debris`, JET1 has minimum debris 1,
/// maximum debris 4, one sprite debris animation and an explosion list of
/// two entries, so that a killing hit would take a debris count draw and
/// a list pick.
fn fall_back_crash_draws(debris: bool) -> (u64, glasnost_sim::rng::Rng) {
    let mut r = pad_bound(None);
    r.sim.general.economy.curley_shuffle = Some(true);
    if debris {
        let c = &mut r.sim.aircraft.get_mut("JET1").unwrap().combat;
        c.min_debris = Some(1);
        c.max_debris = Some(4);
        c.debris_anims = Some(vec!["DA".into()]);
        c.explosion = Some(vec!["EA".into(), "EB".into()]);
    }
    let mut w = game(&r);
    ready(&mut w);
    let v = put(&mut w, Kind::Vehicle, "VICTIM", 1, 1, 8);
    let a = put(&mut w, Kind::Aircraft, "JET1", 0, 0, -4);
    click(&mut w, a, ClickOn::Object(v));
    for _ in 0..3200 {
        let before = w.rng.taken();
        step(&mut w);
        if w.obj(a).unwrap().health == 0 {
            crashed(&w, a);
            return (w.rng.taken() - before, w.rng.clone());
        }
    }
    panic!("no crash: {}", describe(&w, a));
}

// spec: S-0312 claim 16 (the fall-back crash of S-0501 claim 46 takes the three tumbling draws of a crash start and none of the killing-hit effects: no debris draws and no explosion-list pick; scenario D11 variant g) and S-0501 claim 46
#[test]
fn s0312_claim_16_the_fall_back_crash_takes_no_debris_or_list_draw() {
    let (plain, a) = fall_back_crash_draws(false);
    let (with, b) = fall_back_crash_draws(true);
    assert!(plain >= 3, "the three tumbling draws: {plain}");
    assert_eq!(with, plain, "no debris draw and no list pick");
    assert_eq!(a, b, "the same stream state");
}

// spec: S-0501 claim 45 (a visiting aircraft with its ammunition spent takes T2's visiting row, which queues retreat; a pad-bound visiting type is the exception: T2's after-the-row entry follows the visiting row, its pad search finds no pad since it has no dock types, and it crashes at this fall-back, claim 46, instead of retreating) and T2 (the crash row)
#[test]
fn pad_bound_visiting_type_crashes_instead_of_retreating() {
    let mut r = pad_bound(None);
    r.sim.general.economy.curley_shuffle = Some(true);
    r.sim.aircraft.get_mut("JET1").unwrap().econ.landable = Some(false);
    let mut w = game(&r);
    ready(&mut w);
    let v = put(&mut w, Kind::Vehicle, "VICTIM", 1, 1, 8);
    let a = put(&mut w, Kind::Aircraft, "JET1", 0, 0, -4);
    assert!(st(&w, a).visiting, "marked by the type test");
    click(&mut w, a, ClickOn::Object(v));
    let losses = glasnost_sim::econ::stats::of(&w, ALPHA).losses;
    assert!(run_until(&mut w, 3000, |w| w.obj(a).is_none_or(|o| o
        .combat
        .ammo
        == 0
        || o.health == 0)));
    let crash = run_until(&mut w, 200, |w| w.obj(a).is_none_or(|o| o.health == 0));
    assert!(crash, "a crash: {}", describe(&w, a));
    assert_ne!(
        w.obj(a).and_then(|o| o.beh.current),
        Some(Behaviour::Retreat),
        "not retreating"
    );
    crashed(&w, a);
    falls_to_impact(&mut w, a, losses);
}

// spec: S-0501 claim 45 (at any other height, above or below the cruise height, an armed aircraft with no target takes claim 26's rows and the delay of claim 26 with k = 2, and area guard is kept; this is the case below the cruise height, Q-1837)
// spec: S-0215 claim 26 and F0 (D = k * D_r + U(1, 5), k = 2 for aircraft)
#[test]
fn claim_45_below_the_cruise_height_takes_claim_26_with_k_2() {
    let mut w = game(&rules());
    ready(&mut w);
    let a = lifted(&mut w, "JET", 0, 0);
    let dr = w.rules().behaviour(Behaviour::AreaGuard).rate_delay;
    assert!(dr > 0, "a rate delay, so that 1 tick is not claim 26's");
    for below in [1, 40, 300] {
        let d = area_guard_processing_at(&mut w, a, -below);
        assert!(
            (2 * dr + 1..=2 * dr + 5).contains(&d),
            "claim 26's delay with k = 2 at {below} below: {d}, D_r {dr}"
        );
        let o = w.obj(a).unwrap();
        assert_eq!(
            o.beh.current,
            Some(Behaviour::AreaGuard),
            "{}",
            describe(&w, a)
        );
        assert_eq!(o.target, None);
        assert_eq!(o.beh.queued, None, "no attack queued, no fall-back");
    }
}

// spec: S-0501 claim 45 (below the cruise height an aircraft holding a target has attack queued after the processing and its next processing comes 1 tick later, as above it; Q-1837)
#[test]
fn claim_45_below_the_cruise_height_with_a_target_queues_attack() {
    let mut w = game(&rules());
    ready(&mut w);
    let a = lifted(&mut w, "JET", 0, 0);
    let v = put(&mut w, Kind::Vehicle, "VICTIM", 1, -10, 10);
    force(&mut w, a, Behaviour::AreaGuard);
    glasnost_sim::spawn::set_own_target(w.obj_mut(a).unwrap(), Some(Target::Object(v)));
    let d = area_guard_processing_at(&mut w, a, -200);
    assert_eq!(d, 1, "1 tick with a held target");
    let o = w.obj(a).unwrap();
    assert_eq!(
        o.beh.current,
        Some(Behaviour::Attack),
        "{}",
        describe(&w, a)
    );
    assert_eq!(o.target, Some(Target::Object(v)));
}
