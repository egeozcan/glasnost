//! The object under the cursor (#435; S-0214 claim 35 and F10) through the
//! windowless client on synthetic data: the invented rules and map of
//! `glasnost_ai::fixture` and an invented rules text that names the
//! fixture's vehicle types, so that a vehicle placed after the start takes
//! part in the body drawing. No art is loaded: nothing is drawn at any
//! pixel, so whatever is found is found by the candidates' points and the
//! cell under the cursor alone.

use super::access_tests::{client, own, session_mut, start_by_keyboard_with};
use super::*;
use glasnost_ai::fixture;
use glasnost_convert::inputs::{InputFile, Inputs};
use glasnost_convert::{ImportOptions, convert};
use glasnost_map::CellXY;
use glasnost_render::projection::project_lifted;
use glasnost_sim::rules::Kind;
use glasnost_sim::testkit;
use glasnost_sim::world::ObjectId;

/// An import that names the fixture's vehicle types, so that the view finds
/// an import type for a vehicle met after the start.
fn import() -> glasnost_convert::Import {
    let text = "[VehicleTypes]\n0=MCV\n1=MINR\n2=GUNR\n3=LNCH\n4=TRUK\n\
        [BuildingTypes]\n0=POWR\n[MCV]\n[MINR]\n[GUNR]\n[LNCH]\n[TRUK]\n[POWR]\n";
    convert(
        &Inputs {
            base: Some(InputFile {
                name: "rules.ini".into(),
                origin: "test".into(),
                bytes: text.as_bytes().to_vec(),
            }),
            ..Default::default()
        },
        &ImportOptions::default(),
    )
}

/// A started game with the cells around the local base unit explored (an
/// area reveal of radius 8, S-0409 claim 17), and the base unit.
fn game() -> (Client, ObjectId) {
    let mut c = client((800, 600), 100);
    start_by_keyboard_with(&mut c, fixture::rules());
    let mcv = own(&c, Kind::Vehicle)[0];
    let s = session_mut(&mut c);
    s.loaded.import = import();
    let me = s.play.me();
    let at = s.play.world().obj(mcv).unwrap().pos;
    glasnost_sim::econ::shroud::area_reveal(&mut s.play.game.world, me, at, 8);
    (c, mcv)
}

/// The cell of an object.
fn cell_of(s: &Session, id: ObjectId) -> CellXY {
    s.play.world().obj(id).unwrap().pos.cell()
}

/// An object of type `name` of the local house, `(dx, dy)` cells from the
/// base unit's cell.
fn place(s: &mut Session, base: ObjectId, kind: Kind, name: &str, d: (i32, i32)) -> ObjectId {
    let b = cell_of(s, base);
    let house = s.play.me().0;
    testkit::spawn(
        &mut s.play.game.world,
        kind,
        name,
        house,
        CellXY::new(b.x + d.0, b.y + d.1),
        0,
    )
}

/// The screen pixel of an object's point (S-0214 Terms, *box point*: its
/// position projected with the world-height lift, S-0101 F1 and F2),
/// moved by `(dx, dy)` pixels.
fn pixel(s: &Session, id: ObjectId, d: (i32, i32)) -> Point {
    let o = s.play.world().obj(id).unwrap();
    let (qx, qy) = project_lifted(o.pos.x, o.pos.y, o.pos.z);
    let v = s.view();
    let m = s.map_rect();
    Point::new(qx - v.origin.0 + m.x + d.0, qy - v.origin.1 + m.y + d.1)
}

/// #435. spec: S-0214 claim 35 (a candidate is tested at its box point: a unit in the air at its
/// lifted drawing origin, which rises and falls with its height lift from one drawn frame to the
/// next; the pixels drawn play no part) and (a), F10 (in reach exactly when Δx² + ⌊Δy²/2⌋ < 200:
/// up to 14 pixels to either side and 19 rows above or below)
///
/// A vehicle in the air, its height changed between the picks as a hovering unit's is from one
/// drawn frame to the next, is found at a pointer held within its region of each height, though
/// nothing is drawn at the pointer.
#[test]
fn claim_35_a_flying_unit_is_found_at_its_point_whatever_its_frame() {
    let (mut c, mcv) = game();
    let s = session_mut(&mut c);
    let unit = place(s, mcv, Kind::Vehicle, "GUNR", (2, 0));
    // The view meets the vehicle in a tick (its layer and import type).
    s.step();
    for z in [1000, 1040, 960, 1000] {
        s.play.game.world.obj_mut(unit).unwrap().pos.z = z;
        let at = pixel(s, unit, (0, 0));
        for d in [(0, 0), (14, 0), (-14, 0), (0, 19), (0, -19), (9, 9)] {
            let p = Point::new(at.x + d.0, at.y + d.1);
            assert_eq!(s.under(p).1, Some(unit), "height {z}, {d:?} from its point");
        }
        // spec: S-0214 F10 (|Δx| = 15 at Δy = 0 is out of reach); its cell lies far below
        let p = Point::new(at.x + 15, at.y);
        assert_ne!(s.under(p).1, Some(unit), "height {z}: out of reach");
    }
}

/// #435. spec: S-0214 claim 35 (a building is tested at its height-lifted projected position,
/// claim 24) and (a), F10; (d) (a shrouded cell under the cursor gives no clicked object, even
/// when a candidate's region holds the pointer)
#[test]
fn claim_35_a_building_is_found_at_its_lifted_point() {
    let (mut c, mcv) = game();
    let s = session_mut(&mut c);
    let powr = place(s, mcv, Kind::Building, "POWR", (3, -3));
    for d in [(0, 0), (14, 0), (-14, 0), (0, 19), (0, -19)] {
        let p = pixel(s, powr, d);
        assert_eq!(s.under(p).1, Some(powr), "{d:?} from its point");
    }
}

/// #435. spec: S-0214 claim 35 (d) (before any candidate is tested, a shrouded cell under the
/// cursor gives no clicked object, even when a candidate's region holds the pointer)
#[test]
fn claim_35_a_shrouded_cell_finds_nothing() {
    let (mut c, mcv) = game();
    let s = session_mut(&mut c);
    let me = s.play.me();
    let m = s.map_rect();
    let inside =
        |p: Point| p.x > m.x + 40 && p.x < m.x + m.w - 40 && p.y > m.y + 40 && p.y < m.y + m.h - 40;
    // A building on a shrouded cell in the view, the cell under its point shrouded.
    let b = cell_of(s, mcv);
    let v = s.view();
    let d = (6..14)
        .flat_map(|r| [(r, 0), (0, r), (-r, 0), (0, -r), (r, r), (-r, -r)])
        .find(|&(dx, dy)| {
            let c = CellXY::new(b.x + dx, b.y + dy);
            let centre = glasnost_render::projection::cell_centre(c.x, c.y);
            let (qx, qy) = project_lifted(centre.0, centre.1, 0);
            let p = Point::new(qx - v.origin.0 + m.x, qy - v.origin.1 + m.y);
            inside(p) && !glasnost_sim::econ::shroud::explored(s.play.world(), me, c)
        })
        .expect("a shrouded cell in the view");
    // Another house's building: placing one of the local house's would
    // reveal its cells.
    let other = s
        .play
        .world()
        .objects
        .values()
        .find_map(|o| o.owner.filter(|&h| h != me))
        .expect("another house's object");
    let id = testkit::spawn(
        &mut s.play.game.world,
        Kind::Building,
        "POWR",
        other.0,
        CellXY::new(b.x + d.0, b.y + d.1),
        0,
    );
    let p = pixel(s, id, (0, 0));
    let (cell, _) = s.under(p);
    assert!(
        !glasnost_sim::econ::shroud::explored(s.play.world(), me, cell),
        "{cell:?} under the building's point is shrouded"
    );
    assert_eq!(s.under(p), (cell, None), "a shrouded cell");
}

/// #435. spec: S-0214 claim 35 (a) (between candidates of equal d the one first in claim 25's
/// order is kept: earlier-created buildings, then mobile objects) and F10 (Δy = 0 and Δy = 1 at
/// the same Δx give the same d)
#[test]
fn claim_35_a_tie_keeps_the_first_candidate() {
    let (mut c, mcv) = game();
    let s = session_mut(&mut c);
    let first = place(s, mcv, Kind::Building, "POWR", (3, -3));
    let second = place(s, mcv, Kind::Building, "POWR", (-3, 3));
    let unit = place(s, mcv, Kind::Vehicle, "GUNR", (2, 0));
    s.step();
    // The later building one row above the first: at its own point d is 0
    // for both, ⌊1/2⌋ = 0 for the first.
    let at = s.play.world().obj(first).unwrap().pos;
    let lifted = |z: i32| project_lifted(at.x, at.y, z).1;
    let dz = (1..64)
        .find(|&dz| lifted(at.z + dz) == lifted(at.z) - 1)
        .expect("a height one row up");
    s.play.game.world.obj_mut(second).unwrap().pos =
        glasnost_sim::world::Pos::new(at.x, at.y, at.z + dz);
    let p = pixel(s, second, (0, 0));
    assert_eq!(pixel(s, first, (0, 0)).y, p.y + 1);
    assert_eq!(s.under(p).1, Some(first), "the earlier building is kept");
    // A mobile object at the first building's point: the building is kept.
    s.play.game.world.obj_mut(second).unwrap().pos =
        glasnost_sim::world::Pos::new(at.x, at.y, at.z + 4000);
    s.play.game.world.obj_mut(unit).unwrap().pos = at;
    let p = pixel(s, first, (5, 0));
    assert_eq!(
        s.under(p).1,
        Some(first),
        "a building before a mobile object"
    );
    // The unit 15 pixels east of the building's point, the pointer 3 west of
    // the unit's: d = 9 for the unit, 144 for the building.
    let p = pixel(s, first, (0, 0));
    s.play.game.world.obj_mut(unit).unwrap().pos =
        glasnost_sim::world::Pos::new(at.x + 64, at.y - 64, at.z);
    let q = pixel(s, unit, (0, 0));
    assert_eq!((q.x, q.y), (p.x + 15, p.y), "the unit 15 pixels east");
    assert_eq!(
        s.under(Point::new(q.x - 3, q.y)).1,
        Some(unit),
        "the nearer candidate"
    );
}

/// #435. spec: S-0214 claim 35 (c) (no candidate in reach: the first occupant of the cell under the
/// cursor, S-0101 claim 10, on its ground; with no occupant, no clicked object) and S-0209 claim 13
/// (listing order); claim 24 (an object that is no candidate is found this way)
///
/// The base unit is no candidate: its type was met before the import named it, so the view finds
/// no import type for it and it takes no part in the body drawing.
#[test]
fn claim_35_out_of_reach_the_cells_occupant_or_nothing() {
    let (mut c, mcv) = game();
    let s = session_mut(&mut c);
    s.step();
    let cell = cell_of(s, mcv);
    // 20 pixels east of its point, still inside its cell's diamond.
    let p = pixel(s, mcv, (20, 0));
    assert_eq!(s.under(p), (cell, Some(mcv)), "the cell's occupant");
    // An empty explored cell two cells away from every object: the cell, no object.
    let p = pixel(s, mcv, (0, 90));
    let (empty, id) = s.under(p);
    assert!(
        s.play
            .world()
            .cells
            .list(empty, glasnost_sim::world::Layer::Ground)
            .is_empty(),
        "{empty:?} is empty"
    );
    assert_eq!(id, None, "no occupant, no object");
}
