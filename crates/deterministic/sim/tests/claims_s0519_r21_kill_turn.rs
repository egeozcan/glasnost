//! S-0519 revision 21 claim 4 (Q-3159, #384): a carrier that its own timed
//! bomb kills and that stays in play because it starts to sink or to crash
//! does not end its turn at the detonation. A sinking vessel takes its first
//! descent in that turn (a), an aircraft in ordinary flying movement the
//! first step of its crash fall (b), and a jumpjet vehicle its first crash
//! update, in our engine whether or not it is moving (c). Synthetic rules
//! (`tests/econ_common`, `tests/jump_common`); every type and number is
//! invented.

mod econ_common;
mod jump_common;

use glasnost_map::CellXY;
use glasnost_map::terrain::land;
use glasnost_num::F64;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{
    Decimal, LocomotionClass, Locomotor, MovementZone, UnitType, WarheadType,
};
use glasnost_sim::air;
use glasnost_sim::bomb::Bomb;
use glasnost_sim::combat::damage;
use glasnost_sim::jump::{self, Phase};
use glasnost_sim::naval;
use glasnost_sim::orders::Order;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit;
use glasnost_sim::world::{ObjectId, Target, World};

/// The bomb: warhead KILLWH (every armour at 100 %), damage 1000, timed
/// delay 50.
fn bomb_rules(r: &mut Ruleset) {
    r.sim.warheads.insert(
        "KILLWH".into(),
        WarheadType {
            verses: Some(vec![Decimal("1".into()); 11]),
            ..Default::default()
        },
    );
    let g = &mut r.sim.general;
    g.ivan_warhead = Some("KILLWH".into());
    g.ivan_damage = Some(1000);
    g.ivan_timed_delay = Some(50);
}

/// A bomb with no planter that goes off in the carrier's turn of the next
/// step.
fn arm(w: &mut World, id: ObjectId) {
    let planted = w.tick - 51;
    w.obj_mut(id).unwrap().combat.bomb = Some(Bomb {
        planter: None,
        house: None,
        planted,
    });
}

fn killed(w: &World, id: ObjectId) -> bool {
    w.obj(id).is_some_and(damage::dead)
}

// ---------------------------------------------------------------------------
// (a) A sinking vessel

const SEA: i32 = 24;

fn sea(dx: i32, dy: i32) -> CellXY {
    testkit::mid(SEA, dx, dy)
}

/// SKIFF: a naval ship heavier than the sinking weight 3, strength 50.
fn ship_world() -> (World, ObjectId) {
    let mut r = testkit::combat_ruleset();
    bomb_rules(&mut r);
    let mut skiff = UnitType {
        strength: Some(50),
        speed: Some(5),
        rot: Some(5),
        locomotion_class: Some(LocomotionClass::Float),
        locomotor: Some(Locomotor::Ship),
        movement_zone: Some(MovementZone::Water),
        weight: Some(Decimal("4".into())),
        ..Default::default()
    };
    skiff.combat.naval = Some(true);
    r.registries
        .entry("vehicles".into())
        .or_default()
        .push("SKIFF".into());
    r.sim.vehicles.insert("SKIFF".into(), skiff);
    r.sim.general.ship_sinking_weight = Some(Decimal("3".into()));
    let mut w = testkit::game_with(&r, SEA, 11);
    let s = testkit::spawn(&mut w, Kind::Vehicle, "SKIFF", 1, sea(0, 0), 0);
    water(&mut w);
    testkit::run(&mut w, 5);
    water(&mut w);
    (w, s)
}

/// Sets the cells within 3 of the sea's middle to water (the land the cell
/// tests read; the synthetic map's tiles are clear ground, and placing an
/// object recomputes its cell's land).
fn water(w: &mut World) {
    for dx in -3..=3 {
        for dy in -3..=3 {
            w.cells.grid.get_mut(sea(dx, dy)).unwrap().land = land::WATER;
        }
    }
}

// spec: S-0519 claim 4 (a) (revision 21, Q-3159: a vessel its own bomb kills and sets sinking takes its first descent of S-0504 claim 4 in that turn: from height 0 it is at −5 leptons at the end of the detonation tick and leaves play in the 80th tick after it) and S-0504 claim 4 (5 leptons per own update; it leaves play below −400 leptons)
#[test]
fn a_vessel_its_bomb_sinks_descends_in_the_detonation_turn() {
    let (mut w, s) = ship_world();
    assert_eq!(w.height_above_surface(s), 0, "idle at height 0");
    assert!(naval::sinks_at_kill(&w, s), "set-up: it would sink");
    arm(&mut w, s);
    testkit::run(&mut w, 1);
    // spec: S-0312 claim 10 (a vessel that sinks has its health restored to 1 health point; it keeps the killing mark)
    assert!(
        w.obj(s).is_some_and(|o| o.combat.killed),
        "killed by its bomb"
    );
    assert!(naval::sinking(&w, s), "it sinks");
    assert_eq!(
        w.height_above_surface(s),
        -5,
        "the first descent comes in the detonation tick"
    );
    let mut after = 0;
    while w.obj(s).is_some() {
        testkit::run(&mut w, 1);
        after += 1;
        assert!(after <= 200, "it leaves play");
    }
    assert_eq!(
        after, 80,
        "it leaves play in the 80th tick after the detonation"
    );
}

// ---------------------------------------------------------------------------
// (b) An aircraft in ordinary flying movement

/// FLYER: a landable aircraft of house 0 with flying movement, strength 50,
/// unarmed; flying east at cruise height when returned.
fn flying() -> (World, ObjectId) {
    let mut r = econ_common::ruleset();
    bomb_rules(&mut r);
    let mut u = UnitType {
        strength: Some(50),
        speed: Some(30),
        rot: Some(12),
        locomotor: Some(Locomotor::Fly),
        tech_level: Some(1),
        owner: Some(vec!["Alphaland".into(), "Betaland".into()]),
        cost: Some(1000),
        ..Default::default()
    };
    u.econ.landable = Some(true);
    r.registries
        .entry("aircraft".into())
        .or_default()
        .push("FLYER".into());
    r.sim.aircraft.insert("FLYER".into(), u);
    let mut w = econ_common::game(&r);
    econ_common::ready(&mut w);
    let a = testkit::spawn(
        &mut w,
        Kind::Aircraft,
        "FLYER",
        0,
        testkit::mid(econ_common::SIZE, -10, 0),
        0,
    );
    econ_common::step(&mut w);
    let c = w.obj(a).unwrap().pos.cell();
    assert!(econ_common::issue(
        &mut w,
        0,
        Order::Behaviour {
            actor: a,
            behaviour: Behaviour::Move,
            attack_move: false,
            target: None,
            dest: Some(Target::Cell(CellXY::new(c.x + 18, c.y))),
        }
    ));
    for _ in 0..600 {
        econ_common::step(&mut w);
        let s = w.obj(a).unwrap().air.clone().unwrap_or_default();
        let h = air::height(&w, a);
        if h > 0 && !s.taking_off && h == s.held {
            return (w, a);
        }
    }
    panic!("never at cruise height");
}

// spec: S-0519 claim 4 (b) (revision 21, Q-3159: an aircraft in ordinary flying movement that its own bomb kills takes the first step of S-0501 claim 9's crash fall in that turn, its fall counter at 1)
#[test]
fn an_aircraft_its_bomb_kills_takes_its_first_crash_step_in_that_turn() {
    let (mut w, a) = flying();
    let before = air::height(&w, a);
    arm(&mut w, a);
    econ_common::step(&mut w);
    assert!(killed(&w, a), "killed by its bomb");
    assert!(air::crashing(&w, a), "it crashes");
    let s = w.obj(a).unwrap().air.clone().unwrap_or_default();
    assert_eq!(
        s.crash,
        Some(1),
        "the fall counter after the detonation tick"
    );
    assert!(
        air::height(&w, a) < before,
        "it is lower at the end of the detonation tick"
    );
}

// ---------------------------------------------------------------------------
// (c) A vehicle in jumpjet movement

fn jump_world() -> World {
    let mut r = jump_common::ruleset();
    bomb_rules(&mut r);
    let mut w = testkit::game_with(&r, jump_common::SIZE, 7);
    testkit::run(&mut w, 4);
    w
}

/// JV (a crashable jumpjet vehicle) in the air at 500 leptons in `phase`;
/// cruising toward a cell 10 east when `moving`.
fn jumper(w: &mut World, phase: Phase, moving: bool) -> ObjectId {
    let u = jump_common::put(w, "JV", Kind::Vehicle, 0, -4, 0);
    if moving {
        jump_common::go(w, u, jump_common::at(10, 0));
        let t = w.tick;
        w.obj_mut(u).unwrap().beh.force(Behaviour::Move, t, true);
    }
    jump::start_in_flight(w, u, phase, 500);
    if moving {
        let mode = w.rules().mode;
        w.obj_mut(u).unwrap().jump.as_mut().unwrap().vc = F64::from_i64(14, mode);
    }
    u
}

// spec: S-0519 claim 4 (c) (revision 21, Q-3159: a vehicle in jumpjet movement that its own bomb kills takes S-0502 claim 18's first crash update at that turn's movement update; in our engine, by S-0502 claim 3's safety departure, also when it is neither moving nor moving now)
#[test]
fn a_jumpjet_its_bomb_kills_takes_its_first_crash_update_in_that_turn() {
    for (phase, moving) in [(Phase::Cruising, true), (Phase::Hovering, false)] {
        let mut w = jump_world();
        let u = jumper(&mut w, phase, moving);
        testkit::run(&mut w, 2);
        let before = jump_common::height(&w, u);
        assert!(before > 0, "{phase:?}: in the air");
        arm(&mut w, u);
        testkit::run(&mut w, 1);
        assert!(killed(&w, u), "{phase:?}: killed by its bomb");
        assert!(jump::crashing(&w, u), "{phase:?}: it crashes");
        assert_eq!(
            jump_common::phase(&w, u),
            Phase::Crashing,
            "{phase:?}: the first crash update comes in the detonation tick"
        );
    }
}
