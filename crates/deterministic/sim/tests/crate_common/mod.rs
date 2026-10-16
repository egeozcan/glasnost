//! Shared fixtures of the S-0520 claim tests: a synthetic ruleset with crate
//! overlays, a power-up table, vehicles, tiberium and the combat rules of the
//! testkit. Every name and number is invented; no game data.
#![allow(dead_code)]

use glasnost_map::CellXY;
use glasnost_map::terrain::land;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{
    BuildingType, Decimal, LandRow, LandType, LocomotionClass, Locomotor, OverlayType, UnitType,
    WarheadType, WeaponType,
};
use glasnost_rules::sim_econ::{Powerup, TiberiumType};
use glasnost_sim::rules::Kind;
use glasnost_sim::testkit;
use glasnost_sim::world::{ObjectId, World};

pub const SIZE: i32 = 40;

/// The overlay list positions: 0 the wooden crate, 1 the water crate, 2 a
/// crate-trigger crate, 3 a plain overlay, 4 … 27 the ORE run (4, 12, 8).
pub const WOOD: u8 = 0;
pub const WATER: u8 = 1;
pub const TRIGGER: u8 = 2;
pub const PLAIN: u8 = 3;
pub const ORE: u8 = 4;

pub fn d(s: &str) -> Decimal {
    Decimal(s.into())
}

pub fn at(dx: i32, dy: i32) -> CellXY {
    testkit::mid(SIZE, dx, dy)
}

fn push(r: &mut Ruleset, reg: &str, n: &str) {
    let l = r.registries.entry(reg.into()).or_default();
    if !l.iter().any(|x| x == n) {
        l.push(n.into());
    }
}

/// The power-up table with every weight 0 and the arguments of the tests:
/// money 100, explosion 50, napalm 40, armour/speed/firepower 2.0, veteran 1,
/// gas 30; animations on money (MONEYA) only.
pub fn table(weights: &[(usize, i32)]) -> Vec<Powerup> {
    let mut t: Vec<Powerup> = (0..19)
        .map(|_| Powerup {
            weight: 0,
            animation: None,
            water: false,
            argument: Some(d("0")),
        })
        .collect();
    for (i, a) in [
        (0, "100"),
        (1, "0"),
        (4, "50"),
        (5, "40"),
        (9, "2.0"),
        (10, "2.0"),
        (11, "2.0"),
        (14, "1"),
        (16, "30"),
    ] {
        t[i].argument = Some(d(a));
    }
    for &(i, w) in weights {
        t[i].weight = w;
    }
    t[0].animation = Some("MONEYA".into());
    t
}

/// The ruleset. Crate overlays WCRATE (wooden), SCRATE (water), TCRATE
/// (crate trigger), a plain overlay, the ORE run; crate regeneration 1 minute
/// (lifetime 450 … 1800 ticks), radius 2.5 cells; vehicles GOODIE (crate
/// goodie, base unit), NOGOODIE, ARMED (weapon), UNARMED and a trainable
/// infantry; the demolition, flame and gas warheads.
pub fn ruleset() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    r.sim.land_types.insert(
        LandType::Clear,
        LandRow {
            buildable: Some(true),
            foot: Some(d("1")),
            track: Some(d("1")),
            wheel: Some(d("1")),
            ..Default::default()
        },
    );
    r.sim.land_types.insert(
        LandType::Water,
        LandRow {
            float: Some(d("1")),
            ..Default::default()
        },
    );
    r.sim
        .land_types
        .insert(LandType::Tiberium, LandRow::default());
    for (n, crate_, trig, tib) in [
        ("WCRATE", true, false, false),
        ("SCRATE", true, false, false),
        ("TCRATE", true, true, false),
        ("PLAINOV", false, false, false),
    ] {
        push(&mut r, "overlays", n);
        r.sim.overlays.insert(
            n.into(),
            OverlayType {
                has_art: Some(true),
                crate_: Some(crate_),
                crate_trigger: Some(trig),
                tiberium: Some(tib),
                ..Default::default()
            },
        );
    }
    for i in 0..24 {
        let n = format!("OV{i:02}");
        push(&mut r, "overlays", &n);
        r.sim.overlays.insert(
            n,
            OverlayType {
                has_art: Some(true),
                tiberium: Some(true),
                ..Default::default()
            },
        );
    }
    push(&mut r, "tiberiums", "ORE");
    r.sim.tiberiums.insert(
        "ORE".into(),
        TiberiumType {
            value: Some(25),
            growth: Some(100),
            growth_percentage: Some(d("1")),
            spread: Some(100),
            spread_percentage: Some(d("0")),
            image_run: Some([i32::from(ORE), 12, 8]),
            ..Default::default()
        },
    );
    for (n, f) in [("DEMO", false), ("FLAME", true), ("GAS", false)] {
        r.sim.warheads.insert(
            n.into(),
            WarheadType {
                verses: Some(vec![d("1"); 11]),
                cell_spread: Some(d(if f { "1" } else { "0.5" })),
                ..Default::default()
            },
        );
    }
    r.sim.weapons.insert(
        "CrateGun".into(),
        WeaponType {
            damage: Some(10),
            rof: Some(40),
            range: Some(d("5")),
            speed: Some(100),
            projectile: Some("Bullet".into()),
            warhead: Some("DEMO".into()),
            ..Default::default()
        },
    );
    let wheel = || UnitType {
        strength: Some(400),
        speed: Some(7),
        rot: Some(6),
        locomotion_class: Some(LocomotionClass::Wheel),
        locomotor: Some(Locomotor::Drive),
        tech_level: Some(1),
        ..Default::default()
    };
    for (n, goodie, gun) in [
        ("GOODIE", true, false),
        ("NOGOODIE", false, false),
        ("ARMED", false, true),
        ("UNARMED", false, false),
    ] {
        let mut u = wheel();
        u.crate_goodie = Some(goodie);
        if gun {
            u.primary = Some("CrateGun".into());
        }
        push(&mut r, "vehicles", n);
        r.sim.vehicles.insert(n.into(), u);
    }
    push(&mut r, "buildings", "BASE");
    r.sim.buildings.insert(
        "BASE".into(),
        BuildingType {
            strength: Some(800),
            foundation: Some([1, 1]),
            ..Default::default()
        },
    );
    let g = &mut r.sim.general;
    g.c4_warhead = Some("DEMO".into());
    let e = &mut g.economy;
    e.wood_crate_img = Some("WCRATE".into());
    e.water_crate_img = Some("SCRATE".into());
    e.crate_regen = Some(d("1"));
    e.crate_radius = Some(d("2.5"));
    e.flame_damage = Some("FLAME".into());
    e.powerups = Some(table(&[(0, 1)]));
    e.crate_minimum = Some(0);
    e.crate_maximum = Some(0);
    r
}

pub fn world(tweak: impl FnOnce(&mut Ruleset)) -> World {
    let mut r = ruleset();
    tweak(&mut r);
    testkit::game_with(&r, SIZE, 7)
}

/// A world whose crate setting is on (the testkit's sandbox has the setting
/// of the rules' lobby default; this switches it explicitly).
pub fn world_with(crates: bool, tweak: impl FnOnce(&mut Ruleset)) -> World {
    let mut w = world(tweak);
    w.econ.settings.crates = Some(crates);
    w.econ.start.bases = true;
    w
}

pub fn spawn(w: &mut World, kind: Kind, name: &str, house: u8, c: CellXY) -> ObjectId {
    testkit::spawn(w, kind, name, house, c, 0)
}

/// Puts a crate with `contents` and overlay `overlay` on a cell, with a place.
pub fn crate_at(w: &mut World, c: CellXY, overlay: u8, contents: u8) {
    if let Some(cell) = w.cells.grid.get_mut(c) {
        cell.overlay = overlay;
        cell.overlay_data = contents;
    }
}

pub fn credits(w: &World, h: u8) -> i32 {
    w.houses[usize::from(h)].credits
}

/// Turns the cells within `r` of `c` into water, with their categories and
/// regions.
pub fn make_water(w: &mut World, c: CellXY, r: i32) {
    let mut cells = Vec::new();
    for dx in -r..=r {
        for dy in -r..=r {
            let n = CellXY::new(c.x + dx, c.y + dy);
            if let Some(cell) = w.cells.grid.get_mut(n) {
                cell.land = land::WATER;
                cells.push(n);
            }
        }
    }
    for n in cells {
        let cat = glasnost_sim::passability::compute_category(w, n);
        if let Some(i) = w.cells.idx(n) {
            w.cells.category[i] = cat;
        }
    }
    glasnost_sim::regions::rebuild(w);
}

pub fn run_to(w: &mut World, tick: i32) {
    while w.tick < tick {
        testkit::run(w, 1);
    }
}
