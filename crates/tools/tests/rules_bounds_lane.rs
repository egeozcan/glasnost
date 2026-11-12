//! Local-only lane (opt-in with `--ignored` and `GLASNOST_GAME_DIR`, never
//! in CI): counts the loops in the links between the install's types that
//! sim batch S2 bounds (#224, #174, #203, #198; `glasnost_sim::loops`), so
//! that an implementer can tell whether the loop marks can change a game on
//! the stock rules. It prints counts only, to the terminal: no type name or
//! rules value, nothing written to a tracked file and no value asserted.

use std::path::Path;

#[test]
#[ignore = "local lane: reads the install named by GLASNOST_GAME_DIR"]
fn install_link_loops() {
    let Some(dir) = std::env::var_os("GLASNOST_GAME_DIR") else {
        eprintln!("rules_bounds_lane: GLASNOST_GAME_DIR is not set; nothing to do");
        return;
    };
    let rules = match glasnost_tools::arena::reference_rules(Path::new(&dir)) {
        Ok(Ok(r)) => r,
        Ok(Err(e)) => panic!("the install's rules did not resolve: {e}"),
        Err(e) => panic!("could not read the install ({:?})", e.kind()),
    };
    let l = &rules.loops;
    println!("animation types on a loop: {}", l.anims.nodes().len());
    println!("debris piece types on a loop: {}", l.pieces.nodes().len());
    println!("airburst weapons on a loop: {}", l.airburst.nodes().len());
    println!("particle types on a loop: {}", l.particles.nodes().len());
}
