//! S-0614 Parasites on invented data: a drone that latches onto vehicles
//! and infantry, a grappling parasite on water, release, exit and host
//! loss. Every type name and number here is made up, except the grab
//! animation's fixed type name, which S-0614's Import mapping gives.

use glasnost_map::CellXY;
use glasnost_map::terrain::land;
use glasnost_num::{F32, F64, Rounding};
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{
    Decimal, LandRow, LandType, LocomotionClass, Locomotor, MovementZone, ProjectileType, UnitType,
    WarheadType, WeaponType,
};
use glasnost_sim::combat::check::{self, Answer};
use glasnost_sim::combat::damage::{self, Hit};
use glasnost_sim::combat::effects::Effect;
use glasnost_sim::combat::fire;
use glasnost_sim::orders;
use glasnost_sim::parasite::{self, Leave, ParasiteState};
use glasnost_sim::rules::Kind;
use glasnost_sim::targeting::{self, ClickOn, Modifiers};
use glasnost_sim::testkit::{self, mid, spawn};
use glasnost_sim::world::{HouseId, Layer, ObjectId, Target, World};
use glasnost_sim::{naval, save};

const SIZE: i32 = 32;
const ALPHA: HouseId = HouseId(0);
/// Rate of fire of the drone's weapon.
const R: i32 = 15;
/// Paralysis length of the drone's warhead.
const P: i32 = 20;
/// The drone weapon's damage.
const DMG: i32 = 10;
/// First tile index of the invented water set (after the testkit's 5 clear tiles).
const WATER_FIRST: i32 = 5;
/// The terrain code of a water cell (S-0010).
const WATER_CODE: u8 = 9;

fn d(s: &str) -> Decimal {
    Decimal(s.into())
}

fn ones() -> Option<Vec<Decimal>> {
    Some((0..11).map(|_| d("1")).collect())
}

fn reg(r: &mut Ruleset, kind: &str, name: &str) {
    r.registries
        .get_mut(kind)
        .expect("registry")
        .push(name.into());
}

/// The combat ruleset with a drone (DRONE), a grappling parasite (SQUID),
/// a vessel (BOAT), a sonic weapon and a healing weapon.
fn ruleset() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    for n in ["DRONE", "SQUID", "BOAT", "HOVER"] {
        reg(&mut r, "vehicles", n);
    }
    let s = &mut r.sim;
    s.general.condition_red = Some(d("0.25"));
    s.general.condition_yellow = Some(d("0.5"));
    s.general.veteran_ratio = Some(d("1"));
    s.general.veteran_cap = Some(d("2"));
    s.general.splash_list = Some(vec!["SPLASHA".into(), "SPLASHB".into()]);
    s.general.wake = Some("FOAM".into());
    let mut water = LandRow {
        foot: Some(d("0")),
        track: Some(d("0")),
        wheel: Some(d("0")),
        hover: Some(d("1")),
        float: Some(d("1")),
        amphibious: Some(d("1")),
        float_beach: Some(d("1")),
        buildable: None,
    };
    water.buildable = Some(false);
    s.land_types.insert(LandType::Water, water);
    s.land_types.insert(
        LandType::Clear,
        LandRow {
            float: Some(d("0")),
            ..LandRow::default()
        },
    );
    s.projectiles
        .insert("Hop".into(), ProjectileType::default());
    s.weapons.insert(
        "Latch".into(),
        WeaponType {
            damage: Some(DMG),
            rof: Some(R),
            range: Some(d("6")),
            speed: Some(60),
            projectile: Some("Hop".into()),
            warhead: Some("Grip".into()),
            limbo_launch: Some(true),
            anim: Some((0..8).map(|k| format!("SPARK{k}")).collect()),
            ..Default::default()
        },
    );
    s.warheads.insert(
        "Grip".into(),
        WarheadType {
            verses: ones(),
            parasite: Some(true),
            paralyzes: Some(P),
            ..Default::default()
        },
    );
    s.weapons.insert(
        "Grab".into(),
        WeaponType {
            damage: Some(25),
            rof: Some(7),
            range: Some(d("6")),
            speed: Some(60),
            projectile: Some("Hop".into()),
            warhead: Some("Squeeze".into()),
            limbo_launch: Some(true),
            ..Default::default()
        },
    );
    s.warheads.insert(
        "Squeeze".into(),
        WarheadType {
            verses: ones(),
            parasite: Some(true),
            paralyzes: Some(5),
            culling: Some(true),
            ..Default::default()
        },
    );
    s.weapons.insert(
        "Sonar".into(),
        WeaponType {
            damage: Some(5),
            rof: Some(10),
            range: Some(d("5")),
            projectile: Some("Bullet".into()),
            warhead: Some("Sonic".into()),
            ..Default::default()
        },
    );
    s.warheads.insert(
        "Sonic".into(),
        WarheadType {
            verses: ones(),
            sonic: Some(true),
            ..Default::default()
        },
    );
    let mut drone = UnitType {
        strength: Some(100),
        speed: Some(9),
        rot: Some(8),
        cost: Some(200),
        locomotion_class: Some(LocomotionClass::Wheel),
        locomotor: Some(Locomotor::Drive),
        primary: Some("Latch".into()),
        ..Default::default()
    };
    drone.combat.armor = Some(glasnost_rules::sim::ArmourClass::Light);
    drone.suppression_threshold = Some(30);
    s.vehicles.insert("DRONE".into(), drone);
    let mut squid = UnitType {
        strength: Some(200),
        speed: Some(5),
        rot: Some(8),
        cost: Some(800),
        locomotion_class: Some(LocomotionClass::Float),
        movement_zone: Some(MovementZone::Water),
        locomotor: Some(Locomotor::Ship),
        primary: Some("Grab".into()),
        ..Default::default()
    };
    squid.combat.naval = Some(true);
    squid.combat.organic = Some(true);
    squid.combat.trainable = Some(true);
    squid.suppression_threshold = Some(1000);
    s.vehicles.insert("SQUID".into(), squid);
    let mut boat = UnitType {
        strength: Some(400),
        speed: Some(5),
        rot: Some(8),
        cost: Some(400),
        locomotion_class: Some(LocomotionClass::Float),
        movement_zone: Some(MovementZone::Water),
        locomotor: Some(Locomotor::Ship),
        ..Default::default()
    };
    boat.combat.naval = Some(true);
    s.vehicles.insert("BOAT".into(), boat);
    s.vehicles.insert(
        "HOVER".into(),
        UnitType {
            strength: Some(300),
            speed: Some(6),
            rot: Some(8),
            locomotion_class: Some(LocomotionClass::Amphibious),
            movement_zone: Some(MovementZone::Amphibious),
            locomotor: Some(Locomotor::Drive),
            ..Default::default()
        },
    );
    // The buggy is a soft target with a cost, and never fires.
    let b = s.vehicles.get_mut("BUGGY").unwrap();
    b.cost = Some(300);
    r
}

fn game_of(r: &Ruleset) -> World {
    let mut w = testkit::game_with(r, SIZE, 11);
    let mut cat = (*testkit::catalogue()).clone();
    let mut water = cat.tiles[0].clone();
    if let Some(Some(c)) = water.cells.first_mut() {
        c.terrain_code = WATER_CODE;
    }
    for _ in 0..14 {
        cat.tiles.push(water.clone());
        cat.allow_tiberium.push(false);
    }
    cat.roles.water = WATER_FIRST;
    w.catalogue = Some(std::sync::Arc::new(cat));
    w
}

fn game() -> World {
    game_of(&ruleset())
}

fn cell(dx: i32, dy: i32) -> CellXY {
    mid(SIZE, dx, dy)
}

/// Makes a cell a water cell (land type and a tile of the water set).
fn wet(w: &mut World, c: CellXY) {
    w.cells.grid.get_mut(c).unwrap().tile = WATER_FIRST;
    glasnost_sim::load::recalc_cells(w, &[c]);
    assert_eq!(w.cells.cell(c).unwrap().land, land::WATER);
    glasnost_sim::regions::cell_changed(w, c);
}

/// A lake: every cell within `r` of `centre`.
fn lake(w: &mut World, centre: CellXY, r: i32) {
    for dy in -r..=r {
        for dx in -r..=r {
            wet(w, CellXY::new(centre.x + dx, centre.y + dy));
        }
    }
}

fn weapon(w: &World, name: &str) -> glasnost_sim::combat::data::WeaponId {
    w.rules().combat.weapon_index[name]
}

fn warhead(w: &World, weapon_name: &str) -> Option<u32> {
    w.rules().combat.weapon(weapon(w, weapon_name)).warhead
}

fn health(w: &World, id: ObjectId) -> i32 {
    w.obj(id).map_or(-1, |o| o.health)
}

fn ps(w: &World, id: ObjectId) -> ParasiteState {
    w.obj(id)
        .and_then(|o| o.parasite.clone())
        .unwrap_or_default()
}

/// Fires the parasite's latching weapon at `target` and runs until the
/// projectile has ended. Returns the tick in which it ended.
fn shoot(w: &mut World, p: ObjectId, target: Target) -> i32 {
    assert!(fire::shot(w, p, 0, target), "the latching shot was made");
    for _ in 0..200 {
        let carrier = ps(w, p).carrier;
        if carrier.is_none() {
            return w.tick - 1;
        }
        testkit::run(w, 1);
    }
    panic!("the projectile never ended");
}

/// A drone of house 0 two cells west of a buggy of house 1, latched on.
fn latched() -> (World, ObjectId, ObjectId, i32) {
    let mut w = game();
    let host = spawn(&mut w, Kind::Vehicle, "BUGGY", 1, cell(2, 0), 0);
    let p = spawn(&mut w, Kind::Vehicle, "DRONE", 0, cell(0, 0), 64);
    let t = shoot(&mut w, p, Target::Object(host));
    assert_eq!(parasite::host_of(&w, p), Some(host));
    (w, p, host, t)
}

/// Runs until the host's health changes; returns that tick.
fn next_hit(w: &mut World, host: ObjectId, limit: i32) -> Option<i32> {
    let h0 = health(w, host);
    for _ in 0..limit {
        let t = w.tick;
        testkit::run(w, 1);
        if health(w, host) != h0 {
            return Some(t);
        }
    }
    None
}

/// Runs until the tick is `t`.
fn run_to(w: &mut World, t: i32) {
    while w.tick < t {
        testkit::run(w, 1);
    }
}

fn hit(amount: i32, warhead: Option<u32>, attacker: Option<ObjectId>) -> Hit {
    Hit {
        amount,
        warhead,
        distance: 0,
        attacker,
        house: None,
        ignore_defences: false,
        no_escape: false,
    }
}

fn set_hold(w: &mut World, id: ObjectId, until: i32) {
    w.obj_mut(id)
        .unwrap()
        .parasite
        .get_or_insert_with(Default::default)
        .hold_until = Some(until);
}

fn quirks(f: impl FnOnce(&mut glasnost_rules::quirks::Quirks)) -> Ruleset {
    let mut r = ruleset();
    f(&mut r.globals.quirks);
    r
}

// ---------------------------------------------------------------------------
// Latching on (claims 1 to 5)
// ---------------------------------------------------------------------------

// spec: S-0614 claim 1 (the firer leaves the map at once and travels inside the projectile; the mobile target's latch block is the current tick plus 20)
#[test]
fn claim_1_firing_leaves_the_map_and_sets_the_latch_block() {
    let mut w = game();
    let host = spawn(&mut w, Kind::Vehicle, "BUGGY", 1, cell(3, 0), 0);
    let p = spawn(&mut w, Kind::Vehicle, "DRONE", 0, cell(0, 0), 64);
    let t = w.tick;
    assert!(fire::shot(&mut w, p, 0, Target::Object(host)));
    let o = w.obj(p).unwrap();
    assert!(!o.on_map);
    assert!(!w.cells.list(cell(0, 0), o.layer).contains(&p));
    let carrier = ps(&w, p).carrier.expect("a carrier projectile");
    assert!(w.projectiles.contains_key(&carrier));
    assert_eq!(ps(&w, host).latch_block_until, Some(t + 20));
}

// spec: S-0614 claim 1, S-0303 claim 2 and S-0306 claim 21 (the detonation delivers no area damage, whether or not the parasite latches)
#[test]
fn claim_1_detonation_makes_no_area_hit() {
    let mut w = game();
    let host = spawn(&mut w, Kind::Vehicle, "BUGGY", 1, cell(2, 0), 0);
    let p = spawn(&mut w, Kind::Vehicle, "DRONE", 0, cell(0, 0), 64);
    let h0 = health(&w, host);
    // A cell shot next to the buggy: a failed latch, no damage to the buggy.
    shoot(&mut w, p, Target::Cell(cell(2, 0)));
    assert_eq!(health(&w, host), h0);
    assert!(w.obj(p).unwrap().on_map);
}

// spec: S-0614 claim 1, S-0303 claim 2 and S-0306 claim 21 (a parasite-flag detonation makes no area hit but keeps its explosion animation and debris, for a latch and for a failed latch)
#[test]
fn claim_1_detonation_keeps_anim_and_debris() {
    let mut r = ruleset();
    let g = r.sim.warheads.get_mut("Grip").unwrap();
    g.anim_list = Some(vec!["BOOM0".into(), "BOOM1".into()]);
    g.min_debris = Some(2);
    g.max_debris = Some(3);
    r.sim.general.metallic_debris = Some(vec!["DEB".into()]);
    for on_host in [true, false] {
        let mut w = game_of(&r);
        let host = spawn(&mut w, Kind::Vehicle, "BUGGY", 1, cell(2, 0), 0);
        let p = spawn(&mut w, Kind::Vehicle, "DRONE", 0, cell(0, 0), 64);
        w.effects.clear();
        let target = if on_host {
            Target::Object(host)
        } else {
            Target::Cell(cell(2, 2))
        };
        shoot(&mut w, p, target);
        assert_eq!(parasite::host_of(&w, p).is_some(), on_host);
        let count = |pre: &str| {
            w.effects
                .iter()
                .filter(|e| matches!(e, Effect::Anim { name, .. } if name.starts_with(pre)))
                .count()
        };
        assert_eq!(count("BOOM"), 1, "explosion animation, on host {on_host}");
        // N = R(2, 3 − 1) = 2 metallic debris pieces.
        assert_eq!(count("DEB"), 2, "debris, on host {on_host}");
    }
}

// spec: S-0614 claim 2 (latchable: a mobile unit, on the map, health not 0, no parasite attached, the can-carry-a-parasite flag; a naval parasite needs a water tile)
#[test]
fn claim_2_latchable() {
    let mut r = ruleset();
    r.sim.vehicles.get_mut("CART").unwrap().parasiteable = Some(false);
    let mut w = game_of(&r);
    let p = spawn(&mut w, Kind::Vehicle, "DRONE", 0, cell(0, 0), 0);
    let buggy = spawn(&mut w, Kind::Vehicle, "BUGGY", 1, cell(2, 0), 0);
    let walker = spawn(&mut w, Kind::Infantry, "WALKER", 1, cell(2, 2), 0);
    let cart = spawn(&mut w, Kind::Vehicle, "CART", 1, cell(-2, 0), 0);
    let tower = spawn(&mut w, Kind::Building, "TOWER", 1, cell(0, 4), 0);
    assert!(parasite::latchable(&w, p, buggy));
    // spec: S-0614 Edge cases (the can-carry-a-parasite default for infantry types is yes)
    assert!(parasite::latchable(&w, p, walker));
    assert!(!parasite::latchable(&w, p, cart), "flag off");
    assert!(!parasite::latchable(&w, p, tower), "not mobile");
    w.obj_mut(walker).unwrap().health = 0;
    assert!(!parasite::latchable(&w, p, walker), "health 0");
    // A naval parasite: the target's cell must be a water tile.
    let squid = spawn(&mut w, Kind::Vehicle, "SQUID", 0, cell(-4, -4), 0);
    assert!(!parasite::latchable(&w, squid, buggy));
    wet(&mut w, cell(2, 0));
    assert!(parasite::latchable(&w, squid, buggy));
}

// spec: S-0614 claim 2 (no parasite attached: a host carries at most one parasite)
#[test]
fn claim_2_a_host_carries_at_most_one() {
    let (mut w, _p, host, _) = latched();
    let other = spawn(&mut w, Kind::Vehicle, "DRONE", 0, cell(0, 3), 0);
    assert!(!parasite::latchable(&w, other, host));
}

// spec: S-0614 claim 3 (attached: off the map, travels with the host, keeps no target; the first delivery at the host's next turn)
#[test]
fn claim_3_attaches_and_delivers_at_the_hosts_next_turn() {
    let (mut w, p, host, t) = latched();
    let o = w.obj(p).unwrap();
    assert!(!o.on_map);
    assert_eq!(o.target, None);
    assert_eq!(parasite::parasite_of(&w, host), Some(p));
    // The host comes before the projectile in the update order: the next tick.
    let first = next_hit(&mut w, host, 5).expect("a delivery");
    assert_eq!(first, t + 1);
    assert_eq!(w.obj(p).unwrap().pos, w.obj(host).unwrap().pos);
}

// spec: S-0614 claim 4 (a cell target: placed at the centre of the cell it last stood in, facing 0, target cleared, not held)
#[test]
fn claim_4_failed_latch_on_a_cell() {
    let mut w = game();
    let p = spawn(&mut w, Kind::Vehicle, "DRONE", 0, cell(0, 0), 64);
    shoot(&mut w, p, Target::Cell(cell(3, 0)));
    let o = w.obj(p).unwrap();
    assert!(o.on_map);
    assert_eq!(o.pos.cell(), cell(0, 0));
    assert_eq!((o.pos.x & 255, o.pos.y & 255), (128, 128));
    assert_eq!(o.body.value(w.tick), 0);
    assert_eq!(o.target, None);
    assert!(!parasite::held(&w, p));
}

// spec: S-0614 claim 4 (a mobile target that died before the detonation: a failed latch)
#[test]
fn claim_4_failed_latch_on_a_dead_target() {
    let mut w = game();
    let host = spawn(&mut w, Kind::Vehicle, "BUGGY", 1, cell(4, 0), 0);
    let p = spawn(&mut w, Kind::Vehicle, "DRONE", 0, cell(0, 0), 64);
    assert!(fire::shot(&mut w, p, 0, Target::Object(host)));
    glasnost_sim::objects::remove(&mut w, host);
    for _ in 0..100 {
        if ps(&w, p).carrier.is_none() {
            break;
        }
        testkit::run(&mut w, 1);
    }
    let o = w.obj(p).unwrap();
    assert!(o.on_map);
    assert_eq!(o.pos.cell(), cell(0, 0));
}

// spec: S-0614 claims 4 and 20 (an allied stationary vehicle in the cell: the entry outcome is not clear, removed silently)
#[test]
fn claim_4_blocked_cell_removes_silently() {
    let mut w = game();
    let p = spawn(&mut w, Kind::Vehicle, "DRONE", 0, cell(0, 0), 64);
    assert!(fire::shot(&mut w, p, 0, Target::Cell(cell(4, 0))));
    let blocker = spawn(&mut w, Kind::Vehicle, "BUGGY", 0, cell(0, 0), 0);
    let kills = w.houses[1].combat.kills;
    for _ in 0..100 {
        if w.obj(p).is_none() {
            break;
        }
        testkit::run(&mut w, 1);
    }
    assert!(w.obj(p).is_none(), "removed silently");
    assert!(w.obj(blocker).is_some());
    assert_eq!(w.houses[1].combat.kills, kills);
}

// spec: S-0614 claim 4 (target and destination kept when it has attack-move memory)
#[test]
fn claim_4_attack_move_memory_keeps_the_target() {
    let mut w = game();
    let p = spawn(&mut w, Kind::Vehicle, "DRONE", 0, cell(0, 0), 64);
    let goal = Target::Cell(cell(8, 8));
    {
        let o = w.obj_mut(p).unwrap();
        o.attack_move = Some(glasnost_sim::world::AttackMoveMemory::Dest(goal));
        o.dest = Some(goal);
    }
    shoot(&mut w, p, Target::Cell(cell(3, 0)));
    assert_eq!(w.obj(p).unwrap().dest, Some(goal));
}

// spec: S-0614 claim 5 (illegal: not latchable, before the latch block, a height difference above 208 either way; buildings and cells are not tested)
#[test]
fn claim_5_firing_tests() {
    let mut w = game();
    let host = spawn(&mut w, Kind::Vehicle, "BUGGY", 1, cell(3, 0), 0);
    let p = spawn(&mut w, Kind::Vehicle, "DRONE", 0, cell(0, 0), 64);
    let q = spawn(&mut w, Kind::Vehicle, "DRONE", 0, cell(0, 2), 64);
    let tower = spawn(&mut w, Kind::Building, "TOWER", 1, cell(0, -4), 0);
    let ans = |w: &World, f, t| check::check(w, f, Some(Target::Object(t)), 0, false);
    assert_ne!(ans(&w, q, host), Answer::Illegal);
    assert_ne!(ans(&w, p, tower), Answer::Illegal);
    assert!(fire::shot(&mut w, p, 0, Target::Object(host)));
    // Another firer in the same tick, after the shot: the latch block.
    assert_eq!(ans(&w, q, host), Answer::Illegal);
    let t0 = w.tick;
    w.tick = t0 + 19;
    assert_eq!(ans(&w, q, host), Answer::Illegal);
    w.tick = t0 + 20;
    assert_ne!(ans(&w, q, host), Answer::Illegal);
    // Height: 208 is allowed, 209 is not, in either direction.
    let z = w.obj(host).unwrap().pos.z;
    w.obj_mut(q).unwrap().pos.z = z + 208;
    assert_ne!(ans(&w, q, host), Answer::Illegal);
    w.obj_mut(q).unwrap().pos.z = z + 209;
    assert_eq!(ans(&w, q, host), Answer::Illegal);
    w.obj_mut(q).unwrap().pos.z = z - 209;
    assert_eq!(ans(&w, q, host), Answer::Illegal);
}

// spec: S-0614 claim 5 (a target that is not latchable makes the shot illegal)
#[test]
fn claim_5_not_latchable_is_illegal() {
    let (mut w, _p, host, _) = latched();
    let q = spawn(&mut w, Kind::Vehicle, "DRONE", 0, cell(0, 3), 64);
    w.tick += 30;
    assert_eq!(
        check::check(&w, q, Some(Target::Object(host)), 0, false),
        Answer::Illegal
    );
}

// ---------------------------------------------------------------------------
// While attached (claims 6 to 11)
// ---------------------------------------------------------------------------

// spec: S-0614 claim 6 and F1 (deliveries at t0 + k × max(1, R): the hit of the weapon's damage, the parasite as attacker)
#[test]
fn claim_6_delivery_cadence() {
    let (mut w, _p, host, _) = latched();
    let h0 = health(&w, host);
    let a = next_hit(&mut w, host, 5).unwrap();
    assert_eq!(health(&w, host), h0 - DMG);
    let b = next_hit(&mut w, host, 40).unwrap();
    let c = next_hit(&mut w, host, 40).unwrap();
    assert_eq!((b - a, c - b), (R, R));
}

// spec: S-0614 F1 (with R = 0 a delivery happens in every host turn)
#[test]
fn f1_rate_zero_delivers_every_turn() {
    let mut r = ruleset();
    r.sim.weapons.get_mut("Latch").unwrap().rof = Some(0);
    let mut w = game_of(&r);
    let host = spawn(&mut w, Kind::Vehicle, "BUGGY", 1, cell(2, 0), 0);
    let p = spawn(&mut w, Kind::Vehicle, "DRONE", 0, cell(0, 0), 64);
    shoot(&mut w, p, Target::Object(host));
    let a = next_hit(&mut w, host, 5).unwrap();
    let b = next_hit(&mut w, host, 5).unwrap();
    assert_eq!(b - a, 1);
}

// spec: S-0614 claim 6 and Edge cases (an infantry host is killed at the first delivery whatever its health, ignoring defences)
#[test]
fn claim_6_infantry_host_dies_at_the_first_delivery() {
    let mut w = game();
    let man = spawn(&mut w, Kind::Infantry, "WALKER", 1, cell(2, 0), 0);
    let p = spawn(&mut w, Kind::Vehicle, "DRONE", 0, cell(0, 0), 64);
    shoot(&mut w, p, Target::Object(man));
    assert_eq!(parasite::host_of(&w, p), Some(man));
    testkit::run(&mut w, 2);
    assert!(w.obj(man).is_none_or(|o| o.health <= 0));
    // spec: S-0614 claim 16 (the destruction is credited to the parasite's house)
    assert!(w.houses[0].combat.kills >= 1);
}

// spec: S-0614 claim 7 and F1 (each delivery restarts the hold at P; with P > R the host is held throughout)
#[test]
fn claim_7_hold_renewed_at_each_delivery() {
    let (mut w, p, host, _) = latched();
    let a = next_hit(&mut w, host, 5).unwrap();
    assert_eq!(ps(&w, host).hold_until, Some(a + P));
    for _ in 0..(3 * R) {
        assert!(parasite::held(&w, host));
        testkit::run(&mut w, 1);
    }
    // When the parasite leaves, the host's hold ends at once.
    parasite::leave_living(&mut w, p, Leave::Exit);
    assert!(!parasite::held(&w, host));
}

// spec: S-0614 claim 7 (the hold after a renewal lasts min(P, max(1, R)) ticks of the interval)
#[test]
fn claim_7_short_paralysis_leaves_gaps() {
    let mut r = ruleset();
    r.sim.warheads.get_mut("Grip").unwrap().paralyzes = Some(4);
    let mut w = game_of(&r);
    let host = spawn(&mut w, Kind::Vehicle, "BUGGY", 1, cell(2, 0), 0);
    let p = spawn(&mut w, Kind::Vehicle, "DRONE", 0, cell(0, 0), 64);
    shoot(&mut w, p, Target::Object(host));
    let a = next_hit(&mut w, host, 5).unwrap();
    let mut held = 0;
    for _ in 0..R {
        if parasite::held(&w, host) {
            held += 1;
        }
        testkit::run(&mut w, 1);
    }
    let _ = a;
    // The tick after the delivery tick is the first one checked: 3 of the 4 held ticks remain.
    assert_eq!(held, 3);
}

// spec: S-0614 claim 8 (a held unit does not move under the driving locomotor and is not movable; the walking locomotor ignores the hold)
#[test]
fn claim_8_held_does_not_drive_and_is_not_movable() {
    let mut w = game();
    let car = spawn(&mut w, Kind::Vehicle, "BUGGY", 0, cell(0, 0), 64);
    let man = spawn(&mut w, Kind::Infantry, "WALKER", 0, cell(0, 4), 64);
    let t = w.tick;
    set_hold(&mut w, car, t + 100);
    set_hold(&mut w, man, t + 100);
    assert!(!targeting::movable(&w, ALPHA, car));
    assert!(!targeting::movable(&w, ALPHA, man));
    let p0 = w.obj(car).unwrap().pos;
    let m0 = w.obj(man).unwrap().pos;
    for (id, to) in [(car, cell(6, 0)), (man, cell(6, 4))] {
        w.tick = w.tick.max(w.orders_blocked_through + 1);
        let o = w.obj_mut(id).unwrap();
        o.dest = Some(Target::Cell(to));
    }
    testkit::run(&mut w, 30);
    assert_eq!(w.obj(car).unwrap().pos, p0, "held: no driving");
    assert_ne!(w.obj(man).unwrap().pos, m0, "walking ignores the hold");
}

// spec: S-0614 claim 8 and S-0301 claim 9 rank 9 (a held organic firer: illegal; not organic: not refused)
#[test]
fn claim_8_held_organic_firer_is_illegal() {
    let mut r = ruleset();
    r.sim.vehicles.get_mut("CART").unwrap().combat.organic = Some(true);
    let mut w = game_of(&r);
    let cart = spawn(&mut w, Kind::Vehicle, "CART", 0, cell(0, 0), 64);
    let crusher = spawn(&mut w, Kind::Vehicle, "CRUSHER", 0, cell(0, 2), 64);
    let foe = spawn(&mut w, Kind::Vehicle, "BUGGY", 1, cell(3, 0), 0);
    let t = w.tick;
    set_hold(&mut w, cart, t + 100);
    set_hold(&mut w, crusher, t + 100);
    let tg = Some(Target::Object(foe));
    assert_eq!(check::check(&w, cart, tg, 0, false), Answer::Illegal);
    assert_ne!(check::check(&w, crusher, tg, 0, false), Answer::Illegal);
    w.tick = t + 100;
    assert_ne!(check::check(&w, cart, tg, 0, false), Answer::Illegal);
}

// spec: S-0614 claim 8, S-0301 claim 9 rank 4 and S-0506 claim 10 (a held mobile firer's spawner weapon: impossible, whatever its organic type)
#[test]
fn claim_8_held_spawner_is_impossible() {
    let mut r = ruleset();
    r.sim.weapons.get_mut("Gun").unwrap().spawner = Some(true);
    let mut w = game_of(&r);
    let cart = spawn(&mut w, Kind::Vehicle, "CRUSHER", 0, cell(0, 0), 64);
    let foe = spawn(&mut w, Kind::Vehicle, "BUGGY", 1, cell(3, 0), 0);
    let tg = Some(Target::Object(foe));
    assert_eq!(check::check(&w, cart, tg, 0, false), Answer::Rearming);
    let t = w.tick;
    set_hold(&mut w, cart, t + 10);
    assert_eq!(check::check(&w, cart, tg, 0, false), Answer::Impossible);
}

// spec: S-0614 claim 9 (the animation indexed directly by the host's 8-direction; then one draw R(0, 1) before the hit)
#[test]
fn claim_9_delivery_animation_and_draw() {
    let (mut w, _p, host, _) = latched();
    // Facing east: 8-direction 2.
    let now = w.tick;
    w.obj_mut(host).unwrap().body.set(16384, now, false);
    w.effects.clear();
    let mut expect = w.rng.clone();
    next_hit(&mut w, host, 5).unwrap();
    let names: Vec<String> = w
        .effects
        .iter()
        .filter_map(|e| match e {
            Effect::Anim { name, .. } if name.starts_with("SPARK") => Some(name.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(names, ["SPARK2"]);
    // The only draw of the delivery tick in this scene is the rocking side.
    let _ = expect.ranged(0, 1, w.rules().quirks.ranged_draw_half_span);
    assert_eq!(w.rng, expect);
}

// spec: S-0614 claim 9 and S-0616 claim 1 (each delivery to a non-infantry host makes one system of the game-wide spark type at the host's position, taking no draw; without the key none) and S-0616 claim 20 (FIZZ has no spark-turns field, so it is marked in its first turn with no draw and, empty, leaves play then)
#[test]
fn claim_9_delivery_spark_system() {
    let mut ids = Vec::new();
    for given in [true, false] {
        let mut r = ruleset();
        if given {
            r.registries
                .entry("particle_systems".into())
                .or_default()
                .push("FIZZ".into());
            r.sim.particle_systems.insert(
                "FIZZ".into(),
                glasnost_rules::sim::ParticleSystemType {
                    behaves_like: Some("spark".into()),
                    ..Default::default()
                },
            );
            r.sim.general.default_spark_system = Some("FIZZ".into());
        }
        let mut w = game_of(&r);
        let host = spawn(&mut w, Kind::Vehicle, "BUGGY", 1, cell(2, 0), 0);
        let p = spawn(&mut w, Kind::Vehicle, "DRONE", 0, cell(0, 0), 64);
        shoot(&mut w, p, Target::Object(host));
        let n = w.gas.systems.len();
        let id0 = w.next_id;
        let mut expect = w.rng.clone();
        next_hit(&mut w, host, 5).unwrap();
        let _ = expect.ranged(0, 1, w.rules().quirks.ranged_draw_half_span);
        assert_eq!(
            w.rng, expect,
            "the spark system takes no draw (given {given})"
        );
        ids.push(w.next_id - id0);
        assert_eq!(w.gas.systems.len(), n, "made and gone, or never made");
    }
    assert_eq!(
        ids[0],
        ids[1] + 1,
        "one system made with the key, none without"
    );
}

// spec: S-0614 claim 9 (an animation list shorter than the 8-direction makes no animation; no modulo reduction)
#[test]
fn claim_9_short_list_makes_no_animation() {
    let mut r = ruleset();
    r.sim.weapons.get_mut("Latch").unwrap().anim = Some(vec!["SPARK0".into()]);
    let mut w = game_of(&r);
    let host = spawn(&mut w, Kind::Vehicle, "BUGGY", 1, cell(2, 0), 32768);
    let p = spawn(&mut w, Kind::Vehicle, "DRONE", 0, cell(0, 0), 64);
    shoot(&mut w, p, Target::Object(host));
    w.effects.clear();
    next_hit(&mut w, host, 5).unwrap();
    assert!(
        !w.effects
            .iter()
            .any(|e| matches!(e, Effect::Anim { name, .. } if name.starts_with("SPARK")))
    );
}

/// A squid of house 0 latched onto a boat of house 1 on a lake.
fn grappled(r: &Ruleset) -> (World, ObjectId, ObjectId, i32) {
    let mut w = game_of(r);
    lake(&mut w, cell(0, 0), 5);
    let boat = spawn(&mut w, Kind::Vehicle, "BOAT", 1, cell(2, 0), 0);
    let squid = spawn(&mut w, Kind::Vehicle, "SQUID", 0, cell(-1, 0), 64);
    let t = shoot(&mut w, squid, Target::Object(boat));
    assert_eq!(parasite::host_of(&w, squid), Some(boat));
    (w, squid, boat, t)
}

// spec: S-0614 claim 10, F3 and table G (strikes at g + 69 and then every 40 ticks; the rate of fire is not used; the host held throughout)
#[test]
fn claim_10_grapple_timing() {
    let (mut w, _s, boat, t) = grappled(&ruleset());
    // The host's first turn after latching is the next tick.
    let g = t + 1;
    let a = next_hit(&mut w, boat, 200).unwrap();
    assert_eq!(a, g + 69);
    let b = next_hit(&mut w, boat, 200).unwrap();
    let c = next_hit(&mut w, boat, 200).unwrap();
    assert_eq!((b, c), (g + 109, g + 149));
    assert!(parasite::held(&w, boat));
}

// spec: S-0614 claim 10 and table G (the draws: R(0, 1) at the grab's end, then 3 × (R(−180, 180), R(−64, 64)); at a strike 3 × (R(0, 1), R(0, n − 1)), then R(0, 1) and six wake draws)
#[test]
fn table_g_draw_order() {
    let (mut w, _s, boat, t) = grappled(&ruleset());
    let g = t + 1;
    let half = w.rules().quirks.ranged_draw_half_span;
    // The parasite's own draws, in its host's turn (the host's other processing is left out).
    run_to(&mut w, g + 29);
    let mut expect = w.rng.clone();
    parasite::host_turn(&mut w, boat);
    let _ = expect.ranged(0, 1, half);
    for _ in 0..3 {
        let _ = expect.ranged(-180, 180, half);
        let _ = expect.ranged(-64, 64, half);
    }
    assert_eq!(w.rng, expect, "the grab's end");
    let wakes = w
        .effects
        .iter()
        .filter(|e| matches!(e, Effect::Anim { name, .. } if name == "FOAM"))
        .count();
    assert_eq!(wakes, 3);
    testkit::run(&mut w, 1);
    run_to(&mut w, g + 69);
    let mut expect = w.rng.clone();
    parasite::host_turn(&mut w, boat);
    for _ in 0..3 {
        let _ = expect.ranged(0, 1, half);
        let _ = expect.ranged(0, 1, half);
    }
    let _ = expect.ranged(0, 1, half);
    for _ in 0..3 {
        let _ = expect.ranged(-180, 180, half);
        let _ = expect.ranged(-64, 64, half);
    }
    assert_eq!(w.rng, expect, "the strike");
}

// spec: S-0614 claim 10 (the host's cell is not a water tile: the parasite leaves at once and is not held afterwards)
#[test]
fn claim_10_off_water_leaves_without_hold() {
    let (mut w, squid, boat, _) = grappled(&ruleset());
    testkit::run(&mut w, 3);
    // The boat's cell dries up.
    let c = w.obj(boat).unwrap().pos.cell();
    let g = w.cells.grid.get_mut(c).unwrap();
    g.tile = 0;
    testkit::run(&mut w, 1);
    assert_eq!(parasite::host_of(&w, squid), None);
    assert!(w.obj(squid).unwrap().on_map);
    assert!(!parasite::held(&w, squid));
    assert!(!parasite::held(&w, boat));
}

// spec: S-0614 claim 11, F6 and S-0312 claim 10 (red health and above 0: the cull; the host starts sinking without a hit; the parasite leaves without a hold)
#[test]
fn claim_11_cull() {
    let (mut w, squid, boat, t) = grappled(&ruleset());
    // Red: 100 of 400 is exactly the red ratio 0.25.
    w.obj_mut(boat).unwrap().health = 100;
    let kills = w.houses[0].combat.kills;
    run_to(&mut w, t + 1 + 69 + 1);
    assert_eq!(health(&w, boat), 100, "no hit");
    assert!(naval::sinking(&w, boat));
    assert_eq!(parasite::host_of(&w, squid), None);
    assert!(w.obj(squid).unwrap().on_map);
    assert!(!parasite::held(&w, squid));
    assert_eq!(w.houses[0].combat.kills, kills, "no kill credit");
    // spec: S-0614 F5 (e′ = S-0311 F3 with V = A_h = 400 and A_r = A_p = 800: 0.5)
    let e = w.obj(squid).unwrap().combat.experience.to_f64();
    assert!(
        e.eq_num(
            F64::from_i64(1, Rounding::TowardZero)
                .div(F64::from_i64(2, Rounding::TowardZero), Rounding::TowardZero)
        )
    );
}

// spec: S-0614 F6 (yellow health culls only for an elite parasite; green never)
#[test]
fn f6_yellow_needs_an_elite_parasite() {
    let r = ruleset();
    for (elite, culled) in [(false, false), (true, true)] {
        let (mut w, squid, boat, t) = grappled(&r);
        w.obj_mut(boat).unwrap().health = 150;
        if elite {
            w.obj_mut(squid).unwrap().combat.experience =
                F64::from_i64(2, Rounding::TowardZero).to_f32(Rounding::TowardZero);
        }
        run_to(&mut w, t + 1 + 69 + 1);
        assert_eq!(naval::sinking(&w, boat), culled, "elite {elite}");
    }
    let (mut w, squid, boat, t) = grappled(&r);
    w.obj_mut(squid).unwrap().combat.experience =
        F64::from_i64(2, Rounding::TowardZero).to_f32(Rounding::TowardZero);
    run_to(&mut w, t + 1 + 69 + 1);
    assert!(!naval::sinking(&w, boat), "green");
}

// spec: S-0614 claim 11 (a warhead without the cull flag never culls)
#[test]
fn claim_11_no_cull_flag() {
    let mut r = ruleset();
    r.sim.warheads.get_mut("Squeeze").unwrap().culling = None;
    let (mut w, _squid, boat, t) = grappled(&r);
    w.obj_mut(boat).unwrap().health = 60;
    run_to(&mut w, t + 1 + 69 + 1);
    assert!(!naval::sinking(&w, boat));
    assert!(health(&w, boat) < 60);
}

// ---------------------------------------------------------------------------
// Leaving (claims 12 to 15)
// ---------------------------------------------------------------------------

// spec: S-0614 claim 12 and S-0315 claim 7 (a sonic hit: the parasite exits before the hit changes health, and is held for 3 × R)
#[test]
fn claim_12_sonic_hit_exits() {
    let (mut w, p, host, _) = latched();
    let wh = warhead(&w, "Sonar");
    let t = w.tick;
    damage::apply(&mut w, host, hit(5, wh, None));
    assert_eq!(parasite::host_of(&w, p), None);
    assert!(w.obj(p).unwrap().on_map);
    assert_eq!(ps(&w, p).hold_until, Some(t + 3 * R));
    assert!(!parasite::held(&w, host));
}

// spec: S-0614 claims 12 and 13 (a lethal sonic hit: the parasite exits before the hit changes health, so it comes out held for 3 × R as the host dies)
#[test]
fn claim_12_lethal_sonic_hit_exits() {
    let (mut w, p, host, _) = latched();
    let wh = warhead(&w, "Sonar");
    let t = w.tick;
    damage::apply(&mut w, host, hit(10_000, wh, None));
    assert!(w.obj(host).is_none_or(|o| o.health <= 0));
    assert!(w.obj(p).is_some_and(|o| o.on_map));
    assert_eq!(parasite::host_of(&w, p), None);
    assert_eq!(ps(&w, p).hold_until, Some(t + 3 * R));
}

// spec: S-0614 claim 12 and S-0315 claim 7 (a healing hit whose warhead is not sonic removes a drone parasite silently)
#[test]
fn claim_12_healing_hit_removes_a_drone() {
    let (mut w, p, host, _) = latched();
    let wh = warhead(&w, "Rifle");
    damage::apply(&mut w, host, hit(-5, wh, None));
    assert!(w.obj(p).is_none());
    assert_eq!(parasite::parasite_of(&w, host), None);
}

// spec: S-0614 claim 12 (a healing hit makes a naval parasite exit)
#[test]
fn claim_12_healing_hit_naval_exits() {
    let (mut w, squid, boat, _) = grappled(&ruleset());
    let wh = warhead(&w, "Rifle");
    damage::apply(&mut w, boat, hit(-5, wh, None));
    assert!(w.obj(squid).is_some_and(|o| o.on_map));
}

// spec: S-0614 claim 12 and S-0314 claim 3 (a repair step that goes ahead: as a healing hit)
#[test]
fn claim_12_repair_step_removes_a_drone() {
    let (mut w, p, host, _) = latched();
    w.obj_mut(host).unwrap().health -= 50;
    w.houses[1].credits = 10_000;
    let out = glasnost_sim::combat::heal::repair_step(&mut w, host);
    assert!(
        out.is_some(),
        "the buggy has a cost, so the step goes ahead"
    );
    assert!(w.obj(p).is_none());
    assert_eq!(parasite::parasite_of(&w, host), None);
}

// spec: S-0614 claim 12 (unloading and consumption: 50 ticks of suppression just before the exit, so a drone parasite is removed silently; a naval one exits)
#[test]
fn claim_12_unloading_and_consumption() {
    let (mut w, p, host, _) = latched();
    parasite::host_unloading(&mut w, host);
    assert!(w.obj(p).is_none());
    let (mut w, p, host, _) = latched();
    parasite::host_consumed(&mut w, host);
    assert!(w.obj(p).is_none());
    let (mut w, squid, boat, _) = grappled(&ruleset());
    parasite::host_unloading(&mut w, boat);
    assert!(w.obj(squid).is_some_and(|o| o.on_map));
}

// spec: S-0614 claim 12 (a teleporting superweapon at launch and a replaced locomotor: only a naval parasite exits; the teleport jump: both exit)
#[test]
fn claim_12_superweapon_locomotor_and_jump() {
    let (mut w, p, host, _) = latched();
    parasite::superweapon_teleport(&mut w, host);
    parasite::locomotor_replaced(&mut w, host);
    assert_eq!(parasite::host_of(&w, p), Some(host));
    parasite::host_jumps(&mut w, host);
    assert_eq!(parasite::host_of(&w, p), None);
    assert!(w.obj(p).unwrap().on_map);
    let (mut w, squid, boat, _) = grappled(&ruleset());
    parasite::locomotor_replaced(&mut w, boat);
    assert_eq!(parasite::host_of(&w, squid), None);
}

// spec: S-0614 claim 12, S-0315 claims 6 and 7 (a hit above T suppresses for W(2d − T); a suppressed drone is removed silently on a sonic exit; its own hits never suppress)
#[test]
fn claim_12_suppression_then_sonic() {
    let (mut w, p, host, _) = latched();
    let gun = warhead(&w, "Gun");
    // T = 30: d = 30 does not suppress, d = 40 suppresses for 50 ticks.
    damage::apply(&mut w, host, hit(30, gun, None));
    assert!(!parasite::suppressed(&w, p));
    damage::apply(&mut w, host, hit(40, gun, Some(p)));
    assert!(!parasite::suppressed(&w, p), "its own hit");
    let t = w.tick;
    damage::apply(&mut w, host, hit(40, gun, None));
    assert_eq!(ps(&w, p).suppressed_until, Some(t + 50));
    assert!(parasite::suppressed(&w, p));
    let wh = warhead(&w, "Sonar");
    damage::apply(&mut w, host, hit(1, wh, None));
    assert!(w.obj(p).is_none(), "suppressed: removed silently");
}

// spec: S-0614 claim 13 and F4 (the exit facing: a quarter turn clockwise for 8-directions 0 to 2, else counter-clockwise, in 8-bit units)
#[test]
fn f4_exit_direction() {
    assert_eq!(parasite::exit_direction(0), 16383);
    assert_eq!(parasite::exit_direction(16384), 32767);
    assert_eq!(parasite::exit_direction(32768), 16385);
    assert_eq!(parasite::exit_direction(49152), 32769);
    // Direction 1 (north-east) goes clockwise, wrapping past nothing.
    assert_eq!(parasite::exit_direction(8192), 24575);
    // Direction 7 (north-west) goes counter-clockwise.
    assert_eq!(parasite::exit_direction(57344), 40961);
}

// spec: S-0614 claim 13 (a drone parasite: the host's cell is never clear, so the search chooses a neighbour; facing the rounded exit direction; held 3 × R)
#[test]
fn claim_13_exit_placement() {
    let (mut w, p, host, _) = latched();
    let hc = w.obj(host).unwrap().pos.cell();
    let f = w.obj(host).unwrap().body.value(w.tick);
    parasite::leave_living(&mut w, p, Leave::Exit);
    let o = w.obj(p).unwrap();
    assert!(o.on_map);
    assert_ne!(o.pos.cell(), hc);
    let dx = (o.pos.cell().x - hc.x).abs();
    let dy = (o.pos.cell().y - hc.y).abs();
    assert!(dx <= 1 && dy <= 1, "next to the host");
    let ex = parasite::exit_direction(f);
    assert_eq!(
        o.body.value(w.tick),
        ((u32::from(ex) + 128) / 256 % 256 * 256) as u16
    );
}

// spec: S-0614 claim 13 (the host-cell test: a drone parasite of a host on water is removed silently)
#[test]
fn claim_13_host_on_water_removes_a_drone() {
    let mut w = game();
    lake(&mut w, cell(2, 0), 1);
    let host = spawn(&mut w, Kind::Vehicle, "BOAT", 1, cell(2, 0), 0);
    let p = spawn(&mut w, Kind::Vehicle, "DRONE", 0, cell(-2, 0), 64);
    shoot(&mut w, p, Target::Object(host));
    assert_eq!(parasite::host_of(&w, p), Some(host));
    parasite::leave_living(&mut w, p, Leave::Exit);
    assert!(w.obj(p).is_none());
}

// spec: S-0614 claim 13 (a naval parasite: the candidate is the neighbour in the exit direction's 8-direction when it is clear)
#[test]
fn claim_13_naval_candidate_beside_the_host() {
    let (mut w, squid, boat, _) = grappled(&ruleset());
    // The boat faces north: exit direction 16383, 8-direction 2 (east).
    let hc = w.obj(boat).unwrap().pos.cell();
    parasite::leave_living(&mut w, squid, Leave::Exit);
    let o = w.obj(squid).unwrap();
    assert_eq!(o.pos.cell(), CellXY::new(hc.x + 1, hc.y));
    // Organic and held: its shots are illegal until the hold runs out (claim 14).
    assert!(parasite::held(&w, squid));
}

// spec: S-0614 claim 14 (after coming out it may fire again: a drone that is not organic at once, an organic parasite once its hold has run out)
#[test]
fn claim_14_after_coming_out() {
    let (mut w, p, host, _) = latched();
    parasite::leave_living(&mut w, p, Leave::Exit);
    let other = spawn(&mut w, Kind::Vehicle, "BUGGY", 1, cell(-3, 0), 0);
    w.tick += 25;
    assert_ne!(
        check::check(&w, p, Some(Target::Object(other)), 0, false),
        Answer::Illegal
    );
    let _ = host;
    let (mut w, squid, boat, _) = grappled(&ruleset());
    parasite::leave_living(&mut w, squid, Leave::Exit);
    let until = ps(&w, squid).hold_until.unwrap();
    // Past its rearm delay, still within its hold of 3 x 7 ticks.
    w.tick += 10;
    assert_eq!(
        check::check(&w, squid, Some(Target::Object(boat)), 0, false),
        Answer::Illegal
    );
    w.tick = until;
    assert!(!parasite::held(&w, squid));
}

// spec: S-0614 claim 15 (the host is removed: the parasite is placed at the host's cell centre, facing the host's facing, not held)
#[test]
fn claim_15_host_lost_places_at_the_host() {
    let (mut w, p, host, _) = latched();
    let now = w.tick;
    w.obj_mut(host).unwrap().body.set(16384, now, false);
    let hc = w.obj(host).unwrap().pos.cell();
    glasnost_sim::objects::remove(&mut w, host);
    let o = w.obj(p).unwrap();
    assert!(o.on_map);
    assert_eq!(o.pos.cell(), hc);
    assert_eq!(o.body.value(w.tick), 16384);
    assert!(!parasite::held(&w, p));
}

// spec: S-0614 claim 15 and S-0302 (the host destroyed by deliveries: released at its removal)
#[test]
fn claim_15_host_destroyed_by_deliveries() {
    let mut r = ruleset();
    r.sim.weapons.get_mut("Latch").unwrap().damage = Some(60);
    let mut w = game_of(&r);
    let host = spawn(&mut w, Kind::Vehicle, "BUGGY", 1, cell(2, 0), 0);
    let p = spawn(&mut w, Kind::Vehicle, "DRONE", 0, cell(0, 0), 64);
    shoot(&mut w, p, Target::Object(host));
    for _ in 0..300 {
        if w.obj(host).is_none() {
            break;
        }
        testkit::run(&mut w, 1);
    }
    assert!(w.obj(host).is_none());
    assert!(w.obj(p).is_some_and(|o| o.on_map));
    assert!(w.houses[0].combat.kills >= 1);
}

// spec: S-0614 claim 15 (suppressed: removed silently)
#[test]
fn claim_15_suppressed_drone_removed() {
    let (mut w, p, host, _) = latched();
    let gun = warhead(&w, "Gun");
    damage::apply(&mut w, host, hit(40, gun, None));
    glasnost_sim::objects::remove(&mut w, host);
    assert!(w.obj(p).is_none());
}

// spec: S-0614 claim 15 and Edge cases (quirks.parasite_host_loss_suppresses_naval: a suppressed naval parasite is removed; off, it is placed)
#[test]
fn quirk_host_loss_suppresses_naval() {
    for on in [true, false] {
        let r = quirks(|q| q.parasite_host_loss_suppresses_naval = on);
        let (mut w, squid, boat, _) = grappled(&r);
        let now = w.tick;
        w.obj_mut(squid)
            .unwrap()
            .parasite
            .as_mut()
            .unwrap()
            .suppressed_until = Some(now + 100);
        glasnost_sim::objects::remove(&mut w, boat);
        assert_eq!(w.obj(squid).is_none(), on, "quirk {on}");
    }
}

// spec: S-0614 claim 15 and Edge cases (quirks.parasite_host_loss_bridge_test: the host's cell fails the test, the neighbour in front is water; on: the host's bridge decides; off: the neighbour's)
#[test]
fn quirk_host_loss_bridge_test() {
    for on in [true, false] {
        let r = quirks(|q| q.parasite_host_loss_bridge_test = on);
        let mut w = game_of(&r);
        lake(&mut w, cell(2, 0), 1);
        let host = spawn(&mut w, Kind::Vehicle, "BOAT", 1, cell(2, 0), 0);
        let p = spawn(&mut w, Kind::Vehicle, "DRONE", 0, cell(-2, 0), 64);
        shoot(&mut w, p, Target::Object(host));
        // Neither cell has a bridge: on or off, the drone is removed.
        glasnost_sim::objects::remove(&mut w, host);
        assert!(w.obj(p).is_none(), "quirk {on}");
        // The host's cell has a bridge, the water neighbour in front has none, and the host is
        // in the air, so the host-cell test fails and the fallback is reached.
        let mut w = game_of(&r);
        lake(&mut w, cell(2, 0), 1);
        let host = spawn(&mut w, Kind::Vehicle, "BOAT", 1, cell(2, 0), 0);
        let p = spawn(&mut w, Kind::Vehicle, "DRONE", 0, cell(-2, 0), 64);
        shoot(&mut w, p, Target::Object(host));
        let i = w.cells.idx(cell(2, 0)).unwrap();
        w.cells.deck[i] = true;
        w.obj_mut(host).unwrap().pos.z += 300;
        glasnost_sim::objects::remove(&mut w, host);
        assert_eq!(w.obj(p).is_some(), on, "quirk {on}");
        if on {
            assert_eq!(w.obj(p).unwrap().pos.cell(), cell(2, -1));
        }
    }
}

// spec: S-0614 claim 15 (the host-cell test fails on water; the neighbour in front is dry land: placed there)
#[test]
fn claim_15_neighbour_in_front() {
    let mut w = game();
    wet(&mut w, cell(2, 0));
    // The boat faces north; the cell north of it stays dry.
    let host = spawn(&mut w, Kind::Vehicle, "BOAT", 1, cell(2, 0), 0);
    let p = spawn(&mut w, Kind::Vehicle, "DRONE", 0, cell(-2, 0), 64);
    shoot(&mut w, p, Target::Object(host));
    assert_eq!(parasite::host_of(&w, p), Some(host));
    glasnost_sim::objects::remove(&mut w, host);
    assert_eq!(w.obj(p).unwrap().pos.cell(), cell(2, -1));
}

/// A boat host of house 1 on water at (2, 0) facing north, a drone of
/// house 0 latched on; the cell in front (2, -1) is dry.
fn boat_host(w: &mut World) -> (ObjectId, ObjectId) {
    wet(w, cell(2, 0));
    let host = spawn(w, Kind::Vehicle, "BOAT", 1, cell(2, 0), 0);
    let p = spawn(w, Kind::Vehicle, "DRONE", 0, cell(-2, 0), 64);
    shoot(w, p, Target::Object(host));
    assert_eq!(parasite::host_of(w, p), Some(host));
    (host, p)
}

// spec: S-0614 claim 23 and Edge cases (quirks.parasite_host_loss_failed_choice_origin: a vehicle-present refusal in the front neighbour; on: the vehicle parasite survives at world position (128, 128, 0), unattached and not held; off: removed silently; no other cell is tried)
#[test]
fn claim_23_failed_choice_vehicle_at_the_origin() {
    for on in [true, false] {
        let r = quirks(|q| q.parasite_host_loss_failed_choice_origin = on);
        let mut w = game_of(&r);
        let (host, p) = boat_host(&mut w);
        spawn(&mut w, Kind::Vehicle, "BUGGY", 0, cell(2, -1), 0);
        glasnost_sim::objects::remove(&mut w, host);
        assert_eq!(w.obj(p).is_some(), on, "quirk {on}");
        if on {
            let o = w.obj(p).unwrap();
            assert_eq!((o.pos.x, o.pos.y, o.pos.z), (128, 128, 0));
            assert_eq!(o.layer, Layer::Ground, "not marked on a bridge");
            assert_eq!(parasite::host_of(&w, p), None);
            assert!(!parasite::held(&w, p));
        }
    }
}

// spec: S-0614 claim 23 (an accepted fallback point in a high-bridge cell: on the deck, at the point's surface plus 416, exactly when the host stood on a bridge or the point's terrain is below the host cell's centre terrain; otherwise on the ground)
#[test]
fn claim_23_fallback_surface() {
    for (front_level, deck) in [(1, false), (0, true)] {
        let mut w = game();
        for c in [cell(2, 0), cell(2, -1)] {
            w.cells.grid.get_mut(c).unwrap().level = 1;
        }
        w.cells.grid.get_mut(cell(2, -1)).unwrap().level = front_level;
        let i = w.cells.idx(cell(2, -1)).unwrap();
        w.cells.deck[i] = true;
        let (host, p) = boat_host(&mut w);
        glasnost_sim::objects::remove(&mut w, host);
        let o = w.obj(p).expect("placed");
        assert_eq!(o.pos.cell(), cell(2, -1));
        let ground = w.surface_at(o.pos.x, o.pos.y);
        if deck {
            assert_eq!(o.layer, Layer::Deck, "front level {front_level}");
            assert_eq!(o.pos.z, ground + 416);
        } else {
            assert_eq!(o.layer, Layer::Ground, "front level {front_level}");
            assert_eq!(o.pos.z, ground);
        }
    }
}

/// Land west of column `x0`, water from it eastwards, around the middle.
fn shore(w: &mut World, x0: i32) {
    for dy in -4..=4 {
        for dx in x0..x0 + 5 {
            wet(w, cell(dx, dy));
        }
    }
}

// spec: S-0614 Edge cases (quirks.parasite_exit_tests_host_cell: an amphibious host on water next to the shore, its water neighbours taken; on: removed, the host's cell is water; off: the chosen dry cell passes)
#[test]
fn quirk_exit_tests_host_cell() {
    for on in [true, false] {
        let r = quirks(|q| q.parasite_exit_tests_host_cell = on);
        let mut w = game_of(&r);
        shore(&mut w, 3);
        let host = spawn(&mut w, Kind::Vehicle, "HOVER", 1, cell(3, 0), 0);
        for (dx, dy) in [(3, -1), (3, 1), (4, -1), (4, 0), (4, 1)] {
            spawn(&mut w, Kind::Vehicle, "BOAT", 1, cell(dx, dy), 0);
        }
        let p = spawn(&mut w, Kind::Vehicle, "DRONE", 0, cell(-2, 0), 64);
        shoot(&mut w, p, Target::Object(host));
        assert_eq!(parasite::host_of(&w, p), Some(host));
        parasite::leave_living(&mut w, p, Leave::Exit);
        assert_eq!(w.obj(p).is_none(), on, "quirk {on}");
        if !on {
            let c = w.obj(p).unwrap().pos.cell();
            assert_eq!(w.cells.cell(c).unwrap().land, land::CLEAR);
        }
    }
}

// spec: S-0614 Edge cases and S-0212 table T5 (quirks.parasite_exit_search_host_mobility: an amphibious host on the shore, its dry neighbours taken; on: the host's mobility offers water, where the drone cannot be placed; off: the drone's own mobility finds land)
#[test]
fn quirk_exit_search_host_mobility() {
    for on in [true, false] {
        let r = quirks(|q| q.parasite_exit_search_host_mobility = on);
        let mut w = game_of(&r);
        shore(&mut w, 3);
        let host = spawn(&mut w, Kind::Vehicle, "HOVER", 1, cell(2, 0), 0);
        for (dx, dy) in [(1, -1), (1, 0), (1, 1), (2, -1), (2, 1)] {
            spawn(&mut w, Kind::Vehicle, "BUGGY", 1, cell(dx, dy), 0);
        }
        let p = spawn(&mut w, Kind::Vehicle, "DRONE", 0, cell(-3, 0), 64);
        shoot(&mut w, p, Target::Object(host));
        assert_eq!(parasite::host_of(&w, p), Some(host));
        parasite::leave_living(&mut w, p, Leave::Exit);
        assert_eq!(w.obj(p).is_none(), on, "quirk {on}");
    }
}

// spec: S-0614 claim 16, F5 and Edge cases (quirks.cull_experience_path: on, an unscored host still gives F5's experience; off, a destruction credited under S-0311 gives none)
#[test]
fn quirk_cull_experience_path() {
    for on in [true, false] {
        let mut r = quirks(|q| q.cull_experience_path = on);
        r.sim.vehicles.get_mut("BOAT").unwrap().combat.dont_score = Some(true);
        let (mut w, squid, boat, t) = grappled(&r);
        w.obj_mut(boat).unwrap().health = 50;
        run_to(&mut w, t + 1 + 69 + 1);
        assert!(naval::sinking(&w, boat));
        let e = w.obj(squid).unwrap().combat.experience;
        assert_eq!(e == F32::ZERO, !on, "quirk {on}");
    }
}

// ---------------------------------------------------------------------------
// Retaliation, order, re-selection (claims 17 to 19)
// ---------------------------------------------------------------------------

// spec: S-0614 claim 17 and S-0309 claim 18 (a host never retaliates against its own parasite)
#[test]
fn claim_17_no_retaliation_against_the_parasite() {
    let mut w = game();
    let host = spawn(&mut w, Kind::Vehicle, "CART", 1, cell(2, 0), 0);
    let p = spawn(&mut w, Kind::Vehicle, "DRONE", 0, cell(0, 0), 64);
    shoot(&mut w, p, Target::Object(host));
    assert_eq!(parasite::host_of(&w, p), Some(host));
    glasnost_sim::combat::acquire::retaliate(&mut w, host, p);
    assert_ne!(w.obj(host).unwrap().target, Some(Target::Object(p)));
}

// spec: S-0614 claim 18 and State and ordering (two deliveries in one tick: the host earlier in the update order is damaged first)
#[test]
fn claim_18_parasites_act_in_their_hosts_order() {
    let mut r = ruleset();
    r.sim.weapons.get_mut("Latch").unwrap().rof = Some(1);
    let mut w = game_of(&r);
    let h1 = spawn(&mut w, Kind::Vehicle, "BUGGY", 1, cell(2, 0), 0);
    let h2 = spawn(&mut w, Kind::Vehicle, "BUGGY", 1, cell(2, 3), 0);
    // The parasites are created in the opposite order to their hosts.
    let p2 = spawn(&mut w, Kind::Vehicle, "DRONE", 0, cell(0, 3), 64);
    let p1 = spawn(&mut w, Kind::Vehicle, "DRONE", 0, cell(0, 0), 64);
    shoot(&mut w, p2, Target::Object(h2));
    shoot(&mut w, p1, Target::Object(h1));
    testkit::run(&mut w, 3);
    let order = w.update.order.clone();
    let i1 = order.iter().position(|&x| x == h1).unwrap();
    let i2 = order.iter().position(|&x| x == h2).unwrap();
    assert!(i1 < i2);
    assert!(
        !order.contains(&p1) && !order.contains(&p2),
        "no turn of their own"
    );
}

// spec: S-0614 claim 18 (the parasite acts after the host's movement: a hold first renewed by a delivery cannot prevent the movement already taken in that turn; the next turn's movement sees it)
#[test]
fn claim_18_delivery_after_the_hosts_movement() {
    // A short paralysis (claim 7) leaves the host free between deliveries.
    let mut r = ruleset();
    r.sim.warheads.get_mut("Grip").unwrap().paralyzes = Some(4);
    let mut w = game_of(&r);
    let host = spawn(&mut w, Kind::Vehicle, "BUGGY", 1, cell(2, 0), 0);
    let p = spawn(&mut w, Kind::Vehicle, "DRONE", 0, cell(0, 0), 64);
    shoot(&mut w, p, Target::Object(host));
    assert_eq!(parasite::host_of(&w, p), Some(host));
    w.tick = w.tick.max(w.orders_blocked_through + 1);
    assert!(orders::issue(
        &mut w,
        HouseId(1),
        orders::Order::Behaviour {
            actor: host,
            behaviour: glasnost_sim::rules::Behaviour::Move,
            attack_move: false,
            target: None,
            dest: Some(Target::Cell(cell(2, 12))),
        },
    ));
    // Skip deliveries until the host is under way (it turns first).
    for _ in 0..3 {
        next_hit(&mut w, host, 100).expect("a delivery");
    }
    let mut moved_on_delivery = None;
    for _ in 0..200 {
        let before = w.obj(host).unwrap().pos;
        let h0 = health(&w, host);
        testkit::run(&mut w, 1);
        if health(&w, host) < h0 {
            moved_on_delivery = Some(w.obj(host).unwrap().pos != before);
            break;
        }
    }
    assert_eq!(
        moved_on_delivery,
        Some(true),
        "it moved in the delivery's turn"
    );
    assert!(parasite::held(&w, host), "the delivery renews the hold");
    let after = w.obj(host).unwrap().pos;
    testkit::run(&mut w, 1);
    assert_eq!(w.obj(host).unwrap().pos, after, "held in the next turn");
}

// spec: S-0614 claim 20 (one attempt in exactly the cell: an enemy vehicle refuses a non-crushing vehicle parasite's return like an allied one; no retry in a clear neighbour, no attack on the obstruction)
#[test]
fn claim_20_enemy_vehicle_refuses_without_retry() {
    let mut w = game();
    let p = spawn(&mut w, Kind::Vehicle, "DRONE", 0, cell(0, 0), 64);
    assert!(fire::shot(&mut w, p, 0, Target::Cell(cell(4, 0))));
    let blocker = spawn(&mut w, Kind::Vehicle, "BUGGY", 1, cell(0, 0), 0);
    let h0 = health(&w, blocker);
    for _ in 0..100 {
        if w.obj(p).is_none() {
            break;
        }
        testkit::run(&mut w, 1);
    }
    assert!(w.obj(p).is_none(), "removed silently");
    assert_eq!(health(&w, blocker), h0);
}

// spec: S-0614 claim 22 (the host-cell air test: at least 208 leptons above the surface fails, 207 passes; a deck-standing host counts its height from the deck)
#[test]
fn claim_22_air_threshold() {
    for (lift, placed) in [(207, true), (208, false)] {
        let (mut w, p, host, _) = latched();
        w.obj_mut(host).unwrap().pos.z += lift;
        glasnost_sim::objects::remove(&mut w, host);
        // Below 208 the host's cell passes; at 208 the fallback neighbour in front is used.
        let hc = cell(2, 0);
        let o = w.obj(p).expect("placed");
        assert_eq!(o.pos.cell() == hc, placed, "lift {lift}");
    }
}

// spec: S-0614 claim 19 (re-selection is presentation: the imported flag resolves; selection is not game state)
#[test]
fn claim_19_reselect_flag_is_presentation() {
    let mut r = ruleset();
    r.sim.vehicles.get_mut("DRONE").unwrap().reselect_if_limboed = Some(true);
    let w = game_of(&r);
    let ti = w.rules().index_of(Kind::Vehicle, "DRONE").unwrap();
    let u = w.rules().unit(Kind::Vehicle, ti).unwrap();
    assert!(u.vehicle.reselect_if_limboed);
    let w = game();
    let ti = w.rules().index_of(Kind::Vehicle, "DRONE").unwrap();
    assert!(
        !w.rules()
            .unit(Kind::Vehicle, ti)
            .unwrap()
            .vehicle
            .reselect_if_limboed
    );
}

// ---------------------------------------------------------------------------
// Hooks into other specs
// ---------------------------------------------------------------------------

// spec: S-0214 claim 6 (may attack as a friend: a sonic weapon at a mobile unit with a parasite on it)
#[test]
fn s0214_sonic_friend_attack_on_a_carrier() {
    let mut r = ruleset();
    r.sim.vehicles.get_mut("CRUSHER").unwrap().primary = Some("Sonar".into());
    let mut w = game_of(&r);
    let friend = spawn(&mut w, Kind::Vehicle, "BUGGY", 0, cell(2, 0), 0);
    let sonic = spawn(&mut w, Kind::Vehicle, "CRUSHER", 0, cell(0, 2), 64);
    let p = spawn(&mut w, Kind::Vehicle, "DRONE", 1, cell(0, 0), 64);
    let act = |w: &World| {
        targeting::action(
            w,
            ALPHA,
            sonic,
            ClickOn::Object(friend),
            Modifiers::default(),
        )
    };
    assert_ne!(act(&w), targeting::Action::Attack);
    shoot(&mut w, p, Target::Object(friend));
    assert_eq!(parasite::host_of(&w, p), Some(friend));
    assert_eq!(act(&w), targeting::Action::Attack);
}

// spec: S-0614 claim 21 and Edge cases (quirks.parasite_carrier_loss_strands_unit: a carrying projectile removed without detonating strands the parasite off the map, unattached, health unchanged; off: claim 4's failed-latch return)
#[test]
fn claim_21_carrier_removed_without_detonation() {
    for on in [true, false] {
        let mut w = game_of(&quirks(|q| q.parasite_carrier_loss_strands_unit = on));
        let p = spawn(&mut w, Kind::Vehicle, "DRONE", 0, cell(0, 0), 64);
        let h0 = health(&w, p);
        assert!(fire::shot(&mut w, p, 0, Target::Cell(cell(4, 0))));
        let carrier = ps(&w, p).carrier.unwrap();
        let wp = w.projectiles[&carrier].weapon;
        glasnost_sim::combat::projectile::remove(&mut w, carrier);
        parasite::carrier_ended(&mut w, carrier, wp, false);
        let o = w.obj(p).expect("never removed");
        assert_eq!(o.on_map, !on, "stranded off the map with the switch on");
        assert_eq!(parasite::host_of(&w, p), None);
        assert_eq!(health(&w, p), h0);
        if on {
            testkit::run(&mut w, 300);
            assert!(w.obj(p).is_some_and(|o| !o.on_map), "no recovery");
        }
    }
}

// ---------------------------------------------------------------------------
// End to end, save and load
// ---------------------------------------------------------------------------

// spec: S-0614 claims 1, 3, 6, 7 and 15 (end to end: a click on an enemy buggy gives the drone an attack order; it fires, leaves the map, latches, delivers, holds the host, and is released when the host dies)
#[test]
fn end_to_end_drone_attack_order() {
    let mut w = game();
    let host = spawn(&mut w, Kind::Vehicle, "BUGGY", 1, cell(5, 0), 0);
    let p = spawn(&mut w, Kind::Vehicle, "DRONE", 0, cell(0, 0), 64);
    let res = targeting::click(&w, ALPHA, &[p], ClickOn::Object(host), Modifiers::default());
    assert!(!res.orders.is_empty(), "the click gives an order");
    w.tick = w.tick.max(w.orders_blocked_through + 1);
    for o in res.orders {
        assert!(orders::issue(&mut w, ALPHA, o));
    }
    let h0 = health(&w, host);
    let mut left = false;
    for _ in 0..300 {
        testkit::run(&mut w, 1);
        if !w.obj(p).unwrap().on_map {
            left = true;
        }
        if parasite::host_of(&w, p) == Some(host) {
            break;
        }
    }
    assert!(left, "the drone left the map");
    assert_eq!(parasite::host_of(&w, p), Some(host), "latched");
    let mut held = false;
    for _ in 0..40 {
        testkit::run(&mut w, 1);
        held |= parasite::held(&w, host);
    }
    assert!(held, "the host is held");
    assert!(health(&w, host) <= h0 - 2 * DMG, "deliveries");
    for _ in 0..400 {
        if w.obj(host).is_none() {
            break;
        }
        testkit::run(&mut w, 1);
    }
    assert!(w.obj(host).is_none(), "the host died");
    assert!(w.obj(p).is_some_and(|o| o.on_map), "released on host death");
    assert!(!parasite::held(&w, p));
}

// spec: S-0614 claims 12 and 13 (end to end: another release, a sonic hit, frees the drone held for 3 × R)
#[test]
fn end_to_end_release_by_a_sonic_hit() {
    let (mut w, p, host, _) = latched();
    testkit::run(&mut w, 20);
    let wh = warhead(&w, "Sonar");
    damage::apply(&mut w, host, hit(5, wh, None));
    assert!(w.obj(p).is_some_and(|o| o.on_map));
    let t = w.tick;
    testkit::run(&mut w, 3 * R);
    assert_eq!(w.tick, t + 3 * R);
    assert!(!parasite::held(&w, p));
}

// spec: S-0614 State and ordering (all parasite state is saved: a save and load mid-attachment gives the same hash and both copies continue identically)
#[test]
fn save_load_mid_attachment() {
    let (mut w, p, host, _) = latched();
    testkit::run(&mut w, 7);
    let (mut s, squid, boat, _) = grappled(&ruleset());
    testkit::run(&mut s, 40);
    for (w, a, b) in [(&mut w, p, host), (&mut s, squid, boat)] {
        let bytes = save::save(w);
        let mut l = save::load(&bytes, w.rules_arc(), w.catalogue_arc()).expect("load");
        assert_eq!(save::state_hash(&l), save::state_hash(w));
        assert_eq!(parasite::host_of(&l, a), Some(b));
        testkit::run(w, 90);
        testkit::run(&mut l, 90);
        assert_eq!(save::state_hash(&l), save::state_hash(w));
    }
}

// spec: S-0614 claim 10 (the squid on water: a synthetic naval vessel on water cells, grappled, struck, then released by a sonic hit)
#[test]
fn squid_on_water() {
    let (mut w, squid, boat, _) = grappled(&ruleset());
    let h0 = health(&w, boat);
    testkit::run(&mut w, 120);
    assert!(health(&w, boat) < h0);
    let wh = warhead(&w, "Sonar");
    damage::apply(&mut w, boat, hit(1, wh, None));
    let o = w.obj(squid).unwrap();
    assert!(o.on_map);
    assert!(parasite::water_tile(&w, o.pos.cell()));
}

// spec: S-0614 claim 12 (the host starts curtain protection: 50 ticks of suppression just before release, so a drone parasite is removed silently and a naval parasite exits; the host's hold ends)
#[test]
fn claim_12_curtain_protection() {
    let (mut w, p, host, _) = latched();
    let until = w.tick + 100;
    set_hold(&mut w, host, until);
    parasite::host_curtain_started(&mut w, host);
    assert!(w.obj(p).is_none(), "the drone is removed silently");
    assert_eq!(parasite::parasite_of(&w, host), None);
    assert!(!parasite::held(&w, host), "the host's hold ends");
    let (mut w, squid, boat, _) = grappled(&ruleset());
    parasite::host_curtain_started(&mut w, boat);
    assert_eq!(parasite::host_of(&w, squid), None);
    assert!(
        w.obj(squid).is_some_and(|o| o.on_map),
        "a naval parasite exits"
    );
}

// spec: S-0614 claim 18 (deliveries due in the same tick are assigned in host order, not parasite order)
#[test]
fn claim_18_deliveries_in_host_order() {
    let mut r = ruleset();
    // A delivery every host turn, so that both deliveries fall in one tick.
    r.sim.weapons.get_mut("Latch").unwrap().rof = Some(1);
    let mut w = game_of(&r);
    let h1 = spawn(&mut w, Kind::Vehicle, "BUGGY", 1, cell(2, 0), 0);
    let h2 = spawn(&mut w, Kind::Vehicle, "BUGGY", 1, cell(2, 3), 0);
    // The parasites are created in the opposite order to their hosts.
    let p2 = spawn(&mut w, Kind::Vehicle, "DRONE", 0, cell(0, 3), 64);
    let p1 = spawn(&mut w, Kind::Vehicle, "DRONE", 0, cell(0, 0), 64);
    // Both shots in one tick, so that both latch in the same tick and deliver together.
    assert!(fire::shot(&mut w, p2, 0, Target::Object(h2)));
    assert!(fire::shot(&mut w, p1, 0, Target::Object(h1)));
    let mut seen = None;
    for _ in 0..300 {
        w.effects.clear();
        testkit::run(&mut w, 1);
        let hits: Vec<ObjectId> = w
            .effects
            .iter()
            .filter_map(|e| match e {
                Effect::Hit {
                    id,
                    attacker: Some(a),
                    ..
                } if *a == p1 || *a == p2 => Some(*id),
                _ => None,
            })
            .collect();
        if hits.len() == 2 {
            seen = Some(hits);
            break;
        }
    }
    assert_eq!(seen, Some(vec![h1, h2]), "host order, not parasite order");
}

// spec: S-0513 claim 34 and Constants (a teleport warp releases a naval parasite with 500 ticks of suppression; S-0614 claim 12: it exits at launch)
#[test]
fn s0513_claim_34_warp_releases_a_naval_parasite_with_suppression() {
    let (mut w, squid, boat, _) = grappled(&ruleset());
    let t = w.tick;
    parasite::superweapon_teleport(&mut w, boat);
    assert_eq!(parasite::host_of(&w, squid), None);
    assert_eq!(ps(&w, squid).suppressed_until, Some(t + 500));
}

/// Runs until the parasite's projectile has ended.
fn shoot_to_end(w: &mut World, p: ObjectId) {
    for _ in 0..200 {
        if ps(w, p).carrier.is_none() {
            return;
        }
        testkit::run(w, 1);
    }
    panic!("the projectile never ended");
}

type Higher = (&'static str, fn(&mut WarheadType));
const HIGHER: [Higher; 3] = [
    ("takeover", |h| h.mind_control = Some(true)),
    ("bomb planting", |h| h.ivan_bomb = Some(true)),
    ("charging", |h| h.electric_assault = Some(true)),
];

// spec: S-0614 claim 24 (answers Q-1032) and Edge cases (quirks.parasite_suppressed_latch_strands_unit on): takeover, bomb planting or charging selected at a detonation strands the carried parasite off the map, unattached and inactive, extant with its health, for 300 ticks and more, with no return, recovery or removal; the target may be latchable, a cell, or no longer there
#[test]
fn claim_24_a_higher_hit_strands_the_carried_parasite() {
    for (name, set) in HIGHER {
        for case in ["latchable", "cell", "lost"] {
            let mut r = ruleset();
            set(r.sim.warheads.get_mut("Grip").unwrap());
            let mut w = game_of(&r);
            let host = spawn(&mut w, Kind::Vehicle, "BUGGY", 1, cell(2, 0), 0);
            let p = spawn(&mut w, Kind::Vehicle, "DRONE", 0, cell(0, 0), 64);
            let h0 = health(&w, p);
            let target = if case == "cell" {
                Target::Cell(cell(2, 0))
            } else {
                Target::Object(host)
            };
            assert!(fire::shot(&mut w, p, 0, target), "{name} {case}");
            if case == "lost" {
                glasnost_sim::objects::remove(&mut w, host);
            }
            shoot_to_end(&mut w, p);
            let o = w
                .obj(p)
                .unwrap_or_else(|| panic!("{name} {case}: never removed"));
            assert!(!o.on_map, "{name} {case}: off the map");
            assert_eq!(parasite::host_of(&w, p), None, "{name} {case}");
            assert_eq!(health(&w, p), h0, "{name} {case}");
            assert_eq!(
                ps(&w, p).hold_until,
                None,
                "{name} {case}: no recovery timer"
            );
            testkit::run(&mut w, 300);
            assert!(
                w.obj(p).is_some_and(|o| !o.on_map && health(&w, p) == h0),
                "{name} {case}: still stranded after 300 ticks"
            );
        }
    }
}

// spec: S-0614 claim 24 (quirks.parasite_suppressed_latch_strands_unit off: after the higher selected effect the surviving parasite attempts claim 4's return, placed in its firing cell, and never attaches, even against a latchable host)
#[test]
fn claim_24_switch_off_returns_the_parasite_without_attaching() {
    for (name, set) in HIGHER {
        let mut r = ruleset();
        set(r.sim.warheads.get_mut("Grip").unwrap());
        r.globals.quirks.parasite_suppressed_latch_strands_unit = false;
        let mut w = game_of(&r);
        let host = spawn(&mut w, Kind::Vehicle, "BUGGY", 1, cell(2, 0), 0);
        let p = spawn(&mut w, Kind::Vehicle, "DRONE", 0, cell(0, 0), 64);
        assert!(fire::shot(&mut w, p, 0, Target::Object(host)));
        shoot_to_end(&mut w, p);
        assert!(w.obj(p).is_some_and(|o| o.on_map), "{name}: returned");
        assert_eq!(parasite::host_of(&w, p), None, "{name}: never attached");
    }
}

// spec: S-0614 claim 4 and claim 24 (control: attachment selected against a target that is gone at detonation returns under claim 4, it is not stranded)
#[test]
fn claim_24_control_selected_attachment_to_a_lost_target_returns() {
    let mut w = game_of(&ruleset());
    let host = spawn(&mut w, Kind::Vehicle, "BUGGY", 1, cell(2, 0), 0);
    let p = spawn(&mut w, Kind::Vehicle, "DRONE", 0, cell(0, 0), 64);
    assert!(fire::shot(&mut w, p, 0, Target::Object(host)));
    glasnost_sim::objects::remove(&mut w, host);
    shoot_to_end(&mut w, p);
    assert!(w.obj(p).is_some_and(|o| o.on_map), "claim 4's return");
}

// spec: S-0614 claims 1, 3 and 4 (revision 8, scenario P14) and S-0514 claim 20: attachment is the selected special hit only when no higher row (takeover, bomb planting, charging) is enabled; the firing-time departure and the latch block still happen, the higher row suppresses attachment and the ordinary damage (the carried parasite's fate is claim 24, tested below)
#[test]
fn p14_a_higher_row_suppresses_attachment() {
    for (name, set) in HIGHER {
        let mut r = ruleset();
        set(r.sim.warheads.get_mut("Grip").unwrap());
        let mut w = game_of(&r);
        let host = spawn(&mut w, Kind::Vehicle, "BUGGY", 1, cell(2, 0), 0);
        let p = spawn(&mut w, Kind::Vehicle, "DRONE", 0, cell(0, 0), 64);
        let t = w.tick;
        let h0 = health(&w, host);
        assert!(fire::shot(&mut w, p, 0, Target::Object(host)));
        assert!(!w.obj(p).unwrap().on_map, "{name}: it left the map");
        assert_eq!(ps(&w, host).latch_block_until, Some(t + 20), "{name}");
        shoot_to_end(&mut w, p);
        assert_eq!(parasite::host_of(&w, p), None, "{name}: no attachment");
        assert_eq!(health(&w, host), h0, "{name}: no ordinary damage");
    }
}

// spec: S-0614 claim 3 (revision 8, scenario P14) and S-0514 claim 20: with attachment selected, enabling temporal erasure as well changes nothing: the parasite attaches
#[test]
fn p14_selected_attachment_outranks_erasure() {
    let mut r = ruleset();
    r.sim.warheads.get_mut("Grip").unwrap().temporal = Some(true);
    let mut w = game_of(&r);
    let host = spawn(&mut w, Kind::Vehicle, "BUGGY", 1, cell(2, 0), 0);
    let p = spawn(&mut w, Kind::Vehicle, "DRONE", 0, cell(0, 0), 64);
    assert!(fire::shot(&mut w, p, 0, Target::Object(host)));
    shoot_to_end(&mut w, p);
    assert_eq!(parasite::host_of(&w, p), Some(host));
}

/// The drone rockings the effects report: host and side.
fn drone_rocks(w: &World) -> Vec<(ObjectId, i32)> {
    w.effects
        .iter()
        .filter_map(|e| match e {
            Effect::DroneRock { host, side } => Some((*host, *side)),
            _ => None,
        })
        .collect()
}

// spec: S-0614 claim 9 and F2 (each delivery to a host that is not infantry rocks it, to the side its one draw R(0, 1) chose; the rocking is presentation and takes no further draw)
#[test]
fn claim_9_each_delivery_reports_its_rocking_side() {
    let (mut w, _p, host, _) = latched();
    w.effects.clear();
    let mut expect = w.rng.clone();
    next_hit(&mut w, host, 5).unwrap();
    // The only draw of the first delivery's tick in this scene is the rocking side.
    let side = expect.ranged(0, 1, w.rules().quirks.ranged_draw_half_span);
    assert_eq!(drone_rocks(&w), [(host, side)]);
    for _ in 0..2 {
        w.effects.clear();
        next_hit(&mut w, host, 50).unwrap();
        let r = drone_rocks(&w);
        assert_eq!(r.len(), 1, "one rocking per delivery");
        assert!(r[0].0 == host && (0..=1).contains(&r[0].1));
    }
}

// spec: S-0614 claim 6 and claim 9 (an infantry host: no rocking choice and no rocking)
#[test]
fn claim_9_an_infantry_host_is_not_rocked() {
    let mut w = game();
    let man = spawn(&mut w, Kind::Infantry, "WALKER", 1, cell(2, 0), 0);
    let p = spawn(&mut w, Kind::Vehicle, "DRONE", 0, cell(0, 0), 64);
    shoot(&mut w, p, Target::Object(man));
    w.effects.clear();
    testkit::run(&mut w, 2);
    assert!(drone_rocks(&w).is_empty());
}

// spec: S-0614 claim 10 and table G (the grab's end starts a push, whose one draw R(0, 1) chooses the way it rocks; reported to the drawing)
#[test]
fn claim_10_each_push_reports_its_way() {
    let (mut w, _s, boat, t) = grappled(&ruleset());
    let g = t + 1;
    let half = w.rules().quirks.ranged_draw_half_span;
    let pushes = |w: &World| -> Vec<(ObjectId, i32)> {
        w.effects
            .iter()
            .filter_map(|e| match e {
                Effect::GrapplePush { host, way } => Some((*host, *way)),
                _ => None,
            })
            .collect()
    };
    run_to(&mut w, g + 29);
    w.effects.clear();
    let mut expect = w.rng.clone();
    parasite::host_turn(&mut w, boat);
    let way = expect.ranged(0, 1, half);
    assert_eq!(pushes(&w), [(boat, way)], "the grab's end");
    w.effects.clear();
    testkit::run(&mut w, 1);
    assert!(pushes(&w).is_empty(), "no push between");
}

// ---------------------------------------------------------------------------
// The grab animation (claim 25)
// ---------------------------------------------------------------------------

/// The grab animation's fixed type name (S-0614 Import mapping).
const GRAB: &str = "SQDG";

/// The ruleset with a grab animation type of 300 frames and a stage delay
/// of 2 ticks (P15's test type has a delay of exactly 2), or of a random
/// rate when `random`.
fn grab_ruleset(random: bool) -> Ruleset {
    let mut r = ruleset();
    r.registries
        .entry("animations".into())
        .or_default()
        .push(GRAB.into());
    r.sim.animations.insert(
        GRAB.into(),
        glasnost_rules::sim::AnimationType {
            frame_count: Some(300),
            rate: Some(2),
            // spec: S-0109 claim 3 and F1 (rates 40 and 30 give the delays 22 and 30: a ranged draw)
            random_rate: random.then_some([40, 30]),
            ..Default::default()
        },
    );
    r
}

/// The grab animations in play.
fn grabs(w: &World) -> Vec<(ObjectId, glasnost_sim::anims::Anim)> {
    w.anims
        .anims
        .iter()
        .filter(|(_, a)| matches!(a.role, glasnost_sim::anims::Role::Grab { .. }))
        .map(|(&id, a)| (id, a.clone()))
        .collect()
}

fn grab_effects(w: &World) -> usize {
    w.effects
        .iter()
        .filter(|e| matches!(e, Effect::Anim { name, .. } if name == GRAB))
        .count()
}

/// The 8-direction of the host's body facing now (S-0204 F4).
fn direction(w: &World, host: ObjectId) -> i32 {
    i32::from(glasnost_num::geom::eight_direction(
        w.obj(host).unwrap().body.value(w.tick),
    ))
}

// spec: S-0614 claim 25 (created once per hold, in the grab's first turn g, at the host's position, attached to the host with no displacement, with the parasite's house and the host's cell; not at the grab's end, a push start or a strike)
#[test]
fn claim_25_the_grab_animation_is_created_once_in_the_grabs_first_turn() {
    let (mut w, squid, boat, t) = grappled(&grab_ruleset(false));
    let g = t + 1;
    assert!(grabs(&w).is_empty(), "nothing before the grab's first turn");
    w.effects.clear();
    testkit::run(&mut w, 1);
    assert_eq!(w.tick, g + 1);
    let made = grabs(&w);
    assert_eq!(made.len(), 1);
    assert_eq!(grab_effects(&w), 1, "one creation report");
    let (id, a) = &made[0];
    assert_eq!(ps(&w, squid).grab_anim, Some(*id));
    let boat_at = w.obj(boat).unwrap().pos;
    assert_eq!(
        a.pos,
        Some(boat_at),
        "at the host's position, height included"
    );
    assert_eq!(
        a.owner,
        Some((boat, glasnost_sim::world::Pos::new(0, 0, 0)))
    );
    assert_eq!(
        a.role,
        glasnost_sim::anims::Role::Grab {
            parasite: squid,
            house: Some(ALPHA),
            cell: Some(boat_at.cell()),
        }
    );
    // No new one at the grab's end, a push start or a strike.
    for _ in 0..150 {
        w.effects.clear();
        testkit::run(&mut w, 1);
        assert_eq!(grab_effects(&w), 0, "tick {}", w.tick);
        assert_eq!(grabs(&w).len(), 1);
    }
    assert!(w.tick > g + 109, "past two strikes");
}

// spec: S-0614 claim 25 and Import mapping (no animation type has the fixed name: none is created, no draw is taken for it, and the cycle runs as claim 10 gives)
#[test]
fn claim_25_an_absent_type_shows_none_and_the_cycle_runs() {
    let (mut w, _squid, boat, t) = grappled(&ruleset());
    let g = t + 1;
    testkit::run(&mut w, 1);
    assert!(grabs(&w).is_empty());
    assert_eq!(next_hit(&mut w, boat, 200), Some(g + 69));
    // A type with a random rate takes its creation draw in turn g; one without takes none.
    let rng = |r: &Ruleset| {
        let (mut w, _, _, _) = grappled(r);
        let before = w.rng.clone();
        testkit::run(&mut w, 1);
        (before, w.rng.clone())
    };
    let (b0, a0) = rng(&ruleset());
    let (b1, a1) = rng(&grab_ruleset(false));
    let (b2, a2) = rng(&grab_ruleset(true));
    assert_eq!((&b0, &a0), (&b1, &a1), "no random rate: no draw");
    assert_eq!(b0, b2, "nothing drawn before the grab's first turn");
    assert_ne!(a0, a2, "the random rate's draw in turn g");
}

// spec: S-0614 claim 25 and table G (k = b + 10 × d + j at each step of phase frame j to 1 … 9: the grab's turns 3, 6, …, 27 with b = 0, a push's turns 4, 8, …, 36 with b = 160 for σ = +1 and 80 for σ = −1; each setting restarts the clock with a 128-tick delay and leaves the direction alone; nothing at a step to 10 or in a push's first three turns)
#[test]
fn claim_25_the_stage_follows_the_cycle() {
    let (mut w, squid, boat, t) = grappled(&grab_ruleset(false));
    let g = t + 1;
    run_to(&mut w, g + 1);
    let id = ps(&w, squid).grab_anim.unwrap();
    let stage = |w: &World| {
        let a = &w.anims.anims[&id];
        (a.player.stage(), a.player.delay())
    };
    // The grab's first two turns: the type's own fields (rate 2 gives the
    // delay ⌊900 / 2⌋ = 450, S-0109 F1; no advance yet).
    assert_eq!(stage(&w), (0, 450));
    testkit::run(&mut w, 1);
    assert_eq!(stage(&w), (0, 450), "turn 2");
    for j in 1..=9 {
        // Turn 3j (tick g + 3j − 1) sets j; the turns between keep it.
        run_to(&mut w, g + 3 * j);
        let d = direction(&w, boat);
        assert_eq!(stage(&w), (10 * d + j, 128), "grab frame {j}");
        if j < 9 {
            testkit::run(&mut w, 2);
            assert_eq!(stage(&w).0, 10 * d + j, "between settings");
        }
    }
    // The grab's end (turn 30, the step to 10) sets nothing; the push's first three turns neither.
    w.effects.clear();
    run_to(&mut w, g + 30);
    let way = w
        .effects
        .iter()
        .find_map(|e| match e {
            Effect::GrapplePush { way, .. } => Some(*way),
            _ => None,
        })
        .expect("the first push");
    let held = stage(&w).0;
    run_to(&mut w, g + 29 + 3);
    assert_eq!(stage(&w).0, held, "a push's first three turns set nothing");
    let b = if way == 0 { 160 } else { 80 };
    for j in 1..=9 {
        // Push turn 4j: tick (g + 29) + 4j − 1.
        run_to(&mut w, g + 29 + 4 * j);
        let d = direction(&w, boat);
        assert_eq!(stage(&w), (b + 10 * d + j, 128), "push frame {j}");
    }
    // The push's turn 40 (step to 10) and the strike set nothing.
    let last = stage(&w).0;
    run_to(&mut w, g + 29 + 41);
    assert_eq!(stage(&w).0, last);
    assert_eq!(ps(&w, squid).push_way.map(|_| ()), Some(()));
}

// spec: S-0614 claim 25 (removed at once when the parasite leaves off water, at a cull and when the strike's own hit destroys the host, which then sinks without it)
#[test]
fn claim_25_removal() {
    let shown = |w: &World| grabs(w).len();
    // Off water.
    let (mut w, squid, boat, _) = grappled(&grab_ruleset(false));
    testkit::run(&mut w, 3);
    assert_eq!(shown(&w), 1);
    let c = w.obj(boat).unwrap().pos.cell();
    w.cells.grid.get_mut(c).unwrap().tile = 0;
    testkit::run(&mut w, 1);
    assert_eq!(parasite::host_of(&w, squid), None);
    assert_eq!(shown(&w), 0, "off water");
    assert_eq!(ps(&w, squid).grab_anim, None);
    // A cull.
    let (mut w, squid, boat, t) = grappled(&grab_ruleset(false));
    w.obj_mut(boat).unwrap().health = 100;
    run_to(&mut w, t + 1 + 69);
    assert_eq!(shown(&w), 1);
    testkit::run(&mut w, 1);
    assert!(naval::sinking(&w, boat));
    assert_eq!(parasite::host_of(&w, squid), None);
    assert_eq!(shown(&w), 0, "the cull");
    // The destroying strike (no cull flag).
    let mut r = grab_ruleset(false);
    r.sim.warheads.get_mut("Squeeze").unwrap().culling = None;
    let (mut w, squid, boat, t) = grappled(&r);
    w.obj_mut(boat).unwrap().health = 20;
    run_to(&mut w, t + 1 + 69);
    assert_eq!(shown(&w), 1);
    testkit::run(&mut w, 1);
    assert!(health(&w, boat) <= 0, "destroyed by the strike");
    assert_eq!(shown(&w), 0, "the destroying strike");
    assert_eq!(ps(&w, squid).grab_anim, None);
}

// spec: S-0614 claim 25 and State and ordering (the grab animation and the cycle's phase are saved: a save and load mid-grab and mid-push gives the same hash and both copies continue identically)
#[test]
fn claim_25_save_load_mid_hold() {
    for ticks in [5, 45] {
        let (mut w, squid, _boat, _) = grappled(&grab_ruleset(true));
        testkit::run(&mut w, ticks);
        assert!(ps(&w, squid).grab_anim.is_some());
        let bytes = save::save(&w);
        let mut l = save::load(&bytes, w.rules_arc(), w.catalogue_arc()).expect("load");
        assert_eq!(save::state_hash(&l), save::state_hash(&w));
        testkit::run(&mut w, 60);
        testkit::run(&mut l, 60);
        assert_eq!(save::state_hash(&l), save::state_hash(&w));
        assert_eq!(grabs(&l), grabs(&w));
    }
}
