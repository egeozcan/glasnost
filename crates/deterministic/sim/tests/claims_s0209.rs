//! Claim tests for cell occupancy and crowding (S-0209): capacity, entry
//! outcomes for walkers, vehicles and aircraft, a walker's reactions to a
//! blocked cell and cell-wide scatter requests, on synthetic data.

mod common;

use common::{FOOT, Scene, TRACK, cell, open_orders, order_move, run, run_until, unit_type};
use glasnost_map::CellXY;
use glasnost_map::objects::{Aircraft, Placed};
use glasnost_rules::sim::{
    BuildingType, LandRow, LandType, LocomotionClass, MovementZone, TerrainObjectType, UnitType,
    WarheadType, WeaponType,
};
use glasnost_sim::cells::position;
use glasnost_sim::occupancy::{self, Outcome};
use glasnost_sim::rules::Kind;
use glasnost_sim::world::{HouseId, Layer, ObjectId, Pos, Target, World};
use glasnost_sim::{crush, objects, path, scatter, tick};

// ---------------------------------------------------------------------------
// Scene helpers (invented types and values).

/// A walker type; armed ones carry the invented weapon "Pea".
fn walker(armed: bool) -> UnitType {
    UnitType {
        primary: armed.then(|| "Pea".to_string()),
        ..unit_type(FOOT, 6, 10)
    }
}

/// A vehicle type; armed ones carry "Pea", crushers crush.
fn vehicle(armed: bool, crusher: bool) -> UnitType {
    UnitType {
        primary: armed.then(|| "Pea".to_string()),
        crusher: Some(crusher),
        ..unit_type(TRACK, 6, 10)
    }
}

/// The base scene: invented unit, building, terrain and weapon types.
fn scene() -> Scene {
    let mut s = Scene::new();
    s.infantry("Mope", walker(false))
        .infantry("Pike", walker(true))
        .infantry(
            "Squish",
            UnitType {
                crushable: Some(true),
                ..walker(true)
            },
        )
        .vehicle("Cart", vehicle(false, false))
        .vehicle("Ram", vehicle(true, false))
        .vehicle("Roller", vehicle(true, true))
        .vehicle(
            "Chopper",
            UnitType {
                primary: Some("Axe".into()),
                ..vehicle(true, false)
            },
        );
    let sim = &mut s.ruleset.sim;
    sim.weapons.insert(
        "Pea".into(),
        WeaponType {
            damage: Some(7),
            warhead: Some("Soft".into()),
            ..Default::default()
        },
    );
    sim.weapons.insert(
        "Axe".into(),
        WeaponType {
            damage: Some(7),
            warhead: Some("Woody".into()),
            ..Default::default()
        },
    );
    sim.warheads.insert("Soft".into(), WarheadType::default());
    sim.warheads.insert(
        "Woody".into(),
        WarheadType {
            wood: Some(true),
            ..Default::default()
        },
    );
    let b = |f: fn(&mut BuildingType)| {
        let mut t = BuildingType {
            strength: Some(300),
            foundation: Some([1, 1]),
            ..Default::default()
        };
        f(&mut t);
        t
    };
    let buildings = [
        ("Shed", b(|_| {})),
        ("Ghost", b(|t| t.invisible_in_game = Some(true))),
        ("Mender", b(|t| t.bridge_repair_hut = Some(true))),
    ];
    for (n, t) in buildings {
        s.register("buildings", n);
        s.ruleset.sim.buildings.insert(n.into(), t);
    }
    for (n, bits) in [("Moss", 0), ("Bramble", 7)] {
        s.register("terrain_objects", n);
        s.ruleset.sim.terrain_objects.insert(
            n.into(),
            TerrainObjectType {
                temperate_occupation_bits: Some(bits),
                snow_occupation_bits: Some(bits),
            },
        );
    }
    s.register("aircraft", "Kite");
    s.ruleset
        .sim
        .aircraft
        .insert("Kite".into(), unit_type(LocomotionClass::Winged, 10, 10));
    s
}

/// The object of `kind` standing in `c` (the first in ID order).
fn at(w: &World, kind: Kind, c: CellXY) -> ObjectId {
    w.objects
        .values()
        .find(|o| o.kind == kind && o.pos.cell() == c)
        .map(|o| o.id)
        .unwrap_or_else(|| panic!("no {kind:?} in {c:?}"))
}

/// The walker's entry outcome for cell `c`.
fn inf(w: &World, id: ObjectId, c: CellXY) -> Outcome {
    occupancy::infantry_outcome(w, id, c, occupancy::reference_level(w, id))
}

/// The vehicle's entry outcome for cell `c`.
fn veh(w: &World, id: ObjectId, c: CellXY) -> Outcome {
    occupancy::vehicle_outcome(w, id, c, occupancy::reference_level(w, id))
}

/// The mover's cell and the asked cell next to it.
fn here() -> CellXY {
    cell(0, 0)
}
fn there() -> CellXY {
    cell(1, 0)
}

/// Builds a scene with a mover of `kind`/`ty` in `here()` (house 0) and
/// `setup` adding the asked cell's contents; returns the world and mover.
fn asked(kind: Kind, ty: &str, setup: impl FnOnce(&mut Scene)) -> (World, ObjectId) {
    let mut s = scene();
    match kind {
        Kind::Infantry => s.place_infantry(0, ty, here(), 2),
        _ => s.place_unit(0, ty, here(), 64),
    };
    setup(&mut s);
    let w = s.build();
    let m = at(&w, kind, here());
    (w, m)
}

fn ground(w: &World, c: CellXY) -> glasnost_sim::cells::Occupancy {
    *w.cells.occ(c, Layer::Ground).expect("cell on the map")
}

// ---------------------------------------------------------------------------
// Claim 2

// spec: S-0209 claim 2 (full when right, left and bottom are taken; centre and top do not count)
#[test]
fn full_needs_the_three_side_positions() {
    let mut s = scene();
    // Centre (sub-cell 0) and top (sub-cell 1) only.
    s.place_infantry(0, "Mope", there(), 0)
        .place_infantry(0, "Mope", there(), 1);
    s.place_infantry(0, "Mope", here(), 2);
    let mut w = s.build();
    let m = at(&w, Kind::Infantry, here());
    assert!(!ground(&w, there()).full());
    // Five infantry fit: the two there leave all three side positions free.
    assert_eq!(inf(&w, m, there()), Outcome::Clear);
    for p in position::SIDES {
        assert!(!ground(&w, there()).taken(p));
    }
    // Taking the three side positions makes it full.
    for (k, p) in position::SIDES.into_iter().enumerate() {
        w.cells.mark_infantry(
            there(),
            Layer::Ground,
            p,
            ObjectId(9000 + k as u32),
            HouseId(0),
        );
    }
    assert!(ground(&w, there()).full());
}

// spec: S-0209 claim 2 (side positions marked by a terrain object count as taken)
#[test]
fn terrain_marks_fill_side_positions() {
    let mut s = scene();
    s.place_terrain("Bramble", there())
        .place_terrain("Moss", cell(2, 0));
    let w = s.build();
    assert!(ground(&w, there()).full());
    assert!(!ground(&w, cell(2, 0)).full());
}

// spec: S-0209 claim 2 (three walkers on the side positions fill the cell for a fourth)
#[test]
fn three_walkers_fill_a_cell() {
    let (w, m) = asked(Kind::Infantry, "Mope", |s| {
        for sub in 2..=4 {
            s.place_infantry(0, "Mope", there(), sub);
        }
    });
    assert!(ground(&w, there()).full());
    assert!(occupancy::first_free(&w, there(), Layer::Ground, &position::SIDES).is_none());
    assert_ne!(inf(&w, m, there()), Outcome::Clear);
}

// ---------------------------------------------------------------------------
// Claim 3

// spec: S-0209 claim 3 (the vehicle-present flag denies a standing position; with clear contents the walker gets moving blocker)
#[test]
fn vehicle_flag_keeps_walkers_out() {
    let (mut w, m) = asked(Kind::Infantry, "Mope", |_| {});
    assert!(!occupancy::no_position_by_flags(&w, there(), Layer::Ground));
    assert_eq!(inf(&w, m, there()), Outcome::Clear);
    // A vehicle that has started a step into the cell: flag set, not listed.
    w.cells.claim_vehicle(there(), Layer::Ground, true);
    assert!(w.cells.list(there(), Layer::Ground).is_empty());
    assert!(occupancy::no_position_by_flags(&w, there(), Layer::Ground));
    assert_eq!(inf(&w, m, there()), Outcome::MovingBlocker);
}

// spec: S-0209 claim 3 (an idle vehicle of either house sets the flag that keeps walkers off its cell)
#[test]
fn idle_vehicle_cell_has_no_position() {
    let (w, _) = asked(Kind::Infantry, "Mope", |s| {
        s.place_unit(1, "Cart", there(), 0);
    });
    assert!(ground(&w, there()).vehicle_present());
    assert!(occupancy::no_position_by_flags(&w, there(), Layer::Ground));
}

// spec: S-0209 claim 3 (the building-present flag does not deny a standing position; an invisible building is passed over)
#[test]
fn building_flag_does_not_keep_walkers_out() {
    let (w, m) = asked(Kind::Infantry, "Mope", |s| {
        s.place_structure(1, "Ghost", there());
    });
    assert_eq!(
        w.cells.list(there(), Layer::Ground).len(),
        1,
        "building placed"
    );
    assert!(ground(&w, there()).buildings > 0);
    assert!(!occupancy::no_position_by_flags(&w, there(), Layer::Ground));
    assert_eq!(inf(&w, m, there()), Outcome::Clear);
}

// spec: S-0209 claim 3 (the object-present flag denies a standing position)
#[test]
fn object_flag_keeps_walkers_out() {
    let (mut w, _) = asked(Kind::Infantry, "Mope", |_| {});
    w.cells.occ_mut(there(), Layer::Ground).unwrap().others = 1;
    assert!(occupancy::no_position_by_flags(&w, there(), Layer::Ground));
}

// ---------------------------------------------------------------------------
// Claim 4

// spec: S-0209 claim 4 (rows WP16 and WP22/WP28: allied idle vehicle stationary friendly; enemy vehicle blocked or enemy obstacle by arming)
#[test]
fn walker_vehicle_rows() {
    let (w, m) = asked(Kind::Infantry, "Mope", |s| {
        s.place_unit(0, "Cart", there(), 0);
        s.place_unit(1, "Cart", cell(0, 1), 0);
        s.place_infantry(0, "Pike", cell(1, 1), 2);
    });
    let armed = at(&w, Kind::Infantry, cell(1, 1));
    assert_eq!(inf(&w, m, there()), Outcome::StationaryFriendly);
    assert_eq!(inf(&w, m, cell(0, 1)), Outcome::Blocked);
    assert_eq!(inf(&w, armed, cell(0, 1)), Outcome::EnemyObstacle);
}

// spec: S-0209 claim 4 (rows WP19, WP22 to WP24 and WP7: buildings by alliance, arming, repair hut and invisibility)
#[test]
fn walker_building_rows() {
    let (w, m) = asked(Kind::Infantry, "Pike", |s| {
        s.place_structure(0, "Shed", there());
        s.place_structure(1, "Shed", cell(1, 1));
        s.place_structure(1, "Mender", cell(0, 1));
        s.place_structure(1, "Ghost", cell(-1, 0));
        s.place_infantry(0, "Mope", cell(-1, 1), 2);
    });
    let unarmed = at(&w, Kind::Infantry, cell(-1, 1));
    assert_eq!(inf(&w, m, there()), Outcome::Blocked, "allied building");
    assert_eq!(inf(&w, m, cell(1, 1)), Outcome::EnemyObstacle);
    assert_eq!(inf(&w, unarmed, cell(1, 1)), Outcome::Blocked);
    assert_eq!(inf(&w, m, cell(0, 1)), Outcome::Blocked, "repair hut");
    assert_eq!(inf(&w, m, cell(-1, 0)), Outcome::Clear, "invisible");
}

// spec: S-0209 claim 4 (rows WR2 and WR3: an enemy infantry house blocks an unarmed walker, gives an armed one enemy obstacle)
#[test]
fn walker_enemy_infantry_house() {
    let (w, m) = asked(Kind::Infantry, "Mope", |s| {
        s.place_infantry(1, "Mope", there(), 2);
        s.place_infantry(0, "Pike", cell(0, 1), 2);
    });
    let armed = at(&w, Kind::Infantry, cell(0, 1));
    assert_eq!(ground(&w, there()).house, Some(HouseId(1)));
    assert_eq!(inf(&w, m, there()), Outcome::Blocked);
    assert_eq!(inf(&w, armed, there()), Outcome::EnemyObstacle);
}

// spec: S-0209 claim 4 (row WR3 holds for a reservation without a listed occupant)
#[test]
fn walker_enemy_reservation() {
    let (mut w, m) = asked(Kind::Infantry, "Pike", |_| {});
    w.cells.mark_infantry(
        there(),
        Layer::Ground,
        position::RIGHT,
        ObjectId(9999),
        HouseId(1),
    );
    assert_eq!(inf(&w, m, there()), Outcome::EnemyObstacle);
}

// spec: S-0209 claim 4 (rows WR4 and WR5: exactly three still allies give stationary friendly, a fourth gives moving blocker)
#[test]
fn walker_full_allied_cell() {
    let (w, m) = asked(Kind::Infantry, "Mope", |s| {
        for sub in 2..=4 {
            s.place_infantry(0, "Mope", there(), sub);
        }
        for sub in 0..=4 {
            s.place_infantry(0, "Mope", cell(0, 1), sub);
        }
    });
    assert_eq!(inf(&w, m, there()), Outcome::StationaryFriendly);
    assert_eq!(inf(&w, m, cell(0, 1)), Outcome::MovingBlocker);
}

// spec: S-0209 claim 4 (row WR6: no infantry house and three side positions taken gives blocked)
#[test]
fn walker_terrain_filled_cell() {
    let (w, m) = asked(Kind::Infantry, "Pike", |s| {
        s.place_terrain("Bramble", there());
        s.place_terrain("Moss", cell(0, 1));
    });
    assert_eq!(ground(&w, there()).house, None);
    assert_eq!(inf(&w, m, there()), Outcome::Blocked);
    // Row WP27: a terrain object gives no value of its own to an armed walker.
    assert_eq!(inf(&w, m, cell(0, 1)), Outcome::Clear);
}

// spec: S-0209 claim 4 (row WP20: allied infantry gives no value of its own)
#[test]
fn walker_allied_infantry_none() {
    let (w, m) = asked(Kind::Infantry, "Mope", |s| {
        s.place_infantry(0, "Mope", there(), 2);
    });
    assert_eq!(inf(&w, m, there()), Outcome::Clear);
}

// ---------------------------------------------------------------------------
// Claim 5

// spec: S-0209 claim 5 (rows VR4 and VP1: an empty cell and the vehicle's own cell are clear)
#[test]
fn vehicle_empty_and_own_cell() {
    let (w, m) = asked(Kind::Vehicle, "Cart", |_| {});
    assert_eq!(veh(&w, m, there()), Outcome::Clear);
    assert!(ground(&w, here()).vehicle_present());
    assert_eq!(veh(&w, m, here()), Outcome::Clear);
}

// spec: S-0209 claim 5 (rows VP20, VP26 and VP30: allied idle units stationary friendly; enemy units blocked or enemy obstacle by arming)
#[test]
fn vehicle_unit_rows() {
    let (w, m) = asked(Kind::Vehicle, "Cart", |s| {
        s.place_infantry(0, "Mope", there(), 2);
        s.place_unit(0, "Cart", cell(1, 1), 0);
        s.place_unit(1, "Cart", cell(0, 1), 0);
        s.place_infantry(1, "Mope", cell(-1, 0), 2);
        s.place_unit(0, "Ram", cell(-1, -1), 0);
    });
    let armed = at(&w, Kind::Vehicle, cell(-1, -1));
    assert_eq!(veh(&w, m, there()), Outcome::StationaryFriendly);
    assert_eq!(veh(&w, m, cell(1, 1)), Outcome::StationaryFriendly);
    assert_eq!(veh(&w, m, cell(0, 1)), Outcome::Blocked);
    assert_eq!(veh(&w, m, cell(-1, 0)), Outcome::Blocked);
    assert_eq!(veh(&w, armed, cell(0, 1)), Outcome::EnemyObstacle);
    assert_eq!(veh(&w, armed, cell(-1, 0)), Outcome::EnemyObstacle);
}

// spec: S-0209 claim 5 (rows VP19, VP27, VP28 and VP13: buildings for a vehicle)
#[test]
fn vehicle_building_rows() {
    let (w, m) = asked(Kind::Vehicle, "Ram", |s| {
        s.place_structure(0, "Shed", there());
        s.place_structure(1, "Shed", cell(1, 1));
        s.place_structure(1, "Mender", cell(0, 1));
        s.place_structure(1, "Ghost", cell(-1, 0));
    });
    assert_eq!(veh(&w, m, there()), Outcome::Blocked);
    assert_eq!(veh(&w, m, cell(1, 1)), Outcome::EnemyObstacle);
    assert_eq!(veh(&w, m, cell(0, 1)), Outcome::Blocked);
    assert_eq!(veh(&w, m, cell(-1, 0)), Outcome::Clear);
}

// spec: S-0209 claim 5 (rows VP25 and VR3: crushable enemy infantry is clear for a crusher)
#[test]
fn vehicle_crusher_over_crushable() {
    let (w, m) = asked(Kind::Vehicle, "Roller", |s| {
        s.place_infantry(1, "Squish", there(), 2);
        s.place_infantry(1, "Pike", cell(0, 1), 2);
    });
    assert_eq!(veh(&w, m, there()), Outcome::Clear);
    // Not crushable: row VP30.
    assert_eq!(veh(&w, m, cell(0, 1)), Outcome::EnemyObstacle);
}

// spec: S-0209 claim 5 (row VR2: crushable occupant and the flag set; no crushable vehicle first gives moving blocker)
#[test]
fn vehicle_crusher_with_flag() {
    let (mut w, m) = asked(Kind::Vehicle, "Roller", |s| {
        s.place_infantry(1, "Squish", there(), 2);
    });
    w.cells.claim_vehicle(there(), Layer::Ground, true);
    assert_eq!(veh(&w, m, there()), Outcome::MovingBlocker);
}

// spec: S-0209 claim 5 (row VP29: a tree is an enemy obstacle when the weapon harms wood, blocked otherwise)
#[test]
fn vehicle_terrain_object_row() {
    let (w, m) = asked(Kind::Vehicle, "Ram", |s| {
        s.place_terrain("Moss", there());
        s.place_unit(0, "Chopper", cell(0, 1), 0);
        s.place_unit(0, "Cart", cell(-1, 0), 0);
    });
    let axe = at(&w, Kind::Vehicle, cell(0, 1));
    let cart = at(&w, Kind::Vehicle, cell(-1, 0));
    assert_eq!(veh(&w, m, there()), Outcome::Blocked);
    assert_eq!(veh(&w, axe, there()), Outcome::EnemyObstacle);
    // Row VP26: not armed.
    assert_eq!(veh(&w, cart, there()), Outcome::Blocked);
}

// spec: S-0209 claim 5 (rows VR5 to VR10: the record decides when contents are clear)
#[test]
fn vehicle_record_rows() {
    let (mut w, ram) = asked(Kind::Vehicle, "Ram", |s| {
        s.place_unit(0, "Cart", cell(0, 2), 0);
        s.place_unit(0, "Roller", cell(0, 3), 0);
    });
    let cart = at(&w, Kind::Vehicle, cell(0, 2));
    let roller = at(&w, Kind::Vehicle, cell(0, 3));
    // VR5: flag set, nothing listed.
    w.cells.claim_vehicle(there(), Layer::Ground, true);
    assert_eq!(veh(&w, ram, there()), Outcome::MovingBlocker);
    // A reservation without a listed occupant (a walker that chose its target).
    let c = cell(1, 1);
    w.cells.mark_infantry(
        c,
        Layer::Ground,
        position::RIGHT,
        ObjectId(9999),
        HouseId(0),
    );
    // VR6: allied infantry house.
    assert_eq!(veh(&w, ram, c), Outcome::MovingBlocker);
    let e = cell(-1, 1);
    w.cells.mark_infantry(
        e,
        Layer::Ground,
        position::RIGHT,
        ObjectId(9998),
        HouseId(1),
    );
    // VR7 crusher clear; VR9 no primary blocked; VR10 hits ground: enemy obstacle.
    assert_eq!(veh(&w, roller, e), Outcome::Clear);
    assert_eq!(veh(&w, cart, e), Outcome::Blocked);
    assert_eq!(veh(&w, ram, e), Outcome::EnemyObstacle);
}

// ---------------------------------------------------------------------------
// Walker scenes: a closed corridor along y = 0 from x = -3 to x = 8, walled
// by cells four levels up (a step no unit may take).

fn corridor(s: &mut Scene) {
    for x in -4..=9 {
        s.set_cell(cell(x, -1), 0, 4);
        s.set_cell(cell(x, 1), 0, 4);
    }
    s.set_cell(cell(-4, 0), 0, 4);
    s.set_cell(cell(9, 0), 0, 4);
}

/// A corridor scene with walker `ty` (house 0) at x = -2 and `setup`.
fn corridor_scene(ty: &str, setup: impl FnOnce(&mut Scene)) -> (World, ObjectId) {
    let mut s = scene();
    corridor(&mut s);
    s.place_infantry(0, ty, cell(-2, 0), 0);
    setup(&mut s);
    let mut w = s.build();
    let m = at(&w, Kind::Infantry, cell(-2, 0));
    open_orders(&mut w);
    (w, m)
}

fn walk_target_cell(w: &World, id: ObjectId) -> Option<CellXY> {
    w.obj(id)
        .and_then(|o| o.walk.as_ref())
        .and_then(|s| s.target)
        .map(|p| p.cell())
}

/// Runs until the walker stands in `c` with no walking target, past its
/// arrival tick: the next tick is an asking tick.
fn stand_in(w: &mut World, id: ObjectId, c: CellXY) {
    let n = run_until(w, 300, |w| {
        let o = w.obj(id).unwrap();
        o.pos.cell() == c
            && o.walk
                .as_ref()
                .is_some_and(|s| s.target.is_none() && s.arrived.is_some())
    });
    assert!(n < 300, "walker never stood in {c:?}");
}

fn dest(w: &World, id: ObjectId) -> Option<Target> {
    w.obj(id).unwrap().dest
}

fn steps(w: &World, id: ObjectId) -> Vec<u8> {
    w.obj(id).unwrap().path.steps.clone()
}

// ---------------------------------------------------------------------------
// Claims 6 and 7

/// Open field: a walker from x = -2 to x = 4 along y = 0. With `block`, cell
/// (0, 0) becomes full of terrain marks once the first walking target is
/// chosen and the retry timer is pushed far out. Returns the tick and cell
/// of the second walking target, and the arrival tick before it.
fn second_target(block: bool) -> (i32, CellXY, i32) {
    let mut s = scene();
    s.place_infantry(0, "Mope", cell(-2, 0), 0);
    let mut w = s.build();
    let m = at(&w, Kind::Infantry, cell(-2, 0));
    open_orders(&mut w);
    order_move(&mut w, m, cell(4, 0));
    run_until(&mut w, 100, |w| walk_target_cell(w, m).is_some());
    assert_eq!(walk_target_cell(&w, m), Some(cell(-1, 0)));
    if block {
        let ty = w.rules().index_of(Kind::Terrain, "Bramble").unwrap();
        let b = objects::create(&mut w, Kind::Terrain, ty, None);
        objects::place(&mut w, b, Pos::centre(cell(0, 0), 0), Default::default());
        assert!(ground(&w, cell(0, 0)).full());
        w.obj_mut(m).unwrap().path.retry_at = w.tick + 100_000;
    }
    stand_in(&mut w, m, cell(-1, 0));
    let arrived = w.obj(m).unwrap().walk.as_ref().unwrap().arrived.unwrap();
    run_until(&mut w, 100, |w| walk_target_cell(w, m).is_some());
    (w.tick, walk_target_cell(&w, m).unwrap(), arrived)
}

// spec: S-0209 claim 6 (a clear next cell: the walking target is chosen there on the asking tick)
#[test]
fn walker_walks_on_into_clear_cell() {
    let (t, c, arrived) = second_target(false);
    assert_eq!(c, cell(0, 0));
    // The tick after the arrival tick is the asking tick; `w.tick` counts
    // the ticks already run, so the target exists after tick arrived + 1.
    assert_eq!(t, arrived + 2);
}

// spec: S-0209 claim 7 (blocked next cell: a new path at once, whatever the retry timer, and no tick lost)
#[test]
fn walker_repaths_at_once() {
    let (t0, c0, _) = second_target(false);
    let (t1, c1, _) = second_target(true);
    assert_eq!(c0, cell(0, 0));
    assert_ne!(c1, cell(0, 0));
    assert_eq!(t1, t0, "the detour starts on the same asking tick");
}

// spec: S-0209 claim 6 (a clear cell without a free position: the walker stops for the tick and asks again)
#[test]
fn walker_waits_without_free_position() {
    let (mut w, m) = corridor_scene("Mope", |_| {});
    order_move(&mut w, m, cell(6, 0));
    stand_in(&mut w, m, cell(0, 0));
    w.cells.occ_mut(cell(1, 0), Layer::Ground).unwrap().others = 1;
    assert_eq!(inf(&w, m, cell(1, 0)), Outcome::Clear);
    let before = steps(&w, m);
    run(&mut w, 5);
    assert_eq!(walk_target_cell(&w, m), None);
    assert_eq!(steps(&w, m), before, "the path is kept");
    w.cells.occ_mut(cell(1, 0), Layer::Ground).unwrap().others = 0;
    tick::step(&mut w);
    assert_eq!(walk_target_cell(&w, m), Some(cell(1, 0)));
}

// ---------------------------------------------------------------------------
// Claims 8, 11 and 12

fn friends_ahead(dest_x: i32) -> (World, ObjectId, Vec<ObjectId>) {
    let (mut w, m) = corridor_scene("Mope", |s| {
        for sub in 2..=4 {
            s.place_infantry(0, "Mope", cell(2, 0), sub);
        }
    });
    let friends = w.cells.list(cell(2, 0), Layer::Ground).to_vec();
    assert_eq!(friends.len(), 3);
    order_move(&mut w, m, cell(dest_x, 0));
    stand_in(&mut w, m, cell(1, 0));
    assert_eq!(inf(&w, m, cell(2, 0)), Outcome::StationaryFriendly);
    (w, m, friends)
}

// spec: S-0209 claim 8 (far from the destination: a scatter request to every occupant on the asking tick)
// spec: S-0209 claim 11 (a move-anywhere request reaches every listed occupant on one tick)
// spec: S-0209 claim 12 (the walker sends move-anywhere, forced requests)
#[test]
fn stationary_friendly_far_asks_them_to_move() {
    let (mut w, m, friends) = friends_ahead(6);
    // Defaults: no player-scatter rule, skill 0 below the threshold, so only
    // move-anywhere lets the request through; forced makes idle units obey.
    assert!(!w.rules().general.player_scatter);
    assert!(friends.iter().all(|&f| dest(&w, f).is_none()));
    tick::step(&mut w);
    assert!(friends.iter().all(|&f| dest(&w, f).is_some()));
    assert!(dest(&w, m).is_some(), "the walker keeps its move");
    assert_eq!(walk_target_cell(&w, m), None, "and waits for the tick");
}

// spec: S-0209 claim 8 (within the close-enough distance and in contact with nothing the move ends)
#[test]
fn stationary_friendly_close_gives_up() {
    let (mut w, m, friends) = friends_ahead(3);
    tick::step(&mut w);
    assert_eq!(dest(&w, m), None);
    assert!(steps(&w, m).is_empty());
    assert!(friends.iter().all(|&f| dest(&w, f).is_none()));
}

// spec: S-0209 claim 8 (a walker in contact with something does not give up but asks)
#[test]
fn stationary_friendly_contact_asks() {
    let (mut w, m, friends) = friends_ahead(3);
    w.obj_mut(m).unwrap().contact = Some(friends[0]);
    tick::step(&mut w);
    assert!(dest(&w, m).is_some());
    assert!(friends.iter().all(|&f| dest(&w, f).is_some()));
}

/// Two idle infantry of house 1 in one cell; returns the world and them.
fn idle_pair(tune: impl FnOnce(&mut Scene)) -> (World, Vec<ObjectId>) {
    let mut s = scene();
    s.place_infantry(1, "Mope", cell(0, 0), 2)
        .place_infantry(1, "Mope", cell(0, 0), 3);
    tune(&mut s);
    let mut w = s.build();
    open_orders(&mut w);
    let ids = w.cells.list(cell(0, 0), Layer::Ground).to_vec();
    assert_eq!(ids.len(), 2);
    (w, ids)
}

fn request(anywhere: bool) -> scatter::ScatterRequest {
    scatter::ScatterRequest {
        threat: None,
        forced: true,
        anywhere,
    }
}

fn obeyed(w: &World, ids: &[ObjectId]) -> Vec<bool> {
    ids.iter().map(|&i| dest(w, i).is_some()).collect()
}

// spec: S-0209 claim 11 (move-anywhere reaches all; otherwise nobody at the defaults)
#[test]
fn cell_request_move_anywhere_only() {
    let (mut w, ids) = idle_pair(|_| {});
    scatter::cell_request(&mut w, cell(0, 0), Layer::Ground, request(false));
    assert_eq!(obeyed(&w, &ids), [false, false]);
    scatter::cell_request(&mut w, cell(0, 0), Layer::Ground, request(true));
    assert_eq!(obeyed(&w, &ids), [true, true]);
}

// spec: S-0209 claim 11 (the game-wide player-scatter rule lets a request through)
#[test]
fn cell_request_player_scatter() {
    let (mut w, ids) = idle_pair(|s| s.ruleset.sim.general.player_scatter = Some(true));
    scatter::cell_request(&mut w, cell(0, 0), Layer::Ground, request(false));
    assert_eq!(obeyed(&w, &ids), [true, true]);
}

// spec: S-0209 claim 11 (a house skill level at or above the scatter threshold lets a request through)
#[test]
fn cell_request_skill_threshold() {
    let (mut w, ids) = idle_pair(|s| s.ruleset.sim.general.iq_scatter = Some(2));
    w.houses[1].skill = 1;
    scatter::cell_request(&mut w, cell(0, 0), Layer::Ground, request(false));
    assert_eq!(obeyed(&w, &ids), [false, false]);
    w.houses[1].skill = 2;
    scatter::cell_request(&mut w, cell(0, 0), Layer::Ground, request(false));
    assert_eq!(obeyed(&w, &ids), [true, true]);
}

// spec: S-0209 claim 11 (a house of our engine has skill level 0)
#[test]
fn houses_have_skill_zero() {
    let (w, _) = idle_pair(|_| {});
    assert!(w.houses.iter().all(|h| h.skill == 0));
}

// spec: S-0209 claim 12 (a crushing vehicle's warning is forced and not move-anywhere)
#[test]
fn crusher_warning_is_forced_not_anywhere() {
    for scatter_rule in [false, true] {
        let (mut w, ids) = idle_pair(|s| {
            s.ruleset.sim.general.player_scatter = Some(scatter_rule);
            s.place_unit(0, "Roller", cell(-1, 0), 64);
        });
        let roller = at(&w, Kind::Vehicle, cell(-1, 0));
        crush::warn_ahead(&mut w, roller, cell(0, 0));
        // Not move-anywhere: nobody obeys at the defaults. Forced: idle
        // units obey once the player-scatter rule lets it through.
        assert_eq!(obeyed(&w, &ids), [scatter_rule, scatter_rule]);
    }
}

// ---------------------------------------------------------------------------
// Claim 9

// spec: S-0209 claim 9 (an obstacle on the repeated ask: the first listed object, not allied, becomes the target; the path is dropped)
#[test]
fn walker_targets_first_listed_enemy() {
    let (mut w, m) = corridor_scene("Pike", |s| {
        s.place_unit(1, "Cart", cell(2, 0), 0);
    });
    let cart = at(&w, Kind::Vehicle, cell(2, 0));
    order_move(&mut w, m, cell(6, 0));
    stand_in(&mut w, m, cell(1, 0));
    assert_eq!(inf(&w, m, cell(2, 0)), Outcome::EnemyObstacle);
    tick::step(&mut w);
    let o = w.obj(m).unwrap();
    assert_eq!(o.target, Some(Target::Object(cart)));
    assert!(o.path.steps.is_empty());
}

// spec: S-0209 claim 9 (an allied object listed first: nothing is attacked)
#[test]
fn walker_spares_allied_first_listed() {
    let (mut w, m) = corridor_scene("Pike", |s| {
        s.place_structure(1, "Shed", cell(2, 0));
        s.place_infantry(0, "Mope", cell(2, 0), 2);
    });
    order_move(&mut w, m, cell(6, 0));
    stand_in(&mut w, m, cell(1, 0));
    let first = w.cells.list(cell(2, 0), Layer::Ground)[0];
    assert_eq!(w.obj(first).unwrap().kind, Kind::Infantry);
    assert_eq!(inf(&w, m, cell(2, 0)), Outcome::EnemyObstacle);
    tick::step(&mut w);
    let o = w.obj(m).unwrap();
    assert_eq!(o.target, None);
    assert!(o.path.steps.is_empty());
}

// ---------------------------------------------------------------------------
// Claim 10

// spec: S-0209 claim 10 (moving blocker: wait and start the blockage delay; walk on when it clears; the delay restarts after a step)
#[test]
fn walker_waits_for_moving_blocker() {
    let (mut w, m) = corridor_scene("Mope", |_| {});
    order_move(&mut w, m, cell(6, 0));
    stand_in(&mut w, m, cell(0, 0));
    w.cells.claim_vehicle(cell(1, 0), Layer::Ground, true);
    let pos = w.obj(m).unwrap().pos;
    tick::step(&mut w);
    let first = w.tick - 1;
    assert_eq!(w.obj(m).unwrap().path.blocked_since, Some(first));
    run(&mut w, 20);
    let o = w.obj(m).unwrap();
    assert_eq!(o.pos, pos, "it waits");
    assert_eq!(o.path.blocked_since, Some(first), "one wait, one start");
    assert!(o.dest.is_some());
    w.cells.claim_vehicle(cell(1, 0), Layer::Ground, false);
    run_until(&mut w, 50, |w| walk_target_cell(w, m).is_some());
    assert_eq!(walk_target_cell(&w, m), Some(cell(1, 0)));
    tick::step(&mut w);
    assert_eq!(w.obj(m).unwrap().path.blocked_since, None);
}

// ---------------------------------------------------------------------------
// Walls (claims 4 and 5), overrides (claim 4) and aircraft (claim 14)

/// Adds a wall overlay type (registration index 0; its land row passable so
/// that only the overlay rows decide) and the weapon "Breaker" that harms
/// walls. `crushable`: the wall can be crushed.
fn walls(s: &mut Scene, crushable: bool) {
    s.register("overlays", "Fence");
    s.ruleset.sim.overlays.insert(
        "Fence".into(),
        glasnost_rules::sim::OverlayType {
            wall: Some(true),
            crushable: Some(crushable),
            land: Some(LandType::Wall),
            ..Default::default()
        },
    );
    s.ruleset
        .sim
        .land_types
        .insert(LandType::Wall, LandRow::default());
    s.map.has_overlay_data = true;
    s.cell_mut(there()).overlay = 0;
    s.ruleset.sim.weapons.insert(
        "Breaker".into(),
        WeaponType {
            damage: Some(7),
            warhead: Some("Hammer".into()),
            ..Default::default()
        },
    );
    s.ruleset.sim.warheads.insert(
        "Hammer".into(),
        WarheadType {
            wall: Some(true),
            ..Default::default()
        },
    );
    s.infantry(
        "Sapper",
        UnitType {
            primary: Some("Breaker".into()),
            movement_zone: Some(MovementZone::InfantryDestroyer),
            ..walker(true)
        },
    );
    // Destroyer-zone vehicles may enter a wall cell (the terrain check), so
    // the overlay rows decide for them.
    for (n, primary) in [
        ("Dozer", Some("Breaker")),
        ("Plough", Some("Pea")),
        ("Barrow", None),
    ] {
        s.vehicle(
            n,
            UnitType {
                primary: primary.map(str::to_string),
                movement_zone: Some(MovementZone::Destroyer),
                ..vehicle(true, false)
            },
        );
    }
    s.vehicle(
        "Smasher",
        UnitType {
            movement_zone: Some(MovementZone::Crusher),
            ..vehicle(true, true)
        },
    );
}

// spec: S-0209 claim 4 (rows WO2 and WO4: for an infantry-destroyer walker a wall is an enemy obstacle when its weapon harms walls, blocked otherwise)
#[test]
fn walker_wall_rows() {
    let (w, m) = asked(Kind::Infantry, "Sapper", |s| {
        walls(s, false);
        s.infantry(
            "Digger",
            UnitType {
                movement_zone: Some(MovementZone::InfantryDestroyer),
                ..walker(true)
            },
        );
        s.place_infantry(0, "Digger", cell(0, 1), 2);
    });
    assert_eq!(w.cells.cell(there()).unwrap().overlay, 0, "wall placed");
    let digger = at(&w, Kind::Infantry, cell(0, 1));
    // Row WO2: the weapon cannot damage walls.
    assert_eq!(inf(&w, digger, there()), Outcome::Blocked);
    // Row WO4 (map walls have no owner here, so "another house").
    assert_eq!(inf(&w, m, there()), Outcome::EnemyObstacle);
}

// spec: S-0209 claim 5 (rows VO4 to VO6: a wall by primary weapon, for vehicles the terrain check lets in)
#[test]
fn vehicle_wall_rows() {
    let (w, m) = asked(Kind::Vehicle, "Dozer", |s| {
        walls(s, false);
        s.place_unit(0, "Plough", cell(0, 1), 0);
        s.place_unit(0, "Barrow", cell(-1, 1), 0);
    });
    let plough = at(&w, Kind::Vehicle, cell(0, 1));
    let barrow = at(&w, Kind::Vehicle, cell(-1, 1));
    assert_eq!(veh(&w, m, there()), Outcome::EnemyObstacle);
    assert_eq!(veh(&w, plough, there()), Outcome::Blocked);
    assert_eq!(veh(&w, barrow, there()), Outcome::Blocked);
}

// spec: S-0209 claim 5 (row VO3: a wall the crusher crushes, owned by another house, gives none)
#[test]
fn vehicle_crushes_wall() {
    let (w, m) = asked(Kind::Vehicle, "Smasher", |s| {
        walls(s, true);
    });
    assert_eq!(veh(&w, m, there()), Outcome::Clear);
}

// spec: S-0209 claim 9 (an obstacle cell that lists nothing but holds a wall: the wall is the target)
#[test]
fn walker_targets_wall() {
    let mut s = scene();
    corridor(&mut s);
    walls(&mut s, false);
    s.cell_mut(there()).overlay = glasnost_map::NO_OVERLAY;
    s.cell_mut(cell(2, 0)).overlay = 0;
    s.place_infantry(0, "Sapper", cell(-2, 0), 0);
    let mut w = s.build();
    let m = at(&w, Kind::Infantry, cell(-2, 0));
    // The wall outcome itself (fails first, at the cause).
    assert_eq!(inf(&w, m, cell(2, 0)), Outcome::EnemyObstacle);
    open_orders(&mut w);
    order_move(&mut w, m, cell(6, 0));
    stand_in(&mut w, m, cell(1, 0));
    tick::step(&mut w);
    let o = w.obj(m).unwrap();
    assert_eq!(o.target, Some(Target::Cell(cell(2, 0))));
    assert!(o.path.steps.is_empty());
}

// spec: S-0209 claim 4 (row WP3 overrides: clear; a blocked value listed before the override decides)
#[test]
fn walker_mission_target_override() {
    let (mut w, m) = asked(Kind::Infantry, "Mope", |s| {
        s.place_structure(1, "Shed", there());
    });
    let shed = at(&w, Kind::Building, there());
    assert_eq!(inf(&w, m, there()), Outcome::Blocked);
    {
        let o = w.obj_mut(m).unwrap();
        o.beh.current = Some(glasnost_sim::rules::Behaviour::Enter);
        o.dest = Some(Target::Object(shed));
    }
    assert_eq!(inf(&w, m, there()), Outcome::Clear);
    // An enemy aircraft listed ahead of the building gives the unarmed
    // walker blocked (row WP22), which beats the override.
    let ty = w.rules().index_of(Kind::Aircraft, "Kite").unwrap();
    let k = objects::create(&mut w, Kind::Aircraft, ty, Some(HouseId(1)));
    objects::place(&mut w, k, Pos::centre(there(), 0), Default::default());
    assert_eq!(w.cells.list(there(), Layer::Ground)[0], k);
    assert_eq!(inf(&w, m, there()), Outcome::Blocked);
}

fn with_aircraft(setup: impl FnOnce(&mut Scene)) -> (World, ObjectId) {
    let mut s = scene();
    s.map.objects.aircraft.push(Aircraft {
        placed: Placed {
            key: b"kite-1".to_vec(),
            owner: b"Alpha".to_vec(),
            type_name: b"Kite".to_vec(),
            health: 256,
            cell: here(),
            facing: 64,
            raw: Vec::new(),
        },
    });
    setup(&mut s);
    let w = s.build();
    let a = at(&w, Kind::Aircraft, here());
    (w, a)
}

// spec: S-0209 claim 14 (an aircraft gets clear whatever the cell's occupants)
#[test]
fn aircraft_clear_over_occupants() {
    let (w, a) = with_aircraft(|s| {
        s.place_structure(1, "Shed", there());
        s.place_unit(0, "Cart", cell(0, 1), 0);
        s.place_terrain("Bramble", cell(1, 1));
    });
    let r = occupancy::reference_level(&w, a);
    for c in [there(), cell(0, 1), cell(1, 1)] {
        assert_eq!(path::step_outcome(&w, a, here(), c, r), Outcome::Clear);
    }
}

// spec: S-0209 claim 4 (row WP27 before WP22: a terrain object gives no value even to an unarmed walker)
#[test]
fn unarmed_walker_terrain_object_none() {
    let (w, m) = asked(Kind::Infantry, "Mope", |s| {
        s.place_terrain("Moss", there());
    });
    assert_eq!(inf(&w, m, there()), Outcome::Clear);
}

// spec: S-0209 claim 4 (row WP3 decides, and the zero walker speed check still applies to it)
#[test]
fn mission_target_override_keeps_speed_check() {
    let (mut w, m) = asked(Kind::Infantry, "Mope", |s| {
        s.set_cell(there(), common::code_tile(7), 0);
    });
    assert!(!glasnost_sim::passability::enterable_by_class(
        w.rules(),
        &w,
        there(),
        Some(FOOT),
        MovementZone::Infantry,
        false
    ));
    let ty = w.rules().index_of(Kind::Vehicle, "Cart").unwrap();
    let cart = objects::create(&mut w, Kind::Vehicle, ty, Some(HouseId(0)));
    objects::place(&mut w, cart, Pos::centre(there(), 0), Default::default());
    {
        let o = w.obj_mut(m).unwrap();
        o.beh.current = Some(glasnost_sim::rules::Behaviour::Enter);
        o.dest = Some(Target::Object(cart));
    }
    assert_eq!(inf(&w, m, there()), Outcome::Blocked);
}

// spec: S-0209 claim 10 (when the retry timer has run out the walker searches again and, once the blockage delay is over, routes around)
#[test]
fn walker_routes_around_moving_blocker() {
    let mut s = scene();
    s.ruleset.sim.general.blockage_path_delay = Some(5);
    s.place_infantry(0, "Mope", cell(-2, 0), 0);
    let mut w = s.build();
    let m = at(&w, Kind::Infantry, cell(-2, 0));
    open_orders(&mut w);
    order_move(&mut w, m, cell(4, 0));
    stand_in(&mut w, m, cell(-1, 0));
    w.cells.claim_vehicle(cell(0, 0), Layer::Ground, true);
    w.obj_mut(m).unwrap().path.retry_at = w.tick;
    tick::step(&mut w);
    assert!(w.obj(m).unwrap().path.blocked_since.is_some());
    assert_eq!(walk_target_cell(&w, m), None);
    let n = run_until(&mut w, 200, |w| walk_target_cell(w, m).is_some());
    assert!(n < 200, "the walker walks on");
    let c = walk_target_cell(&w, m).unwrap();
    assert_ne!(c, cell(0, 0));
    assert_eq!(c.x, cell(0, 0).x, "a detour beside the blocked cell");
}
