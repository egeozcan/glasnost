//! S-0414 claim 21 and S-0508 claim 19 (#382): an occupant that its own
//! house's defeat kills inside a building of another house counts one
//! loss, as the same unit outside does. Synthetic rules
//! (`tests/structures_common`).

mod structures_common;

use glasnost_sim::econ::stats;
use glasnost_sim::rules::Kind;
use glasnost_sim::structures::garrison;
use glasnost_sim::testkit;
use structures_common::*;

// spec: S-0414 claim 21 (a destruction with neither an attacking object nor an attacking house still gives the victim's house a loss; this includes defeat hits) and S-0508 claim 19 (defeat kills the house's occupants inside a surviving house's building by individual killing hits)
#[test]
fn defeat_hit_on_an_occupant_inside_counts_one_loss() {
    // Control: the same unit type outside any building.
    let mut w = game();
    let u = put(&mut w, Kind::Infantry, "GARRI", 0, -6, -6);
    testkit::run(&mut w, 2);
    let before = stats::of(&w, A).losses;
    let combat_before = w.houses[0].combat.losses;
    glasnost_sim::econ::victory::defeat(&mut w, A);
    assert!(w.obj(u).is_none_or(|o| o.health <= 0));
    assert_eq!(
        (
            stats::of(&w, A).losses - before,
            w.houses[0].combat.losses - combat_before
        ),
        (1, 1),
        "one unit outside: one loss on each counter"
    );

    // The same unit inside a building that belongs to another house.
    let mut w = game();
    let b = put(&mut w, Kind::Building, "HUT", 2, 0, 0);
    let u = put(&mut w, Kind::Infantry, "GARRI", 0, -6, -6);
    assert!(garrison::attempt(&mut w, u, b));
    testkit::run(&mut w, 2);
    glasnost_sim::structures::ownership::change(
        &mut w,
        b,
        Some(B),
        glasnost_sim::structures::ownership::Cause::Trigger,
    );
    let before = stats::of(&w, A).losses;
    let combat_before = w.houses[0].combat.losses;
    glasnost_sim::econ::victory::defeat(&mut w, A);
    assert!(w.obj(u).is_none(), "killed inside");
    assert!(w.obj(b).is_some(), "the foreign building stays");
    assert_eq!(
        (
            stats::of(&w, A).losses - before,
            w.houses[0].combat.losses - combat_before
        ),
        (1, 1),
        "one occupant killed inside: one loss on each counter"
    );
}
