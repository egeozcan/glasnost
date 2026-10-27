//! Claim tests of S-0405 claim 13 (revision 21, Q-1505 answered): a
//! vehicle of the powered unit on its own house's repair depot at the loss
//! of the last control centre, on synthetic data (invented types, rates and
//! costs, no game data). Scenario W8 part (m) on our own types: the
//! vehicle reaches each state through the depot's own exchanges (S-0314
//! claims 16 and 17), never by writing the tether.

use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{BuildingType, LocomotionClass, Locomotor, UnitType};
use glasnost_sim::econ::power;
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::targeting::{self, ClickOn, Modifiers};
use glasnost_sim::world::{HouseId, ObjectId, World};
use glasnost_sim::{depot, objects, save, testkit, tick};

const SIZE: i32 = 30;
const A: HouseId = HouseId(0);

fn building(strength: i32, power: i32) -> BuildingType {
    let mut b = BuildingType {
        strength: Some(strength),
        foundation: Some([1, 1]),
        ..Default::default()
    };
    b.econ.power = Some(power);
    b
}

/// PLANT (output 120); CENTRE (a control centre for DRONE, drain 10);
/// DEPOT (a 3 × 3 repair depot, one place, every cell passable, no
/// power need); PAD (DEPOT with the pad flag); PAD2 (PAD with two places,
/// both at the foundation centre); DRONE (a powered-unit
/// wheeled vehicle of strength 600); repair step 10 at 25 %.
fn rules() -> Ruleset {
    let mut r = testkit::ruleset();
    for n in ["PLANT", "CENTRE", "DEPOT", "PAD", "PAD2"] {
        r.registries
            .entry("buildings".into())
            .or_default()
            .push(n.into());
    }
    r.registries
        .entry("vehicles".into())
        .or_default()
        .push("DRONE".into());
    let s = &mut r.sim;
    s.buildings.insert("PLANT".into(), building(500, 120));
    let mut c = building(500, -10);
    c.powered = Some(true);
    c.econ.powers_unit = Some("DRONE".into());
    s.buildings.insert("CENTRE".into(), c);
    let depot = BuildingType {
        strength: Some(800),
        foundation: Some([3, 3]),
        unit_repair: Some(true),
        number_impassable_rows: Some(0),
        cost: Some(800),
        ..Default::default()
    };
    let mut pad = depot.clone();
    pad.helipad = Some(true);
    let mut pad2 = pad.clone();
    pad2.econ.number_of_docks = Some(2);
    pad2.econ.docking_offsets = Some(vec![[0, 0, 0], [0, 0, 0]]);
    s.buildings.insert("DEPOT".into(), depot);
    s.buildings.insert("PAD".into(), pad);
    s.buildings.insert("PAD2".into(), pad2);
    let mut v = UnitType {
        strength: Some(600),
        speed: Some(8),
        rot: Some(8),
        locomotion_class: Some(LocomotionClass::Wheel),
        locomotor: Some(Locomotor::Drive),
        cost: Some(900),
        ..Default::default()
    };
    v.econ.powered_unit = Some(true);
    s.vehicles.insert("DRONE".into(), v);
    s.general.repair_step = Some(10);
    s.general.repair_percent = Some(glasnost_rules::sim::Decimal("25%".into()));
    r
}

/// The power plant and the control centre far from the depot, the depot
/// at the middle of the map, a damaged drone six cells west of it; run past
/// the ticks in which orders are blocked.
fn scene(name: &str) -> (World, ObjectId, ObjectId, ObjectId) {
    let mut w = testkit::game_with(&rules(), SIZE, 5);
    for h in &mut w.houses {
        h.credits = 10_000;
    }
    let at = |dx, dy| testkit::mid(SIZE, dx, dy);
    testkit::spawn(&mut w, Kind::Building, "PLANT", 0, at(-10, -10), 0);
    let k = testkit::spawn(&mut w, Kind::Building, "CENTRE", 0, at(-8, -10), 0);
    let d = testkit::spawn(&mut w, Kind::Building, name, 0, at(0, 0), 0);
    let v = testkit::spawn(&mut w, Kind::Vehicle, "DRONE", 0, at(-6, 1), 0);
    w.obj_mut(v).unwrap().health = 200;
    testkit::run(&mut w, 4);
    (w, k, d, v)
}

fn run_until(w: &mut World, n: i32, mut f: impl FnMut(&World) -> bool) -> bool {
    for _ in 0..n {
        if f(w) {
            return true;
        }
        tick::step(w);
    }
    f(w)
}

/// The click a person gives: the drone over its own depot is *enter*.
fn click_enter(w: &mut World, v: ObjectId, d: ObjectId) {
    let r = targeting::click(w, A, &[v], ClickOn::Object(d), Modifiers::default());
    assert!(!r.orders.is_empty(), "the click gives an enter order");
    for o in r.orders {
        assert!(orders::issue(w, A, o));
    }
}

/// Whether the vehicle stands in a cell of the depot's foundation.
fn on_depot(w: &World, v: ObjectId, d: ObjectId) -> bool {
    let b = w.obj(d).unwrap();
    objects::foundation_cells(b.pos.cell(), b.foundation).contains(&w.obj(v).unwrap().pos.cell())
}

/// Runs until the drone is in contact with the depot, asleep, and the
/// depot repairs it (S-0314 claim 6).
fn serviced(w: &mut World, v: ObjectId, d: ObjectId) {
    assert!(
        run_until(w, 900, |w| w.obj(v).unwrap().contact == Some(d)
            && w.obj(d).unwrap().beh.current == Some(Behaviour::Repair)
            && w.obj(v).unwrap().beh.current == Some(Behaviour::Sleep)),
        "the drone arrives and the depot repairs it"
    );
    assert!(on_depot(w, v, d), "it stands on its depot");
    let h = w.obj(v).unwrap().health;
    assert!(
        run_until(w, 300, |w| w.obj(v).unwrap().health > h),
        "paid repair runs"
    );
}

fn centre_off(w: &mut World, k: ObjectId) {
    assert!(orders::issue(w, A, Order::PowerOff { actor: k }));
    for _ in 0..3 {
        tick::step(w);
    }
}

fn round_trip(w: &World) {
    let h = save::state_hash(w);
    let l = save::load(&save::save(w), w.rules_arc(), testkit::catalogue()).unwrap();
    assert_eq!(save::state_hash(&l), h);
}

// spec: S-0405 claim 13 (revision 21, Q-1505: at an ordinary depot without the pad flag and without either unloading flag the approach and repair exchanges establish no tether, so its untethered customer is disabled at the 1-to-0 transition, though serviced and standing on the depot)
// spec: S-0314 claim 17 (an ordinary depot's repeated request sets no tether; contact alone does not)
#[test]
fn claim_13_ordinary_depot_customer_is_disabled() {
    let (mut w, k, d, v) = scene("DEPOT");
    click_enter(&mut w, v, d);
    serviced(&mut w, v, d);
    assert!(!depot::tethered(&w, v), "no tether at an ordinary depot");
    centre_off(&mut w, k);
    assert!(power::unit_disabled(&w, v), "disabled on its depot");
    round_trip(&w);
}

// spec: S-0405 claim 13 (revision 21, Q-1505: the pad flag does not guarantee tether; a pad depot can repair an untethered vehicle, which is disabled at the transition too; the test reads neither the flag nor the repair phase)
// spec: S-0314 claim 17 (a straight approach to a one-place pad depot: place 0 is the requester itself at the centre, so the repeated request tethers only with no moving destination, which the arrival exchange's sleep does not leave room for)
#[test]
fn claim_13_untethered_pad_customer_is_disabled() {
    let (mut w, k, d, v) = scene("PAD");
    click_enter(&mut w, v, d);
    serviced(&mut w, v, d);
    assert!(!depot::tethered(&w, v), "a straight approach: no tether");
    centre_off(&mut w, k);
    assert!(power::unit_disabled(&w, v), "disabled on a pad depot");
}

// spec: S-0405 claim 13 (revision 21, Q-1505: a vehicle tethered to its own house's depot standing in its current cell is passed over at the 1-to-0 transition and stays enabled; no new disabling event occurs because repair finishes or contact or tether later end; leaving, it is disabled at its first completed arrival in a cell without a building)
// spec: S-0314 claim 17 (a pad depot's repeated request tethers a vehicle in the foundation centre's cell when the vehicle in contact place 0 is more than 128 leptons from that centre)
#[test]
fn claim_13_tethered_pad_customer_is_passed_over() {
    let (mut w, k, d, v) = scene("PAD2");
    // Place 0 goes to a second drone far from the depot: its request is the
    // depot's ordinary entry exchange, and it then stays where it is.
    let far = testkit::spawn(
        &mut w,
        Kind::Vehicle,
        "DRONE",
        0,
        testkit::mid(SIZE, 8, 8),
        0,
    );
    assert!(depot::request(&mut w, far, d, false));
    click_enter(&mut w, v, d);
    assert!(
        run_until(&mut w, 900, |w| depot::tethered(w, v)),
        "the drone is tethered on its way through the centre cell"
    );
    // The far drone is ordered elsewhere: its contact ends and the depot
    // serves the tethered one (S-0314 claim 20).
    assert!(orders::issue(
        &mut w,
        A,
        Order::Behaviour {
            actor: far,
            behaviour: Behaviour::Move,
            attack_move: false,
            target: None,
            dest: Some(glasnost_sim::world::Target::Cell(testkit::mid(SIZE, 10, 8))),
        }
    ));
    serviced(&mut w, v, d);
    assert!(depot::tethered(&w, v));
    assert!(power::tethered_in_own_cell(&w, v));
    centre_off(&mut w, k);
    assert!(!power::unit_disabled(&w, v), "passed over");
    round_trip(&w);
    // Repair runs on and ends; the depot sends it away (S-0314 claim 6).
    // Nothing disables it while it is still on the depot: the repair's
    // end and the end of contact and tether are no events.
    assert!(
        run_until(&mut w, 3000, |w| w.obj(v).unwrap().contact.is_none()),
        "repaired and sent away"
    );
    assert!(on_depot(&w, v, d), "still on its depot");
    assert!(!power::unit_disabled(&w, v), "no event at the repair's end");
    // Disabled at its first arrival in a cell without a building, not
    // before.
    assert!(
        run_until(&mut w, 900, |w| power::unit_disabled(w, v)
            || !on_depot(w, v, d)),
        "it leaves"
    );
    assert!(
        !power::unit_disabled(&w, v),
        "not before it leaves the depot"
    );
    assert!(
        run_until(&mut w, 900, |w| power::unit_disabled(w, v)),
        "disabled on leaving"
    );
    assert!(!on_depot(&w, v, d), "in a cell without a building");
    let _ = far;
}

// spec: S-0405 claim 20 and S-0314 Terms (a disabling's switch-off makes the vehicle settled, also after the guard reset's cleared destination; an enabling ends its being settled)
#[test]
fn claim_20_disabling_settles_and_enabling_ends_it() {
    let (mut w, k, d, v) = scene("DEPOT");
    click_enter(&mut w, v, d);
    serviced(&mut w, v, d);
    w.obj_mut(v).unwrap().depot.settled = depot::Settled::No;
    centre_off(&mut w, k);
    assert!(power::unit_disabled(&w, v));
    assert!(depot::settled(&w, v), "the disabling settles it");
    round_trip(&w);
    assert!(orders::issue(&mut w, A, Order::PowerOn { actor: k }));
    assert!(
        run_until(&mut w, 30, |w| !power::unit_disabled(w, v)),
        "enabled when the centre runs again"
    );
    assert!(!depot::settled(&w, v), "the enabling ends being settled");
}
