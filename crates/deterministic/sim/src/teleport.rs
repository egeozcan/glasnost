//! The teleporting locomotor's own move (S-0207 claim 20, F9): the
//! landing point fixed when the destination is given, the jump on the next
//! movement update, and phasing in.
//!
//! Vehicles, infantry and aircraft move with it; an infantry unit's landing
//! point is a standing position (S-0208 claims 19 and 20), and an aircraft
//! moves its object-present mark (S-0207 claim 20 a).

use glasnost_map::CellXY;
use glasnost_num::geom::root_floor;
use serde::{Deserialize, Serialize};

use crate::combat::effects::{Effect, Sound};
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
pub(crate) const DECK_HEIGHT: i32 = 416;
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
    /// object's own cell (part a: the cell of *L*), also after an empty
    /// assignment cancelled the jump to *L* (part h: `landing` is then
    /// `None` and the reservation stays).
    pub mark: Option<CellXY>,
    /// Phasing in: the jump tick *t*₀ and the delay *D* (part f, F9).
    pub phasing: Option<(i32, i64)>,
    /// The record layer of the reservation at `mark`: an infantry unit's
    /// standing position (S-0208 claim 19), a vehicle-present or
    /// object-present mark (S-0207 claim 20 a).
    #[serde(default)]
    pub mark_layer: Layer,
    /// The reservation of the remembered landing point after a completed
    /// jump, its cell and record layer (S-0207 claim 20 a: finishing a jump
    /// does not erase the remembered point; S-0208 claim 19 for infantry).
    #[serde(default)]
    pub held: Option<(CellXY, Layer)>,
    /// The deck record a vehicle or aircraft marked by deck placement
    /// beside its ground reservation, cleared when the next jump removes it
    /// from that cell (S-0207 claim 20 e).
    #[serde(default)]
    pub deck_mark: Option<CellXY>,
}

impl TeleportState {
    /// The state with no landing point, keeping the remembered reservation
    /// and deck mark (S-0207 claim 20 a and e): a pending reservation not
    /// jumped to becomes the held one.
    fn without_landing(&self) -> Self {
        Self {
            held: self.mark.map(|c| (c, self.mark_layer)).or(self.held),
            deck_mark: self.deck_mark,
            ..Default::default()
        }
    }
}

/// Sets or clears the mark a teleporting vehicle (vehicle-present) or
/// aircraft (object-present) holds in a record (S-0207 claim 20 a, S-0209
/// claim 1).
fn reserve(w: &mut World, kind: Kind, c: CellXY, l: Layer, add: bool) {
    if kind == Kind::Aircraft {
        if let Some(rec) = w.cells.occ_mut(c, l) {
            rec.others = if add {
                rec.others.saturating_add(1)
            } else {
                rec.others.saturating_sub(1)
            };
        }
    } else {
        w.cells.claim_vehicle(c, l, add);
    }
}

/// Whether the ground-reservation quirk of a vehicle or aircraft is on.
fn ground_reservation(w: &World, kind: Kind) -> bool {
    let q = &w.rules().quirks;
    match kind {
        // spec: S-0207 Edge cases (quirks.teleport_aircraft_ground_reservation)
        Kind::Aircraft => q.teleport_aircraft_ground_reservation,
        // spec: S-0207 Edge cases (quirks.teleport_vehicle_ground_reservation)
        _ => q.teleport_vehicle_ground_reservation,
    }
}

/// The records a teleporting vehicle or aircraft clears when it is taken
/// off the map, or when a vehicle is handed driving locomotion for its
/// factory exit (S-0207 claims 20 k and 21, S-0209 claim 1). It holds its
/// pending or remembered reservation, any deck mark and, with neither, the
/// mark of its actual cell. Under the respective ground-reservation quirk
/// only the record of its actual cell at its actual layer is cleared; a
/// separate remembered or pending reservation stays marked. With the quirk
/// off every record it holds is cleared.
pub fn removal_marks(w: &World, id: ObjectId) -> Vec<(CellXY, Layer)> {
    let Some(o) = w.obj(id) else {
        return Vec::new();
    };
    let actual = (o.pos.cell(), o.layer);
    // spec: S-0607 claim 23 (a walking stand-in, and a teleporting one with no landing point, hold no mark beyond what the seize left)
    if let Some(m) = crate::lift::stand_in_marks(w, id) {
        return m;
    }
    // spec: S-0209 claim 1 (a flying aircraft holds its object-present mark only while on the ground; a landed aircraft seized with the teleporting stand-in holds that stand-in's reservation instead, S-0607 claim 23)
    if crate::lift::moving_stand_in(w, id) != Some(crate::lift::StandIn::Teleport)
        && let Some(m) = crate::air::removal_marks(w, id)
    {
        return m;
    }
    let Some(t) = o.teleport.as_ref() else {
        return vec![actual];
    };
    let own = t
        .mark
        .map(|c| (c, t.mark_layer))
        .or(t.held)
        .unwrap_or(actual);
    let mut held = vec![own];
    if let Some(d) = t.deck_mark {
        held.push((d, Layer::Deck));
    }
    if ground_reservation(w, o.kind) {
        // spec: S-0207 claim 20 (k) (removal clears the actual cell's occupancy mark at its actual height, not a separate remembered or pending landing reservation)
        // spec: S-0207 claim 21 (installing temporary driving clears neither record; driving departure clears only the actual mark)
        held.retain(|&m| m == actual);
    }
    // spec: S-0207 Edge cases (quirk off: removal clears both the pending reservation and actual occupancy, leaving no orphan reservation)
    held
}

/// Whether the object moves with the teleporting locomotor in our engine.
pub fn is_teleporter(w: &World, id: ObjectId) -> bool {
    // spec: S-0607 claims 22 and 23 (a unit seized with a stand-in moves with that stand-in, not its own locomotor)
    if let Some(k) = crate::lift::moving_stand_in(w, id) {
        return k == crate::lift::StandIn::Teleport;
    }
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
        // spec: S-0513 Terms and claims 39 and 40 (a superweapon warp member phases in from departure until its arrival hold ends)
        || crate::superweapons::warp::phasing(w, id)
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
/// phasing in is dropped. Pending point suppression (claim 27) is tested
/// first: the destination is kept, no landing point is fixed and the
/// suppression is consumed, whatever the object's state. A cleared
/// destination cancels a pending jump but keeps the fixed landing point's
/// reservation (part h).
pub fn take_destination(w: &mut World, id: ObjectId) {
    take_destination_with(w, id, false);
}

/// [`take_destination`] for an assignment treated as new even when
/// unchanged (S-0207 claim 28, the repeated dock test in *enter*).
pub fn take_destination_forced(w: &mut World, id: ObjectId) {
    take_destination_with(w, id, true);
}

fn take_destination_with(w: &mut World, id: ObjectId, forced: bool) {
    if !is_teleporter(w, id) {
        return;
    }
    let Some(o) = w.obj(id).cloned() else { return };
    let mut st = o.teleport.clone().unwrap_or_default();
    // spec: S-0207 claim 27 (pending destination refresh makes an unchanged assignment effective whatever the locomotor)
    if o.dest == st.for_dest && !crate::refresh::pending(w, id) && !forced {
        return;
    }
    // spec: S-0207 claim 28 (a destination that reached no assignment hook, such as a fall-back's queued destination, S-0215 claim 15 row 2: outside a qualified docking assignment driving is installed and receives it)
    if o.dest != st.for_dest && crate::refresh::untaken(w, id, o.dest) {
        return;
    }
    let Some(dest) = o.dest else {
        // spec: S-0207 claim 20 (h) (an empty assignment while a jump is pending cancels the pending jump and the moving state; the fixed landing point stays remembered and its reservation does not move, so the next accepted destination clears it from that point, part a)
        st.landing = None;
        st.for_dest = None;
        set_state(w, id, st);
        return;
    };
    // spec: S-0207 claim 27 (Q-1130 table, second row: suppression already pending skips the landing-point fixing of any teleporting vehicle's nonempty destination, whatever its state; the locomotor is not offered the point, so no claim 20 a refusal runs and no earlier landing point, pending jump or reservation changes; the new order-level destination stays and the suppression is consumed)
    // spec: S-0207 claim 20 (a) (a pending suppression is the only exception to the drop and to the fixing of a landing point)
    if crate::refresh::skip_point(w, id) {
        st.for_dest = Some(dest);
        set_state(w, id, st);
        return;
    }
    // spec: S-0207 claim 20 (a) (dropped while phasing in, held, S-0614 Terms, or being erased by a temporal beam, S-0514 claim 16; the electromagnetic pulse has no state in our engine; a superweapon warp's leaving stage is the warp-pending immobilization of part b below)
    // spec: S-0207 claim 27 (Q-1130 table, third row: with no suppression pending the order-level destination becomes empty and nothing is consumed)
    if st.phasing.is_some() || crate::parasite::held(w, id) || crate::temporal::erased(w, id) {
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
    fix_landing(w, id, dest, dc, st);
}

/// A teleporting stand-in takes the lifter's point as its movement
/// instruction at a magnetic seize (S-0607 claim 23): the landing point is
/// fixed from the cell `dc` of that point as for a destination in that cell
/// (claim 20 a and b), the order-level destination `dest` staying the
/// lifter.
pub fn stand_in_instruction(w: &mut World, id: ObjectId, dest: Target, dc: CellXY) {
    let Some(o) = w.obj(id) else { return };
    let st = o.teleport.clone().unwrap_or_default();
    // spec: S-0607 claims 5 and 23 (the seize has already given up the unit's own mark: fixing the landing point marks the landing cell and clears nothing)
    fix_landing_with(w, id, dest, dc, st, false);
}

/// Fixes a vehicle's or aircraft's landing point for destination `dest` in
/// cell `dc` (claim 20 a and b).
fn fix_landing(w: &mut World, id: ObjectId, dest: Target, dc: CellXY, st: TeleportState) {
    fix_landing_with(w, id, dest, dc, st, true);
}

/// [`fix_landing`]; `release_old`: the old point's mark is cleared as the
/// mark moves to the landing cell.
fn fix_landing_with(
    w: &mut World,
    id: ObjectId,
    dest: Target,
    dc: CellXY,
    mut st: TeleportState,
    release_old: bool,
) {
    let Some(o) = w.obj(id).cloned() else { return };
    let (old, old_layer) = aircraft_mark(w, id);
    // spec: S-0207 claim 20 (a) (the old mark is cleared before part b's entry test, so a vehicle fixing a landing point again in a cell where it holds that reservation does not meet its own reservation there; claim 28, One jump)
    if release_old {
        reserve(w, o.kind, old, old_layer, false);
    }
    // spec: S-0207 claim 20 (b) (revision 34: warp-pending immobilization, Terms, leaves L unchanged, as a search that gives no cell does)
    let found = if crate::superweapons::warp_pending_or_leaving(w, id) {
        None
    } else {
        landing_point(w, id, dc)
    };
    if found.is_none() && release_old {
        // design: S-0207 claim 20 (a) and (b) (no cell leaves L and its reservation unchanged: the old mark cleared for the entry test is marked again)
        reserve(w, o.kind, old, old_layer, true);
    }
    match found {
        Some(l) => {
            // spec: S-0207 claim 20 (a) (a vehicle's vehicle-present mark, an aircraft's object-present mark, moves at once from the old point's record to L's)
            // spec: S-0207 claim 20 (a) and Edge cases (the reservation at L is in the ground record at L's chosen height H, even on a high-bridge cell; quirk off: the eventual landing layer)
            let layer = if ground_reservation(w, o.kind) {
                Layer::Ground
            } else {
                landing_layer(w, l.cell())
            };
            reserve(w, o.kind, l.cell(), layer, true);
            st.mark = Some(l.cell());
            st.mark_layer = layer;
            st.held = None;
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

/// The cell and record layer of the old point's mark of a teleporting
/// vehicle or aircraft (S-0207 claim 20 a): the cell of a landing point
/// fixed and not yet jumped to, in the record it was reserved in; else the
/// remembered landing point's, kept after a completed jump; otherwise its
/// own cell, read in the deck layer only on a high-bridge cell when its
/// height is at least *H* + 416 there, and in the ground layer otherwise.
pub fn aircraft_mark(w: &World, id: ObjectId) -> (CellXY, Layer) {
    let Some(o) = w.obj(id) else {
        return (CellXY::new(0, 0), Layer::Ground);
    };
    if let Some(t) = o.teleport.as_ref() {
        if let Some(c) = t.mark {
            return (c, t.mark_layer);
        }
        // spec: S-0207 claim 20 (a) (the old point is the previous remembered L, including after a completed jump)
        if let Some(h) = t.held {
            return h;
        }
    }
    let c = o.pos.cell();
    // spec: S-0207 claim 20 (a) (the old-point mark read in the deck layer only on a high-bridge cell when the old point's height is at least H + 416 there, an exact integer comparison)
    (c, reservation_layer(w, o.pos))
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
    // spec: S-0207 claim 20 (b) (an aircraft's entry outcome is clear, S-0209 claim 14: it lands at the destination cell's centre even when that cell is occupied)
    if o.kind == Kind::Aircraft {
        return Some(Pos::centre(dc, h));
    }
    // spec: S-0207 claim 20 (b) (a vehicle asks at the destination cell's level plus 4 on a high-bridge cell, at its ground level elsewhere, whatever its own height and the search's deck mode)
    let level =
        w.cells.cell(dc).map_or(0, |c| i32::from(c.level)) + if high { DECK_LEVELS } else { 0 };
    // spec: S-0207 claim 20 (b) (a clear entry outcome: the centre of the destination cell at height H)
    if occupancy::vehicle_outcome(w, id, dc, level) == occupancy::Outcome::Clear {
        return Some(Pos::centre(dc, h));
    }
    crate::regions::ensure(w);
    // spec: S-0207 claim 20 (b) (the nearby-cell search: centred on the destination cell, the own class and unsimplified zone, the destination cell's reach region read for its deck in deck mode, deck mode as above, no height condition, no further test, bridges allowed, no reference cell)
    let reach = crate::regions::reach_read(w, dc, u.zone, deck);
    // spec: S-0207 claim 20 (b) (a destination cell in no reach region: no reach-region restriction on candidates; an off-map cell is not such a cell, S-0212 claim 4)
    let reach = (reach != crate::regions::Reach::None).then_some(reach);
    let c = path::nearby(
        w,
        dc,
        NearbyUse::Settings(NearbySettings {
            class: u.class,
            zone: u.zone,
            reach,
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
    let earlier = st.mark.is_some();
    if let Some(c) = st.mark {
        // spec: S-0208 claim 19 (Q-0235: a new destination reaching landing choice before an earlier landing point was jumped to releases the earlier reservation)
        w.cells.clear_infantry(c, st.mark_layer, id, oq);
    }
    // spec: S-0208 claim 19 and Edge cases (after a completed jump the old entry is cleared by the unit's actual position and height, whoever marked it, before the choice; quirks.teleport_infantry_ground_reservation)
    let by_position =
        !earlier && st.held.is_some() && w.rules().quirks.teleport_infantry_ground_reservation;
    if by_position {
        let p = walk::position_class(within(o.pos).0, within(o.pos).1, top);
        w.cells
            .clear_position(o.pos.cell(), reservation_layer(w, o.pos), p, oq);
    }
    match infantry_landing(w, id, dest) {
        Some((l, layer)) => {
            // spec: S-0208 claim 19 (an infantry unit reserves its chosen standing position, not a vehicle-present mark) and S-0209 claim 1 (the old entry cleared, the new one marked at once)
            if !earlier && !by_position {
                w.cells.clear_infantry(o.pos.cell(), o.layer, id, oq);
            }
            let mark = walk::position_class(within(l).0, within(l).1, top);
            if let Some(h) = o.owner {
                w.cells.mark_infantry(l.cell(), layer, mark, id, h);
            }
            st.mark = Some(l.cell());
            st.mark_layer = layer;
            st.held = None;
            st.landing = Some(l);
            st.for_dest = Some(dest);
            set_state(w, id, st);
        }
        None => {
            // spec: S-0208 claim 19 (no landing point: the destination is removed and the unit does not jump)
            // spec: S-0208 claim 19 (after a completed jump a refused choice marks the unit's current position, its deck position on a deck)
            if earlier || by_position {
                // spec: S-0208 claim 19 (Q-0235: a refused replacement discards the earlier landing point; the unit's entry is restored at its present position, the record chosen by that position's height and cell)
                let mark = walk::position_class(within(o.pos).0, within(o.pos).1, top);
                let layer = reservation_layer(w, o.pos);
                if let Some(h) = o.owner {
                    w.cells.mark_infantry(o.pos.cell(), layer, mark, id, h);
                }
            }
            set_state(w, id, TeleportState::default());
            drop_destination(w, id);
        }
    }
}

/// The occupancy record of an infantry reservation at point `p` (S-0208
/// claim 19): the deck's when its cell carries a high bridge and `p` is at
/// least 416 leptons above the terrain surface there, the ground's
/// otherwise.
pub(crate) fn reservation_layer(w: &World, p: Pos) -> Layer {
    let high = w.cells.idx(p.cell()).is_some_and(|i| w.cells.deck[i]);
    // spec: S-0208 claim 19 and Constants (a high-bridge cell and a height at least 416 above the terrain surface at the point)
    if high && p.z >= w.surface_at(p.x, p.y).wrapping_add(DECK_HEIGHT) {
        Layer::Deck
    } else {
        Layer::Ground
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
    // spec: S-0208 claim 19 (the standing-position choice reads the deck record only in standing deck mode, the ground record otherwise; the fallback choice keeps that mode)
    let choice = if deck { Layer::Deck } else { Layer::Ground };
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
    let first = walk::choose_position(w, c, choice, class, ignore);
    // spec: S-0208 claim 19 (the entry check reads deck occupancy on every high-bridge cell, independently of the choice's record)
    if let Some(pos) = first
        && entry_clear(w, id, c, high(w, c))
    {
        let l = point(w, c, pos);
        return Some((l, reservation_layer(w, l)));
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
    let pos = walk::choose_position(w, f, choice, crate::cells::position::CENTRE, ignore)?;
    let l = point(w, f, pos);
    // spec: S-0208 claim 19 (the reservation uses the found cell and the chosen height, not the nearby search's deck mode)
    Some((l, reservation_layer(w, l)))
}

/// S-0208 claim 19's entry check for an infantry landing on cell `c`.
pub(crate) fn entry_clear(w: &World, id: ObjectId, c: CellXY, high: bool) -> bool {
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
pub(crate) fn landing_layer(w: &World, c: CellXY) -> Layer {
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
    let Some(ob) = w.obj_mut(id) else { return };
    let had = ob.dest.is_some();
    ob.dest = None;
    ob.path.steps.clear();
    if had {
        // spec: S-0207 claim 28 (claim 20 a's drop, 20 b's removal without a jump and 20 d's removal are empty assignments: driving is installed)
        crate::refresh::removed(w, id);
    }
}

/// The movement update of a teleporting object (claim 20 d to f). Returns
/// whether it moved.
pub fn tick(w: &mut World, id: ObjectId) -> bool {
    take_destination(w, id);
    if !is_teleporter(w, id) {
        // spec: S-0207 claim 28 (driving installed by the destination just taken receives it; the teleporting locomotor makes no move)
        return false;
    }
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
    // spec: S-0614 claim 8 (a held unit does not move under the teleporting locomotor)
    if crate::parasite::held(w, id) {
        return false;
    }
    match st.landing {
        Some(l) if l != o.pos => {
            jump(w, id, l);
            true
        }
        _ => {
            // spec: S-0207 claim 27 (Q-1130 table, second row, and Edge cases: a destination kept under pending suppression stays without a landing point; no jump follows until a later effective assignment). Only that assignment leaves the taken destination equal to the order-level one with no landing point.
            if st.landing.is_none() && o.dest.is_some() && o.dest == st.for_dest {
                return false;
            }
            // spec: S-0207 claim 20 (d) (L equal to its own position, or none fixed: no jump, the destination is removed)
            if o.dest.is_some() || st.landing.is_some() {
                set_state(w, id, st.without_landing());
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
    targeters_drop(w, id);
    // spec: S-0207 claim 20 (e) and (i) (a warp animation at the old point, an ordinary animation creation with every applicable creation draw of S-0202 claim 10, before the landing-point animation's; landing choice and position change take no draw of their own; the sounds take no draw)
    if let Some(warp) = r.warp_out.clone() {
        crate::combat::effects::anim(w, &warp, o.pos);
    }
    // spec: S-0207 claim 20 (e) (then the departure sound at the old point; the type's overrides the game-wide one)
    chrono_sound(w, &o, Sound::ChronoOut, o.pos);
    // spec: S-0207 claim 20 (e) and S-0614 claim 12 (a parasite inside exits before the host leaves its old position)
    crate::parasite::host_jumps(w, id);
    let old = o.pos.cell();
    let new = l.cell();
    let high = w.cells.idx(new).is_some_and(|i| w.cells.deck[i]);
    // spec: S-0207 claim 20 (e) (on a high-bridge cell at H + 416, elsewhere at H; it keeps its facing)
    let z = w
        .surface_at(l.x, l.y)
        .wrapping_add(if high { DECK_HEIGHT } else { 0 });
    // spec: S-0207 claim 20 (e) (its cell layer is the bridge deck on a high-bridge cell, ground elsewhere, for every kind)
    let layer = landing_layer(w, new);
    let st = o.teleport.clone().unwrap_or_default();
    if let Some(d) = st.deck_mark {
        // spec: S-0207 claim 20 (e) (the old deck mark is cleared when the next jump removes the object from that cell)
        reserve(w, o.kind, d, Layer::Deck, false);
    }
    w.cells.leave(old, o.layer, id);
    if let Some(ob) = w.obj_mut(id) {
        ob.pos = Pos::new(l.x, l.y, z);
        ob.layer = layer;
    }
    w.cells.enter(new, layer, id, false);
    // spec: S-0212 claims 13 and 16a (recorded in a cell of another threat block: its threat amount moves there)
    crate::threat::relisted(w, id, new);
    // The reservation the object holds after the jump: the remembered landing point's.
    let mut held = st.mark.map(|c| (c, st.mark_layer));
    let mut deck_mark = None;
    if o.kind == Kind::Infantry {
        if let Some((c, from)) = held
            && c == new
            && from != layer
            && !w.rules().quirks.teleport_infantry_ground_reservation
        {
            // spec: S-0208 Edge cases (quirks.teleport_infantry_ground_reservation off: the reservation moves to the actual landing surface at the same standing position)
            let oq = crate::objects::occ_quirks(w);
            let top = w.rules().quirks.top_position_counts_as_centre;
            w.cells.clear_infantry(new, from, id, oq);
            let mark = walk::position_class(within(l).0, within(l).1, top);
            if let Some(h) = o.owner {
                w.cells.mark_infantry(new, layer, mark, id, h);
            }
            held = Some((new, layer));
        }
        // spec: S-0208 claim 19 (landing neither transfers the reservation nor marks the actual position: a ground-chosen point stays taken in ground, the deck gets no mark)
    } else if held != Some((new, layer)) {
        // spec: S-0207 claim 20 (e) (deck placement marks the deck record without clearing or transferring the ground reservation: both records are marked)
        reserve(w, o.kind, new, layer, true);
        if held.is_some() {
            deck_mark = Some(new);
        } else {
            held = Some((new, layer));
        }
    }
    // spec: S-0207 claim 20 (e) (the arrival sound plays at L)
    chrono_sound(w, &o, Sound::ChronoIn, Pos::new(l.x, l.y, z));
    // spec: S-0207 claim 20 (e) (entering L's cell has the arrival effects of the object's own kind) and S-0211 claim 5 (both cells are recomputed, for a walker as for a vehicle)
    crate::load::recalc_cells(w, &[old, new]);
    if o.kind == Kind::Vehicle {
        // spec: S-0207 claim 20 (e) (a vehicle's: those of reaching a track target in a new cell, crushing among them, S-0210 claim 9)
        crate::crush::entered_cell(w, id, new);
        // spec: S-0207 claim 22 (for a vehicle, arrival reporting to a contacted building, S-0402 claim 10, then cell-centre crushing, S-0210 claims 3 and 8, precede the entered-by raise)
        crate::econ::harvest::track_arrival(w, id);
        // spec: S-0207 claims 20 (e) and 22 (reaching a track target in a new cell is a cell-arrival update: a tethered vehicle's clear report to its factory, S-0417 claims 7 and 8, before crushing)
        crate::econ::release::arrival(w, id);
        crate::crush::at_cell_centre(w, id);
    } else if o.kind == Kind::Infantry {
        // spec: S-0207 claim 22 (the jump tick's arrival update, any object kind) and S-0417 claim 10 (a released, tethered teleporting infantry unit reports clear on its jump's arrival update, before the entered-by raise; no foundation-exit or destination test, no run-away reply)
        crate::econ::release::arrival(w, id);
    }
    // spec: S-0516 claims 13 and 22 (a teleporting mover of any kind has its cell-arrival update at the destination in the jump tick, after position, height and layer placement and a vehicle's arrival reporting and crushing, before the landing cell's entered-by actions and crate pickup: the sensor disc moves, then the adjacent sensor test; no cell between departure and landing gets one)
    crate::cloak::machine::cell_movement_completed(w, id);
    // spec: S-0207 claim 22 and S-0208 claim 22 (the arrival update in the jump tick raises the landing cell's entered-by tag, after position, height, listing and a vehicle's arrival effects, before crate collection, destination removal and the landing warp animation; no raise for the cells between)
    crate::objects::arrival_raise(w, id);
    // spec: S-0207 claim 20 (e) and S-0520 claim 6 (crate collection at L after the arrival update, before the destination is removed; the landing is never undone)
    crate::crates::pickup(w, id, l.cell());
    let had = o.dest.is_some();
    if let Some(ob) = w.obj_mut(id) {
        // spec: S-0207 claim 20 (e) (it stops moving and its destination is removed; a behaviour that ends once its destination is gone falls back to rest at a later tick, S-0215 claim 13, as the behaviour was processed before the locomotion)
        ob.dest = None;
        ob.path.steps.clear();
    }
    // spec: S-0207 claim 20 (e) and (i) (a second warp animation at L, with its creation draws after the old point's in the jump tick)
    if let Some(warp) = r.warp_out.clone() {
        crate::combat::effects::anim(w, &warp, Pos::new(l.x, l.y, z));
    }
    let harvester = r.unit(o.kind, o.type_index).is_some_and(|u| u.harvester);
    // spec: S-0207 claim 20 (f) and F9 (a harvester-flag type does not phase in)
    let phasing = (!harvester).then(|| (tick, delay(w, o.pos, l)));
    set_state(
        w,
        id,
        TeleportState {
            phasing,
            // spec: S-0207 claim 20 (a) (finishing a jump does not erase the remembered point)
            held,
            deck_mark,
            ..Default::default()
        },
    );
    if had {
        // spec: S-0207 claim 28 (the jump's removal of the destination, claim 20 d and e, is an empty assignment: driving is the active movement kind from the jump tick on)
        crate::refresh::removed(w, id);
    }
}

/// Every object that has `id` as its target drops it, and a projectile
/// aimed at it retargets (S-0207 claim 20 e, S-0306 claim 17), at the
/// teleporting locomotor's own jump. These drops are not S-0309 claim 27's
/// or claim 28's, so no holder takes the timer draw over 4 to 8 ticks,
/// whatever its timer shows (S-0513 claim 39, Changes for S-0207 claim 20
/// e). A superweapon warp's departure is not such a jump and calls nothing
/// here (S-0513 claim 39): a projectile aimed at the member keeps it and
/// reads its position at departure until it appears, and loses it only if
/// the landing check kills it (S-0306 claim 17, revision 57;
/// `combat::projectile::target_airborne`).
pub(crate) fn targeters_drop(w: &mut World, id: ObjectId) {
    // spec: S-0207 r83 claim 20 (e), S-0513 claim 39 and S-0309 claim 27 (the own jump's drops take no targeting-timer draw: the reset belongs to claim 27's and claim 28's drops only)
    let Some(pos) = w.obj(id).map(|o| o.pos) else {
        return;
    };
    // spec: S-0207 claim 20 (e) (every object that has it as its target drops it)
    // spec: S-0215 claim 10 (the target's own jump: an overridden attacker reverts first, then drops the jumper if it is still its target; a restored other target is kept)
    // The one exception, an attacker whose airstrike is aimed at the living target (S-0207 claim 20 (e),
    // S-0215 claim 10; not an erasing-weapon exception), is a seam: a strike aircraft's
    // target is the cell of the object (S-0519 claim 17) and only a redirect aims it at the object,
    // and S-0207's Open questions leave a stock setup that establishes the exception open.
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
            crate::spawn::set_own_target(ob, t);
            if ob.kind.is_mobile() {
                restored = Some(ob.dest);
                ob.dest = d;
            }
            // spec: S-0301 claim 5, Moment of a retarget's let-go (revision 67, Q-2246: at the change of the firer's target)
            crate::emitters::retarget_let_go(w, x);
            crate::combat::held::note_revert(w, x);
        }
        let Some(ob) = w.obj_mut(x) else { continue };
        if ob.target == Some(Target::Object(id)) {
            crate::spawn::clear_own_target(ob);
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
            p.target = (!airborne).then_some(Target::Cell(pos.cell()));
        }
    }
}

/// The departure or arrival sound of `o`'s own jump at `pos` (S-0207 claim
/// 20 e); the view picks the type's sound or the game-wide one.
pub(crate) fn chrono_sound(w: &mut World, o: &crate::world::Object, sound: Sound, pos: Pos) {
    w.effects.push(Effect::Sound {
        sound,
        of: Some(crate::econ::TypeRef {
            kind: o.kind,
            index: o.type_index,
        }),
        house: None,
        pos: Some(pos),
    });
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
    if let Some(ob) = w.obj_mut(id)
        && let Some(t) = ob.teleport.as_mut()
    {
        t.phasing = None;
    }
    phasing_search(w, id);
}

/// The target search at the end of phasing in (S-0207 claim 20 f), also
/// made by a superweapon warp member (S-0513 claim 39).
pub(crate) fn phasing_search(w: &mut World, id: ObjectId) {
    let tick = w.tick;
    let half = w.rules().quirks.ranged_draw_half_span;
    let Some(o) = w.obj(id).cloned() else { return };
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
