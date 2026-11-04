//! Claim tests of S-0309 revision 70: claim 29, the periodic held-target
//! check, with the changes it makes to claims 4 and 19 and its Changes for
//! S-0304 claim 2 and S-0310 claim 11 (M8 step A3 intake 68). Synthetic
//! data only: every type, weapon, warhead and number is invented.
//!
//! A drop in tick *k* shows after `tick::step` has run that tick, when the
//! tick number reads *k* + 1. An order issued between two steps while the
//! tick number reads *n* is applied after the turn pass of tick *n*
//! (S-0203 claim 7, S-0213 claim 1), so that tick is its tick of
//! application. The ledger is `docs/m8-claims/intake68-S-0309.md`.

use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{
    ArmourClass, Decimal, ProjectileType, UnitType, WarheadType, WeaponType,
};
use glasnost_sim::combat::check::{self, Answer};
use glasnost_sim::combat::effects::Effect;
use glasnost_sim::combat::held;
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, Target, World};

const SIZE: i32 = 24;
const ALPHA: u8 = 0;
const BETA: u8 = 1;
/// The neutral house of `testkit::game_with_non_players`.
const NEUTRAL: u8 = 2;
/// The check period of claim 29, written out for the expected ticks.
const PERIOD: i32 = 16;

fn dec(s: &str) -> Decimal {
    Decimal(s.into())
}

/// Eleven versus values: `v` for class `class`, 1 for the others.
fn only(class: usize, v: &str) -> Option<Vec<Decimal>> {
    let mut l = vec![dec("1"); 11];
    l[class] = dec(v);
    Some(l)
}

/// The armour class *light*'s index in the versus list.
const LIGHT: usize = 3;

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
    t.secondary = None;
    t.combat.armor = None;
    f(&mut t);
    register(r, "vehicles", name);
    r.sim.vehicles.insert(name.into(), t);
}

fn infantry(r: &mut Ruleset, name: &str, f: impl FnOnce(&mut UnitType)) {
    let mut t = r.sim.infantry["WALKER"].clone();
    t.strength = Some(1000);
    t.primary = None;
    t.secondary = None;
    f(&mut t);
    register(r, "infantry", name);
    r.sim.infantry.insert(name.into(), t);
}

fn warhead(r: &mut Ruleset, name: &str, verses: Option<Vec<Decimal>>) {
    r.sim.warheads.insert(
        name.into(),
        WarheadType {
            verses,
            ..Default::default()
        },
    );
}

/// A direct weapon of range 5 cells firing every 30 ticks.
fn weapon(r: &mut Ruleset, name: &str, warhead_name: &str, damage: i32, projectile: &str) {
    r.sim.weapons.insert(
        name.into(),
        WeaponType {
            damage: Some(damage),
            rof: Some(30),
            range: Some(dec("5")),
            speed: Some(100),
            projectile: Some(projectile.into()),
            warhead: Some(warhead_name.into()),
            ..Default::default()
        },
    );
}

/// The rules of this file:
/// - warhead `H`, 0 against class *light* and 1 elsewhere (S-0304 V1's
///   test warhead), with weapon `W`; warhead `PLAIN`, 1 everywhere, with
///   weapon `Plain`; weapon `AirOnly`, an anti-air projectile that is not
///   anti-ground; weapon `Mend`, a healing weapon of warhead `HEAL0`
///   (0 against *light*);
/// - vehicles `DEF` (primary `W`), `TWO` (primary `W`, secondary `Plain`),
///   `SKY` (primary `AirOnly`), `BARE` (no weapon), `TARGET` (class
///   *light*, unarmed), `PLAINV` (class *heavy*, unarmed), `MENDER`
///   (primary `Mend`);
/// - infantry `GI` (primary `W`), `MEDIC` (primary `Mend`), `HURT` (class
///   *light*);
/// - building `POST` (the testkit's tower with primary `W`).
fn rules() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    warhead(&mut r, "H", only(LIGHT, "0"));
    warhead(&mut r, "PLAIN", only(LIGHT, "1"));
    warhead(&mut r, "HEAL0", only(LIGHT, "0"));
    weapon(&mut r, "W", "H", 20, "Bullet");
    weapon(&mut r, "Plain", "PLAIN", 20, "Bullet");
    weapon(&mut r, "Mend", "PLAIN", -20, "Bullet");
    weapon(&mut r, "Mend0", "HEAL0", -20, "Bullet");
    r.sim.projectiles.insert(
        "Sky".into(),
        ProjectileType {
            inviso: Some(true),
            aa: Some(true),
            ag: Some(false),
            ..Default::default()
        },
    );
    weapon(&mut r, "AirOnly", "PLAIN", 20, "Sky");
    vehicle(&mut r, "DEF", |t| t.primary = Some("W".into()));
    vehicle(&mut r, "TWO", |t| {
        t.primary = Some("W".into());
        t.secondary = Some("Plain".into());
    });
    vehicle(&mut r, "SKY", |t| t.primary = Some("AirOnly".into()));
    vehicle(&mut r, "BARE", |_| {});
    vehicle(&mut r, "TARGET", |t| {
        t.combat.armor = Some(ArmourClass::Light)
    });
    vehicle(&mut r, "PLAINV", |t| {
        t.combat.armor = Some(ArmourClass::Heavy)
    });
    vehicle(&mut r, "MENDER", |t| t.primary = Some("Mend0".into()));
    infantry(&mut r, "GI", |t| t.primary = Some("W".into()));
    infantry(&mut r, "MEDIC", |t| t.primary = Some("Mend0".into()));
    infantry(&mut r, "HURT", |t| {
        t.combat.armor = Some(ArmourClass::Light)
    });
    let mut b = r.sim.buildings["TOWER"].clone();
    b.primary = Some("W".into());
    register(&mut r, "buildings", "POST");
    r.sim.buildings.insert("POST".into(), b);
    r
}

fn at(dx: i32, dy: i32) -> CellXY {
    testkit::mid(SIZE, dx, dy)
}

fn put(w: &mut World, kind: Kind, name: &str, house: u8, dx: i32, dy: i32) -> ObjectId {
    testkit::spawn(w, kind, name, house, at(dx, dy), 0)
}

fn target(w: &World, id: ObjectId) -> Option<Target> {
    w.obj(id).and_then(|o| o.target)
}

fn step(w: &mut World) {
    glasnost_sim::tick::step(w);
}

/// The first check tick after tick `n` (the tick of application).
fn next_check(n: i32) -> i32 {
    (n / PERIOD + 1) * PERIOD
}

/// A game of this file's rules with holder `name` of kind `kind` for Alpha
/// and the enemy vehicle `TARGET` (X, for which `W` answers *illegal*)
/// 3 cells away, within `W`'s range.
fn duel(kind: Kind, name: &str) -> (World, ObjectId, ObjectId) {
    let mut w = testkit::game_with(&rules(), SIZE, 1);
    let d = put(&mut w, kind, name, ALPHA, -1, 0);
    let x = put(&mut w, Kind::Vehicle, "TARGET", BETA, 2, 0);
    (w, d, x)
}

fn attack(actor: ObjectId, x: ObjectId) -> Order {
    Order::Behaviour {
        actor,
        behaviour: Behaviour::Attack,
        attack_move: false,
        target: Some(Target::Object(x)),
        dest: None,
    }
}

/// Runs to a tick whose number leaves `rem` when divided by 16 and in which
/// orders are taken, and issues `orders` there; returns that tick, the tick
/// of application.
fn order_at(w: &mut World, rem: i32, orders: &[(ObjectId, ObjectId)]) -> i32 {
    loop {
        if !orders::blocked(w) && w.tick.rem_euclid(PERIOD) == rem {
            for &(a, x) in orders {
                assert!(orders::issue(w, HouseId(ALPHA), attack(a, x)));
            }
            return w.tick;
        }
        step(w);
    }
}

/// Runs whole ticks, at most `limit`, until `id` no longer holds `x`;
/// returns the tick in which it dropped it.
fn drop_tick(w: &mut World, id: ObjectId, x: ObjectId, limit: i32) -> Option<i32> {
    for _ in 0..limit {
        step(w);
        if target(w, id) != Some(Target::Object(x)) {
            return Some(w.tick - 1);
        }
    }
    None
}

fn shots_by(w: &World, id: ObjectId) -> usize {
    w.effects
        .iter()
        .filter(|e| matches!(e, Effect::Shot { firer, .. } if *firer == id))
        .count()
}

/// Makes `id`'s behaviour processing not due for a long time, so that a
/// tick isolates the check from the processing's own drops.
fn no_processing(w: &mut World, id: ObjectId) {
    let t = w.tick;
    w.obj_mut(id).unwrap().beh.processed(t, 1000);
}

/// Gives `id` target `x` and behaviour `b` directly (as an order or a
/// search would), its targeting timer 200 ticks away.
fn hold(w: &mut World, id: ObjectId, x: Target, b: Behaviour, opportunity: bool) {
    let t = w.tick;
    let o = w.obj_mut(id).unwrap();
    o.beh.force(b, t, false);
    o.target = Some(x);
    o.combat.opportunity = opportunity;
    o.targeting_timer = t + 200;
}

// ---------------------------------------------------------------------------
// Run p1: the cadence, the drop of an ordered illegal target, no timer draw.
// ---------------------------------------------------------------------------

/// Run p1 for one holder kind: the order applied in ticks with remainders
/// 1, 5, 9 and 13 is dropped 15, 11, 7 and 3 ticks later, in the holder's
/// turn of the first check tick, with no shot and the targeting timer
/// counting on without a reset.
fn run_p1(kind: Kind, name: &str) {
    for (rem, after) in [(1, 15), (5, 11), (9, 7), (13, 3)] {
        let (mut w, d, x) = duel(kind, name);
        w.obj_mut(d).unwrap().targeting_timer = i32::MAX / 2;
        let n = order_at(&mut w, rem, &[(d, x)]);
        // The timer far away: more than 10 ticks left at the drop tick, where claim 27's reset would draw.
        w.obj_mut(d).unwrap().targeting_timer = n + 200;
        w.effects.clear();
        step(&mut w);
        assert_eq!(target(&w, d), Some(Target::Object(x)), "{name} {rem}: held");
        let dropped = drop_tick(&mut w, d, x, 40);
        // spec: S-0309 claim 29 (Phase: a refusal that begins after the holder's turn in tick t, as with an order applied after the turn pass, is acted on 1 to 16 ticks after t, at the first check tick) and scenario A6 part p run p1 (15, 11, 7 and 3 ticks after the tick of application)
        assert_eq!(dropped, Some(n + after), "{name} {rem}");
        assert_eq!(dropped, Some(next_check(n)), "{name} {rem}");
        // spec: S-0309 claim 29 (The drop: no targeting-timer draw; claim 27's reset belongs only to the drops of claims 27 and 28) and A6 part p run p1 (the timer counts on without a reset)
        assert_eq!(
            w.obj(d).unwrap().targeting_timer,
            n + 200,
            "{name} {rem}: no reset"
        );
        // spec: S-0304 claim 2 and S-0309 A6 part p run p1 (D makes no shot at X)
        assert_eq!(shots_by(&w, d), 0, "{name} {rem}");
    }
}

#[test]
fn p1_an_ordered_illegal_target_is_dropped_at_the_first_check_tick_by_a_vehicle() {
    run_p1(Kind::Vehicle, "DEF");
}

#[test]
fn p1_an_ordered_illegal_target_is_dropped_at_the_first_check_tick_by_infantry() {
    run_p1(Kind::Infantry, "GI");
}

// spec: S-0309 claim 29 (The drop changes no behaviour by itself: the behaviour's next processing does what it does with no target, so an object in attack stops attacking at that processing) and A6 part p run p1
#[test]
fn p1_the_attack_ends_at_the_processing_after_the_drop() {
    let (mut w, d, x) = duel(Kind::Vehicle, "DEF");
    let n = order_at(&mut w, 3, &[(d, x)]);
    w.obj_mut(d).unwrap().targeting_timer = n + 200;
    // The order queues attack; it becomes current at the guard's next processing, before the check tick.
    while w.obj(d).unwrap().beh.current != Some(Behaviour::Attack) {
        step(&mut w);
        assert!(
            w.tick <= next_check(n),
            "attack current before the check tick"
        );
    }
    assert_eq!(target(&w, d), Some(Target::Object(x)));
    // Hold the processing back over the drop tick.
    no_processing(&mut w, d);
    assert_eq!(drop_tick(&mut w, d, x, 40), Some(next_check(n)));
    assert_eq!(
        w.obj(d).unwrap().beh.current,
        Some(Behaviour::Attack),
        "the drop itself changes no behaviour"
    );
    // The next processing, with no target, falls back to rest.
    w.obj_mut(d).unwrap().beh.expired = true;
    step(&mut w);
    assert_ne!(w.obj(d).unwrap().beh.current, Some(Behaviour::Attack));
}

// ---------------------------------------------------------------------------
// Runs p2 to p5: the answers and their ranks.
// ---------------------------------------------------------------------------

// spec: S-0309 claim 29 (Answers: while the rearm countdown runs, rank 5, a target for which only a rank-9 reason holds, a zero versus value, is kept, and it is dropped at the first check tick after the countdown has run out) and A6 part p run p2
#[test]
fn p2_a_running_countdown_keeps_an_illegal_target_to_the_first_check_tick_after_it() {
    let (mut w, d, x) = duel(Kind::Vehicle, "DEF");
    let n = order_at(&mut w, 1, &[(d, x)]);
    w.obj_mut(d).unwrap().targeting_timer = n + 200;
    // The countdown runs while the tick number is below n + 40: check ticks n + 15 and n + 31 fall in it.
    w.obj_mut(d).unwrap().combat.rearm_until = n + 40;
    let dropped = drop_tick(&mut w, d, x, 80);
    assert_eq!(
        dropped,
        Some(n + 47),
        "the first check tick at or after n + 40"
    );
    assert_eq!(dropped, Some(next_check(n + 39)));
}

// spec: S-0309 claim 29 (Answers: while the holder's rearm countdown runs, rank 5, a target for which only a rank-9 reason holds is kept, and it is dropped at the first check tick after the countdown has run out; the rank-5 keep for an infantry holder, rated low, run p2's holder being a vehicle) (M8 seam sweep 5a)
#[test]
fn p2_the_rank_5_keep_for_an_infantry_holder() {
    let (mut w, d, x) = duel(Kind::Infantry, "GI");
    let n = order_at(&mut w, 1, &[(d, x)]);
    w.obj_mut(d).unwrap().targeting_timer = n + 200;
    // The countdown runs while the tick number is below n + 40: check ticks n + 15 and n + 31 fall in it.
    w.obj_mut(d).unwrap().combat.rearm_until = n + 40;
    let dropped = drop_tick(&mut w, d, x, 80);
    assert_eq!(
        dropped,
        Some(next_check(n + 39)),
        "kept past n + 15 and n + 31"
    );
    assert_eq!(dropped, Some(n + 47));
    // Control: with no countdown the same holder drops at the first check tick.
    let (mut c, d2, x2) = duel(Kind::Infantry, "GI");
    let n2 = order_at(&mut c, 1, &[(d2, x2)]);
    c.obj_mut(d2).unwrap().targeting_timer = n2 + 200;
    assert_eq!(drop_tick(&mut c, d2, x2, 80), Some(next_check(n2)));
}

// spec: S-0309 claim 29 (Answers: a vehicle launching spawned aircraft, rank 2 busy, keeps its target, a rank-9 refusal included; S-0301 claim 9 rank 2 and S-0506 claim 10, a launching count above 0) (M8 seam sweep 5a)
#[test]
fn a_launching_vehicle_keeps_an_illegal_target_through_check_ticks() {
    use glasnost_sim::spawn::{Slot, SlotState, SpawnController};
    let (mut w, d, x) = duel(Kind::Vehicle, "DEF");
    let n = order_at(&mut w, 1, &[(d, x)]);
    w.obj_mut(d).unwrap().targeting_timer = n + 200;
    let rail = SpawnController {
        slots: vec![Slot {
            state: SlotState::OnRail,
            object: None,
            until: 0,
        }],
        ..Default::default()
    };
    // Three check ticks: n + 15, n + 31 and n + 47.
    while w.tick <= next_check(n) + 2 * PERIOD {
        // One slot on its launch rail before each tick: the launching count is 1.
        w.obj_mut(d).unwrap().spawn = Some(rail.clone());
        assert_eq!(glasnost_sim::spawn::launching_count(&w, d), 1);
        assert_eq!(
            check::check(&w, d, Some(Target::Object(x)), 0, false),
            Answer::Busy
        );
        step(&mut w);
        assert_eq!(
            target(&w, d),
            Some(Target::Object(x)),
            "kept at tick {}",
            w.tick - 1
        );
    }
    // Control: with the rail empty the next check tick drops it.
    w.obj_mut(d).unwrap().spawn = None;
    let at = w.tick;
    assert_eq!(drop_tick(&mut w, d, x, 40), Some(next_check(at - 1)));
}

// spec: S-0309 claim 29 (Answers: a holder whose chosen weapon is not refused keeps the target, even if its other weapon would be; the weapon it would use, S-0308) and A6 part p run p3
#[test]
fn p3_the_weapon_it_would_use_decides() {
    let (mut w, d, x) = duel(Kind::Vehicle, "TWO");
    let t = Target::Object(x);
    let (slot, _) = glasnost_sim::combat::weapon::would_use(&w, d, Some(t));
    assert_eq!(
        slot, 1,
        "S-0308 table P row 16: the primary's versus value is zero"
    );
    assert_eq!(check::check(&w, d, Some(t), 0, false), Answer::Illegal);
    let n = order_at(&mut w, 1, &[(d, x)]);
    w.obj_mut(d).unwrap().targeting_timer = n + 200;
    w.effects.clear();
    assert_eq!(drop_tick(&mut w, d, x, 100), None, "kept for 100 ticks");
    assert!(shots_by(&w, d) > 0, "fires with its second weapon");
}

// spec: S-0309 claim 29 (Answers: a rank-4 refusal outranks the countdown, so that target is dropped at the first check tick whatever the countdown shows) and A6 part p run p4; the refusal here is S-0306 claim 23's cell row (a projectile that is not anti-ground against a cell), another rank-4 illegal
#[test]
fn p4_a_rank_4_refusal_is_dropped_while_the_countdown_runs() {
    let (mut w, d, _) = duel(Kind::Vehicle, "SKY");
    step(&mut w);
    let n = w.tick;
    let c = Target::Cell(at(2, 2));
    hold(&mut w, d, c, Behaviour::Attack, false);
    w.obj_mut(d).unwrap().combat.rearm_until = n + 200;
    assert_eq!(check::check(&w, d, Some(c), 0, false), Answer::Illegal);
    let mut dropped = None;
    for _ in 0..40 {
        step(&mut w);
        if target(&w, d) != Some(c) {
            dropped = Some(w.tick - 1);
            break;
        }
    }
    assert_eq!(dropped, Some(next_check(n - 1)));
    assert!(w.tick < n + 200, "while the countdown runs");
}

// spec: S-0309 claim 29 (If the answer is impossible, it drops the target) and A6 part p run p5 (the drop on impossible); the impossible here is S-0301 claim 9's rank-4 empty slot, a holder without a weapon; the target came from a search (an opportunity target, claim 19)
#[test]
fn p5_an_impossible_answer_drops_the_target_at_the_check_tick() {
    let (mut w, d, x) = duel(Kind::Vehicle, "BARE");
    while w.tick.rem_euclid(PERIOD) != 6 {
        step(&mut w);
    }
    let n = w.tick;
    hold(&mut w, d, Target::Object(x), Behaviour::Guard, true);
    assert_eq!(
        check::check(&w, d, Some(Target::Object(x)), 0, false),
        Answer::Impossible
    );
    assert_eq!(drop_tick(&mut w, d, x, 40), Some(n + 10));
    assert_eq!(w.obj(d).unwrap().targeting_timer, n + 200, "no reset");
}

// ---------------------------------------------------------------------------
// Run p6 and part m: the shared phase; a searched target in guard.
// ---------------------------------------------------------------------------

/// Run p6 for one holder kind: a holder made in a tick off the check phase
/// drops in the same tick as one made at the start.
fn run_p6(kind: Kind, name: &str) {
    for made in [5, 11] {
        let (mut w, d, x) = duel(kind, name);
        while w.tick.rem_euclid(PERIOD) != made {
            step(&mut w);
        }
        let e = put(&mut w, kind, name, ALPHA, -1, 1);
        let n = order_at(&mut w, 1, &[(d, x), (e, x)]);
        step(&mut w);
        assert_eq!(target(&w, d), Some(Target::Object(x)));
        assert_eq!(target(&w, e), Some(Target::Object(x)));
        let mut first = [None, None];
        for _ in 0..20 {
            step(&mut w);
            for (i, id) in [d, e].into_iter().enumerate() {
                if first[i].is_none() && target(&w, id).is_none() {
                    first[i] = Some(w.tick - 1);
                }
            }
        }
        // spec: S-0309 claim 29 (Phase: every object checks in the same ticks) and A6 part p run p6 (D and E drop X in the same tick, 15 ticks after the order's tick)
        assert_eq!(
            first,
            [Some(n + 15), Some(n + 15)],
            "{name}, made at remainder {made}"
        );
    }
}

#[test]
fn p6_holders_made_in_different_ticks_check_in_the_same_ticks() {
    run_p6(Kind::Vehicle, "DEF");
    run_p6(Kind::Infantry, "GI");
}

// spec: S-0309 claim 29 (the check is the same whichever of an order or a search gave the target; a holder in guard makes it) and claim 19 (an opportunity target the firing check answers illegal for is dropped at a check tick if that comes before the next search) and A6 part m (no later than the first check tick after the refusal begins)
#[test]
fn m_a_searched_target_in_guard_is_dropped_at_the_check_tick_before_the_next_search() {
    for kind in [(Kind::Vehicle, "DEF"), (Kind::Infantry, "GI")] {
        let (mut w, d, x) = duel(kind.0, kind.1);
        while w.tick.rem_euclid(PERIOD) != 3 {
            step(&mut w);
        }
        let n = w.tick;
        hold(&mut w, d, Target::Object(x), Behaviour::Guard, true);
        assert_eq!(drop_tick(&mut w, d, x, 40), Some(n + 13), "{}", kind.1);
    }
}

// ---------------------------------------------------------------------------
// Behaviours, holders, and the drop's consequences.
// ---------------------------------------------------------------------------

/// Whether a holder in behaviour `b` (its processing held back) still holds
/// an illegal target at the end of a check tick.
fn kept_in(b: Behaviour) -> bool {
    let (mut w, d, x) = duel(Kind::Vehicle, "DEF");
    while w.tick.rem_euclid(PERIOD) != PERIOD - 1 {
        step(&mut w);
    }
    hold(&mut w, d, Target::Object(x), b, false);
    no_processing(&mut w, d);
    step(&mut w);
    step(&mut w);
    assert_eq!((w.tick - 1).rem_euclid(PERIOD), 0, "a check tick ran");
    target(&w, d) == Some(Target::Object(x))
}

// spec: S-0309 claim 29 (Behaviours: a holder in attack or guard makes the check; one in capture or sabotage makes none)
#[test]
fn capture_and_sabotage_make_no_check() {
    assert!(!kept_in(Behaviour::Attack));
    assert!(!kept_in(Behaviour::Guard));
    assert!(kept_in(Behaviour::Capture));
    assert!(kept_in(Behaviour::Sabotage));
}

// spec: S-0309 claim 29 revision 94 (Behaviours, Q-2262 answered: a holder makes the check in every behaviour but capture and sabotage)
#[test]
fn other_behaviours_make_the_check() {
    for b in [
        Behaviour::Hunt,
        Behaviour::AreaGuard,
        Behaviour::Sticky,
        Behaviour::Move,
    ] {
        assert!(!kept_in(b), "{b:?}");
    }
}

// spec: S-0309 claim 29 (every building of a human-controlled owner holding a target in guard makes the check; its own attack processing, S-0215 claim 32, is held back here so the periodic drop shows alone)
#[test]
fn a_building_drops_an_illegal_target_at_the_check_tick() {
    let mut w = testkit::game_with(&rules(), SIZE, 1);
    let b = put(&mut w, Kind::Building, "POST", ALPHA, -1, 0);
    let x = put(&mut w, Kind::Vehicle, "TARGET", BETA, 2, 0);
    while w.tick.rem_euclid(PERIOD) != 9 {
        step(&mut w);
    }
    let n = w.tick;
    hold(&mut w, b, Target::Object(x), Behaviour::Guard, false);
    no_processing(&mut w, b);
    assert_eq!(drop_tick(&mut w, b, x, 40), Some(n + 7));
}

// spec: S-0309 claim 29 (Place in the turn: a building makes the check before its pending delayed shot's remaining delay counts down, so in the tick that ends the delay the pending shot still answers rearming and the target is kept, as S-0301 scenario F21 requires) and S-0301 claim 14 (the end of the delay: the check again, with range, and no shot on illegal)
#[test]
fn a_pending_delayed_shot_ending_in_a_check_tick_keeps_the_target() {
    let mut w = testkit::game_with(&rules(), SIZE, 1);
    let b = put(&mut w, Kind::Building, "POST", ALPHA, -1, 0);
    let x = put(&mut w, Kind::Vehicle, "TARGET", BETA, 2, 0);
    while w.tick.rem_euclid(PERIOD) != 12 {
        step(&mut w);
    }
    let k = next_check(w.tick);
    hold(&mut w, b, Target::Object(x), Behaviour::Attack, false);
    no_processing(&mut w, b);
    w.obj_mut(b).unwrap().combat.delayed_shot = Some(k);
    w.effects.clear();
    while w.tick <= k {
        step(&mut w);
    }
    assert_eq!(w.tick - 1, k);
    let o = w.obj(b).unwrap();
    assert_eq!(
        o.combat.delayed_shot, None,
        "the delay ended in the check tick"
    );
    assert_eq!(o.target, Some(Target::Object(x)), "kept at the check tick");
    assert_eq!(shots_by(&w, b), 0, "no shot at an illegal target");
    // The control: with no pending shot the same tick drops it.
    let mut w = testkit::game_with(&rules(), SIZE, 1);
    let b = put(&mut w, Kind::Building, "POST", ALPHA, -1, 0);
    let x = put(&mut w, Kind::Vehicle, "TARGET", BETA, 2, 0);
    while w.tick.rem_euclid(PERIOD) != 12 {
        step(&mut w);
    }
    let k = next_check(w.tick);
    hold(&mut w, b, Target::Object(x), Behaviour::Attack, false);
    no_processing(&mut w, b);
    assert_eq!(drop_tick(&mut w, b, x, 40), Some(k));
}

// spec: S-0309 claim 29 (The drop causes no revert, S-0215 claim 10: clearing the target by another rule does not revert)
#[test]
fn the_drop_reverts_no_override() {
    let (mut w, d, x) = duel(Kind::Vehicle, "DEF");
    let y = put(&mut w, Kind::Vehicle, "PLAINV", BETA, 2, 2);
    while w.tick.rem_euclid(PERIOD) != 10 {
        step(&mut w);
    }
    let n = w.tick;
    hold(&mut w, d, Target::Object(y), Behaviour::Guard, false);
    {
        let o = w.obj_mut(d).unwrap();
        // An override in effect: guard suspended with Y put aside, attack on X current.
        o.beh.aside = Some((Some(Target::Object(y)), None));
        assert!(o.beh.override_with(Behaviour::Attack, false, false));
        o.target = Some(Target::Object(x));
    }
    no_processing(&mut w, d);
    assert_eq!(drop_tick(&mut w, d, x, 40), Some(n + 6));
    let o = w.obj(d).unwrap();
    assert_eq!(o.target, None, "no target until something gives it one");
    assert_eq!(o.beh.suspended, Some(Behaviour::Guard), "no revert");
    assert_eq!(o.beh.aside, Some((Some(Target::Object(y)), None)));
    assert_eq!(o.beh.current, Some(Behaviour::Attack));
}

// spec: S-0309 claim 29 (the drop takes no draw: State and ordering, claim 29's periodic check and its drops take no draw)
#[test]
fn the_check_and_its_drop_take_no_draw() {
    let (mut w, d, x) = duel(Kind::Vehicle, "DEF");
    while w.tick.rem_euclid(PERIOD) != PERIOD - 1 {
        step(&mut w);
    }
    step(&mut w);
    hold(&mut w, d, Target::Object(x), Behaviour::Guard, false);
    no_processing(&mut w, d);
    // The control: the same world in which the holder holds nothing, as it does after the drop.
    let mut control = w.clone();
    control.obj_mut(d).unwrap().target = None;
    assert!(held::check_tick(w.tick));
    step(&mut w);
    step(&mut control);
    assert_eq!(target(&w, d), None, "dropped");
    assert_eq!(w.rng, control.rng, "the check and the drop took no draw");
}

// spec: S-0309 claim 29 revision 94 (Owners and targets, Q-2263 answered: every holder of every house, the neutral and special houses included) and r94's Changes for S-0304 claim 2 (the attack processing's drop is a building's only: a unit of any owner drops an illegal target at the periodic check, not at its attack processing)
#[test]
fn a_neutral_holder_makes_the_check_and_no_processing_drop() {
    let mut w = testkit::game_with_non_players(&rules(), SIZE, 1);
    let d = put(&mut w, Kind::Vehicle, "DEF", NEUTRAL, -1, 0);
    let x = put(&mut w, Kind::Vehicle, "TARGET", BETA, 2, 0);
    // In guard, with its processing held back, it drops the target at the check tick.
    while w.tick.rem_euclid(PERIOD) != PERIOD - 1 {
        step(&mut w);
    }
    hold(&mut w, d, Target::Object(x), Behaviour::Guard, false);
    no_processing(&mut w, d);
    step(&mut w);
    step(&mut w);
    assert_eq!(target(&w, d), None, "dropped at the periodic check");
    // In attack, its next processing keeps it; the check tick drops it.
    while w.tick.rem_euclid(PERIOD) != 4 {
        step(&mut w);
    }
    let n = w.tick;
    hold(&mut w, d, Target::Object(x), Behaviour::Attack, false);
    w.obj_mut(d).unwrap().beh.expired = true;
    assert_eq!(drop_tick(&mut w, d, x, 40), Some(n + 12));
    // A player holder's alike.
    let (mut w, p, x) = duel(Kind::Vehicle, "DEF");
    while w.tick.rem_euclid(PERIOD) != 4 {
        step(&mut w);
    }
    let n = w.tick;
    hold(&mut w, p, Target::Object(x), Behaviour::Attack, false);
    w.obj_mut(p).unwrap().beh.expired = true;
    assert_eq!(drop_tick(&mut w, p, x, 40), Some(n + 12));
}

// ---------------------------------------------------------------------------
// Healers (S-0310 r8 claim 11, from S-0309 r70's Changes for S-0310).
// ---------------------------------------------------------------------------

// spec: S-0310 r8 claim 11 (an infantry unit whose default weapon heals replies to illegal in its own firing step, in each of its turns: it drops a target that is not infantry, also off the check phase) and S-0310 T10 parts g and h
#[test]
fn an_infantry_healer_drops_a_vehicle_target_in_its_firing_step() {
    let mut w = testkit::game_with(&rules(), SIZE, 1);
    let h = put(&mut w, Kind::Infantry, "MEDIC", ALPHA, -1, 0);
    let v = put(&mut w, Kind::Vehicle, "PLAINV", ALPHA, 1, 0);
    w.obj_mut(v).unwrap().health = 500;
    while w.tick.rem_euclid(PERIOD) != 2 {
        step(&mut w);
    }
    let n = w.tick;
    hold(&mut w, h, Target::Object(v), Behaviour::Guard, false);
    no_processing(&mut w, h);
    assert_eq!(drop_tick(&mut w, h, v, 10), Some(n), "in its first turn");
}

// spec: S-0310 r8 claim 11 (an infantry healer keeps an infantry target below full health that still gives illegal, here by a versus value of exactly 0, and does nothing in that step; that keep lasts only to S-0309 claim 29's next check) and S-0309 claim 29
#[test]
fn an_infantry_healers_keep_lasts_to_the_check_tick() {
    let mut w = testkit::game_with(&rules(), SIZE, 1);
    let h = put(&mut w, Kind::Infantry, "MEDIC", ALPHA, -1, 0);
    let y = put(&mut w, Kind::Infantry, "HURT", ALPHA, 1, 0);
    w.obj_mut(y).unwrap().health = 500;
    while w.tick.rem_euclid(PERIOD) != 2 {
        step(&mut w);
    }
    let n = w.tick;
    hold(&mut w, h, Target::Object(y), Behaviour::Guard, false);
    no_processing(&mut w, h);
    assert_eq!(
        check::check(&w, h, Some(Target::Object(y)), 0, true),
        Answer::Illegal
    );
    w.effects.clear();
    assert_eq!(drop_tick(&mut w, h, y, 40), Some(n + 14));
    assert_eq!(shots_by(&w, h), 0);
}

// spec: S-0310 r8 claim 11 and scenario T10 part d (a vehicle healer's keep of a vehicle target below full health on illegal, "keeps it and does nothing in this step", lasts only to S-0309 claim 29's next check: at the first tick whose tick number is a multiple of 16 at which the check answers illegal, it drops it)
#[test]
fn a_vehicle_healers_keep_lasts_to_the_check_tick() {
    let mut w = testkit::game_with(&rules(), SIZE, 1);
    let h = put(&mut w, Kind::Vehicle, "MENDER", ALPHA, -1, 0);
    let y = put(&mut w, Kind::Vehicle, "TARGET", ALPHA, 1, 0);
    w.obj_mut(y).unwrap().health = 500;
    while w.tick.rem_euclid(PERIOD) != 7 {
        step(&mut w);
    }
    let n = w.tick;
    hold(&mut w, h, Target::Object(y), Behaviour::Guard, false);
    no_processing(&mut w, h);
    assert_eq!(drop_tick(&mut w, h, y, 40), Some(n + 9));
}
