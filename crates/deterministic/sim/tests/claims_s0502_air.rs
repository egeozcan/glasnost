//! Claim tests of S-0502 (movement of jumpjets): height control, wobble,
//! ground reference, the step and facing, the cruise bands and arrival.

mod jump_common;

use glasnost_num::{F64, Rounding};
use glasnost_sim::jump::{self, JumpState, Phase};
use glasnost_sim::rules::Kind;
use glasnost_sim::testkit;
use glasnost_sim::turning::Turning;
use glasnost_sim::world::{ObjectId, Pos, Target, World};
use jump_common::*;

fn j(w: &World, id: ObjectId) -> &JumpState {
    w.obj(id).unwrap().jump.as_ref().unwrap()
}

fn js(w: &mut World, id: ObjectId) -> &mut JumpState {
    w.obj_mut(id).unwrap().jump.as_mut().unwrap()
}

fn mode(w: &World) -> Rounding {
    w.rules().mode
}

fn fl(w: &World, v: i32) -> F64 {
    F64::from_i64(i64::from(v), mode(w))
}

fn quirk(w: &mut World, f: impl FnOnce(&mut glasnost_rules::quirks::Quirks)) {
    let mut rules = (*w.rules_arc()).clone();
    f(&mut rules.quirks);
    w.rules = Some(std::sync::Arc::new(rules));
}

/// An enemy vehicle far away, to be the target of a hovering jumpjet.
fn enemy(w: &mut World) -> ObjectId {
    put(w, "CART", Kind::Vehicle, 1, 18, 18)
}

/// A hovering jumpjet holding a move order to its own cell and a target,
/// at `height`.
fn hoverer(w: &mut World, name: &str, kind: Kind, dx: i32, dy: i32, height: i32) -> ObjectId {
    let e = enemy(w);
    let u = put(w, name, kind, 0, dx, dy);
    go(w, u, at(dx, dy));
    w.obj_mut(u).unwrap().target = Some(Target::Object(e));
    jump::start_in_flight(w, u, Phase::Hovering, height);
    commit_here(w, u);
    u
}

/// A jumpjet cruising east at `height` with the maximum speed S, heading
/// east, the destination `cells` cells ahead.
fn cruiser(w: &mut World, name: &str, kind: Kind, cells: i32, height: i32) -> ObjectId {
    let u = put(w, name, kind, 0, -4, 0);
    go(w, u, at(-4 + cells, 0));
    jump::start_in_flight(w, u, Phase::Cruising, height);
    u
}

/// Makes the destination point the exact centre of cell `c` (the order's
/// nearby-cell search may pick another cell of the ring: S-0212 claim 25's
/// position N mod n).
fn aim(w: &mut World, id: ObjectId, c: glasnost_map::CellXY) -> Pos {
    let d = Pos::centre(c, 0);
    js(w, id).dest = Some(d);
    d
}

fn to_pos(w: &mut World, id: ObjectId, p: Pos) {
    w.obj_mut(id).unwrap().pos = p;
}

/// Moves a unit horizontally to (x, y), keeping its height.
fn place_xy(w: &mut World, id: ObjectId, x: i32, y: i32) {
    let z = pos(w, id).z;
    to_pos(w, id, Pos::new(x, y, z));
}

// spec: S-0502 claim 9, F2 and scenario J5 (a hovering or cruising jumpjet's height oscillates around its target height by the table sine of a phase that advances by 2π / (15 / W) per tick; in every other state, and with the no-wobble flag, the phase is 0)
#[test]
fn claim_9_f2_wobble() {
    let mut w = game();
    let u = hoverer(&mut w, "JJ", Kind::Infantry, 0, 0, 500);
    let m = mode(&w);
    let two_pi = glasnost_num::decimal::nearest_double("6.283185307179586");
    let wob = jump::fields(&w, u).unwrap().wobbles;
    let step = two_pi.div(fl(&w, 15).div(wob, m), m);
    let mut phi = F64::ZERO;
    for _ in 0..40 {
        step_world(&mut w);
        phi = phi.add(step, m);
        assert_eq!(j(&w, u).phi, phi);
        let hd = glasnost_num::trig::sin_t(phi, m)
            .mul(fl(&w, 40), m)
            .add(fl(&w, 500), m)
            .to_i32_common();
        // The height follows the desired height within one climb step of 5.
        assert!(
            (height(&w, u) - hd).abs() <= 5,
            "tick {}: {} against {hd}",
            w.tick,
            height(&w, u)
        );
    }
    assert!(j(&w, u).phi.gt(F64::ZERO));
    // Other states have no wobble phase.
    let a = put(&mut w, "JJ", Kind::Infantry, 0, 6, 0);
    go(&mut w, a, at(10, 0));
    for _ in 0..20 {
        step_world(&mut w);
    }
    assert_eq!(j(&w, a).phase, Phase::Ascending);
    assert_eq!(j(&w, a).phi, F64::ZERO);
    // The no-wobble flag suppresses the oscillation: the hover stays at 500.
    let f = hoverer(&mut w, "JJFLAT", Kind::Infantry, -6, 0, 500);
    for _ in 0..30 {
        step_world(&mut w);
        assert_eq!(height(&w, f), 500);
        assert_eq!(j(&w, f).phi, F64::ZERO);
    }
}

fn step_world(w: &mut World) {
    step(w);
}

// spec: S-0502 claim 9, F3 and scenario J5 (the ground reference in the look-ahead: while the speed is above 0 the cell ahead in the 8-direction of the heading counts; a building ahead gives 104 leptons per level of its art height, a unit 85, and a large rise stops the sideways motion (claim 8))
#[test]
fn claim_9_look_ahead_clearance() {
    // A jumpjet with no wobble cruising east at 500 over flat ground, speed 14, heading east.
    let prep = |ahead: Option<(&str, Kind)>| -> (World, ObjectId) {
        let mut w = game();
        let u = cruiser(&mut w, "JJFLAT", Kind::Infantry, 12, 500);
        js(&mut w, u).vc = fl(&w, 14);
        js(&mut w, u).vm = fl(&w, 14);
        let c = pos(&w, u).cell();
        let y = pos(&w, u).y;
        place_xy(&mut w, u, 256 * c.x + 128, y);
        if let Some((n, k)) = ahead {
            put(&mut w, n, k, 1, -4 + 1, 0);
        }
        (w, u)
    };
    // No obstacle: the height is held.
    let (mut w, u) = prep(None);
    let z0 = pos(&w, u).z;
    step_world(&mut w);
    assert_eq!(pos(&w, u).z, z0);
    assert!(j(&w, u).vc.gt(F64::ZERO));
    // A tall building ahead: reference 312, the height above it 188 is under half the desired height: the speed is held at 0 and the jumpjet climbs.
    let (mut w, u) = prep(Some(("TALL", Kind::Building)));
    step_world(&mut w);
    assert_eq!(pos(&w, u).z, 505, "it climbs by c = 5");
    assert_eq!(j(&w, u).vc, F64::ZERO, "ground hold");
    // A low building ahead (art height 1: 104 leptons): it climbs, the speed is kept.
    let (mut w, u) = prep(Some(("LOW", Kind::Building)));
    step_world(&mut w);
    assert_eq!(pos(&w, u).z, 505);
    assert!(j(&w, u).vc.gt(F64::ZERO));
    // A vehicle ahead: 85 leptons.
    let (mut w, u) = prep(Some(("CART", Kind::Vehicle)));
    step_world(&mut w);
    assert_eq!(pos(&w, u).z, 505);
    // Snap: with the reference at 85 the height 585 − 2 snaps to g + h_d exactly.
    let (mut w, u) = prep(Some(("CART", Kind::Vehicle)));
    let p = pos(&w, u);
    to_pos(&mut w, u, Pos::new(p.x, p.y, 85 + 500 - 2));
    step_world(&mut w);
    assert_eq!(pos(&w, u).z, 585);
}

// spec: S-0502 claim 9 and F3 (only infantry, vehicles and aircraft listed in the cell's ground layer give 85 leptons: a building gives its art height, a unit standing only on the bridge deck gives nothing, and a unit's own listing is not counted)
#[test]
fn claim_9_allowance_exclusions() {
    let mut w = game();
    let u = cruiser(&mut w, "JJFLAT", Kind::Infantry, 12, 500);
    js(&mut w, u).vc = fl(&w, 14);
    js(&mut w, u).vm = fl(&w, 14);
    let c = pos(&w, u).cell();
    let ahead = glasnost_map::CellXY::new(c.x + 1, c.y);
    // A unit listed only on the deck layer of the cell ahead supplies nothing.
    let v = put(&mut w, "CART", Kind::Vehicle, 1, -3, 0);
    w.cells.leave_both(ahead, v);
    w.cells
        .enter(ahead, glasnost_sim::world::Layer::Deck, v, false);
    let z0 = pos(&w, u).z;
    step_world(&mut w);
    assert_eq!(pos(&w, u).z, z0, "no allowance for a deck-only object");
}

fn deck_cell(w: &mut World, c: glasnost_map::CellXY) {
    let i = w.cells.idx(c).unwrap();
    w.cells.deck[i] = true;
}

// spec: S-0502 F3 and quirks.jumpjet_lookahead_bridge (approaching a bridge on level empty terrain: the reference rises by 208 leptons with the quirk on, 416 with it off) and scenario J5
#[test]
fn claim_9_f3_lookahead_bridge_quirk() {
    for (on, want) in [(true, 708), (false, 711)] {
        let mut w = game();
        quirk(&mut w, |q| q.jumpjet_lookahead_bridge = on);
        let u = cruiser(&mut w, "JJFLAT", Kind::Infantry, 12, 500);
        js(&mut w, u).vc = fl(&w, 14);
        js(&mut w, u).vm = fl(&w, 14);
        let c = pos(&w, u).cell();
        deck_cell(&mut w, glasnost_map::CellXY::new(c.x + 1, c.y));
        let p = pos(&w, u);
        // z = 208 + 500 − 2: with g = 208 it snaps to 708; with g = 416 it only climbs by c.
        to_pos(&mut w, u, Pos::new(256 * c.x + 128, p.y, 706));
        step_world(&mut w);
        assert_eq!(pos(&w, u).z, want, "quirk {on}");
    }
}

// spec: S-0502 F3 and claim 9 (the direct reference over a high-bridge cell: the deck, 416 leptons above the terrain, once the world height is at least the deck's height less k = 6; otherwise the terrain)
#[test]
fn claim_9_direct_reference_over_a_deck() {
    let mut w = game();
    let u = hoverer(&mut w, "JJFLAT", Kind::Infantry, 0, 0, 500);
    deck_cell(&mut w, at(0, 0));
    // From well above, the hover settles on the deck's reference: 416 + 500.
    let p = pos(&w, u);
    to_pos(&mut w, u, Pos::new(p.x, p.y, 1200));
    for _ in 0..200 {
        step_world(&mut w);
    }
    assert_eq!(pos(&w, u).z, 416 + 500);
}

// spec: S-0502 claim 10, F5 and scenario J6 (each physics step moves trunc(v_c) leptons along the heading with the shifted angle and table trigonometry: north (0, −10), east (+10, 0), south (−1, +10), west (−10, −1), north-east (+7, −8), south-east (+7, +7) for v = 10; v = 1 east moves one lepton)
#[test]
fn claim_10_f5_step_table() {
    let w = game();
    let m = mode(&w);
    let p = Pos::new(5000, 5000, 0);
    for (heading, dx, dy) in [
        (0u16, 0, -10),
        (16384, 10, 0),
        (32768, -1, 10),
        (49152, -10, -1),
        (8192, 7, -8),
        (24576, 7, 7),
    ] {
        let (x, y) = jump::step_position(&w, p, heading, 10, m);
        assert_eq!((x - p.x, y - p.y), (dx, dy), "heading {heading}");
    }
    let (x, y) = jump::step_position(&w, p, 16384, 1, m);
    assert_eq!((x - p.x, y - p.y), (1, 0));
}

// spec: S-0502 Edge cases and quirks.jumpjet_step_truncation (off: x′ = x + round(v sin α), y′ = y − round(v cos α) with exact functions, rounding half away from zero: no drift)
#[test]
fn claim_10_step_truncation_off() {
    let mut w = game();
    quirk(&mut w, |q| q.jumpjet_step_truncation = false);
    let m = mode(&w);
    let p = Pos::new(5000, 5000, 0);
    for (heading, dx, dy) in [
        (0u16, 0, -10),
        (16384, 10, 0),
        (32768, 0, 10),
        (49152, -10, 0),
        (8192, 7, -7),
        (24576, 7, 7),
        (4096, 4, -9),
    ] {
        let (x, y) = jump::step_position(&w, p, heading, 10, m);
        assert_eq!((x - p.x, y - p.y), (dx, dy), "heading {heading}");
    }
    // v sin α = 0.5 exactly would round away from zero; here 1 × sin(30°) is 0.5 − ε: 0, and 1 × cos is 1.
    let (x, y) = jump::step_position(&w, p, 5461, 1, m);
    assert_eq!((x - p.x, y - p.y), (0, -1));
}

// spec: S-0502 claim 10 (the body facing is set to the movement heading in each physics step: it follows the heading exactly and has no turn rate of its own)
#[test]
fn claim_10_body_facing_follows_the_heading() {
    let mut w = game();
    let u = cruiser(&mut w, "JJ", Kind::Infantry, 12, 500);
    js(&mut w, u).heading = Turning::at(40000);
    for _ in 0..6 {
        let heading = j(&w, u).heading.value(w.tick);
        step_world(&mut w);
        assert_eq!(w.obj(u).unwrap().body.value(w.tick), heading);
    }
}

// spec: S-0502 claim 10 and scenario J6 (a hovering infantry jumpjet with the jumpjet-turn flag and a target sets its movement heading at once to the direction of the target; its body facing is not set from the heading in that step; a type without the flag keeps copying the heading)
#[test]
fn claim_10_jumpjet_turn_exception() {
    let mut w = game();
    let t = put(&mut w, "CART", Kind::Vehicle, 1, 0, 8); // south of the jumpjets
    let a = put(&mut w, "JJTURN", Kind::Infantry, 0, 0, 0);
    let b = put(&mut w, "JJ", Kind::Infantry, 0, 4, 0);
    for u in [a, b] {
        let cell = w.obj(u).unwrap().pos.cell();
        go(&mut w, u, cell);
        w.obj_mut(u).unwrap().target = Some(Target::Object(t));
        jump::start_in_flight(&mut w, u, Phase::Hovering, 500);
        // An infantry order to its own cell takes a side standing position (S-0502 claim 23): the hover keeps the movement point at its position.
        let at_p = w.obj(u).unwrap().pos;
        w.obj_mut(u).unwrap().jump.as_mut().unwrap().dest = Some(at_p);
        // The own-cell order is not accepted (claim 23); the commitment stands as after a flight.
        w.obj_mut(u).unwrap().jump.as_mut().unwrap().moving = true;
        w.obj_mut(u).unwrap().body = Turning::at(0);
    }
    step_world(&mut w);
    // South, by the table direction (S-0204 claim 8).
    let (pa, pt) = (pos(&w, a), pos(&w, t));
    let want = glasnost_num::geom::direction(i64::from(pt.x - pa.x), i64::from(pt.y - pa.y), true);
    assert_eq!(j(&w, a).heading.value(w.tick), want);
    assert_eq!(
        w.obj(a).unwrap().body.value(w.tick),
        0,
        "the body is not set"
    );
    // The other copies its heading (east, the new jumpjet's).
    assert_eq!(w.obj(b).unwrap().body.value(w.tick), 16384);
}

/// The speed and target height after one cruise outcome at the distance
/// `rho` leptons from the destination, heading aligned with it (or turned
/// by `turn`).
fn band(w: &mut World, name: &str, kind: Kind, rho: i32, target: bool) -> (F64, i32) {
    let u = put(w, name, kind, 0, -4, 0);
    let c = at(2, 0);
    go(w, u, c);
    jump::start_in_flight(w, u, Phase::Cruising, 500);
    // Place it rho leptons west of the destination point's centre, on its row.
    let d = aim(w, u, c);
    place_xy(w, u, d.x - rho, d.y);
    js(w, u).heading = Turning::at(16384);
    js(w, u).vc = F64::ZERO;
    if target {
        let e = enemy(w);
        let t = w.tick;
        let o = w.obj_mut(u).unwrap();
        o.target = Some(Target::Object(e));
        // The behaviour processing must not move the destination (an approach would).
        o.beh.force(glasnost_sim::rules::Behaviour::Move, t, true);
    }
    step(w);
    (j(w, u).vm, j(w, u).ht)
}

// spec: S-0502 claim 11, F6 and scenario J7 (the cruise bands by distance ρ: ρ < S: S / 8 at H / 2; ρ < 2S: S / 4 at H / 2; ρ < (50 S) div T: S / 2 at three quarters of H; otherwise S at H; the nearer bands change the target height only when there is no target)
#[test]
fn claim_11_f6_bands() {
    let mut w = game();
    // The first band (ρ < S) is reached only by a type whose speed is above the arrival radius.
    let (vm, h) = band(&mut w, "JJFAST", Kind::Infantry, 30, false);
    assert!(vm.eq_num(fl(&w, 5)) && h == 250, "S / 8 at H / 2");
    let (vm, h) = band(&mut w, "JJFAST", Kind::Infantry, 30, true);
    assert!(
        vm.eq_num(fl(&w, 5)) && h == 500,
        "with a target the height is unchanged"
    );
    for (rho, speed, ht) in [
        (25, 3, 250),
        (100, 7, 375),
        (174, 7, 375),
        (176, 14, 500),
        (900, 14, 500),
    ] {
        let (vm, h) = band(&mut w, "JJ", Kind::Infantry, rho, false);
        assert!(vm.eq_num(fl(&w, speed)), "rho {rho}");
        assert_eq!(h, ht, "rho {rho}");
    }
    // With a target the nearer bands leave the target height alone; the far band sets H.
    for (rho, speed, ht) in [(25, 3, 500), (100, 7, 500), (900, 14, 500)] {
        let (vm, h) = band(&mut w, "JJ", Kind::Infantry, rho, true);
        assert!(vm.eq_num(fl(&w, speed)), "target, rho {rho}");
        assert_eq!(h, ht);
    }
}

// spec: S-0502 claim 11 and F6 (a large remaining turn slows the second and third bands: max(S / 10, 1.0) when D > T, max(S / 5, 1.0) when D > 5T)
#[test]
fn claim_11_large_turn_slows() {
    let mut w = game();
    let slow = |w: &mut World, rho: i32| -> F64 {
        let u = put(w, "JJ", Kind::Infantry, 0, -4, 0);
        let c = at(2, 0);
        go(w, u, c);
        jump::start_in_flight(w, u, Phase::Cruising, 500);
        let d = aim(w, u, c);
        // East of the destination: the destination lies west, the heading east: a half turn remains.
        place_xy(w, u, d.x + rho, d.y);
        js(w, u).heading = Turning::at(16384);
        js(w, u).vc = F64::ZERO;
        step(w);
        j(w, u).vm
    };
    assert!(slow(&mut w, 25).eq_num(fl(&w, 1)), "max(S / 10, 1.0)");
    assert!(slow(&mut w, 100).eq_num(fl(&w, 2)), "max(S / 5, 1.0)");
}

// spec: S-0502 F6, Edge cases and quirks.jumpjet_turn_slowdown_one_sided (a small anticlockwise remaining turn reads as nearly 256 steps and slows the band; off: the smaller of the two differences is used)
#[test]
fn claim_11_one_sided_turn_slowdown() {
    for (on, want) in [(true, 1), (false, 3)] {
        let mut w = game();
        quirk(&mut w, |q| q.jumpjet_turn_slowdown_one_sided = on);
        let u = put(&mut w, "JJ", Kind::Infantry, 0, -4, 0);
        let c = at(2, 0);
        go(&mut w, u, c);
        jump::start_in_flight(&mut w, u, Phase::Cruising, 500);
        let d = aim(&mut w, u, c);
        place_xy(&mut w, u, d.x - 25, d.y);
        // The heading is 1024 facing units clockwise of east: an anticlockwise turn of 1024 remains.
        js(&mut w, u).heading = Turning::at(16384 + 1024);
        js(&mut w, u).vc = F64::ZERO;
        step_world(&mut w);
        assert!(j(&w, u).vm.eq_num(fl(&w, want)), "quirk {on}");
    }
}

// spec: S-0502 claim 11 (full height kept: a balloon-hover type, a destination cell of water or beach, a deploy-to-land vehicle keep target height H through the nearer bands)
#[test]
fn claim_11_full_height_kept() {
    let mut w = game();
    for (name, kind) in [("BALLOON", Kind::Vehicle), ("DEPL", Kind::Vehicle)] {
        let (_, h) = band(&mut w, name, kind, 25, false);
        assert_eq!(h, 500, "{name}");
    }
    let mut w = game();
    // A destination on beach.
    let c = at(2, 0);
    w.cells.grid.get_mut(c).unwrap().land = glasnost_map::terrain::land::BEACH;
    let (_, h) = band(&mut w, "JJ", Kind::Infantry, 25, false);
    assert_eq!(h, 500, "beach");
}

// spec: S-0502 claim 12 and scenario J7 (arrival below 20 leptons: the speeds become 0 and the position moves exactly onto the destination point; the outcome by the table: no target and not hovering-type: descending; a target, a balloon-hover type or a deploy-to-land vehicle: hovering)
#[test]
fn claim_12_arrival() {
    for (name, kind, target, want) in [
        ("JJ", Kind::Infantry, false, Phase::Descending),
        ("JJ", Kind::Infantry, true, Phase::Hovering),
        ("BALLOON", Kind::Vehicle, false, Phase::Hovering),
        ("DEPL", Kind::Vehicle, false, Phase::Hovering),
        ("DEPN", Kind::Vehicle, false, Phase::Descending),
    ] {
        let mut w = game();
        let u = put(&mut w, name, kind, 0, -4, 0);
        let c = at(2, 0);
        go(&mut w, u, c);
        jump::start_in_flight(&mut w, u, Phase::Cruising, 500);
        let d = j(&w, u).dest.unwrap();
        place_xy(&mut w, u, d.x - 15, d.y + 4);
        js(&mut w, u).vc = fl(&w, 14);
        if target {
            let e = enemy(&mut w);
            w.obj_mut(u).unwrap().target = Some(Target::Object(e));
        }
        step_world(&mut w);
        assert_eq!(j(&w, u).phase, want, "{name}, target {target}");
        let p = pos(&w, u);
        assert_eq!((p.x, p.y), (d.x, d.y), "snapped onto the destination point");
        assert_eq!((j(&w, u).vc, j(&w, u).vm), (F64::ZERO, F64::ZERO));
    }
}

#[allow(dead_code)]
fn unused(w: &mut World) {
    testkit::run(w, 0);
}
