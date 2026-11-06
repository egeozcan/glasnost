//! Claim tests for an infantry unit's being in the middle of firing (S-0215
//! r118 claim 11, Q-2566 answered: S-0107 r40 claim 11's attack state, a
//! readiness blocker), its begin, its end at the shot and its other endings,
//! the request at a change of target, and `quirks.prone_attack_state_persists`
//! (S-0107 claim 11, Quirks). Built on S-0215 scenario B45 part (c) and
//! S-0107 scenario M8 parts (a) and (d), on synthetic data.
//!
//! The ruleset is `testkit::combat_ruleset()` with an invented weapon and
//! invented sequence lengths (no stock values). Each scenario runs a dry pass
//! to find the firing cycle's updates (the start *k* of a firing kind, the
//! shot *u*, the update *v* at whose end *stand* follows the kind), then
//! replays the same seed with the order placed at the window under test.
//! Every update is one tick here (one object per house).

use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{Decimal, SequenceEntry, WeaponType};
use glasnost_sim::infantry::Seq;
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, Target, World};

const N: i32 = 24;
/// Facing east.
const EAST: u16 = 16384;
/// Facing west.
const WEST: u16 = 49152;

/// The WALKER armed with RIFLEX (rate `rof`, range 10, damage 1), its
/// standing and prone fire frames `fs` and `fp`, *fire-standing* and
/// *fire-prone* of `fire` image frames, *stand* and *prone* of 6; a durable
/// BUGGY (unarmed) as the enemy.
fn rules(fire: i32, fs: i32, fp: i32, rof: i32) -> Ruleset {
    let mut r = testkit::combat_ruleset();
    r.sim.weapons.insert(
        "RIFLEX".into(),
        WeaponType {
            damage: Some(1),
            rof: Some(rof),
            range: Some(Decimal("10".into())),
            speed: Some(100),
            projectile: Some("Bullet".into()),
            warhead: Some("SA".into()),
            ..Default::default()
        },
    );
    r.sim.vehicles.get_mut("BUGGY").unwrap().strength = Some(30000);
    let walker = r.sim.infantry.get_mut("WALKER").unwrap();
    walker.primary = Some("RIFLEX".into());
    walker.strength = Some(30000);
    walker.fire_frames = Some([fs, fp, 0, 0]);
    for (k, c) in [
        ("stand", 6),
        ("prone", 6),
        ("fire_standing", fire),
        ("fire_prone", fire),
    ] {
        walker.sequences.insert(
            k.into(),
            SequenceEntry {
                start: 0,
                count: c,
                multiplier: 0,
                direction: None,
            },
        );
    }
    r
}

/// A world of `r` with the Alpha WALKER two cells west of the middle in
/// *guard* and, once the opening order block is over, the Beta BUGGY two
/// cells east of it. The unit's own search takes the BUGGY as its target.
fn scene(r: &Ruleset) -> (World, ObjectId, ObjectId) {
    let mut w = testkit::game_with(r, N, 7);
    let a = testkit::spawn(
        &mut w,
        Kind::Infantry,
        "WALKER",
        0,
        testkit::mid(N, -2, 0),
        EAST,
    );
    while w.tick <= w.orders_blocked_through {
        testkit::run(&mut w, 1);
    }
    let b = testkit::spawn(
        &mut w,
        Kind::Vehicle,
        "BUGGY",
        1,
        testkit::mid(N, 2, 0),
        WEST,
    );
    (w, a, b)
}

/// The read after one update of the unit (S-0215 B45 c's record).
#[derive(Clone, Debug)]
#[allow(dead_code)]
struct Row {
    tick: i32,
    current: Option<Behaviour>,
    queued: Option<Behaviour>,
    target: Option<Target>,
    seq: Option<Seq>,
    seq_t0: i32,
    stage: i32,
    attack: bool,
    /// Its current behaviour was processed in this update (the processing
    /// tick, and no start since then).
    processed: bool,
    shot: bool,
    facing: u16,
    rearm_until: i32,
}

/// Runs one tick and reads the unit.
fn step(w: &mut World, a: ObjectId) -> Row {
    let t = w.tick;
    testkit::run(w, 1);
    let o = w.obj(a).unwrap();
    let st = o.infantry.clone().unwrap();
    Row {
        tick: t,
        current: o.beh.current,
        queued: o.beh.queued,
        target: o.target,
        seq: st.seq,
        seq_t0: st.seq_t0,
        stage: st.stage(t),
        attack: st.attack,
        processed: o.beh.last.is_some_and(|(x, _)| x == t) && !o.beh.expired,
        shot: o.last_fired == t,
        facing: o.body.value(t),
        rearm_until: o.combat.rearm_until,
    }
}

/// A firing kind `q` that starts in a row: current there with stage 0 and
/// started in that tick.
fn starts(x: &Row, q: Seq) -> bool {
    x.seq == Some(q) && x.seq_t0 == x.tick
}

/// The firing cycle of a dry run: the ticks of the first *k* (the kind `q`
/// starts), *u* (its shot), *v* (the end of the update at whose end the
/// follow-up `after` is current once `q` completes), and of the next start
/// of `q` after *v*, if any within `ticks`.
#[derive(Debug, Clone, Copy)]
struct Cycle {
    k: i32,
    u: i32,
    v: i32,
    next: Option<i32>,
}

fn cycle(r: &Ruleset, q: Seq, after: Seq, prep: fn(&mut World, ObjectId), ticks: i32) -> Cycle {
    let (mut w, a, _) = scene(r);
    prep(&mut w, a);
    let rows: Vec<Row> = (0..ticks).map(|_| step(&mut w, a)).collect();
    let k = rows
        .iter()
        .find(|x| starts(x, q))
        .unwrap_or_else(|| panic!("set-up: {q:?} starts: {rows:#?}"))
        .tick;
    let u = rows
        .iter()
        .find(|x| x.tick >= k && x.shot)
        .expect("set-up: the shot")
        .tick;
    let v = rows
        .windows(2)
        .find(|p| p[0].tick >= u && p[0].seq == Some(q) && p[1].seq == Some(after))
        .unwrap_or_else(|| panic!("set-up: the kind completes: {rows:#?}"))[1]
        .tick;
    let next = rows
        .iter()
        .find(|x| x.tick > v && starts(x, q))
        .map(|x| x.tick);
    Cycle { k, u, v, next }
}

fn no_prep(_: &mut World, _: ObjectId) {}

/// Replays `r`'s scene and issues `order` so that it precedes the update of
/// tick `at`: an order is applied after the turn pass of the tick in which
/// it is given (S-0203 claim 7, S-0213 claim 1), so it is given in tick
/// `at` − 1, the order's tick. Returns the rows from the order's tick on,
/// `ticks` of them.
fn ordered(
    r: &Ruleset,
    prep: fn(&mut World, ObjectId),
    at: i32,
    order: impl Fn(ObjectId, ObjectId) -> Order,
    ticks: i32,
) -> (World, ObjectId, ObjectId, Vec<Row>) {
    let (mut w, a, b) = scene(r);
    prep(&mut w, a);
    while w.tick < at - 1 {
        step(&mut w, a);
    }
    assert_eq!(w.tick, at - 1);
    assert!(
        orders::issue(&mut w, HouseId(0), order(a, b)),
        "the order is taken"
    );
    let rows = (0..ticks).map(|_| step(&mut w, a)).collect();
    (w, a, b, rows)
}

fn attack_on(a: ObjectId, b: ObjectId) -> Order {
    Order::Behaviour {
        actor: a,
        behaviour: Behaviour::Attack,
        attack_move: false,
        target: Some(Target::Object(b)),
        dest: None,
    }
}

fn row(rows: &[Row], t: i32) -> &Row {
    rows.iter().find(|x| x.tick == t).unwrap()
}

/// The judged condition of B45 c1 and c2: from the order to `to` the target
/// is unchanged and nothing but *attack* is queued.
fn judged(rows: &[Row], b: ObjectId, to: i32) {
    for x in rows.iter().filter(|x| x.tick <= to) {
        assert_eq!(
            x.target,
            Some(Target::Object(b)),
            "set-up: target held: {x:?}"
        );
        assert!(
            matches!(x.queued, None | Some(Behaviour::Attack)),
            "set-up: nothing else queued: {x:?}"
        );
    }
}

/// B45 c1's rules: *R* longer than the whole *fire-standing* by at least
/// 10 ticks; the stage reaches *F* at least 3 updates after the start and
/// the kind completes at least 6 updates after that.
fn c1_rules() -> Ruleset {
    rules(20, 5, 0, 200)
}

// spec: S-0215 r118 claim 11 (In the middle of firing: begins in the firing turn with the request for a firing kind; ends at the shot, in the same firing turn, so the late start of that update does not see it; not at the end of the firing kind or of the rearm countdown) and scenario B45 run c1, window W1
// spec: S-0107 r40 claim 11 (End of the attack state: when the weapon fires at the fire frame F, the attack state ends with that shot)
#[test]
fn b45_c1_w1_attack_queued_inside_the_firing_kind_becomes_current_after_the_shot() {
    let r = c1_rules();
    let c = cycle(&r, Seq::FireStanding, Seq::Stand, no_prep, 120);
    assert!(c.u - c.k >= 3 && c.v - c.u >= 6, "set-up: {c:?}");
    let (_, _, b, rows) = ordered(&r, no_prep, c.k + 1, attack_on, c.u - c.k + 4);
    judged(&rows, b, c.u + 1);
    for x in rows.iter().filter(|x| x.tick <= c.u) {
        assert_eq!(x.queued, Some(Behaviour::Attack), "W1: still queued: {x:?}");
        assert_ne!(x.current, Some(Behaviour::Attack), "W1: {x:?}");
    }
    assert!(row(&rows, c.u).shot, "set-up: the shot in u");
    assert!(!row(&rows, c.u).attack, "the shot ends the attack state");
    let x = row(&rows, c.u + 1);
    assert_eq!(
        x.current,
        Some(Behaviour::Attack),
        "W1: current after u + 1: {x:?}"
    );
    assert!(x.processed, "W1: update u + 1 processes it: {x:?}");
}

// spec: S-0215 r118 claim 11 (A unit that keeps its target: after the shot it is not in the attack state until its firing check answers can fire again, which does not depend on the end of the firing kind) and scenario B45 run c1, windows W2 and W3
#[test]
fn b45_c1_w2_and_w3_attack_after_the_shot_is_current_at_the_first_update() {
    let r = c1_rules();
    let c = cycle(&r, Seq::FireStanding, Seq::Stand, no_prep, 120);
    // W2: after u and not after v.
    let at = c.u + 2;
    assert!(at <= c.v, "set-up: {c:?}");
    let (_, _, b, rows) = ordered(&r, no_prep, at, attack_on, 3);
    judged(&rows, b, at);
    let x = row(&rows, at);
    assert_eq!(
        x.seq,
        Some(Seq::FireStanding),
        "W2: the kind still plays: {x:?}"
    );
    assert_eq!(x.current, Some(Behaviour::Attack), "W2: {x:?}");
    assert!(x.processed, "W2: processed in the first update: {x:?}");
    // W3: after v and not after the next fire-standing start (the
    // countdown still runs).
    let at = c.v + 1;
    assert!(c.next.is_none_or(|n| at <= n), "set-up: {c:?}");
    let (_, _, b, rows) = ordered(&r, no_prep, at, attack_on, 3);
    judged(&rows, b, at);
    let x = row(&rows, at);
    assert!(
        x.rearm_until > at,
        "set-up: the countdown still runs: {x:?}"
    );
    assert_eq!(x.current, Some(Behaviour::Attack), "W3: {x:?}");
    assert!(x.processed, "W3: processed in the first update: {x:?}");
}

/// B45 c2's rules: *R* longer than the stage delay of *fire-standing*, and
/// the time from the shot to *v* longer than *R* by at least 6 ticks.
fn c2_rules() -> Ruleset {
    rules(40, 3, 0, 12)
}

// spec: S-0215 r118 claim 11 (A unit that keeps its target: if its firing check answers can fire again while the firing kind is still current, the attack state begins again with no new sequence, the request turned down; for a standing unit it lasts to the stage-delay-0 ending in the first update that begins with stand current after the kind completes, so a behaviour queued meanwhile becomes current at that update's late start and is first processed at the next update) and scenario B45 run c2
// spec: S-0215 r118 claim 11 (Other endings: a current sequence whose stage delay is 0, after the update's processing and before its late start) and S-0107 r40 claim 11 (a unit in the attack state when its firing kind completes on tick t with stand as its follow-up leaves the attack state on tick t + 1 and can start a new firing kind on that same tick)
#[test]
fn b45_c2_a_second_begin_inside_the_kind_holds_the_order_to_the_late_start_after_it() {
    let r = c2_rules();
    let c = cycle(&r, Seq::FireStanding, Seq::Stand, no_prep, 200);
    let dry = {
        let (mut w, a, _) = scene(&r);
        (0..=c.u - w.tick).map(|_| step(&mut w, a)).last().unwrap()
    };
    let rr = dry.rearm_until - c.u;
    assert!(rr > 1 && c.v - c.u > rr + 6, "set-up: R {rr}, {c:?}");
    let at = c.u + rr + 3;
    let (_, _, b, rows) = ordered(&r, no_prep, at, attack_on, c.v - at + 4);
    judged(&rows, b, c.v + 1);
    // No new fire-standing before v: the stage keeps rising.
    for x in rows.iter().filter(|x| x.tick <= c.v) {
        assert!(
            !starts(x, Seq::FireStanding),
            "no new fire-standing before v: {x:?}"
        );
        assert_eq!(
            x.queued,
            Some(Behaviour::Attack),
            "still queued to v: {x:?}"
        );
    }
    assert!(
        rows.iter().any(|x| x.tick < c.v && x.attack),
        "the attack state began again inside the kind: {rows:#?}"
    );
    let x = row(&rows, c.v + 1);
    assert_eq!(
        x.current,
        Some(Behaviour::Attack),
        "current after v + 1: {x:?}"
    );
    assert!(
        !x.processed,
        "v + 1 does not process it (the late start): {x:?}"
    );
    assert!(
        starts(x, Seq::FireStanding),
        "a new fire-standing in v + 1: {x:?}"
    );
    let x = row(&rows, c.v + 2);
    assert_eq!(x.current, Some(Behaviour::Attack));
    assert!(x.processed, "its first attack processing in v + 2: {x:?}");
}

/// A clear cell two cells north of the unit.
fn move_north(a: ObjectId, _: ObjectId) -> Order {
    let c = testkit::mid(N, -2, -2);
    Order::Behaviour {
        actor: a,
        behaviour: Behaviour::Move,
        attack_move: false,
        target: None,
        dest: Some(Target::Cell(c)),
    }
}

// spec: S-0215 r118 claim 11 (Other endings: a change of its target by an order, such as a move order, when the order takes effect, so no start after it sees the state) and scenario B45 run c3
// spec: S-0107 r40 claim 11 (a change of target asks, not forced, for stand, so a firing kind still playing gives way to it)
#[test]
fn b45_c3_a_move_order_ends_the_state_as_it_takes_effect() {
    let r = c1_rules();
    let c = cycle(&r, Seq::FireStanding, Seq::Stand, no_prep, 120);
    assert!(c.u - c.k >= 3, "set-up: {c:?}");
    let at = c.k + 1;
    let (_, _, _, rows) = ordered(&r, no_prep, at, move_north, 8);
    // The read after the order's tick.
    let x = row(&rows, at - 1);
    assert_ne!(
        x.seq,
        Some(Seq::FireStanding),
        "c3: the kind gave way: {x:?}"
    );
    assert!(!x.attack, "c3: {x:?}");
    assert_eq!(x.queued, Some(Behaviour::Move), "c3: {x:?}");
    let x = row(&rows, at);
    assert_eq!(x.current, Some(Behaviour::Move), "c3: move current: {x:?}");
    assert!(x.processed, "c3: processed in the first update: {x:?}");
    assert!(
        rows.iter().all(|x| !x.shot),
        "c3: no shot from that kind: {rows:#?}"
    );
}

/// Lays the unit down, with no fear, so that it stays prone.
fn lie_down(w: &mut World, a: ObjectId) {
    w.obj_mut(a).unwrap().infantry.as_mut().unwrap().prone = true;
    glasnost_sim::infantry::request(w, a, Seq::Prone, true);
}

/// Shots of a prone run, each with the sequence current after its update.
fn prone_shots(r: &Ruleset, ticks: i32) -> Vec<Row> {
    let (mut w, a, _) = scene(r);
    lie_down(&mut w, a);
    (0..ticks)
        .map(|_| step(&mut w, a))
        .filter(|x| x.shot)
        .collect()
}

fn with_persist(mut r: Ruleset, on: bool) -> Ruleset {
    r.globals.quirks.prone_attack_state_persists = on;
    r
}

// spec: S-0107 r40 claim 11 (a prone unit that is in the attack state when fire-prone completes stays in it, quirks.prone_attack_state_persists on, and fires again with no firing sequence when the stage of prone equals F, at most once per start of prone) and Quirks (quirks.prone_attack_state_persists, both positions) and scenario M8 part (a), the short-delay prone runs
// spec: S-0215 r118 claim 11 (for a prone unit the attack state lasts into prone, to S-0107 claim 11's repeat shot; with the switch off it ends when fire-prone completes)
#[test]
fn m8_a_prone_short_rearm_repeat_shot_in_prone_by_the_switch() {
    // Dry run with a long rearm to find fire-prone's start, shot and
    // completion.
    let long = rules(20, 0, 2, 400);
    let c = cycle(&long, Seq::FireProne, Seq::Prone, lie_down, 120);
    let r_short = (c.v - c.u) / 2;
    let fp = {
        let (mut w, a, _) = scene(&long);
        lie_down(&mut w, a);
        (0..=c.k - w.tick).map(|_| step(&mut w, a)).last().unwrap()
    };
    assert_eq!(fp.seq, Some(Seq::FireProne), "set-up");
    for on in [true, false] {
        let r = with_persist(rules(20, 0, 2, r_short), on);
        let shots = prone_shots(&r, 200);
        assert!(shots.len() >= 3, "set-up: shots: {shots:#?}");
        let first = &shots[0];
        let rr = first.rearm_until - first.tick;
        // The countdown does not end while the stage still equals F, and
        // ends at least 3 ticks before fire-prone completes.
        assert!(
            rr > 1 && first.tick + rr + 3 <= c.v - c.u + first.tick,
            "set-up: R {rr}, {c:?}"
        );
        let in_prone: Vec<&Row> = shots.iter().filter(|x| x.seq == Some(Seq::Prone)).collect();
        if on {
            assert!(
                !in_prone.is_empty(),
                "on: an extra shot during prone: {shots:#?}"
            );
        } else {
            assert!(in_prone.is_empty(), "off: no shot during prone: {shots:#?}");
            assert!(
                shots.iter().all(|x| x.seq == Some(Seq::FireProne)),
                "off: each shot from a fire-prone: {shots:#?}"
            );
        }
    }
}

// spec: S-0107 r40 claim 11 (If the rearm countdown outlasts the firing kind, the unit is not in the attack state when the kind completes, and its next shot comes from a new firing kind) and scenario M8 part (a), the long-delay prone runs (prone fire frame of at least 2, a countdown that ends after fire-prone completes and fewer than F × d ticks after it); both positions of quirks.prone_attack_state_persists
#[test]
fn m8_a_prone_long_rearm_every_shot_from_a_new_fire_prone() {
    let long = rules(20, 0, 2, 400);
    let c = cycle(&long, Seq::FireProne, Seq::Prone, lie_down, 120);
    // The countdown ends 5 ticks after fire-prone completes, inside the
    // window of F × d = 2 × 6 ticks.
    let rof = c.v - c.u + 5;
    for on in [true, false] {
        let r = with_persist(rules(20, 0, 2, rof), on);
        let shots = prone_shots(&r, 300);
        assert!(shots.len() >= 3, "set-up: shots: {shots:#?}");
        let rr = shots[0].rearm_until - shots[0].tick;
        assert!(
            rr > c.v - c.u && rr < c.v - c.u + 12,
            "set-up: R {rr} ends in the window, {c:?}"
        );
        for x in &shots {
            assert_eq!(
                x.seq,
                Some(Seq::FireProne),
                "a shot from a fire-prone: {shots:#?}"
            );
        }
        let starts: Vec<i32> = shots.iter().map(|x| x.seq_t0).collect();
        let mut distinct = starts.clone();
        distinct.dedup();
        assert_eq!(starts, distinct, "each shot from a new fire-prone");
    }
}

/// A second BUGGY three cells south of the unit, in a clearly different
/// direction and within range.
fn second_enemy(w: &mut World) -> ObjectId {
    testkit::spawn(w, Kind::Vehicle, "BUGGY", 1, testkit::mid(N, -2, 3), WEST)
}

fn dir(w: &World, a: ObjectId, b: ObjectId) -> u16 {
    let (p, q) = (w.reference_point(a), w.reference_point(b));
    glasnost_num::geom::direction(
        i64::from(q.x - p.x),
        i64::from(q.y - p.y),
        w.rules().quirks.table_direction,
    )
}

// spec: S-0107 r40 claim 11 (a change of target by an order asks, not forced, for stand, so a firing kind still playing gives way to it; a new fire-standing starts at its first image frame once the weapon is ready, its body facing set at once to the new target's direction) and scenario M8 part (d)
// spec: S-0215 r118 claim 11 (Other endings: an attack order on another target, when the order takes effect)
#[test]
fn m8_d_an_attack_order_on_another_target_after_the_shot() {
    let r = rules(40, 3, 0, 12);
    let c = cycle(&r, Seq::FireStanding, Seq::Stand, no_prep, 200);
    let (mut w, a, b) = scene(&r);
    while w.tick < c.u + 1 {
        step(&mut w, a);
    }
    let before = w.obj(a).unwrap().infantry.clone().unwrap();
    assert_eq!(
        before.seq,
        Some(Seq::FireStanding),
        "set-up: the kind still plays"
    );
    assert_eq!(
        w.obj(a).unwrap().target,
        Some(Target::Object(b)),
        "set-up: on A"
    );
    // Wall B appears only now, so the unit's own search has not taken it.
    let b2 = second_enemy(&mut w);
    let at = w.tick;
    assert!(orders::issue(&mut w, HouseId(0), attack_on(a, b2)));
    let rows: Vec<Row> = (0..40).map(|_| step(&mut w, a)).collect();
    let x = row(&rows, at);
    assert_eq!(x.target, Some(Target::Object(b2)), "set-up: the new target");
    assert!(
        x.seq == Some(Seq::Stand) || starts(x, Seq::FireStanding),
        "the earlier fire-standing gave way: {x:?}"
    );
    let s = rows
        .iter()
        .find(|x| starts(x, Seq::FireStanding))
        .expect("a new fire-standing within three rearm delays");
    assert!(s.tick - at <= 3 * 12, "within three rearm delays: {s:?}");
    assert_eq!(s.stage, 0, "from its first image frame");
    assert_eq!(
        s.facing,
        dir(&w, a, b2),
        "the facing set at once to the new target"
    );
}

// spec: S-0215 r118 claim 11 (Infantry is ready unless it is in the middle of firing, S-0107 claim 11's attack state and nothing else: the rearm countdown and a firing kind still playing do not hold readiness back by themselves)
#[test]
fn claim_11_readiness_is_the_attack_state() {
    let r = c1_rules();
    let (mut w, a, _) = scene(&r);
    w.obj_mut(a).unwrap().infantry.as_mut().unwrap().attack = true;
    assert!(
        !glasnost_sim::process::ready(&w, a),
        "in the attack state: not ready"
    );
    w.obj_mut(a).unwrap().infantry.as_mut().unwrap().attack = false;
    assert!(glasnost_sim::process::ready(&w, a), "control: ready");
    // A firing kind still playing and a running countdown, outside the
    // attack state: ready.
    glasnost_sim::infantry::request(&mut w, a, Seq::FireStanding, true);
    let t = w.tick;
    w.obj_mut(a).unwrap().combat.rearm_until = t + 100;
    assert!(
        glasnost_sim::process::ready(&w, a),
        "kind and countdown alone: ready"
    );
}
