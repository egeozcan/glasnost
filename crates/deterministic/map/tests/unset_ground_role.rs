//! Ground smoothing with `quirks.unset_role_range` off and a pair whose
//! ground role is unset (#287).

use glasnost_map::terrain::{
    OverlayLand, RecalcRules, Roles, TerrainQuirks, TileCatalogue, TileCell, TileInfo, recalc,
};
use glasnost_map::{Cell, CellGrid, CellXY, Footprint};

fn no_overlay(_: u8) -> Option<OverlayLand> {
    None
}

// spec: S-0010 Edge cases (quirks.unset_role_range off: a transition pair with an unset ground or transition role is skipped)
#[test]
fn unset_ground_role_pair_is_skipped_with_the_switch_off() {
    // 40 flat one-cell tiles; tiles 20 … 35 are the sand transition set, the
    // sand ground role is unset.
    let tiles: Vec<TileInfo> = (0..40)
        .map(|_| TileInfo {
            width: 1,
            height: 1,
            has_image: true,
            cells: vec![Some(TileCell {
                terrain_code: 0,
                slope: 0,
            })],
            variants: 0,
            anim: None,
            no_burrowing: false,
        })
        .collect();
    let roles = Roles {
        sand_transition: 20,
        ..Roles::default()
    };
    let cat = TileCatalogue {
        tiles,
        roles,
        allow_tiberium: Vec::new(),
        bridge_positions: Default::default(),
    };
    let ov = no_overlay;
    let rules = RecalcRules {
        catalogue: &cat,
        overlay: &ov,
        cliff_back: 0,
        quirks: TerrainQuirks {
            unset_role_range: false,
            ..TerrainQuirks::default()
        },
    };
    let mut g = CellGrid::new(Footprint::new(8, 8, [0, 0, 8, 8]));
    let all: Vec<CellXY> = g.iter().map(|(c, _)| c).collect();
    for c in all {
        if let Some(cell) = g.get_mut(c) {
            *cell = Cell {
                tile: 0,
                ..Cell::BLANK
            };
        }
    }
    // A plus of five transition cells around (8, 8).
    for (dx, dy) in [(0, 0), (0, -1), (1, 0), (0, 1), (-1, 0)] {
        let cell = g
            .get_mut(CellXY::new(8 + dx, 8 + dy))
            .expect("the plus lies on the map");
        cell.tile = 25;
    }
    // The centre (all four neighbours belong) and an arm (three out).
    for c in [CellXY::new(8, 8), CellXY::new(9, 8)] {
        recalc(&mut g, c, &rules);
        assert_eq!(g.get(c).map(|x| x.tile), Some(25), "{c:?}");
    }
}
