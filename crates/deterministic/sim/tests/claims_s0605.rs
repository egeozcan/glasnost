//! Claim tests of S-0605 (absorbing buildings) on an empty synthetic map
//! with invented types: every name and number here is made up; no game
//! data. Houses Alpha (0) and Beta (1) of the testkit's sandbox game, not
//! allied; house 2 is the neutral house of `testkit::game_with_non_players`.

use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{BuildingType, Decimal, UnitType, WarheadType, WeaponType};
use glasnost_sim::combat::damage::{self, Hit};
use glasnost_sim::combat::effects::{Effect, Sound};
use glasnost_sim::mind::{self, MindSound};
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
/// - BIO0: BIO with no extra power; BIOS: BIO with a drain of 10 (it can
///   be switched off, S-0405 claim 10);
/// - BIOZ: BIO with capacity 0;
/// - VBIO: a 2 × 2 vehicle absorber, capacity 2, size limit 3; VBIO1D:
///   VBIO with capacity 1 and two docks;
/// - infantry HEAVY (size 2), HALF (size 1.5); vehicle CAR (unarmed);
/// - BIO11: a 1 × 1 BIO; BIO1D: BIO with capacity 1 and two docks;
/// - WALL: a plain 1 × 1 building;
/// - warheads MIND (takeover) and BOOM (plain, versus 1) and weapon Ray (MIND, capacity 3); vehicle
///   CTRL (Ray) and infantry PSY (Ray);
/// - BALLOON: a balloon-hovering walker;
/// - the rules-wide entry and exit sounds In and Out.
fn rules() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    absorber(&mut r, "BIO", |_| {});
    absorber(&mut r, "BIO11", |t| t.foundation = Some([1, 1]));
    absorber(&mut r, "BIO1D", |t| {
        t.passengers = Some(1);
        t.econ.number_of_docks = Some(2);
    });
    register(&mut r, "buildings", "WALL");
    r.sim.buildings.insert(
        "WALL".into(),
        BuildingType {
            strength: Some(1000),
            foundation: Some([1, 1]),
            ..Default::default()
        },
    );
    r.sim.warheads.insert(
        "MIND".into(),
        WarheadType {
            verses: Some(vec![dec("1"); 11]),
            mind_control: Some(true),
            ..Default::default()
        },
    );
    r.sim.warheads.insert(
        "BOOM".into(),
        WarheadType {
            verses: Some(vec![dec("1"); 11]),
            ..Default::default()
        },
    );
    r.sim.weapons.insert(
        "Ray".into(),
        WeaponType {
            damage: Some(3),
            rof: Some(30),
            range: Some(dec("10")),
            speed: Some(100),
            projectile: Some("Bullet".into()),
            warhead: Some("MIND".into()),
            ..Default::default()
        },
    );
    vehicle(&mut r, "CTRL", |t| t.primary = Some("Ray".into()));
    infantry(&mut r, "PSY", |t| t.primary = Some("Ray".into()));
    infantry(&mut r, "BALLOON", |t| t.balloon_hover = Some(true));
    absorber(&mut r, "BIO2", |t| t.econ.number_of_docks = Some(2));
    absorber(&mut r, "BIO0", |t| t.econ.extra_power = None);
    absorber(&mut r, "BIOS", |t| t.econ.power = Some(-10));
    absorber(&mut r, "BIOZ", |t| t.passengers = Some(0));
    absorber(&mut r, "VBIO", |t| {
        t.infantry_absorb = None;
        t.unit_absorb = Some(true);
        t.passengers = Some(2);
        t.size_limit = Some(dec("3"));
    });
    absorber(&mut r, "VBIO1D", |t| {
        t.infantry_absorb = None;
        t.unit_absorb = Some(true);
        t.passengers = Some(1);
        t.size_limit = Some(dec("3"));
        t.econ.number_of_docks = Some(2);
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

/// An Alpha absorber of type `name` at the middle cell, settled for a few
/// ticks (its idle body range sets the ready mark, S-0215 claim 11).
fn bio(w: &mut World, name: &str) -> ObjectId {
    let b = put(w, Kind::Building, name, 0, 0, 0);
    testkit::run(w, 3);
    b
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

/// The enter order of S-0214 table M on building `b`, applied in the next
/// tick (S-0213 claim 1).
fn send(w: &mut World, u: ObjectId, b: ObjectId) {
    issue_enter(w, u, b);
    tick::step(w);
}

fn issue_enter(w: &mut World, u: ObjectId, b: ObjectId) {
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

fn quirked(f: impl FnOnce(&mut glasnost_rules::quirks::Quirks)) -> Ruleset {
    let mut r = rules();
    f(&mut r.globals.quirks);
    r
}

/// Sends a fresh Alpha walker from (3, 0) into `b` and runs until it is
/// inside.
fn fill_one(w: &mut World, b: ObjectId) -> ObjectId {
    let u = walker(w, 3, 0);
    send(w, u, b);
    assert!(until(w, 600, |w| inside(w, u, b)), "filled");
    u
}

fn power_off(w: &mut World, b: ObjectId) {
    orders::issue(w, ALPHA, Order::PowerOff { actor: b });
    tick::step(w);
}

fn cur(w: &World, u: ObjectId) -> Option<Behaviour> {
    w.obj(u).unwrap().beh.current
}

fn queued(w: &World, u: ObjectId) -> Option<Behaviour> {
    w.obj(u).unwrap().beh.queued
}

// ---------------------------------------------------------------------------
// Claims 1 and 2: who may enter.

// spec: S-0605 claim 1 (refused: not allied, switched off, wrong kind, balloon hover, a controller holding a victim; a controlled unit is not refused; otherwise F1)
#[test]
fn claim_1_acceptance_rows() {
    let mut w = game();
    let b = bio(&mut w, "BIO");
    let v = put(&mut w, Kind::Building, "VBIO", 0, 6, 6);
    let u = walker(&mut w, 4, 0);
    let foe = put(&mut w, Kind::Infantry, "WALKER", 1, 4, 3);
    let car = put(&mut w, Kind::Vehicle, "CAR", 0, -4, 0);
    let bal = put(&mut w, Kind::Infantry, "BALLOON", 0, 4, 4);
    assert_eq!(absorb::answer(&w, b, u), Answer::Accepts);
    assert_eq!(absorb::answer(&w, b, foe), Answer::Refuses, "not allied");
    assert_eq!(absorb::answer(&w, b, car), Answer::Refuses, "vehicle at an infantry absorber");
    assert_eq!(absorb::answer(&w, v, car), Answer::Accepts);
    assert_eq!(absorb::answer(&w, v, u), Answer::Refuses, "infantry at a vehicle absorber");
    assert_eq!(absorb::answer(&w, b, bal), Answer::Refuses, "balloon hover");
    // A controller holding a victim is refused; its victim is not.
    let psy = put(&mut w, Kind::Infantry, "PSY", 0, -3, 3);
    let held = put(&mut w, Kind::Infantry, "WALKER", 1, -3, 5);
    assert_eq!(absorb::answer(&w, b, psy), Answer::Accepts);
    assert!(mind::take(&mut w, psy, held));
    assert_eq!(absorb::answer(&w, b, psy), Answer::Refuses, "controller with a victim");
    assert_eq!(absorb::answer(&w, b, held), Answer::Accepts, "a controlled unit");
    // Switched off refuses.
    let s = put(&mut w, Kind::Building, "BIOS", 0, -6, 4);
    testkit::run(&mut w, 3);
    assert_eq!(absorb::answer(&w, s, u), Answer::Accepts);
    power_off(&mut w, s);
    assert!(w.obj(s).unwrap().econ.building.switched_off);
    assert_eq!(absorb::answer(&w, s, u), Answer::Refuses, "switched off");
    // Not an absorbing building: no answer of its own.
    let wall = put(&mut w, Kind::Building, "WALL", 0, -6, -6);
    assert_eq!(absorb::answer(&w, wall, u), Answer::Neither);
}

// spec: S-0605 claim 2 and F1 (passengers counted, not sizes; an entrant above the size limit is refused even by an empty building)
// spec: S-0605 Edge cases (capacity zero: F1 never holds)
#[test]
fn claim_2_counts_passengers_not_sizes() {
    let mut w = game();
    let b = bio(&mut w, "BIO");
    let z = put(&mut w, Kind::Building, "BIOZ", 0, -6, -6);
    let half = put(&mut w, Kind::Infantry, "HALF", 0, 5, 5);
    let u = walker(&mut w, 5, 3);
    assert_eq!(absorb::answer(&w, b, half), Answer::Refuses, "1.5 above 1");
    assert_eq!(absorb::answer(&w, z, u), Answer::Refuses, "capacity 0");
    for _ in 0..3 {
        fill_one(&mut w, b);
    }
    assert_eq!(absorb::count(&w, b), 3);
    assert_eq!(absorb::answer(&w, b, u), Answer::Refuses, "n + 1 above c");
}

// ---------------------------------------------------------------------------
// Claims 3 to 5: contact and waiting in line.

// spec: S-0605 claims 3 and 4 (a second entrant while the building holds a contact waits in line, destination cleared; it gets contact after the first is inside; one at a time)
#[test]
fn claim_3_4_waiting_in_line() {
    let mut w = game();
    let b = bio(&mut w, "BIO");
    let a = walker(&mut w, 4, 0);
    let c = walker(&mut w, 4, 2);
    send(&mut w, a, b);
    assert_eq!(w.obj(a).unwrap().contact, Some(b));
    send(&mut w, c, b);
    assert_eq!(absorb::waiting_for(&w, c), Some(b));
    assert_eq!(w.obj(c).unwrap().dest, None);
    assert_eq!(w.obj(c).unwrap().contact, None);
    round_trip(&w);
    let mut most = 0;
    for _ in 0..800 {
        tick::step(&mut w);
        most = most.max(absorb::contacts(&w, b).len());
        if inside(&w, c, b) {
            break;
        }
    }
    assert!(inside(&w, a, b) && inside(&w, c, b));
    assert_eq!(most, 1, "one link place: one at a time");
    assert_eq!(absorb::passengers(&w, b), vec![c, a], "front first");
}

// spec: S-0605 claim 3 (switched off: the actor asks to load, is refused and gives up the contact)
#[test]
fn claim_3_switched_off_refuses_loading() {
    let mut w = game();
    let b = bio(&mut w, "BIOS");
    let u = walker(&mut w, 4, 0);
    power_off(&mut w, b);
    send(&mut w, u, b);
    assert_eq!(w.obj(u).unwrap().contact, None);
    testkit::run(&mut w, 200);
    assert!(!inside(&w, u, b));
}

// spec: S-0605 claim 4 (granted after the wait but refused by the acceptance test: contact given up, behaviour none, destination cleared, waiting ends)
#[test]
fn claim_4_refused_after_waiting() {
    let mut w = game();
    let b = bio(&mut w, "BIO");
    fill_one(&mut w, b);
    fill_one(&mut w, b);
    let a = walker(&mut w, 4, 0);
    let c = walker(&mut w, 4, 2);
    send(&mut w, a, b);
    send(&mut w, c, b);
    assert_eq!(absorb::waiting_for(&w, c), Some(b));
    assert!(until(&mut w, 600, |w| absorb::waiting_for(w, c).is_none()));
    assert!(inside(&w, a, b));
    let o = w.obj(c).unwrap();
    assert_eq!(o.contact, None);
    assert_eq!(o.dest, None);
    assert_eq!((o.beh.current, o.beh.queued), (None, None));
    assert_eq!(absorb::count(&w, b), 3);
}

// spec: S-0605 claim 4 and S-0213 table T3 (b) (a new order ends the wait; a building that left the game ends it)
#[test]
fn claim_4_wait_ends() {
    let mut w = game();
    let b = bio(&mut w, "BIO");
    let a = walker(&mut w, 4, 0);
    let c = walker(&mut w, 4, 2);
    let d = walker(&mut w, 4, -2);
    send(&mut w, a, b);
    send(&mut w, c, b);
    send(&mut w, d, b);
    assert_eq!(absorb::waiting_for(&w, d), Some(b));
    orders::issue(
        &mut w,
        ALPHA,
        Order::Behaviour {
            actor: c,
            behaviour: Behaviour::Move,
            attack_move: false,
            target: None,
            dest: Some(Target::Cell(at(8, 8))),
        },
    );
    tick::step(&mut w);
    assert_eq!(absorb::waiting_for(&w, c), None, "new order");
    glasnost_sim::objects::remove(&mut w, b);
    tick::step(&mut w);
    assert_eq!(absorb::waiting_for(&w, d), None, "building gone");
}

// spec: S-0605 claim 5 (switched off while approaching: the entrant gives up the contact at its next processing and falls back to rest)
#[test]
fn claim_5_switched_off_while_approaching() {
    let mut w = game();
    let b = bio(&mut w, "BIOS");
    let u = walker(&mut w, 8, 0);
    send(&mut w, u, b);
    assert_eq!(w.obj(u).unwrap().contact, Some(b));
    testkit::run(&mut w, 2);
    power_off(&mut w, b);
    assert!(until(&mut w, 200, |w| w.obj(u).unwrap().contact.is_none()));
    testkit::run(&mut w, 300);
    assert!(!inside(&w, u, b));
    assert_ne!(cur(&w, u), Some(Behaviour::Enter));
}

// ---------------------------------------------------------------------------
// Claims 6 to 8: going inside.

/// Two walkers sent at once to BIO1D (capacity 1, two link places).
fn two_at_once(r: &Ruleset) -> (World, ObjectId, ObjectId, ObjectId) {
    let mut w = game_on(r);
    let b = bio(&mut w, "BIO1D");
    let a = walker(&mut w, 4, 0);
    let c = walker(&mut w, 4, 1);
    issue_enter(&mut w, a, b);
    issue_enter(&mut w, c, b);
    tick::step(&mut w);
    assert_eq!(absorb::waiting_for(&w, c), Some(b), "a contact held: waits");
    // The second link place is granted at its next retry (claim 4).
    tick::step(&mut w);
    assert_eq!(absorb::contacts(&w, b), vec![a, c], "two link places");
    (w, b, a, c)
}

// spec: S-0605 claim 6 and Edge cases (quirks.absorber_contact_no_capacity_recheck on: no re-check at completion, two link places overfill)
#[test]
fn claim_6_no_recheck_overfills() {
    let (mut w, b, a, c) = two_at_once(&rules());
    assert!(until(&mut w, 600, |w| inside(w, a, b) && inside(w, c, b)));
    assert_eq!(absorb::count(&w, b), 2, "above capacity 1");
}

// spec: S-0605 Edge cases (quirks.absorber_contact_no_capacity_recheck off: claim 1 again at completion; the refused entrant is treated as a refused vehicle: destination cleared, guard queued)
#[test]
fn claim_6_recheck_off_turns_away() {
    let r = quirked(|q| q.absorber_contact_no_capacity_recheck = false);
    let (mut w, b, a, c) = two_at_once(&r);
    assert!(until(&mut w, 600, |w| absorb::count(w, b) == 1));
    let (inn, out) = if inside(&w, a, b) { (a, c) } else { (c, a) };
    assert!(inside(&w, inn, b));
    assert!(until(&mut w, 600, |w| w.obj(out).unwrap().dest.is_none()
        && cur(w, out) != Some(Behaviour::Enter)));
    testkit::run(&mut w, 100);
    assert_eq!(absorb::count(&w, b), 1);
    assert!(w.obj(out).unwrap().on_map);
}

// spec: S-0605 claim 6 and S-0524 claim 12 (one entry raise on the building's tag at the entry, the entrant still outside)
#[test]
fn claim_6_entry_raise() {
    let mut w = testkit::game_with_triggers(&rules(), SIZE, 7, testkit::entry_triggers());
    testkit::run(&mut w, 2);
    let b = bio(&mut w, "BIO");
    glasnost_sim::triggers::object_tag(&mut w, b, Some(b"ENT"));
    let u = walker(&mut w, 4, 0);
    send(&mut w, u, b);
    assert!(!w.triggers.variables[0]);
    assert!(until(&mut w, 600, |w| inside(w, u, b)));
    testkit::run(&mut w, 2);
    assert!(w.triggers.variables[0], "the entry raised the tag");
}

// spec: S-0605 claim 7 (a controlled entrant: its controller releases it, the release sound before the entry sound; it goes in as its original owner's object; n up by 1, front passenger)
// spec: S-0601 claim 9 (the release)
#[test]
fn claim_7_controlled_entrant_released_first() {
    let mut w = game();
    let b = bio(&mut w, "BIO");
    let ctrl = put(&mut w, Kind::Vehicle, "CTRL", 0, -5, 0);
    let foe = put(&mut w, Kind::Infantry, "WALKER", 1, 4, 0);
    assert!(mind::take(&mut w, ctrl, foe));
    assert_eq!(w.obj(foe).unwrap().owner, Some(ALPHA));
    w.effects.clear();
    send(&mut w, foe, b);
    let mut fx = Vec::new();
    for _ in 0..600 {
        tick::step(&mut w);
        fx.extend(w.effects.iter().cloned());
        if inside(&w, foe, b) {
            break;
        }
    }
    assert!(inside(&w, foe, b));
    assert_eq!(w.obj(foe).unwrap().owner, Some(HouseId(1)), "the original owner");
    assert!(mind::controller(&w, ctrl).is_none_or(|r| r.victims.is_empty()));
    let cleared = fx
        .iter()
        .position(|e| matches!(e, Effect::Mind { sound: MindSound::Cleared, .. }))
        .expect("release sound");
    let entry = fx
        .iter()
        .position(|e| matches!(e, Effect::Sound { sound: Sound::EnterBioReactor, .. }))
        .expect("entry sound");
    assert!(cleared < entry);
}

// spec: S-0605 claim 7 (power due at the building's owner when extra output is above 0; not counted in the crate infantry count; spin reset), S-0405 claims 1 and 7, S-0520 Terms, S-0609 claim 11
#[test]
fn claim_7_power_crate_count_and_spin() {
    let mut w = game();
    let b = bio(&mut w, "BIO");
    testkit::run(&mut w, 2);
    let (out0, _) = glasnost_sim::econ::power::totals(&w, ALPHA);
    let n0 = glasnost_sim::crates::infantry_count(&w, ALPHA);
    let u = walker(&mut w, 4, 0);
    assert_eq!(glasnost_sim::crates::infantry_count(&w, ALPHA), n0 + 1);
    send(&mut w, u, b);
    {
        let o = w.obj_mut(u).unwrap();
        o.combat.spin = 7;
        o.combat.gattling_stage = 1;
    }
    w.effects.clear();
    assert!(until(&mut w, 600, |w| inside(w, u, b)));
    assert_eq!(sounds(&w, Sound::EnterBioReactor), 1, "the entry sound");
    w.effects.clear();
    assert_eq!(w.obj(u).unwrap().combat.spin, 0);
    assert_eq!(w.obj(u).unwrap().combat.gattling_stage, 0);
    assert_eq!(glasnost_sim::crates::infantry_count(&w, ALPHA), n0);
    testkit::run(&mut w, 3);
    let (out1, _) = glasnost_sim::econ::power::totals(&w, ALPHA);
    assert_eq!(out1, out0 + 25, "one passenger's extra power");
    assert_eq!(sounds(&w, Sound::EnterBioReactor), 0);
}

// spec: S-0605 claim 8 (a vehicle at a vehicle absorber: the full test on arrival; accept: in, silent; refuse when full: destination cleared, guard queued)
#[test]
fn claim_8_vehicle_absorber() {
    let mut w = game();
    let v = put(&mut w, Kind::Building, "VBIO", 0, 0, 0);
    testkit::run(&mut w, 3);
    let cars: Vec<ObjectId> = (0..3)
        .map(|i| put(&mut w, Kind::Vehicle, "CAR", 0, 5, -3 + 3 * i))
        .collect();
    w.effects.clear();
    let mut fx = 0;
    for &c in &cars[..2] {
        send(&mut w, c, v);
        for _ in 0..800 {
            tick::step(&mut w);
            fx += w.effects.iter().filter(|e| matches!(e, Effect::Sound { sound: Sound::EnterBioReactor, .. })).count();
            if inside(&w, c, v) {
                break;
            }
        }
        assert!(inside(&w, c, v));
    }
    assert_eq!(fx, 0, "no entry sound for a vehicle");
    assert_eq!(absorb::passengers(&w, v), vec![cars[1], cars[0]]);
    // The third: the docking answer refuses at the order already, so the
    // order is checked through the arrival directly.
    let third = cars[2];
    assert_eq!(absorb::answer(&w, v, third), Answer::Refuses);
    round_trip(&w);
}

// ---------------------------------------------------------------------------
// Claims 9 and 10: inside.

// spec: S-0605 claims 9 and 10 (a passenger keeps its owner; a building owner change leaves passengers as they are and the power follows the building)
#[test]
fn claim_10_owner_change_keeps_passengers() {
    let mut w = game();
    let b = bio(&mut w, "BIO");
    let u = fill_one(&mut w, b);
    assert!(!w.obj(u).unwrap().on_map);
    glasnost_sim::structures::ownership::change(
        &mut w,
        b,
        Some(HouseId(1)),
        glasnost_sim::structures::ownership::Cause::Trigger,
    );
    testkit::run(&mut w, 3);
    assert_eq!(w.obj(u).unwrap().owner, Some(ALPHA));
    assert!(inside(&w, u, b));
    assert_eq!(absorb::count(&w, b), 1);
    assert_eq!(glasnost_sim::econ::power::building_output(&w, b), 10 + 25);
}

// ---------------------------------------------------------------------------
// Claims 11 to 13: the voluntary release.

// spec: S-0605 claims 11 to 13 and F3 (deploy unloads one per processing, most recent first; exit at (x0 − 1, y0 + 1) facing 5 × 32 of 256, moving to (x0 − 2, y0 + 2); exit sound; power due)
// spec: S-0214 row B5 and table M (deploy on an absorbing building with passengers)
#[test]
fn claim_11_13_release_by_deploy() {
    let mut w = game();
    let b = bio(&mut w, "BIO");
    assert_ne!(cursor(&w, &[b], b), Action::Deploy, "empty");
    let first = fill_one(&mut w, b);
    let second = fill_one(&mut w, b);
    assert_eq!(cursor(&w, &[b], b), Action::Deploy);
    w.effects.clear();
    click(&mut w, &[b], ClickOn::Object(b));
    assert!(until(&mut w, 300, |w| !inside(w, second, b)));
    assert_eq!(sounds(&w, Sound::LeaveBioReactor), 1);
    assert!(inside(&w, first, b), "one per processing");
    let x0 = at(0, 0);
    let o = w.obj(second).unwrap();
    assert!(o.on_map);
    assert_eq!(o.pos.cell(), CellXY::new(x0.x - 1, x0.y + 1));
    assert_eq!(o.dest, Some(Target::Cell(CellXY::new(x0.x - 2, x0.y + 2))));
    assert_eq!(o.owner, Some(ALPHA));
    assert!(until(&mut w, 300, |w| !inside(w, first, b)));
    assert_eq!(sounds(&w, Sound::LeaveBioReactor), 2);
    assert_eq!(absorb::count(&w, b), 0);
}

// spec: S-0605 F3 (exit facing 32 × e: direction 5 gives 160 of 256)
#[test]
fn f3_exit_facing() {
    let mut w = game();
    let b = bio(&mut w, "BIO");
    let u = fill_one(&mut w, b);
    orders::issue(&mut w, ALPHA, Order::Deploy { actor: b });
    let mut facing = None;
    for _ in 0..300 {
        tick::step(&mut w);
        if w.obj(u).unwrap().on_map {
            facing = Some(w.obj(u).unwrap().body.value(w.tick));
            break;
        }
    }
    assert_eq!(facing, Some(5 * 32 * 256));
}

// spec: S-0605 claim 11 (an empty processing of an ordinary absorber queues guard and chooses 1 tick, without a draw)
#[test]
fn claim_11_empty_processing() {
    let mut w = game();
    let b = bio(&mut w, "BIO");
    let rng = w.rng.clone();
    assert_eq!(absorb::process_unload(&mut w, b), 1);
    assert_eq!(w.rng, rng, "no draw");
    assert!(cur(&w, b) == Some(Behaviour::Guard) || queued(&w, b) == Some(Behaviour::Guard));
}

/// A 1 × 1 absorber BIO11 at the middle cell holding one walker, with WALL
/// blockers in `blocked` (offsets from its cell), facing `facing`.
fn boxed(r: &Ruleset, facing: u16, blocked: &[(i32, i32)]) -> (World, ObjectId, ObjectId) {
    let mut w = game_on(r);
    let b = testkit::spawn(&mut w, Kind::Building, "BIO11", 0, at(0, 0), facing);
    testkit::run(&mut w, 3);
    let u = fill_one(&mut w, b);
    for &(dx, dy) in blocked {
        put(&mut w, Kind::Building, "WALL", 0, dx, dy);
    }
    (w, b, u)
}

/// The far cells of all eight directions.
fn far_ring() -> Vec<(i32, i32)> {
    glasnost_sim::regions::STEPS
        .iter()
        .map(|&(x, y)| (2 * x, 2 * y))
        .collect()
}

fn try_release(w: &mut World, b: ObjectId) {
    absorb::process_unload(w, b);
}

// spec: S-0605 claim 12 and Edge cases (quirks.absorber_release_second_round_skips_rear on: the second round skips the rear neighbour; off: the rear first)
#[test]
fn claim_12_second_round_rear_both_positions() {
    // Facing 0: rear direction 4, the neighbour (0, 1).
    let (mut w, b, u) = boxed(&rules(), 0, &far_ring());
    try_release(&mut w, b);
    let x0 = at(0, 0);
    assert_eq!(
        w.obj(u).unwrap().pos.cell(),
        CellXY::new(x0.x - 1, x0.y + 1),
        "skips the rear: direction 5"
    );
    assert_eq!(w.obj(u).unwrap().dest, Some(Target::Cell(CellXY::new(x0.x - 1, x0.y + 1))));
    let r = quirked(|q| q.absorber_release_second_round_skips_rear = false);
    let (mut w, b, u) = boxed(&r, 0, &far_ring());
    try_release(&mut w, b);
    assert_eq!(w.obj(u).unwrap().pos.cell(), CellXY::new(x0.x, x0.y + 1), "rear first");
}

// spec: S-0605 claim 12 and Edge cases (quirks.absorber_release_bridge_ends_search on: a deck neighbour at the eighth first-round direction that passes both cell tests ends the search; off: the second round starts); Q-1262 interim
#[test]
fn claim_12_deck_at_the_eighth_direction_both_positions() {
    // Facing 0: the first round runs 4, 5, 6, 7, 0, 1, 2, 3. Directions 4
    // to 2 have their far cells blocked; direction 3's neighbour (1, 1)
    // carries a deck, its far cell (2, 2) open.
    let blocked: Vec<(i32, i32)> = far_ring()
        .into_iter()
        .filter(|&c| c != (2, 2))
        .collect();
    let deck = |w: &mut World| {
        let c = at(1, 3);
        // Marks (1, 4), (1, 3), (1, 2) and (1, 1).
        w.cells.mark_high_bridge(c, 24);
        assert!(w.cells.is_bridge(at(1, 1)));
    };
    let (mut w, b, u) = boxed(&rules(), 0, &blocked);
    deck(&mut w);
    try_release(&mut w, b);
    assert!(inside(&w, u, b), "the search ended");
    let r = quirked(|q| q.absorber_release_bridge_ends_search = false);
    let (mut w, b, u) = boxed(&r, 0, &blocked);
    deck(&mut w);
    try_release(&mut w, b);
    assert!(!inside(&w, u, b), "the second round found an exit");
    let x0 = at(0, 0);
    assert_eq!(w.obj(u).unwrap().pos.cell(), CellXY::new(x0.x - 1, x0.y + 1));
}

// spec: S-0605 claim 13 (no exit: the passenger goes back to the front and blocks the others)
#[test]
fn claim_13_no_exit_blocks() {
    let mut ring: Vec<(i32, i32)> = glasnost_sim::regions::STEPS.to_vec();
    ring.extend(far_ring());
    let mut w = game();
    let b = testkit::spawn(&mut w, Kind::Building, "BIO11", 0, at(0, 0), 0);
    testkit::run(&mut w, 3);
    let a = fill_one(&mut w, b);
    let c = fill_one(&mut w, b);
    for &(dx, dy) in &ring {
        put(&mut w, Kind::Building, "WALL", 0, dx, dy);
    }
    try_release(&mut w, b);
    try_release(&mut w, b);
    assert_eq!(absorb::passengers(&w, b), vec![c, a]);
    assert!(!w.obj(c).unwrap().on_map);
}

// ---------------------------------------------------------------------------
// Claims 14 and 15: other ways out.

fn lethal(w: &World, id: ObjectId, ignore_defences: bool) -> Hit {
    let boom = rules().sim.warheads.keys().position(|k| k == "BOOM").unwrap() as u32;
    Hit {
        amount: w.obj(id).unwrap().health * 10,
        warhead: (!ignore_defences).then_some(boom),
        distance: 0,
        attacker: None,
        house: None,
        ignore_defences,
        no_escape: false,
    }
}

// spec: S-0605 claim 14 and S-0312 claim 24 (destruction: front first, one per foundation cell in S-0302's order; owner kept; crate count again)
#[test]
fn claim_14_destruction_releases_front_first() {
    let mut w = game();
    let b = bio(&mut w, "BIO");
    let a = fill_one(&mut w, b);
    let c = fill_one(&mut w, b);
    let n = glasnost_sim::crates::infantry_count(&w, ALPHA);
    let h = lethal(&w, b, false);
    damage::apply(&mut w, b, h);
    testkit::run(&mut w, 2);
    for p in [a, c] {
        let o = w.obj(p).expect("alive");
        assert!(o.on_map);
        assert_eq!(o.owner, Some(ALPHA));
        assert_eq!(absorb::absorbed_in(&w, p), None);
    }
    let order = damage::foundation_order(at(0, 0), (2, 2));
    assert_eq!(w.obj(c).unwrap().pos.cell(), order[0], "the most recent takes the first cell");
    assert_eq!(w.obj(a).unwrap().pos.cell(), order[1]);
    assert_eq!(glasnost_sim::crates::infantry_count(&w, ALPHA), n + 2);
}

// spec: S-0312 claim 24 (no crew, killing damage that ignores defences: every passenger dies)
#[test]
fn claim_14_destruction_without_crew_kills() {
    let mut w = game();
    let b = bio(&mut w, "BIO");
    let a = fill_one(&mut w, b);
    let h = lethal(&w, b, true);
    damage::apply(&mut w, b, h);
    testkit::run(&mut w, 2);
    assert!(w.obj(a).is_none_or(|o| !o.on_map && o.health <= 0));
}

// spec: S-0605 claim 14 and S-0408 claim 8 (a sale lets the passengers out, front first, owner kept)
#[test]
fn claim_14_sale_releases() {
    let mut w = game();
    let b = bio(&mut w, "BIO");
    let a = fill_one(&mut w, b);
    let c = fill_one(&mut w, b);
    glasnost_sim::econ::sell::sell(&mut w, ALPHA, b);
    assert!(until(&mut w, 600, |w| w.obj(a).unwrap().on_map && w.obj(c).unwrap().on_map));
    assert_eq!(w.obj(a).unwrap().owner, Some(ALPHA));
    assert_eq!(absorb::count(&w, b), 0);
}

// spec: S-0605 claim 14 and S-0514 claim 12 (erasure deletes the passengers)
#[test]
fn claim_14_erasure_deletes() {
    let mut w = game();
    let b = bio(&mut w, "BIO");
    let a = fill_one(&mut w, b);
    absorb::erased(&mut w, b);
    assert!(w.obj(a).is_none());
    assert_eq!(absorb::count(&w, b), 0);
}

// spec: S-0605 claim 15 (a passenger leaving the game inside lowers n); Q-1263 interim (power due)
#[test]
fn claim_15_removal_inside() {
    let mut w = game();
    let b = bio(&mut w, "BIO");
    let a = fill_one(&mut w, b);
    let c = fill_one(&mut w, b);
    testkit::run(&mut w, 3);
    let (out0, _) = glasnost_sim::econ::power::totals(&w, ALPHA);
    glasnost_sim::objects::remove(&mut w, a);
    assert_eq!(absorb::passengers(&w, b), vec![c]);
    testkit::run(&mut w, 3);
    let (out1, _) = glasnost_sim::econ::power::totals(&w, ALPHA);
    assert_eq!(out1, out0 - 25);
    round_trip(&w);
}

// ---------------------------------------------------------------------------
// The bio-reactor fate (S-0601 claim 8).

// spec: S-0601 claim 8 and S-0215 claim 24 (the bio-reactor fate: the released unit of a computer-controlled house enters the nearest absorbing building of its own owner that accepts it); Q-1260 interim
#[test]
fn bio_reactor_fate_sends_the_released_unit_in() {
    let mut r = rules();
    let g = &mut r.sim.general;
    g.ai_capture_normal = Some(vec![0, 0, 100, 0]);
    g.ai_capture_wounded = Some(vec![0, 0, 0, 0]);
    g.ai_capture_low_power = Some(vec![0, 0, 0, 0]);
    g.ai_capture_low_money = Some(vec![0, 0, 0, 0]);
    let mut w = testkit::game_with_non_players(&r, SIZE, 1);
    testkit::run(&mut w, 2);
    let near = put(&mut w, Kind::Building, "BIO", 2, 6, 0);
    let _far = put(&mut w, Kind::Building, "BIO", 2, -9, -9);
    // Far beyond its weapon's range, so that it does not take the walker again.
    let ctrl = put(&mut w, Kind::Vehicle, "CTRL", 1, -12, -12);
    let n = put(&mut w, Kind::Infantry, "WALKER", 2, 3, 0);
    testkit::run(&mut w, 3);
    assert!(mind::take(&mut w, ctrl, n));
    mind::release(&mut w, ctrl, n);
    assert_eq!(w.obj(n).unwrap().owner, Some(HouseId(2)));
    assert_eq!(w.obj(n).unwrap().dest, Some(Target::Object(near)));
    assert!(until(&mut w, 600, |w| inside(w, n, near)), "went in");
}


// spec: S-0605 claim 8 (the vehicle's full acceptance test on arrival: refused when full, its destination cleared, guard queued, a forced scatter request)
#[test]
fn claim_8_vehicle_refused_on_arrival() {
    let mut w = game();
    let v = put(&mut w, Kind::Building, "VBIO1D", 0, 0, 0);
    testkit::run(&mut w, 3);
    let a = put(&mut w, Kind::Vehicle, "CAR", 0, 5, 0);
    let c = put(&mut w, Kind::Vehicle, "CAR", 0, 0, 5);
    issue_enter(&mut w, a, v);
    issue_enter(&mut w, c, v);
    assert!(until(&mut w, 5, |w| absorb::contacts(w, v).len() == 2), "both in contact");
    assert!(until(&mut w, 800, |w| absorb::count(w, v) == 1));
    let out = if inside(&w, a, v) { c } else { a };
    assert!(until(&mut w, 800, |w| w.obj(out).unwrap().dest.is_none()));
    testkit::run(&mut w, 30);
    assert!(w.obj(out).unwrap().on_map);
    assert_eq!(absorb::count(&w, v), 1, "capacity re-checked");
    assert_ne!(cur(&w, out), Some(Behaviour::Enter));
}

// spec: S-0214 table I rows x3 and x4 (the docking answer: accepts gives enter; a full absorbing building refuses and gives no enter)
#[test]
fn table_i_x_rows_cursor() {
    let mut w = game();
    let b = bio(&mut w, "BIO");
    let u = walker(&mut w, 5, 5);
    assert_eq!(cursor(&w, &[u], b), Action::Enter);
    for _ in 0..3 {
        fill_one(&mut w, b);
    }
    assert_ne!(cursor(&w, &[u], b), Action::Enter, "full");
}
