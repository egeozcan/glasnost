//! A held target that leaves play (S-0309 claims 27 and 28): every holder
//! of an object as its target drops it when the object is destroyed, goes
//! off the map or is removed, inside that action; each drop resets the
//! holder's targeting timer; a revert can give the object back, and our
//! engine's design rule for such a target (claim 27, DECIDED). Also the
//! periodic held-target check of claim 29: a held target that the firing
//! check refuses is dropped in a check tick.

use crate::combat::check::Answer;
use crate::rules::{Behaviour, Kind};
use crate::world::{Object, ObjectId, Target, World};

// spec: S-0309 claim 29 and Constants (held-target check period 16 ticks: checks fall in the ticks whose tick number is a multiple of it)
const CHECK_PERIOD: i32 = 16;

// spec: S-0309 claim 27 (a draw only when the targeting timer has more than 10 integer ticks left; S-0501 claim 37 for an aircraft)
const DROP_TIMER_LEFT: i32 = 10;
// spec: S-0309 claim 27 (one ranged draw over 4 to 8 ticks, both included; S-0501 claim 37 for an aircraft)
const DROP_TIMER_DRAW: (i32, i32) = (4, 8);

/// How a held object leaves play (S-0309 claim 27; glossary, *in play …
/// destroyed, removed*).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Leaving {
    /// The killing hit, also of an object that stays in play after it.
    Destroyed,
    /// It goes off the map (a transport, a building entered, a master).
    OffMap,
    /// It is removed (a crush included).
    Removed,
}

/// The drop's timer reset for holder `id` (S-0309 claims 27 and 28, S-0501
/// claim 37): with more than 10 ticks left on its targeting timer, one
/// ranged draw over 4 to 8 ticks restarts it; otherwise no draw.
pub(crate) fn timer_draw(w: &mut World, id: ObjectId) {
    let tick = w.tick;
    let half = w.rules().quirks.ranged_draw_half_span;
    if w.obj(id)
        .is_some_and(|o| o.targeting_timer.wrapping_sub(tick) > DROP_TIMER_LEFT)
    {
        // spec: S-0309 claim 27 and S-0501 claim 37 (the draw, from the scenario random stream, before the target is cleared, inside the action that drops it)
        let v = w.rng.ranged(DROP_TIMER_DRAW.0, DROP_TIMER_DRAW.1, half);
        if let Some(ob) = w.obj_mut(id) {
            ob.targeting_timer = tick.wrapping_add(v);
        }
    }
}

/// Every infantry unit, vehicle, building and aircraft holding `gone` as
/// its target drops it now (S-0309 claim 27): the timer draw, then the
/// clear, and an overridden holder reverts (S-0215 claim 10), which can
/// give it `gone` again. A removal drops such a restored `gone` once more
/// within the same action, with no second draw (after a first draw at most
/// 8 ticks are left). A destroyed object that stays in play, or one off the
/// map, stays the restored target; [`turn_start_drop`] and
/// [`firing_target`] are our engine's rule for it.
pub(crate) fn holders_drop(w: &mut World, gone: ObjectId, how: Leaving) {
    // spec: S-0309 claim 27 (no exception for the holder's owner or for what covers the cell); claim 27 and Terms (several holders draw in creation order, whatever their update order or distances: object keys are given out in creation order and never reused, so key order is creation order)
    let ids: Vec<ObjectId> = w
        .objects
        .values()
        .filter(|o| o.target == Some(Target::Object(gone)))
        .map(|o| o.id)
        .collect();
    for x in ids {
        // spec: S-0309 claim 27 (the drop resets the targeting timer: the draw first, S-0501 claim 37)
        timer_draw(w, x);
        let Some(ob) = w.obj_mut(x) else { continue };
        let mut restored = None;
        let mut reverted = false;
        if ob.kind == Kind::Aircraft && how != Leaving::Destroyed {
            // spec: S-0501 claim 32 (an aircraft holds no target, then an overridden one is reverted right after the loss)
            crate::spawn::clear_own_target(ob);
            if ob.beh.suspended.is_some()
                && let Some((t, d)) = ob.beh.revert()
            {
                // spec: S-0501 claim 32 and S-0215 claim 10 (the revert brings back the target put aside, even the object that left the map)
                crate::spawn::set_own_target(ob, t);
                ob.dest = d;
                reverted = true;
            }
        } else if ob.beh.suspended.is_some()
            && let Some((t, d)) = ob.beh.revert()
        {
            // spec: S-0215 claim 10 and S-0309 claim 27 (the drop ends an override: the revert can make the same object the target again)
            crate::spawn::set_own_target(ob, t);
            if ob.kind.is_mobile() {
                restored = Some(ob.dest);
                ob.dest = d;
            }
            reverted = true;
        } else {
            crate::spawn::clear_own_target(ob);
        }
        if reverted {
            // spec: S-0301 claim 5, Moment of a retarget's let-go (revision 67, Q-2246: at the change of the firer's target)
            crate::emitters::retarget_let_go(w, x);
        }
        let Some(ob) = w.obj_mut(x) else { continue };
        if how == Leaving::Removed && ob.target == Some(Target::Object(gone)) {
            // spec: S-0309 claim 27 (the removal drops a restored object again; together the drops take at most one timer draw)
            crate::spawn::clear_own_target(ob);
        }
        ob.combat.burst = 0; // spec: S-0301 claim 4 (a cleared target resets the burst counter)
        if let Some(before) = restored {
            // spec: S-0107 claim 15 (a destination brought back is assigned: the destination-driven get-up)
            crate::infantry::destination_assigned(w, x, before);
        }
    }
}

/// Whether object `x`, held as a target, is one that only a revert can
/// have given back (S-0309 claim 27, DECIDED): off the map, or destroyed
/// and still in play. A superweapon warp member is also held off the map
/// after its departure, which drops nobody's target and is no revert
/// (S-0513 r62 claim 39). Claim 27's rule does not apply to a holder that
/// holds it through the departure; what the firing check answers for the
/// departed member, and so what that holder's own processing does with it,
/// is open there (Q-2211 question 3). Our interim gives it the same
/// handling, as our own rule rather than claim 27's. A member that a revert
/// gives back while it is off the map is claim 27's case.
fn left(w: &World, x: ObjectId) -> bool {
    w.obj(x)
        .is_some_and(|o| !o.on_map || crate::combat::damage::dead(o))
}

/// The target a holder's firing step uses (S-0309 claim 27, DECIDED): a
/// held object that is off the map, or destroyed and still in play, counts
/// as no target, so no firing check is asked about it and no shot is fired
/// at it or at its place.
pub fn firing_target(w: &World, o: &Object) -> Option<Target> {
    // spec: S-0309 claim 27 (DECIDED, Q-2006 question 3: a restored object off the map or destroyed in play is no target in the firing step); S-0513 r62 claim 39 (claim 27's rule does not apply to a holder of a departed warp member that no revert gave back; what the firing check answers for that member is open: interim Q-2211 question 3, the same handling as our own rule)
    o.target
        .filter(|t| !matches!(*t, Target::Object(x) if left(w, x)))
}

/// Whether claim 29's periodic check covers holder `id` by its owner
/// (S-0309 claim 29): every holder of a human-controlled owner, which in
/// our engine is every player house, AI houses included (S-0309 Terms,
/// *Houses in our engine*). Whether the original applies the check to
/// holders of the neutral and special houses is open (S-0309 Open
/// questions, *Other holders and targets*): interim Q-2263, they are not
/// checked and keep S-0304 claim 2's drop at their attack processing. The
/// check is the same whatever gave the target (an order, a search, a
/// retaliation or a revert: the engine does not record which; the last two
/// are open in the same question, interim Q-2263, checked).
pub fn periodic_covers(w: &World, id: ObjectId) -> bool {
    // spec: S-0309 claim 29 (holders of a human-controlled owner: every player house in our engine) and Terms (Test-only exception: a marked player house takes the computer-owner rules)
    crate::combat::acquire::player_owned(w, id)
}

/// Whether tick number `t` is a check tick of claim 29: a multiple of the
/// check period, the same ticks for every object (shared phase).
pub fn check_tick(t: i32) -> bool {
    // spec: S-0309 claim 29 (Phase: every object checks in the same ticks, those whose tick number is a multiple of 16)
    t.rem_euclid(CHECK_PERIOD) == 0
}

/// The periodic held-target check (S-0309 claim 29), in holder `id`'s own
/// turn. In a check tick, a covered infantry unit, vehicle, aircraft or
/// building whose current behaviour is *attack* or *guard* asks the firing
/// check about its target, without the range test, for the weapon it would
/// use against it; on *illegal* or *impossible* it drops the target. The
/// drop takes no targeting-timer draw, causes no revert and changes no
/// behaviour; a spawner gets no exception (claim 4's keep on *impossible*
/// ends here).
///
/// The caller places it at the start of the turn, after the warp update
/// (S-0513 claim 51 and S-0203 claim 4: a warp member takes no check from
/// its first pending update through its arrival effects; in the arrival
/// hold it comes after the hold-end test) and before the behaviour
/// processing, the searches and the firing step. Where in the turn the
/// check falls against those is open (S-0309 Open questions, *Place of the
/// periodic check*, and *A refusal that begins in the holder's turn*):
/// interim Q-2261, this place. Against a building's pending delayed shot it
/// comes first, as claim 29 requires: that shot still answers *rearming*.
pub(crate) fn periodic_check(w: &mut World, id: ObjectId) {
    if !check_tick(w.tick) {
        return;
    }
    let Some(o) = w.obj(id) else { return };
    // spec: S-0309 claim 29 (every infantry unit, vehicle, aircraft and building)
    if !matches!(
        o.kind,
        Kind::Infantry | Kind::Vehicle | Kind::Aircraft | Kind::Building
    ) {
        return;
    }
    let Some(t) = o.target else { return };
    // spec: S-0309 claim 29 (Behaviours: a holder whose current behaviour is attack or guard makes the check; capture and sabotage make none; the others are open, S-0309 Open questions, Other behaviours: interim Q-2262, no check)
    if !matches!(o.beh.current, Some(Behaviour::Attack | Behaviour::Guard)) {
        return;
    }
    // spec: S-0309 claim 29 (Turns cut short: a building being erased makes no check; an infantry unit, vehicle or landed aircraft being erased drops its target instead in each of its turns, S-0215 claim 2; a materialising holder on which a temporal beam is locked is open, interim Q-2264: being erased, no check)
    // spec: S-0513 claim 39a (a warp member phasing in with a beam attached has its turns cut short as a vehicle or aircraft, and as an infantry unit only while being erased or with quirks.phase_in_beam_spares_infantry off; otherwise it takes the turns of a phasing-in unit with no beam attached)
    if crate::temporal::cut_short(w, id) {
        return;
    }
    if !periodic_covers(w, id) {
        return;
    }
    // spec: S-0309 claim 29 (the weapon it would use against the target, S-0308 and S-0304 claim 6)
    let (slot, _) = crate::combat::weapon::would_use(w, id, Some(t));
    // spec: S-0309 claim 29 (Answers: S-0301 claim 9's, with its ranks, without the range row)
    let answer = crate::combat::check::check(w, id, Some(t), slot, false);
    // spec: S-0309 claim 29 (illegal or impossible drops the target; any other answer leaves it held)
    if !matches!(answer, Answer::Illegal | Answer::Impossible) {
        return;
    }
    if let Some(ob) = w.obj_mut(id) {
        // spec: S-0309 claim 29 (The drop: the target set to none with claim 24's consequences, a spawner's conditional pending-target clear and an airstrike's cancellation through the target change; no targeting-timer draw; no revert, S-0215 claim 10; no behaviour change)
        crate::spawn::clear_own_target(ob);
        ob.combat.burst = 0; // spec: S-0301 claim 4 (a cleared target resets the burst counter)
    }
}

/// The start of a holder's turn (S-0309 claim 27, DECIDED): a held object
/// that is off the map, or destroyed and still in play, is dropped, with no
/// timer draw and no revert (S-0215 claim 10: clearing the target by another
/// rule does not revert).
pub(crate) fn turn_start_drop(w: &mut World, id: ObjectId) {
    let Some(Target::Object(x)) = w.obj(id).and_then(|o| o.target) else {
        return;
    };
    // spec: S-0513 claim 39a (The firer: while its beam is attached, a firer keeps the member at the checks and makes no further shot at it; Release: the warp is no release cause, and a firer in attack approaches a member that lands beyond range) and S-0514 claim 14; interim Q-2317: a holder whose beam is attached to the departed member keeps it
    if !left(w, x)
        || (crate::temporal::holds(w, id, x) && crate::superweapons::warp::departed(w, x))
    {
        return;
    }
    if let Some(ob) = w.obj_mut(id) {
        // spec: S-0309 claim 27 (DECIDED: dropped at the start of the holder's next turn, no draw, no revert); S-0513 r62 claim 39 (a departed warp member that no revert gave back is not claim 27's case, and what the holder's own processing does with it is open: interim Q-2211 question 3, this same drop as our own rule, a holder's own processing, never the member's step, with no draw over 4 to 8 ticks and no revert, as claim 39 gives for a drop by the holder's own processing)
        crate::spawn::clear_own_target(ob);
        ob.combat.burst = 0; // spec: S-0301 claim 4 (a cleared target resets the burst counter)
    }
}
