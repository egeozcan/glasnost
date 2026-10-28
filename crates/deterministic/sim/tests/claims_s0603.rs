//! Claim tests of S-0603 (recycling buildings) on an empty synthetic map
//! with invented types: every name and number here is made up; no game
//! data. Houses Alpha (0, country Alphaland) and Beta (1, country
//! Betaland) of the testkit's sandbox game; house 2 is the neutral house of
//! `testkit::game_with_non_players`.

use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{
    BuildingType, Decimal, MovementZone, ProjectileType, UnitType, WarheadType, WeaponType,
};
use glasnost_sim::combat::damage::{self, Hit};
use glasnost_sim::combat::effects::Effect;
use glasnost_sim::combat::fire;
use glasnost_sim::econ::TypeRef;
use glasnost_sim::econ::sell;
use glasnost_sim::mind::{self, MindSound};
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::structures::grinder;
use glasnost_sim::structures::ownership::{self, Cause};
use glasnost_sim::targeting::{self, Action, ClickOn, Modifiers};
use glasnost_sim::transport::{self, Answer};
use glasnost_sim::world::{HouseId, ObjectId, Target, World};
use glasnost_sim::{parasite, save, testkit, tick};

const SIZE: i32 = 28;
const ALPHA: HouseId = HouseId(0);
const BETA: HouseId = HouseId(1);

fn dec(s: &str) -> Decimal {
    Decimal(s.into())
}

fn ones() -> Option<Vec<Decimal>> {
    Some(vec![dec("1"); 11])
}

fn register(r: &mut Ruleset, reg: &str, name: &str) {
    let l = r.registries.entry(reg.into()).or_default();
    if !l.iter().any(|n| n == name) {
        l.push(name.into());
    }
}

fn building(r: &mut Ruleset, name: &str, f: impl FnOnce(&mut BuildingType)) {
    let mut t = BuildingType {
        strength: Some(1000),
        foundation: Some([2, 2]),
        grinding: Some(true),
        ..Default::default()
    };
    // A drain, so that it can be switched off (S-0405 claim 10).
    t.econ.power = Some(-10);
    // A build-up sprite, so that it can be sold (S-0408 claim 4).
    t.econ.buildup = Some(true);
    t.econ.buildup_frames = Some(4);
    f(&mut t);
    register(r, "buildings", name);
    r.sim.buildings.insert(name.into(), t);
}

fn infantry(r: &mut Ruleset, name: &str, f: impl FnOnce(&mut UnitType)) {
    let mut t = r.sim.infantry["WALKER"].clone();
    t.primary = None;
    f(&mut t);
    register(r, "infantry", name);
    r.sim.infantry.insert(name.into(), t);
}

fn vehicle(r: &mut Ruleset, name: &str, f: impl FnOnce(&mut UnitType)) {
    let mut t = r.sim.vehicles["BUGGY"].clone();
    t.primary = None;
    f(&mut t);
    register(r, "vehicles", name);
    r.sim.vehicles.insert(name.into(), t);
}

/// The combat ruleset with:
/// - GRND: a 2 × 2 recycling building with a drain of 10 and a build-up
///   (switchable and sellable); GRNDS: GRND with a long build-up (a slow
///   sale); GRND2: GRND with two docks; GABS: GRND
///   that also absorbs infantry (capacity 1, size limit 1); WALL: a plain
///   1 × 1 building;
/// - costs: WALKER 300, BUGGY 700, CART 900; infantry SOY with a refund
///   override of 77; ENG an engineer (cost 500);
/// - CART carries two (size limit 1), BIGT (cost 1100) carries one vehicle;
/// - FLY: a vehicle in the fly zone; BALLOON: a balloon-hovering vehicle;
///   BOAT: a naval vehicle; AMPH: a naval amphibious vehicle;
/// - CTRL: a controller vehicle (Ray, warhead MIND, cost 1000); PSY an
///   infantry controller;
/// - DRONE: a drone parasite (Latch, Hop, Grip; cost 200);
/// - refund percentage 0.5; Betaland's unit and infantry cost factors 0.6
///   and 0.4;
/// - the rules-wide enter-grinder sound Grind.
fn rules() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    building(&mut r, "GRND", |_| {});
    building(&mut r, "GRNDS", |t| t.econ.buildup_frames = Some(60));
    building(&mut r, "GRND2", |t| t.econ.number_of_docks = Some(2));
    building(&mut r, "GABS", |t| {
        t.infantry_absorb = Some(true);
        t.passengers = Some(1);
        t.size_limit = Some(dec("1"));
    });
    building(&mut r, "WALL", |t| {
        t.grinding = None;
        t.foundation = Some([1, 1]);
        t.econ.power = None;
    });
    r.sim.infantry.get_mut("WALKER").unwrap().cost = Some(300);
    r.sim.vehicles.get_mut("BUGGY").unwrap().cost = Some(700);
    infantry(&mut r, "SOY", |t| t.econ.soylent = Some(77));
    infantry(&mut r, "ENG", |t| {
        t.engineer = Some(true);
        t.cost = Some(500);
    });
    vehicle(&mut r, "CART", |t| {
        t.cost = Some(900);
        t.passengers = Some(2);
        t.size_limit = Some(dec("1"));
    });
    vehicle(&mut r, "BIGT", |t| {
        t.cost = Some(1100);
        t.passengers = Some(1);
        t.size_limit = Some(dec("10"));
    });
    vehicle(&mut r, "FLY", |t| t.movement_zone = Some(MovementZone::Fly));
    vehicle(&mut r, "BALLOON", |t| t.balloon_hover = Some(true));
    vehicle(&mut r, "BOAT", |t| t.combat.naval = Some(true));
    vehicle(&mut r, "AMPH", |t| {
        t.combat.naval = Some(true);
        t.movement_zone = Some(MovementZone::Amphibious);
    });
    r.sim.warheads.insert(
        "MIND".into(),
        WarheadType {
            verses: ones(),
            mind_control: Some(true),
            ..Default::default()
        },
    );
    r.sim.weapons.insert(
        "Ray".into(),
        WeaponType {
            damage: Some(1),
            rof: Some(30),
            range: Some(dec("2")),
            speed: Some(100),
            projectile: Some("Bullet".into()),
            warhead: Some("MIND".into()),
            ..Default::default()
        },
    );
    vehicle(&mut r, "CTRL", |t| {
        t.primary = Some("Ray".into());
        t.cost = Some(1000);
    });
    infantry(&mut r, "PSY", |t| t.primary = Some("Ray".into()));
    r.sim
        .projectiles
        .insert("Hop".into(), ProjectileType::default());
    r.sim.weapons.insert(
        "Latch".into(),
        WeaponType {
            damage: Some(1),
            rof: Some(15),
            range: Some(dec("6")),
            speed: Some(60),
            projectile: Some("Hop".into()),
            warhead: Some("Grip".into()),
            limbo_launch: Some(true),
            ..Default::default()
        },
    );
    r.sim.warheads.insert(
        "Grip".into(),
        WarheadType {
            verses: ones(),
            parasite: Some(true),
            ..Default::default()
        },
    );
    vehicle(&mut r, "DRONE", |t| {
        t.primary = Some("Latch".into());
        t.cost = Some(200);
    });
    r.sim.general.economy.refund_percent = Some(dec("0.5"));
    let beta = r.sim.countries.get_mut("Betaland").unwrap();
    beta.econ.cost_units_mult = Some(dec("0.6"));
    beta.econ.cost_infantry_mult = Some(dec("0.4"));
    r.sim.general.enter_grinder_sound = Some("Grind".into());
    r
}

fn game_on(r: &Ruleset) -> World {
    let mut w = testkit::game_with(r, SIZE, 7);
    testkit::run(&mut w, 2);
    w
}

fn game() -> World {
    game_on(&rules())
}

fn at(dx: i32, dy: i32) -> CellXY {
    testkit::mid(SIZE, dx, dy)
}

fn put(w: &mut World, kind: Kind, name: &str, house: u8, dx: i32, dy: i32) -> ObjectId {
    testkit::spawn(w, kind, name, house, at(dx, dy), 0)
}

/// A recycling building of type `name` and `house` at (dx, dy), settled for
/// a few ticks.
fn grnd(w: &mut World, name: &str, house: u8, dx: i32, dy: i32) -> ObjectId {
    let b = put(w, Kind::Building, name, house, dx, dy);
    testkit::run(w, 3);
    b
}

fn ally(w: &mut World, a: HouseId, b: HouseId) {
    let h = &mut w.houses[usize::from(a.0)];
    if !h.allies.contains(&b) {
        h.allies.push(b);
        h.allies.sort();
    }
}

fn cursor_as(w: &World, house: HouseId, sel: &[ObjectId], on: ObjectId) -> Action {
    targeting::cursor(w, house, sel, ClickOn::Object(on), Modifiers::default())
}

/// Clicks `on` with `sel` selected as `house` and issues the click's orders;
/// they apply in the next tick (S-0213 claim 1).
fn click_as(w: &mut World, house: HouseId, sel: &[ObjectId], on: ObjectId) -> Vec<Order> {
    let r = targeting::click(w, house, sel, ClickOn::Object(on), Modifiers::default());
    for o in r.orders.clone() {
        orders::issue(w, house, o);
    }
    r.orders
}

/// The recycle order of S-0214 table M (eaten, destination `b`, no
/// target), applied in the next tick.
fn send(w: &mut World, u: ObjectId, b: ObjectId) {
    let house = w.obj(u).unwrap().owner.unwrap();
    orders::issue(
        w,
        house,
        Order::Behaviour {
            actor: u,
            behaviour: Behaviour::Eaten,
            attack_move: false,
            target: None,
            dest: Some(Target::Object(b)),
        },
    );
    tick::step(w);
}

fn gone(w: &World, u: ObjectId) -> bool {
    w.obj(u).is_none_or(|o| !o.on_map)
}

/// Steps until `u` has left the game (at most `max` ticks), collecting the
/// effects. Returns the effects and the last cell it stood in.
fn until_gone(w: &mut World, u: ObjectId, max: i32) -> (Vec<Effect>, Option<CellXY>) {
    let mut fx = Vec::new();
    let mut last = None;
    for _ in 0..max {
        if gone(w, u) {
            return (fx, last);
        }
        last = w.obj(u).map(|o| o.pos.cell());
        w.effects.clear();
        tick::step(w);
        fx.extend(w.effects.iter().cloned());
    }
    assert!(gone(w, u), "never consumed");
    (fx, last)
}

fn credits(w: &World, h: HouseId) -> i32 {
    w.houses[usize::from(h.0)].credits
}

fn refund(w: &World, seller: HouseId, kind: Kind, name: &str) -> i32 {
    let index = w.rules().index_of(kind, name).expect("type");
    sell::refund(w, seller, TypeRef { kind, index })
}

fn consumed(fx: &[Effect]) -> Vec<&Effect> {
    fx.iter()
        .filter(|e| matches!(e, Effect::Consumed { .. }))
        .collect()
}

fn stats(w: &World, h: HouseId) -> (i32, i32) {
    let s = &w.houses[usize::from(h.0)].econ.result.stats;
    (s.kills, s.losses)
}

fn round_trip(w: &World) {
    let bytes = save::save(w);
    let l = save::load(&bytes, w.rules_arc(), w.catalogue_arc()).expect("load");
    assert_eq!(save::state_hash(&l), save::state_hash(w));
}

fn power_off(w: &mut World, house: HouseId, b: ObjectId) {
    orders::issue(w, house, Order::PowerOff { actor: b });
    tick::step(w);
    assert!(w.obj(b).unwrap().econ.building.switched_off);
}

fn bound(w: &World, u: ObjectId, b: ObjectId) -> bool {
    let o = w.obj(u).unwrap();
    // The order's behaviour may still be queued until the next processing.
    (o.beh.current == Some(Behaviour::Eaten) || o.beh.queued == Some(Behaviour::Eaten))
        && o.dest == Some(Target::Object(b))
}

// ---------------------------------------------------------------------------
// Claim 1: who may be sent in.

// spec: S-0603 claim 1 and S-0214 table I row I26 (infantry only into a recycling building of its own house), row I5 (an engineer into an engineer-friendly one at full health, an allied one included), table V rows n1 and n2 (a vehicle into an accepting allied one; fly zone not offered)
#[test]
fn claim_1_who_may_be_sent_in() {
    let mut w = game();
    ally(&mut w, ALPHA, BETA);
    ally(&mut w, BETA, ALPHA);
    let own = grnd(&mut w, "GRND", 0, 0, 0);
    let theirs = grnd(&mut w, "GRND", 1, 6, 0);
    let walker = put(&mut w, Kind::Infantry, "WALKER", 0, 3, 4);
    let eng = put(&mut w, Kind::Infantry, "ENG", 0, 4, 4);
    let car = put(&mut w, Kind::Vehicle, "BUGGY", 0, -4, 4);
    let fly = put(&mut w, Kind::Vehicle, "FLY", 0, -5, 4);
    testkit::run(&mut w, 2);
    assert_eq!(
        cursor_as(&w, ALPHA, &[walker], own),
        Action::Recycle,
        "I26 own"
    );
    assert_ne!(
        cursor_as(&w, ALPHA, &[walker], theirs),
        Action::Recycle,
        "I26 an ally's"
    );
    assert_eq!(
        cursor_as(&w, ALPHA, &[eng], theirs),
        Action::Recycle,
        "I5 an ally's"
    );
    assert_eq!(cursor_as(&w, ALPHA, &[car], own), Action::Recycle, "n2 own");
    assert_eq!(
        cursor_as(&w, ALPHA, &[car], theirs),
        Action::Recycle,
        "n2 an ally's"
    );
    assert_eq!(cursor_as(&w, ALPHA, &[fly], own), Action::NoEnter, "n1 fly");
    // An enemy's building offers no recycling.
    let foe = grnd(&mut w, "GRND", 1, -8, -8);
    w.houses[0].allies.retain(|&h| h != BETA);
    w.houses[1].allies.retain(|&h| h != ALPHA);
    assert_ne!(cursor_as(&w, ALPHA, &[car], foe), Action::Recycle);
    assert_ne!(cursor_as(&w, ALPHA, &[walker], foe), Action::Recycle);
    // spec: S-0214 table M (recycle gives eaten with the building as destination and no target)
    let o = click_as(&mut w, ALPHA, &[car], own);
    assert_eq!(
        o,
        vec![Order::Behaviour {
            actor: car,
            behaviour: Behaviour::Eaten,
            attack_move: false,
            target: None,
            dest: Some(Target::Object(own)),
        }]
    );
}

// ---------------------------------------------------------------------------
// Claim 2: the docking answer.

// spec: S-0603 claim 2 (refused: not allied one-way, being sold, no free link place, naval mismatch unless amphibious, balloon hover, switched off; low power, size, passengers and a controller holding victims do not refuse)
#[test]
fn claim_2_docking_answer_rows() {
    let mut w = game();
    let b = grnd(&mut w, "GRND", 0, 0, 0);
    let car = put(&mut w, Kind::Vehicle, "BUGGY", 0, 5, 0);
    let foe = put(&mut w, Kind::Vehicle, "BUGGY", 1, 5, 3);
    let bal = put(&mut w, Kind::Vehicle, "BALLOON", 0, -5, 0);
    let boat = put(&mut w, Kind::Vehicle, "BOAT", 0, -5, 3);
    let amph = put(&mut w, Kind::Vehicle, "AMPH", 0, -5, 6);
    let cart = put(&mut w, Kind::Vehicle, "CART", 0, 5, 6);
    let p = put(&mut w, Kind::Infantry, "WALKER", 0, 6, 6);
    transport::board(&mut w, p, cart);
    let ctrl = put(&mut w, Kind::Vehicle, "CTRL", 0, -3, -5);
    let held = put(&mut w, Kind::Infantry, "WALKER", 1, -3, -4);
    assert!(mind::take(&mut w, ctrl, held));
    assert!(
        glasnost_sim::econ::power::low_power(&w, ALPHA),
        "a drain alone: low power"
    );
    assert_eq!(
        grinder::answer(&w, b, car),
        Answer::Accepts,
        "low power does not refuse"
    );
    assert_eq!(
        grinder::answer(&w, b, cart),
        Answer::Accepts,
        "a loaded vehicle"
    );
    assert_eq!(
        grinder::answer(&w, b, ctrl),
        Answer::Accepts,
        "a controller with a victim"
    );
    assert_eq!(
        grinder::answer(&w, b, held),
        Answer::Accepts,
        "infantry: no kind test"
    );
    assert_eq!(grinder::answer(&w, b, foe), Answer::Refuses, "not allied");
    // One-way: the building's owner must count the entrant's house as allied.
    ally(&mut w, BETA, ALPHA);
    assert_eq!(
        grinder::answer(&w, b, foe),
        Answer::Refuses,
        "Beta's alliance alone"
    );
    ally(&mut w, ALPHA, BETA);
    assert_eq!(
        grinder::answer(&w, b, foe),
        Answer::Accepts,
        "Alpha counts Beta allied"
    );
    assert_eq!(
        grinder::answer(&w, b, bal),
        Answer::Refuses,
        "balloon hover"
    );
    assert_eq!(
        grinder::answer(&w, b, boat),
        Answer::Refuses,
        "naval mismatch"
    );
    assert_eq!(grinder::answer(&w, b, amph), Answer::Accepts, "amphibious");
    // spec: S-0603 claim 2 and F2 (link places max(d, 1); a place held by U counts as free)
    w.obj_mut(foe).unwrap().contact = Some(b);
    assert_eq!(
        grinder::answer(&w, b, car),
        Answer::Refuses,
        "every link place held"
    );
    assert_eq!(
        grinder::answer(&w, b, foe),
        Answer::Accepts,
        "its own place is free"
    );
    w.obj_mut(foe).unwrap().contact = None;
    let b2 = grnd(&mut w, "GRND2", 0, 8, -8);
    w.obj_mut(foe).unwrap().contact = Some(b2);
    assert_eq!(
        grinder::answer(&w, b2, car),
        Answer::Accepts,
        "two docks: one free"
    );
    w.obj_mut(foe).unwrap().contact = None;
    // Switched off refuses.
    power_off(&mut w, ALPHA, b);
    assert_eq!(grinder::answer(&w, b, car), Answer::Refuses, "switched off");
    // Being sold refuses.
    let s = grnd(&mut w, "GRND", 0, -8, 8);
    assert_eq!(grinder::answer(&w, s, car), Answer::Accepts);
    sell::sell(&mut w, ALPHA, s);
    tick::step(&mut w);
    assert_eq!(grinder::answer(&w, s, car), Answer::Refuses, "being sold");
    // Not a recycling building: no answer of this spec.
    let wall = put(&mut w, Kind::Building, "WALL", 0, 10, 10);
    assert_eq!(grinder::answer(&w, wall, car), Answer::Neither);
}

// spec: S-0603 claim 2 (a building with both features refuses the wrong kind and a controller holding victims by S-0605 claim 1, and otherwise accepts through its recycling feature even when the absorption capacity is full)
#[test]
fn claim_2_both_features() {
    let mut w = game();
    let b = grnd(&mut w, "GABS", 0, 0, 0);
    let car = put(&mut w, Kind::Vehicle, "BUGGY", 0, 5, 0);
    let u = put(&mut w, Kind::Infantry, "WALKER", 0, 5, 3);
    let filler = put(&mut w, Kind::Infantry, "WALKER", 0, 4, 3);
    assert_eq!(grinder::answer(&w, b, car), Answer::Refuses, "wrong kind");
    // Fill its absorption capacity of 1.
    orders::issue(
        &mut w,
        ALPHA,
        Order::Behaviour {
            actor: filler,
            behaviour: Behaviour::Enter,
            attack_move: false,
            target: None,
            dest: Some(Target::Object(b)),
        },
    );
    for _ in 0..600 {
        tick::step(&mut w);
        if glasnost_sim::structures::absorb::count(&w, b) == 1 {
            break;
        }
    }
    assert_eq!(glasnost_sim::structures::absorb::count(&w, b), 1);
    assert_eq!(
        grinder::answer(&w, b, u),
        Answer::Accepts,
        "capacity full, recycling applies"
    );
    // An infantry controller holding a victim is refused (S-0605 claim 1).
    let psy = put(&mut w, Kind::Infantry, "PSY", 0, -4, 3);
    let held = put(&mut w, Kind::Infantry, "WALKER", 1, -4, 4);
    assert!(mind::take(&mut w, psy, held));
    assert_eq!(
        grinder::answer(&w, b, psy),
        Answer::Refuses,
        "a controller holding a victim"
    );
}

// ---------------------------------------------------------------------------
// Claims 3 to 5: approaching and the moment of consumption.

// spec: S-0603 claims 1, 4, 5 and 8 (an own infantry unit clicked onto its recycling building walks in and is consumed in the tick it first arrives in a foundation cell; the owner is paid F1 with its own factors)
// spec: S-0214 table I row I26
#[test]
fn click_walk_in_end_to_end() {
    let mut w = game();
    let b = grnd(&mut w, "GRND", 0, 0, 0);
    let u = put(&mut w, Kind::Infantry, "WALKER", 0, 6, 1);
    testkit::run(&mut w, 2);
    click_as(&mut w, ALPHA, &[u], b);
    tick::step(&mut w);
    assert!(bound(&w, u, b));
    round_trip(&w);
    let c0 = credits(&w, ALPHA);
    let (_, last) = until_gone(&mut w, u, 600);
    assert!(w.obj(u).is_none(), "removed from the game");
    // Coming from the east, it was consumed at the arrival update of a step into an eastern foundation cell, not the building's own cell.
    let foundation = damage::foundation_order(at(0, 0), (2, 2));
    assert!(
        last.is_some_and(|c| foundation.contains(&c) && c != at(0, 0)),
        "{last:?}"
    );
    assert_eq!(
        credits(&w, ALPHA) - c0,
        refund(&w, ALPHA, Kind::Infantry, "WALKER")
    );
    // F1 by hand: trunc(trunc(300 × 1 × 1) × 0.5) = 150.
    assert_eq!(credits(&w, ALPHA) - c0, 150);
}

// spec: S-0603 claims 4 and 5 (a vehicle drives onto the foundation by S-0209 claim 5 rows VP6 and VP7 and is consumed at the cell-arrival update of the first foundation cell)
#[test]
fn claim_4_5_vehicle_consumed_on_the_foundation() {
    let mut w = game();
    let b = grnd(&mut w, "GRND", 0, 0, 0);
    let car = put(&mut w, Kind::Vehicle, "BUGGY", 0, 7, 0);
    let c0 = credits(&w, ALPHA);
    send(&mut w, car, b);
    let mut last = None;
    for _ in 0..600 {
        if gone(&w, car) {
            break;
        }
        last = Some(w.obj(car).unwrap().pos.cell());
        tick::step(&mut w);
    }
    assert!(gone(&w, car), "consumed");
    // Driving west along the row of the building's top-left cell, the first foundation cell is its east one; the building's own cell is never reached.
    assert_eq!(
        last,
        Some(at(1, 0)),
        "consumed in the first foundation cell"
    );
    // F1 by hand: trunc(trunc(700 × 1 × 1) × 0.5) = 350.
    assert_eq!(credits(&w, ALPHA) - c0, 350);
}

// spec: S-0603 claim 3 (no contact, no link place: two vehicles sent in at once with one link place do not wait; both consumed)
#[test]
fn claim_3_no_contact() {
    let mut w = game();
    let b = grnd(&mut w, "GRND", 0, 0, 0);
    let a = put(&mut w, Kind::Vehicle, "BUGGY", 0, 6, 0);
    let c = put(&mut w, Kind::Vehicle, "BUGGY", 0, -6, 0);
    let house = ALPHA;
    for u in [a, c] {
        orders::issue(
            &mut w,
            house,
            Order::Behaviour {
                actor: u,
                behaviour: Behaviour::Eaten,
                attack_move: false,
                target: None,
                dest: Some(Target::Object(b)),
            },
        );
    }
    for _ in 0..600 {
        tick::step(&mut w);
        for u in [a, c] {
            assert!(
                w.obj(u).is_none_or(|o| o.contact.is_none()),
                "never a contact"
            );
        }
        assert!(glasnost_sim::structures::absorb::contacts(&w, b).is_empty());
        if gone(&w, a) && gone(&w, c) {
            break;
        }
    }
    assert!(gone(&w, a) && gone(&w, c));
}

// spec: S-0603 claim 3 and S-0215 claims 14, 15 and 28 (no destination: infantry falls back to rest; a vehicle in eaten queues nothing and stays in eaten)
#[test]
fn claim_3_fall_back_without_destination() {
    let mut w = game();
    let u = put(&mut w, Kind::Infantry, "WALKER", 0, 3, 3);
    let v = put(&mut w, Kind::Vehicle, "BUGGY", 0, -3, 3);
    for x in [u, v] {
        let o = w.obj_mut(x).unwrap();
        o.beh.current = Some(Behaviour::Eaten);
        o.beh.queued = None;
        o.dest = None;
    }
    testkit::run(&mut w, 60);
    assert_ne!(
        w.obj(u).unwrap().beh.current,
        Some(Behaviour::Eaten),
        "infantry rests"
    );
    let o = w.obj(v).unwrap();
    assert_eq!(
        o.beh.current,
        Some(Behaviour::Eaten),
        "the vehicle stays in eaten"
    );
    assert_eq!(o.beh.queued, None, "nothing queued");
}

// ---------------------------------------------------------------------------
// Claims 6 to 11: consumption.

// spec: S-0603 claim 8 and F1 (health plays no part; a refund override is valued as S-0408 F1's v × a)
#[test]
fn claim_8_payment_ignores_health_and_uses_the_override() {
    let mut w = game();
    let b = grnd(&mut w, "GRND", 0, 0, 0);
    let hurt = put(&mut w, Kind::Infantry, "WALKER", 0, 5, 0);
    w.obj_mut(hurt).unwrap().health = 7;
    let c0 = credits(&w, ALPHA);
    send(&mut w, hurt, b);
    until_gone(&mut w, hurt, 600);
    assert_eq!(credits(&w, ALPHA) - c0, 150, "the full-health amount");
    let soy = put(&mut w, Kind::Infantry, "SOY", 0, 5, 0);
    let c1 = credits(&w, ALPHA);
    send(&mut w, soy, b);
    until_gone(&mut w, soy, 600);
    // v ≠ 0: trunc(77 × a) with a = 1, no refund percentage.
    assert_eq!(credits(&w, ALPHA) - c1, 77);
    assert_eq!(
        credits(&w, ALPHA) - c1,
        refund(&w, ALPHA, Kind::Infantry, "SOY")
    );
}

/// A tagged game: entry triggers ENT (var 0), DIE (var 1), DIE_CARRY (var 2).
fn tagged() -> World {
    let mut w = testkit::game_with_triggers(&rules(), SIZE, 7, testkit::entry_triggers());
    testkit::run(&mut w, 2);
    w
}

// spec: S-0603 claims 3, 8 and 10 and F1 (an ally's loaded vehicle: Alpha is paid the vehicle's and both passengers' refunds, each with Beta's factors and refund use; the passengers are removed with it)
// spec: S-0603 claims 6 and 11 (in the tick of consumption the building's tag gets an entry raise and the entrant's tag the destroyed-by-anything raise; passengers' tags do not raise; no kill, no loss)
#[test]
fn claim_6_8_10_11_allys_loaded_vehicle() {
    let mut w = tagged();
    ally(&mut w, ALPHA, BETA);
    ally(&mut w, BETA, ALPHA);
    let b = grnd(&mut w, "GRND", 0, 0, 0);
    glasnost_sim::triggers::object_tag(&mut w, b, Some(b"ENT"));
    let cart = put(&mut w, Kind::Vehicle, "CART", 1, 7, 0);
    glasnost_sim::triggers::object_tag(&mut w, cart, Some(b"DIE"));
    let p1 = put(&mut w, Kind::Infantry, "WALKER", 1, 8, 2);
    let p2 = put(&mut w, Kind::Infantry, "WALKER", 0, 8, 3);
    for p in [p1, p2] {
        glasnost_sim::triggers::object_tag(&mut w, p, Some(b"DIE_CARRY"));
        transport::board(&mut w, p, cart);
    }
    testkit::run(&mut w, 2);
    assert_eq!(cursor_as(&w, BETA, &[cart], b), Action::Recycle);
    click_as(&mut w, BETA, &[cart], b);
    tick::step(&mut w);
    assert!(bound(&w, cart, b));
    round_trip(&w);
    let expected = refund(&w, BETA, Kind::Vehicle, "CART")
        + refund(&w, BETA, Kind::Infantry, "WALKER")
        + refund(&w, ALPHA, Kind::Infantry, "WALKER");
    // By hand: CART trunc(trunc(900 × 0.6) × 0.5) = 270; Beta's walker trunc(trunc(300 × 0.4) × 0.5) = 60; Alpha's walker 150.
    assert_eq!(expected, 270 + 60 + 150);
    let (ca, cb) = (credits(&w, ALPHA), credits(&w, BETA));
    let s0 = [stats(&w, ALPHA), stats(&w, BETA)];
    let mut tick_of = None;
    let mut fx = Vec::new();
    for _ in 0..600 {
        w.effects.clear();
        tick::step(&mut w);
        fx.extend(w.effects.iter().cloned());
        if gone(&w, cart) {
            tick_of = Some(w.tick);
            break;
        }
        assert!(
            !w.triggers.variables[0] && !w.triggers.variables[1],
            "nothing before"
        );
    }
    assert!(tick_of.is_some(), "consumed");
    assert_eq!(credits(&w, ALPHA) - ca, expected);
    assert_eq!(credits(&w, BETA), cb, "nothing to the vehicle's owner");
    assert!(
        w.triggers.variables[0],
        "the entry raise in the consumption tick"
    );
    assert!(
        w.triggers.variables[1],
        "the entrant's destroyed-by-anything raise"
    );
    assert!(!w.triggers.variables[2], "no raise on the passengers' tags");
    for p in [cart, p1, p2] {
        assert!(w.obj(p).is_none(), "removed");
    }
    assert_eq!([stats(&w, ALPHA), stats(&w, BETA)], s0, "no kill, no loss");
    // spec: S-0603 claim 7 (one consumption effect, for the entrant only: passengers get no sounds; no death effects)
    let c = consumed(&fx);
    assert_eq!(c.len(), 1);
    assert!(matches!(
        c[0],
        Effect::Consumed {
            owner: Some(BETA),
            grinding: true,
            ..
        }
    ));
    assert!(
        !fx.iter().any(|e| matches!(e, Effect::Killed { .. })),
        "not a death"
    );
}

// spec: S-0603 claims 8 and 10 and Edge cases (passengers of passengers are paid for with their own owner's factors; only two levels)
#[test]
fn claim_10_passengers_of_passengers() {
    let mut w = game();
    let b = grnd(&mut w, "GRND", 0, 0, 0);
    let big = put(&mut w, Kind::Vehicle, "BIGT", 0, 7, 0);
    let cart = put(&mut w, Kind::Vehicle, "CART", 0, 7, 4);
    let p = put(&mut w, Kind::Infantry, "WALKER", 1, 8, 4);
    transport::board(&mut w, p, cart);
    transport::board(&mut w, cart, big);
    let expected = refund(&w, ALPHA, Kind::Vehicle, "BIGT")
        + refund(&w, ALPHA, Kind::Vehicle, "CART")
        + refund(&w, BETA, Kind::Infantry, "WALKER");
    // By hand: 550 + 450 + 60.
    assert_eq!(expected, 550 + 450 + 60);
    let c0 = credits(&w, ALPHA);
    send(&mut w, big, b);
    until_gone(&mut w, big, 600);
    assert_eq!(credits(&w, ALPHA) - c0, expected);
    assert!(
        w.obj(cart).is_none() && w.obj(p).is_none(),
        "removed with it"
    );
}

// spec: S-0603 claims 8 and 10 (a vehicle with a parasite attached: the parasite's refund with its own owner's factors, then S-0614 claim 12's release, which removes it silently; no second payment)
#[test]
fn claim_8_10_parasite() {
    let mut w = game();
    let b = grnd(&mut w, "GRND", 0, 0, 0);
    let host = put(&mut w, Kind::Vehicle, "BUGGY", 0, 6, 0);
    let drone = put(&mut w, Kind::Vehicle, "DRONE", 1, 9, 0);
    assert!(fire::shot(&mut w, drone, 0, Target::Object(host)));
    for _ in 0..200 {
        if parasite::host_of(&w, drone) == Some(host) {
            break;
        }
        tick::step(&mut w);
    }
    assert_eq!(parasite::parasite_of(&w, host), Some(drone), "latched");
    w.obj_mut(host).unwrap().health = 1000;
    let expected =
        refund(&w, ALPHA, Kind::Vehicle, "BUGGY") + refund(&w, BETA, Kind::Vehicle, "DRONE");
    // By hand: 350 + trunc(trunc(200 × 0.6) × 0.5) = 350 + 60.
    assert_eq!(expected, 410);
    let c0 = credits(&w, ALPHA);
    send(&mut w, host, b);
    let (fx, _) = until_gone(&mut w, host, 600);
    assert_eq!(credits(&w, ALPHA) - c0, expected);
    assert!(
        w.obj(drone).is_none_or(|o| !o.on_map),
        "the parasite does not survive"
    );
    assert_eq!(
        consumed(&fx).len(),
        1,
        "no consumption effect for the parasite"
    );
    testkit::run(&mut w, 60);
    assert!(
        w.obj(drone).is_none_or(|o| !o.on_map),
        "and does not come back"
    );
    assert_eq!(credits(&w, ALPHA) - c0, expected, "no second payment");
}

// spec: S-0603 claim 9 (a controlled entrant is not released first: valued with the controlling house's factors, it leaves its controller's list silently, the slot is freed, no release sound, no return)
// spec: S-0601 claim 12 (a victim leaving the game leaves the list silently)
#[test]
fn claim_9_controlled_entrant() {
    let mut w = game();
    let b = grnd(&mut w, "GRND", 1, 0, 0);
    let ctrl = put(&mut w, Kind::Vehicle, "CTRL", 1, -8, 0);
    let v = put(&mut w, Kind::Vehicle, "BUGGY", 0, -7, 0);
    assert!(mind::take(&mut w, ctrl, v));
    assert_eq!(w.obj(v).unwrap().owner, Some(BETA));
    assert!(mind::at_capacity(&w, ctrl));
    assert_eq!(cursor_as(&w, BETA, &[v], b), Action::Recycle);
    click_as(&mut w, BETA, &[v], b);
    let (cb, ca) = (credits(&w, BETA), credits(&w, ALPHA));
    let (fx, _) = until_gone(&mut w, v, 800);
    // Beta's unit factor 0.6: trunc(trunc(700 × 0.6) × 0.5) = 210.
    assert_eq!(credits(&w, BETA) - cb, 210);
    assert_eq!(credits(&w, ALPHA), ca);
    assert!(w.obj(v).is_none(), "gone for good, not returned");
    assert!(mind::controller(&w, ctrl).is_none_or(|r| r.victims.is_empty()));
    assert!(!mind::at_capacity(&w, ctrl), "the slot is free");
    assert!(
        !fx.iter().any(|e| matches!(
            e,
            Effect::Mind {
                sound: MindSound::Cleared,
                ..
            }
        )),
        "no release sound"
    );
    let next = put(&mut w, Kind::Vehicle, "BUGGY", 0, -9, 1);
    assert!(mind::can_take(&w, ctrl, next), "it can take again at once");
}

// spec: S-0603 claim 9 (a consumed controller releases its victims by S-0601 claim 10 as it leaves the game, after the payment)
#[test]
fn claim_9_controller_entrant_releases() {
    let mut w = game();
    let b = grnd(&mut w, "GRND", 0, 0, 0);
    let ctrl = put(&mut w, Kind::Vehicle, "CTRL", 0, 6, 0);
    let v = put(&mut w, Kind::Infantry, "WALKER", 1, 7, 1);
    assert!(mind::take(&mut w, ctrl, v));
    assert_eq!(w.obj(v).unwrap().owner, Some(ALPHA));
    let c0 = credits(&w, ALPHA);
    send(&mut w, ctrl, b);
    until_gone(&mut w, ctrl, 600);
    assert_eq!(
        credits(&w, ALPHA) - c0,
        refund(&w, ALPHA, Kind::Vehicle, "CTRL")
    );
    assert_eq!(credits(&w, ALPHA) - c0, 500);
    assert_eq!(
        w.obj(v).unwrap().owner,
        Some(BETA),
        "its victim is released"
    );
}

// spec: S-0603 claim 7 (the consumption effect: the entrant's type and owner at its position, with the enter-grinder sound at a recycling building)
#[test]
fn claim_7_consumption_effect() {
    let mut w = game();
    let b = grnd(&mut w, "GRND", 0, 0, 0);
    let u = put(&mut w, Kind::Infantry, "WALKER", 0, 5, 0);
    send(&mut w, u, b);
    let (fx, _) = until_gone(&mut w, u, 600);
    let c = consumed(&fx);
    assert_eq!(c.len(), 1);
    let walker = w.rules().index_of(Kind::Infantry, "WALKER").unwrap();
    assert!(matches!(
        c[0],
        Effect::Consumed { of, owner: Some(ALPHA), grinding: true, .. }
            if of.kind == Kind::Infantry && of.index == walker
    ));
}

// spec: S-0603 claim 13 (an eaten destination without the grinding flag still consumes and pays, without the enter-grinder sound; set up directly, unreachable through orders)
#[test]
fn claim_13_non_grinding_destination() {
    let mut w = game();
    let wall = put(&mut w, Kind::Building, "WALL", 0, 0, 0);
    let u = put(&mut w, Kind::Infantry, "WALKER", 0, 1, 0);
    testkit::run(&mut w, 2);
    let c0 = credits(&w, ALPHA);
    {
        let o = w.obj_mut(u).unwrap();
        o.beh.current = Some(Behaviour::Eaten);
        o.beh.queued = None;
    }
    glasnost_sim::econ::assign_destination(&mut w, u, Some(Target::Object(wall)));
    let (fx, _) = until_gone(&mut w, u, 300);
    assert_eq!(credits(&w, ALPHA) - c0, 150);
    let c = consumed(&fx);
    assert_eq!(c.len(), 1);
    assert!(matches!(
        c[0],
        Effect::Consumed {
            grinding: false,
            ..
        }
    ));
}

// ---------------------------------------------------------------------------
// Claim 15 and the quirk: interruptions.

fn quirked(on: bool) -> Ruleset {
    let mut r = rules();
    r.globals.quirks.grinder_no_arrival_recheck = on;
    r
}

// spec: S-0603 claims 5 and 15 and Edge cases (quirks.grinder_no_arrival_recheck on: a building switched off, being sold or changed owner after the order still consumes; off: refused at consumption, not consumed, destination cleared)
#[test]
fn quirk_no_arrival_recheck_both_positions() {
    for on in [true, false] {
        // Switched off while a vehicle is bound.
        let mut w = game_on(&quirked(on));
        let b = grnd(&mut w, "GRND", 0, 0, 0);
        let car = put(&mut w, Kind::Vehicle, "BUGGY", 0, 7, 0);
        send(&mut w, car, b);
        power_off(&mut w, ALPHA, b);
        assert!(bound(&w, car, b), "switching off does not touch entrants");
        let mut eaten = false;
        for _ in 0..600 {
            tick::step(&mut w);
            if gone(&w, car) {
                eaten = true;
                break;
            }
            if !on && w.obj(car).unwrap().dest.is_none() {
                break;
            }
        }
        assert_eq!(eaten, on, "switched off, quirk {on}");
        if !on {
            assert_eq!(w.obj(car).unwrap().dest, None, "destination cleared");
        }

        // Changed owner while an infantry unit is bound: the new owner is paid.
        let mut w = game_on(&quirked(on));
        let b = grnd(&mut w, "GRND", 0, 0, 0);
        let u = put(&mut w, Kind::Infantry, "WALKER", 0, 6, 0);
        send(&mut w, u, b);
        ownership::change(&mut w, b, Some(BETA), Cause::Trigger);
        assert!(bound(&w, u, b));
        let cb = credits(&w, BETA);
        let mut eaten = false;
        for _ in 0..600 {
            tick::step(&mut w);
            if gone(&w, u) {
                eaten = true;
                break;
            }
            if !on && w.obj(u).unwrap().dest.is_none() {
                break;
            }
        }
        assert_eq!(eaten, on, "owner change, quirk {on}");
        if on {
            assert_eq!(
                credits(&w, BETA) - cb,
                150,
                "the capturer is paid with Alpha's factors"
            );
        }
    }
}

// spec: S-0603 claim 15 (a sale does not send entrants away; one arriving before the building is removed is consumed and paid)
#[test]
fn claim_15_sale_while_bound() {
    let mut w = game();
    let b = grnd(&mut w, "GRNDS", 0, 0, 0);
    let u = put(&mut w, Kind::Infantry, "WALKER", 0, 2, 0);
    send(&mut w, u, b);
    sell::sell(&mut w, ALPHA, b);
    assert!(bound(&w, u, b), "not sent away");
    let c0 = credits(&w, ALPHA);
    for _ in 0..600 {
        if gone(&w, u) || w.obj(b).is_none() {
            break;
        }
        tick::step(&mut w);
    }
    assert!(w.obj(b).is_some(), "still being sold");
    assert!(gone(&w, u), "consumed during the sale");
    assert!(credits(&w, ALPHA) - c0 >= 150);
}

fn kill(w: &mut World, b: ObjectId) {
    let h = Hit {
        amount: w.obj(b).unwrap().health * 10,
        warhead: None,
        distance: 0,
        attacker: None,
        house: None,
        ignore_defences: true,
        no_escape: false,
    };
    damage::apply(w, b, h);
}

// spec: S-0603 claim 15 and S-0215 claims 14 and 15 (the building destroyed: the destination is cleared; infantry falls back to rest, a vehicle stays in eaten with nothing queued for at least 10 processings)
#[test]
fn claim_15_destroyed_building() {
    let mut w = game();
    let b = grnd(&mut w, "GRND", 0, 0, 0);
    let u = put(&mut w, Kind::Infantry, "WALKER", 0, 9, 0);
    let car = put(&mut w, Kind::Vehicle, "BUGGY", 0, -9, 0);
    send(&mut w, u, b);
    send(&mut w, car, b);
    kill(&mut w, b);
    testkit::run(&mut w, 2);
    for x in [u, car] {
        assert_eq!(w.obj(x).unwrap().dest, None, "destination cleared");
    }
    round_trip(&w);
    testkit::run(&mut w, 300);
    assert_ne!(
        w.obj(u).unwrap().beh.current,
        Some(Behaviour::Eaten),
        "infantry rests"
    );
    let o = w.obj(car).unwrap();
    assert_eq!(
        o.beh.current,
        Some(Behaviour::Eaten),
        "the vehicle stays in eaten"
    );
    assert_eq!(o.beh.queued, None);
    assert!(o.on_map);
}

// spec: S-0603 claim 15 and S-0213 claim 5 (a new order replaces eaten: not consumed)
#[test]
fn claim_15_new_order() {
    let mut w = game();
    let b = grnd(&mut w, "GRND", 0, 0, 0);
    let car = put(&mut w, Kind::Vehicle, "BUGGY", 0, 7, 0);
    send(&mut w, car, b);
    orders::issue(
        &mut w,
        ALPHA,
        Order::Behaviour {
            actor: car,
            behaviour: Behaviour::Move,
            attack_move: false,
            target: None,
            dest: Some(Target::Cell(at(10, 8))),
        },
    );
    testkit::run(&mut w, 400);
    assert!(!gone(&w, car), "not consumed");
    assert_ne!(w.obj(car).unwrap().beh.current, Some(Behaviour::Eaten));
}

// ---------------------------------------------------------------------------
// Claim 14: the grinder fate's search.

// spec: S-0603 claim 14 (every recycling building of the unit's owner whatever its state; the nearest by ρ₃; ties to the most recently placed; eaten queued without an immediate start)
#[test]
fn claim_14_search() {
    let mut w = testkit::game_with_non_players(&rules(), SIZE, 3);
    testkit::run(&mut w, 2);
    let n = put(&mut w, Kind::Infantry, "WALKER", 2, 0, 0);
    let east = grnd(&mut w, "GRND", 2, 6, 0);
    let west = grnd(&mut w, "GRND", 2, -6, 0);
    let near = grnd(&mut w, "GRND", 2, 0, 4);
    power_off(&mut w, HouseId(2), near);
    let _theirs = grnd(&mut w, "GRND", 1, 0, -3);
    assert!(grinder::grinder_fate(&mut w, n));
    let o = w.obj(n).unwrap();
    assert_eq!(
        o.dest,
        Some(Target::Object(near)),
        "the nearest, switched off or not"
    );
    assert_eq!(o.beh.queued, Some(Behaviour::Eaten), "queued");
    assert_ne!(o.beh.current, Some(Behaviour::Eaten), "not started at once");
    round_trip(&w);
    // Without the nearest one: east and west tie; the one placed last wins.
    glasnost_sim::objects::remove(&mut w, near);
    assert_eq!(
        w.obj(n).unwrap().dest,
        None,
        "removal clears a queued entrant's destination"
    );
    assert!(grinder::grinder_fate(&mut w, n));
    assert_eq!(
        w.obj(n).unwrap().dest,
        Some(Target::Object(west)),
        "placed last"
    );
    let _ = east;
    // No recycling building of its own: the assignment fails.
    let mut w = testkit::game_with_non_players(&rules(), SIZE, 3);
    let n = put(&mut w, Kind::Infantry, "WALKER", 2, 0, 0);
    grnd(&mut w, "GRND", 1, 4, 0);
    assert!(!grinder::grinder_fate(&mut w, n));
}

// spec: S-0601 claim 8 and S-0603 claim 14 (the grinder fate through a release: the neutral house's released unit walks into its nearest recycling building and is consumed; the neutral house receives P, without the refund percentage)
#[test]
fn grinder_fate_end_to_end() {
    let mut r = rules();
    let g = &mut r.sim.general;
    g.ai_capture_normal = Some(vec![0, 100, 0, 0]);
    g.ai_capture_wounded = Some(vec![0, 0, 0, 0]);
    g.ai_capture_low_power = Some(vec![0, 0, 0, 0]);
    g.ai_capture_low_money = Some(vec![0, 0, 0, 0]);
    let mut w = testkit::game_with_non_players(&r, SIZE, 1);
    testkit::run(&mut w, 2);
    let near = grnd(&mut w, "GRND", 2, 6, 0);
    let ctrl = put(&mut w, Kind::Vehicle, "CTRL", 1, -12, -12);
    let n = put(&mut w, Kind::Infantry, "WALKER", 2, 3, 0);
    testkit::run(&mut w, 3);
    assert!(mind::take(&mut w, ctrl, n));
    mind::release(&mut w, ctrl, n);
    assert_eq!(w.obj(n).unwrap().dest, Some(Target::Object(near)));
    let c0 = credits(&w, HouseId(2));
    until_gone(&mut w, n, 600);
    assert_eq!(
        credits(&w, HouseId(2)) - c0,
        300,
        "P for a non-player house"
    );
}

/// What became of a unit sent past a line of buildings.
#[derive(Debug)]
struct PastALine {
    /// It was consumed (or, for a move, reached its cell).
    arrived: bool,
    /// The tick its destination was first cleared, if it was.
    lost: Option<i32>,
    /// Its current and queued behaviours just after that, and 10 ticks later.
    after: Option<(Option<Behaviour>, Option<Behaviour>)>,
    later: Option<(Option<Behaviour>, Option<Behaviour>)>,
}

/// [`past`] with a north–south line of WALL buildings at x = -4 from
/// y = `shift` − `half` to `shift` + `half`.
fn past_a_line(
    half: i32,
    shift: i32,
    grinder: bool,
    infantry: bool,
    no_fallback: bool,
) -> PastALine {
    let walls: Vec<(i32, i32)> = (-half..=half).map(|y| (-4, y + shift)).collect();
    past(&walls, grinder, infantry, no_fallback)
}

/// [`past`] with a closed ring of WALL buildings round the grinder, 2 cells
/// from its foundation: no cell next to the grinder can be reached.
fn ringed(grinder: bool, infantry: bool) -> PastALine {
    let mut walls = Vec::new();
    for x in -3..=4 {
        for y in -3..=4 {
            if x == -3 || x == 4 || y == -3 || y == 4 {
                walls.push((x, y));
            }
        }
    }
    past(&walls, grinder, infantry, true)
}

/// Sends a BUGGY (or, with `infantry`, a WALKER) from (-20, 0) towards the
/// grinder at (0, 0) with the recycle order (`grinder`), or with a move to
/// the cell (-2, 0), with WALL buildings at `walls`, under
/// `quirks.corridor_retry_no_fallback` = `no_fallback`. On a larger map
/// than [`game`]'s (size 50).
fn past(walls: &[(i32, i32)], grinder: bool, infantry: bool, no_fallback: bool) -> PastALine {
    const BIG: i32 = 50;
    let at = |dx, dy| testkit::mid(BIG, dx, dy);
    let mut r = rules();
    r.globals.quirks.corridor_retry_no_fallback = no_fallback;
    let mut w = testkit::game_with(&r, BIG, 7);
    testkit::run(&mut w, 2);
    let b = testkit::spawn(&mut w, Kind::Building, "GRND", 0, at(0, 0), 0);
    for &(x, y) in walls {
        testkit::spawn(&mut w, Kind::Building, "WALL", 0, at(x, y), 0);
    }
    testkit::run(&mut w, 3);
    let car = if infantry {
        testkit::spawn(&mut w, Kind::Infantry, "WALKER", 0, at(-20, 0), 0)
    } else {
        testkit::spawn(&mut w, Kind::Vehicle, "BUGGY", 0, at(-20, 0), 0)
    };
    let (behaviour, dest) = if grinder {
        (Behaviour::Eaten, Target::Object(b))
    } else {
        (Behaviour::Move, Target::Cell(at(-2, 0)))
    };
    orders::issue(
        &mut w,
        ALPHA,
        Order::Behaviour {
            actor: car,
            behaviour,
            attack_move: false,
            target: None,
            dest: Some(dest),
        },
    );
    let mut out = PastALine {
        arrived: false,
        lost: None,
        after: None,
        later: None,
    };
    let beh = |w: &World| w.obj(car).map(|o| (o.beh.current, o.beh.queued));
    for t in 0..900 {
        tick::step(&mut w);
        if gone(&w, car) || (!grinder && w.obj(car).unwrap().pos.cell() == at(-2, 0)) {
            out.arrived = true;
            return out;
        }
        if out.lost.is_none() && w.obj(car).unwrap().dest.is_none() {
            out.lost = Some(t);
            out.after = beh(&w);
        }
        if out.lost.is_some_and(|l| t == l + 10) {
            out.later = beh(&w);
        }
    }
    out
}

// spec: S-0603 claims 3 and 4 and S-0212 Edge cases (short line of buildings, Q-1280 answered in r82) with claims 1 and 13 (the entrant's approach is an ordinary path computation to the building; the corridor exemption includes every building's exemption rectangle, so the first cell search goes round the line's end: lines of 3, 5, 7 and 9 are passed, for a vehicle and an infantry unit alike, and so is a move to a cell behind the line)
#[test]
fn claims_3_4_approach_past_a_line_of_buildings() {
    for half in 1..=4 {
        assert!(
            past_a_line(half, 0, true, false, true).arrived,
            "round a line of {}",
            2 * half + 1
        );
    }
    for infantry in [false, true] {
        let x = past_a_line(3, 0, true, infantry, true);
        assert!(x.arrived, "past a line of seven (infantry {infantry})");
        assert_eq!(x.lost, None, "the destination stays (infantry {infantry})");
        let m = past_a_line(3, 0, false, infantry, true);
        assert!(m.arrived, "a move past the line (infantry {infantry})");
    }
}

// spec: S-0212 claim 13 and Edge cases (short line of buildings: the first cell search goes round the line on the buildings' exemption rectangles, so no retry is made and quirks.corridor_retry_no_fallback, off here, changes nothing) and S-0603 claims 3 and 4
#[test]
fn lines_are_passed_with_corridor_retry_no_fallback_off() {
    for shift in 0..8 {
        assert!(
            past_a_line(3, shift, true, false, false).arrived,
            "a line of seven, shift {shift}"
        );
    }
    assert!(
        past_a_line(4, 0, true, false, false).arrived,
        "a line of nine"
    );
    assert!(
        past_a_line(3, 0, true, true, false).arrived,
        "infantry past a line of seven"
    );
    assert!(
        past_a_line(3, 0, false, false, false).arrived,
        "a move past a line of seven"
    );
}

// spec: S-0212 Edge cases (short line of buildings, Q-1280 answered in r82) and scenario P12 run S with quirks.corridor_retry_no_fallback on (the original): the corridor exemption includes every building's exemption rectangle (claim 13), so lines of 3, 5, 7 and 9 are passed at all eight shifts
#[test]
fn run_s_shifts_with_the_switch_on() {
    let mut fails = Vec::new();
    for half in 1..=4 {
        for shift in 0..8 {
            let x = past_a_line(half, shift, true, false, true);
            if !x.arrived {
                assert_eq!(
                    x.lost,
                    Some(1),
                    "an immediate failure: line {} shift {shift}",
                    2 * half + 1
                );
                fails.push((2 * half + 1, shift));
            }
        }
    }
    assert_eq!(fails, Vec::<(i32, i32)>::new());
}

// spec: S-0603 claim 3 and S-0212 claim 23 row 2 and Departure D5 (an entrant whose approach search failed more than 1 cell from the building gives up: its destination is cleared and it queues guard, an AI house's unit as a human-controlled one); revision 6, Q-1157 answered (eaten does not hold the queued guard back: a unit idle at the order is ready at the failing tick's late start, so guard is current at the end of that tick, nothing queued, and still 10 ticks later; scenario G9); S-0212 Edge cases (buildings do not separate regions: a destination enclosed by buildings passes the gate and fails in the cell searches)
#[test]
fn claim_3_failed_approach_gives_up() {
    for infantry in [false, true] {
        let x = ringed(true, infantry);
        assert!(!x.arrived, "infantry {infantry}");
        assert_eq!(x.lost, Some(1), "infantry {infantry}");
        assert_eq!(
            x.after,
            Some((Some(Behaviour::Guard), None)),
            "infantry {infantry}"
        );
        assert_eq!(
            x.later,
            Some((Some(Behaviour::Guard), None)),
            "infantry {infantry}"
        );
    }
}
