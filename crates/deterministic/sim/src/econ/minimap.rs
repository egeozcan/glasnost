//! What the radar map shows (S-0410 claims 4 to 10): pure functions on
//! world data that the interface calls. Radar reaches no game state, so
//! nothing here is saved or hashed; the arrival order of claim 8 is kept by
//! the caller in an [`Arrivals`] value, which it updates once per tick.
//!
//! Numbers follow S-0410's formulas: the scale is a single-precision value
//! and the other operations are double precision under round toward zero
//! (soft floats of `glasnost_num`, no host floats).

use std::collections::BTreeMap;

use glasnost_map::CellXY;
use glasnost_num::{F32, F64, Rounding};

use crate::rules::{Kind, nearest_double};
use crate::world::{HouseId, ObjectId, Pos, World};

/// An 8-bit triple (S-0410 Terms): red, green, blue, 0 to 255 each.
pub type Rgb = [u8; 3]; // spec: S-0410 Terms (an 8-bit triple: red, green, blue)

// spec: S-0410 Constants (radar map 140 × 108 pixels)
const MAP_W: i32 = 140;
// spec: S-0410 Constants (radar map 140 × 108 pixels)
const MAP_H: i32 = 108;
const RZ: Rounding = Rounding::TowardZero; // spec: S-0410 F2, F3 and F6 (round toward zero)

/// The radar point grid of a map (claim 4, F1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Grid {
    /// *a* of F1.
    pub a: i32,
    /// *b* of F1: the smallest *x* + *y* of a usable cell.
    pub b: i32,
    /// *W*: columns of the point grid.
    pub w: i32,
    /// *N*: rows of the point grid.
    pub n: i32,
    /// *L*w, the local area's width.
    pub lw: i32,
}

/// The radar points a cell covers under *a*, *b* and *L*w (claim 4, F1).
pub fn cell_points(a: i32, b: i32, lw: i32, c: CellXY) -> Vec<(i32, i32)> {
    // spec: S-0410 F1 (u = x − y + a, v = x + y − b; points (u, v) and (u + 1, v))
    let u = c.x - c.y + a;
    let v = c.x + c.y - b;
    let last = 2 * lw - 1; // spec: S-0410 F1 (the last column 2·Lw − 1)
    if u == -1 {
        // spec: S-0410 F1 (a cell with u = −1 covers only (0, v))
        vec![(0, v)]
    } else if u == last {
        // spec: S-0410 F1 (a cell with u = 2·Lw − 1 covers only (u, v))
        vec![(u, v)]
    } else {
        vec![(u, v), (u + 1, v)]
    }
}

/// The point grid of the world's map (claim 4, F1); `None` when it has no
/// usable cell.
pub fn grid(w: &World) -> Option<Grid> {
    let fp = *w.cells.footprint();
    let usable: Vec<CellXY> = w
        .cells
        .grid
        .iter()
        .map(|(c, _)| c)
        .filter(|&c| w.cells.usable(c))
        .collect();
    // spec: S-0410 F1 (b = min(x + y) over the usable cells)
    let b = usable.iter().map(|c| c.x + c.y).min()?;
    // spec: S-0410 F1 (a = Wₘ − 2·Lx − 1)
    let a = fp.width - 2 * fp.local[0] - 1;
    let lw = fp.local[2]; // spec: S-0009 Terms (the local area (Lx, Ly, Lw, Lh): Lw is the third number)
    let mut wcols = 0;
    let mut n = 0;
    for &c in &usable {
        for (u, v) in cell_points(a, b, lw, c) {
            // spec: S-0410 F1 (W = the largest c + e; N = max(v) + 1)
            wcols = wcols.max(u + 1);
            n = n.max(v + 1);
        }
    }
    Some(Grid {
        a,
        b,
        w: wcols,
        n,
        lw,
    })
}

/// The radar image inside the radar map (claim 5, F2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Image {
    /// The scale *s*, single precision.
    pub s: F32,
    /// Width in pixels.
    pub width: i32,
    /// Height in pixels.
    pub height: i32,
    /// The image's top-left pixel in the radar map.
    pub left: i32,
    /// The image's top-left pixel in the radar map.
    pub top: i32,
}

fn d(v: i64) -> F64 {
    F64::from_i64(v, RZ)
}

/// The quotient of two integers rounded toward zero to single precision.
fn single_quotient(a: i64, b: i64) -> F32 {
    // Rounding toward zero to double and then to single equals rounding toward zero to single once.
    d(a).div(d(b), RZ).to_f32(RZ)
}

/// The scale and size of the radar image (claim 5, F2).
pub fn image(g: &Grid) -> Image {
    let (wc, n) = (i64::from(g.w.max(1)), i64::from(g.n.max(1)));
    // spec: S-0410 F2 (s₀ = 140 / W, single precision)
    let s0 = single_quotient(i64::from(MAP_W), wc);
    // spec: S-0410 F2 (the test N × s₀ < 108 on the exact product)
    let (s, width, height) = if d(n).mul(s0.to_f64(), RZ).lt(d(i64::from(MAP_H))) {
        let h = s0.to_f64().mul(d(n), RZ).to_i32_common();
        (s0, MAP_W, h)
    } else {
        // spec: S-0410 F2 (s = 108 / N; width trunc(s × W))
        let s = single_quotient(i64::from(MAP_H), n);
        (s, s.to_f64().mul(d(wc), RZ).to_i32_common(), MAP_H)
    };
    Image {
        s,
        width,
        height,
        // spec: S-0410 F2 (top-left at (trunc((140 − width) / 2), trunc((108 − height) / 2)))
        left: (MAP_W - width) / 2,
        top: (MAP_H - height) / 2, // spec: S-0410 F2 (the image centred)
    }
}

/// The cell shown at radar pixel (*p*, *q*) of the image (claim 6, F3;
/// `quirks.radar_pixel_cell_shift`).
pub fn cell_at_pixel(g: &Grid, img: &Image, p: i32, q: i32, shift: bool) -> CellXY {
    let s = img.s.to_f64();
    let pu = d(i64::from(p)).div(s, RZ);
    let qv = d(i64::from(q)).div(s, RZ);
    if !shift {
        // spec: S-0410 Edge cases (switch off: the cell that covers point (⌊p / s⌋, ⌊q / s⌋) of F1)
        let (u, v) = (pu.to_i32_common(), qv.to_i32_common());
        let sum = v + g.b;
        let mut diff = u - g.a;
        let odd = (diff - sum).rem_euclid(2) != 0; // spec: S-0410 Edge cases (x − y has the parity of x + y)
        if odd {
            diff -= 1;
        }
        return CellXY::new((sum + diff) / 2, (sum - diff) / 2); // spec: S-0410 Edge cases (the cell (x, y) from x + y and x − y)
    }
    // spec: S-0410 F3 (u′ = p / s − a, v′ = q / s + b; cell = (trunc((u′ + v′) × 0.5 + 0.5), trunc((v′ − u′) × 0.5 + 0.5)))
    let u1 = pu.sub(d(i64::from(g.a)), RZ);
    let v1 = qv.add(d(i64::from(g.b)), RZ);
    let half = nearest_double("0.5");
    let x = u1.add(v1, RZ).mul(half, RZ).add(half, RZ).to_i32_common();
    let y = v1.sub(u1, RZ).mul(half, RZ).add(half, RZ).to_i32_common();
    CellXY::new(x, y)
}

/// The radar pixel of a stored position (claim 8, F4); `clamp` for an
/// object of the shown house that is not a building.
pub fn object_pixel(g: &Grid, img: &Image, x: i32, y: i32, clamp: bool) -> (i32, i32) {
    let s = img.s.to_f64();
    // spec: S-0410 F4 (p = trunc((256·a + X − Y) × s / 256), q = trunc((X + Y − 256·b) × s / 256))
    let pv = d(256 * i64::from(g.a) + i64::from(x) - i64::from(y))
        .mul(s, RZ)
        .div(d(256), RZ)
        .to_i32_common();
    let qv = d(i64::from(x) + i64::from(y) - 256 * i64::from(g.b)) // spec: S-0410 F4 (256 leptons per cell)
        .mul(s, RZ)
        .div(d(256), RZ) // spec: S-0410 F4 (divided by 256)
        .to_i32_common();
    if clamp {
        // spec: S-0410 F4 (the own object's pixel clamped to the image)
        (pv.clamp(0, img.width - 1), qv.clamp(0, img.height - 1))
    } else {
        (pv, qv)
    }
}

/// Whether an object flashes inverted in a tick (claim 10, F5): *t* the
/// countdown's remaining ticks (≥ 1), *F* the flash frame time.
pub fn flash_inverted(t: i32, f: i32) -> bool {
    if f == 0 {
        // spec: S-0410 F5 (F = 0: a departure, no inversion at all)
        return false;
    }
    // spec: S-0410 F5 (inverted when trunc((t − 1) / F) is odd, also for a negative F)
    ((t - 1) / f) % 2 != 0
}

/// The point colour of a tile cell's tile radar colour (claim 9, F7); `None`
/// (no tile radar colour for the sub-tile) gives the fixed dark grey.
pub fn tile_colour(c: Option<Rgb>, snow: bool) -> Rgb {
    let Some(c) = c else {
        // spec: S-0410 Constants (fallback terrain colour (60, 60, 60), not scaled and not halved)
        return [60, 60, 60];
    };
    // spec: S-0410 Constants (theater brightness 0.8 for snow, the single-precision value 0.800000011920929; 1 otherwise)
    let f = if snow {
        nearest_double("0.800000011920929")
            .to_f32(Rounding::Nearest)
            .to_f64()
    } else {
        d(1)
    };
    c.map(|ch| {
        // spec: S-0410 F7 (c′ = ⌊trunc(min(c · f, 255)) / 2⌋)
        let prod = d(i64::from(ch)).mul(f, RZ).min(d(255));
        (prod.to_i32_common() / 2) as u8 // spec: S-0410 F7 (the division by 2 is integer)
    })
}

/// What decides a cell's point colour (claim 9), first case first.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PointSource {
    /// The cell holds a terrain object.
    TerrainObject,
    /// The cell counts as a bridge cell: its bridge sprite's frame radar
    /// colour, image frame 0.
    Bridge(Rgb),
    /// An overlay's colour after [`overlay_rule`] and [`swap_green_blue`].
    Overlay(Rgb),
    /// A resource whose type has no sprite.
    ResourceWithoutSprite,
    /// The tile colour (F7).
    Tile {
        /// The first tile radar colour of the tile cell, if it has one.
        colour: Option<Rgb>,
        /// The snow theater.
        snow: bool,
    },
}

/// A cell's point colour (claim 9).
pub fn point_colour(src: PointSource) -> Rgb {
    match src {
        // spec: S-0410 Constants (terrain-object cell colour (200, 200, 160))
        PointSource::TerrainObject => [200, 200, 160],
        PointSource::Bridge(c) | PointSource::Overlay(c) => c,
        // spec: S-0410 Constants (resource colour without sprite (170, 170, 130))
        PointSource::ResourceWithoutSprite => [170, 170, 130],
        PointSource::Tile { colour, snow } => tile_colour(colour, snow),
    }
}

/// Which image frame of an overlay's sprite colours its cell, and whether
/// green and blue are exchanged (claim 9, table T1 of Edge cases); `None`:
/// the cell counts as having no overlay.
pub fn overlay_rule(
    position: u32,
    resource: bool,
    cell_frame: u32,
    quirk: bool,
) -> Option<(u32, bool)> {
    if !quirk {
        // spec: S-0410 Edge cases (quirks.overlay_list_position_rules off: the cell's own frame, as stored)
        return Some((cell_frame, false));
    }
    // spec: S-0410 T1 (position 239: every overlay type counts as no overlay)
    if position == 239 {
        return None;
    }
    if resource {
        // spec: S-0410 T1 (a resource overlay takes the cell's own image frame, as stored)
        return Some((cell_frame, false));
    }
    // spec: S-0410 T1 (positions 74 to 99 and 205 to 230: image frame 1)
    if (74..=99).contains(&position) || (205..=230).contains(&position) {
        return Some((1, false));
    }
    // spec: S-0410 T1 (positions 127 to 138 and 147 to 158: green and blue exchanged)
    let swap = (127..=138).contains(&position) || (147..=158).contains(&position);
    Some((cell_frame, swap))
}

/// Exchanges the green and blue channels (claim 9, T1).
pub fn swap_green_blue(c: Rgb) -> Rgb {
    [c[0], c[2], c[1]] // spec: S-0410 T1 (green and blue exchanged)
}

/// The terrain picture (claim 5, F6): the area-weighted mean of the point
/// colours (`points`, *W* × *N*, row-major) over each pixel of the image,
/// as 8-bit triples (row-major, width × height).
pub fn terrain_picture(g: &Grid, img: &Image, points: &[Rgb]) -> Vec<Rgb> {
    let s = img.s.to_f64();
    let (wc, n) = (g.w.max(0), g.n.max(0));
    // spec: S-0410 F6 (gₓ = W / w, g_y = N / h, single precision)
    let gx = single_quotient(i64::from(wc), i64::from(img.width.max(1))).to_f64();
    let gy = single_quotient(i64::from(n), i64::from(img.height.max(1))).to_f64();
    let area = gx.mul(gy, RZ);
    let single = |v: F64| v.to_f32(RZ).to_f64();
    let overlap = |j: i32, lo: F64, hi: F64| -> F64 {
        // spec: S-0410 F6 (the length of [j, j + 1) ∩ [lo, hi), double precision, rounded toward zero)
        let a = d(i64::from(j)).max(lo);
        let b = d(i64::from(j) + 1).min(hi);
        if b.gt(a) { b.sub(a, RZ) } else { F64::ZERO }
    };
    let mut out = Vec::with_capacity((img.width.max(0) * img.height.max(0)) as usize);
    for q in 0..img.height {
        // spec: S-0410 F6 (v₀ = q / s, v₁ = v₀ + g_y, single precision)
        let v0 = single(d(i64::from(q)).div(s, RZ));
        let v1 = single(v0.add(gy, RZ));
        for p in 0..img.width {
            let u0 = single(d(i64::from(p)).div(s, RZ));
            let u1 = single(u0.add(gx, RZ));
            let mut sum = [F64::ZERO; 3]; // spec: S-0410 Terms (three channels)
            let (j0, j1) = (u0.to_i32_common().max(0), u1.to_i32_common().min(wc - 1));
            let (i0, i1) = (v0.to_i32_common().max(0), v1.to_i32_common().min(n - 1));
            for i in i0..=i1 {
                let oy = overlap(i, v0, v1);
                for j in j0..=j1 {
                    let ox = overlap(j, u0, u1);
                    let wgt = ox.mul(oy, RZ);
                    if wgt.eq_num(F64::ZERO) {
                        continue;
                    }
                    let c = points
                        .get((i * wc + j) as usize)
                        .copied()
                        .unwrap_or([0, 0, 0]);
                    for (k, sk) in sum.iter_mut().enumerate() {
                        *sk = sk.add(wgt.mul(d(i64::from(c[k])), RZ), RZ);
                    }
                }
            }
            let half = nearest_double("0.5");
            // spec: S-0410 F6 (channel = min(255, trunc(mean + 0.5)))
            out.push(
                sum.map(|sk| sk.div(area, RZ).add(half, RZ).to_i32_common().clamp(0, 255) as u8),
            );
        }
    }
    out
}

/// The bits per channel of the display surface (S-0410 Terms, *radar
/// colour*): 5, 6 and 5, or 5 in every channel.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Bits(pub [u8; 3]); // spec: S-0410 Terms (bits per channel of the three channels)

/// A radar colour: channel values at the display's bit widths.
pub type RadarColour = [u16; 3]; // spec: S-0410 Terms (three channels at the display precision)

/// An 8-bit triple at the display's precision (S-0410 Terms): the low bits
/// of each channel dropped.
pub fn radar_colour(c: Rgb, bits: Bits) -> RadarColour {
    // spec: S-0410 Terms (c₈ becomes ⌊c₈ / 2^(8 − k)⌋)
    [0, 1, 2].map(|i| u16::from(c[i]) >> (8 - bits.0[i].min(8)))
}

/// Each channel halved, truncated (claim 6, the gap darkening).
pub fn halved(c: RadarColour) -> RadarColour {
    // spec: S-0410 claim 6 (c becomes ⌊c / 2⌋ at its own bit width)
    c.map(|x| x / 2)
}

/// Each channel inverted (claim 10): 2^k − 1 − c.
pub fn inverted(c: RadarColour, bits: Bits) -> RadarColour {
    // spec: S-0410 claim 10 (c replaced by 2^k − 1 − c)
    [0, 1, 2].map(|i| ((1u16 << bits.0[i]) - 1).saturating_sub(c[i]))
}

/// Whether a cell is shrouded for the shown house, by S-0110's point test on
/// the cell's centre at the cell's surface height there, heights below 0
/// included (claim 6); a cell that is not usable is not shrouded (claim 6,
/// our engine's choice).
pub fn cell_shrouded(w: &World, h: HouseId, c: CellXY) -> bool {
    cell_shrouded_on(w, h, c, true)
}

/// [`cell_shrouded`] under the renderer's `quirks.shroud_on_flat_grid`
/// (claim 6: its direct-cell alternative takes precedence over the pair
/// choice).
pub fn cell_shrouded_on(w: &World, h: HouseId, c: CellXY, on_flat_grid: bool) -> bool {
    if !w.cells.usable(c) {
        return false;
    }
    let p = ground_centre(w, c);
    // spec: S-0410 claim 6 (S-0110 claim 8's point test, signed heights, no clamp to 0;
    // quirks.shroud_negative_odd_pair and the precedence of quirks.shroud_on_flat_grid off)
    crate::econ::shroud::point_shrouded_on(w, h, p, on_flat_grid)
}

fn ground_centre(w: &World, c: CellXY) -> Pos {
    let p = Pos::centre(c, 0);
    // spec: S-0410 claim 6 (the cell surface height at its centre, S-0106 claims 2 and 3; a slope's centre may lie above its level)
    let z = w.surface_at(p.x, p.y);
    Pos::new(p.x, p.y, z)
}

/// The cells covered by gap fields friendly to the house (S-0410 Terms):
/// its own or an ally's (its side), or any field once it is defeated.
fn friendly_gap_cells(w: &World, h: HouseId) -> std::collections::BTreeSet<u32> {
    let defeated = crate::econ::victory::defeated(w, h);
    let mut out = std::collections::BTreeSet::new();
    for o in w.objects.values() {
        if let Some(g) = &o.econ.sight.gap
            && (defeated || w.allied(Some(h), g.owner))
        {
            out.extend(g.cells.iter().copied());
        }
    }
    out
}

/// Whether a cell lies in a friendly gap field of the house, by the
/// projection of the point test (claim 6): with an odd number of height
/// levels, both candidate cells must lie in one.
pub fn cell_in_friendly_gap(w: &World, h: HouseId, c: CellXY) -> bool {
    in_friendly_gap(w, c, &friendly_gap_cells(w, h), true)
}

/// [`cell_in_friendly_gap`] under the renderer's
/// `quirks.shroud_on_flat_grid` (claim 6).
pub fn cell_in_friendly_gap_on(w: &World, h: HouseId, c: CellXY, on_flat_grid: bool) -> bool {
    in_friendly_gap(w, c, &friendly_gap_cells(w, h), on_flat_grid)
}

fn in_friendly_gap(
    w: &World,
    c: CellXY,
    cells: &std::collections::BTreeSet<u32>,
    on_flat_grid: bool,
) -> bool {
    if !w.cells.usable(c) {
        return false;
    }
    let p = ground_centre(w, c);
    // spec: S-0410 claim 6, S-0110 claim 8 (the same candidate cells as the shroud test, heights below
    // 0 included, quirks.shroud_negative_odd_pair; quirks.shroud_on_flat_grid off takes precedence)
    let (a, b) = crate::econ::shroud::point_candidates_on(
        p,
        w.rules().quirks.shroud_negative_odd_pair,
        on_flat_grid,
    );
    let inside = |c: CellXY| w.cells.idx(c).is_some_and(|i| cells.contains(&(i as u32)));
    // spec: S-0410 claim 6 (for odd z the gap test needs both candidate cells in a friendly gap field)
    inside(a) && b.is_none_or(inside)
}

fn type_flag(
    w: &World,
    id: ObjectId,
    f: impl Fn(&glasnost_rules::sim_econ::EconFields) -> Option<bool>,
) -> bool {
    let Some(o) = w.obj(id) else { return false };
    let r = w.rules();
    // spec: S-0410 Import mapping (RadarInvisible, Insignificant, Invisible, RadarVisible absent: no)
    let e = match o.kind {
        Kind::Building => r.buildings.get(o.type_index as usize).map(|b| &b.raw.econ),
        Kind::Terrain => None,
        _ => r.unit(o.kind, o.type_index).map(|u| &u.raw.econ),
    };
    e.and_then(f).unwrap_or(false)
}

fn insignificant(w: &World, id: ObjectId) -> bool {
    let Some(o) = w.obj(id) else { return false };
    let r = w.rules();
    // spec: S-0410 Import mapping (Insignificant absent: no)
    match o.kind {
        Kind::Building => r
            .buildings
            .get(o.type_index as usize)
            .and_then(|b| b.raw.combat.insignificant),
        Kind::Terrain => None,
        _ => r
            .unit(o.kind, o.type_index)
            .and_then(|u| u.raw.combat.insignificant),
    }
    .unwrap_or(false)
}

/// The radar pixels at which an object is listed for house `h` (claims 7
/// and 8): empty when it is not listed. A building is listed at the pixel
/// set of F8 inside the image.
pub fn listed(w: &World, h: HouseId, id: ObjectId, g: &Grid, img: &Image) -> Vec<(i32, i32)> {
    let Some(o) = w.obj(id) else {
        return Vec::new();
    };
    // spec: S-0410 Terms (only radar objects: infantry, vehicles, aircraft, buildings)
    if o.kind == Kind::Terrain {
        return Vec::new();
    }
    // spec: S-0410 claim 7 (on the map, not inside a transport or building, type not invisible)
    if !o.on_map || type_flag(w, id, |e| e.invisible) {
        return Vec::new();
    }
    let own = o.owner == Some(h);
    if own {
        // spec: S-0410 claim 7 (owned by H: the discovered-by-H mark, S-0409 claim 20; ownership alone does not pass)
        if !crate::econ::shroud::discovered(w, h, id) {
            return Vec::new();
        }
    } else {
        // spec: S-0410 claim 7 (not owned: inside the usable area at its position, its point not shrouded)
        if !w.cells.usable(o.pos.cell()) || crate::econ::shroud::point_shrouded(w, h, o.pos) {
            return Vec::new();
        }
        // spec: S-0410 claim 7 and S-0516 claim 27 (another house's fully cloaked object, an ally's included, only where H's own sensor coverage covers its cell)
        if !crate::cloak::view::on_radar(w, id, Some(h)) {
            return Vec::new();
        }
        // Being underground (claim 7) needs its topic (seam).
        if crate::combat::veterancy::has_ability(
            w,
            id,
            crate::combat::data::Ability::RadarInvisible,
        ) {
            return Vec::new();
        }
    }
    if o.kind == Kind::Building {
        // spec: S-0410 F8 (around the unclamped F4 pixel of its stored position)
        let top = object_pixel(g, img, o.pos.x, o.pos.y, false);
        building_pixels(img, top, o.foundation)
            .into_iter()
            // spec: S-0410 F8 (pixels outside the radar image are omitted, never clamped)
            .filter(|&(p, q)| (0..img.width).contains(&p) && (0..img.height).contains(&q))
            .collect()
    } else {
        vec![object_pixel(g, img, o.pos.x, o.pos.y, own)]
    }
}

/// The building pixel set of F8 (claim 8) for a foundation of *A* × *B*
/// cells whose top pixel is `top`, in row order, before the pixels outside
/// the image are left out.
pub fn building_pixels(img: &Image, top: (i32, i32), foundation: (i32, i32)) -> Vec<(i32, i32)> {
    let s = img.s.to_f64();
    let half = nearest_double("0.5");
    let extent = |dim: i32| -> i32 {
        // spec: S-0410 F8 (E(d) = trunc(max(d·s + 0.5, 1)) when d = 1, else trunc(max(d·s + 0.5, 2)), double precision, round toward zero)
        let floor = if dim == 1 { 1 } else { 2 };
        let v = d(i64::from(dim)).mul(s, RZ).add(half, RZ);
        let m = if v.lt(d(floor)) { d(floor) } else { v };
        m.to_i32_common()
    };
    let (ab, bb) = (extent(foundation.0), extent(foundation.1));
    let (p, q) = top;
    let mut out = Vec::new();
    // spec: S-0410 F8 (rows r from 0 through a_b + b_b − 2)
    for r in 0..=(ab + bb - 2) {
        // spec: S-0410 F8 (left(r) = −r for r < b_b, else r − 2b_b + 2)
        let left = if r < bb { -r } else { r - 2 * bb + 2 };
        // spec: S-0410 F8 (right(r) = r for r < a_b, else 2a_b − r − 2)
        let right = if r < ab { r } else { 2 * ab - r - 2 };
        for i in left..=right {
            out.push((p + i, q + r));
        }
    }
    out
}

/// The draw-time tests of claim 7 for a listed object at a pixel.
pub fn shown(
    w: &World,
    h: HouseId,
    id: ObjectId,
    pixel: (i32, i32),
    g: &Grid,
    img: &Image,
) -> bool {
    let Some(o) = w.obj(id) else { return false };
    let shift = w.rules().quirks.radar_pixel_cell_shift;
    // spec: S-0410 claim 7 (not owned by H: the cell shown at its pixel is not shrouded)
    if o.owner != Some(h) && cell_shrouded(w, h, cell_at_pixel(g, img, pixel.0, pixel.1, shift)) {
        return false;
    }
    // spec: S-0410 claim 7 (radar-invisible, unless H allies its owner, H's side)
    if type_flag(w, id, |e| e.radar_invisible) && !w.allied(Some(h), o.owner) {
        return false;
    }
    // spec: S-0410 claim 7 (insignificant, unless radar-visible or owned by a house whose country is not passive)
    if insignificant(w, id) && !type_flag(w, id, |e| e.radar_visible) {
        let active_owner = o.owner.is_some_and(|ow| {
            let c = w.houses[usize::from(ow.0)].country;
            w.rules()
                .countries
                .get(c as usize)
                .is_some_and(|x| !x.passive)
        });
        if !active_owner {
            return false;
        }
    }
    true
}

/// The arrivals of listed objects at pixels (claim 8, Terms *listed,
/// arrival*), kept by the interface: [`Arrivals::observe`] once per tick,
/// after the tick, stands for the start-of-turn evaluation of the next
/// tick's turns.
#[derive(Clone, Debug, Default)]
pub struct Arrivals {
    /// (object, pixel) to its arrival: (tick, place in the update order).
    pub at: BTreeMap<(ObjectId, (i32, i32)), (i32, usize)>,
}

impl Arrivals {
    /// Brings the arrivals up to date for house `h`.
    pub fn observe(&mut self, w: &World, h: HouseId, g: &Grid, img: &Image) {
        let mut now = BTreeMap::new();
        for &id in w.objects.keys() {
            let place = w
                .update
                .order
                .iter()
                .position(|&x| x == id)
                .unwrap_or(usize::MAX);
            for px in listed(w, h, id, g, img) {
                // spec: S-0410 Terms (an object that stays listed at one pixel keeps its arrival; arrivals ordered by tick, then by turn order)
                let arr = self.at.get(&(id, px)).copied().unwrap_or((w.tick, place));
                now.insert((id, px), arr);
            }
        }
        self.at = now;
    }

    /// The object whose colour a pixel shows (claim 8): the shown house's
    /// own objects first, latest arrival first; then the others, earliest
    /// arrival first; the first that passes the draw-time tests.
    pub fn winner(
        &self,
        w: &World,
        h: HouseId,
        pixel: (i32, i32),
        g: &Grid,
        img: &Image,
    ) -> Option<ObjectId> {
        let mut own = Vec::new();
        let mut other = Vec::new();
        for (&(id, px), &arr) in &self.at {
            if px != pixel {
                continue;
            }
            if w.obj(id).is_some_and(|o| o.owner == Some(h)) {
                own.push((arr, id));
            } else {
                other.push((arr, id));
            }
        }
        // spec: S-0410 claim 8 (own objects first, the latest arrival first; then the others, the earliest first)
        own.sort_by(|a, b| b.cmp(a));
        other.sort();
        own.into_iter()
            .chain(other)
            .map(|(_, id)| id)
            .find(|&id| shown(w, h, id, pixel, g, img))
    }
}

/// Inputs of the radar map that are not game state: house colours, the
/// neutral colour, flash countdowns and the display's bit widths.
pub struct View<'a> {
    /// The house whose radar map is shown.
    pub house: HouseId,
    /// A house's colour (S-0103), or the colour it is disguised as against
    /// the shown house.
    pub house_colour: &'a dyn Fn(HouseId) -> Rgb,
    /// The fixed neutral colour of an object without an owner.
    pub neutral: Rgb,
    /// An object's radar flash countdown (remaining ticks, 0 when none).
    pub flash: &'a dyn Fn(ObjectId) -> i32,
    /// The flash frame time *F* (claim 10).
    pub flash_frame_time: i32,
    /// Bits per channel of the display surface.
    pub bits: Bits,
    /// The renderer's `quirks.shroud_on_flat_grid` (S-0110 Edge cases),
    /// which the radar's shroud and friendly-gap tests share (claim 6).
    pub on_flat_grid: bool,
}

/// The radar colour of every pixel of the image (claim 6), row-major; the
/// terrain picture comes from [`terrain_picture`].
pub fn compose(
    w: &World,
    view: &View,
    arrivals: &Arrivals,
    g: &Grid,
    img: &Image,
    terrain: &[Rgb],
) -> Vec<RadarColour> {
    let h = view.house;
    let shift = w.rules().quirks.radar_pixel_cell_shift;
    let gaps = friendly_gap_cells(w, h);
    let mut out = Vec::with_capacity(terrain.len());
    for q in 0..img.height {
        for p in 0..img.width {
            let t = terrain
                .get((q * img.width + p) as usize)
                .copied()
                .unwrap_or([0, 0, 0]);
            // spec: S-0410 claim 6 (the colour of the first object shown at the pixel)
            if let Some(id) = arrivals.winner(w, h, (p, q), g, img) {
                let o = w.obj(id);
                // spec: S-0410 claim 8 and S-0516 claim 38 (a disguised unit: its owner's colour when the owner allies H, otherwise its disguise house's, the neutral fallback when the disguise has no house)
                let c = crate::cloak::view::radar_house(w, id, Some(h))
                    .map_or(view.neutral, |ow| (view.house_colour)(ow));
                let mut rc = radar_colour(c, view.bits);
                let t_left = (view.flash)(id);
                // spec: S-0410 claim 10 (only objects of the shown house flash)
                if o.is_some_and(|o| o.owner == Some(h))
                    && t_left >= 1
                    && flash_inverted(t_left, view.flash_frame_time)
                {
                    rc = inverted(rc, view.bits);
                }
                out.push(rc);
                continue;
            }
            let c = cell_at_pixel(g, img, p, q, shift);
            if in_friendly_gap(w, c, &gaps, view.on_flat_grid) {
                // spec: S-0410 claim 6 (in a friendly gap field: the pixel's own terrain colour, halved)
                out.push(halved(radar_colour(t, view.bits)));
            } else if cell_shrouded_on(w, h, c, view.on_flat_grid) {
                // spec: S-0410 claim 6 (shrouded: black)
                out.push([0, 0, 0]);
            } else {
                out.push(radar_colour(t, view.bits));
            }
        }
    }
    out
}

/// The imported radar-event zoom settings (S-0410 F9, Import mapping).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ZoomRules {
    /// The contraction speed *S*, pixels per drawn frame, single.
    pub speed: F32,
    /// The minimum radius *M*, pixels.
    pub min_radius: i32,
    /// The initial angular speed *Ω*, radians per drawn frame, single.
    pub rotation: F32,
}

/// A radar event's zoom square (claims 14 and 15, F9): radius *R*, angle
/// *θ* and angular increment *ω*, all single precision.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Zoom {
    /// The radius *R* in pixels.
    pub radius: F32,
    /// The angle *θ* in radians.
    pub angle: F32,
    /// The angular increment *ω* in radians per drawn frame.
    pub step: F32,
}

/// The centre (*p*, *q*) of an event at usable cell (*x*, *y*) relative to
/// the radar image, whose top-left pixel is `origin` on the screen (F9).
pub fn event_centre(g: &Grid, img: &Image, origin: (i32, i32), x: i32, y: i32) -> (i32, i32) {
    let s = img.s.to_f64();
    // spec: S-0410 F9 (p = trunc((x − y + a) × s + Oₓ) − Oₓ, q = trunc((x + y − b) × s + Oᵧ) − Oᵧ, in double precision)
    let p = d(i64::from(x) - i64::from(y) + i64::from(g.a))
        .mul(s, RZ)
        .add(d(i64::from(origin.0)), RZ)
        .to_i32_common()
        - origin.0;
    let q = d(i64::from(x) + i64::from(y) - i64::from(g.b))
        .mul(s, RZ)
        .add(d(i64::from(origin.1)), RZ)
        .to_i32_common()
        - origin.1;
    // spec: S-0410 F9 (a resulting column −1 is replaced by 0)
    (if p == -1 { 0 } else { p }, q)
}

/// *RZ₃₂* of F9: rounding toward zero to single precision.
fn rz32(v: F64) -> F32 {
    v.to_f32(RZ)
}

/// F9's angle wrap: above 6.283185307179586, that value subtracted once.
fn wrap_angle(t: F32) -> F32 {
    let full = nearest_double("6.283185307179586"); // spec: S-0410 F9 (strictly greater than 6.283185307179586 radians)
    if t.to_f64().gt(full) {
        // spec: S-0410 F9 (replaced by RZ₃₂ of that value minus 6.283185307179586, once)
        rz32(t.to_f64().sub(full, RZ))
    } else {
        t
    }
}

impl Zoom {
    /// The initial square of an event centred at (*p*, *q*) of an image of
    /// `w` × `h` pixels (F9).
    pub fn new(r: &ZoomRules, centre: (i32, i32), w: i32, h: i32) -> Self {
        let (p, q) = centre;
        // spec: S-0410 F9 (R₀ = max(p, q, w − p, h − q) pixels)
        let r0 = p.max(q).max(w - p).max(h - q);
        Zoom {
            radius: rz32(d(i64::from(r0))),
            // spec: S-0410 F9 (θ₀ = 0.7853981852531433 radians)
            angle: rz32(nearest_double("0.7853981852531433")),
            step: r.rotation, // spec: S-0410 F9 (ω₀ = Ω)
        }
    }

    /// One drawn frame's advance (claim 14, F9); true when zoom completes
    /// on this frame.
    pub fn advance(&mut self, r: &ZoomRules) -> bool {
        let m = d(i64::from(r.min_radius));
        // spec: S-0410 F9 (ρ = max(Rₙ − S, RZ₃₂(M)), Rₙ₊₁ = RZ₃₂(ρ))
        let rho = self
            .radius
            .to_f64()
            .sub(r.speed.to_f64(), RZ)
            .max(rz32(m).to_f64());
        self.radius = rz32(rho);
        // spec: S-0410 F9 (a = θₙ + 0.7853981633974483; k = trunc(a × 0.6366197723675814); d = RZ₃₂(a − k × 1.5707963267948966))
        let a = self
            .angle
            .to_f64()
            .add(nearest_double("0.7853981633974483"), RZ);
        let k = a
            .mul(nearest_double("0.6366197723675814"), RZ)
            .trunc_i128()
            .unwrap_or(0);
        let k = F64::from_i64(i64::try_from(k).unwrap_or(0), RZ);
        let rem = rz32(a.sub(k.mul(nearest_double("1.5707963267948966"), RZ), RZ));
        // spec: S-0410 F9 (|ρ − M| < 0.01 pixels; equality does not complete)
        let near = rho.sub(m, RZ).abs().lt(nearest_double("0.01"));
        let omega = self.step;
        // spec: S-0410 F9 (completes when |ρ − M| < 0.01 and d < ωₙ, strictly)
        if near && rem.to_f64().lt(omega.to_f64()) {
            // spec: S-0410 F9 (the angle on completion becomes RZ₃₂(θₙ + d))
            self.angle = wrap_angle(rz32(self.angle.to_f64().add(rem.to_f64(), RZ)));
            return true;
        }
        // spec: S-0410 F9 (otherwise the next angle is RZ₃₂(θₙ + ωₙ))
        self.angle = wrap_angle(rz32(self.angle.to_f64().add(omega.to_f64(), RZ)));
        if near {
            // spec: S-0410 F9 (ωₙ₊₁ = RZ₃₂(max(Ω × 0.3333333432674408, RZ₃₂(ωₙ − Ω × 0.019999999552965164))))
            let big = r.rotation.to_f64();
            let floor = big.mul(nearest_double("0.3333333432674408"), RZ);
            let slowed = rz32(
                omega
                    .to_f64()
                    .sub(big.mul(nearest_double("0.019999999552965164"), RZ), RZ),
            );
            self.step = rz32(floor.max(slowed.to_f64()));
        }
        // spec: S-0410 F9 (|ρ − M| ≥ 0.01: ωₙ₊₁ = ωₙ)
        false
    }
}
