//! Which cells are shadowed (S-0104 claim 19): the marks of intact high
//! bridges and of the tiles of shadow-casting tilesets. A mobile object in a
//! shadowed cell takes claim 13's penalty.

use glasnost_convert::theater::Catalogue;
use glasnost_map::CellXY;

/// The terrain a mark reads in a cell: its tile number as stored and its
/// overlay index.
pub type CellTerrain = (i32, u8);

/// The position of the tile stored as `stored` in its tileset when that
/// tileset casts shadows (S-0104 claim 19, S-0010 claim 18), else `None`.
pub fn casting_position(_catalogue: &Catalogue, _stored: i32) -> Option<i32> {
    None
}

/// Whether cell `c` is shadowed (S-0104 claim 19), with `cell` the terrain
/// of a cell of the map (`None` outside it) and `position` the
/// shadow-casting position of a stored tile number ([`casting_position`]).
pub fn cell_shadowed(
    _c: CellXY,
    _cell: &dyn Fn(CellXY) -> Option<CellTerrain>,
    _position: &dyn Fn(i32) -> Option<i32>,
) -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use glasnost_map::NO_OVERLAY;
    use std::collections::{BTreeMap, BTreeSet};

    /// A tile number of ours standing for a shadow-casting tile at position
    /// `p`; plain ground is tile 0, which casts none.
    const CASTING: i32 = 1000;

    fn position(t: i32) -> Option<i32> {
        (t >= CASTING).then_some(t - CASTING)
    }

    /// A 20 × 20 map of plain ground with `placed` terrain, and the cells
    /// it shadows.
    fn shadowed(placed: &[(CellXY, CellTerrain)]) -> BTreeSet<(i32, i32)> {
        let mut grid: BTreeMap<CellXY, CellTerrain> = (0..20)
            .flat_map(|x| (0..20).map(move |y| (CellXY::new(x, y), (0, NO_OVERLAY))))
            .collect();
        grid.extend(placed.iter().copied());
        let cell = |c: CellXY| grid.get(&c).copied();
        grid.keys()
            .filter(|&&c| cell_shadowed(c, &cell, &position))
            .map(|c| (c.x, c.y))
            .collect()
    }

    fn set(cells: &[(i32, i32)]) -> BTreeSet<(i32, i32)> {
        cells.iter().copied().collect()
    }

    /// spec: S-0104 claim 19 (Bridge marks: an overlay of marking direction map north, S-0211
    /// claim 15's indices 24 and 237, marks (x, y + 1), (x, y), (x, y − 1) and (x, y − 2); one of
    /// map west, 25 and 238, also (x + 2, y): (x + 2, y) … (x − 2, y))
    #[test]
    fn claim_19_bridge_marks() {
        let at = CellXY::new(8, 8);
        for overlay in [24, 237] {
            assert_eq!(
                shadowed(&[(at, (0, overlay))]),
                set(&[(8, 9), (8, 8), (8, 7), (8, 6)]),
                "{overlay}"
            );
        }
        for overlay in [25, 238] {
            assert_eq!(
                shadowed(&[(at, (0, overlay))]),
                set(&[(10, 8), (9, 8), (8, 8), (7, 8), (6, 8)]),
                "{overlay}"
            );
        }
        // A destroyed bridge (its overlay gone) and any other overlay, a low bridge's too, mark
        // nothing.
        for overlay in [NO_OVERLAY, 0, 23, 26, 236, 239] {
            assert!(shadowed(&[(at, (0, overlay))]).is_empty(), "{overlay}");
        }
        // Marks that would fall outside the map mark nothing; the rest still do.
        assert_eq!(
            shadowed(&[(CellXY::new(0, 1), (0, 24))]),
            set(&[(0, 2), (0, 1), (0, 0)])
        );
        assert_eq!(
            shadowed(&[(CellXY::new(18, 3), (0, 25))]),
            set(&[(19, 3), (18, 3), (17, 3), (16, 3)])
        );
    }

    /// spec: S-0104 claim 19 (Tile marks: a cell whose tile belongs to a shadow-casting tileset
    /// marks (x + dx, y + dy) for table T's displacements at the tile's position; positions it
    /// does not list, 40 and above included, mark nothing), checked at every position from 0 to 41
    #[test]
    fn claim_19_tile_marks_follow_table_t() {
        let table: BTreeMap<i32, &[(i32, i32)]> = [
            (20, &[(0, -1), (-1, -2)][..]),
            (21, &[(0, -1), (-1, -2)]),
            (22, &[(-1, -3), (0, -3)]),
            (23, &[(-1, -3), (0, -3)]),
            (24, &[(-1, -3), (0, -3)]),
            (25, &[(0, -3)]),
            (26, &[(0, -3), (1, -3), (-1, -3), (-2, -3)]),
            (27, &[(0, -3), (-1, -3), (-2, -3), (-3, -2)]),
            (28, &[(0, -3), (-1, -3), (-2, -3)]),
            (29, &[(0, -3), (-1, -3), (-2, -3)]),
            (30, &[(-2, -3)]),
            (31, &[(-2, -3)]),
            (32, &[(0, -3)]),
        ]
        .into();
        let (x, y) = (9, 10);
        for p in 0..=41 {
            let expected: BTreeSet<(i32, i32)> = table
                .get(&p)
                .copied()
                .unwrap_or(&[])
                .iter()
                .map(|&(dx, dy)| (x + dx, y + dy))
                .collect();
            assert_eq!(
                shadowed(&[(CellXY::new(x, y), (CASTING + p, NO_OVERLAY))]),
                expected,
                "position {p}"
            );
        }
        // A tileset that casts no shadow marks nothing, whatever the position.
        assert!(shadowed(&[(CellXY::new(x, y), (25, NO_OVERLAY))]).is_empty());
        // Marks outside the map mark nothing.
        assert!(shadowed(&[(CellXY::new(4, 1), (CASTING + 25, NO_OVERLAY))]).is_empty());
        // A tile covering several cells marks from each; marks add up.
        assert_eq!(
            shadowed(&[
                (CellXY::new(5, 10), (CASTING + 25, NO_OVERLAY)),
                (CellXY::new(6, 10), (CASTING + 25, NO_OVERLAY)),
                (CellXY::new(6, 9), (CASTING + 30, NO_OVERLAY)),
            ]),
            set(&[(5, 7), (6, 7), (4, 6)])
        );
    }

    /// spec: S-0104 claim 19 (a cell is shadowed while at least one mark covers it: a bridge
    /// destroyed over a cell a tile also marks leaves the tile's mark; our engine's rule in place
    /// of the original's single state)
    #[test]
    fn claim_19_one_mark_is_enough() {
        let tile = (CellXY::new(8, 11), (CASTING + 25, NO_OVERLAY));
        let bridge = (CellXY::new(8, 8), (0, 24));
        assert!(shadowed(&[tile, bridge]).contains(&(8, 8)));
        let destroyed = (CellXY::new(8, 8), (0, NO_OVERLAY));
        assert_eq!(shadowed(&[tile, destroyed]), set(&[(8, 8)]));
    }

    /// spec: S-0104 claim 19 and S-0010 claim 18 (the position is the tile number minus its
    /// tileset's first tile, for a tileset with the shadow-caster flag and a shadow-tile count
    /// other than 0); S-0009 claim 8 (the marker shows the clear tile)
    #[test]
    fn claim_19_positions_come_from_shadow_casting_tilesets() {
        use glasnost_ini::{Document, Quirks};
        let text = "[General]\nClearTile=0\n[TileSet0000]\nTilesInSet=2\n\
                    [TileSet0001]\nTilesInSet=40\nShadowCaster=yes\nShadowTiles=4\n\
                    [TileSet0002]\nTilesInSet=30\nShadowCaster=yes\n";
        let doc = Document::parse(text.as_bytes(), Quirks::default(), &mut Vec::new());
        let c = Catalogue::read(&doc, glasnost_vfs::theater::Theater::Temperate);
        assert_eq!(casting_position(&c, 0), None, "a tileset that casts none");
        assert_eq!(casting_position(&c, 2), Some(0));
        assert_eq!(casting_position(&c, 2 + 25), Some(25));
        assert_eq!(casting_position(&c, 41), Some(39));
        assert_eq!(casting_position(&c, 42 + 25), None, "a count of 0");
        assert_eq!(casting_position(&c, glasnost_map::NO_TILE), None, "the clear tile");
    }
}
