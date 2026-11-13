//! The own AI and the psychic dominator (S-0612, PLAN §4.7): a ready
//! dominator is fired at the best target it sees by value: the seen enemy
//! units it would take and the seen enemy objects its area damage would
//! destroy, against the allied units it would take and the own objects its
//! damage would destroy (the house's own units change no owner, claim 14);
//! a group smaller than `dominator::MIN_UNITS` is fired at too (M7b AI
//! round 3). A cell that takes at least one enemy unit beats every cell
//! that takes none, refused units and units the damage destroys count no
//! take, so does a unit on the move, and the aim looks off a unit's own
//! cell for one (M8 step B3); with
//! nothing to take it holds the shot for `dominator::HOLD_PERCENT` of the
//! charge time and then fires for the damage, at enemy buildings too.
//! Never while a domination is in force; a launch takes the group over for
//! the house; the house builds the giving building once its tech tree
//! allows. The tests use the
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
    game_with(kit::rules())
}

/// The skirmish on `rules` (the scenario's, or a test's own copy).
fn game_with(rules: std::sync::Arc<glasnost_sim::rules::SimRules>) -> Game {
    game_on(rules, &kit::map(), kit::setup(5))
}

/// [`game_with`] on map `map` with setup `setup`.
fn game_on(
    rules: std::sync::Arc<glasnost_sim::rules::SimRules>,
    map: &glasnost_map::Map,
    setup: glasnost_sim::world::GameSetup,
) -> Game {
    let mut g = Game::new(rules, map, kit::MAP_NAME, fixture::catalogue(), setup)
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
    armed_on(game())
}

/// [`armed`] on the skirmish `g`.
fn armed_on(g: Game) -> World {
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

// S-0612 claim 14 (the house's own objects change no owner) and claim 9 (the area damage hurts them): own units in the reach that the damage spares do not stop the launch.
#[test]
fn own_units_the_damage_spares_do_not_stop_it() {
    let mut w = armed();
    group(&mut w, 1, &GROUP);
    let own = [(6, 5), (7, 7), (5, 6), (7, 5)];
    group(&mut w, 0, &own);
    let mut br = brain(&w);
    assert_eq!(fired(&br.think(&w), &w).len(), 1);
}

// S-0612 claim 9 (the area damage hurts every object in its radius, the house's own too) and claim 11 (the swept cells): where every cell from which the sweep reaches the enemy walker has a wounded own walker in the damage's radius, the damage would destroy at least as much own value as the launch takes, and it holds fire.
#[test]
fn holds_fire_where_the_damage_destroys_more_own_value_than_it_takes() {
    let mut w = armed();
    group(&mut w, 1, &[(6, 6)]);
    let r = w.rules().superweapons.domination.range;
    let around: Vec<(i32, i32)> = (-r..=r)
        .flat_map(|dx| (-r..=r).map(move |dy| (dx, dy)))
        .filter(|&(dx, dy)| (dx, dy) != (0, 0))
        // spec: S-0303 Terms (the ring metric) and S-0612 claim 11 (the swept rings up to the capture range)
        .filter(|&(dx, dy)| {
            let (a, b) = (dx.abs(), dy.abs());
            a.max(b) + a.min(b) / 2 <= r
        })
        .map(|(dx, dy)| (6 + dx, 6 + dy))
        .collect();
    let own = group(&mut w, 0, &around);
    // Wounded below the launch's damage, the own walkers would die.
    for id in own {
        w.objects.get_mut(&id).unwrap().health = 1;
    }
    let mut br = brain(&w);
    assert!(
        fired(&br.think(&w), &w).is_empty(),
        "{:?}",
        glasnost_ai::dominator::probe(&w, HouseId(0))
    );
    assert_eq!(br.special_uses().dominator_launches, 0);
}

#[test]
fn fires_when_the_enemy_value_outweighs_the_own() {
    let mut w = armed();
    group(&mut w, 1, &[(6, 6), (7, 6), (6, 7), (7, 7)]);
    let own = group(&mut w, 0, &[(5, 5)]);
    w.objects.get_mut(&own[0]).unwrap().health = 1;
    let mut br = brain(&w);
    assert_eq!(fired(&br.think(&w), &w).len(), 1);
}

// PLAN §4.7 (M7b AI round 3): a person fires a ready dominator at the best target it sees, a group smaller than `MIN_UNITS` included.
#[test]
fn fires_at_a_small_group() {
    let mut w = armed();
    group(&mut w, 1, &GROUP[..glasnost_ai::dominator::MIN_UNITS - 1]);
    let mut br = brain(&w);
    let out = br.think(&w);
    let cells = fired(&out, &w);
    assert_eq!(cells.len(), 1);
    assert!(in_reach(&w, cells[0], &GROUP[..2]));
}

// S-0612 claims 9 and 13 (buildings are never taken, but the area damage hurts them): with no enemy unit in sight a ready dominator is fired at a seen enemy building.
#[test]
fn fires_at_a_seen_enemy_building_with_no_unit_in_sight() {
    let mut w = armed();
    let b = spawn(&mut w, Kind::Building, "POWR", 1, 12, 12);
    let at = w.obj(b).unwrap().pos.cell();
    // Alpha has scouted it.
    glasnost_sim::econ::shroud::area_reveal(
        &mut w,
        HouseId(0),
        glasnost_sim::world::Pos::centre(at, 0),
        4,
    );
    // Nothing to take: the shot is held at first (M8 step B3).
    assert!(fired(&brain(&w).think(&w), &w).is_empty(), "held");
    hold_over(&mut w);
    let mut br = brain(&w);
    let cells = fired(&br.think(&w), &w);
    assert_eq!(
        cells.len(),
        1,
        "{:?}",
        glasnost_ai::dominator::probe(&w, HouseId(0))
    );
    let (r, _) =
        glasnost_sim::combat::area::radii(&w, w.rules().superweapons.domination.warhead.unwrap());
    let c = cells[0];
    let d2 = i64::from(c.x - at.x).pow(2) + i64::from(c.y - at.y).pow(2);
    assert!(
        d2 * 256 * 256 <= i64::from(r) * i64::from(r) * 4,
        "aimed at the building"
    );
}

/// Runs the world (no brain) to the first thinking tick at which the ready
/// dominator has been ready for `HOLD_PERCENT` of its charge time.
fn hold_over(w: &mut World) {
    let i = dom(w);
    let charge = w.rules().superweapons.types[i as usize].charge;
    let r = glasnost_sim::superweapons::charge::get(w, HouseId(0), i).unwrap();
    let until = r.ready_tick + charge * glasnost_ai::dominator::HOLD_PERCENT / 100;
    while w.tick < until {
        tick::step(w);
    }
    settle(w);
    assert!(ready(w));
}

/// Enemy objects of house 1 at `(dx, dy)` from Alpha's home that the house
/// has scouted.
fn scouted(w: &mut World, kind: Kind, name: &str, dx: i32, dy: i32) -> ObjectId {
    let id = spawn(w, kind, name, 1, dx, dy);
    let at = w.obj(id).unwrap().pos.cell();
    glasnost_sim::econ::shroud::area_reveal(
        w,
        HouseId(0),
        glasnost_sim::world::Pos::centre(at, 0),
        4,
    );
    id
}

// M8 step B3 (S-0612 claims 9 and 13): one takeable enemy walker beats an enemy factory the damage would destroy, though the factory is worth more.
#[test]
fn a_takeable_unit_beats_a_richer_damage_cell() {
    let mut w = armed();
    let b = scouted(&mut w, Kind::Building, "FACT", -12, 10);
    w.objects.get_mut(&b).unwrap().health = 1;
    group(&mut w, 1, &[(6, 6)]);
    let mut br = brain(&w);
    let cells = fired(&br.think(&w), &w);
    assert_eq!(cells.len(), 1);
    assert!(in_reach(&w, cells[0], &[(6, 6)]), "aimed at the walker");
    let taken = glasnost_sim::domination::preview(&w, HouseId(0), cells[0]);
    assert!(
        taken
            .iter()
            .any(|&id| w.obj(id).unwrap().owner == Some(HouseId(1)))
    );
}

// S-0612 claim 13 and S-0610 claim 1 (a type with psionics immunity is not taken): immune enemy walkers count as no take, so the shot is held, and then fired for the area damage (claim 9).
#[test]
fn immune_units_count_no_take() {
    let mut rs = kit::ruleset();
    rs.sim
        .infantry
        .get_mut("CROWD")
        .unwrap()
        .combat
        .immune_to_psionics = Some(true);
    let rules = std::sync::Arc::new(glasnost_sim::rules::SimRules::new(&rs, kit::MAP_NAME));
    let mut w = armed_on(game_with(rules));
    group(&mut w, 1, &GROUP);
    let p = glasnost_ai::dominator::probe(&w, HouseId(0));
    assert_eq!(p.dominable, 0, "{p:?}");
    assert!(p.refused.iter().all(|&x| x == "immune") && p.refused.len() >= GROUP.len());
    assert_eq!(p.best.map(|(_, _, n)| n), Some(0), "the damage alone");
    assert!(fired(&brain(&w).think(&w), &w).is_empty(), "held");
    hold_over(&mut w);
    let cells = fired(&brain(&w).think(&w), &w);
    assert_eq!(cells.len(), 1, "fired for the damage after the hold");
}

// S-0612 claim 13 (a unit that hovers like a balloon is not taken): a seen enemy start disc counts as no take.
#[test]
fn a_balloon_unit_counts_no_take() {
    let mut w = armed();
    scouted(&mut w, Kind::Vehicle, "DISC", 6, 6);
    let p = glasnost_ai::dominator::probe(&w, HouseId(0));
    assert_eq!((p.units, p.dominable), (1, 0), "{p:?}");
    assert!(fired(&brain(&w).think(&w), &w).is_empty(), "held");
}

// S-0612 claim 10 (an object the area damage destroys is not taken over): a wounded walker the damage would kill at its own cell is taken from a cell whose damage misses it.
#[test]
fn aims_where_the_damage_spares_the_unit_it_takes() {
    let mut w = armed();
    let id = group(&mut w, 1, &[(6, 6)])[0];
    w.objects.get_mut(&id).unwrap().health = 1;
    let mut br = brain(&w);
    let out = br.think(&w);
    let cells = fired(&out, &w);
    assert_eq!(
        cells.len(),
        1,
        "{:?}",
        glasnost_ai::dominator::probe(&w, HouseId(0))
    );
    let c = cells[0];
    assert!(glasnost_sim::domination::preview(&w, HouseId(0), c).contains(&id));
    let (r, _) =
        glasnost_sim::combat::area::radii(&w, w.rules().superweapons.domination.warhead.unwrap());
    let p = w.obj(id).unwrap().pos;
    let (dx, dy) = (
        i64::from(p.x) - (i64::from(c.x) * 256 + 128),
        i64::from(p.y) - (i64::from(c.y) * 256 + 128),
    );
    assert!(
        dx * dx + dy * dy > i64::from(r) * i64::from(r),
        "outside the damage's radius"
    );
    // The sweep takes it.
    for o in out {
        orders::issue(&mut w, HouseId(0), o);
    }
    for _ in 0..60 {
        tick::step(&mut w);
    }
    assert_eq!(w.obj(id).map(|o| o.owner), Some(Some(HouseId(0))));
}

// S-0612 claim 10 and S-0303 claim 11 (a death weapon a hit sets off runs in the middle of the detonation): a walker inside a blast that destroys something (here a wounded enemy disc in its cell, which no launch takes) counts no take, so the aim takes it from a cell whose blast misses them both.
#[test]
fn a_unit_in_a_blast_that_destroys_counts_no_take() {
    let mut w = armed();
    let walker = group(&mut w, 1, &[(6, 6)])[0];
    let disc = scouted(&mut w, Kind::Vehicle, "DISC", 6, 6);
    w.objects.get_mut(&disc).unwrap().health = 1;
    let out = brain(&w).think(&w);
    let cells = fired(&out, &w);
    assert_eq!(
        cells.len(),
        1,
        "{:?}",
        glasnost_ai::dominator::probe(&w, HouseId(0))
    );
    let c = cells[0];
    assert!(glasnost_sim::domination::preview(&w, HouseId(0), c).contains(&walker));
    let h = w.rules().superweapons.domination.warhead.unwrap();
    let (r, _) = glasnost_sim::combat::area::radii(&w, h);
    let p = glasnost_sim::superweapons::cell_point(&w, c, false);
    assert!(
        glasnost_sim::combat::area::object_distance(&w, p, walker) > r,
        "the walker is outside the blast"
    );
}

// M8 step B3 (S-0612 claim 4: the sweep comes some time after the launch): an enemy walker on the move counts no take, so with nothing else in sight the shot is held; standing still, it is taken.
#[test]
fn a_unit_on_the_move_counts_no_take() {
    let mut w = armed();
    let id = group(&mut w, 1, &[(6, 6)])[0];
    let h = home(&w);
    orders::issue(
        &mut w,
        HouseId(1),
        Order::Behaviour {
            actor: id,
            behaviour: glasnost_sim::rules::Behaviour::Move,
            attack_move: false,
            target: None,
            dest: Some(glasnost_sim::world::Target::Cell(CellXY::new(
                h.x + 12,
                h.y + 12,
            ))),
        },
    );
    settle(&mut w);
    assert!(w.obj(id).unwrap().dest.is_some(), "on the move");
    let p = glasnost_ai::dominator::probe(&w, HouseId(0));
    assert_eq!((p.dominable, p.moving), (1, 1), "{p:?}");
    assert!(fired(&brain(&w).think(&w), &w).is_empty(), "held");
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

// design: own AI (#33) and S-0612 claim 9 (the area damage hurts every object in its radius): the damage to an ally's objects the launch does not take counts as loss, as the house's own does. Wounded allied walkers immune to the takeover (S-0612 claim 13) stand on every cell around a lone enemy walker; the damage would destroy more allied value than the launch gains, and it holds fire, after the hold too.
#[test]
fn holds_fire_where_the_damage_destroys_an_allys_units() {
    use glasnost_sim::econ::setup::{self, HouseValues, Player, Settings};
    let mut rs = kit::ruleset();
    let mut ward = rs.sim.infantry.get("CROWD").unwrap().clone();
    ward.combat.immune_to_psionics = Some(true);
    rs.sim.infantry.insert("WARD".into(), ward);
    rs.registries
        .entry("infantry".into())
        .or_default()
        .push("WARD".into());
    let rules = std::sync::Arc::new(glasnost_sim::rules::SimRules::new(&rs, kit::MAP_NAME));
    // A third start, north of the middle, for Gamma: a house with no AI on
    // Alpha's lobby team, so allied (S-0412 claim 8).
    let mut map = kit::map();
    let c = fixture::at(0, -30);
    map.waypoints.insert(2, (c.x as i16, c.y as i16));
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
        &rules,
        &[
            player("Alpha", "Alphaland", 0, true, 1),
            player("Beta", "Betaland", 1, true, 2),
            player("Gamma", "Alphaland", 2, false, 1),
        ],
        Settings {
            credits: Some(glasnost_ai::fixture_m7a::CREDITS),
            ..Default::default()
        },
        0,
        5,
    );
    let mut w = armed_on(game_on(rules, &map, s));
    assert!(w.allied(Some(HouseId(0)), Some(HouseId(2))), "allied");
    group(&mut w, 1, &[(6, 6)]);
    let r = w.rules().superweapons.domination.range;
    let around: Vec<(i32, i32)> = (-r..=r)
        .flat_map(|dx| (-r..=r).map(move |dy| (dx, dy)))
        .filter(|&(dx, dy)| (dx, dy) != (0, 0))
        // spec: S-0303 Terms (the ring metric) and S-0612 claim 11 (the swept rings up to the capture range)
        .filter(|&(dx, dy)| {
            let (a, b) = (dx.abs(), dy.abs());
            a.max(b) + a.min(b) / 2 <= r
        })
        .map(|(dx, dy)| (6 + dx, 6 + dy))
        .collect();
    for (dx, dy) in around {
        let id = spawn(&mut w, Kind::Infantry, "WARD", 2, dx, dy);
        w.objects.get_mut(&id).unwrap().health = 1;
    }
    let p = glasnost_ai::dominator::probe(&w, HouseId(0));
    assert!(
        fired(&brain(&w).think(&w), &w).is_empty(),
        "fired at once: {p:?}"
    );
    hold_over(&mut w);
    let p = glasnost_ai::dominator::probe(&w, HouseId(0));
    assert!(
        fired(&brain(&w).think(&w), &w).is_empty(),
        "fired after the hold: {p:?}"
    );
}
