//! Claim tests of S-0417 revision 20 claim 16 (answers Q-1110 question 3):
//! the lifetime of a teleporting vehicle's temporary driving after a
//! factory exit, with S-0207 revision 34 claim 25 (*Factory-exit lifetime*)
//! and S-0402 revision 27 claim 17 (*Temporary driving after production*).
//! The fixture is the harvest ruleset with a vehicle factory VF and a chrono
//! miner CHRONOT whose own locomotor teleports. Every name and number is
//! invented; no game data.

mod harvest_common;

use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{BuildingType, Locomotor};
use glasnost_sim::econ::harvest::{self, Phase};
use glasnost_sim::econ::release::{self, Outcome};
use glasnost_sim::objects;
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, Target, World};
use harvest_common::*;

fn ruleset() -> Ruleset {
    let mut r = harvest_common::ruleset();
    let mut vf = BuildingType {
        strength: Some(500),
        foundation: Some([3, 2]),
        factory: Some("vehicle".into()),
        weapons_factory: Some(true),
        ..Default::default()
    };
    vf.deploy_time = dec("0.01");
    vf.combat.armor = None;
    r.registries.get_mut("buildings").unwrap().push("VF".into());
    r.sim.buildings.insert("VF".into(), vf);
    let mut chrono = harvester(CAPACITY);
    chrono.teleporter = Some(true);
    chrono.locomotor = Some(Locomotor::Teleport);
    chrono.econ.dock = Some(vec!["REFN".into()]);
    r.registries
        .get_mut("vehicles")
        .unwrap()
        .push("CHRONOT".into());
    r.sim.vehicles.insert("CHRONOT".into(), chrono);
    r
}

/// The factory's top-left cell.
fn factory_cell() -> CellXY {
    at(0, 0)
}

/// A 3 × 3 ORE field with its top-left cell at (x, y).
fn field(x: i32, y: i32) -> Vec<(CellXY, u8, u8)> {
    let mut v = Vec::new();
    for dy in 0..3 {
        for dx in 0..3 {
            v.push((at(x + dx, y + dy), ORE, 5));
        }
    }
    v
}

/// A game with the factory and a refinery of house 0, and `ore`.
fn game(ore: &[(CellXY, u8, u8)]) -> (World, ObjectId) {
    let mut w = game_with_ore(&ruleset(), ore, 7);
    testkit::run(&mut w, 4);
    let k = place_building(&mut w, "VF", 0, factory_cell());
    place_building(&mut w, "REFN", 0, at(-9, -9));
    (w, k)
}

/// Releases a chrono miner from the factory; runs until its clear report.
fn released(w: &mut World, k: ObjectId) -> ObjectId {
    let ti = w.rules().index_of(Kind::Vehicle, "CHRONOT").unwrap();
    let u = objects::create(w, Kind::Vehicle, ti, Some(HouseId(0)));
    assert_eq!(release::release(w, k, u), Outcome::Released);
    let mut drove = false;
    assert!(
        until(w, 600, |w| {
            drove |= release::temporary_drive(w, u);
            w.obj(u).unwrap().contact.is_none()
        }),
        "the clear report"
    );
    assert!(drove, "it drives for its exit");
    u
}

fn until(w: &mut World, max: i32, mut f: impl FnMut(&World) -> bool) -> bool {
    for _ in 0..max {
        testkit::run(w, 1);
        if f(w) {
            return true;
        }
    }
    false
}

fn in_field(c: CellXY, x: i32, y: i32) -> bool {
    let tl = at(x, y);
    (tl.x..tl.x + 3).contains(&c.x) && (tl.y..tl.y + 3).contains(&c.y)
}

// spec: S-0417 claim 16 (the clear report, the loss of contact and tether and queueing harvest are no stop or restoration event; a factory-built chrono miner finding ore in a different cell has the ore destination replace the exit point and drives its first leg there with its temporary driving locomotion; arrival at that first ore cell and starting loading do not restore it)
// spec: S-0402 claim 17 (Temporary driving after production: the first leg to ore found in another cell is driven; queueing or starting harvest, arriving at the first ore cell and starting loading do not themselves restore teleporting locomotion)
// spec: S-0207 claim 25 (Factory-exit lifetime: a later accepted ore destination replaces the retained point in the active driving locomotion)
#[test]
fn claim_16_a_chrono_miner_drives_its_first_leg_to_the_ore() {
    let (mut w, k) = game(&field(8, 8));
    let u = released(&mut w, k);
    let o = w.obj(u).unwrap();
    assert!(
        release::temporary_drive(&w, u),
        "the report does not restore"
    );
    assert!(
        o.beh.current == Some(Behaviour::Harvest) || o.beh.queued == Some(Behaviour::Harvest),
        "the run-away reply queues harvest"
    );
    // The seeking ring search offers an ore cell: the driving locomotor takes it.
    assert!(until(&mut w, 400, |w| w.obj(u).unwrap().dest.is_some_and(
        |d| matches!(d, Target::Cell(c) if in_field(c, 8, 8))
    )));
    assert!(release::temporary_drive(&w, u), "still driving");
    let o = w.obj(u).unwrap();
    let goal = o
        .drive
        .as_ref()
        .unwrap()
        .goal
        .expect("the ore point replaced the exit point");
    assert!(in_field(goal.cell(), 8, 8));
    // It drives there (no jump of more than two cells in one tick) and starts loading, still driving.
    let mut last = w.obj(u).unwrap().pos.cell();
    let loading = until(&mut w, 2000, |w| {
        let c = w.obj(u).unwrap().pos.cell();
        assert!(
            (c.x - last.x).abs() <= 2 && (c.y - last.y).abs() <= 2,
            "no teleport on the first leg"
        );
        last = c;
        harvest::state_of(w, u).phase == Phase::Loading
    });
    assert!(loading, "it reaches the field and loads");
    assert!(in_field(w.obj(u).unwrap().pos.cell(), 8, 8));
    assert!(
        release::temporary_drive(&w, u),
        "arrival and loading do not restore"
    );
}

// spec: S-0417 claim 16 (a later accepted destination replaces the exit point; its completion clears the point under S-0207 claims 25 and 26, and a subsequent fall-back to rest that finds every claim 5 predicate restores the suspended locomotion; the next accepted distant move then teleports)
#[test]
fn claim_16_a_later_move_clears_the_point_and_the_fallback_restores() {
    let (mut w, k) = game(&[]);
    let u = released(&mut w, k);
    assert!(release::temporary_drive(&w, u));
    while w.tick <= w.orders_blocked_through {
        testkit::run(&mut w, 1);
    }
    let c = at(-4, 7);
    let mv = |c: CellXY| Order::Behaviour {
        actor: u,
        behaviour: Behaviour::Move,
        attack_move: false,
        target: None,
        dest: Some(Target::Cell(c)),
    };
    assert!(orders::issue(&mut w, HouseId(0), mv(c)));
    // The move replaces the exit point in driving locomotion.
    testkit::run(&mut w, 1);
    assert!(release::temporary_drive(&w, u));
    assert_eq!(
        w.obj(u)
            .unwrap()
            .drive
            .as_ref()
            .unwrap()
            .goal
            .map(|g| g.cell()),
        Some(c)
    );
    assert!(
        until(&mut w, 2000, |w| !release::temporary_drive(w, u)),
        "restored at the fall-back after the move"
    );
    assert_eq!(w.obj(u).unwrap().pos.cell(), c, "it drove there");
    // The next accepted distant move teleports.
    let far = at(8, -8);
    assert!(orders::issue(&mut w, HouseId(0), mv(far)));
    assert!(until(&mut w, 5, |w| w.obj(u).unwrap().pos.cell() == far));
}

// spec: S-0417 claim 16 (No follow-up goal: there is no factory-exit timeout clearing the exit point; without a replacement or a stop it keeps preventing restoration, here for 300 ticks after the clear report with no field)
// spec: S-0207 claim 25 (Factory-exit lifetime: completing the exit track with no order-level destination retains its locomotor destination point)
#[test]
fn claim_16_no_field_no_timer() {
    let (mut w, k) = game(&[]);
    let u = released(&mut w, k);
    for _ in 0..300 {
        testkit::run(&mut w, 1);
        let o = w.obj(u).unwrap();
        // Nothing in this scene supplies a destination or a stop (no field: harvest goes idle, then guard).
        assert_eq!(o.dest, None);
        assert!(release::temporary_drive(&w, u), "no timer restores it");
        assert!(
            o.drive.as_ref().unwrap().goal.is_some(),
            "the exit point is retained"
        );
    }
}
