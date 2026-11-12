//! Claim tests of the depot cycle: S-0314 claims 6 and 16 to 24, S-0504
//! claim 14 and S-0215 claim 29 (repair depots and repairing naval yards)
//! on synthetic data: invented types, rates and costs, no game data.

use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{BuildingType, LocomotionClass, Locomotor, MovementZone, UnitType};
use glasnost_sim::combat::damage::{self, Hit};
use glasnost_sim::combat::effects::{Effect, Sound};
use glasnost_sim::drive::Track;
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
/// - WEED: DEPOT with the obsolete-unloading flag;
/// - PAD: DEPOT with the pad flag (claim 17's tether exchange);
/// - PAD2: PAD with DEPOT2's two places and displacements;
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
    push("buildings", "WEED");
    push("buildings", "PAD");
    push("buildings", "PAD2");
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
    let mut weed = depot.clone();
    weed.econ.weeder = Some(true);
    s.buildings.insert("DEPOT".into(), depot);
    s.buildings.insert("DEPOT2".into(), two);
    s.buildings.insert("YARD".into(), yard);
    s.buildings.insert("WEED".into(), weed);
    let mut pad = s.buildings["DEPOT"].clone();
    pad.helipad = Some(true);
    let mut pad2 = s.buildings["DEPOT2"].clone();
    pad2.helipad = Some(true);
    s.buildings.insert("PAD".into(), pad);
    s.buildings.insert("PAD2".into(), pad2);
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

/// Sends a damaged truck to a fresh depot of type `name` by a click and
/// runs until it is in contact asleep and the depot is in its repair
/// behaviour; `tether` asks that it is tethered too.
fn scene_at(r: &Ruleset, name: &str, health_at: i32, tether: bool) -> (World, ObjectId, ObjectId) {
    let mut w = game(r);
    let d = depot_at(&mut w, name);
    let v = truck(&mut w, 0, 6, 1, health_at);
    open(&mut w);
    click_enter(&mut w, v, d);
    assert!(
        run_until(&mut w, 600, |w| contact(w, v) == Some(d)
            && (!tether || docked(w, v))
            && w.obj(d).unwrap().beh.current == Some(Behaviour::Repair)
            && w.obj(v).unwrap().beh.current == Some(Behaviour::Sleep)),
        "the truck arrives in sleep and the depot starts repair"
    );
    (w, d, v)
}

/// An ordinary depot (no pad flag): the truck is serviced untethered
/// (claim 17).
fn docked_scene(health_at: i32) -> (World, ObjectId, ObjectId) {
    let (w, d, v) = scene_at(&rules(), "DEPOT", health_at, false);
    assert!(!docked(&w, v), "no tether at an ordinary depot");
    (w, d, v)
}

/// [`docked_scene`] with a truck that is at full health after its arrival,
/// so that the settling step finds it *not needed* (claim 6): one at full
/// health when it is ordered is refused before it arrives (claim 16,
/// revision 107).
fn full_scene() -> (World, ObjectId, ObjectId) {
    let (mut w, d, v) = docked_scene(300);
    w.obj_mut(v).unwrap().health = 600;
    (w, d, v)
}

/// Marks the truck's contact as tethered (docked): the state claim 17's
/// pad exchange gives, set directly, since a straight approach to a
/// one-place pad depot never meets that exchange's conditions.
fn tether(w: &mut World, v: ObjectId) {
    w.obj_mut(v).unwrap().depot.link.as_mut().unwrap().tethered = true;
    assert!(docked(w, v));
}

/// A docked truck: [`docked_scene`] with the tether set.
fn pad_scene(health_at: i32) -> (World, ObjectId, ObjectId) {
    let (mut w, d, v) = docked_scene(health_at);
    tether(&mut w, v);
    (w, d, v)
}

// spec: S-0314 claims 6, 16, 17 and 18; S-0215 claim 29; S-0214 table V row n3 (end to end: the click over an allied depot is enter; the truck takes contact, drives to the docking point, where its movement arrival queues repair and sleep with no tether, the depot repairs it untethered step by step at the owner's cost and sends it away)
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
    assert!(run_until(&mut w, 600, |w| w.obj(d).unwrap().beh.current
        == Some(Behaviour::Repair)));
    assert_eq!(w.obj(v).unwrap().pos.cell(), dock_cell());
    assert!(run_until(&mut w, 100, |w| w.obj(v).unwrap().beh.current
        == Some(Behaviour::Sleep)));
    assert!(!docked(&w, v), "no tether at an ordinary depot");
    assert!(run_until(&mut w, 600, |w| health(w, v) > 300));
    assert!(!docked(&w, v), "paid repair needs no tether");
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

// spec: S-0314 claim 17 and S-0524 claim 14 (ordinary vehicle service docking raises neither an entered-by notification on the depot's tag nor a destroyed-by-anything notification on the vehicle's tag for the entry; departure adds none; C16)
#[test]
fn claim_17_service_docking_raises_no_tag() {
    let r = rules();
    let mut w = testkit::game_with_triggers(&r, SIZE, 5, testkit::entry_triggers());
    for h in &mut w.houses {
        h.credits = 10_000;
    }
    let d = depot_at(&mut w, "DEPOT");
    let v = truck(&mut w, 0, 7, 1, 300);
    objects_tag(&mut w, d, "ENT");
    objects_tag(&mut w, v, "DIE");
    open(&mut w);
    let f0 = w.triggers.fired;
    click_enter(&mut w, v, d);
    assert!(run_until(&mut w, 3000, |w| health(w, v) == 600
        && contact(w, v).is_none()));
    assert!(run_until(&mut w, 600, |w| w.obj(v).unwrap().dest.is_none()));
    assert_eq!(w.triggers.fired, f0, "no raise reached a firing");
    assert!(!w.triggers.variables[0] && !w.triggers.variables[1]);
    assert!(w.obj(v).is_some(), "docking does not consume the vehicle");
}

fn objects_tag(w: &mut World, id: ObjectId, name: &str) {
    glasnost_sim::triggers::object_tag(w, id, Some(name.as_bytes()));
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

// spec: S-0314 claim 6 (settling, not needed: a vehicle at full health by the settling step is sent away at once with no payment and no announcement); claim 16 (revision 107: one at full health when it is ordered is refused before it arrives, so the health is set after the arrival)
#[test]
fn claim_6_full_health_vehicle_is_sent_away_unpaid() {
    let (mut w, _d, v) = docked_scene(300);
    w.obj_mut(v).unwrap().health = 600;
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

// spec: S-0314 claim 25 (without a rally point the departure cell is the first candidate of the 3 × 3 foundation exit list that gives free; an occupied candidate is skipped; Q-0682 answered)
#[test]
fn claim_25_departure_cell_from_the_exit_list() {
    let list = glasnost_sim::econ::release::exit_list(3, 3, false, true);
    let (mut w, _d, v) = full_scene();
    let first = testkit::mid(SIZE, list[0].0, list[0].1);
    let second = testkit::mid(SIZE, list[1].0, list[1].1);
    spawn(&mut w, Kind::Vehicle, "TRUCK", 0, list[0].0, list[0].1);
    assert!(run_until(&mut w, 600, |w| contact(w, v).is_none()));
    assert_ne!(first, second);
    assert_eq!(w.obj(v).unwrap().dest, Some(Target::Cell(second)));
}

// spec: S-0314 claims 6 and 25 (with every exit-list cell occupied and no rally point there is no cell: no departure is ordered and contact is retained, the vehicle stays in its sleep; freeing a cell lets a later retry send it away)
#[test]
fn claim_25_no_departure_cell_keeps_contact() {
    let list = glasnost_sim::econ::release::exit_list(3, 3, false, true);
    let (mut w, d, v) = full_scene();
    let mut blockers = Vec::new();
    for &(a, b) in &list {
        if w.cells.footprint().is_legal(testkit::mid(SIZE, a, b))
            && !blockers
                .iter()
                .any(|&x: &ObjectId| w.obj(x).unwrap().pos.cell() == testkit::mid(SIZE, a, b))
        {
            blockers.push(spawn(&mut w, Kind::Vehicle, "TRUCK", 0, a, b));
        }
    }
    for _ in 0..300 {
        tick::step(&mut w);
    }
    assert_eq!(contact(&w, v), Some(d), "contact retained");
    // spec: S-0314 claim 21 (the processing that takes the settling step empties the order-level destination); claims 6 and 25 (no departure cell was ordered)
    assert_eq!(w.obj(v).unwrap().dest, None, "no departure ordered");
    assert_eq!(sounds(&w, Sound::UnitRepaired), 0);
    objects::remove(&mut w, blockers[0]);
    let dd = d_rep(&w);
    assert!(run_until(&mut w, dd + 2, |w| contact(w, v).is_none()));
    assert_eq!(
        w.obj(v).unwrap().dest,
        Some(Target::Cell(testkit::mid(SIZE, list[0].0, list[0].1)))
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
    // Damaged until the depot accepts it (claim 16, revision 107: at full health its repeated request is refused), then at full health, so the settling step sends it away.
    let v = truck(&mut w, neutral, 6, 1, 300);
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
    assert!(run_until(&mut w, 1500, |w| w.obj(d).unwrap().beh.current
        == Some(Behaviour::Repair)));
    w.obj_mut(v).unwrap().health = 600;
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

// spec: S-0314 claim 18 (docking point: the foundation centre with height plus the sole displacement at a count of 1, the assigned place's at a count above 1; the obsolete-unloading flag selects the fixed centre of the cell (2, 1) instead; the vehicle-factory flag does not)
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
    // The vehicle-factory flag: the foundation centre (no displacement).
    assert_eq!(depot::docking_point(&w, y, a), w.reference_point(y));
    let k = spawn(&mut w, Kind::Building, "WEED", 0, 8, 6);
    let tl = w.obj(k).unwrap().pos.cell();
    let kp = depot::docking_point(&w, k, a);
    assert_eq!(kp.cell(), glasnost_map::CellXY::new(tl.x + 2, tl.y + 1));
    assert_eq!((kp.x % 256, kp.y % 256), (128, 128), "the cell's centre");
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
    let (mut w, d, v) = pad_scene(300);
    assert!(orders::issue(&mut w, ALPHA, mv(v, 9, 9)));
    tick::step(&mut w);
    assert!(docked(&w, v), "a move order keeps the tether");
    assert_eq!(contact(&w, v), Some(d));
}

// spec: S-0314 claim 20 and table (revision 28, Q-1158 answered: a docked damaged vehicle standing on the depot given an enter order on that depot takes claim 23's queued entry in full: enter queued, the depot remembered, no destination, contact and tether kept; enter is current at its next first start and its processing queues nothing; the retry later in that turn is refused for standing on the depot and clears contact, queued entry and destination, queueing no behaviour; enter falls back to rest at its next processing. At full health the contact is lost at once)
#[test]
fn claim_20_same_depot_enter_while_docked() {
    let (mut w, d, v) = pad_scene(300);
    assert!(orders::issue(&mut w, ALPHA, enter(v, d)));
    let dq = w.obj(d).unwrap().beh.queued;
    tick::step(&mut w);
    // The order's tick (orders apply before the turns of that tick).
    let o = w.obj(v).unwrap();
    let after_order = (o.beh.current, o.beh.queued, contact(&w, v));
    // Either the order's tick already made enter current, or it is still
    // queued with the contact held, the depot remembered and no destination.
    if after_order.0 != Some(Behaviour::Enter) {
        assert_eq!(o.beh.queued, Some(Behaviour::Enter));
        assert_eq!(contact(&w, v), Some(d), "the tether keeps the contact");
        assert_eq!(o.depot.queued, Some(d));
        assert_eq!(o.depot.entering, Some(d));
        assert_eq!(o.dest, None);
        tick::step(&mut w);
    }
    // The first turn with enter current: contact lost in that turn.
    let o = w.obj(v).unwrap();
    assert_eq!(o.beh.current, Some(Behaviour::Enter), "enter current");
    assert_eq!(contact(&w, v), None, "refused while standing on it");
    assert_eq!(o.depot.queued, None);
    assert_eq!(o.dest, None, "no destination");
    assert_ne!(o.beh.queued, Some(Behaviour::Sleep), "no sleep queued");
    assert_eq!(
        w.obj(d).unwrap().beh.queued,
        dq,
        "the depot queues no new repair"
    );
    // Enter falls back to rest at its next processing.
    assert!(run_until(&mut w, 60, |w| w.obj(v).unwrap().beh.current
        != Some(Behaviour::Enter)));
    assert_ne!(w.obj(v).unwrap().beh.current, Some(Behaviour::Sleep));
    // spec: S-0314 claim 20 (revision 107: released by the order itself, which takes claim 23's full-health case: the order gives no destination and no queued entry, so the order's tick end does not show the contact, whatever the pad flag)
    for tethered in [true, false] {
        let (mut w, d, v) = if tethered {
            pad_scene(300)
        } else {
            docked_scene(300)
        };
        w.obj_mut(v).unwrap().health = 600;
        let dest = w.obj(v).unwrap().dest;
        assert!(orders::issue(&mut w, ALPHA, enter(v, d)));
        tick::step(&mut w);
        assert_eq!(contact(&w, v), None, "full health: contact lost at once");
        let o = w.obj(v).unwrap();
        assert_eq!(o.dest, dest, "full health: the order gives no destination");
        assert_eq!(o.depot.queued, None, "full health: no queued entry");
        assert!(depot::link(&w, v).is_none(), "tethered {tethered}");
    }
}

// spec: S-0314 claim 21 and Edge cases (quirks.depot_remote_repair on: a docked vehicle ordered away keeps receiving paid steps when it stands still far from the depot)
#[test]
fn claim_21_remote_repair() {
    let (mut w, _d, v) = pad_scene(300);
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

// spec: S-0314 claim 21 (settling at 200 leptons or more from the docking point: a moving vehicle takes no step and stops being settled, checked again after D ticks; once it stands still there it takes the settling step, and no processing there makes it settled; Q-0681 answered)
#[test]
fn claim_21_distant_settling() {
    let (mut w, d, v) = pad_scene(300);
    // The settling step and one working step, then cannot pay: back to settling.
    w.houses[0].credits = 6;
    next_step(&mut w, v, 600).unwrap();
    next_step(&mut w, v, 600).unwrap();
    assert!(next_step(&mut w, v, 100).is_none());
    assert!(orders::issue(&mut w, ALPHA, mv(v, 10, 10)));
    assert!(run_until(&mut w, 300, |w| w.obj(v).unwrap().pos.cell().x
        >= testkit::mid(SIZE, 4, 0).x));
    w.obj_mut(v).unwrap().depot.settled = depot::Settled::AtDepot;
    let dd = d_rep(&w);
    assert!(
        run_until(&mut w, dd + 1, |w| !depot::settled(w, v)),
        "the distant moving check ends being settled"
    );
    assert_eq!(contact(&w, v), Some(d), "settling and contact kept");
    assert!(run_until(&mut w, 900, |w| w.obj(v).unwrap().pos.cell()
        == testkit::mid(SIZE, 10, 10)
        && !glasnost_sim::occupancy::in_progress(w, v)));
    w.houses[0].credits = 1000;
    let h = health(&w, v);
    assert!(
        next_step(&mut w, v, dd + 2).is_some(),
        "the settling step far away"
    );
    assert!(health(&w, v) > h);
    assert!(
        !depot::settled(&w, v),
        "no processing at 200 leptons or more makes it settled"
    );
}

// spec: S-0314 Edge cases (quirks.depot_remote_repair off: a docked vehicle given a move order loses its contact)
#[test]
fn quirk_depot_remote_repair_off() {
    let mut r = rules();
    r.globals.quirks.depot_remote_repair = false;
    let (mut w, _d, v) = scene_at(&r, "DEPOT", 300, false);
    tether(&mut w, v);
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

/// A vehicle carried off its depot (beyond S-0312 claim 17's kill
/// distance) after its movement arrival, asleep, docked when `tethered`,
/// then the depot destroyed.
fn destroyed_while_docked(r: &Ruleset) -> (World, ObjectId) {
    destroyed_after_arrival(r, true, false)
}

/// [`destroyed_while_docked`], tethered or not, and with the vehicle put in
/// *guard* before the destruction when `guard`, as the measured driving
/// vehicle of S-0314 claim 17 is.
fn destroyed_after_arrival(r: &Ruleset, tethered: bool, guard: bool) -> (World, ObjectId) {
    let (mut w, d, v) = scene_at(r, "DEPOT", 300, false);
    assert!(
        depot::link(&w, v).unwrap().arrived,
        "the movement arrival is kept"
    );
    if tethered {
        tether(&mut w, v);
    }
    if guard {
        let t = w.tick;
        w.obj_mut(v)
            .unwrap()
            .beh
            .queue_and_start_at_once(Behaviour::Guard, t, false);
        assert_eq!(w.obj(v).unwrap().beh.current, Some(Behaviour::Guard));
    }
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

// spec: S-0314 Edge cases (quirks.depot_loss_leaves_sleep off: destruction treats contacts as a sale does: guard; revision 107: for every arrived vehicle, docked or not)
#[test]
fn quirk_depot_loss_leaves_sleep_off() {
    let mut r = rules();
    r.globals.quirks.depot_loss_leaves_sleep = false;
    let (w, v) = destroyed_while_docked(&r);
    assert_eq!(contact(&w, v), None);
    assert_eq!(w.obj(v).unwrap().beh.current, Some(Behaviour::Guard));
    let (w, v) = destroyed_after_arrival(&r, false, false);
    assert_eq!(contact(&w, v), None);
    assert_eq!(
        w.obj(v).unwrap().beh.current,
        Some(Behaviour::Guard),
        "untethered"
    );
}

// spec: S-0314 claim 22 (revision 107: the release leaves the current behaviour of a contacted vehicle after its movement arrival in the docking-point cell as it was, docked or not: one in sleep stays in sleep, one in guard stays in guard); Edge cases (quirks.depot_loss_leaves_sleep on)
#[test]
fn claim_22_arrived_vehicle_keeps_its_behaviour() {
    let r = rules();
    let (w, v) = destroyed_after_arrival(&r, false, false);
    assert_eq!(contact(&w, v), None);
    assert!(depot::link(&w, v).is_none());
    assert_eq!(
        w.obj(v).unwrap().beh.current,
        Some(Behaviour::Sleep),
        "untethered, asleep"
    );
    for tethered in [false, true] {
        let (w, v) = destroyed_after_arrival(&r, tethered, true);
        assert_eq!(contact(&w, v), None);
        assert_eq!(
            w.obj(v).unwrap().beh.current,
            Some(Behaviour::Guard),
            "in guard, tethered {tethered}"
        );
    }
}

// spec: S-0314 claim 22 (revision 107: a contacted vehicle before its movement arrival in the docking-point cell loses its target and falls back to guard where it stands, the tether playing no part; the arrival mark is not set before it)
#[test]
fn claim_22_destruction_before_arrival_tethered() {
    for quirk in [true, false] {
        let mut r = rules();
        r.globals.quirks.depot_loss_leaves_sleep = quirk;
        let mut w = game(&r);
        let d = depot_at(&mut w, "DEPOT");
        let v = truck(&mut w, 0, 12, 1, 300);
        open(&mut w);
        click_enter(&mut w, v, d);
        tether(&mut w, v);
        assert!(!depot::link(&w, v).unwrap().arrived, "not yet arrived");
        damage::apply(&mut w, d, lethal());
        tick::step(&mut w);
        assert_eq!(contact(&w, v), None);
        let o = w.obj(v).unwrap();
        assert_eq!(o.beh.current, Some(Behaviour::Guard), "quirk {quirk}");
        assert_eq!(o.dest, None, "quirk {quirk}");
    }
}

// spec: S-0314 claims 16 and 17 (revision 107: each repeated entry request to a repair depot also asks the vehicle whether it needs repair; a vehicle at full health answers no and the depot refuses, so one without a tether loses its contact and falls back to rest at its first enter processing after the order, before it reaches the docking point: no settling processing, no step and no payment, however often it is ordered; a naval yard with the repair flag is a repair depot too); S-0215 claim 29
#[test]
fn claim_16_full_health_refused_at_the_repeated_request() {
    for name in ["DEPOT", "YARD", "PAD"] {
        let r = rules();
        let mut w = game(&r);
        let d = depot_at(&mut w, name);
        let v = truck(&mut w, 0, 6, 1, 600);
        open(&mut w);
        let credits = w.houses[0].credits;
        for n in 0..3 {
            assert!(
                orders::issue(&mut w, ALPHA, enter(v, d)),
                "{name}: order {n}"
            );
            tick::step(&mut w);
            // The order's tick end: the order took effect (enter queued or current, or the first acceptance's contact).
            let b = &w.obj(v).unwrap().beh;
            assert!(
                b.current == Some(Behaviour::Enter)
                    || b.queued == Some(Behaviour::Enter)
                    || contact(&w, v).is_some(),
                "{name}: the order took effect, order {n}"
            );
            assert!(
                run_until(&mut w, 3, |w| contact(w, v).is_none()
                    && w.obj(v).unwrap().beh.current == Some(Behaviour::Enter)),
                "{name}: refused at its first enter processing, order {n}"
            );
            assert!(depot::link(&w, v).is_none(), "{name}");
            assert_ne!(
                w.obj(v).unwrap().pos.cell(),
                dock_cell(),
                "{name}: before the docking point"
            );
            // Enter falls back to rest.
            assert!(
                run_until(&mut w, 60, |w| w.obj(v).unwrap().beh.current
                    != Some(Behaviour::Enter)),
                "{name}: falls back, order {n}"
            );
        }
        for _ in 0..200 {
            tick::step(&mut w);
            assert_eq!(contact(&w, v), None, "{name}");
            assert_ne!(
                w.obj(d).unwrap().beh.current,
                Some(Behaviour::Repair),
                "{name}: no settling processing"
            );
        }
        assert_eq!(w.houses[0].credits, credits, "{name}: no payment");
        assert_eq!(sounds(&w, Sound::UnitRepaired), 0, "{name}");
        // Damaged, the same vehicle is accepted at its repeated request.
        w.obj_mut(v).unwrap().health = 300;
        assert!(orders::issue(&mut w, ALPHA, enter(v, d)));
        assert!(run_until(&mut w, 600, |w| w.obj(d).unwrap().beh.current
            == Some(Behaviour::Repair)));
    }
}

// spec: S-0314 claim 16 (revision 107: the depot's permission does not require damage, and its first acceptance makes no damage test) and claim 23 (an enter order on a depot asks for entry at once)
#[test]
fn claim_16_first_acceptance_takes_a_full_health_vehicle() {
    for name in ["DEPOT", "YARD", "PAD"] {
        let r = rules();
        let mut w = game(&r);
        let d = depot_at(&mut w, name);
        let v = truck(&mut w, 0, 6, 1, 600);
        open(&mut w);
        assert!(
            depot::accepts(&w, d, v, true),
            "{name}: permitted at full health"
        );
        depot::order_entered(&mut w, v, Behaviour::Enter, Some(Target::Object(d)));
        assert_eq!(
            depot::link(&w, v).map(|l| l.depot),
            Some(d),
            "{name}: the first acceptance takes the contact"
        );
        assert_eq!(contact(&w, v), Some(d), "{name}");
    }
}

// spec: S-0314 claim 16 (revision 107: a tethered vehicle keeps its contact on the full-health refusal, S-0215 claim 29: an existing tether allows entry to continue; without a tether the refusal ends the contact)
#[test]
fn claim_16_full_health_refusal_and_the_tether() {
    let (mut w, d, v) = pad_scene(300);
    w.obj_mut(v).unwrap().health = 600;
    let _ = depot::process_enter(&mut w, v);
    assert_eq!(contact(&w, v), Some(d), "tethered: contact kept");
    assert!(docked(&w, v));
    let (mut w, _d, v) = docked_scene(300);
    w.obj_mut(v).unwrap().health = 600;
    let _ = depot::process_enter(&mut w, v);
    assert_eq!(contact(&w, v), None, "untethered: contact lost");
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

/// The depot's (behaviour, phase) after each tick, from now until it is
/// back in *guard* (at most `n` ticks).
fn trace_to_guard(w: &mut World, d: ObjectId, n: i32) -> Vec<(Option<Behaviour>, i32)> {
    let mut out = Vec::new();
    for _ in 0..n {
        tick::step(w);
        let b = &w.obj(d).unwrap().beh;
        out.push((b.current, b.progress));
        if b.current == Some(Behaviour::Guard) {
            break;
        }
    }
    out
}

// spec: S-0314 claim 22 (revision 31: a depot left without any contact reacts at its next repair processing: in waiting it returns to guard without an animation wait; in working it enters settling and rechecks after 1 tick; in settling it returns to guard once special-3 no longer runs, at once for a depot type with no slot names, whose special-3 never runs, S-0108 claim 20)
// spec: S-0314 state table (waiting, working and settling with no contact)
#[test]
fn claim_22_contact_loss_by_phase() {
    for phase in 0..3 {
        let (mut w, d, v) = docked_scene(300);
        let dd = d_rep(&w);
        if phase == 0 {
            // The truck arrives close enough that the first processing
            // already passed the waiting check; put the depot back in
            // waiting, as for a vehicle that came to rest out of range.
            w.obj_mut(d).unwrap().beh.progress = 0;
        }
        assert!(
            run_until(&mut w, 600, |w| {
                let b = &w.obj(d).unwrap().beh;
                b.current == Some(Behaviour::Repair) && b.progress == phase
            }),
            "phase {phase} reached"
        );
        damage::apply(&mut w, v, lethal());
        assert!(w.obj(v).is_none());
        let trace = trace_to_guard(&mut w, d, 4 * dd + 20);
        let (last, before) = trace.split_last().expect("a trace");
        assert_eq!(last.0, Some(Behaviour::Guard), "phase {phase}: {trace:?}");
        assert!(
            before.iter().all(|&(b, _)| b == Some(Behaviour::Repair)),
            "phase {phase}: in repair until the return: {trace:?}"
        );
        match phase {
            // Waiting: the processing that finds no contact sets the next
            // one 1 tick later, and the depot is in guard from that tick
            // (revision 107, claim 6's table), so its last tick in repair is
            // that processing's.
            0 => {
                let (wait, rest) = before.split_last().expect("a waiting processing");
                assert!(rest.iter().all(|&(_, p)| p == 0), "phase 0: {trace:?}");
                assert_eq!(wait.0, Some(Behaviour::Repair), "phase 0: {trace:?}");
            }
            // Settling never passes through another phase.
            1 => assert!(
                before.iter().all(|&(_, p)| p == phase),
                "phase {phase}: {trace:?}"
            ),
            // Working: settling for exactly one tick, then guard.
            _ => {
                let settled = before.iter().position(|&(_, p)| p == 1);
                assert_eq!(
                    settled,
                    Some(before.len() - 1),
                    "working enters settling, then guard 1 tick later: {trace:?}"
                );
            }
        }
        // The serviced place is free for the next vehicle.
        let other = truck(&mut w, 0, -4, -4, 300);
        assert!(depot::accepts(&w, d, other, true));
    }
}

// spec: S-0314 claims 6 and 22 and S-0108 claim 20 (a depot type whose art names no slot animation runs none of the modelled slots, so neither hold applies)
#[test]
fn no_slot_names_no_running_slot() {
    let (w, d, _) = docked_scene(300);
    for s in [depot::DepotSlot::Production, depot::DepotSlot::Special3] {
        assert!(!depot::slot_running(&w, d, s));
    }
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

// spec: S-0314 Terms and claims 21 and 22 (the vehicle is settled at its settling step; a destination received while a depot occupies the cell ends its being settled)
#[test]
fn claim_22_locomotor_switched_on_by_a_destination() {
    let (mut w, _d, v) = docked_scene(300);
    next_step(&mut w, v, 600).expect("settling step");
    assert!(depot::settled(&w, v));
    assert!(orders::issue(&mut w, ALPHA, mv(v, 9, 9)));
    tick::step(&mut w);
    assert!(!depot::settled(&w, v), "a destination on the depot's cell");
}

// spec: S-0314 claim 22 (a sale: the depot still occupies its cells until its removal, so the destination of the scatter request the run-away notice brings ends a settled vehicle's being settled; S-0207 claim 16 accepts it for a driving vehicle, whose locomotor is never switched off)
#[test]
fn claim_22_sale_scatter_ends_being_settled() {
    let (mut w, d, v) = docked_scene(300);
    next_step(&mut w, v, 600).expect("settling step");
    assert!(depot::settled(&w, v));
    assert_eq!(w.obj(v).unwrap().dest, None);
    sell::sell(&mut w, ALPHA, d);
    assert!(
        w.obj(v).unwrap().dest.is_some(),
        "the scatter request's destination"
    );
    assert!(!depot::settled(&w, v));
}

// spec: S-0314 claim 22 and Edge cases (quirks.depot_loss_strands_hover: on, a destroyed depot's settled vehicle that receives no destination stays settled; off, losing contact with a depot ends being settled)
#[test]
fn quirk_depot_loss_strands_hover_off() {
    for strands in [true, false] {
        let mut r = rules();
        r.globals.quirks.depot_loss_strands_hover = strands;
        let (w, v) = destroyed_while_docked(&r);
        assert_eq!(contact(&w, v), None);
        assert_eq!(depot::settled(&w, v), strands, "strands {strands}");
    }
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
    let mut at = w.obj(b).unwrap().pos;
    assert!(run_until(&mut w, 3000, |w| {
        let hit = contact(w, b) == Some(d);
        if !hit {
            at = w.obj(b).unwrap().pos;
        }
        hit
    }));
    assert_eq!(contact(&w, a), None);
    // spec: S-0314 claim 23 (the retry follows the turn's movement: the turn that takes the place does not move the vehicle; a ready vehicle makes enter current at once without processing it; Q-0683 answered)
    assert_eq!(w.obj(b).unwrap().pos, at, "no movement in the retry's turn");
    assert_eq!(w.obj(b).unwrap().beh.current, Some(Behaviour::Enter));
    assert!(run_until(&mut w, 6000, |w| health(w, b) == 600));
}

// spec: S-0214 claim 23 (revision 41: for a group click on a depot, event order is selection order, S-0213, so the first selected vehicle takes contact and a damaged second one waits with a queued entry) and S-0314 claim 23
#[test]
fn s0214_group_click_first_selected_takes_contact() {
    let r = rules();
    let mut w = game(&r);
    let d = depot_at(&mut w, "DEPOT");
    let a = truck(&mut w, 0, 6, 1, 300);
    let b = truck(&mut w, 0, 6, 4, 300);
    open(&mut w);
    // Selected b first, then a.
    let orders =
        targeting::click(&w, ALPHA, &[b, a], ClickOn::Object(d), Modifiers::default()).orders;
    assert_eq!(orders.len(), 2, "one enter order each");
    for o in orders {
        assert!(orders::issue(&mut w, ALPHA, o));
    }
    tick::step(&mut w);
    assert_eq!(contact(&w, b), Some(d), "the first selected takes contact");
    assert_eq!(contact(&w, a), None);
    assert_eq!(w.obj(a).unwrap().depot.queued, Some(d));
}

/// Two damaged trucks sent to DEPOT in one tick, the first at health
/// `first`, the second (`b`) far from the depot at 300: returns the world,
/// the depot, the first truck and `b` right after the orders' tick.
fn queued_scene(r: &Ruleset, first: i32) -> (World, ObjectId, ObjectId, ObjectId) {
    let mut w = game(r);
    let d = depot_at(&mut w, "DEPOT");
    let a = truck(&mut w, 0, 6, 1, first);
    let b = truck(&mut w, 0, 9, 9, 300);
    open(&mut w);
    for o in [enter(a, d), enter(b, d)] {
        assert!(orders::issue(&mut w, ALPHA, o));
    }
    tick::step(&mut w);
    assert_eq!(contact(&w, a), Some(d));
    (w, d, a, b)
}

// spec: S-0314 claim 23 (revision 25: the order queues enter without an immediate start, records the depot as the remembered entry object and gives a queued entry; enter becomes current at the next ready per-tick start and repeats its request, which a full depot accepts without contact; no destination and no walk; it enters by itself at most one tick after the first leaves) and table row "queued entry, enter queued"; scenario H18 f(ii)
#[test]
fn claim_23_queued_enter_current_no_walk() {
    let r = rules();
    let (mut w, d, a, b) = queued_scene(&r, 590);
    let o = w.obj(b).unwrap();
    assert_eq!(o.depot.queued, Some(d));
    assert_eq!(o.depot.entering, Some(d), "the depot is remembered");
    assert_eq!(o.dest, None);
    assert!(
        o.beh.current == Some(Behaviour::Enter) || o.beh.queued == Some(Behaviour::Enter),
        "enter queued by the order"
    );
    assert!(run_until(&mut w, 2, |w| w.obj(b).unwrap().beh.current
        == Some(Behaviour::Enter)));
    let start = w.obj(b).unwrap().pos;
    // Every tick until the first leaves: enter current, no destination, no
    // contact, the same position, the queued entry and the memory kept.
    let mut left = None;
    for _ in 0..3000 {
        if contact(&w, a).is_none() {
            left = Some(w.tick);
            break;
        }
        let o = w.obj(b).unwrap();
        assert_eq!(o.beh.current, Some(Behaviour::Enter), "tick {}", w.tick);
        assert_eq!(o.dest, None, "tick {}", w.tick);
        assert_eq!(
            o.contact, None,
            "tick {}: not before the first leaves",
            w.tick
        );
        assert_eq!(o.pos, start, "tick {}: no walk", w.tick);
        assert_eq!(o.depot.queued, Some(d));
        assert_eq!(o.depot.entering, Some(d));
        tick::step(&mut w);
    }
    let left = left.expect("the first leaves");
    assert!(run_until(&mut w, 2, |w| contact(w, b) == Some(d)));
    assert!(
        w.tick - left <= 1,
        "enters at most one tick after the first leaves"
    );
    let o = w.obj(b).unwrap();
    assert_eq!(
        o.dest,
        Some(Target::Object(d)),
        "the retry assigns the depot"
    );
    assert_eq!(o.depot.queued, None);
    assert!(run_until(&mut w, 6000, |w| health(w, b) == 600));
}

// spec: S-0314 claim 23 (a depot refusing the repeated request, here switched off: the vehicle ends contact and falls back to rest, its queued entry stays and the retry still runs; when the place frees, permission is refused and it gives up) and scenario H18 f(ii) with the depot switched off
#[test]
fn claim_23_switched_off_queued_vehicle() {
    // The first truck is in the depot's repair, asleep, so the switch does
    // not end its contact (claim 22).
    let (mut w, d, a) = docked_scene(100);
    let b = truck(&mut w, 0, 9, 9, 300);
    assert!(orders::issue(&mut w, ALPHA, enter(b, d)));
    tick::step(&mut w);
    assert_eq!(w.obj(b).unwrap().depot.queued, Some(d));
    assert!(run_until(&mut w, 2, |w| w.obj(b).unwrap().beh.current
        == Some(Behaviour::Enter)));
    w.obj_mut(d).unwrap().econ.building.switched_off = true;
    assert!(run_until(&mut w, 30, |w| w.obj(b).unwrap().beh.current
        != Some(Behaviour::Enter)));
    assert_eq!(
        w.obj(b).unwrap().depot.queued,
        Some(d),
        "the queued entry stays"
    );
    assert_eq!(contact(&w, b), None);
    assert_ne!(contact(&w, a), None, "the first is still serviced");
    assert!(run_until(&mut w, 3000, |w| contact(w, a).is_none()));
    assert!(run_until(&mut w, 2, |w| w
        .obj(b)
        .unwrap()
        .depot
        .queued
        .is_none()));
    let o = w.obj(b).unwrap();
    assert_eq!(o.contact, None, "gives up");
    assert_eq!(o.dest, None);
    testkit::run(&mut w, 300);
    assert_eq!(health(&w, b), 300, "never enters");
}

// spec: S-0314 claim 23 (a depot holding any contact, whatever its place count: with a place free when the processing runs, that request takes the contact without a destination; the retry later in the same turn finds the contact held, asks permission and, granted, assigns the depot and clears the queued entry)
#[test]
fn claim_23_free_place_taken_by_the_processing() {
    let r = rules();
    let mut w = game(&r);
    let d = depot_at(&mut w, "DEPOT2");
    let a = truck(&mut w, 0, 6, 1, 300);
    let b = truck(&mut w, 0, 9, 9, 300);
    open(&mut w);
    for o in [enter(a, d), enter(b, d)] {
        assert!(orders::issue(&mut w, ALPHA, o));
    }
    tick::step(&mut w);
    assert_eq!(contact(&w, a), Some(d));
    let start = w.obj(b).unwrap().pos;
    if contact(&w, b).is_none() {
        assert_eq!(
            w.obj(b).unwrap().depot.queued,
            Some(d),
            "queued though a place is free"
        );
        assert_eq!(w.obj(b).unwrap().dest, None);
    }
    assert!(run_until(&mut w, 2, |w| contact(w, b) == Some(d)));
    let o = w.obj(b).unwrap();
    assert_eq!(o.beh.current, Some(Behaviour::Enter));
    assert_eq!(o.dest, Some(Target::Object(d)));
    assert_eq!(o.depot.queued, None);
    assert_eq!(o.pos, start, "no movement before the retry");
    assert!(run_until(&mut w, 6000, |w| health(w, b) == 600));
}

// spec: S-0314 claim 23 (the queued entry and the remembered depot are saved: the state hash survives a save and load mid-wait)
#[test]
fn claim_23_queued_state_round_trips() {
    let r = rules();
    let (mut w, d, _a, b) = queued_scene(&r, 590);
    testkit::run(&mut w, 5);
    assert_eq!(w.obj(b).unwrap().depot.entering, Some(d));
    let bytes = save::save(&w);
    let mut l = save::load(&bytes, w.rules_arc(), testkit::catalogue()).expect("loads");
    assert_eq!(save::state_hash(&l), save::state_hash(&w));
    for _ in 0..300 {
        tick::step(&mut w);
        tick::step(&mut l);
    }
    assert_eq!(save::state_hash(&l), save::state_hash(&w));
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
    assert_eq!(
        w.obj(b).unwrap().depot.entering,
        None,
        "the remembered depot is replaced"
    );
}

/// A queued vehicle given `order`: its queued entry after the order, and
/// whether it later takes the depot's place by itself (#197).
fn queued_then(order: fn(ObjectId) -> Order) -> (Option<ObjectId>, bool) {
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
    assert_eq!(contact(&w, a), Some(d));
    assert_eq!(w.obj(b).unwrap().depot.queued, Some(d));
    assert!(orders::issue(&mut w, ALPHA, order(b)));
    tick::step(&mut w);
    let queued = w.obj(b).unwrap().depot.queued;
    let pulled = run_until(&mut w, 3000, |w| contact(w, b) == Some(d));
    (queued, pulled)
}

// spec: S-0314 claim 23 (any later order clears the queued entry) and S-0213 claim 8 (stop)
#[test]
fn claim_23_stop_clears_queued_entry() {
    assert_eq!(queued_then(|actor| Order::Stop { actor }), (None, false));
}

// spec: S-0314 claim 23 (any later order clears the queued entry) and S-0213 claim 10 (a deploy order that passes its checks is applied)
#[test]
fn claim_23_deploy_clears_queued_entry() {
    assert_eq!(queued_then(|actor| Order::Deploy { actor }), (None, false));
}

// spec: S-0314 claim 23 (any later order clears the queued entry) and S-0213 claim 9 (a scatter order that passes its checks is applied)
#[test]
fn claim_23_scatter_clears_queued_entry() {
    assert_eq!(queued_then(|actor| Order::Scatter { actor }), (None, false));
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

// spec: S-0314 claims 6 and 18 and S-0504 claims 12 and 14 (a vehicle factory with the repair flag docks at its foundation centre, the vehicle-factory flag selecting no fixed point, and repairs; the vehicle is then sent away)
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
    assert_eq!(w.obj(v).unwrap().pos.cell(), testkit::mid(SIZE, 1, 1));
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

/// A damaged hovering vehicle (S-0503) docked at a fresh depot by a
/// click, past its settling step, with the depot then sold; returns the
/// world and the vehicle once it has lost contact. `strands` sets
/// `quirks.depot_loss_strands_hover`.
fn hover_after_depot_sale(strands: bool) -> (World, ObjectId) {
    let mut r = rules();
    r.globals.quirks.depot_loss_strands_hover = strands;
    r.registries
        .entry("vehicles".into())
        .or_default()
        .push("SKIMMER".into());
    r.sim.vehicles.insert(
        "SKIMMER".into(),
        UnitType {
            locomotion_class: Some(LocomotionClass::Hover),
            locomotor: Some(Locomotor::Hover),
            movement_zone: Some(MovementZone::Amphibious),
            ..wheel(600)
        },
    );
    // A 1 × 1 depot: its position and its docking point share one cell.
    r.registries
        .entry("buildings".into())
        .or_default()
        .push("PAD".into());
    let mut pad = r.sim.buildings["DEPOT"].clone();
    pad.foundation = Some([1, 1]);
    // Build-up frames, so that the sale's build-down ends and the depot
    // leaves (S-0408 claims 4 and 17, S-0108 claim 5).
    pad.econ.buildup_frames = Some(10);
    r.sim.buildings.insert("PAD".into(), pad);
    let mut w = game(&r);
    let d = depot_at(&mut w, "PAD");
    let v = spawn(&mut w, Kind::Vehicle, "SKIMMER", 0, 6, 1);
    w.obj_mut(v).unwrap().health = 300;
    open(&mut w);
    click_enter(&mut w, v, d);
    assert!(
        run_until(&mut w, 900, |w| health(w, v) > 300),
        "the hover docks and takes its settling step"
    );
    assert!(depot::settled(&w, v));
    sell::sell(&mut w, ALPHA, d);
    assert!(run_until(&mut w, 5, |w| contact(w, v).is_none()));
    // Once the sold depot has left the cell: a destination given while a
    // depot occupies it would switch the locomotor on (claim 22).
    assert!(
        run_until(&mut w, 900, |w| w.obj(d).is_none_or(|o| !o.on_map)),
        "the sale ends"
    );
    (w, v)
}

// spec: S-0314 claim 22 and Edge cases (quirks.depot_loss_strands_hover on: a hovering vehicle whose depot is sold after its settling step, with no destination received before the removal, stays settled and switched off and, down on the ground, cannot move); S-0503 claim 10 and F1 (switched off and down: not powered, T = 0)
#[test]
fn sold_depot_strands_a_hover_end_to_end() {
    let (mut w, v) = hover_after_depot_sale(true);
    assert!(depot::settled(&w, v));
    let o = w.obj(v).unwrap();
    let before = (o.pos.x, o.pos.y);
    assert!(orders::issue(&mut w, ALPHA, mv(v, 10, 10)));
    for _ in 0..300 {
        tick::step(&mut w);
    }
    let o = w.obj(v).unwrap();
    assert_eq!((o.pos.x, o.pos.y), before, "the stranded hover stays put");
}

// spec: S-0314 Edge cases (quirks.depot_loss_strands_hover off: losing contact ends being settled and switches the hover on); S-0503 claim 10 (the lift returns and it moves again)
#[test]
fn sold_depot_frees_a_hover_with_the_quirk_off() {
    let (mut w, v) = hover_after_depot_sale(false);
    assert!(!depot::settled(&w, v));
    let o = w.obj(v).unwrap();
    let before = (o.pos.x, o.pos.y);
    assert!(orders::issue(&mut w, ALPHA, mv(v, 10, 10)));
    for _ in 0..300 {
        tick::step(&mut w);
    }
    let o = w.obj(v).unwrap();
    assert_ne!((o.pos.x, o.pos.y), before, "the hover moves away");
}

// spec: S-0314 claim 18 (the supplied docking point, not the building's position, is the destination position, for a hovering vehicle too); S-0503 claim 7 (the hover stops in its destination's cell); S-0314 claims 6, 16 and 17 (it docks and the depot repairs it)
#[test]
fn hover_docks_at_a_three_by_three_depot_and_is_repaired() {
    let mut r = rules();
    r.registries
        .entry("vehicles".into())
        .or_default()
        .push("SKIMMER".into());
    r.sim.vehicles.insert(
        "SKIMMER".into(),
        UnitType {
            locomotion_class: Some(LocomotionClass::Hover),
            locomotor: Some(Locomotor::Hover),
            movement_zone: Some(MovementZone::Amphibious),
            ..wheel(600)
        },
    );
    let mut w = game(&r);
    let d = depot_at(&mut w, "DEPOT");
    let v = spawn(&mut w, Kind::Vehicle, "SKIMMER", 0, 6, 1);
    w.obj_mut(v).unwrap().health = 300;
    open(&mut w);
    click_enter(&mut w, v, d);
    assert!(
        run_until(&mut w, 900, |w| contact(w, v) == Some(d)
            && w.obj(d).unwrap().beh.current == Some(Behaviour::Repair)),
        "the hover arrives and the depot starts repair"
    );
    assert_eq!(
        w.obj(v).unwrap().pos.cell(),
        dock_cell(),
        "in the docking cell"
    );
    assert!(
        run_until(&mut w, 900, |w| health(w, v) > 300),
        "the depot repairs the hover"
    );
}

// spec: S-0314 claims 17 and 20 (an untethered vehicle at a depot without the pad flag, in sleep while the depot waits, settles or works, is released at once by a move order; no remote repair under the lost contact)
#[test]
fn claim_20_untethered_vehicle_released_in_every_phase() {
    for phase in 0..3 {
        let (mut w, d, v) = docked_scene(300);
        let dd = d_rep(&w);
        if phase >= 1 {
            next_step(&mut w, v, 600).expect("settling step");
        }
        if phase == 2 {
            assert!(run_until(&mut w, dd, |w| w.obj(d).unwrap().beh.progress == 2));
        }
        assert!(!docked(&w, v));
        assert!(orders::issue(&mut w, ALPHA, mv(v, 10, 10)));
        tick::step(&mut w);
        assert_eq!(contact(&w, v), None, "phase {phase}: released");
        let h = health(&w, v);
        assert!(run_until(&mut w, 900, |w| w.obj(v).unwrap().pos.cell()
            == testkit::mid(SIZE, 10, 10)
            && !glasnost_sim::occupancy::in_progress(w, v)));
        common_wait(&mut w, 200);
        assert_eq!(health(&w, v), h, "phase {phase}: no remote repair");
    }
}

fn common_wait(w: &mut World, n: i32) {
    for _ in 0..n {
        tick::step(w);
    }
}

/// Gives `v` the order-level destination `t` as after a qualifying track
/// completion: the locomotor destination point it supplied is already
/// cleared (S-0207 claim 25).
fn arrived(w: &mut World, v: ObjectId, t: Target) {
    let o = w.obj_mut(v).unwrap();
    o.dest = Some(t);
    let d = o.drive.as_mut().unwrap();
    d.for_dest = Some(t);
    d.goal = None;
}

// spec: S-0314 claim 17 (a pad depot's repeated request tethers a vehicle in the foundation centre's cell with no moving destination; an ordinary depot's request sets no tether; revision 29: neither supplies the completed-entry confirmation, so later processings neither turn the tethered vehicle toward 16384 facing units nor queue sleep)
#[test]
fn claim_17_pad_depot_tethers_in_the_enter_processing() {
    for (name, want) in [("PAD", true), ("DEPOT", false)] {
        let r = rules();
        let mut w = game(&r);
        let d = depot_at(&mut w, name);
        let v = truck(&mut w, 0, 1, 1, 300);
        open(&mut w);
        assert!(depot::request(&mut w, v, d, false));
        w.obj_mut(v).unwrap().beh.current = Some(Behaviour::Enter);
        arrived(&mut w, v, Target::Object(d));
        depot::process_enter(&mut w, v);
        assert_eq!(docked(&w, v), want, "{name}");
        for _ in 0..3 {
            depot::process_enter(&mut w, v);
            let t = w.tick;
            let o = w.obj(v).unwrap();
            assert!(!o.body.turning(t), "{name}: no turn");
            assert_eq!(o.body.value(t), 0, "{name}: facing kept");
            assert_eq!(o.beh.queued, None, "{name}: no sleep queued");
            tick::step(&mut w);
        }
    }
}

// spec: S-0314 claim 17 and Terms (the pad exchange: in the centre cell with no moving destination; with one, only when the vehicle in place 0 is more than 128 leptons from the centre by the truncated spatial distance, so 128 does not pass and 129 does)
#[test]
fn claim_17_pad_exchange_place_zero_distance() {
    let r = rules();
    let mut w = game(&r);
    let d = depot_at(&mut w, "PAD2");
    let first = truck(&mut w, 0, 8, 8, 300);
    let v = truck(&mut w, 0, 1, 1, 300);
    open(&mut w);
    assert!(depot::request(&mut w, first, d, false));
    assert!(depot::request(&mut w, v, d, false));
    assert_eq!(depot::link(&w, first).unwrap().place, 0);
    let c = w.reference_point(d);
    assert_eq!(w.obj(v).unwrap().pos.cell(), c.cell());
    arrived(&mut w, v, Target::Object(d));
    assert!(
        depot::pad_exchange(&w, d, v),
        "a destination whose point a completion cleared is not a moving destination"
    );
    // A destination away from its cell: the locomotor destination point is
    // present and the locomotor reports movement.
    w.obj_mut(v).unwrap().dest = Some(Target::Cell(testkit::mid(SIZE, 10, 10)));
    for (off, want) in [(128, false), (129, true)] {
        w.obj_mut(first).unwrap().pos = Pos::new(c.x + off, c.y, c.z);
        assert_eq!(depot::pad_exchange(&w, d, v), want, "place 0 at {off}");
    }
    // An ordinary depot has no such exchange.
    let mut w2 = game(&r);
    let d2 = depot_at(&mut w2, "DEPOT");
    let v2 = truck(&mut w2, 0, 1, 1, 300);
    open(&mut w2);
    assert!(depot::request(&mut w2, v2, d2, false));
    assert!(!depot::pad_exchange(&w2, d2, v2));
}

// spec: S-0314 claim 17 and Terms, S-0207 claim 25 (the movement report of a driving vehicle: its locomotor destination point present, even in its own cell, or its heading point present and differing from its horizontal position, x and y only; a turn in place, a track in progress or stored steps are not sufficient by themselves; the order-level destination is needed in every row)
#[test]
fn claim_17_driving_movement_report() {
    let r = rules();
    let mut w = game(&r);
    let d = depot_at(&mut w, "PAD");
    let v = truck(&mut w, 0, 1, 1, 300);
    open(&mut w);
    assert!(depot::request(&mut w, v, d, false));
    let c = w.reference_point(d);
    // Place 0 holds the requester itself, at the centre: the distance never
    // passes, so the exchange tethers exactly when no moving destination.
    let still = |w: &World| depot::pad_exchange(w, d, v);
    w.obj_mut(v).unwrap().dest = Some(Target::Object(d));
    assert!(!still(&w), "a point assigned in its own cell is present");
    // Its point cleared by a qualifying completion: no report.
    arrived(&mut w, v, Target::Object(d));
    assert!(
        still(&w),
        "the point cleared, the destination kept: no report"
    );
    let pos = w.obj(v).unwrap().pos;
    let track = |target: (i32, i32), z: i32| Track {
        d: 2,
        e: None,
        target,
        target_cell: Pos::new(target.0, target.1, z).cell(),
        height: z,
        next: 0,
        last: (pos.x, pos.y),
        cells: vec![pos.cell()],
        crush: false,
        exit: false,
        bunker: None,
    };
    // Stored steps, a turn in place and a track whose heading point is at
    // the current horizontal position (only its height differs) do not
    // report movement.
    w.obj_mut(v).unwrap().path.steps.push(0);
    w.obj_mut(v)
        .unwrap()
        .drive
        .as_mut()
        .unwrap()
        .turning_in_place = true;
    w.obj_mut(v).unwrap().drive.as_mut().unwrap().track = Some(track((pos.x, pos.y), pos.z + 100));
    assert!(still(&w), "steps, a turn and a same-position heading point");
    // A heading point differing in x only reports movement.
    w.obj_mut(v).unwrap().drive.as_mut().unwrap().track = Some(track((pos.x + 1, pos.y), pos.z));
    assert!(!still(&w), "heading point one lepton away");
    // Without the order-level destination there is no moving destination.
    w.obj_mut(v).unwrap().dest = None;
    assert!(still(&w), "no order-level destination");
    // A destination point away from its cell reports movement without a
    // heading point.
    w.obj_mut(v).unwrap().drive.as_mut().unwrap().track = None;
    w.obj_mut(v).unwrap().dest = Some(Target::Cell(testkit::mid(SIZE, 10, 10)));
    assert!(!still(&w), "destination point present");
    assert_eq!(w.obj(v).unwrap().pos.cell(), c.cell());
}

// spec: S-0314 claims 17 and 18, S-0207 claim 25 (Q-0794 question 3: an accepted enter order from rest in the supplied docking point's cell supplies that point to the driving locomotor despite cell equality, with no search or track; the retained order-level destination makes it a moving destination, so the pad exchange does not tether at the repeated request); S-0207 claim 26 (Q-0797: the destination names a building, so the drive updates at rest in its docking point's cell remove neither destination)
#[test]
fn claim_17_same_cell_enter_supplies_the_point() {
    let mut r = rules();
    r.sim.buildings.get_mut("PAD").unwrap().econ.docking_offsets = Some(vec![[0, 400, 0]]);
    let mut w = game(&r);
    let d = depot_at(&mut w, "PAD");
    let v = truck(&mut w, 0, 1, 3, 300);
    open(&mut w);
    let dp = depot::docking_point(&w, d, v);
    assert_eq!(
        dp.cell(),
        w.obj(v).unwrap().pos.cell(),
        "standing in its cell"
    );
    assert!(orders::issue(&mut w, ALPHA, enter(v, d)));
    orders::apply_pending(&mut w);
    assert_eq!(contact(&w, v), Some(d), "accepted");
    assert_eq!(w.obj(v).unwrap().dest, Some(Target::Object(d)));
    assert_eq!(glasnost_sim::drive::goal(&w, v), Some(dp), "point present");
    assert!(w.obj(v).unwrap().drive.as_ref().unwrap().track.is_none());
    assert!(
        !depot::pad_exchange(&w, d, v),
        "a moving destination despite no translation"
    );
    // The repeated request of the first enter processing finds it so.
    let pos = w.obj(v).unwrap().pos;
    depot::process_enter(&mut w, v);
    assert!(!docked(&w, v), "no tether at the repeated request");
    assert_eq!(w.obj(v).unwrap().pos, pos);
    // spec: S-0207 claim 26 (a building destination is kept by locomotion updates at rest in the supplied point's cell: cell equality removes neither the order-level destination nor the point)
    for _ in 0..3 {
        glasnost_sim::drive::tick(&mut w, v);
    }
    assert_eq!(w.obj(v).unwrap().dest, Some(Target::Object(d)));
    assert_eq!(glasnost_sim::drive::goal(&w, v), Some(dp));
    assert_eq!(w.obj(v).unwrap().pos, pos);
}

/// A truck in guard in the pad depot's docking-point cell with the accepted
/// same-cell enter order, the docking point offset `offset` in y and the
/// truck's height raised by `dz`.
fn guard_row_scene(offset: i32, dz: i32) -> (World, ObjectId, ObjectId) {
    let mut r = rules();
    r.sim.buildings.get_mut("PAD").unwrap().econ.docking_offsets = Some(vec![[0, offset, 0]]);
    let mut w = game(&r);
    let d = depot_at(&mut w, "PAD");
    let v = truck(&mut w, 0, 1, 3, 300);
    open(&mut w);
    w.obj_mut(v).unwrap().pos.z += dz;
    assert!(orders::issue(&mut w, ALPHA, enter(v, d)));
    orders::apply_pending(&mut w);
    assert_eq!(w.obj(v).unwrap().dest, Some(Target::Object(d)), "accepted");
    assert_eq!(w.obj(v).unwrap().beh.current, Some(Behaviour::Guard));
    (w, d, v)
}

// spec: S-0207 claim 26 (third row: at rest in guard with the locomotor destination point present, the vehicle's world point equal to it in all three coordinates removes the goal, a building destination included, and the point with it)
#[test]
fn claim_26_guard_row_removes_the_building_goal_at_the_exact_point() {
    let (mut w, _d, v) = guard_row_scene(512, 0);
    let dp = glasnost_sim::drive::goal(&w, v).expect("point");
    assert_eq!(w.obj(v).unwrap().pos, dp, "all three coordinates equal");
    glasnost_sim::drive::tick(&mut w, v);
    assert_eq!(w.obj(v).unwrap().dest, None, "goal removed");
    assert!(!glasnost_sim::drive::goal_present(&w, v));
}

// spec: S-0207 claim 26 (the guard row needs exact integer-lepton equality of all coordinates; the same cell, a horizontal displacement inside it, or a height difference is insufficient, and a building destination is not removed by cell equality)
#[test]
fn claim_26_guard_row_needs_exact_equality() {
    for (offset, dz) in [(400, 0), (512, 1)] {
        let (mut w, d, v) = guard_row_scene(offset, dz);
        let dp = glasnost_sim::drive::goal(&w, v).expect("point");
        assert_eq!(w.obj(v).unwrap().pos.cell(), dp.cell(), "same cell");
        assert_ne!(w.obj(v).unwrap().pos, dp, "not the same point");
        for _ in 0..3 {
            glasnost_sim::drive::tick(&mut w, v);
        }
        assert_eq!(
            w.obj(v).unwrap().dest,
            Some(Target::Object(d)),
            "offset {offset}, dz {dz}"
        );
        assert!(glasnost_sim::drive::goal_present(&w, v));
    }
}

// spec: S-0215 claim 29 (Q-0797: an accepted entry with contact and no approach destinations keeps the building destination through due enter processings at the docking-point cell; the movement arrival's queued sleep is not a removal of the order-level destination)
#[test]
fn s0215_claim_29_entry_keeps_the_building_destination() {
    let mut r = rules();
    r.sim.buildings.get_mut("PAD").unwrap().econ.docking_offsets = Some(vec![[0, 400, 0]]);
    let mut w = game(&r);
    let d = depot_at(&mut w, "PAD");
    let v = truck(&mut w, 0, 1, 3, 300);
    open(&mut w);
    assert!(orders::issue(&mut w, ALPHA, enter(v, d)));
    orders::apply_pending(&mut w);
    for _ in 0..3 {
        depot::process_enter(&mut w, v);
        glasnost_sim::drive::tick(&mut w, v);
    }
    depot::movement_arrival(&mut w, v);
    assert_eq!(
        w.obj(v).unwrap().dest,
        Some(Target::Object(d)),
        "kept through the repeated processings and the arrival exchange"
    );
}

// spec: S-0314 claim 17 (with place 0 empty, a requester with a moving destination cannot tether through the exchange; one without a moving destination still can)
#[test]
fn claim_17_pad_exchange_empty_place_zero() {
    let r = rules();
    let mut w = game(&r);
    let d = depot_at(&mut w, "PAD2");
    let first = truck(&mut w, 0, 8, 8, 300);
    let v = truck(&mut w, 0, 1, 1, 300);
    open(&mut w);
    assert!(depot::request(&mut w, first, d, false));
    assert!(depot::request(&mut w, v, d, false));
    assert_eq!(depot::link(&w, v).unwrap().place, 1);
    depot::stop_order(&mut w, first);
    assert!(depot::link(&w, first).is_none(), "place 0 empty");
    w.obj_mut(v).unwrap().dest = Some(Target::Cell(testkit::mid(SIZE, 10, 10)));
    assert!(
        !depot::pad_exchange(&w, d, v),
        "moving destination: no substitute position"
    );
    w.obj_mut(v).unwrap().dest = None;
    assert!(depot::pad_exchange(&w, d, v), "no moving destination");
}

// spec: S-0314 claim 17 and S-0215 claim 29 (reaching the docking-point cell during movement sets the depot's ready mark and queues repair at the depot and sleep at the vehicle, neither started at once, and creates no tether)
#[test]
fn claim_17_movement_arrival_sets_the_ready_mark() {
    for name in ["DEPOT", "PAD"] {
        let r = rules();
        let mut w = game(&r);
        let d = depot_at(&mut w, name);
        let v = truck(&mut w, 0, 1, 1, 300);
        open(&mut w);
        assert!(depot::request(&mut w, v, d, false));
        w.obj_mut(v).unwrap().beh.current = Some(Behaviour::Enter);
        w.obj_mut(v).unwrap().dest = Some(Target::Object(d));
        w.obj_mut(d).unwrap().beh.ready_mark = false;
        let before = w.obj(d).unwrap().beh.current;
        depot::movement_arrival(&mut w, v);
        let b = &w.obj(d).unwrap().beh;
        assert!(b.ready_mark, "{name}: ready mark set");
        assert_eq!(b.queued, Some(Behaviour::Repair), "{name}: repair queued");
        assert_eq!(b.current, before, "{name}: not started at once");
        let o = w.obj(v).unwrap();
        assert_eq!(o.beh.queued, Some(Behaviour::Sleep), "{name}: sleep queued");
        assert_eq!(o.beh.current, Some(Behaviour::Enter));
        assert!(!docked(&w, v), "{name}: no tether");
    }
}

// spec: S-0314 claim 17 (revision 29, Q-1162: movement arrival itself requests no map-east body turn, tethered or not; sleep is queued and, the vehicle being ready, current at the next start) and S-0215 claim 29 (revision 67, the depot table's movement-arrival row)
#[test]
fn claim_17_arrival_requests_no_turn() {
    for tethered in [true, false] {
        let r = rules();
        let mut w = game(&r);
        let d = depot_at(&mut w, "DEPOT");
        let v = truck(&mut w, 0, 1, 1, 300);
        open(&mut w);
        assert!(depot::request(&mut w, v, d, false));
        if tethered {
            tether(&mut w, v);
        }
        w.obj_mut(v).unwrap().beh.current = Some(Behaviour::Enter);
        arrived(&mut w, v, Target::Object(d));
        assert_eq!(w.body_facing(v), 0);
        depot::movement_arrival(&mut w, v);
        assert_eq!(w.obj(v).unwrap().beh.queued, Some(Behaviour::Sleep));
        assert!(
            !w.obj(v).unwrap().body.turning(w.tick),
            "tethered {tethered}: arrival starts no turn"
        );
        tick::step(&mut w);
        assert_eq!(w.obj(v).unwrap().beh.current, Some(Behaviour::Sleep));
        assert_eq!(w.body_facing(v), 0, "not rotated");
    }
}

// spec: S-0314 claim 17 (revision 29: a driving vehicle already turning when arrival queues sleep is held by S-0215 claim 11's ordinary readiness test until the first start after the turn ends; arrival adds no gate of its own) and S-0215 claim 11 (revision 67: the turn ends by tick number)
#[test]
fn claim_17_existing_turn_holds_the_arrival_sleep() {
    let r = rules();
    let mut w = game(&r);
    let d = depot_at(&mut w, "DEPOT");
    let v = truck(&mut w, 0, 1, 1, 300);
    open(&mut w);
    assert!(depot::request(&mut w, v, d, false));
    w.obj_mut(v).unwrap().beh.current = Some(Behaviour::Enter);
    arrived(&mut w, v, Target::Object(d));
    let t0 = w.tick;
    let rate = {
        let o = w.obj(v).unwrap();
        w.rules().unit(o.kind, o.type_index).unwrap().turn_rate
    };
    assert!(w.obj_mut(v).unwrap().body.turn_to(32768, t0, rate));
    depot::movement_arrival(&mut w, v);
    assert_eq!(w.obj(v).unwrap().beh.queued, Some(Behaviour::Sleep));
    let mut ended = None;
    for _ in 0..400 {
        tick::step(&mut w);
        let k = w.tick - 1;
        let o = w.obj(v).unwrap();
        if o.body.turning(k) {
            assert_ne!(
                o.beh.current,
                Some(Behaviour::Sleep),
                "tick {k}: still turning"
            );
        } else {
            assert_eq!(
                o.beh.current,
                Some(Behaviour::Sleep),
                "tick {k}: turn ended"
            );
            assert_eq!(o.beh.start_tick, k, "the first start of the end tick");
            ended = Some(k);
            break;
        }
    }
    assert!(
        ended.is_some_and(|k| k > t0 + 1),
        "the turn takes several ticks"
    );
}

/// Runs `n` ticks and returns the depot's processings, as (tick, delay
/// chosen) pairs from its behaviour record, in order.
fn processings(w: &mut World, d: ObjectId, n: i32) -> Vec<(i32, i32)> {
    let mut out = Vec::new();
    let mut last = w.obj(d).unwrap().beh.last;
    for _ in 0..n {
        tick::step(w);
        let l = w.obj(d).unwrap().beh.last;
        if l != last {
            out.extend(l);
            last = l;
        }
    }
    out
}

// spec: S-0314 Terms (a vehicle becomes settled by its movement arrival in the docking point's cell)
#[test]
fn terms_movement_arrival_makes_the_vehicle_settled() {
    let r = rules();
    let mut w = game(&r);
    let d = depot_at(&mut w, "DEPOT");
    let v = truck(&mut w, 0, 1, 1, 300);
    open(&mut w);
    assert!(depot::request(&mut w, v, d, false));
    w.obj_mut(v).unwrap().beh.current = Some(Behaviour::Enter);
    arrived(&mut w, v, Target::Object(d));
    assert!(!depot::settled(&w, v));
    depot::movement_arrival(&mut w, v);
    assert!(depot::settled(&w, v), "settled at the arrival");
}

// spec: S-0314 claim 6 (the processing spacing table: settling, the step answering more needed starts working and the first working processing comes D ticks later; working is processed every tick, with the second step due D + K_u − 1 ticks after the settling step and each later one K_u after the one before; the working step answering done is followed 1 tick later)
#[test]
fn claim_6_spacing_table_through_settling_and_working() {
    let (mut w, d, v) = docked_scene(560);
    let (k, dd) = (k_u(&w), d_rep(&w));
    let mut steps = Vec::new();
    let mut procs = Vec::new();
    let mut last = w.obj(d).unwrap().beh.last;
    for _ in 0..(dd + 4 * k + 40) {
        let (t, h) = (w.tick, health(&w, v));
        tick::step(&mut w);
        if health(&w, v) != h {
            steps.push(t);
        }
        let l = w.obj(d).unwrap().beh.last;
        if l != last {
            procs.extend(l);
            last = l;
        }
    }
    assert_eq!(steps.len(), 4, "four steps to full health: {steps:?}");
    let at = |t: i32| {
        procs
            .iter()
            .position(|&(p, _)| p == t)
            .expect("a processing at the step")
    };
    let s = at(steps[0]);
    assert_eq!(procs[s].1, dd, "settling step: the next processing D later");
    assert_eq!(procs[s + 1].0, steps[0] + dd, "{procs:?}");
    let done = at(steps[3]);
    for i in s + 1..=done {
        assert_eq!(procs[i].1, 1, "working: every tick: {procs:?}");
        if i > s + 1 {
            assert_eq!(procs[i].0, procs[i - 1].0 + 1, "{procs:?}");
        }
    }
    assert_eq!(steps[1] - steps[0], dd + k - 1);
    assert_eq!(steps[2] - steps[1], k);
    assert_eq!(
        procs[done + 1].0,
        steps[3] + 1,
        "1 tick after the done step"
    );
}

// spec: S-0314 Terms and claim 21 (within 200 leptons a settling processing that finds the vehicle not settled takes no step and rechecks after 1 tick; standing still by its locomotor's movement report, that processing makes it settled, so the step comes at the recheck)
#[test]
fn claim_21_near_vehicle_not_settled_rechecks_after_one_tick() {
    let (mut w, _d, v) = docked_scene(300);
    let s0 = next_step(&mut w, v, 600).expect("settling step");
    let (mut w, d, v) = docked_scene(300);
    w.obj_mut(v).unwrap().depot.settled = depot::Settled::No;
    let s1 = next_step(&mut w, v, 600).expect("settling step");
    assert_eq!(s1, s0 + 1, "one recheck 1 tick later");
    assert!(depot::settled(&w, v), "the recheck's processing settled it");
    assert_eq!(
        w.obj(v).unwrap().dest,
        None,
        "the step empties the destination"
    );
    assert_eq!(contact(&w, v), Some(d));
}

/// A track whose heading point is `to`, from the vehicle's position.
fn heading(w: &World, v: ObjectId, to: (i32, i32)) -> Track {
    let pos = w.obj(v).unwrap().pos;
    Track {
        d: 2,
        e: None,
        target: to,
        target_cell: Pos::new(to.0, to.1, pos.z).cell(),
        height: pos.z,
        next: 0,
        last: (pos.x, pos.y),
        cells: vec![pos.cell()],
        crush: false,
        exit: false,
        bunker: None,
    }
}

// spec: S-0314 Terms and claims 6 and 21 (a waiting processing that does not accept the vehicle, which moves by the moving-destination test, ends its being settled; the next processing comes D ticks later, the depot still waiting)
#[test]
fn claim_21_waiting_refusal_ends_being_settled() {
    let (mut w, d, v) = docked_scene(300);
    let dd = d_rep(&w);
    assert!(depot::settled(&w, v));
    w.obj_mut(d).unwrap().beh.progress = 0;
    let p = w.obj(v).unwrap().pos;
    let t = heading(&w, v, (p.x + 256, p.y));
    let o = w.obj_mut(v).unwrap();
    o.dest = Some(Target::Object(d));
    o.drive.as_mut().unwrap().track = Some(t);
    let procs = processings(&mut w, d, 4);
    assert_eq!(procs.len(), 1, "{procs:?}");
    assert_eq!(procs[0].1, dd, "not accepted: D ticks later");
    assert_eq!(w.obj(d).unwrap().beh.progress, 0, "still waiting");
    assert!(!depot::settled(&w, v), "the refusal ends being settled");
}

// spec: S-0314 claim 6 (settling, not needed with a failed departure: the vehicle stays in contact and the next processing comes D ticks later, every retry)
#[test]
fn claim_6_failed_departure_retries_d_apart() {
    let list = glasnost_sim::econ::release::exit_list(3, 3, false, true);
    let (mut w, d, v) = full_scene();
    let dd = d_rep(&w);
    let mut blocked = Vec::new();
    for &(a, b) in &list {
        let c = testkit::mid(SIZE, a, b);
        if w.cells.footprint().is_legal(c) && !blocked.contains(&c) {
            spawn(&mut w, Kind::Vehicle, "TRUCK", 0, a, b);
            blocked.push(c);
        }
    }
    let procs = processings(&mut w, d, 5 * dd + 10);
    assert_eq!(contact(&w, v), Some(d), "no departure cell: contact kept");
    let tail: Vec<_> = procs.iter().rev().take(3).collect();
    assert_eq!(tail.len(), 3, "{procs:?}");
    for p in &tail {
        assert_eq!(p.1, dd, "{procs:?}");
    }
    assert_eq!(tail[0].0 - tail[1].0, dd);
    assert_eq!(tail[1].0 - tail[2].0, dd);
}

// spec: S-0314 Terms and claim 22, S-0207 claim 27 (only an effective destination ends being settled: one that changes the destination; the same destination received again with no refresh pending is no end)
#[test]
fn terms_only_an_effective_destination_ends_being_settled() {
    let (mut w, d, v) = docked_scene(300);
    let p = w.obj(v).unwrap().pos;
    w.obj_mut(v).unwrap().dest = Some(Target::Object(d));
    w.obj_mut(v).unwrap().depot.settled = depot::Settled::AtDepot;
    glasnost_sim::infantry::destination_assigned(&mut w, v, Some(Target::Object(d)));
    assert!(depot::settled(&w, v), "the same destination: not effective");
    w.obj_mut(v).unwrap().dest = Some(Target::Cell(p.cell()));
    glasnost_sim::infantry::destination_assigned(&mut w, v, Some(Target::Object(d)));
    assert!(
        !depot::settled(&w, v),
        "a changed destination on the depot's cell"
    );
}

/// The rules plus SKIMMER, a hovering vehicle otherwise like TRUCK.
fn skimmer_rules() -> Ruleset {
    let mut r = rules();
    r.registries
        .entry("vehicles".into())
        .or_default()
        .push("SKIMMER".into());
    r.sim.vehicles.insert(
        "SKIMMER".into(),
        UnitType {
            locomotion_class: Some(LocomotionClass::Hover),
            locomotor: Some(Locomotor::Hover),
            movement_zone: Some(MovementZone::Amphibious),
            ..wheel(600)
        },
    );
    r
}

// spec: S-0314 Terms and claim 21 (the movement arrival makes a hovering vehicle settled and switches it off; within 200 leptons the settling step waits, rechecking after 1 tick, until it is settled and down); S-0503 claim 10 (switched off: it comes down)
#[test]
fn claim_21_hover_settling_step_waits_until_down() {
    let r = skimmer_rules();
    let mut w = game(&r);
    let d = depot_at(&mut w, "DEPOT");
    let v = spawn(&mut w, Kind::Vehicle, "SKIMMER", 0, 6, 1);
    w.obj_mut(v).unwrap().health = 300;
    open(&mut w);
    click_enter(&mut w, v, d);
    assert!(run_until(&mut w, 900, |w| depot::settled(w, v)));
    assert!(
        glasnost_sim::hover::switched_off(&w, v),
        "switched off at the arrival"
    );
    let mut seen = Vec::new();
    let mut last = w.obj(d).unwrap().beh.last;
    let mut step = None;
    for _ in 0..900 {
        let (h, z) = (health(&w, v), glasnost_sim::hover::hover_height(&w, v));
        let phase = w.obj(d).unwrap().beh.progress;
        tick::step(&mut w);
        let l = w.obj(d).unwrap().beh.last;
        if l != last {
            seen.push((l.unwrap(), phase, z));
            last = l;
        }
        if health(&w, v) != h {
            step = Some(z);
            break;
        }
    }
    let z = step.expect("the settling step");
    assert!(z <= 0, "the step comes once it is down: {seen:?}");
    let up: Vec<_> = seen
        .iter()
        .filter(|&&(_, phase, z)| phase == 1 && z > 0)
        .collect();
    assert!(!up.is_empty(), "settling met it not yet down: {seen:?}");
    for &&((_, delay), _, _) in &up {
        assert_eq!(
            delay, 1,
            "not down: no step, a recheck 1 tick later: {seen:?}"
        );
    }
    assert!(depot::settled(&w, v) && glasnost_sim::hover::switched_off(&w, v));
}

// spec: S-0606 claim 20 and S-0314 Terms (the tank bunker's release switches a hovering vehicle on; switching on is not itself an end of being settled, which only the Terms' listed events end)
#[test]
fn switching_on_alone_is_not_an_end_of_being_settled() {
    let r = skimmer_rules();
    let mut w = game(&r);
    let v = spawn(&mut w, Kind::Vehicle, "SKIMMER", 0, 6, 1);
    open(&mut w);
    w.obj_mut(v).unwrap().depot.settled = depot::Settled::AtDepot;
    glasnost_sim::hover::switched(&mut w, v, true);
    assert!(glasnost_sim::hover::switched_off(&w, v));
    glasnost_sim::hover::switched(&mut w, v, false);
    assert!(!glasnost_sim::hover::switched_off(&w, v));
    assert!(depot::settled(&w, v), "switching on alone ends nothing");
}

/// A damaged SKIMMER clicked onto DEPOT and run to its movement arrival:
/// settled at a depot, switched off and still above the ground (S-0314
/// claim 17 and Terms; S-0503 claim 10). Returns the world and the vehicle.
fn skimmer_sinking() -> (World, ObjectId) {
    let r = skimmer_rules();
    let mut w = game(&r);
    let d = depot_at(&mut w, "DEPOT");
    let v = spawn(&mut w, Kind::Vehicle, "SKIMMER", 0, 6, 1);
    w.obj_mut(v).unwrap().health = 300;
    open(&mut w);
    click_enter(&mut w, v, d);
    assert!(run_until(&mut w, 900, |w| depot::settled(w, v)));
    assert!(glasnost_sim::hover::switched_off(&w, v));
    assert!(
        glasnost_sim::hover::hover_height(&w, v) > 0,
        "still above the ground at the arrival"
    );
    assert!(
        depot::settled_at_depot(&w, v),
        "the arrival makes it settled at a depot"
    );
    (w, v)
}

/// Whether a building occupies vehicle `v`'s cell (here: the depot).
fn on_the_depot(w: &World, v: ObjectId) -> bool {
    let c = w.obj(v).unwrap().pos.cell();
    w.cells
        .list(c, glasnost_sim::world::Layer::Ground)
        .iter()
        .any(|&x| w.obj(x).is_some_and(|o| o.kind == Kind::Building))
}

// spec: S-0314 Terms (What ends it: a destination received while a depot occupies its cell ends being settled only by a vehicle whose locomotor then counts as switched off; a hovering vehicle still above the ground is powered in S-0503's sense, so a destination it receives then ends nothing, an empty one included; its lift stays off and it goes on sinking; once down, the same destination ends it and switches it on) and claim 22 (Settled vehicles)
#[test]
fn terms_a_destination_while_a_hover_is_still_above_the_ground_ends_nothing() {
    let (mut w, v) = skimmer_sinking();
    assert!(on_the_depot(&w, v), "the docking point lies in the depot");
    let to = Some(Target::Cell(testkit::mid(SIZE, 1, 2)));
    glasnost_sim::econ::assign_destination(&mut w, v, to);
    assert_eq!(w.obj(v).unwrap().dest, to, "the assignment takes effect");
    assert!(depot::settled(&w, v), "above the ground: not an end");
    assert!(
        glasnost_sim::hover::switched_off(&w, v),
        "not switched on: its lift stays off"
    );
    // A cleared destination (as a disabling's guard reset gives) above the ground: nothing either.
    glasnost_sim::econ::assign_destination(&mut w, v, None);
    assert!(depot::settled(&w, v) && glasnost_sim::hover::switched_off(&w, v));
    assert!(
        run_until(&mut w, 300, |w| glasnost_sim::hover::hover_height(w, v)
            <= 0),
        "it goes on sinking and comes down"
    );
    assert!(depot::settled(&w, v));
    assert!(on_the_depot(&w, v));
    glasnost_sim::econ::assign_destination(&mut w, v, to);
    assert!(
        !depot::settled(&w, v),
        "down: the destination ends being settled"
    );
    assert!(
        !glasnost_sim::hover::switched_off(&w, v),
        "switched on again at the end"
    );
}

// spec: S-0314 Terms and claim 22 (a person's move order in the sinking window releases the untethered vehicle's contact, claim 20, and ends no being settled while it is above the ground; down, a settled hovering vehicle cannot move while it stays settled; a later order with a new destination while the depot occupies its cell ends being settled and it leaves; the same destination again is rejected by S-0207 claim 27 and ends nothing); Open questions (whether the order moves it in the window: our hover takes the move while powered, S-0503 claim 4)
#[test]
fn claim_22_move_order_in_the_sinking_window_then_again_once_down() {
    let (mut w, v) = skimmer_sinking();
    assert!(orders::issue(&mut w, ALPHA, mv(v, 9, 9)));
    tick::step(&mut w);
    assert_eq!(contact(&w, v), None, "the order releases the contact");
    assert!(depot::settled(&w, v), "above the ground: still settled");
    assert!(run_until(&mut w, 300, |w| {
        glasnost_sim::hover::hover_height(w, v) <= 0
    }));
    assert!(depot::settled(&w, v), "down and still settled");
    let at = w.obj(v).unwrap().pos;
    glasnost_sim::testkit::run(&mut w, 60);
    assert_eq!(
        w.obj(v).unwrap().pos,
        at,
        "a settled hover that is down stays put"
    );
    assert!(
        on_the_depot(&w, v),
        "in this scene it comes down in the depot's cells: {:?}",
        at.cell()
    );
    // The same destination again is no receipt (S-0207 claim 27: an
    // assignment that leaves the destination unchanged is rejected).
    assert_eq!(
        w.obj(v).unwrap().dest,
        Some(Target::Cell(testkit::mid(SIZE, 9, 9)))
    );
    assert!(orders::issue(&mut w, ALPHA, mv(v, 9, 9)));
    tick::step(&mut w);
    assert!(depot::settled(&w, v), "the same destination ends nothing");
    assert!(orders::issue(&mut w, ALPHA, mv(v, 9, 8)));
    tick::step(&mut w);
    assert!(
        !depot::settled(&w, v),
        "a new destination ends being settled"
    );
    assert!(
        run_until(&mut w, 900, |w| w.obj(v).unwrap().pos.cell()
            == testkit::mid(SIZE, 9, 8)),
        "and it leaves"
    );
}

// spec: S-0314 Terms (Settled vehicle, Settled at a depot: the state, at a depot or away, survives a save and load)
#[test]
fn settled_where_round_trips() {
    for s in [depot::Settled::AtDepot, depot::Settled::Away] {
        let (mut w, v) = skimmer_sinking();
        w.obj_mut(v).unwrap().depot.settled = s;
        let h = save::state_hash(&w);
        let l = save::load(&save::save(&w), w.rules_arc(), testkit::catalogue()).unwrap();
        assert_eq!(l.obj(v).unwrap().depot.settled, s);
        assert_eq!(save::state_hash(&l), h);
    }
}
