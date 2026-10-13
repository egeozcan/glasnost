//! The synthetic skirmish: an invented ruleset and map (no game data) on
//! which two scripted houses build a base and fight, for the determinism
//! gate and the tests. Every number here is test data of our own, not game
//! behaviour.
//!
//! - Buildings: YARD (construction yard, 3 × 3, builds buildings), POWR
//!   (power plant, 2 × 2), PROC (dock-unload refinery, 3 × 3), PILL (barracks, 2 × 2),
//!   FACT (war factory, 3 × 3).
//! - Vehicles: MCV (deploys into YARD, the base unit), MINR (harvester,
//!   docks at PROC), GUNR (a gun vehicle); the testkit's CART, CRUSHER and
//!   BUGGY stay registered.
//! - Infantry: WALKER (the testkit's rifleman).
//! - Resource: ORE (overlays 0 … 11, stages 0 … 11).
//! - Map: a flat clear square of side [`SIZE`] (cell coordinates 0 … 2·SIZE
//!   on both axes), start waypoints 0 and 1 on opposite sides of the
//!   middle, and an ore field beside each.

use std::collections::BTreeMap;
use std::sync::Arc;

use glasnost_map::{CellXY, Map};
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{
    BuildingType, Decimal, LandRow, LandType, LocomotionClass, Locomotor, OverlayType, UnitType,
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
    for (n, b) in [
        ("YARD", yard),
        ("POWR", powr),
        ("PROC", proc_),
        ("PILL", pill),
        ("FACT", fact),
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
    for (n, u) in [("MCV", mcv), ("MINR", minr), ("GUNR", gunr)] {
        r.sim.vehicles.insert(n.into(), u);
        push(&mut r, "vehicles", n);
    }
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
