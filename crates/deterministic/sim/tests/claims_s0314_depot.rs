//! Claim tests of the depot cycle: S-0314 claims 6 and 16 to 24, S-0504
//! claim 14 and S-0215 claim 29 (repair depots and repairing naval yards)
//! on synthetic data: invented types, rates and costs, no game data.

use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{BuildingType, LocomotionClass, Locomotor, MovementZone, UnitType};
use glasnost_sim::combat::damage::{self, Hit};
use glasnost_sim::combat::effects::{Effect, Sound};
use glasnost_sim::econ::sell;
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::targeting::{self, Action, ClickOn, Modifiers};
use glasnost_sim::world::{HouseId, HouseRole, ObjectId, Pos, Target, World};
use glasnost_sim::{depot, objects, save, testkit, tick};

const SIZE: i32 = 30;
const ALPHA: HouseId = HouseId(0);

fn wheel(strength: i32) -> UnitType {
    UnitType {
        strength: Some(strength),
        speed: Some(8),
        rot: Some(8),
        locomotion_class: Some(LocomotionClass::Wheel),
        locomotor: Some(Locomotor::Drive),
        cost: Some(900),
        ..Default::default()
    }
}

/// The combat ruleset plus
/// - TRUCK: an unarmed wheeled vehicle of strength 600 and cost 900;
/// - HOPPER: the same in movement zone fly;
/// - DEPOT: a 3 × 3 repair depot whose cells all let vehicles through
///   (no impassable rows), one place, no docking displacement;
/// - DEPOT2: the same with two places and two displacements;
/// - YARD: a 3 × 3 vehicle factory with the repair flag (land, so a land
///   vehicle may use it: the naval match does not refuse);
/// - repair step 10, repair percentage 25 %: a step costs 3 credits.
fn rules() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    let mut push = |reg: &str, n: &str| {
        r.registries.entry(reg.into()).or_default().push(n.into());
    };
    push("vehicles", "TRUCK");
    push("vehicles", "HOPPER");
    push("buildings", "DEPOT");
    push("buildings", "DEPOT2");
    push("buildings", "YARD");
    let s = &mut r.sim;
    s.vehicles.insert("TRUCK".into(), wheel(600));
    let mut hop = wheel(600);
    hop.movement_zone = Some(MovementZone::Fly);
    s.vehicles.insert("HOPPER".into(), hop);
    let depot = BuildingType {
        strength: Some(800),
        foundation: Some([3, 3]),
        unit_repair: Some(true),
        number_impassable_rows: Some(0),
        cost: Some(800),
        ..Default::default()
    };
    let mut depot = depot;
    // A build-up sprite, so that a sale runs (S-0408 claim 4).
    depot.econ.buildup = Some(true);
    let mut two = depot.clone();
    two.econ.number_of_docks = Some(2);
    two.econ.docking_offsets = Some(vec![[-256, 0, 0], [256, 0, 0]]);
    let yard = BuildingType {
        factory: Some("vehicle".into()),
        ..depot.clone()
    };
    s.buildings.insert("DEPOT".into(), depot);
    s.buildings.insert("DEPOT2".into(), two);
    s.buildings.insert("YARD".into(), yard);
    s.general.repair_step = Some(10);
    s.general.repair_percent = Some(glasnost_rules::sim::Decimal("25%".into()));
    r
}

fn game(r: &Ruleset) -> World {
    let mut w = testkit::game_with(r, SIZE, 5);
    for h in &mut w.houses {
        h.credits = 10_000;
    }
    w
}

fn spawn(w: &mut World, kind: Kind, name: &str, house: u8, dx: i32, dy: i32) -> ObjectId {
    testkit::spawn(w, kind, name, house, testkit::mid(SIZE, dx, dy), 0)
}

/// A depot of type `name` with its top-left cell at the middle of the map.
fn depot_at(w: &mut World, name: &str) -> ObjectId {
    spawn(w, Kind::Building, name, 0, 0, 0)
}

fn truck(w: &mut World, house: u8, dx: i32, dy: i32, health: i32) -> ObjectId {
    let id = spawn(w, Kind::Vehicle, "TRUCK", house, dx, dy);
    w.obj_mut(id).unwrap().health = health;
    id
}

fn open(w: &mut World) {
    while w.tick < 4 {
        tick::step(w);
    }
}

fn click(w: &mut World, actor: ObjectId, on: ObjectId) -> Vec<Order> {
    targeting::click(
        w,
        ALPHA,
        &[actor],
        ClickOn::Object(on),
        Modifiers::default(),
    )
    .orders
}

fn click_enter(w: &mut World, actor: ObjectId, on: ObjectId) {
    let o = click(w, actor, on);
    assert!(!o.is_empty(), "the click gives an enter order");
    for o in o {
        assert!(orders::issue(w, ALPHA, o));
    }
    tick::step(w);
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

fn mv(actor: ObjectId, dx: i32, dy: i32) -> Order {
    Order::Behaviour {
        actor,
        behaviour: Behaviour::Move,
        attack_move: false,
        target: None,
        dest: Some(Target::Cell(testkit::mid(SIZE, dx, dy))),
    }
}

fn contact(w: &World, id: ObjectId) -> Option<ObjectId> {
    w.obj(id).unwrap().contact
}

fn health(w: &World, id: ObjectId) -> i32 {
    w.obj(id).unwrap().health
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

fn docked(w: &World, id: ObjectId) -> bool {
    depot::tethered(w, id)
}

/// The docking cell of the plain depot at the middle of the map: the
/// foundation centre, cell (1, 1).
fn dock_cell() -> glasnost_map::CellXY {
    testkit::mid(SIZE, 1, 1)
}

fn sounds(w: &World, s: Sound) -> usize {
    w.effects
        .iter()
        .filter(|e| matches!(e, Effect::Sound { sound, .. } if *sound == s))
        .count()
}

/// Docks a damaged truck at a fresh depot by a click and runs until it is
/// docked and the depot is in its repair behaviour.
fn docked_scene(health_at: i32) -> (World, ObjectId, ObjectId) {
    let r = rules();
    let mut w = game(&r);
    let d = depot_at(&mut w, "DEPOT");
    let v = truck(&mut w, 0, 6, 1, health_at);
    open(&mut w);
    click_enter(&mut w, v, d);
    assert!(
        run_until(&mut w, 600, |w| docked(w, v)
            && w.obj(d).unwrap().beh.current == Some(Behaviour::Repair)
            && w.obj(v).unwrap().beh.current == Some(Behaviour::Sleep)),
        "the truck docks in sleep and the depot starts repair"
    );
    (w, d, v)
}

// spec: S-0314 claims 6, 16, 17 and 18; S-0215 claim 29; S-0214 table V row n3 (end to end: the click over an allied depot is enter; the truck takes contact, drives to the docking point, docks facing east, the depot repairs it step by step at the owner's cost and sends it away)
#[test]
fn end_to_end_click_repair_and_release() {
    let r = rules();
    let mut w = game(&r);
    let d = depot_at(&mut w, "DEPOT");
    let v = truck(&mut w, 0, 7, 1, 300);
    open(&mut w);
    assert_eq!(
        targeting::action(&w, ALPHA, v, ClickOn::Object(d), Modifiers::default()),
        Action::Enter
    );
    click_enter(&mut w, v, d);
    assert_eq!(contact(&w, v), Some(d), "contact before arrival");
    assert!(!docked(&w, v));
    let credits = w.houses[0].credits;
    assert!(run_until(&mut w, 600, |w| docked(w, v)
        && w.obj(d).unwrap().beh.current == Some(Behaviour::Repair)));
    assert_eq!(w.obj(v).unwrap().pos.cell(), dock_cell());
    assert!(run_until(&mut w, 100, |w| w.obj(v).unwrap().beh.current
        == Some(Behaviour::Sleep)));
    assert_eq!(
        w.obj(v).unwrap().body.value(w.tick),
        16384,
        "it docked facing east"
    );
    assert!(run_until(&mut w, 600, |w| health(w, v) > 300));
    assert!(run_until(&mut w, 3000, |w| contact(w, v).is_none()));
    assert_eq!(health(&w, v), 600, "repaired to full strength");
    // 30 steps of 10 health at 3 credits.
    assert_eq!(credits - w.houses[0].credits, 90);
    assert_eq!(sounds(&w, Sound::UnitRepaired), 1);
    assert_eq!(w.obj(v).unwrap().beh.current, Some(Behaviour::Move));
    assert!(
        w.obj(v).unwrap().dest.is_some(),
        "sent away to an exit cell"
    );
    assert!(run_until(&mut w, 600, |w| !depot::is_depot(w, v)
        && w.obj(v).unwrap().dest.is_none()));
    let c = w.obj(v).unwrap().pos.cell();
    let tl = testkit::mid(SIZE, 0, 0);
    assert!(
        !(tl.x..tl.x + 3).contains(&c.x) || !(tl.y..tl.y + 3).contains(&c.y),
        "it left the foundation"
    );
}

fn k_u(w: &World) -> i32 {
    w.rules().combat.general.repair_count_vehicle
}

fn d_rep(w: &World) -> i32 {
    w.rules().behaviour(Behaviour::Repair).rate_delay.max(1)
}

/// Runs until `id`'s health changes or `n` ticks pass; returns the tick of
/// the change.
fn next_step(w: &mut World, id: ObjectId, n: i32) -> Option<i32> {
    let h = health(w, id);
    for _ in 0..n {
        let t = w.tick;
        tick::step(w);
        if health(w, id) != h {
            return Some(t);
        }
    }
    None
}

fn lethal() -> Hit {
    Hit {
        amount: 100_000,
        warhead: None,
        distance: 0,
        attacker: None,
        house: None,
        ignore_defences: true,
        no_escape: true,
    }
}

// spec: S-0314 claim 6 and F1 (settling takes one step; the second falls due D + K_u − 1 ticks later, each later one K_u after the one before; done in working sends the vehicle away with the repair-complete announcement)
#[test]
fn claim_6_phases_and_timing() {
    let (mut w, _d, v) = docked_scene(560);
    let (k, dd) = (k_u(&w), d_rep(&w));
    let s1 = next_step(&mut w, v, 600).expect("settling step");
    assert_eq!(health(&w, v), 570);
    let s2 = next_step(&mut w, v, 600).expect("second step");
    assert_eq!(s2 - s1, dd + k - 1);
    assert_eq!(
        w.obj(v).unwrap().beh.current,
        Some(Behaviour::Sleep),
        "it sleeps while serviced"
    );
    let s3 = next_step(&mut w, v, 600).expect("third step");
    assert_eq!(s3 - s2, k);
    let s4 = next_step(&mut w, v, 600).expect("fourth step, done");
    assert_eq!(s4 - s3, k);
    assert_eq!(health(&w, v), 600);
    assert_eq!(contact(&w, v), None, "done in working: sent away");
    assert_eq!(sounds(&w, Sound::UnitRepaired), 1);
}

// spec: S-0314 claim 6 (a vehicle that the settling step brings to full health waits for one more step, which reports not needed, D + K_u − 1 ticks later, before it is sent away with the announcement)
#[test]
fn claim_6_done_at_settling_waits_one_step() {
    let (mut w, d, v) = docked_scene(595);
    let (k, dd) = (k_u(&w), d_rep(&w));
    let s1 = next_step(&mut w, v, 600).expect("settling step");
    assert_eq!(health(&w, v), 600);
    assert_eq!(contact(&w, v), Some(d));
    assert!(run_until(&mut w, 600, |w| contact(w, v).is_none()));
    assert_eq!(w.tick - 1 - s1, dd + k - 1);
    assert_eq!(sounds(&w, Sound::UnitRepaired), 1);
}

// spec: S-0314 claim 6 (settling, not needed: a full-health vehicle is sent away at once with no payment and no announcement)
#[test]
fn claim_6_full_health_vehicle_is_sent_away_unpaid() {
    let (mut w, _d, v) = docked_scene(600);
    let credits = w.houses[0].credits;
    assert!(run_until(&mut w, 600, |w| contact(w, v).is_none()));
    assert_eq!(w.houses[0].credits, credits);
    assert_eq!(sounds(&w, Sound::UnitRepaired), 0);
    assert!(w.obj(v).unwrap().dest.is_some());
}

// spec: S-0314 claim 6 (cannot pay in working: contact kept, back to settling; the depot owner at exactly 0 credits hears low funds; restored funds restart repair without an order)
#[test]
fn claim_6_cannot_pay_and_low_funds() {
    let (mut w, d, v) = docked_scene(300);
    // Two steps' worth: the settling step and one working step.
    w.houses[0].credits = 6;
    next_step(&mut w, v, 600).unwrap();
    next_step(&mut w, v, 600).unwrap();
    assert_eq!(w.houses[0].credits, 0);
    assert!(next_step(&mut w, v, 300).is_none(), "nothing more is paid");
    assert_eq!(contact(&w, v), Some(d), "contact kept");
    assert_eq!(
        sounds(&w, Sound::LowFunds),
        1,
        "heard once, on leaving working"
    );
    w.houses[0].credits = 1000;
    let dd = d_rep(&w);
    assert!(
        next_step(&mut w, v, dd + 2).is_some(),
        "repair restarts by itself"
    );
}

// spec: S-0314 claim 6 (a depot with a rally point sends the vehicle there)
#[test]
fn claim_6_sent_to_the_rally_point() {
    let (mut w, d, v) = docked_scene(590);
    let rally = Target::Cell(testkit::mid(SIZE, -5, 6));
    w.obj_mut(d).unwrap().guard_spot = Some(rally);
    assert!(run_until(&mut w, 1500, |w| contact(w, v).is_none()));
    assert_eq!(w.obj(v).unwrap().dest, Some(rally));
}

// spec: S-0314 claim 6 (a vehicle of the neutral or special house with a guard spot returns to it and the spot is cleared)
#[test]
fn claim_6_neutral_vehicle_returns_to_its_guard_spot() {
    let r = rules();
    let mut w = testkit::game_with_non_players(&r, SIZE, 5);
    for h in &mut w.houses {
        h.credits = 1000;
    }
    let neutral = w
        .houses
        .iter()
        .position(|h| h.role == HouseRole::Neutral)
        .unwrap() as u8;
    let d = spawn(&mut w, Kind::Building, "DEPOT", neutral, 0, 0);
    let v = truck(&mut w, neutral, 6, 1, 600);
    let spot = Target::Cell(testkit::mid(SIZE, 8, 8));
    open(&mut w);
    assert!(depot::request(&mut w, v, d, true));
    let t = w.tick;
    {
        let o = w.obj_mut(v).unwrap();
        o.beh.force(Behaviour::Enter, t, false);
        o.dest = Some(Target::Object(d));
        o.guard_spot = Some(spot);
    }
    assert!(run_until(&mut w, 1500, |w| docked(w, v)));
    assert!(run_until(&mut w, 1500, |w| contact(w, v).is_none()));
    assert_eq!(w.obj(v).unwrap().dest, Some(spot));
    assert_eq!(w.obj(v).unwrap().guard_spot, None);
}

// spec: S-0314 claim 16; S-0214 table V rows n1 and n3 (permission: allied owner, not selling, not switched off, not standing on the depot; no damage or money gate; zone fly gets no-enter)
#[test]
fn claim_16_permission() {
    let r = rules();
    let mut w = game(&r);
    let d = depot_at(&mut w, "DEPOT");
    let v = truck(&mut w, 0, 6, 1, 600);
    let act = |w: &World, v: ObjectId| {
        targeting::action(w, ALPHA, v, ClickOn::Object(d), Modifiers::default())
    };
    open(&mut w);
    w.houses[0].credits = 0;
    assert_eq!(
        act(&w, v),
        Action::Enter,
        "full health and no money still enter"
    );
    assert!(depot::accepts(&w, d, v, true));
    let hop = spawn(&mut w, Kind::Vehicle, "HOPPER", 0, 6, 3);
    assert_eq!(act(&w, hop), Action::NoEnter, "row n1: zone fly");
    let enemy = truck(&mut w, 1, 6, 5, 300);
    assert!(
        !depot::accepts(&w, d, enemy, true),
        "the depot's owner must count it allied"
    );
    w.obj_mut(d).unwrap().econ.building.switched_off = true;
    assert!(!depot::accepts(&w, d, v, true), "switched off");
    assert_ne!(act(&w, v), Action::Enter);
    w.obj_mut(d).unwrap().econ.building.switched_off = false;
    let on = truck(&mut w, 0, 1, 1, 300);
    assert!(
        !depot::accepts(&w, d, on, true),
        "standing on this very depot"
    );
    sell::sell(&mut w, ALPHA, d);
    assert!(!depot::accepts(&w, d, v, true), "selling");
}

// spec: S-0314 claim 16 and S-0504 claim 7 (the naval match: a naval yard refuses a land vehicle)
#[test]
fn claim_16_naval_match() {
    let mut r = rules();
    let mut yard = r.sim.buildings["YARD"].clone();
    yard.naval = Some(true);
    r.sim.buildings.insert("NAVALYARD".into(), yard);
    r.registries
        .get_mut("buildings")
        .unwrap()
        .push("NAVALYARD".into());
    let mut w = game(&r);
    let y = depot_at(&mut w, "NAVALYARD");
    let v = truck(&mut w, 0, 6, 1, 300);
    assert!(!depot::accepts(&w, y, v, true));
}

// spec: S-0314 claim 17 (contact in the first free place before arrival; at a held one-place depot a second click is not enter and does not displace the first)
#[test]
fn claim_17_contact_places() {
    let r = rules();
    let mut w = game(&r);
    let d = depot_at(&mut w, "DEPOT");
    let a = truck(&mut w, 0, 7, 1, 300);
    let b = truck(&mut w, 0, 7, 4, 300);
    open(&mut w);
    click_enter(&mut w, a, d);
    assert_eq!(contact(&w, a), Some(d));
    assert_eq!(depot::link(&w, a).unwrap().place, 0);
    assert_eq!(
        targeting::action(&w, ALPHA, b, ClickOn::Object(d), Modifiers::default()),
        Action::Select,
        "a held one-place depot: select, not enter"
    );
    assert_eq!(contact(&w, a), Some(d));
}

// spec: S-0314 claim 17 and Import mapping (max(d, 1) places: a two-place depot accepts a second vehicle into place 1)
#[test]
fn claim_17_two_places() {
    let r = rules();
    let mut w = game(&r);
    let d = depot_at(&mut w, "DEPOT2");
    let a = truck(&mut w, 0, 7, 1, 300);
    let b = truck(&mut w, 0, 7, 4, 300);
    open(&mut w);
    assert!(depot::request(&mut w, a, d, true));
    assert!(depot::accepts(&w, d, b, true));
    assert!(depot::request(&mut w, b, d, true));
    assert_eq!(depot::link(&w, a).unwrap().place, 0);
    assert_eq!(depot::link(&w, b).unwrap().place, 1);
    let c = truck(&mut w, 0, 7, 7, 300);
    assert!(!depot::accepts(&w, d, c, true), "no third place");
}

// spec: S-0314 claim 17 and S-0215 claim 29 (the earlier guard start: an unarmed depot with a contacted entering vehicle standing still strictly within 64 leptons of the foundation centre queues repair, 1 tick)
#[test]
fn claim_17_early_guard_start() {
    let r = rules();
    for (off, want) in [(0, Some(1)), (63, Some(1)), (64, None)] {
        let mut w = game(&r);
        let d = depot_at(&mut w, "DEPOT");
        let v = truck(&mut w, 0, 1, 1, 300);
        assert!(depot::request(&mut w, v, d, false));
        let t = w.tick;
        w.obj_mut(v).unwrap().beh.force(Behaviour::Enter, t, false);
        let p = w.obj(v).unwrap().pos;
        objects::place(&mut w, v, Pos::new(p.x + off, p.y, p.z), 0);
        assert_eq!(depot::early_start(&mut w, d), want, "{off} leptons off");
        let q = w.obj(d).unwrap().beh.queued;
        assert_eq!(q == Some(Behaviour::Repair), want.is_some());
    }
}

// spec: S-0314 claim 18 (docking point: the foundation centre with height plus the sole displacement at a count of 1, the assigned place's at a count above 1; a vehicle-factory flag selects S-0504 claim 12's point instead)
#[test]
fn claim_18_docking_point() {
    let mut r = rules();
    r.sim
        .buildings
        .get_mut("DEPOT")
        .unwrap()
        .econ
        .docking_offsets = Some(vec![[0, 256, 0]]);
    let mut w = game(&r);
    let d = depot_at(&mut w, "DEPOT");
    let d2 = spawn(&mut w, Kind::Building, "DEPOT2", 0, -8, -8);
    let y = spawn(&mut w, Kind::Building, "YARD", 0, 8, -8);
    let a = truck(&mut w, 0, 6, 1, 300);
    let b = truck(&mut w, 0, -2, -8, 300);
    let e = truck(&mut w, 0, -2, -4, 300);
    let c = w.reference_point(d);
    let dp = depot::docking_point(&w, d, a);
    assert_eq!((dp.x - c.x, dp.y - c.y, dp.z - c.z), (0, 256, 0));
    assert!(depot::request(&mut w, b, d2, false));
    assert!(depot::request(&mut w, e, d2, false));
    let c2 = w.reference_point(d2);
    assert_eq!(depot::docking_point(&w, d2, b).x - c2.x, -256, "place 0");
    assert_eq!(depot::docking_point(&w, d2, e).x - c2.x, 256, "place 1");
    let tl = w.obj(y).unwrap().pos.cell();
    let yp = depot::docking_point(&w, y, a);
    assert_eq!(yp.cell(), glasnost_map::CellXY::new(tl.x + 2, tl.y + 1));
    assert_eq!((yp.x % 256, yp.y % 256), (128, 128), "the cell's centre");
}

// spec: S-0314 claim 20 (an approaching vehicle given another order loses its contact at once; a repeated enter on the same depot while in enter is ignored)
#[test]
fn claim_20_approaching_vehicle() {
    let r = rules();
    let mut w = game(&r);
    let d = depot_at(&mut w, "DEPOT");
    let v = truck(&mut w, 0, 12, 1, 300);
    open(&mut w);
    click_enter(&mut w, v, d);
    tick::step(&mut w);
    assert_eq!(w.obj(v).unwrap().beh.current, Some(Behaviour::Enter));
    assert!(orders::issue(&mut w, ALPHA, enter(v, d)));
    tick::step(&mut w);
    assert_eq!(contact(&w, v), Some(d), "the repeat is ignored");
    assert!(orders::issue(&mut w, ALPHA, mv(v, 12, 6)));
    tick::step(&mut w);
    assert_eq!(contact(&w, v), None, "another order releases the place");
    assert!(depot::link(&w, v).is_none());
}

// spec: S-0314 claim 20 (a docked vehicle keeps contact and tether under a move order)
#[test]
fn claim_20_docked_vehicle_keeps_contact() {
    let (mut w, d, v) = docked_scene(300);
    assert!(orders::issue(&mut w, ALPHA, mv(v, 9, 9)));
    tick::step(&mut w);
    assert!(docked(&w, v), "a move order keeps the tether");
    assert_eq!(contact(&w, v), Some(d));
}

// spec: S-0314 claim 20 (an enter order on the same depot ends the docking: a damaged vehicle queues and the standing-on-depot refusal clears contact and queue next tick; at full health contact is lost at once)
#[test]
fn claim_20_same_depot_enter_while_docked() {
    let (mut w, d, v) = docked_scene(300);
    assert!(orders::issue(&mut w, ALPHA, enter(v, d)));
    tick::step(&mut w);
    tick::step(&mut w);
    assert_eq!(
        contact(&w, v),
        None,
        "queued, then refused while standing on it"
    );
    assert_eq!(w.obj(v).unwrap().depot.queued, None);
    let (mut w, d, v) = docked_scene(300);
    w.obj_mut(v).unwrap().health = 600;
    assert!(orders::issue(&mut w, ALPHA, enter(v, d)));
    tick::step(&mut w);
    assert_eq!(contact(&w, v), None, "full health: contact lost at once");
}

// spec: S-0314 claim 21 and Edge cases (quirks.depot_remote_repair on: a docked vehicle ordered away keeps receiving paid steps when it stands still far from the depot)
#[test]
fn claim_21_remote_repair() {
    let (mut w, _d, v) = docked_scene(300);
    next_step(&mut w, v, 600).expect("settling step");
    assert!(orders::issue(&mut w, ALPHA, mv(v, 10, 10)));
    assert!(run_until(&mut w, 900, |w| w.obj(v).unwrap().pos.cell()
        == testkit::mid(SIZE, 10, 10)
        && !glasnost_sim::occupancy::in_progress(w, v)));
    let h = health(&w, v);
    assert!(next_step(&mut w, v, 200).is_some());
    assert!(health(&w, v) > h, "paid steps far from the depot");
    assert!(docked(&w, v));
}

// spec: S-0314 Edge cases (quirks.depot_remote_repair off: a docked vehicle given a move order loses its contact)
#[test]
fn quirk_depot_remote_repair_off() {
    let mut r = rules();
    r.globals.quirks.depot_remote_repair = false;
    let mut w = game(&r);
    let d = depot_at(&mut w, "DEPOT");
    let v = truck(&mut w, 0, 6, 1, 300);
    open(&mut w);
    click_enter(&mut w, v, d);
    assert!(run_until(&mut w, 600, |w| docked(w, v)));
    assert!(orders::issue(&mut w, ALPHA, mv(v, 10, 10)));
    tick::step(&mut w);
    assert_eq!(contact(&w, v), None);
}

// spec: S-0314 claim 22 (sale: run-away notice, sleep to guard at once, contacts released at the next selling processing)
#[test]
fn claim_22_sale() {
    let (mut w, d, v) = docked_scene(300);
    assert!(run_until(&mut w, 100, |w| w.obj(v).unwrap().beh.current
        == Some(Behaviour::Sleep)));
    sell::sell(&mut w, ALPHA, d);
    assert_eq!(w.obj(v).unwrap().beh.current, Some(Behaviour::Guard));
    assert!(run_until(&mut w, 5, |w| contact(w, v).is_none()));
}

// spec: S-0314 claim 22 (destruction: an approaching vehicle beyond S-0312's kill distance is released without notice and falls back to guard)
#[test]
fn claim_22_destruction_approaching() {
    let r = rules();
    let mut w = game(&r);
    let d = depot_at(&mut w, "DEPOT");
    let v = truck(&mut w, 0, 12, 1, 300);
    open(&mut w);
    click_enter(&mut w, v, d);
    damage::apply(&mut w, d, lethal());
    tick::step(&mut w);
    assert_eq!(contact(&w, v), None);
    assert_eq!(w.obj(v).unwrap().beh.current, Some(Behaviour::Guard));
}

/// A docked vehicle carried off its depot (beyond S-0312 claim 17's kill
/// distance) while asleep, then the depot destroyed.
fn destroyed_while_docked(r: &Ruleset) -> (World, ObjectId) {
    let mut w = game(r);
    let d = depot_at(&mut w, "DEPOT");
    let v = truck(&mut w, 0, 6, 1, 300);
    open(&mut w);
    click_enter(&mut w, v, d);
    assert!(run_until(&mut w, 600, |w| w.obj(v).unwrap().beh.current
        == Some(Behaviour::Sleep)));
    let p = w.obj(v).unwrap().pos;
    objects::place(&mut w, v, Pos::new(p.x + 1024, p.y, p.z), 16384);
    // Placing a unit makes it fall back to rest; the test's carrying is not
    // an order, so that fall-back is undone.
    w.obj_mut(v).unwrap().beh.queued = None;
    damage::apply(&mut w, d, lethal());
    tick::step(&mut w);
    (w, v)
}

// spec: S-0314 claim 22 (a docked vehicle stays in sleep after its depot is destroyed, quirks.depot_loss_leaves_sleep on)
#[test]
fn claim_22_docked_vehicle_left_asleep() {
    let (w, v) = destroyed_while_docked(&rules());
    assert_eq!(contact(&w, v), None);
    assert_eq!(w.obj(v).unwrap().beh.current, Some(Behaviour::Sleep));
}

// spec: S-0314 Edge cases (quirks.depot_loss_leaves_sleep off: destruction treats contacts as a sale does: guard)
#[test]
fn quirk_depot_loss_leaves_sleep_off() {
    let mut r = rules();
    r.globals.quirks.depot_loss_leaves_sleep = false;
    let (w, v) = destroyed_while_docked(&r);
    assert_eq!(contact(&w, v), None);
    assert_eq!(w.obj(v).unwrap().beh.current, Some(Behaviour::Guard));
}

// spec: S-0314 claim 22 (a vehicle destroyed while in contact frees its place; a depot left without any contact returns to guard)
#[test]
fn claim_22_destroyed_vehicle_frees_its_place() {
    let (mut w, d, v) = docked_scene(300);
    damage::apply(&mut w, v, lethal());
    let other = truck(&mut w, 0, -4, -4, 300);
    assert!(depot::accepts(&w, d, other, true), "the place is free");
    assert!(run_until(&mut w, 100, |w| w.obj(d).unwrap().beh.current
        == Some(Behaviour::Guard)));
}

// spec: S-0314 claim 22 (power: a depot switched off keeps servicing a vehicle in contact; it refuses only new acceptance)
#[test]
fn claim_22_switched_off_keeps_servicing() {
    let (mut w, d, v) = docked_scene(300);
    w.obj_mut(d).unwrap().econ.building.switched_off = true;
    assert!(next_step(&mut w, v, 600).is_some());
    let other = truck(&mut w, 0, -4, -4, 300);
    assert!(!depot::accepts(&w, d, other, true));
}

// spec: S-0314 claims 21 and 22 (the settling step switches the locomotor off; a destination given while a depot occupies the cell switches it on)
#[test]
fn claim_22_locomotor_switched_on_by_a_destination() {
    let (mut w, _d, v) = docked_scene(300);
    next_step(&mut w, v, 600).expect("settling step");
    assert!(depot::locomotor_off(&w, v));
    assert!(orders::issue(&mut w, ALPHA, mv(v, 9, 9)));
    tick::step(&mut w);
    assert!(
        !depot::locomotor_off(&w, v),
        "a destination on the depot's cell"
    );
}

// spec: S-0314 claim 22 (quirks.depot_loss_strands_hover on: a sale after settling leaves the locomotor off, the seam S-0503's hovering locomotor reads)
#[test]
fn claim_22_sale_leaves_the_locomotor_off() {
    let (mut w, d, v) = docked_scene(300);
    next_step(&mut w, v, 600).expect("settling step");
    sell::sell(&mut w, ALPHA, d);
    assert!(run_until(&mut w, 5, |w| contact(w, v).is_none()));
    assert!(depot::locomotor_off(&w, v));
}

// spec: S-0314 Edge cases (quirks.depot_loss_strands_hover off: losing depot contact switches the locomotor on)
#[test]
fn quirk_depot_loss_strands_hover_off() {
    let mut r = rules();
    r.globals.quirks.depot_loss_strands_hover = false;
    let mut w = game(&r);
    let d = depot_at(&mut w, "DEPOT");
    let v = truck(&mut w, 0, 6, 1, 300);
    open(&mut w);
    click_enter(&mut w, v, d);
    assert!(run_until(&mut w, 900, |w| depot::locomotor_off(w, v)));
    sell::sell(&mut w, ALPHA, d);
    assert!(run_until(&mut w, 5, |w| contact(w, v).is_none()));
    assert!(!depot::locomotor_off(&w, v));
}

// spec: S-0314 claim 23 (enter orders in one tick: the first takes contact; a damaged later one queues with no destination and enters by itself when the place frees; a full-health later one does not queue)
#[test]
fn claim_23_queued_entry() {
    let r = rules();
    let mut w = game(&r);
    let d = depot_at(&mut w, "DEPOT");
    let a = truck(&mut w, 0, 6, 1, 590);
    let b = truck(&mut w, 0, 6, 4, 300);
    let c = truck(&mut w, 0, 6, 7, 600);
    open(&mut w);
    for o in [enter(a, d), enter(b, d), enter(c, d)] {
        assert!(orders::issue(&mut w, ALPHA, o));
    }
    tick::step(&mut w);
    assert_eq!(contact(&w, a), Some(d));
    assert_eq!(contact(&w, b), None);
    assert_eq!(w.obj(b).unwrap().depot.queued, Some(d));
    assert_eq!(w.obj(b).unwrap().dest, None);
    assert_eq!(
        w.obj(c).unwrap().depot.queued,
        None,
        "full health does not queue"
    );
    assert_eq!(w.obj(c).unwrap().dest, None);
    assert!(run_until(&mut w, 3000, |w| contact(w, b) == Some(d)));
    assert_eq!(contact(&w, a), None);
    assert!(run_until(&mut w, 2, |w| w.obj(b).unwrap().beh.current
        == Some(Behaviour::Enter)));
    assert!(run_until(&mut w, 6000, |w| health(w, b) == 600));
}

// spec: S-0314 claims 20 and 23 (any later order clears a queued entry)
#[test]
fn claim_23_later_order_clears_queued_entry() {
    let r = rules();
    let mut w = game(&r);
    let d = depot_at(&mut w, "DEPOT");
    let a = truck(&mut w, 0, 6, 1, 590);
    let b = truck(&mut w, 0, 6, 4, 300);
    open(&mut w);
    for o in [enter(a, d), enter(b, d)] {
        assert!(orders::issue(&mut w, ALPHA, o));
    }
    tick::step(&mut w);
    assert_eq!(w.obj(b).unwrap().depot.queued, Some(d));
    assert!(orders::issue(&mut w, ALPHA, mv(b, 9, 9)));
    tick::step(&mut w);
    assert_eq!(w.obj(b).unwrap().depot.queued, None);
}

// spec: S-0314 claim 24 (no automatic return: a damaged vehicle next to a depot is not sent to it)
#[test]
fn claim_24_no_automatic_return() {
    let r = rules();
    let mut w = game(&r);
    let _d = depot_at(&mut w, "DEPOT");
    let v = truck(&mut w, 0, 5, 1, 100);
    testkit::run(&mut w, 1500);
    assert_eq!(contact(&w, v), None);
    assert_eq!(w.obj(v).unwrap().pos.cell(), testkit::mid(SIZE, 5, 1));
    assert_eq!(health(&w, v), 100);
}

// spec: S-0215 claim 29 (without a tether a non-accepting reply ends contact and the vehicle falls back to rest)
#[test]
fn s0215_claim_29_refusal_while_approaching() {
    let r = rules();
    let mut w = game(&r);
    let d = depot_at(&mut w, "DEPOT");
    let v = truck(&mut w, 0, 12, 1, 300);
    open(&mut w);
    click_enter(&mut w, v, d);
    w.obj_mut(d).unwrap().econ.building.switched_off = true;
    assert!(run_until(&mut w, 100, |w| contact(w, v).is_none()));
    assert!(run_until(&mut w, 300, |w| w.obj(v).unwrap().beh.current
        != Some(Behaviour::Enter)));
}

// spec: S-0314 claims 6 and 18 and S-0504 claims 12 and 14 (a vehicle factory with the repair flag docks at its fixed point and repairs; the vehicle is then sent away)
#[test]
fn s0504_claim_14_yard_repair() {
    let r = rules();
    let mut w = game(&r);
    let y = depot_at(&mut w, "YARD");
    let v = truck(&mut w, 0, 7, 1, 500);
    open(&mut w);
    click_enter(&mut w, v, y);
    assert!(run_until(&mut w, 900, |w| w.obj(v).unwrap().beh.current
        == Some(Behaviour::Sleep)));
    assert_eq!(w.obj(v).unwrap().pos.cell(), testkit::mid(SIZE, 2, 1));
    assert!(run_until(&mut w, 3000, |w| health(w, v) == 600));
    assert!(run_until(&mut w, 600, |w| contact(w, v).is_none()));
}

// spec: S-0314 claims 6 and 17 (depot state survives a save and load: the hash round-trips mid-repair and both runs agree)
#[test]
fn depot_state_round_trips() {
    let (mut w, _d, v) = docked_scene(300);
    next_step(&mut w, v, 600).expect("settling step");
    let bytes = save::save(&w);
    let mut l = save::load(&bytes, w.rules_arc(), testkit::catalogue()).expect("loads");
    assert_eq!(save::state_hash(&l), save::state_hash(&w));
    for _ in 0..200 {
        tick::step(&mut w);
        tick::step(&mut l);
    }
    assert_eq!(save::state_hash(&l), save::state_hash(&w));
    assert_eq!(health(&l, v), health(&w, v));
}
