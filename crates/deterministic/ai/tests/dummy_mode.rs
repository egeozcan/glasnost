//! Our skirmish dummy mode (#439, a testing tool of ours): with
//! `GameSetup::dummy` on, the AI house never plans against the person's
//! house (attack, hunt, harass, strikes, special abilities) and none of its
//! objects takes a person's object as a target; with it off, the same game
//! goes to the person's base. Synthetic fixture only.

use glasnost_ai::brain::Brain;
use glasnost_ai::fixture;
use glasnost_sim::econ::victory;
use glasnost_sim::orders::{self, Order};
use glasnost_sim::tick;
use glasnost_sim::world::{GameSetup, HouseId, Target, World};

/// The fixture's setup with house 0 played by a person (its MCV stays
/// where it starts: the person gives no orders) and house 1 the AI's.
fn setup(seed: u32, dummy: bool) -> GameSetup {
    let mut s = fixture::setup(seed);
    s.houses[0].ai = false;
    s.dummy = dummy;
    s
}

fn persons(w: &World, t: &Option<Target>) -> bool {
    matches!(t, Some(Target::Object(x)) if w.obj(*x).is_some_and(|o| o.owner == Some(HouseId(0))))
}

/// Whether an AI order names a person's object as its target, destination
/// or guard spot.
fn aims_at_person(w: &World, o: &Order) -> bool {
    match o {
        Order::Behaviour { target, dest, .. } => persons(w, target) || persons(w, dest),
        Order::GuardSpot { spot, .. } => persons(w, &Some(*spot)),
        _ => false,
    }
}

/// What `ticks` ticks of the game came to: whether an AI order or an AI
/// object's target ever named a person's object, the person's health lost
/// over its objects, and the tick the game stopped at.
fn play(seed: u32, dummy: bool, ticks: i32) -> (bool, bool, i32, i32) {
    let map = fixture::map();
    let mut w =
        glasnost_sim::load::new_game(fixture::rules(), &map, fixture::catalogue(), &setup(seed, dummy))
            .expect("the fixture loads");
    assert_eq!(w.houses[0].econ.setup.dummy, dummy);
    let mut brain = Brain::new(&w, &map, HouseId(1));
    let health = |w: &World| -> i32 {
        w.objects
            .values()
            .filter(|o| o.owner == Some(HouseId(0)) && o.on_map)
            .map(|o| o.health.max(0))
            .sum()
    };
    let before = health(&w);
    let (mut ordered, mut aimed) = (false, false);
    while w.tick < ticks && victory::outcome(&w).is_none() {
        for o in brain.think(&w) {
            ordered |= aims_at_person(&w, &o);
            orders::issue(&mut w, HouseId(1), o);
        }
        tick::step(&mut w);
        aimed |= w
            .objects
            .values()
            .any(|o| o.owner == Some(HouseId(1)) && persons(&w, &o.target));
    }
    (ordered, aimed, before - health(&w), w.tick)
}

/// The AI house builds its base and army for a while and never orders an
/// attack on the person's objects nor takes one as a target.
// design: #439 (the AI never picks the person's objects in its planning, and its objects never target them)
#[test]
fn the_ai_leaves_the_person_alone_early() {
    let (ordered, aimed, lost, _) = play(5, true, 4000);
    assert!(!ordered, "an AI order named a person's object");
    assert!(!aimed, "an AI object took a person's object as target");
    assert_eq!(lost, 0);
}

/// The whole of a long game: with the mode off the AI house goes for the
/// person's base; with it on it never does, and the person's objects are
/// never hurt.
// design: #439 (dummy mode on: no AI order or target against the person; off: the game as before)
#[test]
#[ignore = "long in debug builds; run in release (`cargo test --release -p glasnost-ai -- --include-ignored`)"]
fn the_ai_leaves_the_person_alone_all_game() {
    let limit = 40_000; // nocite: a test budget, not a behaviour
    let (ordered, aimed, lost, end) = play(5, false, limit);
    assert!(ordered || aimed, "off: the AI goes for the person (to tick {end})");
    assert!(lost > 0, "off: the person is hurt");
    let (ordered, aimed, lost, end) = play(5, true, limit);
    assert!(!ordered, "on: an AI order named a person's object (to tick {end})");
    assert!(!aimed, "on: an AI object took a person's object as target");
    assert_eq!(lost, 0, "on: the person's objects were hurt");
    assert_eq!(end, limit, "on: the game goes on");
}
