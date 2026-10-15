//! The synthetic skirmish: an invented ruleset and map (no game data) on
//! which two scripted houses build a base and fight, for the determinism
//! gate and the tests. Every number here is test data of our own, not game
//! behaviour.
//!
//! - Buildings: YARD (construction yard, 3 × 3, builds buildings), POWR
//!   (power plant, 2 × 2), PROC (dock-unload refinery, 3 × 3), PILL (barracks, 2 × 2),
//!   FACT (war factory, 3 × 3), and a tech chain: RADR (2 × 2, needs
//!   FACT), LAB (2 × 2, needs RADR; a name that is not a generic group word).
//! - Vehicles: MCV (deploys into YARD, the base unit), MINR (harvester,
//!   docks at PROC), GUNR (a gun vehicle), LNCH (a V3-like launcher with
//!   one rocket slot of the set-A type RKT, needs LAB); the testkit's
//!   CART, CRUSHER and BUGGY stay registered.
//! - Aircraft: RKT (the set-A rocket type, S-0506).
//! - Infantry: WALKER (the testkit's rifleman).
//! - Resource: ORE (overlays 0 … 11, stages 0 … 11).
//! - Map: a flat clear square of side [`SIZE`] (cell coordinates 0 … 2·SIZE
//!   on both axes), start waypoints 0 and 1 on opposite sides of the
//!   middle, and an ore field beside each; [`map_diagonal`] puts the starts
//!   on a diagonal with a third ore field between them.

use std::collections::BTreeMap;
use std::sync::Arc;

use glasnost_map::{CellXY, Map};
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{
    BuildingType, CombatFields, Decimal, LandRow, LandType, LocomotionClass, Locomotor,
    OverlayType, UnitType, WarheadType, WeaponType,
};
use glasnost_rules::sim_econ::TiberiumType;
use glasnost_sim::econ::setup::{self, Player, Settings};
use glasnost_sim::rules::SimRules;
use glasnost_sim::testkit;
use glasnost_sim::world::GameSetup;

use crate::scripted::Plan;

/// Half the map's side: the middle cell is (SIZE, SIZE).
pub const SIZE: i32 = 40;

/// The start offset from the middle along x.
pub const START_DX: i32 = 22;

fn owners() -> Option<Vec<String>> {
    Some(vec!["Alphaland".into(), "Betaland".into()])
}

fn push(r: &mut Ruleset, reg: &str, n: &str) {
    let v = r.registries.entry(reg.into()).or_default();
    if !v.iter().any(|x| x == n) {
        v.push(n.into());
    }
}

fn building(w: i32, h: i32, cost: i32, power: i32) -> BuildingType {
    let mut b = BuildingType {
        strength: Some(1000),
        foundation: Some([w, h]),
        tech_level: Some(1),
        owner: owners(),
        cost: Some(cost),
        ..Default::default()
    };
    b.econ.power = Some(power);
    b.econ.sight = Some(6);
    b
}

fn vehicle(strength: i32, speed: i32, cost: i32) -> UnitType {
    UnitType {
        strength: Some(strength),
        speed: Some(speed),
        rot: Some(6),
        locomotion_class: Some(LocomotionClass::Wheel),
        locomotor: Some(Locomotor::Drive),
        tech_level: Some(1),
        owner: owners(),
        cost: Some(cost),
        sight: Some(6),
        ..Default::default()
    }
}

/// The skirmish ruleset: the testkit's combat ruleset (weapons, warheads,
/// projectiles) plus the base, harvest and army types above.
pub fn ruleset() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    let s = &mut r.sim;
    s.land_types.insert(
        LandType::Clear,
        LandRow {
            buildable: Some(true),
            ..Default::default()
        },
    );
    s.land_types.insert(LandType::Tiberium, LandRow::default());
    s.general.base_unit = Some(vec!["MCV".into()]);

    let mut yard = building(3, 3, 3000, 0);
    yard.factory = Some("building".into());
    yard.econ.base_normal = Some(true);
    let powr = building(2, 2, 600, 100);
    let mut proc_ = building(3, 3, 1500, -40);
    proc_.econ.refinery = Some(true);
    proc_.econ.dock_unload = Some(true);
    // Harvesters queue beside the docking cell (east of the foundation).
    proc_.econ.queueing_cell = Some([3, 2]);
    let mut pill = building(2, 2, 500, -20);
    pill.factory = Some("infantry".into());
    let mut fact = building(3, 3, 1500, -30);
    fact.factory = Some("vehicle".into());
    let mut radr = building(2, 2, 1000, -40);
    radr.econ.prerequisite = Some(vec!["FACT".into()]);
    let mut lab = building(2, 2, 1500, -60);
    lab.econ.prerequisite = Some(vec!["RADR".into()]);
    for (n, b) in [
        ("YARD", yard),
        ("POWR", powr),
        ("PROC", proc_),
        ("PILL", pill),
        ("FACT", fact),
        ("RADR", radr),
        ("LAB", lab),
    ] {
        r.sim.buildings.insert(n.into(), b);
        push(&mut r, "buildings", n);
    }

    let mut mcv = vehicle(1000, 4, 3000);
    mcv.deploys_into = Some("YARD".into());
    let mut minr = vehicle(800, 5, 1000);
    minr.harvester = Some(true);
    minr.storage = Some(20);
    minr.econ.dock = Some(vec!["PROC".into()]);
    let mut gunr = vehicle(400, 7, 700);
    gunr.primary = Some("Gun".into());
    gunr.turret = Some(true);
    gunr.combat.primary_fire_flh = Some([90, 0, 70]);
    let mut lnch = vehicle(300, 4, 1200);
    lnch.primary = Some("Launch".into());
    lnch.missile_spawn = Some(true);
    lnch.econ.prerequisite = Some(vec!["LAB".into()]);
    lnch.combat.spawns = Some("RKT".into());
    lnch.combat.spawns_number = Some(1);
    lnch.combat.spawn_regen_rate = Some(200);
    lnch.combat.spawn_reload_rate = Some(0);
    lnch.combat.primary_fire_flh = Some([60, 0, 80]);
    for (n, u) in [("MCV", mcv), ("MINR", minr), ("GUNR", gunr), ("LNCH", lnch)] {
        r.sim.vehicles.insert(n.into(), u);
        push(&mut r, "vehicles", n);
    }
    r.sim.aircraft.insert(
        "RKT".into(),
        UnitType {
            strength: Some(40),
            speed: Some(50),
            rot: Some(20),
            missile_spawn: Some(true),
            spawned: Some(true),
            locomotor: Some(Locomotor::Rocket),
            // Not scored (S-0414 claim 4): a rocket stalled at the map edge
            // (S-0506 claim 18) would otherwise keep a beaten house alive.
            combat: CombatFields {
                dont_score: Some(true),
                ..Default::default()
            },
            ..Default::default()
        },
    );
    push(&mut r, "aircraft", "RKT");
    r.sim.weapons.insert(
        "Launch".into(),
        WeaponType {
            damage: Some(1),
            rof: Some(60),
            range: Some(Decimal("18".into())),
            projectile: Some("Bullet".into()),
            warhead: Some("HE".into()),
            spawner: Some(true),
            ..Default::default()
        },
    );
    r.sim.warheads.insert(
        "RKTW".into(),
        WarheadType {
            cell_spread: Some(Decimal("2".into())),
            percent_at_max: Some(Decimal("0.5".into())),
            verses: Some((0..11).map(|_| Decimal("1".into())).collect()),
            ..Default::default()
        },
    );
    let g = &mut r.sim.general;
    g.v3_rocket_type = Some("RKT".into());
    g.v3_rocket_pause_frames = Some(6);
    g.v3_rocket_tilt_frames = Some(12);
    g.v3_rocket_altitude = Some(700);
    g.v3_rocket_pitch_final = Some(Decimal("0.6".into()));
    g.v3_rocket_acceleration = Some(Decimal("3".into()));
    g.v3_rocket_damage = Some(250);
    g.v3_rocket_elite_damage = Some(350);
    g.v3_warhead = Some("RKTW".into());
    g.v3_elite_warhead = Some("RKTW".into());
    if let Some(i) = r.sim.infantry.get_mut("WALKER") {
        i.tech_level = Some(1);
        i.owner = owners();
        i.cost = Some(200);
        i.sight = Some(5);
    }

    for i in 0..12 {
        let n = format!("ORE{i:02}");
        r.sim.overlays.insert(
            n.clone(),
            OverlayType {
                tiberium: Some(true),
                ..Default::default()
            },
        );
        push(&mut r, "overlays", &n);
    }
    r.sim.tiberiums.insert(
        "ORE".into(),
        TiberiumType {
            value: Some(25),
            growth: Some(100),
            growth_percentage: Some(Decimal("0.05".into())),
            spread: Some(100),
            spread_percentage: Some(Decimal("0.01".into())),
            image_run: Some([0, 12, 8]),
            ..Default::default()
        },
    );
    push(&mut r, "tiberiums", "ORE");
    r
}

/// The skirmish rules.
pub fn rules() -> Arc<SimRules> {
    Arc::new(SimRules::new(&ruleset(), "synthetic-skirmish"))
}

/// The cell (dx, dy) from the middle.
pub fn at(dx: i32, dy: i32) -> CellXY {
    testkit::mid(SIZE, dx, dy)
}

/// The skirmish map: flat clear ground, start waypoints 0 (west) and 1
/// (east), and an ore field of 5 × 5 cells beside each start.
pub fn map() -> Map {
    let mut m = testkit::empty_map(SIZE);
    let mut waypoints = BTreeMap::new();
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
    m
}

/// The second skirmish map: the same flat ground with the start waypoints
/// on a diagonal (0 north-west, 1 south-east), an ore field of 5 × 5 cells
/// beside each start and a third one in the middle, contested.
pub fn map_diagonal() -> Map {
    let mut m = testkit::empty_map(SIZE);
    let mut waypoints = BTreeMap::new();
    let d = START_DX * 2 / 3;
    let field = |m: &mut Map, cx: i32, cy: i32| {
        for ox in 0..5 {
            for oy in 0..5 {
                if let Some(cell) = m.cells.get_mut(at(cx + ox - 2, cy + oy - 2)) {
                    cell.overlay = ((ox * 3 + oy) % 12) as u8;
                    cell.overlay_data = 6;
                }
            }
        }
    };
    for (k, s) in [(0u16, -1), (1u16, 1)] {
        let c = at(s * d, s * d);
        waypoints.insert(k, (c.x as i16, c.y as i16));
        field(&mut m, s * d - s * 8, s * d + s * 2);
    }
    field(&mut m, 0, 0);
    m.waypoints = waypoints;
    m
}

/// The lobby setup (S-0412): two AI houses, Alpha and Beta, random start
/// positions, every other setting at the rules' default.
pub fn setup(seed: u32) -> GameSetup {
    let player = |name: &str, country: &str| Player {
        name: name.into(),
        country: country.into(),
        start: None,
        ai: true,
        values: Default::default(),
    };
    setup::skirmish(
        &rules(),
        &[player("Alpha", "Alphaland"), player("Beta", "Betaland")],
        Settings::default(),
        0,
        seed,
    )
}

/// The build plan both houses follow: types by role from the rules, two
/// harvesters, waves of six.
pub fn plan() -> Plan {
    Plan {
        names: Default::default(),
        harvesters: 2,
        wave: 6,
    }
}
