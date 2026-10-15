//! Multiplayer trigger completion (S-0524): the remaining stock actions of
//! the official multiplayer maps and the object-entry raises, on top of
//! S-0011's runtime in [`crate::triggers`].
//!
//! Built here: the repeated-strike switch of the lightning strike (code 90,
//! claim 2), the demolition blast (code 63, claim 5), the large light flash
//! (code 66, claim 6), the base ambient (code 73, claim 7), the local-area
//! resize (code 40, claim 8), the starting-building record and its
//! restoration (code 130, claim 9), and the trigger side of the entry raises
//! and tag carry-over (claims 10 to 12). Codes 102 (claim 1) and 95 (claim
//! 3) hand over to [`crate::superweapons`].
//!
//! Seams (their owning specs are not built yet): the crate action (code 108,
//! S-0520), engineer capture (S-0509), vehicle hijacking (the takeover of
//! S-0312 claim 13 belongs to the hijack ability), garrison entry (S-0508),
//! spy infiltration (S-0510) and sabotage. Each is a named function below
//! that states the order its owner must keep.

use std::collections::BTreeSet;

use glasnost_map::{CellXY, NO_OVERLAY};
use serde::{Deserialize, Serialize};

use crate::combat::area::Detonation;
use crate::combat::damage::Hit;
use crate::combat::effects::Effect;
use crate::rules::Kind;
use crate::superweapons::StormState;
use crate::world::{HouseId, Layer, ObjectId, Pos, World};

/// Largest waypoint number (S-0011 claim 33).
const LAST_WAYPOINT: i32 = 701; // spec: S-0011 claim 33 (waypoint numbers 0 … 701)

/// Damage per demolition blast (S-0524 Constants).
const BLAST_DAMAGE: i32 = 100; // spec: S-0524 claim 5 and Constants (each 100 hit points)
/// Blast displacement along each map axis (S-0524 Constants).
const BLAST_OFFSET: i32 = 85; // spec: S-0524 claim 5 and Constants (85 leptons along each map axis)
/// The five blast displacements in their observable precedence (claim 5).
// spec: S-0524 claim 5 (P, P + (85, 85, 0), P + (−85, 85, 0), P + (85, −85, 0), P + (−85, −85, 0))
const BLASTS: [(i32, i32); 5] = [
    (0, 0),
    (BLAST_OFFSET, BLAST_OFFSET),
    (-BLAST_OFFSET, BLAST_OFFSET),
    (BLAST_OFFSET, -BLAST_OFFSET),
    (-BLAST_OFFSET, -BLAST_OFFSET),
];

/// The large flash's size (S-0524 Constants).
const FLASH_SIZE: i32 = 300; // spec: S-0524 claim 6 and Constants (large flash size 300)
/// The fallback flash-size bounds (S-0524 Formulas).
const FLASH_MIN: i32 = 21; // spec: S-0524 Formulas (r = min(63, max(21, ⌊300 × 64 / 256⌋)))
const FLASH_MAX: i32 = 63; // spec: S-0524 Formulas (an upper bound of 63)
/// The fallback level's scale ⌊size × 64 / 256⌋ (S-0524 Formulas).
const FLASH_NUM: i32 = 64; // spec: S-0524 Formulas (× 64)
const FLASH_DEN: i32 = 256; // spec: S-0524 Formulas (/ 256)

// House values of S-0011 claim 23.
const TRIGGERING_HOUSE: i32 = 9013; // spec: S-0011 claim 23 (9013: the triggering house)
const NO_HOUSE: i32 = -1; // spec: S-0011 claim 23 (−1: none)
const POSITION_A: i32 = 4475; // spec: S-0011 claim 23 (4475 … 4482: the player at position A … H)
const POSITION_H: i32 = 4482; // spec: S-0011 claim 23 (4475 … 4482: the player at position A … H)

/// The base and target ambient that code 73 set (S-0524 claim 7; S-0104
/// claim 14), in hundredths; `None` keeps the map's own value. The fade of
/// S-0104 claim 4 and the lighting re-evaluations are the renderer's.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ambient {
    /// The base ambient.
    pub base: Option<i32>,
    /// The target ambient.
    pub target: Option<i32>,
}

/// One entry of the starting-building record (S-0524 Terms, claim 9): a
/// house, a building type and the building's top-left cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StartingBuilding {
    /// The owner house.
    pub house: HouseId,
    /// The building type.
    pub type_index: u32,
    /// The top-left cell.
    pub cell: CellXY,
}

/// The S-0524 state kept with the triggers.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MpState {
    /// The waypoints at which a code-90 action made a cloud (for
    /// `quirks.repeated_lightning_strike_suppressed`).
    pub struck: BTreeSet<i32>,
    /// Code 73's ambient values.
    pub ambient: Ambient,
    /// The starting-building record, in house order, then object order.
    pub starting: Vec<StartingBuilding>,
}

/// How this engine covers a trigger action code (for reports such as the
/// local lane's counts).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Coverage {
    /// The action takes effect.
    Implemented,
    /// Recognised; its effect belongs to a spec or system not built yet.
    Seam,
    /// Outside the covered subset.
    Unknown,
}

/// The coverage of action code `code` (S-0011 I4; S-0524 I1; S-0512 claims
/// 12 to 15; S-0513 claims 20 and 22).
pub fn action_coverage(code: i32) -> Coverage {
    match code {
        // spec: S-0011 I4 (the subset), S-0512 claims 12 to 15, S-0513 claims 20 and 22, S-0524 I1
        11 | 17 | 19 | 21 | 23..=27 | 33 | 34 | 40 | 44 | 45 | 53..=57 | 63 | 66 | 73 | 90
        | 95 | 99 | 102 | 103 | 129 | 130 | 133 => Coverage::Implemented,
        // spec: S-0011 claim 30 (teams, computer targeting, change owner, destroy attached, explosion, cheer, waypoint animations) and S-0524 claim 4 (crates, S-0520)
        4 | 5 | 7 | 8 | 14 | 32 | 41 | 42 | 80 | 108 | 113 | 135 => Coverage::Seam,
        _ => Coverage::Unknown,
    }
}

/// The coverage of event code `code` (S-0011 I3; the entry raises of S-0524
/// claims 10 to 12 use code 1).
pub fn event_coverage(code: i32) -> Coverage {
    match code {
        // spec: S-0011 I3 (the supported subset)
        1 | 8 | 13 | 31 | 36 | 37 | 47 | 48 | 51 | 60 | 61 => Coverage::Implemented,
        _ => Coverage::Unknown,
    }
}

/// The house of a house value (S-0011 claim 23), or none.
pub fn resolve_house(w: &World, value: i32, triggering: Option<HouseId>) -> Option<HouseId> {
    match value {
        TRIGGERING_HOUSE => triggering,
        NO_HOUSE => None,
        POSITION_A..=POSITION_H => {
            // spec: S-0011 claim 23 and S-0009 claim 21 (the house holding start position 0 … 7; S-0009 claim 29's rule for header counts of 9 or more is not built, so it is not applied)
            let k = (value - POSITION_A) as u32;
            w.houses
                .iter()
                .position(|h| !h.removed && h.start == Some(k))
                .map(|i| HouseId(i as u8))
        }
        // spec: S-0011 claim 23 (the first house in house order playing the country with registration index c)
        c => u32::try_from(c)
            .ok()
            .filter(|&c| (c as usize) < w.rules().country_names.len())
            .and_then(|c| crate::triggers::house_of_country(w, c)),
    }
}

/// The point of an undefined waypoint marker (S-0524 claims 5 and 6): the
/// centre of the blank stand-in cell (0, 0) at height 0.
// spec: S-0524 claim 5 (an undefined marker's centre is (128, 128, 0) integer leptons when the stand-in cell is blank; S-0009 claim 44: our stand-in always reads blank)
const ABSENT_POINT: (i32, i32, i32) = (128, 128, 0);

/// The waypoint of a number slot (codes 63 and 66, S-0524 claims 5 and 6).
enum Marker {
    /// A defined marker: its cell.
    Defined(CellXY),
    /// An undefined marker: the stand-in point, no cell.
    Absent,
}

/// The waypoint of a number slot: its marker, or none for an invalid number
/// (no effect, S-0011 claim 33's departure).
fn number_waypoint(w: &mut World, n: i32) -> Option<Marker> {
    if !(0..=LAST_WAYPOINT).contains(&n) {
        // spec: S-0524 claim 5 (invalid waypoint numbers follow S-0011 claim 33's departure: no effect)
        w.note("S-0524 claim 5: a blast or flash waypoint number lies outside 0 … 701; no effect");
        return None;
    }
    Some(match w.triggers.waypoints.get(&(n as u16)).copied() {
        Some((x, y)) => Marker::Defined(CellXY::new(i32::from(x), i32::from(y))),
        None => {
            // spec: S-0524 claims 5 and 6 (an undefined marker is not suppressed: the request uses the stand-in point; S-0011 claim 33's importer warning)
            w.note("S-0524 claim 5: a blast or flash names an undefined waypoint marker; it acts at the stand-in point (128, 128, 0)");
            Marker::Absent
        }
    })
}

/// The point of a marker: a defined marker's waypoint point, or the
/// stand-in point.
fn marker_point(w: &World, m: &Marker) -> Pos {
    match m {
        Marker::Defined(c) => waypoint_point(w, *c),
        Marker::Absent => Pos::new(ABSENT_POINT.0, ABSENT_POINT.1, ABSENT_POINT.2),
    }
}

/// The waypoint point of a cell (S-0524 Terms; S-0011 claim 28): the
/// cell's centre at the terrain surface, plus 416 leptons on a high bridge.
pub fn waypoint_point(w: &World, c: CellXY) -> Pos {
    // spec: S-0011 claim 28 (the centre of the cell at its centre height, raised 416 leptons on a high bridge)
    crate::superweapons::cell_point(w, c, true)
}

/// The lightning strike at a waypoint (code 90, S-0524 claim 2).
pub fn lightning_strike(w: &mut World, waypoint: i32, cell: CellXY) {
    let suppress = w.rules().quirks.repeated_lightning_strike_suppressed;
    if suppress && w.triggers.mp.struck.contains(&waypoint) {
        // spec: S-0524 Edge cases (quirks.repeated_lightning_strike_suppressed on: no further cloud at that waypoint)
        return;
    }
    let before = w.sw.clouds.len();
    // spec: S-0524 claim 2 (each action makes a cloud, whether or not a storm is active; no previous-strike exclusion)
    crate::superweapons::storm::cloud(w, cell);
    if w.sw.clouds.len() > before {
        w.triggers.mp.struck.insert(waypoint);
    }
}

/// The demolition blast at a waypoint (code 63, S-0524 claim 5).
pub fn demolition_blast(w: &mut World, waypoint: i32) {
    let Some(m) = number_waypoint(w, waypoint) else {
        return;
    };
    let p = marker_point(w, &m);
    let warhead = w.rules().combat.general.c4_warhead;
    // spec: S-0524 claim 5 (five successive area damages of 100 with the demolition warhead, no attacker and no attacking house, in this precedence)
    for (dx, dy) in BLASTS {
        crate::combat::area::detonate(
            w,
            Detonation {
                point: Pos::new(p.x + dx, p.y + dy, p.z),
                damage: BLAST_DAMAGE,
                warhead,
                source: None,
                house: None,
                // spec: S-0303 claim 17 (the trigger producer allows resource reduction: "the other producers named here allow reduction")
                resources: true,
            },
        );
    }
    // spec: S-0524 claim 5 (then up to four direct bridge-destruction attempts at the cell, stopping at the first success, whatever the bridge-destruction setting; none for an undefined marker)
    if let Marker::Defined(c) = m {
        crate::bridges::cell_attempts(w, c);
    }
}

/// The flash-size level of the large flash (S-0524 claim 6, Formulas), or
/// none when the demolition warhead does not have the bright flag.
pub fn flash_level(w: &World) -> Option<i32> {
    let r = w.rules();
    let wh = r.combat.general.c4_warhead?;
    let h = r.combat.warhead(wh);
    if !h.bright {
        // spec: S-0524 claim 6 (a flash only when the demolition warhead has the bright flag)
        return None;
    }
    let mode = r.mode;
    let f = h.light_size;
    Some(if f.gt(glasnost_num::F64::ZERO) {
        let one = glasnost_num::F64::from_i64(1, mode);
        // spec: S-0524 Formulas (level = trunc(RZ₅₃(63 × min(f, 1))), the common integer conversion; no lower bound)
        glasnost_num::F64::from_i64(i64::from(FLASH_MAX), mode)
            .mul(f.min(one), mode)
            .to_i32_common()
    } else {
        // spec: S-0524 Formulas (r = min(63, max(21, ⌊300 × 64 / 256⌋)) = 63)
        (FLASH_SIZE * FLASH_NUM / FLASH_DEN).clamp(FLASH_MIN, FLASH_MAX)
    })
}

/// The large light flash at a waypoint (code 66, S-0524 claim 6): a
/// drawing-only request at the waypoint point, as an [`Effect::Trigger`]
/// whose number is the flash-size level. The presentation applies the
/// detail-level and frame-rate gate; the action takes no draw and deals no
/// damage.
pub fn light_flash(w: &mut World, a: &glasnost_map::triggers::Action) {
    let Some(m) = number_waypoint(w, a.number) else {
        return;
    };
    let Some(level) = flash_level(w) else {
        return;
    };
    let pos = marker_point(w, &m);
    w.effects.push(Effect::Trigger {
        code: a.code,
        name: String::new(),
        text: String::new(),
        number: level,
        pos: Some(pos),
    });
}

/// Whether a weather storm is active for the lighting (S-0104 Terms: in
/// force from its start until it ends, clearing included).
fn storm_in_force(w: &World) -> bool {
    // spec: S-0104 Terms (a clearing ion storm remains in force until it ends)
    matches!(
        w.sw.storm.state,
        StormState::Raging | StormState::Clearing
    )
}

/// Set base ambient (code 73, S-0524 claim 7; S-0104 claim 14).
pub fn set_ambient(w: &mut World, value: i32) {
    let storm = storm_in_force(w);
    let a = &mut w.triggers.mp.ambient;
    // spec: S-0524 claim 7 (the number slot becomes base ambient in hundredths, without division or rounding)
    a.base = Some(value);
    if !storm {
        // spec: S-0524 claim 7 (it also sets target ambient unless a weather storm is active)
        a.target = Some(value);
    }
}

/// Resize local area (code 40, S-0524 claim 8): items four to seven become
/// the local area's left, top, width and height; every legal cell is
/// recalculated in the visiting order, and the passability categories and
/// the reach regions are rebuilt. The radar (`econ::minimap`) and the view
/// read the local area from the cells, so they follow at once.
// spec: S-0524 claim 8 (four items: left, top, width, height)
pub fn resize_local_area(w: &mut World, items: [i32; 4]) {
    // spec: S-0524 claim 8 (left, top, width and height, as signed integers in S-0009 claim 4's units)
    w.cells.grid.set_local(items);
    let narrow = w.rules().quirks.narrow_map_partial_pass;
    let order = w.cells.footprint().visiting_order(narrow);
    // spec: S-0524 claim 8 (every legal cell receives terrain recalculation under S-0010 claims 8 and 19; the once-only tile-animation state is kept, S-0010 claim 19)
    crate::load::recalc_cells(w, &order);
    // spec: S-0524 claim 8 (subsequent usable-area tests use the new area: S-0211 claim 11 rule 1)
    for &c in &order {
        let cat = crate::passability::compute_category(w, c);
        if let Some(i) = w.cells.idx(c) {
            w.cells.category[i] = cat;
        }
    }
    // spec: S-0524 claim 8 (pathfinding connectivity is rebuilt for the new area, S-0212 claims 3 and 4)
    crate::regions::rebuild(w);
    // spec: S-0524 claim 8 (shrinking does not remove, relocate, stop or switch off existing buildings or mobile units and does not clear their orders; an excluded power source still contributes and an excluded building still counts; nothing here touches them)
}

/// Records the starting buildings (S-0524 claim 9, Terms): before the
/// first tick, after tag setup and the game-start terrain pass, every
/// house's buildings by type and top-left foundation cell. Neither health
/// nor tags are recorded. A house's entries are in their surviving object
/// creation order, oldest first (the object order, which is creation order),
/// not in type or cell order.
pub fn record_starting_buildings(w: &mut World) {
    let mut rec = Vec::new();
    for h in 0..w.houses.len() {
        let house = HouseId(h as u8);
        for o in w.objects.values() {
            if o.kind == Kind::Building && o.on_map && o.owner == Some(house) {
                rec.push(StartingBuilding {
                    house,
                    type_index: o.type_index,
                    cell: o.pos.cell(),
                });
            }
        }
    }
    w.triggers.mp.starting = rec;
}

/// The present building of the record entry's type and house that covers
/// its cell.
// spec: S-0524 claim 9 (a recorded cell covered by any ground-layer foundation cell of a building of the recorded type and house; the covering building need not have its top-left cell there)
fn holder(w: &World, b: &StartingBuilding) -> Option<ObjectId> {
    w.objects
        .iter()
        .find(|(_, o)| {
            o.kind == Kind::Building
                && o.on_map
                && o.type_index == b.type_index
                && o.owner == Some(b.house)
                && crate::econ::placement::foundation_cells(w, o.type_index, o.pos.cell())
                    .contains(&b.cell)
        })
        .map(|(&id, _)| id)
}

/// Restore starting buildings (code 130, S-0524 claim 9) for the house that
/// `value` resolves to (S-0011 claim 23).
pub fn restore_starting_buildings(w: &mut World, value: i32, triggering: Option<HouseId>) {
    let Some(h) = resolve_house(w, value, triggering) else {
        // spec: S-0524 claim 9 (an unresolved house gives no effect)
        return;
    };
    let rec: Vec<StartingBuilding> = w
        .triggers
        .mp
        .starting
        .iter()
        .filter(|b| b.house == h)
        .copied()
        .collect();
    for b in rec {
        if let Some(id) = holder(w, &b) {
            heal(w, id);
        } else {
            replace(w, &b);
        }
    }
}

/// The healing hit of a restoration (S-0524 claim 9): minus the type's
/// strength, distance 0, the demolition warhead, no attacker or attacking
/// house, neither ignoring defences nor preventing escape; S-0302 decides.
fn heal(w: &mut World, id: ObjectId) {
    let s = crate::combat::damage::strength(w, id);
    let warhead = w.rules().combat.general.c4_warhead;
    // spec: S-0524 claim 9 (a healing hit of minus the type's strength, distance 0, demolition warhead, no attacker or house, no markers)
    crate::combat::damage::apply(
        w,
        id,
        Hit {
            amount: s.wrapping_neg(),
            warhead,
            distance: 0,
            attacker: None,
            house: None,
            ignore_defences: false,
            no_escape: false,
        },
    );
}

/// A displaced occupant (S-0524 claim 13): its id, the point it stood at and
/// its layer.
struct Displaced {
    id: ObjectId,
    pos: Pos,
    layer: Layer,
}

/// The ground-layer mobile occupants of the foundation cells (S-0524
/// claim 13), in lifting order: the foundation's cell order, then each
/// cell's listing order.
// spec: S-0524 claim 13 (only ground-layer mobile occupants of the attempted foundation cells are lifted: a landed aircraft is included; a bridge-deck occupant, an airborne aircraft and a mobile in an adjacent non-foundation cell are not; ties in one cell are unmeasured, so the listing order is used)
fn occupants(w: &World, cells: &[CellXY]) -> Vec<Displaced> {
    let mut out: Vec<Displaced> = Vec::new();
    for &c in cells {
        for &id in w.cells.list(c, Layer::Ground) {
            let Some(o) = w.obj(id) else { continue };
            let airborne = o.kind == Kind::Aircraft
                && (o.rocket.is_some() || crate::combat::weapon::in_air(w, id));
            if o.kind.is_mobile()
                && o.on_map
                && o.layer == Layer::Ground
                && !airborne
                && !out.iter().any(|d| d.id == id)
            {
                out.push(Displaced {
                    id,
                    pos: o.pos,
                    layer: o.layer,
                });
            }
        }
    }
    out
}

/// The forced replacement of a missing starting building (S-0524 claims 9
/// and 13).
fn replace(w: &mut World, b: &StartingBuilding) {
    let cells = crate::econ::placement::foundation_cells(w, b.type_index, b.cell);
    // spec: S-0524 claim 9 (its foundation overlays are removed; not restored on failure)
    for &c in &cells {
        if w.cells.cell(c).is_some_and(|x| x.overlay != NO_OVERLAY) {
            crate::crush::remove_overlay(w, c);
        }
    }
    let lifted = occupants(w, &cells);
    for d in &lifted {
        crate::objects::take_inside(w, d.id);
    }
    // spec: S-0524 claim 13 (a missing building is a fresh object at full type strength, not the former object's saved state)
    let id = crate::objects::create(w, Kind::Building, b.type_index, Some(b.house));
    let test = crate::econ::placement::Test::order(b.house);
    let placed = crate::econ::placement::cell_test(w, b.type_index, b.cell, &test);
    if placed {
        // spec: S-0524 claim 13 (ordinary during-play placement, not map-loading placement: construction is queued and its build-up follows S-0215 claim 1 and S-0406 claims 14, 15 and 17; the power switch begins on; no special restoration shortcut; a deploy alone sets the ready mark at entry, S-0406 claim 17)
        crate::econ::placement::begin_construction(w, id, false);
        // spec: S-0406 claim 1 and F4 (the centre of the cell at the surface height)
        let z = w.surface_at(256 * b.cell.x + 128, 256 * b.cell.y + 128);
        // spec: S-0524 claim 9 (a fresh building is attempted at the recorded top-left cell, facing 0) and S-0406 claim 16 (a placed building faces 0)
        crate::objects::place(w, id, Pos::centre(b.cell, z), 0);
        crate::regions::ensure(w);
        let mut taken: BTreeSet<CellXY> = BTreeSet::new();
        // spec: S-0524 claim 13 (displaced occupants are placed one at a time in reverse lifting order)
        for d in lifted.iter().rev() {
            if !place_near(w, d, &mut taken) {
                // spec: S-0524 claim 13 (search failure leaves no unit under the new building or off the map: the original attempts placement at the stand-in cell (0, 0), which is never legal here, and removes the unit when that placement fails; a found cell whose placement fails also removes it)
                w.note("S-0524 claim 13: a displaced unit found no placeable nearby cell; it is removed");
                crate::objects::remove(w, d.id);
            }
        }
    } else {
        // spec: S-0524 claim 9 (a blocking building or other placement failure leaves no replacement)
        w.objects.remove(&id);
        // spec: S-0524 claim 13 (previously lifted occupants are offered their original positions, facing 0; a failed return placement removes that occupant: quirks.restoration_failed_return_removes_unit)
        let mut taken: BTreeSet<CellXY> = BTreeSet::new();
        for d in lifted.iter().rev() {
            if return_free(w, d) {
                put_back(w, d, d.pos);
            } else if w.rules().quirks.restoration_failed_return_removes_unit {
                crate::objects::remove(w, d.id);
            } else {
                // spec: S-0524 Edge cases (switch off: kept, placed at a nearby cell)
                crate::regions::ensure(w);
                if !place_near(w, d, &mut taken) {
                    crate::objects::remove(w, d.id);
                }
            }
        }
    }
}

/// Whether a displaced unit's original cell can take it back: no building
/// holds it.
fn return_free(w: &World, d: &Displaced) -> bool {
    let c = d.pos.cell();
    w.cells.idx(c).is_some()
        && !w
            .objects
            .values()
            .any(|o| o.kind == Kind::Building && o.on_map && o.pos.cell() == c)
}

/// Places a displaced unit at the cell S-0212 claim 25's search chooses
/// (S-0524 claim 13): the 1 × 1-cell search centred on its old cell, with
/// its type's locomotion class, movement zone *normal*, no reach region,
/// deck mode off, and no height, further or no-overlay condition, bridges
/// allowed, and its old cell as reference. An infantry unit refuses a cell
/// that another displaced infantry unit took. Returns whether it was placed.
fn place_near(w: &mut World, d: &Displaced, taken: &mut BTreeSet<CellXY>) -> bool {
    let Some((kind, ti)) = w.obj(d.id).map(|o| (o.kind, o.type_index)) else {
        return false;
    };
    let Some(class) = w.rules().unit(kind, ti).map(|u| u.class) else {
        return false;
    };
    let old = d.pos.cell();
    // spec: S-0524 claim 13 (the nearby search arguments: type's class, zone normal, no reach restriction, deck off, no height, further or no-overlay condition, bridges allowed, old cell as reference)
    let settings = crate::path::NearbySettings {
        class,
        zone: glasnost_rules::sim::MovementZone::Normal,
        reach: None,
        deck: false,
        height: false,
        reference: Some(old),
        no_bridges: false,
        no_overlay: false,
    };
    let infantry = kind == Kind::Infantry;
    let refuse: BTreeSet<CellXY> = if infantry { taken.clone() } else { BTreeSet::new() };
    let free = |_: &World, c: CellXY| !refuse.contains(&c);
    let Some(c) = crate::path::nearby_filtered(
        w,
        old,
        crate::path::NearbyUse::Settings(settings),
        Some(&free),
    ) else {
        return false;
    };
    // spec: S-0524 claim 13 (the placement point is the selected cell's terrain-surface centre, without a bridge lift; an infantry unit can choose a standing position there by its ordinary placement rules)
    let (u, v) = if infantry {
        let top = w.rules().quirks.top_position_counts_as_centre;
        let class = crate::walk::position_class(128, 128, top); // spec: S-0208 table P (the centre)
        let Some(p) = crate::walk::choose_position(w, c, Layer::Ground, class, false) else {
            return false;
        };
        crate::walk::standing_point(p)
    } else {
        (128, 128) // spec: S-0406 F4 (the cell centre)
    };
    let (x, y) = (256 * c.x + u, 256 * c.y + v);
    let at = Pos::new(x, y, w.surface_at(x, y));
    if infantry {
        taken.insert(c);
    }
    put_back(w, d, at);
    true
}

/// Puts a displaced unit back on the map at `pos`, facing 0; the ordinary
/// placement fall-back of S-0215 claims 13 to 15 applies, and its old order
/// is neither saved nor reinstated (S-0524 claim 13).
fn put_back(w: &mut World, d: &Displaced, pos: Pos) {
    if let Some(o) = w.obj_mut(d.id) {
        o.layer = d.layer;
    }
    // spec: S-0524 claim 13 (facing 0 for every re-placed mobile; its facing is not preserved)
    crate::objects::place(w, d.id, pos, 0);
}

/// Whether the tag carried by `entrant` passes on (S-0011 I1; S-0524 claim
/// 10): any trigger of its chain has the carry-over flag.
pub fn carries_over(w: &World, entrant: ObjectId) -> Option<u32> {
    let &tag = w.triggers.objects.get(&entrant)?;
    let chain = &w.triggers.map.tags.get(tag as usize)?.chain;
    // spec: S-0011 I1 (when any trigger in the entrant's tag chain has the carry-over flag)
    chain
        .iter()
        .any(|&t| {
            w.triggers
                .map
                .triggers
                .get(t as usize)
                .is_some_and(|d| d.carry_over)
        })
        .then_some(tag)
}

/// The tag carry-over of a capture or hijack (S-0524 claims 10 and 11):
/// when the entrant still carries a tag that passes on, the target receives
/// it, replacing its previous tag. The previous tag loses one carrier and is
/// not deleted (S-0011 claim 2). Returns whether the tag passed.
pub fn carry_over(w: &mut World, entrant: ObjectId, target: ObjectId) -> bool {
    let Some(tag) = carries_over(w, entrant) else {
        return false;
    };
    // spec: S-0524 claim 10 (replacing its previous tag and dropping one of the old tag's carriers; losing its last carrier does not delete it)
    if let Some(old) = w.triggers.objects.insert(target, tag)
        && let Some(t) = w.triggers.tags.get_mut(&old)
    {
        t.carriers -= 1;
    }
    if let Some(t) = w.triggers.tags.get_mut(&tag) {
        t.carriers += 1;
    }
    true
}

/// An entry raise (S-0524 Terms, claim 12): the entered object's tag with
/// the entrant as causing object.
pub fn entry_raise(w: &mut World, entered: ObjectId, entrant: ObjectId) {
    // spec: S-0524 Terms (an entered-by raise on the entered object's tag, with the entrant as causing object)
    crate::triggers::building_entered(w, entered, entrant);
}

/// Seam: the trigger notifications of an engineer capture (S-0524 claim 10;
/// S-0509 claims 2 and 8), for S-0509's capture when it is built. In order:
/// the general building-entry raise and the capture raise, both before the
/// capture's own changes; then the tag carry-over, and the ownership change
/// that `takeover` makes (S-0509; no relative order between the two is
/// prescribed, Q-0765 interim: carry-over first); then the engineer's
/// destroyed-by-anything raise, then the engineer is removed.
pub fn capture_seam(
    w: &mut World,
    building: ObjectId,
    engineer: ObjectId,
    takeover: impl FnOnce(&mut World),
) {
    // spec: S-0524 claim 10 (two entry raises, the general one and the capture raise, before capture's own ownership and tag changes)
    entry_raise(w, building, engineer);
    entry_raise(w, building, engineer);
    // spec: S-0524 claim 10 (the engineer's tag passes when its chain has the carry-over flag)
    carry_over(w, engineer, building);
    takeover(w);
    // spec: S-0524 claim 10 (the engineer's own tag: one destroyed-by-anything raise with the engineer as causing object before it is removed)
    crate::triggers::destroyed_by_anything(w, engineer);
    crate::objects::remove(w, engineer);
}

/// Seam: the trigger notifications of a vehicle hijack (S-0524 claim 11;
/// S-0312 claim 13), for the hijack ability when it is built: the vehicle's
/// tag raised with the thief as causing object; then carry-over and the
/// takeover `takeover` makes (the thief's house, its type as the hijacker
/// type); the thief is removed with no destroyed-by-anything raise.
pub fn hijack_seam(
    w: &mut World,
    vehicle: ObjectId,
    thief: ObjectId,
    takeover: impl FnOnce(&mut World),
) {
    // spec: S-0524 claim 11 (the takeover raises the vehicle's existing tag with the thief as causing object, seeing the old owner and tag)
    entry_raise(w, vehicle, thief);
    // spec: S-0524 claim 11 (then the carry-over rule of claim 10)
    carry_over(w, thief, vehicle);
    takeover(w);
    // spec: S-0524 claim 11 (the thief is removed without a destroyed-by-anything raise on its own tag)
    crate::objects::remove(w, thief);
}

/// Seam: a consuming entry (S-0524 claim 12; S-0510 claim 9 for a spy):
/// the building's entry raise precedes the entry's effect `effect`, and the
/// entrant's own tag receives destroyed-by-anything after the effect and
/// before removal. For spy infiltration (S-0510) and the other consuming
/// building entries when their specs are built.
pub fn consuming_entry_seam(
    w: &mut World,
    building: ObjectId,
    entrant: ObjectId,
    effect: impl FnOnce(&mut World),
) {
    // spec: S-0524 claim 12 (the building's entry raise precedes the infiltration effect)
    entry_raise(w, building, entrant);
    effect(w);
    // spec: S-0524 claim 12 (the spy's own tag receives destroyed-by-anything with the spy as causing object after that effect and before removal)
    crate::triggers::destroyed_by_anything(w, entrant);
    crate::objects::remove(w, entrant);
}

/// Seam: a retaining entry (S-0524 claim 12): garrison entry (S-0508),
/// sabotage entry and vehicle entry into an ordered target building raise
/// the entered object's tag before the entry's own effect `effect`; being
/// retained raises nothing on the entrant. Transport, pad, dock and
/// refinery entries raise nothing (Q-0762 interim).
pub fn retaining_entry_seam(
    w: &mut World,
    building: ObjectId,
    entrant: ObjectId,
    effect: impl FnOnce(&mut World),
) {
    // spec: S-0524 claim 12 (the entered-by action effects can occur before the entrant's garrison, sabotage or building-entry effect)
    entry_raise(w, building, entrant);
    effect(w);
}

/// Seam: create a crate at a waypoint (code 108, S-0524 claim 4). Crates
/// (S-0520: the crate places, overlays, lifetime, expiry and power-ups) are
/// not built, so the action is recognised and has no effect. When they are:
/// S-0212 claim 25's single-cell search at the waypoint-slot cell (float
/// class on water, track otherwise, normal zone), the 256-place capacity,
/// the overlay and usable-area gates, the overlay image by water, the
/// lifetime draw of Formulas, the number slot's kind (20 random; otherwise
/// reduced modulo 256, 0 … 18 fixed, 19 … 255 random), and the switches
/// `quirks.trigger_crate_respects_setting` and
/// `quirks.trigger_crate_expiry_while_disabled`.
pub fn crate_seam(w: &mut World, _a: &glasnost_map::triggers::Action) {
    w.note("S-0524 claim 4: the create-crate action hands over to crates (S-0520), not built yet; no effect");
}
