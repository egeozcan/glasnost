//! The flee rule of S-0107 claim 15 (sim issue #186): every tick, a
//! fraidycat at rest that is not deploying and not falling scatters while
//! its fear level after the tick's decay is above 50, whatever its owner
//! (S-0315 claim 2). Synthetic data: one fraidycat infantry type on the
//! flat test map.

mod common;

use common::{FOOT, Scene, cell, objects_of, unit_type};
use glasnost_sim::infantry::Seq;
use glasnost_sim::rules::Kind;
use glasnost_sim::scatter::{self, ScatterRequest};
use glasnost_sim::tick;
use glasnost_sim::world::{ObjectId, World};

fn frightened(human: bool, fear: i32) -> (World, ObjectId) {
    let mut s = Scene::new();
    let mut t = unit_type(FOOT, 4, 5);
    t.fraidycat = Some(true);
    s.infantry("FRAID", t);
    s.place_infantry(0, "FRAID", cell(10, 10), 0);
    s.setup.houses[0].human = human;
    let mut w = s.build();
    let id = objects_of(&w, Kind::Infantry)[0];
    let st = w.obj_mut(id).unwrap().infantry.as_mut().unwrap();
    st.seq = Some(Seq::Stand);
    st.fear = fear;
    // No idle attempt during the run: only the flee rule could move it.
    st.idle_until = 1_000_000;
    (w, id)
}

// spec: S-0107 claim 15 (Flee row: every tick, a fraidycat not deploying, not falling, with no movement in progress and no destination, f′ > 50: scatters), S-0315 claim 2 (whatever the owner) and S-0208 Terms (the flee rule's request: forced, no threat point, not move-anywhere)
#[test]
fn a_resting_frightened_fraidycat_flees() {
    for human in [true, false] {
        // Control: the request the flee rule sends is accepted here.
        let (mut c, cid) = frightened(human, 300);
        let accepted = scatter::request(
            &mut c,
            cid,
            ScatterRequest {
                threat: None,
                forced: true,
                anywhere: false,
            },
        );
        assert!(accepted, "control: the request is accepted (human {human})");

        let (mut w, id) = frightened(human, 300);
        let start = w.obj(id).unwrap().pos;
        let mut moved = false;
        for _ in 0..40 {
            tick::step(&mut w);
            let o = w.obj(id).unwrap();
            moved |= o.dest.is_some() || o.pos != start;
        }
        assert!(moved, "the fraidycat flees (human {human})");
    }
}

// spec: S-0107 claim 15 (the rules test f′, after the tick's decay: a fraidycat whose level is 51 at the start of a tick does not scatter on that tick)
#[test]
fn the_flee_rule_tests_the_decayed_level() {
    for (fear, flees) in [(51, false), (52, true)] {
        let (mut w, id) = frightened(true, fear);
        tick::step(&mut w);
        let o = w.obj(id).unwrap();
        assert_eq!(o.infantry.as_ref().unwrap().fear, fear - 1);
        assert_eq!(o.dest.is_some(), flees, "start-of-tick level {fear}");
    }
}
