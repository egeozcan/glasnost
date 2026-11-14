//! A tile cell's slope code outside 0 … 20 makes a flat cell (#288).

use glasnost_map::terrain::{
    OverlayLand, RecalcRules, Roles, TerrainQuirks, TileCatalogue, TileCell, TileInfo, land,
    recalc,
};
use glasnost_map::{Cell, CellGrid, CellXY, Footprint};

fn one(slope: u8) -> TileInfo {
    TileInfo {
        width: 1,
        height: 1,
        has_image: true,
        cells: vec![Some(TileCell {
            terrain_code: 0,
            slope,
        })],
        variants: 0,
        anim: None,
        no_burrowing: false,
    }
}

// spec: S-0106 Edge cases (a slope code outside 0 … 20: the cell is flat, shape 0), S-0010 claims 6 and 7 (the cell's shape from its tile cell), S-0211 claim 7 (ore stays on shapes 0 … 4)
#[test]
fn slope_code_above_20_is_flat() {
    let cat = TileCatalogue {
        tiles: vec![one(0), one(21), one(255)],
        roles: Roles::default(),
        allow_tiberium: Vec::new(),
        bridge_positions: Default::default(),
    };
    let ov = |_: u8| {
        Some(OverlayLand {
            land: land::CLEAR,
            own_land: false,
            tiberium: true,
        })
    };
    let rules = RecalcRules {
        catalogue: &cat,
        overlay: &ov,
        cliff_back: 0,
        quirks: TerrainQuirks::default(),
    };
    for tile in [1, 2] {
        let mut g = CellGrid::new(Footprint::new(8, 8, [0, 0, 8, 8]));
        let c = CellXY::new(8, 8);
        *g.get_mut(c).expect("cell on the map") = Cell {
            tile,
            overlay: 5,
            ..Cell::BLANK
        };
        let out = recalc(&mut g, c, &rules);
        let cell = g.get(c).copied().expect("cell on the map");
        assert_eq!(
            (cell.slope, cell.overlay, cell.land, out.overlay_removed),
            (0, 5, land::TIBERIUM, false),
            "tile {tile}"
        );
    }
}
