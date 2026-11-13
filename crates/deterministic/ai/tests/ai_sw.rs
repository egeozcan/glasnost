//! The own AI's use of the superweapon kinds other than the missile, the
//! storm and the paradrop (S-0512, S-0513; PLAN §4.7): an iron curtain on a
//! group of its armed vehicles that is fighting, a force shield on buildings
//! with a seen enemy at them, a spy plane at the enemy base, and the
//! teleport's two stages. The tests check the orders a brain gives on the
//! synthetic skirmish with invented superweapon types; the effects are
//! S-0513's and tested there.

use glasnost_ai::brain::Brain;
use glasnost_ai::{fixture, game::Game};
use glasnost_map::CellXY;
use glasnost_rules::sim::{BuildingType, Decimal};
use glasnost_rules::sim_econ::SuperweaponType;
use glasnost_sim::orders::Order;
use glasnost_sim::rules::Kind;
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, World};

/// The skirmish ruleset with one superweapon type per kind word and the
/// building that gives it (test data of our own): STRM, IRON, SHLD, SPYW,
/// CHRN (first stage, linked) and CHWP (second stage), the giving buildings
/// STRMB, IRONB, SHLDB, SPYB and CHRNB (both teleport stages), none of which a
/// house can build. Every charge time is 0.1 minutes and none depends on
/// power.
fn rules() -> std::sync::Arc<glasnost_sim::rules::SimRules> {
    let mut r = fixture::ruleset();
    let reg = |r: &mut glasnost_rules::schema::Ruleset, k: &str, n: &str| {
        r.registries.entry(k.into()).or_default().push(n.into());
    };
    let sw = |kind: &str, action: &str| SuperweaponType {
        kind: Some(kind.into()),
        action: Some(action.into()),
        recharge_time: Some(Decimal("0.1".into())),
        is_powered: Some(false),
        ..Default::default()
    };
    let mut chrn = sw("ChronoSphere", "ChronoSphere");
    chrn.pre_click = Some(true);
    let mut chwp = sw("ChronoWarp", "ChronoWarp");
    chwp.post_click = Some(true);
    chwp.pre_dependent = Some("ChronoSphere".into());
    for (n, t) in [
        ("STRM", sw("LightningStorm", "LightningStorm")),
        ("IRON", sw("IronCurtain", "IronCurtain")),
        ("SHLD", sw("ForceShield", "ForceShield")),
        ("SPYW", sw("SpyPlane", "SpyPlane")),
        ("CHRN", chrn),
        ("CHWP", chwp),
    ] {
        r.sim.superweapons.insert(n.into(), t);
        reg(&mut r, "superweapons", n);
    }
    let giver = |a: &str, b: Option<&str>| {
        let mut x = BuildingType {
            strength: Some(500),
            foundation: Some([1, 1]),
            ..Default::default()
        };
        x.econ.super_weapon = Some(a.into());
        x.econ.super_weapon2 = b.map(Into::into);
        x
    };
    for (n, b) in [
        ("STRMB", giver("STRM", None)),
        ("IRONB", giver("IRON", None)),
        ("SHLDB", giver("SHLD", None)),
        ("SPYB", giver("SPYW", None)),
        ("CHRNB", giver("CHRN", Some("CHWP"))),
    ] {
        r.sim.buildings.insert(n.into(), b);
        reg(&mut r, "buildings", n);
    }
    std::sync::Arc::new(glasnost_sim::rules::SimRules::new(&r, "synthetic-sw-kinds"))
}

fn free(w: &World, c: CellXY) -> bool {
    w.cells.usable(c) && w.objects.values().all(|o| o.pos.cell() != c)
}

/// A game on the synthetic map with house 0's yard standing, the giving
/// building `giver` placed for it beside the yard, and its superweapons
/// ready.
fn ready(giver: &str) -> (Game, CellXY) {
    ready_until(giver, None)
}

/// Like [`ready`], but with `until` named the loop ends in the step after
/// which that type's record is ready, before the house's own brain has
/// thought again (the brain fires a ready record at once, and its order is
/// pending in the world: a fixed wait of 400 ticks can end after that).
fn ready_until(giver: &str, until: Option<&str>) -> (Game, CellXY) {
    ready_in(giver, until, &fixture::map(), fixture::setup(5))
}

/// [`ready_until`] on map `map` with setup `setup`.
fn ready_in(
    giver: &str,
    until: Option<&str>,
    map: &glasnost_map::Map,
    setup: glasnost_sim::world::GameSetup,
) -> (Game, CellXY) {
    let mut g = Game::new(
        rules(),
        map,
        "synthetic-sw-kinds",
        fixture::catalogue(),
        setup,
    )
    .expect("the synthetic skirmish loads");
    while g.world.tick < 3000
        && !g.world.objects.values().any(|o| {
            o.owner == Some(HouseId(0))
                && o.kind == Kind::Building
                && g.world.rules().buildings[o.type_index as usize].name == "YARD"
        })
    {
        g.step();
    }
    let yard = g
        .world
        .objects
        .values()
        .find(|o| {
            o.owner == Some(HouseId(0))
                && o.kind == Kind::Building
                && g.world.rules().buildings[o.type_index as usize].name == "YARD"
        })
        .expect("the yard stands")
        .pos
        .cell();
    let at = (2..12)
        .map(|d| CellXY::new(yard.x - d, yard.y))
        .find(|&c| free(&g.world, c))
        .expect("a free cell for the giving building");
    testkit::spawn(&mut g.world, Kind::Building, giver, 0, at, 0);
    // Granted at the next recheck; ready after the charge time.
    let index = until.map(|n| {
        g.world
            .rules()
            .superweapons
            .types
            .iter()
            .position(|t| t.name == n)
            .expect("the type exists") as u32
    });
    for _ in 0..if index.is_some() { 1000 } else { 400 } {
        g.step();
        if let Some(i) = index
            && glasnost_sim::superweapons::charge::get(&g.world, HouseId(0), i)
                .is_some_and(|r| r.ready)
        {
            return (g, yard);
        }
    }
    (g, yard)
}

/// The cell `dx` east of the yard, `dy` south, as a block centre.
fn spot(yard: CellXY, dx: i32, dy: i32) -> CellXY {
    CellXY::new(yard.x + dx, yard.y + dy)
}

/// Three armed vehicles of house 0 in the block of `c`.
fn group(w: &mut World, c: CellXY) {
    for (dx, dy) in [(0, 0), (1, 0), (0, 1)] {
        testkit::spawn(
            w,
            Kind::Vehicle,
            "GUNR",
            0,
            CellXY::new(c.x + dx, c.y + dy),
            0,
        );
    }
}

fn fires(orders: &[Order], sw: &str, w: &World) -> Vec<CellXY> {
    let i = w
        .rules()
        .superweapons
        .types
        .iter()
        .position(|t| t.name == sw)
        .expect("the type exists") as u32;
    orders
        .iter()
        .filter_map(|o| match *o {
            Order::FireSuperweapon { sw, cell } if sw == i => Some(cell),
            _ => None,
        })
        .collect()
}

fn think(w: &World) -> Vec<Order> {
    let mut brain = Brain::with_plan(
        w,
        &fixture::map(),
        HouseId(0),
        glasnost_ai::brain::default_plan(),
    );
    brain.think(w)
}

// An iron curtain goes on a group of the house's armed vehicles with a
// seen enemy near; not on a quiet group, a group with its own infantry in
// the block (the curtain kills every reached infantry unit, S-0513 claim
// 23) or one with an enemy vehicle in the block (it would be protected
// too, claim 25). Enemy infantry in the block does not hold it back: it
// dies (claim 23; M7b AI round 2).
#[test]
fn curtains_a_fighting_group_of_vehicles() {
    let (g, yard) = ready("IRONB");
    let c = spot(yard, 10, 8);
    let walker = |w: &mut World, house, at: CellXY| {
        testkit::spawn(w, Kind::Infantry, "WALKER", house, at, 0)
    };

    let mut quiet = g.world.clone();
    group(&mut quiet, c);
    glasnost_sim::tick::step(&mut quiet);
    assert!(
        fires(&think(&quiet), "IRON", &quiet).is_empty(),
        "no fight, no curtain"
    );

    let mut fight = g.world.clone();
    group(&mut fight, c);
    walker(&mut fight, 1, spot(c, 6, 0));
    glasnost_sim::tick::step(&mut fight);
    assert_eq!(
        fires(&think(&fight), "IRON", &fight),
        vec![c],
        "the group is curtained"
    );

    let mut with_infantry = g.world.clone();
    group(&mut with_infantry, c);
    walker(&mut with_infantry, 1, spot(c, 6, 0));
    walker(&mut with_infantry, 0, spot(c, 1, 1));
    glasnost_sim::tick::step(&mut with_infantry);
    assert!(
        fires(&think(&with_infantry), "IRON", &with_infantry).is_empty(),
        "its own infantry would die"
    );

    let mut with_enemy_infantry = g.world.clone();
    group(&mut with_enemy_infantry, c);
    walker(&mut with_enemy_infantry, 1, spot(c, 1, 1));
    glasnost_sim::tick::step(&mut with_enemy_infantry);
    assert_eq!(
        fires(&think(&with_enemy_infantry), "IRON", &with_enemy_infantry),
        vec![c],
        "an enemy infantry unit in the block dies, so the curtain goes on"
    );

    let mut with_enemy_vehicle = g.world.clone();
    group(&mut with_enemy_vehicle, c);
    testkit::spawn(
        &mut with_enemy_vehicle,
        Kind::Vehicle,
        "GUNR",
        1,
        spot(c, 1, 1),
        0,
    );
    glasnost_sim::tick::step(&mut with_enemy_vehicle);
    assert!(
        fires(&think(&with_enemy_vehicle), "IRON", &with_enemy_vehicle).is_empty(),
        "an enemy vehicle in the block would be protected too"
    );
}

// Two armed vehicles about to fight are enough for the curtain (M7b AI
// round 2: three in one block was rare, and the curtain stood ready).
#[test]
fn curtains_a_fighting_pair_of_vehicles() {
    let (g, yard) = ready("IRONB");
    let c = spot(yard, 10, 8);
    let mut w = g.world.clone();
    for at in [c, spot(c, 1, 0)] {
        testkit::spawn(&mut w, Kind::Vehicle, "GUNR", 0, at, 0);
    }
    testkit::spawn(&mut w, Kind::Infantry, "WALKER", 1, spot(c, 6, 0), 0);
    glasnost_sim::tick::step(&mut w);
    assert_eq!(
        fires(&think(&w), "IRON", &w),
        vec![c],
        "the pair is curtained"
    );

    let mut one = g.world.clone();
    testkit::spawn(&mut one, Kind::Vehicle, "GUNR", 0, c, 0);
    testkit::spawn(&mut one, Kind::Infantry, "WALKER", 1, spot(c, 6, 0), 0);
    glasnost_sim::tick::step(&mut one);
    assert!(
        fires(&think(&one), "IRON", &one).is_empty(),
        "one vehicle is not worth the curtain"
    );
}

// A building of the house that a seen enemy is attacking is curtained on
// its reference cell (S-0513 claim 25: buildings are protected; M7b AI
// round 2), and not while no enemy attacks it.
#[test]
fn curtains_a_building_under_attack() {
    let (g, yard) = ready("IRONB");
    let b = spot(yard, 12, 12);
    let mut w = g.world.clone();
    let plant = testkit::spawn(&mut w, Kind::Building, "POWR", 0, b, 0);
    let e = testkit::spawn(&mut w, Kind::Infantry, "WALKER", 1, spot(b, 6, 0), 0);
    glasnost_sim::tick::step(&mut w);
    let mut quiet = w.clone();
    if let Some(o) = quiet.obj_mut(e) {
        o.target = None;
    }
    assert!(
        fires(&think(&quiet), "IRON", &quiet).is_empty(),
        "no attack on the building, no curtain"
    );
    if let Some(o) = w.obj_mut(e) {
        o.target = Some(glasnost_sim::world::Target::Object(plant));
    }
    let r = w.reference_point(plant).cell();
    assert_eq!(
        fires(&think(&w), "IRON", &w),
        vec![r],
        "the attacked building is curtained"
    );
}

// A force shield goes on buildings with a seen enemy at them.
#[test]
fn shields_buildings_with_an_enemy_at_them() {
    let (g, yard) = ready("SHLDB");
    let mut w = g.world.clone();
    glasnost_sim::tick::step(&mut w);
    assert!(
        fires(&think(&w), "SHLD", &w).is_empty(),
        "no enemy, no shield"
    );
    let b = spot(yard, 12, 12);
    testkit::spawn(&mut w, Kind::Building, "POWR", 0, b, 0);
    testkit::spawn(&mut w, Kind::Building, "PILL", 0, spot(b, 1, 1), 0);
    testkit::spawn(&mut w, Kind::Infantry, "WALKER", 1, spot(b, 6, 0), 0);
    glasnost_sim::tick::step(&mut w);
    let cells = fires(&think(&w), "SHLD", &w);
    assert_eq!(cells.len(), 1, "one shield order: {cells:?}");
    let c = cells[0];
    assert!(
        (c.x - b.x).abs() <= 1 && (c.y - b.y).abs() <= 1,
        "the shield is on the pair of buildings: {c:?} for {b:?}"
    );
}

// S-0513 claim 31 and F3 (M7b AI audit): the shield takes the buildings of the house within its radius of the target cell, so two buildings with a gap between them are shielded together.
#[test]
fn shields_buildings_with_a_gap_between_them() {
    let (g, yard) = ready("SHLDB");
    let mut w = g.world.clone();
    let b = spot(yard, 12, 12);
    testkit::spawn(&mut w, Kind::Building, "POWR", 0, b, 0);
    testkit::spawn(&mut w, Kind::Building, "PILL", 0, spot(b, 5, 0), 0);
    testkit::spawn(&mut w, Kind::Infantry, "WALKER", 1, spot(b, 8, 0), 0);
    glasnost_sim::tick::step(&mut w);
    let cells = fires(&think(&w), "SHLD", &w);
    assert_eq!(cells.len(), 1, "one shield order: {cells:?}");
}

// A ready spy plane goes at the enemy base.
#[test]
fn sends_the_spy_plane_at_the_enemy() {
    let (g, _) = ready_until("SPYB", Some("SPYW"));
    let mut w = g.world.clone();
    glasnost_sim::tick::step(&mut w);
    let cells = fires(&think(&w), "SPYW", &w);
    assert_eq!(cells.len(), 1, "one spy plane order: {cells:?}");
    let enemy = glasnost_ai::skirmish::start_cells(&fixture::map());
    assert!(
        enemy
            .iter()
            .any(|&s| (s.x - cells[0].x).abs() <= 2 && (s.y - cells[0].y).abs() <= 2),
        "at an enemy start: {cells:?} of {enemy:?}"
    );
}

// A pair of armed vehicles is enough for a teleport source (a person sends
// two tanks too); the iron curtain wants three and a fight.
#[test]
fn teleports_a_pair_of_vehicles() {
    let (g, yard) = ready("CHRNB");
    let c = spot(yard, 10, 8);
    let mut w = g.world.clone();
    for (dx, dy) in [(0, 0), (1, 0)] {
        testkit::spawn(
            &mut w,
            Kind::Vehicle,
            "GUNR",
            0,
            CellXY::new(c.x + dx, c.y + dy),
            0,
        );
    }
    glasnost_sim::tick::step(&mut w);
    assert_eq!(
        fires(&think(&w), "CHRN", &w),
        vec![c],
        "the first stage on the pair"
    );
}

// The teleport: the first stage on a group of armed vehicles, the second
// stage (only once a source is held) on a free block next to the enemy base.
#[test]
fn teleports_a_group_to_the_enemy_base() {
    let (g, yard) = ready("CHRNB");
    let c = spot(yard, 10, 8);
    let mut w = g.world.clone();
    glasnost_sim::tick::step(&mut w);
    assert!(
        fires(&think(&w), "CHRN", &w).is_empty() && fires(&think(&w), "CHWP", &w).is_empty(),
        "no group, no teleport"
    );
    group(&mut w, c);
    glasnost_sim::tick::step(&mut w);
    let orders = think(&w);
    assert_eq!(
        fires(&orders, "CHRN", &w),
        vec![c],
        "the first stage on the group"
    );
    assert!(
        fires(&orders, "CHWP", &w).is_empty(),
        "no second stage without a source"
    );
    for o in orders {
        glasnost_sim::orders::issue(&mut w, HouseId(0), o);
    }
    for _ in 0..200 {
        glasnost_sim::tick::step(&mut w);
    }
    let orders = think(&w);
    assert!(
        fires(&orders, "CHRN", &w).is_empty(),
        "the source is held: no new pick"
    );
    let dest = fires(&orders, "CHWP", &w);
    assert_eq!(dest.len(), 1, "the second stage: {dest:?}");
    let starts = glasnost_ai::skirmish::start_cells(&fixture::map());
    assert!(
        starts.iter().any(|&s| {
            let d = (s.x - dest[0].x).abs().max((s.y - dest[0].y).abs());
            (4..=9).contains(&d)
        }),
        "next to an enemy base: {dest:?} of {starts:?}"
    );
}

/// The teleport ready with a source picked on a group at `c` (the first
/// stage's order issued and 200 ticks run).
fn picked(c_at: (i32, i32)) -> (World, CellXY) {
    let (g, yard) = ready("CHRNB");
    let c = spot(yard, c_at.0, c_at.1);
    let mut w = g.world.clone();
    group(&mut w, c);
    glasnost_sim::tick::step(&mut w);
    let orders = think(&w);
    assert_eq!(fires(&orders, "CHRN", &w), vec![c], "the first stage");
    for o in orders {
        glasnost_sim::orders::issue(&mut w, HouseId(0), o);
    }
    for _ in 0..200 {
        glasnost_sim::tick::step(&mut w);
    }
    (w, c)
}

// design: own AI (#2) and S-0512 claim 10 (the second stage uses the first stage's last picked source and does not consume it; with the checked order, only an unconsumed source selection is accepted): once the second stage has been fired, the brain does not fire it again at the next tick while no new source is picked.
#[test]
fn fires_the_second_stage_once_per_pick() {
    let (mut w, _) = picked((10, 8));
    let orders = think(&w);
    assert_eq!(fires(&orders, "CHWP", &w).len(), 1, "the second stage");
    // The second stage consumes the source selection (S-0512 claim 10); the
    // first stage's record keeps the source cell it last picked.
    for i in 0..w.rules().superweapons.types.len() as u32 {
        if let Some(r) = glasnost_sim::superweapons::record_mut(&mut w, HouseId(0), i) {
            r.source_fresh = false;
        }
    }
    for _ in 0..3 {
        glasnost_sim::tick::step(&mut w);
        let again = think(&w);
        assert!(
            fires(&again, "CHWP", &w).is_empty(),
            "no second stage without a new pick at tick {}",
            w.tick
        );
    }
}

// design: own AI (#2) and S-0512 claim 10 (the player may pick another source): a picked source whose block no longer holds the group (it drove off) is not teleported; the first stage is fired again on the group where it stands now.
#[test]
fn picks_again_when_the_group_has_left_the_source() {
    let (mut w, c) = picked((10, 8));
    let moved = CellXY::new(c.x + 6, c.y + 2);
    let ids: Vec<_> = w
        .objects
        .values()
        .filter(|o| o.owner == Some(HouseId(0)) && o.kind == Kind::Vehicle)
        .filter(|o| (o.pos.cell().x - c.x).abs() <= 1 && (o.pos.cell().y - c.y).abs() <= 1)
        .map(|o| o.id)
        .collect();
    assert_eq!(ids.len(), 3, "the group stands at the source");
    for id in ids {
        glasnost_sim::objects::remove(&mut w, id);
    }
    group(&mut w, moved);
    glasnost_sim::tick::step(&mut w);
    let orders = think(&w);
    assert!(
        fires(&orders, "CHWP", &w).is_empty(),
        "no teleport from a source the group has left"
    );
    assert_eq!(fires(&orders, "CHRN", &w), vec![moved], "a new pick");
}

/// Cloaks unit `id` fully (S-0516 Terms) and checks house 0 does not
/// sense it.
fn cloak(w: &mut World, id: glasnost_sim::world::ObjectId) {
    w.obj_mut(id).unwrap().cloak.state = glasnost_sim::cloak::CloakState::Cloaked;
    assert!(glasnost_sim::cloak::hidden_from(w, id, Some(HouseId(0))));
}

// design: own AI (#38) and S-0513 claim 34 (the teleport leaves a protected object in place): a group of the house's vehicles under its iron curtain is not picked as a teleport source.
#[test]
fn does_not_teleport_a_curtained_group() {
    let (g, yard) = ready("CHRNB");
    let c = spot(yard, 10, 8);
    let mut w = g.world.clone();
    group(&mut w, c);
    glasnost_sim::tick::step(&mut w);
    let tick = w.tick;
    let ids: Vec<_> = w
        .objects
        .values()
        .filter(|o| o.owner == Some(HouseId(0)) && o.kind == Kind::Vehicle)
        .filter(|o| (o.pos.cell().x - c.x).abs() <= 1 && (o.pos.cell().y - c.y).abs() <= 1)
        .map(|o| o.id)
        .collect();
    assert_eq!(ids.len(), 3, "the group");
    for id in ids {
        w.obj_mut(id).unwrap().sw.protection = Some(glasnost_sim::superweapons::Protection {
            start: tick,
            duration: 300,
            kind: glasnost_sim::superweapons::ProtectKind::Curtain,
            tint: Default::default(),
        });
        assert!(glasnost_sim::superweapons::protected(&w, id));
    }
    let orders = think(&w);
    assert!(
        fires(&orders, "CHRN", &w).is_empty(),
        "no first stage on a curtained group: {orders:?}"
    );
}

// design: own AI (#38): the teleport's source block is held back only by another house's unit the house sees: a cloaked enemy tank or an enemy building beside the group does not change the pick.
#[test]
fn a_hidden_tank_or_a_building_beside_a_group_does_not_stop_the_teleport() {
    let (g, yard) = ready("CHRNB");
    let c = spot(yard, 10, 8);
    for what in ["tank", "building"] {
        let mut w = g.world.clone();
        group(&mut w, c);
        let beside = CellXY::new(c.x - 1, c.y - 1);
        assert!(free(&w, beside), "{what}: a free cell beside the group");
        if what == "tank" {
            let tank = testkit::spawn(&mut w, Kind::Vehicle, "GUNR", 1, beside, 0);
            glasnost_sim::tick::step(&mut w);
            cloak(&mut w, tank);
        } else {
            testkit::spawn(&mut w, Kind::Building, "PILL", 1, beside, 0);
            glasnost_sim::tick::step(&mut w);
        }
        assert_eq!(
            fires(&think(&w), "CHRN", &w),
            vec![c],
            "{what}: the first stage on the group"
        );
    }
}

// design: own AI (#38): the teleport's landing block is chosen by what the house sees: a cloaked enemy tank standing on it does not move it.
#[test]
fn a_hidden_tank_does_not_move_the_landing() {
    let (w, _) = picked((10, 8));
    let dest = fires(&think(&w), "CHWP", &w);
    assert_eq!(dest.len(), 1, "the second stage: {dest:?}");
    let mut w2 = w.clone();
    let tank = testkit::spawn(&mut w2, Kind::Vehicle, "GUNR", 1, dest[0], 0);
    cloak(&mut w2, tank);
    assert_eq!(
        fires(&think(&w2), "CHWP", &w2),
        dest,
        "the landing is where it was"
    );
}

/// An enemy scout (house 1) on a free cell near `c` that house 0 sees.
fn scout_near(w: &mut World, c: CellXY) -> glasnost_sim::world::ObjectId {
    let at = (3..12)
        .map(|d| CellXY::new(c.x + d, c.y))
        .find(|&x| free(w, x))
        .expect("a free cell for the scout");
    let p = glasnost_sim::world::Pos::centre(at, 0);
    glasnost_sim::econ::shroud::area_reveal(w, HouseId(0), p, 2);
    let scout = testkit::spawn(w, Kind::Infantry, "WALKER", 1, at, 0);
    glasnost_sim::tick::step(w);
    assert!(glasnost_ai::in_view(w, HouseId(0), scout), "the scout is seen");
    scout
}

// design: own AI (#31): the teleport lands next to the enemy base, not next to the nearest seen enemy unit: an enemy scout by the house's own base does not move the landing there.
#[test]
fn a_scout_at_home_does_not_pull_the_landing() {
    let (mut w, c) = picked((10, 8));
    let scout = scout_near(&mut w, c);
    let s = w.obj(scout).unwrap().pos.cell();
    let dest = fires(&think(&w), "CHWP", &w);
    assert_eq!(dest.len(), 1, "the second stage: {dest:?}");
    let d = dest[0];
    assert!(
        (d.x - s.x).abs().max((d.y - s.y).abs()) > 9,
        "the landing {d:?} is by the scout {s:?}"
    );
}

// design: own AI (#31): the spy plane goes at a cell the house does not see, as the psychic reveal does: an enemy scout seen by the house's base does not draw it.
#[test]
fn a_scout_at_home_does_not_draw_the_spy_plane() {
    let (g, yard) = ready_until("SPYB", Some("SPYW"));
    let mut w = g.world.clone();
    scout_near(&mut w, yard);
    let cells = fires(&think(&w), "SPYW", &w);
    assert_eq!(cells.len(), 1, "one spy plane order: {cells:?}");
    assert!(
        !glasnost_sim::econ::shroud::seen(&w, HouseId(0), cells[0]),
        "the spy plane goes at a seen cell: {cells:?}"
    );
}

/// The cells building `b` stands on (its foundation from its top-left cell).
fn foundation(w: &World, b: glasnost_sim::world::ObjectId) -> Vec<CellXY> {
    let o = w.obj(b).unwrap();
    glasnost_sim::econ::placement::foundation_cells(w, o.type_index, o.pos.cell())
}

// design: own AI (#30): the teleport's landing block holds none of the cells a seen building stands on, not only none of their top-left cells: a plant whose last cell is the block's corner moves the landing. The plant is the house's own, which it sees anywhere and which is no goal.
#[test]
fn a_building_reaching_into_the_landing_moves_it() {
    let (w, _) = picked((10, 8));
    let dest = fires(&think(&w), "CHWP", &w);
    assert_eq!(dest.len(), 1, "the second stage: {dest:?}");
    let c = dest[0];
    let mut w2 = w.clone();
    let plant = testkit::spawn(
        &mut w2,
        Kind::Building,
        "POWR",
        0,
        CellXY::new(c.x - 2, c.y - 2),
        0,
    );
    let cells = foundation(&w2, plant);
    assert!(
        cells.contains(&CellXY::new(c.x - 1, c.y - 1)),
        "the plant reaches into the block: {cells:?}"
    );
    let again = fires(&think(&w2), "CHWP", &w2);
    assert_eq!(again.len(), 1, "the second stage: {again:?}");
    let d = again[0];
    assert!(
        cells
            .iter()
            .all(|f| (f.x - d.x).abs() > 1 || (f.y - d.y).abs() > 1),
        "the landing {d:?} overlaps the plant {cells:?}"
    );
}

// design: own AI (#30): a storm is not aimed within STRIKE_RADIUS of any cell of the house's own building, its top-left cell or another.
#[test]
fn no_storm_beside_any_cell_of_an_own_building() {
    let (g, yard) = ready_until("STRMB", Some("STRM"));
    let b = spot(yard, 12, 12);
    // An enemy diagonal to the plant's last cell, beyond the radius (3) of
    // its top-left cell.
    let e = spot(b, 3, 3);
    let mut g = g;
    let p = glasnost_sim::world::Pos::centre(e, 0);
    glasnost_sim::econ::shroud::area_reveal(&mut g.world, HouseId(0), p, 4);
    let mut alone = g.world.clone();
    let walker = testkit::spawn(&mut alone, Kind::Infantry, "WALKER", 1, e, 0);
    glasnost_sim::tick::step(&mut alone);
    assert!(glasnost_ai::in_view(&alone, HouseId(0), walker), "the enemy is seen");
    assert_eq!(
        fires(&think(&alone), "STRM", &alone),
        vec![e],
        "the storm on the enemy"
    );
    let mut w = g.world.clone();
    let plant = testkit::spawn(&mut w, Kind::Building, "POWR", 0, b, 0);
    testkit::spawn(&mut w, Kind::Infantry, "WALKER", 1, e, 0);
    glasnost_sim::tick::step(&mut w);
    let cells = foundation(&w, plant);
    let near = |c: CellXY| {
        cells
            .iter()
            .any(|f| (f.x - c.x).pow(2) + (f.y - c.y).pow(2) <= 9)
    };
    let shots = fires(&think(&w), "STRM", &w);
    assert!(
        shots.iter().all(|&c| !near(c)),
        "a storm beside the plant {cells:?}: {shots:?}"
    );
}

/// The synthetic map with a third start (waypoint 2, north of the middle)
/// and three houses: Alpha (0) and Gamma (2, a house with no AI) on lobby
/// team 1, so allied (S-0412 claim 8), and Beta (1) on team 2.
fn allied_three() -> (glasnost_map::Map, glasnost_sim::world::GameSetup) {
    use glasnost_sim::econ::setup::{self, HouseValues, Player, Settings};
    let mut m = fixture::map();
    let c = fixture::at(0, -30);
    m.waypoints.insert(2, (c.x as i16, c.y as i16));
    let player = |name: &str, country: &str, start: u32, ai: bool, team: u8| Player {
        name: name.into(),
        country: country.into(),
        start: Some(start),
        ai,
        values: HouseValues {
            team: Some(team),
            ..Default::default()
        },
    };
    let s = setup::skirmish(
        &rules(),
        &[
            player("Alpha", "Alphaland", 0, true, 1),
            player("Beta", "Betaland", 1, true, 2),
            player("Gamma", "Alphaland", 2, false, 1),
        ],
        Settings::default(),
        0,
        5,
    );
    (m, s)
}

// design: own AI (#32): a storm is not aimed within STRIKE_RADIUS of an allied house's object either, as of the house's own.
#[test]
fn no_storm_beside_an_allys_unit() {
    let (map, setup) = allied_three();
    let (mut g, yard) = ready_in("STRMB", Some("STRM"), &map, setup);
    assert!(g.world.allied(Some(HouseId(0)), Some(HouseId(2))), "allied");
    let e = spot(yard, 15, 15);
    let p = glasnost_sim::world::Pos::centre(e, 0);
    glasnost_sim::econ::shroud::area_reveal(&mut g.world, HouseId(0), p, 4);
    let mut alone = g.world.clone();
    testkit::spawn(&mut alone, Kind::Infantry, "WALKER", 1, e, 0);
    glasnost_sim::tick::step(&mut alone);
    assert_eq!(
        fires(&think(&alone), "STRM", &alone),
        vec![e],
        "the storm on the enemy"
    );
    let mut w = g.world.clone();
    testkit::spawn(&mut w, Kind::Infantry, "WALKER", 1, e, 0);
    let a = spot(e, 1, 1);
    let ally = testkit::spawn(&mut w, Kind::Vehicle, "GUNR", 2, a, 0);
    glasnost_sim::tick::step(&mut w);
    let at = w.obj(ally).unwrap().pos.cell();
    let shots = fires(&think(&w), "STRM", &w);
    assert!(
        shots
            .iter()
            .all(|&c| (c.x - at.x).pow(2) + (c.y - at.y).pow(2) > 9),
        "a storm beside the ally's tank at {at:?}: {shots:?}"
    );
}
