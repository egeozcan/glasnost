//! Claim tests for slave miners and their slaves (S-0402 claims 14 and 19
//! to 28, F7; S-0312 claims 30 and 31; S-0408 claim 10a; S-0415 claim 11;
//! S-0213 claim 8 and table
//! T3 (h); S-0209 claim 4 row WP2) on synthetic data: an invented slave
//! infantry type, slave-miner vehicle and building types and values, no
//! stock data. Houses Alpha (0) and Beta (1) of the testkit's sandbox game,
//! both human.

mod harvest_common;

use std::sync::Arc;

use glasnost_map::CellXY;
use glasnost_map::triggers::{Event, TagDef, TriggerDef, Triggers};
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{Country, Decimal, LocomotionClass, Locomotor, UnitType, WarheadType};
use glasnost_sim::combat::damage::{self, Hit};
use glasnost_sim::econ::slaves::{self, GroupState, SlaveGroup, SlavePhase};
use glasnost_sim::econ::{TypeRef, stats, tech};
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind, SimRules};
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, HouseRole, HouseSetup, ObjectId, Target, World};
use harvest_common::*;

/// Slaves per group (invented).
const SLAVES: i32 = 3;
/// Regeneration delay (invented).
const REGEN: i32 = 50;
/// Reload delay (invented).
const RELOAD: i32 = 40;
/// A slave's storage (invented).
const STORE: i32 = 2;

fn group_fields(e: &mut glasnost_rules::sim_econ::EconFields) {
    e.enslaves = Some("SLAVE".into());
    e.slaves_number = Some(SLAVES);
    e.slave_regen_rate = Some(REGEN);
    e.slave_reload_rate = Some(RELOAD);
}

/// The harvest ruleset plus SLAVE (infantry, storage 2, harvest rate 3),
/// SMV (a slave-miner vehicle with three slaves, deploying into SMB), SMV2
/// (the same, deploying into SMB2), SMB (3 × 3, the deployed form, packing
/// up into SMV), SMB2 (2 × 2, packing up into SMV2), SMH (a 2 × 2 building
/// with three slaves that does not pack up) and the country Civland of the
/// civilian side.
fn rules() -> Ruleset {
    let mut r = ruleset();
    r.registries
        .get_mut("infantry")
        .unwrap()
        .push("SLAVE".into());
    let mut slave = UnitType {
        strength: Some(100),
        speed: Some(6),
        locomotion_class: Some(LocomotionClass::Foot),
        locomotor: Some(Locomotor::Walk),
        storage: Some(STORE),
        ..Default::default()
    };
    slave.econ.harvest_rate = Some(3);
    // A cheer sequence, so that a cheer request (S-0107 claim 6) is seen.
    slave.sequences.insert(
        "cheer".into(),
        glasnost_rules::sim::SequenceEntry {
            start: 0,
            count: 8,
            multiplier: 0,
            direction: None,
        },
    );
    r.sim.infantry.insert("SLAVE".into(), slave);
    for (n, into) in [("SMV", "SMB"), ("SMV2", "SMB2")] {
        let mut v = wheel();
        v.econ.resource_gatherer = Some(true);
        v.econ.resource_destination = Some(true);
        group_fields(&mut v.econ);
        v.deploys_into = Some(into.into());
        r.registries.get_mut("vehicles").unwrap().push(n.into());
        r.sim.vehicles.insert(n.into(), v);
    }
    for (n, size, into) in [
        ("SMB", 3, Some("SMV")),
        ("SMB2", 2, Some("SMV2")),
        ("SMH", 2, None),
    ] {
        let mut b = building(size, size);
        group_fields(&mut b.econ);
        b.undeploys_into = into.map(Into::into);
        b.econ.buildup = Some(true);
        b.econ.buildup_frames = Some(10);
        r.registries.get_mut("buildings").unwrap().push(n.into());
        r.sim.buildings.insert(n.into(), b);
    }
    r.registries
        .get_mut("countries")
        .unwrap()
        .push("Civland".into());
    let mut civ = Country::default();
    civ.econ.side = Some("Civilian".into());
    r.sim.countries.insert("Civland".into(), civ);
    r.sim.warheads.insert(
        "DEMO".into(),
        WarheadType {
            verses: Some((0..11).map(|_| Decimal("1".into())).collect()),
            ..Default::default()
        },
    );
    r.sim.general.c4_warhead = Some("DEMO".into());
    r
}

/// A 3 × 3 ORE field of stage `d` with its top-left cell at (x, y).
fn field(x: i32, y: i32, d: u8) -> Vec<(CellXY, u8, u8)> {
    let mut v = Vec::new();
    for dy in 0..3 {
        for dx in 0..3 {
            v.push((at(x + dx, y + dy), ORE, d));
        }
    }
    v
}

/// A game on `r` with `ore`; with `civilian`, a third house of the
/// civilian side (house 2).
fn game_on(r: &Ruleset, ore: &[(CellXY, u8, u8)], civilian: bool) -> World {
    game_with_triggers(r, ore, civilian, Triggers::default())
}

/// As [`game_on`], with the map's trigger data `t`.
fn game_with_triggers(r: &Ruleset, ore: &[(CellXY, u8, u8)], civilian: bool, t: Triggers) -> World {
    let mut map = testkit::empty_map(SIZE);
    map.triggers = t;
    for &(c, o, d) in ore {
        let cell = map.cells.get_mut(c).unwrap();
        cell.overlay = o;
        cell.overlay_data = d;
    }
    let mut s = testkit::setup(7);
    if civilian {
        s.houses.push(HouseSetup {
            name: "Civ".into(),
            country: "Civland".into(),
            human: false,
            start: None,
            role: HouseRole::Neutral,
            ai: false,
        });
    }
    let rules = Arc::new(SimRules::new(r, "synthetic-test"));
    glasnost_sim::load::new_game(rules, &map, testkit::catalogue(), &s).unwrap()
}

fn group(w: &World, m: ObjectId) -> SlaveGroup {
    slaves::group_of(w, m).expect("a slave group")
}

fn until(w: &mut World, max: i32, mut f: impl FnMut(&World) -> bool) -> Option<i32> {
    for _ in 0..max {
        testkit::run(w, 1);
        if f(w) {
            return Some(w.tick);
        }
    }
    None
}

fn unblock(w: &mut World) {
    while w.tick <= w.orders_blocked_through {
        testkit::run(w, 1);
    }
}

fn slave_type(w: &World) -> TypeRef {
    TypeRef {
        kind: Kind::Infantry,
        index: w.rules().index_of(Kind::Infantry, "SLAVE").unwrap(),
    }
}

fn on_map(w: &World, m: ObjectId) -> Vec<ObjectId> {
    slaves::slaves_of(w, m)
        .into_iter()
        .filter(|&s| w.obj(s).is_some_and(|o| o.on_map))
        .collect()
}

/// A deployed master SMB at top-left (0, −1) next to a field at (5, −1),
/// run until its slaves have emerged.
fn working_building(w: &mut World) -> ObjectId {
    let b = place_building(w, "SMB", 0, at(0, -1));
    until(w, 40, |w| on_map(w, b).len() == SLAVES as usize).expect("emerged");
    b
}

// spec: S-0402 claims 19 and 21 (N slaves of the named type, owned by the miner's house, ready, inside and off the map, the miner as master; the group at rest with last-rest tick 0; the first update 10 ticks after creation)
// spec: S-0402 claim 19 and S-0202 claim 10 (each slave creation takes the ordinary creation draw)
// spec: S-0402 claim 20 (created slaves do not raise the ever-built count) and claim 19 (no payment)
#[test]
fn claims_19_to_21_initial_group() {
    let r = rules();
    let mut w = game_on(&r, &[], false);
    testkit::run(&mut w, 3);
    let credits = w.houses[0].credits;
    let built = stats::built_total(&w, HouseId(0));
    let a = w.rng.taken();
    place_vehicle(&mut w, "HARV", 0, at(-8, 5), 0);
    let plain = w.rng.taken() - a;
    let b = w.rng.taken();
    let m = place_vehicle(&mut w, "SMV", 0, at(-8, -5), 0);
    assert_eq!(w.rng.taken() - b, plain + SLAVES as u64);
    let g = group(&w, m);
    assert_eq!(g.members.len(), SLAVES as usize);
    assert_eq!(g.state, GroupState::AtRest);
    assert_eq!(g.last_rest, 0);
    assert_eq!(g.next_update, w.tick + 10);
    for x in &g.members {
        assert!(x.inside && x.ready);
        let s = w.obj(x.id.unwrap()).unwrap();
        assert!(!s.on_map);
        assert_eq!(s.owner, Some(HouseId(0)));
        assert_eq!(slaves::master_of(&w, s.id), Some(m));
    }
    assert_eq!(tech::ever_built(&w, HouseId(0), slave_type(&w)), 0);
    // spec: S-0402 claim 20 and S-0414 claim 20 (no end-screen built total)
    assert_eq!(stats::built_total(&w, HouseId(0)), built);
    // spec: S-0402 claim 20 and S-0407 claim 9 (ordinary ownership counts: being inside does not make a slave uncreated)
    assert_eq!(tech::owned_count(&w, HouseId(0), slave_type(&w)), SLAVES);
    assert_eq!(w.houses[0].credits, credits);
}

// spec: S-0402 claim 20 and S-0414 claim 21 (a slave's later kill and loss count as any object's)
#[test]
fn claim_20_slave_kill_and_loss() {
    let r = rules();
    let mut w = game_on(&r, &field(5, -1, 5), false);
    let attacker = place_vehicle(&mut w, "HARV", 1, at(-8, -8), 0);
    let b = working_building(&mut w);
    let s = on_map(&w, b)[0];
    let (lost, kills) = (
        stats::of(&w, HouseId(0)).losses,
        stats::of(&w, HouseId(1)).kills,
    );
    let hit = Hit {
        amount: 100_000,
        warhead: None,
        distance: 0,
        attacker: Some(attacker),
        house: None,
        ignore_defences: true,
        no_escape: false,
    };
    damage::apply(&mut w, s, hit);
    assert_eq!(stats::of(&w, HouseId(0)).losses, lost + 1);
    assert_eq!(stats::of(&w, HouseId(1)).kills, kills + 1);
}

// spec: S-0402 claim 19 (a count of 0 or less creates no slave)
#[test]
fn claim_19_no_count() {
    let r = rules();
    let mut w = game_on(&r, &[], false);
    let m = place_vehicle(&mut w, "SLAVEM", 0, at(0, 0), 0);
    assert!(group(&w, m).members.is_empty());
}

// spec: S-0402 claims 21, 22 and 27 (an at-rest deployed group enters checking the field at its first update, without a check or emergence; at the next update a short-scan field keeps it and each ready slave is offered the foundation cell (W − 1, ⌊H/2⌋) in reverse slave-place order, one ranged draw each, facing 0)
#[test]
fn claim_21_building_emergence() {
    let r = rules();
    let mut w = game_on(&r, &field(5, -1, 5), false);
    testkit::run(&mut w, 3);
    let b = place_building(&mut w, "SMB", 0, at(1, -1));
    let ids = slaves::slaves_of(&w, b);
    let created = w.tick;
    while group(&w, b).state == GroupState::AtRest {
        testkit::run(&mut w, 1);
    }
    // The first update is in the tick numbered 10 after the creation tick's.
    assert_eq!(w.tick - created, 10 + 1);
    assert_eq!(group(&w, b).state, GroupState::CheckingField);
    assert!(on_map(&w, b).is_empty());
    let mut draws = 0;
    while on_map(&w, b).is_empty() {
        let before = w.rng.taken();
        testkit::run(&mut w, 1);
        draws = w.rng.taken() - before;
    }
    assert_eq!(w.tick - created, 20 + 1);
    // spec: S-0402 claim 27 (checking the field: a found field keeps the group checking)
    assert_eq!(group(&w, b).state, GroupState::CheckingField);
    let out = on_map(&w, b);
    assert_eq!(out.len(), SLAVES as usize);
    let unloading = at(3, 0);
    for &s in &out {
        assert_eq!(w.obj(s).unwrap().pos.cell(), unloading);
        assert_eq!(w.obj(s).unwrap().target, Some(Target::Object(b)));
        // spec: S-0402 claim 21 (facing 0, map north)
        assert_eq!(w.body_facing(s), 0);
    }
    // Reverse initial creation order in the update order.
    let order: Vec<ObjectId> = w
        .update
        .order
        .iter()
        .copied()
        .filter(|x| ids.contains(x))
        .collect();
    assert_eq!(order, vec![ids[2], ids[1], ids[0]]);
    assert!(draws >= SLAVES as u64);
}

// spec: S-0402 claim 21 (a slave taken inside in an update cannot finish reload in that update, even with delay 0; a reload completion and a replacement may emerge in the same update)
#[test]
fn claim_21_same_update_reload_and_replacement() {
    let mut r = rules();
    for b in ["SMB", "SMB2", "SMH"] {
        r.sim.buildings.get_mut(b).unwrap().econ.slave_reload_rate = Some(0);
    }
    let mut w = game_on(&r, &field(5, -1, 5), false);
    let b = working_building(&mut w);
    let s = slaves::slaves_of(&w, b)[0];
    until(&mut w, 600, |w| group(w, b).members[0].inside).expect("delivered");
    assert!(!group(&w, b).members[0].ready);
    let next = group(&w, b).next_update;
    while w.tick <= next {
        testkit::run(&mut w, 1);
    }
    assert!(w.obj(s).unwrap().on_map, "reloaded and out in one update");
    // A lost slave's replacement leaves in the update that creates it.
    let lost = slaves::slaves_of(&w, b)[1];
    glasnost_sim::objects::remove(&mut w, lost);
    until(&mut w, 200, |w| group(w, b).members[1].id.is_some()).expect("replaced");
    let n = group(&w, b).members[1].id.unwrap();
    assert!(w.obj(n).unwrap().on_map);
}

// spec: S-0402 claim 21 and S-0208 claim 3 (a vehicle master in its offered cell: the vehicle-present refusal, no emergence and no draw)
#[test]
fn claim_21_vehicle_master_blocks_emergence() {
    let r = rules();
    let mut w = game_on(&r, &[], false);
    let m = place_vehicle(&mut w, "SMV", 0, at(0, 0), 0);
    w.obj_mut(m).unwrap().econ.group.as_mut().unwrap().state = GroupState::Working;
    w.obj_mut(m).unwrap().econ.deploy.turning = true;
    let twin = {
        let mut x = w.clone();
        x.obj_mut(m)
            .unwrap()
            .econ
            .group
            .as_mut()
            .unwrap()
            .members
            .clear();
        x
    };
    let mut x = twin;
    let due = group(&w, m).next_update;
    while w.tick <= due {
        testkit::run(&mut w, 1);
        testkit::run(&mut x, 1);
    }
    assert!(on_map(&w, m).is_empty());
    assert_eq!(w.rng.taken(), x.rng.taken());
}

// spec: S-0402 claim 14 (slaves search, walk, load one bale per processing up to their storage, walk back and deliver: paid to the miner's owner; inside, the reload delay; health restored at the first update after it)
// spec: S-0209 claim 4 row WP2 (the returning slave enters its master's unloading cell)
#[test]
fn claim_14_slave_cycle_income() {
    let r = rules();
    let mut w = game_on(&r, &field(5, -1, 5), false);
    let b = working_building(&mut w);
    let credits = w.houses[0].credits;
    let s = slaves::slaves_of(&w, b)[0];
    until(&mut w, 600, |w| w.houses[0].credits > credits).expect("income");
    let paid = w.houses[0].credits - credits;
    assert_eq!(paid % (STORE * ORE_VALUE), 0);
    assert!(paid > 0);
    // Wait for this slave's delivery.
    until(&mut w, 600, |w| {
        let g = slaves::group_of(w, b).unwrap();
        g.members[0].inside
    })
    .expect("slave 0 went inside");
    assert!(!w.obj(s).unwrap().on_map);
    let g = group(&w, b);
    assert!(!g.members[0].ready);
    let since = g.members[0].since;
    w.obj_mut(s).unwrap().health = 10;
    until(&mut w, 200, |w| {
        slaves::group_of(w, b).unwrap().members[0].ready
    })
    .expect("reloaded");
    assert!(w.tick - since >= RELOAD);
    assert!(w.tick - since < RELOAD + 10);
    assert_eq!(w.obj(s).unwrap().health, 100);
}

// spec: S-0402 claims 14, 21 and 23 (a lost slave starts the regeneration delay; its replacement, created at the first update after it, belongs to the master's current house; existing slaves keep their owner)
// spec: S-0402 claim 20 (a replacement does not raise the ever-built count)
#[test]
fn claims_14_23_regeneration_and_capture() {
    let r = rules();
    let mut w = game_on(&r, &field(5, -1, 5), false);
    let b = place_building(&mut w, "SMB", 0, at(0, -1));
    let ids = slaves::slaves_of(&w, b);
    glasnost_sim::objects::remove(&mut w, ids[1]);
    let lost_at = w.tick;
    assert_eq!(group(&w, b).members[1].id, None);
    let old = w.obj(b).unwrap().owner;
    w.obj_mut(b).unwrap().owner = Some(HouseId(1));
    glasnost_sim::econ::events::owner_changed(&mut w, b, old);
    let t = until(&mut w, 200, |w| {
        slaves::group_of(w, b).unwrap().members[1].id.is_some()
    })
    .expect("replaced");
    assert!(t - lost_at >= REGEN);
    let g = group(&w, b);
    let n = g.members[1].id.unwrap();
    assert_eq!(w.obj(n).unwrap().owner, Some(HouseId(1)));
    for i in [0, 2] {
        let o = w.obj(g.members[i].id.unwrap()).unwrap();
        assert_eq!(o.owner, Some(HouseId(0)));
    }
    assert_eq!(tech::ever_built(&w, HouseId(1), slave_type(&w)), 0);
}

// spec: S-0402 claim 22 (an explicitly directed slave miner searches for a site near the destination, moves there with move queued and travels; with no destination it attempts deployment; success puts the group into working; the slaves then mine)
#[test]
fn claim_22_directed_harvest_deploys() {
    let r = rules();
    let mut w = game_on(&r, &field(5, -1, 5), false);
    let m = place_vehicle(&mut w, "SMV", 0, at(-8, 6), 0);
    unblock(&mut w);
    let credits = w.houses[0].credits;
    assert!(orders::issue(
        &mut w,
        HouseId(0),
        Order::Behaviour {
            actor: m,
            behaviour: Behaviour::Harvest,
            attack_move: false,
            target: None,
            dest: Some(Target::Cell(at(6, 0))),
        }
    ));
    until(&mut w, 30, |w| {
        slaves::group_of(w, m).is_some_and(|g| g.state == GroupState::Travelling)
    })
    .expect("travelling");
    let dest = w.obj(m).unwrap().dest;
    let Some(Target::Cell(site)) = dest else {
        panic!("a site destination, not {dest:?}")
    };
    assert_ne!(site, at(6, 0));
    until(&mut w, 1500, |w| w.obj(m).is_none()).expect("deployed");
    let b = w
        .objects
        .values()
        .find(|o| o.kind == Kind::Building && o.econ.group.is_some())
        .map(|o| o.id)
        .expect("the deployed master");
    until(&mut w, 1500, |w| w.houses[0].credits > credits).expect("income");
    let g = group(&w, b);
    assert_eq!(slaves::slaves_of(&w, b).len(), SLAVES as usize, "{g:?}");
}

// spec: S-0402 claim 22 (at each processing of harvest: with a destination a site search near it, move to the site and travelling; without one guard queued, at rest with last-rest the current tick; in every case the standard delay with its one ranged draw, S-0215 claim 6)
#[test]
fn claim_22_harvest_processing() {
    let r = rules();
    let mut w = game_on(&r, &field(5, -1, 5), false);
    let m = place_vehicle(&mut w, "SMV", 0, at(-8, 6), 0);
    testkit::run(&mut w, 3);
    w.obj_mut(m).unwrap().beh.queued = None;
    w.obj_mut(m).unwrap().beh.current = Some(Behaviour::Harvest);
    let before = w.rng.taken();
    assert!(slaves::process_harvest(&mut w, m).is_some());
    assert_eq!(w.rng.taken() - before, 1);
    assert_eq!(w.obj(m).unwrap().beh.queued, Some(Behaviour::Guard));
    assert_eq!(group(&w, m).state, GroupState::AtRest);
    assert_eq!(group(&w, m).last_rest, w.tick);
    w.obj_mut(m).unwrap().dest = Some(Target::Cell(at(6, 0)));
    for _ in 0..2 {
        let before = w.rng.taken();
        assert!(slaves::process_harvest(&mut w, m).is_some());
        assert_eq!(w.rng.taken() - before, 1);
        assert_eq!(group(&w, m).state, GroupState::Travelling);
        let Some(Target::Cell(site)) = w.obj(m).unwrap().dest else {
            panic!("a site")
        };
        assert_ne!(site, at(6, 0));
        // The next processing searches again, near the site now held.
    }
}

// spec: S-0402 claim 24 (the top-left cell of a free W × H rectangle near the centre, no overlay; the vehicle destination shifted by one along each axis whose size exceeds 2)
#[test]
fn claim_24_site_search() {
    let r = rules();
    let mut w = game_on(&r, &field(5, -1, 5), false);
    let m3 = place_vehicle(&mut w, "SMV", 0, at(-8, 6), 0);
    let m2 = place_vehicle(&mut w, "SMV2", 0, at(-8, 8), 0);
    let c = at(-2, -6);
    assert_eq!(
        slaves::site_search(&mut w, m3, c),
        Some((c, CellXY::new(c.x + 1, c.y + 1)))
    );
    assert_eq!(slaves::site_search(&mut w, m2, c), Some((c, c)));
    // Near the field: no rectangle cell holds ore.
    let (tl, _) = slaves::site_search(&mut w, m3, at(6, 0)).unwrap();
    for dy in 0..3 {
        for dx in 0..3 {
            let x = CellXY::new(tl.x + dx, tl.y + dy);
            assert!(stage(&w, x).is_none(), "{x:?} holds ore");
        }
    }
}

// spec: S-0402 F7 (packing up exactly when c + d(S, F) < d(A, F): equality stays; coordinate differences and the root reduced to signed 16 bits)
#[test]
fn f7_relocation_comparison() {
    let r = rules();
    let w = game_on(&r, &[], false);
    let mode = w.rules().mode;
    let p = |x, y| CellXY::new(x, y);
    let f7 = |c, s, a, f| slaves::f7_packs_up(c, s, a, f, false, mode);
    assert!(!f7(3, p(3, 0), p(0, 0), p(10, 0)));
    assert!(f7(3, p(4, 0), p(0, 0), p(10, 0)));
    assert!(!f7(0, p(10, 0), p(10, 0), p(10, 0)));
    // d(S, F) = 0 (−65536 reduces to 0) and d(A, F) = 5 (−65541 to −5).
    assert!(f7(0, p(5, 0), p(0, 0), p(65541, 0)));
    assert!(!f7(5, p(5, 0), p(0, 0), p(65541, 0)));
    // d(S, F) = 2 (−65534 reduces to 2) and d(A, F) = 4 (−65540 to −4).
    assert!(f7(0, p(6, 0), p(0, 0), p(65540, 0)));
    // d = ⌊√(2 × 32767²)⌋ = 46339 reduces to −19197: nothing is below it.
    assert!(!f7(0, p(0, 0), p(0, 0), p(32767, 32767)));
}

// spec: S-0312 claim 30 (a killing hit on a master: hidden slaves are removed, slaves on the map lose their master and go to the attacker's house, target and destination cleared, guard queued and cheer requested)
#[test]
fn claim_30_destroyed_masters() {
    let r = rules();
    let mut w = game_on(&r, &field(5, -1, 5), false);
    let attacker = place_vehicle(&mut w, "HARV", 1, at(-8, -8), 0);
    let v = place_vehicle(&mut w, "SMV", 0, at(-8, 8), 0);
    let hidden = slaves::slaves_of(&w, v);
    let b = working_building(&mut w);
    let out = on_map(&w, b);
    let hit = |a| Hit {
        amount: 100_000,
        warhead: None,
        distance: 0,
        attacker: Some(a),
        house: None,
        ignore_defences: true,
        no_escape: false,
    };
    damage::apply(&mut w, v, hit(attacker));
    for s in hidden {
        assert!(w.obj(s).is_none());
    }
    damage::apply(&mut w, b, hit(attacker));
    for s in out {
        let o = w.obj(s).unwrap();
        assert_eq!(o.owner, Some(HouseId(1)));
        assert!(!slaves::is_slave(&w, s));
        assert_eq!(o.target, None);
        assert_eq!(o.dest, None);
        assert!(o.beh.queued == Some(Behaviour::Guard) || o.beh.current == Some(Behaviour::Guard));
        // spec: S-0312 claim 30 (a successful transfer requests cheer after the reset)
        assert_eq!(
            o.infantry.as_ref().unwrap().seq,
            Some(glasnost_sim::infantry::Seq::Cheer)
        );
    }
}

/// A working SMH with its slaves out, and the slaves.
fn sold(civilian: bool) -> (World, Vec<ObjectId>) {
    sold_on(&rules(), civilian)
}

/// As [`sold`], on the ruleset `r`.
fn sold_on(r: &Ruleset, civilian: bool) -> (World, Vec<ObjectId>) {
    let mut w = game_on(r, &field(5, -1, 5), civilian);
    let b = place_building(&mut w, "SMH", 0, at(0, -1));
    until(&mut w, 40, |w| on_map(w, b).len() == SLAVES as usize).expect("emerged");
    let out = on_map(&w, b);
    unblock(&mut w);
    assert!(orders::issue(&mut w, HouseId(0), Order::Sell { actor: b }));
    until(&mut w, 200, |w| w.obj(b).is_none()).expect("sold");
    (w, out)
}

// spec: S-0408 claim 10a (a true sale: slaves on the map go to the first civilian-side house, guard; no master; with no such house a hit of their health with the demolition warhead)
#[test]
fn claim_10a_sold_master() {
    let (w, out) = sold(true);
    for &s in &out {
        let o = w.obj(s).unwrap();
        assert_eq!(o.owner, Some(HouseId(2)));
        assert!(!slaves::is_slave(&w, s));
        assert!(o.beh.queued == Some(Behaviour::Guard) || o.beh.current == Some(Behaviour::Guard));
    }
    let (w, out) = sold(false);
    for &s in &out {
        assert!(w.obj(s).is_none_or(|o| o.health == 0 || o.combat.killed));
    }
}

// spec: S-0408 claim 10a (r15, Q-0440: a sold master's on-map slaves given to the civilian-side house have target and destination cleared, guard queued and then a cheer request; the no-recipient hit requests no cheer and does not itself reset target or destination or queue guard)
#[test]
fn claim_10a_sold_master_cheer_and_no_recipient() {
    let (w, out) = sold(true);
    assert!(!out.is_empty());
    for &s in &out {
        let o = w.obj(s).unwrap();
        assert_eq!(o.target, None);
        assert_eq!(o.dest, None);
        assert_eq!(
            o.infantry.as_ref().unwrap().seq,
            Some(glasnost_sim::infantry::Seq::Cheer)
        );
    }
    // No recipient, with a demolition warhead that does no damage: the
    // slaves survive the bare hit, keep their owner and get no cheer.
    let mut r = rules();
    r.sim.warheads.get_mut("DEMO").unwrap().verses =
        Some((0..11).map(|_| Decimal("0".into())).collect());
    let (w, out) = sold_on(&r, false);
    assert!(!out.is_empty());
    for &s in &out {
        let o = w.obj(s).unwrap();
        assert!(o.health > 0);
        assert_eq!(o.owner, Some(HouseId(0)));
        assert!(!slaves::is_slave(&w, s));
        assert_ne!(
            o.infantry.as_ref().unwrap().seq,
            Some(glasnost_sim::infantry::Seq::Cheer)
        );
    }
    // The no-recipient path at the master's sale itself: the slave's
    // destination stays and no guard is queued.
    let mut w = game_on(&r, &field(5, -1, 5), false);
    let b = place_building(&mut w, "SMH", 0, at(0, -1));
    until(&mut w, 40, |w| !on_map(w, b).is_empty()).expect("emerged");
    let s = on_map(&w, b)[0];
    let spot = Some(Target::Cell(at(-3, -3)));
    if let Some(o) = w.obj_mut(s) {
        o.dest = spot;
        o.beh.queued = None;
    }
    let target = w.obj(s).unwrap().target;
    slaves::master_sold(&mut w, b);
    let o = w.obj(s).unwrap();
    assert!(!slaves::is_slave(&w, s));
    assert_eq!(o.dest, spot);
    assert_eq!(o.target, target);
    assert_eq!(o.beh.queued, None);
}

/// One repeating tag `TAG` over one trigger whose only event is
/// destroyed-by-anything (S-0011 claim 19), with no actions.
fn destroyed_tag() -> Triggers {
    Triggers {
        triggers: vec![TriggerDef {
            events: vec![Event {
                code: 48, // spec: S-0312 Import mapping (event code 48, destroyed by anything)
                kind: 0,
                value: 0,
                type_name: None,
            }],
            ..TriggerDef::created(b"T".to_vec())
        }],
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

/// An SMH placed with its slaves still inside, one of them carrying the
/// tag of [`destroyed_tag`]; then `end` ends the master. Returns the world,
/// the hidden slaves, the owner's added losses and the tag's added firings.
fn hidden_group_ends(
    r: &Ruleset,
    end: impl FnOnce(&mut World, ObjectId),
) -> (World, Vec<ObjectId>, i32, u64) {
    let mut w = game_with_triggers(r, &field(5, -1, 5), true, destroyed_tag());
    unblock(&mut w);
    let b = place_building(&mut w, "SMH", 0, at(0, -1));
    let hidden = slaves::slaves_of(&w, b);
    assert_eq!(hidden.len(), SLAVES as usize);
    assert!(on_map(&w, b).is_empty());
    glasnost_sim::triggers::object_tag(&mut w, hidden[0], Some(b"TAG"));
    let losses = stats::of(&w, HouseId(0)).losses;
    let fired = w.triggers.fired;
    end(&mut w, b);
    let l = stats::of(&w, HouseId(0)).losses - losses;
    let f = w.triggers.fired - fired;
    (w, hidden, l, f)
}

// spec: S-0408 claim 10a (r15, Q-0440: a true sale destroys every hidden slave without placing it, a loss each for its house without attacker credit, with S-0312 claim 29's common destruction notifications without an attacker, as for S-0312 claim 31's removal without a killing hit)
// spec: S-0312 claim 31 (r16: a tagged hidden slave raises destroyed-by-anything without receiving any damage)
#[test]
fn claim_10a_sold_master_hidden_slaves_destroyed() {
    let r = rules();
    let (w, hidden, losses, fired) = hidden_group_ends(&r, |w, b| {
        assert!(orders::issue(w, HouseId(0), Order::Sell { actor: b }));
        until(w, 300, |w| w.obj(b).is_none()).expect("sold");
    });
    for &s in &hidden {
        assert!(w.obj(s).is_none(), "hidden slave {s:?} destroyed");
    }
    assert_eq!(losses, SLAVES, "one loss per hidden slave");
    let (_, _, removed_losses, removed_fired) =
        hidden_group_ends(&r, |w, b| glasnost_sim::objects::remove(w, b));
    assert_eq!(removed_losses, SLAVES);
    assert!(removed_fired > 0, "the tagged hidden slave raises its tag");
    assert_eq!(fired, removed_fired, "the same notifications as a removal");
}

// spec: S-0312 claim 30 (r16: a tagged hidden slave of a master with a killing hit takes claim 29's common group with the slave as causing object, the master's known attacker enabling the preliminary raises; destroyed-by-anything is raised with or without an attacker)
#[test]
fn s0312_claim_30_tagged_hidden_slave_raises() {
    let r = rules();
    for with_attacker in [true, false] {
        let (w, hidden, losses, fired) = hidden_group_ends(&r, |w, b| {
            let attacker = with_attacker.then(|| place_vehicle(w, "HARV", 1, at(-8, -8), 0));
            damage::apply(
                w,
                b,
                Hit {
                    amount: 100_000,
                    warhead: None,
                    distance: 0,
                    attacker,
                    house: None,
                    ignore_defences: true,
                    no_escape: false,
                },
            );
        });
        for &s in &hidden {
            assert!(w.obj(s).is_none());
        }
        // The building itself is one more loss.
        assert_eq!(losses, SLAVES + 1, "attacker {with_attacker}");
        assert!(fired > 0, "attacker {with_attacker}");
    }
}

// spec: S-0312 claim 30 and scenario D21 variant b (r16: the initial lethal hit of a delay-kill warhead on an eligible building master is a reprieve that neither frees nor removes its group; the actual death at expiry ends it under claim 30)
// spec: S-0302 claim 16 (the expiry hit: whole health, the demolition warhead, no attacker)
#[test]
fn s0312_claim_30_delayed_death_master() {
    let mut r = rules();
    r.sim
        .buildings
        .get_mut("SMH")
        .unwrap()
        .eligible_for_delay_kill = Some(true);
    r.sim.warheads.insert(
        "DK".into(),
        WarheadType {
            verses: Some((0..11).map(|_| Decimal("1".into())).collect()),
            cell_spread: dec("2"),
            causes_delay_kill: Some(true),
            delay_kill_frames: Some(5),
            delay_kill_at_max: dec("7"),
            ..Default::default()
        },
    );
    let dk = r.sim.warheads.keys().position(|k| k == "DK").unwrap() as u32;
    let mut w = game_on(&r, &field(5, -1, 5), true);
    let b = place_building(&mut w, "SMH", 0, at(0, -1));
    until(&mut w, 40, |w| on_map(w, b).len() == SLAVES as usize).expect("emerged");
    let ids = slaves::slaves_of(&w, b);
    let out = on_map(&w, b);
    damage::apply(
        &mut w,
        b,
        Hit {
            amount: 100_000,
            warhead: Some(dk),
            distance: 0,
            attacker: None,
            house: None,
            ignore_defences: false,
            no_escape: false,
        },
    );
    assert!(w.obj(b).unwrap().combat.delay_kill_until.is_some());
    assert_eq!(
        slaves::slaves_of(&w, b),
        ids,
        "the reprieve keeps the group"
    );
    for &s in &out {
        assert_eq!(w.obj(s).unwrap().owner, Some(HouseId(0)));
    }
    until(&mut w, 40, |w| w.obj(b).is_none()).expect("dies at expiry");
    for &s in &out {
        let o = w.obj(s).unwrap();
        assert!(!slaves::is_slave(&w, s));
        assert_eq!(o.owner, Some(HouseId(2)), "no attacker: the civilian side");
    }
}

/// The ticks from the start of `b`'s build-down to its removal.
fn build_down_span(w: &mut World, b: ObjectId) -> i32 {
    let mut down = None;
    for _ in 0..400 {
        let t = w.tick;
        testkit::run(w, 1);
        match w.obj(b) {
            Some(o) => {
                if let Some(glasnost_sim::econ::sell::Stage::BuildDown(x)) =
                    o.econ.sale.clone().map(|s| s.stage)
                {
                    down = Some(x);
                }
            }
            None => return t - down.expect("build-down"),
        }
    }
    panic!("not removed");
}

// spec: S-0408 claim 17 (r15: the guard spot that suppresses the stage-23 early report includes automatic relocation's remembered old top-left cell, S-0402 claim 28; the build-down then completes at its last stage)
// spec: S-0415 claim 10 (r9: automatic packing stores the old top-left cell in the guard spot, which the sale does not clear)
// spec: S-0108 claim 5 (60 image frames: a 30-stage range)
#[test]
fn s0408_claim_17_automatic_guard_spot_suppresses_early_report() {
    let mut r = rules();
    r.sim.buildings.get_mut("SMB").unwrap().econ.buildup_frames = Some(60);
    // Control: a manual sale without a guard spot reports at stage 23.
    let mut w = game_on(&r, &field(5, -1, 5), false);
    let b = working_building(&mut w);
    unblock(&mut w);
    assert_eq!(w.obj(b).unwrap().guard_spot, None);
    assert!(orders::issue(&mut w, HouseId(0), Order::Sell { actor: b }));
    let early = build_down_span(&mut w, b);
    // Automatic relocation: the remembered cell is the guard spot.
    let (mut w, b, a) = no_field_building(&r);
    until(&mut w, 15, |w| {
        w.obj(b).is_none() || group(w, b).state == GroupState::Packing
    })
    .expect("packing");
    assert_eq!(w.obj(b).unwrap().guard_spot, Some(Target::Cell(a)));
    let full = build_down_span(&mut w, b);
    assert!(early > 0);
    assert_eq!(early * 29, full * 23, "stage 23 against stage 29");
}

// spec: S-0202 claim 10 (r19, packing up a slave miner: the return vehicle's and its temporary hidden slaves' creation draws come before placement, and with quirks.failed_packup_keeps_vehicle off refusal still takes them)
// spec: S-0415 claim 11 (r9: switching the quirk off removes the refused vehicle and its new group without suppressing their creation or its draws)
#[test]
fn s0202_claim_10_refused_pack_up_draws_match_with_quirk_off() {
    let r = rules();
    let (_, on) = pack_up_now(&r, true);
    let mut off = rules();
    off.globals.quirks.failed_packup_keeps_vehicle = false;
    let (_, off) = pack_up_now(&off, true);
    assert_eq!(on.draws, off.draws);
    // The successful pack-up takes the same creation draws as well.
    let (_, ok) = pack_up_now(&r, false);
    assert!(ok.draws >= SLAVES as u64);
}

/// The vehicle master of the game (after a pack-up).
fn vehicle_master(w: &World) -> ObjectId {
    w.objects
        .values()
        .find(|o| o.kind == Kind::Vehicle && o.econ.group.is_some())
        .map(|o| o.id)
        .expect("the vehicle master")
}

fn recalled(w: &World, s: ObjectId) -> bool {
    slaves::slave_of(w, s).is_some_and(|x| x.recalled)
}

// spec: S-0402 claim 28 and S-0415 claim 10 (a manual pack-up on clear ground: the vehicle takes the same slaves; a group still checking the field becomes at rest at the vehicle's update, last-rest that update, with no recall)
#[test]
fn claim_28_manual_pack_up_rests_without_recall() {
    let r = rules();
    let mut w = game_on(&r, &field(5, -1, 5), false);
    let b = working_building(&mut w);
    let ids = slaves::slaves_of(&w, b);
    unblock(&mut w);
    assert!(orders::issue(&mut w, HouseId(0), Order::Sell { actor: b }));
    until(&mut w, 200, |w| w.obj(b).is_none()).expect("packed up");
    let v = vehicle_master(&w);
    assert_eq!(slaves::slaves_of(&w, v), ids);
    assert_eq!(group(&w, v).state, GroupState::CheckingField);
    let t = until(&mut w, 20, |w| {
        group(w, v).state != GroupState::CheckingField
    })
    .expect("updated");
    let g = group(&w, v);
    assert_eq!(g.state, GroupState::AtRest);
    assert_eq!(g.last_rest, t - 1);
    for &s in &ids {
        assert!(!recalled(&w, s));
    }
}

/// A deployed SMB at top-left `at(0, −1)` with no resource anywhere, run
/// until its group's second update; and its top-left cell A.
fn no_field_building(r: &Ruleset) -> (World, ObjectId, CellXY) {
    let mut w = game_on(r, &[], false);
    let b = place_building(&mut w, "SMB", 0, at(0, -1));
    let a = w.obj(b).unwrap().pos.cell();
    until(&mut w, 30, |w| {
        w.obj(b).is_none() || group(w, b).state == GroupState::CheckingField
    })
    .expect("checking");
    (w, b, a)
}

// spec: S-0402 claim 25 (no field and no site: the comparison uses cell (0, 0), F7 holds, an automatic pack-up with A remembered); S-0415 claim 10 (the vehicle gets A as its destination with move queued in the conversion tick)
// spec: S-0402 claims 22 and 27 (the packing group at the first update that sees the vehicle: recall and field seeking, no search in that update)
#[test]
fn claim_25_failed_search_packs_toward_origin() {
    let r = rules();
    let (mut w, b, a) = no_field_building(&r);
    assert!(w.rules().quirks.slave_relocation_failed_search_origin);
    until(&mut w, 15, |w| {
        w.obj(b).is_none() || group(w, b).state == GroupState::Packing
    })
    .expect("packing");
    // spec: S-0402 claim 28 (the automatic old-cell memory is the building's guard spot)
    assert_eq!(w.obj(b).unwrap().guard_spot, Some(Target::Cell(a)));
    // spec: S-0402 claim 25 (with a packing request no emergence)
    assert!(on_map(&w, b).is_empty());
    until(&mut w, 200, |w| w.obj(b).is_none()).expect("packed up");
    let v = vehicle_master(&w);
    let o = w.obj(v).unwrap();
    assert_eq!(o.dest, Some(Target::Cell(a)));
    assert_eq!(o.beh.queued, Some(Behaviour::Move));
    assert_eq!(group(&w, v).state, GroupState::Packing);
    until(&mut w, 20, |w| group(w, v).state != GroupState::Packing).expect("updated");
    assert_eq!(group(&w, v).state, GroupState::FieldSeeking);
}

// spec: S-0402 claim 25 (quirks.slave_relocation_failed_search_origin off: a failed field or site search leaves the miner deployed; with no packing request, emergence still happens and the check repeats at every update)
#[test]
fn claim_25_quirk_off_stays_deployed() {
    let mut r = rules();
    r.globals.quirks.slave_relocation_failed_search_origin = false;
    let (mut w, b, _) = no_field_building(&r);
    until(&mut w, 15, |w| !on_map(w, b).is_empty()).expect("emerged");
    testkit::run(&mut w, 100);
    assert!(w.obj(b).is_some());
    assert_eq!(group(&w, b).state, GroupState::CheckingField);
}

// spec: S-0402 claim 26 (a deployed master's scans centre on the cell of its reference point and need only tiberium land: ore two cells beyond the centre of a 3 × 3 master, outside the reach of its top-left and unloading cells, is a short-scan field)
#[test]
fn claim_26_deployed_scan_centre() {
    let mut r = rules();
    r.sim.general.economy.slave_miner_short_scan = dec("3");
    r.sim.general.economy.slave_miner_long_scan = dec("0");
    let mut w = game_on(&r, &[(at(-1, 2), ORE, 5)], false);
    let b = place_building(&mut w, "SMB", 0, at(0, -1));
    until(&mut w, 30, |w| !on_map(w, b).is_empty()).expect("emerged");
    assert_eq!(group(&w, b).state, GroupState::CheckingField);
}

// spec: S-0402 claim 27 (an at-rest group of a building whose current behaviour is selling remains at rest)
#[test]
fn claim_27_at_rest_while_selling() {
    let r = rules();
    let mut w = game_on(&r, &field(5, -1, 5), false);
    unblock(&mut w);
    let b = place_building(&mut w, "SMH", 0, at(0, -1));
    let due = group(&w, b).next_update;
    assert!(orders::issue(&mut w, HouseId(0), Order::Sell { actor: b }));
    while w.tick <= due {
        testkit::run(&mut w, 1);
        if w.obj(b).is_none() {
            break;
        }
        assert_eq!(group(&w, b).state, GroupState::AtRest);
        assert!(on_map(&w, b).is_empty());
    }
    assert!(
        w.tick > due,
        "the building was gone before its first update"
    );
}

// spec: S-0402 claim 28 (the first processing of selling or packing of a slave master whose remembered destination is a resource-land cell other than its own top-left cell: the directed site rule; a found site replaces the destination, the slaves are recalled, the group enters packing); S-0408 claim 6; S-0415 claim 10 (the vehicle's destination)
#[test]
fn claim_28_directed_pack_up() {
    let r = rules();
    let mut w = game_on(&r, &field(5, -1, 5), false);
    let b = working_building(&mut w);
    let out = on_map(&w, b);
    unblock(&mut w);
    w.obj_mut(b).unwrap().guard_spot = Some(Target::Cell(at(6, 0)));
    assert!(orders::issue(&mut w, HouseId(0), Order::Sell { actor: b }));
    until(&mut w, 30, |w| group(w, b).state == GroupState::Packing).expect("packing");
    let Some(Target::Cell(site)) = w.obj(b).unwrap().guard_spot else {
        panic!("a site")
    };
    assert_ne!(site, at(6, 0));
    for &s in &out {
        assert!(recalled(&w, s));
    }
    until(&mut w, 200, |w| w.obj(b).is_none()).expect("packed up");
    let v = vehicle_master(&w);
    assert_eq!(w.obj(v).unwrap().dest, Some(Target::Cell(site)));
}

// spec: S-0402 claim 28 (a remembered destination that is not resource land: no change, no recall)
#[test]
fn claim_28_non_resource_destination_changes_nothing() {
    let r = rules();
    let mut w = game_on(&r, &field(5, -1, 5), false);
    let b = working_building(&mut w);
    let out = on_map(&w, b);
    unblock(&mut w);
    let spot = Some(Target::Cell(at(-5, -5)));
    w.obj_mut(b).unwrap().guard_spot = spot;
    assert!(orders::issue(&mut w, HouseId(0), Order::Sell { actor: b }));
    until(&mut w, 30, |w| {
        w.obj(b)
            .is_some_and(|o| o.beh.current == Some(Behaviour::Selling))
    })
    .expect("selling");
    testkit::run(&mut w, 1);
    assert_ne!(group(&w, b).state, GroupState::Packing);
    assert_eq!(w.obj(b).unwrap().guard_spot, spot);
    for &s in &out {
        assert!(!recalled(&w, s));
    }
}

// spec: S-0402 claim 26 (a slave's harvest processing: on a non-resource cell, guard queued with a 1-tick delay; capacity 0, guard queued with a 1-tick delay and nothing taken; a negative capacity under quirks.slave_negative_capacity_stalls stays in harvest with delay h and takes nothing, with the quirk off it is treated as 0)
#[test]
fn claim_26_slave_processing_capacity() {
    for (storage, stalls) in [(0, true), (-1, true), (-1, false)] {
        let mut r = rules();
        r.sim.infantry.get_mut("SLAVE").unwrap().storage = Some(storage);
        r.globals.quirks.slave_negative_capacity_stalls = stalls;
        let mut w = game_on(&r, &field(5, -1, 5), false);
        let b = working_building(&mut w);
        let s = on_map(&w, b)[0];
        if let Some(o) = w.obj_mut(s) {
            o.dest = None;
            o.beh.queued = None;
        }
        let c = w.obj(s).unwrap().pos.cell();
        // A non-resource cell: guard queued, 1 tick.
        assert_eq!(slaves::process_slave(&mut w, s), 1);
        assert_eq!(w.obj(s).unwrap().beh.queued, Some(Behaviour::Guard));
        w.obj_mut(s).unwrap().beh.queued = None;
        put(&mut w, c, ORE, 5);
        let d = slaves::process_slave(&mut w, s);
        assert_eq!(stage(&w, c), Some(5));
        assert!(
            slaves::slave_of(&w, s)
                .unwrap()
                .load
                .iter()
                .all(|&x| x == 0)
        );
        if storage < 0 && stalls {
            assert_eq!(d, 3, "harvest rate h");
            assert_eq!(w.obj(s).unwrap().beh.queued, None);
        } else {
            assert_eq!(d, 1);
            assert_eq!(w.obj(s).unwrap().beh.queued, Some(Behaviour::Guard));
        }
    }
}

// spec: S-0312 claim 31 (a removal without a killing hit: hidden slaves destroyed without credit, a loss each; slaves on the map go to the first civilian-side house, target and destination reset, guard queued and cheer requested)
#[test]
fn s0312_claim_31_removed_master() {
    let r = rules();
    let mut w = game_on(&r, &field(5, -1, 5), true);
    let v = place_vehicle(&mut w, "SMV", 0, at(-8, 8), 0);
    let hidden = slaves::slaves_of(&w, v);
    let b = working_building(&mut w);
    let out = on_map(&w, b);
    let lost = stats::of(&w, HouseId(0)).losses;
    glasnost_sim::objects::remove(&mut w, v);
    for &s in &hidden {
        assert!(w.obj(s).is_none());
    }
    assert_eq!(stats::of(&w, HouseId(0)).losses, lost + SLAVES);
    glasnost_sim::objects::remove(&mut w, b);
    for &s in &out {
        let o = w.obj(s).unwrap();
        assert_eq!(o.owner, Some(HouseId(2)));
        assert!(!slaves::is_slave(&w, s));
        assert_eq!(o.target, None);
        assert_eq!(o.dest, None);
        assert_eq!(o.beh.queued, Some(Behaviour::Guard));
        assert_eq!(
            o.infantry.as_ref().unwrap().seq,
            Some(glasnost_sim::infantry::Seq::Cheer)
        );
    }
}

// spec: S-0312 claim 31 (no recipient: the bare hit of the slave's health with the demolition warhead, no reset; a warhead that does no damage leaves it alive with its owner)
#[test]
fn s0312_claim_31_no_recipient() {
    let r = rules();
    let mut w = game_on(&r, &field(5, -1, 5), false);
    let b = working_building(&mut w);
    let out = on_map(&w, b);
    glasnost_sim::objects::remove(&mut w, b);
    for &s in &out {
        assert!(w.obj(s).is_none_or(|o| o.health == 0 || o.combat.killed));
    }
    let mut r = rules();
    r.sim.warheads.get_mut("DEMO").unwrap().verses =
        Some((0..11).map(|_| Decimal("0".into())).collect());
    let mut w = game_on(&r, &field(5, -1, 5), false);
    let b = working_building(&mut w);
    let out = on_map(&w, b);
    glasnost_sim::objects::remove(&mut w, b);
    for &s in &out {
        let o = w.obj(s).unwrap();
        assert!(o.health > 0);
        assert_eq!(o.owner, Some(HouseId(0)));
        assert!(!slaves::is_slave(&w, s));
        assert_ne!(
            o.infantry.as_ref().unwrap().seq,
            Some(glasnost_sim::infantry::Seq::Cheer)
        );
    }
}

/// A working SMB whose return cell holds another house's vehicle, sold:
/// the pack-up fails (S-0415 claims 11 and 12). The old slaves on the map
/// and the world.
fn failed_pack_up(r: &Ruleset) -> (World, Vec<ObjectId>, Vec<ObjectId>) {
    let mut w = game_on(r, &field(5, -1, 5), true);
    let b = working_building(&mut w);
    let ids = slaves::slaves_of(&w, b);
    let out = on_map(&w, b);
    unblock(&mut w);
    place_vehicle(&mut w, "HARV", 1, at(1, 0), 0);
    assert!(orders::issue(&mut w, HouseId(0), Order::Sell { actor: b }));
    until(&mut w, 200, |w| w.obj(b).is_none()).expect("removed");
    (w, ids, out)
}

// spec: S-0415 claim 11 and S-0312 claim 31 (a refused pack-up does not transfer the old group: its slaves on the map go to the civilian-side house; the retained vehicle keeps its own newly created group, off the map)
#[test]
fn s0415_claim_11_failed_pack_up_ends_old_group() {
    let r = rules();
    let (w, ids, out) = failed_pack_up(&r);
    for &s in &out {
        assert_eq!(w.obj(s).unwrap().owner, Some(HouseId(2)));
        assert!(!slaves::is_slave(&w, s));
    }
    let smv = w.rules().index_of(Kind::Vehicle, "SMV").unwrap();
    let kept: Vec<_> = w
        .objects
        .values()
        .filter(|o| o.kind == Kind::Vehicle && o.type_index == smv)
        .collect();
    assert_eq!(kept.len(), 1);
    assert!(!kept[0].on_map);
    let new = slaves::slaves_of(&w, kept[0].id);
    assert_eq!(new.len(), SLAVES as usize);
    assert!(new.iter().all(|s| !ids.contains(s)));
}

// spec: S-0415 claim 11 (quirks.failed_packup_keeps_vehicle off: neither the vehicle nor a group of its own remains; the old group still ends by S-0312 claim 31)
#[test]
fn s0415_claim_11_failed_pack_up_quirk_off() {
    let mut r = rules();
    r.globals.quirks.failed_packup_keeps_vehicle = false;
    let (w, _, out) = failed_pack_up(&r);
    for &s in &out {
        assert_eq!(w.obj(s).unwrap().owner, Some(HouseId(2)));
    }
    let smv = w.rules().index_of(Kind::Vehicle, "SMV").unwrap();
    assert!(
        w.objects
            .values()
            .all(|o| o.kind != Kind::Vehicle || o.type_index != smv)
    );
    assert!(w.objects.values().all(|o| o.econ.group.is_none()));
}

/// What one direct pack-up of a working SMB did: the building's old
/// slaves, the first object id the pack-up allocated (the vehicle), the
/// random draws it took and the owner's losses it added.
struct PackUp {
    old: Vec<ObjectId>,
    first: u32,
    draws: u64,
    losses: i32,
}

/// Packs up a working SMB at once (S-0415 claims 10 to 12), its return
/// cell blocked by another house's vehicle when `blocked`.
fn pack_up_now(r: &Ruleset, blocked: bool) -> (World, PackUp) {
    let mut w = game_on(r, &field(5, -1, 5), true);
    let b = working_building(&mut w);
    unblock(&mut w);
    if blocked {
        place_vehicle(&mut w, "HARV", 1, at(1, 0), 0);
    }
    let old = slaves::slaves_of(&w, b);
    let first = w.next_id;
    let taken = w.rng.taken();
    let losses = stats::of(&w, HouseId(0)).losses;
    glasnost_sim::econ::deploy::pack_up(&mut w, b);
    assert!(w.obj(b).is_none());
    let p = PackUp {
        old,
        first,
        draws: w.rng.taken() - taken,
        losses: stats::of(&w, HouseId(0)).losses - losses,
    };
    (w, p)
}

/// The ids the return vehicle's own initial slaves were created with: the
/// ones right after the vehicle's.
fn temporary_ids(p: &PackUp) -> Vec<ObjectId> {
    (1..=SLAVES as u32).map(|n| ObjectId(p.first + n)).collect()
}

// spec: S-0402 claim 19a (a successful pack-up creates the return vehicle and its own initial hidden slaves, each with its creation draw, before placement; they are destroyed and removed with S-0312 claim 31's notifications, counted as losses, before the old working group is transferred; their draws are not undone)
// spec: S-0415 claim 10 (r9: the return vehicle and its initial hidden slaves are created before placement; on success those slaves are destroyed and removed, with losses and no undone draws, before the old working group transfers; no old slave is recreated)
// spec: S-0312 claim 31 (r16, temporary hidden slaves during successful pack-up: losses without attacker credit, the old group survives the transfer)
#[test]
fn claim_19a_successful_pack_up_discards_temporary_slaves() {
    let r = rules();
    let (w, p) = pack_up_now(&r, false);
    let v = ObjectId(p.first);
    assert_eq!(w.obj(v).unwrap().kind, Kind::Vehicle);
    assert!(w.obj(v).unwrap().on_map);
    assert_eq!(slaves::slaves_of(&w, v), p.old, "the old group transferred");
    for s in temporary_ids(&p) {
        assert!(w.obj(s).is_none(), "temporary slave {s:?} removed");
    }
    assert!(p.draws > SLAVES as u64, "the creation draws are taken");
    assert_eq!(p.losses, SLAVES, "the discarded slaves count as losses");
}

// spec: S-0402 claim 19a and S-0415 claim 11 (a refused pack-up under quirks.failed_packup_keeps_vehicle: the vehicle's new hidden group, created before placement with its draws, is kept; only the old group ends)
#[test]
fn claim_19a_refused_pack_up_keeps_new_group() {
    let r = rules();
    let (w, p) = pack_up_now(&r, true);
    let v = ObjectId(p.first);
    assert!(!w.obj(v).unwrap().on_map);
    assert_eq!(slaves::slaves_of(&w, v), temporary_ids(&p));
    for s in &p.old {
        assert!(!slaves::is_slave(&w, *s) || w.obj(*s).is_none());
    }
    assert!(p.draws > SLAVES as u64);
}

// spec: S-0402 claim 19a (quirks.failed_packup_keeps_vehicle off: the already-created vehicle and its new group are removed on refusal; their creation draws still occur)
#[test]
fn claim_19a_refused_pack_up_quirk_off_draws_still_taken() {
    let mut r = rules();
    r.globals.quirks.failed_packup_keeps_vehicle = false;
    let (w, p) = pack_up_now(&r, true);
    assert!(w.obj(ObjectId(p.first)).is_none());
    for s in temporary_ids(&p) {
        assert!(w.obj(s).is_none());
    }
    assert!(p.draws > SLAVES as u64);
    assert!(w.objects.values().all(|o| o.econ.group.is_none()));
}

/// A placed SMB whose group is put into `state` (test set-up only: a
/// building reaches these states when its vehicle deployed another way),
/// run through its next group update. Returns the building and the tick of
/// that update.
fn building_in_state(w: &mut World, state: GroupState) -> (ObjectId, i32) {
    let b = place_building(w, "SMB", 0, at(0, -1));
    w.obj_mut(b).unwrap().econ.group.as_mut().unwrap().state = state;
    let due = group(w, b).next_update;
    while w.tick <= due {
        testkit::run(w, 1);
    }
    (b, due)
}

// spec: S-0402 claim 27 (field seeking, building, with a field and site: the site offered as the building's destination, move queued without an immediate start, travelling; no emergence or field-check transition)
#[test]
fn claim_27_seeking_building_with_a_site_travels() {
    let r = rules();
    let mut w = game_on(&r, &field(5, -1, 5), false);
    let (b, _) = building_in_state(&mut w, GroupState::FieldSeeking);
    assert_eq!(group(&w, b).state, GroupState::Travelling);
    let o = w.obj(b).unwrap();
    assert!(matches!(o.dest, Some(Target::Cell(_))));
    assert_eq!(o.beh.queued, Some(Behaviour::Move));
    assert!(on_map(&w, b).is_empty());
    // spec: S-0402 claim 27 (the next update sees travelling and puts the group at rest)
    let next = group(&w, b).next_update;
    while w.tick <= next {
        testkit::run(&mut w, 1);
    }
    let g = group(&w, b);
    assert_eq!(g.state, GroupState::AtRest);
    assert_eq!(g.last_rest, next);
    assert!(on_map(&w, b).is_empty());
}

// spec: S-0402 claim 27 (field seeking, building, with no field: at rest, this tick recorded as last-rest, no emergence)
#[test]
fn claim_27_seeking_building_without_a_field_rests() {
    let r = rules();
    let mut w = game_on(&r, &[], false);
    let (b, due) = building_in_state(&mut w, GroupState::FieldSeeking);
    let g = group(&w, b);
    assert_eq!(g.state, GroupState::AtRest);
    assert_eq!(g.last_rest, due);
    assert_eq!(w.obj(b).unwrap().dest, None);
    assert!(on_map(&w, b).is_empty());
}

// spec: S-0402 claim 27 (travelling or retrying deployment, building: at rest with this tick as last-rest, without field search, deployment attempt or emergence)
#[test]
fn claim_27_travelling_or_retrying_building_rests() {
    for state in [GroupState::Travelling, GroupState::Retrying] {
        let r = rules();
        let mut w = game_on(&r, &field(5, -1, 5), false);
        let (b, due) = building_in_state(&mut w, state);
        let g = group(&w, b);
        assert_eq!(g.state, GroupState::AtRest, "{state:?}");
        assert_eq!(g.last_rest, due, "{state:?}");
        assert_eq!(w.obj(b).unwrap().dest, None, "{state:?}");
        assert!(on_map(&w, b).is_empty(), "{state:?}");
    }
}

// spec: S-0402 claim 28 (one remembered destination: automatic relocation replaces an earlier guard spot with the old top-left cell; a later order that changes the guard spot replaces that cell, and the pack-up heads there)
#[test]
fn claim_28_one_remembered_destination() {
    let r = rules();
    let (mut w, b, a) = no_field_building(&r);
    unblock(&mut w);
    let manual = Target::Cell(at(-6, -6));
    assert!(orders::issue(
        &mut w,
        HouseId(0),
        Order::GuardSpot {
            actor: b,
            spot: manual
        }
    ));
    until(&mut w, 15, |w| {
        w.obj(b).is_none() || group(w, b).state == GroupState::Packing
    })
    .expect("packing");
    assert_eq!(w.obj(b).unwrap().guard_spot, Some(Target::Cell(a)));
    let later = Target::Cell(at(-7, -7));
    assert!(orders::issue(
        &mut w,
        HouseId(0),
        Order::GuardSpot {
            actor: b,
            spot: later
        }
    ));
    testkit::run(&mut w, 1);
    assert_eq!(w.obj(b).unwrap().guard_spot, Some(later));
    until(&mut w, 200, |w| w.obj(b).is_none()).expect("packed up");
    let v = vehicle_master(&w);
    assert_eq!(w.obj(v).unwrap().dest, Some(later));
}

/// A vehicle master whose slaves are out (handed over from a working
/// building, test set-up only).
fn vehicle_with_slaves_out(w: &mut World) -> (ObjectId, Vec<ObjectId>) {
    let b = working_building(w);
    let v = place_vehicle(w, "SLAVEM", 0, at(-8, 8), 0);
    slaves::transfer(w, b, v);
    let out = on_map(w, v);
    assert_eq!(out.len(), SLAVES as usize);
    unblock(w);
    (v, out)
}

// spec: S-0213 claim 8 (stopping a vehicle that controls slave units sends its slaves back to guard it)
#[test]
fn s0213_claim_8_stop_recalls() {
    let r = rules();
    let mut w = game_on(&r, &field(5, -1, 5), false);
    let (v, out) = vehicle_with_slaves_out(&mut w);
    assert!(orders::issue(&mut w, HouseId(0), Order::Stop { actor: v }));
    testkit::run(&mut w, 1);
    for s in out {
        assert!(slaves::slave_of(&w, s).unwrap().recalled);
        assert_eq!(
            slaves::slave_of(&w, s).unwrap().phase,
            SlavePhase::Returning
        );
    }
}

// spec: S-0213 table T3 (h) (a behaviour order other than attack on a slave master sends its slaves back)
#[test]
fn s0213_t3_h_move_recalls() {
    let r = rules();
    let mut w = game_on(&r, &field(5, -1, 5), false);
    let (v, out) = vehicle_with_slaves_out(&mut w);
    assert!(orders::issue(
        &mut w,
        HouseId(0),
        Order::Behaviour {
            actor: v,
            behaviour: Behaviour::Move,
            attack_move: false,
            target: None,
            dest: Some(Target::Cell(at(-6, 8))),
        }
    ));
    testkit::run(&mut w, 1);
    for s in out {
        assert!(slaves::slave_of(&w, s).unwrap().recalled);
    }
}

// spec: S-0213 claim 5 and table T3 (h) (generic slave recall excludes a resulting attack behaviour even for a mobile master)
#[test]
fn s0213_t3_h_attack_keeps_slaves_out() {
    let r = rules();
    let mut w = game_on(&r, &field(5, -1, 5), false);
    let (v, out) = vehicle_with_slaves_out(&mut w);
    assert!(orders::issue(
        &mut w,
        HouseId(0),
        Order::Behaviour {
            actor: v,
            behaviour: Behaviour::Attack,
            attack_move: false,
            target: Some(Target::Cell(at(-6, 8))),
            dest: None,
        }
    ));
    testkit::run(&mut w, 1);
    for s in out {
        assert!(!slaves::slave_of(&w, s).unwrap().recalled);
    }
}

// spec: S-0213 claim 8 (the slave-recall row applies only to vehicles: a stop order to a deployed slave-miner building does not itself recall its slaves) and claim 5 (generic recall excludes buildings)
#[test]
fn s0213_claims_5_8_building_master_keeps_slaves_out() {
    let r = rules();
    let mut w = game_on(&r, &field(5, -1, 5), false);
    let b = working_building(&mut w);
    let out = on_map(&w, b);
    assert!(!out.is_empty());
    assert!(orders::issue(&mut w, HouseId(0), Order::Stop { actor: b }));
    testkit::run(&mut w, 1);
    for &s in &out {
        assert!(!slaves::slave_of(&w, s).unwrap().recalled);
    }
    orders::issue(
        &mut w,
        HouseId(0),
        Order::Behaviour {
            actor: b,
            behaviour: Behaviour::Guard,
            attack_move: false,
            target: None,
            dest: None,
        },
    );
    testkit::run(&mut w, 1);
    for &s in &out {
        assert!(!slaves::slave_of(&w, s).unwrap().recalled);
    }
}

// spec: S-0402 claims 14 and 22 (in guard, past the kick delay, at rest and on tiberium land, the slaves wake: the group starts field seeking and then travels to a site)
#[test]
fn claim_14_wake_starts_field_seeking() {
    let mut r = rules();
    r.sim.general.economy.slave_miner_kick_frame_delay = Some(5);
    let mut w = game_on(&r, &field(5, -1, 5), false);
    testkit::run(&mut w, 2);
    let m = place_vehicle(&mut w, "SMV", 0, at(6, 0), 0);
    assert!(at_rest(&w, m));
    until(&mut w, 100, |w| !slaves::at_rest(w, m)).expect("woken");
    until(&mut w, 100, |w| group(w, m).state == GroupState::Travelling).expect("travelling");
}

fn at_rest(w: &World, m: ObjectId) -> bool {
    slaves::at_rest(w, m)
}

// spec: S-0402 claims 14 and 19 (the slave state is game state: the hash survives a save and load while slaves work)
#[test]
fn slave_state_round_trip() {
    let r = rules();
    let mut w = game_on(&r, &field(5, -1, 5), false);
    working_building(&mut w);
    testkit::run(&mut w, 25);
    let bytes = glasnost_sim::save::save(&w);
    let mut back =
        glasnost_sim::save::load(&bytes, w.rules_arc(), w.catalogue_arc()).expect("load");
    assert_eq!(
        glasnost_sim::save::state_hash(&back),
        glasnost_sim::save::state_hash(&w)
    );
    testkit::run(&mut w, 100);
    testkit::run(&mut back, 100);
    assert_eq!(
        glasnost_sim::save::state_hash(&back),
        glasnost_sim::save::state_hash(&w)
    );
}

/// A slave of `m` that stands delivery-ready: returning, on the map, in
/// the unloading cell and not between steps.
fn delivery_ready(w: &World, m: ObjectId) -> Option<ObjectId> {
    let c = slaves::unloading_cell(w, m)?;
    on_map(w, m).into_iter().find(|&s| {
        let o = w.obj(s).unwrap();
        let st = slaves::slave_of(w, s).unwrap();
        (st.phase == SlavePhase::Returning || st.recalled)
            && o.pos.cell() == c
            && !glasnost_sim::occupancy::in_progress(w, s)
    })
}

// spec: S-0203 claim 12 (a returning slave delivers in its master's group update, not in its own turn: income comes only in due update ticks, and the slave waits in the unloading cell until then)
#[test]
fn s0203_claim_12_delivery_in_group_update() {
    let r = rules();
    let mut w = game_on(&r, &field(5, -1, 5), false);
    let b = working_building(&mut w);
    let mut deliveries = 0;
    let mut waited = false;
    for _ in 0..800 {
        let due = group(&w, b).next_update;
        let t = w.tick;
        let credits = w.houses[0].credits;
        if delivery_ready(&w, b).is_some() && t < due {
            waited = true;
        }
        testkit::run(&mut w, 1);
        if w.houses[0].credits > credits {
            deliveries += 1;
            assert!(t >= due, "a delivery in tick {t}, the update due in {due}");
            assert_eq!(group(&w, b).next_update, t + slaves::PERIOD);
        }
    }
    assert!(deliveries > 0);
    assert!(waited, "a slave stood ready between updates");
}

// spec: S-0203 claim 12 (paid building repair comes after the group update in the master's turn: a delivery in that update pays for the same turn's repair step)
// spec: S-0314 claim 8 (available credits below the step cost turn repair off)
#[test]
fn s0203_claim_12_delivery_before_repair() {
    let mut r = rules();
    // P_b = trunc(0.002 × 900) = 1: a repair step in every tick (S-0314 F1).
    r.sim.general.repair_rate = Some(Decimal("0.002".into()));
    r.sim.general.repair_step = Some(10);
    let mut w = game_on(&r, &field(5, -1, 5), false);
    let b = working_building(&mut w);
    assert_eq!(w.rules().combat.general.repair_period, 1);
    until(&mut w, 800, |w| {
        delivery_ready(w, b).is_some() && w.tick == group(w, b).next_update
    })
    .expect("a slave ready for the due update");
    let s = delivery_ready(&w, b).unwrap();
    w.houses[0].credits = 0;
    w.obj_mut(b).unwrap().health = 100;
    let mut twin = w.clone();
    w.obj_mut(b).unwrap().econ.building.repairing = true;
    testkit::run(&mut w, 1);
    testkit::run(&mut twin, 1);
    assert!(!w.obj(s).unwrap().on_map, "delivered");
    let o = w.obj(b).unwrap();
    assert_eq!(o.health, 110, "the repair step was paid from the delivery");
    assert!(o.econ.building.repairing);
    let cost = glasnost_sim::combat::heal::step_cost(&w, b).unwrap();
    assert!(cost > 0);
    assert_eq!(twin.houses[0].credits - w.houses[0].credits, cost);
}

// spec: S-0203 claim 12 (a vehicle master's group update sees the destination before the vehicle's movement: arriving in the update's tick does not deploy it then; it deploys at the next due update with no destination)
#[test]
fn s0203_claim_12_vehicle_arrival_deploys_at_next_update() {
    let r = rules();
    let mut w = game_on(&r, &[], false);
    let m = place_vehicle(&mut w, "SMV", 0, at(-2, 0), 0);
    unblock(&mut w);
    assert!(orders::issue(
        &mut w,
        HouseId(0),
        Order::Behaviour {
            actor: m,
            behaviour: Behaviour::Move,
            attack_move: false,
            target: None,
            dest: Some(Target::Cell(at(0, 0))),
        }
    ));
    testkit::run(&mut w, 1);
    // Travelling, with no group update before the arrival.
    {
        let t = w.tick;
        let g = w.obj_mut(m).unwrap().econ.group.as_mut().unwrap();
        g.state = GroupState::Travelling;
        g.next_update = t + 1000;
    }
    // Find the arrival tick, then replay it with a due group update.
    let mut prev = w.clone();
    for _ in 0..300 {
        prev = w.clone();
        testkit::run(&mut w, 1);
        if w.obj(m).unwrap().dest.is_none() {
            break;
        }
    }
    assert!(prev.obj(m).unwrap().dest.is_some());
    let mut w = prev;
    let t = w.tick;
    w.obj_mut(m)
        .unwrap()
        .econ
        .group
        .as_mut()
        .unwrap()
        .next_update = t;
    testkit::run(&mut w, 1);
    let o = w.obj(m).expect("not deployed in the arrival tick");
    assert_eq!(o.dest, None, "arrived");
    assert_eq!(group(&w, m).state, GroupState::Travelling);
    assert_eq!(group(&w, m).next_update, t + slaves::PERIOD);
    let travelling =
        |w: &World| slaves::group_of(w, m).map(|g| g.state) == Some(GroupState::Travelling);
    while w.tick < t + slaves::PERIOD {
        testkit::run(&mut w, 1);
        assert!(travelling(&w), "no attempt before the next due update");
    }
    // The deployment attempt at the next due update (deployed, or turning to deploy).
    testkit::run(&mut w, 1);
    assert!(!travelling(&w), "the attempt at the next due update");
}

// spec: S-0203 claim 12 (a destination the group assigns is available to the vehicle's movement in the same turn)
#[test]
fn s0203_claim_12_vehicle_moves_in_assignment_tick() {
    let r = rules();
    let mut w = game_on(&r, &field(5, -1, 5), false);
    let m = place_vehicle(&mut w, "SMV", 0, at(-8, 0), 0);
    testkit::run(&mut w, 1);
    {
        let t = w.tick;
        let g = w.obj_mut(m).unwrap().econ.group.as_mut().unwrap();
        g.state = GroupState::FieldSeeking;
        g.next_update = t;
    }
    let before = w.obj(m).unwrap().clone();
    testkit::run(&mut w, 1);
    assert_eq!(group(&w, m).state, GroupState::Travelling);
    let o = w.obj(m).unwrap();
    assert!(o.dest.is_some());
    assert!(
        o.pos != before.pos || !o.path.steps.is_empty(),
        "the movement of the assignment tick took up the new destination"
    );
}

// spec: S-0203 claim 12 and S-0402 claim 21 (a slave received in an update completes reload only at a later update, even with reload delay 0)
#[test]
fn s0203_claim_12_received_slave_reloads_later() {
    let mut r = rules();
    r.sim
        .buildings
        .get_mut("SMB")
        .unwrap()
        .econ
        .slave_reload_rate = Some(0);
    let mut w = game_on(&r, &field(5, -1, 5), false);
    let b = working_building(&mut w);
    until(&mut w, 800, |w| {
        delivery_ready(w, b).is_some() && w.tick == group(w, b).next_update
    })
    .expect("a slave ready for the due update");
    let s = delivery_ready(&w, b).unwrap();
    let i = group(&w, b)
        .members
        .iter()
        .position(|m| m.id == Some(s))
        .unwrap();
    testkit::run(&mut w, 1);
    let m = group(&w, b).members[i].clone();
    assert!(m.inside && !m.ready, "not reloaded in its delivery update");
    until(&mut w, slaves::PERIOD, |w| {
        let m = &group(w, b).members[i];
        m.ready || !m.inside
    })
    .expect("reloaded at the next update");
}

// spec: S-0203 claims 10 and 12 (taking a slave inside is a removal during the master's turn: it costs the object directly after the master its turn only when the slave stood before the master in update order)
#[test]
fn s0203_claim_12_delivery_removal_counts_before_master() {
    let r = rules();
    let pos = |w: &World, x: ObjectId| w.update.order.iter().position(|&y| y == x);
    // Slaves earlier than the master: a vehicle created after them takes the group.
    let mut w = game_on(&r, &field(5, -1, 5), false);
    let b = working_building(&mut w);
    let v = place_vehicle(&mut w, "SMV", 0, at(-6, 6), 0);
    let obs = place_vehicle(&mut w, "CART", 0, at(-6, 9), 0);
    slaves::transfer(&mut w, b, v);
    {
        let g = w.obj_mut(v).unwrap().econ.group.as_mut().unwrap();
        g.state = GroupState::AtRest;
    }
    assert_eq!(pos(&w, obs), pos(&w, v).map(|p| p + 1));
    slaves::recall(&mut w, v);
    let mut skipped = 0;
    for _ in 0..600 {
        let t = w.tick;
        let inside = group(&w, v).members.iter().filter(|m| m.inside).count();
        let earlier = slaves::slaves_of(&w, v)
            .iter()
            .all(|&s| pos(&w, s).is_some_and(|p| Some(p) < pos(&w, v)));
        testkit::run(&mut w, 1);
        let now = group(&w, v).members.iter().filter(|m| m.inside).count();
        if now > inside {
            assert!(earlier);
            assert_ne!(
                w.obj(obs).unwrap().last_turn,
                t,
                "the observer lost its turn"
            );
            skipped += 1;
        }
        if now == SLAVES as usize {
            break;
        }
    }
    assert!(skipped > 0);

    // Control: slaves that emerged after their master cost nobody a turn.
    let mut w = game_on(&r, &field(5, -1, 5), false);
    let b = place_building(&mut w, "SMB", 0, at(0, -1));
    let obs = place_vehicle(&mut w, "CART", 0, at(-6, 9), 0);
    assert_eq!(pos(&w, obs), pos(&w, b).map(|p| p + 1));
    let mut delivered = 0;
    for _ in 0..800 {
        let t = w.tick;
        let credits = w.houses[0].credits;
        testkit::run(&mut w, 1);
        if w.houses[0].credits > credits {
            assert_eq!(w.obj(obs).unwrap().last_turn, t, "no turn lost");
            delivered += 1;
        }
    }
    assert!(delivered > 0);
}

// spec: S-0208 claim 4 and S-0402 claim 21 (an emerging slave starts facing 0, map north, whatever its master's facing)
#[test]
fn s0208_claim_4_emergence_faces_north() {
    let r = rules();
    let mut w = game_on(&r, &field(5, -1, 5), false);
    let b = testkit::spawn(&mut w, Kind::Building, "SMB", 0, at(0, -1), 16384);
    assert_ne!(w.obj(b).unwrap().body.value(w.tick), 0);
    let mut seen = 0;
    for _ in 0..40 {
        let before = on_map(&w, b);
        testkit::run(&mut w, 1);
        for s in on_map(&w, b) {
            if !before.contains(&s) {
                assert_eq!(w.obj(s).unwrap().body.value(w.tick), 0);
                seen += 1;
            }
        }
    }
    assert_eq!(seen, SLAVES);
}

// spec: S-0209 claim 4 (slave field qualification: a partly occupied infantry cell qualifies when the slave's infantry entry outcome there is clear; another unit is not a blanket refusal)
#[test]
fn s0209_claim_4_partly_occupied_field_cell_qualifies() {
    let r = rules();
    let ore = at(4, 0);
    let mut w = game_on(&r, &[(ore, ORE, 5)], false);
    let other = testkit::spawn(&mut w, Kind::Infantry, "WALKER", 0, ore, 0);
    let b = place_building(&mut w, "SMH", 0, at(0, -1));
    let t = until(&mut w, 400, |w| {
        on_map(w, b).into_iter().any(|s| {
            w.obj(s).unwrap().pos.cell() == ore
                && slaves::slave_of(w, s).unwrap().phase == SlavePhase::Loading
        })
    });
    assert!(
        t.is_some(),
        "a slave loads in the cell the other unit stands in"
    );
    assert_eq!(w.obj(other).unwrap().pos.cell(), ore);
}
