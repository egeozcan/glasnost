//! Local-only lane (opt-in with `--ignored` and `GLASNOST_GAME_DIR`, never
//! in CI): lists the behaviours to which the install's resolved rules give
//! the no-threat property (S-0215 claim 16) and, among them, those that
//! also have the zombie property (claim 17), so that an implementer can
//! tell which behaviours the no-threat fall-back on damage can reach. It
//! prints our behaviour keys only, to the terminal; nothing is written to a
//! tracked file and no value is asserted.

use std::path::Path;

use glasnost_sim::rules::Behaviour;

#[test]
#[ignore = "local lane: reads the install named by GLASNOST_GAME_DIR"]
fn install_no_threat_behaviours() {
    let Some(dir) = std::env::var_os("GLASNOST_GAME_DIR") else {
        eprintln!("no_threat_lane: GLASNOST_GAME_DIR is not set; nothing to do");
        return;
    };
    let rules = match glasnost_tools::arena::reference_rules(Path::new(&dir)) {
        Ok(Ok(r)) => r,
        Ok(Err(e)) => panic!("the install's rules did not resolve: {e}"),
        Err(e) => panic!("could not read the install ({:?})", e.kind()),
    };
    let mut any = false;
    for b in Behaviour::ALL {
        let p = rules.behaviour(b);
        if p.no_threat {
            any = true;
            println!(
                "no-threat: {} (zombie {}, retaliate {}, scatter {})",
                b.key(),
                p.zombie,
                p.retaliate,
                p.scatter
            );
        }
    }
    if !any {
        println!("no-threat: none");
    }
}
