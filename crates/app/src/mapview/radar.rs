//! The live game's radar map in the viewer (S-0410, drawn by our M5a
//! placeholder sidebar): the point grid, the radar image and the terrain
//! picture are worked out once per game, the arrivals are observed after
//! every tick, and the pixels are composed for the controlled house when the
//! sidebar draws. The point colours come from the cells' tile radar colours
//! only (claim 9's tile case); terrain objects, bridges and overlays do not
//! colour their points yet.
//!
//! The cell each pixel shows (F3) is pure in the grid, the image and
//! `quirks.radar_pixel_cell_shift`, so it is worked out once per game too:
//! [`minimap::compose_kept`] gives [`minimap::compose`]'s pixels with those cells kept
//! (M8 step C4 part 3: worked out afresh for every pixel of every tick it was
//! most of a late frame's interface time).
//!
//! The picture of a house is kept between compositions in [`Layers`] (render
//! perf batch RP11, #137): each pixel's shroud test and its terrain, halved
//! or black, are worked out again only for the pixels whose inputs changed
//! since the last picture; the objects are placed afresh every time.

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use glasnost_map::{CellXY, Footprint, Theater};
use glasnost_rules::sim_econ::EconFields;
use glasnost_sim::econ::minimap::{
    self, Arrivals, Bits, Grid, Image, PointSource, RadarColour, Rgb, View,
};
use glasnost_sim::econ::{radar, shroud, victory};
use glasnost_sim::rules::{Kind, SimRules};
use glasnost_sim::world::{HouseId, Object, ObjectId, Pos, World};

/// The radar map's size in pixels (S-0410 Constants).
pub const SIZE: (i32, i32) = (140, 108); // spec: S-0410 Constants (radar map 140 × 108 pixels)

/// Bits per channel of our display surface after S-0103's 16-bit step:
/// 5, 6 and 5 (S-0410 Terms).
pub(crate) const DISPLAY_BITS: Bits = Bits([5, 6, 5]); // spec: S-0410 Terms (5 bits red, 6 green, 5 blue)

/// The colour of a house index the colour list lacks (our placeholder).
const NO_HOUSE_COLOUR: Rgb = [160, 160, 160]; // nocite: placeholder colour of our viewer

/// The radar state of one game.
pub struct Radar {
    grid: Grid,
    img: Image,
    terrain: Vec<Rgb>,
    arrivals: Arrivals,
    /// Each house's colour, by house index (S-0410 claim 8).
    colours: Vec<Rgb>,
    /// The neutral fallback colour (S-0410 claim 8, S-0516 claim 38).
    neutral: Rgb,
    /// The cell shown at each pixel of the image, row-major ([`minimap::pixel_cells`]).
    cells: Vec<CellXY>,
    /// The `quirks.radar_pixel_cell_shift` that `cells` were worked out with.
    shift: bool,
    /// The layers of the last picture (render perf batch RP11, #137).
    kept: RefCell<Option<Layers>>,
}

impl Radar {
    /// The radar of a game, from each cell's tile radar colour (S-0010
    /// claim 20), with each house's colour by house index
    /// ([`super::load::Loaded::house_radar_colours`]); `None` when the map
    /// has no usable cell. `neutral` is the fixed neutral fallback colour
    /// of an ownerless radar object and of a disguise without a disguise
    /// house ([`super::load::Loaded::neutral_radar_colour`]).
    pub fn new(
        w: &World,
        theater: Theater,
        tile_colours: &BTreeMap<CellXY, Rgb>,
        colours: Vec<Rgb>,
        neutral: Rgb,
    ) -> Option<Self> {
        let grid = minimap::grid(w)?;
        let img = minimap::image(&grid);
        let snow = theater == Theater::Snow;
        // spec: S-0410 Terms (a radar point that no usable cell covers has the point colour (0, 0, 0))
        let mut points = vec![[0u8; 3]; (grid.w.max(0) * grid.n.max(0)) as usize];
        for (c, _) in w.cells.grid.iter() {
            if !w.cells.usable(c) {
                continue;
            }
            // spec: S-0410 claim 9 (the tile case: the first tile radar colour of the tile cell)
            let colour = minimap::point_colour(PointSource::Tile {
                colour: tile_colours.get(&c).copied(),
                snow,
            });
            for (u, v) in minimap::cell_points(grid.a, grid.b, grid.lw, c) {
                if (0..grid.w).contains(&u) && (0..grid.n).contains(&v) {
                    points[(v * grid.w + u) as usize] = colour;
                }
            }
        }
        let terrain = minimap::terrain_picture(&grid, &img, &points);
        let shift = w.rules().quirks.radar_pixel_cell_shift;
        let cells = minimap::pixel_cells(&grid, &img, shift);
        Some(Self {
            grid,
            img,
            terrain,
            arrivals: Arrivals::default(),
            colours,
            neutral,
            cells,
            shift,
            kept: RefCell::new(None),
        })
    }

    /// Observes the arrivals after a tick (S-0410 claim 8); a watch game
    /// (`None`) keeps none, as [`Radar::watch_pixels`] reads no arrival.
    pub fn observe(&mut self, w: &World, house: Option<HouseId>) {
        match house {
            Some(h) => self.arrivals.observe(w, h, &self.grid, &self.img),
            None => self.arrivals = Arrivals::default(),
        }
    }

    /// The radar map of `house` as RGBA pixels of [`SIZE`]: black outside
    /// the radar image, and all black while the house has no radar
    /// (S-0410 claim 1). With no house (a watch game) it is the whole map
    /// with every object, by [`Radar::watch_pixels`].
    pub fn rgba(&self, w: &World, house: Option<HouseId>) -> Vec<u8> {
        self.rgba_by(w, house, |view| {
            if self.shift != w.rules().quirks.radar_pixel_cell_shift {
                return minimap::compose(
                    w,
                    view,
                    &self.arrivals,
                    &self.grid,
                    &self.img,
                    &self.terrain,
                );
            }
            let mut kept = self.kept.borrow_mut();
            match kept.as_mut() {
                Some(l) if l.fits(self, w, view) => l.update(self, w),
                _ => *kept = Some(Layers::new(self, w, view)),
            }
            kept.as_ref()
                .map(|l| l.picture(self, w, view, &self.arrivals))
                .unwrap_or_default()
        })
    }

    /// [`Radar::rgba`] by the simulation's [`minimap::compose_kept`], with
    /// nothing kept between pictures: the equivalence tests' old form.
    #[cfg(test)]
    pub(crate) fn rgba_composed(&self, w: &World, house: Option<HouseId>) -> Vec<u8> {
        self.rgba_by(w, house, |view| {
            minimap::compose_kept(
                w,
                view,
                &self.arrivals,
                &self.grid,
                &self.img,
                &self.terrain,
                &self.cells,
            )
        })
    }

    /// [`Radar::rgba`] with the radar image of a shown house composed by
    /// `compose`.
    fn rgba_by(
        &self,
        w: &World,
        house: Option<HouseId>,
        compose: impl FnOnce(&View) -> Vec<RadarColour>,
    ) -> Vec<u8> {
        let (mw, mh) = SIZE;
        let mut out = vec![0u8; (mw * mh * 4) as usize];
        for px in out.as_chunks_mut::<4>().0 {
            px[3] = 255;
        }
        let Some(house) = house else {
            let pixels = self.watch_pixels(w);
            self.blit(&pixels, &mut out);
            return out;
        };
        if !radar::available(w, house) {
            return out;
        }
        // spec: S-0410 claim 8 (a shown object in its owner's house colour)
        let colour = |h: HouseId| {
            self.colours
                .get(usize::from(h.0))
                .copied()
                .unwrap_or(NO_HOUSE_COLOUR)
        };
        let flash = |_| 0;
        let view = View {
            house,
            house_colour: &colour,
            neutral: self.neutral,
            flash: &flash,
            flash_frame_time: 1, // nocite: no object flashes in the viewer yet
            bits: DISPLAY_BITS,
            // spec: S-0410 claim 6 (the radar shares S-0110's quirks.shroud_on_flat_grid)
            on_flat_grid: glasnost_render::shroud::ShroudQuirks::default().shroud_on_flat_grid,
        };
        let pixels = compose(&view);
        self.blit(&pixels, &mut out);
        out
    }

    /// Copies the radar image's pixels into the RGBA map of [`SIZE`].
    fn blit(&self, pixels: &[minimap::RadarColour], out: &mut [u8]) {
        let (mw, mh) = SIZE;
        for q in 0..self.img.height {
            for p in 0..self.img.width {
                let (x, y) = (self.img.left + p, self.img.top + q);
                if !(0..mw).contains(&x) || !(0..mh).contains(&y) {
                    continue;
                }
                let Some(c) = pixels.get((q * self.img.width + p) as usize) else {
                    continue;
                };
                let at = ((y * mw + x) * 4) as usize;
                for ch in 0..3 {
                    // Back to 8 bits for the screen: the channel's high bits.
                    out[at + ch] = (c[ch] << (8 - DISPLAY_BITS.0[ch])) as u8;
                }
            }
        }
    }

    /// The radar image of a watch game, row-major.
    ///
    /// design: watch mode (lead request 2026-11-02, an extra not in the
    /// original): no house is shown, so nothing is shrouded and no gap
    /// field halves a pixel; every point shows its terrain colour. Each
    /// radar object is shown as its own house sees it: on the map, inside
    /// the image, not of an invisible type, and not insignificant unless
    /// radar-visible or owned by an active country (the tests of S-0410
    /// claim 7 that do not depend on the shown house). A building takes its
    /// F8 pixel set and anything else its F4 pixel unclamped. Where several
    /// objects meet at one pixel, the one with the highest object id is
    /// shown (our order: the radar's arrival order is kept per house).
    pub fn watch_pixels(&self, w: &World) -> Vec<minimap::RadarColour> {
        let mut out: Vec<minimap::RadarColour> = self
            .terrain
            .iter()
            .map(|&t| minimap::radar_colour(t, DISPLAY_BITS))
            .collect();
        let size = (self.img.width.max(0) * self.img.height.max(0)) as usize;
        out.resize(size, [0, 0, 0]);
        // The objects in id order, a later one painting over an earlier.
        for o in w.objects.values() {
            if !watch_listed(w, o) {
                continue;
            }
            // spec: S-0410 claim 8 (a shown object in its owner's house colour; the neutral colour without an owner)
            let c = o.owner.map_or(self.neutral, |h| {
                self.colours
                    .get(usize::from(h.0))
                    .copied()
                    .unwrap_or(NO_HOUSE_COLOUR)
            });
            let rc = minimap::radar_colour(c, DISPLAY_BITS);
            let pixels = if o.kind == Kind::Building {
                // spec: S-0410 F8 (around the unclamped F4 pixel of its stored position)
                let top = minimap::object_pixel(&self.grid, &self.img, o.pos.x, o.pos.y, false);
                minimap::building_pixels(&self.img, top, o.foundation)
            } else {
                vec![minimap::object_pixel(
                    &self.grid, &self.img, o.pos.x, o.pos.y, false,
                )]
            };
            for (p, q) in pixels {
                // spec: S-0410 F8 (pixels outside the radar image are omitted, never clamped)
                if (0..self.img.width).contains(&p) && (0..self.img.height).contains(&q) {
                    out[(q * self.img.width + p) as usize] = rc;
                }
            }
        }
        out
    }

    /// The radar image's rectangle in the radar map: left, top, width,
    /// height (S-0410 F2).
    pub fn image_rect(&self) -> (i32, i32, i32, i32) {
        (self.img.left, self.img.top, self.img.width, self.img.height)
    }

    /// The outline of the battlefield view on the radar map (S-0410 claim
    /// 11: the outline that marks the battlefield view's part of the map is
    /// drawn over the radar map), in radar map pixels: the left, top, right
    /// and bottom edges, unclipped (the drawing clips them to the image).
    /// `origin` and `size` are the view's top-left world pixel and its size
    /// (S-0101 F3).
    ///
    /// design: the view's corners are taken at height level 0 and placed
    /// by F4, as an object at that ground point would be; the outline's
    /// look and the height it reads are not specified ("other details
    /// belong to the interface specs").
    pub fn view_outline(&self, origin: (i32, i32), size: (i32, i32)) -> (i32, i32, i32, i32) {
        let corner = |px: i32, py: i32| {
            let (px, py) = (i64::from(px), i64::from(py));
            // The inverse of S-0101 F1 at height 0: 30(X − Y) = 256 px and
            // 15(X + Y) = 256 py, so X = (128 px + 256 py) / 30 and
            // Y = (256 py − 128 px) / 30.
            let x = (128 * px + 256 * py).div_euclid(30);
            let y = (256 * py - 128 * px).div_euclid(30);
            let (p, q) = minimap::object_pixel(&self.grid, &self.img, x as i32, y as i32, false);
            (p + self.img.left, q + self.img.top)
        };
        let (l, t) = corner(origin.0, origin.1);
        let (r, b) = corner(origin.0 + size.0, origin.1 + size.1);
        (l, t, r, b)
    }

    /// The cell a click on radar map pixel (x, y) names (S-0410 F3), when
    /// it is inside the radar image.
    pub fn cell_at(&self, w: &World, x: i32, y: i32) -> Option<CellXY> {
        let (p, q) = (x - self.img.left, y - self.img.top);
        if !(0..self.img.width).contains(&p) || !(0..self.img.height).contains(&q) {
            return None;
        }
        let shift = w.rules().quirks.radar_pixel_cell_shift;
        Some(minimap::cell_at_pixel(&self.grid, &self.img, p, q, shift))
    }
}

/// Whether a watch game's radar shows an object ([`Radar::watch_pixels`]):
/// S-0410 claim 7's tests that do not depend on the shown house.
fn watch_listed(w: &World, o: &Object) -> bool {
    // spec: S-0410 Terms (only radar objects: infantry, vehicles, aircraft, buildings)
    if o.kind == Kind::Terrain {
        return false;
    }
    let (econ, insignificant) = type_fields(w, o);
    // spec: S-0410 Import mapping (Invisible, RadarVisible, Insignificant absent: no)
    let invisible = econ.and_then(|e| e.invisible).unwrap_or(false);
    // spec: S-0410 claim 7 (on the map, not inside a transport or building, type not invisible)
    if !o.on_map || invisible {
        return false;
    }
    !hidden_insignificant(w, o, econ, insignificant)
}

/// An object type's economy fields and its insignificant flag (S-0410
/// Import mapping); none for a terrain object or a type the rules lack.
fn type_fields<'w>(w: &'w World, o: &Object) -> (Option<&'w EconFields>, Option<bool>) {
    let r = w.rules();
    match o.kind {
        Kind::Terrain => (None, None),
        Kind::Building => match r.buildings.get(o.type_index as usize) {
            Some(b) => (Some(&b.raw.econ), b.raw.combat.insignificant),
            None => (None, None),
        },
        _ => match r.unit(o.kind, o.type_index) {
            Some(u) => (Some(&u.raw.econ), u.raw.combat.insignificant),
            None => (None, None),
        },
    }
}

/// Whether claim 7's insignificance test leaves the object off the radar.
fn hidden_insignificant(
    w: &World,
    o: &Object,
    econ: Option<&EconFields>,
    insignificant: Option<bool>,
) -> bool {
    // spec: S-0410 Import mapping (RadarVisible, Insignificant absent: no)
    let radar_visible = econ.and_then(|e| e.radar_visible).unwrap_or(false);
    // spec: S-0410 claim 7 (insignificant, unless radar-visible or owned by a house whose country is not passive)
    if insignificant.unwrap_or(false) && !radar_visible {
        let active_owner = o.owner.is_some_and(|ow| {
            w.houses
                .get(usize::from(ow.0))
                .and_then(|h| w.rules().countries.get(h.country as usize))
                .is_some_and(|c| !c.passive)
        });
        return !active_owner;
    }
    false
}

/// The coordinate range of the point test's candidate cells (S-0110 Edge
/// cases); a candidate outside it may wrap (`quirks.shroud_point_cell_wrap`).
const POINT_CELL_RANGE: std::ops::RangeInclusive<i32> = 0..=511; // spec: S-0110 Edge cases (the coordinate range 0 … 511)

/// The pixels that read each shroud state, by dense cell index (the last
/// index is the state that the cells that are not legal share, S-0409
/// claim 14): `of[starts[j]..starts[j + 1]]` for index *j*.
#[derive(Default)]
struct Readers {
    starts: Vec<u32>,
    of: Vec<u32>,
}

impl Readers {
    fn of(&self, j: usize) -> &[u32] {
        match (self.starts.get(j), self.starts.get(j + 1)) {
            (Some(&a), Some(&b)) => &self.of[a as usize..b as usize],
            _ => &[],
        }
    }
}

/// The kept layers of one house's radar picture (render perf batch RP11,
/// #137).
///
/// design: render (#137). Below the objects, a pixel's colour (S-0410
/// claim 6: halved terrain in a friendly gap field, black when shrouded,
/// else terrain) and its shroud test (which claim 7's draw-time test reads
/// too) depend on: the house, the rules (their quirks), the renderer's
/// `quirks.shroud_on_flat_grid`, the map's footprint (usable area), the
/// level and slope of the pixel's cell (its usability and its ground point,
/// S-0106), the house's shroud marks at the point test's candidate cells,
/// and the friendly gap cells. The first four are the key: any change
/// composes every pixel again. The pixel's cell is checked every picture,
/// and the marks and gap cells are compared whole; a pixel is composed
/// again when its cell's ground changed, when a mark or gap cell its
/// candidates read changed, or always when a candidate lies outside
/// [`POINT_CELL_RANGE`] (where the point test may wrap). The objects are
/// placed on top afresh every picture.
struct Layers {
    house: HouseId,
    rules: Arc<SimRules>,
    on_flat_grid: bool,
    footprint: Footprint,
    /// The shroud's shared index, the number of legal cells.
    shared: usize,
    /// The house's shroud marks at the last picture; none when it keeps no
    /// shroud state (S-0409: such a house has every cell explored).
    marks: Option<Vec<u8>>,
    /// The cells of the friendly gap fields at the last picture.
    gaps: BTreeSet<u32>,
    /// Each pixel's cell's level and slope; none for a cell that is not legal.
    ground: Vec<Option<(i8, u8)>>,
    readers: Readers,
    /// The pixels composed again every picture.
    volatile: Vec<u32>,
    /// Each pixel's terrain at the display's bits.
    plain: Vec<RadarColour>,
    /// Each pixel's shroud test (S-0410 claim 6).
    shrouded: Vec<bool>,
    /// Each pixel's colour below the objects.
    base: Vec<RadarColour>,
}

impl Layers {
    /// Every layer of `view.house`'s picture, worked out afresh.
    fn new(r: &Radar, w: &World, view: &View) -> Self {
        let n = r.cells.len();
        let h = view.house;
        let mut l = Self {
            house: h,
            rules: w.rules_arc(),
            on_flat_grid: view.on_flat_grid,
            footprint: *w.cells.grid.footprint(),
            shared: w.cells.grid.len(),
            marks: house_marks(w, h).map(<[u8]>::to_vec),
            gaps: friendly_gap_cells(w, h),
            ground: r.cells.iter().map(|&c| ground_of(w, c)).collect(),
            readers: Readers::default(),
            volatile: Vec::new(),
            plain: (0..n)
                .map(|i| {
                    let t = r.terrain.get(i).copied().unwrap_or([0, 0, 0]);
                    minimap::radar_colour(t, view.bits)
                })
                .collect(),
            shrouded: vec![false; n],
            base: vec![[0, 0, 0]; n],
        };
        l.index(r, w);
        for i in 0..n {
            l.compose(r, w, i);
        }
        l
    }

    /// Whether the key of the layers is still the world's and the view's.
    fn fits(&self, r: &Radar, w: &World, view: &View) -> bool {
        self.house == view.house
            && std::ptr::eq(Arc::as_ptr(&self.rules), w.rules())
            && self.on_flat_grid == view.on_flat_grid
            && self.footprint == *w.cells.grid.footprint()
            && self.shared == w.cells.grid.len()
            && self.marks.as_ref().map(Vec::len) == house_marks(w, view.house).map(<[u8]>::len)
            && self.plain.len() == r.cells.len()
    }

    /// Finds the pixels each shroud state is read by, and the pixels whose
    /// candidates may wrap.
    fn index(&mut self, r: &Radar, w: &World) {
        let quirk = w.rules().quirks.shroud_negative_odd_pair;
        let mut pairs: Vec<(u32, u32)> = Vec::new();
        self.volatile.clear();
        for (i, &c) in r.cells.iter().enumerate() {
            if !w.cells.usable(c) {
                continue;
            }
            let (a, b) = shroud::point_candidates_on(ground_centre(w, c), quirk, self.on_flat_grid);
            for k in std::iter::once(a).chain(b) {
                if !POINT_CELL_RANGE.contains(&k.x) || !POINT_CELL_RANGE.contains(&k.y) {
                    self.volatile.push(i as u32);
                    continue;
                }
                // spec: S-0409 claim 14 (a cell that is not legal reads the shared state)
                let j = w.cells.idx(k).unwrap_or(self.shared);
                pairs.push((j as u32, i as u32));
            }
        }
        pairs.sort_unstable();
        pairs.dedup();
        self.volatile.dedup();
        let mut starts = vec![0u32; self.shared + 2];
        for &(j, _) in &pairs {
            if let Some(s) = starts.get_mut(j as usize + 1) {
                *s += 1;
            }
        }
        for j in 1..starts.len() {
            starts[j] += starts[j - 1];
        }
        self.readers = Readers {
            starts,
            of: pairs.into_iter().map(|(_, i)| i).collect(),
        };
    }

    /// Brings the layers up to the world: composes again the pixels whose
    /// inputs changed since the last picture.
    fn update(&mut self, r: &Radar, w: &World) {
        let mut again: Vec<u32> = Vec::new();
        let mut moved = false;
        for (i, &c) in r.cells.iter().enumerate() {
            let g = ground_of(w, c);
            if g != self.ground[i] {
                self.ground[i] = g;
                again.push(i as u32);
                moved = true;
            }
        }
        if moved {
            self.index(r, w);
        }
        if let (Some(old), Some(now)) = (self.marks.as_mut(), house_marks(w, self.house)) {
            const CHUNK: usize = 64; // nocite: our comparison's chunk of marks
            for (k, (a, b)) in old.chunks_mut(CHUNK).zip(now.chunks(CHUNK)).enumerate() {
                if a == b {
                    continue;
                }
                for (j, (x, y)) in a.iter().zip(b).enumerate() {
                    if x != y {
                        again.extend_from_slice(self.readers.of(k * CHUNK + j));
                    }
                }
                a.copy_from_slice(b);
            }
        }
        let gaps = friendly_gap_cells(w, self.house);
        if gaps != self.gaps {
            for &j in gaps.symmetric_difference(&self.gaps) {
                again.extend_from_slice(self.readers.of(j as usize));
            }
            self.gaps = gaps;
        }
        again.extend_from_slice(&self.volatile);
        for i in again {
            self.compose(r, w, i as usize);
        }
    }

    /// Composes pixel `i` below the objects (S-0410 claim 6).
    fn compose(&mut self, r: &Radar, w: &World, i: usize) {
        let c = r.cells[i];
        let shrouded = minimap::cell_shrouded_on(w, self.house, c, self.on_flat_grid);
        let t = self.plain[i];
        self.shrouded[i] = shrouded;
        self.base[i] = if in_friendly_gap(w, c, &self.gaps, self.on_flat_grid) {
            // spec: S-0410 claim 6 (in a friendly gap field: the pixel's own terrain colour, halved)
            minimap::halved(t)
        } else if shrouded {
            // spec: S-0410 claim 6 (shrouded: black)
            [0, 0, 0]
        } else {
            t
        };
    }

    /// The picture: the kept colours with the objects on top (S-0410
    /// claims 6 and 8).
    fn picture(&self, r: &Radar, w: &World, view: &View, arrivals: &Arrivals) -> Vec<RadarColour> {
        let h = self.house;
        let mut out = self.base.clone();
        let (iw, ih) = (r.img.width.max(0), r.img.height.max(0));
        // spec: S-0410 claim 8 (only a pixel some object is listed at can have a winner, among the objects listed there)
        let mut listed: Vec<(usize, (i32, usize), ObjectId)> = arrivals
            .at
            .iter()
            .filter(|&(&(_, (p, q)), _)| (0..iw).contains(&p) && (0..ih).contains(&q))
            .map(|(&(id, (p, q)), &arr)| ((q * iw + p) as usize, arr, id))
            .collect();
        listed.sort_unstable();
        for at in listed.chunk_by(|a, b| a.0 == b.0) {
            let i = at[0].0;
            let own = |id: ObjectId| w.obj(id).is_some_and(|o| o.owner == Some(h));
            // spec: S-0410 claim 8 (own objects first, the latest arrival first; then the others, the earliest first; the first that passes the draw-time tests)
            let winner = at
                .iter()
                .rev()
                .filter(|x| own(x.2))
                .chain(at.iter().filter(|x| !own(x.2)))
                .map(|x| x.2)
                .find(|&id| shown(w, h, id, self.shrouded[i]));
            let Some(id) = winner else { continue };
            // spec: S-0410 claim 8 and S-0516 claim 38 (a disguised unit: its owner's colour when the owner allies H, otherwise its disguise house's, the neutral fallback when the disguise has no house)
            let c = glasnost_sim::cloak::view::radar_house(w, id, Some(h))
                .map_or(view.neutral, |ow| (view.house_colour)(ow));
            let mut rc = minimap::radar_colour(c, view.bits);
            let t_left = (view.flash)(id);
            // spec: S-0410 claim 10 (only objects of the shown house flash)
            if own(id) && t_left >= 1 && minimap::flash_inverted(t_left, view.flash_frame_time) {
                rc = minimap::inverted(rc, view.bits);
            }
            out[i] = rc;
        }
        out
    }
}

/// The draw-time tests of S-0410 claim 7 for a listed object at a pixel
/// whose cell's shroud test is `shrouded`.
fn shown(w: &World, h: HouseId, id: ObjectId, shrouded: bool) -> bool {
    let Some(o) = w.obj(id) else { return false };
    // spec: S-0410 claim 7 (not owned by H: the cell shown at its pixel is not shrouded)
    if o.owner != Some(h) && shrouded {
        return false;
    }
    let (econ, insignificant) = type_fields(w, o);
    // spec: S-0410 Import mapping (RadarInvisible absent: no) and claim 7 (radar-invisible, unless H allies its owner, H's side)
    if econ.and_then(|e| e.radar_invisible).unwrap_or(false) && !w.allied(Some(h), o.owner) {
        return false;
    }
    !hidden_insignificant(w, o, econ, insignificant)
}

/// The house's shroud marks, none when it keeps no shroud state (S-0409
/// claim 1).
fn house_marks(w: &World, h: HouseId) -> Option<&[u8]> {
    w.econ
        .shroud
        .houses
        .get(usize::from(h.0))?
        .as_ref()
        .map(|s| s.marks.as_slice())
}

/// A cell's level and slope, which its usability and its ground point read.
fn ground_of(w: &World, c: CellXY) -> Option<(i8, u8)> {
    w.cells.cell(c).map(|x| (x.level, x.slope))
}

/// The point the radar's shroud and gap tests take for a cell (S-0410
/// claim 6).
fn ground_centre(w: &World, c: CellXY) -> Pos {
    let p = Pos::centre(c, 0);
    // spec: S-0410 claim 6 (the cell surface height at its centre, S-0106 claims 2 and 3; a slope's centre may lie above its level)
    let z = w.surface_at(p.x, p.y);
    Pos::new(p.x, p.y, z)
}

/// The cells covered by gap fields friendly to the house (S-0410 Terms):
/// its own or an ally's (its side), or any field once it is defeated.
fn friendly_gap_cells(w: &World, h: HouseId) -> BTreeSet<u32> {
    let defeated = victory::defeated(w, h);
    let mut out = BTreeSet::new();
    for o in w.objects.values() {
        // spec: S-0410 Terms (a friendly gap field: the house's own or an ally's, any once the house is defeated)
        if let Some(g) = &o.econ.sight.gap
            && (defeated || w.allied(Some(h), g.owner))
        {
            out.extend(g.cells.iter().copied());
        }
    }
    out
}

/// Whether a cell lies in a friendly gap field (S-0410 claim 6).
fn in_friendly_gap(w: &World, c: CellXY, cells: &BTreeSet<u32>, on_flat_grid: bool) -> bool {
    // spec: S-0410 claim 6 (in a friendly gap field only when its candidate cells are covered by one: with no covered cell, none is)
    if cells.is_empty() || !w.cells.usable(c) {
        return false;
    }
    // spec: S-0410 claim 6, S-0110 claim 8 (the same candidate cells as the shroud test, heights below 0 included, quirks.shroud_negative_odd_pair; quirks.shroud_on_flat_grid off takes precedence)
    let (a, b) = shroud::point_candidates_on(
        ground_centre(w, c),
        w.rules().quirks.shroud_negative_odd_pair,
        on_flat_grid,
    );
    let inside = |c: CellXY| w.cells.idx(c).is_some_and(|i| cells.contains(&(i as u32)));
    // spec: S-0410 claim 6 (for odd z the gap test needs both candidate cells in a friendly gap field)
    inside(a) && b.is_none_or(inside)
}
