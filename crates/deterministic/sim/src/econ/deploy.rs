//! Deploying a vehicle into a building and packing it up again (S-0415),
//! and the construction behaviour of a building that builds up (S-0108
//! claim 5, S-0406 claims 14, 15, 17 and 18).

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
    /// The form of an in-place deploying vehicle (S-0517 Terms).
    #[serde(default)]
    pub form: super::inplace::Form,
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
/// for every *D* (claim 5; for the packed-up vehicle of claim 10 a working
/// hypothesis, S-0415 Open question 4).
fn facing_of(d: i32) -> Facing {
    // spec: S-0415 claim 5 and Edge cases (the turn target (D × 256) mod 65536 for every integer D, exact)
    (d.wrapping_mul(256)).rem_euclid(65536) as Facing
}

/// The facing a deployed building is inserted with (S-0415 claim 6).
// spec: S-0415 Constants (building insertion facing 0, map north)
const INSERTION_FACING: Facing = 0;

/// Whether the redeploy setting is on (S-0415 Terms; the setup value of
/// S-0412 claim 1, defaulting to the rules' default).
pub fn redeploys(w: &World) -> bool {
    crate::econ::setup::redeploy(w)
}

/// Mind-controlled (S-0601 Terms).
fn mind_controlled(w: &World, id: ObjectId) -> bool {
    crate::mind::mind_controlled(w, id)
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
/// Terms). Being disabled and inside a tunnel are other topics' states and
/// never hold yet. The key command's deploy rule reads it too (S-0213
/// table T2).
pub fn deploy_ready(w: &World, id: ObjectId) -> bool {
    let Some(o) = w.obj(id) else { return false };
    let into_yard = deploys_into(w, o.kind, o.type_index).is_some_and(|b| {
        w.rules().buildings[b as usize]
            .raw
            .construction_yard
            .unwrap_or(false)
    });
    // spec: S-0214 Terms (revision 39: not when it deploys into a construction yard while it is controlled by a controlling object) and S-0601 claims 13 and 15 (revision 9: only control by a controller counts; a permanently controlled vehicle is not refused)
    if into_yard && crate::mind::controlled_by(w, id).is_some() {
        return false;
    }
    // spec: S-0214 Terms and S-0415 claim 2 (not when it deploys into a building while a parasite is on it, S-0614)
    o.health > 0 && !crate::parasite::carries_parasite(w, id)
}

/// Whether building type `b` passes the cell test at `at` for the owner of
/// vehicle `id`, the vehicle itself not counting as an obstacle (S-0415
/// claim 3). The deploy attempt and the click's table VD row VD2 ask it.
pub fn fits(w: &World, id: ObjectId, b: u32, at: CellXY) -> bool {
    let Some(house) = w.obj(id).and_then(|o| o.owner) else {
        return false;
    };
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
    super::placement::cell_test(w, b, at, &test)
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

/// Which rule a vehicle's *unload* processing followed (S-0415 claim 1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnloadRule {
    /// The deploy-fire rule (rank 2), with its delay already chosen and
    /// its draw taken.
    DeployFire(i32),
    /// A deploy attempt (rank 4): the standard delay of S-0215 claim 6
    /// follows.
    Deploy,
    /// A step of the in-place cycle (S-0517 claims 3 to 8, rank 5), with
    /// the delay it chose.
    InPlace(i32),
}

/// The deploy-fire base delay (S-0517 F3).
const DEPLOY_FIRE_DELAY: i32 = 14; // spec: S-0517 F3 and Constants (deploy-fire base delay 14 ticks); S-0415 claim 1

/// What the *unload* of a vehicle does, by the ranks of S-0415 claim 1.
/// `None`: a rule that is not this spec's (transports, harvesting).
pub fn process_unload(w: &mut World, id: ObjectId) -> Option<UnloadRule> {
    let o = w.obj(id)?;
    let r = w.rules();
    let u = r.unit(o.kind, o.type_index)?;
    // spec: S-0415 claim 1 rank 1 (a passenger capacity above 0; bunkers are other topics')
    if u.passengers > 0 {
        return None;
    }
    // spec: S-0415 claim 1 rank 2 (deploy fire without the simple-deployer flag)
    if u.combat.deploy_fire && !u.simple_deployer {
        let half = r.quirks.ranged_draw_half_span;
        // spec: S-0517 claim 2 (it uses its own cell as the target and submits the weapon in slot 0, whatever the deploy weapon index, to the firing check with its range test; on can fire it fires that weapon at its own cell)
        let own = Target::Cell(o.pos.cell());
        if crate::combat::check::check(w, id, Some(own), 0, true)
            == crate::combat::check::Answer::CanFire
        {
            crate::combat::fire::shot(w, id, 0, own);
        }
        // spec: S-0415 claim 1 and S-0517 claim 2, F3 (delay 14 + u, u one ranged draw over 0 … 2 from the scenario stream, taken even when the shot is refused; an exact integer sum; unload stays current)
        let u = w.rng.ranged(0, 2, half);
        return Some(UnloadRule::DeployFire(DEPLOY_FIRE_DELAY + u));
    }
    // spec: S-0415 claim 1 rank 3 (the harvester or weeder flag)
    if u.harvester || u.raw.weeder.unwrap_or(false) {
        return None;
    }
    // spec: S-0415 claim 1 rank 4 (a type that deploys into a building)
    if let Some(b) = deploys_into(w, o.kind, o.type_index) {
        let _ = attempt(w, id, b);
        return Some(UnloadRule::Deploy);
    }
    // spec: S-0517 claim 1 (the in-place cycle comes after every rule of S-0415 claim 1: a vehicle of a type for which any rule above applies never deploys in place)
    if u.simple_deployer && o.kind == Kind::Vehicle {
        return Some(UnloadRule::InPlace(crate::econ::inplace::process(w, id)));
    }
    None
}

/// What one deploy attempt came to (S-0415 claims 2 to 6), for the slave
/// group's deploy attempts (S-0402 claim 22).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Attempt {
    /// The building was put on the map.
    Deployed(ObjectId),
    /// A turn to the deploy facing goes on (the vehicle is deploying).
    Turning,
    /// The attempt failed: *guard* is queued.
    Failed,
    /// No attempt was made (moving, or after an abandoned turn).
    NoAttempt,
}

/// One processing of *unload* by a deploying vehicle (S-0415 claims 2 to
/// 6).
pub(crate) fn attempt(w: &mut World, id: ObjectId, b: u32) -> Attempt {
    let stale = w.rules().quirks.stale_suspended_revert; // spec: S-0215 Edge cases (quirks.stale_suspended_revert)
    let Some(o) = w.obj(id).cloned() else {
        return Attempt::NoAttempt;
    };
    let st = o.econ.deploy.clone();
    let tick = w.tick;
    if st.abandoned {
        // spec: S-0415 claim 5 (after an abandoned turn: no attempt; the queued behaviour becomes current, else unload stays)
        if let Some(ob) = w.obj_mut(id) {
            ob.beh.start_next(tick, stale);
        }
        return Attempt::NoAttempt;
    }
    let has_dest = o.dest.is_some();
    if st.turning {
        // spec: S-0415 claim 5 (a destination abandons the turn, with no announcement)
        if has_dest {
            set(w, id, |s| {
                s.turning = false;
                s.abandoned = true;
            });
            return Attempt::NoAttempt;
        }
        // spec: S-0415 claim 5 (no deploy readiness: no deploy, nothing else changes)
        if !deploy_ready(w, id) {
            return Attempt::Turning;
        }
    } else {
        // spec: S-0415 claim 2 (a moving vehicle makes no attempt before a turn; a queued behaviour other than unload takes over); S-0503 claim 19 (a hover is moving in the sense of S-0503 Terms, a destination D or a heading point, whatever its throttle; once both are absent it is not)
        let moving = if crate::hover::is_hover(w, id) {
            crate::hover::moving(w, id)
        } else {
            crate::occupancy::in_progress(w, id)
        };
        if moving {
            if o.beh.queued.is_some_and(|q| q != Behaviour::Unload)
                && let Some(ob) = w.obj_mut(id)
            {
                ob.beh.start_next(tick, stale);
            }
            return Attempt::NoAttempt;
        }
        set(w, id, |s| s.attempted = true);
        // spec: S-0415 claim 2 (without deploy readiness or with a destination the first attempt fails: guard queued)
        if !deploy_ready(w, id) || has_dest {
            queue_guard(w, id);
            return Attempt::Failed;
        }
    }
    let size = crate::econ::placement::place_type(w, b).size;
    let at = anchor(o.pos.cell(), size);
    if o.owner.is_none() {
        return Attempt::NoAttempt;
    }
    if !fits(w, id, b, at) {
        refuse(w, id);
        return Attempt::Failed;
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
        return Attempt::Turning;
    }
    match complete(w, id, b, at) {
        Some(bid) => Attempt::Deployed(bid),
        None if !st.turning => {
            // spec: S-0415 claims 2 and 6 (a building that cannot be put on the map in a first attempt: guard queued, no announcement)
            queue_guard(w, id);
            Attempt::Failed
        }
        None => Attempt::Turning,
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

/// The completion of a deploy (S-0415 claim 6). Returns the building put
/// on the map, if any.
fn complete(w: &mut World, id: ObjectId, b: u32, at: CellXY) -> Option<ObjectId> {
    let o = w.obj(id).cloned()?;
    let house = o.owner?;
    // spec: S-0415 claim 6 (put on the map only if the anchor passes the cell test of a placement by the house, ignoring the vehicle)
    let test = Test {
        house,
        walls: WallRule::Owner(house),
        anywhere: true,
        ignore: Some(id),
    };
    if !crate::econ::placement::cell_test(w, b, at, &test) {
        return None;
    }
    let strength = w.rules().buildings[b as usize].strength;
    let health = handed_health(w, id, strength);
    // spec: S-0402 claim 19 and S-0415 claim 6 (conversion keeps the working slave group: the building creates no group of its own, it takes the vehicle's slaves)
    let bid = crate::objects::create_converted(w, Kind::Building, b, Some(house));
    super::slaves::transfer(w, id, bid);
    if let Some(ob) = w.obj_mut(bid) {
        // spec: S-0415 claim 6 and F2 (health max(1, trunc(r × S)))
        ob.health = health;
        // spec: S-0415 claim 6 (the vehicle's experience; its target becomes the building's)
        ob.combat.experience = o.combat.experience;
        ob.target = o.target;
    }
    // spec: S-0601 claim 15 (a controlled vehicle that deploys: the new building is the controlling house's; with the switch off it inherits the link)
    crate::mind::converted(w, id, bid);
    // Leave the vehicle's cell before the building enters it.
    crate::objects::remove(w, id);
    // spec: S-0415 claim 6 and Constants (the centre of the anchor at the surface height, inserted facing 0, not the vehicle's facing or D × 256; construction begins; not a place order, S-0415 claim 7)
    crate::econ::placement::put_on_map_facing(w, bid, at, false, INSERTION_FACING);
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
    // spec: S-0415 claim 6 (the building takes over the vehicle's control group and selection: the client's, told by a presentation effect once the building is on the map)
    w.effects.push(Effect::Converted { old: id, new: bid });
    w.note("S-0415 claims 6 and 10: control groups, selection and trigger tags are the input's and the triggers'");
    Some(bid)
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
            crate::spawn::set_own_target(
                ob,
                if thief && yard {
                    None
                } else {
                    Some(Target::Object(new))
                },
            );
        }
        // spec: S-0301 claim 5, Moment of a retarget's let-go (revision 67, Q-2246: at the change of the firer's target)
        crate::emitters::retarget_let_go(w, x);
    }
}

/// The end of a pack-up (S-0415 claims 10 to 12): the vehicle appears at
/// the return cell and the building is removed, or the house receives the
/// sale refund when the return cell refuses the vehicle.
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
    let vid = crate::objects::create_converted(w, Kind::Vehicle, v, Some(house));
    // spec: S-0402 claim 19a, S-0415 claims 10 and 11 and S-0202 claim 10 (the return vehicle creates its own initial hidden slaves, each with its creation draw, before the placement attempt and while the building's old group still exists, on success and refusal alike)
    super::slaves::created(w, vid);
    if let Some(ob) = w.obj_mut(vid) {
        // spec: S-0415 claim 10 and F2 (health max(1, trunc(r × S)); the building's experience)
        ob.health = health;
    }
    // spec: S-0415 claims 10 and 11 (the take-over of the slaves requires successful placement: the group is held apart until the return-cell test)
    let group = super::slaves::detach(w, id);
    crate::objects::remove(w, id);
    // spec: S-0415 claim 12 (the test is made before the vehicle inherits the building's experience or the guard spot)
    let accepted = can_stand(w, vid, c);
    if let Some(ob) = w.obj_mut(vid) {
        ob.combat.experience = o.combat.experience;
    }
    if accepted {
        // spec: S-0415 claim 10 and S-0402 claims 25 and 28 (the building's one remembered destination, its guard spot, which an automatic pack-up set to the old top-left cell, is the vehicle's immediate destination)
        let dest = o.guard_spot;
        if let Some(temporary) = super::slaves::detach(w, vid) {
            // spec: S-0402 claim 19a and S-0415 claim 10 (successful placement destroys and removes the vehicle's newly created hidden slaves before the old group's transfer, with S-0312 claim 31's notifications and losses; their draws are not undone)
            super::slaves::end_removed(w, temporary);
        }
        if let Some(g) = group {
            // spec: S-0415 claim 10 and S-0402 claim 19a (the vehicle takes over the building's slaves; the group state is unchanged by the conversion, S-0402 claims 22 and 28)
            super::slaves::attach(w, vid, g);
        }
        let z = w.surface_at(256 * c.x + 128, 256 * c.y + 128); // spec: S-0204 claim 3 (the cell centre)
        let tick = w.tick;
        if let Some(ob) = w.obj_mut(vid) {
            if dest.is_some() {
                // spec: S-0415 claim 10 (the destination with move queued)
                ob.beh.queue(Some(Behaviour::Move));
            } else {
                ob.beh.force(Behaviour::Guard, tick, stale);
            }
        }
        // spec: S-0415 claim 10 (the centre of the return cell, facing D × 256)
        crate::objects::place(w, vid, Pos::centre(c, z), facing_of(d));
        if let Some(spot) = dest {
            // spec: S-0415 claim 10 and S-0207 claim 20 (a) (the destination given once the vehicle is on the map, in the conversion tick: a teleporter's mark moves at once)
            super::assign_destination(w, vid, Some(spot));
            if let Some(ob) = w.obj_mut(vid) {
                // spec: S-0415 claim 10 (move queued in the conversion tick: queued again once the destination is given, so that a placement fall-back that queued guard without it, S-0215 claim 15, does not replace it; no change when move is already current)
                ob.beh.queue(Some(Behaviour::Move));
            }
        }
        retarget(w, id, vid, false);
        // spec: S-0415 claim 10 (the vehicle takes over the building's control group and selection, on successful placement only: the client's, told by a presentation effect)
        w.effects.push(Effect::Converted { old: id, new: vid });
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
        if let Some(g) = group {
            // spec: S-0415 claim 11 and S-0312 claim 31 (refusal does not transfer the old group: the building's removal ends it without a killing hit)
            super::slaves::end_removed(w, g);
        }
        // spec: S-0415 claim 11 and Edge cases (quirks.failed_packup_keeps_vehicle: the vehicle goes on existing off the map)
        // spec: S-0402 claim 19a (with the quirk on the retained vehicle keeps its new hidden group, created before placement; removing the building ended only the old group)
        if !w.rules().quirks.failed_packup_keeps_vehicle {
            if let Some(temporary) = super::slaves::detach(w, vid) {
                // spec: S-0402 claims 19a and 27, S-0415 claim 11 and Edge cases, S-0202 claim 10 and S-0312 claim 31 (quirk off: the vehicle's new group is removed too, its creation draws already taken)
                super::slaves::end_removed(w, temporary);
            }
            // spec: S-0415 claim 11 and S-0402 claim 19a (quirk off: the refused vehicle does not remain)
            w.objects.remove(&vid);
        }
    }
}

/// Whether the new vehicle `v` can be put on the map at the return cell
/// `c` at the end of a pack-up (S-0415 claim 12). The packing building has
/// been removed already, so it does not occupy the map for this test.
fn can_stand(w: &World, v: ObjectId, c: CellXY) -> bool {
    if !w.cells.usable(c) {
        return false;
    }
    // spec: S-0415 claim 12 (ordinary ground-layer placement: no source height, no bridge-deck request)
    let reference = w.cells.cell(c).map_or(0, |x| i32::from(x.level));
    // spec: S-0415 claim 12 (accepted exactly when the entry outcome of S-0209 claim 5 is clear, with no movement-step substitutions; any other outcome refuses)
    crate::occupancy::vehicle_outcome(w, v, c, reference) == crate::occupancy::Outcome::Clear
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
    let opened = o.econ.construction.opened;
    let Some(c) = o.econ.construction.first else {
        // spec: S-0406 claim 14 (the first processing starts the construction range again at stage 0: C is this tick)
        if let Some(ob) = w.obj_mut(id) {
            ob.econ.construction.first = Some(tick);
        }
        // spec: S-0309 claim 31 (a building opened at an owner change while construction was current and not yet processed: the restart at the first stage is an immediate change that cancels the change's idle-body request, S-0108 claim 6)
        crate::econ::body::leave_idle(w, id);
        return CONSTRUCTION_DELAY;
    };
    if !opened {
        let never = w.rules().quirks.single_stage_buildup_never_completes;
        // spec: S-0406 claim 15 and F6 (H = C + (n − 1) × d_b; a zero-delay range reports at the next processing, H = C + 1); claim 18 (a positive-delay range of one stage or fewer never hands over)
        let Some(after) =
            crate::econ::sell::buildup_range(w, o.type_index).report_after(never)
        else {
            return CONSTRUCTION_DELAY;
        };
        if tick.wrapping_sub(c) < after {
            return CONSTRUCTION_DELAY;
        }
    }
    // spec: S-0309 claim 31 (with construction current and processed at least once at an owner change, its next processing finds the ready mark that the change's grand opening set and completes the construction at once)
    let idle = w.obj(id).is_some_and(|o| o.econ.idle_body.is_some());
    if let Some(ob) = w.obj_mut(id) {
        ob.econ.construction = crate::econ::placement::Construction::default();
        // spec: S-0406 claim 15 (guard queued, current at the second per-tick start of this tick, S-0215 claim 11; the target is kept)
        ob.beh.queue(Some(Behaviour::Guard));
        ob.beh.ready_mark = true;
    }
    // spec: S-0108 claim 6 (a deferred request for the idle state leaves a running idle range unchanged: the idle body requested at an owner change, S-0309 claim 31)
    if !idle {
        // spec: S-0406 claim 15 and S-0108 claim 6 (the idle body is requested at the handover: a deferred change the building's own behaviour requests, shown on this tick, delay N(d, s))
        crate::econ::body::enter_idle(w, id, false);
    }
    if opened {
        // spec: S-0309 claim 31 (without a second grand opening: a building has one at most, so no free unit follows, S-0402 claim 18)
        return CONSTRUCTION_DELAY;
    }
    // spec: S-0402 claim 18 (the grand opening, at the end of the build-up in the building's own turn: its free unit)
    crate::econ::harvest::grand_opening(w, id);
    CONSTRUCTION_DELAY
}
