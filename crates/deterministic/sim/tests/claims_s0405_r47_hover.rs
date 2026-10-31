//! Claim tests of S-0405 revision 47, claim 13's *Hovering vehicles*
//! paragraph (Q-1710) and claim 20's *As a target*, through the hovering
//! locomotor of S-0503 on synthetic rules: the testkit's combat ruleset
//! plus the hover fields of [General] and a hovering vehicle type with the
//! powered-unit flag (every name and number invented, no game data). The
//! house never has an active control centre.

use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{
    BuildingType, Decimal, LandRow, LandType, LocomotionClass, Locomotor, MovementZone, UnitType,
};
use glasnost_sim::econ::power;
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, Target, World};

const SIZE: i32 = 24;

fn dec(s: &str) -> Option<Decimal> {
    Some(Decimal(s.into()))
}

fn land_row() -> LandRow {
    LandRow {
        foot: dec("1"),
        track: dec("1"),
        wheel: dec("1"),
        hover: dec("1"),
        float: dec("0"),
        amphibious: dec("1"),
        float_beach: dec("0"),
        buildable: None,
    }
}

/// HOVERBOT: a hovering vehicle (hover locomotor and class, amphibious
/// zone) with the powered-unit flag; CENTRE: its control centre, never
/// placed.
fn rules() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    let s = &mut r.sim;
    s.land_types.insert(LandType::Clear, land_row());
    s.land_types.insert(LandType::Road, land_row());
    s.general.hover_height = Some(100);
    s.general.hover_bob = dec("0");
    s.general.hover_boost = dec("1.5");
    s.general.hover_acceleration = dec("0.02");
    s.general.hover_brake = dec("0.01");
    s.general.hover_dampen = dec("0.75");
    let mut bot = UnitType {
        strength: Some(150),
        speed: Some(6),
        rot: Some(5),
        locomotion_class: Some(LocomotionClass::Hover),
        locomotor: Some(Locomotor::Hover),
        movement_zone: Some(MovementZone::Amphibious),
        ..Default::default()
    };
    bot.econ.powered_unit = Some(true);
    s.vehicles.insert("HOVERBOT".into(), bot);
    let mut centre = BuildingType {
        strength: Some(500),
        foundation: Some([1, 1]),
        powered: Some(true),
        ..Default::default()
    };
    centre.econ.powers_unit = Some("HOVERBOT".into());
    s.buildings.insert("CENTRE".into(), centre);
    r.registries
        .entry("vehicles".into())
        .or_default()
        .push("HOVERBOT".into());
    r.registries
        .entry("buildings".into())
        .or_default()
        .push("CENTRE".into());
    r
}

fn game() -> World {
    let mut w = testkit::game_with(&rules(), SIZE, 3);
    while orders::blocked(&w) {
        glasnost_sim::tick::step(&mut w);
    }
    w
}

fn put(w: &mut World, name: &str, house: u8, dx: i32, dy: i32) -> ObjectId {
    testkit::spawn(w, Kind::Vehicle, name, house, testkit::mid(SIZE, dx, dy), 0)
}

fn move_to(w: &mut World, id: ObjectId, c: CellXY) {
    assert!(orders::issue(
        w,
        HouseId(0),
        Order::Behaviour {
            actor: id,
            behaviour: Behaviour::Move,
            attack_move: false,
            target: None,
            dest: Some(Target::Cell(c)),
        },
    ));
}

fn cell_of(w: &World, id: ObjectId) -> CellXY {
    w.obj(id).unwrap().pos.cell()
}

// spec: S-0405 claim 13 (revision 47, Q-1710: a placed hovering vehicle with the flag, on a cell without a building, with no active control centre, is disabled on the tick of its first non-refusing heading-point choice, still on its placement cell; one never given a destination makes no choice and stays enabled)
#[test]
fn claim_13_a_placed_hover_is_disabled_on_its_placement_cell() {
    let mut w = game();
    let bot = put(&mut w, "HOVERBOT", 0, 0, 0);
    let home = cell_of(&w, bot);
    // Never given a destination: enabled.
    let mut idle = w.clone();
    testkit::run(&mut idle, 60);
    assert!(
        !power::unit_disabled(&idle, bot),
        "no choice, still enabled"
    );
    assert_eq!(cell_of(&idle, bot), home);
    // Given one: disabled on its first choice, before it leaves its cell.
    move_to(&mut w, bot, testkit::mid(SIZE, 5, 0));
    let mut ticks = 0;
    while !power::unit_disabled(&w, bot) {
        assert_eq!(cell_of(&w, bot), home, "not yet disabled, still home");
        assert!(ticks < 30, "disabled within 30 ticks");
        glasnost_sim::tick::step(&mut w);
        ticks += 1;
    }
    assert_eq!(cell_of(&w, bot), home, "disabled on its placement cell");
    assert_eq!(w.obj(bot).unwrap().dest, None, "the guard reset clears it");
    // It does not leave it afterwards (no destination, locomotor off).
    testkit::run(&mut w, 60);
    assert!(power::unit_disabled(&w, bot));
    assert_eq!(cell_of(&w, bot), home, "never left its placement cell");
}

// spec: S-0405 claim 20 (As a target: a disabled vehicle is targeted, attacked, damaged and destroyed like any other; no targeting or damage rule reads the disabled state; M-0005 measures the targeting part)
#[test]
fn claim_20_a_disabled_hover_is_targeted_damaged_and_destroyed() {
    let mut w = game();
    let bot = put(&mut w, "HOVERBOT", 0, 0, 0);
    move_to(&mut w, bot, testkit::mid(SIZE, 5, 0));
    for _ in 0..30 {
        if power::unit_disabled(&w, bot) {
            break;
        }
        glasnost_sim::tick::step(&mut w);
    }
    assert!(power::unit_disabled(&w, bot));
    let full = w.obj(bot).unwrap().health;
    let cart = put(&mut w, "CART", 1, -4, 0);
    let mut targeted = false;
    let mut damaged = false;
    for _ in 0..2000 {
        glasnost_sim::tick::step(&mut w);
        let Some(o) = w.obj(bot) else { break };
        targeted |= w.obj(cart).unwrap().target == Some(Target::Object(bot));
        damaged |= o.health < full;
        if glasnost_sim::combat::damage::dead(o) {
            break;
        }
    }
    assert!(targeted, "the hunter picked the disabled vehicle");
    assert!(damaged, "it took damage");
    assert!(
        w.obj(bot).is_none_or(glasnost_sim::combat::damage::dead),
        "destroyed"
    );
}
