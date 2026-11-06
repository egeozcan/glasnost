//! Leftovers (S-0203 claim 13): an object removed in a tick is kept to the
//! end of that tick, out of play and off the map, and only the defeat or
//! surrender sweep of S-0414 claim 5 can still reach it.
//!
//! Our engine's form (claim 13 (e)): every infantry unit, vehicle, aircraft
//! and building that `objects::remove` takes out of the game is kept here,
//! as it was when it left, until the start of the next tick. Nothing in the
//! simulation reads this store but the sweep (`econ::victory`), which finds
//! its leftovers at their places in creation order, and the sweep offers its
//! hit only to vehicles that left play at their own death with no health
//! (claim 13 (c)). The arena's recorder lists them, off the map, after the
//! tick has run.

use serde::{Deserialize, Serialize};

use crate::combat::effects::Effect;
use crate::rules::Kind;
use crate::world::{HouseId, Object, ObjectId, World};

/// One leftover.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Leftover {
    /// The object as it left play, off the map.
    pub object: Object,
    /// It left play at its own death (S-0203 claim 6, `death::leave_play`),
    /// not as a passenger dying with its carrier, a crushed or erased
    /// object, or by any other removal.
    pub death: bool,
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
            death: false,
        },
    );
}

/// Marks a leftover as one that left play at its own death (S-0203 claim 6
/// (a), S-0312 claim 15).
pub fn died(w: &mut World, id: ObjectId) {
    if let Some(l) = w.leftovers.get_mut(&id) {
        l.death = true;
    }
}

/// The end of the leftovers of the tick before (S-0203 claim 13 (a)): at the
/// start of a tick, before any of its parts.
pub fn clear(w: &mut World) {
    w.leftovers.clear();
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
/// (S-0203 claim 13 (c)), at its place in creation order. Under
/// `quirks.defeat_sweep_leftovers` it reaches a vehicle of the house that
/// left play at its own death with no health. The hit has no amount, the
/// demolition warhead, ignores defences, has the no-escape marker and
/// neither attacker nor attacking house. It changes no health, registers no
/// destruction (no kill, no loss) and raises no tag; the vehicle's death
/// effects come once more.
pub fn sweep_hit(w: &mut World, id: ObjectId, h: HouseId) {
    // spec: S-0203 claim 13 (c) and Edge cases (quirks.defeat_sweep_leftovers off: the sweep offers no hit to any leftover)
    if !w.rules().quirks.defeat_sweep_leftovers {
        return;
    }
    let Some(l) = w.leftovers.get(&id) else {
        return;
    };
    let o = &l.object;
    // spec: S-0203 claim 13 (c) (our engine offers the hit only to leftover vehicles with no health; one with health left, such as a passenger killed with its transport, and leftovers of other kinds get none) and (b) (the health of none, if damage destroyed it and nothing restored health before its removal)
    if !(l.death && o.kind == Kind::Vehicle && o.health <= 0) {
        return;
    }
    // spec: S-0203 claim 13 (c) and S-0414 claim 5 (the ownership conditions: S-0601 claim 12 took a removed victim out of its controller's list, so a leftover is the house's by its kept owner)
    if o.owner != Some(h) {
        return;
    }
    let o = o.clone();
    let warhead = w.rules().combat.general.c4_warhead;
    let plan = read_restored(w, o.clone(), |w| crate::combat::death::repeat_plan(w, id));
    let Some(plan) = plan else { return };
    if plan.unsettled {
        // Q-2692 (interim: a leftover that meets S-0312 claim 10's sinking or claim 15's crash conditions at the hit is offered no hit, as claim 13 (c) does for the other cases it leaves open)
        return;
    }
    // spec: S-0203 claim 13 (c) (a hit of no amount with the demolition warhead and no attacker; it changes no health)
    w.effects.push(Effect::LeftoverHit {
        id,
        owner: o.owner,
        pos: o.pos,
        warhead,
        health: o.health,
    });
    // spec: S-0203 claim 13 (c) and S-0312 entry of Changes for other specs (the death effects that the vehicle's state allows, once more, with their draws, without the credit and with no tag raise)
    crate::combat::death::repeat(w, id, &plan);
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
