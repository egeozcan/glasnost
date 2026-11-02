//! The approach step (S-0215 claim 37, F10): whether an infantry unit or
//! vehicle with a target out of range keeps it, keeps or renews its
//! destination, or looks for an approach position.
//!
//! The approach-position search is S-0307's (claims 15 to 17 and 19 to 24,
//! `combat::range::approach_search`), and so is part h's balloon-hover approach (claim
//! 18, `combat::range::balloon_hover_approach`). Part h's crushing
//! approaches are S-0210 claim 11's.
//!
//! Seams, each answering "no" or "nothing" until its topic fills it in: the
//! approach destinations of a vehicle (no rule fills them yet, S-0215 Open
//! questions), draining links, bunkers,
//! open-topped passengers and teams.
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
pub(crate) fn rho3(w: &World, a: Pos, b: Pos) -> i64 {
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
/// fails (S-0215 claim 37 f2, S-0301's must-deploy answer): S-0517 claim
/// 14's cell test.
fn deploy_to_fire_refused(w: &World, id: ObjectId) -> bool {
    // spec: S-0517 claim 14 (the approach of a deploy-to-fire vehicle whose own cell fails uses the same test)
    crate::econ::inplace::is_deploy_to_fire(w, id) && !crate::econ::inplace::deploy_test(w, id)
}

/// The approach-position search of S-0307 claims 15 to 24, started with the
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
    if let Approach::Shortcut(p, c) = outcome {
        shortcut(w, id, p, c);
        // spec: S-0107 claim 15 (a destination assigned: the destination-driven get-up)
        infantry::destination_assigned(w, id, before);
        return;
    }
    let Some(ob) = w.obj_mut(id) else { return };
    match outcome {
        Approach::Shortcut(..) => {}
        // spec: S-0307 claims 15 and 16 (a position found: it becomes the destination and the target is kept)
        Approach::Cell(c) => ob.dest = Some(Target::Cell(c)),
        // spec: S-0307 claim 17 (a hunter-seeker or vehicle thief takes the target object as destination and keeps the target)
        Approach::TargetItself => ob.dest = t,
        // spec: S-0307 claim 17 (any other actor drops the target; the destination becomes the nearby cell, or none)
        // spec: S-0215 claim 37 (c) (revision 104: with no nearby cell a gripped unit is given no destination, so while it has one its grip ends, S-0607 claim 9; a nearby cell is a new destination it ignores)
        Approach::Drop(n) => {
            crate::spawn::clear_own_target(ob);
            ob.combat.burst = 0; // spec: S-0301 claim 4 (a cleared target resets the burst counter)
            ob.dest = n.map(Target::Cell);
        }
    }
    // spec: S-0107 claim 15 (a destination assigned: the destination-driven get-up)
    infantry::destination_assigned(w, id, before);
}

/// The close-range shortcut's outcome (S-0307 claim 22 b; S-0208 claim
/// 21): the destination becomes cell `c`, the cell of the target's
/// reference point, and the target is kept. An ordinary walking infantry
/// unit is given an immediate walking target in its own cell, chosen again
/// with the selected point `p` as the desired point and its own entry
/// released. An actor of any other locomotor (S-0307 claim 25) only gets
/// the destination: the immediate request towards `p` changes no movement
/// point and reserves no standing position. Walk state exists only for the
/// walking locomotor, so the test is one of locomotor, not of kind or class.
// spec: S-0307 claim 25 (non-walking locomotors: destination only, no movement point, no reservation, no extra draw)
fn shortcut(w: &mut World, id: ObjectId, p: Pos, c: glasnost_map::CellXY) {
    use crate::world::Layer;
    let Some(o) = w.obj(id).cloned() else { return };
    if let Some(ob) = w.obj_mut(id) {
        // spec: S-0307 claim 22 (b) and S-0208 claim 21 (the destination becomes the cell of the target's reference point; the target is kept)
        ob.dest = Some(Target::Cell(c));
        // spec: S-0208 claim 21 (it walks to its walking target without an intervening path step into a neighbouring cell)
        if o.walk.is_some() {
            ob.path.steps.clear();
        }
    }
    let Some(walk) = o.walk.as_ref() else { return };
    let q = w.rules().quirks;
    let oq = crate::objects::occ_quirks(w);
    let own = o.pos.cell();
    // spec: S-0208 claim 21 (for the second choice the old reservation is released, so its own entry does not count as taken) and S-0209 claim 1
    if let Some(old) = walk.target {
        w.cells.clear_infantry(old.cell(), o.layer, id, oq);
    }
    w.cells.clear_infantry(own, o.layer, id, oq);
    if let Some(s) = w.obj_mut(id).and_then(|ob| ob.walk.as_mut()) {
        s.target = None;
    }
    // spec: S-0208 claim 21 (deck occupancy and the deck standing height when the cell carries a high bridge and the height exceeds the surface height at P by more than 312, strict; ground occupancy otherwise)
    let deck =
        w.high_bridge(own) && o.pos.z > w.surface_at(p.x, p.y).wrapping_add(DECK_CHOICE_ABOVE);
    let layer = if deck { Layer::Deck } else { Layer::Ground };
    // spec: S-0208 claims 3, 4 and 21 (the walking target chosen again with P as the desired point)
    let class = crate::walk::position_class(p.x.rem_euclid(256), p.y.rem_euclid(256), true); // spec: S-0204 claim 2 (the position within the cell)
    let mission = crate::walk::mission_target_in(w, id, own);
    // spec: S-0208 claim 21 (no position: no immediate walking target, not a transfer to the ground)
    let Some(pos) = crate::walk::choose_position(w, own, layer, class, mission) else {
        return;
    };
    let (u, v) = crate::walk::standing_point(pos);
    let (x, y) = (256 * own.x + u, 256 * own.y + v); // spec: S-0208 table P (a standing position within the cell)
    // spec: S-0208 claims 1 and 21 (the surface height there, the deck standing-position height 416 above on the deck)
    let z = w.surface_at(x, y) + if deck { 416 } else { 0 };
    // spec: S-0208 claims 3 and 21 (the chosen walking target is reserved)
    let mark = crate::walk::position_class(u, v, q.top_position_counts_as_centre);
    if let Some(h) = o.owner {
        w.cells.mark_infantry(own, layer, mark, id, h);
    }
    if let Some(s) = w.obj_mut(id).and_then(|ob| ob.walk.as_mut()) {
        s.target = Some(Pos::new(x, y, z));
        s.standing = pos;
    }
}

/// The deck-occupancy margin of the second position choice (S-0208 claim
/// 21): the walker's height must exceed the surface height at *P* by more
/// than this many leptons.
// spec: S-0208 claim 21 (more than 312 leptons, a strict integer comparison)
const DECK_CHOICE_ABOVE: i32 = 312;

/// Whether a vehicle's approach step is replaced by a crushing approach of
/// S-0210 claim 11 (S-0215 claim 37 h):
/// - the omni-crusher variant: its type has the omni-crusher flag and it
///   can crush its target (claim 2), at any distance and also with a move
///   destination; not for an opportunity target of a player house (human
///   or AI), while the neutral and special houses keep the original's
///   exemption;
/// - the original's attack-target variant, kept for the neutral and special
///   houses only: it can crush its target, the distance is below the crush
///   range (F2, strict), and it has no move destination.
fn crushing_approach(w: &World, id: ObjectId) -> bool {
    let Some(o) = w.obj(id) else { return false };
    if o.kind != Kind::Vehicle {
        return false;
    }
    let Some(Target::Object(x)) = o.target else {
        return false;
    };
    let r = w.rules();
    let Some(u) = r.unit(o.kind, o.type_index) else {
        return false;
    };
    // spec: S-0210 claim 11 (both variants need a target it can crush under claim 2)
    if !crate::occupancy::crushable_by(w, id, x) {
        return false;
    }
    // spec: S-0210 claim 11 (our engine: the rules of a player house, human-controlled or AI; the neutral and special houses keep the original's computer-controlled rules, as a test-only computer-controlled player house does, glossary)
    let player = o.owner.is_none_or(|h| w.houses[usize::from(h.0)].human);
    // spec: S-0210 claim 11 (omni-crusher variant: at any distance, also with a move destination; not for an opportunity target of a player house)
    if u.omni_crusher && !(player && o.combat.opportunity) {
        return true;
    }
    if player || o.dest.is_some() {
        // spec: S-0210 claim 11 (the attack-target variant needs a computer-controlled owner and no move destination)
        return false;
    }
    let Some(t) = w.obj(x) else { return false };
    // spec: S-0210 claim 11 (the attack-target variant: a unit or building)
    if !matches!(
        t.kind,
        Kind::Infantry | Kind::Vehicle | Kind::Aircraft | Kind::Building
    ) {
        return false;
    }
    let (dx, dy, dz) = (
        i64::from(t.pos.x - o.pos.x),
        i64::from(t.pos.y - o.pos.y),
        i64::from(t.pos.z - o.pos.z),
    );
    // spec: S-0210 F2 (d = trunc(√̃(Δx² + Δy² + Δz²)) between the centres; the attack-target variant needs d < R, strict)
    let d = root_floor(
        (dx * dx + dy * dy + dz * dz) as u64,
        r.quirks.table_root_distance,
        r.mode,
    ) as i64;
    d < i64::from(r.general.crush_range)
}

/// A part h approach has given a lifter its target as its destination
/// (S-0607 claims 11 and 12, revision 18): the lifter accepts that
/// destination, which ends its grip and clears its target, also when it
/// already held the same destination when it gripped (the end-of-turn test
/// of `lift::lifter_destination` would not see that one).
fn part_h_accepted(w: &mut World, id: ObjectId) {
    // spec: S-0607 claim 11 (revision 18: part h's crushing or balloon-hover approaches come first and are not excepted: such an approach gives the lifter its gripped unit as its destination, which ends the grip) and claim 12 (row 2 and its exception: the grip ends and the lifter's target is cleared)
    crate::lift::lifter_accepts(w, id);
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
    // spec: S-0215 claim 37 (c) (no target: nothing)
    let Some(t) = o.target else { return };
    // spec: S-0215 claim 37 (c) (revision 88: a seized unit whose target is its own lifter takes the step as for any target; a destination it gives has no effect, S-0607 claim 9)
    // spec: S-0215 claim 37 (h) and S-0210 claim 11 (before the other parts: a crushing approach takes the target object itself as destination, and nothing else of the step happens)
    if crushing_approach(w, id) {
        if let Some(ob) = w.obj_mut(id) {
            ob.dest = Some(t);
        }
        part_h_accepted(w, id);
        return;
    }
    // spec: S-0215 claim 37 (h) and S-0307 claim 18 (before the other parts: a balloon-hover vehicle whose weapon chosen for the target has a vertical projectile takes the target as its destination, keeps it, and does nothing else)
    if crate::combat::range::balloon_hover_approach(w, id) {
        if let Some(ob) = w.obj_mut(id) {
            ob.dest = Some(t);
        }
        part_h_accepted(w, id);
        return;
    }
    // spec: S-0215 claim 37 (c) (revision 88, Q-2113: an actor that grips a unit with a magnetic beam and whose target is that unit: nothing; part h is decided before this) and S-0607 claim 11 (revision 18: in every behaviour; the range test, minimum range included, gives no destination towards or away from it, and the keep-or-drop of part d does not run)
    if crate::lift::gripped_unit(w, id).is_some_and(|g| t == Target::Object(g)) {
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
        // spec: S-0215 claim 37 (d) (may approach: the can-approach property, and not draining, not sitting in a bunker, S-0606 claim 9; the open-topped passenger exclusion is a seam)
        let may = u.is_none_or(|u| u.raw.can_approach_target.unwrap_or(true))
            && !crate::structures::bunker::bunkered(w, id)
            && !draining(w, id);
        // spec: S-0215 claim 37 (d) (the keep-or-drop table by current behaviour, may approach and owner)
        let keep = match o.beh.seen() {
            Some(Behaviour::Sticky) => false,
            Some(Behaviour::Attack) => true,
            Some(Behaviour::AreaGuard) => may || human,
            Some(Behaviour::Hunt) => may || !human,
            _ => may,
        };
        if !keep {
            // spec: S-0215 claim 37 (d) (dropping clears the target and the destination at once; nothing else of the step happens)
            // spec: S-0215 claim 37 (c) (revision 104: the drop gives a gripped unit no destination, so while it has one its grip ends, S-0607 claim 9; `lift::steer` and `lift::enforce` read the empty destination)
            if let Some(ob) = w.obj_mut(id) {
                crate::spawn::clear_own_target(ob);
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
                // spec: S-0215 claim 37 (e) (revision 104: no effective clear in S-0502 claim 24's sense, so no jumpjet stop follows)
                // spec: S-0215 claim 37 (e) and S-0607 claim 9 (revision 33, Q-2151: part e removes the destination directly, not by an assignment of an empty destination: no effective clear in S-0502 claim 24's sense and not claim 9's exception, so a gripped unit's grip holds)
                crate::lift::destination_removed(w, id);
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
    let tc = match t {
        Target::Object(x) => match w.obj(x) {
            Some(x) => x.pos.cell(),
            None => return,
        },
        Target::Cell(c) => c,
    };
    if reaches(w, id, tc) {
        return;
    }
    step(w, id);
    let Some(o) = w.obj(id) else { return };
    // spec: S-0215 claim 37 (a) (still no destination and the target out of range: the target is dropped) and S-0607 claim 11 (revision 18: unchanged for a lifter whose target is its gripped unit, after its empty approach step; the grip stays, claim 12 having no row for a target change, and the wave ends, claim 21)
    if o.dest.is_none()
        && let Some(t) = o.target
        && !in_range(w, id, t)
        && let Some(ob) = w.obj_mut(id)
    {
        crate::spawn::clear_own_target(ob);
        ob.combat.burst = 0; // spec: S-0301 claim 4 (a cleared target resets the burst counter)
    }
}

/// The cell that stands for the actor in part a1's comparison (S-0215
/// claim 37 (a1)): the cell of the point its locomotor is moving it towards
/// while there is one, else its own cell. For infantry that point is its
/// walking target (its scatter reference point, S-0208 Terms); for a
/// vehicle its heading point (S-0207 Terms for driving, S-0503 Terms for
/// hovering). Seam: no locomotor of ours travels through a tunnel (S-0212
/// Terms), so the tunnel exit's cell never stands for it; the heading
/// points of other vehicle locomotors have no state, so their own cell does.
fn a1_cell(w: &World, id: ObjectId) -> Option<glasnost_map::CellXY> {
    let o = w.obj(id)?;
    match o.kind {
        // spec: S-0215 claim 37 (a1) (for a walking infantry unit the point is its walking target, not the end of its walk: its scatter reference point, S-0208 Terms)
        Kind::Infantry => crate::scatter::infantry_centre_cell(w, id),
        // spec: S-0215 claim 37 (a1) (for a vehicle its heading point as its movement spec gives it, S-0207 Terms for driving, S-0503 Terms for hovering; its own cell while it has no movement in progress)
        _ => crate::scatter::vehicle_centre_cell(w, id),
    }
}

/// Whether the actor can reach its target's cell `tc` (S-0215 claim 37
/// (a1), Q-2171 question 4): S-0212 claim 6's comparison made with the actor
/// in place of the unit and `tc` in place of the destination cell, skipped
/// in claim 6's exceptions.
pub fn reaches(w: &World, id: ObjectId, tc: glasnost_map::CellXY) -> bool {
    let Some(o) = w.obj(id) else { return true };
    // spec: S-0215 claim 37 (a1) and S-0212 claim 6 (first exception: a type with no movement zone reaches every cell; departure D1, never in our engine)
    let Some(u) = w.rules().unit(o.kind, o.type_index) else {
        return true;
    };
    let Some(from) = a1_cell(w, id) else {
        return true;
    };
    let inside = w.cells.usable(from);
    // spec: S-0215 claim 37 (a1) and S-0212 claim 6 (second exception: the actor's cell outside the usable area but a legal cell)
    if !inside && w.cells.cell(from).is_some() {
        return true;
    }
    // spec: S-0215 claim 37 (a1) and S-0212 claim 14 (the corridor-disabling state: a train or current behaviour retreat; no unit is a loaner or a team member here)
    let disabling = u.train || o.beh.seen() == Some(Behaviour::Retreat);
    // spec: S-0215 claim 37 (a1) and S-0212 claim 6 (third exception: in the corridor-disabling state, the actor's cell inside the usable area and the target's cell a legal cell outside it)
    if disabling && inside && w.cells.cell(tc).is_some() && !w.cells.usable(tc) {
        return true;
    }
    // spec: S-0215 claim 37 (a1) (the actor's cell read for its deck when it is on a bridge deck, the target's cell whenever it carries a high bridge, whichever layer the target is on; two cells of no reach region, or two off-map cells, count as one region)
    crate::regions::compare(
        w,
        u.zone,
        from,
        o.layer == crate::world::Layer::Deck,
        tc,
        w.high_bridge(tc),
    )
}

/// Whether the actor is draining another object (S-0215 claim 37 (d);
/// S-0608 claim 6: a draining drainer takes no ordinary approach step).
fn draining(w: &World, id: ObjectId) -> bool {
    // spec: S-0215 claim 37 (d) and S-0608 claim 6 (draining: the drainer of a drain link)
    crate::drain::draining(w, id)
}
