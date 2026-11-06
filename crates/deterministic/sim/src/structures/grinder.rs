//! Recycling buildings (S-0603): the docking answer (claim 2, F2), the
//! *eaten* processing an entrant gets (claim 3, S-0215 claim 28), the
//! consumption at arrival (claims 5 to 11, F1), the search of S-0601 claim
//! 8's grinder fate (claim 14) and the interruptions (claim 15).
//!
//! No state of its own: an entrant is an infantry unit or vehicle in
//! *eaten* whose destination (or target) is the building. It holds no
//! contact and no link place (claim 3), and the building is never told
//! that it is coming.
//!
//! Not built here: the building's *active-1* / *special-1* slot swap of
//! claim 12 is display only, and our map view starts the set-up slots
//! without a runtime slot state (S-0108 claim 11), so it is a client seam.
//! A consumed magnetic lifter (claim 11, S-0607 claim 12) ends its grip
//! as a lifter that leaves play, through the removal (`lift::removing`).

use crate::combat::effects::Effect;
use crate::econ::TypeRef;
use crate::rules::{Behaviour, Kind};
use crate::transport::Answer;
use crate::world::{HouseId, Layer, ObjectId, Target, World};

// ---------------------------------------------------------------------------
// Queries

/// Whether an object is a recycling building: a building whose type has the
/// grinding flag (Terms).
pub fn is_recycler(w: &World, b: ObjectId) -> bool {
    w.obj(b).is_some_and(|o| {
        o.kind == Kind::Building
            && w.rules()
                .buildings
                .get(o.type_index as usize)
                // spec: S-0603 Import mapping (Grinding absent: no)
                .is_some_and(|t| t.raw.grinding.unwrap_or(false))
    })
}

/// The docking answer of a recycling building to would-be entrant `u`
/// (claim 2). `Neither` for a building that does not recycle. A building
/// that also absorbs answers by S-0605 claim 1, which accepts through its
/// recycling feature (`absorb::answer`).
pub fn answer(w: &World, b: ObjectId, u: ObjectId) -> Answer {
    if !is_recycler(w, b) {
        return Answer::Neither;
    }
    if crate::structures::absorb::is_absorber(w, b) {
        // spec: S-0603 claim 2 (a building with both features refuses the wrong kind and a controller holding victims by S-0605 claim 1; the link-place row does not apply)
        return crate::structures::absorb::answer(w, b, u);
    }
    if recycler_refuses(w, b, u, true) {
        Answer::Refuses
    } else {
        Answer::Accepts
    }
}

/// The refusal rows of claim 2 for a building without an absorption flag;
/// `links` includes the link-place row (left out by the fixed behaviour of
/// `quirks.grinder_no_arrival_recheck`).
fn recycler_refuses(w: &World, b: ObjectId, u: ObjectId, links: bool) -> bool {
    let (Some(bo), Some(uo)) = (w.obj(b), w.obj(u)) else {
        return true;
    };
    // spec: S-0603 claim 2 (the building's owner does not count U's house as allied: one-way)
    if !w.allied(bo.owner, uo.owner) {
        return true;
    }
    // spec: S-0603 claim 2 (under construction, being sold or not finished building up)
    if matches!(
        crate::econ::placement::seen_behaviour(bo),
        Some(Behaviour::Construction | Behaviour::Selling)
    ) || bo.econ.sale.is_some()
    {
        return true;
    }
    // spec: S-0603 claim 2 and F2 (no link place free for U, max(d, 1) places; a place U already holds counts as free)
    if links {
        let held = crate::structures::absorb::contacts(w, b)
            .into_iter()
            .filter(|&x| x != u)
            .count() as u32;
        if held >= crate::econ::release::contact_places(w, b) {
            return true;
        }
    }
    // spec: S-0603 claim 2 and S-0504 claim 7 (exactly one naval, unless U's movement zone is amphibious)
    if crate::naval::naval_match_refuses(w, b, u) {
        return true;
    }
    // spec: S-0603 claim 2 (a type that hovers by balloon)
    if w.rules()
        .unit(uo.kind, uo.type_index)
        .is_some_and(|t| t.raw.balloon_hover.unwrap_or(false))
    {
        return true;
    }
    // spec: S-0603 claim 2 (switched off refuses; low power does not)
    bo.econ.building.switched_off
}

/// The fixed behaviour's answer at consumption (Edge cases,
/// `quirks.grinder_no_arrival_recheck` off): claim 2 without the
/// link-place row, infantry included.
fn refuses_at_arrival(w: &World, b: ObjectId, u: ObjectId) -> bool {
    if crate::structures::absorb::is_absorber(w, b) {
        return crate::structures::absorb::answer(w, b, u) != Answer::Accepts;
    }
    if !is_recycler(w, b) {
        // A destination without the grinding flag (claim 13) has no docking answer of this spec.
        return false;
    }
    recycler_refuses(w, b, u, false)
}

/// Whether `u` is an entrant bound for recycling building `b` (Terms): in
/// *eaten* with `b` as its destination or its target.
pub fn bound_for(w: &World, u: ObjectId, b: ObjectId) -> bool {
    w.obj(u).is_some_and(|o| {
        o.on_map
            && o.beh.current == Some(Behaviour::Eaten)
            && (o.dest == Some(Target::Object(b)) || o.target == Some(Target::Object(b)))
    })
}

// ---------------------------------------------------------------------------
// The eaten processing (claim 3, S-0215 claim 28)

/// The *eaten* processing of an infantry unit, vehicle or aircraft (claim
/// 3, S-0215 claim 28's table with its *eaten* differences, shared with
/// *capture* in `process::process_capture`). Returns the delay.
pub fn process_eaten(w: &mut World, id: ObjectId) -> i32 {
    let Some(o) = w.obj(id).cloned() else {
        return crate::process::standard_delay(w, id);
    };
    if let Some(Target::Object(k)) = o.dest
        && w.obj(k).is_none_or(|x| !x.on_map)
    {
        // spec: S-0603 claim 15 (a destination building that has left the game is cleared)
        crate::econ::assign_destination(w, id, None);
    }
    // spec: S-0603 claim 3 and S-0215 claim 28 (eaten is processed by the capture table, occupiers, assaulters and infiltrators not counting as building takers; a vehicle in eaten queues nothing when it falls back, claim 15)
    crate::process::process_capture(w, id, true)
}

// ---------------------------------------------------------------------------
// Consumption (claims 5 to 11)

/// The building an arriving entrant is consumed by (claim 5): its current
/// behaviour is *eaten* and the building in its cell is its destination or
/// its target. Any building: the grinding flag is not tested (claim 13).
fn arrival_building(w: &World, u: ObjectId) -> Option<ObjectId> {
    let o = w.obj(u)?;
    if !o.on_map || o.beh.current != Some(Behaviour::Eaten) {
        return None;
    }
    let cell = o.pos.cell();
    let here = |t: Option<Target>| match t {
        Some(Target::Object(b))
            if w.obj(b).is_some_and(|x| x.kind == Kind::Building && x.on_map)
                // spec: S-0603 claims 4 and 5 (a cell that holds the building: the first foundation cell it arrives in)
                && w.cells.list(cell, Layer::Ground).contains(&b) =>
        {
            Some(b)
        }
        _ => None,
    };
    // spec: S-0603 claim 5 (the building in that cell is its destination or its target)
    here(o.dest).or_else(|| here(o.target))
}

/// An infantry unit's arrival update at the end of a walking step (claim 5,
/// S-0208 claim 22). Returns whether it was consumed.
pub fn arrival(w: &mut World, u: ObjectId) -> bool {
    if w.obj(u).is_none_or(|o| o.kind != Kind::Infantry) {
        return false;
    }
    arrive(w, u)
}

/// A vehicle's cell-arrival update (claim 5, S-0417 claim 7). Returns
/// whether it was consumed.
pub fn vehicle_arrival(w: &mut World, u: ObjectId) -> bool {
    if w.obj(u).is_none_or(|o| o.kind != Kind::Vehicle) {
        return false;
    }
    arrive(w, u)
}

fn arrive(w: &mut World, u: ObjectId) -> bool {
    let Some(b) = arrival_building(w, u) else {
        return false;
    };
    if !w.rules().quirks.grinder_no_arrival_recheck && refuses_at_arrival(w, b, u) {
        // spec: S-0603 Edge cases (quirks.grinder_no_arrival_recheck off: refused at consumption, it is not consumed, its destination is cleared and it falls back to rest as in claim 15's destroyed case)
        crate::econ::assign_destination(w, u, None);
        if let Some(ob) = w.obj_mut(u)
            && ob.target == Some(Target::Object(b))
        {
            crate::spawn::clear_own_target(ob);
        }
        return false;
    }
    // spec: S-0603 claim 5 and Edge cases (quirks.grinder_no_arrival_recheck: nothing else is tested: not the alliance, owner, power, switch, sale, construction or health)
    consume(w, u, b);
    w.obj(u).is_none_or(|o| !o.on_map)
}

/// Pays house `to` the refund of object `x` computed as if its own current
/// owner sold it (claim 8, F1), straight into its credits.
fn pay_refund(w: &mut World, to: Option<HouseId>, x: ObjectId) {
    let (Some(to), Some(o)) = (to, w.obj(x)) else {
        return;
    };
    let Some(seller) = o.owner else { return };
    let t = TypeRef {
        kind: o.kind,
        index: o.type_index,
    };
    // spec: S-0603 claim 8 and F1 (S-0408 F1 for X's type with X's own current owner as the seller: its factors a and b and its use of the refund percentage; health, veterancy, price paid and a harvester's load play no part)
    let amount = crate::econ::sell::refund(w, seller, t);
    // spec: S-0603 claim 8 and F1 (each refund truncated and added on its own, directly to the building owner's credit balance, not to storage)
    crate::econ::sell::credit(w, to, amount);
}

/// The consumption of entrant `u` by building `b` (claims 6 to 11), through
/// the consuming-entry notifications of S-0524 claim 12.
fn consume(w: &mut World, u: ObjectId, b: ObjectId) {
    let grinding = is_recycler(w, b);
    crate::mp_triggers::consuming_entry_seam(w, b, u, move |w| {
        let Some(o) = w.obj(u).cloned() else { return };
        // spec: S-0603 claims 6 and 7 (after the entry raise: the death response and death sound of the entrant at its position, then the enter-grinder sound at a recycling building; S-0701 claim 20's now-dead choice)
        w.effects.push(Effect::Consumed {
            of: TypeRef {
                kind: o.kind,
                index: o.type_index,
            },
            owner: o.owner,
            pos: w.reference_point(u),
            grinding,
        });
        // spec: S-0603 claim 8 (the building's owner as it is at the moment of consumption)
        let to = w.obj(b).and_then(|x| x.owner);
        // spec: S-0603 claims 6 and 8 (the entrant's refund, credited before it leaves the game)
        pay_refund(w, to, u);
        if o.kind == Kind::Vehicle {
            for p in crate::transport::passengers(w, u) {
                // spec: S-0603 claims 8 and 10 (each direct passenger of a vehicle entrant, whatever its owner)
                pay_refund(w, to, p);
                for q in crate::transport::passengers(w, p) {
                    // spec: S-0603 claims 8 and 10 and Edge cases (each of their direct passengers; only these two levels are refunded)
                    pay_refund(w, to, q);
                }
            }
            if let Some(p) = crate::parasite::parasite_of(w, u) {
                // spec: S-0603 claim 8 (a parasite attached to a vehicle entrant, valued with its own owner's factors)
                pay_refund(w, to, p);
                // spec: S-0603 claim 10 and S-0614 claim 12 (after its refund: the consuming-building row, 50 ticks of suppression and the release attempt)
                crate::parasite::host_consumed(w, u);
            }
        }
        // An infantry entrant's parasite is not refunded; the removal releases it by S-0614 claim 15 (Open questions).
    });
}

// ---------------------------------------------------------------------------
// The grinder fate (claim 14) and interruptions (claim 15)

/// The grinder fate of S-0601 claim 8 for object `u` (claim 14): every
/// recycling building its current owner holds, whatever its state, the
/// nearest by ρ₃ between the two centres, ties to the one the owner placed
/// or gained most recently. Found: that building as destination and
/// *eaten* queued without an immediate start; returns whether one was.
pub fn grinder_fate(w: &mut World, u: ObjectId) -> bool {
    let Some(uo) = w.obj(u) else { return false };
    let Some(owner) = uo.owner else { return false };
    if !uo.on_map {
        return false;
    }
    let acquired = w
        .houses
        .get(usize::from(owner.0))
        .map(|h| h.econ.acquired.clone())
        .unwrap_or_default();
    let rank = |b: ObjectId| acquired.iter().position(|&x| x == b);
    let mut best: Option<(i64, Option<usize>, ObjectId)> = None;
    for o in w.objects.values() {
        // spec: S-0603 claim 14 (every recycling building its owner holds, whatever its power, switch, sale or docking answer)
        if o.owner != Some(owner) || !o.on_map || !is_recycler(w, o.id) {
            continue;
        }
        // spec: S-0603 claim 14 and S-0215 F4 (ρ₃ between the two centre positions, integer leptons truncated toward zero; quirks.table_root_distance)
        let d = crate::combat::acquire::rho3(w, u, o.id);
        let r = rank(o.id);
        // spec: S-0603 claim 14 (the smallest distance; of buildings tied at it, the one its owner placed or gained most recently, S-0404 claim 7's acquisition order)
        let better = best.is_none_or(|(bd, br, _)| d < bd || (d == bd && r > br));
        if better {
            best = Some((d, r, o.id));
        }
    }
    let Some((_, _, b)) = best else {
        return false;
    };
    if let Some(ob) = w.obj_mut(u) {
        // spec: S-0603 claim 14 (it queues eaten, without an immediate start)
        ob.beh.queue(Some(Behaviour::Eaten));
        ob.path.steps.clear();
        // spec: S-0212 claim 21 and its state table, row "destination changes" (r97: for the driving and walk locomotors a different destination, or none, ends a running retry delay, path::destination_changed and path::waits; the ship and the other locomotors keep it, low confidence; the hovering locomotor's is S-0503's)
    }
    // spec: S-0603 claim 14 (it takes that building as its destination)
    crate::econ::assign_destination(w, u, Some(Target::Object(b)));
    true
}

/// A building leaves the game (claim 15): every entrant bound for it has
/// its destination cleared; its next processing falls back to rest (claim
/// 3).
pub fn removing(w: &mut World, id: ObjectId) {
    if w.obj(id).is_none_or(|o| o.kind != Kind::Building) {
        return;
    }
    let bound: Vec<ObjectId> = w
        .objects
        .values()
        .filter(|o| {
            o.id != id
                && o.dest == Some(Target::Object(id))
                && (o.beh.current == Some(Behaviour::Eaten)
                    || o.beh.queued == Some(Behaviour::Eaten))
        })
        .map(|o| o.id)
        .collect();
    for e in bound {
        // spec: S-0603 claim 15 (when the building is destroyed or removed, the entrant's destination is cleared)
        crate::econ::assign_destination(w, e, None);
    }
}
