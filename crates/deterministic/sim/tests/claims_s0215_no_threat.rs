//! S-0215 r118 claim 16 (M8 seam, intake 99): the no-threat fall-back on
//! damage. A mobile actor that is not a team member, whose current behaviour
//! at that point of the hit has the no-threat property and not the zombie
//! property, falls back to rest at once (claims 13 to 15) after a hit that
//! names an attacker, lowers its health and reports unchanged, now yellow or
//! now red. The test comes after the retaliation (claim 18), Q-2586 answered.
//! Every hit here is a real shot: the firer's weapon fires a projectile
//! (`combat::fire::shot`) or a player's attack order makes it fire, and the
//! projectile detonates in the tick pipeline. Synthetic data only (the
//! combat ruleset of `testkit` and the lift fixture), behaviour files
//! invented after scenarios B5 and B29; no stock value. Ledger:
//! `docs/m8-claims/seam-s0215-c16.md`.

#[path = "lift_common/mod.rs"]
mod lift_common;

use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::BehaviourProps;
use glasnost_sim::combat::damage::{self, Hit};
use glasnost_sim::combat::fire;
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit::{self, mid, spawn};
use glasnost_sim::world::{HouseId, ObjectId, Target, World};

const SIZE: i32 = 40;

/// The behaviour file of scenario B5: `harmless` gets the no-threat
/// property and, with `zombie`, the zombie property too.
fn harmless(r: &mut Ruleset, no_threat: bool, zombie: bool) {
    r.sim.behaviours.insert(
        "harmless".into(),
        BehaviourProps {
            no_threat: Some(no_threat),
            zombie: Some(zombie),
            ..Default::default()
        },
    );
}

/// The combat ruleset with `harmless` set as given.
fn rules(no_threat: bool, zombie: bool) -> Ruleset {
    let mut r = testkit::combat_ruleset();
    harmless(&mut r, no_threat, zombie);
    r
}

fn force(w: &mut World, id: ObjectId, b: Behaviour) {
    let t = w.tick;
    let stale = w.rules().quirks.stale_suspended_revert;
    w.obj_mut(id).unwrap().beh.force(b, t, stale);
}

/// The three behaviour slots of `id`.
fn slots(w: &World, id: ObjectId) -> (Option<Behaviour>, Option<Behaviour>, Option<Behaviour>) {
    let o = w.obj(id).unwrap();
    (o.beh.current, o.beh.queued, o.beh.suspended)
}

/// `firer` holds `target` as its target, as a firer does when it shoots (an
/// arcing shell's pitch is taken towards the firer's target, S-0306 claim 6).
fn aim(w: &mut World, firer: ObjectId, target: ObjectId) {
    w.obj_mut(firer).unwrap().target = Some(Target::Object(target));
}

/// `firer` fires its primary weapon at `target`; runs until the shot has
/// detonated (at most 60 ticks). Returns the tick of the detonation.
fn shoot(w: &mut World, firer: ObjectId, target: ObjectId) -> i32 {
    aim(w, firer, target);
    assert!(fire::shot(w, firer, 0, Target::Object(target)), "a shot");
    let t0 = w.tick;
    while !w.projectiles.is_empty() {
        testkit::run(w, 1);
        assert!(w.tick < t0 + 60, "the shot did not detonate");
    }
    if let Some(o) = w.obj_mut(firer) {
        o.target = None;
    }
    w.tick
}

fn health(w: &World, id: ObjectId) -> i32 {
    w.obj(id).map_or(0, |o| o.health)
}

/// The tick of `id`'s last fall-back common effects (claim 13): the trace a
/// fall-back to rest leaves even when its table row queues nothing.
fn common_effects(w: &World, id: ObjectId) -> Option<i32> {
    w.obj(id).and_then(|o| o.beh.last_common_effects)
}

/// The no-threat subject `S` (house 1, behaviour *harmless*), an enemy `G`
/// of house 0 in *guard* next to it, and the third unit `A` of house 0 that
/// damages it (scenario B5). `kind` and `name` give S.
fn b5(r: &Ruleset, kind: Kind, name: &str) -> (World, ObjectId, ObjectId, ObjectId) {
    let mut w = testkit::game_with(r, SIZE, 3);
    let s = spawn(&mut w, kind, name, 1, mid(SIZE, 0, 0), 0);
    force(&mut w, s, Behaviour::Harmless);
    let g = spawn(&mut w, Kind::Vehicle, "CART", 0, mid(SIZE, 2, 0), 192);
    let a = spawn(&mut w, Kind::Vehicle, "CART", 0, mid(SIZE, -3, 0), 64);
    (w, s, g, a)
}

/// No unit of house 0 targets `s` over `n` ticks.
fn never_targeted(w: &mut World, s: ObjectId, by: &[ObjectId], n: i32) -> bool {
    for _ in 0..n {
        testkit::run(w, 1);
        if by.iter().any(|&x| {
            w.obj(x)
                .is_some_and(|o| o.target == Some(Target::Object(s)))
        }) {
            return false;
        }
    }
    true
}

// spec: S-0215 claim 16 (scenario B5: the search exclusion before the damage; a hit from a known attacker that the actor survives makes it fall back to rest on that tick, claims 13 and 15 row 12 for an armed vehicle at the area-guard threshold 0; the guarding enemy then targets it)
#[test]
fn claim_16_b5_a_damaged_no_threat_vehicle_falls_back_and_is_then_targeted() {
    let (mut w, s, g, a) = b5(&rules(true, false), Kind::Vehicle, "CART");
    assert!(
        never_targeted(&mut w, s, &[g, a], 600),
        "no automatic search picks a no-threat actor (claim 16)"
    );
    assert_eq!(slots(&w, s), (Some(Behaviour::Harmless), None, None));
    let h0 = health(&w, s);
    shoot(&mut w, a, s);
    assert!(health(&w, s) < h0 && health(&w, s) > 0, "S survives a hit");
    // On the tick of the hit the fall-back queues the resting behaviour (claim 15 row 12:
    // armed, area-guard threshold 0, not in a bunker), without an immediate start.
    let (cur, queued, _) = slots(&w, s);
    assert!(
        queued == Some(Behaviour::AreaGuard)
            || (cur == Some(Behaviour::AreaGuard) && queued.is_none()),
        "area guard queued (or already current by the late start): {:?}",
        slots(&w, s)
    );
    testkit::run(&mut w, 2);
    assert_eq!(
        slots(&w, s).0,
        Some(Behaviour::AreaGuard),
        "current at the next start"
    );
    // The guarding unit now may pick it (claim 16; B5 "within one targeting delay").
    assert!(
        !never_targeted(&mut w, s, &[g], 200),
        "the guarding enemy targets S once it has left harmless"
    );
}

// spec: S-0215 claim 16 and claim 14 row 12 (an infantry unit of a player house that has neither the guard-area ability nor a type that defaults to area guard queues guard)
#[test]
fn claim_16_a_damaged_no_threat_infantry_unit_falls_back_to_guard() {
    let (mut w, s, _, a) = b5(&rules(true, false), Kind::Infantry, "WALKER");
    let h0 = health(&w, s);
    shoot(&mut w, a, s);
    assert!(health(&w, s) < h0 && health(&w, s) > 0, "S survives a hit");
    let (cur, queued, _) = slots(&w, s);
    assert!(
        queued == Some(Behaviour::Guard) || (cur == Some(Behaviour::Guard) && queued.is_none()),
        "guard queued: {:?}",
        slots(&w, s)
    );
    testkit::run(&mut w, 2);
    assert_eq!(slots(&w, s).0, Some(Behaviour::Guard));
}

// spec: S-0215 claim 16 (the main case through a player's attack order: the attacker fires by its attack processing, and the actor falls back on the tick of the first hit; claim 16 names no alliance condition, so the attacker here is of the actor's own house, as in scenario B29)
#[test]
fn claim_16_a_hit_from_an_own_house_attack_order_triggers_it() {
    let r = rules(true, false);
    let mut w = testkit::game_with(&r, SIZE, 3);
    let s = spawn(&mut w, Kind::Vehicle, "CART", 1, mid(SIZE, 0, 0), 0);
    force(&mut w, s, Behaviour::Harmless);
    let a = spawn(&mut w, Kind::Vehicle, "CART", 1, mid(SIZE, -3, 0), 64);
    w.tick = w.tick.max(w.orders_blocked_through + 1);
    assert!(orders::issue(
        &mut w,
        HouseId(1),
        Order::Behaviour {
            actor: a,
            behaviour: Behaviour::Attack,
            attack_move: false,
            target: Some(Target::Object(s)),
            dest: None,
        },
    ));
    let h0 = health(&w, s);
    let mut hit_tick = None;
    for _ in 0..300 {
        testkit::run(&mut w, 1);
        if health(&w, s) < h0 {
            hit_tick = Some(w.tick);
            break;
        }
    }
    assert!(hit_tick.is_some(), "the attack order lands a hit");
    assert!(health(&w, s) > 0);
    let (cur, queued, _) = slots(&w, s);
    assert!(
        queued == Some(Behaviour::AreaGuard)
            || (cur == Some(Behaviour::AreaGuard) && queued.is_none()),
        "fall-back on the tick of the hit: {:?}",
        slots(&w, s)
    );
}

// spec: S-0215 claim 16 and claim 17 (scenario B5 case b: the zombie property suppresses the fall-back; harmless stays current, nothing queued, and the guarding enemy still never targets it)
#[test]
fn claim_16_the_zombie_property_suppresses_the_fall_back() {
    let (mut w, s, g, a) = b5(&rules(true, true), Kind::Vehicle, "CART");
    let h0 = health(&w, s);
    let c0 = common_effects(&w, s);
    shoot(&mut w, a, s);
    assert!(health(&w, s) < h0 && health(&w, s) > 0);
    assert_eq!(slots(&w, s), (Some(Behaviour::Harmless), None, None));
    // Claim 17 would have the zombie row queue nothing at a fall-back, so the
    // slots alone cannot tell; no fall-back means no common effects either.
    assert_eq!(common_effects(&w, s), c0, "no fall-back to rest");
    assert!(never_targeted(&mut w, s, &[g, a], 600));
    assert_eq!(slots(&w, s).0, Some(Behaviour::Harmless));
}

// spec: S-0215 claim 16 (without the no-threat property there is no fall-back from damage: harmless stays current)
#[test]
fn claim_16_no_fall_back_without_the_no_threat_property() {
    let (mut w, s, _, a) = b5(&rules(false, false), Kind::Vehicle, "CART");
    let h0 = health(&w, s);
    let c0 = common_effects(&w, s);
    shoot(&mut w, a, s);
    assert!(health(&w, s) < h0 && health(&w, s) > 0);
    assert_eq!(slots(&w, s), (Some(Behaviour::Harmless), None, None));
    assert_eq!(common_effects(&w, s), c0, "no fall-back to rest");
}

// spec: S-0215 claim 16 and S-0302 claim 1 (a projectile whose firer has been killed since the shot carries no attacker: no known attacker, no fall-back)
#[test]
fn claim_16_no_fall_back_without_a_known_attacker() {
    let (mut w, s, _, a) = b5(&rules(true, false), Kind::Vehicle, "CART");
    // The firer is killed while its shell is in flight.
    aim(&mut w, a, s);
    assert!(fire::shot(&mut w, a, 0, Target::Object(s)));
    assert!(!w.projectiles.is_empty(), "the shell is in flight");
    let kill = Hit {
        amount: 1_000_000,
        warhead: None,
        distance: 0,
        attacker: None,
        house: None,
        ignore_defences: true,
        no_escape: true,
    };
    damage::apply(&mut w, a, kill);
    assert!(
        w.projectiles.values().all(|p| p.attacker.is_none()),
        "the shell carries no attacker once its firer is killed"
    );
    let h0 = health(&w, s);
    let c0 = common_effects(&w, s);
    let t0 = w.tick;
    while !w.projectiles.is_empty() {
        testkit::run(&mut w, 1);
        assert!(w.tick < t0 + 60);
    }
    assert!(
        health(&w, s) < h0 && health(&w, s) > 0,
        "S is hit and survives"
    );
    assert_eq!(slots(&w, s), (Some(Behaviour::Harmless), None, None));
    assert_eq!(common_effects(&w, s), c0, "no fall-back to rest");
}

// spec: S-0215 r118 claim 16, Results (Q-2586 answered: a healing hit, S-0302 claim 11, raises health and reports unaffected, and neither it nor any other hit that changes no health triggers the fall-back) and scenario B5 part (d) (harmless stays current and nothing is queued). The synthetic ruleset has no healing weapon, so the hits enter at `damage::apply`, the entry a detonation uses, with a known attacker
#[test]
fn claim_16_b5_d_a_heal_or_a_hit_that_lowers_no_health_gives_no_fall_back() {
    let (mut w, s, _, a) = b5(&rules(true, false), Kind::Vehicle, "CART");
    let he = w
        .rules()
        .combat
        .warheads
        .iter()
        .position(|h| h.name == "HE")
        .unwrap() as u32;
    let full = health(&w, s);
    w.obj_mut(s).unwrap().health = full / 2;
    let c0 = common_effects(&w, s);
    for amount in [-10, 0] {
        let h0 = health(&w, s);
        let hit = Hit {
            amount,
            warhead: Some(he),
            distance: 0,
            attacker: Some(a),
            house: Some(HouseId(0)),
            ignore_defences: false,
            no_escape: false,
        };
        damage::apply(&mut w, s, hit);
        assert!(health(&w, s) >= h0, "amount {amount}: no health lost");
        testkit::run(&mut w, 1);
        assert_eq!(
            slots(&w, s),
            (Some(Behaviour::Harmless), None, None),
            "amount {amount}"
        );
        assert_eq!(common_effects(&w, s), c0, "amount {amount}: no fall-back");
    }
    assert!(health(&w, s) > full / 2, "the heal raised health");
}

// spec: S-0215 claim 16 (the actor must survive the damage: a killing hit gives no fall-back; the dead actor leaves play in harmless)
#[test]
fn claim_16_no_fall_back_on_a_killing_hit() {
    let (mut w, s, _, a) = b5(&rules(true, false), Kind::Vehicle, "CART");
    w.obj_mut(s).unwrap().health = 1;
    aim(&mut w, a, s);
    assert!(fire::shot(&mut w, a, 0, Target::Object(s)));
    let mut last = slots(&w, s);
    let c0 = common_effects(&w, s);
    let t0 = w.tick;
    while w.obj(s).is_some_and(|o| o.health > 0) {
        last = slots(&w, s);
        testkit::run(&mut w, 1);
        assert!(w.tick < t0 + 60, "the shot did not kill");
    }
    // A killed vehicle leaves the world within the tick of its death, so a
    // wrong fall-back would leave no lasting trace on it (ledger, mutation
    // table); while it stays, nothing may show one.
    if let Some(o) = w.obj(s) {
        assert_eq!(
            (o.beh.current, o.beh.queued),
            (Some(Behaviour::Harmless), None),
            "no rest queued by the killing hit"
        );
        assert_eq!(o.beh.last_common_effects, c0, "no fall-back to rest");
    }
    assert_eq!(last, (Some(Behaviour::Harmless), None, None));
}

// spec: S-0215 claim 16 (a mobile actor only: a building in a no-threat behaviour keeps it when it survives damage from a known attacker)
#[test]
fn claim_16_a_building_does_not_fall_back() {
    let mut r = testkit::combat_ruleset();
    r.sim.buildings.get_mut("TOWER").unwrap().primary = None;
    r.sim.behaviours.insert(
        "guard".into(),
        BehaviourProps {
            no_threat: Some(true),
            ..Default::default()
        },
    );
    let mut w = testkit::game_with(&r, SIZE, 3);
    let b = spawn(&mut w, Kind::Building, "TOWER", 1, mid(SIZE, 0, 0), 0);
    let a = spawn(&mut w, Kind::Vehicle, "CART", 0, mid(SIZE, -3, 0), 64);
    testkit::run(&mut w, 2);
    let before = slots(&w, b);
    assert_eq!(before.0, Some(Behaviour::Guard));
    let h0 = health(&w, b);
    shoot(&mut w, a, b);
    assert!(health(&w, b) < h0 && health(&w, b) > 0);
    assert_eq!(slots(&w, b), before);
}

// spec: S-0215 claim 16 and claim 15 row 8 (the fall-back runs the rest tables, it forces no behaviour: an armed vehicle with a no-threat guard current queues nothing; scenario B29 case b2)
#[test]
fn claim_16_the_rest_tables_decide_an_armed_vehicle_in_guard_queues_nothing() {
    let mut r = testkit::combat_ruleset();
    // B29's file: guard is no-threat and has neither the retaliate nor the scatter property.
    r.sim.behaviours.insert(
        "guard".into(),
        BehaviourProps {
            no_threat: Some(true),
            retaliate: Some(false),
            scatter: Some(false),
            ..Default::default()
        },
    );
    let mut w = testkit::game_with(&r, SIZE, 3);
    let s = spawn(&mut w, Kind::Vehicle, "CART", 1, mid(SIZE, 0, 0), 0);
    let a = spawn(&mut w, Kind::Vehicle, "CART", 1, mid(SIZE, -3, 0), 64);
    testkit::run(&mut w, 2);
    assert_eq!(slots(&w, s), (Some(Behaviour::Guard), None, None));
    let c0 = common_effects(&w, s);
    let t = shoot(&mut w, a, s);
    assert!(health(&w, s) > 0);
    assert_eq!(slots(&w, s), (Some(Behaviour::Guard), None, None));
    // The fall-back did run: its common effects are marked at the hit (claim 13).
    let c1 = common_effects(&w, s);
    assert!(
        c1 != c0 && c1.is_some_and(|c| c > t - 2 && c <= t),
        "a fall-back to rest at the hit: {c0:?} then {c1:?}, shot detonated by {t}"
    );
}

// spec: S-0215 r118 claim 16, Order within the hit (Q-2586 answered: the test comes after the retaliation, claim 18; an actor that retaliates at that hit has attack current, claim 10, when the test is made, does not fall back unless attack has the no-threat property, and its no-threat behaviour stays suspended until the revert) and scenario B5 part (c) (player return fire on; expected at the end of the tick of the first hit: attack current, harmless suspended and nothing queued)
#[test]
fn claim_16_b5_c_retaliation_comes_before_the_no_threat_test() {
    let mut r = rules(true, false);
    // B5 c's file: harmless with the no-threat and retaliate properties,
    // attack with neither no-threat nor zombie, player return fire on.
    r.sim.behaviours.get_mut("harmless").unwrap().retaliate = Some(true);
    r.sim.behaviours.insert(
        "attack".into(),
        BehaviourProps {
            no_threat: Some(false),
            zombie: Some(false),
            ..Default::default()
        },
    );
    r.sim.general.player_return_fire = Some(true);
    let mut w = testkit::game_with(&r, SIZE, 3);
    // S, an armed vehicle of a human-controlled house in harmless with no
    // target and no destination; A, an enemy vehicle within S's range.
    let s = spawn(&mut w, Kind::Vehicle, "CART", 1, mid(SIZE, 0, 0), 0);
    force(&mut w, s, Behaviour::Harmless);
    let a = spawn(&mut w, Kind::Vehicle, "CART", 0, mid(SIZE, -3, 0), 64);
    assert!(w.houses[1].human, "set-up: a human-controlled owner");
    assert!(!w.allied(w.obj(s).unwrap().owner, w.obj(a).unwrap().owner));
    let c0 = common_effects(&w, s);
    assert_eq!(
        w.obj(s).unwrap().target,
        None,
        "set-up: no target before the hit"
    );
    shoot(&mut w, a, s);
    assert!(health(&w, s) > 0);
    let o = w.obj(s).unwrap();
    assert_eq!(
        o.target,
        Some(Target::Object(a)),
        "set-up: S retaliates at A"
    );
    assert_eq!(
        (o.beh.current, o.beh.queued, o.beh.suspended),
        (Some(Behaviour::Attack), None, Some(Behaviour::Harmless)),
        "attack current, harmless suspended and nothing queued"
    );
    assert_eq!(common_effects(&w, s), c0, "no fall-back to rest");
}

// spec: S-0215 r118 claim 16 (an actor that retaliates falls back when attack has the no-threat property: the test is made on the behaviour current at that point of the hit)
#[test]
fn claim_16_a_retaliation_into_a_no_threat_attack_falls_back() {
    let mut r = rules(true, false);
    r.sim.behaviours.get_mut("harmless").unwrap().retaliate = Some(true);
    r.sim.behaviours.insert(
        "attack".into(),
        BehaviourProps {
            no_threat: Some(true),
            ..Default::default()
        },
    );
    r.sim.general.player_return_fire = Some(true);
    let mut w = testkit::game_with(&r, SIZE, 3);
    let s = spawn(&mut w, Kind::Vehicle, "CART", 1, mid(SIZE, 0, 0), 0);
    force(&mut w, s, Behaviour::Harmless);
    let a = spawn(&mut w, Kind::Vehicle, "CART", 0, mid(SIZE, -3, 0), 64);
    let c0 = common_effects(&w, s);
    let t = shoot(&mut w, a, s);
    assert!(health(&w, s) > 0);
    let c1 = common_effects(&w, s);
    assert!(
        c1 != c0 && c1.is_some_and(|c| c > t - 2 && c <= t),
        "the fall-back runs on attack: {c0:?} then {c1:?}, shot detonated by {t}"
    );
}

// spec: S-0215 claim 16 and S-0607 claim 9 (a fall-back to rest in the grip: guard has the no-threat property and not the zombie property, and the gripped unit survives damage from a known attacker; with no attack-move memory it queues move without an immediate start, claim 15 row 3 for the grip's destination, and it does not get its own locomotor back: the grip holds)
#[test]
fn claim_16_and_s0607_claim_9_a_gripped_unit_falls_back_and_queues_move() {
    use lift_common::*;
    let mut w = world(|r| {
        r.sim.behaviours.insert(
            "guard".into(),
            BehaviourProps {
                no_threat: Some(true),
                retaliate: Some(false),
                scatter: Some(false),
                ..Default::default()
            },
        );
    });
    let l = spawn(&mut w, Kind::Vehicle, "LIFTER", 0, at(0, 0));
    let v = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(6, 0));
    beam(&mut w, l, v);
    assert!(
        glasnost_sim::lift::grips(&w, l, v),
        "the lifter grips the victim"
    );
    let o = w.obj(v).unwrap();
    assert_eq!(o.beh.current, Some(Behaviour::Guard));
    assert!(o.dest.is_some(), "the grip's destination");
    assert_eq!(o.beh.queued, None);
    // A third unit of house 0 hits the gripped victim with its rifle.
    let x = lift_common::spawn(&mut w, Kind::Infantry, "WALKER", 0, at(5, 2));
    let h0 = health(&w, v);
    shoot(&mut w, x, v);
    assert!(
        health(&w, v) < h0 && health(&w, v) > 0,
        "the victim survives a hit"
    );
    assert!(glasnost_sim::lift::grips(&w, l, v), "the grip holds");
    let o = w.obj(v).unwrap();
    assert_eq!(o.beh.queued, Some(Behaviour::Move), "move queued (row 3)");
    assert_eq!(o.beh.current, Some(Behaviour::Guard), "no immediate start");
}
