//! The radar's kept pixel cells give the simulation's pixels (S-0410 claims
//! 6 to 10; M8 step C4 part 3) on synthetic data: the simulation's
//! synthetic ruleset with an invented sight tower and gap generator.

use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::BuildingType;
use glasnost_sim::econ::minimap::{self, Arrivals, Bits, View};
use glasnost_sim::rules::Kind;
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, World};

const SIZE: i32 = 40;

/// TOWER (sight 4) and GAP (gap generator, radius 3), both 1 × 1.
fn rules(shift: bool) -> Ruleset {
    let mut r = testkit::ruleset();
    r.registries.insert(
        "buildings".into(),
        ["TOWER", "GAP"].iter().map(|s| s.to_string()).collect(),
    );
    let b = |sight: i32| {
        let mut b = BuildingType {
            strength: Some(500),
            foundation: Some([1, 1]),
            ..Default::default()
        };
        b.econ.sight = Some(sight);
        b
    };
    r.sim.buildings.insert("TOWER".into(), b(4));
    let mut gap = b(0);
    gap.gap_generator = Some(true);
    gap.econ.gap_radius_in_cells = Some(3);
    r.sim.buildings.insert("GAP".into(), gap);
    r.globals.quirks.radar_pixel_cell_shift = shift;
    r
}

fn spawn(w: &mut World, kind: Kind, name: &str, h: u8, dx: i32, dy: i32) -> ObjectId {
    testkit::spawn(w, kind, name, h, testkit::mid(SIZE, dx, dy), 0)
}

/// Both composes for every house and both `quirks.shroud_on_flat_grid`
/// settings, with a flash on every object so the inversion is reached.
fn assert_same(w: &World, label: &str) {
    let g = minimap::grid(w).expect("usable cells");
    let img = minimap::image(&g);
    let cells = minimap::pixel_cells(&g, &img, w.rules().quirks.radar_pixel_cell_shift);
    let terrain: Vec<_> = (0..img.width * img.height)
        .map(|i| [(i % 251) as u8, (i % 113) as u8 + 40, (i % 61) as u8 + 90])
        .collect();
    for h in 0..2u8 {
        let house = HouseId(h);
        let mut arr = Arrivals::default();
        arr.observe(w, house, &g, &img);
        let colour = |x: HouseId| [40 + 70 * x.0, 200 - 50 * x.0, 30];
        let flash = |id: ObjectId| (id.0 % 9) as i32;
        for on_flat_grid in [true, false] {
            let view = View {
                house,
                house_colour: &colour,
                neutral: [250, 250, 250],
                flash: &flash,
                flash_frame_time: 2,
                bits: Bits([5, 6, 5]),
                on_flat_grid,
            };
            let sim = minimap::compose(w, &view, &arr, &g, &img, &terrain);
            let kept = minimap::compose_kept(w, &view, &arr, &g, &img, &terrain, &cells);
            assert_eq!(sim, kept, "{label}, house {h}, flat grid {on_flat_grid}");
            assert!(
                sim.contains(&[0, 0, 0]),
                "{label}: some shrouded pixels are black"
            );
        }
    }
}

// spec: S-0410 claims 6, 7, 8 and 10 (the kept cells and the listed-pixel winner give the same radar colours as the simulation's compose: objects, flashes, shroud and terrain, with and without gap fields, quirks.radar_pixel_cell_shift on and off)
#[test]
fn radar_compose_matches_the_sim() {
    for shift in [true, false] {
        let mut w = testkit::game_with(&rules(shift), SIZE, 3);
        spawn(&mut w, Kind::Building, "TOWER", 0, 0, 0);
        spawn(&mut w, Kind::Building, "TOWER", 1, 9, 2);
        spawn(&mut w, Kind::Vehicle, "CART", 0, 1, 1);
        spawn(&mut w, Kind::Vehicle, "CART", 1, 2, 0);
        spawn(&mut w, Kind::Vehicle, "CART", 1, 8, 3);
        testkit::run(&mut w, 3);
        assert!(!w.objects.values().any(|o| o.econ.sight.gap.is_some()));
        assert_same(&w, &format!("no gap field, shift {shift}"));
        // A field of house 1 next to house 0's tower (hostile to 0,
        // friendly to 1) and one of house 0 by house 1's tower.
        spawn(&mut w, Kind::Building, "GAP", 1, 3, 0);
        spawn(&mut w, Kind::Building, "GAP", 0, 8, 1);
        testkit::run(&mut w, 3);
        assert!(w.objects.values().any(|o| {
            o.econ
                .sight
                .gap
                .as_ref()
                .is_some_and(|g| !g.cells.is_empty())
        }));
        // The gap branch is reached: a field's own cell lies in a friendly field.
        let at = testkit::mid(SIZE, 3, 0);
        assert!(minimap::cell_in_friendly_gap(&w, HouseId(1), at));
        assert!(minimap::cell_in_friendly_gap(
            &w,
            HouseId(0),
            testkit::mid(SIZE, 8, 1)
        ));
        assert_same(&w, &format!("gap fields, shift {shift}"));
    }
}
