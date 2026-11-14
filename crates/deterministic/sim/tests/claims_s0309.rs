//! Claim tests for target acquisition (S-0309), and for the parts of the
//! behaviours spec (S-0215 claims 18, 26, 27 and 33) that decide searches
//! and retaliation, on synthetic data.
//!
//! Every type, weapon, warhead and number here is invented. The searches
//! are mostly called directly (`acquire::search`, `acquire::retaliate`) on a
//! world in which no tick has run, so that no other search, processing
//! draw or shot interferes. The ledger is `docs/m4-claims/S-0309.md`.

use std::sync::Arc;

use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{ArmourClass, Decimal, UnitType, WarheadType, WeaponType};
use glasnost_sim::combat::acquire::{self, SearchContext, SearchKind};
use glasnost_sim::rules::{Behaviour, Kind, SimRules};
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseRole, ObjectId, Target, World};

/// Map size of the tests (cells around `mid(SIZE, 0, 0)` are legal).
const SIZE: i32 = 30;

fn d(s: &str) -> Decimal {
    Decimal(s.into())
}

fn at(dx: i32, dy: i32) -> CellXY {
    testkit::mid(SIZE, dx, dy)
}

/// The combat ruleset of the testkit with a few invented additions:
/// - weapon `Half` (damage 10, range 5 cells, projectile Bullet, warhead
///   `Halver` with every versus value 0.5),
/// - vehicle types `T0` … `T9`: unarmed light targets of strength 100 whose
///   special threat value is set per test,
/// - the five threat coefficients of CART set to 0 explicitly.
fn base() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    let s = &mut r.sim;
    s.warheads.insert(
        "Halver".into(),
        WarheadType {
            verses: Some(vec![d("0.5"); 11]),
            ..Default::default()
        },
    );
    s.weapons.insert(
        "Half".into(),
        WeaponType {
            damage: Some(10),
            rof: Some(20),
            range: Some(d("5")),
            speed: Some(100),
            projectile: Some("Bullet".into()),
            warhead: Some("Halver".into()),
            ..Default::default()
        },
    );
    for i in 0..10 {
        let name = format!("T{i}");
        let mut t = UnitType {
            strength: Some(100),
            speed: Some(5),
            rot: Some(5),
            locomotion_class: Some(glasnost_rules::sim::LocomotionClass::Wheel),
            locomotor: Some(glasnost_rules::sim::Locomotor::Drive),
            ..Default::default()
        };
        t.combat.armor = Some(ArmourClass::Light);
        s.vehicles.insert(name.clone(), t);
        r.registries
            .get_mut("vehicles")
            .expect("vehicles registry")
            .push(name);
    }
    coefficients(&mut r, "CART", ["0", "0", "0", "0", "0"]);
    r
}

/// Sets the five threat coefficients of a vehicle type (own-effectiveness,
/// target-effectiveness, special-threat, strength, distance).
fn coefficients(r: &mut Ruleset, ty: &str, c: [&str; 5]) {
    let u = r.sim.vehicles.get_mut(ty).expect("vehicle type");
    u.combat.coefficients = Some(c.map(d));
}

/// Sets the guard range of a vehicle type, in cells.
fn guard(r: &mut Ruleset, ty: &str, cells: &str) {
    r.sim
        .vehicles
        .get_mut(ty)
        .expect("vehicle type")
        .guard_range = Some(d(cells));
}

fn game(r: &Ruleset) -> World {
    testkit::game_with(r, SIZE, 11)
}

fn put(w: &mut World, kind: Kind, ty: &str, house: u8, dx: i32, dy: i32) -> ObjectId {
    testkit::spawn(w, kind, ty, house, at(dx, dy), 0)
}

fn veh(w: &mut World, ty: &str, house: u8, dx: i32, dy: i32) -> ObjectId {
    put(w, Kind::Vehicle, ty, house, dx, dy)
}

/// The per-tick range search of claim 1's first row, called directly.
fn range_search(w: &mut World, a: ObjectId) -> Option<ObjectId> {
    acquire::search(w, a, SearchKind::Range, SearchContext::PerTick)
}

fn target(w: &World, a: ObjectId) -> Option<Target> {
    w.obj(a).and_then(|o| o.target)
}

fn obj(x: ObjectId) -> Option<Target> {
    Some(Target::Object(x))
}

fn opportunity(w: &World, a: ObjectId) -> bool {
    w.obj(a).is_some_and(|o| o.combat.opportunity)
}

// ---------------------------------------------------------------------------
// Claim 24 and table "Search outcome and target state"

// spec: S-0309 claim 24 and table Search outcome row 1 (a dropped opportunity target that the same search picks again is not an opportunity target)
#[test]
fn repicked_opportunity_target_is_not_an_opportunity_target() {
    let mut r = base();
    // Guard range 8 cells (2048 leptons) reaches beyond the gun's 5.5 cells.
    guard(&mut r, "CART", "8");
    let mut w = game(&r);
    let a = veh(&mut w, "CART", 0, 0, 0);
    let b = veh(&mut w, "T0", 1, 7, 0);
    assert_eq!(range_search(&mut w, a), Some(b));
    assert!(opportunity(&w, a), "a new target of the per-tick search");
    // Second search: the opportunity target is out of the gun's range, so it is
    // dropped, and the same search picks it again.
    assert_eq!(range_search(&mut w, a), Some(b));
    assert_eq!(target(&w, a), obj(b));
    assert!(
        !opportunity(&w, a),
        "the object that was the target before the search is not an opportunity target"
    );
}

// spec: S-0309 claim 24 (setting the object itself gives its own cell)
#[test]
fn redirect_self_gives_own_cell() {
    let mut w = game(&base());
    let a = veh(&mut w, "CART", 0, 0, 0);
    assert_eq!(
        acquire::redirect(&w, a, Target::Object(a)),
        Some(Target::Cell(at(0, 0)))
    );
}

// spec: S-0309 claim 24 (a dead object, or an infantry unit whose current sequence is a death kind, gives no target; others are held as set)
#[test]
fn redirect_dead_or_dying_gives_no_target() {
    use glasnost_sim::infantry::Seq;
    let mut w = game(&base());
    let a = veh(&mut w, "CART", 0, 0, 0);
    let b = veh(&mut w, "T0", 1, 2, 0);
    let i = put(&mut w, Kind::Infantry, "WALKER", 1, 3, 0);
    assert_eq!(acquire::redirect(&w, a, Target::Object(b)), obj(b));
    assert_eq!(
        acquire::redirect(&w, a, Target::Cell(at(4, 4))),
        Some(Target::Cell(at(4, 4)))
    );
    for seq in [
        Seq::Die1,
        Seq::Die2,
        Seq::Die3,
        Seq::Die4,
        Seq::Die5,
        Seq::WetDie1,
        Seq::WetDie2,
        Seq::AirDeathStart,
        Seq::AirDeathFall,
        Seq::AirDeathEnd,
    ] {
        w.obj_mut(i).unwrap().infantry.as_mut().unwrap().seq = Some(seq);
        assert_eq!(acquire::redirect(&w, a, Target::Object(i)), None, "{seq:?}");
    }
    w.obj_mut(i).unwrap().infantry.as_mut().unwrap().seq = Some(Seq::Stand);
    assert_eq!(acquire::redirect(&w, a, Target::Object(i)), obj(i));
    w.obj_mut(b).unwrap().health = 0;
    assert_eq!(acquire::redirect(&w, a, Target::Object(b)), None, "dead");
}

// ---------------------------------------------------------------------------
// Claim 8 and F1

// spec: S-0309 F1 (R = 0: n = trunc(w / 256) + 1 + trunc(b / 256); a type that is underwater, organic and self-healing uses g for w)
#[test]
fn underwater_organic_self_healing_type_uses_guard_range_for_rings() {
    let mut r = base();
    let cart = r.sim.vehicles.get_mut("CART").unwrap();
    cart.combat.underwater = Some(true);
    cart.combat.organic = Some(true);
    cart.combat.self_healing = Some(true);
    let mut w = game(&r);
    let a = veh(&mut w, "CART", 0, 0, 0);
    // Within the gun's 5.5 cells, but in ring 2: with g = 0, n = 0 + 1 + 0 = 1 looks at ring 0 only.
    veh(&mut w, "T0", 1, 2, 0);
    assert_eq!(range_search(&mut w, a), None);
    // Without the three flags, n = trunc(1408 / 256) + 1 = 6 reaches ring 2.
    let mut w = game(&base());
    let a = veh(&mut w, "CART", 0, 0, 0);
    let b = veh(&mut w, "T0", 1, 2, 0);
    assert_eq!(range_search(&mut w, a), Some(b));
}

// ---------------------------------------------------------------------------
// More helpers

/// Sets the special threat value of a vehicle type.
fn special(r: &mut Ruleset, ty: &str, v: &str) {
    r.sim
        .vehicles
        .get_mut(ty)
        .expect("vehicle type")
        .combat
        .special_threat_value = Some(d(v));
}

/// CART with the special-threat coefficient 1 and every other coefficient
/// 0, so that a candidate's converted score is 100000 + its special threat
/// value (F4).
fn by_special() -> Ruleset {
    let mut r = base();
    coefficients(&mut r, "CART", ["0", "0", "1", "0", "0"]);
    r
}

fn cart_mut(r: &mut Ruleset) -> &mut UnitType {
    r.sim.vehicles.get_mut("CART").expect("CART")
}

/// A game in which house 1 (Beta) has `role`.
fn game_role(r: &Ruleset, role: HouseRole) -> World {
    let mut s = testkit::setup(11);
    s.houses[1].role = role;
    glasnost_sim::load::new_game(
        Arc::new(SimRules::new(r, "claims-s0309")),
        &testkit::empty_map(SIZE),
        testkit::catalogue(),
        &s,
    )
    .expect("synthetic game loads")
}

fn timer(w: &World, a: ObjectId) -> i32 {
    w.obj(a).unwrap().targeting_timer
}

fn expire(w: &mut World, a: ObjectId) {
    let t = w.tick;
    w.obj_mut(a).unwrap().targeting_timer = t;
}

fn set_beh(w: &mut World, a: ObjectId, b: Behaviour) {
    let t = w.tick;
    w.obj_mut(a).unwrap().beh.force(b, t, true);
}

fn set_target(w: &mut World, a: ObjectId, t: Option<Target>, opp: bool) {
    let o = w.obj_mut(a).unwrap();
    o.target = t;
    o.combat.opportunity = opp;
}

fn kill(w: &mut World, x: ObjectId) {
    w.obj_mut(x).unwrap().health = 0;
}

// ---------------------------------------------------------------------------
// Claims 1 to 3: when searches run (and S-0215 claim 33)

// spec: S-0309 claim 1 (the per-tick range search in guard once the targeting timer has run out) and S-0215 claim 33
#[test]
fn per_tick_search_runs_in_guard_when_the_timer_has_run_out() {
    let mut w = game(&base());
    let a = veh(&mut w, "CART", 0, 0, 0);
    let b = veh(&mut w, "T0", 1, 2, 0);
    w.obj_mut(a).unwrap().targeting_timer = w.tick + 1;
    acquire::per_tick(&mut w, a);
    assert_eq!(target(&w, a), None, "timer not run out: no search");
    expire(&mut w, a);
    acquire::per_tick(&mut w, a);
    assert_eq!(target(&w, a), obj(b));
    assert!(opportunity(&w, a));
}

// spec: S-0215 claim 33 (sticky is not in the list of behaviours that run the per-tick search)
#[test]
fn per_tick_search_not_in_sticky() {
    let mut w = game(&base());
    let a = veh(&mut w, "CART", 0, 0, 0);
    veh(&mut w, "T0", 1, 2, 0);
    set_beh(&mut w, a, Behaviour::Sticky);
    expire(&mut w, a);
    let before = timer(&w, a);
    acquire::per_tick(&mut w, a);
    assert_eq!(target(&w, a), None);
    assert_eq!(timer(&w, a), before, "no search, no restart");
}

// spec: S-0309 claim 3 (without the opportunity-fire flag the per-tick search never runs in harvest, nor in move before arrival)
#[test]
fn opportunity_gate_move_and_harvest() {
    for (b, dest) in [
        (Behaviour::Move, Some(Target::Cell(at(-6, 0)))),
        (Behaviour::Harvest, None),
    ] {
        let mut w = game(&base());
        let a = veh(&mut w, "CART", 0, 0, 0);
        veh(&mut w, "T0", 1, 2, 0);
        set_beh(&mut w, a, b);
        w.obj_mut(a).unwrap().dest = dest;
        expire(&mut w, a);
        acquire::per_tick(&mut w, a);
        assert_eq!(target(&w, a), None, "{b:?} without the flag");
        let mut r = base();
        cart_mut(&mut r).opportunity_fire = Some(true);
        let mut w = game(&r);
        let a = veh(&mut w, "CART", 0, 0, 0);
        let t = veh(&mut w, "T0", 1, 2, 0);
        set_beh(&mut w, a, b);
        w.obj_mut(a).unwrap().dest = dest;
        expire(&mut w, a);
        acquire::per_tick(&mut w, a);
        assert_eq!(
            target(&w, a),
            obj(t),
            "{b:?} with the opportunity-fire flag"
        );
    }
}

// spec: S-0309 claim 3 (move: arrived on its own cell with the balloon-hover flag, or a vehicle that deploys in place)
#[test]
fn opportunity_gate_arrived_balloon_or_simple_deployer() {
    for flag in ["balloon", "simple", "none"] {
        let mut r = base();
        match flag {
            "balloon" => cart_mut(&mut r).balloon_hover = Some(true),
            "simple" => cart_mut(&mut r).is_simple_deployer = Some(true),
            _ => {}
        }
        let mut w = game(&r);
        let a = veh(&mut w, "CART", 0, 0, 0);
        let t = veh(&mut w, "T0", 1, 2, 0);
        set_beh(&mut w, a, Behaviour::Move);
        w.obj_mut(a).unwrap().dest = Some(Target::Cell(at(0, 0)));
        expire(&mut w, a);
        acquire::per_tick(&mut w, a);
        let want = if flag == "none" { None } else { obj(t) };
        assert_eq!(target(&w, a), want, "{flag}");
    }
}

// spec: S-0309 claim 3 (guard: not when the area-fire weapon of slot 0 of a spray-attack type is the one it would use with no target; the opportunity-fire flag lifts it)
#[test]
fn opportunity_gate_area_fire_in_guard() {
    for (spray, opp, slot1, runs) in [
        (true, false, false, false),
        (true, true, false, true),
        // Not a spray-attack type: the area-fire secondary is not the weapon of rows 1 to 3 with no target.
        (false, false, true, true),
    ] {
        let mut r = base();
        let gun = r.sim.weapons["Gun"].clone();
        r.sim.weapons.insert(
            "Area".into(),
            WeaponType {
                area_fire: Some(true),
                ..gun
            },
        );
        let c = cart_mut(&mut r);
        c.spray_attack = Some(spray);
        c.opportunity_fire = Some(opp);
        if slot1 {
            c.secondary = Some("Area".into());
        } else {
            c.primary = Some("Area".into());
        }
        let mut w = game(&r);
        let a = veh(&mut w, "CART", 0, 0, 0);
        let t = veh(&mut w, "T0", 1, 2, 0);
        expire(&mut w, a);
        acquire::per_tick(&mut w, a);
        let want = if runs { obj(t) } else { None };
        assert_eq!(target(&w, a), want, "spray {spray}, opportunity {opp}");
    }
}

// spec: S-0309 claim 2 (the passive-acquire flag; the map-wide search of hunt is not limited by it)
#[test]
fn passive_acquire_flag() {
    let mut r = base();
    cart_mut(&mut r).combat.can_passive_aquire = Some(false);
    let mut w = game(&r);
    let a = veh(&mut w, "CART", 0, 0, 0);
    let b = veh(&mut w, "T0", 1, 2, 0);
    assert!(!acquire::may_search_passively(&w, a));
    expire(&mut w, a);
    acquire::per_tick(&mut w, a);
    assert_eq!(target(&w, a), None);
    assert_eq!(
        acquire::search(&mut w, a, SearchKind::MapWide, SearchContext::Hunt),
        Some(b)
    );
}

// spec: S-0309 claim 2 (an unarmed object, an occupiable building without occupants and an engineer of a player house may not search passively)
#[test]
fn passive_acquisition_exclusions() {
    let mut r = base();
    r.sim.buildings.get_mut("TOWER").unwrap().can_be_occupied = Some(true);
    let wk = r.sim.infantry.get_mut("WALKER").unwrap();
    wk.engineer = Some(true);
    let mut w = game(&r);
    let unarmed = veh(&mut w, "T0", 0, 0, 0);
    let tower = put(&mut w, Kind::Building, "TOWER", 0, 3, 3);
    let eng = put(&mut w, Kind::Infantry, "WALKER", 0, -3, 3);
    let cart = veh(&mut w, "CART", 0, -3, -3);
    assert!(!acquire::may_search_passively(&w, unarmed));
    assert!(!acquire::may_search_passively(&w, tower));
    assert!(!acquire::may_search_passively(&w, eng));
    assert!(acquire::may_search_passively(&w, cart));
}

// ---------------------------------------------------------------------------
// Claim 4: the timer draw, keeping and dropping, expected remaining health

// spec: S-0309 claim 4 and Observable ordering (the timer draw N + U(0, 2) is the search's first and only draw for a player house) and S-0215 claim 33 (N: the normal targeting delay)
#[test]
fn search_takes_only_the_timer_draw() {
    let mut r = base();
    r.sim.general.normal_targeting_delay = Some(17);
    r.sim.general.guard_area_targeting_delay = Some(41);
    let mut w = game(&r);
    let a = veh(&mut w, "CART", 0, 0, 0);
    for i in 0..4 {
        veh(&mut w, "T0", 1, 2, i - 2);
    }
    let half = w.rules().quirks.ranged_draw_half_span;
    for _ in 0..3 {
        let mut expect = w.rng.clone();
        let u = expect.ranged(0, 2, half);
        set_target(&mut w, a, None, false);
        assert!(range_search(&mut w, a).is_some());
        assert_eq!(timer(&w, a), w.tick + 17 + u);
        assert_eq!(w.rng, expect, "no draw besides the timer draw");
    }
}

// spec: S-0215 claim 33 (in area guard every search restarts the timer with the area-guard targeting delay)
#[test]
fn search_in_area_guard_uses_the_area_guard_delay() {
    let mut r = base();
    r.sim.general.normal_targeting_delay = Some(17);
    r.sim.general.guard_area_targeting_delay = Some(41);
    let mut w = game(&r);
    let a = veh(&mut w, "CART", 0, 0, 0);
    set_beh(&mut w, a, Behaviour::AreaGuard);
    let half = w.rules().quirks.ranged_draw_half_span;
    let mut expect = w.rng.clone();
    let u = expect.ranged(0, 2, half);
    acquire::search(&mut w, a, SearchKind::Area, SearchContext::Area);
    assert_eq!(timer(&w, a), w.tick + 41 + u);
    assert_eq!(w.rng, expect);
}

// spec: S-0309 claim 4 and table Search outcome rows 7 and 8 (a usable opportunity target, or a target given by an order, is kept; no candidate is looked for; the timer still restarts)
#[test]
fn kept_targets() {
    let mut w = game(&by_special());
    let a = veh(&mut w, "CART", 0, 0, 0);
    let weak = veh(&mut w, "T0", 1, 3, 0);
    veh(&mut w, "T1", 1, 2, 0);
    // An order's target, even out of range, is kept.
    let far = veh(&mut w, "T2", 1, 12, 0);
    for (t, opp) in [(weak, true), (far, false), (weak, false)] {
        set_target(&mut w, a, obj(t), opp);
        let rng = w.rng.clone();
        assert_eq!(range_search(&mut w, a), None);
        assert_eq!(target(&w, a), obj(t));
        assert_eq!(opportunity(&w, a), opp, "the mark stays as it was");
        assert_ne!(w.rng, rng, "the timer draw is taken");
    }
}

// spec: S-0309 claim 19 and table Search outcome row 3 (an opportunity target out of range is dropped and the same search picks a new target, an opportunity target)
#[test]
fn opportunity_target_out_of_range_is_replaced() {
    let mut r = base();
    guard(&mut r, "CART", "8");
    let mut w = game(&r);
    let a = veh(&mut w, "CART", 0, 0, 0);
    let far = veh(&mut w, "T0", 1, 7, 0);
    let near = veh(&mut w, "T1", 1, 2, 0);
    set_target(&mut w, a, obj(far), true);
    assert_eq!(range_search(&mut w, a), Some(near));
    assert_eq!(target(&w, a), obj(near));
    assert!(opportunity(&w, a));
}

fn expected(w: &World, x: ObjectId) -> i32 {
    w.obj(x).unwrap().combat.expected_health
}

// spec: S-0309 claim 4 (a pick lowers the target's expected remaining health by the expected damage of S-0302 claim 18 at once)
#[test]
fn pick_lowers_expected_remaining_health() {
    let mut w = game(&base());
    let a = veh(&mut w, "CART", 0, 0, 0);
    let b = veh(&mut w, "T0", 1, 2, 0);
    let gun = weapon_of(&w, a);
    let e = glasnost_sim::combat::damage::expected_damage(&w, a, gun, b);
    assert!(e > 0);
    assert_eq!(expected(&w, b), 100);
    assert_eq!(range_search(&mut w, a), Some(b));
    assert_eq!(expected(&w, b), 100 - e);
}

fn weapon_of(w: &World, a: ObjectId) -> glasnost_sim::combat::data::WeaponId {
    glasnost_sim::combat::weapon::slot_weapon(w, a, 0).expect("a primary weapon")
}

// spec: S-0309 claim 4 and Import mapping (an inaccurate projectile lowers no expected remaining health)
#[test]
fn inaccurate_projectile_lowers_nothing() {
    let mut r = base();
    r.sim.projectiles.get_mut("Shell").unwrap().inaccurate = Some(true);
    let mut w = game(&r);
    let a = veh(&mut w, "CART", 0, 0, 0);
    let b = veh(&mut w, "T0", 1, 2, 0);
    assert_eq!(range_search(&mut w, a), Some(b));
    assert_eq!(expected(&w, b), 100);
}

// spec: S-0309 claim 4 and claim 20 (a distributed-fire searcher's pick lowers no expected remaining health)
#[test]
fn distributed_fire_pick_lowers_nothing() {
    let mut r = base();
    cart_mut(&mut r).distributed_fire = Some(true);
    let mut w = game(&r);
    let a = veh(&mut w, "CART", 0, 0, 0);
    let b = veh(&mut w, "T0", 1, 2, 0);
    assert_eq!(range_search(&mut w, a), Some(b));
    assert_eq!(expected(&w, b), 100);
}

// spec: S-0309 claim 4, claim 14 and Observable ordering (a searcher that searches later in the same tick sees the lowered value: mode strong rejects it)
#[test]
fn second_searcher_of_a_tick_sees_the_lowered_value() {
    let mut r = base();
    cart_mut(&mut r).combat.vhp_scan = Some("Strong".into());
    let mut w = game(&r);
    let a1 = veh(&mut w, "CART", 0, 0, 0);
    let a2 = veh(&mut w, "CART", 0, 0, 2);
    let b = veh(&mut w, "T0", 1, 2, 1);
    let e = glasnost_sim::combat::damage::expected_damage(&w, a1, weapon_of(&w, a1), b);
    w.obj_mut(b).unwrap().health = e;
    w.obj_mut(b).unwrap().combat.expected_health = e;
    assert_eq!(range_search(&mut w, a1), Some(b));
    assert_eq!(expected(&w, b), 0);
    assert_eq!(range_search(&mut w, a2), None, "strong scan: 0 or less");
}

// ---------------------------------------------------------------------------
// Claim 8 and F1: radius and rings

// spec: S-0309 claim 8 and F1 (area search: R = 2g around the guard spot) and F3 (the distance is measured from the searcher)
#[test]
fn area_search_centre_and_distance() {
    let mut r = base();
    guard(&mut r, "CART", "3"); // R = 1536, n = 6
    let mut w = game(&r);
    let a = veh(&mut w, "CART", 0, 0, 0);
    w.obj_mut(a).unwrap().guard_spot = Some(Target::Cell(at(8, 0)));
    // Ring 1 of the guard spot, but 9 cells (2304 leptons) from the searcher.
    let beyond = veh(&mut w, "T0", 1, 9, 0);
    // Next to the searcher, but in ring 9 of the guard spot.
    let behind = veh(&mut w, "T1", 1, -1, 0);
    let _ = (beyond, behind);
    assert_eq!(
        acquire::search(&mut w, a, SearchKind::Area, SearchContext::Area),
        None
    );
    // Ring 3 of the guard spot, 5 cells (1280 leptons) from the searcher.
    let ok = veh(&mut w, "T2", 1, 5, 0);
    assert_eq!(
        acquire::search(&mut w, a, SearchKind::Area, SearchContext::Area),
        Some(ok)
    );
}

// spec: S-0309 F1 (area search: R limited to 4096, n = trunc(R / 256) = 16)
#[test]
fn area_radius_is_capped() {
    let mut r = base();
    guard(&mut r, "CART", "20");
    let mut w = game(&r);
    let a = veh(&mut w, "CART", 0, 0, 0);
    w.obj_mut(a).unwrap().guard_spot = Some(Target::Cell(at(0, 0)));
    // Ring 16 at 4096 leptons: past the last ring 15.
    veh(&mut w, "T0", 1, 16, 0);
    assert_eq!(
        acquire::search(&mut w, a, SearchKind::Area, SearchContext::Area),
        None
    );
    let b = veh(&mut w, "T1", 1, -15, 0);
    assert_eq!(
        acquire::search(&mut w, a, SearchKind::Area, SearchContext::Area),
        Some(b)
    );
}

// spec: S-0309 F1 (area search with g = 0: G = max(w₀, w₁), R = 2G)
#[test]
fn area_radius_from_weapon_ranges() {
    // Gun 5.5 cells: R = 2816, n = 11.
    let mut w = game(&base());
    let a = veh(&mut w, "CART", 0, 0, 0);
    w.obj_mut(a).unwrap().guard_spot = Some(Target::Cell(at(0, 0)));
    veh(&mut w, "T0", 1, 11, 0);
    assert_eq!(
        acquire::search(&mut w, a, SearchKind::Area, SearchContext::Area),
        None,
        "ring 11 is past n − 1 = 10"
    );
    let b = veh(&mut w, "T1", 1, 10, 0);
    assert_eq!(
        acquire::search(&mut w, a, SearchKind::Area, SearchContext::Area),
        Some(b)
    );
}

// spec: S-0309 claim 8 and F3 (range search with R = 0: the candidate must be within the reach of the weapon it would use)
#[test]
fn range_search_without_guard_range_uses_weapon_reach() {
    // Gun 5.5 cells: n = 6, rings 0 … 5. Ring 5 diagonal (1810 leptons) is beyond the reach of 1408.
    let mut w = game(&base());
    let a = veh(&mut w, "CART", 0, 0, 0);
    veh(&mut w, "T1", 1, 5, 5);
    assert_eq!(range_search(&mut w, a), None);
    let b = veh(&mut w, "T0", 1, 5, 0);
    assert_eq!(
        range_search(&mut w, a),
        Some(b),
        "1280 leptons, within reach"
    );
}

// spec: S-0309 claim 9 (a radius of 1 to 255 leptons gives n = 0: no cell is looked at)
#[test]
fn radius_below_one_cell_looks_at_no_cell() {
    let mut r = base();
    r.sim.infantry.get_mut("WALKER").unwrap().guard_range = Some(d("0.5"));
    let mut w = game(&r);
    let a = put(&mut w, Kind::Infantry, "WALKER", 0, 0, 0);
    // An enemy in the searcher's own cell (ring 0).
    put(&mut w, Kind::Infantry, "WALKER", 1, 0, 0);
    assert_eq!(range_search(&mut w, a), None);
    let mut r = base();
    r.sim.infantry.get_mut("WALKER").unwrap().guard_range = Some(d("1"));
    let mut w = game(&r);
    let a = put(&mut w, Kind::Infantry, "WALKER", 0, 0, 0);
    let b = put(&mut w, Kind::Infantry, "WALKER", 1, 0, 0);
    assert_eq!(range_search(&mut w, a), Some(b), "n = 1: ring 0");
}

// ---------------------------------------------------------------------------
// Claim 9 and F2: ring order and early stops; claim 17: ties

/// The cells of ring `r` around (0, 0) in the order of F2.
fn f2(r: i32) -> Vec<(i32, i32)> {
    let mut v = Vec::new();
    for x in -r..=r {
        v.push((x, -r));
        v.push((x, r));
    }
    for y in -r + 1..r {
        v.push((-r, y));
        v.push((r, y));
    }
    v
}

// spec: S-0309 F2 and claim 17 (the cells of a ring in F2's order; equal scores: the candidate considered first wins)
#[test]
fn ring_cells_are_scanned_in_f2_order() {
    let mut r = base();
    guard(&mut r, "CART", "4");
    for ring in [1, 2] {
        let mut w = game(&r);
        let a = veh(&mut w, "CART", 0, 0, 0);
        let cells = f2(ring);
        // Place the targets in reverse order, so that creation order cannot explain the picks.
        let mut ids: Vec<(ObjectId, (i32, i32))> = cells
            .iter()
            .rev()
            .map(|&(x, y)| (veh(&mut w, "T0", 1, x, y), (x, y)))
            .collect();
        ids.reverse();
        for &(id, c) in &ids {
            set_target(&mut w, a, None, false);
            assert_eq!(range_search(&mut w, a), Some(id), "ring {ring} cell {c:?}");
            kill(&mut w, id);
        }
    }
}

// spec: S-0309 claim 9 (early stop: with n = 4, r₁ = 1; a candidate in ring 1 ends the search, so a better one in ring 2 is never reached)
#[test]
fn early_stop_at_ring_r1() {
    let mut r = by_special();
    guard(&mut r, "CART", "4");
    special(&mut r, "T9", "50");
    let mut w = game(&r);
    let a = veh(&mut w, "CART", 0, 0, 0);
    let inner = veh(&mut w, "T0", 1, 1, 0);
    veh(&mut w, "T9", 1, -2, 0);
    assert_eq!(range_search(&mut w, a), Some(inner));
}

// spec: S-0309 claim 9 (early stop at r₂ = 2 when rings 0 … 1 hold no candidate; ring n − 1 = 3 otherwise)
#[test]
fn early_stop_at_ring_r2_and_last_ring() {
    let mut r = by_special();
    guard(&mut r, "CART", "4");
    special(&mut r, "T9", "50");
    let mut w = game(&r);
    let a = veh(&mut w, "CART", 0, 0, 0);
    let ring2 = veh(&mut w, "T0", 1, 2, 0);
    let ring3 = veh(&mut w, "T9", 1, -3, 0);
    assert_eq!(range_search(&mut w, a), Some(ring2));
    kill(&mut w, ring2);
    set_target(&mut w, a, None, false);
    assert_eq!(range_search(&mut w, a), Some(ring3));
}

// spec: S-0309 claim 9 and claim 17 (with n = 8, r₁ = 2 and r₂ = 4: candidates of rings 3 and 4 compete on score; ring 5 is not reached)
#[test]
fn candidates_up_to_the_stop_ring_compete_on_score() {
    let mut r = by_special();
    guard(&mut r, "CART", "8");
    special(&mut r, "T8", "20");
    special(&mut r, "T9", "50");
    let mut w = game(&r);
    let a = veh(&mut w, "CART", 0, 0, 0);
    veh(&mut w, "T0", 1, 3, 0);
    let better = veh(&mut w, "T8", 1, 0, 4);
    veh(&mut w, "T9", 1, -5, 0);
    assert_eq!(range_search(&mut w, a), Some(better));
}

// spec: S-0309 claim 9 and F2 (rings 0 … n − 1, cells that are not legal left out: a guard range far beyond the map still reaches the legal cell farthest from the centre, #401)
#[test]
fn a_range_far_beyond_the_map_reaches_its_farthest_legal_cell() {
    let mut r = base();
    guard(&mut r, "CART", "100000");
    let mut w = game(&r);
    // From (28, 30) the legal cell (59, 30) is 31 rings out, at the far
    // edge of the coordinate square 0 … 59.
    let a = veh(&mut w, "CART", 0, -2, 0);
    let far = veh(&mut w, "T0", 1, 29, 0);
    assert_eq!(range_search(&mut w, a), Some(far));
}

// ---------------------------------------------------------------------------
// Claim 10: one object per cell

fn ghost(r: &mut Ruleset) {
    let mut g = r.sim.infantry["WALKER"].clone();
    g.legal_target = Some(false);
    r.sim.infantry.insert("GHOST".into(), g);
    r.registries
        .get_mut("infantry")
        .unwrap()
        .push("GHOST".into());
}

// spec: S-0309 claim 10 (a cell offers only its first selected object; when it fails a test the cell gives no candidate) and Edge cases (quirks.search_one_object_per_cell on)
#[test]
fn one_object_per_cell() {
    let mut r = base();
    ghost(&mut r);
    let mut w = game(&r);
    let a = veh(&mut w, "CART", 0, 0, 0);
    put(&mut w, Kind::Infantry, "WALKER", 1, 2, 0);
    // Entered later: listed first (S-0209 claim 13), so offered.
    put(&mut w, Kind::Infantry, "GHOST", 1, 2, 0);
    assert_eq!(range_search(&mut w, a), None);
}

// spec: S-0309 Edge cases (quirks.search_one_object_per_cell off: every object of the cell is tested)
#[test]
fn one_object_per_cell_off() {
    let mut r = base();
    ghost(&mut r);
    r.globals.quirks.search_one_object_per_cell = false;
    let mut w = game(&r);
    let a = veh(&mut w, "CART", 0, 0, 0);
    let b = put(&mut w, Kind::Infantry, "WALKER", 1, 2, 0);
    put(&mut w, Kind::Infantry, "GHOST", 1, 2, 0);
    assert_eq!(range_search(&mut w, a), Some(b));
}

// spec: S-0309 claim 10 (any other searcher selects the first object not allied with it; an allied first object does not hide it)
#[test]
fn allied_first_object_is_passed_over() {
    let mut w = game(&base());
    let a = veh(&mut w, "CART", 0, 0, 0);
    let b = put(&mut w, Kind::Infantry, "WALKER", 1, 2, 0);
    put(&mut w, Kind::Infantry, "WALKER", 0, 2, 0);
    assert_eq!(range_search(&mut w, a), Some(b));
}

// spec: S-0309 claim 10 (a searcher whose type attacks friendlies selects any object: an allied first object is tested and can be picked)
#[test]
fn attack_friendlies_selects_any_object() {
    let mut r = base();
    cart_mut(&mut r).attack_friendlies = Some(true);
    let mut w = game(&r);
    let a = veh(&mut w, "CART", 0, 0, 0);
    let ally = veh(&mut w, "T0", 0, 2, 0);
    assert_eq!(range_search(&mut w, a), Some(ally));
}

// ---------------------------------------------------------------------------
// Claim 5: search classes

fn classes(w: &World, a: ObjectId) -> (bool, bool, bool, bool, bool) {
    let c = acquire::classes(w, a);
    (c.infantry, c.vehicles, c.aircraft, c.buildings, c.friendly)
}

const GROUND: (bool, bool, bool, bool, bool) = (true, true, false, true, false);
const NONE: (bool, bool, bool, bool, bool) = (false, false, false, false, false);

/// Adds projectile `AAOnly` (anti-air, not anti-ground), weapon `Flak` with
/// it, warhead `InfOnly` (0 against medium and wood) and weapon `Needle`
/// with it.
fn special_weapons(r: &mut Ruleset) {
    use glasnost_rules::sim::ProjectileType;
    r.sim.projectiles.insert(
        "AAOnly".into(),
        ProjectileType {
            aa: Some(true),
            ag: Some(false),
            inviso: Some(true),
            ..Default::default()
        },
    );
    let gun = r.sim.weapons["Gun"].clone();
    r.sim.weapons.insert(
        "Flak".into(),
        WeaponType {
            projectile: Some("AAOnly".into()),
            ..gun.clone()
        },
    );
    let mut v = vec![d("1"); 11];
    v[4] = d("0");
    v[6] = d("0");
    r.sim.warheads.insert(
        "InfOnly".into(),
        WarheadType {
            verses: Some(v),
            ..Default::default()
        },
    );
    r.sim.weapons.insert(
        "Needle".into(),
        WeaponType {
            warhead: Some("InfOnly".into()),
            ..gun
        },
    );
}

// spec: S-0309 claim 5 (classes from the weapons: anti-ground gives infantry, vehicles and buildings; anti-air gives aircraft)
#[test]
fn classes_from_weapons() {
    let mut r = base();
    special_weapons(&mut r);
    r.sim.vehicles.get_mut("T1").unwrap().primary = Some("Flak".into());
    r.sim.vehicles.get_mut("T2").unwrap().primary = Some("Flak".into());
    r.sim.vehicles.get_mut("T2").unwrap().secondary = Some("Gun".into());
    let mut w = game(&r);
    let a = veh(&mut w, "CART", 0, 0, 0);
    let aa = veh(&mut w, "T1", 0, 2, 0);
    let both = veh(&mut w, "T2", 0, 4, 0);
    assert_eq!(classes(&w, a), GROUND);
    assert_eq!(classes(&w, aa), (false, false, true, false, false));
    assert_eq!(classes(&w, both), (true, true, true, true, false));
}

// spec: S-0309 claim 5 and table C row class (a search for aircraft only finds no ground vehicle)
#[test]
fn class_row_rejects_other_kinds() {
    let mut r = base();
    special_weapons(&mut r);
    cart_mut(&mut r).primary = Some("Flak".into());
    let mut w = game(&r);
    let a = veh(&mut w, "CART", 0, 0, 0);
    w.obj_mut(a).unwrap().guard_spot = Some(Target::Cell(at(0, 0)));
    veh(&mut w, "T0", 1, 2, 0);
    // An area search looks at the rings, so the cell offers the vehicle, which the class row rejects.
    assert_eq!(
        acquire::search(&mut w, a, SearchKind::Area, SearchContext::Area),
        None
    );
}

// spec: S-0309 claim 5 (a player's vehicle whose type deploys to fire, and a player's object whose type has the no-auto-fire flag, never search)
#[test]
fn deploy_to_fire_and_no_auto_fire_never_search_for_players() {
    let mut r = base();
    r.sim.vehicles.get_mut("T1").unwrap().primary = Some("Gun".into());
    r.sim.vehicles.get_mut("T1").unwrap().deploy_to_fire = Some(true);
    r.sim.vehicles.get_mut("T2").unwrap().primary = Some("Gun".into());
    r.sim.vehicles.get_mut("T2").unwrap().combat.no_auto_fire = Some(true);
    let mut w = game(&r);
    let dtf = veh(&mut w, "T1", 0, 0, 0);
    let naf = veh(&mut w, "T2", 0, 2, 0);
    assert_eq!(classes(&w, dtf), NONE);
    assert_eq!(classes(&w, naf), NONE);
}

// spec: S-0309 claim 25 part b (for the neutral and special houses, a deploy-to-fire vehicle and a no-auto-fire object search like any other)
#[test]
fn neutral_deploy_to_fire_and_no_auto_fire_search() {
    let mut r = base();
    r.sim.vehicles.get_mut("T1").unwrap().primary = Some("Gun".into());
    r.sim.vehicles.get_mut("T1").unwrap().deploy_to_fire = Some(true);
    r.sim.vehicles.get_mut("T2").unwrap().primary = Some("Gun".into());
    r.sim.vehicles.get_mut("T2").unwrap().combat.no_auto_fire = Some(true);
    for role in [HouseRole::Neutral, HouseRole::Special] {
        let mut w = game_role(&r, role);
        let dtf = veh(&mut w, "T1", 1, 0, 0);
        let naf = veh(&mut w, "T2", 1, 2, 0);
        assert_eq!(classes(&w, dtf), GROUND);
        assert_eq!(classes(&w, naf), GROUND);
    }
}

// spec: S-0309 claim 5 (unarmed infantry finds nothing; an unarmed vehicle thief searches for vehicles only)
#[test]
fn unarmed_infantry_classes() {
    let mut r = base();
    let mut thief = r.sim.infantry["WALKER"].clone();
    thief.primary = None;
    thief.vehicle_thief = Some(true);
    let mut plain = thief.clone();
    plain.vehicle_thief = None;
    r.sim.infantry.insert("THIEF".into(), thief);
    r.sim.infantry.insert("PLAIN".into(), plain);
    r.registries
        .get_mut("infantry")
        .unwrap()
        .extend(["THIEF".to_string(), "PLAIN".to_string()]);
    let mut w = game(&r);
    let t = put(&mut w, Kind::Infantry, "THIEF", 0, 0, 0);
    let p = put(&mut w, Kind::Infantry, "PLAIN", 0, 2, 0);
    assert_eq!(classes(&w, t), (false, true, false, false, false));
    assert_eq!(classes(&w, p), NONE);
}

// spec: S-0309 claim 5 (an armed infantry unit whose primary warhead is 0 against medium and wood searches for infantry only)
#[test]
fn infantry_only_warhead() {
    let mut r = base();
    special_weapons(&mut r);
    r.sim.infantry.get_mut("WALKER").unwrap().primary = Some("Needle".into());
    let mut w = game(&r);
    let a = put(&mut w, Kind::Infantry, "WALKER", 0, 0, 0);
    assert_eq!(classes(&w, a), (true, false, false, false, false));
}

// spec: S-0309 claim 5 (an engineer never searches for infantry or vehicles; the combination table: an armed engineer with an infantry-only warhead finds nothing; an unarmed engineer finds nothing)
#[test]
fn engineer_classes() {
    let mut r = base();
    special_weapons(&mut r);
    let mut eng = r.sim.infantry["WALKER"].clone();
    eng.engineer = Some(true);
    let mut needle = eng.clone();
    needle.primary = Some("Needle".into());
    let mut bare = eng.clone();
    bare.primary = None;
    r.sim.infantry.insert("ENG".into(), eng);
    r.sim.infantry.insert("ENGN".into(), needle);
    r.sim.infantry.insert("ENGU".into(), bare);
    r.registries.get_mut("infantry").unwrap().extend([
        "ENG".to_string(),
        "ENGN".to_string(),
        "ENGU".to_string(),
    ]);
    let mut w = game(&r);
    let e = put(&mut w, Kind::Infantry, "ENG", 0, 0, 0);
    let n = put(&mut w, Kind::Infantry, "ENGN", 0, 2, 0);
    let u = put(&mut w, Kind::Infantry, "ENGU", 0, 4, 0);
    assert_eq!(classes(&w, e), (false, false, false, true, false));
    assert_eq!(classes(&w, n), NONE);
    assert_eq!(classes(&w, u), NONE);
}

// spec: S-0309 claim 5 (a building always searches with the range kind, whatever the context asks)
#[test]
fn building_searches_with_the_range_kind() {
    let mut w = game(&base());
    let tower = put(&mut w, Kind::Building, "TOWER", 0, 0, 0);
    w.obj_mut(tower).unwrap().guard_spot = Some(Target::Cell(at(-12, 0)));
    let b = veh(&mut w, "T0", 1, 3, 0);
    assert_eq!(
        acquire::search(&mut w, tower, SearchKind::Area, SearchContext::Area),
        Some(b),
        "centred on the tower, not on the guard spot"
    );
}

// ---------------------------------------------------------------------------
// Claim 6: healers

/// Infantry `MEDIC` (primary −5, secondary 3: M = −1) and vehicle `FIXER`
/// (the same weapons).
fn healers(p: i32, s: i32) -> Ruleset {
    let mut r = base();
    let half = r.sim.weapons["Half"].clone();
    r.sim.weapons.insert(
        "Mend".into(),
        WeaponType {
            damage: Some(p),
            ..half.clone()
        },
    );
    r.sim.weapons.insert(
        "Poke".into(),
        WeaponType {
            damage: Some(s),
            ..half
        },
    );
    let mut medic = r.sim.infantry["WALKER"].clone();
    medic.primary = Some("Mend".into());
    medic.secondary = Some("Poke".into());
    r.sim.infantry.insert("MEDIC".into(), medic);
    r.registries
        .get_mut("infantry")
        .unwrap()
        .push("MEDIC".into());
    let mut fixer = r.sim.vehicles["CART"].clone();
    fixer.primary = Some("Mend".into());
    fixer.secondary = Some("Poke".into());
    r.sim.vehicles.insert("FIXER".into(), fixer);
    r.registries
        .get_mut("vehicles")
        .unwrap()
        .push("FIXER".into());
    r
}

fn hurt(w: &mut World, x: ObjectId, health: i32) {
    let o = w.obj_mut(x).unwrap();
    o.health = health;
    o.combat.expected_health = health;
}

// spec: S-0309 claim 6 (a healer for acquisition searches for infantry (a vehicle: vehicles) with the friendly modifier)
#[test]
fn healer_classes() {
    let mut w = game(&healers(-5, 3));
    let m = put(&mut w, Kind::Infantry, "MEDIC", 0, 0, 0);
    let f = veh(&mut w, "FIXER", 0, 3, 3);
    assert_eq!(classes(&w, m), (true, false, false, false, true));
    assert_eq!(classes(&w, f), (false, true, false, false, true));
}

// spec: S-0309 claim 6, table C rows friendly and healer, and S-0308 scenario W10 (a healer picks a damaged ally, not one at full health, not a damaged enemy)
#[test]
fn healer_picks_damaged_ally() {
    let mut w = game(&healers(-5, 3));
    let m = put(&mut w, Kind::Infantry, "MEDIC", 0, 0, 0);
    let full = put(&mut w, Kind::Infantry, "WALKER", 0, 1, 0);
    let enemy = put(&mut w, Kind::Infantry, "WALKER", 1, 0, 1);
    hurt(&mut w, enemy, 40);
    assert_eq!(range_search(&mut w, m), None);
    let _ = full;
    let ally = put(&mut w, Kind::Infantry, "WALKER", 0, -1, 0);
    hurt(&mut w, ally, 40);
    assert_eq!(range_search(&mut w, m), Some(ally));
}

// spec: S-0308 claim 15 and scenario W10 (M = trunc(0 / 2) = 0 is no healer: it does not pick the damaged ally)
#[test]
fn zero_mean_damage_is_no_healer() {
    let mut w = game(&healers(-3, 3));
    let m = put(&mut w, Kind::Infantry, "MEDIC", 0, 0, 0);
    let ally = put(&mut w, Kind::Infantry, "WALKER", 0, 1, 0);
    hurt(&mut w, ally, 40);
    assert_eq!(range_search(&mut w, m), None);
}

// spec: S-0309 claim 6 and F1 (a healer for acquisition in guard uses R = 512, n = 2)
#[test]
fn healer_guard_radius() {
    let mut w = game(&healers(-5, 3));
    let m = put(&mut w, Kind::Infantry, "MEDIC", 0, 0, 0);
    let far = put(&mut w, Kind::Infantry, "WALKER", 0, 2, 0);
    hurt(&mut w, far, 40);
    assert_eq!(range_search(&mut w, m), None, "ring 2 is past n − 1 = 1");
    let near = put(&mut w, Kind::Infantry, "WALKER", 0, 1, 1);
    hurt(&mut w, near, 40);
    assert_eq!(range_search(&mut w, m), Some(near));
}

// ---------------------------------------------------------------------------
// Claim 7: the map-wide search

// spec: S-0309 claim 7 (no distance limit; objects allied with the searcher are not considered)
#[test]
fn map_wide_search_has_no_distance_limit() {
    let mut w = game(&base());
    let a = veh(&mut w, "CART", 0, 0, 0);
    veh(&mut w, "T0", 0, 2, 0);
    let far = veh(&mut w, "T1", 1, 14, 0);
    assert_eq!(
        acquire::search(&mut w, a, SearchKind::MapWide, SearchContext::Hunt),
        Some(far)
    );
    assert!(
        !opportunity(&w, a),
        "a map-wide search marks no opportunity target"
    );
}

/// A searcher whose score of a candidate 7 cells away sums to exactly 0 in
/// a map-wide search when the distance is in leptons (F4): c_st = 1,
/// c_d = 1, k = trunc(1408 / 256) = 5, d = 1792, s_T = −(100000 + 1787).
fn lepton_hunt(quirk: bool) -> Option<ObjectId> {
    let mut r = base();
    coefficients(&mut r, "CART", ["0", "0", "1", "0", "1"]);
    special(&mut r, "T0", "-101787");
    r.globals.quirks.hunt_distance_in_leptons = quirk;
    let mut w = game(&r);
    let a = veh(&mut w, "CART", 0, 0, 0);
    veh(&mut w, "T0", 1, 7, 0);
    acquire::search(&mut w, a, SearchKind::MapWide, SearchContext::Hunt)
}

// spec: S-0309 F4 and Edge cases (quirks.hunt_distance_in_leptons on: d in leptons for ground-layer objects of the map-wide search)
#[test]
fn hunt_distance_in_leptons_on() {
    assert_eq!(lepton_hunt(true), None, "the sum is exactly 0: rejected");
}

// spec: S-0309 Edge cases (quirks.hunt_distance_in_leptons off: d in cells as in the other searches)
#[test]
fn hunt_distance_in_leptons_off() {
    // d = 7 cells, e = 2: the sum is negative and counts as 1.
    assert!(lepton_hunt(false).is_some());
}

// ---------------------------------------------------------------------------
// Claim 12: table C rows

/// A CART searcher (guard range 4) at the centre and one target of `ty`
/// (house 1) two cells east.
fn one_target(r: &Ruleset, kind: Kind, ty: &str) -> (World, ObjectId, ObjectId) {
    let mut r = r.clone();
    guard(&mut r, "CART", "4");
    let mut w = game(&r);
    let a = veh(&mut w, "CART", 0, 0, 0);
    let b = put(&mut w, kind, ty, 1, 2, 0);
    (w, a, b)
}

fn found(r: &Ruleset, kind: Kind, ty: &str) -> bool {
    let (mut w, a, b) = one_target(r, kind, ty);
    let got = range_search(&mut w, a);
    assert!(got.is_none() || got == Some(b));
    got.is_some()
}

// spec: S-0309 table C row legal target
#[test]
fn table_c_legal_target() {
    let mut r = base();
    assert!(found(&r, Kind::Vehicle, "T1"));
    r.sim.vehicles.get_mut("T1").unwrap().legal_target = Some(false);
    assert!(!found(&r, Kind::Vehicle, "T1"));
}

// spec: S-0309 table C row versus and S-0304 claim 4 (a versus value at most τ_a rejects)
#[test]
fn table_c_versus() {
    for (v, ok) in [("0.01", false), ("0.05", true)] {
        let mut r = base();
        r.sim.warheads.insert(
            "Thin".into(),
            WarheadType {
                verses: Some(vec![d(v); 11]),
                ..Default::default()
            },
        );
        r.sim.weapons.get_mut("Gun").unwrap().warhead = Some("Thin".into());
        assert_eq!(found(&r, Kind::Vehicle, "T1"), ok, "versus {v}");
    }
}

// spec: S-0309 table C row land targeting (land-targeting mode 2 rejects a floor-bound target on a cell that is not water)
#[test]
fn table_c_land_targeting() {
    let mut r = base();
    cart_mut(&mut r).combat.land_targeting = Some(2);
    assert!(!found(&r, Kind::Vehicle, "T1"));
    let (mut w, a, b) = one_target(&r, Kind::Vehicle, "T1");
    let c = w.obj(b).unwrap().pos.cell();
    w.cells.grid.get_mut(c).unwrap().land = glasnost_map::terrain::land::WATER;
    assert_eq!(range_search(&mut w, a), Some(b), "on water");
}

// spec: S-0309 table C row no-threat (the target's current behaviour has the no-threat property)
#[test]
fn table_c_no_threat() {
    let mut r = base();
    r.sim.behaviours.insert(
        "sleep".into(),
        glasnost_rules::sim::BehaviourProps {
            no_threat: Some(true),
            ..Default::default()
        },
    );
    let (mut w, a, b) = one_target(&r, Kind::Vehicle, "T1");
    set_beh(&mut w, b, Behaviour::Sleep);
    assert_eq!(range_search(&mut w, a), None);
    set_beh(&mut w, b, Behaviour::Guard);
    assert_eq!(range_search(&mut w, a), Some(b));
}

// spec: S-0309 table C row underground and Constants (lowest searchable height −20 leptons: −20 passes, −21 is rejected)
#[test]
fn table_c_underground() {
    let r = base();
    let (mut w, a, b) = one_target(&r, Kind::Vehicle, "T1");
    let z = w.obj(b).unwrap().pos.z;
    w.obj_mut(b).unwrap().pos.z = z - 21;
    assert_eq!(range_search(&mut w, a), None);
    w.obj_mut(b).unwrap().pos.z = z - 20;
    assert_eq!(range_search(&mut w, a), Some(b));
}

/// An enemy building type `SHED` (light armour, 1 × 1) with weapon `weapon`
/// and threat posed `threat`.
fn shed(r: &mut Ruleset, weapon: Option<&str>, threat: i32, invisible: bool) {
    let mut b = r.sim.buildings["TOWER"].clone();
    b.primary = weapon.map(str::to_string);
    b.combat.threat_posed = Some(threat);
    b.combat.armor = Some(ArmourClass::Light);
    b.invisible_in_game = Some(invisible);
    r.sim.buildings.insert("SHED".into(), b);
    let reg = r.registries.get_mut("buildings").unwrap();
    if !reg.iter().any(|n| n == "SHED") {
        reg.push("SHED".into());
    }
}

// spec: S-0309 table C row invisible building
#[test]
fn table_c_invisible_building() {
    let mut r = base();
    shed(&mut r, Some("Gun"), 5, false);
    assert!(found(&r, Kind::Building, "SHED"));
    shed(&mut r, Some("Gun"), 5, true);
    assert!(!found(&r, Kind::Building, "SHED"));
}

// spec: S-0309 table C row harmless building (a player's searcher rejects a building with no current weapon or a threat value of 0; not for an engineer)
#[test]
fn table_c_harmless_building() {
    let mut r = base();
    shed(&mut r, None, 5, false);
    assert!(!found(&r, Kind::Building, "SHED"), "no weapon");
    shed(&mut r, Some("Gun"), 0, false);
    assert!(!found(&r, Kind::Building, "SHED"), "threat value 0");
    shed(&mut r, Some("Gun"), 5, false);
    assert!(found(&r, Kind::Building, "SHED"));
    // An engineer is not rejected by this row.
    let mut r = base();
    shed(&mut r, None, 0, false);
    r.sim.infantry.get_mut("WALKER").unwrap().engineer = Some(true);
    r.sim.infantry.get_mut("WALKER").unwrap().guard_range = Some(d("4"));
    let mut w = game(&r);
    let e = put(&mut w, Kind::Infantry, "WALKER", 0, 0, 0);
    let b = put(&mut w, Kind::Building, "SHED", 1, 2, 0);
    assert_eq!(range_search(&mut w, e), Some(b));
}

// spec: S-0309 claim 25 part f (row harmless building does not apply to a searcher of the neutral or special house)
#[test]
fn neutral_searcher_skips_harmless_building_row() {
    let mut r = base();
    shed(&mut r, None, 0, false);
    guard(&mut r, "CART", "4");
    let mut w = game_role(&r, HouseRole::Neutral);
    let a = veh(&mut w, "CART", 1, 0, 0);
    let b = put(&mut w, Kind::Building, "SHED", 0, 2, 0);
    assert_eq!(range_search(&mut w, a), Some(b));
}

// spec: S-0309 table C row passive house (a player's searcher rejects an object of a multiplayer-passive house)
#[test]
fn table_c_passive_house() {
    let mut r = base();
    r.sim
        .countries
        .get_mut("Betaland")
        .unwrap()
        .multiplay_passive = Some(true);
    assert!(!found(&r, Kind::Vehicle, "T1"));
}

// spec: S-0309 table C row insignificant (rejected unless it is a building of an owner that is not passive)
#[test]
fn table_c_insignificant() {
    let mut r = base();
    r.sim.vehicles.get_mut("T1").unwrap().combat.insignificant = Some(true);
    assert!(!found(&r, Kind::Vehicle, "T1"));
    shed(&mut r, Some("Gun"), 5, false);
    r.sim
        .buildings
        .get_mut("SHED")
        .unwrap()
        .combat
        .insignificant = Some(true);
    assert!(found(&r, Kind::Building, "SHED"));
}

// spec: S-0309 table C row train (a vehicle thief rejects a train)
#[test]
fn table_c_train() {
    let mut r = base();
    let wk = r.sim.infantry.get_mut("WALKER").unwrap();
    wk.vehicle_thief = Some(true);
    wk.guard_range = Some(d("4"));
    r.sim.vehicles.get_mut("T1").unwrap().is_train = Some(true);
    let mut w = game(&r);
    let a = put(&mut w, Kind::Infantry, "WALKER", 0, 0, 0);
    veh(&mut w, "T1", 1, 2, 0);
    assert_eq!(range_search(&mut w, a), None);
    let b = veh(&mut w, "T2", 1, -2, 0);
    assert_eq!(range_search(&mut w, a), Some(b));
}

// ---------------------------------------------------------------------------
// Claims 13 and 14, F4 to F6, F9: the threat score

/// The pick of a CART with coefficients `c` (guard range 8) against one
/// T0 seven cells east (d = 7, k = 5, e = 2, health ratio 1) whose special
/// threat value is `s`.
fn score_probe(c: [&str; 5], s: &str, tweak: impl FnOnce(&mut Ruleset)) -> Option<ObjectId> {
    let mut r = base();
    guard(&mut r, "CART", "8");
    coefficients(&mut r, "CART", c);
    special(&mut r, "T0", s);
    tweak(&mut r);
    let mut w = game(&r);
    let a = veh(&mut w, "CART", 0, 0, 0);
    veh(&mut w, "T0", 1, 7, 0);
    range_search(&mut w, a)
}

// spec: S-0309 claim 13, F4 and F9 (c_d × e + ((P + c_me × v) + c_sg × h) + 100000 with d in cells; a final score of exactly 0 rejects, quirks.zero_score_rejected on)
#[test]
fn threat_score_distance_strength_and_special_terms() {
    // 1 × 2 + ((−100003) + 1 × 1) + 100000 = 0.
    assert_eq!(
        score_probe(["0", "0", "1", "1", "1"], "-100003", |_| {}),
        None
    );
    // One more: 1.
    assert!(score_probe(["0", "0", "1", "1", "1"], "-100002", |_| {}).is_some());
}

// spec: S-0309 F9 and Edge cases (quirks.zero_score_rejected off: a score of 0 counts as 1)
#[test]
fn zero_score_rejected_off() {
    let got = score_probe(["0", "0", "1", "1", "1"], "-100003", |r| {
        r.globals.quirks.zero_score_rejected = false;
    });
    assert!(got.is_some());
}

// spec: S-0309 F9 (a negative final score counts as 1)
#[test]
fn negative_score_counts_as_one() {
    assert!(score_probe(["0", "0", "1", "0", "0"], "-200000", |_| {}).is_some());
}

// spec: S-0309 F4 (the own-effectiveness term c_me × v(A → T))
#[test]
fn threat_score_own_effectiveness_term() {
    // CART fires Half (versus 0.5, range 5 cells: k = 5, e = 2): 2 × 0.5 = 1.
    let probe = |s: &str| {
        score_probe(["2", "0", "1", "0", "0"], s, |r| {
            cart_mut(r).primary = Some("Half".into());
        })
    };
    assert_eq!(probe("-100001"), None);
    assert!(probe("-100000").is_some());
}

// spec: S-0309 F4 (the target-effectiveness term t = c_te × v(T → A), negated when T's target is A)
#[test]
fn threat_score_target_effectiveness_term() {
    let mut r = base();
    guard(&mut r, "CART", "8");
    coefficients(&mut r, "CART", ["0", "2", "1", "0", "0"]);
    special(&mut r, "T0", "-100001");
    // T0 fires Half: v(T → A) = 0.5 against the light CART; 2 × 0.5 = 1.
    r.sim.vehicles.get_mut("T0").unwrap().primary = Some("Half".into());
    let mut w = game(&r);
    let a = veh(&mut w, "CART", 0, 0, 0);
    let b = veh(&mut w, "T0", 1, 7, 0);
    assert_eq!(range_search(&mut w, a), None, "1 − 100001 + 100000 = 0");
    w.obj_mut(b).unwrap().target = obj(a);
    // −1 − 100001 + 100000 < 0: counts as 1.
    assert_eq!(range_search(&mut w, a), Some(b));
}

// spec: S-0309 claim 13 and F4 (the enemy-house bonus B when the searcher's owner's enemy house owns T)
#[test]
fn threat_score_enemy_house_bonus() {
    let mut r = base();
    guard(&mut r, "CART", "8");
    coefficients(&mut r, "CART", ["0", "0", "1", "0", "0"]);
    special(&mut r, "T0", "-100003");
    r.sim.general.enemy_house_threat_bonus = Some(d("3"));
    let mut w = game(&r);
    let a = veh(&mut w, "CART", 0, 0, 0);
    let b = veh(&mut w, "T0", 1, 7, 0);
    w.houses[0].combat.enemy = Some(glasnost_sim::world::HouseId(1));
    assert_eq!(range_search(&mut w, a), None, "3 − 100003 + 100000 = 0");
    w.houses[0].combat.enemy = None;
    assert_eq!(range_search(&mut w, a), Some(b), "no bonus: −3 counts as 1");
}

/// CART (scan mode `mode`) and one T0 two cells east whose converted
/// score is 100000 + `s` and whose expected remaining health is `x`.
fn weak_probe(mode: &str, s: &str, x: i32) -> Option<ObjectId> {
    let mut r = by_special();
    cart_mut(&mut r).combat.vhp_scan = Some(mode.into());
    special(&mut r, "T0", s);
    let mut w = game(&r);
    let a = veh(&mut w, "CART", 0, 0, 0);
    let b = veh(&mut w, "T0", 1, 2, 0);
    w.obj_mut(b).unwrap().combat.expected_health = x;
    range_search(&mut w, a)
}

// spec: S-0309 claim 14 and F5 (mode normal: a score of 1 is halved to 0 at expected health 0 or less, and rejected)
#[test]
fn weak_target_preference_halves() {
    assert_eq!(weak_probe("Normal", "-99999", 0), None);
    assert_eq!(weak_probe("Normal", "-99999", -5), None);
    assert!(weak_probe("None", "-99999", 0).is_some(), "mode none");
    assert_eq!(
        weak_probe("Strong", "-99999", 0),
        None,
        "mode strong rejects"
    );
    assert!(weak_probe("Strong", "-99999", 1).is_some());
}

// spec: S-0309 claim 14 and F5 (mode normal: expected health at most trunc(S / 2) doubles the score)
#[test]
fn weak_target_preference_doubles() {
    for (x, doubled) in [(50, true), (51, false), (1, true)] {
        let mut r = by_special();
        cart_mut(&mut r).combat.vhp_scan = Some("Normal".into());
        special(&mut r, "T0", "-99997"); // 3
        special(&mut r, "T1", "-99995"); // 5
        let mut w = game(&r);
        let a = veh(&mut w, "CART", 0, 0, 0);
        // Ring 1: (−1, −1) comes first in F2's order, (1, 0) last.
        let y = veh(&mut w, "T1", 1, -1, -1);
        let x_obj = veh(&mut w, "T0", 1, 1, 0);
        w.obj_mut(x_obj).unwrap().combat.expected_health = x;
        let want = if doubled { x_obj } else { y };
        assert_eq!(range_search(&mut w, a), Some(want), "x = {x}");
    }
}

/// Suppression set-up: CART's gun suppresses, the fire-suppression
/// distance is 3 cells (N = 3: rings 1 and 2 around the candidate).
fn suppress_rules() -> Ruleset {
    let mut r = by_special();
    r.sim.weapons.get_mut("Gun").unwrap().supress = Some(true);
    r.sim.general.fire_supress = Some(d("3"));
    r
}

// spec: S-0309 claim 14 and F6 (× 1/2 for every cell in rings 1 … N − 1 around the candidate holding a building allied with the searcher)
#[test]
fn suppression_halves_per_allied_building_cell() {
    for (bx, counted) in [(4, true), (5, false)] {
        let mut r = suppress_rules();
        special(&mut r, "T0", "-99999"); // 1
        let mut w = game(&r);
        let a = veh(&mut w, "CART", 0, 0, 0);
        let b = veh(&mut w, "T0", 1, 2, 0);
        put(&mut w, Kind::Building, "TOWER", 0, bx, 0);
        let want = if counted { None } else { Some(b) };
        assert_eq!(range_search(&mut w, a), want, "building at ring {}", bx - 2);
    }
}

// spec: S-0309 claim 14 and F6 (a fire-suppression distance far beyond the map still counts the allied building cell farthest from the candidate, #401)
#[test]
fn suppression_far_beyond_the_map_reaches_its_farthest_legal_cell() {
    for building in [false, true] {
        let mut r = suppress_rules();
        r.sim.general.fire_supress = Some(d("100000"));
        special(&mut r, "T0", "-99999"); // 1
        let mut w = game(&r);
        let a = veh(&mut w, "CART", 0, 0, 0);
        // The candidate at (28, 30); the building at (59, 30), 31 rings out
        // at the far edge of the coordinate square 0 … 59.
        let b = veh(&mut w, "T0", 1, -2, 0);
        if building {
            put(&mut w, Kind::Building, "TOWER", 0, 29, 0);
        }
        let want = if building { None } else { Some(b) };
        assert_eq!(range_search(&mut w, a), want, "building {building}");
    }
}

// spec: S-0309 claim 14 (suppression reads the current weapon, slot 0 for a type that is not multi-weapon, the same for every candidate; the second slot's suppress flag does not count)
#[test]
fn suppression_reads_the_current_weapon_slot() {
    for (first_suppresses, counted) in [(true, true), (false, false)] {
        let mut r = by_special();
        r.sim.general.fire_supress = Some(d("3"));
        let mut quiet = r.sim.weapons["Gun"].clone();
        quiet.supress = Some(false);
        let mut loud = quiet.clone();
        loud.supress = Some(true);
        r.sim.weapons.insert("Quiet".into(), quiet);
        r.sim.weapons.insert("Loud".into(), loud);
        let cart = r.sim.vehicles.get_mut("CART").unwrap();
        let (p, s) = if first_suppresses {
            ("Loud", "Quiet")
        } else {
            ("Quiet", "Loud")
        };
        cart.primary = Some(p.into());
        cart.secondary = Some(s.into());
        special(&mut r, "T0", "-99999"); // 1
        let mut w = game(&r);
        let a = veh(&mut w, "CART", 0, 0, 0);
        let b = veh(&mut w, "T0", 1, 2, 0);
        put(&mut w, Kind::Building, "TOWER", 0, 4, 0);
        let want = if counted { None } else { Some(b) };
        assert_eq!(
            range_search(&mut w, a),
            want,
            "first slot suppresses: {first_suppresses}"
        );
    }
}

// spec: S-0309 claim 14 (a building is counted once per cell of it in the rings)
#[test]
fn suppression_counts_each_building_cell() {
    let mut r = suppress_rules();
    special(&mut r, "T0", "-99985"); // 15
    let mut big = r.sim.buildings["TOWER"].clone();
    big.foundation = Some([2, 2]);
    r.sim.buildings.insert("BIG".into(), big);
    r.registries
        .get_mut("buildings")
        .unwrap()
        .push("BIG".into());
    let mut w = game(&r);
    let a = veh(&mut w, "CART", 0, 0, 0);
    veh(&mut w, "T0", 1, 2, 0);
    // Four cells within rings 1 and 2 of the candidate: trunc(15 / 16) = 0.
    put(&mut w, Kind::Building, "BIG", 0, 3, 0);
    assert_eq!(range_search(&mut w, a), None);
}

// spec: S-0309 claim 13 and F9 (the suppression acts on the result of the weak-target preference: 3 doubled to 6, halved to 3, not 2)
#[test]
fn score_modifiers_compose_suppression_after_preference() {
    let mut r = suppress_rules();
    cart_mut(&mut r).combat.vhp_scan = Some("Normal".into());
    special(&mut r, "T0", "-99997"); // 3
    special(&mut r, "T1", "-99998"); // 2
    let mut w = game(&r);
    let a = veh(&mut w, "CART", 0, 0, 0);
    let y = veh(&mut w, "T1", 1, -1, -1);
    let x = veh(&mut w, "T0", 1, 1, 0);
    w.obj_mut(x).unwrap().combat.expected_health = 10;
    w.obj_mut(y).unwrap().combat.expected_health = 100;
    // One allied building cell in ring 2 of x, three cells from y.
    put(&mut w, Kind::Building, "TOWER", 0, 3, 0);
    assert_eq!(
        range_search(&mut w, a),
        Some(x),
        "3 beats 2; the other order gives a tie"
    );
}

// ---------------------------------------------------------------------------
// Claim 15 and F7: expected remaining health

// spec: S-0309 claim 15 and F7 (x′ = h if x ≥ h; max(x, −30) + 1 when ⌊t / 4⌋ is odd; x otherwise)
#[test]
fn expected_health_turn_values() {
    let mut w = game(&base());
    let b = veh(&mut w, "T0", 1, 0, 0);
    let step = |w: &mut World, tick: i32, x: i32| {
        w.tick = tick;
        w.obj_mut(b).unwrap().combat.expected_health = x;
        acquire::expected_health_turn(w, b);
        expected(w, b)
    };
    assert_eq!(step(&mut w, 0, 150), 100, "above the health: the health");
    assert_eq!(step(&mut w, 0, 100), 100);
    for t in [4, 5, 6, 7, 12, 15] {
        assert_eq!(step(&mut w, t, 60), 61, "tick {t}");
    }
    for t in [0, 3, 8, 11, 16] {
        assert_eq!(step(&mut w, t, 60), 60, "tick {t}");
    }
    assert_eq!(step(&mut w, 4, -50), -29, "floor −30, then + 1");
    assert_eq!(step(&mut w, 4, -30), -29);
    assert_eq!(step(&mut w, 4, 99), 100);
}

// ---------------------------------------------------------------------------
// Claim 16 and F8: anger and the enemy house

/// A game of three player houses (0, 1, 2), none allied.
fn game3(r: &Ruleset) -> World {
    let mut s = testkit::setup(11);
    let mut third = s.houses[1].clone();
    third.name = "Gamma".into();
    s.houses.push(third);
    glasnost_sim::load::new_game(
        Arc::new(SimRules::new(r, "claims-s0309")),
        &testkit::empty_map(SIZE),
        testkit::catalogue(),
        &s,
    )
    .expect("synthetic game loads")
}

fn anger_rules() -> Ruleset {
    let mut r = base();
    r.sim.vehicles.get_mut("T0").unwrap().cost = Some(600);
    r
}

// spec: S-0309 claim 16 and F8 (Δ = conv((d / S) × C); the enemy house is worked out again: the greatest anger above 0)
#[test]
fn anger_from_damage_and_enemy_house() {
    let mut w = game3(&anger_rules());
    let victim = veh(&mut w, "T0", 0, 0, 0);
    let x1 = veh(&mut w, "CART", 1, 2, 0);
    let x2 = veh(&mut w, "CART", 2, -2, 0);
    acquire::anger(&mut w, victim, x1, 25);
    assert_eq!(w.houses[0].combat.anger[1], 150, "25 / 100 × 600");
    assert_eq!(
        w.houses[0].combat.enemy,
        Some(glasnost_sim::world::HouseId(1))
    );
    acquire::anger(&mut w, victim, x2, 50);
    assert_eq!(w.houses[0].combat.anger[2], 300);
    assert_eq!(
        w.houses[0].combat.enemy,
        Some(glasnost_sim::world::HouseId(2))
    );
    // A healing hit lowers anger.
    acquire::anger(&mut w, victim, x2, -50);
    assert_eq!(w.houses[0].combat.anger[2], 0);
    assert_eq!(
        w.houses[0].combat.enemy,
        Some(glasnost_sim::world::HouseId(1))
    );
    acquire::anger(&mut w, victim, x1, -25);
    assert_eq!(w.houses[0].combat.enemy, None, "no anger above 0");
}

// spec: S-0309 claim 16 (equal anger: the earlier house in the house order; C does not depend on the attacker's owner)
#[test]
fn anger_ties_go_to_the_earlier_house() {
    let mut w = game3(&anger_rules());
    let victim = veh(&mut w, "T0", 0, 0, 0);
    let x1 = veh(&mut w, "CART", 1, 2, 0);
    let x2 = veh(&mut w, "CART", 2, -2, 0);
    acquire::anger(&mut w, victim, x2, 25);
    acquire::anger(&mut w, victim, x1, 25);
    assert_eq!(w.houses[0].combat.anger[1], w.houses[0].combat.anger[2]);
    assert_eq!(
        w.houses[0].combat.enemy,
        Some(glasnost_sim::world::HouseId(1))
    );
}

// ---------------------------------------------------------------------------
// Claim 20: distributed fire

fn distributed_rules() -> Ruleset {
    let mut r = by_special();
    cart_mut(&mut r).distributed_fire = Some(true);
    special(&mut r, "T9", "50");
    r
}

fn fired_at(w: &World, a: ObjectId) -> Vec<ObjectId> {
    w.obj(a).unwrap().combat.fired_at.clone()
}

fn set_fired_at(w: &mut World, a: ObjectId, l: Vec<ObjectId>) {
    w.obj_mut(a).unwrap().combat.fired_at = l;
}

// spec: S-0309 claim 20 (C empty: nothing, and the fired-at list is emptied)
#[test]
fn distributed_fire_no_candidate() {
    let mut w = game(&distributed_rules());
    let a = veh(&mut w, "CART", 0, 0, 0);
    let far = veh(&mut w, "T0", 1, 12, 0);
    set_fired_at(&mut w, a, vec![far]);
    assert_eq!(range_search(&mut w, a), None);
    assert!(fired_at(&w, a).is_empty());
}

// spec: S-0309 claim 20 (some candidate not on L: the best of those; L keeps only its objects that are in C)
#[test]
fn distributed_fire_prefers_objects_not_fired_at() {
    let mut w = game(&distributed_rules());
    let a = veh(&mut w, "CART", 0, 0, 0);
    let best = veh(&mut w, "T9", 1, 2, 0);
    let other = veh(&mut w, "T0", 1, -2, 0);
    let far = veh(&mut w, "T1", 1, 12, 0);
    set_fired_at(&mut w, a, vec![far, best]);
    assert_eq!(range_search(&mut w, a), Some(other));
    assert_eq!(fired_at(&w, a), vec![best]);
}

// spec: S-0309 claim 20 (every candidate on L: the best of C, and the list is emptied)
#[test]
fn distributed_fire_all_fired_at() {
    let mut w = game(&distributed_rules());
    let a = veh(&mut w, "CART", 0, 0, 0);
    let best = veh(&mut w, "T9", 1, 2, 0);
    let other = veh(&mut w, "T0", 1, -2, 0);
    set_fired_at(&mut w, a, vec![other, best]);
    assert_eq!(range_search(&mut w, a), Some(best));
    assert!(fired_at(&w, a).is_empty());
}

// spec: S-0309 claim 20 and Edge cases (quirks.hunt_ignores_distributed_fire on: a map-wide search picks nothing and empties the list)
#[test]
fn distributed_fire_hunt_on() {
    let mut w = game(&distributed_rules());
    let a = veh(&mut w, "CART", 0, 0, 0);
    let b = veh(&mut w, "T9", 1, 2, 0);
    set_fired_at(&mut w, a, vec![b]);
    assert_eq!(
        acquire::search(&mut w, a, SearchKind::MapWide, SearchContext::Hunt),
        None
    );
    assert!(fired_at(&w, a).is_empty());
}

// spec: S-0309 Edge cases (quirks.hunt_ignores_distributed_fire off: claim 17 among the map-wide candidates, the list empty afterwards)
#[test]
fn distributed_fire_hunt_off() {
    let mut r = distributed_rules();
    r.globals.quirks.hunt_ignores_distributed_fire = false;
    let mut w = game(&r);
    let a = veh(&mut w, "CART", 0, 0, 0);
    let best = veh(&mut w, "T9", 1, 2, 0);
    let other = veh(&mut w, "T0", 1, -2, 0);
    // The best candidate is on the list: claim 17 still picks it.
    set_fired_at(&mut w, a, vec![best]);
    let _ = other;
    assert_eq!(
        acquire::search(&mut w, a, SearchKind::MapWide, SearchContext::Hunt),
        Some(best)
    );
    assert!(fired_at(&w, a).is_empty());
}

// ---------------------------------------------------------------------------
// Claim 18 and S-0215 claim 18: retaliation

/// A CART defender (house 0, guard) and a T1 attacker (house 1) at
/// (`dx`, 0).
fn duel(r: &Ruleset, dx: i32) -> (World, ObjectId, ObjectId) {
    let mut w = game(r);
    let a = veh(&mut w, "CART", 0, 0, 0);
    let x = veh(&mut w, "T1", 1, dx, 0);
    (w, a, x)
}

fn retaliated(w: &World, a: ObjectId, x: ObjectId) -> bool {
    let o = w.obj(a).unwrap();
    o.target == obj(x)
        && (o.beh.current == Some(Behaviour::Attack) || o.beh.queued == Some(Behaviour::Attack))
}

// spec: S-0309 claim 18 and S-0215 claim 18 (a defender in guard retaliates: an override to attack with the attacker as target)
#[test]
fn retaliation_happens() {
    let (mut w, a, x) = duel(&base(), 2);
    acquire::retaliate(&mut w, a, x);
    assert!(retaliated(&w, a, x));
    assert!(!opportunity(&w, a));
}

// spec: S-0309 claim 18 (the retaliate flag)
#[test]
fn retaliation_needs_the_flag() {
    let mut r = base();
    cart_mut(&mut r).combat.can_retaliate = Some(false);
    let (mut w, a, x) = duel(&r, 2);
    acquire::retaliate(&mut w, a, x);
    assert!(!retaliated(&w, a, x));
}

// spec: S-0309 claim 18 (not against an allied attacker)
#[test]
fn no_retaliation_against_allies() {
    let mut w = game(&base());
    let a = veh(&mut w, "CART", 0, 0, 0);
    let x = veh(&mut w, "T1", 0, 2, 0);
    acquire::retaliate(&mut w, a, x);
    assert!(!retaliated(&w, a, x));
}

// spec: S-0309 claim 18 and S-0308 scenario W10 (mean weapon damage above 0: M = trunc((2 − 1) / 2) = 0 does not retaliate, M = trunc((4 − 1) / 2) = 1 does)
#[test]
fn retaliation_needs_positive_mean_damage() {
    for (p, ok) in [(2, false), (4, true)] {
        let mut r = base();
        // The secondary's warhead is 0 against light, so row 15 gives the primary against the light attacker.
        let mut v = vec![d("1"); 11];
        v[3] = d("0");
        r.sim.warheads.insert(
            "Blank".into(),
            WarheadType {
                verses: Some(v),
                ..Default::default()
            },
        );
        let gun = r.sim.weapons["Gun"].clone();
        r.sim.weapons.insert(
            "Pos".into(),
            WeaponType {
                damage: Some(p),
                ..gun.clone()
            },
        );
        r.sim.weapons.insert(
            "Neg".into(),
            WeaponType {
                damage: Some(-1),
                warhead: Some("Blank".into()),
                ..gun
            },
        );
        cart_mut(&mut r).primary = Some("Pos".into());
        cart_mut(&mut r).secondary = Some("Neg".into());
        let (mut w, a, x) = duel(&r, 2);
        acquire::retaliate(&mut w, a, x);
        assert_eq!(retaliated(&w, a, x), ok, "primary {p}");
    }
}

// spec: S-0309 claim 18 (the firing check without range for the weapon it would use must answer neither illegal nor impossible)
#[test]
fn retaliation_needs_a_legal_shot() {
    // Primary 10 with a zero versus value against light: row 16 gives the secondary, a healing
    // weapon (-2). M = trunc(8 / 2) = 4. A vehicle's healing weapon is illegal against an
    // object at full health (S-0310 claim 11) and legal against a damaged one.
    let mut r = base();
    let mut v = vec![d("1"); 11];
    v[3] = d("0");
    r.sim.warheads.insert(
        "Blank".into(),
        WarheadType {
            verses: Some(v),
            ..Default::default()
        },
    );
    let gun = r.sim.weapons["Gun"].clone();
    r.sim.weapons.insert(
        "Big".into(),
        WeaponType {
            damage: Some(10),
            warhead: Some("Blank".into()),
            ..gun.clone()
        },
    );
    r.sim.weapons.insert(
        "Salve".into(),
        WeaponType {
            damage: Some(-2),
            ..gun
        },
    );
    cart_mut(&mut r).primary = Some("Big".into());
    cart_mut(&mut r).secondary = Some("Salve".into());
    let (mut w, a, x) = duel(&r, 2);
    acquire::retaliate(&mut w, a, x);
    assert!(!retaliated(&w, a, x), "illegal against full health");
    w.obj_mut(x).unwrap().health = 50;
    acquire::retaliate(&mut w, a, x);
    assert!(retaliated(&w, a, x));
}

// spec: S-0215 claim 18 (an actor of a player house that already has a target never retaliates)
#[test]
fn player_actor_with_a_target_does_not_retaliate() {
    let (mut w, a, x) = duel(&base(), 2);
    let other = veh(&mut w, "T2", 1, -2, 0);
    set_target(&mut w, a, obj(other), false);
    acquire::retaliate(&mut w, a, x);
    assert_eq!(target(&w, a), obj(other));
}

// spec: S-0215 claim 18 (a mobile actor of a player house retaliates only in guard, area guard or patrol, unless player return fire is on)
#[test]
fn player_mobile_actor_behaviour_list() {
    for (b, prf, ok) in [
        (Behaviour::Guard, false, true),
        (Behaviour::AreaGuard, false, true),
        (Behaviour::Patrol, false, true),
        (Behaviour::Sticky, false, false),
        (Behaviour::Move, false, false),
        (Behaviour::Move, true, true),
    ] {
        let mut r = base();
        r.sim.general.player_return_fire = Some(prf);
        let (mut w, a, x) = duel(&r, 2);
        set_beh(&mut w, a, b);
        acquire::retaliate(&mut w, a, x);
        assert_eq!(retaliated(&w, a, x), ok, "{b:?}, player return fire {prf}");
    }
}

// spec: S-0215 claim 18 and F7 (a player's actor that cannot reach the attacker overrides only within (S + 0.5) × 256 leptons)
#[test]
fn player_actor_distance_rule() {
    // The attacker 7 cells away (1792 leptons), beyond the gun's 1408.
    for (sight, ok) in [(6, false), (7, true)] {
        let mut r = base();
        cart_mut(&mut r).sight = Some(sight);
        let (mut w, a, x) = duel(&r, 7);
        acquire::retaliate(&mut w, a, x);
        assert_eq!(
            retaliated(&w, a, x),
            ok,
            "sight {sight}: {} leptons",
            256 * sight + 128
        );
    }
}

// spec: S-0309 claim 18 (a player's infantry unit with the explosive-charges flag does not retaliate against a building)
#[test]
fn charges_infantry_does_not_retaliate_against_buildings() {
    let mut r = base();
    r.sim.infantry.get_mut("WALKER").unwrap().c4 = Some(true);
    let mut w = game(&r);
    let a = put(&mut w, Kind::Infantry, "WALKER", 0, 0, 0);
    let tower = put(&mut w, Kind::Building, "TOWER", 1, 2, 0);
    acquire::retaliate(&mut w, a, tower);
    assert!(!retaliated(&w, a, tower));
    let x = veh(&mut w, "T1", 1, -2, 0);
    acquire::retaliate(&mut w, a, x);
    assert!(retaliated(&w, a, x), "a vehicle attacker");
}

// spec: S-0309 claim 18 (a player's vehicle that deploys into an artillery building never retaliates)
#[test]
fn artillery_deployer_does_not_retaliate() {
    let mut r = base();
    r.sim.buildings.get_mut("TOWER").unwrap().artillary = Some(true);
    cart_mut(&mut r).deploys_into = Some("TOWER".into());
    let (mut w, a, x) = duel(&r, 2);
    acquire::retaliate(&mut w, a, x);
    assert!(!retaliated(&w, a, x));
}

// spec: S-0309 claim 25 part h and S-0215 claim 18 (the neutral and special houses: the charges condition does not apply, and neither the behaviour list nor a present target holds them back)
#[test]
fn neutral_retaliation() {
    let mut r = base();
    r.sim.infantry.get_mut("WALKER").unwrap().c4 = Some(true);
    for role in [HouseRole::Neutral, HouseRole::Special] {
        let mut w = game_role(&r, role);
        let a = put(&mut w, Kind::Infantry, "WALKER", 1, 0, 0);
        let tower = put(&mut w, Kind::Building, "TOWER", 0, 2, 0);
        acquire::retaliate(&mut w, a, tower);
        assert!(
            retaliated(&w, a, tower),
            "{role:?}: charges against a building"
        );
        let mut w = game_role(&base(), role);
        let a = veh(&mut w, "CART", 1, 0, 0);
        let x = veh(&mut w, "T1", 0, 2, 0);
        set_beh(&mut w, a, Behaviour::Move);
        acquire::retaliate(&mut w, a, x);
        assert!(retaliated(&w, a, x), "{role:?}: in move");
    }
}

// spec: S-0309 claim 25 part h and S-0608 Terms (an object of the neutral or special house does not retaliate while it is draining; a player house's object is not held back by it)
#[test]
fn a_draining_neutral_object_does_not_retaliate() {
    for role in [HouseRole::Neutral, HouseRole::Special, HouseRole::Player] {
        for draining in [false, true] {
            let mut w = game_role(&base(), role);
            let a = veh(&mut w, "CART", 1, 0, 0);
            let x = veh(&mut w, "T1", 0, 2, 0);
            let b = put(&mut w, Kind::Building, "TOWER", 0, -3, 0);
            if draining {
                // A link of `a` to the building (the drain state is S-0608's).
                w.drain.links.insert(b, a);
                w.drain.partner.insert(a, b);
            }
            acquire::retaliate(&mut w, a, x);
            let held_back = draining && role != HouseRole::Player;
            assert_eq!(
                retaliated(&w, a, x),
                !held_back,
                "{role:?}, draining {draining}"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Claims 21 and 22

// spec: S-0309 claim 21 (a fire-once weapon drops its target after the shot)
#[test]
fn fire_once_drops_the_target() {
    let mut r = base();
    r.sim.weapons.get_mut("Gun").unwrap().fire_once = Some(true);
    r.sim.general.normal_targeting_delay = Some(400);
    let mut w = game(&r);
    let a = veh(&mut w, "CART", 0, 0, 0);
    let b = veh(&mut w, "T0", 1, 2, 0);
    set_target(&mut w, a, obj(b), false);
    w.obj_mut(a).unwrap().targeting_timer = 1000;
    let mut fired = false;
    for _ in 0..300 {
        glasnost_sim::tick::step(&mut w);
        if w.obj(a).unwrap().combat.rearm_until != i32::MIN {
            fired = true;
            break;
        }
    }
    assert!(fired, "the cart fires");
    assert_eq!(target(&w, a), None);
}

// spec: S-0309 claim 22 (with the limited-scan mark an area search is a range search; a search that finds nothing clears the mark)
#[test]
fn limited_scan_mark() {
    let mut r = base();
    guard(&mut r, "CART", "3");
    let mut w = game(&r);
    let a = veh(&mut w, "CART", 0, 0, 0);
    w.obj_mut(a).unwrap().guard_spot = Some(Target::Cell(at(10, 0)));
    // Around the guard spot, but 10 cells from the searcher.
    veh(&mut w, "T0", 1, 10, 1);
    w.obj_mut(a).unwrap().combat.limited_scan = true;
    // A range search (R = 768) finds nothing near the searcher: the mark is cleared.
    assert_eq!(
        acquire::search(&mut w, a, SearchKind::Area, SearchContext::Area),
        None
    );
    assert!(!w.obj(a).unwrap().combat.limited_scan);
    let near = veh(&mut w, "T1", 1, 2, 0);
    w.obj_mut(a).unwrap().combat.limited_scan = true;
    assert_eq!(
        acquire::search(&mut w, a, SearchKind::Area, SearchContext::Area),
        Some(near),
        "centred on the searcher"
    );
    assert!(
        w.obj(a).unwrap().combat.limited_scan,
        "found: the mark stays"
    );
}

// ---------------------------------------------------------------------------
// Claim 23: attack-move searches

fn attack_move(w: &mut World, a: ObjectId, mem: Target, engaged: bool) {
    let o = w.obj_mut(a).unwrap();
    // A cell is the memory's destination, an object its target only.
    o.attack_move = Some(match mem {
        Target::Cell(_) => glasnost_sim::world::AttackMoveMemory::Dest(mem),
        Target::Object(_) => glasnost_sim::world::AttackMoveMemory::Target(mem),
    });
    o.combat.engaged = engaged;
}

// spec: S-0309 claim 23 (a), row "a destination, not engaged" (a range search keeping a usable target; a target found queues attack with an immediate start and counts as engaged; not limited by claims 2 and 3; not an opportunity target)
#[test]
fn attack_move_destination_not_engaged() {
    let mut r = base();
    cart_mut(&mut r).combat.can_passive_aquire = Some(false);
    let mut w = game(&r);
    let a = veh(&mut w, "CART", 0, 0, 0);
    let b = veh(&mut w, "T0", 1, 2, 0);
    set_beh(&mut w, a, Behaviour::Move);
    attack_move(&mut w, a, Target::Cell(at(-8, 0)), false);
    expire(&mut w, a);
    acquire::per_tick(&mut w, a);
    assert_eq!(target(&w, a), obj(b));
    assert!(!opportunity(&w, a));
    let o = w.obj(a).unwrap();
    assert!(o.combat.engaged);
    assert_eq!(o.beh.current, Some(Behaviour::Attack));
}

// spec: S-0309 claim 23 (a), row "only a target", memory target within reach (nothing happens; the timer is not restarted)
#[test]
fn attack_move_target_within_reach_does_nothing() {
    let mut w = game(&base());
    let a = veh(&mut w, "CART", 0, 0, 0);
    let m = veh(&mut w, "T0", 1, 2, 0);
    veh(&mut w, "T1", 1, -2, 0);
    attack_move(&mut w, a, Target::Object(m), false);
    expire(&mut w, a);
    let rng = w.rng.clone();
    let t0 = timer(&w, a);
    acquire::per_tick(&mut w, a);
    assert_eq!(target(&w, a), None);
    assert_eq!(timer(&w, a), t0);
    assert_eq!(w.rng, rng);
}

// spec: S-0309 claim 23 (a), row "only a target", any other state (a search as for no target; nothing found: the memory's target again; found: attack and engaged)
#[test]
fn attack_move_target_out_of_reach() {
    let mut w = game(&base());
    let a = veh(&mut w, "CART", 0, 0, 0);
    let m = veh(&mut w, "T0", 1, 12, 0);
    attack_move(&mut w, a, Target::Object(m), false);
    expire(&mut w, a);
    acquire::per_tick(&mut w, a);
    assert_eq!(target(&w, a), obj(m), "nothing found");
    assert!(!w.obj(a).unwrap().combat.engaged);
    let b = veh(&mut w, "T1", 1, -2, 0);
    expire(&mut w, a);
    acquire::per_tick(&mut w, a);
    assert_eq!(
        target(&w, a),
        obj(b),
        "found: its present target is not kept"
    );
    assert!(w.obj(a).unwrap().combat.engaged);
    assert!(!opportunity(&w, a));
}

// spec: S-0309 claim 23 (a), rows "a destination, engaged" (target within reach: nothing; none within reach: a search as for no target, nothing found leaves no target, still engaged)
#[test]
fn attack_move_destination_engaged() {
    let mut w = game(&base());
    let a = veh(&mut w, "CART", 0, 0, 0);
    let near = veh(&mut w, "T0", 1, 2, 0);
    attack_move(&mut w, a, Target::Cell(at(-8, 0)), true);
    set_target(&mut w, a, obj(near), false);
    expire(&mut w, a);
    let t0 = timer(&w, a);
    acquire::per_tick(&mut w, a);
    assert_eq!(target(&w, a), obj(near));
    assert_eq!(timer(&w, a), t0, "nothing happens");
    let far = veh(&mut w, "T1", 1, 12, 0);
    kill(&mut w, near);
    set_target(&mut w, a, obj(far), false);
    expire(&mut w, a);
    acquire::per_tick(&mut w, a);
    assert_eq!(target(&w, a), None);
    assert!(w.obj(a).unwrap().combat.engaged);
}

// ---------------------------------------------------------------------------
// Claim 25: the neutral and special houses

// spec: S-0309 claim 25 part a (an engineer of these houses is not excluded from passive acquisition)
#[test]
fn neutral_engineer_may_search_passively() {
    let mut r = base();
    r.sim.infantry.get_mut("WALKER").unwrap().engineer = Some(true);
    for role in [HouseRole::Neutral, HouseRole::Special] {
        let mut w = game_role(&r, role);
        let e = put(&mut w, Kind::Infantry, "WALKER", 1, 0, 0);
        assert!(acquire::may_search_passively(&w, e), "{role:?}");
    }
}

// spec: S-0309 claim 8 and claim 25 part a (the engineer exception of F1: R = 0 for an engineer, so the rings come from the weapon ranges)
#[test]
fn engineer_range_search_ignores_guard_range() {
    let mut r = base();
    let wk = r.sim.infantry.get_mut("WALKER").unwrap();
    wk.engineer = Some(true);
    wk.guard_range = Some(d("10"));
    shed(&mut r, None, 0, false);
    // Capturable: table C's capture row applies to an engineer of these houses.
    r.sim.buildings.get_mut("SHED").unwrap().capturable = Some(true);
    let mut w = game_role(&r, HouseRole::Neutral);
    let e = put(&mut w, Kind::Infantry, "WALKER", 1, 0, 0);
    // Rifle: 4 cells. A building 8 cells away is within the guard range but beyond the reach.
    put(&mut w, Kind::Building, "SHED", 0, 8, 0);
    assert_eq!(range_search(&mut w, e), None);
    let near = put(&mut w, Kind::Building, "SHED", 0, 3, 0);
    assert_eq!(range_search(&mut w, e), Some(near));
}

// ---------------------------------------------------------------------------
// S-0215 claims 26, 27 and 33: searches from processing, and the drop rule

// spec: S-0215 claim 26 (area guard searches around its guard spot when it may; a search that finds a target ends the processing with a delay of 1 tick) and S-0309 claim 1
#[test]
fn area_guard_processing_searches() {
    let mut w = game(&base());
    let a = veh(&mut w, "CART", 0, 0, 0);
    set_beh(&mut w, a, Behaviour::AreaGuard);
    w.obj_mut(a).unwrap().guard_spot = Some(Target::Cell(at(0, 0)));
    expire(&mut w, a);
    // 8 cells: inside the area search (R = 2816) but beyond a range search's reach.
    let b = veh(&mut w, "T0", 1, 8, 0);
    let t = w.tick;
    glasnost_sim::tick::step(&mut w);
    assert_eq!(target(&w, a), obj(b));
    assert!(
        !opportunity(&w, a),
        "the area search marks no opportunity target"
    );
    assert_eq!(w.obj(a).unwrap().beh.last, Some((t, 1)));
}

// spec: S-0215 claim 26 (a guard spot taken as the own cell already qualifies for the table in that processing: the actor searches around it at once; a target found ends the processing with 1 tick and no delay draw; Q-0231)
#[test]
fn area_guard_own_cell_spot_searches_on_first_processing() {
    let mut w = game(&base());
    let a = veh(&mut w, "CART", 0, 0, 0);
    set_beh(&mut w, a, Behaviour::AreaGuard);
    w.obj_mut(a).unwrap().guard_spot = None;
    expire(&mut w, a);
    let b = veh(&mut w, "T0", 1, 8, 0);
    let t = w.tick;
    glasnost_sim::tick::step(&mut w);
    let o = w.obj(a).unwrap();
    assert_eq!(o.guard_spot, Some(Target::Cell(at(0, 0))));
    assert_eq!(target(&w, a), obj(b));
    assert_eq!(w.obj(a).unwrap().beh.last, Some((t, 1)));
}

// spec: S-0215 claim 26 (a newly taken own-cell guard spot whose search finds nothing: the search restarts the targeting timer and the processing takes the ordinary delay; Q-0231)
#[test]
fn area_guard_own_cell_spot_search_finds_nothing() {
    let mut w = game(&base());
    let a = veh(&mut w, "CART", 0, 0, 0);
    set_beh(&mut w, a, Behaviour::AreaGuard);
    w.obj_mut(a).unwrap().guard_spot = None;
    expire(&mut w, a);
    let t = w.tick;
    glasnost_sim::tick::step(&mut w);
    let o = w.obj(a).unwrap();
    assert_eq!(o.guard_spot, Some(Target::Cell(at(0, 0))));
    assert_eq!(o.target, None);
    assert!(o.targeting_timer > t, "{}", o.targeting_timer);
    let (_, d) = o.beh.last.unwrap();
    assert!(d > 1, "{d}");
}

// spec: S-0215 claim 26 (the leash holds and it may not search: drops any target and takes the guard spot as destination)
#[test]
fn area_guard_leash_drops_target_without_search() {
    let mut r = base();
    guard(&mut r, "CART", "2");
    let mut w = game(&r);
    let a = veh(&mut w, "CART", 0, 0, 0);
    set_beh(&mut w, a, Behaviour::AreaGuard);
    let spot = Target::Cell(at(-10, 0));
    w.obj_mut(a).unwrap().guard_spot = Some(spot);
    let b = veh(&mut w, "T0", 1, 2, 0);
    set_target(&mut w, a, obj(b), false);
    w.obj_mut(a).unwrap().targeting_timer = 1000;
    glasnost_sim::tick::step(&mut w);
    let o = w.obj(a).unwrap();
    assert_eq!(o.target, None, "the leash drops the target");
    assert_eq!(o.dest, Some(spot));
}

// spec: S-0215 claim 26 (the leash holds and it may search: drops any target, takes the guard spot as destination, and searches around the guard spot; a target found ends the processing with 1 tick, so the dropped target is never engaged in that processing)
#[test]
fn area_guard_leash_drops_target_and_searches() {
    let mut r = base();
    // g = 2 cells: leash L = trunc(11 × 1024 / 10) = 1126; area search R = 1024, n = 4.
    guard(&mut r, "CART", "2");
    let mut w = game(&r);
    let a = veh(&mut w, "CART", 0, 0, 0);
    set_beh(&mut w, a, Behaviour::AreaGuard);
    // The guard spot 1280 leptons away: the leash holds.
    let spot = Target::Cell(at(-5, 0));
    w.obj_mut(a).unwrap().guard_spot = Some(spot);
    let held = veh(&mut w, "T0", 1, 2, 0);
    set_target(&mut w, a, obj(held), false);
    // Ring 2 of the guard spot, 768 leptons from the searcher.
    let near_spot = veh(&mut w, "T1", 1, -3, 0);
    expire(&mut w, a);
    let t = w.tick;
    glasnost_sim::tick::step(&mut w);
    let o = w.obj(a).unwrap();
    assert_eq!(o.target, obj(near_spot));
    assert_eq!(o.dest, Some(spot));
    assert_eq!(o.beh.last, Some((t, 1)));
}

// spec: S-0215 claim 27 and S-0309 claim 1 (hunt runs the map-wide search at each processing; a target already held is kept)
#[test]
fn hunt_processing_searches_map_wide() {
    let mut w = game(&base());
    let a = veh(&mut w, "CART", 0, 0, 0);
    set_beh(&mut w, a, Behaviour::Hunt);
    let b = veh(&mut w, "T0", 1, 14, 0);
    glasnost_sim::tick::step(&mut w);
    assert_eq!(target(&w, a), obj(b));
    // Kept: a nearer enemy does not replace it.
    veh(&mut w, "T1", 1, 2, 0);
    let rng_timer = timer(&w, a);
    for _ in 0..40 {
        glasnost_sim::tick::step(&mut w);
    }
    assert_eq!(target(&w, a), obj(b));
    assert_ne!(timer(&w, a), rng_timer, "every search restarts the timer");
}

// spec: S-0215 claim 27 (an actor whose type has the stupid-hunt flag does not search)
#[test]
fn stupid_hunt_does_not_search() {
    let mut r = base();
    cart_mut(&mut r).stupid_hunt = Some(true);
    let mut w = game(&r);
    let a = veh(&mut w, "CART", 0, 0, 0);
    set_beh(&mut w, a, Behaviour::Hunt);
    w.obj_mut(a).unwrap().targeting_timer = 1000;
    veh(&mut w, "T0", 1, 14, 0);
    glasnost_sim::tick::step(&mut w);
    assert_eq!(target(&w, a), None);
    assert_eq!(timer(&w, a), 1000);
}

// spec: S-0215 claim 33 (an opportunity target is dropped in the listed behaviours, here sleep; a target that is not one is kept; guard keeps it)
#[test]
fn drop_rule_for_opportunity_targets() {
    for (b, opp, kept) in [
        (Behaviour::Sleep, true, false),
        (Behaviour::Sleep, false, true),
        (Behaviour::Guard, true, true),
    ] {
        let mut w = game(&base());
        let a = veh(&mut w, "CART", 0, 0, 0);
        let x = veh(&mut w, "T0", 1, 2, 0);
        set_beh(&mut w, a, b);
        set_target(&mut w, a, obj(x), opp);
        w.obj_mut(a).unwrap().targeting_timer = 1000;
        glasnost_sim::tick::step(&mut w);
        let want = if kept { obj(x) } else { None };
        assert_eq!(target(&w, a), want, "{b:?}, opportunity {opp}");
    }
}

// spec: S-0215 claim 33 (c) (a target that the per-tick search finds on a tick is not removed by that tick's drop rule, and is first acted on by a later processing)
#[test]
fn per_tick_search_target_survives_its_tick() {
    let mut w = game(&base());
    let a = veh(&mut w, "CART", 0, 0, 0);
    let b = veh(&mut w, "T0", 1, 2, 0);
    expire(&mut w, a);
    glasnost_sim::tick::step(&mut w);
    assert_eq!(target(&w, a), obj(b));
    assert!(opportunity(&w, a));
    assert_eq!(
        w.obj(a).unwrap().combat.rearm_until,
        i32::MIN,
        "not fired at by the processing of the same tick"
    );
}

// spec: S-0309 claim 25 part e (rows passive house and insignificant: a searcher of the neutral or special house is rejected too unless T is a building and A an infantry unit that may capture or occupy it)
#[test]
fn neutral_searcher_passive_house_and_insignificant_rows() {
    let mut r = base();
    guard(&mut r, "CART", "4");
    r.sim.vehicles.get_mut("T1").unwrap().combat.insignificant = Some(true);
    let mut w = game_role(&r, HouseRole::Neutral);
    let a = veh(&mut w, "CART", 1, 0, 0);
    veh(&mut w, "T1", 0, 2, 0);
    assert_eq!(range_search(&mut w, a), None, "insignificant");
    let b = veh(&mut w, "T2", 0, -2, 0);
    assert_eq!(range_search(&mut w, a), Some(b));
    // House 0's country made multiplayer-passive.
    let mut r = base();
    guard(&mut r, "CART", "4");
    r.sim
        .countries
        .get_mut("Alphaland")
        .unwrap()
        .multiplay_passive = Some(true);
    let mut w = game_role(&r, HouseRole::Special);
    let a = veh(&mut w, "CART", 1, 0, 0);
    veh(&mut w, "T2", 0, 2, 0);
    assert_eq!(range_search(&mut w, a), None, "passive house");
}

// spec: S-0309 claim 25 part b (an infantry unit of the neutral or special house with the explosive-charges flag also searches for buildings; a player's does not)
#[test]
fn neutral_charges_infantry_searches_for_buildings() {
    let mut r = base();
    special_weapons(&mut r);
    let wk = r.sim.infantry.get_mut("WALKER").unwrap();
    wk.primary = Some("Needle".into());
    wk.c4 = Some(true);
    let mut w = game_role(&r, HouseRole::Neutral);
    let p = put(&mut w, Kind::Infantry, "WALKER", 0, 0, 0);
    let n = put(&mut w, Kind::Infantry, "WALKER", 1, 3, 3);
    assert_eq!(classes(&w, p), (true, false, false, false, false));
    assert_eq!(classes(&w, n), (true, false, false, true, false));
}

/// Adds an unarmed aircraft type `PLANE` (light armour).
fn plane(r: &mut Ruleset) {
    let mut p = r.sim.vehicles["T0"].clone();
    p.locomotion_class = Some(glasnost_rules::sim::LocomotionClass::Winged);
    p.locomotor = Some(glasnost_rules::sim::Locomotor::Fly);
    r.sim.aircraft.insert("PLANE".into(), p);
    r.registries
        .entry("aircraft".into())
        .or_default()
        .push("PLANE".into());
}

// spec: S-0309 claim 7 and table C row class (the map-wide search's second pass considers the landed aircraft of the ground layer, which a search for vehicles accepts)
#[test]
fn map_wide_search_finds_landed_aircraft() {
    let mut r = base();
    plane(&mut r);
    let mut w = game(&r);
    let a = veh(&mut w, "CART", 0, 0, 0);
    let p = put(&mut w, Kind::Aircraft, "PLANE", 1, 10, 0);
    assert!(!acquire::classes(&w, a).aircraft);
    assert_eq!(
        acquire::search(&mut w, a, SearchKind::MapWide, SearchContext::Hunt),
        Some(p)
    );
}

// spec: S-0309 claim 5 (an aircraft takes no classes from its weapons)
#[test]
fn aircraft_take_no_classes() {
    let mut r = base();
    plane(&mut r);
    r.sim.aircraft.get_mut("PLANE").unwrap().primary = Some("Gun".into());
    let mut w = game(&r);
    let p = put(&mut w, Kind::Aircraft, "PLANE", 0, 0, 0);
    assert_eq!(classes(&w, p), NONE);
}

// spec: S-0309 claim 25 part h (an object of the neutral or special house with a target retaliates only when the attacker's threat sum is at least its target's)
#[test]
fn neutral_retaliation_compares_threat_sums() {
    let mut r = by_special();
    special(&mut r, "T5", "5");
    special(&mut r, "T6", "6");
    for (attacker, ok) in [("T0", false), ("T5", true), ("T6", true)] {
        let mut w = game_role(&r, HouseRole::Neutral);
        let a = veh(&mut w, "CART", 1, 0, 0);
        let held = veh(&mut w, "T5", 0, -2, 0);
        set_target(&mut w, a, obj(held), false);
        let x = veh(&mut w, attacker, 0, 2, 0);
        acquire::retaliate(&mut w, a, x);
        assert_eq!(retaliated(&w, a, x), ok, "attacker {attacker}");
        if !ok {
            assert_eq!(target(&w, a), obj(held));
        }
    }
}

// ---------------------------------------------------------------------------
// Claims 7 and 11: movement-zone regions

/// A game whose map has a line of rock cells (a land type with no land row:
/// blocked for every class) along cell column `mid + 2`, which splits the
/// map into two regions.
fn walled_game(r: &Ruleset) -> World {
    use glasnost_map::terrain::{TileCell, TileInfo};
    let mut cat = (*testkit::catalogue()).clone();
    let rock_tile = cat.tiles.len() as i32;
    cat.tiles.push(TileInfo {
        width: 1,
        height: 1,
        has_image: true,
        cells: vec![Some(TileCell {
            terrain_code: 7,
            slope: 0,
        })],
        variants: 0,
        anim: None,
        no_burrowing: false,
    });
    let mut map = testkit::empty_map(SIZE);
    let wall_x = at(2, 0).x;
    let cells: Vec<CellXY> = map.cells.iter().map(|(c, _)| c).collect();
    for c in cells {
        if c.x == wall_x {
            map.cells.get_mut(c).unwrap().tile = rock_tile;
        }
    }
    glasnost_sim::load::new_game(
        Arc::new(SimRules::new(r, "claims-s0309")),
        &map,
        Arc::new(cat),
        &testkit::setup(11),
    )
    .expect("synthetic game loads")
}

// spec: S-0309 claim 11 and table C row zone (an area search skips cells outside the searcher's region; a range search does not)
#[test]
fn area_search_skips_other_regions() {
    let mut w = walled_game(&base());
    let a = veh(&mut w, "CART", 0, 0, 0);
    w.obj_mut(a).unwrap().guard_spot = Some(Target::Cell(at(0, 0)));
    let b = veh(&mut w, "T0", 1, 3, 0);
    assert_eq!(
        acquire::search(&mut w, a, SearchKind::Area, SearchContext::Area),
        None
    );
    assert_eq!(range_search(&mut w, a), Some(b));
}

// spec: S-0309 claim 7 (a mobile searcher's map-wide search considers a ground-layer object only in its own region)
#[test]
fn map_wide_search_skips_other_regions() {
    let mut w = walled_game(&base());
    let a = veh(&mut w, "CART", 0, 0, 0);
    veh(&mut w, "T0", 1, 3, 0);
    assert_eq!(
        acquire::search(&mut w, a, SearchKind::MapWide, SearchContext::Hunt),
        None
    );
    let b = veh(&mut w, "T1", 1, -6, 0);
    assert_eq!(
        acquire::search(&mut w, a, SearchKind::MapWide, SearchContext::Hunt),
        Some(b)
    );
}

// spec: S-0309 claim 7 and table C row zone (revision 21, Q-1502 answered: two cells of no reach region of the searcher's movement zone count as the same region; a searcher standing on such a cell considers only objects on such cells, anywhere on the map, and one on open ground never considers an object on such a cell)
#[test]
fn cells_of_no_reach_region_compare_as_one_region() {
    let mut w = walled_game(&base());
    // The rock column at mid + 2 is a cell of no reach region for every zone.
    let a = veh(&mut w, "CART", 0, 2, 0);
    let near = veh(&mut w, "T0", 1, 0, 0);
    let far = veh(&mut w, "T1", 1, 2, 12);
    assert_eq!(
        acquire::search(&mut w, a, SearchKind::MapWide, SearchContext::Hunt),
        Some(far),
        "the cliff searcher picks the far cliff enemy, not the open-ground one"
    );
    let b = veh(&mut w, "CART", 0, -2, 12);
    assert_eq!(
        acquire::search(&mut w, b, SearchKind::MapWide, SearchContext::Hunt),
        Some(near),
        "an open-ground searcher never considers the enemy on the cliff cell"
    );
}

// ---------------------------------------------------------------------------
// The money thief (claim 5, table C rows capture and storage) and claim 25
// part e's capture exception.

fn thief_rules(capturable: bool, storage: Option<i32>) -> Ruleset {
    let mut r = base();
    shed(&mut r, Some("Gun"), 5, false);
    let b = r.sim.buildings.get_mut("SHED").unwrap();
    b.capturable = Some(capturable);
    b.storage = storage;
    let wk = r.sim.infantry.get_mut("WALKER").unwrap();
    wk.thief = Some(true);
    wk.guard_range = Some(d("6"));
    r
}

// spec: S-0309 claim 5 (the money-thief flag also searches for buildings; every candidate a capturable building with storage)
// spec: S-0309 table C row capture (a money thief: T a building whose type is capturable)
// spec: S-0309 table C row storage (a money thief: T's type has storage capacity)
#[test]
fn money_thief_finds_only_capturable_buildings_with_storage() {
    for (capturable, storage, want) in [
        (true, Some(100), true),
        (true, None, false),
        (false, Some(100), false),
    ] {
        let r = thief_rules(capturable, storage);
        let mut w = game(&r);
        let a = put(&mut w, Kind::Infantry, "WALKER", 0, 0, 0);
        let cart = veh(&mut w, "CART", 1, 1, 0);
        let b = put(&mut w, Kind::Building, "SHED", 1, 2, 0);
        let got = range_search(&mut w, a);
        assert_ne!(
            got,
            Some(cart),
            "a vehicle is never a money thief's candidate"
        );
        assert_eq!(got == Some(b), want, "{capturable} {storage:?}: {got:?}");
    }
}

// spec: S-0309 claim 25 part e (a searcher of the neutral house: a passive-house building passes when it needs an engineer, A is an engineer and T's owner is not A's)
#[test]
fn neutral_engineer_passes_a_passive_building_that_needs_an_engineer() {
    for needs in [true, false] {
        let mut r = base();
        shed(&mut r, Some("Gun"), 5, false);
        let b = r.sim.buildings.get_mut("SHED").unwrap();
        b.capturable = Some(true);
        b.needs_engineer = Some(needs);
        r.sim
            .countries
            .get_mut("Alphaland")
            .unwrap()
            .multiplay_passive = Some(true);
        let wk = r.sim.infantry.get_mut("WALKER").unwrap();
        wk.engineer = Some(true);
        wk.guard_range = Some(d("6"));
        let mut w = game_role(&r, HouseRole::Neutral);
        let a = put(&mut w, Kind::Infantry, "WALKER", 1, 0, 0);
        let b = put(&mut w, Kind::Building, "SHED", 0, 2, 0);
        assert_eq!(
            range_search(&mut w, a) == Some(b),
            needs,
            "needs engineer {needs}"
        );
    }
}

// ---------------------------------------------------------------------------
// Revision 3: landed aircraft in both passes of the map-wide search.

/// A hunter (CART, c_st = 1, c_d = 1, k = 5) and a landed aircraft 7 cells
/// away whose special threat makes the lepton-distance sum exactly 0 and
/// the cell-distance sum negative (counting as 1), as in `lepton_hunt`.
fn landed_hunt(anti_air: bool, quirk: bool) -> Option<ObjectId> {
    let mut r = base();
    special_weapons(&mut r);
    coefficients(&mut r, "CART", ["0", "0", "1", "0", "1"]);
    special(&mut r, "T0", "-101787");
    plane(&mut r);
    if anti_air {
        r.sim.vehicles.get_mut("CART").unwrap().secondary = Some("Flak".into());
    }
    r.globals.quirks.hunt_distance_in_leptons = quirk;
    let mut w = game(&r);
    let a = veh(&mut w, "CART", 0, 0, 0);
    assert_eq!(acquire::classes(&w, a).aircraft, anti_air);
    put(&mut w, Kind::Aircraft, "PLANE", 1, 7, 0);
    acquire::search(&mut w, a, SearchKind::MapWide, SearchContext::Hunt)
}

// spec: S-0309 claim 7 and F4 (without the aircraft class a landed aircraft is considered in the ground-layer pass only, with distance in leptons; scenario A11 part h)
#[test]
fn claim7_landed_aircraft_in_the_ground_pass_uses_leptons() {
    assert_eq!(
        landed_hunt(false, true),
        None,
        "the lepton sum is exactly 0"
    );
    // quirks.hunt_distance_in_leptons off: cells, a negative sum counting as 1.
    assert!(landed_hunt(false, false).is_some());
}

// spec: S-0309 claim 7 and F4 (with the aircraft class a landed aircraft is considered in both passes, each with its own score: the aircraft pass in cells supplies it although the ground pass rejects it; scenario A11 part i)
#[test]
fn claim7_landed_aircraft_in_both_passes() {
    assert!(landed_hunt(true, true).is_some());
}

// spec: S-0309 table C row exists (revision 4: a destroyed building awaiting removal keeps its foundation cells with 0 health and is not a candidate; scenario A14 part o)
#[test]
fn table_c_exists_building_awaiting_removal() {
    let mut r = base();
    shed(&mut r, Some("Gun"), 5, false);
    assert!(found(&r, Kind::Building, "SHED"));
    let (mut w, a, b) = one_target(&r, Kind::Building, "SHED");
    let o = w.obj_mut(b).unwrap();
    o.health = 0;
    o.combat.killed = true;
    assert!(w.obj(b).unwrap().on_map, "still in its foundation cells");
    assert_eq!(range_search(&mut w, a), None);
}

// Test-only computer-controlled player houses (S-0309 Terms, Test-only
// exception; S-0215 Terms and claims 18 and 19; glossary).

/// A game whose house 1 is a player house marked as a test-only
/// computer-controlled player house (`human` clear).
fn game_test_house(r: &Ruleset) -> World {
    let mut w = game(r);
    assert_eq!(w.houses[1].role, HouseRole::Player);
    w.houses[1].human = false;
    w
}

// spec: S-0309 claim 25 part b and Terms (Test-only exception: a marked player house's deploy-to-fire vehicle and no-auto-fire object search like any other; an unmarked player house's never search)
#[test]
fn test_house_deploy_to_fire_and_no_auto_fire_search() {
    let mut r = base();
    r.sim.vehicles.get_mut("T1").unwrap().primary = Some("Gun".into());
    r.sim.vehicles.get_mut("T1").unwrap().deploy_to_fire = Some(true);
    r.sim.vehicles.get_mut("T2").unwrap().primary = Some("Gun".into());
    r.sim.vehicles.get_mut("T2").unwrap().combat.no_auto_fire = Some(true);
    let mut w = game_test_house(&r);
    let dtf = veh(&mut w, "T1", 1, 0, 0);
    let naf = veh(&mut w, "T2", 1, 2, 0);
    assert_eq!(classes(&w, dtf), GROUND);
    assert_eq!(classes(&w, naf), GROUND);
    let dtf0 = veh(&mut w, "T1", 0, 0, 4);
    let naf0 = veh(&mut w, "T2", 0, 2, 4);
    assert_eq!(classes(&w, dtf0), NONE);
    assert_eq!(classes(&w, naf0), NONE);
}

// spec: S-0215 claim 18 and S-0309 claim 25 part h (a marked test house's mobile actor retaliates outside guard, area guard and patrol and with a target held, as the neutral and special houses' do; an unmarked player house's does not)
#[test]
fn test_house_retaliation_outside_the_listed_behaviours() {
    for human in [true, false] {
        let mut w = game(&base());
        w.houses[1].human = human;
        let a = veh(&mut w, "CART", 1, 0, 0);
        let x = veh(&mut w, "T1", 0, 2, 0);
        set_beh(&mut w, a, Behaviour::Move);
        acquire::retaliate(&mut w, a, x);
        assert_eq!(retaliated(&w, a, x), !human, "human {human}: in move");
    }
}

// spec: S-0215 claim 18 and Terms (the test-only exception does not remove the retaliate-property requirement)
#[test]
fn test_house_needs_the_retaliate_property() {
    let mut r = base();
    r.sim
        .behaviours
        .entry(Behaviour::Move.key().into())
        .or_default()
        .retaliate = Some(false);
    let mut w = game_test_house(&r);
    let a = veh(&mut w, "CART", 1, 0, 0);
    let x = veh(&mut w, "T1", 0, 2, 0);
    set_beh(&mut w, a, Behaviour::Move);
    acquire::retaliate(&mut w, a, x);
    assert!(!retaliated(&w, a, x));
}

// ---------------------------------------------------------------------------
// Claim 25 part g: the disguise draw of the computer-owner houses

/// Disguises `id` as vehicle type T5 with no house (S-0516 claim 30); with
/// `blink`, its fake blink runs (S-0516 claim 39).
fn disguise(w: &mut World, id: ObjectId, blink: bool) {
    use glasnost_sim::cloak::{Disguise, DisguiseAs};
    let t5 = w.rules().index_of(Kind::Vehicle, "T5").unwrap();
    let tick = w.tick;
    let o = w.obj_mut(id).unwrap();
    o.cloak.disguise = Some(Disguise {
        what: Some(DisguiseAs::Vehicle(t5)),
        house: None,
        since: tick,
    });
    o.cloak.blink_until = if blink { tick + 10 } else { 0 };
}

/// The base rules with CART's guard range 4 cells and the disguise-detection
/// list `list`.
fn detection(list: &[i32]) -> Ruleset {
    let mut r = base();
    guard(&mut r, "CART", "4");
    r.sim.general.disguise_detection_percent = Some(list.to_vec());
    r
}

/// The stream after a search from `before`: the targeting timer's restart
/// draw U(0, 2) (S-0215 claim 33, S-0309 claim 4), then, with `disguise`,
/// exactly one R(0, 99) draw (claim 25 (g): it follows the timer draw).
fn draws(before: &glasnost_sim::rng::Rng, w: &World, disguise: bool) -> glasnost_sim::rng::Rng {
    let half = w.rules().quirks.ranged_draw_half_span;
    let mut x = before.clone();
    let _ = x.ranged(0, 2, half);
    if disguise {
        let _ = x.ranged(0, 99, half);
    }
    x
}

// spec: S-0309 claim 25 (g) (a searcher of the neutral or special house: a target that appears disguised is rejected unless its disguise is blinking and one draw R(0, 99), taken for this test, is at most the house's entry)
#[test]
fn neutral_searcher_sees_through_a_blinking_disguise_by_one_draw() {
    let r = detection(&[100, 100, 100]);
    let mut w = game_role(&r, HouseRole::Neutral);
    let a = veh(&mut w, "CART", 1, 0, 0);
    let b = veh(&mut w, "T0", 0, 2, 0);
    // Not blinking: rejected, no draw.
    disguise(&mut w, b, false);
    let before = w.rng.clone();
    assert_eq!(range_search(&mut w, a), None);
    assert_eq!(
        w.rng,
        draws(&before, &w, false),
        "no draw for a disguise that does not blink"
    );
    // Blinking: one draw, and an entry of 100 lets it through.
    disguise(&mut w, b, true);
    let before = w.rng.clone();
    assert_eq!(range_search(&mut w, a), Some(b));
    assert_eq!(w.rng, draws(&before, &w, true), "exactly one R(0, 99) draw");
}

// spec: S-0309 claim 25 (g) and table C row disguise (the draw is the computer-owner houses' only: a player house's searcher rejects a target that appears disguised, blinking or not, with no draw)
#[test]
fn player_searcher_never_takes_the_disguise_draw() {
    let r = detection(&[100, 100, 100]);
    let mut w = game(&r);
    let a = veh(&mut w, "CART", 1, 0, 0);
    let b = veh(&mut w, "T0", 0, 2, 0);
    disguise(&mut w, b, true);
    let before = w.rng.clone();
    assert_eq!(range_search(&mut w, a), None);
    assert_eq!(w.rng, draws(&before, &w, false));
}

// spec: S-0309 claim 25 (g) (the entry is selected by the house's difficulty index, easy, normal, difficult; the neutral and special houses have index 1, the second entry), Terms (Test-only exception: a marked house reads its oracle index) and Import mapping (no such entry: nothing is let through by the draw, a departure)
#[test]
fn disguise_draw_entry_by_difficulty_index() {
    // A negative entry lets nothing through (every draw is at least 0), but
    // the draw is still taken.
    let r = detection(&[100, -1, -1]);
    let mut w = game_role(&r, HouseRole::Special);
    let a = veh(&mut w, "CART", 1, 0, 0);
    let b = veh(&mut w, "T0", 0, 2, 0);
    disguise(&mut w, b, true);
    let before = w.rng.clone();
    assert_eq!(range_search(&mut w, a), None, "index 1: the second entry");
    assert_eq!(w.rng, draws(&before, &w, true));
    // A marked test house with index 0 reads the first entry.
    let mut w = game_test_house(&r);
    w.houses[1].econ.setup.difficulty = 0;
    let a = veh(&mut w, "CART", 1, 0, 0);
    let b = veh(&mut w, "T0", 0, 2, 0);
    disguise(&mut w, b, true);
    assert_eq!(range_search(&mut w, a), Some(b), "index 0: the first entry");
    // A list without the house's entry lets nothing through by the draw.
    let r = detection(&[100]);
    let mut w = game_role(&r, HouseRole::Neutral);
    let a = veh(&mut w, "CART", 1, 0, 0);
    let b = veh(&mut w, "T0", 0, 2, 0);
    disguise(&mut w, b, true);
    let before = w.rng.clone();
    assert_eq!(range_search(&mut w, a), None, "no second entry");
    assert_eq!(w.rng, draws(&before, &w, true));
}

// spec: S-0309 claim 25 (g) (the draw is taken only for an object that every other row lets through, except the capture, engineer, storage, bridge-layers, healer and score rows; an object that any other row rejects takes none)
#[test]
fn disguise_draw_only_for_an_object_the_other_rows_let_through() {
    let r = detection(&[100, 100, 100]);
    // Beyond the guard range: the distance row rejects it, no draw.
    let mut w = game_role(&r, HouseRole::Neutral);
    let a = veh(&mut w, "CART", 1, 0, 0);
    let far = veh(&mut w, "T0", 0, 7, 0);
    disguise(&mut w, far, true);
    let before = w.rng.clone();
    assert_eq!(range_search(&mut w, a), None);
    assert_eq!(w.rng, draws(&before, &w, false), "distance row: no draw");
    // A friendly disguised object: the friendly row rejects it, no draw.
    let mut w = game_role(&r, HouseRole::Neutral);
    let a = veh(&mut w, "CART", 1, 0, 0);
    let own = veh(&mut w, "T0", 1, 2, 0);
    disguise(&mut w, own, true);
    let before = w.rng.clone();
    assert_eq!(range_search(&mut w, a), None);
    assert_eq!(w.rng, draws(&before, &w, false), "friendly row: no draw");
}

// spec: S-0309 table C row harvester immunity and scenario A14 part i (the map sets harvester immunity and T's type is a game-wide harvester type: never found by itself; on a map without the setting, found; a vehicle of another type is found on either map)
#[test]
fn table_c_harvester_immunity() {
    let mut r = base();
    r.sim.general.harvester_unit = Some(vec!["T1".into()]);
    for (immune, listed_found) in [(false, true), (true, false)] {
        let (mut w, a, b) = one_target(&r, Kind::Vehicle, "T1");
        w.map_flags.harvester_immune = immune;
        let got = range_search(&mut w, a);
        assert_eq!(got, listed_found.then_some(b), "immune {immune}");
    }
    // The flag alone rejects nothing: T1 is no harvester type.
    r.sim.general.harvester_unit = None;
    let (mut w, a, b) = one_target(&r, Kind::Vehicle, "T1");
    w.map_flags.harvester_immune = true;
    assert_eq!(range_search(&mut w, a), Some(b));
}
