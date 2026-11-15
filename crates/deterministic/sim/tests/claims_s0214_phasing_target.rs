//! The target-state conditions of S-0214 values x and q: a transport that
//! is phasing in gives no transport verdict (#358; S-0214 table I value x,
//! table V value q, row G10). On synthetic types on an empty map (`HAULER`
//! an invented wheeled transport for four; `WALKER` and `BUGGY` from the
//! testkit's combat ruleset), no game data.

use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{Decimal, LocomotionClass, Locomotor, UnitType};
use glasnost_sim::rules::Kind;
use glasnost_sim::targeting::{self, Action, ClickOn, Modifiers};
use glasnost_sim::teleport::TeleportState;
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, World};

const ALPHA: HouseId = HouseId(0);
const SIZE: i32 = 24;

fn ruleset() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    r.registries
        .entry("vehicles".into())
        .or_default()
        .push("HAULER".into());
    r.sim.vehicles.insert(
        "HAULER".into(),
        UnitType {
            strength: Some(400),
            speed: Some(6),
            rot: Some(8),
            locomotion_class: Some(LocomotionClass::Wheel),
            locomotor: Some(Locomotor::Drive),
            passengers: Some(4),
            size_limit: Some(Decimal("2".into())),
            ..Default::default()
        },
    );
    r
}

fn at(w: &mut World, kind: Kind, name: &str, dx: i32, dy: i32) -> ObjectId {
    testkit::spawn(w, kind, name, 0, testkit::mid(SIZE, dx, dy), 0)
}

fn cursor(w: &World, sel: &[ObjectId], on: ObjectId) -> Action {
    targeting::cursor(w, ALPHA, sel, ClickOn::Object(on), Modifiers::default())
}

// spec: S-0214 table I value x and table V value q (defined only for a target not being erased and not phasing in; undefined, the action is g: select, row G10) and Terms (phasing in)
#[test]
fn issue_358_a_phasing_in_transport_gives_no_transport_verdict() {
    let r = ruleset();
    let mut w = testkit::game_with(&r, SIZE, 7);
    let t = at(&mut w, Kind::Vehicle, "HAULER", 0, 0);
    let p = at(&mut w, Kind::Infantry, "WALKER", 3, 0);
    let v = at(&mut w, Kind::Vehicle, "BUGGY", -4, 0);
    testkit::run(&mut w, 5);
    assert_eq!(cursor(&w, &[p], t), Action::Enter, "control: infantry");
    assert_eq!(cursor(&w, &[v], t), Action::Enter, "control: vehicle");
    // The transport phases in (S-0207 claim 20 f) for 50 ticks from now.
    let now = w.tick;
    w.obj_mut(t).unwrap().teleport = Some(TeleportState {
        phasing: Some((now, 50)),
        ..Default::default()
    });
    assert!(glasnost_sim::teleport::phasing_in(&w, t), "set-up");
    let click = targeting::click(&w, ALPHA, &[p], ClickOn::Object(t), Modifiers::default());
    assert_eq!(
        (cursor(&w, &[p], t), cursor(&w, &[v], t)),
        (Action::Select, Action::Select),
        "(table I value x, table V value q); infantry click orders: {:?}",
        click.orders
    );
}

// spec: S-0214 table I value e (defined only when the target is a building allied with the actor and not being erased; row e4: a hospital, no force-fire, the actor with all its hit points: no-enter) and Terms (being erased: a superweapon warp member in its leaving stage)
#[test]
fn issue_358_a_hospital_being_erased_gives_no_entry_verdict() {
    let mut r = testkit::combat_ruleset();
    r.registries
        .entry("buildings".into())
        .or_default()
        .push("CLINIC".into());
    r.sim.buildings.insert(
        "CLINIC".into(),
        glasnost_rules::sim::BuildingType {
            strength: Some(1000),
            foundation: Some([1, 1]),
            hospital: Some(true),
            ..Default::default()
        },
    );
    let mut w = testkit::game_with(&r, SIZE, 7);
    let h = at(&mut w, Kind::Building, "CLINIC", 0, 0);
    let p = at(&mut w, Kind::Infantry, "WALKER", 3, 0);
    testkit::run(&mut w, 5);
    assert_eq!(cursor(&w, &[p], h), Action::NoEnter, "control: row e4");
    let pos = w.obj(h).unwrap().pos;
    let now = w.tick;
    w.obj_mut(h).unwrap().sw.warp = Some(glasnost_sim::superweapons::warp::Member {
        stage: glasnost_sim::superweapons::warp::Stage::Leaving(now),
        house: ALPHA,
        landing: pos,
        redirected: false,
        fell_back: false,
        refused: false,
        entry_ok: false,
    });
    assert!(
        glasnost_sim::superweapons::being_erased(&w, h),
        "set-up: being erased"
    );
    assert_ne!(
        cursor(&w, &[p], h),
        Action::NoEnter,
        "value e is undefined for a hospital being erased"
    );
}
