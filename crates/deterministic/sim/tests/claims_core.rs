//! Claim tests for the simulation core: game ticks (S-0201), random streams
//! (S-0202), the tick order (S-0203), coordinates (S-0204) and number
//! domains (S-0205), on synthetic data.

use std::sync::Arc;

use glasnost_map::{CellXY, Structure, TerrainObject};
use glasnost_num::geom::{delta, n_step, root_floor};
use glasnost_num::{F64, Rounding};
use glasnost_rules::sim::{BuildingType, TerrainObjectType};
use glasnost_sim::load::{new_game, sandbox_setup};
use glasnost_sim::orders::{self, Order};
use glasnost_sim::regions::STEPS;
use glasnost_sim::rng::Rng;
use glasnost_sim::rules::{Behaviour, Kind, SimRules};
use glasnost_sim::save;
use glasnost_sim::testkit;
use glasnost_sim::tick;
use glasnost_sim::update::UpdateOrder;
use glasnost_sim::world::{HouseId, ObjectId, Pos, Target, World};

fn to_tick(w: &mut World, t: i32) {
    while w.tick < t {
        tick::step(w);
    }
}

fn mv(actor: ObjectId, c: CellXY) -> Order {
    Order::Behaviour {
        actor,
        behaviour: Behaviour::Move,
        attack_move: false,
        target: None,
        dest: Some(Target::Cell(c)),
    }
}

/// The testkit scenario with a 2 × 2 building and a terrain object added.
fn world_with_building() -> (World, ObjectId, ObjectId) {
    let mut r = testkit::ruleset();
    r.registries
        .insert("buildings".into(), vec!["DEPOT".into()]);
    r.registries
        .insert("terrain_objects".into(), vec!["TREE".into()]);
    r.sim.buildings.insert(
        "DEPOT".into(),
        BuildingType {
            strength: Some(800),
            foundation: Some([2, 2]),
            ..Default::default()
        },
    );
    r.sim
        .terrain_objects
        .insert("TREE".into(), TerrainObjectType::default());
    let rules = Arc::new(SimRules::new(&r, "synthetic+depot"));
    let mut map = testkit::map();
    map.objects.structures.push(Structure {
        placed: glasnost_map::objects::Placed {
            key: b"300".to_vec(),
            owner: b"Alpha".to_vec(),
            type_name: b"DEPOT".to_vec(),
            health: 256,
            cell: testkit::cell(-10, 6),
            facing: 0,
            raw: Vec::new(),
        },
    });
    map.objects.terrain.push(TerrainObject {
        cell: testkit::cell(10, 10),
        type_name: b"TREE".to_vec(),
    });
    let w = new_game(rules, &map, testkit::catalogue(), &testkit::setup(9)).unwrap();
    let b = w
        .objects
        .values()
        .find(|o| o.kind == Kind::Building)
        .unwrap()
        .id;
    let t = w
        .objects
        .values()
        .find(|o| o.kind == Kind::Terrain)
        .unwrap()
        .id;
    (w, b, t)
}

// ---------------------------------------------------------------------------
// S-0201

// spec: S-0201 claim 1 (the tick number is 0 at game start and grows by exactly 1 per tick)
#[test]
fn s0201_tick_numbers() {
    let mut w = testkit::new_game(1);
    assert_eq!(w.tick, 0);
    for n in 1..=5 {
        tick::step(&mut w);
        assert_eq!(w.tick, n);
    }
}

// spec: S-0201 claim 7 (a skirmish starts at the lobby's index; the rules' default when absent is 6)
#[test]
fn s0201_lobby_speed() {
    let r = testkit::rules();
    let (setup, _) = sandbox_setup(&testkit::map(), &r, 1);
    assert_eq!(setup.game_speed, 6);
    let mut rs = testkit::ruleset();
    rs.sim.general.lobby_game_speed = Some(2);
    let r2 = SimRules::new(&rs, "x");
    assert_eq!(sandbox_setup(&testkit::map(), &r2, 1).0.game_speed, 2);
}

// spec: S-0201 claim 14 (orders are blocked during ticks 0 to 3; the first order is issued in tick 4)
#[test]
fn s0201_order_block() {
    let mut w = testkit::new_game(1);
    let id = testkit::house_objects(&w, 0)[0];
    for t in 0..4 {
        assert_eq!(w.tick, t);
        assert!(!orders::issue(
            &mut w,
            HouseId(0),
            mv(id, testkit::cell(0, 4))
        ));
        tick::step(&mut w);
    }
    assert!(orders::issue(
        &mut w,
        HouseId(0),
        mv(id, testkit::cell(0, 4))
    ));
}

// ---------------------------------------------------------------------------
// S-0202

// spec: S-0202 claim 1 (the 250 most recent values determine every later raw value; same seed, same sequence)
#[test]
fn s0202_state_determines_the_future() {
    let mut a = Rng::new(42);
    for _ in 0..1000 {
        a.raw();
    }
    let mut b = a.clone();
    for _ in 0..1000 {
        assert_eq!(a.raw(), b.raw());
    }
    let mut c = Rng::new(42);
    let mut d = Rng::new(42);
    assert_eq!(
        (0..300).map(|_| c.raw()).collect::<Vec<_>>(),
        (0..300).map(|_| d.raw()).collect::<Vec<_>>()
    );
}

// spec: S-0202 claim 3 (seeding discards all previous state; the next raw value is x₀)
#[test]
fn s0202_seeding_discards_state() {
    let mut fresh = Rng::new(7);
    let x0 = fresh.raw();
    let mut used = Rng::new(99);
    for _ in 0..777 {
        used.raw();
    }
    used = Rng::new(7);
    assert_eq!(used.raw(), x0);
    assert_eq!(used.taken(), 1);
}

// spec: S-0202 claim 6 (derived forms: percent draw R(0, 99), unit value below 1, list pick raw mod n)
#[test]
fn s0202_derived_forms() {
    let mut a = Rng::new(5);
    let mut b = Rng::new(5);
    for _ in 0..2000 {
        let p = a.percent();
        assert!((0..=99).contains(&p));
        assert_eq!(p, b.ranged(0, 99, true));
    }
    let mut c = Rng::new(6);
    let one = F64::from_i64(1, Rounding::TowardZero);
    for _ in 0..2000 {
        assert!(c.unit(Rounding::TowardZero).lt(one));
    }
    let mut d = Rng::new(8);
    let mut e = Rng::new(8);
    for _ in 0..100 {
        assert_eq!(d.list_pick(7), e.raw() % 7);
    }
}

// spec: S-0202 claim 8 (the scenario stream is seeded with the setup's seed before the map is read; our engine takes the seed from the setup)
#[test]
fn s0202_seeding_at_game_start() {
    let a = testkit::new_game(11);
    let b = testkit::new_game(11);
    let c = testkit::new_game(12);
    assert_eq!(save::state_hash(&a), save::state_hash(&b));
    assert_ne!(save::state_hash(&a), save::state_hash(&c));
    // The load takes its draws from a stream seeded with the setup's seed.
    let mut fresh = Rng::new(11);
    let mut used = a.rng.clone();
    let taken = used.taken();
    for _ in 0..taken {
        fresh.raw();
    }
    assert_eq!(fresh.raw(), used.raw());
}

// spec: S-0202 claim 9 (a saved game keeps the scenario stream's full state)
#[test]
fn s0202_saves_keep_the_stream() {
    let mut w = testkit::new_game(13);
    to_tick(&mut w, 30);
    let l = save::load(&save::save(&w), testkit::rules(), testkit::catalogue()).unwrap();
    let mut a = w.rng.clone();
    let mut b = l.rng.clone();
    assert_eq!(a.taken(), b.taken());
    for _ in 0..500 {
        assert_eq!(a.raw(), b.raw());
    }
}

// ---------------------------------------------------------------------------
// S-0203

// spec: S-0203 claim 1 (orders, row 8, come after the turn pass; then the tick number grows)
// spec: S-0203 claim 7 (an order issued in tick N takes effect after that tick's turn pass)
#[test]
fn s0203_orders_after_the_turn_pass() {
    let mut w = testkit::new_game(1);
    to_tick(&mut w, 4);
    let id = testkit::house_objects(&w, 0)[0];
    let before = w.obj(id).unwrap().beh.clone();
    assert!(orders::issue(
        &mut w,
        HouseId(0),
        mv(id, testkit::cell(0, 6))
    ));
    tick::step(&mut w);
    let o = w.obj(id).unwrap();
    // Applied at the end of tick 4: queued, not yet current, and nothing moved in tick 4.
    assert_eq!(o.beh.queued, Some(Behaviour::Move));
    assert_eq!(o.beh.current, before.current);
    assert_eq!(o.dest, Some(Target::Cell(testkit::cell(0, 6))));
    // In tick 5 the actor's per-tick start makes it current (S-0215 claim 12).
    tick::step(&mut w);
    assert_eq!(w.obj(id).unwrap().beh.current, Some(Behaviour::Move));
}

// spec: S-0203 claim 2 (terrain objects whose type spawns no resources take no turns)
// spec: S-0203 claim 3 (map objects enter play kind by kind: vehicles, aircraft, infantry, buildings)
#[test]
fn s0203_update_order_of_map_objects() {
    let (w, building, tree) = world_with_building();
    assert!(!w.update.contains(tree));
    assert!(w.update.contains(building));
    let rank = |k: Kind| match k {
        Kind::Vehicle => 0,
        Kind::Aircraft => 1,
        Kind::Infantry => 2,
        Kind::Building => 3,
        Kind::Terrain => 4,
    };
    let kinds: Vec<u8> = w
        .update
        .order
        .iter()
        .map(|&id| rank(w.obj(id).unwrap().kind))
        .collect();
    assert!(kinds.windows(2).all(|p| p[0] <= p[1]), "{kinds:?}");
    assert_eq!(*kinds.last().unwrap(), 3);
}

// spec: S-0203 claim 3 (appending: an object that enters again goes to the end)
// spec: S-0203 claim 4 (an object entering play during the turn pass takes a turn later in that pass)
// spec: S-0203 claim 11 (an object that took its turn and enters again takes a second turn)
#[test]
fn s0203_entering_during_the_pass() {
    let mut u = UpdateOrder::default();
    let (a, b, c) = (ObjectId(1), ObjectId(2), ObjectId(3));
    u.enter(a);
    u.enter(b);
    u.begin_pass();
    let mut turns = Vec::new();
    while let Some(id) = u.next_turn() {
        turns.push(id);
        if id == a && !turns[..turns.len() - 1].contains(&a) {
            u.enter(c);
            // a leaves and enters again: its second turn comes at the end.
            u.leave(a, false);
            u.enter(a);
        }
    }
    u.end_pass();
    assert_eq!(turns, vec![a, b, c, a]);
    assert_eq!(u.order, vec![b, c, a]);
}

// spec: S-0203 claim 9 (timings run on the tick number, not on turns: a delay runs out by ticks)
#[test]
fn s0203_timings_on_tick_numbers() {
    let mut b = glasnost_sim::behaviour::Behaviours::default();
    b.processed(100, 14);
    // Whatever turns were missed in between, the timer has run out at tick 114.
    assert!(!b.due(113));
    assert!(b.due(114));
    assert!(b.due(500));
}

// ---------------------------------------------------------------------------
// S-0204

// spec: S-0204 claim 1 (positions are integer leptons)
// spec: S-0204 claim 2 (cell = (trunc(x / 256), trunc(y / 256)))
// spec: S-0204 claim 3 (centre = (256x + 128, 256y + 128))
#[test]
fn s0204_positions_and_cells() {
    assert_eq!(Pos::new(511, 256, 7).cell(), CellXY::new(1, 1));
    assert_eq!(Pos::new(255, 0, 0).cell(), CellXY::new(0, 0));
    assert_eq!(Pos::new(-1, -255, 0).cell(), CellXY::new(0, 0));
    assert_eq!(Pos::centre(CellXY::new(3, 4), 9), Pos::new(896, 1152, 9));
}

// spec: S-0204 claim 4 (a building is measured from the centre of its foundation: + 128 × (W − 1, H − 1))
#[test]
fn s0204_building_reference_point() {
    let (w, building, _) = world_with_building();
    let o = w.obj(building).unwrap();
    assert_eq!(o.foundation, (2, 2));
    let r = w.reference_point(building);
    assert_eq!((r.x - o.pos.x, r.y - o.pos.y), (128, 128));
    let unit = testkit::house_objects(&w, 0)[0];
    assert_eq!(w.reference_point(unit), w.obj(unit).unwrap().pos);
}

// spec: S-0204 claim 6 (distance = ⌊r(n)⌋ with the table square root; exact for powers of 4, one short above 129 leptons)
#[test]
fn s0204_distance() {
    assert_eq!(root_floor(512 * 512, true, Rounding::TowardZero), 512);
    assert_eq!(root_floor(128 * 128, true, Rounding::TowardZero), 128);
    assert_eq!(root_floor(16641, true, Rounding::TowardZero), 128);
    // The table root is never above the exact root, and at most one below it under 23 173 leptons.
    for n in [3_u64, 99, 16_642, 1_000_003, 536_000_000] {
        let exact = glasnost_num::geom::isqrt(u128::from(n)) as u64;
        let r = root_floor(n, true, Rounding::TowardZero);
        assert!(r <= exact && r + 1 >= exact, "{n}");
    }
}

// spec: S-0204 claim 9 (a step in 8-direction d: 0 is map north, clockwise; its facing is 8192 × d)
#[test]
fn s0204_eight_directions() {
    assert_eq!(STEPS[0], (0, -1));
    assert_eq!(STEPS[2], (1, 0));
    assert_eq!(STEPS[4], (0, 1));
    assert_eq!(STEPS[6], (-1, 0));
    for d in 0..8u16 {
        assert_eq!(glasnost_num::geom::eight_direction(d * 8192), d as u8);
    }
}

// spec: S-0204 claim 10 (coarse facings round to the nearest step, a half step up, with wrap-around)
#[test]
fn s0204_coarse_facings() {
    assert_eq!(n_step(4095, 8), 0);
    assert_eq!(n_step(4096, 8), 1);
    assert_eq!(n_step(65535, 8), 0);
    assert_eq!(n_step(1024, 32), 1);
    assert_eq!(n_step(1023, 32), 0);
}

// spec: S-0204 claim 11 (the signed difference takes the shorter way; half a turn is −32768)
#[test]
fn s0204_signed_difference() {
    assert_eq!(delta(0, 100), 100);
    assert_eq!(delta(100, 0), -100);
    assert_eq!(delta(65000, 500), 1036);
    assert_eq!(delta(0, 32768), -32768);
    assert_eq!(delta(32768, 0), -32768);
}

// ---------------------------------------------------------------------------
// S-0205

// spec: S-0205 claim 1 (integer results outside the range wrap; no saturation)
#[test]
fn s0205_integers_wrap() {
    let mut w = testkit::new_game(1);
    w.tick = i32::MAX;
    tick::step(&mut w);
    assert_eq!(w.tick, i32::MIN);
}

// spec: S-0205 claim 4 (a single widens exactly to a double; comparisons are exact)
#[test]
fn s0205_precision() {
    let rz = Rounding::TowardZero;
    let tenth =
        glasnost_num::decimal::parse("0.1", glasnost_num::decimal::Field::Single, rz).unwrap();
    let back = tenth.to_f32(rz).to_f64();
    assert_eq!(back, tenth);
    let d = glasnost_num::decimal::nearest_double("0.1");
    assert!(!tenth.eq_num(d));
    assert!(tenth.lt(d) || tenth.gt(d));
}

// spec: S-0205 claim 6 (roots and angles come from the tables, not a library)
#[test]
fn s0205_tables_not_library() {
    // The table root and the table direction differ from exact mathematics where the tables do.
    // 16641 = 129²: the table root is one short there (S-0204 claim 5).
    assert_eq!(root_floor(16641, true, Rounding::TowardZero), 128);
    assert_eq!(root_floor(16641, false, Rounding::TowardZero), 129);
    assert_eq!(glasnost_num::geom::direction(9, -97, true), 966);
}

// spec: S-0205 claim 8 (our engine gives every intermediate result the range of a double: overflow goes to the largest double or infinity)
#[test]
fn s0205_double_range() {
    let big = F64::from_i64(i64::MAX, Rounding::TowardZero);
    let mut x = big;
    for _ in 0..20 {
        x = x.mul(big, Rounding::TowardZero);
    }
    // Under round-toward-zero an overflow stops at the largest finite double.
    assert_eq!(x.0, 0x7fef_ffff_ffff_ffff);
    let mut y = F64::from_i64(i64::MAX, Rounding::Nearest);
    for _ in 0..20 {
        y = y.mul(big, Rounding::Nearest);
    }
    assert_eq!(y.0, 0x7ff0_0000_0000_0000);
}
