//! Claim test of S-0516 claim 22 (superweapon teleport, Q-0795): a warp
//! member's sensor disc stays at its old recorded cell through departure,
//! landing retries and appearance, and transfers, with the adjacent sensor
//! test, at the arrival-effects update. Synthetic rules (`tests/sw_common`).

mod sw_common;

use glasnost_rules::schema::Ruleset;
use glasnost_sim::cloak::CloakState;
use glasnost_sim::superweapons::warp::Stage;
use glasnost_sim::world::{ObjectId, World};
use sw_common::*;

fn rules() -> Ruleset {
    let mut r = ruleset();
    let g = &mut r.sim.general.superweapons;
    g.chrono_placement = Some("MARK".into());
    g.chrono_blast = Some("BLASTS".into());
    g.chrono_blast_dest = Some("BLASTD".into());
    g.chrono_delay = Some(7);
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
    // The warped vehicle is a sensor that can cloak by itself.
    let cart = r.sim.vehicles.get_mut("CART").unwrap();
    cart.combat.sensors = Some(true);
    cart.combat.sensors_sight = Some(2);
    cart.combat.cloakable = Some(true);
    cart.combat.cloaking_speed = Some(2);
    r
}

fn stage(w: &World, id: ObjectId) -> Option<Stage> {
    w.obj(id).and_then(|o| o.sw.warp.as_ref()).map(|m| m.stage)
}

// spec: S-0516 claim 22 (Superweapon teleport: the member's removal and appearance are not ordinary leaving-play and entering-play sensor changes; coverage stays at its old recorded cell through departure, landing retries and appearance; a surviving member's cell-arrival update comes at its later arrival-effects update, after the arrival-fate test and before the arrival hold, transferring the disc and applying the adjacency reveal)
#[test]
fn claim_22_warp_member_disc_transfers_at_the_arrival_effects() {
    let mut w = game(&rules());
    spawn(&mut w, "CHRONOB", ALPHA, -9, -9);
    ready_tick(&mut w, ALPHA, SOURCE, 300);
    let m = spawn(&mut w, "CART", ALPHA, 0, 0);
    // An enemy sensor next to the landing cell (5, 5).
    spawn(&mut w, "CART", BETA, 6, 5);
    let old = w.obj(m).unwrap().pos.cell();
    let new = at(5, 5);
    // Fully cloaked by its own flag, from the start.
    {
        let o = w.obj_mut(m).unwrap();
        o.cloak.state = CloakState::Cloaked;
        o.cloak.stage = 0;
    }
    assert_eq!(w.obj(m).unwrap().cloak.disc, Some((ALPHA, old)));
    fire_now(&mut w, ALPHA, SOURCE, at(0, 0));
    fire(&mut w, ALPHA, WARP, at(5, 5));
    let mut seen = Vec::new();
    for _ in 0..200 {
        step(&mut w);
        let st = stage(&w, m);
        let disc = w.obj(m).unwrap().cloak.disc;
        match st {
            Some(Stage::Hold(_)) | None => {
                seen.push(st);
                if matches!(st, Some(Stage::Hold(_))) {
                    break;
                }
            }
            _ => {
                assert_eq!(disc, Some((ALPHA, old)), "stage {st:?}: the disc stays");
                assert_eq!(
                    w.cloak.sensors.get(&(ALPHA.0, old)).copied().unwrap_or(0),
                    1,
                    "stage {st:?}"
                );
                assert_eq!(state_of(&w, m), CloakState::Cloaked, "stage {st:?}");
            }
        }
    }
    assert!(matches!(seen.last(), Some(Some(Stage::Hold(_)))), "it arrived");
    let o = w.obj(m).unwrap();
    assert_eq!(o.pos.cell(), new);
    assert_eq!(o.cloak.disc, Some((ALPHA, new)), "transferred at the arrival effects");
    assert_ne!(state_of(&w, m), CloakState::Cloaked, "the adjacency reveal");
}

fn state_of(w: &World, id: ObjectId) -> CloakState {
    glasnost_sim::cloak::state(w, id)
}
