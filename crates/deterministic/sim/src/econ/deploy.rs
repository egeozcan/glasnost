//! Deploying a vehicle into a building and packing it up again (S-0415),
//! and the construction behaviour of a building that builds up (S-0108
//! claim 5; request Q-0130).

use glasnost_map::CellXY;
use glasnost_num::F64;
use glasnost_num::geom::Facing;
use serde::{Deserialize, Serialize};

use super::TypeRef;
use super::placement::{Test, WallRule};
use crate::combat::effects::{Effect, Sound};
use crate::rules::{Behaviour, Kind};
use crate::world::{ObjectId, Pos, Target, World};

/// A deploy in progress (S-0415 claims 2 to 6): set back by every deploy
/// order.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeployState {
    /// The first attempt after the deploy order has been made (S-0415
    /// Terms).
    pub attempted: bool,
    /// A turn to the deploy facing has begun and has not ended (S-0415
    /// claim 5).
    pub turning: bool,
    /// The turn was abandoned by a destination (S-0415 claim 5).
    pub abandoned: bool,
}

/// The building type a vehicle type deploys into (S-0415 Import mapping).
pub fn deploys_into(w: &World, kind: Kind, t: u32) -> Option<u32> {
    if kind != Kind::Vehicle {
        return None;
    }
    let r = w.rules();
    r.unit(kind, t)
        .and_then(|u| u.raw.deploys_into.as_deref())
        .and_then(|n| r.index_of(Kind::Building, n))
}

/// The vehicle type a building type undeploys into (S-0415 Import
/// mapping: a name matching no vehicle type makes it no packing building).
pub fn undeploys_into(w: &World, t: u32) -> Option<u32> {
    let r = w.rules();
    r.buildings
        .get(t as usize)
        .and_then(|b| b.raw.undeploys_into.as_deref())
        .and_then(|n| r.index_of(Kind::Vehicle, n))
}

/// The deploy facing *D* of a building type (S-0415 Constants: absent
/// 128).
pub fn deploy_facing(w: &World, t: u32) -> i32 {
    // spec: S-0415 Constants (deploy facing absent-key default 128)
    w.rules()
        .buildings
        .get(t as usize)
        .map_or(128, |b| b.raw.econ.deploy_facing.unwrap_or(128))
}

/// The facing *D* × 256 (S-0415 claims 5 and 10), reduced modulo 65536
/// (S-0415 Edge cases, working hypothesis for values outside 0 … 255).
fn facing_of(d: i32) -> Facing {
    // spec: S-0415 claim 5 and Edge cases (D × 256, wrapping modulo 65536)
    (d.wrapping_mul(256)).rem_euclid(65536) as Facing
}

/// Whether the redeploy setting is on (S-0415 Terms; the setup value of
/// S-0412 claim 1, defaulting to the rules' default).
pub fn redeploys(w: &World) -> bool {
    crate::econ::setup::redeploy(w)
}

/// Mind-controlled (the mind-control specs own it). Seam: nothing is.
fn mind_controlled(w: &World, id: ObjectId) -> bool {
    let _ = (w, id);
    false
}

/// Whether a sell order makes building `id` pack up (S-0213 claim 12,
/// S-0415 claim 9).
pub fn packs_up(w: &World, id: ObjectId) -> bool {
    let Some(o) = w.obj(id) else { return false };
    if o.kind != Kind::Building || undeploys_into(w, o.type_index).is_none() {
        return false;
    }
    let yard = w.rules().buildings[o.type_index as usize]
        .raw
        .construction_yard
        .unwrap_or(false);
    if !yard {
        // spec: S-0213 claim 12 (a building whose type undeploys into a vehicle packs up)
        return true;
    }
    // spec: S-0213 claim 12 (a construction yard: a human-controlled house, the redeploy setting, not mind-controlled, a guard spot)
    o.owner
        .is_some_and(|h| super::placement::player_house(w, h))
        && redeploys(w)
        && !mind_controlled(w, id)
        && o.guard_spot.is_some()
}

/// Deploy readiness of a vehicle that deploys into a building (S-0214
/// Terms). Being disabled, inside a tunnel, carrying a parasite and mind
/// control are other topics' states and never hold yet.
fn deploy_ready(w: &World, id: ObjectId) -> bool {
    w.obj(id).is_some_and(|o| o.health > 0)
}

/// Rounded body facing (S-0415 F1).
fn rounded_facing(f: Facing) -> i32 {
    // spec: S-0415 F1 (q = ⌊(f + 128) / 256⌋ mod 256)
    ((i32::from(f) + 128) / 256).rem_euclid(256)
}

/// The deploy anchor of a vehicle in `c` for a building of size `size`
/// (S-0415 claim 3, F3).
pub fn anchor(c: CellXY, size: (i32, i32)) -> CellXY {
    // spec: S-0415 F3 (anchor (x, y) when W ≤ 2 and H ≤ 2, else (x − 1, y − 1))
    if size.0 <= 2 && size.1 <= 2 {
        c
    } else {
        CellXY::new(c.x - 1, c.y - 1)
    }
}

/// The return cell of a packing building with top-left `c` (S-0415 claim
/// 10, F3).
pub fn return_cell(c: CellXY, size: (i32, i32)) -> CellXY {
    // spec: S-0415 F3 (the top-left cell when W ≤ 2 and H ≤ 2, else that cell + (1, 1))
    if size.0 <= 2 && size.1 <= 2 {
        c
    } else {
        CellXY::new(c.x + 1, c.y + 1)
    }
}

/// Health handed over (S-0415 F2): max(1, trunc(RZ₅₃(r × S))), with r the
/// health ratio of the replaced object (S-0214 F2) and S the new type's
/// strength.
fn handed_health(w: &World, old: ObjectId, strength: i32) -> i32 {
    let mode = w.rules().mode;
    let Some(o) = w.obj(old) else { return 1 };
    let s_old = crate::combat::damage::strength(w, old);
    // spec: S-0214 F2 via S-0415 F2 (r = health ÷ strength, a double)
    let r = if s_old == 0 {
        F64::ZERO
    } else {
        F64::from_i64(i64::from(o.health), mode).div(F64::from_i64(i64::from(s_old), mode), mode)
    };
    // spec: S-0415 F2 (h = max(1, trunc(RZ₅₃(r × S))))
    r.mul(F64::from_i64(i64::from(strength), mode), mode)
        .to_i32_common()
        .max(1)
}

/// What the *unload* of a vehicle does, by the ranks of S-0415 claim 1.
/// `None`: a rule that is not this spec's (transports, deploy fire,
/// harvesting).
pub fn process_unload(w: &mut World, id: ObjectId) -> Option<()> {
    let o = w.obj(id)?;
    let r = w.rules();
    let u = r.unit(o.kind, o.type_index)?;
    // spec: S-0415 claim 1 rank 1 (a passenger capacity above 0; bunkers are other topics')
    if u.passengers > 0 {
        return None;
    }
    // spec: S-0415 claim 1 rank 2 (deploy fire without the simple-deployer flag)
    if u.combat.deploy_fire && !u.simple_deployer {
        return None;
    }
    // spec: S-0415 claim 1 rank 3 (the harvester or weeder flag)
    if u.harvester || u.raw.weeder.unwrap_or(false) {
        return None;
    }
    // spec: S-0415 claim 1 rank 4 (a type that deploys into a building)
    let b = deploys_into(w, o.kind, o.type_index)?;
    attempt(w, id, b);
    Some(())
}

/// One processing of *unload* by a deploying vehicle (S-0415 claims 2 to
/// 6).
fn attempt(w: &mut World, id: ObjectId, b: u32) {
    let stale = w.rules().quirks.stale_suspended_revert; // spec: S-0215 Edge cases (quirks.stale_suspended_revert)
    let Some(o) = w.obj(id).cloned() else { return };
    let st = o.econ.deploy.clone();
    let tick = w.tick;
    if st.abandoned {
        // spec: S-0415 claim 5 (after an abandoned turn: no attempt; the queued behaviour becomes current, else unload stays)
        if let Some(ob) = w.obj_mut(id) {
            ob.beh.start_next(tick, stale);
        }
        return;
    }
    let has_dest = o.dest.is_some();
    if st.turning {
        // spec: S-0415 claim 5 (a destination abandons the turn, with no announcement)
        if has_dest {
            set(w, id, |s| {
                s.turning = false;
                s.abandoned = true;
            });
            return;
        }
        // spec: S-0415 claim 5 (no deploy readiness: no deploy, nothing else changes)
        if !deploy_ready(w, id) {
            return;
        }
    } else {
        // spec: S-0415 claim 2 (a moving vehicle makes no attempt before a turn; a queued behaviour other than unload takes over)
        if crate::occupancy::in_progress(w, id) {
            if o.beh.queued.is_some_and(|q| q != Behaviour::Unload)
                && let Some(ob) = w.obj_mut(id)
            {
                ob.beh.start_next(tick, stale);
            }
            return;
        }
        set(w, id, |s| s.attempted = true);
        // spec: S-0415 claim 2 (without deploy readiness or with a destination the first attempt fails: guard queued)
        if !deploy_ready(w, id) || has_dest {
            queue_guard(w, id);
            return;
        }
    }
    let size = crate::econ::placement::place_type(w, b).size;
    let at = anchor(o.pos.cell(), size);
    let owner = o.owner;
    let Some(house) = owner else { return };
    // spec: S-0415 claim 3 (the cell test, ignoring the vehicle; the wall exceptions only for unowned walls, quirks.deploy_refuses_own_walls)
    let walls = if w.rules().quirks.deploy_refuses_own_walls {
        WallRule::Unowned
    } else {
        WallRule::Owner(house)
    };
    let test = Test {
        house,
        walls,
        anywhere: true,
        ignore: Some(id),
    };
    if !crate::econ::placement::cell_test(w, b, at, &test) {
        refuse(w, id);
        return;
    }
    let d = deploy_facing(w, b);
    // spec: S-0415 claim 5 (the rounded facing differs from D: turn in place towards D × 256 at the turn rate)
    if rounded_facing(o.body.value(tick)) != d {
        if !st.turning {
            let omega = w
                .rules()
                .unit(o.kind, o.type_index)
                .map_or(0, |u| u.turn_rate);
            if let Some(ob) = w.obj_mut(id) {
                ob.body.turn_to(facing_of(d), tick, omega);
            }
            set(w, id, |s| s.turning = true);
        }
        // spec: S-0415 claim 5 (the turn goes on; it is not restarted)
        return;
    }
    if !complete(w, id, b, at) && !st.turning {
        // spec: S-0415 claims 2 and 6 (a building that cannot be put on the map in a first attempt: guard queued, no announcement)
        queue_guard(w, id);
    }
}

fn set(w: &mut World, id: ObjectId, f: impl FnOnce(&mut DeployState)) {
    if let Some(ob) = w.obj_mut(id) {
        f(&mut ob.econ.deploy);
    }
}

fn queue_guard(w: &mut World, id: ObjectId) {
    if let Some(ob) = w.obj_mut(id) {
        // spec: S-0415 claims 2 and 4 (guard becomes the vehicle's queued behaviour)
        ob.beh.queue(Some(Behaviour::Guard));
        ob.econ.deploy.turning = false;
    }
}

/// A refused deploy (S-0415 claim 4).
fn refuse(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id) else { return };
    let gatherer = w
        .rules()
        .unit(o.kind, o.type_index)
        .is_some_and(|u| u.raw.econ.resource_gatherer.unwrap_or(false));
    let (house, t) = (
        o.owner,
        TypeRef {
            kind: o.kind,
            index: o.type_index,
        },
    );
    if !gatherer {
        // spec: S-0415 claim 4 (the refusal announcement for the owner's player, unless the type gathers resources)
        w.effects.push(Effect::Sound {
            sound: Sound::CannotDeploy,
            of: Some(t),
            house,
            pos: None,
        });
    }
    // spec: S-0415 claim 4 (the attempt ends unload: guard queued; the turn ends)
    queue_guard(w, id);
}

/// The completion of a deploy (S-0415 claim 6). Returns whether the
/// building was put on the map.
fn complete(w: &mut World, id: ObjectId, b: u32, at: CellXY) -> bool {
    let Some(o) = w.obj(id).cloned() else {
        return false;
    };
    let Some(house) = o.owner else { return false };
    // spec: S-0415 claim 6 (put on the map only if the anchor passes the cell test of a placement by the house, ignoring the vehicle)
    let test = Test {
        house,
        walls: WallRule::Owner(house),
        anywhere: true,
        ignore: Some(id),
    };
    if !crate::econ::placement::cell_test(w, b, at, &test) {
        return false;
    }
    let strength = w.rules().buildings[b as usize].strength;
    let health = handed_health(w, id, strength);
    let bid = crate::objects::create(w, Kind::Building, b, Some(house));
    if let Some(ob) = w.obj_mut(bid) {
        // spec: S-0415 claim 6 and F2 (health max(1, trunc(r × S)))
        ob.health = health;
        // spec: S-0415 claim 6 (the vehicle's experience; its target becomes the building's)
        ob.combat.experience = o.combat.experience;
        ob.target = o.target;
    }
    // Leave the vehicle's cell before the building enters it.
    crate::objects::remove(w, id);
    // spec: S-0415 claim 6 (the centre of the anchor at the surface height; construction begins; not a place order, S-0415 claim 7)
    // The building's facing is S-0415's and not stated there (S-0406 claim 16, request Q-0220): interim, the deploy facing D × 256.
    let d = deploy_facing(w, b);
    crate::econ::placement::put_on_map_facing(w, bid, at, false, facing_of(d));
    let yard = w.rules().buildings[b as usize]
        .raw
        .construction_yard
        .unwrap_or(false);
    retarget(w, id, bid, yard);
    // spec: S-0415 claim 6 (the vehicle type's deploy sound at its position)
    w.effects.push(Effect::Sound {
        sound: Sound::Deploy,
        of: Some(TypeRef {
            kind: o.kind,
            index: o.type_index,
        }),
        house: None,
        pos: Some(o.pos),
    });
    w.note("S-0415 claims 6 and 10: control groups, selection and trigger tags are the input's and the triggers'; slaves are S-0215's (seams)");
    true
}

/// Every object whose target was `old` gets `new` as its target (S-0415
/// claims 6 and 10); a vehicle thief's becomes nothing when `new` is a
/// construction yard (`yard`).
fn retarget(w: &mut World, old: ObjectId, new: ObjectId, yard: bool) {
    let ids: Vec<ObjectId> = w
        .objects
        .values()
        .filter(|x| x.target == Some(Target::Object(old)))
        .map(|x| x.id)
        .collect();
    for x in ids {
        let thief = w
            .obj(x)
            .and_then(|ob| w.rules().unit(ob.kind, ob.type_index))
            .is_some_and(|u| u.vehicle_thief);
        if let Some(ob) = w.obj_mut(x) {
            // spec: S-0415 claim 6 (a vehicle thief's target becomes nothing when the building is a construction yard)
            ob.target = if thief && yard {
                None
            } else {
                Some(Target::Object(new))
            };
        }
    }
}

/// The end of a pack-up (S-0415 claims 10 and 11): the vehicle appears at
/// the return cell and the building is removed, or the house receives the
/// sale refund when the vehicle cannot be put on the map.
pub fn pack_up(w: &mut World, id: ObjectId) {
    let stale = w.rules().quirks.stale_suspended_revert; // spec: S-0215 Edge cases (quirks.stale_suspended_revert)
    let Some(o) = w.obj(id).cloned() else { return };
    let Some(house) = o.owner else { return };
    let Some(v) = undeploys_into(w, o.type_index) else {
        return;
    };
    let c = return_cell(o.pos.cell(), o.foundation);
    let d = deploy_facing(w, o.type_index);
    let strength = w.rules().unit(Kind::Vehicle, v).map_or(1, |u| u.strength);
    let health = handed_health(w, id, strength);
    let vid = crate::objects::create(w, Kind::Vehicle, v, Some(house));
    if let Some(ob) = w.obj_mut(vid) {
        // spec: S-0415 claim 10 and F2 (health max(1, trunc(r × S)); the building's experience)
        ob.health = health;
        ob.combat.experience = o.combat.experience;
    }
    crate::objects::remove(w, id);
    if can_stand(w, c) {
        let z = w.surface_at(256 * c.x + 128, 256 * c.y + 128); // spec: S-0204 claim 3 (the cell centre)
        let tick = w.tick;
        if let Some(ob) = w.obj_mut(vid) {
            if let Some(spot) = o.guard_spot {
                // spec: S-0415 claim 10 (a guard spot: the vehicle's destination, with move)
                ob.dest = Some(spot);
                ob.beh.force(Behaviour::Move, tick, stale);
            } else {
                ob.beh.force(Behaviour::Guard, tick, stale);
            }
        }
        // spec: S-0415 claim 10 (the centre of the return cell, facing D × 256)
        crate::objects::place(w, vid, Pos::centre(c, z), facing_of(d));
        retarget(w, id, vid, false);
    } else {
        // spec: S-0415 claim 11 (the vehicle cannot be put on the map: the sale refund instead)
        let amount = crate::econ::sell::refund(
            w,
            house,
            TypeRef {
                kind: Kind::Building,
                index: o.type_index,
            },
        );
        if let Some(h) = w.houses.get_mut(usize::from(house.0)) {
            h.credits = h.credits.wrapping_add(amount);
        }
        // spec: S-0415 claim 11 and Edge cases (quirks.failed_packup_keeps_vehicle: the vehicle goes on existing off the map)
        if !w.rules().quirks.failed_packup_keeps_vehicle {
            w.objects.remove(&vid);
        }
    }
}

/// Whether a vehicle can be put on the map in `c` at the end of a pack-up
/// (S-0415 claim 11: for example, not where a building now stands). What
/// else refuses is not stated (request Q-0132); our engine asks for a
/// usable cell with no building and no vehicle claim.
fn can_stand(w: &World, c: CellXY) -> bool {
    w.cells.usable(c)
        && w.cells
            .occ(c, crate::world::Layer::Ground)
            .is_some_and(|r| r.buildings == 0 && r.vehicles == 0)
}

/// The construction behaviour's processing delay (S-0406 claim 14).
// spec: S-0406 claim 14 and Constants (construction processing delay: 1 tick, no timing draw, whatever the behaviour's rate)
pub const CONSTRUCTION_DELAY: i32 = 1;

/// The construction behaviour of a building (S-0406 claims 14 and 15,
/// F6): the first processing restarts the build-up at stage 0 and is the
/// tick *C*; a later processing that finds the build-up reported finished
/// hands over: the building's opening effects, then *guard* queued, current
/// at the building's second per-tick start of the same tick. Returns the
/// delay.
pub fn process_construction(w: &mut World, id: ObjectId) -> i32 {
    let Some(o) = w.obj(id) else {
        return CONSTRUCTION_DELAY;
    };
    let tick = w.tick;
    let Some(c) = o.econ.construction.first else {
        // spec: S-0406 claim 14 (the first processing starts the construction range again at stage 0: C is this tick)
        if let Some(ob) = w.obj_mut(id) {
            ob.econ.construction.first = Some(tick);
        }
        return CONSTRUCTION_DELAY;
    };
    // spec: S-0406 claim 15 and F6 (H = C + (n − 1) × d_b; a zero-delay range reports at the next processing, H = C + 1); a one-stage positive-delay range is open (S-0406 open question 6, request Q-0130): C + 1 as well
    if tick.wrapping_sub(c) < crate::econ::sell::buildup_ticks(w, o.type_index).max(1) {
        return CONSTRUCTION_DELAY;
    }
    if let Some(ob) = w.obj_mut(id) {
        ob.econ.construction.building_up = false;
        ob.econ.construction.first = None;
        // spec: S-0406 claim 15 (guard queued, current at the second per-tick start of this tick, S-0215 claim 11; the target is kept)
        ob.beh.queue(Some(Behaviour::Guard));
        ob.beh.ready_mark = true;
    }
    w.note("S-0406 claim 15: the idle body animation state is requested at the handover (a seam: no animation state in the simulation)");
    // spec: S-0402 claim 18 (the grand opening, at the end of the build-up in the building's own turn: its free unit)
    crate::econ::harvest::grand_opening(w, id);
    CONSTRUCTION_DELAY
}
