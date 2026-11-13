//! Superweapon charge and readiness (S-0512): grants, suspension, loss,
//! completion, the trigger actions on records and the sidebar figures.

use glasnost_num::F64;

use super::{DrainState, Record, house_mut, record, record_mut};
use crate::rules::Kind;
use crate::world::{HouseId, ObjectId, World};

/// The giving buildings of a house for type `i` (S-0512 Terms), leaving out
/// `except` (a building being removed at this moment).
pub fn giving(w: &World, h: HouseId, i: u32, except: Option<ObjectId>) -> Vec<ObjectId> {
    let r = w.rules();
    let sw = &r.superweapons;
    let Some(t) = sw.types.get(i as usize) else {
        return Vec::new();
    };
    let ours = |o: &crate::world::Object| {
        o.kind == Kind::Building && o.owner == Some(h) && o.on_map && o.health > 0
    };
    // spec: S-0512 Terms (giving building: the type has no auxiliary building, or the house owns at least one building of it)
    let aux_ok = t.aux.is_none_or(|a| {
        w.objects
            .values()
            .any(|o| ours(o) && o.type_index == a && Some(o.id) != except)
    });
    w.objects
        .values()
        .filter(|o| ours(o) && Some(o.id) != except)
        .filter(|o| {
            let fields = sw.building_sw.get(o.type_index as usize);
            // spec: S-0512 Terms (its type's first or second superweapon field names the type, with the auxiliary condition)
            let own = aux_ok && fields.is_some_and(|f| f.contains(&Some(i)));
            // spec: S-0512 Terms (an upgrade it carries names the type in either field; the auxiliary condition does not apply)
            let upgrade = o.econ.building.held().any(|u| {
                sw.building_sw
                    .get(u as usize)
                    .is_some_and(|f| f.contains(&Some(i)))
            });
            own || upgrade
        })
        .map(|o| o.id)
        .collect()
}

fn defeated(w: &World, h: HouseId) -> bool {
    crate::econ::victory::defeated(w, h)
}

/// Whether the superweapon setting bars a type (S-0512 claims 3 and 7,
/// S-0412 claim 9 b).
fn barred(w: &World, i: u32) -> bool {
    let d = w
        .rules()
        .superweapons
        .types
        .get(i as usize)
        .is_some_and(|t| t.disableable);
    d && !crate::econ::setup::superweapons_allowed(w)
}

fn type_count(w: &World) -> u32 {
    w.rules().superweapons.types.len() as u32
}

/// Assigns the next grant number (claim 18's row order).
fn next_seq(w: &mut World) -> u64 {
    w.sw.seq += 1;
    w.sw.seq
}

/// The check for suspension, resumption and loss (claims 6, 7 and 22), at
/// the moments of claim 2 (a), (b) and (d). `except` is a building being
/// removed at this moment.
pub fn availability(w: &mut World, h: HouseId, except: Option<ObjectId>) {
    let t = w.tick;
    let gone = defeated(w, h);
    let low = crate::econ::power::low_power(w, h);
    let mut due = false;
    for i in 0..type_count(w) {
        let Some(rec) = record(w, h, i).cloned() else {
            continue;
        };
        if !rec.granted {
            continue;
        }
        if gone {
            // spec: S-0512 claim 22 (a defeated house loses every granted record, one-time and permanent ones included, at its next availability check)
            lose(w, h, i);
            due = true;
            continue;
        }
        // spec: S-0512 claims 6 and 7 (one-time records and permanent trigger grants are not judged)
        if rec.one_time || rec.permanent {
            continue;
        }
        let giving = giving(w, h, i, except);
        if giving.is_empty() || barred(w, i) {
            // spec: S-0512 claim 7 (no giving building, or a disableable type with the setting off: lost)
            lose(w, h, i);
            due = true;
            continue;
        }
        // spec: S-0512 claim 6 (powered: at least one giving building switched on and the house without low power)
        let powered = !low
            && giving
                .iter()
                .any(|&b| w.obj(b).is_some_and(|o| !o.econ.building.switched_off));
        let (ty_powered, ty_manual) = {
            let ty = &w.rules().superweapons.types[i as usize];
            (ty.powered, ty.manual)
        };
        let Some(r) = record_mut(w, h, i) else {
            continue;
        };
        if !powered && ty_powered && !r.suspended {
            // spec: S-0512 claim 6 (not powered, power-dependent, not yet suspended: suspended; the clock stops and keeps its remaining time)
            r.suspended = true;
            r.stop(t);
            due = true;
        } else if powered && r.suspended {
            // spec: S-0512 claim 6 (powered and suspended: resumed; an ordinary clock runs again from the time it kept, a manual-charge clock stays stopped)
            r.suspended = false;
            if !r.running && !ty_manual {
                r.start = t;
                r.running = true;
            }
            due = true;
        }
    }
    if due {
        // spec: S-0512 claims 6 and 7 (a suspension, resumption or loss makes a recheck due, S-0407 claim 12 (k))
        crate::econ::tech::recheck_due(w, h);
    }
}

/// Loses a record (claim 7): no longer granted or ready; a suspended record
/// stays marked suspended; the charge is forgotten.
fn lose(w: &mut World, h: HouseId, i: u32) {
    if let Some(r) = record_mut(w, h, i) {
        // spec: S-0512 claim 7 (no longer granted or ready; suspension stays marked; the charge is forgotten)
        r.granted = false;
        r.ready = false;
        r.one_time = false;
        r.permanent = false;
        r.running = false;
        r.silent = false;
        r.drain = DrainState::Charging;
    }
}

/// The grant check (claims 3 and 4) at the moments of claim 2 (a) and (c).
pub fn grants(w: &mut World, h: HouseId) {
    // spec: S-0512 claim 3 (a defeated house is granted nothing)
    if defeated(w, h) {
        return;
    }
    for i in 0..type_count(w) {
        let granted = record(w, h, i).is_some_and(|r| r.granted);
        // spec: S-0512 claim 3 (a not-granted record with a giving building; a one-time record is already granted)
        if granted || barred(w, i) || giving(w, h, i, None).is_empty() {
            continue;
        }
        grant_by_building(w, h, i, false);
    }
}

/// A grant as by buildings (claims 3, 4 and 20); `never_suspended` for a
/// permanent trigger grant (claim 13).
fn grant_by_building(w: &mut World, h: HouseId, i: u32, never_suspended: bool) {
    let t = w.tick;
    let (ty_powered, ty_charge, ty_manual) = {
        let ty = &w.rules().superweapons.types[i as usize];
        (ty.powered, ty.charge, ty.manual)
    };
    let suspended = if never_suspended {
        // spec: S-0512 claim 13 (never suspended at the grant)
        false
    } else if w.rules().quirks.superweapon_grant_suspends_by_house_power {
        // spec: S-0512 claim 4 (quirks.superweapon_grant_suspends_by_house_power: a grant while the house has low power starts suspended, power-dependent or not)
        crate::econ::power::low_power(w, h)
    } else {
        // spec: S-0512 claim 4 (quirk off: suspended exactly when claim 6 would suspend it)
        let low = crate::econ::power::low_power(w, h);
        let on = giving(w, h, i, None)
            .iter()
            .any(|&b| w.obj(b).is_some_and(|o| !o.econ.building.switched_off));
        ty_powered && (low || !on)
    };
    let seq = next_seq(w);
    let Some(r) = record_mut(w, h, i) else { return };
    // spec: S-0512 claim 3 (the grant sets the remaining time to T and starts the clock; no announcement, no draw)
    r.granted = true;
    r.ready = false;
    r.one_time = false;
    r.silent = false;
    r.remain = ty_charge;
    r.start = t;
    // spec: S-0512 claim 20 (a manual-charge record is granted with its clock stopped at the full T)
    r.running = !ty_manual && !suspended;
    r.suspended = suspended;
    // spec: S-0512 claim 19 (a drain-mode record is granted in charging with the full T)
    r.drain = DrainState::Charging;
    r.seq = seq;
}

/// Completion (claims 5 and 19), in every house update before the checks
/// of claim 2 (a).
pub fn complete(w: &mut World, h: HouseId) {
    let t = w.tick;
    for i in 0..type_count(w) {
        let (drain, charge) = {
            let ty = &w.rules().superweapons.types[i as usize];
            (ty.drain, ty.charge)
        };
        let mut announce = false;
        {
            let Some(r) = record_mut(w, h, i) else {
                continue;
            };
            // spec: S-0512 claim 5 (granted, not suspended, not ready, clock running, remaining time 0)
            if !(r.granted && !r.suspended && !r.ready && r.running && r.remaining(t) == 0) {
                continue;
            }
            if drain {
                match r.drain {
                    DrainState::Charging => {
                        // spec: S-0512 claim 19 (expiry in charging makes it ready, with no announcement and no new ready tick)
                        r.drain = DrainState::Ready;
                        r.ready = true;
                    }
                    DrainState::Active => {
                        // spec: S-0512 claim 19 (expiry in active, while not suspended, moves to charging with the full T, without an effect-ending action)
                        r.drain = DrainState::Charging;
                        r.restart(t, charge, true);
                    }
                    DrainState::Ready => {}
                }
                continue;
            }
            r.ready = true;
            if r.silent {
                // spec: S-0512 claim 10 (our engine: a source pick's completion repeats neither the announcement nor the flash window)
                r.silent = false;
            } else {
                // spec: S-0512 claim 5 (the ready tick; the announcement for the record's own player)
                r.ready_tick = t;
                announce = true;
            }
        }
        if announce {
            super::event(w, super::SwEvent::Ready, Some(h), Some(i), None);
        }
    }
}

/// The superweapon part of a house's update before the tech recheck:
/// claim 2 (b) when the low-power state changed, then completion (claim 5).
pub fn house_update(w: &mut World, h: HouseId, low_changed: bool) {
    if w.rules().superweapons.types.is_empty() {
        return;
    }
    house_mut(w, h);
    if low_changed {
        // spec: S-0512 claim 2 (b) (a change of the house's low-power state: the check for suspension, resumption and loss only)
        availability(w, h, None);
    }
    // spec: S-0512 claim 5 and State and ordering (completion before this update's checks of claim 2 (a), after a low-power suspension)
    complete(w, h);
}

/// Claim 2 (a): the checks at a recheck of the house, in the house update
/// where it happens. `recheck` tells whether the tech recheck ran.
pub fn after_recheck(w: &mut World, h: HouseId, recheck: bool) {
    if !recheck || w.rules().superweapons.types.is_empty() {
        return;
    }
    // spec: S-0512 claim 2 (a) (suspension, resumption and loss, then new grants; a record lost there can be granted again there)
    availability(w, h, None);
    grants(w, h);
}

/// Claim 2 (c): a building gained an upgrade.
pub fn upgrade_gained(w: &mut World, building: ObjectId) {
    let Some(h) = w.obj(building).and_then(|o| o.owner) else {
        return;
    };
    if w.rules().superweapons.types.is_empty() {
        return;
    }
    // spec: S-0512 claim 2 (c) (a building gaining an upgrade: the grant check at once)
    grants(w, h);
}

/// Claim 2 (d): a building lost upgrade `u` (a building type).
pub fn upgrade_lost(w: &mut World, building: ObjectId, u: u32) {
    let Some(h) = w.obj(building).and_then(|o| o.owner) else {
        return;
    };
    let fields = w
        .rules()
        .superweapons
        .building_sw
        .get(u as usize)
        .copied()
        .unwrap_or([None, None]);
    let names = if w.rules().quirks.superweapon_upgrade_second_removal_waits {
        // spec: S-0512 claim 2 (d) (quirks.superweapon_upgrade_second_removal_waits: only an upgrade naming a first superweapon makes the immediate check)
        fields[0].is_some()
    } else {
        // spec: S-0512 claim 2 (d) (quirk off: an upgrade naming either field)
        fields.iter().any(Option::is_some)
    };
    if names {
        availability(w, h, None);
    }
}

/// Claim 2 (d): a building is being removed (it still stands when this
/// runs; the check leaves it out).
pub fn building_removed(w: &mut World, building: ObjectId) {
    let Some(o) = w.obj(building) else { return };
    if o.kind != Kind::Building {
        return;
    }
    let Some(h) = o.owner else { return };
    let fields = w
        .rules()
        .superweapons
        .building_sw
        .get(o.type_index as usize)
        .copied()
        .unwrap_or([None, None]);
    // spec: S-0512 claim 2 (d) (a building with a superweapon in its first or second field, its auxiliary condition satisfied, being removed: the check at once)
    let gives = fields.iter().flatten().any(|&i| {
        let aux = w
            .rules()
            .superweapons
            .types
            .get(i as usize)
            .and_then(|t| t.aux);
        aux.is_none_or(|a| {
            w.objects.values().any(|b| {
                b.kind == Kind::Building
                    && b.owner == Some(h)
                    && b.on_map
                    && b.health > 0
                    && b.type_index == a
            })
        })
    });
    if gives {
        availability(w, h, Some(building));
    }
}

// ---------------------------------------------------------------------------
// Trigger actions (S-0011)

/// A one-time grant (claim 12).
pub fn one_time_grant(w: &mut World, h: HouseId, i: u32) {
    if i >= type_count(w) {
        return;
    }
    let t = w.tick;
    let ty_drain = w.rules().superweapons.types[i as usize].drain;
    let keep_stale = w.rules().quirks.one_time_grant_keeps_stale_suspension;
    if record(w, h, i).is_some_and(|r| r.granted) {
        // spec: S-0512 claim 12 (a one-time grant on a granted record does nothing)
        return;
    }
    let seq = next_seq(w);
    let Some(r) = record_mut(w, h, i) else { return };
    // spec: S-0512 claim 12 (granted as one-time and ready at once, no announcement; its ready tick is that tick)
    r.granted = true;
    r.one_time = true;
    r.permanent = false;
    r.ready = true;
    r.silent = false;
    r.ready_tick = t;
    // spec: S-0512 claim 17 (a drain-mode finite window uses the last one-time grant tick)
    r.one_time_tick = t;
    r.remain = 0;
    r.start = t;
    r.running = true;
    // spec: S-0512 claim 19 (a one-time drain-mode record is granted in ready)
    r.drain = if ty_drain {
        DrainState::Ready
    } else {
        DrainState::Charging
    };
    if !keep_stale {
        // spec: S-0512 claim 12 (quirks.one_time_grant_keeps_stale_suspension off: a one-time grant ends any suspension)
        r.suspended = false;
    }
    r.seq = seq;
}

/// A permanent grant (claim 13).
pub fn permanent_grant(w: &mut World, h: HouseId, i: u32) {
    if i >= type_count(w) {
        return;
    }
    let t = w.tick;
    let freeze = w.rules().quirks.permanent_grant_freezes_suspension;
    let manual = w.rules().superweapons.types[i as usize].manual;
    if record(w, h, i).is_some_and(|r| r.granted) {
        let Some(r) = record_mut(w, h, i) else { return };
        // spec: S-0512 claim 13 (an already granted record is only removed from the checks)
        r.permanent = true;
        if !freeze && r.suspended {
            // spec: S-0512 claim 13 (quirks.permanent_grant_freezes_suspension off: the action also resumes the record)
            r.suspended = false;
            if !r.running && !manual {
                r.start = t;
                r.running = true;
            }
        }
        return;
    }
    // spec: S-0512 claim 13 (granted as a building would, never suspended at the grant)
    grant_by_building(w, h, i, true);
    if let Some(r) = record_mut(w, h, i) {
        r.permanent = true;
    }
}

/// Sets a record's charge to `p` percent (claim 14, F4).
pub fn set_charge(w: &mut World, h: HouseId, i: u32, p: i32) {
    // spec: S-0512 claim 14 (values outside 0 to 100 do nothing)
    if !(0..=100).contains(&p) || i >= type_count(w) {
        return;
    }
    if !record(w, h, i).is_some_and(|r| r.granted) {
        // spec: S-0512 claim 14 (only a granted record)
        return;
    }
    let t = w.tick;
    let mode = w.rules().mode;
    let tt = w.rules().superweapons.types[i as usize].charge;
    // spec: S-0512 F4 (c is the single-precision value nearest 0.01)
    let c = glasnost_num::decimal::parse("0.01", glasnost_num::decimal::Field::Single, mode)
        .unwrap_or(F64::ZERO);
    // spec: S-0512 F4 (r = T − trunc(Z(Z(p × c) × T)))
    let r = tt
        - F64::from_i64(i64::from(p), mode)
            .mul(c, mode)
            .mul(F64::from_i64(i64::from(tt), mode), mode)
            .to_i32_common();
    let ignores = w.rules().quirks.superweapon_restart_ignores_suspension;
    let keeps_ready = w.rules().quirks.set_charge_keeps_ready;
    let Some(rec) = record_mut(w, h, i) else {
        return;
    };
    // spec: S-0512 claim 14 (the clock runs from that tick; quirks.superweapon_restart_ignores_suspension: even when suspended)
    let running = !rec.suspended || ignores;
    rec.restart(t, r, running);
    if !keeps_ready {
        // spec: S-0512 claim 14 (quirks.set_charge_keeps_ready off: readiness is cleared)
        rec.ready = false;
    }
}

/// Resets a record's charge (claim 15): the trigger action, and spy
/// infiltration through [`spy_reset`].
pub fn reset(w: &mut World, h: HouseId, i: u32) {
    let t = w.tick;
    let Some(tt) = w
        .rules()
        .superweapons
        .types
        .get(i as usize)
        .map(|t| t.charge)
    else {
        return;
    };
    let Some(r) = record_mut(w, h, i) else { return };
    // spec: S-0512 claim 15 (no grant of a record that is not granted)
    if !r.granted {
        return;
    }
    // spec: S-0512 claim 15 (not ready, remaining T; a running clock restarts from that tick, a stopped one stays stopped)
    r.ready = false;
    r.remain = tt;
    if r.running {
        r.start = t;
    }
}

/// Spy infiltration of building `b` (S-0510 claim 3, row E4; S-0512 claim
/// 15): the owner's record of the building type's first superweapon. The
/// seam S-0510's infiltration calls.
pub fn spy_reset(w: &mut World, b: ObjectId) {
    let Some(o) = w.obj(b) else { return };
    let Some(h) = o.owner else { return };
    let first = w
        .rules()
        .superweapons
        .building_sw
        .get(o.type_index as usize)
        .and_then(|f| f[0]);
    // spec: S-0512 claim 15 (only the building type's first superweapon, never its second)
    if let Some(i) = first {
        reset(w, h, i);
    }
}

// ---------------------------------------------------------------------------
// Sidebar figures (presentation reads them)

/// The clock stage of a record (F3), 0 to 54.
pub fn stage(w: &World, h: HouseId, i: u32) -> i32 {
    let Some(r) = record(w, h, i) else { return 0 };
    let Some(ty) = w.rules().superweapons.types.get(i as usize) else {
        return 0;
    };
    // spec: S-0512 F3 (not granted: 0)
    if !r.granted {
        return 0;
    }
    // spec: S-0512 F3 (ready, not drain-mode: 54)
    if r.ready && !ty.drain {
        return 54;
    }
    let tt = i64::from(ty.charge.max(1));
    let rem = i64::from(r.remaining(w.tick));
    if ty.drain && r.drain == DrainState::Active {
        // spec: S-0512 F3 (drain active: min(53, ⌊54 r / (T ρ)⌋), exact over the integer times and the imported ratio)
        let Some((neg, m, e)) = w.rules().superweapons.ratio.parts() else {
            return 53; // spec: S-0512 F3 (the last charging stage 53)
        };
        if neg {
            return 53; // spec: S-0512 F3 (the last charging stage 53)
        }
        let num = 54u128 * rem.max(0) as u128; // spec: S-0512 F3 (54 stages)
        let den = tt as u128 * u128::from(m);
        let v = if e >= 0 {
            den.checked_shl(e as u32)
                .filter(|d| d.leading_zeros() as i32 > e)
                .map_or(0, |d| num / d.max(1))
        } else {
            let s = (-e) as u32;
            if num.leading_zeros() > s {
                (num << s) / den.max(1)
            } else {
                53 // spec: S-0512 F3 (the last charging stage 53)
            }
        };
        return v.min(53) as i32; // spec: S-0512 F3 (min(53, …))
    }
    // spec: S-0512 F3 (min(53, ⌊54 × (T − r) / T⌋))
    ((54 * (tt - rem)).div_euclid(tt)).clamp(0, 53) as i32
}

/// The readiness labels (claim 16).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Label {
    /// No label.
    None,
    /// On hold.
    OnHold,
    /// Ready.
    Ready,
    /// Charging (drain-mode).
    Charging,
    /// Active (drain-mode).
    Active,
}

/// A record's readiness label (claim 16).
pub fn label(w: &World, h: HouseId, i: u32) -> Label {
    let Some(r) = record(w, h, i) else {
        return Label::None;
    };
    let drain = w
        .rules()
        .superweapons
        .types
        .get(i as usize)
        .is_some_and(|t| t.drain);
    // spec: S-0512 claim 16 (on hold while suspended, ready or not; drain-mode types show their state)
    if !r.granted {
        Label::None
    } else if r.suspended {
        Label::OnHold
    } else if drain {
        match r.drain {
            DrainState::Charging => Label::Charging,
            DrainState::Ready => Label::Ready,
            DrainState::Active => Label::Active,
        }
    } else if r.ready {
        Label::Ready
    } else {
        Label::None
    }
}

/// Whether the icon offers targeting (claim 16).
pub fn offers_targeting(w: &World, h: HouseId, i: u32) -> bool {
    let Some(r) = record(w, h, i) else {
        return false;
    };
    let drain = w
        .rules()
        .superweapons
        .types
        .get(i as usize)
        .is_some_and(|t| t.drain);
    // spec: S-0512 claim 16 (granted and not suspended; ready for an ordinary type, ready or active for a drain-mode type)
    r.granted
        && !r.suspended
        && if drain {
            matches!(r.drain, DrainState::Ready | DrainState::Active)
        } else {
            r.ready
        }
}

/// Whether a record counts as a ready superweapon in S-0411's sense in
/// tick `t` (claim 17).
pub fn flashing(w: &World, h: HouseId, i: u32) -> bool {
    let Some(r) = record(w, h, i) else {
        return false;
    };
    let Some(ty) = w.rules().superweapons.types.get(i as usize) else {
        return false;
    };
    let ready = if ty.drain {
        matches!(r.drain, DrainState::Ready | DrainState::Active)
    } else {
        r.ready
    };
    if !(r.granted && ready && !r.suspended) {
        return false;
    }
    // spec: S-0512 claim 17 (a drain-mode window uses the last one-time grant tick, −100 if none)
    let from = if ty.drain {
        r.one_time_tick
    } else {
        r.ready_tick
    };
    let t = w.tick;
    match ty.flash {
        // spec: S-0512 claim 17 (−1: unlimited)
        -1 => true,
        // spec: S-0512 claim 17 (F > 0: ticks r to r + F − 1; 0 or other values: no window)
        f if f > 0 => t >= from && t < from + f,
        _ => false,
    }
}

/// One countdown row (claim 18).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    /// The house.
    pub house: HouseId,
    /// The type by list position.
    pub sw: u32,
    /// The seconds *s* of F6.
    pub seconds: i32,
}

/// The countdown rows in grant order (claim 18).
pub fn countdown_rows(w: &World) -> Vec<Row> {
    let t = w.tick;
    let mut rows: Vec<(u64, Row)> = Vec::new();
    for (hi, hs) in w.sw.houses.iter().enumerate() {
        let h = HouseId(hi as u8);
        // spec: S-0512 claim 18 (no rows for a passive house)
        if crate::econ::setup::passive(w, h) {
            continue;
        }
        for (i, r) in hs.records.iter().enumerate() {
            let shows = w
                .rules()
                .superweapons
                .types
                .get(i)
                .is_some_and(|ty| ty.show_timer);
            if r.granted && shows {
                rows.push((
                    r.seq,
                    Row {
                        house: h,
                        sw: i as u32,
                        // spec: S-0512 F6 (s = ⌊r / 15⌋)
                        seconds: r.remaining(t).div_euclid(15),
                    },
                ));
            }
        }
    }
    rows.sort_by_key(|(s, _)| *s);
    rows.into_iter().map(|(_, r)| r).collect()
}

/// A record, cloned, for readers outside the module.
pub fn get(w: &World, h: HouseId, i: u32) -> Option<Record> {
    record(w, h, i).cloned()
}
