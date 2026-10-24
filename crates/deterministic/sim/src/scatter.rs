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
    let accepted = answer(w, id, req);
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

/// [`request`] without the trace.
fn answer(w: &mut World, id: ObjectId, req: ScatterRequest) -> bool {
    let t = w.tick;
    let Some(o) = w.obj(id).cloned() else {
        return false;
    };
    if o.kind == Kind::Vehicle {
        return vehicle_request(w, id, req);
    }
    if o.kind != Kind::Infantry {
        // The aircraft specs own how an aircraft answers a scatter request.
        return false;
    }
    let r = w.rules_arc();
    let Some(u) = r.unit(o.kind, o.type_index).cloned() else {
        return false;
    };
    let seq = o.infantry.as_ref().and_then(|s| s.seq);
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
    let dest = if req.threat.is_none() {
        // spec: S-0208 claim 13 (the nearby-cell search of S-0212 claim 25, scatter use)
        path::nearby(
            w,
            centre,
            NearbyUse::Scatter {
                class: u.class,
                deck: o.layer == Layer::Deck,
            },
        )
        .or_else(|| neighbours_choice(w, id, centre, d))
    } else {
        neighbours_choice(w, id, centre, d)
    };
    if let Some(c) = dest {
        // spec: S-0208 claim 12 (the unit then moves there as under a move order)
        if let Some(ob) = w.obj_mut(id) {
            let before = ob.dest;
            ob.dest = Some(Target::Cell(c));
            ob.path.steps.clear();
            ob.beh.queue(Some(Behaviour::Move));
            // spec: S-0107 claim 15 (an accepted scatter that assigns a destination: the destination-driven get-up)
            crate::infantry::destination_assigned(w, id, before);
        }
    }
    true
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
    // spec: S-0207 claim 16 (b): the teleporting locomotor
    let b = u.type_locomotor() == glasnost_rules::sim::Locomotor::Teleport;
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
    // spec: S-0207 Terms (vehicle scatter centre cell: the heading point's cell, else its own) and claim 16 (a drive vehicle's heading point is its track's target from the tick the track begins until it ends; never the move destination as such). The heading point of other locomotors and the tunnel exit have no state in M3.
    let centre = o
        .drive
        .as_ref()
        .and_then(|d| d.track.as_ref())
        .map(|t| t.target_cell)
        // spec: S-0207 Terms (the vehicle scatter centre cell: a hovering vehicle's heading point's cell, S-0503 Terms)
        .or_else(|| crate::hover::heading_cell(w, id))
        .unwrap_or(o.pos.cell());
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
