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
/// Returns whether it accepted.
pub fn request(w: &mut World, id: ObjectId, req: ScatterRequest) -> bool {
    let t = w.tick;
    let Some(o) = w.obj(id).cloned() else {
        return false;
    };
    if o.kind != Kind::Infantry {
        // Q-0019: vehicle scatter acceptance has no spec; vehicles refuse.
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
    // spec: S-0208 claim 11 (permission: player-scatter rule, the scatter ability, or move-anywhere; our engine has no computer owners or teams)
    let permission = r.general.player_scatter || req.anywhere;
    let scatter_prop = o.beh.current.is_some_and(|b| r.behaviour(b).scatter);
    // spec: S-0208 claim 11 (first case: a sequence that is not undeploy and not interruptible)
    let first = seq.is_some_and(|q| q != Seq::Undeploy && !q.table().0);
    // spec: S-0208 claim 11 (second case: deploying, human owner, not both forced and move-anywhere)
    let second = seq.is_some_and(Seq::is_deploy) && !(req.forced && req.anywhere);
    // spec: S-0208 claim 11 (third case: not effectively forced and not a permitted fraidycat in a scatter behaviour)
    let third = !eff_forced && (!u.fraidycat || !scatter_prop || !permission);
    if first || second || third {
        return false;
    }
    let q = r.quirks;
    // spec: S-0208 claims 12 and 13 (one ranged draw j over 0 … 4)
    let j = w.rng.ranged(0, 4, q.ranged_draw_half_span);
    // spec: S-0208 Terms (scatter centre cell: the locomotor destination's cell, else the unit's own)
    let centre = path::locomotor_destination(w, id).map_or(o.pos.cell(), |p| p.cell());
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
            ob.dest = Some(Target::Cell(c));
            ob.path.steps.clear();
            ob.beh.queue(Some(Behaviour::Move));
        }
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
    for id in listed {
        let Some(o) = w.obj(id) else { continue };
        let skill = o.owner.map_or(0, |h| w.houses[usize::from(h.0)].skill);
        // spec: S-0209 claim 11 (move-anywhere, an elite occupant, the player-scatter rule, the ability, or the skill threshold)
        let receives = req.anywhere || r.general.player_scatter || skill >= r.general.iq_scatter;
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
