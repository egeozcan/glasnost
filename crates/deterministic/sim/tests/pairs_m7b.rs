//! Interaction-pair scenario tests of M7b (PLAN, cross-component
//! interactions): each pair runs through the fixed tick pipeline on
//! synthetic data (invented types, weapons and warheads, every number our
//! own). The ledger is `docs/m7-claims/pairs-m7b.md`.

#[path = "lift_common/mod.rs"]
mod lift_common;

use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{AnimationType, Decimal, ParticleSystemType, ParticleType, WarheadType};
use glasnost_sim::combat::effects::Effect;
use glasnost_sim::infantry::Seq;
use glasnost_sim::rules::Kind;
use glasnost_sim::world::{ObjectId, Pos, World};
use glasnost_sim::{gas, lift, save, testkit};

fn register(r: &mut Ruleset, reg: &str, name: &str) {
    let l = r.registries.entry(reg.into()).or_default();
    if !l.iter().any(|n| n == name) {
        l.push(name.into());
    }
}

/// The special-hit animations every pair uses: BOOM (the explosion death,
/// kind 3) and BURN (the burning death, kind 4, a burning walker with 4
/// running frames, 80 frames and damage 0.25).
fn special_hits(r: &mut Ruleset) {
    register(r, "animations", "BOOM");
    r.sim.animations.insert(
        "BOOM".into(),
        AnimationType {
            frame_count: Some(10),
            ..Default::default()
        },
    );
    register(r, "animations", "BURN");
    r.sim.animations.insert(
        "BURN".into(),
        AnimationType {
            frame_count: Some(80),
            is_flaming_guy: Some(true),
            running_frames: Some(4),
            damage: Some(Decimal("0.25".into())),
            ..Default::default()
        },
    );
    r.sim.general.infantry_explode = Some("BOOM".into());
    r.sim.general.flaming_infantry = Some("BURN".into());
}

/// Names of the animations and walkers the effects since the last clear
/// show.
fn shown(w: &World) -> Vec<String> {
    w.effects
        .iter()
        .filter_map(|e| match e {
            Effect::Anim { name, .. } => Some(name.clone()),
            Effect::Walker {
                name, made: true, ..
            } => Some(format!("walker {name}")),
            _ => None,
        })
        .collect()
}

fn round_trip(w: &World) {
    let h = save::state_hash(w);
    let l = save::load(&save::save(w), w.rules_arc(), testkit::catalogue()).unwrap();
    assert_eq!(save::state_hash(&l), h);
}

// Pair: special-hit effects and the magnetic lift's fall (S-0615 × S-0607).
// spec: S-0607 claim 16 (a dropped unit's fall ending on infantry removes it at once, without death effects or death animations, whatever the crushing warhead's death kind)
// spec: S-0607 claims 16 to 18 (a fall ending on a vehicle: blocked land, crashed, two impact detonations, the first with the crushing warhead at ground level)
// spec: S-0615 claims 1, 2 and 4 (an infantry unit that a detonation of a kind-4 warhead kills becomes a burning walker at its position)
// spec: S-0303 claims 6 to 12 (ordinary area damage of the impact detonation reaches the neighbouring cell)
#[test]
fn a_lift_fall_crushes_silently_but_its_impact_makes_a_burning_walker() {
    use lift_common::*;
    let tweak = |r: &mut Ruleset| {
        special_hits(r);
        // The crushing warhead kills infantry by burning (kind 4); its spread reaches the next cell.
        let crush = r.sim.warheads.get_mut("CRUSH").unwrap();
        crush.inf_death = Some(4);
        crush.cell_spread = Some(d("1.5"));
        infantry(r, "FRAIL", |t| t.strength = Some(1));
    };
    // Part 1: the fall ends on an infantry unit, which is crushed silently.
    let mut w = world(tweak);
    let l = spawn(&mut w, Kind::Vehicle, "LIFTER", 0, at(0, 0));
    let v = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(6, 0));
    let under = spawn(&mut w, Kind::Infantry, "FRAIL", 0, at(3, 0));
    beam(&mut w, l, v);
    assert!(lift::seized(&w, v));
    assert!(run_until(&mut w, 400, |w| w
        .obj(v)
        .is_some_and(|o| o.pos.cell() == at(3, 0))
        && height(w, v) > 300));
    glasnost_sim::econ::assign_destination(
        &mut w,
        l,
        Some(glasnost_sim::world::Target::Cell(at(0, 1))),
    );
    let mut seen = Vec::new();
    assert!(run_until(&mut w, 400, |w| {
        seen.extend(shown(w));
        w.obj(under).is_none()
    }));
    assert!(
        !seen.iter().any(|n| n == "walker BURN" || n == "BOOM"),
        "crushed without a death animation: {seen:?}"
    );
    assert!(w.walkers.is_empty());
    round_trip(&w);

    // Part 2: the fall ends on a vehicle; the impact's crushing detonation kills an infantry unit
    // in the next cell, which burns.
    let mut w = world(tweak);
    let l = spawn(&mut w, Kind::Vehicle, "LIFTER", 0, at(0, 0));
    let v = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(6, 0));
    spawn(&mut w, Kind::Vehicle, "VICT", 0, at(3, 0));
    let near = spawn(&mut w, Kind::Infantry, "FRAIL", 0, at(3, 1));
    beam(&mut w, l, v);
    assert!(run_until(&mut w, 400, |w| w
        .obj(v)
        .is_some_and(|o| o.pos.cell() == at(3, 0))
        && height(w, v) > 300));
    glasnost_sim::econ::assign_destination(
        &mut w,
        l,
        Some(glasnost_sim::world::Target::Cell(at(0, 1))),
    );
    let at_death = w.obj(near).unwrap().pos;
    let mut walker = None;
    assert!(run_until(&mut w, 400, |w| {
        for e in &w.effects {
            if let Effect::Walker {
                name,
                pos,
                made: true,
                ..
            } = e
            {
                walker = Some((name.clone(), *pos));
            }
        }
        w.obj(near).is_none()
    }));
    testkit::run(&mut w, 1);
    for e in &w.effects {
        if let Effect::Walker {
            name,
            pos,
            made: true,
            ..
        } = e
        {
            walker = walker.or(Some((name.clone(), *pos)));
        }
    }
    assert_eq!(
        walker,
        Some(("BURN".to_string(), at_death)),
        "a burning walker where it died"
    );
    assert_eq!(w.walkers.len(), 1);
    round_trip(&w);
    // The walker runs its course and leaves.
    assert!(run_until(&mut w, 2000, |w| w.walkers.is_empty()));
}

/// The gas-pair rules: testkit's combat rules with the special-hit
/// animations, the shared gas system GasCloudSys holding FOG (a gas particle
/// of damage 7 with the virus warhead VENOM, infantry-death kind 8), the
/// virus-death animation PLAGUE (μ = 2, 6 frames, 3 FOG particles) and
/// FRAIL, an infantry type of strength 1.
fn gas_rules() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    special_hits(&mut r);
    register(&mut r, "particles", "FOG");
    r.sim.particles.insert(
        "FOG".into(),
        ParticleType {
            behaves_like: Some("gas".into()),
            max_ec: Some(30000),
            max_dc: Some(5),
            damage: Some(7),
            warhead: Some("VENOM".into()),
            state_ai_advance: Some(4),
            end_state_ai: Some(100),
            ..Default::default()
        },
    );
    register(&mut r, "particle_systems", "GasCloudSys");
    r.sim.particle_systems.insert(
        "GasCloudSys".into(),
        ParticleSystemType {
            behaves_like: Some("gas".into()),
            holds_what: Some("FOG".into()),
            ..Default::default()
        },
    );
    register(&mut r, "warheads", "VENOM");
    r.sim.warheads.insert(
        "VENOM".into(),
        WarheadType {
            verses: Some((0..11).map(|_| Decimal("1".into())).collect()),
            inf_death: Some(8),
            ..Default::default()
        },
    );
    register(&mut r, "animations", "PLAGUE");
    r.sim.animations.insert(
        "PLAGUE".into(),
        AnimationType {
            middle_frame: Some(2),
            frame_count: Some(6),
            spawns_particle: Some("FOG".into()),
            num_particles: Some(3),
            ..Default::default()
        },
    );
    r.sim.general.infantry_virus = Some("PLAGUE".into());
    let mut frail = r.sim.infantry["WALKER"].clone();
    frail.strength = Some(1);
    register(&mut r, "infantry", "FRAIL");
    r.sim.infantry.insert("FRAIL".into(), frail);
    r
}

/// A victim's death: who, where, the other animations made, the new clouds
/// and the new lasting virus animations.
type Death = (ObjectId, Pos, Vec<(String, Pos)>, usize, usize);

fn cloud_count(w: &World) -> usize {
    let s = w.gas.shared.expect("a shared gas system");
    w.gas.systems[&s].particles.len()
}

// Pair: special-hit effects and gas deaths (S-0615 × S-0613).
// spec: S-0613 claims 2, 13 and 14 (a cloud's hit with the virus warhead kills an infantry unit: the virus-death animation stays and delivers its count later)
// spec: S-0107 claim 17 and S-0613 claim 3 (a parachutist killed by a virus death: the extra virus-death animation, removed at once, and one cloud)
// spec: S-0615 claim 1 (a parachuting unit's death is kind 3: the explosion death animation as well; each dispatch makes at most one animation)
// spec: S-0615 claim 2 (the explosion animation at the unit's position)
#[test]
fn a_gas_cloud_kills_a_parachutist_with_one_cloud_and_an_explosion() {
    let r = gas_rules();
    let mut w = testkit::game_with(&r, 20, 7);
    let walker = testkit::spawn(
        &mut w,
        Kind::Infantry,
        "FRAIL",
        1,
        testkit::mid(20, -4, 0),
        0,
    );
    let chute = testkit::spawn(
        &mut w,
        Kind::Infantry,
        "FRAIL",
        1,
        testkit::mid(20, 4, 0),
        0,
    );
    let draw = w
        .rules()
        .gas
        .particles
        .iter()
        .find(|p| p.name == "FOG")
        .unwrap()
        .draw;
    // One cloud on each victim in turn, so that the deaths fall in different ticks.
    let mut deaths: Vec<Death> = Vec::new();
    for id in [walker, chute] {
        let p = w.obj(id).unwrap().pos;
        gas::middle_event_particle(&mut w, draw, Some((p.x, p.y, p.z)));
        for _ in 0..200 {
            if let Some(o) = w.obj_mut(chute)
                && let Some(i) = o.infantry.as_mut()
            {
                i.seq = Some(Seq::Parachute);
            }
            let pos = w.obj(id).unwrap().pos;
            let clouds = cloud_count(&w);
            let anims = w.gas.anims.len();
            testkit::run(&mut w, 1);
            if w.obj(id).is_none() {
                let names = w
                    .effects
                    .iter()
                    .filter_map(|e| match e {
                        Effect::Anim { name, pos, .. } if name != "PLAGUE" => {
                            Some((name.clone(), *pos))
                        }
                        _ => None,
                    })
                    .collect();
                deaths.push((
                    id,
                    pos,
                    names,
                    cloud_count(&w) - clouds,
                    w.gas.anims.len() - anims,
                ));
                break;
            }
        }
    }
    assert_eq!(deaths.len(), 2, "both died in the clouds");
    for (id, pos, names, new_clouds, new_anims) in &deaths {
        if *id == chute {
            assert_eq!(
                names,
                &[("BOOM".to_string(), *pos)],
                "the parachutist explodes where it died"
            );
            assert_eq!(*new_clouds, 1, "one extra cloud at once");
            assert_eq!(
                *new_anims, 0,
                "the extra virus animation is removed at once"
            );
        } else {
            assert!(names.is_empty(), "no special-hit animation: {names:?}");
            assert_eq!(*new_anims, 1, "the virus-death animation stays");
            assert_eq!(*new_clouds, 0, "its count comes later");
        }
    }
    assert!(w.walkers.is_empty());
    round_trip(&w);
}

/// Kills `id` at once with the kind-8 warhead VENOM, no attacker.
fn venom_kill(w: &mut World, id: ObjectId) {
    use glasnost_sim::combat::damage::{self, Hit};
    let h = w
        .rules()
        .combat
        .warheads
        .iter()
        .position(|x| x.name == "VENOM")
        .unwrap() as u32;
    damage::apply(
        w,
        id,
        Hit {
            amount: 10_000,
            warhead: Some(h),
            distance: 0,
            attacker: None,
            house: None,
            ignore_defences: false,
            no_escape: false,
        },
    );
}

// Pair: special-hit effects and gas deaths (S-0615 × S-0613), revision 3 (Q-1421 answered).
// spec: S-0615 claim 1 (Order with the extra virus effect: a "not human" type killed on the ground with death kind 8 makes its forced die-1 request and then one extra virus effect; a "not human" parachutist gets exactly one virus effect, not a second one in row 4) and S-0107 claim 17 row 4
#[test]
fn a_not_human_virus_death_makes_one_virus_effect() {
    let mut r = gas_rules();
    r.sim.infantry.get_mut("FRAIL").unwrap().not_human = Some(true);
    for chute in [false, true] {
        let mut w = testkit::game_with(&r, 20, 7);
        let id = testkit::spawn(&mut w, Kind::Infantry, "FRAIL", 1, testkit::mid(20, 0, 0), 0);
        if chute {
            w.obj_mut(id).unwrap().infantry.as_mut().unwrap().seq = Some(Seq::Parachute);
        }
        let (clouds, anims) = (cloud_count(&w), w.gas.anims.len());
        venom_kill(&mut w, id);
        assert_eq!(cloud_count(&w) - clouds, 1, "one extra virus particle (chute {chute})");
        assert_eq!(w.gas.anims.len(), anims, "no lasting virus animation (chute {chute})");
    }
}

// ---------------------------------------------------------------------------
// The drain (S-0608) with the tank bunker, the absorber, the grinder and
// power. What a drained bunker, absorber or grinder does with its occupants
// is request Q-1444; these tests check only what the specs say.

mod drain_pairs {
    use glasnost_map::CellXY;
    use glasnost_rules::schema::Ruleset;
    use glasnost_rules::sim::{
        BuildingType, Decimal, LocomotionClass, Locomotor, MovementZone, ProjectileType, UnitType,
        WeaponType,
    };
    use glasnost_sim::combat::check::Answer as Fire;
    use glasnost_sim::combat::damage::{self, Hit};
    use glasnost_sim::combat::fire;
    use glasnost_sim::orders::{self, Order};
    use glasnost_sim::rules::{Behaviour, Kind};
    use glasnost_sim::structures::{absorb, bunker, grinder};
    use glasnost_sim::transport::Answer;
    use glasnost_sim::world::{ObjectId, Pos, Target, World};
    use glasnost_sim::{drain, econ, objects, save, testkit, tick};

    const SIZE: i32 = 24;
    const ALPHA: u8 = 0;
    const BETA: u8 = 1;

    fn dec(s: &str) -> Decimal {
        Decimal(s.into())
    }

    fn building(r: &mut Ruleset, name: &str, f: impl FnOnce(&mut BuildingType)) {
        let mut t = BuildingType {
            strength: Some(1000),
            foundation: Some([2, 2]),
            ..Default::default()
        };
        // A build-up sprite, so that the building can be sold (S-0408 claim 4).
        t.econ.buildup = Some(true);
        t.econ.buildup_frames = Some(4);
        f(&mut t);
        super::register(r, "buildings", name);
        r.sim.buildings.insert(name.into(), t);
    }

    /// The combat ruleset with:
    /// - DISC, a balloon-hovering jumpjet whose secondary SUCKER is a
    ///   draining weapon (vertical projectile DROP);
    /// - PLANT (output 100, drainable); DEF (drain 20, needs power);
    /// - ABS, an infantry absorber with no base output and an extra output
    ///   of 40 per passenger, drainable;
    /// - GRND, a recycling building of a resource-destination type with a
    ///   drain of 10, drainable;
    /// - BUNK, a tank bunker with no impassable rows, drainable;
    /// - payouts every 10 ticks of 25 credits.
    fn rules() -> Ruleset {
        let mut r = testkit::combat_ruleset();
        r.sim.projectiles.insert(
            "DROP".into(),
            ProjectileType {
                vertical: Some(true),
                inviso: Some(true),
                ..Default::default()
            },
        );
        r.sim.weapons.insert(
            "SUCKER".into(),
            WeaponType {
                damage: Some(1),
                rof: Some(50),
                range: Some(dec("0.1")),
                speed: Some(100),
                projectile: Some("DROP".into()),
                warhead: Some("SA".into()),
                drain_weapon: Some(true),
                omni_fire: Some(true),
                ..Default::default()
            },
        );
        super::register(&mut r, "vehicles", "DISC");
        r.sim.vehicles.insert(
            "DISC".into(),
            UnitType {
                strength: Some(300),
                speed: Some(4),
                locomotion_class: Some(LocomotionClass::Hover),
                movement_zone: Some(MovementZone::Fly),
                locomotor: Some(Locomotor::Jumpjet),
                jumpjet: Some(true),
                jumpjet_height: Some(500),
                balloon_hover: Some(true),
                turret: Some(true),
                primary: Some("Gun".into()),
                secondary: Some("SUCKER".into()),
                ..Default::default()
            },
        );
        building(&mut r, "PLANT", |t| {
            t.econ.power = Some(100);
            t.combat.drainable = Some(true);
        });
        building(&mut r, "DEF", |t| {
            t.econ.power = Some(-20);
            t.powered = Some(true);
        });
        building(&mut r, "ABS", |t| {
            t.infantry_absorb = Some(true);
            t.passengers = Some(3);
            t.size_limit = Some(dec("1"));
            t.econ.power = Some(0);
            t.econ.extra_power = Some(40);
            t.combat.drainable = Some(true);
        });
        building(&mut r, "GRND", |t| {
            t.grinding = Some(true);
            t.econ.power = Some(-10);
            t.econ.resource_destination = Some(true);
            t.combat.drainable = Some(true);
        });
        building(&mut r, "BUNK", |t| {
            t.bunker = Some(true);
            t.number_impassable_rows = Some(0);
            t.combat.drainable = Some(true);
        });
        let g = &mut r.sim.general;
        g.drain_money_frame_delay = Some(10);
        g.drain_money_amount = Some(25);
        r
    }

    fn game() -> World {
        let mut w = testkit::game_with(&rules(), SIZE, 7);
        testkit::run(&mut w, 2);
        w
    }

    fn at(dx: i32, dy: i32) -> CellXY {
        testkit::mid(SIZE, dx, dy)
    }

    fn put(w: &mut World, kind: Kind, name: &str, house: u8, dx: i32, dy: i32) -> ObjectId {
        testkit::spawn(w, kind, name, house, at(dx, dy), 0)
    }

    /// An Alpha DISC moved over `b`'s top-left cell, making a drain
    /// attempt at it with its secondary.
    fn drain_with_disc(w: &mut World, b: ObjectId) -> ObjectId {
        let cell = w.obj(b).unwrap().pos.cell();
        let d = put(w, Kind::Vehicle, "DISC", ALPHA, 9, 9);
        objects::take_inside(w, d);
        let p = Pos::centre(cell, 0);
        let ground = w.surface_at(p.x, p.y);
        objects::place(w, d, Pos::new(p.x, p.y, ground + 200), 0);
        w.obj_mut(d).unwrap().target = Some(Target::Object(b));
        fire::shot(w, d, 1, Target::Object(b));
        assert_eq!(drain::partner(w, d), Some(b), "linked");
        d
    }

    fn output(w: &World, h: u8) -> i32 {
        w.houses[usize::from(h)].econ.power.output
    }

    fn kill(w: &mut World, id: ObjectId) {
        let hit = Hit {
            amount: w.obj(id).unwrap().health * 10,
            warhead: None,
            distance: 0,
            attacker: None,
            house: None,
            ignore_defences: true,
            no_escape: false,
        };
        damage::apply(w, id, hit);
    }

    fn round_trip(w: &World) {
        let l = save::load(&save::save(w), w.rules_arc(), w.catalogue_arc()).unwrap();
        assert_eq!(save::state_hash(&l), save::state_hash(w));
        assert_eq!(l.drain, w.drain);
    }

    // Pair: the drain and power (S-0608 × S-0405).
    // spec: S-0608 claim 5 (the building's house becomes due when the link forms) and S-0405 claims 5 and 7 (a drained building with output above 0: output O = 0 at the house's next working-out, low power with any drain)
    // spec: S-0405 claim 8 (a needs-power building with a base drain above 0 is not in operation under low power)
    // spec: S-0608 claims 13 and 14 (the drainer's killing hit ends the link; the house is due and its power is back at its next working-out)
    #[test]
    fn a_drained_plant_stops_the_defences_until_the_disc_dies() {
        let mut w = game();
        let plant = put(&mut w, Kind::Building, "PLANT", BETA, 0, 0);
        let def = put(&mut w, Kind::Building, "DEF", BETA, 6, 0);
        testkit::run(&mut w, 2);
        assert_eq!(output(&w, BETA), 100);
        assert!(econ::power::in_operation(&w, def));
        let d = drain_with_disc(&mut w, plant);
        testkit::run(&mut w, 2);
        assert_eq!(output(&w, BETA), 0, "blackout");
        assert!(!econ::power::in_operation(&w, def), "low power stops it");
        round_trip(&w);
        kill(&mut w, d);
        assert!(!drain::being_drained(&w, plant));
        testkit::run(&mut w, 2);
        assert_eq!(output(&w, BETA), 100);
        assert!(econ::power::in_operation(&w, def), "back in operation");
    }

    // Pair: the drain and the infantry absorber (S-0608 × S-0605).
    // spec: S-0405 claims 1 and 5 (an absorbing building's output is its base output plus the extra output per passenger; only a drained building with output above 0 blacks its house out)
    // spec: S-0605 claim 1 (an absorbing building refuses an entrant whose type hovers by balloon: the disc cannot go in)
    // spec: S-0608 claims 2 and 5 (any building of a drainable type over whose foundation the drainer is gets linked)
    #[test]
    fn a_drained_absorber_blacks_out_only_with_passengers() {
        let mut w = game();
        let plant = put(&mut w, Kind::Building, "PLANT", BETA, 6, 0);
        let def = put(&mut w, Kind::Building, "DEF", BETA, 6, 4);
        let abs = put(&mut w, Kind::Building, "ABS", BETA, 0, 0);
        testkit::run(&mut w, 2);
        let d = drain_with_disc(&mut w, abs);
        testkit::run(&mut w, 2);
        assert_eq!(output(&w, BETA), 100, "an empty absorber has no output");
        assert!(econ::power::in_operation(&w, def));
        assert_eq!(
            absorb::answer(&w, abs, d),
            Answer::Refuses,
            "no disc inside"
        );
        // The link ends; a passenger walks in (whether one may enter a
        // drained absorber is Q-1444), and a new link forms.
        drain::end(&mut w, abs);
        let u = put(&mut w, Kind::Infantry, "WALKER", BETA, 3, 0);
        while orders::blocked(&w) {
            tick::step(&mut w);
        }
        assert!(orders::issue(
            &mut w,
            glasnost_sim::world::HouseId(BETA),
            Order::Behaviour {
                actor: u,
                behaviour: Behaviour::Enter,
                attack_move: false,
                target: None,
                dest: Some(Target::Object(abs)),
            },
        ));
        let mut inside = false;
        for _ in 0..600 {
            tick::step(&mut w);
            if absorb::absorbed_in(&w, u) == Some(abs) {
                inside = true;
                break;
            }
        }
        assert!(inside, "the walker went in");
        testkit::run(&mut w, 2);
        // The plant's 100 and the passenger's 40, scaled by health (the
        // first disc, no longer draining, may shoot at the buildings).
        assert!(output(&w, BETA) > 100, "the passenger's extra output");
        drain_with_disc(&mut w, abs);
        testkit::run(&mut w, 2);
        assert_eq!(output(&w, BETA), 0, "blackout");
        assert!(!econ::power::in_operation(&w, def));
        round_trip(&w);
        let _ = plant;
    }

    // Pair: the drain and the grinder (S-0608 × S-0603).
    // spec: S-0603 claim 2 (a recycling building refuses an entrant whose type hovers by balloon: an own disc cannot be recycled)
    // spec: S-0608 claim 7 (a drained building of a resource-destination type pays its house's credits to the drainer's house on the payout ticks) and S-0408 claim 3 (a drained building cannot be sold)
    #[test]
    fn a_drained_grinder_pays_and_cannot_be_sold() {
        let mut w = game();
        let own = put(&mut w, Kind::Building, "GRND", ALPHA, -8, 0);
        let g = put(&mut w, Kind::Building, "GRND", BETA, 0, 0);
        let d = drain_with_disc(&mut w, g);
        assert_eq!(
            grinder::answer(&w, own, d),
            Answer::Refuses,
            "no disc recycled"
        );
        w.houses[usize::from(BETA)].credits = 1000;
        let before = w.houses[usize::from(ALPHA)].credits;
        testkit::run(&mut w, 30);
        let paid = w.houses[usize::from(ALPHA)].credits - before;
        assert!(paid >= 50 && paid % 25 == 0, "payouts of 25: {paid}");
        assert_eq!(w.houses[usize::from(BETA)].credits, 1000 - paid);
        assert!(!econ::sell::sellable(&w, g), "no sale while drained");
        round_trip(&w);
    }

    // Pair: the drain and the tank bunker (S-0608 × S-0606).
    // spec: S-0606 claim 2 row P6 (a bunker refuses an actor whose type has the balloon-hover flag: the disc cannot be bunkered)
    // spec: S-0405 claim 9 table T1 (a drained building's firing answer is illegal) and S-0408 claim 3 (no sale); S-0405 claim 5 (a drained building without output does not black its house out)
    #[test]
    fn a_drained_bunker_refuses_fire_and_sale_but_no_blackout() {
        let mut w = game();
        let _plant = put(&mut w, Kind::Building, "PLANT", BETA, 6, 0);
        let own = put(&mut w, Kind::Building, "BUNK", ALPHA, -8, 0);
        let k = put(&mut w, Kind::Building, "BUNK", BETA, 0, 0);
        testkit::run(&mut w, 2);
        let d = drain_with_disc(&mut w, k);
        assert!(!bunker::permits(&w, own, d), "no disc bunkered");
        testkit::run(&mut w, 2);
        assert_eq!(econ::power::firing_answer(&w, k), Some(Fire::Illegal));
        assert!(!econ::sell::sellable(&w, k));
        assert_eq!(output(&w, BETA), 100, "a bunker has no output");
        round_trip(&w);
    }
}
