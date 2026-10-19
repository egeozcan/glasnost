//! The own AI and the genetic mutator (S-0611, PLAN §4.7): a ready mutator
//! is fired at the best group of seen enemy infantry by value: the enemy
//! gain must be at least `mutator::OUTWEIGH` times the own loss, where an
//! own infantry unit in the reach costs its value less the made unit's
//! (S-0611 claims 4, 8 and 9), and a group smaller than
//! `mutator::MIN_GROUP` is fired at too (M7b AI round 3); a launch turns the
//! group into the house's infantry; the house builds the giving building once its tech tree
//! allows. The tests use the invented types of the synthetic M7b skirmish
//! (GENEB giving GENE, Alpha only; MADMAN, the made type, as the target
//! infantry). Alpha's start discs are taken off the map first.

use glasnost_ai::brain::Brain;
use glasnost_ai::fixture;
use glasnost_ai::fixture_m7b as kit;
use glasnost_ai::game::Game;
use glasnost_map::CellXY;
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::Kind;
use glasnost_sim::world::{HouseId, ObjectId, World};
use glasnost_sim::{objects, testkit, tick};

fn game() -> Game {
    let mut g = Game::new(
        kit::rules(),
        &kit::map(),
        kit::MAP_NAME,
        fixture::catalogue(),
        kit::setup(5),
    )
    .expect("the M7b skirmish loads");
    let disc = g.world.rules().index_of(Kind::Vehicle, "DISC").unwrap();
    let discs: Vec<ObjectId> = g
        .world
        .objects
        .values()
        .filter(|o| o.kind == Kind::Vehicle && o.type_index == disc)
        .map(|o| o.id)
        .collect();
    for d in discs {
        objects::remove(&mut g.world, d);
    }
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

fn free(w: &World, c: CellXY) -> bool {
    w.cells.usable(c) && w.objects.values().all(|o| o.pos.cell() != c)
}

fn spawn(w: &mut World, kind: Kind, name: &str, house: u8, dx: i32, dy: i32) -> ObjectId {
    let h = home(w);
    testkit::spawn(w, kind, name, house, CellXY::new(h.x + dx, h.y + dy), 0)
}

fn gene(w: &World) -> u32 {
    w.rules()
        .superweapons
        .types
        .iter()
        .position(|t| t.name == "GENE")
        .expect("the fixture has the mutator") as u32
}

fn ready(w: &World) -> bool {
    glasnost_sim::superweapons::charge::get(w, HouseId(0), gene(w)).is_some_and(|r| r.ready)
}

/// The skirmish with GENEB standing for Alpha and the mutator ready, at a
/// thinking tick, before Alpha's own brain has thought again.
fn armed() -> World {
    let g = game();
    let mut w = g.world.clone();
    let h = home(&w);
    let at = (3..14)
        .flat_map(|d| [CellXY::new(h.x - d, h.y), CellXY::new(h.x, h.y - d)])
        .find(|&c| free(&w, c) && free(&w, CellXY::new(c.x + 1, c.y + 1)))
        .expect("room for the giving building");
    testkit::spawn(&mut w, Kind::Building, "GENEB", 0, at, 0);
    for _ in 0..3000 {
        if ready(&w) {
            break;
        }
        tick::step(&mut w);
    }
    assert!(ready(&w), "the mutator charged (S-0512)");
    settle(&mut w);
    w
}

fn settle(w: &mut World) {
    testkit::run(w, 1);
    while w.tick % 15 != 0 {
        testkit::run(w, 1);
    }
}

fn brain(w: &World) -> Brain {
    Brain::new(w, &kit::map(), HouseId(0))
}

fn fired(out: &[Order], w: &World) -> Vec<CellXY> {
    let i = gene(w);
    out.iter()
        .filter_map(|o| match *o {
            Order::FireSuperweapon { sw, cell } if sw == i => Some(cell),
            _ => None,
        })
        .collect()
}

/// Beta infantry on the cells `(dx, dy)` from Alpha's home.
fn group(w: &mut World, house: u8, cells: &[(i32, i32)]) -> Vec<ObjectId> {
    cells
        .iter()
        .map(|&(dx, dy)| spawn(w, Kind::Infantry, "MADMAN", house, dx, dy))
        .collect()
}

const GROUP: [(i32, i32); 3] = [(6, 6), (7, 6), (6, 7)];

#[test]
fn fires_at_a_dense_group_of_seen_enemy_infantry() {
    let mut w = armed();
    group(&mut w, 1, &GROUP);
    // A lone enemy walker elsewhere is no target of its own.
    spawn(&mut w, Kind::Infantry, "MADMAN", 1, -6, 6);
    let mut br = brain(&w);
    let out = br.think(&w);
    let cells = fired(&out, &w);
    assert_eq!(cells.len(), 1, "one launch");
    let h = home(&w);
    let block: Vec<CellXY> = glasnost_sim::superweapons::block(&w, cells[0])
        .into_iter()
        .map(|(_, c)| c)
        .collect();
    for (dx, dy) in GROUP {
        assert!(
            block.contains(&CellXY::new(h.x + dx, h.y + dy)),
            "the whole group is in reach"
        );
    }
    assert_eq!(br.special_uses().mutator_launches, 1);
}

/// `w` with a conversion animation that makes no unit (S-0611 F4: an
/// index of −1), so that an own victim is a whole loss.
fn making_nothing(w: &mut World) {
    let mut r = (*w.rules_arc()).clone();
    let anim = r
        .superweapons
        .mutation
        .anim
        .expect("the fixture's conversion animation");
    r.animations[anim as usize].make_infantry = -1;
    w.rules = Some(std::sync::Arc::new(r));
}

// S-0611 claim 4 (every listed infantry of any house in the block is hit) and PLAN §4.7 (M7b AI round 3): with no unit made, three enemy infantry outweigh one own unit of the same value at least twice over; the launch goes ahead.
#[test]
fn fires_when_the_enemy_gain_outweighs_the_own_loss() {
    let mut w = armed();
    making_nothing(&mut w);
    group(&mut w, 1, &GROUP);
    group(&mut w, 0, &[(7, 7)]);
    let mut br = brain(&w);
    assert_eq!(fired(&br.think(&w), &w).len(), 1);
    assert_eq!(br.special_uses().mutator_launches, 1);
}

// S-0611 claim 4 and PLAN §4.7 (M7b AI round 3): with no unit made, three enemy infantry against two own of the same value is no clear gain; it holds fire.
#[test]
fn holds_fire_when_the_own_loss_is_not_clearly_outweighed() {
    let mut w = armed();
    making_nothing(&mut w);
    group(&mut w, 1, &GROUP);
    // Two own units on the one free cell that every reach holding an enemy
    // unit also holds.
    group(&mut w, 0, &[(7, 7), (7, 7)]);
    let mut br = brain(&w);
    assert!(fired(&br.think(&w), &w).is_empty());
    assert_eq!(br.special_uses().mutator_launches, 0);
}

// S-0611 claims 8 and 9 (the conversion animation's owner is the launching house, which gets the made unit): an own victim of the made type costs nothing, so own infantry of that type does not stop the launch.
#[test]
fn own_infantry_of_the_made_type_costs_nothing() {
    let mut w = armed();
    group(&mut w, 1, &GROUP);
    group(&mut w, 0, &[(7, 7), (5, 5)]);
    let mut br = brain(&w);
    assert_eq!(fired(&br.think(&w), &w).len(), 1);
}

// PLAN §4.7 (M7b AI round 3): a person fires a ready mutator at the best group it sees, a group smaller than `MIN_GROUP` included.
#[test]
fn fires_at_a_small_group() {
    let mut w = armed();
    group(&mut w, 1, &GROUP[..glasnost_ai::mutator::MIN_GROUP - 1]);
    let mut br = brain(&w);
    assert_eq!(fired(&br.think(&w), &w).len(), 1);
}

#[test]
fn a_launch_turns_the_group_into_the_house_infantry() {
    let mut w = armed();
    let ids = group(&mut w, 1, &GROUP);
    let madman = w.rules().index_of(Kind::Infantry, "MADMAN").unwrap();
    let ours = |w: &World| {
        w.objects
            .values()
            .filter(|o| o.kind == Kind::Infantry && o.type_index == madman)
            .filter(|o| o.owner == Some(HouseId(0)))
            .count()
    };
    let before = ours(&w);
    let mut br = brain(&w);
    for o in br.think(&w) {
        orders::issue(&mut w, HouseId(0), o);
    }
    for _ in 0..300 {
        tick::step(&mut w);
    }
    // spec: S-0611 claims 7 to 10 (the victims leave play; each animation makes the made type for the launching house)
    assert!(
        ids.iter().all(|&id| w.obj(id).is_none()),
        "every victim converted"
    );
    assert_eq!(ours(&w), before + GROUP.len());
}

// A person climbs the tech tree to the superweapon: in an ordinary game
// Alpha's brain builds GENEB (the brain's superweapon step finds it through
// the type it names, S-0512 Terms), and Beta, outside its required houses,
// never does.
#[test]
fn builds_the_mutator_building() {
    let mut g = Game::new(
        kit::rules(),
        &kit::map(),
        kit::MAP_NAME,
        fixture::catalogue(),
        kit::setup(5),
    )
    .expect("the M7b skirmish loads");
    let geneb = g.world.rules().index_of(Kind::Building, "GENEB").unwrap();
    let count = |w: &World, h: u8| {
        w.objects
            .values()
            .filter(|o| o.owner == Some(HouseId(h)) && o.on_map && o.health > 0)
            .filter(|o| o.kind == Kind::Building && o.type_index == geneb)
            .count()
    };
    while g.world.tick < 12_000 && g.outcome().is_none() && count(&g.world, 0) == 0 {
        g.step();
    }
    assert!(
        count(&g.world, 0) > 0,
        "GENEB stands by tick {}",
        g.world.tick
    );
    assert_eq!(count(&g.world, 1), 0);
}

#[test]
fn a_beta_brain_never_builds_or_fires_it() {
    // GENEB is Alpha's only (its required houses): Beta's brain counts no
    // mutator launch over a stretch of play.
    let mut g = game();
    for _ in 0..900 {
        g.step();
    }
    let u = g.special_uses();
    assert!(
        u.iter()
            .filter(|(h, _)| *h == HouseId(1))
            .all(|(_, x)| x.mutator_launches == 0)
    );
}
