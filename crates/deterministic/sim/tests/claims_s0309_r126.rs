//! Claim tests of S-0309 revision 126 (Q-2801 answered), claim 31: the
//! orders reset at an owner change for a building at work. A building
//! being sold requests its idle body, which can cut its build-down short;
//! a building being built up has its grand opening at the change, with no
//! free unit; a vehicle factory's unloading ends with its unit left at the
//! exit point and its door left open (r129: a later release takes the
//! unit's contact, `claims_s0309_r129.rs`); a repair
//! depot stops repairing and its vehicle stays docked in *guard*; switch
//! `quirks.owner_change_interrupts_building_work` keeps the sale, the
//! unloading and the repair. Also a psychic domination's reset and claim
//! 23 (d)'s end-of-update arrival of an infantry unit. Synthetic rules
//! only: invented types and numbers, no game data. Ledger:
//! `docs/m8-claims/intake116.md`.

mod econ_common;
mod structures_common;

use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{BuildingType, LocomotionClass, Locomotor, UnitType};
use glasnost_sim::behaviour::Door;
use glasnost_sim::econ::release::{self, Outcome};
use glasnost_sim::econ::{TypeRef, placement, sell};
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::structures::ownership::{self, Cause};
use glasnost_sim::world::{AttackMoveMemory, HouseId, ObjectId, Target, World};
use glasnost_sim::{depot, objects, testkit, tick};

const SIZE: i32 = 24;
const A: HouseId = HouseId(0);
const B: HouseId = HouseId(1);

fn step(w: &mut World) {
    tick::step(w);
}

fn beh(w: &World, id: ObjectId) -> (Option<Behaviour>, Option<Behaviour>) {
    let o = w.obj(id).unwrap();
    (o.beh.current, o.beh.queued)
}

fn vehicles(w: &World) -> usize {
    w.objects
        .values()
        .filter(|o| o.kind == Kind::Vehicle)
        .count()
}

// ---------------------------------------------------------------------------
// A building being sold, and one being built up.

/// A build-up building type of cost 1000 with idle range `idle` (`AnimIdle`
/// start, count, delay; absent: the default range of delay 0). n = 5
/// build-up frames: the build-down lasts 36 ticks (S-0108 claim 5).
fn hut(idle: Option<[i32; 3]>) -> BuildingType {
    let mut b = BuildingType {
        strength: Some(400),
        cost: Some(1000),
        foundation: Some([1, 1]),
        owner: Some(vec!["Alphaland".into(), "Betaland".into()]),
        ..Default::default()
    };
    b.econ.buildup = Some(true);
    b.econ.buildup_frames = Some(10);
    b.econ.anim_idle = idle;
    b
}

/// HUT (absent idle range), HUT3 (idle range of 3 stages, delay 4), HUT1
/// (idle range of 1 stage, delay 4) and FREE (absent idle range, free unit
/// CART).
fn work_rules() -> Ruleset {
    let mut r = testkit::ruleset();
    let names = ["HUT", "HUT3", "HUT1", "FREE"];
    r.registries.insert(
        "buildings".into(),
        names.iter().map(|s| s.to_string()).collect(),
    );
    let s = &mut r.sim;
    s.buildings.insert("HUT".into(), hut(None));
    s.buildings.insert("HUT3".into(), hut(Some([0, 3, 4])));
    s.buildings.insert("HUT1".into(), hut(Some([0, 1, 4])));
    let mut free = hut(None);
    free.econ.free_unit = Some("CART".into());
    s.buildings.insert("FREE".into(), free);
    r
}

fn work_game(r: &Ruleset) -> World {
    let mut w = testkit::game_with_non_players(r, SIZE, 13);
    testkit::run(&mut w, 4);
    for h in &mut w.houses {
        h.credits = 10_000;
    }
    w
}

fn mid(dx: i32, dy: i32) -> CellXY {
    testkit::mid(SIZE, dx, dy)
}

fn hut_ref(w: &World, name: &str) -> TypeRef {
    TypeRef {
        kind: Kind::Building,
        index: w.rules().index_of(Kind::Building, name).unwrap(),
    }
}

fn stage(w: &World, b: ObjectId) -> Option<sell::Stage> {
    w.obj(b).unwrap().econ.sale.clone().map(|s| s.stage)
}

/// Sells Alpha's `b` and runs until its build-down has begun; the tick
/// *D* of that start.
fn into_builddown(w: &mut World, b: ObjectId) -> i32 {
    assert!(orders::issue(w, A, Order::Sell { actor: b }));
    for _ in 0..10 {
        step(w);
        if let Some(sell::Stage::BuildDown(d)) = stage(w, b) {
            return d;
        }
    }
    panic!("no build-down");
}

/// Runs until `b` is removed, at most `n` ticks; the tick of the step that
/// removed it.
fn removal(w: &mut World, b: ObjectId, n: i32) -> Option<i32> {
    for _ in 0..n {
        let t = w.tick;
        step(w);
        if w.obj(b).is_none() {
            return Some(t);
        }
    }
    None
}

/// A sale of a `name` building whose owner a trigger's change gives to
/// Beta three ticks into its build-down. Returns the world, the building,
/// the build-down start *D* and the tick of the change.
fn changed_in_builddown(r: &Ruleset, name: &str) -> (World, ObjectId, i32, i32) {
    let mut w = work_game(r);
    let b = testkit::spawn(&mut w, Kind::Building, name, 0, mid(0, 0), 0);
    let d = into_builddown(&mut w, b);
    testkit::run(&mut w, 3);
    let t = w.tick;
    ownership::change(&mut w, b, Some(B), Cause::Trigger);
    assert_eq!(beh(&w, b).0, Some(Behaviour::Selling), "it keeps selling");
    (w, b, d, t)
}

// spec: S-0309 claim 31 (a building sold when it changes owner: the fall-back's idle-body request replaces the build-down; with an idle range of stage delay 0, the absent-key range, the sale ends at the building's next turn, cut short; the refund goes to the house that owns it when the sale completes); scenario A19 part j
#[test]
fn a_zero_delay_idle_range_ends_the_sale_at_the_next_turn() {
    let (mut w, b, d, t) = changed_in_builddown(&work_rules(), "HUT");
    assert!(
        w.obj(b).unwrap().econ.idle_body.is_some(),
        "the idle body is requested"
    );
    let (alpha, beta) = (w.houses[0].credits, w.houses[1].credits);
    let refund = sell::refund(&w, B, hut_ref(&w, "HUT"));
    let r = removal(&mut w, b, 60).expect("removed");
    assert_eq!(r, t, "the building's next turn");
    assert!(r < d + 36, "the build-down is cut short");
    assert_eq!(w.houses[1].credits, beta + refund, "the new owner's refund");
    assert_eq!(w.houses[0].credits, alpha, "nothing to the old owner");
}

// spec: S-0309 claim 31 (an idle range of c ≥ 2 stages with a positive delay d: the sale ends on the turn on which the range lands on its last stage, (c − 1) × N(d, s) ticks after the replacement, S-0215 claim 11's rule setting the mark)
#[test]
fn a_staged_idle_range_ends_the_sale_on_its_last_stage() {
    let (mut w, b, d, t) = changed_in_builddown(&work_rules(), "HUT3");
    let n = glasnost_sim::infantry::normalised(4, w.game_speed);
    let r = removal(&mut w, b, 80).expect("removed");
    assert_eq!(r, t + 2 * n, "(c − 1) × N(d, s) after the replacement");
    assert!(r < d + 36);
}

// spec: S-0309 claim 31 (the sale ends at the first selling processing that finds the ready mark set by the idle range: a mark already set when the range replaces the build-down, as for an armed building, does not end it)
#[test]
fn a_mark_set_before_the_replacement_does_not_end_the_sale() {
    let (mut w, b, _, t) = changed_in_builddown(&work_rules(), "HUT3");
    w.obj_mut(b).unwrap().beh.ready_mark = true;
    let n = glasnost_sim::infantry::normalised(4, w.game_speed);
    assert_eq!(removal(&mut w, b, 80), Some(t + 2 * n));
}

// spec: S-0309 claim 31 (an idle range of one stage with a positive delay never sets the mark, so the sale stalls in selling with no refund and no removal, as S-0408 claim 17 gives for a build-down range of one stage)
#[test]
fn a_one_stage_idle_range_stalls_the_sale() {
    let (mut w, b, _, _) = changed_in_builddown(&work_rules(), "HUT1");
    let beta = w.houses[1].credits;
    assert_eq!(removal(&mut w, b, 200), None, "never removed");
    assert_eq!(beh(&w, b).0, Some(Behaviour::Selling));
    assert_eq!(w.houses[1].credits, beta, "no refund");
}

// spec: S-0309 claim 31 (with the change before the build-down starts, the idle-body request takes effect before that start or is cancelled by it, an immediate change: either way the build-down runs its whole range and the sale ends in the tick in which it would have ended without the change, S-0408 claim 17)
#[test]
fn a_change_before_the_builddown_leaves_the_sale_unchanged() {
    let run = |change: bool| {
        let mut w = work_game(&work_rules());
        let b = testkit::spawn(&mut w, Kind::Building, "HUT", 0, mid(0, 0), 0);
        assert!(orders::issue(&mut w, A, Order::Sell { actor: b }));
        for _ in 0..10 {
            if stage(&w, b) == Some(sell::Stage::Begun) {
                break;
            }
            step(&mut w);
        }
        assert_eq!(stage(&w, b), Some(sell::Stage::Begun));
        if change {
            ownership::change(&mut w, b, Some(B), Cause::Trigger);
            assert!(w.obj(b).unwrap().econ.idle_body.is_some(), "requested");
        }
        let credits = w.houses[usize::from(if change { 1u8 } else { 0 })].credits;
        let r = removal(&mut w, b, 80).expect("removed");
        let paid = w.houses[usize::from(if change { 1u8 } else { 0 })].credits - credits;
        (r, paid)
    };
    let (plain, _) = run(false);
    let (changed, paid) = run(true);
    assert_eq!(changed, plain, "the same removal tick");
    let w = work_game(&work_rules());
    assert_eq!(paid, sell::refund(&w, B, hut_ref(&w, "HUT")));
}

// spec: S-0309 Edge cases (quirks.owner_change_interrupts_building_work off: a building selling at the change gets neither the guard request nor the fall-back, so its build-down runs on under the new owner by S-0408)
#[test]
fn quirk_interrupts_building_work_off_keeps_the_builddown() {
    let mut r = work_rules();
    r.globals.quirks.owner_change_interrupts_building_work = false;
    let (mut w, b, d, _) = changed_in_builddown(&r, "HUT");
    assert_eq!(w.obj(b).unwrap().econ.idle_body, None, "no idle request");
    assert_eq!(beh(&w, b), (Some(Behaviour::Selling), None));
    let beta = w.houses[1].credits;
    let refund = sell::refund(&w, B, hut_ref(&w, "HUT"));
    assert_eq!(removal(&mut w, b, 80), Some(d + 36), "the whole range");
    assert_eq!(w.houses[1].credits, beta + refund);
}

/// A FREE building of Alpha's placed in play (by order) at the middle of
/// the map: it enters with *construction* queued (S-0406 claim 17).
fn placed(w: &mut World) -> ObjectId {
    let t = w.rules().index_of(Kind::Building, "FREE").unwrap();
    let id = objects::create(w, Kind::Building, t, Some(A));
    placement::put_on_map(w, id, mid(0, 0), true);
    id
}

fn building_up(w: &World, b: ObjectId) -> bool {
    w.obj(b).unwrap().econ.construction.building_up
}

/// Checks the grand opening of the change: the ready mark set and the
/// opening recorded (the slot animations' start).
fn assert_opened(w: &World, b: ObjectId) {
    let o = w.obj(b).unwrap();
    assert!(o.econ.construction.opened, "the grand opening");
    assert!(o.beh.ready_mark, "its ready mark is set");
}

/// Runs 200 ticks and checks that the building never has *construction*
/// current again and that no free unit appears.
fn assert_no_free_unit(w: &mut World, b: ObjectId) {
    let before = vehicles(w);
    for _ in 0..200 {
        step(w);
        assert_ne!(beh(w, b).0, Some(Behaviour::Construction));
    }
    assert_eq!(vehicles(w), before, "no free unit, now or later");
}

// spec: S-0402 claim 18 (control: the grand opening at the end of an ordinary build-up gives the free unit)
#[test]
fn control_an_ordinary_build_up_gives_the_free_unit() {
    let mut w = work_game(&work_rules());
    let b = placed(&mut w);
    let before = vehicles(&w);
    testkit::run(&mut w, 200);
    assert!(!building_up(&w, b));
    assert_eq!(vehicles(&w), before + 1);
}

// spec: S-0309 claim 31 (a building being built up with construction still queued: the guard request replaces it, so it never becomes current; the grand opening at the change; guard current at the building's next turn's second per-tick start; the idle body from the change's request at the end of that turn ends the construction state; no free unit); scenario A19 part m
#[test]
fn construction_queued_is_replaced_by_guard() {
    let mut w = work_game(&work_rules());
    let b = placed(&mut w);
    testkit::run(&mut w, 3);
    assert_eq!(beh(&w, b), (None, Some(Behaviour::Construction)));
    ownership::change(&mut w, b, Some(B), Cause::Trigger);
    assert_ne!(beh(&w, b).1, Some(Behaviour::Construction), "replaced");
    assert_opened(&w, b);
    step(&mut w);
    assert_eq!(beh(&w, b).0, Some(Behaviour::Guard));
    assert!(!building_up(&w, b), "the construction state has ended");
    assert!(w.obj(b).unwrap().econ.idle_body.is_some());
    assert_no_free_unit(&mut w, b);
}

// spec: S-0309 claim 31 (construction current and processed at least once: its next processing finds the opening's ready mark and completes the construction at once, without a second grand opening; guard current at that turn's second start)
#[test]
fn construction_processed_completes_at_once() {
    let mut w = work_game(&work_rules());
    let b = placed(&mut w);
    for _ in 0..100 {
        if w.obj(b).unwrap().econ.construction.first.is_some() {
            break;
        }
        step(&mut w);
    }
    testkit::run(&mut w, 2);
    assert_eq!(beh(&w, b).0, Some(Behaviour::Construction));
    assert!(building_up(&w, b));
    ownership::change(&mut w, b, Some(B), Cause::Trigger);
    assert_opened(&w, b);
    step(&mut w);
    assert_eq!(beh(&w, b).0, Some(Behaviour::Guard));
    assert!(!building_up(&w, b), "completed at once");
    assert!(w.obj(b).unwrap().econ.idle_body.is_some());
    assert_no_free_unit(&mut w, b);
}

// spec: S-0309 claim 31 (construction current and not yet processed: its first processing restarts the build-up, an immediate change that cancels the idle-body request; guard current at that turn's second start, first processed on the next tick, S-0215 claim 3b, and the idle body follows from the request that guard's first processing makes, S-0108 claim 6)
#[test]
fn construction_not_yet_processed_restarts_then_guard() {
    let mut w = work_game(&work_rules());
    let b = placed(&mut w);
    for _ in 0..100 {
        if beh(&w, b).0 == Some(Behaviour::Construction) {
            break;
        }
        step(&mut w);
    }
    assert_eq!(beh(&w, b).0, Some(Behaviour::Construction));
    assert_eq!(w.obj(b).unwrap().econ.construction.first, None);
    ownership::change(&mut w, b, Some(B), Cause::Trigger);
    assert_opened(&w, b);
    step(&mut w);
    let o = w.obj(b).unwrap();
    assert_eq!(o.beh.current, Some(Behaviour::Guard));
    assert!(o.econ.construction.first.is_some(), "the restart");
    assert_eq!(o.econ.idle_body, None, "the request is cancelled");
    step(&mut w);
    let o = w.obj(b).unwrap();
    assert!(o.econ.idle_body.is_some(), "guard's request");
    assert!(!o.econ.construction.building_up);
    assert_no_free_unit(&mut w, b);
}

// spec: S-0309 claim 31 (a building that has already had its grand opening has none again: its ready mark is not set) and Edge cases (with quirks.owner_change_interrupts_building_work off the grand opening still acts)
#[test]
fn the_grand_opening_acts_once_and_whatever_the_switch() {
    let mut w = work_game(&work_rules());
    let b = testkit::spawn(&mut w, Kind::Building, "HUT", 0, mid(0, 0), 0);
    {
        let o = w.obj_mut(b).unwrap();
        o.beh.current = Some(Behaviour::Sleep);
        o.beh.queued = None;
        o.beh.ready_mark = false;
    }
    ownership::change(&mut w, b, Some(B), Cause::Trigger);
    let o = w.obj(b).unwrap();
    assert!(!o.econ.construction.opened);
    assert!(!o.beh.ready_mark, "no second opening");
    let mut r = work_rules();
    r.globals.quirks.owner_change_interrupts_building_work = false;
    let mut w = work_game(&r);
    let b = placed(&mut w);
    testkit::run(&mut w, 3);
    ownership::change(&mut w, b, Some(B), Cause::Trigger);
    assert_opened(&w, b);
}

// ---------------------------------------------------------------------------
// A vehicle factory.

/// The production ruleset with VF, a ground vehicle factory (vehicle
/// factory flag, 3 × 2 foundation, an invented door time).
fn vf_rules() -> Ruleset {
    let mut r = econ_common::ruleset();
    let mut b = econ_common::factory("vehicle");
    b.foundation = Some([3, 2]);
    b.weapons_factory = Some(true);
    b.deploy_time = econ_common::dec("0.01");
    r.registries.get_mut("buildings").unwrap().push("VF".into());
    r.sim.buildings.insert("VF".into(), b);
    r
}

fn new_unit(w: &mut World, name: &str, h: HouseId) -> ObjectId {
    let ti = w.rules().index_of(Kind::Vehicle, name).unwrap();
    objects::create(w, Kind::Vehicle, ti, Some(h))
}

/// A world with Alpha's VF, past the order block, that has released a
/// TANK (its unloading queued).
fn factory_released(r: &Ruleset) -> (World, ObjectId, ObjectId) {
    let mut w = econ_common::game(r);
    econ_common::ready(&mut w);
    let k = econ_common::building(&mut w, "VF", 0, 0, 0);
    let tank = new_unit(&mut w, "TANK", A);
    assert_eq!(release::release(&mut w, k, tank), Outcome::Released);
    assert_eq!(beh(&w, k).1, Some(Behaviour::Unload));
    (w, k, tank)
}

fn door(w: &World, k: ObjectId) -> Door {
    w.obj(k).unwrap().beh.gate.door
}

// spec: S-0309 claim 31 (a vehicle factory's queued unload is replaced by the guard request, so that unloading never starts; the unit it has released and not yet sent out follows the factory, keeps its contact and tether link and stays at its exit point; r129: a later release by the factory puts its new unit there all the same and takes the contact place, S-0404 claim 13); scenario A19 part k
#[test]
fn a_queued_unload_is_replaced_and_the_unit_is_stranded() {
    let (mut w, k, tank) = factory_released(&vf_rules());
    let p = w.obj(tank).unwrap().pos;
    ownership::change(&mut w, k, Some(B), Cause::Trigger);
    assert_ne!(beh(&w, k).1, Some(Behaviour::Unload), "replaced");
    assert_eq!(structures_common::owner(&w, tank), Some(B), "it follows");
    for _ in 0..60 {
        step(&mut w);
        assert_ne!(beh(&w, k).0, Some(Behaviour::Unload), "never starts");
    }
    let o = w.obj(tank).unwrap();
    assert_eq!(o.pos, p, "it stays at its exit point");
    assert_eq!(o.contact, Some(k), "and keeps its contact");
    assert!(release::tethered(&w, tank), "and its tether link");
    let jeep = new_unit(&mut w, "JEEP", B);
    assert_eq!(release::release(&mut w, k, jeep), Outcome::Released);
    assert_eq!(w.obj(jeep).unwrap().pos, p, "the new unit shares the cell");
    assert_eq!(w.obj(tank).unwrap().contact, None, "the contact is taken");
    assert!(!release::tethered(&w, tank));
    assert_eq!(w.obj(tank).unwrap().pos, p, "the unit stays");
}

// spec: S-0309 claim 31 (a factory ready at the guard request has guard current at once and its unloading ends at the change; the change does not close the door, which stays open until a later unloading's closing stage; an order that moves the stranded unit while it still holds the contact ends its contact at its clear report, r129; the later unloading finds the door open and does not wait for it)
#[test]
fn an_ended_unloading_leaves_the_door_open() {
    let (mut w, k, tank) = factory_released(&vf_rules());
    for _ in 0..100 {
        if door(&w, k) == Door::Open {
            break;
        }
        step(&mut w);
    }
    assert_eq!(door(&w, k), Door::Open);
    assert_eq!(beh(&w, k).0, Some(Behaviour::Unload));
    let p = w.obj(tank).unwrap().pos;
    ownership::change(&mut w, k, Some(B), Cause::Trigger);
    assert_eq!(beh(&w, k).0, Some(Behaviour::Guard), "unloading ends");
    for _ in 0..100 {
        step(&mut w);
        assert_eq!(door(&w, k), Door::Open, "the door stays open");
    }
    assert_eq!(w.obj(tank).unwrap().pos, p, "the unit stays");
    let far = Target::Cell(testkit::mid(econ_common::SIZE, 8, -6));
    assert!(orders::issue(
        &mut w,
        B,
        Order::Behaviour {
            actor: tank,
            behaviour: Behaviour::Move,
            attack_move: false,
            target: None,
            dest: Some(far),
        }
    ));
    let mut left = false;
    for _ in 0..600 {
        step(&mut w);
        if w.obj(tank).unwrap().contact.is_none() && w.obj(tank).unwrap().pos.cell() != p.cell() {
            left = true;
            break;
        }
    }
    assert!(left, "the order ends its contact");
    let jeep = new_unit(&mut w, "JEEP", B);
    assert_eq!(release::release(&mut w, k, jeep), Outcome::Released);
    let mut sent = false;
    for _ in 0..300 {
        step(&mut w);
        let on_track = w
            .obj(jeep)
            .unwrap()
            .drive
            .as_ref()
            .is_some_and(|d| d.track.as_ref().is_some_and(|t| t.exit));
        if on_track {
            sent = true;
            break;
        }
        assert!(
            !matches!(door(&w, k), Door::Opening { .. }),
            "no wait for the door"
        );
    }
    assert!(sent, "the next unit is sent out");
}

// spec: S-0309 Edge cases (quirks.owner_change_interrupts_building_work off: a vehicle factory with unload queued gets neither the guard request nor the fall-back; the unloading goes on under the new owner by S-0417 and the factory sends its unit out)
#[test]
fn quirk_interrupts_building_work_off_sends_the_unit_out() {
    let mut r = vf_rules();
    r.globals.quirks.owner_change_interrupts_building_work = false;
    let (mut w, k, tank) = factory_released(&r);
    ownership::change(&mut w, k, Some(B), Cause::Trigger);
    assert_eq!(beh(&w, k).1, Some(Behaviour::Unload), "kept");
    let mut sent = false;
    for _ in 0..600 {
        step(&mut w);
        if w.obj(tank)
            .unwrap()
            .drive
            .as_ref()
            .is_some_and(|d| d.track.as_ref().is_some_and(|t| t.exit))
        {
            sent = true;
            break;
        }
    }
    assert!(sent, "the factory sends its unit out");
}

// ---------------------------------------------------------------------------
// A repair depot.

const DSIZE: i32 = 30;

/// The combat ruleset with TRUCK (an unarmed wheeled vehicle, strength
/// 600) and DEPOT (a 3 × 3 repair depot whose cells let vehicles through,
/// one place, a build-up sprite); repair step 10 at 25 %.
fn depot_rules() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    r.registries
        .entry("vehicles".into())
        .or_default()
        .push("TRUCK".into());
    r.registries
        .entry("buildings".into())
        .or_default()
        .push("DEPOT".into());
    let s = &mut r.sim;
    s.vehicles.insert(
        "TRUCK".into(),
        UnitType {
            strength: Some(600),
            speed: Some(8),
            rot: Some(8),
            locomotion_class: Some(LocomotionClass::Wheel),
            locomotor: Some(Locomotor::Drive),
            cost: Some(900),
            ..Default::default()
        },
    );
    let mut d = BuildingType {
        strength: Some(800),
        foundation: Some([3, 3]),
        unit_repair: Some(true),
        number_impassable_rows: Some(0),
        cost: Some(800),
        ..Default::default()
    };
    d.econ.buildup = Some(true);
    s.buildings.insert("DEPOT".into(), d);
    s.general.repair_step = Some(10);
    s.general.repair_percent = Some(glasnost_rules::sim::Decimal("25%".into()));
    r
}

/// A damaged truck sent to Alpha's depot, run until it is in contact
/// asleep and the depot repairs it.
fn depot_scene(r: &Ruleset) -> (World, ObjectId, ObjectId) {
    let mut w = testkit::game_with(r, DSIZE, 5);
    for h in &mut w.houses {
        h.credits = 10_000;
    }
    let d = testkit::spawn(
        &mut w,
        Kind::Building,
        "DEPOT",
        0,
        testkit::mid(DSIZE, 0, 0),
        0,
    );
    let v = testkit::spawn(
        &mut w,
        Kind::Vehicle,
        "TRUCK",
        0,
        testkit::mid(DSIZE, 6, 1),
        0,
    );
    w.obj_mut(v).unwrap().health = 300;
    while w.tick < 4 {
        step(&mut w);
    }
    assert!(orders::issue(
        &mut w,
        A,
        Order::Behaviour {
            actor: v,
            behaviour: Behaviour::Enter,
            attack_move: false,
            target: None,
            dest: Some(Target::Object(d)),
        }
    ));
    let mut ok = false;
    for _ in 0..600 {
        step(&mut w);
        if w.obj(v).unwrap().contact == Some(d)
            && beh(&w, d).0 == Some(Behaviour::Repair)
            && beh(&w, v).0 == Some(Behaviour::Sleep)
        {
            ok = true;
            break;
        }
    }
    assert!(ok, "the truck is serviced");
    (w, d, v)
}

// spec: S-0309 claim 31 (the docked vehicle follows the depot, its guard request finds it ready so guard becomes current, and its ordinary fall-back leaves it where it stands, docked and holding its contact place until an order ends the contact; a depot not ready at the change goes on repairing until a per-tick start makes its guard current, which here is its next start, before its next repair step, so no further step is taken); scenario A19 part l
#[test]
fn a_depot_stops_repairing_and_the_vehicle_stays_docked() {
    let (mut w, d, v) = depot_scene(&depot_rules());
    let p = w.obj(v).unwrap().pos;
    let h = w.obj(v).unwrap().health;
    // Between two ticks the depot's mark has been used by its last start,
    // so the change finds it not ready: guard is queued, and the depot's
    // next per-tick start makes it current before any repair step.
    assert!(!w.obj(d).unwrap().beh.ready_mark);
    ownership::change(&mut w, d, Some(B), Cause::Trigger);
    assert_eq!(
        beh(&w, d),
        (Some(Behaviour::Repair), Some(Behaviour::Guard))
    );
    assert_eq!(structures_common::owner(&w, v), Some(B), "it follows");
    assert_eq!(beh(&w, v).0, Some(Behaviour::Guard));
    step(&mut w);
    assert_eq!(beh(&w, d).0, Some(Behaviour::Guard), "repair ends");
    for _ in 0..300 {
        step(&mut w);
    }
    let o = w.obj(v).unwrap();
    assert_eq!(o.health, h, "no further repair step");
    assert_eq!(o.contact, Some(d), "still docked");
    assert_eq!(o.pos, p, "where it stands");
    assert_ne!(beh(&w, d).0, Some(Behaviour::Repair));
    assert!(depot::is_depot(&w, d));
}

// spec: S-0309 claim 31 (a depot ready at the change, as within a turn after its body range has set the mark: its guard request ends its repair at the change)
#[test]
fn a_ready_depot_ends_its_repair_inside_the_change() {
    let (mut w, d, _) = depot_scene(&depot_rules());
    // Set-up: the state of a turn after its body range set the mark and
    // before its start.
    w.obj_mut(d).unwrap().beh.ready_mark = true;
    ownership::change(&mut w, d, Some(B), Cause::Trigger);
    assert_eq!(beh(&w, d).0, Some(Behaviour::Guard));
}

// spec: S-0309 Edge cases (quirks.owner_change_interrupts_building_work off: a depot whose current behaviour is repair gets neither the guard request nor the fall-back, and its repair goes on under the new owner by S-0314)
#[test]
fn quirk_interrupts_building_work_off_keeps_the_repair() {
    let mut r = depot_rules();
    r.globals.quirks.owner_change_interrupts_building_work = false;
    let (mut w, d, _) = depot_scene(&r);
    ownership::change(&mut w, d, Some(B), Cause::Trigger);
    assert_eq!(beh(&w, d), (Some(Behaviour::Repair), None), "kept");
}

// ---------------------------------------------------------------------------
// A psychic domination, and an infantry unit's arrival.

/// An engaged CRUSHER of Alpha's with an attack-move memory to a
/// remembered destination, a destination and a guard spot.
fn engaged(w: &mut World) -> ObjectId {
    let v = structures_common::put(w, Kind::Vehicle, "CRUSHER", 0, 0, 0);
    let o = w.obj_mut(v).unwrap();
    o.attack_move = Some(AttackMoveMemory::Dest(Target::Cell(structures_common::at(
        8, 0,
    ))));
    o.combat.engaged = true;
    o.dest = Some(Target::Cell(structures_common::at(4, 4)));
    o.guard_spot = Some(Target::Cell(structures_common::at(6, 6)));
    // Set-up: the placement's fall-back used this tick's refresh.
    o.beh.last_common_effects = None;
    v
}

// spec: S-0309 claims 30 and 31 (a psychic domination is a change of claim 30's list and resets the orders: the guard spot forgotten and the fall-back's refresh resuming an engaged attack move; it adds no guard reset, which would clear the resumed destination); scenario A19 part n
#[test]
fn a_domination_resets_the_orders_without_a_guard_reset() {
    let mut w = structures_common::game();
    let v = engaged(&mut w);
    assert!(glasnost_sim::mind::dominate(&mut w, v, B));
    let o = w.obj(v).unwrap();
    assert_eq!(o.owner, Some(B));
    assert_eq!(o.guard_spot, None, "the guard spot is forgotten");
    assert_eq!(
        o.dest,
        Some(Target::Cell(structures_common::at(8, 0))),
        "the resumed destination is kept: no guard reset"
    );
}

// spec: S-0309 claim 31 (a change to the house the object already belongs to does none of this: a domination by its own house resets nothing)
#[test]
fn a_domination_by_the_owner_resets_nothing() {
    let mut w = structures_common::game();
    let v = engaged(&mut w);
    assert!(glasnost_sim::mind::dominate(&mut w, v, A));
    let o = w.obj(v).unwrap();
    assert_eq!(o.dest, Some(Target::Cell(structures_common::at(4, 4))));
    assert_eq!(
        o.guard_spot,
        Some(Target::Cell(structures_common::at(6, 6)))
    );
}

// spec: S-0309 claim 23 (d) (an infantry unit's arrival is caught by S-0215 claim 22's end-of-update check, at the end of the update whose locomotion ends the walk: the fall-back there removes the memory and queues the rest behaviour, current at the first start of the next tick if the unit is ready)
#[test]
fn an_infantry_arrival_falls_back_at_the_end_of_its_update() {
    let mut w = structures_common::game();
    let u = structures_common::put(&mut w, Kind::Infantry, "WALKER", 0, 0, 0);
    let here = Target::Cell(structures_common::at(2, 0));
    assert!(orders::issue(
        &mut w,
        A,
        Order::Behaviour {
            actor: u,
            behaviour: Behaviour::Move,
            attack_move: true,
            target: None,
            dest: Some(here),
        }
    ));
    step(&mut w);
    assert!(w.obj(u).unwrap().attack_move.is_some(), "the memory");
    let mut caught = false;
    for _ in 0..300 {
        let walking = w.obj(u).unwrap().dest.is_some();
        step(&mut w);
        let o = w.obj(u).unwrap();
        if o.attack_move.is_none() {
            assert!(walking, "in the update that ends the walk");
            assert_eq!(o.dest, None);
            assert!(
                matches!(o.beh.queued, Some(Behaviour::Guard | Behaviour::AreaGuard))
                    || matches!(o.beh.current, Some(Behaviour::Guard | Behaviour::AreaGuard)),
                "the rest behaviour: {:?} {:?}",
                o.beh.current,
                o.beh.queued
            );
            caught = true;
            break;
        }
    }
    assert!(caught, "the memory is removed at the arrival");
    step(&mut w);
    assert!(matches!(
        w.obj(u).unwrap().beh.current,
        Some(Behaviour::Guard | Behaviour::AreaGuard)
    ));
}
