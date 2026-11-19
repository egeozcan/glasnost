//! #454: an AI house's spy disguises before it infiltrates. The synthetic
//! skirmish with invented types (test data of our own): SPYD, a spy whose
//! weapon DISGW has the disguise-flag warhead DISGH (S-0516 claim 31);
//! SPYU, a spy with no weapon; SPYP, a spy that disguises permanently
//! (S-0516 claim 32); PAWN, unarmed infantry; SPYB, a building a spy
//! enters for its power (S-0510); DETB, a building that detects disguises
//! within three cells (S-0516 claim 40).

use std::sync::Arc;

use glasnost_ai::brain::Brain;
use glasnost_ai::fixture;
use glasnost_ai::game::Game;
use glasnost_ai::scripted::THINK_PERIOD;
use glasnost_ai::scripts::{REGISTRY, Seat};
use glasnost_map::CellXY;
use glasnost_rules::sim::{
    BuildingType, Decimal, LocomotionClass, Locomotor, UnitType, WarheadType, WeaponType,
};
use glasnost_sim::cloak::{Disguise, DisguiseAs};
use glasnost_sim::combat::damage::HitResult;
use glasnost_sim::econ::shroud;
use glasnost_sim::load::new_game;
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind, SimRules};
use glasnost_sim::testkit;
use glasnost_sim::world::{AiDifficulty, AiSeat, HouseId, ObjectId, Pos, Target, World};

const ALPHA: HouseId = HouseId(0);

/// The detection range of DETB, in cells.
const DETECT: i32 = 3; // nocite: test data of our own

fn d(s: &str) -> Decimal {
    Decimal(s.into())
}

/// A walking spy type of our own, with weapon `primary` if any.
fn spy(primary: Option<&str>) -> UnitType {
    let mut u = UnitType {
        strength: Some(100),
        speed: Some(6),
        locomotion_class: Some(LocomotionClass::Foot),
        locomotor: Some(Locomotor::Walk),
        sight: Some(4),
        agent: Some(true),
        infiltrate: Some(true),
        tech_level: Some(99),
        cost: Some(100),
        primary: primary.map(Into::into),
        ..Default::default()
    };
    u.combat.can_disguise = Some(true);
    u
}

fn rules() -> Arc<SimRules> {
    let mut r = fixture::ruleset();
    let reg = |r: &mut glasnost_rules::schema::Ruleset, k: &str, n: &str| {
        r.registries.entry(k.into()).or_default().push(n.into());
    };
    r.sim.warheads.insert(
        "DISGH".into(),
        WarheadType {
            makes_disguise: Some(true),
            verses: Some(vec![d("1"); 11]),
            ..Default::default()
        },
    );
    r.sim.weapons.insert(
        "DISGW".into(),
        WeaponType {
            damage: Some(1),
            rof: Some(30),
            range: Some(d("3")),
            speed: Some(100),
            projectile: Some("Bullet".into()),
            warhead: Some("DISGH".into()),
            ..Default::default()
        },
    );
    r.sim.infantry.insert("SPYD".into(), spy(Some("DISGW")));
    reg(&mut r, "infantry", "SPYD");
    let mut plain = spy(None);
    plain.combat.can_disguise = None;
    r.sim.infantry.insert("SPYU".into(), plain);
    reg(&mut r, "infantry", "SPYU");
    let mut perma = spy(None);
    perma.perma_disguise = Some(true);
    r.sim.infantry.insert("SPYP".into(), perma);
    reg(&mut r, "infantry", "SPYP");
    r.sim.infantry.insert(
        "PAWN".into(),
        UnitType {
            strength: Some(100),
            speed: Some(6),
            locomotion_class: Some(LocomotionClass::Foot),
            locomotor: Some(Locomotor::Walk),
            sight: Some(2),
            tech_level: Some(99),
            ..Default::default()
        },
    );
    reg(&mut r, "infantry", "PAWN");
    let mut spyb = BuildingType {
        strength: Some(400),
        foundation: Some([2, 2]),
        spyable: Some(true),
        tech_level: Some(99),
        ..Default::default()
    };
    spyb.econ.power = Some(100);
    r.sim.buildings.insert("SPYB".into(), spyb);
    reg(&mut r, "buildings", "SPYB");
    let mut det = BuildingType {
        strength: Some(400),
        foundation: Some([1, 1]),
        tech_level: Some(99),
        detect_disguise_range: Some(DETECT),
        ..Default::default()
    };
    det.combat.detect_disguise = Some(true);
    r.sim.buildings.insert("DETB".into(), det);
    reg(&mut r, "buildings", "DETB");
    r.sim.general.allied_disguise = Some("PAWN".into());
    r.sim.general.soviet_disguise = Some("PAWN".into());
    r.sim.general.third_disguise = Some("PAWN".into());
    Arc::new(SimRules::new(&r, "synthetic-skirmish"))
}

fn game(seed: u32) -> Game {
    Game::new(
        rules(),
        &fixture::map(),
        "synthetic-skirmish",
        fixture::catalogue(),
        fixture::setup(seed),
    )
    .expect("the synthetic skirmish loads")
}

/// The game run until orders are allowed, to a think tick.
fn ready() -> Game {
    let mut g = game(5);
    while g.world.tick <= g.world.orders_blocked_through || g.world.tick % 15 != 0 {
        g.step();
    }
    g
}

/// Runs the world on to the next think tick after `ticks` more ticks.
fn run_to_think(w: &mut World, ticks: i32) {
    testkit::run(w, ticks);
    while w.tick % 15 != 0 {
        testkit::run(w, 1);
    }
}

/// The cell of house 0's first object on the map.
fn home_of(w: &World) -> CellXY {
    w.objects
        .values()
        .find(|o| o.owner == Some(ALPHA) && o.on_map)
        .map(|o| o.pos.cell())
        .expect("an object of house 0")
}

fn at(home: CellXY, dx: i32, dy: i32) -> CellXY {
    CellXY::new(home.x + dx, home.y + dy)
}

/// Reveals the area around `c` to house 0.
fn reveal(w: &mut World, c: CellXY) {
    shroud::area_reveal(w, ALPHA, Pos::centre(c, 0), 4);
}

/// The scene: house 0's spy of type `spy` at (2, −8) from home, house 1's
/// SPYB at (12, −8), both shown to house 0; nothing between them shown.
fn scene(g: &Game, spy: &str) -> (World, ObjectId, ObjectId) {
    let mut w = g.world.clone();
    let home = home_of(&w);
    let b = testkit::spawn(&mut w, Kind::Building, "SPYB", 1, at(home, 12, -8), 0);
    let s = testkit::spawn(&mut w, Kind::Infantry, spy, 0, at(home, 2, -8), 0);
    reveal(&mut w, at(home, 13, -8));
    (w, s, b)
}

/// House 1's PAWN at (dx, dy) from home.
fn pawn(w: &mut World, dx: i32, dy: i32) -> ObjectId {
    let home = home_of(w);
    testkit::spawn(w, Kind::Infantry, "PAWN", 1, at(home, dx, dy), 0)
}

/// Gives the spy the disguise of house 1's PAWN, as a hit of its weapon
/// on one would (S-0516 claim 31).
fn disguise_as_victim(w: &mut World, s: ObjectId) {
    let p = w.rules().index_of(Kind::Infantry, "PAWN").unwrap();
    let tick = w.tick;
    w.obj_mut(s).unwrap().cloak.disguise = Some(Disguise {
        what: Some(DisguiseAs::Infantry(p)),
        house: Some(HouseId(1)),
        since: tick,
    });
}

fn orders_to(out: &[Order], id: ObjectId) -> Vec<Order> {
    out.iter()
        .filter(|o| o.actor() == Some(id))
        .cloned()
        .collect()
}

fn attack(actor: ObjectId, t: ObjectId) -> Order {
    Order::Behaviour {
        actor,
        behaviour: Behaviour::Attack,
        attack_move: false,
        target: Some(Target::Object(t)),
        dest: None,
    }
}

/// Whether `o` sends `actor` into building `b` (the capture a spy's click
/// on an enemy building gives, S-0214 row I24).
fn enters(o: &Order, actor: ObjectId, b: ObjectId) -> bool {
    matches!(o, Order::Behaviour { actor: a, behaviour: Behaviour::Capture, dest: Some(Target::Object(d)), .. }
        if *a == actor && *d == b)
}

/// The cells of the straight line from `a` to `b`, both ends included.
fn line(a: CellXY, b: CellXY) -> Vec<CellXY> {
    let n = (b.x - a.x).abs().max((b.y - a.y).abs()).max(1);
    (0..=n)
        .map(|i| {
            CellXY::new(
                a.x + (b.x - a.x) * i / n,
                a.y + (b.y - a.y) * i / n,
            )
        })
        .collect()
}

/// Whether DETB at `det` covers cell `c` (S-0516 claim 40 and Terms: the
/// sensor disc of its range around its centre cell).
fn covered(det: CellXY, c: CellXY) -> bool {
    glasnost_sim::cloak::sensor_disc(DETECT)
        .into_iter()
        .any(|(dx, dy)| det.x + dx == c.x && det.y + dy == c.y)
}

// design: own AI (#454) and S-0516 claims 31 and 35: an AI spy with a disguise weapon that sees an infantry unit of the building's owner first attacks it, to take a disguise that fools the owner, rather than going in at once.
#[test]
fn a_spy_disguises_as_the_victims_infantry_first() {
    let g = ready();
    let (mut w, s, b) = scene(&g, "SPYD");
    // Out of the reach of DISGW, so the spy does not acquire it by itself (S-0309), but shown to house 0.
    let p = pawn(&mut w, 2, -14);
    let shown = at(home_of(&w), 2, -14);
    reveal(&mut w, shown);
    run_to_think(&mut w, 2);
    assert_eq!(w.obj(s).unwrap().cloak.disguise, None);
    let out = Brain::new(&w, &fixture::map(), ALPHA).think(&w);
    let mine = orders_to(&out, s);
    assert_eq!(mine, vec![attack(s, p)], "{out:?}");
    assert!(!out.iter().any(|o| enters(o, s, b)), "{out:?}");
}

// design: own AI (#454), S-0516 claims 31 and 35 and S-0510: played on, the spy hits the infantry, takes its disguise (the owner's house) and only then is sent into the building, which it infiltrates.
#[test]
fn a_spy_goes_in_once_disguised() {
    let g = ready();
    let (mut w, s, b) = scene(&g, "SPYD");
    let _p = pawn(&mut w, 4, -10);
    run_to_think(&mut w, 2);
    let mut br = Brain::new(&w, &fixture::map(), ALPHA);
    let mut sent: Option<Option<Disguise>> = None;
    let mut trace = Vec::new();
    for _ in 0..3000 {
        let out = br.think(&w);
        let mine = orders_to(&out, s);
        if !mine.is_empty() {
            let o = w.obj(s).unwrap();
            trace.push(format!("{} {:?} {:?}", w.tick, o.cloak.disguise, mine));
        }
        if sent.is_none() && mine.iter().any(|o| enters(o, s, b)) {
            sent = Some(w.obj(s).unwrap().cloak.disguise);
        }
        for o in out {
            orders::issue(&mut w, ALPHA, o);
        }
        testkit::run(&mut w, 1);
        if w.obj(s).is_none_or(|o| !o.on_map) {
            break;
        }
    }
    let trace = trace.join("\n");
    let d = sent.expect("the spy was sent into the building").expect("disguised");
    assert_eq!(d.house, Some(HouseId(1)), "{trace}");
    assert!(w.obj(s).is_none_or(|o| !o.on_map), "it went in: {trace}");
}

// design: own AI (#454): with no infantry of the owner (or of a house it allies) in sight, the spy does not wait: it goes in undisguised, as before.
#[test]
fn a_spy_with_no_infantry_to_copy_goes_in_undisguised() {
    let g = ready();
    let (mut w, s, b) = scene(&g, "SPYD");
    run_to_think(&mut w, 2);
    let out = Brain::new(&w, &fixture::map(), ALPHA).think(&w);
    assert!(out.iter().any(|o| enters(o, s, b)), "{out:?}");
}

// design: own AI (#454): a spy with no disguise weapon goes in at once, as before, whatever infantry it sees.
#[test]
fn a_spy_without_a_disguise_weapon_goes_in_as_before() {
    let g = ready();
    let (mut w, s, b) = scene(&g, "SPYU");
    let _p = pawn(&mut w, 4, -10);
    run_to_think(&mut w, 2);
    let out = Brain::new(&w, &fixture::map(), ALPHA).think(&w);
    assert!(out.iter().any(|o| enters(o, s, b)), "{out:?}");
}

// design: own AI (#454) and S-0516 claim 34 (a): a hit clears a spy's disguise on its way in; at the next think the spy is sent to disguise again.
#[test]
fn a_spy_whose_disguise_a_hit_cleared_disguises_again() {
    let g = ready();
    let (mut w, s, b) = scene(&g, "SPYD");
    let p = pawn(&mut w, 4, -10);
    disguise_as_victim(&mut w, s);
    let mut br = Brain::new(&w, &fixture::map(), ALPHA);
    run_to_think(&mut w, 2);
    let out = br.think(&w);
    assert!(out.iter().any(|o| enters(o, s, b)), "disguised, it goes in: {out:?}");
    for o in out {
        orders::issue(&mut w, ALPHA, o);
    }
    testkit::run(&mut w, 5);
    // spec: S-0516 claim 34 (a) (a hit that leaves the unit's state unchanged clears its disguise)
    glasnost_sim::cloak::disguise::hit_taken(&mut w, s, HitResult::Unchanged, 1);
    assert_eq!(w.obj(s).unwrap().cloak.disguise, None);
    run_to_think(&mut w, 1);
    let mut out = Vec::new();
    for _ in 0..THINK_PERIOD {
        out.extend(br.think(&w));
        testkit::run(&mut w, 1);
    }
    assert!(orders_to(&out, s).contains(&attack(s, p)), "{out:?}");
}

// design: own AI (#454) and S-0516 claims 35 and 40: a disguised spy whose straight way in crosses the seen disc of a disguise detector of the owner first walks to a cell off it, from where its way in crosses fewer covered cells.
#[test]
fn a_disguised_spy_goes_round_a_seen_detector() {
    let g = ready();
    let (mut w, s, b) = scene(&g, "SPYD");
    let home = home_of(&w);
    let det = at(home, 7, -8);
    testkit::spawn(&mut w, Kind::Building, "DETB", 1, det, 0);
    reveal(&mut w, det);
    disguise_as_victim(&mut w, s);
    run_to_think(&mut w, 2);
    let out = Brain::new(&w, &fixture::map(), ALPHA).think(&w);
    let mine = orders_to(&out, s);
    let goal = mine.iter().find_map(|o| match o {
        Order::Behaviour {
            behaviour: Behaviour::Move,
            dest: Some(Target::Cell(c)),
            ..
        } => Some(*c),
        _ => None,
    });
    let goal = goal.unwrap_or_else(|| panic!("a move first: {out:?}"));
    let from = w.obj(s).unwrap().pos.cell();
    let to = glasnost_sim::cloak::centre_cell(&w, b);
    let count = |cells: Vec<CellXY>| cells.into_iter().filter(|&c| covered(det, c)).count();
    let straight = count(line(from, to));
    let round = count(line(from, goal)) + count(line(goal, to));
    assert!(straight > 0, "the straight way crosses the disc");
    assert!(round < straight, "{goal:?}: {round} against {straight}");
    assert!(!covered(det, goal), "{goal:?}");
}

/// Every (script, difficulty) the registry offers.
fn every_choice() -> Vec<AiSeat> {
    REGISTRY
        .iter()
        .flat_map(|e| {
            AiDifficulty::ALL.into_iter().map(|d| AiSeat {
                script: e.id,
                difficulty: d,
            })
        })
        .collect()
}

/// For every script at every difficulty, from the same state over a think
/// period: the orders in `w` and in `w` with `hide` applied are the same.
fn same_orders(w: &World, hide: impl Fn(&mut World)) {
    let map = fixture::map();
    for seat in every_choice() {
        let mut w = w.clone();
        let base = Seat::start(seat, &w, &map, ALPHA);
        for _ in 0..THINK_PERIOD {
            let mut other = w.clone();
            hide(&mut other);
            let (mut a, mut b) = (base.clone(), base.clone());
            assert_eq!(
                format!("{:?}", a.think(&w, None, &mut ())),
                format!("{:?}", b.think(&other, None, &mut ())),
                "{seat:?}: same orders at tick {}",
                w.tick
            );
            testkit::run(&mut w, 1);
        }
    }
}

// design: own AI (#449, #454): what the spy copies is chosen from what the house sees: an infantry unit of the owner on a cell house 0 has not explored changes no order of any script at any difficulty.
#[test]
fn a_spy_copies_only_infantry_it_sees() {
    let g = ready();
    let (mut w, _, _) = scene(&g, "SPYD");
    run_to_think(&mut w, 2);
    let home = home_of(&w);
    let hidden = (2..12)
        .map(|dx| at(home, dx, -14))
        .find(|&c| !shroud::explored(&w, ALPHA, c) && w.cells.usable(c))
        .expect("an unexplored cell by the spy");
    same_orders(&w, |o: &mut World| {
        testkit::spawn(o, Kind::Infantry, "PAWN", 1, hidden, 0);
        assert!(!shroud::explored(o, ALPHA, hidden));
    });
}

// design: own AI (#449, #454): a disguised spy's way in is planned on what the house sees: a disguise detector of the owner on an unexplored cell of its straight way changes no order of any script at any difficulty, though the same detector shown to the house does.
#[test]
fn a_spy_avoids_only_detectors_it_sees() {
    let g = ready();
    let (mut w, s, b) = scene(&g, "SPYD");
    disguise_as_victim(&mut w, s);
    run_to_think(&mut w, 2);
    let from = w.obj(s).unwrap().pos.cell();
    let to = glasnost_sim::cloak::centre_cell(&w, b);
    let hidden = line(from, to)
        .into_iter()
        .find(|&c| !shroud::explored(&w, ALPHA, c) && w.cells.usable(c))
        .expect("an unexplored cell on the spy's way");
    let place = |o: &mut World| {
        testkit::spawn(o, Kind::Building, "DETB", 1, hidden, 0);
    };
    same_orders(&w, |o: &mut World| {
        place(o);
        assert!(!shroud::explored(o, ALPHA, hidden));
    });
    // Shown, the same detector turns the spy.
    let mut shown = w.clone();
    place(&mut shown);
    reveal(&mut shown, hidden);
    run_to_think(&mut shown, 1);
    let mut plain = w.clone();
    run_to_think(&mut plain, 1);
    let a = orders_to(&Brain::new(&plain, &fixture::map(), ALPHA).think(&plain), s);
    let b2 = orders_to(&Brain::new(&shown, &fixture::map(), ALPHA).think(&shown), s);
    let moves = |v: &[Order]| {
        v.iter().any(|o| {
            matches!(o, Order::Behaviour { behaviour: Behaviour::Move, dest: Some(Target::Cell(_)), .. })
        })
    };
    assert!(!moves(&a), "unseen, the spy goes straight: {a:?}");
    assert!(moves(&b2), "seen, it goes round: {b2:?}");
}

// spec: S-0516 claim 32 (a unit whose type can disguise and disguises permanently is created disguised: the side default, its owner the disguise house): a person's spy of such a type and an AI house's are both created disguised.
#[test]
fn a_permanent_disguiser_is_created_disguised_for_every_owner() {
    let mut s = fixture::setup(5);
    // House 0 a person's, house 1 an AI house.
    s.houses[0].ai = false;
    let mut w = new_game(rules(), &fixture::map(), fixture::catalogue(), &s).expect("it loads");
    assert!(!w.houses[0].econ.setup.ai && w.houses[1].econ.setup.ai);
    let home = home_of(&w);
    let pawn = w.rules().index_of(Kind::Infantry, "PAWN").unwrap();
    for h in [0u8, 1] {
        let id = testkit::spawn(
            &mut w,
            Kind::Infantry,
            "SPYP",
            h,
            at(home, 3 + i32::from(h), -6),
            0,
        );
        let d = w.obj(id).unwrap().cloak.disguise;
        assert_eq!(
            d.map(|d| (d.what, d.house)),
            Some((Some(DisguiseAs::Infantry(pawn)), Some(HouseId(h)))),
            "house {h}"
        );
    }
}
