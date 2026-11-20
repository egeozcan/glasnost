//! `glasnost-headless siege-watch` (#449): our own tooling, no game
//! behaviour. Plays one game as the AI tournament's install round plays it
//! (`ai_tournament::play`: the same setup, seats, sides and cap) and prints
//! a line per house every `every` ticks: credits, buildings, factories,
//! refineries, harvesters, army size and worth, the house's worth, and for a
//! siege seat its strike. To see why a game of the round went as it did.

use std::fmt::Write as _;
use std::sync::Arc;

use glasnost_ai::game::Game;
use glasnost_ai::scripted::{Role, role_of};
use glasnost_sim::econ::setup::{self, Player, Settings};
use glasnost_sim::econ::{production, tech, victory};
use glasnost_sim::rules::{Kind, SimRules};
use glasnost_sim::world::{AiSeat, HouseId, World};

use crate::ai_tournament::{Ground, parse_entrant, worth};

/// The usage line.
pub const USAGE: &str = "siege-watch map=N seed=N a=key:level b=key:level [start=0|1] [side=0|1] [ticks=27000] [every=1500]";

/// The options.
#[derive(Clone, Debug)]
pub struct Options {
    map: usize,
    seed: u32,
    a: AiSeat,
    b: AiSeat,
    swap_start: bool,
    swap_side: bool,
    ticks: i32,
    every: i32,
}

impl Options {
    /// Parses `key=value` words.
    pub fn parse(args: &[String]) -> Result<Self, String> {
        let mut o = Options {
            map: 0,
            seed: 7,
            a: AiSeat::DEFAULT,
            b: AiSeat::DEFAULT,
            swap_start: false,
            swap_side: false,
            // nocite: our own tooling (the tournament's cap)
            ticks: 27_000,
            // nocite: our own tooling (a line a minute of game time, about)
            every: 1500,
        };
        for a in args {
            let (k, v) = a
                .split_once('=')
                .ok_or_else(|| format!("not key=value: {a}"))?;
            let bad = |_| format!("bad value: {a}");
            match k {
                "map" => o.map = v.parse().map_err(bad)?,
                "seed" => o.seed = v.parse().map_err(bad)?,
                "a" => o.a = parse_entrant(v)?,
                "b" => o.b = parse_entrant(v)?,
                "start" => o.swap_start = v == "1",
                "side" => o.swap_side = v == "1",
                "ticks" => o.ticks = v.parse().map_err(bad)?,
                "every" => o.every = v.parse::<i32>().map_err(bad)?.max(1),
                _ => return Err(format!("unknown option {k}")),
            }
        }
        Ok(o)
    }
}

/// The cost of every object a house got, by category, from the objects
/// seen on the map (a house's new object adds its cost once).
#[derive(Default)]
struct Got {
    seen: std::collections::BTreeSet<glasnost_sim::world::ObjectId>,
    by: [std::collections::BTreeMap<&'static str, i64>; 2],
    homes: [Option<glasnost_map::CellXY>; 2],
    alive: std::collections::BTreeMap<glasnost_sim::world::ObjectId, (usize, i64, String)>,
    lost: [std::collections::BTreeMap<String, i64>; 2],
}

impl Got {
    fn category(w: &World, t: glasnost_sim::econ::TypeRef) -> &'static str {
        if t.kind == Kind::Building {
            let gives = w
                .rules()
                .superweapons
                .building_sw
                .get(t.index as usize)
                .is_some_and(|f| f.iter().any(Option::is_some));
            return match role_of(w, t) {
                Some(Role::Power) => "power",
                Some(Role::Refinery) => "refinery",
                Some(Role::Yard) | Some(Role::Factory(_)) => "factory",
                _ if gives => "superweapon",
                _ => match production::line_kind(w, t) {
                    Some(production::LineKind::Defence) => "defence",
                    _ => "other building",
                },
            };
        }
        match role_of(w, t) {
            Some(Role::Harvester) => "harvester",
            Some(Role::Army) => "army",
            _ => "other unit",
        }
    }

    fn note(&mut self, w: &World) {
        // The army units lost since the last tick: where (by tenths of the
        // way from the owner's home to the enemy's) and to what kind of
        // object that had them as its target at the last tick.
        let gone: Vec<_> = self
            .alive
            .keys()
            .copied()
            .filter(|id| w.obj(*id).is_none_or(|o| o.health <= 0 || !o.on_map))
            .collect();
        for id in gone {
            if let Some((h, cost, key)) = self.alive.remove(&id) {
                *self.lost[h].entry(key).or_default() += cost;
            }
        }
        let mut hunters: std::collections::BTreeMap<glasnost_sim::world::ObjectId, &'static str> =
            Default::default();
        for o in w.objects.values() {
            if let Some(glasnost_sim::world::Target::Object(t)) = o.target {
                let k = match o.kind {
                    Kind::Building => "building",
                    Kind::Infantry => "infantry",
                    Kind::Vehicle => "vehicle",
                    Kind::Aircraft => "aircraft",
                    _ => "other",
                };
                hunters.insert(t, k);
            }
        }
        for o in w.objects.values() {
            let Some(h) = o.owner else { continue };
            let h = usize::from(h.0);
            if h > 1 || !o.on_map || o.health <= 0 {
                continue;
            }
            if self.homes[h].is_none() {
                self.homes[h] = Some(o.pos.cell());
            }
            let t = tech::type_of(o);
            if o.kind == Kind::Building || role_of(w, t) != Some(Role::Army) {
                continue;
            }
            let (Some(a), Some(b)) = (self.homes[h], self.homes[1 - h]) else {
                continue;
            };
            let c = o.pos.cell();
            let d = |p: glasnost_map::CellXY| {
                let (x, y) = (i64::from(c.x - p.x), i64::from(c.y - p.y));
                (x * x + y * y).isqrt()
            };
            let (da, db) = (d(a), d(b));
            let tenth = (10 * da / (da + db).max(1)).min(9);
            let by = hunters.get(&o.id).copied().unwrap_or("none");
            self.alive.insert(
                o.id,
                (
                    h,
                    i64::from(production::cost_field(w, t)),
                    format!("{tenth}/{by}"),
                ),
            );
        }
        for o in w.objects.values() {
            let Some(h) = o.owner else { continue };
            if h.0 > 1 || !o.on_map || self.seen.contains(&o.id) {
                continue;
            }
            self.seen.insert(o.id);
            let t = tech::type_of(o);
            *self.by[usize::from(h.0)]
                .entry(Self::category(w, t))
                .or_default() += i64::from(production::cost_field(w, t));
        }
    }
}

/// One house's line.
fn line(g: &Game, h: HouseId, got: &Got) -> String {
    let w: &World = &g.world;
    let own = || {
        w.objects
            .values()
            .filter(move |o| o.owner == Some(h) && o.on_map && o.health > 0)
    };
    let buildings = own().filter(|o| o.kind == Kind::Building).count();
    let factories = own()
        .filter(|o| {
            o.kind == Kind::Building
                && w.rules()
                    .buildings
                    .get(o.type_index as usize)
                    .is_some_and(|b| b.factory.is_some())
        })
        .count();
    let role = |r: Role| {
        own()
            .filter(|o| role_of(w, tech::type_of(o)) == Some(r))
            .count()
    };
    let army: Vec<_> = own()
        .filter(|o| o.kind != Kind::Building && role_of(w, tech::type_of(o)) == Some(Role::Army))
        .collect();
    let army_worth: i64 = army
        .iter()
        .map(|o| i64::from(production::cost_field(w, tech::type_of(o))))
        .sum();
    let mut s = format!(
        "tick {} house {}: credits {} buildings {buildings} factories {factories} refineries {} harvesters {} army {} worth {army_worth} total {}",
        w.tick,
        h.0,
        w.houses[usize::from(h.0)].credits,
        role(Role::Refinery),
        role(Role::Harvester),
        army.len(),
        worth(w, h),
    );
    let mut kinds: std::collections::BTreeMap<String, usize> = Default::default();
    for o in &army {
        let name = w
            .rules()
            .unit(o.kind, o.type_index)
            .map_or_else(|| format!("{:?}", o.kind), |u| u.name.clone());
        *kinds.entry(name).or_default() += 1;
    }
    let _ = write!(
        s,
        " {kinds:?} got {:?} lost {:?}",
        got.by[usize::from(h.0)],
        got.lost[usize::from(h.0)]
    );
    let lines: Vec<String> = w
        .econ
        .lines
        .lines
        .iter()
        .filter(|l| l.house == h)
        .filter_map(|l| l.item.as_ref().map(|i| (l.kind, i)))
        .map(|(k, i)| {
            format!(
                "{k:?}:{} {}/54 owed {} {:?}",
                Got::category(w, i.t),
                i.progress,
                i.owed,
                i.hold
            )
        })
        .collect();
    let _ = write!(s, " lines [{}]", lines.join(", "));
    if let Some(seat) = g.seats().iter().find(|s| s.house() == h)
        && let Some(siege) = seat.script().siege()
    {
        let _ = write!(
            s,
            " | strikes {} recalls {} out {} remembered {}",
            siege.strikes(),
            siege.recalls(),
            siege.out().len(),
            siege.remembered()
        );
    }
    s
}

/// Plays the game on `ground`, printing as it goes; returns the end line.
pub fn run_on(o: &Options, g: &Ground) -> Result<String, String> {
    let (start_a, start_b) = if o.swap_start { (1, 0) } else { (0, 1) };
    let (side_a, side_b) = if o.swap_side { (1, 0) } else { (0, 1) };
    let player = |name: &str, side: usize, start: u32| Player {
        name: name.into(),
        country: g.countries[side].clone(),
        start: Some(start),
        ai: true,
        values: Default::default(),
    };
    let mut s = setup::skirmish(
        &g.rules,
        &[player("A", side_a, start_a), player("B", side_b, start_b)],
        Settings::default(),
        0,
        o.seed,
    );
    s.ai = vec![o.a, o.b];
    let mut game = Game::new(g.rules.clone(), &g.map, &g.name, g.tiles.clone(), s)
        .map_err(|e| format!("{e:?}"))?;
    let mut got = Got::default();
    while glasnost_sim::tick::ticks_run(&game.world) < o.ticks && game.outcome().is_none() {
        game.step();
        game.world.effects.clear();
        got.note(&game.world);
        if game.world.tick % o.every == 0 {
            for h in [HouseId(0), HouseId(1)] {
                println!("{}", line(&game, h, &got));
            }
        }
    }
    let end = match game.outcome() {
        Some(out) => {
            let winner = out
                .results
                .iter()
                .find(|r| r.1 == victory::Verdict::Win)
                .map(|r| r.0);
            format!("over at tick {}: winner {winner:?}", game.world.tick)
        }
        None => format!(
            "cap at tick {}: worth {} v {}",
            game.world.tick,
            worth(&game.world, HouseId(0)),
            worth(&game.world, HouseId(1))
        ),
    };
    for h in [HouseId(0), HouseId(1)] {
        println!("{}", line(&game, h, &got));
    }
    Ok(end)
}

/// Loads the install's map `o.map` with the first open country of each of
/// the first two sides, as the tournament's install grounds (adapted from
/// `ai_tournament::install_grounds`, our own code), and plays.
pub fn run(o: &Options, dir: &std::path::Path) -> Result<String, String> {
    let ground = pollster::block_on(async {
        let mut vfs = crate::local_lane::mount(dir)
            .await
            .map_err(|e| format!("could not read the install ({:?})", e.kind()))?;
        let layers = glasnost_rules::layers::Layers::from_flag(
            std::env::var("GLASNOST_PLAY_FIXES").ok().as_deref(),
        );
        let listings = glasnost_convert::find::find_maps(&vfs).await;
        let listing = listings
            .get(o.map)
            .ok_or_else(|| format!("no map {}", o.map))?;
        let bytes = vfs
            .read(&listing.file)
            .await
            .map_err(|_| format!("map {} unreadable", o.map))?;
        let inputs = glasnost_convert::game::load(&mut vfs, bytes, "install".into(), layers).await;
        let (Ok(resolved), Some(tiles)) = (inputs.resolved, inputs.tiles) else {
            return Err(format!("map {} did not load", o.map));
        };
        let rules = Arc::new(SimRules::new(&resolved.ruleset, &resolved.sim_rules_hash));
        let by_side = crate::skirmish_lane::side_countries(&rules);
        let mut k = by_side.values();
        let (Some(a), Some(b)) = (k.next(), k.next()) else {
            return Err("fewer than two sides".into());
        };
        Ok::<Ground, String>(Ground {
            name: format!("install map {}", o.map),
            rules: rules.clone(),
            map: inputs.map,
            tiles: Arc::new(tiles),
            countries: [a.clone(), b.clone()],
        })
    })?;
    run_on(o, &ground)
}
