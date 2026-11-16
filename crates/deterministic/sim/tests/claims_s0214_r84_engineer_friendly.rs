//! S-0214 revision 84 (Q-3164, #358): an engineer-friendly building is a
//! repairable building, not vehicle-like, that is allied with the actor
//! (garrisonable or not, erased or not) or belongs to a passive neutral
//! house, can be garrisoned and is not being erased. The repairable and
//! not-vehicle-like tests apply in both cases. On the synthetic rules of
//! `structures_common` plus NOFIX (HUT whose type is not repairable) and
//! VHUT (HUT1 that undeploys into a vehicle, so vehicle-like); no game data.

mod structures_common;

use glasnost_sim::rules::Kind;
use glasnost_sim::superweapons::warp::{Member, Stage};
use glasnost_sim::targeting::{self, Action, ClickOn, Modifiers};
use glasnost_sim::world::{ObjectId, World};
use structures_common::*;

fn rules() -> glasnost_rules::schema::Ruleset {
    let mut r = ruleset();
    let mut nofix = r.sim.buildings["HUT"].clone();
    nofix.repairable = Some(false);
    // A vehicle-like building is 1 × 1 (S-0214 Terms), so VHUT starts from HUT1.
    let mut vhut = r.sim.buildings["HUT1"].clone();
    vhut.undeploys_into = Some("CART".into());
    for (n, b) in [("NOFIX", nofix), ("VHUT", vhut)] {
        r.registries.get_mut("buildings").unwrap().push(n.into());
        r.sim.buildings.insert(n.into(), b);
    }
    r
}

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

/// The action of house A's engineer over a damaged building `name` of house
/// `owner`, erased when `erased`.
fn engineer_over(name: &str, owner: u8, erased: bool) -> Action {
    engineer_over_in(name, owner, erased, false)
}

/// As `engineer_over`; when `ally_civ`, house A counts the civilian house
/// Civ (house 2) as allied (one-way, S-0214 Terms, *Allied, enemy*).
fn engineer_over_in(name: &str, owner: u8, erased: bool, ally_civ: bool) -> Action {
    let mut w = game_on(&rules(), triggers());
    if ally_civ {
        w.houses[0].allies = vec![A, glasnost_sim::world::HouseId(2)];
    }
    let b = put(&mut w, Kind::Building, name, owner, 0, 0);
    let e = put(&mut w, Kind::Infantry, "ENGI", 0, 6, 6);
    let full = w.obj(b).unwrap().health;
    w.obj_mut(b).unwrap().health = full / 2;
    until(&mut w, 10, |w| w.tick >= 4);
    if erased {
        erase(&mut w, b);
    }
    targeting::action(&w, A, e, ClickOn::Object(b), Modifiers::default())
}

// spec: S-0214 table I (revision 84, Q-3164: the repairable test applies to the passive-neutral case too) and rows I4 to I6
#[test]
fn a_passive_neutral_building_that_is_not_repairable_is_not_engineer_friendly() {
    assert_eq!(
        engineer_over("HUT", 2, false),
        Action::RepairBuilding,
        "control: row I6"
    );
    assert_ne!(
        engineer_over("NOFIX", 2, false),
        Action::RepairBuilding,
        "a type that is not repairable is not engineer-friendly"
    );
}

// spec: S-0214 table I (revision 84, Q-3164: the not-vehicle-like test applies to the passive-neutral case too) and rows I4 to I6
#[test]
fn a_passive_neutral_building_that_is_vehicle_like_is_not_engineer_friendly() {
    assert_eq!(
        engineer_over("HUT1", 2, false),
        Action::RepairBuilding,
        "control: row I6"
    );
    assert_ne!(
        engineer_over("VHUT", 2, false),
        Action::RepairBuilding,
        "a vehicle-like building is not engineer-friendly"
    );
}

// spec: S-0214 table I (revision 84, Q-3164: the allied case tests neither the garrison nor the being-erased state) and row I6
// spec: S-0508 claim 8 (an empty garrisonable building not of the civilian house becomes the civilian house's, so the garrisonable HUT is allied through house A's alliance with Civ)
#[test]
fn an_allied_building_is_engineer_friendly_garrisonable_or_not_erased_or_not() {
    for (name, owner, ally_civ) in [("HUT", 2, true), ("PLAIN", 0, false)] {
        for erased in [false, true] {
            assert_eq!(
                engineer_over_in(name, owner, erased, ally_civ),
                Action::RepairBuilding,
                "{name}, erased {erased}"
            );
        }
    }
}
