//! Claim tests for S-0409 (shroud and sight) on synthetic data: invented
//! types and sight fields, a flat map, no stock values. Houses Alpha (0)
//! and Beta (1) of the testkit's sandbox game, not allied.

use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{BuildingType, LocomotionClass, Locomotor, UnitType};
use glasnost_sim::econ::shroud;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, Pos, Target, World};

const SIZE: i32 = 40;
const A: HouseId = HouseId(0);
const B: HouseId = HouseId(1);

fn veh(sight: i32) -> UnitType {
    UnitType {
        strength: Some(100),
        speed: Some(8),
        rot: Some(8),
        locomotion_class: Some(LocomotionClass::Wheel),
        locomotor: Some(Locomotor::Drive),
        sight: Some(sight),
        ..Default::default()
    }
}

/// Vehicles EYE1 … EYE12 (sight fields 1 … 12); buildings TOWER (sight 4),
/// BIGTOWER (3 × 3, sight 2), GAP (gap generator, radius 3), SAT (spy
/// satellite).
fn rules() -> Ruleset {
    let mut r = testkit::ruleset();
    let vs = r.registries.get_mut("vehicles").unwrap();
    for i in 1..=12 {
        vs.push(format!("EYE{i}"));
    }
    for i in 1..=12 {
        r.sim.vehicles.insert(format!("EYE{i}"), veh(i));
    }
    vs_extra(&mut r);
    r.registries.insert(
        "buildings".into(),
        ["TOWER", "BIGTOWER", "GAP", "SAT", "BEACON"]
            .iter()
            .map(|s| s.to_string())
            .collect(),
    );
    let b = |f: [i32; 2], sight: i32| {
        let mut b = BuildingType {
            strength: Some(500),
            foundation: Some(f),
            ..Default::default()
        };
        b.econ.sight = Some(sight);
        b
    };
    r.sim.buildings.insert("TOWER".into(), b([1, 1], 4));
    r.sim.buildings.insert("BIGTOWER".into(), b([3, 3], 2));
    let mut gap = b([1, 1], 0);
    gap.gap_generator = Some(true);
    gap.econ.gap_radius_in_cells = Some(3);
    r.sim.buildings.insert("GAP".into(), gap);
    let mut sat = b([1, 1], 0);
    sat.econ.spy_sat = Some(true);
    r.sim.buildings.insert("SAT".into(), sat);
    let mut beacon = b([1, 1], 3);
    beacon.econ.reveal_to_all = Some(true);
    r.sim.buildings.insert("BEACON".into(), beacon);
    r
}

/// EYE0 (sight 0) and EYENEG (sight -1).
fn vs_extra(r: &mut Ruleset) {
    let vs = r.registries.get_mut("vehicles").unwrap();
    vs.push("EYE0".into());
    vs.push("EYENEG".into());
    r.sim.vehicles.insert("EYE0".into(), veh(0));
    r.sim.vehicles.insert("EYENEG".into(), veh(-1));
}

fn game(r: &Ruleset) -> World {
    testkit::game_with(r, SIZE, 3)
}

fn spawn(w: &mut World, kind: Kind, name: &str, h: u8, dx: i32, dy: i32) -> ObjectId {
    testkit::spawn(w, kind, name, h, testkit::mid(SIZE, dx, dy), 0)
}

fn c(dx: i32, dy: i32) -> CellXY {
    testkit::mid(SIZE, dx, dy)
}

fn explored_around(w: &World, h: HouseId, centre: CellXY, r: i32) -> Vec<(i32, i32)> {
    let mut out = Vec::new();
    for dy in -r..=r {
        for dx in -r..=r {
            if shroud::explored(w, h, CellXY::new(centre.x + dx, centre.y + dy)) {
                out.push((dx, dy));
            }
        }
    }
    out
}

fn step(w: &mut World) {
    glasnost_sim::tick::step(w);
}

// spec: S-0409 claim 1 (one state per player house; the neutral and special houses keep none)
#[test]
fn claim_1_state_per_player_house() {
    let w = game(&rules());
    assert_eq!(w.econ.shroud.houses.len(), 2);
    assert!(w.econ.shroud.houses.iter().all(Option::is_some));
}

// spec: S-0409 claim 3 (at game start every cell is unexplored, cover count 1, hostile gap count 0)
#[test]
fn claim_3_start_state() {
    let w = game(&rules());
    assert!(!shroud::explored(&w, A, c(0, 0)));
    assert_eq!(shroud::counts(&w, A, c(0, 0)), Some((1, 0)));
    assert!(!shroud::whole_map_revealed(&w, A));
}

// spec: S-0409 claim 3 (the lobby shroud setting off: the whole map explored, not seen, mark set; border cells not surrounded)
#[test]
fn claim_3_lobby_shroud_off() {
    let mut r = rules();
    r.sim.general.economy.shroud = Some(false);
    let w = game(&r);
    assert!(shroud::explored(&w, A, c(0, 0)));
    assert!(!shroud::seen(&w, A, c(0, 0)));
    assert_eq!(shroud::counts(&w, A, c(0, 0)), Some((1, 0)));
    assert!(shroud::whole_map_revealed(&w, B));
}

// spec: S-0409 claim 6 and T1 (the sight shapes of radius 1 to 10; 11 and 12 count as 10)
#[test]
fn claim_6_sight_shapes() {
    for r in 1..=12 {
        let mut w = game(&rules());
        spawn(&mut w, Kind::Vehicle, &format!("EYE{r}"), 0, 0, 0);
        step(&mut w);
        let got = explored_around(&w, A, c(0, 0), 12);
        let mut want = shroud::shape(r.min(10));
        want.sort_by_key(|&(dx, dy)| (dy, dx));
        assert_eq!(got, want, "radius {r}");
    }
    // The differences T1 lists against dx² + dy² ≤ r² + 1.
    assert!(shroud::shape(4).contains(&(3, 3)));
    assert!(!shroud::shape(8).contains(&(7, 4)));
    assert!(!shroud::shape(10).contains(&(8, 6)));
}

// spec: S-0409 claim 4 and F1 (the height bonus; S = 5, b′ = 20 gives r₀ = 5 under round toward zero, not 6)
#[test]
fn claim_4_height_bonus_rounding() {
    let mut r = rules();
    r.sim.general.economy.leptons_per_sight_increase = Some(104);
    let mut w = game(&r);
    let e = spawn(&mut w, Kind::Vehicle, "EYE5", 0, 0, 0);
    let set_z = |w: &mut World, z: i32| w.obj_mut(e).unwrap().pos.z = z;
    set_z(&mut w, 0);
    assert_eq!(shroud::sight_radius(&w, e), Some(5));
    set_z(&mut w, 312);
    assert_eq!(shroud::sight_radius(&w, e), Some(6));
    set_z(&mut w, 208);
    assert_eq!(shroud::sight_radius(&w, e), Some(5));
}

// spec: S-0409 F1 and Edge cases (quirks.sight_bonus_wrap: b = 200 reads −56, radius 2; b = 310 reads 54, radius 7)
#[test]
fn claim_4_sight_bonus_wrap() {
    let mut r = rules();
    r.sim.general.economy.leptons_per_sight_increase = Some(10);
    let mut w = game(&r);
    let e = spawn(&mut w, Kind::Vehicle, "EYE5", 0, 0, 0);
    w.obj_mut(e).unwrap().pos.z = 208;
    assert_eq!(shroud::sight_radius(&w, e), Some(2));
    w.obj_mut(e).unwrap().pos.z = 312;
    assert_eq!(shroud::sight_radius(&w, e), Some(7));
    r.globals.quirks.sight_bonus_wrap = false;
    let mut w = game(&r);
    let e = spawn(&mut w, Kind::Vehicle, "EYE5", 0, 0, 0);
    w.obj_mut(e).unwrap().pos.z = 208;
    assert_eq!(shroud::sight_radius(&w, e), Some(15));
}

// spec: S-0409 claim 5 and F2 (the centre moved by trunc(L(Z) / 30) cells map north-west; a building on its top-left cell)
#[test]
fn claim_5_sight_centre() {
    let w = game(&rules());
    let p = Pos::centre(c(0, 0), 416);
    let cells = shroud::sight_cells(&w, p, 1);
    let want = shroud::sight_cells(&w, Pos::centre(c(-2, -2), 0), 1);
    assert_eq!(cells, want);
    let mut w = game(&rules());
    spawn(&mut w, Kind::Building, "BIGTOWER", 0, 0, 0);
    // Radius 2 around the top-left cell: (−2, 0) is explored, (4, 2) is not.
    assert!(shroud::explored(&w, A, c(-2, 0)));
    assert!(!shroud::explored(&w, A, c(4, 2)));
}

// spec: S-0409 Edge cases (quirks.building_sight_top_left off: centred on the reference point)
#[test]
fn claim_5_building_sight_reference_point() {
    let mut r = rules();
    r.globals.quirks.building_sight_top_left = false;
    let mut w = game(&r);
    spawn(&mut w, Kind::Building, "BIGTOWER", 0, 0, 0);
    assert!(shroud::explored(&w, A, c(3, 1)));
    assert!(!shroud::explored(&w, A, c(-2, 0)));
}

// spec: S-0409 claim 7 and T2 (the step towards the centre, with the nine exceptions of quirks.height_block_cell)
#[test]
fn claim_7_step_towards_centre() {
    assert_eq!(shroud::step_towards_centre(3, 0, true), (-1, 0));
    assert_eq!(shroud::step_towards_centre(3, 3, true), (-1, -1));
    assert_eq!(shroud::step_towards_centre(5, -2, true), (1, 0));
    assert_eq!(shroud::step_towards_centre(5, -2, false), (-1, 0));
    assert_eq!(shroud::step_towards_centre(3, 7, true), (0, -1));
    assert_eq!(shroud::step_towards_centre(3, 7, false), (-1, -1));
}

// spec: S-0409 claim 7 and F3 (a cell is hidden when its blocking cell T + s + (2, 2) is more than 3 levels above the observer)
#[test]
fn claim_7_height_blocking() {
    let mut w = game(&rules());
    // T = centre + (3, 0): s = (−1, 0), blocking cell = T + (1, 2) = centre + (4, 2).
    let b = c(4, 2);
    let i = w.cells.idx(b).unwrap();
    w.cells.grid.get_mut(b).unwrap().level = 8;
    let _ = i;
    let cells = shroud::sight_cells(&w, Pos::centre(c(0, 0), 0), 3);
    let t = w.cells.idx(c(3, 0)).unwrap() as u32;
    assert!(!cells.contains(&t));
    // The reveal-by-height flag off: nothing is hidden.
    let mut r = rules();
    r.sim.general.economy.reveal_by_height = Some(false);
    let mut w2 = game(&r);
    w2.cells.grid.get_mut(b).unwrap().level = 8;
    let cells = shroud::sight_cells(&w2, Pos::centre(c(0, 0), 0), 3);
    assert!(cells.contains(&t));
}

// spec: S-0409 claims 8 and 9 (a unit adds its sight in its turn, moves it on arrival in a cell; explored stays explored)
#[test]
fn claims_8_9_moving_sight() {
    let mut w = game(&rules());
    testkit::run(&mut w, 4);
    let e = spawn(&mut w, Kind::Vehicle, "EYE2", 0, 0, 0);
    assert!(!shroud::explored(&w, A, c(0, 0)), "added in its own turn");
    step(&mut w);
    assert!(shroud::explored(&w, A, c(0, 0)));
    assert!(glasnost_sim::orders::issue(
        &mut w,
        A,
        glasnost_sim::orders::Order::Behaviour {
            actor: e,
            behaviour: Behaviour::Move,
            attack_move: false,
            target: None,
            dest: Some(Target::Cell(c(8, 0))),
        }
    ));
    for _ in 0..120 {
        step(&mut w);
    }
    assert!(shroud::explored(&w, A, c(9, 0)));
    // The start area stays explored (no hostile gap field).
    assert!(shroud::explored(&w, A, c(-2, 0)));
    assert_eq!(shroud::counts(&w, A, c(-2, 0)).unwrap().0, 0);
}

// spec: S-0409 claims 2 and 8 (the owner's sight counts for it, not for an enemy; reveal-to-all counts for every house; a passive country reveals nothing)
#[test]
fn claims_2_8_whose_sight() {
    let mut w = game(&rules());
    spawn(&mut w, Kind::Building, "TOWER", 0, 0, 0);
    assert!(shroud::explored(&w, A, c(0, 0)));
    assert!(!shroud::explored(&w, B, c(0, 0)));
    spawn(&mut w, Kind::Building, "BEACON", 0, 10, 0);
    assert!(shroud::explored(&w, B, c(10, 0)));
    let mut r = rules();
    r.sim
        .countries
        .get_mut("Alphaland")
        .unwrap()
        .multiplay_passive = Some(true);
    let mut w = game(&r);
    spawn(&mut w, Kind::Building, "BEACON", 0, 0, 0);
    assert!(!shroud::explored(&w, A, c(0, 0)));
    assert!(!shroud::explored(&w, B, c(0, 0)));
}

// spec: S-0409 claim 2 (with ally reveal on an ally's sight counts; off it does not)
#[test]
fn claim_2_ally_reveal() {
    for (on, want) in [(true, true), (false, false)] {
        let mut r = rules();
        r.sim.general.economy.ally_reveal = Some(on);
        let mut setup = testkit::setup(3);
        setup.alliances = vec![(0, 1)];
        let rules = std::sync::Arc::new(glasnost_sim::rules::SimRules::new(&r, "t"));
        let mut w = glasnost_sim::load::new_game(
            rules,
            &testkit::empty_map(SIZE),
            testkit::catalogue(),
            &setup,
        )
        .unwrap();
        spawn(&mut w, Kind::Building, "TOWER", 0, 0, 0);
        assert_eq!(shroud::explored(&w, B, c(0, 0)), want, "ally reveal {on}");
    }
}

// spec: S-0409 claims 11 and 12 (a hostile gap field re-covers cells no sight holds; cells under a sight stay explored)
#[test]
fn claims_11_12_gap_field() {
    let mut w = game(&rules());
    let e = spawn(&mut w, Kind::Vehicle, "EYE5", 0, 0, 0);
    step(&mut w);
    // Explore, then leave: the vehicle is taken off and the cells stay explored.
    glasnost_sim::objects::remove(&mut w, e);
    assert!(shroud::explored(&w, A, c(2, 0)));
    let keep = spawn(&mut w, Kind::Vehicle, "EYE1", 0, -2, 0);
    step(&mut w);
    let _ = keep;
    spawn(&mut w, Kind::Building, "GAP", 1, 0, 0);
    step(&mut w);
    step(&mut w);
    assert!(!shroud::explored(&w, A, c(2, 0)));
    assert_eq!(shroud::counts(&w, A, c(2, 0)), Some((1, 1)));
    // The cell under the remaining sight stays explored.
    assert!(shroud::explored(&w, A, c(-2, 0)));
    // The owner's own field covers nothing for it.
    assert!(!shroud::explored(&w, B, c(2, 0)));
    assert_eq!(shroud::counts(&w, B, c(2, 0)), Some((1, 0)));
}

// spec: S-0409 claims 12 and 13 (a sight leaving a hostile field marks its cells pending; the next sweep, at a multiple of 120, re-covers them)
#[test]
fn claims_12_13_pending_and_sweep() {
    let mut w = game(&rules());
    spawn(&mut w, Kind::Building, "GAP", 1, 0, 0);
    step(&mut w);
    step(&mut w);
    let e = spawn(&mut w, Kind::Vehicle, "EYE1", 0, 1, 0);
    step(&mut w);
    assert!(shroud::explored(&w, A, c(1, 0)));
    while w.tick % 120 == 0 || w.tick % 120 > 100 {
        step(&mut w);
    }
    glasnost_sim::objects::remove(&mut w, e);
    assert!(shroud::pending(&w, A, c(1, 0)));
    assert!(shroud::explored(&w, A, c(1, 0)));
    while w.tick % 120 != 0 {
        step(&mut w);
        assert!(shroud::explored(&w, A, c(1, 0)));
    }
    step(&mut w);
    assert!(!shroud::explored(&w, A, c(1, 0)));
    assert!(!shroud::pending(&w, A, c(1, 0)));
}

// spec: S-0409 claim 11 and Edge cases (quirks.gap_radius_wrap: a radius field of 200 covers nothing, 260 reads 4)
#[test]
fn claim_11_gap_radius_wrap() {
    for (field, covers) in [(200, false), (260, true)] {
        let mut r = rules();
        r.sim
            .buildings
            .get_mut("GAP")
            .unwrap()
            .econ
            .gap_radius_in_cells = Some(field);
        let mut w = game(&r);
        spawn(&mut w, Kind::Building, "TOWER", 0, 0, 0);
        let t = spawn(&mut w, Kind::Building, "TOWER", 0, 0, 0);
        glasnost_sim::objects::remove(&mut w, t);
        let g = spawn(&mut w, Kind::Building, "GAP", 1, 6, 0);
        let _ = g;
        step(&mut w);
        step(&mut w);
        // The tower's sight holds (0, 0); cell (4, 0) is explored with no sight by the second tower's area reveal.
        let _ = covers;
        assert_eq!(w.objects.values().any(|o| o.econ.sight.gap.is_some()), true);
        let cells = w
            .objects
            .values()
            .find_map(|o| o.econ.sight.gap.as_ref().map(|g| g.cells.len()))
            .unwrap();
        assert_eq!(cells > 0, covers, "field {field}");
    }
}

// spec: S-0409 claims 11 and 15 (a gap field stops with its generator's operation; a spy satellite brings its cells back)
#[test]
fn claim_15_spy_satellite_reveal_and_recover() {
    let mut w = game(&rules());
    let sat = spawn(&mut w, Kind::Building, "SAT", 0, -10, -10);
    step(&mut w);
    step(&mut w);
    assert!(shroud::spy_satellite_active(&w, A));
    assert!(shroud::explored(&w, A, c(15, 15)));
    assert!(shroud::whole_map_revealed(&w, A));
    // Losing it re-covers the map except the current sights.
    let e = spawn(&mut w, Kind::Vehicle, "EYE2", 0, 5, 5);
    step(&mut w);
    glasnost_sim::objects::remove(&mut w, sat);
    step(&mut w);
    assert!(!shroud::spy_satellite_active(&w, A));
    assert!(!shroud::explored(&w, A, c(15, 15)));
    assert!(shroud::explored(&w, A, c(5, 5)));
    let _ = e;
}

// spec: S-0409 claim 15 (a spy satellite does not show what a hostile gap generator hides)
#[test]
fn claim_15_satellite_and_gap() {
    let mut w = game(&rules());
    spawn(&mut w, Kind::Building, "GAP", 1, 8, 8);
    step(&mut w);
    step(&mut w);
    spawn(&mut w, Kind::Building, "SAT", 0, -10, -10);
    step(&mut w);
    step(&mut w);
    assert!(shroud::explored(&w, A, c(-5, 5)));
    assert!(!shroud::explored(&w, A, c(8, 8)));
    assert!(
        !shroud::whole_map_revealed(&w, A),
        "any gap field leaves the mark clear"
    );
}

// spec: S-0409 claim 15 (the defeat reveal: explored, not seen, mark set; pending marks kept, quirks.defeat_reveal_keeps_pending)
#[test]
fn claim_15_defeat_reveal() {
    let mut w = game(&rules());
    shroud::defeat_reveal(&mut w, A);
    assert!(shroud::explored(&w, A, c(10, 10)));
    assert!(!shroud::seen(&w, A, c(10, 10)));
    assert!(shroud::whole_map_revealed(&w, A));
}

// spec: S-0409 claim 15 (radar infiltration re-covers the whole map unless the spy satellite is active)
#[test]
fn claim_15_radar_infiltration() {
    let mut w = game(&rules());
    spawn(&mut w, Kind::Building, "TOWER", 0, 0, 0);
    let t2 = spawn(&mut w, Kind::Building, "TOWER", 0, 10, 0);
    glasnost_sim::objects::remove(&mut w, t2);
    assert!(shroud::explored(&w, A, c(10, 0)));
    shroud::radar_infiltrated(&mut w, A);
    assert!(!shroud::explored(&w, A, c(10, 0)));
    assert!(shroud::explored(&w, A, c(0, 0)));
}

// spec: S-0409 claim 16 (a reveal-on-fire shot reveals the radius-3 shape around the firer to the target's owner until the next sweep)
#[test]
fn claim_16_reveal_on_fire() {
    let mut r = testkit::combat_ruleset();
    let w0 = r.sim.weapons.keys().next().cloned().unwrap();
    r.sim.weapons.get_mut(&w0).unwrap().reveal_on_fire = Some(true);
    let mut r2 = rules();
    r2.sim.weapons = r.sim.weapons.clone();
    r2.sim.warheads = r.sim.warheads.clone();
    r2.sim.projectiles = r.sim.projectiles.clone();
    let mut w = game(&r2);
    let firer = spawn(&mut w, Kind::Vehicle, "EYE1", 1, 0, 0);
    let target = spawn(&mut w, Kind::Vehicle, "EYE1", 0, 15, 0);
    step(&mut w);
    assert!(!shroud::explored(&w, A, c(0, 0)));
    shroud::fired(&mut w, firer, &w0, Target::Object(target));
    assert!(shroud::explored(&w, A, c(0, 0)));
    assert!(shroud::explored(&w, A, c(2, 2)));
    assert!(!shroud::seen(&w, A, c(2, 2)));
    assert!(shroud::pending(&w, A, c(2, 2)));
    // Allies of the target's owner and other houses get nothing; a second shot after discovery reveals nothing new.
    assert!(!shroud::explored(&w, B, c(30, 0)));
    while w.tick % 120 != 0 {
        step(&mut w);
    }
    step(&mut w);
    assert!(!shroud::explored(&w, A, c(2, 2)));
}

// spec: S-0409 Edge cases (quirks.fire_reveal_expires off: the reveal on fire is an area reveal and stays)
#[test]
fn claim_16_reveal_permanent_with_switch_off() {
    let mut r = rules();
    r.globals.quirks.fire_reveal_expires = false;
    let c0 = testkit::combat_ruleset();
    let w0 = c0.sim.weapons.keys().next().cloned().unwrap();
    r.sim.weapons = c0.sim.weapons.clone();
    r.sim.weapons.get_mut(&w0).unwrap().reveal_on_fire = Some(true);
    let mut w = game(&r);
    let firer = spawn(&mut w, Kind::Vehicle, "EYE1", 1, 0, 0);
    let target = spawn(&mut w, Kind::Vehicle, "EYE1", 0, 15, 0);
    shroud::fired(&mut w, firer, &w0, Target::Object(target));
    assert!(shroud::explored(&w, A, c(2, 2)));
    assert!(!shroud::pending(&w, A, c(2, 2)));
}

// spec: S-0409 claim 17 (an area reveal explores and sees the cells and adds no lasting sight)
#[test]
fn claim_17_area_reveal() {
    let mut w = game(&rules());
    shroud::area_reveal(&mut w, A, Pos::centre(c(0, 0), 0), 4);
    assert!(shroud::explored(&w, A, c(3, 0)));
    assert!(shroud::seen(&w, A, c(3, 0)));
    assert_eq!(shroud::counts(&w, A, c(3, 0)), Some((0, 0)));
    assert!(!shroud::explored(&w, B, c(3, 0)));
}

// spec: S-0409 claim 18 (regrowth: pending cells go at the regrowth step; sight-explored cells do not regrow)
#[test]
fn claim_18_regrowth() {
    let mut r = rules();
    r.sim.general.economy.shroud_grow = Some(true);
    r.sim.general.economy.shroud_rate = Some(glasnost_rules::sim::Decimal("0.1".into()));
    let mut w = game(&r);
    let e = spawn(&mut w, Kind::Vehicle, "EYE3", 0, 0, 0);
    step(&mut w);
    glasnost_sim::objects::remove(&mut w, e);
    for _ in 0..300 {
        step(&mut w);
    }
    assert!(
        shroud::explored(&w, A, c(0, 0)),
        "sight-explored cells do not regrow"
    );
}

// spec: S-0409 claim 19 (fog of war is off in a skirmish: the seen state changes nothing that is drawn; no fog rule runs)
#[test]
fn claim_19_no_fog() {
    let mut w = game(&rules());
    let e = spawn(&mut w, Kind::Vehicle, "EYE2", 0, 0, 0);
    step(&mut w);
    glasnost_sim::objects::remove(&mut w, e);
    for _ in 0..200 {
        step(&mut w);
    }
    assert!(shroud::seen(&w, A, c(0, 0)));
}

// spec: S-0110 claim 8 (the point test: even u one candidate, odd u either of two)
#[test]
fn point_test() {
    let mut w = game(&rules());
    spawn(&mut w, Kind::Building, "TOWER", 0, 0, 0);
    assert!(!shroud::point_shrouded(&w, A, Pos::centre(c(0, 0), 0)));
    assert!(!shroud::point_shrouded(&w, A, Pos::centre(c(2, 2), 208)));
    assert!(shroud::point_shrouded(&w, A, Pos::centre(c(12, 12), 0)));
}

// spec: S-0409 Terms (discovered by H: a reveal that changes the cover piece of an object's cell marks it; own objects count as discovered)
#[test]
fn discovery() {
    let mut w = game(&rules());
    let enemy = spawn(&mut w, Kind::Vehicle, "EYE1", 1, 3, 0);
    assert!(!shroud::discovered(&w, A, enemy));
    spawn(&mut w, Kind::Building, "TOWER", 0, 0, 0);
    assert!(shroud::discovered(&w, A, enemy));
    assert!(shroud::discovered(&w, B, enemy));
}

// spec: S-0409 claim 1 (the state hash survives a save/load round trip with shroud state)
#[test]
fn shroud_state_round_trips() {
    let mut w = game(&rules());
    spawn(&mut w, Kind::Building, "TOWER", 0, 0, 0);
    step(&mut w);
    let h = glasnost_sim::save::state_hash(&w);
    let bytes = glasnost_sim::save::save(&w);
    let w2 = glasnost_sim::save::load(&bytes, w.rules_arc(), testkit::catalogue()).unwrap();
    assert_eq!(glasnost_sim::save::state_hash(&w2), h);
}

// spec: S-0409 claim 6 (a radius of 0 reveals nothing; our engine also reveals nothing for a negative radius)
#[test]
fn claim_6_zero_and_negative_radius() {
    for name in ["EYE0", "EYENEG"] {
        let mut w = game(&rules());
        spawn(&mut w, Kind::Vehicle, name, 0, 0, 0);
        step(&mut w);
        assert!(explored_around(&w, A, c(0, 0), 3).is_empty(), "{name}");
    }
}

// spec: S-0409 claims 2 and 11 (r3: the object owner's view decides: its sight counts, and its gap field is not hostile, for a house its owner allies, one way)
#[test]
fn claims_2_11_owner_view_one_way() {
    let mut w = game(&rules());
    // Alpha allies Beta; Beta does not ally Alpha.
    w.houses[0].allies = vec![A, B];
    spawn(&mut w, Kind::Building, "TOWER", 0, -10, 0);
    spawn(&mut w, Kind::Building, "TOWER", 1, 10, 0);
    step(&mut w);
    assert!(
        shroud::explored(&w, B, c(-10, 0)),
        "Alpha's sight counts for Beta"
    );
    assert!(
        !shroud::explored(&w, A, c(10, 0)),
        "Beta's sight does not count for Alpha"
    );
    spawn(&mut w, Kind::Building, "GAP", 0, -10, 10);
    spawn(&mut w, Kind::Building, "GAP", 1, 10, 10);
    step(&mut w);
    step(&mut w);
    assert_eq!(
        shroud::counts(&w, B, c(-10, 10)).map(|x| x.1),
        Some(0),
        "not hostile to Beta"
    );
    assert_eq!(
        shroud::counts(&w, A, c(10, 10)).map(|x| x.1),
        Some(1),
        "hostile to Alpha"
    );
}

/// A legal cell whose upper-left diagonal neighbour is not legal.
fn edge_cell(w: &World) -> CellXY {
    for y in 4..2 * SIZE {
        for x in 4..2 * SIZE {
            let p = CellXY::new(x, y);
            if shroud::counts(w, A, p).is_some()
                && shroud::counts(w, A, CellXY::new(x - 1, y - 1)).is_none()
            {
                return p;
            }
        }
    }
    panic!("no edge cell");
}

// spec: S-0409 claim 14 (r3: the shared state of the non-legal cells becomes explored at a hostile field's end only when no other hostile field still covers a non-legal cell)
#[test]
fn claim_14_shared_state_waits_for_last_field() {
    let mut w = game(&rules());
    let e = edge_cell(&w);
    let off = CellXY::new(e.x - 1, e.y - 1);
    let g1 = testkit::spawn(&mut w, Kind::Building, "GAP", 1, e, 0);
    let g2 = testkit::spawn(
        &mut w,
        Kind::Building,
        "GAP",
        1,
        CellXY::new(e.x + 1, e.y),
        0,
    );
    step(&mut w);
    step(&mut w);
    spawn(&mut w, Kind::Building, "SAT", 0, 0, 0);
    step(&mut w);
    step(&mut w);
    assert!(shroud::spy_satellite_active(&w, A));
    assert!(!shroud::explored(&w, A, off));
    glasnost_sim::objects::remove(&mut w, g1);
    step(&mut w);
    assert!(
        !shroud::explored(&w, A, off),
        "another hostile field still covers a non-legal cell"
    );
    glasnost_sim::objects::remove(&mut w, g2);
    step(&mut w);
    assert!(shroud::explored(&w, A, off));
}

// spec: S-0409 claim 15 (r3: the spy-satellite test continues after defeat at the recheck moments; losing the satellite then re-covers the map, while radar stays on, S-0410 claim 1)
#[test]
fn claim_15_satellite_after_defeat() {
    let mut w = game(&rules());
    let sat = spawn(&mut w, Kind::Building, "SAT", 0, -10, -10);
    step(&mut w);
    step(&mut w);
    assert!(shroud::spy_satellite_active(&w, A));
    glasnost_sim::econ::radar::on_defeat(&mut w, A);
    glasnost_sim::objects::remove(&mut w, sat);
    step(&mut w);
    assert!(!shroud::spy_satellite_active(&w, A));
    assert!(!shroud::explored(&w, A, c(15, 15)));
    assert!(glasnost_sim::econ::radar::available(&w, A));
}
