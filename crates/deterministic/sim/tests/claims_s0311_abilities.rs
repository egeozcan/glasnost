//! S-0311 claim 4: the abilities whose effects other specs own (faster,
//! crusher, area guard), applied at the object's current rank.

use glasnost_num::{F64, Rounding};
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::Decimal;
use glasnost_sim::occupancy;
use glasnost_sim::process;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit::{self, mid, spawn};
use glasnost_sim::walk;
use glasnost_sim::world::{ObjectId, World};

const SIZE: i32 = 32;

fn veteran(w: &mut World, id: ObjectId) {
    let rz = Rounding::TowardZero;
    w.obj_mut(id).unwrap().combat.experience = F64::from_i64(1, rz).to_f32(rz);
}

fn ruleset() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    r.sim.general.veteran_speed = Some(Decimal("1.5".into()));
    r
}

// spec: S-0311 claim 4 (faster: speed times the veteran speed factor, truncated; owner S-0208 F5)
// spec: S-0208 F5 (s = trunc(s₁ × kᵥ) with the faster ability)
#[test]
fn faster_infantry_walk_faster_at_the_granting_rank() {
    let mut r = ruleset();
    r.sim.infantry.get_mut("WALKER").unwrap().veteran_abilities = vec!["FASTER".into()];
    let mut w = testkit::game_with(&r, SIZE, 3);
    let id = spawn(&mut w, Kind::Infantry, "WALKER", 0, mid(SIZE, 0, 0), 0);
    let s1 = walk::walking_speed(&w, id);
    veteran(&mut w, id);
    let s = walk::walking_speed(&w, id);
    let mode = w.rules().mode;
    let want = F64::from_i64(i64::from(s1), mode)
        .mul(glasnost_sim::rules::nearest_double("1.5"), mode)
        .to_i32_common();
    assert!(s1 > 1);
    assert_eq!(s, want);
}

// spec: S-0311 claim 4 (faster for vehicles; owner S-0207 claim 2)
// spec: S-0207 claim 2 (s₂ = trunc(s₁ × ν) with the faster ability)
#[test]
fn faster_vehicles_drive_further() {
    let mut r = ruleset();
    r.sim.vehicles.get_mut("CART").unwrap().veteran_abilities = vec!["FASTER".into()];
    let mut w = testkit::game_with(&r, SIZE, 3);
    let rookie = spawn(&mut w, Kind::Vehicle, "CART", 0, mid(SIZE, -6, -4), 64);
    let vet = spawn(&mut w, Kind::Vehicle, "CART", 0, mid(SIZE, -6, 4), 64);
    veteran(&mut w, vet);
    w.tick = 10;
    for (id, dy) in [(rookie, -4), (vet, 4)] {
        glasnost_sim::orders::issue(
            &mut w,
            glasnost_sim::world::HouseId(0),
            glasnost_sim::orders::Order::Behaviour {
                actor: id,
                behaviour: Behaviour::Move,
                attack_move: false,
                target: None,
                dest: Some(glasnost_sim::world::Target::Cell(mid(SIZE, 10, dy))),
            },
        );
    }
    testkit::run(&mut w, 40);
    let x = |id| w.obj(id).unwrap().pos.x;
    assert!(x(vet) > x(rookie), "{} vs {}", x(vet), x(rookie));
}

// spec: S-0311 claim 4 (crusher: crushes as a crusher type does; owner S-0210 claim 1)
// spec: S-0210 claim 1 (a crusher: the type flag, or the crusher veterancy ability)
#[test]
fn the_crusher_ability_makes_a_crusher_at_its_rank() {
    let mut r = ruleset();
    r.sim.vehicles.get_mut("CART").unwrap().elite_abilities = vec!["CRUSHER".into()];
    let mut w = testkit::game_with(&r, SIZE, 3);
    let id = spawn(&mut w, Kind::Vehicle, "CART", 0, mid(SIZE, 0, 0), 0);
    assert!(!occupancy::is_crusher(&w, id));
    veteran(&mut w, id);
    assert!(
        !occupancy::is_crusher(&w, id),
        "an elite-list ability needs elite rank"
    );
    let rz = Rounding::TowardZero;
    w.obj_mut(id).unwrap().combat.experience = F64::from_i64(2, rz).to_f32(rz);
    assert!(occupancy::is_crusher(&w, id));
}

// spec: S-0311 claim 4 (area guard: rests in area guard instead of guard; owner S-0215 claim 14 row 11)
// spec: S-0215 claim 14 row 11 (infantry with the guard-area ability fall back to area guard)
#[test]
fn guard_area_ability_only_at_its_rank() {
    let mut r = ruleset();
    r.sim.infantry.get_mut("WALKER").unwrap().veteran_abilities = vec!["GUARD_AREA".into()];
    for (vet, want) in [(false, Behaviour::Guard), (true, Behaviour::AreaGuard)] {
        let mut w = testkit::game_with(&r, SIZE, 3);
        let id = spawn(&mut w, Kind::Infantry, "WALKER", 0, mid(SIZE, 0, 0), 0);
        if vet {
            veteran(&mut w, id);
        }
        let t = w.tick;
        w.obj_mut(id).unwrap().beh.force(Behaviour::Stop, t, true);
        process::fall_back(&mut w, id, false);
        let o = w.obj(id).unwrap();
        assert!(
            o.beh.current == Some(want) || o.beh.queued == Some(want),
            "veteran {vet}: {:?} / {:?}",
            o.beh.current,
            o.beh.queued
        );
    }
}
