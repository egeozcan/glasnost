//! The own AI and recycling buildings (S-0603, PLAN §4.7): controlled
//! enemy vehicles are recycled; controlled infantry go to an absorbing
//! building with room first; badly damaged own units at home are recycled;
//! surplus infantry are recycled only while money is short; harvesters,
//! engineers and controllers are kept; the house builds one grinder once it
//! has a use for it. The
//! tests use invented types on the synthetic skirmish.

use std::sync::Arc;

use glasnost_ai::brain::Brain;
use glasnost_ai::fixture;
use glasnost_ai::game::Game;
use glasnost_ai::grinder::LOW_MONEY;
use glasnost_map::CellXY;
use glasnost_rules::sim::{Decimal, WarheadType, WeaponType};
use glasnost_sim::orders::Order;
use glasnost_sim::rules::{Behaviour, Kind, SimRules};
use glasnost_sim::world::{HouseId, ObjectId, Target, World};
use glasnost_sim::{mind, testkit};

fn d(s: &str) -> Decimal {
    Decimal(s.into())
}

/// The fixture's rules with GRND (a 2 × 2 recycling building, cost 400,
/// tech level `grnd_tech`), BIOR (a 2 × 2 infantry absorber for two), the
/// engineer ENGI and the controllers CLONE (infantry) and TANKC (a vehicle)
/// with the takeover weapon RAY.
fn rules(grnd_tech: i32) -> Arc<SimRules> {
    let mut r = fixture::ruleset();
    let reg = |r: &mut glasnost_rules::schema::Ruleset, k: &str, n: &str| {
        r.registries.entry(k.into()).or_default().push(n.into());
    };
    let mut g = fixture::building(2, 2, 400, 0);
    g.grinding = Some(true);
    g.tech_level = Some(grnd_tech);
    r.sim.buildings.insert("GRND".into(), g);
    reg(&mut r, "buildings", "GRND");
    let mut b = fixture::building(2, 2, 600, 50);
    b.infantry_absorb = Some(true);
    b.passengers = Some(2);
    b.size_limit = Some(d("1"));
    b.tech_level = Some(99);
    r.sim.buildings.insert("BIOR".into(), b);
    reg(&mut r, "buildings", "BIOR");
    let mut e = r.sim.infantry["WALKER"].clone();
    e.engineer = Some(true);
    e.tech_level = Some(99);
    r.sim.infantry.insert("ENGI".into(), e);
    reg(&mut r, "infantry", "ENGI");
    r.sim.warheads.insert(
        "MIND".into(),
        WarheadType {
            mind_control: Some(true),
            verses: Some(vec![d("1"); 11]),
            ..Default::default()
        },
    );
    r.sim.weapons.insert(
        "RAY".into(),
        WeaponType {
            damage: Some(4),
            rof: Some(40),
            range: Some(d("2")),
            speed: Some(100),
            projectile: Some("Bullet".into()),
            warhead: Some("MIND".into()),
            ..Default::default()
        },
    );
    let mut c = r.sim.infantry["WALKER"].clone();
    c.primary = Some("RAY".into());
    c.tech_level = Some(99);
    r.sim.infantry.insert("CLONE".into(), c);
    reg(&mut r, "infantry", "CLONE");
    Arc::new(SimRules::new(&r, "synthetic-grinder"))
}

fn game(grnd_tech: i32) -> Game {
    let mut g = Game::new(
        rules(grnd_tech),
        &fixture::map(),
        "synthetic",
        fixture::catalogue(),
        fixture::setup(5),
    )
    .expect("the skirmish loads");
    while g.world.tick <= g.world.orders_blocked_through || g.world.tick % 15 != 0 {
        g.step();
    }
    g
}

fn home(w: &World) -> CellXY {
    w.objects
        .values()
        .find(|o| o.owner == Some(HouseId(0)) && o.on_map)
        .map(|o| o.pos.cell())
        .unwrap()
}

fn spawn(w: &mut World, kind: Kind, name: &str, house: u8, dx: i32, dy: i32) -> ObjectId {
    let h = home(w);
    testkit::spawn(w, kind, name, house, CellXY::new(h.x + dx, h.y + dy), 0)
}

/// A world with house 0's grinder six cells south of its base.
fn scene(g: &Game) -> (World, ObjectId) {
    let mut w = g.world.clone();
    let b = spawn(&mut w, Kind::Building, "GRND", 0, 0, 6);
    // The whole map explored, so no waiting unit is sent to scout (M8 step
    // A4 round 8) and the surplus is what the scene places.
    let spots: Vec<CellXY> = w
        .cells
        .grid
        .iter()
        .map(|(c, _)| c)
        .filter(|c| c.x % 4 == 0 && c.y % 4 == 0)
        .collect();
    for c in spots {
        glasnost_sim::econ::shroud::area_reveal(
            &mut w,
            HouseId(0),
            glasnost_sim::world::Pos::centre(c, 0),
            5,
        );
    }
    testkit::run(&mut w, 1);
    (w, b)
}

fn settle(w: &mut World) {
    testkit::run(w, 1);
    while w.tick % 15 != 0 {
        testkit::run(w, 1);
    }
}

/// The building unit `who`'s last behaviour order sends it into with
/// `behaviour`.
fn sent(orders: &[Order], who: ObjectId, behaviour: Behaviour) -> Option<ObjectId> {
    orders.iter().rev().find_map(|o| match o {
        Order::Behaviour {
            actor,
            behaviour: b,
            dest,
            ..
        } if *actor == who => match dest {
            Some(Target::Object(x)) if *b == behaviour => Some(Some(*x)),
            _ => Some(None),
        },
        _ => None,
    })?
}

fn brain(w: &World) -> Brain {
    Brain::new(w, &fixture::map(), HouseId(0))
}

// S-0603 claims 1, 8 and 9 and S-0214 row n2, PLAN §4.7: a controlled enemy vehicle is recycled for money with the recycle click; the counter counts the order.
#[test]
fn a_controlled_enemy_vehicle_is_recycled() {
    let g = game(99);
    let (mut w, b) = scene(&g);
    let clone = spawn(&mut w, Kind::Infantry, "CLONE", 0, 3, 9);
    let foe = spawn(&mut w, Kind::Vehicle, "GUNR", 1, 4, 9);
    assert!(mind::take(&mut w, clone, foe));
    settle(&mut w);
    let mut br = brain(&w);
    let orders = br.think(&w);
    assert_eq!(sent(&orders, foe, Behaviour::Eaten), Some(b), "{orders:?}");
    assert_eq!(
        sent(&orders, clone, Behaviour::Eaten),
        None,
        "the controller is kept"
    );
    assert_eq!(br.special_uses().grinds, 1);
}

// S-0605 claim 7 and S-0603 claim 9, PLAN §4.7: controlled infantry go to an absorber with room; with no absorber they are recycled.
#[test]
fn controlled_infantry_go_to_an_absorber_first() {
    let g = game(99);
    for with_absorber in [true, false] {
        let (mut w, b) = scene(&g);
        let bio = with_absorber.then(|| spawn(&mut w, Kind::Building, "BIOR", 0, -6, 6));
        let clone = spawn(&mut w, Kind::Infantry, "CLONE", 0, 3, 9);
        let foe = spawn(&mut w, Kind::Infantry, "WALKER", 1, 4, 9);
        assert!(mind::take(&mut w, clone, foe));
        settle(&mut w);
        let orders = brain(&w).think(&w);
        if let Some(bio) = bio {
            assert_eq!(
                sent(&orders, foe, Behaviour::Enter),
                Some(bio),
                "{orders:?}"
            );
        } else {
            assert_eq!(sent(&orders, foe, Behaviour::Eaten), Some(b), "{orders:?}");
        }
    }
}

// S-0603 claim 8, PLAN §4.7: a badly damaged own vehicle is recycled, a healthy one is not, and a harvester and an engineer are kept however damaged.
#[test]
fn badly_damaged_units_are_recycled() {
    let g = game(99);
    let (mut w, b) = scene(&g);
    let hurt = spawn(&mut w, Kind::Vehicle, "GUNR", 0, 2, 10);
    let fine = spawn(&mut w, Kind::Vehicle, "GUNR", 0, 4, 10);
    let harv = spawn(&mut w, Kind::Vehicle, "MINR", 0, 6, 10);
    let eng = spawn(&mut w, Kind::Infantry, "ENGI", 0, 4, 12);
    for x in [hurt, harv, eng] {
        let o = w.obj_mut(x).unwrap();
        o.health /= 5;
    }
    settle(&mut w);
    let orders = brain(&w).think(&w);
    assert_eq!(sent(&orders, hurt, Behaviour::Eaten), Some(b), "{orders:?}");
    assert_ne!(sent(&orders, fine, Behaviour::Eaten), Some(b));
    assert_ne!(sent(&orders, harv, Behaviour::Eaten), Some(b));
    assert_ne!(sent(&orders, eng, Behaviour::Eaten), Some(b));
}

// PLAN §4.7: surplus infantry at home (beyond KEEP, fewer than a wave of the plan, which would take them first) are recycled only while money is short and no absorber takes them.
#[test]
fn surplus_infantry_only_when_money_is_short() {
    let g = game(99);
    for short in [false, true] {
        let (mut w, b) = scene(&g);
        let men: Vec<ObjectId> = (0..glasnost_ai::absorber::KEEP as i32 + 1)
            .map(|i| spawn(&mut w, Kind::Infantry, "WALKER", 0, i - 3, 10))
            .collect();
        if short {
            w.houses[0].credits = LOW_MONEY / 2;
        } else {
            w.houses[0].credits = LOW_MONEY * 10;
        }
        settle(&mut w);
        let orders = brain(&w).think(&w);
        let n = men
            .iter()
            .filter(|&&x| sent(&orders, x, Behaviour::Eaten) == Some(b))
            .count();
        assert_eq!(n, if short { 1 } else { 0 }, "short {short}: {orders:?}");
    }
}

// S-0603 claims 1 and 2, PLAN §4.7: a switched-off grinder offers no recycle cursor to a vehicle, so the brain sends nothing there.
#[test]
fn a_switched_off_grinder_gets_no_vehicle() {
    let g = game(99);
    let (mut w, b) = scene(&g);
    let clone = spawn(&mut w, Kind::Infantry, "CLONE", 0, 3, 9);
    let foe = spawn(&mut w, Kind::Vehicle, "GUNR", 1, 4, 9);
    assert!(mind::take(&mut w, clone, foe));
    w.obj_mut(b).unwrap().econ.building.switched_off = true;
    settle(&mut w);
    let orders = brain(&w).think(&w);
    assert_eq!(sent(&orders, foe, Behaviour::Eaten), None, "{orders:?}");
}

// PLAN §4.7: the house builds a grinder only once it holds a unit it would send (here a badly damaged own vehicle), queued on its line and paid as it goes, and only one.
#[test]
fn the_house_builds_one_grinder_when_it_has_a_use() {
    let mut g = game(1);
    let grnd = g.world.rules().index_of(Kind::Building, "GRND").unwrap();
    let count = |w: &World| {
        w.objects
            .values()
            .filter(|o| {
                o.owner == Some(HouseId(0)) && o.kind == Kind::Building && o.type_index == grnd
            })
            .count()
    };
    for _ in 0..3000 {
        g.step();
    }
    assert_eq!(count(&g.world), 0, "no use, no grinder");
    let hurt = spawn(&mut g.world, Kind::Vehicle, "GUNR", 0, 2, 10);
    g.world.obj_mut(hurt).unwrap().health /= 5;
    for _ in 0..6000 {
        g.step();
        if count(&g.world) > 0 {
            break;
        }
    }
    assert_eq!(count(&g.world), 1, "built");
    for _ in 0..3000 {
        g.step();
    }
    assert!(count(&g.world) <= 1, "only one");
}

// S-0603 claims 1 and 9, PLAN §4.7: a controlled enemy harvester standing idle is recycled (only the house's own harvesters are kept), whatever the plan told it in the same think.
#[test]
fn a_controlled_enemy_harvester_is_recycled() {
    let g = game(99);
    let (mut w, b) = scene(&g);
    let clone = spawn(&mut w, Kind::Infantry, "CLONE", 0, 3, 9);
    let foe = spawn(&mut w, Kind::Vehicle, "MINR", 1, 4, 9);
    assert!(mind::take(&mut w, clone, foe));
    settle(&mut w);
    let mut br = brain(&w);
    let orders = br.think(&w);
    assert_eq!(sent(&orders, foe, Behaviour::Eaten), Some(b), "{orders:?}");
    assert_eq!(br.special_uses().grinds, 1);
}

// S-0603 claim 15, PLAN §4.7: the base defence gives no new order to a unit on its way into the grinder (a new order would end its trip), while a free unit near it is sent at the intruder.
#[test]
fn the_defence_leaves_a_unit_on_its_way_in() {
    let g = game(99);
    let (mut w, b) = scene(&g);
    let hurt = spawn(&mut w, Kind::Vehicle, "GUNR", 0, 2, 10);
    let fine = spawn(&mut w, Kind::Vehicle, "GUNR", 0, 4, 10);
    w.obj_mut(hurt).unwrap().health /= 5;
    settle(&mut w);
    let mut br = brain(&w);
    let orders = br.think(&w);
    assert_eq!(sent(&orders, hurt, Behaviour::Eaten), Some(b), "{orders:?}");
    for o in orders {
        glasnost_sim::orders::issue(&mut w, HouseId(0), o);
    }
    testkit::run(&mut w, 1);
    spawn(&mut w, Kind::Vehicle, "GUNR", 1, 3, 2);
    testkit::run(&mut w, 1);
    let orders = br.think(&w);
    let attacked = |who: ObjectId| {
        orders.iter().any(|o| {
            matches!(o, Order::Behaviour { actor, behaviour: Behaviour::Attack, .. } if *actor == who)
        })
    };
    assert!(attacked(fine), "the defence is on: {orders:?}");
    assert!(!attacked(hurt), "{orders:?}");
}

// PLAN §4.7: with every absorbing building full (two on their way fill BIOR), surplus infantry at home are recycled while money is short.
#[test]
fn surplus_infantry_when_the_absorber_is_full() {
    let g = game(99);
    let (mut w, b) = scene(&g);
    let bio = spawn(&mut w, Kind::Building, "BIOR", 0, -6, 6);
    testkit::run(&mut w, 1);
    for i in 0..2 {
        let m = spawn(&mut w, Kind::Infantry, "WALKER", 0, -6 + i, 10);
        glasnost_sim::orders::issue(
            &mut w,
            HouseId(0),
            Order::Behaviour {
                actor: m,
                behaviour: Behaviour::Enter,
                attack_move: false,
                target: None,
                dest: Some(Target::Object(bio)),
            },
        );
    }
    let men: Vec<ObjectId> = (0..glasnost_ai::absorber::KEEP as i32 + 1)
        .map(|i| spawn(&mut w, Kind::Infantry, "WALKER", 0, i - 3, 12))
        .collect();
    w.houses[0].credits = LOW_MONEY / 2;
    settle(&mut w);
    let orders = brain(&w).think(&w);
    let n = men
        .iter()
        .filter(|&&x| sent(&orders, x, Behaviour::Eaten) == Some(b))
        .count();
    assert_eq!(n, 1, "{orders:?}");
}

// PLAN §4.7: a badly damaged unit far from the grinder (away from home too) is first moved to the grinder's side by a click on the ground, then sent in from there.
#[test]
fn a_far_unit_is_moved_to_the_grinder_first() {
    let g = game(99);
    let (mut w, b) = scene(&g);
    let far = spawn(&mut w, Kind::Infantry, "WALKER", 0, 16, 6);
    w.obj_mut(far).unwrap().health /= 5;
    settle(&mut w);
    let mut br = brain(&w);
    let orders = br.think(&w);
    let side = orders.iter().rev().find_map(|o| match o {
        Order::Behaviour {
            actor,
            behaviour: Behaviour::Move,
            dest: Some(Target::Cell(c)),
            ..
        } if *actor == far => Some(*c),
        _ => None,
    });
    let g0 = w.obj(b).unwrap().pos.cell();
    let side = side.unwrap_or_else(|| panic!("moved first: {orders:?}"));
    assert!(
        (side.x - g0.x).abs().max((side.y - g0.y).abs()) <= 4,
        "{side:?} by {g0:?}"
    );
    assert_eq!(br.special_uses().grinds, 0);
    for o in orders {
        glasnost_sim::orders::issue(&mut w, HouseId(0), o);
    }
    let mut sent_in = false;
    for _ in 0..40 {
        for _ in 0..15 {
            testkit::run(&mut w, 1);
        }
        let orders = br.think(&w);
        if sent(&orders, far, Behaviour::Eaten) == Some(b) {
            sent_in = true;
            break;
        }
        for o in orders {
            glasnost_sim::orders::issue(&mut w, HouseId(0), o);
        }
    }
    assert!(sent_in, "sent in once at the side");
}

// PLAN §4.7 (M8 B4): a far unit that does not take the click to the grinder's side (it stands idle where it was) is sent to another side at the next think, never the same cell again.
#[test]
fn a_far_unit_that_does_not_move_tries_another_side() {
    let g = game(99);
    let (mut w, b) = scene(&g);
    let far = spawn(&mut w, Kind::Infantry, "WALKER", 0, 16, 6);
    w.obj_mut(far).unwrap().health /= 5;
    settle(&mut w);
    let g0 = w.obj(b).unwrap().pos.cell();
    let mut br = brain(&w);
    let side = |orders: &[Order]| {
        orders.iter().rev().find_map(|o| match o {
            Order::Behaviour {
                actor,
                behaviour: Behaviour::Move,
                dest: Some(Target::Cell(c)),
                ..
            } if *actor == far => Some(*c),
            _ => None,
        })
    };
    let start = w.obj(far).unwrap().pos.cell();
    let mut tried: Vec<CellXY> = Vec::new();
    for _ in 0..3 {
        // The orders are dropped: the unit stays idle on its cell, as one
        // whose route does not exist.
        let orders = br.think(&w);
        let c = side(&orders).unwrap_or_else(|| panic!("moved: {orders:?}"));
        assert!(
            (c.x - g0.x).abs().max((c.y - g0.y).abs()) <= 4,
            "{c:?} by {g0:?}"
        );
        assert!(!tried.contains(&c), "{c:?} again after {tried:?}");
        tried.push(c);
        for _ in 0..15 {
            testkit::run(&mut w, 1);
        }
    }
    assert_eq!(w.obj(far).unwrap().pos.cell(), start, "it never moved");
}

// design: own AI (#8) and S-0411 claim 6 (a building line with an item takes no starting or queuing build order): while the building line makes something else, the house that has a use for a grinder does not order one (the sim would refuse it).
#[test]
fn no_grinder_order_while_the_building_line_is_busy() {
    let g = game(1);
    let mut w = g.world.clone();
    let map = fixture::map();
    let mut br = Brain::with_plan(&w, &map, HouseId(0), fixture::plan());
    let grnd = w.rules().index_of(Kind::Building, "GRND").unwrap();
    let (mut refused, mut taken) = (0, 0);
    for step in 0..9000 {
        // The base up, a badly damaged vehicle gives the house a use for a grinder.
        if step == 3000 {
            let hurt = spawn(&mut w, Kind::Vehicle, "GUNR", 0, 2, 10);
            w.obj_mut(hurt).unwrap().health /= 5;
        }
        for o in br.think(&w) {
            let grinder = matches!(&o, Order::Build { what } if what.kind == Kind::Building && what.index == grnd);
            let ok = glasnost_sim::orders::issue(&mut w, HouseId(0), o);
            match (grinder, ok) {
                (true, false) => refused += 1,
                (true, true) => taken += 1,
                _ => {}
            }
        }
        glasnost_sim::tick::step(&mut w);
    }
    assert!(taken >= 1, "the grinder is built");
    assert_eq!(refused, 0, "no refused grinder build order");
}
