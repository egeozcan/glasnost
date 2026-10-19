//! Interaction-pair scenario tests of M7b (PLAN, cross-component
//! interactions): each pair runs through the fixed tick pipeline on
//! synthetic data (invented types, weapons and warheads, every number our
//! own). The ledger is `docs/m7-claims/pairs-m7b.md`.

#[path = "lift_common/mod.rs"]
mod lift_common;

use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{AnimationType, Decimal, ParticleSystemType, ParticleType, WarheadType};
use glasnost_rules::sim_econ::SuperweaponType;
use glasnost_sim::combat::effects::Effect;
use glasnost_sim::infantry::Seq;
use glasnost_sim::rules::Kind;
use glasnost_sim::world::{ObjectId, Pos, World};
use glasnost_sim::{gas, lift, save, superweapons, testkit};

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

/// Adds the force shield: SHIELD (kind and action ForceShield), radius 3
/// cells, 80 ticks of protection, a blackout of 40 ticks, the fade sound 20
/// ticks before the end, the invoke animation SHIELDA.
fn add_shield(r: &mut Ruleset) {
    register(r, "superweapons", "SHIELD");
    r.sim.superweapons.insert(
        "SHIELD".into(),
        SuperweaponType {
            kind: Some("ForceShield".into()),
            action: Some("ForceShield".into()),
            recharge_time: Some(Decimal("0.1".into())),
            ..Default::default()
        },
    );
    register(r, "animations", "SHIELDA");
    r.sim
        .animations
        .insert("SHIELDA".into(), AnimationType::default());
    let g = &mut r.sim.general.superweapons;
    g.force_shield_radius = Some(3);
    g.force_shield_duration = Some(80);
    g.force_shield_blackout_duration = Some(40);
    g.force_shield_play_fade_sound_time = Some(20);
    g.force_shield_invoke_anim = Some("SHIELDA".into());
}

/// House `h`'s force-shield launch at `c` (the record and readiness are
/// S-0512's and play no part here).
fn shield(w: &mut World, h: u8, c: glasnost_map::CellXY) {
    let i = w
        .rules()
        .superweapons
        .first_of_kind(superweapons::SwKind::ForceShield)
        .expect("the force shield type");
    superweapons::protect::force_shield(w, glasnost_sim::world::HouseId(h), i, c);
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
                if id == chute {
                    // spec: S-0615 claim 1 (revision 3, Q-1421 answered: a parachutist's virus effect is created before its kind-3 death animation)
                    let order: Vec<&str> = w
                        .effects
                        .iter()
                        .filter_map(|e| match e {
                            Effect::Anim { name, .. } => Some(name.as_str()),
                            _ => None,
                        })
                        .collect();
                    assert_eq!(order, ["PLAGUE", "BOOM"], "the virus effect first");
                }
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
        let id = testkit::spawn(
            &mut w,
            Kind::Infantry,
            "FRAIL",
            1,
            testkit::mid(20, 0, 0),
            0,
        );
        if chute {
            w.obj_mut(id).unwrap().infantry.as_mut().unwrap().seq = Some(Seq::Parachute);
        }
        let (clouds, anims) = (cloud_count(&w), w.gas.anims.len());
        venom_kill(&mut w, id);
        assert_eq!(
            cloud_count(&w) - clouds,
            1,
            "one extra virus particle (chute {chute})"
        );
        assert_eq!(
            w.gas.anims.len(),
            anims,
            "no lasting virus animation (chute {chute})"
        );
    }
}

// Pair: gas clouds and the force shield (S-0613 × S-0513).
// spec: S-0613 claim 13 (a gas particle hits every object listed in the ground layer of its cell, buildings included, with ordinary damage that does not ignore defences)
// spec: S-0513 claim 31 (the launching house's building within the radius is protected) and S-0302 claim 8 (a hit on a protected object that does not ignore defences changes no health)
#[test]
fn a_gas_cloud_does_nothing_to_a_shielded_building() {
    let mut r = gas_rules();
    add_shield(&mut r);
    register(&mut r, "buildings", "TOWER");
    r.sim.buildings.insert(
        "TOWER".into(),
        glasnost_rules::sim::BuildingType {
            strength: Some(1000),
            foundation: Some([1, 1]),
            ..Default::default()
        },
    );
    let mut w = testkit::game_with(&r, 20, 7);
    let shielded = testkit::spawn(
        &mut w,
        Kind::Building,
        "TOWER",
        0,
        testkit::mid(20, 0, 0),
        0,
    );
    let open = testkit::spawn(
        &mut w,
        Kind::Building,
        "TOWER",
        0,
        testkit::mid(20, 0, 6),
        0,
    );
    testkit::run(&mut w, 3);
    shield(&mut w, 0, testkit::mid(20, 0, 0));
    assert!(superweapons::protected(&w, shielded) && !superweapons::protected(&w, open));
    let draw = w
        .rules()
        .gas
        .particles
        .iter()
        .find(|p| p.name == "FOG")
        .unwrap()
        .draw;
    for id in [shielded, open] {
        let p = w.obj(id).unwrap().pos;
        gas::middle_event_particle(&mut w, draw, Some((p.x, p.y, p.z)));
    }
    let h = |w: &World, id| w.obj(id).unwrap().health;
    let (hs, ho) = (h(&w, shielded), h(&w, open));
    for _ in 0..60 {
        if h(&w, open) < ho {
            break;
        }
        testkit::run(&mut w, 1);
    }
    assert!(h(&w, open) < ho, "the cloud hurts the open building");
    assert!(superweapons::protected(&w, shielded));
    assert_eq!(h(&w, shielded), hs, "and not the shielded one");
    round_trip(&w);
}

// ---------------------------------------------------------------------------
// The drain (S-0608) with the tank bunker, the absorber, the grinder and
// power. Being drained adds no occupant-specific gate to them (S-0608
// claim 18, Q-1444 answered).

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
        let mut tank = r.sim.vehicles["CRUSHER"].clone();
        tank.crusher = None;
        tank.primary = Some("Gun".into());
        super::register(&mut r, "vehicles", "TANK");
        r.sim.vehicles.insert("TANK".into(), tank);
        let mut meat = r.sim.infantry["WALKER"].clone();
        meat.cost = Some(200);
        super::register(&mut r, "infantry", "MEAT");
        r.sim.infantry.insert("MEAT".into(), meat);
        let g = &mut r.sim.general;
        g.drain_money_frame_delay = Some(10);
        g.drain_money_amount = Some(25);
        r
    }

    /// Orders `u` of house `h` into `b` with behaviour `how` (enter, or
    /// the recycle order's eaten, S-0214 table M) and steps until `done`
    /// holds (at most 1500 ticks).
    fn enter_until(
        w: &mut World,
        h: u8,
        (u, how): (ObjectId, Behaviour),
        b: ObjectId,
        done: impl Fn(&World) -> bool,
    ) {
        while orders::blocked(w) {
            tick::step(w);
        }
        assert!(orders::issue(
            w,
            glasnost_sim::world::HouseId(h),
            Order::Behaviour {
                actor: u,
                behaviour: how,
                attack_move: false,
                target: None,
                dest: Some(Target::Object(b)),
            },
        ));
        for _ in 0..1500 {
            if done(w) {
                return;
            }
            tick::step(w);
        }
        assert!(done(w), "entered");
    }

    fn sleep(w: &mut World, id: ObjectId) {
        let t = w.tick;
        w.obj_mut(id).unwrap().beh.force(Behaviour::Sleep, t, true);
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
        // spec: S-0608 claim 18 (Q-1444: absorber entry keeps S-0605's rules while the absorber is drained; the link neither ejects the passenger nor stops its approach)
        // The disc holds still, so that its other weapon leaves the walker alone.
        let t = w.tick;
        w.obj_mut(d).unwrap().beh.force(Behaviour::Sleep, t, true);
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
        assert!(inside, "the walker went in while drained");
        assert_eq!(drain::partner(&w, d), Some(abs), "the link stays");
        testkit::run(&mut w, 2);
        // With a passenger the drained absorber has output, so the house blacks out.
        assert_eq!(output(&w, BETA), 0, "blackout");
        assert!(!econ::power::in_operation(&w, def));
        assert_eq!(absorb::absorbed_in(&w, u), Some(abs), "not ejected");
        // The link ends: the plant's 100 and the passenger's 40, scaled by health.
        drain::end(&mut w, abs);
        testkit::run(&mut w, 2);
        assert!(output(&w, BETA) > 100, "the passenger's extra output");
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

    // Pair: the drain and a bunkered vehicle (S-0608 × S-0606).
    // spec: S-0608 claim 18 (Q-1444: a bunkered vehicle keeps its own firing check, the drained bunker's illegal answer is not forwarded to it; the link neither ejects it nor ends its bunker link) and S-0606 claim 9 (its range bonus stays in force)
    #[test]
    fn a_drained_bunker_keeps_its_vehicle_firing() {
        let mut w = game();
        let k = put(&mut w, Kind::Building, "BUNK", BETA, 0, 0);
        let u = put(&mut w, Kind::Vehicle, "TANK", BETA, 6, 1);
        testkit::run(&mut w, 2);
        enter_until(&mut w, BETA, (u, Behaviour::Enter), k, |w| {
            bunker::bunkered_in(w, u) == Some(k)
        });
        let range = |w: &World| {
            let wpn = glasnost_sim::combat::weapon::slot_weapon(w, u, 0);
            let p = w.obj(u).unwrap().pos;
            let far = CellXY::new((p.x + 6 * 256 + 128) / 256, p.y / 256);
            glasnost_sim::combat::range::in_range(w, u, wpn, Some(Target::Cell(far)), None)
        };
        let bonus = range(&w);
        let d = drain_with_disc(&mut w, k);
        sleep(&mut w, d);
        let foe = put(&mut w, Kind::Vehicle, "TANK", ALPHA, 5, -3);
        testkit::run(&mut w, 2);
        assert_eq!(econ::power::firing_answer(&w, k), Some(Fire::Illegal));
        assert_ne!(
            glasnost_sim::combat::check::check(&w, u, Some(Target::Object(foe)), 0, false),
            Fire::Illegal,
            "its own check"
        );
        assert_eq!(range(&w), bonus, "the bunker's range change stays");
        testkit::run(&mut w, 30);
        assert_eq!(bunker::bunkered_in(&w, u), Some(k), "not ejected");
        assert_eq!(drain::partner(&w, d), Some(k));
        round_trip(&w);
    }

    // Pair: the drain and grinder recycling (S-0608 × S-0603).
    // spec: S-0608 claim 18 (Q-1444: grinder entry, consumption and recycling credit keep S-0603's rules while drained; the credit goes to the grinder's owner, and only a later ordinary payout can take it, claim 7)
    #[test]
    fn a_drained_grinder_still_recycles_for_its_owner() {
        let mut w = game();
        let g = put(&mut w, Kind::Building, "GRND", BETA, 0, 0);
        let u = put(&mut w, Kind::Infantry, "MEAT", BETA, 5, 0);
        testkit::run(&mut w, 2);
        let d = drain_with_disc(&mut w, g);
        sleep(&mut w, d);
        assert_eq!(
            grinder::answer(&w, g, u),
            Answer::Accepts,
            "drained, it accepts"
        );
        w.houses[usize::from(BETA)].credits = 1000;
        let alpha = w.houses[usize::from(ALPHA)].credits;
        enter_until(&mut w, BETA, (u, Behaviour::Eaten), g, |w| {
            w.obj(u).is_none_or(|o| !o.on_map)
        });
        testkit::run(&mut w, 2);
        let paid = w.houses[usize::from(ALPHA)].credits - alpha;
        assert_eq!(paid % 25, 0, "only payouts reach the drainer: {paid}");
        let refund = w.houses[usize::from(BETA)].credits - (1000 - paid);
        assert!(
            refund > 0,
            "the recycling credit went to the grinder's owner"
        );
        assert_eq!(drain::partner(&w, d), Some(g), "the link stays");
        round_trip(&w);
    }

    // Pair: the drain and the force shield (S-0608 × S-0513), interim of request Q-1507.
    // spec: S-0513 claim 31 (the launching house's building within the radius is protected) and claim 27 (the list of what protection refuses besides damage has no drain row) with claim 28 (a player house may fire at a protected object)
    // spec: S-0608 claim 3 (the firing check's drain refusals: the own building, already drained, not drainable; no protection row) and claim 5 (the attempt forms the link)
    // Q-1507 (open): whether protection refuses or ends a drain link; interim: the literal reading, not refused, the link forms and holds while protected.
    #[test]
    fn a_shielded_plant_is_drained_under_the_q1507_interim() {
        let mut r = rules();
        super::add_shield(&mut r);
        let mut w = testkit::game_with(&r, SIZE, 7);
        testkit::run(&mut w, 2);
        let p = put(&mut w, Kind::Building, "PLANT", BETA, 0, 0);
        testkit::run(&mut w, 2);
        super::shield(&mut w, BETA, at(0, 0));
        assert!(glasnost_sim::superweapons::protected(&w, p));
        // The disc over the plant, its drain weapon checked before the
        // attempt: not illegal.
        let d = put(&mut w, Kind::Vehicle, "DISC", ALPHA, 9, 9);
        objects::take_inside(&mut w, d);
        let c = Pos::centre(w.obj(p).unwrap().pos.cell(), 0);
        let ground = w.surface_at(c.x, c.y);
        objects::place(&mut w, d, Pos::new(c.x, c.y, ground + 200), 0);
        w.obj_mut(d).unwrap().target = Some(Target::Object(p));
        assert_ne!(
            glasnost_sim::combat::check::check(&w, d, Some(Target::Object(p)), 1, false),
            Fire::Illegal,
            "the firing check does not refuse"
        );
        fire::shot(&mut w, d, 1, Target::Object(p));
        assert_eq!(drain::partner(&w, d), Some(p), "the link forms");
        sleep(&mut w, d);
        testkit::run(&mut w, 20);
        assert!(glasnost_sim::superweapons::protected(&w, p));
        assert_eq!(drain::partner(&w, d), Some(p), "and holds while protected");
        round_trip(&w);
    }
}

// ---------------------------------------------------------------------------
// Pairs with the genetic mutator (S-0611, M7b step 3.4): its cell-mode
// launch (`mutation::launch`) against a controlled unit and a controller
// (S-0601), a bunkered vehicle (S-0606), an absorbed passenger (S-0605) and
// a seized vehicle (S-0607).
// ---------------------------------------------------------------------------

mod mutator_pairs {
    use super::register;
    use glasnost_map::CellXY;
    use glasnost_rules::schema::Ruleset;
    use glasnost_rules::sim::{
        AnimationType, BuildingType, Decimal, UnitType, WarheadType, WeaponType,
    };
    use glasnost_sim::combat::effects::Effect;
    use glasnost_sim::mutation::{self, MakerEvent};
    use glasnost_sim::orders::{self, Order};
    use glasnost_sim::rules::{Behaviour, Kind};
    use glasnost_sim::structures::{absorb, bunker};
    use glasnost_sim::targeting::{self, ClickOn, Modifiers};
    use glasnost_sim::world::{HouseId, ObjectId, Target, World};
    use glasnost_sim::{lift, mind, save, testkit, tick};

    const SIZE: i32 = 24;
    const ALPHA: u8 = 0;
    const BETA: u8 = 1;

    fn dec(s: &str) -> Decimal {
        Decimal(s.into())
    }

    /// The combat ruleset with: the takeover warhead MIND and its weapon
    /// Ray3 (room for three victims) on infantry CLONE; GRUNT (unarmed
    /// infantry, also the made type); TGT (an unarmed vehicle of strength
    /// 30 000) and GATT (a turreted armed vehicle, bunker-eligible); BIO (an
    /// infantry absorber for three) and BUNK (a tank bunker); MAGL (a
    /// lifter whose beam MagRay seizes vehicles); and the mutation values:
    /// cell mode, the cell warhead MUT (infantry-death kind 9), the
    /// conversion animation MUTA (index 0, four frames) making GRUNT.
    pub(super) fn rules() -> Ruleset {
        let mut r = testkit::combat_ruleset();
        let ones = || Some(vec![dec("1"); 11]);
        for (n, wh) in [
            (
                "MIND",
                WarheadType {
                    verses: ones(),
                    mind_control: Some(true),
                    ..Default::default()
                },
            ),
            (
                "MUT",
                WarheadType {
                    verses: ones(),
                    inf_death: Some(9),
                    ..Default::default()
                },
            ),
            (
                "LIFTWH",
                WarheadType {
                    verses: ones(),
                    is_locomotor: Some(true),
                    locomotor: Some("jumpjet".into()),
                    ..Default::default()
                },
            ),
        ] {
            r.sim.warheads.insert(n.into(), wh);
        }
        for (n, wh, dmg, mag) in [("Ray3", "MIND", 3, false), ("MagRay", "LIFTWH", 50, true)] {
            r.sim.weapons.insert(
                n.into(),
                WeaponType {
                    damage: Some(dmg),
                    rof: Some(20),
                    range: Some(dec("8")),
                    speed: Some(100),
                    projectile: Some("Bullet".into()),
                    warhead: Some(wh.into()),
                    is_mag_beam: mag.then_some(true),
                    ..Default::default()
                },
            );
        }
        for (name, primary) in [("CLONE", Some("Ray3")), ("GRUNT", None)] {
            let mut t = r.sim.infantry["WALKER"].clone();
            t.strength = Some(100);
            t.primary = primary.map(Into::into);
            register(&mut r, "infantry", name);
            r.sim.infantry.insert(name.into(), t);
        }
        let mut tgt: UnitType = r.sim.vehicles["BUGGY"].clone();
        tgt.strength = Some(30000);
        tgt.primary = None;
        let mut gatt: UnitType = r.sim.vehicles["CRUSHER"].clone();
        gatt.turret = Some(true);
        gatt.strength = Some(30000);
        let mut magl: UnitType = r.sim.vehicles["BUGGY"].clone();
        magl.strength = Some(1000);
        magl.primary = Some("MagRay".into());
        for (n, t) in [("TGT", tgt), ("GATT", gatt), ("MAGL", magl)] {
            register(&mut r, "vehicles", n);
            r.sim.vehicles.insert(n.into(), t);
        }
        let bio = BuildingType {
            strength: Some(1000),
            foundation: Some([2, 2]),
            infantry_absorb: Some(true),
            passengers: Some(3),
            size_limit: Some(dec("1")),
            ..Default::default()
        };
        let bunk = BuildingType {
            strength: Some(1000),
            foundation: Some([2, 2]),
            bunker: Some(true),
            number_impassable_rows: Some(0),
            ..Default::default()
        };
        for (n, b) in [("BIO", bio), ("BUNK", bunk)] {
            register(&mut r, "buildings", n);
            r.sim.buildings.insert(n.into(), b);
        }
        register(&mut r, "animations", "MUTA");
        r.sim.animations.insert(
            "MUTA".into(),
            AnimationType {
                make_infantry: Some(0),
                frame_count: Some(4),
                ..Default::default()
            },
        );
        let g = &mut r.sim.general.superweapons;
        g.mutate_explosion = Some(false);
        g.mutate_warhead = Some("MUT".into());
        g.infantry_mutate = Some("MUTA".into());
        g.anim_to_infantry = Some(vec!["GRUNT".into()]);
        r
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

    fn run_until(w: &mut World, max: i32, done: impl Fn(&World) -> bool) -> bool {
        for _ in 0..max {
            if done(w) {
                return true;
            }
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

    /// House `h`'s cell-mode launch at `c` (the superweapon's record and
    /// readiness are S-0512's and play no part here).
    fn launch(w: &mut World, h: u8, c: CellXY) {
        w.effects.clear();
        mutation::launch(w, HouseId(h), 0, c);
    }

    /// Runs until every conversion animation has ended; returns the units
    /// they made.
    fn made(w: &mut World) -> Vec<ObjectId> {
        let mut out = Vec::new();
        for _ in 0..60 {
            if w.mutation.makers.is_empty() {
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
        assert!(w.mutation.makers.is_empty(), "every animation ended");
        out
    }

    fn round_trip(w: &World) {
        let h = save::state_hash(w);
        let l = save::load(&save::save(w), w.rules_arc(), testkit::catalogue()).unwrap();
        assert_eq!(save::state_hash(&l), h);
    }

    // Pair: the genetic mutator and mind control (S-0611 × S-0601).
    // spec: S-0611 claim 7 (a controlled victim's control ends with it; a victim that held control of other units loses it as on any death)
    // spec: S-0611 claims 4, 9 and 10 (any house's listed infantry is hit; the made unit goes to the launching house and inherits nothing)
    // spec: S-0601 claims 10 and 12 (a controller's death releases its victims to their owners; a victim leaving play leaves its controller's list)
    #[test]
    fn a_mutated_controlled_unit_and_a_mutated_controller() {
        let mut w = testkit::game_with(&rules(), SIZE, 1);
        let clone = put(&mut w, Kind::Infantry, "CLONE", ALPHA, -6, 0);
        let grunt = put(&mut w, Kind::Infantry, "GRUNT", BETA, 0, 0);
        let tgt = put(&mut w, Kind::Vehicle, "TGT", BETA, -6, 4);
        // Both are taken at once, before the controller's own fire can.
        assert!(mind::take(&mut w, clone, grunt));
        assert!(mind::take(&mut w, clone, tgt));
        testkit::run(&mut w, 5);
        assert_eq!(owner(&w, tgt), Some(ALPHA));
        assert_eq!(owner(&w, grunt), Some(ALPHA));
        // Beta mutates its own unit, now Alpha's by control.
        launch(&mut w, BETA, at(0, 0));
        assert!(w.obj(grunt).is_none(), "converted");
        assert!(
            mind::controller(&w, clone).is_some_and(|r| !r.victims.contains(&grunt)),
            "the victim left its controller's list"
        );
        round_trip(&w);
        let units = made(&mut w);
        assert_eq!(units.len(), 1);
        assert_eq!(owner(&w, units[0]), Some(BETA), "the launching house");
        assert!(!mind::mind_controlled(&w, units[0]), "no control inherited");
        // Beta mutates the controller: its other victim goes back to Beta.
        launch(&mut w, BETA, at(-6, 0));
        assert!(w.obj(clone).is_none());
        tick::step(&mut w);
        assert_eq!(
            owner(&w, tgt),
            Some(BETA),
            "released by the controller's death"
        );
        assert!(!mind::mind_controlled(&w, tgt));
        let units = made(&mut w);
        assert_eq!(units.len(), 1);
        assert_eq!(owner(&w, units[0]), Some(BETA));
        assert!(mind::controller(&w, units[0]).is_none_or(|r| r.victims.is_empty()));
    }

    // Pair: the genetic mutator and a tank bunker (S-0611 × S-0606).
    // spec: S-0611 claim 4 (no object other than listed infantry receives a direct cell-mode hit: the bunker and its bunkered vehicle are untouched, infantry beside them converts)
    // spec: S-0606 claims 7 and 14 (the bunker link holds while its vehicle is neither destroyed nor removed)
    #[test]
    fn a_launch_on_a_tank_bunker_leaves_it_and_its_vehicle() {
        let mut w = testkit::game_with(&rules(), SIZE, 1);
        let b = put(&mut w, Kind::Building, "BUNK", ALPHA, 0, 0);
        let g = put(&mut w, Kind::Vehicle, "GATT", ALPHA, 6, 1);
        testkit::run(&mut w, 5);
        click(&mut w, ALPHA, &[g], b);
        assert!(
            run_until(&mut w, 1500, |w| bunker::bunkered_in(w, g) == Some(b)),
            "bunkered"
        );
        let beside = put(&mut w, Kind::Infantry, "GRUNT", ALPHA, -1, 0);
        let (hb, hg) = (w.obj(b).unwrap().health, w.obj(g).unwrap().health);
        launch(&mut w, BETA, at(0, 0));
        assert_eq!(w.obj(b).unwrap().health, hb);
        assert_eq!(w.obj(g).unwrap().health, hg);
        assert_eq!(bunker::bunkered_in(&w, g), Some(b), "still bunkered");
        assert!(w.obj(beside).is_none(), "the infantry beside converts");
        round_trip(&w);
        let units = made(&mut w);
        assert_eq!(units.len(), 1);
        assert_eq!(owner(&w, units[0]), Some(BETA));
    }

    // Pair: the genetic mutator and an absorbing building (S-0611 × S-0605).
    // spec: S-0611 claim 5 (infantry held by an absorbing building is not listed in any cell and is never a victim; a unit still on the map on its way in is eligible, and its conversion ends its entry)
    #[test]
    fn a_launch_on_an_absorber_spares_its_passenger_and_takes_an_entrant() {
        let mut w = testkit::game_with(&rules(), SIZE, 1);
        let bio = put(&mut w, Kind::Building, "BIO", ALPHA, 0, 0);
        let inside = put(&mut w, Kind::Infantry, "GRUNT", ALPHA, 4, 0);
        testkit::run(&mut w, 5);
        click(&mut w, ALPHA, &[inside], bio);
        assert!(
            run_until(&mut w, 900, |w| absorb::absorbed_in(w, inside) == Some(bio)),
            "goes in"
        );
        // A second walker on its way in, one cell from the building.
        let entrant = put(&mut w, Kind::Infantry, "GRUNT", ALPHA, -1, 1);
        click(&mut w, ALPHA, &[entrant], bio);
        tick::step(&mut w);
        let h = w.obj(inside).unwrap().health;
        launch(&mut w, BETA, at(0, 0));
        assert_eq!(absorb::absorbed_in(&w, inside), Some(bio), "still inside");
        assert_eq!(w.obj(inside).unwrap().health, h, "not hit");
        assert!(w.obj(entrant).is_none(), "the entrant converts");
        round_trip(&w);
        let units = made(&mut w);
        assert_eq!(units.len(), 1);
        assert_eq!(owner(&w, units[0]), Some(BETA));
        assert_eq!(absorb::count(&w, bio), 1, "the entry ended with it");
    }

    // Pair: the genetic mutator and the magnetic lift (S-0611 × S-0607).
    // spec: S-0611 claim 4 (a seized vehicle is no infantry: it takes no direct cell-mode hit; infantry beneath it converts)
    // spec: S-0607 claims 5 and 12 (the seized vehicle stays gripped through hits that do not reach it)
    #[test]
    fn a_launch_under_a_seized_vehicle_takes_only_the_infantry() {
        let mut w = testkit::game_with(&rules(), SIZE, 1);
        let v = put(&mut w, Kind::Vehicle, "TGT", ALPHA, 6, 0);
        let l = put(&mut w, Kind::Vehicle, "MAGL", BETA, -2, 0);
        testkit::run(&mut w, 5);
        orders::issue(
            &mut w,
            HouseId(BETA),
            Order::Behaviour {
                actor: l,
                behaviour: Behaviour::Attack,
                attack_move: false,
                target: Some(Target::Object(v)),
                dest: None,
            },
        );
        assert!(run_until(&mut w, 600, |w| lift::seized(w, v)), "seized");
        let c = w.obj(v).unwrap().pos.cell();
        let under = testkit::spawn(&mut w, Kind::Infantry, "GRUNT", ALPHA, c, 0);
        let hv = w.obj(v).unwrap().health;
        launch(&mut w, BETA, c);
        assert_eq!(w.obj(v).unwrap().health, hv, "no hit on the vehicle");
        assert!(lift::seized(&w, v), "still seized");
        assert!(w.obj(under).is_none(), "the infantry beneath converts");
        round_trip(&w);
        let units = made(&mut w);
        assert_eq!(units.len(), 1);
        assert_eq!(owner(&w, units[0]), Some(BETA));
    }
}

mod dominator_pairs {
    use super::register;
    use glasnost_map::CellXY;
    use glasnost_rules::schema::Ruleset;
    use glasnost_rules::sim::AnimationType;
    use glasnost_sim::combat::effects::Effect;
    use glasnost_sim::domination::{self, DomEvent};
    use glasnost_sim::rules::Kind;
    use glasnost_sim::structures::{absorb, bunker};
    use glasnost_sim::targeting::{self, ClickOn, Modifiers};
    use glasnost_sim::world::{HouseId, ObjectId, World};
    use glasnost_sim::{mind, orders, save, testkit, tick};

    const SIZE: i32 = 24;
    const ALPHA: u8 = 0;
    const BETA: u8 = 1;

    /// The mutator pairs' ruleset (the controller CLONE, GRUNT, TGT,
    /// GATT, the absorber BIO and the bunker BUNK) with the domination
    /// values: lead-in LEAD (four frames) swept at stage 0, sweep SWEEP
    /// (four frames), capture range 1, no warhead (no damage).
    pub(super) fn rules() -> Ruleset {
        let mut r = super::mutator_pairs::rules();
        for n in ["LEAD", "SWEEP"] {
            register(&mut r, "animations", n);
            r.sim.animations.insert(
                n.into(),
                AnimationType {
                    frame_count: Some(4),
                    ..Default::default()
                },
            );
        }
        let g = &mut r.sim.general.superweapons;
        g.dominator_first_anim = Some("LEAD".into());
        g.dominator_second_anim = Some("SWEEP".into());
        g.dominator_fire_at_percentage = Some(0);
        g.dominator_capture_range = Some(1);
        r
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

    fn run_until(w: &mut World, max: i32, done: impl Fn(&World) -> bool) -> bool {
        for _ in 0..max {
            if done(w) {
                return true;
            }
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

    /// House `h`'s domination at `c` (the superweapon's record and
    /// readiness are S-0512's and play no part here), run to its sweep;
    /// returns the objects it took over.
    pub(super) fn dominate(w: &mut World, h: u8, c: CellXY) -> Vec<ObjectId> {
        domination::launch(w, HouseId(h), 0, c);
        for _ in 0..10 {
            w.effects.clear();
            tick::step(w);
            let ev: Vec<DomEvent> = w
                .effects
                .iter()
                .filter_map(|e| match e {
                    Effect::Domination { event, .. } => Some(*event),
                    _ => None,
                })
                .collect();
            if ev.contains(&DomEvent::Sweep) {
                return ev
                    .into_iter()
                    .filter_map(|e| match e {
                        DomEvent::Taken(id) => Some(id),
                        _ => None,
                    })
                    .collect();
            }
        }
        panic!("no sweep");
    }

    fn round_trip(w: &World) {
        let h = save::state_hash(w);
        let l = save::load(&save::save(w), w.rules_arc(), testkit::catalogue()).unwrap();
        assert_eq!(save::state_hash(&l), h);
    }

    // Pair: the psychic dominator and mind control (S-0612 × S-0601).
    // spec: S-0612 claim 14 (a controlled object is first released by its temporary controller, then changes owner, then becomes permanently controlled)
    // spec: S-0612 claim 15 (a temporary controller that is taken over keeps its victims, S-0601 claim 11)
    // spec: S-0601 claims 9 and 11 (the release ends the link; an owner change of the controller keeps its victims under quirks.mind_control_owner_change_keeps_victims)
    #[test]
    fn a_domination_takes_a_controller_and_a_controlled_unit() {
        let mut w = testkit::game_with(&rules(), SIZE, 1);
        // Beta's controller inside the swept area holds Alpha's unit far
        // outside it; Alpha's controller far outside holds Beta's unit
        // inside it.
        let c_in = put(&mut w, Kind::Infantry, "CLONE", BETA, 0, 0);
        let v_out = put(&mut w, Kind::Vehicle, "TGT", ALPHA, 10, 8);
        let c_out = put(&mut w, Kind::Infantry, "CLONE", ALPHA, -10, -8);
        let v_in = put(&mut w, Kind::Vehicle, "TGT", BETA, 1, 0);
        assert!(mind::take(&mut w, c_in, v_out));
        assert!(mind::take(&mut w, c_out, v_in));
        let taken = dominate(&mut w, ALPHA, at(0, 0));
        assert!(taken.contains(&c_in) && taken.contains(&v_in), "{taken:?}");
        // The victim inside: released by Alpha's controller, then taken
        // over permanently.
        assert!(mind::controlled_by(&w, v_in).is_none(), "released");
        assert!(mind::controller(&w, c_out).is_none_or(|r| !r.victims.contains(&v_in)));
        assert!(mind::permanent(&w, v_in));
        assert_eq!(owner(&w, v_in), Some(ALPHA));
        // The controller inside: now Alpha's, permanently, and it keeps its
        // victim outside the area.
        assert_eq!(owner(&w, c_in), Some(ALPHA));
        assert!(mind::permanent(&w, c_in));
        assert_eq!(
            mind::controlled_by(&w, v_out),
            Some(c_in),
            "keeps its victim"
        );
        assert!(!mind::permanent(&w, v_out), "not swept");
        round_trip(&w);
    }

    // Pair: the psychic dominator and a tank bunker (S-0612 × S-0606).
    // spec: S-0612 claim 13 (a vehicle in a two by two bunker stands at the bunker's foundation centre, listed before it: the start object of its cell, taken over; the bunker never is; the bunker link stays)
    // spec: S-0606 claim 15 (either side changes owner: the link holds)
    #[test]
    fn a_domination_takes_a_bunkered_vehicle_and_leaves_the_link() {
        let mut w = testkit::game_with(&rules(), SIZE, 1);
        let b = put(&mut w, Kind::Building, "BUNK", BETA, 0, 0);
        let g = put(&mut w, Kind::Vehicle, "GATT", BETA, 6, 1);
        testkit::run(&mut w, 5);
        click(&mut w, BETA, &[g], b);
        assert!(
            run_until(&mut w, 1500, |w| bunker::bunkered_in(w, g) == Some(b)),
            "bunkered"
        );
        let taken = dominate(&mut w, ALPHA, at(0, 0));
        assert_eq!(taken, vec![g]);
        assert_eq!(owner(&w, g), Some(ALPHA));
        assert!(mind::permanent(&w, g));
        assert_eq!(owner(&w, b), Some(BETA), "a building is never taken");
        assert_eq!(bunker::bunkered_in(&w, g), Some(b), "the link stays");
        round_trip(&w);
        testkit::run(&mut w, 5);
        assert_eq!(bunker::bunkered_in(&w, g), Some(b), "and holds");
    }

    // Pair: the psychic dominator and an absorbing building (S-0612 × S-0605).
    // spec: S-0612 claim 15 (only examined objects are direct recipients: an absorbed passenger is not listed in any cell and keeps its owner; the building is never taken, claim 13)
    // spec: S-0612 claims 12 and 13 (infantry on the map near the building is examined and taken over)
    // spec: S-0605 claim 1 (absorbed infantry leaves the map)
    #[test]
    fn a_domination_spares_an_absorbed_passenger() {
        let mut w = testkit::game_with(&rules(), SIZE, 1);
        let bio = put(&mut w, Kind::Building, "BIO", BETA, 0, 0);
        let inside = put(&mut w, Kind::Infantry, "GRUNT", BETA, 4, 0);
        testkit::run(&mut w, 5);
        click(&mut w, BETA, &[inside], bio);
        assert!(
            run_until(&mut w, 900, |w| absorb::absorbed_in(w, inside) == Some(bio)),
            "goes in"
        );
        let beside = put(&mut w, Kind::Infantry, "GRUNT", BETA, -1, 0);
        let taken = dominate(&mut w, ALPHA, at(0, 0));
        assert_eq!(taken, vec![beside]);
        assert_eq!(owner(&w, beside), Some(ALPHA));
        assert_eq!(
            owner(&w, inside),
            Some(BETA),
            "the passenger keeps its owner"
        );
        assert!(!mind::permanent(&w, inside));
        assert_eq!(absorb::absorbed_in(&w, inside), Some(bio), "still inside");
        assert_eq!(owner(&w, bio), Some(BETA));
        round_trip(&w);
    }
}

mod shield_pairs {
    use super::{add_shield, register, shield};
    use glasnost_map::CellXY;
    use glasnost_rules::schema::Ruleset;
    use glasnost_rules::sim::{BuildingType, Decimal, UnitType, WarheadType, WeaponType};
    use glasnost_sim::combat::check::{self, Answer};
    use glasnost_sim::econ::power;
    use glasnost_sim::econ::release::{self, Outcome};
    use glasnost_sim::orders::{self, Order};
    use glasnost_sim::rules::{Behaviour, Kind};
    use glasnost_sim::superweapons::protected;
    use glasnost_sim::world::{HouseId, ObjectId, Target, World};
    use glasnost_sim::{gattling, lift, mind, mutation, objects, save, testkit, tick};

    pub(super) const SIZE: i32 = 24;
    pub(super) const ALPHA: u8 = 0;
    pub(super) const BETA: u8 = 1;

    fn dec(s: &str) -> Decimal {
        Decimal(s.into())
    }

    /// The dominator pairs' rules (the controller CLONE with Ray3, GRUNT,
    /// TGT, the lifter MAGL, the absorber BIO, the mutation and domination
    /// values) with the force shield of `add_shield`, and: GATL (a vehicle
    /// gattling with six slots G0 to G5, each of damage 5, rate 1, range 10
    /// and warhead GWH of versus 1 for every armour; three stages, spin-up 7, spin-down 1, thresholds 10,
    /// 30 and 60); TOWER (an unarmed 1 × 1 building of strength 1000); VAT
    /// (a 2 × 2 cloning building, not immune to psionics); BARR (a 2 × 2 infantry factory); and the
    /// domination's area damage 20 with the warhead DOMWH (spread 4 cells,
    /// full damage at the edge).
    pub(super) fn rules() -> Ruleset {
        let mut r = super::dominator_pairs::rules();
        add_shield(&mut r);
        register(&mut r, "warheads", "GWH");
        r.sim.warheads.insert(
            "GWH".into(),
            WarheadType {
                verses: Some(vec![dec("1"); 11]),
                ..Default::default()
            },
        );
        let slots: Vec<String> = (0..6).map(|i| format!("G{i}")).collect();
        for n in &slots {
            r.sim.weapons.insert(
                n.clone(),
                WeaponType {
                    damage: Some(5),
                    rof: Some(1),
                    range: Some(dec("10")),
                    speed: Some(100),
                    projectile: Some("Bullet".into()),
                    warhead: Some("GWH".into()),
                    ..Default::default()
                },
            );
        }
        let mut gat: UnitType = r.sim.vehicles["TGT"].clone();
        gat.turret_count = Some(1);
        gat.weapon_count = Some(6);
        gat.combat.weapon_slots = Some(slots);
        gat.combat.is_gattling = Some(true);
        gat.combat.weapon_stages = Some(3);
        gat.combat.rate_up = Some(7);
        gat.combat.rate_down = Some(1);
        gat.combat.stage_thresholds = Some(vec![10, 30, 60]);
        register(&mut r, "vehicles", "GATL");
        r.sim.vehicles.insert("GATL".into(), gat);
        let tower = BuildingType {
            strength: Some(1000),
            foundation: Some([1, 1]),
            ..Default::default()
        };
        let mut vat = BuildingType {
            strength: Some(1000),
            foundation: Some([2, 2]),
            ..Default::default()
        };
        vat.econ.cloning = Some(true);
        // Not immune to psionics, so that it can be taken (S-0601 claim 3).
        vat.combat.immune_to_psionics = Some(false);
        let barr = BuildingType {
            strength: Some(1000),
            foundation: Some([2, 2]),
            factory: Some("infantry".into()),
            ..Default::default()
        };
        for (n, b) in [("TOWER", tower), ("VAT", vat), ("BARR", barr)] {
            register(&mut r, "buildings", n);
            r.sim.buildings.insert(n.into(), b);
        }
        register(&mut r, "warheads", "DOMWH");
        r.sim.warheads.insert(
            "DOMWH".into(),
            WarheadType {
                verses: Some(vec![dec("1"); 11]),
                cell_spread: Some(dec("4")),
                percent_at_max: Some(dec("1")),
                ..Default::default()
            },
        );
        let g = &mut r.sim.general.superweapons;
        g.dominator_damage = Some(20);
        g.dominator_warhead = Some("DOMWH".into());
        r
    }

    pub(super) fn at(dx: i32, dy: i32) -> CellXY {
        testkit::mid(SIZE, dx, dy)
    }

    pub(super) fn put(
        w: &mut World,
        kind: Kind,
        name: &str,
        house: u8,
        dx: i32,
        dy: i32,
    ) -> ObjectId {
        testkit::spawn(w, kind, name, house, at(dx, dy), 0)
    }

    pub(super) fn owner(w: &World, id: ObjectId) -> Option<u8> {
        w.obj(id).and_then(|o| o.owner).map(|h| h.0)
    }

    fn health(w: &World, id: ObjectId) -> i32 {
        w.obj(id).unwrap().health
    }

    pub(super) fn run_until(w: &mut World, max: i32, done: impl Fn(&World) -> bool) -> bool {
        for _ in 0..max {
            if done(w) {
                return true;
            }
            tick::step(w);
        }
        done(w)
    }

    pub(super) fn attack(w: &mut World, house: u8, actor: ObjectId, target: ObjectId) {
        assert!(orders::issue(
            w,
            HouseId(house),
            Order::Behaviour {
                actor,
                behaviour: Behaviour::Attack,
                attack_move: false,
                target: Some(Target::Object(target)),
                dest: None,
            },
        ));
    }

    /// House `h`'s infantry factory `k` releases a new GRUNT (S-0404
    /// claim 13), once every earlier released unit has left its building's
    /// contact place; returns the GRUNTs that appeared other than it (the
    /// copies), in creation order. The new units are then ordered away, so
    /// that their contact ends (S-0417 claim 10).
    pub(super) fn release_grunt(w: &mut World, h: u8, k: ObjectId) -> Vec<ObjectId> {
        assert!(
            run_until(w, 600, |w| w.objects.values().all(|o| o.contact.is_none())),
            "the earlier units leave"
        );
        let ti = w.rules().index_of(Kind::Infantry, "GRUNT").unwrap();
        let before: Vec<ObjectId> = w.objects.keys().copied().collect();
        let g = objects::create(w, Kind::Infantry, ti, Some(HouseId(h)));
        assert_eq!(release::release(w, k, g), Outcome::Released);
        let new: Vec<(ObjectId, Option<HouseId>)> = w
            .objects
            .values()
            .filter(|o| o.kind == Kind::Infantry && o.type_index == ti && o.on_map)
            .filter(|o| !before.contains(&o.id))
            .map(|o| (o.id, o.owner))
            .collect();
        for (i, &(id, house)) in new.iter().enumerate() {
            let dest = at(i as i32 * 2 - 6, 9);
            assert!(orders::issue(
                w,
                house.unwrap(),
                Order::Behaviour {
                    actor: id,
                    behaviour: Behaviour::Move,
                    attack_move: false,
                    target: None,
                    dest: Some(Target::Cell(dest)),
                },
            ));
        }
        new.into_iter()
            .map(|(id, _)| id)
            .filter(|&id| id != g)
            .collect()
    }

    pub(super) fn round_trip(w: &World) {
        let h = save::state_hash(w);
        let l = save::load(&save::save(w), w.rules_arc(), testkit::catalogue()).unwrap();
        assert_eq!(save::state_hash(&l), h);
    }

    // Pair: the force shield and mind control (S-0513 × S-0601), with the cloning vat (S-0404).
    // spec: S-0513 claims 27 and 31 (a building of the launching house within the radius is protected; a protected object cannot be mind-controlled)
    // spec: S-0601 claim 3 (the takeover test: a building may be taken, a protected object may not), claim 4 (the firing check of a takeover warhead answers illegal when the test fails) and claim 5 (the landing tests again: a target protected by then is not taken)
    // spec: S-0513 claim 30 (the launching house is blacked out) and S-0404 claim 14 (each cloning building the house owns copies the released infantry; no power gate) with S-0405 claim 9 (factories do not stop for low power)
    #[test]
    fn a_shielded_vat_refuses_mind_control_and_keeps_cloning() {
        let mut w = testkit::game_with(&rules(), SIZE, 1);
        let vat = put(&mut w, Kind::Building, "VAT", ALPHA, 0, 0);
        let open = put(&mut w, Kind::Building, "VAT", ALPHA, -8, -8);
        let barr = put(&mut w, Kind::Building, "BARR", ALPHA, 7, -8);
        let clone = put(&mut w, Kind::Infantry, "CLONE", BETA, 4, 4);
        testkit::run(&mut w, 5);
        shield(&mut w, ALPHA, at(0, 0));
        assert!(protected(&w, vat));
        assert!(
            !protected(&w, open) && !protected(&w, barr),
            "outside the radius"
        );
        assert!(power::blackout_running(&w, HouseId(ALPHA)));
        assert!(!mind::can_take(&w, clone, vat));
        assert_eq!(
            check::check(&w, clone, Some(Target::Object(vat)), 0, false),
            Answer::Illegal
        );
        // A shot already on its way lands on the shielded vat: nothing.
        mind::detonation(&mut w, Some(clone), Some(Target::Object(vat)));
        assert_eq!(owner(&w, vat), Some(ALPHA));
        // The same landing on the unshielded vat takes it.
        mind::detonation(&mut w, Some(clone), Some(Target::Object(open)));
        assert_eq!(owner(&w, open), Some(BETA));
        round_trip(&w);
        // Alpha's release: one copy, at the shielded vat, Alpha's, though
        // Alpha is blacked out; the taken vat copies nothing for Alpha.
        let copies = release_grunt(&mut w, ALPHA, barr);
        assert_eq!(copies.len(), 1);
        assert_eq!(w.obj(copies[0]).unwrap().contact, Some(vat));
        assert_eq!(owner(&w, copies[0]), Some(ALPHA));
        assert!(power::blackout_running(&w, HouseId(ALPHA)));
        round_trip(&w);
    }

    // Pair: the force shield and a gattling (S-0513 × S-0609).
    // spec: S-0513 claim 28 (departure: a player house's units may fire at a protected object; their hits do nothing) and S-0302 claim 8 (a hit on a protected object changes no health)
    // spec: S-0609 claim 5 (a vehicle gattling spins up in every tick in which it fires), so it spins up against a shielded building as against an open one
    #[test]
    fn a_gattling_fires_at_a_shielded_building_and_spins_up_for_nothing() {
        let mut w = testkit::game_with(&rules(), SIZE, 1);
        let shielded = put(&mut w, Kind::Building, "TOWER", ALPHA, 0, 0);
        let open = put(&mut w, Kind::Building, "TOWER", ALPHA, 0, 9);
        let g1 = put(&mut w, Kind::Vehicle, "GATL", BETA, 5, 0);
        let g2 = put(&mut w, Kind::Vehicle, "GATL", BETA, 5, 9);
        testkit::run(&mut w, 5);
        shield(&mut w, ALPHA, at(0, 0));
        assert!(protected(&w, shielded) && !protected(&w, open));
        assert_ne!(
            check::check(&w, g1, Some(Target::Object(shielded)), 0, false),
            Answer::Illegal,
            "a player house may fire"
        );
        let (h1, h2) = (health(&w, shielded), health(&w, open));
        attack(&mut w, BETA, g1, shielded);
        attack(&mut w, BETA, g2, open);
        assert!(
            run_until(&mut w, 60, |w| gattling::spin(w, g1).value >= 30
                && gattling::spin(w, g2).value >= 30
                && w.obj(open).unwrap().health < h2),
            "both spin up and the control is hit"
        );
        assert!(protected(&w, shielded), "still within the shield's time");
        assert_eq!(health(&w, shielded), h1, "the hits do nothing");
        assert!(health(&w, open) < h2, "the control is hurt");
        round_trip(&w);
    }

    // Pair: the force shield and the psychic dominator (S-0513 × S-0612).
    // spec: S-0513 claim 31 (only buildings are shielded: infantry, vehicles and aircraft within the radius are not protected) and S-0302 claim 8 (the sweep's area damage does nothing to a shielded building)
    // spec: S-0612 claims 9, 10 and 13 (the area damage before the takeovers; a building is never taken, a protected object is passed over, an unprotected unit is taken)
    #[test]
    fn a_domination_over_a_shielded_base_takes_its_units_and_spares_its_buildings() {
        let mut w = testkit::game_with(&rules(), SIZE, 1);
        let shielded = put(&mut w, Kind::Building, "TOWER", ALPHA, 0, 0);
        let open = put(&mut w, Kind::Building, "TOWER", ALPHA, 3, 0);
        let grunt = put(&mut w, Kind::Infantry, "GRUNT", ALPHA, 1, 1);
        testkit::run(&mut w, 5);
        shield(&mut w, ALPHA, at(0, 0));
        assert!(protected(&w, shielded));
        assert!(!protected(&w, open), "3 cells away: not below the radius");
        assert!(!protected(&w, grunt), "infantry is never shielded");
        let (hs, ho) = (health(&w, shielded), health(&w, open));
        let taken = super::dominator_pairs::dominate(&mut w, BETA, at(0, 0));
        assert!(taken.contains(&grunt), "the unit is taken");
        assert_eq!(owner(&w, grunt), Some(BETA));
        assert_eq!(owner(&w, shielded), Some(ALPHA));
        assert_eq!(owner(&w, open), Some(ALPHA));
        assert_eq!(health(&w, shielded), hs, "no damage under the shield");
        assert!(health(&w, open) < ho, "the unshielded building is hurt");
        round_trip(&w);
    }

    // Pair: the force shield and the genetic mutator (S-0513 × S-0611).
    // spec: S-0513 claim 31 (infantry within the radius is not protected)
    // spec: S-0611 claim 4 (cell mode hits the infantry listed in the swept cells and no building) and claims 9 and 10 (the made unit is the launching house's)
    #[test]
    fn a_mutation_inside_a_shield_converts_its_infantry() {
        let mut w = testkit::game_with(&rules(), SIZE, 1);
        let shielded = put(&mut w, Kind::Building, "TOWER", ALPHA, 0, 0);
        let grunt = put(&mut w, Kind::Infantry, "GRUNT", ALPHA, 1, 0);
        testkit::run(&mut w, 5);
        shield(&mut w, ALPHA, at(0, 0));
        assert!(protected(&w, shielded) && !protected(&w, grunt));
        let h = health(&w, shielded);
        w.effects.clear();
        mutation::launch(&mut w, HouseId(BETA), 0, at(1, 0));
        assert!(w.obj(grunt).is_none(), "converted");
        assert_eq!(health(&w, shielded), h);
        round_trip(&w);
        assert!(
            run_until(&mut w, 60, |w| w.mutation.makers.is_empty()),
            "the animation ends"
        );
        let ti = w.rules().index_of(Kind::Infantry, "GRUNT").unwrap();
        let made: Vec<u8> = w
            .objects
            .values()
            .filter(|o| o.kind == Kind::Infantry && o.type_index == ti)
            .filter_map(|o| o.owner.map(|h| h.0))
            .collect();
        assert_eq!(made, [BETA], "one made unit, the launching house's");
    }

    // Pair: the force shield and the magnetic lift (S-0513 × S-0607).
    // spec: S-0513 claim 31 (a vehicle within the radius is not protected) and claim 27 (only a protected object's locomotor is refused to a seizing warhead)
    // spec: S-0607 claims 3 and 5 (an unprotected vehicle is seized)
    #[test]
    fn a_lift_seizes_a_vehicle_inside_a_shield() {
        let mut w = testkit::game_with(&rules(), SIZE, 1);
        let shielded = put(&mut w, Kind::Building, "TOWER", ALPHA, 0, 0);
        let v = put(&mut w, Kind::Vehicle, "TGT", ALPHA, 1, 1);
        let l = put(&mut w, Kind::Vehicle, "MAGL", BETA, -6, 1);
        testkit::run(&mut w, 5);
        shield(&mut w, ALPHA, at(0, 0));
        assert!(protected(&w, shielded) && !protected(&w, v));
        attack(&mut w, BETA, l, v);
        assert!(run_until(&mut w, 60, |w| lift::seized(w, v)), "seized");
        assert!(protected(&w, shielded));
        round_trip(&w);
    }
}

mod cloning_pairs {
    use super::shield_pairs::{
        ALPHA, BETA, SIZE, at, owner, put, release_grunt, round_trip, rules, run_until,
    };
    use glasnost_sim::orders::{self, Order};
    use glasnost_sim::rules::Kind;
    use glasnost_sim::structures::absorb;
    use glasnost_sim::targeting::{self, ClickOn, Modifiers};
    use glasnost_sim::world::{HouseId, World};
    use glasnost_sim::{mind, mutation, testkit};

    fn grunts(w: &World) -> usize {
        let ti = w.rules().index_of(Kind::Infantry, "GRUNT").unwrap();
        w.objects
            .values()
            .filter(|o| o.kind == Kind::Infantry && o.type_index == ti)
            .count()
    }

    // Pair: cloning vats and mind control (S-0404 × S-0601).
    // spec: S-0601 claims 3 and 6 (a building may be taken; the takeover changes its owner with every consumer of a change of owner) and claim 9 (the release gives it back to its owner)
    // spec: S-0404 claim 14 (each cloning building the house owns, in acquisition order, copies the released infantry for its owner) and claim 7 Terms (an ownership change ranks a building after every building the new owner holds; reacquiring gives a later rank)
    #[test]
    fn a_controlled_vat_clones_for_its_controller_and_ranks_last_when_released() {
        let mut w = testkit::game_with(&rules(), SIZE, 1);
        let v1 = put(&mut w, Kind::Building, "VAT", ALPHA, -7, 4);
        let v2 = put(&mut w, Kind::Building, "VAT", ALPHA, 6, 4);
        let ka = put(&mut w, Kind::Building, "BARR", ALPHA, -7, -7);
        let kb = put(&mut w, Kind::Building, "BARR", BETA, 6, -7);
        let clone = put(&mut w, Kind::Infantry, "CLONE", BETA, -4, 0);
        testkit::run(&mut w, 5);
        let contacts = |w: &World, c: &[_]| -> Vec<_> {
            c.iter().map(|&g| w.obj(g).unwrap().contact).collect()
        };
        let copies = release_grunt(&mut w, ALPHA, ka);
        assert_eq!(
            contacts(&w, &copies),
            [Some(v1), Some(v2)],
            "the first order"
        );
        assert!(mind::take(&mut w, clone, v1));
        assert_eq!(owner(&w, v1), Some(BETA));
        // Beta's release: the controlled vat copies for Beta.
        let copies = release_grunt(&mut w, BETA, kb);
        assert_eq!(contacts(&w, &copies), [Some(v1)]);
        assert_eq!(owner(&w, copies[0]), Some(BETA));
        // Alpha's release: only Alpha's own vat.
        let copies = release_grunt(&mut w, ALPHA, ka);
        assert_eq!(contacts(&w, &copies), [Some(v2)]);
        round_trip(&w);
        // Released back to Alpha, the vat ranks after the other.
        mind::release(&mut w, clone, v1);
        assert_eq!(owner(&w, v1), Some(ALPHA));
        let copies = release_grunt(&mut w, ALPHA, ka);
        assert_eq!(
            contacts(&w, &copies),
            [Some(v2), Some(v1)],
            "the order flips"
        );
        assert!(copies.iter().all(|&c| owner(&w, c) == Some(ALPHA)));
        round_trip(&w);
    }

    // Pair: cloning vats and the genetic mutator (S-0404 × S-0611).
    // spec: S-0611 claims 9 and 10 (the made unit is a fresh object placed by the conversion animation, the launching house's)
    // spec: S-0404 claim 14 (copies come only from an infantry factory's release; a unit that no factory releases is not copied)
    #[test]
    fn a_mutation_made_unit_is_not_cloned() {
        let mut w = testkit::game_with(&rules(), SIZE, 1);
        let vat = put(&mut w, Kind::Building, "VAT", ALPHA, -7, 4);
        let k = put(&mut w, Kind::Building, "BARR", ALPHA, -7, -7);
        put(&mut w, Kind::Infantry, "GRUNT", BETA, 6, 6);
        testkit::run(&mut w, 5);
        mutation::launch(&mut w, HouseId(ALPHA), 0, at(6, 6));
        assert!(
            run_until(&mut w, 60, |w| w.mutation.makers.is_empty()),
            "the animation ends"
        );
        assert_eq!(grunts(&w), 1, "the made unit, no copy");
        round_trip(&w);
        // The contrast: a factory release is copied.
        let copies = release_grunt(&mut w, ALPHA, k);
        assert_eq!(copies.len(), 1);
        assert_eq!(w.obj(copies[0]).unwrap().contact, Some(vat));
    }

    // Pair: cloning vats and the infantry absorber (S-0404 × S-0605).
    // spec: S-0605 claims 11 and 12 (the owner's deploy order lets a passenger out by the exit search)
    // spec: S-0404 claim 14 (copies come only from an infantry factory's release; a passenger let out is not copied)
    #[test]
    fn a_passenger_let_out_of_an_absorber_is_not_cloned() {
        let mut w = testkit::game_with(&rules(), SIZE, 1);
        put(&mut w, Kind::Building, "VAT", ALPHA, -7, 4);
        let bio = put(&mut w, Kind::Building, "BIO", ALPHA, 0, 0);
        let g = put(&mut w, Kind::Infantry, "GRUNT", ALPHA, 4, 0);
        testkit::run(&mut w, 5);
        let r = targeting::click(
            &mut w,
            HouseId(ALPHA),
            &[g],
            ClickOn::Object(bio),
            Modifiers::default(),
        );
        for o in r.orders {
            orders::issue(&mut w, HouseId(ALPHA), o);
        }
        assert!(
            run_until(&mut w, 900, |w| absorb::absorbed_in(w, g) == Some(bio)),
            "goes in"
        );
        assert!(orders::issue(
            &mut w,
            HouseId(ALPHA),
            Order::Deploy { actor: bio }
        ));
        assert!(
            run_until(&mut w, 300, |w| absorb::count(w, bio) == 0
                && w.obj(g).is_some_and(|o| o.on_map)),
            "let out"
        );
        assert_eq!(grunts(&w), 1, "no copy");
        round_trip(&w);
    }

    // Pair: cloning vats and the psychic dominator (S-0404 × S-0612).
    // spec: S-0612 claim 13 (a building is never taken over) and S-0404 claim 14 (the vat still copies for its owner)
    #[test]
    fn a_domination_leaves_a_vat_with_its_owner() {
        let mut w = testkit::game_with(&rules(), SIZE, 1);
        let vat = put(&mut w, Kind::Building, "VAT", ALPHA, 0, 0);
        let k = put(&mut w, Kind::Building, "BARR", ALPHA, -7, -7);
        testkit::run(&mut w, 5);
        super::dominator_pairs::dominate(&mut w, BETA, at(0, 0));
        assert_eq!(owner(&w, vat), Some(ALPHA));
        let copies = release_grunt(&mut w, ALPHA, k);
        assert_eq!(copies.len(), 1);
        assert_eq!(w.obj(copies[0]).unwrap().contact, Some(vat));
        assert_eq!(owner(&w, copies[0]), Some(ALPHA));
        round_trip(&w);
    }
}
