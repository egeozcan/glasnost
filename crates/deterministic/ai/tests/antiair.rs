//! The own AI against drainers in the air (M7b AI round 3, PLAN §4.7): a
//! house that sees an enemy drainer in the air and has nothing that can
//! hit it builds the cheapest unit or defence whose weapon can (an
//! anti-air projectile, S-0306 claim 23; a versus value that is not zero,
//! S-0304 claim 1), one per drainer; a ruleset with no such type leaves
//! the brain as it was. The tests use the synthetic M7b skirmish (Alpha's
//! drainer DISC) and, for Beta, an invented anti-air vehicle FLAKT and
//! (M8 step B6) an invented anti-air defence FLAKD.

use std::sync::Arc;

use glasnost_ai::brain::Brain;
use glasnost_ai::fixture;
use glasnost_ai::fixture_m7b as kit;
use glasnost_ai::game::Game;
use glasnost_map::CellXY;
use glasnost_rules::sim::{ProjectileType, WeaponType};
use glasnost_sim::combat::weapon;
use glasnost_sim::econ::{TypeRef, tech};
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind, SimRules};
use glasnost_sim::world::{HouseId, ObjectId, Target, World};
use glasnost_sim::{drain, objects, testkit, tick};

const BETA: HouseId = HouseId(1);

/// The M7b rules with FLAKT: Beta's war-factory vehicle with an anti-air
/// gun (test data of our own).
fn flak_rules() -> Arc<SimRules> {
    Arc::new(SimRules::new(&flak_ruleset(), kit::MAP_NAME))
}

/// [`flak_rules`] plus FLAKD, a one-cell anti-air defence with the
/// needs-power flag and a drain, cheaper than FLAKT (test data of our own).
fn defence_rules() -> Arc<SimRules> {
    let mut r = flak_ruleset();
    let mut d = fixture::building(1, 1, 300, -10);
    d.powered = Some(true);
    d.primary = Some("FLAKW".into());
    r.registries
        .entry("buildings".into())
        .or_default()
        .push("FLAKD".into());
    r.sim.buildings.insert("FLAKD".into(), d);
    Arc::new(SimRules::new(&r, kit::MAP_NAME))
}

fn flak_ruleset() -> glasnost_rules::schema::Ruleset {
    let mut r = kit::ruleset();
    r.sim.projectiles.insert(
        "FLAKP".into(),
        ProjectileType {
            aa: Some(true),
            ag: Some(true),
            ..Default::default()
        },
    );
    let gun = r.sim.weapons["Gun"].clone();
    r.sim.weapons.insert(
        "FLAKW".into(),
        WeaponType {
            projectile: Some("FLAKP".into()),
            ..gun
        },
    );
    let mut flak = r.sim.vehicles["GUNR"].clone();
    flak.primary = Some("FLAKW".into());
    r.registries
        .entry("vehicles".into())
        .or_default()
        .push("FLAKT".into());
    r.sim.vehicles.insert("FLAKT".into(), flak);
    r
}

fn game(rules: Arc<SimRules>) -> Game {
    let mut g = Game::new(
        rules,
        &kit::map(),
        kit::MAP_NAME,
        fixture::catalogue(),
        kit::setup(5),
    )
    .expect("the M7b skirmish loads");
    while g.world.tick <= g.world.orders_blocked_through
        || !glasnost_ai::scripted::thinks_at(BETA, g.world.tick)
    {
        g.step();
    }
    g
}

fn home(w: &World, h: HouseId) -> CellXY {
    w.objects
        .values()
        .find(|o| o.owner == Some(h) && o.on_map)
        .map(|o| o.pos.cell())
        .unwrap()
}

fn spawn(w: &mut World, name: &str, kind: Kind, house: u8, dx: i32, dy: i32) -> ObjectId {
    let h = home(w, BETA);
    testkit::spawn(w, kind, name, house, CellXY::new(h.x + dx, h.y + dy), 0)
}

/// Beta with a war factory and an Alpha disc in the air over its base;
/// returns the world and the disc.
fn drained(rules: Arc<SimRules>) -> (World, ObjectId) {
    let g = game(rules);
    let mut w = g.world.clone();
    spawn(&mut w, "FACT", Kind::Building, 1, -6, 6);
    let disc = spawn(&mut w, "DISC", Kind::Vehicle, 0, 3, -3);
    let mut n = 0;
    while !weapon::airborne_by_height(&w, disc) && n < 600 {
        testkit::run(&mut w, 1);
        n += 1;
    }
    assert!(
        weapon::airborne_by_height(&w, disc),
        "the disc is in the air"
    );
    while !glasnost_ai::scripted::thinks_at(BETA, w.tick) {
        testkit::run(&mut w, 1);
    }
    (w, disc)
}

/// Beta's orders at this think and its anti-air builds.
fn think(w: &World) -> (Vec<Order>, u32) {
    let mut br = Brain::new(w, &kit::map(), BETA);
    let out = br.think(w);
    (out, br.special_uses().anti_air_builds)
}

fn builds(w: &World, out: &[Order], name: &str) -> bool {
    let i = w.rules().index_of(Kind::Vehicle, name).unwrap();
    out.iter().any(
        |o| matches!(o, Order::Build { what } if what.kind == Kind::Vehicle && what.index == i),
    )
}

// S-0306 claim 23 and S-0304 claim 1: a house that sees an enemy drainer in the air and has nothing that can hit it builds a unit that can.
#[test]
fn builds_an_anti_air_unit_against_a_drainer_in_the_air() {
    let (w, _) = drained(flak_rules());
    let (out, n) = think(&w);
    assert!(builds(&w, &out, "FLAKT"), "{out:?}");
    assert_eq!(n, 1);
}

// PLAN §4.7 (tuning of our own opponent): one able unit per drainer; with one on the map, none more.
#[test]
fn builds_none_with_one_able_unit_per_drainer() {
    let (mut w, _) = drained(flak_rules());
    spawn(&mut w, "FLAKT", Kind::Vehicle, 1, -3, 3);
    let (out, n) = think(&w);
    assert!(!builds(&w, &out, "FLAKT"), "{out:?}");
    assert_eq!(n, 0);
}

// S-0306 claim 23: without a type whose projectile is anti-air the brain builds nothing for it.
#[test]
fn builds_nothing_without_an_anti_air_type() {
    let (w, _) = drained(kit::rules());
    let (_, n) = think(&w);
    assert_eq!(n, 0);
}

// S-0306 claim 23: the disc in the air is a type the anti-air test finds hittable by FLAKT and by no other Beta type.
#[test]
fn only_the_anti_air_type_hits_the_disc() {
    let (w, disc) = drained(flak_rules());
    let armour = glasnost_sim::combat::veterancy::type_combat(&w, disc)
        .unwrap()
        .armour;
    let t = |n: &str| glasnost_sim::econ::TypeRef {
        kind: Kind::Vehicle,
        index: w.rules().index_of(Kind::Vehicle, n).unwrap(),
    };
    assert!(glasnost_ai::antiair::hits_air(&w, t("FLAKT"), armour));
    assert!(!glasnost_ai::antiair::hits_air(&w, t("GUNR"), armour));
}

/// [`flak_rules`] plus ELITT, a vehicle cheaper than FLAKT whose anti-air
/// gun is its elite primary only (test data of our own).
fn elite_rules() -> Arc<SimRules> {
    let mut r = flak_ruleset();
    let mut v = r.sim.vehicles["GUNR"].clone();
    v.elite_primary = Some("FLAKW".into());
    v.cost = r.sim.vehicles["FLAKT"].cost.map(|c| c / 2);
    r.registries
        .entry("vehicles".into())
        .or_default()
        .push("ELITT".into());
    r.sim.vehicles.insert("ELITT".into(), v);
    Arc::new(SimRules::new(&r, kit::MAP_NAME))
}

// S-0311 claims 6 and 11, S-0308 claim 2: a produced unit is not elite and uses its ordinary weapons, so a type whose anti-air weapon is elite only is not built against a drainer; the anti-air type is.
#[test]
fn a_type_that_hits_the_air_only_when_elite_is_not_built() {
    let (w, disc) = drained(elite_rules());
    let armour = glasnost_sim::combat::veterancy::type_combat(&w, disc)
        .unwrap()
        .armour;
    let elite = TypeRef {
        kind: Kind::Vehicle,
        index: w.rules().index_of(Kind::Vehicle, "ELITT").unwrap(),
    };
    assert!(!glasnost_ai::antiair::hits_air(&w, elite, armour));
    let (out, n) = think(&w);
    assert!(!builds(&w, &out, "ELITT"), "{out:?}");
    assert!(builds(&w, &out, "FLAKT"), "{out:?}");
    assert_eq!(n, 1);
}

/// [`flak_rules`] plus ENGRT, FLAKT at half its cost with the engineer
/// flag (test data of our own): an armed anti-air type the brain counts as
/// a non-fighter.
fn engineer_rules() -> Arc<SimRules> {
    let mut r = flak_ruleset();
    let mut v = r.sim.vehicles["FLAKT"].clone();
    v.cost = v.cost.map(|c| c / 2);
    v.engineer = Some(true);
    r.registries
        .entry("vehicles".into())
        .or_default()
        .push("ENGRT".into());
    r.sim.vehicles.insert("ENGRT".into(), v);
    Arc::new(SimRules::new(&r, kit::MAP_NAME))
}

// design: own AI (anti-air repeat, batch 4): while a superweapon building waits on its line for money, the brain gives no build order for a non-fighter (an engineer; M8 step B7), so the anti-air step does not pick one: with the cheaper anti-air type an engineer, the house orders the other, and the anti-air build it counts is one it gives. On main the engineer type was picked and counted at every think and the order dropped, so nothing was built (Allied against Yuri, map 2, seed 7, from tick 45 000).
#[test]
fn no_anti_air_non_fighter_is_picked_while_saving_for_a_superweapon() {
    let (mut w, _) = drained(engineer_rules());
    // A yard for SILO's line (the skirmish starts with units only).
    spawn(&mut w, "YARD", Kind::Building, 1, -10, -2);
    let silo = type_ref(&w, Kind::Building, "SILO");
    glasnost_sim::econ::production::build(&mut w, BETA, silo);
    w.houses[usize::from(BETA.0)].credits = 1;
    assert!(
        w.econ
            .lines
            .lines
            .iter()
            .any(|l| l.house == BETA && l.item.as_ref().is_some_and(|i| i.t == silo)),
        "SILO waits on Beta's line for money"
    );
    let (out, n) = think(&w);
    assert!(!builds(&w, &out, "ENGRT"), "{out:?}");
    assert!(builds(&w, &out, "FLAKT"), "{out:?}");
    assert_eq!(n, 1);
}

/// [`flak_rules`] plus NEVRT, a vehicle cheaper than FLAKT with GUNR's
/// gun and FLAKT's anti-air gun as a never-use secondary (test data of
/// our own: the never-use copy NEVRW).
fn never_rules() -> Arc<SimRules> {
    let mut r = flak_ruleset();
    let never = WeaponType {
        never_use: Some(true),
        ..r.sim.weapons["FLAKW"].clone()
    };
    r.sim.weapons.insert("NEVRW".into(), never);
    let mut v = r.sim.vehicles["GUNR"].clone();
    v.secondary = Some("NEVRW".into());
    v.cost = r.sim.vehicles["FLAKT"].cost.map(|c| c / 2);
    r.registries
        .entry("vehicles".into())
        .or_default()
        .push("NEVRT".into());
    r.sim.vehicles.insert("NEVRT".into(), v);
    Arc::new(SimRules::new(&r, kit::MAP_NAME))
}

// S-0308 claim 4 and table P row 3: a type whose secondary has the never-use flag fires slot 0 at every target, so an anti-air secondary of that kind answers no drainer; the brain builds the anti-air type instead.
#[test]
fn a_never_use_anti_air_secondary_is_not_built() {
    let (w, disc) = drained(never_rules());
    let armour = glasnost_sim::combat::veterancy::type_combat(&w, disc)
        .unwrap()
        .armour;
    let never = TypeRef {
        kind: Kind::Vehicle,
        index: w.rules().index_of(Kind::Vehicle, "NEVRT").unwrap(),
    };
    assert!(!glasnost_ai::antiair::hits_air(&w, never, armour));
    let (out, n) = think(&w);
    assert!(!builds(&w, &out, "NEVRT"), "{out:?}");
    assert!(builds(&w, &out, "FLAKT"), "{out:?}");
    assert_eq!(n, 1);
}

/// [`flak_rules`] plus two infantry types with WALKER's weapon as primary
/// and FLAKT's anti-air gun as secondary: SECIT plain, DEPIT with the
/// deploy-fire flag (test data of our own).
fn deploy_rules() -> Arc<SimRules> {
    let mut r = flak_ruleset();
    for (name, deploy) in [("SECIT", false), ("DEPIT", true)] {
        let mut i = r.sim.infantry["WALKER"].clone();
        i.secondary = Some("FLAKW".into());
        i.deploy_fire = Some(deploy);
        r.registries
            .entry("infantry".into())
            .or_default()
            .push(name.into());
        r.sim.infantry.insert(name.into(), i);
    }
    Arc::new(SimRules::new(&r, kit::MAP_NAME))
}

// S-0308 claim 8 and table P row 19: an infantry type with an anti-air secondary fires it at a drainer in the air, but one with the deploy-fire flag fires slot 0 until it deploys, so only the first hits the air when built.
#[test]
fn a_deploy_fire_infantry_type_does_not_hit_the_air_with_its_secondary() {
    let (w, disc) = drained(deploy_rules());
    let armour = glasnost_sim::combat::veterancy::type_combat(&w, disc)
        .unwrap()
        .armour;
    let t = |n: &str| TypeRef {
        kind: Kind::Infantry,
        index: w.rules().index_of(Kind::Infantry, n).unwrap(),
    };
    assert!(glasnost_ai::antiair::hits_air(&w, t("SECIT"), armour));
    assert!(!glasnost_ai::antiair::hits_air(&w, t("DEPIT"), armour));
}

// M8 step B6: what answers a drainer. Beta's brain plays alone on the
// real tick path (its orders issued as a person's), and an Alpha disc is
// sent with the attack click; Alpha's start discs are taken off the map.

/// The M7b skirmish world with `rules`, without Alpha's start discs, and
/// Beta with its yard vehicle deployed (a construction yard in its place),
/// a war factory and a power plant.
fn bench(rules: Arc<SimRules>) -> World {
    let g = game(rules);
    let mut w = g.world.clone();
    let mcv = w.rules().index_of(Kind::Vehicle, "MCV").unwrap();
    let (yv, at) = w
        .objects
        .values()
        .find(|o| o.owner == Some(BETA) && o.kind == Kind::Vehicle && o.type_index == mcv)
        .map(|o| (o.id, o.pos.cell()))
        .expect("Beta's yard vehicle");
    objects::remove(&mut w, yv);
    testkit::spawn(&mut w, Kind::Building, "YARD", 1, at, 0);
    let disc = w.rules().index_of(Kind::Vehicle, "DISC").unwrap();
    let discs: Vec<ObjectId> = w
        .objects
        .values()
        .filter(|o| o.kind == Kind::Vehicle && o.type_index == disc)
        .map(|o| o.id)
        .collect();
    for d in discs {
        objects::remove(&mut w, d);
    }
    spawn(&mut w, "FACT", Kind::Building, 1, -6, 6);
    spawn(&mut w, "POWR", Kind::Building, 1, 5, -5);
    w
}

/// Thinks watched for an anti-air order: long enough for Beta's base lines
/// to free (the control is the out-of-reach test, which orders within it).
const WATCH: i32 = 120;

/// Plays Beta's brain for `ticks` ticks through the tick.
fn play(w: &mut World, br: &mut Brain, ticks: i32) {
    for _ in 0..ticks {
        if br.thinks_at(w.tick) {
            for o in br.think(w) {
                orders::issue(w, BETA, o);
            }
        }
        tick::step(w);
    }
}

/// Sends Alpha's disc `d` at Beta's building `b` with the attack click
/// and plays until the disc drains it (S-0608 claim 2), at most 600 ticks.
fn drain_it(w: &mut World, br: &mut Brain, d: ObjectId, b: ObjectId) {
    orders::issue(
        w,
        HouseId(0),
        Order::Behaviour {
            actor: d,
            behaviour: Behaviour::Attack,
            attack_move: false,
            target: Some(Target::Object(b)),
            dest: None,
        },
    );
    for _ in 0..40 {
        play(w, br, 15);
        if drain::partner(w, d) == Some(b) {
            return;
        }
    }
    panic!("the disc did not drain the building");
}

fn type_ref(w: &World, kind: Kind, name: &str) -> TypeRef {
    TypeRef {
        kind,
        index: w.rules().index_of(kind, name).unwrap(),
    }
}

/// Whether a line of Beta's holds an item or a queued entry of `t`.
fn lined(w: &World, t: TypeRef) -> bool {
    w.econ
        .lines
        .lines
        .iter()
        .filter(|l| l.house == BETA)
        .any(|l| l.item.as_ref().is_some_and(|i| i.t == t) || l.queue.contains(&t))
}

/// Beta's objects of type `t` on the map.
fn of_type(w: &World, t: TypeRef) -> Vec<ObjectId> {
    w.objects
        .values()
        .filter(|o| o.owner == Some(BETA) && o.on_map && tech::type_of(o) == t)
        .map(|o| o.id)
        .collect()
}

// S-0405 claims 5 and 8, table T1 (a drained power plant blacks out the house; a defence with the needs-power flag is then not in operation and cannot fire): a dark anti-air defence beside the drainer answers nothing, and the house trains an anti-air unit rather than a defence the blackout would keep dark.
#[test]
fn a_dark_defence_does_not_answer_and_no_dark_defence_is_chosen() {
    let mut w = bench(defence_rules());
    let powr = spawn(&mut w, "POWR", Kind::Building, 1, -12, -6);
    spawn(&mut w, "FLAKD", Kind::Building, 1, -9, -6);
    let d = spawn(&mut w, "DISC", Kind::Vehicle, 0, -16, -6);
    let mut br = Brain::new(&w, &kit::map(), BETA);
    drain_it(&mut w, &mut br, d, powr);
    play(&mut w, &mut br, 30);
    assert!(
        w.houses[1].econ.power.drain_blackout,
        "the drained plant blacks out Beta (S-0405 claim 5)"
    );
    assert!(
        br.special_uses().anti_air_builds >= 1,
        "the dark defence answers nothing"
    );
    assert!(
        lined(&w, type_ref(&w, Kind::Vehicle, "FLAKT")),
        "an anti-air unit is trained"
    );
    assert!(
        !lined(&w, type_ref(&w, Kind::Building, "FLAKD")),
        "no defence the blackout keeps dark"
    );
}

// S-0301 claim 9 and S-0307 (the firing check with its range test): an anti-air defence in operation within reach of the drainer answers it, and nothing more is built.
#[test]
fn a_lit_defence_in_reach_answers_and_nothing_more_is_built() {
    let mut w = bench(defence_rules());
    let proc_ = spawn(&mut w, "PROC", Kind::Building, 1, -14, -4);
    spawn(&mut w, "FLAKD", Kind::Building, 1, -10, -4);
    let d = spawn(&mut w, "DISC", Kind::Vehicle, 0, -18, -4);
    let mut br = Brain::new(&w, &kit::map(), BETA);
    drain_it(&mut w, &mut br, d, proc_);
    play(&mut w, &mut br, 15 * WATCH);
    assert!(!w.houses[1].econ.power.drain_blackout);
    assert_eq!(br.special_uses().anti_air_builds, 0);
}

// S-0307 (range) and PLAN §4.7: an anti-air defence in operation but far from the drainer answers nothing; the house builds the cheapest anti-air type, a defence, and places it beside the drained building, where it reaches the drainer.
#[test]
fn a_defence_out_of_reach_does_not_answer_and_the_new_one_goes_beside_the_drainer() {
    let mut w = bench(defence_rules());
    let flakd = type_ref(&w, Kind::Building, "FLAKD");
    let proc_ = spawn(&mut w, "PROC", Kind::Building, 1, -14, -4);
    let far = spawn(&mut w, "FLAKD", Kind::Building, 1, 3, 3);
    let d = spawn(&mut w, "DISC", Kind::Vehicle, 0, -18, -4);
    let mut br = Brain::new(&w, &kit::map(), BETA);
    drain_it(&mut w, &mut br, d, proc_);
    // The brain's lines may be busy with its base; the order waits for one.
    for _ in 0..WATCH {
        if br.special_uses().anti_air_builds > 0 {
            break;
        }
        play(&mut w, &mut br, 15);
    }
    assert!(
        br.special_uses().anti_air_builds >= 1,
        "the far defence answers nothing"
    );
    let mut placed = None;
    for _ in 0..200 {
        play(&mut w, &mut br, 15);
        placed = of_type(&w, flakd).into_iter().find(|&x| x != far);
        if placed.is_some() {
            break;
        }
    }
    let placed = placed.expect("the new defence is placed");
    let at = w.obj(placed).unwrap().pos.cell();
    let pc = w.obj(proc_).unwrap().pos.cell();
    assert!(
        (at.x - pc.x).abs() <= 6 && (at.y - pc.y).abs() <= 6,
        "placed at {at:?}, the drained refinery at {pc:?}"
    );
    assert_eq!(
        br.special_uses().anti_air_builds,
        1,
        "the defence beside the drainer answers it: no second build"
    );
}

// S-0212 claim 4 (reach regions) and S-0301 claim 9: an anti-air unit of the army near home that can drive to the drainer answers it.
#[test]
fn an_anti_air_unit_near_home_answers() {
    let mut w = bench(defence_rules());
    let proc_ = spawn(&mut w, "PROC", Kind::Building, 1, -14, -4);
    spawn(&mut w, "FLAKT", Kind::Vehicle, 1, -3, 3);
    let d = spawn(&mut w, "DISC", Kind::Vehicle, 0, -18, -4);
    let mut br = Brain::new(&w, &kit::map(), BETA);
    drain_it(&mut w, &mut br, d, proc_);
    play(&mut w, &mut br, 15 * WATCH);
    assert_eq!(br.special_uses().anti_air_builds, 0);
}

// S-0301 claim 9 rank 9 and PLAN §4.7 (own AI): an anti-air unit that holds the drainer as its target but stands out of range of it, under no attack order and going nowhere, does not answer it (the near-home test above: the same unit answers while it is free).
#[test]
fn an_anti_air_unit_standing_off_the_drainer_does_not_answer() {
    let mut w = bench(defence_rules());
    let proc_ = spawn(&mut w, "PROC", Kind::Building, 1, -14, -4);
    let flak = spawn(&mut w, "FLAKT", Kind::Vehicle, 1, -1, 3);
    let d = spawn(&mut w, "DISC", Kind::Vehicle, 0, -18, -4);
    let mut br = Brain::new(&w, &kit::map(), BETA);
    drain_it(&mut w, &mut br, d, proc_);
    while !glasnost_ai::scripted::thinks_at(BETA, w.tick) {
        tick::step(&mut w);
    }
    let o = w.obj_mut(flak).unwrap();
    let at = o.pos.cell();
    o.target = Some(Target::Object(d));
    o.dest = Some(Target::Cell(at));
    o.beh.current = None;
    o.beh.queued = None;
    let t = Some(Target::Object(d));
    let slot = weapon::choose_slot(&w, flak, t);
    assert_eq!(
        glasnost_sim::combat::check::check(&w, flak, t, slot, true),
        glasnost_sim::combat::check::Answer::OutOfRange,
        "the unit stands out of range"
    );
    let p = br.antiair_probe(&w);
    assert_eq!(p.drainers, vec![(d, None)], "{p:?}");
}
