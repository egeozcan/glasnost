//! A vehicle absorbed from a bridge deck releases its vehicle-present
//! claim in the deck record (S-0605 claim 8, S-0209 claim 1; #346).
//! Synthetic data only: two banks at height level 4, a river bed at level
//! 0 with a high bridge span over it, a 1 × 1 vehicle absorber pre-placed
//! on the river bed under the deck cell (3, 0), a car on the bank ordered
//! to enter it.

mod common;

use common::*;
use glasnost_rules::sim::{BuildingType, Decimal};
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::structures::absorb;
use glasnost_sim::tick;
use glasnost_sim::world::{HouseId, Layer, Target, World};

/// First tile number of the bridge role in the scene's catalogue.
const BRIDGE_ROLE: i32 = 42;

/// Two banks at height level 4 and a river bed at level 0 under the columns
/// x = 1 … 5 (rock, but for the open cell (3, 0)); a high bridge span from
/// (0, 0) to (6, 0) with a high-bridge overlay in each river cell of row 0
/// (the layout of claims_s0212's river scene, with raised banks).
fn river() -> Scene {
    let mut s = Scene::new();
    for i in 0..=24 {
        let name = format!("OV{i}");
        s.register("overlays", &name);
        s.ruleset.sim.overlays.insert(
            name,
            glasnost_rules::sim::OverlayType {
                has_art: Some(true),
                ..Default::default()
            },
        );
    }
    assert_eq!(s.catalogue.tiles.len() as i32, BRIDGE_ROLE);
    for _ in 0..16 {
        s.catalogue.tiles.push(glasnost_map::terrain::TileInfo {
            width: 3,
            height: 2,
            has_image: true,
            cells: vec![
                Some(glasnost_map::terrain::TileCell {
                    terrain_code: 0,
                    slope: 0,
                });
                6
            ],
            ..Default::default()
        });
    }
    s.catalogue.roles.bridge = BRIDGE_ROLE;
    let all: Vec<_> = s.map.cells.iter().map(|(c, _)| c).collect();
    for c in all {
        if (cell(1, 0).x..=cell(5, 0).x).contains(&c.x) {
            if c == cell(3, 0) {
                s.set_cell(c, 0, 0);
            } else {
                s.set_cell(c, code_tile(7), 0);
            }
        } else {
            s.set_cell(c, 0, 4);
        }
    }
    for x in [0, 6] {
        s.set_cell(cell(x, 0), BRIDGE_ROLE + 6, 4);
        s.cell_mut(cell(x, 0)).sub_tile = 4;
    }
    for x in 1..=5 {
        s.cell_mut(cell(x, 0)).overlay = 24;
    }
    s
}

/// The deck record's vehicle count of the river cells (1, 0) … (5, 0).
fn deck_counts(w: &World) -> Vec<(i32, u8)> {
    (1..=5)
        .map(|x| (x, w.cells.occ(cell(x, 0), Layer::Deck).unwrap().vehicles))
        .collect()
}

// spec: S-0605 claim 8 (on accept the vehicle leaves any bridge deck and leaves the map) and S-0209 claim 1 (a vehicle taken out of a cell's list clears the flag it set, in the cell layer of its position at that moment)
#[test]
fn issue_346_vehicle_absorbed_from_a_deck_releases_its_claim() {
    let mut s = river();
    s.vehicle("CAR", unit_type(WHEEL, 40, 6));
    let mut b = BuildingType {
        strength: Some(1000),
        foundation: Some([1, 1]),
        unit_absorb: Some(true),
        passengers: Some(2),
        size_limit: Some(Decimal("3".into())),
        ..Default::default()
    };
    b.econ.power = Some(10);
    s.building("VBIO", b);
    // The absorber stands on the river bed under the deck cell (3, 0).
    s.place_structure(0, "VBIO", cell(3, 0));
    s.place_unit(0, "CAR", cell(-2, 0), 64);
    let mut w = s.build();
    let car = objects_of(&w, Kind::Vehicle)[0];
    let bio = objects_of(&w, Kind::Building)[0];
    open_orders(&mut w);
    run(&mut w, 3);
    assert!(orders::issue(
        &mut w,
        HouseId(0),
        Order::Behaviour {
            actor: car,
            behaviour: Behaviour::Enter,
            attack_move: false,
            target: None,
            dest: Some(Target::Object(bio)),
        },
    ));
    let mut last = None;
    let mut entered_from = None;
    for _ in 0..2000 {
        tick::step(&mut w);
        let o = w.obj(car).expect("car");
        if absorb::absorbed_in(&w, car) == Some(bio) {
            entered_from = last;
            break;
        }
        last = Some(o.layer);
    }
    let from_layer = entered_from.expect("the car went inside");
    assert_eq!(
        from_layer,
        Layer::Deck,
        "the scenario: it arrived on the deck"
    );
    // The absorbed car is off the map: no deck record keeps its claim.
    assert_eq!(
        deck_counts(&w),
        (1..=5).map(|x| (x, 0)).collect::<Vec<_>>(),
        "(x, the deck record's vehicle count)"
    );
}

/// The ground record's vehicle count of the river cells (1, 0) … (5, 0).
fn ground_counts(w: &World) -> Vec<(i32, u8)> {
    (1..=5)
        .map(|x| (x, w.cells.occ(cell(x, 0), Layer::Ground).unwrap().vehicles))
        .collect()
}

// spec: S-0209 claim 1 (a step sets the flag of the cell it enters in the record of the cell layer its position is in there) and S-0207 claim 5 (into a high-bridge cell exactly 4 levels below the cell left: onto the deck)
#[test]
fn issue_346_a_vehicle_on_the_deck_sets_no_ground_flag() {
    let mut s = river();
    s.vehicle("CAR", unit_type(WHEEL, 40, 6));
    s.place_unit(0, "CAR", cell(-2, 0), 64);
    let mut w = s.build();
    let car = objects_of(&w, Kind::Vehicle)[0];
    open_orders(&mut w);
    run(&mut w, 3);
    assert!(orders::issue(&mut w, HouseId(0), move_to(car, cell(9, 0))));
    let none: Vec<(i32, u8)> = (1..=5).map(|x| (x, 0)).collect();
    let mut on_deck = 0;
    for _ in 0..1500 {
        tick::step(&mut w);
        let o = w.obj(car).expect("car");
        if o.layer == Layer::Deck {
            on_deck += 1;
            // Nothing stands on the river bed: no ground record holds a flag.
            assert_eq!(
                ground_counts(&w),
                none,
                "tick {}: (x, the ground record's vehicle count) with the car on the deck at {:?}",
                w.tick,
                (o.pos.cell().x - SIZE, o.pos.cell().y - SIZE)
            );
        }
        if o.pos.cell() == cell(9, 0) {
            break;
        }
    }
    assert!(on_deck > 0, "the scenario: the car crossed on the deck");
    assert_eq!(ground_counts(&w), none, "after the crossing");
    assert_eq!(deck_counts(&w), none, "after the crossing");
}

// spec: S-0209 claim 1 (a step sets the flag of the cell it enters in the record of the cell layer its position is in there) and S-0207 claim 5; S-0605 claim 8 (the scene of the first test, with the issue's second car)
#[test]
fn issue_346_a_vehicle_entering_from_the_deck_sets_no_ground_flag() {
    let mut s = river();
    s.vehicle("CAR", unit_type(WHEEL, 40, 6));
    let mut b = BuildingType {
        strength: Some(1000),
        foundation: Some([1, 1]),
        unit_absorb: Some(true),
        passengers: Some(2),
        size_limit: Some(Decimal("3".into())),
        ..Default::default()
    };
    b.econ.power = Some(10);
    s.building("VBIO", b);
    s.place_structure(0, "VBIO", cell(3, 0));
    s.place_unit(0, "CAR", cell(-2, 0), 64);
    s.place_unit(0, "CAR", cell(-3, 3), 64);
    let mut w = s.build();
    let car = objects_of(&w, Kind::Vehicle)[0];
    let bio = objects_of(&w, Kind::Building)[0];
    open_orders(&mut w);
    run(&mut w, 3);
    assert!(orders::issue(
        &mut w,
        HouseId(0),
        Order::Behaviour {
            actor: car,
            behaviour: Behaviour::Enter,
            attack_move: false,
            target: None,
            dest: Some(Target::Object(bio)),
        },
    ));
    let none: Vec<(i32, u8)> = (1..=5).map(|x| (x, 0)).collect();
    let mut on_deck = 0;
    for _ in 0..2000 {
        tick::step(&mut w);
        if absorb::absorbed_in(&w, car) == Some(bio) {
            break;
        }
        let o = w.obj(car).expect("car");
        if o.layer == Layer::Deck {
            on_deck += 1;
        }
        // Nothing stands on the river bed: no ground record holds a flag.
        assert_eq!(
            ground_counts(&w),
            none,
            "tick {}: (x, the ground record's vehicle count) with the car at {:?} on {:?}",
            w.tick,
            (o.pos.cell().x - SIZE, o.pos.cell().y - SIZE),
            o.layer
        );
    }
    assert!(on_deck > 0, "the scenario: the car drove on the deck");
    assert_eq!(
        absorb::absorbed_in(&w, car),
        Some(bio),
        "the scenario: it went in"
    );
    assert_eq!(ground_counts(&w), none, "after the entry");
}
