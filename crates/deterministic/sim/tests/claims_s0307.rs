//! Claim tests for S-0307 (the weapon range test), on invented rules and a
//! flat synthetic map. Each test drives `combat::range::in_range` (or the
//! firing check) with objects placed by hand.

use glasnost_map::CellXY;
use glasnost_num::Rounding;
use glasnost_num::geom::root_floor;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{Decimal, OverlayType, ProjectileType, WarheadType, WeaponType};
use glasnost_sim::combat::ballistics;
use glasnost_sim::combat::data::leptons;
use glasnost_sim::combat::range::{firer_weapon_range, in_range, measured_point, source_point};
use glasnost_sim::rules::Kind;
use glasnost_sim::testkit;
use glasnost_sim::world::{Layer, ObjectId, Pos, Target, World};

const N: i32 = 40;
const RZ: Rounding = Rounding::TowardZero;

fn d(s: &str) -> Decimal {
    Decimal(s.into())
}

/// The combat ruleset plus a test weapon `Probe` (straight projectile
/// `Line`, warhead `SA`) carried by the cart, and a wall-damaging warhead.
fn base(range: &str) -> Ruleset {
    let mut r = testkit::combat_ruleset();
    r.sim.weapons.insert(
        "Probe".into(),
        WeaponType {
            damage: Some(10),
            rof: Some(50),
            range: Some(d(range)),
            speed: Some(50),
            projectile: Some("Line".into()),
            warhead: Some("SA".into()),
            ..Default::default()
        },
    );
    r.sim
        .projectiles
        .insert("Line".into(), ProjectileType::default());
    r.sim.warheads.insert(
        "WALLER".into(),
        WarheadType {
            wall: Some(true),
            ..Default::default()
        },
    );
    r.sim.vehicles.get_mut("CART").unwrap().primary = Some("Probe".into());
    r
}

fn weapon(r: &mut Ruleset) -> &mut WeaponType {
    r.sim.weapons.get_mut("Probe").unwrap()
}

fn line(r: &mut Ruleset) -> &mut ProjectileType {
    r.sim.projectiles.get_mut("Line").unwrap()
}

/// Registers 244 overlay types; those at the given indices have the wall
/// property.
fn overlays(r: &mut Ruleset, walls: &[usize]) {
    let names: Vec<String> = (0..244).map(|i| format!("OV{i:03}")).collect();
    for (i, n) in names.iter().enumerate() {
        r.sim.overlays.insert(
            n.clone(),
            OverlayType {
                wall: Some(walls.contains(&i)),
                ..Default::default()
            },
        );
    }
    r.registries.insert("overlays".into(), names);
}

struct S {
    w: World,
    a: ObjectId,
    t: ObjectId,
    wp: u32,
}

/// The firer (a cart of house 0) at the middle cell and an unarmed buggy of
/// house 1 at offset (dx, dy) cells.
fn setup(r: &Ruleset, dx: i32, dy: i32) -> S {
    let mut w = testkit::game_with(r, N, 1);
    let a = testkit::spawn(&mut w, Kind::Vehicle, "CART", 0, testkit::mid(N, 0, 0), 0);
    let t = testkit::spawn(
        &mut w,
        Kind::Vehicle,
        "BUGGY",
        1,
        testkit::mid(N, dx, dy),
        0,
    );
    let wp = w.rules().combat.weapon_index["Probe"];
    S { w, a, t, wp }
}

fn test(s: &S) -> bool {
    in_range(&s.w, s.a, Some(s.wp), Some(Target::Object(s.t)), None)
}

/// The smallest planar offset whose table-root distance exceeds `lim`
/// (S-0204 claim 5: the table root can be one short).
fn first_beyond(lim: i32) -> i32 {
    let mut x = lim;
    while root_floor((i64::from(x) * i64::from(x)) as u64, true, RZ) as i32 <= lim {
        x += 1;
    }
    x
}

fn cell_of(dx: i32, dy: i32) -> CellXY {
    testkit::mid(N, dx, dy)
}

/// Sets a cell's height level.
fn set_level(w: &mut World, c: CellXY, level: i8) {
    w.cells.grid.get_mut(c).unwrap().level = level;
}

/// Marks a cell as a high-bridge cell (with a bridge deck).
fn set_deck(w: &mut World, c: CellXY) {
    let i = w.cells.idx(c).unwrap();
    w.cells.deck[i] = true;
}

fn set_overlay(w: &mut World, c: CellXY, o: u8) {
    w.cells.grid.get_mut(c).unwrap().overlay = o;
}

/// Moves an object to a position (no cell bookkeeping: the range test reads
/// positions only).
fn put(w: &mut World, id: ObjectId, p: Pos) {
    w.obj_mut(id).unwrap().pos = p;
}

/// Puts an object back on the surface of its cell after a height change.
fn ground(w: &mut World, id: ObjectId) {
    let p = w.obj(id).unwrap().pos;
    let z = w.surface_at(p.x, p.y);
    put(w, id, Pos::new(p.x, p.y, z));
}

// spec: S-0307 claim 1 (R₀ = trunc(RZ₅₃(x × 256)), x read as a single; scenario RG1 b and b′)
#[test]
fn claim1_range_field_in_leptons_truncates() {
    assert_eq!(leptons(&d("5"), RZ), 1280);
    assert_eq!(leptons(&d("4.99"), RZ), 1277);
    assert_eq!(leptons(&d("4.9999"), RZ), 1279);
    assert_eq!(leptons(&d("-2"), RZ), -512);
    // Edge cases: a field slightly beyond −2 still truncates to −512.
    assert_eq!(leptons(&d("-2.001"), RZ), -512);
}

// spec: S-0307 claim 1 and claim 9 (scenario RG1 runs a, b and b′: a target 5 cells away)
#[test]
fn claim1_rg1_range_fields_at_five_cells() {
    for (range, fires) in [("5", true), ("4.99", false), ("4.9999", false)] {
        let s = setup(&base(range), 5, 0);
        assert_eq!(test(&s), fires, "range field {range}");
    }
}

// spec: S-0307 claim 1 (the minimum-range field and the air range bonus use the same lepton form)
#[test]
fn claim1_minimum_range_and_air_bonus_lepton_form() {
    let mut r = base("6");
    weapon(&mut r).minimum_range = Some(d("1.999"));
    r.sim
        .vehicles
        .get_mut("CART")
        .unwrap()
        .combat
        .air_range_bonus = Some(d("0.999"));
    let s = setup(&r, 3, 0);
    // trunc(1.999 × 256) = 511 (not 512); trunc(0.999 × 256) = 255.
    assert_eq!(s.w.rules().combat.weapon(s.wp).min_range, 511);
    let bonus = glasnost_sim::combat::veterancy::type_combat(&s.w, s.a)
        .unwrap()
        .air_range_bonus;
    assert_eq!(bonus, 255);
}

// spec: S-0307 claim 2 (R₀ = −512 passes for every target; scenario RG1 run c)
#[test]
fn claim2_unlimited_range_passes_far_targets() {
    let s = setup(&base("-2"), 40, 0);
    assert!(test(&s));
    // Even farther than the map: the distance plays no part.
    let mut s = setup(&base("-2"), 3, 0);
    put(&mut s.w, s.t, Pos::new(200_000, 3000, 0));
    assert!(test(&s));
}

// spec: S-0307 claim 2 (no minimum range, no height rule, no line-of-fire walk for an unlimited weapon)
#[test]
fn claim2_unlimited_range_ignores_minimum_range_and_line_of_fire() {
    let mut r = base("-2");
    weapon(&mut r).minimum_range = Some(d("3"));
    line(&mut r).subject_to_walls = Some(true);
    line(&mut r).subject_to_cliffs = Some(true);
    overlays(&mut r, &[7]);
    let mut s = setup(&r, 4, 0);
    set_overlay(&mut s.w, cell_of(2, 0), 7);
    set_level(&mut s.w, cell_of(1, 0), 6);
    assert!(test(&s));
    // Inside the minimum range too.
    assert!(test(&setup(&r, 1, 0)));
    // A finite range with the same data refuses both.
    let mut r2 = r.clone();
    weapon(&mut r2).range = Some(d("10"));
    let mut t = setup(&r2, 4, 0);
    set_overlay(&mut t.w, cell_of(2, 0), 7);
    assert!(!test(&t));
    let t2 = setup(&r2, 1, 0);
    assert!(!test(&t2));
}

// spec: S-0307 claim 3 (an airborne target adds the firer type's air range bonus; scenario RG2 run a)
#[test]
fn claim3_air_bonus_for_an_airborne_target() {
    for (bonus, fires) in [("1", true), ("0", false)] {
        let mut r = base("5");
        r.sim
            .vehicles
            .get_mut("CART")
            .unwrap()
            .combat
            .air_range_bonus = Some(d(bonus));
        let mut s = setup(&r, 5, 0);
        // Hovering 300 leptons up: spatial distance ⌊r(1280² + 300²)⌋ lies in (1280, 1536].
        let p = s.w.obj(s.t).unwrap().pos;
        put(&mut s.w, s.t, Pos::new(p.x, p.y, 300));
        assert_eq!(test(&s), fires, "air bonus {bonus}");
    }
}

// spec: S-0307 claim 3 (the air bonus is added only when the target is airborne)
#[test]
fn claim3_no_air_bonus_for_a_floor_bound_target() {
    let mut r = base("5");
    r.sim
        .vehicles
        .get_mut("CART")
        .unwrap()
        .combat
        .air_range_bonus = Some(d("1"));
    // 5 cells and a bit: beyond 1280 on the ground.
    let mut s = setup(&r, 5, 0);
    let p = s.w.obj(s.t).unwrap().pos;
    put(&mut s.w, s.t, Pos::new(p.x + 100, p.y, 0));
    assert!(!test(&s));
    // At 207 leptons above the surface the target is still floor-bound (measured on the surface).
    put(&mut s.w, s.t, Pos::new(p.x + 100, p.y, 207));
    assert!(!test(&s));
    // At 208 it is airborne and the bonus applies.
    put(&mut s.w, s.t, Pos::new(p.x + 100, p.y, 208));
    assert!(test(&s));
}

// spec: S-0307 F1 (every sum is reduced with w: R₀ + air bonus wraps past 2³¹ − 1)
#[test]
fn f1_effective_range_wraps_in_32_bits() {
    let mut r = base("8388000");
    r.sim
        .vehicles
        .get_mut("CART")
        .unwrap()
        .combat
        .air_range_bonus = Some(d("1000"));
    let mut s = setup(&r, 2, 0);
    // Floor-bound: R = 2147328000, in range.
    assert!(test(&s));
    // Airborne: R = w(2147328000 + 256000) < 0, out of range.
    let p = s.w.obj(s.t).unwrap().pos;
    put(&mut s.w, s.t, Pos::new(p.x, p.y, 300));
    assert!(!test(&s));
}

/// The elevation test file of scenario RG3: increment 1, bonus 0.5, cap
/// 1.0, range 4.
fn elevation_rules(subject: bool, arcing: bool) -> Ruleset {
    let mut r = base("4");
    r.sim.general.elevation_increment = Some(1);
    r.sim.general.elevation_increment_bonus = Some(d("0.5"));
    r.sim.general.elevation_bonus_cap = Some(d("1.0"));
    line(&mut r).subject_to_elevation = Some(subject);
    line(&mut r).arcing = Some(arcing);
    r
}

/// The firer on a plateau cell 4 levels high, the target at offset (dx, dy)
/// on level 0.
fn plateau(r: &Ruleset, dx: i32, dy: i32) -> S {
    let mut s = setup(r, dx, dy);
    set_level(&mut s.w, cell_of(0, 0), 4);
    ground(&mut s.w, s.a);
    s
}

// spec: S-0307 claim 6 and F2 (scenario RG3: E₂ = 256, E₁ = ⌊r(256² + 416²)⌋ = 488, limit 1512)
#[test]
fn claim6_rg3_straight_subject_to_elevation() {
    assert_eq!(root_floor(256 * 256 + 416 * 416, true, RZ), 488);
    let r = elevation_rules(true, false);
    for ((dx, dy), fires) in [
        ((3, 0), true),
        ((4, 0), true),
        ((5, 0), true),
        ((5, 1), true),
        ((6, 0), false),
    ] {
        assert_eq!(test(&plateau(&r, dx, dy)), fires, "target at ({dx}, {dy})");
    }
}

// spec: S-0307 claim 6 (without the subject-to-elevation property E = 0; scenario RG3 control)
#[test]
fn claim6_rg3_control_without_the_property() {
    let r = elevation_rules(false, false);
    for (dx, fires) in [(3, true), (4, false), (5, false)] {
        assert_eq!(test(&plateau(&r, dx, 0)), fires, "target at {dx}");
    }
}

// spec: S-0307 claim 6 and claim 9 (an arcing projectile takes E₂ on the planar limit 1280; scenario RG3)
#[test]
fn claim6_rg3_arcing_subject_to_elevation() {
    let r = elevation_rules(true, true);
    for ((dx, dy), fires) in [
        ((3, 0), true),
        ((4, 0), true),
        ((5, 0), true),
        ((5, 1), false),
        ((6, 0), false),
    ] {
        assert_eq!(test(&plateau(&r, dx, dy)), fires, "target at ({dx}, {dy})");
    }
}

// spec: S-0307 claim 6 (a firer whose cell is not higher than the target's gets Δh = 0, E = 0)
#[test]
fn claim6_lower_firer_gets_no_bonus() {
    let r = elevation_rules(true, false);
    // Firer on level 0, target on a 4-level plateau 4 cells away: limit stays 1024 + ⌊r(0)⌋.
    let mut s = setup(&r, 4, 0);
    set_level(&mut s.w, cell_of(4, 0), 4);
    ground(&mut s.w, s.t);
    // Spatial distance ⌊r(1024² + 416²)⌋ > 1024.
    assert!(!test(&s));
}

// spec: S-0307 claim 6 (an airborne firer or target gets E = 0)
#[test]
fn claim6_airborne_objects_get_no_bonus() {
    let r = elevation_rules(true, false);
    let mut s = plateau(&r, 5, 0);
    assert!(test(&s));
    // The target hovering 208 above its surface: no bonus (the limit falls to 1024).
    let p = s.w.obj(s.t).unwrap().pos;
    put(&mut s.w, s.t, Pos::new(p.x, p.y, 208));
    assert!(!test(&s));
}

// spec: S-0307 Edge cases (elevation increment 0: E₁ = E₂ = 0, a departure)
#[test]
fn edge_elevation_increment_zero_gives_no_bonus() {
    let mut r = elevation_rules(true, false);
    r.sim.general.elevation_increment = Some(0);
    assert!(test(&plateau(&r, 3, 0)));
    assert!(!test(&plateau(&r, 4, 0)));
}

// spec: S-0307 Edge cases (a negative increment: k = trunc(Δh / I) ≤ 0, E₂ ≤ 0, E₁ the root of the hypotenuse)
#[test]
fn edge_negative_elevation_increment() {
    let mut r = elevation_rules(true, false);
    r.sim.general.elevation_increment = Some(-1);
    // k = −4, f = −2, f′ = −2, E₂ = −512, E₁ = ⌊r(512² + 416²)⌋ = 659; straight limit 1024 + 659.
    let e1 = root_floor(512 * 512 + 416 * 416, true, RZ) as i32;
    assert_eq!(e1, 659);
    // 6 cells: spatial ⌊r(1536² + 416²)⌋ ≤ 1683.
    assert!(test(&plateau(&r, 6, 0)));
    assert!(!test(&plateau(&r, 7, 0)));
    // The arcing limit is 1024 − 512 = 512 planar.
    let mut ra = r.clone();
    line(&mut ra).arcing = Some(true);
    assert!(test(&plateau(&ra, 2, 0)));
    assert!(!test(&plateau(&ra, 3, 0)));
}

// spec: S-0307 Edge cases (a negative cap applies even at Δh = 0: E₂ = 256 × trunc(c), E₁ = |E₂|)
#[test]
fn edge_negative_elevation_cap_on_flat_ground() {
    let mut r = elevation_rules(true, false);
    r.sim.general.elevation_bonus_cap = Some(d("-1"));
    // Straight: limit 1024 + 256 on flat ground.
    assert!(test(&setup(&r, 5, 0)));
    assert!(!test(&setup(&r, 6, 0)));
    // Arcing: limit 1024 − 256 = 768.
    let mut ra = r.clone();
    line(&mut ra).arcing = Some(true);
    assert!(test(&setup(&ra, 3, 0)));
    assert!(!test(&setup(&ra, 4, 0)));
}

// spec: S-0307 F2 and Edge cases (overflow: I = 1, b = 200, c = 200, Δh = 1: σ wraps, E₁ = ⌊r(|σ|)⌋)
#[test]
fn f2_overflow_of_the_bonus_term() {
    let mut r = base("4");
    r.sim.general.elevation_increment = Some(1);
    r.sim.general.elevation_increment_bonus = Some(d("200"));
    r.sim.general.elevation_bonus_cap = Some(d("200"));
    line(&mut r).subject_to_elevation = Some(true);
    // E₂ = 51200; σ = w(51200² + 104²) = 2621450816 − 2³² < 0.
    let sigma = (51200i64 * 51200 + 104 * 104) as i32;
    assert!(sigma < 0);
    let e1 = root_floor(u64::from(sigma.unsigned_abs()), true, RZ) as i32;
    let mut s = setup(&r, 3, 0);
    set_level(&mut s.w, cell_of(0, 0), 1);
    ground(&mut s.w, s.a);
    // A target exactly at the straight limit 1024 + E₁ (spatial, with the 104 drop) passes, one
    // lepton farther does not.
    let a = s.w.obj(s.a).unwrap().pos;
    let lim = i64::from(1024 + e1);
    let dist = |dx: i64| root_floor((dx * dx + 104 * 104) as u64, true, RZ) as i64;
    let mut dx = lim - 5;
    while dist(dx + 1) <= lim {
        dx += 1;
    }
    put(&mut s.w, s.t, Pos::new(a.x + dx as i32, a.y, 0));
    assert!(test(&s));
    put(&mut s.w, s.t, Pos::new(a.x + dx as i32 + 1, a.y, 0));
    assert!(!test(&s));
}

/// A high bridge along the map y axis through column dx.
fn bridge(w: &mut World, dx: i32) {
    for dy in -6..=6 {
        set_deck(w, cell_of(dx, dy));
    }
}

// spec: S-0307 claim 6 and Edge cases (quirks.elevation_deck_cell_level on: a firer under a bridge counts 4 levels higher; scenario RG3 run d)
#[test]
fn quirk_elevation_deck_cell_level_on() {
    let r = elevation_rules(true, false);
    let mut s = setup(&r, 5, 0);
    bridge(&mut s.w, 0);
    assert!(test(&s));
    // Control: the firer on open ground at the same distance.
    let s2 = setup(&r, 5, 0);
    assert!(!test(&s2));
}

// spec: S-0307 Edge cases (quirks.elevation_deck_cell_level off: the 4 levels only for an object on the deck)
#[test]
fn quirk_elevation_deck_cell_level_off() {
    let mut r = elevation_rules(true, false);
    r.globals.quirks.elevation_deck_cell_level = false;
    let mut s = setup(&r, 5, 0);
    bridge(&mut s.w, 0);
    // Under the bridge: no lift.
    assert!(!test(&s));
    // On the deck: lifted. The firer stands on the deck, 416 up; the target is measured on the ground.
    let p = s.w.obj(s.a).unwrap().pos;
    s.w.obj_mut(s.a).unwrap().layer = Layer::Deck;
    put(&mut s.w, s.a, Pos::new(p.x, p.y, 416));
    assert!(test(&s));
}

// spec: S-0307 claim 6 (a target under a bridge counts 4 levels higher and reduces the firer's bonus)
#[test]
fn claim6_target_under_a_bridge_reduces_the_bonus() {
    let r = elevation_rules(true, false);
    let mut s = plateau(&r, 3, 0);
    assert!(test(&s));
    // Target cell marked as a high-bridge cell: λ_T = 4 = λ_A, no bonus; it is also measured at deck height.
    set_deck(&mut s.w, cell_of(3, 0));
    let m = measured_point(&s.w, Target::Object(s.t));
    assert_eq!(m.z, 416);
    // Spatial distance 768 at equal heights, within 1024 even without a bonus.
    assert!(test(&s));
    // At 5 cells (1280) with no bonus: out.
    let mut s5 = plateau(&r, 5, 0);
    set_deck(&mut s5.w, cell_of(5, 0));
    assert!(!test(&s5));
}

// spec: S-0307 claim 7 (a floor-bound target is measured at the terrain height at M_T, plus 416 in a high-bridge cell; quirks.range_target_on_deck)
#[test]
fn claim7_measured_point_of_floor_bound_targets() {
    let r = base("2");
    let mut s = setup(&r, 1, 0);
    // A target standing 150 leptons above flat ground is floor-bound: measured at the surface.
    let p = s.w.obj(s.t).unwrap().pos;
    put(&mut s.w, s.t, Pos::new(p.x, p.y, 150));
    assert_eq!(measured_point(&s.w, Target::Object(s.t)).z, 0);
    // Airborne (208 above): measured as it is.
    put(&mut s.w, s.t, Pos::new(p.x, p.y, 208));
    assert_eq!(measured_point(&s.w, Target::Object(s.t)).z, 208);
    // On a 2-level cell: the terrain height.
    put(&mut s.w, s.t, Pos::new(p.x, p.y, 0));
    set_level(&mut s.w, cell_of(1, 0), 2);
    assert_eq!(measured_point(&s.w, Target::Object(s.t)).z, 208);
}

// spec: S-0307 claim 7 (scenario RG4 runs a and b: a target under the bridge is measured at deck height)
#[test]
fn claim7_rg4_target_under_a_bridge() {
    let r = base("2");
    // 1 cell across the bridge: ⌊r(256² + 416²)⌋ = 488 ≤ 512.
    let mut s = setup(&r, 1, 0);
    bridge(&mut s.w, 1);
    assert!(test(&s));
    // 2 cells: ⌊r(512² + 416²)⌋ = 659 > 512.
    let mut s = setup(&r, 2, 0);
    bridge(&mut s.w, 2);
    assert!(!test(&s));
    // Run b: the same distance on open ground.
    assert!(test(&setup(&r, 2, 0)));
}

// spec: S-0307 Edge cases (quirks.range_target_on_deck off: a target under the bridge is measured at the terrain)
#[test]
fn quirk_range_target_on_deck_off() {
    let mut r = base("2");
    r.globals.quirks.range_target_on_deck = false;
    let mut s = setup(&r, 2, 0);
    bridge(&mut s.w, 2);
    assert_eq!(measured_point(&s.w, Target::Object(s.t)).z, 0);
    assert!(test(&s));
    // On the deck it is measured at the deck.
    let p = s.w.obj(s.t).unwrap().pos;
    s.w.obj_mut(s.t).unwrap().layer = Layer::Deck;
    put(&mut s.w, s.t, Pos::new(p.x, p.y, 416));
    assert_eq!(measured_point(&s.w, Target::Object(s.t)).z, 416);
}

// spec: S-0307 claim 8 (the source point is the firer's reference point, not the fire point)
#[test]
fn claim8_source_point_is_the_reference_point() {
    let s = setup(&base("4"), 3, 0);
    let sp = source_point(&s.w, s.a, s.wp, Target::Object(s.t));
    assert_eq!(sp, s.w.reference_point(s.a));
}

// spec: S-0307 claim 8 (a) (cell rangefinding: the cell centre at the terrain height there; scenario RG5)
#[test]
fn claim8a_cell_rangefinding() {
    for (cell_rf, fires) in [(true, true), (false, false)] {
        let mut r = base("2");
        weapon(&mut r).cell_rangefinding = Some(cell_rf);
        let mut s = setup(&r, 2, 0);
        // The firer stands 100 leptons west of its cell centre; the target 512 east of the centre.
        let a = s.w.obj(s.a).unwrap().pos;
        put(&mut s.w, s.a, Pos::new(a.x - 100, a.y, 0));
        assert_eq!(test(&s), fires, "cell rangefinding {cell_rf}");
        if cell_rf {
            let sp = source_point(&s.w, s.a, s.wp, Target::Object(s.t));
            assert_eq!((sp.x, sp.y, sp.z), (a.x, a.y, 0));
        }
    }
}

// spec: S-0307 claim 8 (a) (cell rangefinding on a bridge deck: plus 416)
#[test]
fn claim8a_cell_rangefinding_on_a_deck() {
    let mut r = base("2");
    weapon(&mut r).cell_rangefinding = Some(true);
    let mut s = setup(&r, 2, 0);
    bridge(&mut s.w, 0);
    let a = s.w.obj(s.a).unwrap().pos;
    s.w.obj_mut(s.a).unwrap().layer = Layer::Deck;
    put(&mut s.w, s.a, Pos::new(a.x + 30, a.y, 416));
    let sp = source_point(&s.w, s.a, s.wp, Target::Object(s.t));
    assert_eq!((sp.x, sp.y, sp.z), (a.x, a.y, 416));
}

// spec: S-0307 claim 8 (b) (an airborne firer measures at the target's reference height; Δz = −416 under a bridge; scenario RG2 run f)
#[test]
fn claim8b_airborne_firer_and_a_target_under_a_bridge() {
    // Planar distance 768; the window p + 20 ≤ Q ≤ ⌊r(p² + 416²)⌋ − 20: Q = 3.2 cells = 819.
    let r = base("3.2");
    let mut s = setup(&r, 3, 0);
    let a = s.w.obj(s.a).unwrap().pos;
    put(&mut s.w, s.a, Pos::new(a.x, a.y, 600));
    // Control: the target on open ground, Δz = 0.
    assert!(test(&s));
    let sp = source_point(&s.w, s.a, s.wp, Target::Object(s.t));
    assert_eq!(sp.z, 0);
    // Under a bridge: the measured point is at the deck, Δz = −416.
    bridge(&mut s.w, 3);
    assert!(!test(&s));
}

// spec: S-0307 claim 8 (b) (against a target that is not floor-bound an airborne firer measures no height difference)
#[test]
fn claim8b_airborne_firer_against_an_airborne_target() {
    let r = base("3");
    let mut s = setup(&r, 3, 0);
    let a = s.w.obj(s.a).unwrap().pos;
    let t = s.w.obj(s.t).unwrap().pos;
    put(&mut s.w, s.a, Pos::new(a.x, a.y, 2000));
    put(&mut s.w, s.t, Pos::new(t.x, t.y, 300));
    let sp = source_point(&s.w, s.a, s.wp, Target::Object(s.t));
    assert_eq!(sp.z, 300);
    assert!(test(&s));
}

// spec: S-0307 claim 8 (a) and (b) together (horizontal position of (a), height of (b))
#[test]
fn claim8_both_substitutions() {
    let mut r = base("3");
    weapon(&mut r).cell_rangefinding = Some(true);
    let mut s = setup(&r, 3, 0);
    let a = s.w.obj(s.a).unwrap().pos;
    let t = s.w.obj(s.t).unwrap().pos;
    put(&mut s.w, s.a, Pos::new(a.x + 50, a.y - 40, 900));
    put(&mut s.w, s.t, Pos::new(t.x, t.y, 120));
    let sp = source_point(&s.w, s.a, s.wp, Target::Object(s.t));
    assert_eq!(sp, Pos::new(a.x, a.y, 120));
}

// spec: S-0307 claim 9 (not arcing: ⌊r(Δx² + Δy² + Δz²)⌋ ≤ R + E₁ + β, β = 64 (W + H) for a building target)
#[test]
fn claim9_building_lengthening() {
    let r = base("4");
    let mut s = setup(&r, 3, 0);
    // A 1 × 1 tower: β = 128. Placed so that its reference point is 1024 + 128 leptons away.
    let tw = testkit::spawn(&mut s.w, Kind::Building, "TOWER", 1, cell_of(-6, 4), 0);
    let a = s.w.obj(s.a).unwrap().pos;
    let x = first_beyond(1152);
    put(&mut s.w, tw, Pos::new(a.x + x - 1, a.y, 0));
    assert!(in_range(
        &s.w,
        s.a,
        Some(s.wp),
        Some(Target::Object(tw)),
        None
    ));
    put(&mut s.w, tw, Pos::new(a.x + x, a.y, 0));
    assert!(!in_range(
        &s.w,
        s.a,
        Some(s.wp),
        Some(Target::Object(tw)),
        None
    ));
    // A vehicle at 1152 is out (no lengthening).
    put(&mut s.w, s.t, Pos::new(a.x + 1152, a.y, 0));
    assert!(!test(&s));
}

// spec: S-0307 claim 9 (arcing: planar distance, no building lengthening)
#[test]
fn claim9_arcing_uses_the_planar_distance_without_lengthening() {
    for arcing in [true, false] {
        let mut r = base("4");
        line(&mut r).arcing = Some(arcing);
        // The firer on a 4-level plateau, the target 1024 away in plan on level 0: spatial
        // distance ⌊r(1024² + 416²)⌋ > 1024.
        let mut s = plateau(&r, 4, 0);
        assert_eq!(test(&s), arcing, "arcing {arcing}");
        // A 1 × 1 tower just beyond 1024 in plan on the plateau's level: β applies only when not
        // arcing.
        let tw = testkit::spawn(&mut s.w, Kind::Building, "TOWER", 1, cell_of(-6, 4), 0);
        let a = s.w.obj(s.a).unwrap().pos;
        let x = first_beyond(1024);
        set_level(&mut s.w, cell_of(4, 0), 4);
        let tz = s.w.surface_at(a.x + x, a.y);
        put(&mut s.w, tw, Pos::new(a.x + x, a.y, tz));
        assert_eq!(
            in_range(&s.w, s.a, Some(s.wp), Some(Target::Object(tw)), None),
            !arcing,
            "tower, arcing {arcing}"
        );
    }
}

// spec: S-0307 claim 9 (minimum range in both branches; scenario RG6)
#[test]
fn claim9_rg6_minimum_range() {
    for arcing in [true, false] {
        let mut r = base("6");
        weapon(&mut r).minimum_range = Some(d("2"));
        line(&mut r).arcing = Some(arcing);
        assert!(!test(&setup(&r, 1, 0)), "arcing {arcing}: 1 cell");
        assert!(test(&setup(&r, 3, 0)), "arcing {arcing}: 3 cells");
        // Exactly the minimum range is not inside it.
        assert!(test(&setup(&r, 2, 0)), "arcing {arcing}: 2 cells");
    }
}

// spec: S-0307 claim 9 and S-0306 F3 (scenario RG7: v(512, 10) = 78, reachable at d = 256 up to 250 leptons)
#[test]
fn claim9_rg7_reachability() {
    assert_eq!(ballistics::arc_speed(512, 10, false, RZ), 78);
    let g = ballistics::gravity(10, false, RZ);
    assert!(ballistics::reachable(78, 256, 250, g, true, RZ));
    assert!(!ballistics::reachable(78, 256, 251, g, true, RZ));
    let mut r = base("2");
    r.sim.general.gravity = Some(10);
    line(&mut r).arcing = Some(true);
    for (levels, fires) in [(2, true), (3, false)] {
        let mut s = setup(&r, 1, 0);
        set_level(&mut s.w, cell_of(1, 0), levels);
        ground(&mut s.w, s.t);
        assert_eq!(test(&s), fires, "{levels} levels higher");
    }
}

// spec: S-0307 claim 10 (not arcing: a source under a deck cannot reach a target at or above that deck; scenario RG4 run c)
#[test]
fn claim10_rg4_firer_under_a_bridge() {
    let r = base("2");
    let mut s = setup(&r, 0, 1);
    bridge(&mut s.w, 0);
    let t = s.w.obj(s.t).unwrap().pos;
    s.w.obj_mut(s.t).unwrap().layer = Layer::Deck;
    put(&mut s.w, s.t, Pos::new(t.x, t.y, 416));
    // 488 ≤ 512, but refused.
    assert!(!test(&s));
    // The same firer on the deck reaches it.
    let a = s.w.obj(s.a).unwrap().pos;
    s.w.obj_mut(s.a).unwrap().layer = Layer::Deck;
    put(&mut s.w, s.a, Pos::new(a.x, a.y, 416));
    assert!(test(&s));
}

// spec: S-0307 claim 10 (arcing: out when M_T's cell carries a high bridge and M_T.z − S.z ≥ 312; scenario RG4 run d)
#[test]
fn claim10_rg4_arcing_deck_refusal() {
    let mut r = base("4");
    r.sim.general.gravity = Some(10);
    line(&mut r).arcing = Some(true);
    // Q = 1024: reachable for 416 at d = 256.
    let v = ballistics::arc_speed(1024, 10, false, RZ);
    assert!(ballistics::reachable(
        v,
        256,
        416,
        ballistics::gravity(10, false, RZ),
        true,
        RZ
    ));
    let deck_target = |w: &mut World, t: ObjectId| {
        let p = w.obj(t).unwrap().pos;
        w.obj_mut(t).unwrap().layer = Layer::Deck;
        put(w, t, Pos::new(p.x, p.y, 416));
    };
    let mut s = setup(&r, 1, 0);
    bridge(&mut s.w, 1);
    deck_target(&mut s.w, s.t);
    assert!(!test(&s));
    // Control (i): the firer 2 levels up, 208 below the deck.
    let mut s = setup(&r, 1, 0);
    bridge(&mut s.w, 1);
    deck_target(&mut s.w, s.t);
    set_level(&mut s.w, cell_of(0, 0), 2);
    ground(&mut s.w, s.a);
    assert!(test(&s));
    // Control (ii): no bridge, the target on a cliff top 4 levels higher.
    let mut s = setup(&r, 1, 0);
    set_level(&mut s.w, cell_of(1, 0), 4);
    ground(&mut s.w, s.t);
    assert!(test(&s));
}

/// Scenario RG8's map: a wall overlay of index `o` in the cell between the
/// firer and a target 3 cells away; range 4.
fn rg8(o: usize, wall_warhead: bool) -> bool {
    let mut r = base("4");
    overlays(&mut r, &[2, 26, 243, 5]);
    line(&mut r).subject_to_walls = Some(true);
    if wall_warhead {
        weapon(&mut r).warhead = Some("WALLER".into());
    }
    let mut s = setup(&r, 3, 0);
    set_overlay(&mut s.w, cell_of(1, 0), o as u8);
    test(&s)
}

// spec: S-0307 claim 11 (scenario RG8 run a: a wall blocks a warhead that is not wall-damaging)
#[test]
fn claim11_rg8_wall_blocks() {
    for o in [2, 26, 243, 5] {
        assert!(!rg8(o, false), "overlay {o}");
    }
}

// spec: S-0307 claim 11 (scenario RG8 run b: a wall-damaging warhead passes the overlays 2, 26 and 243)
#[test]
fn claim11_rg8_wall_damaging_warhead_passes_the_three_walls() {
    for o in [2, 26, 243] {
        assert!(rg8(o, true), "overlay {o}");
    }
}

// spec: S-0307 claim 11 (scenario RG8 run c: a wall overlay at any other index refuses even a wall-damaging warhead)
#[test]
fn claim11_rg8_other_wall_index_refuses() {
    assert!(!rg8(5, true));
}

// spec: S-0307 claim 11 (a cliff refuses even a wall-damaging warhead)
#[test]
fn claim11_cliff_refuses_a_wall_damaging_warhead() {
    let mut r = base("4");
    line(&mut r).subject_to_cliffs = Some(true);
    weapon(&mut r).warhead = Some("WALLER".into());
    let mut s = setup(&r, 3, 0);
    set_level(&mut s.w, cell_of(1, 0), 4);
    assert!(!test(&s));
    // A ridge of 3 levels does not block.
    set_level(&mut s.w, cell_of(1, 0), 3);
    assert!(test(&s));
}

// spec: S-0307 claim 11 (projectiles subject to neither cliffs nor walls always pass)
#[test]
fn claim11_neither_cliffs_nor_walls_always_pass() {
    let mut r = base("4");
    overlays(&mut r, &[2]);
    let mut s = setup(&r, 3, 0);
    set_overlay(&mut s.w, cell_of(1, 0), 2);
    set_level(&mut s.w, cell_of(2, 0), 8);
    assert!(test(&s));
}

// spec: S-0307 claim 12 (the approach-position search passes a candidate source point, without claim 8's substitutions)
#[test]
fn claim12_candidate_source_point() {
    let mut r = base("2");
    weapon(&mut r).cell_rangefinding = Some(true);
    let mut s = setup(&r, 2, 0);
    let a = s.w.obj(s.a).unwrap().pos;
    // The firer airborne and off its cell centre: with the firing check's source point it is in range.
    put(&mut s.w, s.a, Pos::new(a.x - 100, a.y, 600));
    assert!(test(&s));
    // A candidate at the same off-centre ground position: no cell centre, no height swap.
    let cand = Pos::new(a.x - 100, a.y, 0);
    assert!(!in_range(
        &s.w,
        s.a,
        Some(s.wp),
        Some(Target::Object(s.t)),
        Some(cand)
    ));
    let cand = Pos::new(a.x, a.y, 0);
    assert!(in_range(
        &s.w,
        s.a,
        Some(s.wp),
        Some(Target::Object(s.t)),
        Some(cand)
    ));
}

// spec: S-0307 claim 12 (the firing check uses the range test: out of range answers OutOfRange)
#[test]
fn claim12_firing_check_uses_the_range_test() {
    use glasnost_sim::combat::check::{Answer, check};
    let s = setup(&base("2"), 5, 0);
    assert_eq!(
        check(&s.w, s.a, Some(Target::Object(s.t)), 0, true),
        Answer::OutOfRange
    );
    assert_ne!(
        check(&s.w, s.a, Some(Target::Object(s.t)), 0, false),
        Answer::OutOfRange
    );
}

// spec: S-0307 claim 13 (ρ(A, i): the weapon's range field in leptons with no bonus; 0 without a weapon in the slot)
#[test]
fn claim13_firer_weapon_range() {
    let mut r = base("6");
    r.sim
        .vehicles
        .get_mut("CART")
        .unwrap()
        .combat
        .air_range_bonus = Some(d("2"));
    r.sim.general.elevation_increment = Some(1);
    let s = setup(&r, 5, 0);
    assert_eq!(firer_weapon_range(&s.w, s.a, 0), 1536);
    assert_eq!(firer_weapon_range(&s.w, s.a, 1), 0);
    // The unarmed buggy has no weapon.
    assert_eq!(firer_weapon_range(&s.w, s.t, 0), 0);
}

// spec: S-0307 claim 14 (no target: in range; a target and no weapon: out of range)
#[test]
fn claim14_no_target_no_weapon() {
    let s = setup(&base("-2"), 1, 0);
    assert!(in_range(&s.w, s.a, Some(s.wp), None, None));
    assert!(in_range(&s.w, s.a, None, None, None));
    assert!(!in_range(&s.w, s.a, None, Some(Target::Object(s.t)), None));
}

// spec: S-0307 Edge cases (negative range fields other than −2: out of range unless other terms reach the distance)
#[test]
fn edge_negative_range_field() {
    let mut r = base("-1.5");
    r.sim
        .vehicles
        .get_mut("CART")
        .unwrap()
        .combat
        .air_range_bonus = Some(d("3"));
    let mut s = setup(&r, 1, 0);
    assert!(!test(&s));
    // R = −384 + 768 = 384 for an airborne target 250 up at 200 in plan: ⌊r(200² + 250²)⌋ = 320.
    let a = s.w.obj(s.a).unwrap().pos;
    put(&mut s.w, s.t, Pos::new(a.x + 200, a.y, 250));
    assert!(test(&s));
}

// ---------------------------------------------------------------------------
// Claims 15 to 18 and F4 (revision 2): the approach-position search.
// ---------------------------------------------------------------------------

use glasnost_sim::combat::range::{
    Approach, approach_radii, approach_search, balloon_hover_approach,
};

// spec: S-0307 F4 (ρ′ = ρ − 128 for ρ ≥ 512 is the caller's; u = max(0, trunc(ρ′ + β − a)) with a = 281.6 for infantry and 179.2 for vehicles; u′ = 256 for 0 < u ≤ 256; radii step down by 256 while above 204.8)
#[test]
fn f4_approach_radii() {
    let r = base("5");
    let s = setup(&r, 1, 0);
    let w = &s.w;
    // ρ = 1280: ρ′ = 1152; vehicle u = trunc(972.8) = 972, infantry trunc(870.4) = 870.
    assert_eq!(approach_radii(w, 1152, 0, false), vec![972, 716, 460]);
    assert_eq!(approach_radii(w, 1152, 0, true), vec![870, 614, 358]);
    // u′ = 256 for 0 < u ≤ 256: 400 − 179.2 = 220.8 and 482 − 281.6 = 200.4.
    assert_eq!(approach_radii(w, 400, 0, false), vec![256]);
    assert_eq!(approach_radii(w, 482, 0, true), vec![256]);
    // u = 0 (no radius at all) and the 204.8 cut: 435 − 179.2 = 255.8 → 256; 692 − 179.2 = 512.8 → 512, 256.
    assert_eq!(approach_radii(w, 100, 0, false), Vec::<i32>::new());
    assert_eq!(approach_radii(w, 692, 0, false), vec![512, 256]);
    // β, the building lengthening, adds to ρ′.
    assert_eq!(
        approach_radii(w, 1152, 256, false),
        vec![1228, 972, 716, 460]
    );
}

/// The cart (house 0) at the middle and the buggy (house 1) ten cells east,
/// the cart's target, with the regions up to date.
fn approach_setup(r: &Ruleset) -> S {
    let mut s = setup(r, 10, 0);
    s.w.obj_mut(s.a).unwrap().target = Some(Target::Object(s.t));
    glasnost_sim::regions::ensure(&mut s.w);
    s
}

/// A blocking vehicle of house 0 in cell (dx, dy) from the middle.
fn block(s: &mut S, dx: i32, dy: i32) {
    testkit::spawn(&mut s.w, Kind::Vehicle, "BUGGY", 0, cell_of(dx, dy), 0);
    glasnost_sim::regions::ensure(&mut s.w);
}

// spec: S-0307 claim 15 and F4 (open ground: the outer radius first, around the target, towards the attacker: radius 972 straight west of the target gives the cell four cells west of it, which passes the range test from its centre at height 0; scenario RG10)
#[test]
fn claim_15_open_ground_choice() {
    let r = base("5");
    let s = approach_setup(&r);
    assert_eq!(
        approach_search(&s.w, s.a, 1152),
        Some(Approach::Cell(cell_of(6, 0)))
    );
}

// spec: S-0307 claim 16 (a vehicle whose primary cell is not clear takes the first alternative that passes the range and clear-cell tests: (0, −1) and then (1, −2) with quirks.approach_alternative_walk on; scenario RG11)
#[test]
fn quirk_approach_alternative_walk_on() {
    let r = base("5");
    let mut s = approach_setup(&r);
    block(&mut s, 6, 0);
    assert_eq!(
        approach_search(&s.w, s.a, 1152),
        Some(Approach::Cell(cell_of(6, -1)))
    );
    block(&mut s, 6, -1);
    assert_eq!(
        approach_search(&s.w, s.a, 1152),
        Some(Approach::Cell(cell_of(7, -2)))
    );
}

// spec: S-0307 Edge cases (quirks.approach_alternative_walk off: the eight neighbours in step-direction order 0 to 7, so after (0, −1) comes (1, −1))
#[test]
fn quirk_approach_alternative_walk_off() {
    let mut r = base("5");
    r.globals.quirks.approach_alternative_walk = false;
    let mut s = approach_setup(&r);
    block(&mut s, 6, 0);
    block(&mut s, 6, -1);
    assert_eq!(
        approach_search(&s.w, s.a, 1152),
        Some(Approach::Cell(cell_of(7, -1)))
    );
}

// spec: S-0307 claim 16 (an infantry unit skips a primary cell that is not clear and takes a later primary, never an alternative)
#[test]
fn claim_16_infantry_skips_an_obstructed_primary() {
    let mut r = base("5");
    r.sim.infantry.get_mut("WALKER").unwrap().primary = Some("Probe".into());
    let mut w = testkit::game_with(&r, N, 1);
    let a = testkit::spawn(&mut w, Kind::Infantry, "WALKER", 0, cell_of(0, 0), 0);
    let t = testkit::spawn(&mut w, Kind::Vehicle, "BUGGY", 1, cell_of(10, 0), 0);
    w.obj_mut(a).unwrap().target = Some(Target::Object(t));
    glasnost_sim::regions::ensure(&mut w);
    // Infantry: radius 870 west of the target's centre is cell (7, 0).
    assert_eq!(
        approach_search(&w, a, 1152),
        Some(Approach::Cell(cell_of(7, 0)))
    );
    // Blocked: δ = ±1 … ±6 stay in that cell; δ = +8 (170 leptons north) is the next primary, (7, −1).
    testkit::spawn(&mut w, Kind::Vehicle, "BUGGY", 0, cell_of(7, 0), 0);
    glasnost_sim::regions::ensure(&mut w);
    assert_eq!(
        approach_search(&w, a, 1152),
        Some(Approach::Cell(cell_of(7, -1)))
    );
    // Both blocked: δ = −8 gives (7, 1), where a vehicle would have taken the alternative (1, −2).
    testkit::spawn(&mut w, Kind::Vehicle, "BUGGY", 0, cell_of(7, -1), 0);
    glasnost_sim::regions::ensure(&mut w);
    assert_eq!(
        approach_search(&w, a, 1152),
        Some(Approach::Cell(cell_of(7, 1)))
    );
}

// spec: S-0307 claim 17 (no position found: an ordinary actor drops its target and gets the nearby-cell search's cell around the target's reference cell, the radii being empty; scenario RG12)
#[test]
fn claim_17_failed_search_drops_the_target() {
    let r = base("5");
    let s = approach_setup(&r);
    let got = approach_search(&s.w, s.a, 100);
    let Some(Approach::Drop(Some(c))) = got else {
        panic!("{got:?}")
    };
    // The nearby-cell search starts at the target's cell, which holds the target: a cell next to it.
    let t = cell_of(10, 0);
    assert!((c.x - t.x).abs().max((c.y - t.y).abs()) <= 1, "{c:?}");
}

// spec: S-0307 claim 17 (a hunter-seeker type takes the target object as its destination and keeps the target)
#[test]
fn claim_17_hunter_seeker_keeps_the_target() {
    let mut r = base("5");
    r.sim.vehicles.get_mut("CART").unwrap().hunter_seeker = Some(true);
    let s = approach_setup(&r);
    assert_eq!(
        approach_search(&s.w, s.a, 100),
        Some(Approach::TargetItself)
    );
}

// spec: S-0307 claim 18 (a balloon-hover vehicle whose weapon for the target has a vertical projectile takes the target object as its destination; without the vertical flag, or without the property, it does not; scenario RG13)
#[test]
fn claim_18_balloon_hover_vertical() {
    for (hover, vertical, want) in [
        (true, true, true),
        (true, false, false),
        (false, true, false),
    ] {
        let mut r = base("5");
        r.sim.vehicles.get_mut("CART").unwrap().balloon_hover = Some(hover);
        line(&mut r).vertical = Some(vertical);
        let s = approach_setup(&r);
        assert_eq!(balloon_hover_approach(&s.w, s.a), want);
    }
}

// spec: S-0307 claim 15 and S-0215 claim 37 (f) (an actor that keeps a target out of range and has no destination searches, and the cell found becomes its destination with the target kept)
#[test]
fn claim_15_approach_step_moves_towards_the_target() {
    let r = base("5");
    let mut s = approach_setup(&r);
    s.w.obj_mut(s.a).unwrap().beh.current = Some(glasnost_sim::rules::Behaviour::Attack);
    glasnost_sim::approach::step(&mut s.w, s.a);
    let o = s.w.obj(s.a).unwrap();
    assert_eq!(o.dest, Some(Target::Cell(cell_of(6, 0))));
    assert_eq!(o.target, Some(Target::Object(s.t)));
}

// spec: S-0307 claim 18 and S-0215 claim 37 (h) (the balloon-hover vehicle takes the target object as its destination, even with a destination held and without searching)
#[test]
fn claim_18_approach_step_takes_the_target() {
    let mut r = base("5");
    r.sim.vehicles.get_mut("CART").unwrap().balloon_hover = Some(true);
    line(&mut r).vertical = Some(true);
    let mut s = approach_setup(&r);
    let o = s.w.obj_mut(s.a).unwrap();
    o.beh.current = Some(glasnost_sim::rules::Behaviour::Attack);
    o.dest = Some(Target::Cell(cell_of(1, 1)));
    glasnost_sim::approach::step(&mut s.w, s.a);
    let o = s.w.obj(s.a).unwrap();
    assert_eq!(o.dest, Some(Target::Object(s.t)));
    assert_eq!(o.target, Some(Target::Object(s.t)));
}
