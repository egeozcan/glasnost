//! The teleporting locomotor's own move (S-0207 claim 20, F9): the
//! landing point fixed when the destination is given, the jump on the next
//! movement update, and phasing in.
//!
//! In our engine only vehicles move with it: an infantry unit's landing
//! point is open (request Q-0181) and an aircraft holds no vehicle-present
//! mark to move (request Q-0232).

use glasnost_map::CellXY;
use glasnost_num::geom::root_floor;
use serde::{Deserialize, Serialize};

use crate::occupancy;
use crate::path::{self, NearbySettings, NearbyUse};
use crate::rules::{Kind, Locomotor};
use crate::world::{Layer, ObjectId, Pos, Target, World};

/// Height above the surface at the destination from which the landing
/// search is in deck mode, leptons.
// spec: S-0207 Constants (more than 312 leptons, 3 height levels of 104)
const DECK_MODE_ABOVE: i32 = 312;
/// Height added on landing on a high-bridge cell, leptons.
// spec: S-0207 Constants (416 leptons, 4 height levels of 104)
const DECK_HEIGHT: i32 = 416;
/// Levels a deck counts above its cell.
// spec: S-0209 Terms (a deck is 4 levels above its cell)
const DECK_LEVELS: i32 = 4;
/// Targeting-timer threshold at the end of phasing in, ticks.
// spec: S-0207 Constants (more than 10 ticks)
const TIMER_THRESHOLD: i32 = 10;
/// Range of the targeting-timer draw at the end of phasing in, ticks.
// spec: S-0207 Constants (4 to 8, both included)
const TIMER_DRAW: (i32, i32) = (4, 8);

/// The teleporting locomotor's state (S-0207 claim 20).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TeleportState {
    /// The landing point *L*, fixed and not yet jumped to (part a).
    pub landing: Option<Pos>,
    /// The destination *L* was fixed for.
    pub for_dest: Option<Target>,
    /// The cell holding the vehicle-present mark when it is not the
    /// object's own cell (part a: the cell of *L*).
    pub mark: Option<CellXY>,
    /// Phasing in: the jump tick *t*₀ and the delay *D* (part f, F9).
    pub phasing: Option<(i32, i64)>,
}

/// Whether the object moves with the teleporting locomotor in our engine.
pub fn is_teleporter(w: &World, id: ObjectId) -> bool {
    // spec: S-0417 claim 5 (a teleporting vehicle drives temporarily for its factory exit)
    !crate::econ::release::temporary_drive(w, id)
        && w.obj(id)
            .and_then(|o| w.rules().unit(o.kind, o.type_index))
            .is_some_and(|u| u.locomotor == Locomotor::Teleport)
}

/// Whether the object is phasing in (S-0214 Terms, S-0207 claim 20 f).
pub fn phasing_in(w: &World, id: ObjectId) -> bool {
    w.obj(id)
        .and_then(|o| o.teleport.as_ref())
        .is_some_and(|t| t.phasing.is_some())
}

/// The cell holding the object's vehicle-present mark.
pub fn mark_cell(w: &World, id: ObjectId) -> Option<CellXY> {
    let o = w.obj(id)?;
    Some(
        o.teleport
            .as_ref()
            .and_then(|t| t.mark)
            .unwrap_or(o.pos.cell()),
    )
}

/// The locomotor takes the object's destination (claim 20 a): called when
/// a destination is given, and at the movement update for a destination
/// given where no call reached. A changed destination fixes the landing
/// point at once and moves the occupancy mark to its cell; one given while
/// phasing in is dropped. A cleared destination leaves a fixed landing
/// point in place (part d: the jump follows on the next movement update).
pub fn take_destination(w: &mut World, id: ObjectId) {
    if !is_teleporter(w, id) {
        return;
    }
    let Some(o) = w.obj(id).cloned() else { return };
    let mut st = o.teleport.clone().unwrap_or_default();
    if o.dest == st.for_dest {
        return;
    }
    let Some(dest) = o.dest else {
        st.for_dest = None;
        set_state(w, id, st);
        return;
    };
    // spec: S-0207 claim 20 (a) (dropped while phasing in; the electromagnetic pulse, paralysis and being erased have no state in our engine)
    if st.phasing.is_some() {
        st.for_dest = None;
        set_state(w, id, st);
        drop_destination(w, id);
        return;
    }
    let dc = match dest {
        Target::Cell(c) => c,
        Target::Object(x) => match w.obj(x) {
            Some(t) => t.pos.cell(),
            None => return,
        },
    };
    match landing_point(w, id, dc) {
        Some(l) => {
            // spec: S-0207 claim 20 (a) (the vehicle-present mark moves at once from the old point's cell to L's)
            let old = st.mark.unwrap_or(o.pos.cell());
            let old_layer = if st.mark.is_some() {
                Layer::Ground
            } else {
                o.layer
            };
            w.cells.claim_vehicle(old, old_layer, false);
            w.cells.claim_vehicle(l.cell(), Layer::Ground, true);
            st.mark = Some(l.cell());
            st.landing = Some(l);
            st.for_dest = Some(dest);
            set_state(w, id, st);
        }
        None => {
            // spec: S-0207 claim 20 (b) (no cell: L unchanged; when L is still the own position or none was fixed, no jump and the destination is removed)
            if st.landing.is_none_or(|l| l == o.pos) {
                st.for_dest = None;
                set_state(w, id, st);
                drop_destination(w, id);
            } else {
                st.for_dest = Some(dest);
                set_state(w, id, st);
            }
        }
    }
}

/// The landing point of a non-infantry object for destination cell `dc`
/// (claim 20 b), or `None` when the nearby-cell search gives no cell.
fn landing_point(w: &mut World, id: ObjectId, dc: CellXY) -> Option<Pos> {
    let o = w.obj(id)?.clone();
    let u = w.rules().unit(o.kind, o.type_index)?.clone();
    let centre = Pos::centre(dc, 0);
    let h = w.surface_at(centre.x, centre.y);
    let high = w.cells.idx(dc).is_some_and(|i| w.cells.deck[i]);
    // spec: S-0207 claim 20 (b) (deck mode: a high-bridge destination cell and a world height more than 312 above H there)
    let deck = high && o.pos.z > h.wrapping_add(DECK_MODE_ABOVE);
    // The level at which the entry outcome is asked is open (S-0207 Open questions): the destination cell's, its deck's in deck mode (interim, request Q-0232).
    let level =
        w.cells.cell(dc).map_or(0, |c| i32::from(c.level)) + if deck { DECK_LEVELS } else { 0 };
    // spec: S-0207 claim 20 (b) (a clear entry outcome: the centre of the destination cell at height H)
    if occupancy::vehicle_outcome(w, id, dc, level) == occupancy::Outcome::Clear {
        return Some(Pos::centre(dc, h));
    }
    crate::regions::ensure(w);
    // spec: S-0207 claim 20 (b) (the nearby-cell search: centred on the destination cell, the own class and unsimplified zone, the destination cell's reach region, deck mode as above, no height condition, no further test, bridges allowed, no reference cell)
    let reach = crate::regions::reach_read(w, dc, u.zone, deck);
    let c = path::nearby(
        w,
        dc,
        NearbyUse::Settings(NearbySettings {
            class: u.class,
            zone: u.zone,
            reach: Some(reach),
            deck,
            height: false,
            reference: None,
            no_bridges: false,
            no_overlay: false,
        }),
    )?;
    let p = Pos::centre(c, 0);
    // spec: S-0207 claim 20 (b) (L: the chosen cell's centre at the height H there)
    Some(Pos::centre(c, w.surface_at(p.x, p.y)))
}

fn set_state(w: &mut World, id: ObjectId, st: TeleportState) {
    if let Some(ob) = w.obj_mut(id) {
        ob.teleport = Some(st);
    }
}

fn drop_destination(w: &mut World, id: ObjectId) {
    if let Some(ob) = w.obj_mut(id) {
        ob.dest = None;
        ob.path.steps.clear();
    }
}

/// The movement update of a teleporting object (claim 20 d to f). Returns
/// whether it moved.
pub fn tick(w: &mut World, id: ObjectId) -> bool {
    take_destination(w, id);
    let Some(o) = w.obj(id).cloned() else {
        return false;
    };
    let st = o.teleport.clone().unwrap_or_default();
    if let Some((t0, d)) = st.phasing {
        // spec: S-0207 claim 20 (f) (phasing in ends on the first tick t after t₀ with t − t₀ ≥ D)
        let since = i64::from(w.tick.wrapping_sub(t0));
        if since >= 1 && since >= d {
            end_phasing(w, id);
        }
        return false;
    }
    match st.landing {
        Some(l) if l != o.pos => {
            jump(w, id, l);
            true
        }
        _ => {
            // spec: S-0207 claim 20 (d) (L equal to its own position, or none fixed: no jump, the destination is removed)
            if o.dest.is_some() || st.landing.is_some() {
                set_state(w, id, TeleportState::default());
                drop_destination(w, id);
            }
            false
        }
    }
}

/// The jump (claim 20 e) and the start of phasing in (claim 20 f, F9).
fn jump(w: &mut World, id: ObjectId, l: Pos) {
    let Some(o) = w.obj(id).cloned() else { return };
    let tick = w.tick;
    let r = w.rules_arc();
    // spec: S-0207 claim 20 (e) (every object that has it as its target drops it; no revert, S-0215 claim 10)
    let ids: Vec<ObjectId> = w.objects.keys().copied().collect();
    for x in ids {
        if let Some(ob) = w.obj_mut(x)
            && ob.target == Some(Target::Object(id))
        {
            ob.target = None;
            ob.combat.burst = 0; // spec: S-0301 claim 4 (a cleared target resets the burst counter)
        }
    }
    // spec: S-0306 claim 17 (a projectile whose target becomes untargetable by teleporting: the cell the target stood in when not airborne, else nothing)
    let airborne = crate::combat::weapon::airborne_by_height(w, id);
    for p in w.projectiles.values_mut() {
        if p.target == Some(Target::Object(id)) {
            p.target = (!airborne).then_some(Target::Cell(o.pos.cell()));
        }
    }
    // spec: S-0207 claim 20 (e) (a warp animation at the old point; its creation draws as S-0202 claim 10 gives them: interim, request Q-0232; the sounds take no draw)
    if let Some(warp) = r.warp_out.clone() {
        crate::load::anim_creation_draws(w, &warp);
    }
    let old = o.pos.cell();
    let new = l.cell();
    let high = w.cells.idx(new).is_some_and(|i| w.cells.deck[i]);
    // spec: S-0207 claim 20 (e) (on a high-bridge cell at H + 416, elsewhere at H; it stays in the ground layer and keeps its facing)
    let z = l.z.wrapping_add(if high { DECK_HEIGHT } else { 0 });
    w.cells.leave(old, o.layer, id);
    if let Some(ob) = w.obj_mut(id) {
        ob.pos = Pos::new(l.x, l.y, z);
        ob.layer = Layer::Ground;
        // spec: S-0207 claim 20 (e) (it stops moving and its destination is removed; its behaviour falls back to rest, S-0215 claim 13)
        ob.dest = None;
        ob.path.steps.clear();
    }
    w.cells.enter(new, Layer::Ground, id, false);
    // spec: S-0207 claim 20 (e) and S-0211 claim 5 (entering L's cell as a vehicle reaching a track target in a new cell: the cells are recomputed, crushing)
    crate::load::recalc_cells(w, &[old, new]);
    crate::crush::entered_cell(w, id, new);
    // spec: S-0207 claim 20 (e) (a second warp animation at L; crates and parasites have no state in our engine)
    if let Some(warp) = r.warp_out.clone() {
        crate::load::anim_creation_draws(w, &warp);
    }
    let harvester = r.unit(o.kind, o.type_index).is_some_and(|u| u.harvester);
    // spec: S-0207 claim 20 (f) and F9 (a harvester-flag type does not phase in)
    let phasing = (!harvester).then(|| (tick, delay(w, o.pos, l)));
    set_state(
        w,
        id,
        TeleportState {
            phasing,
            ..Default::default()
        },
    );
}

/// The phasing-in delay *D* of F9 for a jump from `from` to `l`.
pub fn delay(w: &World, from: Pos, l: Pos) -> i64 {
    let r = w.rules();
    let g = &r.general;
    let (dx, dy, dz) = (
        i64::from(l.x) - i64::from(from.x),
        i64::from(l.y) - i64::from(from.y),
        i64::from(l.z) - i64::from(from.z),
    );
    // spec: S-0207 F9 (d = ⌊r(Δx² + Δy² + Δz²)⌋, S-0204's table square root; quirks.table_root_distance)
    let n = (dx * dx + dy * dy + dz * dz) as u64;
    let d = root_floor(n, r.quirks.table_root_distance, r.mode) as i64;
    let k = i64::from(g.chrono_distance_factor);
    // spec: S-0207 F9 (trigger on: q = trunc(d / k); off: q = 0) and Edge cases (k = 0 with the trigger on: taken as off)
    let q = if g.chrono_trigger && k != 0 { d / k } else { 0 };
    let m = i64::from(g.chrono_minimum_delay);
    // spec: S-0207 F9 (D = q when max(q, 0) > m, else m; d < g gives m)
    if d < i64::from(g.chrono_range_minimum) {
        m
    } else if q.max(0) > m {
        q
    } else {
        m
    }
}

/// The end of phasing in (claim 20 f).
fn end_phasing(w: &mut World, id: ObjectId) {
    let tick = w.tick;
    let half = w.rules().quirks.ranged_draw_half_span;
    let Some(o) = w.obj(id).cloned() else { return };
    if let Some(ob) = w.obj_mut(id)
        && let Some(t) = ob.teleport.as_mut()
    {
        t.phasing = None;
    }
    if o.target.is_none() && o.targeting_timer.wrapping_sub(tick) > TIMER_THRESHOLD {
        // spec: S-0207 claim 20 (f) (no target and more than 10 ticks left: one draw over 4 to 8 ticks)
        let v = w.rng.ranged(TIMER_DRAW.0, TIMER_DRAW.1, half);
        if let Some(ob) = w.obj_mut(id) {
            ob.targeting_timer = tick.wrapping_add(v);
        }
    }
    // spec: S-0207 claim 20 (f) and S-0309 claim 1 (the teleport search, then the idle behaviour when it sets no target)
    crate::combat::acquire::teleport_search(w, id);
    if w.obj(id)
        .is_some_and(|o| o.target.is_none() && o.kind == Kind::Vehicle)
    {
        crate::process::fall_back(w, id, false);
    }
}
