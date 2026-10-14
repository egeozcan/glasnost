//! The entered-by arrival raise at the movement specs' arrival moments
//! (S-0207 claim 22, S-0208 claim 22, S-0011 claim 18) on a synthetic map:
//! a cell tag whose entered-by trigger sets a variable.

use std::sync::Arc;

use glasnost_map::CellXY;
use glasnost_map::triggers::{Action, Event, TagDef, TriggerDef, Triggers};
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::Locomotor;
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind, SimRules};
use glasnost_sim::testkit;
use glasnost_sim::tick;
use glasnost_sim::world::{HouseId, Layer, ObjectId, Target, World};

/// The tagged cell.
const TAGGED: (i16, i16) = (20, 20);

/// A repeating tag over one trigger: entered-by any house (event 1, value
/// −1) sets variable 0 (action 56), carried by the tagged cell.
fn tagged() -> Triggers {
    let trigger = TriggerDef {
        events: vec![Event {
            code: 1,
            kind: 0,
            value: -1,
            type_name: None,
        }],
        actions: vec![Action {
            code: 56,
            number: 0,
            ..Default::default()
        }],
        ..TriggerDef::created(b"T".to_vec())
    };
    let tag = TagDef {
        id: b"TAG".to_vec(),
        repeat: 2,
        name: Vec::new(),
        first: Some(0),
        chain: vec![0],
    };
    Triggers {
        triggers: vec![trigger],
        tags: vec![tag],
        cell_tags: vec![(TAGGED, 0)],
        ..Default::default()
    }
}

fn game(r: &Ruleset) -> World {
    let mut m = testkit::empty_map(20);
    m.triggers = tagged();
    let rules = Arc::new(SimRules::new(r, "synthetic-test"));
    let mut w = glasnost_sim::load::new_game(rules, &m, testkit::catalogue(), &testkit::setup(7))
        .expect("synthetic game loads");
    // Orders are accepted from tick 4.
    while w.tick < 4 {
        tick::step(&mut w);
    }
    w
}

fn order_move(w: &mut World, actor: ObjectId, c: CellXY) {
    let order = Order::Behaviour {
        actor,
        behaviour: Behaviour::Move,
        attack_move: false,
        target: None,
        dest: Some(Target::Cell(c)),
    };
    assert!(orders::issue(w, HouseId(0), order), "move order refused");
}

fn set(w: &World) -> bool {
    w.triggers.variables[0]
}

fn tagged_cell() -> CellXY {
    CellXY::new(i32::from(TAGGED.0), i32::from(TAGGED.1))
}

fn bridge(w: &mut World, c: CellXY) {
    let i = w.cells.idx(c).expect("cell");
    w.cells.deck[i] = true;
    w.cells.deck_entry[i] = true;
    w.cells.bridge_axis[i] = glasnost_sim::cells::Axis::Y;
}

/// Steps until the variable is set or `limit` ticks have run; for each
/// tick the unit's cell after it. Returns the tick index of the raise.
fn drive(w: &mut World, id: ObjectId, limit: usize) -> (Option<usize>, Vec<CellXY>) {
    let mut cells = Vec::new();
    for i in 0..limit {
        tick::step(w);
        cells.push(w.obj(id).map_or(CellXY::new(0, 0), |o| o.pos.cell()));
        if set(w) {
            return (Some(i), cells);
        }
    }
    (None, cells)
}

// spec: S-0207 claim 22 (a driving vehicle raises the tag at the cell-arrival update when it reaches the track target in the tagged cell, without stopping there; crossing into the cell mid-track does not raise it)
#[test]
fn driving_arrival_raises_at_the_track_target() {
    let mut w = game(&testkit::ruleset());
    let cart = testkit::spawn(&mut w, Kind::Vehicle, "CART", 0, CellXY::new(17, 20), 64);
    order_move(&mut w, cart, CellXY::new(24, 20));
    let (raised, cells) = drive(&mut w, cart, 400);
    let i = raised.expect("the arrival raises the tag");
    let first = cells
        .iter()
        .position(|&c| c == tagged_cell())
        .expect("it enters the tagged cell");
    assert!(i > first, "not at the boundary crossing");
    // It drives on: the destination lies beyond.
    let mut reached = false;
    for _ in 0..400 {
        tick::step(&mut w);
        reached |= w.obj(cart).unwrap().pos.cell() == CellXY::new(24, 20);
    }
    assert!(reached);
}

// spec: S-0207 claim 22 and S-0011 claim 18 (ground passage under a high bridge is no arrival: only units on the deck raise the tag)
#[test]
fn ground_passage_under_a_high_bridge_does_not_raise() {
    let mut w = game(&testkit::ruleset());
    bridge(&mut w, tagged_cell());
    let cart = testkit::spawn(&mut w, Kind::Vehicle, "CART", 0, CellXY::new(17, 20), 64);
    order_move(&mut w, cart, CellXY::new(24, 20));
    let (raised, cells) = drive(&mut w, cart, 400);
    assert!(cells.contains(&tagged_cell()));
    assert_eq!(w.obj(cart).unwrap().layer, Layer::Ground);
    assert_eq!(raised, None);
}

fn teleporting() -> Ruleset {
    let mut r = testkit::ruleset();
    r.sim.vehicles.get_mut("CART").unwrap().locomotor = Some(Locomotor::Teleport);
    r
}

// spec: S-0207 claim 22 (a teleport raises the landing cell's tag in the jump tick, after position and listing have changed and before destination removal; not at reservation)
#[test]
fn teleport_landing_raises_in_the_jump_tick() {
    let mut w = game(&teleporting());
    let cart = testkit::spawn(&mut w, Kind::Vehicle, "CART", 0, CellXY::new(17, 20), 64);
    order_move(&mut w, cart, tagged_cell());
    let (raised, cells) = drive(&mut w, cart, 3);
    let i = raised.expect("the landing raises the tag");
    assert_eq!(cells[i], tagged_cell());
    assert!(
        i == 0 || cells[i - 1] != tagged_cell(),
        "raised in the jump tick"
    );
    assert!(w.cells.list(tagged_cell(), Layer::Ground).contains(&cart));
}

// spec: S-0207 claim 22 (the cells between departure and landing receive no arrival update)
#[test]
fn teleport_passes_no_intermediate_cell() {
    let mut w = game(&teleporting());
    let cart = testkit::spawn(&mut w, Kind::Vehicle, "CART", 0, CellXY::new(17, 20), 64);
    order_move(&mut w, cart, CellXY::new(24, 20));
    let (raised, cells) = drive(&mut w, cart, 20);
    assert!(cells.contains(&CellXY::new(24, 20)));
    assert_eq!(raised, None);
}

// spec: S-0207 claim 22 and S-0011 claim 18 (a teleport onto a high-bridge cell lands on the deck and raises the tag)
#[test]
fn teleport_onto_a_bridge_deck_raises() {
    let mut w = game(&teleporting());
    bridge(&mut w, tagged_cell());
    let cart = testkit::spawn(&mut w, Kind::Vehicle, "CART", 0, CellXY::new(17, 20), 64);
    order_move(&mut w, cart, tagged_cell());
    let (raised, _) = drive(&mut w, cart, 3);
    assert!(raised.is_some());
    assert_eq!(w.obj(cart).unwrap().layer, Layer::Deck);
}

// spec: S-0208 claim 22 (a walker raises the tag on the tick it snaps to a walking target in the tagged cell, even when it walks on; not on entering the cell)
#[test]
fn walking_arrival_raises_at_the_snap() {
    let mut w = game(&testkit::ruleset());
    let walker = testkit::spawn(&mut w, Kind::Infantry, "WALKER", 0, CellXY::new(17, 20), 64);
    order_move(&mut w, walker, CellXY::new(24, 20));
    let mut entered = None;
    let mut raised = None;
    for i in 0..2000 {
        tick::step(&mut w);
        let o = w.obj(walker).unwrap();
        if entered.is_none() && o.pos.cell() == tagged_cell() {
            entered = Some(i);
        }
        if set(&w) {
            raised = Some(i);
            // Snapped: the position is a standing point and the walking target is cleared.
            assert!(o.walk.as_ref().is_some_and(|s| s.target.is_none()));
            assert_eq!(o.pos.cell(), tagged_cell());
            break;
        }
    }
    let (e, r) = (entered.expect("enters"), raised.expect("raises"));
    assert!(r > e, "entering the cell does not raise it");
}
