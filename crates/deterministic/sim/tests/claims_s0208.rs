//! Claim tests for S-0208 (infantry movement: standing positions, the
//! walking step and speed, crawling, scatter acceptance and destinations),
//! on synthetic data with invented type names and values.

mod common;

use common::{FOOT, Scene, cell, code_tile, open_orders, order_move, slope_tile, unit_type};
use glasnost_map::CellXY;
use glasnost_num::F64;
use glasnost_rules::sim::{BehaviourProps, Decimal, LandRow, LandType, SequenceEntry, UnitType};
use glasnost_sim::cells::position;
use glasnost_sim::infantry::Seq;
use glasnost_sim::occupancy::{self, Outcome};
use glasnost_sim::regions::STEPS;
use glasnost_sim::rng::Rng;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::scatter::{self, ScatterRequest};
use glasnost_sim::tick;
use glasnost_sim::walk;
use glasnost_sim::world::{ObjectId, Pos, Target, World};

/// Speed field of the test soldier: type speed V = ⌊256 × 4 / 100⌋ = 10.
const SPEED_FIELD: i32 = 4;
/// The walking speed that speed field gives with every factor at 1.0.
const S: i32 = 10;

fn soldier() -> UnitType {
    unit_type(FOOT, SPEED_FIELD, 0)
}

/// A scene with the soldier type TESTMAN registered.
fn scene() -> Scene {
    let mut s = Scene::new();
    s.infantry("TESTMAN", soldier());
    s
}

/// The infantry objects in ID order.
fn men(w: &World) -> Vec<ObjectId> {
    common::objects_of(w, Kind::Infantry)
}

fn pos(w: &World, id: ObjectId) -> Pos {
    w.obj(id).expect("object").pos
}

/// World position of standing position `p` in cell `c`.
fn at(c: CellXY, p: u8) -> (i32, i32) {
    let (u, v) = walk::standing_point(p);
    (256 * c.x + u, 256 * c.y + v)
}

fn walk_target(w: &World, id: ObjectId) -> Option<Pos> {
    w.obj(id)
        .and_then(|o| o.walk.as_ref())
        .and_then(|s| s.target)
}

fn dec(s: &str) -> Decimal {
    Decimal(s.to_string())
}

fn half_span(w: &World) -> bool {
    w.rules().quirks.ranged_draw_half_span
}

/// One tick of a walk, as seen from outside.
#[derive(Debug, Clone, Copy)]
struct Tick {
    before: Pos,
    after: Pos,
    had_target: bool,
    has_target: bool,
}

/// Runs up to `max` ticks, recording the walker's position and walking
/// target on each, until it has no destination left.
fn trace(w: &mut World, id: ObjectId, max: i32) -> Vec<Tick> {
    let mut out = Vec::new();
    for _ in 0..max {
        let before = pos(w, id);
        let had_target = walk_target(w, id).is_some();
        tick::step(w);
        let after = pos(w, id);
        let has_target = walk_target(w, id).is_some();
        out.push(Tick {
            before,
            after,
            had_target,
            has_target,
        });
        let o = w.obj(id).expect("walker");
        if o.dest.is_none() && !has_target {
            break;
        }
    }
    out
}

/// Ticks from the first step to the last arrival, both included.
fn walk_span(t: &[Tick]) -> i32 {
    let first = t.iter().position(|k| k.before != k.after).expect("a step");
    let last = t.iter().rposition(|k| k.before != k.after).expect("a step");
    (last - first + 1) as i32
}

/// The steps that moved the walker without arriving (it had a target before
/// and after the tick).
fn steps(t: &[Tick]) -> Vec<(i32, i32)> {
    t.iter()
        .filter(|k| k.had_target && k.has_target && k.before != k.after)
        .map(|k| (k.after.x - k.before.x, k.after.y - k.before.y))
        .collect()
}

/// A soldier at standing position `sub` of cell(0, 0) walks `n` cells due
/// map east; returns the world, the walker and the trace.
fn walk_east(s: &Scene, sub: i32, n: i32) -> (World, ObjectId, Vec<Tick>) {
    let mut s2 = Scene::new();
    s2.ruleset = s.ruleset.clone();
    s2.map = s.map.clone();
    s2.catalogue = s.catalogue.clone();
    s2.setup = s.setup.clone();
    s2.place_infantry(0, "TESTMAN", cell(0, 0), sub);
    let mut w = s2.build();
    let id = men(&w)[0];
    open_orders(&mut w);
    order_move(&mut w, id, cell(n, 0));
    let t = trace(&mut w, id, 400);
    (w, id, t)
}

// ---------------------------------------------------------------------------
// Standing positions

// spec: S-0208 claim 1 (an arriving walker stands on a table P point at the terrain surface height)
#[test]
fn arrival_is_on_a_standing_position_at_surface_height() {
    let mut s = scene();
    // The second cell is a ramp, so its surface height varies within it.
    s.set_cell(cell(1, 0), slope_tile(2), 0);
    s.set_cell(cell(2, 0), slope_tile(2), 0);
    let (w, id, t) = walk_east(&s, 2, 2);
    let p = pos(&w, id);
    assert_eq!(p.cell(), cell(2, 0), "notes: {:?}", w.notes);
    assert_eq!((p.x, p.y), at(cell(2, 0), position::RIGHT));
    assert_eq!(p.z, w.surface_at(p.x, p.y));
    // Every arrival lands on a standing position of table P.
    for k in t.iter().filter(|k| k.had_target && !k.has_target) {
        let (u, v) = (k.after.x.rem_euclid(256), k.after.y.rem_euclid(256));
        assert!(
            (0..5).any(|q| walk::standing_point(q) == (u, v)),
            "arrival at ({u}, {v})"
        );
        assert_eq!(k.after.z, w.surface_at(k.after.x, k.after.y));
    }
}

// spec: S-0208 claim 1 (a pre-placed unit stands at the surface height of its standing position)
#[test]
fn pre_placed_unit_on_a_slope_stands_at_surface_height() {
    let mut s = scene();
    s.set_cell(cell(0, 0), slope_tile(3), 0);
    s.place_infantry(0, "TESTMAN", cell(0, 0), 4);
    let w = s.build();
    let p = pos(&w, men(&w)[0]);
    assert_eq!((p.x, p.y), at(cell(0, 0), position::BOTTOM));
    assert_eq!(p.z, w.surface_at(p.x, p.y));
}

// spec: S-0208 claim 3 (the desired point keeps the position within the cell; a free side class is kept)
#[test]
fn walker_keeps_its_side_position_into_a_free_cell() {
    for sub in [2, 3, 4] {
        let (w, id, _) = walk_east(&scene(), sub, 1);
        let p = pos(&w, id);
        assert_eq!((p.x, p.y), at(cell(1, 0), sub as u8), "sub {sub}");
    }
}

/// A walker at standing position `sub` of cell(0, 0) walks into cell(1, 0),
/// where a soldier of the same house stands at each of `taken`. Returns the
/// standing position it ends at in cell(1, 0).
fn enter_occupied(sub: i32, taken: &[i32]) -> u8 {
    enter_occupied_seeded(1, sub, taken)
}

/// [`enter_occupied`] with a game seed.
fn enter_occupied_seeded(seed: u32, sub: i32, taken: &[i32]) -> u8 {
    let mut s = scene();
    s.setup.seed = seed;
    s.place_infantry(0, "TESTMAN", cell(0, 0), sub);
    for &q in taken {
        s.place_infantry(0, "TESTMAN", cell(1, 0), q);
    }
    let mut w = s.build();
    let id = men(&w)[0];
    open_orders(&mut w);
    order_move(&mut w, id, cell(1, 0));
    let _ = trace(&mut w, id, 200);
    let p = pos(&w, id);
    assert_eq!(p.cell(), cell(1, 0), "notes: {:?}", w.notes);
    w.obj(id)
        .and_then(|o| o.walk.as_ref())
        .expect("walk")
        .standing
}

// spec: S-0208 claim 3 (a taken side class falls back along rows S2, S3 and S4 of table O)
#[test]
fn taken_side_class_falls_back_by_table_o() {
    use position::{BOTTOM, LEFT, RIGHT};
    // S2: right taken → bottom, then left.
    assert_eq!(enter_occupied(2, &[2]), BOTTOM);
    assert_eq!(enter_occupied(2, &[2, 4]), LEFT);
    // S3: left taken → bottom, then right.
    assert_eq!(enter_occupied(3, &[3]), BOTTOM);
    assert_eq!(enter_occupied(3, &[3, 4]), RIGHT);
    // S4: bottom taken → right, then left.
    assert_eq!(enter_occupied(4, &[4]), RIGHT);
    assert_eq!(enter_occupied(4, &[4, 2]), LEFT);
}

// spec: S-0208 claim 3 (centre class into an empty cell: right, left or bottom, right about half the time; reserved at once)
#[test]
fn centre_class_into_an_empty_cell() {
    let mut seen = [0u32; 5];
    for seed in 1..=64u32 {
        let mut s = scene();
        s.setup.seed = seed;
        s.place_infantry(0, "TESTMAN", cell(0, 0), 0);
        let mut w = s.build();
        let id = men(&w)[0];
        open_orders(&mut w);
        order_move(&mut w, id, cell(1, 0));
        let mut chosen = None;
        for _ in 0..20 {
            tick::step(&mut w);
            if walk_target(&w, id).is_some() {
                chosen = Some(w.obj(id).unwrap().walk.as_ref().unwrap().standing);
                break;
            }
        }
        let p = chosen.expect("a walking target");
        seen[usize::from(p)] += 1;
        // Reservation on the asking tick: the new position is taken, the
        // walker's old one (the centre of cell(0, 0)) is free.
        let layer = w.obj(id).unwrap().layer;
        assert!(w.cells.occ(cell(1, 0), layer).unwrap().taken(p));
        assert!(
            !w.cells
                .occ(cell(0, 0), layer)
                .unwrap()
                .taken(position::CENTRE)
        );
    }
    let [c, t, r, l, b] = seen;
    assert_eq!((c, t), (0, 0), "never the centre or the top");
    assert!(l > 0 && b > 0, "{seen:?}");
    assert!(
        r > l && r > b,
        "rows R0 and R1 both lead with right: {seen:?}"
    );
}

// spec: S-0208 claim 3 (centre class: the first free position of the drawn row)
#[test]
fn centre_class_takes_the_first_free_of_its_row() {
    use position::{BOTTOM, LEFT, RIGHT};
    // With two side positions taken every row leads to the third.
    for seed in [1u32, 2, 3, 4, 5, 6] {
        for (taken, want) in [([2, 3], BOTTOM), ([2, 4], LEFT), ([3, 4], RIGHT)] {
            assert_eq!(enter_occupied_seeded(seed, 0, &taken), want, "seed {seed}");
        }
    }
}

// spec: S-0208 claim 5 (sub-cell values 0 to 4 name the table P positions; 1 counts as the centre)
#[test]
fn pre_placed_sub_cell_values() {
    let mut s = scene();
    for sub in 0..=4 {
        s.place_infantry(0, "TESTMAN", cell(2 * sub, 0), sub);
    }
    let w = s.build();
    for (sub, id) in men(&w).into_iter().enumerate() {
        let c = cell(2 * sub as i32, 0);
        let p = pos(&w, id);
        assert_eq!((p.x, p.y), at(c, sub as u8), "sub {sub}");
        let rec = w.cells.occ(c, w.obj(id).unwrap().layer).unwrap();
        let mark = if sub == 1 {
            position::CENTRE
        } else {
            sub as u8
        };
        assert!(rec.taken(mark), "sub {sub}");
        if sub == 1 {
            assert!(!rec.taken(position::TOP));
        }
    }
}

// spec: S-0208 claim 5 (a value outside 0 to 4 is placed at the centre and the importer warns)
#[test]
fn pre_placed_sub_cell_out_of_range() {
    let mut s = scene();
    s.place_infantry(0, "TESTMAN", cell(0, 0), 7);
    let w = s.build();
    let p = pos(&w, men(&w)[0]);
    assert_eq!((p.x, p.y), at(cell(0, 0), position::CENTRE));
    assert!(w.notes.iter().any(|n| n.contains("S-0208 claim 5")));
}

// ---------------------------------------------------------------------------
// The walking step

// spec: S-0208 claim 6 (steps of s towards the target, arrival below 17, a non-moving asking tick per cell)
#[test]
fn walk_steps_and_ticks_per_cell() {
    let n = 3;
    let (w, id, t) = walk_east(&scene(), 2, n);
    assert_eq!(walk::walking_speed(&w, id), S);
    let p = pos(&w, id);
    assert_eq!((p.x, p.y), at(cell(n, 0), position::RIGHT));
    // Every step that does not arrive moves s leptons due map east.
    let st = steps(&t);
    assert!(!st.is_empty());
    assert!(st.iter().all(|&d| d == (S, 0)), "{st:?}");
    // Each arrival happens from less than 17 leptons and lands exactly on
    // the target; the walker then has no walking target.
    for k in t.iter().filter(|k| k.had_target && !k.has_target) {
        let (u, v) = (k.after.x.rem_euclid(256), k.after.y.rem_euclid(256));
        assert_eq!((u, v), walk::standing_point(position::RIGHT));
        let d = (k.after.x - k.before.x).abs() + (k.after.y - k.before.y).abs();
        assert!(d < 17, "arrived from {d} leptons");
    }
    // The tick that sets a walking target does not move the walker.
    for k in t.iter().filter(|k| !k.had_target && k.has_target) {
        assert_eq!(k.before, k.after);
    }
    // n × (⌈240 / s⌉ + 2) − 1 ticks from the first step to the arrival.
    let per_cell = (240 + S - 1) / S + 2;
    assert_eq!(walk_span(&t), n * per_cell - 1);
}

// spec: S-0208 claim 6 (arrival as soon as the distance is below 17 leptons, not before)
#[test]
fn arrival_threshold_is_17_leptons() {
    // Speed field 7: V = ⌊256 × 7 / 100⌋ = 17. From one right position to
    // the next is 256 leptons: after 14 steps the walker is 18 away, so it
    // steps once more (to 1 away) and then arrives.
    let mut s = Scene::new();
    s.infantry("TESTMAN", unit_type(FOOT, 7, 0));
    let (w, id, t) = walk_east(&s, 2, 1);
    assert_eq!(walk::walking_speed(&w, id), 17);
    assert_eq!(steps(&t).len(), 15);
    let arrive = t.iter().find(|k| k.had_target && !k.has_target).unwrap();
    assert_eq!(arrive.after.x - arrive.before.x, 1);
}

// spec: S-0208 claim 7 (base speed times the country factor times the speed multiplier, read on every step)
#[test]
fn walking_speed_factors() {
    // Country factor 0.5: s = trunc(10 × 0.5) = 5.
    let mut s = scene();
    s.ruleset
        .sim
        .countries
        .get_mut("Alphaland")
        .unwrap()
        .speed_infantry_mult = Some(dec("0.5"));
    s.place_infantry(0, "TESTMAN", cell(0, 0), 2);
    s.place_infantry(1, "TESTMAN", cell(0, 4), 2);
    let mut w = s.build();
    let (a, b) = (men(&w)[0], men(&w)[1]);
    assert_eq!(walk::walking_speed(&w, a), 5);
    // The other country keeps the factor 1.0.
    assert_eq!(walk::walking_speed(&w, b), S);
    // Speed multiplier 1.5: s = trunc((10 × 1.0) × 1.5) = 15.
    let mode = w.rules().mode;
    let m = F64::from_i64(3, mode).div(F64::from_i64(2, mode), mode);
    w.obj_mut(b).unwrap().speed_bonus = m;
    assert_eq!(walk::walking_speed(&w, b), 15);
    // Both: trunc((10 × 0.5) × 1.5) = 7.
    w.obj_mut(a).unwrap().speed_bonus = m;
    assert_eq!(walk::walking_speed(&w, a), 7);
}

// spec: S-0208 claim 7 (a change of the speed multiplier shows from the next step)
#[test]
fn speed_change_shows_from_the_next_step() {
    let mut s = scene();
    s.place_infantry(0, "TESTMAN", cell(0, 0), 2);
    let mut w = s.build();
    let id = men(&w)[0];
    open_orders(&mut w);
    order_move(&mut w, id, cell(3, 0));
    // Walk until the first step was taken.
    let mut first = None;
    for _ in 0..40 {
        let b = pos(&w, id);
        tick::step(&mut w);
        let a = pos(&w, id);
        if a != b {
            first = Some(a.x - b.x);
            break;
        }
    }
    assert_eq!(first, Some(S));
    let mode = w.rules().mode;
    w.obj_mut(id).unwrap().speed_bonus = F64::from_i64(2, mode);
    let b = pos(&w, id);
    tick::step(&mut w);
    assert_eq!(pos(&w, id).x - b.x, 2 * S);
}

// spec: S-0208 claim 8 (prone: a crawler moves s − ⌊s/3⌋, a non-crawler s + ⌊s/2⌋)
#[test]
fn prone_speeds() {
    let mut s = scene();
    let mut stiff = soldier();
    stiff.crawls = Some(false);
    s.infantry("TESTSTIFF", stiff);
    s.place_infantry(0, "TESTMAN", cell(0, 0), 2);
    s.place_infantry(0, "TESTSTIFF", cell(0, 4), 2);
    let mut w = s.build();
    let (a, b) = (men(&w)[0], men(&w)[1]);
    assert_eq!(walk::walking_speed(&w, a), S);
    assert_eq!(walk::walking_speed(&w, b), S);
    for id in [a, b] {
        w.obj_mut(id).unwrap().infantry.as_mut().unwrap().prone = true;
    }
    assert_eq!(walk::walking_speed(&w, a), S - S / 3);
    assert_eq!(walk::walking_speed(&w, b), S + S / 2);
}

// spec: S-0208 claim 8 (a prone crawler's steps are s − ⌊s/3⌋ long)
#[test]
fn crawling_walker_steps_slower() {
    let mut s = scene();
    s.place_infantry(0, "TESTMAN", cell(0, 0), 2);
    let mut w = s.build();
    let id = men(&w)[0];
    open_orders(&mut w);
    w.obj_mut(id).unwrap().infantry.as_mut().unwrap().prone = true;
    order_move(&mut w, id, cell(1, 0));
    let t = trace(&mut w, id, 200);
    let st = steps(&t);
    assert!(!st.is_empty());
    assert!(st.iter().all(|&d| d == (S - S / 3, 0)), "{st:?}");
}

/// The ticks a soldier takes to walk 3 cells due map east over cells of
/// terrain code `code`, with land row `row` for road.
fn ticks_over(code: u8, road: LandRow) -> (i32, Vec<(i32, i32)>) {
    let mut s = scene();
    s.ruleset.sim.land_types.insert(LandType::Road, road);
    for dx in -1..=4 {
        for dy in -1..=1 {
            s.set_cell(cell(dx, dy), code_tile(code), 0);
        }
    }
    let (_, _, t) = walk_east(&s, 2, 3);
    (walk_span(&t), steps(&t))
}

// spec: S-0208 claim 9 (a walker speed percentage other than zero leaves the walking speed unchanged)
#[test]
fn terrain_percentage_does_not_slow_walkers() {
    // Code 11 is a road tile (S-0211 claim 6), code 0 clear.
    let half = LandRow {
        foot: Some(dec("0.5")),
        ..Default::default()
    };
    let (road, road_steps) = ticks_over(11, half);
    let (clear, _) = ticks_over(0, LandRow::default());
    assert_eq!(road, clear);
    assert!(road_steps.iter().all(|&d| d == (S, 0)), "{road_steps:?}");
}

// spec: S-0208 claim 9 (a percentage of exactly zero makes the cell impassable)
#[test]
fn zero_percentage_blocks_walkers() {
    let mut s = scene();
    s.ruleset.sim.land_types.insert(
        LandType::Road,
        LandRow {
            foot: Some(dec("0")),
            ..Default::default()
        },
    );
    s.set_cell(cell(1, 0), code_tile(11), 0);
    s.place_infantry(0, "TESTMAN", cell(0, 0), 2);
    let w = s.build();
    let id = men(&w)[0];
    let r = occupancy::reference_level(&w, id);
    assert_eq!(
        occupancy::infantry_outcome(&w, id, cell(1, 0), r),
        Outcome::Blocked
    );
    assert_eq!(
        occupancy::infantry_outcome(&w, id, cell(0, 1), r),
        Outcome::Clear
    );
}

// spec: S-0208 claim 10 (a panicking walker keeps the walking speed)
#[test]
fn panic_does_not_change_speed() {
    let mut s = scene();
    let mut cat = soldier();
    cat.fraidycat = Some(true);
    s.infantry("TESTCAT", cat);
    s.place_infantry(0, "TESTCAT", cell(0, 0), 2);
    let mut w = s.build();
    let id = men(&w)[0];
    open_orders(&mut w);
    {
        let st = w.obj_mut(id).unwrap().infantry.as_mut().unwrap();
        st.fear = 250;
        st.seq = Some(Seq::Panic);
    }
    assert_eq!(walk::walking_speed(&w, id), S);
    order_move(&mut w, id, cell(1, 0));
    let t = trace(&mut w, id, 200);
    let st = steps(&t);
    assert!(!st.is_empty());
    assert!(st.iter().all(|&d| d == (S, 0)), "{st:?}");
    assert!(w.obj(id).unwrap().infantry.as_ref().unwrap().fear > 50);
}

// ---------------------------------------------------------------------------
// Scatter acceptance

/// A scene with the soldier, a fraidycat TESTCAT (with deploy sequences),
/// and the scatter property on guard and move.
fn scatter_scene(player_scatter: bool) -> Scene {
    let mut s = scene();
    let mut cat = soldier();
    cat.fraidycat = Some(true);
    s.infantry("TESTCAT", cat);
    let mut dep = soldier();
    for k in ["undeploy", "deploy", "deployed"] {
        dep.sequences.insert(
            k.into(),
            SequenceEntry {
                start: 0,
                count: 3,
                multiplier: 0,
                direction: None,
            },
        );
    }
    s.infantry("TESTDEP", dep);
    let props = BehaviourProps {
        scatter: Some(true),
        ..Default::default()
    };
    s.ruleset
        .sim
        .behaviours
        .insert("guard".into(), props.clone());
    s.ruleset.sim.behaviours.insert("move".into(), props);
    let no_scatter = BehaviourProps {
        scatter: Some(false),
        ..Default::default()
    };
    s.ruleset.sim.behaviours.insert("sleep".into(), no_scatter);
    s.ruleset.sim.general.player_scatter = Some(player_scatter);
    s
}

fn place(s: &mut Scene, ty: &str, c: CellXY, sub: i32) {
    s.place_infantry(0, ty, c, sub);
}

const FORCED: ScatterRequest = ScatterRequest {
    threat: None,
    forced: true,
    anywhere: false,
};
const FORCED_ANYWHERE: ScatterRequest = ScatterRequest {
    threat: None,
    forced: true,
    anywhere: true,
};
const PLAIN: ScatterRequest = ScatterRequest {
    threat: None,
    forced: false,
    anywhere: false,
};
const ANYWHERE: ScatterRequest = ScatterRequest {
    threat: None,
    forced: false,
    anywhere: true,
};

/// Sends `req` to the single infantry unit of a fresh world after
/// `setup` adjusted it; returns (accepted, draws taken, new destination).
fn send(
    s: &Scene,
    req: ScatterRequest,
    setup: impl FnOnce(&mut World, ObjectId),
) -> (bool, u64, Option<Target>) {
    let mut w = s.build();
    let id = men(&w)[0];
    open_orders(&mut w);
    setup(&mut w, id);
    let before = w.rng.taken();
    let ok = scatter::request(&mut w, id, req);
    (ok, w.rng.taken() - before, w.obj(id).unwrap().dest)
}

fn set_seq(w: &mut World, id: ObjectId, q: Seq) {
    w.obj_mut(id).unwrap().infantry.as_mut().unwrap().seq = Some(q);
}

fn set_guard(w: &mut World, id: ObjectId) {
    w.obj_mut(id).unwrap().beh.current = Some(Behaviour::Guard);
}

// spec: S-0208 claim 11 (a forced request moves an idle soldier; one not effectively forced does not)
#[test]
fn scatter_forced_and_not_forced() {
    let mut s = scatter_scene(true);
    place(&mut s, "TESTMAN", cell(0, 0), 2);
    let (ok, draws, dest) = send(&s, FORCED, set_guard);
    assert!(ok);
    assert!(draws > 0);
    assert!(dest.is_some());
    // Not forced, not a fraidycat: refused, no draw, no destination, even
    // with the player-scatter rule on and move-anywhere.
    for req in [PLAIN, ANYWHERE] {
        let (ok, draws, dest) = send(&s, req, set_guard);
        assert!(!ok);
        assert_eq!(draws, 0);
        assert_eq!(dest, None);
    }
}

// spec: S-0208 claim 11 (a forced request to a unit whose locomotor is moving is not effectively forced)
#[test]
fn scatter_forced_while_moving_is_refused() {
    let mut s = scatter_scene(true);
    place(&mut s, "TESTMAN", cell(0, 0), 2);
    let (ok, draws, dest) = send(&s, FORCED, |w, id| {
        order_move(w, id, cell(6, 0));
        common::run(w, 10);
        assert!(occupancy::in_progress(w, id));
    });
    assert!(!ok);
    assert_eq!(draws, 0);
    assert_eq!(dest, Some(Target::Cell(cell(6, 0))));
}

// spec: S-0208 claim 11 (first case: a sequence that is neither undeploy nor interruptible refuses)
#[test]
fn scatter_first_case_sequence() {
    let mut s = scatter_scene(true);
    place(&mut s, "TESTMAN", cell(0, 0), 2);
    // Cheer is not interruptible: refused even when forced and move-anywhere.
    let (ok, draws, _) = send(&s, FORCED_ANYWHERE, |w, id| set_seq(w, id, Seq::Cheer));
    assert!(!ok);
    assert_eq!(draws, 0);
    // Get-up is not interruptible either.
    let (ok, _, _) = send(&s, FORCED, |w, id| set_seq(w, id, Seq::GetUp));
    assert!(!ok);
    // Undeploy is not interruptible but exempt; idle A is interruptible.
    for q in [Seq::Undeploy, Seq::IdleA] {
        let (ok, draws, _) = send(&s, FORCED, |w, id| set_seq(w, id, q));
        assert!(ok, "{q:?}");
        assert!(draws > 0);
    }
}

// spec: S-0208 claim 11 (second case: a deploying unit refuses unless the request is forced and move-anywhere)
#[test]
fn scatter_second_case_deploying() {
    let mut s = scatter_scene(true);
    place(&mut s, "TESTDEP", cell(0, 0), 2);
    // Deployed is interruptible, so only the second case can refuse.
    let (ok, _, _) = send(&s, FORCED, |w, id| set_seq(w, id, Seq::Deployed));
    assert!(!ok);
    // Forced and move-anywhere: the unit is also asked to undeploy, which
    // starts, and the scatter is accepted.
    let mut w = s.build();
    let id = men(&w)[0];
    open_orders(&mut w);
    set_seq(&mut w, id, Seq::Deployed);
    assert!(scatter::request(&mut w, id, FORCED_ANYWHERE));
    let seq = w.obj(id).unwrap().infantry.as_ref().unwrap().seq;
    assert_eq!(seq, Some(Seq::Undeploy));
}

// spec: S-0208 claim 11 (undeploy table: deploy keeps playing and refuses; deployed without undeploy is accepted)
#[test]
fn scatter_undeploy_table() {
    let mut s = scatter_scene(true);
    place(&mut s, "TESTDEP", cell(0, 0), 2);
    // Deploy is not interruptible: the non-forced undeploy request is turned
    // down, the unit keeps playing deploy and the first case refuses.
    let mut w = s.build();
    let id = men(&w)[0];
    open_orders(&mut w);
    set_seq(&mut w, id, Seq::Deploy);
    assert!(!scatter::request(&mut w, id, FORCED_ANYWHERE));
    let seq = w.obj(id).unwrap().infantry.as_ref().unwrap().seq;
    assert_eq!(seq, Some(Seq::Deploy));
    // A type without an undeploy sequence: undeploy is turned down, the
    // unit keeps playing deployed (interruptible) and accepts.
    let mut s = scatter_scene(true);
    place(&mut s, "TESTMAN", cell(0, 0), 2);
    let (ok, draws, _) = send(&s, FORCED_ANYWHERE, |w, id| set_seq(w, id, Seq::Deployed));
    assert!(ok);
    assert!(draws > 0);
}

// spec: S-0208 claim 11 (third case: a request not effectively forced moves only a permitted fraidycat in a scatter behaviour)
#[test]
fn scatter_third_case_fraidycat() {
    // Player-scatter rule on: permission.
    let mut s = scatter_scene(true);
    place(&mut s, "TESTCAT", cell(0, 0), 2);
    let (ok, draws, dest) = send(&s, PLAIN, set_guard);
    assert!(ok);
    assert!(draws > 0);
    assert!(dest.is_some());
    // Without the scatter property of its behaviour: refused.
    let (ok, _, _) = send(&s, PLAIN, |w, id| {
        w.obj_mut(id).unwrap().beh.current = Some(Behaviour::Sleep);
    });
    assert!(!ok);
    // Player-scatter rule off: refused, unless the request is move-anywhere.
    let mut s = scatter_scene(false);
    place(&mut s, "TESTCAT", cell(0, 0), 2);
    let (ok, draws, _) = send(&s, PLAIN, set_guard);
    assert!(!ok);
    assert_eq!(draws, 0);
    let (ok, _, _) = send(&s, ANYWHERE, set_guard);
    assert!(ok);
}

// spec: S-0208 claim 11 (an owner that is not human-controlled, such as the neutral or special house, gives scatter permission and escapes the second case; an AI house counts as human-controlled)
#[test]
fn scatter_computer_controlled_owner() {
    let computer = |w: &mut World, id: ObjectId| {
        let h = w.obj(id).unwrap().owner.unwrap();
        w.houses[usize::from(h.0)].human = false;
    };
    // Third case: player-scatter rule off, a fraidycat of a computer-controlled
    // owner in a scatter behaviour accepts a request that is not forced.
    let mut s = scatter_scene(false);
    place(&mut s, "TESTCAT", cell(0, 0), 2);
    let (ok, draws, dest) = send(&s, PLAIN, |w, id| {
        computer(w, id);
        set_guard(w, id);
    });
    assert!(ok);
    assert!(draws > 0);
    assert!(dest.is_some());
    // Second case: a deployed unit of a computer-controlled owner accepts a
    // forced request that is not move-anywhere.
    let mut s = scatter_scene(true);
    place(&mut s, "TESTDEP", cell(0, 0), 2);
    let (ok, draws, _) = send(&s, FORCED, |w, id| {
        computer(w, id);
        set_seq(w, id, Seq::Deployed);
    });
    assert!(ok);
    assert!(draws > 0);
}

// ---------------------------------------------------------------------------
// Scatter destinations

/// A scene for the neighbour choice: a soldier at the centre of cell(0, 0);
/// every neighbour except those in `open` is a road cell that walkers
/// cannot enter.
fn neighbour_scene(open: &[usize]) -> Scene {
    let mut s = scatter_scene(true);
    s.ruleset.sim.land_types.insert(
        LandType::Road,
        LandRow {
            foot: Some(dec("0")),
            ..Default::default()
        },
    );
    for (d, &(dx, dy)) in STEPS.iter().enumerate() {
        if !open.contains(&d) {
            s.set_cell(cell(dx, dy), code_tile(11), 0);
        }
    }
    place(&mut s, "TESTMAN", cell(0, 0), 0);
    s
}

/// Sends a forced request with a threat point at offset (tx, ty) from the
/// unit; returns (the draw j it should take, the destination cell).
fn threat_scatter(w: &mut World, id: ObjectId, tx: i32, ty: i32) -> (i32, Option<CellXY>) {
    let p = pos(w, id);
    let mut r = w.rng.clone();
    let j = r.ranged(0, 4, half_span(w));
    let req = ScatterRequest {
        threat: Some(Pos::new(p.x + tx, p.y + ty, p.z)),
        forced: true,
        anywhere: false,
    };
    assert!(scatter::request(w, id, req));
    let dest = match w.obj(id).unwrap().dest {
        Some(Target::Cell(c)) => Some(c),
        _ => None,
    };
    (j, dest)
}

fn neighbour(c: CellXY, d: usize) -> CellXY {
    let (dx, dy) = STEPS[d % 8];
    CellXY::new(c.x + dx, c.y + dy)
}

// spec: S-0208 claim 12 (first choice d = d₀ + j − 2 from the facing away from the threat; then d + 1, … in order)
#[test]
fn threat_scatter_direction_and_preference() {
    // Open ground: the destination is the neighbour in direction d.
    for seed in 1..=10u32 {
        let mut s = neighbour_scene(&[0, 1, 2, 3, 4, 5, 6, 7]);
        s.setup.seed = seed;
        let mut w = s.build();
        let id = men(&w)[0];
        open_orders(&mut w);
        set_guard(&mut w, id);
        // Threat to map west: d₀ = 2 (map east), so d = j.
        let (j, dest) = threat_scatter(&mut w, id, -600, 0);
        assert_eq!(dest, Some(neighbour(cell(0, 0), j as usize)), "seed {seed}");
    }
    // Only directions 1 and 6 enterable: d ≤ 1 gives 1, d from 2 to 4
    // passes 2 … 5 and gives 6.
    let mut both = [false; 2];
    for seed in 1..=16u32 {
        let mut s = neighbour_scene(&[1, 6]);
        s.setup.seed = seed;
        let mut w = s.build();
        let id = men(&w)[0];
        open_orders(&mut w);
        let (j, dest) = threat_scatter(&mut w, id, -600, 0);
        let want = if j <= 1 { 1 } else { 6 };
        assert_eq!(
            dest,
            Some(neighbour(cell(0, 0), want)),
            "seed {seed}, j {j}"
        );
        both[usize::from(want == 6)] = true;
    }
    assert!(both[0] && both[1], "both outcomes seen");
    // Threat to map north: d₀ = 4, d = j + 2 from 2 to 6: always 6.
    let mut s = neighbour_scene(&[1, 6]);
    s.setup.seed = 3;
    let mut w = s.build();
    let id = men(&w)[0];
    open_orders(&mut w);
    let (_, dest) = threat_scatter(&mut w, id, 0, -600);
    assert_eq!(dest, Some(neighbour(cell(0, 0), 6)));
}

// spec: S-0208 claim 12 (no qualifying neighbour: accepted, but no destination)
#[test]
fn threat_scatter_with_no_qualifying_neighbour() {
    let s = neighbour_scene(&[]);
    let mut w = s.build();
    let id = men(&w)[0];
    open_orders(&mut w);
    let (_, dest) = threat_scatter(&mut w, id, -600, 0);
    assert_eq!(dest, None);
}

// spec: S-0208 claim 12 (a candidate failing the screen-cover check is passed over for one that passes)
#[test]
fn threat_scatter_screen_cover() {
    // Directions 1 and 6 open. The cell one step towards the bottom of the
    // screen from the direction-6 neighbour, cell(0, 1), is raised by one
    // level, so that neighbour fails the screen-cover check.
    let mut s = neighbour_scene(&[1, 6]);
    s.set_cell(cell(0, 1), code_tile(11), 1);
    let mut w = s.build();
    let id = men(&w)[0];
    // Both open neighbours qualify; only the screen-cover check tells them
    // apart.
    let r = occupancy::reference_level(&w, id);
    for d in [1, 6] {
        let c = neighbour(cell(0, 0), d);
        assert_eq!(occupancy::infantry_outcome(&w, id, c, r), Outcome::Clear);
    }
    assert!(!scatter::screen_cover_passes(&w, neighbour(cell(0, 0), 6)));
    assert!(scatter::screen_cover_passes(&w, neighbour(cell(0, 0), 1)));
    open_orders(&mut w);
    // Threat to map north: d from 2 to 6, so 6 comes first but fails; 1
    // follows (7, 0 are closed) and passes.
    let (_, dest) = threat_scatter(&mut w, id, 0, -600);
    assert_eq!(dest, Some(neighbour(cell(0, 0), 1)), "notes: {:?}", w.notes);
}

// spec: S-0208 claim 12 (the candidates surround the scatter centre cell, the cell of a walking unit's destination)
#[test]
fn threat_scatter_around_the_locomotor_destination() {
    let mut s = scatter_scene(true);
    place(&mut s, "TESTCAT", cell(0, 0), 2);
    let mut w = s.build();
    let id = men(&w)[0];
    open_orders(&mut w);
    let goal = cell(8, 0);
    order_move(&mut w, id, goal);
    common::run(&mut w, 6);
    assert!(occupancy::in_progress(&w, id));
    let p = pos(&w, id);
    let mut r = w.rng.clone();
    let j = r.ranged(0, 4, half_span(&w));
    // Not forced: the fraidycat accepts it in move (scatter property on).
    let req = ScatterRequest {
        threat: Some(Pos::new(p.x - 600, p.y, p.z)),
        forced: false,
        anywhere: false,
    };
    assert!(scatter::request(&mut w, id, req));
    // d₀ from the threat (to map west of the unit) to the unit: map east.
    let d = (2 + j - 2).rem_euclid(8) as usize;
    assert_eq!(
        w.obj(id).unwrap().dest,
        Some(Target::Cell(neighbour(goal, d)))
    );
}

// spec: S-0208 claim 13 (no threat point: one draw, the nearby-cell search gives a neighbour, independent of the draw)
#[test]
fn scatter_without_threat_uses_the_search() {
    let mut s = scatter_scene(true);
    place(&mut s, "TESTMAN", cell(0, 0), 2);
    let mut w = s.build();
    let id = men(&w)[0];
    open_orders(&mut w);
    let mut dests = Vec::new();
    for _ in 0..8 {
        let mut a = w.clone();
        let mut b = w.clone();
        b.rng = Rng::new(0x5eed);
        let ta = a.rng.taken();
        assert!(scatter::request(&mut a, id, FORCED));
        assert!(a.rng.taken() > ta, "a draw is taken");
        assert!(scatter::request(&mut b, id, FORCED));
        let da = a.obj(id).unwrap().dest;
        assert_eq!(da, b.obj(id).unwrap().dest, "independent of the draw");
        let Some(Target::Cell(c)) = da else {
            panic!("no destination")
        };
        assert!(
            (0..8).any(|d| neighbour(cell(0, 0), d) == c),
            "{c:?} is a neighbour"
        );
        dests.push(c);
        tick::step(&mut w);
    }
    // Requests on eight consecutive ticks (N mod 8) give eight different
    // neighbours.
    dests.sort_by_key(|c| (c.x, c.y));
    dests.dedup();
    assert_eq!(dests.len(), 8, "{dests:?}");
}

/// A soldier at standing position `sub` of cell(0, 0) is ordered to enter a
/// unit of its house in cell(1, 0): a vehicle (`vehicle`), or else a soldier
/// standing at the right position. Returns the standing position of its
/// walking target there.
fn enter_mission(sub: i32, vehicle: bool) -> u8 {
    use glasnost_sim::orders::{self, Order};
    use glasnost_sim::world::HouseId;
    let mut s = scene();
    s.vehicle("TESTVAN", unit_type(common::WHEEL, 4, 6));
    s.place_infantry(0, "TESTMAN", cell(0, 0), sub);
    if vehicle {
        s.place_unit(0, "TESTVAN", cell(1, 0), 64);
    } else {
        s.place_infantry(0, "TESTMAN", cell(1, 0), 2);
    }
    let mut w = s.build();
    let id = men(&w)[0];
    let friend = if vehicle {
        common::objects_of(&w, Kind::Vehicle)[0]
    } else {
        men(&w)[1]
    };
    open_orders(&mut w);
    let o = Order::Behaviour {
        actor: id,
        behaviour: Behaviour::Enter,
        attack_move: false,
        target: None,
        dest: Some(Target::Object(friend)),
    };
    assert!(orders::issue(&mut w, HouseId(0), o));
    for _ in 0..20 {
        tick::step(&mut w);
        if let Some(t) = walk_target(&w, id) {
            assert_eq!(t.cell(), cell(1, 0));
            assert_eq!(w.obj(id).unwrap().beh.current, Some(Behaviour::Enter));
            let p = w.obj(id).unwrap().walk.as_ref().unwrap().standing;
            assert_eq!((t.x, t.y), at(cell(1, 0), p));
            return p;
        }
    }
    panic!("no walking target; notes: {:?}", w.notes);
}

// spec: S-0208 claim 4 (into the cell of a mission target that is a vehicle the walker heads for its class position, occupants ignored; a mission target that is an infantry unit does not make the case)
#[test]
fn mission_target_cell_ignores_occupants() {
    // Right class into the vehicle's cell (vehicle present): still right.
    assert_eq!(enter_mission(2, true), position::RIGHT);
    // Centre class: the centre position, which table O never gives.
    assert_eq!(enter_mission(0, true), position::CENTRE);
    // A mission target that is an infantry unit standing at the right
    // position: claim 3 applies and row S2 gives the bottom position.
    assert_eq!(enter_mission(2, false), position::BOTTOM);
}

/// spec: S-0208 Edge cases (quirks.top_position_counts_as_centre: once the unit sharing the record's centre entry leaves, the record shows the centre free; the remaining unit, standing still, still keeps an allied vehicle out through its own row, so the switch changes nothing for a vehicle); S-0209 claim 5 (row VP20)
#[test]
fn remaining_top_unit_keeps_allied_vehicle_out_either_way() {
    for quirk in [true, false] {
        let mut s = scene();
        s.vehicle("CART", unit_type(common::TRACK, 6, 10));
        s.place_unit(0, "CART", cell(0, 0), 64);
        s.ruleset.globals.quirks.top_position_counts_as_centre = quirk;
        let mut w = s.build();
        let ty = w.rules().index_of(Kind::Infantry, "TESTMAN").unwrap();
        let c = cell(1, 0);
        let put = |w: &mut World, p: u8| {
            let id = glasnost_sim::objects::create(
                w,
                Kind::Infantry,
                ty,
                Some(glasnost_sim::world::HouseId(0)),
            );
            let (x, y) = at(c, p);
            glasnost_sim::objects::place(w, id, Pos::new(x, y, 0), Default::default());
            id
        };
        let _top = put(&mut w, position::TOP);
        let centre = put(&mut w, position::CENTRE);
        glasnost_sim::objects::remove(&mut w, centre);
        let occ = *w.cells.occ(c, glasnost_sim::world::Layer::Ground).unwrap();
        // With the switch on, the record lost the shared centre entry.
        assert_eq!(
            occ.taken(position::CENTRE) || occ.taken(position::TOP),
            !quirk
        );
        let cart = common::objects_of(&w, Kind::Vehicle)[0];
        let out = occupancy::vehicle_outcome(&w, cart, c, occupancy::reference_level(&w, cart));
        assert_eq!(out, Outcome::StationaryFriendly, "switch {quirk}");
    }
}
