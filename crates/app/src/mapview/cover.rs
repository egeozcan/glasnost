//! Shroud covers of the live game (S-0110): the cover sprites read at load
//! and the light-alpha values of a view, from the simulation's explored
//! states for the house the user controls (S-0409).

use glasnost_fmt::shp::Shp;
use glasnost_map::CellXY;
use glasnost_render::projection::TERRAIN_LIFT;
use glasnost_render::scene::View;
use glasnost_render::shroud::{
    self, CANVAS_HEIGHT, CANVAS_WIDTH, Cover, CoverSprite, Covered, ShroudQuirks, Sprites,
};
use glasnost_sim::econ::shroud::explored;
use glasnost_sim::world::{HouseId, World};

use super::build::Assets;

/// The shroud sprite's file (S-0110 Import mapping, resolved by S-0005).
pub const SHROUD_FILE: &[u8] = b"SHROUD.SHP"; // spec: S-0110 Import mapping (file SHROUD.SHP)

/// The fog sprite's file (S-0110 Import mapping, resolved by S-0005).
pub const FOG_FILE: &[u8] = b"FOG.SHP"; // spec: S-0110 Import mapping (file FOG.SHP)

/// The two cover sprites (S-0110 claim 3), when the data has them.
#[derive(Clone, Debug, Default)]
pub struct CoverSprites {
    /// The shroud sprite.
    pub shroud: Option<CoverSprite>,
    /// The fog sprite.
    pub fog: Option<CoverSprite>,
}

impl CoverSprites {
    /// Reads both sprites from `assets`, with a warning for each that is
    /// missing or does not decode.
    pub fn load(assets: &Assets) -> (Self, Vec<String>) {
        let mut warnings = Vec::new();
        let mut read = |file: &[u8], what: &str| {
            let sprite = assets
                .get(file)
                .and_then(|b| Shp::parse(b).ok())
                .map(|s| CoverSprite::from_shp(&s));
            if sprite.is_none() {
                // spec: S-0110 Import mapping (a missing sprite draws no cover; the importer warns)
                warnings.push(format!(
                    "{} is missing or unreadable: {what} (S-0110)",
                    String::from_utf8_lossy(file)
                ));
            }
            sprite
        };
        let shroud = read(SHROUD_FILE, "no shroud pieces are drawn");
        let fog = read(FOG_FILE, "no pieces would be drawn with fog of war on");
        (Self { shroud, fog }, warnings)
    }
}

/// The covers of the legal cells whose canvas meets `view`, in the
/// observable order (S-0110 State and ordering: increasing *x* + *y*).
///
/// Only the cells whose canvas can reach the view are asked: a canvas's
/// column depends on *x* − *y* alone and its row on *x* + *y* and the cell's
/// level (S-0110 F2, S-0101 F1), so the candidates are a band of each,
/// widened by every level a cell can have; each candidate still takes the
/// full test. Explored states are read once per cell.
pub fn covers(world: &World, house: HouseId, view: &View, q: &ShroudQuirks) -> Vec<Cover> {
    let grid = &world.cells.grid;
    let (ox, oy) = view.origin;
    let (vw, vh) = (view.size.0 as i32, view.size.1 as i32);
    // How far a level can move a canvas up or down (S-0110 Edge cases: only with
    // quirks.shroud_on_flat_grid off; S-0101 claim 4).
    let (lift_lo, lift_hi) = if q.shroud_on_flat_grid {
        (0, 0)
    } else {
        (
            TERRAIN_LIFT * i32::from(i8::MIN),
            TERRAIN_LIFT * i32::from(i8::MAX),
        )
    };
    // S-0110 F2 with S-0101 F1: the canvas of (x, y) spans columns 30(x − y) − 30 to
    // 30(x − y) + 29 and rows 15(x + y) − lift to 15(x + y) − lift + 29; a band one cell
    // wider on each side than the inequalities need.
    let (cw, ch) = (CANVAS_WIDTH / 2, CANVAS_HEIGHT / 2);
    let u_lo = (ox - CANVAS_WIDTH).div_euclid(cw) - 1;
    let u_hi = (ox + vw + CANVAS_WIDTH).div_euclid(cw) + 1;
    let v_lo = (oy - CANVAS_HEIGHT + lift_lo).div_euclid(ch) - 1;
    let v_hi = (oy + vh + lift_hi).div_euclid(ch) + 1;
    let (x_lo, y_lo) = (
        (u_lo + v_lo).div_euclid(2) - 1,
        (v_lo - u_hi).div_euclid(2) - 1,
    );
    let (x_hi, y_hi) = (
        (u_hi + v_hi).div_euclid(2) + 2,
        (v_hi - u_lo).div_euclid(2) + 2,
    );
    let bw = (x_hi - x_lo + 1).max(0) as usize;
    let bh = (y_hi - y_lo + 1).max(0) as usize;
    // Each cell's answer once asked: 0 not yet, 1 covered, 2 explored.
    let memo: Vec<std::cell::Cell<u8>> = vec![std::cell::Cell::new(0); bw * bh];
    let covered = |x: i32, y: i32| -> bool {
        let at = (x >= x_lo && y >= y_lo && x <= x_hi && y <= y_hi)
            .then(|| &memo[(y - y_lo) as usize * bw + (x - x_lo) as usize]);
        match at.map(std::cell::Cell::get) {
            Some(1) => true,
            Some(2) => false,
            _ => {
                let v = !explored(world, house, CellXY::new(x, y));
                if let Some(m) = at {
                    m.set(if v { 1 } else { 2 });
                }
                v
            }
        }
    };
    let mut out = Vec::new();
    // spec: S-0110 State and ordering (cells by increasing x + y): v is x + y and, at one v,
    // x = (u + v) / 2 grows with u, so the loops emit the covers by (x + y, x) without a sort
    // (design: render (#119))
    for v in v_lo..=v_hi {
        // u and v have the parity of x + y and x − y: equal.
        let u_first = if (u_lo - v).rem_euclid(2) == 0 {
            u_lo
        } else {
            u_lo + 1
        };
        for u in (u_first..=u_hi).step_by(2) {
            let c = CellXY::new((u + v) / 2, (v - u) / 2);
            let Some(cell) = grid.get(c) else { continue };
            let level = i32::from(cell.level);
            // spec: S-0110 claim 1 (every legal cell whose canvas meets the battlefield view)
            if !shroud::canvas_in_view((c.x, c.y), level, view.origin, view.size, q) {
                continue;
            }
            // spec: S-0110 claims 1 and 2 (the explored states of the cell and its map neighbours;
            // a non-legal neighbour reads the shared state through S-0409 claim 14)
            let own = covered(c.x, c.y);
            let piece = shroud::cell_piece(own, Covered::of(c.x, c.y, covered));
            out.push(Cover {
                cell: (c.x, c.y),
                level,
                shroud: piece,
                // spec: S-0110 claim 4, Import mapping (fog of war is always off in skirmish: no fog piece)
                fog: None,
            });
        }
    }
    out
}

/// The light-alpha values of `view` (row by row) after the covers of the
/// house's explored states (S-0110 claims 1 to 10). No light-glow images
/// are drawn yet, so none act (claim 6).
pub fn light_alpha(
    world: &World,
    house: HouseId,
    view: &View,
    sprites: &CoverSprites,
    q: &ShroudQuirks,
) -> Vec<u8> {
    alpha_of(&covers(world, house, view, q), view, sprites, q)
}

/// The light-alpha values of `view` (row by row) after `covers` (the
/// [`covers`] of the view; [`light_alpha`] with them given).
pub fn alpha_of(
    covers: &[Cover],
    view: &View,
    sprites: &CoverSprites,
    q: &ShroudQuirks,
) -> Vec<u8> {
    shroud::light_alpha(view.origin, view.size, covers, sprites_of(sprites), &[], q)
}

/// The cover sprites as the light-alpha pass takes them.
fn sprites_of(sprites: &CoverSprites) -> Sprites<'_> {
    Sprites {
        shroud: sprites.shroud.as_ref(),
        fog: sprites.fog.as_ref(),
        // spec: S-0110 Import mapping (fog of war always off in skirmish)
        fog_of_war: false,
    }
}

/// [`alpha_of`] for `view`, given `kept`, the values of a view of the same
/// size at world pixel `kept_origin` after `kept_covers` (both cover lists
/// [`covers`] of their views, with the same sprites): the same values.
///
/// design: render (#118) (a scrolled view moves the kept values and makes
/// only the strips it uncovers): a pixel's value depends only on the covers
/// whose canvas holds it, in their order (S-0110 claims 5, 7 and 10), and
/// every cover whose canvas meets the part both views show is in both lists,
/// in the same order (State and ordering). So when those covers are the same
/// in both lists, that part keeps its values; the strips the view uncovers
/// are made from all of `covers`, each pixel clipped to its strip as the
/// view clips it. Otherwise, or with no part shown by both, the view is
/// made afresh.
pub fn alpha_moved(
    kept: Vec<u8>,
    kept_origin: (i32, i32),
    kept_covers: &[Cover],
    covers: &[Cover],
    view: &View,
    sprites: &CoverSprites,
    q: &ShroudQuirks,
) -> Vec<u8> {
    let (w, h) = (view.size.0 as i32, view.size.1 as i32);
    let (ox, oy) = view.origin;
    let (kx, ky) = kept_origin;
    // The world pixels both views show: [x0, x1) × [y0, y1).
    let (x0, x1) = (ox.max(kx), (ox + w).min(kx + w));
    let (y0, y1) = (oy.max(ky), (oy + h).min(ky + h));
    if kept.len() != (w.max(0) * h.max(0)) as usize || x0 >= x1 || y0 >= y1 {
        return alpha_of(covers, view, sprites, q);
    }
    // spec: S-0110 claim 7 (a cover acts only inside its 60 × 30 canvas)
    let meets = |c: &&Cover| {
        let (cl, ct) = shroud::canvas_origin(c.cell, c.level, q);
        cl < x1 && cl + CANVAS_WIDTH > x0 && ct < y1 && ct + CANVAS_HEIGHT > y0
    };
    if !kept_covers
        .iter()
        .filter(meets)
        .eq(covers.iter().filter(meets))
    {
        return alpha_of(covers, view, sprites, q);
    }
    let mut a = kept;
    // Each shown row moves from its kept place to its new one; rows are
    // taken before they are written over (moving up, from the top).
    let n = (x1 - x0) as usize;
    let mut move_row = |y: i32| {
        let from = ((y - ky) * w + (x0 - kx)) as usize;
        let to = ((y - oy) * w + (x0 - ox)) as usize;
        a.copy_within(from..from + n, to);
    };
    if oy > ky {
        (y0..y1).for_each(&mut move_row);
    } else {
        (y0..y1).rev().for_each(&mut move_row);
    }
    // The uncovered strips: the rows above and below the shown part, then
    // its rows' columns to the left and right of it.
    let strips = [
        (ox, oy, w, y0 - oy),
        (ox, y1, w, oy + h - y1),
        (ox, y0, x0 - ox, y1 - y0),
        (x1, y0, ox + w - x1, y1 - y0),
    ];
    for (sx, sy, sw, sh) in strips {
        if sw <= 0 || sh <= 0 {
            continue;
        }
        let part = shroud::light_alpha(
            (sx, sy),
            (sw as u32, sh as u32),
            covers,
            sprites_of(sprites),
            &[],
            q,
        );
        for (r, values) in part.chunks_exact(sw as usize).enumerate() {
            let to = ((sy - oy + r as i32) * w + (sx - ox)) as usize;
            a[to..to + sw as usize].copy_from_slice(values);
        }
    }
    a
}

#[cfg(test)]
mod tests {
    use glasnost_fmt::image::IndexedImage;
    use glasnost_render::shroud::{CoverFrame, FULL_COVER, canvas_origin};
    use glasnost_sim::testkit;
    use glasnost_sim::world::Pos;

    use super::*;

    /// A shroud sprite whose piece 15 is black (index 0) and every other
    /// piece transparent (index 254) over the whole canvas.
    fn sprite() -> CoverSprites {
        let frame = |v: u8| {
            Some(CoverFrame {
                image: IndexedImage::new(60, 30, vec![v; 60 * 30]).unwrap(),
                x: 0,
                y: 0,
            })
        };
        let frames = (0..shroud::PIECES)
            .map(|i| frame(if i == usize::from(FULL_COVER) { 0 } else { 254 }))
            .collect();
        CoverSprites {
            shroud: Some(CoverSprite { frames }),
            fog: None,
        }
    }

    /// A synthetic game in which house 0 has explored one area (S-0409
    /// claim 17) and nothing else.
    fn game() -> World {
        let mut w = testkit::game_with(&testkit::ruleset(), 40, 3);
        let centre = Pos::centre(testkit::mid(40, 0, 0), 0);
        glasnost_sim::econ::shroud::area_reveal(&mut w, HouseId(0), centre, 4);
        w
    }

    /// A view of exactly the canvas of `cell`.
    fn canvas_view(cell: CellXY, q: &ShroudQuirks) -> View {
        View {
            origin: canvas_origin((cell.x, cell.y), 0, q),
            size: (CANVAS_WIDTH as u32, CANVAS_HEIGHT as u32),
            reference: 0,
        }
    }

    /// spec: S-0110 claims 1 and 2 (unexplored: 15; explored: the piece of its unexplored
    /// neighbours); State and ordering (cells by increasing x + y); claim 1 (only cells whose
    /// canvas meets the view)
    #[test]
    fn covers_follow_the_explored_states() {
        let w = game();
        let h = HouseId(0);
        let q = ShroudQuirks::default();
        let cells: Vec<CellXY> = w.cells.grid.iter().map(|(c, _)| c).collect();
        let dark = *cells
            .iter()
            .find(|&&c| !explored(&w, h, c))
            .expect("some cell is not explored");
        let open = *cells
            .iter()
            .find(|&&c| {
                (-1..=1)
                    .all(|dy| (-1..=1).all(|dx| explored(&w, h, CellXY::new(c.x + dx, c.y + dy))))
            })
            .expect("some cell is explored with all its neighbours");
        let c = covers(&w, h, &canvas_view(dark, &q), &q);
        assert!(
            c.windows(2)
                .all(|p| p[0].cell.0 + p[0].cell.1 <= p[1].cell.0 + p[1].cell.1)
        );
        // The canvas and its overlapping neighbours, no more.
        assert!(c.len() <= 9);
        let own = c.iter().find(|k| k.cell == (dark.x, dark.y)).unwrap();
        assert_eq!(own.shroud, FULL_COVER);
        assert_eq!(own.fog, None);
        let c = covers(&w, h, &canvas_view(open, &q), &q);
        let own = c.iter().find(|k| k.cell == (open.x, open.y)).unwrap();
        assert_eq!(own.shroud, 0);
    }

    /// spec: S-0110 claims 5 and 10 (a cell's own pixels set the values of the upper-left quarter
    /// of its canvas, which no lower cell's canvas overlaps); S-0110 Import mapping (no sprite, no
    /// cover)
    #[test]
    fn light_alpha_darkens_unexplored_cells() {
        let w = game();
        let h = HouseId(0);
        let q = ShroudQuirks::default();
        let s = sprite();
        let (mut dark, mut open) = (0, 0);
        for c in w.cells.grid.iter().map(|(c, _)| c).step_by(3) {
            let a = light_alpha(&w, h, &canvas_view(c, &q), &s, &q);
            let want = if !explored(&w, h, c) {
                dark += 1;
                0
            } else if explored(&w, h, CellXY::new(c.x - 1, c.y)) {
                open += 1;
                127
            } else {
                continue;
            };
            assert_eq!(a[(7 * CANVAS_WIDTH + 15) as usize], want, "cell {c:?}");
        }
        assert!(dark > 0 && open > 0);
        let c = w.cells.grid.iter().map(|(c, _)| c).next().unwrap();
        let a = light_alpha(&w, h, &canvas_view(c, &q), &CoverSprites::default(), &q);
        assert!(a.iter().all(|&v| v == 127));
    }

    /// Equivalence test (design: render (#118); passes before and after
    /// the change): with no covers the light-alpha values are 127 at every
    /// pixel of any view, the values a frame starts with, so a caller may
    /// pass none instead.
    /// spec: S-0110 claim 5 (127 before any cover or glow acts)
    #[test]
    fn no_covers_give_the_starting_alpha() {
        let q = ShroudQuirks::default();
        for (origin, size) in [
            ((0, 0), (1, 1)),
            ((-300, 45), (640, 480)),
            ((17, -9), (61, 3)),
        ] {
            let view = View {
                origin,
                size,
                reference: 0,
            };
            let a = alpha_of(&[], &view, &sprite(), &q);
            assert_eq!(a.len(), (size.0 * size.1) as usize);
            assert!(
                a.iter()
                    .all(|&v| i32::from(v) == glasnost_render::light::DEFAULT_LIGHT_ALPHA)
            );
        }
    }

    /// design: render (#118) (moved values are the values made afresh):
    /// views moved by every kind of step from one whose values are kept, the
    /// shroud unchanged, opened inside the part both views show, or opened
    /// outside it.
    /// spec: S-0110 claims 5, 7 and 10 (a pixel's value after the covers whose canvas holds it)
    #[test]
    fn moved_alpha_is_the_alpha_made_afresh() {
        let w = game();
        let h = HouseId(0);
        let q = ShroudQuirks::default();
        let s = sprite();
        let size = (601, 401);
        let mid = testkit::mid(40, 0, 0);
        let (cx, cy) = canvas_origin((mid.x, mid.y), 0, &q);
        let start = (cx - 300, cy - 200);
        let view = |origin: (i32, i32)| View {
            origin,
            size,
            reference: 0,
        };
        let mut opened_inside = w.clone();
        glasnost_sim::econ::shroud::area_reveal(
            &mut opened_inside,
            h,
            Pos::centre(CellXY::new(mid.x + 6, mid.y + 2), 0),
            1,
        );
        assert!(
            !explored(&w, h, CellXY::new(mid.x + 6, mid.y + 2)),
            "the inside opening is new"
        );
        let mut opened_outside = w.clone();
        glasnost_sim::econ::shroud::area_reveal(
            &mut opened_outside,
            h,
            Pos::centre(CellXY::new(mid.x + 30, mid.y + 30), 0),
            2,
        );
        let mut checked = 0;
        for after in [&w, &opened_inside, &opened_outside] {
            for d in [
                (0, 1),
                (1, 0),
                (-1, 0),
                (0, -1),
                (37, -12),
                (-200, 90),
                (600, 0),
                (601, 0),
                (0, 400),
                (-5, -401),
                (700, 700),
            ] {
                let (from, to) = (view(start), view((start.0 + d.0, start.1 + d.1)));
                let kept_covers = covers(&w, h, &from, &q);
                let kept = alpha_of(&kept_covers, &from, &s, &q);
                let now = covers(after, h, &to, &q);
                let fresh = alpha_of(&now, &to, &s, &q);
                let moved = alpha_moved(kept, from.origin, &kept_covers, &now, &to, &s, &q);
                assert!(moved == fresh, "a move by {d:?}");
                checked += usize::from(fresh.contains(&0) && fresh.contains(&127));
            }
        }
        assert!(checked > 20, "{checked} moves show both cover and light");
    }

    /// [`covers`] as before M8 C4 part 4: every cell of the grid asked.
    fn covers_by_scan(world: &World, house: HouseId, view: &View, q: &ShroudQuirks) -> Vec<Cover> {
        let covered = |x: i32, y: i32| !explored(world, house, CellXY::new(x, y));
        let mut out: Vec<Cover> = world
            .cells
            .grid
            .iter()
            .filter_map(|(c, cell)| {
                let level = i32::from(cell.level);
                if !shroud::canvas_in_view((c.x, c.y), level, view.origin, view.size, q) {
                    return None;
                }
                let piece = shroud::cell_piece(covered(c.x, c.y), Covered::of(c.x, c.y, covered));
                Some(Cover {
                    cell: (c.x, c.y),
                    level,
                    shroud: piece,
                    fog: None,
                })
            })
            .collect();
        out.sort_by_key(|c| (c.cell.0 + c.cell.1, c.cell.0));
        out
    }

    /// spec: S-0110 claims 1 and 2, State and ordering, Edge cases (the covers from the band of
    /// candidate cells are the full scan's: views inside, across and outside the map's edges, of
    /// several sizes, over cells of levels −128 to 127, with quirks.shroud_on_flat_grid on and
    /// off)
    #[test]
    fn covers_of_the_view_band_match_the_full_scan() {
        let mut w = game();
        let h = HouseId(0);
        let cells: Vec<CellXY> = w.cells.grid.iter().map(|(c, _)| c).collect();
        for (i, &c) in cells.iter().enumerate() {
            let level = match i % 7 {
                0 => 0,
                1 => 4,
                2 => -3,
                3 => 127,
                4 => -128,
                5 => 12,
                _ => 1,
            };
            w.cells.grid.get_mut(c).unwrap().level = level;
        }
        let (lo, hi) = cells.iter().fold((i32::MAX, i32::MIN), |(lo, hi), c| {
            let (x, _) = glasnost_render::projection::cell_top(c.x, c.y);
            (lo.min(x), hi.max(x))
        });
        let mut nonempty = 0;
        for flat in [true, false] {
            let q = ShroudQuirks {
                shroud_on_flat_grid: flat,
            };
            for size in [(1, 1), (60, 30), (333, 177), (1920, 1080), (4000, 3000)] {
                for ox in (lo - 2000..hi + 2000).step_by(397) {
                    for oy in (-2500..3500).step_by(311) {
                        let view = View {
                            origin: (ox, oy),
                            size,
                            reference: 0,
                        };
                        let fast = covers(&w, h, &view, &q);
                        assert_eq!(
                            fast,
                            covers_by_scan(&w, h, &view, &q),
                            "flat {flat}, view {ox}, {oy}, {size:?}"
                        );
                        nonempty += usize::from(!fast.is_empty());
                    }
                }
            }
        }
        assert!(nonempty > 100, "{nonempty} views have covers");
    }

    /// spec: S-0110 Import mapping (a missing SHROUD.SHP or FOG.SHP draws no cover; the importer
    /// warns)
    #[test]
    fn missing_sprites_warn() {
        let (s, warnings) = CoverSprites::load(&Assets::default());
        assert!(s.shroud.is_none() && s.fog.is_none());
        assert_eq!(warnings.len(), 2);
    }
}
