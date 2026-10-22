//! The movement trace (M8 A4) is read-only: a run with it on takes the
//! same steps as a run without it, tick for tick, and it is never saved.
//! Our own diagnostics; no spec applies.

use glasnost_sim::occupancy::Outcome;
use glasnost_sim::trace::{self, What};
use glasnost_sim::{save, testkit, tick};

/// The determinism gate's scripted replay (group moves whose paths cross,
/// a stop, scatters) with and without the trace: the state hash after
/// every tick is the same, and the trace recorded path computations and
/// refused steps.
#[test]
fn a_traced_run_hashes_like_an_untraced_one() {
    // nocite: our own diagnostics (a test length, the gate's replay)
    let rep = testkit::scripted_replay(11, 900);
    let run = |traced: bool| {
        let mut w = glasnost_sim::load::new_game(
            testkit::rules(),
            &testkit::map(),
            testkit::catalogue(),
            &rep.setup,
        )
        .expect("the synthetic game loads");
        if traced {
            trace::switch_on(&mut w);
        }
        let mut hashes = Vec::new();
        let mut events = Vec::new();
        while tick::ticks_run(&w) < rep.ticks {
            rep.issue_due(&mut w);
            tick::step(&mut w);
            hashes.push(save::state_hash(&w));
            events.extend(trace::take(&w));
        }
        (hashes, events, w)
    };
    let (plain, none, _) = run(false);
    let (traced, events, w) = run(true);
    assert!(none.is_empty(), "an untraced run records nothing");
    assert_eq!(plain.len(), traced.len());
    for (i, (a, b)) in plain.iter().zip(&traced).enumerate() {
        assert_eq!(a, b, "the hashes part after tick {i}");
    }
    let searches = events
        .iter()
        .filter(|e| matches!(e.what, What::Search { .. }))
        .count();
    let refused = events
        .iter()
        .filter(|e| matches!(e.what, What::Step { outcome, .. } if outcome != Outcome::Clear))
        .count();
    assert!(searches > 0, "no path computation was recorded");
    assert!(refused > 0, "no refused step was recorded");
    // The trace is not saved: a saved and loaded world has none, and the
    // hash is the same.
    assert!(trace::on(&w));
    let loaded = save::load(&save::save(&w), testkit::rules(), testkit::catalogue()).unwrap();
    assert!(!trace::on(&loaded));
    assert_eq!(save::state_hash(&loaded), save::state_hash(&w));
    // A clone has no trace either.
    assert!(!trace::on(&w.clone()));
}
