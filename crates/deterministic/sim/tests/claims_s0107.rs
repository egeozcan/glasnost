//! Claim tests for S-0107's idle actions on synthetic data, for owners
//! that are not human-controlled (the neutral and special houses, glossary
//! *AI house*, *Player houses only*).

mod common;

use common::{Scene, cell, unit_type};
use glasnost_sim::infantry::{self, Seq};
use glasnost_sim::rules::Kind;

fn fraidycat(human: bool) -> glasnost_sim::world::World {
    let mut s = Scene::new();
    let mut t = unit_type(common::FOOT, 4, 5);
    t.fraidycat = Some(true);
    s.infantry("FRAID", t);
    s.place_infantry(0, "FRAID", cell(0, 0), 0);
    s.setup.houses[0].human = human;
    let mut w = s.build();
    let id = common::objects_of(&w, Kind::Infantry)[0];
    let st = w.obj_mut(id).unwrap().infantry.as_mut().unwrap();
    st.seq = Some(Seq::Stand);
    st.fear = 60;
    st.idle_until = 0;
    infantry::idle_attempt(&mut w, id, false);
    w
}

/// spec: S-0107 claim 14 (a fraidycat above fear 50 whose owner is not human-controlled scatters at an idle attempt; one of a human-controlled owner, or of an AI house, does not)
#[test]
fn claim_14_fraidycat_idle_scatter() {
    let scattered = |w: &glasnost_sim::world::World| {
        let id = common::objects_of(w, Kind::Infantry)[0];
        let o = w.obj(id).unwrap();
        o.dest.is_some() || o.walk.as_ref().is_some_and(|s| s.target.is_some())
    };
    assert!(scattered(&fraidycat(false)));
    assert!(!scattered(&fraidycat(true)));
}

/// The scenario draws one idle attempt of a resting unit of a type
/// registered as `name` takes, with seed `seed`.
fn idle_draws(name: &str, seed: u32) -> u64 {
    let mut s = Scene::new();
    s.setup.seed = seed;
    s.infantry(name, unit_type(common::FOOT, 4, 5));
    s.place_infantry(0, name, cell(0, 0), 0);
    let mut w = s.build();
    let id = common::objects_of(&w, Kind::Infantry)[0];
    let st = w.obj_mut(id).unwrap().infantry.as_mut().unwrap();
    st.seq = Some(Seq::Stand);
    st.idle_until = 0;
    let before = w.rng.taken();
    infantry::idle_attempt(&mut w, id, false);
    w.rng.taken() - before
}

/// spec: S-0107 claim 14 and Import mapping (the hard-coded cow is the infantry type whose
/// registry name is the cow identifier, compared case-insensitively; it takes the extra draw e)
#[test]
fn claim_14_cow_by_registry_name_in_any_case() {
    for seed in 1..8 {
        let cow = idle_draws("COW", seed);
        assert_eq!(idle_draws("cow", seed), cow);
        assert_eq!(idle_draws("Cow", seed), cow);
    }
    // Another name takes no draw e, so over many seeds it takes fewer draws.
    let cow: u64 = (1..16).map(|s| idle_draws("cow", s)).sum();
    let other: u64 = (1..16).map(|s| idle_draws("COWS", s)).sum();
    assert!(cow > other);
}

/// A frightened, walking soldier of a type that can crawl, after one
/// sequence update while it moves; `human`: its owner's control.
fn frightened_walker(human: bool) -> Option<Seq> {
    let mut s = Scene::new();
    let mut t = unit_type(common::FOOT, 4, 5);
    for k in ["stand", "walk", "go_prone", "prone", "crawl"] {
        t.sequences.insert(
            k.into(),
            glasnost_rules::sim::SequenceEntry {
                start: 0,
                count: 4,
                multiplier: 0,
                direction: None,
            },
        );
    }
    s.infantry("SCARED", t);
    s.place_infantry(0, "SCARED", cell(0, 0), 0);
    s.setup.houses[0].human = human;
    let mut w = s.build();
    let id = common::objects_of(&w, Kind::Infantry)[0];
    let st = w.obj_mut(id).unwrap().infantry.as_mut().unwrap();
    st.seq = Some(Seq::Walk);
    st.fear = 60;
    infantry::update(&mut w, id, true, true, true);
    w.obj(id).unwrap().infantry.as_ref().unwrap().seq
}

/// spec: S-0107 claim 15 (the lie-down rule: a unit whose owner is not human-controlled, in our engine one of the neutral or special house, lies down whether or not it is moving; one of a human-controlled owner or an AI house waits until it is at rest)
#[test]
fn claim_15_non_player_houses_lie_down_while_moving() {
    assert_eq!(frightened_walker(false), Some(Seq::GoProne));
    assert_eq!(frightened_walker(true), Some(Seq::Walk));
}
