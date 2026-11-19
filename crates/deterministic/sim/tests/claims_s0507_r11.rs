//! Claim tests of S-0507 revision 11 (Q-3207 answered): claim 6a, a
//! destination that goes inside, and its scenario T16. On a synthetic map
//! with a lake: a hovering transport on the water, an infantry unit on the
//! shore, a driving transport and a garrisonable building of a neutral
//! house. Every name and number here is made up; no game data.

mod common;

use common::{Scene, cell, code_tile};
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{
    ArmourClass, BuildingType, Country, Decimal, LandRow, LandType, LocomotionClass, Locomotor,
    MovementZone, UnitType,
};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::targeting::{self, ClickOn, Modifiers};
use glasnost_sim::world::{HouseId, HouseRole, HouseSetup, ObjectId, Pos, Target, World};
use glasnost_sim::{drive, econ, hover, orders, testkit, tick, transport};

const ALPHA: HouseId = HouseId(0);
/// The neutral civilian house of the scene.
const CIV: u8 = 2;
/// The terrain code of a water tile in the scene's catalogue (S-0211 Terms).
const WATER_CODE: u8 = 9;

fn dec(s: &str) -> Option<Decimal> {
    Some(Decimal(s.into()))
}

fn land_row(ground: &str, float: &str) -> LandRow {
    LandRow {
        foot: dec(ground),
        track: dec(ground),
        wheel: dec(ground),
        hover: dec("1"),
        float: dec(float),
        amphibious: dec("1"),
        float_beach: dec(float),
        buildable: None,
    }
}

/// The combat ruleset with water rows and hover fields, and
/// - SLOWHOV: a slow amphibious hovering transport for four;
/// - HAULER: a wheeled transport for four;
/// - HUT1: a 1 × 1 garrisonable building of the civilian tech level;
/// - the testkit's WALKER, made an occupier;
/// - PLANE: a flying aircraft;
/// - BIO: a 2 × 2 infantry absorber for three;
/// - PIT: a 2 × 2 tank bunker, and TANK: a turreted vehicle with a weapon
///   (the combat ruleset's CRUSHER, not a crusher), bunker-eligible.
fn rules() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    let s = &mut r.sim;
    s.land_types.insert(LandType::Clear, land_row("1", "0"));
    s.land_types.insert(LandType::Road, land_row("1", "0"));
    s.land_types.insert(LandType::Water, land_row("0", "1"));
    s.general.wake = Some("Ripple".into());
    s.general.hover_height = Some(100);
    s.general.hover_bob = dec("0.02");
    s.general.hover_boost = dec("1.5");
    s.general.hover_acceleration = dec("0.02");
    s.general.hover_brake = dec("0.01");
    s.general.hover_dampen = dec("0.75");
    let slow = UnitType {
        strength: Some(100),
        speed: Some(1),
        rot: Some(5),
        locomotion_class: Some(LocomotionClass::Hover),
        locomotor: Some(Locomotor::Hover),
        movement_zone: Some(MovementZone::Amphibious),
        passengers: Some(4),
        size_limit: dec("2"),
        ..Default::default()
    };
    let hauler = UnitType {
        strength: Some(400),
        speed: Some(6),
        rot: Some(8),
        locomotion_class: Some(LocomotionClass::Wheel),
        locomotor: Some(Locomotor::Drive),
        passengers: Some(4),
        size_limit: dec("2"),
        ..Default::default()
    };
    for (n, u) in [("SLOWHOV", slow), ("HAULER", hauler)] {
        r.sim.vehicles.insert(n.into(), u);
        r.registries
            .entry("vehicles".into())
            .or_default()
            .push(n.into());
    }
    let hut = BuildingType {
        strength: Some(400),
        foundation: Some([1, 1]),
        can_be_occupied: Some(true),
        max_number_occupants: Some(3),
        tech_level: Some(-1),
        combat: glasnost_rules::sim::CombatFields {
            armor: Some(ArmourClass::Wood),
            ..Default::default()
        },
        ..Default::default()
    };
    r.sim.buildings.insert("HUT1".into(), hut);
    r.registries
        .entry("buildings".into())
        .or_default()
        .push("HUT1".into());
    if let Some(u) = r.sim.infantry.get_mut("WALKER") {
        u.occupier = Some(true);
    }
    let plane = UnitType {
        strength: Some(100),
        speed: Some(30),
        rot: Some(12),
        locomotor: Some(Locomotor::Fly),
        ..Default::default()
    };
    r.sim.aircraft.insert("PLANE".into(), plane);
    r.registries
        .entry("aircraft".into())
        .or_default()
        .push("PLANE".into());
    let pit = BuildingType {
        strength: Some(1000),
        foundation: Some([2, 2]),
        bunker: Some(true),
        number_impassable_rows: Some(0),
        ..Default::default()
    };
    r.sim.buildings.insert("PIT".into(), pit);
    r.registries
        .entry("buildings".into())
        .or_default()
        .push("PIT".into());
    let mut bio = BuildingType {
        strength: Some(1000),
        foundation: Some([2, 2]),
        infantry_absorb: Some(true),
        passengers: Some(3),
        size_limit: dec("1"),
        ..Default::default()
    };
    bio.econ.power = Some(10);
    r.sim.buildings.insert("BIO".into(), bio);
    r.registries
        .entry("buildings".into())
        .or_default()
        .push("BIO".into());
    let mut tank = r.sim.vehicles["CRUSHER"].clone();
    tank.crusher = None;
    r.sim.vehicles.insert("TANK".into(), tank);
    r.registries
        .entry("vehicles".into())
        .or_default()
        .push("TANK".into());
    r.registries
        .entry("countries".into())
        .or_default()
        .push("Civland".into());
    r.sim.countries.insert(
        "Civland".into(),
        Country {
            multiplay_passive: Some(true),
            econ: glasnost_rules::sim_econ::CountryEcon {
                side: Some("Civilian".into()),
                ..Default::default()
            },
            ..Default::default()
        },
    );
    r
}

/// The lake: the cells with −3 ≤ dx ≤ 3 and −6 ≤ dy ≤ 6.
fn lake(dx: i32, dy: i32) -> bool {
    (-3..=3).contains(&dx) && (-6..=6).contains(&dy)
}

fn world() -> World {
    let mut s = Scene::new();
    s.ruleset = rules();
    s.setup.houses.push(HouseSetup {
        name: "Civ".into(),
        country: "Civland".into(),
        human: false,
        start: None,
        role: HouseRole::Neutral,
        ai: false,
    });
    for dx in -10..=8 {
        for dy in -8..=8 {
            if lake(dx, dy) {
                s.set_cell(cell(dx, dy), code_tile(WATER_CODE), 0);
            }
        }
    }
    let mut w = s.build();
    testkit::run(&mut w, 5);
    w
}

fn at(w: &mut World, kind: Kind, name: &str, house: u8, dx: i32, dy: i32) -> ObjectId {
    testkit::spawn(w, kind, name, house, cell(dx, dy), 0)
}

fn click(w: &mut World, sel: &[ObjectId], on: ClickOn) {
    let r = targeting::click(w, ALPHA, sel, on, Modifiers::default());
    assert!(!r.orders.is_empty(), "the click gives an order");
    for o in r.orders {
        assert!(orders::issue(w, ALPHA, o));
    }
}

fn dest(w: &World, id: ObjectId) -> Option<Target> {
    w.obj(id).unwrap().dest
}

fn inside(w: &World, id: ObjectId) -> bool {
    !w.obj(id).unwrap().on_map
}

fn dist(a: Pos, b: Pos) -> i64 {
    let (dx, dy) = (i64::from(a.x - b.x), i64::from(a.y - b.y));
    dx * dx + dy * dy
}

/// T16's start: the slow hovering transport A on the water next to the
/// shore, I on the shore three cells from A, the driving transport C next to
/// I and the neutral building B next to I. I is ordered to enter A; A takes
/// I as its destination by claim 5's reply on water and I falls back to
/// rest. Returns (A, I, C, B) and A's locomotor point *D* then.
fn t16_start() -> (World, ObjectId, ObjectId, ObjectId, ObjectId, Pos) {
    let mut w = world();
    let a = at(&mut w, Kind::Vehicle, "SLOWHOV", 0, -3, 0);
    let i = at(&mut w, Kind::Infantry, "WALKER", 0, -7, 0);
    let c = at(&mut w, Kind::Vehicle, "HAULER", 0, -8, 0);
    let b = at(&mut w, Kind::Building, "HUT1", CIV, -7, -1);
    testkit::run(&mut w, 5);
    click(&mut w, &[i], ClickOn::Object(a));
    let mut taken = None;
    for _ in 0..200 {
        tick::step(&mut w);
        if dest(&w, a) == Some(Target::Object(i)) {
            taken = hover::destination_point(&w, a);
            break;
        }
    }
    // spec: S-0507 T16 (void the run if A does not take I as its destination)
    let d = taken.expect("A takes I as its destination by claim 5");
    for _ in 0..100 {
        if w.obj(i).unwrap().beh.current != Some(Behaviour::Enter) {
            break;
        }
        tick::step(&mut w);
    }
    assert_ne!(
        w.obj(i).unwrap().beh.current,
        Some(Behaviour::Enter),
        "I falls back to rest (claim 5)"
    );
    (w, a, i, c, b, d)
}

/// Runs T16 after I's second order: until I goes inside, then 150 ticks.
/// Checks claim 6a's expectations for A against `d`, the point recorded at
/// the assignment.
fn t16_after(mut w: World, a: ObjectId, i: ObjectId, d: Pos) {
    let mut gone_in = false;
    for _ in 0..300 {
        let before = dest(&w, a);
        tick::step(&mut w);
        if inside(&w, i) {
            // spec: S-0507 T16 (void a run in which A no longer holds I at the end of the tick before)
            assert_eq!(before, Some(Target::Object(i)), "A held I the tick before");
            gone_in = true;
            break;
        }
    }
    // spec: S-0507 T16 (void a run in which I has not gone inside within 300 ticks)
    assert!(gone_in, "I went inside");
    // spec: S-0507 claim 6a and T16 (no destination at the end of the tick in which I goes inside)
    assert_eq!(dest(&w, a), None, "A loses I as its destination");
    // spec: S-0507 claim 6a (the point A's locomotion last received, at the claim 5 assignment, stays: not where I went inside) and S-0503 claim 20 (r29, Q-3235: D does not follow I before it goes inside either)
    assert_eq!(
        hover::destination_point(&w, a),
        Some(d),
        "A keeps the recorded point"
    );
    let start = dist(w.obj(a).unwrap().pos, d);
    let mut closest = start;
    for _ in 0..150 {
        tick::step(&mut w);
        // spec: S-0507 claim 6a and T16 (no destination at every later recorded tick; A is given no order)
        assert_eq!(dest(&w, a), None, "still no destination");
        closest = closest.min(dist(w.obj(a).unwrap().pos, d));
    }
    // spec: S-0507 claim 6a and T16 (while A still moves, it heads toward the recorded point); S-0503 claims 5, 7 and 15
    assert!(closest < start, "A moves on toward the recorded point");
}

// spec: S-0507 claim 6a and T16 (a): a hovering transport holding an infantry unit by claim 5 loses it as its destination when the unit boards another transport, and keeps its point
#[test]
fn t16a_the_unit_boards_another_transport() {
    let (mut w, a, i, c, _, d) = t16_start();
    click(&mut w, &[i], ClickOn::Object(c));
    t16_after(w, a, i, d);
}

// spec: S-0507 claim 6a and T16 (b): the same when the unit goes into a building as an occupant (S-0508)
#[test]
fn t16b_the_unit_goes_into_a_building() {
    let (mut w, a, i, _, b, d) = t16_start();
    assert!(orders::issue(
        &mut w,
        ALPHA,
        orders::Order::Behaviour {
            actor: i,
            behaviour: Behaviour::Capture,
            attack_move: false,
            target: None,
            dest: Some(Target::Object(b)),
        }
    ));
    t16_after(w, a, i, d);
}

// spec: S-0507 claim 6a (the transport it boards loses it too: a hovering transport holding it by claim 5's reply on water; its position while inside, S-0312 claim 34 (b), is not a destination point) (Q-3207, S26's trace)
#[test]
fn the_transport_it_boards_loses_it() {
    let mut w = world();
    let a = at(&mut w, Kind::Vehicle, "SLOWHOV", 0, -3, 0);
    let i = at(&mut w, Kind::Infantry, "WALKER", 0, -6, 0);
    testkit::run(&mut w, 5);
    // spec: S-0507 claim 5 (a transport on water with no destination takes the requester as its destination)
    assert_eq!(
        transport::load_request(&mut w, a, i),
        transport::Reply::Approach
    );
    assert_eq!(dest(&w, a), Some(Target::Object(i)));
    let d = hover::destination_point(&w, a).unwrap();
    transport::board(&mut w, i, a);
    assert_eq!(transport::riding(&w, i), Some(a));
    assert_eq!(dest(&w, a), None, "lost at the boarding");
    assert_eq!(hover::destination_point(&w, a), Some(d), "the point stays");
    for _ in 0..3 {
        tick::step(&mut w);
        assert_eq!(dest(&w, a), None, "no destination after");
    }
}

// spec: S-0507 claim 6a (any vehicle or infantry holder, transport or not, loses the object as its destination at once; the driving locomotor's destination point stays as it was, S-0207 claims 25 and 26 as r11's Changes for other specs amend claim 26's closing paragraph)
#[test]
fn other_holders_lose_it_and_a_driver_keeps_its_point() {
    let mut w = world();
    let i = at(&mut w, Kind::Infantry, "WALKER", 0, -7, 0);
    let c = at(&mut w, Kind::Vehicle, "HAULER", 0, -9, 0);
    let v = at(&mut w, Kind::Vehicle, "HAULER", 0, -8, 6);
    let f = at(&mut w, Kind::Infantry, "WALKER", 0, -6, 6);
    testkit::run(&mut w, 5);
    econ::assign_destination(&mut w, v, Some(Target::Object(i)));
    econ::assign_destination(&mut w, f, Some(Target::Object(i)));
    let goal = drive::goal(&w, v).expect("a driving point at the assignment");
    click(&mut w, &[i], ClickOn::Object(c));
    let mut gone_in = false;
    for _ in 0..300 {
        tick::step(&mut w);
        if inside(&w, i) {
            gone_in = true;
            break;
        }
    }
    assert!(gone_in, "I boarded C");
    assert_eq!(dest(&w, v), None, "the vehicle lost it");
    assert_eq!(dest(&w, f), None, "the infantry unit lost it");
    assert_eq!(drive::goal(&w, v), Some(goal), "the driving point stays");
    for _ in 0..3 {
        tick::step(&mut w);
        assert_eq!(dest(&w, v), None);
        assert_eq!(dest(&w, f), None);
    }
    // spec: S-0207 claim 25 (only a stop, a replacement or a qualifying completion removes the point)
    assert_eq!(
        drive::goal(&w, v),
        Some(goal),
        "not cleared at the next takes"
    );
}

// spec: S-0507 claim 6a (in the same moment the object leaves the holder's queued and approach destinations, wherever it stands in them; the other entries keep their order, and a destination that is another target stays)
#[test]
fn it_leaves_queued_destinations() {
    let mut w = world();
    let i = at(&mut w, Kind::Infantry, "WALKER", 0, -7, 0);
    let c = at(&mut w, Kind::Vehicle, "HAULER", 0, -8, 0);
    let v = at(&mut w, Kind::Vehicle, "HAULER", 0, -8, 6);
    testkit::run(&mut w, 5);
    let (p, q) = (Target::Cell(cell(-9, 6)), Target::Cell(cell(-9, 7)));
    {
        let o = w.obj_mut(v).unwrap();
        o.dest = Some(Target::Cell(cell(-8, 7)));
        o.queued_dests = vec![p, Target::Object(i), q, Target::Object(i)];
    }
    transport::board(&mut w, i, c);
    let o = w.obj(v).unwrap();
    assert_eq!(o.queued_dests, vec![p, q]);
    assert_eq!(
        o.dest,
        Some(Target::Cell(cell(-8, 7))),
        "its own destination stays"
    );
}

// spec: S-0507 claim 6a (a revert in the same moment comes first: the destination given back that is the object itself is then cleared at once, and the locomotion keeps the object's movement point at the moment of the loss) and S-0309 claims 27 and 27a
#[test]
fn a_revert_in_the_same_moment() {
    let mut w = world();
    let i = at(&mut w, Kind::Infantry, "WALKER", 0, -7, 0);
    let c = at(&mut w, Kind::Vehicle, "HAULER", 0, -8, 0);
    let h = at(&mut w, Kind::Vehicle, "HAULER", 0, -8, 6);
    testkit::run(&mut w, 5);
    let here = w.obj(i).unwrap().pos;
    {
        let o = w.obj_mut(h).unwrap();
        o.target = Some(Target::Object(i));
        o.beh.current = Some(Behaviour::Attack);
        o.beh.queued = None;
        o.beh.suspended = Some(Behaviour::Guard);
        o.beh.aside = Some((Some(Target::Object(i)), Some(Target::Object(i))));
    }
    transport::board(&mut w, i, c);
    let o = w.obj(h).unwrap();
    assert_eq!(o.beh.current, Some(Behaviour::Guard), "reverted");
    assert_eq!(o.target, Some(Target::Object(i)), "claim 27: given back");
    assert_eq!(o.dest, None, "the destination given back is lost");
    assert_eq!(
        drive::goal(&w, h),
        Some(here),
        "the locomotion keeps the object's movement point at the loss"
    );
    tick::step(&mut w);
    assert_eq!(dest(&w, h), None);
    assert_eq!(drive::goal(&w, h), Some(here));
}

// spec: S-0507 claim 6a (aircraft holders follow S-0501 claim 53 instead) and S-0501 claim 53 (an aircraft loses an object as its destination when it leaves the map by boarding any transport, from that moment in the tick)
#[test]
fn an_aircraft_holder_follows_s0501_claim_53() {
    let mut w = world();
    let i = at(&mut w, Kind::Infantry, "WALKER", 0, -7, 0);
    let c = at(&mut w, Kind::Vehicle, "HAULER", 0, -8, 0);
    let p = at(&mut w, Kind::Aircraft, "PLANE", 0, 6, 6);
    testkit::run(&mut w, 5);
    assert!(w.obj(p).unwrap().air.is_some(), "void: no aircraft state");
    w.obj_mut(p).unwrap().dest = Some(Target::Object(i));
    transport::board(&mut w, i, c);
    assert_eq!(transport::riding(&w, i), Some(c), "void: not boarded");
    assert_eq!(dest(&w, p), None, "the aircraft lost it");
}

// spec: S-0507 claim 6a (r12, Q-3237: going into an absorber, as an infantry unit or as a vehicle, S-0605 claims 7 and 8, is going inside: a holder loses it as its destination)
#[test]
fn going_into_an_absorber_is_going_inside() {
    let mut w = world();
    let b = at(&mut w, Kind::Building, "BIO", 0, 5, 3);
    let i = at(&mut w, Kind::Infantry, "WALKER", 0, 6, 7);
    let v = at(&mut w, Kind::Vehicle, "HAULER", 0, -8, 6);
    testkit::run(&mut w, 5);
    econ::assign_destination(&mut w, v, Some(Target::Object(i)));
    assert!(orders::issue(
        &mut w,
        ALPHA,
        orders::Order::Behaviour {
            actor: i,
            behaviour: Behaviour::Enter,
            attack_move: false,
            target: None,
            dest: Some(Target::Object(b)),
        }
    ));
    let mut gone_in = false;
    for _ in 0..600 {
        let before = dest(&w, v);
        tick::step(&mut w);
        if glasnost_sim::structures::absorb::absorbed_in(&w, i) == Some(b) {
            assert_eq!(before, Some(Target::Object(i)), "void: V lost I before");
            gone_in = true;
            break;
        }
    }
    assert!(gone_in, "void: I was not absorbed");
    assert!(inside(&w, i));
    assert_eq!(dest(&w, v), None, "V loses I");
}

// spec: S-0507 claim 6a (a vehicle that enters a tank bunker stays on the map, S-0606 claim 7, so claim 6a does not apply to it: a holder keeps it as its destination)
#[test]
fn a_tank_bunker_is_not_going_inside() {
    let mut w = world();
    let b = at(&mut w, Kind::Building, "PIT", 0, 5, -7);
    let u = at(&mut w, Kind::Vehicle, "TANK", 0, 5, -3);
    let v = at(&mut w, Kind::Vehicle, "HAULER", 0, -8, 6);
    testkit::run(&mut w, 5);
    econ::assign_destination(&mut w, v, Some(Target::Object(u)));
    assert!(orders::issue(
        &mut w,
        ALPHA,
        orders::Order::Behaviour {
            actor: u,
            behaviour: Behaviour::Enter,
            attack_move: false,
            target: None,
            dest: Some(Target::Object(b)),
        }
    ));
    let mut bunkered = false;
    for _ in 0..600 {
        assert_eq!(
            dest(&w, v),
            Some(Target::Object(u)),
            "void: V lost U before it was bunkered"
        );
        tick::step(&mut w);
        if glasnost_sim::structures::bunker::bunkered_in(&w, u) == Some(b) {
            bunkered = true;
            break;
        }
    }
    assert!(bunkered, "void: U was not bunkered");
    assert!(!inside(&w, u), "S-0606 claim 7: it stays on the map");
    assert_eq!(dest(&w, v), Some(Target::Object(u)), "V keeps it");
}

/// From T16's start, orders I to a far shore cell and runs until I has left
/// the cell of A's *D* while A still holds I. Returns whether that came.
fn walk_away(w: &mut World, a: ObjectId, i: ObjectId, d: Pos) -> bool {
    click(w, &[i], ClickOn::Cell(cell(-10, 7)));
    for _ in 0..200 {
        tick::step(w);
        if dest(w, a) != Some(Target::Object(i)) {
            return false;
        }
        if w.obj(i).unwrap().pos.cell() != d.cell() {
            return true;
        }
    }
    false
}

// spec: S-0503 claim 20 (r29, Q-3235: D is the object's movement point at the effective assignment and does not follow it; a search looks for D's cell, not the cell the object is in) and S-0507 T16 part (c) (the object walks away without going inside)
#[test]
fn t16c_d_stays_where_the_unit_was_at_the_assignment() {
    let (mut w, a, i, _, _, d) = t16_start();
    assert!(walk_away(&mut w, a, i, d), "void: I did not leave D's cell");
    assert_eq!(hover::destination_point(&w, a), Some(d), "D stays");
    assert_eq!(
        glasnost_sim::path::goal_cell(&w, a),
        Some(d.cell()),
        "a search looks for D's cell"
    );
}

// spec: S-0503 claim 20 (r29: a vehicle ignores an unchanged order-level destination unless its destination-refresh state is on, S-0502 claims 24 and 27, and the ignored assignment leaves D; a destination first emptied and then given again takes the object's later position, with the height of Terms)
#[test]
fn an_unchanged_assignment_leaves_d_and_a_new_one_takes_the_later_position() {
    let (mut w, a, i, _, _, d) = t16_start();
    assert!(walk_away(&mut w, a, i, d), "void: I did not leave D's cell");
    econ::assign_destination(&mut w, a, Some(Target::Object(i)));
    assert_eq!(hover::destination_point(&w, a), Some(d), "ignored");
    econ::assign_destination(&mut w, a, None);
    econ::assign_destination(&mut w, a, Some(Target::Object(i)));
    // spec: S-0503 claim 20 (r30: a walking infantry unit gives its heading point, its next movement point, or its position when it has none)
    let o = w.obj(i).unwrap();
    let p = o.walk.as_ref().and_then(|s| s.target).unwrap_or(o.pos);
    // spec: S-0503 Terms (unless held, D's height is the surface height, plus 416 leptons on a high-bridge cell; none here)
    let later = Pos::new(p.x, p.y, w.surface_at(p.x, p.y));
    assert_eq!(
        hover::destination_point(&w, a),
        Some(later),
        "the later position"
    );
}

// spec: S-0507 claim 6a (r12, Q-3236: walking: S-0208 claim 16 with the point as its locomotor destination; the walker goes on along its stored path, and its fresh searches look for that point's cell) and S-0208 claims 6 and 16 (r67: the loss does not end the walk by claim 16's first case; its second case does, in the point's cell)
#[test]
fn a_walking_holder_goes_on_to_the_point() {
    let mut w = world();
    let i = at(&mut w, Kind::Infantry, "WALKER", 0, -7, 0);
    let c = at(&mut w, Kind::Vehicle, "HAULER", 0, -8, 0);
    let f = at(&mut w, Kind::Infantry, "WALKER", 0, -8, 7);
    testkit::run(&mut w, 5);
    let p = w.obj(i).unwrap().pos;
    econ::assign_destination(&mut w, f, Some(Target::Object(i)));
    testkit::run(&mut w, 3);
    assert!(
        !w.obj(f).unwrap().path.steps.is_empty(),
        "void: F has a stored path"
    );
    click(&mut w, &[i], ClickOn::Object(c));
    let mut gone_in = false;
    for _ in 0..300 {
        tick::step(&mut w);
        if inside(&w, i) {
            gone_in = true;
            break;
        }
    }
    assert!(gone_in, "void: I did not board C");
    assert_eq!(dest(&w, f), None, "F lost I");
    let mut ended = false;
    for _ in 0..300 {
        tick::step(&mut w);
        assert_eq!(dest(&w, f), None, "still no destination");
        if !glasnost_sim::occupancy::in_progress(&w, f) {
            ended = true;
            break;
        }
    }
    assert!(ended, "F's walk ends");
    assert_eq!(
        w.obj(f).unwrap().pos.cell(),
        p.cell(),
        "it ends in the point's cell"
    );
}

// spec: S-0208 Terms (r67: for a destination that names an object, the locomotor destination is the object's movement point at the assignment; it does not follow the object as it moves) and claim 16 (its second case tests that point)
#[test]
fn a_walker_ends_its_walk_at_the_point_of_the_assignment() {
    let mut w = world();
    let i = at(&mut w, Kind::Infantry, "WALKER", 0, -7, 0);
    let f = at(&mut w, Kind::Infantry, "WALKER", 0, -8, 5);
    testkit::run(&mut w, 5);
    let p = w.obj(i).unwrap().pos;
    econ::assign_destination(&mut w, f, Some(Target::Object(i)));
    testkit::run(&mut w, 3);
    assert!(
        !w.obj(f).unwrap().path.steps.is_empty(),
        "void: F has a stored path"
    );
    click(&mut w, &[i], ClickOn::Cell(cell(-10, -7)));
    let mut ended = false;
    for _ in 0..300 {
        tick::step(&mut w);
        if dest(&w, f).is_none() && !glasnost_sim::occupancy::in_progress(&w, f) {
            ended = true;
            break;
        }
    }
    assert!(ended, "F's walk ends");
    assert_ne!(w.obj(i).unwrap().pos.cell(), p.cell(), "void: I moved");
    assert_eq!(
        w.obj(f).unwrap().pos.cell(),
        p.cell(),
        "it ends in the point's cell"
    );
}
