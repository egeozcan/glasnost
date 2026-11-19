//! Claim tests of S-0406 revision 20 and S-0309 revision 145 (Q-3261
//! answered): an owner change of the building a place order names while it
//! is in its factory break-off window. The change ends the window: from it
//! the building is construction-completed, with its idle body kept;
//! *guard* becomes current at the first per-tick start of its next turn,
//! *construction* is never processed and no free unit is made. Synthetic
//! rules only: invented types and numbers, no game data. Ledger:
//! `docs/m8-claims/sim-issues.md`, *Batch S42*.

use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::BuildingType;
use glasnost_sim::econ::placement;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::structures::ownership::{self, Cause};
use glasnost_sim::world::{HouseId, ObjectId, World};
use glasnost_sim::{objects, testkit, tick};

const SIZE: i32 = 24;
const A: HouseId = HouseId(0);
const B: HouseId = HouseId(1);

fn beh(w: &World, id: ObjectId) -> (Option<Behaviour>, Option<Behaviour>) {
    let o = w.obj(id).unwrap();
    (o.beh.current, o.beh.queued)
}

fn vehicles(w: &World) -> usize {
    w.objects
        .values()
        .filter(|o| o.kind == Kind::Vehicle)
        .count()
}

/// A free-unit building type of cost 1000 with the absent-key idle range
/// (delay 0): FREE has a build-up sprite of 10 frames, FREE0 none (a
/// zero-delay construction range, S-0406 claim 17).
fn rules() -> Ruleset {
    let mut r = testkit::ruleset();
    r.registries.insert(
        "buildings".into(),
        vec!["FREE".to_string(), "FREE0".to_string()],
    );
    let mut b = BuildingType {
        strength: Some(400),
        cost: Some(1000),
        foundation: Some([1, 1]),
        owner: Some(vec!["Alphaland".into(), "Betaland".into()]),
        ..Default::default()
    };
    b.econ.buildup = Some(true);
    b.econ.buildup_frames = Some(10);
    b.econ.free_unit = Some("CART".into());
    r.sim.buildings.insert("FREE".into(), b.clone());
    b.econ.buildup = Some(false);
    r.sim.buildings.insert("FREE0".into(), b);
    r
}

fn game() -> World {
    let mut w = testkit::game_with_non_players(&rules(), SIZE, 13);
    testkit::run(&mut w, 4);
    for h in &mut w.houses {
        h.credits = 10_000;
    }
    w
}

fn mid() -> CellXY {
    testkit::mid(SIZE, 0, 0)
}

/// A building of Alpha's of type `name` placed in play by order at the
/// middle of the map: it enters with *construction* queued.
fn placed(w: &mut World, name: &str) -> ObjectId {
    let t = w.rules().index_of(Kind::Building, name).unwrap();
    let id = objects::create(w, Kind::Building, t, Some(A));
    placement::put_on_map(w, id, mid(), true);
    id
}

fn building_up(w: &World, b: ObjectId) -> bool {
    w.obj(b).unwrap().econ.construction.building_up
}

/// Runs 200 ticks: *construction* is never current again and no free unit
/// appears.
fn assert_no_free_unit(w: &mut World, b: ObjectId) {
    let before = vehicles(w);
    for _ in 0..200 {
        tick::step(w);
        assert_ne!(beh(w, b).0, Some(Behaviour::Construction));
    }
    assert_eq!(vehicles(w), before, "no free unit, now or later");
}

// spec: S-0309 claim 31 (r145: in the factory break-off window with construction current, the guard request finds the building not ready and guard is queued, construction staying current until the next start; the change ends the window, the building construction-completed from it, its idle body kept with stage and delay unchanged; until that start a rule that reads the current behaviour sees construction; at the next turn the first start makes guard current, construction never processed, no free unit) and S-0406 claim 17 and its state table (r20)
#[test]
fn r20_construction_current_in_the_window_the_change_ends_it() {
    let mut w = game();
    // spec: S-0406 claim 17 (a zero-delay construction range: construction current from the second start of T + 1, in the window until C = T + 2)
    let b = placed(&mut w, "FREE0");
    for _ in 0..100 {
        if beh(&w, b).0 == Some(Behaviour::Construction) {
            break;
        }
        tick::step(&mut w);
    }
    let o = w.obj(b).unwrap();
    assert_eq!(o.beh.current, Some(Behaviour::Construction));
    assert_eq!(o.econ.construction.first, None, "not yet processed");
    assert!(placement::in_break_off_window(o), "in the window");
    let idle = o.econ.idle_body.clone();
    assert!(idle.is_some(), "the break-off's idle body");
    ownership::change(&mut w, b, Some(B), Cause::Trigger);
    let o = w.obj(b).unwrap();
    assert!(o.econ.construction.opened, "the grand opening");
    assert!(o.beh.ready_mark, "its ready mark is set");
    assert!(!building_up(&w, b), "construction-completed from the change");
    assert!(!placement::in_break_off_window(o), "the window has ended");
    assert_eq!(o.econ.idle_body, idle, "the idle body is kept unchanged");
    assert_eq!(o.beh.current, Some(Behaviour::Construction), "still current");
    assert_eq!(o.beh.queued, Some(Behaviour::Guard), "guard queued");
    assert!(
        placement::refused_as_in_construction(o),
        "construction is seen until the next start"
    );
    tick::step(&mut w);
    let o = w.obj(b).unwrap();
    assert_eq!(o.beh.current, Some(Behaviour::Guard));
    assert_eq!(o.econ.construction.first, None, "never processed");
    assert_eq!(o.econ.idle_body, idle, "still the break-off's idle body");
    assert!(!placement::refused_as_in_construction(o));
    assert_no_free_unit(&mut w, b);
}

// spec: S-0309 claim 31 (r145: with construction still queued in the window, guard replaces it; the change ends the window and the building is construction-completed from it, with its idle body kept; the result is the same as with construction current) and S-0406 claim 17 (r20)
#[test]
fn r20_construction_queued_in_the_window_the_change_ends_it() {
    let mut w = game();
    let b = placed(&mut w, "FREE");
    // spec: S-0406 claim 17 (placed in the orders part of T, the building is in the window with construction still queued after its first turn T + 1)
    testkit::run(&mut w, 1);
    let o = w.obj(b).unwrap();
    assert_eq!(beh(&w, b), (None, Some(Behaviour::Construction)));
    assert!(placement::in_break_off_window(o), "in the window");
    let idle = o.econ.idle_body.clone();
    ownership::change(&mut w, b, Some(B), Cause::Trigger);
    let o = w.obj(b).unwrap();
    assert_eq!(o.beh.queued, Some(Behaviour::Guard), "replaced by guard");
    assert!(!building_up(&w, b), "construction-completed from the change");
    assert_eq!(o.econ.idle_body, idle, "the idle body is kept unchanged");
    assert!(!placement::refused_as_in_construction(o));
    tick::step(&mut w);
    let o = w.obj(b).unwrap();
    assert_eq!(o.beh.current, Some(Behaviour::Guard));
    assert_eq!(o.econ.construction.first, None, "never processed");
    assert_no_free_unit(&mut w, b);
}
