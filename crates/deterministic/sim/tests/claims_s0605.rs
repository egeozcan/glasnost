//! Claim tests of S-0605 (absorbing buildings) on an empty synthetic map
//! with invented types: every name and number here is made up; no game
//! data. Houses Alpha (0) and Beta (1) of the testkit's sandbox game, not
//! allied; house 2 is the neutral house of `testkit::game_with_non_players`.

use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{BuildingType, Decimal, UnitType};
use glasnost_sim::combat::effects::{Effect, Sound};
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::structures::absorb;
use glasnost_sim::targeting::{self, Action, ClickOn, Modifiers};
use glasnost_sim::transport::Answer;
use glasnost_sim::world::{HouseId, ObjectId, Target, World};
use glasnost_sim::{save, testkit, tick};

const SIZE: i32 = 24;
const ALPHA: HouseId = HouseId(0);

fn dec(s: &str) -> Decimal {
    Decimal(s.into())
}

fn register(r: &mut Ruleset, reg: &str, name: &str) {
    let l = r.registries.entry(reg.into()).or_default();
    if !l.iter().any(|n| n == name) {
        l.push(name.into());
    }
}

fn absorber(r: &mut Ruleset, name: &str, f: impl FnOnce(&mut BuildingType)) {
    let mut t = BuildingType {
        strength: Some(1000),
        foundation: Some([2, 2]),
        infantry_absorb: Some(true),
        passengers: Some(3),
        size_limit: Some(dec("1")),
        ..Default::default()
    };
    t.econ.power = Some(10);
    t.econ.extra_power = Some(25);
    // A build-up sprite, so that the building can be sold (S-0408 claim 4).
    t.econ.buildup = Some(true);
    t.econ.buildup_frames = Some(4);
    f(&mut t);
    register(r, "buildings", name);
    r.sim.buildings.insert(name.into(), t);
}

fn infantry(r: &mut Ruleset, name: &str, f: impl FnOnce(&mut UnitType)) {
    let mut t = r.sim.infantry["WALKER"].clone();
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
/// - BIO: a 2 × 2 infantry absorber, capacity 3, size limit 1, power 10,
///   extra power 25, one link place;
/// - BIO2: BIO with two docks (two link places);
/// - BIO0: BIO with no extra power;
/// - BIOZ: BIO with capacity 0;
/// - VBIO: a 2 × 2 vehicle absorber, capacity 2, size limit 3;
/// - infantry HEAVY (size 2), HALF (size 1.5); vehicle CAR (unarmed);
/// - the rules-wide entry and exit sounds In and Out.
fn rules() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    absorber(&mut r, "BIO", |_| {});
    absorber(&mut r, "BIO2", |t| t.econ.number_of_docks = Some(2));
    absorber(&mut r, "BIO0", |t| t.econ.extra_power = None);
    absorber(&mut r, "BIOZ", |t| t.passengers = Some(0));
    absorber(&mut r, "VBIO", |t| {
        t.infantry_absorb = None;
        t.unit_absorb = Some(true);
        t.passengers = Some(2);
        t.size_limit = Some(dec("3"));
    });
    infantry(&mut r, "HEAVY", |t| t.size = Some(dec("2")));
    infantry(&mut r, "HALF", |t| t.size = Some(dec("1.5")));
    vehicle(&mut r, "CAR", |_| {});
    r.sim.general.enter_bio_reactor_sound = Some("In".into());
    r.sim.general.leave_bio_reactor_sound = Some("Out".into());
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

fn bio(w: &mut World, name: &str) -> ObjectId {
    put(w, Kind::Building, name, 0, 0, 0)
}

fn walker(w: &mut World, dx: i32, dy: i32) -> ObjectId {
    put(w, Kind::Infantry, "WALKER", 0, dx, dy)
}

/// Clicks `on` with `sel` selected as Alpha and issues the click's orders.
fn click(w: &mut World, sel: &[ObjectId], on: ClickOn) -> targeting::ClickResult {
    let r = targeting::click(w, ALPHA, sel, on, Modifiers::default());
    for o in r.orders.clone() {
        orders::issue(w, ALPHA, o);
    }
    r
}

fn cursor(w: &World, sel: &[ObjectId], on: ObjectId) -> Action {
    targeting::cursor(w, ALPHA, sel, ClickOn::Object(on), Modifiers::default())
}

/// The enter order of S-0214 table M on building `b`.
fn send(w: &mut World, u: ObjectId, b: ObjectId) {
    let house = w.obj(u).unwrap().owner.unwrap();
    orders::issue(
        w,
        house,
        Order::Behaviour {
            actor: u,
            behaviour: Behaviour::Enter,
            attack_move: false,
            target: None,
            dest: Some(Target::Object(b)),
        },
    );
}

fn until(w: &mut World, max: i32, done: impl Fn(&World) -> bool) -> bool {
    for _ in 0..max {
        if done(w) {
            return true;
        }
        tick::step(w);
    }
    done(w)
}

fn inside(w: &World, u: ObjectId, b: ObjectId) -> bool {
    absorb::absorbed_in(w, u) == Some(b)
}

fn sounds(w: &World, s: Sound) -> usize {
    w.effects
        .iter()
        .filter(|e| matches!(e, Effect::Sound { sound, .. } if *sound == s))
        .count()
}

fn round_trip(w: &World) {
    let bytes = save::save(w);
    let l = save::load(&bytes, w.rules_arc(), w.catalogue_arc()).expect("load");
    assert_eq!(save::state_hash(&l), save::state_hash(w));
}

// ---------------------------------------------------------------------------
// End to end.

// spec: S-0605 claims 1, 3, 6 and 7 (an own infantry unit clicked onto an absorbing building walks to it and goes in: off the map, the front passenger, n = 1)
// spec: S-0214 table I row x3 (docking answer accepts: enter)
#[test]
fn click_walk_in_end_to_end() {
    let mut w = game();
    let b = bio(&mut w, "BIO");
    let u = walker(&mut w, 5, 0);
    testkit::run(&mut w, 3);
    assert_eq!(cursor(&w, &[u], b), Action::Enter);
    let c = click(&mut w, &[u], ClickOn::Object(b));
    assert_eq!(c.orders.len(), 1);
    assert!(until(&mut w, 600, |w| inside(w, u, b)), "went in");
    assert!(!w.obj(u).unwrap().on_map);
    assert_eq!(absorb::passengers(&w, b), vec![u]);
    assert_eq!(absorb::count(&w, b), 1);
    round_trip(&w);
}
