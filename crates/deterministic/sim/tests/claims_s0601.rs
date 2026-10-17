//! Claim tests for mind control (S-0601) on synthetic data: invented
//! types, weapons and warheads, every number our own. Houses Alpha (0) and
//! Beta (1) of the testkit's sandbox game, not allied; house 2 is the
//! neutral house of `testkit::game_with_non_players`.

use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{BuildingType, Decimal, UnitType, WarheadType, WeaponType};
use glasnost_sim::combat::check::{Answer, check};
use glasnost_sim::combat::damage::{self, Hit};
use glasnost_sim::combat::effects::Effect;
use glasnost_sim::mind::{self, MindSound};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::world::{HouseId, ObjectId, Pos, Target, World};
use glasnost_sim::{save, testkit};

const SIZE: i32 = 24;
const ALPHA: u8 = 0;
const BETA: u8 = 1;
/// The neutral house of `testkit::game_with_non_players`.
const NEUTRAL: u8 = 2;

fn dec(s: &str) -> Decimal {
    Decimal(s.into())
}

fn every(v: &str) -> Option<Vec<Decimal>> {
    Some(vec![dec(v); 11])
}

fn register(r: &mut Ruleset, reg: &str, name: &str) {
    let l = r.registries.entry(reg.into()).or_default();
    if !l.iter().any(|n| n == name) {
        l.push(name.into());
    }
}

fn vehicle(r: &mut Ruleset, name: &str, f: impl FnOnce(&mut UnitType)) {
    let mut t = r.sim.vehicles["BUGGY"].clone();
    t.strength = Some(1000);
    t.primary = None;
    f(&mut t);
    register(r, "vehicles", name);
    r.sim.vehicles.insert(name.into(), t);
}

fn infantry(r: &mut Ruleset, name: &str, f: impl FnOnce(&mut UnitType)) {
    let mut t = r.sim.infantry["WALKER"].clone();
    t.strength = Some(1000);
    t.primary = None;
    f(&mut t);
    register(r, "infantry", name);
    r.sim.infantry.insert(name.into(), t);
}

fn building(r: &mut Ruleset, name: &str, f: impl FnOnce(&mut BuildingType)) {
    let mut t = BuildingType {
        strength: Some(1000),
        foundation: Some([1, 1]),
        ..Default::default()
    };
    f(&mut t);
    register(r, "buildings", name);
    r.sim.buildings.insert(name.into(), t);
}

fn weapon(r: &mut Ruleset, name: &str, warhead: &str, damage: i32, unlimited: bool) {
    r.sim.weapons.insert(
        name.into(),
        WeaponType {
            damage: Some(damage),
            rof: Some(30),
            range: Some(dec("10")),
            speed: Some(100),
            projectile: Some("Bullet".into()),
            warhead: Some(warhead.into()),
            infinite_mind_control: unlimited.then_some(true),
            ..Default::default()
        },
    );
}

/// The combat ruleset with:
/// - warheads MIND (takeover, versus 1), MINDZ (takeover, versus 0) and
///   BOOM (the demolition warhead, versus 1);
/// - weapons Ray1 (MIND, damage 1), Ray3 (MIND, damage 3), RayU (MIND,
///   damage 3, unlimited), RayU1 (MIND, damage 1, unlimited), RayZ (MINDZ, damage 3);
/// - vehicles CTRL1 (Ray1), CTRL3 (Ray3), MASTER (RayU), ZERO (RayZ),
///   SIDE (Gun, secondary Ray3), PROMO (Ray1, elite Ray3), ELITE (Gun,
///   elite Ray3), BOX (plain), BOX2 (plain, its own release sound, link
///   height 33, ring height 44), STOIC (psionics-immune), TANK (Gun), DEP
///   (a simple deployer), CART (open-topped transport);
/// - infantry CLONE (Ray3), GRUNT (plain);
/// - buildings PSI (Ray3, not immune), HUT (immunity omitted), HUTOFF
///   (immunity explicitly off);
/// - the rules-wide release sound, rings, line frames 15 and the overload
///   lists count (0, 5), damage (0, 7), frames (0, 3).
fn rules() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    for (name, v) in [("MIND", "1"), ("MINDZ", "0")] {
        r.sim.warheads.insert(
            name.into(),
            WarheadType {
                verses: every(v),
                mind_control: Some(true),
                ..Default::default()
            },
        );
    }
    r.sim.warheads.insert(
        "BOOM".into(),
        WarheadType {
            verses: every("1"),
            ..Default::default()
        },
    );
    r.sim.general.c4_warhead = Some("BOOM".into());
    weapon(&mut r, "Ray1", "MIND", 1, false);
    weapon(&mut r, "Ray3", "MIND", 3, false);
    weapon(&mut r, "RayU", "MIND", 3, true);
    weapon(&mut r, "RayU1", "MIND", 1, true);
    weapon(&mut r, "RayZ", "MINDZ", 3, false);
    vehicle(&mut r, "CTRL1", |t| t.primary = Some("Ray1".into()));
    vehicle(&mut r, "CTRL3", |t| t.primary = Some("Ray3".into()));
    vehicle(&mut r, "MASTER", |t| t.primary = Some("RayU".into()));
    vehicle(&mut r, "MASTER1", |t| t.primary = Some("RayU1".into()));
    vehicle(&mut r, "ZERO", |t| t.primary = Some("RayZ".into()));
    vehicle(&mut r, "SIDE", |t| {
        t.primary = Some("Gun".into());
        t.secondary = Some("Ray3".into());
    });
    vehicle(&mut r, "PROMO", |t| {
        t.primary = Some("Ray1".into());
        t.elite_primary = Some("Ray3".into());
    });
    vehicle(&mut r, "ELITE", |t| {
        t.primary = Some("Gun".into());
        t.elite_primary = Some("Ray3".into());
    });
    vehicle(&mut r, "BOX", |_| {});
    vehicle(&mut r, "BOX2", |t| {
        t.combat.mind_cleared_sound = Some("Bye".into());
        t.combat.lepton_mind_control_offset = Some(33);
        t.combat.mind_control_ring_offset = Some(44);
    });
    vehicle(&mut r, "STOIC", |t| {
        t.combat.immune_to_psionics = Some(true)
    });
    vehicle(&mut r, "TANK", |t| t.primary = Some("Gun".into()));
    vehicle(&mut r, "DEP", |t| t.is_simple_deployer = Some(true));
    vehicle(&mut r, "CART", |t| {
        t.passengers = Some(3);
        t.size_limit = Some(dec("2"));
        t.open_topped = Some(true);
    });
    infantry(&mut r, "CLONE", |t| t.primary = Some("Ray3".into()));
    infantry(&mut r, "GRUNT", |_| {});
    building(&mut r, "PSI", |b| {
        b.primary = Some("Ray3".into());
        b.combat.immune_to_psionics = Some(false);
    });
    building(&mut r, "HUT", |_| {});
    building(&mut r, "HUTOFF", |b| {
        b.combat.immune_to_psionics = Some(false);
    });
    let g = &mut r.sim.general;
    g.mind_cleared_sound = Some("Free".into());
    g.yuri_mind_control_sound = Some("Mine".into());
    g.master_mind_overload_death_sound = Some("Pop".into());
    g.controlled_animation_type = Some("RING".into());
    g.perma_controlled_animation_type = Some("PERMA".into());
    g.mind_control_attack_line_frames = Some(15);
    g.overload_count = Some(vec![0, 5]);
    g.overload_damage = Some(vec![0, 7]);
    g.overload_frames = Some(vec![0, 3]);
    r
}

fn game() -> World {
    testkit::game_with(&rules(), SIZE, 1)
}

fn at(dx: i32, dy: i32) -> CellXY {
    testkit::mid(SIZE, dx, dy)
}

fn put(w: &mut World, kind: Kind, name: &str, house: u8, dx: i32, dy: i32) -> ObjectId {
    testkit::spawn(w, kind, name, house, at(dx, dy), 0)
}

fn veh(w: &mut World, name: &str, house: u8, dx: i32, dy: i32) -> ObjectId {
    put(w, Kind::Vehicle, name, house, dx, dy)
}

fn owner(w: &World, id: ObjectId) -> Option<u8> {
    w.obj(id).and_then(|o| o.owner).map(|h| h.0)
}

fn victims(w: &World, c: ObjectId) -> Vec<ObjectId> {
    mind::controller(w, c).map_or(Vec::new(), |r| r.victims.clone())
}

fn mind_effects(w: &World) -> Vec<(MindSound, Option<Pos>)> {
    w.effects
        .iter()
        .filter_map(|e| match e {
            Effect::Mind { sound, pos, .. } => Some((*sound, *pos)),
            _ => None,
        })
        .collect()
}

fn cleared(w: &World) -> usize {
    mind_effects(w)
        .iter()
        .filter(|(s, _)| *s == MindSound::Cleared)
        .count()
}

fn lethal(id: ObjectId, w: &World) -> Hit {
    Hit {
        amount: w.obj(id).unwrap().health * 10,
        warhead: None,
        distance: 0,
        attacker: None,
        house: None,
        ignore_defences: true,
        no_escape: false,
    }
}

// ---------------------------------------------------------------------------
// Claim 1: records and capacity.

// spec: S-0601 claim 1 and F1 (a record when the type's primary weapon has a takeover warhead: capacity its damage, unlimited its flag, countdown 30, highlight and sound mark cleared, no victims; infantry, vehicles and buildings)
#[test]
fn claim_1_records_and_capacity() {
    let mut w = game();
    let c3 = veh(&mut w, "CTRL3", BETA, 0, 0);
    let m = veh(&mut w, "MASTER", BETA, 1, 0);
    let cl = put(&mut w, Kind::Infantry, "CLONE", BETA, 2, 0);
    let ps = put(&mut w, Kind::Building, "PSI", BETA, 4, 4);
    let r = mind::controller(&w, c3).expect("a record");
    assert_eq!(
        (
            r.capacity,
            r.unlimited,
            r.countdown,
            r.highlight,
            r.sound_mark
        ),
        (3, false, 30, 0, false)
    );
    assert!(r.victims.is_empty());
    let r = mind::controller(&w, m).unwrap();
    assert_eq!((r.capacity, r.unlimited), (3, true));
    assert_eq!(mind::controller(&w, cl).unwrap().capacity, 3);
    assert_eq!(mind::controller(&w, ps).unwrap().capacity, 3);
}

// spec: S-0601 claim 1 (only the rookie primary slot: a secondary or elite takeover weapon makes no record; the capacity of a type with an elite takeover weapon of another damage is the rookie's and stays after promotion)
#[test]
fn claim_1_primary_slot_only() {
    let mut w = game();
    let side = veh(&mut w, "SIDE", BETA, 0, 0);
    let elite = veh(&mut w, "ELITE", BETA, 1, 0);
    let promo = veh(&mut w, "PROMO", BETA, 2, 0);
    assert!(mind::controller(&w, side).is_none());
    assert!(mind::controller(&w, elite).is_none());
    assert_eq!(mind::controller(&w, promo).unwrap().capacity, 1);
    // Promotion to elite (S-0311): the record is not read again.
    w.obj_mut(promo).unwrap().combat.experience = glasnost_num::soft::F32(0x4080_0000);
    testkit::run(&mut w, 2);
    assert_eq!(mind::controller(&w, promo).unwrap().capacity, 1);
}

// ---------------------------------------------------------------------------
// Claim 2: at capacity.

// spec: S-0601 claim 2 and F1 (at capacity ⇔ not unlimited and N ≥ capacity; a capacity-1 controller holding a victim is at capacity; an unlimited one never)
#[test]
fn claim_2_at_capacity() {
    let mut w = game();
    let c1 = veh(&mut w, "CTRL1", BETA, 0, 0);
    let m = veh(&mut w, "MASTER", BETA, 1, 0);
    let a = veh(&mut w, "BOX", ALPHA, 3, 0);
    let b = veh(&mut w, "BOX", ALPHA, 4, 0);
    assert!(!mind::at_capacity(&w, c1));
    assert!(mind::take(&mut w, c1, a));
    assert!(mind::at_capacity(&w, c1));
    assert!(
        mind::can_take(&w, c1, b),
        "an explicit order is not refused for it"
    );
    assert!(mind::take(&mut w, m, b));
    assert!(!mind::at_capacity(&w, m));
}

// ---------------------------------------------------------------------------
// Claim 3: the takeover test.

// spec: S-0601 claim 3 (each row: the same house, psionics immunity, a building's default immunity, mind-controlled, permanently controlled, protection, capacity, construction and selling)
#[test]
fn claim_3_takeover_test_rows() {
    let mut w = game();
    let c3 = veh(&mut w, "CTRL3", BETA, 0, 0);
    let c1 = veh(&mut w, "CTRL1", BETA, 0, 2);
    let box_a = veh(&mut w, "BOX", ALPHA, 3, 0);
    let own = veh(&mut w, "BOX", BETA, 4, 0);
    let stoic = veh(&mut w, "STOIC", ALPHA, 5, 0);
    let hut = put(&mut w, Kind::Building, "HUT", ALPHA, 6, 6);
    let hutoff = put(&mut w, Kind::Building, "HUTOFF", ALPHA, 8, 8);
    let grunt = put(&mut w, Kind::Infantry, "GRUNT", ALPHA, 3, 3);
    assert!(mind::can_take(&w, c3, box_a));
    assert!(mind::can_take(&w, c3, grunt));
    assert!(!mind::can_take(&w, c3, own), "the same house");
    assert!(!mind::can_take(&w, c3, stoic), "psionics immunity");
    assert!(
        !mind::can_take(&w, c3, hut),
        "a building is immune by default"
    );
    assert!(mind::can_take(&w, c3, hutoff));
    // Construction and selling.
    w.obj_mut(hutoff).unwrap().beh.current = Some(Behaviour::Selling);
    assert!(!mind::can_take(&w, c3, hutoff), "selling");
    w.obj_mut(hutoff).unwrap().beh.current = Some(Behaviour::Construction);
    assert!(!mind::can_take(&w, c3, hutoff), "construction");
    // Protection (S-0513 claim 27).
    let tick = w.tick;
    w.obj_mut(grunt).unwrap().sw.protection = Some(glasnost_sim::superweapons::Protection {
        start: tick,
        duration: 100,
        kind: glasnost_sim::superweapons::ProtectKind::Curtain,
        tint: Default::default(),
    });
    assert!(!mind::can_take(&w, c3, grunt), "protected");
    // Mind-controlled, by another controller or permanently.
    assert!(mind::take(&mut w, c1, box_a));
    assert!(!mind::can_take(&w, c3, box_a), "already controlled");
    let b2 = veh(&mut w, "BOX", ALPHA, 3, 4);
    assert!(mind::dominate(&mut w, b2, HouseId(BETA)));
    // b2 now belongs to Beta and is permanent; Alpha's controller sees it.
    let ca = veh(&mut w, "CTRL3", ALPHA, -3, 0);
    assert!(!mind::can_take(&w, ca, b2), "permanently controlled");
    // Capacity: a full capacity-3 controller fails; capacity 1 passes.
    let t: Vec<ObjectId> = (0..3).map(|i| veh(&mut w, "BOX", ALPHA, -2, i)).collect();
    for &x in &t {
        assert!(mind::take(&mut w, c3, x));
    }
    let more = veh(&mut w, "BOX", ALPHA, -4, 4);
    assert!(!mind::can_take(&w, c3, more), "full");
    assert!(mind::can_take(&w, c1, more), "capacity 1 replaces");
}

// ---------------------------------------------------------------------------
// Claim 4: the firing check.

// spec: S-0601 claim 4 (illegal when the takeover test fails; a passing test goes on to the zero-versus test; a cell target is illegal; a takeover weapon of a firer without a record is illegal, a departure)
#[test]
fn claim_4_firing_check() {
    let mut w = game();
    let c3 = veh(&mut w, "CTRL3", BETA, 0, 0);
    let zero = veh(&mut w, "ZERO", BETA, 0, 2);
    let side = veh(&mut w, "SIDE", BETA, 0, -2);
    let box_a = veh(&mut w, "BOX", ALPHA, 2, 0);
    let stoic = veh(&mut w, "STOIC", ALPHA, 2, 2);
    let aim = |w: &mut World, a: ObjectId, t: Target, slot: usize| {
        w.obj_mut(a).unwrap().target = Some(t);
        check(w, a, Some(t), slot, false)
    };
    assert_ne!(aim(&mut w, c3, Target::Object(box_a), 0), Answer::Illegal);
    assert_eq!(aim(&mut w, c3, Target::Object(stoic), 0), Answer::Illegal);
    assert_eq!(aim(&mut w, c3, Target::Cell(at(2, 0)), 0), Answer::Illegal);
    assert!(mind::can_take(&w, zero, box_a), "the takeover test passes");
    assert_eq!(
        aim(&mut w, zero, Target::Object(box_a), 0),
        Answer::Illegal,
        "the zero-versus test refuses it"
    );
    assert_eq!(
        aim(&mut w, side, Target::Object(box_a), 1),
        Answer::Illegal,
        "no record"
    );
    assert!(mind::firing_illegal(
        &w,
        side,
        w.rules().combat.weapon_index["Ray3"],
        Target::Object(box_a)
    ));
}

// spec: S-0601 claim 7 (a full limited controller of capacity 2 or more cannot fire a takeover weapon at all; an unlimited one can)
#[test]
fn claim_7_full_controller_cannot_fire() {
    let mut w = game();
    let c3 = veh(&mut w, "CTRL3", BETA, 0, 0);
    let m = veh(&mut w, "MASTER", BETA, 0, 2);
    for i in 0..3 {
        let x = veh(&mut w, "BOX", ALPHA, -3, i);
        assert!(mind::take(&mut w, c3, x));
        let y = veh(&mut w, "BOX", ALPHA, -4, i);
        assert!(mind::take(&mut w, m, y));
    }
    let t = veh(&mut w, "BOX", ALPHA, 2, 1);
    w.obj_mut(c3).unwrap().target = Some(Target::Object(t));
    assert_eq!(
        check(&w, c3, Some(Target::Object(t)), 0, false),
        Answer::Illegal
    );
    w.obj_mut(m).unwrap().target = Some(Target::Object(t));
    assert_ne!(
        check(&w, m, Some(Target::Object(t)), 0, false),
        Answer::Illegal
    );
}

// ---------------------------------------------------------------------------
// Claim 5: detonation.

// spec: S-0601 claims 5 and 6 (a takeover shot fired through the ordinary attack path takes the target without damage; the take-over sound names the controller's house and the target's house before)
#[test]
fn claim_5_shot_takes_without_damage() {
    let mut w = game();
    let c3 = veh(&mut w, "CTRL3", BETA, 0, 0);
    let t = veh(&mut w, "BOX", ALPHA, 3, 0);
    let health = w.obj(t).unwrap().health;
    testkit::run(&mut w, 4);
    assert!(glasnost_sim::orders::issue(
        &mut w,
        HouseId(BETA),
        glasnost_sim::orders::Order::Behaviour {
            actor: c3,
            behaviour: Behaviour::Attack,
            attack_move: false,
            target: Some(Target::Object(t)),
            dest: None,
        }
    ));
    let mut taken = false;
    for _ in 0..200 {
        w.effects.clear();
        testkit::run(&mut w, 1);
        if owner(&w, t) == Some(BETA) {
            taken = true;
            break;
        }
    }
    assert!(taken, "the shot lands and takes the target");
    assert_eq!(w.obj(t).unwrap().health, health, "no damage");
    assert_eq!(mind::controlled_by(&w, t), Some(c3));
    assert_eq!(mind::original_owner(&w, t), Some(HouseId(ALPHA)));
    let houses = w.effects.iter().find_map(|e| match e {
        Effect::Mind {
            sound: MindSound::Takeover,
            houses,
            ..
        } => Some(*houses),
        _ => None,
    });
    assert_eq!(houses, Some([Some(HouseId(BETA)), Some(HouseId(ALPHA))]));
}

// spec: S-0601 claim 5 (a cell target takes nothing; a firer gone or without a record does nothing; the takeover test is made again at landing)
#[test]
fn claim_5_detonation_guards() {
    let mut w = game();
    let c3 = veh(&mut w, "CTRL3", BETA, 0, 0);
    let c1 = veh(&mut w, "CTRL1", BETA, 0, 2);
    let tank = veh(&mut w, "TANK", BETA, 0, -2);
    let t = veh(&mut w, "BOX", ALPHA, 3, 0);
    mind::detonation(&mut w, Some(c3), Some(Target::Cell(at(3, 0))));
    mind::detonation(&mut w, None, Some(Target::Object(t)));
    mind::detonation(&mut w, Some(tank), Some(Target::Object(t)));
    assert_eq!(owner(&w, t), Some(ALPHA));
    // Landing recheck: taken by another controller while in flight.
    assert!(mind::take(&mut w, c1, t));
    mind::detonation(&mut w, Some(c3), Some(Target::Object(t)));
    assert_eq!(mind::controlled_by(&w, t), Some(c1));
    // A removed firer.
    let u = veh(&mut w, "BOX", ALPHA, 3, 3);
    let gone = veh(&mut w, "CTRL3", BETA, 1, 3);
    glasnost_sim::objects::remove(&mut w, gone);
    mind::detonation(&mut w, Some(gone), Some(Target::Object(u)));
    assert_eq!(owner(&w, u), Some(ALPHA));
}

mod tags {
    use super::*;
    use glasnost_map::triggers::{Action, Event, TagDef, TriggerDef, Triggers};

    /// Tag `TAG` (repeating) over one trigger on the any-event code 8
    /// (S-0011 claim 17), satisfied in every raise: each raise of the tag
    /// fires it once. Events 6 and 44 themselves are not judged by this
    /// engine yet (S-0011's subset; Q-1203).
    fn triggers() -> Triggers {
        let trig = |name: &[u8], code: i32| TriggerDef {
            events: vec![Event {
                code, // spec: S-0011 I3 (8: any event)
                kind: 0,
                value: 0,
                type_name: None,
            }],
            actions: vec![Action {
                code: 99,
                kind: 7,
                name: b"Bell".to_vec(),
                number: -1,
                waypoint: 0,
                ..Default::default()
            }],
            ..TriggerDef::created(name.to_vec())
        };
        Triggers {
            triggers: vec![trig(b"TANY", 8)],
            tags: vec![TagDef {
                id: b"TAG".to_vec(),
                repeat: 2,
                name: Vec::new(),
                first: Some(0),
                chain: vec![0],
            }],
            ..Default::default()
        }
    }

    // spec: S-0601 claim 5 (a live tagged target hears attack events 6 and 44 whether or not the takeover then succeeds; a firer without a record raises nothing)
    #[test]
    fn claim_5_tag_events() {
        let mut w = testkit::game_with_triggers(&rules(), SIZE, 1, triggers());
        let c3 = veh(&mut w, "CTRL3", BETA, 0, 0);
        let tank = veh(&mut w, "TANK", BETA, 0, 2);
        let stoic = veh(&mut w, "STOIC", ALPHA, 3, 0);
        glasnost_sim::triggers::object_tag(&mut w, stoic, Some(b"TAG"));
        let f0 = w.triggers.fired;
        mind::detonation(&mut w, Some(tank), Some(Target::Object(stoic)));
        assert_eq!(w.triggers.fired, f0, "no record: no notification");
        mind::detonation(&mut w, Some(c3), Some(Target::Object(stoic)));
        assert_eq!(
            w.triggers.fired - f0,
            2,
            "two raises (6 and 44), the takeover failing"
        );
        assert_eq!(owner(&w, stoic), Some(ALPHA));
    }
}

// ---------------------------------------------------------------------------
// Claim 6: the takeover.

// spec: S-0601 claim 6 (original owner recorded, owner changed, appended to the list, the link timer started, a guard reset: target and destination cleared, guard forced)
#[test]
fn claim_6_takeover_results() {
    let mut w = game();
    let c3 = veh(&mut w, "CTRL3", BETA, 0, 0);
    let a = veh(&mut w, "BOX", ALPHA, 3, 0);
    let b = veh(&mut w, "BOX", ALPHA, 4, 0);
    let foe = veh(&mut w, "BOX", BETA, 6, 6);
    {
        let o = w.obj_mut(b).unwrap();
        o.target = Some(Target::Object(foe));
        o.beh.current = Some(Behaviour::Attack);
    }
    assert!(mind::take(&mut w, c3, a));
    assert!(mind::take(&mut w, c3, b));
    assert_eq!(victims(&w, c3), vec![a, b]);
    assert_eq!(owner(&w, b), Some(BETA));
    assert_eq!(mind::original_owner(&w, b), Some(HouseId(ALPHA)));
    let v = w.mind.victims[&b];
    assert_eq!(v.line_until, w.tick + 15);
    let o = w.obj(b).unwrap();
    assert_eq!(o.target, None);
    assert_eq!(o.beh.current, Some(Behaviour::Guard));
}

// spec: S-0601 claim 6 (a simple-deployer vehicle with unload current keeps its behaviour and gets no guard reset)
#[test]
fn claim_6_simple_deployer_in_unload_keeps_its_behaviour() {
    let mut w = game();
    let c3 = veh(&mut w, "CTRL3", BETA, 0, 0);
    let d = veh(&mut w, "DEP", ALPHA, 3, 0);
    w.obj_mut(d).unwrap().beh.current = Some(Behaviour::Unload);
    assert!(mind::take(&mut w, c3, d));
    assert_eq!(w.obj(d).unwrap().beh.current, Some(Behaviour::Unload));
    // Released, it skips the guard request too (claim 9).
    mind::release(&mut w, c3, d);
    assert_eq!(owner(&w, d), Some(ALPHA));
    assert_eq!(w.obj(d).unwrap().beh.current, Some(Behaviour::Unload));
}

// spec: S-0601 claim 6 (a building can be taken and belongs to the controller's house in every respect)
#[test]
fn claim_6_building_taken() {
    let mut w = game();
    let c3 = veh(&mut w, "CTRL3", BETA, 0, 0);
    let hut = put(&mut w, Kind::Building, "HUTOFF", ALPHA, 4, 4);
    assert!(mind::take(&mut w, c3, hut));
    assert_eq!(owner(&w, hut), Some(BETA));
    assert!(mind::ring(&w, hut).unwrap().building);
}

// ---------------------------------------------------------------------------
// Claim 7: capacity 1 replaces.

// spec: S-0601 claim 7 (the old victim is released with claim 9's effects before the new transfer; a refused landing keeps it)
#[test]
fn claim_7_capacity_1_replaces() {
    let mut w = game();
    let c1 = veh(&mut w, "CTRL1", BETA, 0, 0);
    let a = veh(&mut w, "BOX", ALPHA, 3, 0);
    let b = veh(&mut w, "BOX", ALPHA, 4, 0);
    let stoic = veh(&mut w, "STOIC", ALPHA, 5, 0);
    assert!(mind::take(&mut w, c1, a));
    mind::detonation(&mut w, Some(c1), Some(Target::Object(stoic)));
    assert_eq!(
        mind::controlled_by(&w, a),
        Some(c1),
        "a refused landing keeps it"
    );
    w.effects.clear();
    mind::detonation(&mut w, Some(c1), Some(Target::Object(b)));
    assert_eq!(owner(&w, a), Some(ALPHA));
    assert_eq!(mind::controlled_by(&w, a), None);
    assert_eq!(victims(&w, c1), vec![b]);
    let e = mind_effects(&w);
    assert_eq!(e[0].0, MindSound::Cleared, "the release comes first");
    assert_eq!(e[1].0, MindSound::Takeover);
}

// spec: S-0601 claim 7 and scenario MC11 (an unlimited controller of capacity exactly 1 still replaces its victim)
#[test]
fn claim_7_unlimited_capacity_1_replaces() {
    let mut w = game();
    let m = veh(&mut w, "MASTER1", BETA, 0, 0);
    let a = veh(&mut w, "BOX", ALPHA, 3, 0);
    let b = veh(&mut w, "BOX", ALPHA, 4, 0);
    assert!(mind::take(&mut w, m, a));
    assert!(mind::take(&mut w, m, b));
    assert_eq!(victims(&w, m), vec![b]);
    assert_eq!(owner(&w, a), Some(ALPHA));
}

// ---------------------------------------------------------------------------
// Claim 8: the fate decision.

fn fate_rules(f: impl FnOnce(&mut glasnost_rules::sim::General)) -> Ruleset {
    let mut r = rules();
    let g = &mut r.sim.general;
    g.ai_capture_normal = Some(vec![0, 0, 0, 100]);
    g.ai_capture_wounded = Some(vec![0, 0, 0, 0]);
    g.ai_capture_low_power = Some(vec![0, 0, 0, 0]);
    g.ai_capture_low_money = Some(vec![0, 0, 0, 0]);
    f(g);
    r
}

// spec: S-0601 claim 8 (no player house is computer-controlled: no draw for a player house's takeover; the neutral house takes one draw and its selected fate, hunt)
#[test]
fn claim_8_neutral_house_draws_player_house_does_not() {
    let r = fate_rules(|_| {});
    let mut w = testkit::game_with_non_players(&r, SIZE, 1);
    let cb = veh(&mut w, "CTRL3", BETA, 0, 0);
    let cn = veh(&mut w, "CTRL3", NEUTRAL, 0, 3);
    let a = veh(&mut w, "BOX", ALPHA, 3, 0);
    let b = veh(&mut w, "BOX", ALPHA, 3, 3);
    let rng = w.rng.clone();
    assert!(mind::take(&mut w, cb, a));
    assert_eq!(w.rng, rng, "no draw for a player house");
    assert!(mind::take(&mut w, cn, b));
    let mut expect = rng.clone();
    let half = w.rules().quirks.ranged_draw_half_span;
    expect.ranged(1, 100, half);
    assert_eq!(w.rng, expect, "one draw in 1 to 100");
    assert_eq!(w.obj(b).unwrap().beh.queued, Some(Behaviour::Hunt));
}

// spec: S-0601 claim 8 (table precedence: low money first, then insufficient power, then wounded, otherwise normal; a draw past every weight selects nothing, the draw still taken)
#[test]
fn claim_8_table_precedence() {
    // Low money: the neutral house has no credits; the mark is 1000.
    let r = fate_rules(|g| g.ai_capture_low_money_mark = Some(1000));
    let mut w = testkit::game_with_non_players(&r, SIZE, 1);
    w.houses[usize::from(NEUTRAL)].credits = 0;
    let cn = veh(&mut w, "CTRL3", NEUTRAL, 0, 3);
    let b = veh(&mut w, "BOX", ALPHA, 3, 3);
    w.obj_mut(b).unwrap().beh.queued = None;
    let rng = w.rng.clone();
    assert!(mind::take(&mut w, cn, b));
    assert_ne!(w.rng, rng, "the draw is taken");
    assert_ne!(
        w.obj(b).unwrap().beh.queued,
        Some(Behaviour::Hunt),
        "low money: nothing"
    );
    // Wounded: health 499 of 1000 below 0.5.
    let r = fate_rules(|g| g.ai_capture_wounded_mark = Some(dec("0.5")));
    let mut w = testkit::game_with_non_players(&r, SIZE, 1);
    let cn = veh(&mut w, "CTRL3", NEUTRAL, 0, 3);
    let b = veh(&mut w, "BOX", ALPHA, 3, 3);
    let c = veh(&mut w, "BOX", ALPHA, 4, 3);
    w.obj_mut(b).unwrap().health = 499;
    w.obj_mut(c).unwrap().health = 500;
    for x in [b, c] {
        w.obj_mut(x).unwrap().beh.queued = None;
    }
    assert!(mind::take(&mut w, cn, b));
    assert_ne!(
        w.obj(b).unwrap().beh.queued,
        Some(Behaviour::Hunt),
        "wounded"
    );
    let cn2 = veh(&mut w, "CTRL3", NEUTRAL, 0, 5);
    assert!(mind::take(&mut w, cn2, c));
    assert_eq!(
        w.obj(c).unwrap().beh.queued,
        Some(Behaviour::Hunt),
        "exactly at the mark is not wounded"
    );
}

// spec: S-0601 claim 8 (on release to the neutral house the decision runs for that owner, with the controller's money and power)
#[test]
fn claim_8_release_to_the_neutral_house_draws() {
    let r = fate_rules(|_| {});
    let mut w = testkit::game_with_non_players(&r, SIZE, 1);
    let cb = veh(&mut w, "CTRL3", BETA, 0, 0);
    let n = veh(&mut w, "BOX", NEUTRAL, 3, 0);
    assert!(mind::take(&mut w, cb, n));
    w.obj_mut(n).unwrap().beh.queued = None;
    let rng = w.rng.clone();
    mind::release(&mut w, cb, n);
    assert_eq!(owner(&w, n), Some(NEUTRAL));
    assert_ne!(w.rng, rng);
    assert_eq!(w.obj(n).unwrap().beh.queued, Some(Behaviour::Hunt));
}

// ---------------------------------------------------------------------------
// Claim 9: release.

// spec: S-0601 claim 9 (the release sound at V, the type's when set, else the rules-wide one; back to the original owner; target cleared and guard requested; V leaves the list; the ring goes)
#[test]
fn claim_9_release() {
    let mut w = game();
    let c3 = veh(&mut w, "CTRL3", BETA, 0, 0);
    let a = veh(&mut w, "BOX2", ALPHA, 3, 0);
    let foe = veh(&mut w, "BOX", ALPHA, 6, 6);
    assert!(mind::take(&mut w, c3, a));
    assert!(mind::ring(&w, a).is_some());
    {
        let o = w.obj_mut(a).unwrap();
        o.target = Some(Target::Object(foe));
        o.beh.current = Some(Behaviour::Attack);
    }
    w.effects.clear();
    mind::release(&mut w, c3, a);
    assert_eq!(owner(&w, a), Some(ALPHA));
    assert_eq!(mind::controlled_by(&w, a), None);
    assert!(victims(&w, c3).is_empty());
    assert!(mind::ring(&w, a).is_none());
    let o = w.obj(a).unwrap();
    assert_eq!(o.target, None);
    assert!(
        o.beh.current == Some(Behaviour::Guard) || o.beh.queued == Some(Behaviour::Guard),
        "guard requested"
    );
    let of = w.effects.iter().find_map(|e| match e {
        Effect::Mind {
            sound: MindSound::Cleared,
            of,
            ..
        } => Some(*of),
        _ => None,
    });
    let of = of.flatten().expect("the release sound names the type");
    let o = w.obj(a).unwrap();
    assert_eq!((of.kind, of.index), (o.kind, o.type_index));
    assert_eq!(
        glasnost_sim::combat::veterancy::type_combat(&w, a)
            .unwrap()
            .mind_cleared_sound
            .as_deref(),
        Some("Bye")
    );
}

// ---------------------------------------------------------------------------
// Claim 10: what releases every victim.

/// A Beta controller holding Alpha boxes a, b (in that order).
fn held(w: &mut World, ctrl: &str, kind: Kind) -> (ObjectId, ObjectId, ObjectId) {
    let c = put(w, kind, ctrl, BETA, 0, 0);
    let a = veh(w, "BOX", ALPHA, 3, 0);
    let b = veh(w, "BOX", ALPHA, 3, 3);
    assert!(mind::take(w, c, a));
    assert!(mind::take(w, c, b));
    w.effects.clear();
    (c, a, b)
}

fn all_free(w: &World, a: ObjectId, b: ObjectId) -> bool {
    owner(w, a) == Some(ALPHA)
        && owner(w, b) == Some(ALPHA)
        && !mind::mind_controlled(w, a)
        && !mind::mind_controlled(w, b)
}

// spec: S-0601 claim 10 (the controller's killing hit releases all victims at once, the most recently taken first)
#[test]
fn claim_10_killing_hit_most_recent_first() {
    let mut w = game();
    let (c, a, b) = held(&mut w, "CTRL3", Kind::Vehicle);
    let pa = w.obj(a).unwrap().pos;
    let pb = w.obj(b).unwrap().pos;
    let h = lethal(c, &w);
    damage::apply(&mut w, c, h);
    assert!(all_free(&w, a, b));
    let order: Vec<Option<Pos>> = mind_effects(&w)
        .into_iter()
        .filter(|(s, _)| *s == MindSound::Cleared)
        .map(|(_, p)| p)
        .collect();
    assert_eq!(order, vec![Some(pb), Some(pa)]);
}

// spec: S-0601 claim 10 (removal from the game, an erasing hit, a trigger owner change and a building not in operation each release every victim)
#[test]
fn claim_10_other_paths() {
    let mut w = game();
    let (c, a, b) = held(&mut w, "CTRL3", Kind::Vehicle);
    glasnost_sim::objects::remove(&mut w, c);
    assert!(all_free(&w, a, b));
    assert!(mind::controller(&w, c).is_none());

    let mut w = game();
    let (c, a, b) = held(&mut w, "CTRL3", Kind::Vehicle);
    mind::erasing_hit(&mut w, c);
    assert!(all_free(&w, a, b));

    let mut w = game();
    let (c, a, b) = held(&mut w, "CTRL3", Kind::Vehicle);
    mind::trigger_owner_change(&mut w, c);
    assert!(all_free(&w, a, b));

    let mut w = game();
    let (c, a, b) = held(&mut w, "PSI", Kind::Building);
    mind::building_turn(&mut w, c, true);
    assert_eq!(victims(&w, c).len(), 2, "in operation: kept");
    mind::building_turn(&mut w, c, false);
    assert!(all_free(&w, a, b));
    assert_eq!(cleared(&w), 2);
}

// spec: S-0601 claim 10 and S-0405 claims 8 and 9 (a controller building switched off is not in operation and releases its victims in its own turn of the tick, not at the switch)
#[test]
fn claim_10_building_not_in_operation_releases_in_its_turn() {
    let mut w = game();
    let (c, a, b) = held(&mut w, "PSI", Kind::Building);
    testkit::run(&mut w, 2);
    assert_eq!(victims(&w, c).len(), 2, "in operation: kept");
    w.obj_mut(c).unwrap().econ.building.switched_off = true;
    assert_eq!(victims(&w, c).len(), 2, "not at the switch");
    testkit::run(&mut w, 1);
    assert!(all_free(&w, a, b));
}

// ---------------------------------------------------------------------------
// Claim 11: an owner change of the controller.

// spec: S-0601 claim 11 and Edge cases (quirks.mind_control_owner_change_keeps_victims on: nothing released, the victims keep their owner and original owner; off: every victim released first)
#[test]
fn claim_11_owner_change_quirk_both_positions() {
    for on in [true, false] {
        let mut r = rules();
        r.globals.quirks.mind_control_owner_change_keeps_victims = on;
        let mut w = testkit::game_with(&r, SIZE, 1);
        let c = veh(&mut w, "CTRL3", BETA, 0, 0);
        let a = veh(&mut w, "BOX", ALPHA, 3, 0);
        assert!(mind::take(&mut w, c, a));
        let g = veh(&mut w, "BOX", BETA, 5, 5);
        // An owner change by other means (S-0414 claim 21's consumers).
        mind::set_owner(&mut w, c, Some(HouseId(ALPHA)));
        let _ = g;
        if on {
            assert_eq!(victims(&w, c), vec![a]);
            assert_eq!(owner(&w, a), Some(BETA), "the controller's old house");
            assert_eq!(mind::original_owner(&w, a), Some(HouseId(ALPHA)));
            // Still released later, back to its original owner.
            let h = lethal(c, &w);
            damage::apply(&mut w, c, h);
            assert_eq!(owner(&w, a), Some(ALPHA));
        } else {
            assert!(victims(&w, c).is_empty());
            assert!(!mind::mind_controlled(&w, a));
            assert_eq!(owner(&w, a), Some(ALPHA));
        }
    }
}

// ---------------------------------------------------------------------------
// Claim 12: the victim leaves the game.

// spec: S-0601 claim 12 (a victim removed from the game silently leaves the list: no owner change, no sound, no fate)
#[test]
fn claim_12_removed_victim_leaves_silently() {
    let mut w = game();
    let (c, a, b) = held(&mut w, "CTRL3", Kind::Vehicle);
    let rng = w.rng.clone();
    glasnost_sim::objects::remove(&mut w, a);
    assert_eq!(victims(&w, c), vec![b]);
    assert_eq!(cleared(&w), 0);
    assert_eq!(w.rng, rng);
}

// spec: S-0601 claim 12 and Edge cases (quirks.dying_victim_holds_slot on: a dying victim keeps its slot while it exists; off: it leaves the list at its killing hit)
#[test]
fn claim_12_dying_victim_quirk_both_positions() {
    for on in [true, false] {
        let mut r = rules();
        r.globals.quirks.dying_victim_holds_slot = on;
        let mut w = testkit::game_with(&r, SIZE, 1);
        let c = veh(&mut w, "CTRL3", BETA, 0, 0);
        let a = veh(&mut w, "BOX", ALPHA, 3, 0);
        assert!(mind::take(&mut w, c, a));
        // The killing hit's step for an object that still exists (a
        // sinking ship, a building awaiting delayed destruction).
        mind::killed(&mut w, a);
        assert_eq!(victims(&w, c).len(), usize::from(on));
        assert_eq!(owner(&w, a), Some(BETA), "no owner change either way");
        assert_eq!(cleared(&w), 0);
    }
}

// ---------------------------------------------------------------------------
// Claim 13: permanent control.

// spec: S-0601 claim 13 (a controlled object is first released, then belongs to the dominating house, permanently controlled with the permanent ring; no original owner; mind-controlled for every consumer; another house's domination takes it again)
#[test]
fn claim_13_domination() {
    let mut w = game();
    let c = veh(&mut w, "CTRL3", BETA, 0, 0);
    let a = veh(&mut w, "BOX", ALPHA, 3, 0);
    assert!(mind::take(&mut w, c, a));
    w.effects.clear();
    assert!(mind::dominate(&mut w, a, HouseId(ALPHA)));
    assert_eq!(cleared(&w), 1, "released first");
    assert!(victims(&w, c).is_empty());
    assert!(mind::permanent(&w, a));
    assert!(mind::mind_controlled(&w, a));
    assert_eq!(owner(&w, a), Some(ALPHA));
    assert_eq!(mind::original_owner(&w, a), Some(HouseId(ALPHA)));
    let ring = mind::ring(&w, a).unwrap();
    assert!(ring.permanent);
    assert_eq!(ring.anim, "PERMA");
    assert!(!mind::can_take(&w, c, a));
    // Another house dominates it again.
    assert!(mind::dominate(&mut w, a, HouseId(BETA)));
    assert_eq!(owner(&w, a), Some(BETA));
    assert!(mind::permanent(&w, a));
}

// spec: S-0601 claim 13 (not a building, not psionics-immune, not protected; one already of the dominating house becomes permanent without an owner change)
#[test]
fn claim_13_domination_conditions() {
    let mut w = game();
    let hut = put(&mut w, Kind::Building, "HUTOFF", ALPHA, 4, 4);
    let stoic = veh(&mut w, "STOIC", ALPHA, 3, 0);
    let own = veh(&mut w, "BOX", BETA, 3, 3);
    assert!(!mind::dominate(&mut w, hut, HouseId(BETA)));
    assert!(!mind::dominate(&mut w, stoic, HouseId(BETA)));
    let fx = w.effects.len();
    assert!(mind::dominate(&mut w, own, HouseId(BETA)));
    assert!(mind::permanent(&w, own));
    assert_eq!(owner(&w, own), Some(BETA));
    assert_eq!(w.effects.len(), fx);
}

// ---------------------------------------------------------------------------
// Claim 14: the original-owner query.

// spec: S-0601 claim 14 and S-0414 claim 5 (a defeated house's controlled object held by another house is hit when no civilian-side house is in the game; the holder's defeat does not hit it)
#[test]
fn claim_14_defeat_by_original_house() {
    let mut w = game();
    let c = veh(&mut w, "CTRL3", BETA, 0, 0);
    let a = veh(&mut w, "BOX", ALPHA, 3, 0);
    let b = veh(&mut w, "BOX", ALPHA, 3, 3);
    assert!(mind::take(&mut w, c, a));
    assert!(mind::take(&mut w, c, b));
    assert_eq!(mind::original_owner(&w, c), Some(HouseId(BETA)));
    glasnost_sim::econ::victory::defeat(&mut w, HouseId(ALPHA));
    let gone = |w: &World, x: ObjectId| w.obj(x).is_none_or(|o| o.health <= 0);
    assert!(gone(&w, a) && gone(&w, b), "judged by their original house");
    assert!(!gone(&w, c));
}

// spec: S-0601 claim 14 (a controlled object of Alpha held by Beta survives Beta's defeat: its original house is Alpha)
#[test]
fn claim_14_holder_defeat_spares_victims() {
    let mut w = game();
    let c = veh(&mut w, "CTRL3", BETA, 0, 0);
    let a = veh(&mut w, "BOX", ALPHA, 3, 0);
    assert!(mind::take(&mut w, c, a));
    glasnost_sim::econ::victory::defeat(&mut w, HouseId(BETA));
    assert!(w.obj(a).is_some_and(|o| o.health > 0));
    assert_eq!(
        owner(&w, a),
        Some(ALPHA),
        "released by the controller's death"
    );
}

// ---------------------------------------------------------------------------
// Claim 16: overload.

// spec: S-0601 F2 (the first level whose count is at least N, else the last)
#[test]
fn f2_overload_level() {
    assert_eq!(mind::overload_level(&[0, 5], 0), Some(0));
    assert_eq!(mind::overload_level(&[0, 5], 1), Some(1));
    assert_eq!(mind::overload_level(&[0, 5], 5), Some(1));
    assert_eq!(mind::overload_level(&[0, 5], 9), Some(1));
    assert_eq!(mind::overload_level(&[], 9), None);
}

// spec: S-0601 claim 16 (controllers that are not unlimited never pulse)
#[test]
fn claim_16_limited_never_pulses() {
    let mut w = game();
    let c = veh(&mut w, "CTRL3", BETA, 0, 0);
    for _ in 0..100 {
        mind::overload_turn(&mut w, c);
    }
    assert_eq!(mind::controller(&w, c).unwrap().countdown, 30);
}

// spec: S-0601 claim 16 and F2, F3, F4 (the first check on the 31st update; d ≥ 1: hit of d with the demolition warhead, highlight 10, the sound once per run, 5 × 2 spark draws then a rocking draw at level i > 0; checks every frames + 1 updates)
#[test]
fn claim_16_pulse_timing_and_draws() {
    let mut w = game();
    let m = veh(&mut w, "MASTER", BETA, 0, 0);
    let a = veh(&mut w, "BOX", ALPHA, 3, 0);
    let b = veh(&mut w, "BOX", ALPHA, 3, 3);
    assert!(mind::take(&mut w, m, a));
    assert!(mind::take(&mut w, m, b));
    let full = w.obj(m).unwrap().health;
    for _ in 0..30 {
        mind::overload_turn(&mut w, m);
    }
    assert_eq!(
        w.obj(m).unwrap().health,
        full,
        "no check in the first 30 updates"
    );
    assert_eq!(mind::controller(&w, m).unwrap().countdown, 0);
    w.effects.clear();
    let mut expect = w.rng.clone();
    let half = w.rules().quirks.ranged_draw_half_span;
    for _ in 0..10 {
        expect.ranged(-200, 200, half);
    }
    expect.ranged(0, 100, half);
    mind::overload_turn(&mut w, m);
    assert_eq!(w.obj(m).unwrap().health, full - 7, "the 31st update pulses");
    let r = mind::controller(&w, m).unwrap();
    assert_eq!((r.highlight, r.countdown, r.sound_mark), (10, 3, true));
    assert_eq!(w.rng, expect, "10 spark draws then the rocking draw");
    let sounds = |w: &World| {
        mind_effects(w)
            .iter()
            .filter(|(s, _)| *s == MindSound::Overload)
            .count()
    };
    assert_eq!(sounds(&w), 1);
    // The next check is frames + 1 = 4 updates later; no second sound.
    for _ in 0..3 {
        mind::overload_turn(&mut w, m);
    }
    assert_eq!(w.obj(m).unwrap().health, full - 7);
    assert_eq!(mind::highlight(&w, m), 7);
    mind::overload_turn(&mut w, m);
    assert_eq!(w.obj(m).unwrap().health, full - 14);
    assert_eq!(sounds(&w), 1, "once per uninterrupted run");
}

// spec: S-0601 claim 16 (d < 1: the sound mark is cleared, no hit, no draws; the countdown becomes frames)
#[test]
fn claim_16_harmless_level_clears_the_mark() {
    let mut w = game();
    let m = veh(&mut w, "MASTER", BETA, 0, 0);
    w.mind.controllers.get_mut(&m).unwrap().sound_mark = true;
    w.mind.controllers.get_mut(&m).unwrap().countdown = 0;
    let rng = w.rng.clone();
    let full = w.obj(m).unwrap().health;
    mind::overload_turn(&mut w, m);
    let r = mind::controller(&w, m).unwrap();
    assert!(!r.sound_mark);
    assert_eq!(r.countdown, 0);
    assert_eq!(w.obj(m).unwrap().health, full);
    assert_eq!(w.rng, rng);
}

// spec: S-0601 claim 16 and claim 10 (a killing pulse releases the victims; the sparks still draw, the rocking draw does not)
#[test]
fn claim_16_killing_pulse() {
    let mut w = game();
    let m = veh(&mut w, "MASTER", BETA, 0, 0);
    let a = veh(&mut w, "BOX", ALPHA, 3, 0);
    assert!(mind::take(&mut w, m, a));
    w.obj_mut(m).unwrap().health = 5;
    w.mind.controllers.get_mut(&m).unwrap().countdown = 0;
    mind::overload_turn(&mut w, m);
    assert!(w.obj(m).is_none_or(|o| o.health <= 0));
    assert_eq!(owner(&w, a), Some(ALPHA));
}

// spec: S-0601 claim 16 (the overload update runs in the controller's own turn)
#[test]
fn claim_16_runs_in_the_turn() {
    let mut w = game();
    let m = veh(&mut w, "MASTER", BETA, 0, 0);
    let a = veh(&mut w, "BOX", ALPHA, 3, 0);
    assert!(mind::take(&mut w, m, a));
    let c0 = mind::controller(&w, m).unwrap().countdown;
    testkit::run(&mut w, 5);
    assert_eq!(mind::controller(&w, m).unwrap().countdown, c0 - 5);
}

// ---------------------------------------------------------------------------
// Claims 17 and 18: presentation.

// spec: S-0601 claim 17 and F5 (one line per victim in the controller's colour; the anchor is alternate anchor k mod 5; the end is the victim raised by its link height, 70 by default; the timer runs for the line frames; removal closes the gap)
#[test]
fn claim_17_link_lines() {
    let mut r = rules();
    let mut flh = Vec::new();
    for k in 0..5 {
        flh.push([10 * k, 0, 100 + k]);
    }
    r.sim
        .vehicles
        .get_mut("MASTER")
        .unwrap()
        .combat
        .alternate_flh = Some(flh);
    let mut w = testkit::game_with(&r, SIZE, 1);
    let m = veh(&mut w, "MASTER", BETA, 0, 0);
    let t: Vec<ObjectId> = (0..6)
        .map(|i| veh(&mut w, "BOX", ALPHA, 3, i - 3))
        .collect();
    for &x in &t {
        assert!(mind::take(&mut w, m, x));
    }
    let b2 = veh(&mut w, "BOX2", ALPHA, -3, 0);
    assert!(mind::take(&mut w, m, b2));
    let lines = mind::link_lines(&w);
    assert_eq!(lines.len(), 7);
    assert!(
        lines
            .iter()
            .all(|l| l.house == Some(HouseId(BETA)) && l.timer)
    );
    for (k, l) in lines.iter().enumerate() {
        assert_eq!(l.from, mind::anchor(&w, m, k));
    }
    assert_eq!(lines[5].from, lines[0].from, "k mod 5");
    assert_ne!(lines[1].from, lines[0].from);
    let p = w.obj(t[0]).unwrap().pos;
    assert_eq!(lines[0].to, Pos::new(p.x, p.y, p.z + 70));
    let p = w.obj(b2).unwrap().pos;
    assert_eq!(lines[6].to, Pos::new(p.x, p.y, p.z + 33));
    // The timer expires after 15 ticks.
    w.tick += 15;
    assert!(mind::link_lines(&w).iter().all(|l| !l.timer));
    // Removing the first victim moves the others up one anchor.
    glasnost_sim::objects::remove(&mut w, t[0]);
    let lines = mind::link_lines(&w);
    assert_eq!(lines[0].victim, t[1]);
    assert_eq!(lines[0].from, mind::anchor(&w, m, 0));
}

// spec: S-0601 claim 18 (the controlled ring on a controlled object, raised by its ring height, 140 by default; removed at release)
#[test]
fn claim_18_rings() {
    let mut w = game();
    let c = veh(&mut w, "CTRL3", BETA, 0, 0);
    let a = veh(&mut w, "BOX", ALPHA, 3, 0);
    let b = veh(&mut w, "BOX2", ALPHA, 3, 3);
    assert!(mind::ring(&w, a).is_none());
    assert!(mind::take(&mut w, c, a));
    assert!(mind::take(&mut w, c, b));
    let ra = mind::ring(&w, a).unwrap();
    assert_eq!(ra.anim, "RING");
    assert!(!ra.permanent && !ra.building);
    let p = w.obj(a).unwrap().pos;
    assert_eq!(ra.at, Pos::new(p.x, p.y, p.z + 140));
    let p = w.obj(b).unwrap().pos;
    assert_eq!(mind::ring(&w, b).unwrap().at, Pos::new(p.x, p.y, p.z + 44));
    mind::release(&mut w, c, a);
    assert!(mind::ring(&w, a).is_none());
}

// ---------------------------------------------------------------------------
// S-0309: the capacity exclusions.

// spec: S-0309 claims 2 and 18 and S-0601 claim 2 (a controller at capacity does not search for targets by itself)
#[test]
fn s0309_full_controller_does_not_search() {
    let mut w = game();
    let c1 = veh(&mut w, "CTRL1", BETA, 0, 0);
    let a = veh(&mut w, "BOX", ALPHA, 6, 6);
    let b = veh(&mut w, "BOX", ALPHA, 2, 0);
    assert!(mind::take(&mut w, c1, a));
    testkit::run(&mut w, 60);
    assert_eq!(owner(&w, b), Some(ALPHA), "no search at capacity");
    assert_eq!(w.obj(c1).unwrap().target, None);
}

// ---------------------------------------------------------------------------
// The transport pair (S-0507 claims 4 and 6).

// spec: S-0601 Terms and S-0507 claim 4 (a vehicle transport answers neither to a controlled unit and to a controller holding victims)
#[test]
fn transport_refuses_controlled_and_controllers() {
    let mut w = game();
    let c = put(&mut w, Kind::Infantry, "CLONE", BETA, 0, 0);
    let a = put(&mut w, Kind::Infantry, "GRUNT", ALPHA, 2, 0);
    let cart_a = veh(&mut w, "CART", ALPHA, 4, 0);
    let cart_b = veh(&mut w, "CART", BETA, 4, 4);
    let free = put(&mut w, Kind::Infantry, "CLONE", BETA, 1, 4);
    use glasnost_sim::transport::{Answer as Dock, answer};
    assert_eq!(answer(&w, cart_b, c), Dock::Accepts, "no victims yet");
    assert!(mind::take(&mut w, c, a));
    assert_eq!(
        answer(&w, cart_b, c),
        Dock::Neither,
        "a controller of objects"
    );
    assert_eq!(
        answer(&w, cart_b, a),
        Dock::Neither,
        "a mind-controlled unit"
    );
    assert_eq!(answer(&w, cart_a, a), Dock::Neither, "not allied any more");
    assert_eq!(answer(&w, cart_b, free), Dock::Accepts);
}

// spec: S-0601 claim 10 and S-0507 claim 6 (a controlled unit that boards is released; a controller inside a destroyed transport releases its victims)
#[test]
fn transport_boarding_releases() {
    let mut w = game();
    let c = put(&mut w, Kind::Infantry, "CLONE", BETA, 0, 0);
    let a = put(&mut w, Kind::Infantry, "GRUNT", ALPHA, 2, 0);
    let cart_b = veh(&mut w, "CART", BETA, 4, 4);
    assert!(mind::take(&mut w, c, a));
    glasnost_sim::transport::board(&mut w, a, cart_b);
    assert_eq!(owner(&w, a), Some(ALPHA));
    assert!(!mind::mind_controlled(&w, a));
    // A controller holding a victim inside a transport that is removed.
    let b = put(&mut w, Kind::Infantry, "GRUNT", ALPHA, 2, 2);
    let cart = veh(&mut w, "CART", BETA, -4, 0);
    let c2 = put(&mut w, Kind::Infantry, "CLONE", BETA, -2, 0);
    glasnost_sim::transport::board(&mut w, c2, cart);
    assert!(mind::take(&mut w, c2, b));
    // The transport leaves the game with its passengers (S-0312 claim 15).
    glasnost_sim::objects::remove(&mut w, cart);
    assert!(w.obj(c2).is_none());
    assert_eq!(owner(&w, b), Some(ALPHA));
}

// ---------------------------------------------------------------------------
// Save and load.

// spec: S-0601 State and ordering (controller and victim records, the permanent set and the overload state survive a save/load round trip with the same state hash)
#[test]
fn save_round_trip() {
    let mut w = game();
    let plain = save::state_hash(&w);
    let m = veh(&mut w, "MASTER", BETA, 0, 0);
    let a = veh(&mut w, "BOX", ALPHA, 3, 0);
    let b = veh(&mut w, "BOX", ALPHA, 3, 3);
    assert!(mind::take(&mut w, m, a));
    assert!(mind::dominate(&mut w, b, HouseId(BETA)));
    w.mind.controllers.get_mut(&m).unwrap().highlight = 4;
    let h = save::state_hash(&w);
    assert_ne!(h, plain);
    let l = save::load(&save::save(&w), w.rules_arc(), testkit::catalogue()).unwrap();
    assert_eq!(save::state_hash(&l), h);
    assert_eq!(l.mind.controllers, w.mind.controllers);
    assert_eq!(l.mind.victims, w.mind.victims);
    assert_eq!(l.mind.permanent, w.mind.permanent);
}
