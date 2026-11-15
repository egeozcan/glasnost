//! S-0212 claims 7, 17 and 18 (#307): path straightening judges a shape
//! that starts on a cell an earlier replacement put on the path from the
//! layer the path reaches that cell on (the deck), not from the ground under
//! a bridge deck. Synthetic types and map from the shared `common` helpers
//! (TESTCAR, TESTTRAIN); every name and number is made up.

mod common;

use common::unit_type;
use common::{Scene, WHEEL, cell, objects_of};
use glasnost_map::CellXY;
use glasnost_rules::sim::UnitType;
use glasnost_sim::path::{self, Mode, Pathfinder, Request, S0212, Search};
use glasnost_sim::rules::Kind;
use glasnost_sim::world::{Layer, Pos, Target, World};

const CAR: &str = "TESTCAR";
const TRAIN: &str = "TESTTRAIN";

/// A deck over flat open ground, x = -3..=4, y = -3..=6, with holes (cells
/// that carry no deck) where S-0212 claim 18's example (a) has its walls.
/// A train stands on the deck at (0, 0) facing north and is sent to (1, 4).
/// With `under`, a car stands on the ground under deck cell (1, 2).
/// Returns the path the search finds and the stored path.
fn run(under: bool) -> (Vec<u8>, Vec<u8>) {
    let mut s = Scene::new();
    s.vehicle(CAR, unit_type(WHEEL, 40, 6));
    s.vehicle(
        TRAIN,
        UnitType {
            is_train: Some(true),
            ..unit_type(WHEEL, 40, 6)
        },
    );
    s.place_unit(0, TRAIN, cell(0, 0), 0);
    if under {
        s.place_unit(0, CAR, cell(1, 2), 0);
    }
    let mut w: World = s.build();
    let holes = [cell(-2, 0), cell(-1, 0), cell(1, 0), cell(0, 2), cell(0, 3)];
    for y in -3..=6 {
        for x in -3..=4 {
            let c: CellXY = cell(x, y);
            if holes.contains(&c) {
                continue;
            }
            let i = w.cells.idx(c).unwrap();
            w.cells.deck[i] = true;
            w.cells.deck_entry[i] = true;
            w.cells.bridge_axis[i] = glasnost_sim::cells::Axis::X;
        }
    }
    w.regions.dirty = true;
    let t = objects_of(&w, Kind::Vehicle)
        .into_iter()
        .find(|&v| w.obj(v).unwrap().pos.cell() == cell(0, 0))
        .unwrap();
    {
        let o = w.obj_mut(t).unwrap();
        o.pos = Pos::centre(cell(0, 0), 416);
        o.layer = Layer::Deck;
    }
    glasnost_sim::regions::ensure(&mut w);
    let goal = cell(1, 4);
    let found = S0212
        .find(
            &w,
            &Request {
                unit: t,
                from: cell(0, 0),
                to: goal,
                mode: Mode::Fresh,
                congested: Default::default(),
                avoided: Default::default(),
            },
        )
        .expect("a path on the deck");
    // Every deck cell the shortened path would enter is clear on the deck.
    for c in [cell(1, 1), cell(1, 2), cell(1, 3), cell(1, 4)] {
        assert!(
            w.cells.list(c, Layer::Deck).is_empty(),
            "deck cell {c:?} is empty"
        );
    }
    w.obj_mut(t).unwrap().dest = Some(Target::Cell(goal));
    assert_eq!(path::compute(&mut w, t, Mode::Fresh), Search::Found);
    (found, w.obj(t).unwrap().path.steps.clone())
}

// spec: S-0212 claim 18 (example (a); the acceptance test sees the path as the earlier replacements of this pass have left it), claim 17 (every cell entered has entry outcome clear) and claim 7 (the layer a step reaches) (#307)
#[test]
fn a_unit_under_the_deck_changes_the_straightened_deck_path() {
    let (found_a, stored_a) = run(false);
    let (found_b, stored_b) = run(true);
    // The search runs on the deck and does not see the car under it.
    assert_eq!(found_a, vec![1, 3, 4, 5, 4, 4]);
    assert_eq!(found_b, found_a);
    // S-0212 claim 18, example (a): 1, 3, 4, 5, 4, 4 becomes 3, 4, 4, 4.
    assert_eq!(stored_a, vec![3, 4, 4, 4]);
    // The same deck, the same found path, every deck cell clear: the same result.
    assert_eq!(stored_b, stored_a);
}
