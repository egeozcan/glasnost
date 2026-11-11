//! Slave groups (S-0402 claims 14 and 19 to 28): a slave master's hidden
//! and working slaves, the group update in slave-place order, emergence,
//! the slaves' own load steps, delivery, regeneration and reload, the
//! slave miner's directed harvest, field seeking, deployment and
//! relocation, and what becomes of the slaves when the master is destroyed,
//! sold or otherwise removed (S-0312 claims 30 and 31, S-0408 claim 10a,
//! S-0415 claim 11).
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
use crate::rules::{Behaviour, FactoryKind, Kind};
use crate::world::{HouseId, Layer, ObjectId, Pos, Target, World};

/// The period of uninterrupted group updates, and the delay of the first
/// one after creation.
pub const PERIOD: i32 = 10; // spec: S-0402 claim 21 (the first group update 10 ticks after creation; later ones 10 ticks apart)

/// The wait after a failed automatic deployment.
pub const RETRY_WAIT: i32 = 30; // spec: S-0402 claims 21 and 22 (a failed automatic deployment starts a 30-tick wait)

/// The state of a slave group (S-0402 claims 21, 22 and 27).
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
    /// Working: a vehicle's ready slaves are offered places; a building
    /// waits here for the end of its construction state.
    Working,
    /// Checking the field (a deployed master): the field and relocation
    /// rule, and emergence unless packing is requested.
    CheckingField,
    /// Packing: waiting for the conversion into a vehicle.
    Packing,
}

/// One slave place of a group (S-0402 Terms, *slave place*), numbered by
/// initial creation order; a replacement keeps its place.
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
    w.rules().econ.harvest.group_of(o.kind, o.type_index)
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
fn new_slave(
    w: &mut World,
    master: ObjectId,
    ty: u32,
    owner: Option<HouseId>,
    chain: &mut crate::objects::Chain,
) -> ObjectId {
    // spec: S-0402 claim 19 (each initial or replacement slave takes the ordinary object-creation draw, S-0202 claim 10, then its creation number; keeping it inside takes no placement draw)
    let s = crate::objects::create_in(w, Kind::Infantry, ty, owner, chain);
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
    let Some(t) = w.obj(id).map(|o| (o.kind, o.type_index)) else {
        return;
    };
    created_in(w, id, &mut vec![t]);
}

/// [`created`] inside a creation set whose enclosing creations, the new
/// master's own type last, are `chain` (S-0611 claim 14 and F5).
pub(crate) fn created_in(w: &mut World, id: ObjectId, chain: &mut crate::objects::Chain) {
    let Some(g) = group_type(w, id) else { return };
    // spec: S-0402 claim 19 (owned by the miner's house)
    let owner = w.obj(id).and_then(|o| o.owner);
    let t = w.tick;
    let mut members = Vec::new();
    // spec: S-0402 claim 19 (a positive count N creates N infantry of the named type; 0 or less creates none)
    for _ in 0..g.count.max(0) {
        // spec: S-0402 claim 19 and S-0611 F5 (in place order, each slave's own creation set before the next slave)
        let s = new_slave(w, id, g.slave, owner, chain);
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
        });
    }
}

/// A conversion hands the group of `from` to `to` (S-0415 claims 6 and
/// 10; S-0402 claims 19 and 22).
pub fn transfer(w: &mut World, from: ObjectId, to: ObjectId) {
    if let Some(g) = detach(w, from) {
        attach(w, to, g);
    }
}

/// Takes a master's group off it, to be attached to the object of a
/// successful conversion or ended (S-0415 claims 10 and 11).
pub fn detach(w: &mut World, id: ObjectId) -> Option<SlaveGroup> {
    w.obj_mut(id).and_then(|o| o.econ.group.take())
}

/// Makes `to` the master of a detached group (S-0415 claims 6 and 10).
/// The conversion itself does not change the group state (S-0415 claim
/// 10; S-0402 claims 22, 25 and 28 give the transitions at its updates).
pub fn attach(w: &mut World, to: ObjectId, g: SlaveGroup) {
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
/// that still has its group ends it as a removal without a killing hit
/// (S-0312 claim 31), and a slave is lost to its master (S-0402 claim 14).
pub fn removing(w: &mut World, id: ObjectId) {
    if let Some(g) = detach(w, id) {
        // spec: S-0312 claim 31 (removal without a killing hit ends an untransferred group)
        end_group(w, g, End::Removed);
    }
    lost(w, id);
}

/// A production item's master is deleted before it was ever placed
/// (S-0404 claim 6): the slaves created with it at production start
/// (S-0402 claim 19), hidden inside it, go out of the game with it,
/// silently, as a removed spawner's docked slot objects do (S-0506 claim
/// 13). S-0312 claim 31 leaves never-placed masters out (Q-0193), and
/// Q-2973 asks whether the game counts or announces them.
pub(crate) fn unplaced_master_deleted(w: &mut World, id: ObjectId) {
    let Some(g) = detach(w, id) else { return };
    for m in &g.members {
        let Some(s) = m.id else { continue };
        if let Some(ob) = w.obj_mut(s) {
            ob.econ.slave = None;
        }
        // spec: S-0407 claim 12 (g) and (o) (a unit's end is the tech tree's event, as for its master)
        super::tech::object_ends(w, s);
        // nocite: our own input validation (#156): a cancelled item leaves no hidden slave behind; removed without destruction or notification until Q-2973 is answered
        crate::objects::remove(w, s);
    }
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
/// queued (S-0312 claims 30 and 31; S-0408 claim 10a).
fn free_to_guard(w: &mut World, s: ObjectId) {
    if let Some(ob) = w.obj_mut(s) {
        crate::spawn::clear_own_target(ob);
        ob.combat.burst = 0; // spec: S-0301 claim 4 (a cleared target resets the burst counter)
        ob.queued_dests.clear();
        ob.path.steps.clear();
        // spec: S-0212 claim 21 and its state table, row "destination changes" (r97: for the driving and walk locomotors a different destination, or none, ends a running retry delay, path::destination_changed and path::waits; the ship and the other locomotors keep it, low confidence; the hovering locomotor's is S-0503's)
        // spec: S-0312 claims 30 and 31 and S-0408 claim 10a (queued to guard)
        ob.beh.queue(Some(Behaviour::Guard));
    }
    super::assign_destination(w, s, None);
}

/// How a group ends.
#[derive(Clone, Copy)]
enum End<'a> {
    /// A killing hit on the master (S-0312 claim 30).
    Killed(&'a Hit),
    /// A true sale of a building master (S-0408 claim 10a).
    Sold,
    /// Any other removal of the master without a killing hit, a failed
    /// pack-up included (S-0312 claim 31; S-0415 claim 11).
    Removed,
}

/// Ends a detached group (S-0312 claims 30 and 31; S-0408 claim 10a).
fn end_group(w: &mut World, g: SlaveGroup, how: End<'_>) {
    let hit = match how {
        End::Killed(h) => Some(h),
        End::Sold | End::Removed => None,
    };
    // spec: S-0312 claim 30 (an attacking house supplied without an attacker object does not stand in for it)
    let attacker = hit.and_then(|h| h.attacker).filter(|&a| w.obj(a).is_some());
    let attacker_house = attacker.and_then(|a| w.obj(a)).and_then(|a| a.owner);
    for m in &g.members {
        let Some(s) = m.id else { continue };
        let Some(so) = w.obj(s).cloned() else {
            continue;
        };
        // spec: S-0312 claims 30 and 31 and S-0408 claim 10a (every existing slave loses its master; the group makes no further replacements)
        if let Some(ob) = w.obj_mut(s) {
            ob.econ.slave = None;
        }
        if !so.on_map {
            match how {
                End::Killed(h) => {
                    // spec: S-0312 claim 30 (a slave inside is destroyed, credited to the killing hit's known attacker; claim 29's common group of notifications)
                    crate::combat::death::destruction_raises(w, s, h);
                    // spec: S-0312 claim 30 (credited to the killing hit's known attacker, if any) and S-0414 claim 21 (a destruction that is not a hit counts for the house of the credited object)
                    super::stats::destruction(w, s, super::stats::Credit::object(attacker));
                }
                // spec: S-0312 claim 31 (destroyed without attacker credit, with claim 30's common destruction notifications; no death effects of its own)
                // spec: S-0408 claim 10a (a sold master's slave inside is destroyed without being placed: a loss for its current house without attacker credit, S-0414 claim 21, and claim 29's common group without an attacker; no death effects of its own; Q-0440)
                End::Removed | End::Sold => {
                    crate::combat::death::destruction_raises(w, s, &no_attacker_hit(w, 0));
                    super::stats::destruction(w, s, super::stats::Credit::Nobody);
                }
            }
            // spec: S-0312 claims 30 and 31 and S-0408 claim 10a (a slave inside is destroyed and removed without being placed; the notifications' actions finish first)
            crate::objects::remove(w, s);
            continue;
        }
        // spec: S-0312 claim 30 (with a known attacker: the attacker's current house, even an ally of the master's; otherwise the first civilian-side house); claim 31 and S-0408 claim 10a (the first civilian-side house)
        let recipient = match (hit, attacker_house) {
            (Some(_), Some(h)) => Some(h),
            _ => civilian_recipient(w),
        };
        match recipient {
            Some(h) => {
                // spec: S-0312 claims 30 and 31 and S-0408 claim 10a (a transferred slave: target and destination cleared, guard queued)
                free_to_guard(w, s);
                if so.owner != Some(h) {
                    if let Some(ob) = w.obj_mut(s) {
                        ob.owner = Some(h);
                    }
                    super::events::owner_changed(w, s, so.owner);
                }
                // spec: S-0312 claims 30 and 31 and S-0408 claim 10a (a successful transfer requests cheer, S-0107 claim 21, after the target and destination reset; not forced, S-0107 claim 6)
                crate::infantry::request(w, s, crate::infantry::Seq::Cheer, false);
            }
            None => {
                let health = w.obj(s).map_or(0, |o| o.health);
                // spec: S-0312 claims 30 and 31 and S-0408 claim 10a (no recipient: instead a hit of its current health with the demolition warhead, no attacker, neither marker; no target or destination reset, guard or cheer)
                crate::combat::damage::apply(w, s, no_attacker_hit(w, health));
            }
        }
    }
}

/// A hit of `amount` with the demolition warhead, no attacker and neither
/// marker (S-0312 claims 30 and 31; S-0408 claim 10a).
fn no_attacker_hit(w: &World, amount: i32) -> Hit {
    Hit {
        amount,
        warhead: w.rules().combat.general.c4_warhead,
        distance: 0,
        attacker: None,
        house: None,
        ignore_defences: false,
        no_escape: false,
    }
}

/// A killing hit on a master, before its debris (S-0312 claim 30).
pub fn master_killed(w: &mut World, id: ObjectId, hit: &Hit) {
    if let Some(g) = detach(w, id) {
        end_group(w, g, End::Killed(hit));
    }
}

/// A building master about to be removed by a true sale (S-0408 claim
/// 10a), called before its removal.
pub fn master_sold(w: &mut World, id: ObjectId) {
    if let Some(g) = detach(w, id) {
        end_group(w, g, End::Sold);
    }
}

/// A detached group ended as a removal without a killing hit (S-0312
/// claim 31): the old group of a building whose pack-up failed at its
/// return cell (S-0415 claim 11), and the return vehicle's temporary
/// hidden group of a pack-up (S-0402 claim 19a).
pub fn end_removed(w: &mut World, g: SlaveGroup) {
    end_group(w, g, End::Removed);
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
            crate::spawn::clear_own_target(ob);
            ob.combat.burst = 0; // spec: S-0301 claim 4 (a cleared target resets the burst counter)
            ob.guard_spot = None;
            ob.queued_dests.clear();
            ob.path.steps.clear();
            // spec: S-0212 claim 21 and its state table, row "destination changes" (r97: for the driving and walk locomotors a different destination, or none, ends a running retry delay, path::destination_changed and path::waits; the ship and the other locomotors keep it, low confidence; the hovering locomotor's is S-0503's)
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
        // spec: S-0212 claim 21 and its state table, row "destination changes" (r97: for the driving and walk locomotors a different destination, or none, ends a running retry delay, path::destination_changed and path::waits; the ship and the other locomotors keep it, low confidence; the hovering locomotor's is S-0503's)
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

/// The group update of a master, called in its turn (S-0402 claims 21,
/// 22 and 25 to 28; S-0203 claim 12).
pub fn group_turn(w: &mut World, id: ObjectId) {
    let Some(g) = group(w, id) else { return };
    let t = w.tick;
    // spec: S-0402 claim 21 and S-0203 claim 12 (a due group update happens in the master's turn)
    if t.wrapping_sub(g.next_update) < 0 {
        return;
    }
    // spec: S-0402 claim 21 (subsequent uninterrupted group updates 10 ticks apart)
    set_group(w, id, |g| g.next_update = t.wrapping_add(PERIOD));
    // spec: S-0402 claim 26 and S-0203 claim 12 (the slaves' group-state effects in slave-place order, earliest first, one pass, before the master's decisions)
    for i in 0..g.members.len() {
        place_effects(w, id, i);
    }
    let emergence = match w.obj(id).map(|o| o.kind) {
        Some(Kind::Vehicle) => vehicle_update(w, id),
        Some(Kind::Building) => building_update(w, id),
        _ => false,
    };
    if emergence {
        emerge(w, id);
    }
}

/// The group-state effects of slave place `i` (S-0402 claims 14, 21 and
/// 26): a replacement, a reload completion, or an outside slave's search,
/// transitions and delivery.
fn place_effects(w: &mut World, id: ObjectId, i: usize) {
    let Some(g) = group(w, id) else { return };
    let Some(m) = g.members.get(i).cloned() else {
        return;
    };
    let t = w.tick;
    let Some(s) = m.id else {
        // spec: S-0402 claims 21 and 26 (every place whose loss timer has expired, t − a ≥ r, gets a replacement attempt)
        if t.wrapping_sub(m.since) >= g.regen {
            // spec: S-0402 claim 23 (a replacement belongs to the master's current house)
            let owner = w.obj(id).and_then(|o| o.owner);
            // A fresh creation, outside any other creation in progress (an empty chain).
            let s = new_slave(w, id, g.slave_type, owner, &mut Vec::new());
            set_group(w, id, |g| {
                if let Some(x) = g.members.get_mut(i) {
                    // spec: S-0402 claim 14 (a replacement is created inside the miner), claim 19 (ready) and the Terms (it keeps its place)
                    *x = Member {
                        id: Some(s),
                        inside: true,
                        ready: true,
                        since: t,
                    };
                }
            });
        }
        return;
    };
    if m.inside {
        // spec: S-0402 claim 21 (completion at the first update with t − a ≥ r)
        if !m.ready && t.wrapping_sub(m.since) >= g.reload {
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
        return;
    }
    if w.obj(s).is_some_and(|o| o.on_map) {
        outside_slave(w, id, s);
    }
}

/// An outside slave's group-update effects (S-0402 claims 14 and 26):
/// seeking, arrival at a resource cell, a full or exhausted load and
/// delivery.
fn outside_slave(w: &mut World, master: ObjectId, s: ObjectId) {
    let Some(st) = slave_state(w, s) else { return };
    if st.phase == SlavePhase::Returning {
        return_step(w, s, master);
        return;
    }
    let Some(o) = w.obj(s).cloned() else { return };
    let here = o.pos.cell();
    if slave_full(w, s, &st.load) {
        // spec: S-0402 claims 14 and 26 (a full load, detected at a group update: it remembers its cell and walks back to the miner)
        set_slave(w, s, |x| {
            x.remembered = Some(here);
            x.phase = SlavePhase::Returning;
        });
        return_step(w, s, master);
        return;
    }
    // spec: S-0402 claim 26 (arrival is checked at a group update: a slave still walking waits)
    if o.dest.is_some() || crate::occupancy::in_progress(w, s) {
        return;
    }
    if let Some(c) = st.remembered {
        set_slave(w, s, |x| x.remembered = None);
        if c != here {
            // spec: S-0402 claim 14 (it walks to the cell it remembered)
            walk_to(w, s, c);
            return;
        }
    }
    // spec: S-0402 claim 26 (a slave actually loading in harvest on its resource cell keeps loading in its own turns)
    if o.beh.current == Some(Behaviour::Harvest) && resource::resource_at(w, here).is_some() {
        return;
    }
    let range = w.rules().econ.harvest.slave_scan;
    let reference = crate::occupancy::reference_level(w, s);
    // spec: S-0402 claims 14 and 26 (it searches again: ring search with the slave scan range, from itself; no draws)
    let found = harvest::ring_search_with(w, here, range, &mut |w, c| {
        slave_accepts(w, s, c, reference)
    });
    match found {
        Some(c) if c == here => {
            set_slave(w, s, |x| x.phase = SlavePhase::Loading);
            if o.beh.current != Some(Behaviour::Harvest)
                && let Some(ob) = w.obj_mut(s)
            {
                // spec: S-0402 claim 26 (arrival at a selected resource cell queues harvest again)
                ob.beh.queue(Some(Behaviour::Harvest));
            }
        }
        Some(c) => {
            set_slave(w, s, |x| x.phase = SlavePhase::Seeking);
            // spec: S-0402 claim 14 (it walks to the cell found)
            walk_to(w, s, c);
        }
        None => {
            // spec: S-0402 claim 14 (when a search finds nothing it walks back with what it has)
            set_slave(w, s, |x| x.phase = SlavePhase::Returning);
            return_step(w, s, master);
        }
    }
}

/// A returning slave at a group update: delivery in the unloading cell
/// with no destination, else it walks there (S-0402 claims 14 and 26;
/// S-0213 claim 8).
fn return_step(w: &mut World, id: ObjectId, m: ObjectId) {
    let Some(c) = unloading_cell(w, m) else {
        return;
    };
    let Some(o) = w.obj(id).cloned() else { return };
    let idle = o.dest.is_none() && !crate::occupancy::in_progress(w, id);
    if o.pos.cell() == c && idle {
        // spec: S-0402 claim 14 (at a group update, a returning slave in the unloading cell with no destination delivers)
        deliver(w, m, id);
    } else if o.dest != Some(Target::Cell(c)) {
        // spec: S-0402 claim 14 and S-0209 Terms (it walks back to the miner, to the unloading cell, with a plain move)
        walk_to(w, id, c);
    }
}

/// A slave delivers to its master and goes inside (S-0402 claims 14 and 23).
fn deliver(w: &mut World, master: ObjectId, s: ObjectId) {
    let load = slave_state(w, s).map(|x| x.load).unwrap_or_default();
    let mut paid = false;
    for (tib, &a) in load.iter().enumerate() {
        if a > 0 {
            // spec: S-0402 claim 14 (paid as in claim 12, to the miner's current owner, for each tiberium in turn, lowest index first; claim 23)
            harvest::pay(w, master, tib, a);
            paid = true;
        }
    }
    if paid {
        // spec: S-0402 claim 14 and S-0616 claim 1 (one smoke event on the master per delivery, after all of its payments, only when at least one paid a positive amount; an undeployed master is a vehicle and makes nothing)
        crate::emitters::refinery_smoke(w, master);
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
                // spec: S-0402 claim 14 (it starts the slave reload field's delay); claim 21 (it cannot finish reload in this update, even with delay 0)
                m.inside = true;
                m.ready = false;
                m.since = t;
            }
        }
    });
}

/// The group update's decision for a vehicle master (S-0402 claims 22, 25
/// and 28). Returns whether ready slaves are offered places.
fn vehicle_update(w: &mut World, id: ObjectId) -> bool {
    let Some(g) = group(w, id) else { return false };
    let t = w.tick;
    let Some(o) = w.obj(id).cloned() else {
        return false;
    };
    match g.state {
        GroupState::Packing => {
            recall(w, id);
            // spec: S-0402 claims 22, 25 and 27 (a packing group at the first update that sees the vehicle: recalls its slaves and enters field seeking; that update does not also seek a field, and offers no emergence)
            set_group(w, id, |g| g.state = GroupState::FieldSeeking);
            return false;
        }
        GroupState::CheckingField => {
            // spec: S-0402 claim 28 (a group still checking the field becomes at rest when its update sees a vehicle, last-rest this update, no recall)
            set_group(w, id, |g| {
                g.state = GroupState::AtRest;
                g.last_rest = t;
            });
            return false;
        }
        GroupState::FieldSeeking => {
            if o.dest.is_some() {
                // spec: S-0402 claims 22 and 25 (a seeking vehicle already with a destination enters travelling without replacing it)
                set_group(w, id, |g| g.state = GroupState::Travelling);
                return false;
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
        GroupState::Travelling => {
            // spec: S-0402 claims 22 and 28 (a travelling vehicle with no destination attempts deployment at a due group update, not on arrival)
            if o.dest.is_none() {
                try_deploy(w, id, GroupState::Travelling);
            }
        }
        GroupState::Retrying => {
            // spec: S-0402 claim 22 (the next update retries deployment)
            try_deploy(w, id, GroupState::Retrying);
        }
        GroupState::Working => {
            if o.econ.deploy.turning {
                try_deploy(w, id, GroupState::Working);
            } else if o.beh.seen() == Some(Behaviour::Guard) {
                // spec: S-0402 claim 22 (a working vehicle no longer deploying and in guard re-enters travelling)
                set_group(w, id, |g| g.state = GroupState::Travelling);
            }
        }
        // spec: S-0402 claim 28 (an at-rest group stays at rest; waking is claim 14's)
        GroupState::AtRest => {}
    }
    // spec: S-0402 claim 21 (a working group offers each ready hidden slave a place)
    w.obj(id).is_some_and(|o| o.kind == Kind::Vehicle)
        && group(w, id).is_some_and(|g| g.state == GroupState::Working)
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

/// Whether *construction* or *selling* is a building's current behaviour
/// (S-0402 claim 27's at-rest rows).
fn in_construction_or_selling(w: &World, id: ObjectId) -> bool {
    w.obj(id).is_some_and(|o| {
        matches!(
            crate::econ::placement::seen_behaviour(o),
            Some(Behaviour::Construction | Behaviour::Selling)
        )
    })
}

/// A deployed master's short or long scan (S-0402 claim 26): centred on
/// the cell of its reference point; any candidate of land type tiberium,
/// with no entry, occupancy, reach-region, shroud or usable-area test.
fn building_ring(w: &mut World, id: ObjectId, range: i32) -> Option<CellXY> {
    // spec: S-0402 claim 26 (the scans centre on the cell of the master's reference point, not its top-left or unloading cell)
    let centre = w.reference_point(id).cell();
    harvest::ring_search_with(w, centre, range, &mut |w, c| {
        // spec: S-0402 claim 26 (other candidates need only tiberium land)
        w.cells.cell(c).is_some_and(|x| x.land == land::TIBERIUM)
    })
}

/// Whether a building's type takes a field-seeking site offer as its guard
/// spot (S-0402 claim 27a): a factory of infantry or vehicles, a cloning
/// building, a repair depot or a construction yard. A slave group or an
/// undeploys-into type alone does not qualify it.
fn takes_site_offer(w: &World, id: ObjectId) -> bool {
    let Some(o) = w.obj(id) else { return false };
    if o.kind != Kind::Building {
        return false;
    }
    w.rules()
        .buildings
        .get(o.type_index as usize)
        .is_some_and(|b| {
            // spec: S-0402 claim 27a and Import mapping (an infantry or vehicle factory category, S-0003 claim 21; absent: no factory)
            matches!(b.factory, Some(FactoryKind::Infantry | FactoryKind::Vehicle))
            // spec: S-0402 claim 27a and Import mapping (Cloning, UnitRepair, ConstructionYard; each absent: no)
            || b.raw.cloning.unwrap_or(false)
            || b.raw.unit_repair.unwrap_or(false)
            || b.raw.construction_yard.unwrap_or(false)
        })
}

/// The field-seeking building's site offer (S-0402 claim 27a): the guard
/// spot of a qualifying building that is not currently selling becomes the
/// site cell; any other keeps its earlier guard spot, or its absence. There
/// is no separate movement destination. *Move* is requested independently,
/// without an immediate start.
fn offer_site(w: &mut World, id: ObjectId, site: CellXY) {
    let qualifies = takes_site_offer(w, id);
    let Some(o) = w.obj_mut(id) else { return };
    // spec: S-0402 claim 27a (currently in selling: keeps its earlier guard spot; construction state, power, owner and the redeploy setting add no exclusion)
    let selling = o.beh.current == Some(Behaviour::Selling);
    if !selling && qualifies {
        // spec: S-0402 claims 27a and 28 (the site cell replaces the guard spot, the building's one remembered destination)
        o.guard_spot = Some(Target::Cell(site));
    }
    // spec: S-0402 claim 27a and S-0215 claim 7 (move requested without an immediate start, independently of the destination result; current selling refuses it and keeps the earlier queued behaviour)
    o.beh.queue(Some(Behaviour::Move));
}

/// The group update's decision for a building master (S-0402 claims 22
/// and 25 to 27). Returns whether ready slaves are offered places.
fn building_update(w: &mut World, id: ObjectId) -> bool {
    let Some(g) = group(w, id) else { return false };
    match g.state {
        GroupState::AtRest => {
            // spec: S-0402 claim 27 (at rest, building in construction or selling: remains at rest; otherwise enters checking the field; no check or emergence in this update)
            if !in_construction_or_selling(w, id) {
                set_group(w, id, |g| g.state = GroupState::CheckingField);
            }
            false
        }
        GroupState::Working => {
            // spec: S-0402 claims 22 and 27 (working, building outside its construction state: enters checking the field; no check or emergence in this update)
            if !crate::econ::placement::first_start_withheld(w, id) {
                set_group(w, id, |g| g.state = GroupState::CheckingField);
            }
            false
        }
        // spec: S-0402 claim 27 (checking the field: the field and relocation rule, emergence unless packing is requested; no construction or selling exclusion)
        GroupState::CheckingField => !check_field(w, id),
        // spec: S-0402 claim 27 (packing, still a building: waits for conversion; no field check or emergence)
        GroupState::Packing => false,
        GroupState::FieldSeeking => {
            let t = w.tick;
            let range = w.rules().econ.harvest.slave_long_scan;
            // spec: S-0402 claim 27 (field seeking, building: the long scan and site search of claim 22, the scan a deployed master's, claim 26; construction or selling does not suppress this row)
            let field = building_ring(w, id, range);
            match field.and_then(|f| site_search(w, id, f)) {
                Some((_, s)) => {
                    // spec: S-0402 claims 27 and 27a (success offers the shifted site cell of claim 24, requests move without an immediate start and enters travelling)
                    offer_site(w, id, s);
                    set_group(w, id, |g| g.state = GroupState::Travelling);
                }
                None => {
                    // spec: S-0402 claim 27 (failure puts the group at rest and records this tick as last-rest)
                    set_group(w, id, |g| {
                        g.state = GroupState::AtRest;
                        g.last_rest = t;
                    });
                }
            }
            // spec: S-0402 claim 27 (no emergence or field-check transition in this update)
            false
        }
        GroupState::Travelling | GroupState::Retrying => {
            let t = w.tick;
            // spec: S-0402 claims 22 and 27 (travelling or retrying deployment, building: at rest, this tick recorded as last-rest, without field search, deployment attempt or emergence)
            set_group(w, id, |g| {
                g.state = GroupState::AtRest;
                g.last_rest = t;
            });
            false
        }
    }
}

/// The field and relocation rule of a deployed master checking the field
/// (S-0402 claims 22, 24 and 25, F7). Returns whether packing was
/// requested.
fn check_field(w: &mut World, id: ObjectId) -> bool {
    let Some(a) = w.obj(id).map(|o| o.pos.cell()) else {
        return false;
    };
    let (short, long, correction) = {
        let h = &w.rules().econ.harvest;
        (h.slave_short_scan, h.slave_long_scan, h.scan_correction)
    };
    // spec: S-0402 claim 22 (a short-scan field keeps it where it is and permits release of ready slaves)
    if building_ring(w, id, short).is_some() {
        return false;
    }
    // spec: S-0402 claim 22 (with no short-scan field the long scan seeks another field; claim 24's site near it)
    let field = building_ring(w, id, long);
    // spec: S-0402 claim 25 (a missing result is cell (0, 0) for the comparison, not a usable destination)
    let origin = CellXY::new(0, 0);
    // spec: S-0402 claim 25 (with no field, the site search is still made around (0, 0))
    let f = field.unwrap_or(origin);
    let site = site_search(w, id, f).map(|(_, s)| s);
    // spec: S-0402 claim 25 (quirks.slave_relocation_failed_search_origin off: either failure leaves the miner deployed without a packing request)
    if (field.is_none() || site.is_none())
        && !w.rules().quirks.slave_relocation_failed_search_origin
    {
        return false;
    }
    // spec: S-0402 claim 25 (a missing site is (0, 0), without the foundation-size shifts)
    let s = site.unwrap_or(origin);
    let (table, mode) = {
        let r = w.rules();
        (r.quirks.table_root_distance, r.mode)
    };
    // spec: S-0402 F7 and claim 25 (packing-up is requested exactly when c + d(S, F) < d(A, F); equality does not)
    if f7_packs_up(correction, s, a, f, table, mode) && super::deploy::packs_up(w, id) {
        if let Some(ob) = w.obj_mut(id) {
            // spec: S-0402 claims 22, 25 and 28 (the remembered cell of an automatic pack-up is the building's own top-left cell A; it is the building's one remembered destination, its guard spot, and replaces any earlier guard spot)
            ob.guard_spot = Some(Target::Cell(a));
        }
        // spec: S-0402 claim 27 (the group packs)
        set_group(w, id, |g| g.state = GroupState::Packing);
        // spec: S-0402 claim 22 and S-0415 claims 9 and 10 (the automatic packing-up)
        super::sell::start_pack_up(w, id);
        return true;
    }
    false
}

/// The remembered destination of a slave-master building (S-0402 claim
/// 28): its guard spot, which an automatic pack-up sets to its old
/// top-left cell and a later order can replace.
fn remembered_destination(w: &World, id: ObjectId) -> Option<Target> {
    // spec: S-0402 claim 28 (one remembered destination: the automatic old-cell memory and the guard spot are the same, with no automatic-over-manual priority)
    w.obj(id)?.guard_spot
}

/// The first processing of selling or packing of a slave-master building
/// (S-0402 claim 28; S-0408 claim 6): the directed site rule, only for a
/// remembered destination on a resource-land cell other than the
/// building's own top-left cell.
pub fn first_sale_processing(w: &mut World, id: ObjectId) {
    if !is_master(w, id) {
        return;
    }
    let Some(tl) = w.obj(id).map(|o| o.pos.cell()) else {
        return;
    };
    // spec: S-0402 claim 28 (only a remembered destination that is a resource-land cell other than its own top-left cell; otherwise nothing)
    let Some(Target::Cell(d)) = remembered_destination(w, id) else {
        return;
    };
    if d == tl || w.cells.cell(d).is_none_or(|x| x.land != land::TIBERIUM) {
        return;
    }
    // spec: S-0402 claims 24 and 28 (the directed site search near the remembered destination; a failed search changes nothing)
    let Some((_, s)) = site_search(w, id, d) else {
        return;
    };
    if let Some(ob) = w.obj_mut(id) {
        // spec: S-0402 claim 28 and S-0415 claim 10 (a found site replaces the remembered destination the vehicle inherits)
        ob.guard_spot = Some(Target::Cell(s));
    }
    // spec: S-0402 claim 28 (the group enters packing)
    set_group(w, id, |g| g.state = GroupState::Packing);
    // spec: S-0402 claim 28 (it recalls its slaves)
    recall(w, id);
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
    // spec: S-0402 claim 21 (offers in reverse slave-place order)
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
        // spec: S-0402 claim 21 (every successful emergence starts with facing 0, map north, independent of the master's facing)
        crate::objects::place(w, s, Pos::new(x, y, z), EMERGENCE_FACING);
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
        // spec: S-0301 claim 5, Moment of a retarget's let-go (revision 67, Q-2246: at the change of the firer's target)
        crate::emitters::retarget_let_go(w, s);
    }
}

/// The facing of an emerging slave (S-0402 claim 21).
// spec: S-0402 claim 21 (facing 0, map north)
const EMERGENCE_FACING: glasnost_num::geom::Facing = 0;

/// Whether a slave's load is full (S-0402 claims 14 and 26): only a
/// positive capacity fills; a capacity of 0 or less never makes an empty
/// slave full (its fill ratio is 0).
fn slave_full(w: &World, id: ObjectId, load: &[i32]) -> bool {
    let cap = slave_rules(w, id).1;
    let total: i32 = load.iter().sum();
    // spec: S-0402 claim 14 (its capacity is its type's storage field); claim 26 (capacity 0 or less: fill ratio 0, not full)
    cap > 0 && total >= cap
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

/// The acceptability test of a slave's ring search (S-0402 claim 26).
fn slave_accepts(w: &mut World, id: ObjectId, c: CellXY, reference: i32) -> bool {
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
    // spec: S-0209 claim 4 and S-0402 claim 26 (the connection is from the slave's heading-point cell: its own position when it has no destination)
    let Some(from) = w
        .obj(id)
        .map(|o| crate::path::heading_point(w, id).unwrap_or(o.pos).cell())
    else {
        return false;
    };
    regions::ensure(w);
    // spec: S-0402 claim 26 (its own movement-zone connection from its heading-point cell)
    let connected = matches!(
        (regions::reach_in(w, from, z), regions::reach_in(w, c, z)),
        (Reach::Region(x), Reach::Region(y)) if x == y
    );
    // spec: S-0402 claim 26 (its own infantry entry outcome, S-0209 claim 4, must be clear; another slave or unit is not a blanket refusal)
    connected
        && crate::occupancy::infantry_outcome(w, id, c, reference)
            == crate::occupancy::Outcome::Clear
}

/// The delay after a slave's *harvest* processing that queued *guard*
/// (S-0402 claim 26).
// spec: S-0402 claim 26 (a 1-tick next-processing delay)
const GUARD_QUEUED_DELAY: i32 = 1;

/// The processing of *harvest* by a slave (S-0402 claims 14 and 26): one
/// load step at its type's interval *h*; searching, arrival, fullness and
/// delivery belong to the group update. Returns the delay.
pub fn process_slave(w: &mut World, id: ObjectId) -> i32 {
    let (h, cap) = slave_rules(w, id);
    let Some(st) = slave_state(w, id) else {
        return h;
    };
    let Some(o) = w.obj(id).cloned() else {
        return h;
    };
    // spec: S-0402 claim 26 (only a slave actually loading takes load steps: one still walking takes none)
    if o.dest.is_some() || crate::occupancy::in_progress(w, id) {
        return h;
    }
    let here = o.pos.cell();
    let resource = resource::resource_at(w, here);
    // spec: S-0402 claim 26 (a processing on a non-resource cell or while full, and a capacity of 0, or a negative one with quirks.slave_negative_capacity_stalls off: guard queued, a 1-tick delay)
    let negative_stalls = w.rules().quirks.slave_negative_capacity_stalls;
    let guard = resource.is_none()
        || slave_full(w, id, &st.load)
        || cap == 0
        || (cap < 0 && !negative_stalls);
    if guard {
        if let Some(ob) = w.obj_mut(id) {
            ob.beh.queue(Some(Behaviour::Guard));
        }
        return GUARD_QUEUED_DELAY;
    }
    let Some((tib, _)) = resource else { return h };
    let total: i32 = st.load.iter().sum();
    // spec: S-0402 claim 26 and S-0205 (the bale amount min(1, C − load), truncated toward zero; for a negative capacity it is negative, and a non-positive take delivers nothing and changes no stage, S-0401 claim 3)
    let take = 1.min(cap.saturating_sub(total));
    if take <= 0 {
        // spec: S-0402 claim 26 (quirks.slave_negative_capacity_stalls: it remains in harvest with the next processing after h ticks)
        return h;
    }
    // spec: S-0402 claim 14 (it loads one bale, S-0401 claim 3, at each successful processing of its harvest, every h ticks)
    let got = resource::take_bales(w, here, take);
    if got > 0 {
        set_slave(w, id, |x| {
            if let Some(a) = x.load.get_mut(tib) {
                *a += got;
            }
            x.phase = SlavePhase::Loading;
        });
    }
    h
}

/// The *harvest* processing of a slave miner with a slave group (S-0402
/// claim 22), none for other objects.
pub fn process_harvest(w: &mut World, id: ObjectId) -> Option<i32> {
    if !is_master(w, id) || !harvest::is_slave_miner(w, id) {
        return None;
    }
    let t = w.tick;
    let o = w.obj(id)?.clone();
    let centre = match o.dest {
        Some(Target::Cell(c)) => Some(c),
        Some(Target::Object(x)) => w.obj(x).map(|x| x.pos.cell()),
        None => None,
    };
    // spec: S-0402 claim 22 (at each processing of harvest a vehicle with a destination searches for a site near it; without a destination no field or site search)
    match centre.and_then(|c| site_search(w, id, c)) {
        Some((_, s)) => {
            // spec: S-0402 claim 22 (it moves to a found site with move queued and puts the group into travelling)
            move_to(w, id, s);
            set_group(w, id, |g| g.state = GroupState::Travelling);
        }
        None => {
            if let Some(ob) = w.obj_mut(id) {
                // spec: S-0402 claim 22 (a failed site search, or no destination: guard queued, at rest, last-rest the current tick)
                ob.beh.queue(Some(Behaviour::Guard));
            }
            set_group(w, id, |g| {
                g.state = GroupState::AtRest;
                g.last_rest = t;
            });
        }
    }
    // spec: S-0402 claim 22 and S-0215 claim 6 (in every case the standard delay of harvest, with its one ranged draw)
    Some(crate::process::standard_delay(w, id))
}
