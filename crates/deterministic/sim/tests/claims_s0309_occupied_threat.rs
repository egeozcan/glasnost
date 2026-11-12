//! S-0309 Terms, *Threat value* (#171): a building that holds occupants has
//! the threat value of its occupant count times the game-wide threat per
//! occupant, which table C's row "harmless building" reads. Synthetic data:
//! the garrison fixtures of `structures_common`; every name and number
//! invented.

mod structures_common;

use glasnost_map::triggers::Triggers;
use glasnost_rules::schema::Ruleset;
use glasnost_sim::combat::acquire::{self, SearchContext, SearchKind};
use glasnost_sim::combat::weapon;
use glasnost_sim::rules::Kind;
use glasnost_sim::structures::garrison;
use structures_common::*;

/// Beta garrisons a HUT1 with one GARRI; Alpha's armed CART stands three
/// cells away and runs the per-tick range search. Returns whether the
/// search picked the building.
fn picks_the_garrisoned_hut(r: &Ruleset) -> bool {
    let mut w = game_on(r, Triggers::default());
    let hut = put(&mut w, Kind::Building, "HUT1", 2, 3, 0);
    let u = put(&mut w, Kind::Infantry, "GARRI", 1, -6, -6);
    assert!(garrison::attempt(&mut w, u, hut));
    glasnost_sim::tick::step(&mut w);
    assert_eq!(garrison::count(&w, hut), 1);
    assert_eq!(owner(&w, hut), Some(B));
    assert!(weapon::current_weapon(&w, hut).is_some(), "the occupant's weapon");
    let a = put(&mut w, Kind::Vehicle, "CART", 0, 0, 0);
    let got = acquire::search(&mut w, a, SearchKind::Range, SearchContext::PerTick);
    assert!(got.is_none() || got == Some(hut));
    got.is_some()
}

fn rules(per_occupant: Option<i32>, posed: i32) -> Ruleset {
    let mut r = ruleset();
    r.sim.general.threat_per_occupant = per_occupant;
    r.sim.buildings.get_mut("HUT1").unwrap().combat.threat_posed = Some(posed);
    r
}

// spec: S-0309 Terms (threat value of a building that holds occupants: its occupant count times the game-wide threat per occupant) and table C row "harmless building" (a threat value of 0 is rejected)
#[test]
fn occupied_building_threat_value_is_count_times_per_occupant() {
    assert!(picks_the_garrisoned_hut(&rules(Some(7), 5)), "control");
    assert!(
        picks_the_garrisoned_hut(&rules(Some(7), 0)),
        "1 × 7 = 7 although the type's own field is 0"
    );
    assert!(
        !picks_the_garrisoned_hut(&rules(None, 5)),
        "1 × 0 = 0 although the type's own field is 5 (S-0309 Import mapping: absent is 0)"
    );
}
