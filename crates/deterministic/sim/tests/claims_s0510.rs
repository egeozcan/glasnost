//! Claim tests of S-0510 (spies and infiltration) on synthetic rules:
//! invented types and numbers, no game data.

mod structures_common;

use glasnost_rules::schema::Ruleset;
use glasnost_sim::combat::effects::{Effect, InfiltrationRow};
use glasnost_sim::econ::{TypeRef, power, shroud};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::structures::spy;
use glasnost_sim::superweapons::charge;
use glasnost_sim::testkit;
use glasnost_sim::world::{ObjectId, Target, World};
use structures_common::*;

fn counted() -> World {
    game_on(&ruleset(), triggers())
}

fn with(f: impl FnOnce(&mut Ruleset)) -> World {
    let mut r = ruleset();
    f(&mut r);
    game_on(&r, triggers())
}

fn quirk(w: &mut World, f: impl FnOnce(&mut glasnost_rules::quirks::Quirks)) {
    let mut rules = (*w.rules_arc()).clone();
    f(&mut rules.quirks);
    w.rules = Some(std::sync::Arc::new(rules));
}

/// Puts a unit of `name` and `house` in the cell of building `b` with the
/// building as destination in `beh`.
fn at_building(
    w: &mut World,
    name: &str,
    house: u8,
    b: ObjectId,
    beh: Behaviour,
    dest: bool,
) -> ObjectId {
    let c = w.obj(b).unwrap().pos.cell();
    let u = testkit::spawn(w, Kind::Infantry, name, house, c, 0);
    let t = w.tick;
    let o = w.obj_mut(u).unwrap();
    if dest {
        o.dest = Some(Target::Object(b));
    } else {
        o.target = Some(Target::Object(b));
    }
    o.beh.force(beh, t, true);
    u
}

/// A spy of house 0 infiltrating a building of `name` owned by `owner`;
/// returns the building and the effects the entry produced.
fn infiltrate(w: &mut World, name: &str, owner: u8) -> (ObjectId, Vec<Effect>) {
    let b = put(w, Kind::Building, name, owner, 0, 0);
    w.effects.clear();
    let s = at_building(w, "SPYX", 0, b, Behaviour::Capture, true);
    assert!(
        glasnost_sim::structures::entry::arrival(w, s),
        "the spy is used up"
    );
    assert!(w.obj(s).is_none());
    (b, std::mem::take(&mut w.effects))
}

fn rows_of(effects: &[Effect]) -> Vec<InfiltrationRow> {
    effects
        .iter()
        .find_map(|e| match e {
            Effect::Infiltrated { rows, .. } => Some(rows.clone()),
            _ => None,
        })
        .unwrap_or_default()
}

// spec: S-0510 claim 1 (a spy enters in capture, area guard or patrol when its destination or target building stands in its cell; nothing else is tested at entry: not the spyable flag, not alliance)
#[test]
fn claim_1_when_a_spy_enters() {
    for (beh, dest, enters) in [
        (Behaviour::Capture, true, true),
        (Behaviour::AreaGuard, false, true),
        (Behaviour::Patrol, true, true),
        (Behaviour::Guard, true, false),
    ] {
        let mut w = counted();
        let b = put(&mut w, Kind::Building, "BANK", 1, 0, 0);
        let s = at_building(&mut w, "SPYX", 0, b, beh, dest);
        assert_eq!(
            glasnost_sim::structures::entry::arrival(&mut w, s),
            enters,
            "{beh:?}"
        );
    }
    // An allied building: the full effect against the ally (claim 3).
    let mut w = counted();
    w.houses[0].allies = vec![A, B];
    w.houses[1].credits = 1000;
    let (_, fx) = infiltrate(&mut w, "BANK", 1);
    assert_eq!(rows_of(&fx), vec![InfiltrationRow::Cash]);
    assert_eq!(w.houses[1].credits, 500);
}

// spec: S-0510 claim 1 (a spy type that also has the engineer flag is handled as an engineer and never infiltrates; an occupier or assaulter spy in capture makes the garrison attempt; a vehicle-like destination is taken over, whatever the type)
#[test]
fn claim_1_exclusions() {
    let mut w = counted();
    let b = put(&mut w, Kind::Building, "BANK", 1, 0, 0);
    w.houses[1].credits = 1000;
    let e = at_building(&mut w, "ENGSPY", 0, b, Behaviour::Capture, true);
    assert!(glasnost_sim::structures::entry::arrival(&mut w, e));
    assert_eq!(w.houses[1].credits, 1000, "an engineer: no theft");
    assert_eq!(
        owner(&w, b),
        Some(B),
        "BANK is not capturable: used up, nothing changes"
    );
    let mut r = ruleset();
    r.sim.infantry.get_mut("SPYX").unwrap().occupier = Some(true);
    let mut w = game_on(&r, triggers());
    let b = put(&mut w, Kind::Building, "BANK", 1, 0, 0);
    let s = at_building(&mut w, "SPYX", 0, b, Behaviour::Capture, true);
    assert!(
        !glasnost_sim::structures::entry::arrival(&mut w, s),
        "refused by S-0508, not infiltrating"
    );
    let v = put(&mut w, Kind::Building, "VLIKE", 1, 4, 0);
    let s = at_building(&mut w, "SPYX", 0, v, Behaviour::AreaGuard, true);
    assert!(glasnost_sim::structures::entry::arrival(&mut w, s));
    assert_eq!(
        owner(&w, v),
        Some(A),
        "taken over: the spy flag plays no part"
    );
}

// spec: S-0510 claim 2 (every entry removes the spy, whatever the building and whether or not an effect results: no kill, loss or score, no experience; its own tag gets the destroyed raise) and claim 9 (the building's entry raise comes first, then the effect, then the spy's raise)
#[test]
fn claim_2_and_9_the_spy_is_used_up_and_the_raises() {
    let mut w = counted();
    let b = put(&mut w, Kind::Building, "BANK", 1, 0, 0);
    w.houses[1].credits = 800;
    tag(&mut w, b, "ENT");
    let before: Vec<_> = w
        .houses
        .iter()
        .map(|h| (h.combat.kills, h.combat.losses, h.combat.score))
        .collect();
    let s = at_building(&mut w, "SPYX", 0, b, Behaviour::Capture, true);
    tag(&mut w, s, "DIE");
    let f = w.triggers.fired;
    assert!(glasnost_sim::structures::entry::arrival(&mut w, s));
    assert_eq!(
        w.triggers.fired - f,
        2,
        "the entry raise and the spy's destroyed raise"
    );
    assert!(w.triggers.variables[0] && w.triggers.variables[1]);
    assert!(w.obj(s).is_none());
    let after: Vec<_> = w
        .houses
        .iter()
        .map(|h| (h.combat.kills, h.combat.losses, h.combat.score))
        .collect();
    assert_eq!(before, after);
    // A building of no row and the own house's building: still used up, no effect.
    for (name, owner_house) in [("PLAIN", 1u8), ("BANK", 0u8)] {
        let mut w = counted();
        w.houses[0].credits = 100;
        w.houses[1].credits = 100;
        let b = put(&mut w, Kind::Building, name, owner_house, 0, 0);
        let s = at_building(&mut w, "SPYX", 0, b, Behaviour::Capture, true);
        assert!(glasnost_sim::structures::entry::arrival(&mut w, s));
        assert_eq!((w.houses[0].credits, w.houses[1].credits), (100, 100));
    }
}

// spec: S-0510 claim 3 (one effect, chosen by the building's type: E1 radar, E2 power, E3 tech-lab list, E4 first superweapon, E5 storage, E6 vehicle factory, E7 infantry factory; aircraft factories and others: nothing)
#[test]
fn claim_3_each_row() {
    for (name, row) in [
        ("RADARB", Some(InfiltrationRow::Radar)),
        ("POWERB", Some(InfiltrationRow::Power)),
        ("LABB", Some(InfiltrationRow::Tech)),
        ("SWB", Some(InfiltrationRow::Superweapon)),
        ("BANK", Some(InfiltrationRow::Cash)),
        ("WARF", Some(InfiltrationRow::VehicleFactory)),
        ("NAVYB", Some(InfiltrationRow::VehicleFactory)),
        ("BARR", Some(InfiltrationRow::InfantryFactory)),
        ("AIRF", None),
        ("PLAIN", None),
    ] {
        let mut w = counted();
        let b = put(&mut w, Kind::Building, name, 1, 0, 0);
        assert_eq!(
            spy::rows(&w, b),
            row.into_iter().collect::<Vec<_>>(),
            "{name}"
        );
    }
}

// spec: S-0510 claim 3 and Edge cases (quirks.infiltration_single_effect on: a building matching several rows gives only the first: a radar building with output re-covers and causes no blackout, a power plant on the tech-lab list blacks out and gives no stolen tech, a factory with storage is robbed and its flag is not set; off: every matching row applies in table order and E3 requires only membership)
#[test]
fn claim_3_precedence_and_the_single_effect_switch() {
    let mut w = counted();
    let rlab = put(&mut w, Kind::Building, "RLAB", 1, 0, 0);
    let plab = put(&mut w, Kind::Building, "PLAB", 1, 4, 0);
    let bankf = put(&mut w, Kind::Building, "BANKF", 1, 8, 0);
    assert_eq!(spy::rows(&w, rlab), vec![InfiltrationRow::Radar]);
    assert_eq!(spy::rows(&w, plab), vec![InfiltrationRow::Power]);
    assert_eq!(spy::rows(&w, bankf), vec![InfiltrationRow::Cash]);
    quirk(&mut w, |q| q.infiltration_single_effect = false);
    assert_eq!(
        spy::rows(&w, rlab),
        vec![
            InfiltrationRow::Radar,
            InfiltrationRow::Power,
            InfiltrationRow::Tech
        ],
        "E3 requires only list membership"
    );
    assert_eq!(
        spy::rows(&w, plab),
        vec![InfiltrationRow::Power, InfiltrationRow::Tech]
    );
    assert_eq!(
        spy::rows(&w, bankf),
        vec![InfiltrationRow::Cash, InfiltrationRow::VehicleFactory]
    );
    // Every row applies in the same update.
    let (_, fx) = infiltrate(&mut w, "BANKF", 1);
    assert_eq!(rows_of(&fx).len(), 2);
    assert!(w.houses[0].infiltrated_vehicle_factory);
}

// spec: S-0510 claim 3 (the infiltrating house is the victim house: no effect; only identities are compared, so a spy at an allied building has the full effect)
#[test]
fn claim_3_own_house_no_effect() {
    let mut w = counted();
    w.houses[0].credits = 10;
    let (_, fx) = infiltrate(&mut w, "BANK", 0);
    assert!(rows_of(&fx).is_empty());
    assert_eq!(w.houses[0].credits, 10);
    assert!(
        !fx.iter().any(|e| matches!(e, Effect::Infiltrated { .. })),
        "no event for the same house"
    );
}

// spec: S-0510 claim 3 row E1 and claim 4 (the victim's whole map is re-covered at once, an area it had explored becomes unexplored; the infiltrator's map is untouched; an active spy satellite stops it, S-0409 claim 15)
#[test]
fn claim_3_row_e1_recover() {
    let mut w = counted();
    let far = at(8, 8);
    for h in [A, B] {
        shroud::area_reveal(&mut w, h, glasnost_sim::world::Pos::centre(far, 0), 2);
        assert!(shroud::explored(&w, h, far));
    }
    infiltrate(&mut w, "RADARB", 1);
    assert!(
        !shroud::explored(&w, B, far),
        "the victim's map is covered again"
    );
    assert!(shroud::explored(&w, A, far), "the infiltrator's is not");
}

// spec: S-0510 claim 3 row E2 and claim 4 (a timed blackout of the victim of the game-wide spy blackout length, replacing a running one)
#[test]
fn claim_3_row_e2_blackout() {
    let mut w = counted();
    assert!(!power::blackout_running(&w, B));
    infiltrate(&mut w, "POWERB", 1);
    assert!(power::blackout_running(&w, B));
    assert!(!power::blackout_running(&w, A));
    testkit::run(&mut w, 50);
    assert!(
        power::blackout_running(&w, B),
        "still running 50 ticks in (length 100)"
    );
    testkit::run(&mut w, 120);
    assert!(!power::blackout_running(&w, B), "over after its length");
}

// spec: S-0510 claim 3 row E3 (the infiltrating house gains the stolen-tech flag the type's base-planning side chooses; the victim loses nothing)
#[test]
fn claim_3_row_e3_stolen_tech() {
    let mut w = counted();
    assert!(!w.houses[0].econ.tech.stolen[1]);
    infiltrate(&mut w, "LABB", 1);
    assert!(w.houses[0].econ.tech.stolen[1], "side 1: the second flag");
    assert!(!w.houses[1].econ.tech.stolen[1]);
}

// spec: S-0510 claim 3 row E4 and claim 4 (the victim's record for the building type's first superweapon is reset, a running clock keeps running)
#[test]
fn claim_3_row_e4_superweapon_reset() {
    let mut w = counted();
    let sw = w
        .rules()
        .superweapons
        .types
        .iter()
        .position(|t| t.name == "CURT")
        .unwrap() as u32;
    let giver = put(&mut w, Kind::Building, "SWB", 1, 8, 0);
    let _ = giver;
    testkit::run(&mut w, 200);
    assert!(
        charge::get(&w, B, sw).is_some_and(|r| r.granted),
        "granted to the victim"
    );
    let before = charge::get(&w, B, sw).unwrap();
    assert!(
        before.ready,
        "charged and ready after 200 ticks of a 90-tick recharge"
    );
    let t = w.tick;
    infiltrate(&mut w, "SWB", 1);
    let after = charge::get(&w, B, sw).unwrap();
    let full = w.rules().superweapons.types[sw as usize].charge;
    assert!(!after.ready, "no longer ready");
    assert_eq!(after.remain, full, "the full charge time remains");
    assert!(
        after.running && after.start == t,
        "a running clock restarts from that tick"
    );
}

// spec: S-0510 claim 5 (the infiltrated-factory flags belong to the infiltrator's house, stay set after the building is destroyed, sold or captured; setting a set flag changes nothing)
#[test]
fn claim_5_factory_flags() {
    let mut w = counted();
    assert!(!w.houses[0].infiltrated_vehicle_factory && !w.houses[0].infiltrated_infantry_factory);
    let (b, _) = infiltrate(&mut w, "WARF", 1);
    assert!(w.houses[0].infiltrated_vehicle_factory);
    assert!(!w.houses[0].infiltrated_infantry_factory);
    assert!(!w.houses[1].infiltrated_vehicle_factory, "not the victim's");
    glasnost_sim::objects::remove(&mut w, b);
    testkit::run(&mut w, 5);
    assert!(w.houses[0].infiltrated_vehicle_factory, "stays set");
    infiltrate(&mut w, "BARR", 1);
    assert!(w.houses[0].infiltrated_infantry_factory);
    infiltrate(&mut w, "BARR", 1);
    assert!(w.houses[0].infiltrated_infantry_factory);
}

// spec: S-0510 claim 5 and S-0311 claim 11 (a unit its own house creates after the infiltration is a veteran when its type is trainable: an infantry unit after a barracks, a vehicle that is not naval and a building that undeploys into a vehicle after a war factory; aircraft never; a type not trainable never)
#[test]
fn claim_5_veterans_from_infiltrated_factories() {
    let mut r = ruleset();
    for n in ["WALKER"] {
        r.sim.infantry.get_mut(n).unwrap().combat.trainable = Some(true);
    }
    r.sim.vehicles.get_mut("CART").unwrap().combat.trainable = Some(true);
    r.sim.buildings.get_mut("VLIKE").unwrap().combat.trainable = Some(true);
    r.sim.vehicles.get_mut("CRUSHER").unwrap().combat.trainable = Some(false);
    r.sim.buildings.get_mut("HUT").unwrap().combat.trainable = Some(false);
    let mut w = game_on(&r, triggers());
    let exp = |w: &World, id: ObjectId| w.obj(id).unwrap().combat.experience.to_f64();
    let rank1 =
        |w: &World, id: ObjectId| exp(w, id).ge(glasnost_num::F64::from_i64(1, w.rules().mode));
    let make = |w: &mut World, kind, name: &str| {
        let ti = w.rules().index_of(kind, name).unwrap();
        glasnost_sim::objects::create(w, kind, ti, Some(A))
    };
    let before = make(&mut w, Kind::Infantry, "WALKER");
    assert!(!rank1(&w, before));
    infiltrate(&mut w, "BARR", 1);
    let inf = make(&mut w, Kind::Infantry, "WALKER");
    let veh = make(&mut w, Kind::Vehicle, "CART");
    assert!(rank1(&w, inf), "infantry after a barracks");
    assert!(!rank1(&w, veh), "a vehicle needs the war factory");
    infiltrate(&mut w, "WARF", 1);
    let veh = make(&mut w, Kind::Vehicle, "CART");
    let building = make(&mut w, Kind::Building, "VLIKE");
    let other = make(&mut w, Kind::Building, "HUT");
    assert!(rank1(&w, veh), "a trainable vehicle after a war factory");
    assert!(
        rank1(&w, building),
        "a building that undeploys into a vehicle"
    );
    assert!(!rank1(&w, other));
    // Not trainable: never.
    let crusher = make(&mut w, Kind::Vehicle, "CRUSHER");
    assert!(!rank1(&w, crusher));
}

// spec: S-0510 claim 6 and F1 (money theft: s = trunc(B × f), the thief gains s, the victim pays it; 0 ≤ f ≤ 1 makes the victim lose exactly s; the example 10001 × 0.5 = 5000)
#[test]
fn claim_6_theft_amount_and_balances() {
    let mut w = counted();
    w.houses[1].credits = 10_001;
    w.houses[0].credits = 7;
    infiltrate(&mut w, "BANK", 1);
    assert_eq!(w.houses[0].credits, 7 + 5000);
    assert_eq!(w.houses[1].credits, 10_001 - 5000);
    // A repeat entry takes half of what is left (claim 7).
    infiltrate(&mut w, "BANK", 1);
    assert_eq!(w.houses[1].credits, 5001 - 2500);
}

// spec: S-0510 F1 and Import mapping (SpyMoneyStealPercent kept in single precision: 57% is the single just below 0.57, so 100 gives 56; 33% with 1000 gives 329, 330 with quirks.chop_rounding off; absent: 0.5)
#[test]
fn f1_the_percentage_in_single_precision() {
    for (text, balance, want) in [("57%", 100, 56), ("33%", 1000, 329)] {
        let w = with(|r| {
            r.sim.general.economy.spy_money_steal_percent =
                Some(glasnost_rules::sim::Decimal(text.into()));
        });
        assert_eq!(spy::theft_amount(&w, balance), want, "{text}");
    }
    // With rounding to nearest the product narrows to the single above 0.33: 330.
    let w = with(|r| {
        r.globals.quirks.chop_rounding = false;
        r.sim.general.economy.spy_money_steal_percent =
            Some(glasnost_rules::sim::Decimal("33%".into()));
    });
    assert_eq!(spy::theft_amount(&w, 1000), 330);
    // Absent: 0.5.
    let w = counted();
    assert_eq!(spy::theft_amount(&w, 10_001), 5000);
}

// spec: S-0510 claim 6 and Edge cases (f above 1: the victim's balance is emptied while the thief gains the full s; f below 0: the money flows the other way; quirks.spy_theft_unbacked_gain off: s is clamped to 0 ≤ s ≤ max(0, the victim's balance))
#[test]
fn claim_6_out_of_range_fractions_and_the_quirk() {
    let set = |f: &str| {
        let f = f.to_owned();
        with(move |r| {
            r.sim.general.economy.spy_money_steal_percent = Some(glasnost_rules::sim::Decimal(f));
        })
    };
    // f = 2: s = 2B, the victim pays only its balance.
    let mut w = set("2");
    w.houses[1].credits = 100;
    w.houses[0].credits = 0;
    infiltrate(&mut w, "BANK", 1);
    assert_eq!((w.houses[0].credits, w.houses[1].credits), (200, 0));
    // The switch off: the thief gains no more than the victim pays.
    let mut w = set("2");
    quirk(&mut w, |q| q.spy_theft_unbacked_gain = false);
    w.houses[1].credits = 100;
    w.houses[0].credits = 0;
    infiltrate(&mut w, "BANK", 1);
    assert_eq!((w.houses[0].credits, w.houses[1].credits), (100, 0));
    // f = −0.5: the money flows the other way and the thief can go below 0.
    let mut w = set("-0.5");
    w.houses[1].credits = 100;
    w.houses[0].credits = 10;
    infiltrate(&mut w, "BANK", 1);
    assert_eq!((w.houses[0].credits, w.houses[1].credits), (-40, 150));
    let mut w = set("-0.5");
    quirk(&mut w, |q| q.spy_theft_unbacked_gain = false);
    w.houses[1].credits = 100;
    w.houses[0].credits = 10;
    infiltrate(&mut w, "BANK", 1);
    assert_eq!(
        (w.houses[0].credits, w.houses[1].credits),
        (10, 100),
        "no negative theft"
    );
}

// spec: S-0510 claim 7 (each later entry applies its row again; two spies entering in the same tick steal serially from the balance the first left)
#[test]
fn claim_7_same_tick_thefts_are_serial() {
    let mut w = counted();
    w.houses[1].credits = 1000;
    let b = put(&mut w, Kind::Building, "BANK", 1, 0, 0);
    let s1 = at_building(&mut w, "SPYX", 0, b, Behaviour::Capture, true);
    let s2 = at_building(&mut w, "SPYX", 0, b, Behaviour::Capture, true);
    assert!(glasnost_sim::structures::entry::arrival(&mut w, s1));
    assert!(glasnost_sim::structures::entry::arrival(&mut w, s2));
    assert_eq!(w.houses[1].credits, 250, "1000 → 500 → 250");
    assert_eq!(w.houses[0].credits, 750 + 0, "500 and then 250");
}

// spec: S-0510 claim 8 (an infiltration leaves no mark on the building and shows nothing about it: no state is recorded; the building's discovery or its production is untouched)
#[test]
fn claim_8_nothing_is_revealed() {
    let mut w = counted();
    let b = put(&mut w, Kind::Building, "WARF", 1, 0, 0);
    let discovered = shroud::discovered(&w, A, b);
    let before = w.obj(b).unwrap().clone();
    let s = at_building(&mut w, "SPYX", 0, b, Behaviour::Capture, true);
    assert!(glasnost_sim::structures::entry::arrival(&mut w, s));
    assert_eq!(shroud::discovered(&w, A, b), discovered);
    let after = w.obj(b).unwrap();
    assert_eq!(after.econ.sight, before.econ.sight, "no marks");
    assert_eq!(after.health, before.health);
    assert_eq!(after.owner, before.owner);
}

// spec: S-0510 claim 10 (an infantry unit that is neither a spy nor an engineer and meets the entry conditions has no effect: destination cleared, a forced move-anywhere scatter from the building's centre; this covers a type with only the infiltrate flag)
#[test]
fn claim_10_other_entrants() {
    let mut r = ruleset();
    r.sim.infantry.get_mut("WALKER").unwrap().infiltrate = Some(true);
    let mut w = game_on(&r, triggers());
    w.houses[1].credits = 1000;
    let b = put(&mut w, Kind::Building, "BANK", 1, 0, 0);
    tag_it(&mut w, b);
    let f = w.triggers.fired;
    let u = at_building(&mut w, "WALKER", 0, b, Behaviour::Capture, true);
    assert!(!glasnost_sim::structures::entry::arrival(&mut w, u));
    assert!(on_map(&w, u));
    assert_eq!(w.houses[1].credits, 1000, "no effect");
    assert_eq!(
        w.triggers.fired - f,
        1,
        "the entry raise happens for the other entrants too"
    );
    assert!(
        matches!(w.obj(u).unwrap().dest, Some(Target::Cell(_))),
        "scattering"
    );
}

fn tag_it(w: &mut World, b: ObjectId) {
    tag(w, b, "ENT");
}

// spec: S-0510 claim 11 (the infiltrated radar event and the announcement lines: an event for every row, with the rows that applied and the satellite state, for the audio crate; none when the houses are the same)
#[test]
fn claim_11_the_presentation_event() {
    let mut w = counted();
    let (b, fx) = infiltrate(&mut w, "POWERB", 1);
    let ev: Vec<_> = fx
        .iter()
        .filter_map(|e| match e {
            Effect::Infiltrated {
                building,
                infiltrator,
                victim,
                rows,
                satellite,
                ..
            } => Some((*building, *infiltrator, *victim, rows.clone(), *satellite)),
            _ => None,
        })
        .collect();
    assert_eq!(
        ev,
        vec![(b, Some(A), Some(B), vec![InfiltrationRow::Power], false)]
    );
    // A building of no row: the event is still made (the radar event), with no rows.
    let (_, fx) = infiltrate(&mut w, "PLAIN", 1);
    assert_eq!(rows_of(&fx), vec![]);
    assert!(fx.iter().any(|e| matches!(e, Effect::Infiltrated { .. })));
}

// spec: S-0007 claim 25 (the sidebar's alternate icon: the type is trainable, and the house has infiltrated a barracks for an infantry type or a war factory for a vehicle type or a building type that undeploys into a vehicle; aircraft and other buildings never)
#[test]
fn s0007_claim_25_alternate_icon_choice() {
    let mut r = ruleset();
    r.sim.infantry.get_mut("WALKER").unwrap().combat.trainable = Some(true);
    r.sim.vehicles.get_mut("CART").unwrap().combat.trainable = Some(true);
    r.sim.buildings.get_mut("VLIKE").unwrap().combat.trainable = Some(true);
    r.sim.buildings.get_mut("HUT").unwrap().combat.trainable = Some(true);
    r.sim.vehicles.get_mut("CRUSHER").unwrap().combat.trainable = Some(false);
    r.sim.buildings.get_mut("HUT").unwrap().combat.trainable = Some(false);
    let mut w = game_on(&r, triggers());
    let t = |w: &World, kind, name: &str| TypeRef {
        kind,
        index: w.rules().index_of(kind, name).unwrap(),
    };
    let (inf, veh, vl, hut, crusher) = (
        t(&w, Kind::Infantry, "WALKER"),
        t(&w, Kind::Vehicle, "CART"),
        t(&w, Kind::Building, "VLIKE"),
        t(&w, Kind::Building, "HUT"),
        t(&w, Kind::Vehicle, "CRUSHER"),
    );
    let all = |w: &World| [inf, veh, vl, hut, crusher].map(|t| spy::alternate_icon(w, A, t));
    assert_eq!(all(&w), [false; 5]);
    infiltrate(&mut w, "BARR", 1);
    assert_eq!(all(&w), [true, false, false, false, false]);
    infiltrate(&mut w, "WARF", 1);
    assert_eq!(
        all(&w),
        [true, true, true, false, false],
        "the untrainable crusher and the plain building keep the main icon"
    );
    assert!(
        !spy::alternate_icon(&w, B, inf),
        "the victim's house is unchanged"
    );
}

// spec: S-0510 claim 3 row E1 and Edge cases (a victim with an active spy satellite loses nothing, the spy is used up, and neither side hears the radar lines)
#[test]
fn claim_3_e1_with_a_spy_satellite_is_a_no_op_with_no_radar_lines() {
    let mut w = counted();
    let (_, fx) = infiltrate(&mut w, "RADARB", 1);
    let sat = fx.iter().find_map(|e| match e {
        Effect::Infiltrated { satellite, .. } => Some(*satellite),
        _ => None,
    });
    assert_eq!(sat, Some(false), "no satellite here: the lines would play");
}
