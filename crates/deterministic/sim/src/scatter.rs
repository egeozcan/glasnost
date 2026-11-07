//! Scatter requests: who accepts them and where they go (S-0208 claims 11
//! to 13; S-0209 claim 11 for cell-wide requests).

use glasnost_map::CellXY;
use glasnost_num::geom::{direction, n_step};

use crate::infantry::{self, Seq};
use crate::occupancy::{self, Outcome};
use crate::path::{self, NearbyUse};
use crate::regions::STEPS;
use crate::rules::{Behaviour, Kind};
use crate::world::{Layer, ObjectId, Pos, Target, World};

/// A scatter request (S-0208 Terms).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScatterRequest {
    /// The point to move away from.
    pub threat: Option<Pos>,
    /// Forced.
    pub forced: bool,
    /// Move anywhere.
    pub anywhere: bool,
}

/// The screen-cover check of a cell at its own level (S-0208 Terms).
pub fn screen_cover_passes(w: &World, c: CellXY) -> bool {
    let Some(cell) = w.cells.cell(c) else {
        return true;
    };
    let l = i32::from(cell.level);
    // spec: S-0208 Terms (screen-cover check: C fails when e(C_i) ≥ ℓ + 2i − 1 for some i from 1 to 6)
    for i in 1..=6 {
        let ci = CellXY::new(c.x + i, c.y + i);
        // spec: S-0208 Terms (outside the map: level 0, not a high-bridge cell; a high-bridge cell counts 4 higher)
        let e = match (w.cells.cell(ci), w.cells.idx(ci)) {
            (Some(x), Some(j)) => i32::from(x.level) + if w.cells.deck[j] { 4 } else { 0 },
            _ => 0,
        };
        if e >= l + 2 * i - 1 {
            return false;
        }
    }
    true
}

/// Rounds a facing to a step direction (S-0208 F6).
fn eighth(f: u16) -> i32 {
    // spec: S-0208 F6 (the 8-step facing of S-0204 F4)
    n_step(f, 8) as i32
}

/// An infantry unit receives a scatter request (S-0208 claims 11 to 13).
/// Returns whether it accepted. With the movement trace on, the answer is
/// recorded.
pub fn request(w: &mut World, id: ObjectId, req: ScatterRequest) -> bool {
    request_with(w, id, req, Assign::MoveOrder)
}

/// How an accepted infantry request assigns its destination.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Assign {
    /// As under a move order: *move* is queued (S-0208 claim 12).
    MoveOrder,
    /// S-0417 claim 17's request to a resting unit on a building cell.
    Resting,
}

/// S-0417 claim 17's request to an infantry unit resting on a cell that
/// holds a building: judged and placed by S-0208 claims 11 and 13, but a
/// destination from the nearby-cell search leaves the current behaviour as
/// it was, and a teleporting unit given one jumps at once (*During the
/// walk-off*). Returns whether it accepted.
pub fn request_resting(w: &mut World, id: ObjectId, req: ScatterRequest) -> bool {
    request_with(w, id, req, Assign::Resting)
}

fn request_with(w: &mut World, id: ObjectId, req: ScatterRequest, how: Assign) -> bool {
    let accepted = answer(w, id, req, how);
    crate::trace::record(w, id, || crate::trace::What::Asked {
        accepted,
        dest: w.obj(id).and_then(|o| match o.dest {
            Some(Target::Cell(c)) => Some(c),
            Some(Target::Object(t)) => w.obj(t).map(|x| x.pos.cell()),
            None => None,
        }),
    });
    accepted
}

/// [`request_with`] without the trace.
fn answer(w: &mut World, id: ObjectId, req: ScatterRequest, how: Assign) -> bool {
    let t = w.tick;
    let Some(o) = w.obj(id).cloned() else {
        return false;
    };
    if o.kind == Kind::Vehicle {
        return vehicle_request(w, id, req);
    }
    if o.kind == Kind::Aircraft {
        // spec: S-0501 claim 49 (r93, Q-2251: every request alike, whoever sends it; its threat point, forced and move-anywhere options, the height, the pad contact and the queued behaviour play no part. The current behaviour with the scatter property, or none at all, an exception to S-0215 claim 1: the aircraft falls back to rest, claim 18; otherwise no effect. It never gives a nearby cell as destination and never makes it take off itself)
        let gate = o.beh.current.is_none_or(|b| w.rules().behaviour(b).scatter);
        if gate {
            crate::process::fall_back(w, id, false);
        }
        return false;
    }
    if o.kind != Kind::Infantry {
        return false;
    }
    let r = w.rules_arc();
    let Some(u) = r.unit(o.kind, o.type_index).cloned() else {
        return false;
    };
    let seq = o.infantry.as_ref().and_then(|s| s.seq);
    // spec: S-0417 claim 17 (No path: a walking unit waiting for the retry delay with a destination has movement in progress, one whose search failed has none; occupancy::in_progress)
    let moving = occupancy::in_progress(w, id);
    let deploying = seq.is_some_and(Seq::is_deploy);
    if req.forced && req.anywhere && deploying {
        // spec: S-0208 claim 11 (a forced, move-anywhere request to a deploying unit also requests undeploy)
        infantry::request(w, id, Seq::Undeploy, false);
    }
    let seq = w
        .obj(id)
        .and_then(|o| o.infantry.as_ref())
        .and_then(|s| s.seq);
    // spec: S-0208 claim 11 (effectively forced: forced and the locomotor not moving)
    let eff_forced = req.forced && !moving;
    // S-0208 Terms: an owner that is not human-controlled is a computer-controlled owner; an
    // AI house counts as human-controlled (glossary *AI house*), the neutral and special houses
    // do not.
    let human = o.owner.is_some_and(|h| w.houses[usize::from(h.0)].human);
    // spec: S-0208 claim 11 (permission: player-scatter rule, the scatter ability, a computer-controlled owner, or move-anywhere; no unit is a team member)
    let permission = r.general.player_scatter
        || !human
        || req.anywhere
        || crate::combat::veterancy::has_ability(w, id, crate::combat::data::Ability::Scatter);
    let scatter_prop = o.beh.seen().is_some_and(|b| r.behaviour(b).scatter);
    // spec: S-0208 claim 11 (first case: a sequence that is not undeploy and not interruptible)
    let first = seq.is_some_and(|q| q != Seq::Undeploy && !q.table().0);
    // spec: S-0208 claim 11 (second case: deploying, human owner, not both forced and move-anywhere)
    let second = seq.is_some_and(Seq::is_deploy) && human && !(req.forced && req.anywhere);
    // spec: S-0208 claim 11 (third case: not effectively forced and not a permitted fraidycat in a scatter behaviour)
    let third = !eff_forced && (!u.fraidycat || !scatter_prop || !permission);
    if first || second || third {
        return false;
    }
    let q = r.quirks;
    // spec: S-0208 claim 6 (r52, A stored path kept under enter: a destination given by a rule that is not an order breaks off no contact, so the walker keeps its path when, at the assignment, its current or queued behaviour is enter and it is in contact with nothing)
    let keeps = crate::walk::keeps_stored_path(w, id);
    // spec: S-0208 claim 6 (r46: the extra locomotion is for a walker standing with no walking target and no stored path at the scatter, read before the request drops the path; r52: or one that keeps its stored path there)
    let free = crate::walk::standing_free(w, id, keeps);
    // spec: S-0208 claims 12 and 13 (one ranged draw j over 0 … 4)
    let j = w.rng.ranged(0, 4, q.ranged_draw_half_span);
    // spec: S-0208 Terms (scatter centre cell: the cell of the scatter reference point, the walking target while there is one, else the unit's own position; never a farther destination)
    let centre = o
        .walk
        .as_ref()
        .and_then(|s| s.target)
        .map_or(o.pos.cell(), |p| p.cell());
    let d0 = match req.threat {
        // spec: S-0208 claim 12 (d₀: the facing from the threat point to the unit, rounded)
        Some(tp) => eighth(direction(
            i64::from(o.pos.x - tp.x),
            i64::from(o.pos.y - tp.y),
            q.table_direction,
        )),
        None => {
            let cc = Pos::centre(o.pos.cell(), 0);
            if (o.pos.x, o.pos.y) == (cc.x, cc.y) {
                // spec: S-0208 claim 13 (exactly on the centre: the body facing)
                eighth(o.body.value(t))
            } else {
                // spec: S-0208 claim 13 (d₀: the facing from the cell centre to the unit's position)
                eighth(direction(
                    i64::from(o.pos.x - cc.x),
                    i64::from(o.pos.y - cc.y),
                    q.table_direction,
                ))
            }
        }
    };
    // spec: S-0208 claim 12 (d = (d₀ + j − 2) mod 8)
    let d = (d0 + j - 2).rem_euclid(8);
    crate::regions::ensure(w);
    let searched = if req.threat.is_none() {
        // spec: S-0208 claim 13 (the nearby-cell search of S-0212 claim 25, scatter use)
        path::nearby(
            w,
            centre,
            NearbyUse::Scatter {
                class: u.class,
                deck: o.layer == Layer::Deck,
            },
        )
    } else {
        None
    };
    let dest = searched.or_else(|| neighbours_choice(w, id, centre, d));
    let Some(c) = dest else {
        return true;
    };
    // spec: S-0417 claim 17 (During the walk-off: a destination from S-0212 claim 25's search leaves the current behaviour as it was; one from S-0208 claim 13's fall-back to the neighbours queues move)
    let keep_behaviour = how == Assign::Resting && searched.is_some();
    if let Some(ob) = w.obj_mut(id) {
        let before = ob.dest;
        ob.dest = Some(Target::Cell(c));
        if !keeps {
            // spec: S-0208 claim 6 (r46: a new destination drops the stored path; r52: except where the paragraph on enter keeps it)
            ob.path.steps.clear();
        }
        if !keep_behaviour {
            // spec: S-0208 claim 12 (the unit then moves there as under a move order)
            ob.beh.queue(Some(Behaviour::Move));
        }
        // spec: S-0107 claim 15 (an accepted scatter that assigns a destination: the destination-driven get-up)
        crate::infantry::destination_assigned(w, id, before);
    }
    if keep_behaviour
        && crate::teleport::is_teleporter(w, id)
        && w.obj(id).is_some_and(|o| o.dest.is_some())
    {
        // spec: S-0417 claim 17 (During the walk-off: a teleporting unit given a destination from the search gets one movement update at once and jumps, with no wait, to the landing point the destination fixed, S-0208 claim 19 and S-0207 claim 20 d; with no landing point fixed the destination was removed and it does not jump)
        crate::teleport::tick(w, id);
    }
    if free && searched.is_some() && w.obj(id).is_some_and(|o| o.dest.is_some()) {
        // spec: S-0208 claim 6 (r46, Destinations given during a unit's update: a destination from the nearby-cell search gives such a walker one more locomotion at once, after the scatter's own draw; a destination chosen among the eight neighbours, claim 12 or claim 13's fall-back, gives none)
        crate::walk::extra_locomotion(w, id);
    }
    true
}

/// S-0417 claim 17 at the end of an infantry unit's turn: a unit in play on
/// the map, whose current behaviour is *guard* or *area guard* and whose
/// cell holds a building that counts, is sent a forced, move-anywhere
/// scatter request with no threat point. The callers leave out the turns
/// that end early (being erased, a warp's leaving stage and its earlier
/// update boundaries); no tunnel is built.
pub fn resting_turn(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id) else { return };
    if o.kind != Kind::Infantry || crate::combat::damage::dead(o) {
        return;
    }
    // spec: S-0417 claim 17 (in play on the map: placed in the world, not inside a building or riding a transport, glossary *in play* and *on the map*)
    if !o.on_map || o.structure.inside.is_some() || crate::transport::riding(w, id).is_some() {
        return;
    }
    // spec: S-0417 claim 17 (its current behaviour is guard or area guard)
    if !matches!(o.beh.current, Some(Behaviour::Guard | Behaviour::AreaGuard)) {
        return;
    }
    // spec: S-0417 claim 17 (the cell under its position on the ground plane, whatever its height)
    let c = o.pos.cell();
    let master = crate::econ::slaves::master_of(w, id);
    let counts = w.cells.list(c, Layer::Ground).iter().any(|&b| {
        w.obj(b).is_some_and(|x| x.kind == Kind::Building) && building_counts(w, b, master, c)
    });
    if !counts {
        return;
    }
    // spec: S-0417 claim 17 (no threat point, forced, move-anywhere)
    let req = ScatterRequest {
        threat: None,
        forced: true,
        anywhere: true,
    };
    request_resting(w, id, req);
}

/// Whether building `b` in cell `c` counts for S-0417 claim 17, for a unit
/// whose master is `master`.
fn building_counts(w: &World, b: ObjectId, master: Option<ObjectId>, c: CellXY) -> bool {
    // spec: S-0417 claim 17 (S-0209 claim 4's rows WP7, WP10 and WP11, whose value is none, leave a building out)
    if occupancy::building_gives_none(w, b) {
        return false;
    }
    let fence = w.obj(b).is_some_and(|o| {
        w.rules()
            .buildings
            .get(o.type_index as usize)
            .is_some_and(|t| t.building.laser_fence)
    });
    // spec: S-0417 claim 17 and Open question 14 (row WP8: a laser-fence section in the states in which it is down and shows no beam, design); section states are not modelled, so every section counts as down, as for S-0613 claim 10
    if fence {
        return false;
    }
    // spec: S-0417 claim 17 (row WP2: not for an enslaved unit when the building is its master and the cell is the master's unloading cell)
    !(master == Some(b) && crate::econ::slaves::unloading_cell(w, b) == Some(c))
}

/// The cell of an infantry unit's scatter reference point (S-0208 Terms):
/// its walking target while it has one, else its own position.
pub fn infantry_centre_cell(w: &World, id: ObjectId) -> Option<CellXY> {
    let o = w.obj(id)?;
    // spec: S-0208 Terms (scatter centre cell: the cell of the scatter reference point, the walking target while there is one, else the unit's own position; never a farther destination)
    Some(
        o.walk
            .as_ref()
            .and_then(|s| s.target)
            .map_or(o.pos.cell(), |p| p.cell()),
    )
}

/// A vehicle's scatter centre cell (S-0207 Terms): its heading point's cell,
/// else its own.
pub fn vehicle_centre_cell(w: &World, id: ObjectId) -> Option<CellXY> {
    let o = w.obj(id)?;
    // spec: S-0207 Terms (vehicle scatter centre cell: the heading point's cell, else its own) and claim 16 (a drive vehicle's heading point is its track's target from the tick the track begins until it ends; never the move destination as such). The heading point of other locomotors and the tunnel exit have no state in M3.
    Some(
        o.drive
            .as_ref()
            .and_then(|d| d.track.as_ref())
            .map(|t| t.target_cell)
            // spec: S-0207 Terms (the vehicle scatter centre cell: a hovering vehicle's heading point's cell, S-0503 Terms)
            .or_else(|| crate::hover::heading_cell(w, id))
            .unwrap_or(o.pos.cell()),
    )
}

/// A vehicle, whatever its locomotor, receives a scatter request (S-0207
/// claim 16). Returns whether it accepted.
pub fn vehicle_request(w: &mut World, id: ObjectId, req: ScatterRequest) -> bool {
    let t = w.tick;
    let Some(o) = w.obj(id).cloned() else {
        return false;
    };
    let r = w.rules_arc();
    let Some(u) = r.unit(o.kind, o.type_index).cloned() else {
        return false;
    };
    let q = r.quirks;
    // spec: S-0207 claim 16 (a): the current behaviour, or the queued one when there is no current one
    let slot = o.beh.current.or(o.beh.queued);
    let a = matches!(
        slot,
        Some(Behaviour::Sleep | Behaviour::Sticky | Behaviour::Unload)
    ) || u.train;
    // spec: S-0207 claim 16 (b): the teleporting locomotor, read as the current one (scenario D17 d, the active movement kind; claim 28, a chrono miner driving on the docking cell accepts a sale's request)
    let b = crate::teleport::is_teleporter(w, id);
    // spec: S-0207 claim 16 (c): not forced and the current behaviour lacks the scatter property (S-0215 claim 19)
    let c = !req.forced && !o.beh.seen().is_some_and(|x| r.behaviour(x).scatter);
    // spec: S-0207 claim 16 (d): its body is turning
    let d = o.body.turning(t);
    // spec: S-0207 claim 16 (e): a destination and a request that is not move-anywhere
    let e = o.dest.is_some() && !req.anywhere;
    // spec: S-0207 claim 16 (f): deployed, deploying or undeploying have no state yet
    let f = false;
    // spec: S-0207 claim 16 (g): a locomotor switched off. A hovering vehicle's only while it is not powered in S-0503's sense, switched off and down at a hover height of 0 (S-0405 claim 20 and Changes for other specs, revision 36; S-0503 Terms and claim 10). Any other vehicle's locomotor is switched off by the disabling of S-0405 claims 13 and 20 and switched on again at enabling; no other switch-off of a vehicle locomotor has state here.
    let g = if crate::hover::is_hover(w, id) {
        !crate::hover::powered(w, id)
    } else {
        crate::econ::power::unit_disabled(w, id)
    };
    if a || b || c || d || e || f || g {
        return false;
    }
    // spec: S-0207 Terms (vehicle scatter centre cell)
    let centre = vehicle_centre_cell(w, id).unwrap_or(o.pos.cell());
    let Some(threat) = req.threat else {
        // spec: S-0207 claim 16 (no threat point: no draw; the nearby-cell search's cell; behaviours unchanged)
        crate::regions::ensure(w);
        let dest = path::nearby(
            w,
            centre,
            NearbyUse::Scatter {
                class: u.class,
                deck: o.layer == Layer::Deck,
            },
        );
        if let Some(cell) = dest
            && let Some(ob) = w.obj_mut(id)
        {
            ob.dest = Some(Target::Cell(cell));
            ob.path.steps.clear();
            // spec: S-0314 claim 22 (the destination of a scatter request ends a settled vehicle's being settled while a depot occupies its cell)
            crate::depot::destination_given(w, id, o.dest != Some(Target::Cell(cell)));
            // spec: S-0207 claim 28, A sale of the refinery (the request's destination is an assignment: claim 27's dock test and claim 28's rule act on it)
            crate::refresh::adopted(w, id, o.dest);
        }
        return true;
    };
    // spec: S-0207 claim 16 (h): the current behaviour has the paralysed property (S-0215 claim 17)
    let h = o.beh.seen().is_some_and(|x| r.behaviour(x).paralysed);
    // spec: S-0207 claim 16 (i): unloading passengers has no state in M3
    let i = false;
    // spec: S-0207 claim 16 (j): a destination and its turret turning
    let j = o.dest.is_some() && o.turret.turning(t);
    if h || i || j {
        return false;
    }
    if o.target.is_some() && !req.forced {
        // spec: S-0207 claim 16 (a target and a request not forced: one draw over 1 to 4, refused unless 1)
        let k = w.rng.ranged(1, 4, q.ranged_draw_half_span);
        if k != 1 {
            return false;
        }
    }
    // spec: S-0207 claim 16 (the direction draw j over 0 to 2, after the draw over 1 to 4)
    let jj = w.rng.ranged(0, 2, q.ranged_draw_half_span);
    // spec: S-0207 claim 16 (d₀: the facing from the threat point to its own position, rounded, S-0208 F6)
    let d0 = eighth(direction(
        i64::from(o.pos.x - threat.x),
        i64::from(o.pos.y - threat.y),
        q.table_direction,
    ));
    // spec: S-0207 claim 16 (d = (d₀ + j − 1) mod 8, the residue from 0 to 7)
    let dir = (d0 + jj - 1).rem_euclid(8);
    let reference = occupancy::reference_level(w, id);
    let mut first_q = None;
    let mut chosen = None;
    // spec: S-0207 claim 16 (the eight neighbours in the step directions d, d + 1, …, d + 7)
    for k in 0..8 {
        let (dx, dy) = STEPS[((dir + k).rem_euclid(8)) as usize];
        let cell = CellXY::new(centre.x + dx, centre.y + dy);
        // spec: S-0207 claim 16 (qualifies: inside the usable area and the vehicle's entry outcome clear)
        if !w.cells.usable(cell)
            || occupancy::vehicle_outcome(w, id, cell, reference) != Outcome::Clear
        {
            continue;
        }
        let high = w.cells.idx(cell).is_some_and(|x| w.cells.deck[x]);
        // spec: S-0207 claim 16 (the first that passes the screen-cover check and is not a high-bridge cell)
        if screen_cover_passes(w, cell) && !high {
            chosen = Some(cell);
            break;
        }
        first_q.get_or_insert(cell);
    }
    if let Some(cell) = chosen.or(first_q)
        && let Some(ob) = w.obj_mut(id)
    {
        // spec: S-0207 claim 16 (it queues move and takes that destination)
        ob.dest = Some(Target::Cell(cell));
        ob.path.steps.clear();
        ob.beh.queue(Some(Behaviour::Move));
        // spec: S-0314 claim 22 (the destination of a scatter request ends a settled vehicle's being settled while a depot occupies its cell)
        crate::depot::destination_given(w, id, o.dest != Some(Target::Cell(cell)));
        // spec: S-0207 claim 28 (the request's destination is an assignment: claim 27's dock test and claim 28's rule act on it)
        crate::refresh::adopted(w, id, o.dest);
    }
    true
}

/// The choice among the eight neighbours of claim 12.
fn neighbours_choice(w: &World, id: ObjectId, centre: CellXY, d: i32) -> Option<CellXY> {
    let reference = occupancy::reference_level(w, id);
    let mut first_q = None;
    // spec: S-0208 claim 12 (directions taken mod 8)
    for k in 0..8 {
        // spec: S-0208 claim 12 (the eight neighbours, from direction d)
        let (dx, dy) = STEPS[((d + k).rem_euclid(8)) as usize];
        let c = CellXY::new(centre.x + dx, centre.y + dy);
        // spec: S-0208 claim 12 (qualifies: inside the usable area and enterable)
        let qualifies =
            w.cells.usable(c) && occupancy::infantry_outcome(w, id, c, reference) == Outcome::Clear;
        if !qualifies {
            continue;
        }
        let high = w.cells.idx(c).is_some_and(|i| w.cells.deck[i]);
        // spec: S-0208 claim 12 (the first that passes the screen-cover check and is not a high-bridge cell)
        if screen_cover_passes(w, c) && !high {
            return Some(c);
        }
        first_q.get_or_insert(c);
    }
    first_q
}

/// A cell-wide scatter request (S-0209 claim 11): each object listed in the
/// cell layer, in listing order, receives it under the conditions of the
/// claim, judged when the request is sent.
pub fn cell_request(w: &mut World, c: CellXY, l: Layer, req: ScatterRequest) {
    let listed: Vec<ObjectId> = w.cells.list(c, l).to_vec();
    let r = w.rules_arc();
    // spec: S-0209 claim 11 (a unit or building of elite rank listed in the cell layer at the time of the request)
    let elite_listed = listed.iter().any(|&x| {
        w.obj(x).is_some_and(|o| {
            matches!(
                o.kind,
                Kind::Infantry | Kind::Vehicle | Kind::Aircraft | Kind::Building
            )
        }) && crate::combat::veterancy::rank(w, x) == crate::combat::veterancy::Rank::Elite
    });
    for id in listed {
        let Some(o) = w.obj(id) else { continue };
        let skill = o.owner.map_or(0, |h| w.houses[usize::from(h.0)].skill);
        // spec: S-0209 claim 11 (move-anywhere, an elite occupant, the player-scatter rule, the ability, or the skill threshold)
        let receives = req.anywhere
            || elite_listed
            || r.general.player_scatter
            || crate::combat::veterancy::has_ability(w, id, crate::combat::data::Ability::Scatter)
            || skill >= r.general.iq_scatter;
        if receives {
            request(w, id, req);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // spec: S-0208 F6 (rounding a facing to a step direction, halves clockwise)
    #[test]
    fn eighths() {
        assert_eq!(eighth(0), 0);
        assert_eq!(eighth(4095), 0);
        assert_eq!(eighth(4096), 1);
        assert_eq!(eighth(16383), 2);
        assert_eq!(eighth(61440), 0);
    }
}
