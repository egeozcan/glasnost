//! Claim tests for the teleporting locomotor's own move (S-0207 claim 20,
//! F9) on synthetic data: invented vehicle types and rules values on the
//! flat scene of `common`.

mod common;

use common::*;
use glasnost_map::CellXY;
use glasnost_num::geom::root_floor;
use glasnost_rules::sim::{Locomotor, UnitType};
use glasnost_sim::rules::Kind;
use glasnost_sim::teleport;
use glasnost_sim::world::{Layer, ObjectId, Pos, Target, World};

/// A vehicle type with the teleporting locomotor; `harvester` sets the
/// harvester flag.
fn jumper(harvester: bool) -> UnitType {
    let mut t = unit_type(WHEEL, 6, 5);
    t.locomotor = Some(Locomotor::Teleport);
    t.harvester = Some(harvester);
    t
}

/// A scene with JUMPER placed at the middle cell facing map east.
fn scene(harvester: bool) -> Scene {
    let mut s = Scene::new();
    s.vehicle("JUMPER", jumper(harvester));
    s.place_unit(0, "JUMPER", cell(0, 0), 64);
    s
}

fn start(s: &Scene) -> (World, ObjectId) {
    let mut w = s.build();
    open_orders(&mut w);
    let id = objects_of(&w, Kind::Vehicle)[0];
    (w, id)
}

fn centre(w: &World, c: CellXY) -> Pos {
    let p = Pos::centre(c, 0);
    Pos::centre(c, w.surface_at(p.x, p.y))
}

fn marked(w: &World, c: CellXY) -> bool {
    w.cells
        .occ(c, Layer::Ground)
        .is_some_and(|o| o.vehicle_present())
}

/// F9 with the rules' values, computed here from the formula.
fn f9(w: &World, from: Pos, to: Pos) -> i64 {
    let g = &w.rules().general;
    let (dx, dy, dz) = (
        i64::from(to.x - from.x),
        i64::from(to.y - from.y),
        i64::from(to.z - from.z),
    );
    let d = root_floor(
        (dx * dx + dy * dy + dz * dz) as u64,
        w.rules().quirks.table_root_distance,
        w.rules().mode,
    ) as i64;
    let k = i64::from(g.chrono_distance_factor);
    let q = if g.chrono_trigger && k != 0 { d / k } else { 0 };
    let m = i64::from(g.chrono_minimum_delay);
    if d < i64::from(g.chrono_range_minimum) {
        m
    } else if q.max(0) > m {
        q
    } else {
        m
    }
}

// spec: S-0207 claim 20 (a) (the landing point fixed and the occupancy mark moved when the order applies; the object stays listed in its old cell until the jump)
// spec: S-0207 claim 20 (d) and (e) (the jump on the next movement update: no turn, the centre of the destination cell at H, listed there, the facing kept, the destination removed)
// spec: S-0207 claim 20 (f) (phasing in from the jump)
#[test]
fn claim_20_jump_timing_and_effects() {
    let s = scene(false);
    let (mut w, id) = start(&s);
    let from = w.obj(id).unwrap().pos;
    let f0 = w.obj(id).unwrap().body.value(w.tick);
    let dc = cell(0, -15);
    order_move(&mut w, id, dc);
    run(&mut w, 1);
    // The order applied after the turn pass: the mark moved, the object did not.
    let o = w.obj(id).unwrap();
    assert_eq!(o.pos, from);
    assert_eq!(o.dest, Some(Target::Cell(dc)));
    assert!(marked(&w, dc) && !marked(&w, cell(0, 0)));
    assert!(w.cells.list(cell(0, 0), Layer::Ground).contains(&id));
    assert!(glasnost_sim::occupancy::counts_as_moving(&w, id));
    run(&mut w, 1);
    let t0 = w.tick - 1;
    let o = w.obj(id).unwrap();
    assert_eq!(o.pos, centre(&w, dc));
    assert_eq!(o.layer, Layer::Ground);
    assert_eq!(o.body.value(w.tick), f0);
    assert_eq!(o.dest, None);
    assert!(w.cells.list(dc, Layer::Ground).contains(&id));
    assert!(!w.cells.list(cell(0, 0), Layer::Ground).contains(&id));
    assert!(marked(&w, dc) && !marked(&w, cell(0, 0)));
    assert!(teleport::phasing_in(&w, id));
    // spec: S-0207 F9 (D from the distance before the jump; k = 32 and m = g = 0 when no layer gives them)
    let d = f9(&w, from, centre(&w, dc));
    assert_eq!(d, (15 * 256) / 32);
    assert_eq!(teleport::delay(&w, from, centre(&w, dc)), d);
    while w.tick < t0 + d as i32 {
        assert!(teleport::phasing_in(&w, id), "tick {}", w.tick);
        run(&mut w, 1);
    }
    // The turn of tick t₀ + D ends it.
    assert!(teleport::phasing_in(&w, id));
    run(&mut w, 1);
    assert!(!teleport::phasing_in(&w, id));
}

// spec: S-0207 claim 20 (f) and (g) (a harvester-flag type does not phase in: a further order is jumped to on the next movement update)
#[test]
fn claim_20_harvester_does_not_phase_in() {
    let s = scene(true);
    let (mut w, id) = start(&s);
    order_move(&mut w, id, cell(5, 0));
    run(&mut w, 2);
    assert_eq!(w.obj(id).unwrap().pos, centre(&w, cell(5, 0)));
    assert!(!teleport::phasing_in(&w, id));
    order_move(&mut w, id, cell(-5, 3));
    run(&mut w, 2);
    assert_eq!(w.obj(id).unwrap().pos, centre(&w, cell(-5, 3)));
}

// spec: S-0207 claim 20 (a) and (f) (a destination given while phasing in is dropped: lost, not delayed)
#[test]
fn claim_20_orders_while_phasing_in_are_lost() {
    let s = scene(false);
    let (mut w, id) = start(&s);
    order_move(&mut w, id, cell(0, -15));
    run(&mut w, 3);
    assert!(teleport::phasing_in(&w, id));
    let at = w.obj(id).unwrap().pos;
    order_move(&mut w, id, cell(6, -15));
    run(&mut w, 1);
    assert_eq!(w.obj(id).unwrap().dest, None);
    run(&mut w, 200);
    assert!(!teleport::phasing_in(&w, id));
    assert_eq!(w.obj(id).unwrap().pos, at);
}

// spec: S-0207 claim 20 (b) (an occupied destination: the nearby-cell search's cell, not the destination cell)
#[test]
fn claim_20_occupied_destination() {
    let mut s = scene(false);
    s.vehicle("BLOCK", unit_type(WHEEL, 6, 5));
    let dc = cell(0, -8);
    s.place_unit(0, "BLOCK", dc, 0);
    let (mut w, id) = start(&s);
    order_move(&mut w, id, dc);
    run(&mut w, 2);
    let c = w.obj(id).unwrap().pos.cell();
    assert_ne!(c, dc);
    assert_eq!(w.obj(id).unwrap().pos, centre(&w, c));
    assert!((c.x - dc.x).abs() <= 1 && (c.y - dc.y).abs() <= 1);
}

// spec: S-0207 claim 20 (b) (the nearby-cell pick uses the landing-point fixation tick: the point fixed when the order applies is where the jump lands, a tick later)
#[test]
fn claim_20_landing_pick_at_fixation() {
    let mut s = scene(false);
    s.vehicle("BLOCK", unit_type(WHEEL, 6, 5));
    let dc = cell(0, -8);
    s.place_unit(0, "BLOCK", dc, 0);
    let (mut w, id) = start(&s);
    order_move(&mut w, id, dc);
    run(&mut w, 1);
    let fixed = w
        .obj(id)
        .unwrap()
        .teleport
        .as_ref()
        .unwrap()
        .landing
        .expect("fixed when the order applies");
    let fixation = w.tick;
    run(&mut w, 1);
    assert!(w.tick > fixation);
    assert_eq!(w.obj(id).unwrap().pos, fixed);
}

// spec: S-0207 claim 20 (d) (L equal to its own position: no jump, the destination removed, no phasing in)
#[test]
fn claim_20_own_cell_destination() {
    let s = scene(false);
    let (mut w, id) = start(&s);
    let p = w.obj(id).unwrap().pos;
    assert_eq!(p, centre(&w, cell(0, 0)));
    order_move(&mut w, id, cell(0, 0));
    run(&mut w, 2);
    let o = w.obj(id).unwrap();
    assert_eq!((o.pos, o.dest), (p, None));
    assert!(!teleport::phasing_in(&w, id));
}

// spec: S-0207 claim 20 (e) (every object that has it as its target drops it at the jump)
#[test]
fn claim_20_targets_drop_the_jumper() {
    let mut s = scene(false);
    s.vehicle("WATCHER", unit_type(WHEEL, 6, 5));
    s.place_unit(1, "WATCHER", cell(5, 5), 0);
    let (mut w, id) = start(&s);
    let watcher = objects_of(&w, Kind::Vehicle)[1];
    w.obj_mut(watcher).unwrap().target = Some(Target::Object(id));
    order_move(&mut w, id, cell(0, -10));
    run(&mut w, 1);
    assert_eq!(w.obj(watcher).unwrap().target, Some(Target::Object(id)));
    run(&mut w, 1);
    assert_eq!(w.obj(watcher).unwrap().target, None);
}

// spec: S-0215 claim 10 (the target's own jump: an overridden attacker reverts, keeps a restored other target, and drops the jumper when the revert brings it back)
#[test]
fn claim_20_jump_reverts_overridden_attackers() {
    use glasnost_sim::rules::Behaviour;
    let mut s = scene(false);
    s.vehicle("WATCHER", unit_type(WHEEL, 6, 5));
    s.place_unit(1, "WATCHER", cell(5, 5), 0);
    s.place_unit(1, "WATCHER", cell(6, 5), 0);
    s.place_unit(1, "WATCHER", cell(7, 5), 0);
    let (mut w, id) = start(&s);
    let v = objects_of(&w, Kind::Vehicle);
    let (keeps, loses, other) = (v[1], v[2], v[3]);
    for (x, aside) in [(keeps, other), (loses, id)] {
        let o = w.obj_mut(x).unwrap();
        o.target = Some(Target::Object(id));
        o.beh.current = Some(Behaviour::Attack);
        o.beh.suspended = Some(Behaviour::Guard);
        o.beh.aside = Some((Some(Target::Object(aside)), None));
    }
    order_move(&mut w, id, cell(0, -10));
    run(&mut w, 2);
    let k = w.obj(keeps).unwrap();
    assert_eq!(k.beh.current, Some(Behaviour::Guard));
    assert_eq!(k.beh.suspended, None);
    assert_eq!(k.target, Some(Target::Object(other)));
    let l = w.obj(loses).unwrap();
    assert_eq!(l.beh.suspended, None);
    assert_eq!(l.target, None, "the restored jumper is dropped");
}

// spec: S-0207 claim 20 (f) and (i) (at the end of phasing in, with no target and more than 10 ticks on the targeting timer, one ranged draw over 4 to 8 ticks; the jump itself takes none)
#[test]
fn claim_20_draw_at_the_end_of_phasing_in() {
    let s = scene(false);
    // The jump tick takes the draws of the same tick without a jump (an order to the own cell): the move behaviour's own.
    let taken_in_order_tick = |to: CellXY| {
        let (mut w, id) = start(&s);
        order_move(&mut w, id, to);
        run(&mut w, 1);
        let n = w.rng.taken();
        run(&mut w, 1);
        (w.rng.taken() - n, teleport::phasing_in(&w, id))
    };
    let (jump, phasing) = taken_in_order_tick(cell(0, -4));
    let (stay, control) = taken_in_order_tick(cell(0, 0));
    assert!(phasing && !control);
    assert_eq!(jump, stay, "the jump takes no draw");
    // The end tick, with the targeting timer `left` ticks ahead of it: the
    // tick, the draw state before and after it, and the timer after it.
    let end_tick = |left: i32| {
        let (mut w, id) = start(&s);
        order_move(&mut w, id, cell(0, -4));
        run(&mut w, 2);
        assert!(teleport::phasing_in(&w, id));
        loop {
            let t = w.tick;
            w.obj_mut(id).unwrap().targeting_timer = t + left;
            let before = w.rng.clone();
            run(&mut w, 1);
            if !teleport::phasing_in(&w, id) {
                let timer = w.obj(id).unwrap().targeting_timer;
                return (
                    t,
                    before,
                    w.rng.clone(),
                    timer,
                    w.rules().quirks.ranged_draw_half_span,
                );
            }
        }
    };
    // 10 ticks left: no draw of its own, the timer kept.
    let (end, before, short, timer, half) = end_tick(10);
    assert_eq!(timer, end + 10);
    // More than 10 ticks left: the same tick plus one ranged draw over 4 to
    // 8, after the draws of that tick's processing, which sets the timer.
    let (end_long, before_long, long, timer) = {
        let (e, b, a, t, _) = end_tick(11);
        (e, b, a, t)
    };
    assert_eq!((end_long, &before_long), (end, &before));
    let mut r = before.clone();
    for _ in 0..short.taken() - before.taken() {
        r.raw();
    }
    let v = r.ranged(4, 8, half);
    assert_eq!(r, long);
    assert_eq!(timer, end + v);
}

// spec: S-0207 F9 (trigger off: D = m; d below the range minimum: D = m; D = q when max(q, 0) > m, so a negative m gives q) and Edge cases (k = 0 with the trigger on: q = 0)
#[test]
fn f9_phasing_in_delay() {
    let from = Pos::new(0, 0, 0);
    let to = Pos::new(256 * 20, 0, 0);
    let delay = |k: i32, trigger: bool, m: i32, g: i32| {
        let mut s = scene(false);
        let gl = &mut s.ruleset.sim.general;
        gl.chrono_distance_factor = Some(k);
        gl.chrono_trigger = Some(trigger);
        gl.chrono_minimum_delay = Some(m);
        gl.chrono_range_minimum = Some(g);
        let w = s.build();
        teleport::delay(&w, from, to)
    };
    // d = 5120.
    assert_eq!(delay(32, true, 0, 0), 160);
    assert_eq!(delay(32, true, 200, 0), 200);
    assert_eq!(delay(32, false, 7, 0), 7);
    assert_eq!(delay(32, true, 7, 6000), 7);
    assert_eq!(delay(-32, true, -500, 0), -160);
    assert_eq!(delay(0, true, 3, 0), 3);
}

// spec: S-0417 claim 5 (a teleporting vehicle temporarily receives driving locomotion: it takes no landing point and does not jump), S-0215 claim 13 row 1 (it gets its own locomotor back) and S-0207 claim 21 (destinations while driving fix no landing point and move no reservation; an empty suspended state stays empty and the first destination after restoration jumps)
#[test]
fn temporary_drive_then_own_locomotor_back() {
    let s = scene(false);
    let (mut w, id) = start(&s);
    // As the factory exit gives it (S-0417 claim 5).
    glasnost_sim::drive::release_claims(&mut w, id);
    {
        let o = w.obj_mut(id).unwrap();
        o.drive = Some(glasnost_sim::drive::DriveState::new());
        o.econ.temporary_drive = true;
    }
    let dc = cell(0, -15);
    order_move(&mut w, id, dc);
    run(&mut w, 2);
    let o = w.obj(id).unwrap();
    assert_ne!(o.pos, centre(&w, dc));
    assert_eq!(o.teleport.as_ref().and_then(|t| t.landing), None);
    assert!(!marked(&w, dc));
    // It gets its own locomotor back at a fall-back to rest, with no destination.
    {
        let o = w.obj_mut(id).unwrap();
        o.dest = None;
        o.path.steps.clear();
    }
    assert!(glasnost_sim::econ::release::restore_locomotor(&mut w, id));
    let here = w.obj(id).unwrap().pos.cell();
    assert!(marked(&w, here));
    assert_eq!(
        w.obj(id).unwrap().teleport,
        Some(teleport::TeleportState::default())
    );
    let to = cell(6, -3);
    order_move(&mut w, id, to);
    run(&mut w, 1);
    assert!(marked(&w, to) && !marked(&w, here));
    run(&mut w, 1);
    assert_eq!(w.obj(id).unwrap().pos, centre(&w, to));
}

// spec: S-0207 claim 21 (restoration retains a fixed landing point, a pending jump and a phasing-in state with its original expiry, and moves no occupancy mark)
#[test]
fn claim_21_restoration_retains_the_teleporting_state() {
    let s = scene(false);
    let (mut w, id) = start(&s);
    let here = w.obj(id).unwrap().pos.cell();
    let l = cell(6, -3);
    // A suspended teleporting state with a pending jump to L and a
    // phasing in from an earlier jump; its reservation is on L.
    glasnost_sim::drive::release_claims(&mut w, id);
    w.cells.claim_vehicle(l, Layer::Ground, true);
    let st = teleport::TeleportState {
        landing: Some(centre(&w, l)),
        for_dest: Some(Target::Cell(l)),
        mark: Some(l),
        phasing: Some((w.tick, 50)),
        mark_layer: Layer::Ground,
        ..Default::default()
    };
    {
        let o = w.obj_mut(id).unwrap();
        o.teleport = Some(st.clone());
        o.drive = Some(glasnost_sim::drive::DriveState::new());
        o.econ.temporary_drive = true;
    }
    assert!(glasnost_sim::econ::release::restore_locomotor(&mut w, id));
    let o = w.obj(id).unwrap();
    assert!(o.drive.is_none());
    assert_eq!(o.teleport.as_ref(), Some(&st));
    assert!(marked(&w, l));
    assert!(!marked(&w, here));
}

/// The end tick of phasing in of an armed teleporting cart whose
/// targeting timer stands `left` ticks ahead of each tick: the tick, the
/// draw state before and after it, the timer after it and the targeting
/// delay of its behaviour. `passive` is the type's passive-acquire flag (the
/// search gate of S-0309 claim 2; claim 3's opportunity gate is open).
fn armed_end_tick(
    passive: bool,
    left: i32,
) -> (
    i32,
    glasnost_sim::rng::Rng,
    glasnost_sim::rng::Rng,
    i32,
    i32,
) {
    use glasnost_sim::testkit;
    let mut r = testkit::combat_ruleset();
    let cart = r.sim.vehicles.get_mut("CART").unwrap();
    cart.primary = Some("Gun".into());
    cart.locomotor = Some(Locomotor::Teleport);
    cart.harvester = Some(false);
    cart.combat.can_passive_aquire = Some(passive);
    // The opportunity gate of S-0309 claim 3 opens for a type that fires on the move.
    cart.opportunity_fire = Some(true);
    let mut w = testkit::game_with(&r, 24, 3);
    let id = testkit::spawn(&mut w, Kind::Vehicle, "CART", 0, testkit::mid(24, 0, 0), 0);
    open_orders(&mut w);
    order_move(&mut w, id, testkit::mid(24, 0, -4));
    run(&mut w, 2);
    assert!(teleport::phasing_in(&w, id));
    loop {
        let t = w.tick;
        w.obj_mut(id).unwrap().targeting_timer = t + left;
        let before = w.rng.clone();
        run(&mut w, 1);
        if !teleport::phasing_in(&w, id) {
            assert_eq!(w.obj(id).unwrap().target, None);
            // spec: S-0309 claim 4 (the guard-area delay under area guard, the normal one otherwise); the jump left the cart at rest (claim 20 e)
            let g = &w.rules().general;
            let n = if w.obj(id).unwrap().beh.current
                == Some(glasnost_sim::rules::Behaviour::AreaGuard)
            {
                g.guard_area_targeting_delay
            } else {
                g.normal_targeting_delay
            };
            return (
                t,
                before,
                w.rng.clone(),
                w.obj(id).unwrap().targeting_timer,
                n,
            );
        }
    }
}

// spec: S-0207 claim 20 (f) and (i) (only with no target: above 10 ticks left the conditional draw over 4 to 8 comes first; a search the gates allow then takes its normal timer draw, which replaces the temporary value; with the gates refusing, only the conditional draw; at 10 or fewer, only the search's draw); S-0309 claim 4 (the search's timer restart N + U(0, 2))
#[test]
fn claim_20_end_of_phasing_in_draws_with_the_search_gates() {
    let half = glasnost_sim::testkit::combat_rules()
        .quirks
        .ranged_draw_half_span;
    // Neither draw: the gates refuse and the timer is at 10.
    let (end, before, none, timer, _) = armed_end_tick(false, 10);
    assert_eq!(timer, end + 10);
    let base = none.taken() - before.taken();
    let replay = |b: &glasnost_sim::rng::Rng| {
        let mut r = b.clone();
        for _ in 0..base {
            r.raw();
        }
        r
    };
    // Gates refusing, 11 ticks left: the conditional draw only.
    let (e, b, after, timer, _) = armed_end_tick(false, 11);
    assert_eq!((e, &b), (end, &before));
    let mut r = replay(&b);
    let v = r.ranged(4, 8, half);
    assert_eq!(r, after);
    assert_eq!(timer, end + v);
    // Gates passing, 10 ticks left: the search's draw only.
    let (e, b, after, timer, n) = armed_end_tick(true, 10);
    assert_eq!((e, &b), (end, &before));
    let mut r = replay(&b);
    let u = r.ranged(0, 2, half);
    assert_eq!(r, after);
    assert_eq!(timer, end + n + u);
    // Gates passing, 11 ticks left: the conditional draw, then the search's, whose value stands.
    let (e, b, after, timer, n) = armed_end_tick(true, 11);
    assert_eq!((e, &b), (end, &before));
    let mut r = replay(&b);
    let _ = r.ranged(4, 8, half);
    let u = r.ranged(0, 2, half);
    assert_eq!(r, after);
    assert_eq!(timer, end + n + u);
}

/// Makes `c` a high-bridge cell.
fn bridge(w: &mut World, c: CellXY) {
    let i = w.cells.idx(c).expect("cell");
    w.cells.deck[i] = true;
    w.cells.deck_entry[i] = true;
    w.cells.bridge_axis[i] = glasnost_sim::cells::Axis::Y;
}

/// The vehicle-present flags of `c`'s ground and deck records.
fn vehicle_flags(w: &World, c: CellXY) -> (bool, bool) {
    let f = |l| w.cells.occ(c, l).is_some_and(|o| o.vehicle_present());
    (f(Layer::Ground), f(Layer::Deck))
}

/// Runs until the object has finished phasing in.
fn settle(w: &mut World, id: ObjectId) {
    run_until(w, 400, |w| !teleport::phasing_in(w, id));
}

// spec: S-0207 claim 20 (a) and (e) (on a high-bridge cell a vehicle lands at H + 416 and is listed in the deck layer; its reservation is the ground record, kept through deck placement, which also marks the deck: both records are marked; a later accepted destination clears the remembered ground reservation and the old deck mark clears when the next jump removes it, Q-0240)
// spec: S-0209 claim 1 (teleporting vehicles: reservation layer need not match listing layer)
#[test]
fn claim_20_vehicle_landing_on_a_high_bridge() {
    let s = scene(false);
    let (mut w, id) = start(&s);
    let here = w.obj(id).unwrap().pos.cell();
    let dc = cell(0, -6);
    bridge(&mut w, dc);
    order_move(&mut w, id, dc);
    run(&mut w, 1);
    assert_eq!(vehicle_flags(&w, dc), (true, false), "ground reservation");
    assert_eq!(vehicle_flags(&w, here), (false, false));
    run(&mut w, 1);
    let o = w.obj(id).unwrap();
    let ground = centre(&w, dc);
    assert_eq!(o.pos, Pos::new(ground.x, ground.y, ground.z + 416));
    assert_eq!(o.layer, Layer::Deck);
    assert!(w.cells.list(dc, Layer::Deck).contains(&id));
    assert!(!w.cells.list(dc, Layer::Ground).contains(&id));
    assert_eq!(vehicle_flags(&w, dc), (true, true), "both records");
    settle(&mut w, id);
    let next = cell(6, 0);
    order_move(&mut w, id, next);
    run(&mut w, 1);
    assert_eq!(
        vehicle_flags(&w, dc),
        (false, true),
        "remembered ground cleared"
    );
    assert_eq!(vehicle_flags(&w, next), (true, false));
    run_until(&mut w, 10, |w| w.obj(id).unwrap().pos.cell() == next);
    assert_eq!(
        vehicle_flags(&w, dc),
        (false, false),
        "deck cleared at departure"
    );
    assert_eq!(
        vehicle_flags(&w, next),
        (true, false),
        "one record off a bridge"
    );
}

// spec: S-0207 Edge cases (quirks.teleport_vehicle_ground_reservation off: the reservation is in the eventual landing layer, deck on a high bridge, with no separate ground reservation, and is cleared from that layer)
#[test]
fn claim_20_vehicle_ground_reservation_switch_off() {
    let mut s = scene(false);
    s.ruleset.globals.quirks.teleport_vehicle_ground_reservation = false;
    let (mut w, id) = start(&s);
    let dc = cell(0, -6);
    bridge(&mut w, dc);
    order_move(&mut w, id, dc);
    run(&mut w, 1);
    assert_eq!(vehicle_flags(&w, dc), (false, true));
    run(&mut w, 1);
    assert_eq!(w.obj(id).unwrap().layer, Layer::Deck);
    assert_eq!(vehicle_flags(&w, dc), (false, true));
    settle(&mut w, id);
    order_move(&mut w, id, cell(6, 0));
    run(&mut w, 1);
    assert_eq!(vehicle_flags(&w, dc), (false, false));
}

/// Whether the object-present flag of cell `c`'s record in layer `l` is set.
fn present(w: &World, c: CellXY, l: Layer) -> bool {
    w.cells.occ(c, l).is_some_and(|o| o.others > 0)
}

/// An aircraft with the teleporting locomotor at the middle cell, the
/// aircraft reservation quirk set to `on`.
fn flash(on: bool) -> (World, ObjectId) {
    let mut s = Scene::new();
    let mut t = unit_type(WHEEL, 6, 5);
    t.locomotor = Some(Locomotor::Teleport);
    s.aircraft("FLASH", t);
    s.place_aircraft(0, "FLASH", cell(0, 0), 64);
    s.ruleset
        .globals
        .quirks
        .teleport_aircraft_ground_reservation = on;
    let mut w = s.build();
    open_orders(&mut w);
    let id = objects_of(&w, Kind::Aircraft)[0];
    (w, id)
}

/// The object-present flags of `c`'s ground and deck records.
fn present_flags(w: &World, c: CellXY) -> (bool, bool) {
    (present(w, c, Layer::Ground), present(w, c, Layer::Deck))
}

// spec: S-0207 claim 20 (an aircraft with the teleporting locomotor moves by it) and (a) (its reservation is the object-present flag of the landing cell's ground record) and (e) (it lands at H or, on a high-bridge cell, H + 416; deck placement also sets the deck flag, keeping the ground reservation, Q-0310)
// spec: S-0209 claim 1 (teleporting aircraft: both records marked, only the deck lists the aircraft; removal on the deck clears only the deck)
#[test]
fn claim_20_aircraft_moves_its_object_present_mark() {
    for high in [false, true] {
        let (mut w, id) = flash(true);
        let here = w.obj(id).unwrap().pos.cell();
        let dc = cell(0, -6);
        if high {
            bridge(&mut w, dc);
        }
        assert!(present(&w, here, Layer::Ground));
        order_move(&mut w, id, dc);
        run(&mut w, 1);
        assert_eq!(present_flags(&w, dc), (true, false), "high {high}");
        assert!(!present(&w, here, Layer::Ground), "high {high}");
        run(&mut w, 1);
        let o = w.obj(id).unwrap();
        let ground = centre(&w, dc);
        let z = if high { ground.z + 416 } else { ground.z };
        assert_eq!(o.pos, Pos::new(ground.x, ground.y, z), "high {high}");
        let layer = if high { Layer::Deck } else { Layer::Ground };
        assert_eq!(o.layer, layer);
        assert_eq!(present_flags(&w, dc), (true, high), "high {high}");
        glasnost_sim::objects::remove(&mut w, id);
        // On the deck removal clears the deck flag only; on the ground its one mark.
        assert_eq!(present_flags(&w, dc), (high, false), "high {high}");
        // spec: S-0209 claim 6 (the retained object-present flag alone refuses a walker's standing position below the bridge, even after the aircraft is destroyed)
        assert_eq!(
            glasnost_sim::occupancy::no_position_by_flags(&w, dc, Layer::Ground),
            high
        );
    }
}

// spec: S-0207 claim 20 (e) (an aircraft's next accepted destination clears the remembered ground reservation; the old deck flag clears when the next jump removes it)
#[test]
fn claim_20_aircraft_next_jump_from_the_deck() {
    let (mut w, id) = flash(true);
    let dc = cell(0, -6);
    bridge(&mut w, dc);
    order_move(&mut w, id, dc);
    run(&mut w, 2);
    settle(&mut w, id);
    let next = cell(6, 0);
    order_move(&mut w, id, next);
    run(&mut w, 1);
    assert_eq!(present_flags(&w, dc), (false, true));
    assert_eq!(present_flags(&w, next), (true, false));
    run_until(&mut w, 10, |w| w.obj(id).unwrap().pos.cell() == next);
    assert_eq!(present_flags(&w, dc), (false, false));
    assert_eq!(present_flags(&w, next), (true, false));
}

// spec: S-0207 Edge cases (quirks.teleport_aircraft_ground_reservation off: only the eventual landing layer is reserved, and removal clears the landing-layer mark, leaving no separate ground mark)
#[test]
fn claim_20_aircraft_ground_reservation_switch_off() {
    let (mut w, id) = flash(false);
    let dc = cell(0, -6);
    bridge(&mut w, dc);
    order_move(&mut w, id, dc);
    run(&mut w, 1);
    assert_eq!(present_flags(&w, dc), (false, true));
    run(&mut w, 1);
    assert_eq!(w.obj(id).unwrap().layer, Layer::Deck);
    assert_eq!(present_flags(&w, dc), (false, true));
    glasnost_sim::objects::remove(&mut w, id);
    assert_eq!(present_flags(&w, dc), (false, false));
}

// spec: S-0207 claim 20 (k) (an aircraft removed after fixation, before its jump: its actual cell's flag is clear and the pending ground reservation stays; another aircraft's mark is untouched)
#[test]
fn claim_20_aircraft_removed_before_the_jump() {
    let mut s = Scene::new();
    let mut t = unit_type(WHEEL, 6, 5);
    t.locomotor = Some(Locomotor::Teleport);
    s.aircraft("FLASH", t);
    s.place_aircraft(0, "FLASH", cell(0, 0), 64);
    s.place_aircraft(0, "FLASH", cell(0, 1), 64);
    let mut w = s.build();
    open_orders(&mut w);
    let id = objects_of(&w, Kind::Aircraft)[0];
    let dc = cell(0, -6);
    order_move(&mut w, id, dc);
    run(&mut w, 1);
    assert!(present(&w, dc, Layer::Ground));
    glasnost_sim::objects::remove(&mut w, id);
    assert!(
        present(&w, dc, Layer::Ground),
        "pending reservation retained"
    );
    assert!(!present(&w, cell(0, 0), Layer::Ground));
    assert!(present(&w, cell(0, 1), Layer::Ground));
}

// spec: S-0207 claim 20 (k) and table (a vehicle's removal clears only its actual cell's mark at its actual height: on a deck after landing the ground reservation stays; on a deck with a replacement accepted the new ground reservation stays; off a bridge both marks coincide and clear; fixed elsewhere before the jump the pending reservation stays)
#[test]
fn claim_20k_vehicle_removal_rows() {
    let dc = cell(0, -6);
    let next = cell(6, 0);
    // Row 1: on the deck after landing, no replacement destination.
    let (mut w, id) = start(&scene(false));
    bridge(&mut w, dc);
    order_move(&mut w, id, dc);
    run(&mut w, 2);
    settle(&mut w, id);
    assert_eq!(vehicle_flags(&w, dc), (true, true));
    glasnost_sim::objects::remove(&mut w, id);
    assert_eq!(vehicle_flags(&w, dc), (true, false), "row 1");
    // Row 2: on the deck, a replacement accepted but not jumped to.
    let (mut w, id) = start(&scene(false));
    bridge(&mut w, dc);
    order_move(&mut w, id, dc);
    run(&mut w, 2);
    settle(&mut w, id);
    order_move(&mut w, id, next);
    run(&mut w, 1);
    assert_eq!(w.obj(id).unwrap().pos.cell(), dc);
    assert_eq!(vehicle_flags(&w, dc), (false, true));
    glasnost_sim::objects::remove(&mut w, id);
    assert_eq!(vehicle_flags(&w, dc), (false, false), "row 2, old cell");
    assert_eq!(vehicle_flags(&w, next), (true, false), "row 2, new ground");
    // Row 3: landed off a bridge.
    let (mut w, id) = start(&scene(false));
    order_move(&mut w, id, dc);
    run(&mut w, 2);
    settle(&mut w, id);
    assert_eq!(vehicle_flags(&w, dc), (true, false));
    glasnost_sim::objects::remove(&mut w, id);
    assert_eq!(vehicle_flags(&w, dc), (false, false), "row 3");
    // Row 4: landing point fixed elsewhere, not yet jumped to.
    let (mut w, id) = start(&scene(false));
    let here = w.obj(id).unwrap().pos.cell();
    order_move(&mut w, id, dc);
    run(&mut w, 1);
    assert_eq!(w.obj(id).unwrap().pos.cell(), here);
    glasnost_sim::objects::remove(&mut w, id);
    assert_eq!(vehicle_flags(&w, here), (false, false), "row 4, actual");
    assert_eq!(vehicle_flags(&w, dc), (true, false), "row 4, pending");
}

// spec: S-0207 claim 21 (a previous deck landing: installing temporary driving clears neither record of it; driving away clears the actual deck mark only; after leaving and at restoration the former landing cell keeps ground marked with deck clear, and restoration relocates no mark)
#[test]
fn claim_21_previous_deck_landing() {
    let dc = cell(0, -6);
    let (mut w, id) = start(&scene(false));
    for k in 4..=12 {
        bridge(&mut w, cell(0, -k));
    }
    order_move(&mut w, id, dc);
    run(&mut w, 2);
    settle(&mut w, id);
    assert_eq!(vehicle_flags(&w, dc), (true, true));
    // As the factory exit gives it (S-0417 claim 5).
    glasnost_sim::econ::release::install_temporary_drive(&mut w, id);
    assert_eq!(
        vehicle_flags(&w, dc),
        (true, true),
        "installation clears neither"
    );
    let away = cell(0, -10);
    order_move(&mut w, id, away);
    run_until(&mut w, 800, |w| w.obj(id).unwrap().pos.cell() == away);
    assert_eq!(vehicle_flags(&w, dc), (true, false), "after leaving");
    {
        let o = w.obj_mut(id).unwrap();
        o.dest = None;
        o.path.steps.clear();
    }
    assert!(glasnost_sim::econ::release::restore_locomotor(&mut w, id));
    assert_eq!(vehicle_flags(&w, dc), (true, false), "at restoration");
    // Driving marked its current cell at its actual height: the deck.
    let o = w.obj(id).unwrap();
    assert_eq!(o.layer, Layer::Deck);
    assert_eq!(
        vehicle_flags(&w, o.pos.cell()),
        (false, true),
        "the current cell's mark"
    );
}

// spec: S-0207 Edge cases (quirks.teleport_vehicle_ground_reservation off: removal clears both a pending reservation and actual occupancy, leaving no orphan)
#[test]
fn claim_20k_switch_off_clears_the_pending_reservation() {
    let mut s = scene(false);
    s.ruleset.globals.quirks.teleport_vehicle_ground_reservation = false;
    let (mut w, id) = start(&s);
    let dc = cell(0, -6);
    order_move(&mut w, id, dc);
    run(&mut w, 1);
    assert_eq!(vehicle_flags(&w, dc), (true, false));
    glasnost_sim::objects::remove(&mut w, id);
    assert_eq!(vehicle_flags(&w, dc), (false, false));
}

// spec: S-0207 claim 20 (b) (on a high-bridge destination a vehicle asks the entry test at the cell's level plus 4, whatever its own height: an occupied deck refuses and the search lands it elsewhere; an empty deck over an occupied ground record accepts)
#[test]
fn claim_20_high_bridge_entry_is_asked_at_deck_level() {
    for deck_taken in [true, false] {
        let s = scene(false);
        let (mut w, id) = start(&s);
        let dc = cell(0, -6);
        let i = w.cells.idx(dc).expect("cell");
        w.cells.deck[i] = true;
        w.cells.deck_entry[i] = true;
        w.cells.bridge_axis[i] = glasnost_sim::cells::Axis::Y;
        let taken = if deck_taken {
            Layer::Deck
        } else {
            Layer::Ground
        };
        w.cells.claim_vehicle(dc, taken, true);
        order_move(&mut w, id, dc);
        run(&mut w, 1);
        let l = w.obj(id).unwrap().teleport.as_ref().and_then(|t| t.landing);
        let l = l.expect("a landing point");
        assert_eq!(l.cell() == dc, !deck_taken, "deck taken {deck_taken}");
    }
}

// spec: S-0214 claims 11 and 26 (a left click on a cell with a teleporting unit selected gives a move to that cell)
// spec: S-0207 claim 20 (a), (d) and (e) (in play: the landing point is fixed when the order applies and the unit jumps on its next movement update; a warp animation and the departure sound at the old point, the arrival sound and a second warp animation at L, in that order)
#[test]
fn claim_20_click_move_teleports_in_play() {
    use glasnost_sim::combat::effects::{Effect, Sound};
    use glasnost_sim::targeting::{self, ClickOn, Modifiers};
    use glasnost_sim::world::HouseId;
    let mut s = scene(false);
    s.ruleset.sim.general.warp_out = Some("WARPX".into());
    let (mut w, id) = start(&s);
    let from = w.obj(id).unwrap().pos;
    let dc = cell(5, -9);
    let r = targeting::click(
        &w,
        HouseId(0),
        &[id],
        ClickOn::Cell(dc),
        Modifiers::default(),
    );
    assert_eq!(r.orders.len(), 1, "the click gives a move order");
    for o in r.orders {
        assert!(glasnost_sim::orders::issue(&mut w, HouseId(0), o));
    }
    w.effects.clear();
    let mut seen = Vec::new();
    let landed = run_until(&mut w, 5, |w| w.obj(id).unwrap().pos.cell() == dc);
    assert!(
        landed <= 2,
        "it jumps on the movement update after the order"
    );
    seen.extend(w.effects.iter().cloned());
    let to = centre(&w, dc);
    assert_eq!(w.obj(id).unwrap().pos, to);
    let jumper = w.rules().index_of(Kind::Vehicle, "JUMPER").unwrap();
    let order: Vec<(&str, Pos)> = seen
        .iter()
        .filter_map(|e| match e {
            Effect::Anim { name, pos } if name == "WARPX" => Some(("warp", *pos)),
            Effect::Sound {
                sound: Sound::ChronoOut,
                of: Some(t),
                pos: Some(p),
                ..
            } if t.kind == Kind::Vehicle && t.index == jumper => Some(("out", *p)),
            Effect::Sound {
                sound: Sound::ChronoIn,
                of: Some(t),
                pos: Some(p),
                ..
            } if t.kind == Kind::Vehicle && t.index == jumper => Some(("in", *p)),
            _ => None,
        })
        .collect();
    assert_eq!(
        order,
        [("warp", from), ("out", from), ("in", to), ("warp", to)]
    );
}

// spec: S-0207 claim 20 (j) (a request to turn the body turns it at S-0206's turn rate, as for other locomotors)
#[test]
fn claim_20j_body_turns_at_the_turn_rate() {
    let mut s = scene(false);
    let mut driver = jumper(false);
    driver.locomotor = None;
    s.vehicle("DRIVER", driver);
    s.place_unit(0, "DRIVER", cell(3, 3), 64);
    let mut w = s.build();
    open_orders(&mut w);
    let ids = objects_of(&w, Kind::Vehicle);
    let t = w.tick;
    let omega = w.rules().unit(Kind::Vehicle, 0).unwrap().turn_rate;
    for &id in &ids {
        let o = w.obj_mut(id).unwrap();
        o.body.turn_to(32768, t, omega);
    }
    let facings = |w: &World| -> Vec<u16> {
        ids.iter()
            .map(|&id| w.obj(id).unwrap().body.value(w.tick))
            .collect()
    };
    let mut turned = false;
    for _ in 0..20 {
        run(&mut w, 1);
        let f = facings(&w);
        assert_eq!(f[0], f[1], "the teleporter turns as the driver does");
        turned |= f[0] != 32768;
    }
    assert!(turned, "the turn takes time");
    assert_eq!(facings(&w), [32768, 32768]);
}

// spec: S-0207 claim 20 (a) (a held object, S-0614 Terms, drops the destination and does nothing: no landing point, no mark moved, no jump)
#[test]
fn claim_20_held_object_drops_the_destination() {
    let s = scene(false);
    let (mut w, id) = start(&s);
    let until = w.tick + 50;
    w.obj_mut(id)
        .unwrap()
        .parasite
        .get_or_insert_with(Default::default)
        .hold_until = Some(until);
    let from = w.obj(id).unwrap().pos;
    let dc = cell(0, -8);
    let _ = glasnost_sim::orders::issue(&mut w, glasnost_sim::world::HouseId(0), move_to(id, dc));
    run(&mut w, 3);
    let o = w.obj(id).unwrap();
    assert_eq!(o.pos, from);
    assert_eq!(o.dest, None);
    assert!(o.teleport.as_ref().is_none_or(|t| t.landing.is_none()));
    assert!(!marked(&w, dc));
}

// spec: S-0207 claim 20 (e) and (j), claim 22 (a teleporting crusher landing on a cell has a vehicle's arrival effects: cell-centre crushing, S-0210 claims 3 and 4, before the entered-by raise)
#[test]
fn claim_20_crusher_landing_crushes() {
    use glasnost_sim::combat::effects::{Effect, Sound};
    let mut s = Scene::new();
    let mut t = jumper(false);
    t.crusher = Some(true);
    s.vehicle("JUMPER", t);
    let mut g = unit_type(FOOT, 4, 8);
    g.crushable = Some(true);
    s.infantry("GRUNT", g);
    s.place_unit(0, "JUMPER", cell(0, 0), 64);
    s.place_infantry(1, "GRUNT", cell(0, -6), 0);
    let (mut w, id) = start(&s);
    let grunt = objects_of(&w, Kind::Infantry)[0];
    order_move(&mut w, id, cell(0, -6));
    w.effects.clear();
    run_until(&mut w, 5, |w| w.obj(id).unwrap().pos.cell() == cell(0, -6));
    assert_eq!(
        w.obj(id).unwrap().pos.cell(),
        cell(0, -6),
        "it landed there"
    );
    assert!(w.obj(grunt).is_none(), "the infantry is crushed");
    assert!(w.effects.iter().any(|e| matches!(
        e,
        Effect::Sound {
            sound: Sound::Crush,
            ..
        }
    )));
}
