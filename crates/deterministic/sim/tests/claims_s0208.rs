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
    // The walk is under way before the fear is raised: a frightened
    // fraidycat at rest flees (S-0107 claim 15), at the start and on
    // arrival, so only the ordered walk is traced.
    order_move(&mut w, id, cell(1, 0));
    tick::step(&mut w);
    assert!(w.obj(id).unwrap().dest.is_some());
    {
        let st = w.obj_mut(id).unwrap().infantry.as_mut().unwrap();
        st.fear = 250;
        st.seq = Some(Seq::Panic);
    }
    assert_eq!(walk::walking_speed(&w, id), S);
    let mut t = Vec::new();
    for _ in 0..200 {
        let k = trace(&mut w, id, 1);
        if w.obj(id).unwrap().dest != Some(Target::Cell(cell(1, 0))) {
            break;
        }
        t.extend(k);
    }
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

/// A fraidycat walking from cell(0, 0) to cell(8, 0), stopped once `pick`
/// holds after a tick; sends it a threat request from map west and returns
/// (the cell its centre must be, the destination it got).
fn scatter_while_walking(pick: impl Fn(&World, ObjectId) -> bool) -> (CellXY, Option<Target>) {
    let mut s = scatter_scene(true);
    place(&mut s, "TESTCAT", cell(0, 0), 2);
    let mut w = s.build();
    let id = men(&w)[0];
    open_orders(&mut w);
    order_move(&mut w, id, cell(8, 0));
    for _ in 0..200 {
        tick::step(&mut w);
        if pick(&w, id) {
            break;
        }
    }
    assert!(pick(&w, id), "the walk reached the wanted state");
    let p = pos(&w, id);
    // spec: S-0208 Terms (the scatter reference point: the walking target while there is one, else the unit's own position)
    let centre = walk_target(&w, id).map_or(p.cell(), |t| t.cell());
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
    (neighbour(centre, d), w.obj(id).unwrap().dest)
}

// spec: S-0208 Terms and claim 12 (the scatter centre cell is the cell of the walking target while there is one, never the cell of a farther destination)
#[test]
fn threat_scatter_around_the_walking_target() {
    let (want, got) = scatter_while_walking(|w, id| {
        walk_target(w, id).is_some_and(|t| t.cell() != pos(w, id).cell())
    });
    assert_eq!(got, Some(Target::Cell(want)));
    assert_ne!(want, neighbour(cell(8, 0), 2), "not around the destination");
}

// spec: S-0208 Terms and claim 12 (without a walking target, on an arrival or asking tick, the scatter centre cell is the unit's own cell)
#[test]
fn threat_scatter_around_the_own_cell_between_steps() {
    let (want, got) = scatter_while_walking(|w, id| {
        let o = w.obj(id).unwrap();
        o.dest.is_some() && walk_target(w, id).is_none() && o.pos.cell() != cell(0, 0)
    });
    assert_eq!(got, Some(Target::Cell(want)));
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

// ---------------------------------------------------------------------------
// End of a walk and the approaching walker (claims 16 and 17)

fn fraction(w: &World, id: ObjectId) -> bool {
    w.obj(id)
        .and_then(|o| o.walk.as_ref())
        .is_some_and(|s| s.fraction)
}

// spec: S-0208 claim 16 (the walk ends on the arrival tick in the destination's cell: no destination, speed fraction 0, standing where it arrived)
#[test]
fn walk_ends_on_the_arrival_tick_in_the_destination_cell() {
    let s = scene();
    let (w, id, t) = walk_east(&s, 0, 3);
    let last = t.last().expect("ticks");
    // The trace stops on the first tick that ends with no destination and
    // no walking target: that is the arrival tick itself.
    assert!(last.had_target && !last.has_target, "an arrival tick");
    assert_eq!(last.after.cell(), cell(3, 0));
    let o = w.obj(id).unwrap();
    assert_eq!(o.dest, None);
    assert!(o.path.steps.is_empty());
    assert!(!fraction(&w, id));
    assert!(!occupancy::in_progress(&w, id));
}

// spec: S-0310 Terms (the infantry speed fraction is 1 from a walking step until the walker stops, also over arrival ticks of a walk that continues)
#[test]
fn speed_fraction_while_walking() {
    let mut s = scene();
    s.place_infantry(0, "TESTMAN", cell(0, 0), 0);
    let mut w = s.build();
    let id = men(&w)[0];
    open_orders(&mut w);
    assert!(!fraction(&w, id));
    order_move(&mut w, id, cell(4, 0));
    let mut stepped = false;
    for _ in 0..400 {
        let before = pos(&w, id);
        tick::step(&mut w);
        if pos(&w, id) != before {
            stepped = true;
        }
        if w.obj(id).unwrap().dest.is_none() {
            break;
        }
        if stepped {
            assert!(fraction(&w, id), "1 from the first step on");
        }
    }
    assert!(stepped);
    assert!(!fraction(&w, id));
}

/// The synthetic combat rules with the WALKER armed with the 4-cell Rifle.
fn armed_rules() -> glasnost_rules::schema::Ruleset {
    let mut r = glasnost_sim::testkit::combat_ruleset();
    r.sim.infantry.get_mut("WALKER").unwrap().primary = Some("Rifle".into());
    r
}

/// A WALKER of house 0 at the middle cell, whose target is a CART of house
/// 1 `tx` cells map east, in behaviour `b`, with a destination `dx` cells
/// east. Returns (world, walker, cart).
fn approach_duel(b: Behaviour, tx: i32, dx: i32) -> (World, ObjectId, ObjectId) {
    use glasnost_sim::testkit;
    let mut w = testkit::game_with(&armed_rules(), 32, 3);
    let a = testkit::spawn(
        &mut w,
        Kind::Infantry,
        "WALKER",
        0,
        testkit::mid(32, 0, 0),
        0,
    );
    let t = testkit::spawn(&mut w, Kind::Vehicle, "CART", 1, testkit::mid(32, tx, 0), 0);
    let o = w.obj_mut(a).unwrap();
    o.target = Some(Target::Object(t));
    o.beh.current = Some(b);
    o.dest = Some(Target::Cell(testkit::mid(32, dx, 0)));
    (w, a, t)
}

/// Runs the duel until the walker has no destination; returns, for each
/// arrival tick, (the target in range at its cell centre, the destination
/// kept); the last entry is the tick it stopped on.
fn run_approach(w: &mut World, a: ObjectId) -> Vec<(bool, bool)> {
    let mut out = Vec::new();
    for _ in 0..600 {
        let arrived = w.obj(a).unwrap().walk.as_ref().unwrap().arrived;
        tick::step(w);
        let o = w.obj(a).unwrap();
        if o.walk.as_ref().unwrap().arrived != arrived {
            let Some(Target::Object(t)) = o.target else {
                panic!("the target is kept");
            };
            out.push((walk::walker_in_range(w, a, t), o.dest.is_some()));
        }
        if w.obj(a).unwrap().dest.is_none() {
            break;
        }
    }
    out
}

// spec: S-0208 claim 17 (on an arrival tick, a walker in attack or area guard whose mobile target is in range at its cell centre loses its destination and keeps its target; the walk ends there, claim 16)
#[test]
fn approaching_walker_stops_in_range() {
    for b in [Behaviour::Attack, Behaviour::AreaGuard] {
        // The target 10 cells east, the destination 9 cells east: the
        // Rifle's 4 cells are reached on the way.
        let (mut w, a, t) = approach_duel(b, 10, 9);
        let arrivals = run_approach(&mut w, a);
        let (&(in_range, kept), before) = arrivals.split_last().expect("arrivals");
        assert!(
            in_range && !kept,
            "{b:?}: stopped on an in-range arrival: {arrivals:?} {:?}",
            w.obj(a).map(|o| (o.dest, o.target, o.beh.current))
        );
        assert!(
            before.iter().all(|&(r, k)| !r && k),
            "{b:?}: no earlier arrival in range: {before:?}"
        );
        let o = w.obj(a).unwrap();
        assert_eq!(o.target, Some(Target::Object(t)), "{b:?}");
        assert!(
            o.pos.cell().x < glasnost_sim::testkit::mid(32, 9, 0).x,
            "{b:?}: short of the destination"
        );
        assert!(!fraction(&w, a));
    }
}

// spec: S-0208 claim 17 (under another behaviour there is no early stop: the walker continues to its destination)
#[test]
fn walker_in_move_does_not_stop_in_range() {
    let (mut w, a, _) = approach_duel(Behaviour::Move, 10, 8);
    let arrivals = run_approach(&mut w, a);
    assert!(
        arrivals.iter().any(|&(r, k)| r && k),
        "in range and walking on: {arrivals:?}"
    );
    assert_eq!(
        w.obj(a).unwrap().pos.cell(),
        glasnost_sim::testkit::mid(32, 8, 0)
    );
}

// spec: S-0208 claim 17 (exception: a destination that is the target object itself is lost as the firing sequence starts). The firing sequence's start is the observation: with a fire frame of 0 the shot leaves on that tick and ends the attack state at once (S-0107 r40 claim 11, End of the attack state)
#[test]
fn destination_on_the_target_is_lost_on_firing() {
    let (mut w, a, t) = approach_duel(Behaviour::Attack, 2, 0);
    w.obj_mut(a).unwrap().dest = Some(Target::Object(t));
    let mut fired = false;
    for _ in 0..60 {
        let now = w.tick;
        tick::step(&mut w);
        let o = w.obj(a).unwrap();
        if o.last_fired == now
            || o.infantry.as_ref().is_some_and(|s| {
                s.attack || matches!(s.seq, Some(Seq::FireStanding | Seq::FireProne))
            })
        {
            fired = true;
            assert_eq!(o.dest, None, "lost with the firing sequence");
            assert_eq!(o.target, Some(Target::Object(t)));
            break;
        }
    }
    assert!(fired, "the walker started a firing sequence");
}

/// A soldier standing at the centre of cell(4, 0), under a high-bridge deck
/// laid over cells (3 … 5, 0) of flat ground, or on that deck with `deck`;
/// it has just its own standing point left as walking target, the cell as
/// destination and no path steps.
fn under_deck(quirk: bool, deck: bool) -> (World, ObjectId) {
    use glasnost_sim::world::Layer;
    let mut s = scene();
    s.ruleset.globals.quirks.walker_bridge_destination_deck = quirk;
    let c = cell(4, 0);
    s.place_infantry(0, "TESTMAN", c, 0);
    let mut w = s.build();
    for x in 3..=5 {
        let i = w.cells.idx(cell(x, 0)).expect("cell");
        w.cells.deck[i] = true;
        w.cells.deck_entry[i] = true;
        w.cells.bridge_axis[i] = glasnost_sim::cells::Axis::X;
    }
    w.regions.dirty = true;
    let id = men(&w)[0];
    if deck {
        w.cells.leave(c, Layer::Ground, id);
        w.cells.enter(c, Layer::Deck, id, false);
    }
    let o = w.obj_mut(id).expect("soldier");
    if deck {
        o.layer = Layer::Deck;
        o.pos.z += 416; // spec: S-0208 claim 18 (an arrival on the deck has height H + 416)
    }
    o.dest = Some(Target::Cell(c));
    o.path.steps.clear();
    let p = o.pos;
    let ws = o.walk.as_mut().expect("walk state");
    ws.target = Some(p);
    ws.fraction = true;
    (w, id)
}

// spec: S-0208 claim 18 and F9 (scenario W10 part e: a cell destination on a high-bridge cell is the centre at H + 414 in both layers; an arrival on the deck completes it, an arrival on the ground under it keeps the destination with speed fraction 0; a later failed search clears it by S-0212 claim 23)
// spec: S-0208 Edge cases (`quirks.walker_bridge_destination_deck` off: the arrival layer's height, so either arrival completes the destination)
#[test]
fn claim_18_cell_destination_under_a_deck() {
    let c = cell(4, 0);
    for (quirk, deck, kept) in [
        (true, false, true),
        (true, true, false),
        (false, false, false),
        (false, true, false),
    ] {
        let (mut w, id) = under_deck(quirk, deck);
        let z = w.obj(id).expect("s").pos.z;
        let h = w.surface_at(256 * c.x + 128, 256 * c.y + 128);
        let expect = match (quirk, deck) {
            (true, _) => h + 414,
            (false, true) => h + 416,
            (false, false) => h,
        };
        assert_eq!(walk::cell_destination_height(&w, id, c), expect);
        tick::step(&mut w);
        let o = w.obj(id).expect("s");
        assert_eq!(o.pos.z, z, "never moved vertically");
        assert_eq!(o.layer == glasnost_sim::world::Layer::Deck, deck);
        assert_eq!(
            o.dest.is_some(),
            kept,
            "quirk {quirk}, deck {deck}: the destination on the arrival tick"
        );
        assert!(
            !o.walk.as_ref().expect("w").fraction,
            "speed fraction 0 on the arrival tick"
        );
        if kept {
            // The deck cannot be reached from the ground: the next search fails
            // and gives up the destination (S-0212 claim 23).
            let mut cleared = false;
            for _ in 0..200 {
                tick::step(&mut w);
                let o = w.obj(id).expect("s");
                assert_eq!(o.pos.cell(), c);
                if o.dest.is_none() {
                    cleared = true;
                    break;
                }
            }
            assert!(cleared, "the failed search clears the destination");
        }
    }
}

/// Plateau at height level 4 everywhere but a pit at level 0 under
/// x = 3 … 5, y = −1 … 1, whose middle row carries a high-bridge deck with
/// the deck-entry mark; a soldier stands on the plateau at cell(1, 0).
fn high_bridge() -> (World, ObjectId) {
    let mut s = scene();
    let cells: Vec<CellXY> = s.map.cells.iter().map(|(c, _)| c).collect();
    for c in cells {
        let pit = (cell(3, 0).x..=cell(5, 0).x).contains(&c.x) && (c.y - cell(0, 0).y).abs() <= 1;
        s.set_cell(c, 0, if pit { 0 } else { 4 });
    }
    s.place_infantry(0, "TESTMAN", cell(1, 0), 0);
    let mut w = s.build();
    for x in 3..=5 {
        let i = w.cells.idx(cell(x, 0)).expect("cell");
        w.cells.deck[i] = true;
        w.cells.deck_entry[i] = true;
        w.cells.bridge_axis[i] = glasnost_sim::cells::Axis::X;
    }
    w.regions.dirty = true;
    glasnost_sim::regions::ensure(&mut w);
    let id = men(&w)[0];
    (w, id)
}

// spec: S-0208 claim 1 (scenario W2 e: a walking target on a bridge deck lies 416 leptons above H, and the arriving walker stands at that height)
// spec: S-0209 Terms (cell layer: an object on a bridge deck belongs to the deck layer) and claim 15 (deck on crossing from a bank onto a high bridge 4 levels down, kept along the deck, ground on crossing onto a bank)
// spec: S-0212 claim 10 (Δ = −4 onto a marked deck enters T's deck; Δ = 0 along the deck stays on it; Δ = +4 off the deck enters T's ground)
#[test]
fn a_walker_crosses_a_high_bridge_on_its_deck() {
    use glasnost_sim::world::Layer;
    let (mut w, id) = high_bridge();
    open_orders(&mut w);
    order_move(&mut w, id, cell(7, 0));
    let mut seen = Vec::new();
    let n = common::run_until(&mut w, 2000, |w| {
        let o = w.obj(id).unwrap();
        let c = o.pos.cell();
        if (3..=5).contains(&(c.x - cell(0, 0).x)) {
            let h = w.surface_at(o.pos.x, o.pos.y);
            seen.push((
                c,
                o.layer,
                o.pos.z - h,
                w.cells.list(c, Layer::Deck).contains(&id),
            ));
        }
        o.pos.cell() == cell(7, 0) && o.walk.as_ref().is_some_and(|s| s.target.is_none())
    });
    assert!(n < 2000, "the walker never crossed");
    assert!(!seen.is_empty(), "the walker crossed the pit");
    for (c, layer, above, listed) in &seen {
        assert_eq!(*layer, Layer::Deck, "on the deck in {c:?}");
        assert!(*listed, "listed in the deck layer of {c:?}");
        // The plateau and the deck both lie 416 leptons above the pit's ground.
        assert_eq!(*above, 416, "height above H in {c:?}");
    }
    let o = w.obj(id).unwrap();
    assert_eq!(o.layer, Layer::Ground, "off the deck again");
    assert_eq!(o.pos.z, w.surface_at(o.pos.x, o.pos.y));
}

// spec: S-0208 claim 3 and S-0209 claim 15 (a walker at z = F + 416 over flat ground under a deck chooses and reserves its position in the deck record, not the ground's under it)
#[test]
fn a_walker_reserves_its_deck_position_in_the_deck_record() {
    use glasnost_sim::world::Layer;
    let (mut w, id) = high_bridge();
    open_orders(&mut w);
    order_move(&mut w, id, cell(7, 0));
    let c = cell(4, 0);
    common::run_until(&mut w, 2000, |w| {
        walk_target(w, id).is_some_and(|t| t.cell() == c)
    });
    let p = walk_target(&w, id).expect("a target on the deck");
    assert_eq!(p.z, w.surface_at(p.x, p.y) + 416);
    let (u, v) = (p.x.rem_euclid(256), p.y.rem_euclid(256));
    let class = walk::position_class(u, v, w.rules().quirks.top_position_counts_as_centre);
    assert!(w.cells.occ(c, Layer::Deck).is_some_and(|r| r.taken(class)));
    assert!(
        !w.cells
            .occ(c, Layer::Ground)
            .is_some_and(|r| r.taken(class))
    );
}

// spec: S-0208 claim 1 and S-0209 Terms (a walker on the deck is game state: the state hash survives a save and load round trip, and both copies walk on alike)
#[test]
fn a_walker_on_a_deck_survives_save_and_load() {
    use glasnost_sim::world::Layer;
    let (mut w, id) = high_bridge();
    open_orders(&mut w);
    order_move(&mut w, id, cell(7, 0));
    common::run_until(&mut w, 2000, |w| {
        w.obj(id)
            .is_some_and(|o| o.layer == Layer::Deck && o.pos.cell() == cell(4, 0))
    });
    let bytes = glasnost_sim::save::save(&w);
    let mut back =
        glasnost_sim::save::load(&bytes, w.rules_arc(), w.catalogue_arc()).expect("load");
    assert_eq!(
        glasnost_sim::save::state_hash(&back),
        glasnost_sim::save::state_hash(&w)
    );
    assert_eq!(back.obj(id).unwrap().layer, Layer::Deck);
    common::run(&mut w, 200);
    common::run(&mut back, 200);
    assert_eq!(
        glasnost_sim::save::state_hash(&back),
        glasnost_sim::save::state_hash(&w)
    );
}

// spec: S-0209 claim 15 (scenario C22 e: the asking tick changes the reservation but not the walker's layer or listing) and claim 1 (the walker is listed in the next cell only once its position moves into it)
#[test]
fn choosing_a_deck_target_does_not_change_the_walkers_layer() {
    use glasnost_sim::world::Layer;
    let (mut w, id) = high_bridge();
    open_orders(&mut w);
    order_move(&mut w, id, cell(7, 0));
    let bridge = cell(3, 0);
    common::run_until(&mut w, 2000, |w| {
        walk_target(w, id).is_some_and(|t| t.cell() == bridge)
    });
    let o = w.obj(id).unwrap();
    assert_eq!(o.pos.cell(), cell(2, 0), "still on the bank");
    assert_eq!(o.layer, Layer::Ground, "the target choice keeps its layer");
    assert!(w.cells.list(cell(2, 0), Layer::Ground).contains(&id));
    assert!(!w.cells.list(bridge, Layer::Deck).contains(&id));
    let t = walk_target(&w, id).unwrap();
    let class = walk::position_class(
        t.x.rem_euclid(256),
        t.y.rem_euclid(256),
        w.rules().quirks.top_position_counts_as_centre,
    );
    assert!(
        w.cells
            .occ(bridge, Layer::Deck)
            .is_some_and(|r| r.taken(class))
    );
}

// spec: S-0209 claim 15 (scenario C22 e: the first tick the position lies in the bridge cell makes the walker a deck object, the first tick it lies in the bank cell a ground object; neither waits for arrival)
#[test]
fn the_layer_changes_on_the_first_crossing_before_arrival() {
    use glasnost_sim::world::Layer;
    let (mut w, id) = high_bridge();
    open_orders(&mut w);
    order_move(&mut w, id, cell(7, 0));
    for (into, layer) in [(cell(3, 0), Layer::Deck), (cell(6, 0), Layer::Ground)] {
        let n = common::run_until(&mut w, 2000, |w| w.obj(id).unwrap().pos.cell() == into);
        assert!(n < 2000, "the walker reached {into:?}");
        let o = w.obj(id).unwrap();
        assert_eq!(o.layer, layer, "layer on entering {into:?}");
        assert!(w.cells.list(into, layer).contains(&id));
        let t = walk_target(&w, id).expect("still walking");
        assert_ne!(
            (t.x, t.y),
            (o.pos.x, o.pos.y),
            "not yet arrived in {into:?}"
        );
    }
}

// spec: S-0209 claim 15 (the crossing table, including third cells on the way to a diagonal target: into a high-bridge cell 4 levels down, deck; into one at any other relation, unchanged; from a high-bridge cell into one without, ground; neither, unchanged)
#[test]
fn crossing_a_cell_follows_the_layer_table() {
    use glasnost_sim::world::Layer::{Deck, Ground};
    let (w, _) = high_bridge();
    let t = |cur, a: (i32, i32), b: (i32, i32)| {
        walk::entered_layer(&w, cur, cell(a.0, a.1), cell(b.0, b.1))
    };
    // Bank (level 4) onto the bridge (ground level 0), straight or diagonal.
    assert_eq!(t(Ground, (2, 0), (3, 0)), Deck);
    assert_eq!(t(Ground, (2, 1), (3, 0)), Deck);
    // Along the deck, and under it from the pit: the layer it had.
    assert_eq!(t(Deck, (3, 0), (4, 0)), Deck);
    assert_eq!(t(Ground, (4, 1), (4, 0)), Ground);
    // A deck walker crossing a third cell without a bridge: ground.
    assert_eq!(t(Deck, (4, 0), (4, 1)), Ground);
    assert_eq!(t(Deck, (5, 0), (6, 0)), Ground);
    // Neither cell carries a bridge: unchanged.
    assert_eq!(t(Ground, (1, 0), (2, 0)), Ground);
    assert_eq!(t(Deck, (1, 0), (2, 0)), Deck);
}

// spec: S-0208 claim 3 and S-0209 claim 15 (deck mode exactly when z > F + 312 leptons: equality chooses and reserves ground at H, one lepton more the deck at H + 416)
#[test]
fn the_target_surface_threshold_is_strict() {
    use glasnost_sim::world::Layer;
    for (above, deck) in [(312, false), (313, true)] {
        let (mut w, id) = high_bridge();
        open_orders(&mut w);
        // Once it stands on the deck in cell(4, 0), still holding its path,
        // the walker is lowered to `above` leptons over the pit's ground
        // before its next ask.
        order_move(&mut w, id, cell(7, 0));
        let n = common::run_until(&mut w, 2000, |w| {
            let o = w.obj(id).unwrap();
            o.pos.cell() == cell(4, 0) && o.walk.as_ref().is_some_and(|s| s.target.is_none())
        });
        assert!(n < 2000, "the walker stood on the deck");
        let p = w.obj(id).unwrap().pos;
        let f = w.surface_at(p.x, p.y);
        assert_eq!(p.z, f + 416);
        w.obj_mut(id).unwrap().pos.z = f + above;
        let next = cell(5, 0);
        common::run_until(&mut w, 2000, |w| {
            walk_target(w, id).is_some_and(|t| t.cell() == next)
        });
        let t = walk_target(&w, id).expect("a target in the next cell");
        let h = w.surface_at(t.x, t.y);
        assert_eq!(t.z, h + if deck { 416 } else { 0 }, "z = F + {above}");
        let class = walk::position_class(
            t.x.rem_euclid(256),
            t.y.rem_euclid(256),
            w.rules().quirks.top_position_counts_as_centre,
        );
        let (on, off) = if deck {
            (Layer::Deck, Layer::Ground)
        } else {
            (Layer::Ground, Layer::Deck)
        };
        assert!(
            w.cells.occ(next, on).is_some_and(|r| r.taken(class)),
            "z = F + {above}"
        );
        assert!(
            !w.cells.occ(next, off).is_some_and(|r| r.taken(class)),
            "z = F + {above}"
        );
        assert_eq!(w.obj(id).unwrap().layer, Layer::Deck, "listing unchanged");
    }
}

// spec: S-0208 claim 24 (a scatter destination is not an escape path: a stationary walker on a legal cell that belongs to no reach region of its zone accepts a forced request and gets a destination, but normal pathfinding's reachability gate gives it no path, so it never leaves its cell, even for an adjacent destination; being placed there sends no request) and S-0212 claim 6
#[test]
fn claim_24_scatter_is_no_escape() {
    let stranded = || {
        let mut s = scatter_scene(true);
        // Code 9 is water (S-0211 claim 6): a legal cell inside the usable area in no reach region of the walker's zone (S-0212 claim 4).
        s.set_cell(cell(0, 0), code_tile(9), 0);
        place(&mut s, "TESTMAN", cell(0, 0), 0);
        s
    };
    // Placed there: no request of its own, no destination.
    let s = stranded();
    let mut w = s.build();
    let id = men(&w)[0];
    assert_eq!(
        pos(&w, id).cell(),
        cell(0, 0),
        "placed on the stranded cell"
    );
    // The walker's zone: MovementZone absent reads normal (S-0211 Import mapping); its cell is in no reach region, the next one is.
    let z = glasnost_rules::sim::MovementZone::Normal;
    assert_eq!(
        glasnost_sim::regions::reach_in(&w, cell(0, 0), z),
        glasnost_sim::regions::Reach::None
    );
    assert!(matches!(
        glasnost_sim::regions::reach_in(&w, cell(1, 0), z),
        glasnost_sim::regions::Reach::Region(_)
    ));
    common::run(&mut w, 30);
    assert_eq!(w.obj(id).unwrap().dest, None);
    // A forced request, with and without move-anywhere: accepted, a destination, no escape.
    for req in [FORCED, FORCED_ANYWHERE] {
        let s = stranded();
        let mut w = s.build();
        let id = men(&w)[0];
        open_orders(&mut w);
        set_guard(&mut w, id);
        assert!(scatter::request(&mut w, id, req), "accepted");
        // spec: S-0208 claim 6 (r46: the destination comes from the nearby-cell search, so the standing walker has its extra locomotion at once; its search fails at the gate, which clears the destination, S-0212 claim 23)
        assert_eq!(w.obj(id).unwrap().dest, None, "the gate cleared it at once");
        assert!(walk_target(&w, id).is_none(), "no path, no walking target");
        for _ in 0..300 {
            tick::step(&mut w);
            assert_eq!(pos(&w, id).cell(), cell(0, 0), "it never leaves its cell");
        }
        // A direct move order to the adjacent cell fails the same way.
        order_move(&mut w, id, cell(1, 0));
        common::run(&mut w, 100);
        assert_eq!(pos(&w, id).cell(), cell(0, 0));
    }
}
