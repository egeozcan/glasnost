//! The own AI and the psychic dominator (S-0612, PLAN §4.7): a ready
//! dominator is fired at a group of seen enemy units, never where the own
//! and allied value in the reach is at least the enemy value, not at a
//! group smaller than `dominator::MIN_UNITS`, and not while a domination is
//! in force; a launch takes the group over for the house; the house builds
//! the giving building once its tech tree allows. The tests use the
//! invented types of the synthetic M7b skirmish (DOMB giving DOM, Alpha
//! only; CROWD, an unarmed walker, as the target units). Alpha's start
//! discs are taken off the map first.

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

fn dom(w: &World) -> u32 {
    w.rules()
        .superweapons
        .types
        .iter()
        .position(|t| t.name == "DOM")
        .expect("the fixture has the dominator") as u32
}

fn ready(w: &World) -> bool {
    glasnost_sim::superweapons::charge::get(w, HouseId(0), dom(w)).is_some_and(|r| r.ready)
}

/// The skirmish with DOMB standing for Alpha and the dominator ready, at a
/// thinking tick, before Alpha's own brain has thought again.
fn armed() -> World {
    let g = game();
    let mut w = g.world.clone();
    let h = home(&w);
    let at = (3..14)
        .flat_map(|d| [CellXY::new(h.x - d, h.y), CellXY::new(h.x, h.y - d)])
        .find(|&c| free(&w, c) && free(&w, CellXY::new(c.x + 1, c.y + 1)))
        .expect("room for the giving building");
    testkit::spawn(&mut w, Kind::Building, "DOMB", 0, at, 0);
    for _ in 0..3000 {
        if ready(&w) {
            break;
        }
        tick::step(&mut w);
    }
    assert!(ready(&w), "the dominator charged (S-0512)");
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
    let i = dom(w);
    out.iter()
        .filter_map(|o| match *o {
            Order::FireSuperweapon { sw, cell } if sw == i => Some(cell),
            _ => None,
        })
        .collect()
}

/// CROWD walkers of `house` on the cells `(dx, dy)` from Alpha's home.
fn group(w: &mut World, house: u8, cells: &[(i32, i32)]) -> Vec<ObjectId> {
    cells
        .iter()
        .map(|&(dx, dy)| spawn(w, Kind::Infantry, "CROWD", house, dx, dy))
        .collect()
}

const GROUP: [(i32, i32); 3] = [(6, 6), (7, 6), (6, 7)];

/// Whether every cell of `cells` (from Alpha's home) is in the reach of a
/// launch at `c` (S-0612 claim 11).
fn in_reach(w: &World, c: CellXY, cells: &[(i32, i32)]) -> bool {
    let h = home(w);
    let swept: Vec<CellXY> = glasnost_sim::domination::swept(w, c)
        .into_iter()
        .map(|(x, _)| x)
        .collect();
    cells
        .iter()
        .all(|&(dx, dy)| swept.contains(&CellXY::new(h.x + dx, h.y + dy)))
}

#[test]
fn fires_at_a_group_of_seen_enemy_units() {
    let mut w = armed();
    group(&mut w, 1, &GROUP);
    // A lone enemy walker elsewhere is no target of its own.
    spawn(&mut w, Kind::Infantry, "CROWD", 1, -8, 8);
    let mut br = brain(&w);
    let out = br.think(&w);
    let cells = fired(&out, &w);
    assert_eq!(cells.len(), 1, "one launch");
    assert!(
        in_reach(&w, cells[0], &GROUP),
        "the whole group is in reach"
    );
    assert_eq!(br.special_uses().dominator_launches, 1);
}

#[test]
fn holds_fire_where_it_takes_more_own_value_than_enemy_value() {
    let mut w = armed();
    group(&mut w, 1, &GROUP);
    // Four own walkers of the same type in every reach that holds the group.
    let own = [(6, 5), (7, 7), (5, 6), (7, 5)];
    group(&mut w, 0, &own);
    let h = home(&w);
    for (dx, dy) in GROUP {
        assert!(in_reach(&w, CellXY::new(h.x + dx, h.y + dy), &own));
    }
    let mut br = brain(&w);
    let out = br.think(&w);
    // spec: S-0612 claim 13 (own objects are taken over too, no owner test) and claim 9 (the area damage hurts them)
    assert!(fired(&out, &w).is_empty());
    assert_eq!(br.special_uses().dominator_launches, 0);
}

#[test]
fn fires_when_the_enemy_value_outweighs_the_own() {
    let mut w = armed();
    group(&mut w, 1, &[(6, 6), (7, 6), (6, 7), (7, 7)]);
    group(&mut w, 0, &[(5, 5)]);
    let mut br = brain(&w);
    assert_eq!(fired(&br.think(&w), &w).len(), 1);
}

#[test]
fn holds_fire_at_a_small_group() {
    let mut w = armed();
    group(&mut w, 1, &GROUP[..glasnost_ai::dominator::MIN_UNITS - 1]);
    let mut br = brain(&w);
    let out = br.think(&w);
    assert!(fired(&out, &w).is_empty());
}

#[test]
fn holds_fire_while_a_domination_is_in_force() {
    let mut w = armed();
    group(&mut w, 1, &GROUP);
    let h = home(&w);
    let i = dom(&w);
    glasnost_sim::domination::launch(&mut w, HouseId(1), i, CellXY::new(h.x - 12, h.y + 12));
    assert!(glasnost_sim::domination::in_force(&w));
    let mut br = brain(&w);
    let out = br.think(&w);
    // spec: S-0612 claim 3 (a launch while a domination is in force is refused)
    assert!(fired(&out, &w).is_empty());
}

#[test]
fn a_launch_takes_the_group_over() {
    let mut w = armed();
    let ids = group(&mut w, 1, &GROUP);
    let mut br = brain(&w);
    for o in br.think(&w) {
        orders::issue(&mut w, HouseId(0), o);
    }
    for _ in 0..60 {
        tick::step(&mut w);
    }
    // spec: S-0612 claims 4 and 14 (the sweep changes the owner and makes the control permanent)
    for id in ids {
        let o = w.obj(id).expect("still in play");
        assert_eq!(o.owner, Some(HouseId(0)));
        assert!(glasnost_sim::mind::permanent(&w, id));
    }
}

// A person climbs the tech tree to the superweapon: in an ordinary game
// Alpha's brain builds DOMB (the brain's superweapon step finds it through
// the type it names, S-0512 Terms), and Beta, outside its required houses,
// never does.
#[test]
fn builds_the_dominator_building() {
    let mut g = Game::new(
        kit::rules(),
        &kit::map(),
        kit::MAP_NAME,
        fixture::catalogue(),
        kit::setup(5),
    )
    .expect("the M7b skirmish loads");
    let domb = g.world.rules().index_of(Kind::Building, "DOMB").unwrap();
    let count = |w: &World, h: u8| {
        w.objects
            .values()
            .filter(|o| o.owner == Some(HouseId(h)) && o.on_map && o.health > 0)
            .filter(|o| o.kind == Kind::Building && o.type_index == domb)
            .count()
    };
    while g.world.tick < 12_000 && g.outcome().is_none() && count(&g.world, 0) == 0 {
        g.step();
    }
    assert!(
        count(&g.world, 0) > 0,
        "DOMB stands by tick {}",
        g.world.tick
    );
    assert_eq!(count(&g.world, 1), 0);
}

#[test]
fn a_beta_brain_never_fires_it() {
    let mut g = game();
    for _ in 0..900 {
        g.step();
    }
    let u = g.special_uses();
    assert!(
        u.iter()
            .filter(|(h, _)| *h == HouseId(1))
            .all(|(_, x)| x.dominator_launches == 0)
    );
}
