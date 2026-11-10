//! Claim tests for S-0501 r178 (Q-2868 answered), M8 intake 119: a
//! kamikaze (S-0506 Terms) is removed from the game without dying at the
//! end of its first firing pass with an ordinary attempt, after the pass's
//! last attempt; the removal is no death and spends no round, and the
//! pass's projectiles have no firer, so a kill they make credits no house
//! (claim 20, *Kamikaze removal*; scenario AC36 parts a to c). Synthetic
//! data through the real tick path; every type, weapon, map and number is
//! invented. Ledger: `docs/m8-claims/intake119.md`.

mod econ_common;

use econ_common::{SIZE, game, issue, ready, wheel};
use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{Locomotor, UnitType, WeaponType};
use glasnost_sim::air;
use glasnost_sim::combat::effects::Effect;
use glasnost_sim::econ::stats;
use glasnost_sim::orders::Order;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, Pos, Target, World};

fn push(r: &mut Ruleset, reg: &str, n: &str) {
    r.registries.entry(reg.into()).or_default().push(n.into());
}

/// The ruleset: K (a flying aircraft type with the missile-spawn flag, not
/// a rocket type, the ordinary weapon Agm in its primary slot, a burst of
/// `burst`, a rocket projectile that flies for many ticks) and the target
/// vehicle T (unarmed, strength `strength`).
fn rules(burst: i32, strength: i32) -> Ruleset {
    let mut r = econ_common::ruleset();
    let c = testkit::combat_ruleset();
    r.sim.weapons.extend(c.sim.weapons.clone());
    r.sim.projectiles.extend(c.sim.projectiles.clone());
    r.sim.warheads.extend(c.sim.warheads.clone());
    r.sim.weapons.insert(
        "Agm".into(),
        WeaponType {
            damage: Some(40),
            rof: Some(30),
            burst: Some(burst),
            range: econ_common::dec("6"),
            speed: Some(30),
            projectile: Some("Rocket".into()),
            warhead: Some("HE".into()),
            ..Default::default()
        },
    );
    let mut k = UnitType {
        strength: Some(200),
        speed: Some(30),
        rot: Some(12),
        locomotor: Some(Locomotor::Fly),
        tech_level: Some(1),
        owner: Some(vec!["Alphaland".into(), "Betaland".into()]),
        cost: Some(1000),
        primary: Some("Agm".into()),
        missile_spawn: Some(true),
        ..Default::default()
    };
    k.combat.ammo = Some(3);
    k.econ.landable = Some(true);
    push(&mut r, "aircraft", "K");
    r.sim.aircraft.insert("K".into(), k);
    let mut t = wheel(500);
    t.strength = Some(strength);
    push(&mut r, "vehicles", "T");
    r.sim.vehicles.insert("T".into(), t);
    r
}

fn step(w: &mut World) {
    glasnost_sim::tick::step(w);
}

fn describe(w: &World, id: ObjectId) -> String {
    let Some(o) = w.obj(id) else {
        return format!("t={} gone", w.tick);
    };
    format!(
        "t={} pos={:?} on_map={} h={} hp={} ammo={} beh={:?}/{:?} dest={:?} target={:?}",
        w.tick,
        o.pos,
        o.on_map,
        air::height(w, id),
        o.health,
        o.combat.ammo,
        o.beh.current,
        o.beh.queued,
        o.dest,
        o.target,
    )
}

/// AC36's set-up on synthetic data: K of house 0 standing landed, T of
/// house 1 in *guard* 16 cells east, an attack order on T. With
/// `kamikaze`, K is enrolled as a kamikaze with T's cell as its release
/// cell, as a release enrolls it (S-0506 claim 12), and given 1 round as
/// S-0506 claim 26's orders give it; the shared re-order is put off so
/// that its orders do not change the target the attack order gave. Returns
/// (world, K, T).
fn ac36(r: &Ruleset, kamikaze: bool) -> (World, ObjectId, ObjectId) {
    ac36_in(game(r), kamikaze)
}

/// [`ac36`] on a game already made.
fn ac36_in(mut w: World, kamikaze: bool) -> (World, ObjectId, ObjectId) {
    ready(&mut w);
    let k = testkit::spawn(&mut w, Kind::Aircraft, "K", 0, testkit::mid(SIZE, -8, 4), 0);
    let t = testkit::spawn(&mut w, Kind::Vehicle, "T", 1, testkit::mid(SIZE, 8, 4), 0);
    step(&mut w);
    if kamikaze {
        let cell: CellXY = w.obj(t).unwrap().pos.cell();
        w.spawned.kamikaze.push((k, cell));
        w.spawned.next_reorder = i32::MAX;
        w.obj_mut(k).unwrap().combat.ammo = 1;
    }
    assert!(issue(
        &mut w,
        0,
        Order::Behaviour {
            actor: k,
            behaviour: Behaviour::Attack,
            attack_move: false,
            target: Some(Target::Object(t)),
            dest: None,
        }
    ));
    (w, k, t)
}

/// The shots K made in the tick just run, with their fire points.
fn shots(w: &World, k: ObjectId) -> Vec<Pos> {
    w.effects
        .iter()
        .filter_map(|e| match e {
            Effect::Shot { firer, from, .. } if *firer == k => Some(*from),
            _ => None,
        })
        .collect()
}

fn totals(w: &World) -> [(i32, i32); 2] {
    [0u8, 1].map(|h| {
        let s = stats::of(w, HouseId(h));
        (s.kills, s.losses)
    })
}

/// Steps to the tick *s* of K's first firing pass; returns (the shots of
/// *s*, K's health, T's health and the totals before *s*).
fn to_first_pass(w: &mut World, k: ObjectId, t: ObjectId) -> (Vec<Pos>, i32, i32, [(i32, i32); 2]) {
    for _ in 0..2000 {
        let hp = w.obj(k).unwrap().health;
        let thp = w.obj(t).unwrap().health;
        let before = totals(w);
        w.effects.clear();
        step(w);
        let s = shots(w, k);
        if !s.is_empty() {
            return (s, hp, thp, before);
        }
        assert!(
            w.obj(k).is_some_and(|o| o.on_map),
            "void: K lost before its pass: {}",
            describe(w, k)
        );
    }
    panic!("no firing pass: {}", describe(w, k));
}

// spec: S-0501 claim 20, Kamikaze removal (r178, Q-2868: a kamikaze is removed from the game without dying at the end of its first firing pass with an ordinary attempt, after the pass's attempts; the removal is not a death: no hit, explosion, death weapon or crash, no kill or loss for any house, no round spent; it leaves the map and play as any removed object does and is a leftover for the rest of the tick, S-0203 claim 13) and scenario AC36 part a; the control, the same aircraft not enrolled, stays in play
#[test]
fn ac36_a_a_kamikaze_is_removed_at_the_end_of_its_first_pass() {
    for kamikaze in [true, false] {
        let (mut w, k, t) = ac36(&rules(1, 500), kamikaze);
        let (s, hp, thp, before) = to_first_pass(&mut w, k, t);
        let d = describe(&w, k);
        assert_eq!(s.len(), 1, "{kamikaze}: one projectile: {d}");
        if !kamikaze {
            assert!(w.obj(k).is_some_and(|o| o.on_map), "control: in play: {d}");
            step(&mut w);
            assert!(w.obj(k).is_some(), "control: {}", describe(&w, k));
            continue;
        }
        if let Some(o) = w.obj(k) {
            assert!(!o.on_map, "off the map at the end of s: {d}");
            assert_eq!(o.health, hp, "its health as before s: {d}");
            assert_eq!(o.combat.ammo, 1, "no round spent: {d}");
        }
        assert!(
            !w.effects
                .iter()
                .any(|e| matches!(e, Effect::Killed { id, .. } if *id == k)),
            "no death"
        );
        assert_eq!(totals(&w), before, "no kill or loss in s");
        assert_eq!(w.obj(t).unwrap().health, thp, "T unhurt in s");
        assert!(
            !w.spawned.kamikaze.iter().any(|e| e.0 == k),
            "out of the group"
        );
        step(&mut w);
        assert!(w.obj(k).is_none(), "gone in s + 1");
        // T's health first falls at the projectile's detonation.
        let mut fell = false;
        for _ in 0..300 {
            w.effects.clear();
            step(&mut w);
            let det = w
                .effects
                .iter()
                .any(|e| matches!(e, Effect::Detonation { .. }));
            let now = w.obj(t).unwrap().health;
            if now < thp {
                assert!(det, "T's health falls at a detonation");
                fell = true;
                break;
            }
        }
        assert!(fell, "the projectile hits T");
        assert_eq!(totals(&w), before, "no kill or loss");
    }
}

// spec: S-0501 claim 20, Kamikaze removal (r178, Q-2868: the removal neither cuts the pass short nor adds to it: with a weapon whose every attempt makes a projectile, all B attempts are made from the place where the aircraft is, and the removal comes after the last of them) and scenario AC36 part b
#[test]
fn ac36_b_the_whole_burst_comes_before_the_removal() {
    let (mut w, k, t) = ac36(&rules(3, 500), true);
    let (s, _, _, _) = to_first_pass(&mut w, k, t);
    assert_eq!(s.len(), 3, "three projectiles in s: {}", describe(&w, k));
    assert!(s.iter().all(|p| *p == s[0]), "one start place: {s:?}");
    assert!(w.obj(k).is_none_or(|o| !o.on_map), "removed");
    for _ in 0..60 {
        w.effects.clear();
        step(&mut w);
        assert!(shots(&w, k).is_empty(), "none after s");
    }
}

// spec: S-0501 claim 20, Kamikaze removal (r178, Q-2868: every projectile of that pass has no firer when it detonates, S-0306 Terms; its detonation carries neither attacker nor attacking house, S-0302 claim 1, so a target it kills gives no house a kill and gives the target's house a loss, S-0414 claim 21) and scenario AC36 part c
#[test]
fn ac36_c_a_kill_by_the_pass_credits_no_house() {
    let (mut w, k, t) = ac36(&rules(1, 1), true);
    let (_, _, _, before) = to_first_pass(&mut w, k, t);
    let mut dead = false;
    for _ in 0..300 {
        step(&mut w);
        if w.obj(t).is_none_or(|o| o.health <= 0) {
            dead = true;
            break;
        }
    }
    assert!(dead, "T dies at the detonation");
    let after = totals(&w);
    assert_eq!(after[0].0, before[0].0, "house 0 gets no kill");
    assert_eq!(after[1].0, before[1].0, "house 1 gets no kill");
    assert_eq!(after[1].1, before[1].1 + 1, "house 1 takes a loss");
}

// spec: S-0501 claim 20, Kamikaze removal (r178, Q-2868: the removal is not a death: it makes none of S-0011's destruction raises, claim 19, and a tag the kamikaze carries stays attached, S-0011 claim 2; a bomb planted on it is removed with no detonation, S-0519 claim 7); S-0519 claim 5 (a bomb set off on the map detonates at its carrier); the control, the same aircraft not enrolled, keeps its bomb and its tag in play
#[test]
fn seams7b_the_removal_makes_no_raise_keeps_the_tag_and_drops_the_bomb() {
    let mut r = rules(1, 500);
    // A bomb that would reach T from where the pass is made, and a timer
    // that does not come due in these runs.
    r.sim.warheads.insert(
        "BigBoom".into(),
        glasnost_rules::sim::WarheadType {
            cell_spread: econ_common::dec("12"),
            percent_at_max: econ_common::dec("1"),
            verses: Some(vec![glasnost_rules::sim::Decimal("1".into()); 11]),
            ..Default::default()
        },
    );
    let g = &mut r.sim.general;
    g.ivan_warhead = Some("BigBoom".into());
    g.ivan_damage = Some(77);
    g.ivan_timed_delay = Some(100_000);
    for kamikaze in [true, false] {
        let w = testkit::game_with_triggers(&r, SIZE, 7, testkit::entry_triggers());
        let (mut w, k, t) = ac36_in(w, kamikaze);
        // The destroyed-by-anything trigger DIE sets variable 1.
        glasnost_sim::triggers::object_tag(&mut w, k, Some(b"DIE"));
        let tag = *w.triggers.objects.get(&k).expect("void: tagged");
        let carriers = w.triggers.tags[&tag].carriers;
        let tick = w.tick;
        w.obj_mut(k).unwrap().combat.bomb = Some(glasnost_sim::bomb::Bomb {
            planter: None,
            house: Some(HouseId(1)),
            planted: tick,
        });
        let (s, _, thp, _) = to_first_pass(&mut w, k, t);
        let d = describe(&w, k);
        assert_eq!(s.len(), 1, "{kamikaze}: void: one projectile: {d}");
        assert!(
            !w.effects
                .iter()
                .any(|e| matches!(e, Effect::Detonation { .. })),
            "{kamikaze}: no detonation in s: {d}"
        );
        assert_eq!(w.obj(t).unwrap().health, thp, "{kamikaze}: T unhurt in s");
        assert_eq!(w.triggers.objects.get(&k), Some(&tag), "{kamikaze}: tag");
        assert_eq!(w.triggers.tags[&tag].carriers, carriers, "{kamikaze}");
        if kamikaze {
            assert!(w.obj(k).is_none_or(|o| !o.on_map), "removed: {d}");
            assert!(!glasnost_sim::bomb::carries(&w, k), "the bomb is gone");
        } else {
            assert!(w.obj(k).is_some_and(|o| o.on_map), "control: {d}");
            assert!(glasnost_sim::bomb::carries(&w, k), "control: kept");
        }
        for _ in 0..30 {
            step(&mut w);
        }
        assert!(!w.triggers.variables[1], "{kamikaze}: no destruction raise");
        if kamikaze {
            assert_eq!(w.triggers.objects.get(&k), Some(&tag), "still attached");
            continue;
        }
        // The control's death shows what the removal does not make: the
        // bomb goes off on the map (S-0519 claim 5) and the raise fires.
        w.effects.clear();
        glasnost_sim::combat::damage::apply(
            &mut w,
            k,
            glasnost_sim::combat::damage::Hit {
                amount: 100_000,
                warhead: None,
                distance: 0,
                attacker: None,
                house: None,
                ignore_defences: true,
                no_escape: false,
            },
        );
        step(&mut w);
        assert!(
            w.effects
                .iter()
                .any(|e| matches!(e, Effect::Detonation { .. })),
            "control: the bomb goes off at its death"
        );
        assert!(w.triggers.variables[1], "control: the death's raise");
    }
}
