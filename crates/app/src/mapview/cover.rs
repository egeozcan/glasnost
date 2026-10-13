//! Shroud covers of the live game (S-0110): the cover sprites read at load
//! and the light-alpha values of a view, from the simulation's explored
//! states for the house the user controls (S-0409).

use glasnost_fmt::shp::Shp;
use glasnost_map::CellXY;
use glasnost_render::scene::View;
use glasnost_render::shroud::{self, Cover, CoverSprite, Covered, ShroudQuirks, Sprites};
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
pub fn covers(world: &World, house: HouseId, view: &View, q: &ShroudQuirks) -> Vec<Cover> {
    let covered = |x: i32, y: i32| !explored(world, house, CellXY::new(x, y));
    let mut out: Vec<Cover> = world
        .cells
        .grid
        .iter()
        .filter_map(|(c, cell)| {
            let level = i32::from(cell.level);
            // spec: S-0110 claim 1 (every legal cell whose canvas meets the battlefield view)
            if !shroud::canvas_in_view((c.x, c.y), level, view.origin, view.size, q) {
                return None;
            }
            // spec: S-0110 claims 1 and 2 (the explored states of the cell and its map neighbours;
            // a non-legal neighbour reads the shared state through S-0409 claim 14)
            let piece = shroud::cell_piece(covered(c.x, c.y), Covered::of(c.x, c.y, covered));
            Some(Cover {
                cell: (c.x, c.y),
                level,
                shroud: piece,
                // spec: S-0110 claim 4, Import mapping (fog of war is always off in skirmish: no fog piece)
                fog: None,
            })
        })
        .collect();
    // spec: S-0110 State and ordering (cells by increasing x + y)
    out.sort_by_key(|c| (c.cell.0 + c.cell.1, c.cell.0));
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
    let covers = covers(world, house, view, q);
    shroud::light_alpha(
        view.origin,
        view.size,
        &covers,
        Sprites {
            shroud: sprites.shroud.as_ref(),
            fog: sprites.fog.as_ref(),
            // spec: S-0110 Import mapping (fog of war always off in skirmish)
            fog_of_war: false,
        },
        &[],
        q,
    )
}

#[cfg(test)]
mod tests {
    use glasnost_fmt::image::IndexedImage;
    use glasnost_render::shroud::{
        CANVAS_HEIGHT, CANVAS_WIDTH, CoverFrame, FULL_COVER, canvas_origin,
    };
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

    /// spec: S-0110 Import mapping (a missing SHROUD.SHP or FOG.SHP draws no cover; the importer
    /// warns)
    #[test]
    fn missing_sprites_warn() {
        let (s, warnings) = CoverSprites::load(&Assets::default());
        assert!(s.shroud.is_none() && s.fog.is_none());
        assert_eq!(warnings.len(), 2);
    }
}
