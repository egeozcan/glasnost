//! The teleport (S-0513 claims 33 to 41): the source pick, the warp group
//! and each member's warp (State and ordering, T2).
//!
//! A member's warp runs in its own updates (S-0513 State and ordering: "a
//! warp member's later draws come in its own updates"): [`turn`] is called
//! at the start of the member's living turn. A member still leaving takes no
//! other part of its turn (S-0513 claim 48, confirmed); for the other stages
//! before its arrival hold, whether it takes the rest of its turn is
//! unmeasured in the original; our engine's rule (S-0513 claim 51, engine
//! design, with S-0203 claim 4 revision 32) is that the warp update is the
//! whole member turn there, apart from the intrinsic exceptions and the
//! elapsed reload clock (`combat::fire::reload_elapsed`).

use glasnost_map::CellXY;
use glasnost_map::terrain::land;
use glasnost_rules::sim::{LocomotionClass, MovementZone};
use serde::{Deserialize, Serialize};

use super::SwEvent;
use crate::combat::effects::Sound;
use crate::rules::{Kind, Locomotor};
use crate::world::{HouseId, Layer, ObjectId, Pos, World};

/// The blank stand-in cell that the nearby-cell search's callers use when
/// it finds nothing (S-0212 claim 25; S-0507 claims 8 and 10).
// spec: S-0513 claim 50 (both exceptional cases use fallback cell coordinates (0, 0), the stand-in cell of S-0212 claim 25 and S-0009 claim 44)
const STAND_IN: CellXY = CellXY::new(0, 0);

/// The leaving stage's fixed wait (Constants).
// spec: S-0513 Constants (warp leaving wait 60 elapsed ticks; claim 39: an engine constant, not the rules' teleport delay)
const LEAVING_WAIT: i32 = 60;

/// A warp member's stage (T2).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Stage {
    /// Joined the group; its first update comes next.
    Pending,
    /// The leaving stage, waiting since the tick held.
    Leaving(i32),
    /// The leaving stage, the wait expired; departs on the next update.
    Expired,
    /// Off the map, its landing check failed (claim 37).
    Blocked,
    /// Off the map, its landing check passed: placed on the next update.
    Away,
    /// Placed at its landing point: arrival effects on the next update.
    Placed,
    /// The arrival hold since the tick of the arrival effects (claim 40).
    Hold(i32),
}

/// A warp member's state (T2).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Member {
    /// The stage.
    pub stage: Stage,
    /// The launching house (claims 35 and 41).
    pub house: HouseId,
    /// The landing point (claim 36), moved by a redirection (claim 37).
    pub landing: Pos,
    /// This warp's landing was redirected (claim 40, quirk off).
    pub redirected: bool,
    /// The first fallback of this member's warp has been reported (claim 50).
    #[serde(default)]
    pub fell_back: bool,
    /// The first placement failure of this member's warp has been reported
    /// (claim 50).
    #[serde(default)]
    pub refused: bool,
    /// The landing cell could be entered by the member when its last landing
    /// check passed (claim 50, Q-0871: a later change that makes it
    /// unenterable refuses the placement; terrain that was already fatal at
    /// the check is not a new refusal).
    #[serde(default)]
    pub entry_ok: bool,
}

impl Member {
    /// In the leaving stage (Terms): from its first update after launch
    /// until departure, the expiry update included (claim 39).
    pub fn leaving(&self) -> bool {
        matches!(self.stage, Stage::Leaving(_) | Stage::Expired)
    }

    /// Warp pending or leaving (claims 22 and 34).
    pub fn pending_or_leaving(&self) -> bool {
        self.stage == Stage::Pending || self.leaving()
    }

    /// Phasing in (Terms): from departure until the arrival hold ends.
    pub fn phasing(&self) -> bool {
        matches!(
            self.stage,
            Stage::Blocked | Stage::Away | Stage::Placed | Stage::Hold(_)
        )
    }

    /// Departed and not yet appeared (claim 39): off the map, at the
    /// position it had at departure.
    pub fn departed(&self) -> bool {
        matches!(self.stage, Stage::Blocked | Stage::Away)
    }
}

fn member(w: &World, id: ObjectId) -> Option<Member> {
    w.obj(id).and_then(|o| o.sw.warp.clone())
}

fn set_member(w: &mut World, id: ObjectId, m: Option<Member>) {
    if let Some(o) = w.obj_mut(id) {
        o.sw.warp = m;
    }
}

/// Whether an object is a warp member: from joining the group at launch
/// until its arrival hold ends (`turn`).
pub fn is_member(w: &World, id: ObjectId) -> bool {
    member(w, id).is_some()
}

/// Whether an object is phasing in by a superweapon warp (Terms).
pub fn phasing(w: &World, id: ObjectId) -> bool {
    member(w, id).is_some_and(|m| m.phasing())
}

/// Whether an object is a warp member between its departure and its
/// appearance at its landing point (claim 39). It is off the map then, in
/// no cell, and keeps the position it had at departure; a projectile aimed
/// at it keeps it as its target (S-0306 claim 17).
pub fn departed(w: &World, id: ObjectId) -> bool {
    member(w, id).is_some_and(|m| m.departed())
}

/// Whether the warp row of S-0102's layer table applies, so that a
/// superweapon warp keeps an object in the ground render layer whatever
/// its height (S-0306 claim 17): a member whose warp has been launched and
/// whose arrival hold has not ended (the member state exists from joining
/// the group at launch until the hold ends, `turn`). Any former member
/// that did not move from one bridge deck to another stands at no height
/// above its surface after the hold until its own movement raises it, so
/// its own rows (`temporal::ground_layer`) give the ground layer until then
/// and another layer from that movement's moment. For a deck to deck
/// member the height test is kept: when it leaves the layer is not
/// established (S-0306 Open questions). An aircraft member appears on the
/// ground at its landing point whatever its visiting mark (`air::placed`,
/// S-0306 r75 claim 17), so the ground list that an aircraft whose type
/// cannot land stays in afterwards is no warp row but that ground
/// placement's, kept by S-0102 claim 4's rule that no take-off re-files
/// such a type (`air::kept_filed`). Only S-0309 claim 7's ground-layer pass
/// and the drawing read it. `quirks.layer_change_on_events_only` is a
/// drawing setting (S-0102 r43): with it off the drawing's sweep moves such
/// a member once the warp row has ended, and the simulation's ground-layer
/// pass reads the on position's list in both positions.
pub fn in_warp_row(w: &World, id: ObjectId) -> bool {
    // spec: S-0306 r72 claim 17 (a warp member is in the ground render layer from its warp's launch until its own movement applies again after its arrival hold, whatever its height; once it flies after its hold, an aircraft whose type cannot land stays in the ground list, S-0102 claim 4, while the erasure test, the seizing test, the airborne scan and claim 20 (c) read the layer from state and height) and S-0102 claim 3, claim 4 and Edge cases (the warp row applies from the warp's launch until the member's own movement applies again after its arrival hold; once it has ended a member is treated as any object)
    member(w, id).is_some()
}

/// Whether a departed member's landing check has passed for a landing cell
/// on a deck and awaits its placement, so that S-0306 claim 17 counts it as
/// standing on a deck. A placement refused after the pass (claim 50) takes
/// it back to the landing retry and ends this until a later check passes
/// for a deck cell; each passed check sets the count from its own landing
/// cell (S-0306 claim 17, engine design, Q-2232).
pub fn passed_for_deck(w: &World, id: ObjectId) -> bool {
    // spec: S-0306 claim 17 (standing on a deck only while a landing check that passed for a landing cell on a deck awaits its placement; after a refused placement, by its height again until a later check passes for a deck cell; engine design with no switch, Q-2232) and S-0513 T2 (check passed: placed on the next update)
    member(w, id).is_some_and(|m| {
        m.stage == Stage::Away && crate::teleport::landing_layer(w, m.landing.cell()) == Layer::Deck
    })
}

/// Seam: the types that cannot be teleported. An object being erased by a
/// chrono weapon (S-0514) is already left in place through
/// `econ::power::being_erased`; no spec names a type that cannot be
/// teleported, so none is kept out here.
pub fn chrono_seam(w: &World, id: ObjectId) -> bool {
    let _ = (w, id);
    false
}

/// A source pick (claim 33). The record side (the source, its retention
/// and the charge) is S-0512 claim 10's, done by the fire order.
pub fn pick_source(w: &mut World, h: HouseId, i: u32, cell: CellXY) {
    let mut p = super::cell_point(w, cell, true);
    p.z += super::protect::INVOKE_LIFT;
    // spec: S-0513 claim 33 (the marker is created whichever house is local, taking its ordinary animation creation draws on the scenario stream, S-0202 claim 10 and S-0109 claim 13; hiding it and silencing it for a viewer other than the launching player does not undo them)
    for old in crate::anims::with_role(w, crate::anims::Role::SourceMarker(h)) {
        // spec: S-0513 claim 33 (a new pick replaces the source and its marker)
        crate::anims::remove(w, old);
    }
    if let Some(name) = w.rules().superweapons.chrono_placement.clone() {
        let spawn = crate::anims::Spawn {
            role: crate::anims::Role::SourceMarker(h),
            ..Default::default()
        };
        crate::anims::spawn(w, &name, Some(p), spawn);
    }
    // spec: S-0513 claim 33 (the source marker over the cell, seen and heard by the launching player only; a new pick replaces it; claim 5: no announcement, no radar event)
    super::event(w, SwEvent::SourceMarker, Some(h), Some(i), Some(p));
}

/// Where in a cell a member keeps its position (claim 36).
fn within(p: Pos) -> (i32, i32) {
    // spec: S-0204 claim 2 (the position within its cell, 0 … 255 per axis)
    (p.x.rem_euclid(256), p.y.rem_euclid(256))
}

/// The landing point in cell `c` of an object at `from` (claim 36).
fn landing_in(w: &World, kind: Kind, from: Pos, c: CellXY) -> Pos {
    let centre = Pos::centre(c, 0);
    let (x, y) = if kind == Kind::Vehicle {
        // spec: S-0513 claim 36 (a vehicle lands at that cell's centre)
        (centre.x, centre.y)
    } else {
        // spec: S-0513 claim 36 (an infantry unit or aircraft keeps its position within the cell)
        let (dx, dy) = within(from);
        (c.x * 256 + dx, c.y * 256 + dy) // spec: S-0204 claim 2 (256 leptons per cell)
    };
    let mut z = w.surface_at(x, y);
    if w.high_bridge(c) {
        // spec: S-0513 claim 36 (on a high-bridge cell the bridge lift is added)
        z += crate::teleport::DECK_HEIGHT;
    }
    Pos::new(x, y, z)
}

/// Whether a listed object is in the air, so that it is not examined
/// (claim 34, revision 108): an aircraft by its in-the-air answer, any
/// other object when it is airborne by height. No jumpjet state and no
/// layer is read.
fn in_air(w: &World, id: ObjectId) -> bool {
    let Some(o) = w.obj(id) else { return false };
    if o.kind == Kind::Aircraft {
        // spec: S-0513 r108 claim 34 (an aircraft by S-0501 claim 18's in-the-air answer)
        return crate::combat::weapon::in_air(w, id);
    }
    // spec: S-0513 r108 claim 34 (any other object when it is airborne, S-0307, its height taken above the deck while its on-deck state is set; neither a jumpjet state nor a layer is read, Q-2419) and S-0307 Terms (airborne: at least 208 leptons above the surface)
    // spec: S-0102 r42 claim 3 (a unit with jumpjet movement that is airborne, S-0307, is never a member, whether or not it is listed, while one listed in the block and not airborne is examined, S-0513 claim 34)
    crate::combat::weapon::airborne_by_height(w, id)
}

/// The organic flag without the teleporter flag (claim 35).
fn dies(w: &World, id: ObjectId) -> bool {
    let Some(o) = w.obj(id) else { return false };
    let organic = crate::combat::veterancy::type_combat(w, id).is_some_and(|t| t.organic);
    let teleporter = w
        .rules()
        .unit(o.kind, o.type_index)
        .and_then(|u| u.raw.teleporter)
        .unwrap_or(false);
    organic && !teleporter
}

/// A vehicle still inside the vehicle factory it is in contact with
/// (claim 34).
fn in_factory(w: &World, id: ObjectId) -> bool {
    let Some(o) = w.obj(id) else { return false };
    if o.kind != Kind::Vehicle {
        return false;
    }
    let Some(k) = o.contact else { return false };
    let Some(b) = w.obj(k) else { return false };
    crate::econ::release::vehicle_factory(w, k)
        && crate::econ::placement::foundation_cells(w, b.type_index, b.pos.cell())
            .contains(&o.pos.cell())
}

/// A warp launch (claim 34).
pub fn launch(w: &mut World, h: HouseId, source: CellXY, dest: CellXY) {
    // spec: S-0513 claim 5 (teleport warp: activation announcement; radar events at the source and destination cells)
    super::event(w, SwEvent::Launch, Some(h), None, None);
    let ps = super::cell_point(w, source, false);
    super::event(w, SwEvent::Radar, Some(h), None, Some(ps));
    let pd = super::cell_point(w, dest, false);
    super::event(w, SwEvent::Radar, Some(h), None, Some(pd));
    let (blast, blast_dest) = {
        let s = &w.rules().superweapons;
        (s.chrono_blast.clone(), s.chrono_blast_dest.clone())
    };
    // spec: S-0513 claim 34 (the blast animations, one at the source centre, one at the destination centre; Constants: 5 leptons lift)
    super::protect::invoke_anim(w, blast, source, true);
    super::protect::invoke_anim(w, blast_dest, dest, true);
    // spec: S-0513 claim 34 (the launch ends the source marker)
    for old in crate::anims::with_role(w, crate::anims::Role::SourceMarker(h)) {
        crate::anims::remove(w, old);
    }
    super::event(w, SwEvent::MarkerEnd, Some(h), None, None);
    let targets: Vec<(usize, CellXY)> = super::block(w, dest);
    for (i, c) in super::block(w, source) {
        // spec: S-0513 F1 (a warp maps source cell i to destination cell i)
        let to = targets.iter().find(|(j, _)| *j == i).map(|&(_, d)| d);
        for l in super::swept_layers(w, c) {
            super::walk_list(w, c, l, &mut |w, id| examine(w, h, id, to));
        }
    }
}

/// One examined object of the source block (claims 34 and 35).
fn examine(w: &mut World, h: HouseId, id: ObjectId, to: Option<CellXY>) {
    // spec: S-0209 r45 claim 13 and S-0513 claims 4 and 34 (only objects listed in a cell are examined: units with drop-pod movement and tunnelling units while digging are in no cell layer and never taken; our engine builds neither movement, `rules::Locomotor::None`, and the drop-pod stand-in ends at once, S-0607 claims 23 and 24, so every such unit stands listed in its cell)
    let Some(o) = w.obj(id) else { return };
    if !o.kind.is_mobile() {
        // spec: S-0513 claim 34 (buildings and other objects are never moved)
        return;
    }
    if in_air(w, id) {
        // spec: S-0513 r108 claim 34 (a listed object in the air is not examined, so nothing below applies to it, a parasite's release included)
        return;
    }
    // spec: S-0513 claim 34 (a naval parasite inside is released with 500 ticks of suppression before any other effect of the warp on that object; S-0614 claim 12)
    crate::parasite::superweapon_teleport(w, id);
    let protected = super::protected(w, id);
    if dies(w, id) && (!protected || w.rules().quirks.warp_kill_before_protection) {
        // spec: S-0513 claim 35 (organic without the teleporter flag: a whole-strength demolition hit ignoring defences, the launching house; quirks.warp_kill_before_protection: protection does not prevent it)
        super::demolish(w, id, true, Some(h));
        return;
    }
    let busy = w.obj(id).is_some_and(|o| o.sw.warp.is_some());
    // spec: S-0513 claim 34 (left in place if protected, already has a warp pending, is being erased, is in a leaving stage, is phasing in, or is a vehicle still inside the vehicle factory it is in contact with)
    if protected
        || busy
        || crate::econ::power::being_erased(w, id)
        || crate::teleport::phasing_in(w, id)
        || in_factory(w, id)
        || chrono_seam(w, id)
    {
        return;
    }
    // spec: S-0513 claims 49 and 50 (an off-map destination cell is no early rejection: the object joins the warp after the parasite and organic tests above, with the fallback cell coordinates (0, 0); the first fallback is reported)
    join(w, h, id, to.unwrap_or(STAND_IN), to.is_none());
}

/// The object joins the warp group (claim 34).
fn join(w: &mut World, h: HouseId, id: ObjectId, to: CellXY, fell_back: bool) {
    let Some(o) = w.obj(id).cloned() else { return };
    // spec: S-0513 claim 34 and S-0606 claim 14 (a bunkered vehicle's link ends in place with the walls-down effects; the bunker goes back to clearing)
    crate::structures::bunker::release_in_place(w, id);
    // spec: S-0513 claim 34 (its docking, repair and similar contact links end)
    if crate::depot::link(w, id).is_some() {
        crate::depot::release(w, id);
    }
    if o.econ.harvest.is_some() {
        crate::econ::harvest::break_contact(w, id);
    }
    let ids: Vec<ObjectId> = w.objects.keys().copied().collect();
    for x in ids {
        if let Some(ob) = w.obj_mut(x)
            && ob.contact == Some(id)
        {
            ob.contact = None;
        }
    }
    if o.kind == Kind::Vehicle {
        // spec: S-0513 claim 34 (a vehicle ceases to occupy its cell)
        crate::drive::release_claims(w, id);
        if let Some(ob) = w.obj_mut(id)
            && ob.drive.is_some()
        {
            ob.drive = Some(crate::drive::DriveState::new());
        }
    }
    let landing = landing_in(w, o.kind, o.pos, to);
    if let Some(ob) = w.obj_mut(id) {
        ob.contact = None;
        // spec: S-0513 claim 51 (launch immediately clears the old path, the queued destinations and the current driving step; none resumes when the warp ends. Target and current destination are dropped by the leaving-stage rule of claim 48, in the first update, not here)
        ob.queued_dests.clear();
        ob.path.steps.clear();
        if let Some(wk) = ob.walk.as_mut() {
            wk.fraction = false; // spec: S-0513 claim 51 (the driving step: a walker's speed fraction ends with it)
        }
        if let Some(hv) = ob.hover.as_mut() {
            hv.point = None; // spec: S-0513 claim 51 (the driving step: a hover's heading point and taken destination)
            hv.for_dest = None;
        }
        if let Some(t) = ob.teleport.as_mut() {
            t.landing = None;
            t.for_dest = None;
            t.mark = None;
        }
        // spec: S-0513 claim 34 and T2 (warp pending; landing point fixed)
        ob.sw.warp = Some(Member {
            stage: Stage::Pending,
            house: h,
            landing,
            redirected: false,
            fell_back,
            refused: false,
            entry_ok: false,
        });
    }
    if fell_back {
        w.note(format!(
            "warp: object {} has no destination cell on the map; it falls back to cell (0, 0) (S-0513 claim 50)",
            id.0
        ));
    }
    release_walk(w, id);
}

/// Ends a walker's step at launch and frees the standing position it was
/// reserving.
fn release_walk(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id).cloned() else { return };
    let Some(wt) = o.walk.as_ref().and_then(|s| s.target) else {
        return;
    };
    let q = crate::objects::occ_quirks(w);
    // spec: S-0513 claim 51 (launch clears the driving step: a walker's target and the standing position it reserved)
    w.cells.clear_infantry(wt.cell(), o.layer, id, q);
    if let Some(s) = w.obj_mut(id).and_then(|ob| ob.walk.as_mut()) {
        s.target = None;
    }
}

/// A member's warp update at the start of its living turn (T2). Returns
/// whether the update was its whole turn: true in every stage before the
/// arrival hold. For a member still leaving that is S-0513 claim 48's
/// confirmed exclusion; for the pending, departure, retry, placement and
/// arrival-effects turns it is the engine design of claim 51 (no ordinary
/// self-healing, behaviour processing, movement, firing or turret turning
/// before or after the warp work). In the arrival hold the hold-end test
/// comes first and the ordinary turn follows (claims 48 and 51).
pub fn turn(w: &mut World, id: ObjectId) -> bool {
    let Some(m) = member(w, id) else {
        return false;
    };
    let t = w.tick;
    match m.stage {
        Stage::Pending => {
            // spec: S-0215 claim 2 (a vehicle in the leaving stage under a pending point suppression, S-0207 claim 27, keeps its destination; the target is dropped in every case)
            let keeps = crate::refresh::suppression_pending(w, id);
            if let Some(ob) = w.obj_mut(id) {
                // spec: S-0513 claim 48 (a member in its leaving stage drops its target and destination) and claim 51 (this design adds no earlier target clearing)
                crate::spawn::clear_own_target(ob);
                ob.combat.burst = 0; // spec: S-0301 claim 4 (a cleared target resets the burst counter)
                if !keeps {
                    ob.dest = None;
                }
            }
            // spec: S-0513 claim 39 and T2 (the leaving stage in t_L + 1; the fixed wait starts)
            set_member(
                w,
                id,
                Some(Member {
                    stage: Stage::Leaving(t),
                    ..m
                }),
            );
            true
        }
        Stage::Leaving(since) => {
            if t.wrapping_sub(since) >= LEAVING_WAIT {
                // spec: S-0513 claim 39 and T2 (the wait expires 60 elapsed ticks later; it stays at its old position, still being erased)
                set_member(
                    w,
                    id,
                    Some(Member {
                        stage: Stage::Expired,
                        ..m
                    }),
                );
                // spec: S-0513 claim 39 (when the wait expires, a member with no target tries a target search with the conditional 4-to-8-tick draw of S-0207 claim 20 f)
                crate::teleport::phasing_search(w, id);
            }
            true
        }
        Stage::Expired => {
            depart(w, id, m);
            true
        }
        Stage::Blocked => {
            check_or_redirect(w, id);
            true
        }
        Stage::Away => {
            place(w, id, m);
            true
        }
        Stage::Placed => {
            arrive(w, id, m);
            true
        }
        Stage::Hold(ta) => {
            let held = w.obj(id).map_or(0, |o| o.sw.held_delay);
            let hold = if w.rules().quirks.warp_hold_persists || m.redirected {
                held
            } else {
                // spec: S-0513 claim 40 (quirks.warp_hold_persists off: 0 for a warp that was not redirected)
                0
            };
            // spec: S-0513 claim 40 (phasing in ends in tick t_A + max(1, H))
            if t.wrapping_sub(ta) >= hold.max(1) {
                set_member(w, id, None);
                // spec: S-0513 claim 39 and T2 (when the hold ends, a member with no target tries a target search)
                crate::teleport::phasing_search(w, id);
            }
            false
        }
    }
}

/// Whether the member's turn is the warp update alone: from its first
/// pending update through arrival effects, the arrival hold excluded
/// (S-0513 claim 51; Q-0870 answered).
pub fn suppresses_ordinary_work(w: &World, id: ObjectId) -> bool {
    // spec: S-0513 claim 51 and S-0203 claim 4 (the warp update is the whole member turn before the arrival hold; the hold's ordinary opportunities return after the hold-end test)
    member(w, id).is_some_and(|m| !matches!(m.stage, Stage::Hold(_)))
}

/// The departure in t_L + 62 (claim 39).
fn depart(w: &mut World, id: ObjectId, m: Member) {
    let Some(o) = w.obj(id).cloned() else { return };
    let warp_out = w.rules().warp_out.clone();
    if let Some(a) = warp_out {
        // spec: S-0513 claim 39 (the warp-out animation at its old position, with its creation draws)
        crate::combat::effects::anim(w, &a, o.pos);
    }
    // spec: S-0513 claim 39 (the departure sound at its old position)
    crate::teleport::chrono_sound(w, &o, Sound::ChronoOut, o.pos);
    // The parasite is not released here: a naval parasite already left at
    // launch (claim 34), and a drone parasite's release at this departure is
    // unresolved (S-0513 claim 39; S-0614 claim 12, P7). So neither
    // `parasite::host_jumps` nor `objects::take_inside` (which releases it,
    // S-0614 claim 15) is used.
    // The departure drops nobody's target: no `teleport::targeters_drop`
    // (S-0207 claim 20 e's jump is not this departure) and no
    // `combat::held::holders_drop` (S-0309 claim 27 does not apply to it).
    // spec: S-0513 claim 39 (departure clears no holder's target, takes no holder timer draw over 4 to 8 ticks whatever its timer shows, reverts no holder with an override in effect, S-0215 claim 10, and leaves a projectile in flight towards the member aimed at it, S-0306 claim 17)
    lift_off(w, id);
    // spec: S-0513 claim 39a (Departure: a temporal beam attached to the member stays attached with its amount, and the member is no longer being erased)
    crate::temporal::departed(w, id);
    // spec: S-0513 claim 39 and T2 (it disappears from its old position and is off the map until it appears; being erased ends; phasing in; the landing check)
    set_member(
        w,
        id,
        Some(Member {
            stage: Stage::Blocked,
            ..m
        }),
    );
    check_or_redirect(w, id);
}

/// Takes the member off the map without leaving the update order and
/// without releasing a parasite (claim 39).
fn lift_off(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id).cloned() else { return };
    // spec: S-0615 claim 14 (taken off the map in any other way: lost to its disc lasers at once)
    crate::disc::object_lost(w, id);
    // spec: S-0516 claim 22 (superweapon teleport: the member's removal and appearance are not ordinary leaving-play and entering-play sensor changes; its sensor disc stays at the old recorded cell through departure, landing retries and appearance, and moves at the arrival-effects update)
    let disc = w.obj_mut(id).and_then(|ob| ob.cloak.disc.take());
    crate::econ::events::leaving_map(w, id, false);
    // spec: S-0609 claims 11 and 13 (leaving the map ends the spin sound's loop and clears the playing mark)
    crate::gattling::leaving_map(w, id);
    if let Some(ob) = w.obj_mut(id) {
        ob.cloak.disc = disc;
    }
    if o.kind == Kind::Aircraft {
        // spec: S-0209 claim 1 (an aircraft leaving clears its object-present mark)
        crate::air::clear_mark(w, id);
        // spec: S-0501 claim 47 (b) (leaving the map in any way ends the tracking)
        crate::air::leaving_map(w, id);
    }
    if o.kind == Kind::Vehicle {
        crate::drive::release_claims(w, id);
    }
    let cell = o.pos.cell();
    w.cells.leave_both(cell, id);
    let q = crate::objects::occ_quirks(w);
    if let Some(wt) = o.walk.as_ref().and_then(|s| s.target) {
        w.cells.clear_infantry(wt.cell(), o.layer, id, q);
    }
    w.cells.clear_infantry(cell, o.layer, id, q);
    if let Some(ob) = w.obj_mut(id) {
        ob.on_map = false;
        if let Some(s) = ob.walk.as_mut() {
            *s = crate::walk::WalkState::default();
        }
    }
    if o.layer == Layer::Ground {
        // spec: S-0202 claim 14 (taking an object out of its occupied ground cell recalculates it)
        crate::load::recalc_cells(w, &[cell]);
    }
    crate::regions::cell_changed(w, cell);
}

/// The movement zone of the redirect search (claim 37).
fn redirect_zone(z: MovementZone) -> MovementZone {
    use MovementZone as Z;
    match z {
        // spec: S-0513 claim 37 (fly and destroyer read as normal, amphibious destroyer as amphibious)
        Z::Fly | Z::Destroyer => Z::Normal,
        Z::AmphibiousDestroyer => Z::Amphibious,
        other => other,
    }
}

/// What the landing check found (claim 37).
enum Check {
    Pass,
    Blocked,
    Killed,
}

/// The landing check (claim 37), then the redirection of a blocked member.
fn check_or_redirect(w: &mut World, id: ObjectId) {
    let Some(m) = member(w, id) else { return };
    match check(w, id, &m) {
        Check::Killed => {}
        Check::Pass => {
            // spec: S-0513 claim 50 (a passed check is not a reservation; the entry test of the checked cell is remembered to detect a terrain or deck change before the placement)
            let entry_ok = enterable_now(w, id, m.landing.cell());
            // spec: S-0513 T2 (check passed: placed on the next update)
            set_member(
                w,
                id,
                Some(Member {
                    stage: Stage::Away,
                    entry_ok,
                    ..m
                }),
            );
        }
        Check::Blocked => redirect(w, id, m),
    }
}

/// The landing check of claim 37.
fn check(w: &mut World, id: ObjectId, m: &Member) -> Check {
    let Some(o) = w.obj(id).cloned() else {
        return Check::Killed;
    };
    let c = m.landing.cell();
    // spec: S-0513 claim 50 (a point outside the map is blocked, never accepted as an empty legal cell)
    if w.cells.cell(c).is_none() {
        return Check::Blocked;
    }
    let r = w.rules_arc();
    if w.high_bridge(c) {
        let (class, zone) = r
            .unit(o.kind, o.type_index)
            .map_or((None, MovementZone::Normal), |u| (u.class, u.zone));
        // spec: S-0513 claim 37 (a high-bridge cell whose deck may not be entered blocks the landing; S-0211 claim 10 ii)
        if !crate::passability::enterable_by_class(&r, w, c, class, zone, true) {
            return Check::Blocked;
        }
    }
    let mut blocked = false;
    let mut killed = false;
    for l in super::swept_layers(w, c) {
        super::walk_list(w, c, l, &mut |w, v| {
            if killed || v == id {
                return;
            }
            let Some(vo) = w.obj(v) else { return };
            let vk = vo.kind;
            let vpos = vo.pos;
            if super::protected(w, v) {
                // spec: S-0513 claim 37 (any protected object, a shielded building too, kills the member: whole strength, distance 0, demolition warhead, ignoring defences, no attacker and no attacking house)
                super::demolish(w, id, true, None);
                killed = true;
                return;
            }
            match vk {
                Kind::Infantry if o.kind == Kind::Infantry => {
                    if vpos.x == m.landing.x && vpos.y == m.landing.y {
                        // spec: S-0513 claim 37 (an unprotected infantry unit, the member infantry too, is killed only when it stands exactly on the landing point)
                        super::demolish(w, v, true, None);
                    }
                    // spec: S-0513 claim 37 (otherwise both share the cell)
                }
                Kind::Infantry | Kind::Vehicle | Kind::Aircraft => {
                    // spec: S-0513 claim 37 (any other unprotected infantry unit, vehicle or aircraft is killed)
                    super::demolish(w, v, true, None);
                }
                // spec: S-0513 claim 37 (any other object, an unprotected building or a terrain object, blocks the landing)
                _ => blocked = true,
            }
        });
    }
    if killed || w.obj(id).is_none_or(crate::combat::damage::dead) {
        set_member(w, id, None);
        return Check::Killed;
    }
    if blocked { Check::Blocked } else { Check::Pass }
}

/// A blocked member's redirected landing (claims 37 and 40).
fn redirect(w: &mut World, id: ObjectId, m: Member) {
    let Some(o) = w.obj(id).cloned() else { return };
    let r = w.rules_arc();
    let Some(u) = r.unit(o.kind, o.type_index) else {
        return;
    };
    let c = m.landing.cell();
    let (class, zone) = if r.quirks.warp_redirect_as_tracked {
        // spec: S-0513 claim 37 (locomotion class track whatever the member's own; its zone with fly and destroyer as normal, amphibious destroyer as amphibious; quirks.warp_redirect_as_tracked)
        (Some(LocomotionClass::Track), redirect_zone(u.zone))
    } else {
        // spec: S-0513 claim 37 (quirk off: the member's own class and unsimplified zone)
        (u.class, u.zone)
    };
    crate::regions::ensure(w);
    let deck = w.high_bridge(c);
    // spec: S-0513 claim 37 (reach region of the member's current cell)
    let reach = crate::regions::reach_read(w, o.pos.cell(), zone, o.layer == Layer::Deck);
    // spec: S-0513 claim 37 (the nearby-cell search of S-0212 claim 25 centred on the landing cell: no height condition, bridge cells allowed, deck mode on a deck landing cell, no reference cell)
    let settings = crate::path::NearbySettings {
        class,
        zone,
        reach: Some(reach),
        deck,
        height: false,
        reference: None,
        no_bridges: false,
        no_overlay: false,
    };
    let found = crate::path::nearby(w, c, crate::path::NearbyUse::Settings(settings));
    let mut fell_back = m.fell_back;
    let landing = match found {
        Some(f) => {
            // spec: S-0513 claim 37 (its landing point moves to the found cell with the same position within the cell)
            landing_in(w, o.kind, m.landing, f)
        }
        None => {
            // spec: S-0513 claim 49 (a failed search replaces the landing point without a success test and does not keep the old point; later checks use the replaced point) and claim 50 (the fallback cell (0, 0), keeping the position within the previous landing cell; height 0 and no bridge lift while that cell is not a legal cell)
            fell_back = true;
            landing_in(w, o.kind, m.landing, STAND_IN)
        }
    };
    let delay = r.superweapons.chrono_delay;
    if let Some(ob) = w.obj_mut(id) {
        // spec: S-0513 claim 40 (a landing that had to be redirected sets the held delay to the rules' teleport delay; nothing resets it) and claim 49 (a retry sets it also when the check afterwards passes, and also when the search failed)
        ob.sw.held_delay = delay;
    }
    // spec: S-0513 T2 (check failed: the check again in each later update)
    set_member(
        w,
        id,
        Some(Member {
            stage: Stage::Blocked,
            landing,
            redirected: true,
            fell_back,
            ..m
        }),
    );
    if fell_back && !m.fell_back {
        w.note(format!(
            "warp: object {} found no nearby landing cell; it falls back to cell (0, 0) and retries (S-0513 claim 50)",
            id.0
        ));
    }
}

/// Whether the member could enter cell `c` on the layer it would land on
/// (the entry test of claim 38).
fn enterable_now(w: &World, id: ObjectId, c: CellXY) -> bool {
    let Some(o) = w.obj(id) else { return false };
    let r = w.rules_arc();
    let (class, zone) = r
        .unit(o.kind, o.type_index)
        .map_or((None, MovementZone::Normal), |u| (u.class, u.zone));
    let deck = crate::teleport::landing_layer(w, c) == Layer::Deck;
    crate::passability::enterable_by_class(&r, w, c, class, zone, deck)
}

/// Whether the placement at the passed landing check's point may go ahead
/// now (claim 50, Q-0871 answered): a passed check is not a reservation. It
/// refuses an illegal cell, a newly blocking building or terrain object, a
/// newly present protected object, a member that does not fit on the selected
/// layer with the occupants there (no vehicle or aircraft shares, an infantry
/// unit only takes a free standing position), and a terrain or deck change
/// since the check that makes the landing unenterable. Current occupants are
/// never killed, displaced or ignored; an occupant already destroyed does not
/// count.
fn placement_accepted(w: &World, id: ObjectId, m: &Member) -> bool {
    let c = m.landing.cell();
    let Some(o) = w.obj(id) else { return false };
    // spec: S-0513 claim 50 (an illegal cell refuses the placement)
    if w.cells.cell(c).is_none() {
        return false;
    }
    let layer = crate::teleport::landing_layer(w, c);
    for l in super::swept_layers(w, c) {
        for &v in w.cells.list(c, l) {
            let Some(vo) = w.obj(v) else { continue };
            if v == id || crate::combat::damage::dead(vo) {
                continue;
            }
            // spec: S-0513 claim 50 (a newly present protected object refuses the placement; the next update's full check then kills the member)
            if super::protected(w, v) {
                return false;
            }
            match vo.kind {
                Kind::Infantry | Kind::Vehicle | Kind::Aircraft => {}
                // spec: S-0513 claim 50 (a newly blocking building or terrain object refuses the placement)
                _ => return false,
            }
            if l != layer {
                continue;
            }
            match vo.kind {
                // spec: S-0513 claim 50 and S-0209 claims 4 and 5 (incompatible vehicles or aircraft refuse sharing; the member takes no cell with a vehicle or aircraft in it, and no other member shares with an infantry unit)
                Kind::Vehicle | Kind::Aircraft => return false,
                _ if o.kind != Kind::Infantry => return false,
                _ => {}
            }
        }
    }
    if o.kind == Kind::Infantry
        && let Some(rec) = w.cells.occ(c, layer)
    {
        let top = w.rules().quirks.top_position_counts_as_centre;
        let p = crate::walk::position_class(
            m.landing.x.rem_euclid(256), // spec: S-0208 claim 2 (the position within its cell, 0 to 255 per axis)
            m.landing.y.rem_euclid(256), // spec: S-0208 claim 2 (the position within its cell, 0 to 255 per axis)
            top,
        );
        // spec: S-0513 claim 50 and S-0209 claim 4 (infantry share a cell only when the landing position is free: no over-capacity overlap; a mark of an infantry unit already destroyed does not count)
        let live = |mk: &crate::cells::Mark| match mk.by {
            crate::cells::Marker::Infantry(x, _) => w
                .obj(x)
                .is_some_and(|xo| x != id && !crate::combat::damage::dead(xo)),
            crate::cells::Marker::Terrain(_) | crate::cells::Marker::Anim(..) => true,
        };
        if rec.marks[usize::from(p)].as_ref().is_some_and(live) {
            return false;
        }
    }
    // spec: S-0513 claim 50 (a terrain or deck change since the passed check that makes the landing no longer enterable refuses it; terrain that was already unenterable at the check keeps its explicit arrival fate of claim 38)
    !(m.entry_ok && !enterable_now(w, id, c))
}

/// The placement at the landing point (T2: check passed, next update).
fn place(w: &mut World, id: ObjectId, m: Member) {
    let Some(o) = w.obj(id).cloned() else { return };
    let c = m.landing.cell();
    // spec: S-0513 claim 50 (a placement failure never deletes a member: it stays alive and off the map in the landing-retry stage with its landing point, owner and passengers retained; the next update repeats the full landing check of claim 37; the failed placement itself applies no landing-check hits and makes no redirect or draw; no retry cap and no timeout death; the first failure per member and warp is reported)
    if !placement_accepted(w, id, &m) {
        if !m.refused {
            w.note(format!(
                "warp: object {} could not be placed at its landing point and stays in the landing retry (S-0513 claim 50)",
                id.0
            ));
        }
        set_member(
            w,
            id,
            Some(Member {
                stage: Stage::Blocked,
                refused: true,
                ..m
            }),
        );
        return;
    }
    let layer = crate::teleport::landing_layer(w, c);
    if let Some(ob) = w.obj_mut(id) {
        ob.layer = layer;
        if let Some(t) = ob.teleport.as_mut() {
            *t = crate::teleport::TeleportState::default();
        }
    }
    let facing = o.body.value(w.tick);
    // spec: S-0513 claim 39 (appears at its landing point in its next update)
    // spec: S-0513 r115 claim 38 (the placement at its landing point is an appearance at the height of claim 36, not S-0501 claim 1's rule: `air::placed` reads the member state, which still holds here)
    crate::objects::place(w, id, m.landing, facing);
    // spec: S-0507 claim 15 (its passengers' positions become its position at once)
    crate::transport::sync(w, id);
    set_member(
        w,
        id,
        Some(Member {
            stage: Stage::Placed,
            ..m
        }),
    );
}

/// Whether the member hovers (claim 38).
fn hovering(w: &World, id: ObjectId) -> bool {
    let Some(o) = w.obj(id) else { return false };
    let r = w.rules();
    let hover = r
        .unit(o.kind, o.type_index)
        .is_some_and(|u| u.locomotor == Locomotor::Hover);
    if !hover {
        return false;
    }
    let powered = o.kind == Kind::Vehicle
        && r.econ
            .power
            .powered_unit
            .get(o.type_index as usize)
            .copied()
            .unwrap_or(false);
    // spec: S-0513 claim 38 (a hover type counts as not hovering when its type has the powered-unit flag and its owner has no active control centre, S-0405 claim 13)
    !(powered
        && o.owner
            .is_none_or(|h| !crate::econ::power::has_centre(w, h, o.type_index)))
}

/// The arrival effects (claims 38, 39 and 41).
fn arrive(w: &mut World, id: ObjectId, m: Member) {
    let Some(o) = w.obj(id).cloned() else { return };
    let t = w.tick;
    // spec: S-0513 claim 39 (the arrival sound)
    crate::teleport::chrono_sound(w, &o, Sound::ChronoIn, o.pos);
    let c = o.pos.cell();
    let ground_protected = w
        .cells
        .list(c, Layer::Ground)
        .to_vec()
        .into_iter()
        .any(|x| x != id && super::protected(w, x));
    let r = w.rules_arc();
    let tc = crate::combat::veterancy::type_combat(w, id);
    let naval = tc.as_ref().is_some_and(|t| t.naval);
    // spec: S-0513 r115 claim 36 (an aircraft member that lands on a high-bridge cell from the ground appears on the deck; its own deck mark is its standing-on-deck state, S-0501 claim 43, `air::placed`)
    let (land_type, deck) = (
        w.cells.cell(c).map_or(land::CLEAR, |x| x.land),
        o.layer == Layer::Deck || crate::air::deck_marked(w, id),
    );
    let water = land_type == land::WATER;
    let infantry = o.kind == Kind::Infantry;
    let (class, zone) = r
        .unit(o.kind, o.type_index)
        .map_or((None, MovementZone::Normal), |u| (u.class, u.zone));
    let mut fate_done = false;
    if ground_protected {
        // spec: S-0513 claim 38 (a protected object listed in the landing cell's ground layer, even under a deck, kills the member as in claim 37)
        super::demolish(w, id, true, None);
        fate_done = true;
    } else if water && !infantry && !naval && !hovering(w, id) && !deck {
        // spec: S-0513 claims 38 and 41 (a non-infantry, non-naval, non-hovering member on water off a deck sinks, its kill credited to the launching house)
        sink(w, id, Some(m.house));
        fate_done = true;
    } else {
        let road = land_type == land::ROAD && (r.quirks.warp_naval_road_cell || !water);
        let enters = crate::passability::enterable_by_class(&r, w, c, class, zone, deck);
        // spec: S-0513 claim 38 (a member for which the cell cannot be entered, or a naval member on a deck or road cell; quirks.warp_naval_road_cell)
        if !enters || (naval && (deck || road)) {
            if water && !infantry {
                // spec: S-0513 claims 38 and 41 (sinks with no credit on a water tile when not infantry)
                sink(w, id, None);
            } else {
                // spec: S-0513 claims 38 and 41 (otherwise killed with no credit, as in claim 37)
                super::demolish(w, id, true, None);
            }
            fate_done = true;
        }
    }
    let warp_out = r.warp_out.clone();
    let pos = w.obj(id).map_or(o.pos, |x| x.pos);
    let gone = fate_done
        && w.obj(id)
            .is_none_or(|x| crate::combat::damage::dead(x) || crate::naval::sinking(w, id));
    if !gone {
        // spec: S-0516 claim 22 (a surviving member's cell-arrival update comes at this arrival-effects update, after the arrival sound and the arrival-fate test and before the landing animation and the arrival hold: its disc transfers to the destination and the adjacent sensor test applies; a member killed by the fate test gets none; it does not wait for its own locomotor or the hold)
        crate::cloak::machine::cell_movement_completed(w, id);
    }
    if let Some(a) = warp_out {
        // spec: S-0513 claim 39 (the warp-out animation at the new position)
        crate::combat::effects::anim(w, &a, pos);
    }
    if gone {
        set_member(w, id, None);
        return;
    }
    // spec: S-0513 claims 39 and 40 (the arrival hold starts)
    set_member(
        w,
        id,
        Some(Member {
            stage: Stage::Hold(t),
            ..m
        }),
    );
}

/// A sinking on arrival, credited to `house` or to nobody (claims 38 and
/// 41; as a cull's sinking, S-0614 claim 11, without a hit).
fn sink(w: &mut World, id: ObjectId, house: Option<HouseId>) {
    let warhead = w.rules().combat.general.c4_warhead;
    let hit = crate::combat::damage::Hit {
        amount: 0,
        warhead,
        distance: 0,
        attacker: None,
        house,
        ignore_defences: true,
        no_escape: false,
    };
    // spec: S-0513 claim 41 (the sinking's kill credited to the launching house, or to nobody)
    crate::combat::damage::credit(w, id, &hit);
    crate::naval::start_sinking(w, id);
}
