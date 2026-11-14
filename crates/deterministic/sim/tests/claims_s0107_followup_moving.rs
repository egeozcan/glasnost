//! S-0107 State and ordering: "not moving" in the completion follow-up
//! table means no movement in progress, or a speed fraction of at most 0.1
//! (#275). Synthetic type and map.

mod common;

use common::{Scene, cell, unit_type};
use glasnost_sim::infantry::Seq;
use glasnost_sim::occupancy;
use glasnost_sim::rules::Kind;

// spec: S-0107 State and ordering ("Not moving" in the follow-up table: no movement in progress, or a speed fraction of at most 0.1; a completed kind while not moving and not prone is followed by stand, forced) and Constants (moving test at completion 0.1); claim 7 (walk becomes panic above fear 199)
#[test]
fn issue_275_completed_panic_with_movement_in_progress_and_speed_fraction_0_is_followed_by_stand() {
    let mut s = Scene::new();
    let mut t = unit_type(common::FOOT, 4, 5);
    for k in ["stand", "walk", "panic"] {
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
    s.infantry("SOLDIER", t);
    s.place_infantry(0, "SOLDIER", cell(0, 0), 0);
    s.setup.houses[0].human = true;
    let mut w = s.build();
    common::open_orders(&mut w);
    let id = common::objects_of(&w, Kind::Infantry)[0];
    common::order_move(&mut w, id, cell(8, 0));
    common::run(&mut w, 1);
    let state = |w: &glasnost_sim::world::World| {
        let o = w.obj(id).unwrap();
        (
            occupancy::in_progress(w, id),
            o.walk.as_ref().is_some_and(|s| s.fraction),
            o.infantry.as_ref().unwrap().seq,
        )
    };
    // The destination has reached the walker; it has not stepped yet.
    let before = state(&w);
    assert!(before.0 && !before.1, "in progress, fraction 0: {before:?}");
    // A frightened unit whose panic sequence is complete (stage 25 of 4).
    let now = w.tick;
    {
        let st = w.obj_mut(id).unwrap().infantry.as_mut().unwrap();
        st.seq = Some(Seq::Panic);
        st.seq_t0 = now - 100;
        st.seq_delay = 4;
        st.fear = 300;
    }
    common::run(&mut w, 1);
    let after = state(&w);
    // Still no step on this tick: movement in progress, speed fraction 0.
    assert!(after.0 && !after.1, "in progress, fraction 0: {after:?}");
    assert_eq!(after.2, Some(Seq::Stand), "not moving: stand (forced)");
}
