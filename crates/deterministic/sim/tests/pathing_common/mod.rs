//! The synthetic pathing scenario set of M8 step C7 part 4 (our own pathing
//! improvements, `docs/m8-claims/c7-pathing.md`): maps, types and layouts of
//! our own (no game data), a runner that plays one scenario and counts what
//! a person would notice, and the two rule sets the scenarios run under:
//! the reference (every quirk switch at its default) and the reference with
//! our patch layer *Pathing and other fixes* (`data/layers/play-fixes.jsonc`)
//! applied through `Layers::apply_quirks`.
//!
//! Every name and number here is invented: the scenarios are design
//! fixtures, not measurements of the original.

#![allow(dead_code)]
#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use std::sync::Arc;
use std::time::Instant;

use glasnost_map::CellXY;
use glasnost_map::terrain::{TileCell, TileInfo};
use glasnost_rules::layers::Layers;
use glasnost_rules::quirks::Quirks;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{
    BuildingType, Decimal, LandRow, LandType, LocomotionClass, Locomotor, OverlayType, UnitType,
};
use glasnost_rules::sim_econ::TiberiumType;
use glasnost_sim::econ::release::{self, Outcome};
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind, SimRules};
use glasnost_sim::testkit;
use glasnost_sim::tick;
use glasnost_sim::trace::{self, What};
use glasnost_sim::world::{HouseId, ObjectId, Target, World};

/// The map size (`testkit::empty_map`).
pub const SIZE: i32 = 50;

/// The rules a scenario runs under.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(clippy::large_enum_variant)]
pub enum Rules {
    /// Every quirk switch at its default (the original's behaviour).
    Reference,
    /// The reference with the play-fixes layer applied.
    Layer,
    /// The reference with these switches.
    Custom(Quirks),
}

/// The reference switches with the named ones off.
pub fn off(names: &[&str]) -> Rules {
    let text = format!(
        "{{\"globals\": {{\"quirks\": {{{}}}}}}}",
        names
            .iter()
            .map(|n| format!("\"{n}\": false"))
            .collect::<Vec<_>>()
            .join(", ")
    );
    let src = glasnost_rules::resolve::Source {
        layer: "test",
        file: "switches.jsonc",
        text: &text,
    };
    Rules::Custom(
        glasnost_rules::layers::quirks_with(Quirks::default(), &src).expect("known switches"),
    )
}

/// The play-fixes layer with the named switches back on (the original's
/// position): the layer less those fixes.
pub fn layer_without(names: &[&str]) -> Rules {
    let text = format!(
        "{{\"globals\": {{\"quirks\": {{{}}}}}}}",
        names
            .iter()
            .map(|n| format!("\"{n}\": true"))
            .collect::<Vec<_>>()
            .join(", ")
    );
    let src = glasnost_rules::resolve::Source {
        layer: "test",
        file: "switches.jsonc",
        text: &text,
    };
    let base = ruleset(Rules::Layer).globals.quirks;
    Rules::Custom(glasnost_rules::layers::quirks_with(base, &src).expect("known switches"))
}

fn push(r: &mut Ruleset, reg: &str, n: &str) {
    r.registries.entry(reg.into()).or_default().push(n.into());
}

fn dec(s: &str) -> Option<Decimal> {
    Some(Decimal(s.into()))
}

fn wheel(speed: i32) -> UnitType {
    UnitType {
        strength: Some(300),
        speed: Some(speed),
        rot: Some(6),
        locomotion_class: Some(LocomotionClass::Wheel),
        locomotor: Some(Locomotor::Drive),
        ..Default::default()
    }
}

/// The scenario ruleset: the testkit's (WALKER, an infantry type), plus
/// TANK (a plain unarmed vehicle) and SLOW (a slower one); BLOCK, a plain
/// 1 × 1 building; VF, a 3 × 2 vehicle factory; REFN, a 3 × 3 dock-unload
/// refinery; HARV, a harvester of capacity 5 docking at REFN; the overlay
/// run and tiberium ORE; land types clear (buildable), road and tiberium.
/// Rock and water have no land row, so nothing can enter them (S-0211
/// claim 2).
pub fn ruleset(rules: Rules) -> Ruleset {
    let mut r = testkit::ruleset();
    r.sim.land_types.insert(
        LandType::Clear,
        LandRow {
            buildable: Some(true),
            ..Default::default()
        },
    );
    r.sim
        .land_types
        .insert(LandType::Tiberium, LandRow::default());
    for (n, u) in [("TANK", wheel(7)), ("SLOW", wheel(4))] {
        push(&mut r, "vehicles", n);
        r.sim.vehicles.insert(n.into(), u);
    }
    let block = BuildingType {
        strength: Some(500),
        foundation: Some([1, 1]),
        ..Default::default()
    };
    let mut vf = BuildingType {
        strength: Some(500),
        foundation: Some([3, 2]),
        factory: Some("vehicle".into()),
        weapons_factory: Some(true),
        deploy_time: dec("0.01"),
        ..Default::default()
    };
    vf.tech_level = Some(1);
    let mut refn = BuildingType {
        strength: Some(500),
        foundation: Some([3, 3]),
        ..Default::default()
    };
    refn.econ.refinery = Some(true);
    refn.econ.dock_unload = Some(true);
    // REFQ: REFN with a queueing shift, so its queueing cell lies off the
    // docking cell (C7 part 6). The shift is invented: one cell east of the
    // docking cell's column and two rows below it.
    let mut refq = refn.clone();
    refq.econ.queueing_cell = Some([4, 3]);
    for (n, b) in [("BLOCK", block), ("VF", vf), ("REFN", refn), ("REFQ", refq)] {
        push(&mut r, "buildings", n);
        r.sim.buildings.insert(n.into(), b);
    }
    for i in 0..12 {
        let n = format!("OV{i:02}");
        push(&mut r, "overlays", &n);
        r.sim.overlays.insert(
            n,
            OverlayType {
                tiberium: Some(true),
                ..Default::default()
            },
        );
    }
    push(&mut r, "tiberiums", "ORE");
    r.sim.tiberiums.insert(
        "ORE".into(),
        TiberiumType {
            value: Some(25),
            growth: Some(100),
            growth_percentage: dec("0"),
            spread: Some(100),
            spread_percentage: dec("0"),
            image_run: Some([0, 12, 8]),
            ..Default::default()
        },
    );
    let mut harv = wheel(6);
    harv.harvester = Some(true);
    harv.storage = Some(5);
    harv.econ.dock = Some(vec!["REFN".into(), "REFQ".into()]);
    push(&mut r, "vehicles", "HARV");
    r.sim.vehicles.insert("HARV".into(), harv);
    match rules {
        Rules::Reference => {}
        Rules::Layer => {
            r.globals.quirks = Layers::PLAY_FIXES
                .apply_quirks(r.globals.quirks)
                .expect("the layer applies");
        }
        Rules::Custom(q) => r.globals.quirks = q,
    }
    r
}

/// What a cell of a scenario's map holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ground {
    /// Clear ground.
    Clear,
    /// Rock: nothing enters it.
    Rock,
    /// Water: no ground unit enters it.
    Water,
    /// Ore of stage 3 on clear ground.
    Ore,
}

/// The cell (dx, dy) from the map's middle.
pub fn at(dx: i32, dy: i32) -> CellXY {
    testkit::mid(SIZE, dx, dy)
}

/// A new game whose map is clear ground except where `paint` (offsets from
/// the middle) says otherwise. Houses 0 and 1, sandbox; only house 0 has
/// objects in the scenarios.
pub fn world(r: &Ruleset, seed: u32, paint: impl Fn(i32, i32) -> Ground) -> World {
    let mut cat = (*testkit::catalogue()).clone();
    let tile = |code: u8| TileInfo {
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
    };
    let rock = cat.tiles.len() as i32;
    // S-0211 claim 6: tile land code 7 is rock, 9 water.
    cat.tiles.push(tile(7));
    let water = cat.tiles.len() as i32;
    cat.tiles.push(tile(9));
    cat.allow_tiberium.extend([true, true]);
    let mut map = testkit::empty_map(SIZE);
    let mid = testkit::mid(SIZE, 0, 0);
    let cells: Vec<CellXY> = map.cells.iter().map(|(c, _)| c).collect();
    for c in cells {
        let g = paint(c.x - mid.x, c.y - mid.y);
        let cell = map.cells.get_mut(c).unwrap();
        match g {
            Ground::Clear => {}
            Ground::Rock => cell.tile = rock,
            Ground::Water => cell.tile = water,
            Ground::Ore => {
                cell.overlay = 0;
                cell.overlay_data = 3;
            }
        }
    }
    glasnost_sim::load::new_game(
        Arc::new(SimRules::new(r, "pathing-scenarios")),
        &map,
        Arc::new(cat),
        &testkit::setup(seed),
    )
    .expect("synthetic pathing scenario loads")
}

/// Runs the world past the order block of the first ticks.
pub fn ready(w: &mut World) {
    testkit::run(w, 1);
    while w.tick <= w.orders_blocked_through {
        testkit::run(w, 1);
    }
}

/// A move order.
pub fn mv(actor: ObjectId, c: CellXY) -> Order {
    Order::Behaviour {
        actor,
        behaviour: Behaviour::Move,
        attack_move: false,
        target: None,
        dest: Some(Target::Cell(c)),
    }
}

/// Places `n` units of type `name` in a block `cols` wide whose top-left
/// cell is (dx, dy); infantry stand one to a cell.
pub fn block(
    w: &mut World,
    kind: Kind,
    name: &str,
    n: usize,
    cols: i32,
    (dx, dy): (i32, i32),
) -> Vec<ObjectId> {
    (0..n as i32)
        .map(|i| testkit::spawn(w, kind, name, 0, at(dx + i % cols, dy + i / cols), 0))
        .collect()
}

/// Sends `ids` to `goal` as one group (one run of move orders in one tick,
/// S-0213 claim 7).
pub fn send(w: &mut World, ids: &[ObjectId], goal: CellXY) {
    for &id in ids {
        orders::issue(w, HouseId(0), mv(id, goal));
    }
}

/// One unit the runner follows: where it must get to, and how near counts
/// as there.
#[derive(Clone, Copy, Debug)]
pub struct Goal {
    /// The unit.
    pub id: ObjectId,
    /// The cell it was sent to.
    pub cell: CellXY,
    /// The largest distance (in cells, the larger of the two axes) that
    /// counts as arrived.
    pub radius: i32,
}

/// A hook called before each tick.
pub type Each = Box<dyn FnMut(&mut World)>;

/// A scenario ready to run.
pub struct Scene {
    /// Its name.
    pub name: &'static str,
    /// The world, with the orders given.
    pub w: World,
    /// The units followed.
    pub goals: Vec<Goal>,
    /// The ticks run.
    pub bound: i32,
    /// Whether every goal is reachable (a give-up then means the unit gave
    /// up although an open way exists).
    pub open: bool,
    /// Called before each tick (a factory's releases, say).
    pub each: Option<Each>,
}

/// What a run gives.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Metrics {
    /// Units followed.
    pub units: usize,
    /// Arrival ticks of those that arrived, counted from the order (from
    /// the unit's release, for a factory's units).
    pub arrived: Vec<i32>,
    /// Units that never arrived within the bound.
    pub missing: usize,
    /// Give-ups: a failed path computation that cleared the destination,
    /// or a given-up move.
    pub give_ups: u32,
    /// Give-ups of units that had not arrived, in a scenario whose goals
    /// are all reachable.
    pub open_give_ups: u32,
    /// Give-ups of units that had already arrived (a later move inside the
    /// group's spread failing).
    pub late_give_ups: u32,
    /// Back-and-forth steps: a unit entering the cell it left two cell
    /// changes before, before its arrival.
    pub reversals: u32,
    /// Back-and-forth steps after the unit's arrival (jostling).
    pub jostles: u32,
    /// Units still with a destination at the end (not settled).
    pub unsettled: usize,
    /// Units whose first destination after the order was also another
    /// followed unit's (shared group destinations).
    pub shared: usize,
    /// Path computations.
    pub searches: u32,
    /// Path computations that failed.
    pub failed: u32,
    /// The world's credits of house 0 at the end, less those at the start.
    pub credits: i32,
    /// The state hash at the end.
    pub hash: String,
    /// Mean wall time of a tick, in microseconds (not compared).
    pub micros: u64,
}

impl Metrics {
    fn count_give_up(&mut self, open: bool, arrived: Option<bool>) {
        self.give_ups += 1;
        match arrived {
            Some(true) => self.late_give_ups += 1,
            Some(false) if open => self.open_give_ups += 1,
            _ => {}
        }
    }

    /// The median arrival tick, if any arrived.
    pub fn median(&self) -> Option<i32> {
        let mut v = self.arrived.clone();
        v.sort_unstable();
        v.get(v.len() / 2).copied()
    }

    /// The last arrival tick, if any arrived.
    pub fn max(&self) -> Option<i32> {
        self.arrived.iter().copied().max()
    }
}

fn cheb(a: CellXY, b: CellXY) -> i32 {
    (a.x - b.x).abs().max((a.y - b.y).abs())
}

/// Plays `s` and counts.
pub fn play(mut s: Scene) -> Metrics {
    trace::switch_on(&mut s.w);
    let start = s.w.tick;
    let c0 = s.w.houses[0].credits;
    let n = s.goals.len();
    let mut arrived: Vec<Option<i32>> = vec![None; n];
    // The tick each unit came on the map (a factory's units come later).
    let mut starts: Vec<Option<i32>> = s
        .goals
        .iter()
        .map(|g| s.w.obj(g.id).filter(|o| o.on_map).map(|_| start))
        .collect();
    // The last two distinct cells of each unit.
    let mut cells: Vec<(Option<CellXY>, CellXY)> = s
        .goals
        .iter()
        .map(|g| (None, s.w.obj(g.id).map(|o| o.pos.cell()).unwrap_or(g.cell)))
        .collect();
    let mut m = Metrics {
        units: n,
        ..Default::default()
    };
    let index = |id: ObjectId| s.goals.iter().position(|g| g.id == id);
    let mut spent = 0u128;
    let mut firsts: Vec<Option<Target>> = vec![None; n];
    for _ in 0..s.bound {
        if let Some(f) = s.each.as_mut() {
            f(&mut s.w);
        }
        let t0 = Instant::now();
        tick::step(&mut s.w);
        spent += t0.elapsed().as_nanos();
        for e in trace::take(&s.w) {
            let i = index(e.unit);
            match e.what {
                What::Search { .. } => m.searches += 1,
                What::Failed { gave_up, .. } => {
                    m.failed += 1;
                    if gave_up {
                        m.count_give_up(s.open, i.map(|i| arrived[i].is_some()));
                    }
                }
                What::GaveUp => m.count_give_up(s.open, i.map(|i| arrived[i].is_some())),
                _ => {}
            }
        }
        for (i, g) in s.goals.iter().enumerate() {
            let Some(o) = s.w.obj(g.id) else { continue };
            if !o.on_map {
                continue;
            }
            let from = *starts[i].get_or_insert(s.w.tick);
            if firsts[i].is_none() {
                firsts[i] = o.dest;
            }
            let c = o.pos.cell();
            let (prev, now) = cells[i];
            if c != now {
                if prev == Some(c) {
                    if arrived[i].is_none() {
                        m.reversals += 1;
                    } else {
                        m.jostles += 1;
                    }
                }
                cells[i] = (Some(now), c);
            }
            if arrived[i].is_none() && cheb(c, g.cell) <= g.radius {
                arrived[i] = Some(s.w.tick - from);
            }
        }
    }
    m.arrived = arrived.iter().flatten().copied().collect();
    m.missing = arrived.iter().filter(|a| a.is_none()).count();
    m.unsettled = s
        .goals
        .iter()
        .filter(|g| s.w.obj(g.id).is_some_and(|o| o.dest.is_some()))
        .count();
    m.shared = (0..n)
        .filter(|&i| firsts[i].is_some() && (0..n).any(|j| j != i && firsts[j] == firsts[i]))
        .count();
    m.credits = s.w.houses[0].credits - c0;
    m.hash = glasnost_sim::save::state_hash(&s.w);
    m.micros = (spent / 1000 / s.bound.max(1) as u128) as u64;
    m
}

/// The radius a group of `n` counts as arrived within: the side of the
/// smallest square that holds it, halved, plus one.
pub fn group_radius(n: usize) -> i32 {
    let mut side = 1;
    while side * side < n as i32 {
        side += 1;
    }
    side / 2 + 1
}

fn goals(ids: &[ObjectId], cell: CellXY, radius: i32) -> Vec<Goal> {
    ids.iter().map(|&id| Goal { id, cell, radius }).collect()
}

/// A north-south rock wall at x = 0 from y = −20 to 20 with an opening of
/// `gap` cells starting at y = 0.
fn wall_with_gap(gap: i32) -> impl Fn(i32, i32) -> Ground {
    move |x, y| {
        if x == 0 && (-20..=20).contains(&y) && !(0..gap).contains(&y) {
            Ground::Rock
        } else {
            Ground::Clear
        }
    }
}

/// `n` vehicles 4 wide, 9 cells west of a wall with a `gap`-cell opening,
/// sent 9 cells east of it.
pub fn gap(rules: Rules, gap: i32, n: usize, seed: u32) -> Scene {
    let r = ruleset(rules);
    let mut w = world(&r, seed, wall_with_gap(gap));
    ready(&mut w);
    let ids = block(&mut w, Kind::Vehicle, "TANK", n, 4, (-11, -2));
    testkit::run(&mut w, 1);
    let goal = at(9, 0);
    send(&mut w, &ids, goal);
    Scene {
        name: match (gap, n) {
            (1, 20) => "gap 1, 20",
            (1, _) => "gap 1",
            _ => "gap 2",
        },
        w,
        goals: goals(&ids, goal, group_radius(n)),
        bound: 1500,
        open: true,
        each: None,
    }
}

/// A corridor 2 cells wide and 14 long between two rock bands; two groups
/// of `n` vehicles enter it from each end at once, each sent to the other
/// end.
pub fn corridor(rules: Rules, width: i32, n: usize, seed: u32) -> Scene {
    let r = ruleset(rules);
    let mut w = world(&r, seed, move |x, y| {
        if (-7..=7).contains(&x) && !(0..width).contains(&y) && (-6..=7).contains(&y) {
            Ground::Rock
        } else {
            Ground::Clear
        }
    });
    ready(&mut w);
    let a = block(&mut w, Kind::Vehicle, "TANK", n, 2, (-12, 0));
    let b = block(&mut w, Kind::Vehicle, "TANK", n, 2, (11, 0));
    testkit::run(&mut w, 1);
    let ga = at(12, 0);
    let gb = at(-12, 0);
    send(&mut w, &a, ga);
    send(&mut w, &b, gb);
    let mut g = goals(&a, ga, group_radius(n));
    g.extend(goals(&b, gb, group_radius(n)));
    Scene {
        name: match (width, n) {
            (2, 1) => "corridor 2, 1 each way",
            (2, 3) => "corridor 2, 3 each way",
            (3, 6) => "corridor 3, 6 each way",
            (1, 1) => "corridor 1, 1 each way",
            _ => "corridor 2, 6 each way",
        },
        w,
        goals: g,
        bound: 2000,
        open: true,
        each: None,
    }
}

/// `n` vehicles sent from 12 cells away to one cell in the open.
pub fn open_cell(rules: Rules, n: usize, seed: u32) -> Scene {
    let r = ruleset(rules);
    let mut w = world(&r, seed, |_, _| Ground::Clear);
    ready(&mut w);
    let ids = block(&mut w, Kind::Vehicle, "TANK", n, 4, (-14, -2));
    testkit::run(&mut w, 1);
    let goal = at(0, 0);
    send(&mut w, &ids, goal);
    Scene {
        name: "open, one cell",
        w,
        goals: goals(&ids, goal, group_radius(n)),
        bound: 1200,
        open: true,
        each: None,
    }
}

/// `n` vehicles sent to the cell in the angle of an L of buildings (a row
/// of 6 and a column of 6 of BLOCK).
pub fn by_buildings(rules: Rules, n: usize, seed: u32) -> Scene {
    let r = ruleset(rules);
    let mut w = world(&r, seed, |_, _| Ground::Clear);
    ready(&mut w);
    for i in 0..6 {
        testkit::spawn(&mut w, Kind::Building, "BLOCK", 0, at(i, -1), 0);
        testkit::spawn(&mut w, Kind::Building, "BLOCK", 0, at(-1, i), 0);
    }
    testkit::spawn(&mut w, Kind::Building, "BLOCK", 0, at(-1, -1), 0);
    let ids = block(&mut w, Kind::Vehicle, "TANK", n, 4, (10, 10));
    testkit::run(&mut w, 1);
    let goal = at(0, 0);
    send(&mut w, &ids, goal);
    Scene {
        name: "by buildings",
        w,
        goals: goals(&ids, goal, group_radius(n) + 1),
        bound: 1200,
        open: true,
        each: None,
    }
}

/// `n` harvesters sharing one refinery and a 6 × 6 field 10 cells east of
/// it, for `bound` ticks.
pub fn harvesters(rules: Rules, n: usize, seed: u32) -> Scene {
    harvesters_at(rules, "REFN", n, seed)
}

/// [`harvesters`] with REFQ, whose queueing cell lies off the docking cell
/// (C7 part 6).
pub fn harvesters_queue(rules: Rules, n: usize, seed: u32) -> Scene {
    let mut s = harvesters_at(rules, "REFQ", n, seed);
    s.name = "harvesters, queueing cell off the dock";
    s
}

fn harvesters_at(rules: Rules, refinery: &str, n: usize, seed: u32) -> Scene {
    let r = ruleset(rules);
    let mut w = world(&r, seed, |x, y| {
        if (8..14).contains(&x) && (-3..3).contains(&y) {
            Ground::Ore
        } else {
            Ground::Clear
        }
    });
    ready(&mut w);
    testkit::spawn(&mut w, Kind::Building, refinery, 0, at(-3, -1), 0);
    let ids: Vec<ObjectId> = (0..n as i32)
        .map(|i| testkit::spawn(&mut w, Kind::Vehicle, "HARV", 0, at(3, i * 2 - 4), 0))
        .collect();
    testkit::run(&mut w, 1);
    for &h in &ids {
        let t = w.tick;
        w.obj_mut(h).unwrap().beh.force(Behaviour::Harvest, t, true);
    }
    Scene {
        name: "harvesters",
        w,
        // Followed to the field only; the credits tell the rest.
        goals: goals(&ids, at(10, 0), 3),
        bound: 3000,
        open: true,
        each: None,
    }
}

/// A vehicle factory with a rally point 12 cells south-east, `parked`
/// vehicles parked near its exit, releasing `n` vehicles one after another
/// as soon as it can.
pub fn factory(rules: Rules, n: usize, parked: usize, seed: u32) -> Scene {
    let r = ruleset(rules);
    let mut w = world(&r, seed, |_, _| Ground::Clear);
    ready(&mut w);
    let k = testkit::spawn(&mut w, Kind::Building, "VF", 0, at(0, 0), 0);
    let rally = at(10, 12);
    w.obj_mut(k).unwrap().guard_spot = Some(Target::Cell(rally));
    for i in 0..parked as i32 {
        testkit::spawn(
            &mut w,
            Kind::Vehicle,
            "TANK",
            0,
            at(i % 3 - 1, -3 - i / 3),
            0,
        );
    }
    testkit::run(&mut w, 1);
    let ti = w.rules().index_of(Kind::Vehicle, "TANK").unwrap();
    let ids: Vec<ObjectId> = (0..n)
        .map(|_| glasnost_sim::objects::create(&mut w, Kind::Vehicle, ti, Some(HouseId(0))))
        .collect();
    let mut queue = ids.clone();
    queue.reverse();
    let each = move |w: &mut World| {
        if let Some(&u) = queue.last()
            && release::release(w, k, u) == Outcome::Released
        {
            queue.pop();
        }
    };
    Scene {
        name: "factory stream",
        w,
        goals: goals(&ids, rally, group_radius(n) + 1),
        bound: 3000,
        open: true,
        each: Some(Box::new(each)),
    }
}

/// `n` infantry and `n` vehicles, mixed in one block, sent through a
/// 2-cell opening.
pub fn mixed(rules: Rules, n: usize, seed: u32) -> Scene {
    let r = ruleset(rules);
    let mut w = world(&r, seed, wall_with_gap(2));
    ready(&mut w);
    let mut ids = Vec::new();
    for i in 0..(2 * n) as i32 {
        let c = (-11 + i % 4, -2 + i / 4);
        let id = if i % 2 == 0 {
            testkit::spawn(&mut w, Kind::Vehicle, "TANK", 0, at(c.0, c.1), 0)
        } else {
            testkit::spawn(&mut w, Kind::Infantry, "WALKER", 0, at(c.0, c.1), 0)
        };
        ids.push(id);
    }
    testkit::run(&mut w, 1);
    let goal = at(9, 0);
    send(&mut w, &ids, goal);
    Scene {
        name: "mixed through gap 2",
        w,
        goals: goals(&ids, goal, group_radius(2 * n)),
        bound: 2000,
        open: true,
        each: None,
    }
}

/// `n` vehicles sent to a cell walled in by buildings (a ring of BLOCK
/// round it), so not reachable.
pub fn walled_in(rules: Rules, n: usize, seed: u32) -> Scene {
    let r = ruleset(rules);
    let mut w = world(&r, seed, |_, _| Ground::Clear);
    ready(&mut w);
    for dx in -2..=2 {
        for dy in -2..=2 {
            if dx != 0 || dy != 0 {
                testkit::spawn(&mut w, Kind::Building, "BLOCK", 0, at(dx, dy), 0);
            }
        }
    }
    let ids = block(&mut w, Kind::Vehicle, "TANK", n, 4, (-14, -2));
    testkit::run(&mut w, 1);
    let goal = at(0, 0);
    send(&mut w, &ids, goal);
    Scene {
        name: "inside buildings",
        w,
        // Near enough is the ring's outside.
        goals: goals(&ids, goal, group_radius(n) + 3),
        bound: 1500,
        open: false,
        each: None,
    }
}

/// `n` vehicles sent to a cell inside a closed ring of rock.
pub fn unreachable(rules: Rules, n: usize, seed: u32) -> Scene {
    let r = ruleset(rules);
    let mut w = world(&r, seed, |x, y| {
        if x.abs().max(y.abs()) == 3 {
            Ground::Rock
        } else {
            Ground::Clear
        }
    });
    ready(&mut w);
    let ids = block(&mut w, Kind::Vehicle, "TANK", n, 4, (-14, -2));
    testkit::run(&mut w, 1);
    let goal = at(0, 0);
    send(&mut w, &ids, goal);
    Scene {
        name: "unreachable cell",
        w,
        goals: goals(&ids, goal, group_radius(n) + 4),
        bound: 1500,
        open: false,
        each: None,
    }
}

/// A lake of water 10 cells across with a causeway `width` cells wide
/// through it; `n` vehicles sent across.
pub fn causeway(rules: Rules, width: i32, n: usize, seed: u32) -> Scene {
    let r = ruleset(rules);
    let mut w = world(&r, seed, move |x, y| {
        if (-5..=5).contains(&x) && !(0..width).contains(&y) && (-18..=18).contains(&y) {
            Ground::Water
        } else {
            Ground::Clear
        }
    });
    ready(&mut w);
    let ids = block(&mut w, Kind::Vehicle, "TANK", n, 4, (-14, -2));
    testkit::run(&mut w, 1);
    let goal = at(12, 0);
    send(&mut w, &ids, goal);
    Scene {
        name: "causeway 1",
        w,
        goals: goals(&ids, goal, group_radius(n)),
        bound: 2000,
        open: true,
        each: None,
    }
}

/// A long rock wall (x = 0, y = −16 to 16) with the units 3 cells west of
/// its middle, sent 3 cells east of it: the way is round an end.
pub fn round_wall(rules: Rules, n: usize, seed: u32) -> Scene {
    let r = ruleset(rules);
    let mut w = world(&r, seed, |x, y| {
        if x == 0 && (-16..=16).contains(&y) {
            Ground::Rock
        } else {
            Ground::Clear
        }
    });
    ready(&mut w);
    let ids = block(&mut w, Kind::Vehicle, "TANK", n, 2, (-4, -1));
    testkit::run(&mut w, 1);
    let goal = at(3, 0);
    send(&mut w, &ids, goal);
    Scene {
        name: "round a wall",
        w,
        goals: goals(&ids, goal, group_radius(n)),
        bound: 2000,
        open: true,
        each: None,
    }
}

/// `n` vehicles sent to the cell on the shore of a lake (water east of
/// x = 1), so the group's spread runs into the water.
pub fn shore(rules: Rules, n: usize, seed: u32) -> Scene {
    let r = ruleset(rules);
    let mut w = world(&r, seed, |x, y| {
        if (1..=8).contains(&x) && (-10..=10).contains(&y) {
            Ground::Water
        } else {
            Ground::Clear
        }
    });
    ready(&mut w);
    let ids = block(&mut w, Kind::Vehicle, "TANK", n, 4, (-14, -2));
    testkit::run(&mut w, 1);
    let goal = at(0, 0);
    send(&mut w, &ids, goal);
    Scene {
        name: "shore",
        w,
        goals: goals(&ids, goal, group_radius(n) + 1),
        bound: 1500,
        open: true,
        each: None,
    }
}

/// `n` infantry sent through a 1-cell opening.
pub fn infantry_gap(rules: Rules, n: usize, seed: u32) -> Scene {
    let r = ruleset(rules);
    let mut w = world(&r, seed, wall_with_gap(1));
    ready(&mut w);
    let ids = block(&mut w, Kind::Infantry, "WALKER", n, 4, (-11, -2));
    testkit::run(&mut w, 1);
    let goal = at(9, 0);
    send(&mut w, &ids, goal);
    Scene {
        name: "infantry gap 1",
        w,
        goals: goals(&ids, goal, group_radius(n)),
        bound: 2000,
        open: true,
        each: None,
    }
}

/// `n` TANK and `n` SLOW vehicles, mixed in one block, sent through a
/// 2-cell opening (a group of two speeds).
pub fn mixed_speeds(rules: Rules, n: usize, seed: u32) -> Scene {
    let r = ruleset(rules);
    let mut w = world(&r, seed, wall_with_gap(2));
    ready(&mut w);
    let ids: Vec<ObjectId> = (0..(2 * n) as i32)
        .map(|i| {
            let name = if i % 2 == 0 { "TANK" } else { "SLOW" };
            testkit::spawn(
                &mut w,
                Kind::Vehicle,
                name,
                0,
                at(-11 + i % 4, -2 + i / 4),
                0,
            )
        })
        .collect();
    testkit::run(&mut w, 1);
    let goal = at(9, 0);
    send(&mut w, &ids, goal);
    Scene {
        name: "two speeds through gap 2",
        w,
        goals: goals(&ids, goal, group_radius(2 * n)),
        bound: 2000,
        open: true,
        each: None,
    }
}

/// Two closed rooms of 6 × 6 joined only by a corridor 2 cells wide and
/// 16 long (no way round); `n` vehicles in each room sent to the other.
pub fn rooms(rules: Rules, n: usize, seed: u32) -> Scene {
    let r = ruleset(rules);
    let mut w = world(&r, seed, |x, y| {
        let room = (9..=14).contains(&x.abs()) && (-3..=2).contains(&y);
        let hall = x.abs() < 9 && (0..2).contains(&y);
        if room || hall {
            Ground::Clear
        } else {
            Ground::Rock
        }
    });
    ready(&mut w);
    let a = block(&mut w, Kind::Vehicle, "TANK", n, 3, (-14, -2));
    let b = block(&mut w, Kind::Vehicle, "TANK", n, 3, (12, -2));
    testkit::run(&mut w, 1);
    let ga = at(12, 0);
    let gb = at(-12, 0);
    send(&mut w, &a, ga);
    send(&mut w, &b, gb);
    let mut g = goals(&a, ga, 3);
    g.extend(goals(&b, gb, 3));
    Scene {
        name: if n == 3 {
            "two rooms, 3 each way"
        } else {
            "two rooms, 6 each way"
        },
        w,
        goals: g,
        bound: 3000,
        open: true,
        each: None,
    }
}

/// Three closed rooms of 6 × 6, west, east and north, joined only by halls
/// 1 cell wide that meet at one cell (C7 part 6); `n` vehicles in each
/// room, sent round: west to east, east to north, north to west. Every
/// group needs the meeting cell, and two of the three pairs head across
/// each other rather than head-on.
pub fn three_ways(rules: Rules, n: usize, seed: u32) -> Scene {
    let r = ruleset(rules);
    let mut w = world(&r, seed, |x, y| {
        let west = (-16..=-11).contains(&x) && (-3..=2).contains(&y);
        let east = (11..=16).contains(&x) && (-3..=2).contains(&y);
        let north = (-3..=2).contains(&x) && (-16..=-11).contains(&y);
        let hall = (y == 0 && x.abs() <= 10) || (x == 0 && (-10..=0).contains(&y));
        if west || east || north || hall {
            Ground::Clear
        } else {
            Ground::Rock
        }
    });
    ready(&mut w);
    let a = block(&mut w, Kind::Vehicle, "TANK", n, 3, (-15, -2));
    let b = block(&mut w, Kind::Vehicle, "TANK", n, 3, (12, -2));
    let c = block(&mut w, Kind::Vehicle, "TANK", n, 3, (-2, -15));
    testkit::run(&mut w, 1);
    let (ga, gb, gc) = (at(13, 0), at(0, -13), at(-13, 0));
    send(&mut w, &a, ga);
    send(&mut w, &b, gb);
    send(&mut w, &c, gc);
    let mut g = goals(&a, ga, 3);
    g.extend(goals(&b, gb, 3));
    g.extend(goals(&c, gc, 3));
    Scene {
        name: "three ways, 3 each",
        w,
        goals: g,
        bound: 4000,
        open: true,
        each: None,
    }
}

/// The two closed rooms of [`rooms`] joined by a hall 1 cell wide instead
/// (C7 part 6); `n` vehicles in each room sent to the other. With `parked`
/// one more vehicle of the same house stands in the middle of the hall with
/// no order; it is not followed.
pub fn narrow_rooms(rules: Rules, n: usize, parked: bool, seed: u32) -> Scene {
    let r = ruleset(rules);
    let mut w = world(&r, seed, |x, y| {
        let room = (9..=14).contains(&x.abs()) && (-3..=2).contains(&y);
        let hall = x.abs() < 9 && y == 0;
        if room || hall {
            Ground::Clear
        } else {
            Ground::Rock
        }
    });
    ready(&mut w);
    if parked {
        testkit::spawn(&mut w, Kind::Vehicle, "TANK", 0, at(0, 0), 0);
    }
    let a = block(&mut w, Kind::Vehicle, "TANK", n, 3, (-14, -2));
    let b = block(&mut w, Kind::Vehicle, "TANK", n, 3, (12, -2));
    testkit::run(&mut w, 1);
    let ga = at(12, 0);
    let gb = at(-12, 0);
    send(&mut w, &a, ga);
    send(&mut w, &b, gb);
    let mut g = goals(&a, ga, 3);
    g.extend(goals(&b, gb, 3));
    Scene {
        name: if parked {
            "narrow rooms, 3 each way, one parked"
        } else {
            "narrow rooms, 3 each way"
        },
        w,
        goals: g,
        bound: 3000,
        open: true,
        each: None,
    }
}

/// Every scenario of the set, under `rules` with `seed`.
pub fn all(rules: Rules, seed: u32) -> Vec<Scene> {
    vec![
        gap(rules, 1, 12, seed),
        gap(rules, 2, 16, seed),
        corridor(rules, 2, 6, seed),
        corridor(rules, 2, 1, seed),
        corridor(rules, 2, 3, seed),
        corridor(rules, 3, 6, seed),
        corridor(rules, 1, 1, seed),
        open_cell(rules, 16, seed),
        by_buildings(rules, 16, seed),
        harvesters(rules, 4, seed),
        factory(rules, 8, 5, seed),
        mixed(rules, 6, seed),
        walled_in(rules, 8, seed),
        unreachable(rules, 8, seed),
        causeway(rules, 1, 10, seed),
        round_wall(rules, 4, seed),
        shore(rules, 16, seed),
        infantry_gap(rules, 16, seed),
        gap(rules, 1, 20, seed),
        mixed_speeds(rules, 6, seed),
        rooms(rules, 3, seed),
        rooms(rules, 6, seed),
        harvesters_queue(rules, 4, seed),
        three_ways(rules, 3, seed),
        narrow_rooms(rules, 3, false, seed),
        narrow_rooms(rules, 3, true, seed),
    ]
}

/// One table line.
pub fn line(name: &str, rules: Rules, m: &Metrics) -> String {
    let o = |v: Option<i32>| v.map_or("-".to_string(), |v| v.to_string());
    format!(
        "| {name} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |",
        match rules {
            Rules::Reference => "off",
            Rules::Layer => "on",
            Rules::Custom(_) => "custom",
        },
        m.units,
        o(m.median()),
        o(m.max()),
        m.missing,
        m.give_ups,
        m.open_give_ups,
        m.late_give_ups,
        m.reversals,
        m.jostles,
        m.unsettled,
        m.shared,
        m.searches,
        m.failed,
        m.credits,
        m.micros,
    )
}

/// The table's header.
pub const HEADER: &str = "| Scenario | Layer | Units | Median | Max | Missing | Give-ups | Open give-ups | Late give-ups | Reversals | Jostles | Unsettled | Shared | Searches | Failed | Credits | µs/tick |\n|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|";
