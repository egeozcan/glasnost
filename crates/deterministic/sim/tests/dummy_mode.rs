//! Our skirmish dummy mode (#439, a testing tool of ours, not the
//! original's): with `GameSetup::dummy` on, no object of another house
//! takes a person's object as a target (its own search, retaliation, area
//! guard), a person's hits make no house angry, and the person's house has
//! no shroud from tick 0 on. Off, nothing changes: the house setup state
//! keeps its bytes, so the state hash is the one the game had before the
//! mode existed. Synthetic data only.

use std::sync::Arc;

use glasnost_sim::econ::setup::{HouseSetupState, Row};
use glasnost_sim::econ::shroud;
use glasnost_sim::rules::{Kind, SimRules};
use glasnost_sim::save;
use glasnost_sim::testkit::{self, mid, spawn};
use glasnost_sim::world::{GameSetup, HouseId, ObjectId, Pos, Target, World};
use serde::Serialize;

const SIZE: i32 = 30;

/// The sandbox setup of `testkit` with house 1 an AI house and dummy mode
/// as given: house 0 is the person's.
fn setup(seed: u32, dummy: bool) -> GameSetup {
    let mut s = testkit::setup(seed);
    s.houses[1].ai = true;
    s.dummy = dummy;
    s
}

fn game(seed: u32, dummy: bool) -> World {
    let rules = Arc::new(SimRules::new(&testkit::combat_ruleset(), "synthetic-test"));
    glasnost_sim::load::new_game(
        rules,
        &testkit::empty_map(SIZE),
        testkit::catalogue(),
        &setup(seed, dummy),
    )
    .expect("synthetic test game loads")
}

/// Two armed squads side by side: the person's (house 0) and the AI
/// house's (house 1), every unit in *guard*.
fn squads(w: &mut World) -> (Vec<ObjectId>, Vec<ObjectId>) {
    let mut person = Vec::new();
    let mut ai = Vec::new();
    for dy in -1..=1 {
        person.push(spawn(w, Kind::Vehicle, "CART", 0, mid(SIZE, -2, dy * 2), 0));
        ai.push(spawn(w, Kind::Vehicle, "CART", 1, mid(SIZE, 2, dy * 2), 0));
    }
    (person, ai)
}

fn owner(w: &World, t: Option<Target>) -> Option<HouseId> {
    match t {
        Some(Target::Object(x)) => w.obj(x).and_then(|o| o.owner),
        _ => None,
    }
}

/// What a fight of [`squads`] came to over `ticks` ticks: whether a unit of
/// the AI house ever held a person's object as its target, whether a
/// person's unit ever held an AI object, and the person's units' health
/// lost.
fn fight(dummy: bool, ticks: i32) -> (bool, bool, i32, World) {
    let mut w = game(7, dummy);
    let (person, ai) = squads(&mut w);
    let health = |w: &World| -> i32 { person.iter().filter_map(|&p| w.obj(p)).map(|o| o.health).sum() };
    let before = health(&w);
    let (mut ai_aimed, mut person_aimed) = (false, false);
    for _ in 0..ticks {
        testkit::run(&mut w, 1);
        for &a in &ai {
            if let Some(o) = w.obj(a) {
                ai_aimed |= owner(&w, o.target) == Some(HouseId(0));
            }
        }
        for &p in &person {
            if let Some(o) = w.obj(p) {
                person_aimed |= owner(&w, o.target) == Some(HouseId(1));
            }
        }
    }
    let lost = before - health(&w);
    (ai_aimed, person_aimed, lost, w)
}

/// The person-played house is the dummy-mode house; the AI house is not,
/// and with the mode off neither is.
// design: #439 (dummy mode marks the person-played houses at game start)
#[test]
fn the_person_played_house_is_marked() {
    let w = game(7, true);
    assert!(w.houses[0].econ.setup.dummy);
    assert!(!w.houses[1].econ.setup.dummy);
    assert!(w.dummy_spares(Some(HouseId(1)), Some(HouseId(0))));
    assert!(!w.dummy_spares(Some(HouseId(0)), Some(HouseId(1))));
    let w = game(7, false);
    assert!(w.houses.iter().all(|h| !h.econ.setup.dummy));
    assert!(!w.dummy_spares(Some(HouseId(1)), Some(HouseId(0))));
}

/// With the mode on, the AI house's units never take a person's object as
/// target (search, area guard, retaliation) and never hurt it, while the
/// person's units attack them; with it off, the same seed fights both ways.
// design: #439 (no object of another house targets a dummy-mode house's objects; the person may attack, and they do not fight back)
#[test]
fn ai_objects_never_target_the_person() {
    let (ai_aimed, person_aimed, lost, w) = fight(true, 400);
    assert!(!ai_aimed, "an AI unit took a person's object as its target");
    assert!(person_aimed, "the person's units attack the AI's");
    assert_eq!(lost, 0, "the person's units were hurt");
    // No house is angry with the person (S-0309 F8 is not run for its hits).
    assert!(w.houses[1].combat.anger.iter().all(|&a| a == 0));
    assert_eq!(w.houses[1].combat.enemy, None);
    let (ai_aimed, person_aimed, lost, _) = fight(false, 400);
    assert!(ai_aimed && person_aimed && lost > 0, "with the mode off they fight");
}

/// The person's house has every cell explored and seen at tick 0, no point
/// is shrouded for it, and none waits to be covered again; the AI house
/// keeps its shroud. With the mode off the person's house has a shroud.
// design: #439 (dummy mode: no shroud for the person from tick 0 on)
#[test]
fn the_person_sees_every_cell_at_tick_0() {
    let w = game(7, true);
    assert_eq!(w.tick, 0);
    let cells: Vec<_> = w.cells.grid.iter().map(|(c, _)| c).collect();
    assert!(!cells.is_empty());
    let h = HouseId(0);
    for &c in &cells {
        assert!(shroud::explored(&w, h, c), "{c:?} unexplored");
        assert!(shroud::seen(&w, h, c), "{c:?} unseen");
        assert!(!shroud::pending(&w, h, c), "{c:?} pending");
        assert!(!shroud::point_shrouded(&w, h, Pos::centre(c, 0)), "{c:?} shrouded");
    }
    assert!(cells.iter().any(|&c| !shroud::explored(&w, HouseId(1), c)), "the AI house keeps its shroud");
    let off = game(7, false);
    assert!(cells.iter().any(|&c| !shroud::explored(&off, h, c)), "off: the person has a shroud");
}

/// The shroud stays removed: after a fight, every cell is still explored
/// for the person.
// design: #439 (the shroud stays removed)
#[test]
fn the_shroud_stays_removed() {
    let (_, _, _, w) = fight(true, 120);
    assert!(w.cells.grid.iter().all(|(c, _)| shroud::explored(&w, HouseId(0), c)));
}

/// A save round trip keeps the mark and the state hash, and the game goes
/// on alike.
// design: #439 (the dummy-mode mark is saved and hashed)
#[test]
fn a_save_round_trip_keeps_the_mark_and_the_hash() {
    let (_, _, _, mut w) = fight(true, 120);
    let bytes = save::save(&w);
    let mut l = save::load(&bytes, w.rules_arc(), testkit::catalogue()).expect("loads");
    assert!(l.houses[0].econ.setup.dummy && !l.houses[1].econ.setup.dummy);
    assert_eq!(save::state_hash(&l), save::state_hash(&w));
    testkit::run(&mut w, 60);
    testkit::run(&mut l, 60);
    assert_eq!(save::state_hash(&l), save::state_hash(&w));
    // The mark is in the hash: the same world without it hashes otherwise.
    let mut other = w.clone();
    other.houses[0].econ.setup.dummy = false;
    assert_ne!(save::state_hash(&other), save::state_hash(&w));
}

/// [`HouseSetupState`] as it was saved before dummy mode: the same fields
/// in the same order, the person-played value a boolean.
#[derive(Serialize)]
struct Before {
    team: Option<u8>,
    handicap: Option<Row>,
    colour: Option<u8>,
    observer: bool,
    person_played: bool,
    difficulty: i32,
    ai: bool,
}

/// With the mode off the house setup state has the bytes it had before
/// the mode existed, so a game's saves and state hashes are unchanged; a
/// person-played byte of 3 is the mark, and one above 3 is refused.
// design: #439 (off, nothing changes: the mark rides in the person-played byte)
#[test]
fn off_the_house_setup_bytes_are_unchanged() {
    for (pp, ai, team, handicap) in [
        (false, false, None, None),
        (true, false, Some(2), Some(Row::Normal)),
        (false, true, Some(1), None),
        (true, true, None, Some(Row::Easy)),
    ] {
        let s = HouseSetupState {
            team,
            handicap,
            colour: Some(3),
            observer: false,
            person_played: pp,
            difficulty: 1,
            ai,
            dummy: false,
        };
        let before = Before {
            team,
            handicap,
            colour: Some(3),
            observer: false,
            person_played: pp,
            difficulty: 1,
            ai,
        };
        let bytes = postcard::to_allocvec(&s).unwrap();
        assert_eq!(bytes, postcard::to_allocvec(&before).unwrap());
        assert_eq!(postcard::from_bytes::<HouseSetupState>(&bytes).unwrap(), s);
    }
    let marked = HouseSetupState {
        person_played: true,
        dummy: true,
        ..Default::default()
    };
    let bytes = postcard::to_allocvec(&marked).unwrap();
    assert_eq!(postcard::from_bytes::<HouseSetupState>(&bytes).unwrap(), marked);
    let mut bad = bytes.clone();
    // The person-played byte follows the three absent options and the observer byte.
    assert_eq!(bad[4], 3);
    bad[4] = 4;
    assert!(postcard::from_bytes::<HouseSetupState>(&bad).is_err());
}

/// The setup's field reaches the world only through the marks: in a game
/// with no person-played house (two AI houses, a watched game) the mode
/// marks no house, and the game with the field on has the state hashes of
/// the game with it off, tick by tick.
// design: #439 (the setup field is not in the world; it marks the person-played houses only)
#[test]
fn the_field_alone_changes_no_world_byte() {
    let all_ai = |dummy: bool| {
        let mut s = setup(7, dummy);
        s.houses[0].ai = true;
        let rules = Arc::new(SimRules::new(&testkit::combat_ruleset(), "synthetic-test"));
        glasnost_sim::load::new_game(rules, &testkit::empty_map(SIZE), testkit::catalogue(), &s)
            .expect("synthetic test game loads")
    };
    let mut a = all_ai(false);
    let mut b = all_ai(true);
    assert!(b.houses.iter().all(|h| !h.econ.setup.dummy));
    squads(&mut a);
    squads(&mut b);
    for _ in 0..200 {
        testkit::run(&mut a, 1);
        testkit::run(&mut b, 1);
        assert_eq!(save::state_hash(&a), save::state_hash(&b));
    }
}
