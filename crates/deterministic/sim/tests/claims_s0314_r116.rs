//! Claim tests of S-0314 r116 (the answer to Q-3389): a driving vehicle
//! loses its depot destination at its movement arrival in its docking-point
//! cell (claim 17), a failed own-cell search of a contacted driving vehicle
//! in that cell counts as the arrival (claim 21), and hovering vehicles keep
//! theirs. Synthetic data only: invented types, rates and costs.

use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{BuildingType, LocomotionClass, Locomotor, MovementZone, UnitType};
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::world::{HouseId, ObjectId, Target, World};
use glasnost_sim::{depot, drive, testkit, tick};

const SIZE: i32 = 30;
const ALPHA: HouseId = HouseId(0);

fn wheel() -> UnitType {
    UnitType {
        strength: Some(600),
        speed: Some(8),
        rot: Some(8),
        locomotion_class: Some(LocomotionClass::Wheel),
        locomotor: Some(Locomotor::Drive),
        cost: Some(900),
        ..Default::default()
    }
}

/// The combat ruleset plus
/// - LORRY: an unarmed wheeled vehicle of strength 600;
/// - FLOATER: the same with the hovering locomotor;
/// - BAY: a 3 × 3 repair depot whose cells all let vehicles through, one
///   place, no docking displacement;
/// - BAY2: BAY with two places, displaced one cell west and one cell east;
/// - BAYS: BAY with its sole displacement 480 leptons south of the centre,
///   so that a vehicle standing at the centre of the cell below the
///   foundation is in its docking-point cell before its order, 32 leptons
///   from the point (inside the starting distance, S-0504 claim 8);
/// - repair step 10, repair percentage 25 %.
fn rules() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    let mut push = |reg: &str, n: &str| {
        r.registries.entry(reg.into()).or_default().push(n.into());
    };
    push("vehicles", "LORRY");
    push("vehicles", "FLOATER");
    push("buildings", "BAY");
    push("buildings", "BAY2");
    push("buildings", "BAYS");
    let s = &mut r.sim;
    s.vehicles.insert("LORRY".into(), wheel());
    s.vehicles.insert(
        "FLOATER".into(),
        UnitType {
            locomotion_class: Some(LocomotionClass::Hover),
            locomotor: Some(Locomotor::Hover),
            movement_zone: Some(MovementZone::Amphibious),
            ..wheel()
        },
    );
    let bay = BuildingType {
        strength: Some(800),
        foundation: Some([3, 3]),
        unit_repair: Some(true),
        number_impassable_rows: Some(0),
        cost: Some(800),
        ..Default::default()
    };
    let mut two = bay.clone();
    two.econ.number_of_docks = Some(2);
    two.econ.docking_offsets = Some(vec![[-256, 0, 0], [256, 0, 0]]);
    let mut south = bay.clone();
    south.econ.docking_offsets = Some(vec![[0, 480, 0]]);
    s.buildings.insert("BAY".into(), bay);
    s.buildings.insert("BAY2".into(), two);
    s.buildings.insert("BAYS".into(), south);
    s.general.repair_step = Some(10);
    s.general.repair_percent = Some(glasnost_rules::sim::Decimal("25%".into()));
    r
}

fn game() -> World {
    let mut w = testkit::game_with(&rules(), SIZE, 5);
    for h in &mut w.houses {
        h.credits = 10_000;
    }
    w
}

fn spawn(w: &mut World, kind: Kind, name: &str, dx: i32, dy: i32) -> ObjectId {
    testkit::spawn(w, kind, name, 0, testkit::mid(SIZE, dx, dy), 0)
}

fn damaged(w: &mut World, name: &str, dx: i32, dy: i32) -> ObjectId {
    let id = spawn(w, Kind::Vehicle, name, dx, dy);
    w.obj_mut(id).unwrap().health = 300;
    id
}

fn open(w: &mut World) {
    while w.tick < 4 {
        tick::step(w);
    }
}

fn enter(actor: ObjectId, d: ObjectId) -> Order {
    Order::Behaviour {
        actor,
        behaviour: Behaviour::Enter,
        attack_move: false,
        target: None,
        dest: Some(Target::Object(d)),
    }
}

fn arrived(w: &World, v: ObjectId) -> bool {
    depot::link(w, v).is_some_and(|l| l.arrived)
}

/// Per tick after the orders, until `v` is repaired or `n` ticks have run:
/// checks that `v` holds the depot `d` as its destination in every row
/// before its arrival, the rows in which it is already inside its
/// docking-point cell included, and none from the arrival's row on, with its
/// locomotor destination point absent.
fn watch_arrival(w: &mut World, d: ObjectId, v: ObjectId, n: i32) {
    let mut at = None;
    for _ in 0..n {
        tick::step(w);
        let t = w.tick;
        let o = w.obj(v).unwrap();
        if o.health == 600 {
            break;
        }
        if depot::link(w, v).is_none() {
            continue;
        }
        if at.is_none() && arrived(w, v) {
            at = Some(t);
        }
        match at {
            None => assert_eq!(
                o.dest,
                Some(Target::Object(d)),
                "tick {t}: the depot is held before the arrival"
            ),
            Some(_) => {
                assert_eq!(o.dest, None, "tick {t}: no destination from the arrival on");
                assert!(
                    !drive::goal_present(w, v),
                    "tick {t}: no point is installed again"
                );
            }
        }
    }
    assert!(at.is_some(), "the vehicle arrives");
}

// spec: S-0314 claim 17 (r116, Q-3389: a driving vehicle whose order-level destination is the depot loses it at its movement arrival in its docking-point cell, in the arrival's update, the one in which its final track ends there, not an earlier update in which it is already inside that cell; the arrival's exchange is made in that same update; S-0207 claim 30 then installs no point, so it has no moving destination until it is given a new one) and state table (the arrival row)
#[test]
fn claim_17_driving_vehicle_loses_the_depot_at_its_arrival() {
    let mut w = game();
    let d = spawn(&mut w, Kind::Building, "BAY", 0, 0);
    let v = damaged(&mut w, "LORRY", 7, 1);
    open(&mut w);
    assert!(orders::issue(&mut w, ALPHA, enter(v, d)));
    watch_arrival(&mut w, d, v, 3000);
    assert!(
        run_until(&mut w, 3000, |w| w.obj(v).unwrap().health == 600),
        "repaired"
    );
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

// spec: S-0314 claim 17 (r116: the arrival's exchange and the loss of the destination share the arrival's update: in the tick the ready mark is set and sleep queued, the destination is already gone)
#[test]
fn claim_17_the_exchange_and_the_loss_share_the_update() {
    let mut w = game();
    let d = spawn(&mut w, Kind::Building, "BAY", 0, 0);
    let v = damaged(&mut w, "LORRY", 7, 1);
    open(&mut w);
    assert!(orders::issue(&mut w, ALPHA, enter(v, d)));
    let dock = depot::docking_point(&w, d, v).cell();
    let mut seen = false;
    for _ in 0..3000 {
        let was = arrived(&w, v);
        tick::step(&mut w);
        if !was && arrived(&w, v) {
            let o = w.obj(v).unwrap();
            assert_eq!(o.pos.cell(), dock, "in its docking-point cell");
            assert!(
                o.drive.as_ref().unwrap().track.is_none(),
                "its final track has ended"
            );
            assert!(
                o.beh.queued == Some(Behaviour::Sleep) || o.beh.current == Some(Behaviour::Sleep),
                "sleep queued in the arrival"
            );
            assert_eq!(o.dest, None, "the destination went in the same update");
            seen = true;
            break;
        }
    }
    assert!(seen, "the vehicle arrives");
}

// spec: S-0314 claim 17 (r116: the loss applies to a driving vehicle in another contact place too) and Edge cases (several contact places, r116: a driving vehicle that docked in a later place holds no order-level destination by then, so the restarted waiting accepts it once it stands still inside the starting distance); scenario H18 h's expectation for a second driving vehicle
#[test]
fn claim_17_second_place_vehicle_loses_the_depot_and_is_repaired() {
    let mut w = game();
    let d = spawn(&mut w, Kind::Building, "BAY2", 0, 0);
    let a = damaged(&mut w, "LORRY", 7, 1);
    let b = damaged(&mut w, "LORRY", 7, 4);
    open(&mut w);
    assert!(orders::issue(&mut w, ALPHA, enter(a, d)));
    assert!(orders::issue(&mut w, ALPHA, enter(b, d)));
    // spec: S-0314 claim 23 (the later order on a depot that holds a contact gives a queued entry; the retry takes the free place)
    assert!(run_until(&mut w, 60, |w| depot::link(w, b).is_some()));
    assert_eq!(depot::link(&w, a).unwrap().place, 0);
    assert_eq!(depot::link(&w, b).unwrap().place, 1);
    watch_arrival(&mut w, d, b, 6000);
    assert!(
        run_until(&mut w, 6000, |w| w.obj(b).unwrap().health == 600),
        "the second vehicle is repaired"
    );
}

// spec: S-0314 claim 21 (r116, same-cell enter order: a driving vehicle holding the depot and its point in its own cell searches for that cell, which fails and clears the point, S-0207 claim 31; that failure, made while it holds contact and stands in the docking-point cell of its contact place, counts as its movement arrival: the arrival's exchange is made and the order-level destination goes, so claim 30 installs no point again and the depot accepts and repairs it) and claim 17 (r116, the arrival paragraph)
#[test]
fn claim_21_same_cell_failed_search_counts_as_the_arrival() {
    let mut w = game();
    let d = spawn(&mut w, Kind::Building, "BAYS", 0, 0);
    let v = damaged(&mut w, "LORRY", 1, 3);
    open(&mut w);
    let dock = depot::docking_point(&w, d, v);
    let start = w.obj(v).unwrap().pos;
    assert_eq!(
        dock.cell(),
        start.cell(),
        "standing in its docking-point cell"
    );
    assert!(orders::issue(&mut w, ALPHA, enter(v, d)));
    let mut seen = false;
    for _ in 0..600 {
        let was = arrived(&w, v);
        tick::step(&mut w);
        if !was && arrived(&w, v) {
            let o = w.obj(v).unwrap();
            assert_eq!(o.pos, start, "no movement: no track ended");
            assert_eq!(o.dest, None, "the destination goes at the failure");
            assert!(!drive::goal_present(&w, v), "the point is cleared");
            assert!(
                o.beh.queued == Some(Behaviour::Sleep) || o.beh.current == Some(Behaviour::Sleep),
                "sleep queued"
            );
            let b = &w.obj(d).unwrap().beh;
            assert!(
                b.queued == Some(Behaviour::Repair) || b.current == Some(Behaviour::Repair),
                "repair queued at the depot"
            );
            assert!(
                depot::settled(&w, v),
                "read as arrived by movement: settled"
            );
            seen = true;
            break;
        }
    }
    assert!(seen, "the failure counts as the arrival");
    for _ in 0..3000 {
        if w.obj(v).unwrap().health == 600 {
            break;
        }
        assert!(
            w.obj(v).unwrap().dest.is_none() && !drive::goal_present(&w, v),
            "tick {}: no point installed again",
            w.tick
        );
        tick::step(&mut w);
    }
    assert_eq!(w.obj(v).unwrap().health, 600, "the depot repairs it");
}

// spec: S-0314 claim 17 (r116: hovering vehicles are not affected, their destinations follow claim 21) and S-0314 claim 17 (r116: a driving vehicle's arrival exchange removes its depot destination)
#[test]
fn claim_17_only_the_driving_vehicle_loses_its_destination() {
    for (name, keeps) in [("FLOATER", true), ("LORRY", false)] {
        let mut w = game();
        let d = spawn(&mut w, Kind::Building, "BAY", 0, 0);
        let v = damaged(&mut w, name, 1, 1);
        open(&mut w);
        assert!(depot::request(&mut w, v, d, false));
        let t = w.tick;
        w.obj_mut(v).unwrap().beh.force(Behaviour::Enter, t, false);
        w.obj_mut(v).unwrap().dest = Some(Target::Object(d));
        depot::movement_arrival(&mut w, v);
        assert!(arrived(&w, v), "{name}: the exchange is made");
        assert_eq!(
            w.obj(v).unwrap().dest == Some(Target::Object(d)),
            keeps,
            "{name}"
        );
    }
}
