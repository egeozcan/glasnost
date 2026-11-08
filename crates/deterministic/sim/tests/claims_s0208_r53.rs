//! Claim tests of S-0208 revision 53 (Q-2746 answered), claim 6: an order
//! whose behaviour is *capture* is not an enter order for the rule that
//! keeps a stored path. The order to garrison a building gives *capture*
//! (S-0214 row I8, S-0508 claim 2), so a walker under *move* ordered to
//! garrison drops its path, finishes the step under way and heads for the
//! building; a walker under *capture* ordered to a free cell drops its path
//! too and heads for the new cell. Both positions of
//! `quirks.enter_keeps_stored_path`. Scenario W17 parts (j) and (k), on
//! synthetic data: invented type names and values on the flat scene of
//! `common`. No game data.

mod common;

use common::{FOOT, Scene, WHEEL, cell, open_orders, order_move, unit_type};
use glasnost_map::CellXY;
use glasnost_rules::sim::BuildingType;
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::tick;
use glasnost_sim::world::{HouseId, ObjectId, Pos, Target, World};

/// The garrisoning soldier type GARRI (speed field 4), the garrisonable
/// building HUT (one cell, three occupants) and the wheeled transport VAN.
fn scene(keep: bool) -> Scene {
    let mut s = Scene::new();
    let mut man = unit_type(FOOT, 4, 0);
    man.occupier = Some(true);
    s.infantry("GARRI", man);
    s.building(
        "HUT",
        BuildingType {
            strength: Some(400),
            foundation: Some([1, 1]),
            can_be_occupied: Some(true),
            max_number_occupants: Some(3),
            ..Default::default()
        },
    );
    let mut van = unit_type(WHEEL, 6, 8);
    van.passengers = Some(4);
    s.vehicle("VAN", van);
    s.ruleset.globals.quirks.enter_keeps_stored_path = keep;
    s
}

/// A soldier at standing position 0 of the middle cell and an empty HUT at
/// `hut`, orders open.
fn with_hut(keep: bool, hut: CellXY) -> (World, ObjectId, ObjectId) {
    let mut s = scene(keep);
    s.place_infantry(0, "GARRI", cell(0, 0), 0);
    s.place_structure(0, "HUT", hut);
    let mut w = s.build();
    open_orders(&mut w);
    let man = common::objects_of(&w, Kind::Infantry)[0];
    let g = common::objects_of(&w, Kind::Building)[0];
    (w, man, g)
}

fn pos(w: &World, id: ObjectId) -> Pos {
    w.obj(id).unwrap().pos
}

fn walk_target(w: &World, id: ObjectId) -> Option<Pos> {
    w.obj(id)
        .and_then(|o| o.walk.as_ref())
        .and_then(|s| s.target)
}

fn steps(w: &World, id: ObjectId) -> Vec<u8> {
    w.obj(id).unwrap().path.steps.clone()
}

/// A garrison order on `g` as S-0214 row I8 gives it: a *capture* order
/// with `g` as destination, issued for the actor's owner.
fn order_garrison(w: &mut World, actor: ObjectId, g: ObjectId) {
    let order = Order::Behaviour {
        actor,
        behaviour: Behaviour::Capture,
        attack_move: false,
        target: None,
        dest: Some(Target::Object(g)),
    };
    assert!(
        orders::issue(w, HouseId(0), order),
        "garrison order refused"
    );
}

/// Runs until the soldier is between its first and second standing
/// positions (a walking target in `next`, still in the start cell).
fn until_mid_step(w: &mut World, id: ObjectId, next: CellXY) -> Pos {
    for _ in 0..200 {
        tick::step(w);
        if walk_target(w, id).is_some_and(|t| t.cell() == next)
            && pos(w, id).cell() == cell(0, 0)
            && w.obj(id).unwrap().walk.as_ref().unwrap().arrived != Some(w.tick - 1)
        {
            break;
        }
    }
    walk_target(w, id).expect("a step under way")
}

/// The cell of the first walking target after the step under way.
fn first_next(w: &mut World, id: ObjectId) -> CellXY {
    let under_way = walk_target(w, id);
    for _ in 0..80 {
        tick::step(w);
        if let Some(b) = walk_target(w, id)
            && Some(b) != under_way
        {
            return b.cell();
        }
    }
    panic!("no next walking target");
}

// spec: S-0208 claim 6 (r53, Q-2746: an order whose behaviour is capture is not an enter order for the rule, and the order to garrison a building gives capture, S-0214 row I8; so a walker under move ordered to garrison drops its path, finishes the step under way and then heads for the building, in both positions of quirks.enter_keeps_stored_path); scenario W17 part j
#[test]
fn w17_j_a_garrison_order_to_a_walker_under_move_drops_its_path() {
    for keep in [true, false] {
        let (mut w, id, g) = with_hut(keep, cell(-2, 0));
        order_move(&mut w, id, cell(4, 0));
        until_mid_step(&mut w, id, cell(1, 0));
        assert!(
            !steps(&w, id).is_empty(),
            "keep {keep}: a stored path ahead"
        );
        order_garrison(&mut w, id, g);
        orders::apply_pending(&mut w);
        let o = w.obj(id).unwrap();
        assert!(
            o.path.steps.is_empty(),
            "keep {keep}: the stored path is dropped"
        );
        assert_eq!(o.dest, Some(Target::Object(g)));
        assert!(
            o.beh.seen() == Some(Behaviour::Capture) || o.beh.queued == Some(Behaviour::Capture),
            "keep {keep}: capture current or queued"
        );
        let next = first_next(&mut w, id);
        assert!(
            next.x <= cell(1, 0).x,
            "keep {keep}: back along its line towards the HUT, not on: {next:?}"
        );
        let mut inside = false;
        for _ in 0..400 {
            tick::step(&mut w);
            if w.obj(id).is_none_or(|o| !o.on_map) {
                inside = true;
                break;
            }
        }
        assert!(inside, "keep {keep}: it garrisons the HUT");
    }
}

// spec: S-0208 claim 6 (r53, Q-2746: a walker under capture ordered to a free cell drops its path too, and after the step under way it heads for the new cell, not for the building, with move becoming current and the new cell as destination; both positions of quirks.enter_keeps_stored_path); scenario W17 part k
#[test]
fn w17_k_a_move_order_to_a_walker_under_capture_drops_its_path() {
    for keep in [true, false] {
        let (mut w, id, g) = with_hut(keep, cell(8, 0));
        order_garrison(&mut w, id, g);
        orders::apply_pending(&mut w);
        until_mid_step(&mut w, id, cell(1, 0));
        assert_eq!(w.obj(id).unwrap().beh.current, Some(Behaviour::Capture));
        assert!(
            !steps(&w, id).is_empty(),
            "keep {keep}: a stored path ahead"
        );
        let new = cell(0, 3);
        order_move(&mut w, id, new);
        orders::apply_pending(&mut w);
        assert!(
            steps(&w, id).is_empty(),
            "keep {keep}: the stored path is dropped"
        );
        let first = first_next(&mut w, id);
        assert_ne!(first, cell(2, 0), "keep {keep}: not on towards the HUT");
        assert!(
            first.y > cell(0, 0).y,
            "keep {keep}: towards the new cell: {first:?}"
        );
        let o = w.obj(id).unwrap();
        assert_eq!(o.beh.current, Some(Behaviour::Move), "keep {keep}");
        assert_eq!(o.dest, Some(Target::Cell(new)), "keep {keep}");
    }
}

// spec: S-0208 claim 6 (r53: an enter order, into a transport for example, still keeps the path of a walker under move with the switch on); the control of part j, part f on the same layout with the VAN in the HUT's place
#[test]
fn w17_j_control_an_enter_order_on_the_same_layout_keeps_the_path() {
    let mut s = scene(true);
    s.place_infantry(0, "GARRI", cell(0, 0), 0);
    s.place_unit(0, "VAN", cell(-2, 0), 0);
    let mut w = s.build();
    open_orders(&mut w);
    let id = common::objects_of(&w, Kind::Infantry)[0];
    let v = common::objects_of(&w, Kind::Vehicle)[0];
    order_move(&mut w, id, cell(4, 0));
    until_mid_step(&mut w, id, cell(1, 0));
    let before = steps(&w, id);
    assert!(!before.is_empty());
    let order = Order::Behaviour {
        actor: id,
        behaviour: Behaviour::Enter,
        attack_move: false,
        target: None,
        dest: Some(Target::Object(v)),
    };
    assert!(orders::issue(&mut w, HouseId(0), order));
    orders::apply_pending(&mut w);
    assert_eq!(steps(&w, id), before, "kept under enter");
    assert_eq!(first_next(&mut w, id), cell(2, 0), "on along its line");
}
