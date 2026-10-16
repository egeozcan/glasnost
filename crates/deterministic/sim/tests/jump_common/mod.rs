//! Shared fixtures of the jumpjet claim tests (S-0502): a synthetic ruleset
//! with jumpjet infantry and vehicle types and a game of houses Alpha (0)
//! and Beta (1) on an empty flat map. Every name and number is invented; no
//! game data.
#![allow(dead_code)]

pub mod factory;

use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{
    BuildingType, Decimal, LocomotionClass, Locomotor, MovementZone, UnitType, WeaponType,
};
use glasnost_sim::econ::assign_destination;
use glasnost_sim::jump::{self, Phase};
use glasnost_sim::rules::Kind;
use glasnost_sim::testkit;
use glasnost_sim::world::{ObjectId, Pos, Target, World};

pub const SIZE: i32 = 40;

pub fn d(s: &str) -> Decimal {
    Decimal(s.into())
}

pub fn at(dx: i32, dy: i32) -> CellXY {
    testkit::mid(SIZE, dx, dy)
}

fn jumper(kind_infantry: bool) -> UnitType {
    UnitType {
        strength: Some(200),
        speed: Some(4),
        locomotion_class: Some(if kind_infantry {
            LocomotionClass::Foot
        } else {
            LocomotionClass::Hover
        }),
        movement_zone: Some(MovementZone::Fly),
        locomotor: Some(Locomotor::Jumpjet),
        jumpjet: Some(true),
        sight: Some(3),
        crushable: Some(true),
        jumpjet_speed: Some(14),
        jumpjet_climb: Some(d("5.0")),
        jumpjet_crash: Some(d("6.0")),
        jumpjet_height: Some(500),
        jumpjet_accel: Some(d("2.0")),
        jumpjet_wobbles: Some(d("0.15")),
        jumpjet_deviation: Some(40),
        jumpjet_turn_rate: Some(4),
        primary: Some("Rifle".into()),
        ..Default::default()
    }
}

/// The ruleset: infantry JJ (a jumpjet), JJTURN (with the jumpjet-turn
/// flag), JJFLAT (no wobble); vehicles JV (a crashable jumpjet), BALLOON
/// (balloon-hover), DEPL (a simple deployer that deploys to land) and
/// DEPN (a simple deployer without the landing flag); buildings TALL (art
/// height 3) and LOW (art height 1).
pub fn ruleset() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    r.registries.get_mut("infantry").unwrap().extend(
        [
            "JJ", "JJTURN", "JJFLAT", "JJFAST", "JJCRASH", "JJTALL", "JJLOCO", "JJBAL",
        ]
        .map(String::from),
    );
    r.registries
        .get_mut("vehicles")
        .unwrap()
        .extend(["JV", "JVTALL", "BALLOON", "DEPL", "DEPN", "BDEPN"].map(String::from));
    r.registries.insert(
        "buildings".into(),
        ["TALL", "LOW"].map(String::from).to_vec(),
    );
    let s = &mut r.sim;
    s.infantry.insert("JJ".into(), jumper(true));
    let mut turn = jumper(true);
    turn.jump_jet_turn = Some(true);
    s.infantry.insert("JJTURN".into(), turn);
    let mut fast = jumper(true);
    fast.jumpjet_speed = Some(40);
    s.infantry.insert("JJFAST".into(), fast);
    let mut crash = jumper(true);
    crash.crashable = Some(true);
    s.infantry.insert("JJCRASH".into(), crash);
    let mut tall = jumper(true);
    tall.jumpjet_height = Some(2000);
    s.infantry.insert("JJTALL".into(), tall);
    let mut flat = jumper(true);
    flat.jumpjet_no_wobbles = Some(true);
    s.infantry.insert("JJFLAT".into(), flat);
    // A balloon-hover infantry jumpjet (S-0502 claims 24 and 28: stop makes two empty assignments, with no vehicle equality suppression).
    let mut jjbal = jumper(true);
    jjbal.balloon_hover = Some(true);
    s.infantry.insert("JJBAL".into(), jjbal);
    let mut jv = jumper(false);
    jv.crashable = Some(true);
    s.vehicles.insert("JV".into(), jv);
    // The jumpjet locomotor without the JumpJet type flag: the own-cell filter of S-0502 claim 23 concerns the flag.
    let mut loco = jumper(true);
    loco.jumpjet = Some(false);
    s.infantry.insert("JJLOCO".into(), loco);
    let mut jvtall = jumper(false);
    jvtall.crashable = Some(true);
    jvtall.jumpjet_height = Some(2000);
    s.vehicles.insert("JVTALL".into(), jvtall);
    let mut bal = jumper(false);
    bal.balloon_hover = Some(true);
    bal.mobile_fire = Some(true);
    bal.primary = Some("Cannon".into());
    s.vehicles.insert("BALLOON".into(), bal);
    let mut depl = jumper(false);
    depl.is_simple_deployer = Some(true);
    depl.deploy_to_land = Some(true);
    s.vehicles.insert("DEPL".into(), depl);
    let mut bdepn = jumper(false);
    bdepn.balloon_hover = Some(true);
    bdepn.is_simple_deployer = Some(true);
    s.vehicles.insert("BDEPN".into(), bdepn);
    let mut depn = jumper(false);
    depn.is_simple_deployer = Some(true);
    s.vehicles.insert("DEPN".into(), depn);
    for (n, art) in [("TALL", 3), ("LOW", 1)] {
        let mut b = BuildingType {
            strength: Some(500),
            foundation: Some([1, 1]),
            art_height: Some(art),
            ..Default::default()
        };
        b.combat.armor = None;
        s.buildings.insert(n.into(), b);
    }
    s.weapons.insert(
        "Cannon".into(),
        WeaponType {
            damage: Some(10),
            rof: Some(20),
            range: Some(d("5")),
            speed: Some(100),
            projectile: Some("Bullet".into()),
            warhead: Some("SA".into()),
            fire_while_moving: Some(false),
            ..Default::default()
        },
    );
    s.weapons.insert(
        "Rifle".into(),
        WeaponType {
            damage: Some(10),
            rof: Some(20),
            range: Some(d("5")),
            speed: Some(100),
            projectile: Some("Bullet".into()),
            warhead: Some("SA".into()),
            ..Default::default()
        },
    );
    r
}

pub fn game() -> World {
    let mut w = testkit::game_with(&ruleset(), SIZE, 7);
    testkit::run(&mut w, 4);
    w
}

pub fn put(w: &mut World, name: &str, kind: Kind, house: u8, dx: i32, dy: i32) -> ObjectId {
    testkit::spawn(w, kind, name, house, at(dx, dy), 0)
}

/// Gives a unit a destination cell through the assignment (a move order's
/// effect on the locomotor).
pub fn go(w: &mut World, id: ObjectId, c: CellXY) {
    assign_destination(w, id, Some(Target::Cell(c)));
}

/// Orders a jumpjet vehicle to its own cell with the exact point kept
/// (S-0502 claim 16: a vehicle whose behaviour is *enter* keeps the ordered
/// point), the way an accepted same-cell movement point arises (claim 7;
/// claim 23: an ordinary vehicle's own mark rejects its own cell).
pub fn go_here_entering(w: &mut World, id: ObjectId) {
    let c = pos(w, id).cell();
    w.obj_mut(id)
        .unwrap()
        .beh
        .queue(Some(glasnost_sim::rules::Behaviour::Enter));
    go(w, id, c);
}

/// Gives a jumpjet already placed in the air a movement commitment with the
/// centre of its own cell as the movement point (S-0502 Terms, *moving*): the
/// state a completed flight leaves a hovering unit in. An order to its own
/// cell does not give it, since claim 23 filters infantry and the vehicle's
/// search rejects its own mark.
pub fn commit_here(w: &mut World, id: ObjectId) {
    let c = pos(w, id).cell();
    let j = w.obj_mut(id).unwrap().jump.as_mut().unwrap();
    j.moving = true;
    j.dest = Some(Pos::centre(c, 0));
}

pub fn phase(w: &World, id: ObjectId) -> Phase {
    jump::phase(w, id).expect("a jumpjet")
}

pub fn height(w: &World, id: ObjectId) -> i32 {
    w.height_above_surface(id)
}

pub fn pos(w: &World, id: ObjectId) -> Pos {
    w.obj(id).unwrap().pos
}

/// One tick.
pub fn step(w: &mut World) {
    testkit::run(w, 1);
}

/// Runs ticks until `f` holds, at most `max`; whether it held.
pub fn until(w: &mut World, max: i32, f: impl Fn(&World) -> bool) -> bool {
    for _ in 0..max {
        if f(w) {
            return true;
        }
        step(w);
    }
    f(w)
}
