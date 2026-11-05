//! A four-house game of our own AI outside the network (local lane, M9
//! rehearsal follow-up): the room's setup (four rows, teams 1, 2, 1, 2,
//! random countries and starts) on an install map, played either by the
//! AI inside the tick (AI rows) or by one brain per person row outside the
//! simulation whose orders run a fixed delay later, as in a network game,
//! with the bot's repeat filter. Prints when and how each house's objects
//! leave the map. Our own diagnostics; no spec applies.
//!
//! `GLASNOST_GAME_DIR=<install> cargo test --release -p glasnost-tools
//! --test four_houses -- --ignored --nocapture`, with `GLASNOST_FH_SEEDS`
//! (`1-8`), `GLASNOST_FH_MAP` (30), `GLASNOST_FH_TICKS` (6000),
//! `GLASNOST_FH_DELAY` (comma list of delays; `in` for the AI rows inside
//! the tick; default `in,0,8`), `GLASNOST_FH_LOG` (0: summary only, 1: the
//! losses, 2: also every order before `GLASNOST_FH_ORDERS_TO`, 600) and
//! `GLASNOST_FH_WATCH=ID,FROM,TO` (one object's movement state every tick
//! from FROM to TO). The output also goes to `four-houses.txt` in
//! `GLASNOST_LANE_OUT` (default `local-lane/m9/reh-ai/`, kept out of the
//! repository).

use std::collections::{BTreeMap, VecDeque};
use std::fmt::Write as _;
use std::path::Path;

use glasnost_ai::brain::Brain;
use glasnost_app::game::client::{Client, Job};
use glasnost_sim::combat::effects::Effect;
use glasnost_sim::orders::Order;
use glasnost_sim::rules::Kind;
use glasnost_sim::world::{HouseId, HouseRole, ObjectId, Target, World};
use glasnost_tools::net_client;
use glasnost_ui::setup::{MapEntry, Setup, SlotKind};

fn env<T: std::str::FromStr>(k: &str, d: T) -> T {
    std::env::var(k)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(d)
}

fn name(w: &World, kind: Kind, index: u32) -> String {
    match kind {
        Kind::Building => w
            .rules()
            .buildings
            .get(index as usize)
            .map(|b| b.name.clone()),
        _ => w.rules().unit(kind, index).map(|u| u.name.clone()),
    }
    .unwrap_or_default()
}

// nocite: our own harness (the soak's rows and teams, `soak::TEAMS`)
const TEAMS: [u8; 4] = [1, 2, 1, 2];

/// The rows' teams: `GLASNOST_FH_TEAMS` (four comma-separated team numbers,
/// `1,2,3,4` for every house on its own), else [`TEAMS`].
fn teams() -> [u8; 4] {
    let mut t = TEAMS;
    if let Ok(v) = std::env::var("GLASNOST_FH_TEAMS") {
        for (i, x) in v.split(',').take(4).enumerate() {
            if let Ok(n) = x.trim().parse() {
                t[i] = n;
            }
        }
    }
    t
}

#[derive(Clone)]
struct Seen {
    owner: HouseId,
    kind: Kind,
    ty: u32,
    cell: (i32, i32),
    health: i32,
}

fn start(c: &mut Client, map: usize, seed: u32, ai: bool) -> Result<(), String> {
    let install = c.install.as_ref().ok_or("no install")?;
    let maps: Vec<MapEntry> = install.maps.iter().map(|(_, m)| m.clone()).collect();
    let mut setup = Setup::new(&install.setup);
    setup.map = map;
    // The reference ruleset, not our own fixes layer (M8 step C7).
    setup.fixes = false;
    for (i, slot) in setup.slots.iter_mut().enumerate() {
        if i < 4 {
            slot.kind = if ai { SlotKind::Ai } else { SlotKind::Human };
            slot.team = Some(teams()[i]);
        } else {
            slot.kind = SlotKind::Closed;
        }
    }
    let lobby = setup
        .lobby(&install.setup, &maps)
        .map_err(|p| format!("{p:?}"))?;
    let vfs = c.vfs.take().ok_or("no vfs")?;
    let job = Job::Start {
        vfs,
        file: install.maps[map].0.file,
        lobby,
        offered: install.setup.countries.clone(),
        seed,
        speech: Default::default(),
        local: if ai { None } else { Some(HouseId(0)) },
        cursor: install.cursor_palette.clone(),
        colour_blind: false,
        watch: ai,
    };
    c.begin_job(job);
    net_client::run_jobs(c, false);
    if c.session.is_none() {
        return Err(format!("no session: {}", c.status));
    }
    Ok(())
}

/// One game; returns its summary line.
fn play(
    c: &mut Client,
    map: usize,
    seed: u32,
    delay: Option<i32>,
    ticks: i32,
    log: u8,
) -> Result<String, String> {
    start(c, map, seed, delay.is_none())?;
    let s = c.session.as_mut().unwrap();
    let game = &mut s.play.game;
    let players: Vec<HouseId> = game
        .world
        .houses
        .iter()
        .enumerate()
        .filter(|(_, h)| h.role == HouseRole::Player)
        .map(|(i, _)| HouseId(i as u8))
        .collect();
    let mut out = String::new();
    let tag = match delay {
        None => "in".to_string(),
        Some(d) => format!("d{d}"),
    };
    {
        let w = &game.world;
        let mut starts = Vec::new();
        for &h in &players {
            let hs = &w.houses[usize::from(h.0)];
            let objs: Vec<&glasnost_sim::world::Object> = w
                .objects
                .values()
                .filter(|o| o.owner == Some(h) && o.on_map)
                .collect();
            let n = objs.len() as i32;
            let (sx, sy) = objs.iter().fold((0, 0), |(x, y), o| {
                let c = o.pos.cell();
                (x + c.x, y + c.y)
            });
            let centre = if n > 0 { (sx / n, sy / n) } else { (0, 0) };
            starts.push((h, centre));
            if log > 0 {
                let _ = writeln!(
                    out,
                    "[{tag} seed {seed}] house {} country {} start {:?} allies {:?} objects {} centre {:?}",
                    h.0,
                    hs.country,
                    hs.start,
                    hs.allies.iter().map(|a| a.0).collect::<Vec<_>>(),
                    n,
                    centre
                );
            }
        }
        if log > 0 {
            for (i, (a, ca)) in starts.iter().enumerate() {
                for (b, cb) in starts.iter().skip(i + 1) {
                    let d2 = (ca.0 - cb.0).pow(2) + (ca.1 - cb.1).pow(2);
                    let _ = writeln!(
                        out,
                        "[{tag} seed {seed}] start distance {}-{}: {} cells",
                        a.0,
                        b.0,
                        isqrt(d2)
                    );
                }
            }
        }
    }
    let map_data = s.loaded.map.clone();
    let game = &mut s.play.game;
    let mut brains: Vec<Brain> = if delay.is_some() {
        players
            .iter()
            .map(|&h| Brain::new(&game.world, &map_data, h))
            .collect()
    } else {
        Vec::new()
    };
    // (tick to run at, house, order)
    let mut queue: VecDeque<(i32, HouseId, Order)> = VecDeque::new();
    let mut in_flight: Vec<Vec<(i32, Order)>> = vec![Vec::new(); players.len()];
    let mut drained = 0usize;
    let mut first_loss: BTreeMap<u8, i32> = BTreeMap::new();
    // M8 step A4 round 8: the first building each house lost, and per
    // house the objects it killed by the victim's house.
    let mut first_building_loss: BTreeMap<u8, i32> = BTreeMap::new();
    let mut kills: BTreeMap<u8, BTreeMap<u8, u32>> = BTreeMap::new();
    let mut no_yard_at: BTreeMap<u8, i32> = BTreeMap::new();
    let mut ever_yard: BTreeMap<u8, i32> = BTreeMap::new();
    let mut lost: BTreeMap<u8, (u32, u32)> = BTreeMap::new();
    let mut given = vec![0u32; players.len()];
    let mut over = None;
    let orders_to = env("GLASNOST_FH_ORDERS_TO", 600i32);
    let watch: Option<(u32, i32, i32)> = std::env::var("GLASNOST_FH_WATCH").ok().and_then(|v| {
        let p: Vec<i32> = v.split(',').filter_map(|x| x.trim().parse().ok()).collect();
        Some((*p.first()? as u32, *p.get(1)?, *p.get(2)?))
    });
    while game.world.tick < ticks {
        let tick = game.world.tick;
        if let Some(d) = delay {
            for (i, b) in brains.iter_mut().enumerate() {
                let h = b.house();
                if glasnost_sim::econ::victory::defeated(&game.world, h) {
                    continue;
                }
                let mut orders = b.think(&game.world);
                // The bot's repeat filter (`BotConfig::dedupe`).
                let window = d + 2; // nocite: our own harness (the bot's window)
                in_flight[i].retain(|(t, _)| tick - *t <= window);
                orders.retain(|o| !in_flight[i].iter().any(|(t, f)| *t != tick && f == o));
                in_flight[i].extend(orders.iter().map(|o| (tick, o.clone())));
                for o in orders {
                    given[i] += 1;
                    if log > 1 && tick < orders_to {
                        let _ = writeln!(out, "[{tag} seed {seed}] t{tick} h{} order {o:?}", h.0);
                    }
                    queue.push_back((tick + d, h, o));
                }
            }
            while queue.front().is_some_and(|(t, _, _)| *t <= tick) {
                let (_, h, o) = queue.pop_front().unwrap();
                game.issue(h, o);
            }
        }
        // Before the step: what every player object is and targets.
        let before: BTreeMap<ObjectId, Seen> = game
            .world
            .objects
            .values()
            .filter(|o| o.on_map && o.owner.is_some_and(|h| players.contains(&h)))
            .map(|o| {
                let c = o.pos.cell();
                (
                    o.id,
                    Seen {
                        owner: o.owner.unwrap(),
                        kind: o.kind,
                        ty: o.type_index,
                        cell: (c.x, c.y),
                        health: o.health,
                    },
                )
            })
            .collect();
        let targets: Vec<(ObjectId, ObjectId)> = game
            .world
            .objects
            .values()
            .filter_map(|o| match o.target {
                Some(Target::Object(t)) => Some((o.id, t)),
                _ => None,
            })
            .collect();
        game.step();
        let w = &game.world;
        if let Some((id, from, to)) = watch
            && (from..=to).contains(&w.tick)
        {
            match w.obj(ObjectId(id)) {
                Some(o) => {
                    let c = o.pos.cell();
                    let _ = writeln!(
                        out,
                        "[{tag} seed {seed}] watch t{} {} at ({},{}) pos {:?} beh {:?} q {:?} dest {:?} target {:?} hp {} path {:?} drive {:?} deploy {:?}",
                        w.tick,
                        id,
                        c.x,
                        c.y,
                        o.pos,
                        o.beh.current,
                        o.beh.queued,
                        o.dest,
                        o.target,
                        o.health,
                        o.path,
                        o.drive.as_ref().map(|d| (
                            d.track.is_some(),
                            d.turning_in_place,
                            d.between_steps,
                            d.goal,
                            d.for_dest,
                            d.claims.clone(),
                            d.requests.clone()
                        )),
                        o.econ.deploy
                    );
                }
                None => {
                    let _ = writeln!(out, "[{tag} seed {seed}] watch t{} {} gone", w.tick, id);
                }
            }
        }
        let mut killed: Vec<ObjectId> = Vec::new();
        for e in w.effects.iter().skip(drained) {
            if let Effect::Killed { id, .. } = e {
                killed.push(*id);
            }
        }
        drained = w.effects.len();
        for (id, b) in &before {
            let now = w.obj(*id);
            let gone = now.is_none_or(|o| !o.on_map || o.health <= 0 || o.owner != Some(b.owner));
            if !gone {
                continue;
            }
            let how = if killed.contains(id) {
                "killed"
            } else if let Some(o) = now {
                if o.owner != Some(b.owner) {
                    "changed owner"
                } else {
                    "left the map"
                }
            } else {
                "removed"
            };
            let by: Vec<String> = targets
                .iter()
                .filter(|(_, t)| t == id)
                .filter_map(|(f, _)| {
                    let fo = w
                        .obj(*f)
                        .map(|o| (o.owner, o.kind, o.type_index))
                        .or_else(|| before.get(f).map(|s| (Some(s.owner), s.kind, s.ty)))?;
                    Some(format!(
                        "{}:{}",
                        fo.0.map_or("-".into(), |h| h.0.to_string()),
                        name(w, fo.1, fo.2)
                    ))
                })
                .collect();
            let e = lost.entry(b.owner.0).or_default();
            if how == "killed" {
                e.0 += 1;
            } else {
                e.1 += 1;
            }
            first_loss.entry(b.owner.0).or_insert(w.tick);
            if b.kind == Kind::Building && how == "killed" {
                first_building_loss.entry(b.owner.0).or_insert(w.tick);
            }
            if how == "killed" {
                let mut by_house: Vec<u8> = targets
                    .iter()
                    .filter(|(_, t)| t == id)
                    .filter_map(|(f, _)| {
                        w.obj(*f)
                            .and_then(|o| o.owner)
                            .or_else(|| before.get(f).map(|s| s.owner))
                    })
                    .filter(|h| players.contains(h) && *h != b.owner)
                    .map(|h| h.0)
                    .collect();
                by_house.sort_unstable();
                by_house.dedup();
                for a in by_house {
                    *kills.entry(a).or_default().entry(b.owner.0).or_default() += 1;
                }
            }
            if log > 0 && (how == "killed" || how == "changed owner" || b.kind != Kind::Infantry) {
                let _ = writeln!(
                    out,
                    "[{tag} seed {seed}] t{} house {} {} {:?} {} at {:?} hp {} {how}{}",
                    w.tick,
                    b.owner.0,
                    id.0,
                    b.kind,
                    name(w, b.kind, b.ty),
                    b.cell,
                    b.health,
                    if by.is_empty() {
                        String::new()
                    } else {
                        format!(" targeted by {}", by.join(","))
                    }
                );
            }
        }
        for &h in &players {
            let yard = w
                .objects
                .values()
                .any(|o| o.owner == Some(h) && o.on_map && glasnost_ai::scripted::is_yard(w, o));
            if yard {
                ever_yard.entry(h.0).or_insert(w.tick);
                no_yard_at.remove(&h.0);
            } else if ever_yard.contains_key(&h.0) {
                no_yard_at.entry(h.0).or_insert(w.tick);
            }
        }
        if let Some(o) = game.outcome() {
            over = Some(o.over);
            break;
        }
    }
    let w = &game.world;
    let mut line = format!("[{tag} seed {seed}] end tick {} over {:?}:", w.tick, over);
    for (i, &h) in players.iter().enumerate() {
        let n = |k: Kind| {
            w.objects
                .values()
                .filter(|o| o.owner == Some(h) && o.on_map && o.health > 0 && o.kind == k)
                .count()
        };
        let (k, r) = lost.get(&h.0).copied().unwrap_or_default();
        let _ = write!(
            line,
            " | h{} team{} yard@{} lostyard@{} inf {} veh {} bld {} killed {} other-gone {} orders {}",
            h.0,
            teams()[i],
            ever_yard.get(&h.0).map_or("-".into(), |t| t.to_string()),
            no_yard_at.get(&h.0).map_or("-".into(), |t| t.to_string()),
            n(Kind::Infantry),
            n(Kind::Vehicle),
            n(Kind::Building),
            k,
            r,
            given[i]
        );
        let _ = write!(
            line,
            " firstbldlost@{} kills-by-victim {:?}",
            first_building_loss
                .get(&h.0)
                .map_or("-".into(), |t| t.to_string()),
            kills.get(&h.0).cloned().unwrap_or_default()
        );
        for o in w.objects.values().filter(|o| {
            o.owner == Some(h)
                && o.kind == Kind::Vehicle
                && glasnost_ai::scripted::deploys_into_yard(w, o.kind, o.type_index)
        }) {
            let c = o.pos.cell();
            let _ = write!(
                line,
                " mcv {} at ({},{}) map {} beh {:?} dest {:?}",
                o.id.0, c.x, c.y, o.on_map, o.beh.current, o.dest
            );
        }
    }
    out.push_str(&line);
    out.push('\n');
    Ok(out)
}

fn isqrt(v: i32) -> i32 {
    let mut r = 0;
    while (r + 1) * (r + 1) <= v {
        r += 1;
    }
    r
}

#[test]
#[ignore = "local lane: needs GLASNOST_GAME_DIR"]
fn four_houses_on_an_install_map() {
    let Ok(dir) = std::env::var("GLASNOST_GAME_DIR") else {
        eprintln!("set GLASNOST_GAME_DIR");
        return;
    };
    let seeds = std::env::var("GLASNOST_FH_SEEDS").unwrap_or_else(|_| "1-8".into());
    let (a, b) = seeds
        .split_once('-')
        .map(|(a, b)| (a.parse().unwrap(), b.parse().unwrap()))
        .unwrap_or_else(|| (seeds.parse().unwrap(), seeds.parse().unwrap()));
    let map = env("GLASNOST_FH_MAP", 30usize);
    let ticks = env("GLASNOST_FH_TICKS", 6000i32);
    let log = env("GLASNOST_FH_LOG", 1u8);
    let delays: Vec<Option<i32>> = std::env::var("GLASNOST_FH_DELAY")
        .unwrap_or_else(|_| "in,0,8".into())
        .split(',')
        .map(|d| d.trim().parse().ok())
        .collect();
    let out_dir =
        std::env::var("GLASNOST_LANE_OUT").unwrap_or_else(|_| "../../local-lane/m9/reh-ai".into());
    let mut all = String::new();
    for seed in a..=b {
        for &d in &delays {
            let mut c = net_client::client(Some(Path::new(&dir))).expect("install");
            match play(&mut c, map, seed, d, ticks, log) {
                Ok(s) => {
                    print!("{s}");
                    all.push_str(&s);
                }
                Err(e) => println!("seed {seed} {d:?}: {e}"),
            }
        }
    }
    let _ = std::fs::create_dir_all(&out_dir);
    let _ = std::fs::write(Path::new(&out_dir).join("four-houses.txt"), all);
}
