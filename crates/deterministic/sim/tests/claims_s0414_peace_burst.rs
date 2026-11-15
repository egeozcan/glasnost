//! Claim test for S-0414 claim 12 with S-0301 claim 4 (sim batch S20,
//! #371): making peace drops a unit's target of the new ally, and the
//! cleared target sets its burst counter to 0. Synthetic data.

mod common;

use common::{Scene, TRACK, cell, unit_type};
use glasnost_rules::sim::Country;
use glasnost_sim::econ::victory;
use glasnost_sim::rules::Kind;
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, HouseSetup, Target};

const A: HouseId = HouseId(0);
const B: HouseId = HouseId(1);

// spec: S-0301 claim 4 (a cleared target sets the burst counter to 0); S-0414 claim 12 (making peace drops the target)
#[test]
fn making_peace_drops_the_target_and_resets_the_burst_counter() {
    let mut s = Scene::new();
    let country = || Country {
        multiplay_passive: Some(false),
        ..Default::default()
    };
    for n in ["Alphaland", "Betaland"] {
        s.ruleset.sim.countries.insert(n.into(), country());
    }
    s.register("countries", "Gammaland");
    s.ruleset
        .sim
        .countries
        .insert("Gammaland".into(), country());
    s.setup.houses.push(HouseSetup {
        name: "Gamma".into(),
        country: "Gammaland".into(),
        ..s.setup.houses[0].clone()
    });
    for (i, (x, y)) in [(40i16, 40i16), (52, 40), (40, 52), (52, 52)]
        .into_iter()
        .enumerate()
    {
        s.map.waypoints.insert(i as u16, (x, y));
    }
    s.setup.sandbox = false;
    s.setup.bases = false;
    s.setup.unit_count = 0;
    s.vehicle("TANK", unit_type(TRACK, 4, 5));
    s.ruleset.sim.general.economy.allies_allowed = Some(true);
    let mut w = s.build();
    let [a, b, _] = [0u8, 1, 2].map(|h| {
        let c = cell(i32::from(h) * 4 - 12, 0);
        testkit::spawn(&mut w, Kind::Vehicle, "TANK", h, c, 0)
    });
    // One shot of a burst has been made at Beta's unit.
    w.obj_mut(a).unwrap().target = Some(Target::Object(b));
    w.obj_mut(a).unwrap().combat.burst = 1;
    victory::alliance(&mut w, A, B);
    assert!(w.allied(Some(A), Some(B)));
    assert_eq!(w.obj(a).unwrap().target, None, "target dropped");
    assert_eq!(w.obj(a).unwrap().combat.burst, 0, "burst counter reset");
}
