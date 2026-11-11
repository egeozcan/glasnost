//! A save that decodes but does not hold together is an error, not a
//! panic (#157). Such bytes reach `save::load` from a rejoin transfer
//! another peer sends. Synthetic data only: the testkit's empty map.

use glasnost_map::{Cell, Footprint};
use glasnost_sim::orders::Order;
use glasnost_sim::rules::Behaviour;
use glasnost_sim::save::{self, SaveError};
use glasnost_sim::world::{HouseId, ObjectId, Target, World};
use glasnost_sim::{load, testkit};

fn world() -> World {
    let map = testkit::empty_map(30);
    load::new_game(
        testkit::rules(),
        &map,
        testkit::catalogue(),
        &testkit::setup(1),
    )
    .expect("synthetic test game loads")
}

/// The save of `w` after `edit` changed its decoded world. `Save` is
/// private; postcard writes a struct as its fields in order, so a tuple of
/// the same fields reads and writes the same bytes.
fn edited(w: &World, edit: impl FnOnce(&mut World)) -> Vec<u8> {
    let good = save::save(w);
    let (version, hash, mut world): (u32, String, World) =
        postcard::from_bytes(&good).expect("the save decodes");
    assert_eq!(
        postcard::to_allocvec(&(version, &hash, &world)).unwrap(),
        good,
        "the tuple stands for the save byte for byte"
    );
    edit(&mut world);
    postcard::to_allocvec(&(version, hash, world)).expect("serializes")
}

fn loads(w: &World, bytes: &[u8]) -> Result<World, SaveError> {
    save::load(bytes, w.rules_arc(), testkit::catalogue())
}

fn refused(w: &World, bytes: &[u8]) -> bool {
    matches!(loads(w, bytes), Err(SaveError::Format(_)))
}

#[test]
fn an_untouched_save_still_loads() {
    let w = world();
    assert!(loads(&w, &edited(&w, |_| {})).is_ok());
}

// nocite: our own input validation (#157): a per-cell table shorter than the grid
#[test]
fn a_save_with_a_short_category_table_is_refused_not_a_panic() {
    let w = world();
    let bad = edited(&w, |x| x.cells.category.clear());
    assert!(refused(&w, &bad));
}

// nocite: our own input validation (#157)
#[test]
fn a_save_with_a_short_side_table_is_refused() {
    let w = world();
    let bad = edited(&w, |x| {
        x.cells.wall_owner.pop();
    });
    assert!(refused(&w, &bad));
    let bad = edited(&w, |x| x.cells.lists.truncate(3));
    assert!(refused(&w, &bad));
}

// nocite: our own input validation (#157): a grid whose side and cells do not agree
#[test]
fn a_save_with_a_grid_of_side_zero_is_refused() {
    let w = world();
    let bad = edited(&w, |x| {
        let bytes = postcard::to_allocvec(&x.cells.grid).unwrap();
        let (fp, _side, cells): (Footprint, u16, Vec<Cell>) =
            postcard::from_bytes(&bytes).expect("the grid decodes as its fields");
        x.cells.grid = postcard::from_bytes(&postcard::to_allocvec(&(fp, 0u16, cells)).unwrap())
            .expect("the edited grid decodes");
    });
    assert!(refused(&w, &bad));
}

// nocite: our own input validation (#157): a random stream with a short state or a position past it
#[test]
fn a_save_with_a_broken_random_stream_is_refused() {
    let w = world();
    let rng = |x: &mut World, edit: &dyn Fn(&mut Vec<u32>, &mut u64)| {
        let bytes = postcard::to_allocvec(&x.rng).unwrap();
        let (mut state, mut next, taken): (Vec<u32>, u64, u64) =
            postcard::from_bytes(&bytes).expect("the stream decodes as its fields");
        edit(&mut state, &mut next);
        x.rng = postcard::from_bytes(&postcard::to_allocvec(&(state, next, taken)).unwrap())
            .expect("the edited stream decodes");
    };
    let bad = edited(&w, |x| rng(x, &|s, _| s.truncate(10)));
    assert!(refused(&w, &bad));
    let bad = edited(&w, |x| rng(x, &|_, n| *n = 250));
    assert!(refused(&w, &bad));
}

// nocite: our own input validation (#157, #219): a pending order whose cell is far off every map
#[test]
fn a_save_with_a_far_cell_in_a_pending_order_is_refused() {
    let w = world();
    let bad = edited(&w, |x| {
        x.houses[0].pending.push(Order::Behaviour {
            actor: ObjectId(1),
            behaviour: Behaviour::Move,
            attack_move: false,
            target: None,
            dest: Some(Target::Cell(glasnost_map::CellXY::new(10_000_000, 0))),
        });
    });
    assert!(refused(&w, &bad));
}

// nocite: our own input validation (#157): an object owned by a house the game does not have
#[test]
fn a_save_with_an_object_of_a_missing_house_is_refused() {
    let mut w = world();
    let id = testkit::spawn(
        &mut w,
        glasnost_sim::rules::Kind::Vehicle,
        "BUGGY",
        0,
        testkit::mid(30, 0, 0),
        0,
    );
    let bad = edited(&w, |x| {
        x.objects.get_mut(&id).unwrap().owner = Some(HouseId(200));
    });
    assert!(refused(&w, &bad));
    let bad = edited(&w, |x| {
        x.objects.get_mut(&id).unwrap().type_index = 10_000;
    });
    assert!(refused(&w, &bad));
}
