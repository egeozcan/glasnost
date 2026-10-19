//! The fire order (S-0512 claims 8, 10, 11 and 19; S-0513 claims 2 and
//! 3) and the dispatch of a launch to its kind's effect.

use glasnost_map::CellXY;
use glasnost_num::F64;

use super::{DrainState, SwEvent, SwKind, record, record_mut};
use crate::world::{HouseId, World};

/// The type a fire order acts on (S-0513 claim 2).
fn resolve(w: &World, sw: u32) -> u32 {
    let r = w.rules();
    let types = &r.superweapons.types;
    if !r.quirks.superweapon_order_first_type_of_action {
        // spec: S-0513 claim 2 (quirk off: the order names the selected type)
        return sw;
    }
    let Some(action) = types.get(sw as usize).and_then(|t| t.action.as_ref()) else {
        return sw;
    };
    // spec: S-0513 claim 2 (quirks.superweapon_order_first_type_of_action: the first type in list order whose aim action is the action under the cursor)
    types
        .iter()
        .position(|t| t.action.as_ref() == Some(action))
        .map_or(sw, |p| p as u32)
}

/// Applies a fire order of house `h` (S-0512 claim 8).
pub fn fire(w: &mut World, h: HouseId, sw: u32, cell: CellXY) {
    let n = w.rules().superweapons.types.len() as u32;
    if sw >= n {
        return;
    }
    let i = resolve(w, sw);
    let ty = w.rules().superweapons.types[i as usize].clone();
    super::house_mut(w, h);
    if ty.post_click {
        // spec: S-0512 claim 8 (second stages follow claim 10)
        second_stage(w, h, i, cell);
        return;
    }
    if ty.drain {
        // spec: S-0512 claim 8 (drain-mode superweapons follow claim 19)
        drain(w, h, i, cell);
        return;
    }
    let Some(rec) = record(w, h, i).cloned() else {
        return;
    };
    let clock_ok = if w.rules().quirks.superweapon_order_tests_clock {
        // spec: S-0512 claim 8 (quirks.superweapon_order_tests_clock: granted, ready and the clock running)
        rec.running
    } else {
        // spec: S-0512 claim 8 (quirk off: granted, ready and not suspended)
        !rec.suspended
    };
    // spec: S-0512 claim 8 and S-0513 claim 3 (otherwise the order does nothing)
    if !(rec.granted && rec.ready && clock_ok) {
        return;
    }
    if ty.kind == Some(SwKind::Storm) && w.sw.storm.state != super::StormState::Idle {
        // spec: S-0512 claim 8 and S-0513 claim 14 (refused while a countdown runs or the storm is raging or clearing: the charge and the ready state are kept; a refusal message for its own player)
        super::event(w, SwEvent::Refused, Some(h), Some(i), None);
        return;
    }
    let mut silo = None;
    if ty.kind == Some(SwKind::Missile) && !rec.one_time {
        let choice = super::nuke::silo_choice(w, h, i);
        let accepts = choice.is_some_and(|s| super::nuke::accepts(w, s));
        if !accepts && !w.rules().quirks.missile_without_launcher_consumes_charge {
            // spec: S-0512 claim 8 (quirks.missile_without_launcher_consumes_charge off: an order that would produce no new launch sequence is refused; the record stays ready)
            return;
        }
        silo = Some(choice);
    }
    if ty.kind == Some(SwKind::Missile) {
        // spec: S-0407 claim 12 (j) (the house launches a superweapon of the missile kind)
        crate::econ::tech::recheck_due(w, h);
    }
    consume(w, h, i, &ty, &rec, cell);
    // spec: S-0513 claim 3 (the launch requests the kind's effect)
    effect(w, h, i, cell, rec.one_time, silo);
}

/// The consumption of a carried-out order (S-0512 claims 8, 10, 12 and
/// 20).
fn consume(
    w: &mut World,
    h: HouseId,
    i: u32,
    ty: &super::SwType,
    rec: &super::Record,
    cell: CellXY,
) {
    let t = w.tick;
    if rec.one_time {
        // spec: S-0512 claim 12 (a one-time record stops being granted and ready when it fires; the icon goes at once)
        if let Some(r) = record_mut(w, h, i) {
            r.granted = false;
            r.ready = false;
            r.one_time = false;
            r.running = false;
            if ty.pre_click {
                r.source = Some(cell);
                r.source_fresh = true;
            }
        }
        return;
    }
    let Some(r) = record_mut(w, h, i) else { return };
    if ty.pre_click {
        // spec: S-0512 claim 10 (a first stage picks the source and does not consume the charge: the remaining time is kept and readiness returns at the next eligible completion)
        r.ready = false;
        r.silent = true;
        r.source = Some(cell);
        r.source_fresh = true;
    } else if ty.manual {
        // spec: S-0512 claim 20 (after it fires its clock stops again at the full T)
        r.ready = false;
        r.restart(t, ty.charge, false);
    } else if r.suspended && r.running {
        // spec: S-0512 claim 8 (a firing through the suspended-but-running bypass consumes readiness but does not restart the clock)
        r.ready = false;
    } else {
        // spec: S-0512 claim 8 (the record stops being ready and its clock restarts with remaining time T)
        r.ready = false;
        r.restart(t, ty.charge, true);
    }
}

/// The first stage a second-stage type links to (S-0512 claim 11).
fn linked_first(w: &World, ty: &super::SwType) -> Option<u32> {
    let n = w.rules().superweapons.types.len() as u32;
    ty.pre_dependent.and_then(|k| {
        if w.rules().quirks.predependent_kind_as_list_position {
            // spec: S-0512 claim 11 (quirks.predependent_kind_as_list_position: the kind number as a list position, whatever type stands there)
            (k < n).then_some(k)
        } else {
            // spec: S-0512 claim 11 (quirk off: the first type in the list of the named kind)
            SwKind::from_number(k).and_then(|k| w.rules().superweapons.first_of_kind(k))
        }
    })
}

/// The second stage's order (S-0512 claims 10 and 11).
fn second_stage(w: &mut World, h: HouseId, i: u32, cell: CellXY) {
    let t = w.tick;
    let ty = w.rules().superweapons.types[i as usize].clone();
    let first = linked_first(w, &ty);
    let Some(first) = first else {
        // spec: S-0512 claim 11 (a link outside the type list: our engine refuses the second-stage order, a departure)
        w.note(
            "S-0512 claim 11: a second-stage order whose link names no type in the list is refused",
        );
        return;
    };
    let Some(frec) = record(w, h, first).cloned() else {
        return;
    };
    if !w.rules().quirks.second_stage_order_unchecked
        && !(frec.granted && !frec.suspended && frec.source_fresh)
    {
        // spec: S-0512 claim 10 (quirks.second_stage_order_unchecked off: refused unless the first stage is granted, not suspended and has an unconsumed source selection)
        return;
    }
    // spec: S-0512 claim 10 (before any source has been picked the source is cell (0, 0), not an absent selection; the warp uses it and its effect block even outside the playable area; with the unchecked-order quirk off the order was refused above)
    let source = frec.source.unwrap_or(CellXY::new(0, 0));
    // spec: S-0512 claim 10 (it starts the second effect stage with the first stage's last picked source)
    super::warp::launch(w, h, source, cell);
    let ignores = w.rules().quirks.superweapon_restart_ignores_suspension;
    let ft = w.rules().superweapons.types[first as usize].charge;
    if let Some(r) = record_mut(w, h, first)
        && r.granted
    {
        // spec: S-0512 claim 10 (the first stage becomes not ready with its clock restarted at T from that tick; quirks.superweapon_restart_ignores_suspension: running even when suspended)
        r.ready = false;
        r.silent = false;
        let running = !r.suspended || ignores;
        r.restart(t, ft, running);
        r.source_fresh = false;
    }
    if let Some(r) = record_mut(w, h, i)
        && r.one_time
        && r.granted
    {
        // spec: S-0512 claim 10 (a trigger-granted one-time second stage is consumed when fired, as in claim 12)
        r.granted = false;
        r.ready = false;
        r.one_time = false;
        r.running = false;
    }
}

/// A drain-mode order (S-0512 claim 19, F5).
fn drain(w: &mut World, h: HouseId, i: u32, cell: CellXY) {
    let t = w.tick;
    let mode = w.rules().mode;
    let ty = w.rules().superweapons.types[i as usize].clone();
    let rho = w.rules().superweapons.ratio;
    let bypass = w.rules().quirks.drain_orders_bypass_normal_checks;
    let Some(rec) = record(w, h, i).cloned() else {
        return;
    };
    if !bypass && !(rec.granted && !rec.suspended) {
        // spec: S-0512 claim 19 (quirks.drain_orders_bypass_normal_checks off: granted and not suspended)
        return;
    }
    let r = rec.remaining(t);
    match rec.drain {
        DrainState::Ready => {
            // spec: S-0512 F5 (active time trunc(Z((T − r) × ρ)))
            let active = F64::from_i64(i64::from(ty.charge - r), mode)
                .mul(rho, mode)
                .to_i32_common();
            if let Some(x) = record_mut(w, h, i) {
                // spec: S-0512 claim 19 (an order in ready starts the launch effect and moves to active, the clock running for the active time, even if suspended)
                x.drain = DrainState::Active;
                x.ready = false;
                x.restart(t, active, true);
            }
            if !bypass && rec.one_time {
                // spec: S-0512 claim 19 (quirk off: a one-time record is consumed on launch)
                if let Some(x) = record_mut(w, h, i) {
                    x.granted = false;
                    x.ready = false;
                    x.one_time = false;
                    x.running = false;
                    x.drain = DrainState::Charging;
                }
            }
            effect(w, h, i, cell, rec.one_time, None);
        }
        DrainState::Active => {
            // spec: S-0512 F5 (recharge time trunc(Z(T − Z(r / ρ))))
            let back = F64::from_i64(i64::from(ty.charge), mode)
                .sub(F64::from_i64(i64::from(r), mode).div(rho, mode), mode)
                .to_i32_common();
            if let Some(x) = record_mut(w, h, i) {
                // spec: S-0512 claim 19 (an order in active moves to ready with the clock running for the recharge time; no effect and no effect-ending action)
                x.drain = DrainState::Ready;
                x.ready = true;
                x.restart(t, back, true);
            }
        }
        // spec: S-0512 claim 19 (an order in charging does nothing)
        DrainState::Charging => {}
    }
}

/// Starts the kind's effect of a launch (S-0513 claim 3).
fn effect(
    w: &mut World,
    h: HouseId,
    i: u32,
    cell: CellXY,
    one_time: bool,
    silo: Option<Option<crate::world::ObjectId>>,
) {
    let kind = w.rules().superweapons.types[i as usize].kind;
    match kind {
        Some(SwKind::Missile) => {
            if one_time {
                // spec: S-0513 claim 10 (a one-time record does not use a building)
                super::nuke::one_time(w, Some(h), i, cell);
            } else {
                super::nuke::silo_launch(w, h, i, cell, silo.flatten());
            }
        }
        Some(SwKind::IronCurtain) => {
            // spec: S-0513 claim 5 (invulnerability: activation announcement and a radar event at the target cell)
            super::event(w, SwEvent::Launch, Some(h), Some(i), None);
            super::event(
                w,
                SwEvent::Radar,
                Some(h),
                Some(i),
                Some(super::cell_point(w, cell, false)),
            );
            super::protect::curtain(w, Some(h), cell, false);
        }
        Some(SwKind::Storm) => {
            // spec: S-0513 claim 14 (the weather launch plays the storm-created announcement to every player)
            super::event(w, SwEvent::Launch, Some(h), Some(i), None);
            let (dur, def) = {
                let s = &w.rules().superweapons.storm;
                (s.duration, s.deferment)
            };
            super::storm::request(w, Some(h), cell, dur, def);
        }
        Some(SwKind::Chrono) => super::warp::pick_source(w, h, i, cell),
        Some(SwKind::ChronoWarp) => {
            // A warp-kind type fired without the second-stage flag: the
            // warp from the linked first stage's source, as claim 34 says.
            let ty = w.rules().superweapons.types[i as usize].clone();
            let first = linked_first(w, &ty);
            // spec: S-0512 claim 10 (no source picked yet: cell (0, 0))
            let source = first.map(|f| {
                record(w, h, f)
                    .and_then(|r| r.source)
                    .unwrap_or(CellXY::new(0, 0))
            });
            if let Some(s) = source {
                super::warp::launch(w, h, s, cell);
            }
        }
        Some(SwKind::Paradrop) => {
            super::planes::paradrop(w, h, cell, false);
        }
        Some(SwKind::AmerParadrop) => {
            super::planes::paradrop(w, h, cell, true);
        }
        Some(SwKind::SpyPlane) => {
            super::planes::spy_plane(w, h, cell);
        }
        Some(SwKind::ForceShield) => super::protect::force_shield(w, h, i, cell),
        // spec: S-0610 claims 13 to 15 (the psychic reveal: the area reveal, the activation sound; no launch announcement and no radar event)
        Some(SwKind::PsychicReveal) => crate::psychic::reveal(w, h, i, cell),
        // spec: S-0611 claims 2 to 4 (the genetic mutation launch: animation, announcement, sound, radar event, then the effect)
        Some(SwKind::Genetic) => crate::mutation::launch(w, h, i, cell),
        Some(k @ SwKind::Dominator) => {
            yuri_seam(w, h, k, cell);
        }
        // A type with no kind launches nothing.
        None => {}
    }
}

/// Seam: S-0612 (psychic domination) is not built (M7b). The launch still
/// counts (the charge is consumed) and nothing happens.
pub fn yuri_seam(w: &mut World, h: HouseId, k: SwKind, cell: CellXY) {
    let _ = (h, cell);
    w.note(format!(
        "superweapon kind {k:?} is not built (S-0612): the launch does nothing"
    ));
}
