//! With `quirks.update_order_removal_skip` off no object takes more than
//! one turn per tick, also one that boards an open-topped transport after
//! its own turn (#365; S-0203 claims 3, 4 and 11, Edge cases). On the
//! synthetic combat ruleset of the testkit plus an invented open-topped
//! transport, no game data.

use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{Decimal, LocomotionClass, Locomotor, UnitType};
use glasnost_sim::probe::{Phase, Probe, TurnClass};
use glasnost_sim::rules::Kind;
use glasnost_sim::targeting::{self, ClickOn, Modifiers};
use glasnost_sim::world::HouseId;
use glasnost_sim::{orders, testkit, tick, transport};

const SIZE: i32 = 24;

struct InfantryTurns(u32);

impl Probe for InfantryTurns {
    fn mark(&mut self, phase: Phase) {
        if phase == Phase::Turn(TurnClass::Infantry) {
            self.0 += 1;
        }
    }
}

fn ruleset(skip: bool) -> Ruleset {
    let mut r = testkit::combat_ruleset();
    r.registries
        .entry("vehicles".into())
        .or_default()
        .push("OPEN".into());
    r.sim.vehicles.insert(
        "OPEN".into(),
        UnitType {
            strength: Some(400),
            speed: Some(6),
            rot: Some(8),
            locomotion_class: Some(LocomotionClass::Wheel),
            locomotor: Some(Locomotor::Drive),
            passengers: Some(4),
            size_limit: Some(Decimal("2".into())),
            open_topped: Some(true),
            ..Default::default()
        },
    );
    r.globals.quirks.update_order_removal_skip = skip;
    r
}

/// The infantry turns in the tick in which the only infantry unit boards;
/// `bystander`: a vehicle after the passenger in the update order.
fn infantry_turns_in_the_boarding_tick(skip: bool, bystander: bool) -> u32 {
    let r = ruleset(skip);
    let mut w = testkit::game_with(&r, SIZE, 7);
    let t = testkit::spawn(&mut w, Kind::Vehicle, "OPEN", 0, testkit::mid(SIZE, 0, 0), 0);
    let p = testkit::spawn(&mut w, Kind::Infantry, "WALKER", 0, testkit::mid(SIZE, 3, 0), 0);
    if bystander {
        // With the switch on it is the one that loses its turn to the passenger's removal (S-0203 claim 10).
        testkit::spawn(&mut w, Kind::Vehicle, "BUGGY", 0, testkit::mid(SIZE, -8, -8), 0);
    }
    testkit::run(&mut w, 5);
    let c = targeting::click(&w, HouseId(0), &[p], ClickOn::Object(t), Modifiers::default());
    for o in c.orders {
        orders::issue(&mut w, HouseId(0), o);
    }
    for _ in 0..600 {
        let mut probe = InfantryTurns(0);
        tick::step_with(&mut w, &mut probe);
        if transport::riding(&w, p) == Some(t) {
            assert!(w.update.contains(p), "set-up: back in the update order");
            return probe.0;
        }
        assert_eq!(probe.0, 1, "set-up: one turn per tick before boarding");
    }
    panic!("never boarded");
}

// spec: S-0203 claims 3 and 11 (switch on: the unit that boards an open-topped transport after its own turn takes a second turn)
#[test]
fn issue_365_control_switch_on_second_turn() {
    assert_eq!(infantry_turns_in_the_boarding_tick(true, true), 2);
}

// spec: S-0203 claim 11 and Edge cases (switch off: at most one turn per object per tick, also for a unit that boards an open-topped transport after its own turn)
#[test]
fn issue_365_switch_off_one_turn() {
    assert_eq!(
        (
            infantry_turns_in_the_boarding_tick(false, true),
            infantry_turns_in_the_boarding_tick(false, false)
        ),
        (1, 1),
        "(with a bystander after the passenger, with the passenger last)"
    );
}
