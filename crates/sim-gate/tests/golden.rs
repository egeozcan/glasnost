//! The committed golden hashes of the determinism gate, one file per
//! scenario. Regenerate with
//! `GLASNOST_UPDATE_GOLDEN=1 cargo test --release -p glasnost-sim-gate -- --include-ignored`.

use glasnost_sim_gate::{DEBUG_TICKS, Scenario, TICKS, golden_text};

fn path(s: Scenario) -> String {
    format!("{}/golden/{}.txt", env!("CARGO_MANIFEST_DIR"), s.name())
}

fn golden(s: Scenario) -> String {
    std::fs::read_to_string(path(s)).expect("the golden file exists")
}

/// Compares the first `ticks` lines after the header.
fn check(s: Scenario, ticks: i32) {
    let want = golden(s);
    let got = golden_text(s, ticks);
    let want_lines: Vec<&str> = want.lines().skip(1).collect();
    let got_lines: Vec<&str> = got.lines().skip(1).collect();
    assert!(
        want_lines.len() >= got_lines.len(),
        "the {} golden file is shorter than {ticks} ticks",
        s.name()
    );
    for (i, (g, w)) in got_lines.iter().zip(&want_lines).enumerate() {
        assert_eq!(
            g,
            w,
            "{}: state hash differs after tick {} (tick number {})",
            s.name(),
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
    for s in Scenario::ALL {
        check(s, DEBUG_TICKS);
    }
}

#[test]
#[ignore = "long; run in release by `cargo xtask gate`"]
fn full_run_matches_golden() {
    if std::env::var_os("GLASNOST_UPDATE_GOLDEN").is_some() {
        for s in Scenario::ALL {
            std::fs::write(path(s), golden_text(s, TICKS)).expect("golden file written");
        }
        return;
    }
    for s in Scenario::ALL {
        check(s, TICKS);
    }
}
