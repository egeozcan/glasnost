//! Claim tests for S-0410 (radar availability, and the pure functions of
//! the radar map) on synthetic data: invented types, no stock values.

mod common;

use glasnost_map::CellXY;
use glasnost_num::Rounding;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::BuildingType;
use glasnost_sim::econ::minimap::{self, Bits, Grid, PointSource};
use glasnost_sim::econ::{power, radar, shroud};
use glasnost_sim::orders::Order;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, World};

const SIZE: i32 = 24;
const A: HouseId = HouseId(0);

fn bt(power: i32, radar: bool) -> BuildingType {
    let mut b = BuildingType {
        strength: Some(500),
        foundation: Some([1, 1]),
        ..Default::default()
    };
    b.econ.power = Some(power);
    b.econ.radar = Some(radar);
    b
}

fn rules() -> Ruleset {
    let mut r = testkit::ruleset();
    r.registries.insert(
        "buildings".into(),
        ["PLANT", "DISH", "CONS", "UPDISH"]
            .iter()
            .map(|s| s.to_string())
            .collect(),
    );
    r.sim.buildings.insert("PLANT".into(), bt(100, false));
    r.sim.buildings.insert("DISH".into(), bt(-20, true));
    r.sim.buildings.insert("CONS".into(), bt(-200, false));
    let mut up = bt(0, true);
    up.econ.powers_up_building = Some("PLANT".into());
    r.sim.buildings.insert("UPDISH".into(), up);
    r
}

fn game(r: &Ruleset) -> World {
    let mut w = testkit::game_with(r, SIZE, 1);
    testkit::run(&mut w, 4);
    w
}

fn put(w: &mut World, n: &str, dx: i32) -> ObjectId {
    testkit::spawn(w, Kind::Building, n, 0, testkit::mid(SIZE, dx, 0), 0)
}

fn step(w: &mut World) {
    glasnost_sim::tick::step(w);
}

// spec: S-0410 claim 1 (radar with a radar building and full power; none on low power, none without a qualifying building)
#[test]
fn claim_1_availability() {
    let mut w = game(&rules());
    assert!(!radar::available(&w, A));
    put(&mut w, "PLANT", 0);
    let d = put(&mut w, "DISH", 2);
    step(&mut w);
    assert!(radar::available(&w, A));
    // A radar building in build-up counts (the switch is all that is tested).
    let t = w.tick;
    w.obj_mut(d)
        .unwrap()
        .beh
        .force(Behaviour::Construction, t, true);
    radar::mark_due(&mut w, A);
    step(&mut w);
    assert!(radar::available(&w, A));
    // Low power: no radar.
    put(&mut w, "CONS", 4);
    step(&mut w);
    assert!(power::low_power(&w, A));
    assert!(!radar::available(&w, A));
}

// spec: S-0410 claim 1 (a radar building being sold, its queued behaviour selling, does not qualify)
#[test]
fn claim_1_selling_does_not_qualify() {
    let mut w = game(&rules());
    put(&mut w, "PLANT", 0);
    let d = put(&mut w, "DISH", 2);
    step(&mut w);
    assert!(radar::available(&w, A));
    w.obj_mut(d).unwrap().beh.queued = Some(Behaviour::Selling);
    step(&mut w);
    step(&mut w);
    assert!(!radar::available(&w, A));
}

// spec: S-0410 claim 1 (the free-radar flag gives radar without a building or power; an outage takes it away)
#[test]
fn claim_1_free_radar_and_outage() {
    let r = rules();
    let mut map = testkit::empty_map(SIZE);
    map.basic.free_radar = true;
    let rules = std::sync::Arc::new(glasnost_sim::rules::SimRules::new(&r, "t"));
    let mut w = glasnost_sim::load::new_game(rules, &map, testkit::catalogue(), &testkit::setup(1))
        .unwrap();
    step(&mut w);
    assert!(radar::available(&w, A));
    radar::start_outage(&mut w, A, 10);
    step(&mut w);
    assert!(!radar::available(&w, A));
    for _ in 0..12 {
        step(&mut w);
    }
    assert!(radar::available(&w, A));
}

// spec: S-0410 claim 1 (after defeat radar is on for good, without a recheck)
#[test]
fn claim_1_defeat() {
    let mut w = game(&rules());
    radar::on_defeat(&mut w, A);
    assert!(radar::available(&w, A));
    assert!(shroud::whole_map_revealed(&w, A));
    put(&mut w, "CONS", 0);
    step(&mut w);
    assert!(radar::available(&w, A));
}

// spec: S-0410 claim 2 (rechecked in the house's next update after an event: an accepted power-switch change in tick n changes the panel in tick n + 1)
#[test]
fn claim_2_recheck_timing() {
    let mut w = game(&rules());
    put(&mut w, "PLANT", 0);
    let d = put(&mut w, "DISH", 2);
    step(&mut w);
    assert!(radar::available(&w, A));
    assert!(glasnost_sim::orders::issue(
        &mut w,
        A,
        Order::PowerOff { actor: d }
    ));
    step(&mut w);
    assert!(radar::available(&w, A));
    step(&mut w);
    assert!(!radar::available(&w, A));
    assert!(w.houses[0].econ.radar.changed.is_some());
}

// spec: S-0410 Edge cases (quirks.radar_first_building_decides off: any qualifying building that is not locked or erased decides; locks are a seam, so both settings agree here)
#[test]
fn claim_1_first_building_switch() {
    for q in [true, false] {
        let mut r = rules();
        r.globals.quirks.radar_first_building_decides = q;
        let mut w = game(&r);
        put(&mut w, "PLANT", 0);
        put(&mut w, "DISH", 2);
        put(&mut w, "DISH", 4);
        step(&mut w);
        assert!(radar::claim_1(&w, A));
    }
}

fn grid_of(w: &World) -> Grid {
    minimap::grid(w).unwrap()
}

// spec: S-0410 claim 4 and F1 (two radar points per cell, clipped at the columns −1 and 2·Lw − 1)
#[test]
fn claim_4_radar_points() {
    assert_eq!(
        minimap::cell_points(5, 10, 8, CellXY::new(6, 6)),
        vec![(5, 2), (6, 2)]
    );
    assert_eq!(
        minimap::cell_points(-1, 0, 8, CellXY::new(3, 3)),
        vec![(0, 6)]
    );
    assert_eq!(
        minimap::cell_points(15, 0, 8, CellXY::new(3, 3)),
        vec![(15, 6)]
    );
    let w = game(&rules());
    let g = grid_of(&w);
    assert!(g.w > 0 && g.n > 0);
}

// spec: S-0410 claim 5 and F2 (the scale 140 / W, or 108 / N when N × 140 / W ≥ 108; the image centred)
#[test]
fn claim_5_scale_and_size() {
    let wide = Grid {
        a: 0,
        b: 0,
        w: 70,
        n: 20,
        lw: 35,
    };
    let img = minimap::image(&wide);
    assert_eq!((img.width, img.height), (140, 40));
    assert_eq!((img.left, img.top), (0, 34));
    let tall = Grid {
        a: 0,
        b: 0,
        w: 70,
        n: 108,
        lw: 35,
    };
    let img = minimap::image(&tall);
    assert_eq!((img.width, img.height), (70, 108));
    assert_eq!((img.left, img.top), (35, 0));
}

// spec: S-0410 claim 6 (the point test on the cell's centre at the cell's surface height there, S-0106: a slope whose centre lies a level up uses two candidate cells)
#[test]
fn claim_6_surface_height_at_the_centre() {
    let mut s = common::Scene::new();
    let cells: Vec<CellXY> = (1..=20).map(|k| common::cell(3 * k - 30, 0)).collect();
    for (k, &c) in (1u8..).zip(&cells) {
        s.set_cell(c, common::slope_tile(k), 0);
    }
    let mut w = s.build();
    let mut differs = 0;
    for &c in &cells {
        // Explore the diagonal neighbour up and left of c, not c.
        let d = CellXY::new(c.x - 1, c.y - 1);
        let p = glasnost_sim::world::Pos::centre(CellXY::new(c.x - 2, c.y - 2), 0);
        shroud::area_reveal(&mut w, A, p, 1);
        let centre = glasnost_sim::world::Pos::centre(c, 0);
        let z = w.surface_at(centre.x, centre.y);
        let at_surface =
            shroud::point_shrouded(&w, A, glasnost_sim::world::Pos::new(centre.x, centre.y, z));
        assert_eq!(minimap::cell_shrouded(&w, A, c), at_surface, "{c:?}");
        let floor = w
            .cells
            .cell(c)
            .unwrap()
            .floor_height(w.rules().quirks.negative_level_floor_raised);
        let at_floor = shroud::point_shrouded(
            &w,
            A,
            glasnost_sim::world::Pos::new(centre.x, centre.y, floor),
        );
        assert!(shroud::explored(&w, A, d) && !shroud::explored(&w, A, c));
        if at_floor != at_surface {
            differs += 1;
        }
    }
    assert!(differs > 0, "no slope's centre is a level up");
}

// spec: S-0410 claim 6 (r15, Q-0480: both radar tests use S-0110 claim 8's candidates at the centre height, heights
// below 0 included, z = trunc(Z / 104); odd z < 0 reads (x − k, y − k) and (x − k + 1, y − k + 1), k = (z + 3)/2;
// quirks.shroud_negative_odd_pair off: k = (z + 1)/2; quirks.shroud_on_flat_grid off: the cell itself takes precedence),
// scenario R4 part G's centre heights −103, −207 and −311
#[test]
fn claim_6_negative_centre_heights() {
    use glasnost_sim::world::Pos;
    let c = CellXY::new(10, 10);
    let at = |j: i32| CellXY::new(c.x + j, c.y + j);
    let p = |z: i32| Pos::new(256 * c.x + 128, 256 * c.y + 128, z);
    // −103: z = 0, the cell itself (truncation toward zero, not floor; no clamp to height 0).
    assert_eq!(
        shroud::point_candidates_on(p(-103), true, true),
        (at(0), None)
    );
    // −207: z = −1, k = 1: c − 1 and c. With only c + 1 explored the pixel is black, and the gap
    // test needs both c − 1 and c covered.
    assert_eq!(
        shroud::point_candidates_on(p(-207), true, true),
        (at(-1), Some(at(0)))
    );
    // −311: z = −2, the one cell c + 1.
    assert_eq!(
        shroud::point_candidates_on(p(-311), true, true),
        (at(1), None)
    );
    // The pair switch off: k = 0, c and c + 1 (the both-covered replay shows ordinary terrain
    // when c + 1 is explored but not covered).
    assert_eq!(
        shroud::point_candidates_on(p(-207), false, true),
        (at(0), Some(at(1)))
    );
    // The flat-grid switch off: the cell itself, whatever the height or the pair switch.
    for z in [-311, -207, -103, 0, 104, 207] {
        for pair in [true, false] {
            assert_eq!(
                shroud::point_candidates_on(p(z), pair, false),
                (at(0), None)
            );
        }
    }
    // The radar's tests on flat ground at levels −1, −2 and −3, against the explored cells.
    let mut s = common::Scene::new();
    let cells: Vec<CellXY> = (-4..=4).map(|k| common::cell(k, 0)).collect();
    for (i, &c) in cells.iter().enumerate() {
        s.set_cell(c, 0, -1 - (i % 3) as i8);
    }
    let mut w = s.build();
    let centre = Pos::centre(common::cell(-1, 1), 0);
    shroud::area_reveal(&mut w, A, centre, 1);
    let explored = |c: CellXY| shroud::explored(&w, A, c);
    let (mut black, mut shown) = (0, 0);
    for &c in &cells {
        let mid = Pos::centre(c, 0);
        let z = w.surface_at(mid.x, mid.y);
        let level = i32::from(w.cells.cell(c).unwrap().level);
        // Flat cells at levels −1, −2 and −3 have centre heights −103, −207 and −311.
        assert_eq!(z, [-103, -207, -311][(-1 - level) as usize], "{c:?}");
        let (a, b) = shroud::point_candidates_on(Pos::new(mid.x, mid.y, z), true, true);
        let want = !explored(a) && !b.is_some_and(explored);
        assert_eq!(minimap::cell_shrouded(&w, A, c), want, "{c:?}");
        assert_eq!(minimap::cell_shrouded_on(&w, A, c, true), want, "{c:?}");
        assert_eq!(
            minimap::cell_shrouded_on(&w, A, c, false),
            !explored(c),
            "{c:?}"
        );
        if want {
            black += 1;
        } else {
            shown += 1;
        }
    }
    assert!(black > 0 && shown > 0, "both outcomes are reached");
}

// spec: S-0410 claim 6 and F3 (the cell shown at a pixel; at scale 1 the right-hand point of (x, y) shows (x + 1, y), quirks.radar_pixel_cell_shift)
#[test]
fn claim_6_cell_at_pixel() {
    let g = Grid {
        a: 9,
        b: 4,
        w: 140,
        n: 108,
        lw: 70,
    };
    let img = minimap::image(&Grid { w: 140, n: 50, ..g });
    let cell = CellXY::new(5, 3);
    let pts = minimap::cell_points(g.a, g.b, g.lw, cell);
    let (u0, v) = pts[0];
    assert_eq!(minimap::cell_at_pixel(&g, &img, u0, v, true), cell);
    assert_eq!(
        minimap::cell_at_pixel(&g, &img, u0 + 1, v, true),
        CellXY::new(6, 3)
    );
    assert_eq!(minimap::cell_at_pixel(&g, &img, u0 + 1, v, false), cell);
}

// spec: S-0410 claim 8 and F4 (an object at the centre of cell (x, y) lies on point row v + 1; an own object is clamped to the image)
#[test]
fn claim_8_object_pixel() {
    let g = Grid {
        a: 9,
        b: 4,
        w: 140,
        n: 50,
        lw: 70,
    };
    let img = minimap::image(&g);
    let cell = CellXY::new(5, 3);
    let (u, v) = minimap::cell_points(g.a, g.b, g.lw, cell)[0];
    let p = minimap::object_pixel(&g, &img, 256 * 5 + 128, 256 * 3 + 128, false);
    assert_eq!(p, (u, v + 1));
    assert_eq!(minimap::object_pixel(&g, &img, -5000, 0, true).0, 0);
}

// spec: S-0410 claim 10 and F5 (inverted when trunc((t − 1) / F) is odd; a negative F alternates too; F = 0 never inverts)
#[test]
fn claim_10_flash_phase() {
    let f = 7;
    let got: Vec<bool> = (1..=21).map(|t| minimap::flash_inverted(t, f)).collect();
    assert!(!got[0] && !got[6] && got[7] && got[13] && !got[14]);
    assert_eq!(
        minimap::flash_inverted(9, -7),
        minimap::flash_inverted(9, 7)
    );
    assert!(!minimap::flash_inverted(9, 0));
}

// spec: S-0410 claim 9 and F7 (tile colour ⌊trunc(min(c · f, 255)) / 2⌋, f 0.8 in snow; fixed colours of the other cases, not halved)
#[test]
fn claim_9_point_colours() {
    assert_eq!(
        minimap::tile_colour(Some([200, 101, 0]), false),
        [100, 50, 0]
    );
    // 200 × 0.8 (single precision, rounded toward zero) = 160.000…: trunc 160, half 80.
    assert_eq!(
        minimap::tile_colour(Some([200, 255, 10]), true),
        [80, 102, 4]
    );
    assert_eq!(minimap::tile_colour(None, false), [60, 60, 60]);
    assert_eq!(
        minimap::point_colour(PointSource::TerrainObject),
        [200, 200, 160]
    );
    assert_eq!(
        minimap::point_colour(PointSource::ResourceWithoutSprite),
        [170, 170, 130]
    );
    assert_eq!(
        minimap::point_colour(PointSource::Bridge([1, 2, 3])),
        [1, 2, 3]
    );
}

// spec: S-0410 claim 9 and T1 (overlay colours by list position, quirks.overlay_list_position_rules)
#[test]
fn claim_9_overlay_positions() {
    assert_eq!(minimap::overlay_rule(239, true, 4, true), None);
    assert_eq!(minimap::overlay_rule(80, false, 4, true), Some((1, false)));
    assert_eq!(minimap::overlay_rule(80, true, 4, true), Some((4, false)));
    assert_eq!(minimap::overlay_rule(130, false, 4, true), Some((4, true)));
    assert_eq!(
        minimap::overlay_rule(239, false, 4, false),
        Some((4, false))
    );
    assert_eq!(minimap::swap_green_blue([1, 2, 3]), [1, 3, 2]);
}

// spec: S-0410 claim 5 and F6 (the area-weighted mean of the point colours over a pixel, rounded half up)
#[test]
fn claim_5_terrain_mean() {
    // W = 210 points, scale 2/3: each pixel spans 1.5 points.
    let g = Grid {
        a: 0,
        b: 0,
        w: 210,
        n: 2,
        lw: 105,
    };
    let img = minimap::image(&g);
    assert_eq!(img.width, 140);
    let points: Vec<[u8; 3]> = (0..2 * 210)
        .map(|i| {
            if (i % 210) % 2 == 0 {
                [0, 0, 0]
            } else {
                [90, 90, 90]
            }
        })
        .collect();
    let pic = minimap::terrain_picture(&g, &img, &points);
    // Pixel 0 covers point 0 fully (black) and half of point 1 (90): 45 / 1.5 = 30.
    assert_eq!(pic[0], [30, 30, 30]);
    // Pixel 1 covers half of point 1 and point 2 fully: 30 as well; pixel 2 starts at point 3: 60.
    assert_eq!(pic[1], [30, 30, 30]);
}

// spec: S-0410 Terms and claims 6 and 10 (radar colours drop the low bits; halving and inversion at the channel's width)
#[test]
fn radar_colour_conversion() {
    let bits = Bits([5, 6, 5]);
    let c = minimap::radar_colour([170, 170, 130], bits);
    assert_eq!(c, [21, 42, 16]);
    assert_eq!(minimap::halved(c), [10, 21, 8]);
    assert_eq!(minimap::inverted(c, bits), [10, 21, 15]);
}

// spec: S-0410 claims 6, 7 and 8 (own discovered objects listed; enemy objects in shroud not listed; the winner at a pixel; shrouded cells black)
#[test]
fn claims_6_7_8_listing_and_compose() {
    let mut w = game(&rules());
    let d = put(&mut w, "PLANT", 0);
    let enemy = testkit::spawn(
        &mut w,
        Kind::Vehicle,
        "CART",
        1,
        testkit::mid(SIZE, 8, 8),
        0,
    );
    step(&mut w);
    let g = grid_of(&w);
    let img = minimap::image(&g);
    assert!(!minimap::listed(&w, A, d, &g, &img).is_empty());
    assert!(minimap::listed(&w, A, enemy, &g, &img).is_empty());
    let mut arr = minimap::Arrivals::default();
    arr.observe(&w, A, &g, &img);
    let px = minimap::listed(&w, A, d, &g, &img)[0];
    assert_eq!(arr.winner(&w, A, px, &g, &img), Some(d));
    let terrain = vec![[100, 100, 100]; (img.width * img.height) as usize];
    let colour = |_h: HouseId| [255, 0, 0];
    let flash = |_id: ObjectId| 0;
    let view = minimap::View {
        house: A,
        house_colour: &colour,
        neutral: [255, 255, 255],
        flash: &flash,
        flash_frame_time: 7,
        bits: Bits([5, 6, 5]),
        on_flat_grid: true,
    };
    let out = minimap::compose(&w, &view, &arr, &g, &img, &terrain);
    assert_eq!(out[(px.1 * img.width + px.0) as usize], [31, 0, 0]);
    assert!(out.contains(&[0, 0, 0]), "shrouded pixels are black");
    let _ = Rounding::TowardZero;
}

// spec: S-0410 claim 2 (r5: a successful upgrade placement is a recheck event even without a power change; the upgrade's own radar flag grants nothing)
#[test]
fn claim_2_upgrade_placement_rechecks() {
    let mut w = game(&rules());
    let p = put(&mut w, "PLANT", 0);
    step(&mut w);
    assert!(!w.houses[0].econ.radar.due);
    // UPDISH is the fourth building type (index 3).
    glasnost_sim::econ::placement::add_upgrade(&mut w, p, 3);
    assert!(w.houses[0].econ.radar.due);
    step(&mut w);
    assert!(!w.houses[0].econ.radar.due);
    assert!(!radar::available(&w, A));
}

fn image_at_scale_one() -> minimap::Image {
    let one = glasnost_num::F64::from_i64(1, Rounding::TowardZero).to_f32(Rounding::TowardZero);
    minimap::Image {
        s: one,
        width: 140,
        height: 108,
        left: 0,
        top: 0,
    }
}

fn row_widths(px: &[(i32, i32)]) -> Vec<usize> {
    let rows = px.iter().map(|p| p.1).max().map_or(0, |m| m + 1);
    (0..rows)
        .map(|r| px.iter().filter(|p| p.1 == r).count())
        .collect()
}

// spec: S-0410 F8 and claim 8 (the stepped diamond from the foundation's width and height and the scale; top pixel (p, q); the zero-size foundation is nonempty)
#[test]
fn claim_8_building_pixel_set() {
    let img = image_at_scale_one();
    let set = |f| minimap::building_pixels(&img, (20, 0), f);
    assert_eq!(set((1, 1)), vec![(20, 0)]);
    assert_eq!(row_widths(&set((2, 2))), vec![1, 3, 1]);
    assert_eq!(row_widths(&set((3, 3))), vec![1, 3, 5, 3, 1]);
    assert_eq!(row_widths(&set((0, 0))), vec![1, 3, 1]);
    // A parallelogram: a wider than b.
    let p = set((3, 2));
    assert_eq!(row_widths(&p), vec![1, 3, 3, 1]);
    assert_eq!(p[0], (20, 0));
}

// spec: S-0410 F8 (pixels outside the radar image are omitted, never clamped)
#[test]
fn claim_8_building_set_not_clamped() {
    let mut w = game(&rules());
    let d = put(&mut w, "PLANT", 0);
    step(&mut w);
    let g = grid_of(&w);
    let img = minimap::image(&g);
    let o = w.obj(d).unwrap();
    let top = minimap::object_pixel(&g, &img, o.pos.x, o.pos.y, false);
    let all = minimap::building_pixels(&img, top, o.foundation);
    let listed = minimap::listed(&w, A, d, &g, &img);
    let inside: Vec<_> = all
        .into_iter()
        .filter(|&(p, q)| p >= 0 && q >= 0 && p < img.width && q < img.height)
        .collect();
    assert_eq!(listed, inside);
}

// spec: S-0410 claim 7 (an own object is listed only with the discovered-by-H mark; ownership alone does not pass)
#[test]
fn claim_7_own_object_needs_the_mark() {
    let mut w = game(&rules());
    let d = put(&mut w, "PLANT", 0);
    step(&mut w);
    let g = grid_of(&w);
    let img = minimap::image(&g);
    assert!(!minimap::listed(&w, A, d, &g, &img).is_empty());
    for st in w.econ.shroud.houses.iter_mut().flatten() {
        st.discovered.clear();
    }
    assert!(!shroud::discovered(&w, A, d));
    assert!(minimap::listed(&w, A, d, &g, &img).is_empty());
}

// spec: S-0410 claim 1 (r6: the qualifying building is switched on, on the map and not being sold; no further test of its strength)
#[test]
fn claim_1_no_alive_test() {
    let mut w = game(&rules());
    put(&mut w, "PLANT", 0);
    let d = put(&mut w, "DISH", 2);
    step(&mut w);
    w.obj_mut(d).unwrap().health = 0;
    assert!(radar::claim_1(&w, A));
}

const RZ: Rounding = Rounding::TowardZero;

fn single(text: &str) -> glasnost_num::F32 {
    glasnost_num::decimal::nearest_double(text).to_f32(RZ)
}

fn zoom_rules(speed: &str, min_radius: i32, rotation: &str) -> minimap::ZoomRules {
    minimap::ZoomRules {
        speed: single(speed),
        min_radius,
        rotation: single(rotation),
    }
}

// spec: S-0410 Import mapping (RadarEventSpeed 1, RadarEventMinRadius 5, RadarEventRotationSpeed 0.10000000149011612 when absent; floating number or percent, stored single)
#[test]
fn f9_zoom_import() {
    let z = radar::Rules::new(&rules(), RZ).zoom;
    assert_eq!(z, zoom_rules("1", 5, "0.10000000149011612"));
    let mut r = rules();
    let g = &mut r.sim.general.economy;
    g.radar_event_speed = Some(glasnost_rules::sim::Decimal("250%".into()));
    g.radar_event_min_radius = Some(7);
    g.radar_event_rotation_speed = Some(glasnost_rules::sim::Decimal("0.3".into()));
    let z = radar::Rules::new(&r, RZ).zoom;
    assert_eq!(z.speed, single("2.5"));
    assert_eq!(z.min_radius, 7);
    assert_eq!(
        z.rotation,
        glasnost_num::decimal::parse("0.3", glasnost_num::decimal::Field::Single, RZ)
            .unwrap()
            .to_f32(RZ)
    );
}

// spec: S-0410 F9 (the event centre: trunc((x − y + a) × s + Oₓ) − Oₓ and trunc((x + y − b) × s + Oᵧ) − Oᵧ; a column −1 becomes 0)
#[test]
fn f9_event_centre() {
    let g = Grid {
        a: 3,
        b: 4,
        w: 40,
        n: 40,
        lw: 20,
    };
    let one = image_at_scale_one();
    assert_eq!(minimap::event_centre(&g, &one, (10, 20), 5, 6), (2, 7));
    assert_eq!(minimap::event_centre(&g, &one, (10, 20), 2, 6), (0, 4));
    let half = minimap::Image {
        s: single("0.5"),
        ..one
    };
    // 3 × 0.5 = 1.5: truncated after adding the screen origin.
    assert_eq!(minimap::event_centre(&g, &half, (10, 0), 4, 4).0, 1);
    assert_eq!(minimap::event_centre(&g, &half, (-10, 0), 4, 4).0, 2);
}

// spec: S-0410 claim 14 and F9 (R₀ = max(p, q, w − p, h − q); θ₀ = 0.7853981852531433; ω₀ = Ω)
#[test]
fn f9_initial_square() {
    let r = zoom_rules("1", 5, "0.25");
    let z = minimap::Zoom::new(&r, (10, 20), 140, 108);
    assert_eq!(z.radius, single("130"));
    assert_eq!(z.angle, single("0.7853981852531433"));
    assert_eq!(z.step, r.rotation);
    assert_eq!(
        minimap::Zoom::new(&r, (70, 100), 140, 108).radius,
        single("100")
    );
}

// spec: S-0410 claim 14 and F9 (reaching the minimum radius alone does not complete: rotation slows until d < ω)
#[test]
fn f9_zoom_progression() {
    let r = zoom_rules("1", 5, "0.5");
    let mut z = minimap::Zoom::new(&r, (8, 8), 16, 16);
    assert_eq!(z.radius, single("8"));
    // Frames 1 and 2 contract by S at the full angular speed.
    assert!(!z.advance(&r));
    assert_eq!(z.radius, single("7"));
    assert!(!z.advance(&r));
    assert_eq!(z.radius, single("6"));
    assert_eq!(z.step, r.rotation);
    // Frame 3 reaches the minimum, but d ≈ 1.0 is not below ω = 0.5:
    // ω becomes RZ₃₂(max(Ω × 0.3333333432674408, RZ₃₂(ω − Ω × 0.019999999552965164))).
    assert!(!z.advance(&r));
    assert_eq!(z.radius, single("5"));
    assert_eq!(z.step, single("0.4899999797344207763671875"));
    // Frame 4: d ≈ 1.5, not below ω ≈ 0.49.
    assert!(!z.advance(&r));
    let before = z.angle;
    // Frame 5: d ≈ 0.42 < ω ≈ 0.48 completes; the angle takes d, not ω.
    assert!(z.advance(&r));
    assert_eq!(z.radius, single("5"));
    let turned = z.angle.to_f64().sub(before.to_f64(), RZ);
    assert!(turned.lt(single("0.45").to_f64()) && turned.gt(single("0.4").to_f64()));
}

// spec: S-0410 claim 14 and F9 (a zoom already at the minimum completes on the first drawn frame when d < ω)
#[test]
fn f9_first_frame_completion() {
    let r = zoom_rules("1", 5, "0.10000000149011612");
    let mut z = minimap::Zoom::new(&r, (6, 6), 12, 12);
    assert!(z.advance(&r));
    assert_eq!(z.radius, single("5"));
}

// spec: S-0410 F9 (a zero angular speed keeps the initial angle and never completes zoom; the radius stays at the minimum)
#[test]
fn f9_zero_rotation_never_completes() {
    let r = zoom_rules("1", 5, "0");
    let mut z = minimap::Zoom::new(&r, (10, 10), 20, 20);
    for _ in 0..10_000 {
        assert!(!z.advance(&r));
    }
    assert_eq!(z.angle, single("0.7853981852531433"));
    assert_eq!(z.radius, single("5"));
}

// spec: S-0410 F9 (an angle above 6.283185307179586 has that value subtracted once, not an unrestricted modulo)
#[test]
fn f9_angle_wraps_once() {
    let six = zoom_rules("1", 5, "6");
    let mut z = minimap::Zoom::new(&six, (50, 50), 100, 100);
    assert!(!z.advance(&six));
    assert!(z.angle.to_f64().lt(single("0.6").to_f64()));
    assert!(z.angle.to_f64().gt(single("0.4").to_f64()));
    let big = zoom_rules("1", 5, "13");
    let mut z = minimap::Zoom::new(&big, (50, 50), 100, 100);
    assert!(!z.advance(&big));
    assert!(z.angle.to_f64().gt(single("7.4").to_f64()));
}
