//! Claim tests for S-0501 r93 (Q-2291): the spy plane after its target
//! (claim 51), claim 4's side rule for the edge push-back, claim 10's
//! removal at the edge and `quirks.aircraft_edge_hold`, on synthetic data
//! through the real tick path: the spy plane of the superweapon test
//! ruleset, with the flight flags each test sets. Every type, map and
//! number is invented.

mod sw_common;

use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_sim::air;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::superweapons::planes;
use glasnost_sim::world::{HouseId, ObjectId, Pos, World};
use sw_common::*;

/// The superweapon ruleset with one spy plane per launch, its type given
/// the fly-by and fly-back flags, and `quirks.aircraft_edge_hold` at `hold`.
fn rules(fly_by: bool, fly_back: bool, hold: bool) -> Ruleset {
    let mut r = ruleset();
    r.globals.quirks.spy_plane_count_from_paradrop_list = false;
    r.globals.quirks.aircraft_edge_hold = hold;
    let u = r.sim.aircraft.get_mut("SPYP").expect("spy plane type");
    u.econ.fly_by = Some(fly_by);
    u.econ.fly_back = Some(fly_back);
    r
}

/// A spy plane of house BETA sent at `target`, run until it is in
/// spy-plane overfly. `banded`: the usable area lies inside the legal map,
/// two cells in; otherwise it covers the whole map, so its boundary lies on
/// or beyond the legal map's (S-0009 claims 1 and 4).
fn overflying(r: &Ruleset, banded: bool, target: CellXY) -> (World, ObjectId) {
    overflying_for(r, banded, target, BETA)
}

/// [`overflying`] for house `h`.
fn overflying_for(r: &Ruleset, banded: bool, target: CellXY, h: HouseId) -> (World, ObjectId) {
    let mut w = if banded { banded_game(r) } else { game(r) };
    let s = planes::spy_plane(&mut w, h, target)[0];
    until(&mut w, 3000, |w| {
        w.obj(s)
            .is_some_and(|o| o.beh.current == Some(Behaviour::SpyPlaneOverfly))
    });
    (w, s)
}

/// A new game like `game` on a map SIZE wide and 8 high whose usable area
/// covers it (S-0009 claims 1 and 4): Lx = 0 and T = ⌊(20 + 8 − 1) / 2⌋ = 13,
/// so claim 4's side rule gives −128 to every cell with d ≥ 6.
fn short_game(r: &Ruleset) -> World {
    let m = glasnost_sim::testkit::empty_map(SIZE);
    let fp = glasnost_map::Footprint::new(SIZE, 8, [0, 0, SIZE, 8]);
    let mut cells = glasnost_map::CellGrid::new(fp);
    let all: Vec<_> = cells.iter().map(|(c, _)| c).collect();
    for c in all {
        if let (Some(cell), Some(old)) = (cells.get_mut(c), m.cells.get(c)) {
            *cell = *old;
        }
    }
    let mut m = m;
    m.footprint = fp;
    m.cells = cells;
    let rules = std::sync::Arc::new(glasnost_sim::rules::SimRules::new(r, "synthetic-test"));
    let mut w = glasnost_sim::load::new_game(
        rules,
        &m,
        glasnost_sim::testkit::catalogue(),
        &glasnost_sim::testkit::setup(1),
    )
    .expect("synthetic test game loads");
    while glasnost_sim::orders::blocked(&w) {
        step(&mut w);
    }
    w
}

fn legal(w: &World, c: CellXY) -> bool {
    w.cells.cell(c).is_some()
}

/// What happened to the plane over at most `n` more ticks.
#[derive(Debug, Default)]
struct Run {
    /// The tick it was removed in, if it was.
    removed: Option<i32>,
    /// Whether it was ever seen at the end of a tick outside the usable area after entering it.
    outside_usable: bool,
    /// Whether it was ever seen at the end of a tick in a cell that is not legal.
    off_map: bool,
}

fn watch(w: &mut World, s: ObjectId, n: i32) -> Run {
    let mut run = Run::default();
    for _ in 0..n {
        step(w);
        let Some(o) = w.obj(s) else {
            run.removed = Some(w.tick);
            break;
        };
        assert!(o.health > 0, "alive");
        let c = o.pos.cell();
        if o.air.as_ref().is_some_and(|a| a.entered) && !w.cells.usable(c) {
            run.outside_usable = true;
        }
        run.off_map |= !legal(w, c);
    }
    run
}

// spec: S-0501 claim 51 (r93, Q-2291: in spy-plane overfly the aircraft makes no turn of its own: it holds the run lock from the moment it queues overfly, nothing clears it in that behaviour, guidance makes no body-facing request with it, so the desired body facing stays the last one guidance requested in spy-plane approach; the exit destination does not steer it; nothing ends overfly: no timer or count, its processing queues nothing, and it stays in overfly) and claim 5 (the fixed full speed with the run lock), AC32
#[test]
fn claim_51_the_heading_is_held_and_nothing_ends_overfly() {
    for (banded, target) in [(true, at(-3, 2)), (false, at(4, -1))] {
        // The fly-back flag alone, so that it stays at the edge it meets for the rest of the game.
        let (mut w, s) = overflying(&rules(false, true, true), banded, target);
        let held = w.obj(s).unwrap().body.target;
        assert!(w.obj(s).unwrap().dest.is_some(), "an exit destination");
        let mut away = false;
        for _ in 0..2000 {
            step(&mut w);
            let o = w
                .obj(s)
                .expect("never removed with the fly-back flag alone");
            let a = o.air.clone().unwrap();
            assert_eq!(o.beh.current, Some(Behaviour::SpyPlaneOverfly));
            assert_eq!(o.beh.queued, None, "its processing queues nothing");
            assert!(a.run_lock, "nothing clears the run lock");
            assert_eq!(o.body.target, held, "no body-facing request");
            assert!(!a.landing, "no landing start");
            // spec: S-0501 claim 5 (the target is fixed at 1 while the aircraft carries the run lock)
            assert!(a.tgt.eq_num(glasnost_num::F64::from_i64(1, w.rules().mode)));
            // spec: S-0501 claim 51 (an absent destination is chosen again)
            assert!(o.dest.is_some());
            if let Some(glasnost_sim::world::Target::Cell(c)) = o.dest {
                let to = glasnost_num::geom::direction(
                    i64::from(c.x * 256 + 128 - o.pos.x),
                    i64::from(c.y * 256 + 128 - o.pos.y),
                    w.rules().quirks.table_direction,
                );
                away |= (to.wrapping_sub(held) as i16).unsigned_abs() > 2048;
            }
        }
        assert!(
            away,
            "the destination lay off its heading, and it was not turned toward it"
        );
    }
}

// spec: S-0501 claim 51 (r93, Q-2291: at the map's edge, with neither flag it is pushed back only by a step that crosses the legal map's boundary and is removed in the update tick in which its position leaves the usable area; with the fly-by flag, alone or with the fly-back flag, it is not pushed back and is removed in the update tick in which its position leaves the legal map; with the fly-back flag alone it is never removed at an edge) and claim 10 (removals are tested on its position after that tick's movement, including any push-back), AC32
#[test]
fn claim_51_the_edge_outcome_by_flight_flags() {
    let target = at(-3, 2);
    // Neither flag, the usable area inside the legal map: removed on leaving the usable area.
    let (mut w, s) = overflying(&rules(false, false, true), true, target);
    let r = watch(&mut w, s, 3000);
    assert!(r.removed.is_some(), "removed: {r:?}");
    assert!(
        !r.outside_usable,
        "removed in the tick it leaves the usable area"
    );
    // The fly-by flag, alone or with the fly-back flag: removed on leaving the legal map.
    for fly_back in [false, true] {
        let (mut w, s) = overflying(&rules(true, fly_back, true), true, target);
        let r = watch(&mut w, s, 3000);
        assert!(r.removed.is_some(), "removed: {r:?}");
        assert!(r.outside_usable, "it flies on through the band: {r:?}");
        assert!(!r.off_map, "removed in the tick it leaves the legal map");
    }
    // The fly-back flag alone: never removed at the edge.
    let (mut w, s) = overflying(&rules(false, true, true), true, target);
    let r = watch(&mut w, s, 3000);
    assert_eq!(r.removed, None, "held at the edge: {r:?}");
    assert!(r.outside_usable && !r.off_map, "{r:?}");
}

// spec: S-0501 claim 10 (r93: with neither flag, at an edge where the usable area's boundary lies on or beyond the legal map's, claim 4's push-back keeps the aircraft in legal cells, so it is never removed there, quirks.aircraft_edge_hold) and claim 51 (an aircraft with neither flag is held the same way at such an edge)
#[test]
fn claim_10_neither_flag_is_held_where_the_bounds_coincide() {
    let (mut w, s) = overflying(&rules(false, false, true), false, at(4, -1));
    let r = watch(&mut w, s, 3000);
    assert_eq!(r.removed, None, "held: {r:?}");
    assert!(!r.off_map);
}

// spec: S-0501 Edge cases (quirks.aircraft_edge_hold: on, the push-back holds an aircraft with the fly-back flag alone, or with neither flag where the bounds coincide, at the edge for good; off, an aircraft that may leave the map now is not pushed back, whatever its flags, and one claim 10 would not remove before it leaves the legal map is removed in the update tick in which its position leaves the legal map; with neither flag and a usable area inside the legal map it is still removed when it leaves the usable area) and claims 4, 10 and 51
#[test]
fn quirk_aircraft_edge_hold_both_positions() {
    for hold in [true, false] {
        // The fly-back flag alone, banded.
        let (mut w, s) = overflying(&rules(false, true, hold), true, at(-3, 2));
        let r = watch(&mut w, s, 3000);
        assert_eq!(
            r.removed.is_some(),
            !hold,
            "fly-back alone, hold {hold}: {r:?}"
        );
        assert!(!r.off_map);
        // Neither flag, where the bounds coincide.
        let (mut w, s) = overflying(&rules(false, false, hold), false, at(4, -1));
        let r = watch(&mut w, s, 3000);
        assert_eq!(
            r.removed.is_some(),
            !hold,
            "neither flag, hold {hold}: {r:?}"
        );
        assert!(!r.off_map);
        // Neither flag, banded: removed on leaving the usable area either way.
        let (mut w, s) = overflying(&rules(false, false, hold), true, at(-3, 2));
        let r = watch(&mut w, s, 3000);
        assert!(r.removed.is_some() && !r.outside_usable, "{r:?}");
    }
}

/// Claim 4's shift for a new position in cell `c`: +128 when its local
/// column a = ⌊(d + W) / 2⌋ − Lx is below T = ⌊(W + H − 1) / 2⌋, else −128.
fn side(w: &World, c: CellXY) -> i32 {
    let fp = w.cells.footprint();
    // spec: S-0501 claim 4 (r93: a = ⌊(d + W) / 2⌋ − Lx with d = cₓ − cᵧ, floor division; T = ⌊(W + H − 1) / 2⌋)
    let a = (c.x - c.y + fp.width).div_euclid(2) - fp.local[0];
    let t = (fp.width + fp.height - 1).div_euclid(2);
    if a < t { 128 } else { -128 }
}

/// Follows the held plane `s` for `n` ticks and checks claim 4's push-back
/// in every tick whose forward step would leave the legal map: the position
/// shifted by the side rule when that cell is legal, with no draw;
/// otherwise one raw draw and the 64-lepton candidate when it is legal,
/// else no relocation. Returns the shifts met.
fn check_push_back(w: &mut World, s: ObjectId, n: i32) -> Vec<i32> {
    let mode = w.rules().mode;
    let stall = w.rules().quirks.aircraft_unit_step_stall;
    let ti = w.obj(s).unwrap().type_index;
    let v = w.rules().unit(Kind::Aircraft, ti).unwrap().speed;
    let mut shifts = Vec::new();
    for _ in 0..n {
        let o = w.obj(s).expect("held").clone();
        let a = o.air.clone().unwrap();
        let h0 = air::height(w, s);
        let turning = o.body.turning(w.tick) || o.body.turning(w.tick + 1);
        let f = o.body.value(w.tick);
        let rng = w.rng.clone();
        step(w);
        let p1 = w.obj(s).expect("held").pos;
        // A descent adds the 5-lepton nudge toward the destination (claim 4): those ticks are not checked.
        if turning || air::height(w, s) != h0 {
            continue;
        }
        let (nx, ny) = air::step(o.pos.x, o.pos.y, f, v, a.cur, stall, mode);
        if legal(w, Pos::new(nx, ny, 0).cell()) {
            continue;
        }
        let sh = side(w, Pos::new(nx, ny, 0).cell());
        shifts.push(sh);
        let shifted = Pos::new(nx + sh, ny, 0);
        if legal(w, shifted.cell()) {
            assert_eq!((p1.x, p1.y), (shifted.x, shifted.y), "the shifted point");
            assert_eq!(w.rng, rng, "no draw");
        } else {
            let mut r = rng.clone();
            let raw = r.raw();
            assert_eq!(w.rng, r, "one raw draw");
            // spec: S-0501 claim 4 (its low eight bits times 256 as the facing, F3 with length 64)
            let (rx, ry) = glasnost_sim::walk::step_point(
                shifted.x,
                shifted.y,
                ((raw & 0xff) * 256) as u16,
                64,
                true,
                mode,
            );
            let rc = Pos::new(rx, ry, 0).cell();
            let cand = if (0..=511).contains(&rc.x) && (0..=511).contains(&rc.y) {
                (rx, ry)
            } else {
                (shifted.x, shifted.y)
            };
            let want = if legal(w, Pos::new(cand.0, cand.1, 0).cell()) {
                cand
            } else {
                (o.pos.x, o.pos.y)
            };
            assert_eq!((p1.x, p1.y), want, "the candidate, or no relocation");
        }
    }
    shifts
}

// spec: S-0501 claim 4 (r93, Q-2291: the push-back's side rule: +128 when the new position's cell has local column a < T, else −128; with H ≥ W and Lx ≥ 2 every cell with d ≤ W + 1 gets +128; the shifted point when legal, with no draw; otherwise one raw scenario-stream draw for the 64-lepton fallback, which is taken only when legal) and claim 51 (the fly-back flag alone: at every blocked step the shifted position when that cell is legal, else one draw and a move only when the candidate is legal), AC32 with the random-state recorder
#[test]
fn claim_4_the_side_rule_of_the_push_back() {
    // The banded map has Lx = 2 and H = W: every shift is +128.
    let mut banded = Vec::new();
    for target in [at(-3, 2), at(4, -1), at(2, 4)] {
        let (mut w, s) = overflying(&rules(false, true, true), true, target);
        banded.extend(check_push_back(&mut w, s, 2500));
    }
    assert!(!banded.is_empty(), "blocked steps met");
    assert!(banded.iter().all(|&x| x == 128), "{banded:?}");
    // A short map, 20 wide and 8 high, wholly usable: Lx = 0 and T = 13, so
    // cells with d ≥ 6 shift by −128 and the others by +128.
    let mut short = Vec::new();
    for target in [
        CellXY::new(23, 11),
        CellXY::new(17, 11),
        CellXY::new(12, 18),
    ] {
        let r = rules(false, false, true);
        let mut w = short_game(&r);
        let s = planes::spy_plane(&mut w, BETA, target)[0];
        until(&mut w, 3000, |w| {
            w.obj(s)
                .is_some_and(|o| o.beh.current == Some(Behaviour::SpyPlaneOverfly))
        });
        short.extend(check_push_back(&mut w, s, 2500));
    }
    assert!(
        short.contains(&128) && short.contains(&-128),
        "both sides met: {short:?}"
    );
}
