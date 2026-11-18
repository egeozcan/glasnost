//! #434: a person's tank in *guard*, with neither a target nor a
//! destination, hit by an enemy jumpjet infantry unit hovering in flight.
//! What it does is the spec's: a tank whose weapon cannot fire at an
//! airborne target may not retaliate (S-0309 claim 18 with S-0306 claim
//! 23), so it scatters only by S-0215 claim 19 case 2, which needs the
//! player-scatter setting for a player house; with that setting off it
//! stands and takes the hits. A tank with an anti-air secondary
//! (S-0308 Table P row 19) retaliates within S-0215 F7's distance and is
//! held beyond it, then scatters by claim 19 case 1 only with the setting
//! on. Synthetic data only.

mod jump_common;

use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{BehaviourProps, ProjectileType, WeaponType};
use glasnost_sim::combat::{acquire, check, fire, weapon};
use glasnost_sim::jump::{self, Phase};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit::{self, mid, spawn};
use glasnost_sim::trace::{self, What};
use glasnost_sim::world::{ObjectId, Target, World};
use jump_common::{SIZE, d};

/// The tanks' sight in cells: S-0215 F7's distance is (5 + 0.5) × 256 =
/// 1408 leptons.
const SIGHT: i32 = 5; // nocite: a synthetic type's sight

/// The jumpjet's hover height in leptons, at or above S-0306 claim 23's
/// airborne height.
const HOVER: i32 = 500; // nocite: a synthetic hover height

/// The jump fixtures with: the jumpjet armed with `Pack` (a ground-hitting
/// bullet that reaches the tanks from 7 cells), tank `TANKG` with only the
/// fixture's ground gun and tank `TANKA` with an anti-air secondary
/// `Flak` that reaches 3 cells; *guard* and *area guard* with the scatter
/// and retaliate properties; the player-scatter setting as given.
fn rules(player_scatter: bool) -> Ruleset {
    let mut r = jump_common::ruleset();
    r.sim.weapons.insert(
        "Pack".into(),
        WeaponType {
            damage: Some(30),
            rof: Some(20),
            range: Some(d("10")),
            speed: Some(100),
            projectile: Some("Bullet".into()),
            warhead: Some("SA".into()),
            ..Default::default()
        },
    );
    r.sim.weapons.insert(
        "Flak".into(),
        WeaponType {
            damage: Some(20),
            rof: Some(20),
            range: Some(d("3")),
            speed: Some(100),
            projectile: Some("FlakP".into()),
            warhead: Some("SA".into()),
            ..Default::default()
        },
    );
    r.sim.projectiles.insert(
        "FlakP".into(),
        ProjectileType {
            aa: Some(true),
            ag: Some(false),
            inviso: Some(true),
            ..Default::default()
        },
    );
    r.sim.infantry.get_mut("JJ").unwrap().primary = Some("Pack".into());
    let mut g = r.sim.vehicles.get("CART").unwrap().clone();
    g.strength = Some(2000);
    g.sight = Some(SIGHT);
    let mut a = g.clone();
    a.secondary = Some("Flak".into());
    r.registries
        .get_mut("vehicles")
        .unwrap()
        .extend(["TANKG", "TANKA"].map(String::from));
    r.sim.vehicles.insert("TANKG".into(), g);
    r.sim.vehicles.insert("TANKA".into(), a);
    for b in [Behaviour::Guard, Behaviour::AreaGuard] {
        r.sim.behaviours.insert(
            b.key().into(),
            BehaviourProps {
                scatter: Some(true),
                retaliate: Some(true),
                ..Default::default()
            },
        );
    }
    r.sim.general.player_scatter = Some(player_scatter);
    r
}

/// The scene: the tank of house 0 (a player house, so human-controlled,
/// S-0215 Terms) in *guard* in the middle, the jumpjet of house 1 `dx`
/// cells west hovering at [`HOVER`] in *sleep*, so that it fires only the
/// shots the test gives it.
fn scene(tank: &str, dx: i32, player_scatter: bool) -> (World, ObjectId, ObjectId) {
    let mut w = testkit::game_with(&rules(player_scatter), SIZE, 7);
    testkit::run(&mut w, 4);
    let v = spawn(&mut w, Kind::Vehicle, tank, 0, mid(SIZE, 0, 0), 0);
    testkit::run(&mut w, 4);
    let j = spawn(&mut w, Kind::Infantry, "JJ", 1, mid(SIZE, -dx, 0), 0);
    let (t, stale) = (w.tick, w.rules().quirks.stale_suspended_revert);
    w.obj_mut(j).unwrap().beh.force(Behaviour::Sleep, t, stale);
    jump::start_in_flight(&mut w, j, Phase::Hovering, HOVER);
    trace::switch_on(&mut w);
    testkit::run(&mut w, 2);
    let o = w.obj(v).unwrap();
    assert_eq!((o.target, o.dest, o.beh.seen()), (None, None, Some(Behaviour::Guard)));
    assert!(weapon::airborne(&w, j), "the jumpjet is airborne");
    (w, v, j)
}

/// What the tank did after one hit by the jumpjet.
#[derive(Debug)]
struct After {
    lost: i32,
    behaviour: Option<Behaviour>,
    target: Option<Target>,
    dest: Option<Target>,
    /// The scatter requests the tank answered, accepted or not.
    scatters: Vec<bool>,
}

/// One shot of the jumpjet at the tank, run to its detonation.
fn hit(w: &mut World, v: ObjectId, j: ObjectId) -> After {
    let before = w.obj(v).unwrap().health;
    let _ = trace::take(w);
    w.obj_mut(j).unwrap().target = Some(Target::Object(v));
    assert!(fire::shot(w, j, 0, Target::Object(v)), "the jumpjet fires");
    let t0 = w.tick;
    while !w.projectiles.is_empty() {
        testkit::run(w, 1);
        assert!(w.tick < t0 + 120, "the shot detonates");
    }
    w.obj_mut(j).unwrap().target = None;
    let scatters = trace::take(w)
        .into_iter()
        .filter(|e| e.unit == v)
        .filter_map(|e| match e.what {
            What::Asked { accepted, .. } => Some(accepted),
            _ => None,
        })
        .collect();
    let o = w.obj(v).unwrap();
    After {
        lost: before - o.health,
        behaviour: o.beh.seen(),
        target: o.target,
        dest: o.dest,
        scatters,
    }
}

/// The weapon the tank would use against the jumpjet and the firing
/// check's answer without range.
fn choice(w: &World, v: ObjectId, j: ObjectId) -> (usize, check::Answer) {
    let t = Some(Target::Object(j));
    let (slot, _) = weapon::would_use(w, v, t);
    (slot, check::check(w, v, t, slot, false))
}

/// A tank with only a ground weapon may not retaliate against a hovering
/// jumpjet: its weapon is the primary (Table P row 3) and the firing check
/// answers *illegal* for a target at the airborne height (S-0306 claim 23),
/// which S-0309 claim 18 excludes. With the player-scatter setting off,
/// claim 19 case 2 sends no scatter to a player house's tank, so it stands
/// in *guard* with no target and no destination, at either distance and at
/// every hit: the scene of #434.
// spec: S-0309 claim 18, S-0306 claim 23, S-0308 Table P row 3, S-0215 claims 18 and 19 case 2
#[test]
fn issue_434_a_ground_only_tank_stands_under_a_hovering_jumpjet() {
    for dx in [4, 7] {
        let (mut w, v, j) = scene("TANKG", dx, false);
        assert_eq!(choice(&w, v, j), (0, check::Answer::Illegal), "{dx} cells");
        for n in 0..3 {
            let a = hit(&mut w, v, j);
            assert!(a.lost > 0, "{dx} cells, hit {n}: {a:?}");
            assert_eq!(a.behaviour, Some(Behaviour::Guard), "{dx} cells, hit {n}");
            assert_eq!((a.target, a.dest), (None, None), "{dx} cells, hit {n}");
            assert!(a.scatters.is_empty(), "{dx} cells, hit {n}: {a:?}");
            testkit::run(&mut w, 30);
            let o = w.obj(v).unwrap();
            assert_eq!(o.pos.cell(), mid(SIZE, 0, 0), "{dx} cells, hit {n}: it stays");
        }
    }
}

/// With the player-scatter setting on, the same tank is sent claim 19
/// case 2's scatter at a hit that finds it standing free, and accepts it
/// (a forced request, S-0207 claim 16): it moves away. A hit while it is
/// still driving there sends none (case 2: its locomotor is moving and it
/// has a destination).
// spec: S-0215 claim 19 case 2 (the player-scatter setting; not moving; neither a target nor a destination)
#[test]
fn issue_434_a_ground_only_tank_scatters_with_the_player_scatter_setting() {
    for dx in [4, 7] {
        let (mut w, v, j) = scene("TANKG", dx, true);
        let a = hit(&mut w, v, j);
        assert_eq!(a.scatters, vec![true], "{dx} cells: {a:?}");
        assert!(matches!(a.dest, Some(Target::Cell(_))), "{dx} cells: {a:?}");
        assert_eq!(a.target, None);
        testkit::run(&mut w, 30);
        let a = hit(&mut w, v, j);
        assert!(a.dest.is_some() && a.scatters.is_empty(), "{dx} cells, still driving: {a:?}");
    }
}

/// A tank with an anti-air secondary uses it against the hovering jumpjet
/// (Table P row 19), whose firing check answers neither *illegal* nor
/// *impossible*. Within F7's distance (1139 leptons at 4 cells, at most
/// 1408) it retaliates although the secondary does not reach: an override
/// to *attack* with the jumpjet as target, and no scatter.
// spec: S-0308 Table P row 19, S-0309 claim 18, S-0215 claim 18 and F7, claim 10, claim 19 (an actor that retaliates is never sent case 1's request)
#[test]
fn issue_434_an_anti_air_tank_retaliates_within_f7() {
    for ps in [false, true] {
        let (mut w, v, j) = scene("TANKA", 4, ps);
        let (slot, answer) = choice(&w, v, j);
        assert_eq!(slot, 1);
        assert!(!matches!(answer, check::Answer::Illegal | check::Answer::Impossible), "{answer:?}");
        let f7 = i64::from((2 * SIGHT + 1) * 128); // spec: S-0215 F7 ((S + 0.5) × 256 leptons)
        assert!(acquire::rho3(&w, v, j) <= f7);
        let a = hit(&mut w, v, j);
        assert_eq!(a.behaviour, Some(Behaviour::Attack), "player-scatter {ps}: {a:?}");
        assert_eq!(a.target, Some(Target::Object(j)));
        assert!(a.scatters.is_empty(), "{a:?}");
    }
}

/// Beyond F7's distance (1860 leptons at 7 cells) the anti-air tank may
/// retaliate but is held back, since its secondary does not reach: it keeps
/// *guard* with no target. Claim 19 case 1 then sends the scatter only with
/// the player-scatter setting on.
// spec: S-0215 claim 18 and F7 (a human-controlled owner's actor that cannot reach overrides only within (S + 0.5) × 256), claim 19 case 1
#[test]
fn issue_434_an_anti_air_tank_beyond_f7_is_held() {
    for ps in [false, true] {
        let (mut w, v, j) = scene("TANKA", 7, ps);
        assert_eq!(choice(&w, v, j).0, 1);
        let f7 = i64::from((2 * SIGHT + 1) * 128); // spec: S-0215 F7 ((S + 0.5) × 256 leptons)
        assert!(acquire::rho3(&w, v, j) > f7);
        let a = hit(&mut w, v, j);
        assert_eq!(a.behaviour, Some(Behaviour::Guard), "player-scatter {ps}: {a:?}");
        assert_eq!(a.target, None);
        if ps {
            assert_eq!(a.scatters, vec![true], "{a:?}");
            assert!(matches!(a.dest, Some(Target::Cell(_))));
        } else {
            assert!(a.scatters.is_empty() && a.dest.is_none(), "{a:?}");
        }
    }
}
