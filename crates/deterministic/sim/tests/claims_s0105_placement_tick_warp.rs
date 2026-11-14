//! Claim test for S-0105 r50 claim 6 (*When the turn starts*, the
//! chronosphere warp's exception), sim issues S15 (#81): a warp member's
//! appearance at its landing point keeps the tick of its last placement.
//! Synthetic rules (`tests/sw_common`); every name and number is invented.
//! Ledger: `docs/m8-claims/sim-issues.md`, *Batch S15*.

mod sw_common;

use glasnost_rules::schema::Ruleset;
use glasnost_sim::superweapons::warp::Stage;
use glasnost_sim::world::{ObjectId, World};
use sw_common::*;

fn rules() -> Ruleset {
    let mut r = ruleset();
    {
        let g = &mut r.sim.general.superweapons;
        g.chrono_placement = Some("MARK".into());
        g.chrono_blast = Some("BLASTS".into());
        g.chrono_blast_dest = Some("BLASTD".into());
        g.chrono_delay = Some(7);
    }
    r.sim.general.warp_out = Some("WARPA".into());
    for n in ["MARK", "BLASTS", "BLASTD", "WARPA"] {
        r.sim
            .animations
            .insert(n.into(), glasnost_rules::sim::AnimationType::default());
        r.registries
            .entry("animations".into())
            .or_default()
            .push(n.into());
    }
    r
}

fn stage(w: &World, id: ObjectId) -> Option<Stage> {
    w.obj(id).and_then(|o| o.sw.warp.as_ref()).map(|m| m.stage)
}

// spec: S-0105 claim 6 (When the turn starts: a chronosphere warp's member is off the map between its departure and its appearance at the landing point, but that appearance keeps the elevation; S-0513 claim 39)
#[test]
fn issue_81_a_warp_appearance_keeps_the_placement_tick() {
    let mut w = game(&rules());
    for _ in 0..5 {
        step(&mut w);
    }
    let m = spawn(&mut w, "CART", ALPHA, 0, 0);
    let t = w.obj(m).unwrap().placed_tick;
    spawn(&mut w, "CHRONOB", ALPHA, -9, -9);
    ready_tick(&mut w, ALPHA, SOURCE, 300);
    fire_now(&mut w, ALPHA, SOURCE, at(0, 0));
    fire(&mut w, ALPHA, WARP, at(5, 5));
    // Until the member has left the map and appeared again.
    let mut away = false;
    for _ in 0..300 {
        step(&mut w);
        let on = w.obj(m).is_some_and(|o| o.on_map);
        away |= !on;
        if away && on {
            break;
        }
    }
    assert!(away, "set-up: the member departed");
    assert!(stage(&w, m).is_some(), "set-up: still a warp member");
    let o = w.obj(m).expect("alive");
    assert!(o.on_map, "set-up: the member appeared");
    assert_eq!(o.pos.cell(), at(5, 5), "set-up: at its landing point");
    assert_eq!(o.placed_tick, t, "the appearance keeps it");
}
