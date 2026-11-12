//! Paradrop delivery after the launch (S-0522 claims 6 to 14): the plane's
//! approach and overfly processing, its passes, drop attempts (from the
//! overfly and from a carrying aircraft's shot) and the start of the
//! paratroopers' fall. The launch side, which sends the planes and loads
//! their paratroopers (S-0522 claims 1 to 3 and 16), is
//! `superweapons::planes`; the fall itself is [`crate::fall`].
//!
//! Seams: the parachute's own playback, which could end it before the
//! landing (S-0522 claim 13), needs an animation clock the simulation does
//! not keep (S-0109 claim 28), so the parachute stays attached until the
//! landing; the outcome of a dead "not human" paratrooper is S-0107 claim
//! 17's open question.

use glasnost_map::CellXY;

use crate::air;
use crate::rules::{Behaviour, Kind};
use crate::walk;
use crate::world::{Layer, ObjectId, Pos, World};

/// The pass counter's start and reset value.
// spec: S-0522 Constants (pass counter start and reset: 5 passes)
pub const PASSES: i32 = 5;
/// The processing delay of *paradrop approach*, ticks.
// spec: S-0522 claim 6 and Constants (approach processing delay 3 ticks, no draw)
const APPROACH_DELAY: i32 = 3;
/// The processing delay of *paradrop overfly*, ticks.
// spec: S-0522 claim 7 and Constants (overfly processing delay 5 ticks, no draw)
const OVERFLY_DELAY: i32 = 5;
/// The drop point's distance from the plane, leptons.
// spec: S-0522 F4 and Constants (drop-point distance 128 leptons)
const DROP_DISTANCE: i32 = 128;
/// The side turn of the drop direction, facing units.
// spec: S-0522 F4 and Constants (side turn 16383)
const SIDE_TURN: u16 = 16383;
/// The facing of a dropped paratrooper.
// spec: S-0522 claim 9 and Constants (32768, map south)
const TROOPER_FACING: u16 = 32768;

/// The planar distance from the plane to its target (S-0204 claim 6; a
/// cell target has no building term).
fn target_distance(w: &World, id: ObjectId) -> Option<i32> {
    let o = w.obj(id)?;
    let tg = o.target?;
    // spec: S-0522 claim 6 (planar distance to the target, S-0204 claim 6)
    Some(air::planar(w, o.pos, crate::process::target_pos(w, tg)))
}

fn lose_target(w: &mut World, id: ObjectId) {
    if let Some(o) = w.obj_mut(id) {
        crate::spawn::clear_own_target(o);
        o.combat.burst = 0; // spec: S-0301 claim 4 (a cleared target resets the burst counter)
    }
}

fn passes(w: &World, id: ObjectId) -> i32 {
    w.obj(id)
        .and_then(|o| o.air.as_ref())
        .map_or(0, |a| a.passes)
}

fn set_passes(w: &mut World, id: ObjectId, n: i32) {
    if let Some(a) = w.obj_mut(id).and_then(|o| o.air.as_mut()) {
        a.passes = n;
    }
}

/// *Paradrop approach* (S-0522 claim 6). Returns the delay.
pub fn approach(w: &mut World, id: ObjectId) -> i32 {
    let Some(o) = w.obj(id).cloned() else {
        return APPROACH_DELAY;
    };
    let radius = w.rules().superweapons.para_radius;
    match (o.target, o.dest) {
        (None, _) => {
            // spec: S-0522 claim 6 (no target: it loses its destination and its next behaviour is retreat)
            air::set_dest(w, id, None);
            air::queue(w, id, Behaviour::Retreat);
        }
        (Some(tg), None) => {
            // spec: S-0522 claim 6 (no destination: the target becomes its destination)
            air::set_dest(w, id, Some(tg));
        }
        (Some(_), Some(_)) => {
            // spec: S-0522 claim 6 (planar distance at most the paradrop radius: next behaviour paradrop overfly, pass counter down by 1)
            if target_distance(w, id).is_some_and(|d| d <= radius) {
                air::queue(w, id, Behaviour::ParadropOverfly);
                set_passes(w, id, passes(w, id) - 1);
            }
        }
    }
    APPROACH_DELAY
}

/// *Paradrop overfly* (S-0522 claim 7). Returns the delay.
pub fn overfly(w: &mut World, id: ObjectId) -> i32 {
    let Some(o) = w.obj(id).cloned() else {
        return OVERFLY_DELAY;
    };
    let radius = w.rules().superweapons.para_radius;
    if o.target.is_none() || crate::transport::count(w, id) == 0 {
        // spec: S-0522 claim 7 (no target or no passenger left: run lock off, target and destination lost, next behaviour retreat)
        air::set_lock(w, id, false);
        lose_target(w, id);
        air::set_dest(w, id, None);
        air::queue(w, id, Behaviour::Retreat);
        return OVERFLY_DELAY;
    }
    let inside = target_distance(w, id).is_some_and(|d| d <= radius);
    if !inside {
        // spec: S-0522 claim 7 (outside the radius: the run lock is released)
        air::set_lock(w, id, false);
        if passes(w, id) > 0 {
            // spec: S-0522 claim 7 (pass counter above 0: another pass at the same target)
            air::queue(w, id, Behaviour::ParadropApproach);
        } else {
            // spec: S-0522 claims 7 and 8 (counter at 0: target and destination lost, next behaviour retreat)
            lose_target(w, id);
            air::set_dest(w, id, None);
            air::queue(w, id, Behaviour::Retreat);
        }
        return OVERFLY_DELAY;
    }
    // spec: S-0522 claim 7 (inside the radius it holds the run lock)
    air::set_lock(w, id, true);
    // spec: S-0522 claim 7 (one drop attempt when the cell of its own position is inside the usable area, height-aware; none outside it)
    if w.cells.usable(o.pos.cell()) {
        attempt(w, id);
    }
    OVERFLY_DELAY
}

/// One drop attempt of aircraft `plane` (S-0522 claim 9): its first
/// passenger leaves when the drop point allows it. Returns whether the
/// drop succeeded. Also the shot of a carrying aircraft (claim 11).
pub fn attempt(w: &mut World, plane: ObjectId) -> bool {
    let Some(p) = crate::transport::passengers(w, plane).first().copied() else {
        return false;
    };
    let Some(o) = w.obj(plane).cloned() else {
        return false;
    };
    // spec: S-0522 claim 9 (the plane's ammunition goes down by 1)
    let ammo = o.combat.ammo - 1;
    if let Some(ob) = w.obj_mut(plane) {
        ob.combat.ammo = ammo;
    }
    let f = o.body.value(w.tick);
    // spec: S-0522 F4 (odd new ammunition: f − 16383, the plane's left; even: f + 16383; wrapped to 16 bits)
    let side = if ammo.rem_euclid(2) == 1 {
        f.wrapping_sub(SIDE_TURN)
    } else {
        f.wrapping_add(SIDE_TURN)
    };
    let mode = w.rules().mode;
    // spec: S-0522 F4 (the plane's position moved 128 leptons in direction f′ by S-0501 F3's horizontal step, table trigonometry)
    let bias = crate::table_trig::bias(w);
    let (x, y) = walk::step_point_b(o.pos.x, o.pos.y, side, DROP_DISTANCE, true, mode, bias);
    match landing(w, p, x, y) {
        Some((c, at)) => {
            release(w, plane, p, c, Pos::new(at.0, at.1, o.pos.z));
            true
        }
        None => {
            // spec: S-0522 claim 9 (on failure the ammunition goes back up by 1; the paratrooper stays first passenger)
            if let Some(ob) = w.obj_mut(plane) {
                ob.combat.ammo = ammo + 1;
            }
            false
        }
    }
}

/// The landing test of claim 9 at drop point (`x`, `y`) for paratrooper
/// `p`: the cell and the standing position's point, or `None` when the
/// drop is refused.
///
/// The order of the tests is claim 9's: the entry outcome first (a refusal
/// there suppresses the standing-position selection entirely, so it takes no
/// draw); once entry is clear the standing position is selected, and the
/// usable-area, deck-entry-mark and nearby-cell qualification refusals that
/// follow have each taken the centre-class row's draw (claims 9 and 16).
fn landing(w: &mut World, p: ObjectId, x: i32, y: i32) -> Option<(CellXY, (i32, i32))> {
    let c = Pos::new(x, y, 0).cell();
    w.cells.cell(c)?;
    let (kind, ti) = w.obj(p).map(|o| (o.kind, o.type_index))?;
    let u = w.rules().unit(kind, ti)?.clone();
    let top = w.rules().quirks.top_position_counts_as_centre;
    let high = w.cells.is_bridge(c);
    // spec: S-0522 claim 9 (S-0209's entry outcome clear under S-0208 claim 19's landing-entry rules: deck objects and record on a high-bridge cell, the level −6 bypass; a refusal here suppresses the standing-position selection, so it takes no draw)
    if !crate::teleport::entry_clear(w, p, c, high) {
        return None;
    }
    // spec: S-0522 claim 9 (S-0208 claim 3 with the drop point as the desired point and the cell's ground occupancy record, also on a bridge cell; no standing-position result also refuses the drop, without reaching the later conditions)
    let class = walk::position_class(x.rem_euclid(256), y.rem_euclid(256), top); // spec: S-0204 claim 2 (the position within its cell, 0 … 255 per axis)
    let pos = walk::choose_position(w, c, Layer::Ground, class, false)?;
    let (su, sv) = walk::standing_point(pos);
    let at = (256 * c.x + su, 256 * c.y + sv); // spec: S-0208 table P (a standing position within the cell)
    // spec: S-0522 claim 9 (the standing position lies inside the usable area)
    if !w.cells.usable(Pos::new(at.0, at.1, 0).cell()) {
        return None;
    }
    // spec: S-0522 claim 9 (a bridge cell has the deck-entry mark)
    if high && !w.cells.deck_entry_at(c) {
        return None;
    }
    // spec: S-0522 claim 9 (S-0212 claim 25's nearby-cell qualification for its class and zone, the whole record empty, a bridge cell judged on its deck)
    if !crate::path::qualifies(w, c, u.class, u.zone, None, true, false, 0) {
        return None;
    }
    Some((c, at))
}

/// A successful drop (S-0522 claim 9): paratrooper `p` leaves `plane` at
/// `at` in cell `c`, falling under its parachute.
fn release(w: &mut World, plane: ObjectId, p: ObjectId, c: CellXY, at: Pos) {
    let plane_pos = w.obj(plane).map(|o| o.pos);
    let owner = w.obj(plane).and_then(|o| o.owner);
    // spec: S-0522 claim 9 (a paratrooper whose cell is a bridge cell is on the bridge deck: its standing-position mark is in the deck record throughout descent and after landing, not the ground record that chose the position; the marking test uses the unit's actual height, the deck at or above the terrain surface plus 416 leptons on a high-bridge cell, the ground below it; exact integer comparisons)
    let layer = if crate::teleport::landing_layer(w, c) == Layer::Deck
        && at.z >= w.surface_at(at.x, at.y) + crate::teleport::DECK_HEIGHT
    {
        Layer::Deck
    } else {
        Layer::Ground
    };
    // spec: S-0522 claim 9 (at the standing position, at the plane's own height, facing 32768) and claim 17 (the ordinary placement fall-back to rest: current behaviour none, guard or area guard queued, no target and no destination)
    crate::transport::drop_out(w, plane, p, at, TROOPER_FACING, layer);
    // spec: S-0522 claim 12 (it falls, from the fall rate it already holds)
    crate::fall::start(w, p, false);
    // spec: S-0522 claim 13 (the parachute is attached when a type is held; an unlisted name was created as an animation type by the importer, S-0004 claims 2 and 13, and takes no draw when it has no fields; only a genuinely absent type attaches nothing)
    let chute = w.rules().superweapons.parachute.clone();
    let slow_fall = w.rules().quirks.paratrooper_any_animation_slow_fall;
    if let Some(ob) = w.obj_mut(p) {
        // spec: S-0522 claim 13 (an absent type: with quirks.paratrooper_any_animation_slow_fall on, the release-tick fall update uses the parachute recurrence once and the no-parachute one from the next tick, with no durable attachment; with it off, the no-parachute recurrence from the first update)
        ob.fall.chute = chute.is_some();
        ob.fall.chute_once = chute.is_none() && slow_fall;
        ob.fall.chute |= ob.fall.chute_once;
    }
    // spec: S-0522 claim 12 and S-0107 claim 21 (the parachute sequence, forced, while it falls)
    crate::infantry::request(w, p, crate::infantry::Seq::Parachute, true);
    if let Some(name) = chute {
        // spec: S-0522 claims 13 and 16 (the parachute's creation draws, S-0202 claim 10, after the standing-position draw)
        // spec: S-0522 claim 13 and S-0109 claim 22 (attached at the paratrooper's position, no creation loop delay, repeat factor 1)
        let spawn = crate::anims::Spawn {
            owner: Some((p, crate::world::Pos::new(0, 0, 0))),
            role: crate::anims::Role::Parachute,
            ..Default::default()
        };
        let made = crate::anims::spawn(w, &name, Some(at), spawn);
        crate::superweapons::event(
            w,
            crate::superweapons::SwEvent::Parachute {
                trooper: p,
                delay: made.delay,
                anim: made.id,
            },
            owner,
            None,
            Some(at),
        );
    }
    if w.rules().superweapons.chute_sound.is_some() {
        // spec: S-0522 claim 9 (the parachute sound plays at the plane's position)
        crate::superweapons::event(
            w,
            crate::superweapons::SwEvent::Chute,
            owner,
            None,
            plane_pos,
        );
    }
    // spec: S-0522 claims 8 and 11 (a successful drop resets the pass counter to 5)
    set_passes(w, plane, PASSES);
}

/// Whether a shot of aircraft `firer` is a drop (S-0501 claim 20, S-0522
/// claim 11): an aircraft that carries passengers.
pub fn shot_drops(w: &World, firer: ObjectId) -> bool {
    w.obj(firer).is_some_and(|o| o.kind == Kind::Aircraft) && crate::transport::count(w, firer) > 0
}
