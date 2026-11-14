//! S-0517 claim 16 with S-0309 claim 24 on the synthetic deploy fixture:
//! a target the deployed-firing search finds is not an opportunity target
//! (sim issues S12, issue 246).

mod deploy_common;

use deploy_common::*;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::world::{Target, World};

fn until(w: &mut World, max: i32, f: impl Fn(&World) -> bool) -> bool {
    for _ in 0..max {
        if f(w) {
            return true;
        }
        step(w);
    }
    f(w)
}

// spec: S-0309 claim 24 (only the per-tick range search of claim 1's first row and the teleport search mark an opportunity target; no other search marks one), claim 1 (the deployed-firing search, the fifth row) and S-0517 claim 16 (the deployed-firing rule's search)
#[test]
fn claim_16_deployed_firing_search_target_is_not_an_opportunity_target() {
    let mut w = game();
    let i = put(&mut w, "DEPINF", Kind::Infantry, 0, 0, 0);
    deploy(&mut w, i);
    assert!(
        until(&mut w, 120, |w| {
            w.obj(i)
                .unwrap()
                .infantry
                .as_ref()
                .and_then(|s| s.seq)
                .is_some_and(glasnost_sim::infantry::Seq::is_deploy)
        }),
        "the infantry deployed"
    );
    let near = put(&mut w, "TANK", Kind::Vehicle, 1, 3, 0);
    let far = put(&mut w, "TANK", Kind::Vehicle, 1, 14, 0);
    let tick = w.tick;
    let o = w.obj_mut(i).unwrap();
    o.target = Some(Target::Object(far));
    o.combat.opportunity = false;
    o.beh.force(Behaviour::Attack, tick, true);
    // Step until the attack processing's deployed-firing rule replaces the
    // out-of-range target. The per-tick search of claim 1's first row never
    // runs in attack, so the setter is the deployed-firing search.
    let mut set = false;
    for _ in 0..80 {
        step(&mut w);
        let ob = w.obj(i).unwrap();
        if ob.target == Some(Target::Object(near)) {
            assert_eq!(
                ob.beh.current,
                Some(Behaviour::Attack),
                "still in attack when the target was set"
            );
            set = true;
            break;
        }
    }
    assert!(set, "the deployed-firing search found the near target");
    assert!(
        !w.obj(i).unwrap().combat.opportunity,
        "a target set by the deployed-firing search is not an opportunity target"
    );
}
