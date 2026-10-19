//! Invulnerability and the force shield (S-0513 claims 22 to 32).
//!
//! Protection itself is the [`super::Protection`] timer on each object;
//! its consumers read [`super::protected`] and [`super::protection_kind`]:
//! the hit table (`combat::damage`, S-0302 claim 8), the nullified small
//! detonation (`combat::area`, S-0303 claim 15), the firing answers
//! (`combat::check`, claims 27 and 28), crushing and the mission-target
//! occupancy row (`occupancy`, S-0210 claim 2, S-0209 row WP3), and the
//! warp (claims 34, 37 and 38). The refusals of other specs read the same
//! test there: mind control (S-0601, `crate::mind::can_take` and
//! `crate::mind::dominate`, which the psychic domination's sweep uses,
//! S-0612), locomotor seizing (S-0607, `crate::lift::seize_hit`), and
//! those not built yet (erasure S-0514, flinging, charge planting S-0519).

use glasnost_map::CellXY;
use glasnost_num::geom::root_floor;

use super::{ProtectKind, Protection, SwEvent, Tint};
use crate::rules::Kind;
use crate::world::{HouseId, ObjectId, Pos, World};

/// The invoke animations' lift above the cell point (Constants).
// spec: S-0513 Constants (invoke animation lift 5 leptons; claims 22, 31, 33 and 34)
pub const INVOKE_LIFT: i32 = 5;

/// The length of the envelope's Initial rise (claim 29).
// spec: S-0513 claim 29 (Initial rise: 6 ticks, starting at q)
pub const TINT_RISE: i32 = 6;

/// The length of the envelope's Bright hold (claim 29).
// spec: S-0513 claim 29 (Bright hold: 4 ticks)
pub const TINT_HOLD: i32 = 4;

/// The tint draw's range (Constants).
// spec: S-0513 Constants (tint draw range −5 … 5 ticks)
const TINT_DRAW: (i32, i32) = (-5, 5);

/// Places an invoke animation at a cell's centre, `INVOKE_LIFT` above its
/// surface (with the bridge lift when `lift`), if the type is named.
pub fn invoke_anim(w: &mut World, name: Option<String>, cell: CellXY, lift: bool) {
    let Some(name) = name else {
        // spec: S-0513 Edge cases (a missing invoke animation type: no animation; the importer warns)
        return;
    };
    let mut p = super::cell_point(w, cell, lift);
    p.z += INVOKE_LIFT;
    // spec: S-0202 claim 10 (an animation's creation draws)
    crate::combat::effects::anim(w, &name, p);
}

/// Starts protection of `kind` for `duration` ticks on an object (claim 25).
pub fn start(w: &mut World, id: ObjectId, kind: ProtectKind, duration: i32) {
    let t = w.tick;
    let Some(o) = w.obj(id) else { return };
    let okind = o.kind;
    let replaces = w.rules().quirks.protection_restart_replaces;
    let old = o.sw.protection;
    let in_force = super::protection_kind(w, id).is_some();
    let new = Protection {
        start: t,
        duration,
        kind,
        // spec: S-0513 claim 29 (reapplication restarts the envelope, even with another kind)
        tint: Tint::BeforeFirstTurn,
    };
    let keep_old = !replaces
        && in_force
        && old.is_some_and(|p| {
            // spec: S-0513 claim 25 (quirk off: the longer remaining time is kept, with the kind of the effect that gave it)
            p.start + p.duration - t >= duration
        });
    if !keep_old && let Some(ob) = w.obj_mut(id) {
        // spec: S-0513 claim 25 (quirks.protection_restart_replaces: the new timer and kind replace the old ones)
        ob.sw.protection = Some(new);
    }
    match okind {
        Kind::Vehicle | Kind::Aircraft => {
            // spec: S-0513 claim 25 (a vehicle or aircraft: releases a parasite inside it with 50 ticks of suppression set just before the exit, and ends a paralysis; S-0614 claim 12)
            crate::parasite::host_curtain_started(w, id);
        }
        Kind::Building => {
            if let Some(ob) = w.obj_mut(id)
                && ob.combat.delay_kill_until.is_some()
            {
                // spec: S-0513 claim 25 (a building: cancels a pending delayed-kill countdown, S-0302 claim 16; no charge planter is ever recorded in our engine)
                ob.combat.delay_kill_until = None;
                // The countdown's kill mark goes with it: the building, still at
                // its health, is alive again for every later rule.
                ob.combat.killed = false;
            }
        }
        _ => {}
    }
}

/// The curtain (claims 22 to 25). `h` is the launching house (none for the
/// waypoint trigger action); `waypoint` selects the trigger action's rules
/// (ground layer only, no leaving-stage skip).
pub fn curtain(w: &mut World, h: Option<HouseId>, cell: CellXY, waypoint: bool) {
    let anim = w.rules().superweapons.curtain_anim.clone();
    // spec: S-0513 claim 22 (the curtain invoke animation at the target cell's centre, 5 leptons above its surface plus the bridge lift; the waypoint action: the terrain surface plus 5 leptons, no bridge lift)
    invoke_anim(w, anim, cell, !waypoint);
    let duration = w.rules().superweapons.curtain_duration;
    let organic_defended = w.rules().quirks.curtain_organic_vehicle_defended;
    for (_, c) in super::block(w, cell) {
        let layers = if waypoint {
            // spec: S-0513 claim 22 (the waypoint action: the ground layer, whatever the bridge)
            vec![crate::world::Layer::Ground]
        } else {
            super::swept_layers(w, c)
        };
        for l in layers {
            super::walk_list(w, c, l, &mut |w, id| {
                let Some(o) = w.obj(id) else { return };
                let kind = o.kind;
                if kind.is_mobile() && !waypoint && super::warp_pending_or_leaving(w, id) {
                    // spec: S-0513 claim 22 (a unit in the leaving stage of a warp, or with that warp pending, is skipped)
                    return;
                }
                match kind {
                    Kind::Infantry => {
                        // spec: S-0513 claim 23 (infantry: whole strength, demolition warhead, ignoring defences, the launching house; never protected)
                        super::demolish(w, id, true, h);
                    }
                    Kind::Vehicle | Kind::Aircraft => {
                        let organic =
                            crate::combat::veterancy::type_combat(w, id).is_some_and(|t| t.organic);
                        if organic && organic_defended {
                            // spec: S-0513 claim 24 (an organic vehicle or aircraft: a hit that does not ignore defences, no attacking house; quirks.curtain_organic_vehicle_defended)
                            super::demolish(w, id, false, None);
                        } else if organic {
                            // spec: S-0513 claim 24 (quirk off: ignoring defences, crediting the launching house)
                            super::demolish(w, id, true, h);
                        } else {
                            // spec: S-0513 claim 25 (every other reached vehicle and aircraft: curtain kind for the curtain duration)
                            start(w, id, ProtectKind::Curtain, duration);
                        }
                    }
                    Kind::Building => {
                        // spec: S-0513 claim 25 (a reached building: curtain kind for the curtain duration; once per listing, claim 4)
                        start(w, id, ProtectKind::Curtain, duration);
                    }
                    // spec: S-0513 claim 22 (a non-mobile object other than a building is unaffected)
                    _ => {}
                }
            });
        }
    }
}

/// The force shield (claims 30 to 32).
pub fn force_shield(w: &mut World, h: HouseId, i: u32, cell: CellXY) {
    let (blackout, anim, radius, duration, fade) = {
        let s = &w.rules().superweapons;
        (
            s.shield_blackout,
            s.shield_anim.clone(),
            s.shield_radius,
            s.shield_duration,
            s.shield_fade,
        )
    };
    // spec: S-0513 claim 30 (a timed blackout of the launching house first, S-0405 claim 6)
    crate::econ::power::start_blackout(w, h, blackout);
    // spec: S-0513 claim 31 (the force-shield invoke animation at the target cell's centre; Constants: 5 leptons lift)
    invoke_anim(w, anim, cell, true);
    // spec: S-0513 claim 32 (the type's start sound at the target when it has one)
    if w.rules().superweapons.types[i as usize]
        .start_sound
        .is_some()
    {
        let p = super::cell_point(w, cell, false);
        super::event(w, SwEvent::ShieldStart, Some(h), Some(i), Some(p));
    }
    let centre = Pos::centre(cell, 0);
    // spec: S-0513 F3 (the target cell centre (256x + 128, 256y + 128) at the terrain surface height, no bridge lift)
    let z = w.surface_at(centre.x, centre.y);
    let table = w.rules().quirks.table_root_distance;
    let mode = w.rules().mode;
    let ids: Vec<ObjectId> = w.objects.keys().copied().collect();
    for id in ids {
        let Some(o) = w.obj(id) else { continue };
        if o.kind != Kind::Building {
            // spec: S-0513 claim 31 (infantry, vehicles and aircraft are not protected)
            continue;
        }
        // spec: S-0513 claim 31 (the building's owner counts the launching house as allied, itself included)
        if !w.allied(o.owner, Some(h)) {
            continue;
        }
        // spec: S-0513 claim 31 (the reference point, S-0204 claim 4)
        let p = w.reference_point(id);
        let (dx, dy, dz) = (
            i64::from(p.x - centre.x),
            i64::from(p.y - centre.y),
            i64::from(p.z - z),
        );
        let n = (dx * dx + dy * dy + dz * dz) as u64;
        // spec: S-0513 F3 (the table square root truncated, strictly below 256 × R)
        let d = root_floor(n, table, mode) as i64;
        let limit = 256 * i64::from(radius); // spec: S-0513 F3 (256 leptons per cell of the radius R)
        if d < limit {
            // spec: S-0513 claim 31 (force-shield kind for the force-shield duration, with claim 25's effects)
            start(w, id, ProtectKind::ForceShield, duration);
        }
    }
    let lead = duration - fade;
    let p = super::cell_point(w, cell, false);
    let hs = super::house_mut(w, h);
    hs.fade = if lead < 0 {
        // spec: S-0513 claim 32 (never when D − P is negative; a second launch replaces the pending sound)
        None
    } else {
        // spec: S-0513 claim 32 (the house update D − P house updates after the launch, the next one when D − P = 0)
        Some((lead.max(1), p, i))
    };
}

/// The fading sound's countdown in the house update (claim 32).
pub fn fade_update(w: &mut World, h: HouseId) {
    let Some(hs) = w.sw.houses.get_mut(usize::from(h.0)) else {
        return;
    };
    let Some((left, p, i)) = hs.fade else { return };
    if left > 1 {
        hs.fade = Some((left - 1, p, i));
        return;
    }
    hs.fade = None;
    if w.rules().superweapons.types[i as usize]
        .special_sound
        .is_some()
    {
        // spec: S-0513 claim 32 (the type's special sound at the target; presentation)
        super::event(w, SwEvent::ShieldFade, Some(h), Some(i), Some(p));
    }
}

/// The protection envelope's phases up to its draw, in the object's own
/// turn (claim 29).
pub fn tint_turn(w: &mut World, id: ObjectId) {
    let t = w.tick;
    let Some(p) = w.obj(id).and_then(|o| o.sw.protection) else {
        return;
    };
    if !super::protected(w, id) {
        // spec: S-0513 claim 29 (expiry before Initial dimming prevents the draw; no turns while unprotected advance the envelope)
        return;
    }
    // spec: S-0513 claim 29 (at most one phase transition per turn, when t − s ≥ L; the next phase is entered in that turn's tick)
    let next = match p.tint {
        // spec: S-0513 claim 29 (Initial rise starts at q, the first turn while this application protects the object)
        Tint::BeforeFirstTurn => Tint::Rise(t),
        Tint::Rise(s) if t - s >= TINT_RISE => Tint::BrightHold(t),
        Tint::BrightHold(s) if t - s >= TINT_HOLD => Tint::Dimming,
        _ => return,
    };
    if let Some(p) = w.obj_mut(id).and_then(|o| o.sw.protection.as_mut()) {
        p.tint = next;
    }
    if next == Tint::Dimming {
        let half = w.rules().quirks.ranged_draw_half_span;
        // spec: S-0513 claim 29 (exactly one ranged draw R(−5, 5) from the scenario stream on entering Initial dimming; its length 20 + the draw is presentation)
        let _ = w.rng.ranged(TINT_DRAW.0, TINT_DRAW.1, half);
    }
}
