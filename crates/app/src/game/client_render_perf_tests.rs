//! Guards of the render perf batch RP3's changes to the game's drawing,
//! through the windowless client on synthetic data (invented rules and
//! map, `glasnost_ai::fixture`). The specs cited are the ones the drawing
//! implements; the rest is our own design, named so where it is tested.

use std::sync::Arc;

use super::access_tests::{client, own, session_mut, start_by_keyboard_with};
use super::*;
use glasnost_ai::fixture;
use glasnost_map::CellXY;
use glasnost_sim::mind::Victim;
use glasnost_sim::rules::{Kind, SimRules};
use glasnost_sim::testkit;
use glasnost_sim::world::ObjectId;
use glasnost_ui::draw::Prim;

/// The fixture's rules with invented control and permanent control ring
/// animation types, so that `glasnost_sim::mind::ring` gives rings.
fn ring_rules() -> Arc<SimRules> {
    let mut r = fixture::ruleset();
    r.sim.general.controlled_animation_type = Some("TESTRING".into());
    r.sim.general.perma_controlled_animation_type = Some("TESTPERMA".into());
    Arc::new(SimRules::new(&r, "synthetic-skirmish"))
}

/// The stand-in ring markers' inks (our own, `session.rs`).
const RING: [u8; 4] = [200, 80, 230, 255];
const PERMA: [u8; 4] = [255, 120, 255, 255];

/// The inks of the ring-marker lines in `d`, in drawing order.
fn ring_inks(d: &DrawList) -> Vec<[u8; 4]> {
    let mut out = Vec::new();
    for p in &d.prims {
        match p {
            Prim::Line { rgba, .. } if *rgba == RING || *rgba == PERMA => out.push(*rgba),
            Prim::Layer { list, .. } => out.extend(ring_inks(list)),
            _ => {}
        }
    }
    out
}

/// spec: S-0601 claim 18 (the control ring on a controlled object, the
/// permanent ring on a permanently controlled one) and design: render
/// (#122) (the rings are found from the victims and the permanently
/// controlled objects, not from every object): each ring marker in object
/// order, the permanent one where an object is both, none for an object
/// off the map or a record whose object is gone.
#[test]
fn render_perf_122_the_rings_in_object_order() {
    let mut c = client((800, 600), 100);
    start_by_keyboard_with(&mut c, ring_rules());
    let mcv = own(&c, Kind::Vehicle)[0];
    let s = session_mut(&mut c);
    let me = s.play.me();
    let at = s.play.world().obj(mcv).unwrap().pos;
    glasnost_sim::econ::shroud::area_reveal(&mut s.play.game.world, me, at, 8);
    let base = at.cell();
    let w = &mut s.play.game.world;
    let units: Vec<ObjectId> = (1..=4)
        .map(|dx| {
            let cell = CellXY::new(base.x + dx, base.y + 1);
            testkit::spawn(w, Kind::Vehicle, "MINR", me.0, cell, 0)
        })
        .collect();
    let [a, b, both, off] = units[..] else {
        panic!("four units");
    };
    assert!(a < b && b < both, "spawned in id order");
    let victim = Victim {
        controller: mcv,
        original_owner: None,
        line_until: 0,
    };
    for id in [both, a, off, ObjectId(999_999)] {
        w.mind.victims.insert(id, victim);
    }
    w.mind.permanent.insert(both);
    w.mind.permanent.insert(b);
    w.obj_mut(off).unwrap().on_map = false;
    let d = s.draw(DrawList::new());
    let mut want = vec![RING; 8];
    want.extend([PERMA; 16]);
    assert_eq!(ring_inks(&d), want);
}
