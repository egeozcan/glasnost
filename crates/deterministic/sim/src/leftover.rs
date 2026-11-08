//! Leftovers (S-0203 claim 13): an object removed in a tick is kept to the
//! end of that tick, out of play and off the map, and only the defeat or
//! surrender sweep of S-0414 claim 5 can still reach it.
//!
//! Our engine's form (claim 13 (e)): every infantry unit, vehicle, aircraft
//! and building that `objects::remove` takes out of the game is kept here,
//! as it was when it left, until the start of the next tick, except a
//! vehicle that a sweep's hit set sinking again and whose type cannot
//! crash, which is kept to the end of the game (claim 13 (g)). Nothing in
//! the simulation reads this store but the sweep (`econ::victory`), which
//! finds every leftover at its place in creation order (claim 13 (h)) and
//! offers its hit to the house's leftover vehicles, with or without health
//! left (claim 13 (c) and (g)). The arena's recorder lists them, off the
//! map, after the tick has run.

use serde::{Deserialize, Serialize};

use crate::combat::damage::Hit;
use crate::combat::effects::Effect;
use crate::rules::Kind;
use crate::world::{HouseId, Object, ObjectId, World};

/// One leftover.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Leftover {
    /// The object as it left play, off the map.
    pub object: Object,
    /// A sweep's hit set it sinking again and its type cannot crash, so it
    /// is kept to the end of the game (S-0203 claim 13 (a) and (g)).
    pub kept: bool,
}

/// Whether an object is of the four kinds a leftover is kept for (S-0203
/// claim 13 (a); the four kinds of S-0414 claim 15).
fn kept_kind(k: Kind) -> bool {
    matches!(
        k,
        Kind::Infantry | Kind::Vehicle | Kind::Aircraft | Kind::Building
    )
}

/// Keeps an object that has just been removed from the game (S-0203 claim
/// 13 (a) and (b)): off the map, with its owner and its health.
pub fn keep(w: &mut World, mut o: Object) {
    if !kept_kind(o.kind) {
        return;
    }
    // spec: S-0203 claim 13 (b) (a leftover is out of play and off the map; it keeps its owner and the health it had when it left play)
    o.on_map = false;
    w.leftovers.insert(
        o.id,
        Leftover {
            object: o,
            kept: false,
        },
    );
}

/// Gives leftover `id` back the health it had before its death as a
/// passenger killed with its transport (S-0203 claim 13 (b): such a
/// passenger keeps its health, S-0312 claims 5 and 12).
pub fn keep_health(w: &mut World, id: ObjectId, health: i32) {
    if let Some(l) = w.leftovers.get_mut(&id) {
        l.object.health = health;
    }
}

/// The end of the leftovers of the tick before (S-0203 claim 13 (a)): at the
/// start of a tick, before any of its parts. A leftover kept to the end of
/// the game stays (claim 13 (g)).
pub fn clear(w: &mut World) {
    w.leftovers.retain(|_, l| l.kept);
}

/// The first leftover at or after `from` in creation order (S-0414 claim 15:
/// object numbers are given in creation order).
pub fn next(w: &World, from: ObjectId) -> Option<ObjectId> {
    w.leftovers.range(from..).next().map(|(&id, _)| id)
}

/// The object a leftover was, for a reader outside the simulation (the
/// arena's recorder).
pub fn object(w: &World, id: ObjectId) -> Option<&Object> {
    w.leftovers.get(&id).map(|l| &l.object)
}

/// The sweep's hit on leftover `id` for defeated or surrendering house `h`
/// (S-0203 claim 13 (c) and (g)), at its place in creation order. Under
/// `quirks.defeat_sweep_leftovers` it reaches every leftover vehicle of the
/// house, whatever removed it, with or without health left. The hit's
/// amount is the leftover's health, with the demolition warhead, ignoring
/// defences, with the no-escape marker and neither attacker nor attacking
/// house. On a leftover with no health it changes no health and registers
/// no destruction; on one with health left it is a killing hit whose
/// destruction is registered once more with no attacker. Either way the
/// vehicle's death effects come once more, as its state where it left play
/// allows.
pub fn sweep_hit(w: &mut World, id: ObjectId, h: HouseId) {
    // spec: S-0203 claim 13 (c) and Edge cases (quirks.defeat_sweep_leftovers off: the sweep offers no hit to any leftover)
    if !w.rules().quirks.defeat_sweep_leftovers {
        return;
    }
    let Some(l) = w.leftovers.get(&id) else {
        return;
    };
    let o = l.object.clone();
    // spec: S-0203 claim 13 (c) (our engine offers the hit to every leftover vehicle of the house, with or without health left, and to no leftover of another kind until its effects are traced) and (f) (whatever removed it, a sweep included)
    if o.kind != Kind::Vehicle {
        return;
    }
    // spec: S-0203 claim 13 (c) and S-0414 claim 5 (the ownership conditions: S-0601 claim 12 took a removed victim out of its controller's list, so a leftover is the house's by its kept owner)
    if o.owner != Some(h) {
        return;
    }
    let warhead = w.rules().combat.general.c4_warhead;
    // spec: S-0203 claim 13 (c) and (g) (the hit's amount is the leftover's current health: no amount on one with none, a killing hit on one with health left)
    let before = o.health.max(0);
    let health_left = before > 0;
    // spec: S-0203 claim 13 (g), Sinking (the vehicle's state where it left play decides, as at any killing hit; one with no health starts no sinking)
    let sinks = health_left && read_restored(w, o, |w| crate::naval::sinks_at_kill(w, id));
    // spec: S-0203 claim 13 (g), Sinking and S-0312 claim 10 (one that sinks has its health back to 1 health point; any other killing hit leaves it none)
    let after = if sinks { 1 } else { 0 };
    w.effects.push(Effect::LeftoverHit {
        id,
        owner: Some(h),
        pos: w.leftovers[&id].object.pos,
        warhead,
        health: (before, after),
    });
    if let Some(l) = w.leftovers.get_mut(&id) {
        l.object.health = after;
    }
    if health_left {
        let hit = Hit {
            amount: before,
            warhead,
            distance: 0,
            attacker: None,
            house: None,
            ignore_defences: true,
            no_escape: true,
        };
        // spec: S-0203 claim 13 (g) (a killing hit: its destruction is registered once more, with no attacker: no kill, and counted as S-0414 claim 21 counts a destruction with no attacker; it is killed, as at any killing hit, S-0302 claim 1)
        with_restored(w, id, |w| {
            crate::combat::damage::credit(w, id, &hit);
            crate::combat::death::mark_killed(w, id);
        });
        // spec: S-0203 claim 13 (g) and S-0011 claim 2 (the raises S-0312 claim 29 gives a killing hit with no attacker and no attacking house come again for a tag the leftover still carries; out of the world, so the actions they fire find nothing of it)
        crate::combat::death::leftover_destruction_raises(w, id);
    }
    let Some(o) = w.leftovers.get(&id).map(|l| l.object.clone()) else {
        return;
    };
    let plan = read_restored(w, o, |w| {
        crate::combat::death::repeat_plan(w, id, health_left)
    });
    let Some(plan) = plan else { return };
    // spec: S-0203 claim 13 (c) and (g) and S-0312 entry of Changes for other specs (the death effects of S-0312 once more, as the vehicle's state where it left play allows, with their draws and the sweep's markers, after the raises)
    crate::combat::death::repeat(w, id, &plan);
    if plan.kept() {
        // spec: S-0203 claim 13 (g), Sinking (marked as sinking, out of play and off the map; it never descends, and a type that cannot crash is never removed: kept to the end of the game); S-0504 Terms (sinking)
        with_restored(w, id, |w| crate::naval::start_sinking(w, id));
        if let Some(l) = w.leftovers.get_mut(&id) {
            l.kept = true;
        }
    }
}

/// Reads `f` with leftover object `o` put back among the world's objects for
/// the read alone, and taken out again before anything acts: `f` gets the
/// world for reading only, so nothing else can run while it is there and
/// nothing else finds it (S-0203 claim 13 (e)). The readers are the death
/// effects' own, which look an object up by its number.
fn read_restored<T>(w: &mut World, o: Object, f: impl FnOnce(&World) -> T) -> T {
    let id = o.id;
    let before = w.objects.insert(id, o);
    let out = f(w);
    w.objects.remove(&id);
    if let Some(b) = before {
        w.objects.insert(id, b);
    }
    out
}

/// Runs `f` with leftover `id` put back among the world's objects and
/// stores the object it leaves in the leftover again (S-0203 claim 13 (e)).
/// For the bookkeeping of the sweep's killing hit alone, whose helpers look
/// the victim up by its number and change only it and the scores: the
/// destruction's registration and the sinking mark. No turn, rule or
/// search runs while it is there.
fn with_restored(w: &mut World, id: ObjectId, f: impl FnOnce(&mut World)) {
    let Some(l) = w.leftovers.get(&id) else {
        return;
    };
    let o = l.object.clone();
    let before = w.objects.insert(id, o);
    f(w);
    let back = w.objects.remove(&id);
    if let Some(b) = before {
        w.objects.insert(id, b);
    }
    if let (Some(b), Some(l)) = (back, w.leftovers.get_mut(&id)) {
        l.object = b;
    }
}
