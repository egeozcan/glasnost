//! Claim tests for S-0414 (victory and defeat) on synthetic data.

mod common;

use common::{FOOT, Scene, TRACK, cell, unit_type};
use glasnost_rules::sim::{BuildingType, Country, Decimal, UnitType};
use glasnost_sim::econ::victory::{self, Verdict, defeated, verdict};
use glasnost_sim::rules::Kind;
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, HouseRole, HouseSetup, ObjectId, Target, World};

const A: HouseId = HouseId(0);
const B: HouseId = HouseId(1);
const C: HouseId = HouseId(2);

fn country(passive: bool) -> Country {
    Country {
        multiplay_passive: Some(passive),
        ..Default::default()
    }
}

fn building(f: [i32; 2]) -> BuildingType {
    BuildingType {
        strength: Some(300),
        foundation: Some(f),
        ..Default::default()
    }
}

/// A skirmish of Alpha, Beta and Gamma (person houses, AI for Beta and
/// Gamma when `ai`), with the types:
/// - vehicles TANK, HARV (resource gatherer), DS (not scored), INS
///   (insignificant), MCV1 … MCV4 (base-unit list positions 1 … 4);
/// - infantry MAN, NOM (nominal), DSMAN (not scored); aircraft JET;
/// - buildings HUT, REF (third of the refinery list), VB1 (1 × 1, packs up
///   into TANK), VB2 (2 × 2, packs up into HARV), VB3 (2 × 2, packs up into
///   TANK: a counted building), INSB (insignificant).
fn scene(ai: bool) -> Scene {
    let mut s = Scene::new();
    for n in ["Alphaland", "Betaland"] {
        s.ruleset.sim.countries.insert(n.into(), country(false));
    }
    s.register("countries", "Gammaland");
    s.ruleset
        .sim
        .countries
        .insert("Gammaland".into(), country(false));
    s.setup.houses.push(HouseSetup {
        name: "Gamma".into(),
        country: "Gammaland".into(),
        ..s.setup.houses[0].clone()
    });
    if ai {
        s.setup.houses[1].ai = true;
        s.setup.houses[2].ai = true;
    }
    for (i, (x, y)) in [(40i16, 40i16), (52, 40), (40, 52), (52, 52)]
        .into_iter()
        .enumerate()
    {
        s.map.waypoints.insert(i as u16, (x, y));
    }
    s.setup.sandbox = false;
    s.setup.bases = false;
    s.setup.unit_count = 0;
    let v = |f: &dyn Fn(&mut UnitType)| {
        let mut t = unit_type(TRACK, 4, 5);
        f(&mut t);
        t
    };
    s.vehicle("TANK", v(&|_| {}));
    s.vehicle("HARV", v(&|t| t.econ.resource_gatherer = Some(true)));
    s.vehicle("DS", v(&|t| t.combat.dont_score = Some(true)));
    s.vehicle("INS", v(&|t| t.combat.insignificant = Some(true)));
    for i in 1..=4 {
        s.vehicle(&format!("MCV{i}"), v(&|_| {}));
    }
    s.ruleset.sim.general.base_unit = Some((1..=4).map(|i| format!("MCV{i}")).collect());
    s.infantry("MAN", unit_type(FOOT, 4, 5));
    let mut nom = unit_type(FOOT, 4, 5);
    nom.econ.nominal = Some(true);
    s.infantry("NOM", nom);
    let mut dsman = unit_type(FOOT, 4, 5);
    dsman.combat.dont_score = Some(true);
    s.infantry("DSMAN", dsman);
    s.aircraft("JET", unit_type(TRACK, 4, 5));
    s.building("HUT", building([1, 1]));
    s.building("REF", building([2, 2]));
    let mut vb1 = building([1, 1]);
    vb1.undeploys_into = Some("TANK".into());
    s.building("VB1", vb1);
    let mut vb2 = building([2, 2]);
    vb2.undeploys_into = Some("HARV".into());
    s.building("VB2", vb2);
    let mut vb3 = building([2, 2]);
    vb3.undeploys_into = Some("TANK".into());
    s.building("VB3", vb3);
    let mut insb = building([1, 1]);
    insb.combat.insignificant = Some(true);
    s.building("INSB", insb);
    s.ruleset.sim.general.economy.build_refinery =
        Some(vec!["HUT".into(), "HUT".into(), "REF".into()]);
    s
}

/// Gives each of the three houses a TANK so that nobody fails.
fn armed(w: &mut World) -> [ObjectId; 3] {
    [0u8, 1, 2].map(|h| spawn(w, Kind::Vehicle, "TANK", h, i32::from(h) * 4 - 12, 0))
}

fn spawn(w: &mut World, kind: Kind, name: &str, house: u8, dx: i32, dy: i32) -> ObjectId {
    testkit::spawn(w, kind, name, house, cell(dx, dy), 0)
}

fn kill(w: &mut World, id: ObjectId) {
    glasnost_sim::objects::remove(w, id);
}

fn on_map(w: &World, h: HouseId) -> [i32; 4] {
    w.houses[usize::from(h.0)].econ.result.on_map
}

// spec: S-0414 claim 1 (no test in tick 0; every contender in every later tick; passive houses and observers' houses never)
#[test]
fn claim_1_when_houses_are_tested() {
    let mut s = scene(false);
    s.ruleset
        .sim
        .countries
        .get_mut("Gammaland")
        .unwrap()
        .multiplay_passive = Some(true);
    let mut w = s.build();
    spawn(&mut w, Kind::Vehicle, "TANK", 1, 0, 0);
    common::run(&mut w, 1);
    assert!(!defeated(&w, A), "no test in tick 0");
    common::run(&mut w, 1);
    assert!(defeated(&w, A));
    assert!(!defeated(&w, B));
    assert!(!defeated(&w, C), "a passive house is never tested");
    // An observer's house is never tested.
    let mut s = scene(false);
    s.setup.skirmish.houses = vec![Default::default(); 3];
    s.setup.skirmish.houses[0].observer = true;
    let mut w = s.build();
    armed(&mut w);
    let id = w.objects.values().find(|o| o.owner == Some(A)).unwrap().id;
    kill(&mut w, id);
    common::run(&mut w, 3);
    assert!(!defeated(&w, A));
}

// spec: S-0414 claim 1 (the sandbox is no skirmish: no test)
#[test]
fn claim_1_sandbox_untested() {
    let s = Scene::new();
    let mut w = s.build();
    common::run(&mut w, 3);
    assert!(!defeated(&w, A));
}

// spec: S-0414 claim 2 (short game: a counted building or a vehicle of the first three base-unit types; insignificant vehicles and buildings counting as vehicles do not keep a house alive)
#[test]
fn claim_2_short_game_test() {
    let mut s = scene(false);
    s.setup.skirmish.short_game = Some(true);
    s.setup.bases = true;
    let mut w = s.build();
    spawn(&mut w, Kind::Vehicle, "MCV3", 0, -10, 0);
    spawn(&mut w, Kind::Vehicle, "MCV4", 1, 0, 0);
    spawn(&mut w, Kind::Vehicle, "TANK", 1, 2, 0);
    spawn(&mut w, Kind::Building, "VB1", 1, 4, 4);
    spawn(&mut w, Kind::Building, "HUT", 2, 10, 0);
    common::run(&mut w, 2);
    assert!(!defeated(&w, A), "third base-unit position");
    assert!(
        defeated(&w, B),
        "fourth position, a tank and a vehicle building"
    );
    assert!(!defeated(&w, C), "counted building");
}

// spec: S-0414 Edge cases (quirks.short_game_first_three_base_units off: every position counts)
#[test]
fn claim_2_every_position() {
    let mut s = scene(false);
    s.setup.skirmish.short_game = Some(true);
    s.setup.bases = true;
    s.ruleset.globals.quirks.short_game_first_three_base_units = false;
    let mut w = s.build();
    armed(&mut w);
    spawn(&mut w, Kind::Vehicle, "MCV4", 1, 0, 0);
    common::run(&mut w, 2);
    assert!(defeated(&w, A));
    assert!(!defeated(&w, B));
}

// spec: S-0414 claim 3 (S = B + V + I + A + R ≠ 0; a building counting as a vehicle keeps nobody alive unless it is the refinery-list building)
#[test]
fn claim_3_normal_test() {
    let mut s = scene(false);
    let mut w = s.build();
    spawn(&mut w, Kind::Building, "VB1", 0, -10, 0);
    spawn(&mut w, Kind::Building, "REF", 1, 0, 0);
    spawn(&mut w, Kind::Building, "VB3", 2, 10, 0);
    common::run(&mut w, 2);
    assert!(defeated(&w, A), "a building that counts as a vehicle");
    assert!(!defeated(&w, B), "R");
    assert!(!defeated(&w, C), "a counted building");
    // Switch off: every building that counts as a vehicle is on R.
    s.ruleset.globals.quirks.vehicle_buildings_keep_nobody_alive = false;
    let mut w = s.build();
    spawn(&mut w, Kind::Building, "VB2", 0, -10, 0);
    armed(&mut w);
    let id = w
        .objects
        .values()
        .find(|o| o.owner == Some(A) && o.kind == Kind::Vehicle)
        .unwrap()
        .id;
    kill(&mut w, id);
    common::run(&mut w, 2);
    assert!(!defeated(&w, A));
    assert_eq!(on_map(&w, A)[3], 1);
}

// spec: S-0414 claim 4 (the on-map counts: entry and leaving; insignificant counted; not-scored vehicle never taken off; not-scored infantry never counted)
#[test]
fn claim_4_counts() {
    let mut w = scene(false).build();
    let t = spawn(&mut w, Kind::Vehicle, "TANK", 0, 0, 0);
    let i = spawn(&mut w, Kind::Vehicle, "INS", 0, 2, 0);
    let d = spawn(&mut w, Kind::Vehicle, "DS", 0, 4, 0);
    let m = spawn(&mut w, Kind::Infantry, "MAN", 0, 6, 0);
    spawn(&mut w, Kind::Infantry, "DSMAN", 0, 8, 0);
    let j = spawn(&mut w, Kind::Aircraft, "JET", 0, 10, 0);
    assert_eq!(on_map(&w, A), [3, 1, 1, 0]);
    for id in [t, i, d, m, j] {
        kill(&mut w, id);
    }
    assert_eq!(on_map(&w, A), [1, 0, 0, 0], "the not-scored vehicle stays");
    assert_eq!(victory::unscored_vehicle_entries(&w, A, 2), 1);
    assert!(victory::passes(&w, A), "it keeps the house alive");
}

// spec: S-0414 Edge cases (quirks.unscored_vehicle_stays_counted off: never counted)
#[test]
fn claim_4_unscored_switch_off() {
    let mut s = scene(false);
    s.ruleset.globals.quirks.unscored_vehicle_stays_counted = false;
    let mut w = s.build();
    let d = spawn(&mut w, Kind::Vehicle, "DS", 0, 4, 0);
    assert_eq!(on_map(&w, A), [0; 4]);
    kill(&mut w, d);
    assert_eq!(on_map(&w, A), [0; 4]);
    assert_eq!(victory::unscored_vehicle_entries(&w, A, 2), 0);
}

// spec: S-0414 claim 4 (a nominal survivor is not added at its entry but taken off when it leaves, quirks.nominal_survivor_count)
#[test]
fn claim_4_nominal_survivor() {
    for on in [true, false] {
        let mut s = scene(false);
        s.ruleset.globals.quirks.nominal_survivor_count = on;
        let mut w = s.build();
        let r = w.rules_arc();
        let ti = r.index_of(Kind::Infantry, "NOM").unwrap();
        let id = glasnost_sim::objects::create(&mut w, Kind::Infantry, ti, Some(A));
        victory::mark_nominal_survivor(&mut w, id);
        let p = glasnost_sim::world::Pos::centre(cell(0, 0), 0);
        glasnost_sim::objects::place(&mut w, id, p, 0);
        assert_eq!(on_map(&w, A)[1], if on { 0 } else { 1 });
        kill(&mut w, id);
        assert_eq!(on_map(&w, A)[1], if on { -1 } else { 0 });
        // A negative S passes.
        assert_eq!(victory::passes(&w, A), on);
    }
}

// spec: S-0414 claim 4 (a change of owner on the map: a leaving for the old owner and an entry for the new; a not-scored vehicle stays on the old owner's V)
#[test]
fn claim_4_change_of_owner() {
    let mut w = scene(false).build();
    let t = spawn(&mut w, Kind::Vehicle, "TANK", 0, 0, 0);
    let d = spawn(&mut w, Kind::Vehicle, "DS", 0, 2, 0);
    for id in [t, d] {
        w.obj_mut(id).unwrap().owner = Some(B);
        glasnost_sim::econ::events::owner_changed(&mut w, id, Some(A));
    }
    assert_eq!(on_map(&w, A)[0], 1, "the not-scored vehicle stays");
    assert_eq!(on_map(&w, B)[0], 2);
}

// spec: S-0414 claims 2 and 4 (B and the short-game count cover objects off the map, in production included)
#[test]
fn claim_4_off_map_existence() {
    let mut s = scene(false);
    s.setup.skirmish.short_game = Some(true);
    s.setup.bases = true;
    let mut w = s.build();
    armed(&mut w);
    let r = w.rules_arc();
    let hut = r.index_of(Kind::Building, "HUT").unwrap();
    glasnost_sim::objects::create(&mut w, Kind::Building, hut, Some(A));
    let id = w
        .objects
        .values()
        .find(|o| o.owner == Some(A) && o.kind == Kind::Vehicle)
        .unwrap()
        .id;
    kill(&mut w, id);
    common::run(&mut w, 2);
    assert!(!defeated(&w, A), "a building off the map still counts");
}

// spec: S-0414 claim 4 (map-placed vehicles and starting units enter the map through the same hook)
#[test]
fn claim_4_pre_placed_vehicle() {
    let mut s = scene(false);
    s.place_unit(0, "TANK", cell(0, 0), 0);
    let mut w = s.build();
    assert_eq!(on_map(&w, A)[0], 1);
    common::run(&mut w, 2);
    assert!(!defeated(&w, A));
}

// spec: S-0414 claim 5 (defeat: every object hit for its health, ignoring defences; the house marked defeated; radar on and the map explored)
#[test]
fn claim_5_defeat() {
    let mut w = scene(false).build();
    armed(&mut w);
    let man = spawn(&mut w, Kind::Infantry, "DSMAN", 1, 6, 6);
    let hut = spawn(&mut w, Kind::Building, "INSB", 1, 8, 8);
    let id = w
        .objects
        .values()
        .find(|o| o.owner == Some(B) && o.kind == Kind::Vehicle)
        .unwrap()
        .id;
    kill(&mut w, id);
    common::run(&mut w, 2);
    assert!(defeated(&w, B));
    assert!(w.obj(man).is_none_or(|o| o.health == 0 || !o.on_map));
    assert!(w.obj(hut).is_none_or(|o| o.health == 0 || !o.on_map));
    assert!(glasnost_sim::econ::radar::available(&w, B));
    assert!(glasnost_sim::seams::game_over(&w, B));
    assert!(!victory::ended(&w), "two hostile contenders remain");
}

// spec: S-0414 claim 6 (the end test after a defeat: allied victory; one-way alliances do not end it; a person's defeat does not end the game while its allies fight)
#[test]
fn claim_6_end_test() {
    let mut w = scene(false).build();
    let [_, _, c] = armed(&mut w);
    // Alpha allies Beta one way.
    w.houses[0].allies = vec![A, B];
    kill(&mut w, c);
    common::run(&mut w, 2);
    assert!(defeated(&w, C));
    assert!(!victory::ended(&w), "one-way alliance");
    // Mutual alliance: the end.
    let mut w = scene(false).build();
    let [a, _, c] = armed(&mut w);
    w.houses[0].allies = vec![A, B];
    w.houses[1].allies = vec![A, B];
    kill(&mut w, a);
    common::run(&mut w, 2);
    assert!(defeated(&w, A));
    assert!(!victory::ended(&w), "the person's allies fight on");
    kill(&mut w, c);
    common::run(&mut w, 1);
    assert!(victory::ended(&w));
    assert_eq!(verdict(&w, A), Some(Verdict::Loss));
    assert_eq!(verdict(&w, B), Some(Verdict::Win));
    assert_eq!(verdict(&w, C), Some(Verdict::Loss));
}

// spec: S-0414 claim 7 (a result for every player house; none for the neutral and special houses or an observer's house)
#[test]
fn claim_7_results() {
    let mut s = scene(false);
    for (n, role) in [
        ("Neutral", HouseRole::Neutral),
        ("Special", HouseRole::Special),
    ] {
        s.register("countries", n);
        s.ruleset.sim.countries.insert(n.into(), country(true));
        s.setup.houses.push(HouseSetup {
            name: n.into(),
            country: n.into(),
            human: false,
            start: None,
            role,
            ai: false,
        });
    }
    s.setup.skirmish.houses = vec![Default::default(); 3];
    s.setup.skirmish.houses[2].observer = true;
    let mut w = s.build();
    spawn(&mut w, Kind::Vehicle, "TANK", 1, 0, 0);
    common::run(&mut w, 2);
    assert!(victory::ended(&w));
    assert_eq!(verdict(&w, A), Some(Verdict::Loss));
    assert_eq!(verdict(&w, B), Some(Verdict::Win));
    assert_eq!(verdict(&w, C), None, "observer");
    assert_eq!(verdict(&w, HouseId(3)), None);
    assert_eq!(verdict(&w, HouseId(4)), None);
}

// spec: S-0414 claim 8 and Formulas (D = trunc(RZ₅₃(d × 900)): 26 at the default, 27 with quirks.chop_rounding off; over in tick t + max(D, 1))
#[test]
fn claim_8_end_delay() {
    let mut s = scene(false);
    assert_eq!(s.sim_rules().econ.victory.delay, 26);
    s.ruleset.globals.quirks.chop_rounding = false;
    assert_eq!(s.sim_rules().econ.victory.delay, 27);
    let mut s = scene(false);
    s.ruleset.sim.general.economy.savour_delay = Some(Decimal("0".into()));
    let mut w = s.build();
    spawn(&mut w, Kind::Vehicle, "TANK", 1, 0, 0);
    spawn(&mut w, Kind::Vehicle, "TANK", 2, 4, 0);
    w.houses[1].allies = vec![B, C];
    w.houses[2].allies = vec![B, C];
    common::run(&mut w, 2);
    assert_eq!(w.econ.end.ended, Some(1));
    assert_eq!(
        w.econ.end.over,
        Some(2),
        "a delay of 0 still lets one tick run"
    );
    assert!(victory::outcome(&w).is_none());
    common::run(&mut w, 1);
    let o = victory::outcome(&w).expect("over");
    assert_eq!(
        o.results,
        vec![(A, Verdict::Loss), (B, Verdict::Win), (C, Verdict::Win)]
    );
}

// spec: S-0414 claim 9 (a house defeated during the end delay loses; the delay restarts only for a person-played house with a win pending; quirks.defeat_during_end_delay)
#[test]
fn claim_9_defeat_during_delay() {
    for (on, ai) in [(true, false), (true, true), (false, false)] {
        let mut s = scene(ai);
        s.ruleset.globals.quirks.defeat_during_end_delay = on;
        let mut w = s.build();
        let [_, b, _] = armed(&mut w);
        w.houses[0].allies = vec![A, B];
        w.houses[1].allies = vec![A, B];
        let id = w.objects.values().find(|o| o.owner == Some(C)).unwrap().id;
        kill(&mut w, id);
        common::run(&mut w, 2);
        assert_eq!(w.econ.end.ended, Some(1));
        assert_eq!(w.econ.end.over, Some(27));
        // Beta (person-played unless ai) is wiped out during the delay.
        kill(&mut w, b);
        common::run(&mut w, 3);
        assert!(defeated(&w, B));
        let expect = if on { Verdict::Loss } else { Verdict::Win };
        assert_eq!(verdict(&w, B), Some(expect), "on {on} ai {ai}");
        // Beta is defeated in tick 2: a restart gives 2 + 26.
        let over = if on && !ai { 2 + 26 } else { 27 };
        assert_eq!(w.econ.end.over, Some(over), "on {on} ai {ai}");
    }
}

// spec: S-0414 claim 10 (the last houses of two sides failing in one tick: the first defeat ends the game, the later house fails too and loses with quirks.defeat_during_end_delay on, keeps its win with it off)
#[test]
fn claim_10_same_tick() {
    for on in [true, false] {
        let mut s = scene(false);
        s.ruleset.globals.quirks.defeat_during_end_delay = on;
        s.setup.houses.truncate(2);
        let mut w = s.build();
        common::run(&mut w, 2);
        assert!(defeated(&w, A) && defeated(&w, B));
        assert_eq!(verdict(&w, A), Some(Verdict::Loss));
        let b = if on { Verdict::Loss } else { Verdict::Win };
        assert_eq!(verdict(&w, B), Some(b));
    }
}

// spec: S-0414 claim 11 (surrender: marked in the order's tick, objects destroyed in the next house update, defeated in that tick; nothing for a defeated or pending house or an observer)
#[test]
fn claim_11_surrender() {
    let mut w = scene(false).build();
    let [a, ..] = armed(&mut w);
    common::run(&mut w, 2);
    victory::surrender(&mut w, A);
    assert!(w.houses[0].econ.result.doomed);
    assert!(w.obj(a).is_some_and(|o| o.health > 0));
    let t = w.tick;
    common::run(&mut w, 1);
    assert!(defeated(&w, A));
    assert_eq!(w.tick, t + 1);
    assert!(w.obj(a).is_none_or(|o| o.health == 0));
    // A defeated house: nothing.
    victory::surrender(&mut w, A);
    assert!(!w.houses[0].econ.result.doomed);
}

// spec: S-0414 claims 11 and Edge cases (a pending result or an observer's house: the order does nothing)
#[test]
fn claim_11_refused() {
    let mut s = scene(false);
    s.setup.skirmish.houses = vec![Default::default(); 3];
    s.setup.skirmish.houses[2].observer = true;
    let mut w = s.build();
    spawn(&mut w, Kind::Vehicle, "TANK", 1, 0, 0);
    victory::surrender(&mut w, C);
    assert!(!w.houses[2].econ.result.doomed);
    common::run(&mut w, 2);
    assert!(victory::ended(&w));
    victory::surrender(&mut w, B);
    assert!(!w.houses[1].econ.result.doomed);
}

fn allies_scene() -> Scene {
    let mut s = scene(false);
    s.ruleset.sim.general.economy.allies_allowed = Some(true);
    s
}

// spec: S-0414 claim 12 (peace: one way; refused at the last enemy, for B = A, a civilian B or a defeated A; targets on now-allied houses dropped)
#[test]
fn claim_12_making_peace() {
    let mut w = allies_scene().build();
    let [a, b, _] = armed(&mut w);
    w.obj_mut(a).unwrap().target = Some(Target::Object(b));
    victory::alliance(&mut w, A, B);
    assert!(w.allied(Some(A), Some(B)));
    assert!(!w.allied(Some(B), Some(A)), "one way");
    assert_eq!(w.obj(a).unwrap().target, None, "target dropped");
    // Gamma is Alpha's last enemy: refused, whoever is named.
    victory::alliance(&mut w, A, C);
    assert!(!w.allied(Some(A), Some(C)));
    // A defeated issuer cannot make peace.
    let mut w = allies_scene().build();
    armed(&mut w);
    w.houses[0].econ.result.defeated = true;
    victory::alliance(&mut w, A, B);
    assert!(!w.allied(Some(A), Some(B)));
    // A civilian-side house cannot be allied.
    let mut s = allies_scene();
    s.ruleset
        .sim
        .countries
        .get_mut("Betaland")
        .unwrap()
        .econ
        .side = Some("Civilian".into());
    let mut w = s.build();
    armed(&mut w);
    victory::alliance(&mut w, A, B);
    assert!(!w.allied(Some(A), Some(B)));
}

// spec: S-0414 claim 13 (breaking: both directions, also by a defeated house; a passive A or B, or naming oneself, changes nothing)
#[test]
fn claim_13_breaking() {
    let mut w = allies_scene().build();
    armed(&mut w);
    w.houses[0].allies = vec![A, B];
    w.houses[1].allies = vec![A, B];
    w.houses[0].econ.result.defeated = true;
    victory::alliance(&mut w, A, B);
    assert!(!w.allied(Some(A), Some(B)) && !w.allied(Some(B), Some(A)));
    victory::alliance(&mut w, A, A);
    assert!(w.allied(Some(A), Some(A)));
    let mut s = allies_scene();
    s.ruleset
        .sim
        .countries
        .get_mut("Betaland")
        .unwrap()
        .multiplay_passive = Some(true);
    let mut w = s.build();
    w.houses[0].allies = vec![A, B];
    w.houses[1].allies = vec![A, B];
    victory::alliance(&mut w, A, B);
    assert!(w.allied(Some(A), Some(B)), "passive B");
}

// spec: S-0414 claim 14 (both flags must be on; the start alliances ignore them)
#[test]
fn claim_14_flags() {
    for (allies, change) in [(false, true), (true, false)] {
        let mut s = scene(false);
        s.ruleset.sim.general.economy.allies_allowed = Some(allies);
        s.ruleset.sim.general.economy.ally_change_allowed = Some(change);
        let mut w = s.build();
        armed(&mut w);
        victory::alliance(&mut w, A, B);
        assert!(!w.allied(Some(A), Some(B)));
    }
    // Start alliances: the default flags (allies not allowed) do not stop them.
    let mut s = scene(false);
    s.setup.skirmish.houses = vec![Default::default(); 2];
    s.setup.skirmish.houses[0].team = Some(1);
    s.setup.skirmish.houses[1].team = Some(1);
    let w = s.build();
    assert!(w.allied(Some(A), Some(B)));
}

// spec: S-0414 claim 11 through the order (S-0213 table T1: the order applied after the house updates of its tick)
#[test]
fn claim_11_order() {
    let mut w = scene(false).build();
    armed(&mut w);
    common::open_orders(&mut w);
    glasnost_sim::orders::issue(&mut w, A, glasnost_sim::orders::Order::Surrender);
    let t = w.tick;
    common::run(&mut w, 1);
    assert!(w.houses[0].econ.result.doomed, "marked in tick {t}");
    assert!(!defeated(&w, A));
    common::run(&mut w, 1);
    assert!(defeated(&w, A));
}

// spec: S-0309 claim 16 (revision 4: made an ally, the anger towards the new ally becomes 0 and the enemy house is worked out again; made an enemy, the anger grows by 1 point, for each house whose alliance ended)
#[test]
fn s0309_claim_16_anger_on_alliance_changes() {
    let mut w = allies_scene().build();
    armed(&mut w);
    w.houses[0].combat.anger[1] = 7;
    glasnost_sim::combat::acquire::recompute_enemy(&mut w, 0);
    assert_eq!(w.houses[0].combat.enemy, Some(B));
    victory::alliance(&mut w, A, B);
    assert_eq!(w.houses[0].combat.anger[1], 0);
    assert_eq!(w.houses[0].combat.enemy, None, "the new ally was the enemy house");
    // Breaking it: Alpha is made an enemy of Beta; Beta, which never allied Alpha, is not.
    let before = w.houses[1].combat.anger[0];
    victory::alliance(&mut w, A, B);
    assert!(!w.allied(Some(A), Some(B)));
    assert_eq!(w.houses[0].combat.anger[1], 1);
    assert_eq!(w.houses[1].combat.anger[0], before);
    assert_eq!(w.houses[0].combat.enemy, Some(B));
}
