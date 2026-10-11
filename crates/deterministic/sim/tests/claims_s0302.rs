//! Claim tests for damage resolution (S-0302) on synthetic data: invented
//! types, weapons and warheads, every number our own. Expected values are
//! computed from the spec's formulas with `glasnost_num`'s soft floats or
//! taken from the spec's own scenarios.

use glasnost_map::CellXY;
use glasnost_num::decimal::nearest_double;
use glasnost_num::{F64, Rounding};
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{
    ArmourClass, BuildingType, Decimal, TerrainObjectType, UnitType, WarheadType, WeaponType,
};
use glasnost_sim::combat::area::{self, Detonation};
use glasnost_sim::combat::damage::{self, Hit, HitResult};
use glasnost_sim::combat::data::{WarheadId, WeaponId};
use glasnost_sim::combat::effects::Effect;
use glasnost_sim::rules::Kind;
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, Pos, World};

const SIZE: i32 = 24;
const RZ: Rounding = Rounding::TowardZero;
const ALPHA: u8 = 0;
const BETA: u8 = 1;

fn dec(s: &str) -> Decimal {
    Decimal(s.into())
}

fn verses(v: &[&str]) -> Option<Vec<Decimal>> {
    Some(v.iter().map(|s| dec(s)).collect())
}

fn every(v: &str) -> Option<Vec<Decimal>> {
    Some(vec![dec(v); 11])
}

fn d(n: i64) -> F64 {
    F64::from_i64(n, RZ)
}

fn register(r: &mut Ruleset, reg: &str, name: &str) {
    let l = r.registries.entry(reg.into()).or_default();
    if !l.iter().any(|n| n == name) {
        l.push(name.into());
    }
}

/// An unarmed test vehicle type of strength 1000, class *none* unless `f`
/// says otherwise.
fn vehicle(r: &mut Ruleset, name: &str, f: impl FnOnce(&mut UnitType)) {
    let mut t = r.sim.vehicles["BUGGY"].clone();
    t.strength = Some(1000);
    t.primary = None;
    t.combat.armor = None;
    f(&mut t);
    register(r, "vehicles", name);
    r.sim.vehicles.insert(name.into(), t);
}

/// An unarmed test infantry type of strength 1000, class *none*.
fn infantry(r: &mut Ruleset, name: &str, f: impl FnOnce(&mut UnitType)) {
    let mut t = r.sim.infantry["WALKER"].clone();
    t.strength = Some(1000);
    t.primary = None;
    // Sequences of its own, so that it can go prone (S-0107 claim 6).
    t.crawls = Some(true);
    for k in ["stand", "walk", "go_prone", "prone", "crawl"] {
        t.sequences.insert(
            k.into(),
            glasnost_rules::sim::SequenceEntry {
                start: 0,
                count: 4,
                multiplier: 0,
                direction: None,
            },
        );
    }
    f(&mut t);
    register(r, "infantry", name);
    r.sim.infantry.insert(name.into(), t);
}

/// An unarmed test building type of strength 1000, one cell.
fn building(r: &mut Ruleset, name: &str, f: impl FnOnce(&mut BuildingType)) {
    let mut t = BuildingType {
        strength: Some(1000),
        foundation: Some([1, 1]),
        ..Default::default()
    };
    f(&mut t);
    register(r, "buildings", name);
    r.sim.buildings.insert(name.into(), t);
}

fn warhead(r: &mut Ruleset, name: &str, f: impl FnOnce(&mut WarheadType)) {
    let mut h = WarheadType::default();
    f(&mut h);
    r.sim.warheads.insert(name.into(), h);
}

fn weapon(r: &mut Ruleset, name: &str, f: impl FnOnce(&mut WeaponType)) {
    let mut t = WeaponType {
        projectile: Some("Bullet".into()),
        ..Default::default()
    };
    f(&mut t);
    r.sim.weapons.insert(name.into(), t);
}

fn wh(w: &World, name: &str) -> WarheadId {
    w.rules()
        .combat
        .warheads
        .iter()
        .position(|h| h.name == name)
        .unwrap_or_else(|| panic!("no warhead {name}")) as u32
}

fn wp(w: &World, name: &str) -> WeaponId {
    w.rules().combat.weapon_index[name]
}

/// A direct hit of `amount` with warhead `h`, no attacker.
fn hit(amount: i32, h: WarheadId) -> Hit {
    Hit {
        amount,
        warhead: Some(h),
        distance: 0,
        attacker: None,
        house: None,
        ignore_defences: false,
        no_escape: false,
    }
}

fn at(dx: i32, dy: i32) -> CellXY {
    testkit::mid(SIZE, dx, dy)
}

fn put(w: &mut World, kind: Kind, name: &str, house: u8, dx: i32) -> ObjectId {
    testkit::spawn(w, kind, name, house, at(dx, 0), 0)
}

fn health(w: &World, id: ObjectId) -> i32 {
    w.obj(id).map_or(0, |o| o.health)
}

fn set_health(w: &mut World, id: ObjectId, h: i32) {
    w.obj_mut(id).unwrap().health = h;
}

/// Applies a hit (the hit evaluated first, so it may read the world).
macro_rules! apply {
    ($w:expr, $t:expr, $h:expr) => {{
        let h = $h;
        damage::apply(&mut $w, $t, h)
    }};
}

/// Applies a hit and returns (health lost, result).
macro_rules! loss {
    ($w:expr, $t:expr, $h:expr) => {{
        let h = $h;
        loss_fn(&mut $w, $t, h)
    }};
}

/// Applies a hit and returns (health lost, result).
fn loss_fn(w: &mut World, id: ObjectId, h: Hit) -> (i32, HitResult) {
    let before = health(w, id);
    let res = damage::apply(w, id, h);
    (before - health(w, id), res)
}

fn country(r: &mut Ruleset, name: &str, f: impl FnOnce(&mut glasnost_rules::sim::Country)) {
    f(r.sim.countries.get_mut(name).unwrap());
}

fn make_veteran(w: &mut World, id: ObjectId) {
    w.obj_mut(id).unwrap().combat.experience = d(1).to_f32(RZ);
}

// ---------------------------------------------------------------------------
// Claim 1: hits, their markers and where they come from.
// ---------------------------------------------------------------------------

// spec: S-0302 claim 1 (a projectile's detonation does not ignore defences and carries the firing object and its owner, so the kill is credited to the firer's house)
#[test]
fn detonation_hits_take_defences_and_carry_the_source() {
    let mut r = testkit::combat_ruleset();
    vehicle(&mut r, "BOX", |t| t.strength = Some(40));
    warhead(&mut r, "PLAIN", |h| h.verses = every("1"));
    // The defender's country halves every vehicle hit.
    country(&mut r, "Betaland", |c| c.armor_units_mult = Some(dec("2")));
    let mut w = testkit::game_with(&r, SIZE, 1);
    let firer = put(&mut w, Kind::Vehicle, "BOX", ALPHA, -6);
    let target = put(&mut w, Kind::Vehicle, "BOX", BETA, 0);
    let p = w.obj(target).unwrap().pos;
    let plain = wh(&w, "PLAIN");
    let det = |damage| Detonation {
        point: p,
        damage,
        warhead: Some(plain),
        source: Some(firer),
        house: Some(HouseId(ALPHA)),
        resources: true,
    };
    let first = det(30);
    area::detonate(&mut w, first);
    // 30 / 2 = 15: the defences applied, so the hit does not ignore them.
    assert_eq!(health(&w, target), 25);
    let second = det(60);
    area::detonate(&mut w, second);
    assert!(w.obj(target).is_none() || health(&w, target) == 0);
    assert_eq!(w.houses[usize::from(ALPHA)].combat.kills, 1);
    assert_eq!(w.houses[usize::from(BETA)].combat.kills, 0);
}

// spec: S-0302 claim 1 (a projectile carries its firer as attacker and the firer's owner as attacking house until the firer is killed; then it carries neither)
#[test]
fn a_projectile_loses_its_attacker_when_the_firer_is_killed() {
    let mut r = testkit::combat_ruleset();
    warhead(&mut r, "PLAIN", |h| h.verses = every("1"));
    vehicle(&mut r, "GUNNER", |t| {
        t.primary = Some("Gun".into());
        t.strength = Some(50);
    });
    vehicle(&mut r, "BOX", |_| {});
    let mut w = testkit::game_with(&r, SIZE, 1);
    let firer = put(&mut w, Kind::Vehicle, "GUNNER", ALPHA, -1);
    let target = put(&mut w, Kind::Vehicle, "BOX", BETA, 2);
    // The gunner, guarding, picks the box and fires at it.
    for _ in 0..600 {
        if !w.projectiles.is_empty() {
            break;
        }
        glasnost_sim::tick::step(&mut w);
    }
    assert!(w.obj(target).is_some());
    let carried = |w: &World| {
        let p = w
            .projectiles
            .values()
            .next()
            .expect("a projectile in flight");
        (p.attacker, p.house)
    };
    assert_eq!(carried(&w), (Some(firer), Some(HouseId(ALPHA))));
    let plain = wh(&w, "PLAIN");
    assert_eq!(apply!(w, firer, hit(500, plain)), HitResult::NowDead);
    assert_eq!(carried(&w), (None, None));
}

// ---------------------------------------------------------------------------
// Claim 2 and F1: shot damage.
// ---------------------------------------------------------------------------

fn firepower_world(dmg: i32, f: impl FnOnce(&mut WeaponType)) -> (World, ObjectId, WeaponId) {
    let mut r = testkit::combat_ruleset();
    weapon(&mut r, "Zap", |t| {
        t.damage = Some(dmg);
        t.warhead = Some("SA".into());
        f(t);
    });
    vehicle(&mut r, "SHOOTER", |t| {
        t.primary = Some("Zap".into());
        t.veteran_abilities = vec!["firepower".into()];
    });
    country(&mut r, "Alphaland", |c| c.firepower = Some(dec("1.25")));
    r.sim.general.veteran_combat = Some(dec("1.5"));
    let mut w = testkit::game_with(&r, SIZE, 1);
    let s = put(&mut w, Kind::Vehicle, "SHOOTER", ALPHA, 0);
    let z = wp(&w, "Zap");
    (w, s, z)
}

// spec: S-0302 claim 2 and F1 (E₀ = trunc(RZ₅₃(RZ₅₃(F_h × F_o) × D)), then E₁ = trunc(RZ₅₃(E₀ × V_c)) for a veteran firepower bonus, each truncated on its own)
#[test]
fn shot_damage_truncates_each_factor() {
    let (mut w, s, z) = firepower_world(7, |_| {});
    // D2: trunc(7 × 1.25) = 8.
    assert_eq!(damage::shot_damage(&w, s, z), 8);
    // The object multiplier joins the house multiplier before D.
    w.obj_mut(s).unwrap().combat.firepower_mult = nearest_double("1.5");
    let fh = nearest_double("1.25");
    let e0 = fh
        .mul(nearest_double("1.5"), RZ)
        .mul(d(7), RZ)
        .to_i32_common();
    assert_eq!(e0, 13);
    assert_eq!(damage::shot_damage(&w, s, z), e0);
    // The veteran factor scales the truncated E₀: trunc(8 × 1.5) = 12, not
    // trunc(7 × 1.25 × 1.5) = 13.
    w.obj_mut(s).unwrap().combat.firepower_mult = d(1);
    make_veteran(&mut w, s);
    assert_eq!(damage::shot_damage(&w, s, z), 12);
}

// spec: S-0302 claim 2 and F1 (E₀ = D for D ≤ 0: the firepower multipliers and the veteran factor leave a non-positive damage unchanged)
#[test]
fn shot_damage_leaves_non_positive_damage_unchanged() {
    for dmg in [-7, 0] {
        let (mut w, s, z) = firepower_world(dmg, |_| {});
        make_veteran(&mut w, s);
        w.obj_mut(s).unwrap().combat.firepower_mult = d(3);
        assert_eq!(damage::shot_damage(&w, s, z), dmg);
    }
}

// spec: S-0302 claim 2 and F1 (E₀ = 0 for a sonic-wave or fire-particle weapon)
#[test]
fn sonic_and_fire_particle_weapons_carry_zero() {
    let (w, s, z) = firepower_world(40, |t| t.is_sonic = Some(true));
    assert_eq!(damage::shot_damage(&w, s, z), 0);
    let (w, s, z) = firepower_world(40, |t| t.use_fire_particles = Some(true));
    assert_eq!(damage::shot_damage(&w, s, z), 0);
}

// ---------------------------------------------------------------------------
// Claims 3 and 4: house and object multipliers.
// ---------------------------------------------------------------------------

// spec: S-0302 claim 3 (F_h = RZ₅₃(φᵣ × φ_c) with the normal row for every house; the easy and difficult rows are not used)
#[test]
fn house_firepower_is_the_normal_row_times_the_country() {
    let mut r = testkit::combat_ruleset();
    let row = |fp: &str| glasnost_rules::sim::DifficultyRow {
        firepower: Some(dec(fp)),
        ..Default::default()
    };
    r.sim.difficulty.insert("normal".into(), row("1.1"));
    r.sim.difficulty.insert("easy".into(), row("3"));
    r.sim.difficulty.insert("difficult".into(), row("5"));
    country(&mut r, "Alphaland", |c| c.firepower = Some(dec("1.3")));
    let w = testkit::game_with(&r, SIZE, 1);
    let mode = w.rules().mode;
    let read = |t: &str| {
        glasnost_num::decimal::parse(t, glasnost_num::decimal::Field::Decimal, mode).unwrap()
    };
    let expect = read("1.1").mul(read("1.3"), mode);
    assert!(w.houses[usize::from(ALPHA)].combat.firepower.eq_num(expect));
    assert!(
        w.houses[usize::from(BETA)]
            .combat
            .firepower
            .eq_num(read("1.1"))
    );
}

// spec: S-0302 claim 4 (each object's firepower and armour multipliers start at 1.0)
#[test]
fn object_multipliers_start_at_one() {
    let r = testkit::combat_ruleset();
    let mut w = testkit::game_with(&r, SIZE, 1);
    let id = put(&mut w, Kind::Vehicle, "CART", ALPHA, 0);
    let c = &w.obj(id).unwrap().combat;
    assert!(c.firepower_mult.eq_num(d(1)));
    assert!(c.armour_mult.eq_num(d(1)));
}

// ---------------------------------------------------------------------------
// Claim 5 and F2: defences.
// ---------------------------------------------------------------------------

// spec: S-0302 claim 5 and F2 (scenario D2: shot damage 8 against a vehicle armour factor of 0.75 loses trunc(8 / 0.75) = 10)
#[test]
fn defences_divide_by_the_house_armour_factor() {
    let mut r = testkit::combat_ruleset();
    vehicle(&mut r, "BOX", |_| {});
    warhead(&mut r, "PLAIN", |h| h.verses = every("1"));
    country(&mut r, "Betaland", |c| {
        c.armor_units_mult = Some(dec("0.75"))
    });
    let mut w = testkit::game_with(&r, SIZE, 1);
    let t = put(&mut w, Kind::Vehicle, "BOX", BETA, 0);
    let (l, _) = loss!(w, t, hit(8, wh(&w, "PLAIN")));
    assert_eq!(l, 10);
}

// spec: S-0302 claim 5 and F2 (scenario D2 map 2: the defended amount is at least 1, before the versus value: 1 / 4.0 gives 1 at 100 % and 0 at 50 %)
#[test]
fn defences_never_go_below_one_before_the_versus_value() {
    let mut r = testkit::combat_ruleset();
    vehicle(&mut r, "BOX", |_| {});
    warhead(&mut r, "FULL", |h| h.verses = every("1"));
    warhead(&mut r, "HALF", |h| h.verses = every("0.5"));
    country(&mut r, "Betaland", |c| c.armor_units_mult = Some(dec("4")));
    let mut w = testkit::game_with(&r, SIZE, 1);
    let t = put(&mut w, Kind::Vehicle, "BOX", BETA, 0);
    assert_eq!(loss!(w, t, hit(1, wh(&w, "FULL"))).0, 1);
    assert_eq!(loss!(w, t, hit(1, wh(&w, "HALF"))).0, 0);
}

// spec: S-0302 claim 5 and F2 (the house armour factor is the owner country's factor for the target's kind: infantry, vehicles, other buildings, defence buildings)
#[test]
fn defences_use_the_factor_for_the_targets_kind() {
    let mut r = testkit::combat_ruleset();
    vehicle(&mut r, "BOX", |_| {});
    infantry(&mut r, "MAN", |_| {});
    building(&mut r, "SHED", |_| {});
    building(&mut r, "FORT", |b| b.defense = Some(true));
    warhead(&mut r, "PLAIN", |h| h.verses = every("1"));
    country(&mut r, "Betaland", |c| {
        c.armor_infantry_mult = Some(dec("2"));
        c.armor_units_mult = Some(dec("4"));
        c.armor_buildings_mult = Some(dec("5"));
        c.armor_defenses_mult = Some(dec("8"));
    });
    let mut w = testkit::game_with(&r, SIZE, 1);
    let h = wh(&w, "PLAIN");
    let man = put(&mut w, Kind::Infantry, "MAN", BETA, -4);
    let bx = put(&mut w, Kind::Vehicle, "BOX", BETA, -2);
    let shed = put(&mut w, Kind::Building, "SHED", BETA, 2);
    let fort = put(&mut w, Kind::Building, "FORT", BETA, 4);
    assert_eq!(loss!(w, man, hit(80, h)).0, 40);
    assert_eq!(loss!(w, bx, hit(80, h)).0, 20);
    assert_eq!(loss!(w, shed, hit(80, h)).0, 16);
    assert_eq!(loss!(w, fort, hit(80, h)).0, 10);
}

// spec: S-0302 claim 5 and F2 (the object armour multiplier joins the house factor in one divisor: d₂ = trunc(RZ₅₃(d₁ / RZ₅₃(A_h × A_o))))
#[test]
fn defences_use_the_object_armour_multiplier() {
    let mut r = testkit::combat_ruleset();
    vehicle(&mut r, "BOX", |_| {});
    warhead(&mut r, "PLAIN", |h| h.verses = every("1"));
    country(&mut r, "Betaland", |c| {
        c.armor_units_mult = Some(dec("0.75"))
    });
    let mut w = testkit::game_with(&r, SIZE, 1);
    let t = put(&mut w, Kind::Vehicle, "BOX", BETA, 0);
    w.obj_mut(t).unwrap().combat.armour_mult = d(2);
    // 100 / (0.75 × 2) = 66.67 → 66.
    assert_eq!(loss!(w, t, hit(100, wh(&w, "PLAIN"))).0, 66);
}

// spec: S-0302 claim 5 and F2 (scenario D7: a prone infantry target takes trunc(d₀ × P), at least 1, before the armour: 9 → 4, 1 → 1)
#[test]
fn prone_infantry_take_the_prone_factor() {
    let mut r = testkit::combat_ruleset();
    infantry(&mut r, "MAN", |_| {});
    warhead(&mut r, "PRONE", |h| {
        h.verses = every("1");
        h.prone_damage = Some(dec("0.5"));
    });
    let mut w = testkit::game_with(&r, SIZE, 1);
    let h = wh(&w, "PRONE");
    let standing = put(&mut w, Kind::Infantry, "MAN", BETA, -2);
    let lying = put(&mut w, Kind::Infantry, "MAN", BETA, 2);
    w.obj_mut(lying).unwrap().infantry.as_mut().unwrap().prone = true;
    assert_eq!(loss!(w, standing, hit(9, h)).0, 9);
    assert_eq!(loss!(w, lying, hit(9, h)).0, 4);
    assert_eq!(loss!(w, standing, hit(1, h)).0, 1);
    assert_eq!(loss!(w, lying, hit(1, h)).0, 1);
}

// spec: S-0302 claim 5 and F2 (the veteran armour factor is a second divisor with its own truncation: d₂′ = trunc(RZ₅₃(d₂ / V_a)))
#[test]
fn veteran_armour_is_a_second_truncated_divisor() {
    let mut r = testkit::combat_ruleset();
    vehicle(&mut r, "BOX", |t| {
        t.veteran_abilities = vec!["stronger".into()]
    });
    warhead(&mut r, "PLAIN", |h| h.verses = every("1"));
    country(&mut r, "Betaland", |c| {
        c.armor_units_mult = Some(dec("0.75"))
    });
    r.sim.general.veteran_armor = Some(dec("1.5"));
    let mut w = testkit::game_with(&r, SIZE, 1);
    let t = put(&mut w, Kind::Vehicle, "BOX", BETA, 0);
    make_veteran(&mut w, t);
    // trunc(8 / 0.75) = 10, then trunc(10 / 1.5) = 6 (one division would give 7).
    assert_eq!(loss!(w, t, hit(8, wh(&w, "PLAIN"))).0, 6);
}

// spec: S-0302 claim 5 (a negative amount is not changed by the defences: scenario D25 part c)
#[test]
fn healing_ignores_the_armour_factor() {
    let mut r = testkit::combat_ruleset();
    vehicle(&mut r, "BOX", |_| {});
    warhead(&mut r, "PLAIN", |h| h.verses = every("1"));
    country(&mut r, "Betaland", |c| c.armor_units_mult = Some(dec("2")));
    let mut w = testkit::game_with(&r, SIZE, 1);
    let t = put(&mut w, Kind::Vehicle, "BOX", BETA, 0);
    set_health(&mut w, t, 500);
    assert_eq!(loss!(w, t, hit(-20, wh(&w, "PLAIN"))).0, -20);
}

// ---------------------------------------------------------------------------
// Claim 6 and F3: versus value, falloff and cap.
// ---------------------------------------------------------------------------

// spec: S-0302 claim 6 and F3 (scenario D1: damage 37 against eleven versus values loses trunc(37 × v) each: 37, 18, 12, 9, 3, 0, 55, 18, 0, 36, 27)
#[test]
fn versus_values_scale_and_truncate() {
    let vs = [
        "1", "0.5", "0.33", "0.25", "0.1", "0.01", "1.5", "0.5", "0.02", "0.99", "0.75",
    ];
    let expect = [37, 18, 12, 9, 3, 0, 55, 18, 0, 36, 27];
    let mut r = testkit::combat_ruleset();
    for (i, c) in ArmourClass::ALL.iter().enumerate() {
        vehicle(&mut r, &format!("V{i}"), |t| t.combat.armor = Some(*c));
    }
    warhead(&mut r, "D1", |h| h.verses = verses(&vs));
    let mut w = testkit::game_with(&r, SIZE, 1);
    let h = wh(&w, "D1");
    for (i, e) in expect.iter().enumerate() {
        let t = put(&mut w, Kind::Vehicle, &format!("V{i}"), BETA, i as i32 - 5);
        assert_eq!(loss!(w, t, hit(37, h)).0, *e, "class {i}");
    }
}

// spec: S-0302 claim 6 and F3 (scenario D3: the game-wide maximum damage caps the result: 25 → 10, 3 → 3)
#[test]
fn maximum_damage_caps_the_result() {
    let mut r = testkit::combat_ruleset();
    vehicle(&mut r, "BOX", |_| {});
    warhead(&mut r, "PLAIN", |h| h.verses = every("1"));
    r.sim.general.max_damage = Some(10);
    let mut w = testkit::game_with(&r, SIZE, 1);
    let t = put(&mut w, Kind::Vehicle, "BOX", BETA, 0);
    let h = wh(&w, "PLAIN");
    assert_eq!(loss!(w, t, hit(25, h)).0, 10);
    assert_eq!(loss!(w, t, hit(3, h)).0, 3);
}

// spec: S-0302 Edge cases (maximum damage of 0 or below: scenario D3 part c, health 500 stays 500 with 0, rises to 505 with −5, also at a versus value of 0)
#[test]
fn maximum_damage_of_zero_or_below() {
    for (max, vs, expect) in [(0, "1", 500), (-5, "1", 505), (-5, "0", 505)] {
        let mut r = testkit::combat_ruleset();
        vehicle(&mut r, "BOX", |_| {});
        warhead(&mut r, "W", |h| h.verses = every(vs));
        r.sim.general.max_damage = Some(max);
        let mut w = testkit::game_with(&r, SIZE, 1);
        let t = put(&mut w, Kind::Vehicle, "BOX", BETA, 0);
        set_health(&mut w, t, 500);
        apply!(w, t, hit(25, wh(&w, "W")));
        assert_eq!(health(&w, t), expect, "max {max}, versus {vs}");
    }
}

// spec: S-0302 claim 6 and F3 cases (iii) and (iv) (a negative amount heals by itself, ignoring the versus value, below 8 leptons only)
#[test]
fn healing_reaches_below_eight_leptons_only() {
    let mut r = testkit::combat_ruleset();
    vehicle(&mut r, "BOX", |_| {});
    warhead(&mut r, "HALF", |h| h.verses = every("0.5"));
    let mut w = testkit::game_with(&r, SIZE, 1);
    let t = put(&mut w, Kind::Vehicle, "BOX", BETA, 0);
    set_health(&mut w, t, 100);
    let h = wh(&w, "HALF");
    let near = Hit {
        distance: 7,
        ..hit(-20, h)
    };
    let far = Hit {
        distance: 8,
        ..hit(-20, h)
    };
    assert_eq!(loss!(w, t, near), (-20, HitResult::Unaffected));
    assert_eq!(loss!(w, t, far), (0, HitResult::Unaffected));
}

// spec: S-0302 claim 6 and F3 case (ii) (a hit without a warhead that does not ignore defences has m₀ = 0)
#[test]
fn a_hit_without_a_warhead_does_nothing() {
    let mut r = testkit::combat_ruleset();
    vehicle(&mut r, "BOX", |_| {});
    let mut w = testkit::game_with(&r, SIZE, 1);
    let t = put(&mut w, Kind::Vehicle, "BOX", BETA, 0);
    let h = Hit {
        warhead: None,
        ..hit(100, 0)
    };
    assert_eq!(loss!(w, t, h), (0, HitResult::Unaffected));
}

// spec: S-0302 claim 6 (a negative versus value gives a negative result that heals, and the maximum does not limit it: scenario D25)
#[test]
fn a_negative_versus_value_heals() {
    let mut r = testkit::combat_ruleset();
    vehicle(&mut r, "BOX", |_| {});
    warhead(&mut r, "NEG", |h| h.verses = every("-0.5"));
    r.sim.general.max_damage = Some(10);
    let mut w = testkit::game_with(&r, SIZE, 1);
    let t = put(&mut w, Kind::Vehicle, "BOX", BETA, 0);
    set_health(&mut w, t, 500);
    assert_eq!(
        loss!(w, t, hit(100, wh(&w, "NEG"))),
        (-50, HitResult::Unaffected)
    );
}

// spec: S-0302 F3 case (vi) (the falloff g, floored at 0, then times v, truncated, capped; S-0303 F3 shares it)
#[test]
fn falloff_then_versus_at_a_distance() {
    let mut r = testkit::combat_ruleset();
    vehicle(&mut r, "BOX", |_| {});
    warhead(&mut r, "WIDE", |h| {
        h.verses = every("0.5");
        h.cell_spread = Some(dec("2"));
        h.percent_at_max = Some(dec("0.5"));
    });
    let mut w = testkit::game_with(&r, SIZE, 1);
    let t = put(&mut w, Kind::Vehicle, "BOX", BETA, 0);
    let h = Hit {
        distance: 256,
        ..hit(100, wh(&w, "WIDE"))
    };
    // R = 512, g = trunc((100 − 50) × 256 / 512 + 50) = 75, then trunc(75 × 0.5) = 37.
    assert_eq!(loss!(w, t, h).0, 37);
}

// ---------------------------------------------------------------------------
// Claim 7: undemolishable buildings.
// ---------------------------------------------------------------------------

fn undemolishable_world(quirk: bool) -> (World, ObjectId, ObjectId, WarheadId, WarheadId) {
    let mut r = testkit::combat_ruleset();
    building(&mut r, "SOFT", |_| {});
    building(&mut r, "HARD", |b| b.can_c4 = Some(false));
    warhead(&mut r, "TINY", |h| h.verses = every("0.01"));
    warhead(&mut r, "FULL", |h| h.verses = every("1"));
    r.globals.quirks.undemolishable_heal_is_damage = quirk;
    let mut w = testkit::game_with(&r, SIZE, 1);
    let soft = put(&mut w, Kind::Building, "SOFT", BETA, -3);
    let hard = put(&mut w, Kind::Building, "HARD", BETA, 3);
    let (t, f) = (wh(&w, "TINY"), wh(&w, "FULL"));
    (w, soft, hard, t, f)
}

// spec: S-0302 claim 7 (scenario D5: damage 50 at 1 % costs a demolishable building 0 and an undemolishable one 1)
#[test]
fn undemolishable_buildings_lose_at_least_one() {
    let (mut w, soft, hard, tiny, _) = undemolishable_world(true);
    assert_eq!(loss!(w, soft, hit(50, tiny)).0, 0);
    assert_eq!(loss!(w, hard, hit(50, tiny)).0, 1);
    // Also a hit that ignores defences with an amount of 0.
    let ign = Hit {
        ignore_defences: true,
        ..hit(0, tiny)
    };
    assert_eq!(loss!(w, hard, ign).0, 1);
    assert_eq!(loss!(w, soft, ign).0, 0);
}

// spec: S-0302 Edge cases (quirks.undemolishable_heal_is_damage on: a healing hit costs an undemolishable building 1, scenario D5 run 2)
#[test]
fn undemolishable_heal_is_damage_on() {
    let (mut w, _, hard, _, full) = undemolishable_world(true);
    set_health(&mut w, hard, 500);
    assert_eq!(loss!(w, hard, hit(-20, full)).0, 1);
}

// spec: S-0302 Edge cases (quirks.undemolishable_heal_is_damage off: a result of 0 still becomes 1, a negative result heals)
#[test]
fn undemolishable_heal_is_damage_off() {
    let (mut w, _, hard, tiny, full) = undemolishable_world(false);
    set_health(&mut w, hard, 500);
    assert_eq!(loss!(w, hard, hit(-20, full)).0, -20);
    assert_eq!(loss!(w, hard, hit(50, tiny)).0, 1);
}

// ---------------------------------------------------------------------------
// Claim 8: hits that do nothing.
// ---------------------------------------------------------------------------

fn ign(h: Hit) -> Hit {
    Hit {
        ignore_defences: true,
        ..h
    }
}

// spec: S-0302 claim 8 (stage 1: a building hit by itself is stopped unless its type allows self-damage, in every column)
#[test]
fn a_building_does_not_damage_itself() {
    let mut r = testkit::combat_ruleset();
    building(&mut r, "SHED", |_| {});
    building(&mut r, "SELFY", |b| b.combat.damage_self = Some(true));
    warhead(&mut r, "PLAIN", |h| h.verses = every("1"));
    let mut w = testkit::game_with(&r, SIZE, 1);
    let h = wh(&w, "PLAIN");
    let shed = put(&mut w, Kind::Building, "SHED", BETA, -3);
    let selfy = put(&mut w, Kind::Building, "SELFY", BETA, 3);
    let by = |id| Hit {
        attacker: Some(id),
        ..hit(100, h)
    };
    assert_eq!(loss!(w, shed, by(shed)), (0, HitResult::Unaffected));
    assert_eq!(loss!(w, shed, ign(by(shed))).0, 0);
    assert_eq!(loss!(w, selfy, by(selfy)).0, 100);
}

// spec: S-0302 claim 8 (stage 1: a laser-fence section stops + and − hits, not hits that ignore defences)
#[test]
fn laser_fence_sections_stop_defended_hits() {
    let mut r = testkit::combat_ruleset();
    building(&mut r, "FENCE", |b| b.laser_fence = Some(true));
    warhead(&mut r, "PLAIN", |h| h.verses = every("1"));
    let mut w = testkit::game_with(&r, SIZE, 1);
    let h = wh(&w, "PLAIN");
    let f = put(&mut w, Kind::Building, "FENCE", BETA, 0);
    set_health(&mut w, f, 500);
    assert_eq!(loss!(w, f, hit(100, h)).0, 0);
    assert_eq!(loss!(w, f, hit(-20, h)).0, 0);
    assert_eq!(loss!(w, f, ign(hit(100, h))).0, 100);
}

// spec: S-0302 claim 8 (stage 1: an immune bridge repair hut stops every hit, also one that ignores defences)
#[test]
fn an_immune_bridge_repair_hut_stops_every_hit() {
    let mut r = testkit::combat_ruleset();
    building(&mut r, "HUT", |b| {
        b.bridge_repair_hut = Some(true);
        b.immune = Some(true);
    });
    warhead(&mut r, "PLAIN", |h| h.verses = every("1"));
    let mut w = testkit::game_with(&r, SIZE, 1);
    let h = wh(&w, "PLAIN");
    let hut = put(&mut w, Kind::Building, "HUT", BETA, 0);
    assert_eq!(loss!(w, hut, hit(100, h)).0, 0);
    assert_eq!(loss!(w, hut, ign(hit(100, h))).0, 0);
}

// spec: S-0302 claim 8 (stage 1: a vehicle standing inside the vehicle factory it is in contact with stops + and − hits, not hits that ignore defences)
#[test]
fn a_vehicle_inside_its_factory_is_not_damaged() {
    let mut r = testkit::combat_ruleset();
    building(&mut r, "WORKS", |b| {
        b.weapons_factory = Some(true);
        b.foundation = Some([2, 2]);
    });
    vehicle(&mut r, "BOX", |_| {});
    warhead(&mut r, "PLAIN", |h| h.verses = every("1"));
    let mut w = testkit::game_with(&r, SIZE, 1);
    let h = wh(&w, "PLAIN");
    let works = put(&mut w, Kind::Building, "WORKS", BETA, 0);
    let v = put(&mut w, Kind::Vehicle, "BOX", BETA, 6);
    // Inside the factory's foundation, in contact with it.
    let p = Pos::centre(at(1, 1), 0);
    {
        let o = w.obj_mut(v).unwrap();
        o.pos = p;
        o.contact = Some(works);
    }
    assert_eq!(loss!(w, v, hit(100, h)).0, 0);
    assert_eq!(loss!(w, v, ign(hit(100, h))).0, 100);
    // Without the contact the hit lands.
    w.obj_mut(v).unwrap().contact = None;
    assert_eq!(loss!(w, v, hit(100, h)).0, 100);
}

// spec: S-0302 claim 8 (stage 1: a type immune to its own type stops + hits by an attacker of the same type and owner only)
#[test]
fn type_immunity_stops_hits_of_the_same_type_and_owner() {
    let mut r = testkit::combat_ruleset();
    vehicle(&mut r, "TWIN", |t| t.combat.type_immune = Some(true));
    warhead(&mut r, "PLAIN", |h| h.verses = every("1"));
    let mut w = testkit::game_with(&r, SIZE, 1);
    let h = wh(&w, "PLAIN");
    let a = put(&mut w, Kind::Vehicle, "TWIN", BETA, -6);
    let b = put(&mut w, Kind::Vehicle, "TWIN", BETA, 0);
    let foe = put(&mut w, Kind::Vehicle, "TWIN", ALPHA, 6);
    let by = |x, amount| Hit {
        attacker: Some(x),
        ..hit(amount, h)
    };
    assert_eq!(loss!(w, b, by(a, 100)).0, 0);
    assert_eq!(loss!(w, b, ign(by(a, 100))).0, 100);
    assert_eq!(loss!(w, b, by(a, -20)).0, -20);
    assert_eq!(loss!(w, b, by(foe, 100)).0, 100);
}

// spec: S-0302 claim 8 (stage 1: a building that already has no health stops every hit)
#[test]
fn a_building_without_health_takes_nothing() {
    let mut r = testkit::combat_ruleset();
    building(&mut r, "SHED", |_| {});
    warhead(&mut r, "PLAIN", |h| h.verses = every("1"));
    let mut w = testkit::game_with(&r, SIZE, 1);
    let h = wh(&w, "PLAIN");
    let s = put(&mut w, Kind::Building, "SHED", BETA, 0);
    set_health(&mut w, s, 0);
    assert_eq!(apply!(w, s, ign(hit(100, h))), HitResult::Unaffected);
    assert_eq!(apply!(w, s, hit(-50, h)), HitResult::Unaffected);
    assert_eq!(health(&w, s), 0);
}

// spec: S-0302 claim 8 (stage 2: radiation, psychic and poison warheads on types immune to them stop every hit, also one that ignores defences)
#[test]
fn warhead_kind_immunities_stop_every_hit() {
    let mut r = testkit::combat_ruleset();
    vehicle(&mut r, "LEAD", |t| t.immune_to_radiation = Some(true));
    vehicle(&mut r, "CALM", |t| {
        t.combat.immune_to_psionic_weapons = Some(true)
    });
    vehicle(&mut r, "CLEAN", |t| t.combat.immune_to_poison = Some(true));
    warhead(&mut r, "RAD", |h| {
        h.verses = every("1");
        h.radiation = Some(true);
    });
    warhead(&mut r, "PSI", |h| {
        h.verses = every("1");
        h.psychic_damage = Some(true);
    });
    warhead(&mut r, "TOX", |h| {
        h.verses = every("1");
        h.poison = Some(true);
    });
    let mut w = testkit::game_with(&r, SIZE, 1);
    for (ty, warhead_name, other) in [
        ("LEAD", "RAD", "PSI"),
        ("CALM", "PSI", "TOX"),
        ("CLEAN", "TOX", "RAD"),
    ] {
        let t = put(&mut w, Kind::Vehicle, ty, BETA, 0);
        let h = wh(&w, warhead_name);
        assert_eq!(loss!(w, t, hit(100, h)).0, 0, "{ty}");
        assert_eq!(loss!(w, t, ign(hit(100, h))).0, 0, "{ty}");
        assert_eq!(loss!(w, t, hit(100, wh(&w, other))).0, 100, "{ty}");
        w.obj_mut(t).unwrap().health = 0;
        glasnost_sim::objects::remove(&mut w, t);
    }
}

// spec: S-0302 claim 8 (stage 2: a warhead that does not affect allies stops a hit whose attacker's owner is allied with the target's owner; a hit with no attacker is never stopped by it)
#[test]
fn warheads_that_spare_allies() {
    let mut r = testkit::combat_ruleset();
    vehicle(&mut r, "BOX", |_| {});
    warhead(&mut r, "KIND", |h| {
        h.verses = every("1");
        h.affects_allies = Some(false);
    });
    let mut w = testkit::game_with(&r, SIZE, 1);
    let h = wh(&w, "KIND");
    let own = put(&mut w, Kind::Vehicle, "BOX", BETA, -6);
    let foe = put(&mut w, Kind::Vehicle, "BOX", ALPHA, 6);
    let t = put(&mut w, Kind::Vehicle, "BOX", BETA, 0);
    let by = |x| Hit {
        attacker: Some(x),
        ..hit(100, h)
    };
    // A house is allied with itself.
    assert_eq!(loss!(w, t, by(own)).0, 0);
    assert_eq!(loss!(w, t, ign(by(own))).0, 0);
    assert_eq!(loss!(w, t, hit(100, h)).0, 100);
    assert_eq!(loss!(w, t, by(foe)).0, 100);
    // The target's house counting the attacker's as allied is not enough:
    // the attacker's owner must count the target's owner as allied.
    w.houses[usize::from(BETA)].allies = vec![HouseId(ALPHA), HouseId(BETA)];
    assert_eq!(loss!(w, t, by(foe)).0, 100);
    w.houses[usize::from(ALPHA)].allies = vec![HouseId(ALPHA), HouseId(BETA)];
    assert_eq!(loss!(w, t, by(foe)).0, 0);
}

// spec: S-0302 claim 8 (stage 4: a type immune to all damage stops + and − hits but not hits that ignore defences)
#[test]
fn immune_types_stop_defended_hits_only() {
    let mut r = testkit::combat_ruleset();
    vehicle(&mut r, "ROCK", |t| t.immune = Some(true));
    warhead(&mut r, "PLAIN", |h| h.verses = every("1"));
    let mut w = testkit::game_with(&r, SIZE, 1);
    let h = wh(&w, "PLAIN");
    let t = put(&mut w, Kind::Vehicle, "ROCK", BETA, 0);
    set_health(&mut w, t, 500);
    assert_eq!(loss!(w, t, hit(100, h)), (0, HitResult::Unaffected));
    assert_eq!(loss!(w, t, hit(-20, h)).0, 0);
    assert_eq!(loss!(w, t, ign(hit(100, h))).0, 100);
}

// spec: S-0302 claim 8 (stage 4: an object that is not a building and already has no health takes nothing)
#[test]
fn a_unit_without_health_takes_nothing() {
    let mut r = testkit::combat_ruleset();
    vehicle(&mut r, "BOX", |_| {});
    warhead(&mut r, "PLAIN", |h| h.verses = every("1"));
    let mut w = testkit::game_with(&r, SIZE, 1);
    let h = wh(&w, "PLAIN");
    let t = put(&mut w, Kind::Vehicle, "BOX", BETA, 0);
    set_health(&mut w, t, 0);
    assert_eq!(apply!(w, t, hit(-50, h)), HitResult::Unaffected);
    assert_eq!(apply!(w, t, ign(hit(50, h))), HitResult::Unaffected);
    assert_eq!(health(&w, t), 0);
}

// spec: S-0302 claim 8 (stage 4: a hit whose amount is 0 after claims 5 to 7 does nothing and reports unaffected; a delivered 0 that ignores defences does nothing)
#[test]
fn a_zero_amount_does_nothing() {
    let mut r = testkit::combat_ruleset();
    vehicle(&mut r, "BOX", |_| {});
    warhead(&mut r, "HALF", |h| h.verses = every("0.5"));
    warhead(&mut r, "FULL", |h| h.verses = every("1"));
    let mut w = testkit::game_with(&r, SIZE, 1);
    let t = put(&mut w, Kind::Vehicle, "BOX", BETA, 0);
    assert_eq!(
        loss!(w, t, hit(1, wh(&w, "HALF"))),
        (0, HitResult::Unaffected)
    );
    assert_eq!(
        loss!(w, t, ign(hit(0, wh(&w, "FULL")))),
        (0, HitResult::Unaffected)
    );
    // A delivered 0 that does not ignore defences becomes 1, then trunc(v).
    assert_eq!(loss!(w, t, hit(0, wh(&w, "FULL"))).0, 1);
}

// ---------------------------------------------------------------------------
// Claim 9: what ignoring defences skips.
// ---------------------------------------------------------------------------

// spec: S-0302 claim 9 and F3 case (i) (a hit that ignores defences skips the armour, the versus value, the cap and the healing reach)
#[test]
fn ignoring_defences_skips_armour_versus_cap_and_reach() {
    let mut r = testkit::combat_ruleset();
    vehicle(&mut r, "BOX", |_| {});
    warhead(&mut r, "HALF", |h| h.verses = every("0.5"));
    country(&mut r, "Betaland", |c| c.armor_units_mult = Some(dec("4")));
    r.sim.general.max_damage = Some(10);
    let mut w = testkit::game_with(&r, SIZE, 1);
    let h = wh(&w, "HALF");
    let t = put(&mut w, Kind::Vehicle, "BOX", BETA, 0);
    assert_eq!(loss!(w, t, ign(hit(100, h))).0, 100);
    let far = Hit {
        distance: 300,
        ..ign(hit(-30, h))
    };
    assert_eq!(loss!(w, t, far).0, -30);
}

// ---------------------------------------------------------------------------
// Claim 10 and F6: readiness loss and berserk.
// ---------------------------------------------------------------------------

fn readiness_world(f: impl FnOnce(&mut Ruleset)) -> (World, ObjectId) {
    let mut r = testkit::combat_ruleset();
    vehicle(&mut r, "READY", |t| {
        t.combat.ammo = Some(10);
        t.combat.initial_ammo = Some(10);
        t.combat.damage_reduces_readiness = Some(true);
        t.combat.readiness_reduction_multiplier = Some(dec("0.5"));
        t.immune_to_radiation = Some(true);
        t.combat.type_immune = Some(true);
    });
    warhead(&mut r, "LOW", |h| h.verses = every("0.1"));
    warhead(&mut r, "RAD", |h| {
        h.verses = every("1");
        h.radiation = Some(true);
    });
    f(&mut r);
    let mut w = testkit::game_with(&r, SIZE, 1);
    let t = put(&mut w, Kind::Vehicle, "READY", BETA, 0);
    w.obj_mut(t).unwrap().combat.ammo = 10;
    (w, t)
}

fn ammo(w: &World, id: ObjectId) -> i32 {
    w.obj(id).unwrap().combat.ammo
}

/// F6: max(trunc(RZ₅₃(A − RZ₅₃(A_max × RZ₅₃(κ × RZ₅₃(d / S))))), 0).
fn f6(a: i32, amax: i32, kappa: &str, dd: i32, s: i32) -> i32 {
    let q = d(i64::from(dd)).div(d(i64::from(s)), RZ);
    let k = nearest_double(kappa).mul(q, RZ);
    d(i64::from(a))
        .sub(d(i64::from(amax)).mul(k, RZ), RZ)
        .to_i32_common()
        .max(0)
}

// spec: S-0302 claim 10 and F6 (quirks.readiness_from_defended_damage on: ammunition lost in proportion to the amount after the defences, before the versus value)
#[test]
fn readiness_loss_uses_the_defended_amount() {
    let (mut w, t) = readiness_world(|_| {});
    let h = wh(&w, "LOW");
    apply!(w, t, hit(300, h));
    assert_eq!(ammo(&w, t), f6(10, 10, "0.5", 300, 1000));
    assert_eq!(ammo(&w, t), 8);
    assert!(w.obj(t).unwrap().combat.reload_left.is_some());
}

// spec: S-0302 claim 10 (readiness loss happens for a hit that a stage 2 or 4 row then stops, and not for one a stage 1 row stops)
#[test]
fn readiness_loss_follows_the_stages() {
    let (mut w, t) = readiness_world(|_| {});
    // Stage 2: the radiation immunity stops the hit, the ammunition is lost.
    apply!(w, t, hit(300, wh(&w, "RAD")));
    assert_eq!(health(&w, t), 1000);
    assert_eq!(ammo(&w, t), 8);
    // Stage 4: a zero amount (1 at 10 %) still costs ammunition.
    w.obj_mut(t).unwrap().combat.ammo = 10;
    apply!(w, t, hit(1, wh(&w, "LOW")));
    assert_eq!(ammo(&w, t), f6(10, 10, "0.5", 1, 1000));
    // Stage 1: a hit by the same type and owner costs nothing.
    let twin = put(&mut w, Kind::Vehicle, "READY", BETA, 6);
    w.obj_mut(t).unwrap().combat.ammo = 10;
    let by = Hit {
        attacker: Some(twin),
        ..hit(300, wh(&w, "LOW"))
    };
    apply!(w, t, by);
    assert_eq!(ammo(&w, t), 10);
}

// spec: S-0302 Edge cases (quirks.readiness_from_defended_damage off: the final amount m, and no loss when claim 8 stops the hit)
#[test]
fn readiness_from_defended_damage_off() {
    let (mut w, t) = readiness_world(|r| r.globals.quirks.readiness_from_defended_damage = false);
    apply!(w, t, hit(300, wh(&w, "LOW")));
    assert_eq!(ammo(&w, t), f6(10, 10, "0.5", 30, 1000));
    w.obj_mut(t).unwrap().combat.ammo = 10;
    apply!(w, t, hit(300, wh(&w, "RAD")));
    assert_eq!(ammo(&w, t), 10);
}

// spec: S-0302 Edge cases (quirks.readiness_heal_over_max on: a negative hit raises ammunition above the type's field)
#[test]
fn readiness_heal_over_max_on() {
    let (mut w, t) = readiness_world(|_| {});
    set_health(&mut w, t, 500);
    apply!(w, t, hit(-400, wh(&w, "LOW")));
    // 10 − 10 × 0.5 × (−0.4): above the field of 10 (11 here, since −0.4 is
    // held just inside it under quirks.chop_rounding).
    assert_eq!(ammo(&w, t), f6(10, 10, "0.5", -400, 1000));
    assert!(ammo(&w, t) > 10);
}

// spec: S-0302 Edge cases (quirks.readiness_heal_over_max off: the ammunition after the hit is at most the type's field)
#[test]
fn readiness_heal_over_max_off() {
    let (mut w, t) = readiness_world(|r| r.globals.quirks.readiness_heal_over_max = false);
    set_health(&mut w, t, 500);
    apply!(w, t, hit(-400, wh(&w, "LOW")));
    assert_eq!(ammo(&w, t), 10);
}

// spec: S-0302 F6 and Edge cases (a strength of 0 empties the ammunition at any hit that passes stage 1)
#[test]
fn readiness_with_a_strength_of_zero_empties_the_ammunition() {
    let (mut w, t) = readiness_world(|r| {
        r.sim.vehicles.get_mut("READY").unwrap().strength = Some(0);
    });
    apply!(w, t, hit(5, wh(&w, "LOW")));
    assert_eq!(ammo(&w, t), 0);
}

fn berserk_world() -> (World, WarheadId, WarheadId) {
    let mut r = testkit::combat_ruleset();
    vehicle(&mut r, "BOX", |_| {});
    vehicle(&mut r, "STOIC", |t| {
        t.combat.immune_to_psionics = Some(true)
    });
    vehicle(&mut r, "ROCK", |t| t.immune = Some(true));
    vehicle(&mut r, "LEAD", |t| t.immune_to_radiation = Some(true));
    building(&mut r, "SHED", |_| {});
    warhead(&mut r, "MAD", |h| {
        h.verses = every("0.5");
        h.psychedelic = Some(true);
    });
    warhead(&mut r, "MADRAD", |h| {
        h.verses = every("0.5");
        h.psychedelic = Some(true);
        h.radiation = Some(true);
    });
    let w = testkit::game_with(&r, SIZE, 1);
    let (a, b) = (wh(&w, "MAD"), wh(&w, "MADRAD"));
    (w, a, b)
}

fn berserk_left(w: &World, id: ObjectId) -> Option<i32> {
    w.obj(id).unwrap().combat.berserk_until.map(|u| u - w.tick)
}

// spec: S-0302 claim 10 and Formulas (berserk duration: T is F3 at r = 0 of the defended amount; the hit changes no health and reports unchanged; a further hit sets the new T, even when shorter)
#[test]
fn berserk_hits_set_a_duration_instead_of_damage() {
    let (mut w, mad, _) = berserk_world();
    let t = put(&mut w, Kind::Vehicle, "BOX", BETA, 0);
    assert_eq!(loss!(w, t, hit(100, mad)), (0, HitResult::Unchanged));
    assert_eq!(berserk_left(&w, t), Some(50));
    apply!(w, t, hit(20, mad));
    assert_eq!(berserk_left(&w, t), Some(10));
}

// spec: S-0302 claim 10 (no berserk when the attacking house is allied with the target's owner, the type is immune to psionics, or the target is a building: unaffected)
#[test]
fn berserk_exceptions() {
    let (mut w, mad, _) = berserk_world();
    let t = put(&mut w, Kind::Vehicle, "BOX", BETA, 0);
    let stoic = put(&mut w, Kind::Vehicle, "STOIC", BETA, 3);
    let shed = put(&mut w, Kind::Building, "SHED", BETA, -3);
    let own = Hit {
        house: Some(HouseId(BETA)),
        ..hit(100, mad)
    };
    assert_eq!(apply!(w, t, own), HitResult::Unaffected);
    assert_eq!(berserk_left(&w, t), None);
    assert_eq!(apply!(w, stoic, hit(100, mad)), HitResult::Unaffected);
    assert_eq!(berserk_left(&w, stoic), None);
    assert_eq!(loss!(w, shed, hit(100, mad)), (0, HitResult::Unaffected));
    // A hit with the other house as attacking house does make it berserk.
    let foe = Hit {
        house: Some(HouseId(ALPHA)),
        ..hit(100, mad)
    };
    assert_eq!(apply!(w, t, foe), HitResult::Unchanged);
}

// spec: S-0302 claim 10 (a berserk hit is stopped by stage 2 rows but not by stage 4 rows: an immune type still goes berserk)
#[test]
fn berserk_and_the_stages() {
    let (mut w, mad, madrad) = berserk_world();
    let rock = put(&mut w, Kind::Vehicle, "ROCK", BETA, 0);
    let lead = put(&mut w, Kind::Vehicle, "LEAD", BETA, 3);
    assert_eq!(apply!(w, rock, hit(100, mad)), HitResult::Unchanged);
    assert_eq!(berserk_left(&w, rock), Some(50));
    assert_eq!(apply!(w, lead, hit(100, madrad)), HitResult::Unaffected);
    assert_eq!(berserk_left(&w, lead), None);
}

// ---------------------------------------------------------------------------
// Claim 11 and F4: health change.
// ---------------------------------------------------------------------------

// spec: S-0302 claim 11 and F4 (healing never above the strength and reports unaffected; damage above the remaining health counts as the remaining health)
#[test]
fn health_stays_within_zero_and_the_strength() {
    let mut r = testkit::combat_ruleset();
    vehicle(&mut r, "BOX", |_| {});
    warhead(&mut r, "PLAIN", |h| h.verses = every("1"));
    let mut w = testkit::game_with(&r, SIZE, 1);
    let h = wh(&w, "PLAIN");
    let t = put(&mut w, Kind::Vehicle, "BOX", BETA, 0);
    set_health(&mut w, t, 990);
    assert_eq!(loss!(w, t, hit(-20, h)), (-10, HitResult::Unaffected));
    assert_eq!(health(&w, t), 1000);
    set_health(&mut w, t, 100);
    assert_eq!(loss!(w, t, hit(300, h)), (100, HitResult::NowDead));
}

// ---------------------------------------------------------------------------
// Claim 12 and F5: thresholds.
// ---------------------------------------------------------------------------

fn threshold_world(f: impl FnOnce(&mut Ruleset)) -> (World, ObjectId, WarheadId) {
    let mut r = testkit::combat_ruleset();
    vehicle(&mut r, "BOX", |_| {});
    warhead(&mut r, "PLAIN", |h| h.verses = every("1"));
    r.sim.general.condition_red = Some(dec("0.25"));
    f(&mut r);
    let mut w = testkit::game_with(&r, SIZE, 1);
    let t = put(&mut w, Kind::Vehicle, "BOX", BETA, 0);
    let h = wh(&w, "PLAIN");
    (w, t, h)
}

fn result_from(w: &mut World, t: ObjectId, h: WarheadId, from: i32, amount: i32) -> HitResult {
    set_health(w, t, from);
    damage::apply(w, t, hit(amount, h))
}

// spec: S-0302 claim 12 and F5 (now yellow below ⌊S / 2⌋, now red below S × ρ, red before yellow, dead before both, unchanged otherwise)
#[test]
fn damage_state_results() {
    let (mut w, t, h) = threshold_world(|_| {});
    assert_eq!(result_from(&mut w, t, h, 600, 150), HitResult::NowYellow);
    assert_eq!(result_from(&mut w, t, h, 500, 1), HitResult::NowYellow);
    assert_eq!(result_from(&mut w, t, h, 499, 1), HitResult::Unchanged);
    assert_eq!(result_from(&mut w, t, h, 300, 100), HitResult::NowRed);
    assert_eq!(result_from(&mut w, t, h, 600, 400), HitResult::NowRed);
    assert_eq!(result_from(&mut w, t, h, 400, 50), HitResult::Unchanged);
    assert_eq!(result_from(&mut w, t, h, 251, 1), HitResult::Unchanged);
    assert_eq!(result_from(&mut w, t, h, 251, 2), HitResult::NowRed);
    assert_eq!(result_from(&mut w, t, h, 600, 600), HitResult::NowDead);
}

// spec: S-0302 F5 (X = RZ₅₃(S × ρ) compared exactly with the integers H and H − m′, in both settings of quirks.chop_rounding; ρ = 0.6 as the rules hold it)
#[test]
fn the_red_threshold_is_compared_exactly() {
    for chop in [true, false] {
        let (mut w, t, h) = threshold_world(|r| {
            r.sim.general.condition_red = Some(dec("0.6"));
            r.globals.quirks.chop_rounding = chop;
        });
        let mode = w.rules().mode;
        let rho = glasnost_num::decimal::parse("0.6", glasnost_num::decimal::Field::Decimal, mode)
            .unwrap();
        let x = F64::from_i64(1000, mode).mul(rho, mode);
        let f = |n: i64| F64::from_i64(n, mode);
        // F5: now red ⇔ X < H and H − m′ < X.
        let red = |h: i64, after: i64| x.lt(f(h)) && f(after).lt(x);
        for (from, amount) in [(600, 1), (601, 1), (601, 2)] {
            let got = result_from(&mut w, t, h, from, amount);
            let want = if red(i64::from(from), i64::from(from - amount)) {
                HitResult::NowRed
            } else {
                HitResult::Unchanged
            };
            assert_eq!(got, want, "chop {chop}, {from} − {amount}");
        }
        // The ratio is read as S-0003 claim 6 reads decimals (a single just
        // above 0.6), so X lies just above 600 and a hit ending at 600 is
        // red; S-0302's rounding paragraph assumes the nearest double
        // instead (request Q-0075).
        assert!(f(600).lt(x));
        assert_eq!(result_from(&mut w, t, h, 601, 1), HitResult::NowRed);
    }
}

// spec: S-0302 Edge cases (quirks.yellow_state_at_half on: now yellow at half the strength whatever the yellow ratio)
#[test]
fn yellow_state_at_half_on() {
    let (mut w, t, h) = threshold_world(|r| r.sim.general.condition_yellow = Some(dec("0.75")));
    assert_eq!(result_from(&mut w, t, h, 800, 100), HitResult::Unchanged);
    assert_eq!(result_from(&mut w, t, h, 600, 150), HitResult::NowYellow);
}

// spec: S-0302 Edge cases (quirks.yellow_state_at_half off: now yellow when health crosses strength × yellow ratio)
#[test]
fn yellow_state_at_half_off() {
    let (mut w, t, h) = threshold_world(|r| {
        r.sim.general.condition_yellow = Some(dec("0.75"));
        r.globals.quirks.yellow_state_at_half = false;
    });
    assert_eq!(result_from(&mut w, t, h, 800, 100), HitResult::NowYellow);
    assert_eq!(result_from(&mut w, t, h, 600, 150), HitResult::Unchanged);
}

// ---------------------------------------------------------------------------
// Claim 13: the damage effect and building fires.
// ---------------------------------------------------------------------------

// spec: S-0302 claim 13 (after now yellow or now red at or below the yellow ratio, one ranged draw over 0 … n − 1 picks a damage effect; a later hit that leaves the ratio above it removes the effect)
#[test]
fn damage_effect_draw_and_removal() {
    let mut r = testkit::combat_ruleset();
    vehicle(&mut r, "SMOKER", |t| {
        t.combat.damage_smoke_systems = Some(vec!["PuffA".into(), "PuffB".into(), "PuffC".into()]);
    });
    warhead(&mut r, "PLAIN", |h| h.verses = every("1"));
    r.sim.general.condition_red = Some(dec("0.25"));
    let mut w = testkit::game_with(&r, SIZE, 1);
    let h = wh(&w, "PLAIN");
    let t = put(&mut w, Kind::Vehicle, "SMOKER", BETA, 0);
    let mut expect = w.rng.clone();
    let half = w.rules().quirks.ranged_draw_half_span;
    let _ = expect.ranged(0, 2, half);
    assert_eq!(apply!(w, t, hit(600, h)), HitResult::NowYellow);
    assert!(w.obj(t).unwrap().combat.damage_effect);
    assert_eq!(w.rng.taken(), expect.taken());
    assert_eq!(w.rng.raw(), expect.raw());
    // A second threshold hit takes no draw: the object already has an effect.
    let before = w.rng.taken();
    assert_eq!(apply!(w, t, hit(200, h)), HitResult::NowRed);
    assert_eq!(w.rng.taken(), before);
    // Healing above the yellow ratio removes it.
    apply!(w, t, hit(-500, h));
    assert!(!w.obj(t).unwrap().combat.damage_effect);
}

// spec: S-0302 claim 13 (no draw when the list is empty, or when the ratio is above the yellow ratio after the hit)
#[test]
fn damage_effect_needs_a_list_and_a_low_ratio() {
    let mut r = testkit::combat_ruleset();
    vehicle(&mut r, "PLAINBOX", |_| {});
    vehicle(&mut r, "SMOKER", |t| {
        t.combat.damage_smoke_systems = Some(vec!["PuffA".into()]);
    });
    warhead(&mut r, "PLAIN", |h| h.verses = every("1"));
    r.sim.general.condition_yellow = Some(dec("0.25"));
    r.sim.general.condition_red = Some(dec("0.1"));
    let mut w = testkit::game_with(&r, SIZE, 1);
    let h = wh(&w, "PLAIN");
    let a = put(&mut w, Kind::Vehicle, "PLAINBOX", BETA, -3);
    let b = put(&mut w, Kind::Vehicle, "SMOKER", BETA, 3);
    let before = w.rng.taken();
    assert_eq!(apply!(w, a, hit(900, h)), HitResult::NowRed);
    // Ratio 0.4 > 0.25 after a now yellow: no effect.
    assert_eq!(apply!(w, b, hit(600, h)), HitResult::NowYellow);
    assert_eq!(w.rng.taken(), before);
    assert!(!w.obj(b).unwrap().combat.damage_effect);
}

// spec: S-0302 claim 13 (building fires: one draw over 0 … W + H + 5 per foundation cell, row by row; 1 to 5, 6 to 8 and 9 start the three fires, the first two each followed at once by a loop draw over 1 … 3)
#[test]
fn building_fires_and_their_draws() {
    let mut r = testkit::combat_ruleset();
    building(&mut r, "BARN", |b| b.foundation = Some([3, 2]));
    warhead(&mut r, "SPARK", |h| {
        h.verses = every("1");
        h.sparky = Some(true);
    });
    r.sim.general.condition_red = Some(dec("0.25"));
    r.sim.general.damage_fire_types = Some(vec!["FlameA".into(), "FlameB".into(), "FlameC".into()]);
    for seed in 1..6 {
        let mut w = testkit::game_with(&r, SIZE, seed);
        let h = wh(&w, "SPARK");
        let b = put(&mut w, Kind::Building, "BARN", BETA, 0);
        let top = w.obj(b).unwrap().pos.cell();
        let mut rng = w.rng.clone();
        let half = w.rules().quirks.ranged_draw_half_span;
        let mut want = Vec::new();
        for dy in 0..2 {
            for dx in 0..3 {
                let k = rng.ranged(0, 3 + 2 + 5, half);
                let kind = match k {
                    1..=5 => Some("FlameA"),
                    6..=8 => Some("FlameB"),
                    9 => Some("FlameC"),
                    _ => None,
                };
                if matches!(kind, Some("FlameA" | "FlameB")) {
                    let _ = rng.ranged(1, 3, half);
                }
                if let Some(n) = kind {
                    want.push((n.to_owned(), CellXY::new(top.x + dx, top.y + dy)));
                }
            }
        }
        w.effects.clear();
        assert_eq!(apply!(w, b, hit(600, h)), HitResult::NowYellow);
        let got: Vec<(String, CellXY)> = w
            .effects
            .iter()
            .filter_map(|e| match e {
                Effect::Anim { name, pos } => Some((name.clone(), pos.cell())),
                _ => None,
            })
            .collect();
        assert_eq!(got, want, "seed {seed}");
        assert_eq!(w.rng.taken(), rng.taken());
    }
}

// ---------------------------------------------------------------------------
// Claim 14: death and credit.
// ---------------------------------------------------------------------------

// spec: S-0302 claim 14 (the kill is credited to the attacker, to the attacking house when it differs from the attacker's owner or there is no attacker, and to nobody when both are absent)
#[test]
fn kills_are_credited() {
    let mut r = testkit::combat_ruleset();
    vehicle(&mut r, "BOX", |t| t.strength = Some(50));
    warhead(&mut r, "PLAIN", |h| h.verses = every("1"));
    let mut w = testkit::game_with(&r, SIZE, 1);
    let h = wh(&w, "PLAIN");
    let kills = |w: &World| (w.houses[0].combat.kills, w.houses[1].combat.kills);
    let shooter = put(&mut w, Kind::Vehicle, "BOX", ALPHA, -8);
    let t1 = put(&mut w, Kind::Vehicle, "BOX", BETA, 0);
    let by = Hit {
        attacker: Some(shooter),
        ..hit(100, h)
    };
    assert_eq!(apply!(w, t1, by), HitResult::NowDead);
    assert_eq!(kills(&w), (1, 0));
    let t2 = put(&mut w, Kind::Vehicle, "BOX", BETA, 2);
    apply!(w, t2, hit(100, h));
    assert_eq!(kills(&w), (1, 0));
    let t3 = put(&mut w, Kind::Vehicle, "BOX", ALPHA, 4);
    let house_only = Hit {
        house: Some(HouseId(BETA)),
        ..hit(100, h)
    };
    apply!(w, t3, house_only);
    assert_eq!(kills(&w), (1, 1));
    let t4 = put(&mut w, Kind::Vehicle, "BOX", ALPHA, 6);
    let other_house = Hit {
        attacker: Some(shooter),
        house: Some(HouseId(BETA)),
        ..hit(100, h)
    };
    apply!(w, t4, other_house);
    assert_eq!(kills(&w), (1, 2));
}

// ---------------------------------------------------------------------------
// Claim 15: cyborgs.
// ---------------------------------------------------------------------------

// spec: S-0302 claim 15 and F5 (a killing hit on a cyborg that is not prone leaves ⌊S / 4⌋, at least 1, makes it prone and reports now red; a prone cyborg or a hit that ignores defences kills it)
#[test]
fn cyborgs_are_crippled_before_they_die() {
    let mut r = testkit::combat_ruleset();
    infantry(&mut r, "BORG", |t| t.cyborg = Some(true));
    infantry(&mut r, "TINYBORG", |t| {
        t.cyborg = Some(true);
        t.strength = Some(3);
    });
    warhead(&mut r, "PLAIN", |h| h.verses = every("1"));
    let mut w = testkit::game_with(&r, SIZE, 1);
    let h = wh(&w, "PLAIN");
    let c = put(&mut w, Kind::Infantry, "BORG", BETA, 0);
    assert_eq!(apply!(w, c, hit(2000, h)), HitResult::NowRed);
    assert_eq!(health(&w, c), 250);
    assert!(w.obj(c).unwrap().infantry.as_ref().unwrap().prone);
    assert_eq!(apply!(w, c, hit(2000, h)), HitResult::NowDead);
    let tiny = put(&mut w, Kind::Infantry, "TINYBORG", BETA, 3);
    assert_eq!(apply!(w, tiny, hit(10, h)), HitResult::NowRed);
    assert_eq!(health(&w, tiny), 1);
    let c2 = put(&mut w, Kind::Infantry, "BORG", BETA, -3);
    assert_eq!(apply!(w, c2, ign(hit(2000, h))), HitResult::NowDead);
}

// ---------------------------------------------------------------------------
// Claim 16 and F7: delayed kills.
// ---------------------------------------------------------------------------

fn delay_world(spread: &str, f: impl FnOnce(&mut Ruleset)) -> (World, ObjectId, ObjectId) {
    let mut r = testkit::combat_ruleset();
    building(&mut r, "DEPOT", |b| b.eligible_for_delay_kill = Some(true));
    vehicle(&mut r, "BOX", |_| {});
    warhead(&mut r, "SLOW", |h| {
        h.verses = every("1");
        h.causes_delay_kill = Some(true);
        h.delay_kill_frames = Some(100);
        h.delay_kill_at_max = Some(dec("0.5"));
        h.cell_spread = Some(dec(spread));
    });
    warhead(&mut r, "PLAIN", |h| h.verses = every("1"));
    f(&mut r);
    let mut w = testkit::game_with(&r, SIZE, 1);
    let b = put(&mut w, Kind::Building, "DEPOT", BETA, 0);
    let a = put(&mut w, Kind::Vehicle, "BOX", ALPHA, -8);
    (w, b, a)
}

fn countdown(w: &World, id: ObjectId) -> Option<i32> {
    w.obj(id)
        .unwrap()
        .combat
        .delay_kill_until
        .map(|u| u - w.tick)
}

fn by_at(a: ObjectId, amount: i32, h: WarheadId, r: i32) -> Hit {
    Hit {
        attacker: Some(a),
        distance: r,
        ..hit(amount, h)
    }
}

// spec: S-0302 claim 16 and F7 (a killing hit with a delay-kill warhead leaves 1 health, starts K = trunc(F + ((αF − F) / R′) × r), reports post-mortem and credits the kill at once; the shorter countdown is kept)
#[test]
fn delayed_kill_countdown_and_credit() {
    let (mut w, b, a) = delay_world("2", |_| {});
    let slow = wh(&w, "SLOW");
    assert_eq!(apply!(w, b, by_at(a, 5000, slow, 0)), HitResult::PostMortem);
    assert_eq!(health(&w, b), 1);
    assert_eq!(countdown(&w, b), Some(100));
    assert_eq!(w.houses[0].combat.kills, 1);
    // R′ = 512: at r = 256, K = 100 + (50 − 100) / 512 × 256 = 75, shorter.
    assert_eq!(apply!(w, b, by_at(a, 50, slow, 256)), HitResult::PostMortem);
    assert_eq!(countdown(&w, b), Some(75));
    // A longer one does not replace it.
    apply!(w, b, by_at(a, 50, slow, 0));
    assert_eq!(countdown(&w, b), Some(75));
}

// spec: S-0302 Edge cases (quirks.delay_kill_credited_again on: every further hit that changes health credits the destruction again; a hit by another warhead destroys the building)
#[test]
fn delay_kill_credited_again_on() {
    let (mut w, b, a) = delay_world("2", |_| {});
    let (slow, plain) = (wh(&w, "SLOW"), wh(&w, "PLAIN"));
    apply!(w, b, by_at(a, 5000, slow, 0));
    apply!(w, b, by_at(a, 50, slow, 0));
    assert_eq!(w.houses[0].combat.kills, 2);
    assert_eq!(apply!(w, b, by_at(a, 50, plain, 0)), HitResult::NowDead);
    assert_eq!(w.houses[0].combat.kills, 3);
}

// spec: S-0302 Edge cases (quirks.delay_kill_credited_again off: credited once, the countdown still shortened)
#[test]
fn delay_kill_credited_again_off() {
    let (mut w, b, a) = delay_world("2", |r| r.globals.quirks.delay_kill_credited_again = false);
    let (slow, plain) = (wh(&w, "SLOW"), wh(&w, "PLAIN"));
    apply!(w, b, by_at(a, 5000, slow, 0));
    apply!(w, b, by_at(a, 50, slow, 256));
    assert_eq!(countdown(&w, b), Some(75));
    assert_eq!(w.houses[0].combat.kills, 1);
    apply!(w, b, by_at(a, 50, plain, 0));
    assert_eq!(w.houses[0].combat.kills, 1);
}

// spec: S-0302 Edge cases (quirks.delay_kill_whole_cell_spread on: R′ = 256 × trunc(s), so a spread below one cell gives K = 0)
#[test]
fn delay_kill_whole_cell_spread_on() {
    let (mut w, b, a) = delay_world("0.5", |_| {});
    apply!(w, b, by_at(a, 5000, wh(&w, "SLOW"), 0));
    assert_eq!(countdown(&w, b), Some(0));
}

// spec: S-0302 Edge cases (quirks.delay_kill_whole_cell_spread off: R of F3, so a spread of half a cell gives R = 128 and K = F at r = 0)
#[test]
fn delay_kill_whole_cell_spread_off() {
    let (mut w, b, a) = delay_world("0.5", |r| {
        r.globals.quirks.delay_kill_whole_cell_spread = false
    });
    apply!(w, b, by_at(a, 5000, wh(&w, "SLOW"), 0));
    assert_eq!(countdown(&w, b), Some(100));
    let (mut w, b, a) = delay_world("0.5", |r| {
        r.globals.quirks.delay_kill_whole_cell_spread = false
    });
    // r = 64: K = 100 + (−50 / 128) × 64 = 75.
    apply!(w, b, by_at(a, 5000, wh(&w, "SLOW"), 64));
    assert_eq!(countdown(&w, b), Some(75));
}

// ---------------------------------------------------------------------------
// Claim 17: airborne infantry and the mutating warhead.
// ---------------------------------------------------------------------------

fn airborne_world(quirk: bool) -> (World, ObjectId, WarheadId, WarheadId) {
    let mut r = testkit::combat_ruleset();
    infantry(&mut r, "FLYER", |_| {});
    warhead(&mut r, "MUTA", |h| {
        h.verses = every("1");
        h.inf_death = Some(9);
    });
    warhead(&mut r, "MUTAHALF", |h| {
        h.verses = every("0.5");
        h.inf_death = Some(9);
    });
    r.globals.quirks.airborne_mutation_hit_raised = quirk;
    let mut w = testkit::game_with(&r, SIZE, 1);
    let t = put(&mut w, Kind::Infantry, "FLYER", BETA, 0);
    w.obj_mut(t).unwrap().pos.z += 100;
    let (a, b) = (wh(&w, "MUTA"), wh(&w, "MUTAHALF"));
    (w, t, a, b)
}

// spec: S-0302 claim 17 and Edge cases (quirks.airborne_mutation_hit_raised on: an infantry unit in the air hit by infantry-death kind 9 takes d₀ = 0, raised to 1, so trunc(v); a hit that ignores defences does nothing)
#[test]
fn airborne_mutation_hit_raised_on() {
    let (mut w, t, full, half) = airborne_world(true);
    assert_eq!(loss!(w, t, hit(500, full)).0, 1);
    assert_eq!(loss!(w, t, hit(500, half)).0, 0);
    assert_eq!(loss!(w, t, ign(hit(500, full))).0, 0);
    // On the ground the warhead deals its damage.
    w.obj_mut(t).unwrap().pos.z -= 100;
    assert_eq!(loss!(w, t, hit(500, full)).0, 500);
}

// spec: S-0302 Edge cases (quirks.airborne_mutation_hit_raised off: an airborne infantry unit takes no damage from such a warhead)
#[test]
fn airborne_mutation_hit_raised_off() {
    let (mut w, t, full, _) = airborne_world(false);
    assert_eq!(loss!(w, t, hit(500, full)).0, 0);
}

// ---------------------------------------------------------------------------
// Claim 18 and F8: expected damage.
// ---------------------------------------------------------------------------

fn expected_world(quirk: bool, dmg: i32, f: impl FnOnce(&mut WeaponType)) -> i32 {
    let mut r = testkit::combat_ruleset();
    weapon(&mut r, "Zap", |t| {
        t.damage = Some(dmg);
        t.warhead = Some("PRONE".into());
        f(t);
    });
    warhead(&mut r, "PRONE", |h| {
        h.verses = every("1");
        h.prone_damage = Some(dec("0.5"));
    });
    vehicle(&mut r, "SHOOTER", |t| t.primary = Some("Zap".into()));
    infantry(&mut r, "MAN", |_| {});
    country(&mut r, "Betaland", |c| {
        c.armor_units_mult = Some(dec("2"));
        c.armor_infantry_mult = Some(dec("4"));
    });
    r.globals.quirks.expected_damage_firer_armour = quirk;
    let mut w = testkit::game_with(&r, SIZE, 1);
    let s = put(&mut w, Kind::Vehicle, "SHOOTER", ALPHA, -3);
    let t = put(&mut w, Kind::Infantry, "MAN", BETA, 3);
    w.obj_mut(t).unwrap().infantry.as_mut().unwrap().prone = true;
    let z = wp(&w, "Zap");
    damage::expected_damage(&w, s, z, t)
}

// spec: S-0302 claim 18 and Edge cases (quirks.expected_damage_firer_armour on: the target owner's factor for the firer's kind, no prone factor)
#[test]
fn expected_damage_firer_armour_on() {
    assert_eq!(expected_world(true, 100, |_| {}), 50);
}

// spec: S-0302 Edge cases (quirks.expected_damage_firer_armour off: the target's kind, as a real hit, still without the prone factor)
#[test]
fn expected_damage_firer_armour_off() {
    assert_eq!(expected_world(false, 100, |_| {}), 25);
}

// spec: S-0302 claim 18 and F8 (a sonic-wave or fire-particle weapon expects 0, whatever its damage; any other weapon of damage 0 or below expects its damage field)
#[test]
fn expected_damage_special_cases() {
    assert_eq!(expected_world(true, 100, |t| t.is_sonic = Some(true)), 0);
    assert_eq!(expected_world(true, -30, |t| t.is_sonic = Some(true)), 0);
    assert_eq!(
        expected_world(true, 100, |t| t.use_fire_particles = Some(true)),
        0
    );
    assert_eq!(expected_world(true, -30, |_| {}), -30);
    assert_eq!(expected_world(true, 0, |_| {}), 0);
}

// ---------------------------------------------------------------------------
// Claim 19: when damage lands.
// ---------------------------------------------------------------------------

// spec: S-0302 claim 19 (a detonation's hits take effect at once, inside the detonation, with no delay)
#[test]
fn detonation_damage_lands_at_once() {
    let mut r = testkit::combat_ruleset();
    vehicle(&mut r, "BOX", |_| {});
    warhead(&mut r, "PLAIN", |h| h.verses = every("1"));
    let mut w = testkit::game_with(&r, SIZE, 1);
    let t = put(&mut w, Kind::Vehicle, "BOX", BETA, 0);
    let tick = w.tick;
    let det = Detonation {
        point: w.obj(t).unwrap().pos,
        damage: 70,
        warhead: Some(wh(&w, "PLAIN")),
        source: None,
        house: None,
        resources: true,
    };
    area::detonate(&mut w, det);
    assert_eq!(w.tick, tick);
    assert_eq!(health(&w, t), 930);
}

// ---------------------------------------------------------------------------
// Claim 20: terrain objects.
// ---------------------------------------------------------------------------

fn terrain_world(f: impl FnOnce(&mut Ruleset)) -> (World, ObjectId) {
    let mut r = testkit::combat_ruleset();
    register(&mut r, "terrain_objects", "TREE");
    r.sim.terrain_objects.insert(
        "TREE".into(),
        TerrainObjectType {
            strength: Some(200),
            armor: Some(ArmourClass::Wood),
            ..Default::default()
        },
    );
    warhead(&mut r, "AXE", |h| {
        h.verses = verses(&["1", "1", "1", "1", "1", "1", "0.5", "1", "1", "1", "1"]);
        h.wood = Some(true);
    });
    warhead(&mut r, "PLAIN", |h| h.verses = every("1"));
    f(&mut r);
    let mut w = testkit::game_with(&r, SIZE, 1);
    let ty = w.rules().index_of(Kind::Terrain, "TREE").unwrap();
    let id = glasnost_sim::objects::create(&mut w, Kind::Terrain, ty, None);
    let p = Pos::centre(at(0, 0), 0);
    glasnost_sim::objects::place(&mut w, id, p, 0);
    (w, id)
}

// spec: S-0302 claim 20 (a terrain object takes hits only from a warhead with the wood flag, with the versus value of its class and no defences; no warhead or no wood flag: unaffected)
#[test]
fn terrain_objects_need_the_wood_flag() {
    let (mut w, t) = terrain_world(|_| {});
    assert_eq!(health(&w, t), 200);
    assert_eq!(
        loss!(w, t, hit(100, wh(&w, "PLAIN"))),
        (0, HitResult::Unaffected)
    );
    let none = Hit {
        warhead: None,
        ..hit(100, 0)
    };
    assert_eq!(loss!(w, t, none).0, 0);
    assert_eq!(loss!(w, t, hit(100, wh(&w, "AXE"))).0, 50);
    // A hit that ignores defences lands as it is.
    assert_eq!(loss!(w, t, ign(hit(60, wh(&w, "AXE")))).0, 60);
    // A negative amount heals up to the strength.
    assert_eq!(loss!(w, t, hit(-500, wh(&w, "AXE"))).0, -110);
    // An amount of 0 after the versus value does nothing (no minimum of 1).
    assert_eq!(loss!(w, t, hit(1, wh(&w, "AXE"))).0, 0);
}

// spec: S-0302 claim 20 (a terrain type marked immune stops every hit)
#[test]
fn immune_terrain_objects_take_nothing() {
    let (mut w, t) = terrain_world(|r| {
        r.sim.terrain_objects.get_mut("TREE").unwrap().immune = Some(true);
    });
    assert_eq!(loss!(w, t, hit(100, wh(&w, "AXE"))).0, 0);
    assert_eq!(loss!(w, t, ign(hit(100, wh(&w, "AXE")))).0, 0);
}

// spec: S-0302 claim 20 (a terrain object destroyed at 0 health leaves play and credits nobody)
#[test]
fn destroyed_terrain_objects_credit_nobody() {
    let (mut w, t) = terrain_world(|_| {});
    let shooter = put(&mut w, Kind::Vehicle, "CART", ALPHA, -8);
    let by = Hit {
        attacker: Some(shooter),
        house: Some(HouseId(ALPHA)),
        ..hit(1000, wh(&w, "AXE"))
    };
    assert_eq!(apply!(w, t, by), HitResult::NowDead);
    assert!(w.obj(t).is_none());
    assert_eq!(w.houses[0].combat.kills, 0);
}

// spec: S-0302 claim 20 (an ore-spawning terrain object sets off a detonation of the demolition warhead of 100 at its position when destroyed: scenario D28 part d, a vehicle next to it loses 100)
#[test]
fn ore_spawning_terrain_bursts() {
    let (mut w, t) = terrain_world(|r| {
        r.sim
            .terrain_objects
            .get_mut("TREE")
            .unwrap()
            .spawns_tiberium = Some(true);
        warhead(r, "DEMO", |h| {
            h.verses = every("1");
            h.cell_spread = Some(dec("1.5"));
        });
        r.sim.general.c4_warhead = Some("DEMO".into());
        vehicle(r, "BOX", |_| {});
    });
    let v = put(&mut w, Kind::Vehicle, "BOX", BETA, 1);
    apply!(w, t, hit(1000, wh(&w, "AXE")));
    assert!(w.obj(t).is_none());
    // R = 384, the vehicle 256 leptons away: g = trunc((100 − 100) × … + 100) = 100 (no falloff at p = 1).
    assert_eq!(health(&w, v), 900);
}

// spec: S-0302 claim 8 (stage 2: a building type immune to radiation stops every hit of a radiation warhead)
#[test]
fn radiation_immune_building() {
    for (immune, want) in [(Some(true), 0), (None, 100)] {
        let mut r = testkit::combat_ruleset();
        building(&mut r, "BUNK", |b| b.immune_to_radiation = immune);
        warhead(&mut r, "RAD", |h| {
            h.verses = every("1");
            h.radiation = Some(true);
        });
        let mut w = testkit::game_with(&r, SIZE, 1);
        let t = put(&mut w, Kind::Building, "BUNK", BETA, 0);
        let h = wh(&w, "RAD");
        assert_eq!(loss!(w, t, hit(100, h)).0, want, "{immune:?}");
    }
}
