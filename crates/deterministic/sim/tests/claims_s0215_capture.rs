//! Claim tests for S-0215 claim 28 (*capture* and *sabotage*, and the
//! *eaten* differences) and claim 5 (*sabotage* is processed as
//! *capture*), with table T1's kinds. Synthetic data only: the combat
//! ruleset of `testkit` with invented infantry types (an engineer, a charge
//! planter, an occupier, an assaulter and an infiltrator, each a copy of
//! WALKER), a plain 2 × 2 building HUT, an unarmed flyer FLYER and
//! invented behaviour rates; no stock value.

use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{BehaviourProps, BuildingType, Decimal, Locomotor, UnitType};
use glasnost_sim::process;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit;
use glasnost_sim::tick;
use glasnost_sim::world::{ObjectId, Target, World};

const SIZE: i32 = 20;

fn register(r: &mut Ruleset, reg: &str, name: &str) {
    let l = r.registries.entry(reg.into()).or_default();
    if !l.iter().any(|n| n == name) {
        l.push(name.into());
    }
}

fn infantry(r: &mut Ruleset, name: &str, f: impl FnOnce(&mut UnitType)) {
    let mut t = r.sim.infantry["WALKER"].clone();
    f(&mut t);
    register(r, "infantry", name);
    r.sim.infantry.insert(name.into(), t);
}

/// The combat ruleset with ENG (engineer), BOOM (explosive charges), OCC
/// (occupier), ASL (assaulter), SPYI (infiltrator), the plain building HUT,
/// the unarmed landable flyer FLYER, and invented rates for *capture*
/// (0.05 game minutes) and *sabotage* (0.1), so that their D_r differ.
fn rules() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    infantry(&mut r, "ENG", |t| t.engineer = Some(true));
    infantry(&mut r, "BOOM", |t| t.c4 = Some(true));
    infantry(&mut r, "OCC", |t| t.occupier = Some(true));
    infantry(&mut r, "ASL", |t| t.assaulter = Some(true));
    infantry(&mut r, "SPYI", |t| t.infiltrate = Some(true));
    register(&mut r, "buildings", "HUT");
    r.sim.buildings.insert(
        "HUT".into(),
        BuildingType {
            strength: Some(500),
            foundation: Some([2, 2]),
            ..Default::default()
        },
    );
    let mut fly = UnitType {
        strength: Some(150),
        speed: Some(30),
        rot: Some(12),
        locomotor: Some(Locomotor::Fly),
        ..Default::default()
    };
    fly.econ.landable = Some(true);
    register(&mut r, "aircraft", "FLYER");
    r.sim.aircraft.insert("FLYER".into(), fly);
    for (b, rate) in [(Behaviour::Capture, "0.05"), (Behaviour::Sabotage, "0.1")] {
        r.sim.behaviours.insert(
            b.key().into(),
            BehaviourProps {
                rate: Some(Decimal(rate.into())),
                ..Default::default()
            },
        );
    }
    r
}

fn game() -> World {
    let mut w = testkit::game_with_non_players(&rules(), SIZE, 3);
    testkit::run(&mut w, 4);
    w
}

fn put(w: &mut World, kind: Kind, name: &str, house: u8, dx: i32, dy: i32) -> ObjectId {
    testkit::spawn(w, kind, name, house, testkit::mid(SIZE, dx, dy), 0)
}

/// Makes `b` current with `target` and `dest`, nothing queued, and the
/// processing due now.
fn set(w: &mut World, id: ObjectId, b: Behaviour, target: Option<Target>, dest: Option<Target>) {
    let o = w.obj_mut(id).unwrap();
    o.beh.current = Some(b);
    o.beh.queued = None;
    o.beh.last = None;
    o.beh.expired = true;
    o.target = target;
    o.dest = dest;
}

/// One tick; the actor's processing comes in it. Returns the delay it chose.
fn processed(w: &mut World, id: ObjectId) -> i32 {
    tick::step(w);
    let (at, d) = w.obj(id).unwrap().beh.last.expect("processed");
    assert_eq!(at, w.tick - 1, "processed in that tick");
    d
}

fn rested(w: &World, id: ObjectId, b: Behaviour) -> bool {
    let o = w.obj(id).unwrap();
    o.beh.current == Some(b) || o.beh.queued == Some(b)
}

fn no_unbuilt_note(w: &World) {
    assert!(
        !w.notes.iter().any(|n| n.contains("is not implemented yet")),
        "no processing falls through to the unbuilt note: {:?}",
        w.notes
    );
}

fn d_r(w: &World, b: Behaviour) -> i32 {
    w.rules().behaviour(b).rate_delay
}

// spec: S-0215 claim 28 row 1 (no destination; the target a building and the actor a building taker: the target becomes its destination) and the building-taker list (charges, engineer, occupier, assaulter, infiltrator)
#[test]
fn claim_28_row_1_a_building_taker_takes_its_target_building_as_destination() {
    for name in ["ENG", "BOOM", "OCC", "ASL", "SPYI"] {
        let mut w = game();
        let hut = put(&mut w, Kind::Building, "HUT", 1, 6, 0);
        let u = put(&mut w, Kind::Infantry, name, 0, -4, 0);
        set(
            &mut w,
            u,
            Behaviour::Capture,
            Some(Target::Object(hut)),
            None,
        );
        processed(&mut w, u);
        let o = w.obj(u).unwrap();
        assert_eq!(o.dest, Some(Target::Object(hut)), "{name}: row 1");
        assert_eq!(o.beh.current, Some(Behaviour::Capture), "{name}");
        no_unbuilt_note(&w);
    }
}

// spec: S-0215 claim 28 row 2 (a target building but not a building taker: it falls back to rest) and claim 14 row 2 (with a target, an infantry unit in capture queues capture again)
#[test]
fn claim_28_row_2_a_non_taker_with_a_target_building_falls_back() {
    let mut w = game();
    let hut = put(&mut w, Kind::Building, "HUT", 1, 6, 0);
    let u = put(&mut w, Kind::Infantry, "WALKER", 0, -4, 0);
    set(
        &mut w,
        u,
        Behaviour::Capture,
        Some(Target::Object(hut)),
        None,
    );
    processed(&mut w, u);
    let o = w.obj(u).unwrap();
    assert_eq!(o.dest, None, "the target does not become its destination");
    assert_eq!(o.target, Some(Target::Object(hut)));
    assert!(
        rested(&w, u, Behaviour::Capture),
        "capture again (claim 14 row 2)"
    );
}

// spec: S-0215 claim 28 row 2 (no destination and no target: it falls back to rest, claim 14) and scenario B12 (e), the ordinary cell (it stays on its cell)
#[test]
fn claim_28_row_2_infantry_with_nothing_falls_back_and_stays() {
    let mut w = game();
    let u = put(&mut w, Kind::Infantry, "WALKER", 0, 0, 0);
    let cell = w.obj(u).unwrap().pos.cell();
    set(&mut w, u, Behaviour::Capture, None, None);
    processed(&mut w, u);
    // spec: S-0215 claim 14 row 12 (a player house's infantry without the area-guard ability or default: guard)
    assert!(rested(&w, u, Behaviour::Guard), "guard");
    assert_eq!(
        w.obj(u).unwrap().dest,
        None,
        "no scatter off an ordinary cell"
    );
    testkit::run(&mut w, 30);
    assert_eq!(w.obj(u).unwrap().pos.cell(), cell, "it stays on its cell");
    no_unbuilt_note(&w);
}

// spec: S-0215 claim 28 (the scatter off a building: no threat point, forced, not move-anywhere; S-0208 claims 11 and 13 for infantry) and scenario B12 (e), the building's cell (it walks to a neighbouring cell)
#[test]
fn claim_28_scatter_off_a_building_infantry() {
    let mut w = game();
    let hut = put(&mut w, Kind::Building, "HUT", 0, 0, 0);
    let u = put(&mut w, Kind::Infantry, "WALKER", 0, 0, 0);
    let cell = w.obj(u).unwrap().pos.cell();
    assert!(
        w.cells
            .list(cell, glasnost_sim::world::Layer::Ground)
            .contains(&hut)
    );
    set(&mut w, u, Behaviour::Capture, None, None);
    processed(&mut w, u);
    let Some(Target::Cell(c)) = w.obj(u).unwrap().dest else {
        panic!("a scatter destination: {:?}", w.obj(u).unwrap().dest);
    };
    assert_ne!(c, cell);
    assert!(
        (c.x - cell.x).abs() <= 1 && (c.y - cell.y).abs() <= 1,
        "a neighbouring cell (S-0208 claim 13, no threat point)"
    );
    no_unbuilt_note(&w);
}

// spec: S-0215 claim 28 row 2 and claim 15 row 7 (an unarmed vehicle of a human-controlled owner in capture with nothing queues guard), scenario B29 (c); and the eaten difference (claim 15: nothing is queued while eaten is current)
#[test]
fn claim_28_row_2_an_unarmed_vehicle_queues_guard_and_eaten_does_not() {
    let mut w = game();
    let cap = put(&mut w, Kind::Vehicle, "BUGGY", 0, -4, 0);
    let eat = put(&mut w, Kind::Vehicle, "BUGGY", 0, 4, 0);
    set(&mut w, cap, Behaviour::Capture, None, None);
    set(&mut w, eat, Behaviour::Eaten, None, None);
    tick::step(&mut w);
    assert!(rested(&w, cap, Behaviour::Guard), "capture: guard (row 7)");
    for _ in 0..10 {
        let o = w.obj_mut(eat).unwrap();
        o.beh.last = None;
        o.beh.expired = true;
        tick::step(&mut w);
        let o = w.obj(eat).unwrap();
        assert_eq!(o.beh.current, Some(Behaviour::Eaten), "eaten stays current");
        assert_eq!(o.beh.queued, None, "nothing queued");
    }
    no_unbuilt_note(&w);
}

// spec: S-0215 claim 28 (a vehicle judges the forced scatter off a building by S-0207 claim 16, the nearby-cell search without a draw; claim 19: the scatter property does not decide) and T2 (the standard delay's one draw)
#[test]
fn claim_28_scatter_off_a_building_vehicle_takes_no_draw() {
    let mut w = game();
    let hut = put(&mut w, Kind::Building, "HUT", 0, 0, 0);
    let v = put(&mut w, Kind::Vehicle, "BUGGY", 0, 0, 0);
    let cell = w.obj(v).unwrap().pos.cell();
    assert!(
        w.cells
            .list(cell, glasnost_sim::world::Layer::Ground)
            .contains(&hut)
    );
    set(&mut w, v, Behaviour::Capture, None, None);
    let before = w.rng.taken();
    process::tail(&mut w, v);
    assert_eq!(w.rng.taken() - before, 1, "only the delay's draw");
    let o = w.obj(v).unwrap();
    let Some(Target::Cell(c)) = o.dest else {
        panic!("a nearby-cell destination: {:?}", o.dest);
    };
    assert_ne!(c, cell);
    assert!(
        !w.cells
            .list(c, glasnost_sim::world::Layer::Ground)
            .contains(&hut),
        "off the building"
    );
    no_unbuilt_note(&w);
}

// spec: S-0215 claim 28 rows 3 and 4 (a destination: no change with a target; without one the hunt rule, which never applies in our engine), for a player house and for the neutral house (computer-controlled)
#[test]
fn claim_28_rows_3_and_4_a_destination_changes_nothing() {
    for house in [0u8, 2] {
        let mut w = game();
        let hut = put(&mut w, Kind::Building, "HUT", 1, 6, 0);
        let row3 = put(&mut w, Kind::Infantry, "ENG", house, -4, -2);
        let row4 = put(&mut w, Kind::Infantry, "ENG", house, -4, 2);
        let tv = put(&mut w, Kind::Vehicle, "BUGGY", house, -4, 5);
        let h = Some(Target::Object(hut));
        set(&mut w, row3, Behaviour::Capture, h, h);
        set(&mut w, row4, Behaviour::Capture, None, h);
        set(&mut w, tv, Behaviour::Capture, None, h);
        tick::step(&mut w);
        for (u, t) in [(row3, h), (row4, None), (tv, None)] {
            let o = w.obj(u).unwrap();
            assert!(o.beh.last.is_some(), "processed");
            assert_eq!(o.beh.current, Some(Behaviour::Capture), "house {house}");
            assert_eq!(o.beh.queued, None, "house {house}: nothing queued, no hunt");
            assert_eq!(o.dest, h, "house {house}: the destination stays");
            assert_eq!(o.target, t, "house {house}: the target stays");
        }
        no_unbuilt_note(&w);
    }
}

// spec: S-0215 claim 28 (hunt rule, original only: "In our engine the hunt rule never applies"); scenario B12 (a) records the original's hunt, which our engine does not take
#[test]
fn claim_28_a_computer_engineer_without_a_target_does_not_hunt() {
    let mut w = game();
    let u = put(&mut w, Kind::Infantry, "ENG", 2, 0, 0);
    set(&mut w, u, Behaviour::Capture, None, None);
    processed(&mut w, u);
    let o = w.obj(u).unwrap();
    assert_ne!(o.beh.queued, Some(Behaviour::Hunt));
    assert_ne!(o.beh.current, Some(Behaviour::Hunt));
    // spec: S-0215 claim 14 row 9 (not human-controlled, an engineer: area guard)
    assert!(rested(&w, u, Behaviour::AreaGuard), "area guard");
}

// spec: S-0215 claim 5 (sabotage is processed exactly as capture, with the rate of sabotage) and claim 28 row 1; T2 (capture, sabotage, eaten: always the standard delay D_r + U(0, 2))
#[test]
fn claim_5_sabotage_is_processed_as_capture_with_its_own_rate() {
    let mut w = game();
    let hut = put(&mut w, Kind::Building, "HUT", 1, 6, 0);
    let s = put(&mut w, Kind::Infantry, "BOOM", 0, -4, -2);
    let c = put(&mut w, Kind::Infantry, "BOOM", 0, -4, 2);
    let h = Some(Target::Object(hut));
    set(&mut w, s, Behaviour::Sabotage, h, None);
    set(&mut w, c, Behaviour::Capture, h, None);
    tick::step(&mut w);
    let (ds, dc) = (
        w.obj(s).unwrap().beh.last.unwrap().1,
        w.obj(c).unwrap().beh.last.unwrap().1,
    );
    for (u, b, d) in [(s, Behaviour::Sabotage, ds), (c, Behaviour::Capture, dc)] {
        let o = w.obj(u).unwrap();
        assert_eq!(o.dest, h, "{b:?}: row 1");
        assert_eq!(o.beh.current, Some(b));
        let dr = d_r(&w, b);
        assert!(
            (dr..=dr + 2).contains(&d),
            "{b:?}: {d} in D_r {dr} + U(0, 2)"
        );
        assert_ne!(d, 450);
    }
    assert_ne!(d_r(&w, Behaviour::Sabotage), d_r(&w, Behaviour::Capture));
    no_unbuilt_note(&w);
}

// spec: S-0215 claim 28 and T2 (the delay is always the standard delay, every row): many processings stay within D_r … D_r + 2
#[test]
fn claim_28_the_delay_is_always_the_standard_delay() {
    let mut w = game();
    let hut = put(&mut w, Kind::Building, "HUT", 1, 6, 0);
    let u = put(&mut w, Kind::Infantry, "ENG", 0, -6, 0);
    let dr = d_r(&w, Behaviour::Capture);
    let mut seen = std::collections::BTreeSet::new();
    for i in 0..30 {
        let (t, d) = match i % 3 {
            0 => (Some(Target::Object(hut)), None),
            1 => (None, Some(Target::Object(hut))),
            _ => (Some(Target::Object(hut)), Some(Target::Object(hut))),
        };
        set(&mut w, u, Behaviour::Capture, t, d);
        let delay = processed(&mut w, u);
        assert!((dr..=dr + 2).contains(&delay), "{delay}");
        seen.insert(delay);
    }
    assert!(seen.len() > 1, "a drawn delay");
}

// spec: S-0215 claim 28 (for eaten, occupiers, assaulters and infiltrators do not count as building takers; charge planters and engineers do)
#[test]
fn claim_28_eaten_counts_fewer_building_takers() {
    for (name, taker) in [
        ("OCC", false),
        ("ASL", false),
        ("SPYI", false),
        ("ENG", true),
        ("BOOM", true),
    ] {
        let mut w = game();
        let hut = put(&mut w, Kind::Building, "HUT", 1, 6, 0);
        let e = put(&mut w, Kind::Infantry, name, 0, -4, -2);
        let c = put(&mut w, Kind::Infantry, name, 0, -4, 2);
        let h = Some(Target::Object(hut));
        set(&mut w, e, Behaviour::Eaten, h, None);
        set(&mut w, c, Behaviour::Capture, h, None);
        tick::step(&mut w);
        assert_eq!(
            w.obj(e).unwrap().dest.is_some(),
            taker,
            "{name} in eaten: a building taker only as charge planter or engineer"
        );
        assert_eq!(w.obj(c).unwrap().dest, h, "{name} in capture: row 1");
    }
}

// spec: S-0215 table T1 and claim 28 (capture has its own processing for aircraft: row 2's fall-back is S-0501 claim 18, on the ground guard) and T2 (the standard delay, not 450); Q-2251 (interim: the scatter off a building has no effect on an aircraft)
#[test]
fn claim_28_an_aircraft_in_capture_falls_back_with_the_standard_delay() {
    let mut w = game();
    let a = put(&mut w, Kind::Aircraft, "FLYER", 0, -4, 0);
    assert!(glasnost_sim::air::on_ground(&w, a));
    set(&mut w, a, Behaviour::Capture, None, None);
    let d = processed(&mut w, a);
    // spec: S-0501 claim 18 (on the ground it forgets its target and destination and queues guard)
    assert!(rested(&w, a, Behaviour::Guard), "guard");
    let dr_now = d_r(&w, w.obj(a).unwrap().beh.current.unwrap());
    let dr_cap = d_r(&w, Behaviour::Capture);
    assert!(
        (dr_cap..=dr_cap + 2).contains(&d) || (dr_now..=dr_now + 2).contains(&d),
        "the standard delay: {d}"
    );
    assert_ne!(d, 450);
    // On a building's cell the scatter request goes out and, by Q-2251's interim, does nothing.
    let hut = put(&mut w, Kind::Building, "HUT", 0, 4, 0);
    let b = put(&mut w, Kind::Aircraft, "FLYER", 0, 4, 0);
    let cell = w.obj(b).unwrap().pos.cell();
    assert!(
        w.cells
            .list(cell, glasnost_sim::world::Layer::Ground)
            .contains(&hut)
    );
    set(&mut w, b, Behaviour::Sabotage, None, None);
    processed(&mut w, b);
    assert_eq!(
        w.obj(b).unwrap().dest,
        None,
        "Q-2251 interim: no destination"
    );
    assert_eq!(w.obj(b).unwrap().pos.cell(), cell);
    no_unbuilt_note(&w);
}

// spec: S-0215 claim 4 and table T1 (buildings have no processing rule for capture, sabotage or eaten: nothing, 450 ticks, no draw)
#[test]
fn claim_4_a_building_in_capture_has_no_processing_rule() {
    for b in [Behaviour::Capture, Behaviour::Sabotage, Behaviour::Eaten] {
        let mut w = game();
        let hut = put(&mut w, Kind::Building, "HUT", 0, 0, 0);
        testkit::run(&mut w, 3);
        set(&mut w, hut, b, None, None);
        let before = w.rng.taken();
        process::tail(&mut w, hut);
        assert_eq!(w.obj(hut).unwrap().beh.last.unwrap().1, 450, "{b:?}");
        assert_eq!(w.rng.taken(), before, "no draw");
        no_unbuilt_note(&w);
    }
}
