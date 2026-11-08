//! Aircraft retaliation: S-0215 claims 10 and 18, S-0309 claim 18 and
//! claim 25 part h, with S-0501 (claim 32, T1 and T2) for what the
//! overridden aircraft does next, in the order of the hit of S-0215 claim 16
//! (anger, retaliation, the scatter from damage of claim 19). Scenario B7's
//! cases (a) to (d) run with an armed aircraft in place of the vehicle.
//! Hits are real shots detonating in the tick pipeline (`testkit::run`),
//! or, where no tick may pass between the setup and the hit, a hit entering
//! the damage path at `damage::apply`, the entry a detonation uses.
//! Synthetic data only: the combat ruleset of `testkit` with an invented
//! armed aircraft, a pad and behaviour files after scenario B7; no stock
//! value. Ledger: `docs/m8-claims/air-retaliate.md`.

use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{BehaviourProps, BuildingType, Decimal, Locomotor, UnitType, WeaponType};
use glasnost_sim::combat::acquire::{self, Retaliation};
use glasnost_sim::combat::check::{self, Answer};
use glasnost_sim::combat::damage::{self, Hit};
use glasnost_sim::combat::fire;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit::{self, mid, spawn};
use glasnost_sim::world::{ObjectId, Target, World};

const SIZE: i32 = 40;

fn push(r: &mut Ruleset, reg: &str, n: &str) {
    r.registries.entry(reg.into()).or_default().push(n.into());
}

/// The combat ruleset with the weapon LONG (range 10 cells, an arcing
/// shell) on a BUGGY of strength 3000; the aircraft JET (armed with AGM,
/// range `range` cells, sight `sight`, two rounds, landable, docking at
/// PAD), the helipad PAD; and the player-scatter setting as given.
fn rules(player_scatter: bool, range: &str, sight: i32) -> Ruleset {
    let mut r = testkit::combat_ruleset();
    let d = |s: &str| Decimal(s.into());
    r.sim.weapons.insert(
        "Long".into(),
        WeaponType {
            damage: Some(20),
            rof: Some(90),
            range: Some(d("10")),
            speed: Some(40),
            projectile: Some("Lob".into()),
            warhead: Some("SA".into()),
            ..Default::default()
        },
    );
    r.sim.weapons.insert(
        "Agm".into(),
        WeaponType {
            damage: Some(40),
            rof: Some(30),
            range: Some(d(range)),
            speed: Some(60),
            projectile: Some("Rocket".into()),
            warhead: Some("HE".into()),
            ..Default::default()
        },
    );
    let buggy = r.sim.vehicles.get_mut("BUGGY").unwrap();
    buggy.primary = Some("Long".into());
    buggy.strength = Some(3000);
    let mut jet = UnitType {
        strength: Some(400),
        speed: Some(30),
        rot: Some(12),
        sight: Some(sight),
        locomotor: Some(Locomotor::Fly),
        primary: Some("Agm".into()),
        ..Default::default()
    };
    jet.combat.ammo = Some(2);
    jet.econ.landable = Some(true);
    jet.econ.dock = Some(vec!["PAD".into()]);
    push(&mut r, "aircraft", "JET");
    r.sim.aircraft.insert("JET".into(), jet);
    push(&mut r, "buildings", "PAD");
    r.sim.buildings.insert(
        "PAD".into(),
        BuildingType {
            strength: Some(500),
            foundation: Some([2, 2]),
            helipad: Some(true),
            ..Default::default()
        },
    );
    r.sim.general.player_scatter = Some(player_scatter);
    r
}

/// Sets the scatter and retaliate properties of behaviour `b`.
fn props(r: &mut Ruleset, b: Behaviour, scatter: bool, retaliate: bool) {
    r.sim.behaviours.insert(
        b.key().into(),
        BehaviourProps {
            scatter: Some(scatter),
            retaliate: Some(retaliate),
            ..Default::default()
        },
    );
}

fn force(w: &mut World, id: ObjectId, b: Behaviour) {
    let t = w.tick;
    let stale = w.rules().quirks.stale_suspended_revert;
    w.obj_mut(id).unwrap().beh.force(b, t, stale);
}

fn health(w: &World, id: ObjectId) -> i32 {
    w.obj(id).map_or(0, |o| o.health)
}

/// A JET of house `h` landed at the middle of a fresh game (houses 0 and 1
/// player houses, 2 neutral and 3 special), with no target and no
/// destination and `b` current, and a BUGGY of house 0 in *sleep* `dx`
/// cells map west of it.
fn scene(r: &Ruleset, h: u8, dx: i32, b: Behaviour) -> (World, ObjectId, ObjectId) {
    let mut w = testkit::game_with_non_players(r, SIZE, 5);
    let j = spawn(&mut w, Kind::Aircraft, "JET", h, mid(SIZE, 0, 0), 0);
    let a = spawn(&mut w, Kind::Vehicle, "BUGGY", 0, mid(SIZE, -dx, 0), 64);
    force(&mut w, a, Behaviour::Sleep);
    testkit::run(&mut w, 4);
    {
        let o = w.obj_mut(j).unwrap();
        o.dest = None;
        o.target = None;
    }
    force(&mut w, j, b);
    assert!(glasnost_sim::air::on_ground(&w, j), "landed");
    (w, j, a)
}

/// `firer` fires its primary weapon at `target` and the shell detonates in
/// the tick pipeline; the target loses health and survives. Gives the tick
/// of the hit.
fn shoot(w: &mut World, firer: ObjectId, target: ObjectId) -> i32 {
    let h0 = health(w, target);
    w.obj_mut(firer).unwrap().target = Some(Target::Object(target));
    assert!(fire::shot(w, firer, 0, Target::Object(target)), "a shot");
    let t0 = w.tick;
    loop {
        let tick = w.tick;
        testkit::run(w, 1);
        if w.projectiles.is_empty() {
            w.obj_mut(firer).unwrap().target = None;
            let h = health(w, target);
            assert!(h < h0 && h > 0, "a hit it survives: {h0} to {h}");
            return tick;
        }
        assert!(w.tick < t0 + 120, "the shot did not detonate");
    }
}

/// A hit of 20 with the SA warhead on `v` from `a`, entering the damage path
/// at `damage::apply` with no tick passing.
fn hit(w: &mut World, v: ObjectId, a: ObjectId) {
    let sa = w
        .rules()
        .combat
        .warheads
        .iter()
        .position(|h| h.name == "SA")
        .unwrap() as u32;
    let h0 = health(w, v);
    let house = w.obj(a).and_then(|o| o.owner);
    damage::apply(
        w,
        v,
        Hit {
            amount: 20,
            warhead: Some(sa),
            distance: 0,
            attacker: Some(a),
            house,
            ignore_defences: false,
            no_escape: false,
        },
    );
    assert!(health(w, v) < h0 && health(w, v) > 0, "a hit it survives");
}

/// A killing hit on `id` with no attacker.
fn kill(w: &mut World, id: ObjectId) {
    damage::apply(
        w,
        id,
        Hit {
            amount: 100_000,
            warhead: None,
            distance: 0,
            attacker: None,
            house: None,
            ignore_defences: true,
            no_escape: true,
        },
    );
    assert!(w.obj(id).is_none_or(damage::dead), "killed");
}

/// Whether `j` retaliated against `a`: an override to *attack* with `a` as
/// target, `was` suspended, and the target and destination it had put aside.
fn overridden(w: &World, j: ObjectId, a: ObjectId, was: Behaviour) -> bool {
    let o = w.obj(j).unwrap();
    o.target == Some(Target::Object(a))
        && o.beh.current == Some(Behaviour::Attack)
        && o.beh.suspended == Some(was)
        && o.beh.aside == Some((None, None))
}

/// Whether `j` was left as it was: no target, `b` current, nothing suspended.
fn untouched(w: &World, j: ObjectId, b: Behaviour) -> bool {
    let o = w.obj(j).unwrap();
    o.target.is_none() && o.beh.current == Some(b) && o.beh.suspended.is_none()
}

// spec: S-0215 claim 18 and scenario B7 (a) (a mobile actor of a human-controlled owner, AI houses alike, retaliates only while its current behaviour is guard, area guard or patrol: an aircraft in move, fired at by an enemy within its weapon range, does not retaliate and move continues)
#[test]
fn b7_a_an_aircraft_in_move_does_not_retaliate() {
    let (mut w, j, a) = scene(&rules(false, "6", 0), 1, 5, Behaviour::Move);
    let far = Target::Cell(mid(SIZE, 12, 0));
    w.obj_mut(j).unwrap().dest = Some(far);
    hit(&mut w, j, a);
    assert!(
        untouched(&w, j, Behaviour::Move),
        "{:?}",
        w.obj(j).unwrap().beh
    );
    assert_eq!(w.obj(j).unwrap().dest, Some(far), "move goes on");
}

// spec: S-0215 claim 18 and scenario B7 (c) (with the player return-fire setting on, the behaviour list does not hold back the aircraft in move of (a): it retaliates, an override to attack with the attacker as target and move suspended); S-0215 claim 10 (the target and destination put aside, the destination replaced by none)
#[test]
fn b7_c_with_player_return_fire_an_aircraft_in_move_retaliates() {
    let mut r = rules(false, "6", 0);
    r.sim.general.player_return_fire = Some(true);
    props(&mut r, Behaviour::Move, false, true);
    let (mut w, j, a) = scene(&r, 1, 5, Behaviour::Move);
    let far = Target::Cell(mid(SIZE, 12, 0));
    w.obj_mut(j).unwrap().dest = Some(far);
    hit(&mut w, j, a);
    let o = w.obj(j).unwrap();
    assert_eq!(o.target, Some(Target::Object(a)), "{:?}", o.beh);
    assert_eq!(o.beh.current, Some(Behaviour::Attack));
    assert_eq!(o.beh.suspended, Some(Behaviour::Move));
    assert_eq!(o.beh.aside, Some((None, Some(far))), "put aside");
    assert_eq!(o.dest, None, "no destination");
}

// spec: S-0215 claims 10 and 18 and scenario B7 (b) (an aircraft of a human-controlled owner in guard whose weapon reaches the attacker overrides to attack at once, the suspended slot showing guard; S-0309 claim 18's conditions hold for it) and S-0501 claim 32 and T1 (the aircraft then flies its attack run at the attacker: it leaves the ground); S-0501 claim 32 and S-0215 claim 10 (at the killing hit on the attacker it loses its target and the revert brings back guard and the target put aside)
#[test]
fn b7_b_an_aircraft_in_guard_overrides_to_attack_and_reverts_at_the_attackers_death() {
    let (mut w, j, a) = scene(&rules(false, "6", 0), 1, 5, Behaviour::Guard);
    shoot(&mut w, a, j);
    assert!(
        overridden(&w, j, a, Behaviour::Guard),
        "{:?}",
        w.obj(j).unwrap().beh
    );
    let mut flew = false;
    for _ in 0..150 {
        testkit::run(&mut w, 1);
        let o = w.obj(j).unwrap();
        assert_eq!(o.target, Some(Target::Object(a)), "it keeps the attacker");
        assert_eq!(o.beh.suspended, Some(Behaviour::Guard));
        flew |= !glasnost_sim::air::on_ground(&w, j);
    }
    assert!(flew, "the attack run takes it off the ground");
    kill(&mut w, a);
    let o = w.obj(j).unwrap();
    assert_eq!(o.beh.current, Some(Behaviour::Guard), "{:?}", o.beh);
    assert_eq!(o.beh.suspended, None);
    assert_eq!(o.target, None, "the target put aside, none");
}

/// The JET of house 1 in *guard*, sight 3 and AGM range 2 cells, which does
/// not reach the BUGGY 4 cells west; the JET moved along the row so that
/// its 3-D distance to the BUGGY is `d` leptons.
fn f7_scene(d: i64) -> (World, ObjectId, ObjectId) {
    let (mut w, j, a) = scene(&rules(false, "2", 3), 1, 4, Behaviour::Guard);
    let x0 = w.obj(j).unwrap().pos.x;
    let found = (-127..=127).any(|s| {
        w.obj_mut(j).unwrap().pos.x = x0 + s;
        acquire::rho3(&w, j, a) == d
    });
    assert!(found, "a position at {d} leptons");
    (w, j, a)
}

// spec: S-0215 claim 18 and F7, scenario B7 (d) (an aircraft of a human-controlled owner whose weapon does not reach the attacker overrides only when its 3-D distance is at most (S + 0.5) × 256 leptons: with sight 3, at 896 leptons it overrides, at 897 it may retaliate but does not)
#[test]
fn b7_d_f7_distance_boundary_for_an_aircraft() {
    let (mut w, j, a) = f7_scene(896);
    assert_eq!(acquire::retaliate(&mut w, j, a), Retaliation::Done);
    assert!(overridden(&w, j, a, Behaviour::Guard));
    let (mut w, j, a) = f7_scene(897);
    assert_eq!(acquire::retaliate(&mut w, j, a), Retaliation::Held);
    assert!(untouched(&w, j, Behaviour::Guard));
    // The same through the damage path.
    let (mut w, j, a) = f7_scene(896);
    hit(&mut w, j, a);
    assert!(overridden(&w, j, a, Behaviour::Guard));
    let (mut w, j, a) = f7_scene(897);
    hit(&mut w, j, a);
    assert!(untouched(&w, j, Behaviour::Guard));
}

// spec: S-0215 claim 18 (an actor of a computer-controlled owner overrides at once; in our engine the neutral and special houses keep this rule) and S-0309 claim 25 part h (with no target of its own, no threat-sum test): a neutral-house aircraft retaliates against an attacker beyond its weapon range and beyond F7's distance
#[test]
fn claim_18_a_neutral_aircraft_overrides_at_once_beyond_f7() {
    let (mut w, j, a) = scene(&rules(false, "2", 0), 2, 9, Behaviour::Guard);
    assert!(!w.allied(w.obj(j).unwrap().owner, w.obj(a).unwrap().owner));
    hit(&mut w, j, a);
    assert!(
        overridden(&w, j, a, Behaviour::Guard),
        "{:?}",
        w.obj(j).unwrap().beh
    );
}

// spec: S-0215 claim 19 case 1 (only an actor that may retaliate and does not is sent the request: an aircraft that retaliates is not, so it does not fall back to rest at the hit by S-0501 claims 49 and 18 even with the player-scatter setting on and the scatter property on guard) and S-0215 claim 16 (retaliation comes before the scatter from damage in the hit); the control beyond F7 is sent it and falls back
#[test]
fn claim_19_case_1_no_longer_reaches_an_aircraft_that_retaliates() {
    let mut r = rules(true, "6", 0);
    props(&mut r, Behaviour::Guard, true, true);
    let (mut w, j, a) = scene(&r, 1, 5, Behaviour::Guard);
    let t = shoot(&mut w, a, j);
    let o = w.obj(j).unwrap();
    assert_ne!(
        o.beh.last_common_effects,
        Some(t),
        "no fall-back at the hit"
    );
    assert!(overridden(&w, j, a, Behaviour::Guard), "{:?}", o.beh);
    // The control: beyond the weapon's reach and F7's distance (sight 0).
    let (mut w, j, a) = scene(&r, 1, 9, Behaviour::Guard);
    let t = shoot(&mut w, a, j);
    let o = w.obj(j).unwrap();
    assert_eq!(o.beh.last_common_effects, Some(t), "case 1: the fall-back");
    assert_eq!(o.target, None);
}

// spec: S-0309 claim 18 (the firing check without range for an armed aircraft with no ammunition left answers out of ammunition, which is neither illegal nor impossible, so it may retaliate) and S-0215 claims 10 and 18 (it overrides to attack); S-0501 T1 (start with a target: choosing; choosing with no ammunition: done) and claim 17, T2 row 1 (the run ends, it falls back to rest and the overridden guard is restored with the target put aside)
#[test]
fn claim_18_an_aircraft_with_no_ammunition_retaliates_and_falls_back() {
    let (mut w, j, a) = scene(&rules(false, "6", 0), 1, 5, Behaviour::Guard);
    w.obj_mut(j).unwrap().combat.ammo = 0;
    assert_eq!(
        check::check(&w, j, Some(Target::Object(a)), 0, false),
        Answer::OutOfAmmo
    );
    shoot(&mut w, a, j);
    assert!(
        overridden(&w, j, a, Behaviour::Guard),
        "{:?}",
        w.obj(j).unwrap().beh
    );
    let mut back = None;
    for k in 0..120 {
        testkit::run(&mut w, 1);
        let o = w.obj(j).unwrap();
        if o.beh.current == Some(Behaviour::Guard) {
            back = Some(k);
            break;
        }
    }
    assert!(
        back.is_some(),
        "guard restored: {:?}",
        w.obj(j).unwrap().beh
    );
    let o = w.obj(j).unwrap();
    assert_eq!(o.beh.suspended, None);
    assert_eq!(o.target, None, "the target put aside, none");
}

// spec: S-0501 claim 52 (r154, Q-2792 answered: retaliation has no exception for an aircraft; standing on a pad and holding a contact add no condition, so an aircraft in guard on its pad retaliates as the one with no contact: attack current, the attacker its target, guard suspended, no destination, and the empty destination does not end the contact) and S-0215 claims 10 and 18; with the scatter property on guard and the player-scatter setting on, a retaliating aircraft is never sent case 1 of S-0215 claim 19
#[test]
fn claim_52_an_aircraft_on_its_pad_retaliates() {
    let mut r = rules(true, "6", 0);
    props(&mut r, Behaviour::Guard, true, true);
    for on_pad in [true, false] {
        let (mut w, j, a) = scene(&r, 1, 5, Behaviour::Guard);
        let pad = spawn(&mut w, Kind::Building, "PAD", 1, mid(SIZE, 4, 4), 0);
        if on_pad {
            w.obj_mut(j).unwrap().contact = Some(pad);
        }
        let t = w.tick;
        hit(&mut w, j, a);
        let o = w.obj(j).unwrap();
        assert_eq!(o.target, Some(Target::Object(a)), "on its pad {on_pad}");
        assert_eq!(o.beh.current, Some(Behaviour::Attack));
        assert_eq!(o.beh.suspended, Some(Behaviour::Guard));
        assert_eq!(o.dest, None);
        assert_eq!(o.contact, on_pad.then_some(pad), "the contact held");
        assert_ne!(o.beh.last_common_effects, Some(t), "no case 1 fall-back");
        assert!(glasnost_sim::air::on_ground(&w, j), "it does not move");
    }
}

// spec: S-0309 claim 18 and Terms (an object of a spawner type never retaliates, even while none of its spawned aircraft is launching or alive); the control without spawned aircraft retaliates
#[test]
fn claim_18_a_spawner_type_never_retaliates() {
    for spawner in [false, true] {
        let mut r = rules(false, "6", 0);
        if spawner {
            let c = &mut r.sim.vehicles.get_mut("CART").unwrap().combat;
            c.spawns = Some("JET".into());
            c.spawns_number = Some(1);
        }
        r.sim.vehicles.get_mut("CART").unwrap().primary = Some("Gun".into());
        let mut w = testkit::game_with(&r, SIZE, 5);
        let v = spawn(&mut w, Kind::Vehicle, "CART", 1, mid(SIZE, 0, 0), 0);
        let a = spawn(&mut w, Kind::Vehicle, "BUGGY", 0, mid(SIZE, -4, 0), 64);
        force(&mut w, a, Behaviour::Sleep);
        let want = if spawner {
            Retaliation::MayNot
        } else {
            Retaliation::Done
        };
        assert_eq!(acquire::retaliate(&mut w, v, a), want, "spawner {spawner}");
    }
}

// spec: S-0309 claim 18 (a slave master, here a slave miner by S-0402 claim 14 and Terms: gatherer, destination and an enslaves field, never retaliates); the control without the fields retaliates
#[test]
fn claim_18_a_slave_master_never_retaliates() {
    for master in [false, true] {
        let mut r = rules(false, "6", 0);
        let cart = r.sim.vehicles.get_mut("CART").unwrap();
        cart.primary = Some("Gun".into());
        if master {
            cart.econ.resource_gatherer = Some(true);
            cart.econ.resource_destination = Some(true);
            cart.econ.enslaves = Some("BUGGY".into());
        }
        let mut w = testkit::game_with(&r, SIZE, 5);
        let v = spawn(&mut w, Kind::Vehicle, "CART", 1, mid(SIZE, 0, 0), 0);
        let a = spawn(&mut w, Kind::Vehicle, "BUGGY", 0, mid(SIZE, -4, 0), 64);
        force(&mut w, a, Behaviour::Sleep);
        force(&mut w, v, Behaviour::Guard);
        assert_eq!(glasnost_sim::seams::is_slave_master(&w, v), master);
        let want = if master {
            Retaliation::MayNot
        } else {
            Retaliation::Done
        };
        assert_eq!(acquire::retaliate(&mut w, v, a), want, "master {master}");
    }
}
