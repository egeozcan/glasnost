//! Shroud and sight (S-0409): the explored and seen states of every player
//! house, the sights of objects (radius, centre, shape, height blocking),
//! when sights are applied, gap fields and the cover count (table S1), the
//! re-cover sweep, spy satellites and the whole-map reveal and re-cover, the
//! reveal on fire, area reveals, the defeat reveal and the optional regrowth.
//!
//! Our engine keeps the states for every player house (claim 1); the
//! neutral and special houses keep none. The states are held per house as
//! the cover count *k*, the hostile gap count *G* and four marks per legal
//! cell (S1), plus one shared state for every cell that is not legal
//! (`quirks.offmap_cells_shared`, claim 14).

use std::collections::BTreeSet;

use glasnost_map::CellXY;
use glasnost_num::{F64, Rounding};
use glasnost_rules::schema::Ruleset;
use serde::{Deserialize, Serialize};

use crate::rules::{Kind, nearest_double};
use crate::world::{HouseId, HouseRole, ObjectId, Pos, World};

/// Explored mark (S-0409 Terms).
const EXPLORED: u8 = 1; // nocite: a bit of our own state layout
/// Seen mark (S-0409 Terms).
const SEEN: u8 = 2; // nocite: a bit of our own state layout
/// Pending re-cover mark (S-0409 Terms).
const PENDING: u8 = 4; // nocite: a bit of our own state layout
/// Surrounded mark (S-0409 Terms).
const SURROUNDED: u8 = 8; // nocite: a bit of our own state layout

// spec: S-0409 Constants (largest sight radius: 10 cells)
const MAX_RADIUS: i32 = 10;
// spec: S-0409 claim 13 (the re-cover sweep every 120 ticks)
const SWEEP_PERIOD: i32 = 120;
// spec: S-0409 claim 8 (the moving refresh every 15 ticks)
const REFRESH_PERIOD: i32 = 15;
// spec: S-0409 claim 16 (the reveal on fire uses radius 3)
const FIRE_RADIUS: i32 = 3;
// spec: S-0409 Constants (a height level is 104 leptons)
const LEVEL: i32 = 104;
/// Above this many explored cells a reveal's discovery keeps grid-sized
/// tables rather than sets.
const DENSE_REVEAL: usize = 256; // nocite: our tooling's bound (M8 C4 part 6), not game behaviour

/// The game-wide shroud rules (S-0409 Import mapping).
#[derive(Clone, Debug, Default)]
pub struct Rules {
    /// Leptons per sight increase *P* (F1).
    pub leptons_per_sight: i32,
    /// The reveal-by-height flag (claim 7).
    pub reveal_by_height: bool,
    /// Ally reveal (claim 2).
    pub ally_reveal: bool,
    /// The regrowth flag (claim 18).
    pub regrowth: bool,
    /// The regrowth period in ticks (F5); `None` when the rate is 0.
    pub regrowth_period: Option<i32>,
    /// The lobby shroud setting (claim 3).
    pub lobby_shroud: bool,
    /// Spy-satellite flag of each building type (claim 15).
    pub spy_sat: Vec<bool>,
    /// Radar flag of each building type (claim 15, infiltration).
    pub radar: Vec<bool>,
}

impl Rules {
    /// The shroud rules of a resolved ruleset.
    pub fn new(r: &Ruleset, mode: Rounding) -> Self {
        let g = &r.sim.general.economy;
        let reg = r.registries.get("buildings").cloned().unwrap_or_default();
        let bflag = |f: &dyn Fn(&glasnost_rules::sim::BuildingType) -> Option<bool>| -> Vec<bool> {
            reg.iter()
                .map(|n| r.sim.buildings.get(n).and_then(f).unwrap_or(false))
                .collect()
        };
        // spec: S-0409 Import mapping (ShroudRate absent: 4.0 game minutes)
        let rate = g
            .shroud_rate
            .as_ref()
            .map_or_else(|| F64::from_i64(4, mode), |d| d.value(mode));
        // spec: S-0409 F5 (regrowth period = trunc(RZ₅₃(g × 900)) ticks; 900 ticks per game minute)
        let period = rate.mul(F64::from_i64(900, mode), mode).to_i32_common();
        Self {
            // spec: S-0409 Import mapping (LeptonsPerSightIncrease absent: 50; Edge cases: 0 is bad data, 50 is used)
            leptons_per_sight: g
                .leptons_per_sight_increase
                .filter(|&p| p != 0)
                .unwrap_or(50),
            // spec: S-0409 Import mapping (RevealByHeight absent: yes)
            reveal_by_height: g.reveal_by_height.unwrap_or(true),
            // spec: S-0409 Import mapping (AllyReveal absent: yes)
            ally_reveal: g.ally_reveal.unwrap_or(true),
            // spec: S-0409 Import mapping (ShroudGrow absent: no)
            regrowth: g.shroud_grow.unwrap_or(false),
            // spec: S-0409 claim 18 (a regrowth rate other than 0)
            regrowth_period: (!rate.eq_num(F64::ZERO)).then_some(period),
            // spec: S-0409 Import mapping (the lobby shroud setting, absent: yes)
            lobby_shroud: g.shroud.unwrap_or(true),
            // spec: S-0409 Import mapping (SpySat absent: no)
            spy_sat: bflag(&|b| b.econ.spy_sat),
            // spec: S-0409 Import mapping (Radar absent: no)
            radar: bflag(&|b| b.econ.radar),
        }
    }
}

/// One player house's states (S-0409 Terms, S1).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HouseShroud {
    /// Cover count *k* per dense cell index; the last entry is the state
    /// shared by every cell that is not legal.
    pub k: Vec<i16>,
    /// Hostile gap count *G*, as `k`.
    pub g: Vec<i16>,
    /// Explored, seen, pending and surrounded marks, as `k`.
    pub marks: Vec<u8>,
    /// The whole-map-revealed mark (S-0409 Terms).
    pub revealed: bool,
    /// The spy satellite is active (claim 15).
    pub spy_sat: bool,
    /// Objects discovered by the house (S-0409 Terms).
    pub discovered: BTreeSet<ObjectId>,
}

/// The explored state of every player house (S-0409 claim 1).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Shroud {
    /// By house index; `None` for the neutral and special houses.
    pub houses: Vec<Option<HouseShroud>>,
}

/// A sight an object holds (S-0409 Terms, *a sight*): the houses it counts
/// for and the cells it covers, recorded when it was applied.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Applied {
    /// The object's owner when it was applied.
    pub owner: Option<HouseId>,
    /// The houses it counts for (claim 2).
    pub houses: Vec<HouseId>,
    /// The dense indices of its cells.
    pub cells: Vec<u32>,
}

/// A running gap field (claim 11): the houses it is hostile to and its cells.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Gap {
    /// The generator's owner when it started.
    pub owner: Option<HouseId>,
    /// The houses it is hostile to (applied with S1 row 3).
    pub houses: Vec<HouseId>,
    /// Its cells (dense indices; the shared index for cells that are not legal).
    pub cells: Vec<u32>,
    /// With `quirks.gap_end_keeps_cover` off: per house, the cells that were
    /// explored when it started.
    pub explored_at_start: Vec<(HouseId, Vec<u32>)>,
}

/// The sight an object has applied (S-0409 claim 8) and its gap field.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObjectSight {
    /// The sight it holds.
    pub sight: Option<Applied>,
    /// Its running gap field.
    pub gap: Option<Gap>,
    /// Its cell when its sight was last applied (arrival in a cell).
    pub cell: Option<CellXY>,
    /// The tick of its last moving refresh (claim 8).
    pub refreshed: Option<i32>,
}

// ---------------------------------------------------------------------------
// Geometry

/// The coordinate side of the dense index (row-major over the coordinate
/// square, `CellGrid::index`).
fn side(w: &World) -> i32 {
    i32::from(w.cells.footprint().coordinate_side())
}

/// The shared index of the cells that are not legal.
fn shared(w: &World) -> u32 {
    w.cells.grid.len() as u32
}

fn idx(w: &World, c: CellXY) -> Option<u32> {
    w.cells.idx(c).map(|i| i as u32)
}

fn cell_of(w: &World, i: u32) -> CellXY {
    let s = side(w).max(1);
    CellXY::new(i as i32 % s, i as i32 / s)
}

fn level(w: &World, c: CellXY) -> i32 {
    // spec: S-0409 claim 7 (a blocking cell that is not a legal cell has height level 0)
    w.cells.cell(c).map_or(0, |x| i32::from(x.level))
}

/// The simulation lift *L(Z)* of the sight centre (S-0409 F2), pixels: a
/// floating-point lift distinct from S-0101 claim 3's exact drawing lift,
/// because the centre decides explored cells (S-0409 claim 5).
pub fn height_lift(z: i32, mode: Rounding) -> i32 {
    let k = nearest_double("0.14350360082660085"); // spec: S-0409 F2 (a, the binary64 constant nearest 0.14350360082660085 pixels per lepton)
    // spec: S-0409 F2 (L(Z) = trunc(RZ(RZ(RZ(Z·a) + c(Z)) + 0.5)), c(Z) = 1 when Z ≥ 728, else 0)
    let step = i64::from(z >= 728);
    F64::from_i64(i64::from(z), mode)
        .mul(k, mode)
        .add(F64::from_i64(step, mode), mode)
        .add(nearest_double("0.5"), mode)
        .to_i32_common()
}

/// The sight centre of a point (claim 5, F2) and the cell of the unmoved
/// point.
fn centre(w: &World, p: Pos) -> (CellXY, CellXY) {
    // spec: S-0409 F2 (k = trunc(L(Z) / 30); centre = (trunc((X − 256k) / 256), trunc((Y − 256k) / 256)))
    let k = height_lift(p.z, w.rules().mode) / 30;
    let c = CellXY::new((p.x - 256 * k) / 256, (p.y - 256 * k) / 256); // spec: S-0409 F2 (256 leptons per cell)
    (c, p.cell())
}

/// The half-widths of table T1 (claim 6) for |dy| = 0 … r.
// spec: S-0409 T1 (half-widths of the sight shape, radius 1 to 10)
const T1: [&[i32]; 10] = [
    &[1, 1],
    &[2, 2, 1],
    &[3, 3, 2, 1],
    &[4, 4, 3, 3, 1],
    &[5, 5, 4, 4, 3, 1],
    &[6, 6, 5, 5, 4, 3, 1],
    &[7, 7, 6, 6, 5, 5, 3, 1],
    &[8, 8, 7, 7, 6, 6, 5, 3, 1],
    &[9, 9, 8, 8, 7, 7, 6, 5, 3, 1],
    &[10, 10, 9, 9, 8, 8, 7, 7, 5, 3, 1],
];

/// The displacements of the sight shape of radius `r` (claim 6).
pub fn shape(r: i32) -> Vec<(i32, i32)> {
    let mut out = Vec::new();
    if r <= 0 {
        return out;
    }
    let row = T1[(r.min(MAX_RADIUS) - 1) as usize];
    for dy in -(row.len() as i32 - 1)..=(row.len() as i32 - 1) {
        let hw = row[dy.unsigned_abs() as usize];
        for dx in -hw..=hw {
            out.push((dx, dy));
        }
    }
    out
}

/// The step towards the centre of table T2 (claim 7).
pub fn step_towards_centre(dx: i32, dy: i32, quirk: bool) -> (i32, i32) {
    if dx == 0 && dy == 0 {
        return (0, 0);
    }
    if quirk {
        // spec: S-0409 T2 (nine displacements take another step, quirks.height_block_cell)
        const EXCEPTIONS: [((i32, i32), (i32, i32)); 9] = [
            ((5, -2), (1, 0)),
            ((-3, -7), (0, 1)),
            ((3, -7), (0, 1)),
            ((-7, -3), (1, 0)),
            ((7, -3), (-1, 0)),
            ((-7, 3), (1, 0)),
            ((7, 3), (-1, 0)),
            ((-3, 7), (0, -1)),
            ((3, 7), (0, -1)),
        ];
        if let Some(&(_, s)) = EXCEPTIONS.iter().find(|(d, _)| *d == (dx, dy)) {
            return s;
        }
    }
    // spec: S-0409 T2 (the step direction nearest in angle to (−dx, −dy): diagonal when minor/major > tan 22.5°)
    let (ax, ay) = (dx.abs(), dy.abs());
    let (minor, major) = (ax.min(ay), ax.max(ay));
    let diagonal = (minor + major) * (minor + major) > 2 * major * major; // spec: S-0409 T2 (diagonal when (minor + major)² > 2·major², tan 22.5° = √2 − 1)
    let (sx, sy) = (-dx.signum(), -dy.signum());
    if diagonal {
        (sx, sy)
    } else if ax >= ay {
        (sx, 0)
    } else {
        (0, sy)
    }
}

/// The sight radius of an object (claim 4, F1), before the cap; `None`
/// when it reveals nothing.
pub fn sight_radius(w: &World, id: ObjectId) -> Option<i32> {
    let o = w.obj(id)?;
    let field = sight_field(w, id)?;
    let r = w.rules();
    let mode = r.mode;
    let p = r.econ.shroud.leptons_per_sight;
    // spec: S-0409 F1 (q = trunc(Z / P); b = 10q)
    let q = o.pos.z / p;
    let b = q.saturating_mul(10); // spec: S-0409 F1 (b = 10 · q)
    // spec: S-0409 F1 (b′ = b reduced modulo 256 into −128 … 127, quirks.sight_bonus_wrap)
    let b = if r.quirks.sight_bonus_wrap {
        i32::from(b as i8)
    } else {
        b
    };
    // spec: S-0409 F1 (r₀ = trunc(RZ₅₃(S × RZ₅₃(RZ₅₃(b′ × 0.01) + 1))))
    let factor = F64::from_i64(i64::from(b), mode)
        .mul(nearest_double("0.01"), mode)
        .add(F64::from_i64(1, mode), mode);
    let mut rad = F64::from_i64(i64::from(field), mode)
        .mul(factor, mode)
        .to_i32_common();
    if crate::combat::veterancy::has_ability(w, id, crate::combat::data::Ability::Sight) {
        let f = r.combat.general.veteran_sight;
        // spec: S-0311 F4 (r′ = trunc(RZ₅₃(r × f)) when f ≠ 0)
        if !f.eq_num(F64::ZERO) {
            rad = F64::from_i64(i64::from(rad), mode)
                .mul(f, mode)
                .to_i32_common();
        }
    }
    // spec: S-0409 claim 4 (a radius of 0 or less reveals nothing; a negative r₀ from the wrap reveals nothing, Edge cases)
    (rad > 0).then_some(rad)
}

/// The type's sight field *S* (claim 4).
fn sight_field(w: &World, id: ObjectId) -> Option<i32> {
    let o = w.obj(id)?;
    let r = w.rules();
    match o.kind {
        Kind::Building => r
            .buildings
            .get(o.type_index as usize)
            // spec: S-0409 Import mapping (Sight absent: 0)
            .map(|b| b.raw.econ.sight.unwrap_or(0)),
        Kind::Terrain => None,
        _ => r.unit(o.kind, o.type_index).map(|u| u.sight),
    }
}

fn reveal_to_all(w: &World, id: ObjectId) -> bool {
    let Some(o) = w.obj(id) else { return false };
    let r = w.rules();
    // spec: S-0409 Import mapping (RevealToAll absent: no)
    match o.kind {
        Kind::Building => r
            .buildings
            .get(o.type_index as usize)
            .and_then(|b| b.raw.econ.reveal_to_all)
            .unwrap_or(false),
        Kind::Terrain => false,
        _ => r
            .unit(o.kind, o.type_index)
            .and_then(|u| u.raw.econ.reveal_to_all)
            .unwrap_or(false),
    }
}

/// The cells of a sight or area reveal of radius `r` from point `p`
/// (claims 5 to 7); empty when the centre is not legal (with
/// `quirks.sight_centre_off_map`).
pub fn sight_cells(w: &World, p: Pos, r: i32) -> Vec<u32> {
    sight_cells_with(w, p, r, true)
}

/// The cells of claims 5 and 6 around `p` with radius `r`, with claim 7's
/// high-ground test when `height_test` is set (and the reveal-by-height
/// flag is on). The psychic reveal skips that test (claim 17, S-0610 claim
/// 14).
fn sight_cells_with(w: &World, p: Pos, r: i32, height_test: bool) -> Vec<u32> {
    let mut out = Vec::new();
    if r <= 0 {
        return out;
    }
    let q = w.rules().quirks;
    let (c, unmoved) = centre(w, p);
    // spec: S-0409 claim 5 (a centre that is not a legal cell reveals nothing, quirks.sight_centre_off_map)
    if q.sight_centre_off_map && idx(w, c).is_none() {
        return out;
    }
    let observer = p.z / LEVEL; // spec: S-0409 Terms (observer level = trunc(Z / 104))
    let block = height_test && w.rules().econ.shroud.reveal_by_height;
    for (dx, dy) in shape(r.min(MAX_RADIUS)) {
        let t = CellXY::new(c.x + dx, c.y + dy);
        // spec: S-0409 claim 6 (only legal cells are covered; no sight wraps)
        let Some(i) = idx(w, t) else { continue };
        if block {
            let (sx, sy) = step_towards_centre(dx, dy, q.height_block_cell);
            // spec: S-0409 F3 (blocking cell = T + s + (2 + cₓ − cx, 2 + c_y − cy); quirks.height_block_cell off: T + s)
            let b = if q.height_block_cell {
                CellXY::new(
                    t.x + sx + 2 + unmoved.x - c.x,
                    t.y + sy + 2 + unmoved.y - c.y,
                )
            } else {
                CellXY::new(t.x + sx, t.y + sy)
            };
            // spec: S-0409 claim 7 (hidden when the blocking cell's level is above the observer level + 3)
            if level(w, b) > observer + 3 {
                continue;
            }
        }
        out.push(i);
    }
    out
}

/// The point of an object for its sight, area reveal and gap field
/// (claim 5, `quirks.building_sight_top_left`).
fn sight_point(w: &World, id: ObjectId) -> Pos {
    let Some(o) = w.obj(id) else {
        return Pos::default();
    };
    // spec: S-0409 claim 5 (a building: the centre of its top-left foundation cell, quirks.building_sight_top_left; off: its reference point)
    if o.kind == Kind::Building && !w.rules().quirks.building_sight_top_left {
        w.reference_point(id)
    } else {
        o.pos
    }
}

/// A gap field's cells around a centre (claims 11 and 14, F4).
fn gap_cells(w: &World, centre: CellXY, field: i32) -> Vec<u32> {
    let q = w.rules().quirks;
    // spec: S-0409 claim 11 (the gap radius wrapped into −128 … 127, quirks.gap_radius_wrap)
    let r = if q.gap_radius_wrap {
        i32::from(field as i8)
    } else {
        field
    };
    let mut out = Vec::new();
    if r < 0 {
        return out;
    }
    let lim = (r + 1) * (r + 1);
    for dy in -r..=r {
        for dx in -r..=r {
            // spec: S-0409 F4 (covered when dx² + dy² < (R + 1)²)
            if dx * dx + dy * dy >= lim {
                continue;
            }
            let (mut x, mut y) = (centre.x + dx, centre.y + dy);
            let max = 511; // spec: S-0409 claim 14 (the coordinate range 0 … 511)
            if !(0..=max).contains(&x) {
                if !q.shroud_point_cell_wrap {
                    // spec: S-0409 Edge cases (quirks.shroud_point_cell_wrap off: such a cell changes nothing)
                    continue;
                }
                // spec: S-0409 claim 14 ((x, y) is read as (x mod 512, y + ⌊x / 512⌋))
                y += x.div_euclid(512);
                x = x.rem_euclid(512); // spec: S-0409 claim 14 (x mod 512)
            }
            if !(0..=max).contains(&y) {
                continue;
            }
            match idx(w, CellXY::new(x, y)) {
                Some(i) => out.push(i),
                // spec: S-0409 claim 14 (a cell that is not legal: the shared state, quirks.offmap_cells_shared)
                None if q.offmap_cells_shared => out.push(shared(w)),
                None => {}
            }
        }
    }
    out
}

// ---------------------------------------------------------------------------
// State and the rows of S1

fn hs(w: &World, h: HouseId) -> Option<&HouseShroud> {
    w.econ.shroud.houses.get(usize::from(h.0))?.as_ref()
}

fn hs_mut(w: &mut World, h: HouseId) -> Option<&mut HouseShroud> {
    w.econ.shroud.houses.get_mut(usize::from(h.0))?.as_mut()
}

/// The player houses (claim 1).
fn player_houses(w: &World) -> Vec<HouseId> {
    (0..w.houses.len())
        .filter(|&i| w.econ.shroud.houses.get(i).is_some_and(Option::is_some))
        .map(|i| HouseId(i as u8))
        .collect()
}

impl HouseShroud {
    fn new(n: usize) -> Self {
        Self {
            // spec: S-0409 claim 3 (every cover count starts at 1, every hostile gap count at 0, no mark set)
            k: vec![1; n + 1],
            g: vec![0; n + 1],
            marks: vec![0; n + 1],
            revealed: false,
            spy_sat: false,
            discovered: BTreeSet::new(),
        }
    }

    fn has(&self, i: usize, m: u8) -> bool {
        self.marks[i] & m != 0
    }

    fn set(&mut self, i: usize, m: u8, on: bool) {
        if on {
            self.marks[i] |= m;
        } else {
            self.marks[i] &= !m;
        }
    }

    /// S1 row 1: a sight of H is added over the cell. Returns whether the
    /// cell became explored.
    fn add_sight(&mut self, i: usize) -> bool {
        let k = self.k[i];
        // spec: S-0409 S1 row 1 (k′ = −1 if k = 1, otherwise k − 1)
        let k2 = if k == 1 { -1 } else { k.saturating_sub(1) };
        self.k[i] = k2;
        if k2 > 0 {
            return false;
        }
        let was = self.has(i, EXPLORED);
        // spec: S-0409 S1 row 1 (m cleared only if the cell was already explored and marked surrounded)
        if was && self.has(i, SURROUNDED) {
            self.set(i, PENDING, false);
        }
        self.set(i, EXPLORED | SEEN | SURROUNDED, true);
        !was
    }

    /// S1 row 2: a sight of H is removed from the cell.
    fn remove_sight(&mut self, i: usize) {
        let k = self.k[i];
        // spec: S-0409 S1 row 2 (k⁺ = 1 if k = −1, otherwise k + 1; k′ = min(k⁺, G))
        let kp = if k == -1 { 1 } else { k.saturating_add(1) };
        let k2 = kp.min(self.g[i]);
        self.k[i] = k2;
        // spec: S-0409 S1 row 2 (k from 0 or less to above 0: m set)
        if k <= 0 && k2 > 0 {
            self.set(i, PENDING, true);
        }
    }

    /// S1 row 3: a gap field hostile to H starts over the cell.
    fn gap_start(&mut self, i: usize) {
        let k = self.k[i];
        // spec: S-0409 S1 row 3 (k′ = k + 1 if k = 0 or k ≥ 2, otherwise k; G′ = G + 1)
        let k2 = if k == 0 || k >= 2 {
            k.saturating_add(1)
        } else {
            k
        };
        self.k[i] = k2;
        self.g[i] = self.g[i].saturating_add(1);
        // spec: S-0409 S1 row 3 (k′ ≥ 1: unexplored at once; seen unchanged)
        if k2 >= 1 {
            self.set(i, EXPLORED, false);
            self.set(i, SURROUNDED, false);
        }
    }

    /// S1 row 5: a gap field hostile to H stops over the cell. Returns
    /// whether the cell became explored.
    fn gap_stop(&mut self, i: usize) -> bool {
        let g2 = self.g[i].saturating_sub(1);
        self.g[i] = g2;
        // spec: S-0409 S1 row 5 (k′ = k − 1 if the spy satellite is active and G′ ≤ 0)
        if self.spy_sat && g2 <= 0 {
            self.k[i] = self.k[i].saturating_sub(1);
            if self.k[i] <= 0 {
                let was = self.has(i, EXPLORED);
                self.set(i, EXPLORED | SURROUNDED, true);
                return !was;
            }
        }
        false
    }

    /// S1 row 6, plain reveal. Returns whether the cell became explored.
    fn plain_reveal(&mut self, i: usize) -> bool {
        let was = self.has(i, EXPLORED);
        // spec: S-0409 S1 row 6 (plain reveal: explored and marked surrounded, not seen; with k > 0, m set)
        self.set(i, EXPLORED | SURROUNDED, true);
        if self.k[i] > 0 {
            self.set(i, PENDING, true);
        }
        !was
    }

    /// S1 row 6, area reveal: k₁ from k as row 1 gives, k₂ from k₁ as row
    /// 2 gives, k′ = min(k₂, G); the pending mark as rows 1 and 2 change it.
    fn area_reveal(&mut self, i: usize) -> bool {
        // spec: S-0409 S1 row 6 (r10: k₁ = −1 when k = 1, else k − 1; k₂ = 1 when k₁ = −1, else k₁ + 1; k′ = min(k₂, G), G unchanged)
        // spec: S-0409 claims 12 and 17 (a cell inside a hostile gap field is marked pending unless a sight of the house covers it; outside, the cells stay explored)
        let became = self.add_sight(i);
        self.remove_sight(i);
        // spec: S-0409 S1 row 6 (area reveal: explored, seen and marked surrounded)
        let was = self.has(i, EXPLORED);
        self.set(i, EXPLORED | SEEN | SURROUNDED, true);
        became || !was
    }

    /// S1 row 4: the re-cover sweep on one cell.
    fn sweep(&mut self, i: usize) {
        if self.has(i, PENDING) {
            // spec: S-0409 S1 row 4 (unexplored and unseen; m cleared)
            self.set(i, EXPLORED | SEEN | PENDING | SURROUNDED, false);
        }
    }
}

/// Whether a cell is explored for a house (S-0409 Terms): the query that
/// placement (S-0406 claims 5 and 6) and the AI read. A house that keeps no
/// state (neutral, special) is answered yes.
pub fn explored(w: &World, house: HouseId, cell: CellXY) -> bool {
    let Some(s) = hs(w, house) else { return true };
    match idx(w, cell) {
        Some(i) => s.has(i as usize, EXPLORED),
        // spec: S-0409 claim 14 (cells that are not legal share one state, quirks.offmap_cells_shared; off: never explored)
        None => w.rules().quirks.offmap_cells_shared && s.has(shared(w) as usize, EXPLORED),
    }
}

/// Whether a cell is seen for a house (S-0409 Terms).
pub fn seen(w: &World, house: HouseId, cell: CellXY) -> bool {
    let Some(s) = hs(w, house) else { return true };
    idx(w, cell).is_some_and(|i| s.has(i as usize, SEEN))
}

/// Whether a cell holds a pending re-cover mark for a house (S-0409 Terms).
pub fn pending(w: &World, house: HouseId, cell: CellXY) -> bool {
    hs(w, house).is_some_and(|s| idx(w, cell).is_some_and(|i| s.has(i as usize, PENDING)))
}

/// The cover count *k* and hostile gap count *G* of a cell (S1), for tests.
pub fn counts(w: &World, house: HouseId, cell: CellXY) -> Option<(i32, i32)> {
    let s = hs(w, house)?;
    let i = idx(w, cell)? as usize;
    Some((i32::from(s.k[i]), i32::from(s.g[i])))
}

/// Whether the house's whole-map-revealed mark is set (S-0409 Terms).
pub fn whole_map_revealed(w: &World, house: HouseId) -> bool {
    hs(w, house).is_some_and(|s| s.revealed)
}

/// Whether the house's spy satellite is active (claim 15).
pub fn spy_satellite_active(w: &World, house: HouseId) -> bool {
    hs(w, house).is_some_and(|s| s.spy_sat)
}

/// Whether the object is discovered by the house (S-0409 Terms, claim 20):
/// its discovery mark alone; ownership does not count. A house that keeps
/// no state (neutral, special) is answered yes.
pub fn discovered(w: &World, house: HouseId, id: ObjectId) -> bool {
    // spec: S-0409 claim 20 (ownership alone does not set the discovery mark)
    hs(w, house).is_none_or(|s| s.discovered.contains(&id))
}

/// The candidate cells of S-0110 claim 8's point test for a world point,
/// before legality and wrap: one cell for an even height count, two for an
/// odd one. `negative_odd_pair` is `quirks.shroud_negative_odd_pair`. The
/// test always follows the height-0 grid (`quirks.shroud_on_flat_grid` on),
/// so the pair switch always applies here.
pub fn point_candidates(p: Pos, negative_odd_pair: bool) -> (CellXY, Option<CellXY>) {
    let u = p.z / LEVEL; // spec: S-0110 claim 8 (u = trunc(Z / 104), heights below 0 included)
    let (cx, cy) = (p.x / 256, p.y / 256); // spec: S-0110 claim 8 (trunc(X / 256), trunc(Y / 256))
    let at = |n: i32| CellXY::new(cx - n, cy - n);
    // spec: S-0110 claim 8 (even or odd u; a negative odd u leaves the remainder −1, not 0)
    if u % 2 == 0 {
        // spec: S-0110 claim 8 (even u: the one cell (c_x − u/2, c_y − u/2), exact for negative u too)
        return (at(u / 2), None);
    }
    let k = if u < 0 && negative_odd_pair {
        // spec: S-0110 claim 8, Edge cases (odd u < 0: k = (u + 3)/2, quirks.shroud_negative_odd_pair)
        (u + 3) / 2
    } else {
        // spec: S-0110 claim 8, Edge cases (odd u > 0, or the switch off: k = (u + 1)/2)
        (u + 1) / 2
    };
    // spec: S-0110 claim 8 (odd u: cells (c_x − k, c_y − k) and (c_x − k + 1, c_y − k + 1))
    (at(k), Some(at(k - 1)))
}

/// [`point_candidates`] under the renderer's `quirks.shroud_on_flat_grid`
/// (S-0110 Edge cases), for the radar, which shares it (S-0410 claim 6):
/// off, the cell under the point (trunc(X / 256), trunc(Y / 256)) is the
/// one candidate, whatever the height and the pair switch.
pub fn point_candidates_on(
    p: Pos,
    negative_odd_pair: bool,
    on_flat_grid: bool,
) -> (CellXY, Option<CellXY>) {
    if !on_flat_grid {
        // spec: S-0110 Edge cases (quirks.shroud_on_flat_grid off: the point test uses the cell under
        // the point directly; its direct-cell test takes precedence over the pair switch)
        return (CellXY::new(p.x / 256, p.y / 256), None);
    }
    point_candidates(p, negative_odd_pair)
}

/// Whether a world point is shrouded for a house (S-0110 claim 8, with
/// `quirks.shroud_point_cell_wrap` and `quirks.shroud_negative_odd_pair`).
pub fn point_shrouded(w: &World, house: HouseId, p: Pos) -> bool {
    point_shrouded_on(w, house, p, true)
}

/// [`point_shrouded`] under the renderer's `quirks.shroud_on_flat_grid`
/// (S-0110 Edge cases; the radar shares it, S-0410 claim 6).
pub fn point_shrouded_on(w: &World, house: HouseId, p: Pos, on_flat_grid: bool) -> bool {
    let (a, b) = point_candidates_on(p, w.rules().quirks.shroud_negative_odd_pair, on_flat_grid);
    let cand = |c: CellXY| candidate_explored(w, house, c.x, c.y);
    // spec: S-0110 claim 8 (even u: one candidate; odd u: shrouded when neither of two is explored)
    !cand(a) && !b.is_some_and(cand)
}

/// A candidate cell of the point test (S-0110 claim 8, Edge cases).
fn candidate_explored(w: &World, house: HouseId, mut x: i32, mut y: i32) -> bool {
    let max = 511; // spec: S-0110 Edge cases (the coordinate range 0 … 511)
    if !(0..=max).contains(&x) {
        if !w.rules().quirks.shroud_point_cell_wrap {
            return false;
        }
        // spec: S-0110 Edge cases ((x mod 512, y + ⌊x / 512⌋) when that row lies in 0 … 511)
        y += x.div_euclid(512);
        x = x.rem_euclid(512); // spec: S-0110 Edge cases (x mod 512)
    }
    if !(0..=max).contains(&y) {
        return false;
    }
    explored(w, house, CellXY::new(x, y))
}

// ---------------------------------------------------------------------------
// Discovery (S-0409 Terms, *discovered by H*)

/// The cover-piece key of a cell (S-0110 claims 1 and 2): 15 when it is not
/// explored, else its reduced set of covered neighbours as a mask. The
/// cells `flipped` tells (by dense index) read as unexplored.
fn piece_key(w: &World, h: HouseId, c: CellXY, flipped: impl Fn(u32) -> bool) -> u16 {
    let ex = |x: i32, y: i32| -> bool {
        let cc = CellXY::new(x, y);
        let e = explored(w, h, cc);
        // The state before the reveal: the cells it explored read as unexplored.
        let f = idx(w, cc).is_some_and(&flipped);
        e && !f
    };
    if !ex(c.x, c.y) {
        return 0x100; // nocite: a key of our own for "fully covered"
    }
    let (x, y) = (c.x, c.y);
    // spec: S-0110 Terms (map neighbours N, E, S, W and NW, NE, SE, SW)
    let n = !ex(x, y - 1);
    let e = !ex(x + 1, y);
    let s = !ex(x, y + 1);
    let wv = !ex(x - 1, y);
    let nw = !ex(x - 1, y - 1) && !n && !wv;
    let ne = !ex(x + 1, y - 1) && !n && !e;
    let se = !ex(x + 1, y + 1) && !s && !e;
    let sw = !ex(x - 1, y + 1) && !s && !wv;
    // spec: S-0110 claim 2 (the reduced set: covered sides, and covered corners neither of whose adjacent sides is covered)
    [n, e, s, wv, nw, ne, se, sw]
        .iter()
        .enumerate()
        .fold(0u16, |m, (b, &on)| m | (u16::from(on) << b))
}

/// Marks the objects discovered by a reveal for `h` that explored the cells
/// `flipped` (S-0409 Terms).
fn discover(w: &mut World, h: HouseId, flipped: &BTreeSet<u32>) {
    if flipped.is_empty() {
        return;
    }
    // Each cell's two keys are the same whichever flipped neighbour asks for
    // them, so each cell is compared once and `changed` lists each changed
    // cell once (its order does not matter: the discovered objects are a
    // set). A large reveal (a defeat's whole-map reveal) keeps its cells and
    // the compared cells in grid-sized tables, a small one (a unit's sight)
    // in sets (our tooling for speed, M8 C4 part 6).
    let dense = flipped.len() > DENSE_REVEAL;
    let cells = if dense { w.cells.grid.len() } else { 0 };
    let mut was = vec![false; cells];
    for &i in flipped.iter().filter(|_| dense) {
        was[i as usize] = true;
    }
    let was = |i: u32| {
        if dense {
            was.get(i as usize).copied().unwrap_or(false)
        } else {
            flipped.contains(&i)
        }
    };
    let mut asked = vec![false; cells];
    let mut asked_few = BTreeSet::new();
    let mut changed = Vec::new();
    for &i in flipped {
        let c = cell_of(w, i);
        for dy in -1..=1 {
            for dx in -1..=1 {
                let a = CellXY::new(c.x + dx, c.y + dy);
                let Some(ai) = idx(w, a) else {
                    continue;
                };
                let first = match asked.get_mut(ai as usize) {
                    Some(seen) => !std::mem::replace(seen, true),
                    None => asked_few.insert(ai),
                };
                if first && piece_key(w, h, a, was) != piece_key(w, h, a, |_| false) {
                    changed.push(a);
                }
            }
        }
    }
    let mut found = Vec::new();
    for a in changed {
        // spec: S-0409 Terms (the object's cell n steps map south-east, n from 0 to 6, with 2n − 1 ≤ h ≤ 2n + 1)
        for n in 0..=6 {
            let x = CellXY::new(a.x + n, a.y + n);
            if idx(w, x).is_none() {
                continue;
            }
            let lv = level(w, x);
            if !(2 * n - 1 <= lv && lv <= 2 * n + 1) {
                continue;
            }
            if let Some(id) = nearest_in_cell(w, x) {
                found.push(id);
            }
        }
    }
    if let Some(s) = hs_mut(w, h) {
        s.discovered.extend(found);
    }
}

/// The object a reveal selects in a cell (claim 20): among the infantry,
/// vehicles, aircraft and buildings of the cell's ground layer, the one with
/// the smallest selection distance (F6). Ties go to the first in the
/// layer's listing order, which is the claim's placement priority: the
/// non-buildings most recently placed first, then the buildings in
/// placement order (S-0209 claim 13).
fn nearest_in_cell(w: &World, c: CellXY) -> Option<ObjectId> {
    let table = w.rules().quirks.table_root_distance;
    let mode = w.rules().mode;
    // spec: S-0409 claim 20 (only the ground layer; objects on the bridge-deck layer are not selected)
    w.cells
        .list(c, crate::world::Layer::Ground)
        .iter()
        .filter_map(|&id| w.obj(id).map(|o| (id, o)))
        // spec: S-0409 claim 20 (only infantry, vehicles, aircraft and buildings compete)
        .filter(|(_, o)| o.kind != Kind::Terrain)
        .map(|(id, o)| {
            // spec: S-0409 F6 (a = X mod 256, b = Y mod 256: from the cell's top corner)
            let a = u64::from(o.pos.x.rem_euclid(256).unsigned_abs());
            let b = u64::from(o.pos.y.rem_euclid(256).unsigned_abs());
            // spec: S-0409 F6 (⌊r(a² + b²)⌋, the table root under quirks.table_root_distance)
            (
                id,
                glasnost_num::geom::root_floor(a * a + b * b, table, mode),
            )
        })
        // spec: S-0409 claim 20 (equal distances: the first in placement priority)
        .fold(None, |best: Option<(ObjectId, u64)>, (id, d)| match best {
            Some((_, bd)) if bd <= d => best,
            _ => Some((id, d)),
        })
        .map(|(id, _)| id)
}

/// Placement discovery (claim 20): an object placed in a cell's ground or
/// bridge-deck layer is marked discovered by every player house, whatever
/// its owner, the shroud or the fog; placed outside the usable area, the
/// mark is cleared when its placement finishes.
fn placed(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id) else { return };
    // spec: S-0409 claim 20 (placement outside the usable area clears the mark when the placement finishes)
    let usable = w.cells.usable(o.pos.cell());
    // spec: S-0409 claim 20 (every player house, as for the local player, under claim 1)
    for s in w.econ.shroud.houses.iter_mut().flatten() {
        if usable {
            s.discovered.insert(id);
        } else {
            s.discovered.remove(&id);
        }
    }
}

// ---------------------------------------------------------------------------
// Applying sights, reveals and gap fields

/// The houses an object's sight counts for (claim 2, claim 8's
/// reveal-to-all).
fn sight_houses(w: &World, id: ObjectId) -> Vec<HouseId> {
    let Some(o) = w.obj(id) else {
        return Vec::new();
    };
    let Some(owner) = o.owner else {
        return Vec::new();
    };
    let r = w.rules();
    let country = w.houses.get(usize::from(owner.0)).map(|h| h.country);
    // spec: S-0409 claim 2 (an object whose owner's country is passive reveals nothing, before reveal-to-all)
    if country.is_some_and(|c| r.countries.get(c as usize).is_some_and(|x| x.passive)) {
        return Vec::new();
    }
    let all = o.kind == Kind::Building && reveal_to_all(w, id);
    let ally = r.econ.shroud.ally_reveal;
    player_houses(w)
        .into_iter()
        // spec: S-0409 claims 2 and 8 (the owner, its allies with ally reveal on (the owner's side), or every house for reveal-to-all)
        .filter(|&h| all || h == owner || (ally && w.allied(Some(owner), Some(h))))
        .collect()
}

/// Applies S1 row 1 to cells for one house and marks discoveries.
fn add_cells(w: &mut World, h: HouseId, cells: &[u32]) {
    let mut flipped = BTreeSet::new();
    if let Some(s) = hs_mut(w, h) {
        for &i in cells {
            if s.add_sight(i as usize) {
                flipped.insert(i);
            }
        }
    }
    discover(w, h, &flipped);
}

fn remove_cells(w: &mut World, h: HouseId, cells: &[u32]) {
    if let Some(s) = hs_mut(w, h) {
        for &i in cells {
            s.remove_sight(i as usize);
        }
    }
}

/// Adds an object's sight (claim 8).
fn add_sight(w: &mut World, id: ObjectId) {
    add_sight_with(w, id, None);
}

/// Adds an object's sight, with the radius `radius` in cells instead of
/// the computed one when given (a camera, S-0513 claim 44).
fn add_sight_with(w: &mut World, id: ObjectId, radius: Option<i32>) {
    let Some(o) = w.obj(id) else { return };
    let (owner, cell) = (o.owner, o.pos.cell());
    let houses = sight_houses(w, id);
    let cells = radius
        .or_else(|| sight_radius(w, id))
        .map_or_else(Vec::new, |r| sight_cells(w, sight_point(w, id), r));
    for &h in &houses {
        add_cells(w, h, &cells);
    }
    if let Some(ob) = w.obj_mut(id) {
        ob.econ.sight.sight = Some(Applied {
            owner,
            houses,
            cells,
        });
        ob.econ.sight.cell = Some(cell);
    }
}

/// Brings an object's sight to its present position: a walking or driving
/// unit at each arrival in a cell (S-0409 claim 8 and Terms, the movement
/// arrival update), a jumpjet that has touched down (S-0502 claim 15). An
/// object that holds no sight yet adds it in its own turn.
pub fn refresh_sight(w: &mut World, id: ObjectId) {
    if !may_hold(w, id) {
        return;
    }
    if w.obj(id).is_some_and(|o| o.econ.sight.sight.is_some()) {
        remove_sight(w, id);
        add_sight(w, id);
    }
}

/// Removes an object's sight (claim 8).
fn remove_sight(w: &mut World, id: ObjectId) {
    let Some(a) = w.obj_mut(id).and_then(|o| o.econ.sight.sight.take()) else {
        return;
    };
    for &h in &a.houses {
        remove_cells(w, h, &a.cells);
    }
}

/// A camera (S-0513 claim 44): the object's sight is replaced by one
/// centred on its own position, with its height, whose radius in cells is
/// `radius` instead of the computed sight radius; a radius of 0 or less
/// gives the ordinary radius. It is a sight held for the owner, not an
/// area reveal, so the ordinary refresh may later restore the ordinary
/// radius and the sight ends with the object (claim 8).
pub fn camera(w: &mut World, id: ObjectId, radius: i32) {
    if !may_hold(w, id) {
        // spec: S-0409 claim 10 (no sight while outside the usable area)
        return;
    }
    // spec: S-0513 claim 44 (a camera replaces the plane's sight)
    remove_sight(w, id);
    // spec: S-0513 claim 44 (the radius is the primary weapon's damage; a damage of 0 gives the ordinary radius; the cap of 10 is S-0409 claim 6's, in sight_cells)
    add_sight_with(w, id, (radius > 0).then_some(radius));
}

/// Whether an object may hold a sight now (claims 8 and 10): on the map
/// and inside the usable area.
fn may_hold(w: &World, id: ObjectId) -> bool {
    w.obj(id).is_some_and(|o| {
        o.on_map && o.kind != Kind::Terrain && o.health > 0 && w.cells.usable(o.pos.cell())
    })
}

/// The shroud's reaction to an object put on the map (claim 8): a building
/// adds its sight when it is placed, with its one-time area reveal; other
/// objects add theirs in their own turn.
pub fn entered_map(w: &mut World, id: ObjectId) {
    // spec: S-0409 claim 20 (placing an object in a ground or bridge-deck layer marks it discovered)
    placed(w, id);
    let Some(o) = w.obj(id) else { return };
    if o.kind != Kind::Building {
        return;
    }
    if !may_hold(w, id) {
        return;
    }
    // spec: S-0409 claim 8 (a building's one-time area reveal with its plain sight field when it is placed)
    let plain = sight_field(w, id).unwrap_or(0);
    let p = sight_point(w, id);
    let cells = sight_cells(w, p, plain.min(MAX_RADIUS));
    for h in sight_houses(w, id) {
        area_cells(w, h, &cells);
    }
    // spec: S-0409 claim 8 (the placement of a building adds its sight when it is placed)
    add_sight(w, id);
}

/// The shroud's reaction to an object leaving the map (claims 8 and 11).
pub fn leaving_map(w: &mut World, id: ObjectId, for_good: bool) {
    // spec: S-0409 claim 8 (it removes its sight when it leaves the map)
    remove_sight(w, id);
    // spec: S-0409 claim 11 (the gap field stops when the generator is removed from the map or destroyed)
    stop_gap(w, id);
    if for_good {
        for s in w.econ.shroud.houses.iter_mut().flatten() {
            s.discovered.remove(&id);
        }
    }
}

/// The shroud's reaction to a change of owner (claims 8 and 11).
pub fn owner_changed(w: &mut World, id: ObjectId) {
    // A running gap field is taken back under the old owner and started under the new one.
    if w.obj(id).is_some_and(|o| o.econ.sight.gap.is_some()) {
        stop_gap(w, id);
        start_gap(w, id);
    }
}

/// An object's own turn (claim 8): it adds its sight once it may hold one,
/// moves it every 15 ticks while it moves, and follows a change of owner.
/// The move on each arrival in a cell of a walking or driving unit is made
/// at the arrival update itself ([`refresh_sight`]).
pub fn object_turn(w: &mut World, id: ObjectId, moved: bool) {
    let t = w.tick;
    if !may_hold(w, id) {
        // spec: S-0409 claim 10 (no sight while outside the usable area)
        if w.obj(id).is_some_and(|o| o.econ.sight.sight.is_some()) {
            remove_sight(w, id);
        }
        return;
    }
    let Some(o) = w.obj(id) else { return };
    let refreshed = o.econ.sight.refreshed;
    let Some(applied_owner) = o.econ.sight.sight.as_ref().map(|a| a.owner) else {
        // spec: S-0409 claim 8 (it adds its sight when it enters the map, in its own turn)
        add_sight(w, id);
        return;
    };
    // A walking or driving unit's arrival in a cell is not judged here: the
    // locomotor's arrival update calls `refresh_sight` (claim 8 and Terms).
    // spec: S-0409 claim 8 (a moving object refreshes when at least 15 ticks have passed since its last such refresh)
    let refresh = moved && refreshed.is_none_or(|r| t.wrapping_sub(r) >= REFRESH_PERIOD);
    // spec: S-0409 claim 8 (a change of owner: removed for the old owner, added for the new one)
    let owner = applied_owner != o.owner;
    if refresh && let Some(ob) = w.obj_mut(id) {
        ob.econ.sight.refreshed = Some(t);
    }
    if refresh || owner {
        remove_sight(w, id);
        add_sight(w, id);
    }
}

/// Applies S1 row 6 (area reveal) to cells for one house.
fn area_cells(w: &mut World, h: HouseId, cells: &[u32]) {
    let mut flipped = BTreeSet::new();
    if let Some(s) = hs_mut(w, h) {
        for &i in cells {
            if s.area_reveal(i as usize) {
                flipped.insert(i);
            }
        }
    }
    discover(w, h, &flipped);
}

/// An area reveal that another spec orders (claim 17): around `p` with
/// radius `r` (10 above 10, nothing at 0 or less), made for `made_for` and
/// counted for its allies with ally reveal on (claim 2).
pub fn area_reveal(w: &mut World, made_for: HouseId, p: Pos, r: i32) {
    let cells = sight_cells(w, p, r.min(MAX_RADIUS));
    reveal_for(w, made_for, &cells);
}

/// The psychic reveal's area reveal (claim 17 and S-0610 claim 14): as
/// [`area_reveal`], but claim 7's high-ground test is not made, so every
/// legal cell of the shape is explored and seen whatever the
/// reveal-by-height flag says.
pub fn psychic_reveal(w: &mut World, made_for: HouseId, p: Pos, r: i32) {
    // spec: S-0409 claim 17 and S-0610 claim 14 (claims 5 and 6 with the radius capped at 10 and nothing at 0; claim 7's high-ground test skipped)
    let cells = sight_cells_with(w, p, r.min(MAX_RADIUS), false);
    reveal_for(w, made_for, &cells);
}

/// Applies an area reveal's cells for the house it is made for and the
/// houses that claim 2 lets share it.
fn reveal_for(w: &mut World, made_for: HouseId, cells: &[u32]) {
    let ally = w.rules().econ.shroud.ally_reveal;
    for h in player_houses(w) {
        // spec: S-0409 claim 2 (an area reveal counts for the house it is made for and, with ally reveal, its allies)
        if h == made_for || (ally && w.allied(Some(made_for), Some(h))) {
            area_cells(w, h, cells);
        }
    }
}

/// The reveal on fire (claim 16): `firer` fired a shot from a weapon named
/// `weapon` at `target`.
pub fn fired(w: &mut World, firer: ObjectId, weapon: &str, target: crate::world::Target) {
    // spec: S-0409 Import mapping (revision 16: RevealOnFire absent: yes)
    let flag = w
        .rules()
        .weapons
        .get(weapon)
        .and_then(|x| x.reveal_on_fire)
        .unwrap_or(true);
    if !flag {
        return;
    }
    let crate::world::Target::Object(t) = target else {
        return;
    };
    // spec: S-0409 claims 2 and 16 (it counts only for the house that owns the target)
    let Some(h) = w.obj(t).and_then(|o| o.owner) else {
        return;
    };
    if hs(w, h).is_none() {
        return;
    }
    let Some(f) = w.obj(firer) else { return };
    let (fk, fo) = (f.kind, f.owner);
    // spec: S-0409 claim 16 (centred on the firer's reference point, not its position)
    let p = w.reference_point(firer);
    let shrouded = point_shrouded(w, h, p);
    // spec: S-0409 claim 16 table (the reveal happens unless: not shrouded and owned or discovered by H; shrouded and an aircraft of H)
    let skip = if shrouded {
        fk == Kind::Aircraft && fo == Some(h)
    } else {
        fo == Some(h) || discovered(w, h, firer)
    };
    if skip {
        return;
    }
    let cells = sight_cells(w, p, FIRE_RADIUS);
    if w.rules().quirks.fire_reveal_expires {
        // spec: S-0409 claim 16 (a plain reveal, quirks.fire_reveal_expires)
        let mut flipped = BTreeSet::new();
        if let Some(s) = hs_mut(w, h) {
            for &i in &cells {
                if s.plain_reveal(i as usize) {
                    flipped.insert(i);
                }
            }
        }
        discover(w, h, &flipped);
    } else {
        // spec: S-0409 Edge cases (quirks.fire_reveal_expires off: an area reveal)
        area_cells(w, h, &cells);
    }
}

/// Whether a building is a gap generator (S-0409 Terms) and its radius field.
fn gap_field(w: &World, id: ObjectId) -> Option<i32> {
    let o = w.obj(id)?;
    if o.kind != Kind::Building {
        // Gap generators that are units need the in-operation rule of their own specs (S-0409 claim 11 names buildings' operation only).
        return None;
    }
    let b = w.rules().buildings.get(o.type_index as usize)?;
    // spec: S-0409 Import mapping (GapGenerator absent: no; GapRadiusInCells absent: 0)
    if !b.raw.gap_generator.unwrap_or(false) {
        return None;
    }
    let normal = b.raw.econ.gap_radius_in_cells.unwrap_or(0);
    // spec: S-0405 claim 16 (while the super-gap is active the gap field has the super-gap radius)
    if o.econ.building.super_gap {
        return Some(b.raw.econ.super_gap_radius_in_cells.unwrap_or(0));
    }
    Some(normal)
}

/// Whether house `h` is defeated (S-0414), for the hostility of gap fields.
fn defeated(w: &World, h: HouseId) -> bool {
    crate::econ::victory::defeated(w, h)
}

/// Starts a building's gap field (claim 11).
fn start_gap(w: &mut World, id: ObjectId) {
    let Some(field) = gap_field(w, id) else {
        return;
    };
    if w.obj(id).is_some_and(|o| o.econ.sight.gap.is_some()) {
        return;
    }
    let owner = w.obj(id).and_then(|o| o.owner);
    let (c, _) = centre(w, sight_point(w, id));
    // spec: S-0409 claim 14 (a gap field is applied even when its centre is not a legal cell)
    let cells = gap_cells(w, c, field);
    let keeps = w.rules().quirks.gap_end_keeps_cover;
    let mut houses = Vec::new();
    let mut at_start = Vec::new();
    for h in player_houses(w) {
        // spec: S-0409 claim 11 (every gap field that starts clears H's whole-map-revealed mark)
        if let Some(s) = hs_mut(w, h) {
            s.revealed = false;
        }
        // spec: S-0409 claim 11 (hostile when the owner is not allied with H, the owner's side, and H is not defeated)
        if w.allied(owner, Some(h)) || defeated(w, h) {
            continue;
        }
        if let Some(s) = hs_mut(w, h) {
            if !keeps {
                let ex: Vec<u32> = cells
                    .iter()
                    .copied()
                    .filter(|&i| s.has(i as usize, EXPLORED))
                    .collect();
                at_start.push((h, ex));
            }
            for &i in &cells {
                s.gap_start(i as usize);
            }
        }
        houses.push(h);
    }
    if let Some(o) = w.obj_mut(id) {
        o.econ.sight.gap = Some(Gap {
            owner,
            houses,
            cells,
            explored_at_start: at_start,
        });
    }
}

/// Stops a building's gap field (claim 11).
fn stop_gap(w: &mut World, id: ObjectId) {
    let Some(gap) = w.obj_mut(id).and_then(|o| o.econ.sight.gap.take()) else {
        return;
    };
    for h in player_houses(w) {
        // spec: S-0409 claim 11 (every gap field that stops clears H's whole-map-revealed mark)
        if let Some(s) = hs_mut(w, h) {
            s.revealed = false;
        }
    }
    for &h in &gap.houses {
        let mut flipped = BTreeSet::new();
        if let Some(s) = hs_mut(w, h) {
            for &i in &gap.cells {
                if s.gap_stop(i as usize) {
                    flipped.insert(i);
                }
            }
            // spec: S-0409 Edge cases (quirks.gap_end_keeps_cover off: cells explored when the field started become explored again)
            if let Some((_, ex)) = gap.explored_at_start.iter().find(|(x, _)| *x == h) {
                for &i in ex {
                    if !s.has(i as usize, EXPLORED) {
                        s.set(i as usize, EXPLORED | SURROUNDED, true);
                        flipped.insert(i);
                    }
                }
            }
        }
        flipped.remove(&shared(w));
        discover(w, h, &flipped);
    }
}

/// An accepted switch-off ends the building's gap field at once (S-0405
/// claim 10), without waiting for its next turn.
pub fn gap_switched_off(w: &mut World, id: ObjectId) {
    // spec: S-0405 claim 10 and S-0409 claim 11 (an accepted power-off order stops the field immediately, even while the building stays in operation)
    stop_gap(w, id);
}

/// A building's gap follows its operation (claim 11, S-0405 claim 9):
/// called in the building's own turn when its in-operation result changes.
pub fn gap_refresh(w: &mut World, id: ObjectId) {
    let on = w.obj(id).is_some_and(|o| o.on_map) && crate::econ::power::in_operation(w, id);
    let running = w.obj(id).is_some_and(|o| o.econ.sight.gap.is_some());
    // spec: S-0409 claim 11 (a gap generator generates while it is on the map and in operation)
    if on && !running {
        start_gap(w, id);
    } else if !on && running {
        stop_gap(w, id);
    }
}

// ---------------------------------------------------------------------------
// Game start, the sweep, whole-map operations, regrowth

/// Sets up the states at game start (claim 3), before any object is placed.
pub fn start(w: &mut World) {
    let n = w.cells.grid.len();
    w.econ.shroud.houses = w
        .houses
        .iter()
        // spec: S-0409 claim 1 (every player house keeps the states; the neutral and special houses none)
        .map(|h| (h.role == HouseRole::Player).then(|| HouseShroud::new(n)))
        .collect();
    // spec: S-0409 claim 3 (with the lobby shroud setting off the whole map is revealed for every player house, its mark set)
    if !w.rules().econ.shroud.lobby_shroud {
        for h in player_houses(w) {
            explored_only_reveal(w, h, false);
            if let Some(s) = hs_mut(w, h) {
                s.revealed = true;
            }
        }
    }
}

/// The legal cells' dense indices.
fn legal(w: &World) -> Vec<u32> {
    w.cells.grid.iter().filter_map(|(c, _)| idx(w, c)).collect()
}

/// S1 row 10: the explored-only whole-map reveal (claim 3; defeat,
/// claim 15). `clear_pending`: also clear every pending mark.
fn explored_only_reveal(w: &mut World, h: HouseId, clear_pending: bool) {
    let cells = legal(w);
    let mut flipped = Vec::new();
    if let Some(s) = hs_mut(w, h) {
        for &i in &cells {
            if !s.has(i as usize, EXPLORED) {
                flipped.push(i);
            }
            // spec: S-0409 S1 row 10 (explored, not made seen; m kept)
            s.set(i as usize, EXPLORED, true);
            if clear_pending {
                s.set(i as usize, PENDING, false);
            }
        }
    }
    // spec: S-0409 S1 row 10 (marked surrounded where all eight map neighbours are explored)
    let sur: Vec<u32> = cells
        .iter()
        .copied()
        .filter(|&i| {
            let c = cell_of(w, i);
            (-1..=1).all(|dy| (-1..=1).all(|dx| explored(w, h, CellXY::new(c.x + dx, c.y + dy))))
        })
        .collect();
    if let Some(s) = hs_mut(w, h) {
        for i in sur {
            s.set(i as usize, SURROUNDED, true);
        }
    }
    // Built at once from the list (our tooling for speed, M8 C4 part 6).
    let flipped: BTreeSet<u32> = flipped.into_iter().collect();
    discover(w, h, &flipped);
}

/// The re-cover sweep (claim 13): in every tick whose number is a multiple
/// of 120, before the turn pass.
pub fn sweep(w: &mut World) {
    // spec: S-0409 claim 13 (every tick whose tick number is a multiple of 120, tick 0 included)
    if w.tick.rem_euclid(SWEEP_PERIOD) != 0 {
        return;
    }
    for s in w.econ.shroud.houses.iter_mut().flatten() {
        for i in 0..s.marks.len() {
            s.sweep(i);
        }
    }
}

/// The objects whose sights are re-applied for `h` at a whole-map
/// operation, and the gap fields hostile to it, in the order the objects
/// rank (claim 15): (object, its sight is re-applied, its field is).
fn reapplied(w: &World, h: HouseId) -> Vec<(ObjectId, bool, bool)> {
    let skip_units = w.rules().quirks.recover_skips_allied_units;
    let ally = w.rules().econ.shroud.ally_reveal;
    w.objects
        .values()
        .filter_map(|o| {
            let sight = o.econ.sight.sight.as_ref().is_some_and(|a| {
                a.houses.contains(&h)
                    // spec: S-0409 claim 15 (own objects, allied buildings with ally reveal, other houses' reveal-to-all buildings; quirks.recover_skips_allied_units)
                    && (!skip_units
                        || o.owner == Some(h)
                        || (o.kind == Kind::Building
                            && (ally || reveal_to_all(w, o.id))))
            });
            let gap = o
                .econ
                .sight
                .gap
                .as_ref()
                .is_some_and(|g| g.houses.contains(&h));
            (sight || gap).then_some((o.id, sight, gap))
        })
        .collect()
}

/// The spy satellite's whole-map reveal (`reveal`) or the whole-map
/// re-cover of a house (claim 15, S1 rows 7 and 8). `sat`: whether the
/// spy satellite counts as active while the gap fields are taken back and
/// restored (claim 21).
fn whole_map(w: &mut World, h: HouseId, reveal: bool, sat: bool) {
    if reveal && hs(w, h).is_none_or(|s| s.revealed) {
        // spec: S-0409 claim 15 (the reveal happens only when the whole-map-revealed mark is clear)
        return;
    }
    // spec: S-0409 claim 15 (objects rank in creation order; a gap generator's field just after its own sight)
    let list = reapplied(w, h);
    let cells_of = |w: &World, id: ObjectId, sight: bool| -> Vec<u32> {
        w.obj(id).map_or_else(Vec::new, |o| {
            if sight {
                o.econ
                    .sight
                    .sight
                    .as_ref()
                    .map_or_else(Vec::new, |a| a.cells.clone())
            } else {
                o.econ
                    .sight
                    .gap
                    .as_ref()
                    .map_or_else(Vec::new, |g| g.cells.clone())
            }
        })
    };
    // spec: S-0409 claim 21 (the field changes take effect with the satellite inactive on a gain and active on a loss)
    let now = hs(w, h).is_some_and(|s| s.spy_sat);
    if let Some(st) = hs_mut(w, h) {
        st.spy_sat = sat;
    }
    // Taking back (S1 rows 2 and 5).
    for &(id, s, g) in &list {
        if s {
            let c = cells_of(w, id, true);
            remove_cells(w, h, &c);
        }
        if g {
            // spec: S-0409 claim 21 (the shared state of the cells that are not legal takes part, one change per field cell)
            let c = cells_of(w, id, false);
            if let Some(st) = hs_mut(w, h) {
                for i in c {
                    st.gap_stop(i as usize);
                }
            }
        }
    }
    // spec: S-0409 claim 21 (neither whole-map operation directly changes the shared quantities)
    let cells = legal(w);
    let mut flipped = BTreeSet::new();
    if let Some(st) = hs_mut(w, h) {
        for &i in &cells {
            let i = i as usize;
            let was = st.has(i, EXPLORED);
            st.g[i] = 0;
            if reveal {
                // spec: S-0409 claim 15 (reveal: every legal cell explored and seen, cover count 0, hostile gap count 0)
                st.k[i] = 0;
                st.set(i, EXPLORED | SEEN | SURROUNDED, true);
                if !was {
                    flipped.insert(i as u32);
                }
            } else {
                // spec: S-0409 claim 15 (re-cover: every legal cell unexplored and unseen, cover count 1, hostile gap count 0)
                st.k[i] = 1;
                st.set(i, EXPLORED | SEEN | SURROUNDED, false);
            }
        }
        st.revealed = reveal;
    }
    discover(w, h, &flipped);
    // Re-applying in the same order (S1 rows 1 and 3).
    for &(id, s, g) in &list {
        if s {
            let c = cells_of(w, id, true);
            add_cells(w, h, &c);
        }
        if g {
            // spec: S-0409 claim 21 (the shared state takes part in the restore too)
            let c = cells_of(w, id, false);
            if let Some(st) = hs_mut(w, h) {
                for i in c {
                    st.gap_start(i as usize);
                }
            }
        }
    }
    if let Some(st) = hs_mut(w, h) {
        st.spy_sat = now;
    }
    // spec: S-0409 claim 15 (any gap field on the map leaves the mark clear)
    let any_gap = w.objects.values().any(|o| o.econ.sight.gap.is_some());
    let keeps = w.rules().quirks.recover_keeps_pending;
    if let Some(st) = hs_mut(w, h) {
        if any_gap {
            st.revealed = false;
        }
        // spec: S-0409 Edge cases (quirks.recover_keeps_pending off: every pending mark cleared once the sights and fields are back)
        if !keeps {
            for &i in &cells {
                st.set(i as usize, PENDING, false);
            }
        }
    }
}

/// The crate's reveal (S-0520 claim 18): the whole map for the collecting
/// house, S-0409 claim 15's result.
pub fn crate_reveal(w: &mut World, h: HouseId) {
    let sat = spy_satellite_active(w, h);
    whole_map(w, h, true, sat);
}

/// The crate's darkness (S-0520 claim 18): the whole map re-covered for the
/// collecting house, S-0409 claim 15's result.
pub fn crate_darkness(w: &mut World, h: HouseId) {
    whole_map(w, h, false, false);
}

/// The whole-map re-cover of a house when a spy of any other house, allies
/// included, infiltrates one of its radar buildings (claim 15: alliance is
/// not tested for the effect; entry and its effect selection are S-0510's):
/// nothing happens while its spy satellite is active.
pub fn radar_infiltrated(w: &mut World, h: HouseId) {
    // spec: S-0409 claim 15 (radar infiltration re-covers the whole map, unless H has an active spy satellite)
    if !spy_satellite_active(w, h) {
        whole_map(w, h, false, false);
    }
}

/// The whole-map reveal of a defeated house (claim 15), which S-0414 calls
/// in the tick of the defeat.
pub fn defeat_reveal(w: &mut World, h: HouseId) {
    if hs(w, h).is_none_or(|s| s.revealed) {
        return;
    }
    // spec: S-0409 claim 15 (defeat: explored as in claim 3, mark set; pending marks kept, quirks.defeat_reveal_keeps_pending)
    let clear = !w.rules().quirks.defeat_reveal_keeps_pending;
    explored_only_reveal(w, h, clear);
    if let Some(s) = hs_mut(w, h) {
        s.revealed = true;
    }
}

/// The spy satellite check of a house (claim 15), run whenever its radar
/// state is rechecked (S-0410 claim 2).
pub fn spy_satellite_check(w: &mut World, h: HouseId) {
    let Some(was) = hs(w, h).map(|s| s.spy_sat) else {
        return;
    };
    let first_decides = w.rules().quirks.spy_satellite_first_building_decides;
    let flags = &w.rules().econ.shroud.spy_sat;
    let qualifying: Vec<ObjectId> = w.houses[usize::from(h.0)]
        .econ
        .acquired
        .iter()
        .copied()
        .filter(|&id| {
            w.obj(id).is_some_and(|o| {
                use crate::rules::Behaviour::Selling;
                // spec: S-0409 claim 15 (a spy-satellite building on the map, alive, neither current nor queued behaviour selling)
                o.kind == Kind::Building
                    && o.owner == Some(h)
                    && flags.get(o.type_index as usize).copied().unwrap_or(false)
                    && o.on_map
                    && o.health > 0
                    && o.beh.seen() != Some(Selling)
                    && o.beh.queued != Some(Selling)
            })
        })
        .collect();
    let erased = |id: ObjectId| crate::econ::power::being_erased(w, id);
    // spec: S-0409 claim 15 (the first qualifying building, not being erased, quirks.spy_satellite_first_building_decides; power and locks are not tested)
    let active = if first_decides {
        qualifying.first().is_some_and(|&id| !erased(id))
    } else {
        qualifying.iter().any(|&id| !erased(id))
    };
    if active == was {
        return;
    }
    if let Some(s) = hs_mut(w, h) {
        s.spy_sat = active;
    }
    // spec: S-0409 claim 15 (the whole map revealed once when it becomes active; re-covered when it stops)
    // spec: S-0409 claim 21 (the fields change with the state before the change: inactive on a gain, active on a loss)
    whole_map(w, h, active, was);
}

/// Alliance changes (S-0409 Edge cases, *ally reveal and alliance
/// changes*): buildings' sights and gap fields are taken back under the old
/// alliance and applied under the new one; units follow at their next
/// refresh. S-0414's alliance order calls it.
pub fn alliance_changed(w: &mut World) {
    let ids: Vec<ObjectId> = w.objects.keys().copied().collect();
    for id in ids {
        let Some(o) = w.obj(id) else { continue };
        let building = o.kind == Kind::Building;
        if building && let Some(a) = o.econ.sight.sight.clone() {
            let now = sight_houses(w, id);
            if now != a.houses {
                remove_sight(w, id);
                add_sight(w, id);
            }
        }
        if w.obj(id).is_some_and(|o| o.econ.sight.gap.is_some()) {
            stop_gap(w, id);
            start_gap(w, id);
        }
    }
}

/// The regrowth step (claim 18), with its period; run once per tick before
/// the turn pass (S-0203 claim 1 leaves its place free).
pub fn regrowth(w: &mut World) {
    let rules = w.rules_arc();
    let sr = &rules.econ.shroud;
    let Some(period) = sr.regrowth_period.filter(|_| sr.regrowth) else {
        return;
    };
    // spec: S-0409 claim 18 (every trunc(rate × 900) ticks, the first at tick 0; every tick when the period is 0 or less)
    if period > 0 && w.tick.rem_euclid(period) != 0 {
        return;
    }
    let cells = legal(w);
    for h in player_houses(w) {
        let mut taken = Vec::new();
        if let Some(s) = hs(w, h) {
            for &i in &cells {
                let i = i as usize;
                // spec: S-0409 claim 18 (taken: explored and not marked surrounded, or pending)
                if (s.has(i, EXPLORED) && !s.has(i, SURROUNDED)) || s.has(i, PENDING) {
                    taken.push(i as u32);
                }
            }
        }
        let mut lost = Vec::new();
        if let Some(s) = hs_mut(w, h) {
            for &i in &taken {
                let i = i as usize;
                s.set(i, PENDING, false);
                if s.has(i, EXPLORED) {
                    // spec: S-0409 claim 18 (a taken explored cell becomes unexplored, seen unchanged; its neighbours lose the surrounded mark)
                    s.set(i, EXPLORED | SURROUNDED, false);
                    lost.push(i as u32);
                }
            }
        }
        let mut neighbours = Vec::new();
        for &i in &lost {
            let c = cell_of(w, i);
            for dy in -1..=1 {
                for dx in -1..=1 {
                    if let Some(n) = idx(w, CellXY::new(c.x + dx, c.y + dy)) {
                        neighbours.push(n);
                    }
                }
            }
        }
        if let Some(s) = hs_mut(w, h) {
            for n in neighbours {
                s.set(n as usize, SURROUNDED, false);
            }
        }
        // spec: S-0409 claim 18 (the sights of H's own objects and allied buildings applied again as uncounted sights, quirks.uncounted_sight)
        let uncounted = rules.quirks.uncounted_sight;
        let objs: Vec<ObjectId> = w
            .objects
            .values()
            .filter(|o| {
                o.econ
                    .sight
                    .sight
                    .as_ref()
                    .is_some_and(|a| a.houses.contains(&h))
                    && (o.owner == Some(h) || o.kind == Kind::Building)
            })
            .map(|o| o.id)
            .collect();
        for id in objs {
            if uncounted {
                let c = w
                    .obj(id)
                    .and_then(|o| o.econ.sight.sight.as_ref())
                    .map_or_else(Vec::new, |a| a.cells.clone());
                uncounted_sight(w, h, &c);
            } else {
                // spec: S-0409 Edge cases (quirks.uncounted_sight off: regrowth re-applies the sights as counted sights)
                remove_sight(w, id);
                add_sight(w, id);
            }
        }
    }
}

/// S1 row 9: an uncounted sight of `h` over cells.
fn uncounted_sight(w: &mut World, h: HouseId, cells: &[u32]) {
    let todo: Vec<u32> = cells
        .iter()
        .copied()
        .filter(|&i| {
            let c = cell_of(w, i);
            // spec: S-0409 S1 row 9 (only when the cell is not yet explored, seen and surrounded with all neighbours seen)
            let full = hs(w, h).is_some_and(|s| {
                s.has(i as usize, EXPLORED)
                    && s.has(i as usize, SEEN)
                    && s.has(i as usize, SURROUNDED)
            }) && (-1..=1)
                .all(|dy| (-1..=1).all(|dx| seen(w, h, CellXY::new(c.x + dx, c.y + dy))));
            !full
        })
        .collect();
    add_cells(w, h, &todo);
}

#[cfg(test)]
mod discover_tests {
    use super::*;
    use crate::testkit;

    /// The discovery as it was before M8 C4 part 6: each flipped cell's nine
    /// cells compared, whichever flipped neighbour asked before.
    fn discover_as_before(w: &mut World, h: HouseId, flipped: &BTreeSet<u32>) {
        let mut changed = BTreeSet::new();
        for &i in flipped {
            let c = cell_of(w, i);
            for dy in -1..=1 {
                for dx in -1..=1 {
                    let a = CellXY::new(c.x + dx, c.y + dy);
                    if idx(w, a).is_none() {
                        continue;
                    }
                    if piece_key(w, h, a, |i| flipped.contains(&i)) != piece_key(w, h, a, |_| false)
                    {
                        changed.insert(a);
                    }
                }
            }
        }
        let mut found = Vec::new();
        for a in changed {
            for n in 0..=6 {
                let x = CellXY::new(a.x + n, a.y + n);
                if idx(w, x).is_none() {
                    continue;
                }
                let lv = level(w, x);
                if !(2 * n - 1 <= lv && lv <= 2 * n + 1) {
                    continue;
                }
                if let Some(id) = nearest_in_cell(w, x) {
                    found.push(id);
                }
            }
        }
        if let Some(s) = hs_mut(w, h) {
            s.discovered.extend(found);
        }
    }

    /// Our tooling for speed (M8 C4 part 6): a small reveal (kept in sets)
    /// and a whole-map one (in grid-sized tables) discover the objects the
    /// discovery as it was before discovers (S-0409 Terms, *discovered by
    /// H*; S-0110 claims 1 and 2), over a patchy explored state.
    #[test]
    fn small_and_whole_map_reveals_discover_as_before() {
        const SIZE: i32 = 40; // nocite: test data of our own
        let h = HouseId(0);
        let mut w = testkit::game_with(&testkit::ruleset(), SIZE, 3);
        for y in (-30..=30).step_by(3) {
            for x in (-30..=30).step_by(2) {
                testkit::spawn(
                    &mut w,
                    Kind::Vehicle,
                    "CART",
                    1,
                    testkit::mid(SIZE, x, y),
                    0,
                );
            }
        }
        for st in w.econ.shroud.houses.iter_mut().flatten() {
            st.discovered.clear();
        }
        let cells = legal(&w);
        // A patchy explored state.
        let patch: BTreeSet<u32> = cells
            .iter()
            .copied()
            .filter(|&i| {
                let c = cell_of(&w, i);
                (c.x * 7 + c.y * 3).rem_euclid(5) == 0 || (c.x / 6 + c.y / 4) % 3 == 0
            })
            .collect();
        let small: BTreeSet<u32> = cells
            .iter()
            .copied()
            .filter(|&i| {
                let c = cell_of(&w, i);
                (c.x - SIZE).abs() <= 6 && (c.y - SIZE).abs() <= 6 && !patch.contains(&i)
            })
            .collect();
        let whole: BTreeSet<u32> = cells
            .iter()
            .copied()
            .filter(|i| !patch.contains(i))
            .collect();
        assert!(!small.is_empty() && small.len() <= DENSE_REVEAL);
        assert!(whole.len() > DENSE_REVEAL);
        for flipped in [small, whole] {
            let mut a = w.clone();
            if let Some(s) = hs_mut(&mut a, h) {
                for &i in &cells {
                    let on = patch.contains(&i) || flipped.contains(&i);
                    s.set(i as usize, EXPLORED, on);
                }
            }
            let mut b = a.clone();
            discover(&mut a, h, &flipped);
            discover_as_before(&mut b, h, &flipped);
            let found = &hs(&b, h).expect("a player house").discovered;
            assert!(!found.is_empty());
            assert_eq!(&hs(&a, h).expect("a player house").discovered, found);
        }
    }
}
