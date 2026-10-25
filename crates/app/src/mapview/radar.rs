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
use glasnost_sim::world::{HouseId, World};

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

    /// Observes the arrivals after a tick (S-0410 claim 8).
    pub fn observe(&mut self, w: &World, house: HouseId) {
        self.arrivals.observe(w, house, &self.grid, &self.img);
    }

    /// The radar map of `house` as RGBA pixels of [`SIZE`]: black outside
    /// the radar image, and all black while the house has no radar
    /// (S-0410 claim 1).
    pub fn rgba(&self, w: &World, house: HouseId) -> Vec<u8> {
        let (mw, mh) = SIZE;
        let mut out = vec![0u8; (mw * mh * 4) as usize];
        for px in out.as_chunks_mut::<4>().0 {
            px[3] = 255;
        }
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
        for q in 0..self.img.height {
            for p in 0..self.img.width {
                let (x, y) = (self.img.left + p, self.img.top + q);
                if !(0..mw).contains(&x) || !(0..mh).contains(&y) {
                    continue;
                }
                let c = pixels[(q * self.img.width + p) as usize];
                let at = ((y * mw + x) * 4) as usize;
                for ch in 0..3 {
                    // Back to 8 bits for the screen: the channel's high bits.
                    out[at + ch] = (c[ch] << (8 - DISPLAY_BITS.0[ch])) as u8;
                }
            }
        }
        out
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
