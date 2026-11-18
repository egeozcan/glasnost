//! S-0009 r47 claim 29, scenario M13b variant F2: a late-phase
//! re-assignment beside marks that name no house. Synthetic maps only; the
//! start waypoints are those of `claims_s0009_r45_header_marks.rs`.

mod common;

use common::{Scene, TRACK, unit_type};
use glasnost_map::Header;
use glasnost_sim::mp_triggers::resolve_house;
use glasnost_sim::world::{HouseId, World};

/// "Player at position A"; B … H follow it.
const POSITION_A: i32 = 4475; // spec: S-0011 claim 23 (4475 … 4482: the player at position A … H)

/// Start waypoint markers 0 … 3 whose pairwise distances all differ
/// (scenario M13b): 0–1 is 10, 0–2 30, 0–3 21, 1–2 20, 1–3 16 and 2–3 22
/// cells (truncated), so W*, the one of markers 2 and 3 with the larger
/// distance sum to markers 0 and 1, is marker 2 (50 against 37).
const WAYPOINTS: [(i16, i16); 4] = [(30, 40), (40, 40), (60, 40), (44, 56)];

fn load(s: &Scene) -> World {
    glasnost_sim::load::new_game(
        s.sim_rules(),
        &s.map,
        std::sync::Arc::new(s.catalogue.clone()),
        &s.setup,
    )
    .expect("loads")
}

/// spec: S-0009 claim 29 (late phase: a non-passive house that holds no position is assigned anew by claim 19 against list 3, every position whose mark is not −1 counting as taken; scenario M13b variant F2)
#[test]
fn variant_f2_players_whose_positions_a_pair_overwrote_are_assigned_beside_them() {
    for seed in 1..=5 {
        let mut s = Scene::new();
        s.vehicle("TANK", unit_type(TRACK, 4, 5));
        s.setup.sandbox = false;
        s.setup.seed = seed;
        for (k, &c) in WAYPOINTS.iter().enumerate() {
            s.map.waypoints.insert(k as u16, c);
        }
        // N_h = 9, pair 9 = (5, 5); the first player on chosen 0, the second
        // on chosen 1.
        s.map.header = Header {
            count: 9,
            origin: (0, 0),
            extent: (10, 10),
            pairs: [
                vec![Some([Some(0), Some(0)]); 8],
                vec![Some([Some(5), Some(5)])],
            ]
            .concat(),
        };
        s.setup.houses[0].start = Some(0);
        s.setup.houses[1].start = Some(1);
        let w = load(&s);
        // The late phase gives 0 and 1 the mark 5 again: both players hold
        // none, and are assigned anew with 0 and 1 taken. The first takes
        // W*, by the farthest-sum rule; the second the other one.
        let starts: Vec<_> = w.houses.iter().map(|h| h.start).collect();
        assert_eq!(starts, [Some(2), Some(3)], "seed {seed}");
        // Positions 0 and 1 name no house (variant F).
        assert_eq!(resolve_house(&w, POSITION_A, None), None);
        assert_eq!(resolve_house(&w, POSITION_A + 1, None), None);
        assert_eq!(resolve_house(&w, POSITION_A + 2, None), Some(HouseId(0)));
        assert_eq!(resolve_house(&w, POSITION_A + 3, None), Some(HouseId(1)));
    }
}
