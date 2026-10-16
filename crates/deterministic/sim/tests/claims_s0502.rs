//! Claim tests of S-0502 (movement of jumpjets) on synthetic rules.

mod jump_common;

use glasnost_sim::jump::Phase;
use glasnost_sim::rules::Kind;
use jump_common::*;

#[test]
fn trace() {
    let mut w = game();
    let u = put(&mut w, "JJ", Kind::Infantry, 0, -8, 0);
    go(&mut w, u, at(2, 0));
    let mut last = Phase::Cruising;
    for _ in 0..500 {
        step(&mut w);
        let p = pos(&w, u);
        if phase(&w, u) != last || w.tick % 20 == 0 {
            last = phase(&w, u);
            println!(
                "tick {} {:?} pos ({}, {}, {}) h {} v {:?} ",
                w.tick,
                phase(&w, u),
                p.x,
                p.y,
                p.z,
                height(&w, u),
                w.obj(u).unwrap().jump.as_ref().unwrap().vc
            );
        }
    }
    assert_eq!(phase(&w, u), Phase::Grounded);
}
