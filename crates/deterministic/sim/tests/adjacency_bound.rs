//! The adjacency condition looks only at cells that can hold a building,
//! whatever the type's adjacency field (#201). Synthetic data only.

use std::sync::mpsc;
use std::time::Duration;

use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::BuildingType;
use glasnost_sim::econ::{TypeRef, placement};
use glasnost_sim::rules::Kind;
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, World};

const SIZE: i32 = 24;
const A: HouseId = HouseId(0);

fn building(adjacent: Option<i32>) -> BuildingType {
    let mut b = BuildingType {
        strength: Some(400),
        foundation: Some([1, 1]),
        owner: Some(vec!["Alphaland".into(), "Betaland".into()]),
        ..Default::default()
    };
    b.econ.adjacent = adjacent;
    b
}

const TYPES: [(&str, Option<i32>); 6] = [
    ("HUT", None),
    ("WIDE", Some(9999)),
    ("VAST", Some(2_000_000_000)),
    ("HUGE", Some(i32::MAX)),
    ("NONE", Some(-1)),
    ("LEAST", Some(i32::MIN)),
];

fn rules() -> Ruleset {
    let mut r = testkit::ruleset();
    r.registries.insert(
        "buildings".into(),
        TYPES.iter().map(|(n, _)| n.to_string()).collect(),
    );
    for (n, a) in TYPES {
        r.sim.buildings.insert(n.into(), building(a));
    }
    r
}

fn c(dx: i32, dy: i32) -> CellXY {
    testkit::mid(SIZE, dx, dy)
}

/// The sandbox game with one HUT of house 0 at the middle cell.
fn game() -> World {
    let mut w = testkit::game_with(&rules(), SIZE, 11);
    testkit::run(&mut w, 4);
    testkit::spawn(&mut w, Kind::Building, "HUT", 0, c(0, 0), 0);
    w
}

fn ty(w: &World, name: &str) -> u32 {
    w.rules().index_of(Kind::Building, name).unwrap()
}

/// Whether type `name` of house 0 is adjacent at `cell`, asked on its own
/// thread; `None` when the answer has not come within ten seconds or the
/// thread panicked.
fn adjacent(name: &'static str, cell: CellXY) -> Option<bool> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let w = game();
        let _ = tx.send(placement::adjacent(&w, A, ty(&w, name), cell));
    });
    rx.recv_timeout(Duration::from_secs(10)).ok()
}

// spec: S-0406 claim 4 and F1 (d = A + 1: the rectangle reaches d cells past the foundation on every side, so a supporter two cells away is inside it for every A ≥ 1)
#[test]
fn a_large_adjacency_finds_a_near_supporter() {
    for name in ["HUT", "WIDE", "VAST", "HUGE"] {
        assert_eq!(adjacent(name, c(2, 0)), Some(true), "{name}");
    }
}

// spec: S-0406 F1 (every cell of the map lies in the rectangle of the largest A, so a supporter across the map is found)
#[test]
fn the_largest_adjacency_reaches_across_the_map() {
    let far = c(7, 0);
    assert_eq!(adjacent("HUT", far), Some(false), "out of the default reach");
    for name in ["WIDE", "VAST", "HUGE"] {
        assert_eq!(adjacent(name, far), Some(true), "{name}");
    }
}

// spec: S-0406 Edge cases (negative adjacency makes placement impossible for that type) and F1
#[test]
fn a_negative_adjacency_is_never_adjacent() {
    for name in ["NONE", "LEAST"] {
        assert_eq!(adjacent(name, c(1, 0)), Some(false), "{name}");
    }
}

/// The cells, for type `name`, where the placement search and the
/// condition itself disagree, found on its own thread; `None` when the
/// answer has not come within twenty seconds or the thread panicked.
fn disagreements(name: &'static str) -> Option<Vec<CellXY>> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let w = game();
        let cells: Vec<CellXY> = (-6..=6)
            .flat_map(|dy| (-6..=6).map(move |dx| c(dx * 2, dy * 2)))
            .collect();
        let t = TypeRef {
            kind: Kind::Building,
            index: ty(&w, name),
        };
        let search = placement::PlaceSearch::new(&w, A, t, &cells);
        let differ = cells
            .iter()
            .copied()
            .filter(|&cell| {
                search.place_legal(&w, cell) != placement::place_legal(&w, A, t, cell)
            })
            .collect();
        let _ = tx.send(differ);
    });
    rx.recv_timeout(Duration::from_secs(20)).ok()
}

// spec: S-0406 claim 4 and F1 (the placement search answers as the condition itself, cell by cell, for any A)
#[test]
fn the_placement_search_answers_as_the_condition() {
    for (name, _) in TYPES {
        assert_eq!(disagreements(name), Some(Vec::new()), "{name}");
    }
}
