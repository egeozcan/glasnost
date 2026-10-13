//! Claim tests for the approach step (S-0215 claim 37, F10), on synthetic
//! data: the combat ruleset's CRUSHER (weapon range 5.5 cells, 1408
//! leptons, so r′ = 1280) against a CART.

use std::sync::Arc;

use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_sim::approach;
use glasnost_sim::rules::{Behaviour, Kind, SimRules};
use glasnost_sim::testkit;
use glasnost_sim::world::{ObjectId, Target, World};

const SIZE: i32 = 32;

fn at(dx: i32, dy: i32) -> CellXY {
    testkit::mid(SIZE, dx, dy)
}

fn rules(can_approach: Option<bool>, m: Option<i32>) -> Ruleset {
    let mut r = testkit::combat_ruleset();
    r.sim
        .vehicles
        .get_mut("CRUSHER")
        .unwrap()
        .can_approach_target = can_approach;
    r.sim.general.approach_target_reset_multiplier = m;
    r
}

/// An attacker of `house` at (0, 0) whose target is a CART of house 1 at
/// (`tx`, 0), in behaviour `b`.
fn duel(w: &mut World, house: u8, tx: i32, b: Behaviour) -> (ObjectId, ObjectId) {
    let a = testkit::spawn(w, Kind::Vehicle, "CRUSHER", house, at(0, 0), 0);
    let t = testkit::spawn(w, Kind::Vehicle, "CART", 1, at(tx, 0), 0);
    let o = w.obj_mut(a).unwrap();
    o.target = Some(Target::Object(t));
    o.beh.current = Some(b);
    (a, t)
}

// spec: S-0215 claim 37 (d) (a target out of range is kept or dropped by the current behaviour, the can-approach property and the owner; dropping clears the destination too)
// spec: S-0215 claim 37 (b) (a target in range is never dropped)
#[test]
fn keep_or_drop_table() {
    // (behaviour, may approach, house: 0 human-controlled, 2 the neutral house, kept)
    let rows = [
        (Behaviour::Sticky, true, 0, false),
        (Behaviour::Attack, false, 0, true),
        (Behaviour::AreaGuard, true, 2, true),
        (Behaviour::AreaGuard, false, 0, true),
        (Behaviour::AreaGuard, false, 2, false),
        (Behaviour::Hunt, true, 0, true),
        (Behaviour::Hunt, false, 2, true),
        (Behaviour::Hunt, false, 0, false),
        (Behaviour::Guard, true, 0, true),
        (Behaviour::Guard, false, 0, false),
    ];
    for (b, may, house, kept) in rows {
        let r = rules(Some(may), None);
        let mut w = testkit::game_with_non_players(&r, SIZE, 3);
        let (a, t) = duel(&mut w, house, 10, b);
        w.obj_mut(a).unwrap().dest = Some(Target::Cell(at(9, 0)));
        approach::step(&mut w, a);
        let o = w.obj(a).unwrap();
        assert_eq!(
            o.target == Some(Target::Object(t)),
            kept,
            "{b:?} {may} {house}"
        );
        if !kept {
            assert_eq!(o.dest, None, "{b:?}: dropped with the destination");
        }
        // In range: never dropped.
        let mut w = testkit::game_with_non_players(&r, SIZE, 3);
        let (a, t) = duel(&mut w, house, 3, b);
        approach::step(&mut w, a);
        assert_eq!(
            w.obj(a).unwrap().target,
            Some(Target::Object(t)),
            "{b:?} in range"
        );
    }
}

/// The destination after the step, for an attacker with the target out of
/// range at (10, 0) and a destination at (`dx`, 0).
fn renewed(m: Option<i32>, dx: i32, tx: i32) -> Option<Target> {
    let r = rules(None, m);
    let mut w = testkit::game_with(&r, SIZE, 3);
    let (a, _) = duel(&mut w, 0, tx, Behaviour::Attack);
    w.obj_mut(a).unwrap().dest = Some(Target::Cell(at(dx, 0)));
    approach::step(&mut w, a);
    w.obj(a).unwrap().dest
}

// spec: S-0215 claim 37 (e) (a destination is cleared when the target is out of range and more than m × r′ from it; kept otherwise, also in range)
// spec: S-0215 F10 (r′ = ρ − 128 for ρ ≥ 512; ρ₃(target, destination) > m × r′, strict; m × r′ wraps as a signed 32-bit product)
// spec: S-0215 Constants (approach reset multiplier absent: 1)
// spec: S-0215 claim 37 (e) (a cleared destination: part f applies in the same step, here S-0307 claim 15's search)
#[test]
fn destination_renewal_threshold() {
    let keep = Some(Target::Cell(at(9, 0)));
    // ρ₃ = 256 ≤ 1280: kept.
    assert_eq!(renewed(None, 9, 10), keep);
    // ρ₃ = 1280 = 1 × r′: kept (strict comparison); 1536 > 1280: cleared.
    assert_eq!(renewed(None, 5, 10), Some(Target::Cell(at(5, 0))));
    // Cleared: part f searches in the same step (S-0307 claim 15) and gives a new destination.
    let fresh = renewed(None, 4, 10);
    assert!(fresh.is_some() && fresh != Some(Target::Cell(at(4, 0))));
    // m = 2: 2560 ≥ 1536, kept.
    assert_eq!(renewed(Some(2), 4, 10), Some(Target::Cell(at(4, 0))));
    // m × r′ = 1677722 × 1280 wraps below 0: even a destination on the target is cleared.
    assert_ne!(
        renewed(Some(1_677_722), 10, 10),
        Some(Target::Cell(at(10, 0)))
    );
    // The target in range: the destination is kept whatever its distance.
    assert_eq!(renewed(Some(1_677_722), 9, 3), keep);
}

/// A game whose map has a column of rock at x = 2, splitting the map.
fn walled_game(r: &Ruleset) -> World {
    use glasnost_map::terrain::{TileCell, TileInfo};
    let mut cat = (*testkit::catalogue()).clone();
    let rock_tile = cat.tiles.len() as i32;
    cat.tiles.push(TileInfo {
        width: 1,
        height: 1,
        has_image: true,
        cells: vec![Some(TileCell {
            terrain_code: 7,
            slope: 0,
        })],
        variants: 0,
        anim: None,
        no_burrowing: false,
    });
    let mut map = testkit::empty_map(SIZE);
    let wall_x = at(2, 0).x;
    let cells: Vec<CellXY> = map.cells.iter().map(|(c, _)| c).collect();
    for c in cells {
        if c.x == wall_x {
            map.cells.get_mut(c).unwrap().tile = rock_tile;
        }
    }
    glasnost_sim::load::new_game(
        Arc::new(SimRules::new(r, "claims-s0215-approach")),
        &map,
        Arc::new(cat),
        &testkit::setup(3),
    )
    .expect("synthetic game loads")
}

// spec: S-0215 claim 37 (a) (every tick, an actor with a target and no destination that cannot reach the target's cell takes the step, and drops a target out of range if it still has no destination)
#[test]
fn per_tick_drop_of_an_unreachable_target() {
    let r = rules(None, None);
    // Unreachable and out of range: dropped, in any behaviour (here guard).
    let mut w = walled_game(&r);
    let (a, _) = duel(&mut w, 0, 8, Behaviour::Guard);
    approach::per_tick(&mut w, a);
    assert_eq!(w.obj(a).unwrap().target, None);
    // The same through a tick.
    let mut w = walled_game(&r);
    let (a, _) = duel(&mut w, 0, 8, Behaviour::Guard);
    testkit::run(&mut w, 1);
    assert_eq!(w.obj(a).unwrap().target, None);
    // Unreachable but in range: kept.
    let mut w = walled_game(&r);
    let (a, t) = duel(&mut w, 0, 4, Behaviour::Guard);
    approach::per_tick(&mut w, a);
    assert_eq!(w.obj(a).unwrap().target, Some(Target::Object(t)));
    // Reachable and out of range: the per-tick rule does not apply.
    let mut w = testkit::game_with(&r, SIZE, 3);
    let (a, t) = duel(&mut w, 0, 8, Behaviour::Guard);
    approach::per_tick(&mut w, a);
    assert_eq!(w.obj(a).unwrap().target, Some(Target::Object(t)));
}

// spec: S-0207 claim 19 (a) (a driving vehicle never stops, nor loses its destination or a stored step, because its target came in range; it drives on to its destination)
#[test]
fn driving_attacker_keeps_its_move_in_range() {
    let r = rules(None, None);
    let mut w = testkit::game_with(&r, SIZE, 3);
    let (a, t) = duel(&mut w, 0, 12, Behaviour::Attack);
    // A sturdy target, so that it outlives the drive.
    w.obj_mut(t).unwrap().health = i32::MAX / 2;
    let goal = at(10, 0);
    w.obj_mut(a).unwrap().dest = Some(Target::Cell(goal));
    let mut in_range_moving = false;
    for _ in 0..400 {
        let before = w.obj(a).unwrap().path.steps.clone();
        testkit::run(&mut w, 1);
        let o = w.obj(a).unwrap();
        if o.dest.is_none() {
            assert_eq!(o.pos.cell(), goal, "stopped only at the destination");
            break;
        }
        if approach::in_range(&w, a, Target::Object(t)) {
            in_range_moving = true;
            assert_eq!(o.dest, Some(Target::Cell(goal)));
            // No step is dropped: at most the one just taken is gone.
            assert!(
                o.path.steps.len() + 1 >= before.len(),
                "steps kept: {before:?} -> {:?}",
                o.path.steps
            );
        }
    }
    assert!(in_range_moving, "the target came in range on the way");
    assert_eq!(w.obj(a).unwrap().target, Some(Target::Object(t)));
}
