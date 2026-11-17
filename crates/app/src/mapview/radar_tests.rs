//! The radar's kept pixel cells give the simulation's pixels (S-0410 claims
//! 6 to 10; M8 step C4 part 3) on synthetic data: the simulation's
//! synthetic ruleset with an invented sight tower and gap generator. A
//! radar kept over a game follows its changes as a radar made afresh does
//! (render perf batch RP11, #137).

use std::collections::BTreeMap;

use glasnost_map::{CellXY, Theater};
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::BuildingType;
use glasnost_sim::econ::minimap::{self, Arrivals, Bits, Rgb, View};
use glasnost_sim::econ::shroud;
use glasnost_sim::rules::Kind;
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, World};

use super::radar::Radar;

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

/// Each cell's own tile colour, unlike its neighbours'.
fn tiles(w: &World) -> BTreeMap<CellXY, Rgb> {
    w.cells
        .grid
        .iter()
        .map(|(c, _)| {
            let (x, y) = (c.x.rem_euclid(256), c.y.rem_euclid(256));
            let colour = [
                (x * 37 % 256) as u8,
                (y * 53 % 256) as u8,
                ((x + y) * 11 % 256) as u8,
            ];
            (c, colour)
        })
        .collect()
}

fn radar_of(w: &World) -> Radar {
    let colours = vec![[200, 40, 40], [40, 200, 40], [40, 40, 200], [200, 200, 40]];
    Radar::new(w, Theater::Temperate, &tiles(w), colours, [250, 250, 250]).expect("usable cells")
}

/// The picture a radar made afresh for the world shows `h`.
fn fresh(w: &World, h: HouseId) -> Vec<u8> {
    let mut r = radar_of(w);
    r.observe(w, Some(h));
    r.rgba(w, Some(h))
}

/// Every house has radar (S-0410 claim 1), so that its picture is drawn.
fn with_radar(w: &mut World) {
    for h in &mut w.houses {
        h.econ.radar.available = true;
    }
}

/// Radars kept over the whole game: one for each house, and one asked for
/// both houses in turn.
struct KeptRadars {
    by_house: [Radar; 2],
    both: Radar,
}

/// Asks every kept radar for its picture and checks it against a radar made
/// afresh; gives the pictures of houses 0 and 1.
fn follows(w: &mut World, kept: &mut KeptRadars, label: &str) -> [Vec<u8>; 2] {
    with_radar(w);
    let w = &*w;
    assert_eq!(
        radar_of(w).image_rect(),
        kept.both.image_rect(),
        "{label}: the radar image stays"
    );
    [0u8, 1].map(|h| {
        let house = HouseId(h);
        let want = fresh(w, house);
        let r = &mut kept.by_house[usize::from(h)];
        r.observe(w, Some(house));
        assert_eq!(r.rgba(w, Some(house)), want, "{label}, house {h}");
        kept.both.observe(w, Some(house));
        assert_eq!(
            kept.both.rgba(w, Some(house)),
            want,
            "{label}, house {h}, the radar asked for both houses"
        );
        want
    })
}

// spec: S-0410 claims 6, 7 and 8 (render perf batch RP11, #137: a kept radar shows what a radar made afresh shows after one cell's shroud changes either way, after one object changes its house, after a friendly gap field starts and turns hostile, and after one cell's height changes which cell its point test reads, quirks.radar_pixel_cell_shift on and off)
#[test]
fn a_kept_radar_follows_shroud_houses_gaps_and_heights() {
    for shift in [true, false] {
        let mut w = testkit::game_with(&rules(shift), SIZE, 3);
        spawn(&mut w, Kind::Building, "TOWER", 0, 0, 0);
        spawn(&mut w, Kind::Building, "TOWER", 1, 9, 2);
        let cart = spawn(&mut w, Kind::Vehicle, "CART", 0, -3, 0);
        spawn(&mut w, Kind::Vehicle, "CART", 1, 0, 3);
        testkit::run(&mut w, 3);
        with_radar(&mut w);
        let mut kept = KeptRadars {
            by_house: [radar_of(&w), radar_of(&w)],
            both: radar_of(&w),
        };
        let start = follows(&mut w, &mut kept, &format!("start, shift {shift}"));
        assert!(
            follows(&mut w, &mut kept, &format!("again, shift {shift}")) == start,
            "nothing changed, nothing moves"
        );

        // One cell's shroud: an explored cell is covered again.
        let me = HouseId(0);
        let covered = testkit::mid(SIZE, 2, -2);
        assert!(shroud::explored(&w, me, covered));
        let i = w.cells.idx(covered).expect("a legal cell");
        let marks = &mut w.econ.shroud.houses[0].as_mut().expect("a player").marks;
        let explored_marks = marks[w.cells.idx(testkit::mid(SIZE, 0, 1)).unwrap()];
        marks[i] = 0;
        assert!(!shroud::explored(&w, me, covered));
        let once = follows(&mut w, &mut kept, &format!("a cell covered, shift {shift}"));
        assert!(once[0] != start[0], "the covered cell shows black");

        // One cell's shroud: a far cell is explored.
        let far = testkit::mid(SIZE, -12, 12);
        assert!(!shroud::explored(&w, me, far));
        let j = w.cells.idx(far).expect("a legal cell");
        w.econ.shroud.houses[0].as_mut().unwrap().marks[j] = explored_marks;
        assert!(shroud::explored(&w, me, far));
        let twice = follows(
            &mut w,
            &mut kept,
            &format!("a cell explored, shift {shift}"),
        );
        assert!(twice[0] != once[0], "the explored cell shows its terrain");

        // One cell's houses: house 0's cart goes over to house 1.
        w.obj_mut(cart).expect("the cart").owner = Some(HouseId(1));
        let owner = follows(
            &mut w,
            &mut kept,
            &format!("an owner changed, shift {shift}"),
        );
        assert!(owner[0] != twice[0], "the cart shows house 1's colour");

        // A friendly gap field starts, then turns hostile.
        let gap = spawn(&mut w, Kind::Building, "GAP", 0, 2, 2);
        testkit::run(&mut w, 3);
        let field = testkit::mid(SIZE, 2, 1);
        assert!(minimap::cell_in_friendly_gap(&w, me, field));
        let started = follows(&mut w, &mut kept, &format!("a gap field, shift {shift}"));
        let o = w.obj_mut(gap).expect("the generator");
        o.econ.sight.gap.as_mut().expect("a running field").owner = Some(HouseId(1));
        assert!(!minimap::cell_in_friendly_gap(&w, me, field));
        let hostile = follows(
            &mut w,
            &mut kept,
            &format!("a hostile field, shift {shift}"),
        );
        assert!(
            hostile[0] != started[0],
            "the field's cells are no longer halved"
        );

        // One cell's height: raised two levels, its point test reads the
        // cell up the slope of the grid, whose shroud differs.
        let raised = (-7..=7)
            .flat_map(|dx| (-7..=7).map(move |dy| testkit::mid(SIZE, dx, dy)))
            .find(|&c| {
                let before = minimap::cell_shrouded(&w, me, c);
                let Some(cell) = w.cells.grid.get_mut(c) else {
                    return false;
                };
                cell.level += 2;
                if w.cells.usable(c) && minimap::cell_shrouded(&w, me, c) != before {
                    return true;
                }
                w.cells.grid.get_mut(c).unwrap().level -= 2;
                false
            });
        assert!(raised.is_some(), "a cell whose height changes its shroud");
        let height = follows(&mut w, &mut kept, &format!("a cell raised, shift {shift}"));
        assert!(height[0] != hostile[0], "the raised cell's pixel follows");
    }
}
