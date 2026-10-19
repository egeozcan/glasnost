//! Local relation tests on the reference ruleset (M7b step 4b, PLAN §M7).
//! Never in CI: every test is `#[ignore]` and needs `GLASNOST_GAME_DIR`.
//!
//! One test per pair row of `docs/m7-claims/pairs-m7a.md` and
//! `docs/m7-claims/pairs-m7b.md`, named after the synthetic test of that
//! row (`m7a_…` and `m7b_…`). Each loads the installed rules once
//! (`glasnost_tools::arena::reference_rules`), builds a flat map with two
//! human player houses (as `arena::setup_trial` does), finds the types it
//! needs by their rules flags at run time (never by name) and checks the
//! relation's direction against a control in the same world: protected
//! against open, before against after, never a rules number. Where the
//! installed data cannot form the pair the test prints why and passes as
//! skipped. Each test prints one verdict line,
//! `relation <test>: PASS`, `relation <test>: PASS (…)` or
//! `relation <test>: SKIP <reason>`, with no type names or rules values.
//!
//! Run with `GLASNOST_GAME_DIR=<your install> cargo test --release -p
//! glasnost-tools --test relation_lane -- --ignored --nocapture
//! --test-threads=1`.

#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;
use std::sync::{Arc, OnceLock};

use glasnost_map::CellXY;
use glasnost_sim::combat::check::{self, Answer};
use glasnost_sim::combat::damage::{self, Hit};
use glasnost_sim::combat::effects::Effect;
use glasnost_sim::combat::fire;
use glasnost_sim::econ::power;
use glasnost_sim::econ::release::{self, Outcome};
use glasnost_sim::mutation::MakerEvent;
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, FactoryKind, Kind, SimRules, UnitRules};
use glasnost_sim::structures::{absorb, bunker};
use glasnost_sim::superweapons::{self, SwKind};
use glasnost_sim::targeting::{self, ClickOn, Modifiers};
use glasnost_sim::world::{
    GameSetup, HouseId, HouseRole, HouseSetup, ObjectId, Pos, Target, World,
};
use glasnost_sim::{
    domination, drain, gas, gattling, lift, mind, mutation, objects, parasite, psychic, testkit,
    tick, transport,
};
use glasnost_tools::arena;
use glasnost_tools::local_lane::GAME_DIR_VAR;

const SIZE: i32 = 48;
const A: u8 = 0;
const B: u8 = 1;
/// The neutral house of `world_with_neutral`.
const N: u8 = 2;

// ---------------------------------------------------------------------------
// The harness

static RULES: OnceLock<Option<Arc<SimRules>>> = OnceLock::new();

/// The reference rules, loaded once; `None` without an install. Rules that
/// do not resolve are a finding: the test fails.
fn reference() -> Option<Arc<SimRules>> {
    RULES
        .get_or_init(|| {
            let dir = std::env::var_os(GAME_DIR_VAR)?;
            match arena::reference_rules(Path::new(&dir)) {
                Err(e) => {
                    eprintln!("the install cannot be read ({:?})", e.kind());
                    None
                }
                Ok(Err(e)) => panic!("the reference rules do not resolve: {e}"),
                Ok(Ok(r)) => Some(r),
            }
        })
        .clone()
}

fn pass(name: &str) {
    println!("relation {name}: PASS");
}

/// Skips the test with a reason (no type names, no values).
macro_rules! need {
    ($name:expr, $e:expr, $why:expr) => {
        match $e {
            Some(x) => x,
            None => {
                println!("relation {}: SKIP {}", $name, $why);
                return;
            }
        }
    };
}

/// The rules' first country that is not passive, nor the neutral or
/// special one (as the arena's default).
fn default_country(r: &SimRules) -> Option<String> {
    r.country_names
        .iter()
        .zip(&r.countries)
        .find(|(n, c)| {
            !c.passive
                && n.as_str() != glasnost_sim::load::NEUTRAL
                && n.as_str() != glasnost_sim::load::SPECIAL
        })
        .map(|(n, _)| n.clone())
}

/// A flat map with houses A and B (human players of the default country,
/// not allied), run past the opening ticks; with `neutral`, also the
/// neutral and special houses (2 and 3, not human).
fn build(r: &Arc<SimRules>, seed: u32, neutral: bool) -> World {
    let country = default_country(r).expect("a player country");
    let house = |name: &str, country: &str, human: bool, role: HouseRole| HouseSetup {
        name: name.into(),
        country: country.into(),
        human,
        start: None,
        role,
        ai: false,
    };
    let mut houses = vec![
        house("A", &country, true, HouseRole::Player),
        house("B", &country, true, HouseRole::Player),
    ];
    if neutral {
        houses.push(house(
            "Neutral",
            glasnost_sim::load::NEUTRAL,
            false,
            HouseRole::Neutral,
        ));
        houses.push(house(
            "Special",
            glasnost_sim::load::SPECIAL,
            false,
            HouseRole::Special,
        ));
    }
    let setup = GameSetup {
        seed,
        game_speed: r.general.lobby_game_speed,
        houses,
        alliances: Vec::new(),
        sandbox: true,
        unit_count: 0,
        bases: false,
        skirmish: Default::default(),
    };
    let mut w = glasnost_sim::load::new_game(
        r.clone(),
        &testkit::empty_map(SIZE),
        testkit::catalogue(),
        &setup,
    )
    .expect("the reference game loads");
    testkit::run(&mut w, 5);
    w
}

fn world(r: &Arc<SimRules>) -> World {
    build(r, 1, false)
}

fn at(dx: i32, dy: i32) -> CellXY {
    testkit::mid(SIZE, dx, dy)
}

fn put(w: &mut World, kind: Kind, name: &str, house: u8, dx: i32, dy: i32) -> ObjectId {
    testkit::spawn(w, kind, name, house, at(dx, dy), 0)
}

fn owner(w: &World, id: ObjectId) -> Option<u8> {
    w.obj(id).and_then(|o| o.owner).map(|h| h.0)
}

fn health(w: &World, id: ObjectId) -> i32 {
    w.obj(id).map_or(0, |o| o.health)
}

/// The full strength of `id`'s type.
fn strength(w: &World, id: ObjectId) -> i32 {
    let Some(o) = w.obj(id) else { return 0 };
    let r = w.rules();
    match o.kind {
        Kind::Building => r.buildings[o.type_index as usize].strength,
        k => r.unit(k, o.type_index).map_or(0, |u| u.strength),
    }
}

/// Keeps the objects alive at full health (the scene shows a relation,
/// not a kill).
fn heal(w: &mut World, ids: &[ObjectId]) {
    for &id in ids {
        let s = strength(w, id);
        if let Some(o) = w.obj_mut(id)
            && o.health > 0
        {
            o.health = s;
        }
    }
}

fn run_until(w: &mut World, max: i32, done: impl Fn(&World) -> bool) -> bool {
    for _ in 0..max {
        if done(w) {
            return true;
        }
        tick::step(w);
    }
    done(w)
}

/// As `run_until`, healing `keep` before every tick.
fn run_healed(w: &mut World, max: i32, keep: &[ObjectId], done: impl Fn(&World) -> bool) -> bool {
    for _ in 0..max {
        if done(w) {
            return true;
        }
        heal(w, keep);
        tick::step(w);
    }
    done(w)
}

fn click(w: &mut World, house: u8, sel: &[ObjectId], on: ObjectId) {
    let r = targeting::click(
        w,
        HouseId(house),
        sel,
        ClickOn::Object(on),
        Modifiers::default(),
    );
    for o in r.orders {
        orders::issue(w, HouseId(house), o);
    }
}

fn attack(w: &mut World, house: u8, actor: ObjectId, target: ObjectId) {
    orders::issue(
        w,
        HouseId(house),
        Order::Behaviour {
            actor,
            behaviour: Behaviour::Attack,
            attack_move: false,
            target: Some(Target::Object(target)),
            dest: None,
        },
    );
}

fn move_to(w: &mut World, house: u8, actor: ObjectId, c: CellXY) {
    orders::issue(
        w,
        HouseId(house),
        Order::Behaviour {
            actor,
            behaviour: Behaviour::Move,
            attack_move: false,
            target: None,
            dest: Some(Target::Cell(c)),
        },
    );
}

fn kill(w: &mut World, id: ObjectId) {
    let amount = health(w, id).max(1) * 10;
    damage::apply(
        w,
        id,
        Hit {
            amount,
            warhead: None,
            distance: 0,
            attacker: None,
            house: None,
            ignore_defences: true,
            no_escape: false,
        },
    );
}

// ---------------------------------------------------------------------------
// Type finders, by flags

/// The infantry and vehicle types as (kind, index).
fn units(r: &SimRules, kinds: &[Kind]) -> Vec<(Kind, u32)> {
    let mut v = Vec::new();
    for &k in kinds {
        let n = match k {
            Kind::Infantry => r.infantry.len(),
            Kind::Vehicle => r.vehicles.len(),
            _ => 0,
        };
        v.extend((0..n as u32).map(|i| (k, i)));
    }
    v
}

fn find_unit(
    r: &SimRules,
    kinds: &[Kind],
    f: impl Fn(&UnitRules) -> bool,
) -> Option<(Kind, String)> {
    units(r, kinds)
        .into_iter()
        .find(|&(k, i)| r.unit(k, i).is_some_and(&f))
        .map(|(k, i)| (k, r.unit(k, i).unwrap().name.clone()))
}

fn find_building(
    r: &SimRules,
    f: impl Fn(&glasnost_sim::rules::BuildingRules) -> bool,
) -> Option<String> {
    r.buildings.iter().find(|b| f(b)).map(|b| b.name.clone())
}

/// The warhead of a unit's first weapon.
fn warhead0<'a>(
    r: &'a SimRules,
    u: &UnitRules,
) -> Option<&'a glasnost_sim::combat::data::WarheadRules> {
    let p = u.combat.slots[0]?;
    r.combat.weapon(p).warhead.map(|h| r.combat.warhead(h))
}

/// An ordinary ground unit: some strength, not psionics-immune, no
/// harvester, no jumpjet, no passengers, no deploying, not naval.
fn ordinary(u: &UnitRules) -> bool {
    u.strength > 0
        && !u.combat.immune_psionics
        && !u.harvester
        && !u.jumpjet
        && u.passengers == 0
        && !u.deployer
        && !u.combat.naval
        && u.class != Some(glasnost_rules::sim::LocomotionClass::Hover)
        && !u.raw.deploys_into.as_deref().is_some_and(|d| !d.is_empty())
}

fn is_controller(r: &SimRules, u: &UnitRules, unlimited: bool) -> bool {
    u.combat.slots[0].is_some_and(|p| {
        let wr = r.combat.weapon(p);
        wr.infinite_mind_control == unlimited
            && wr.warhead.is_some_and(|h| r.combat.warhead(h).mind_control)
    })
}

/// A plain unit of `kinds` for a victim: ordinary, no special first weapon
/// (no takeover, berserk, seizing, parasite or drain); unarmed first.
fn victim(r: &SimRules, kinds: &[Kind]) -> Option<(Kind, String)> {
    let plain = |u: &UnitRules| {
        ordinary(u)
            && !u.fraidycat
            && !u.engineer
            && !u.raw.not_human.unwrap_or(false)
            && !u.combat.immune_psionics
            && warhead0(r, u)
                .is_none_or(|h| !h.mind_control && !h.psychedelic && !h.is_locomotor && !h.parasite)
    };
    find_unit(r, kinds, |u| plain(u) && u.combat.slots[0].is_none())
        .or_else(|| find_unit(r, kinds, plain))
}

/// A limited controller (an infantry type first).
fn controller(r: &SimRules) -> Option<(Kind, String)> {
    find_unit(r, &[Kind::Infantry, Kind::Vehicle], |u| {
        is_controller(r, u, false) && u.strength > 0
    })
}

/// A gattling vehicle that can be taken over; `bunkered`: one a tank bunker
/// takes (S-0606 claim 1: bunkerable, turret, armed, not hover).
fn gattling_vehicle(r: &SimRules, bunkered: bool) -> Option<String> {
    find_unit(r, &[Kind::Vehicle], |u| {
        u.combat.gattling
            && u.combat.slots[0].is_some()
            && ordinary(u)
            && (!bunkered || (u.raw.bunkerable.unwrap_or(true) && u.turret))
    })
    .map(|(_, n)| n)
}

/// A vehicle a tank bunker takes (S-0606 claim 1), not psionics-immune.
fn bunker_vehicle(r: &SimRules) -> Option<String> {
    find_unit(r, &[Kind::Vehicle], |u| {
        ordinary(u) && u.raw.bunkerable.unwrap_or(true) && u.turret && u.combat.slots[0].is_some()
    })
    .map(|(_, n)| n)
}

fn absorber(r: &SimRules) -> Option<String> {
    find_building(r, |b| b.absorb.infantry && b.absorb.capacity > 0)
}

fn grinder(r: &SimRules) -> Option<String> {
    find_building(r, |b| b.raw.grinding.unwrap_or(false))
}

fn tank_bunker(r: &SimRules) -> Option<String> {
    find_building(r, |b| b.raw.bunker.unwrap_or(false))
}

/// An infantry type an absorber of `name` accepts by size.
fn absorbable(r: &SimRules, name: &str) -> Option<String> {
    let b = r.buildings.iter().find(|b| b.name == name)?;
    let limit = b.absorb.size_limit;
    find_unit(r, &[Kind::Infantry], |u| {
        ordinary(u)
            && !u.fraidycat
            && !u.engineer
            && !limit.lt(u.size)
            && warhead0(r, u).is_none_or(|h| !h.mind_control)
    })
    .map(|(_, n)| n)
}

/// A lifter vehicle and its weapon damage.
fn lifter(r: &SimRules) -> Option<(String, i32)> {
    units(r, &[Kind::Vehicle]).into_iter().find_map(|(k, i)| {
        let u = r.unit(k, i)?;
        let wp = u.combat.slots[0]?;
        let weapon = r.combat.weapon(wp);
        let h = weapon.warhead?;
        r.combat
            .warhead(h)
            .is_locomotor
            .then(|| (u.name.clone(), weapon.damage))
    })
}

/// A vehicle a lifter of damage `damage` may seize (S-0607 claim 3 and
/// F1), with `extra`; an unarmed type first.
fn liftable(r: &SimRules, damage: i32, extra: impl Fn(&UnitRules) -> bool) -> Option<String> {
    let d = glasnost_num::F64::from_i64(i64::from(damage), r.mode);
    let ok = |u: &UnitRules| {
        ordinary(u)
            && u.size.lt(d)
            && warhead0(r, u).is_none_or(|h| !h.is_locomotor && !h.parasite)
            && extra(u)
    };
    find_unit(r, &[Kind::Vehicle], |u| {
        ok(u) && u.combat.slots[0].is_none()
    })
    .or_else(|| find_unit(r, &[Kind::Vehicle], ok))
    .map(|(_, n)| n)
}

/// The building with the largest power output.
fn power_plant(r: &SimRules) -> Option<String> {
    r.buildings
        .iter()
        .filter(|b| b.raw.econ.power.unwrap_or(0) > 0 && b.strength > 0)
        .max_by_key(|b| b.raw.econ.power.unwrap_or(0))
        .map(|b| b.name.clone())
}

/// Gives house `h` power plants in a far corner, so that low power plays
/// no part.
fn powered(w: &mut World, h: u8) {
    let Some(p) = power_plant(w.rules()) else {
        return;
    };
    for i in 0..4 {
        put(
            w,
            Kind::Building,
            &p,
            h,
            -20 + 5 * i,
            -20 + 6 * i32::from(h),
        );
    }
}

/// An ordinary building without weapons, of some strength.
fn plain_building(r: &SimRules) -> Option<String> {
    power_plant(r).or_else(|| {
        find_building(r, |b| {
            b.strength > 0 && b.combat.slots[0].is_none() && !b.absorb.absorbing()
        })
    })
}

/// A cloning building, and an infantry factory that is not one.
fn cloning(r: &SimRules) -> Option<(String, String)> {
    let vat = find_building(r, |b| b.raw.econ.cloning.unwrap_or(false))?;
    let k = find_building(r, |b| {
        b.factory == Some(FactoryKind::Infantry) && !b.raw.econ.cloning.unwrap_or(false)
    })?;
    Some((vat, k))
}

fn sw_index(r: &SimRules, k: SwKind) -> Option<u32> {
    r.superweapons.first_of_kind(k)
}

fn shield(w: &mut World, h: u8, c: CellXY) {
    let i = sw_index(w.rules(), SwKind::ForceShield).expect("a force shield");
    superweapons::protect::force_shield(w, HouseId(h), i, c);
}

/// House `h`'s domination at `c`, run to its sweep; the objects taken.
fn dominate(w: &mut World, h: u8, c: CellXY) -> Option<Vec<ObjectId>> {
    let i = sw_index(w.rules(), SwKind::Dominator)?;
    domination::launch(w, HouseId(h), i, c);
    for _ in 0..600 {
        w.effects.clear();
        tick::step(w);
        let ev: Vec<domination::DomEvent> = w
            .effects
            .iter()
            .filter_map(|e| match e {
                Effect::Domination { event, .. } => Some(*event),
                _ => None,
            })
            .collect();
        if ev.contains(&domination::DomEvent::Sweep) {
            return Some(
                ev.into_iter()
                    .filter_map(|e| match e {
                        domination::DomEvent::Taken(id) => Some(id),
                        _ => None,
                    })
                    .collect(),
            );
        }
    }
    None
}

/// The reference rules with the domination's sweep damage set to 0, for
/// a pair whose building the sweep's damage destroys on a run before the
/// takeover can be seen (S-0612 claim 10: the damage comes first).
fn without_sweep_damage(r: &Arc<SimRules>) -> Arc<SimRules> {
    let mut x = (**r).clone();
    x.superweapons.domination.damage = 0;
    Arc::new(x)
}

/// House `h`'s genetic mutation at `c`; then runs until every conversion
/// animation has ended; returns the units made.
fn mutate(w: &mut World, h: u8, c: CellXY) -> Vec<ObjectId> {
    let i = sw_index(w.rules(), SwKind::Genetic).unwrap_or(0);
    w.effects.clear();
    mutation::launch(w, HouseId(h), i, c);
    let mut out = Vec::new();
    // The deaths come in the following ticks; then every conversion
    // animation runs to its end.
    for t in 0..400 {
        if t >= 30 && w.mutation.makers.is_empty() {
            break;
        }
        w.effects.clear();
        tick::step(w);
        for e in &w.effects {
            if let Effect::Maker {
                event:
                    MakerEvent::Ended {
                        unit: Some(u),
                        conversion: true,
                    },
                ..
            } = e
            {
                out.push(*u);
            }
        }
    }
    out
}

/// The genetic mutation's own data (the conversion animation and its made
/// types); `None` when the rules have none.
fn mutation_ready(r: &SimRules) -> Option<()> {
    let m = &r.superweapons.mutation;
    (m.anim.is_some() && m.made.iter().any(Option::is_some)).then_some(())
}

/// Sends vehicle `v` into bunker `b` with its owner's enter click.
fn bunker_in(w: &mut World, v: ObjectId, b: ObjectId) -> bool {
    let house = owner(w, v).unwrap();
    click(w, house, &[v], b);
    run_until(w, 1500, |w| bunker::bunkered_in(w, v) == Some(b))
}

/// Attack-orders lifter `l` at `v` and runs until `v` is seized.
fn lift_up(w: &mut World, l: ObjectId, v: ObjectId) -> bool {
    let house = owner(w, l).unwrap();
    attack(w, house, l, v);
    run_until(w, 900, |w| lift::seized(w, v))
}

/// The infantry of type `ti` on the map.
fn count_type(w: &World, kind: Kind, ti: u32) -> usize {
    w.objects
        .values()
        .filter(|o| o.kind == kind && o.type_index == ti && o.on_map)
        .count()
}

// ---------------------------------------------------------------------------
// M7a pairs (docs/m7-claims/pairs-m7a.md)

// spec: S-0507 claims 4, 6 and 14, S-0213 table T3 (f), S-0601 claims 5 and 10, S-0312 claim 15
#[test]
#[ignore = "local relation lane: needs GLASNOST_GAME_DIR"]
fn m7a_a_controller_riding_an_open_transport_takes_and_loses_a_victim() {
    let name = "m7a_a_controller_riding_an_open_transport_takes_and_loses_a_victim";
    let r = need!(name, reference(), "no install");
    let (ck, cn) = need!(name, controller(&r), "no limited controller");
    let size = r.unit(ck, r.index_of(ck, &cn).unwrap()).unwrap().size;
    let cart = need!(
        name,
        find_unit(&r, &[Kind::Vehicle], |u| u.open_topped
            && u.passengers > 0
            && !u.size_limit.lt(size)
            && u.strength > 0),
        "no open-topped transport that takes the controller"
    );
    let (vk, vn) = need!(
        name,
        victim(&r, &[Kind::Infantry, Kind::Vehicle]),
        "no plain victim"
    );
    let mut w = world(&r);
    let ctl = put(&mut w, ck, &cn, B, 0, 0);
    let cart = put(&mut w, Kind::Vehicle, &cart.1, B, 2, 0);
    let v = put(&mut w, vk, &vn, A, 7, 0);
    click(&mut w, B, &[ctl], cart);
    assert!(
        run_until(&mut w, 900, |w| transport::riding(w, ctl) == Some(cart)),
        "the controller boards"
    );
    attack(&mut w, B, cart, v);
    assert!(
        run_healed(&mut w, 1500, &[cart, v], |w| mind::mind_controlled(w, v)),
        "taken by passenger fire"
    );
    assert_eq!(owner(&w, v), Some(B));
    kill(&mut w, cart);
    testkit::run(&mut w, 2);
    assert!(
        w.obj(ctl).is_none(),
        "the passenger leaves with its transport"
    );
    assert_eq!(owner(&w, v), Some(A), "the victim goes home");
    pass(name);
}

// spec: S-0609 claims 1 and 4, S-0601 claims 4, 5 and 6
#[test]
#[ignore = "local relation lane: needs GLASNOST_GAME_DIR"]
fn m7a_a_gattling_taken_over_while_firing_keeps_its_stage() {
    let name = "m7a_a_gattling_taken_over_while_firing_keeps_its_stage";
    let r = need!(name, reference(), "no install");
    let g = need!(
        name,
        gattling_vehicle(&r, false),
        "no gattling vehicle that can be taken"
    );
    let (ck, cn) = need!(name, controller(&r), "no limited controller");
    let (tk, tn) = need!(name, victim(&r, &[Kind::Vehicle]), "no plain vehicle");
    let mut w = world(&r);
    let gat = put(&mut w, Kind::Vehicle, &g, A, 0, 0);
    let tgt = put(&mut w, tk, &tn, B, 4, 0);
    attack(&mut w, A, gat, tgt);
    assert!(
        run_healed(&mut w, 900, &[gat, tgt], |w| gattling::spin(w, gat).stage
            >= 1),
        "the gattling spins up"
    );
    let ctl = put(&mut w, ck, &cn, B, 0, 4);
    attack(&mut w, B, ctl, gat);
    let mut before = 0;
    let mut taken = false;
    for _ in 0..1500 {
        before = gattling::spin(&w, gat).stage;
        heal(&mut w, &[gat, tgt, ctl]);
        tick::step(&mut w);
        if mind::mind_controlled(&w, gat) {
            taken = true;
            break;
        }
    }
    assert!(taken, "the controller takes the gattling");
    assert!(before >= 1);
    assert!(
        gattling::spin(&w, gat).stage >= before,
        "no drop of stage at the change of owner"
    );
    pass(name);
}

// spec: S-0213 claim 15, S-0203 claim 7, S-0601 claim 6
#[test]
#[ignore = "local relation lane: needs GLASNOST_GAME_DIR"]
fn m7a_an_order_of_the_former_owner_still_applies_in_the_takeover_tick() {
    let name = "m7a_an_order_of_the_former_owner_still_applies_in_the_takeover_tick";
    let r = need!(name, reference(), "no install");
    let armed = need!(
        name,
        find_unit(&r, &[Kind::Vehicle], |u| ordinary(u)
            && u.combat.slots[0].is_some()),
        "no armed vehicle that can be taken"
    );
    let (ck, cn) = need!(name, controller(&r), "no limited controller");
    let (tk, tn) = need!(name, victim(&r, &[Kind::Vehicle]), "no plain vehicle");
    let mut w = world(&r);
    let v = put(&mut w, Kind::Vehicle, &armed.1, A, 0, 0);
    let tgt = put(&mut w, tk, &tn, B, 4, 0);
    let ctl = put(&mut w, ck, &cn, B, 0, 4);
    attack(&mut w, A, v, tgt);
    assert!(mind::take(&mut w, ctl, v));
    tick::step(&mut w);
    assert_eq!(owner(&w, v), Some(B));
    assert_eq!(
        w.obj(v).unwrap().target,
        Some(Target::Object(tgt)),
        "the former owner's order reached the new owner's object"
    );
    pass(name);
}

// spec: S-0605 claims 1, 6, 7, 10 and 11 to 13, S-0601 claims 5 and 9, S-0214 table I
#[test]
#[ignore = "local relation lane: needs GLASNOST_GAME_DIR"]
fn m7a_a_controlled_enemy_goes_into_an_absorber_and_comes_out_an_enemy() {
    let name = "m7a_a_controlled_enemy_goes_into_an_absorber_and_comes_out_an_enemy";
    let r = need!(name, reference(), "no install");
    let bio = need!(name, absorber(&r), "no infantry absorber");
    let inf = need!(name, absorbable(&r, &bio), "no infantry the absorber takes");
    let (ck, cn) = need!(name, controller(&r), "no limited controller");
    let mut w = world(&r);
    powered(&mut w, B);
    let ctl = put(&mut w, ck, &cn, B, 0, 0);
    let bio = put(&mut w, Kind::Building, &bio, B, -5, -5);
    let g = put(&mut w, Kind::Infantry, &inf, A, 4, 0);
    attack(&mut w, B, ctl, g);
    assert!(
        run_healed(&mut w, 1500, &[ctl, g], |w| mind::mind_controlled(w, g)),
        "taken by fire"
    );
    click(&mut w, B, &[g], bio);
    assert!(
        run_until(&mut w, 1500, |w| absorb::absorbed_in(w, g) == Some(bio)),
        "goes in"
    );
    assert_eq!(owner(&w, g), Some(A), "inside as its original owner's unit");
    assert!(!mind::mind_controlled(&w, g));
    click(&mut w, B, &[bio], bio);
    assert!(
        run_until(&mut w, 600, |w| absorb::count(w, bio) == 0),
        "let out"
    );
    assert!(w.obj(g).is_some_and(|o| o.on_map));
    assert_eq!(owner(&w, g), Some(A), "comes out an enemy");
    pass(name);
}

/// A neutral unit taken by B's controller, the controller killed: runs
/// over a few seeds until the neutral house's fate decision (a weighted
/// draw from the rules' capture tables) sends the released unit to
/// `building`; `arrived` tells when it got there.
fn neutral_fate(
    r: &Arc<SimRules>,
    building: &str,
    arrived: impl Fn(&World, ObjectId, ObjectId) -> bool,
) -> Option<bool> {
    let (ck, cn) = controller(r)?;
    let inf = absorbable(r, building).or_else(|| victim(r, &[Kind::Infantry]).map(|v| v.1))?;
    for seed in 1..=12 {
        let mut w = build(r, seed, true);
        powered(&mut w, N);
        let ctl = put(&mut w, ck, &cn, B, -6, 0);
        let b = put(&mut w, Kind::Building, building, N, 6, -5);
        let civ = put(&mut w, Kind::Infantry, &inf, N, 1, 0);
        attack(&mut w, B, ctl, civ);
        if !run_healed(&mut w, 1500, &[ctl, civ], |w| mind::mind_controlled(w, civ)) {
            return Some(false);
        }
        kill(&mut w, ctl);
        testkit::run(&mut w, 2);
        assert_eq!(owner(&w, civ), Some(N), "released to its owner");
        if run_until(&mut w, 1500, |w| arrived(w, civ, b)) {
            return Some(true);
        }
    }
    None
}

// spec: S-0601 claims 8 and 10, S-0605 claims 1, 3, 6, 7 and 16, S-0215 claim 24
#[test]
#[ignore = "local relation lane: needs GLASNOST_GAME_DIR"]
fn m7a_a_released_neutral_unit_takes_the_bio_reactor_fate() {
    let name = "m7a_a_released_neutral_unit_takes_the_bio_reactor_fate";
    let r = need!(name, reference(), "no install");
    let bio = need!(name, absorber(&r), "no infantry absorber");
    let ok = need!(
        name,
        neutral_fate(&r, &bio, |w, u, b| absorb::absorbed_in(w, u) == Some(b)),
        "no seed of twelve drew the bio-reactor fate"
    );
    assert!(ok, "the controller takes the neutral unit");
    pass(name);
}

// spec: S-0603 claims 1, 5, 6, 8, 9 and 11, S-0601 claims 5 and 12, S-0214 table I row I26
#[test]
#[ignore = "local relation lane: needs GLASNOST_GAME_DIR"]
fn m7a_a_controlled_enemy_is_ground_for_its_controllers_house() {
    let name = "m7a_a_controlled_enemy_is_ground_for_its_controllers_house";
    let r = need!(name, reference(), "no install");
    let grnd = need!(name, grinder(&r), "no recycling building");
    let (vk, vn) = need!(name, victim(&r, &[Kind::Infantry]), "no plain infantry");
    let (ck, cn) = need!(name, controller(&r), "no limited controller");
    let mut w = world(&r);
    powered(&mut w, B);
    let ctl = put(&mut w, ck, &cn, B, 0, 0);
    let grnd = put(&mut w, Kind::Building, &grnd, B, -6, -6);
    let g = put(&mut w, vk, &vn, A, 4, 0);
    attack(&mut w, B, ctl, g);
    assert!(
        run_healed(&mut w, 1500, &[ctl, g], |w| mind::mind_controlled(w, g)),
        "taken by fire"
    );
    let (ca, cb) = (w.houses[0].credits, w.houses[1].credits);
    click(&mut w, B, &[g], grnd);
    assert!(run_until(&mut w, 1500, |w| w.obj(g).is_none()), "consumed");
    assert!(w.houses[1].credits > cb, "the controller's house is paid");
    assert_eq!(w.houses[0].credits, ca, "the original owner gets nothing");
    assert!(
        mind::controller(&w, ctl).is_none_or(|c| c.victims.is_empty()),
        "the slot is free"
    );
    pass(name);
}

// spec: S-0601 claims 8 and 10, S-0603 claims 3, 5, 8 and 14
#[test]
#[ignore = "local relation lane: needs GLASNOST_GAME_DIR"]
fn m7a_a_released_neutral_unit_takes_the_grinder_fate() {
    let name = "m7a_a_released_neutral_unit_takes_the_grinder_fate";
    let r = need!(name, reference(), "no install");
    let grnd = need!(name, grinder(&r), "no recycling building");
    let ok = need!(
        name,
        neutral_fate(&r, &grnd, |w, u, b| w
            .obj(u)
            .is_none_or(|o| o.dest == Some(Target::Object(b)))),
        "no seed of twelve drew the grinder fate"
    );
    assert!(ok, "the controller takes the neutral unit");
    pass(name);
}

/// The spin changes of gattling `g` from its first spin-up to its top
/// stage, healing `keep`.
fn climb(w: &mut World, g: ObjectId, keep: &[ObjectId]) -> Option<Vec<i32>> {
    let top = w
        .rules()
        .unit(Kind::Vehicle, w.obj(g)?.type_index)?
        .combat
        .spin
        .stages
        - 1;
    let mut steps = Vec::new();
    let mut last = gattling::spin(w, g).value;
    for _ in 0..1500 {
        heal(w, keep);
        tick::step(w);
        let s = gattling::spin(w, g);
        if s.value != last || !steps.is_empty() {
            steps.push(s.value - last);
        }
        last = s.value;
        if s.stage >= top {
            return Some(steps);
        }
    }
    None
}

// spec: S-0609 claim 5, S-0606 claims 1, 7 and 9
#[test]
#[ignore = "local relation lane: needs GLASNOST_GAME_DIR"]
fn m7a_a_gattling_in_a_bunker_spins_up_as_in_the_open() {
    let name = "m7a_a_gattling_in_a_bunker_spins_up_as_in_the_open";
    let r = need!(name, reference(), "no install");
    let bunk = need!(name, tank_bunker(&r), "no tank bunker");
    let g = need!(
        name,
        gattling_vehicle(&r, true),
        "no gattling vehicle a bunker takes"
    );
    let (tk, tn) = need!(name, victim(&r, &[Kind::Vehicle]), "no plain vehicle");
    let mut w = world(&r);
    powered(&mut w, A);
    let b = put(&mut w, Kind::Building, &bunk, A, 0, 0);
    let gat = put(&mut w, Kind::Vehicle, &g, A, 5, 1);
    assert!(bunker_in(&mut w, gat, b), "bunkered");
    let tgt = put(&mut w, tk, &tn, B, -3, 1);
    attack(&mut w, A, gat, tgt);
    let inside = climb(&mut w, gat, &[gat, tgt, b]).expect("the top stage in the bunker");
    let mut w = world(&r);
    let gat = put(&mut w, Kind::Vehicle, &g, A, 1, 1);
    let tgt = put(&mut w, tk, &tn, B, -3, 1);
    attack(&mut w, A, gat, tgt);
    let open = climb(&mut w, gat, &[gat, tgt]).expect("the top stage in the open");
    let up = |s: &[i32]| s.iter().copied().filter(|&d| d > 0).collect::<Vec<_>>();
    let (ui, uo) = (up(&inside), up(&open));
    assert!(!ui.is_empty() && !uo.is_empty());
    assert!(
        ui.iter().all(|&d| d == uo[0]) && uo.iter().all(|&d| d == uo[0]),
        "the same spin-up step in the bunker as in the open"
    );
    pass(name);
}

// spec: S-0601 claims 3 and 6, S-0606 claims 2, 3 and 7
#[test]
#[ignore = "local relation lane: needs GLASNOST_GAME_DIR"]
fn m7a_a_controlled_vehicle_goes_into_its_controllers_bunker() {
    let name = "m7a_a_controlled_vehicle_goes_into_its_controllers_bunker";
    let r = need!(name, reference(), "no install");
    let bunk = need!(name, tank_bunker(&r), "no tank bunker");
    let vn = need!(name, bunker_vehicle(&r), "no vehicle a bunker takes");
    let (ck, cn) = need!(name, controller(&r), "no limited controller");
    let mut w = world(&r);
    powered(&mut w, A);
    let b = put(&mut w, Kind::Building, &bunk, A, 0, 0);
    let v = put(&mut w, Kind::Vehicle, &vn, B, 6, 1);
    let ctl = put(&mut w, ck, &cn, A, 6, 4);
    assert!(mind::take(&mut w, ctl, v));
    tick::step(&mut w);
    assert_eq!(owner(&w, v), Some(A));
    assert!(bunker_in(&mut w, v, b), "the controlled vehicle goes in");
    pass(name);
}

// spec: S-0513 claim 34, S-0606 claim 14
#[test]
#[ignore = "local relation lane: needs GLASNOST_GAME_DIR"]
fn m7a_a_warp_takes_a_bunkered_vehicle_out_of_its_bunker() {
    let name = "m7a_a_warp_takes_a_bunkered_vehicle_out_of_its_bunker";
    let r = need!(name, reference(), "no install");
    let bunk = need!(name, tank_bunker(&r), "no tank bunker");
    let vn = need!(name, bunker_vehicle(&r), "no vehicle a bunker takes");
    let mut w = world(&r);
    powered(&mut w, A);
    let b = put(&mut w, Kind::Building, &bunk, A, 0, 0);
    let v = put(&mut w, Kind::Vehicle, &vn, A, 6, 1);
    assert!(bunker_in(&mut w, v, b), "bunkered");
    let from = w.obj(v).unwrap().pos.cell();
    superweapons::warp::launch(&mut w, HouseId(B), from, at(0, 10));
    assert!(!bunker::bunkered(&w, v), "the link ends in place");
    assert_eq!(bunker::occupant(&w, b), None);
    pass(name);
}

// spec: S-0610 claim 3, S-0606 Terms
#[test]
#[ignore = "local relation lane: needs GLASNOST_GAME_DIR"]
fn m7a_a_berserk_weapon_is_refused_at_a_bunkered_vehicle_and_its_bunker() {
    let name = "m7a_a_berserk_weapon_is_refused_at_a_bunkered_vehicle_and_its_bunker";
    let r = need!(name, reference(), "no install");
    let bunk = need!(name, tank_bunker(&r), "no tank bunker");
    let vn = need!(name, bunker_vehicle(&r), "no vehicle a bunker takes");
    let (fk, fname) = need!(
        name,
        find_unit(&r, &[Kind::Infantry, Kind::Vehicle], |u| u.strength > 0
            && warhead0(&r, u).is_some_and(|h| h.psychedelic)),
        "no unit with a berserk first weapon"
    );
    let mut w = world(&r);
    powered(&mut w, A);
    let b = put(&mut w, Kind::Building, &bunk, A, 0, 0);
    let v = put(&mut w, Kind::Vehicle, &vn, A, 6, 1);
    let f = put(&mut w, fk, &fname, B, -8, 8);
    let refused = |w: &World, x| psychic::firing_refused(w, f, true, Target::Object(x));
    assert!(!refused(&w, v), "outside: not refused");
    assert!(bunker_in(&mut w, v, b), "bunkered");
    assert!(
        refused(&w, v) && refused(&w, b),
        "refused at the vehicle and its bunker"
    );
    pass(name);
}

// spec: S-0607 claims 3, 5, 12, 13 and 18, S-0601 claim 12
#[test]
#[ignore = "local relation lane: needs GLASNOST_GAME_DIR"]
fn m7a_a_lifted_controlled_vehicle_dies_and_frees_its_controllers_slot() {
    let name = "m7a_a_lifted_controlled_vehicle_dies_and_frees_its_controllers_slot";
    let r = need!(name, reference(), "no install");
    let (ln, dmg) = need!(name, lifter(&r), "no lifter");
    let vn = need!(
        name,
        liftable(&r, dmg, |_| true),
        "no vehicle the lifter seizes"
    );
    let (ck, cn) = need!(name, controller(&r), "no limited controller");
    let mut w = world(&r);
    let v = put(&mut w, Kind::Vehicle, &vn, B, 6, 0);
    let ctl = put(&mut w, ck, &cn, A, 6, 4);
    let l = put(&mut w, Kind::Vehicle, &ln, B, -2, 0);
    assert!(mind::take(&mut w, ctl, v));
    tick::step(&mut w);
    assert!(lift_up(&mut w, l, v), "seized");
    assert!(
        mind::mind_controlled(&w, v),
        "still controlled while gripped"
    );
    // The pull may arrive at the lifter before it crosses a fixed cell
    // (claim 8), so the blocker (of the controller's own house) is put
    // under the unit once it is high over a cell other than the lifter's.
    let mut blocker = None;
    for _ in 0..900 {
        heal(&mut w, &[l]);
        tick::step(&mut w);
        let Some(o) = w.obj(v) else { break };
        let c = o.pos.cell();
        if !lift::grips(&w, l, v) {
            break;
        }
        if w.height_above_surface(v) > 300 && c != w.obj(l).unwrap().pos.cell() {
            blocker = Some(testkit::spawn(&mut w, Kind::Vehicle, &vn, A, c, 0));
            break;
        }
    }
    need!(
        name,
        blocker,
        "the pull never lifts the unit high over a free cell"
    );
    let away = w.obj(l).unwrap().pos.cell();
    glasnost_sim::econ::assign_destination(
        &mut w,
        l,
        Some(Target::Cell(CellXY::new(away.x, away.y + 2))),
    );
    assert!(
        run_until(&mut w, 900, |w| w.obj(v).is_none()),
        "dropped onto the blocker and gone"
    );
    assert!(
        mind::controller(&w, ctl).is_none_or(|c| c.victims.is_empty()),
        "the slot is free"
    );
    pass(name);
}

// spec: S-0606 claim 9, S-0309 claim 24 row 1, S-0607 claims 1 and 3
#[test]
#[ignore = "local relation lane: needs GLASNOST_GAME_DIR"]
fn m7a_a_lift_attack_on_a_bunkered_vehicle_goes_to_its_bunker() {
    let name = "m7a_a_lift_attack_on_a_bunkered_vehicle_goes_to_its_bunker";
    let r = need!(name, reference(), "no install");
    let bunk = need!(name, tank_bunker(&r), "no tank bunker");
    let (ln, dmg) = need!(name, lifter(&r), "no lifter");
    let vn = need!(
        name,
        liftable(&r, dmg, |u| u.raw.bunkerable.unwrap_or(true) && u.turret),
        "no vehicle both a bunker takes and the lifter could seize"
    );
    let mut w = world(&r);
    powered(&mut w, A);
    let b = put(&mut w, Kind::Building, &bunk, A, 0, 0);
    let v = put(&mut w, Kind::Vehicle, &vn, A, 6, 1);
    assert!(bunker_in(&mut w, v, b), "bunkered");
    let l = put(&mut w, Kind::Vehicle, &ln, B, -5, 5);
    let hv = health(&w, v);
    attack(&mut w, B, l, v);
    // The lifter's weapon makes no projectile object: its shots are read
    // from the shot effects. The bunkered vehicle fires back, so the lifter
    // is kept whole.
    let mut fired = false;
    for _ in 0..300 {
        heal(&mut w, &[l]);
        w.effects.clear();
        tick::step(&mut w);
        fired |= w
            .effects
            .iter()
            .any(|e| matches!(e, Effect::Shot { firer, .. } if *firer == l));
    }
    assert!(fired, "the lifter fired");
    assert!(
        !lift::seized(&w, v) && !lift::seized(&w, b),
        "nothing seized"
    );
    assert_eq!(bunker::bunkered_in(&w, v), Some(b), "still bunkered");
    assert_eq!(health(&w, v), hv, "the vehicle is unhurt");
    pass(name);
}

// spec: S-0607 claims 5 and 10, S-0609 claim 5
#[test]
#[ignore = "local relation lane: needs GLASNOST_GAME_DIR"]
fn m7a_a_seized_gattling_spins_down() {
    let name = "m7a_a_seized_gattling_spins_down";
    let r = need!(name, reference(), "no install");
    let (ln, dmg) = need!(name, lifter(&r), "no lifter");
    let g = need!(
        name,
        liftable(&r, dmg, |u| u.combat.gattling
            && u.combat.slots[0].is_some()),
        "no gattling vehicle the lifter seizes"
    );
    let (tk, tn) = need!(name, victim(&r, &[Kind::Vehicle]), "no plain vehicle");
    let mut w = world(&r);
    let gat = put(&mut w, Kind::Vehicle, &g, A, 0, 0);
    let tgt = put(&mut w, tk, &tn, B, 4, 0);
    attack(&mut w, A, gat, tgt);
    assert!(climb(&mut w, gat, &[gat, tgt]).is_some(), "spun up");
    let l = put(&mut w, Kind::Vehicle, &ln, B, 0, 5);
    assert!(lift_up(&mut w, l, gat), "seized");
    let mut last = gattling::spin(&w, gat).value;
    let mut steps = Vec::new();
    for _ in 0..20 {
        tick::step(&mut w);
        if !lift::seized(&w, gat) {
            break;
        }
        let s = gattling::spin(&w, gat).value;
        steps.push(s - last);
        last = s;
    }
    // It steps down each tick until it reaches the bottom, then stays.
    assert!(
        steps.first().is_some_and(|&d| d < 0) && steps.iter().all(|&d| d <= 0),
        "it spins down"
    );
    pass(name);
}

// spec: S-0607 claim 9, S-0603 claim 1, S-0214 table I row I26
#[test]
#[ignore = "local relation lane: needs GLASNOST_GAME_DIR"]
fn m7a_a_seized_vehicle_ignores_its_owners_recycle_click() {
    let name = "m7a_a_seized_vehicle_ignores_its_owners_recycle_click";
    let r = need!(name, reference(), "no install");
    let grnd = need!(name, grinder(&r), "no recycling building");
    let (ln, dmg) = need!(name, lifter(&r), "no lifter");
    let vn = need!(
        name,
        liftable(&r, dmg, |_| true),
        "no vehicle the lifter seizes"
    );
    let mut w = world(&r);
    powered(&mut w, A);
    let grnd = put(&mut w, Kind::Building, &grnd, A, -6, -6);
    let v = put(&mut w, Kind::Vehicle, &vn, A, 6, 0);
    let l = put(&mut w, Kind::Vehicle, &ln, B, 6, 7);
    assert!(lift_up(&mut w, l, v), "seized");
    let bound = w.obj(v).unwrap().dest;
    let credits = w.houses[0].credits;
    click(&mut w, A, &[v], grnd);
    testkit::run(&mut w, 2);
    assert_eq!(w.obj(v).unwrap().dest, bound, "the click changes nothing");
    testkit::run(&mut w, 60);
    assert_eq!(w.houses[0].credits, credits, "not ground");
    pass(name);
}

// spec: S-0614 claim 12, S-0607 claim 5
#[test]
#[ignore = "local relation lane: needs GLASNOST_GAME_DIR"]
fn m7a_a_drone_stays_on_a_seized_host() {
    let name = "m7a_a_drone_stays_on_a_seized_host";
    let r = need!(name, reference(), "no install");
    let (ln, dmg) = need!(name, lifter(&r), "no lifter");
    let drone = need!(
        name,
        find_unit(&r, &[Kind::Vehicle], |u| u.strength > 0
            && !u.combat.naval
            && warhead0(&r, u).is_some_and(|h| h.parasite)),
        "no ground parasite"
    );
    let host = need!(
        name,
        liftable(&r, dmg, |_| true),
        "no vehicle the lifter seizes"
    );
    let mut w = world(&r);
    let h = put(&mut w, Kind::Vehicle, &host, B, 2, 0);
    let d = put(&mut w, Kind::Vehicle, &drone.1, A, 0, 0);
    let l = put(&mut w, Kind::Vehicle, &ln, A, 2, 6);
    attack(&mut w, A, d, h);
    assert!(
        run_healed(&mut w, 600, &[h], |w| parasite::host_of(w, d) == Some(h)),
        "latched"
    );
    assert!(lift_up(&mut w, l, h), "seized");
    assert_eq!(parasite::host_of(&w, d), Some(h), "the drone stays on");
    pass(name);
}

// ---------------------------------------------------------------------------
// M7b pairs (docs/m7-claims/pairs-m7b.md)

/// A seizing beam's projectile has landed: the lifter `l` fires once at `v`.
fn seize_now(w: &mut World, l: ObjectId, v: ObjectId) -> bool {
    lift_up(w, l, v)
}

// spec: S-0607 claim 16 (a fall ending on infantry removes it without death effects), S-0615 claims 1, 2 and 4
#[test]
#[ignore = "local relation lane: needs GLASNOST_GAME_DIR"]
fn m7b_a_lift_fall_crushes_silently_but_its_impact_makes_a_burning_walker() {
    let name = "m7b_a_lift_fall_crushes_silently_but_its_impact_makes_a_burning_walker";
    let r = need!(name, reference(), "no install");
    let (ln, dmg) = need!(name, lifter(&r), "no lifter");
    let vn = need!(
        name,
        liftable(&r, dmg, |_| true),
        "no vehicle the lifter seizes"
    );
    let (ik, iname) = need!(name, victim(&r, &[Kind::Infantry]), "no plain infantry");
    let mut w = world(&r);
    let l = put(&mut w, Kind::Vehicle, &ln, A, -2, 0);
    let v = put(&mut w, Kind::Vehicle, &vn, B, 6, 0);
    assert!(seize_now(&mut w, l, v), "seized");
    // The infantry waits in a cell the pull crosses, once the unit is
    // high over it.
    let mut under = None;
    for _ in 0..900 {
        heal(&mut w, &[l]);
        tick::step(&mut w);
        let Some(o) = w.obj(v) else { break };
        let c = o.pos.cell();
        if w.height_above_surface(v) > 300 && c != w.obj(l).unwrap().pos.cell() {
            under = Some(testkit::spawn(&mut w, ik, &iname, A, c, 0));
            break;
        }
    }
    let under = need!(
        name,
        under,
        "the pull never lifts the unit high over a free cell"
    );
    let away = w.obj(l).unwrap().pos.cell();
    glasnost_sim::econ::assign_destination(
        &mut w,
        l,
        Some(Target::Cell(CellXY::new(away.x - 3, away.y + 3))),
    );
    let mut deaths = 0;
    let gone = {
        let mut done = false;
        for _ in 0..900 {
            w.effects.clear();
            tick::step(&mut w);
            deaths += w
                .effects
                .iter()
                .filter(|e| matches!(e, Effect::Walker { made: true, .. }))
                .count();
            if w.obj(under).is_none() || w.obj(v).is_none() {
                done = true;
                break;
            }
        }
        done
    };
    assert!(gone, "the fall ends");
    if w.obj(under).is_some() {
        println!("relation {name}: SKIP the fall did not end on the infantry's cell");
        return;
    }
    assert_eq!(deaths, 0, "crushed with no burning walker");
    pass(name);
}

/// A gas particle type of the shared gas system with damage.
fn gas_particle(r: &SimRules) -> Option<u32> {
    r.gas.shared?;
    r.gas
        .particles
        .iter()
        .position(|p| p.damage > 0 && p.warhead.is_some())
        .map(|i| i as u32)
}

fn cloud_at(w: &mut World, ti: u32, p: Pos) {
    let draw = w.rules().gas.particles[ti as usize].draw;
    gas::middle_event_particle(w, draw, Some((p.x, p.y, p.z)));
}

// spec: S-0613 claims 2, 13 and 14, S-0107 claim 17, S-0615 claims 1 and 2
#[test]
#[ignore = "local relation lane: needs GLASNOST_GAME_DIR"]
fn m7b_a_gas_cloud_kills_a_parachutist_with_one_cloud_and_an_explosion() {
    let name = "m7b_a_gas_cloud_kills_a_parachutist_with_one_cloud_and_an_explosion";
    let r = need!(name, reference(), "no install");
    let ti = need!(name, gas_particle(&r), "no damaging gas particle");
    let boom = need!(
        name,
        r.combat.general.infantry_explode.clone(),
        "no explosion death"
    );
    let (ik, iname) = need!(name, victim(&r, &[Kind::Infantry]), "no plain infantry");
    let mut w = world(&r);
    let chute = put(&mut w, ik, &iname, B, 0, 0);
    w.obj_mut(chute).unwrap().health = 1;
    let p = w.obj(chute).unwrap().pos;
    cloud_at(&mut w, ti, p);
    let mut seen = false;
    for _ in 0..600 {
        if let Some(o) = w.obj_mut(chute)
            && let Some(i) = o.infantry.as_mut()
        {
            i.seq = Some(glasnost_sim::infantry::Seq::Parachute);
        }
        w.effects.clear();
        tick::step(&mut w);
        seen |= w
            .effects
            .iter()
            .any(|e| matches!(e, Effect::Anim { name, .. } if *name == boom));
        if w.obj(chute).is_none() {
            break;
        }
    }
    need!(
        name,
        w.obj(chute).is_none().then_some(()),
        "the cloud never killed the parachutist"
    );
    assert!(seen, "a parachutist killed by a cloud explodes");
    pass(name);
}

/// A drainer vehicle and the slot of its draining weapon.
fn drainer(r: &SimRules) -> Option<(String, usize)> {
    units(r, &[Kind::Vehicle]).into_iter().find_map(|(k, i)| {
        let u = r.unit(k, i)?;
        let s = u
            .combat
            .slots
            .iter()
            .position(|p| p.is_some_and(|p| r.combat.weapon(p).drain))?;
        Some((u.name.clone(), s))
    })
}

/// House A's drainer put over `b`'s top-left cell, making a drain attempt
/// at it with its draining weapon.
fn drain_with(w: &mut World, dn: &str, slot: usize, b: ObjectId) -> ObjectId {
    let cell = w.obj(b).unwrap().pos.cell();
    let d = put(w, Kind::Vehicle, dn, A, 12, 12);
    objects::take_inside(w, d);
    let p = Pos::centre(cell, 0);
    let ground = w.surface_at(p.x, p.y);
    objects::place(w, d, Pos::new(p.x, p.y, ground + 200), 0);
    w.obj_mut(d).unwrap().target = Some(Target::Object(b));
    fire::shot(w, d, slot, Target::Object(b));
    d
}

fn drainable(
    r: &SimRules,
    f: impl Fn(&glasnost_sim::rules::BuildingRules) -> bool,
) -> Option<String> {
    find_building(r, |b| b.combat.drainable && f(b))
}

// spec: S-0608 claims 5, 13 and 14, S-0405 claims 5, 7 and 8
#[test]
#[ignore = "local relation lane: needs GLASNOST_GAME_DIR"]
fn m7b_drain_a_drained_plant_stops_the_defences_until_the_disc_dies() {
    let name = "m7b_drain_a_drained_plant_stops_the_defences_until_the_disc_dies";
    let r = need!(name, reference(), "no install");
    let (dn, slot) = need!(name, drainer(&r), "no drainer");
    let plant = need!(
        name,
        drainable(&r, |b| b.raw.econ.power.unwrap_or(0) > 0),
        "no drainable power plant"
    );
    let output = r
        .buildings
        .iter()
        .find(|b| b.name == plant)
        .unwrap()
        .raw
        .econ
        .power
        .unwrap_or(0);
    let def = need!(
        name,
        find_building(&r, |b| b.combat.slots[0].is_some()
            && b.raw.powered.unwrap_or(false)
            && (1..output).contains(&-b.raw.econ.power.unwrap_or(0))),
        "no needs-power defence the plant alone powers"
    );
    let mut w = world(&r);
    let p = put(&mut w, Kind::Building, &plant, B, 0, 0);
    let k = put(&mut w, Kind::Building, &def, B, -8, 4);
    testkit::run(&mut w, 30);
    assert!(power::in_operation(&w, k), "in operation before");
    let d = drain_with(&mut w, &dn, slot, p);
    assert_eq!(drain::partner(&w, d), Some(p), "linked");
    testkit::run(&mut w, 30);
    assert!(!power::in_operation(&w, k), "stopped while drained");
    kill(&mut w, d);
    testkit::run(&mut w, 30);
    assert!(power::in_operation(&w, k), "back when the drainer dies");
    pass(name);
}

// spec: S-0608 claim 18, S-0405 claims 5 and 7, S-0605 claims 6 and 7
#[test]
#[ignore = "local relation lane: needs GLASNOST_GAME_DIR"]
fn m7b_drain_a_drained_absorber_blacks_out_only_with_passengers() {
    let name = "m7b_drain_a_drained_absorber_blacks_out_only_with_passengers";
    let r = need!(name, reference(), "no install");
    let (dn, slot) = need!(name, drainer(&r), "no drainer");
    let bio = need!(
        name,
        drainable(&r, |b| b.absorb.infantry && b.absorb.capacity > 0),
        "no drainable infantry absorber"
    );
    let inf = need!(name, absorbable(&r, &bio), "no infantry the absorber takes");
    let mut w = world(&r);
    let b = put(&mut w, Kind::Building, &bio, B, 0, 0);
    let g = put(&mut w, Kind::Infantry, &inf, B, 5, 0);
    click(&mut w, B, &[g], b);
    assert!(
        run_until(&mut w, 1500, |w| absorb::absorbed_in(w, g) == Some(b)),
        "goes in"
    );
    testkit::run(&mut w, 30);
    let (with, _) = power::totals(&w, HouseId(B));
    need!(
        name,
        (with > 0).then_some(()),
        "the absorber with a passenger gives no output"
    );
    let d = drain_with(&mut w, &dn, slot, b);
    assert_eq!(drain::partner(&w, d), Some(b), "linked");
    testkit::run(&mut w, 30);
    let (drained, _) = power::totals(&w, HouseId(B));
    assert!(drained < with, "the drained absorber's output is gone");
    let hb = health(&w, b);
    kill(&mut w, d);
    testkit::run(&mut w, 30);
    let restored = power::totals(&w, HouseId(B)).0;
    if health(&w, b) == hb {
        assert_eq!(restored, with, "restored when the link ends");
        pass(name);
    } else {
        // The drainer's death hurt the absorber, whose output follows its
        // health (S-0405): only the direction is checked.
        assert!(restored > drained, "restored when the link ends");
        println!("relation {name}: PASS (the drainer's death hurt the absorber)");
    }
}

// spec: S-0608 claims 7 and 17, S-0603, S-0408
#[test]
#[ignore = "local relation lane: needs GLASNOST_GAME_DIR"]
fn m7b_drain_a_drained_grinder_pays_and_cannot_be_sold() {
    let name = "m7b_drain_a_drained_grinder_pays_and_cannot_be_sold";
    let r = need!(name, reference(), "no install");
    let (dn, slot) = need!(name, drainer(&r), "no drainer");
    let grnd = need!(
        name,
        drainable(&r, |b| b.raw.grinding.unwrap_or(false)),
        "no drainable recycling building"
    );
    let mut w = world(&r);
    powered(&mut w, B);
    let g = put(&mut w, Kind::Building, &grnd, B, 0, 0);
    let d = drain_with(&mut w, &dn, slot, g);
    assert_eq!(drain::partner(&w, d), Some(g), "linked");
    let resource = r
        .buildings
        .iter()
        .find(|b| b.name == grnd)
        .unwrap()
        .raw
        .econ
        .resource_destination
        .unwrap_or(false);
    let ca = w.houses[0].credits;
    testkit::run(&mut w, 600);
    orders::issue(&mut w, HouseId(B), Order::Sell { actor: g });
    testkit::run(&mut w, 30);
    assert!(
        w.obj(g)
            .is_some_and(|o| o.beh.current != Some(Behaviour::Selling)),
        "not sold while drained"
    );
    if resource {
        assert!(w.houses[0].credits > ca, "the drainer's house is paid");
        pass(name);
    } else {
        assert_eq!(w.houses[0].credits, ca);
        println!(
            "relation {name}: PASS (no payouts: the recycling building is no resource destination)"
        );
    }
}

// spec: S-0608 claims 5 and 17, S-0606, S-0405 claim 9 table T1, S-0408
#[test]
#[ignore = "local relation lane: needs GLASNOST_GAME_DIR"]
fn m7b_drain_a_drained_bunker_refuses_fire_and_sale_but_no_blackout() {
    let name = "m7b_drain_a_drained_bunker_refuses_fire_and_sale_but_no_blackout";
    let r = need!(name, reference(), "no install");
    let (dn, slot) = need!(name, drainer(&r), "no drainer");
    let bunk = need!(
        name,
        drainable(&r, |b| b.raw.bunker.unwrap_or(false)),
        "no drainable tank bunker"
    );
    let mut w = world(&r);
    powered(&mut w, B);
    let b = put(&mut w, Kind::Building, &bunk, B, 0, 0);
    testkit::run(&mut w, 30);
    let low = power::low_power(&w, HouseId(B));
    assert_ne!(power::firing_answer(&w, b), Some(Answer::Illegal));
    let d = drain_with(&mut w, &dn, slot, b);
    assert_eq!(drain::partner(&w, d), Some(b), "linked");
    testkit::run(&mut w, 30);
    assert_eq!(
        power::firing_answer(&w, b),
        Some(Answer::Illegal),
        "illegal while drained"
    );
    orders::issue(&mut w, HouseId(B), Order::Sell { actor: b });
    testkit::run(&mut w, 30);
    assert!(
        w.obj(b)
            .is_some_and(|o| o.beh.current != Some(Behaviour::Selling)),
        "not sold while drained"
    );
    assert_eq!(
        power::low_power(&w, HouseId(B)),
        low,
        "no change of power state"
    );
    pass(name);
}

// spec: S-0608 claim 18, S-0606
#[test]
#[ignore = "local relation lane: needs GLASNOST_GAME_DIR"]
fn m7b_drain_a_drained_bunker_keeps_its_vehicle_firing() {
    let name = "m7b_drain_a_drained_bunker_keeps_its_vehicle_firing";
    let r = need!(name, reference(), "no install");
    let (dn, slot) = need!(name, drainer(&r), "no drainer");
    let bunk = need!(
        name,
        drainable(&r, |b| b.raw.bunker.unwrap_or(false)),
        "no drainable tank bunker"
    );
    let vn = need!(name, bunker_vehicle(&r), "no vehicle a bunker takes");
    let (tk, tn) = need!(name, victim(&r, &[Kind::Vehicle]), "no plain vehicle");
    let mut w = world(&r);
    powered(&mut w, B);
    let b = put(&mut w, Kind::Building, &bunk, B, 0, 0);
    let v = put(&mut w, Kind::Vehicle, &vn, B, 6, 1);
    assert!(bunker_in(&mut w, v, b), "bunkered");
    let foe = put(&mut w, tk, &tn, A, -3, 1);
    let before = check::check(&w, v, Some(Target::Object(foe)), 0, false);
    let d = drain_with(&mut w, &dn, slot, b);
    assert_eq!(drain::partner(&w, d), Some(b), "linked");
    testkit::run(&mut w, 5);
    assert_eq!(bunker::bunkered_in(&w, v), Some(b), "not ejected");
    let after = check::check(&w, v, Some(Target::Object(foe)), 0, false);
    assert_ne!(after, Answer::Illegal, "its own answer, not the bunker's");
    assert_eq!(after == Answer::Illegal, before == Answer::Illegal);
    pass(name);
}

// spec: S-0608 claim 18, S-0603
#[test]
#[ignore = "local relation lane: needs GLASNOST_GAME_DIR"]
fn m7b_drain_a_drained_grinder_still_recycles_for_its_owner() {
    let name = "m7b_drain_a_drained_grinder_still_recycles_for_its_owner";
    let r = need!(name, reference(), "no install");
    let (dn, slot) = need!(name, drainer(&r), "no drainer");
    let grnd = need!(
        name,
        drainable(&r, |b| b.raw.grinding.unwrap_or(false)),
        "no drainable recycling building"
    );
    let (ik, iname) = need!(name, victim(&r, &[Kind::Infantry]), "no plain infantry");
    let mut w = world(&r);
    powered(&mut w, B);
    let g = put(&mut w, Kind::Building, &grnd, B, 0, 0);
    let u = put(&mut w, ik, &iname, B, 6, 0);
    let d = drain_with(&mut w, &dn, slot, g);
    assert_eq!(drain::partner(&w, d), Some(g), "linked");
    // Payouts to the drainer come every so often; recycling's credit is
    // the jump at the consumption tick, beyond any payout of that tick.
    click(&mut w, B, &[u], g);
    let mut gained = false;
    for _ in 0..1500 {
        let cb = w.houses[1].credits;
        tick::step(&mut w);
        if w.obj(u).is_none() {
            gained = w.houses[1].credits > cb || {
                testkit::run(&mut w, 2);
                w.houses[1].credits > cb
            };
            break;
        }
    }
    assert!(w.obj(u).is_none(), "consumed while drained");
    assert!(gained, "the recycling credit goes to the grinder's owner");
    assert_eq!(drain::partner(&w, d), Some(g), "the link stays");
    pass(name);
}

// spec: S-0611 claims 4, 7, 9 and 10, S-0601 claims 10 and 12
#[test]
#[ignore = "local relation lane: needs GLASNOST_GAME_DIR"]
fn m7b_mutator_a_mutated_controlled_unit_and_a_mutated_controller() {
    let name = "m7b_mutator_a_mutated_controlled_unit_and_a_mutated_controller";
    let r = need!(name, reference(), "no install");
    need!(name, mutation_ready(&r), "no genetic mutation data");
    let (ck, cn) = need!(name, controller(&r), "no limited controller");
    need!(
        name,
        (ck == Kind::Infantry).then_some(()),
        "no infantry controller"
    );
    let (ik, iname) = need!(name, victim(&r, &[Kind::Infantry]), "no plain infantry");
    let (tk, tn) = need!(name, victim(&r, &[Kind::Vehicle]), "no plain vehicle");
    let mut w = world(&r);
    let ctl = put(&mut w, ck, &cn, A, -10, 0);
    let g = put(&mut w, ik, &iname, B, 0, 0);
    let t = put(&mut w, tk, &tn, B, -10, 5);
    assert!(mind::take(&mut w, ctl, g) && mind::take(&mut w, ctl, t));
    testkit::run(&mut w, 3);
    let made = mutate(&mut w, B, at(0, 0));
    assert!(w.obj(g).is_none(), "converted");
    assert!(
        mind::controller(&w, ctl).is_none_or(|c| !c.victims.contains(&g)),
        "left its controller's list"
    );
    assert!(
        made.iter()
            .all(|&u| owner(&w, u) == Some(B) && !mind::mind_controlled(&w, u))
    );
    mutate(&mut w, B, at(-10, 0));
    testkit::run(&mut w, 2);
    assert!(w.obj(ctl).is_none(), "the controller converts");
    assert_eq!(owner(&w, t), Some(B), "its other victim goes home");
    pass(name);
}

// spec: S-0611 claim 4, S-0606 claims 7 and 14
#[test]
#[ignore = "local relation lane: needs GLASNOST_GAME_DIR"]
fn m7b_mutator_a_launch_on_a_tank_bunker_leaves_it_and_its_vehicle() {
    let name = "m7b_mutator_a_launch_on_a_tank_bunker_leaves_it_and_its_vehicle";
    let r = need!(name, reference(), "no install");
    need!(name, mutation_ready(&r), "no genetic mutation data");
    let bunk = need!(name, tank_bunker(&r), "no tank bunker");
    let vn = need!(name, bunker_vehicle(&r), "no vehicle a bunker takes");
    let (ik, iname) = need!(name, victim(&r, &[Kind::Infantry]), "no plain infantry");
    let mut w = world(&r);
    powered(&mut w, A);
    let b = put(&mut w, Kind::Building, &bunk, A, 0, 0);
    let v = put(&mut w, Kind::Vehicle, &vn, A, 6, 1);
    assert!(bunker_in(&mut w, v, b), "bunkered");
    let beside = put(&mut w, ik, &iname, A, -1, 0);
    let hv = health(&w, v);
    mutate(&mut w, B, at(0, 0));
    assert_eq!(bunker::bunkered_in(&w, v), Some(b), "still bunkered");
    assert_eq!(health(&w, v), hv, "the bunkered vehicle is not hit");
    assert!(w.obj(beside).is_none(), "the infantry beside converts");
    pass(name);
}

// spec: S-0611 claim 5, S-0605
#[test]
#[ignore = "local relation lane: needs GLASNOST_GAME_DIR"]
fn m7b_mutator_a_launch_on_an_absorber_spares_its_passenger_and_takes_an_entrant() {
    let name = "m7b_mutator_a_launch_on_an_absorber_spares_its_passenger_and_takes_an_entrant";
    let r = need!(name, reference(), "no install");
    need!(name, mutation_ready(&r), "no genetic mutation data");
    let bio = need!(name, absorber(&r), "no infantry absorber");
    let inf = need!(name, absorbable(&r, &bio), "no infantry the absorber takes");
    let mut w = world(&r);
    powered(&mut w, A);
    let b = put(&mut w, Kind::Building, &bio, A, 0, 0);
    let inside = put(&mut w, Kind::Infantry, &inf, A, 5, 0);
    click(&mut w, A, &[inside], b);
    assert!(
        run_until(&mut w, 1500, |w| absorb::absorbed_in(w, inside) == Some(b)),
        "goes in"
    );
    let h = health(&w, inside);
    let entrant = put(&mut w, Kind::Infantry, &inf, A, -1, 1);
    click(&mut w, A, &[entrant], b);
    tick::step(&mut w);
    mutate(&mut w, B, at(0, 0));
    assert_eq!(absorb::absorbed_in(&w, inside), Some(b), "still inside");
    assert_eq!(health(&w, inside), h, "not hit");
    assert!(w.obj(entrant).is_none(), "the entrant converts");
    pass(name);
}

// spec: S-0611 claim 4, S-0607 claims 5 and 12
#[test]
#[ignore = "local relation lane: needs GLASNOST_GAME_DIR"]
fn m7b_mutator_a_launch_under_a_seized_vehicle_takes_only_the_infantry() {
    let name = "m7b_mutator_a_launch_under_a_seized_vehicle_takes_only_the_infantry";
    let r = need!(name, reference(), "no install");
    need!(name, mutation_ready(&r), "no genetic mutation data");
    let (ln, dmg) = need!(name, lifter(&r), "no lifter");
    let vn = need!(
        name,
        liftable(&r, dmg, |_| true),
        "no vehicle the lifter seizes"
    );
    let (ik, iname) = need!(name, victim(&r, &[Kind::Infantry]), "no plain infantry");
    let mut w = world(&r);
    let v = put(&mut w, Kind::Vehicle, &vn, A, 6, 0);
    let l = put(&mut w, Kind::Vehicle, &ln, B, -2, 0);
    assert!(lift_up(&mut w, l, v), "seized");
    let c = w.obj(v).unwrap().pos.cell();
    let under = testkit::spawn(&mut w, ik, &iname, A, c, 0);
    let hv = health(&w, v);
    let i = sw_index(w.rules(), SwKind::Genetic).unwrap_or(0);
    mutation::launch(&mut w, HouseId(B), i, c);
    // The launch's hits land at once; the death follows in a later tick.
    assert_eq!(health(&w, v), hv, "no hit on the vehicle");
    assert!(
        run_until(&mut w, 30, |w| w.obj(under).is_none()),
        "the infantry beneath converts"
    );
    assert!(lift::seized(&w, v), "still seized");
    pass(name);
}

// spec: S-0612 claims 13, 14 and 15, S-0601 claims 9 and 11
#[test]
#[ignore = "local relation lane: needs GLASNOST_GAME_DIR"]
fn m7b_dominator_a_domination_takes_a_controller_and_a_controlled_unit() {
    let name = "m7b_dominator_a_domination_takes_a_controller_and_a_controlled_unit";
    let r = need!(name, reference(), "no install");
    need!(
        name,
        sw_index(&r, SwKind::Dominator),
        "no psychic domination type"
    );
    let (ck, cn) = need!(name, controller(&r), "no limited controller");
    let (tk, tn) = need!(name, victim(&r, &[Kind::Vehicle]), "no plain vehicle");
    // A controlled unit in the swept area, its controller far outside.
    let mut w = world(&r);
    let ctl = put(&mut w, ck, &cn, A, -18, 0);
    let v = put(&mut w, tk, &tn, B, 0, 0);
    assert!(mind::take(&mut w, ctl, v));
    testkit::run(&mut w, 2);
    let taken = need!(
        name,
        dominate(&mut w, B, at(0, 0)),
        "the domination never sweeps"
    );
    assert!(taken.contains(&v), "taken over");
    assert_eq!(owner(&w, v), Some(B));
    assert!(mind::permanent(&w, v), "permanently controlled");
    assert!(
        mind::controller(&w, ctl).is_none_or(|c| c.victims.is_empty()),
        "released first"
    );
    // A controller in the swept area keeps its victim outside it.
    let immune = r
        .unit(ck, r.index_of(ck, &cn).unwrap())
        .unwrap()
        .combat
        .immune_psionics;
    if immune {
        // spec: S-0612 claim 13 (a psionics-immune unit is not taken)
        println!("relation {name}: PASS (part 2 not formed: the controller is psionics-immune)");
        return;
    }
    let mut w = world(&r);
    let ctl = put(&mut w, ck, &cn, B, 0, 0);
    let v = put(&mut w, tk, &tn, A, -18, 0);
    assert!(mind::take(&mut w, ctl, v));
    testkit::run(&mut w, 2);
    let taken = dominate(&mut w, A, at(0, 0)).unwrap();
    if w.obj(ctl).is_none() {
        println!(
            "relation {name}: PASS (part 2 not formed: the sweep's damage killed the controller)"
        );
        return;
    }
    assert!(taken.contains(&ctl), "the controller is taken");
    assert_eq!(
        mind::controlled_by(&w, v),
        Some(ctl),
        "and keeps its victim"
    );
    pass(name);
}

// spec: S-0612 claims 12 and 13, S-0606 claim 15
#[test]
#[ignore = "local relation lane: needs GLASNOST_GAME_DIR"]
fn m7b_dominator_a_domination_takes_a_bunkered_vehicle_and_leaves_the_link() {
    let name = "m7b_dominator_a_domination_takes_a_bunkered_vehicle_and_leaves_the_link";
    let r = need!(name, reference(), "no install");
    need!(
        name,
        sw_index(&r, SwKind::Dominator),
        "no psychic domination type"
    );
    let bunk = need!(
        name,
        find_building(&r, |b| b.raw.bunker.unwrap_or(false)
            && b.foundation == (2, 2)),
        "no two by two tank bunker (claim 12's case)"
    );
    let vn = need!(name, bunker_vehicle(&r), "no vehicle a bunker takes");
    let mut note = None;
    for rr in [r.clone(), without_sweep_damage(&r)] {
        let mut w = world(&rr);
        powered(&mut w, A);
        let b = put(&mut w, Kind::Building, &bunk, A, 0, 0);
        let v = put(&mut w, Kind::Vehicle, &vn, A, 6, 1);
        assert!(bunker_in(&mut w, v, b), "bunkered");
        let c = w.obj(v).unwrap().pos.cell();
        let taken = need!(name, dominate(&mut w, B, c), "the domination never sweeps");
        if w.obj(b).is_none() {
            // The sweep's damage destroyed the bunker first: the vehicle
            // left it; run again without the damage.
            note = Some("sweep damage 0: the reference sweep destroys the bunker");
            continue;
        }
        assert!(taken.contains(&v), "the bunkered vehicle is taken");
        assert_eq!(owner(&w, b), Some(A), "the bunker is not");
        assert_eq!(bunker::bunkered_in(&w, v), Some(b), "the link stays");
        match note {
            Some(n) => println!("relation {name}: PASS ({n})"),
            None => pass(name),
        }
        return;
    }
    unreachable!("the bunker survives a sweep without damage");
}

// spec: S-0612 claims 12, 13 and 15, S-0605
#[test]
#[ignore = "local relation lane: needs GLASNOST_GAME_DIR"]
fn m7b_dominator_a_domination_spares_an_absorbed_passenger() {
    let name = "m7b_dominator_a_domination_spares_an_absorbed_passenger";
    let r = need!(name, reference(), "no install");
    need!(
        name,
        sw_index(&r, SwKind::Dominator),
        "no psychic domination type"
    );
    let bio = need!(name, absorber(&r), "no infantry absorber");
    let inf = need!(name, absorbable(&r, &bio), "no infantry the absorber takes");
    let (tk, tn) = need!(name, victim(&r, &[Kind::Vehicle]), "no plain vehicle");
    let fw = r
        .buildings
        .iter()
        .find(|x| x.name == bio)
        .unwrap()
        .foundation;
    let mut note = None;
    for rr in [r.clone(), without_sweep_damage(&r)] {
        let mut w = world(&rr);
        powered(&mut w, A);
        let b = put(&mut w, Kind::Building, &bio, A, 0, 0);
        let inside = put(&mut w, Kind::Infantry, &inf, A, 5, 0);
        click(&mut w, A, &[inside], b);
        assert!(
            run_until(&mut w, 1500, |w| absorb::absorbed_in(w, inside) == Some(b)),
            "goes in"
        );
        let beside = put(&mut w, tk, &tn, A, fw.0, 0);
        let taken = need!(
            name,
            dominate(&mut w, B, at(0, 0)),
            "the domination never sweeps"
        );
        if w.obj(b).is_none() {
            // The sweep's damage destroyed the absorber first and let its
            // passenger out; run again without the damage.
            note = Some("sweep damage 0: the reference sweep destroys the absorber");
            continue;
        }
        assert!(
            !taken.contains(&inside) && owner(&w, inside) == Some(A),
            "the passenger keeps its owner"
        );
        assert_eq!(owner(&w, b), Some(A), "the building is not taken");
        let beside_taken = taken.contains(&beside);
        if beside_taken {
            assert_eq!(owner(&w, beside), Some(B), "the unit beside is taken");
        }
        let mut parts: Vec<&str> = note.into_iter().collect();
        if !beside_taken {
            // spec: S-0612 claim 11 (the swept cells: ring metric at most min(r, 10), S-0303's metric)
            let c = w.obj(beside).unwrap().pos.cell();
            let t = at(0, 0);
            let (dx, dy) = ((c.x - t.x).abs(), (c.y - t.y).abs());
            let ring = dx.max(dy) + dx.min(dy) / 2;
            let reach = rr.superweapons.domination.range.min(10);
            assert!(ring > reach, "the unit beside in a swept cell is taken");
            parts.push("the unit beside lies outside the swept cells");
        }
        if parts.is_empty() {
            pass(name);
        } else {
            println!("relation {name}: PASS ({})", parts.join("; "));
        }
        return;
    }
    unreachable!("the absorber survives a sweep without damage");
}

/// House `h`'s infantry factory `k` releases a new infantry unit of type
/// `ti` (S-0404 claim 13), once every earlier unit has left its building's
/// contact place; returns the units of that type that appeared besides it
/// (the copies), in creation order, and orders all of them away.
fn release_one(w: &mut World, h: u8, k: ObjectId, ti: u32) -> Vec<ObjectId> {
    assert!(
        run_until(w, 900, |w| w.objects.values().all(|o| o.contact.is_none())),
        "the earlier units leave"
    );
    let before: Vec<ObjectId> = w.objects.keys().copied().collect();
    let g = objects::create(w, Kind::Infantry, ti, Some(HouseId(h)));
    assert_eq!(release::release(w, k, g), Outcome::Released);
    let new: Vec<(ObjectId, u8)> = w
        .objects
        .values()
        .filter(|o| o.kind == Kind::Infantry && o.type_index == ti && o.on_map)
        .filter(|o| !before.contains(&o.id))
        .map(|o| (o.id, o.owner.map_or(h, |x| x.0)))
        .collect();
    for (i, &(id, house)) in new.iter().enumerate() {
        move_to(w, house, id, at(i as i32 * 2 - 8, 14));
    }
    new.into_iter()
        .map(|(id, _)| id)
        .filter(|&id| id != g)
        .collect()
}

/// The buildings the copies of `release_one` came from.
fn released(w: &mut World, h: u8, k: ObjectId, ti: u32) -> Vec<Option<ObjectId>> {
    let c = release_one(w, h, k, ti);
    contacts(w, &c)
}

fn contacts(w: &World, c: &[ObjectId]) -> Vec<Option<ObjectId>> {
    c.iter()
        .map(|&g| w.obj(g).and_then(|o| o.contact))
        .collect()
}

fn type_index(r: &SimRules, kind: Kind, name: &str) -> u32 {
    r.index_of(kind, name).unwrap()
}

// spec: S-0513 claims 27, 30 and 31, S-0601 claims 3, 4 and 5, S-0404 claim 14, S-0405 claim 9
#[test]
#[ignore = "local relation lane: needs GLASNOST_GAME_DIR"]
fn m7b_shield_a_shielded_vat_refuses_mind_control_and_keeps_cloning() {
    let name = "m7b_shield_a_shielded_vat_refuses_mind_control_and_keeps_cloning";
    let r = need!(name, reference(), "no install");
    need!(
        name,
        sw_index(&r, SwKind::ForceShield),
        "no force shield type"
    );
    let (vat, fac) = need!(
        name,
        cloning(&r),
        "no cloning building and infantry factory"
    );
    let (ck, cn) = need!(name, controller(&r), "no limited controller");
    let (_, inf) = need!(name, victim(&r, &[Kind::Infantry]), "no plain infantry");
    let ti = type_index(&r, Kind::Infantry, &inf);
    let mut w = world(&r);
    let v = put(&mut w, Kind::Building, &vat, A, 0, 0);
    let open = put(&mut w, Kind::Building, &vat, A, -16, -16);
    let k = put(&mut w, Kind::Building, &fac, A, 14, -16);
    let ctl = put(&mut w, ck, &cn, B, 4, 6);
    shield(&mut w, A, at(0, 0));
    assert!(superweapons::protected(&w, v), "shielded");
    assert!(!superweapons::protected(&w, open), "outside the radius");
    assert!(
        power::blackout_running(&w, HouseId(A)),
        "the launching house is blacked out"
    );
    assert!(!mind::can_take(&w, ctl, v));
    assert_eq!(
        check::check(&w, ctl, Some(Target::Object(v)), 0, false),
        Answer::Illegal
    );
    mind::detonation(&mut w, Some(ctl), Some(Target::Object(v)));
    assert_eq!(owner(&w, v), Some(A), "a landing takes nothing");
    let copies = release_one(&mut w, A, k, ti);
    assert!(
        contacts(&w, &copies).contains(&Some(v)),
        "the shielded vat copies, blacked out"
    );
    if w.rules()
        .buildings
        .iter()
        .find(|b| b.name == vat)
        .unwrap()
        .combat
        .immune_psionics
    {
        println!(
            "relation {name}: PASS (control part: the cloning building is psionics-immune, so only the refusal shows)"
        );
        return;
    }
    mind::detonation(&mut w, Some(ctl), Some(Target::Object(open)));
    assert_eq!(
        owner(&w, open),
        Some(B),
        "the same landing takes the open vat"
    );
    pass(name);
}

/// A unit of house `h` attacks `b` with gattling `g` and runs until it
/// has spun up past its first stage and `hurt` (if any) has lost health.
fn gattling_at(w: &mut World, gats: &[ObjectId], hurt: ObjectId) -> bool {
    let h0 = health(w, hurt);
    run_healed(w, 900, gats, |w| {
        gats.iter().all(|&g| gattling::spin(w, g).stage >= 1) && health(w, hurt) < h0
    })
}

// spec: S-0513 claim 28, S-0302 claim 8, S-0609 claim 5
#[test]
#[ignore = "local relation lane: needs GLASNOST_GAME_DIR"]
fn m7b_shield_a_gattling_fires_at_a_shielded_building_and_spins_up_for_nothing() {
    let name = "m7b_shield_a_gattling_fires_at_a_shielded_building_and_spins_up_for_nothing";
    let r = need!(name, reference(), "no install");
    need!(
        name,
        sw_index(&r, SwKind::ForceShield),
        "no force shield type"
    );
    let g = need!(name, gattling_vehicle(&r, false), "no gattling vehicle");
    let bn = need!(name, plain_building(&r), "no plain building");
    let mut w = world(&r);
    let shielded = put(&mut w, Kind::Building, &bn, A, 0, 0);
    let open = put(&mut w, Kind::Building, &bn, A, 0, 16);
    let g1 = put(&mut w, Kind::Vehicle, &g, B, 5, 0);
    let g2 = put(&mut w, Kind::Vehicle, &g, B, 5, 16);
    shield(&mut w, A, at(0, 0));
    assert!(superweapons::protected(&w, shielded) && !superweapons::protected(&w, open));
    assert_ne!(
        check::check(&w, g1, Some(Target::Object(shielded)), 0, false),
        Answer::Illegal,
        "a player house may fire"
    );
    let h = health(&w, shielded);
    attack(&mut w, B, g1, shielded);
    attack(&mut w, B, g2, open);
    assert!(
        gattling_at(&mut w, &[g1, g2], open),
        "both spin up; the open one is hurt"
    );
    assert!(superweapons::protected(&w, shielded), "still shielded");
    assert_eq!(health(&w, shielded), h, "the hits do nothing");
    pass(name);
}

// spec: S-0613 claim 13, S-0513 claim 31, S-0302 claim 8
#[test]
#[ignore = "local relation lane: needs GLASNOST_GAME_DIR"]
fn m7b_a_gas_cloud_does_nothing_to_a_shielded_building() {
    let name = "m7b_a_gas_cloud_does_nothing_to_a_shielded_building";
    let r = need!(name, reference(), "no install");
    need!(
        name,
        sw_index(&r, SwKind::ForceShield),
        "no force shield type"
    );
    let ti = need!(name, gas_particle(&r), "no damaging gas particle");
    let bn = need!(name, plain_building(&r), "no plain building");
    let mut w = world(&r);
    let shielded = put(&mut w, Kind::Building, &bn, A, 0, 0);
    let open = put(&mut w, Kind::Building, &bn, A, 0, 16);
    shield(&mut w, A, at(0, 0));
    assert!(superweapons::protected(&w, shielded) && !superweapons::protected(&w, open));
    let (hs, ho) = (health(&w, shielded), health(&w, open));
    for id in [shielded, open] {
        let p = w.obj(id).unwrap().pos;
        cloud_at(&mut w, ti, p);
    }
    let hurt = run_until(&mut w, 300, |w| health(w, open) < ho);
    need!(
        name,
        hurt.then_some(()),
        "the cloud never hurt the open building"
    );
    assert!(superweapons::protected(&w, shielded));
    assert_eq!(health(&w, shielded), hs, "nothing to the shielded one");
    pass(name);
}

// spec: S-0513 claim 31, S-0302 claim 8, S-0612 claims 9, 10 and 13
#[test]
#[ignore = "local relation lane: needs GLASNOST_GAME_DIR"]
fn m7b_shield_a_domination_over_a_shielded_base_takes_its_units_and_spares_its_buildings() {
    let name =
        "m7b_shield_a_domination_over_a_shielded_base_takes_its_units_and_spares_its_buildings";
    let r = need!(name, reference(), "no install");
    need!(
        name,
        sw_index(&r, SwKind::ForceShield),
        "no force shield type"
    );
    need!(
        name,
        sw_index(&r, SwKind::Dominator),
        "no psychic domination type"
    );
    let bn = need!(name, plain_building(&r), "no plain building");
    let (tk, tn) = need!(name, victim(&r, &[Kind::Vehicle]), "no plain vehicle");
    let fw = r
        .buildings
        .iter()
        .find(|b| b.name == bn)
        .unwrap()
        .foundation
        .0;
    let mut w = world(&r);
    // A's building west of the centre, shielded; B's east of it, the
    // same distance, not shielded (not allied with A).
    let mine = put(&mut w, Kind::Building, &bn, A, -1 - fw, 0);
    let theirs = put(&mut w, Kind::Building, &bn, B, 1, 0);
    let unit = put(&mut w, tk, &tn, A, 0, 0);
    shield(&mut w, A, at(-1 - fw, 0));
    assert!(superweapons::protected(&w, mine) && !superweapons::protected(&w, theirs));
    assert!(
        !superweapons::protected(&w, unit),
        "a unit is never shielded"
    );
    let (hm, ht) = (health(&w, mine), health(&w, theirs));
    let taken = need!(
        name,
        dominate(&mut w, B, at(0, 0)),
        "the domination never sweeps"
    );
    assert!(taken.contains(&unit), "the unit is taken");
    assert_eq!(owner(&w, mine), Some(A), "no building is taken");
    assert_eq!(health(&w, mine), hm, "no damage under the shield");
    if health(&w, theirs) < ht {
        pass(name);
    } else {
        println!("relation {name}: PASS (the sweep's damage reached neither building)");
    }
}

// spec: S-0513 claim 31, S-0611 claims 4, 9 and 10
#[test]
#[ignore = "local relation lane: needs GLASNOST_GAME_DIR"]
fn m7b_shield_a_mutation_inside_a_shield_converts_its_infantry() {
    let name = "m7b_shield_a_mutation_inside_a_shield_converts_its_infantry";
    let r = need!(name, reference(), "no install");
    need!(
        name,
        sw_index(&r, SwKind::ForceShield),
        "no force shield type"
    );
    need!(name, mutation_ready(&r), "no genetic mutation data");
    let bn = need!(name, plain_building(&r), "no plain building");
    let (ik, iname) = need!(name, victim(&r, &[Kind::Infantry]), "no plain infantry");
    let fw = r
        .buildings
        .iter()
        .find(|b| b.name == bn)
        .unwrap()
        .foundation
        .0;
    let mut w = world(&r);
    let b = put(&mut w, Kind::Building, &bn, A, 0, 0);
    let g = put(&mut w, ik, &iname, A, fw, 0);
    shield(&mut w, A, at(0, 0));
    assert!(superweapons::protected(&w, b) && !superweapons::protected(&w, g));
    let made = mutate(&mut w, B, at(fw, 0));
    assert!(w.obj(g).is_none(), "converted inside the shield");
    assert!(
        made.iter().all(|&u| owner(&w, u) == Some(B)),
        "for the launching house"
    );
    pass(name);
}

// spec: S-0513 claims 27 and 31, S-0607 claims 3 and 5
#[test]
#[ignore = "local relation lane: needs GLASNOST_GAME_DIR"]
fn m7b_shield_a_lift_seizes_a_vehicle_inside_a_shield() {
    let name = "m7b_shield_a_lift_seizes_a_vehicle_inside_a_shield";
    let r = need!(name, reference(), "no install");
    need!(
        name,
        sw_index(&r, SwKind::ForceShield),
        "no force shield type"
    );
    let bn = need!(name, plain_building(&r), "no plain building");
    let (ln, dmg) = need!(name, lifter(&r), "no lifter");
    let vn = need!(
        name,
        liftable(&r, dmg, |_| true),
        "no vehicle the lifter seizes"
    );
    let fw = r
        .buildings
        .iter()
        .find(|b| b.name == bn)
        .unwrap()
        .foundation
        .0;
    let mut w = world(&r);
    let b = put(&mut w, Kind::Building, &bn, A, 0, 0);
    let v = put(&mut w, Kind::Vehicle, &vn, A, fw, 0);
    let l = put(&mut w, Kind::Vehicle, &ln, B, fw + 6, 0);
    shield(&mut w, A, at(0, 0));
    assert!(superweapons::protected(&w, b) && !superweapons::protected(&w, v));
    assert!(lift_up(&mut w, l, v), "seized inside the shield");
    pass(name);
}

// spec: S-0513 claims 27, 28 and 31, S-0608 claims 3 and 5; Q-1507 (open, interim: not refused, the link forms)
#[test]
#[ignore = "local relation lane: needs GLASNOST_GAME_DIR"]
fn m7b_drain_a_shielded_plant_is_drained_under_the_q1507_interim() {
    let name = "m7b_drain_a_shielded_plant_is_drained_under_the_q1507_interim";
    let r = need!(name, reference(), "no install");
    need!(
        name,
        sw_index(&r, SwKind::ForceShield),
        "no force shield type"
    );
    let (dn, slot) = need!(name, drainer(&r), "no drainer");
    let plant = need!(name, drainable(&r, |_| true), "no drainable building");
    let mut w = world(&r);
    let p = put(&mut w, Kind::Building, &plant, B, 0, 0);
    shield(&mut w, B, at(0, 0));
    assert!(superweapons::protected(&w, p));
    let cell = w.obj(p).unwrap().pos.cell();
    let d = put(&mut w, Kind::Vehicle, &dn, A, 12, 12);
    objects::take_inside(&mut w, d);
    let c = Pos::centre(cell, 0);
    let ground = w.surface_at(c.x, c.y);
    objects::place(&mut w, d, Pos::new(c.x, c.y, ground + 200), 0);
    w.obj_mut(d).unwrap().target = Some(Target::Object(p));
    assert_ne!(
        check::check(&w, d, Some(Target::Object(p)), slot, false),
        Answer::Illegal,
        "not refused"
    );
    fire::shot(&mut w, d, slot, Target::Object(p));
    assert_eq!(drain::partner(&w, d), Some(p), "the link forms");
    pass(name);
}

// spec: S-0404 claims 7 and 14, S-0601 claims 3, 6 and 9
#[test]
#[ignore = "local relation lane: needs GLASNOST_GAME_DIR"]
fn m7b_cloning_a_controlled_vat_clones_for_its_controller_and_ranks_last_when_released() {
    let name =
        "m7b_cloning_a_controlled_vat_clones_for_its_controller_and_ranks_last_when_released";
    let r = need!(name, reference(), "no install");
    let (vat, fac) = need!(
        name,
        cloning(&r),
        "no cloning building and infantry factory"
    );
    let immune = r
        .buildings
        .iter()
        .find(|b| b.name == vat)
        .unwrap()
        .combat
        .immune_psionics;
    need!(
        name,
        (!immune).then_some(()),
        "the cloning building is psionics-immune: it cannot be taken"
    );
    let (ck, cn) = need!(name, controller(&r), "no limited controller");
    let (_, inf) = need!(name, victim(&r, &[Kind::Infantry]), "no plain infantry");
    let ti = type_index(&r, Kind::Infantry, &inf);
    let mut w = world(&r);
    let v1 = put(&mut w, Kind::Building, &vat, A, -12, 4);
    let v2 = put(&mut w, Kind::Building, &vat, A, 8, 4);
    let ka = put(&mut w, Kind::Building, &fac, A, -12, -12);
    let kb = put(&mut w, Kind::Building, &fac, B, 8, -12);
    let ctl = put(&mut w, ck, &cn, B, -6, 0);
    assert_eq!(released(&mut w, A, ka, ti), [Some(v1), Some(v2)]);
    assert!(mind::take(&mut w, ctl, v1));
    assert_eq!(
        released(&mut w, B, kb, ti),
        [Some(v1)],
        "copies for the controller"
    );
    assert_eq!(released(&mut w, A, ka, ti), [Some(v2)]);
    mind::release(&mut w, ctl, v1);
    assert_eq!(owner(&w, v1), Some(A));
    assert_eq!(
        released(&mut w, A, ka, ti),
        [Some(v2), Some(v1)],
        "released back, it ranks last"
    );
    pass(name);
}

// spec: S-0611 claims 9 and 10, S-0404 claim 14
#[test]
#[ignore = "local relation lane: needs GLASNOST_GAME_DIR"]
fn m7b_cloning_a_mutation_made_unit_is_not_cloned() {
    let name = "m7b_cloning_a_mutation_made_unit_is_not_cloned";
    let r = need!(name, reference(), "no install");
    need!(name, mutation_ready(&r), "no genetic mutation data");
    let (vat, _) = need!(
        name,
        cloning(&r),
        "no cloning building and infantry factory"
    );
    let (ik, iname) = need!(name, victim(&r, &[Kind::Infantry]), "no plain infantry");
    let mut w = world(&r);
    put(&mut w, Kind::Building, &vat, A, -12, 4);
    put(&mut w, ik, &iname, B, 6, 6);
    let made = mutate(&mut w, A, at(6, 6));
    need!(
        name,
        (!made.is_empty()).then_some(()),
        "the mutation made no unit"
    );
    testkit::run(&mut w, 2);
    for &u in &made {
        let o = w.obj(u).unwrap();
        let n = count_type(&w, o.kind, o.type_index);
        let same = made
            .iter()
            .filter(|&&m| w.obj(m).is_some_and(|x| x.type_index == o.type_index))
            .count();
        assert_eq!(n, same, "no copy of a made unit");
    }
    pass(name);
}

// spec: S-0605 claims 11 and 12, S-0404 claim 14
#[test]
#[ignore = "local relation lane: needs GLASNOST_GAME_DIR"]
fn m7b_cloning_a_passenger_let_out_of_an_absorber_is_not_cloned() {
    let name = "m7b_cloning_a_passenger_let_out_of_an_absorber_is_not_cloned";
    let r = need!(name, reference(), "no install");
    let (vat, _) = need!(
        name,
        cloning(&r),
        "no cloning building and infantry factory"
    );
    let bio = need!(name, absorber(&r), "no infantry absorber");
    let inf = need!(name, absorbable(&r, &bio), "no infantry the absorber takes");
    let ti = type_index(&r, Kind::Infantry, &inf);
    let mut w = world(&r);
    powered(&mut w, A);
    put(&mut w, Kind::Building, &vat, A, -12, 4);
    let b = put(&mut w, Kind::Building, &bio, A, 0, 0);
    let g = put(&mut w, Kind::Infantry, &inf, A, 5, 0);
    click(&mut w, A, &[g], b);
    assert!(
        run_until(&mut w, 1500, |w| absorb::absorbed_in(w, g) == Some(b)),
        "goes in"
    );
    orders::issue(&mut w, HouseId(A), Order::Deploy { actor: b });
    assert!(
        run_until(&mut w, 600, |w| absorb::count(w, b) == 0
            && w.obj(g).is_some_and(|o| o.on_map)),
        "let out"
    );
    testkit::run(&mut w, 2);
    assert_eq!(count_type(&w, Kind::Infantry, ti), 1, "no copy");
    pass(name);
}

// spec: S-0612 claim 13, S-0404 claim 14
#[test]
#[ignore = "local relation lane: needs GLASNOST_GAME_DIR"]
fn m7b_cloning_a_domination_leaves_a_vat_with_its_owner() {
    let name = "m7b_cloning_a_domination_leaves_a_vat_with_its_owner";
    let r = need!(name, reference(), "no install");
    need!(
        name,
        sw_index(&r, SwKind::Dominator),
        "no psychic domination type"
    );
    let (vat, fac) = need!(
        name,
        cloning(&r),
        "no cloning building and infantry factory"
    );
    let (_, inf) = need!(name, victim(&r, &[Kind::Infantry]), "no plain infantry");
    let ti = type_index(&r, Kind::Infantry, &inf);
    for (rr, note) in [
        (r.clone(), None),
        (
            without_sweep_damage(&r),
            Some("sweep damage 0: the reference sweep destroys the cloning building"),
        ),
    ] {
        let mut w = world(&rr);
        let v = put(&mut w, Kind::Building, &vat, A, 0, 0);
        let k = put(&mut w, Kind::Building, &fac, A, -14, -14);
        need!(
            name,
            dominate(&mut w, B, at(0, 0)),
            "the domination never sweeps"
        );
        if w.obj(v).is_none() {
            continue;
        }
        assert_eq!(owner(&w, v), Some(A), "the vat stays its owner's");
        let copies = release_one(&mut w, A, k, ti);
        assert_eq!(contacts(&w, &copies), [Some(v)], "and copies for it");
        assert_eq!(owner(&w, copies[0]), Some(A));
        match note {
            Some(n) => println!("relation {name}: PASS ({n})"),
            None => pass(name),
        }
        return;
    }
    unreachable!("the cloning building survives a sweep without damage");
}
