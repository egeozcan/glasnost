//! Interaction-pair scenario tests of M7a (PLAN, cross-component
//! interactions): each pair runs through orders and the fixed tick pipeline
//! on synthetic data (invented types, weapons and warheads, every number our
//! own). Houses Alpha (0) and Beta (1) of the testkit's sandbox game, not
//! allied.

use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{Decimal, UnitType, WarheadType, WeaponType};
use glasnost_sim::combat::damage::{self, Hit};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::targeting::{self, Action, ClickOn, Modifiers};
use glasnost_sim::world::{HouseId, ObjectId, Target, World};
use glasnost_sim::{mind, orders, save, testkit, tick, transport};

const SIZE: i32 = 24;
const ALPHA: u8 = 0;
const BETA: u8 = 1;

fn dec(s: &str) -> Decimal {
    Decimal(s.into())
}

fn register(r: &mut Ruleset, reg: &str, name: &str) {
    let l = r.registries.entry(reg.into()).or_default();
    if !l.iter().any(|n| n == name) {
        l.push(name.into());
    }
}

/// The combat ruleset with the takeover warhead MIND, the weapon Ray3
/// (MIND, damage 3: room for three victims), infantry CLONE (Ray3) and
/// GRUNT (plain), and vehicle CART (an open-topped transport for three).
fn rules() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    r.sim.warheads.insert(
        "MIND".into(),
        WarheadType {
            verses: Some(vec![dec("1"); 11]),
            mind_control: Some(true),
            ..Default::default()
        },
    );
    r.sim.weapons.insert(
        "Ray3".into(),
        WeaponType {
            damage: Some(3),
            rof: Some(30),
            range: Some(dec("8")),
            speed: Some(100),
            projectile: Some("Bullet".into()),
            warhead: Some("MIND".into()),
            ..Default::default()
        },
    );
    for (name, primary) in [("CLONE", Some("Ray3")), ("GRUNT", None)] {
        let mut t = r.sim.infantry["WALKER"].clone();
        t.strength = Some(1000);
        t.primary = primary.map(Into::into);
        register(&mut r, "infantry", name);
        r.sim.infantry.insert(name.into(), t);
    }
    let mut cart: UnitType = r.sim.vehicles["BUGGY"].clone();
    cart.strength = Some(1000);
    cart.primary = None;
    cart.passengers = Some(3);
    cart.size_limit = Some(dec("2"));
    cart.open_topped = Some(true);
    register(&mut r, "vehicles", "CART");
    r.sim.vehicles.insert("CART".into(), cart);
    r
}

fn at(dx: i32, dy: i32) -> CellXY {
    testkit::mid(SIZE, dx, dy)
}

fn put(w: &mut World, kind: Kind, name: &str, house: u8, dx: i32, dy: i32) -> ObjectId {
    testkit::spawn(w, kind, name, house, at(dx, dy), 0)
}

fn owner(w: &World, id: ObjectId) -> Option<u8> {
    w.obj(id).and_then(|o| o.owner).map(|h| h.0)
}

fn click(w: &mut World, house: u8, sel: &[ObjectId], on: ObjectId) {
    let r = targeting::click(
        w,
        HouseId(house),
        sel,
        ClickOn::Object(on),
        Modifiers::default(),
    );
    for o in r.orders {
        orders::issue(w, HouseId(house), o);
    }
}

fn run_until(w: &mut World, max: i32, done: impl Fn(&World) -> bool) -> bool {
    for _ in 0..max {
        if done(w) {
            return true;
        }
        tick::step(w);
    }
    done(w)
}

// Pair: a controlled unit and a transport.
// spec: S-0507 claims 4, 6 and 14 (a controller with no victims boards an open-topped transport; its passengers stay in play)
// spec: S-0213 table T3 (f) (an open-topped transport gives its target to its passengers)
// spec: S-0601 claims 5 and 10 and Terms (a takeover by passenger fire; a transport answers neither to a controlled unit or to a controller holding victims; the controller leaving the game frees its victim)
// spec: S-0312 claim 15 (a destroyed transport's passengers leave the game with it)
// spec: S-0601 State and ordering (the records survive a save/load round trip mid-scenario)
#[test]
fn a_controller_riding_an_open_transport_takes_and_loses_a_victim() {
    let mut w = testkit::game_with(&rules(), SIZE, 1);
    let clone = put(&mut w, Kind::Infantry, "CLONE", BETA, 0, 0);
    let cart = put(&mut w, Kind::Vehicle, "CART", BETA, 2, 0);
    let grunt = put(&mut w, Kind::Infantry, "GRUNT", ALPHA, 9, 0);
    testkit::run(&mut w, 5);

    // The controller, holding no victims, boards by a click on the transport.
    let enter = targeting::cursor(
        &w,
        HouseId(BETA),
        &[clone],
        ClickOn::Object(cart),
        Modifiers::default(),
    );
    assert_eq!(enter, Action::Enter);
    click(&mut w, BETA, &[clone], cart);
    assert!(
        run_until(&mut w, 600, |w| transport::riding(w, clone) == Some(cart)),
        "the controller boards"
    );

    // The transport is ordered to attack; its passenger's takeover lands.
    orders::issue(
        &mut w,
        HouseId(BETA),
        orders::Order::Behaviour {
            actor: cart,
            behaviour: Behaviour::Attack,
            attack_move: false,
            target: Some(Target::Object(grunt)),
            dest: None,
        },
    );
    assert!(
        run_until(&mut w, 900, |w| mind::mind_controlled(w, grunt)),
        "taken by passenger fire"
    );
    assert_eq!(owner(&w, grunt), Some(BETA));
    assert_eq!(
        mind::controller(&w, clone).map(|r| r.victims.clone()),
        Some(vec![grunt])
    );

    // Save and load with the controller inside and its victim held.
    let h = save::state_hash(&w);
    let l = save::load(&save::save(&w), w.rules_arc(), testkit::catalogue()).unwrap();
    assert_eq!(save::state_hash(&l), h);

    // The controlled unit may not board its new owner's transport.
    let on_cart = targeting::cursor(
        &w,
        HouseId(BETA),
        &[grunt],
        ClickOn::Object(cart),
        Modifiers::default(),
    );
    assert_ne!(on_cart, Action::Enter, "a controlled unit is refused");
    assert_eq!(
        transport::answer(&w, cart, grunt),
        transport::Answer::Neither
    );

    // The transport is destroyed with its passenger: the victim goes home.
    damage::apply(
        &mut w,
        cart,
        Hit {
            amount: 100_000,
            warhead: None,
            distance: 0,
            attacker: None,
            house: None,
            ignore_defences: true,
            no_escape: false,
        },
    );
    testkit::run(&mut w, 2);
    assert!(w.obj(cart).is_none() && w.obj(clone).is_none());
    assert_eq!(owner(&w, grunt), Some(ALPHA));
    assert!(!mind::mind_controlled(&w, grunt));
}
