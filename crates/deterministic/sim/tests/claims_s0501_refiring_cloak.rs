//! Claim test for S-0501 table T1's *refiring* row with the answer
//! *cloaked* (sim issues S11, #228): an aircraft that becomes fully cloaked
//! while it waits in *refiring* starts to uncloak there, and its run goes
//! on. Synthetic rules of `cloak_common`; every name and number is
//! invented. Ledger: `docs/m8-claims/sim-issues.md`, *Batch S11*.

mod cloak_common;

use cloak_common::*;
use glasnost_sim::air;
use glasnost_sim::cloak::{CloakState, machine};
use glasnost_sim::world::{ObjectId, World};

fn stage_of(w: &World, id: ObjectId) -> air::Stage {
    w.obj(id).unwrap().air.clone().unwrap_or_default().stage
}

// spec: S-0501 T1 (refiring, answer cloaked or rearming: uncloak when cloaked; standard delay); S-0516 claim 11 (a) (a firer that tries to fire and gets cloaked starts uncloaking)
#[test]
fn issue_228_refiring_uncloaks_a_cloaked_aircraft() {
    // A cloak-capable aircraft with a non-strafing weapon and a cloaking
    // speed slow enough that the cloak completes after its first burst.
    let mut r = ruleset();
    let mut jet = r.sim.aircraft.get("WRAITH").unwrap().clone();
    jet.primary = Some("Cannon".into());
    jet.combat.cloaking_speed = Some(14);
    r.sim.aircraft.insert("WRAITH".into(), jet);
    let mut victim = r.sim.vehicles.get("PLAIN").unwrap().clone();
    victim.strength = Some(5000);
    r.sim.vehicles.insert("PLAIN".into(), victim);
    let mut w = game(&r);
    let a = spawn(&mut w, "WRAITH", ALPHA, -6, 0);
    let v = spawn(&mut w, "PLAIN", BETA, 4, 0);
    machine::grant_own_flag(&mut w, a);
    // It starts cloaking on the ground and is ordered to attack at once.
    until(&mut w, 50, |w| state(w, a) == CloakState::Cloaking);
    assert_eq!(state(&w, a), CloakState::Cloaking);
    assert!(attack(&mut w, a, v), "attack refused");
    let mut seen = false;
    for _ in 0..400 {
        run(&mut w, 1);
        if stage_of(&w, a) == air::Stage::Refiring && state(&w, a) == CloakState::Cloaked {
            seen = true;
            break;
        }
    }
    assert!(seen, "never fully cloaked in refiring");
    let at = w.tick;
    // Let the first burst's projectile land and the weapon rearm.
    run(&mut w, 100);
    let health = w.obj(v).unwrap().health;
    assert!(health < 5000, "the first burst hit");
    run(&mut w, 600);
    assert!(
        state(&w, a) != CloakState::Cloaked || w.obj(v).unwrap().health < health,
        "700 ticks after becoming fully cloaked in refiring at tick {at}: stage {:?}, cloak {:?}, target health {} (was {health} 600 ticks ago)",
        stage_of(&w, a),
        state(&w, a),
        w.obj(v).unwrap().health,
    );
}
