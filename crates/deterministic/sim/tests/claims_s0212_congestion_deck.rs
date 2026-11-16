//! S-0212 claim 19 (#305; r107 answers Q-3122): a re-path on a bridge deck
//! reads the cell ahead on its deck when the blocked unit is on a deck, so
//! the unit blocking it there is considered; part (ii) reads each cell of
//! the 5 × 5 block from its ground record (`quirks.path_congestion_ground_record`
//! on), or a bridge cell from the layer read in the cell ahead (off).
//! Synthetic types and map from the shared
//! `common` helpers: a fast car TESTCAR at (3, 0) facing east, a slower
//! vehicle TESTSLOW of another type at (4, 0) with five stored steps east;
//! every name and number is made up.

mod common;

use common::{Scene, WHEEL, cell, objects_of, unit_type};
use glasnost_map::CellXY;
use glasnost_sim::path::{self, Mode};
use glasnost_sim::regions;
use glasnost_sim::rules::Kind;
use glasnost_sim::trace::{self, What};
use glasnost_sim::world::{Layer, ObjectId, Pos, Target, World};

/// Moves a placed vehicle from the ground list of its cell to the deck list.
fn raise(w: &mut World, id: ObjectId) {
    let c = w.obj(id).unwrap().pos.cell();
    w.cells.leave(c, Layer::Ground, id);
    w.cells.claim_vehicle(c, Layer::Ground, false);
    let o = w.obj_mut(id).unwrap();
    o.pos = Pos::centre(c, 416);
    o.layer = Layer::Deck;
    w.cells.enter(c, Layer::Deck, id, false);
    w.cells.claim_vehicle(c, Layer::Deck, true);
}

/// Returns the mode A's mode-1 re-path ran in and its congestion-marked cells.
fn run(deck: bool) -> (Option<Mode>, Vec<CellXY>) {
    run_with(deck, true)
}

/// [`run`] with `quirks.path_congestion_ground_record` set to `ground_record`.
fn run_with(deck: bool, ground_record: bool) -> (Option<Mode>, Vec<CellXY>) {
    let mut s = Scene::new();
    s.ruleset.globals.quirks.path_congestion_ground_record = ground_record;
    s.vehicle("TESTCAR", unit_type(WHEEL, 40, 6));
    s.vehicle("TESTSLOW", unit_type(WHEEL, 20, 6));
    if deck {
        let cells: Vec<CellXY> = s.map.cells.iter().map(|(c, _)| c).collect();
        for c in cells {
            let pit = (cell(3, 0).x..=cell(12, 0).x).contains(&c.x)
                && (cell(0, -1).y..=cell(0, 2).y).contains(&c.y);
            s.set_cell(c, 0, if pit { 0 } else { 4 });
        }
    }
    s.place_unit(0, "TESTCAR", cell(3, 0), 64);
    s.place_unit(0, "TESTSLOW", cell(4, 0), 64);
    let mut w = s.build();
    let at = |w: &World, c: CellXY| {
        objects_of(w, Kind::Vehicle)
            .into_iter()
            .find(|&v| w.obj(v).unwrap().pos.cell() == c)
            .unwrap()
    };
    let a = at(&w, cell(3, 0));
    let b = at(&w, cell(4, 0));
    if deck {
        for x in 3..=12 {
            for y in 0..=1 {
                let i = w.cells.idx(cell(x, y)).unwrap();
                w.cells.deck[i] = true;
                w.cells.deck_entry[i] = true;
                w.cells.bridge_axis[i] = glasnost_sim::cells::Axis::X;
            }
        }
        w.regions.dirty = true;
        raise(&mut w, a);
        raise(&mut w, b);
        regions::ensure(&mut w);
        assert!(w.cells.list(cell(4, 0), Layer::Deck).contains(&b));
        assert!(w.cells.list(cell(4, 0), Layer::Ground).is_empty());
    }
    {
        let o = w.obj_mut(b).unwrap();
        o.dest = Some(Target::Cell(cell(-20, 10)));
        o.path.retry_at = i32::MAX;
        o.path.steps = vec![2; 5];
    }
    if deck {
        let r = glasnost_sim::occupancy::reference_level(&w, a);
        assert_eq!(
            path::step_outcome(&w, a, cell(3, 0), cell(4, 0), r),
            glasnost_sim::occupancy::Outcome::MovingBlocker,
            "reference level {r}"
        );
    }
    trace::switch_on(&mut w);
    w.obj_mut(a).unwrap().dest = Some(Target::Cell(cell(12, 0)));
    let res = path::compute(&mut w, a, Mode::Blocked);
    let events = trace::take(&w);
    for e in &events {
        if let What::Search {
            asked,
            mode,
            congested,
            ..
        } = &e.what
            && e.unit == a
        {
            assert_eq!(*asked, Mode::Blocked);
            return (*mode, congested.clone());
        }
    }
    panic!("no search event: {res:?} {events:?}");
}

// spec: S-0212 claim 19 (mode 1: a slower unit of another type in the cell ahead with two or more stored steps qualifies; (i) counts the cells of its remaining stored steps) (#305)
#[test]
fn control_on_the_ground() {
    let (mode, marked) = run(false);
    assert_eq!(mode, Some(Mode::Blocked));
    for x in 6..=9 {
        assert!(marked.contains(&cell(x, 0)), "{marked:?}");
    }
}

// spec: S-0212 claim 19 (the same two units on a bridge deck: the cell ahead is read on its deck, the blocked unit being on a deck) (#305)
#[test]
fn the_same_on_a_bridge_deck() {
    let (mode, marked) = run(true);
    assert_eq!(mode, Some(Mode::Blocked), "marked: {marked:?}");
    for x in 6..=9 {
        assert!(marked.contains(&cell(x, 0)), "{marked:?}");
    }
}

// spec: S-0212 claim 19 ((ii): a cell of the block is occupied from its ground record only; the cell ahead, with its blocker on the deck, is counted only in (iii) and so carries the mark) and Edge cases (quirks.path_congestion_ground_record off: a bridge cell reads the record of the layer read in the cell ahead, the deck here, so the cell ahead is counted twice) (#305)
#[test]
fn deck_blocker_cell_counts_by_the_ground_record() {
    for ground_record in [true, false] {
        let (_, marked) = run_with(true, ground_record);
        assert_eq!(
            marked.contains(&cell(4, 0)),
            ground_record,
            "switch {ground_record}: {marked:?}"
        );
        let (_, flat) = run_with(false, ground_record);
        assert!(!flat.contains(&cell(4, 0)), "on the ground: {flat:?}");
    }
}

/// The 5 × 5 search of claim 19's (i) on the ground: the cell ahead (4, 0)
/// is empty, and TESTSLOW stands at (5, 1) facing north-west with five
/// stored steps south; with `heading` it is on a track that ends in the
/// cell ahead. Returns the mode A's mode-1 re-path ran in and its
/// congestion-marked cells.
fn run_block(heading: bool) -> (Option<Mode>, Vec<CellXY>) {
    let mut s = Scene::new();
    s.vehicle("TESTCAR", unit_type(WHEEL, 40, 6));
    s.vehicle("TESTSLOW", unit_type(WHEEL, 20, 6));
    s.place_unit(0, "TESTCAR", cell(3, 0), 64);
    // Facing 224: north-west, so that its heading cell is the cell ahead.
    s.place_unit(0, "TESTSLOW", cell(5, 1), 224);
    let mut w = s.build();
    let at = |w: &World, c: CellXY| {
        objects_of(w, Kind::Vehicle)
            .into_iter()
            .find(|&v| w.obj(v).unwrap().pos.cell() == c)
            .unwrap()
    };
    let a = at(&w, cell(3, 0));
    let b = at(&w, cell(5, 1));
    assert!(w.cells.list(cell(4, 0), Layer::Ground).is_empty(), "set-up: nothing listed ahead");
    {
        let o = w.obj_mut(b).unwrap();
        o.dest = Some(Target::Cell(cell(4, 10)));
        o.path.retry_at = i32::MAX;
        o.path.steps = vec![4; 5];
        let z = o.pos.z;
        let (x, y) = (o.pos.x, o.pos.y);
        if heading {
            let d = o.drive.as_mut().expect("set-up: a driving vehicle");
            let t = cell(4, 0);
            d.track = Some(glasnost_sim::drive::Track {
                d: 7,
                e: None,
                target: (256 * t.x + 128, 256 * t.y + 128),
                target_cell: t,
                height: z,
                next: 0,
                last: (x, y),
                cells: vec![t],
                crush: false,
                exit: false,
                bunker: None,
            });
        }
    }
    trace::switch_on(&mut w);
    w.obj_mut(a).unwrap().dest = Some(Target::Cell(cell(12, 0)));
    let res = path::compute(&mut w, a, Mode::Blocked);
    let events = trace::take(&w);
    for e in &events {
        if let What::Search {
            asked,
            mode,
            congested,
            ..
        } = &e.what
            && e.unit == a
        {
            assert_eq!(*asked, Mode::Blocked);
            return (*mode, congested.clone());
        }
    }
    panic!("no search event: {res:?} {events:?}");
}

// spec: S-0212 claim 19 (the cell ahead lists no unit: the first unit of the 5 × 5 block around it, rows by increasing y and each by increasing x, whose locomotion heads to the cell ahead by S-0511 claim 11's test is considered; (i) counts the cells of its remaining stored steps after the cell it moves into; mode 1 with no cell from (i) runs as mode 0) (S20's seam of #305)
#[test]
fn an_empty_cell_ahead_considers_a_unit_of_the_block_heading_to_it() {
    let (mode, marked) = run_block(true);
    assert_eq!(mode, Some(Mode::Blocked), "marked: {marked:?}");
    for y in 3..=5 {
        assert!(marked.contains(&cell(4, y)), "{marked:?}");
    }
    // Control: not heading to the cell ahead, it is not found.
    let (mode, marked) = run_block(false);
    assert_ne!(mode, Some(Mode::Blocked), "marked: {marked:?}");
}
