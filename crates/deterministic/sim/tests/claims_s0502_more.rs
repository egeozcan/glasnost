//! Claim tests of S-0502 (movement of jumpjets): the bridge and sight cases.

mod jump_common;

use glasnost_sim::jump::{self, JumpState, Phase};
use glasnost_sim::rules::Kind;
use glasnost_sim::testkit;
use glasnost_sim::world::{ObjectId, World};
use jump_common::*;

fn j(w: &World, id: ObjectId) -> &JumpState {
    w.obj(id).unwrap().jump.as_ref().unwrap()
}

fn quirk(w: &mut World, f: impl FnOnce(&mut glasnost_rules::quirks::Quirks)) {
    let mut rules = (*w.rules_arc()).clone();
    f(&mut rules.quirks);
    w.rules = Some(std::sync::Arc::new(rules));
}

// spec: S-0502 claim 7, Edge cases and quirks.jumpjet_bridge_early_cruise_height (below a high bridge, at or above deck height while not on the deck: the take-off threshold uses h = r − 416; an own-cell early cruise retains r with the switch on, h with it off)
#[test]
fn claim_7_bridge_early_cruise_height() {
    for (on, delta) in [(true, 0), (false, 416)] {
        let mut w = game();
        quirk(&mut w, |q| q.jumpjet_bridge_early_cruise_height = on);
        let c = at(0, 0);
        let u = put(&mut w, "JVTALL", Kind::Vehicle, 0, 0, 0);
        let i = w.cells.idx(c).unwrap();
        w.cells.deck[i] = true;
        go_here_entering(&mut w, u);
        let mut early = None;
        for _ in 0..400 {
            step(&mut w);
            if early.is_none() && phase(&w, u) == Phase::Cruising {
                early = Some((height(&w, u), j(&w, u).ht));
            }
        }
        let (r, ht) = early.expect("it cruises early");
        // The threshold is a quarter of 2000: past it, h = r − 416 exceeds 500, so r is above 916.
        assert!(r > 916 && r <= 921, "{r}");
        assert_eq!(ht, r - delta, "quirk {on}");
    }
}

// spec: S-0502 claim 15 (a completed touchdown refreshes the jumpjet's sight at its new position)
#[test]
fn claim_15_touchdown_refreshes_sight() {
    let mut w = game();
    let u = put(&mut w, "JJ", Kind::Infantry, 0, -8, 0);
    testkit::run(&mut w, 3);
    let start = w.obj(u).unwrap().econ.sight.cell;
    assert!(start.is_some(), "it has sight on the ground");
    go(&mut w, u, at(1, 0));
    assert!(until(&mut w, 400, |w| phase(w, u) == Phase::Grounded
        && w.tick > 20));
    let landed = jump_common::pos(&w, u).cell();
    assert_eq!(
        w.obj(u).unwrap().econ.sight.cell,
        Some(landed),
        "sight at the landing cell"
    );
    assert_ne!(start, Some(landed));
    let _ = jump::moving(&w, u);
}
