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
/// - BIOZ: BIO with capacity 0; BIOM: BIO with capacity -1;
/// - HEALER: infantry with the negative-damage weapon Fix;
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
    // A factory-marked absorber: a factory category that makes nothing.
    absorber(&mut r, "BIOF", |t| t.factory = Some("terrain".into()));
    absorber(&mut r, "BIO2", |t| t.econ.number_of_docks = Some(2));
    absorber(&mut r, "BIO0", |t| t.econ.extra_power = None);
    absorber(&mut r, "BIOS", |t| t.econ.power = Some(-10));
    absorber(&mut r, "BIOZ", |t| t.passengers = Some(0));
    absorber(&mut r, "BIOM", |t| t.passengers = Some(-1));
    for (n, count) in [("SHEDZ", 0), ("SHEDM", -1), ("SHEDP", 2)] {
        absorber(&mut r, n, |t| {
            t.infantry_absorb = None;
            t.passengers = Some(count);
        });
    }
    r.sim.weapons.insert(
        "Fix".into(),
        WeaponType {
            damage: Some(-10),
            rof: Some(30),
            range: Some(dec("3")),
            speed: Some(100),
            projectile: Some("Bullet".into()),
            warhead: Some("BOOM".into()),
            ..Default::default()
        },
    );
    infantry(&mut r, "HEALER", |t| t.primary = Some("Fix".into()));
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
    assert_eq!(
        absorb::answer(&w, b, car),
        Answer::Refuses,
        "vehicle at an infantry absorber"
    );
    assert_eq!(absorb::answer(&w, v, car), Answer::Accepts);
    assert_eq!(
        absorb::answer(&w, v, u),
        Answer::Refuses,
        "infantry at a vehicle absorber"
    );
    assert_eq!(absorb::answer(&w, b, bal), Answer::Refuses, "balloon hover");
    // A controller holding a victim is refused; its victim is not.
    let psy = put(&mut w, Kind::Infantry, "PSY", 0, -3, 3);
    let held = put(&mut w, Kind::Infantry, "WALKER", 1, -3, 5);
    assert_eq!(absorb::answer(&w, b, psy), Answer::Accepts);
    assert!(mind::take(&mut w, psy, held));
    assert_eq!(
        absorb::answer(&w, b, psy),
        Answer::Refuses,
        "controller with a victim"
    );
    assert_eq!(
        absorb::answer(&w, b, held),
        Answer::Accepts,
        "a controlled unit"
    );
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

// spec: S-0605 claim 17 and Edge cases (Contact outlives enter: an entrant that leaves enter with its contact keeps the link place and the one waiting is refused for good; off, the contact ends and the waiting one goes in); M8 step C7 part 3: the play-fixes layer sets the switch off, read from the layer file
#[test]
fn play_fixes_layer_lets_the_waiting_entrant_in() {
    for layered in [false, true] {
        let r = quirked(|q| {
            if layered {
                *q = glasnost_rules::layers::Layers::PLAY_FIXES
                    .apply_quirks(*q)
                    .unwrap();
            }
        });
        let mut w = game_on(&r);
        let b = bio(&mut w, "BIO");
        let a = walker(&mut w, 8, 0);
        let c = walker(&mut w, 4, 2);
        send(&mut w, a, b);
        send(&mut w, c, b);
        assert_eq!(absorb::waiting_for(&w, c), Some(b));
        {
            let o = w.obj_mut(a).unwrap();
            o.beh.current = Some(Behaviour::Guard);
            o.beh.queued = None;
            o.dest = None;
            o.path.steps.clear();
        }
        let in_c = until(&mut w, 800, |w| inside(w, c, b));
        assert_eq!(
            in_c, layered,
            "the waiting entrant goes in only with the layer"
        );
    }
}

// spec: S-0605 claim 17 and Edge cases (quirks.absorber_stale_contact_kept on: an entrant that leaves enter with its contact keeps its link place, and the one waiting in line is refused for good; off: the contact ends and the waiting one gets it and goes in)
#[test]
fn claim_17_stale_contact_both_positions() {
    for kept in [true, false] {
        let r = quirked(|q| q.absorber_stale_contact_kept = kept);
        let mut w = game_on(&r);
        let b = bio(&mut w, "BIO");
        let a = walker(&mut w, 8, 0);
        let c = walker(&mut w, 4, 2);
        send(&mut w, a, b);
        send(&mut w, c, b);
        assert_eq!(absorb::waiting_for(&w, c), Some(b));
        // Taken out of enter with its contact intact, as by a rule that does so.
        {
            let o = w.obj_mut(a).unwrap();
            o.beh.current = Some(Behaviour::Guard);
            o.beh.queued = None;
            o.dest = None;
            o.path.steps.clear();
        }
        assert_eq!(w.obj(a).unwrap().contact, Some(b));
        if kept {
            testkit::run(&mut w, 600);
            assert_eq!(w.obj(a).unwrap().contact, Some(b), "contact kept");
            assert_eq!(absorb::waiting_for(&w, c), Some(b), "still waiting");
            assert!(!inside(&w, c, b) && !inside(&w, a, b));
            round_trip(&w);
        } else {
            tick::step(&mut w);
            assert_eq!(w.obj(a).unwrap().contact, None);
            assert!(until(&mut w, 800, |w| inside(w, c, b)));
            assert!(!inside(&w, a, b));
        }
    }
}

/// Houses Alpha and Beta of the sandbox game with Beta counting Alpha as
/// allied and not the other way round.
fn one_way_alliance(w: &mut World) {
    let h = &mut w.houses[1].allies;
    if let Err(i) = h.binary_search(&ALPHA) {
        h.insert(i, ALPHA);
    }
}

// spec: S-0605 claims 3 and 5 (contact not granted, an alliance missing in one direction: the actor does not wait in line and keeps the building as its destination; at its enter processing it falls back to rest at once and never goes in)
// spec: S-0605 claim 6 and S-0524 claim 12 (the entry raise is not conditional on contact; without contact no entry)
#[test]
fn claim_3_5_6_not_granted() {
    let mut w = testkit::game_with_triggers(&rules(), SIZE, 7, testkit::entry_triggers());
    testkit::run(&mut w, 2);
    one_way_alliance(&mut w);
    let b = bio(&mut w, "BIO");
    glasnost_sim::triggers::object_tag(&mut w, b, Some(b"ENT"));
    let u = put(&mut w, Kind::Infantry, "WALKER", 1, 4, 0);
    send(&mut w, u, b);
    let o = w.obj(u).unwrap();
    assert_eq!(o.contact, None);
    assert_eq!(absorb::waiting_for(&w, u), None, "no wait");
    assert_eq!(o.dest, Some(Target::Object(b)), "destination kept");
    testkit::run(&mut w, 400);
    assert!(!inside(&w, u, b), "never goes in");
    assert_eq!(absorb::count(&w, b), 0);
    assert_ne!(cur(&w, u), Some(Behaviour::Enter));
    assert!(w.triggers.variables[0], "raised without contact");
}

// spec: S-0605 claim 4 (approach: the first enter processing of a waiting actor chooses a free location near the building's centre cell, assigns it and starts move at once, still waiting; retry: granted and accepted, enter queued with an immediate start and the building as its destination)
#[test]
fn claim_4_approach_and_retry() {
    let mut w = game();
    let b = bio(&mut w, "BIO");
    let a = walker(&mut w, 4, 0);
    let c = walker(&mut w, 8, 6);
    send(&mut w, a, b);
    send(&mut w, c, b);
    assert_eq!(absorb::waiting_for(&w, c), Some(b));
    assert!(until(&mut w, 20, |w| matches!(
        w.obj(c).unwrap().dest,
        Some(Target::Cell(_))
    )));
    assert_eq!(absorb::waiting_for(&w, c), Some(b), "the waiting goes on");
    let Some(Target::Cell(k)) = w.obj(c).unwrap().dest else {
        unreachable!()
    };
    let centre = w.reference_point(b).cell();
    assert!(
        (k.x - centre.x).abs() <= 2 && (k.y - centre.y).abs() <= 2,
        "near the centre cell"
    );
    assert!(cur(&w, c) == Some(Behaviour::Move));
    // The retry once a has gone in.
    assert!(until(&mut w, 800, |w| absorb::waiting_for(w, c).is_none()));
    let o = w.obj(c).unwrap();
    assert_eq!(o.contact, Some(b));
    assert_eq!(o.dest, Some(Target::Object(b)));
    assert!(cur(&w, c) == Some(Behaviour::Enter) || queued(&w, c) == Some(Behaviour::Enter));
    assert!(until(&mut w, 800, |w| inside(w, c, b)));
}

/// The eight neighbours of the cell of `b`'s reference point that are not
/// cells of `b`, in step-direction order.
fn ring_one(w: &World, b: ObjectId) -> Vec<CellXY> {
    let centre = w.reference_point(b).cell();
    glasnost_sim::regions::STEPS
        .iter()
        .map(|&(dx, dy)| CellXY::new(centre.x + dx, centre.y + dy))
        .filter(|&c| {
            !w.cells
                .list(c, glasnost_sim::world::Layer::Ground)
                .contains(&b)
        })
        .collect()
}

/// A waiting walker's destination after its first enter processing, with
/// WALLs on every free ring-one cell of BIO except `keep` (indices into
/// [`ring_one`]); the waiting walker starts at (dx, dy) after `delay`
/// extra ticks. The first walker takes the link place from (-5, -5).
fn approach_cell(keep: &[usize], dx: i32, dy: i32, delay: i32) -> (CellXY, Vec<CellXY>) {
    let mut w = game();
    let b = bio(&mut w, "BIO");
    let ring = ring_one(&w, b);
    for (i, &c) in ring.iter().enumerate() {
        if !keep.contains(&i) {
            testkit::spawn(&mut w, Kind::Building, "WALL", 0, c, 0);
        }
    }
    testkit::run(&mut w, 2 + delay);
    let a = walker(&mut w, -5, -5);
    let c = walker(&mut w, dx, dy);
    send(&mut w, a, b);
    send(&mut w, c, b);
    assert_eq!(absorb::waiting_for(&w, c), Some(b));
    assert!(until(&mut w, 20, |w| matches!(
        w.obj(c).unwrap().dest,
        Some(Target::Cell(_))
    )));
    let Some(Target::Cell(k)) = w.obj(c).unwrap().dest else {
        unreachable!()
    };
    (
        k,
        ring.iter()
            .enumerate()
            .filter(|(i, _)| keep.contains(i))
            .map(|(_, &c)| c)
            .collect(),
    )
}

// spec: S-0605 claim 4 and scenario A2c search variant (i) with S-0212 claim 25 and table T5 (the waiting absorber approach searches from the cell of the building's reference point: with exactly one free cell next to it, the waiting unit walks to that cell whichever side it starts on); Q-1152 answered
#[test]
fn claim_4_approach_one_free_cell() {
    for (dx, dy) in [(6, 0), (-6, 1), (1, 6), (0, -6)] {
        let (k, kept) = approach_cell(&[2], dx, dy, 0);
        assert_eq!(k, kept[0], "start ({dx}, {dy})");
    }
}

// spec: S-0605 claim 4 and scenario A2c search variant (ii) with S-0212 claim 25 (no reference cell: the preferred cell at position N mod n, N the tick of the enter processing; two equally near free cells alternate with the tick's parity)
#[test]
fn claim_4_approach_tie_by_tick() {
    let picks: Vec<CellXY> = (0..4).map(|d| approach_cell(&[0, 4], 6, 0, d).0).collect();
    let (_, kept) = approach_cell(&[0, 4], 6, 0, 0);
    assert!(
        picks.iter().all(|k| kept.contains(k)),
        "{picks:?} among {kept:?}"
    );
    assert_ne!(picks[0], picks[1], "the choice follows the tick");
    assert_eq!(picks[0], picks[2]);
    assert_eq!(picks[1], picks[3]);
}

// spec: S-0605 claim 4 and scenario A2c search variant (iii) (no cell qualifies: the actor falls back to rest in that same processing and stays where it is; it is still waiting, the retry goes on, and it goes in once the place is free)
#[test]
fn claim_4_approach_no_cell() {
    // A small map, so that few cells need filling.
    const SMALL: i32 = 10;
    let mut w = testkit::game_with(&rules(), SMALL, 7);
    testkit::run(&mut w, 2);
    let b = testkit::spawn(
        &mut w,
        Kind::Building,
        "BIO",
        0,
        testkit::mid(SMALL, 0, 0),
        0,
    );
    testkit::run(&mut w, 3);
    let a = testkit::spawn(
        &mut w,
        Kind::Infantry,
        "WALKER",
        0,
        testkit::mid(SMALL, -1, 0),
        0,
    );
    let c = testkit::spawn(
        &mut w,
        Kind::Infantry,
        "WALKER",
        0,
        testkit::mid(SMALL, 2, 2),
        0,
    );
    // Every other free cell of the map holds a building, so no cell qualifies.
    let (ac, cc) = (w.obj(a).unwrap().pos.cell(), w.obj(c).unwrap().pos.cell());
    for x in 0..4 * SMALL {
        for y in 0..4 * SMALL {
            let cell = CellXY::new(x, y);
            if cell != ac
                && cell != cc
                && w.cells.usable(cell)
                && w.cells
                    .list(cell, glasnost_sim::world::Layer::Ground)
                    .is_empty()
            {
                testkit::spawn(&mut w, Kind::Building, "WALL", 0, cell, 0);
            }
        }
    }
    testkit::run(&mut w, 2);
    send(&mut w, a, b);
    send(&mut w, c, b);
    assert_eq!(absorb::waiting_for(&w, c), Some(b));
    testkit::run(&mut w, 3);
    let o = w.obj(c).unwrap();
    assert_eq!(o.dest, None, "no destination");
    assert_ne!(o.beh.current, Some(Behaviour::Enter), "fell back to rest");
    assert_ne!(o.beh.current, Some(Behaviour::Move));
    assert_eq!(o.pos.cell(), cc, "it stays where it is");
    assert_eq!(absorb::waiting_for(&w, c), Some(b), "still waiting");
    assert!(until(&mut w, 800, |w| inside(w, a, b)));
    assert!(
        until(&mut w, 800, |w| inside(w, c, b)),
        "it goes in once the place is free"
    );
}

// spec: S-0605 claim 3 (switched off: the actor asks to load, is refused and gives up the contact; the destination assignment goes on, so it holds the building as its destination without contact)
#[test]
fn claim_3_switched_off_refuses_loading() {
    let mut w = game();
    let b = bio(&mut w, "BIOS");
    let u = walker(&mut w, 4, 0);
    power_off(&mut w, b);
    send(&mut w, u, b);
    assert_eq!(w.obj(u).unwrap().contact, None);
    assert_eq!(
        absorb::contacts(&w, b),
        vec![],
        "the link place is free again"
    );
    assert_eq!(w.obj(u).unwrap().dest, Some(Target::Object(b)));
    testkit::run(&mut w, 200);
    assert!(!inside(&w, u, b));
}

// spec: S-0605 claim 4 (granted after the wait but refused by the acceptance test: contact given up, the none behaviour queued without a start so the current one goes on, destination cleared, waiting ends)
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
    assert_ne!(o.beh.current, Some(Behaviour::Enter));
    assert_ne!(o.beh.queued, Some(Behaviour::Enter));
    testkit::run(&mut w, 200);
    assert!(!inside(&w, c, b));
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

// spec: S-0605 claim 5 (switched off while approaching: the entrant gives up the contact at its next processing and falls back to rest; its destination is not cleared, and it never goes in)
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
    assert_eq!(
        w.obj(u).unwrap().dest,
        Some(Target::Object(b)),
        "destination kept"
    );
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
    assert_eq!(
        w.obj(foe).unwrap().owner,
        Some(HouseId(1)),
        "the original owner"
    );
    assert!(mind::controller(&w, ctrl).is_none_or(|r| r.victims.is_empty()));
    let cleared = fx
        .iter()
        .position(|e| {
            matches!(
                e,
                Effect::Mind {
                    sound: MindSound::Cleared,
                    ..
                }
            )
        })
        .expect("release sound");
    let entry = fx
        .iter()
        .position(|e| {
            matches!(
                e,
                Effect::Sound {
                    sound: Sound::EnterBioReactor,
                    ..
                }
            )
        })
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
            fx += w
                .effects
                .iter()
                .filter(|e| {
                    matches!(
                        e,
                        Effect::Sound {
                            sound: Sound::EnterBioReactor,
                            ..
                        }
                    )
                })
                .count();
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

// spec: S-0605 claim 3 (revision 6: a vehicle ordered in to a vehicle absorber always starts waiting in line for it, even at an empty building: no destination, no contact and no remembered entry object at the order; claim 4's retry then grants contact and it goes in) and scenario A2c's vehicle variant
#[test]
fn claim_3_vehicle_always_waits_in_line() {
    let mut w = game();
    let v = put(&mut w, Kind::Building, "VBIO", 0, 0, 0);
    testkit::run(&mut w, 3);
    let car = put(&mut w, Kind::Vehicle, "CAR", 0, 5, 0);
    let walker_id = walker(&mut w, -5, -6);
    let inf = put(&mut w, Kind::Building, "BIO", 0, -8, -8);
    testkit::run(&mut w, 3);
    assert!(absorb::contacts(&w, v).is_empty(), "an empty building");
    issue_enter(&mut w, car, v);
    issue_enter(&mut w, walker_id, inf);
    orders::apply_pending(&mut w);
    let o = w.obj(car).unwrap();
    assert_eq!(absorb::waiting_for(&w, car), Some(v), "waiting in line");
    assert_eq!(o.dest, None, "no destination at the order");
    assert_eq!(o.contact, None, "no contact at the order");
    assert_eq!(o.depot.entering, None, "no remembered entry object");
    // The contrast: infantry at an empty infantry absorber gets contact.
    assert_eq!(w.obj(walker_id).unwrap().contact, Some(inf));
    assert_eq!(absorb::waiting_for(&w, walker_id), None);
    assert!(
        until(&mut w, 800, |w| inside(w, car, v)),
        "the retry grants contact and it goes in"
    );
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

// spec: S-0214 table M part 3 (deploy on a building: a primary order if it is factory-marked, and an unload behaviour order if it is an absorbing building) and S-0605 claim 11 (the unload releases the passengers)
#[test]
fn a_factory_marked_absorber_gets_both_deploy_orders() {
    let mut w = game();
    let b = bio(&mut w, "BIOF");
    w.obj_mut(b).unwrap().econ.building.primary = false;
    let u = fill_one(&mut w, b);
    assert_eq!(cursor(&w, &[b], b), Action::Deploy);
    let r = click(&mut w, &[b], ClickOn::Object(b));
    assert_eq!(
        r.orders,
        vec![Order::Primary { actor: b }, Order::Deploy { actor: b }]
    );
    assert!(until(&mut w, 300, |w| !inside(w, u, b)), "released");
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
    assert_eq!(
        w.obj(u).unwrap().dest,
        Some(Target::Cell(CellXY::new(x0.x - 1, x0.y + 1)))
    );
    let r = quirked(|q| q.absorber_release_second_round_skips_rear = false);
    let (mut w, b, u) = boxed(&r, 0, &far_ring());
    try_release(&mut w, b);
    assert_eq!(
        w.obj(u).unwrap().pos.cell(),
        CellXY::new(x0.x, x0.y + 1),
        "rear first"
    );
}

// spec: S-0605 claim 12 and Edge cases (quirks.absorber_release_bridge_ends_search on: a deck neighbour at the eighth first-round direction that passes both cell tests ends the search; off: the second round starts)
#[test]
fn claim_12_deck_at_the_eighth_direction_both_positions() {
    // Facing 0: the first round runs 4, 5, 6, 7, 0, 1, 2, 3. Directions 4
    // to 2 have their far cells blocked; direction 3's neighbour (1, 1)
    // carries a deck, its far cell (2, 2) open.
    let blocked: Vec<(i32, i32)> = far_ring().into_iter().filter(|&c| c != (2, 2)).collect();
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
    assert_eq!(
        w.obj(u).unwrap().pos.cell(),
        CellXY::new(x0.x - 1, x0.y + 1)
    );
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
    let boom = rules()
        .sim
        .warheads
        .keys()
        .position(|k| k == "BOOM")
        .unwrap() as u32;
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
    assert_eq!(
        w.obj(c).unwrap().pos.cell(),
        order[0],
        "the most recent takes the first cell"
    );
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
    assert!(until(&mut w, 600, |w| w.obj(a).unwrap().on_map
        && w.obj(c).unwrap().on_map));
    assert_eq!(w.obj(a).unwrap().owner, Some(ALPHA));
    assert_eq!(absorb::count(&w, b), 0);
}

// spec: S-0408 claim 8 (revision 22: the release ignores the type's crewed flag: a building whose type lacks it releases its absorbed passengers as any other does; S-0312's no-crew state never holds at a sale) and claim 7 (revision 22: a type without the crewed flag makes no survivor attempts, C = 0 by S-0312 F5; scenario S6 part e)
#[test]
fn a_sale_of_an_uncrewed_type_releases_its_passengers_and_no_survivors() {
    use glasnost_sim::combat::survivors;
    for crewed in [false, true] {
        let mut r = rules();
        let t = r.sim.buildings.get_mut("BIO").unwrap();
        t.crewed = Some(crewed);
        // The refund override (K = 250) and side divisors of 100: a crewed
        // type's C is positive (S-0312 F5).
        t.econ.soylent = Some(250);
        let g = &mut r.sim.general;
        g.allied_survivor_divisor = Some(100);
        g.soviet_survivor_divisor = Some(100);
        g.third_survivor_divisor = Some(100);
        // House 0's country on side 0, so that F5's divisor applies.
        r.sides = vec![glasnost_rules::schema::Side {
            name: "First".into(),
            members: vec![],
        }];
        r.sim.countries.get_mut("Alphaland").unwrap().econ.side = Some("First".into());
        let mut w = game_on(&r);
        let b = bio(&mut w, "BIO");
        let a = fill_one(&mut w, b);
        let c = fill_one(&mut w, b);
        let src = survivors::source(&w, b).unwrap();
        assert!(!src.no_crew, "the no-crew state never holds at a sale");
        let refund = glasnost_sim::seams::building_refund(&w, b);
        let n = survivors::count(&w, &src, refund);
        if crewed {
            assert!(n > 0, "a crewed type makes attempts: {n}");
        } else {
            assert_eq!(n, 0, "no attempts without the crewed flag");
        }
        glasnost_sim::econ::sell::sell(&mut w, ALPHA, b);
        assert!(
            until(&mut w, 600, |w| w.obj(a).unwrap().on_map
                && w.obj(c).unwrap().on_map),
            "crewed {crewed}: the passengers come out"
        );
        assert_eq!(absorb::count(&w, b), 0);
    }
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

// spec: S-0605 claim 14 and S-0208 claim 3 (destruction: each infantry passenger placed in an open cell takes the sale's choice, the centre placement with one draw over 0 to 3; the same draws and positions as a sale)
#[test]
fn claim_14_destruction_draws_as_the_sale() {
    let mut w = game();
    let b = bio(&mut w, "BIO");
    let a = fill_one(&mut w, b);
    let c = fill_one(&mut w, b);
    let h = lethal(&w, b, false);
    let mut sale = w.clone();
    let before = w.rng.taken();
    absorb::destroyed(&mut w, b, &h);
    let destruction = w.rng.taken() - before;
    let before = sale.rng.taken();
    absorb::sold(&mut sale, b);
    assert_eq!(sale.rng.taken() - before, destruction, "the sale's choice");
    assert_eq!(w.rng, sale.rng);
    for p in [a, c] {
        assert_eq!(w.obj(p).unwrap().pos, sale.obj(p).unwrap().pos);
    }
}

// spec: S-0605 claim 15 and Edge cases (a passenger leaving the game inside lowers n at once; quirks.absorber_removal_keeps_stale_power on: nothing makes the owner's power due, the output keeps the removed passenger's share; off: power due, the output drops)
#[test]
fn claim_15_removal_inside_both_positions() {
    for stale in [true, false] {
        let r = quirked(|q| q.absorber_removal_keeps_stale_power = stale);
        let mut w = game_on(&r);
        let b = bio(&mut w, "BIO");
        let a = fill_one(&mut w, b);
        let c = fill_one(&mut w, b);
        testkit::run(&mut w, 3);
        let (out0, _) = glasnost_sim::econ::power::totals(&w, ALPHA);
        glasnost_sim::objects::remove(&mut w, a);
        assert_eq!(absorb::passengers(&w, b), vec![c]);
        testkit::run(&mut w, 3);
        let (out1, _) = glasnost_sim::econ::power::totals(&w, ALPHA);
        if stale {
            assert_eq!(out1, out0, "stale totals");
        } else {
            assert_eq!(out1, out0 - 25);
        }
        round_trip(&w);
    }
}

// ---------------------------------------------------------------------------
// The bio-reactor fate (claim 16, S-0601 claim 8).

/// The combat rules with the capture fate weights set so that a released
/// unit of the neutral house takes the bio-reactor fate.
fn fate_rules(f: impl FnOnce(&mut glasnost_rules::quirks::Quirks)) -> Ruleset {
    let mut r = rules();
    let g = &mut r.sim.general;
    g.ai_capture_normal = Some(vec![0, 0, 100, 0]);
    g.ai_capture_wounded = Some(vec![0, 0, 0, 0]);
    g.ai_capture_low_power = Some(vec![0, 0, 0, 0]);
    g.ai_capture_low_money = Some(vec![0, 0, 0, 0]);
    f(&mut r.globals.quirks);
    r
}

fn neutral_game(r: &Ruleset) -> World {
    let mut w = testkit::game_with_non_players(r, SIZE, 1);
    testkit::run(&mut w, 2);
    w
}

// spec: S-0605 claim 16 and S-0601 claim 8 (the bio-reactor fate: the released unit of a house that is not human-controlled takes the instruction and enters the nearest absorbing building of its own owner that accepts it)
#[test]
fn bio_reactor_fate_sends_the_released_unit_in() {
    let mut w = neutral_game(&fate_rules(|_| {}));
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
    assert!(absorb::holds_instruction(&w, n));
    assert!(until(&mut w, 600, |w| inside(w, n, near)), "went in");
}

// spec: S-0605 claim 3 (revision 7: claim 16's fate assigning a vehicle victim a vehicle absorber applies the waiting-in-line rule at once: no contact, no destination and no contact request in that tick, whatever the building's contact state; its first request is claim 4's retry) and scenario A13 variant (g)
#[test]
fn claim_3_vehicle_fate_waits_in_line() {
    let mut w = neutral_game(&fate_rules(|_| {}));
    let v = put(&mut w, Kind::Building, "VBIO", 2, 6, 0);
    let car = put(&mut w, Kind::Vehicle, "CAR", 2, 0, 0);
    testkit::run(&mut w, 3);
    assert!(absorb::contacts(&w, v).is_empty(), "an empty building");
    assert!(absorb::bio_reactor_fate(&mut w, car));
    let o = w.obj(car).unwrap();
    assert_eq!(absorb::waiting_for(&w, car), Some(v), "waiting in line");
    assert_eq!(o.contact, None, "no contact from the fate itself");
    assert_eq!(o.dest, None);
    assert_eq!(o.beh.queued, Some(glasnost_sim::rules::Behaviour::Enter));
    // spec: S-0605 scenario A13 (g) (no contact request in the release tick, even from a turn later in that tick; the first comes in its first update)
    absorb::waiting_turn(&mut w, car);
    assert_eq!(
        w.obj(car).unwrap().contact,
        None,
        "no request in the release tick"
    );
    assert_eq!(absorb::waiting_for(&w, car), Some(v));
    // Between ticks the tick number is already the next tick's, so this
    // call counts as made in that tick: its turn pass sends no request.
    testkit::run(&mut w, 1);
    assert_eq!(
        w.obj(car).unwrap().contact,
        None,
        "the rest of the release tick"
    );
    testkit::run(&mut w, 1);
    assert_eq!(
        w.obj(car).unwrap().contact,
        Some(v),
        "its first update's retry"
    );
    assert!(
        until(&mut w, 800, |w| inside(w, car, v)),
        "the retry grants contact and it goes in"
    );
}

// spec: S-0605 claim 16 (equal ρ₃: the building later in the absorber list wins, the one placed most recently)
#[test]
fn claim_16_tie_goes_to_the_later_building() {
    for left_first in [true, false] {
        let mut w = neutral_game(&fate_rules(|_| {}));
        let n = put(&mut w, Kind::Infantry, "WALKER", 2, 0, 0);
        let (l, r) = if left_first {
            let l = put(&mut w, Kind::Building, "BIO", 2, -5, 0);
            (l, put(&mut w, Kind::Building, "BIO", 2, 4, 0))
        } else {
            let r = put(&mut w, Kind::Building, "BIO", 2, 4, 0);
            (put(&mut w, Kind::Building, "BIO", 2, -5, 0), r)
        };
        testkit::run(&mut w, 3);
        let (p, pl, pr) = (
            w.obj(n).unwrap().pos,
            w.reference_point(l),
            w.reference_point(r),
        );
        assert_eq!(pl.y, pr.y);
        assert_eq!(p.x - pl.x, pr.x - p.x, "equal distances");
        assert!(absorb::bio_reactor_fate(&mut w, n));
        let later = if left_first { r } else { l };
        assert_eq!(w.obj(n).unwrap().dest, Some(Target::Object(later)));
        assert_eq!(w.obj(n).unwrap().contact, Some(later));
    }
}

// spec: S-0605 claim 16 (none found: the instruction is cleared and the fate fails; a building that holds another contact can win)
#[test]
fn claim_16_none_found_and_held_contact() {
    let mut w = neutral_game(&fate_rules(|_| {}));
    let n = put(&mut w, Kind::Infantry, "WALKER", 2, 0, 0);
    testkit::run(&mut w, 3);
    assert!(!absorb::bio_reactor_fate(&mut w, n));
    assert!(!absorb::holds_instruction(&w, n));
    let b = put(&mut w, Kind::Building, "BIO", 2, 6, 0);
    let m = put(&mut w, Kind::Infantry, "WALKER", 2, 3, 3);
    testkit::run(&mut w, 3);
    assert!(absorb::bio_reactor_fate(&mut w, m));
    assert_eq!(w.obj(m).unwrap().contact, Some(b));
    assert!(
        absorb::bio_reactor_fate(&mut w, n),
        "held contact does not count"
    );
    assert_eq!(
        absorb::waiting_for(&w, n),
        Some(b),
        "claim 3: waits in line"
    );
}

// spec: S-0215 claim 24 and scenario B21 (d) (an instruction holder waiting in line for the building its guard processing finds is already bound for it: no second enter, the current behaviour stays guard) and S-0605 claims 3 and 16
#[test]
fn claim_16_waiting_in_line_is_already_bound() {
    let mut w = neutral_game(&fate_rules(|_| {}));
    let b = put(&mut w, Kind::Building, "BIO", 2, 6, 0);
    let m = put(&mut w, Kind::Infantry, "WALKER", 2, 3, 3);
    let n = put(&mut w, Kind::Infantry, "WALKER", 2, 0, 0);
    testkit::run(&mut w, 3);
    assert!(absorb::bio_reactor_fate(&mut w, m));
    assert_eq!(w.obj(m).unwrap().contact, Some(b));
    assert!(absorb::bio_reactor_fate(&mut w, n));
    assert_eq!(absorb::waiting_for(&w, n), Some(b));
    let o = w.obj_mut(n).unwrap();
    o.beh.current = Some(Behaviour::Guard);
    o.beh.queued = None;
    absorb::instruction_turn(&mut w, n);
    assert_eq!(queued(&w, n), None, "no second enter");
    assert_eq!(cur(&w, n), Some(Behaviour::Guard));
    assert!(absorb::holds_instruction(&w, n));
    assert_eq!(absorb::waiting_for(&w, n), Some(b));
}

// spec: S-0605 claim 16 and Edge cases (quirks.enter_nearest_survives_absorption on: entry keeps the instruction and a neutral passenger let out walks back in at its next guard processing; off: entry clears it and the passenger stays out)
// spec: S-0215 claim 24 (the guard processing of an instruction holder searches again)
#[test]
fn claim_16_back_in_after_release_both_positions() {
    for survives in [true, false] {
        let mut w = neutral_game(&fate_rules(|q| {
            q.enter_nearest_survives_absorption = survives
        }));
        let b = put(&mut w, Kind::Building, "BIO", 2, 0, 0);
        let n = put(&mut w, Kind::Infantry, "WALKER", 2, 4, 0);
        testkit::run(&mut w, 3);
        assert!(absorb::bio_reactor_fate(&mut w, n));
        assert!(until(&mut w, 600, |w| inside(w, n, b)), "went in");
        assert_eq!(absorb::holds_instruction(&w, n), survives);
        try_release(&mut w, b);
        assert!(w.obj(n).unwrap().on_map, "let out");
        let back = until(&mut w, 800, |w| inside(w, n, b));
        assert_eq!(back, survives);
    }
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
    assert!(
        until(&mut w, 5, |w| absorb::contacts(w, v).len() == 2),
        "both in contact"
    );
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

// spec: S-0214 row I13 and Import mapping (building types Passengers: for any building target, not only an absorbing one, I13's no-passenger test is a count equal to 0; rows x and q test above 0)
#[test]
fn s0214_i13_any_building_count() {
    let mut w = game();
    let z = put(&mut w, Kind::Building, "SHEDZ", 0, -6, -6);
    let m = put(&mut w, Kind::Building, "SHEDM", 0, 6, 6);
    let p = put(&mut w, Kind::Building, "SHEDP", 0, -6, 6);
    let h = put(&mut w, Kind::Infantry, "HEALER", 0, 4, 0);
    testkit::run(&mut w, 3);
    assert!(!absorb::is_absorber(&w, z) && !absorb::is_absorber(&w, m));
    assert_eq!(cursor(&w, &[h], z), Action::Select, "I13: count 0");
    // Count −1 and count 2 skip I13; for this healer the later rows give select too.
    assert_eq!(cursor(&w, &[h], m), cursor(&w, &[h], p));
}

// spec: S-0605 claim 1 (rows x and q test capacity above 0, row I13 equality with 0: no enter over capacity 0 or -1; a healer over capacity 0 takes I13's no-passenger answer, over -1 the row's test does not hold)
// spec: S-0214 table I row I13
#[test]
fn claim_1_i13_capacity_zero_and_negative() {
    let mut w = game();
    let z = put(&mut w, Kind::Building, "BIOZ", 0, -6, -6);
    let m = put(&mut w, Kind::Building, "BIOM", 0, 6, 6);
    let u = walker(&mut w, 0, 4);
    let h = put(&mut w, Kind::Infantry, "HEALER", 0, 4, 0);
    testkit::run(&mut w, 3);
    assert_ne!(cursor(&w, &[u], z), Action::Enter);
    assert_ne!(cursor(&w, &[u], m), Action::Enter);
    assert_eq!(absorb::carries_no_passengers(&w, z), Some(true));
    assert_eq!(absorb::carries_no_passengers(&w, m), Some(false));
    assert_eq!(cursor(&w, &[h], z), Action::Select, "I13");
}
