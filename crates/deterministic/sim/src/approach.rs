//! The approach step (S-0215 claim 37, F10): whether an infantry unit or
//! vehicle with a target out of range keeps it, keeps or renews its
//! destination, or looks for an approach position.
//!
//! The approach-position search is S-0307's (claims 15 to 17,
//! `combat::range::approach_search`, request Q-0172 for the cases its
//! revision leaves open), and so is part h's balloon-hover approach (claim
//! 18, `combat::range::balloon_hover_approach`).
//!
//! Seams, each answering "no" or "nothing" until its topic fills it in: the
//! approach destinations of a vehicle (no rule fills them yet, S-0215 Open
//! questions), part h's crushing approaches (S-0210 claim 11), the magnetic
//! beam of part c, draining links, bunkers, open-topped passengers and teams.
//! Every object on the map has the in-the-playfield mark (no object arrives
//! from outside the usable area yet).

use glasnost_num::geom::root_floor;

use crate::infantry::{self, Seq};
use crate::rules::{Behaviour, Kind};
use crate::world::{ObjectId, Pos, Target, World};

/// The reference point of a target or destination (S-0215 F10): an
/// object's reference point, or a cell's centre at its height.
fn point(w: &World, t: Target) -> Pos {
    match t {
        Target::Object(x) => w.reference_point(x),
        Target::Cell(_) => crate::process::target_pos(w, t),
    }
}

/// ρ₃ between two points (S-0215 F4).
fn rho3(w: &World, a: Pos, b: Pos) -> i64 {
    let r = w.rules();
    let (dx, dy, dz) = (
        i64::from(a.x - b.x),
        i64::from(a.y - b.y),
        i64::from(a.z - b.z),
    );
    // spec: S-0215 F4 (ρ₃ = trunc(r(n₃)), the table square root; quirks.table_root_distance)
    root_floor(
        (dx * dx + dy * dy + dz * dz) as u64,
        r.quirks.table_root_distance,
        r.mode,
    ) as i64
}

/// Whether the actor's target is in range for the approach step (S-0215
/// claim 37 b): S-0307's range test from its current position, with the
/// weapon chosen for the target.
pub fn in_range(w: &World, id: ObjectId, t: Target) -> bool {
    let (_, weapon) = crate::combat::weapon::would_use(w, id, Some(t));
    // spec: S-0215 claim 37 (b) (the range test of S-0307 claims 1 to 12, not the whole firing check)
    crate::combat::range::in_range(w, id, weapon, Some(t), None)
}

/// Whether a deploy-to-fire vehicle stands on a cell where its deploy test
/// fails (S-0215 claim 37 f2, S-0301's must-deploy answer). The deploy test
/// of S-0301 is not implemented yet, so no vehicle is in that case.
fn deploy_to_fire_refused(w: &World, id: ObjectId) -> bool {
    let _ = (w, id);
    false
}

/// The approach-position search of S-0307 claims 15 to 17, started with the
/// radius r′ of F10, and its outcome applied (S-0215 claim 37 f, last row).
fn search(w: &mut World, id: ObjectId, r_prime: i32) {
    use crate::combat::range::Approach;
    // The search reads the reach regions, which must be up to date.
    crate::regions::ensure(w);
    let Some(outcome) = crate::combat::range::approach_search(w, id, r_prime) else {
        return;
    };
    let Some(o) = w.obj(id) else { return };
    let (before, t) = (o.dest, o.target);
    let Some(ob) = w.obj_mut(id) else { return };
    match outcome {
        // spec: S-0307 claims 15 and 16 (a position found: it becomes the destination and the target is kept)
        Approach::Cell(c) => ob.dest = Some(Target::Cell(c)),
        // spec: S-0307 claim 17 (a hunter-seeker or vehicle thief takes the target object as destination and keeps the target)
        Approach::TargetItself => ob.dest = t,
        // spec: S-0307 claim 17 (any other actor drops the target; the destination becomes the nearby cell, or none)
        Approach::Drop(n) => {
            ob.target = None;
            ob.combat.burst = 0; // spec: S-0301 claim 4 (a cleared target resets the burst counter)
            ob.dest = n.map(Target::Cell);
        }
    }
    // spec: S-0107 claim 15 (a destination assigned: the destination-driven get-up)
    infantry::destination_assigned(w, id, before);
}

/// r′ of F10 for the actor's weapon chosen for the target.
fn r_prime(w: &World, id: ObjectId, t: Target, f2: bool) -> i32 {
    let (slot, _) = crate::combat::weapon::would_use(w, id, Some(t));
    // spec: S-0215 F10 (ρ: the weapon range of S-0307 claim 13 for the weapon chosen for the target)
    let rho = crate::combat::range::firer_weapon_range(w, id, slot);
    // spec: S-0215 F10 and Constants (r′ = ρ − 128 when ρ ≥ 512, else ρ)
    let mut r = if rho >= 512 { rho - 128 } else { rho };
    if f2 {
        let d = rho3(w, w.reference_point(id), point(w, t));
        // spec: S-0215 F10 and claim 37 f2 (the deploy-to-fire vehicle: r′ = min(r′, ρ₃ + 512))
        let alt = d + 512;
        if alt <= i64::from(r) {
            r = alt as i32;
        }
    }
    r
}

/// The approach step (S-0215 claim 37) of an infantry unit or vehicle.
pub fn step(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id).cloned() else { return };
    // spec: S-0215 claim 37 (g) (buildings and aircraft never take the approach step)
    if !matches!(o.kind, Kind::Infantry | Kind::Vehicle) {
        return;
    }
    // spec: S-0215 claim 37 (c) (no target: nothing; the magnetic beam is a seam)
    let Some(t) = o.target else { return };
    // spec: S-0215 claim 37 (h) and S-0307 claim 18 (before the other parts: a balloon-hover vehicle whose weapon chosen for the target has a vertical projectile takes the target as its destination, keeps it, and does nothing else; the crushing approaches before it are a seam)
    if crate::combat::range::balloon_hover_approach(w, id) {
        if let Some(ob) = w.obj_mut(id) {
            ob.dest = Some(t);
        }
        return;
    }
    let human = o.owner.is_some_and(|h| w.houses[usize::from(h.0)].human);
    let rules = w.rules_arc();
    let u = rules.unit(o.kind, o.type_index);
    let deployed = o.kind == Kind::Infantry
        && o.infantry
            .as_ref()
            .and_then(|s| s.seq)
            .is_some_and(Seq::is_deploy);
    if deployed {
        // spec: S-0215 claim 37 (g) (a deployed infantry unit takes no approach step; original only: one of a computer-controlled owner whose type fires when deployed undeploys when its target is out of range)
        if !human && u.is_some_and(|u| u.raw.deploy_fire.unwrap_or(false)) && !in_range(w, id, t) {
            infantry::request(w, id, Seq::Undeploy, false);
        }
        return;
    }
    let ranged = in_range(w, id, t);
    if !ranged {
        // spec: S-0215 claim 37 (d) (may approach: the can-approach property, and not draining, in a bunker or an open-topped passenger, all seams)
        let may = u.is_none_or(|u| u.raw.can_approach_target.unwrap_or(true));
        // spec: S-0215 claim 37 (d) (the keep-or-drop table by current behaviour, may approach and owner)
        let keep = match o.beh.current {
            Some(Behaviour::Sticky) => false,
            Some(Behaviour::Attack) => true,
            Some(Behaviour::AreaGuard) => may || human,
            Some(Behaviour::Hunt) => may || !human,
            _ => may,
        };
        if !keep {
            // spec: S-0215 claim 37 (d) (dropping clears the target and the destination at once; nothing else of the step happens)
            if let Some(ob) = w.obj_mut(id) {
                ob.target = None;
                ob.combat.burst = 0; // spec: S-0301 claim 4 (a cleared target resets the burst counter)
                ob.dest = None;
            }
            return;
        }
    }
    // spec: S-0215 claim 37 (f2) (a vehicle whose type deploys to fire, on a cell where its deploy test fails)
    let f2 = o.kind == Kind::Vehicle
        && u.is_some_and(|u| u.vehicle.deploy_to_fire)
        && deploy_to_fire_refused(w, id);
    let in_air = crate::combat::weapon::in_air(w, id);
    if let Some(d) = o.dest {
        // spec: S-0215 claim 37 (e) (the can-recalculate property: engine default yes, no key named)
        let can_recalculate = true;
        let mut cleared = false;
        if can_recalculate && !ranged {
            let rp = r_prime(w, id, t, f2);
            let m = rules.general.approach_reset_multiplier;
            // spec: S-0215 F10 (m × r′, a signed 32-bit product that wraps)
            let threshold = m.wrapping_mul(rp);
            // spec: S-0215 F10 (cleared when ρ₃(target, destination) > m × r′, strictly)
            if rho3(w, point(w, t), point(w, d)) > i64::from(threshold) {
                if let Some(ob) = w.obj_mut(id) {
                    ob.dest = None;
                }
                cleared = true;
            }
        }
        // spec: S-0215 claim 37 (e) (a kept destination ends the step for an actor not in the air, also in range)
        if !cleared && !in_air {
            return;
        }
    }
    // spec: S-0215 claim 37 (f) (in range, in the playfield and not the deploy-to-fire vehicle of f2: the step ends)
    if ranged && !f2 {
        return;
    }
    // spec: S-0215 claim 37 (f) (a vehicle with approach destinations takes the first; no rule fills them yet)
    let rp = r_prime(w, id, t, f2);
    search(w, id, rp);
}

/// The per-tick approach of S-0215 claim 37 (a): an infantry unit or
/// vehicle with a target and no destination, not a team member, that cannot
/// reach its target's cell (or is an open-topped passenger, a seam) takes
/// the approach step in every behaviour, and drops a target out of range
/// if it still has no destination afterwards.
pub fn per_tick(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id) else { return };
    if !matches!(o.kind, Kind::Infantry | Kind::Vehicle) || !o.on_map {
        return;
    }
    let (Some(t), None) = (o.target, o.dest) else {
        return;
    };
    // spec: S-0215 claim 37 (a) (the target's cell is not in the actor's reach region for its movement zone; no movement zone reaches every cell)
    let Some(u) = w.rules().unit(o.kind, o.type_index) else {
        return;
    };
    let tc = match t {
        Target::Object(x) => match w.obj(x) {
            Some(x) => x.pos.cell(),
            None => return,
        },
        Target::Cell(c) => c,
    };
    if crate::regions::same_region(w, u.zone, o.pos.cell(), o.layer, tc, o.layer) {
        return;
    }
    step(w, id);
    let Some(o) = w.obj(id) else { return };
    // spec: S-0215 claim 37 (a) (still no destination and the target out of range: the target is dropped)
    if o.dest.is_none()
        && let Some(t) = o.target
        && !in_range(w, id, t)
        && let Some(ob) = w.obj_mut(id)
    {
        ob.target = None;
        ob.combat.burst = 0; // spec: S-0301 claim 4 (a cleared target resets the burst counter)
    }
}
