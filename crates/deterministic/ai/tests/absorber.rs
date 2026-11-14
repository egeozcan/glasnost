//! The own AI and absorbing buildings (S-0605, PLAN §4.7): spare infantry
//! at home beyond the ones kept go into an own infantry absorber, all of
//! them while the house is short of power; a full absorber gets no more;
//! a controlled enemy infantry unit is sent whatever the house keeps;
//! engineers are kept. The tests use invented types on the synthetic
//! skirmish.

use std::sync::Arc;

use glasnost_ai::absorber::KEEP;
use glasnost_ai::brain::Brain;
use glasnost_ai::fixture;
use glasnost_ai::game::Game;
use glasnost_map::CellXY;
use glasnost_rules::sim::{Decimal, WarheadType, WeaponType};
use glasnost_sim::orders::Order;
use glasnost_sim::rules::{Behaviour, Kind, SimRules};
use glasnost_sim::world::{HouseId, ObjectId, Target, World};
use glasnost_sim::{mind, testkit};

fn d(s: &str) -> Decimal {
    Decimal(s.into())
}

/// The fixture's rules with BIOR (a 2 × 2 infantry absorber for `cap`,
/// size limit 1, power 50, extra power 20), BIOD (BIOR with a drain of
/// 500), the engineer ENGI, and the controller CLONE (weapon RAY with the
/// takeover warhead MIND), all beyond the sidebar's tech level.
fn rules(cap: i32) -> Arc<SimRules> {
    let mut r = fixture::ruleset();
    let reg = |r: &mut glasnost_rules::schema::Ruleset, k: &str, n: &str| {
        r.registries.entry(k.into()).or_default().push(n.into());
    };
    for (name, power) in [("BIOR", 50), ("BIOD", -500)] {
        let mut b = fixture::building(2, 2, 600, power);
        b.infantry_absorb = Some(true);
        b.passengers = Some(cap);
        b.size_limit = Some(d("1"));
        b.econ.extra_power = Some(20);
        b.tech_level = Some(99);
        r.sim.buildings.insert(name.into(), b);
        reg(&mut r, "buildings", name);
    }
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
            damage: Some(2),
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
    Arc::new(SimRules::new(&r, "synthetic-absorber"))
}

fn game(cap: i32) -> Game {
    let mut g = Game::new(
        rules(cap),
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

/// A world with house 0's absorber `bio` six cells south of its base and
/// `n` infantry of type `name` of house 0 in a row two cells further south.
fn scene(g: &Game, bio: &str, name: &str, n: usize) -> (World, ObjectId, Vec<ObjectId>) {
    let mut w = g.world.clone();
    let h = home(&w);
    let b = testkit::spawn(&mut w, Kind::Building, bio, 0, CellXY::new(h.x, h.y + 6), 0);
    let units = (0..n)
        .map(|i| {
            testkit::spawn(
                &mut w,
                Kind::Infantry,
                name,
                0,
                CellXY::new(h.x - 3 + i as i32, h.y + 9),
                0,
            )
        })
        .collect();
    testkit::run(&mut w, 1);
    while w.tick % 15 != 0 {
        testkit::run(&mut w, 1);
    }
    (w, b, units)
}

/// The building unit `who`'s last behaviour order sends it into.
fn entered(orders: &[Order], who: ObjectId) -> Option<ObjectId> {
    orders.iter().rev().find_map(|o| match o {
        Order::Behaviour {
            actor,
            behaviour,
            dest,
            ..
        } if *actor == who => match (behaviour, dest) {
            (Behaviour::Enter, Some(Target::Object(b))) => Some(Some(*b)),
            _ => Some(None),
        },
        _ => None,
    })?
}

fn brain(w: &World) -> Brain {
    Brain::new(w, &fixture::map(), HouseId(0))
}

// S-0605 claims 1, 3 and 7, PLAN §4.7: with power to spare the house keeps KEEP idle infantry at home and sends the rest into its absorber; the counter counts the orders.
#[test]
fn sends_spare_infantry_beyond_the_ones_kept() {
    let g = game(10);
    let (w, b, u) = scene(&g, "BIOR", "WALKER", KEEP + 2);
    let (out, drain) = glasnost_sim::econ::power::totals(&w, HouseId(0));
    assert!(out > drain, "power to spare: {out} {drain}");
    let mut br = brain(&w);
    let orders = br.think(&w);
    let sent = u
        .iter()
        .filter(|&&x| entered(&orders, x) == Some(b))
        .count();
    assert_eq!(sent, 2, "{orders:?}");
    assert_eq!(br.special_uses().absorbs, 2);
}

// PLAN §4.7: no more than KEEP idle infantry at home and power to spare: none is sent.
#[test]
fn keeps_its_infantry_when_power_is_enough() {
    let g = game(10);
    let (w, b, u) = scene(&g, "BIOR", "WALKER", KEEP);
    let mut br = brain(&w);
    let orders = br.think(&w);
    assert!(
        u.iter().all(|&x| entered(&orders, x) != Some(b)),
        "{orders:?}"
    );
    assert_eq!(br.special_uses().absorbs, 0);
}

// S-0605 claim 7 and S-0405 claim 1, PLAN §4.7: short of power, idle infantry at home go in, the room permitting.
#[test]
fn short_of_power_sends_them_all_the_room_permitting() {
    let g = game(2);
    let (w, b, u) = scene(&g, "BIOD", "WALKER", 3);
    let (out, drain) = glasnost_sim::econ::power::totals(&w, HouseId(0));
    assert!(out <= drain, "short of power: {out} {drain}");
    let mut br = brain(&w);
    let orders = br.think(&w);
    let sent = u
        .iter()
        .filter(|&&x| entered(&orders, x) == Some(b))
        .count();
    assert_eq!(sent, 2, "capacity 2: {orders:?}");
}

// design: own AI (#15) and S-0405 claims 5 and 6 (a timed blackout sets the house's output to 0 whatever plants stand): with power to spare a blackout does not make the house short of power, so it still keeps KEEP idle infantry at home.
#[test]
fn a_blackout_does_not_send_the_kept_infantry_in() {
    let g = game(10);
    let (mut w, b, u) = scene(&g, "BIOR", "WALKER", KEEP + 2);
    assert!(!glasnost_ai::power::short(&w, HouseId(0)), "power to spare");
    glasnost_sim::econ::power::start_blackout(&mut w, HouseId(0), 1000);
    testkit::run(&mut w, 1);
    while w.tick % 15 != 0 {
        testkit::run(&mut w, 1);
    }
    let (out, _) = glasnost_sim::econ::power::totals(&w, HouseId(0));
    assert_eq!(out, 0, "the blackout zeroes the house's output");
    let mut br = brain(&w);
    let orders = br.think(&w);
    let sent = u
        .iter()
        .filter(|&&x| entered(&orders, x) == Some(b))
        .count();
    assert_eq!(sent, 2, "only the infantry beyond KEEP: {orders:?}");
}

// S-0605 claim 1 and S-0214 rows x3 and x4, PLAN §4.7: engineers are kept for their own work.
#[test]
fn engineers_are_kept() {
    let g = game(10);
    let (w, b, u) = scene(&g, "BIOD", "ENGI", 3);
    let mut br = brain(&w);
    let orders = br.think(&w);
    assert!(
        u.iter().all(|&x| entered(&orders, x) != Some(b)),
        "{orders:?}"
    );
}

// design (PLAN §4.7, the opponent is ours): a slave with a master is its master's worker, not spare infantry; it is never sent into the absorber while others are.
#[test]
fn slaves_are_kept() {
    let g = game(10);
    let (mut w, b, u) = scene(&g, "BIOR", "WALKER", KEEP + 2);
    let master = u[0];
    for &x in &u[KEEP..] {
        w.obj_mut(x).unwrap().econ.slave = Some(glasnost_sim::econ::slaves::SlaveState {
            master: Some(master),
            phase: Default::default(),
            load: Vec::new(),
            remembered: None,
            recalled: false,
        });
    }
    let mut br = brain(&w);
    let orders = br.think(&w);
    assert!(
        u[KEEP..].iter().all(|&x| entered(&orders, x) != Some(b)),
        "{orders:?}"
    );
}

// S-0605 claims 7 and 10 and S-0601 claim 9, PLAN §4.7: a controlled enemy infantry unit is sent in whatever the house keeps.
#[test]
fn a_controlled_enemy_is_sent_in() {
    let g = game(10);
    let (mut w, b, u) = scene(&g, "BIOR", "CLONE", 1);
    let h = home(&w);
    let foe = testkit::spawn(
        &mut w,
        Kind::Infantry,
        "WALKER",
        1,
        CellXY::new(h.x + 3, h.y + 9),
        0,
    );
    assert!(mind::take(&mut w, u[0], foe));
    while w.tick % 15 != 0 {
        testkit::run(&mut w, 1);
    }
    let mut br = brain(&w);
    let orders = br.think(&w);
    assert_eq!(entered(&orders, foe), Some(b), "{orders:?}");
    assert_eq!(
        entered(&orders, u[0]),
        None,
        "a controller holding a victim is refused"
    );
}

// S-0601 claim 13 (a dominated object belongs to the house for good; no controller holds it), design: own AI (#35): a permanently controlled infantry unit frees no controller, so it counts as the house's own: with power to spare and no more than KEEP idle at home it is kept.
#[test]
fn a_dominated_infantry_unit_counts_as_own() {
    let g = game(10);
    let (mut w, b, _) = scene(&g, "BIOR", "WALKER", 0);
    let h = home(&w);
    let taken = testkit::spawn(
        &mut w,
        Kind::Infantry,
        "WALKER",
        1,
        CellXY::new(h.x + 3, h.y + 9),
        0,
    );
    assert!(mind::dominate(&mut w, taken, HouseId(0)));
    assert!(mind::permanent(&w, taken) && mind::controlled_by(&w, taken).is_none());
    testkit::run(&mut w, 1);
    while w.tick % 15 != 0 {
        testkit::run(&mut w, 1);
    }
    let (out, drain) = glasnost_sim::econ::power::totals(&w, HouseId(0));
    assert!(out > drain, "power to spare: {out} {drain}");
    let mut br = brain(&w);
    let orders = br.think(&w);
    assert_ne!(entered(&orders, taken), Some(b), "{orders:?}");
    assert_eq!(br.special_uses().absorbs, 0);
}

// S-0601 claims 1 and 2, PLAN §4.7: mind controllers are kept for their own work, even short of power.
#[test]
fn controllers_are_kept() {
    let g = game(10);
    let (w, b, u) = scene(&g, "BIOD", "CLONE", 3);
    let mut br = brain(&w);
    let orders = br.think(&w);
    assert!(
        u.iter().all(|&x| entered(&orders, x) != Some(b)),
        "{orders:?}"
    );
}

// S-0605 claims 3 and 4, PLAN §4.7: a unit already waiting in line is not sent again (a new order would end its wait).
#[test]
fn a_unit_waiting_in_line_is_not_sent_again() {
    let g = game(10);
    let (mut w, b, u) = scene(&g, "BIOD", "WALKER", 2);
    let mut br = brain(&w);
    let orders = br.think(&w);
    assert_eq!(
        u.iter()
            .filter(|&&x| entered(&orders, x) == Some(b))
            .count(),
        2
    );
    for o in orders {
        glasnost_sim::orders::issue(&mut w, HouseId(0), o);
    }
    testkit::run(&mut w, 1);
    let waiting: Vec<ObjectId> = u
        .iter()
        .copied()
        .filter(|&x| glasnost_sim::structures::absorb::waiting_for(&w, x) == Some(b))
        .collect();
    assert_eq!(waiting.len(), 1, "one link place: the second waits");
    while w.tick % 15 != 0 {
        testkit::run(&mut w, 1);
    }
    // As seen in AI games: the one waiting rests in guard while it waits.
    w.obj_mut(waiting[0]).unwrap().beh.current = Some(Behaviour::Guard);
    assert_eq!(
        glasnost_sim::structures::absorb::waiting_for(&w, waiting[0]),
        Some(b)
    );
    // A fresh brain: no memory of the first order.
    let orders = brain(&w).think(&w);
    assert_eq!(entered(&orders, waiting[0]), None, "{orders:?}");
}

// S-0605 claims 5 and 17 and S-0213 table T3 (b), PLAN §4.7: an entrant that has left enter while holding its link place is given the enter click again (the new order ends the contact), so the unit waiting behind it is not stuck for good.
#[test]
fn a_stalled_entrant_is_sent_again() {
    let g = game(10);
    let (mut w, b, u) = scene(&g, "BIOD", "WALKER", 2);
    let orders = brain(&w).think(&w);
    for o in orders {
        glasnost_sim::orders::issue(&mut w, HouseId(0), o);
    }
    testkit::run(&mut w, 1);
    let holder = u
        .iter()
        .copied()
        .find(|&x| w.obj(x).unwrap().contact == Some(b))
        .expect("one in contact");
    // Taken out of enter with its contact intact.
    {
        let o = w.obj_mut(holder).unwrap();
        o.beh.current = Some(Behaviour::Guard);
        o.beh.queued = None;
        o.dest = None;
    }
    while w.tick % 15 != 0 {
        testkit::run(&mut w, 1);
    }
    assert_eq!(glasnost_ai::absorber::stalled(&w, holder), Some(b));
    let orders = brain(&w).think(&w);
    assert_eq!(entered(&orders, holder), Some(b), "{orders:?}");
}

// S-0605 claim 1 (a switched-off building refuses) and S-0214 rows x3 and x4, design: own AI (#44): when the nearest absorber refuses the enter click, the unit goes to the next nearest one that takes it.
#[test]
fn a_refusing_nearest_absorber_does_not_block_the_others() {
    let g = game(10);
    let (mut w, near, u) = scene(&g, "BIOD", "WALKER", 1);
    let h = home(&w);
    let far = testkit::spawn(
        &mut w,
        Kind::Building,
        "BIOD",
        0,
        CellXY::new(h.x + 6, h.y + 6),
        0,
    );
    w.obj_mut(near).unwrap().econ.building.switched_off = true;
    testkit::run(&mut w, 1);
    while w.tick % 15 != 0 {
        testkit::run(&mut w, 1);
    }
    let at = w.obj(u[0]).unwrap().pos.cell();
    let d2 = |b: ObjectId| {
        let c = w.obj(b).unwrap().pos.cell();
        let (dx, dy) = (i64::from(c.x - at.x), i64::from(c.y - at.y));
        dx * dx + dy * dy
    };
    assert!(d2(near) < d2(far), "the switched-off one is the nearer");
    let orders = brain(&w).think(&w);
    assert_eq!(entered(&orders, u[0]), Some(far), "{orders:?}");
}

// S-0605 claims 3, 4 and 17, design: own AI (#44): an entrant sent again after it stalled counts against its building's room; no more entrants are sent than the building has room for.
#[test]
fn a_resent_stalled_entrant_takes_its_room() {
    let g = game(2);
    let (mut w, b, u) = scene(&g, "BIOD", "WALKER", 2);
    let orders = brain(&w).think(&w);
    for o in orders {
        glasnost_sim::orders::issue(&mut w, HouseId(0), o);
    }
    testkit::run(&mut w, 1);
    let holder = u
        .iter()
        .copied()
        .find(|&x| w.obj(x).unwrap().contact == Some(b))
        .expect("one in contact");
    {
        let o = w.obj_mut(holder).unwrap();
        o.beh.current = Some(Behaviour::Guard);
        o.beh.queued = None;
        o.dest = None;
    }
    // A third walker at home: capacity 2 holds the stalled one and the one
    // waiting behind it, so there is no room for it.
    let h = home(&w);
    let third = testkit::spawn(
        &mut w,
        Kind::Infantry,
        "WALKER",
        0,
        CellXY::new(h.x + 2, h.y + 9),
        0,
    );
    testkit::run(&mut w, 1);
    while w.tick % 15 != 0 {
        testkit::run(&mut w, 1);
    }
    assert_eq!(glasnost_ai::absorber::stalled(&w, holder), Some(b));
    let orders = brain(&w).think(&w);
    assert_eq!(entered(&orders, holder), Some(b), "{orders:?}");
    assert_eq!(entered(&orders, third), None, "{orders:?}");
}
