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

use std::collections::BTreeMap;

use glasnost_map::{CellXY, Theater};
use glasnost_sim::econ::minimap::{self, Arrivals, Bits, Grid, Image, PointSource, Rgb, View};
use glasnost_sim::econ::radar;
use glasnost_sim::rules::Kind;
use glasnost_sim::world::{HouseId, Object, World};

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
        let pixels = if self.shift == w.rules().quirks.radar_pixel_cell_shift {
            minimap::compose_kept(
                w,
                &view,
                &self.arrivals,
                &self.grid,
                &self.img,
                &self.terrain,
                &self.cells,
            )
        } else {
            minimap::compose(
                w,
                &view,
                &self.arrivals,
                &self.grid,
                &self.img,
                &self.terrain,
            )
        };
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
    let r = w.rules();
    let (econ, insignificant) = match o.kind {
        // spec: S-0410 Terms (only radar objects: infantry, vehicles, aircraft, buildings)
        Kind::Terrain => return false,
        Kind::Building => match r.buildings.get(o.type_index as usize) {
            Some(b) => (Some(&b.raw.econ), b.raw.combat.insignificant),
            None => (None, None),
        },
        _ => match r.unit(o.kind, o.type_index) {
            Some(u) => (Some(&u.raw.econ), u.raw.combat.insignificant),
            None => (None, None),
        },
    };
    // spec: S-0410 Import mapping (Invisible, RadarVisible, Insignificant absent: no)
    let invisible = econ.and_then(|e| e.invisible).unwrap_or(false);
    let radar_visible = econ.and_then(|e| e.radar_visible).unwrap_or(false);
    // spec: S-0410 claim 7 (on the map, not inside a transport or building, type not invisible)
    if !o.on_map || invisible {
        return false;
    }
    // spec: S-0410 claim 7 (insignificant, unless radar-visible or owned by a house whose country is not passive)
    if insignificant.unwrap_or(false) && !radar_visible {
        let active_owner = o.owner.is_some_and(|ow| {
            w.houses
                .get(usize::from(ow.0))
                .and_then(|h| r.countries.get(h.country as usize))
                .is_some_and(|c| !c.passive)
        });
        return active_owner;
    }
    true
}
