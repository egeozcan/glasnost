//! Shared synthetic rules of the S-0512 and S-0513 claim tests
//! (superweapons). Every name and number is invented; no game data.
//!
//! On top of the testkit's combat ruleset (CART, CRUSHER, BUGGY, WALKER,
//! TOWER, Gun, Rifle, Cannon, HE, SA):
//! - superweapon types, registered in this (non-alphabetical) list order:
//!   0 STRIKE (missile kind, action `Nuke`, upward weapon Lift),
//!   1 CURTAIN (invulnerability), 2 STORM (weather), 3 SOURCE (teleport
//!   source, first stage), 4 WARP (teleport warp, second stage linked to
//!   the teleport-source kind), 5 DROP (paradrop), 6 SPY (spy plane),
//!   7 SHIELD (force shield, start sound `Hum`, special sound `Fade`),
//!   8 DRAINER (drain mode, no kind), 9 MANUAL (manual charge, no kind),
//!   10 SHARED (no kind, action `Nuke` like STRIKE), 11 TIMED (no kind,
//!   not power-dependent, countdown flag, flash window 30, disableable,
//!   auxiliary building AUXB);
//!   every charge time is 0.1 minutes: 0.1 read in single precision is
//!   just above 0.1, so × 900 truncates to 90 ticks;
//! - buildings, one per superweapon: SILO (STRIKE, missile silo), IRONB,
//!   WEATHERB, CHRONOB (SOURCE and WARP), AIRB (DROP and SPY), SHIELDB,
//!   DRAINB, MANB, SHAREB, TIMEB; the auxiliary AUXB; PLANT (output 100);
//!   HOG (drain 500); BASE (one upgrade slot) and its upgrade UPG, whose
//!   first superweapon field names CURTAIN; UPG2, whose second field names
//!   CURTAIN;
//! - weapons Lift (upward, warhead Maker with the missile-maker flag,
//!   projectile UpShot with detonation altitude 900), NukePayload (falling,
//!   projectile Fall with detonation altitude 50 000, warhead HE), Camera (the spy plane's: damage 4,
//!   range 3);
//! - aircraft SPYP and PDPLANE (unarmed); paradrop lists: first side
//!   group WALKER ×1, WALKER ×1; second WALKER ×3; third JUMPER ×2;
//!   American WALKER ×1, JUMPER ×2;
//! - vehicle BLOB (organic); infantry JUMPER (organic and
//!   teleporter);
//! - storm values: countdown 30, duration 60, hit period 10, scatter
//!   period 5, spread 6, separation 2, damage 50, warhead Bolt, clouds
//!   CLOUD1 and CLOUD2 (7 image frames, no rate field so a stage delay of 1), bolt BOLT (canvas height 0), sounds Zap1 and Zap2;
//! - curtain duration 100; shield radius 3, duration 80, blackout 120,
//!   fade lead 20; siren `Siren`; invoke animations CURTA (curtain) and
//!   SHIELDA (force shield).

#![allow(dead_code)]

use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{
    AnimationType, BuildingType, Decimal, LocomotionClass, Locomotor, ProjectileType, UnitType,
    WarheadType, WeaponType,
};
use glasnost_rules::sim_econ::SuperweaponType;
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::Kind;
use glasnost_sim::superweapons::Record;
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, World};

/// Map size parameter of [`testkit::game_with`].
pub const SIZE: i32 = 20;
/// T of every type: trunc(0.1 in single precision × 900) = 90 (F1).
pub const T: i32 = 90;

pub const ALPHA: HouseId = HouseId(0);
pub const BETA: HouseId = HouseId(1);

pub const STRIKE: u32 = 0;
pub const CURTAIN: u32 = 1;
pub const STORM: u32 = 2;
pub const SOURCE: u32 = 3;
pub const WARP: u32 = 4;
pub const DROP: u32 = 5;
pub const SPY: u32 = 6;
pub const SHIELD: u32 = 7;
pub const DRAINER: u32 = 8;
pub const MANUAL: u32 = 9;
pub const SHARED: u32 = 10;
pub const TIMED: u32 = 11;

/// Curtain duration.
pub const CURTAIN_D: i32 = 100;
/// Storm values.
pub const DEFER: i32 = 30;
pub const STORM_D: i32 = 60;
pub const HIT: i32 = 10;
pub const SCATTER: i32 = 5;
/// Force-shield values.
pub const SHIELD_R: i32 = 3;
pub const SHIELD_D: i32 = 80;
pub const BLACKOUT: i32 = 120;
pub const FADE: i32 = 20;
/// The upward projectile's detonation altitude.
pub const UP_ALT: i32 = 900;
/// The falling projectile's detonation altitude (high, so that only the
/// ground ends its fall).
pub const FALL_ALT: i32 = 50_000;

fn reg(r: &mut Ruleset, registry: &str, name: &str) {
    r.registries
        .entry(registry.into())
        .or_default()
        .push(name.into());
}

fn d(s: &str) -> Decimal {
    Decimal(s.into())
}

fn sw(kind: Option<&str>, action: Option<&str>) -> SuperweaponType {
    SuperweaponType {
        kind: kind.map(Into::into),
        action: action.map(Into::into),
        recharge_time: Some(d("0.1")),
        ..Default::default()
    }
}

fn building(sw1: Option<&str>, sw2: Option<&str>) -> BuildingType {
    let mut b = BuildingType {
        strength: Some(500),
        foundation: Some([1, 1]),
        ..Default::default()
    };
    b.econ.super_weapon = sw1.map(Into::into);
    b.econ.super_weapon2 = sw2.map(Into::into);
    b
}

/// The superweapon ruleset.
pub fn ruleset() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    let mut types: Vec<(&str, SuperweaponType)> = vec![
        ("STRIKE", {
            let mut t = sw(Some("MultiMissile"), Some("Nuke"));
            t.weapon_type = Some("Lift".into());
            t
        }),
        ("CURTAIN", sw(Some("IronCurtain"), Some("IronCurtain"))),
        ("STORM", sw(Some("LightningStorm"), Some("LightningStorm"))),
        ("SOURCE", {
            let mut t = sw(Some("ChronoSphere"), Some("ChronoSphere"));
            t.pre_click = Some(true);
            t
        }),
        ("WARP", {
            let mut t = sw(Some("ChronoWarp"), Some("ChronoWarp"));
            t.post_click = Some(true);
            t.pre_dependent = Some("ChronoSphere".into());
            t
        }),
        ("DROP", sw(Some("Paradrop"), Some("ParaDrop"))),
        ("SPY", sw(Some("SpyPlane"), Some("SpyPlane"))),
        ("SHIELD", {
            let mut t = sw(Some("ForceShield"), Some("ForceShield"));
            t.start_sound = Some("Hum".into());
            t.special_sound = Some("Fade".into());
            t
        }),
        ("DRAINER", {
            let mut t = sw(None, Some("Sabotage"));
            t.use_charge_drain = Some(true);
            t.flash_sidebar_tab_frames = Some(30);
            t
        }),
        ("MANUAL", {
            let mut t = sw(None, Some("Tote"));
            t.manual_control = Some(true);
            t
        }),
        ("SHARED", sw(None, Some("nuke"))),
        ("TIMED", {
            let mut t = sw(None, Some("Demolish"));
            t.is_powered = Some(false);
            t.show_timer = Some(true);
            t.flash_sidebar_tab_frames = Some(30);
            t.disableable_from_shell = Some(true);
            t.aux_building = Some("AUXB".into());
            t
        }),
    ];
    for (n, t) in types.drain(..) {
        r.sim.superweapons.insert(n.into(), t);
        reg(&mut r, "superweapons", n);
    }
    {
        let g = &mut r.sim.general.superweapons;
        g.lightning_deferment = Some(DEFER);
        g.lightning_storm_duration = Some(STORM_D);
        g.lightning_hit_delay = Some(HIT);
        g.lightning_scatter_delay = Some(SCATTER);
        g.lightning_cell_spread = Some(6);
        g.lightning_separation = Some(2);
        g.lightning_damage = Some(50);
        g.lightning_warhead = Some("Bolt".into());
        g.weather_con_clouds = Some(vec!["CLOUD1".into(), "CLOUD2".into()]);
        g.weather_con_bolts = Some(vec!["BOLT".into()]);
        g.lightning_sounds = Some(vec!["Zap1".into(), "Zap2".into()]);
        g.iron_curtain_duration = Some(CURTAIN_D);
        g.force_shield_radius = Some(SHIELD_R);
        g.force_shield_duration = Some(SHIELD_D);
        g.force_shield_blackout_duration = Some(BLACKOUT);
        g.force_shield_play_fade_sound_time = Some(FADE);
        g.dig_sound = Some("Siren".into());
        g.iron_curtain_invoke_anim = Some("CURTA".into());
        g.force_shield_invoke_anim = Some("SHIELDA".into());
        g.ally_para_drop_inf = Some(vec!["WALKER".into(), "WALKER".into()]);
        g.ally_para_drop_num = Some(vec![1, 1]);
        g.sov_para_drop_inf = Some(vec!["WALKER".into()]);
        g.sov_para_drop_num = Some(vec![3]);
        g.yuri_para_drop_inf = Some(vec!["JUMPER".into()]);
        g.yuri_para_drop_num = Some(vec![2]);
        g.amer_para_drop_inf = Some(vec!["WALKER".into(), "JUMPER".into()]);
        g.amer_para_drop_num = Some(vec![1, 2]);
    }
    let mut silo = building(Some("STRIKE"), None);
    silo.econ.nuke_silo = Some(true);
    let mut plant = building(None, None);
    plant.econ.power = Some(100);
    let mut hog = building(None, None);
    hog.econ.power = Some(-500);
    let mut base = building(None, None);
    base.econ.upgrades = Some(1);
    let mut upg = building(Some("CURTAIN"), None);
    upg.econ.powers_up_building = Some("BASE".into());
    let mut upg2 = building(None, Some("CURTAIN"));
    upg2.econ.powers_up_building = Some("BASE".into());
    for (n, b) in [
        ("SILO", silo),
        ("IRONB", building(Some("CURTAIN"), None)),
        ("WEATHERB", building(Some("STORM"), None)),
        ("CHRONOB", building(Some("SOURCE"), Some("WARP"))),
        ("AIRB", building(Some("DROP"), Some("SPY"))),
        ("SHIELDB", building(Some("SHIELD"), None)),
        ("DRAINB", building(Some("DRAINER"), None)),
        ("MANB", building(Some("MANUAL"), None)),
        ("SHAREB", building(Some("SHARED"), None)),
        ("TIMEB", building(Some("TIMED"), None)),
        ("AUXB", building(None, None)),
        ("PLANT", plant),
        ("HOG", hog),
        ("BASE", base),
        ("UPG", upg),
        ("UPG2", upg2),
    ] {
        r.sim.buildings.insert(n.into(), b);
        reg(&mut r, "buildings", n);
    }
    let verses = || Some(vec![d("1"); 11]);
    r.sim.warheads.insert(
        "Maker".into(),
        WarheadType {
            nuke_maker: Some(true),
            verses: verses(),
            ..Default::default()
        },
    );
    r.sim.warheads.insert(
        "Bolt".into(),
        WarheadType {
            cell_spread: Some(d("1")),
            verses: verses(),
            ..Default::default()
        },
    );
    r.sim.projectiles.insert(
        "UpShot".into(),
        ProjectileType {
            vertical: Some(true),
            detonation_altitude: Some(UP_ALT),
            ..Default::default()
        },
    );
    r.sim.projectiles.insert(
        "Fall".into(),
        ProjectileType {
            vertical: Some(true),
            detonation_altitude: Some(FALL_ALT),
            ..Default::default()
        },
    );
    for (n, wt) in [
        (
            "Lift",
            WeaponType {
                damage: Some(100),
                speed: Some(255),
                range: Some(d("1")),
                projectile: Some("UpShot".into()),
                warhead: Some("Maker".into()),
                ..Default::default()
            },
        ),
        (
            "NukePayload",
            WeaponType {
                damage: Some(300),
                speed: Some(40),
                range: Some(d("1")),
                projectile: Some("Fall".into()),
                warhead: Some("HE".into()),
                ..Default::default()
            },
        ),
        (
            "Camera",
            WeaponType {
                damage: Some(4),
                range: Some(d("3")),
                projectile: Some("Bullet".into()),
                warhead: Some("SA".into()),
                ..Default::default()
            },
        ),
    ] {
        r.sim.weapons.insert(n.into(), wt);
    }
    let mut spyp = UnitType {
        strength: Some(100),
        speed: Some(30),
        rot: Some(12),
        locomotor: Some(Locomotor::Fly),
        primary: Some("Camera".into()),
        ..Default::default()
    };
    spyp.sight = Some(2);
    r.sim.aircraft.insert("SPYP".into(), spyp);
    reg(&mut r, "aircraft", "SPYP");
    r.sim.aircraft.insert(
        "PDPLANE".into(),
        UnitType {
            strength: Some(100),
            speed: Some(30),
            rot: Some(12),
            locomotor: Some(Locomotor::Fly),
            ..Default::default()
        },
    );
    reg(&mut r, "aircraft", "PDPLANE");
    let mut blob = UnitType {
        strength: Some(200),
        speed: Some(6),
        rot: Some(6),
        locomotion_class: Some(LocomotionClass::Wheel),
        locomotor: Some(Locomotor::Drive),
        ..Default::default()
    };
    blob.combat.organic = Some(true);
    r.sim.vehicles.insert("BLOB".into(), blob);
    reg(&mut r, "vehicles", "BLOB");
    let mut jumper = UnitType {
        strength: Some(100),
        speed: Some(4),
        locomotion_class: Some(LocomotionClass::Foot),
        locomotor: Some(Locomotor::Walk),
        teleporter: Some(true),
        ..Default::default()
    };
    jumper.combat.organic = Some(true);
    r.sim.infantry.insert("JUMPER".into(), jumper);
    reg(&mut r, "infantry", "JUMPER");
    for n in ["CURTA", "SHIELDA"] {
        r.sim.animations.insert(n.into(), AnimationType::default());
        reg(&mut r, "animations", n);
    }
    for n in ["CLOUD1", "CLOUD2", "BOLT"] {
        r.sim.animations.insert(
            n.into(),
            AnimationType {
                middle_frame: Some(3),
                // 7 image frames of canvas height 0 (S-0513 claim 46, F4)
                frame_count: Some(7),
                canvas_height: Some(0),
                ..Default::default()
            },
        );
        reg(&mut r, "animations", n);
    }
    r
}

/// A new game under `r`, run to tick 4, when orders open (S-0213 claim 1).
pub fn game(r: &Ruleset) -> World {
    let mut w = testkit::game_with(r, SIZE, 1);
    while orders::blocked(&w) {
        glasnost_sim::tick::step(&mut w);
    }
    w
}

/// A new game like [`game`] on a map whose usable area leaves a band of
/// legal cells, two wide, around it (as `banded_game` of claims_s0501.rs),
/// so that the edge cells lie outside the usable area (S-0522 claim 4) and
/// S-0501 claim 10's removals outside the usable area can happen before the
/// map's edge.
pub fn banded_game(r: &Ruleset) -> World {
    let mut m = testkit::empty_map(SIZE);
    let fp = glasnost_map::Footprint::new(SIZE, SIZE, [2, 2, SIZE - 4, SIZE - 4]);
    let mut cells = glasnost_map::CellGrid::new(fp);
    let all: Vec<_> = cells.iter().map(|(c, _)| c).collect();
    for c in all {
        if let (Some(cell), Some(old)) = (cells.get_mut(c), m.cells.get(c)) {
            *cell = *old;
        }
    }
    m.footprint = fp;
    m.cells = cells;
    let rules = std::sync::Arc::new(glasnost_sim::rules::SimRules::new(r, "synthetic-test"));
    let mut w = glasnost_sim::load::new_game(rules, &m, testkit::catalogue(), &testkit::setup(1))
        .expect("synthetic test game loads");
    while orders::blocked(&w) {
        glasnost_sim::tick::step(&mut w);
    }
    w
}

/// The map's middle cell offset by (dx, dy).
pub fn at(dx: i32, dy: i32) -> CellXY {
    testkit::mid(SIZE, dx, dy)
}

/// Spawns `name` (its kind looked up) for house `h` at `at(dx, dy)`.
pub fn spawn(w: &mut World, name: &str, h: HouseId, dx: i32, dy: i32) -> ObjectId {
    let kind = [
        Kind::Vehicle,
        Kind::Infantry,
        Kind::Building,
        Kind::Aircraft,
    ]
    .into_iter()
    .find(|&k| w.rules().index_of(k, name).is_some())
    .unwrap_or_else(|| panic!("no type {name}"));
    testkit::spawn(w, kind, name, h.0, at(dx, dy), 0)
}

/// House `h`'s record of type `i` (a default record when it has none).
pub fn rec(w: &World, h: HouseId, i: u32) -> Record {
    glasnost_sim::superweapons::charge::get(w, h, i).unwrap_or_default()
}

/// One tick.
pub fn step(w: &mut World) {
    glasnost_sim::tick::step(w);
}

/// Runs `n` ticks.
pub fn run(w: &mut World, n: i32) {
    testkit::run(w, n);
}

/// Runs ticks until `f` holds (at most `max`); returns the tick number then.
pub fn until(w: &mut World, max: i32, mut f: impl FnMut(&World) -> bool) -> i32 {
    for _ in 0..max {
        if f(w) {
            return w.tick;
        }
        step(w);
    }
    assert!(f(w), "condition not reached in {max} ticks");
    w.tick
}

/// The tick in which house `h`'s record `i` becomes ready (runs up to
/// `max` ticks).
pub fn ready_tick(w: &mut World, h: HouseId, i: u32, max: i32) -> i32 {
    until(w, max, |w| rec(w, h, i).ready)
}

/// Puts a fire order into house `h`'s pending list; it is applied in the
/// next tick's order step (S-0203 claim 7).
pub fn fire(w: &mut World, h: HouseId, i: u32, c: CellXY) {
    let ok = orders::issue(w, h, Order::FireSuperweapon { sw: i, cell: c });
    assert!(ok, "fire order not taken");
}

/// Fires and steps one tick, so the order is applied.
pub fn fire_now(w: &mut World, h: HouseId, i: u32, c: CellXY) {
    fire(w, h, i, c);
    step(w);
}

/// Makes `a` ally `b` (one way: `a`'s view).
pub fn ally(w: &mut World, a: HouseId, b: HouseId) {
    let h = &mut w.houses[usize::from(a.0)];
    if !h.allies.contains(&b) {
        h.allies.push(b);
        h.allies.sort();
    }
}

/// Switches a building off or on through the power switch's event.
pub fn switch(w: &mut World, b: ObjectId, on: bool) {
    w.obj_mut(b).unwrap().econ.building.switched_off = !on;
    glasnost_sim::econ::events::switched(w, b);
}

/// Removes an object from play for good (as a sale's end would).
pub fn remove(w: &mut World, id: ObjectId) {
    glasnost_sim::objects::remove(w, id);
}

/// A killing hit with no attacker.
pub fn kill(w: &mut World, id: ObjectId) {
    glasnost_sim::combat::damage::apply(
        w,
        id,
        glasnost_sim::combat::damage::Hit {
            amount: 100_000,
            warhead: None,
            distance: 0,
            attacker: None,
            house: None,
            ignore_defences: true,
            no_escape: true,
        },
    );
}

/// The superweapon events pushed since the effects were last cleared.
pub fn events(
    w: &World,
) -> Vec<(
    glasnost_sim::superweapons::SwEvent,
    Option<HouseId>,
    Option<u32>,
)> {
    w.effects
        .iter()
        .filter_map(|e| match e {
            glasnost_sim::combat::effects::Effect::Superweapon {
                event, house, sw, ..
            } => Some((*event, *house, *sw)),
            _ => None,
        })
        .collect()
}

/// The animation names created since the effects were last cleared.
pub fn anims(w: &World) -> Vec<String> {
    w.effects
        .iter()
        .filter_map(|e| match e {
            glasnost_sim::combat::effects::Effect::Anim { name, .. } => Some(name.clone()),
            _ => None,
        })
        .collect()
}
