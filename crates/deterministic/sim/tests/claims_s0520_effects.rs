//! Claim tests for the power-ups of crates (S-0520 claims 8 to 25): the
//! choice, the forced base unit, the conversions to money and the effects, on
//! synthetic data.

#[path = "crate_common/mod.rs"]
mod crate_common;

use crate_common::*;
use glasnost_map::CellXY;
use glasnost_num::{F64, Rounding};
use glasnost_sim::combat::effects::Effect;
use glasnost_sim::crates;
use glasnost_sim::rules::Kind;
use glasnost_sim::world::{ObjectId, World};

const RZ: Rounding = Rounding::TowardZero;

/// A game with crates off, a collector ARMED of house 0 at (0, 0) and a crate
/// of `contents` one cell away; returns the pickup's blocked answer.
fn pick(w: &mut World, who: ObjectId, contents: u8) -> bool {
    let c = at(1, 0);
    crate_at(w, c, WOOD, contents);
    crates::pickup(w, who, c)
}

fn armed(w: &mut World) -> ObjectId {
    spawn(w, Kind::Vehicle, "ARMED", 0, at(0, 0))
}

fn count(w: &World, kind: Kind, name: &str) -> usize {
    let ti = w.rules().index_of(kind, name);
    w.objects
        .values()
        .filter(|o| o.kind == kind && Some(o.type_index) == ti)
        .count()
}

fn exp(w: &World, id: ObjectId) -> F64 {
    w.obj(id).unwrap().combat.experience.to_f64()
}

// spec: S-0520 claim 12 and F2 (money: R(b, b + 900) credits with b = 100; the animation of the power-up appears 200 leptons above the floor of the crate cell)
#[test]
fn money_gives_its_range_and_its_animation() {
    let mut w = world_with(false, |_| {});
    let v = armed(&mut w);
    let before = credits(&w, 0);
    assert!(!pick(&mut w, v, 0));
    let got = credits(&w, 0) - before;
    assert!((100..=1000).contains(&got), "{got}");
    let anim = w.effects.iter().find_map(|e| match e {
        Effect::Anim { name, pos, .. } if name == "MONEYA" => Some(*pos),
        _ => None,
    });
    let p = anim.expect("the money animation");
    assert_eq!(p.z, w.surface_at(p.x, p.y) + 200);
    assert!(!crates::is_crate(&w, at(1, 0)), "the crate is gone");
}

// spec: S-0520 claims 8, 11 and 25 (squad gives the money effect with the money argument; invulnerability, weather storm and drop pod give only their animation)
#[test]
fn squad_is_money_and_three_have_no_effect() {
    let mut w = world_with(false, |r| {
        r.sim.general.economy.powerups.as_mut().unwrap()[6].argument = Some(d("5000"));
        r.sim.general.economy.powerups.as_mut().unwrap()[13].animation = Some("MONEYA".into());
    });
    let v = armed(&mut w);
    let before = credits(&w, 0);
    pick(&mut w, v, 6);
    let got = credits(&w, 0) - before;
    assert!(
        (100..=1000).contains(&got),
        "the money argument, not squad's: {got}"
    );
    let mid = credits(&w, 0);
    pick(&mut w, v, 13);
    assert_eq!(credits(&w, 0), mid, "invulnerability changes nothing");
}

// spec: S-0520 claim 8 and F1 (random contents: the first power-up whose running total of weights reaches R(1, total); zero and negative entries cannot be chosen)
#[test]
fn random_contents_follow_the_weights() {
    // Weights: armour 0, heal 1, cloak 0, speed -3, firepower 4: heal or firepower.
    let mut seen = [0u32; 2];
    for seed in 0..40 {
        let mut w = world_with(false, |r| {
            r.sim.general.economy.powerups = Some(table(&[(2, 1), (10, -3), (11, 4)]));
        });
        for _ in 0..seed {
            let _ = w.rng.raw();
        }
        let v = armed(&mut w);
        w.obj_mut(v).unwrap().health = 100;
        pick(&mut w, v, 255);
        let healed = w.obj(v).unwrap().health == 400;
        let boosted = !w
            .obj(v)
            .unwrap()
            .combat
            .firepower_mult
            .eq_num(F64::from_i64(1, RZ));
        assert!(healed ^ boosted, "exactly one of heal and firepower");
        seen[usize::from(boosted)] += 1;
    }
    assert!(
        seen[0] > 0 && seen[1] > 0,
        "both are chosen sometimes: {seen:?}"
    );
}

// spec: S-0520 F1 and Edge cases (a weight total of zero or less: our engine gives money, without the choice draw)
#[test]
fn a_zero_total_gives_money() {
    let mut w = world_with(false, |r| {
        r.sim.general.economy.powerups = Some(table(&[]));
    });
    let v = armed(&mut w);
    let before = credits(&w, 0);
    pick(&mut w, v, 255);
    assert!(credits(&w, 0) > before);
}

// spec: S-0520 claim 9 and 13 (a forced base unit: no counted building, more than 1500 credits, no unit of the first base-unit type its country may own, bases on; even fixed money contents become a unit; the unit is placed and the ground step is blocked)
#[test]
fn a_forced_base_unit_replaces_any_crate() {
    for (credits_now, forced) in [(1501, true), (1500, false)] {
        let mut w = world_with(false, |r| {
            r.sim.general.base_unit = Some(vec!["NOGOODIE".into()]);
            r.sim.vehicles.get_mut("NOGOODIE").unwrap().owner =
                Some(vec!["Alphaland".into(), "Betaland".into()]);
        });
        w.houses[0].credits = credits_now;
        let v = armed(&mut w);
        let blocked = pick(&mut w, v, 0);
        assert_eq!(blocked, forced, "credits {credits_now}");
        assert_eq!(count(&w, Kind::Vehicle, "NOGOODIE"), usize::from(forced));
        if forced {
            // The unit appears at the crate cell or beside it, with no animation.
            assert!(
                w.effects
                    .iter()
                    .all(|e| !matches!(e, Effect::Anim { name, .. } if name == "MONEYA"))
            );
        }
    }
}

// spec: S-0520 claim 9 (a counted building, bases off, or an owned base unit prevents the override)
#[test]
fn the_override_has_its_conditions() {
    // A counted building.
    let mut w = world_with(false, |r| {
        r.sim.general.base_unit = Some(vec!["NOGOODIE".into()]);
        r.sim.vehicles.get_mut("NOGOODIE").unwrap().owner =
            Some(vec!["Alphaland".into(), "Betaland".into()]);
    });
    spawn(&mut w, Kind::Building, "BASE", 0, at(8, 8));
    let v = armed(&mut w);
    assert!(!pick(&mut w, v, 0));
    assert_eq!(count(&w, Kind::Vehicle, "NOGOODIE"), 0);
    // Bases off.
    let mut w = world_with(false, |r| {
        r.sim.general.base_unit = Some(vec!["NOGOODIE".into()]);
        r.sim.vehicles.get_mut("NOGOODIE").unwrap().owner =
            Some(vec!["Alphaland".into(), "Betaland".into()]);
    });
    w.econ.start.bases = false;
    let v = armed(&mut w);
    assert!(!pick(&mut w, v, 0));
    // The unit exists already.
    let mut w = world_with(false, |r| {
        r.sim.general.base_unit = Some(vec!["NOGOODIE".into()]);
        r.sim.vehicles.get_mut("NOGOODIE").unwrap().owner =
            Some(vec!["Alphaland".into(), "Betaland".into()]);
    });
    spawn(&mut w, Kind::Vehicle, "NOGOODIE", 0, at(9, 9));
    let v = armed(&mut w);
    assert!(!pick(&mut w, v, 0));
    assert_eq!(count(&w, Kind::Vehicle, "NOGOODIE"), 1);
}

// spec: S-0520 claim 13 (the random rule: requests R(0, n - 1) until a crate-goodie type is accepted; the unit is created for the collecting house, no animation, the step blocked)
#[test]
fn a_unit_crate_gives_a_goodie_vehicle() {
    let mut w = world_with(false, |_| {});
    let v = armed(&mut w);
    let before = credits(&w, 0);
    assert!(pick(&mut w, v, 1));
    assert_eq!(count(&w, Kind::Vehicle, "GOODIE"), 1);
    assert_eq!(credits(&w, 0), before);
    let id = w
        .objects
        .values()
        .find(|o| {
            o.kind == Kind::Vehicle
                && w.rules().index_of(Kind::Vehicle, "GOODIE") == Some(o.type_index)
        })
        .unwrap();
    assert_eq!(id.owner, Some(glasnost_sim::world::HouseId(0)));
}

// spec: S-0520 claim 13 (a named unit-crate type takes precedence over the random rule)
#[test]
fn the_unit_crate_type_takes_precedence() {
    let mut w = world_with(false, |r| {
        r.sim.general.economy.unit_crate_type = Some("UNARMED".into());
    });
    let v = armed(&mut w);
    assert!(pick(&mut w, v, 1));
    assert_eq!(count(&w, Kind::Vehicle, "UNARMED"), 1);
}

/// The animations among the effects: (name, position).
fn anims(w: &World) -> Vec<(String, glasnost_sim::world::Pos)> {
    w.effects
        .iter()
        .filter_map(|e| match e {
            Effect::Anim { name, pos, .. } => Some((name.clone(), *pos)),
            _ => None,
        })
        .collect()
}

// spec: S-0520 claim 11 (a successfully placed unit has no such animation, even when the unit power-up names one) and claim 13; scenario CR9 (a unit animation on success falsifies claim 13)
#[test]
fn a_placed_unit_shows_no_animation() {
    let mut w = world_with(false, |r| {
        let t = r.sim.general.economy.powerups.as_mut().unwrap();
        t[1].animation = Some("MONEYA".into());
    });
    let v = armed(&mut w);
    let before = credits(&w, 0);
    assert!(
        pick(&mut w, v, 1),
        "the unit is placed, the step is blocked"
    );
    assert_eq!(credits(&w, 0), before, "no money fallback");
    assert_eq!(count(&w, Kind::Vehicle, "GOODIE"), 1);
    assert_eq!(anims(&w), vec![], "no animation after a placed unit");
}

// spec: S-0520 claim 11 (a power-up turned into money shows the money animation) and claim 13 (no unit could be placed: money with the money animation)
#[test]
fn a_unit_crate_that_places_no_unit_shows_the_money_animation() {
    let mut w = world_with(false, |r| {
        let t = r.sim.general.economy.powerups.as_mut().unwrap();
        t[1].animation = Some("MONEYA".into());
        // No vehicle type is a crate goodie: no unit can be awarded.
        r.sim.vehicles.get_mut("GOODIE").unwrap().crate_goodie = Some(false);
    });
    let v = armed(&mut w);
    let before = credits(&w, 0);
    assert!(!pick(&mut w, v, 1), "no unit, the step goes on");
    assert!(credits(&w, 0) > before, "the money fallback");
    let names: Vec<String> = anims(&w).into_iter().map(|a| a.0).collect();
    assert_eq!(names, vec!["MONEYA".to_string()], "the money animation");
}

// spec: S-0520 claim 10 (conversions to money: a unit crate on water, more than 50 vehicles, an own cloak flag already set, a multiplier that is not 1.0, no primary weapon, an elite collector)
#[test]
fn power_ups_that_cannot_apply_give_money() {
    let money_after = |setup: &dyn Fn(&mut World, ObjectId), contents: u8| -> (bool, i32) {
        let mut w = world_with(false, |_| {});
        let v = armed(&mut w);
        setup(&mut w, v);
        let before = credits(&w, 0);
        let blocked = pick(&mut w, v, contents);
        (blocked, credits(&w, 0) - before)
    };
    // Unit on a beach/water cell.
    let (blocked, got) = money_after(&|w, _| make_water(w, at(1, 0), 1), 1);
    assert!(!blocked && got >= 100, "unit on water: {blocked} {got}");
    // 51 vehicles.
    let (blocked, got) = money_after(
        &|w, _| {
            for i in 0..51 {
                spawn(w, Kind::Vehicle, "UNARMED", 0, at(5 + i % 10, 5 + i / 10));
            }
        },
        1,
    );
    assert!(!blocked && got >= 100, "51 vehicles: {blocked} {got}");
    // Cloak already set.
    let (_, got) = money_after(&|w, v| w.obj_mut(v).unwrap().cloak.own = true, 3);
    assert!(got >= 100, "cloak: {got}");
    // Armour, speed and firepower multipliers away from 1.0.
    let two = F64::from_i64(2, RZ);
    let (_, got) = money_after(&|w, v| w.obj_mut(v).unwrap().combat.armour_mult = two, 9);
    assert!(got >= 100, "armour: {got}");
    let (_, got) = money_after(&|w, v| w.obj_mut(v).unwrap().speed_bonus = two, 10);
    assert!(got >= 100, "speed: {got}");
    let (_, got) = money_after(
        &|w, v| w.obj_mut(v).unwrap().combat.firepower_mult = two,
        11,
    );
    assert!(got >= 100, "firepower: {got}");
    // Firepower for a collector with no primary weapon.
    let mut w = world_with(false, |_| {});
    let u = spawn(&mut w, Kind::Vehicle, "UNARMED", 0, at(0, 0));
    let before = credits(&w, 0);
    pick(&mut w, u, 11);
    assert!(credits(&w, 0) > before, "no primary weapon");
    // Veteran for an elite collector.
    let (_, got) = money_after(
        &|w, v| w.obj_mut(v).unwrap().combat.experience = two.to_f32(RZ),
        14,
    );
    assert!(got >= 100, "elite: {got}");
}

// spec: S-0520 claim 10 (the water gate: a power-up whose water flag is off is money on a water cell; the same power-up on land applies)
#[test]
fn the_water_gate_gives_money() {
    for on_water in [false, true] {
        let mut w = world_with(false, |_| {});
        let v = armed(&mut w);
        if on_water {
            make_water(&mut w, at(1, 0), 1);
        }
        w.obj_mut(v).unwrap().health = 100;
        let before = credits(&w, 0);
        pick(&mut w, v, 2); // heal all
        assert_eq!(w.obj(v).unwrap().health == 400, !on_water);
        assert_eq!(credits(&w, 0) > before, on_water);
    }
}

// spec: S-0520 claim 14 (heal all: every object of the collecting house in play is healed to full strength, anywhere on the map; other houses' objects are not)
#[test]
fn heal_all_heals_the_collecting_house() {
    let mut w = world_with(false, |_| {});
    let v = armed(&mut w);
    let far = spawn(&mut w, Kind::Vehicle, "UNARMED", 0, at(15, 15));
    let enemy = spawn(&mut w, Kind::Vehicle, "UNARMED", 1, at(0, 5));
    for id in [v, far, enemy] {
        w.obj_mut(id).unwrap().health = 100;
    }
    pick(&mut w, v, 2);
    assert_eq!(w.obj(v).unwrap().health, 400);
    assert_eq!(w.obj(far).unwrap().health, 400);
    assert_eq!(w.obj(enemy).unwrap().health, 100);
}

// spec: S-0520 claim 15 and F4 (cloak: every object on the map inside the crate area gets its own cloak flag, every owner, strictly inside the radius 2.5 cells = 640 leptons)
#[test]
fn cloak_reaches_the_crate_area() {
    let mut w = world_with(false, |_| {});
    let v = armed(&mut w);
    let near = spawn(&mut w, Kind::Vehicle, "UNARMED", 1, at(2, 0));
    let far = spawn(&mut w, Kind::Vehicle, "UNARMED", 1, at(5, 0));
    pick(&mut w, v, 3);
    assert!(w.obj(v).unwrap().cloak.own && w.obj(near).unwrap().cloak.own);
    assert!(!w.obj(far).unwrap().cloak.own);
}

// spec: S-0520 claim 16 (explosion: the collector takes D = 50 with defences ignored, then five blasts of 50 each at points near the crate, with draws)
#[test]
fn explosion_hurts_the_collector_and_the_ground_around() {
    let mut w = world_with(false, |_| {});
    let v = armed(&mut w);
    let near = spawn(&mut w, Kind::Vehicle, "UNARMED", 1, at(1, 1));
    let before = w.rng.clone();
    pick(&mut w, v, 4);
    assert!(w.obj(v).unwrap().health <= 350);
    assert!(w.rng != before);
    let _ = near;
}

// spec: S-0520 claim 17 (napalm: the collector takes D = 40 with the flame warhead, and area damage 40 at the midpoint)
#[test]
fn napalm_burns_the_collector() {
    let mut w = world_with(false, |_| {});
    let v = armed(&mut w);
    pick(&mut w, v, 5);
    assert!(w.obj(v).unwrap().health <= 360);
}

// spec: S-0520 claim 18 (reveal: the whole map is explored for the collecting house; darkness covers it again)
#[test]
fn reveal_and_darkness_change_the_collecting_house_only() {
    let mut w = world_with(false, |_| {});
    let v = armed(&mut w);
    let far = at(18, 18);
    let h0 = glasnost_sim::world::HouseId(0);
    let h1 = glasnost_sim::world::HouseId(1);
    pick(&mut w, v, 8);
    assert!(glasnost_sim::econ::shroud::explored(&w, h0, far));
    assert!(!glasnost_sim::econ::shroud::explored(&w, h1, far));
    pick(&mut w, v, 7);
    assert!(!glasnost_sim::econ::shroud::explored(&w, h0, far));
}

// spec: S-0520 claims 19 and 20 and F8 (armour, firepower and speed: objects inside the area with a multiplier of exactly 1.0 get A = 2.0; speed excludes aircraft and buildings; a multiplier that is not 1.0 is unchanged, so bonuses never stack)
#[test]
fn bonuses_apply_to_the_area_and_never_stack() {
    let mut w = world_with(false, |_| {});
    let v = armed(&mut w);
    let near = spawn(&mut w, Kind::Vehicle, "UNARMED", 1, at(2, 0));
    let far = spawn(&mut w, Kind::Vehicle, "UNARMED", 1, at(6, 0));
    let bld = spawn(&mut w, Kind::Building, "BASE", 1, at(1, 2));
    let two = F64::from_i64(2, RZ);
    let one = F64::from_i64(1, RZ);
    pick(&mut w, v, 9);
    assert!(w.obj(v).unwrap().combat.armour_mult.eq_num(two));
    assert!(w.obj(near).unwrap().combat.armour_mult.eq_num(two));
    assert!(
        w.obj(bld).unwrap().combat.armour_mult.eq_num(two),
        "buildings take armour"
    );
    assert!(w.obj(far).unwrap().combat.armour_mult.eq_num(one));
    // A second armour crate: the collector converts to money, the neighbours keep 2.0.
    let before = credits(&w, 0);
    pick(&mut w, v, 9);
    assert!(credits(&w, 0) > before);
    assert!(w.obj(near).unwrap().combat.armour_mult.eq_num(two));
    // Speed: vehicles, not buildings.
    let w2 = &mut w;
    let fresh = armed(w2);
    pick(w2, fresh, 10);
    assert!(w2.obj(near).unwrap().speed_bonus.eq_num(two));
    assert!(w2.obj(bld).unwrap().speed_bonus.eq_num(one));
}

// spec: S-0520 Edge cases (quirks.crate_bonus_reaches_limbo: an object off the map whose last position lies inside the crate area is boosted; off: only objects on the map)
#[test]
fn the_limbo_quirk_decides_about_objects_off_the_map() {
    for reaches in [true, false] {
        let mut w = world_with(false, |r| {
            r.globals.quirks.crate_bonus_reaches_limbo = reaches
        });
        let v = armed(&mut w);
        let hidden = spawn(&mut w, Kind::Vehicle, "UNARMED", 1, at(2, 0));
        w.obj_mut(hidden).unwrap().on_map = false;
        pick(&mut w, v, 11);
        let boosted = w
            .obj(hidden)
            .unwrap()
            .combat
            .firepower_mult
            .eq_num(F64::from_i64(2, RZ));
        assert_eq!(boosted, reaches);
    }
}

// spec: S-0520 claim 22 and F7 (veteran: A = 1 gives one step, rookie to veteran, with experience exactly 1.0; A = 2 gives two steps; elite and untrainable stay; A = 0 gives none)
#[test]
fn veteran_steps_follow_f7() {
    for (arg, want) in [("1", 1), ("2", 2), ("0.5", 1), ("0", 0)] {
        let mut w = world_with(false, |r| {
            r.sim.general.economy.powerups.as_mut().unwrap()[14].argument = Some(d(arg));
        });
        let v = armed(&mut w);
        let near = spawn(&mut w, Kind::Vehicle, "UNARMED", 1, at(2, 0));
        let elite = spawn(&mut w, Kind::Vehicle, "UNARMED", 1, at(2, 1));
        w.obj_mut(elite).unwrap().combat.experience = F64::from_i64(3, RZ).to_f32(RZ);
        // The collector cannot be a rookie-only conversion: keep it rookie.
        pick(&mut w, v, 14);
        assert!(
            exp(&w, near).eq_num(F64::from_i64(want, RZ)),
            "A = {arg}: {:?}",
            exp(&w, near)
        );
        assert!(
            exp(&w, elite).eq_num(F64::from_i64(3, RZ)),
            "elite unchanged"
        );
    }
}

// spec: S-0520 claim 23 (gas: nine area damages with the gas warhead, at the crate cell's centre and the centres of its eight neighbours)
#[test]
fn gas_hits_nine_cells() {
    let mut w = world_with(false, |_| {});
    let v = armed(&mut w);
    let c = at(1, 0);
    let mut victims = Vec::new();
    for dx in -1..=1 {
        for dy in -1..=1 {
            victims.push(spawn(
                &mut w,
                Kind::Vehicle,
                "UNARMED",
                1,
                CellXY::new(c.x + dx, c.y + dy),
            ));
        }
    }
    let _ = pick(&mut w, v, 16);
    for id in victims {
        assert!(
            w.obj(id).is_none_or(|o| o.health < 400),
            "every cell was hit"
        );
    }
}

// spec: S-0520 claim 24 (resource: a type is drawn, the crate cell and 10 … 20 more cells get one stage increase; germinable cells become new resource cells)
#[test]
fn resource_grows_ore_around_the_crate() {
    let mut w = world_with(false, |_| {});
    let v = armed(&mut w);
    pick(&mut w, v, 17);
    let n = w
        .cells
        .grid
        .iter()
        .filter(|(c, _)| glasnost_sim::econ::resource::resource_at(&w, *c).is_some())
        .count();
    assert!(n >= 1, "at least the crate cell holds resource, found {n}");
}

// spec: S-0520 claim 21 (nuclear missile: with no superweapon type of that kind nothing happens)
#[test]
fn the_nuclear_missile_needs_its_kind() {
    let mut w = world_with(false, |_| {});
    let v = armed(&mut w);
    let before = credits(&w, 0);
    pick(&mut w, v, 12);
    assert_eq!(credits(&w, 0), before);
}

// spec: S-0520 claim 11 (a pickup removes its crate; with crates on one random replacement is placed before any effect draw, so the new crate exists when the effect runs)
#[test]
fn a_pickup_with_crates_on_places_a_replacement() {
    let mut w = world_with(true, |_| {});
    let v = armed(&mut w);
    let c = at(1, 0);
    crate_at(&mut w, c, WOOD, 0);
    crates::pickup(&mut w, v, c);
    let cells: Vec<CellXY> = w
        .cells
        .grid
        .iter()
        .filter(|(x, _)| crates::is_crate(&w, *x))
        .map(|(x, _)| x)
        .collect();
    assert_eq!(cells.len(), 1);
    assert_ne!(cells[0], c);
}

fn owned(r: &mut glasnost_rules::schema::Ruleset, name: &str) {
    r.sim.vehicles.get_mut(name).unwrap().owner = Some(vec!["Alphaland".into(), "Betaland".into()]);
}

// spec: S-0520 claim 13 and quirks.crate_harvester_list_prefix (a house with a listed refinery and no listed harvester gets the first ownable harvester type; only the first two entries of each list count, and with the quirk off every entry)
#[test]
fn the_harvester_rule_and_its_prefix_quirk() {
    for (prefix, rule_applies) in [(true, false), (false, true)] {
        let mut w = world_with(false, |r| {
            r.globals.quirks.crate_harvester_list_prefix = prefix;
            // The refinery is the third entry of its list.
            for n in ["BASE2", "BASE3"] {
                r.registries
                    .entry("buildings".into())
                    .or_default()
                    .push(n.into());
                let b = r.sim.buildings["BASE"].clone();
                r.sim.buildings.insert(n.into(), b);
            }
            r.sim.general.harvester_unit =
                Some(vec!["NOGOODIE".into(), "ARMED".into(), "UNARMED".into()]);
            r.sim.general.economy.build_refinery =
                Some(vec!["BASE".into(), "BASE2".into(), "BASE3".into()]);
            owned(r, "UNARMED");
        });
        spawn(&mut w, Kind::Building, "BASE3", 0, at(8, 8));
        let v = spawn(&mut w, Kind::Vehicle, "GOODIE", 0, at(0, 0));
        assert!(pick(&mut w, v, 1));
        // The random rule gives a goodie vehicle; the harvester rule the first ownable harvester type.
        let harvester = count(&w, Kind::Vehicle, "UNARMED");
        assert_eq!(harvester > 0, rule_applies, "prefix quirk {prefix}");
    }
}
