//! Claim tests for the landing point of infantry with the teleporting
//! locomotor (S-0208 claims 19 and 20) on synthetic data: invented type
//! names and values on the flat scene of `common`.

mod common;

use common::*;
use glasnost_map::CellXY;
use glasnost_rules::sim::{Locomotor, UnitType};
use glasnost_sim::cells::position;
use glasnost_sim::rules::Kind;
use glasnost_sim::walk;
use glasnost_sim::world::{HouseId, Layer, ObjectId, Pos, Target, World};

/// An infantry type with the teleporting locomotor.
fn blinker() -> UnitType {
    let mut t = unit_type(FOOT, 4, 0);
    t.locomotor = Some(Locomotor::Teleport);
    t
}

/// BLINKER at the middle cell, and the walking type STANDER.
fn scene() -> Scene {
    let mut s = Scene::new();
    s.infantry("BLINKER", blinker());
    s.infantry("STANDER", unit_type(FOOT, 4, 0));
    s.place_infantry(0, "BLINKER", cell(0, 0), 0);
    s
}

fn start(s: &Scene) -> (World, ObjectId) {
    let mut w = s.build();
    open_orders(&mut w);
    let id = objects_of(&w, Kind::Infantry)[0];
    (w, id)
}

fn taken(w: &World, c: CellXY, p: u8) -> bool {
    w.cells.occ(c, Layer::Ground).is_some_and(|r| r.taken(p))
}

fn standing(w: &World, id: ObjectId) -> (CellXY, (i32, i32)) {
    let p = w.obj(id).unwrap().pos;
    (p.cell(), (p.x.rem_euclid(256), p.y.rem_euclid(256)))
}

// spec: S-0208 claim 19 (a cell destination: the centre class takes a side position of table O and reserves it; the jump follows S-0207 claim 20)
#[test]
fn claim_19_cell_destination_lands_on_a_side_position() {
    let (mut w, id) = start(&scene());
    let to = cell(0, -6);
    order_move(&mut w, id, to);
    run_until(&mut w, 10, |w| w.obj(id).unwrap().pos.cell() == to);
    let (c, uv) = standing(&w, id);
    assert_eq!(c, to);
    let sides = [position::RIGHT, position::LEFT, position::BOTTOM];
    let p = sides
        .into_iter()
        .find(|&p| walk::standing_point(p) == uv)
        .expect("a side position of table O");
    assert!(taken(&w, to, p), "the standing position is reserved");
    assert!(
        !taken(&w, cell(0, 0), position::CENTRE),
        "the departure entry is cleared"
    );
    assert_eq!(w.obj(id).unwrap().dest, None);
}

// spec: S-0208 claim 19 (a refused position choice for a cell destination: no jump, the destination is removed, no neighbouring cell)
#[test]
fn claim_19_full_cell_refuses_without_a_fallback() {
    let mut s = scene();
    let to = cell(0, -6);
    for sub in 2..=4 {
        s.place_infantry(0, "STANDER", to, sub);
    }
    let (mut w, id) = start(&s);
    let from = w.obj(id).unwrap().pos;
    order_move(&mut w, id, to);
    run(&mut w, 6);
    let o = w.obj(id).unwrap();
    assert_eq!(o.pos, from, "no jump");
    assert_eq!(o.dest, None, "the destination is removed");
    assert!(taken(&w, cell(0, 0), position::CENTRE));
}

// spec: S-0208 claim 19 (the first row of claim 3: a vehicle-present cell refuses a cell destination)
#[test]
fn claim_19_vehicle_cell_refuses() {
    let mut s = scene();
    let to = cell(0, -6);
    s.vehicle("CART", unit_type(WHEEL, 6, 5));
    s.place_unit(0, "CART", to, 0);
    let (mut w, id) = start(&s);
    let from = w.obj(id).unwrap().pos;
    order_move(&mut w, id, to);
    run(&mut w, 6);
    assert_eq!(w.obj(id).unwrap().pos, from);
    assert_eq!(w.obj(id).unwrap().dest, None);
}

/// A scene with impassable rock at level `level` on the destination cell.
fn rock_at(level: i8) -> (Scene, CellXY) {
    let mut s = scene();
    let to = cell(0, -6);
    // spec: S-0211 claim 5 (terrain code 15: rock)
    s.set_cell(to, code_tile(15), level);
    (s, to)
}

// spec: S-0208 claim 19 and Edge cases (a cell at level −6 or lower: the entry outcome is clear without a terrain test; at −5 the rock refuses)
#[test]
fn claim_19_negative_level_entry() {
    let (s, to) = rock_at(-6);
    let (mut w, id) = start(&s);
    order_move(&mut w, id, to);
    run(&mut w, 6);
    assert_eq!(w.obj(id).unwrap().pos.cell(), to, "−6 is accepted");

    let (s, to) = rock_at(-5);
    let (mut w, id) = start(&s);
    order_move(&mut w, id, to);
    run(&mut w, 6);
    assert_eq!(w.obj(id).unwrap().pos.cell(), cell(0, 0), "−5 is refused");
}

// spec: S-0208 Edge cases (quirks.teleport_infantry_negative_level_entry off: the ordinary entry check applies)
#[test]
fn claim_19_negative_level_entry_switch_off() {
    let (mut s, to) = rock_at(-6);
    s.ruleset
        .globals
        .quirks
        .teleport_infantry_negative_level_entry = false;
    let (mut w, id) = start(&s);
    order_move(&mut w, id, to);
    run(&mut w, 6);
    assert_eq!(w.obj(id).unwrap().pos.cell(), cell(0, 0));
}

// spec: S-0208 claim 19 (one ranged draw over 0 to 3 for the centre-class choice; the jump itself takes none, S-0207 claim 20 i)
#[test]
fn claim_19_one_draw_for_a_centre_destination() {
    let (mut w, id) = start(&scene());
    let to = cell(0, -6);
    let mut probe = w.rng.clone();
    w.obj_mut(id).unwrap().dest = Some(Target::Cell(to));
    glasnost_sim::teleport::take_destination(&mut w, id);
    let half = w.rules().quirks.ranged_draw_half_span;
    let k = probe.ranged(0, 3, half);
    assert_eq!(w.rng, probe, "the landing choice took exactly one draw");
    let expect = match k {
        0 | 1 => position::RIGHT,
        2 => position::LEFT,
        _ => position::BOTTOM,
    };
    assert!(taken(&w, to, expect), "row R{k} of table O");
}

// spec: S-0208 claim 20 (the placement fall-back supplies a destination: the destination cell's ground occupants get a forced, move-anywhere scatter request before the landing choice)
#[test]
fn claim_20_placement_clears_the_destination_cell() {
    let mut s = Scene::new();
    s.infantry("BLINKER", blinker());
    s.infantry("STANDER", unit_type(FOOT, 4, 0));
    let to = cell(0, -6);
    s.place_infantry(0, "STANDER", to, 2);
    let mut w = s.build();
    open_orders(&mut w);
    let stander = objects_of(&w, Kind::Infantry)[0];
    let ty = w.rules().index_of(Kind::Infantry, "BLINKER").unwrap();
    let id = glasnost_sim::objects::create(&mut w, Kind::Infantry, ty, Some(HouseId(0)));
    w.obj_mut(id).unwrap().guard_spot = Some(Target::Cell(to));
    let at = Pos::centre(cell(0, 0), 0);
    glasnost_sim::objects::place(&mut w, id, at, Default::default());
    assert!(
        w.obj(stander).unwrap().dest.is_some(),
        "the occupant accepted the scatter request"
    );
    assert_eq!(w.obj(id).unwrap().dest, Some(Target::Cell(to)));
}

// spec: S-0208 claim 20 (an ordinary later move to the same occupied cell sends no such request)
#[test]
fn claim_20_ordinary_move_does_not_clear() {
    let mut s = scene();
    let to = cell(0, -6);
    s.place_infantry(0, "STANDER", to, 2);
    let (mut w, id) = start(&s);
    let stander = objects_of(&w, Kind::Infantry)[1];
    order_move(&mut w, id, to);
    run(&mut w, 3);
    assert_eq!(w.obj(id).unwrap().pos.cell(), to, "it jumped");
    assert_eq!(w.obj(stander).unwrap().dest, None);
}

/// BLINKER at the middle cell and a CART of its house at (0, −6).
fn with_cart() -> (World, ObjectId, ObjectId, CellXY) {
    let mut s = scene();
    let to = cell(0, -6);
    s.vehicle("CART", unit_type(WHEEL, 6, 5));
    s.place_unit(0, "CART", to, 0);
    let (w, id) = start(&s);
    let cart = objects_of(&w, Kind::Vehicle)[0];
    (w, id, cart, to)
}

fn landing(w: &World, id: ObjectId) -> Option<Pos> {
    w.obj(id)
        .and_then(|o| o.teleport.as_ref())
        .and_then(|t| t.landing)
}

// spec: S-0208 claim 19 (enter with a vehicle in C as destination: occupants ignored, the standing position of P's class without a draw; the entry outcome still decides)
#[test]
fn claim_19_enter_destination_ignores_occupants() {
    use glasnost_sim::rules::Behaviour;
    let (mut w, id, cart, to) = with_cart();
    let t = w.tick;
    let o = w.obj_mut(id).unwrap();
    o.beh.force(Behaviour::Enter, t, true);
    o.dest = Some(Target::Object(cart));
    let before = w.rng.clone();
    glasnost_sim::teleport::take_destination(&mut w, id);
    assert_eq!(w.rng, before, "no draw");
    let l = landing(&w, id).expect("a landing point");
    assert_eq!(l.cell(), to);
    assert_eq!(
        (l.x.rem_euclid(256), l.y.rem_euclid(256)),
        walk::standing_point(position::CENTRE)
    );
}

// spec: S-0208 claim 19 (a refused object destination: the nearby-cell search around the object's heading-point cell, then a centre-desired position choice; the destination is kept)
#[test]
fn claim_19_object_destination_searches_nearby() {
    let (mut w, id, cart, to) = with_cart();
    w.obj_mut(id).unwrap().dest = Some(Target::Object(cart));
    glasnost_sim::teleport::take_destination(&mut w, id);
    let l = landing(&w, id).expect("a landing point");
    let c = l.cell();
    assert_ne!(c, to);
    assert!(
        (c.x - to.x).abs() <= 1 && (c.y - to.y).abs() <= 1,
        "a neighbour"
    );
    let uv = (l.x.rem_euclid(256), l.y.rem_euclid(256));
    assert_ne!(
        uv,
        walk::standing_point(position::CENTRE),
        "table O gives a side position"
    );
    assert_eq!(w.obj(id).unwrap().dest, Some(Target::Object(cart)));
}

// spec: S-0208 claim 19 (a replacement destination clears the earlier reservation first; refused, it restores the entry at the present position and the unit does not jump, Q-0235)
#[test]
fn claim_19_refused_replacement_restores_the_present_entry() {
    let mut s = scene();
    let a = cell(0, -6);
    let b = cell(6, 0);
    for sub in 2..=4 {
        s.place_infantry(0, "STANDER", b, sub);
    }
    let (mut w, id) = start(&s);
    let from = w.obj(id).unwrap().pos;
    order_move(&mut w, id, a);
    run(&mut w, 1);
    let reserved = |w: &World| (2..=4).any(|p| taken(w, a, p));
    assert!(reserved(&w), "the landing position at A is reserved");
    assert!(!taken(&w, cell(0, 0), position::CENTRE));
    // The destination is replaced before the jump's movement update.
    w.obj_mut(id).unwrap().dest = Some(Target::Cell(b));
    run(&mut w, 1);
    let o = w.obj(id).unwrap();
    assert_eq!(o.pos, from, "no jump");
    assert_eq!(o.dest, None);
    assert!(!reserved(&w), "the earlier reservation is cleared");
    assert!(
        taken(&w, cell(0, 0), position::CENTRE),
        "the present entry is restored"
    );
}

/// A world with BLINKER and a high-bridge cell `to`, the quirk set to `on`.
fn bridge_world(on: bool) -> (World, ObjectId, CellXY) {
    let mut s = scene();
    s.ruleset
        .globals
        .quirks
        .teleport_infantry_ground_reservation = on;
    let (mut w, id) = start(&s);
    let to = cell(0, -6);
    let i = w.cells.idx(to).expect("cell");
    w.cells.deck[i] = true;
    w.cells.deck_entry[i] = true;
    w.cells.bridge_axis[i] = glasnost_sim::cells::Axis::Y;
    (w, id, to)
}

fn deck_taken(w: &World, c: CellXY, p: u8) -> bool {
    w.cells.occ(c, Layer::Deck).is_some_and(|r| r.taken(p))
}

/// The taken side positions of `c` in the ground and deck records.
fn sides(w: &World, c: CellXY) -> (Vec<u8>, Vec<u8>) {
    (
        (2..=4).filter(|&p| taken(w, c, p)).collect(),
        (2..=4).filter(|&p| deck_taken(w, c, p)).collect(),
    )
}

/// The standing position class of the unit's own position.
fn own_class(w: &World, id: ObjectId) -> u8 {
    let (_, (u, v)) = standing(w, id);
    walk::position_class(u, v, w.rules().quirks.top_position_counts_as_centre)
}

/// Runs until the unit stands in `c` and has finished phasing in.
fn land(w: &mut World, id: ObjectId, c: CellXY) {
    run_until(w, 10, |w| w.obj(id).unwrap().pos.cell() == c);
    run_until(w, 400, |w| !glasnost_sim::teleport::phasing_in(w, id));
}

// spec: S-0208 claim 19 (a ground-height departure onto a high-bridge cell chooses and reserves with the ground record; at the jump the unit is listed on the deck with deck height, S-0207 claim 20 e; landing neither transfers the reservation nor marks the deck, Q-0311; a deck position marked by another occupant stays marked)
#[test]
fn claim_19_low_departure_onto_a_high_bridge() {
    let (mut w, id, to) = bridge_world(true);
    order_move(&mut w, id, to);
    run(&mut w, 1);
    let (ground, deck) = sides(&w, to);
    assert_eq!((ground.len(), deck.len()), (1, 0), "ground reservation");
    let p = ground[0];
    // Another occupant's mark at the same deck position.
    let other = HouseId(1);
    w.cells
        .mark_infantry(to, Layer::Deck, p, ObjectId(9999), other);
    run_until(&mut w, 10, |w| w.obj(id).unwrap().pos.cell() == to);
    let o = w.obj(id).unwrap();
    assert_eq!(o.layer, Layer::Deck);
    let c = Pos::centre(to, 0);
    assert_eq!(o.pos.z, w.surface_at(c.x, c.y) + 416);
    assert!(w.cells.list(to, Layer::Deck).contains(&id));
    assert_eq!(own_class(&w, id), p);
    assert_eq!(sides(&w, to), (vec![p], vec![p]), "both marks stay");
    let by = w.cells.occ(to, Layer::Deck).unwrap().marks[usize::from(p)]
        .unwrap()
        .by;
    assert_eq!(
        by,
        glasnost_sim::cells::Marker::Infantry(ObjectId(9999), other),
        "the deck mark is still the other occupant's"
    );
}

// spec: S-0208 claim 19 (after the jump a later destination choice clears by the unit's actual position and height: the deck position, whoever marked it, leaving the ground mark; a successful choice marks the next landing point)
#[test]
fn claim_19_later_choice_clears_by_actual_position() {
    let (mut w, id, to) = bridge_world(true);
    order_move(&mut w, id, to);
    run(&mut w, 1);
    let p = sides(&w, to).0[0];
    w.cells
        .mark_infantry(to, Layer::Deck, p, ObjectId(9999), HouseId(1));
    land(&mut w, id, to);
    let next = cell(6, 0);
    order_move(&mut w, id, next);
    run(&mut w, 1);
    assert_eq!(sides(&w, to), (vec![p], vec![]), "deck cleared, ground left");
    assert_eq!(sides(&w, next).0.len(), 1, "the next landing point is marked");
    run_until(&mut w, 10, |w| w.obj(id).unwrap().pos.cell() == next);
    assert!(w.cells.list(to, Layer::Ground).is_empty());
    assert!(w.cells.list(to, Layer::Deck).is_empty());
    assert_eq!(sides(&w, to).0, vec![p], "a ground mark with no one listed");
}

// spec: S-0208 claim 19 (after the jump a refused choice marks the unit's current deck position)
#[test]
fn claim_19_refused_choice_after_the_jump_marks_the_deck() {
    let (mut w, id, to) = bridge_world(true);
    order_move(&mut w, id, to);
    run(&mut w, 1);
    let p = sides(&w, to).0[0];
    land(&mut w, id, to);
    assert_eq!(sides(&w, to), (vec![p], vec![]));
    // A full destination cell refuses a cell destination.
    let full = cell(6, 0);
    for sub in 2..=4 {
        w.cells
            .mark_infantry(full, Layer::Ground, sub, ObjectId(9998), HouseId(1));
    }
    order_move(&mut w, id, full);
    run(&mut w, 1);
    assert_eq!(w.obj(id).unwrap().pos.cell(), to, "no jump");
    assert_eq!(sides(&w, to), (vec![p], vec![p]));
}

// spec: S-0208 Edge cases (quirks.teleport_infantry_ground_reservation off: at the jump the reservation moves to the actual landing surface at the same standing position, and later clearing uses that deck entry)
#[test]
fn claim_19_ground_reservation_switch_off() {
    let (mut w, id, to) = bridge_world(false);
    order_move(&mut w, id, to);
    run(&mut w, 1);
    let p = sides(&w, to).0[0];
    land(&mut w, id, to);
    assert_eq!(sides(&w, to), (vec![], vec![p]));
    order_move(&mut w, id, cell(6, 0));
    run(&mut w, 1);
    assert_eq!(sides(&w, to), (vec![], vec![]));
}
