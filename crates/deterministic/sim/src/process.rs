//! Behaviour processing (S-0215): the per-tick start, readiness, falling
//! back to rest and the processings of the general-purpose behaviours.
//!
//! Seams for milestone M4 (combat): the target-acquisition specs are not
//! written, so no automatic search runs (claims 26, 27 and 33 see "may not
//! search"), and weapon ranges and rearm times are unknown (the close-range
//! shortening of F3 applies only through the infantry close-range flag; no
//! actor is ever rearming). Approaching a target is the approach step of
//! [`crate::approach`] (claim 37), whose position search (S-0307) finds
//! nothing yet. The behaviours that other topics own (claim 34) and *hunt*,
//! *capture*, *sabotage*, *enter*, and *unload* for vehicles, are processed as behaviours
//! without a processing rule until their topics are implemented; that is
//! recorded as a note on the world.
//!
//! Gates (claim 35): a gate's *open* runs its stages here, the door moves at
//! every turn of the gate before its per-tick start and processing (claim
//! 35 (d)), [`gate_open`] gives the passage state that S-0209's gate rows
//! read, and [`gate_met`] is the effect of the *closed friendly gate*
//! outcome, for the entry rules to call.
//!
//! The rules that S-0215 leaves to the harvest cycle, power and the other
//! M5a topics call [`crate::seams`].

use glasnost_map::CellXY;
use glasnost_num::geom::root_floor;

use crate::behaviour::Door;
use crate::infantry::{self, Seq};
use crate::occupancy;
use crate::rules::{Behaviour, Kind};
use crate::world::{AttackMoveMemory, Layer, ObjectId, Pos, Target, World};

/// The fixed delay of a behaviour without a processing rule (S-0215 claim 4).
// spec: S-0215 claim 4 (nothing happens; a delay of exactly 450 ticks, no draw)
const NO_RULE_DELAY: i32 = 450;

/// Whether the actor is moving on this tick (S-0215 claim 11): its
/// locomotor is moving.
fn moving(w: &World, id: ObjectId) -> bool {
    occupancy::in_progress(w, id)
}

/// Ready to change (S-0215 claim 11).
pub fn ready(w: &World, id: ObjectId) -> bool {
    let Some(o) = w.obj(id) else { return false };
    let cur = o.beh.current;
    // spec: S-0215 claim 11 (sticky or rescue is never ready, every kind)
    if matches!(cur, Some(Behaviour::Sticky | Behaviour::Rescue)) {
        return false;
    }
    match o.kind {
        Kind::Infantry => {
            let seq = o.infantry.as_ref().and_then(|s| s.seq);
            // spec: S-0215 claim 11 (infantry: moving while not in guard, hunt or attack without a target)
            let excused = matches!(cur, Some(Behaviour::Guard | Behaviour::Hunt))
                || (cur == Some(Behaviour::Attack) && o.target.is_none());
            if moving(w, id) && !excused {
                return false;
            }
            // spec: S-0215 claim 11 (infantry: a current sequence that is not interruptible)
            !seq.is_some_and(|q| !q.table().0)
        }
        Kind::Vehicle => {
            // spec: S-0215 claim 11 (vehicles: moving and not below the surface, except the three cases)
            let excused = o.beh.queued == Some(Behaviour::Enter)
                || cur == Some(Behaviour::Guard)
                || (cur == Some(Behaviour::Attack) && o.target.is_none())
                || (o.beh.queued == Some(Behaviour::Guard) && o.beh.allowance);
            let below = o.pos.z < w.surface_at(o.pos.x, o.pos.y);
            // spec: S-0215 claim 11 (letting out under its own unload processing) and S-0402 claim 11 (a harvester in the unloading state is not ready to change)
            if crate::econ::harvest::in_unloading_state(w, id) {
                return false;
            }
            !(moving(w, id) && !below && !excused)
        }
        Kind::Building => o.beh.ready_mark,
        Kind::Aircraft | Kind::Terrain => false,
    }
}

/// The standard delay D = D_r + U(0, 2) (S-0215 F2), the draw last.
pub(crate) fn standard_delay(w: &mut World, id: ObjectId) -> i32 {
    let dr = rate_delay(w, id);
    let half = w.rules().quirks.ranged_draw_half_span;
    // spec: S-0215 F2 and F0 (D = D_r + U(0, 2), wrapping 32-bit)
    dr.wrapping_add(w.rng.ranged(0, 2, half))
}

/// D_r of the behaviour current now (S-0215 claim 6, F1).
fn rate_delay(w: &World, id: ObjectId) -> i32 {
    let cur = w.obj(id).and_then(|o| o.beh.current);
    match cur {
        Some(b) => w.rules().behaviour(b).rate_delay,
        // A current behaviour of none has no rate; it takes the claim 4 delay instead.
        None => NO_RULE_DELAY,
    }
}

/// The close-range condition of S-0215 F3 for an actor with a target.
fn close_range(w: &World, id: ObjectId) -> bool {
    let Some(o) = w.obj(id) else { return false };
    let Some(t) = o.target else { return false };
    let r = w.rules();
    // spec: S-0215 claim 23 and F3 (an infantry type with the close-range flag, or a primary weapon range of at most 512)
    let flag = o.kind == Kind::Infantry
        && r.unit(o.kind, o.type_index)
            .is_some_and(|u| u.raw.close_range.unwrap_or(false));
    let short = crate::combat::weapon::slot_weapon(w, id, 0)
        .is_some_and(|x| r.combat.weapon(x).range <= 512); // spec: S-0215 F3 (a primary weapon range of at most 512 leptons)
    if !flag && !short {
        return false;
    }
    let tp = target_pos(w, t);
    let dx = i64::from(o.pos.x - tp.x);
    let dy = i64::from(o.pos.y - tp.y);
    // spec: S-0215 F3 and F4 (282 ≤ ρ₂ ≤ 768, ρ₂ the table-root 2-D distance)
    let rho = root_floor(
        (dx * dx + dy * dy) as u64,
        r.quirks.table_root_distance,
        r.mode,
    );
    (282..=768).contains(&rho) // spec: S-0215 F3 (282 ≤ ρ₂ ≤ 768)
}

/// A target's position: an object's position, or a cell's centre at its
/// height (S-0215 F4).
pub(crate) fn target_pos(w: &World, t: Target) -> Pos {
    match t {
        Target::Object(x) => w.obj(x).map_or(Pos::default(), |o| o.pos),
        Target::Cell(c) => {
            let raised = w.rules().quirks.negative_level_floor_raised;
            // spec: S-0215 F4 (a cell guard spot: the cell's centre at the cell's height, B(ℓ) of S-0106 F1)
            let z = w.cells.cell(c).map_or(0, |cell| {
                crate::surface::surface_height(i32::from(cell.level), 0, 0, 0, raised)
            });
            Pos::centre(c, z)
        }
    }
}

/// Falling back to rest (S-0215 claims 13 to 15). `placement` marks the
/// placement fall-back.
pub fn fall_back(w: &mut World, id: ObjectId, placement: bool) {
    let t = w.tick;
    let Some(o) = w.obj(id).cloned() else { return };
    if !o.kind.is_mobile() {
        return;
    }
    // spec: S-0215 claim 13 (the common effects at most once per actor per tick)
    if o.beh.last_common_effects != Some(t) {
        if let Some(ob) = w.obj_mut(id) {
            ob.beh.last_common_effects = Some(t);
        }
        // spec: S-0215 claim 13 (an infantry unit whose guard spot is on another cell: it becomes the destination)
        if o.kind == Kind::Infantry
            && let Some(gs) = o.guard_spot
        {
            let gc = match gs {
                Target::Cell(c) => Some(c),
                Target::Object(x) => w.obj(x).map(|g| g.pos.cell()),
            };
            if gc.is_some_and(|c| c != o.pos.cell())
                && let Some(ob) = w.obj_mut(id)
            {
                ob.dest = Some(gs);
                if ob.beh.current != Some(Behaviour::AreaGuard) {
                    // spec: S-0215 claim 13 (unless in area guard, it also queues move and forgets the guard spot)
                    ob.beh.queue(Some(Behaviour::Move));
                    ob.guard_spot = None;
                }
                // spec: S-0107 claim 15 (a destination assigned: the destination-driven get-up)
                crate::infantry::destination_assigned(w, id, o.dest);
            }
        }
    }
    match o.kind {
        Kind::Infantry => fall_back_infantry(w, id),
        Kind::Vehicle => fall_back_vehicle(w, id, placement),
        // The aircraft spec owns aircraft fall-backs (claim 13).
        _ => {}
    }
}

/// The guard-area veterancy ability at the actor's current rank (S-0311 claim 3).
fn guard_area_ability(w: &World, id: ObjectId) -> bool {
    crate::combat::veterancy::has_ability(w, id, crate::combat::data::Ability::GuardArea)
}

/// Claim 14 (infantry).
fn fall_back_infantry(w: &mut World, id: ObjectId) {
    let r = w.rules_arc();
    let Some(o) = w.obj(id).cloned() else { return };
    let Some(u) = r.unit(o.kind, o.type_index) else {
        return;
    };
    let cur = o.beh.current;
    // spec: S-0215 claim 14 (nothing is queued while the current behaviour is patrol or area guard)
    let blocked = matches!(cur, Some(Behaviour::Patrol | Behaviour::AreaGuard));
    let again = matches!(cur, Some(Behaviour::Sabotage | Behaviour::Capture));
    let queue = |w: &mut World, b: Behaviour| {
        if !blocked && let Some(ob) = w.obj_mut(id) {
            ob.beh.queue(Some(b));
        }
    };
    // spec: S-0215 claim 14 row 1 (attack-move memory, or wait: nothing changes)
    if o.attack_move.is_some() || cur == Some(Behaviour::Wait) {
        return;
    }
    // spec: S-0215 claim 14 row 2 (a target: attack, or the current sabotage or capture again)
    if o.target.is_some() {
        queue(
            w,
            if again {
                cur.unwrap_or(Behaviour::Attack)
            } else {
                Behaviour::Attack
            },
        );
        return;
    }
    let mut dest = o.dest;
    if dest.is_none() && !o.queued_dests.is_empty() {
        // spec: S-0215 claim 14 row 3 (the first queued destination becomes the destination)
        if let Some(ob) = w.obj_mut(id) {
            let first = ob.queued_dests.remove(0);
            ob.dest = Some(first);
            dest = Some(first);
            // spec: S-0107 claim 15 (a queued destination assigned: the destination-driven get-up)
            crate::infantry::destination_assigned(w, id, None);
        }
    }
    // spec: S-0215 claim 14 row 4 (a destination: move, or the current sabotage or capture again)
    if dest.is_some() {
        queue(
            w,
            if again {
                cur.unwrap_or(Behaviour::Move)
            } else {
                Behaviour::Move
            },
        );
        return;
    }
    // spec: S-0215 claim 14 rows 5 and 6 (guard or area guard; zombie or paralysed: nothing changes)
    if matches!(cur, Some(Behaviour::Guard | Behaviour::AreaGuard)) {
        return;
    }
    if cur.is_some_and(|b| r.behaviour(b).zombie || r.behaviour(b).paralysed) {
        return;
    }
    let human = o.owner.is_none_or(|h| w.houses[usize::from(h.0)].human);
    if !human {
        // The neutral and special houses keep rows 7 to 10 (S-0215 Terms, houses in our engine); no unit is a team member.
        let skill = o.owner.map_or(0, |h| w.houses[usize::from(h.0)].skill);
        let armed = u.primary.is_some();
        let b = if skill < r.general.iq_guard_area {
            // spec: S-0215 claim 14 row 7 (not human-controlled, skill below the area-guard threshold: guard)
            Behaviour::Guard
        } else if crate::seams::is_harvesting_slave(w, id) {
            // spec: S-0215 claim 14 row 8 (a harvesting slave: guard)
            Behaviour::Guard
        } else if armed || u.engineer || u.vehicle_thief {
            // spec: S-0215 claim 14 row 9 (armed, an engineer or a vehicle thief: area guard)
            Behaviour::AreaGuard
        } else {
            // spec: S-0215 claim 14 row 10 (otherwise guard)
            Behaviour::Guard
        };
        queue(w, b);
        return;
    }
    // spec: S-0215 claim 14 rows 11 and 12 (a player house: area guard with the ability or the type default, else guard)
    if guard_area_ability(w, id) || u.default_to_guard_area {
        queue(w, Behaviour::AreaGuard);
    } else {
        queue(w, Behaviour::Guard);
    }
}

/// Claim 15 (vehicles).
fn fall_back_vehicle(w: &mut World, id: ObjectId, placement: bool) {
    let r = w.rules_arc();
    let Some(o) = w.obj(id).cloned() else { return };
    let Some(u) = r.unit(o.kind, o.type_index) else {
        return;
    };
    let cur = o.beh.current;
    // spec: S-0215 claim 15 (nothing is queued while patrol, area guard, unload or eaten is current)
    let blocked = matches!(
        cur,
        Some(Behaviour::Patrol | Behaviour::AreaGuard | Behaviour::Unload | Behaviour::Eaten)
    );
    let queue = |w: &mut World, b: Behaviour| {
        if !blocked && let Some(ob) = w.obj_mut(id) {
            ob.beh.queue(Some(b));
        }
    };
    let clear = |w: &mut World| {
        if let Some(ob) = w.obj_mut(id) {
            ob.target = None;
            ob.combat.burst = 0; // spec: S-0301 claim 4 (a cleared target resets the burst counter)
            ob.dest = None;
        }
    };
    // spec: S-0215 claim 15 row 1 (attack-move memory, or wait: nothing changes)
    if o.attack_move.is_some() || cur == Some(Behaviour::Wait) {
        return;
    }
    let mut dest = o.dest;
    if dest.is_none() && !o.queued_dests.is_empty() {
        // spec: S-0215 claim 15 row 2 (the first queued destination becomes the destination)
        if let Some(ob) = w.obj_mut(id) {
            let first = ob.queued_dests.remove(0);
            ob.dest = Some(first);
            dest = Some(first);
        }
    }
    // spec: S-0215 claim 15 row 3 (a destination: queue move)
    if dest.is_some() {
        queue(w, Behaviour::Move);
        return;
    }
    if u.harvester {
        // spec: S-0215 claim 15 row 4 (harvesters; the resource land test is skipped in the placement fall-back)
        if o.contact.is_some()
            || cur == Some(Behaviour::Harvest)
            || o.beh.queued == Some(Behaviour::Harvest)
        {
            return;
        }
        clear(w);
        let on_ore = w
            .cells
            .cell(o.pos.cell())
            .is_some_and(|c| c.land == glasnost_map::terrain::land::TIBERIUM);
        // spec: S-0215 claim 15 row 4 and Terms (the exception off the resource land type is for human-controlled harvesters; the neutral and special houses' harvesters queue harvest wherever they stand)
        let human = o.owner.is_none_or(|h| w.houses[usize::from(h.0)].human);
        queue(
            w,
            if placement || on_ore || !human {
                Behaviour::Harvest
            } else {
                Behaviour::Guard
            },
        );
        return;
    }
    let armed = u.primary.is_some();
    if !armed {
        // spec: S-0215 claim 15 row 6 (unarmed and deploying, or with area guard current: nothing changes)
        if cur == Some(Behaviour::AreaGuard) {
            return;
        }
        // spec: S-0215 claim 15 row 7 (unarmed: clear target and destination, queue guard)
        clear(w);
        queue(w, Behaviour::Guard);
        return;
    }
    // spec: S-0215 claim 15 rows 8 and 9 (armed with guard or area guard; zombie or paralysed: nothing changes)
    if matches!(cur, Some(Behaviour::Guard | Behaviour::AreaGuard)) {
        return;
    }
    if cur.is_some_and(|b| r.behaviour(b).zombie || r.behaviour(b).paralysed) {
        return;
    }
    let skill = o.owner.map_or(0, |h| w.houses[usize::from(h.0)].skill);
    let area = guard_area_ability(w, id) || u.default_to_guard_area;
    // spec: S-0215 claim 15 row 10 (skill below the area-guard threshold, without the ability or the type default)
    if skill < r.general.iq_guard_area && !area {
        queue(w, Behaviour::Guard);
        return;
    }
    // spec: S-0215 claim 15 row 11 (armed, a harvesting slave or a slave master: guard; no vehicle is a team member)
    if crate::seams::is_harvesting_slave(w, id) || crate::seams::is_slave_master(w, id) {
        queue(w, Behaviour::Guard);
        return;
    }
    // spec: S-0215 claim 15 row 12 (otherwise area guard)
    queue(w, Behaviour::AreaGuard);
}

/// One actor's behaviour part of its turn (S-0215 claims 2, 3, 11 and 33).
pub fn turn(w: &mut World, id: ObjectId) {
    let stale = w.rules().quirks.stale_suspended_revert; // spec: S-0215 Edge cases (quirks.stale_suspended_revert)
    let t = w.tick;
    let Some(o) = w.obj(id) else { return };
    let kind = o.kind;
    if kind == Kind::Terrain || kind == Kind::Aircraft {
        return;
    }
    if kind == Kind::Building {
        door_update(w, id);
    }
    resume_attack_move(w, id);
    // spec: S-0215 claim 11 (the per-tick start of infantry, vehicles and buildings)
    let ready_now = ready(w, id);
    let Some(o) = w.obj(id) else { return };
    if ready_now {
        if kind == Kind::Infantry && o.beh.current.is_none() && o.beh.queued.is_none() {
            // spec: S-0215 claim 11 (an infantry unit with neither falls back to rest at its per-tick start)
            fall_back(w, id, false);
        }
        if let Some(ob) = w.obj_mut(id)
            && ob.beh.start_next(t, stale)
            && kind == Kind::Building
        {
            // spec: S-0215 claim 11 (each successful per-tick start of a building clears its ready mark)
            ob.beh.ready_mark = false;
        }
    }
    // spec: S-0215 claim 33 (the drop rule: an opportunity target is dropped in these behaviours, judged after the per-tick start)
    if let Some(ob) = w.obj_mut(id)
        && ob.target.is_some()
        && ob.combat.opportunity
        && matches!(
            ob.beh.current,
            Some(
                Behaviour::Ambush
                    | Behaviour::Construction
                    | Behaviour::Enter
                    | Behaviour::Harmless
                    | Behaviour::Missile
                    | Behaviour::Open
                    | Behaviour::Repair
                    | Behaviour::Selling
                    | Behaviour::Sleep
                    | Behaviour::Stop
                    | Behaviour::Unload
                    | Behaviour::Wait
            )
        )
    {
        ob.target = None;
        ob.combat.opportunity = false;
        ob.combat.burst = 0; // spec: S-0301 claim 4 (a cleared target resets the burst counter)
    }
    let Some(o) = w.obj(id) else { return };
    // spec: S-0215 claim 2 (alive with health above 0 and the timer expired)
    if o.health <= 0 || !o.beh.due(t) {
        if kind == Kind::Building {
            second_start(w, id);
        }
        // spec: S-0215 claim 37 (a) (the per-tick approach of an actor that cannot reach its target; its place in the tick is interim, request Q-0161)
        crate::approach::per_tick(w, id);
        // spec: S-0215 claim 33 (b) and (d) (the per-tick search after the processing)
        crate::combat::acquire::per_tick(w, id);
        return;
    }
    let d = process(w, id);
    if let Some(ob) = w.obj_mut(id) {
        ob.beh.processed(t, d);
    }
    if kind == Kind::Building {
        second_start(w, id);
    }
    // spec: S-0215 claim 37 (a) (the per-tick approach of an actor that cannot reach its target; its place in the tick is interim, request Q-0161)
    crate::approach::per_tick(w, id);
    // spec: S-0215 claim 33 (b) and (d) (the per-tick search after the processing, judged on the behaviour it leaves)
    crate::combat::acquire::per_tick(w, id);
}

/// The attack-move resumption at the start of an object's tick (S-0309
/// claim 23 (b)): when engaged (which it then no longer is), or not engaged
/// and in *guard*.
fn resume_attack_move(w: &mut World, id: ObjectId) {
    let stale = w.rules().quirks.stale_suspended_revert; // spec: S-0215 Edge cases (quirks.stale_suspended_revert)
    let t = w.tick;
    let Some(ob) = w.obj_mut(id) else { return };
    let Some(mem) = ob.attack_move else { return };
    // spec: S-0309 claim 23 (b) (resumed when engaged, or not engaged and in guard)
    if !(ob.combat.engaged || ob.beh.current == Some(Behaviour::Guard)) {
        return;
    }
    // spec: S-0309 claim 23 (c) (resuming ends the engaged mark)
    ob.combat.engaged = false;
    match mem {
        AttackMoveMemory::Dest(d) => {
            // spec: S-0309 claim 23 (b) (a destination: move queued with an immediate start, the memory's destination the object's)
            let before = ob.dest;
            ob.beh.queue_and_start_at_once(Behaviour::Move, t, stale);
            ob.dest = Some(d);
            // spec: S-0107 claim 15 (a destination assigned: the destination-driven get-up)
            crate::infantry::destination_assigned(w, id, before);
        }
        AttackMoveMemory::Target(x) => {
            // spec: S-0309 claim 23 (b) (only a target: attack queued with an immediate start, the memory's target the object's)
            ob.beh.queue_and_start_at_once(Behaviour::Attack, t, stale);
            ob.target = Some(x);
        }
    }
}

/// A building's second per-tick start (S-0215 claim 11).
fn second_start(w: &mut World, id: ObjectId) {
    let stale = w.rules().quirks.stale_suspended_revert; // spec: S-0215 Edge cases (quirks.stale_suspended_revert)
    let t = w.tick;
    if let Some(ob) = w.obj_mut(id)
        && ob.beh.ready_mark
        && ob.beh.start_next(t, stale)
    {
        // spec: S-0215 claim 11 (the second start, after the processing; it clears the mark when it succeeds)
        ob.beh.ready_mark = false;
    }
}

/// One processing; returns the delay chosen.
fn process(w: &mut World, id: ObjectId) -> i32 {
    let Some(o) = w.obj(id).cloned() else {
        return NO_RULE_DELAY;
    };
    let cur = o.beh.current;
    // spec: S-0215 claim 31 (deploying infantry: its rules take precedence in guard and area guard)
    if o.kind == Kind::Infantry
        && matches!(
            cur,
            Some(Behaviour::Guard | Behaviour::Sticky | Behaviour::AreaGuard)
        )
        && let Some(d) = deployer_rule(w, id)
    {
        return d;
    }
    match (cur, o.kind) {
        (Some(Behaviour::Move), _) => process_move(w, id),
        (Some(Behaviour::Attack), Kind::Infantry | Kind::Vehicle) => process_attack(w, id),
        // spec: S-0215 claim 5 (sticky is processed exactly as guard)
        (Some(Behaviour::Guard | Behaviour::Sticky), Kind::Infantry | Kind::Vehicle) => {
            process_guard(w, id)
        }
        (Some(Behaviour::AreaGuard), Kind::Infantry | Kind::Vehicle) => process_area_guard(w, id),
        (Some(Behaviour::Hunt), Kind::Infantry | Kind::Vehicle) => process_hunt(w, id),
        (Some(Behaviour::Unload), Kind::Infantry) => process_unload_infantry(w, id),
        // spec: S-0402 State and ordering (a harvester's harvest, enter and unload)
        (Some(Behaviour::Harvest), Kind::Vehicle) if crate::econ::harvest::is_harvester(w, id) => {
            crate::econ::harvest::process_harvest(w, id)
        }
        (Some(Behaviour::Enter), Kind::Vehicle) if crate::econ::harvest::is_harvester(w, id) => {
            crate::econ::harvest::process_enter(w, id)
        }
        (Some(Behaviour::Unload), Kind::Vehicle)
            if crate::econ::harvest::unload_is_harvesting(w, id) =>
        {
            crate::econ::harvest::process_unload(w, id)
        }
        // spec: S-0415 claim 1 (unload of a vehicle: ranks 1 to 3 are other topics' rules, rank 4 deploys; the standard delay of S-0215 claim 6)
        (Some(Behaviour::Unload), Kind::Vehicle)
            if crate::econ::deploy::process_unload(w, id).is_some() =>
        {
            standard_delay(w, id)
        }
        // spec: S-0408 claim 16 (selling: a processing delay of 1, no draw)
        (Some(Behaviour::Selling), Kind::Building) => crate::econ::sell::process_selling(w, id),
        // spec: S-0108 claim 5 (construction ends when the build-up does; request Q-0130)
        (Some(Behaviour::Construction), Kind::Building) => {
            crate::econ::deploy::process_construction(w, id)
        }
        (
            Some(Behaviour::Guard | Behaviour::Sticky | Behaviour::AreaGuard | Behaviour::Attack),
            Kind::Building,
        ) => crate::combat::behave::building_process(w, id),
        // spec: S-0215 claim 35 (c) (open for buildings)
        (Some(Behaviour::Open), Kind::Building) => process_open(w, id),
        // spec: S-0215 claim 4 and table T1 (open for infantry and vehicles: no processing rule, 450 ticks)
        (Some(Behaviour::Open), _) => NO_RULE_DELAY,
        // spec: S-0215 claim 5 (queued move and attack move are processed as sleep, quirks.queued_move_sleeps)
        (Some(Behaviour::QueuedMove | Behaviour::AttackMove), _)
            if w.rules().quirks.queued_move_sleeps =>
        {
            NO_RULE_DELAY
        }
        // spec: S-0215 claim 4 (sleep, harmless, ambush, return, stop, wait and none: 450 ticks, no draw)
        (
            None
            | Some(
                Behaviour::Sleep
                | Behaviour::Harmless
                | Behaviour::Ambush
                | Behaviour::Return
                | Behaviour::Stop
                | Behaviour::Wait,
            ),
            _,
        ) => NO_RULE_DELAY,
        (Some(b), _) => {
            w.note(format!(
                "S-0215: the processing of {} is not implemented yet; it waits as a behaviour without a processing rule",
                b.key()
            ));
            NO_RULE_DELAY
        }
    }
}

/// The undeploy delay of an infantry unit's type (S-0214 Import mapping:
/// absent −1, none).
fn undeploy_delay(w: &World, id: ObjectId) -> i32 {
    let o = w.obj(id);
    // spec: S-0214 Import mapping (UndeployDelay absent: −1, none)
    o.and_then(|o| w.rules().unit(o.kind, o.type_index))
        .map_or(-1, |u| u.raw.undeploy_delay.unwrap_or(-1))
}

/// The frame count of an infantry sequence (S-0107 claim 1).
fn frame_count(w: &World, id: ObjectId, q: Seq) -> i32 {
    w.obj(id)
        .and_then(|o| w.rules().unit(o.kind, o.type_index))
        .and_then(|u| u.sequences.get(q.key()))
        .map_or(0, |e| e.count)
}

/// The deployer rules of S-0215 claim 31, applied when one applies
/// (returning its delay); `None` when none applies.
fn deployer_rule(w: &mut World, id: ObjectId) -> Option<i32> {
    let o = w.obj(id)?;
    let u = w.rules().unit(o.kind, o.type_index)?;
    if !u.deployer {
        return None;
    }
    let deployed = o
        .infantry
        .as_ref()
        .and_then(|s| s.seq)
        .is_some_and(Seq::is_deploy);
    if !deployed {
        return auto_deploy(w, id);
    }
    if undeploy_delay(w, id) >= 0 {
        // spec: S-0215 claim 31 row 1 (deployed, undeploy delay 0 or more: starts undeploy; its frame count)
        infantry::request(w, id, Seq::Undeploy, false);
        return Some(frame_count(w, id, Seq::Undeploy));
    }
    let fires = w
        .obj(id)
        .and_then(|o| w.rules().unit(o.kind, o.type_index))
        .is_some_and(|u| {
            u.raw.deploy_fire.unwrap_or(false) && !u.raw.immune_to_radiation.unwrap_or(false)
        });
    if fires {
        // spec: S-0215 claim 31 row 2 (deploys to fire, not immune to radiation: the deployed-firing rule, M4; the standard delay)
        return Some(standard_delay(w, id));
    }
    // The radiation rows need weapon radiation levels (combat); a type that does not deploy to fire follows claims 24 and 26.
    None
}

/// The last row of S-0215 claim 31 for an undeployed unit: the auto-deploy
/// of an owner that is not human-controlled. In our engine no player house
/// reaches it; the neutral and special houses keep it, with A the list's
/// entry for their difficulty index 1, and without that entry the row does
/// not apply (claim 31, a documented departure).
fn auto_deploy(w: &mut World, id: ObjectId) -> Option<i32> {
    let t = w.tick;
    let o = w.obj(id)?.clone();
    let r = w.rules_arc();
    let u = r.unit(o.kind, o.type_index)?;
    // spec: S-0215 claim 31 (the last row: an owner that is not human-controlled; Terms, houses in our engine: the neutral and special houses only)
    let human = o.owner.is_none_or(|h| w.houses[usize::from(h.0)].human);
    if human {
        return None;
    }
    // spec: S-0215 claim 31 (A: the entry for difficulty index 1 of the neutral and special houses; no such entry: the row does not apply)
    let a = r.general.auto_deploy_delay?;
    let fires = u.raw.deploy_fire.unwrap_or(false);
    let immune = u.raw.immune_to_radiation.unwrap_or(false);
    let on_spot = match o.guard_spot {
        None => true,
        Some(Target::Cell(c)) => c == o.pos.cell(),
        Some(Target::Object(x)) => w.obj(x).is_some_and(|g| g.pos.cell() == o.pos.cell()),
    };
    // spec: S-0215 claim 31 (deploys to fire, not immune, undeploy delay negative, no destination, no guard spot or on its cell, and t > t_s + A with the sum wrapping)
    let due = t > o.beh.start_tick.wrapping_add(a);
    if !(fires && !immune && undeploy_delay(w, id) < 0 && o.dest.is_none() && on_spot && due) {
        return None;
    }
    if moving(w, id) {
        // spec: S-0215 claim 31 (moving: it is told to stop, the locomotor stop of S-0213 claim 8; the standard delay)
        if let Some(ob) = w.obj_mut(id) {
            ob.path.steps.clear();
        }
        return Some(standard_delay(w, id));
    }
    // spec: S-0215 claim 31 (otherwise it starts its deploy sequence; the delay is that sequence's frame count)
    infantry::request(w, id, Seq::Deploy, false);
    Some(frame_count(w, id, Seq::Deploy))
}

/// Unload for infantry (S-0215 claim 30).
fn process_unload_infantry(w: &mut World, id: ObjectId) -> i32 {
    let stale = w.rules().quirks.stale_suspended_revert; // spec: S-0215 Edge cases (quirks.stale_suspended_revert)
    let t = w.tick;
    let Some(o) = w.obj(id).cloned() else {
        return NO_RULE_DELAY;
    };
    let deployer = w
        .rules()
        .unit(o.kind, o.type_index)
        .is_some_and(|u| u.deployer);
    if !deployer {
        // spec: S-0215 claim 30 (any other infantry unit does nothing in unload: 450 ticks, no draw)
        return NO_RULE_DELAY;
    }
    let deployed = o
        .infantry
        .as_ref()
        .and_then(|s| s.seq)
        .is_some_and(Seq::is_deploy);
    let delay = undeploy_delay(w, id);
    let d = if deployed {
        if delay < 0 {
            // spec: S-0215 claim 30 (a deployed unit undeploys only if its undeploy delay is negative)
            infantry::request(w, id, Seq::Undeploy, false);
        }
        // spec: S-0215 claim 30 (deployed: 450 ticks)
        NO_RULE_DELAY
    } else {
        // spec: S-0215 claim 30 (an undeployed unit starts its deploy sequence; area-fire weapons wait for M4)
        infantry::request(w, id, Seq::Deploy, false);
        // spec: S-0215 claim 30 (undeployed: the undeploy delay when 0 or more, else 450 ticks)
        if delay >= 0 { delay } else { NO_RULE_DELAY }
    };
    if let Some(ob) = w.obj_mut(id) {
        // spec: S-0215 claim 30 (in every case it forces guard and clears its destination)
        ob.beh.force(Behaviour::Guard, t, stale);
        ob.dest = None;
    }
    d
}

/// Move (S-0215 claim 22).
fn process_move(w: &mut World, id: ObjectId) -> i32 {
    let Some(o) = w.obj(id).cloned() else {
        return NO_RULE_DELAY;
    };
    let deployed = o.kind == Kind::Infantry
        && o.infantry
            .as_ref()
            .and_then(|s| s.seq)
            .is_some_and(Seq::is_deploy);
    if deployed {
        // spec: S-0215 claim 22 (deployed infantry: clears its destination, 1 tick)
        if let Some(ob) = w.obj_mut(id) {
            ob.dest = None;
        }
        return 1;
    }
    // spec: S-0215 claim 22 (no destination, not moving, nothing queued: falls back to rest, 1 tick)
    if o.dest.is_none() && !moving(w, id) && o.beh.queued.is_none() {
        fall_back(w, id, false);
        return 1;
    }
    standard_delay(w, id)
}

/// Attack (S-0215 claim 23).
fn process_attack(w: &mut World, id: ObjectId) -> i32 {
    let Some(o) = w.obj(id).cloned() else {
        return NO_RULE_DELAY;
    };
    if o.kind == Kind::Infantry {
        let deployed = o
            .infantry
            .as_ref()
            .and_then(|s| s.seq)
            .is_some_and(Seq::is_deploy);
        if deployed && o.target.is_some() {
            // spec: S-0215 claim 23 (deployed infantry: its deployed-firing rule, the standard delay not halved)
            return standard_delay(w, id);
        }
    }
    if let Some(t) = o.target {
        let (slot, _) = crate::combat::weapon::would_use(w, id, Some(t));
        // spec: S-0304 claim 2 (the target is dropped, and the attack stopped, at the first check that answers illegal)
        if crate::combat::check::check(w, id, Some(t), slot, true)
            == crate::combat::check::Answer::Illegal
            && let Some(ob) = w.obj_mut(id)
        {
            ob.target = None;
            ob.combat.burst = 0; // spec: S-0301 claim 4 (a cleared target resets the burst counter)
        }
    }
    if w.obj(id).is_some_and(|o| o.target.is_none()) {
        // spec: S-0215 claim 23 (fall-back: no target)
        fall_back(w, id, false);
    }
    if w.obj(id).is_some_and(|o| o.target.is_some()) {
        // spec: S-0215 claim 23 (approach: an actor with a target takes the approach step of claim 37)
        crate::approach::step(w, id);
    }
    let d = standard_delay(w, id);
    // spec: S-0215 claim 23 and F3 (halved under the close-range condition, the draw taken either way)
    if close_range(w, id) { d / 2 } else { d }
}

/// Guard (S-0215 claim 24).
fn process_guard(w: &mut World, id: ObjectId) -> i32 {
    let Some(o) = w.obj(id).cloned() else {
        return NO_RULE_DELAY;
    };
    if o.kind == Kind::Vehicle {
        // spec: S-0215 claim 24 vehicle table row 1 (a slave master for which S-0402's wake-up rule holds: wakes its slaves, the standard delay; excludes the rest)
        if crate::seams::slave_wake_rule_holds(w, id) {
            crate::seams::wake_slaves(w, id);
            return standard_delay(w, id);
        }
        // spec: S-0215 claim 24 vehicle table row 2 (a guard rule of S-0402 that queues harvest again: 1 tick, no draw; excludes the rest)
        if crate::seams::harvester_guard_rule(w, id) {
            return 1;
        }
        // Row 3 (the weeder's rule) is left by S-0402 to a later revision; row 4 is the original's computer players' only.
    }
    if o.target.is_none() && o.kind == Kind::Infantry {
        // spec: S-0215 claim 24 (no target: attempts its idle action, S-0107 claim 14)
        let in_progress = moving(w, id);
        infantry::idle_attempt(w, id, in_progress);
    }
    let t = w.tick;
    let Some(o) = w.obj(id).cloned() else {
        return NO_RULE_DELAY;
    };
    // spec: S-0215 claim 24 (the actor's weapon is still rearming: the rearm time remaining, no draw)
    if t < o.combat.rearm_until {
        return o.combat.rearm_until.wrapping_sub(t);
    }
    // spec: S-0215 claim 24 (distributed fire with candidates: 0, no draw)
    let distributed = w
        .rules()
        .unit(o.kind, o.type_index)
        .is_some_and(|u| u.raw.distributed_fire.unwrap_or(false));
    if distributed && !o.combat.fired_at.is_empty() {
        return 0;
    }
    standard_delay(w, id)
}

/// Hunt (S-0215 claim 27).
fn process_hunt(w: &mut World, id: ObjectId) -> i32 {
    let stale = w.rules().quirks.stale_suspended_revert; // spec: S-0215 Edge cases (quirks.stale_suspended_revert)
    let t = w.tick;
    let r = w.rules_arc();
    let Some(o) = w.obj(id).cloned() else {
        return NO_RULE_DELAY;
    };
    let Some(u) = r.unit(o.kind, o.type_index).cloned() else {
        return NO_RULE_DELAY;
    };
    let stupid = u.raw.stupid_hunt.unwrap_or(false);
    if o.kind == Kind::Vehicle && u.raw.deploys_into.is_some() {
        // spec: S-0215 claim 27 (a deploying vehicle seeks a clear spot to deploy; the deploy specs own it)
        w.note("S-0215 claim 27: a hunting vehicle that deploys does not seek a spot yet (deploy specs)");
        return standard_delay(w, id);
    }
    if !stupid {
        // spec: S-0215 claim 27 (the normal search from its own position, unless stupid hunt)
        crate::combat::acquire::search(
            w,
            id,
            crate::combat::acquire::SearchKind::MapWide,
            crate::combat::acquire::SearchContext::Hunt,
        );
    }
    let target = if stupid {
        None
    } else {
        w.obj(id).and_then(|x| x.target)
    };
    match target {
        Some(tg) => {
            let charges = u.raw.c4.unwrap_or(false)
                || crate::combat::veterancy::has_ability(w, id, crate::combat::data::Ability::C4);
            let building = matches!(tg, Target::Object(x) if w.obj(x).is_some_and(|b| b.kind == Kind::Building));
            let ready_now = ready(w, id);
            if o.kind == Kind::Infantry && ((u.engineer && !charges) || u.vehicle_thief) {
                // spec: S-0215 claim 27 (an engineer or vehicle thief: the target as destination, capture queued with an immediate start if ready)
                if let Some(ob) = w.obj_mut(id) {
                    let before = ob.dest;
                    ob.dest = Some(tg);
                    ob.beh
                        .queue_and_start(Some(Behaviour::Capture), ready_now, t, stale);
                    // spec: S-0107 claim 15 (a destination assigned: the destination-driven get-up)
                    crate::infantry::destination_assigned(w, id, before);
                }
            } else if o.kind == Kind::Infantry && charges && building {
                // spec: S-0215 claim 27 (explosive charges against a building: sabotage)
                if let Some(ob) = w.obj_mut(id) {
                    let before = ob.dest;
                    ob.dest = Some(tg);
                    ob.beh
                        .queue_and_start(Some(Behaviour::Sabotage), ready_now, t, stale);
                    // spec: S-0107 claim 15 (a destination assigned: the destination-driven get-up)
                    crate::infantry::destination_assigned(w, id, before);
                }
            } else {
                if o.kind == Kind::Infantry
                    && let Some(ob) = w.obj_mut(id)
                {
                    // spec: S-0215 claim 27 (any other infantry unit clears its destination)
                    ob.dest = None;
                }
                // spec: S-0215 claim 27 (and takes the approach step of claim 37; other kinds take that step)
                crate::approach::step(w, id);
            }
        }
        None => {
            if o.kind == Kind::Infantry {
                // spec: S-0215 claim 27 (without a target: it attempts its idle action)
                let ip = moving(w, id);
                infantry::idle_attempt(w, id, ip);
            }
        }
    }
    // spec: S-0215 claim 27 (the standard delay with the rate of the behaviour current once the outcome took effect)
    standard_delay(w, id)
}

/// Area guard (S-0215 claim 26).
fn process_area_guard(w: &mut World, id: ObjectId) -> i32 {
    let stale = w.rules().quirks.stale_suspended_revert; // spec: S-0215 Edge cases (quirks.stale_suspended_revert)
    let t = w.tick;
    let r = w.rules_arc();
    let Some(o) = w.obj(id).cloned() else {
        return NO_RULE_DELAY;
    };
    let Some(u) = r.unit(o.kind, o.type_index).cloned() else {
        return NO_RULE_DELAY;
    };
    let half = r.quirks.ranged_draw_half_span;
    // spec: S-0215 claim 26 row 1 (a slave master for which S-0402's wake-up rule holds: wakes its slaves, the standard delay)
    if o.kind == Kind::Vehicle && crate::seams::slave_wake_rule_holds(w, id) {
        crate::seams::wake_slaves(w, id);
        return standard_delay(w, id);
    }
    if o.kind == Kind::Vehicle && u.harvester {
        // spec: S-0215 claim 26 row 4 (a harvester: harvest at once; 1 + U(1, 10))
        if let Some(ob) = w.obj_mut(id) {
            ob.beh.queue_and_start_at_once(Behaviour::Harvest, t, stale);
        }
        return 1i32.wrapping_add(w.rng.ranged(1, 10, half)); // spec: S-0215 claim 26 row 4 (1 + U(1, 10))
    }
    // spec: S-0215 claim 26 (no row ended the processing: a slave master wakes its slaves anyway when its slave group is at rest, S-0402 claim 14)
    if o.kind == Kind::Vehicle && crate::seams::slave_group_at_rest(w, id) {
        crate::seams::wake_slaves(w, id);
    }
    if let Some(gs) = o.guard_spot {
        let gp = match gs {
            Target::Object(x) => w.reference_point(x),
            Target::Cell(_) => target_pos(w, gs),
        };
        let dx = i64::from(o.pos.x - gp.x);
        let dy = i64::from(o.pos.y - gp.y);
        let dz = i64::from(o.pos.z - gp.z);
        // spec: S-0215 F4 (ρ₃ to the guard spot, minus 64 × (W + H) for a building, not below 0)
        let mut dist = root_floor(
            (dx * dx + dy * dy + dz * dz) as u64,
            r.quirks.table_root_distance,
            r.mode,
        ) as i64;
        if let Target::Object(x) = gs
            && let Some(g) = w.obj(x)
            && g.kind == Kind::Building
        {
            dist = (dist - 64 * i64::from(g.foundation.0 + g.foundation.1)).max(0); // spec: S-0215 F4 (β = 64 × (W + H))
        }
        let mobile_spot =
            matches!(gs, Target::Object(x) if w.obj(x).is_some_and(|g| g.kind.is_mobile()));
        let leash = if mobile_spot {
            // spec: S-0215 F5 (guarding a mobile actor: the guard stray distance)
            // spec: S-0215 claim 36 (b) and Questions for the project lead (decided: given by no layer, the area-guard leash instead)
            r.general
                .guard_stray
                .unwrap_or_else(|| leash(u.guard_range))
        } else {
            leash(u.guard_range)
        };
        let firing = t < o.combat.rearm_until;
        // spec: S-0215 claim 26 (the leash: not firing, no destination, distance above L)
        let holds = !firing && o.dest.is_none() && dist > i64::from(leash);
        // spec: S-0215 claim 26 (may search: passive acquisition allowed and the targeting timer run out)
        let may = crate::combat::acquire::may_search_passively(w, id) && t >= o.targeting_timer;
        let idle = |w: &mut World| {
            if o.kind == Kind::Infantry {
                let ip = moving(w, id);
                infantry::idle_attempt(w, id, ip);
            }
        };
        if holds {
            // spec: S-0215 claim 26 (drops any target, takes the guard spot as destination)
            if let Some(ob) = w.obj_mut(id) {
                ob.target = None;
                ob.combat.burst = 0;
                ob.dest = Some(gs);
                // spec: S-0107 claim 15 (a destination assigned: the destination-driven get-up; the leash holds only without one)
                crate::infantry::destination_assigned(w, id, None);
            }
        }
        if (holds || o.target.is_none()) && may {
            // spec: S-0215 claim 26 (the area search; a search that finds a target ends the processing with 1 tick)
            let found = crate::combat::acquire::search(
                w,
                id,
                crate::combat::acquire::SearchKind::Area,
                crate::combat::acquire::SearchContext::Area,
            );
            if found.is_some() {
                return 1;
            }
            idle(w);
        } else if holds || o.target.is_none() {
            // spec: S-0215 claim 26 (may not search: attempts its idle action)
            idle(w);
        } else {
            // spec: S-0215 claim 26 (the leash does not hold and a target is present: the approach step of claim 37)
            crate::approach::step(w, id);
        }
    }
    let Some(o) = w.obj(id).cloned() else {
        return NO_RULE_DELAY;
    };
    if o.guard_spot.is_none() && o.beh.queued.is_none() {
        // spec: S-0215 claim 26 (no guard spot and nothing queued: takes its own cell)
        if let Some(ob) = w.obj_mut(id) {
            ob.guard_spot = Some(Target::Cell(o.pos.cell()));
        }
    }
    let dr = rate_delay(w, id);
    // spec: S-0215 claim 26 and F0 (D = k × D_r + U(1, 5), k = 1 for every kind but aircraft)
    let d = dr.wrapping_add(w.rng.ranged(1, 5, half));
    // spec: S-0215 F3 (trunc(D / 6) under the close-range condition)
    if close_range(w, id) { d / 6 } else { d }
}

// ---------------------------------------------------------------------------
// Gates (S-0215 claim 35)

/// The stages of a gate's *open*, held in the progress count (S-0215 Terms,
/// *progress count*: start 0, opening 1, waiting 2, closing-start 3,
/// closing 4, finished 5).
mod stage {
    // spec: S-0215 Terms (progress count: the stages of a gate's open are 0 to 5 in the order of its table)
    pub const START: i32 = 0;
    pub const OPENING: i32 = 1; // spec: S-0215 Terms (stage opening 1)
    pub const WAITING: i32 = 2; // spec: S-0215 Terms (stage waiting 2)
    pub const CLOSING_START: i32 = 3; // spec: S-0215 Terms (stage closing-start 3)
    pub const CLOSING: i32 = 4; // spec: S-0215 Terms (stage closing 4)
    pub const FINISHED: i32 = 5; // spec: S-0215 Terms (stage finished 5)
}

/// Whether a building is a gate (S-0209 Terms, the gate flag).
fn is_gate(w: &World, id: ObjectId) -> bool {
    w.obj(id).is_some_and(|o| {
        o.kind == Kind::Building
            && w.rules()
                .buildings
                .get(o.type_index as usize)
                .is_some_and(|b| b.gate)
    })
}

/// A gate's door time Td and close delay Tc in ticks, a negative value
/// behaving as 0 (S-0215 F9).
fn gate_times(w: &World, id: ObjectId) -> (i32, i32) {
    w.obj(id)
        .and_then(|o| w.rules().buildings.get(o.type_index as usize))
        // spec: S-0215 F9 (a negative result behaves as 0)
        .map_or((0, 0), |b| {
            (b.gate_door_time.max(0), b.gate_close_delay.max(0))
        })
}

/// A gate's passage state (S-0215 claim 35): open when its current
/// behaviour is *open* and its door is at rest open, closed otherwise. This
/// is what S-0209's gate rows call an open or a closed gate.
pub fn gate_open(w: &World, gate: ObjectId) -> bool {
    // spec: S-0215 claim 35 (passage state open: open current and the door at rest open)
    w.obj(gate)
        .is_some_and(|o| o.beh.current == Some(Behaviour::Open) && o.beh.gate.door == Door::Open)
}

/// A closed gate in cell `c` whose owner is allied with `mover`'s house,
/// the gate owner's view (S-0209 rows WP12 and VP9; S-0207 claim 15).
pub fn closed_friendly_gate(w: &World, mover: ObjectId, c: CellXY) -> Option<ObjectId> {
    let owner = w.obj(mover)?.owner;
    w.cells.list(c, Layer::Ground).iter().copied().find(|&g| {
        is_gate(w, g) && !gate_open(w, g) && w.allied(w.obj(g).and_then(|x| x.owner), owner)
    })
}

/// The *closed friendly gate* outcome of S-0209 has arisen for `gate`
/// (S-0215 claim 35 (b)): unless *open* is current with the door opening or
/// at rest open, the gate drops any queued behaviour and *open* becomes
/// current at once, whatever its readiness (a behaviour start). The entry
/// rules call this when the outcome arises for a unit's move, never while
/// only asking for an outcome (S-0214 claim 2: asking changes nothing).
pub fn gate_met(w: &mut World, gate: ObjectId) {
    let stale = w.rules().quirks.stale_suspended_revert; // spec: S-0215 Edge cases (quirks.stale_suspended_revert)
    let t = w.tick;
    if !is_gate(w, gate) {
        return;
    }
    let Some(o) = w.obj_mut(gate) else { return };
    let moving_open = matches!(o.beh.gate.door, Door::Opening { .. } | Door::Open);
    // spec: S-0215 claim 35 (b) (open current with the door opening or at rest open: nothing changes)
    if o.beh.current == Some(Behaviour::Open) && moving_open {
        return;
    }
    // spec: S-0215 claim 35 (b) (drops any queued behaviour; open current at once, a behaviour start)
    o.beh.queued = None;
    o.beh.force(Behaviour::Open, t, stale);
}

/// The door motion at every turn of the gate (S-0215 claim 35 (d)): a
/// moving door is at rest on the first tick on which its time has passed
/// since the motion began. It moves whatever the gate's behaviour and
/// whether or not the gate processes in that turn, before its processing,
/// so a processing in the turn in which the door reaches rest finds it at
/// rest.
fn door_update(w: &mut World, id: ObjectId) {
    let t = w.tick;
    let Some(o) = w.obj_mut(id) else { return };
    // spec: S-0215 claim 35 (d) (at rest once Td ticks have passed since the motion began; at every turn of the gate, before its processing)
    o.beh.gate.door = match o.beh.gate.door {
        Door::Opening { since, ticks } if t.wrapping_sub(since) >= ticks => Door::Open,
        Door::Closing { since, ticks } if t.wrapping_sub(since) >= ticks => Door::Closed,
        d => d,
    };
}

/// Whether an object other than the gate is in the ground layer of one of
/// its foundation cells (S-0215 claim 35, stage *waiting*).
fn gate_cells_taken(w: &World, id: ObjectId) -> bool {
    let Some(o) = w.obj(id) else { return false };
    crate::objects::foundation_cells(o.pos.cell(), o.foundation)
        .into_iter()
        .any(|c| {
            w.cells
                .list(c, crate::world::Layer::Ground)
                .iter()
                .any(|&x| x != id)
        })
}

/// *Open* for a building (S-0215 claim 35 (c)); returns the delay.
fn process_open(w: &mut World, id: ObjectId) -> i32 {
    let t = w.tick;
    if !is_gate(w, id) {
        if let Some(ob) = w.obj_mut(id) {
            ob.beh.queue(Some(Behaviour::Guard));
        }
        // spec: S-0215 claim 35 (c) and Constants (a building that is not a gate: queues guard, 1 tick, no draw)
        return 1;
    }
    let (td, tc) = gate_times(w, id);
    let snaps = w.rules().quirks.gate_reverse_snaps;
    let Some(o) = w.obj(id).cloned() else {
        return NO_RULE_DELAY;
    };
    let gate = o.beh.gate;
    let set = |w: &mut World, next: i32, door: Door, restart: bool| {
        if let Some(ob) = w.obj_mut(id) {
            ob.beh.progress = next;
            ob.beh.gate.door = door;
            if restart {
                ob.beh.gate.countdown = t;
            }
        }
    };
    let queue_guard = |w: &mut World| {
        if let Some(ob) = w.obj_mut(id) {
            ob.beh.queue(Some(Behaviour::Guard));
        }
    };
    // spec: S-0215 Constants (the delay of a gate's open stages other than Std: 0 ticks, no draw)
    const NOW: i32 = 0;
    // spec: S-0215 claim 35 (c) (the stage table; Std is the standard delay with the rate of open, one draw)
    match o.beh.progress {
        stage::START => match gate.door {
            Door::Open => {
                // spec: S-0215 claim 35 (c) (start, at rest open: countdown restarted, waiting, Std)
                set(w, stage::WAITING, Door::Open, true);
                standard_delay(w, id)
            }
            Door::Closed => {
                // spec: S-0215 claim 35 (c) (start, at rest closed: the door starts opening, Td ticks; countdown restarted)
                let door = Door::Opening {
                    since: t,
                    ticks: td,
                };
                set(w, stage::OPENING, door, true);
                NOW
            }
            Door::Opening { .. } => {
                // spec: S-0215 claim 35 (c) (start, opening: countdown restarted)
                set(w, stage::OPENING, gate.door, true);
                NOW
            }
            Door::Closing { since, .. } => {
                // spec: S-0215 claim 35 (quirks.gate_reverse_snaps: the reversed door is at rest open at the next update; off: it retraces the closing's ticks)
                let ticks = if snaps {
                    0
                } else {
                    t.wrapping_sub(since).max(0)
                };
                set(w, stage::OPENING, Door::Opening { since: t, ticks }, true);
                NOW
            }
        },
        stage::OPENING => {
            match gate.door {
                // spec: S-0215 claim 35 (c) (opening, at rest open: waiting)
                Door::Open => set(w, stage::WAITING, Door::Open, false),
                // spec: S-0215 claim 35 (c) (opening, at rest closed: queues guard, finished)
                Door::Closed => {
                    queue_guard(w);
                    set(w, stage::FINISHED, Door::Closed, false);
                }
                // spec: S-0215 claim 35 (c) (opening, still moving: unchanged)
                _ => {}
            }
            NOW
        }
        stage::WAITING => {
            if gate_cells_taken(w, id) {
                // spec: S-0215 claim 35 (c) (waiting, an object in the ground layer of a foundation cell: countdown restarted)
                set(w, stage::WAITING, gate.door, true);
            } else if t.wrapping_sub(gate.countdown) >= tc {
                // spec: S-0215 claim 35 (c) (waiting, cells empty and the countdown run out: closing-start)
                set(w, stage::CLOSING_START, gate.door, false);
            }
            standard_delay(w, id)
        }
        stage::CLOSING_START => {
            // spec: S-0215 claim 35 (c) (closing-start: the door starts closing, Td ticks)
            let door = Door::Closing {
                since: t,
                ticks: td,
            };
            set(w, stage::CLOSING, door, false);
            NOW
        }
        stage::CLOSING => {
            if gate.door == Door::Closed {
                // spec: S-0215 claim 35 (c) (closing, at rest closed: queues guard, finished)
                queue_guard(w);
                set(w, stage::FINISHED, Door::Closed, false);
            }
            NOW
        }
        // spec: S-0215 claim 35 (c) (a finished gate changes nothing and takes Std)
        _ => standard_delay(w, id),
    }
}

/// The area-guard leash L (S-0215 F5) for a guard range g in leptons;
/// weapon ranges (g = 0) wait for M4 and count as 0.
fn leash(g: i32) -> i32 {
    // spec: S-0215 F5 (G = min(max(2g, 0), 4096); L = ⌊11G / 10⌋)
    let big_g = g.saturating_mul(2).clamp(0, 4096);
    11 * big_g / 10 // spec: S-0215 F5 (L = ⌊11G / 10⌋)
}

#[cfg(test)]
mod tests {
    use super::*;

    // spec: S-0215 F5 (L = ⌊11 × G / 10⌋ with G capped at 4096: at most 4505)
    #[test]
    fn leash_values() {
        assert_eq!(leash(0), 0);
        assert_eq!(leash(256), 563);
        assert_eq!(leash(5000), 4505);
        assert_eq!(leash(-3), 0);
    }
}
