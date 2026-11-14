//! The building slot model (S-0108 Terms and claims 12 and 25): the
//! *production* and *special-1* to *special-3* slot animations of a repair
//! depot, including a naval yard with the repair flag (S-0108 claim 20;
//! S-0314 claims 6 and 22; S-0504 claim 8), and the docking, payment and
//! unloading animations (*pre-production*, *special-1* and *production*) of
//! the building beside an unloading harvester (S-0402 claim 11), as
//! deterministic simulation state.
//!
//! Each running slot animation is an animation object of the update order
//! (S-0109 claims 5 and 29): created at the repair-cycle or unloading event
//! that starts it, with its creation draws on the scenario stream (S-0109 claim 28,
//! S-0202 claim 10), first updated later in that turn pass after the
//! objects already in the order, and played by S-0109's stage clock with
//! its duration from the art data. Visibility and drawing never change it.
//!
//! The depot's *idle* and *active-1* slots are presentation: the rows that
//! start or stop them report [`Effect::DepotSlot`] for the drawing, which
//! plays them itself.
//!
//! Not modelled here: S-0108 claim 13's power rules and claim 14's
//! special-outage slot (no slot of this model freezes or starts with
//! power: an unpowered refinery's unloading animation plays on, where
//! claim 13 freezes it and so holds the harvester), and the roles a type
//! may combine with the repair flag (delayed fire, silo fill, recycling,
//! bunker), whose own starts are the client's or absent.

use std::collections::BTreeMap;

use glasnost_num::F64;
use serde::{Deserialize, Serialize};

use crate::anim_clock::{AnimQuirks, Player};
use crate::combat::effects::Effect;
use crate::depot::DepotSlot;
use crate::rules::{Behaviour, Kind};
use crate::world::{ObjectId, World};

/// A running slot animation of a repair depot.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SlotAnim {
    /// The depot.
    pub depot: ObjectId,
    /// Its slot.
    pub slot: DepotSlot,
    /// The current type by registration index (`None` after a follow-up
    /// switch to a name no type is registered under).
    pub anim: Option<u32>,
    /// Its stage clock.
    pub player: Player,
}

/// The depots' slot animations in play, by number in the update order.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DepotAnimWorld {
    /// The running slot animations.
    pub anims: BTreeMap<ObjectId, SlotAnim>,
}

/// The art base key of a slot (S-0108 Import mapping).
fn base(slot: DepotSlot) -> &'static str {
    match slot {
        DepotSlot::PreProduction => "PreProductionAnim",
        DepotSlot::Production => "ProductionAnim",
        DepotSlot::Special1 => "SpecialAnim",
        DepotSlot::Special2 => "SpecialAnimTwo",
        DepotSlot::Special3 => "SpecialAnimThree",
    }
}

/// The name a start of `slot` uses in condition `damaged` (S-0108 Terms,
/// *variant choice*: the damaged name when damaged, the normal name
/// otherwise; never the occupied one here).
fn name_of(w: &World, k: ObjectId, slot: DepotSlot, damaged: bool) -> Option<String> {
    let o = w.obj(k)?;
    let r = w.rules().buildings.get(o.type_index as usize)?;
    let pair = r.raw.econ.slot_names.as_ref()?.get(base(slot))?;
    let name = &pair[usize::from(damaged)];
    // spec: S-0108 Terms (nothing happens when the chosen name is empty)
    (!name.is_empty()).then(|| name.clone())
}

/// Whether depot `k` is damaged for its slots (S-0108 F3): its health
/// ratio, in binary64, at or below the yellow ratio.
fn damaged(w: &World, k: ObjectId) -> bool {
    let Some(o) = w.obj(k) else { return false };
    let r = w.rules();
    let strength = crate::combat::damage::strength(w, k);
    if strength == 0 {
        return false;
    }
    let mode = r.mode;
    // spec: S-0108 F3 (h = health / strength in binary64, damaged when h ≤ the yellow ratio)
    F64::from_i64(i64::from(o.health), mode)
        .div(F64::from_i64(i64::from(strength), mode), mode)
        .le(r.general.condition_yellow)
}

/// Whether depot `k`'s `slot` holds a running animation.
pub fn running(w: &World, k: ObjectId, slot: DepotSlot) -> bool {
    w.obj(k).is_some_and(|o| o.depot.slots.contains_key(&slot))
}

/// Creates the animation of type `name` for depot `k`'s `slot`, keeping
/// stage `from` when it replaces a running one (S-0108 claim 25). `None`
/// when the name names no animation type (S-0108 Terms: nothing happens).
fn create(
    w: &mut World,
    k: ObjectId,
    slot: DepotSlot,
    name: &str,
    from: Option<i32>,
) -> Option<ObjectId> {
    let rules = w.rules_arc();
    // spec: S-0108 Terms (the names of an animation type are looked up when a slot starts; one that names no animation type starts nothing)
    let index = rules
        .animations
        .iter()
        .position(|a| a.name.eq_ignore_ascii_case(name))?;
    let a = &rules.animations[index];
    let pos = w.obj(k)?.pos;
    // spec: S-0109 claim 29 and S-0202 claim 10 (an animation's creation and start-event draws at the creation, on the scenario stream, claim 28)
    let drawn = crate::load::anim_created(w, &a.name, Some((pos.x, pos.y, pos.z)), true);
    let play = crate::mutation::playback(w, a);
    let id = w.new_id();
    let aq = AnimQuirks::from(&w.rules().quirks);
    let t = w.tick;
    let anim = u32::try_from(index).ok();
    let mut env = crate::mutation::SimEnv {
        w: &mut *w,
        pos,
        anim,
    };
    // spec: S-0109 claims 1 and 7 (no loop delay, repeat factor 1, no reverse request)
    let mut player = Player::new(a.name.as_bytes(), play, t, drawn, &mut env, &aq);
    if let Some(k) = from {
        // spec: S-0108 claim 25 (a running slot is replaced: the new animation keeps the replaced one's stage, and takes its passes afresh)
        player.keep_stage(k);
    }
    w.depot_anims.anims.insert(
        id,
        SlotAnim {
            depot: k,
            slot,
            anim,
            player,
        },
    );
    // spec: S-0109 claims 5 and 29 (created during the turn pass, its first turn comes later in that pass, after every object already in the order)
    w.update.enter(id);
    Some(id)
}

/// Removes animation `id` from play (S-0108 Terms, *stopping a slot*;
/// S-0203 claim 10).
fn remove(w: &mut World, id: ObjectId) {
    w.depot_anims.anims.remove(&id);
    let skip = w.rules().quirks.update_order_removal_skip;
    w.update.leave(id, skip);
}

/// Starts depot `k`'s `slot` in its normal or damaged variant (S-0108
/// Terms, *starting a slot*; claims 12 and 20).
pub fn start(w: &mut World, k: ObjectId, slot: DepotSlot) {
    let d = damaged(w, k);
    if w.obj(k).is_some_and(|o| o.depot.slot_damaged != d) {
        // spec: S-0108 claim 12 (a start in the condition opposite to the slot condition swaps every running slot first)
        swap(w, k, d);
    }
    let Some(name) = name_of(w, k, slot, d) else {
        return;
    };
    let old = w.obj(k).and_then(|o| o.depot.slots.get(&slot).copied());
    let from = old
        .and_then(|a| w.depot_anims.anims.get(&a))
        .map(|a| a.player.stage());
    let Some(id) = create(w, k, slot, &name, from) else {
        return;
    };
    if let Some(a) = old {
        remove(w, a);
    }
    if let Some(o) = w.obj_mut(k) {
        o.depot.slots.insert(slot, id);
    }
}

/// Stops depot `k`'s `slot` (S-0108 Terms, *stopping a slot*): no
/// follow-up row reacts.
pub fn stop(w: &mut World, k: ObjectId, slot: DepotSlot) {
    let Some(id) = w.obj_mut(k).and_then(|o| o.depot.slots.remove(&slot)) else {
        return;
    };
    remove(w, id);
}

/// The condition swap of S-0108 claim 12 for depot `k`'s modelled slots:
/// the slot condition becomes `damaged`, and every running slot restarts
/// with its name for it, keeping its stage (claim 25), in claim 12's fixed
/// slot order; a slot whose name for it is empty keeps its animation.
fn swap(w: &mut World, k: ObjectId, damaged: bool) {
    let Some(o) = w.obj_mut(k) else { return };
    o.depot.slot_damaged = damaged;
    let slots: Vec<(DepotSlot, ObjectId)> = o.depot.slots.iter().map(|(s, a)| (*s, *a)).collect();
    // spec: S-0108 claim 12 (the running slots are restarted one after another in the fixed order upgrade-1 … idle, of which production comes before special-1 to special-3: DepotSlot's order; each takes its creation draws as it is restarted)
    for (slot, old) in slots {
        // spec: S-0108 claim 12 (the damaged name when now damaged, the normal name when not; a slot whose name for the new condition is empty keeps its animation)
        let Some(name) = name_of(w, k, slot, damaged) else {
            continue;
        };
        let from = w.depot_anims.anims.get(&old).map(|a| a.player.stage());
        if let Some(id) = create(w, k, slot, &name, from) {
            remove(w, old);
            if let Some(o) = w.obj_mut(k) {
                o.depot.slots.insert(slot, id);
            }
        }
    }
}

/// Reports a start or stop of depot `k`'s presentation slot (S-0108 claim
/// 20: *idle* or *active-1*, normal or damaged variant).
fn report(w: &mut World, k: ObjectId, active1: bool, start: bool) {
    let damaged = damaged(w, k);
    w.effects.push(Effect::DepotSlot {
        building: k,
        active1,
        start,
        damaged,
    });
}

/// Whether depot `k` is in contact (S-0108 claim 20): it holds a contact
/// with a vehicle in any of its contact places, tethered or not.
fn in_contact(w: &World, k: ObjectId) -> bool {
    crate::depot::in_contact(w, k)
}

/// Whether depot `k` is in its *repair* behaviour.
fn in_repair(w: &World, k: ObjectId) -> bool {
    w.obj(k)
        .is_some_and(|o| o.beh.current == Some(Behaviour::Repair))
}

/// The repair cycle's working entry (S-0108 claim 20): *special-1* starts,
/// replacing a running one; *active-1* and *idle* stop.
pub fn working_entered(w: &mut World, k: ObjectId) {
    // spec: S-0108 claim 20 (the settling step answers more needed or done: special-1 starts, a running one replaced; active-1 and idle stop)
    start(w, k, DepotSlot::Special1);
    report(w, k, true, false);
    report(w, k, false, false);
}

/// The repair cycle's end of service (S-0108 claim 20): a working step
/// answers *done*, *not needed* or *cannot pay*, or the depot is no longer
/// in contact in waiting, in working, or in settling with *production* or
/// *special-2* running: *production* and *special-2* stop; *special-3* and
/// *active-1* start.
pub fn service_ended(w: &mut World, k: ObjectId) {
    // spec: S-0108 claim 20 (production and special-2 stop; special-3 and active-1 start, normal or damaged variant, never occupied)
    stop(w, k, DepotSlot::Production);
    stop(w, k, DepotSlot::Special2);
    start(w, k, DepotSlot::Special3);
    report(w, k, true, true);
}

/// Whether depot `k`'s *production* or *special-2* runs (S-0108 claim 20:
/// the settling contact-loss row and the every-tick row test it).
pub fn production_or_special2(w: &World, k: ObjectId) -> bool {
    running(w, k, DepotSlot::Production) || running(w, k, DepotSlot::Special2)
}

/// The start row of *unload* (S-0402 claim 11): building `b` in the
/// harvester's neighbouring cell plays its docking animation.
pub(crate) fn docking(w: &mut World, b: ObjectId) {
    // spec: S-0402 Terms and claim 11 (the docking animation is the building's pre-production animation, normal or damaged form; none when its art gives none)
    start(w, b, DepotSlot::PreProduction);
}

/// A payment row of *unload* (S-0402 claim 11): building `b` plays its
/// payment animation unless that is already playing.
pub(crate) fn payment(w: &mut World, b: ObjectId) {
    // spec: S-0402 Terms and claim 11 (the payment animation is the building's special animation, played at a payment unless it is already playing)
    if !running(w, b, DepotSlot::Special1) {
        start(w, b, DepotSlot::Special1);
    }
}

/// The end of the empty-load row of *unload* (S-0402 claim 11), after the
/// row's smoke event and payment animation: a refinery-flag building
/// (`refinery`) plays its unloading animation, then building `b`'s payment
/// animation is stopped.
pub(crate) fn load_emptied(w: &mut World, b: ObjectId, refinery: bool) {
    if refinery {
        // spec: S-0402 Terms and claim 11 (a refinery-flag building plays its unloading animation, its production animation; r84: its draws after the payment animation's)
        start(w, b, DepotSlot::Production);
    }
    // spec: S-0402 claim 11 (r84, Order within the two rows: the payment animation is always stopped last, so one started in this row leaves play in the same processing; stopping takes no draw, S-0108 Terms)
    stop(w, b, DepotSlot::Special1);
}

/// Whether building `b` is playing its unloading animation (S-0402 claim
/// 11's *finishing* row; the caller tests the refinery flag).
pub(crate) fn unloading_playing(w: &World, b: ObjectId) -> bool {
    running(w, b, DepotSlot::Production)
}

/// Building `k`'s every-tick slot checks, in its own turn after its
/// behaviour processing (our placement; S-0108 claims 12 and 20): the
/// condition swap for a depot or a building with a modelled slot running,
/// and the repair depot's every-tick row.
pub fn building_turn(w: &mut World, k: ObjectId) {
    let depot = crate::depot::is_depot(w, k);
    if !depot && w.obj(k).is_none_or(|o| o.depot.slots.is_empty()) {
        return;
    }
    let d = damaged(w, k);
    if w.obj(k).is_some_and(|o| o.depot.slot_damaged != d) {
        // spec: S-0108 claim 12 (the damaged state differs from the slot condition: the swap; here at the building's own turn)
        swap(w, k, d);
    }
    if !depot {
        return;
    }
    let naval = w
        .obj(k)
        .and_then(|o| w.rules().buildings.get(o.type_index as usize))
        .is_some_and(|r| r.naval);
    // spec: S-0108 claim 20 (a repair depot, not naval, not in its repair behaviour, with production or special-2 running, checked every tick: special-3 starts, production and special-2 stop)
    if !naval && !in_repair(w, k) && production_or_special2(w, k) {
        start(w, k, DepotSlot::Special3);
        stop(w, k, DepotSlot::Production);
        stop(w, k, DepotSlot::Special2);
    }
}

/// A depot slot animation's turn (S-0109 claims 5, 15 and 16): it plays;
/// at its end it leaves play and S-0108 claim 20's follow-up rows run.
pub fn anim_turn(w: &mut World, id: ObjectId) {
    let Some(mut a) = w.depot_anims.anims.remove(&id) else {
        return;
    };
    let k = a.depot;
    let skip = w.rules().quirks.update_order_removal_skip;
    let held = w.obj(k).is_some_and(|o| {
        o.kind == Kind::Building && o.on_map && o.depot.slots.get(&a.slot) == Some(&id)
    });
    if !held {
        // Its building left play: the animation goes with it (S-0108 Terms, removal of the building).
        w.update.leave(id, skip);
        return;
    }
    let aq = AnimQuirks::from(&w.rules().quirks);
    let t = w.tick;
    let pos = w.obj(k).map(|o| o.pos).unwrap_or_default();
    let mut env = crate::mutation::SimEnv {
        w: &mut *w,
        pos,
        anim: a.anim,
    };
    let playing = a.player.update(t, &mut env, &aq);
    a.anim = env.anim;
    if playing {
        w.depot_anims.anims.insert(id, a);
        return;
    }
    // spec: S-0109 claim 16 (the last pass ends with no follow-up type: removed); S-0203 claim 10 (a removal from the update order)
    w.update.leave(id, skip);
    if let Some(o) = w.obj_mut(k) {
        o.depot.slots.remove(&a.slot);
    }
    if !crate::depot::is_depot(w, k) {
        // spec: S-0108 claim 20 (only the listed kinds react to special-1 or special-3 ending; a building beside an unloading harvester is none of them, S-0402 claim 11)
        return;
    }
    match a.slot {
        DepotSlot::Special1 => {
            if in_repair(w, k) && in_contact(w, k) {
                // spec: S-0108 claim 20 (special-1 ends while in the repair behaviour and in contact: special-2 starts, normal or damaged variant)
                start(w, k, DepotSlot::Special2);
            } else {
                // spec: S-0108 claim 20 (special-1 ends while not in the repair behaviour or not in contact: idle starts, normal or damaged variant)
                report(w, k, false, true);
            }
        }
        // spec: S-0108 claim 20 (special-3 finishes on its own: idle starts, normal or damaged variant)
        DepotSlot::Special3 => report(w, k, false, true),
        // spec: S-0108 claim 20 (no row reacts to production or special-2 ending); a depot never starts pre-production
        DepotSlot::PreProduction | DepotSlot::Production | DepotSlot::Special2 => {}
    }
}

/// A running modelled slot animation as the drawing shows it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SlotDraw {
    /// The slot.
    pub slot: DepotSlot,
    /// The current type's name.
    pub name: String,
    /// Image frame *a* + *k*.
    pub frame: i32,
    /// Stage *k*.
    pub stage: i32,
    /// End field *E* in play.
    pub end: i32,
}

/// The running slot animations of depot `k` for the drawing, in slot
/// order.
pub fn draws(w: &World, k: ObjectId) -> Vec<SlotDraw> {
    let Some(o) = w.obj(k) else {
        return Vec::new();
    };
    o.depot
        .slots
        .iter()
        .filter_map(|(slot, id)| {
            let a = w.depot_anims.anims.get(id)?;
            Some(SlotDraw {
                slot: *slot,
                name: a.player.name_text(),
                frame: a.player.frame(),
                stage: a.player.stage(),
                end: a.player.play.end,
            })
        })
        .collect()
}

/// Test hook: starts depot `k`'s `slot` as the repair cycle would (S-0314
/// claim 6's production hold matters only for a type combining the repair
/// flag with a role that starts *production*, which the simulation does
/// not model at a depot).
#[doc(hidden)]
pub fn start_for_test(w: &mut World, k: ObjectId, slot: DepotSlot) {
    start(w, k, slot);
}
