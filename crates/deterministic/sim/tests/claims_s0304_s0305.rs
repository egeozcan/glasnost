//! Claim tests for the special versus values (S-0304) and the armour
//! classes (S-0305) on synthetic data: invented types, weapons and
//! warheads, every number our own.

use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{
    ArmourClass, Decimal, OverlayType, TerrainObjectType, UnitType, WarheadType, WeaponType,
};
use glasnost_sim::combat::acquire::{self, SearchContext, SearchKind};
use glasnost_sim::combat::check::{self, Answer};
use glasnost_sim::combat::damage::{self, Hit};
use glasnost_sim::combat::data::WarheadId;
use glasnost_sim::combat::effects::Effect;
use glasnost_sim::combat::weapon;
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, Pos, Target, World};

const SIZE: i32 = 24;
const ALPHA: u8 = 0;
const BETA: u8 = 1;

/// τ_r and τ_a written as the exact decimals of S-0304's Constants.
const TAU_R: &str = "0.00999999977648258209228515625";
const TAU_A: &str = "0.0199999995529651641845703125";

fn dec(s: &str) -> Decimal {
    Decimal(s.into())
}

fn every(v: &str) -> Option<Vec<Decimal>> {
    Some(vec![dec(v); 11])
}

/// Eleven items, `v` for class `class` and 1 for the others.
fn only(class: usize, v: &str) -> Option<Vec<Decimal>> {
    let mut l = vec![dec("1"); 11];
    l[class] = dec(v);
    Some(l)
}

fn register(r: &mut Ruleset, reg: &str, name: &str) {
    let l = r.registries.entry(reg.into()).or_default();
    if !l.iter().any(|n| n == name) {
        l.push(name.into());
    }
}

fn vehicle(r: &mut Ruleset, name: &str, f: impl FnOnce(&mut UnitType)) {
    let mut t = r.sim.vehicles["BUGGY"].clone();
    t.strength = Some(1000);
    t.primary = None;
    t.combat.armor = None;
    f(&mut t);
    register(r, "vehicles", name);
    r.sim.vehicles.insert(name.into(), t);
}

fn infantry(r: &mut Ruleset, name: &str, f: impl FnOnce(&mut UnitType)) {
    let mut t = r.sim.infantry["WALKER"].clone();
    t.strength = Some(1000);
    t.primary = None;
    f(&mut t);
    register(r, "infantry", name);
    r.sim.infantry.insert(name.into(), t);
}

fn warhead(r: &mut Ruleset, name: &str, f: impl FnOnce(&mut WarheadType)) {
    let mut h = WarheadType {
        verses: every("1"),
        ..Default::default()
    };
    f(&mut h);
    r.sim.warheads.insert(name.into(), h);
}

/// A direct weapon of damage 20, range 5 cells, firing every 30 ticks.
fn weapon_with(r: &mut Ruleset, name: &str, warhead_name: &str) {
    r.sim.weapons.insert(
        name.into(),
        WeaponType {
            damage: Some(20),
            rof: Some(30),
            range: Some(dec("5")),
            speed: Some(100),
            projectile: Some("Bullet".into()),
            warhead: Some(warhead_name.into()),
            ..Default::default()
        },
    );
}

fn wh(w: &World, name: &str) -> WarheadId {
    w.rules()
        .combat
        .warheads
        .iter()
        .position(|h| h.name == name)
        .unwrap_or_else(|| panic!("no warhead {name}")) as u32
}

fn at(dx: i32, dy: i32) -> CellXY {
    testkit::mid(SIZE, dx, dy)
}

fn put(w: &mut World, kind: Kind, name: &str, house: u8, dx: i32, dy: i32) -> ObjectId {
    testkit::spawn(w, kind, name, house, at(dx, dy), 0)
}

fn health(w: &World, id: ObjectId) -> i32 {
    w.obj(id).map_or(0, |o| o.health)
}

// ---------------------------------------------------------------------------
// S-0304: the three tests of scenario V1.
// ---------------------------------------------------------------------------

/// A defender D of house Alpha armed with a weapon whose warhead gives `v`
/// against class *light* (and 1 elsewhere), and an enemy vehicle X of class
/// *light* `dx` cells away, unarmed unless `armed`.
fn v1(v: &str, armed: bool) -> (World, ObjectId, ObjectId) {
    let mut r = testkit::combat_ruleset();
    warhead(&mut r, "H", |h| h.verses = only(3, v));
    weapon_with(&mut r, "W", "H");
    warhead(&mut r, "PLAIN", |_| {});
    weapon_with(&mut r, "Plain", "PLAIN");
    vehicle(&mut r, "DEFENDER", |t| t.primary = Some("W".into()));
    vehicle(&mut r, "TARGET", |t| {
        t.combat.armor = Some(ArmourClass::Light);
        if armed {
            t.primary = Some("Plain".into());
        }
    });
    let mut w = testkit::game_with(&r, SIZE, 1);
    let d = put(&mut w, Kind::Vehicle, "DEFENDER", ALPHA, -1, 0);
    let x = put(&mut w, Kind::Vehicle, "TARGET", BETA, 2, 0);
    (w, d, x)
}

/// Run 1: whether the firing check of D's weapon against X answers
/// *illegal*.
fn illegal(v: &str) -> bool {
    let (w, d, x) = v1(v, false);
    check::check(&w, d, Some(Target::Object(x)), 0, false) == Answer::Illegal
}

/// Run 2: whether D, hit by X, retaliates.
fn retaliates(v: &str) -> bool {
    let (mut w, d, x) = v1(v, true);
    let plain = wh(&w, "PLAIN");
    let hit = Hit {
        amount: 10,
        warhead: Some(plain),
        distance: 0,
        attacker: Some(x),
        house: Some(HouseId(BETA)),
        ignore_defences: false,
        no_escape: false,
    };
    damage::apply(&mut w, d, hit);
    assert_eq!(health(&w, d), 990);
    w.obj(d).unwrap().target == Some(Target::Object(x))
}

/// Run 3: whether D's automatic search picks X.
fn picked(v: &str) -> bool {
    let (mut w, d, x) = v1(v, false);
    acquire::search(&mut w, d, SearchKind::Range, SearchContext::PerTick) == Some(x)
}

// spec: S-0304 claim 1 and Formulas (legal(v) ⇔ v ≠ 0: exactly 0, also −0, makes a unit illegal; any other value, however small or negative, is legal)
#[test]
fn a_zero_versus_value_makes_the_target_illegal() {
    assert!(illegal("0"));
    assert!(illegal("-0"));
    assert!(illegal("0.0"));
    for v in ["0.0001", "-0.5", "0.005", "1"] {
        assert!(!illegal(v), "{v}");
    }
}

// spec: S-0304 claim 1 (a weapon fired at a cell or at a terrain object is not affected by any versus value)
#[test]
fn cells_and_terrain_objects_are_never_illegal_by_versus() {
    let (w0, d0, _) = v1("0", false);
    assert_ne!(
        check::check(&w0, d0, Some(Target::Cell(at(2, 2))), 0, false),
        Answer::Illegal
    );
    let mut r = testkit::combat_ruleset();
    register(&mut r, "terrain_objects", "TREE");
    r.sim.terrain_objects.insert(
        "TREE".into(),
        TerrainObjectType {
            strength: Some(100),
            armor: Some(ArmourClass::Light),
            ..Default::default()
        },
    );
    warhead(&mut r, "H", |h| h.verses = only(3, "0"));
    weapon_with(&mut r, "W", "H");
    vehicle(&mut r, "DEFENDER", |t| t.primary = Some("W".into()));
    let mut w = testkit::game_with(&r, SIZE, 1);
    let d = put(&mut w, Kind::Vehicle, "DEFENDER", ALPHA, -1, 0);
    let ty = w.rules().index_of(Kind::Terrain, "TREE").unwrap();
    let tree = glasnost_sim::objects::create(&mut w, Kind::Terrain, ty, None);
    glasnost_sim::objects::place(&mut w, tree, Pos::centre(at(2, 0), 0), 0);
    assert_ne!(
        check::check(&w, d, Some(Target::Object(tree)), 0, false),
        Answer::Illegal
    );
}

/// Issues an attack order on X for D (versus value `v`), as a force-fire
/// order would be accepted, with D's rearm countdown running for `rearm`
/// more ticks, and runs `limit` ticks. Returns the tick D dropped X (if it
/// did), the end of the countdown and the number of shots D made.
fn ordered_attack(v: &str, rearm: i32, limit: i32) -> (Option<i32>, i32, usize) {
    let (mut w, d, x) = v1(v, false);
    let order = Order::Behaviour {
        actor: d,
        behaviour: Behaviour::Attack,
        attack_move: false,
        target: Some(Target::Object(x)),
        dest: None,
    };
    // Orders are refused during the start-up order block (S-0213 claim 1).
    while !orders::issue(&mut w, HouseId(ALPHA), order.clone()) {
        glasnost_sim::tick::step(&mut w);
    }
    let until = w.tick + rearm;
    w.obj_mut(d).unwrap().combat.rearm_until = until;
    w.effects.clear();
    let mut held = false;
    let mut dropped = None;
    for _ in 0..limit {
        glasnost_sim::tick::step(&mut w);
        let t = w.obj(d).unwrap().target;
        if t == Some(Target::Object(x)) {
            held = true;
        } else if held && dropped.is_none() {
            dropped = Some(w.tick);
        }
    }
    assert!(held, "the order was accepted");
    let shots = w
        .effects
        .iter()
        .filter(|e| matches!(e, Effect::Shot { firer, .. } if *firer == d))
        .count();
    (dropped, until, shots)
}

// spec: S-0304 claim 2 (an attack order on an illegal target is accepted and the actor never fires) and S-0309 r70's Changes for S-0304 claim 2 (a vehicle that S-0309 claim 29 covers drops it at that claim's periodic check, in a tick whose tick number is a multiple of 16, 1 to 16 ticks after the tick in which the order is applied)
#[test]
fn an_attack_on_an_illegal_target_never_fires() {
    let (dropped, _, shots) = ordered_attack("0", 0, 120);
    // `dropped` is read after the step, one above the tick of the drop.
    let k = dropped.expect("dropped") - 1;
    assert_eq!(k.rem_euclid(16), 0, "dropped in a check tick: {k}");
    assert_eq!(shots, 0);
    // The same order on a legal target fires and keeps it.
    let (dropped, _, shots) = ordered_attack("1", 0, 120);
    assert_eq!(dropped, None);
    assert!(shots > 0);
}

// spec: S-0304 claim 2 (while the rearm countdown runs the check answers rearming, so the target is kept until the countdown has run out, then dropped) and S-0309 r70's Changes for S-0304 claim 2 with S-0309 claim 29 (otherwise the first tick whose tick number is a multiple of 16 after they have ended; scenario V1 run 4b)
#[test]
fn an_illegal_target_is_kept_while_rearming() {
    let (dropped, until, shots) = ordered_attack("0", 80, 300);
    // `dropped` is read after the step, one above the tick of the drop.
    let k = dropped.expect("dropped after the countdown") - 1;
    assert!(k >= until, "kept until the countdown has run out");
    assert_eq!(k.rem_euclid(16), 0, "dropped in a check tick: {k}");
    assert!(k < until + 16, "at the first check tick after it");
    assert_eq!(shots, 0);
}

// spec: S-0304 claim 3 and Formulas (may retaliate(v) ⇔ v > τ_r: 1 % and 0.0099999999 retaliate; τ_r itself, 0.005, 0 and negative values do not)
#[test]
fn retaliation_needs_a_versus_value_above_tau_r() {
    for v in ["0.0099999999", "0.01", "0.015", "0.02", "1"] {
        assert!(retaliates(v), "{v}");
    }
    for v in [TAU_R, "0.005", "0", "-0.5"] {
        assert!(!retaliates(v), "{v}");
    }
}

// spec: S-0304 claim 4 and Formulas (automatic candidate(v) ⇔ v > τ_a: 2 % and 0.0199999999 are picked; τ_a itself, 1.5 %, 1 % and below are not)
#[test]
fn automatic_searches_need_a_versus_value_above_tau_a() {
    for v in ["0.0199999999", "0.02", "0.025", "1"] {
        assert!(picked(v), "{v}");
    }
    for v in [TAU_A, "0.015", "0.01", "0.0099999999", "0.005", "0", "-0.5"] {
        assert!(!picked(v), "{v}");
    }
}

// spec: S-0304 claim 5 (the bands of scenario V1's expected table: ordered attack, retaliation, automatic search)
#[test]
fn the_bands_of_scenario_v1() {
    let table = [
        ("-0.5", true, false, false),
        ("0", false, false, false),
        ("0.005", true, false, false),
        ("0.0099999999", true, true, false),
        ("0.01", true, true, false),
        ("0.015", true, true, false),
        ("0.0199999999", true, true, true),
        ("0.02", true, true, true),
        ("0.025", true, true, true),
    ];
    for (v, legal, ret, pick) in table {
        assert_eq!(
            (!illegal(v), retaliates(v), picked(v)),
            (legal, ret, pick),
            "{v}"
        );
    }
}

// spec: S-0304 Constants (τ_r = 5368709 / 2²⁹ and τ_a = 5368709 / 2²⁸, the singles nearest 0.01 and 0.02)
#[test]
fn the_thresholds() {
    let rz = glasnost_num::Rounding::TowardZero;
    let n = glasnost_num::F64::from_i64(5368709, rz);
    let tr = n.div(glasnost_num::F64::from_i64(1 << 29, rz), rz);
    let ta = n.div(glasnost_num::F64::from_i64(1 << 28, rz), rz);
    assert!(acquire::tau_r().eq_num(tr));
    assert!(acquire::tau_a().eq_num(ta));
    assert!(acquire::tau_r().eq_num(glasnost_num::decimal::nearest_double(TAU_R)));
}

// spec: S-0304 claim 6 (weapon choice: a weapon whose versus value is exactly 0 is not chosen if the other's is not; both 0 gives the primary; small values play no part)
#[test]
fn weapon_choice_avoids_a_zero_versus_value() {
    for (p, s, slot) in [
        ("1", "0", 0),
        ("0", "1", 1),
        ("0", "0", 0),
        ("0.0001", "1", 0),
        ("1", "0.0001", 0),
    ] {
        let mut r = testkit::combat_ruleset();
        warhead(&mut r, "P", |h| h.verses = only(3, p));
        warhead(&mut r, "S", |h| h.verses = only(3, s));
        weapon_with(&mut r, "Pw", "P");
        weapon_with(&mut r, "Sw", "S");
        vehicle(&mut r, "TWOGUN", |t| {
            t.primary = Some("Pw".into());
            t.secondary = Some("Sw".into());
        });
        vehicle(&mut r, "TARGET", |t| {
            t.combat.armor = Some(ArmourClass::Light)
        });
        let mut w = testkit::game_with(&r, SIZE, 1);
        let a = put(&mut w, Kind::Vehicle, "TWOGUN", ALPHA, -1, 0);
        let x = put(&mut w, Kind::Vehicle, "TARGET", BETA, 2, 0);
        assert_eq!(
            weapon::choose_slot(&w, a, Some(Target::Object(x))),
            slot,
            "primary {p}, secondary {s}"
        );
    }
}

/// An armed infantry unit of house Alpha whose warhead has `v4` and `v6`
/// for classes 4 and 6 (1 for the rest), an enemy infantry unit 3 cells
/// away and an enemy vehicle 2 cells away, both of class *light*.
fn infantry_only_world(v4: &str, v6: &str) -> (World, ObjectId, ObjectId, ObjectId) {
    let mut r = testkit::combat_ruleset();
    warhead(&mut r, "PICKY", |h| {
        let mut l = vec![dec("1"); 11];
        l[4] = dec(v4);
        l[6] = dec(v6);
        h.verses = Some(l);
    });
    weapon_with(&mut r, "Picky", "PICKY");
    infantry(&mut r, "SNIPER", |t| t.primary = Some("Picky".into()));
    infantry(&mut r, "MAN", |t| t.combat.armor = Some(ArmourClass::Light));
    vehicle(&mut r, "BOX", |t| t.combat.armor = Some(ArmourClass::Light));
    let mut w = testkit::game_with(&r, SIZE, 1);
    let a = put(&mut w, Kind::Infantry, "SNIPER", ALPHA, -1, 0);
    let man = put(&mut w, Kind::Infantry, "MAN", BETA, 2, 0);
    let bx = put(&mut w, Kind::Vehicle, "BOX", BETA, 1, 0);
    (w, a, man, bx)
}

// spec: S-0304 claim 8 and S-0305 claim 6 (a primary warhead whose versus values for classes 4 and 6 are both exactly 0 makes an armed infantry unit search for infantry only; a merely small value does not)
#[test]
fn the_infantry_only_rule() {
    let (w, a, ..) = infantry_only_world("0", "0");
    let c = acquire::classes(&w, a);
    assert!(c.infantry && !c.vehicles && !c.buildings);
    let (w, a, ..) = infantry_only_world("0.0001", "0");
    assert!(acquire::classes(&w, a).vehicles);
    let (w, a, ..) = infantry_only_world("0", "1");
    assert!(acquire::classes(&w, a).vehicles);
}

// spec: S-0305 claim 6 (scenario A7: the infantry-only unit picks the infantry unit, the other picks the nearer vehicle)
#[test]
fn infantry_only_units_pick_infantry() {
    let (mut w, a, man, _) = infantry_only_world("0", "0");
    assert_eq!(
        acquire::search(&mut w, a, SearchKind::Range, SearchContext::PerTick),
        Some(man)
    );
    let (mut w, a, man, _) = infantry_only_world("0.0001", "0");
    let pick = acquire::search(&mut w, a, SearchKind::Range, SearchContext::PerTick);
    // The nearer vehicle (class light, value 1) is a candidate again.
    assert!(pick.is_some_and(|p| p != man));
}

// ---------------------------------------------------------------------------
// S-0305: armour classes.
// ---------------------------------------------------------------------------

// spec: S-0305 claim 1 (eleven classes in a fixed order: none, flak, plate, light, medium, heavy, wood, steel, concrete, special 1, special 2)
#[test]
fn the_class_list() {
    let words = [
        "none",
        "flak",
        "plate",
        "light",
        "medium",
        "heavy",
        "wood",
        "steel",
        "concrete",
        "special_1",
        "special_2",
    ];
    assert_eq!(ArmourClass::ALL.len(), 11);
    assert_eq!(ArmourClass::WORDS, words);
    for (i, word) in words.iter().enumerate() {
        let (c, known) = ArmourClass::from_word(word.as_bytes());
        assert!(known);
        assert_eq!(c.index(), i, "{word}");
        assert_eq!(ArmourClass::ALL[i], c);
    }
}

// spec: S-0305 claim 1 (scenario A1: a warhead of k/16 for the k-th class, damage 160, costs the vehicle of the k-th class 10 k)
#[test]
fn class_order_through_damage() {
    let vs: Vec<String> = (1..=11)
        .map(|k| {
            // k / 16, exact in binary.
            let n = k * 625;
            format!("0.{n:04}")
        })
        .collect();
    let mut r = testkit::combat_ruleset();
    for (i, c) in ArmourClass::ALL.iter().enumerate() {
        vehicle(&mut r, &format!("V{i}"), |t| t.combat.armor = Some(*c));
    }
    vehicle(&mut r, "BARE", |_| {});
    warhead(&mut r, "A1", |h| {
        h.verses = Some(vs.iter().map(|s| dec(s)).collect())
    });
    let mut w = testkit::game_with(&r, SIZE, 1);
    let h = wh(&w, "A1");
    let shot = Hit {
        amount: 160,
        warhead: Some(h),
        distance: 0,
        attacker: None,
        house: None,
        ignore_defences: false,
        no_escape: false,
    };
    for i in 0..11 {
        let v = put(&mut w, Kind::Vehicle, &format!("V{i}"), BETA, i - 5, 0);
        damage::apply(&mut w, v, shot);
        assert_eq!(1000 - health(&w, v), 10 * (i + 1), "class {i}");
    }
    // spec: S-0305 claim 2 (scenario A2: a type with no armour field has class none and loses 10)
    let bare = put(&mut w, Kind::Vehicle, "BARE", BETA, 0, 4);
    damage::apply(&mut w, bare, shot);
    assert_eq!(1000 - health(&w, bare), 10);
}

// spec: S-0305 claim 2 (every object type has one class; a type no layer has given one has class none: infantry, vehicles, buildings, overlays, terrain objects)
#[test]
fn types_without_the_field_have_class_none() {
    let mut r = testkit::combat_ruleset();
    vehicle(&mut r, "BARE", |_| {});
    infantry(&mut r, "MAN", |_| {});
    register(&mut r, "buildings", "SHED");
    r.sim.buildings.insert(
        "SHED".into(),
        glasnost_rules::sim::BuildingType {
            strength: Some(100),
            ..Default::default()
        },
    );
    register(&mut r, "terrain_objects", "TREE");
    r.sim
        .terrain_objects
        .insert("TREE".into(), TerrainObjectType::default());
    register(&mut r, "overlays", "ROCKS");
    r.sim
        .overlays
        .insert("ROCKS".into(), OverlayType::default());
    let mut w = testkit::game_with(&r, SIZE, 1);
    let v = put(&mut w, Kind::Vehicle, "BARE", BETA, 0, 0);
    let m = put(&mut w, Kind::Infantry, "MAN", BETA, 2, 0);
    let s = put(&mut w, Kind::Building, "SHED", BETA, -3, 0);
    for id in [v, m, s] {
        assert_eq!(
            weapon::armour_class(&w, Target::Object(id)),
            Some(ArmourClass::None.index())
        );
    }
    let rules = w.rules();
    let tree = rules.index_of(Kind::Terrain, "TREE").unwrap() as usize;
    assert_eq!(rules.terrain[tree].armour, 0);
    let rocks = rules.overlays.iter().find(|o| o.name == "ROCKS").unwrap();
    assert_eq!(rocks.armour, 0);
}

// spec: S-0305 claim 3 and Formulas (the whole value is compared, folding A to Z only: Heavy, HEAVY and heavy give class 5; heavy2, 5 and heavy armour match nothing)
#[test]
fn armour_words_are_matched_whole_ignoring_case() {
    for word in ["Heavy", "HEAVY", "heavy", "hEaVy"] {
        assert_eq!(
            ArmourClass::from_word(word.as_bytes()),
            (ArmourClass::Heavy, true),
            "{word}"
        );
    }
    assert_eq!(
        ArmourClass::from_word(b"SPECIAL_2"),
        (ArmourClass::Special2, true)
    );
    for word in [
        &b"heavy2"[..],
        b"5",
        b"heavy armour",
        b"hea",
        b"special 1",
        b"HEAV\xdd",
    ] {
        assert_eq!(ArmourClass::from_word(word), (ArmourClass::None, false));
    }
}

fn overlay_class(armor: Option<&str>, tiberium: bool) -> usize {
    let mut r = testkit::combat_ruleset();
    register(&mut r, "overlays", "TEST");
    r.sim.overlays.insert(
        "TEST".into(),
        OverlayType {
            armor: armor.map(String::from),
            tiberium: Some(tiberium),
            ..Default::default()
        },
    );
    let w = testkit::game_with(&r, SIZE, 1);
    let rules = w.rules();
    rules
        .overlays
        .iter()
        .find(|o| o.name == "TEST")
        .unwrap()
        .armour
}

// spec: S-0305 claims 3 and 4 (an overlay's armour word becomes its class by the same matching; a word that matches nothing gives class none)
#[test]
fn overlay_armour_words() {
    assert_eq!(overlay_class(Some("Steel"), false), 7);
    assert_eq!(overlay_class(Some("WOOD"), false), 6);
    assert_eq!(overlay_class(Some("nosuchclass"), false), 0);
    assert_eq!(overlay_class(None, false), 0);
}

// spec: S-0305 claim 5 (a resource overlay has class wood whatever its armour field says)
#[test]
fn resource_overlays_are_always_wood() {
    assert_eq!(overlay_class(Some("steel"), true), 6);
    assert_eq!(overlay_class(Some("nosuchclass"), true), 6);
    assert_eq!(overlay_class(None, true), 6);
}

// spec: S-0305 claim 6 (the class selects the versus value whenever damage is resolved: the same warhead costs two classes different amounts)
#[test]
fn the_class_selects_the_versus_value() {
    let mut r = testkit::combat_ruleset();
    vehicle(&mut r, "TIN", |t| t.combat.armor = Some(ArmourClass::Light));
    vehicle(&mut r, "IRON", |t| {
        t.combat.armor = Some(ArmourClass::Heavy)
    });
    warhead(&mut r, "AP", |h| {
        let mut l = vec![dec("1"); 11];
        l[3] = dec("0.5");
        l[5] = dec("0.25");
        h.verses = Some(l);
    });
    let mut w = testkit::game_with(&r, SIZE, 1);
    let h = wh(&w, "AP");
    let tin = put(&mut w, Kind::Vehicle, "TIN", BETA, -2, 0);
    let iron = put(&mut w, Kind::Vehicle, "IRON", BETA, 2, 0);
    let shot = Hit {
        amount: 100,
        warhead: Some(h),
        distance: 0,
        attacker: None,
        house: None,
        ignore_defences: false,
        no_escape: false,
    };
    damage::apply(&mut w, tin, shot);
    damage::apply(&mut w, iron, shot);
    assert_eq!((health(&w, tin), health(&w, iron)), (950, 975));
}

// spec: S-0305 claim 6 (a wall overlay of class wood takes wall hits from a warhead with the wood flag; one of class steel does not)
#[test]
fn wood_walls_and_the_wood_flag() {
    let mut r = testkit::combat_ruleset();
    for (name, armor) in [("PALISADE", "wood"), ("RAMPART", "steel")] {
        register(&mut r, "overlays", name);
        r.sim.overlays.insert(
            name.into(),
            OverlayType {
                wall: Some(true),
                strength: Some(10),
                damage_levels: Some(3),
                armor: Some(armor.into()),
                has_art: Some(true),
                ..Default::default()
            },
        );
    }
    warhead(&mut r, "CHOP", |h| h.wood = Some(true));
    let mut w = testkit::game_with(&r, SIZE, 1);
    let h = wh(&w, "CHOP");
    let idx =
        |w: &World, n: &str| w.rules().overlays.iter().position(|o| o.name == n).unwrap() as u8;
    let (pal, ram) = (idx(&w, "PALISADE"), idx(&w, "RAMPART"));
    for (dx, o) in [(0, pal), (5, ram)] {
        let cell = w.cells.grid.get_mut(at(dx, 0)).unwrap();
        cell.overlay = o;
        cell.overlay_data = 0;
    }
    for dx in [0, 5] {
        glasnost_sim::combat::area::detonate(
            &mut w,
            glasnost_sim::combat::area::Detonation {
                point: Pos::centre(at(dx, 0), 0),
                damage: 50,
                warhead: Some(h),
                source: None,
                house: None,
                resources: true,
            },
        );
    }
    let data = |w: &World, dx| w.cells.cell(at(dx, 0)).unwrap().overlay_data;
    assert_eq!(data(&w, 0), 16);
    assert_eq!(data(&w, 5), 0);
}

// spec: S-0305 claim 6 (infantry-only is exact: classes 4 and 6 both exactly 0; a small value or a missing list, eleven items of 100 %, is not)
#[test]
fn the_infantry_only_mark_of_a_warhead() {
    let mut r = testkit::combat_ruleset();
    warhead(&mut r, "ZERO", |h| {
        let mut l = vec![dec("1"); 11];
        l[4] = dec("0");
        l[6] = dec("-0");
        h.verses = Some(l);
    });
    warhead(&mut r, "SMALL", |h| {
        let mut l = vec![dec("1"); 11];
        l[4] = dec("0.0001");
        l[6] = dec("0");
        h.verses = Some(l);
    });
    r.sim
        .warheads
        .insert("BLANK".into(), WarheadType::default());
    let w = testkit::game_with(&r, SIZE, 1);
    let c = &w.rules().combat;
    assert!(c.warhead(wh(&w, "ZERO")).infantry_only());
    assert!(!c.warhead(wh(&w, "SMALL")).infantry_only());
    assert!(!c.warhead(wh(&w, "BLANK")).infantry_only());
}
