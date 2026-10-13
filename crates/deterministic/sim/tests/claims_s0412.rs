//! Claim tests for S-0412 (skirmish setup rules) on synthetic data.

mod common;

use common::Scene;
use glasnost_rules::sim::{Country, Decimal, DifficultyRow};
use glasnost_sim::econ::setup::{
    self, HouseValues, LobbyChoice, Player, Row, SetupError, Settings,
};
use glasnost_sim::load::LoadError;
use glasnost_sim::world::{GameSetup, HouseId, HouseRole, HouseSetup, World};

const A: HouseId = HouseId(0);
const B: HouseId = HouseId(1);

fn country(passive: bool) -> Country {
    Country {
        multiplay_passive: Some(passive),
        ..Default::default()
    }
}

/// A skirmish scene: Alpha and Beta (person houses, not passive), start
/// waypoints 0 to 3, no starting units, bases off.
fn skirmish() -> Scene {
    let mut s = Scene::new();
    for n in ["Alphaland", "Betaland"] {
        s.ruleset.sim.countries.insert(n.into(), country(false));
    }
    for (i, x) in [40i16, 52, 40, 52].into_iter().enumerate() {
        let y = if i < 2 { 40 } else { 52 };
        s.map.waypoints.insert(i as u16, (x, y));
    }
    s.setup.sandbox = false;
    s.setup.bases = false;
    s.setup.unit_count = 0;
    s
}

/// Adds a country and a house of it.
fn add_house(s: &mut Scene, name: &str, c: &str, role: HouseRole, human: bool, passive: bool) {
    if !s.ruleset.sim.countries.contains_key(c) {
        s.register("countries", c);
        s.ruleset.sim.countries.insert(c.into(), country(passive));
    }
    s.setup.houses.push(HouseSetup {
        name: name.into(),
        country: c.into(),
        human,
        start: None,
        role,
        ai: false,
    });
}

/// Adds the neutral and special houses (countries "Neutral" and
/// "Special").
fn add_neutral_special(s: &mut Scene, passive: bool) {
    add_house(s, "Neutral", "Neutral", HouseRole::Neutral, false, passive);
    add_house(s, "Special", "Special", HouseRole::Special, false, passive);
}

fn values(s: &mut Scene, i: usize) -> &mut HouseValues {
    let v = &mut s.setup.skirmish.houses;
    if v.len() <= i {
        v.resize(i + 1, HouseValues::default());
    }
    &mut v[i]
}

fn load(s: &Scene) -> Result<World, LoadError> {
    glasnost_sim::load::new_game(
        s.sim_rules(),
        &s.map,
        std::sync::Arc::new(s.catalogue.clone()),
        &s.setup,
    )
}

// spec: S-0412 claim 1 (the setup supplies every value; a value left unset takes the default the rules offer)
#[test]
fn claim_1_setup_values_and_defaults() {
    let mut s = skirmish();
    let w = s.build();
    // spec: S-0412 Import mapping (absent keys: credits 10000, short game no, crates yes, superweapons yes, build off ally yes, redeploy yes)
    assert_eq!(setup::credits_setting(&w), 10000);
    assert!(!setup::short_game(&w));
    assert!(setup::crates(&w));
    assert!(setup::superweapons_allowed(&w));
    assert!(setup::build_off_ally(&w));
    assert!(setup::redeploy(&w));
    s.ruleset.sim.general.economy.money = Some(4000);
    s.ruleset.sim.general.economy.crates = Some(false);
    let w = s.build();
    assert_eq!(setup::credits_setting(&w), 4000);
    assert!(!setup::crates(&w));
    s.setup.skirmish = Settings {
        credits: Some(2500),
        short_game: Some(true),
        superweapons: Some(false),
        crates: Some(true),
        build_off_ally: Some(false),
        redeploy: Some(false),
        ..Default::default()
    };
    s.setup.bases = true;
    let w = s.build();
    assert_eq!(setup::credits_setting(&w), 2500);
    assert!(setup::short_game(&w));
    assert!(!setup::superweapons_allowed(&w));
    assert!(setup::crates(&w));
    assert!(!glasnost_sim::econ::placement::build_off_ally(&w));
    assert!(!glasnost_sim::econ::deploy::redeploys(&w));
}

// spec: S-0412 claim 2 (every player house, observer included, starts with the credits setting; the neutral and special houses with 0)
#[test]
fn claim_2_starting_credits() {
    let mut s = skirmish();
    add_neutral_special(&mut s, true);
    s.setup.skirmish.credits = Some(7300);
    values(&mut s, 1).observer = true;
    let w = s.build();
    assert_eq!(w.houses[0].credits, 7300);
    assert_eq!(w.houses[1].credits, 7300, "observer");
    assert_eq!(w.houses[2].credits, 0);
    assert_eq!(w.houses[3].credits, 0);
}

// spec: S-0412 claim 3 (extra = trunc(RZ₅₃(RZ₅₃(p × c) × M)); only a house neither human-controlled nor passive, never an AI house)
#[test]
fn claim_3_extra_credits() {
    let mode = glasnost_num::Rounding::TowardZero;
    let c = glasnost_sim::rules::nearest_double("0.01");
    // Each product rounds toward zero, so 20 % of 10000 comes out just below 2000 and truncates.
    assert_eq!(setup::extra_amount(20, 10000, c, mode), 1999);
    assert_eq!(
        setup::extra_amount(20, 10000, c, glasnost_num::Rounding::Nearest),
        2000
    );
    assert_eq!(setup::extra_amount(30, 0, c, mode), 0);
    // A non-passive special house with no left-over budget: M = 0, so 0; an AI house gets nothing extra.
    let mut s = skirmish();
    s.ruleset.sim.general.economy.multiplayer_aicm = Some(vec![10, 20, 30]);
    s.setup.houses[1].ai = true;
    add_neutral_special(&mut s, false);
    let w = s.build();
    assert_eq!(w.houses[1].credits, 10000, "AI house: the credits setting only");
    assert!(!w.houses[3].removed, "an AI house stops the cleanup");
    assert_eq!(w.houses[3].credits, 0);
}

// spec: S-0412 claim 3 (Edge cases: a percentage list too short for the index gives 0)
#[test]
fn claim_3_short_list() {
    let mut s = skirmish();
    s.ruleset.sim.general.economy.multiplayer_aicm = Some(vec![10]);
    s.setup.houses[1].ai = true;
    add_neutral_special(&mut s, false);
    let w = s.build();
    assert_eq!(w.houses[2].credits, 0);
}

// spec: S-0412 claims 4 and 5 (every house is created with difficulty index 1 in our engine)
#[test]
fn claim_4_difficulty_index() {
    let mut s = skirmish();
    s.setup.houses[1].ai = true;
    add_neutral_special(&mut s, true);
    let w = s.build();
    for h in &w.houses {
        assert_eq!(h.econ.setup.difficulty, 1);
    }
}

fn row(rof: &str, fp: &str) -> DifficultyRow {
    DifficultyRow {
        rof: Some(Decimal(rof.into())),
        firepower: Some(Decimal(fp.into())),
        ..Default::default()
    }
}

// spec: S-0412 claim 5 (the normal row for every player house, the handicap's row instead; neutral and special houses exactly 1.0); S-0413 Formulas (h = RZ₅₃(r × g))
#[test]
fn claim_5_rows_and_handicap() {
    let mut s = skirmish();
    let d = &mut s.ruleset.sim.difficulty;
    d.insert("easy".into(), row("0.5", "2"));
    d.insert("normal".into(), row("1", "1"));
    d.insert("difficult".into(), row("2", "0.5"));
    s.ruleset.sim.countries.get_mut("Betaland").unwrap().firepower = Some(Decimal("3".into()));
    add_neutral_special(&mut s, true);
    s.ruleset.sim.countries.get_mut("Special").unwrap().firepower = Some(Decimal("3".into()));
    values(&mut s, 0).handicap = Some(Row::Easy);
    values(&mut s, 1).handicap = Some(Row::Difficult);
    let w = s.build();
    let mode = w.rules().mode;
    let v = |t: &str| Decimal(t.into()).value(mode);
    assert_eq!(w.houses[0].combat.firepower, v("2"));
    assert_eq!(w.houses[0].combat.rof_factor, v("0.5"));
    assert_eq!(w.houses[1].combat.firepower, v("1.5"));
    assert_eq!(w.houses[1].combat.rof_factor, v("2"));
    for h in 2..4 {
        assert_eq!(w.houses[h].combat.firepower, v("1"));
        assert_eq!(w.houses[h].combat.rof_factor, v("1"));
    }
    // No handicap: the normal row.
    let mut s2 = skirmish();
    s2.ruleset.sim.difficulty = s.ruleset.sim.difficulty.clone();
    let w = s2.build();
    assert_eq!(w.houses[0].combat.firepower, v("1"));
}

fn lobby_scene(countries: usize) -> (Scene, GameSetup) {
    let mut s = skirmish();
    for i in 2..countries {
        let n = format!("Land{i}");
        s.register("countries", &n);
        s.ruleset.sim.countries.insert(n, country(i % 2 == 0));
    }
    let setup = s.setup.clone();
    (s, setup)
}

// spec: S-0412 claim 6 (k = R(0, 9), quirks.random_country_fixed_range: the first ten countries only, passive ones included; an index with no country draws again)
#[test]
fn claim_6_random_country() {
    let (s, _) = lobby_scene(12);
    let r = s.sim_rules();
    let first_ten: Vec<String> = r.country_names[..10].to_vec();
    let mut seen = std::collections::BTreeSet::new();
    for seed in 0..200 {
        let mut g = s.setup.clone();
        let choices = [LobbyChoice { country: true, colour: false }; 2];
        setup::resolve_lobby(&mut g, &r, &choices, &[], seed).unwrap();
        for h in &g.houses {
            assert!(first_ten.contains(&h.country), "{}", h.country);
            seen.insert(h.country.clone());
        }
    }
    assert!(seen.len() > 5);
    // A list of three: every draw ends on one of them.
    let (s, _) = lobby_scene(3);
    let r = s.sim_rules();
    for seed in 0..50 {
        let mut g = s.setup.clone();
        let choices = [LobbyChoice { country: true, colour: false }; 2];
        setup::resolve_lobby(&mut g, &r, &choices, &[], seed).unwrap();
        assert!(g.houses.iter().all(|h| r.country_names.contains(&h.country)));
    }
}

// spec: S-0412 claim 6 (switch off: uniform among the countries the setup offers)
#[test]
fn claim_6_switch_off() {
    let (mut s, _) = lobby_scene(12);
    s.ruleset.globals.quirks.random_country_fixed_range = false;
    let r = s.sim_rules();
    let offered = vec!["Land11".to_string(), "Land10".to_string()];
    for seed in 0..50 {
        let mut g = s.setup.clone();
        let choices = [LobbyChoice { country: true, colour: false }; 2];
        setup::resolve_lobby(&mut g, &r, &choices, &offered, seed).unwrap();
        assert!(g.houses.iter().all(|h| offered.contains(&h.country)));
    }
}

// spec: S-0412 claim 7 (the first free draw of R(0, 7); fixed slots and earlier players' slots are taken; an observer gets none; no free slot refuses the setup)
#[test]
fn claim_7_random_colour() {
    let (mut s, _) = lobby_scene(2);
    for _ in 0..6 {
        let n = s.setup.houses.len();
        s.setup.houses.push(HouseSetup {
            name: format!("P{n}"),
            ..s.setup.houses[0].clone()
        });
    }
    let r = s.sim_rules();
    for seed in 0..30 {
        let mut g = s.setup.clone();
        g.skirmish.houses = vec![HouseValues::default(); 8];
        g.skirmish.houses[0].colour = Some(5);
        g.skirmish.houses[7].observer = true;
        let mut choices = [LobbyChoice { country: false, colour: true }; 8];
        choices[0].colour = false;
        setup::resolve_lobby(&mut g, &r, &choices, &[], seed).unwrap();
        let slots: Vec<u8> = g.skirmish.houses.iter().filter_map(|v| v.colour).collect();
        assert_eq!(slots.len(), 7, "the observer has none");
        assert_eq!(g.skirmish.houses[0].colour, Some(5));
        let mut u = slots.clone();
        u.sort();
        u.dedup();
        assert_eq!(u.len(), 7, "unique");
        assert!(slots.iter().all(|&c| c <= 7));
    }
    // Nine players on random: the ninth finds no free slot.
    let mut g = s.setup.clone();
    g.houses.push(g.houses[0].clone());
    let choices = [LobbyChoice { country: false, colour: true }; 9];
    assert_eq!(
        setup::resolve_lobby(&mut g, &r, &choices, &[], 1),
        Err(SetupError::NoFreeColour(8))
    );
}

// spec: S-0412 claim 7 (the lobby's draws take nothing from the game's stream)
#[test]
fn claim_7_no_game_draw() {
    let (s, _) = lobby_scene(2);
    let r = s.sim_rules();
    let mut g = s.setup.clone();
    let before = load(&s).unwrap().rng.clone();
    let choices = [LobbyChoice { country: false, colour: true }; 2];
    setup::resolve_lobby(&mut g, &r, &choices, &[], 99).unwrap();
    let mut s2 = s;
    s2.setup = g;
    let after = load(&s2).unwrap().rng;
    assert_eq!(before, after);
}

// spec: S-0412 claim 8 (same lobby team allies both ways; the special house allies and is allied by every house; no team: enemies)
#[test]
fn claim_8_start_alliances() {
    let mut s = skirmish();
    add_house(&mut s, "Gamma", "Gammaland", HouseRole::Player, true, false);
    add_neutral_special(&mut s, true);
    values(&mut s, 0).team = Some(1);
    values(&mut s, 1).team = Some(1);
    values(&mut s, 2).team = Some(2);
    let w = s.build();
    let al = |a: u8, b: u8| w.allied(Some(HouseId(a)), Some(HouseId(b)));
    assert!(al(0, 1) && al(1, 0));
    assert!(!al(0, 2) && !al(2, 0));
    for h in 0..4u8 {
        assert!(al(h, 4) && al(4, h), "special and {h}");
    }
    assert!(!al(0, 3), "the neutral house is on no team");
}

// spec: S-0412 claim 8 (a house of the civilian side is never made allied: the civilian special house allies the others, no house allies it)
#[test]
fn claim_8_civilian_side() {
    let mut s = skirmish();
    add_neutral_special(&mut s, true);
    s.ruleset.sim.countries.get_mut("Special").unwrap().econ.side = Some("Civilian".into());
    s.ruleset.sim.countries.get_mut("Neutral").unwrap().econ.side = Some("Civilian".into());
    let w = s.build();
    let al = |a: u8, b: u8| w.allied(Some(HouseId(a)), Some(HouseId(b)));
    assert!(al(3, 0) && al(3, 1));
    assert!(!al(0, 3) && !al(1, 3) && !al(2, 3));
    assert!(!al(3, 2), "the neutral house is of the civilian side too");
}

// spec: S-0412 claim 9 (the superweapon setting S-0407 claim 7 reads)
#[test]
fn claim_9_superweapon_setting() {
    let mut s = skirmish();
    s.setup.skirmish.superweapons = Some(false);
    let w = s.build();
    assert!(!setup::superweapons_allowed(&w));
    s.ruleset.sim.general.economy.super_weapons_allowed = Some(false);
    s.setup.skirmish.superweapons = None;
    assert!(!setup::superweapons_allowed(&s.build()));
}

// spec: S-0412 claim 10 (n = min(max(C_min, H), C_max), H the human participants; quirks.crate_count_humans_only; crate setting off: none)
#[test]
fn claim_10_crate_count() {
    let mut s = skirmish();
    let crates = |s: &Scene| s.build().econ.start.crates;
    assert_eq!(crates(&s), 2);
    s.setup.houses[1].ai = true;
    assert_eq!(crates(&s), 1);
    s.setup.houses[0].ai = true;
    assert_eq!(crates(&s), 1, "C_min 1");
    s.ruleset.sim.general.economy.crate_minimum = Some(0);
    assert_eq!(crates(&s), 0, "no human participant and C_min 0");
    s.ruleset.globals.quirks.crate_count_humans_only = false;
    assert_eq!(crates(&s), 2, "switch off: every player house");
    s.ruleset.sim.general.economy.crate_maximum = Some(1);
    assert_eq!(crates(&s), 1);
    s.setup.skirmish.crates = Some(false);
    assert_eq!(crates(&s), 0);
}

// spec: S-0412 claim 11 (quirks.short_game_without_bases off: the setup is refused)
#[test]
fn claim_11_refused() {
    let mut s = skirmish();
    s.setup.skirmish.short_game = Some(true);
    s.ruleset.globals.quirks.short_game_without_bases = false;
    assert!(matches!(
        load(&s),
        Err(LoadError::Setup(SetupError::ShortGameWithoutBases))
    ));
}

// spec: S-0412 claim 11 (short game, bases off, nothing that counts: every house fails in tick 1 and every player house loses)
#[test]
fn claim_11_everyone_fails_in_tick_1() {
    let mut s = skirmish();
    s.setup.skirmish.short_game = Some(true);
    let mut w = s.build();
    common::run(&mut w, 1);
    assert!(!glasnost_sim::econ::victory::defeated(&w, A));
    common::run(&mut w, 1);
    assert!(glasnost_sim::econ::victory::defeated(&w, A));
    assert!(glasnost_sim::econ::victory::defeated(&w, B));
    use glasnost_sim::econ::victory::{Verdict, verdict};
    assert_eq!(verdict(&w, A), Some(Verdict::Loss));
    assert_eq!(verdict(&w, B), Some(Verdict::Loss));
    assert_eq!(w.econ.end.ended, Some(1));
}

// spec: S-0412 claim 12 (an AI house takes a computer slot: the cleanup does not run; it is a contender and never person-played)
#[test]
fn claim_12_ai_houses() {
    let mut s = skirmish();
    add_neutral_special(&mut s, false);
    let w = s.build();
    assert!(w.houses[2].removed, "no AI house: the cleanup runs");
    s.setup.houses[1].ai = true;
    let w = s.build();
    assert!(!w.houses[2].removed);
    assert!(!setup::person_played(&w, B));
    assert!(setup::person_played(&w, A));
    assert!(glasnost_sim::econ::victory::contender(&w, B));
}

// spec: S-0412 claim 13 (a mode of the all-base-units family: a seam, refused)
#[test]
fn claim_13_all_base_units_refused() {
    let mut s = skirmish();
    s.setup.skirmish.all_base_units = true;
    assert!(matches!(
        load(&s),
        Err(LoadError::Setup(SetupError::AllBaseUnitsMode))
    ));
}

// spec: S-0412 claim 14 (removed houses get no alliance and no wall-owner rewrite; alliances and flags are in place before tick 0)
#[test]
fn claim_14_game_start() {
    let mut s = skirmish();
    add_neutral_special(&mut s, true);
    // A non-passive extra house the cleanup removes.
    add_house(&mut s, "Extra", "Extraland", HouseRole::Player, false, false);
    s.ruleset.sim.countries.get_mut("Extraland").unwrap().wall_owner = Some(false);
    let w = s.build();
    assert_eq!(w.tick, 0);
    assert!(w.houses[4].removed);
    assert!(!w.allied(Some(HouseId(4)), Some(HouseId(3))));
    assert!(!w.allied(Some(HouseId(3)), Some(HouseId(4))));
    assert!(w.allied(Some(A), Some(HouseId(3))));
    let extra = w.houses[4].country;
    assert!(!setup::wall_owner(&w, extra), "no rewrite for a removed house");
    assert!(w.econ.end.active);
}

// spec: S-0412 claim 1 and S-0009 claim 19 (the setup builder: players in house order, then the neutral and special houses)
#[test]
fn skirmish_builder() {
    let mut s = skirmish();
    add_neutral_special(&mut s, true);
    let r = s.sim_rules();
    let players = [
        Player {
            name: "Ann".into(),
            country: "Alphaland".into(),
            ..Default::default()
        },
        Player {
            name: "Bot".into(),
            country: "Betaland".into(),
            ai: true,
            values: HouseValues {
                team: Some(3),
                ..Default::default()
            },
            ..Default::default()
        },
    ];
    let g = setup::skirmish(&r, &players, Settings::default(), 0, 7);
    assert_eq!(g.houses.len(), 4);
    assert!(g.houses.iter().take(2).all(|h| h.human && h.role == HouseRole::Player));
    assert!(g.houses[1].ai);
    assert_eq!(g.houses[2].role, HouseRole::Neutral);
    assert_eq!(g.houses[3].role, HouseRole::Special);
    assert_eq!(g.skirmish.houses[1].team, Some(3));
    assert!(!g.sandbox);
    let mut s2 = s;
    s2.setup = g;
    let w = s2.build();
    assert_eq!(w.houses[0].credits, 10000);
}
