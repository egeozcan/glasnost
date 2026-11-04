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
//! - Aircraft: RKT (the set-A rocket type, S-0506), JET (an armed plane,
//!   two rounds, docks at HPAD); HPAD is a 2 × 2 aircraft factory, helipad
//!   and reload pad with one docking place, needing RADR.
//! - Infantry: WALKER (the testkit's rifleman).
//! - Resource: ORE (overlays 0 … 11, stages 0 … 11).
//! - Naval: NAVY (naval yard, 3 × 3, needs FACT) and BOAT (a gunboat with
//!   the ship locomotor, float class and water zone); water is open only
//!   to the float, hover and amphibious classes, ground closed to float.
//! - Superweapon (S-0512, S-0513): NUKE (the missile kind, charge 1
//!   minute = 900 ticks), granted by SILO (a 2 × 2 missile silo, needs
//!   LAB); the upward weapon LIFT (warhead MAKER with the missile-maker
//!   flag, projectile UPSHOT) and the falling NukePayload (projectile
//!   FALL, warhead RKTW).
//! - Hover: HOVR (a gun hovercraft with the hovering locomotor and class
//!   and the amphibious zone, built at FACT once NAVY and LAB stand; S-0503).
//! - Transports (S-0507): TRUK (an unarmed wheeled transport for four,
//!   size limit 2) and LIFT (the same, amphibious and hovering), both
//!   built at FACT.
//! - Map: a flat clear square of side [`SIZE`] (cell coordinates 0 … 2·SIZE
//!   on both axes), start waypoints 0 and 1 on opposite sides of the
//!   middle, and an ore field beside each; [`map_diagonal`] puts the starts
//!   on a diagonal with a third ore field between them; [`map_naval`] adds a
//!   sea along the north side; [`map_strait`] a strait between the starts.
//!   [`catalogue`] has the water tile.

use std::collections::BTreeMap;
use std::sync::Arc;

use glasnost_map::terrain::{Roles, TileCatalogue, TileCell, TileInfo};
use glasnost_map::{CellXY, Map};
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{
    BuildingType, CombatFields, Decimal, LandRow, LandType, LocomotionClass, Locomotor,
    MovementZone, OverlayType, ProjectileType, UnitType, WarheadType, WeaponType,
};
use glasnost_rules::sim_econ::{SuperweaponType, TiberiumType};
use glasnost_sim::econ::setup::{self, Player, Settings};
use glasnost_sim::rules::SimRules;
use glasnost_sim::testkit;
use glasnost_sim::world::GameSetup;

use crate::scripted::Plan;

/// Half the map's side: the middle cell is (SIZE, SIZE).
pub const SIZE: i32 = 40;

/// The start offset from the middle along x.
pub const START_DX: i32 = 22;

fn dec(s: &str) -> Option<Decimal> {
    Some(Decimal(s.into()))
}

fn owners() -> Option<Vec<String>> {
    Some(vec!["Alphaland".into(), "Betaland".into()])
}

fn push(r: &mut Ruleset, reg: &str, n: &str) {
    let v = r.registries.entry(reg.into()).or_default();
    if !v.iter().any(|x| x == n) {
        v.push(n.into());
    }
}

/// A building type of the synthetic skirmish: buildable by both sides at
/// tech level 1, with the given foundation, cost and power.
pub fn building(w: i32, h: i32, cost: i32, power: i32) -> BuildingType {
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
    // Ground rows closed to the float class, so ships keep to the water.
    s.land_types.insert(
        LandType::Clear,
        LandRow {
            buildable: Some(true),
            float: dec("0"),
            float_beach: dec("0"),
            ..Default::default()
        },
    );
    s.land_types.insert(
        LandType::Tiberium,
        LandRow {
            float: dec("0"),
            float_beach: dec("0"),
            ..Default::default()
        },
    );
    // Water: open to the float, hover and amphibious classes only.
    s.land_types.insert(
        LandType::Water,
        LandRow {
            foot: dec("0"),
            track: dec("0"),
            wheel: dec("0"),
            ..Default::default()
        },
    );
    s.general.wake = Some("Ripple".into());
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
    let mut hpad = building(2, 2, 800, -20);
    hpad.factory = Some("aircraft".into());
    hpad.helipad = Some(true);
    hpad.econ.unit_reload = Some(true);
    hpad.econ.number_of_docks = Some(1);
    hpad.econ.prerequisite = Some(vec!["RADR".into()]);
    let mut navy = building(3, 3, 1000, -20);
    navy.factory = Some("vehicle".into());
    navy.naval = Some(true);
    navy.econ.prerequisite = Some(vec!["FACT".into()]);
    let mut silo = building(2, 2, 1500, -50);
    silo.econ.prerequisite = Some(vec!["LAB".into()]);
    silo.econ.super_weapon = Some("NUKE".into());
    silo.econ.nuke_silo = Some(true);
    for (n, b) in [
        ("YARD", yard),
        ("POWR", powr),
        ("PROC", proc_),
        ("PILL", pill),
        ("FACT", fact),
        ("RADR", radr),
        ("LAB", lab),
        ("NAVY", navy),
        ("HPAD", hpad),
        ("SILO", silo),
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
    let mut boat = vehicle(600, 6, 900);
    boat.locomotion_class = Some(LocomotionClass::Float);
    boat.locomotor = Some(Locomotor::Ship);
    boat.movement_zone = Some(MovementZone::Water);
    boat.primary = Some("Gun".into());
    boat.turret = Some(true);
    boat.weight = dec("4");
    boat.combat.naval = Some(true);
    boat.combat.primary_fire_flh = Some([90, 0, 70]);
    // A hovercraft (S-0503): the hovering locomotor and class, the
    // amphibious movement zone, built at the land vehicle factory once the
    // house has a naval yard (so only on maps with water) and a LAB (so the
    // naval game still reaches its launchers before hovers join the army).
    let mut hovr = vehicle(350, 8, 800);
    hovr.econ.prerequisite = Some(vec!["NAVY".into(), "LAB".into()]);
    hovr.locomotion_class = Some(LocomotionClass::Hover);
    hovr.locomotor = Some(Locomotor::Hover);
    hovr.movement_zone = Some(MovementZone::Amphibious);
    hovr.primary = Some("Gun".into());
    hovr.turret = Some(true);
    hovr.combat.primary_fire_flh = Some([80, 0, 60]);
    // An unarmed wheeled transport for four (S-0507), and an unarmed
    // hovering one that crosses water too (S-0503): the AI ferries
    // infantry in the cheaper one over land, in the hovering one across
    // water.
    let mut truk = vehicle(500, 7, 600);
    truk.passengers = Some(4);
    truk.size_limit = dec("2");
    truk.econ.prerequisite = Some(vec!["RADR".into()]);
    let mut lift = vehicle(500, 7, 900);
    lift.econ.prerequisite = Some(vec!["RADR".into()]);
    lift.locomotion_class = Some(LocomotionClass::Hover);
    lift.locomotor = Some(Locomotor::Hover);
    lift.movement_zone = Some(MovementZone::Amphibious);
    lift.passengers = Some(4);
    lift.size_limit = dec("2");
    for (n, u) in [
        ("MCV", mcv),
        ("MINR", minr),
        ("GUNR", gunr),
        ("LNCH", lnch),
        ("BOAT", boat),
        ("HOVR", hovr),
        ("TRUK", truk),
        ("LIFT", lift),
    ] {
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
    let mut jet = UnitType {
        strength: Some(200),
        speed: Some(30),
        rot: Some(12),
        locomotor: Some(Locomotor::Fly),
        tech_level: Some(1),
        owner: owners(),
        cost: Some(1000),
        sight: Some(8),
        primary: Some("Missile".into()),
        ..Default::default()
    };
    jet.combat.ammo = Some(2);
    jet.econ.landable = Some(true);
    jet.econ.dock = Some(vec!["HPAD".into()]);
    r.sim.aircraft.insert("JET".into(), jet);
    // The re-approach flag (S-0501 claim 14): an ordinary attacker ends each
    // run, spends its round and so flies back to rearm.
    r.sim.general.economy.curley_shuffle = Some(true);
    push(&mut r, "aircraft", "JET");
    r.sim.weapons.insert(
        "Missile".into(),
        WeaponType {
            damage: Some(60),
            rof: Some(30),
            range: dec("6"),
            speed: Some(60),
            projectile: Some("Rocket".into()),
            warhead: Some("HE".into()),
            ..Default::default()
        },
    );
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
    superweapon(&mut r);
    r
}

/// The superweapon of the skirmish: NUKE, its weapons, warhead and
/// projectiles (test data of our own).
fn superweapon(r: &mut Ruleset) {
    r.sim.superweapons.insert(
        "NUKE".into(),
        SuperweaponType {
            kind: Some("MultiMissile".into()),
            action: Some("Nuke".into()),
            recharge_time: dec("1"),
            weapon_type: Some("LIFT".into()),
            ..Default::default()
        },
    );
    push(r, "superweapons", "NUKE");
    r.sim.warheads.insert(
        "MAKER".into(),
        WarheadType {
            nuke_maker: Some(true),
            verses: Some((0..11).map(|_| Decimal("1".into())).collect()),
            ..Default::default()
        },
    );
    for (n, alt) in [("UPSHOT", 900), ("FALL", 50_000)] {
        r.sim.projectiles.insert(
            n.into(),
            ProjectileType {
                vertical: Some(true),
                detonation_altitude: Some(alt),
                ..Default::default()
            },
        );
    }
    for (n, damage, speed, projectile, warhead) in [
        ("LIFT", 100, 255, "UPSHOT", "MAKER"),
        ("NukePayload", 300, 40, "FALL", "RKTW"),
    ] {
        r.sim.weapons.insert(
            n.into(),
            WeaponType {
                damage: Some(damage),
                speed: Some(speed),
                range: dec("1"),
                projectile: Some(projectile.into()),
                warhead: Some(warhead.into()),
                ..Default::default()
            },
        );
    }
}

/// The skirmish rules.
pub fn rules() -> Arc<SimRules> {
    Arc::new(SimRules::new(&ruleset(), "synthetic-skirmish"))
}

/// The water tile of [`catalogue`].
pub const WATER_TILE: i32 = 5;

/// The terrain code of a water tile (S-0211 Terms).
const WATER_CODE: u8 = 9;

/// The skirmish catalogue: the testkit's tiles 0 to 4 (clear, flat and
/// ramps) and tile [`WATER_TILE`], a flat water tile where the water role
/// starts, so that only it takes a naval yard (S-0406 claim 3 d).
pub fn catalogue() -> Arc<TileCatalogue> {
    let mut c = (*testkit::catalogue()).clone();
    c.tiles.push(TileInfo {
        width: 1,
        height: 1,
        has_image: true,
        cells: vec![Some(TileCell {
            terrain_code: WATER_CODE,
            slope: 0,
        })],
        variants: 0,
        anim: None,
        no_burrowing: false,
    });
    c.roles = Roles {
        water: WATER_TILE,
        ..c.roles
    };
    c.allow_tiberium.push(false);
    Arc::new(c)
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

/// The cramped map: [`map`] with start 0 in a small clearing (the cells
/// within `pocket` cells of the start) inside a moat of water four
/// cells wide, left open by a land bridge of three cells to the east, so
/// that the clearing is too small for the plan's buildings with the usual
/// gap between them.
pub fn map_cramped(pocket: i32) -> Map {
    let mut m = map();
    let s = at(-START_DX, 0);
    let cells: Vec<CellXY> = m.cells.iter().map(|(c, _)| c).collect();
    for c in cells {
        let d = (c.x - s.x).abs().max((c.y - s.y).abs());
        let bridge = (c.y - s.y).abs() <= 1 && c.x > s.x;
        if d > pocket
            && d <= pocket + 4
            && !bridge
            && let Some(cell) = m.cells.get_mut(c)
        {
            cell.tile = WATER_TILE;
            cell.overlay = glasnost_map::NO_OVERLAY;
        }
    }
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

/// The naval skirmish map: [`map`] with a sea of water tiles along its
/// north side, from [`SEA_DY`] cells north of the starts to the edge, within
/// reach of both bases' building spirals.
pub fn map_naval() -> Map {
    let mut m = map();
    let shore = at(0, SEA_DY).y;
    let cells: Vec<CellXY> = m.cells.iter().map(|(c, _)| c).collect();
    for c in cells {
        if c.y <= shore
            && let Some(cell) = m.cells.get_mut(c)
        {
            cell.tile = WATER_TILE;
        }
    }
    m
}

/// The offset along y, from the middle, of the sea's southernmost row in
/// [`map_naval`].
pub const SEA_DY: i32 = -9;

/// Half the width of the strait of [`map_strait`], in cells.
pub const STRAIT_HALF: i32 = 4;

/// The strait map: [`map`] with a strait of water tiles from the north
/// edge to the south edge between the starts (the columns within
/// [`STRAIT_HALF`] cells of the middle), so that no ground unit walks from
/// one base to the other: infantry crosses by transport.
pub fn map_strait() -> Map {
    let mut m = map();
    let mid = at(0, 0).x;
    let cells: Vec<CellXY> = m.cells.iter().map(|(c, _)| c).collect();
    for c in cells {
        if (c.x - mid).abs() <= STRAIT_HALF
            && let Some(cell) = m.cells.get_mut(c)
        {
            cell.tile = WATER_TILE;
        }
    }
    m
}

/// The island map: [`map_naval`] with [`map_strait`]'s strait from the
/// shore to the south edge, so that each start stands on its own island
/// and no ground unit walks from one base to the other: the sea along
/// the north side and the strait join, and the ships meet there. Air
/// units and transports still cross (M8 step B9: the naval game is
/// played here, so that a land war cannot end it first).
pub fn map_islands() -> Map {
    let mut m = map_naval();
    let mid = at(0, 0).x;
    let cells: Vec<CellXY> = m.cells.iter().map(|(c, _)| c).collect();
    for c in cells {
        if (c.x - mid).abs() <= STRAIT_HALF
            && let Some(cell) = m.cells.get_mut(c)
        {
            cell.tile = WATER_TILE;
            cell.overlay = glasnost_map::NO_OVERLAY;
        }
    }
    m
}

/// The lobby setup (S-0412): two AI houses, Alpha at start 0 (west) and
/// Beta at start 1 (east), every other setting at the rules' default.
/// The tests' cells are laid out for that order. Until intake 87 both
/// starts were random, and seed 5 happened to give this order; S-0202
/// claim 14's house draws now come before the position draw, so the
/// starts are fixed instead of left to the seed.
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
