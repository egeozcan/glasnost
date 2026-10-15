//! Shared synthetic rules of the S-0516 claim tests (cloak and stealth).
//! Every name and number is invented; no game data.
//!
//! On top of the testkit's combat ruleset (CART, CRUSHER, BUGGY, WALKER,
//! TOWER, Gun, Rifle, Cannon, HE, SA):
//! - game-wide: N = 8 stages, a cloak delay of 0.01 minutes (8 ticks: 0.01 is
//!   read in single precision, just below 0.01, so × 900 truncates to 8), the
//!   cloak sound `Shimmer`, the allied default disguise WALKER, the
//!   default-disguise list [SHRUB, BUSH], an infantry-blink time of 5;
//! - GHOST: an unarmed vehicle with the cloak flag, cloaking speed 2;
//! - GUNSHIP: GHOST armed with the Gun (decloak to fire);
//! - SHADE: an unarmed infantry type with the cloak flag, speed 2;
//! - PLAIN: an unarmed vehicle that cannot cloak by itself (cloaking speed
//!   2 inside a field);
//! - SEEKER: an unarmed vehicle with the sensors flag and sensors-sight 3;
//! - MIRAGE: a vehicle that can disguise and disguises when still, armed
//!   with the Flash (fake-blink time 6);
//! - SPY: an infantry type that disguises permanently;
//! - COPIER: an infantry type armed with the Mimic, whose warhead has the
//!   disguise flag;
//! - buildings GENNY (cloak generator, radius 3), ARRAY (sensor array,
//!   sensors-sight 4), DETECTOR (disguise detection, range 3), VEIL (a
//!   building with the cloak flag), WARF (a war factory);
//! - WRAITH: a landable aircraft with the cloak flag, speed 2;
//! - terrain-object types SHRUB and BUSH.

#![allow(dead_code)]

use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{
    BuildingType, Decimal, LocomotionClass, Locomotor, TerrainObjectType, UnitType, WarheadType,
    WeaponType,
};
use glasnost_sim::cloak::CloakState;
use glasnost_sim::rules::Kind;
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, World};

/// Map size parameter of [`testkit::game_with`].
pub const SIZE: i32 = 20;
/// N, the stage count of the ruleset.
pub const STAGES: i32 = 8;
/// S, the cloaking speed of the cloaking types.
pub const SPEED: i32 = 2;
/// D in ticks: trunc(0.01 minutes, read in single precision, × 900) = 8
/// (S-0516 F3).
pub const DELAY: i32 = 8;
/// The generator's cloak radius.
pub const GEN_RADIUS: i32 = 3;
/// The sensor vehicle's sensors-sight.
pub const SEEKER_SIGHT: i32 = 3;
/// The sensor array's sensors-sight.
pub const ARRAY_SIGHT: i32 = 4;
/// The detector's detection range.
pub const DETECT_RANGE: i32 = 3;
/// The Flash's fake-blink time.
pub const BLINK: i32 = 6;
/// The infantry-blink time.
pub const INFANTRY_BLINK: i32 = 5;

pub const ALPHA: HouseId = HouseId(0);
pub const BETA: HouseId = HouseId(1);

fn reg(r: &mut Ruleset, registry: &str, name: &str) {
    r.registries
        .entry(registry.into())
        .or_default()
        .push(name.into());
}

fn unarmed_vehicle() -> UnitType {
    UnitType {
        strength: Some(200),
        speed: Some(6),
        rot: Some(6),
        locomotion_class: Some(LocomotionClass::Wheel),
        locomotor: Some(Locomotor::Drive),
        ..Default::default()
    }
}

fn unarmed_infantry() -> UnitType {
    UnitType {
        strength: Some(100),
        speed: Some(4),
        locomotion_class: Some(LocomotionClass::Foot),
        locomotor: Some(Locomotor::Walk),
        ..Default::default()
    }
}

/// The cloak ruleset.
pub fn ruleset() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    let d = |s: &str| Decimal(s.into());
    {
        let g = &mut r.sim.general;
        g.cloaking_stages = Some(STAGES);
        g.cloak_delay = Some(d("0.01"));
        g.cloak_sound = Some("Shimmer".into());
        g.allied_disguise = Some("WALKER".into());
        g.default_mirage_disguises = Some(vec!["SHRUB".into(), "BUSH".into()]);
        g.infantry_blink_disguise_time = Some(INFANTRY_BLINK);
    }
    let mut ghost = unarmed_vehicle();
    ghost.combat.cloakable = Some(true);
    ghost.combat.cloaking_speed = Some(SPEED);
    let mut gunship = ghost.clone();
    gunship.primary = Some("Gun".into());
    gunship.combat.primary_fire_flh = Some([90, 0, 70]);
    let mut plain = unarmed_vehicle();
    plain.combat.cloaking_speed = Some(SPEED);
    let mut seeker = unarmed_vehicle();
    seeker.combat.sensors = Some(true);
    seeker.combat.sensors_sight = Some(SEEKER_SIGHT);
    let mut mirage = unarmed_vehicle();
    mirage.combat.can_disguise = Some(true);
    mirage.combat.disguise_when_still = Some(true);
    mirage.primary = Some("Flash".into());
    mirage.combat.primary_fire_flh = Some([90, 0, 70]);
    for (n, u) in [
        ("GHOST", ghost),
        ("GUNSHIP", gunship),
        ("PLAIN", plain),
        ("SEEKER", seeker),
        ("MIRAGE", mirage),
    ] {
        r.sim.vehicles.insert(n.into(), u);
        reg(&mut r, "vehicles", n);
    }
    let mut shade = unarmed_infantry();
    shade.combat.cloakable = Some(true);
    shade.combat.cloaking_speed = Some(SPEED);
    let mut spy = unarmed_infantry();
    spy.combat.can_disguise = Some(true);
    spy.perma_disguise = Some(true);
    let mut copier = unarmed_infantry();
    copier.combat.can_disguise = Some(true);
    copier.primary = Some("Mimic".into());
    copier.combat.primary_fire_flh = Some([30, 0, 50]);
    for (n, u) in [("SHADE", shade), ("SPY", spy), ("COPIER", copier)] {
        r.sim.infantry.insert(n.into(), u);
        reg(&mut r, "infantry", n);
    }
    r.sim.weapons.insert(
        "Flash".into(),
        WeaponType {
            damage: Some(20),
            rof: Some(30),
            range: Some(d("5")),
            speed: Some(100),
            projectile: Some("Bullet".into()),
            warhead: Some("SA".into()),
            disguise_fake_blink_time: Some(BLINK),
            ..Default::default()
        },
    );
    r.sim.weapons.insert(
        "Mimic".into(),
        WeaponType {
            damage: Some(1),
            rof: Some(30),
            range: Some(d("3")),
            speed: Some(100),
            projectile: Some("Bullet".into()),
            warhead: Some("Copy".into()),
            ..Default::default()
        },
    );
    r.sim.warheads.insert(
        "Copy".into(),
        WarheadType {
            makes_disguise: Some(true),
            verses: Some(vec![Decimal("1".into()); 11]),
            ..Default::default()
        },
    );
    let building = |f: [i32; 2]| BuildingType {
        strength: Some(500),
        foundation: Some(f),
        ..Default::default()
    };
    let mut genny = building([1, 1]);
    genny.cloak_generator = Some(true);
    genny.cloak_radius_in_cells = Some(GEN_RADIUS);
    let mut array = building([1, 1]);
    array.sensor_array = Some(true);
    array.combat.sensors_sight = Some(ARRAY_SIGHT);
    let mut detector = building([1, 1]);
    detector.combat.detect_disguise = Some(true);
    detector.detect_disguise_range = Some(DETECT_RANGE);
    let mut veil = building([2, 2]);
    veil.combat.cloakable = Some(true);
    for (n, b) in [
        ("GENNY", genny),
        ("ARRAY", array),
        ("DETECTOR", detector),
        ("VEIL", veil),
    ] {
        r.sim.buildings.insert(n.into(), b);
        reg(&mut r, "buildings", n);
    }
    let mut warf = building([1, 1]);
    warf.weapons_factory = Some(true);
    r.sim.buildings.insert("WARF".into(), warf);
    reg(&mut r, "buildings", "WARF");
    let mut wraith = UnitType {
        strength: Some(200),
        speed: Some(30),
        rot: Some(12),
        locomotor: Some(Locomotor::Fly),
        ..Default::default()
    };
    wraith.econ.landable = Some(true);
    wraith.combat.cloakable = Some(true);
    wraith.combat.cloaking_speed = Some(SPEED);
    r.sim.aircraft.insert("WRAITH".into(), wraith);
    reg(&mut r, "aircraft", "WRAITH");
    for n in ["SHRUB", "BUSH"] {
        r.sim
            .terrain_objects
            .insert(n.into(), TerrainObjectType::default());
        reg(&mut r, "terrain_objects", n);
    }
    r
}

/// A new game under `r`.
/// Orders are open from tick 4 (S-0213 claim 1); the game is run to it
/// before anything is placed.
pub fn game(r: &Ruleset) -> World {
    let mut w = testkit::game_with(r, SIZE, 1);
    while glasnost_sim::orders::blocked(&w) {
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

/// The object's cloak state.
pub fn state(w: &World, id: ObjectId) -> CloakState {
    glasnost_sim::cloak::state(w, id)
}

/// The object's stage.
pub fn stage(w: &World, id: ObjectId) -> i32 {
    w.obj(id).unwrap().cloak.stage
}

/// Runs ticks until `f` holds (at most `max`); returns the ticks run.
pub fn until(w: &mut World, max: i32, mut f: impl FnMut(&World) -> bool) -> i32 {
    for n in 0..max {
        if f(w) {
            return n;
        }
        glasnost_sim::tick::step(w);
    }
    assert!(f(w), "condition not reached in {max} ticks");
    max
}

/// Runs `n` ticks.
pub fn run(w: &mut World, n: i32) {
    testkit::run(w, n);
}

/// Makes `a` ally `b` (one way: `a`'s view).
pub fn ally(w: &mut World, a: HouseId, b: HouseId) {
    let h = &mut w.houses[usize::from(a.0)];
    if !h.allies.contains(&b) {
        h.allies.push(b);
        h.allies.sort();
    }
}

/// Forces an object fully cloaked (its stage 0).
pub fn force_cloaked(w: &mut World, id: ObjectId) {
    let o = w.obj_mut(id).unwrap();
    o.cloak.state = CloakState::Cloaked;
    o.cloak.stage = 0;
}

/// Spawns an object and forces it fully cloaked: placement within the
/// usable area starts every object uncloaked (S-0516 claim 16), so tests
/// that need a hidden unit at once set it.
pub fn spawn_cloaked(w: &mut World, name: &str, h: HouseId, dx: i32, dy: i32) -> ObjectId {
    let id = spawn(w, name, h, dx, dy);
    force_cloaked(w, id);
    id
}

/// Forces an object uncloaked with no delay running.
pub fn force_uncloaked(w: &mut World, id: ObjectId) {
    let o = w.obj_mut(id).unwrap();
    o.cloak.state = CloakState::Uncloaked;
    o.cloak.stage = 0;
    o.cloak.delay_until = 0;
}

/// The health of an object.
pub fn set_health(w: &mut World, id: ObjectId, h: i32) {
    w.obj_mut(id).unwrap().health = h;
}

/// The cloak sounds pushed since the effects were last cleared.
pub fn cloak_sounds(w: &World) -> usize {
    w.effects
        .iter()
        .filter(|e| {
            matches!(
                e,
                glasnost_sim::combat::effects::Effect::Sound {
                    sound: glasnost_sim::combat::effects::Sound::Cloak,
                    ..
                }
            )
        })
        .count()
}

/// Orders `actor` to move to `c` for its owner; panics if refused.
pub fn move_to(w: &mut World, actor: ObjectId, c: CellXY) {
    use glasnost_sim::orders::{self, Order};
    let owner = w.obj(actor).and_then(|o| o.owner).unwrap_or(ALPHA);
    let ok = orders::issue(
        w,
        owner,
        Order::Behaviour {
            actor,
            behaviour: glasnost_sim::rules::Behaviour::Move,
            attack_move: false,
            target: None,
            dest: Some(glasnost_sim::world::Target::Cell(c)),
        },
    );
    assert!(ok, "move order refused");
}

/// Orders `actor` to attack `t` for its owner; returns whether accepted.
pub fn attack(w: &mut World, actor: ObjectId, t: ObjectId) -> bool {
    use glasnost_sim::orders::{self, Order};
    let owner = w.obj(actor).and_then(|o| o.owner).unwrap_or(ALPHA);
    orders::issue(
        w,
        owner,
        Order::Behaviour {
            actor,
            behaviour: glasnost_sim::rules::Behaviour::Attack,
            attack_move: false,
            target: Some(glasnost_sim::world::Target::Object(t)),
            dest: None,
        },
    )
}

/// Switches a building off (out of operation).
pub fn switch_off(w: &mut World, id: ObjectId) {
    w.obj_mut(id).unwrap().econ.building.switched_off = true;
}

/// A plain hit of `amount` (negative heals) with no warhead or attacker.
pub fn hit(w: &mut World, id: ObjectId, amount: i32) -> glasnost_sim::combat::damage::HitResult {
    glasnost_sim::combat::damage::apply(
        w,
        id,
        glasnost_sim::combat::damage::Hit {
            amount,
            warhead: None,
            distance: 0,
            attacker: None,
            house: None,
            ignore_defences: false,
            no_escape: false,
        },
    )
}

/// A killing hit.
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

/// Runs up to `max` ticks and returns each state change as (offset, new
/// state): the offset of the tick in which it happened from the tick
/// number current at the call. An event between ticks (a test's hit, say)
/// happens at that tick number, before its turn pass.
pub fn transitions(w: &mut World, id: ObjectId, max: i32) -> Vec<(i32, CloakState)> {
    let mut out = Vec::new();
    let mut last = state(w, id);
    let t0 = w.tick;
    for _ in 0..max {
        let t = w.tick;
        glasnost_sim::tick::step(w);
        let s = state(w, id);
        if s != last {
            out.push((t - t0, s));
            last = s;
        }
    }
    out
}

/// Gives `id` a new owner and runs the owner-change hooks.
pub fn change_owner(w: &mut World, id: ObjectId, to: HouseId) {
    let old = w.obj(id).unwrap().owner;
    w.obj_mut(id).unwrap().owner = Some(to);
    glasnost_sim::econ::events::owner_changed(w, id, old);
}

/// Whether house `h` senses cell `at(dx, dy)`.
pub fn sensed_at(w: &World, h: HouseId, dx: i32, dy: i32) -> bool {
    glasnost_sim::cloak::sensed(w, Some(h), at(dx, dy))
}

/// House `h`'s sensor count at `at(dx, dy)`.
pub fn count_at(w: &World, h: HouseId, dx: i32, dy: i32) -> i32 {
    w.cloak
        .sensors
        .get(&(h.0, at(dx, dy)))
        .copied()
        .unwrap_or(0)
}

/// Whether `at(dx, dy)` is in house `h`'s cloak field.
pub fn field_at(w: &World, h: HouseId, dx: i32, dy: i32) -> bool {
    glasnost_sim::cloak::in_field(w, Some(h), at(dx, dy))
}
