//! Claim tests for area damage (S-0303) on synthetic data: invented types,
//! warheads and overlays, every number our own. Expected values come from
//! the spec's formulas and worked examples.

use glasnost_map::{CellXY, NO_OVERLAY};
use glasnost_num::geom::root_floor;
use glasnost_num::{F64, Rounding};
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{BuildingType, Decimal, OverlayType, UnitType, WarheadType, WeaponType};
use glasnost_sim::combat::area::{self, Detonation, DetonationResult};
use glasnost_sim::combat::damage;
use glasnost_sim::combat::data::{WarheadId, WeaponId};
use glasnost_sim::combat::effects::Effect;
use glasnost_sim::rules::Kind;
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, Layer, ObjectId, Pos, World};

const SIZE: i32 = 30;
const RZ: Rounding = Rounding::TowardZero;
const ALPHA: u8 = 0;
const BETA: u8 = 1;

fn dec(s: &str) -> Decimal {
    Decimal(s.into())
}

fn every(v: &str) -> Option<Vec<Decimal>> {
    Some(vec![dec(v); 11])
}

fn register(r: &mut Ruleset, reg: &str, name: &str) {
    let l = r.registries.entry(reg.into()).or_default();
    if !l.iter().any(|n| n == name) {
        l.push(name.into());
    }
}

/// An unarmed test vehicle type of strength 5000, class *none*.
fn vehicle(r: &mut Ruleset, name: &str, f: impl FnOnce(&mut UnitType)) {
    let mut t = r.sim.vehicles["BUGGY"].clone();
    t.strength = Some(5000);
    t.primary = None;
    t.combat.armor = None;
    f(&mut t);
    register(r, "vehicles", name);
    r.sim.vehicles.insert(name.into(), t);
}

fn infantry(r: &mut Ruleset, name: &str, f: impl FnOnce(&mut UnitType)) {
    let mut t = r.sim.infantry["WALKER"].clone();
    t.strength = Some(5000);
    t.primary = None;
    f(&mut t);
    register(r, "infantry", name);
    r.sim.infantry.insert(name.into(), t);
}

fn building(r: &mut Ruleset, name: &str, f: impl FnOnce(&mut BuildingType)) {
    let mut t = BuildingType {
        strength: Some(5000),
        foundation: Some([1, 1]),
        ..Default::default()
    };
    f(&mut t);
    register(r, "buildings", name);
    r.sim.buildings.insert(name.into(), t);
}

fn warhead(r: &mut Ruleset, name: &str, f: impl FnOnce(&mut WarheadType)) {
    let mut h = WarheadType {
        verses: every("1"),
        ..Default::default()
    };
    f(&mut h);
    r.sim.warheads.insert(name.into(), h);
}

fn weapon(r: &mut Ruleset, name: &str, f: impl FnOnce(&mut WeaponType)) {
    let mut t = WeaponType {
        projectile: Some("Shell".into()),
        ..Default::default()
    };
    f(&mut t);
    r.sim.weapons.insert(name.into(), t);
}

fn overlay(r: &mut Ruleset, name: &str, f: impl FnOnce(&mut OverlayType)) {
    let mut o = OverlayType {
        has_art: Some(true),
        ..Default::default()
    };
    f(&mut o);
    register(r, "overlays", name);
    r.sim.overlays.insert(name.into(), o);
}

/// A warhead whose damage falls linearly to 0 at the edge of a spread of
/// `spread` cells, so that a hit of D at distance d leaves D (R − d) / R.
fn linear(r: &mut Ruleset, name: &str, spread: &str) {
    warhead(r, name, |h| {
        h.cell_spread = Some(dec(spread));
        h.percent_at_max = Some(dec("0"));
    });
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

fn ov(w: &World, name: &str) -> u8 {
    w.rules()
        .overlays
        .iter()
        .position(|o| o.name == name)
        .unwrap_or_else(|| panic!("no overlay {name}")) as u8
}

fn at(dx: i32, dy: i32) -> CellXY {
    testkit::mid(SIZE, dx, dy)
}

fn put(w: &mut World, kind: Kind, name: &str, house: u8, dx: i32, dy: i32) -> ObjectId {
    testkit::spawn(w, kind, name, house, at(dx, dy), 0)
}

fn centre(dx: i32, dy: i32) -> Pos {
    Pos::centre(at(dx, dy), 0)
}

fn health(w: &World, id: ObjectId) -> i32 {
    w.obj(id).map_or(0, |o| o.health)
}

fn lost(w: &World, id: ObjectId) -> i32 {
    damage::strength(w, id) - health(w, id)
}

/// Runs a detonation, its arguments evaluated first (they may read the world).
macro_rules! boom {
    ($w:expr, $p:expr, $d:expr, $h:expr, $s:expr, $hs:expr $(,)?) => {{
        let (p, d, h, s, hs) = ($p, $d, $h, $s, $hs);
        boom_fn(&mut $w, p, d, h, s, hs)
    }};
}

fn boom_fn(
    w: &mut World,
    point: Pos,
    damage: i32,
    warhead: Option<WarheadId>,
    source: Option<ObjectId>,
    house: Option<HouseId>,
) -> DetonationResult {
    area::detonate(
        w,
        Detonation {
            point,
            damage,
            warhead,
            source,
            house,
            resources: true,
        },
    )
}

/// ⌊r(n)⌋ for a displacement, as S-0204 gives it.
fn dist(w: &World, dx: i64, dy: i64, dz: i64) -> i32 {
    let r = w.rules();
    root_floor(
        (dx * dx + dy * dy + dz * dz) as u64,
        r.quirks.table_root_distance,
        r.mode,
    ) as i32
}

fn set_overlay(w: &mut World, c: CellXY, o: u8, data: u8) {
    let cell = w.cells.grid.get_mut(c).unwrap();
    cell.overlay = o;
    cell.overlay_data = data;
}

fn overlay_at(w: &World, c: CellXY) -> (u8, u8) {
    w.cells
        .cell(c)
        .map_or((NO_OVERLAY, 0), |c| (c.overlay, c.overlay_data))
}

fn anims(w: &World) -> Vec<(String, Pos)> {
    w.effects
        .iter()
        .filter_map(|e| match e {
            Effect::Anim { name, pos } => Some((name.clone(), *pos)),
            _ => None,
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Claims 1, 5 and 11: one detonation, all at once, in order.
// ---------------------------------------------------------------------------

/// A detonation of house Alpha over two Beta vehicles of strength 10: one
/// (BANG) explodes with a death weapon that reaches the other (QUIET).
/// Returns the kills of (Alpha, Beta).
fn nested(bang_first: bool) -> (i32, i32) {
    let mut r = testkit::combat_ruleset();
    warhead(&mut r, "BLAST", |h| h.cell_spread = Some(dec("1.5")));
    weapon(&mut r, "Blast", |t| {
        t.damage = Some(500);
        t.warhead = Some("BLAST".into());
    });
    vehicle(&mut r, "BANG", |t| {
        t.strength = Some(10);
        t.combat.explodes = Some(true);
        t.combat.death_weapon = Some("Blast".into());
    });
    vehicle(&mut r, "QUIET", |t| t.strength = Some(10));
    warhead(&mut r, "WIDE", |h| h.cell_spread = Some(dec("1.5")));
    let mut w = testkit::game_with(&r, SIZE, 1);
    // Scan order: the centre cell (ring 0) before (1, 0) of ring 1.
    let (first, second) = if bang_first {
        ("BANG", "QUIET")
    } else {
        ("QUIET", "BANG")
    };
    put(&mut w, Kind::Vehicle, first, BETA, 0, 0);
    put(&mut w, Kind::Vehicle, second, BETA, 1, 0);
    let h = wh(&w, "WIDE");
    let p = centre(0, 0);
    let res = boom!(w, p, 20, Some(h), None, Some(HouseId(ALPHA)));
    assert_eq!(res, DetonationResult::Hit);
    (w.houses[0].combat.kills, w.houses[1].combat.kills)
}

// spec: S-0303 claims 1 and 11 (each hit is resolved completely before the next, a death weapon it sets off runs to its end in the middle of the order, and a later hit on an object that death weapon killed does nothing)
#[test]
fn a_nested_detonation_runs_inside_the_order() {
    // BANG in the centre cell dies first; its death weapon kills QUIET,
    // credited to Beta (the dying object's house); QUIET's own hit does
    // nothing.
    assert_eq!(nested(true), (1, 1));
    // QUIET first: Alpha kills both.
    assert_eq!(nested(false), (2, 0));
}

// spec: S-0303 claim 5 (within one cell layer the objects are hit in the cell's listing order)
#[test]
fn objects_in_one_cell_are_hit_in_listing_order() {
    let mut r = testkit::combat_ruleset();
    warhead(&mut r, "BLAST", |h| h.cell_spread = Some(dec("1")));
    weapon(&mut r, "Blast", |t| {
        t.damage = Some(500);
        t.warhead = Some("BLAST".into());
    });
    infantry(&mut r, "BANG", |t| {
        t.strength = Some(10);
        t.combat.explodes = Some(true);
        t.combat.death_weapon = Some("Blast".into());
    });
    infantry(&mut r, "QUIET", |t| t.strength = Some(10));
    warhead(&mut r, "WIDE", |h| h.cell_spread = Some(dec("1")));
    let mut w = testkit::game_with(&r, SIZE, 1);
    let a = put(&mut w, Kind::Infantry, "BANG", BETA, 0, 0);
    let b = put(&mut w, Kind::Infantry, "QUIET", BETA, 0, 0);
    let list = w.cells.list(at(0, 0), Layer::Ground).to_vec();
    assert!(list.contains(&a) && list.contains(&b));
    let bang_first = list.iter().position(|&x| x == a) < list.iter().position(|&x| x == b);
    let h = wh(&w, "WIDE");
    boom!(w, centre(0, 0), 20, Some(h), None, Some(HouseId(ALPHA)));
    let kills = (w.houses[0].combat.kills, w.houses[1].combat.kills);
    if bang_first {
        assert_eq!(kills, (1, 1));
    } else {
        assert_eq!(kills, (2, 0));
    }
}

// ---------------------------------------------------------------------------
// Claim 2: detonations that do nothing.
// ---------------------------------------------------------------------------

// spec: S-0303 claim 2 (a damage amount of 0 or no warhead: no hit, no wall hit, no draw, result missed)
#[test]
fn a_detonation_of_zero_or_without_a_warhead_does_nothing() {
    let mut r = testkit::combat_ruleset();
    vehicle(&mut r, "BOX", |_| {});
    warhead(&mut r, "WALLER", |h| {
        h.wall = Some(true);
        h.cell_spread = Some(dec("1"));
    });
    overlay(&mut r, "FENCE", |o| {
        o.wall = Some(true);
        o.strength = Some(100);
        o.damage_levels = Some(3);
    });
    let mut w = testkit::game_with(&r, SIZE, 1);
    let v = put(&mut w, Kind::Vehicle, "BOX", BETA, 0, 0);
    let fence = ov(&w, "FENCE");
    set_overlay(&mut w, at(1, 0), fence, 0);
    let h = wh(&w, "WALLER");
    let taken = w.rng.taken();
    assert_eq!(
        boom!(w, centre(0, 0), 0, Some(h), None, None),
        DetonationResult::Missed
    );
    assert_eq!(
        boom!(w, centre(0, 0), 50, None, None, None),
        DetonationResult::Missed
    );
    assert_eq!(lost(&w, v), 0);
    assert_eq!(overlay_at(&w, at(1, 0)), (fence, 0));
    assert_eq!(w.rng.taken(), taken);
}

// ---------------------------------------------------------------------------
// Claim 3, F1 and F2: spread radius and scanned cells.
// ---------------------------------------------------------------------------

// spec: S-0303 F1 and F2 (R = trunc(256 S); k = trunc(RZ₅₃(S + c)): S = 1.0 gives 1, 1.01 gives 1, 1.02 gives 2, 0.01 gives 0, 0.02 gives 1; Edge cases: k clamped to 0 … 11)
#[test]
fn spread_and_scan_radii() {
    let cases = [
        ("1", 256, 1),
        ("1.01", 258, 1),
        ("1.02", 261, 2),
        ("0.01", 2, 0),
        ("0.02", 5, 1),
        ("0", 0, 0),
        ("2.5", 640, 3),
        ("12", 3072, 11),
        ("-3", -768, 0),
    ];
    let mut r = testkit::combat_ruleset();
    for (i, (s, _, _)) in cases.iter().enumerate() {
        warhead(&mut r, &format!("S{i}"), |h| h.cell_spread = Some(dec(s)));
    }
    let w = testkit::game_with(&r, SIZE, 1);
    for (i, (s, big_r, k)) in cases.iter().enumerate() {
        assert_eq!(
            area::radii(&w, wh(&w, &format!("S{i}"))),
            (*big_r, *k),
            "spread {s}"
        );
    }
}

// spec: S-0303 claim 3 (an object within R in a cell that is not scanned is not hit, and one in a scanned cell further than R is not hit)
#[test]
fn spread_radius_and_scanned_cells_are_separate_limits() {
    let mut r = testkit::combat_ruleset();
    vehicle(&mut r, "BOX", |_| {});
    warhead(&mut r, "TWO", |h| h.cell_spread = Some(dec("2")));
    warhead(&mut r, "ONE", |h| h.cell_spread = Some(dec("1")));
    let mut w = testkit::game_with(&r, SIZE, 1);
    // S = 2: k = 2, R = 512. Cell (2, 2) has ring metric 3: not scanned. Its
    // vehicle stands at the cell's corner nearest P, 363 leptons away.
    let outside = put(&mut w, Kind::Vehicle, "BOX", BETA, 2, 2);
    let corner = Pos::new(centre(2, 2).x - 128, centre(2, 2).y - 128, 0);
    w.obj_mut(outside).unwrap().pos = corner;
    let p = Pos::new(centre(0, 0).x + 127, centre(0, 0).y + 127, 0);
    let d = dist(&w, i64::from(corner.x - p.x), i64::from(corner.y - p.y), 0);
    assert!(d <= 512);
    let inside = put(&mut w, Kind::Vehicle, "BOX", BETA, 1, 1);
    let h = wh(&w, "TWO");
    boom!(w, p, 100, Some(h), None, None);
    assert_eq!(lost(&w, outside), 0);
    assert_eq!(lost(&w, inside), 100);
    // S = 1: k = 1 scans (1, 1), but its centre is 362 leptons from the
    // centre cell's centre, beyond R = 256.
    let h = wh(&w, "ONE");
    boom!(w, centre(0, 0), 100, Some(h), None, None);
    assert_eq!(lost(&w, inside), 100);
}

// ---------------------------------------------------------------------------
// Claim 7: epicentre distance.
// ---------------------------------------------------------------------------

// spec: S-0303 claim 7 (an object that is not a building is measured from P to its position, in three dimensions, as ⌊r(n)⌋)
#[test]
fn units_are_measured_to_their_position() {
    let mut r = testkit::combat_ruleset();
    vehicle(&mut r, "BOX", |_| {});
    linear(&mut r, "LIN", "2");
    let mut w = testkit::game_with(&r, SIZE, 1);
    let v = put(&mut w, Kind::Vehicle, "BOX", BETA, 0, 0);
    let q = w.obj(v).unwrap().pos;
    let p = Pos::new(q.x + 100, q.y - 30, q.z + 50);
    let h = wh(&w, "LIN");
    boom!(w, p, 512, Some(h), None, None);
    let d = dist(&w, 100, 30, 50);
    assert_eq!(lost(&w, v), 512 - d);
}

// spec: S-0303 claim 7 (a building is measured to each foundation cell's centre at the surface; 0 in the centre cell, unless P is more than 208 leptons above it, then the distance less 208)
#[test]
fn buildings_are_measured_to_cell_centres() {
    let mut r = testkit::combat_ruleset();
    building(&mut r, "SHED", |_| {});
    linear(&mut r, "LIN", "2");
    for (dz, dx_cell, expect_d) in [(100, 0, 0), (300, 0, 92), (0, 1, 256)] {
        let mut w = testkit::game_with(&r, SIZE, 1);
        let b = put(&mut w, Kind::Building, "SHED", BETA, 0, 0);
        let c = centre(-dx_cell, 0);
        let p = Pos::new(c.x, c.y, c.z + dz);
        let h = wh(&w, "LIN");
        boom!(w, p, 512, Some(h), None, None);
        assert_eq!(lost(&w, b), 512 - expect_d, "dz {dz}, cell {dx_cell}");
    }
}

// ---------------------------------------------------------------------------
// Claim 8: buildings take one hit per scanned foundation cell.
// ---------------------------------------------------------------------------

fn barn_world(f: impl FnOnce(&mut Ruleset)) -> (World, ObjectId, WarheadId) {
    let mut r = testkit::combat_ruleset();
    building(&mut r, "BARN", |b| b.foundation = Some([2, 2]));
    linear(&mut r, "LIN", "2");
    f(&mut r);
    let mut w = testkit::game_with(&r, SIZE, 1);
    let b = put(&mut w, Kind::Building, "BARN", BETA, 0, 0);
    let h = wh(&w, "LIN");
    (w, b, h)
}

// spec: S-0303 claim 8 and Edge cases (quirks.area_damage_per_foundation_cell on: one hit for each scanned foundation cell within R, each at its own distance)
#[test]
fn area_damage_per_foundation_cell_on() {
    let (mut w, b, h) = barn_world(|_| {});
    boom!(w, centre(0, 0), 512, Some(h), None, None);
    let diag = dist(&w, 256, 256, 0);
    assert_eq!(lost(&w, b), 512 + 256 + 256 + (512 - diag));
}

// spec: S-0303 Edge cases (quirks.area_damage_per_foundation_cell off: one hit only, at the smallest distance)
#[test]
fn area_damage_per_foundation_cell_off() {
    let (mut w, b, h) = barn_world(|r| r.globals.quirks.area_damage_per_foundation_cell = false);
    let p = centre(-1, 0);
    boom!(w, p, 512, Some(h), None, None);
    // The nearest foundation cell is 256 leptons away.
    assert_eq!(lost(&w, b), 256);
}

// spec: S-0303 claim 8 (an undemolishable building loses 1 from each hit that the versus value brings to 0)
#[test]
fn undemolishable_buildings_lose_one_per_cell() {
    let (mut w, b, _) = barn_world(|r| {
        r.sim.buildings.get_mut("BARN").unwrap().can_c4 = Some(false);
        warhead(r, "NONE", |h| {
            h.verses = every("0");
            h.cell_spread = Some(dec("2"));
        });
    });
    boom!(w, centre(0, 0), 100, Some(wh(&w, "NONE")), None, None);
    assert_eq!(lost(&w, b), 4);
}

// spec: S-0303 claim 8 (once a hit has destroyed a building, its remaining hits from the same detonation do nothing)
#[test]
fn a_destroyed_building_takes_no_further_hits() {
    let (mut w, b, h) = barn_world(|r| {
        r.sim.buildings.get_mut("BARN").unwrap().strength = Some(300);
    });
    let a = put(&mut w, Kind::Vehicle, "CART", ALPHA, -8, 0);
    boom!(w, centre(0, 0), 512, Some(h), Some(a), Some(HouseId(ALPHA)));
    assert!(w.obj(b).is_none_or(|o| o.health == 0));
    assert_eq!(w.houses[0].combat.kills, 1);
}

/// A 2 × 2 building eligible for delayed kills, hit by a delay-kill
/// warhead whose spread covers all four cells; returns the kills credited.
fn delayed_barn(quirk: bool) -> i32 {
    let (mut w, b, _) = barn_world(|r| {
        let barn = r.sim.buildings.get_mut("BARN").unwrap();
        barn.strength = Some(300);
        barn.eligible_for_delay_kill = Some(true);
        warhead(r, "SLOW", |h| {
            h.cell_spread = Some(dec("2"));
            h.causes_delay_kill = Some(true);
            h.delay_kill_frames = Some(100);
        });
        r.globals.quirks.delay_kill_credited_again = quirk;
    });
    let a = put(&mut w, Kind::Vehicle, "CART", ALPHA, -8, 0);
    let h = wh(&w, "SLOW");
    boom!(
        w,
        centre(0, 0),
        1000,
        Some(h),
        Some(a),
        Some(HouseId(ALPHA))
    );
    assert_eq!(health(&w, b), 1);
    w.houses[0].combat.kills
}

// spec: S-0302 claim 16 and S-0303 claim 8 (quirks.delay_kill_credited_again on: the later hits of the detonation that started a delayed kill land on the other foundation cells and credit the destruction again)
#[test]
fn delayed_kill_credited_again_for_each_cell() {
    assert_eq!(delayed_barn(true), 4);
}

// spec: S-0302 Edge cases (quirks.delay_kill_credited_again off: the destruction is credited once, at the first hit)
#[test]
fn delayed_kill_credited_once_for_the_detonation() {
    assert_eq!(delayed_barn(false), 1);
}

// ---------------------------------------------------------------------------
// Claim 9: objects left out.
// ---------------------------------------------------------------------------

// spec: S-0303 claim 9 (the source object is left out unless its type has the self-damage flag or the warhead is the game-wide crushing warhead)
#[test]
fn the_source_object_is_left_out() {
    let mut r = testkit::combat_ruleset();
    vehicle(&mut r, "BOX", |_| {});
    vehicle(&mut r, "MASO", |t| t.combat.damage_self = Some(true));
    warhead(&mut r, "PLAIN", |_| {});
    warhead(&mut r, "CRUSH", |_| {});
    r.sim.general.crush_warhead = Some("CRUSH".into());
    let mut w = testkit::game_with(&r, SIZE, 1);
    let bx = put(&mut w, Kind::Vehicle, "BOX", BETA, 0, 0);
    let maso = put(&mut w, Kind::Vehicle, "MASO", BETA, 4, 0);
    let plain = wh(&w, "PLAIN");
    let res = boom!(w, centre(0, 0), 100, Some(plain), Some(bx), None);
    assert_eq!(res, DetonationResult::Missed);
    assert_eq!(lost(&w, bx), 0);
    boom!(w, centre(4, 0), 100, Some(plain), Some(maso), None);
    assert_eq!(lost(&w, maso), 100);
    let crush = wh(&w, "CRUSH");
    boom!(w, centre(0, 0), 100, Some(crush), Some(bx), None);
    assert_eq!(lost(&w, bx), 100);
}

// spec: S-0303 claim 9 (an object with health 0 or less, or a building marked invisible in game, receives no hit)
#[test]
fn dead_objects_and_invisible_buildings_are_left_out() {
    let mut r = testkit::combat_ruleset();
    vehicle(&mut r, "BOX", |_| {});
    building(&mut r, "GHOST", |b| b.invisible_in_game = Some(true));
    warhead(&mut r, "PLAIN", |_| {});
    let mut w = testkit::game_with(&r, SIZE, 1);
    let v = put(&mut w, Kind::Vehicle, "BOX", BETA, 0, 0);
    w.obj_mut(v).unwrap().health = 0;
    let g = put(&mut w, Kind::Building, "GHOST", BETA, 4, 0);
    let h = wh(&w, "PLAIN");
    assert_eq!(
        boom!(w, centre(0, 0), 100, Some(h), None, None),
        DetonationResult::Missed
    );
    assert_eq!(health(&w, v), 0);
    assert_eq!(
        boom!(w, centre(4, 0), 100, Some(h), None, None),
        DetonationResult::Missed
    );
    assert_eq!(lost(&w, g), 0);
}

// ---------------------------------------------------------------------------
// Claim 11 and table T1: scan order.
// ---------------------------------------------------------------------------

// spec: S-0303 table T1 (ring 0, ring 1 in its own order, rings 2 to 10 by increasing Δy then Δx; cumulative counts 1, 9, 21, 37, 61, 89, 121, 161, 205, 253, 309, 369)
#[test]
fn scan_order_and_counts() {
    assert_eq!(area::ring(0, true), vec![(0, 0)]);
    assert_eq!(
        area::ring(1, true),
        vec![
            (1, -1),
            (0, -1),
            (-1, -1),
            (-1, 0),
            (1, 0),
            (-1, 1),
            (0, 1),
            (1, 1)
        ]
    );
    let r2 = area::ring(2, true);
    assert_eq!(&r2[..4], &[(-1, -2), (0, -2), (1, -2), (-2, -1)]);
    for k in 2..=10 {
        let ring = area::ring(k, true);
        let mut sorted = ring.clone();
        sorted.sort_by_key(|&(dx, dy)| (dy, dx));
        assert_eq!(ring, sorted, "ring {k}");
        assert!(ring.iter().all(|&(dx, dy)| area::ring_metric(dx, dy) == k));
    }
    let want = [1, 9, 21, 37, 61, 89, 121, 161, 205, 253, 309, 369];
    let mut total = 0;
    for (k, w) in want.iter().enumerate() {
        total += area::ring(k as i32, true).len();
        assert_eq!(total, *w, "k = {k}");
    }
}

// spec: S-0303 Edge cases (quirks.spread_ring_eleven on: ring 11 is the table's list of 60, with (−3, 11) twice, without (3, −11) and with cells of metric 12)
#[test]
fn spread_ring_eleven_on() {
    let ring = area::ring(11, true);
    assert_eq!(ring.len(), 60);
    assert_eq!(ring.iter().filter(|&&c| c == (-3, 11)).count(), 2);
    assert!(!ring.contains(&(3, -11)));
    assert!(!ring.contains(&(2, 10)));
    assert!(ring.contains(&(2, 11)));
    assert_eq!(&ring[..3], &[(0, 11), (0, -11), (-1, 11)]);
    assert_eq!(&ring[58..], &[(11, 0), (-11, 0)]);
}

// spec: S-0303 Edge cases (quirks.spread_ring_eleven off: ring 11 is the cells of ring metric 11 by increasing Δy then Δx)
#[test]
fn spread_ring_eleven_off() {
    let ring = area::ring(11, false);
    let mut want = Vec::new();
    for dy in -11..=11 {
        for dx in -11..=11 {
            if area::ring_metric(dx, dy) == 11 {
                want.push((dx, dy));
            }
        }
    }
    assert_eq!(ring, want);
}

// spec: S-0303 table T1 (the cell that ring 11 scans twice gives each object there two hits)
#[test]
fn the_doubled_ring_eleven_cell_is_hit_twice() {
    let mut r = testkit::combat_ruleset();
    vehicle(&mut r, "BOX", |_| {});
    warhead(&mut r, "HUGE", |h| h.cell_spread = Some(dec("11")));
    let mut w = testkit::game_with(&r, 40, 1);
    let mid = testkit::mid(40, 0, 0);
    let v = testkit::spawn(
        &mut w,
        Kind::Vehicle,
        "BOX",
        BETA,
        testkit::mid(40, -3, 11),
        0,
    );
    // P at the centre cell's corner nearest the vehicle: k = 11, R = 2816.
    let q = Pos::new(mid.x * 256, mid.y * 256 + 255, 0);
    let vp = w.obj(v).unwrap().pos;
    let d = dist(&w, i64::from(vp.x - q.x), i64::from(vp.y - q.y), 0);
    assert!(d <= 2816);
    let h = wh(&w, "HUGE");
    boom!(w, q, 100, Some(h), None, None);
    assert_eq!(lost(&w, v), 200);
}

// ---------------------------------------------------------------------------
// Claim 12 and F3: falloff.
// ---------------------------------------------------------------------------

// spec: S-0303 F3 (worked example: d₃ = 100, p = 0.5, R = 512 gives 100, 75, 50 and 50 at 0, 256, 511 and 512; p = 1 or R = 0 leave d₃ unchanged; p above 1 rises towards the edge)
#[test]
fn falloff_worked_example() {
    let mut r = testkit::combat_ruleset();
    warhead(&mut r, "HALF", |h| {
        h.cell_spread = Some(dec("2"));
        h.percent_at_max = Some(dec("0.5"));
    });
    warhead(&mut r, "FLAT", |h| h.cell_spread = Some(dec("2")));
    warhead(&mut r, "POINT", |h| h.percent_at_max = Some(dec("0.5")));
    warhead(&mut r, "RISE", |h| {
        h.cell_spread = Some(dec("2"));
        h.percent_at_max = Some(dec("2"));
    });
    let w = testkit::game_with(&r, SIZE, 1);
    let one = F64::from_i64(1, RZ);
    let f = |name: &str, d3: i32, d: i32| damage::falloff_versus(&w, wh(&w, name), d3, d, one);
    assert_eq!(f("HALF", 100, 0), 100);
    assert_eq!(f("HALF", 100, 256), 75);
    assert_eq!(f("HALF", 100, 511), 50);
    assert_eq!(f("HALF", 100, 512), 50);
    assert_eq!(f("FLAT", 100, 400), 100);
    assert_eq!(f("POINT", 100, 400), 100);
    assert_eq!(f("RISE", 100, 256), 150);
}

// spec: S-0303 claim 12 and F3 (the falloff works on the defended amount: D = 1000 with an armour divisor of 3 gives 333, and 249 at d = 512 with R = 1024 and p = 0.5)
#[test]
fn falloff_works_on_the_defended_amount() {
    let mut r = testkit::combat_ruleset();
    vehicle(&mut r, "BOX", |_| {});
    warhead(&mut r, "WIDE", |h| {
        h.cell_spread = Some(dec("4"));
        h.percent_at_max = Some(dec("0.5"));
    });
    r.sim
        .countries
        .get_mut("Betaland")
        .unwrap()
        .armor_units_mult = Some(dec("3"));
    let mut w = testkit::game_with(&r, SIZE, 1);
    let v = put(&mut w, Kind::Vehicle, "BOX", BETA, 2, 0);
    let h = wh(&w, "WIDE");
    boom!(w, centre(0, 0), 1000, Some(h), None, None);
    assert_eq!(lost(&w, v), 249);
}

// ---------------------------------------------------------------------------
// Claim 13: healing detonations.
// ---------------------------------------------------------------------------

// spec: S-0303 claim 13 (a negative amount heals only below 8 leptons, by the full amount; an undemolishable building in the spread loses 1 per hit instead)
#[test]
fn healing_detonations_reach_the_point_only() {
    let mut r = testkit::combat_ruleset();
    vehicle(&mut r, "BOX", |_| {});
    building(&mut r, "HARD", |b| b.can_c4 = Some(false));
    warhead(&mut r, "MEND", |h| {
        h.cell_spread = Some(dec("2"));
        h.verses = every("0.5");
    });
    let mut w = testkit::game_with(&r, SIZE, 1);
    let a = put(&mut w, Kind::Vehicle, "BOX", BETA, 0, 0);
    let b = put(&mut w, Kind::Vehicle, "BOX", BETA, 1, 0);
    let hard = put(&mut w, Kind::Building, "HARD", BETA, 0, 1);
    for id in [a, b, hard] {
        w.obj_mut(id).unwrap().health = 1000;
    }
    let h = wh(&w, "MEND");
    let p = w.obj(a).unwrap().pos;
    boom!(w, p, -50, Some(h), None, None);
    assert_eq!(health(&w, a), 1050);
    assert_eq!(health(&w, b), 1000);
    assert_eq!(health(&w, hard), 999);
}

// ---------------------------------------------------------------------------
// Claim 14: friendly damage.
// ---------------------------------------------------------------------------

// spec: S-0303 claim 14 (area damage hits the source house's own objects; with the affects-allies field off and an attacker, objects allied with the attacker's house take nothing; without an attacker they are hit)
#[test]
fn friendly_damage() {
    let mut r = testkit::combat_ruleset();
    vehicle(&mut r, "BOX", |_| {});
    warhead(&mut r, "PLAIN", |h| h.cell_spread = Some(dec("1")));
    warhead(&mut r, "KIND", |h| {
        h.cell_spread = Some(dec("1"));
        h.affects_allies = Some(false);
    });
    let mut w = testkit::game_with(&r, SIZE, 1);
    let firer = put(&mut w, Kind::Vehicle, "BOX", ALPHA, -8, 0);
    let own = put(&mut w, Kind::Vehicle, "BOX", ALPHA, 0, 0);
    let foe = put(&mut w, Kind::Vehicle, "BOX", BETA, 1, 0);
    let (plain, kind) = (wh(&w, "PLAIN"), wh(&w, "KIND"));
    let p = centre(0, 0);
    let alpha = Some(HouseId(ALPHA));
    boom!(w, p, 100, Some(plain), Some(firer), alpha);
    assert_eq!((lost(&w, own), lost(&w, foe)), (100, 100));
    boom!(w, p, 100, Some(kind), Some(firer), alpha);
    assert_eq!((lost(&w, own), lost(&w, foe)), (100, 200));
    boom!(w, p, 100, Some(kind), None, alpha);
    assert_eq!((lost(&w, own), lost(&w, foe)), (200, 300));
}

// ---------------------------------------------------------------------------
// Claim 16 and F4: walls.
// ---------------------------------------------------------------------------

fn wall_world(f: impl FnOnce(&mut Ruleset)) -> World {
    let mut r = testkit::combat_ruleset();
    overlay(&mut r, "PALISADE", |o| {
        o.wall = Some(true);
        o.strength = Some(100);
        o.damage_levels = Some(3);
        o.armor = Some("wood".into());
    });
    overlay(&mut r, "RAMPART", |o| {
        o.wall = Some(true);
        o.strength = Some(100);
        o.damage_levels = Some(3);
        o.armor = Some("steel".into());
    });
    warhead(&mut r, "PLAIN", |h| h.cell_spread = Some(dec("1")));
    warhead(&mut r, "WALLER", |h| {
        h.cell_spread = Some(dec("1"));
        h.wall = Some(true);
    });
    warhead(&mut r, "CHOP", |h| {
        h.cell_spread = Some(dec("1"));
        h.wood = Some(true);
    });
    warhead(&mut r, "RAZE", |h| {
        h.cell_spread = Some(dec("1"));
        h.wall_absolute_destroyer = Some(true);
    });
    f(&mut r);
    testkit::game_with(&r, SIZE, 1)
}

// spec: S-0303 claim 16 and F4 (a wall hit for the wall flag, or the wood flag on a wood wall, in every scanned cell, with the full amount and no falloff: a ≥ T succeeds with no draw and raises the stage by one)
#[test]
fn wall_hits_by_flag_and_class() {
    let mut w = wall_world(|_| {});
    let (pal, ram) = (ov(&w, "PALISADE"), ov(&w, "RAMPART"));
    set_overlay(&mut w, at(1, 0), pal, 0);
    set_overlay(&mut w, at(-1, 0), ram, 0);
    let taken = w.rng.taken();
    let p = centre(0, 0);
    boom!(w, p, 150, Some(wh(&w, "PLAIN")), None, None);
    assert_eq!(overlay_at(&w, at(1, 0)), (pal, 0));
    assert_eq!(overlay_at(&w, at(-1, 0)), (ram, 0));
    boom!(w, p, 150, Some(wh(&w, "CHOP")), None, None);
    assert_eq!(overlay_at(&w, at(1, 0)), (pal, 16));
    assert_eq!(overlay_at(&w, at(-1, 0)), (ram, 0));
    // The neighbour cells are 256 leptons from P: no falloff applies.
    boom!(w, p, 100, Some(wh(&w, "WALLER")), None, None);
    assert_eq!(overlay_at(&w, at(-1, 0)), (ram, 16));
    assert_eq!(w.rng.taken(), taken);
}

// spec: S-0303 F4 (a wall hit below the wall strength takes one draw over 0 … T and succeeds when a > r; a negative amount still draws and never succeeds)
#[test]
fn weak_wall_hits_draw() {
    for (amount, seed) in [(60, 1), (60, 2), (60, 3), (30, 4), (-20, 5)] {
        let mut w = wall_world(|_| {});
        let ram = ov(&w, "RAMPART");
        set_overlay(&mut w, at(0, 0), ram, 0);
        for _ in 0..seed {
            w.rng.raw();
        }
        let mut rng = w.rng.clone();
        let half = w.rules().quirks.ranged_draw_half_span;
        let r = rng.ranged(0, 100, half);
        let stage = if amount > r { 16 } else { 0 };
        boom!(w, centre(0, 0), amount, Some(wh(&w, "WALLER")), None, None);
        assert_eq!(overlay_at(&w, at(0, 0)), (ram, stage), "{amount}");
        assert_eq!(w.rng.taken(), rng.taken());
    }
}

// spec: S-0303 claim 16 and F4 (an absolute wall hit always succeeds with no draw and removes the wall, whatever the sign of D)
#[test]
fn absolute_wall_hits_remove_the_wall() {
    let mut w = wall_world(|_| {});
    let ram = ov(&w, "RAMPART");
    set_overlay(&mut w, at(0, 0), ram, 0);
    let taken = w.rng.taken();
    boom!(w, centre(0, 0), -5, Some(wh(&w, "RAZE")), None, None);
    assert_eq!(overlay_at(&w, at(0, 0)).0, NO_OVERLAY);
    assert_eq!(w.rng.taken(), taken);
}

// spec: S-0303 F4 (a success that brings the stage to L − 1 with L > 2 hits the four neighbours of the same type at stage 0 with 200; a wall at L − 1 with no link is removed, one with a link stays)
#[test]
fn wall_stages_neighbours_and_removal() {
    let mut w = wall_world(|r| {
        warhead(r, "POINTWALL", |h| h.wall = Some(true));
    });
    let ram = ov(&w, "RAMPART");
    let pw = wh(&w, "POINTWALL");
    set_overlay(&mut w, at(0, 0), ram, 16);
    set_overlay(&mut w, at(0, -1), ram, 0);
    set_overlay(&mut w, at(1, 0), ram, 0);
    set_overlay(&mut w, at(0, 1), ram, 16);
    boom!(w, centre(0, 0), 150, Some(pw), None, None);
    // Stage 2 = L − 1, no link: removed; the stage-0 neighbours took 200.
    assert_eq!(overlay_at(&w, at(0, 0)).0, NO_OVERLAY);
    assert_eq!(overlay_at(&w, at(0, -1)), (ram, 16));
    assert_eq!(overlay_at(&w, at(1, 0)), (ram, 16));
    assert_eq!(overlay_at(&w, at(0, 1)), (ram, 16));
    // With a link bit the wall stays at L − 1.
    set_overlay(&mut w, at(5, 5), ram, 16 + 1);
    boom!(w, centre(5, 5), 150, Some(pw), None, None);
    assert_eq!(overlay_at(&w, at(5, 5)), (ram, 32 + 1));
}

// spec: S-0303 Edge cases (quirks.wall_hit_minus_one_is_absolute on: a wall hit of exactly −1 succeeds with no draw and removes the wall)
#[test]
fn wall_hit_minus_one_is_absolute_on() {
    let mut w = wall_world(|_| {});
    let ram = ov(&w, "RAMPART");
    set_overlay(&mut w, at(0, 0), ram, 0);
    let taken = w.rng.taken();
    boom!(w, centre(0, 0), -1, Some(wh(&w, "WALLER")), None, None);
    assert_eq!(overlay_at(&w, at(0, 0)).0, NO_OVERLAY);
    assert_eq!(w.rng.taken(), taken);
}

// spec: S-0303 Edge cases (quirks.wall_hit_minus_one_is_absolute off: −1 is like any negative amount, a draw and no damage)
#[test]
fn wall_hit_minus_one_is_absolute_off() {
    let mut w = wall_world(|r| r.globals.quirks.wall_hit_minus_one_is_absolute = false);
    let ram = ov(&w, "RAMPART");
    set_overlay(&mut w, at(0, 0), ram, 0);
    let taken = w.rng.taken();
    boom!(w, centre(0, 0), -1, Some(wh(&w, "WALLER")), None, None);
    assert_eq!(overlay_at(&w, at(0, 0)), (ram, 0));
    assert!(w.rng.taken() > taken);
}

// spec: S-0303 State and ordering (every scanned cell's wall hit and its draw come before the first hit's draws, even for a wall later in the scan order)
#[test]
fn wall_draws_come_before_the_hits() {
    let mut told_apart = 0;
    for seed in 0..40 {
        let mut w = wall_world(|r| {
            vehicle(r, "SMOKER", |t| {
                t.strength = Some(1000);
                t.combat.damage_smoke_systems = Some(vec!["PuffA".into(), "PuffB".into()]);
            });
            r.sim.general.condition_red = Some(dec("0.1"));
        });
        for _ in 0..seed {
            w.rng.raw();
        }
        let ram = ov(&w, "RAMPART");
        // The wall is in a later cell of the scan order than the vehicle.
        set_overlay(&mut w, at(1, 1), ram, 0);
        let v = put(&mut w, Kind::Vehicle, "SMOKER", BETA, 0, 0);
        // 520 − 60 crosses half the strength: the hit takes its effect draw.
        w.obj_mut(v).unwrap().health = 520;
        let half = w.rules().quirks.ranged_draw_half_span;
        let mut wall_first = w.rng.clone();
        let r_first = wall_first.ranged(0, 100, half);
        let _ = wall_first.ranged(0, 1, half);
        let mut hit_first = w.rng.clone();
        let _ = hit_first.ranged(0, 1, half);
        let r_second = hit_first.ranged(0, 100, half);
        boom!(w, centre(0, 0), 60, Some(wh(&w, "WALLER")), None, None);
        assert_eq!(health(&w, v), 460);
        assert!(w.obj(v).unwrap().combat.damage_effect);
        let stage = |r: i32| if 60 > r { 16 } else { 0 };
        assert_eq!(
            overlay_at(&w, at(1, 1)),
            (ram, stage(r_first)),
            "seed {seed}"
        );
        assert_eq!(w.rng.taken(), wall_first.taken());
        if stage(r_first) != stage(r_second) {
            told_apart += 1;
        }
    }
    assert!(told_apart > 0, "some seed tells the two orders apart");
}

// ---------------------------------------------------------------------------
// Claim 19: exploding overlays.
// ---------------------------------------------------------------------------

fn barrel_world() -> (World, u8) {
    let mut r = testkit::combat_ruleset();
    overlay(&mut r, "KEG", |o| o.explodes = Some(true));
    vehicle(&mut r, "BOX", |t| t.strength = Some(50));
    warhead(&mut r, "DEMO", |h| h.cell_spread = Some(dec("1.5")));
    warhead(&mut r, "PLAIN", |_| {});
    let g = &mut r.sim.general;
    g.c4_warhead = Some("DEMO".into());
    g.ammo_crate_damage = Some(80);
    g.barrel_explode = Some("KegBoom".into());
    g.barrel_debris = Some(vec!["StaveA".into(), "StaveB".into(), "StaveC".into()]);
    g.barrel_particle = Some("KegSmoke".into());
    let w = testkit::game_with(&r, SIZE, 1);
    let keg = ov(&w, "KEG");
    (w, keg)
}

// spec: S-0303 claim 19 (an exploding overlay in the centre cell is removed, the barrel explosion appears at P, a nested detonation of the ammunition-crate damage with the demolition warhead, no source object and the outer house follows, then percent draws for the debris (first below 15) and one for the particle system; whatever the sign of D)
#[test]
fn exploding_overlays() {
    for (seed, outer) in [(1, 10), (2, -10), (3, 10), (4, 10)] {
        let (mut w, keg) = barrel_world();
        for _ in 0..seed {
            w.rng.raw();
        }
        set_overlay(&mut w, at(0, 0), keg, 0);
        set_overlay(&mut w, at(3, 0), keg, 0);
        let v = put(&mut w, Kind::Vehicle, "BOX", BETA, 1, 0);
        let mut rng = w.rng.clone();
        let mut want = vec![("KegBoom".to_owned(), centre(0, 0))];
        for name in ["StaveA", "StaveB", "StaveC"] {
            if rng.percent() < 15 {
                want.push((name.to_owned(), centre(0, 0)));
                break;
            }
        }
        let _ = rng.percent();
        w.effects.clear();
        boom!(
            w,
            centre(0, 0),
            outer,
            Some(wh(&w, "PLAIN")),
            None,
            Some(HouseId(ALPHA)),
        );
        assert_eq!(overlay_at(&w, at(0, 0)).0, NO_OVERLAY);
        assert_eq!(overlay_at(&w, at(3, 0)).0, keg, "only the centre cell");
        assert!(
            w.obj(v).is_none_or(|o| o.health == 0),
            "the nested detonation"
        );
        assert_eq!(w.houses[0].combat.kills, 1, "credited to the outer house");
        assert_eq!(anims(&w), want, "seed {seed}");
        assert_eq!(w.rng.taken(), rng.taken());
    }
}

// ---------------------------------------------------------------------------
// Claims 21 to 23: explosion animation, debris and the result.
// ---------------------------------------------------------------------------

fn anim_world(f: impl FnOnce(&mut WarheadType)) -> (World, WeaponId) {
    let mut r = testkit::combat_ruleset();
    warhead(&mut r, "SHOW", |h| {
        h.anim_list = Some(vec!["BoomS".into(), "BoomM".into(), "BoomL".into()]);
        f(h);
    });
    weapon(&mut r, "Show", |t| {
        t.damage = Some(10);
        t.warhead = Some("SHOW".into());
    });
    warhead(&mut r, "BOLT", |_| {});
    weapon(&mut r, "Bolt", |t| {
        t.damage = Some(10);
        t.warhead = Some("BOLT".into());
    });
    r.sim.general.lightning_warhead = Some("BOLT".into());
    r.sim.general.weather_con_bolt_explosion = Some("Zigzag".into());
    r.sim.general.metallic_debris = Some(vec!["ScrapA".into(), "ScrapB".into()]);
    let w = testkit::game_with(&r, SIZE, 1);
    let s = wp(&w, "Show");
    (w, s)
}

fn detonation_anims(w: &mut World, weapon: WeaponId, damage: i32, invisible: bool) -> Vec<String> {
    w.effects.clear();
    let p = centre(0, 0);
    glasnost_sim::combat::projectile::projectile_detonation(
        w, weapon, damage, p, None, None, p, invisible,
    );
    anims(w).into_iter().map(|(n, _)| n).collect()
}

// spec: S-0303 claim 21 and F5 (graded case: entry trunc(min(D, 25 n_W − 1) / 25) of the warhead's list; D = 0: none; Edge cases: a negative entry shows nothing)
#[test]
fn explosion_animation_graded() {
    let (mut w, s) = anim_world(|_| {});
    for (dmg, want) in [
        (10, Some("BoomS")),
        (30, Some("BoomM")),
        (60, Some("BoomL")),
        (500, Some("BoomL")),
        (-10, Some("BoomS")),
        (-30, None),
        (0, None),
    ] {
        let got = detonation_anims(&mut w, s, dmg, false);
        assert_eq!(
            got,
            want.into_iter().map(String::from).collect::<Vec<_>>(),
            "{dmg}"
        );
    }
}

// spec: S-0303 F5 (the lightning warhead shows the bolt explosion; an electromagnetic warhead takes one ranged draw over its list)
#[test]
fn explosion_animation_lightning_and_electromagnetic() {
    let (mut w, _) = anim_world(|_| {});
    let bolt = wp(&w, "Bolt");
    assert_eq!(detonation_anims(&mut w, bolt, 10, false), vec!["Zigzag"]);
    let (mut w, s) = anim_world(|h| h.em_effect = Some(true));
    let mut rng = w.rng.clone();
    let half = w.rules().quirks.ranged_draw_half_span;
    let k = rng.ranged(0, 2, half);
    let names = ["BoomS", "BoomM", "BoomL"];
    assert_eq!(
        detonation_anims(&mut w, s, 10, false),
        vec![names[k as usize]]
    );
    assert_eq!(w.rng.taken(), rng.taken());
}

// spec: S-0303 claim 21 and State and ordering (an invisible projectile's animation stands at N(P, 32, u mod 256), its raw value after the area damage and before F5's draw)
#[test]
fn invisible_projectile_animation_point() {
    let (mut w, s) = anim_world(|h| h.em_effect = Some(true));
    let mut rng = w.rng.clone();
    let half = w.rules().quirks.ranged_draw_half_span;
    let u = rng.raw();
    let k = rng.ranged(0, 2, half);
    let p = centre(0, 0);
    let want = glasnost_sim::combat::death::nearby_point(&w, p, 32, u & 255, false);
    w.effects.clear();
    glasnost_sim::combat::projectile::projectile_detonation(&mut w, s, 10, p, None, None, p, true);
    let names = ["BoomS", "BoomM", "BoomL"];
    assert_eq!(anims(&w), vec![(names[k as usize].to_owned(), want)]);
    assert_eq!(w.rng.taken(), rng.taken());
}

// spec: S-0303 claim 22 and F6 (warhead debris whatever the damage amount: N = R(Y, X − 1), and without debris types N metallic debris from R(0, c − 1) each, at Q raised by 20)
#[test]
fn metallic_debris() {
    for dmg in [10, 0] {
        let (mut w, s) = anim_world(|h| {
            h.max_debris = Some(4);
            h.min_debris = Some(1);
            h.anim_list = None;
        });
        let mut rng = w.rng.clone();
        let half = w.rules().quirks.ranged_draw_half_span;
        let n = rng.ranged(1, 3, half);
        let p = centre(0, 0);
        let mut want = Vec::new();
        for _ in 0..n {
            let k = rng.ranged(0, 1, half);
            want.push((
                ["ScrapA", "ScrapB"][k as usize].to_owned(),
                Pos::new(p.x, p.y, p.z + 20),
            ));
        }
        w.effects.clear();
        glasnost_sim::combat::projectile::projectile_detonation(
            &mut w, s, dmg, p, None, None, p, false,
        );
        assert_eq!(anims(&w), want, "damage {dmg}");
        assert_eq!(w.rng.taken(), rng.taken());
    }
}

/// One detonation of a warhead with listed debris types after `seed` raw
/// values: (pieces per type created, pieces per type by F6, whether F6's
/// deals ran a full pass over the types that placed nothing, whether the
/// draws taken agree).
fn listed_debris(seed: u32) -> ([i32; 2], [i32; 2], bool, bool) {
    let (mut w, s) = anim_world(|h| {
        h.max_debris = Some(6);
        h.min_debris = Some(3);
        h.anim_list = None;
        h.debris_types = Some(vec!["ChunkA".into(), "ChunkB".into()]);
        h.debris_maximums = Some(vec![2, 1]);
    });
    for _ in 0..seed {
        w.rng.raw();
    }
    let mut rng = w.rng.clone();
    let half = w.rules().quirks.ranged_draw_half_span;
    let mut left = rng.ranged(3, 5, half);
    let mut counts = [0; 2];
    let maxima = [2, 1];
    let mut j = 0;
    let mut pass_placed = 0;
    let mut zero_pass = false;
    while left > 0 {
        let i = j % 2;
        if i == 0 && j > 0 {
            if pass_placed == 0 {
                zero_pass = true;
            }
            pass_placed = 0;
        }
        let raw = rng.raw() as i32;
        let g = (raw.wrapping_abs() % (maxima[i] + 1)).min(left);
        counts[i] += g;
        pass_placed += g;
        left -= g;
        j += 1;
    }
    w.effects.clear();
    let p = centre(0, 0);
    glasnost_sim::combat::projectile::projectile_detonation(&mut w, s, 10, p, None, None, p, false);
    let got = anims(&w);
    let a = got.iter().filter(|(n, _)| n == "ChunkA").count() as i32;
    let b = got.iter().filter(|(n, _)| n == "ChunkB").count() as i32;
    let same_draws = w.rng.taken() == rng.taken();
    ([a, b], counts, zero_pass, same_draws)
}

// spec: S-0303 F6 (listed debris types are dealt in turn, each deal taking one raw value and giving min(|raw| mod (u + 1), pieces left) pieces)
#[test]
fn listed_debris_types() {
    let mut checked = 0;
    for seed in 0..30 {
        let (got, want, zero_pass, draws) = listed_debris(seed);
        if zero_pass {
            continue;
        }
        assert_eq!(got, want, "seed {seed}");
        assert!(draws, "seed {seed}");
        checked += 1;
    }
    assert!(checked > 0);
}

// spec: S-0303 F6 (the deals go on while pieces are left; only maxima that are all 0 bring in the departure of Edge cases)
#[test]
fn listed_debris_continues_after_a_pass_that_places_nothing() {
    let mut checked = 0;
    for seed in 0..30 {
        let (got, want, zero_pass, draws) = listed_debris(seed);
        if !zero_pass {
            continue;
        }
        assert_eq!(got, want, "seed {seed}");
        assert!(draws, "seed {seed}");
        checked += 1;
    }
    assert!(checked > 0);
}

// spec: S-0303 claim 23 (a detonation reports hit when at least one object received a hit, missed otherwise)
#[test]
fn detonation_results() {
    let mut r = testkit::combat_ruleset();
    vehicle(&mut r, "BOX", |_| {});
    warhead(&mut r, "PLAIN", |h| h.cell_spread = Some(dec("1")));
    let mut w = testkit::game_with(&r, SIZE, 1);
    put(&mut w, Kind::Vehicle, "BOX", BETA, 0, 0);
    let h = wh(&w, "PLAIN");
    assert_eq!(
        boom!(w, centre(0, 0), 10, Some(h), None, None),
        DetonationResult::Hit
    );
    assert_eq!(
        boom!(w, centre(6, 6), 10, Some(h), None, None),
        DetonationResult::Missed
    );
}

// spec: S-0303 claim 21 (over water the splash list replaces the warhead's own, except when the area damage hit something and the first object in the ground layer of Q's cell is a naval vehicle that is not an underwater type: L counts as air)
#[test]
fn naval_vehicle_over_water_shows_the_warheads_own_animation() {
    for (naval, underwater, want) in [
        (None, None, "Splash"),
        (Some(true), Some(false), "BoomS"),
        (Some(true), Some(true), "Splash"),
        (Some(false), None, "Splash"),
    ] {
        let mut r = testkit::combat_ruleset();
        warhead(&mut r, "SHOW", |h| {
            h.anim_list = Some(vec!["BoomS".into(), "BoomM".into(), "BoomL".into()]);
            h.conventional = Some(true);
            h.cell_spread = Some(glasnost_rules::sim::Decimal("1".into()));
        });
        weapon(&mut r, "Show", |t| {
            t.damage = Some(10);
            t.warhead = Some("SHOW".into());
        });
        r.sim.general.splash_list = Some(vec!["Splash".into()]);
        let cart = r.sim.vehicles.get_mut("CART").unwrap();
        cart.combat.naval = naval;
        cart.combat.underwater = underwater;
        let mut w = testkit::game_with(&r, SIZE, 1);
        let s = wp(&w, "Show");
        let c = centre(0, 0).cell();
        if naval.is_some() {
            testkit::spawn(&mut w, Kind::Vehicle, "CART", BETA, c, 0);
        }
        // After the placement, which recalculates the cell's terrain.
        w.cells.grid.get_mut(c).unwrap().land = glasnost_map::terrain::land::WATER;
        let got = detonation_anims(&mut w, s, 10, false);
        assert_eq!(
            got,
            vec![want.to_string()],
            "naval {naval:?} underwater {underwater:?}"
        );
    }
}

// ---------------------------------------------------------------------------
// Claim 24 (revision 5): defaults of bridge and barrel damage.
// ---------------------------------------------------------------------------

// spec: S-0303 claim 24 (absent from every layer: bridge strength 1000 and ammunition-crate damage 100; the barrel's nested detonation still happens with that damage)
#[test]
fn claim_24_defaults_of_bridge_and_barrel_damage() {
    let mut r = testkit::combat_ruleset();
    r.sim.general.bridge_strength = None;
    r.sim.general.ammo_crate_damage = None;
    let w = testkit::game_with(&r, SIZE, 1);
    assert_eq!(w.rules().combat.general.bridge_strength, 1000);
    assert_eq!(w.rules().combat.general.ammo_crate_damage, 100);
    r.sim.general.ammo_crate_damage = Some(80);
    let w = testkit::game_with(&r, SIZE, 1);
    assert_eq!(w.rules().combat.general.ammo_crate_damage, 80);
}
