//! Claim tests for S-0406 (building placement) and the construction state
//! it begins (S-0108 claim 5, S-0406 claims 14 to 18) on synthetic data: invented
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
    w.obj_mut(y).unwrap().beh.force(Behaviour::Selling, t, true);
    assert!(!place(&mut w, "HUT", c(0, 0)));
}

// spec: S-0406 claim 1 (revision 5: a factory building queued to be sold also counts as unable to make)
#[test]
fn claim_1_factory_queued_to_be_sold_cannot_make() {
    let r = rules();
    let mut w = game(&r);
    let y = put(&mut w, "YARD", 0, c(-6, -6));
    w.obj_mut(y).unwrap().beh.queued = Some(Behaviour::Selling);
    assert!(!place(&mut w, "HUT", c(0, 0)));
    w.obj_mut(y).unwrap().beh.queued = None;
    assert!(place(&mut w, "HUT", c(0, 0)));
}

// spec: S-0406 claim 1 and F4 (placed at the centre of the named cell, (256x + 128, 256y + 128), at the surface height; construction begins)
// spec: S-0108 claim 5 (the construction state begins)
// spec: S-0406 claim 17 (at entry: no current behaviour and construction queued, which a rule asking for the behaviour sees)
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
    assert_eq!(o.beh.current, None);
    assert_eq!(o.beh.queued, Some(Behaviour::Construction));
    assert!(!o.beh.ready_mark);
    assert_eq!(placement::seen_behaviour(o), Some(Behaviour::Construction));
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

/// A building of type `name` put on the map at the middle in its
/// construction state, with no other object in play (no other draws), as a
/// place order would put it; returns it and the entry tick *T*. It enters
/// before the turn pass of tick *T* rather than after it, so its first turn
/// is on *T*, not *T* + 1; the ready-mark rule of S-0406 claim 17 counts
/// ticks from entry, so the ticks of F7 are unchanged.
fn lone_placed(w: &mut World, name: &str) -> (ObjectId, i32) {
    let t = ty(w, name);
    let id = glasnost_sim::objects::create(w, Kind::Building, t, Some(A));
    let entry = w.tick;
    placement::put_on_map(w, id, c(0, 0), true);
    (id, entry)
}

/// The ticks of a building's construction (S-0406 F6 and F7).
#[derive(Debug, PartialEq, Eq)]
struct Ticks {
    /// *construction* current by the end of this tick.
    r: i32,
    /// The first construction processing.
    c: i32,
    /// The handover.
    h: i32,
}

/// Runs `id` through its construction, checking on the way that nothing is
/// current and construction queued until *R* (S-0406 claim 17), claim 14's
/// cadence (one processing a tick, no draw of the scenario stream) from *C*,
/// and that guard is current by the end of the handover tick.
fn construction_ticks(w: &mut World, id: ObjectId) -> Ticks {
    let mut r = None;
    let mut first = None;
    let mut last = w.obj(id).unwrap().beh.last;
    for _ in 0..300 {
        let rng = w.rng.clone();
        let t = w.tick;
        glasnost_sim::tick::step(w);
        let o = w.obj(id).unwrap();
        // spec: S-0406 claim 17 (no step before C takes a draw)
        assert_eq!(w.rng, rng, "no draw at {t}");
        if first.is_none() {
            first = o.econ.construction.first;
            if let Some(c) = first {
                assert_eq!(c, t);
            }
        }
        if !o.econ.construction.building_up {
            assert_eq!(o.beh.current, Some(Behaviour::Guard));
            return Ticks {
                r: r.expect("construction current"),
                c: first.expect("a first processing"),
                h: t,
            };
        }
        if r.is_none() {
            if o.beh.current == Some(Behaviour::Construction) {
                r = Some(t);
            } else {
                // spec: S-0406 claim 17 (no current behaviour and construction queued until it starts)
                assert_eq!(o.beh.current, None, "at {t}");
                assert_eq!(o.beh.queued, Some(Behaviour::Construction), "at {t}");
            }
        }
        if first.is_some() {
            assert_eq!(o.beh.current, Some(Behaviour::Construction));
            // spec: S-0406 claim 14 (consecutive processings)
            assert_ne!(o.beh.last, last, "processed at {t}");
        }
        last = o.beh.last;
    }
    panic!("construction does not complete");
}

// spec: S-0108 claim 5 (d_b = trunc(B × 900 / n): n = 5, B = 0.05 give 9, finished 36 ticks after)
// spec: S-0406 claims 14 and 15 and F6 (delay 1 and no draw whatever the rate; H = C + (n − 1) × d_b; guard current by the end of the handover tick)
// spec: S-0406 claim 17 and F7 (placed: R = T + (n − 1) × d_b, C = R + 1, H = T + 2(n − 1) × d_b + 1; the build-up runs twice)
#[test]
fn claims_14_15_construction_handover() {
    for rate in ["0.016", "2"] {
        let mut r = rules();
        r.sim.behaviours.insert(
            "construction".into(),
            glasnost_rules::sim::BehaviourProps {
                rate: Some(Decimal(rate.into())),
                ..Default::default()
            },
        );
        let mut w = game(&r);
        assert_eq!(
            glasnost_sim::econ::sell::buildup_ticks(&w, ty(&w, "HUT")),
            36
        );
        let (id, t) = lone_placed(&mut w, "HUT");
        let k = construction_ticks(&mut w, id);
        assert_eq!(
            k,
            Ticks {
                r: t + 36,
                c: t + 37,
                h: t + 73
            },
            "rate {rate}"
        );
    }
}

// spec: S-0406 claim 15 and F6 (a type with no build-up sprite: a zero-delay range, H = C + 1)
// spec: S-0406 claim 17 and F7 (placed, zero-delay range: the mark at the first turn after entry, C = T + 2, H = T + 3)
#[test]
fn claim_15_no_buildup_sprite() {
    let mut r = rules();
    r.sim.buildings.get_mut("HUT").unwrap().econ.buildup = None;
    let mut w = game(&r);
    let (id, t) = lone_placed(&mut w, "HUT");
    let k = construction_ticks(&mut w, id);
    assert_eq!(
        k,
        Ticks {
            r: t + 1,
            c: t + 2,
            h: t + 3
        }
    );
}

// spec: S-0406 Edge cases (quirks.placed_buildup_twice off: the place order sets the ready mark at entry; construction current at the first turn, the build-up seen once, H by F6)
#[test]
fn claim_17_placed_buildup_twice_off() {
    let mut r = rules();
    r.globals.quirks.placed_buildup_twice = false;
    let mut w = game(&r);
    let (id, t) = lone_placed(&mut w, "HUT");
    assert!(w.obj(id).unwrap().beh.ready_mark);
    let k = construction_ticks(&mut w, id);
    // The first turn is on T here (see lone_placed): C = T + 1, H = C + 36.
    assert_eq!(
        k,
        Ticks {
            r: t,
            c: t + 1,
            h: t + 37
        }
    );
}

/// HUT with a build-up sprite of 3 image frames: n = 1, d_b = trunc(0.05
/// × 900 / 1) = 45 (S-0108 claim 5).
fn single_stage() -> Ruleset {
    let mut r = rules();
    r.sim.buildings.get_mut("HUT").unwrap().econ.buildup_frames = Some(3);
    r
}

// spec: S-0406 claim 18 (quirks.single_stage_buildup_never_completes: a placed building with a positive-delay one-stage range keeps construction queued for good; never constructed, never guard, no sell)
#[test]
fn claim_18_single_stage_never_completes() {
    for frames in [3, 1, 0] {
        let mut r = single_stage();
        r.sim.buildings.get_mut("HUT").unwrap().econ.buildup_frames = Some(frames);
        let mut w = game(&r);
        let (id, _) = lone_placed(&mut w, "HUT");
        testkit::run(&mut w, 400);
        let o = w.obj(id).unwrap();
        assert_eq!(o.beh.current, None, "frames {frames}");
        assert_eq!(o.beh.queued, Some(Behaviour::Construction));
        assert!(o.econ.construction.building_up);
        assert!(!glasnost_sim::econ::sell::sellable(&w, id));
    }
}

// spec: S-0406 Edge cases (quirks.single_stage_buildup_never_completes off: the report d_b ticks after the range starts; a placed building reaches construction at T + d_b and hands over d_b ticks after C)
#[test]
fn claim_18_switch_off() {
    let mut r = single_stage();
    r.globals.quirks.single_stage_buildup_never_completes = false;
    let mut w = game(&r);
    let (id, t) = lone_placed(&mut w, "HUT");
    let k = construction_ticks(&mut w, id);
    assert_eq!(
        k,
        Ticks {
            r: t + 45,
            c: t + 46,
            h: t + 91
        }
    );
}

// spec: S-0406 claim 15 (a gate uses its gate range: n = G + 1, GateStages absent 9; S-0108 claim 5)
#[test]
fn claim_15_gate_range() {
    let mut r = rules();
    let w = with_yard(&r);
    // n = 10, d_b = trunc(0.05 × 900 / 10) = 4: 9 × 4.
    assert_eq!(
        glasnost_sim::econ::sell::buildup_ticks(&w, ty(&w, "GATE1")),
        36
    );
    r.sim.buildings.get_mut("GATE1").unwrap().econ.gate_stages = Some(3);
    let w = with_yard(&r);
    // n = 4, d_b = trunc(0.05 × 900 / 4) = 11: 3 × 11.
    assert_eq!(
        glasnost_sim::econ::sell::buildup_ticks(&w, ty(&w, "GATE1")),
        33
    );
}

// spec: S-0406 claim 15 (the handover does not clear the building's target)
// spec: S-0309 claim 29 (a weaponless holder drops its target at a check tick, so none falls in the construction)
#[test]
fn claim_15_target_kept() {
    // The weaponless HUT answers *impossible* (S-0301 claim 9) and drops a
    // held target at a check tick (S-0309 claim 29 r94), so the short
    // construction of a type with no build-up sprite (H = T + 3, F6 and
    // F7) is placed just after a check tick, with none before the handover.
    let mut r = rules();
    r.sim.buildings.get_mut("HUT").unwrap().econ.buildup = None;
    let mut w = game(&r);
    while w.tick % 16 != 1 {
        testkit::run(&mut w, 1);
    }
    let (id, t) = lone_placed(&mut w, "HUT");
    let aim = glasnost_sim::world::Target::Cell(c(4, 4));
    w.obj_mut(id).unwrap().target = Some(aim);
    let k = construction_ticks(&mut w, id);
    assert_eq!(k.h, t + 3);
    assert!((t..=k.h).all(|n| n % 16 != 0), "no check tick");
    assert_eq!(w.obj(id).unwrap().target, Some(aim));
}

// spec: S-0406 claim 16 (a placed building faces 0, its turret too, whatever the factory's facing)
#[test]
fn claim_16_placed_facing() {
    let r = rules();
    let mut w = game(&r);
    testkit::spawn(&mut w, Kind::Building, "YARD", 0, c(-6, -6), 96);
    assert!(place(&mut w, "HUT", c(0, 0)));
    let id = buildings_at(&w, c(0, 0))[0];
    let o = w.obj(id).unwrap();
    assert_eq!((o.body.start, o.body.target), (0, 0));
    assert_eq!(o.turret.target, 0);
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
    assert!(
        !place(&mut w, "BIG", c(1, -1)),
        "one cell refusing refuses all"
    );
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
    assert!(!placement::cell_test(
        &w,
        t,
        c(0, 0),
        &placement::Test::order(A)
    ));
    w.cells.grid.get_mut(c(1, 0)).unwrap().tile = 9999;
    // A unit in the passing cell does not matter under the to-tile rule.
    testkit::spawn(&mut w, Kind::Vehicle, "CART", 0, c(1, 0), 0);
    assert!(placement::cell_test(
        &w,
        t,
        c(0, 0),
        &placement::Test::order(A)
    ));
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

// spec: S-0406 claim 5 and PL6 e (the 8-cell 3 × 3 word: the shroud test includes the rectangle's missing cell (2, 1))
#[test]
fn claim_5_missing_cell_is_shroud_tested() {
    let r = rules();
    let mut w = game(&r);
    let refy = ty(&w, "REFY");
    let at = |dx: i32, dy: i32| c(-9 + dx, 6 + dy);
    let p = glasnost_sim::world::Pos::centre(at(1, 1), 0);
    glasnost_sim::econ::shroud::area_reveal(&mut w, A, p, 4);
    assert!(placement::explored(&w, A, refy, at(0, 0)));
    // Only the missing cell (2, 1) is shrouded again; the eight covered cells stay explored.
    let i = w.cells.idx(at(2, 1)).expect("on the map");
    let marks = w.econ.shroud.houses[usize::from(A.0)]
        .as_mut()
        .expect("player house");
    let kept = marks.marks[i];
    marks.marks[i] = 0;
    for cell in placement::foundation_cells(&w, refy, at(0, 0)) {
        assert!(glasnost_sim::econ::shroud::explored(&w, A, cell));
    }
    assert!(!glasnost_sim::econ::shroud::explored(&w, A, at(2, 1)));
    assert!(!placement::explored(&w, A, refy, at(0, 0)));
    w.econ.shroud.houses[usize::from(A.0)]
        .as_mut()
        .expect("player house")
        .marks[i] = kept;
    assert!(placement::explored(&w, A, refy, at(0, 0)));
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

// spec: S-0406 claim 9 (a gate-flag type: the own wall of index 0, 2 or 26 in each foundation cell, by the silent wall sale, and own laser-fence buildings, removed without refund)
#[test]
fn claim_9_gate_row() {
    let mut r = rules();
    let gate = r.sim.buildings.get_mut("GATE1").unwrap();
    gate.foundation = Some([2, 1]);
    let mut fence = building(1, 1);
    fence.laser_fence = Some(true);
    fence.cost = Some(500);
    r.registries
        .get_mut("buildings")
        .unwrap()
        .push("FENCE".into());
    r.sim.buildings.insert("FENCE".into(), fence);
    let mut w = with_yard(&r);
    // Left cell: an own wall of O0 (WALLA builds it). Right cell: an own laser-fence building.
    set_overlay(&mut w, c(0, 0), 0, 0, Some(A));
    let f = put(&mut w, "FENCE", 0, c(1, 0));
    let credits = w.houses[0].credits;
    w.effects.clear();
    assert!(place(&mut w, "GATE1", c(0, 0)));
    assert_eq!(overlay(&w, c(0, 0)).0, glasnost_map::NO_OVERLAY);
    assert!(
        w.obj(f).is_none(),
        "the own laser-fence building is removed"
    );
    assert_eq!(w.houses[0].credits, credits, "no refund");
    assert!(w.effects.is_empty(), "no sound");
}

// spec: S-0406 Edge cases (quirks.wall_exception_skips_ground: on, a wall exception skips (d); off, (d) is still asked)
#[test]
fn quirk_wall_exception_skips_ground_off() {
    let mut r = rules();
    let mut w = with_yard(&r);
    let test = placement::Test::order(A);
    set_overlay(&mut w, c(0, 0), 0, 0, Some(A));
    w.cells.grid.get_mut(c(0, 0)).unwrap().slope = 2;
    assert!(placement::cell_test(&w, ty(&w, "TOWER"), c(0, 0), &test));
    r.globals.quirks.wall_exception_skips_ground = false;
    let mut w = with_yard(&r);
    set_overlay(&mut w, c(0, 0), 0, 0, Some(A));
    w.cells.grid.get_mut(c(0, 0)).unwrap().slope = 2;
    assert!(!placement::cell_test(&w, ty(&w, "TOWER"), c(0, 0), &test));
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
    assert_eq!(o.econ.building.upgrades, [Some(ty(&w, "UPG")), None, None]);
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
    assert_eq!(o.econ.building.upgrades, [Some(ty(&w, "LVL2")), None, None]);
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

mod econ_common;

// spec: S-0406 claim 1 (a place order on a done item puts the item's object on the map in its construction state; the line ends, S-0404 claim 7)
// spec: S-0406 claim 1 (a failed placement leaves the item done)
#[test]
fn claim_1_place_order_through_production() {
    use econ_common as e;
    let mut r = e::ruleset();
    r.sim.land_types.insert(
        LandType::Clear,
        LandRow {
            buildable: Some(true),
            ..Default::default()
        },
    );
    r.sim.buildings.get_mut("PLANT").unwrap().cost = Some(50);
    let mut w = e::game(&r);
    e::building(&mut w, "CONYARD", 0, -6, 0);
    e::ready(&mut w);
    e::set_credits(&mut w, 0, 1_000_000);
    let p = Pos::centre(testkit::mid(e::SIZE, 0, 0), 0);
    glasnost_sim::econ::shroud::area_reveal(&mut w, A, p, 10);
    let plant = e::t(&w, Kind::Building, "PLANT");
    assert!(e::issue(
        &mut w,
        0,
        glasnost_sim::orders::Order::Build { what: plant }
    ));
    e::run_to_done(&mut w, 0, plant, 400).expect("done");
    let obj = e::item(&w, 0, plant).unwrap().object;
    // A cell under the yard fails the cell test: the item stays done.
    e::force(
        &mut w,
        0,
        glasnost_sim::orders::Order::Place {
            what: plant,
            cell: Some(testkit::mid(e::SIZE, -6, 0)),
        },
    );
    w.effects.clear();
    e::step(&mut w);
    assert!(e::item(&w, 0, plant).is_some_and(|i| i.done_tick.is_some()));
    // spec: S-0404 claim 7 (placement refused by its cell tests as it is applied: the cannot-deploy-here announcement for its house)
    {
        use glasnost_sim::combat::effects::{Effect, Sound};
        assert!(w.effects.iter().any(|x| matches!(
            x,
            Effect::Sound { sound: Sound::CannotDeploy, house: Some(h), .. } if *h == A
        )));
    }
    let at = testkit::mid(e::SIZE, -3, 0);
    assert!(placement::place_legal(&w, A, plant, at));
    let objects = w.objects.len();
    assert!(e::issue(
        &mut w,
        0,
        glasnost_sim::orders::Order::Place {
            what: plant,
            cell: Some(at),
        }
    ));
    e::step(&mut w);
    assert_eq!(w.objects.len(), objects, "the item's object, no new one");
    let o = w.obj(obj).unwrap();
    assert!(o.on_map);
    assert_eq!(o.pos.cell(), at);
    // spec: S-0406 claim 17 (at entry: construction queued, none current)
    assert_eq!(o.beh.current, None);
    assert_eq!(placement::seen_behaviour(o), Some(Behaviour::Construction));
    assert!(e::line(&w, 0, plant).is_none());
}
