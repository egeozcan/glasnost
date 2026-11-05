//! Claim tests of what a garrisoned building does (the occupant claims of
//! S-0308, S-0313, S-0301, S-0302, S-0307, S-0311 and S-0215 that S-0508
//! fills in), the map's eviction and hunt actions (S-0011, S-0508 claim 13)
//! and the death-inside rule of claim 23, on synthetic rules.

mod structures_common;

use glasnost_map::triggers::{Action, Event, Triggers};
use glasnost_rules::schema::Ruleset;
use glasnost_sim::combat::effects::Effect;
use glasnost_sim::combat::{damage, range, veterancy, weapon};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::structures::garrison::{self, Evict};
use glasnost_sim::testkit;
use glasnost_sim::world::{ObjectId, Target, World};
use structures_common::*;

fn rules_with(f: impl FnOnce(&mut Ruleset)) -> Ruleset {
    let mut r = ruleset();
    // An infantry type whose occupying weapon is a disc laser, one that is
    // elite-capable, and muzzle points for the 3 × 2 hut.
    r.registries
        .get_mut("infantry")
        .unwrap()
        .push("BEAMER".into());
    let mut beamer = r.sim.infantry.get("GARRI").unwrap().clone();
    beamer.occupy_weapon = Some("Beam".into());
    r.sim.infantry.insert("BEAMER".into(), beamer);
    r.sim.buildings.get_mut("HUT3").unwrap().garrison_muzzles =
        Some(vec![[0, 0], [10, 0], [0, 20]]);
    // A target that costs something, for experience.
    r.sim.vehicles.get_mut("CART").unwrap().cost = Some(500);
    f(&mut r);
    r
}

/// A game with a 3 × 2 hut of Alpha at (0, 0) and one occupant of each
/// named type inside, and an enemy cart `cells` cells east of the hut.
fn fixture(
    r: &Ruleset,
    occupants: &[&str],
    cells: i32,
) -> (World, ObjectId, Vec<ObjectId>, ObjectId) {
    let mut w = game_on(r, Triggers::default());
    let b = put(&mut w, Kind::Building, "HUT3", 0, 0, 0);
    let us: Vec<ObjectId> = occupants
        .iter()
        .map(|n| {
            let u = put(&mut w, Kind::Infantry, n, 0, -8, -8);
            assert!(garrison::attempt(&mut w, u, b), "{n} went inside");
            u
        })
        .collect();
    let cart = put(&mut w, Kind::Vehicle, "CART", 1, 1 + cells, 0);
    (w, b, us, cart)
}

fn shots(
    w: &mut World,
) -> Vec<(
    glasnost_sim::combat::data::WeaponId,
    glasnost_sim::world::Pos,
)> {
    std::mem::take(&mut w.effects)
        .into_iter()
        .filter_map(|e| match e {
            Effect::Shot { weapon, from, .. } => Some((weapon, from)),
            // A disc laser's shot has no report or projectile (S-0615 claim 12): its own event.
            Effect::Disc {
                weapon,
                event: glasnost_sim::combat::effects::DiscEvent::Shot,
                pos,
                ..
            } => Some((weapon, pos)),
            _ => None,
        })
        .collect()
}

fn weapon_named(w: &World, n: &str) -> glasnost_sim::combat::data::WeaponId {
    let r = w.rules();
    (0..r.combat.weapons.len() as u32)
        .find(|&i| r.combat.weapon(i).name == n)
        .unwrap_or_else(|| panic!("no weapon {n}"))
}

// spec: S-0308 claim 13 (a garrisoned building's weapon comes from its current occupant: the occupant's occupying weapon, else its own slot 0; an elite occupant's elite occupying weapon; quirks.elite_occupant_skips_occupy_weapon)
#[test]
fn s0308_claim_13_the_weapon_of_the_current_occupant() {
    let r = rules_with(|_| {});
    let (mut w, b, us, _) = fixture(&r, &["GARRI", "BARE", "ELITE"], 2);
    let occ = weapon_named(&w, "OccGun");
    let elite = weapon_named(&w, "EliteOccGun");
    let rifle = weapon_named(&w, "Rifle");
    assert_eq!(weapon::slot_weapon(&w, b, 0), Some(occ));
    assert_eq!(
        weapon::slot_weapon(&w, b, 1),
        Some(occ),
        "every slot a rule asks about"
    );
    w.obj_mut(b).unwrap().structure.index = 1;
    assert_eq!(
        weapon::slot_weapon(&w, b, 0),
        Some(rifle),
        "no occupying weapon: its own slot 0"
    );
    // An elite occupant with an elite occupying weapon.
    w.obj_mut(us[2]).unwrap().combat.experience =
        glasnost_num::F64::from_i64(3, w.rules().mode).to_f32(w.rules().mode);
    w.obj_mut(b).unwrap().structure.index = 2;
    assert_eq!(veterancy::rank(&w, us[2]), veterancy::Rank::Elite);
    assert_eq!(weapon::slot_weapon(&w, b, 0), Some(elite));
    // Without an elite occupying weapon, an elite occupant fires its own primary (the quirk) or the occupying weapon.
    let mut r2 = rules_with(|r| {
        r.sim.infantry.get_mut("ELITE").unwrap().elite_occupy_weapon = None;
    });
    r2.sim.general.condition_red = Some(d("0.5"));
    for skips in [true, false] {
        let (mut w, b, us, _) = fixture(&r2, &["ELITE"], 2);
        let mut rules = (*w.rules_arc()).clone();
        rules.quirks.elite_occupant_skips_occupy_weapon = skips;
        w.rules = Some(std::sync::Arc::new(rules));
        w.obj_mut(us[0]).unwrap().combat.experience =
            glasnost_num::F64::from_i64(3, w.rules().mode).to_f32(w.rules().mode);
        let want = if skips { rifle } else { occ };
        assert_eq!(weapon::slot_weapon(&w, b, 0), Some(want), "skips {skips}");
    }
}

// spec: S-0309 claim 2 (only an occupiable building that holds no occupants is kept from searching passively: a garrisoned building picks an enemy in range by itself, before anything shoots at it)
#[test]
fn s0309_claim_2_a_garrisoned_building_searches() {
    let r = rules_with(|_| {});
    let (mut w, b, _, cart) = fixture(&r, &["GARRI"], 2);
    assert!(glasnost_sim::combat::acquire::may_search_passively(&w, b));
    let full = w.obj(b).unwrap().health;
    let mut picked = None;
    for t in 0..60 {
        testkit::run(&mut w, 1);
        if w.obj(b).unwrap().target == Some(Target::Object(cart)) {
            picked = Some(t);
            break;
        }
    }
    assert!(picked.is_some(), "the garrison took the enemy in range");
    assert_eq!(
        w.obj(b).unwrap().health,
        full,
        "it was not shot first: no retaliation"
    );
    // Emptied, the same building is kept from searching.
    let (w, b, _, _) = fixture(&r, &[], 2);
    assert!(!glasnost_sim::combat::acquire::may_search_passively(&w, b));
}

// spec: S-0308 table P row 2 (a garrisoned building whose occupants fire uses slot 0)
#[test]
fn s0308_table_p_row_2() {
    let r = rules_with(|_| {});
    let (w, b, _, cart) = fixture(&r, &["GARRI"], 2);
    assert_eq!(weapon::choose_slot(&w, b, Some(Target::Object(cart))), 0);
    assert!(
        weapon::armed(&w, b),
        "a building with occupants that fire counts as armed"
    );
}

// spec: S-0313 claim 6 and S-0308 claim 13 (after every shot the index becomes (k + 1) mod n and the next occupant's weapon fires; a disc laser shot does not advance it)
#[test]
fn s0313_claim_6_the_index_cycles_and_the_disc_laser_stays() {
    let r = rules_with(|_| {});
    let (mut w, b, _us, cart) = fixture(&r, &["GARRI", "ELITE", "BEAMER"], 2);
    let beam = weapon_named(&w, "Beam");
    let occ = weapon_named(&w, "OccGun");
    let mut seen = Vec::new();
    for k in 0..4 {
        let slot = weapon::choose_slot(&w, b, Some(Target::Object(cart)));
        w.effects.clear();
        assert!(
            glasnost_sim::combat::fire::shot(&mut w, b, slot, Target::Object(cart)),
            "shot {k}"
        );
        seen.push((w.obj(b).unwrap().structure.index, shots(&mut w)[0].0));
    }
    // Occupants 0 and 1 (ordinary weapons) advance the index; occupant 2's disc laser leaves it.
    assert_eq!(seen[0].0, 1);
    assert_eq!(seen[0].1, occ);
    assert_eq!(seen[1].0, 2);
    assert_eq!(seen[2].0, 2, "a disc laser shot does not advance it");
    assert_eq!(seen[2].1, beam);
    assert_eq!(seen[3].0, 2);
    assert_eq!(seen[3].1, beam, "the same occupant shoots again");
}

// spec: S-0313 claim 6 (a garrisoned building fires from its base point shifted by its muzzle point k, map x and y only; a point the art does not give is (0, 0))
#[test]
fn s0313_claim_6_the_muzzle_point_follows_the_index() {
    let r = rules_with(|_| {});
    let (mut w, b, _, cart) = fixture(&r, &["GARRI", "GARRI", "GARRI"], 2);
    let base = w.obj(b).unwrap().pos;
    let c = base.cell();
    let mut from = Vec::new();
    for _ in 0..3 {
        w.effects.clear();
        glasnost_sim::combat::fire::shot(&mut w, b, 0, Target::Object(cart));
        from.push(shots(&mut w)[0].1);
    }
    let (dx1, dy1) = garrison::muzzle_shift(&w, b, 1);
    let (dx2, dy2) = garrison::muzzle_shift(&w, b, 2);
    assert_eq!(
        (from[0].x, from[0].y),
        (256 * c.x, 256 * c.y),
        "muzzle point 0 is (0, 0)"
    );
    assert_eq!((from[1].x, from[1].y), (256 * c.x + dx1, 256 * c.y + dy1));
    assert_eq!((from[2].x, from[2].y), (256 * c.x + dx2, 256 * c.y + dy2));
    assert!(dx1 != 0 && dy2 != 0);
    assert_eq!(from[1].z, base.z, "the height is the base point's");
}

// spec: S-0301 claim 7 and F1 (a garrisoned building's delay is D₂ / n, truncated, then divided by the occupy factor m when m is above 0)
#[test]
fn s0301_claim_7_the_delay_quotient() {
    for (n, m, lo, hi) in [
        (1, "1", 10, 12),
        (3, "1", 3, 4),
        (1, "2", 5, 6),
        (3, "2", 1, 2),
    ] {
        let r = rules_with(|r| r.sim.general.occupy_rof_multiplier = Some(d(m)));
        let names = vec!["GARRI"; n];
        let (mut w, b, _, cart) = fixture(&r, &names, 2);
        glasnost_sim::combat::fire::shot(&mut w, b, 0, Target::Object(cart));
        let delay = w.obj(b).unwrap().combat.rearm_delay;
        assert!((lo..=hi).contains(&delay), "n {n} m {m}: delay {delay}");
    }
    // m = 0 does not divide.
    let r = rules_with(|r| r.sim.general.occupy_rof_multiplier = Some(d("0")));
    let (mut w, b, _, cart) = fixture(&r, &["GARRI"], 2);
    glasnost_sim::combat::fire::shot(&mut w, b, 0, Target::Object(cart));
    assert!((10..=12).contains(&w.obj(b).unwrap().combat.rearm_delay));
}

// spec: S-0302 claim 2 and F1 (the garrison multiplier applies to a garrisoned building's shot damage; an empty building does not take it)
#[test]
fn s0302_claim_2_the_garrison_damage_multiplier() {
    let r = rules_with(|r| r.sim.general.occupy_damage_multiplier = Some(d("1.5")));
    let (mut w, b, _, _) = fixture(&r, &["GARRI"], 2);
    let occ = weapon_named(&w, "OccGun");
    assert_eq!(damage::shot_damage(&w, b, occ), 30);
    garrison::evict(&mut w, b, Evict::PLAIN);
    assert_eq!(
        damage::shot_damage(&w, b, occ),
        20,
        "no occupants: no multiplier"
    );
}

// spec: S-0307 claim 4 (a garrisoned building's range is 256 × (⌊min(W, H) / 2⌋ + G): the weapon's own range is not used)
#[test]
fn s0307_claim_4_the_garrison_range() {
    let r = rules_with(|r| r.sim.general.occupy_weapon_range = Some(5));
    let (mut w, b, _, _) = fixture(&r, &["GARRI"], 2);
    let occ = weapon_named(&w, "OccGun");
    // Range (1 + 5) cells from the measured point; the weapon's own is 3.
    let near = put(&mut w, Kind::Vehicle, "CART", 1, 5, 0);
    let far = put(&mut w, Kind::Vehicle, "CART", 1, 9, 2);
    assert!(range::in_range(
        &w,
        b,
        Some(occ),
        Some(Target::Object(near)),
        None
    ));
    assert!(!range::in_range(
        &w,
        b,
        Some(occ),
        Some(Target::Object(far)),
        None
    ));
    garrison::evict(&mut w, b, Evict::PLAIN);
    assert!(
        !range::in_range(&w, b, Some(occ), Some(Target::Object(near)), None),
        "its own range of 3 cells"
    );
}

// spec: S-0311 claim 8 (a destruction credited to a garrisoned building gives the experience to the occupant the shared index selects, regardless of its trainability; quirks.occupant_experience_next_in_turn and occupant_experience_untrained)
#[test]
fn s0311_claim_8_experience_goes_to_the_selected_occupant() {
    let r = rules_with(|_| {});
    let (mut w, b, us, cart) = fixture(&r, &["GARRI", "GARRI", "GARRI"], 2);
    w.obj_mut(b).unwrap().structure.index = 1;
    veterancy::destruction_credited(&mut w, cart, Some(b), Some(A));
    let gained = |w: &World, u: ObjectId| {
        w.obj(u)
            .unwrap()
            .combat
            .experience
            .to_f64()
            .gt(glasnost_num::F64::ZERO)
    };
    assert!(!gained(&w, us[0]));
    assert!(gained(&w, us[1]), "the occupant whose turn it is");
    assert!(!gained(&w, us[2]));
    // The switch off: the shooter, the occupant before it.
    let (mut w, b, us, cart) = fixture(&r, &["GARRI", "GARRI", "GARRI"], 2);
    let mut rules = (*w.rules_arc()).clone();
    rules.quirks.occupant_experience_next_in_turn = false;
    w.rules = Some(std::sync::Arc::new(rules));
    w.obj_mut(b).unwrap().structure.index = 1;
    veterancy::destruction_credited(&mut w, cart, Some(b), Some(A));
    assert!(gained(&w, us[0]));
    assert!(!gained(&w, us[1]));
}

// spec: S-0215 claim 32 (a building that can be occupied but has no occupants chooses the anti-air delay whether or not it has a target; with occupants it attacks)
#[test]
fn s0215_claim_32_an_occupied_building_attacks() {
    let r = rules_with(|_| {});
    let (mut w, b, _, cart) = fixture(&r, &["GARRI"], 2);
    let t = w.tick;
    w.obj_mut(b).unwrap().target = Some(Target::Object(cart));
    testkit::run(&mut w, 3);
    let _ = t;
    let beh = w.obj(b).unwrap().beh.current;
    assert_eq!(beh, Some(Behaviour::Attack), "garrisoned: attack");
    // Empty: it stays in guard.
    let (mut w, b, _, cart) = fixture(&r, &[], 2);
    w.obj_mut(b).unwrap().target = Some(Target::Object(cart));
    testkit::run(&mut w, 3);
    assert_ne!(w.obj(b).unwrap().beh.current, Some(Behaviour::Attack));
}

// spec: S-0405 table T1 (an occupiable building without occupants that fire is illegal to fire; with occupants it is not)
#[test]
fn s0405_t1_occupants_make_the_building_fire() {
    let r = rules_with(|_| {});
    let (mut w, b, _, cart) = fixture(&r, &["GARRI"], 2);
    let ans = |w: &World| glasnost_sim::econ::power::firing_answer(w, b);
    assert_eq!(ans(&w), None);
    garrison::evict(&mut w, b, Evict::PLAIN);
    assert_eq!(ans(&w), Some(glasnost_sim::combat::check::Answer::Illegal));
    let _ = cart;
}

// spec: S-0508 claim 13 and S-0011 (a map trigger's eviction action, code 111: the first attached building with occupants evicts them, no hunting, no inside fallback)
#[test]
fn claim_13_trigger_eviction_action() {
    let ev = Event {
        code: 1,
        kind: 0,
        value: -1,
        type_name: None,
    };
    let t = trigger(
        "EVICT",
        ev,
        vec![Action {
            code: 111,
            ..Default::default()
        }],
    );
    let tr = Triggers {
        tags: vec![tag_def(0, &t, 2)],
        triggers: vec![t],
        ..Default::default()
    };
    let mut w = game_on(&ruleset(), tr);
    let b = put(&mut w, Kind::Building, "HUT", 2, 0, 0);
    let other = put(&mut w, Kind::Building, "HUT", 2, 8, 0);
    let us: Vec<ObjectId> = (0..2)
        .map(|_| {
            let u = put(&mut w, Kind::Infantry, "GARRI", 0, -8, -8);
            garrison::attempt(&mut w, u, b);
            u
        })
        .collect();
    let o = put(&mut w, Kind::Infantry, "GARRI", 0, -8, -8);
    garrison::attempt(&mut w, o, other);
    tag(&mut w, b, "EVICT");
    // An entry raises the tag.
    let late = put(&mut w, Kind::Infantry, "GARRI", 0, -8, -9);
    garrison::attempt(&mut w, late, b);
    assert!(us.iter().all(|&u| on_map(&w, u)), "evicted by the action");
    assert!(
        us.iter()
            .all(|&u| w.obj(u).unwrap().beh.queued != Some(Behaviour::Hunt)),
        "no hunting"
    );
    assert!(
        !on_map(&w, o),
        "a building the trigger is not attached to is untouched"
    );
}

// spec: S-0508 claim 13 and S-0011 claim 23 (the house-wide hunt action, code 6: the number slot is a house value; the garrison evictions hunt, without inside fallback)
#[test]
fn claim_13_house_wide_hunt_action() {
    let ev = Event {
        code: 1,
        kind: 0,
        value: -1,
        type_name: None,
    };
    // House value 0: the first house playing country 0 (Alpha).
    let t = trigger(
        "HUNT",
        ev,
        vec![Action {
            code: 6,
            number: 0,
            ..Default::default()
        }],
    );
    let tr = Triggers {
        tags: vec![tag_def(0, &t, 2)],
        triggers: vec![t],
        ..Default::default()
    };
    let mut w = game_on(&ruleset(), tr);
    let b = put(&mut w, Kind::Building, "HUT", 0, 0, 0);
    let us: Vec<ObjectId> = (0..2)
        .map(|_| {
            let u = put(&mut w, Kind::Infantry, "GARRI", 0, -8, -8);
            garrison::attempt(&mut w, u, b);
            u
        })
        .collect();
    tag(&mut w, b, "HUNT");
    let late = put(&mut w, Kind::Infantry, "GARRI", 0, -8, -9);
    let before = w.obj(late).unwrap().beh.queued;
    garrison::attempt(&mut w, late, b);
    let _ = before;
    for &u in &us {
        let o = w.obj(u).unwrap();
        assert!(o.on_map);
        assert!(
            o.beh.queued == Some(Behaviour::Hunt) || o.beh.current == Some(Behaviour::Hunt),
            "hunting"
        );
    }
}

fn kill_inside(w: &mut World, id: ObjectId) {
    let hit = damage::Hit {
        amount: 10_000,
        warhead: w.rules().combat.general.c4_warhead,
        distance: 0,
        attacker: None,
        house: None,
        ignore_defences: true,
        no_escape: false,
    };
    glasnost_sim::combat::death::kill(w, id, &hit);
}

// spec: S-0508 claim 23 (engine design rule, G18, answers Q-0970; S-0308 claim 13, S-0313 claim 6, S-0301 claim 7, S-0311 claim 8): occupants U, V, W in entry order, index 1, V dies inside: the order is U, W, the count 2, the index 1, W's weapon fires from muzzle 1 on the next ordinary shot and the successful shot advances the index to 0
#[test]
fn claim_23_death_inside_the_firing_choices() {
    let r = rules_with(|_| {});
    let (mut w, b, us, cart) = fixture(&r, &["GARRI", "BEAMER", "GARRI"], 2);
    w.obj_mut(b).unwrap().structure.index = 1;
    kill_inside(&mut w, us[1]);
    assert_eq!(garrison::occupants(&w, b), vec![us[0], us[2]]);
    assert_eq!(garrison::count(&w, b), 2, "the count after removal");
    assert_eq!(w.obj(b).unwrap().structure.index, 1);
    let beam = weapon_named(&w, "Beam");
    let occ = weapon_named(&w, "OccGun");
    assert_ne!(weapon::slot_weapon(&w, b, 0), Some(beam));
    assert_eq!(weapon::slot_weapon(&w, b, 0), Some(occ), "W's weapon");
    assert_eq!(garrison::current_occupant(&w, b), Some(us[2]));
    w.effects.clear();
    assert!(glasnost_sim::combat::fire::shot(
        &mut w,
        b,
        0,
        Target::Object(cart)
    ));
    assert_eq!(
        w.obj(b).unwrap().structure.index,
        0,
        "advanced from W's place by the new count"
    );
    assert!(garrison::evict(&mut w, b, Evict::PLAIN));
    assert!(on_map(&w, us[0]) && on_map(&w, us[2]));
}

// spec: S-0508 claim 23 and S-0311 claim 8 (a credited destruction after a death inside gives the experience to the occupant the repaired index names; with no occupant left there is no occupant recipient)
#[test]
fn claim_23_experience_follows_the_repaired_index() {
    let r = rules_with(|_| {});
    let (mut w, b, us, cart) = fixture(&r, &["GARRI", "GARRI", "GARRI"], 2);
    w.obj_mut(b).unwrap().structure.index = 1;
    kill_inside(&mut w, us[1]);
    let gained = |w: &World, u: ObjectId| {
        w.obj(u)
            .unwrap()
            .combat
            .experience
            .to_f64()
            .gt(glasnost_num::F64::ZERO)
    };
    veterancy::destruction_credited(&mut w, cart, Some(b), Some(A));
    assert!(gained(&w, us[2]), "W, the occupant at the repaired index");
    assert!(!gained(&w, us[0]));
    // All of them dead: nobody receives it.
    kill_inside(&mut w, us[0]);
    kill_inside(&mut w, us[2]);
    assert_eq!(garrison::experience_recipient(&w, b), None);
    assert_eq!(garrison::count(&w, b), 0);
}

// spec: S-0508 claim 23 and S-0301 claim 7 (the delay quotient uses the count after removal)
#[test]
fn claim_23_the_delay_uses_the_count_after_removal() {
    // Multiplier 1, rate 10: three occupants give 3 to 4, two give 5 to 6, one gives 10 to 12.
    let r = rules_with(|r| r.sim.general.occupy_rof_multiplier = Some(d("1")));
    let (mut w, b, us, cart) = fixture(&r, &["GARRI", "GARRI", "GARRI"], 2);
    kill_inside(&mut w, us[1]);
    glasnost_sim::combat::fire::shot(&mut w, b, 0, Target::Object(cart));
    let delay = w.obj(b).unwrap().combat.rearm_delay;
    assert!((5..=6).contains(&delay), "two occupants: delay {delay}");
}
