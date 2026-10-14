//! Claim tests for a house's base records and its house-selected centre
//! (S-0215 claim 38, F11 and F12) on synthetic data: invented building
//! types and prices on the flat scene of `common`.

mod common;

use common::*;
use glasnost_map::CellXY;
use glasnost_num::geom::root_floor;
use glasnost_rules::sim::BuildingType;
use glasnost_sim::base;
use glasnost_sim::econ::TypeRef;
use glasnost_sim::rules::Kind;
use glasnost_sim::world::{HouseId, ObjectId, Pos, World};

const A: HouseId = HouseId(0);

/// A scene with two buildings of house 0: DEAR (price 1500, weight 2) at
/// cell (0, 0) and CHEAP (price 500, weight 1) at cell (6, 0).
fn scene() -> Scene {
    let mut s = Scene::new();
    for (name, cost) in [("DEAR", 1500), ("CHEAP", 500)] {
        s.building(
            name,
            BuildingType {
                strength: Some(100),
                foundation: Some([1, 1]),
                cost: Some(cost),
                ..Default::default()
            },
        );
    }
    s.place_structure(0, "DEAR", cell(0, 0));
    s.place_structure(0, "CHEAP", cell(6, 0));
    s
}

fn buildings(w: &World) -> Vec<ObjectId> {
    objects_of(w, Kind::Building)
}

fn weight(w: &World, id: ObjectId) -> i32 {
    let t = TypeRef {
        kind: Kind::Building,
        index: w.obj(id).unwrap().type_index,
    };
    let p = glasnost_sim::econ::production::price(w, A, t);
    (p / 1000 + 1).max(0)
}

fn cell_point(w: &World, c: CellXY) -> Pos {
    let p = Pos::centre(c, 0);
    Pos::centre(c, w.surface_at(p.x, p.y))
}

fn rho3(w: &World, a: Pos, b: Pos) -> i32 {
    let (dx, dy, dz) = (
        i64::from(a.x - b.x),
        i64::from(a.y - b.y),
        i64::from(a.z - b.z),
    );
    root_floor(
        (dx * dx + dy * dy + dz * dz) as u64,
        w.rules().quirks.table_root_distance,
        w.rules().mode,
    ) as i32
}

// spec: S-0215 claim 38 and F12 (an owned building's entry refreshes the base-start cell from the price-weighted mean of the buildings' reference points, through a nearby-cell search, and the radius max(512, trunc(Σ distance₃ / Q)), each building counted once in the sum)
#[test]
fn claim_38_refresh_on_entry() {
    let w = scene().build();
    let ids = buildings(&w);
    let ws: Vec<i32> = ids.iter().map(|&id| weight(&w, id)).collect();
    assert_eq!(ws, vec![2, 1]);
    let q: i32 = ws.iter().sum();
    let pts: Vec<Pos> = ids.iter().map(|&id| w.reference_point(id)).collect();
    let mx = (ws[0] * pts[0].x + ws[1] * pts[1].x) / q / 256;
    let my = (ws[0] * pts[0].y + ws[1] * pts[1].y) / q / 256;
    let rec = &w.houses[0].base;
    let start = rec.start.expect("a base-start cell");
    // The mean's cell is free open ground, so the nearby search keeps it.
    assert_eq!(start, CellXY::new(mx, my));
    let p = cell_point(&w, start);
    let sum: i32 = pts.iter().map(|&b| rho3(&w, p, b)).sum();
    assert_eq!(rec.radius, (sum / q).max(512));
    assert_eq!(rec.centre, None, "no map action sets a base centre");
}

// spec: S-0215 claim 38 and F12 (a removal refreshes from the surviving buildings: one building of weight 1 gives Q ≤ 1 and radius 512; none leaves the base-start cell absent and the radius 0)
#[test]
fn claim_38_refresh_on_removal() {
    let mut w = scene().build();
    let ids = buildings(&w);
    glasnost_sim::objects::remove(&mut w, ids[0]);
    let rec = w.houses[0].base.clone();
    assert_eq!(rec.radius, 512);
    // The survivor's reference point is the whole mean; the building
    // stands on that cell, so the nearby search gives a neighbouring cell.
    let r = w.reference_point(ids[1]);
    let s = rec.start.expect("a base-start cell");
    let (dx, dy) = (s.x - r.x / 256, s.y - r.y / 256);
    assert!(dx.abs() <= 1 && dy.abs() <= 1 && (dx, dy) != (0, 0), "{dx} {dy}");
    glasnost_sim::objects::remove(&mut w, ids[1]);
    let rec = &w.houses[0].base;
    assert_eq!((rec.start, rec.radius), (None, 0));
}

/// The rng after one U(0, R) and one raw draw from `w`'s.
fn after_draws(w: &World, r: i32) -> String {
    let mut rng = w.rng.clone();
    let half = w.rules().quirks.ranged_draw_half_span;
    let _ = rng.ranged(0, r, half);
    let _ = rng.raw();
    format!("{rng:?}")
}

// spec: S-0215 claim 38 and F11 (with neither record the displacement starts from the world origin: one U(0, R) with R clamped up to 768, then exactly one raw value; the point is snapped and corrected into the usable area)
#[test]
fn claim_38_no_base_centre_from_the_origin() {
    let mut w = Scene::new().build();
    assert_eq!(w.houses[0].base, base::BaseRecord::default());
    let want = after_draws(&w, 768);
    let c = base::selected_centre(&mut w, A).expect("a centre");
    assert_eq!(format!("{:?}", w.rng), want);
    assert!(w.cells.usable(c), "corrected into the usable area");
}

// spec: S-0215 claim 38 (the radius of the random choice is the recorded radius clamped to 768 … 2048; the displacement starts from the preferred recorded cell)
#[test]
fn claim_38_radius_clamp_and_preferred_cell() {
    let mut w = Scene::new().build();
    w.houses[0].base.start = Some(cell(0, 0));
    w.houses[0].base.radius = 5000;
    let want = after_draws(&w, 2048);
    let c = base::selected_centre(&mut w, A).expect("a centre");
    assert_eq!(format!("{:?}", w.rng), want);
    // Within the clamped radius of the base-start cell, snapped to a cell.
    let (dx, dy) = (c.x - cell(0, 0).x, c.y - cell(0, 0).y);
    assert!(dx.abs() <= 9 && dy.abs() <= 9, "{dx} {dy}");
    // A recorded base centre is preferred to the base-start cell.
    let mut v = Scene::new().build();
    v.houses[0].base.start = Some(cell(-20, -20));
    v.houses[0].base.centre = Some(cell(0, 0));
    v.houses[0].base.radius = 0;
    let c = base::selected_centre(&mut v, A).expect("a centre");
    let (dx, dy) = (c.x - cell(0, 0).x, c.y - cell(0, 0).y);
    assert!(dx.abs() <= 4 && dy.abs() <= 4, "{dx} {dy}");
}
