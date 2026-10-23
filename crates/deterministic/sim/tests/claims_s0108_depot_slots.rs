//! Claim tests of the repair depot's slot model (S-0108 claim 20; S-0314
//! claims 6 and 22; S-0504 claim 8) on synthetic data: invented types,
//! slot names, frame counts and costs, no game data.

use std::collections::BTreeMap;

use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{AnimationType, BuildingType, LocomotionClass, Locomotor, UnitType};
use glasnost_sim::combat::damage::{self, Hit};
use glasnost_sim::combat::effects::Effect;
use glasnost_sim::depot::{self, DepotSlot};
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::world::{HouseId, ObjectId, Target, World};
use glasnost_sim::{depot_anim, save, testkit, tick};

/// One tick, its effects alone in the list afterwards.
fn step(w: &mut World) {
    w.effects.clear();
    tick::step(w);
}

const SIZE: i32 = 30;
const ALPHA: HouseId = HouseId(0);
/// The slot types' image-frame counts: special-1 short, special-2
/// endless, special-3 and production finite.
const SP1_FRAMES: i32 = 6;
const SP3_FRAMES: i32 = 12;
const PROD_FRAMES: i32 = 40;

fn anim(frames: i32, loop_count: Option<i32>) -> AnimationType {
    AnimationType {
        frame_count: Some(frames),
        middle_frame: Some(frames / 2),
        loop_count,
        ..Default::default()
    }
}

/// The combat ruleset plus TRUCK (an unarmed wheeled vehicle of strength
/// 600), DEPOT (a 3 × 3 repair depot with slot names PROD, SP1, SP2, SP3,
/// damaged SP2D for special-2) and NDEPOT (the same, naval); repair step
/// 10 at 25 %.
fn rules() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    let mut push = |reg: &str, n: &str| {
        r.registries.entry(reg.into()).or_default().push(n.into());
    };
    push("vehicles", "TRUCK");
    push("buildings", "DEPOT");
    push("buildings", "NDEPOT");
    for n in ["PROD", "SP1", "SP2", "SP2D", "SP3", "RND"] {
        push("animations", n);
    }
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
    s.animations.insert("PROD".into(), anim(PROD_FRAMES, None));
    s.animations.insert("SP1".into(), anim(SP1_FRAMES, None));
    // spec: S-0109 Import mapping (LoopCount −1: endless)
    s.animations.insert("SP2".into(), anim(8, Some(-1)));
    s.animations.insert("SP2D".into(), anim(8, Some(-1)));
    s.animations.insert("SP3".into(), anim(SP3_FRAMES, None));
    let mut rnd = anim(SP1_FRAMES, None);
    rnd.random_rate = Some([300, 100]);
    s.animations.insert("RND".into(), rnd);
    let mut depot = BuildingType {
        strength: Some(800),
        foundation: Some([3, 3]),
        unit_repair: Some(true),
        number_impassable_rows: Some(0),
        cost: Some(800),
        ..Default::default()
    };
    let pair = |n: &str, d: &str| [n.to_owned(), d.to_owned()];
    depot.econ.slot_names = Some(BTreeMap::from([
        ("ProductionAnim".to_owned(), pair("PROD", "PROD")),
        ("SpecialAnim".to_owned(), pair("SP1", "SP1")),
        ("SpecialAnimTwo".to_owned(), pair("SP2", "SP2D")),
        ("SpecialAnimThree".to_owned(), pair("SP3", "SP3")),
    ]));
    let mut naval = depot.clone();
    naval.naval = Some(true);
    s.buildings.insert("DEPOT".into(), depot);
    s.buildings.insert("NDEPOT".into(), naval);
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

fn spawn(w: &mut World, kind: Kind, name: &str, dx: i32, dy: i32) -> ObjectId {
    testkit::spawn(w, kind, name, 0, testkit::mid(SIZE, dx, dy), 0)
}

fn run_until(w: &mut World, n: i32, mut f: impl FnMut(&World) -> bool) -> bool {
    for _ in 0..n {
        if f(w) {
            return true;
        }
        step(w);
    }
    f(w)
}

fn beh(w: &World, k: ObjectId) -> (Option<Behaviour>, i32) {
    let b = &w.obj(k).unwrap().beh;
    (b.current, b.progress)
}

fn runs(w: &World, k: ObjectId, s: DepotSlot) -> bool {
    depot::slot_running(w, k, s)
}

/// The presentation slot reports of the last tick: (active-1, start).
fn reports(w: &World, k: ObjectId) -> Vec<(bool, bool)> {
    w.effects
        .iter()
        .filter_map(|e| match e {
            Effect::DepotSlot {
                building,
                active1,
                start,
                ..
            } if *building == k => Some((*active1, *start)),
            _ => None,
        })
        .collect()
}

/// A damaged truck sent into a fresh depot of type `name`, run until the
/// depot is in its repair behaviour with the truck in contact.
fn scene(name: &str, health_at: i32) -> (World, ObjectId, ObjectId) {
    scene_with(&rules(), name, health_at)
}

/// [`scene`] on the given rules.
fn scene_with(r: &Ruleset, name: &str, health_at: i32) -> (World, ObjectId, ObjectId) {
    let mut w = game(r);
    let k = spawn(&mut w, Kind::Building, name, 0, 0);
    let v = spawn(&mut w, Kind::Vehicle, "TRUCK", 6, 1);
    w.obj_mut(v).unwrap().health = health_at;
    while w.tick < 4 {
        step(&mut w);
    }
    let o = Order::Behaviour {
        actor: v,
        behaviour: Behaviour::Enter,
        attack_move: false,
        target: None,
        dest: Some(Target::Object(k)),
    };
    assert!(orders::issue(&mut w, ALPHA, o));
    assert!(
        run_until(&mut w, 600, |w| w.obj(v).unwrap().contact == Some(k)
            && w.obj(k).unwrap().beh.current == Some(Behaviour::Repair)),
        "the truck arrives and the depot starts repair"
    );
    (w, k, v)
}

/// Runs until the depot enters working (phase 2); returns that tick.
fn to_working(w: &mut World, k: ObjectId) -> i32 {
    assert!(run_until(w, 600, |w| beh(w, k).1 == 2), "working reached");
    w.tick - 1
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

// spec: S-0108 claim 20 (the repair cycle: entering the working phase, on the tick of the first paid repair step, starts special-1 and stops active-1 and idle; docking and the waiting check change no slot)
#[test]
fn claim_20_working_entry_starts_special_1() {
    let (mut w, k, v) = scene("DEPOT", 300);
    assert!(!runs(&w, k, DepotSlot::Special1), "docking starts no slot");
    let h = w.obj(v).unwrap().health;
    to_working(&mut w, k);
    assert!(w.obj(v).unwrap().health > h, "the first paid step");
    assert!(runs(&w, k, DepotSlot::Special1));
    assert_eq!(reports(&w, k), vec![(true, false), (false, false)]);
}

// spec: S-0108 claim 20 (special-1 ending while the depot is in its repair behaviour and in contact starts special-2; a working step that answers more needed changes no slot)
#[test]
fn claim_20_special_1_ends_into_special_2() {
    let (mut w, k, _) = scene("DEPOT", 300);
    to_working(&mut w, k);
    assert!(run_until(&mut w, 200, |w| !runs(w, k, DepotSlot::Special1)));
    assert!(runs(&w, k, DepotSlot::Special2));
    assert!(reports(&w, k).is_empty(), "no idle start while in contact");
    assert_eq!(beh(&w, k), (Some(Behaviour::Repair), 2));
}

// spec: S-0108 claim 20 (a working step that answers done: production and special-2 stop, special-3 and active-1 start on the tick of that step, before the contact is released; special-3 finishing on its own starts idle; After a send-away, Q-1620 answered: the next processing, 1 tick later, finds no contact in settling with neither production nor special-2 running, so it changes no slot)
#[test]
fn claim_20_done_ends_the_service() {
    let (mut w, k, v) = scene("DEPOT", 560);
    to_working(&mut w, k);
    assert!(run_until(&mut w, 600, |w| w
        .obj(v)
        .unwrap()
        .contact
        .is_none()));
    assert!(!runs(&w, k, DepotSlot::Special2));
    assert!(runs(&w, k, DepotSlot::Special3));
    assert_eq!(reports(&w, k), vec![(true, true)]);
    let started = w.tick - 1;
    step(&mut w);
    assert!(reports(&w, k).is_empty(), "no second change");
    let mut idle = None;
    for _ in 0..200 {
        step(&mut w);
        if reports(&w, k) == vec![(false, true)] {
            idle = Some(w.tick - 1);
            break;
        }
    }
    let idle = idle.expect("idle starts when special-3 finishes");
    assert!(!runs(&w, k, DepotSlot::Special3));
    assert!(
        idle - started >= SP3_FRAMES,
        "its duration from the art data"
    );
}

// spec: S-0314 claim 22 and S-0108 claim 20 (B18 g: a vehicle destroyed while working: the next processing enters settling and starts special-3; the settling depot stays in its repair behaviour, checking every tick, until special-3 no longer runs, then returns to guard)
#[test]
fn claim_22_settling_hold_end_to_end() {
    let (mut w, k, v) = scene("DEPOT", 300);
    to_working(&mut w, k);
    assert!(run_until(&mut w, 200, |w| runs(w, k, DepotSlot::Special2)));
    damage::apply(&mut w, v, lethal());
    assert!(run_until(&mut w, 200, |w| beh(w, k).1 == 1));
    assert!(runs(&w, k, DepotSlot::Special3));
    assert!(!runs(&w, k, DepotSlot::Special2));
    let mut end = None;
    let mut guard = None;
    for _ in 0..300 {
        step(&mut w);
        let t = w.tick - 1;
        if end.is_none() && !runs(&w, k, DepotSlot::Special3) {
            end = Some(t);
        }
        if beh(&w, k).0 == Some(Behaviour::Guard) {
            guard = Some(t);
            break;
        }
        assert_eq!(beh(&w, k), (Some(Behaviour::Repair), 1), "held in settling");
    }
    let (end, guard) = (end.expect("special-3 ends"), guard.expect("guard"));
    // The slot animation takes its turns after the depot (it entered the
    // update order later), so the depot first finds it gone the next tick.
    assert_eq!(guard, end + 1);
}

// spec: S-0314 claim 6 (while the depot's production slot animation runs, settling takes no repair step and rechecks after 1 tick); S-0108 claim 20 (nothing in the repair cycle starts production: the test starts it as a combined role would)
#[test]
fn claim_6_production_hold() {
    let (mut w, k, v) = scene("DEPOT", 300);
    assert!(run_until(&mut w, 600, |w| beh(w, k).1 == 1), "settling");
    depot_anim::start_for_test(&mut w, k, DepotSlot::Production);
    let h = w.obj(v).unwrap().health;
    let mut freed = None;
    for _ in 0..400 {
        step(&mut w);
        if !runs(&w, k, DepotSlot::Production) {
            freed = Some(w.tick - 1);
            break;
        }
        assert_eq!(w.obj(v).unwrap().health, h, "no step while it runs");
    }
    let freed = freed.expect("production ends");
    let step = run_until(&mut w, 200, |w| w.obj(v).unwrap().health > h);
    assert!(step, "the settling step once production is gone");
    assert!(w.tick - 1 - freed <= 2, "rechecked every tick");
}

// spec: S-0108 claim 20 (a repair depot that is not naval and not in its repair behaviour, with production or special-2 running: special-3 starts, production and special-2 stop, checked every tick; the naval row exception, S-0504 claim 8)
#[test]
fn claim_20_every_tick_row_and_the_naval_exception() {
    for (name, naval) in [("DEPOT", false), ("NDEPOT", true)] {
        let mut w = game(&rules());
        let k = spawn(&mut w, Kind::Building, name, 0, 0);
        step(&mut w);
        depot_anim::start_for_test(&mut w, k, DepotSlot::Special2);
        assert!(runs(&w, k, DepotSlot::Special2));
        step(&mut w);
        assert_eq!(runs(&w, k, DepotSlot::Special2), naval, "{name}");
        assert_eq!(runs(&w, k, DepotSlot::Special3), !naval, "{name}");
    }
}

// spec: S-0108 claims 12 and 25 (the damaged state differs from the slot condition: every running slot restarts with its damaged name, keeping its stage, not its image frame: the new animation shows its own start field plus that stage)
#[test]
fn claim_12_condition_swap_restarts_the_modelled_slots() {
    let mut r = rules();
    r.sim.animations.get_mut("SP2D").unwrap().start = Some(2);
    let (mut w, k, _) = scene_with(&r, "DEPOT", 300);
    to_working(&mut w, k);
    assert!(run_until(&mut w, 200, |w| runs(w, k, DepotSlot::Special2)
        && depot_anim::draws(w, k)[0].stage > 0));
    let before = depot_anim::draws(&w, k);
    assert_eq!(before[0].name, "SP2");
    w.obj_mut(k).unwrap().health = 100;
    step(&mut w);
    let after = depot_anim::draws(&w, k);
    assert_eq!(after[0].slot, DepotSlot::Special2);
    assert_eq!(after[0].name, "SP2D");
    // The swap comes in the depot's turn, before the old animation's turn of that tick, and the new one's first update makes no advance (S-0109 claim 5).
    assert_eq!(after[0].stage, before[0].stage, "the stage is kept");
    assert_eq!(
        after[0].frame,
        2 + before[0].stage,
        "its own start field plus the stage"
    );
}

// spec: S-0108 claim 25 (a start replacing a running animation of the same type: the stage is kept and the passes take their creation value, the replaced one's completed passes not carried over)
#[test]
fn claim_25_a_replacing_start_counts_its_passes_afresh() {
    let mut r = rules();
    r.sim.animations.get_mut("SP3").unwrap().loop_count = Some(3);
    let (mut w, k, _) = scene_with(&r, "DEPOT", 300);
    depot_anim::start_for_test(&mut w, k, DepotSlot::Special3);
    let passes = |w: &World| {
        let id = w.obj(k).unwrap().depot.slots[&DepotSlot::Special3];
        w.depot_anims.anims[&id].player.remaining_passes()
    };
    assert_eq!(passes(&w), Some(3));
    assert!(run_until(&mut w, 400, |w| passes(w) == Some(2)
        && depot_anim::draws(w, k)[0].stage > 0));
    let stage = depot_anim::draws(&w, k)[0].stage;
    depot_anim::start_for_test(&mut w, k, DepotSlot::Special3);
    assert_eq!(passes(&w), Some(3), "the passes afresh");
    assert_eq!(depot_anim::draws(&w, k)[0].stage, stage, "the stage kept");
}

// spec: S-0109 claims 28 and 29 and S-0202 claim 10 (a slot animation's creation takes its draws on the scenario stream at the creation: one ranged draw for a type with a random rate)
#[test]
fn a_slot_creation_draws_on_the_scenario_stream() {
    let mut r = rules();
    let names = r.sim.buildings.get_mut("DEPOT").unwrap();
    names
        .econ
        .slot_names
        .as_mut()
        .unwrap()
        .insert("SpecialAnimThree".into(), ["RND".into(), "RND".into()]);
    let mut w = game(&r);
    let k = spawn(&mut w, Kind::Building, "DEPOT", 0, 0);
    step(&mut w);
    let before = w.rng.clone();
    depot_anim::start_for_test(&mut w, k, DepotSlot::Special3);
    assert_ne!(w.rng, before, "a draw at the creation");
    let mut w2 = game(&r);
    let k2 = spawn(&mut w2, Kind::Building, "DEPOT", 0, 0);
    step(&mut w2);
    let before = w2.rng.clone();
    depot_anim::start_for_test(&mut w2, k2, DepotSlot::Special1);
    assert_eq!(w2.rng, before, "no draw for a fixed rate");
}

// spec: S-0108 claim 20 (the slot lifetime is deterministic simulation state: it survives a save and load, and the loaded game goes on alike)
#[test]
fn depot_slots_survive_a_save_and_load() {
    let (mut w, k, _) = scene("DEPOT", 300);
    to_working(&mut w, k);
    let bytes = save::save(&w);
    let h = save::state_hash(&w);
    let mut w2 = save::load(&bytes, w.rules_arc(), w.catalogue.clone().unwrap()).unwrap();
    assert_eq!(save::state_hash(&w2), h);
    for _ in 0..120 {
        step(&mut w);
        step(&mut w2);
    }
    assert_eq!(save::state_hash(&w2), save::state_hash(&w));
    assert_eq!(depot_anim::draws(&w2, k), depot_anim::draws(&w, k));
}

// spec: S-0108 claim 20 (After a send-away, Q-1620 answered: after a failed departure special-1 ends in contact and starts special-2; the later settling step that sends the vehicle away changes no slot; the following no-contact processing, D ticks later, finds special-2 running and so stops it and starts special-3 and active-1); S-0314 claim 6 (a settling send-away is followed D ticks later)
#[test]
fn claim_20_special_2_after_a_failed_departure() {
    let mut r = rules();
    // A special-1 long enough to outlast the working phase.
    r.sim.animations.get_mut("SP1").unwrap().frame_count = Some(160);
    let (mut w, k, v) = scene_with(&r, "DEPOT", 590);
    let list = glasnost_sim::econ::release::exit_list(3, 3, false, true);
    let mut blockers = Vec::new();
    for &(a, b) in &list {
        let c = testkit::mid(SIZE, a, b);
        if w.cells.footprint().is_legal(c)
            && !blockers
                .iter()
                .any(|&x: &ObjectId| w.obj(x).unwrap().pos.cell() == c)
        {
            blockers.push(spawn(&mut w, Kind::Vehicle, "TRUCK", a, b));
        }
    }
    to_working(&mut w, k);
    assert!(
        run_until(&mut w, 600, |w| beh(w, k).1 == 1),
        "the working step answers not needed and the departure fails"
    );
    assert_eq!(w.obj(v).unwrap().contact, Some(k), "contact kept");
    assert!(
        runs(&w, k, DepotSlot::Special1),
        "special-1 outlasts working"
    );
    assert!(
        run_until(&mut w, 900, |w| runs(w, k, DepotSlot::Special2)),
        "special-1 ends in contact: special-2"
    );
    assert_eq!(w.obj(v).unwrap().contact, Some(k));
    glasnost_sim::objects::remove(&mut w, blockers[0]);
    let mut sent = None;
    for _ in 0..200 {
        step(&mut w);
        if w.obj(v).unwrap().contact.is_none() {
            sent = Some(w.tick - 1);
            break;
        }
    }
    let sent = sent.expect("a later settling step sends it away");
    assert!(
        reports(&w, k).is_empty(),
        "the settling send-away changes no slot"
    );
    assert!(runs(&w, k, DepotSlot::Special2));
    let dd = w.rules().behaviour(Behaviour::Repair).rate_delay.max(1);
    let mut changed = None;
    for _ in 0..dd + 2 {
        step(&mut w);
        if !reports(&w, k).is_empty() {
            changed = Some((w.tick - 1, reports(&w, k)));
            break;
        }
    }
    let (t, rep) = changed.expect("the no-contact processing changes the slots");
    assert_eq!(t - sent, dd, "D ticks after the send-away");
    assert_eq!(rep, vec![(true, true)], "active-1 starts");
    assert!(!runs(&w, k, DepotSlot::Special2));
    assert!(runs(&w, k, DepotSlot::Special3));
}
