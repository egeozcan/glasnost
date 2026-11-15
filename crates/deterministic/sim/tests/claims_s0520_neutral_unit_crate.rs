//! Claim tests for S-0520 claim 13 (*In our engine*), sim issues S15
//! (#282): with the bases setting on, a player house's unit crate may give a
//! base-unit-list type, while the neutral house keeps the non-human
//! restriction and never gets one from the random rule. Synthetic data of
//! `crate_common`; every name and number is invented. Ledger:
//! `docs/m8-claims/sim-issues.md`, *Batch S15*.

#[path = "crate_common/mod.rs"]
mod crate_common;

use crate_common::*;
use glasnost_sim::crates;
use glasnost_sim::rules::Kind;
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseRole, World};

fn count(w: &World, name: &str) -> usize {
    let ti = w.rules().index_of(Kind::Vehicle, name);
    w.objects
        .values()
        .filter(|o| o.kind == Kind::Vehicle && Some(o.type_index) == ti)
        .count()
}

/// Two crate-goodie types, GOODIE (in the base-unit list) and UNARMED (not);
/// the collector belongs to `house`. Returns how many of 40 unit crates gave
/// the base-unit-list type.
fn base_units_given(house: u8) -> usize {
    let mut given = 0;
    for skip in 0..40 {
        let mut r = ruleset();
        r.sim.general.base_unit = Some(vec!["GOODIE".into()]);
        r.sim.vehicles.get_mut("UNARMED").unwrap().crate_goodie = Some(true);
        let mut w = testkit::game_with_non_players(&r, SIZE, 7);
        w.econ.settings.crates = Some(false);
        w.econ.start.bases = true;
        assert!(matches!(w.houses[2].role, HouseRole::Neutral));
        assert!(!w.houses[2].human && w.houses[0].human);
        for _ in 0..skip {
            let _ = w.rng.raw();
        }
        // A counted building keeps claim 9's forced base unit out of the way.
        spawn(&mut w, Kind::Building, "BASE", house, at(8, 8));
        let v = spawn(&mut w, Kind::Vehicle, "ARMED", house, at(0, 0));
        let c = at(1, 0);
        crate_at(&mut w, c, WOOD, 1);
        assert!(crates::pickup(&mut w, v, c), "set-up: a unit was placed");
        assert_eq!(count(&w, "GOODIE") + count(&w, "UNARMED"), 1);
        given += count(&w, "GOODIE");
    }
    given
}

// spec: S-0520 claim 13 (In our engine: our engine accepts the type for every player house)
#[test]
fn issue_282_a_player_house_may_draw_a_base_unit_type() {
    assert!(base_units_given(0) > 0);
}

// spec: S-0520 claim 13 (In our engine: neutral and special houses retain the original's non-human restriction: with the bases setting on, a base-unit-list type is refused)
#[test]
fn issue_282_a_neutral_house_never_draws_a_base_unit_type() {
    assert_eq!(
        base_units_given(2),
        0,
        "the neutral house got a base-unit-list type from the random rule"
    );
}
