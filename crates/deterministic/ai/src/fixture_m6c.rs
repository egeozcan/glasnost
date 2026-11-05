//! The synthetic M6c skirmish for the determinism gate (M6c step 5): the
//! fixture's ruleset and map with invented types and pre-placed objects, so
//! that a plain AI-versus-AI game uses every special ability the own AI has
//! (garrisons, engineers, spies, tech buildings, jumpjets, deployers,
//! radiation, chrono erasure, linked weapons, terror and strikes, crates).
//! Every name and number here is test data of our own, not game behaviour;
//! the abilities' behaviour comes from the specs the sim crate cites, and
//! this file only supplies the types and the places.
//!
//! - Infantry (each house starts with them on the map, in rows south of its
//!   start; the barracks builds the armed ones too): GARRI (occupier), ENGI
//!   (engineer), SPYU (spy), JETI and JET2 (armed jumpjet units, built
//!   only), DEPI (deploys to fire), RADI (the same, immune to radiation,
//!   with a radiation weapon in its second slot), ERAS (a temporal beam),
//!   ZAPR (an electric charger), PLNT (plants bombs), DISA (disarms them),
//!   KAMI (a suicide weapon), SABO (explosive charges); DECOY, an unarmed
//!   unit of the *other* house on an islet, of an armour class that only
//!   the jumpjets' own warhead (JETW, on their weapon JetRifle) affects
//!   (M8 step B4: before, the strike aircraft killed it early, or a
//!   radiation deployer deployed among its own army for it and irradiated
//!   that army; seed 5 of 1 to 32 then built no jumpjet).
//! - Vehicles: DEPT (deploys in place), FIRE (deploys to fire), DESIG (an
//!   air-strike designator), DISV (a disarmer that deploys into a
//!   building); STRK, the strike aircraft; HULK, an unarmed, sturdy, costly
//!   vehicle of the *other* house near each start whose armour only the
//!   temporal, bomb and disarm warheads touch, so that the erasers and bomb
//!   planters have a target from the start, the house that owns it a bomb
//!   to disarm, and the armies leave it alone.
//! - Buildings: PRSM (the prism type, also overpowerable, built by the AI),
//!   HUT (a civilian garrison building), DERR (a tech building that pays,
//!   held by the neutral house), SPYB (a spyable neutral power building);
//!   the fixture's POWR, PILL, FACT and PROC are spyable.
//! - Map: [`fixture::map`]'s flat ground with the starts at ±[`START_DX`], an
//!   islet of three by three land cells ringed by water two cells wide
//!   north of each start (a walking unit cannot reach it), holding the other
//!   house's DECOY, the other house's two HULKs toward the middle, and the
//!   neutral house's two HUTs, two DERRs and two SPYBs near each start.
//!
//! The counts of each start unit and the HULKs were chosen (M7a step 2b)
//! so that every counter of the gate's M6c duel has a margin over a seed
//! sweep (`m6c_seed_sweep` in `crates/sim-gate/tests/m6c.rs`); M8 step B4
//! added the decoy and the prism's support reach of 24 cells
//! after a sweep of seeds 1 to 32 with each reached quirk switch flipped
//! (docs/development.md, "Own AI (M8 step B4: gate margins)").
//! - Crates: the setting is on, with the power-up table of money only.

use std::sync::Arc;

use glasnost_map::objects::{Infantry, Placed, Structure, Unit};
use glasnost_map::{CellXY, Map};
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{
    BuildingType, Country, Decimal, OverlayType, SequenceEntry, UnitType, WarheadType, WeaponType,
};
use glasnost_rules::sim_econ::Powerup;
use glasnost_sim::econ::setup::{self, Player, Settings};
use glasnost_sim::rules::SimRules;
use glasnost_sim::world::GameSetup;

use crate::fixture;

/// The start offset from the middle along x (closer than the fixture's, so
/// that the armies meet earlier).
pub const START_DX: i32 = 16;

/// The name of the scenario's map.
pub const MAP_NAME: &str = "synthetic-m6c";

fn d(s: &str) -> Decimal {
    Decimal(s.into())
}

fn register(r: &mut Ruleset, reg: &str, name: &str) {
    let l = r.registries.entry(reg.into()).or_default();
    if !l.iter().any(|n| n == name) {
        l.push(name.into());
    }
}

fn weapon(damage: i32, range: &str, warhead: &str) -> WeaponType {
    WeaponType {
        damage: Some(damage),
        rof: Some(40),
        range: Some(d(range)),
        speed: Some(100),
        projectile: Some("Bullet".into()),
        warhead: Some(warhead.into()),
        ..Default::default()
    }
}

fn warhead(f: impl FnOnce(&mut WarheadType)) -> WarheadType {
    let mut h = WarheadType {
        verses: Some(vec![d("1"); 11]),
        ..Default::default()
    };
    f(&mut h);
    h
}

/// An infantry type derived from the fixture's rifleman: not buildable.
fn infantry(r: &mut Ruleset, name: &str, f: impl FnOnce(&mut UnitType)) {
    let mut t = r.sim.infantry["WALKER"].clone();
    t.tech_level = Some(99);
    t.owner = None;
    t.cost = Some(100);
    f(&mut t);
    register(r, "infantry", name);
    r.sim.infantry.insert(name.into(), t);
}

fn sequences(t: &mut UnitType) {
    for (k, n) in [
        ("deploy", 4),
        ("deployed", 1),
        ("undeploy", 4),
        ("deployed_fire", 2),
    ] {
        t.sequences.insert(
            k.into(),
            SequenceEntry {
                start: 0,
                count: n,
                multiplier: 0,
                direction: None,
            },
        );
    }
}

/// The scenario's ruleset.
pub fn ruleset() -> Ruleset {
    let mut r = fixture::ruleset();
    // The neutral country the tech and garrison buildings belong to.
    register(&mut r, "countries", "Neutral");
    let mut neutral = Country {
        multiplay_passive: Some(true),
        ..Default::default()
    };
    // The civilian side (S-0414 claim 12 and Import mapping): its house is
    // the civilian house, so a HUT a player garrisons becomes the player's
    // (S-0508 claim 7) and returns to it when emptied (claim 8). Without it
    // a garrisoned HUT stayed the neutral house's (claim 24) and, once a
    // garrisoned building searched (S-0309 claim 2, M8 step A4 round 8),
    // fired at both players, its occupants' own house too.
    neutral.econ.side = Some(glasnost_sim::econ::setup::CIVILIAN_SIDE.into());
    r.sim.countries.insert("Neutral".into(), neutral);

    // Spyable buildings of the houses: the power plant, the barracks and
    // the war factory have an infiltration effect each.
    for n in ["POWR", "PILL", "FACT", "PROC"] {
        if let Some(b) = r.sim.buildings.get_mut(n) {
            b.spyable = Some(true);
        }
    }
    // The barracks has a door kind, as every real one has: only such a
    // building gives a released jumpjet unit its ascent report (S-0417
    // claim 15).
    if let Some(b) = r.sim.buildings.get_mut("PILL") {
        b.econ.door_first = Some(true);
    }
    // The refinery stores money, so a spy can steal it (the first base
    // building an army sees).
    if let Some(b) = r.sim.buildings.get_mut("PROC") {
        b.storage = Some(1000);
    }

    // Garrison building of the neutral house (civilian tech level).
    let mut hut = BuildingType {
        strength: Some(400),
        foundation: Some([2, 2]),
        can_be_occupied: Some(true),
        can_occupy_fire: Some(true),
        max_number_occupants: Some(3),
        tech_level: Some(-1),
        ..Default::default()
    };
    hut.econ.sight = Some(3);
    register(&mut r, "buildings", "HUT");
    r.sim.buildings.insert("HUT".into(), hut);
    // A spyable building of the neutral house that produces power.
    let mut spyb = BuildingType {
        strength: Some(400),
        foundation: Some([2, 2]),
        spyable: Some(true),
        tech_level: Some(99),
        ..Default::default()
    };
    spyb.econ.power = Some(100);
    register(&mut r, "buildings", "SPYB");
    r.sim.buildings.insert("SPYB".into(), spyb);
    // Tech building that pays (acquisition bonus and income).
    register(&mut r, "buildings", "DERR");
    r.sim.buildings.insert(
        "DERR".into(),
        BuildingType {
            strength: Some(400),
            foundation: Some([1, 1]),
            capturable: Some(true),
            needs_engineer: Some(true),
            tech_level: Some(99),
            produce_cash_startup: Some(300),
            produce_cash_amount: Some(25),
            produce_cash_delay: Some(21),
            ..Default::default()
        },
    );

    // Infantry.
    infantry(&mut r, "GARRI", |t| t.occupier = Some(true));
    infantry(&mut r, "ENGI", |t| {
        t.engineer = Some(true);
        t.primary = None;
    });
    infantry(&mut r, "SPYU", |t| {
        // Slower than the rest, so that the crates (sent to the fastest
        // idle unit) leave it where it is.
        t.speed = Some(6);
        t.agent = Some(true);
        t.infiltrate = Some(true);
        t.primary = None;
    });
    infantry(&mut r, "JETI", |t| {
        t.locomotion_class = Some(glasnost_rules::sim::LocomotionClass::Foot);
        t.movement_zone = Some(glasnost_rules::sim::MovementZone::Fly);
        t.locomotor = Some(glasnost_rules::sim::Locomotor::Jumpjet);
        t.jumpjet = Some(true);
        t.jumpjet_height = Some(300);
    });
    // A second jumpjet type, so that the army builder (fewest on the map
    // first) builds another one while the first still flies.
    let jet2 = r.sim.infantry["JETI"].clone();
    register(&mut r, "infantry", "JET2");
    r.sim.infantry.insert("JET2".into(), jet2);
    infantry(&mut r, "DEPI", |t| {
        t.deployer = Some(true);
        t.deploy_fire = Some(true);
        t.undeploy_delay = Some(-1);
        sequences(t);
    });

    // Radiation: the general settings, the weapon and the immune deployer.
    let g = &mut r.sim.general;
    g.rad_duration_multiple = Some(2);
    g.rad_application_delay = Some(15);
    g.rad_level_max = Some(10_000);
    g.rad_level_delay = Some(60);
    g.rad_light_delay = Some(60);
    g.rad_level_factor = Some(d("0.0625"));
    g.rad_light_factor = Some(d("0.5"));
    g.rad_tint_factor = Some(d("1.0"));
    g.rad_color = Some([0, 255, 0]);
    g.rad_site_warhead = Some("RADSITE".into());
    r.sim.warheads.insert("RADSITE".into(), warhead(|_| {}));
    r.sim.warheads.insert(
        "RADW".into(),
        warhead(|h| {
            h.cell_spread = Some(d("2"));
            h.verses = Some(vec![d("0"); 11]);
        }),
    );
    let mut radg = weapon(0, "5", "RADW");
    radg.rof = Some(30);
    radg.rad_level = Some(900);
    radg.omni_fire = Some(true);
    r.sim.weapons.insert("RADG".into(), radg);
    infantry(&mut r, "RADI", |t| {
        t.deployer = Some(true);
        t.deploy_fire = Some(true);
        t.undeploy_delay = Some(-1);
        t.immune_to_radiation = Some(true);
        t.secondary = Some("RADG".into());
        sequences(t);
    });

    // Chrono erasure: a temporal warhead and a beam.
    r.sim
        .warheads
        .insert("TICK".into(), warhead(|h| h.temporal = Some(true)));
    r.sim.weapons.insert("TIME".into(), weapon(30, "2", "TICK"));
    infantry(&mut r, "ERAS", |t| t.primary = Some("TIME".into()));

    // Linked weapons: the prism type (also overpowerable, so that the
    // chargers have a building of the house to charge) and the charger.
    let mut prsm = fixture::building(2, 2, 500, -20);
    // The master's shot, the support reach (the secondary weapon's range)
    // and the delayed fire that the beam waits for. The reach spans a base
    // (M8 step B4: at 10 cells, prisms the brain placed 10 to 18 cells
    // apart never supported each other).
    r.sim
        .weapons
        .insert("PrismGun".into(), weapon(40, "18", "HE"));
    r.sim
        .weapons
        .insert("PrismReach".into(), weapon(1, "24", "HE"));
    prsm.primary = Some("PrismGun".into());
    prsm.secondary = Some("PrismReach".into());
    prsm.delayed_fire_delay = Some(5);
    prsm.overpowerable = Some(true);
    let g = &mut r.sim.general;
    g.prism_support_modifier = Some(50);
    g.prism_support_max = Some(3);
    g.prism_support_delay = Some(100);
    g.prism_support_duration = Some(15);
    register(&mut r, "buildings", "PRSM");
    r.sim.buildings.insert("PRSM".into(), prsm);
    r.sim.general.prism_type = Some("PRSM".into());
    r.sim
        .warheads
        .insert("ELEC".into(), warhead(|h| h.electric_assault = Some(true)));
    r.sim.weapons.insert("Zap".into(), weapon(0, "2", "ELEC"));
    infantry(&mut r, "ZAPR", |t| t.secondary = Some("Zap".into()));

    // Terror: bombs, disarming, suicide, charges, air strikes.
    r.sim
        .warheads
        .insert("IVAN".into(), warhead(|h| h.ivan_bomb = Some(true)));
    r.sim
        .warheads
        .insert("DISARM".into(), warhead(|h| h.bomb_disarm = Some(true)));
    r.sim
        .warheads
        .insert("BOOM".into(), warhead(|h| h.cell_spread = Some(d("1.5"))));
    r.sim.warheads.insert("DEMO".into(), warhead(|_| {}));
    r.sim
        .warheads
        .insert("STRIKE".into(), warhead(|h| h.airstrike = Some(true)));
    r.sim
        .weapons
        .insert("PlantW".into(), weapon(0, "2", "IVAN"));
    r.sim
        .weapons
        .insert("DisarmW".into(), weapon(0, "2", "DISARM"));
    r.sim.weapons.insert("Boom".into(), weapon(80, "5", "BOOM"));
    let mut kami = weapon(5, "2", "DEMO");
    kami.suicide = Some(true);
    r.sim.weapons.insert("KamiW".into(), kami);
    let mut sabo = weapon(1, "2", "DEMO");
    sabo.rof = Some(30);
    r.sim.weapons.insert("SaboGun".into(), sabo);
    r.sim
        .weapons
        .insert("Marker".into(), weapon(10, "12", "STRIKE"));
    infantry(&mut r, "PLNT", |t| {
        t.primary = Some("PlantW".into());
        t.ivan = Some(true);
    });
    // The disarmer is built by the barracks, so that some stand at home
    // (waiting for a wave) when a bomb is planted on an object of the house.
    infantry(&mut r, "DISA", |t| {
        t.primary = Some("DisarmW".into());
        // It sees the bombs planted within 30 cells of it.
        t.combat.bomb_sight = Some(30);
        t.tech_level = Some(1);
        t.owner = Some(vec!["Alphaland".into(), "Betaland".into()]);
    });
    infantry(&mut r, "KAMI", |t| {
        t.strength = Some(50);
        t.primary = Some("KamiW".into());
        t.combat.explodes = Some(true);
        t.combat.death_weapon = Some("Boom".into());
    });
    infantry(&mut r, "SABO", |t| {
        t.c4 = Some(true);
        t.secondary = Some("SaboGun".into());
    });
    let g = &mut r.sim.general;
    g.ivan_warhead = Some("BOOM".into());
    g.ivan_damage = Some(100);
    g.ivan_timed_delay = Some(900);
    g.ivan_icon_flicker_rate = Some(8);
    g.can_detonate_time_bomb = Some(true);
    g.c4_warhead = Some("DEMO".into());
    g.c4_delay = Some(d("0.0111"));
    // The strike aircraft and the designator.
    let mut strk = UnitType {
        strength: Some(200),
        speed: Some(30),
        rot: Some(12),
        locomotor: Some(glasnost_rules::sim::Locomotor::Fly),
        primary: Some("Missile".into()),
        ..Default::default()
    };
    strk.combat.ammo = Some(2);
    register(&mut r, "aircraft", "STRK");
    r.sim.aircraft.insert("STRK".into(), strk);

    // Vehicles.
    let gun = r.sim.vehicles["GUNR"].clone();
    let veh = |r: &mut Ruleset, n: &str, t: UnitType| {
        register(r, "vehicles", n);
        r.sim.vehicles.insert(n.into(), t);
    };
    let mut dept = gun.clone();
    dept.is_simple_deployer = Some(true);
    dept.tech_level = Some(99);
    dept.owner = None;
    veh(&mut r, "DEPT", dept);
    let mut fire = gun.clone();
    fire.deploy_fire = Some(true);
    fire.tech_level = Some(99);
    fire.owner = None;
    veh(&mut r, "FIRE", fire);
    let mut desig = gun;
    desig.primary = Some("Marker".into());
    desig.combat.airstrike_team = Some(2);
    desig.combat.elite_airstrike_team = Some(3);
    desig.combat.airstrike_team_type = Some("STRK".into());
    desig.combat.elite_airstrike_team_type = Some("STRK".into());
    desig.combat.airstrike_recharge_time = Some(300);
    desig.combat.elite_airstrike_recharge_time = Some(150);
    desig.tech_level = Some(99);
    desig.owner = None;
    veh(&mut r, "DESIG", desig);
    // A disarming vehicle that deploys into a building: the army builder
    // leaves such a type out of its waves (it deploys into a yard or
    // another building), so it stays home to disarm the bombs planted on
    // the house's objects.
    let mut disv = r.sim.vehicles["GUNR"].clone();
    disv.primary = Some("DisarmW".into());
    disv.combat.bomb_sight = Some(30);
    disv.deploys_into = Some("DISB".into());
    disv.tech_level = Some(99);
    disv.owner = None;
    veh(&mut r, "DISV", disv);
    register(&mut r, "buildings", "DISB");
    r.sim.buildings.insert(
        "DISB".into(),
        BuildingType {
            strength: Some(400),
            foundation: Some([1, 1]),
            tech_level: Some(99),
            ..Default::default()
        },
    );

    // The hulk: an unarmed, sturdy and costly vehicle of each house that
    // stands near the other's start (pre-placed buildings of a player's
    // house are dropped in a skirmish, S-0009 claim 27, units are kept).
    // Its own house has no use for it (unarmed, it is no army unit); the
    // other house sees it from the start.
    let mut hulk = r.sim.vehicles["GUNR"].clone();
    hulk.primary = None;
    hulk.secondary = None;
    hulk.strength = Some(2000);
    hulk.cost = Some(1000);
    hulk.speed = Some(1);
    hulk.tech_level = Some(99);
    hulk.owner = None;
    // Its armour class (the last) is one that only a temporal beam, a
    // bomb planter and a disarmer affect: every other warhead has a versus
    // value of zero there, so no other weapon may fire at it (S-0304
    // claim 1) and the armies leave it to the special units.
    hulk.combat.armor = Some(glasnost_rules::sim::ArmourClass::Special2);
    veh(&mut r, "HULK", hulk);
    for (n, h) in r.sim.warheads.iter_mut() {
        if ["TICK", "IVAN", "DISARM"].contains(&n.as_str()) {
            continue;
        }
        let v = h.verses.get_or_insert_with(|| vec![d("1"); 11]);
        if let Some(last) = v.get_mut(10) {
            *last = d("0");
        }
    }

    // The decoy: an unarmed infantry unit whose armour class (special 1,
    // index 9) only the jumpjets' warhead affects: every other warhead has
    // a versus value of zero there, so no other weapon may fire at it
    // (S-0304 claim 1). Neither the strike aircraft nor a deployer near the
    // islet takes it, and no deployer deploys among its own house's army
    // for a target it may not fire at; it waits for a jumpjet (M8 step B4).
    infantry(&mut r, "DECOY", |t| {
        t.primary = None;
        t.combat.armor = Some(glasnost_rules::sim::ArmourClass::Special1);
    });
    let mut jetw = r.sim.warheads["SA"].clone();
    if let Some(v) = jetw.verses.as_mut().and_then(|v| v.get_mut(9)) {
        *v = d("1");
    }
    r.sim.warheads.insert("JETW".into(), jetw);
    let mut jet_rifle = r.sim.weapons["Rifle"].clone();
    jet_rifle.warhead = Some("JETW".into());
    r.sim.weapons.insert("JetRifle".into(), jet_rifle);
    for n in ["JETI", "JET2"] {
        if let Some(t) = r.sim.infantry.get_mut(n) {
            t.primary = Some("JetRifle".into());
        }
    }
    for (n, h) in r.sim.warheads.iter_mut() {
        if n == "JETW" {
            continue;
        }
        let v = h.verses.get_or_insert_with(|| vec![d("1"); 11]);
        if let Some(special1) = v.get_mut(9) {
            *special1 = d("0");
        }
    }

    // The armed special types the barracks builds too (the army builder
    // takes them in turn with the rifleman), so that the abilities are used
    // all game long and not only by the units each house starts with. The
    // engineer and the spy are not built: the sidebar rules of the brain
    // order them. The jumpjet unit is built and leaves its barracks by its
    // ascent (S-0417 claim 15, which answers Q-1070).
    for n in [
        "GARRI", "JETI", "JET2", "DEPI", "RADI", "ERAS", "ZAPR", "PLNT", "DISA", "KAMI", "SABO",
    ] {
        if let Some(t) = r.sim.infantry.get_mut(n) {
            t.tech_level = Some(1);
            t.owner = Some(vec!["Alphaland".into(), "Betaland".into()]);
        }
    }

    // Crates: the overlay, the power-up table (money only), the timing.
    register(&mut r, "overlays", "WCRATE");
    register(&mut r, "overlays", "SCRATE");
    for n in ["WCRATE", "SCRATE"] {
        r.sim.overlays.insert(
            n.into(),
            OverlayType {
                has_art: Some(true),
                crate_: Some(true),
                ..Default::default()
            },
        );
    }
    let e = &mut r.sim.general.economy;
    e.wood_crate_img = Some("WCRATE".into());
    e.water_crate_img = Some("SCRATE".into());
    e.crate_regen = Some(d("1"));
    e.crate_radius = Some(d("2.5"));
    let mut table: Vec<Powerup> = (0..19)
        .map(|_| Powerup {
            weight: 0,
            animation: None,
            water: false,
            argument: Some(d("0")),
        })
        .collect();
    table[0].weight = 1;
    table[0].argument = Some(d("100"));
    e.powerups = Some(table);
    e.crate_minimum = Some(6);
    e.crate_maximum = Some(6);
    r
}

/// The scenario's rules.
pub fn rules() -> Arc<SimRules> {
    Arc::new(SimRules::new(&ruleset(), MAP_NAME))
}

/// The cell (dx, dy) from the middle.
fn at(dx: i32, dy: i32) -> CellXY {
    fixture::at(dx, dy)
}

fn placed(owner: &str, ty: &str, c: CellXY, key: usize) -> Placed {
    Placed {
        key: key.to_string().into_bytes(),
        owner: owner.as_bytes().to_vec(),
        type_name: ty.as_bytes().to_vec(),
        health: 256,
        cell: c,
        facing: 0,
        raw: Vec::new(),
    }
}

/// The infantry each house starts with, in placement order, then the
/// vehicles.
const INFANTRY: [&str; 27] = [
    "GARRI", "GARRI", "GARRI", "GARRI", "ENGI", "ENGI", "ENGI", "SPYU", "SPYU", "SPYU", "DEPI",
    "RADI", "RADI", "ERAS", "ERAS", "ERAS", "ZAPR", "ZAPR", "PLNT", "PLNT", "PLNT", "PLNT", "DISA",
    "DISA", "KAMI", "KAMI", "SABO",
];
const VEHICLES: [&str; 5] = ["DEPT", "FIRE", "DESIG", "DESIG", "DISV"];

/// The scenario's map.
pub fn map() -> Map {
    let mut m = glasnost_sim::testkit::empty_map(fixture::SIZE);
    let mut waypoints = std::collections::BTreeMap::new();
    for (k, dx) in [(0u16, -START_DX), (1u16, START_DX)] {
        let c = at(dx, 0);
        waypoints.insert(k, (c.x as i16, c.y as i16));
        let side = dx.signum();
        for ox in 0..5 {
            for oy in 0..5 {
                let cell = at(dx + side * (ox - 2), 7 + oy);
                if let Some(cell) = m.cells.get_mut(cell) {
                    cell.overlay = ((ox + oy) % 12) as u8;
                    cell.overlay_data = 6;
                }
            }
        }
        // The islet north of the start: water two cells wide around 3 × 3 land.
        let centre = at(dx, -7);
        for ox in -3i32..=3 {
            for oy in -3i32..=3 {
                let ring = ox.abs().max(oy.abs());
                if ring >= 2
                    && let Some(cell) = m.cells.get_mut(CellXY::new(centre.x + ox, centre.y + oy))
                {
                    cell.tile = fixture::WATER_TILE;
                    cell.overlay = glasnost_map::NO_OVERLAY;
                }
            }
        }
    }
    m.waypoints = waypoints;
    let mut key = 0;
    for (name, other, dx) in [("Alpha", "Beta", -START_DX), ("Beta", "Alpha", START_DX)] {
        // Rows of units south of the start, five to a row.
        for (i, ty) in INFANTRY.iter().enumerate() {
            // Two cells apart, so that no bomb or blast reaches a neighbour.
            let c = at(dx + 2 * (i as i32 % 5) - 4, 3 + 2 * (i as i32 / 5));
            m.objects.infantry.push(Infantry {
                placed: placed(name, ty, c, key),
                sub_cell: 2,
            });
            key += 1;
        }
        for (i, ty) in VEHICLES.iter().enumerate() {
            let c = at(dx + i as i32 * 2 - 2, 6);
            m.objects.units.push(Unit {
                placed: placed(name, ty, c, key),
            });
            key += 1;
        }
        // The other house's decoy on the islet north of this start.
        m.objects.infantry.push(Infantry {
            placed: placed(other, "DECOY", at(dx, -7), key),
            sub_cell: 2,
        });
        key += 1;
        let side = dx.signum();
        // The other house's hulks on this start's side toward the middle.
        for oy in [-3, 0] {
            m.objects.units.push(Unit {
                placed: placed(other, "HULK", at(dx - side * 6, oy), key),
            });
            key += 1;
        }
        // The neutral house's buildings, either side of the start.
        for (ty, (ox, oy)) in [
            ("HUT", (side * 5, -2)),
            ("HUT", (side * 5, 1)),
            ("DERR", (side * 7, 4)),
            ("DERR", (side * 9, 1)),
            ("SPYB", (side * 8, -2)),
            ("SPYB", (side * 8, 7)),
        ] {
            m.objects.structures.push(Structure {
                placed: placed("Neutral", ty, at(dx + ox, oy), key),
            });
            key += 1;
        }
    }
    m
}

/// The lobby setup: Alpha at the west start, Beta at the east, crates on.
pub fn setup(seed: u32) -> GameSetup {
    let player = |name: &str, country: &str, start: u32| Player {
        name: name.into(),
        country: country.into(),
        start: Some(start),
        ai: true,
        values: Default::default(),
    };
    setup::skirmish(
        &rules(),
        &[
            player("Alpha", "Alphaland", 0),
            player("Beta", "Betaland", 1),
        ],
        Settings {
            crates: Some(true),
            ..Default::default()
        },
        0,
        seed,
    )
}
