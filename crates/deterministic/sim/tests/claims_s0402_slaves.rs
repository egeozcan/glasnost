//! Claim tests for slave miners and their slaves (S-0402 claims 14 and 19
//! to 25, F7; S-0312 claim 30; S-0408 claim 10a; S-0213 claim 8 and table
//! T3 (h); S-0209 claim 4 row WP2) on synthetic data: an invented slave
//! infantry type, slave-miner vehicle and building types and values, no
//! stock data. Houses Alpha (0) and Beta (1) of the testkit's sandbox game,
//! both human.

mod harvest_common;

use std::sync::Arc;

use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{Country, Decimal, LocomotionClass, Locomotor, UnitType, WarheadType};
use glasnost_sim::combat::damage::{self, Hit};
use glasnost_sim::econ::slaves::{self, GroupState, SlaveGroup, SlavePhase};
use glasnost_sim::econ::{TypeRef, tech};
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
    let mut map = testkit::empty_map(SIZE);
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
    assert_eq!(w.houses[0].credits, credits);
}

// spec: S-0402 claim 19 (a count of 0 or less creates no slave)
#[test]
fn claim_19_no_count() {
    let r = rules();
    let mut w = game_on(&r, &[], false);
    let m = place_vehicle(&mut w, "SLAVEM", 0, at(0, 0), 0);
    assert!(group(&w, m).members.is_empty());
}

// spec: S-0402 claims 21 and 22 (a deployed master with a short-scan field works; each ready slave is offered the foundation cell (W − 1, ⌊H/2⌋) in reverse creation order, one ranged draw each)
#[test]
fn claim_21_building_emergence() {
    let r = rules();
    let mut w = game_on(&r, &field(5, -1, 5), false);
    testkit::run(&mut w, 3);
    let b = place_building(&mut w, "SMB", 0, at(1, -1));
    let ids = slaves::slaves_of(&w, b);
    let created = w.tick;
    let mut draws = 0;
    while group(&w, b).state == GroupState::AtRest {
        assert!(on_map(&w, b).is_empty());
        let before = w.rng.taken();
        testkit::run(&mut w, 1);
        draws = w.rng.taken() - before;
    }
    // The first update is in the tick numbered 10 after the creation tick's.
    assert_eq!(w.tick - created, 10 + 1);
    assert_eq!(group(&w, b).state, GroupState::Working);
    let out = on_map(&w, b);
    assert_eq!(out.len(), SLAVES as usize);
    let unloading = at(3, 0);
    for &s in &out {
        assert_eq!(w.obj(s).unwrap().pos.cell(), unloading);
        assert_eq!(w.obj(s).unwrap().target, Some(Target::Object(b)));
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
    while w.tick < due {
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
    for o in w.objects.values() {
        eprintln!(
            "{:?} {:?} {} {:?} {:?} {:?}",
            o.id,
            o.kind,
            o.type_index,
            o.pos.cell(),
            o.beh.current,
            o.econ.group.as_ref().map(|g| g.state)
        );
    }
    let g = group(&w, b);
    assert_eq!(slaves::slaves_of(&w, b).len(), SLAVES as usize, "{g:?}");
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

// spec: S-0312 claim 30 (a killing hit on a master: hidden slaves are removed, slaves on the map lose their master and go to the attacker's house, target and destination cleared and guard queued)
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
    }
}

/// A working SMH with its slaves out, and the slaves.
fn sold(civilian: bool) -> (World, Vec<ObjectId>) {
    let r = rules();
    let mut w = game_on(&r, &field(5, -1, 5), civilian);
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

// spec: S-0402 claim 22 and S-0415 claim 10 (packing up hands the group to the vehicle, no new slaves; its first group update recalls the slaves, which go inside, and starts field seeking)
#[test]
fn claim_22_pack_up_transfers_and_recalls() {
    let r = rules();
    let mut w = game_on(&r, &field(5, -1, 5), false);
    let b = working_building(&mut w);
    let ids = slaves::slaves_of(&w, b);
    unblock(&mut w);
    assert!(orders::issue(&mut w, HouseId(0), Order::Sell { actor: b }));
    until(&mut w, 200, |w| w.obj(b).is_none()).expect("packed up");
    let v = w
        .objects
        .values()
        .find(|o| o.kind == Kind::Vehicle && o.econ.group.is_some())
        .map(|o| o.id)
        .expect("the vehicle master");
    assert_eq!(slaves::slaves_of(&w, v), ids);
    until(&mut w, 20, |w| {
        ids.iter()
            .all(|&s| !w.obj(s).unwrap().on_map || slaves::slave_of(w, s).unwrap().recalled)
    })
    .expect("recalled");
    until(&mut w, 600, |w| {
        group(w, v).members.iter().all(|m| m.inside)
    })
    .expect("inside");
}

/// A vehicle master whose slaves are out (handed over from a working
/// building, test set-up only).
fn vehicle_with_slaves_out(w: &mut World) -> (ObjectId, Vec<ObjectId>) {
    let b = working_building(w);
    let v = place_vehicle(w, "SLAVEM", 0, at(-8, 8), 0);
    slaves::transfer(w, b, v);
    w.obj_mut(v)
        .unwrap()
        .econ
        .group
        .as_mut()
        .unwrap()
        .pending_seek = false;
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
