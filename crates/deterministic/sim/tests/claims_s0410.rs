//! Claim tests for S-0410 (radar availability, and the pure functions of
//! the radar map) on synthetic data: invented types, no stock values.

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
        ["PLANT", "DISH", "CONS"]
            .iter()
            .map(|s| s.to_string())
            .collect(),
    );
    r.sim.buildings.insert("PLANT".into(), bt(100, false));
    r.sim.buildings.insert("DISH".into(), bt(-20, true));
    r.sim.buildings.insert("CONS".into(), bt(-200, false));
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

// spec: S-0410 claim 2 (rechecked in the house's next update after an event: a power order in tick n changes the panel in tick n + 1)
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
    };
    let out = minimap::compose(&w, &view, &arr, &g, &img, &terrain);
    assert_eq!(out[(px.1 * img.width + px.0) as usize], [31, 0, 0]);
    assert!(out.contains(&[0, 0, 0]), "shrouded pixels are black");
    let _ = Rounding::TowardZero;
}
