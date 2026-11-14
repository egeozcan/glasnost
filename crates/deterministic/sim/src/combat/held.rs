//! A held target that leaves play (S-0309 claims 27 and 28): every holder
//! of an object as its target drops it when the object is destroyed, goes
//! off the map or is removed, inside that action; each drop resets the
//! holder's targeting timer; a revert can give the object back, and our
//! engine's design rule for such a target (claim 27, DECIDED). Also the
//! periodic held-target check of claim 29: a held target that the firing
//! check refuses is dropped in a check tick.

use crate::combat::check::Answer;
use crate::rules::{Behaviour, Kind};
use crate::world::{HouseId, HouseRole, Object, ObjectId, Target, World};

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
    let mut reverted_holders = Vec::new();
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
            reverted_holders.push(x);
            // spec: S-0301 claim 5, Moment of a retarget's let-go (revision 67, Q-2246: at the change of the firer's target)
            crate::emitters::retarget_let_go(w, x);
            note_revert(w, x);
        }
        let Some(ob) = w.obj_mut(x) else { continue };
        if how == Leaving::Removed && ob.target == Some(Target::Object(gone)) {
            // spec: S-0309 claim 27 (the removal drops a restored object again; together the drops take at most one timer draw)
            crate::spawn::clear_own_target(ob);
        }
        ob.combat.burst = 0; // spec: S-0301 claim 4 (a cleared target resets the burst counter)
        // spec: S-0215 r118 claim 11 (Other endings: a change of target because the target leaves play ends the attack state when the target leaves, so no start after it sees the state) and S-0107 r40 claim 11 (the change asks for deployed, prone or stand); the drop is the change even where a revert gives the same object back
        crate::infantry::target_changed(w, x);
        if let Some(before) = restored {
            // spec: S-0107 claim 15 (a destination brought back is assigned: the destination-driven get-up)
            crate::infantry::destination_assigned(w, x, before);
        }
    }
    // spec: S-0501 claim 53 (r183, Q-3041: an aircraft loses the object as its destination at the moments claim 32 gives for the loss of a target, the killing hit, going off the map and removal; a revert comes first, in the same moment, before the loss clears the object as destination)
    crate::air::destination_lost(w, gone);
    // spec: S-0309 claim 27a (r140, Q-3061, Order within the action: a put-aside destination that is the object, which the revert of claim 27's drop gave back, is lost from the destination later in the same action, so the holder is left with no destination; an aircraft's was cleared by claim 53's loss just above)
    for x in reverted_holders {
        if w.obj(x)
            .is_some_and(|o| o.dest == Some(Target::Object(gone)))
        {
            crate::econ::assign_destination(w, x, None);
        }
    }
    // spec: S-0309 claim 27a (r140, Q-3061: then every holder whose override goes on loses the object from its put-aside target and destination; no exception)
    aside_lost(w, gone, |_, _| false);
    // spec: S-0309 claim 23 (d) and claim 27a (r140, Q-3061: the loss of a remembered target, at the killing hit, going off the map or removal, turns the memory in the same action into a remembered destination at the cell of the target's position then; the engaged mark is kept)
    if let Some(cell) = w.obj(gone).map(|o| o.pos.cell()) {
        let remembered = crate::world::AttackMoveMemory::Target(Target::Object(gone));
        for ob in w.objects.values_mut() {
            if ob.attack_move == Some(remembered) {
                ob.attack_move = Some(crate::world::AttackMoveMemory::Dest(Target::Cell(cell)));
            }
        }
    }
}

/// Every holder whose override is in effect (S-0215 claim 10) loses `gone`
/// from its put-aside slots (S-0309 claim 27a): a put-aside target that is
/// `gone` unless `keep_target` says the holder's house keeps it, and a
/// put-aside destination that is `gone` always. No draw, no revert: the
/// override stays in effect. Slots of another object or a cell are kept.
pub(crate) fn aside_lost(
    w: &mut World,
    gone: ObjectId,
    keep_target: impl Fn(&World, Option<HouseId>) -> bool,
) {
    let it = Some(Target::Object(gone));
    // spec: S-0309 claim 27a (r140, Q-3061: every holder, in key order; a slot that names another object or a cell is kept)
    let changes: Vec<(ObjectId, (Option<Target>, Option<Target>))> = w
        .objects
        .values()
        .filter_map(|o| {
            let (t, d) = o.beh.aside?;
            if t != it && d != it {
                return None;
            }
            // spec: S-0309 claim 27a (r140: at claim 28's moments a put-aside target is kept when the holder's house senses the object's cell; a put-aside destination that is the object is lost whatever senses it)
            let t = if t == it && !keep_target(w, o.owner) {
                None
            } else {
                t
            };
            let d = if d == it { None } else { d };
            Some((o.id, (t, d)))
        })
        .collect();
    for (x, slots) in changes {
        if let Some(ob) = w.obj_mut(x) {
            ob.beh.aside = Some(slots);
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
/// handling, as our own rule rather than claim 27's. An infantry or vehicle
/// holder whose temporal beam is attached to the departed member is not
/// that open case: S-0513 r89 claim 39a (Q-2317 answered) keeps the member
/// as its target ([`beam_held`]). A member that a revert gives back while
/// it is off the map is claim 27's case.
fn left(w: &World, x: ObjectId) -> bool {
    w.obj(x)
        .is_some_and(|o| !o.on_map || crate::combat::damage::dead(o))
}

/// The target a holder's firing step uses (S-0309 claim 27, DECIDED): a
/// held object that is off the map, or destroyed and still in play, counts
/// as no target, so no firing check is asked about it and no shot is fired
/// at it or at its place.
pub fn firing_target(w: &World, o: &Object) -> Option<Target> {
    // spec: S-0309 claim 27 (DECIDED, Q-2006 question 3: a restored object off the map or destroyed in play is no target in the firing step); S-0513 r89 claim 39a (The firer, Q-2317 answered: an infantry or vehicle firer whose beam is attached to a departed member keeps it as its target, its firing check answering rearming, its range and aim reading the old position) and S-0309 r95 claim 27 (while the member waits to land such a firer within range keeps it, makes no shot at it and neither moves nor turns for it); S-0513 r91 claim 39a (Q-2376 answered: a building firer without the SAM flag keeps it too, and its attack processing treats it as a target, not as none, so it takes S-0215 claim 32's rearming row and not the no-target row; an infantry or vehicle firer beyond range of the old position keeps it as for a target on the map there) and r91's Changes for S-0309 claim 27; a SAM-flag building reads its target in its own rows (`behave::sam_attack`); an aircraft firer: the aircraft specs' (S-0513 claim 39a), no target here meanwhile as our own rule
    o.target.filter(|t| match *t {
        Target::Object(x) => {
            !left(w, x)
                || (matches!(o.kind, Kind::Infantry | Kind::Vehicle | Kind::Building)
                    && beam_held(w, o, x))
        }
        Target::Cell(_) => true,
    })
}

/// Whether `holder`'s temporal beam is attached to `x`, a departed warp
/// member (S-0513 claim 39a, *The firer*), and no revert gave `x` back to
/// it while `x` was off the map ([`note_revert`]).
fn beam_held(w: &World, holder: &Object, x: ObjectId) -> bool {
    // spec: S-0513 r108 claim 39a (The firer: a holder that holds the member again after the lock holds it as one that held it at the lock does, unless a revert gave it back while the member is off the map, which is the case of S-0309 claim 27's rule) and S-0309 r113 claim 27 (a member that a revert gives back to a holder while it is off the map is such a target)
    crate::superweapons::warp::departed(w, x)
        && crate::temporal::holds(w, holder.id, x)
        && holder.combat.revert_gave_departed != Some(x)
}

/// After a revert (S-0215 claim 10) gave `holder` its target back: marks a
/// superweapon warp member that is off the map then as S-0309 claim 27's
/// case, which the beam exception of S-0513 claim 39a does not cover. Every
/// revert site calls it once the target is restored.
pub(crate) fn note_revert(w: &mut World, holder: ObjectId) {
    let Some(o) = w.obj(holder) else { return };
    // spec: S-0309 r113 claim 27 (a member that a revert gives back to a holder while it is off the map is a target of this claim) and S-0513 r108 claim 39a (unless a revert gave it back while the member is off the map)
    let gave = match o.target {
        Some(Target::Object(x)) if crate::superweapons::warp::departed(w, x) => Some(x),
        _ => None,
    };
    if let Some(ob) = w.obj_mut(holder) {
        ob.combat.revert_gave_departed = gave;
    }
}

/// Whether tick number `t` is a check tick of claim 29: a multiple of the
/// check period, the same ticks for every object (shared phase).
pub fn check_tick(t: i32) -> bool {
    // spec: S-0309 claim 29 (Phase: every object checks in the same ticks, those whose tick number is a multiple of 16)
    t.rem_euclid(CHECK_PERIOD) == 0
}

/// Whether `o` is a holder kind of claims 25 part k and 29: an infantry
/// unit, vehicle, aircraft or building.
fn holder_kind(o: &Object) -> bool {
    // spec: S-0309 claim 29 (every infantry unit, vehicle, aircraft and building that holds a target)
    matches!(
        o.kind,
        Kind::Infantry | Kind::Vehicle | Kind::Aircraft | Kind::Building
    )
}

/// The house whose owner test and alliance claim 25 part k reads for
/// holder `id`: the house that controls the open-topped transport it rides
/// in when that transport is under mind control, else its own owner.
fn deciding_house(w: &World, id: ObjectId) -> Option<HouseId> {
    // spec: S-0309 claim 25 part k (for a passenger of an open-topped transport that is under mind control, both the owner test and the alliance are those of the house that controls the transport)
    crate::combat::acquire::controlled_carrier_house(w, id)
        .unwrap_or_else(|| w.obj(id).and_then(|o| o.owner))
}

/// Whether house `h` is human-controlled for the unit rules (S-0309 Terms,
/// *Houses in our engine* and *Test-only exception*): every player house
/// but a marked test-only computer-controlled one.
fn human_house(w: &World, h: Option<HouseId>) -> bool {
    h.and_then(|h| w.houses.get(usize::from(h.0)))
        // spec: S-0309 Terms (Houses in our engine: a rule for a human-controlled owner applies to every player house, AI houses included; Test-only exception: a marked player house takes the computer-owner rules)
        .is_some_and(|h| h.role == HouseRole::Player && h.human)
}

/// Whether one of claim 25 part k's exceptions lets holder `id` keep the
/// allied target `t`.
fn allied_keep(w: &World, id: ObjectId, t: ObjectId) -> bool {
    let (Some(o), Some(to)) = (w.obj(id), w.obj(t)) else {
        return true;
    };
    // spec: S-0309 claim 25 part k (kept when the holder is berserk, S-0610)
    if crate::psychic::berserk(w, id) {
        return true;
    }
    let building = to.kind == Kind::Building;
    if o.kind == Kind::Infantry {
        // spec: S-0309 claim 25 part k (kept when the holder is an infantry unit of an engineer type)
        if crate::structures::engineer::is_engineer(w, id) {
            return true;
        }
        // spec: S-0309 claim 25 part k (kept when the holder is an infantry unit, the target a building and the holder can occupy it: the structures specs' test, S-0508 claim 1 and S-0214 row I8)
        if building && crate::structures::garrison::may_enter(w, id, t) {
            return true;
        }
    }
    // spec: S-0309 claim 25 part k (kept when the holder's second weapon is an electric-charge weapon, S-0518, and the target a building of an overpowerable type)
    building
        && crate::linked::overpowerable(w, t)
        && crate::combat::weapon::slot_weapon(w, id, 1)
            .and_then(|x| w.rules().combat.warhead_of(x))
            .is_some_and(|h| h.electric_assault)
}

/// The drop of an allied held target by a holder of a computer-controlled
/// owner (S-0309 claim 25 part k), in holder `id`'s own turn. In our engine
/// those owners are the neutral and special houses and a marked test-only
/// house; a holder of a player house keeps an allied target here. The drop
/// is made in every turn that reaches the check's place, whatever the tick
/// number and whatever the behaviour (*capture* and *sabotage* included),
/// ahead of the check. Like claim 29's drop it takes no draw, causes no
/// revert and changes no behaviour.
pub(crate) fn allied_drop(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id) else { return };
    if !holder_kind(o) {
        return;
    }
    let Some(Target::Object(t)) = o.target else {
        return;
    };
    // spec: S-0309 claim 25 part k (not in a turn cut short, claim 29 Turns cut short: a building being erased; an erased mobile holder's turn ends at its own drop, S-0215 claim 2) and S-0513 claim 39a (a warp member phasing in with a beam attached: as for the check)
    if crate::temporal::cut_short(w, id) {
        return;
    }
    let h = deciding_house(w, id);
    // spec: S-0309 claim 25 part k (a holder of a human-controlled owner has no such drop: in our engine the holders of every player house keep an allied target)
    if human_house(w, h) {
        return;
    }
    // spec: S-0309 claim 25 part k (a target allied with the holder, its own house included, S-0209 Terms)
    if !w.allied(h, w.obj(t).and_then(|x| x.owner)) || allied_keep(w, id, t) {
        return;
    }
    if let Some(ob) = w.obj_mut(id) {
        // spec: S-0309 claim 25 part k (the drop has claim 29's consequences: claim 24's for a setting that leaves no target, no timer draw, no revert and no change of behaviour by itself)
        crate::spawn::clear_own_target(ob);
        ob.combat.burst = 0; // spec: S-0301 claim 4 (a cleared target resets the burst counter)
    }
}

/// The held-target work at claim 29's place in holder `id`'s own turn
/// (S-0309 claim 29, *Place in the turn*): claim 25 part k's allied drop,
/// then the periodic check. The callers place it after the turn's first
/// per-tick start (an infantry unit, vehicle or building: S-0215 claim 11)
/// and after a falling holder's fall step and landing hit (S-0511 claim
/// 10), before the behaviour processing, its searches, the per-tick search
/// (S-0215 claim 33), the movement and the firing step, and before a
/// building's pending delayed shot counts down; an aircraft, whose per-tick
/// start follows its processing and movement, before its processing. A
/// warp turn (S-0513 claim 51, S-0203 claim 4) and a turn cut short never
/// reach it.
pub(crate) fn turn(w: &mut World, id: ObjectId) {
    // spec: S-0309 claim 25 part k (at claim 29's place in the turn, ahead of the check where one is made)
    allied_drop(w, id);
    periodic_check(w, id);
}

/// The periodic held-target check (S-0309 claim 29), in holder `id`'s own
/// turn. In a check tick, an infantry unit, vehicle, aircraft or building
/// of any house, in any behaviour but *capture* and *sabotage*, asks the
/// firing check about its target, without the range test, for the weapon
/// it would use against it; on *illegal* or *impossible* it drops the
/// target. The check reads only the held target, so it is the same
/// whatever gave it (an order, a search, retaliation or a revert). The
/// drop takes no targeting-timer draw, causes no revert and changes no
/// behaviour; a spawner gets no exception (claim 4's keep on *impossible*
/// ends here). Its place in the turn is [`turn`]'s. A holder that takes a
/// second turn in a check tick (S-0203 claim 11) checks in that turn too.
fn periodic_check(w: &mut World, id: ObjectId) {
    if !check_tick(w.tick) {
        return;
    }
    let Some(o) = w.obj(id) else { return };
    if !holder_kind(o) {
        return;
    }
    let Some(t) = o.target else { return };
    // spec: S-0309 claim 29 (Behaviours: a holder whose current behaviour is capture or sabotage makes no check; every other behaviour makes it, those in which no weapon is aimed at the held target included; the behaviour current when the check is made counts) and S-0215 claim 1 (with no current behaviour, rules that ask for its behaviour see the queued one)
    if matches!(o.beh.seen(), Some(Behaviour::Capture | Behaviour::Sabotage)) {
        return;
    }
    // spec: S-0309 claim 29 (Turns cut short: a building being erased makes no check; an infantry unit, vehicle or landed aircraft being erased, a materialising one with a temporal beam locked on it included, drops its target instead in each of its turns, S-0215 claim 2, and that turn ends there with no check)
    // spec: S-0513 claim 39a (a warp member phasing in with a beam attached has its turns cut short as a vehicle or aircraft, and as an infantry unit only while being erased or with quirks.phase_in_beam_spares_infantry off; otherwise it takes the turns of a phasing-in unit with no beam attached)
    if crate::temporal::cut_short(w, id) {
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
    let Some(o) = w.obj(id) else { return };
    let target = o.target;
    let kept = match target {
        Some(Target::Object(x)) => !left(w, x) || beam_held(w, o, x),
        _ => true,
    };
    // spec: S-0309 claim 27 (the revert's case lasts until the start of the holder's next turn, where the drop is made)
    if let Some(ob) = w.obj_mut(id) {
        ob.combat.revert_gave_departed = None;
    }
    // spec: S-0513 r89 claim 39a (The firer, Q-2317 answered, no longer an interim: an infantry or vehicle firer whose beam is attached to the member, within range of its old position, keeps it as its target from the departure to the appearance; Release: the warp is no release cause) and S-0514 claim 14; S-0309 r113 claim 27 (the departure is not this drop for a firer whose temporal beam is attached, a building firer that holds the member at the lock or again after it included, unless a revert gave it back while the member is off the map); S-0513 r108 claim 39a (a building firer, with or without the SAM flag, keeps it while it waits to land as its rows give, and an infantry or vehicle firer beyond its weapon's range of the old position is treated as one holding a target on the map there); an aircraft firer: the aircraft specs' (S-0513 claim 39a), kept here meanwhile as our own rule
    if kept {
        return;
    }
    if let Some(ob) = w.obj_mut(id) {
        // spec: S-0309 claim 27 (DECIDED: dropped at the start of the holder's next turn, no draw, no revert); S-0513 r62 claim 39 (a departed warp member that no revert gave back is not claim 27's case, and what the holder's own processing does with it is open: interim Q-2211 question 3, this same drop as our own rule, a holder's own processing, never the member's step, with no draw over 4 to 8 ticks and no revert, as claim 39 gives for a drop by the holder's own processing)
        crate::spawn::clear_own_target(ob);
        ob.combat.burst = 0; // spec: S-0301 claim 4 (a cleared target resets the burst counter)
    }
}
