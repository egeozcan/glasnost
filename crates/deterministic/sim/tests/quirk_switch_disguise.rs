//! Both positions of `quirks.permadisguise_attack_cursor` (#327), on the
//! synthetic rules of `tests/cloak_common` (SPY disguises permanently,
//! COPIER can disguise but not permanently, CART is an armed vehicle). The
//! test is the one given in #327.

mod cloak_common;

use cloak_common::*;
use glasnost_sim::cloak::{Disguise, DisguiseAs, appears_disguised, disguise};
use glasnost_sim::rules::Kind;
use glasnost_sim::targeting::{Action, ClickOn, Modifiers, cursor};

// spec: S-0214 claim 6 and Edge cases (quirks.permadisguise_attack_cursor: off, a permanently disguised target is treated like any other disguised one)
#[test]
fn permadisguise_attack_cursor_both_positions() {
    let mut seen = Vec::new();
    for quirk in [true, false] {
        let mut r = ruleset();
        r.sim.general.attack_cursor_on_disguise = Some(false);
        r.globals.quirks.permadisguise_attack_cursor = quirk;
        let mut w = game(&r);
        let gun = spawn(&mut w, "CART", ALPHA, -4, 0);
        let spy = spawn(&mut w, "SPY", BETA, 0, 0);
        let other = spawn(&mut w, "COPIER", BETA, 0, 2);
        let walker = w.rules().index_of(Kind::Infantry, "WALKER").unwrap();
        let t = w.tick;
        for id in [spy, other] {
            w.obj_mut(id).unwrap().cloak.disguise = Some(Disguise {
                what: Some(DisguiseAs::Infantry(walker)),
                house: Some(ALPHA),
                since: t,
            });
            assert!(appears_disguised(&w, id, Some(ALPHA)));
        }
        assert!(disguise::perma(&w, spy));
        assert!(!disguise::perma(&w, other));
        let plain = |id| cursor(&w, ALPHA, &[gun], ClickOn::Object(id), Modifiers::default());
        seen.push((quirk, plain(spy), plain(other)));
    }
    let (_, spy_on, other_on) = seen[0];
    let (_, spy_off, other_off) = seen[1];
    assert_eq!(spy_on, Action::Attack, "switch on: the permanent disguiser passes");
    assert_ne!(other_on, Action::Attack, "an ordinary disguised target fails");
    assert_eq!(other_on, other_off);
    assert_eq!(
        spy_off, other_off,
        "switch off: the permanent disguiser is treated like any other disguised target: {seen:?}"
    );
}
