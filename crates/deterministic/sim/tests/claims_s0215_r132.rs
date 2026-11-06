//! Claim tests of S-0215 revision 132 (Q-2656 answered): claim 22's
//! end-of-update check of an infantry unit. At the end of each update, after
//! its late start, an infantry unit whose current behaviour is *move* (or
//! none, with *move* queued), that is not moving and has no destination,
//! falls back to rest (claims 13 and 14). What that queues becomes current at
//! the next tick's first start. Built on scenario B49 parts (a) to (d), B15
//! part (c) and B6 part (e), on synthetic data: the test kit's invented
//! walker, our own water cells and an invented teleporting copy of the
//! walker. No game data.

use glasnost_map::CellXY;
use glasnost_map::terrain::land;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{BehaviourProps, Locomotor};
use glasnost_sim::orders::{self, Order};
use glasnost_sim::process;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit::{self, SIZE, mid};
use glasnost_sim::tick;
use glasnost_sim::world::{HouseId, ObjectId, Pos, Target, World};

/// The combat ruleset with `BLINK`, a copy of the walker with the
/// teleporting locomotor (B49 d's named test rules file).
fn ruleset() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    let mut blink = r.sim.infantry["WALKER"].clone();
    blink.locomotor = Some(Locomotor::Teleport);
    r.sim.infantry.insert("BLINK".into(), blink);
    r.registries
        .get_mut("infantry")
        .unwrap()
        .push("BLINK".into());
    r
}

/// A game of `r` with one unit of type `name` of house 0 resting in *guard*
/// at the map's middle, orders open (S-0201 claim 14).
fn one_of(r: &Ruleset, name: &str) -> (World, ObjectId) {
    let mut w = testkit::game_with(r, SIZE, 3);
    let id = testkit::spawn(&mut w, Kind::Infantry, name, 0, mid(SIZE, 0, 0), 0);
    while orders::blocked(&w) {
        tick::step(&mut w);
    }
    (w, id)
}

fn one(name: &str) -> (World, ObjectId) {
    one_of(&ruleset(), name)
}

/// Makes `c` an island: its eight neighbours become water, so `c` lies in
/// another reach region (M-0023's walled pen, our own cells).
fn island(w: &mut World, c: CellXY) {
    for dy in -1..=1 {
        for dx in -1..=1 {
            if (dx, dy) != (0, 0) {
                let n = CellXY::new(c.x + dx, c.y + dy);
                w.cells.grid.get_mut(n).unwrap().land = land::WATER;
                glasnost_sim::regions::cell_changed(w, n);
            }
        }
    }
}

fn order(w: &mut World, o: Order) {
    assert!(orders::issue(w, HouseId(0), o), "order refused");
}

fn move_to(w: &mut World, id: ObjectId, c: CellXY) {
    order(
        w,
        Order::Behaviour {
            actor: id,
            behaviour: Behaviour::Move,
            attack_move: false,
            target: None,
            dest: Some(Target::Cell(c)),
        },
    );
    // Orders take effect after the turn pass (S-0203 claim 7): applied now,
    // as at the end of the tick just run, the order takes effect at the
    // start of the next tick, the order's tick of B49.
    orders::apply_pending(w);
}

/// The recorder of B49: read at the end of a tick.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Row {
    tick: i32,
    current: Option<Behaviour>,
    queued: Option<Behaviour>,
    dest: Option<Target>,
    pos: Pos,
}

fn read(w: &World, id: ObjectId) -> Row {
    let o = w.obj(id).unwrap();
    Row {
        tick: w.tick - 1,
        current: o.beh.current,
        queued: o.beh.queued,
        dest: o.dest,
        pos: o.pos,
    }
}

/// Runs `n` ticks and records the unit at the end of each.
fn record(w: &mut World, id: ObjectId, n: i32) -> Vec<Row> {
    (0..n)
        .map(|_| {
            tick::step(w);
            read(w, id)
        })
        .collect()
}

/// The first row at whose end the unit holds no destination.
fn first_without_dest(rows: &[Row]) -> usize {
    rows.iter()
        .position(|r| r.dest.is_none())
        .expect("a tick without a destination")
}

// spec: S-0215 claim 22 (r132, End-of-update check: a failed gate in the order's tick leaves move current, guard queued and no destination at the end of that update; guard current at the next tick's first start)
// spec: S-0212 claim 23 (the gate row clears the walker's destination)
#[test]
fn b49_a_gate_failure_queues_the_rest_at_the_end_of_the_update() {
    let (mut w, g) = one("WALKER");
    let pen = mid(SIZE, 4, 0);
    island(&mut w, pen);
    let start = w.obj(g).unwrap().pos;
    move_to(&mut w, g, pen);
    let t = w.tick;
    let rows = record(&mut w, g, 20);
    let r0 = rows[0];
    assert_eq!(r0.tick, t);
    assert_eq!(r0.current, Some(Behaviour::Move), "{r0:?}");
    assert_eq!(r0.queued, Some(Behaviour::Guard), "{r0:?}");
    assert_eq!(r0.dest, None);
    let r1 = rows[1];
    assert_eq!(r1.current, Some(Behaviour::Guard), "{r1:?}");
    assert_eq!(r1.queued, None);
    assert_eq!(r1.dest, None);
    assert!(rows.iter().all(|r| r.pos == start), "no step in 20 ticks");
}

// spec: S-0215 claim 22 (r132: in the update whose locomotion clears the destination, the move processing earlier in it saw the destination as before the locomotion and chose the standard delay, not the fall-back's 1 tick)
#[test]
fn b49_a_the_move_processing_of_that_update_chooses_the_standard_delay() {
    let (mut w, g) = one("WALKER");
    let pen = mid(SIZE, 4, 0);
    island(&mut w, pen);
    move_to(&mut w, g, pen);
    let t = w.tick;
    tick::step(&mut w);
    let (at, delay) = w.obj(g).unwrap().beh.last.expect("a processing");
    assert_eq!(at, t, "move was processed in the order's tick");
    assert!(
        delay > 1,
        "the standard delay, not the fall-back's 1 tick: {delay}"
    );
}

// spec: S-0215 claim 22 (r132: an arrival that ends the walk at its destination, S-0208 claims 6 and 16: move current and guard queued at the end of the first tick without a destination, guard current at the end of the next)
#[test]
fn b49_b_arrival_queues_the_rest_at_the_end_of_the_arrival_update() {
    let (mut w, g) = one("WALKER");
    move_to(&mut w, g, mid(SIZE, 5, 0));
    let rows = record(&mut w, g, 400);
    let k = first_without_dest(&rows);
    assert!(k > 0);
    assert_eq!(rows[k].current, Some(Behaviour::Move), "{:?}", rows[k]);
    assert_eq!(rows[k].queued, Some(Behaviour::Guard), "{:?}", rows[k]);
    assert_eq!(rows[k].pos.cell(), mid(SIZE, 5, 0));
    assert_eq!(
        rows[k + 1].current,
        Some(Behaviour::Guard),
        "{:?}",
        rows[k + 1]
    );
    assert_eq!(rows[k + 1].queued, None);
    // Every earlier tick ends with move current and nothing queued.
    assert!(
        rows[..k]
            .iter()
            .all(|r| r.current == Some(Behaviour::Move) && r.queued.is_none()),
        "{:?}",
        &rows[..k]
    );
}

// spec: S-0215 claim 22 (r132: after a stop order, S-0215 claim 12, the update whose locomotion ends the step under way is the tick of the last change of position or the tick after it; move current and guard queued at its end, guard current at the end of the next)
#[test]
fn b49_c_stop_order_queues_the_rest_at_the_end_of_the_step() {
    let (mut w, g) = one("WALKER");
    move_to(&mut w, g, mid(SIZE, 5, 0));
    let before = record(&mut w, g, 30);
    assert!(before.iter().all(|r| r.dest.is_some()), "still walking");
    order(&mut w, Order::Stop { actor: g });
    let mut last = read(&w, g).pos;
    let rows = record(&mut w, g, 40);
    let mut last_change = None;
    for (i, r) in rows.iter().enumerate() {
        if r.pos != last {
            last_change = Some(i);
            last = r.pos;
        }
    }
    let changed = last_change.expect("the step under way is finished");
    let caught: Vec<usize> = rows
        .iter()
        .enumerate()
        .filter(|(_, r)| r.current == Some(Behaviour::Move) && r.queued == Some(Behaviour::Guard))
        .map(|(i, _)| i)
        .collect();
    assert_eq!(caught.len(), 1, "exactly one such tick: {rows:?}");
    let k = caught[0];
    assert!(k == changed || k == changed + 1, "{k} against {changed}");
    assert_eq!(rows[k + 1].current, Some(Behaviour::Guard));
}

// spec: S-0215 claim 22 (r132: the jump tick of a teleporting infantry unit, whose jump stops its movement and removes its destination, S-0207 claim 20 (e): move current, guard queued and no destination at the end of the jump tick)
#[test]
fn b49_d_teleporting_infantry_queues_the_rest_in_its_jump_tick() {
    let (mut w, b) = one("BLINK");
    let to = mid(SIZE, 5, 0);
    move_to(&mut w, b, to);
    let rows = record(&mut w, b, 40);
    let k = rows
        .iter()
        .position(|r| r.pos.cell() == to)
        .expect("the jump");
    assert_eq!(rows[k].current, Some(Behaviour::Move), "{:?}", rows[k]);
    assert_eq!(rows[k].queued, Some(Behaviour::Guard), "{:?}", rows[k]);
    assert_eq!(rows[k].dest, None);
    assert_eq!(rows[k + 1].current, Some(Behaviour::Guard));
}

// spec: S-0215 claim 22 (r132: the check is made whatever is queued, with move queued and nothing current too, and whether or not move was processed in the update)
#[test]
fn claim_22_check_with_move_queued_and_nothing_current() {
    let (mut w, g) = one("WALKER");
    {
        let o = w.obj_mut(g).unwrap();
        o.beh.current = None;
        o.beh.queued = Some(Behaviour::Move);
        o.dest = None;
    }
    process::end_of_update_check(&mut w, g);
    assert_eq!(w.obj(g).unwrap().beh.queued, Some(Behaviour::Guard));
}

// spec: S-0215 claim 22 (r132: the check needs no destination; a unit in move with a destination is left alone by our engine, Open questions, Other cases of the end-of-update check) and S-0215 claim 22 (it falls back only for move, as current or seen)
#[test]
fn claim_22_no_check_with_a_destination_or_another_behaviour() {
    let (mut w, g) = one("WALKER");
    {
        let o = w.obj_mut(g).unwrap();
        o.beh.current = Some(Behaviour::Move);
        o.beh.queued = None;
        o.dest = Some(Target::Cell(mid(SIZE, 3, 0)));
    }
    let before = w.obj(g).unwrap().beh.clone();
    process::end_of_update_check(&mut w, g);
    assert_eq!(w.obj(g).unwrap().beh, before, "a destination: nothing");
    {
        let o = w.obj_mut(g).unwrap();
        o.beh.current = Some(Behaviour::Sleep);
        o.dest = None;
    }
    let before = w.obj(g).unwrap().beh.clone();
    process::end_of_update_check(&mut w, g);
    assert_eq!(w.obj(g).unwrap().beh, before, "not move: nothing");
}

// spec: S-0215 claim 22 (r132: a vehicle has no such check in our engine, Open questions)
#[test]
fn claim_22_vehicles_have_no_check() {
    let mut w = testkit::game_with(&ruleset(), SIZE, 3);
    let v = testkit::spawn(&mut w, Kind::Vehicle, "BUGGY", 0, mid(SIZE, 0, 0), 0);
    {
        let o = w.obj_mut(v).unwrap();
        o.beh.current = Some(Behaviour::Move);
        o.beh.queued = None;
        o.dest = None;
    }
    process::end_of_update_check(&mut w, v);
    assert_eq!(w.obj(v).unwrap().beh.queued, None);
}

/// A route point add of a move to `c` (S-0213 claim 16a).
fn add(actor: ObjectId, c: CellXY) -> Order {
    Order::RoutePoint {
        actor,
        behaviour: Behaviour::Move,
        attack_move: false,
        target: None,
        dest: Some(Target::Cell(c)),
        place: Some(Target::Cell(c)),
    }
}

// spec: S-0215 claim 22 (r132: the route row's fall-back comes from the end-of-update check at the end of the update whose locomotion ends the walk at the first point; a holder ready to change has wait current there and queues no guard) and claim 13 (r132: the route row queues wait with an immediate start before the issue, S-0213 claim 16c); scenario B15 part c
#[test]
fn b15_c_route_holder_waits_at_the_end_of_its_arrival_update() {
    let (mut w, g) = one("WALKER");
    let (c1, c2) = (mid(SIZE, 4, 0), mid(SIZE, 4, 4));
    order(&mut w, add(g, c1));
    order(&mut w, add(g, c2));
    tick::step(&mut w);
    order(&mut w, Order::RouteCommit);
    let rows = record(&mut w, g, 400);
    let k = rows
        .iter()
        .position(|r| r.pos.cell() == c1 && r.dest.is_none_or(|d| d != Target::Cell(c1)))
        .expect("the walk ends at the first point");
    assert_eq!(
        rows[k - 1].current,
        Some(Behaviour::Move),
        "valid run: move current before the arrival update"
    );
    assert_eq!(rows[k].current, Some(Behaviour::Wait), "{:?}", rows[k]);
    assert_ne!(rows[k].queued, Some(Behaviour::Guard), "{:?}", rows[k]);
    assert!(
        rows[k..].iter().any(|r| r.pos.cell() == c2),
        "the next point is taken without further input"
    );
}

/// [`ruleset`] with `props` on *move*.
fn move_props(props: BehaviourProps) -> Ruleset {
    let mut r = ruleset();
    r.sim.behaviours.insert("move".into(), props);
    r
}

/// B6 (e)'s infantry unit: ordered 5 cells, arrives; returns the rows from
/// the order on.
fn b6e_infantry(r: &Ruleset) -> Vec<Row> {
    let (mut w, g) = one_of(r, "WALKER");
    move_to(&mut w, g, mid(SIZE, 5, 0));
    record(&mut w, g, 300)
}

// spec: S-0215 claim 22 (r132, scenario B6 e run C: the infantry unit has move current and guard queued at the end of the first tick at whose end it has no destination, before any move processing after it has stopped)
#[test]
fn b6_e_run_c_infantry_queues_guard_at_the_end_of_the_arrival_update() {
    let rows = b6e_infantry(&ruleset());
    let k = first_without_dest(&rows);
    assert_eq!(rows[k].current, Some(Behaviour::Move));
    assert_eq!(rows[k].queued, Some(Behaviour::Guard));
}

// spec: S-0215 claim 22 and claim 14 row 6 (B6 e runs Z and P: with the zombie or the paralysed property on move, the infantry unit queues nothing, also at the end of each update by the end-of-update check, and move stays current)
#[test]
fn b6_e_runs_z_and_p_infantry_queues_nothing() {
    for props in [
        BehaviourProps {
            zombie: Some(true),
            ..Default::default()
        },
        BehaviourProps {
            paralysed: Some(true),
            ..Default::default()
        },
    ] {
        let rows = b6e_infantry(&move_props(props.clone()));
        let k = first_without_dest(&rows);
        assert!(
            rows[k..]
                .iter()
                .all(|r| r.current == Some(Behaviour::Move) && r.queued.is_none()),
            "{props:?}: {:?}",
            &rows[k..k + 3]
        );
    }
}
