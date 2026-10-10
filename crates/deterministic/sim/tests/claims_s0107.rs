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
