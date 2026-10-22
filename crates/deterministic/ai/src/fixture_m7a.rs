//! The synthetic M7a skirmish for the determinism gate (M7a step 4): the
//! fixture's ruleset and map with invented types, so that a plain
//! AI-versus-AI game uses the psychic weapons of S-0610 (berserk hits, the
//! psychic sensor and the psychic reveal) and a slave-miner economy. Every
//! name and number here is test data of our own, not game behaviour; the
//! abilities' behaviour comes from the specs the sim crate cites, and this
//! file only supplies the types and the places.
//!
//! - **Alpha** plays a Yuri-style side: no refinery and no harvester of the
//!   fixture, but SMV (a slave-miner vehicle with three SLAVE walkers,
//!   built at the war factory) and SMB (its deployed form, which is also
//!   the side's refinery on the sidebar). **Beta** keeps the fixture's
//!   refinery and harvester.
//! - Both sides: MADMAN (a rifleman whose weapon MADRAY has the
//!   berserk-causing warhead MAD; each house starts with a row of them and
//!   the barracks builds more), EYE (a psychic sensor building, built by
//!   the AI) and MINDB (a building that gives the psychic
//!   reveal REVEAL, buildable once a war factory stands).
//! - Alpha only (S-0609, S-0601): GATV (a gattling vehicle whose six
//!   stage slots hold the weapons SPIN0 … SPIN2, built at the war
//!   factory), GATB (a gattling defence building) and TAKER (a rifleman
//!   whose weapon THRALL has the takeover warhead MIND, capacity 4,
//!   range 2). Its victims are Beta's units, none psionics-immune.
//! - Alpha only (S-0613): GATV's and GATB's top-stage weapon SPIN2 fires
//!   VENOM, the gun's warhead with the virus death for the infantry it
//!   kills and a gas cloud system VAPOUR at each hit. The virus death's
//!   animation PLAGUE and the shared system GasCloudSys hold the gas
//!   particle FOG. No new buildable type: one would take a share of the
//!   barracks from the other abilities' units.
//! - Alpha only (S-0605): BIOP, an infantry absorber, is Alpha's power
//!   plant in place of the fixture's POWR (which only Beta builds); the AI
//!   sends spare and controlled infantry in.
//! - Alpha only (S-0603): GRND, a recycling building, built by the AI,
//!   and HULK, an unarmed vehicle nobody can build: Alpha starts with
//!   [`HULKS`] of them, badly damaged, which a person would recycle.
//!   A skirmish drops
//!   the structures a map gives a player's house (S-0009 claim 27), so the
//!   houses start with units only.
//! - Both sides (S-0606): BUNK, a tank bunker built by the AI, which
//!   sends GUNR and GATV in and lets them out again.
//! - Both sides (S-0607): MAGN, a magnetic lifter built at the war
//!   factory, whose beam MAGRAY seizes the enemy's vehicles; the general
//!   crushing and demolition warheads SQUASH and BLAST for the falls.
//!   Both sides have it because an Alpha-only lifter tipped the duel:
//!   Alpha won the gate seed before its last tick.
//!
//! - The gate's margin setup (M7b step 2a; [`ruleset`] and [`map`] over
//!   [`base_ruleset`] and [`base_map`], which the M7b scenario uses
//!   unchanged): the ore does not spread, so each house keeps the plan's
//!   two harvesters; PREY, an unarmed sturdy vehicle that only the
//!   lifter's and the controller's warheads affect, five of each house
//!   near the other's start, each worth 1 credit; start vehicles
//!   ([`START_VEHICLES`]): GATV and MAGN for Alpha, a second MINR for
//!   Beta; a start TAKER for Alpha ([`START_INFANTRY`]); Alpha's HULKs
//!   behind its start, of PREY's armour class, so that only the lifters
//!   and the controllers touch them before the grinder stands; a frail
//!   EYE, which a raid can destroy and its house builds again; WARD, an
//!   unbuildable turreted vehicle whose gun sits in a multi-weapon slot
//!   (no primary weapon, so the AI's waves leave it at home), one behind
//!   each start, which the house's bunkers take in. The PREY, the TAKER,
//!   the HULKs' class and WARD came in M7b AI round 3, for seeds of 1 to
//!   16 with no takeover order, no unit ground or no vehicle bunkered; the
//!   TAKER begins [`START_INFANTRY_AHEAD`] cells toward the middle since
//!   M8 step B4, for seed 20 of 1 to 32 with no takeover order.
//!
//! To add a type, insert it in [`base_ruleset`] (with `register`) and, for
//! a start unit, add its name to [`INFANTRY`] or [`VEHICLES`] (or, for the
//! gate's duel only, to [`START_VEHICLES`]); the gate's module
//! (`crates/sim-gate/src/m7a.rs`) has the counters.

use std::sync::Arc;

use glasnost_map::objects::{Infantry, Placed, Unit};
use glasnost_map::{CellXY, Map};
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{
    AnimationType, CombatFields, Decimal, LocomotionClass, Locomotor, ParticleSystemType,
    ParticleType, UnitType, WarheadType, WeaponType,
};
use glasnost_rules::sim_econ::SuperweaponType;
use glasnost_sim::econ::setup::{self, Player, Settings};
use glasnost_sim::rules::SimRules;
use glasnost_sim::world::GameSetup;

use crate::fixture;

/// The start offset from the middle along x (closer than the fixture's, so
/// that the armies meet earlier).
pub const START_DX: i32 = 16;

/// The lobby's starting credits (more than the fixture's, so that both
/// houses reach the psychic buildings early).
pub const CREDITS: i32 = 15_000;

/// The name of the scenario's map.
pub const MAP_NAME: &str = "synthetic-m7a";

fn d(s: &str) -> Decimal {
    Decimal(s.into())
}

fn register(r: &mut Ruleset, reg: &str, name: &str) {
    let l = r.registries.entry(reg.into()).or_default();
    if !l.iter().any(|n| n == name) {
        l.push(name.into());
    }
}

/// The required-country list of one country (S-0407 claim 5 (b)): only
/// that country's houses may build the type. The owner lists stay the
/// fixture's, which the factories share.
fn only(country: &str) -> Option<Vec<String>> {
    Some(vec![country.into()])
}

/// The scenario's ruleset: [`base_ruleset`] with the gate's margin setup
/// ([`margins`]).
pub fn ruleset() -> Ruleset {
    let mut r = base_ruleset();
    margins(&mut r);
    r
}

/// The gate's margin setup (M7b step 2a), on top of [`base_ruleset`].
fn margins(r: &mut Ruleset) {
    // The resource does not spread: the fields stay at their 25 cells
    // each, so that a house keeps the plan's harvesters (the AI wants one
    // per 20 resource cells near its refineries) and spends on its army.
    if let Some(t) = r.sim.tiberiums.get_mut("ORE") {
        t.spread = Some(0);
    }
    // A frail psychic sensor, which raids destroy and the house builds
    // again.
    if let Some(b) = r.sim.buildings.get_mut("EYE") {
        b.strength = Some(EYE_STRENGTH);
    }
    // PREY: an unarmed, sturdy vehicle nobody can build, of the last
    // armour class, which only the lifter's beam and the mind
    // controller's warhead affect: every other warhead has a versus value
    // of zero there, so no other weapon may fire at it (S-0304 claim 1)
    // and the armies leave it to the lifters and the controllers. No
    // other type of the scenario has that class.
    let mut prey = r.sim.vehicles["GUNR"].clone();
    prey.primary = None;
    prey.secondary = None;
    prey.strength = Some(2000);
    // Worth next to nothing, so that grinding a controlled one pays its
    // taker's house next to nothing (S-0603 claim 8).
    prey.cost = Some(1);
    prey.tech_level = Some(-1);
    prey.combat.armor = Some(glasnost_rules::sim::ArmourClass::Special2);
    register(r, "vehicles", "PREY");
    r.sim.vehicles.insert("PREY".into(), prey);
    // Alpha's HULKs of the same class (M7b AI round 3): the raids' berserk
    // hits sent them hunting and killed them before the grinder stood
    // (the sweep's seeds with no unit ground); only the lifters and the
    // controllers touch them now, and they wait for the grinder.
    if let Some(h) = r.sim.vehicles.get_mut("HULK") {
        h.combat.armor = Some(glasnost_rules::sim::ArmourClass::Special2);
    }
    // WARD: a turreted vehicle nobody can build, whose one gun sits in a
    // weapon slot of a multi-weapon turret (no primary weapon). It is
    // bunker-eligible (S-0606 claim 1: turreted, with a current weapon),
    // and the AI, which counts a vehicle with a primary weapon (or a
    // gattling's slots) as army, keeps it at home: each house starts with
    // one, which its bunkers take in (the sweep's seeds with no vehicle
    // bunkered had every tank fighting when the bunkers stood). Its armour
    // class (special 1, index 9) has a versus value of zero for every
    // warhead, so no weapon may fire at it, the lifters' and the
    // controllers' included (S-0304 claim 1; M8 step B4: a berserk gattling
    // of its own house, or the enemy's, killed it before the bunkers stood,
    // or a controller took it, in the seeds of 1 to 32 that bunkered no
    // vehicle with a quirk switch flipped).
    let mut ward = r.sim.vehicles["GUNR"].clone();
    ward.primary = None;
    ward.secondary = None;
    ward.turret_count = Some(1);
    ward.weapon_count = Some(1);
    ward.combat.weapon_slots = Some(vec!["Gun".into()]);
    ward.strength = Some(2000);
    ward.tech_level = Some(-1);
    ward.combat.armor = Some(glasnost_rules::sim::ArmourClass::Special1);
    register(r, "vehicles", "WARD");
    r.sim.vehicles.insert("WARD".into(), ward);
    for (n, h) in r.sim.warheads.iter_mut() {
        let v = h.verses.get_or_insert_with(|| vec![d("1"); 11]);
        if let Some(special1) = v.get_mut(9) {
            *special1 = d("0");
        }
        if n == "MAGWH" || n == "MIND" {
            continue;
        }
        if let Some(last) = v.get_mut(10) {
            *last = d("0");
        }
    }
}

/// The M7a types on the fixture's ruleset, without the gate's margin
/// setup; the M7b scenario (`fixture_m7b`) builds on this.
pub fn base_ruleset() -> Ruleset {
    let mut r = fixture::ruleset();

    // Beta's economy: the fixture's refinery and harvester.
    if let Some(b) = r.sim.buildings.get_mut("PROC") {
        b.econ.required_houses = only("Betaland");
    }
    if let Some(v) = r.sim.vehicles.get_mut("MINR") {
        v.econ.required_houses = only("Betaland");
    }

    // Alpha's economy: the slave miner, its slaves and its deployed form.
    let group = |e: &mut glasnost_rules::sim_econ::EconFields| {
        e.enslaves = Some("SLAVE".into());
        e.slaves_number = Some(3);
        e.slave_regen_rate = Some(50);
        e.slave_reload_rate = Some(40);
    };
    let mut slave = UnitType {
        strength: Some(100),
        speed: Some(6),
        locomotion_class: Some(LocomotionClass::Foot),
        locomotor: Some(Locomotor::Walk),
        storage: Some(2),
        sight: Some(4),
        ..Default::default()
    };
    slave.econ.harvest_rate = Some(3);
    register(&mut r, "infantry", "SLAVE");
    r.sim.infantry.insert("SLAVE".into(), slave);
    let mut smv = UnitType {
        strength: Some(1000),
        speed: Some(4),
        rot: Some(6),
        locomotion_class: Some(LocomotionClass::Wheel),
        locomotor: Some(Locomotor::Drive),
        tech_level: Some(1),
        owner: Some(vec!["Alphaland".into(), "Betaland".into()]),
        cost: Some(1500),
        sight: Some(6),
        deploys_into: Some("SMB".into()),
        ..Default::default()
    };
    smv.econ.resource_gatherer = Some(true);
    smv.econ.resource_destination = Some(true);
    smv.econ.required_houses = only("Alphaland");
    group(&mut smv.econ);
    register(&mut r, "vehicles", "SMV");
    r.sim.vehicles.insert("SMV".into(), smv);
    let mut smb = fixture::building(3, 3, 1500, -40);
    smb.econ.required_houses = only("Alphaland");
    smb.undeploys_into = Some("SMV".into());
    smb.econ.refinery = Some(true);
    smb.econ.buildup = Some(true);
    smb.econ.buildup_frames = Some(10);
    group(&mut smb.econ);
    register(&mut r, "buildings", "SMB");
    r.sim.buildings.insert("SMB".into(), smb);

    // The berserk weapon and its carrier.
    r.sim.warheads.insert(
        "MAD".into(),
        WarheadType {
            psychedelic: Some(true),
            verses: Some(vec![d("1"); 11]),
            ..Default::default()
        },
    );
    r.sim.weapons.insert(
        "MADRAY".into(),
        WeaponType {
            damage: Some(200),
            rof: Some(60),
            range: Some(d("5")),
            speed: Some(100),
            projectile: Some("Bullet".into()),
            warhead: Some("MAD".into()),
            ..Default::default()
        },
    );
    let mut madman = r.sim.infantry["WALKER"].clone();
    madman.primary = Some("MADRAY".into());
    madman.secondary = None;
    madman.cost = Some(300);
    register(&mut r, "infantry", "MADMAN");
    r.sim.infantry.insert("MADMAN".into(), madman);

    // The psychic sensor.
    let mut eye = fixture::building(1, 1, 150, 0);
    eye.econ.psychic_detection_radius = Some(12);
    register(&mut r, "buildings", "EYE");
    r.sim.buildings.insert("EYE".into(), eye);

    // The psychic reveal and the building that gives it.
    r.sim.superweapons.insert(
        "REVEAL".into(),
        SuperweaponType {
            kind: Some("PsychicReveal".into()),
            action: Some("PsychicReveal".into()),
            recharge_time: Some(d("1")),
            is_powered: Some(false),
            ..Default::default()
        },
    );
    register(&mut r, "superweapons", "REVEAL");
    let mut mind = fixture::building(2, 2, 600, 0);
    mind.econ.prerequisite = Some(vec!["FACT".into()]);
    mind.econ.super_weapon = Some("REVEAL".into());
    register(&mut r, "buildings", "MINDB");
    r.sim.buildings.insert("MINDB".into(), mind);

    // Alpha's gattling vehicle and defence: three weapons of growing
    // damage, one per stage (slots 0 and 1, 2 and 3, 4 and 5 alike).
    let warhead = r.sim.weapons["Gun"].warhead.clone();
    for (i, dmg) in [(0, 6), (1, 9), (2, 12)] {
        r.sim.weapons.insert(
            format!("SPIN{i}"),
            WeaponType {
                damage: Some(dmg),
                rof: Some(20),
                range: Some(d("6")),
                speed: Some(100),
                projectile: Some("Bullet".into()),
                warhead: warhead.clone(),
                ..Default::default()
            },
        );
    }
    let spin = |c: &mut CombatFields| {
        c.weapon_slots = Some((0..6).map(|i| format!("SPIN{}", i / 2)).collect());
        c.is_gattling = Some(true);
        c.weapon_stages = Some(3);
        c.rate_up = Some(4);
        c.rate_down = Some(2);
        c.stage_thresholds = Some(vec![20, 60, 120]);
    };
    let mut gatv = r.sim.vehicles["GUNR"].clone();
    gatv.primary = None;
    gatv.secondary = None;
    gatv.turret_count = Some(1);
    gatv.weapon_count = Some(6);
    gatv.cost = Some(900);
    gatv.econ.required_houses = only("Alphaland");
    spin(&mut gatv.combat);
    register(&mut r, "vehicles", "GATV");
    r.sim.vehicles.insert("GATV".into(), gatv);
    let mut gatb = fixture::building(1, 1, 500, 0);
    gatb.strength = Some(1200);
    gatb.turret_count = Some(1);
    gatb.econ.required_houses = only("Alphaland");
    spin(&mut gatb.combat);
    register(&mut r, "buildings", "GATB");
    r.sim.buildings.insert("GATB".into(), gatb);

    // Alpha's mind controller.
    r.sim.warheads.insert(
        "MIND".into(),
        WarheadType {
            mind_control: Some(true),
            verses: Some(vec![d("1"); 11]),
            ..Default::default()
        },
    );
    r.sim.weapons.insert(
        "THRALL".into(),
        WeaponType {
            damage: Some(4),
            rof: Some(60),
            range: Some(d("2")),
            speed: Some(100),
            projectile: Some("Bullet".into()),
            warhead: Some("MIND".into()),
            ..Default::default()
        },
    );
    let mut taker = r.sim.infantry["WALKER"].clone();
    taker.primary = Some("THRALL".into());
    taker.secondary = None;
    taker.cost = Some(200);
    taker.econ.required_houses = only("Alphaland");
    register(&mut r, "infantry", "TAKER");
    r.sim.infantry.insert("TAKER".into(), taker);

    // Alpha's gas weapon (S-0613): the gattling's top-stage weapon SPIN2
    // fires VENOM, the gun's warhead with the virus death (kind 8) for the
    // infantry it kills and a cloud system VAPOUR at each hit. The virus
    // death's animation PLAGUE puts FOG clouds into the shared system
    // GasCloudSys.
    let fog = ParticleType {
        behaves_like: Some("gas".into()),
        max_ec: Some(150),
        max_dc: Some(30),
        damage: Some(3),
        warhead: Some("VENOM".into()),
        state_ai_advance: Some(4),
        end_state_ai: Some(100),
        ..Default::default()
    };
    register(&mut r, "particles", "FOG");
    r.sim.particles.insert("FOG".into(), fog);
    for name in ["GasCloudSys", "VAPOUR"] {
        register(&mut r, "particle_systems", name);
        r.sim.particle_systems.insert(
            name.into(),
            ParticleSystemType {
                behaves_like: Some("gas".into()),
                holds_what: Some("FOG".into()),
                lifetime: None,
                ..Default::default()
            },
        );
    }
    register(&mut r, "animations", "PLAGUE");
    r.sim.animations.insert(
        "PLAGUE".into(),
        AnimationType {
            middle_frame: Some(3),
            frame_count: Some(8),
            spawns_particle: Some("FOG".into()),
            num_particles: Some(2),
            ..Default::default()
        },
    );
    r.sim.general.infantry_virus = Some("PLAGUE".into());
    let mut venom = warhead
        .as_ref()
        .and_then(|h| r.sim.warheads.get(h))
        .cloned()
        .unwrap_or_default();
    venom.inf_death = Some(8);
    venom.particle = Some("VAPOUR".into());
    r.sim.warheads.insert("VENOM".into(), venom);
    if let Some(top) = r.sim.weapons.get_mut("SPIN2") {
        top.warhead = Some("VENOM".into());
    }

    // Alpha's power plant (S-0605): BIOP takes the fixture's POWR's place
    // on Alpha's sidebar (POWR is Beta's only), with the same foundation
    // and cost but half the power (50), and absorbs up to five infantry of
    // size 1 or less, each adding 40 power, so that a short house sends
    // its infantry in.
    if let Some(b) = r.sim.buildings.get_mut("POWR") {
        b.econ.required_houses = only("Betaland");
    }
    let mut biop = fixture::building(2, 2, 600, 50);
    biop.infantry_absorb = Some(true);
    biop.passengers = Some(5);
    biop.size_limit = Some(d("1"));
    biop.econ.extra_power = Some(40);
    biop.econ.required_houses = only("Alphaland");
    register(&mut r, "buildings", "BIOP");
    r.sim.buildings.insert("BIOP".into(), biop);

    // Alpha's grinder (S-0603): GRND, a 2 × 2 recycling building with no
    // power of its own; the AI builds one and recycles controlled enemy
    // units, badly damaged units and surplus infantry in it.
    let mut grnd = fixture::building(2, 2, 400, 0);
    grnd.grinding = Some(true);
    grnd.econ.required_houses = only("Alphaland");
    register(&mut r, "buildings", "GRND");
    r.sim.buildings.insert("GRND".into(), grnd);
    // Both sides' tank bunker (S-0606): BUNK, a 2 × 2 bunker with no
    // impassable rows, built by the AI near its base; GUNR and Alpha's
    // gattling GATV are turreted and armed, so they may go in.
    let mut bunk = fixture::building(2, 2, 600, 0);
    bunk.strength = Some(1500);
    bunk.bunker = Some(true);
    bunk.number_impassable_rows = Some(0);
    register(&mut r, "buildings", "BUNK");
    r.sim.buildings.insert("BUNK".into(), bunk);
    // The magnetic lifter of both sides (S-0607): MAGN, a vehicle built at the war
    // factory whose beam MAGRAY (damage 10, so every fixture vehicle of
    // size 1 can be seized) has the seizing warhead MAGWH with the jumpjet
    // locomotor. The falls of S-0607 claim 18 detonate SQUASH (crushing)
    // and BLAST (demolition); the fixture has no game-wide death weapon, so
    // they are plain area detonations (S-0607 claim 18, the design rule).
    r.sim.warheads.insert(
        "MAGWH".into(),
        WarheadType {
            is_locomotor: Some(true),
            locomotor: Some("jumpjet".into()),
            verses: Some(vec![d("1"); 11]),
            ..Default::default()
        },
    );
    r.sim.weapons.insert(
        "MAGRAY".into(),
        WeaponType {
            damage: Some(10),
            rof: Some(40),
            range: Some(d("7")),
            speed: Some(100),
            projectile: Some("Bullet".into()),
            warhead: Some("MAGWH".into()),
            is_mag_beam: Some(true),
            ..Default::default()
        },
    );
    for name in ["SQUASH", "BLAST"] {
        r.sim.warheads.insert(
            name.into(),
            WarheadType {
                cell_spread: Some(d("1")),
                verses: Some(vec![d("1"); 11]),
                ..Default::default()
            },
        );
    }
    let g = &mut r.sim.general;
    g.crush_warhead.get_or_insert_with(|| "SQUASH".into());
    g.c4_warhead.get_or_insert_with(|| "BLAST".into());
    let mut magn = r.sim.vehicles["GUNR"].clone();
    magn.primary = Some("MAGRAY".into());
    magn.secondary = None;
    magn.strength = Some(400);
    magn.cost = Some(400);
    register(&mut r, "vehicles", "MAGN");
    r.sim.vehicles.insert("MAGN".into(), magn);
    // HULK: an unarmed vehicle no house can build (tech level -1); Alpha
    // starts with a few of them, badly damaged ([`HULKS`]).
    let mut hulk = r.sim.vehicles["TRUK"].clone();
    hulk.passengers = None;
    hulk.size_limit = None;
    hulk.econ.prerequisite = None;
    hulk.tech_level = Some(-1);
    register(&mut r, "vehicles", "HULK");
    r.sim.vehicles.insert("HULK".into(), hulk);
    r
}

/// The scenario's rules.
pub fn rules() -> Arc<SimRules> {
    Arc::new(SimRules::new(&ruleset(), MAP_NAME))
}

/// The rules of [`base_ruleset`], without the gate's margin setup.
pub fn base_rules() -> Arc<SimRules> {
    Arc::new(SimRules::new(&base_ruleset(), MAP_NAME))
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

/// The infantry each house starts with, in placement order, in rows of five
/// south of its start.
pub const INFANTRY: [&str; 6] = ["MADMAN", "MADMAN", "MADMAN", "MADMAN", "MADMAN", "MADMAN"];
/// The vehicles each house starts with, in a row south of its infantry.
pub const VEHICLES: [&str; 0] = [];
/// The badly damaged HULKs Alpha starts with, in a row north of its start,
/// and their health on the map's scale of 256 (about a fifth).
pub const HULKS: usize = 4;
const HULK_HEALTH: i32 = 51;

/// The scenario's map: [`base_map`] with the gate's margin setup: each
/// house's extra start vehicles ([`START_VEHICLES`]) in a row north of
/// its start.
pub fn map() -> Map {
    let mut m = base_map();
    // Alpha's HULKs stand behind its start, away from the middle, out of
    // the way of the first raids.
    let mut hulk = 0;
    for u in &mut m.objects.units {
        if u.placed.type_name == b"HULK" {
            u.placed.cell = at(-START_DX - HULK_BEHIND, 2 * hulk - 3);
            hulk += 1;
        }
    }
    let mut key = m.objects.infantry.len() + m.objects.units.len();
    for (name, dx, list) in START_VEHICLES {
        for (i, ty) in list.iter().enumerate() {
            let c = at(dx + 2 * i as i32 - 4, START_VEHICLE_ROW);
            m.objects.units.push(Unit {
                placed: placed(name, ty, c, key),
            });
            key += 1;
        }
    }
    for (name, dx, list) in START_INFANTRY {
        for (i, ty) in list.iter().enumerate() {
            let c = at(
                dx + dx.signum() * (2 * i as i32 - START_INFANTRY_AHEAD),
                START_INFANTRY_ROW,
            );
            m.objects.infantry.push(Infantry {
                placed: placed(name, ty, c, key),
                sub_cell: 2,
            });
            key += 1;
        }
    }
    // Each house's WARD behind its start, beside the base its bunkers
    // stand in, out of the first raids' way.
    for (name, dx) in [("Alpha", -START_DX), ("Beta", START_DX)] {
        m.objects.units.push(Unit {
            placed: placed(name, "WARD", at(dx + dx.signum() * HULK_BEHIND, 3), key),
        });
        key += 1;
    }
    // The other house's prey on each start's side toward the middle.
    for (other, dx) in [("Beta", -START_DX), ("Alpha", START_DX)] {
        for (ox, oy) in PREY_AT {
            m.objects.units.push(Unit {
                placed: placed(other, "PREY", at(dx - dx.signum() * ox, oy), key),
            });
            key += 1;
        }
    }
    m
}

/// The psychic sensor's strength in the gate's scenario.
const EYE_STRENGTH: i32 = 10;

/// Cells behind Alpha's start (away from the middle) where its HULKs
/// stand in the gate's scenario, in a column.
const HULK_BEHIND: i32 = 7;

/// Where the other house's PREY stand near each start: cells toward the
/// middle and rows.
const PREY_AT: [(i32, i32); 5] = [(6, -3), (6, 0), (10, -6), (14, -3), (14, 2)];

/// The extra vehicles each house starts with in the gate's scenario (by
/// house name and start offset).
pub const START_VEHICLES: [(&str, i32, &[&str]); 2] = [
    ("Alpha", -START_DX, &["GATV", "MAGN"]),
    ("Beta", START_DX, &["MINR"]),
];
/// The row (dy from the middle) of the extra start vehicles.
const START_VEHICLE_ROW: i32 = -3;

/// The extra infantry each house starts with in the gate's scenario (by
/// house name and start offset), in a row north of the extra vehicles.
pub const START_INFANTRY: [(&str, i32, &[&str]); 1] = [("Alpha", -START_DX, &["TAKER"])];
/// The row (dy from the middle) of the extra start infantry.
const START_INFANTRY_ROW: i32 = -5;
/// Cells toward the middle from its start where the extra start infantry
/// begin (M8 step B4): the TAKER stands within the brain's mind-control
/// reach of the nearest PREY from the first think, so that the brain gives
/// a takeover order early. Before, seed 20 of the 32-seed sweep ended with
/// four PREY changed owner and no takeover order given at all.
const START_INFANTRY_AHEAD: i32 = 2;

/// The fixture's flat ground and resource fields with the starts at
/// ±[`START_DX`] and each house's start objects, without the gate's margin
/// setup; the M7b scenario (`fixture_m7b`) builds on this.
pub fn base_map() -> Map {
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
    }
    m.waypoints = waypoints;
    let mut key = 0;
    for (name, dx) in [("Alpha", -START_DX), ("Beta", START_DX)] {
        for (i, ty) in INFANTRY.iter().enumerate() {
            let c = at(dx + 2 * (i as i32 % 5) - 4, 3 + 2 * (i as i32 / 5));
            m.objects.infantry.push(Infantry {
                placed: placed(name, ty, c, key),
                sub_cell: 2,
            });
            key += 1;
        }
        let row = 3 + 2 * ((INFANTRY.len() as i32 + 4) / 5);
        for (i, ty) in VEHICLES.iter().enumerate() {
            let c = at(dx + i as i32 * 2 - 2, row);
            m.objects.units.push(Unit {
                placed: placed(name, ty, c, key),
            });
            key += 1;
        }
        if name == "Alpha" {
            for i in 0..HULKS {
                let c = at(dx + 2 * i as i32 - 3, -6);
                let mut p = placed(name, "HULK", c, key);
                p.health = HULK_HEALTH;
                m.objects.units.push(Unit { placed: p });
                key += 1;
            }
        }
    }
    m
}

/// The lobby setup: Alpha (the Yuri-style side) at the west start, Beta at
/// the east.
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
            credits: Some(CREDITS),
            ..Default::default()
        },
        0,
        seed,
    )
}
