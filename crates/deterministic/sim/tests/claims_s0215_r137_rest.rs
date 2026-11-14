//! Claim tests for S-0215 r137 claim 15 row 6, sim issues S11 (#236): an
//! unarmed vehicle that is deploying, or that has *unload* current and a
//! deploys-into type, changes nothing when it falls back to rest. State set
//! by hand on synthetic data (the testkit's combat ruleset with invented
//! types); every name and number is invented. Ledger:
//! `docs/m8-claims/sim-issues.md`, *Batch S11*.

use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::BuildingType;
use glasnost_sim::econ::inplace::Form;
use glasnost_sim::process;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit;
use glasnost_sim::world::{ObjectId, Target, World};

const SIZE: i32 = 20;

fn register(r: &mut Ruleset, reg: &str, name: &str) {
    r.registries
        .entry(reg.into())
        .or_default()
        .push(name.into());
}

/// HUT (a building), DEPV (an unarmed vehicle that deploys into HUT) and
/// INPL (an unarmed vehicle that deploys in place), from the unarmed BUGGY.
fn rules() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    register(&mut r, "buildings", "HUT");
    r.sim.buildings.insert(
        "HUT".into(),
        BuildingType {
            strength: Some(500),
            foundation: Some([2, 2]),
            ..Default::default()
        },
    );
    let mut depv = r.sim.vehicles["BUGGY"].clone();
    depv.deploys_into = Some("HUT".into());
    register(&mut r, "vehicles", "DEPV");
    r.sim.vehicles.insert("DEPV".into(), depv);
    let mut inpl = r.sim.vehicles["BUGGY"].clone();
    inpl.is_simple_deployer = Some(true);
    register(&mut r, "vehicles", "INPL");
    r.sim.vehicles.insert("INPL".into(), inpl);
    r
}

fn game() -> World {
    let mut w = testkit::game_with(&rules(), SIZE, 3);
    testkit::run(&mut w, 4);
    w
}

fn put(w: &mut World, name: &str, house: u8, dx: i32, dy: i32) -> ObjectId {
    testkit::spawn(w, Kind::Vehicle, name, house, testkit::mid(SIZE, dx, dy), 0)
}

/// Gives `v` the current behaviour `b`, nothing queued, `t` as target and
/// no destination, and makes it fall back to rest.
fn fall_back(w: &mut World, v: ObjectId, b: Behaviour, t: ObjectId) {
    let o = w.obj_mut(v).unwrap();
    o.beh.current = Some(b);
    o.beh.queued = None;
    o.target = Some(Target::Object(t));
    o.dest = None;
    process::fall_back(w, v, false);
}

// spec: S-0215 claim 15 row 6 (unarmed with unload current and a deploys-into type: nothing changes)
#[test]
fn issue_236_row_6_unload_current_and_a_deploys_into_type_changes_nothing() {
    let mut w = game();
    let enemy = put(&mut w, "BUGGY", 1, 6, 0);
    let v = put(&mut w, "DEPV", 0, -4, 0);
    fall_back(&mut w, v, Behaviour::Unload, enemy);
    let o = w.obj(v).unwrap();
    assert_eq!(o.target, Some(Target::Object(enemy)), "the target is kept");
    // Control: a type that does not deploy into a building takes row 7.
    let plain = put(&mut w, "BUGGY", 0, -4, 3);
    fall_back(&mut w, plain, Behaviour::Unload, enemy);
    assert_eq!(
        w.obj(plain).unwrap().target,
        None,
        "row 7 clears the target"
    );
}

// spec: S-0215 claim 15 row 6 (unarmed and deploying: nothing changes, whatever is current); S-0517 Terms (deploying: the in-place form in transition toward deployed)
#[test]
fn issue_236_row_6_unarmed_and_deploying_changes_nothing() {
    let mut w = game();
    let enemy = put(&mut w, "BUGGY", 1, 6, 0);
    let v = put(&mut w, "INPL", 0, -4, 0);
    let since = w.tick;
    w.obj_mut(v).unwrap().econ.deploy.form = Form::Deploying { since };
    fall_back(&mut w, v, Behaviour::Move, enemy);
    let o = w.obj(v).unwrap();
    assert_eq!(
        (o.target, o.beh.queued),
        (Some(Target::Object(enemy)), None),
        "the target is kept and nothing is queued"
    );
}
