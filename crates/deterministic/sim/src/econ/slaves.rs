//! Slave groups (S-0402 claims 14 and 19 to 25): a slave master's hidden
//! and working slaves, the group update, emergence, the slaves' own
//! harvest, delivery, regeneration and reload, the slave miner's directed
//! harvest, field seeking, deployment and relocation, and what becomes of
//! the slaves when the master is destroyed or sold (S-0312 claim 30,
//! S-0408 claim 10a).
//!
//! The interim rules this module takes where the specs leave a case open
//! are listed in docs/development.md (slave miners).

use glasnost_map::CellXY;
use glasnost_map::terrain::land;
use glasnost_num::Rounding;
use glasnost_num::geom::root_floor;
use glasnost_rules::sim::{LocomotionClass, MovementZone};
use serde::{Deserialize, Serialize};

use super::{harvest, resource};
use crate::cells::position;
use crate::combat::damage::Hit;
use crate::path::{NearbySettings, NearbyUse};
use crate::regions::{self, Reach};
use crate::rules::{Behaviour, Kind};
use crate::world::{HouseId, Layer, ObjectId, Pos, Target, World};

/// The period of uninterrupted group updates, and the delay of the first
/// one after creation.
pub const PERIOD: i32 = 10; // spec: S-0402 claim 21 (the first group update 10 ticks after creation; later ones 10 ticks apart)

/// The wait after a failed automatic deployment.
pub const RETRY_WAIT: i32 = 30; // spec: S-0402 claims 21 and 22 (a failed automatic deployment starts a 30-tick wait)

/// The state of a slave group (S-0402 claims 21 and 22).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum GroupState {
    /// At rest: no slave sent out since the group last came together.
    #[default]
    AtRest,
    /// Seeking a field (a vehicle master).
    FieldSeeking,
    /// Travelling to a deployment site (a vehicle master).
    Travelling,
    /// Waiting to retry a failed deployment.
    Retrying,
    /// Working: ready slaves are offered places.
    Working,
}

/// One slot of a slave group, in initial creation order.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Member {
    /// The slave, none after it was lost (until its replacement).
    pub id: Option<ObjectId>,
    /// Inside the master, off the map.
    pub inside: bool,
    /// Ready to leave.
    pub ready: bool,
    /// The tick its reload (inside, not ready) or regeneration (lost)
    /// delay started.
    pub since: i32,
}

/// A master's slave group (S-0402 claims 19 to 22).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SlaveGroup {
    /// The slave infantry type.
    pub slave_type: u32,
    /// Regeneration delay in ticks.
    pub regen: i32,
    /// Reload delay in ticks.
    pub reload: i32,
    /// The slots, in initial creation order.
    pub members: Vec<Member>,
    /// The group state.
    pub state: GroupState,
    /// The tick the group last came to rest (claim 14's s_g).
    pub last_rest: i32,
    /// The tick the next group update becomes due.
    pub next_update: i32,
    /// The cell remembered at an automatic pack-up (claim 22).
    pub remembered: Option<CellXY>,
    /// The group came to a vehicle by conversion and has not yet been seen
    /// by a group update of it (claim 22).
    pub pending_seek: bool,
    /// The site the current *harvest* sent the vehicle to, with that
    /// behaviour's start tick (claim 22): the vehicle drives there in
    /// *harvest* until *move* can start.
    pub directed: Option<(i32, CellXY)>,
}

/// A slave's harvest phase (S-0402 claim 14).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SlavePhase {
    /// Searching for a cell, or walking to one.
    #[default]
    Seeking,
    /// Loading bales in its cell.
    Loading,
    /// Walking back to the master.
    Returning,
}

/// A slave's own state (S-0402 claim 14).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SlaveState {
    /// Its master, none once it lost it.
    pub master: Option<ObjectId>,
    /// Its harvest phase.
    pub phase: SlavePhase,
    /// Bales per tiberium.
    pub load: Vec<i32>,
    /// The cell it remembered when it became full.
    pub remembered: Option<CellXY>,
    /// Sent back to its master by a recall (S-0213 claims 5 and 8).
    pub recalled: bool,
}

fn group_type(w: &World, id: ObjectId) -> Option<harvest::GroupType> {
    let o = w.obj(id)?;
    let h = &w.rules().econ.harvest;
    match o.kind {
        Kind::Vehicle => h.vehicles.get(o.type_index as usize)?.group,
        Kind::Building => h.buildings.get(o.type_index as usize)?.group,
        _ => None,
    }
}

fn group(w: &World, id: ObjectId) -> Option<SlaveGroup> {
    w.obj(id).and_then(|o| o.econ.group.clone())
}

fn set_group(w: &mut World, id: ObjectId, f: impl FnOnce(&mut SlaveGroup)) {
    if let Some(g) = w.obj_mut(id).and_then(|o| o.econ.group.as_mut()) {
        f(g);
    }
}

fn slave_state(w: &World, id: ObjectId) -> Option<SlaveState> {
    w.obj(id).and_then(|o| o.econ.slave.clone())
}

fn set_slave(w: &mut World, id: ObjectId, f: impl FnOnce(&mut SlaveState)) {
    let n = w.rules().econ.resource.tiberiums.len();
    if let Some(s) = w.obj_mut(id).and_then(|o| o.econ.slave.as_mut()) {
        if s.load.len() < n {
            // spec: S-0402 claim 15 (an amount for every tiberium)
            s.load.resize(n, 0);
        }
        f(s);
    }
}

fn stale(w: &World) -> bool {
    w.rules().quirks.stale_suspended_revert // spec: S-0215 Edge cases (quirks.stale_suspended_revert)
}

/// Whether an object has a slave group (S-0402 claim 19).
pub fn is_master(w: &World, id: ObjectId) -> bool {
    w.obj(id).is_some_and(|o| o.econ.group.is_some())
}

/// Whether an object is a slave with a master (S-0402 claim 14).
pub fn is_slave(w: &World, id: ObjectId) -> bool {
    master_of(w, id).is_some()
}

/// The master of a slave.
pub fn master_of(w: &World, id: ObjectId) -> Option<ObjectId> {
    w.obj(id)
        .and_then(|o| o.econ.slave.as_ref())
        .and_then(|s| s.master)
}

/// The slaves of a master's group that exist, in initial creation order.
pub fn slaves_of(w: &World, id: ObjectId) -> Vec<ObjectId> {
    group(w, id)
        .map(|g| g.members.iter().filter_map(|m| m.id).collect())
        .unwrap_or_default()
}

/// A copy of a master's group, if any.
pub fn group_of(w: &World, id: ObjectId) -> Option<SlaveGroup> {
    group(w, id)
}

/// A copy of a slave's state, if any.
pub fn slave_of(w: &World, id: ObjectId) -> Option<SlaveState> {
    slave_state(w, id)
}

/// Whether the master's group is at rest (S-0402 claim 14).
pub fn at_rest(w: &World, id: ObjectId) -> bool {
    group(w, id).is_some_and(|g| g.state == GroupState::AtRest)
}

/// Waking the slaves (S-0402 claims 14 and 22): a group at rest starts
/// field seeking.
pub fn wake(w: &mut World, id: ObjectId) {
    // spec: S-0402 claim 22 (waking under claim 14 starts field seeking, not immediate emergence)
    set_group(w, id, |g| {
        if g.state == GroupState::AtRest {
            g.state = GroupState::FieldSeeking;
        }
    });
}

/// Creates one slave of type `ty` for `master`, inside and off the map.
fn new_slave(w: &mut World, master: ObjectId, ty: u32, owner: Option<HouseId>) -> ObjectId {
    // spec: S-0402 claim 19 (each initial or replacement slave takes the ordinary object-creation draw, S-0202 claim 10; keeping it inside takes no placement draw)
    let s = crate::objects::create(w, Kind::Infantry, ty, owner);
    let n = w.rules().econ.resource.tiberiums.len();
    if let Some(o) = w.obj_mut(s) {
        o.econ.slave = Some(SlaveState {
            master: Some(master),
            load: vec![0; n],
            ..SlaveState::default()
        });
    }
    s
}

/// The creation of an object whose type names a slave type (S-0402 claims
/// 19 to 21): its group, with its initial slaves.
pub fn created(w: &mut World, id: ObjectId) {
    let Some(g) = group_type(w, id) else { return };
    // spec: S-0402 claim 19 (owned by the miner's house)
    let owner = w.obj(id).and_then(|o| o.owner);
    let t = w.tick;
    let mut members = Vec::new();
    // spec: S-0402 claim 19 (a positive count N creates N infantry of the named type; 0 or less creates none)
    for _ in 0..g.count.max(0) {
        let s = new_slave(w, id, g.slave, owner);
        // spec: S-0402 claim 19 (they start ready, inside and off the map)
        members.push(Member {
            id: Some(s),
            inside: true,
            ready: true,
            since: t,
        });
    }
    if let Some(o) = w.obj_mut(id) {
        o.econ.group = Some(SlaveGroup {
            slave_type: g.slave,
            regen: g.regen,
            reload: g.reload,
            members,
            // spec: S-0402 claim 21 (a new group is at rest, with last-rest tick 0)
            state: GroupState::AtRest,
            last_rest: 0,
            // spec: S-0402 claim 21 (its first group update becomes due 10 ticks after creation)
            next_update: t.wrapping_add(PERIOD),
            remembered: None,
            pending_seek: false,
            directed: None,
        });
    }
}

/// A conversion hands the group of `from` to `to` (S-0415 claims 6 and
/// 10; S-0402 claims 19 and 22).
pub fn transfer(w: &mut World, from: ObjectId, to: ObjectId) {
    let Some(mut g) = w.obj_mut(from).and_then(|o| o.econ.group.take()) else {
        return;
    };
    let to_vehicle = w.obj(to).is_some_and(|o| o.kind == Kind::Vehicle);
    if to_vehicle {
        // spec: S-0402 claim 22 (after conversion the group recalls its slaves and enters field seeking at the first group update that sees the vehicle)
        g.pending_seek = true;
    }
    for m in &g.members {
        if let Some(s) = m.id {
            set_slave(w, s, |x| x.master = Some(to));
        }
    }
    if let Some(o) = w.obj_mut(to) {
        // spec: S-0402 claim 19 (conversion preserves the working group; the slaves are not replaced)
        o.econ.group = Some(g);
    }
}

/// An object is about to be removed (`crate::objects::remove`): a master
/// that still has its group frees its slaves as after a sale (S-0408 claim
/// 10a), and a slave is lost to its master (S-0402 claim 14).
pub fn removing(w: &mut World, id: ObjectId) {
    if is_master(w, id) {
        master_sold(w, id);
    }
    lost(w, id);
}

/// A slave is killed or lost (S-0402 claim 14): its slot starts the
/// regeneration delay.
pub fn lost(w: &mut World, id: ObjectId) {
    let Some(m) = master_of(w, id) else { return };
    let t = w.tick;
    if let Some(o) = w.obj_mut(id) {
        o.econ.slave = None;
    }
    set_group(w, m, |g| {
        for x in g.members.iter_mut() {
            if x.id == Some(id) {
                // spec: S-0402 claim 14 (a killed or lost slave starts the slave regeneration field's delay)
                *x = Member {
                    id: None,
                    inside: false,
                    ready: false,
                    since: t,
                };
            }
        }
    });
}

/// The first existing house, in house-creation order, whose country
/// belongs to the civilian side (S-0312 claim 30; S-0408 claim 10a).
fn civilian_recipient(w: &World) -> Option<HouseId> {
    (0..w.houses.len())
        .map(|i| HouseId(i as u8))
        .find(|&h| super::setup::civilian(w, h))
}

/// A freed slave's guard reset: target and destination cleared, *guard*
/// queued (S-0312 claim 30; S-0408 claim 10a).
fn free_to_guard(w: &mut World, s: ObjectId) {
    if let Some(ob) = w.obj_mut(s) {
        ob.target = None;
        ob.combat.burst = 0; // spec: S-0301 claim 4 (a cleared target resets the burst counter)
        ob.queued_dests.clear();
        ob.path.steps.clear();
        ob.path.retry_at = 0;
        // spec: S-0312 claim 30 and S-0408 claim 10a (queued to guard)
        ob.beh.queue(Some(Behaviour::Guard));
    }
    super::assign_destination(w, s, None);
}

/// Ends a master's group: `hit` is the killing hit (S-0312 claim 30),
/// none for a removal by sale (S-0408 claim 10a).
fn end_group(w: &mut World, id: ObjectId, hit: Option<&Hit>) {
    let Some(g) = w.obj_mut(id).and_then(|o| o.econ.group.take()) else {
        return;
    };
    // spec: S-0312 claim 30 (an attacking house supplied without an attacker object does not stand in for it)
    let attacker = hit.and_then(|h| h.attacker).filter(|&a| w.obj(a).is_some());
    let attacker_house = attacker.and_then(|a| w.obj(a)).and_then(|a| a.owner);
    for m in &g.members {
        let Some(s) = m.id else { continue };
        let Some(so) = w.obj(s).cloned() else {
            continue;
        };
        // spec: S-0312 claim 30 and S-0408 claim 10a (every existing slave loses its master; the group makes no further replacements)
        if let Some(ob) = w.obj_mut(s) {
            ob.econ.slave = None;
        }
        if !so.on_map {
            if let Some(h) = hit {
                // spec: S-0312 claim 30 (a slave inside is destroyed, credited to the killing hit's known attacker; claim 29's common group of notifications)
                crate::combat::death::destruction_raises(w, s, h);
                super::stats::destruction(w, s, attacker);
            }
            // spec: S-0312 claim 30 and S-0408 claim 10a (a slave inside is removed without being placed)
            crate::objects::remove(w, s);
            continue;
        }
        // spec: S-0312 claim 30 (with a known attacker: the attacker's current house, even an ally of the master's; otherwise, and for a sale, the first civilian-side house, S-0408 claim 10a)
        let recipient = match (hit, attacker_house) {
            (Some(_), Some(h)) => Some(h),
            _ => civilian_recipient(w),
        };
        free_to_guard(w, s);
        match recipient {
            Some(h) => {
                if so.owner != Some(h) {
                    if let Some(ob) = w.obj_mut(s) {
                        ob.owner = Some(h);
                    }
                    super::events::owner_changed(w, s, so.owner);
                }
            }
            None => {
                let health = w.obj(s).map_or(0, |o| o.health);
                let warhead = w.rules().combat.general.c4_warhead;
                // spec: S-0312 claim 30 and S-0408 claim 10a (no recipient: a hit of its current health with the demolition warhead, no attacker, neither marker)
                crate::combat::damage::apply(
                    w,
                    s,
                    Hit {
                        amount: health,
                        warhead,
                        distance: 0,
                        attacker: None,
                        house: None,
                        ignore_defences: false,
                        no_escape: false,
                    },
                );
            }
        }
    }
}

/// A killing hit on a master, before its debris (S-0312 claim 30).
pub fn master_killed(w: &mut World, id: ObjectId, hit: &Hit) {
    end_group(w, id, Some(hit));
}

/// A master removed by a true sale (S-0408 claim 10a).
pub fn master_sold(w: &mut World, id: ObjectId) {
    end_group(w, id, None);
}

/// The master's unloading cell (S-0209 Terms; S-0402 claim 14): the cell
/// a vehicle stands in, or the foundation cell (W − 1, ⌊H/2⌋) of a
/// building counted from its top-left cell.
pub fn unloading_cell(w: &World, master: ObjectId) -> Option<CellXY> {
    let o = w.obj(master)?;
    if !o.on_map {
        return None;
    }
    let c = o.pos.cell();
    match o.kind {
        Kind::Building => {
            let (fw, fh) = o.foundation;
            // spec: S-0402 claim 14 and S-0209 Terms (the foundation cell W − 1 along x and ⌊H / 2⌋ along y from the top-left cell)
            Some(CellXY::new(c.x + fw - 1, c.y + fh.div_euclid(2)))
        }
        _ => Some(c),
    }
}

/// Sends the master's slaves on the map back to it (S-0213 claim 8 and
/// table T3 part (h); S-0402 claim 22): a guard reset, and each walks back
/// and goes inside.
pub fn recall(w: &mut World, master: ObjectId) {
    let t = w.tick;
    let stale = stale(w);
    for s in slaves_of(w, master) {
        if !w.obj(s).is_some_and(|o| o.on_map) {
            continue;
        }
        if let Some(ob) = w.obj_mut(s) {
            ob.target = None;
            ob.combat.burst = 0; // spec: S-0301 claim 4 (a cleared target resets the burst counter)
            ob.guard_spot = None;
            ob.queued_dests.clear();
            ob.path.steps.clear();
            ob.path.retry_at = 0;
            // spec: S-0213 table T3 part (h) (its slaves are sent back to guard it)
            ob.beh.force(Behaviour::Guard, t, stale);
        }
        super::assign_destination(w, s, None);
        set_slave(w, s, |x| {
            x.recalled = true;
            x.phase = SlavePhase::Returning;
        });
    }
}

fn walk_to(w: &mut World, id: ObjectId, c: CellXY) {
    if let Some(o) = w.obj_mut(id) {
        // A new destination starts a new path search (S-0212 claim 20).
        o.path.steps.clear();
        o.path.retry_at = 0;
    }
    super::assign_destination(w, id, Some(Target::Cell(c)));
}

fn move_to(w: &mut World, id: ObjectId, c: CellXY) {
    walk_to(w, id, c);
    if let Some(o) = w.obj_mut(id) {
        // spec: S-0402 claim 22 (it moves there with move queued)
        o.beh.queue(Some(Behaviour::Move));
    }
}

/// The group update of a master, called in its turn (S-0402 claims 21 and
/// 22).
pub fn group_turn(w: &mut World, id: ObjectId) {
    let Some(g) = group(w, id) else { return };
    let t = w.tick;
    // spec: S-0402 claim 21 (a group update when due, in the master's turn)
    if t.wrapping_sub(g.next_update) < 0 {
        return;
    }
    // spec: S-0402 claim 21 (subsequent uninterrupted group updates 10 ticks apart)
    set_group(w, id, |g| g.next_update = t.wrapping_add(PERIOD));
    regenerate(w, id);
    reload(w, id);
    recalled(w, id);
    match w.obj(id).map(|o| o.kind) {
        Some(Kind::Vehicle) => vehicle_update(w, id),
        Some(Kind::Building) => building_update(w, id),
        _ => {}
    }
    emerge(w, id);
}

/// Replacements for lost slaves whose regeneration delay has elapsed
/// (S-0402 claims 14, 19, 21 and 23).
fn regenerate(w: &mut World, id: ObjectId) {
    let Some(g) = group(w, id) else { return };
    let t = w.tick;
    // spec: S-0402 claim 23 (a replacement belongs to the master's current house)
    let owner = w.obj(id).and_then(|o| o.owner);
    for (i, m) in g.members.iter().enumerate() {
        // spec: S-0402 claim 21 (completion at the first update with t − a ≥ r)
        if m.id.is_some() || t.wrapping_sub(m.since) < g.regen {
            continue;
        }
        let s = new_slave(w, id, g.slave_type, owner);
        set_group(w, id, |g| {
            if let Some(x) = g.members.get_mut(i) {
                // spec: S-0402 claim 14 (a replacement is created inside the miner) and claim 19 (ready)
                *x = Member {
                    id: Some(s),
                    inside: true,
                    ready: true,
                    since: t,
                };
            }
        });
    }
}

/// Slaves inside whose reload delay has elapsed (S-0402 claims 14 and 21).
fn reload(w: &mut World, id: ObjectId) {
    let Some(g) = group(w, id) else { return };
    let t = w.tick;
    for (i, m) in g.members.iter().enumerate() {
        let Some(s) = m.id else { continue };
        // spec: S-0402 claim 21 (completion at the first update with t − a ≥ r)
        if !m.inside || m.ready || t.wrapping_sub(m.since) < g.reload {
            continue;
        }
        let strength = crate::combat::damage::strength(w, s);
        if let Some(o) = w.obj_mut(s) {
            // spec: S-0402 claim 14 (its health is restored to its strength)
            o.health = strength;
        }
        set_group(w, id, |g| {
            if let Some(x) = g.members.get_mut(i) {
                // spec: S-0402 claim 14 (it is ready to leave again)
                x.ready = true;
            }
        });
    }
}

/// Recalled slaves walk back to the master and go inside.
fn recalled(w: &mut World, id: ObjectId) {
    let Some(c) = unloading_cell(w, id) else {
        return;
    };
    for s in slaves_of(w, id) {
        let Some(o) = w.obj(s).cloned() else { continue };
        if !o.on_map || !slave_state(w, s).is_some_and(|x| x.recalled) {
            continue;
        }
        if o.pos.cell() == c && !crate::occupancy::in_progress(w, s) {
            deliver(w, id, s);
        } else if o.dest != Some(Target::Cell(c)) {
            // spec: S-0209 Terms (returning slaves are sent to the unloading cell with a plain move)
            move_to(w, s, c);
        }
    }
}

/// A slave delivers to its master and goes inside (S-0402 claims 14 and 23).
fn deliver(w: &mut World, master: ObjectId, s: ObjectId) {
    let load = slave_state(w, s).map(|x| x.load).unwrap_or_default();
    for (tib, &a) in load.iter().enumerate() {
        if a > 0 {
            // spec: S-0402 claim 14 (paid as in claim 12, to the miner's current owner, for each tiberium in turn, lowest index first; claim 23)
            harvest::pay(w, master, tib, a);
        }
    }
    // spec: S-0402 claim 14 (it goes inside the miner)
    crate::objects::take_inside(w, s);
    set_slave(w, s, |x| {
        x.load.iter_mut().for_each(|a| *a = 0);
        x.phase = SlavePhase::Seeking;
        x.recalled = false;
    });
    let t = w.tick;
    set_group(w, master, |g| {
        for m in g.members.iter_mut() {
            if m.id == Some(s) {
                // spec: S-0402 claim 14 (it starts the slave reload field's delay)
                m.inside = true;
                m.ready = false;
                m.since = t;
            }
        }
    });
}

/// The group update of a vehicle master (S-0402 claim 22).
fn vehicle_update(w: &mut World, id: ObjectId) {
    let Some(g) = group(w, id) else { return };
    let t = w.tick;
    if g.pending_seek {
        recall(w, id);
        // spec: S-0402 claim 22 (after conversion: recall and field seeking at the first group update that sees the vehicle)
        set_group(w, id, |g| {
            g.pending_seek = false;
            g.state = GroupState::FieldSeeking;
        });
    }
    let Some(o) = w.obj(id).cloned() else { return };
    match group(w, id).map(|g| g.state) {
        Some(GroupState::FieldSeeking) => {
            if o.dest.is_some() {
                // spec: S-0402 claim 22 (a seeking vehicle already with a destination enters travelling without replacing it)
                set_group(w, id, |g| g.state = GroupState::Travelling);
                return;
            }
            let range = w.rules().econ.harvest.slave_long_scan;
            // spec: S-0402 claim 22 (it seeks a field with the miner's long scan range and searches for a site near the result)
            let field = harvest::ring_search(w, id, o.pos.cell(), range);
            match field.and_then(|f| site_search(w, id, f)) {
                Some((_, s)) => {
                    // spec: S-0402 claim 22 (with a site it moves there and enters travelling)
                    move_to(w, id, s);
                    set_group(w, id, |g| g.state = GroupState::Travelling);
                }
                None => {
                    // spec: S-0402 claim 22 (no field or no site: at rest, the current tick as last-rest)
                    set_group(w, id, |g| {
                        g.state = GroupState::AtRest;
                        g.last_rest = t;
                    });
                }
            }
        }
        Some(GroupState::Travelling) => {
            // spec: S-0402 claim 22 (a travelling vehicle with no destination attempts deployment at its next group update)
            if o.dest.is_none() {
                try_deploy(w, id, GroupState::Travelling);
            }
        }
        Some(GroupState::Retrying) => {
            // spec: S-0402 claim 22 (the next update retries deployment)
            try_deploy(w, id, GroupState::Retrying);
        }
        Some(GroupState::Working) => {
            if o.econ.deploy.turning {
                try_deploy(w, id, GroupState::Working);
            } else if o.beh.seen() == Some(Behaviour::Guard) {
                // spec: S-0402 claim 22 (a working vehicle no longer deploying and in guard re-enters travelling)
                set_group(w, id, |g| g.state = GroupState::Travelling);
            }
        }
        _ => {}
    }
}

/// One automatic deployment attempt (S-0402 claim 22; S-0415 claims 2 to
/// 6), from the group state `from`.
fn try_deploy(w: &mut World, id: ObjectId, from: GroupState) {
    let t = w.tick;
    let Some(o) = w.obj(id).cloned() else { return };
    let attempt = match super::deploy::deploys_into(w, o.kind, o.type_index) {
        Some(b) => {
            if !o.econ.deploy.turning
                && let Some(ob) = w.obj_mut(id)
            {
                // A new attempt, not the continuation of a turn (S-0415 Terms: the first attempt).
                ob.econ.deploy = super::deploy::DeployState::default();
            }
            super::deploy::attempt(w, id, b)
        }
        // A vehicle without a deploys-into type cannot deploy.
        None => super::deploy::Attempt::Failed,
    };
    match attempt {
        super::deploy::Attempt::Deployed(b) => {
            // spec: S-0402 claim 22 (success puts the group into working)
            set_group(w, b, |g| g.state = GroupState::Working);
        }
        super::deploy::Attempt::Turning => {
            // spec: S-0402 claim 22 (a deploying vehicle's group is working)
            set_group(w, id, |g| g.state = GroupState::Working);
        }
        super::deploy::Attempt::Failed => {
            if from == GroupState::Retrying {
                // spec: S-0402 claim 22 (another failure puts the group into field seeking)
                set_group(w, id, |g| g.state = GroupState::FieldSeeking);
            } else {
                // spec: S-0402 claims 21 and 22 (failure enters a retry state and starts the 30-tick wait)
                set_group(w, id, |g| {
                    g.state = GroupState::Retrying;
                    g.next_update = t.wrapping_add(RETRY_WAIT);
                });
            }
        }
        super::deploy::Attempt::NoAttempt => {}
    }
}

fn in_construction_or_selling(w: &World, id: ObjectId) -> bool {
    w.obj(id).is_some_and(|o| {
        matches!(
            crate::econ::placement::seen_behaviour(o),
            Some(Behaviour::Construction | Behaviour::Selling)
        )
    })
}

/// The ring search of a building master: usable tiberium-land cells only
/// (a building has no movement zone for claim 4's connection test).
fn building_ring(w: &mut World, from: CellXY, range: i32) -> Option<CellXY> {
    harvest::ring_search_with(w, from, range, &mut |w, c| {
        // spec: S-0402 claim 4 (in the usable area; land type tiberium)
        w.cells.usable(c) && w.cells.cell(c).is_some_and(|x| x.land == land::TIBERIUM)
    })
}

/// The group update of a building master (S-0402 claims 22 and 25).
fn building_update(w: &mut World, id: ObjectId) {
    let Some(g) = group(w, id) else { return };
    // spec: S-0402 claim 22 (relocation checks after construction, excluding current construction and selling)
    if in_construction_or_selling(w, id) {
        return;
    }
    if g.state == GroupState::Travelling {
        // spec: S-0402 claim 22 (a travelling master that has become a building puts the group at rest)
        set_group(w, id, |g| g.state = GroupState::AtRest);
        return;
    }
    let Some(tl) = w.obj(id).map(|o| o.pos.cell()) else {
        return;
    };
    let (short, long, correction) = {
        let h = &w.rules().econ.harvest;
        (h.slave_short_scan, h.slave_long_scan, h.scan_correction)
    };
    // spec: S-0402 claim 22 (a short-scan field keeps it where it is and permits release of ready slaves)
    if building_ring(w, tl, short).is_some() {
        set_group(w, id, |g| g.state = GroupState::Working);
        return;
    }
    // spec: S-0402 claim 22 (with no short-scan field the long scan seeks another field; claim 24's site near it)
    let field = building_ring(w, tl, long);
    let site = field.and_then(|f| site_search(w, id, f).map(|(_, s)| (f, s)));
    let Some((f, s)) = site else {
        // Claim 25: no long-scan field, or no site; interim: it keeps working (docs/development.md).
        set_group(w, id, |g| g.state = GroupState::Working);
        return;
    };
    let (table, mode) = {
        let r = w.rules();
        (r.quirks.table_root_distance, r.mode)
    };
    // spec: S-0402 F7 (packing-up is requested exactly when c + d(S, F) < d(A, F); equality does not)
    if f7_packs_up(correction, s, tl, f, table, mode) && super::deploy::packs_up(w, id) {
        // spec: S-0402 claim 22 (the remembered cell of an automatic pack-up is the building's own top-left cell)
        set_group(w, id, |g| g.remembered = Some(tl));
        // spec: S-0402 claim 22 and S-0415 claims 9 and 10 (the automatic packing-up)
        super::sell::start_pack_up(w, id);
    } else {
        set_group(w, id, |g| g.state = GroupState::Working);
    }
}

/// F7 (S-0402): whether a deployed master at top-left cell `a` requests
/// packing-up for field `f` and shifted site `s`, with scan correction `c`
/// in whole cells.
pub fn f7_packs_up(c: i32, s: CellXY, a: CellXY, f: CellXY, table: bool, mode: Rounding) -> bool {
    let d = |p: CellXY, q: CellXY| -> i32 {
        // spec: S-0402 F7 (each coordinate difference reduced modulo 65536 to the signed range before squaring)
        let dx = i64::from(p.x.wrapping_sub(q.x) as i16);
        let dy = i64::from(p.y.wrapping_sub(q.y) as i16);
        // spec: S-0402 F7 (products and sum exact here; the table root of S-0204, quirks.table_root_distance, truncated)
        let n = (dx * dx + dy * dy) as u64;
        // spec: S-0402 F7 (the root reduced modulo 65536 to the signed range)
        i32::from(root_floor(n, table, mode) as u16 as i16)
    };
    // spec: S-0402 F7 (the final addition is signed 32-bit wrapping arithmetic)
    c.wrapping_add(d(s, f)) < d(a, f)
}

/// The deployment-site search (S-0402 claim 24) near `centre`: the
/// top-left cell found and the shifted vehicle destination.
pub fn site_search(w: &mut World, master: ObjectId, centre: CellXY) -> Option<(CellXY, CellXY)> {
    let o = w.obj(master)?.clone();
    // spec: S-0402 claim 24 (the master's building dimensions when deployed, its deploys-into type's when mobile, 1 × 1 without one)
    let size = match o.kind {
        Kind::Building => o.foundation,
        _ => super::deploy::deploys_into(w, o.kind, o.type_index)
            .map_or((1, 1), |b| super::placement::place_type(w, b).size),
    };
    regions::ensure(w);
    // spec: S-0402 claim 24 (foot, zone normal, the reach region of the master's own ground cell for normal; no deck mode, height or burrowing condition; no bridge top-left cell; no overlay)
    let reach = regions::reach_read(w, o.pos.cell(), MovementZone::Normal, false);
    let settings = NearbySettings {
        class: Some(LocomotionClass::Foot),
        zone: MovementZone::Normal,
        reach: Some(reach),
        deck: false,
        height: false,
        reference: Some(centre),
        no_bridges: true,
        no_overlay: true,
    };
    let extra = |w: &World, c: CellXY| footprint_ok(w, c, size, reach);
    let tl = crate::path::nearby_filtered(w, centre, NearbyUse::Settings(settings), Some(&extra))?;
    // spec: S-0402 claim 24 (x + 1 when W > 2, y + 1 when H > 2, independently)
    let s = CellXY::new(
        tl.x + i32::from(size.0 > 2), // spec: S-0402 claim 24 (the x shift)
        tl.y + i32::from(size.1 > 2), // spec: S-0402 claim 24 (the y shift)
    );
    Some((tl, s))
}

/// Every cell of the W × H rectangle at top-left `tl` qualifies (S-0402
/// claim 24).
fn footprint_ok(w: &World, tl: CellXY, size: (i32, i32), reach: Reach) -> bool {
    for dy in 0..size.1 {
        for dx in 0..size.0 {
            let c = CellXY::new(tl.x + dx, tl.y + dy);
            // spec: S-0402 claim 24 (the class, reach-region and empty-occupancy tests of S-0212 claim 25, which include the height-aware usable area, so the four corners too; no overlay)
            if !crate::path::qualifies(
                w,
                c,
                Some(LocomotionClass::Foot),
                MovementZone::Normal,
                Some(reach),
                false,
                false,
                0,
            ) || !crate::path::no_overlay(w, c)
            {
                return false;
            }
            // spec: S-0402 claim 24 (passability category open, S-0211 claim 11)
            if w.cells.category(c) != Some(crate::passability::category::OPEN) {
                return false;
            }
            // spec: S-0402 claim 24 (no terrain object or building)
            if w.cells.list(c, Layer::Ground).iter().any(|&x| {
                w.obj(x)
                    .is_some_and(|o| matches!(o.kind, Kind::Terrain | Kind::Building))
            }) {
                return false;
            }
            // spec: S-0402 claim 24 (no slope)
            if w.cells.cell(c).is_none_or(|x| x.slope != 0) {
                return false;
            }
        }
    }
    true
}

/// Ready hidden slaves are offered places (S-0402 claim 21).
fn emerge(w: &mut World, id: ObjectId) {
    let Some(g) = group(w, id) else { return };
    if g.state != GroupState::Working || in_construction_or_selling(w, id) {
        return;
    }
    let Some(o) = w.obj(id).cloned() else { return };
    let Some(c) = unloading_cell(w, id) else {
        return;
    };
    let layer = if o.kind == Kind::Building {
        Layer::Ground
    } else {
        o.layer
    };
    let t = w.tick;
    let stale = stale(w);
    let facing = o.body.value(t);
    // spec: S-0402 claim 21 (offers in reverse initial creation order)
    for (i, m) in g.members.iter().enumerate().rev() {
        let Some(s) = m.id else { continue };
        if !m.inside || !m.ready {
            continue;
        }
        // spec: S-0402 claim 21 and S-0208 claim 3 (the centre-class row: one ranged draw unless the vehicle-present or object-present refusal applies)
        let Some(p) = crate::walk::choose_position(w, c, layer, position::CENTRE, false) else {
            continue;
        };
        let (u, v) = crate::walk::standing_point(p);
        let (x, y) = (256 * c.x + u, 256 * c.y + v); // spec: S-0208 table P (a standing position within the cell)
        let z = w.surface_at(x, y);
        if let Some(so) = w.obj_mut(s) {
            so.layer = layer;
        }
        crate::objects::place(w, s, Pos::new(x, y, z), facing);
        if !w.obj(s).is_some_and(|x| x.on_map) {
            continue;
        }
        set_group(w, id, |g| {
            if let Some(x) = g.members.get_mut(i) {
                x.inside = false;
            }
        });
        set_slave(w, s, |x| {
            x.phase = SlavePhase::Seeking;
            x.recalled = false;
        });
        if let Some(so) = w.obj_mut(s) {
            // spec: S-0402 claim 21 (on emergence its target is the master and it starts claim 14's resource search)
            so.target = Some(Target::Object(id));
            so.beh.force(Behaviour::Harvest, t, stale);
        }
    }
}

fn slave_full(w: &World, id: ObjectId, load: &[i32]) -> bool {
    let cap = slave_rules(w, id).1;
    let total: i32 = load.iter().sum();
    // spec: S-0402 claim 14 (its capacity is its type's storage field); interim for a capacity of 0 or less: counted as full
    cap <= 0 || total >= cap
}

/// (harvest rate h, capacity) of a slave's type.
fn slave_rules(w: &World, id: ObjectId) -> (i32, i32) {
    let Some(o) = w.obj(id) else { return (1, 0) };
    w.rules()
        .econ
        .harvest
        .infantry
        .get(o.type_index as usize)
        .map_or((1, 0), |x| (x.harvest_rate, x.capacity))
}

/// The acceptability test of a slave's ring search: claim 4's, for an
/// infantry mover.
fn slave_accepts(w: &mut World, id: ObjectId, from: CellXY, c: CellXY, reference: i32) -> bool {
    // spec: S-0402 claim 4 (in the usable area, height-aware; land type tiberium)
    if !w.cells.usable(c) || w.cells.cell(c).is_none_or(|x| x.land != land::TIBERIUM) {
        return false;
    }
    let Some(z) = w
        .obj(id)
        .and_then(|o| w.rules().unit(o.kind, o.type_index))
        .map(|u| u.zone)
    else {
        return false;
    };
    regions::ensure(w);
    // spec: S-0402 claim 4 (the movement zone connects it with the searcher's cell)
    let connected = matches!(
        (regions::reach_in(w, from, z), regions::reach_in(w, c, z)),
        (Reach::Region(x), Reach::Region(y)) if x == y
    );
    // spec: S-0402 claim 4 (the entry outcome clear, here for an infantry mover, S-0209 claim 4)
    connected
        && crate::occupancy::infantry_outcome(w, id, c, reference)
            == crate::occupancy::Outcome::Clear
}

/// The processing of *harvest* by a slave (S-0402 claim 14). Returns the
/// delay: the harvest rate h of its type.
pub fn process_slave(w: &mut World, id: ObjectId) -> i32 {
    let (h, _) = slave_rules(w, id);
    let Some(m) = master_of(w, id) else { return h };
    let Some(st) = slave_state(w, id) else {
        return h;
    };
    let Some(o) = w.obj(id).cloned() else {
        return h;
    };
    let here = o.pos.cell();
    let moving = o.dest.is_some() || crate::occupancy::in_progress(w, id);
    match st.phase {
        SlavePhase::Seeking => {
            if slave_full(w, id, &st.load) {
                set_slave(w, id, |x| x.phase = SlavePhase::Returning);
                return_step(w, id, m);
                return h;
            }
            if let Some(c) = st.remembered {
                // spec: S-0402 claim 14 (it walks to the cell it remembered)
                set_slave(w, id, |x| x.remembered = None);
                walk_to(w, id, c);
                return h;
            }
            if moving {
                return h;
            }
            let range = w.rules().econ.harvest.slave_scan;
            let reference = crate::occupancy::reference_level(w, id);
            // spec: S-0402 claim 14 (ring search with the slave scan range, from itself)
            let found = harvest::ring_search_with(w, here, range, &mut |w, c| {
                slave_accepts(w, id, here, c, reference)
            });
            match found {
                Some(c) if c == here => set_slave(w, id, |x| x.phase = SlavePhase::Loading),
                // spec: S-0402 claim 14 (it walks to the cell found)
                Some(c) => walk_to(w, id, c),
                None => {
                    // spec: S-0402 claim 14 (when a search finds nothing it walks back with what it has)
                    set_slave(w, id, |x| x.phase = SlavePhase::Returning);
                    return_step(w, id, m);
                }
            }
        }
        SlavePhase::Loading => {
            if moving {
                return h;
            }
            let Some((tib, _)) = resource::resource_at(w, here) else {
                // spec: S-0402 claim 14 (when its cell is exhausted it searches again)
                set_slave(w, id, |x| x.phase = SlavePhase::Seeking);
                return h;
            };
            // spec: S-0402 claim 14 (it loads one bale, S-0401 claim 3, at each processing of its harvest)
            let got = resource::take_bales(w, here, 1);
            if got <= 0 {
                set_slave(w, id, |x| x.phase = SlavePhase::Seeking);
                return h;
            }
            set_slave(w, id, |x| {
                if let Some(a) = x.load.get_mut(tib) {
                    *a += got;
                }
            });
            let load = slave_state(w, id).map(|x| x.load).unwrap_or_default();
            if slave_full(w, id, &load) {
                // spec: S-0402 claim 14 (when full it remembers its cell and walks back to the miner)
                set_slave(w, id, |x| {
                    x.remembered = Some(here);
                    x.phase = SlavePhase::Returning;
                });
                return_step(w, id, m);
            }
        }
        SlavePhase::Returning => return_step(w, id, m),
    }
    h
}

/// A returning slave: delivery at the unloading cell, else it walks there
/// (S-0402 claim 14).
fn return_step(w: &mut World, id: ObjectId, m: ObjectId) {
    let Some(c) = unloading_cell(w, m) else {
        return;
    };
    let Some(o) = w.obj(id).cloned() else { return };
    if o.pos.cell() == c && !crate::occupancy::in_progress(w, id) {
        // spec: S-0402 claim 14 (a slave that reaches the miner delivers)
        deliver(w, m, id);
    } else if o.dest != Some(Target::Cell(c)) {
        // spec: S-0402 claim 14 (it walks back to the miner, to the unloading cell)
        walk_to(w, id, c);
    }
}

/// The *harvest* processing of a slave miner with a slave group (S-0402
/// claim 22), none for other objects.
pub fn process_harvest(w: &mut World, id: ObjectId) -> Option<i32> {
    if !is_master(w, id) || !harvest::is_slave_miner(w, id) {
        return None;
    }
    let t = w.tick;
    let o = w.obj(id)?.clone();
    if let Some((start, site)) = group(w, id).and_then(|g| g.directed)
        && start == o.beh.start_tick
        && o.dest == Some(Target::Cell(site))
    {
        // spec: S-0402 claim 22 (it moves to the site found, move queued; one search per harvest)
        return Some(1);
    }
    let centre = match o.dest {
        Some(Target::Cell(c)) => Some(c),
        Some(Target::Object(x)) => w.obj(x).map(|x| x.pos.cell()),
        None => None,
    };
    match centre {
        Some(c) => match site_search(w, id, c) {
            Some((_, s)) => {
                // spec: S-0402 claim 22 (an explicitly directed vehicle moves to a found site with move queued, travelling)
                move_to(w, id, s);
                let start = o.beh.start_tick;
                set_group(w, id, |g| {
                    g.state = GroupState::Travelling;
                    g.directed = Some((start, s));
                });
            }
            None => {
                if let Some(ob) = w.obj_mut(id) {
                    // spec: S-0402 claim 22 (the site search fails: guard queued, at rest, last-rest the current tick)
                    ob.beh.queue(Some(Behaviour::Guard));
                }
                set_group(w, id, |g| {
                    g.state = GroupState::AtRest;
                    g.last_rest = t;
                });
            }
        },
        None => {
            // Interim (docs/development.md): harvest without a destination queues guard.
            if let Some(ob) = w.obj_mut(id) {
                ob.beh.queue(Some(Behaviour::Guard));
            }
        }
    }
    // Interim (docs/development.md): the delay of 1 tick.
    Some(1)
}
