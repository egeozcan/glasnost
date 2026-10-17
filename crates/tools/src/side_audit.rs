//! The local lane's side audit (M7a step 4): every building and unit type
//! one side can build in the install's converted rules, with a fingerprint
//! of the flags that give it its behaviour (field names of our schema whose
//! value is set, never a value), and a sandbox game in which a house of that
//! side builds each type in turn as its prerequisites come to stand: whether
//! the type is offered on its sidebar (S-0407 claim 1), is built (S-0404)
//! and is placed (a building, S-0406) or released (a unit, S-0417).
//!
//! The house plays alone against an idle house of another side, with a
//! large credits setting; it deploys its construction vehicle, then orders
//! every offered type of its side that it has not built yet, one per line,
//! places each finished building on the first cell of the own AI's
//! placement spiral (`Script::place_cell`) and releases each finished unit.
//! The report holds kinds, classes, flags, counts and ticks only (no names,
//! no values, no paths). Never in CI.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::Path;
use std::sync::Arc;

use glasnost_ai::scripted::{Plan, Script, all_types, role_of};
use glasnost_rules::schema::Ruleset;
use glasnost_sim::econ::setup::{self, Player, Settings};
use glasnost_sim::econ::{TypeRef, production, tech};
use glasnost_sim::load::new_game;
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Kind, SimRules};
use glasnost_sim::world::{HouseId, World};

/// The house that builds (the first player).
const HOUSE: HouseId = HouseId(0);
/// Ticks the sandbox runs at most.
const MAX_TICKS: i32 = 90_000;
/// The credits setting of the sandbox (enough for every type).
const CREDITS: i32 = 1_000_000;
/// Cells from a factory to its rally point.
const RALLY: i32 = 10;

/// What the sandbox saw of one type.
#[derive(Debug, Default, Clone)]
struct Seen {
    offered: Option<i32>,
    ordered: Option<i32>,
    done: Option<i32>,
    placed: Option<i32>,
}

/// The names of the fields of a type's debug form whose value is `Some(true)`
/// (flags), and of those that hold a name (`Some("…")`): the fingerprint.
fn flags(debug: &str) -> (Vec<String>, Vec<String>) {
    let mut on = Vec::new();
    let mut named = Vec::new();
    for (i, _) in debug.match_indices(": Some(") {
        let head = &debug[..i];
        let name: String = head
            .chars()
            .rev()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();
        let rest = &debug[i + ": Some(".len()..];
        if rest.starts_with("true)") {
            on.push(name);
        } else if rest.starts_with('"') {
            named.push(name);
        }
    }
    on.sort();
    on.dedup();
    named.sort();
    named.dedup();
    (on, named)
}

/// The fingerprint of a weapon by name: its flags and its warhead's flags.
fn weapon_print(rs: &Ruleset, name: &str) -> String {
    let Some(w) = rs.sim.weapons.get(name) else {
        return "missing weapon".into();
    };
    let (won, _) = flags(&format!("{w:?}"));
    let head = w
        .warhead
        .as_ref()
        .and_then(|h| rs.sim.warheads.get(h))
        .map(|h| flags(&format!("{h:?}")).0)
        .unwrap_or_default();
    format!("weapon {won:?} warhead {head:?}")
}

/// One type's line of the fingerprint.
fn fingerprint(w: &World, rs: &Ruleset, t: TypeRef) -> String {
    let r = w.rules();
    let mut s = String::new();
    match t.kind {
        Kind::Building => {
            let Some(b) = r.buildings.get(t.index as usize) else {
                return s;
            };
            let (on, named) = flags(&format!("{:?}", b.raw));
            let _ = write!(s, "flags {on:?} names {named:?}");
            for wp in [&b.raw.primary, &b.raw.secondary].into_iter().flatten() {
                let _ = write!(s, "; {}", weapon_print(rs, wp));
            }
            if let Some(sw) = b
                .raw
                .econ
                .super_weapon
                .as_ref()
                .and_then(|n| rs.sim.superweapons.get(n))
            {
                let _ = write!(s, "; superweapon kind {:?}", sw.kind);
            }
            if let Some(sw) = b
                .raw
                .econ
                .super_weapon2
                .as_ref()
                .and_then(|n| rs.sim.superweapons.get(n))
            {
                let _ = write!(s, "; superweapon 2 kind {:?}", sw.kind);
            }
        }
        _ => {
            let Some(u) = r.unit(t.kind, t.index) else {
                return s;
            };
            let (on, named) = flags(&format!("{:?}", u.raw));
            let _ = write!(
                s,
                "class {} locomotor {:?} flags {on:?} names {named:?}",
                crate::skirmish_lane::unit_class(w, t.kind, t.index),
                u.locomotor
            );
            for wp in [
                &u.raw.primary,
                &u.raw.secondary,
                &u.raw.elite_primary,
                &u.raw.elite_secondary,
                &u.raw.occupy_weapon,
            ]
            .into_iter()
            .flatten()
            {
                let _ = write!(s, "; {}", weapon_print(rs, wp));
            }
        }
    }
    s
}

/// Runs the audit for `side` on map `index` (the map's start positions and
/// terrain decide which buildings find a cell: a naval yard needs water).
pub fn run(dir: &Path, side: i32, index: usize, seed: u32) -> std::io::Result<String> {
    pollster::block_on(run_async(dir, side, index, seed))
}

async fn run_async(dir: &Path, side: i32, index: usize, seed: u32) -> std::io::Result<String> {
    let mut vfs = crate::local_lane::mount(dir).await?;
    let listings = glasnost_convert::find::find_maps(&vfs).await;
    let mut out = String::new();
    let Some(listing) = listings.get(index) else {
        return Ok(format!("side audit: no map with index {index}\n"));
    };
    let bytes = vfs
        .read(&listing.file)
        .await
        .map_err(|e| std::io::Error::other(format!("{e:?}")))?;
    let inputs = glasnost_convert::game::load(&mut vfs, bytes, "install".into()).await;
    let (Ok(resolved), Some(tiles)) = (inputs.resolved, inputs.tiles) else {
        return Ok("side audit: the map's inputs did not resolve\n".into());
    };
    let rs = resolved.ruleset;
    let rules = Arc::new(SimRules::new(&rs, &resolved.sim_rules_hash));
    let tiles = Arc::new(tiles);
    let open = crate::skirmish_lane::countries(&rules);
    let side_of = |c: u32| {
        rules
            .econ
            .tech
            .country_side
            .get(c as usize)
            .copied()
            .flatten()
    };
    let Some((country, cname)) = open
        .iter()
        .find(|(c, _)| side_of(*c) == Some(side))
        .cloned()
    else {
        return Ok(format!("side audit: no open country of side {side}\n"));
    };
    let Some((_, other)) = open
        .iter()
        .find(|(c, _)| side_of(*c) != Some(side))
        .cloned()
    else {
        return Ok("side audit: no open country of another side\n".into());
    };
    let player = |name: &str, country: String| Player {
        name: name.into(),
        country,
        start: None,
        ai: false,
        values: Default::default(),
    };
    let setup = setup::skirmish(
        &rules,
        &[player("Audit", cname), player("Idle", other)],
        Settings {
            credits: Some(CREDITS),
            ..Default::default()
        },
        0,
        seed,
    );
    let mut w = new_game(rules.clone(), &inputs.map, tiles, &setup)
        .map_err(|e| std::io::Error::other(format!("{e:?}")))?;
    let level = tech::house_tech_level(&w, HOUSE);
    // The side's types: a tech level of 0 or more and an owner country of
    // the house (S-0407 claims 1 and 5).
    let mut side_types: Vec<TypeRef> = all_types(&w)
        .into_iter()
        .filter(|&t| {
            rules
                .econ
                .tech
                .of(t)
                .is_some_and(|f| f.tech_level >= 0 && f.owners.contains(&country))
        })
        .collect();
    side_types.sort_by_key(|t| (t.kind as u8, t.index));
    let within: BTreeSet<TypeRef> = side_types
        .iter()
        .copied()
        .filter(|&t| rules.econ.tech.of(t).is_some_and(|f| f.tech_level <= level))
        .collect();

    // The sandbox.
    let script = Script::new(
        HOUSE,
        Plan {
            names: Default::default(),
            harvesters: 2,
            wave: 6,
        },
    );
    let starting: BTreeSet<u32> = w.objects.keys().map(|id| id.0).collect();
    let mut seen: BTreeMap<TypeRef, Seen> =
        side_types.iter().map(|&t| (t, Seen::default())).collect();
    let mut unplaceable: BTreeSet<TypeRef> = BTreeSet::new();
    while w.tick < MAX_TICKS {
        if w.tick % 15 == 0 && !orders::blocked(&w) {
            let mut list: Vec<Order> = Vec::new();
            let own: Vec<_> = w
                .objects
                .values()
                .filter(|o| o.owner == Some(HOUSE) && o.on_map)
                .map(|o| o.id)
                .collect();
            let has_yard = own.iter().any(|&id| {
                w.obj(id)
                    .is_some_and(|o| glasnost_ai::scripted::is_yard(&w, o))
            });
            if !has_yard {
                for &id in &own {
                    if let Some(o) = w.obj(id)
                        && o.kind == Kind::Vehicle
                        && glasnost_ai::scripted::deploys_into_yard(&w, o.kind, o.type_index)
                    {
                        list.push(Order::Deploy { actor: id });
                    }
                }
            }
            // Every factory gets a rally point toward the other house's
            // start, as a person sets one, so that a released unit leaves the
            // door free for the next (S-0417 claim 14).
            let middle = w
                .objects
                .values()
                .find(|o| o.owner == Some(HouseId(1)) && o.on_map)
                .map_or(glasnost_map::CellXY::new(0, 0), |o| o.pos.cell());
            for &id in &own {
                let Some(o) = w.obj(id) else { continue };
                let factory = o.kind == Kind::Building
                    && w.rules()
                        .buildings
                        .get(o.type_index as usize)
                        .is_some_and(|b| b.factory.is_some());
                if factory && o.guard_spot.is_none() {
                    let c = o.pos.cell();
                    let step = |a: i32, b: i32| a + (b - a).clamp(-RALLY, RALLY);
                    list.push(Order::GuardSpot {
                        actor: id,
                        spot: glasnost_sim::world::Target::Cell(glasnost_map::CellXY::new(
                            step(c.x, middle.x),
                            step(c.y, middle.y),
                        )),
                    });
                }
            }
            for l in &w.econ.lines.lines {
                if l.house != HOUSE {
                    continue;
                }
                let Some(item) = &l.item else { continue };
                if item.done_tick.is_none() {
                    continue;
                }
                if item.t.kind == Kind::Building {
                    match script.place_cell(&w, item.t) {
                        Some(c) => list.push(Order::Place {
                            what: item.t,
                            cell: Some(c),
                        }),
                        None => {
                            unplaceable.insert(item.t);
                            list.push(Order::Cancel { what: item.t });
                        }
                    }
                } else {
                    list.push(Order::Place {
                        what: item.t,
                        cell: None,
                    });
                }
            }
            for (&t, s) in seen.iter_mut() {
                let offered = tech::offered(&w, HOUSE, t);
                if offered && s.offered.is_none() {
                    s.offered = Some(w.tick);
                }
                if s.ordered.is_some() && s.placed.is_none() && !unplaceable.contains(&t) {
                    continue;
                }
                if s.ordered.is_none()
                    && offered
                    && tech::buildable(&w, HOUSE, t)
                    && !unplaceable.contains(&t)
                    && production::line_of(&w, HOUSE, t)
                        .is_none_or(|l| l.item.is_none() && l.queue.is_empty())
                    && !list
                        .iter()
                        .any(|o| matches!(o, Order::Build { what } if production::line_kind(&w, *what) == production::line_kind(&w, t)))
                {
                    list.push(Order::Build { what: t });
                    s.ordered = Some(w.tick);
                }
            }
            for o in list {
                orders::issue(&mut w, HOUSE, o);
            }
        }
        glasnost_sim::tick::step(&mut w);
        for l in &w.econ.lines.lines {
            if l.house == HOUSE
                && let Some(item) = &l.item
                && item.done_tick.is_some()
                && let Some(s) = seen.get_mut(&item.t)
                && s.done.is_none()
            {
                s.done = Some(w.tick);
            }
        }
        for o in w.objects.values() {
            if o.owner == Some(HOUSE) && o.on_map && !starting.contains(&o.id.0) {
                let t = tech::type_of(o);
                if let Some(s) = seen.get_mut(&t)
                    && s.placed.is_none()
                {
                    s.placed = Some(w.tick);
                    s.done.get_or_insert(w.tick);
                }
            }
        }
        w.effects.clear();
        if seen
            .iter()
            .all(|(t, s)| s.placed.is_some() || !within.contains(t) || unplaceable.contains(t))
            && w.tick > 3000
        {
            break;
        }
    }

    // The report.
    let count = |k: Kind| side_types.iter().filter(|t| t.kind == k).count();
    let _ = writeln!(
        out,
        "glasnost-headless side-audit: side {side}, map index {index}, seed {seed}; house tech level {level}; {} types of the side (buildings {}, infantry {}, vehicles {}, aircraft {}), {} within the house tech level; sandbox ran {} ticks",
        side_types.len(),
        count(Kind::Building),
        count(Kind::Infantry),
        count(Kind::Vehicle),
        count(Kind::Aircraft),
        within.len(),
        w.tick
    );
    let (mut offered, mut built, mut placed) = (0, 0, 0);
    for (n, &t) in side_types.iter().enumerate() {
        let s = &seen[&t];
        offered += usize::from(s.offered.is_some());
        built += usize::from(s.done.is_some());
        placed += usize::from(s.placed.is_some());
        let role = role_of(&w, t).map_or("none".into(), |r| format!("{r:?}"));
        let reason = if !within.contains(&t) {
            "above the house tech level".to_string()
        } else if s.offered.is_none() {
            let unmet = tech::unmet_prerequisites(&w, HOUSE, t).unwrap_or_default();
            // Whether a building type the side was offered meets each
            // unmet entry (otherwise only another side's building does).
            let side_meets: Vec<String> = unmet
                .iter()
                .map(|&p| {
                    let meeting: Vec<u32> = match p {
                        tech::Prereq::Building(i) => vec![i],
                        tech::Prereq::Group(g) => rules.econ.tech.groups[g as usize].clone(),
                        _ => Vec::new(),
                    };
                    let side = meeting.iter().any(|&i| {
                        let b = TypeRef {
                            kind: Kind::Building,
                            index: i,
                        };
                        seen.get(&b).is_some_and(|x| x.offered.is_some())
                    });
                    let listed: Vec<String> = meeting
                        .iter()
                        .filter_map(|&i| {
                            side_types
                                .iter()
                                .position(|x| x.kind == Kind::Building && x.index == i)
                                .map(|n| format!("#{n}"))
                        })
                        .collect();
                    if side {
                        "a side building".to_string()
                    } else if listed.is_empty() {
                        "another side's building".to_string()
                    } else {
                        format!("never offered here: {}", listed.join(" or "))
                    }
                })
                .collect();
            let f = rules.econ.tech.of(t).cloned().unwrap_or_default();
            format!(
                "never offered (answer {:?}, unmet prerequisites met by {side_meets:?}; required countries {}, forbidden {}, stolen tech {:?})",
                tech::answer(&w, HOUSE, t),
                f.required.as_ref().map_or("any".into(), |v| format!(
                    "{} (house's {})",
                    v.len(),
                    v.contains(&country)
                )),
                f.forbidden.as_ref().map_or("none".into(), |v| format!(
                    "{} (house's {})",
                    v.len(),
                    v.contains(&country)
                )),
                f.stolen
            )
        } else if unplaceable.contains(&t) {
            "no cell on this map".into()
        } else if s.placed.is_none() {
            "not placed or released".into()
        } else {
            "ok".into()
        };
        let _ = writeln!(
            out,
            "  #{n} {:?} role {role}: offered {:?} ordered {:?} done {:?} placed/released {:?}: {reason}; {}",
            t.kind,
            s.offered,
            s.ordered,
            s.done,
            s.placed,
            fingerprint(&w, &rs, t)
        );
    }
    let _ = writeln!(
        out,
        "  offered {offered}, built {built}, placed or released {placed} of {} ({} within the tech level)",
        side_types.len(),
        within.len()
    );
    Ok(out)
}
