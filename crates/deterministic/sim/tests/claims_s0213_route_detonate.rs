//! Claim tests for S-0213 claim 16b's detonate click in route mode, on
//! the synthetic S-0519 fixtures (an invented bomb planter and carrier).
//! No stock values.

#[path = "terror_common/mod.rs"]
mod terror_common;

use glasnost_sim::orders::Order;
use glasnost_sim::routes::{self, HOLDER_LIMIT, RouteClick};
use glasnost_sim::rules::Kind;
use glasnost_sim::targeting::{self, Action, ClickOn, Modifiers};
use glasnost_sim::world::{HouseId, ObjectId, World};
use terror_common::*;

const A: HouseId = HouseId(0);

/// An Alpha vehicle that carries a bomb Alpha's planter set on it.
fn carrier(w: &mut World) -> ObjectId {
    let v = quiet(w, Kind::Vehicle, "VICT", 1, at(0, 0));
    let p = quiet(w, Kind::Infantry, "BOMBER", 0, at(-8, -8));
    assert!(shoot(w, p, v));
    w.obj_mut(v).unwrap().owner = Some(A);
    assert!(w.obj(v).unwrap().combat.bomb.is_some(), "planted");
    v
}

/// spec: S-0213 claim 16b (a route-mode click that would order a detonate adds no point; when claim 16d's tests pass, the detonate order is sent at the release, as outside route mode; when one refuses, the click is refused with its message and no detonate order is sent) and claim 16d (a detonate click meets both release-time tests); Scenario O8 part v
#[test]
fn claim_16b_a_detonate_click_sends_the_detonate_order() {
    let mut w = world(|_| {});
    let v = carrier(&mut w);
    let on = ClickOn::Object(v);
    assert_eq!(
        targeting::cursor(&w, A, &[v], on, Modifiers::default()),
        Action::Detonate
    );
    // (1) The detonate order, and no point.
    assert_eq!(
        routes::click(&w, A, &[v], on, Modifiers::default()),
        RouteClick::Orders(vec![Order::Detonate { actor: v }])
    );
    assert!(w.routes.points.is_empty());
    // (2) At the holder limit, with V holding no route: refused, no order.
    w.routes.counts.insert(A, HOLDER_LIMIT);
    assert_eq!(
        routes::click(&w, A, &[v], on, Modifiers::default()),
        RouteClick::Refused("MSG:PlannerMaximum")
    );
}
