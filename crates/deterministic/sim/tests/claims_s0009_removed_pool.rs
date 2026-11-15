//! Claim test for S-0009 claim 45 with claim 28 (sim batch S20, #376): a
//! house that the unused-house cleanup removed is no longer in the game,
//! so its country does not widen the starting-unit budget pool. Synthetic
//! data.

mod common;

use common::{Scene, TRACK, unit_type};
use glasnost_sim::world::{HouseId, HouseRole, HouseSetup, World};

fn load(s: &Scene) -> World {
    glasnost_sim::load::new_game(
        s.sim_rules(),
        &s.map,
        std::sync::Arc::new(s.catalogue.clone()),
        &s.setup,
    )
    .expect("loads")
}

/// One human player (Alpha) and a neutral house whose country is not
/// passive, so the cleanup removes it. TANK (100) and MAN (50) belong to
/// Alpha's country; NTANK (1000) belongs to `ntank_owner` only.
fn scene(ntank_owner: Option<&str>) -> Scene {
    let mut s = Scene::new();
    s.setup.sandbox = false;
    let mut tank = unit_type(TRACK, 4, 5);
    tank.cost = Some(100);
    tank.tech_level = Some(1);
    tank.owner = Some(vec!["Alphaland".into()]);
    s.vehicle("TANK", tank);
    let mut man = unit_type(common::FOOT, 4, 5);
    man.cost = Some(50);
    man.tech_level = Some(1);
    man.owner = Some(vec!["Alphaland".into()]);
    s.infantry("MAN", man);
    s.register("countries", "Neutral");
    s.ruleset.sim.countries.insert(
        "Neutral".into(),
        glasnost_rules::sim::Country {
            multiplay_passive: Some(false),
            ..Default::default()
        },
    );
    if let Some(owner) = ntank_owner {
        let mut ntank = unit_type(TRACK, 4, 5);
        ntank.cost = Some(1000);
        ntank.tech_level = Some(1);
        ntank.owner = Some(vec![owner.into()]);
        s.vehicle("NTANK", ntank);
    }
    s.map.waypoints.insert(0, (40, 40));
    s.map.waypoints.insert(1, (52, 40));
    s.setup.unit_count = 1;
    s.setup.bases = false;
    s.setup.skirmish.credits = Some(0);
    s.setup.houses.truncate(1);
    s.setup.houses.push(HouseSetup {
        name: "Neutral".into(),
        country: "Neutral".into(),
        human: false,
        start: None,
        role: HouseRole::Neutral,
        ai: false,
    });
    s
}

fn alpha_units(w: &World) -> usize {
    w.objects
        .values()
        .filter(|o| o.owner == Some(HouseId(0)))
        .count()
}

// spec: S-0009 claim 45 (the pool: types of the country of at least one house that is not passive; a house the cleanup of claim 28 removed is no longer in the game)
#[test]
fn claim_45_removed_house_country_stays_out_of_the_budget_pool() {
    // Control: no NTANK. Pool = TANK, MAN: n = 2, C = 150, B = trunc(151 / 2) = 75: one TANK.
    let control = load(&scene(None));
    assert!(
        control.houses[1].removed,
        "the cleanup removed the neutral house"
    );
    assert_eq!(alpha_units(&control), 1, "control");
    // NTANK owned only by the removed house's country.
    let w = load(&scene(Some("Neutral")));
    assert!(w.houses[1].removed, "the cleanup removed the neutral house");
    assert_eq!(
        alpha_units(&w),
        alpha_units(&control),
        "a type owned only by the removed house's country changed the budget"
    );
    assert_eq!(w.rng, control.rng, "and the draws");
}
