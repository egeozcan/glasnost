//! Behaviour processing (S-0215): the per-tick start, readiness, falling
//! back to rest and the processings of the general-purpose behaviours.
//!
//! Seams for milestone M4 (combat): the target-acquisition specs are not
//! written, so no automatic search runs (claims 26, 27 and 33 see "may not
//! search"), approaching and engaging a target does nothing, and weapon
//! ranges and rearm times are unknown (the close-range shortening of F3
//! applies only through the infantry close-range flag; no actor is ever
//! rearming). The behaviours that other topics own (claim 34) and *hunt*,
//! *capture*, *sabotage*, *enter*, and *unload* for vehicles, are processed as behaviours
//! without a processing rule until their topics are implemented; that is
//! recorded as a note on the world.

use glasnost_num::geom::root_floor;

use crate::infantry::{self, Seq};
use crate::occupancy;
use crate::rules::{Behaviour, Kind};
use crate::world::{ObjectId, Pos, Target, World};

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
            !(moving(w, id) && !below && !excused)
        }
        Kind::Building => o.beh.ready_mark,
        Kind::Aircraft | Kind::Terrain => false,
    }
}

/// The standard delay D = D_r + U(0, 2) (S-0215 F2), the draw last.
fn standard_delay(w: &mut World, id: ObjectId) -> i32 {
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
    // spec: S-0215 F3 (an infantry type with the close-range flag; weapon ranges wait for M4)
    let flag = o.kind == Kind::Infantry
        && r.unit(o.kind, o.type_index)
            .is_some_and(|u| u.raw.close_range.unwrap_or(false));
    if !flag {
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
fn target_pos(w: &World, t: Target) -> Pos {
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

fn has_ability(u: &crate::rules::UnitRules, name: &str) -> bool {
    u.abilities
        .0
        .iter()
        .chain(u.abilities.1.iter())
        .any(|a| a == name)
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
    // spec: S-0215 claim 14 rows 11 and 12 (our engine: area guard with the ability or the type default, else guard)
    if has_ability(u, "GUARD_AREA") || u.default_to_guard_area {
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
        queue(
            w,
            if placement || on_ore {
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
    let area = has_ability(u, "GUARD_AREA") || u.default_to_guard_area;
    // spec: S-0215 claim 15 row 10 (skill below the area-guard threshold, without the ability or the type default)
    if skill < r.general.iq_guard_area && !area {
        queue(w, Behaviour::Guard);
        return;
    }
    // spec: S-0215 claim 15 row 12 (otherwise area guard)
    queue(w, Behaviour::AreaGuard);
}

/// One actor's behaviour part of its turn (S-0215 claims 2, 3, 11 and 33).
pub fn turn(w: &mut World, id: ObjectId) {
    let t = w.tick;
    let Some(o) = w.obj(id) else { return };
    let kind = o.kind;
    if kind == Kind::Terrain || kind == Kind::Aircraft {
        return;
    }
    // spec: S-0215 claim 11 (the per-tick start of infantry, vehicles and buildings)
    let ready_now = ready(w, id);
    let Some(o) = w.obj(id) else { return };
    if ready_now {
        if kind == Kind::Infantry && o.beh.current.is_none() && o.beh.queued.is_none() {
            // spec: S-0215 claim 11 (an infantry unit with neither falls back to rest at its per-tick start)
            fall_back(w, id, false);
        }
        if let Some(ob) = w.obj_mut(id)
            && ob.beh.start_next(t)
            && kind == Kind::Building
        {
            // spec: S-0215 claim 11 (each successful per-tick start of a building clears its ready mark)
            ob.beh.ready_mark = false;
        }
    }
    // The drop rule of opportunity targets (claim 33) needs opportunity targets, which only the M4 search finds.
    let Some(o) = w.obj(id) else { return };
    // spec: S-0215 claim 2 (alive with health above 0 and the timer expired)
    if o.health <= 0 || !o.beh.due(t) {
        if kind == Kind::Building {
            second_start(w, id);
        }
        return;
    }
    let d = process(w, id);
    if let Some(ob) = w.obj_mut(id) {
        ob.beh.processed(t, d);
    }
    if kind == Kind::Building {
        second_start(w, id);
    }
}

/// A building's second per-tick start (S-0215 claim 11).
fn second_start(w: &mut World, id: ObjectId) {
    let t = w.tick;
    if let Some(ob) = w.obj_mut(id)
        && ob.beh.ready_mark
        && ob.beh.start_next(t)
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
        (Some(Behaviour::Unload), Kind::Infantry) => process_unload_infantry(w, id),
        (
            Some(Behaviour::Guard | Behaviour::Sticky | Behaviour::AreaGuard | Behaviour::Attack),
            Kind::Building,
        ) => {
            // spec: S-0215 claim 32 (an armed building's guard sets its ready mark); firing waits for M4
            let armed = w.rules().buildings[o.type_index as usize]
                .raw
                .primary
                .is_some();
            if armed && let Some(ob) = w.obj_mut(id) {
                ob.beh.ready_mark = true;
            }
            w.note("S-0215 claim 32: building guard and attack processings are partial until combat (M4)");
            standard_delay(w, id)
        }
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

/// The deployer rules of S-0215 claim 31 for a deployed unit, applied when
/// one applies (returning its delay); `None` when none applies (an
/// undeployed unit: the last row is the original's computer owners only).
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
        return None;
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

/// Unload for infantry (S-0215 claim 30).
fn process_unload_infantry(w: &mut World, id: ObjectId) -> i32 {
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
        ob.beh.force(Behaviour::Guard, t);
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
    if o.target.is_none() {
        // spec: S-0215 claim 23 (fall-back: no target)
        fall_back(w, id, false);
    }
    // Approaching and engaging the target belong to the combat topic (M4).
    let d = standard_delay(w, id);
    // spec: S-0215 claim 23 and F3 (halved under the close-range condition, the draw taken either way)
    if close_range(w, id) { d / 2 } else { d }
}

/// Guard (S-0215 claim 24).
fn process_guard(w: &mut World, id: ObjectId) -> i32 {
    let Some(o) = w.obj(id).cloned() else {
        return NO_RULE_DELAY;
    };
    if o.target.is_none() && o.kind == Kind::Infantry {
        // spec: S-0215 claim 24 (no target: attempts its idle action, S-0107 claim 14)
        let in_progress = moving(w, id);
        infantry::idle_attempt(w, id, in_progress);
    }
    // spec: S-0215 claim 24 (the delay table: rearming, distributed fire, otherwise the standard delay)
    standard_delay(w, id)
}

/// Area guard (S-0215 claim 26).
fn process_area_guard(w: &mut World, id: ObjectId) -> i32 {
    let t = w.tick;
    let r = w.rules_arc();
    let Some(o) = w.obj(id).cloned() else {
        return NO_RULE_DELAY;
    };
    let Some(u) = r.unit(o.kind, o.type_index).cloned() else {
        return NO_RULE_DELAY;
    };
    let half = r.quirks.ranged_draw_half_span;
    if o.kind == Kind::Vehicle && u.harvester {
        // spec: S-0215 claim 26 row 4 (a harvester: harvest at once; 1 + U(1, 10))
        if let Some(ob) = w.obj_mut(id) {
            ob.beh.queue_and_start_at_once(Behaviour::Harvest, t);
        }
        return 1i32.wrapping_add(w.rng.ranged(1, 10, half)); // spec: S-0215 claim 26 row 4 (1 + U(1, 10))
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
            r.general
                .guard_stray
                .unwrap_or_else(|| leash(u.guard_range))
        } else {
            leash(u.guard_range)
        };
        // spec: S-0215 claim 26 (the leash: not firing, no destination, distance above L)
        let holds = o.dest.is_none() && dist > i64::from(leash);
        if holds {
            // spec: S-0215 claim 26 (drops any target, takes the guard spot as destination; may not search: idle action)
            if let Some(ob) = w.obj_mut(id) {
                ob.target = None;
                ob.dest = Some(gs);
            }
            if o.kind == Kind::Infantry {
                let ip = moving(w, id);
                infantry::idle_attempt(w, id, ip);
            }
        } else if o.target.is_none() && o.kind == Kind::Infantry {
            // spec: S-0215 claim 26 (no leash, no target, may not search: attempts its idle action)
            let ip = moving(w, id);
            infantry::idle_attempt(w, id, ip);
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
