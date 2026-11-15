//! A gap field of a large radius keeps the shared off-map state's counts
//! exact (S-0409 claim 21; #338), and a large radius with
//! `quirks.gap_radius_wrap` off neither overflows nor runs without end.
//! Synthetic data only: buildings TOWER (sight 2), GAP (gap generator of
//! the radius given) and SAT (spy satellite), 1 × 1, on a flat map of 40.

use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::BuildingType;
use glasnost_sim::econ::shroud;
use glasnost_sim::rules::Kind;
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, World};

const SIZE: i32 = 40;
const A: HouseId = HouseId(0);

fn rules(radius: i32) -> Ruleset {
    let mut r = testkit::ruleset();
    r.registries.insert(
        "buildings".into(),
        ["TOWER", "GAP", "SAT"].iter().map(|s| s.to_string()).collect(),
    );
    let b = |sight: i32| {
        let mut b = BuildingType {
            strength: Some(500),
            foundation: Some([1, 1]),
            ..Default::default()
        };
        b.econ.sight = Some(sight);
        b
    };
    r.sim.buildings.insert("TOWER".into(), b(2));
    let mut gap = b(0);
    gap.gap_generator = Some(true);
    gap.econ.gap_radius_in_cells = Some(radius);
    r.sim.buildings.insert("GAP".into(), gap);
    let mut sat = b(0);
    sat.econ.spy_sat = Some(true);
    r.sim.buildings.insert("SAT".into(), sat);
    r
}

fn step(w: &mut World) {
    glasnost_sim::tick::step(w);
}

/// The shared state's (k, G) for house A.
fn shared_counts(w: &World) -> (i64, i64) {
    let s = w.econ.shroud.houses[0].as_ref().unwrap();
    let i = s.k.len() - 1;
    (i64::from(s.k[i]), i64::from(s.g[i]))
}

/// (shared contributions of the field, shared k and G after the stop,
/// explored after the stop, explored after the restart).
fn run(r: Ruleset) -> (usize, (i64, i64), bool, bool) {
    let mut w = testkit::game_with(&r, SIZE, 3);
    let off = CellXY::new(0, 0);
    assert!(shroud::counts(&w, A, off).is_none(), "not a legal cell");
    let at = testkit::mid(SIZE, 0, 25);
    assert!(shroud::counts(&w, A, at).is_some(), "a legal cell");
    testkit::spawn(&mut w, Kind::Building, "SAT", 0, testkit::mid(SIZE, -10, -10), 0);
    // House 1 keeps a building throughout.
    testkit::spawn(&mut w, Kind::Building, "TOWER", 1, testkit::mid(SIZE, 10, -10), 0);
    step(&mut w);
    step(&mut w);
    assert!(shroud::spy_satellite_active(&w, A));
    let g = testkit::spawn(&mut w, Kind::Building, "GAP", 1, at, 0);
    step(&mut w);
    step(&mut w);
    let shared = w.econ.shroud.houses[0].as_ref().unwrap().k.len() - 1;
    let n = w
        .obj(g)
        .and_then(|o| o.econ.sight.gap.as_ref())
        .expect("the field runs")
        .cells
        .iter()
        .filter(|&&i| i as usize == shared)
        .count();
    assert!(!shroud::explored(&w, A, off), "covered while the field runs");
    assert_eq!(shared_counts(&w).1, n as i64, "G counts every contribution");
    glasnost_sim::objects::remove(&mut w, g);
    step(&mut w);
    let after_stop = shared_counts(&w);
    let explored_after_stop = shroud::explored(&w, A, off);
    testkit::spawn(&mut w, Kind::Building, "GAP", 1, at, 0);
    step(&mut w);
    step(&mut w);
    assert!(w.objects.values().any(|o| o.econ.sight.gap.is_some()));
    (
        n,
        after_stop,
        explored_after_stop,
        shroud::explored(&w, A, off),
    )
}

// spec: S-0409 claim 21 (the shared count counts field-cell contributions; when hostile coverage is present again a shared cover count of 0 becomes 1 and the state is unexplored)
#[test]
fn issue_338_shared_state_is_covered_again_whatever_the_radius() {
    let small = run(rules(60));
    assert!(small.0 > 0 && small.0 <= 32767);
    assert_eq!((small.1, small.2, small.3), ((0, 0), true, false));
    let large = run(rules(127));
    assert!(large.0 > 32767, "the field has more than 32 767 non-legal cells");
    assert_eq!(
        (large.1, large.2, large.3),
        ((0, 0), true, false),
        "(shared k and G after the stop, explored after the stop, explored after the restart)"
    );
}

// design: #338 (with quirks.gap_radius_wrap off a radius past our engine's bound is used as the bound; the field starts and stops without overflow)
#[test]
fn issue_338_a_huge_radius_with_the_wrap_off_starts_and_stops() {
    let mut r = rules(50_000);
    r.globals.quirks.gap_radius_wrap = false;
    let out = run(r);
    assert!(out.0 > 0);
    assert_eq!((out.1, out.2, out.3), ((0, 0), true, false));
}
