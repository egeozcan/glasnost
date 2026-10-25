//! Claim tests for S-0216 (units placed after the game has started), on
//! synthetic data with invented type names and values.
//!
//! The placing step is the testkit's play-time harness
//! (`testkit::spawn_in_play`), which creates a unit (its creation draw,
//! S-0202 claim 10) and places it by `objects::place_in_play`, as the
//! measurement harness of scenarios PT1 to PT5 does. Draws are checked
//! against a cloned scenario stream. The ledger is
//! `docs/m8-claims/intake21-S-0216.md`.

use std::sync::Arc;

use glasnost_map::CellXY;
use glasnost_map::terrain::{TileCell, TileInfo, land};
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{LandRow, LandType, LocomotionClass, Locomotor, OverlayType, UnitType};
use glasnost_sim::cells::position;
use glasnost_sim::combat::acquire::{self, SearchContext, SearchKind};
use glasnost_sim::rng::Rng;
use glasnost_sim::rules::{Behaviour, Kind, SimRules};
use glasnost_sim::testkit::{self, mid};
use glasnost_sim::walk;
use glasnost_sim::world::{HouseId, ObjectId, Target, World};

const SIZE: i32 = 32;
/// The terrain code of a water cell (S-0010).
const WATER_CODE: u8 = 9;
/// The terrain code of a rock cell (S-0010).
const ROCK_CODE: u8 = 7;

fn cell(dx: i32, dy: i32) -> CellXY {
    mid(SIZE, dx, dy)
}

fn reg(r: &mut Ruleset, kind: &str, name: &str) {
    r.registries
        .entry(kind.into())
        .or_default()
        .push(name.into());
}

/// The combat ruleset with a vehicle type without a turret (TRUCK), a
/// visceroid-flagged vehicle type (BLOB, test type data as in PT3 and
/// PT5), a water row that no ground class may enter and an intact wall
/// overlay type (FENCE) that no type here crushes.
fn ruleset() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    reg(&mut r, "vehicles", "TRUCK");
    reg(&mut r, "vehicles", "BLOB");
    reg(&mut r, "overlays", "FENCE");
    let s = &mut r.sim;
    let truck = UnitType {
        strength: Some(200),
        speed: Some(6),
        rot: Some(5),
        locomotion_class: Some(LocomotionClass::Wheel),
        locomotor: Some(Locomotor::Drive),
        ..Default::default()
    };
    s.vehicles.insert("TRUCK".into(), truck.clone());
    s.vehicles.insert(
        "BLOB".into(),
        UnitType {
            small_visceroid: Some(true),
            ..truck
        },
    );
    s.land_types.insert(
        LandType::Water,
        LandRow {
            foot: Some(glasnost_rules::sim::Decimal("0".into())),
            track: Some(glasnost_rules::sim::Decimal("0".into())),
            wheel: Some(glasnost_rules::sim::Decimal("0".into())),
            ..LandRow::default()
        },
    );
    s.land_types.insert(LandType::Wall, LandRow::default());
    s.overlays.insert(
        "FENCE".into(),
        OverlayType {
            wall: Some(true),
            land: Some(LandType::Wall),
            ..Default::default()
        },
    );
    r
}

/// A game on an empty map whose catalogue adds a water tile (index 5)
/// and a rock tile (index 6). With `local`, the usable area is the map
/// less a 4-cell border.
fn game_of(r: &Ruleset, seed: u32, local: bool) -> World {
    let mut cat = (*testkit::catalogue()).clone();
    for code in [WATER_CODE, ROCK_CODE] {
        cat.tiles.push(TileInfo {
            width: 1,
            height: 1,
            has_image: true,
            cells: vec![Some(TileCell {
                terrain_code: code,
                slope: 0,
            })],
            variants: 0,
            anim: None,
            no_burrowing: false,
        });
        cat.allow_tiberium.push(false);
    }
    cat.roles.water = 5;
    let mut map = testkit::empty_map(SIZE);
    map.has_overlay_data = true;
    if local {
        let fp = glasnost_map::Footprint::new(SIZE, SIZE, [4, 4, SIZE - 8, SIZE - 8]);
        let mut cells = glasnost_map::CellGrid::new(fp);
        let all: Vec<CellXY> = cells.iter().map(|(c, _)| c).collect();
        for c in all {
            *cells.get_mut(c).unwrap() = glasnost_map::Cell {
                tile: 0,
                stream: true,
                ..glasnost_map::Cell::BLANK
            };
        }
        map.footprint = fp;
        map.cells = cells;
    }
    glasnost_sim::load::new_game(
        Arc::new(SimRules::new(r, "claims-s0216")),
        &map,
        Arc::new(cat),
        &testkit::setup(seed),
    )
    .expect("synthetic game loads")
}

fn game(seed: u32) -> World {
    game_of(&ruleset(), seed, false)
}

/// Gives cell `c` tile `tile` and recomputes it.
fn retile(w: &mut World, c: CellXY, tile: i32) {
    w.cells.grid.get_mut(c).unwrap().tile = tile;
    glasnost_sim::load::recalc_cells(w, &[c]);
    glasnost_sim::regions::cell_changed(w, c);
}

fn wet(w: &mut World, c: CellXY) {
    retile(w, c, 5);
    assert_eq!(w.cells.cell(c).unwrap().land, land::WATER);
}

fn rock(w: &mut World, c: CellXY) {
    retile(w, c, 6);
    assert_eq!(w.cells.cell(c).unwrap().land, land::ROCK);
}

fn fence(w: &mut World, c: CellXY) {
    w.cells.grid.get_mut(c).unwrap().overlay = 0;
    glasnost_sim::load::recalc_cells(w, &[c]);
    glasnost_sim::regions::cell_changed(w, c);
}

/// Places a unit at play time (no behaviour given); `None` when refused.
fn play(w: &mut World, kind: Kind, ty: &str, house: u8, c: CellXY, v: u8) -> Option<ObjectId> {
    testkit::spawn_in_play(w, kind, ty, house, c, v, None)
}

fn half(w: &World) -> bool {
    w.rules().quirks.ranged_draw_half_span
}

/// The standing position of infantry unit `id` in its cell.
fn standing(w: &World, id: ObjectId) -> u8 {
    let p = w.obj(id).unwrap().pos;
    let (u, v) = (p.x.rem_euclid(256), p.y.rem_euclid(256));
    [
        position::CENTRE,
        position::TOP,
        position::RIGHT,
        position::LEFT,
        position::BOTTOM,
    ]
    .into_iter()
    .find(|&k| walk::standing_point(k) == (u, v))
    .expect("at a standing position")
}

/// Row R*k* of S-0208 table O for a desired point of class centre.
fn row(k: i32) -> [u8; 3] {
    use position::{BOTTOM, LEFT, RIGHT};
    // spec: S-0208 table O (rows R0 to R3: k = 0 or 1 right first, 2 left first, 3 bottom first)
    match k {
        0 | 1 => [RIGHT, LEFT, BOTTOM],
        2 => [LEFT, BOTTOM, RIGHT],
        _ => [BOTTOM, RIGHT, LEFT],
    }
}

fn on_map_in(w: &World, id: ObjectId, c: CellXY) -> bool {
    w.obj(id).is_some_and(|o| o.on_map && o.pos.cell() == c)
}

// ---------------------------------------------------------------------------
// Claim 1: vehicle facings (PT1)

// spec: S-0216 claim 1 and Formulas (body and turret both exactly 256 × v facing units, set at once, the same in every seed; scenario PT1's facing values, including 37, 101 and 201)
#[test]
fn claim_1_vehicle_facings_are_256_v_in_every_seed() {
    const VALUES: [u8; 11] = [0, 32, 64, 96, 128, 160, 192, 224, 37, 101, 201];
    let mut seen: Vec<Vec<(u16, u16)>> = Vec::new();
    for seed in [3, 4, 5] {
        let mut w = game(seed);
        let t = w.tick;
        let mut row = Vec::new();
        for (i, &v) in VALUES.iter().enumerate() {
            for (j, ty) in ["CART", "TRUCK"].into_iter().enumerate() {
                let c = cell(-10 + 2 * i as i32, -6 + 6 * j as i32);
                let id = play(&mut w, Kind::Vehicle, ty, 0, c, v).expect("placed");
                let o = w.obj(id).unwrap();
                let want = 256 * u16::from(v); // spec: S-0216 Formulas (256 × v)
                assert_eq!(o.body.value(t), want, "{ty} v {v} body");
                assert_eq!(o.turret.value(t), want, "{ty} v {v} turret");
                assert!(!o.body.turning(t) && !o.turret.turning(t), "no turn");
                row.push((o.body.value(t), o.turret.value(t)));
            }
        }
        seen.push(row);
    }
    assert!(
        seen.windows(2).all(|p| p[0] == p[1]),
        "the same in every seed"
    );
}

// ---------------------------------------------------------------------------
// Claim 2: infantry standing positions (PT2)

// spec: S-0216 claim 2 and Formulas (an empty usable cell: one ranged draw R(0, 3) after the creation draw; right for 0 and 1, left for 2, bottom for 3; never centre or top; scenario PT2 part a)
#[test]
fn claim_2_empty_cells_stand_by_the_draw() {
    let mut counts = [0; 5];
    for seed in 1..=8 {
        let mut w = game(seed);
        for i in 0..16 {
            let c = cell(-12 + 4 * (i % 4), -12 + 4 * (i / 4));
            let mut probe: Rng = w.rng.clone();
            let id = play(&mut w, Kind::Infantry, "WALKER", 0, c, 0).expect("placed");
            probe.raw(); // spec: S-0202 claim 10 (the creation draw)
            let k = probe.ranged(0, 3, half(&w));
            assert_eq!(w.rng, probe, "creation draw, then one R(0, 3)");
            let p = standing(&w, id);
            assert_eq!(p, row(k)[0], "draw {k}");
            assert!(p != position::CENTRE && p != position::TOP);
            counts[usize::from(p)] += 1;
        }
    }
    // PT2 part a's bounds over the 128 placements.
    assert!(
        (46..=82).contains(&counts[usize::from(position::RIGHT)]),
        "{counts:?}"
    );
    assert!(
        (17..=49).contains(&counts[usize::from(position::LEFT)]),
        "{counts:?}"
    );
    assert!(
        (17..=49).contains(&counts[usize::from(position::BOTTOM)]),
        "{counts:?}"
    );
}

// spec: S-0216 claim 2 (units already standing in the cell count: the second unit takes the first free position of row Rk for its own draw k, never the first unit's; scenario PT2 part b)
#[test]
fn claim_2_occupied_cell_takes_the_first_free_position_of_its_row() {
    for seed in 1..=8 {
        let mut w = game(seed);
        let c = cell(0, 0);
        let first = play(&mut w, Kind::Infantry, "WALKER", 0, c, 0).expect("first");
        let mut probe: Rng = w.rng.clone();
        let second = play(&mut w, Kind::Infantry, "WALKER", 0, c, 0).expect("second");
        probe.raw();
        let k = probe.ranged(0, 3, half(&w));
        assert_eq!(w.rng, probe);
        let taken = standing(&w, first);
        let want = row(k).into_iter().find(|&p| p != taken).unwrap();
        assert_eq!(standing(&w, second), want, "seed {seed} draw {k}");
    }
}

// spec: S-0216 claim 2 and claim 3 (a cell whose ground record has its vehicle-present flag on: no position, no draw, the placement fails; scenario PT2 part d)
#[test]
fn claim_2_cell_with_a_vehicle_refuses_without_a_draw() {
    for seed in 1..=4 {
        let mut w = game(seed);
        let c = cell(0, 0);
        play(&mut w, Kind::Vehicle, "TRUCK", 0, c, 0).expect("vehicle");
        let mut probe: Rng = w.rng.clone();
        let before = w.objects.len();
        assert_eq!(play(&mut w, Kind::Infantry, "WALKER", 0, c, 0), None);
        probe.raw();
        assert_eq!(w.rng, probe, "the creation draw only");
        assert_eq!(w.objects.len(), before, "the harness discards the unit");
    }
}

// spec: S-0216 claim 2 and claim 3 (a full row: the draw is taken, then the placement fails; scenario PT2 part e)
#[test]
fn claim_2_full_row_refuses_after_one_draw() {
    for seed in 1..=4 {
        let mut w = game(seed);
        let c = cell(0, 0);
        let three: Vec<u8> = (0..3)
            .map(|_| {
                let id = play(&mut w, Kind::Infantry, "WALKER", 0, c, 0).expect("placed");
                standing(&w, id)
            })
            .collect();
        let mut sorted = three.clone();
        sorted.sort_unstable();
        assert_eq!(
            sorted,
            vec![position::RIGHT, position::LEFT, position::BOTTOM],
            "right, left and bottom whatever the draws"
        );
        let mut probe: Rng = w.rng.clone();
        assert_eq!(play(&mut w, Kind::Infantry, "WALKER", 0, c, 0), None);
        probe.raw();
        probe.ranged(0, 3, half(&w));
        assert_eq!(w.rng, probe, "creation draw and exactly one R(0, 3)");
    }
}

// spec: S-0216 claim 2 (outside the usable area: not established; our design, as S-0614 claim 20's placement: no position choice and no draw, the cell's centre, then the entry test of claim 5)
#[test]
fn claim_2_outside_the_usable_area_is_our_design() {
    let mut w = game_of(&ruleset(), 1, true);
    let c = (0..2 * SIZE)
        .flat_map(|x| (0..2 * SIZE).map(move |y| CellXY::new(x, y)))
        .find(|&c| w.cells.cell(c).is_some() && !w.cells.usable(c))
        .expect("a legal cell outside the usable area");
    let mut probe: Rng = w.rng.clone();
    let got = play(&mut w, Kind::Infantry, "WALKER", 0, c, 0);
    probe.raw();
    assert_eq!(w.rng, probe, "no standing-position draw");
    let id = got.expect("a clear entry outcome accepts it (our design)");
    assert_eq!(standing(&w, id), position::CENTRE);
    assert!(on_map_in(&w, id, c));
}

// ---------------------------------------------------------------------------
// Claim 3: draw count and order (PT3)

/// PT3's placing order: vehicle, infantry, vehicle (`third`), infantry.
/// Returns the expected stream after the four placements and the world's.
fn pt3(seed: u32, third: &str) -> (Rng, Rng) {
    let mut w = game(seed);
    let h = half(&w);
    let mut probe: Rng = w.rng.clone();
    let order = [
        (Kind::Vehicle, "TRUCK"),
        (Kind::Infantry, "WALKER"),
        (Kind::Vehicle, third),
        (Kind::Infantry, "WALKER"),
    ];
    for (i, (kind, ty)) in order.into_iter().enumerate() {
        play(&mut w, kind, ty, 0, cell(-6 + 4 * i as i32, 0), 0).expect("placed");
        probe.raw(); // spec: S-0202 claim 10 (the creation draw, at the unit's creation)
        match kind {
            // spec: S-0216 claim 3 (an infantry unit: the standing-position draw R(0, 3))
            Kind::Infantry => {
                probe.ranged(0, 3, h);
            }
            // spec: S-0216 claim 3 (a visceroid-flagged vehicle: one R(0, 29) once accepted)
            _ if ty == "BLOB" => {
                probe.ranged(0, 29, h);
            }
            // spec: S-0216 claim 3 (any other vehicle: no draw)
            _ => {}
        }
    }
    (probe, w.rng)
}

// spec: S-0216 claim 3 and State and ordering (per unit in placing order: the creation draw, then its placement draw if any; no other draw before the first update; scenario PT3)
#[test]
fn claim_3_draws_in_placing_order() {
    for seed in [2, 3, 4] {
        let (want, got) = pt3(seed, "TRUCK");
        assert_eq!(got, want, "plain vehicles, seed {seed}");
        let (want, got) = pt3(seed, "BLOB");
        assert_eq!(got, want, "visceroid in the third slot, seed {seed}");
    }
}

// ---------------------------------------------------------------------------
// Claim 4: behaviour before the first update (PT4)

/// The resting behaviour the placement fall-back queues for `ty` (the
/// ordinary placement, which only queues it, S-0215 claims 14 and 15).
fn resting(kind: Kind, ty: &str) -> Option<Behaviour> {
    let mut w = game(1);
    let ti = w.rules().index_of(kind, ty).unwrap();
    let id = glasnost_sim::objects::create(&mut w, kind, ti, Some(HouseId(0)));
    let p = glasnost_sim::world::Pos::centre(cell(0, 0), 0);
    glasnost_sim::objects::place(&mut w, id, p, 0);
    let o = w.obj(id).unwrap();
    assert_eq!(o.beh.current, None);
    o.beh.queued
}

// spec: S-0216 claim 4 (a unit ready to change when placed has the fall-back's resting behaviour current and nothing queued; a behaviour the placing step gives is queued by S-0215 claim 7 without a start, then made current and processed at the first update at which the unit is ready at the first per-tick start; scenario PT4)
#[test]
fn claim_4_resting_current_then_hunt_at_the_first_update() {
    for (kind, ty, c) in [
        (Kind::Vehicle, "CART", cell(-8, 0)),
        (Kind::Infantry, "WALKER", cell(-8, 4)),
    ] {
        let rest = resting(kind, ty);
        assert!(
            rest.is_some(),
            "{ty}: the fall-back chooses a resting behaviour"
        );
        let mut w = game(1);
        play(&mut w, Kind::Vehicle, "TRUCK", 1, cell(12, 12), 0).expect("far enemy");
        let id = play(&mut w, kind, ty, 0, c, 64).expect("placed");
        assert!(
            glasnost_sim::process::ready(&w, id),
            "{ty} ready at placement"
        );
        let o = w.obj(id).unwrap();
        assert_eq!(
            o.beh.current, rest,
            "{ty}: the resting behaviour is current"
        );
        assert_eq!(o.beh.queued, None, "{ty}: nothing queued");
        // The placing step's hunt order (S-0215 claim 7, no start).
        w.obj_mut(id).unwrap().beh.queue(Some(Behaviour::Hunt));
        let o = w.obj(id).unwrap();
        assert_eq!(
            o.beh.current, rest,
            "{ty}: hunt not current before the update"
        );
        assert_eq!(o.beh.queued, Some(Behaviour::Hunt));
        let t0 = w.tick;
        testkit::run(&mut w, 1);
        let o = w.obj(id).unwrap();
        assert_eq!(o.beh.current, Some(Behaviour::Hunt), "{ty}: hunt current");
        assert_eq!(
            o.beh.last.map(|l| l.0),
            Some(t0),
            "{ty}: hunt processed in its first update"
        );
    }
}

// spec: S-0216 claim 4 (the harness's queued behaviour: queued by S-0215 claim 7, not started, until the first update)
#[test]
fn claim_4_harness_queues_the_given_behaviour() {
    let mut w = game(1);
    let id = testkit::spawn_in_play(
        &mut w,
        Kind::Infantry,
        "WALKER",
        0,
        cell(0, 0),
        0,
        Some(Behaviour::Hunt),
    )
    .expect("placed");
    let o = w.obj(id).unwrap();
    assert_eq!(o.beh.queued, Some(Behaviour::Hunt));
    assert_ne!(o.beh.current, Some(Behaviour::Hunt));
}

// ---------------------------------------------------------------------------
// Claim 5: the entry test (PT5)

// spec: S-0216 claim 5 and claim 3 (a vehicle refused on water or on an intact wall it does not crush is not put on the map and takes no placement draw; a visceroid-flagged vehicle refused takes no R(0, 29); the controls on clear cells are placed with their draws; scenario PT5 parts a and b)
#[test]
fn claim_5_vehicles_refused_on_water_and_wall_without_a_draw() {
    for seed in [1, 2, 3] {
        let mut w = game(seed);
        let (water, wall) = (cell(0, 0), cell(4, 0));
        wet(&mut w, water);
        fence(&mut w, wall);
        for (ty, c) in [("TRUCK", water), ("TRUCK", wall), ("BLOB", water)] {
            let mut probe: Rng = w.rng.clone();
            assert_eq!(play(&mut w, Kind::Vehicle, ty, 0, c, 0), None, "{ty}");
            probe.raw();
            assert_eq!(w.rng, probe, "{ty}: the creation draw only");
            assert!(
                w.cells
                    .list(c, glasnost_sim::world::Layer::Ground)
                    .is_empty()
            );
        }
        // Controls.
        let mut probe: Rng = w.rng.clone();
        let blob = play(&mut w, Kind::Vehicle, "BLOB", 0, cell(-6, 0), 0).expect("control");
        probe.raw();
        probe.ranged(0, 29, half(&w)); // spec: S-0216 claim 3 (R(0, 29) once accepted)
        assert_eq!(w.rng, probe, "the visceroid control takes its draw");
        assert!(on_map_in(&w, blob, cell(-6, 0)));
        let truck = play(&mut w, Kind::Vehicle, "TRUCK", 0, cell(-8, 0), 0).expect("control");
        assert!(on_map_in(&w, truck, cell(-8, 0)));
    }
}

// spec: S-0216 claim 5 and claim 3 (an infantry unit on an empty water cell, or on a flat rock cell its movement class cannot enter: one R(0, 3) is taken and kept, then the entry test refuses; scenario PT5 parts c and d)
#[test]
fn claim_5_infantry_refused_on_water_and_rock_after_its_draw() {
    for seed in [1, 2, 3] {
        let mut w = game(seed);
        let (water, stone) = (cell(0, 0), cell(4, 0));
        wet(&mut w, water);
        rock(&mut w, stone);
        for c in [water, stone] {
            let mut probe: Rng = w.rng.clone();
            assert_eq!(play(&mut w, Kind::Infantry, "WALKER", 0, c, 0), None);
            probe.raw();
            probe.ranged(0, 3, half(&w));
            assert_eq!(w.rng, probe, "creation draw and one R(0, 3)");
            assert!(
                w.cells
                    .list(c, glasnost_sim::world::Layer::Ground)
                    .is_empty()
            );
        }
        let control = play(&mut w, Kind::Infantry, "WALKER", 0, cell(-6, 0), 0).expect("control");
        assert!(on_map_in(&w, control, cell(-6, 0)));
    }
}

// ---------------------------------------------------------------------------
// Claim 6: units on cliff cells of no reach region (Q-1713)

/// A game whose column at `cell(2, _)` is rock: a cell of no reach region
/// for every zone (as in S-0309's tests of claim 7). With `computer`, both
/// houses are test-only computer-controlled player houses, as the arena's
/// (S-0215 and S-0309 Terms).
fn cliff_game(computer: bool) -> World {
    let mut w = game(11);
    for y in -15..15 {
        rock(&mut w, cell(2, y));
    }
    if computer {
        for h in w.houses.iter_mut().take(2) {
            h.human = false;
        }
    }
    w
}

/// An infantry unit standing on the rock cell `cell(2, 0)` (set-up's
/// placement: occupancy and the entry test skipped, as the arena's interim
/// does), hunting, and an enemy vehicle on open ground within its weapon's
/// range (4 cells), of type `enemy`.
fn cliff_pair(w: &mut World, enemy: &str) -> (ObjectId, ObjectId) {
    let stale = w.rules().quirks.stale_suspended_revert; // spec: S-0215 Edge cases (quirks.stale_suspended_revert)
    let a = testkit::spawn(w, Kind::Infantry, "WALKER", 0, cell(2, 0), 0);
    let e = testkit::spawn(w, Kind::Vehicle, enemy, 1, cell(0, 0), 0);
    let t = w.tick;
    w.obj_mut(a).unwrap().beh.force(Behaviour::Hunt, t, stale);
    (a, e)
}

// spec: S-0216 claim 6, question 2 (a hunting infantry unit on a cell of no reach region does not make an enemy on open ground its target because it comes within its weapon's range: its hunt search does not consider it, S-0309 claim 7; the limited-scan mark is not an infantry unit's, S-0309 claim 22)
#[test]
fn claim_6_cliff_hunter_ignores_an_enemy_in_range() {
    let mut w = cliff_game(true);
    // An unarmed enemy, so that no hit starts a retaliation.
    let (a, _) = cliff_pair(&mut w, "TRUCK");
    assert_eq!(
        acquire::search(&mut w, a, SearchKind::MapWide, SearchContext::Hunt),
        None
    );
    testkit::run(&mut w, 120);
    let o = w.obj(a).unwrap();
    assert_eq!(o.target, None, "no target in 120 ticks");
    assert!(!o.combat.limited_scan);
}

// spec: S-0216 claim 6, questions 2 and 3 (retaliation has no reach-region test: for a computer-controlled house, a hunter with no target makes its attacker its target after a hit, from a cell of no reach region or against one; S-0309 claim 18, S-0215 claim 18)
#[test]
fn claim_6_computer_hunters_retaliate_across_the_cliff() {
    let mut w = cliff_game(true);
    let (a, e) = cliff_pair(&mut w, "CART");
    acquire::retaliate(&mut w, a, e);
    assert_eq!(
        w.obj(a).unwrap().target,
        Some(Target::Object(e)),
        "from the cliff"
    );
    let mut w = cliff_game(true);
    let (a, e) = cliff_pair(&mut w, "CART");
    let stale = w.rules().quirks.stale_suspended_revert; // spec: S-0215 Edge cases (quirks.stale_suspended_revert)
    let t = w.tick;
    w.obj_mut(e).unwrap().beh.force(Behaviour::Hunt, t, stale);
    assert_eq!(
        acquire::search(&mut w, e, SearchKind::MapWide, SearchContext::Hunt),
        None,
        "the open-ground hunter's search rejects the cliff unit (S-0309 claim 7)"
    );
    acquire::retaliate(&mut w, e, a);
    assert_eq!(
        w.obj(e).unwrap().target,
        Some(Target::Object(a)),
        "against the cliff"
    );
}

// spec: S-0216 claim 6, question 2, contrast (a hunting unit of a human-controlled player house does not retaliate while hunt is current, unless the player return-fire setting is on; S-0215 claim 18)
#[test]
fn claim_6_player_hunter_does_not_retaliate() {
    let mut w = cliff_game(false);
    let (a, e) = cliff_pair(&mut w, "CART");
    acquire::retaliate(&mut w, a, e);
    assert_eq!(w.obj(a).unwrap().target, None);
}
