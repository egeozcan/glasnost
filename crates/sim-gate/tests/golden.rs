//! The committed golden hashes of the determinism gate. Regenerate with
//! `GLASNOST_UPDATE_GOLDEN=1 cargo test --release -p glasnost-sim-gate -- --include-ignored`.

use glasnost_sim_gate::{DEBUG_TICKS, TICKS, golden_text};

const GOLDEN: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/golden/scripted.txt");

fn golden() -> String {
    std::fs::read_to_string(GOLDEN).expect("golden/scripted.txt exists")
}

/// Compares the first `ticks` lines after the header.
fn check(ticks: i32) {
    let want = golden();
    let got = golden_text(ticks);
    let want_lines: Vec<&str> = want.lines().skip(1).collect();
    let got_lines: Vec<&str> = got.lines().skip(1).collect();
    assert!(
        want_lines.len() >= got_lines.len(),
        "the golden file is shorter than {ticks} ticks"
    );
    for (i, (g, w)) in got_lines.iter().zip(&want_lines).enumerate() {
        assert_eq!(
            g,
            w,
            "state hash differs after tick {} (tick number {})",
            i + 1,
            i
        );
    }
}

#[test]
fn debug_prefix_matches_golden() {
    if std::env::var_os("GLASNOST_UPDATE_GOLDEN").is_some() {
        return;
    }
    check(DEBUG_TICKS);
}

#[test]
#[ignore = "long; run in release by `cargo xtask gate`"]
fn full_run_matches_golden() {
    if std::env::var_os("GLASNOST_UPDATE_GOLDEN").is_some() {
        std::fs::write(GOLDEN, golden_text(TICKS)).expect("golden file written");
        return;
    }
    check(TICKS);
}
