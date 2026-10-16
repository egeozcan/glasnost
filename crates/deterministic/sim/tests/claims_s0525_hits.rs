//! Claim tests for who radiation hits, the hit itself, the order within a
//! tick and which detonations leave radiation (S-0525 claims 1, 8 to 12,
//! scenarios RF6 to RF10) on synthetic data.

#[path = "rad_common/mod.rs"]
mod rad_common;

use glasnost_rules::sim::{ProjectileType, UnitType, WeaponType};
use glasnost_sim::combat::damage::{self, Hit};
use glasnost_sim::econ::stats;
use glasnost_sim::radiation;
use glasnost_sim::rules::Kind;
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, Layer, ObjectId, World};
use rad_common::*;

/// A field at X, made by a shot at tick 20, and the world after the next
/// hit tick (30) plus the probes' health before it.
fn fielded(tweak: impl FnOnce(&mut glasnost_rules::schema::Ruleset)) -> (World, ObjectId) {
    let (mut w, s) = base(tweak);
    run_to(&mut w, 20);
    (w, s)
}

// spec: S-0525 claim 8 (RF6: a vehicle is hit on the hit ticks, a building never, an immune infantry unit gets no hit at all)
#[test]
fn vehicles_are_hit_buildings_and_immune_types_are_not() {
    let (mut w, s) = fielded(|_| {});
    let car = testkit::spawn(&mut w, Kind::Vehicle, "VPROBE", 1, at(1, 0), 0);
    let tower = testkit::spawn(&mut w, Kind::Building, "TOWER", 1, at(0, 1), 0);
    let immune = probe(&mut w, "IMMUNE", at(0, 0));
    let tower_health = health(&w, tower);
    detonate(&mut w, s, X);
    run_to(&mut w, 40);
    // 600 / 1000 of the level at one cell: 37.
    assert_eq!(health(&w, car), 100_000 - 37);
    assert_eq!(health(&w, tower), tower_health);
    assert_eq!(health(&w, immune), 100_000);
}

// spec: S-0525 claim 8 (RF6: an immune type gets no hit at all, so none of S-0302 claim 10's side effects happen: a hit that would set it berserk or change its state is never made)
#[test]
fn an_immune_type_takes_no_hit_even_from_a_radiation_site_warhead_without_the_flag() {
    let (mut w, s) = fielded(|r| {
        r.sim.warheads.get_mut("SITE").unwrap().radiation = Some(false);
    });
    let immune = probe(&mut w, "IMMUNE", at(0, 0));
    detonate(&mut w, s, X);
    run_to(&mut w, 100);
    assert_eq!(health(&w, immune), 100_000);
}

// spec: S-0525 claim 8 (RF6: objects off the map, such as passengers or occupants, are never hit)
#[test]
fn objects_off_the_map_are_not_hit() {
    let (mut w, s) = fielded(|_| {});
    let inside = probe(&mut w, "PROBE", at(0, 0));
    w.obj_mut(inside).unwrap().on_map = false;
    detonate(&mut w, s, X);
    run_to(&mut w, 100);
    assert_eq!(health(&w, inside), 100_000);
}

// spec: S-0525 claim 8 and Terms (RF6: a unit at least 208 leptons above the surface is airborne and not hit; just below it is hit)
#[test]
fn an_airborne_unit_is_not_hit() {
    for (lift, hit) in [(300, false), (207, true), (208, false)] {
        let (mut w, s) = fielded(|_| {});
        let flier = probe(&mut w, "PROBE", at(0, 0));
        w.obj_mut(flier).unwrap().pos.z += lift;
        detonate(&mut w, s, X);
        run_to(&mut w, 40);
        assert_eq!(health(&w, flier) < 100_000, hit, "lift {lift}");
    }
}

// spec: S-0525 claim 8 (RF6: a unit on a bridge deck reads the radiation of the cell below the deck and is not airborne there)
#[test]
fn a_unit_on_a_deck_reads_the_cell_below() {
    let (mut w, s) = fielded(|_| {});
    let on_deck = testkit::spawn(&mut w, Kind::Vehicle, "VPROBE", 1, at(0, 0), 0);
    {
        let o = w.obj_mut(on_deck).unwrap();
        o.layer = Layer::Deck;
        o.pos.z += 416; // spec: S-0307 Constants (bridge deck height 416 leptons)
    }
    detonate(&mut w, s, X);
    run_to(&mut w, 40);
    assert_eq!(health(&w, on_deck), 100_000 - 62);
}

// spec: S-0525 claim 8 (RF6: a landed aircraft is hit)
#[test]
fn a_landed_aircraft_is_hit() {
    let (mut w, s) = fielded(|r| {
        let plane = UnitType {
            strength: Some(100_000),
            speed: Some(30),
            rot: Some(12),
            locomotor: Some(glasnost_rules::sim::Locomotor::Fly),
            ..Default::default()
        };
        r.sim.aircraft.insert("PLANE".into(), plane);
        r.registries
            .entry("aircraft".into())
            .or_default()
            .push("PLANE".into());
    });
    let plane = testkit::spawn(&mut w, Kind::Aircraft, "PLANE", 1, at(0, 0), 0);
    detonate(&mut w, s, X);
    run_to(&mut w, 40);
    assert_eq!(health(&w, plane), 100_000 - 62);
}

// spec: S-0525 claim 9 (a unit that enters a field just after a damage tick takes nothing until the next one; between damage ticks no radiation damage happens)
#[test]
fn a_late_arrival_waits_for_the_next_damage_tick() {
    let (mut w, s) = fielded(|_| {});
    detonate(&mut w, s, X);
    run_to(&mut w, 31);
    let late = probe(&mut w, "PROBE", at(0, 0));
    let l = losses(&mut w, late, 20);
    assert_eq!(l, vec![(45, 62)]);
}

// spec: S-0525 claim 10 (RF7 part a: a kill by radiation credits no object and no house, whoever made the field: the firer gains no experience and its house no kill)
#[test]
fn a_radiation_kill_credits_nobody() {
    let (mut w, s) = fielded(|r| {
        vehicle(r, "FRAIL", |t| t.strength = Some(40));
    });
    let victim = testkit::spawn(&mut w, Kind::Vehicle, "FRAIL", 1, at(0, 0), 0);
    let xp = w.obj(s).unwrap().combat.experience;
    detonate(&mut w, s, X);
    run_to(&mut w, 40);
    assert!(
        w.obj(victim).is_none() || health(&w, victim) == 0,
        "the victim is dead"
    );
    assert_eq!(w.obj(s).unwrap().combat.experience, xp);
    assert_eq!(stats::of(&w, HouseId(0)).kills, 0);
    assert_eq!(stats::of(&w, HouseId(1)).losses, 1);
}

// spec: S-0525 claim 10 (RF7 part b: the hit carries the no-escape marker, so a crewed vehicle killed by radiation leaves no crew, where an ordinary kill does)
#[test]
fn a_radiation_kill_leaves_no_crew() {
    let tweak = |r: &mut glasnost_rules::schema::Ruleset| {
        vehicle(r, "FRAIL", |t| {
            t.strength = Some(40);
            t.crewed = Some(true);
        });
        infantry(r, "TECH", |_| {});
        r.sim.general.crew_escape = Some(d("1"));
        r.sim.general.technician = Some("TECH".into());
    };
    let crews = |w: &World| {
        w.objects
            .values()
            .filter(|o| {
                o.kind == Kind::Infantry
                    && w.rules().index_of(Kind::Infantry, "TECH") == Some(o.type_index)
            })
            .count()
    };
    let (mut w, s) = fielded(tweak);
    let victim = testkit::spawn(&mut w, Kind::Vehicle, "FRAIL", 1, at(0, 0), 0);
    detonate(&mut w, s, X);
    run_to(&mut w, 40);
    assert!(w.obj(victim).is_none() || health(&w, victim) == 0);
    assert_eq!(crews(&w), 0, "radiation kill");
    // The control: an ordinary kill of the same vehicle leaves a crew.
    let (mut w, _) = fielded(tweak);
    let victim = testkit::spawn(&mut w, Kind::Vehicle, "FRAIL", 1, at(0, 0), 0);
    damage::apply(
        &mut w,
        victim,
        Hit {
            amount: 100_000,
            warhead: None,
            distance: 0,
            attacker: None,
            house: None,
            ignore_defences: true,
            no_escape: false,
        },
    );
    assert_eq!(crews(&w), 1, "ordinary kill");
}

// spec: S-0525 claim 11 (RF8 part a: a field made in the turn pass is read by the objects whose turns come after the detonation in that tick, not by those before it)
#[test]
fn a_field_made_in_the_turn_pass_hits_only_later_turns() {
    let (mut w, s) = base(|r| r.sim.general.rad_application_delay = Some(1));
    let p = probe(&mut w, "PROBE", at(0, 0));
    fire_at(&mut w, s, X);
    // Created after the bullet, with a filler between (the bullet's removal costs the next object its turn, S-0203 claim 10).
    probe(&mut w, "PROBE", at(9, 9));
    let q = probe(&mut w, "PROBE", at(0, 0));
    testkit::run(&mut w, 1);
    assert!(w.radiation.fields.len() == 1);
    assert_eq!(
        health(&w, p),
        100_000,
        "P took its turn before the detonation"
    );
    assert_eq!(health(&w, q), 100_000 - 62, "Q took its turn after it");
}

// spec: S-0525 claim 11 (RF8 part b: self-healing comes before the radiation hit in the same turn, so a self-healing unit at full strength that is hit in a healing tick ends the tick below full strength by the whole hit)
#[test]
fn self_healing_comes_before_the_hit() {
    let (mut w, s) = base(|r| {
        vehicle(r, "HEALER", |t| t.combat.self_healing = Some(true));
        r.sim.general.repair_rate = Some(d("0.0166667"));
    });
    assert_eq!(w.rules().combat.general.repair_period, 15);
    let healer = testkit::spawn(&mut w, Kind::Vehicle, "HEALER", 1, at(0, 0), 0);
    run_to(&mut w, 20);
    detonate(&mut w, s, X);
    run_to(&mut w, 30);
    // Full health just before the common tick 30.
    w.obj_mut(healer).unwrap().health = 100_000;
    testkit::run(&mut w, 1);
    assert_eq!(health(&w, healer), 100_000 - 62);
    // The next common tick: 1 healed before the hit.
    run_to(&mut w, 45);
    let h = health(&w, healer);
    testkit::run(&mut w, 1);
    assert_eq!(health(&w, healer), h + 1 - 62);
}

fn bullet(r: &mut glasnost_rules::schema::Ruleset) -> &mut ProjectileType {
    r.sim.projectiles.get_mut("Bullet").unwrap()
}

// spec: S-0525 claim 1 (RF9 part a: a projectile with cluster count 2 leaves a field at each detonation point, its two points in different cells)
#[test]
fn each_cluster_detonation_leaves_radiation() {
    let (mut w, s) = base(|r| bullet(r).cluster = Some(2));
    detonate(&mut w, s, X);
    assert_eq!(
        w.radiation.fields.len(),
        2,
        "{:?}",
        w.radiation
            .fields
            .iter()
            .map(|f| f.centre)
            .collect::<Vec<_>>()
    );
    assert_eq!(w.radiation.fields[0].centre, at(0, 0));
}

// spec: S-0525 claim 1 (RF9 part b: an airburst projectile whose airburst weapon has radiation level 1000 and whose own weapon has 0 leaves no field at all; its nine children carry no weapon)
#[test]
fn airburst_children_leave_no_radiation() {
    for (own, fields) in [(0, 0), (1000, 1)] {
        let (mut w, s) = base(|r| {
            bullet(r).airburst = Some(true);
            bullet(r).airburst_weapon = Some("RadChild".into());
            r.sim.weapons.get_mut("RadGun").unwrap().rad_level = Some(own);
            r.sim.weapons.insert(
                "RadChild".into(),
                WeaponType {
                    damage: Some(0),
                    range: Some(d("6")),
                    speed: Some(77),
                    projectile: Some("Pellet".into()),
                    warhead: Some("RADW".into()),
                    rad_level: Some(1000),
                    ..Default::default()
                },
            );
            r.sim
                .projectiles
                .insert("Pellet".into(), ProjectileType::default());
        });
        detonate(&mut w, s, X);
        testkit::run(&mut w, 120);
        assert!(w.projectiles.is_empty(), "the children have landed");
        assert_eq!(w.radiation.fields.len(), fields, "own weapon level {own}");
    }
}

// spec: S-0525 claim 1 (RF9 part c: an exploding vehicle whose death weapon is a radiation weapon leaves a field at its cell)
#[test]
fn a_death_weapon_leaves_radiation() {
    let (mut w, _s) = base(|r| {
        vehicle(r, "BOMB", |t| {
            t.combat.explodes = Some(true);
            t.combat.death_weapon = Some("RadGun".into());
        });
    });
    let bomb = testkit::spawn(&mut w, Kind::Vehicle, "BOMB", 1, at(3, 3), 0);
    damage::apply(
        &mut w,
        bomb,
        Hit {
            amount: 1_000_000,
            warhead: None,
            distance: 0,
            attacker: None,
            house: None,
            ignore_defences: true,
            no_escape: false,
        },
    );
    assert_eq!(w.radiation.fields.len(), 1);
    assert_eq!(w.radiation.fields[0].centre, at(3, 3));
}

// spec: S-0525 claim 1 (RF9 part d: a parasite warhead leaves radiation too, whatever the warhead does)
#[test]
fn a_parasite_warhead_leaves_radiation() {
    let (mut w, s) = base(|r| r.sim.warheads.get_mut("RADW").unwrap().parasite = Some(true));
    detonate(&mut w, s, X);
    assert_eq!(w.radiation.fields.len(), 1);
}

// spec: S-0525 claim 12 (RF10: making a field, adding to it, decay steps, light changes and the end take no value from the scenario random stream: a shot of a radiation weapon and an identical shot of one of level 0 leave identical random states)
#[test]
fn fields_take_no_draws() {
    let run = |level: i32| {
        let (mut w, s) = base(|r| r.sim.weapons.get_mut("RadGun").unwrap().rad_level = Some(level));
        detonate(&mut w, s, X);
        run_to(&mut w, 200);
        detonate(&mut w, s, X);
        run_to(&mut w, 7000);
        (w.rng.clone(), w.radiation.fields.len())
    };
    let (rad, fields) = run(1000);
    let (none, _) = run(0);
    assert_eq!(fields, 0, "the fields have ended");
    assert_eq!(rad, none);
}

// spec: S-0525 claim 9 (the exposure of a cell is read at the unit's own position cell, so an unarmed unit anywhere reads its cell: checked through a hit at the corner of the reach, 2 cells away at spread 2)
#[test]
fn the_hit_amount_follows_the_cell() {
    let (mut w, s) = fielded(|_| {});
    let p = probe(&mut w, "PROBE", at(0, 2));
    detonate(&mut w, s, X);
    assert_eq!(
        radiation::exposure(&w, at(0, 2)),
        199,
        "just below 200 in doubles"
    );
    run_to(&mut w, 31);
    assert_eq!(health(&w, p), 100_000 - 12);
}
