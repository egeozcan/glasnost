//! Claim tests of S-0502 revision 6 (movement of jumpjets) on synthetic data:
//! the exact listing predicate (claim 4, answers Q-1010), own-cell orders
//! (claim 23, answers Q-1011) and effective destination clears (claim 24,
//! answers Q-1012). Every name and number is invented; no game data.

mod jump_common;

use glasnost_sim::econ::assign_destination;
use glasnost_sim::jump::{self, JumpState, Phase};
use glasnost_sim::rules::Kind;
use glasnost_sim::world::{ObjectId, Pos, Target, World};
use jump_common::*;

fn j(w: &World, id: ObjectId) -> &JumpState {
    w.obj(id).unwrap().jump.as_ref().unwrap()
}

fn js(w: &mut World, id: ObjectId) -> &mut JumpState {
    w.obj_mut(id).unwrap().jump.as_mut().unwrap()
}

fn deck_cell(w: &mut World, c: glasnost_map::CellXY) {
    let i = w.cells.idx(c).unwrap();
    w.cells.deck[i] = true;
}

/// A jumpjet hovering `r` leptons above the surface of its cell.
fn at_height(w: &mut World, name: &str, kind: Kind, dx: i32, dy: i32, r: i32) -> ObjectId {
    let u = put(w, name, kind, 0, dx, dy);
    jump::start_in_flight(w, u, Phase::Hovering, r);
    let p = pos(w, u);
    let z = w.surface_at(p.x, p.y) + r;
    w.obj_mut(u).unwrap().pos.z = z;
    u
}

// spec: S-0502 claim 4 (revision 6, answers Q-1010): on flat land a new listing qualifies exactly when r < 208 leptons; 1, 50 and 100 qualify, 208 and 250 do not, whatever the type's cruise height
#[test]
fn claim_4_flat_land_predicate() {
    for name in ["JJ", "JJTALL"] {
        for (r, want) in [
            (1, true),
            (50, true),
            (100, true),
            (207, true),
            (208, false),
            (250, false),
        ] {
            let mut w = game();
            let u = at_height(&mut w, name, Kind::Infantry, 0, 0, r);
            assert_eq!(jump::placement_qualifies(&w, u), want, "{name} r = {r}");
        }
    }
}

// spec: S-0502 claim 4 and S-0102 claim 3 (revision 6): over a high bridge with the on-deck state unset, r′ = r − 416 for r >= 416 and no falling state; a listing qualifies when r < 208 or r′ = 0, so 100 and 416 qualify and 417 and 516 do not (positive clearance below 208 above the deck is not enough)
#[test]
fn claim_4_bridge_predicate_uses_two_heights() {
    for (r, want) in [(100, true), (416, true), (417, false), (516, false)] {
        let mut w = game();
        deck_cell(&mut w, at(0, 0));
        let u = at_height(&mut w, "JJ", Kind::Infantry, 0, 0, r);
        assert!(!j(&w, u).on_deck);
        assert_eq!(jump::placement_qualifies(&w, u), want, "r = {r}");
    }
}

// spec: S-0502 claim 4 (revision 6): starting a crash does not set the bridge/parachute falling state, so a crashing jumpjet still gets the subtraction; an independently falling one does not
#[test]
fn claim_4_crashing_is_not_falling() {
    let mut w = game();
    deck_cell(&mut w, at(0, 0));
    let u = at_height(&mut w, "JJ", Kind::Infantry, 0, 0, 416);
    js(&mut w, u).phase = Phase::Crashing;
    assert!(jump::placement_qualifies(&w, u), "crashing alone");
    w.obj_mut(u).unwrap().fall.falling = true;
    assert!(
        !jump::placement_qualifies(&w, u),
        "independently falling at 416"
    );
    // Falling at a low clearance still qualifies by r < 208.
    let low = at_height(&mut w, "JJ", Kind::Infantry, 0, 0, 100);
    w.obj_mut(low).unwrap().fall.falling = true;
    assert!(jump::placement_qualifies(&w, low));
}

// spec: S-0502 claim 23 and S-0213 claim 21 (revision 6, answers Q-1011): outside an ion storm a jumpjet-type infantry unit ignores a nonempty destination assignment whose cell equals its current cell, also while grounded, before any search or draw; S-0502 claim 23 (r24, Q-3043: the order-level destination keeps its preceding value, here empty)
#[test]
fn claim_23_infantry_own_cell_order_is_ignored() {
    let mut w = game();
    let u = put(&mut w, "JJ", Kind::Infantry, 0, 0, 0);
    let before = j(&w, u).clone();
    let taken = w.rng.taken();
    go(&mut w, u, at(0, 0));
    let now = j(&w, u).clone();
    assert_eq!(now, before, "the locomotor takes nothing");
    assert_eq!(w.rng.taken(), taken, "no standing-position draw");
    assert_eq!(
        w.obj(u).unwrap().dest,
        None,
        "the order-level destination keeps its preceding value"
    );
    for _ in 0..40 {
        step(&mut w);
    }
    assert_eq!(phase(&w, u), Phase::Grounded);
    assert_eq!(height(&w, u), 0);
}

// spec: S-0502 claim 23 (revision 6): the filter concerns the imported jumpjet flag, not merely the locomotor: an infantry type with the jumpjet locomotor and no flag accepts the order, and its search does not lift its own occupancy, so the movement point is in another cell
#[test]
fn claim_23_the_filter_reads_the_type_flag() {
    let mut w = game();
    let u = put(&mut w, "JJLOCO", Kind::Infantry, 0, 0, 0);
    go(&mut w, u, at(0, 0));
    assert!(j(&w, u).moving, "accepted");
    let d = j(&w, u).dest.expect("a movement point");
    assert_ne!(d.cell(), at(0, 0), "the unit's own mark rejects its cell");
}

// spec: S-0502 claim 23 (revision 6, answers Q-1011): claim 16's search does not lift a grounded vehicle's own occupancy, so an ordinary vehicle ordered to its own cell gets a nearby cell, not an own-cell take-off
#[test]
fn claim_23_vehicle_own_mark_excludes_its_own_cell() {
    let mut w = game();
    let u = put(&mut w, "JV", Kind::Vehicle, 0, 0, 0);
    go(&mut w, u, at(0, 0));
    assert!(j(&w, u).moving);
    let d = j(&w, u).dest.expect("a movement point");
    assert_ne!(d.cell(), at(0, 0));
    assert!(w.obj(u).unwrap().dest.is_some());
}

// spec: S-0502 claim 7 and claim 16 (revision 6): an entering vehicle keeps the exact ordered point, so an accepted point can lie in the ascent cell; it then leaves the ground and cruises early
#[test]
fn claim_7_entering_vehicle_keeps_an_own_cell_point() {
    let mut w = game();
    let u = put(&mut w, "JV", Kind::Vehicle, 0, 0, 0);
    go_here_entering(&mut w, u);
    assert_eq!(j(&w, u).dest.map(|d| d.cell()), Some(at(0, 0)));
    assert!(until(&mut w, 200, |w| phase(w, u) == Phase::Cruising));
}

// spec: S-0502 claim 24 (revision 6, answers Q-1012): an effective order-level destination clear from behaviour processing reroutes a committed jumpjet, whatever its source, and leaves the destination empty with a replacement point and the commitment; a repeated empty assignment of an ordinary vehicle is suppressed, with no reroute, no draw and the point kept
#[test]
fn claim_24_effective_clear_then_suppressed_assignment() {
    let mut w = game();
    let u = put(&mut w, "JV", Kind::Vehicle, 0, -4, 0);
    go(&mut w, u, at(8, 0));
    jump::start_in_flight(&mut w, u, Phase::Cruising, 500);
    assign_destination(&mut w, u, None);
    assert!(w.obj(u).unwrap().dest.is_none());
    assert!(j(&w, u).moving, "the commitment survives");
    assert_eq!(
        j(&w, u).dest.map(|d| d.cell()),
        Some(pos(&w, u).cell()),
        "a replacement point in the current cell"
    );
    let kept = j(&w, u).dest;
    let taken = w.rng.taken();
    // Move the unit so that a second reroute would pick a different point.
    let p = pos(&w, u);
    w.obj_mut(u).unwrap().pos = Pos::new(p.x + 256 * 3, p.y, p.z);
    assign_destination(&mut w, u, None);
    assert_eq!(j(&w, u).dest, kept, "the preceding point stays");
    assert_eq!(w.rng.taken(), taken);
    assert!(j(&w, u).moving);
}

// spec: S-0502 claim 24 (revision 6): without a movement commitment an effective clear does nothing (claim 17)
#[test]
fn claim_24_clear_without_a_commitment_does_nothing() {
    let mut w = game();
    let u = put(&mut w, "JV", Kind::Vehicle, 0, 0, 0);
    jump::start_in_flight(&mut w, u, Phase::Hovering, 500);
    w.obj_mut(u).unwrap().dest = Some(Target::Cell(at(3, 0)));
    js(&mut w, u).order = Some(Target::Cell(at(3, 0)));
    let mut before = j(&w, u).clone();
    before.order = None; // the locomotor forgets the cleared order; nothing else changes
    assign_destination(&mut w, u, None);
    assert_eq!(j(&w, u), &before);
}
