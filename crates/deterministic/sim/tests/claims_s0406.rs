//! Claim tests for S-0406 (building placement) and the construction state
//! it begins (S-0108 claim 5, request Q-0130) on synthetic data: invented
//! building, overlay and vehicle types, no stock values. Houses Alpha (0,
//! Alphaland) and Beta (1, Betaland) of the testkit's sandbox game, not
//! allied.

use glasnost_map::CellXY;
use glasnost_map::terrain::land;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{BuildingType, Decimal, LandRow, LandType, OverlayType};
use glasnost_sim::econ::{TypeRef, placement, power};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, Pos, World};

const SIZE: i32 = 24;
const A: HouseId = HouseId(0);
const B: HouseId = HouseId(1);

fn building(w: i32, h: i32) -> BuildingType {
    let mut b = BuildingType {
        strength: Some(400),
        foundation: Some([w, h]),
        owner: Some(vec!["Alphaland".into(), "Betaland".into()]),
        ..Default::default()
    };
    b.econ.buildup = Some(true);
    // n = 5 build-up frames (S-0108 claim 5).
    b.econ.buildup_frames = Some(10);
    b
}

/// YARD (building factory), HUT (1 × 1, upgrade count 1), BIG (2 × 2),
/// REFY (3 × 3, the 8-cell word), NEAR (adjacency 0), ANY (place-anywhere),
/// TILE (to-tile field), WALLA (wall type of overlay O0, guard range 3
/// cells), TOWER (the wall tower), GATE1 (first gate pair), UPG (upgrade of
/// HUT, power 25), LVL2 (upgrade of HUT, level 2), LONE (country Gammaland
/// only); overlays O0 … O29 with O0, O2 and O26 walls and O5 ore.
fn rules() -> Ruleset {
    let mut r = testkit::ruleset();
    let names = [
        "WALLA", "YARD", "HUT", "BIG", "REFY", "NEAR", "ANY", "TILE", "TOWER", "GATE1", "UPG",
        "LVL2", "LONE",
    ];
    r.registries.insert(
        "buildings".into(),
        names.iter().map(|s| s.to_string()).collect(),
    );
    let overlays: Vec<String> = (0..30).map(|i| format!("O{i}")).collect();
    r.registries.insert("overlays".into(), overlays.clone());
    let s = &mut r.sim;
    s.land_types.insert(
        LandType::Clear,
        LandRow {
            buildable: Some(true),
            ..Default::default()
        },
    );
    for (i, n) in overlays.iter().enumerate() {
        s.overlays.insert(
            n.clone(),
            OverlayType {
                wall: Some(matches!(i, 0 | 2 | 26)),
                tiberium: Some(i == 5),
                has_art: Some(true),
                ..Default::default()
            },
        );
    }
    let mut yard = building(2, 2);
    yard.factory = Some("building".into());
    s.buildings.insert("YARD".into(), yard);
    let mut hut = building(1, 1);
    hut.econ.upgrades = Some(1);
    s.buildings.insert("HUT".into(), hut);
    s.buildings.insert("BIG".into(), building(2, 2));
    let mut refy = building(3, 3);
    refy.foundation_refinery = Some(true);
    s.buildings.insert("REFY".into(), refy);
    let mut near = building(1, 1);
    near.econ.adjacent = Some(0);
    s.buildings.insert("NEAR".into(), near);
    let mut any = building(1, 1);
    any.econ.place_anywhere = Some(true);
    s.buildings.insert("ANY".into(), any);
    let mut tile = building(2, 1);
    tile.econ.to_tile = Some("SOMETILE".into());
    s.buildings.insert("TILE".into(), tile);
    let mut wall = building(1, 1);
    wall.econ.wall = Some(true);
    wall.econ.to_overlay = Some("O0".into());
    wall.econ.guard_range = Some(Decimal("3".into()));
    s.buildings.insert("WALLA".into(), wall);
    s.buildings.insert("TOWER".into(), building(1, 1));
    let mut gate = building(1, 1);
    gate.gate = Some(true);
    s.buildings.insert("GATE1".into(), gate);
    let mut upg = building(1, 1);
    upg.econ.powers_up_building = Some("HUT".into());
    upg.econ.power = Some(25);
    s.buildings.insert("UPG".into(), upg);
    let mut lvl = building(1, 1);
    lvl.econ.powers_up_building = Some("HUT".into());
    lvl.econ.powers_up_to_level = Some(2);
    s.buildings.insert("LVL2".into(), lvl);
    let mut lone = building(1, 1);
    lone.owner = Some(vec!["Gammaland".into()]);
    s.buildings.insert("LONE".into(), lone);
    s.general.wall_tower = Some("TOWER".into());
    s.general.allied_gate_one = Some("GATE1".into());
    r
}

/// A sandbox game, past the ticks in which orders are blocked, with the
/// middle of the map explored by Alpha.
fn game(r: &Ruleset) -> World {
    let mut w = testkit::game_with(r, SIZE, 11);
    testkit::run(&mut w, 4);
    let p = Pos::centre(c(0, 0), 0);
    glasnost_sim::econ::shroud::area_reveal(&mut w, A, p, 10);
    w
}

fn c(dx: i32, dy: i32) -> CellXY {
    testkit::mid(SIZE, dx, dy)
}

fn ty(w: &World, name: &str) -> u32 {
    w.rules().index_of(Kind::Building, name).unwrap()
}

fn tr(w: &World, name: &str) -> TypeRef {
    TypeRef {
        kind: Kind::Building,
        index: ty(w, name),
    }
}

fn put(w: &mut World, name: &str, house: u8, cell: CellXY) -> ObjectId {
    testkit::spawn(w, Kind::Building, name, house, cell, 0)
}

/// A game with Alpha's YARD at (−6, −6), far from the cells the tests use.
fn with_yard(r: &Ruleset) -> World {
    let mut w = game(r);
    put(&mut w, "YARD", 0, c(-6, -6));
    w
}

fn place(w: &mut World, name: &str, cell: CellXY) -> bool {
    let t = tr(w, name);
    placement::place_building(w, A, t, cell)
}

fn buildings_at(w: &World, cell: CellXY) -> Vec<ObjectId> {
    w.cells
        .list(cell, glasnost_sim::world::Layer::Ground)
        .iter()
        .copied()
        .filter(|&id| w.obj(id).is_some_and(|o| o.kind == Kind::Building))
        .collect()
}

fn set_overlay(w: &mut World, cell: CellXY, overlay: u8, data: u8, owner: Option<HouseId>) {
    let g = w.cells.grid.get_mut(cell).unwrap();
    g.overlay = overlay;
    g.overlay_data = data;
    let i = w.cells.idx(cell).unwrap();
    w.cells.wall_owner[i] = owner;
}

fn overlay(w: &World, cell: CellXY) -> (u8, u8) {
    let g = w.cells.cell(cell).unwrap();
    (g.overlay, g.overlay_data)
}

// spec: S-0406 claim 1 (a place order is applied only when a factory building of the house could make the type; otherwise nothing happens)
// spec: S-0406 claim 1 (the factory's type shares an owner country with the item type, S-0404 Terms)
#[test]
fn claim_1_factory_condition() {
    let r = rules();
    let mut w = game(&r);
    assert!(!place(&mut w, "HUT", c(0, 0)), "no factory building");
    assert!(buildings_at(&w, c(0, 0)).is_empty());
    // Beta's factory does not make Alpha's buildings.
    put(&mut w, "YARD", 1, c(-6, -6));
    assert!(!place(&mut w, "HUT", c(0, 0)));
    put(&mut w, "YARD", 0, c(-6, 6));
    assert!(place(&mut w, "HUT", c(0, 0)));
    // LONE's countries share none with YARD's.
    assert!(!place(&mut w, "LONE", c(2, 0)));
}

// spec: S-0406 claim 1 (a factory building being sold already counts as unable to make)
#[test]
fn claim_1_factory_being_sold_cannot_make() {
    let r = rules();
    let mut w = game(&r);
    let y = put(&mut w, "YARD", 0, c(-6, -6));
    let t = w.tick;
    w.obj_mut(y).unwrap().beh.force(Behaviour::Selling, t);
    assert!(!place(&mut w, "HUT", c(0, 0)));
}

// spec: S-0406 claim 1 and F4 (placed at the centre of the named cell, (256x + 128, 256y + 128), at the surface height; construction begins)
// spec: S-0108 claim 5 (the construction state begins)
#[test]
fn claim_1_placement_point_and_construction() {
    let r = rules();
    let mut w = with_yard(&r);
    let at = c(1, 2);
    assert!(place(&mut w, "BIG", at));
    let id = buildings_at(&w, at)[0];
    let o = w.obj(id).unwrap();
    assert_eq!(o.pos.x, 256 * at.x + 128);
    assert_eq!(o.pos.y, 256 * at.y + 128);
    assert_eq!(o.pos.z, w.surface_at(o.pos.x, o.pos.y));
    assert_eq!(o.beh.current, Some(Behaviour::Construction));
    assert!(o.econ.construction.building_up);
    assert!(o.on_map);
    // Every cell of the 2 × 2 rectangle holds it.
    for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
        assert_eq!(
            buildings_at(&w, CellXY::new(at.x + dx, at.y + dy)),
            vec![id]
        );
    }
}

// spec: S-0108 claim 5 (the build-up is finished (n − 1) × d_b ticks after it began, d_b = trunc(B × 900 / n): n = 5, B = 0.05 give 36)
// spec: S-0406 claim 1 (construction completes; request Q-0130: at the first processing on or after that tick, then guard)
#[test]
fn construction_completes_after_buildup() {
    let r = rules();
    let mut w = with_yard(&r);
    assert_eq!(glasnost_sim::econ::sell::buildup_ticks(&w, ty(&w, "HUT")), 36);
    let start = w.tick;
    assert!(place(&mut w, "HUT", c(0, 0)));
    let id = buildings_at(&w, c(0, 0))[0];
    let mut done = None;
    for _ in 0..60 {
        glasnost_sim::tick::step(&mut w);
        let o = w.obj(id).unwrap();
        if !o.econ.construction.building_up {
            done = Some(w.tick - 1);
            assert_eq!(o.beh.current, Some(Behaviour::Guard));
            break;
        }
        assert_eq!(o.beh.current, Some(Behaviour::Construction));
    }
    let done = done.expect("construction completes");
    assert!(done - start >= 36, "not before the build-up ends");
    assert!(done - start <= 37, "at the first processing after it");
}

// spec: S-0406 claim 2 (the 8-cell 3 × 3 word lacks the cell 2 along x and 1 along y from the top-left)
#[test]
fn claim_2_refinery_foundation() {
    let r = rules();
    let mut w = with_yard(&r);
    let cells = placement::foundation_cells(&w, ty(&w, "REFY"), c(0, 0));
    assert_eq!(cells.len(), 8);
    assert!(!cells.contains(&c(2, 1)));
    // A unit in the missing cell does not refuse the placement.
    testkit::spawn(&mut w, Kind::Vehicle, "CART", 0, c(2, 1), 0);
    assert!(place(&mut w, "REFY", c(0, 0)));
    assert!(buildings_at(&w, c(2, 1)).is_empty());
    assert_eq!(buildings_at(&w, c(1, 1)).len(), 1);
}

// spec: S-0406 claim 3 b (a unit of the placing house in a foundation cell blocks the placement; it is not moved away)
// spec: S-0406 claim 3 b (a building in the cell refuses)
#[test]
fn claim_3b_nothing_in_the_way() {
    let r = rules();
    let mut w = with_yard(&r);
    let v = testkit::spawn(&mut w, Kind::Vehicle, "CART", 0, c(1, 1), 0);
    assert!(!place(&mut w, "BIG", c(0, 0)));
    assert_eq!(w.obj(v).unwrap().pos.cell(), c(1, 1));
    testkit::spawn(&mut w, Kind::Infantry, "WALKER", 1, c(4, 0), 0);
    assert!(!place(&mut w, "HUT", c(4, 0)));
    put(&mut w, "HUT", 1, c(-2, 0));
    assert!(!place(&mut w, "HUT", c(-2, 0)));
    assert!(place(&mut w, "HUT", c(-2, 2)));
}

// spec: S-0406 claim 3 c (an overlay refuses: ore, a wall of another house)
#[test]
fn claim_3c_overlay_refuses() {
    let r = rules();
    let mut w = with_yard(&r);
    set_overlay(&mut w, c(0, 0), 5, 3, None);
    assert!(!place(&mut w, "HUT", c(0, 0)));
    set_overlay(&mut w, c(2, 0), 0, 0, Some(B));
    assert!(!place(&mut w, "HUT", c(2, 0)));
    assert!(!place(&mut w, "TOWER", c(2, 0)), "a wall of another house");
}

// spec: S-0406 claim 3 d (slope shape 0 and the buildable flag of the land type)
#[test]
fn claim_3d_ground() {
    let r = rules();
    let mut w = with_yard(&r);
    w.cells.grid.get_mut(c(0, 0)).unwrap().slope = 2;
    assert!(!place(&mut w, "HUT", c(0, 0)));
    w.cells.grid.get_mut(c(2, 0)).unwrap().land = land::WATER;
    assert!(!place(&mut w, "HUT", c(2, 0)));
    assert!(!place(&mut w, "BIG", c(1, -1)), "one cell refusing refuses all");
    assert!(place(&mut w, "HUT", c(4, 0)));
}

// spec: S-0406 claim 3 (the place-anywhere flag: none of (a) to (d) is asked)
#[test]
fn claim_3_place_anywhere() {
    let r = rules();
    let mut w = with_yard(&r);
    set_overlay(&mut w, c(0, 0), 5, 3, None);
    w.cells.grid.get_mut(c(0, 0)).unwrap().slope = 2;
    assert!(place(&mut w, "ANY", c(0, 0)));
}

// spec: S-0406 claim 3 (to-tile rule: one foundation cell passing is enough; its tile outside the tile range passes the tileset test)
#[test]
fn claim_3_to_tile_one_cell() {
    let r = rules();
    let mut w = with_yard(&r);
    let t = ty(&w, "TILE");
    // Both cells of the 2 × 1 rectangle hold tile 0, inside the catalogue's range: the tileset test refuses both.
    assert!(!placement::cell_test(&w, t, c(0, 0), &placement::Test::order(A)));
    w.cells.grid.get_mut(c(1, 0)).unwrap().tile = 9999;
    // A unit in the passing cell does not matter under the to-tile rule.
    testkit::spawn(&mut w, Kind::Vehicle, "CART", 0, c(1, 0), 0);
    assert!(placement::cell_test(&w, t, c(0, 0), &placement::Test::order(A)));
}

// spec: S-0406 claim 4 and F1 (the ring of A + 1 cells around the foundation rectangle, less the rectangle; own base-normal buildings support)
#[test]
fn claim_4_adjacency_ring() {
    let r = rules();
    let mut w = game(&r);
    put(&mut w, "HUT", 0, c(0, 0));
    let near = ty(&w, "NEAR");
    // A = 0: d = 1, only the touching ring, diagonals included.
    assert!(placement::adjacent(&w, A, near, c(1, 1)));
    assert!(placement::adjacent(&w, A, near, c(-1, 0)));
    assert!(!placement::adjacent(&w, A, near, c(2, 0)));
    // A = 3 (absent-key default): d = 4.
    let hut = ty(&w, "HUT");
    assert!(placement::adjacent(&w, A, hut, c(4, -4)));
    assert!(!placement::adjacent(&w, A, hut, c(5, 0)));
    // Beta's buildings do not support Alpha.
    assert!(!placement::adjacent(&w, B, near, c(1, 1)));
}

// spec: S-0406 claim 4 (any building counts whatever its state: in construction)
#[test]
fn claim_4_support_in_construction() {
    let r = rules();
    let mut w = with_yard(&r);
    assert!(place(&mut w, "HUT", c(0, 0)));
    assert!(placement::adjacent(&w, A, ty(&w, "NEAR"), c(1, 0)));
}

// spec: S-0406 claim 5 (every cell of the foundation rectangle explored by the placing house; a to-tile type is exempt)
#[test]
fn claim_5_explored_ground() {
    let r = rules();
    let mut w = game(&r);
    put(&mut w, "HUT", 0, c(0, 0));
    let hut = ty(&w, "HUT");
    assert!(placement::explored(&w, A, hut, c(1, 0)));
    assert!(!placement::explored(&w, B, hut, c(1, 0)));
    let far = c(-20, -20);
    assert!(!placement::explored(&w, A, hut, far));
    assert!(placement::explored(&w, A, ty(&w, "TILE"), far));
    // place_legal: adjacency and shroud together.
    assert!(placement::place_legal(&w, A, tr(&w, "HUT"), c(1, 0)));
    assert!(!placement::place_legal(&w, B, tr(&w, "HUT"), c(1, 0)));
}

// spec: S-0406 claim 6 (claims 4 and 5 are not judged again when the order is applied)
#[test]
fn claim_6_not_judged_at_application() {
    let r = rules();
    let mut w = with_yard(&r);
    // No support near and an unexplored cell: the application still places.
    let far = c(-9, 9);
    let hut = ty(&w, "HUT");
    assert!(!placement::explored(&w, A, hut, far));
    assert!(!placement::adjacent(&w, A, hut, far));
    assert!(place(&mut w, "HUT", far));
}

// spec: S-0406 claim 7 and F2 (the click names (x − ⌊(W − 1)/2⌋, y − ⌊(H − 1)/2⌋))
// spec: S-0406 claim 6 (the cursor shroud result: claim 5 for the rectangle the click would name)
#[test]
fn claim_7_click_cell() {
    let r = rules();
    let w = game(&r);
    let k = CellXY::new(10, 10);
    assert_eq!(placement::click_cell(&w, ty(&w, "HUT"), k), k);
    assert_eq!(placement::click_cell(&w, ty(&w, "BIG"), k), k);
    assert_eq!(
        placement::click_cell(&w, ty(&w, "REFY"), k),
        CellXY::new(9, 9)
    );
    assert_eq!(
        placement::cursor_shroud_result(&w, A, ty(&w, "REFY"), c(1, 1)),
        placement::explored(&w, A, ty(&w, "REFY"), c(0, 0))
    );
}

// spec: S-0406 claim 8 (a damaged own wall of the type's overlay, and the wall tower on an own wall of index 0 or 2, are exempt from (c) and (d))
// spec: S-0406 claim 8 (an undamaged own wall refuses its own wall type)
#[test]
fn claim_8_wall_exceptions() {
    let r = rules();
    let mut w = with_yard(&r);
    let test = placement::Test::order(A);
    set_overlay(&mut w, c(0, 0), 0, 0, Some(A));
    assert!(!placement::cell_test(&w, ty(&w, "WALLA"), c(0, 0), &test));
    assert!(placement::cell_test(&w, ty(&w, "TOWER"), c(0, 0), &test));
    assert!(placement::cell_test(&w, ty(&w, "GATE1"), c(0, 0), &test));
    assert!(!placement::cell_test(&w, ty(&w, "HUT"), c(0, 0), &test));
    // Damaged (data value 16 and above): its own wall type is exempt.
    set_overlay(&mut w, c(0, 0), 0, 16, Some(A));
    assert!(placement::cell_test(&w, ty(&w, "WALLA"), c(0, 0), &test));
    // Overlay 26 accepts neither the tower nor the first gate pair.
    set_overlay(&mut w, c(2, 0), 26, 0, Some(A));
    assert!(!placement::cell_test(&w, ty(&w, "TOWER"), c(2, 0), &test));
    assert!(!placement::cell_test(&w, ty(&w, "GATE1"), c(2, 0), &test));
}

// spec: S-0406 claim 9 (the wall tower removes the own wall of index 0 or 2 in its top-left cell: a silent wall sale)
// spec: S-0406 claim 1 (the tower then stands in the cell)
#[test]
fn claim_9_tower_removes_wall() {
    let r = rules();
    let mut w = with_yard(&r);
    set_overlay(&mut w, c(0, 0), 0, 0, Some(A));
    w.effects.clear();
    let credits = w.houses[0].credits;
    assert!(place(&mut w, "TOWER", c(0, 0)));
    assert_eq!(overlay(&w, c(0, 0)).0, glasnost_map::NO_OVERLAY);
    let i = w.cells.idx(c(0, 0)).unwrap();
    assert_eq!(w.cells.wall_owner[i], None);
    assert_eq!(buildings_at(&w, c(0, 0)).len(), 1);
    assert!(w.effects.is_empty(), "no sound for a silent wall sale");
    assert_eq!(w.houses[0].credits, credits, "no refund");
}

// spec: S-0406 claim 9 and S-0408 claim 12 (the wall stays when its overlay has no builder; the placement goes on)
#[test]
fn claim_9_wall_without_builder_stays() {
    let r = rules();
    let mut w = with_yard(&r);
    // No building type has O2 as its wall overlay.
    set_overlay(&mut w, c(0, 0), 2, 0, Some(A));
    assert!(place(&mut w, "TOWER", c(0, 0)));
    assert_eq!(overlay(&w, c(0, 0)).0, 2);
    assert_eq!(buildings_at(&w, c(0, 0)).len(), 1);
}

// spec: S-0406 claim 10 (a wall type becomes its wall overlay, owned by the placing house; no building remains)
// spec: S-0406 claim 10 (on a damaged own wall the cell ends with an undamaged wall)
#[test]
fn claim_10_wall_becomes_overlay() {
    let r = rules();
    let mut w = with_yard(&r);
    let before = w.objects.len();
    assert!(place(&mut w, "WALLA", c(0, 0)));
    assert_eq!(w.objects.len(), before);
    assert_eq!(overlay(&w, c(0, 0)).0, 0);
    let i = w.cells.idx(c(0, 0)).unwrap();
    assert_eq!(w.cells.wall_owner[i], Some(A));
    set_overlay(&mut w, c(4, 0), 0, 32, Some(A));
    assert!(place(&mut w, "WALLA", c(4, 0)));
    assert!(overlay(&w, c(4, 0)).1 < 16, "undamaged");
}

// spec: S-0406 claim 10 (the builder of the overlay unsellable: the damaged wall is not removed; working hypothesis, the old wall stays)
#[test]
fn claim_10_unsellable_builder_keeps_damaged_wall() {
    let mut r = rules();
    r.sim.buildings.get_mut("WALLA").unwrap().econ.unsellable = Some(true);
    let mut w = with_yard(&r);
    set_overlay(&mut w, c(0, 0), 0, 32, Some(A));
    assert!(place(&mut w, "WALLA", c(0, 0)));
    assert_eq!(overlay(&w, c(0, 0)), (0, 32));
}

// spec: S-0406 claim 11 and F3 (L = ⌊G / 256⌋ = 3: a gap of up to L − 1 cells to an own wall of the same kind is filled, one creation draw per wall)
// spec: S-0406 claim 11 (a direction with no own wall within c₁ … c_L fills nothing)
#[test]
fn claim_11_wall_fill() {
    let r = rules();
    let mut w = with_yard(&r);
    // East: own wall at c₃, gap of two cells. North: own wall at c₄, beyond L.
    set_overlay(&mut w, c(3, 0), 0, 0, Some(A));
    set_overlay(&mut w, c(0, -4), 0, 0, Some(A));
    // West: another house's wall at c₂.
    set_overlay(&mut w, c(-2, 0), 0, 0, Some(B));
    let mut rng = w.rng.clone();
    assert!(place(&mut w, "WALLA", c(0, 0)));
    for x in [1, 2] {
        assert_eq!(overlay(&w, c(x, 0)).0, 0, "filled at {x}");
        let i = w.cells.idx(c(x, 0)).unwrap();
        assert_eq!(w.cells.wall_owner[i], Some(A));
    }
    for y in [-1, -2, -3] {
        assert_eq!(overlay(&w, c(0, y)).0, glasnost_map::NO_OVERLAY);
    }
    assert_eq!(overlay(&w, c(-1, 0)).0, glasnost_map::NO_OVERLAY);
    // Three creation draws: the item's object and the two filling walls.
    for _ in 0..3 {
        let _ = rng.raw();
    }
    assert_eq!(w.rng, rng);
}

// spec: S-0406 claim 11 (a cell before the nearest wall failing the cell test stops the direction)
#[test]
fn claim_11_fill_stops_at_failing_cell() {
    let r = rules();
    let mut w = with_yard(&r);
    set_overlay(&mut w, c(3, 0), 0, 0, Some(A));
    testkit::spawn(&mut w, Kind::Vehicle, "CART", 1, c(2, 0), 0);
    assert!(place(&mut w, "WALLA", c(0, 0)));
    assert_eq!(overlay(&w, c(1, 0)).0, glasnost_map::NO_OVERLAY);
}

// spec: S-0406 claim 12 and F5 (an upgrade goes on an own building of the named type with room: count + 1, full health, recorded; power counts it)
// spec: S-0406 claim 12 (no room: refused)
#[test]
fn claim_12_upgrade() {
    let r = rules();
    let mut w = with_yard(&r);
    let hut = put(&mut w, "HUT", 0, c(0, 0));
    w.obj_mut(hut).unwrap().health = 50;
    assert!(!place(&mut w, "UPG", c(1, 0)), "no building there");
    let before = w.objects.len();
    assert!(place(&mut w, "UPG", c(0, 0)));
    assert_eq!(w.objects.len(), before, "no building of the upgrade type");
    let o = w.obj(hut).unwrap();
    assert_eq!(o.econ.building.upgrade_count, 1);
    assert_eq!(o.health, 400);
    assert_eq!(o.econ.building.upgrades, vec![ty(&w, "UPG")]);
    glasnost_sim::tick::step(&mut w);
    assert_eq!(power::totals(&w, A).0, 25);
    // The type's upgrade count is 1: no more room.
    assert!(!place(&mut w, "UPG", c(0, 0)));
    // A level field needs a count of 0.
    assert!(!place(&mut w, "LVL2", c(0, 0)));
}

// spec: S-0406 claim 12 and F5 (a level field n: the count raised by n, capped at the type's upgrade count; recorded in position c′)
#[test]
fn claim_12_level_field() {
    let r = rules();
    let mut w = with_yard(&r);
    let hut = put(&mut w, "HUT", 0, c(0, 0));
    assert!(place(&mut w, "LVL2", c(0, 0)));
    let o = w.obj(hut).unwrap();
    assert_eq!(o.econ.building.upgrade_count, 1);
    assert_eq!(o.econ.building.upgrades.len(), 1);
}

// spec: S-0406 claim 12 (an upgrade order that names such a building passes claim 4 whatever its ring holds; claim 5 still asked)
#[test]
fn claim_12_upgrade_legality() {
    let r = rules();
    let mut w = game(&r);
    put(&mut w, "HUT", 0, c(0, 0));
    assert!(placement::place_legal(&w, A, tr(&w, "UPG"), c(0, 0)));
    assert!(!placement::place_legal(&w, B, tr(&w, "UPG"), c(0, 0)));
}
