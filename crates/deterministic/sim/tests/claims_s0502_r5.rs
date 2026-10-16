//! Claim tests of S-0502 revision 5 (movement of jumpjets) on synthetic data:
//! cell listing and the low-flight placements (claim 4), the clearance
//! listings (claim 22), the search settings and the infantry standing
//! choice (claim 23), a stop order in each flight state (claim 24, S-0213
//! claim 8), zero flight fields (claim 25) and the flight recomputes
//! (claim 26, S-0211 claim 5).

mod jump_common;

use glasnost_map::CellXY;
use glasnost_map::terrain::land;
use glasnost_num::F64;
use glasnost_sim::jump::{self, JumpState, Phase};
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::Kind;
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, Layer, ObjectId, Pos, Target, World};
use jump_common::*;

fn j(w: &World, id: ObjectId) -> &JumpState {
    w.obj(id).unwrap().jump.as_ref().unwrap()
}

fn js(w: &mut World, id: ObjectId) -> &mut JumpState {
    w.obj_mut(id).unwrap().jump.as_mut().unwrap()
}

/// Runs until orders are accepted (S-0213 claim 1: the order block at game start).
fn unblock(w: &mut World) {
    while w.tick <= w.orders_blocked_through {
        step(w);
    }
}

fn stop(w: &mut World, id: ObjectId) {
    unblock(w);
    assert!(orders::issue(w, HouseId(0), Order::Stop { actor: id }));
    // The order is applied at the next order pass.
    step(w);
}

// spec: S-0502 claim 4 (a cruising or hovering jumpjet makes no new listing and no mark, and its existing listing stays in the old cell when it crosses cells; any other state removes its listing at the start of the next physics step): an own-cell order climbs past a quarter of H below 208 leptons, its last ascent placement survives into cruise, stays where it was while it crosses cells and goes at the first non-hover, non-cruise step
#[test]
fn claim_4_a_listing_survives_into_cruise_and_is_not_transferred() {
    let mut w = game();
    let u = put(&mut w, "JJ", Kind::Infantry, 0, 0, 0);
    let cell = pos(&w, u).cell();
    go(&mut w, u, cell);
    assert!(until(&mut w, 200, |w| phase(w, u) == Phase::Cruising));
    assert!(
        height(&w, u) < 208,
        "a low own-cell ascent-to-cruise transition"
    );
    assert_eq!(j(&w, u).listed.map(|l| l.0), Some(cell));
    assert!(w.cells.list(cell, Layer::Ground).contains(&u));
    // Cruise towards another cell: the listing is not moved.
    let far = at(6, 0);
    js(&mut w, u).dest = Some(Pos::centre(far, 0));
    js(&mut w, u).vm = F64::from_i64(14, glasnost_num::Rounding::Nearest);
    for _ in 0..160 {
        step(&mut w);
        if pos(&w, u).cell() != cell {
            break;
        }
    }
    assert_ne!(pos(&w, u).cell(), cell, "it crossed into another cell");
    assert!(
        w.cells.list(cell, Layer::Ground).contains(&u),
        "the old listing stays"
    );
    assert!(!w.cells.list(pos(&w, u).cell(), Layer::Ground).contains(&u));
    // A descending physics step removes it.
    js(&mut w, u).phase = Phase::Descending;
    js(&mut w, u).ht = 0;
    step(&mut w);
    assert!(
        !w.cells.list(cell, Layer::Ground).contains(&u),
        "removed by the descent"
    );
}

// spec: S-0502 claim 4 (a jumpjet at 208 leptons or more on flat land is not listed or marked; a higher jumpjet in hover or cruise makes no new placement)
#[test]
fn claim_4_no_listing_high_up() {
    let mut w = game();
    let u = put(&mut w, "JJ", Kind::Infantry, 0, 0, 0);
    jump::start_in_flight(&mut w, u, Phase::Hovering, 500);
    go(&mut w, u, at(0, 0));
    for _ in 0..5 {
        step(&mut w);
    }
    let c = pos(&w, u).cell();
    assert!(j(&w, u).listed.is_none() && j(&w, u).mark.is_none());
    assert!(!w.cells.list(c, Layer::Ground).contains(&u));
}

// spec: S-0502 claim 23 and S-0212 table T5 (a vehicle that is not balloon-hover: bridge cells are excluded, and the destination is the centre of the cell found; an infantry unit or balloon-hover vehicle may land on a bridge cell, in deck mode, and an infantry unit takes a side standing position after one ranged draw over 0 to 3, never the centre; a vehicle takes no draw)
#[test]
fn claim_23_search_settings_and_the_standing_choice() {
    let mut w = game();
    let c = at(6, 6);
    let i = w.cells.idx(c).unwrap();
    w.cells.deck[i] = true;
    let v = put(&mut w, "JV", Kind::Vehicle, 0, -6, 6);
    let before = w.rng.taken();
    go(&mut w, v, c);
    assert_eq!(w.rng.taken(), before, "a vehicle takes no draw");
    let d = j(&w, v).dest.unwrap();
    assert_ne!(d.cell(), c, "bridge cells are excluded for a vehicle");
    assert_eq!((d.x % 256, d.y % 256), (128, 128));
    // Infantry: the bridge cell may be found; one ranged draw, a side position, the deck height.
    for k in 0..6 {
        let mut w = game();
        let i = w.cells.idx(at(6, 6)).unwrap();
        w.cells.deck[i] = true;
        let u = put(&mut w, "JJ", Kind::Infantry, 0, -6 + k, 6);
        let before = w.rng.taken();
        go(&mut w, u, at(6, 6));
        assert_eq!(w.rng.taken(), before + 1, "one ranged draw over 0 to 3");
        let d = j(&w, u).dest.unwrap();
        assert_ne!((d.x % 256, d.y % 256), (128, 128), "never the centre");
        assert!(
            [(64, 64), (192, 64), (64, 192), (192, 192)].contains(&(d.x % 256, d.y % 256)),
            "a standing position of table P"
        );
        if d.cell() == at(6, 6) {
            assert_eq!(d.z, 416, "terrain height plus 416 on any high bridge");
        }
    }
}

// spec: S-0502 claim 16 and claim 24 (a failed adjustment keeps the ordered point as the movement destination, flight state, moving flag and speeds unchanged; one that would retain an off-map point is refused and the preceding point, state and landing reservation stay; a descending unit does not re-ascend on a failure)
#[test]
fn claim_16_and_24_failed_adjustment() {
    let mut w = game();
    let u = put(&mut w, "JV", Kind::Vehicle, 0, 0, 0);
    go(&mut w, u, at(8, 0));
    step(&mut w);
    assert_eq!(phase(&w, u), Phase::Ascending);
    let before = j(&w, u).clone();
    // Off the map, far beyond the search: refused.
    let off = CellXY::new(-300, -300);
    go(&mut w, u, off);
    assert_eq!(j(&w, u).dest, before.dest, "the preceding point stays");
    assert_eq!(j(&w, u).phase, before.phase);
    assert!(j(&w, u).moving);
    // A descending unit keeps its reservation when the refusal retains the movement.
    let d = {
        let d = put(&mut w, "JV", Kind::Vehicle, 0, 4, 4);
        go(&mut w, d, at(4, 4));
        jump::start_in_flight(&mut w, d, Phase::Descending, 40);
        js(&mut w, d).dest = Some(Pos::centre(at(4, 4), 0));
        js(&mut w, d).ht = 0;
        d
    };
    step(&mut w);
    let res = j(&w, d).reservation;
    assert!(res.is_some());
    go(&mut w, d, off);
    assert_eq!(j(&w, d).reservation, res, "the reservation stays");
    assert_eq!(j(&w, d).phase, Phase::Descending);
    // Every cell occupied: the search fails on the map; the ordered point stays, a descending unit does not re-ascend.
    let cells: Vec<CellXY> = w.cells.grid.iter().map(|(c, _)| c).collect();
    for c in cells {
        w.cells.claim_vehicle(c, Layer::Ground, true);
    }
    let target = at(10, 10);
    go(&mut w, d, target);
    assert_eq!(
        j(&w, d).phase,
        Phase::Descending,
        "no re-ascent on a failure"
    );
    assert_eq!(
        j(&w, d).dest,
        Some(Pos::centre(target, 0)),
        "the ordered point itself"
    );
}

// spec: S-0213 claim 8 and S-0502 claims 17 and 24 (a stop order clears the order-level destination and reroutes a jumpjet with a movement commitment: ascending stays ascending, cruising stays cruising, a hovering unit with a move order keeps its commitment, a descending one releases its reservation and re-ascends; an idle one is unchanged; the replacement is a movement point, the order-level destination stays empty)
#[test]
fn claim_24_stop_in_each_flight_state() {
    // Ascending.
    let mut w = game();
    let u = put(&mut w, "JV", Kind::Vehicle, 0, 0, 0);
    go(&mut w, u, at(10, 0));
    step(&mut w);
    step(&mut w);
    assert_eq!(phase(&w, u), Phase::Ascending);
    stop(&mut w, u);
    assert_eq!(phase(&w, u), Phase::Ascending);
    assert!(
        w.obj(u).unwrap().dest.is_none(),
        "{:?}",
        w.obj(u).unwrap().dest
    );
    assert!(j(&w, u).moving);
    assert!(j(&w, u).dest.is_some());
    assert_ne!(
        j(&w, u).dest.unwrap().cell(),
        at(10, 0),
        "a replacement point"
    );
    // Cruising.
    let mut w = game();
    let u = put(&mut w, "JV", Kind::Vehicle, 0, -4, 0);
    go(&mut w, u, at(8, 0));
    jump::start_in_flight(&mut w, u, Phase::Cruising, 500);
    stop(&mut w, u);
    assert_eq!(phase(&w, u), Phase::Cruising);
    assert!(w.obj(u).unwrap().dest.is_none() && j(&w, u).moving);
    assert_eq!(
        j(&w, u).dest.unwrap().cell(),
        pos(&w, u).cell(),
        "the current cell's centre"
    );
    // Hovering with a move order.
    let mut w = game();
    let u = put(&mut w, "JV", Kind::Vehicle, 0, 0, 0);
    go(&mut w, u, at(0, 0));
    let e = put(&mut w, "CART", Kind::Vehicle, 1, 18, 18);
    w.obj_mut(u).unwrap().target = Some(Target::Object(e));
    jump::start_in_flight(&mut w, u, Phase::Hovering, 500);
    js(&mut w, u).moving = true;
    stop(&mut w, u);
    assert!(j(&w, u).moving, "it keeps its movement commitment");
    assert_eq!(phase(&w, u), Phase::Hovering, "at application");
    assert!(w.obj(u).unwrap().dest.is_none());
    // Descending: the reservation goes and it re-ascends.
    let mut w = game();
    let u = put(&mut w, "JV", Kind::Vehicle, 0, 4, 4);
    go(&mut w, u, at(4, 4));
    jump::start_in_flight(&mut w, u, Phase::Descending, 60);
    js(&mut w, u).dest = Some(Pos::centre(at(4, 4), 0));
    js(&mut w, u).ht = 0;
    step(&mut w);
    assert!(j(&w, u).reservation.is_some());
    stop(&mut w, u);
    assert!(j(&w, u).reservation.is_none());
    assert_eq!(phase(&w, u), Phase::Ascending);
    assert_eq!(j(&w, u).ht, 500);
    // Grounded and idle: nothing changes.
    let mut w = game();
    let u = put(&mut w, "JJ", Kind::Infantry, 0, 0, 0);
    let before = j(&w, u).clone();
    stop(&mut w, u);
    assert_eq!(&before, j(&w, u));
    // Hovering without a move order: nothing changes.
    let mut w = game();
    let u = put(&mut w, "JJ", Kind::Infantry, 0, 0, 0);
    jump::start_in_flight(&mut w, u, Phase::Hovering, 500);
    let before = j(&w, u).clone();
    stop(&mut w, u);
    assert_eq!(&before, j(&w, u));
}

// spec: S-0502 claim 25 (zero flight fields, an engine design rule with no quirk switch): a speed of 0 gives a speed fraction of 0 at take-off and speed 0 in every cruise band; a turn rate of 0 leaves the middle cruise band empty, with no division by it
#[test]
fn claim_25_zero_speed_and_zero_turn_rate() {
    // S = 0.
    let mut r = ruleset();
    r.sim.infantry.get_mut("JJ").unwrap().jumpjet_speed = Some(0);
    let mut w = testkit::game_with(&r, SIZE, 7);
    testkit::run(&mut w, 4);
    let u = put(&mut w, "JJ", Kind::Infantry, 0, 0, 0);
    go(&mut w, u, at(8, 0));
    step(&mut w);
    assert_eq!(phase(&w, u), Phase::Ascending);
    assert!(
        j(&w, u).fraction.eq_num(F64::ZERO),
        "the take-off fraction is 0 for S = 0"
    );
    for _ in 0..80 {
        step(&mut w);
        assert!(j(&w, u).vm.eq_num(F64::ZERO) && j(&w, u).vc.eq_num(F64::ZERO));
    }
    assert!(height(&w, u) > 100, "vertical take-off keeps its own rules");
    assert_eq!(pos(&w, u).cell(), at(0, 0), "no horizontal travel");
    // T = 0: no middle band. At 100 leptons the default turn rate gives S / 2, zero gives S.
    let mut r = ruleset();
    r.sim.infantry.get_mut("JJ").unwrap().jumpjet_turn_rate = Some(0);
    let mut w0 = testkit::game_with(&r, SIZE, 7);
    testkit::run(&mut w0, 4);
    let mut w4 = game();
    let mut vm = Vec::new();
    for w in [&mut w0, &mut w4] {
        let u = put(w, "JJ", Kind::Infantry, 0, 0, 0);
        let target = w.obj(u).unwrap().pos;
        let p = Pos::new(target.x + 100, target.y, 0);
        go(w, u, at(0, 0));
        jump::start_in_flight(w, u, Phase::Cruising, 500);
        js(w, u).dest = Some(Pos::new(p.x, p.y, 0));
        step(w);
        vm.push(j(w, u).vm);
    }
    assert!(
        vm[0].eq_num(F64::from_i64(14, glasnost_num::Rounding::Nearest)),
        "no middle band: S"
    );
    assert!(
        vm[1].eq_num(F64::from_i64(7, glasnost_num::Rounding::Nearest)),
        "the default turn rate: S / 2"
    );
}

// spec: S-0502 claim 26 and S-0211 claim 5 (the flight recomputes): hovering and cruising do not recompute a cell merely by crossing it; in the other states the cell before every physics step is recomputed, even high above it or standing still; the cell recompute rebuilds the land type from the tile
#[test]
fn claim_26_flight_recomputes() {
    let marker = land::WATER;
    // Cruise crossing: no recompute of the cells passed.
    let mut w = game();
    let u = put(&mut w, "JJ", Kind::Infantry, 0, -4, 0);
    go(&mut w, u, at(8, 0));
    jump::start_in_flight(&mut w, u, Phase::Cruising, 500);
    js(&mut w, u).vm = F64::from_i64(14, glasnost_num::Rounding::Nearest);
    let path: Vec<CellXY> = (-4..=2).map(|x| at(x, 0)).collect();
    for &c in &path {
        w.cells.grid.get_mut(c).unwrap().land = marker;
    }
    for _ in 0..110 {
        step(&mut w);
    }
    assert!(pos(&w, u).cell().x >= at(0, 0).x, "it crossed cells");
    for &c in &path {
        assert_eq!(
            w.cells.grid.get(c).unwrap().land,
            marker,
            "no recompute at {c:?}"
        );
    }
    // High ascent: the cell left is recomputed at every step, with no horizontal travel.
    let mut w = game();
    let u = put(&mut w, "JJ", Kind::Infantry, 0, 0, 0);
    let c = pos(&w, u).cell();
    go(&mut w, u, at(0, 0));
    jump::start_in_flight(&mut w, u, Phase::Ascending, 300);
    js(&mut w, u).ht = 500;
    w.cells.grid.get_mut(c).unwrap().land = marker;
    step(&mut w);
    assert_ne!(
        w.cells.grid.get(c).unwrap().land,
        marker,
        "recomputed though standing still, high above"
    );
    // A hovering jumpjet recomputes nothing.
    let mut w = game();
    let u = put(&mut w, "JJ", Kind::Infantry, 0, 0, 0);
    let c = pos(&w, u).cell();
    go(&mut w, u, at(0, 0));
    jump::start_in_flight(&mut w, u, Phase::Hovering, 500);
    w.cells.grid.get_mut(c).unwrap().land = marker;
    for _ in 0..5 {
        step(&mut w);
    }
    assert_eq!(w.cells.grid.get(c).unwrap().land, marker);
}

// spec: S-0502 claim 22 (a listed jumpjet supplies the 85-lepton allowance in a cell's ground layer, including to itself; the clearance reads actual listings): a cruising jumpjet that kept a low listing in its cell raises its own ground reference by the allowance
#[test]
fn claim_22_a_retained_listing_counts_for_its_owner() {
    let mut w = game();
    let u = put(&mut w, "JJ", Kind::Infantry, 0, 0, 0);
    let cell = pos(&w, u).cell();
    // Cruising over its own cell with no listing: the reference is the plain terrain; with a retained listing it is raised.
    go(&mut w, u, at(6, 0));
    jump::start_in_flight(&mut w, u, Phase::Cruising, 300);
    js(&mut w, u).vm = F64::from_i64(1, glasnost_num::Rounding::Nearest);
    let free = {
        let mut w2 = game();
        let u2 = put(&mut w2, "JJ", Kind::Infantry, 0, 0, 0);
        go(&mut w2, u2, at(6, 0));
        jump::start_in_flight(&mut w2, u2, Phase::Cruising, 300);
        js(&mut w2, u2).vm = F64::from_i64(1, glasnost_num::Rounding::Nearest);
        step(&mut w2);
        height(&w2, u2)
    };
    // Give it the retained listing.
    w.cells.enter(cell, Layer::Ground, u, false);
    js(&mut w, u).listed = Some((cell, Layer::Ground));
    step(&mut w);
    assert!(
        height(&w, u) > free,
        "the listed unit raises its own ground reference by the 85-lepton allowance: {} against {free}",
        height(&w, u)
    );
    assert!(
        w.cells.list(cell, Layer::Ground).contains(&u),
        "the listing stays in the old cell"
    );
}

#[allow(dead_code)]
fn unused(_: Target) {}

// spec: S-0502 claim 4 and S-0511 claim 9 (a listing that another rule moved, as a bridge collapse moves a deck listing to the ground, does not outlive the next placement removal of the jumpjet)
#[test]
fn claim_4_a_moved_listing_is_not_left_behind() {
    let mut w = game();
    let u = put(&mut w, "JJ", Kind::Infantry, 0, 0, 0);
    let cell = pos(&w, u).cell();
    // The tracked listing is stale: the unit is really in the ground list.
    js(&mut w, u).listed = Some((cell, Layer::Deck));
    go(&mut w, u, at(8, 0));
    assert!(until(&mut w, 200, |w| height(w, u) >= 208));
    for c in [cell, pos(&w, u).cell()] {
        assert!(!w.cells.list(c, Layer::Ground).contains(&u));
        assert!(!w.cells.list(c, Layer::Deck).contains(&u));
    }
}
