//! A house's base records and its house-selected centre (S-0215 claim 38,
//! F11 and F12): the recorded base centre, base-start cell and base
//! radius, refreshed when an owned building enters, leaves or changes
//! owner, and the centre a neutral or special house's unit gets after
//! giving up on a failed path search (S-0212 claim 23a).

use glasnost_map::CellXY;
use glasnost_rules::sim::{LocomotionClass, MovementZone};
use serde::{Deserialize, Serialize};

use crate::path::{self, NearbySettings, NearbyUse};
use crate::rules::Kind;
use crate::world::{HouseId, ObjectId, Pos, World};

/// A house's base records (S-0215 claim 38).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BaseRecord {
    /// The recorded base centre, set and cleared by map actions only (none
    /// of the supported trigger actions does, so it stays absent).
    pub centre: Option<CellXY>,
    /// The recorded base-start cell.
    pub start: Option<CellXY>,
    /// The recorded base radius, leptons.
    pub radius: i32,
}

/// The bounds of the radius of the random choice, leptons.
// spec: S-0215 claim 38 (R: the recorded radius clamped to 768 … 2048 leptons)
const CHOICE_RADIUS: (i32, i32) = (768, 2048);
/// The smallest recorded radius of a house with buildings, leptons.
// spec: S-0215 F12 (512 leptons when N > 0 and Q ≤ 1; otherwise at least 512)
const MIN_RADIUS: i32 = 512;
/// Credits per unit of a building's weight.
// spec: S-0215 F12 (wᵢ = max(0, trunc(pᵢ / 1000) + 1))
const WEIGHT_CREDITS: i32 = 1000;
/// Leptons per cell.
// spec: S-0204 claim 2 (256 leptons per cell)
const CELL: i32 = 256;
/// The largest cell coordinate of the in-range test of F11.
// spec: S-0215 F11 (trunc(A / 256) and trunc(B / 256) within 0 … 511)
const MAX_CELL: i32 = 511;

fn house_record(w: &World, h: HouseId) -> Option<&BaseRecord> {
    w.houses.get(usize::from(h.0)).map(|x| &x.base)
}

/// The cell's reference point at its terrain height.
fn cell_point(w: &World, c: CellXY) -> Pos {
    let p = Pos::centre(c, 0);
    Pos::centre(c, w.surface_at(p.x, p.y))
}

/// The preferred recorded cell: the base centre, else the base-start cell
/// (S-0215 claim 38). S-0309 claim 25, part j, takes the same cell.
pub(crate) fn preferred(r: &BaseRecord) -> Option<CellXY> {
    r.centre.or(r.start)
}

/// A start assignment supplies the house's base-start cell (S-0215 claim
/// 38, S-0009 claim 19).
pub fn start_assigned(w: &mut World, h: HouseId, c: CellXY) {
    if let Some(house) = w.houses.get_mut(usize::from(h.0)) {
        house.base.start = Some(c);
    }
}

/// The refresh of a house's base-start cell and radius (F12) after an
/// owned building entered or left the map or changed owner. `leaving`: a
/// building about to leave, no longer counted.
pub fn refresh(w: &mut World, h: HouseId, leaving: Option<ObjectId>) {
    if house_record(w, h).is_none() {
        return;
    }
    let owned: Vec<ObjectId> = w
        .objects
        .values()
        .filter(|o| o.kind == Kind::Building && o.owner == Some(h) && Some(o.id) != leaving)
        .map(|o| o.id)
        .collect();
    // spec: S-0215 F12 (N, the house's owned buildings; B, those on the map with positive health)
    let n = owned.len();
    let b: Vec<ObjectId> = owned
        .iter()
        .copied()
        .filter(|&id| w.obj(id).is_some_and(|o| o.on_map && o.health > 0))
        .collect();
    let mut q: i32 = 0;
    let (mut sx, mut sy) = (0i32, 0i32);
    for &id in &b {
        let Some(o) = w.obj(id) else { continue };
        let t = crate::econ::TypeRef {
            kind: Kind::Building,
            index: o.type_index,
        };
        // spec: S-0215 F12 (pᵢ, the type's current purchase price for this house, S-0404 claim 2; wᵢ = max(0, trunc(pᵢ / 1000) + 1))
        let p = crate::econ::production::price(w, h, t);
        let wi = (p / WEIGHT_CREDITS).wrapping_add(1).max(0);
        let r = w.reference_point(id);
        // spec: S-0215 F12 (integer sums and products wrap as signed 32-bit values)
        q = q.wrapping_add(wi);
        sx = sx.wrapping_add(wi.wrapping_mul(r.x));
        sy = sy.wrapping_add(wi.wrapping_mul(r.y));
    }
    // spec: S-0215 claim 38 (the refresh clears the previous base-start cell and radius)
    let start = if n > 0 && q > 0 {
        // spec: S-0215 F12 (the mean cell trunc(trunc(Σ wᵢXᵢ / Q) / 256), each coordinate reduced modulo 65536 into −32768 … 32767)
        let mx = i32::from(((sx / q) / CELL) as i16);
        let my = i32::from(((sy / q) / CELL) as i16);
        crate::regions::ensure(w);
        // spec: S-0215 claim 38 (the mean's cell replaced by a nearby-cell result: foot class, normal zone, no reach-region restriction, deck mode, no height, no-overlay or burrowing condition, no reference cell, no bridge exclusion; S-0212 claim 25)
        path::nearby(
            w,
            CellXY::new(mx, my),
            NearbyUse::Settings(NearbySettings {
                class: Some(LocomotionClass::Foot),
                zone: MovementZone::Normal,
                reach: None,
                deck: true,
                height: false,
                reference: None,
                no_bridges: false,
                no_overlay: false,
            }),
        )
    } else {
        // spec: S-0215 F12 (N = 0 or Q ≤ 0: the base-start cell stays absent)
        None
    };
    let centre = house_record(w, h).and_then(|r| r.centre);
    // spec: S-0215 F12 (P: the preferred recorded cell after the search, or (0, 0, 0))
    let pp = centre
        .or(start)
        .map_or(Pos::new(0, 0, 0), |c| cell_point(w, c));
    let radius = if n == 0 {
        // spec: S-0215 F12 (the radius is 0 when N = 0)
        0
    } else if q <= 1 {
        MIN_RADIUS
    } else {
        // spec: S-0215 F12 (each member of B once: max(512, trunc(Σ distance₃(P, (Xᵢ, Yᵢ, Zᵢ)) / Q)), the sum wrapping)
        let sum = b.iter().fold(0i32, |s, &id| {
            let d = crate::approach::rho3(w, pp, w.reference_point(id)) as i32;
            s.wrapping_add(d)
        });
        (sum / q).max(MIN_RADIUS)
    };
    if let Some(house) = w.houses.get_mut(usize::from(h.0)) {
        house.base.start = start;
        house.base.radius = radius;
    }
}

/// The house-selected centre for a unit of house `h` that gave up after a
/// failed path search (S-0215 claim 38, F11). Takes one ranged and one raw
/// scenario-stream draw.
pub fn selected_centre(w: &mut World, h: HouseId) -> Option<CellXY> {
    let rec = house_record(w, h)?.clone();
    let q = w.rules().quirks;
    let mode = w.rules().mode;
    // spec: S-0215 claim 38 (prefer the recorded base centre, then the base-start cell, its reference point at its terrain height; neither: world position (0, 0, 0))
    let p = preferred(&rec).map_or(Pos::new(0, 0, 0), |c| cell_point(w, c));
    // spec: S-0215 claim 38 (R, the recorded radius clamped to 768 … 2048 in exact integer arithmetic)
    let big_r = rec.radius.clamp(CHOICE_RADIUS.0, CHOICE_RADIUS.1);
    // spec: S-0215 claim 38 (one U(0, R) for the displacement radius, then exactly one raw value for the direction; no outer-choice draw)
    let r = w.rng.ranged(0, big_r, q.ranged_draw_half_span);
    // spec: S-0215 F11 (b, the raw value mod 256; f = 256b facing units)
    let b = w.rng.raw() % 256;
    // spec: S-0215 F11 (f = 256b facing units)
    let f = (b * 256) as u16;
    // spec: S-0215 F11 (A = trunc(X + r × cosT θ), B = trunc(Y − r × sinT θ): S-0208 F3's scale and grouping with distance r)
    // spec: S-0215 F11 (the shared quirks.table_trig_negative_bias has S-0105 F1's alternative here too)
    let (a, bb) = crate::walk::step_point_b(
        p.x,
        p.y,
        f,
        r,
        q.table_direction,
        mode,
        q.table_trig_negative_bias,
    );
    // spec: S-0215 F11 (a point whose trunc(A / 256) or trunc(B / 256) lies outside 0 … 511 is P instead)
    let inside = |v: i32| (0..=MAX_CELL).contains(&(v / CELL));
    let (a, bb) = if inside(a) && inside(bb) {
        (a, bb)
    } else {
        (p.x, p.y)
    };
    // spec: S-0215 F11 (snapped to (256⌊A / 256⌋ + 128, 256⌊B / 256⌋ + 128), floor division)
    let snap = |v: i32| CELL * v.div_euclid(CELL) + CELL / 2;
    // spec: S-0215 F11 (the selected cell: each snapped coordinate divided by 256, truncating toward zero)
    let c = CellXY::new(snap(a) / CELL, snap(bb) / CELL);
    Some(correct(w, c))
}

/// The usable-area correction of F11.
fn correct(w: &World, c: CellXY) -> CellXY {
    // spec: S-0215 F11 (a cell already passing the height-aware usable test is unchanged)
    if w.cells.usable(c) {
        return c;
    }
    let fp = *w.cells.footprint();
    let ww = i64::from(fp.width);
    let [lx, ly, lw, lh] = fp.local.map(i64::from);
    let d = c.d();
    // spec: S-0215 F11 (the lateral displacement a, by the table on d = x − y; the divisions truncate toward zero)
    let a = if d >= 2 * (lx + lw) - ww {
        -((2 * (1 - lx - lw) + ww + d) / 2)
    } else if d <= 2 * lx - ww {
        (2 * lx + 2 - ww - d) / 2
    } else {
        0
    };
    let (u, v) = (i64::from(c.x) + a, i64::from(c.y) - a);
    let lateral = CellXY::new(u as i32, v as i32);
    // spec: S-0215 F11 (h at the candidate: its terrain level, plus 1 when sloped and u + v < W + 4 + 2Ly + level; a non-legal candidate reads S-0009 claim 44's stand-in cell, level 0 and flat)
    let (level, slope) = w
        .cells
        .cell(lateral)
        .map_or((0, false), |x| (i64::from(x.level), x.slope != 0));
    let s = u + v;
    // spec: S-0215 F11 (plus 1 when sloped and u + v < W + 4 + 2Ly + level)
    let h = level + i64::from(slope && s < ww + 4 + 2 * ly + level);
    // The corrected centre steps along the diagonal; no legal cell lies
    // beyond twice the largest coordinate.
    // spec: S-0009 claim 2 (coordinates within 0 … 511)
    let steps = 2 * (MAX_CELL + 1);
    let along = |sign: i32| {
        (1..=steps)
            .map(|k| CellXY::new(lateral.x + sign * k, lateral.y + sign * k))
            .find(|&x| w.cells.usable(x))
            .unwrap_or(lateral)
    };
    // spec: S-0215 F11 (u + v ≤ W + 2Ly + h: the smallest positive k with (u + k, v + k) usable; u + v > W + 2 + 2(Ly + Lh) + h: (u − k, v − k); otherwise (u, v))
    if s <= ww + 2 * ly + h {
        along(1)
    } else if s > ww + 2 + 2 * (ly + lh) + h {
        along(-1)
    } else {
        lateral
    }
}
