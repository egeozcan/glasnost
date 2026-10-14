//! The teleporting locomotor's own move (S-0207 claim 20, F9): the
//! landing point fixed when the destination is given, the jump on the next
//! movement update, and phasing in.
//!
//! Vehicles and infantry move with it; an infantry unit's landing point is
//! a standing position (S-0208 claims 19 and 20). An aircraft holds no
//! vehicle-present mark to move (request Q-0232).

use glasnost_map::CellXY;
use glasnost_num::geom::root_floor;
use serde::{Deserialize, Serialize};

use crate::occupancy;
use crate::path::{self, NearbySettings, NearbyUse};
use crate::rules::{Behaviour, Kind, Locomotor};
use crate::walk;
use crate::world::{Layer, ObjectId, Pos, Target, World};

/// Terrain level at or below which a teleporting infantry unit's landing
/// entry check is clear without a test.
// spec: S-0208 claim 19 (level −6 or lower: the check's initial reference level −1 is more than four levels above)
const NEGATIVE_ENTRY_LEVEL: i32 = -6;

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
    /// The record layer of an infantry unit's reserved standing position
    /// at `mark` (S-0208 claim 19).
    #[serde(default)]
    pub mark_layer: Layer,
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
    if o.kind == Kind::Infantry {
        take_infantry_destination(w, id, dest, st);
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
                landing_layer(w, old)
            } else {
                o.layer
            };
            w.cells.claim_vehicle(old, old_layer, false);
            // The mark is held in the record of the layer the object is listed in after the jump (interim, request Q-0240).
            w.cells.claim_vehicle(l.cell(), landing_layer(w, l.cell()), true);
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

/// An infantry unit takes destination `dest` (S-0208 claim 19): the
/// landing point is a standing position, which the unit reserves in place
/// of a vehicle-present mark.
fn take_infantry_destination(w: &mut World, id: ObjectId, dest: Target, mut st: TeleportState) {
    let Some(o) = w.obj(id).cloned() else { return };
    let oq = crate::objects::occ_quirks(w);
    let top = w.rules().quirks.top_position_counts_as_centre;
    // The cell and record layer of the unit's present entry in the occupancy record.
    let (old, old_layer) = match st.mark {
        Some(c) => (c, st.mark_layer),
        None => (o.pos.cell(), o.layer),
    };
    match infantry_landing(w, id, dest) {
        Some((l, layer)) => {
            // spec: S-0208 claim 19 (an infantry unit reserves its chosen standing position, not a vehicle-present mark) and S-0209 claim 1 (the old entry cleared, the new one marked at once)
            w.cells.clear_infantry(old, old_layer, id, oq);
            let mark = walk::position_class(within(l).0, within(l).1, top);
            if let Some(h) = o.owner {
                w.cells.mark_infantry(l.cell(), layer, mark, id, h);
            }
            st.mark = Some(l.cell());
            st.mark_layer = layer;
            st.landing = Some(l);
            st.for_dest = Some(dest);
            set_state(w, id, st);
        }
        None => {
            // spec: S-0208 claim 19 (no landing point: the destination is removed and the unit does not jump)
            if st.mark.is_some() {
                // A landing reserved for an earlier destination is given up and the unit's
                // entry goes back to where it stands (interim, request Q-0235).
                w.cells.clear_infantry(old, old_layer, id, oq);
                let mark = walk::position_class(within(o.pos).0, within(o.pos).1, top);
                if let Some(h) = o.owner {
                    w.cells.mark_infantry(o.pos.cell(), o.layer, mark, id, h);
                }
            }
            set_state(w, id, TeleportState::default());
            drop_destination(w, id);
        }
    }
}

/// A point's position within its cell.
fn within(p: Pos) -> (i32, i32) {
    (p.x.rem_euclid(256), p.y.rem_euclid(256)) // spec: S-0204 claim 2 (the position within the cell, 0 … 255 per axis)
}

/// The cell of an object's heading point for S-0208 claim 19's search: its
/// next movement point, or its own position when it has none.
pub(crate) fn heading_cell(w: &World, x: ObjectId) -> Option<CellXY> {
    let o = w.obj(x)?;
    // spec: S-0207 Terms (a drive vehicle's heading point: its track's target; a teleporting object has none, S-0208 claim 19 its own position)
    let track = o
        .drive
        .as_ref()
        .and_then(|d| d.track.as_ref())
        .map(|t| t.target_cell);
    // spec: S-0208 claim 19 (a walker's next movement point: its walking target)
    let walking = o.walk.as_ref().and_then(|s| s.target).map(|p| p.cell());
    Some(track.or(walking).unwrap_or(o.pos.cell()))
}

/// The landing point of an infantry unit for destination `dest` (S-0208
/// claim 19) and the record layer of its standing position, or `None`
/// when no landing point is found.
fn infantry_landing(w: &mut World, id: ObjectId, dest: Target) -> Option<(Pos, Layer)> {
    let o = w.obj(id)?.clone();
    let u = w.rules().unit(o.kind, o.type_index)?.clone();
    // spec: S-0208 claim 19 (P: the destination point supplied to the locomotor, C its cell)
    let p = match dest {
        Target::Cell(_) => path::heading_point(w, id)?,
        Target::Object(x) => w.obj(x)?.pos,
    };
    let c = p.cell();
    let high = |w: &World, c: CellXY| w.cells.idx(c).is_some_and(|i| w.cells.deck[i]);
    let h = w.surface_at(p.x, p.y);
    // spec: S-0208 claim 19 (standing deck mode: C carries a high bridge and the present height exceeds H(P) by more than 312)
    let deck = high(w, c) && o.pos.z > h.wrapping_add(DECK_MODE_ABOVE);
    // The record a standing position is chosen in and reserved: the deck's on a high-bridge cell, as for the entry check (interim, request Q-0235).
    let layer_of = |w: &World, c: CellXY| {
        if high(w, c) {
            Layer::Deck
        } else {
            Layer::Ground
        }
    };
    // spec: S-0208 claim 19 (occupants ignored in capture, sabotage, enter or eaten with a vehicle or aircraft in C, or the building occupying C, as destination)
    let ignore = matches!(
        o.beh.seen(),
        Some(Behaviour::Capture | Behaviour::Sabotage | Behaviour::Enter | Behaviour::Eaten)
    ) && match dest {
        Target::Object(x) => w.obj(x).is_some_and(|m| match m.kind {
            Kind::Vehicle | Kind::Aircraft => m.pos.cell() == c,
            Kind::Building => w.cells.list(c, Layer::Ground).contains(&x),
            _ => false,
        }),
        Target::Cell(_) => false,
    };
    let point = |w: &World, c: CellXY, pos: u8| {
        let (su, sv) = walk::standing_point(pos);
        let (x, y) = (256 * c.x + su, 256 * c.y + sv); // spec: S-0208 table P (a standing position within the cell)
        // spec: S-0208 claims 1 and 19 (the surface height there, 416 above in standing deck mode)
        let z = w.surface_at(x, y) + if deck { DECK_HEIGHT } else { 0 };
        Pos::new(x, y, z)
    };
    // spec: S-0208 claim 19 (the desired class: the position class of P)
    let class = walk::position_class(within(p).0, within(p).1, true);
    let first = walk::choose_position(w, c, layer_of(w, c), class, ignore);
    if let Some(pos) = first
        && entry_clear(w, id, c, high(w, c))
    {
        return Some((point(w, c, pos), layer_of(w, c)));
    }
    // spec: S-0208 claim 19 (a refusal: only an object destination enables a nearby-cell search)
    let Target::Object(x) = dest else {
        return None;
    };
    let centre = heading_cell(w, x)?;
    crate::regions::ensure(w);
    // spec: S-0208 claim 19 (reach: the unit's own heading-point cell, its own position for the teleporting locomotor, with its present deck status)
    let reach = crate::regions::reach_read(w, o.pos.cell(), u.zone, o.layer == Layer::Deck);
    // spec: S-0208 claim 19 (the search: the heading-point cell, own class and unsimplified zone, deck mode on a high-bridge centre, height condition, no further test, no no-overlay, bridges allowed, no reference cell)
    let f = path::nearby(
        w,
        centre,
        NearbyUse::Settings(NearbySettings {
            class: u.class,
            zone: u.zone,
            reach: Some(reach),
            deck: high(w, centre),
            height: true,
            reference: None,
            no_bridges: false,
            no_overlay: false,
        }),
    )?;
    // spec: S-0208 claim 19 (a second standing-position choice with the found cell's centre as desired point; no second entry test)
    let pos = walk::choose_position(w, f, layer_of(w, f), crate::cells::position::CENTRE, ignore)?;
    Some((point(w, f, pos), layer_of(w, f)))
}

/// S-0208 claim 19's entry check for an infantry landing on cell `c`.
fn entry_clear(w: &World, id: ObjectId, c: CellXY, high: bool) -> bool {
    let level = w.cells.cell(c).map_or(0, |cell| i32::from(cell.level));
    // spec: S-0208 claim 19 and Edge cases (a cell at level −6 or lower is clear without a test; quirks.teleport_infantry_negative_level_entry)
    if w.rules().quirks.teleport_infantry_negative_level_entry && level <= NEGATIVE_ENTRY_LEVEL {
        return true;
    }
    // spec: S-0208 claim 19 (S-0209's infantry entry outcome on the deck's objects and record on a high-bridge cell, the ground's otherwise; no connecting-step height restriction)
    let reference = level + if high { DECK_LEVELS } else { 0 };
    occupancy::infantry_outcome(w, id, c, reference) == occupancy::Outcome::Clear
}

/// Placement-time clearing (S-0208 claim 20): when the placement fall-back
/// supplies a destination to a teleporting infantry unit, the destination
/// cell's ground occupants get a cell-wide scatter request before the
/// landing point is chosen.
pub fn placement_destination(w: &mut World, id: ObjectId) {
    if !is_teleporter(w, id) {
        return;
    }
    let Some(o) = w.obj(id).cloned() else { return };
    let for_dest = o.teleport.as_ref().and_then(|t| t.for_dest);
    if o.kind != Kind::Infantry || o.dest.is_none() || o.dest == for_dest {
        return;
    }
    if let Some(c) = path::goal_cell(w, id) {
        // spec: S-0208 claim 20 (a cell-wide request to the destination cell's ground occupants: no threat point, forced, move-anywhere; no owner test)
        crate::scatter::cell_request(
            w,
            c,
            Layer::Ground,
            crate::scatter::ScatterRequest {
                threat: None,
                forced: true,
                anywhere: true,
            },
        );
    }
    take_destination(w, id);
}

/// The cell layer an object lands in on cell `c` (S-0207 claim 20 e): the
/// bridge deck on a high-bridge cell, the ground elsewhere.
fn landing_layer(w: &World, c: CellXY) -> Layer {
    if w.cells.idx(c).is_some_and(|i| w.cells.deck[i]) {
        Layer::Deck
    } else {
        Layer::Ground
    }
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
    // spec: S-0207 claim 20 (e) (every object that has it as its target drops it)
    // spec: S-0215 claim 10 (the target's own jump: an overridden attacker reverts first, then drops the jumper if it is still its target; a restored other target is kept)
    // The erasing-weapon exception is a seam: no erasing weapon is modelled.
    let ids: Vec<ObjectId> = w.objects.keys().copied().collect();
    for x in ids {
        let Some(ob) = w.obj_mut(x) else { continue };
        if ob.target != Some(Target::Object(id)) {
            continue;
        }
        let mut restored = None;
        if ob.beh.suspended.is_some()
            && let Some((t, d)) = ob.beh.revert()
        {
            ob.target = t;
            if ob.kind.is_mobile() {
                restored = Some(ob.dest);
                ob.dest = d;
            }
        }
        if ob.target == Some(Target::Object(id)) {
            ob.target = None;
        }
        ob.combat.burst = 0; // spec: S-0301 claim 4 (a cleared target resets the burst counter)
        if let Some(before) = restored {
            // spec: S-0107 claim 15 (a destination brought back is assigned: the destination-driven get-up)
            crate::infantry::destination_assigned(w, x, before);
        }
    }
    // spec: S-0306 claim 17 (a projectile whose target becomes untargetable by teleporting: the cell the target stood in when not airborne, else nothing)
    let airborne = crate::combat::weapon::airborne_by_height(w, id);
    for p in w.projectiles.values_mut() {
        if p.target == Some(Target::Object(id)) {
            p.target = (!airborne).then_some(Target::Cell(o.pos.cell()));
        }
    }
    // spec: S-0207 claim 20 (e) (a warp animation at the old point; its creation and start-event draws as S-0202 claims 10 and 15 give them for an animation created in play: interim, request Q-0232; the sounds take no draw)
    if let Some(warp) = r.warp_out.clone() {
        crate::load::anim_created_at(w, &warp, Some((o.pos.x, o.pos.y, o.pos.z)));
    }
    let old = o.pos.cell();
    let new = l.cell();
    let high = w.cells.idx(new).is_some_and(|i| w.cells.deck[i]);
    // spec: S-0207 claim 20 (e) (on a high-bridge cell at H + 416, elsewhere at H; it keeps its facing)
    let z = w
        .surface_at(l.x, l.y)
        .wrapping_add(if high { DECK_HEIGHT } else { 0 });
    // spec: S-0207 claim 20 (e) (its cell layer is the bridge deck on a high-bridge cell, ground elsewhere, for every kind)
    let layer = landing_layer(w, new);
    w.cells.leave(old, o.layer, id);
    if let Some(ob) = w.obj_mut(id) {
        ob.pos = Pos::new(l.x, l.y, z);
        ob.layer = layer;
        // spec: S-0207 claim 20 (e) (it stops moving and its destination is removed; its behaviour falls back to rest, S-0215 claim 13)
        ob.dest = None;
        ob.path.steps.clear();
    }
    w.cells.enter(new, layer, id, false);
    // spec: S-0207 claim 20 (e) (entering L's cell has the arrival effects of the object's own kind) and S-0211 claim 5 (both cells are recomputed, for a walker as for a vehicle)
    crate::load::recalc_cells(w, &[old, new]);
    if o.kind == Kind::Vehicle {
        // spec: S-0207 claim 20 (e) (a vehicle's: those of reaching a track target in a new cell, crushing among them, S-0210 claim 9)
        crate::crush::entered_cell(w, id, new);
    }
    // spec: S-0207 claim 20 (e) (a second warp animation at L; crates and parasites have no state in our engine)
    if let Some(warp) = r.warp_out.clone() {
        crate::load::anim_created_at(w, &warp, Some((l.x, l.y, z)));
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
        .is_some_and(|o| o.target.is_none() && o.kind.is_mobile())
    {
        crate::process::fall_back(w, id, false);
    }
}
