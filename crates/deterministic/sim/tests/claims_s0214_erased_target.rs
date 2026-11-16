//! The target-state condition of S-0214 table V value *n* (an accepting
//! allied building is not being erased) and of the engineer-friendly
//! building's passive-neutral branch (#358). On the synthetic rules of
//! `structures_common` plus an invented 3 × 3 repair depot DEPOT; no game
//! data.

mod structures_common;

use glasnost_rules::sim::BuildingType;
use glasnost_sim::rules::Kind;
use glasnost_sim::superweapons::warp::{Member, Stage};
use glasnost_sim::targeting::{self, Action, ClickOn, Modifiers};
use glasnost_sim::world::{ObjectId, World};
use structures_common::*;

/// Makes `id` be erased: a superweapon warp member in its leaving stage
/// (S-0214 Terms, *being erased*).
fn erase(w: &mut World, id: ObjectId) {
    let pos = w.obj(id).unwrap().pos;
    let now = w.tick;
    w.obj_mut(id).unwrap().sw.warp = Some(Member {
        stage: Stage::Leaving(now),
        house: A,
        landing: pos,
        redirected: false,
        fell_back: false,
        refused: false,
        entry_ok: false,
    });
    assert!(
        glasnost_sim::superweapons::being_erased(w, id),
        "set-up: being erased"
    );
}

fn action(w: &World, actor: ObjectId, on: ObjectId) -> Action {
    targeting::action(w, A, actor, ClickOn::Object(on), Modifiers::default())
}

// spec: S-0214 table V value n (an accepting allied building: a building allied with the actor, not being erased, whose docking answer is accepts; row n3: enter) and row n4 (otherwise b)
#[test]
fn issue_358_a_depot_being_erased_is_no_accepting_allied_building() {
    let mut r = ruleset();
    r.registries
        .get_mut("buildings")
        .unwrap()
        .push("DEPOT".into());
    r.sim.buildings.insert(
        "DEPOT".into(),
        BuildingType {
            strength: Some(800),
            foundation: Some([3, 3]),
            unit_repair: Some(true),
            number_impassable_rows: Some(0),
            ..Default::default()
        },
    );
    let mut w = game_on(&r, triggers());
    let d = put(&mut w, Kind::Building, "DEPOT", 0, 0, 0);
    let v = put(&mut w, Kind::Vehicle, "CART", 0, 6, 1);
    until(&mut w, 10, |w| w.tick >= 4);
    assert_eq!(action(&w, v, d), Action::Enter, "control: row n3");
    erase(&mut w, d);
    assert_ne!(
        action(&w, v, d),
        Action::Enter,
        "value n: a depot being erased is not an accepting allied building"
    );
}

// spec: S-0214 table I (an engineer-friendly building: one that belongs to a passive neutral house, can be garrisoned and is not being erased) and rows I4 to I6
#[test]
fn issue_358_a_passive_neutral_building_being_erased_is_not_engineer_friendly() {
    let mut w = game();
    let hut = put(&mut w, Kind::Building, "HUT", 2, 0, 0);
    let e = put(&mut w, Kind::Infantry, "ENGI", 0, 6, 6);
    let full = w.obj(hut).unwrap().health;
    w.obj_mut(hut).unwrap().health = full / 2;
    until(&mut w, 10, |w| w.tick >= 4);
    let before = action(&w, e, hut);
    assert_eq!(before, Action::RepairBuilding, "control: row I6");
    erase(&mut w, hut);
    assert_ne!(
        action(&w, e, hut),
        Action::RepairBuilding,
        "a building being erased is not engineer-friendly"
    );
}
