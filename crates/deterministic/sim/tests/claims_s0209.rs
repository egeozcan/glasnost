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
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::world::{HouseId, Layer, ObjectId, Pos, Target, World};
use glasnost_sim::{crush, objects, path, scatter, tick, trace};

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
        ("Door", b(|t| t.gate = Some(true))),
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
                ..Default::default()
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

// spec: S-0209 claim 7 and the walker table of State and ordering (blocked next cell: a new path at once, whatever the retry timer; on a repeated ask that gives clear with a position free, the walking target is chosen in the new path's first cell on the same asking tick, so no tick is lost)
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

/// Whether every one of `ids` accepted a scatter request in the trace.
/// A friend's destination may be gone by the end of the tick: one whose
/// cell lies on the corridor's raised side fails the reachability gate in
/// the friend's own turn, which clears a walker's destination (S-0417
/// claim 17, No path; S-0212 claim 23). Since S-0208 r46 claim 6 that
/// failure can come already inside the request: a standing friend given a
/// destination by the nearby-cell search has its extra locomotion at once,
/// so the destination the trace reads after the request may be gone; the
/// failed search is then traced for it before its answer.
fn all_accepted(w: &World, ids: &[ObjectId]) -> bool {
    let ev = trace::take(w);
    ids.iter().all(|&f| {
        let mine: Vec<&trace::What> = ev.iter().filter(|e| e.unit == f).map(|e| &e.what).collect();
        mine.iter().enumerate().any(|(i, what)| match what {
            trace::What::Asked {
                accepted: true,
                dest: Some(_),
            } => true,
            trace::What::Asked {
                accepted: true,
                dest: None,
            } => mine[..i].iter().any(|x| {
                matches!(
                    x,
                    trace::What::Failed {
                        dest_cleared: true,
                        ..
                    }
                )
            }),
            _ => false,
        })
    })
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
    trace::switch_on(&mut w);
    tick::step(&mut w);
    assert!(all_accepted(&w, &friends));
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
    trace::switch_on(&mut w);
    tick::step(&mut w);
    assert!(dest(&w, m).is_some());
    assert!(all_accepted(&w, &friends));
}

/// [`friends_ahead`] with the whole corridor raised to level 4 (walls at
/// level 8) and, at x = 3, a one-cell high-bridge deck at level 4 over
/// level-0 ground: the walker's destination is that deck cell. With
/// `bridge` false, x = 3 is level-4 ground like the rest (the control).
fn friends_before_a_deck(bridge: bool) -> (World, ObjectId, Vec<ObjectId>) {
    let mut s = scene();
    for x in -4..=9 {
        s.set_cell(cell(x, -1), 0, 8);
        s.set_cell(cell(x, 1), 0, 8);
        s.set_cell(cell(x, 0), 0, 4);
    }
    s.set_cell(cell(-4, 0), 0, 8);
    s.set_cell(cell(9, 0), 0, 8);
    if bridge {
        s.set_cell(cell(3, 0), 0, 0);
    }
    s.place_infantry(0, "Mope", cell(-2, 0), 0);
    for sub in 2..=4 {
        s.place_infantry(0, "Mope", cell(2, 0), sub);
    }
    let mut w = s.build();
    if bridge {
        let i = w.cells.idx(cell(3, 0)).unwrap();
        w.cells.deck[i] = true;
        w.cells.deck_entry[i] = true;
        w.cells.bridge_axis[i] = glasnost_sim::cells::Axis::X;
        // spec: S-0212 claim 4 (a deck is read through the first enabled bridge link within 1 cell: the reach region of its first end, here the corridor's)
        w.cells.links.push(glasnost_sim::cells::BridgeLink {
            first: cell(4, 0),
            second: cell(2, 0),
            along_x: true,
            enabled: true,
        });
        w.regions.dirty = true;
    }
    let m = at(&w, Kind::Infantry, cell(-2, 0));
    open_orders(&mut w);
    let friends = w.cells.list(cell(2, 0), Layer::Ground).to_vec();
    assert_eq!(friends.len(), 3);
    order_move(&mut w, m, cell(3, 0));
    stand_in(&mut w, m, cell(1, 0));
    assert_eq!(inf(&w, m, cell(2, 0)), Outcome::StationaryFriendly);
    (w, m, friends)
}

// spec: S-0209 claim 8 and F1 (within the close-enough distance, |Δz| < 208 and in contact with nothing the move ends; the walker's locomotor destination, both heights world heights) and S-0208 claim 18 and F9 (a cell destination carrying a high bridge is 414 leptons above the terrain, whatever the walker's layer): a walker at deck height close to a deck destination over low ground gives up, as on flat ground
#[test]
fn stationary_friendly_close_to_a_bridge_destination_gives_up() {
    for bridge in [false, true] {
        let (mut w, m, friends) = friends_before_a_deck(bridge);
        let o = w.obj(m).unwrap();
        assert_eq!(o.layer, Layer::Ground);
        assert_eq!(
            o.pos.z,
            w.surface_at(o.pos.x, o.pos.y),
            "the walker stands on level-4 ground"
        );
        trace::switch_on(&mut w);
        tick::step(&mut w);
        assert_eq!(dest(&w, m), None, "the move ends (bridge {bridge})");
        assert!(steps(&w, m).is_empty());
        let mine: Vec<trace::What> = trace::take(&w)
            .into_iter()
            .filter(|e| e.unit == m)
            .map(|e| e.what)
            .collect();
        assert!(
            mine.iter().any(|x| matches!(x, trace::What::GaveUp))
                && !mine.iter().any(|x| matches!(x, trace::What::Failed { .. })),
            "given up by claim 8, not by a failed search (bridge {bridge}): {mine:?}"
        );
        assert!(
            friends.iter().all(|&f| dest(&w, f).is_none()),
            "no friend is asked (bridge {bridge})"
        );
    }
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
    // spec: S-0209 claim 9 (r55: the take is an override, S-0215 claim 10, to attack with that target and no destination; the move is suspended and the destination put aside)
    assert_eq!(o.beh.current, Some(Behaviour::Attack));
    assert_eq!(o.dest, None);
    assert_eq!(o.beh.suspended, Some(Behaviour::Move));
    assert_eq!(o.beh.aside, Some((None, Some(Target::Cell(cell(6, 0))))));
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
    // spec: S-0209 claim 9 (no take, so no override: the move stays current with its destination)
    assert_eq!(o.beh.current, Some(Behaviour::Move));
    assert_eq!(o.dest, Some(Target::Cell(cell(6, 0))));
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

/// The corridor walker waiting behind a moving blocker in (1, 0) on its
/// way to (6, 0): returns the tick its blockage delay started.
fn blocked_walker() -> (World, ObjectId, i32) {
    let (mut w, m) = corridor_scene("Mope", |_| {});
    order_move(&mut w, m, cell(6, 0));
    stand_in(&mut w, m, cell(0, 0));
    w.cells.claim_vehicle(cell(1, 0), Layer::Ground, true);
    tick::step(&mut w);
    let first = w.tick - 1;
    assert_eq!(w.obj(m).unwrap().path.blocked_since, Some(first));
    run(&mut w, 10);
    assert_eq!(w.obj(m).unwrap().path.blocked_since, Some(first));
    (w, m, first)
}

// spec: S-0209 claim 10 (r60: a new destination given by an order starts the blockage delay afresh, counted from the first moving-blocker answer after it, when its path leads through the same cell); scenario C5 part c
#[test]
fn c5_c_a_new_destination_restarts_the_blockage_delay() {
    let (mut w, m, first) = blocked_walker();
    let pos = w.obj(m).unwrap().pos;
    order_move(&mut w, m, cell(5, 0));
    let n = run_until(&mut w, 20, |w| {
        w.obj(m)
            .unwrap()
            .path
            .blocked_since
            .is_some_and(|s| s > first)
    });
    assert!(n < 20, "it waits again behind the blocker");
    let o = w.obj(m).unwrap();
    assert_eq!(o.dest, Some(Target::Cell(cell(5, 0))));
    assert_eq!(o.pos, pos, "it has not moved");
    let again = o.path.blocked_since.unwrap();
    assert_eq!(again, w.tick - 1, "from the first answer after the order");
}

// spec: S-0209 claim 10 (r60: a new destination by any other rule starts it afresh too, here one written without reaching the locomotor)
#[test]
fn a_destination_written_by_another_rule_restarts_the_blockage_delay() {
    let (mut w, m, first) = blocked_walker();
    w.obj_mut(m).unwrap().dest = Some(Target::Cell(cell(5, 0)));
    tick::step(&mut w);
    let again = w.obj(m).unwrap().path.blocked_since;
    assert_eq!(again, Some(w.tick - 1), "afresh after {first}");
}

// spec: S-0209 claim 10 (r60: a walker whose destination is cleared does not wait, and its later wait follows a new destination, which starts the delay afresh)
#[test]
fn a_cleared_destination_ends_the_wait() {
    let (mut w, m, first) = blocked_walker();
    w.obj_mut(m).unwrap().dest = None;
    tick::step(&mut w);
    assert_eq!(w.obj(m).unwrap().path.blocked_since, None, "no wait");
    order_move(&mut w, m, cell(6, 0));
    let n = run_until(&mut w, 20, |w| {
        w.obj(m).unwrap().path.blocked_since.is_some()
    });
    assert!(n < 20, "it waits again behind the blocker");
    let again = w.obj(m).unwrap().path.blocked_since.unwrap();
    assert!(again > first, "afresh: {again} after {first}");
    assert_eq!(again, w.tick - 1, "from the first answer after the order");
}

/// Runs the walker (with the trace on) at most `max` ticks until it makes
/// its first path computation that avoids moving blockers (search mode 2,
/// S-0212 Terms), and returns that tick.
fn avoids_at(w: &mut World, m: ObjectId, max: i32) -> Option<i32> {
    for _ in 0..max {
        tick::step(w);
        let hit = trace::take(w).iter().find_map(|e| match e.what {
            trace::What::Search {
                asked: path::Mode::Avoid,
                ..
            } if e.unit == m => Some(e.tick),
            _ => None,
        });
        if hit.is_some() {
            return hit;
        }
    }
    None
}

// spec: S-0209 claim 10 (r76, Q-2822: an order to the cell that is already the walker's destination starts the blockage delay afresh, as an order to another cell does; infantry has no unchanged-destination suppression, S-0213 claim 21, and the wait counts from the first moving-blocker answer after the order; the search that avoids the blocker comes no earlier than the blockage delay after that answer); scenario C5 part c2
#[test]
fn c5_c2_the_same_destination_again_restarts_the_blockage_delay() {
    let (mut w, m, first) = blocked_walker();
    let d = w.rules().general.blockage_delay;
    let r = w.rules().general.retry_delay;
    assert!(r <= d / 2 - 3, "C5's set-up: R = {r}, D = {d}");
    // C5 part c2: the order comes D/2 ticks after the first stop.
    let wait = first + d / 2 - w.tick;
    run(&mut w, wait);
    assert_eq!(w.obj(m).unwrap().path.blocked_since, Some(first));
    let mut control = w.clone();
    trace::switch_on(&mut w);
    let pos = w.obj(m).unwrap().pos;
    order_move(&mut w, m, cell(6, 0));
    let ordered = w.tick;
    let n = run_until(&mut w, 20, |w| {
        w.obj(m)
            .unwrap()
            .path
            .blocked_since
            .is_some_and(|s| s > first)
    });
    assert!(n < 20, "it waits again behind the blocker");
    let o = w.obj(m).unwrap();
    assert_eq!(o.dest, Some(Target::Cell(cell(6, 0))));
    assert_eq!(o.pos, pos, "it has not moved");
    let again = o.path.blocked_since.unwrap();
    assert_eq!(again, w.tick - 1, "from the first answer after the order");
    // The trace is not carried by a copy of the game: switched on in each.
    trace::switch_on(&mut control);
    let g0 = avoids_at(&mut control, m, 4 * d).expect("the control avoids");
    assert!(g0 >= first + d, "the control waits the delay: {g0}");
    assert!(g0 < ordered + d, "the control is not restarted: {g0}");
    let g1 = avoids_at(&mut w, m, 4 * d).expect("it avoids");
    assert!(g1 >= again + d, "a whole delay after the restart: {g1}");
    assert!(g1 > g0, "later than the control: {g1} against {g0}");
}

/// The corridor walker of [`blocked_walker`], but on its way to (1, 0), the
/// cell the moving blocker holds, in a ruleset whose road land type has a
/// zero foot multiplier (its other multipliers stay at 1, so the category
/// stays passable). Half a blockage delay after the wait starts, (1, 0)
/// becomes road: the next ask answers blocked, claim 7's search at once
/// fails, the destination being next to the walker (S-0212 claim 15), and
/// the walker keeps it (S-0212 claim 23 row 3). If `assign`, the same
/// destination is then assigned through the assignment hook. (1, 0) is made
/// clear again a tick later. Returns the game with the trace on, the
/// walker, the tick the wait started and the tick of the failed search.
fn kept_after_failure(assign: bool) -> (World, ObjectId, i32, i32) {
    let (mut w, m) = corridor_scene("Mope", |s| {
        s.ruleset.sim.land_types.insert(
            LandType::Road,
            LandRow {
                foot: Some(glasnost_rules::sim::Decimal("0".into())),
                ..Default::default()
            },
        );
    });
    order_move(&mut w, m, cell(1, 0));
    stand_in(&mut w, m, cell(0, 0));
    w.cells.claim_vehicle(cell(1, 0), Layer::Ground, true);
    tick::step(&mut w);
    let first = w.tick - 1;
    assert_eq!(w.obj(m).unwrap().path.blocked_since, Some(first));
    let d = w.rules().general.blockage_delay;
    let wait = first + d / 2 - w.tick;
    run(&mut w, wait);
    assert_eq!(w.obj(m).unwrap().path.blocked_since, Some(first));
    w.cells.grid.get_mut(cell(1, 0)).unwrap().land = glasnost_map::terrain::land::ROAD;
    glasnost_sim::regions::cell_changed(&mut w, cell(1, 0));
    trace::switch_on(&mut w);
    tick::step(&mut w);
    let failed = w.tick - 1;
    let fresh: Vec<_> = trace::take(&w)
        .iter()
        .filter(|e| e.unit == m)
        .filter_map(|e| match e.what {
            trace::What::Search { asked, result, .. } => Some((asked, result)),
            _ => None,
        })
        .collect();
    assert_eq!(
        fresh,
        vec![(path::Mode::Fresh, path::Search::SearchFailed)],
        "claim 7's search at once, which fails"
    );
    let o = w.obj(m).unwrap();
    assert_eq!(o.dest, Some(Target::Cell(cell(1, 0))), "kept within 1 cell");
    let r = w.rules().general.retry_delay;
    assert_eq!(o.path.retry_at, failed + r, "S-0212 claim 21: it waits R");
    if assign {
        let before = o.dest;
        glasnost_sim::econ::assign_destination(&mut w, m, before);
        assert_eq!(w.obj(m).unwrap().dest, before);
    }
    tick::step(&mut w);
    trace::take(&w);
    w.cells.grid.get_mut(cell(1, 0)).unwrap().land = glasnost_map::terrain::land::CLEAR;
    glasnost_sim::regions::cell_changed(&mut w, cell(1, 0));
    (w, m, first, failed)
}

// spec: S-0209 claim 10 (r76: the blockage delay starts afresh only after a walking step or at an assignment of a destination; a failed search after which the walker keeps its destination, S-0212 claim 23 row 3, is not an assignment, so the delay counts on from the first moving-blocker answer: the search that avoids the blocker comes no earlier than the blockage delay after that answer and earlier than the blockage delay after the failure); S-0209 claim 7 (a blocked answer at the first ask searches at once)
#[test]
fn seams7b_a_failed_search_that_keeps_the_destination_leaves_the_blockage_delay_running() {
    let (mut w, m, first, failed) = kept_after_failure(false);
    let d = w.rules().general.blockage_delay;
    let r = w.rules().general.retry_delay;
    assert!(r <= d / 2 - 3, "C5's set-up: R = {r}, D = {d}");
    assert_eq!(
        w.obj(m).unwrap().path.blocked_since,
        Some(first),
        "not restarted by the failure"
    );
    let pos = w.obj(m).unwrap().pos;
    let g = avoids_at(&mut w, m, 4 * d).expect("it avoids");
    assert!(g >= first + d, "the whole delay from the first answer: {g}");
    assert!(g < failed + d, "not counted from the failure: {g}");
    assert_eq!(
        w.obj(m).unwrap().pos.cell(),
        pos.cell(),
        "it has not walked"
    );
}

// spec: S-0209 claim 10 (r76: control: an assignment of the destination the walker already has, made in the tick of that failed search, does start the delay afresh, from the first moving-blocker answer after it)
#[test]
fn seams7b_the_control_an_assignment_after_the_failed_search_restarts_it() {
    let (mut w, m, first, failed) = kept_after_failure(true);
    let d = w.rules().general.blockage_delay;
    let since = w.obj(m).unwrap().path.blocked_since;
    assert!(
        since.is_none_or(|s| s > failed),
        "restarted after {first}: {since:?}"
    );
    let g = avoids_at(&mut w, m, 4 * d).expect("it avoids");
    assert!(g >= failed + d, "the whole delay after the assignment: {g}");
}

// spec: S-0209 claim 10 (r76: the restart holds at every assignment that is made, here one of the destination the walker already has by another rule, through the assignment hook)
#[test]
fn r76_another_rules_assignment_of_the_same_destination_restarts_it() {
    let (mut w, m, first) = blocked_walker();
    let before = w.obj(m).unwrap().dest;
    glasnost_sim::econ::assign_destination(&mut w, m, before);
    assert_eq!(w.obj(m).unwrap().dest, before);
    assert_eq!(w.obj(m).unwrap().path.blocked_since, None, "restarted");
    tick::step(&mut w);
    let again = w.obj(m).unwrap().path.blocked_since;
    assert_eq!(again, Some(w.tick - 1), "afresh after {first}");
}

// spec: S-0209 claim 10 (r76: the same assignment of the destination the walker already has also ends its running retry delay) and S-0212 claim 21 (r98, Q-2896: for the walk locomotor the destination it already holds, given again by any rule, ends a running retry delay, a delay started earlier in the same tick included)
#[test]
fn r98_the_same_destination_again_ends_the_retry_delay() {
    let (mut w, m, _) = blocked_walker();
    let t = w.tick;
    let r = w.rules().general.retry_delay;
    path::start_delay(w.obj_mut(m).unwrap(), t, r);
    assert!(path::delay_runs(&w, m), "a delay started in this tick");
    let before = w.obj(m).unwrap().dest;
    glasnost_sim::econ::assign_destination(&mut w, m, before);
    assert_eq!(w.obj(m).unwrap().dest, before);
    assert_eq!(
        w.obj(m).unwrap().path.retry_at,
        t,
        "ended at the assignment"
    );
    assert!(!path::delay_runs(&w, m), "the retry delay has ended");
    assert_eq!(w.obj(m).unwrap().path.blocked_since, None);
}

/// The corridor walker of [`blocked_walker`], of type `ty`, with an idle
/// enemy vehicle "Cart" outside the corridor at (3, 4), an own shed K at
/// (-1, 4) and an own transport "Wagon" (room for two) at (6, 3), outside
/// the corridor too. Returns the walker, the tick its blockage delay
/// started, the cart, K and the wagon.
fn blocked_with_objects(ty: &str) -> (World, ObjectId, i32, [ObjectId; 3]) {
    let (mut w, m) = corridor_scene(ty, |s| {
        s.vehicle(
            "Wagon",
            UnitType {
                passengers: Some(2),
                ..vehicle(false, false)
            },
        );
        s.place_unit(1, "Cart", cell(3, 4), 0);
        s.place_structure(0, "Shed", cell(-1, 4));
        s.place_unit(0, "Wagon", cell(6, 3), 0);
    });
    let cart = at(&w, Kind::Vehicle, cell(3, 4));
    let shed = at(&w, Kind::Building, cell(-1, 4));
    let wagon = at(&w, Kind::Vehicle, cell(6, 3));
    order_move(&mut w, m, cell(6, 0));
    stand_in(&mut w, m, cell(0, 0));
    w.cells.claim_vehicle(cell(1, 0), Layer::Ground, true);
    tick::step(&mut w);
    let first = w.tick - 1;
    run(&mut w, 10);
    assert_eq!(w.obj(m).unwrap().path.blocked_since, Some(first));
    (w, m, first, [cart, shed, wagon])
}

/// Applies `order` of house 0 at once, without the rest of the tick.
fn apply(w: &mut World, order: Order) {
    assert!(orders::issue(w, HouseId(0), order), "order refused");
    orders::apply_pending(w);
}

fn behaviour_order(
    actor: ObjectId,
    b: Behaviour,
    target: Option<Target>,
    dest: Option<Target>,
) -> Order {
    Order::Behaviour {
        actor,
        behaviour: b,
        attack_move: false,
        target,
        dest,
    }
}

/// Makes the waiting walker hold `dest` as the destination its wait was met
/// under, with behaviour `b` current.
fn waits_for(w: &mut World, m: ObjectId, b: Behaviour, dest: Target) {
    let ob = w.obj_mut(m).unwrap();
    ob.beh.current = Some(b);
    ob.dest = Some(dest);
    ob.path.blocked_for = Some(dest);
}

// spec: S-0209 claim 10 (r76: an attack order on the walker's target, given again, makes its destination nothing, S-0213 table T3 part f, so its wait ends)
#[test]
fn r76_an_attack_order_given_again_ends_the_wait() {
    let (mut w, m, first, [cart, ..]) = blocked_with_objects("Pike");
    w.obj_mut(m).unwrap().target = Some(Target::Object(cart));
    apply(
        &mut w,
        behaviour_order(m, Behaviour::Attack, Some(Target::Object(cart)), None),
    );
    let o = w.obj(m).unwrap();
    assert_eq!(o.target, Some(Target::Object(cart)));
    assert_eq!(o.dest, None, "the destination becomes nothing");
    assert_eq!(o.path.blocked_since, None, "the wait ends");
    tick::step(&mut w);
    let o = w.obj(m).unwrap();
    assert_ne!(o.path.blocked_since, Some(first), "no wait counts on");
}

// spec: S-0209 claim 10 (r76: a duplicate enter order, given by a walker already in enter and in contact with the building it enters, is ignored by S-0213 table T3 part a, so nothing is assigned and the wait counts on)
// spec: S-0212 claim 21 (r98: only an assignment that is made ends a walker's running retry delay at the destination it already holds, so this one runs on)
#[test]
fn r76_a_duplicate_enter_order_assigns_nothing() {
    let (mut w, m, first, [_, shed, _]) = blocked_with_objects("Mope");
    waits_for(&mut w, m, Behaviour::Enter, Target::Object(shed));
    w.obj_mut(m).unwrap().contact = Some(shed);
    let t = w.tick;
    let r = w.rules().general.retry_delay;
    path::start_delay(w.obj_mut(m).unwrap(), t, r);
    apply(
        &mut w,
        behaviour_order(m, Behaviour::Enter, None, Some(Target::Object(shed))),
    );
    let o = w.obj(m).unwrap();
    assert_eq!(o.dest, Some(Target::Object(shed)));
    assert_eq!(o.contact, Some(shed));
    assert_eq!(o.path.blocked_since, Some(first), "the wait counts on");
    assert_eq!(o.path.retry_at, t + r);
    assert!(path::delay_runs(&w, m), "the retry delay runs on");
}

// spec: S-0209 claim 10 (r76: an enter order given again by a tethered walker that is not a duplicate: by S-0213 table T3 part b it keeps its link, and with it its contact, since K is not a building of the unloading kind; the assignment is made, starting the blockage delay afresh, and drops its stored path, S-0208 claim 6 for a walker in contact)
#[test]
fn r76_a_tethered_walkers_enter_order_given_again_restarts_it() {
    let (mut w, m, _, [_, shed, wagon]) = blocked_with_objects("Mope");
    waits_for(&mut w, m, Behaviour::Enter, Target::Object(wagon));
    {
        let ob = w.obj_mut(m).unwrap();
        ob.contact = Some(shed);
        ob.econ.exit = Some(glasnost_sim::econ::release::Exit {
            factory: shed,
            tethered: true,
            place: 0,
        });
        ob.path.steps = vec![2, 2, 2];
    }
    assert!(glasnost_sim::econ::tethered(&w, m));
    apply(
        &mut w,
        behaviour_order(m, Behaviour::Enter, None, Some(Target::Object(wagon))),
    );
    let o = w.obj(m).unwrap();
    assert_eq!(o.dest, Some(Target::Object(wagon)), "assigned");
    assert_eq!(o.contact, Some(shed), "the link and its contact kept");
    assert!(glasnost_sim::econ::tethered(&w, m));
    assert_eq!(o.path.blocked_since, None, "the blockage delay restarts");
    assert!(o.path.steps.is_empty(), "the stored path is dropped");
}

// spec: S-0209 claim 10 (r76: an enter order given again by a walker with no contact, to an idle transport that has room for it, no destination and no other contact, on land in its reach region: the assignment is made and starts the blockage delay afresh; whether the destination is the transport or its cell is open, Open questions, and our engine gives the transport)
// spec: S-0212 claim 21 (r98, Q-2896: the same object given again by an order ends the walker's running retry delay)
#[test]
fn r76_an_enter_order_given_again_to_an_idle_transport_restarts_it() {
    let (mut w, m, _, [_, _, wagon]) = blocked_with_objects("Mope");
    waits_for(&mut w, m, Behaviour::Enter, Target::Object(wagon));
    let t = w.tick;
    let r = w.rules().general.retry_delay;
    path::start_delay(w.obj_mut(m).unwrap(), t, r);
    assert!(path::delay_runs(&w, m));
    apply(
        &mut w,
        behaviour_order(m, Behaviour::Enter, None, Some(Target::Object(wagon))),
    );
    let o = w.obj(m).unwrap();
    assert_eq!(o.dest, Some(Target::Object(wagon)));
    assert_eq!(o.path.blocked_since, None, "the blockage delay restarts");
    assert_eq!(o.path.retry_at, t, "the retry delay ends");
    assert!(!path::delay_runs(&w, m));
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

// spec: S-0209 claim 4 (row WP3 overrides: clear; row WP4: the other occupants of a mission-target building's cell give none unless being erased; a blocked value listed before the override decides; scenario C17)
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
    // An enemy aircraft listed ahead of the mission-target building gives
    // none (row WP4), so the building's override decides: clear (C17 (a)).
    let ty = w.rules().index_of(Kind::Aircraft, "Kite").unwrap();
    let k = objects::create(&mut w, Kind::Aircraft, ty, Some(HouseId(1)));
    objects::place(&mut w, k, Pos::centre(there(), 0), Default::default());
    assert_eq!(w.cells.list(there(), Layer::Ground)[0], k);
    assert_eq!(inf(&w, m, there()), Outcome::Clear);
    // A mission target that is the building's cell: the same (row WP4's
    // reference-point form).
    w.obj_mut(m).unwrap().dest = Some(Target::Cell(there()));
    assert_eq!(inf(&w, m, there()), Outcome::Clear);
    // While the aircraft is being erased its own row decides: an enemy
    // object gives the unarmed walker blocked (row WP22), which beats the
    // override (C17 (b)).
    w.obj_mut(m).unwrap().dest = Some(Target::Object(shed));
    w.temporal.locks.insert(
        k,
        glasnost_sim::temporal::Lock {
            leader: shed,
            joiners: Vec::new(),
            amount: 1_000_000,
            was_off: false,
            suspended: false,
        },
    );
    assert!(glasnost_sim::econ::power::being_erased(&w, k));
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

// ---------------------------------------------------------------------------
// When a driving vehicle clears its vehicle-present flag (claim 1, scenario C28)

/// A scene with one slow driving vehicle (speed field 1: s = 2 per tick,
/// acceleration off) facing map north in the middle cell, ordered to the
/// end of `steps`, whose path is then set to `steps`.
fn slow_driver(steps: &[u8]) -> (World, ObjectId) {
    let mut s = Scene::new();
    let mut t = unit_type(TRACK, 1, 10);
    t.accelerates = Some(false);
    s.vehicle("Snail", t);
    s.place_unit(0, "Snail", cell(0, 0), 0);
    let mut w = s.build();
    let id = w
        .objects
        .values()
        .find(|o| o.kind == Kind::Vehicle)
        .map(|o| o.id)
        .expect("vehicle");
    open_orders(&mut w);
    let mut end = cell(0, 0);
    for &d in steps {
        let (dx, dy) = glasnost_sim::regions::STEPS[usize::from(d)];
        end = CellXY::new(end.x + dx, end.y + dy);
    }
    order_move(&mut w, id, end);
    tick::step(&mut w);
    w.obj_mut(id).expect("vehicle").path.steps = steps.to_vec();
    (w, id)
}

fn flag(w: &World, c: CellXY) -> bool {
    ground(w, c).vehicle_present()
}

// spec: S-0209 claim 1 (a step sets the next cell's flag at once; the cell left is cleared when the first track point is passed, while the position is still in it)
#[test]
fn driving_vehicle_clears_the_cell_it_leaves_at_the_first_track_point() {
    let (mut w, id) = slow_driver(&[0, 0, 0, 0, 0]);
    let start = cell(0, 0);
    let next = cell(0, -1);
    assert!(flag(&w, start));
    // Run until the vehicle's position first moves.
    let p0 = w.obj(id).expect("v").pos;
    let mut moving = 0;
    for _ in 0..200 {
        tick::step(&mut w);
        let o = w.obj(id).expect("v");
        if moving == 0 && o.pos == p0 {
            continue;
        }
        moving += 1;
        assert!(
            flag(&w, next),
            "the next cell's flag is set from the start of the step"
        );
        assert_eq!(
            o.pos.cell(),
            start,
            "the position is still in the start cell"
        );
        // Budgets 2, 4, 6 buy no point; 8 buys the first (S-0207 claim 3).
        if moving < 4 {
            assert!(
                flag(&w, start),
                "tick {moving} of movement keeps the start cell's flag"
            );
        } else {
            assert!(!flag(&w, start), "the first point passed clears it");
            break;
        }
    }
    assert_eq!(moving, 4);
}

// spec: S-0209 claim 1 (on a two-cell curve the first cell's flag clears at the release point, position still in that cell; the target keeps its flag)
// spec: S-0207 table T1 (curve D: release point 19)
#[test]
fn curve_first_cell_clears_at_the_release_point() {
    let (mut w, id) = slow_driver(&[0, 2]);
    let first = cell(0, -1);
    let target = cell(1, -1);
    let mut cleared = false;
    for _ in 0..600 {
        tick::step(&mut w);
        let o = w.obj(id).expect("v");
        let Some(t) = o.drive.as_ref().and_then(|d| d.track.clone()) else {
            if cleared {
                break;
            }
            continue;
        };
        assert!(flag(&w, target), "the curve's target keeps its flag");
        if t.next <= 19 {
            assert!(
                flag(&w, first),
                "before the release point (next point {})",
                t.next
            );
        } else if !cleared {
            assert_eq!(
                t.next, 20,
                "cleared on the tick the release point is reached"
            );
            assert!(!flag(&w, first));
            assert_eq!(
                o.pos.cell(),
                first,
                "the position is still in the first cell"
            );
            cleared = true;
        }
    }
    assert!(cleared);
    assert!(
        flag(&w, target),
        "at rest the vehicle keeps its cell's flag"
    );
}

// spec: S-0209 claim 1 (a vehicle taken out of a cell's list clears that cell's flag)
#[test]
fn removed_vehicle_clears_its_flag() {
    let (mut w, id) = slow_driver(&[0]);
    assert!(flag(&w, cell(0, 0)));
    objects::remove(&mut w, id);
    assert!(!flag(&w, cell(0, 0)));
}

// ---------------------------------------------------------------------------
// Wall owners in the wall rows (claims 4 and 5; S-0210 claim 14)

/// A scene with a wall type STONE (overlay index 0) in `there()`, a walker
/// and a vehicle whose primary weapon can damage walls, and a crusher-all
/// vehicle; the wall's owner is set to `owner`.
fn wall_rows(owner: Option<HouseId>) -> (World, ObjectId, ObjectId, ObjectId) {
    let mut s = scene();
    let sim = &mut s.ruleset.sim;
    sim.weapons.insert(
        "Drill".into(),
        WeaponType {
            damage: Some(7),
            warhead: Some("Breaker".into()),
            ..Default::default()
        },
    );
    sim.warheads.insert(
        "Breaker".into(),
        WarheadType {
            wall: Some(true),
            ..Default::default()
        },
    );
    s.infantry(
        "Sapper",
        UnitType {
            primary: Some("Drill".into()),
            // S-0211 claim 10 (iii): a zone that may enter a wall cell.
            movement_zone: Some(MovementZone::InfantryDestroyer),
            ..walker(true)
        },
    )
    .vehicle(
        "Digger",
        UnitType {
            primary: Some("Drill".into()),
            movement_zone: Some(MovementZone::Destroyer),
            ..vehicle(true, false)
        },
    )
    .vehicle(
        "Flattener",
        UnitType {
            movement_zone: Some(MovementZone::CrusherAll),
            ..vehicle(true, true)
        },
    );
    s.register("overlays", "STONE");
    s.ruleset.sim.overlays.insert(
        "STONE".into(),
        glasnost_rules::sim::OverlayType {
            wall: Some(true),
            has_art: Some(true),
            ..Default::default()
        },
    );
    s.place_infantry(0, "Sapper", here(), 2);
    s.place_unit(0, "Digger", cell(0, 1), 0);
    s.place_unit(0, "Flattener", cell(1, 1), 0);
    s.cell_mut(there()).overlay = 0;
    let mut w = s.build();
    let i = w.cells.idx(there()).expect("cell");
    w.cells.wall_owner[i] = owner;
    let sapper = at(&w, Kind::Infantry, here());
    let digger = at(&w, Kind::Vehicle, cell(0, 1));
    let flat = at(&w, Kind::Vehicle, cell(1, 1));
    (w, sapper, digger, flat)
}

// spec: S-0209 claim 4 (rows WO3 and WO4: a wall of an allied house is a friendly obstacle, of another house or none an enemy obstacle)
// spec: S-0209 claim 5 (rows VO2, VO3 and VO5 by the wall's owner)
// spec: S-0210 claim 14 (b) (a wall without an owner counts as another house's)
#[test]
fn wall_rows_follow_the_wall_owner() {
    for (owner, friendly) in [
        (None, false),
        (Some(HouseId(1)), false),
        (Some(HouseId(0)), true),
    ] {
        let (w, sapper, digger, flat) = wall_rows(owner);
        let (obstacle, crushed) = if friendly {
            (Outcome::FriendlyObstacle, Outcome::FriendlyObstacle)
        } else {
            (Outcome::EnemyObstacle, Outcome::Clear)
        };
        assert_eq!(
            inf(&w, sapper, there()),
            obstacle,
            "walker, owner {owner:?}"
        );
        assert_eq!(
            veh(&w, digger, there()),
            obstacle,
            "vehicle, owner {owner:?}"
        );
        assert_eq!(
            veh(&w, flat, there()),
            crushed,
            "crusher-all, owner {owner:?}"
        );
    }
}

// spec: S-0209 claim 1 (a vehicle without the driving locomotor, taken off the map, clears its cell's flag)
#[test]
fn removed_non_driving_vehicle_clears_its_flag() {
    let mut s = Scene::new();
    let mut t = unit_type(TRACK, 6, 10);
    t.locomotor = Some(glasnost_rules::sim::Locomotor::Hover);
    s.raw_unit("vehicles", "Skimmer", t);
    s.place_unit(0, "Skimmer", cell(0, 0), 0);
    let mut w = s.build();
    let id = w
        .objects
        .values()
        .find(|o| o.kind == Kind::Vehicle)
        .map(|o| o.id)
        .expect("vehicle");
    assert!(w.obj(id).expect("v").drive.is_none());
    assert!(ground(&w, cell(0, 0)).vehicle_present());
    objects::remove(&mut w, id);
    assert!(!ground(&w, cell(0, 0)).vehicle_present());
}

// ---------------------------------------------------------------------------
// Gates (rows WP11 to WP14 and VP8 to VP11, with S-0215 claim 35)

/// Makes gate `g` open: *open* current and its door at rest open.
fn open_gate(w: &mut World, g: ObjectId) {
    let o = w.obj_mut(g).unwrap();
    o.beh.current = Some(glasnost_sim::rules::Behaviour::Open);
    o.beh.gate.door = glasnost_sim::behaviour::Door::Open;
}

// spec: S-0209 claim 4 (rows WP11 to WP14: an open gate gives none; a closed one allied (the gate owner's view) gives closed friendly gate, another enemy obstacle for an armed walker and blocked for an unarmed one)
#[test]
fn walker_gate_rows() {
    for (ty, enemy) in [("Pike", Outcome::EnemyObstacle), ("Mope", Outcome::Blocked)] {
        let (mut w, m) = asked(Kind::Infantry, ty, |s| {
            s.place_structure(0, "Door", there());
            s.place_structure(1, "Door", cell(1, 1));
        });
        assert_eq!(inf(&w, m, there()), Outcome::ClosedFriendlyGate, "{ty}");
        assert_eq!(inf(&w, m, cell(1, 1)), enemy, "{ty}");
        for c in [there(), cell(1, 1)] {
            let g = at(&w, Kind::Building, c);
            open_gate(&mut w, g);
            assert_eq!(inf(&w, m, c), Outcome::Clear, "{ty} {c:?}");
        }
    }
}

// spec: S-0209 claim 5 (rows VP8 to VP11: an open gate gives none; a closed one allied (the gate owner's view) gives closed friendly gate, another enemy obstacle for an armed vehicle and blocked for an unarmed one)
#[test]
fn vehicle_gate_rows() {
    for (ty, enemy) in [("Ram", Outcome::EnemyObstacle), ("Cart", Outcome::Blocked)] {
        let (mut w, m) = asked(Kind::Vehicle, ty, |s| {
            s.place_structure(0, "Door", there());
            s.place_structure(1, "Door", cell(1, 1));
        });
        assert_eq!(veh(&w, m, there()), Outcome::ClosedFriendlyGate, "{ty}");
        assert_eq!(veh(&w, m, cell(1, 1)), enemy, "{ty}");
        for c in [there(), cell(1, 1)] {
            let g = at(&w, Kind::Building, c);
            open_gate(&mut w, g);
            assert_eq!(veh(&w, m, c), Outcome::Clear, "{ty} {c:?}");
        }
        // Asking changes nothing: the gates stay as they were.
        let (w, m) = asked(Kind::Vehicle, ty, |s| {
            s.place_structure(0, "Door", there());
        });
        let g = at(&w, Kind::Building, there());
        let before = w.obj(g).unwrap().beh.clone();
        let _ = veh(&w, m, there());
        assert_eq!(w.obj(g).unwrap().beh, before);
    }
}

/// A corridor scene with a gate of house `house` in cell(2, 0) and a mover
/// of `kind`/`ty` (house 0) at cell(-2, 0) ordered to cell(5, 0). Returns
/// the world, the mover and the gate.
fn gate_corridor(kind: Kind, ty: &str, house: u8) -> (World, ObjectId, ObjectId) {
    let mut s = scene();
    corridor(&mut s);
    match kind {
        Kind::Infantry => s.place_infantry(0, ty, cell(-2, 0), 0),
        _ => s.place_unit(0, ty, cell(-2, 0), 64),
    };
    s.place_structure(house, "Door", cell(2, 0));
    let mut w = s.build();
    let m = at(&w, kind, cell(-2, 0));
    let g = at(&w, Kind::Building, cell(2, 0));
    open_orders(&mut w);
    order_move(&mut w, m, cell(5, 0));
    (w, m, g)
}

// spec: S-0209 claim 9 (a walker meeting a closed friendly gate makes it open, waits and walks on); S-0215 claim 35 (b) (the outcome makes open current)
#[test]
fn walker_passes_a_friendly_gate() {
    let (mut w, m, g) = gate_corridor(Kind::Infantry, "Mope", 0);
    let mut opened = false;
    let n = run_until(&mut w, 600, |w| {
        opened |= w.obj(g).unwrap().beh.current == Some(glasnost_sim::rules::Behaviour::Open);
        w.obj(m).unwrap().pos.cell() == cell(5, 0)
    });
    assert!(n < 600, "the walker never passed the gate");
    assert!(opened, "the gate never opened");
    // An enemy gate keeps an unarmed walker out.
    let (mut w, m, g) = gate_corridor(Kind::Infantry, "Mope", 1);
    run(&mut w, 300);
    assert!(w.obj(m).unwrap().pos.x < 256 * cell(2, 0).x);
    assert_ne!(
        w.obj(g).unwrap().beh.current,
        Some(glasnost_sim::rules::Behaviour::Open)
    );
}

// spec: S-0207 claim 15 (a vehicle in front of an allied closed gate tells it to open, waits and drives on once it is open)
#[test]
fn vehicle_passes_a_friendly_gate() {
    let (mut w, m, g) = gate_corridor(Kind::Vehicle, "Cart", 0);
    let mut opened = false;
    let n = run_until(&mut w, 900, |w| {
        opened |= w.obj(g).unwrap().beh.current == Some(glasnost_sim::rules::Behaviour::Open);
        w.obj(m).unwrap().pos.cell() == cell(5, 0)
    });
    assert!(n < 900, "the vehicle never passed the gate");
    assert!(opened, "the gate never opened");
    // An enemy gate keeps an unarmed vehicle out.
    let (mut w, m, _) = gate_corridor(Kind::Vehicle, "Cart", 1);
    run(&mut w, 300);
    assert!(w.obj(m).unwrap().pos.x < 256 * cell(2, 0).x);
}

// ---------------------------------------------------------------------------
// Row WO1 and the neutral and special houses (claims 4 and 11)

/// A scene with a crate overlay (registration index 0, with art) in
/// `there()` and a walker of house 0 in `here()`; `human`: house 0's
/// control. The map's crates are kept (quirks.skirmish_drops_map_crates
/// off), since claim 4 is about a crate in the cell, however it came there.
fn crate_scene(human: bool) -> (World, ObjectId) {
    asked(Kind::Infantry, "Pike", |s| {
        s.register("overlays", "Box");
        s.ruleset.sim.overlays.insert(
            "Box".into(),
            glasnost_rules::sim::OverlayType {
                crate_: Some(true),
                has_art: Some(true),
                ..Default::default()
            },
        );
        s.ruleset.globals.quirks.skirmish_drops_map_crates = false;
        s.map.has_overlay_data = true;
        s.cell_mut(there()).overlay = 0;
        s.setup.houses[0].human = human;
    })
}

// spec: S-0209 claim 4 (row WO1: a crate blocks a walker whose owner is not human-controlled, in our engine a walker of the neutral or special house; a human-controlled or AI house's walker gets no value from it)
#[test]
fn crate_blocks_walkers_of_non_player_houses() {
    let (w, m) = crate_scene(false);
    assert_eq!(w.cells.cell(there()).unwrap().overlay, 0, "crate placed");
    assert_eq!(inf(&w, m, there()), Outcome::Blocked);
    let (w, m) = crate_scene(true);
    assert_eq!(inf(&w, m, there()), Outcome::Clear);
}

// spec: S-0209 claim 11 (the neutral and special houses keep skill level 0: a crusher's warning, not move-anywhere, reaches their idle infantry only when the scatter threshold is 0 or less, as for a human player's; scenario C7 (c))
#[test]
fn non_player_houses_keep_skill_zero_for_the_crusher_warning() {
    for threshold in [3, 0] {
        let (mut w, ids) = idle_pair(|s| {
            s.setup.houses[1].human = false;
            s.ruleset.sim.general.iq_scatter = Some(threshold);
            s.place_unit(0, "Roller", cell(-1, 0), 64);
        });
        assert_eq!(w.houses[1].skill, 0);
        let roller = at(&w, Kind::Vehicle, cell(-1, 0));
        crush::warn_ahead(&mut w, roller, cell(0, 0));
        let reached = threshold <= 0;
        assert_eq!(
            obeyed(&w, &ids),
            [reached, reached],
            "threshold {threshold}"
        );
    }
}

// spec: S-0212 claim 27 and S-0209 claim 5 (rows VP20 and VP23, part 5e: an allied infantry unit gives a vehicle stationary friendly while it does not count as moving; once it counts as moving it gives no outcome of its own, and the record's allied infantry house gives moving blocker)
#[test]
fn moving_allied_infantry_answers_through_the_record() {
    let (mut w, m) = asked(Kind::Vehicle, "Cart", |s| {
        s.place_infantry(0, "Mope", there(), 2);
    });
    assert_eq!(veh(&w, m, there()), Outcome::StationaryFriendly);
    let man = at(&w, Kind::Infantry, there());
    w.obj_mut(man).unwrap().dest = Some(Target::Cell(cell(5, 5)));
    assert!(occupancy::counts_as_moving(&w, man));
    assert_eq!(veh(&w, m, there()), Outcome::MovingBlocker);
}

// spec: S-0209 claims 4 and 5 (the outcome reads the mover's primary weapon: walls, wood and, row VR10, the projectile hitting ground); our tooling for speed (M8 C4 part 5): a mover read once keeps what its primary weapon says, and answers every cell, in any order, as a mover read afresh for each cell does
#[test]
fn a_mover_read_once_answers_as_a_mover_read_afresh() {
    let cells = [
        there(),
        cell(0, 1),
        cell(1, 1),
        cell(2, 0),
        cell(0, 2),
        cell(2, 2),
        cell(3, 3),
    ];
    let movers = [
        (Kind::Infantry, "Mope"),
        (Kind::Infantry, "Pike"),
        (Kind::Infantry, "Sapper"),
        (Kind::Vehicle, "Cart"),
        (Kind::Vehicle, "Ram"),
        (Kind::Vehicle, "Chopper"),
        (Kind::Vehicle, "Dozer"),
        (Kind::Vehicle, "Plough"),
        (Kind::Vehicle, "Barrow"),
        (Kind::Vehicle, "Smasher"),
        (Kind::Vehicle, "Lobber"),
    ];
    let mut answers = std::collections::BTreeSet::new();
    for (kind, ty) in movers {
        let (w, m) = asked(kind, ty, |s| {
            walls(s, false);
            s.ruleset.sim.weapons.insert(
                "Lob".into(),
                WeaponType {
                    damage: Some(7),
                    warhead: Some("Woody".into()),
                    projectile: Some("Arc".into()),
                    ..Default::default()
                },
            );
            s.ruleset.sim.projectiles.insert(
                "Arc".into(),
                glasnost_rules::sim::ProjectileType {
                    ag: Some(false),
                    ..Default::default()
                },
            );
            s.vehicle(
                "Lobber",
                UnitType {
                    primary: Some("Lob".into()),
                    ..vehicle(true, false)
                },
            );
            s.place_terrain("Bramble", cell(0, 1));
            s.place_unit(1, "Cart", cell(1, 1), 0);
            s.place_infantry(1, "Mope", cell(2, 0), 2);
            s.place_infantry(0, "Mope", cell(0, 2), 2);
            s.place_infantry(1, "Pike", cell(2, 2), 2);
        });
        let level = occupancy::reference_level(&w, m);
        let fresh = |c: CellXY| match kind {
            Kind::Infantry => inf(&w, m, c),
            _ => veh(&w, m, c),
        };
        let once = |mv: &occupancy::Mover<'_>, c: CellXY| match kind {
            Kind::Infantry => occupancy::infantry_outcome_of(&w, mv, c, level),
            _ => occupancy::vehicle_outcome_of(&w, mv, c, level),
        };
        for start in 0..cells.len() {
            let mv = occupancy::Mover::of(&w, m).expect("the mover exists");
            for k in 0..cells.len() {
                let c = cells[(start + k) % cells.len()];
                let a = once(&mv, c);
                assert_eq!(a, fresh(c), "{ty} asked {c:?} from {start}");
                answers.insert(format!("{a:?}"));
            }
        }
    }
    assert!(
        answers.len() >= 4,
        "the scene asks varied cells: {answers:?}"
    );
}
