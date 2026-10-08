//! Local-only lane (PLAN.md 6.2, layer 0). Skipped unless `GLASNOST_GAME_DIR`
//! names an install folder; never set in CI. Prints counts only.

use glasnost_tools::local_lane::{self, GAME_DIR_VAR, labels};

#[test]
fn real_install_parses_completely() {
    let Some(dir) = std::env::var_os(GAME_DIR_VAR) else {
        eprintln!("skipped: {GAME_DIR_VAR} is not set (local-only lane)");
        return;
    };
    let report = local_lane::run(std::path::Path::new(&dir)).expect("install folder readable");
    eprint!("{report}");
    for &label in labels::REQUIRED {
        let t = report.get(label).expect("tally present");
        assert!(t.total() > 0, "{label}: nothing found");
        assert!(t.all_ok(), "{label}: {t:?}");
    }
}
