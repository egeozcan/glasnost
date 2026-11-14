//! Ramp smoothing with `quirks.unset_role_range` off and one ramp role
//! unset (#286).

use glasnost_map::terrain::{
    OverlayLand, RecalcRules, Roles, TerrainQuirks, TileCatalogue, TileCell, TileInfo, recalc,
};
use glasnost_map::{Cell, CellGrid, CellXY, Footprint};

fn catalogue() -> TileCatalogue {
    let one = |slope: u8| TileInfo {
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
    };
    // Tiles 0 … 3 flat, tiles 4 … 7 of slope shapes 1 … 4, tiles 8 … 40 flat.
    let mut tiles = vec![one(0), one(0), one(0), one(0)];
    for s in 1..=4 {
        tiles.push(one(s));
    }
    while tiles.len() < 41 {
        tiles.push(one(0));
    }
    TileCatalogue {
        tiles,
        roles: Roles::default(),
        allow_tiberium: Vec::new(),
        bridge_positions: Default::default(),
    }
}

fn grid() -> CellGrid {
    let mut g = CellGrid::new(Footprint::new(8, 8, [0, 0, 8, 8]));
    let cells: Vec<CellXY> = g.iter().map(|(c, _)| c).collect();
    for c in cells {
        if let Some(cell) = g.get_mut(c) {
            *cell = Cell {
                tile: 0,
                ..Cell::BLANK
            };
        }
    }
    g
}

fn no_overlay(_: u8) -> Option<OverlayLand> {
    None
}

fn off() -> TerrainQuirks {
    TerrainQuirks {
        unset_role_range: false,
        ..TerrainQuirks::default()
    }
}

// spec: S-0010 Edge cases (quirks.unset_role_range off: ramp smoothing is skipped when either ramp role is unset)
#[test]
fn switch_off_ramp_set_smooth_ramp_unset() {
    let mut cat = catalogue();
    cat.roles.ramp = 4; // the smooth-ramp role stays unset (−1)
    let ov = no_overlay;
    let r = RecalcRules {
        catalogue: &cat,
        overlay: &ov,
        cliff_back: 0,
        quirks: off(),
    };
    let mut g = grid();
    let c = CellXY::new(8, 8);
    // East neighbour a slope, west neighbour flat: s = 1, m = 1.
    if let Some(e) = g.get_mut(CellXY::new(9, 8)) {
        e.slope = 1;
    }
    if let Some(cell) = g.get_mut(c) {
        cell.tile = 4;
    }
    recalc(&mut g, c, &r);
    assert_eq!(g.get(c).map(|x| x.tile), Some(4));
}

// spec: S-0010 Edge cases (quirks.unset_role_range off: ramp smoothing is skipped when either ramp role is unset)
#[test]
fn switch_off_ramp_unset_smooth_ramp_set() {
    let mut cat = catalogue();
    cat.roles.smooth_ramp = 4; // the ramp role stays unset (−1); tile 6 has shape 3
    let ov = no_overlay;
    let r = RecalcRules {
        catalogue: &cat,
        overlay: &ov,
        cliff_back: 0,
        quirks: off(),
    };
    let mut g = grid();
    let c = CellXY::new(8, 8);
    // Both tested neighbours (east and west) are slopes: m = 0.
    for x in [7, 9] {
        if let Some(n) = g.get_mut(CellXY::new(x, 8)) {
            n.slope = 1;
        }
    }
    if let Some(cell) = g.get_mut(c) {
        cell.tile = 6;
    }
    recalc(&mut g, c, &r);
    assert_eq!(g.get(c).map(|x| x.tile), Some(6));
}
