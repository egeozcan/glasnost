//! S-0213 claim 7 (group destinations, revision 46) on synthetic data: the
//! deploy fixtures' plain vehicle TANK and infantry WALKER on the flat test
//! map. Every name and number is invented. The layouts follow Scenario O10
//! (main part, parts b and f) and the claim's statements for compact blocks,
//! rows and the infantry of one starting cell.

mod deploy_common;

use deploy_common::*;
use glasnost_map::CellXY;
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::world::{ObjectId, Target, World};

/// A move order to `c`.
fn mv(actor: ObjectId, c: CellXY) -> Order {
    Order::Behaviour {
        actor,
        behaviour: Behaviour::Move,
        attack_move: false,
        target: None,
        dest: Some(Target::Cell(c)),
    }
}

/// Issues one run of move orders to `c` in the order of `ids`, applies it and
/// returns each member's destination cell.
fn group_move(w: &mut World, ids: &[ObjectId], c: CellXY) -> Vec<CellXY> {
    for &id in ids {
        orders::issue(w, A, mv(id, c));
    }
    step(w);
    ids.iter()
        .map(|&id| match w.obj(id).unwrap().dest {
            Some(Target::Cell(d)) => d,
            other => panic!("{other:?}"),
        })
        .collect()
}

fn cell_of(w: &World, id: ObjectId) -> CellXY {
    w.obj(id).unwrap().pos.cell()
}

fn plus(c: CellXY, dx: i32, dy: i32) -> CellXY {
    CellXY::new(c.x + dx, c.y + dy)
}

/// Vehicles of TANK at the given offsets, one per cell at its centre.
fn vehicles(w: &mut World, at: &[(i32, i32)]) -> Vec<ObjectId> {
    at.iter()
        .map(|&(dx, dy)| put(w, "TANK", Kind::Vehicle, 0, dx, dy))
        .collect()
}

/// Checks that every destination is the common cell plus the member's
/// displacement from `leader`.
fn reproduces_shape(
    w: &World,
    ids: &[ObjectId],
    dests: &[CellXY],
    leader: ObjectId,
    common: CellXY,
) {
    let l = cell_of(w, leader);
    for (&id, &d) in ids.iter().zip(dests) {
        let m = cell_of(w, id);
        assert_eq!(d, plus(common, m.x - l.x, m.y - l.y), "member at {m:?}");
    }
}

// spec: S-0213 claim 7 (Scenario O10 main part: a 3 × 3 block of vehicles one per cell at its cell's centre reproduces its shape around the common cell; the centre vehicle leads and keeps the common cell; a diagonal vehicle one cell out gets the diagonal neighbour, the first point lying one cell out)
#[test]
fn o10_block_3x3_reproduces_its_shape() {
    let mut w = game();
    let mut at = Vec::new();
    for dy in -1..=1 {
        for dx in -1..=1 {
            at.push((dx, dy));
        }
    }
    let ids = vehicles(&mut w, &at);
    let common = deploy_common::at(0, 10);
    let dests = group_move(&mut w, &ids, common);
    reproduces_shape(&w, &ids, &dests, ids[4], common);
    assert_eq!(dests[4], common);
    assert_eq!(dests[0], plus(common, -1, -1));
}

// spec: S-0213 claim 7 (Scenario O10 part f (i): a 4 × 3 block reproduces its shape, no two share; among equal distances to the centre the member latest in the run leads)
#[test]
fn o10f_block_4x3_reproduces_its_shape() {
    let mut w = game();
    let mut at = Vec::new();
    for dy in -1..=1 {
        for dx in -2..=1 {
            at.push((dx, dy));
        }
    }
    let ids = vehicles(&mut w, &at);
    let common = deploy_common::at(0, 10);
    let dests = group_move(&mut w, &ids, common);
    // The centre lies between the columns −1 and 0 of the middle row, so the
    // two members there are equally near; the later in the run, (0, 0), leads.
    let leader = ids[at.iter().position(|&p| p == (0, 0)).unwrap()];
    reproduces_shape(&w, &ids, &dests, leader, common);
    let mut u = dests.clone();
    u.sort();
    u.dedup();
    assert_eq!(u.len(), 12);
}

// spec: S-0213 claim 7 (Scenario O10 part f (ii): a row of 13 along a map axis reproduces itself; in a row of 14 only the vehicle 7 cells from the leader shares, with the one 6 out on its side, since all 6 points of its look are taken: the limit of 6 points for each member on its own and the fallback to the last taken cell)
#[test]
fn o10f_rows_of_13_and_14() {
    let mut w = game();
    let at: Vec<(i32, i32)> = (-6..=6).map(|dx| (dx, -6)).collect();
    let ids = vehicles(&mut w, &at);
    let common = deploy_common::at(0, 8);
    let dests = group_move(&mut w, &ids, common);
    reproduces_shape(&w, &ids, &dests, ids[6], common);

    let mut w = game();
    let at: Vec<(i32, i32)> = (-7..=6).map(|dx| (dx, -6)).collect();
    let ids = vehicles(&mut w, &at);
    let common = deploy_common::at(0, 8);
    let dests = group_move(&mut w, &ids, common);
    // Centre at −0.5 cells: (−1) and (0) are equally near, (0) is later.
    let leader = ids[7];
    assert_eq!(dests[7], common);
    for (k, (&id, &d)) in ids.iter().zip(&dests).enumerate() {
        let dx = cell_of(&w, id).x - cell_of(&w, leader).x;
        if dx == -7 {
            assert_eq!(d, plus(common, -6, 0), "the 7th out shares the 6th's cell");
            assert_eq!(d, dests[1]);
        } else {
            assert_eq!(d, plus(common, dx, 0), "member {k}");
        }
    }
}

// spec: S-0213 claim 7 (Scenario O10 part f (iii) and part b: infantry of one starting cell share one destination, the shared-cell rule; the leader's cell gets the common cell through the leader, and each other starting cell gets the neighbour of the common cell in its own direction)
#[test]
fn o10f_infantry_three_per_cell() {
    let mut w = game();
    let mut ids = Vec::new();
    for &(dx, dy) in &[(0, 0), (1, 0), (0, 1), (1, 1)] {
        for _ in 0..3 {
            ids.push(put(&mut w, "WALKER", Kind::Infantry, 0, dx, dy));
        }
    }
    let common = deploy_common::at(4, 10);
    let dests = group_move(&mut w, &ids, common);
    // All twelve stand at their cells' centres, so the four cells are equally
    // near the centre and the member latest in the run leads: cell (1, 1).
    let lead = cell_of(&w, ids[11]);
    for (k, &id) in ids.iter().enumerate() {
        let m = cell_of(&w, id);
        let want = plus(common, m.x - lead.x, m.y - lead.y);
        assert_eq!(dests[k], want, "member {k} at {m:?}");
        assert_eq!(
            dests[k],
            dests[k / 3 * 3],
            "one starting cell, one destination"
        );
    }
    let mut u = dests.clone();
    u.sort();
    u.dedup();
    assert_eq!(u.len(), 4);
}

// spec: S-0213 claim 7 (open class: a cell whose entry outcome for the member is a stationary friendly is open and can be given, Q-1750 question 3)
#[test]
fn a_cell_held_by_a_friendly_unit_is_open() {
    let mut w = game();
    // Issue order: b first, then a, so a (later, equally near) leads.
    let a = put(&mut w, "TANK", Kind::Vehicle, 0, 0, 0);
    let b = put(&mut w, "TANK", Kind::Vehicle, 0, 1, 0);
    let common = deploy_common::at(0, 10);
    let friend = put(&mut w, "TANK", Kind::Vehicle, 0, 1, 10);
    assert_eq!(cell_of(&w, friend), plus(common, 1, 0));
    let dests = group_move(&mut w, &[b, a], common);
    assert_eq!(dests, vec![plus(common, 1, 0), common]);
}

// spec: S-0213 claim 7 (an obstructed cell spoils the look: the member gets its fallback cell, here the common cell, and no further search follows, quirks.group_spread_no_detour)
#[test]
fn an_enemy_vehicle_spoils_the_look() {
    let mut w = game();
    let a = put(&mut w, "TANK", Kind::Vehicle, 0, 0, 0);
    let b = put(&mut w, "TANK", Kind::Vehicle, 0, 1, 0);
    let common = deploy_common::at(0, 10);
    put(&mut w, "TANK", Kind::Vehicle, 1, 1, 10);
    assert!(w.rules().quirks.group_spread_no_detour);
    let dests = group_move(&mut w, &[b, a], common);
    assert_eq!(dests, vec![common, common]);
}

// spec: S-0213 claim 7 (the obstructed class's entry check takes the common cell's height level as its reference, plus 4 when the common cell is on a high-bridge deck: from a deck common cell the ground layer of a deck look cell does not count)
#[test]
fn entry_check_reference_is_the_common_cell() {
    let mut w = game();
    let a = put(&mut w, "TANK", Kind::Vehicle, 0, 0, 0);
    let b = put(&mut w, "TANK", Kind::Vehicle, 0, 1, 0);
    let common = deploy_common::at(0, 10);
    // An enemy vehicle on the ground under a deck cell next to a deck common
    // cell: with the common cell's reference (4 above the ground) the deck's
    // objects count, and the deck is empty, so the cell is open. The member
    // itself stands on the ground (reference 0), which would make the cell
    // obstructed.
    put(&mut w, "TANK", Kind::Vehicle, 1, 1, 10);
    for c in [common, plus(common, 1, 0)] {
        let i = w.cells.idx(c).unwrap();
        w.cells.deck[i] = true;
    }
    let dests = group_move(&mut w, &[b, a], common);
    assert_eq!(dests, vec![plus(common, 1, 0), common]);
}

// spec: S-0213 claim 7 (a cell met twice uses up two of the 6 points: on a diagonal the first two points fall in one cell, so in a diagonal row the vehicle 5 out finds all 6 points taken and shares the cell of the one 4 out)
#[test]
fn a_cell_met_twice_uses_up_two_points() {
    let mut w = game();
    let at: Vec<(i32, i32)> = (-5..=5).map(|k| (k, k)).collect();
    let ids = vehicles(&mut w, &at);
    let common = deploy_common::at(0, 8);
    let dests = group_move(&mut w, &ids, common);
    assert_eq!(dests[5], common);
    for (&(k, _), &d) in at.iter().zip(&dests) {
        let want = match k {
            5 => plus(common, 4, 4),
            -5 => plus(common, -4, -4),
            _ => plus(common, k, k),
        };
        assert_eq!(d, want, "member {k} out");
    }
}
