//! Design C7-P5 yield (#300): an order that fails its checks, or one that
//! does not replace the unit's order (a route-point add), leaves a unit
//! giving way its saved move (S-0213 claim 4: a failed order has no effect
//! apart from ending the planned route). The synthetic scenario `rooms` of
//! `pathing_common` under the play-fixes layer.

mod pathing_common;

use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::Behaviour;
use glasnost_sim::world::{HouseId, ObjectId, Target, World};
use pathing_common::*;

fn arrived(w: &World, g: &Goal) -> bool {
    let c = w.obj(g.id).unwrap().pos.cell();
    (c.x - g.cell.x).abs().max((c.y - g.cell.y).abs()) <= g.radius
}

// spec: S-0213 claim 4 (a failed order has no effect apart from ending the planned route) and claim 17 (a route-point add extends the planned route) (#300)
// design: C7-P5 yield (an order that takes effect on the unit's destination ends its giving way)
#[test]
fn a_failed_order_leaves_a_unit_giving_way_its_move() {
    let mut s = rooms(Rules::Layer, 6, 7);
    let mut t = 0;
    let y = loop {
        if let Some(o) = s.w.objects.values().find(|o| o.path.yielding.is_some()) {
            break o.id;
        }
        glasnost_sim::tick::step(&mut s.w);
        t += 1;
        assert!(t < s.bound, "a unit gives way");
    };
    let goal = *s.goals.iter().find(|g| g.id == y).unwrap();

    // Control: no order.
    let mut a = s.w.clone();
    // An attack order whose target is no longer live when the order is applied.
    let mut b = s.w.clone();
    let ghost = ObjectId(u32::MAX - 7);
    assert!(b.obj(ghost).is_none());
    assert!(orders::issue(
        &mut b,
        HouseId(0),
        Order::Behaviour {
            actor: y,
            behaviour: Behaviour::Attack,
            attack_move: false,
            target: Some(Target::Object(ghost)),
            dest: None,
        }
    ));
    // A route-point add (it only extends the planned route).
    let mut c = s.w.clone();
    assert!(orders::issue(
        &mut c,
        HouseId(0),
        Order::RoutePoint {
            actor: y,
            behaviour: Behaviour::Move,
            attack_move: false,
            target: None,
            dest: Some(Target::Cell(goal.cell)),
            place: Some(Target::Cell(goal.cell)),
        }
    ));
    for _ in 0..3001 {
        glasnost_sim::tick::step(&mut a);
        glasnost_sim::tick::step(&mut b);
        glasnost_sim::tick::step(&mut c);
    }
    assert!(arrived(&a, &goal), "control: the unit resumes and arrives");
    assert!(
        arrived(&b, &goal),
        "failed order: the unit still resumes its move and arrives"
    );
    assert!(
        arrived(&c, &goal),
        "route point: the unit still resumes its move and arrives"
    );
}
