//! Local lane for the M7a features (never in CI): with `GLASNOST_GAME_DIR`
//! set, every scene of [`SCENES`] starts its own skirmish through the
//! windowless client, puts the objects it needs on the map by their rules
//! flags (never by type name), plays the scene and writes its screens to
//! `local-lane/m7a/` (git-ignored; `GLASNOST_LANE_OUT` overrides the
//! folder). Prints counts, cells and ticks only, never type names or rules
//! values.
//!
//! Scenes now (S-0610, psychic weapons): a unit with a berserk weapon turns
//! an enemy vehicle berserk among its fellows; a psychic sensor shows the
//! hostile-intent line of an enemy heading into its reach; a psychic reveal
//! uncovers an area of a shrouded map. S-0601 (mind control): a
//! controller takes an enemy vehicle over (the link line and the ring),
//! then a second, and its death releases both; an unlimited controller
//! taken past its overload counts pulses. S-0609 (gattling spin-up): a
//! gattling vehicle and a gattling building fire at enemies and climb their
//! stages; the vehicle's target removed, it spins down. S-0613 (gas
//! clouds): a unit whose weapon kills with the virus death fires at enemy
//! infantry; the deaths leave clouds that drift and hit what stands in them.
//! S-0605 (infantry absorber): three infantry are clicked into an absorber
//! one at a time and the house's output rises; deployed, it lets them out.
//! S-0603 (grinder): three infantry, an own vehicle and a mind-controlled
//! enemy vehicle are clicked into a recycling building and consumed.
//! S-0606 (vehicle bunker): a turreted vehicle is clicked into a tank
//! bunker, drives in, fires at an enemy from inside and is let out by the
//! deploy click. S-0607 (magnetic lift): a lifter is clicked onto an enemy
//! vehicle, seizes it and pulls it up; a move click on the lifter lets go
//! over a second enemy vehicle, and the falling one is destroyed on impact.
//!
//! Run with `cargo test --release -p glasnost-app --test m7a_lane --
//! --ignored --nocapture`. `GLASNOST_LANE_SCENE=a,b` plays only the scenes
//! with those keys; `GLASNOST_UI_MAP=n` picks the n-th offered map.

#![cfg(not(target_arch = "wasm32"))]
#![allow(dead_code)]

use std::rc::Rc;

use glasnost_app::game::client::{Client, run_job};
use glasnost_app::mapview::png;
use glasnost_map::CellXY;
use glasnost_map::terrain::land;
use glasnost_sim::rules::Kind;
use glasnost_sim::targeting::{self, ClickOn, Modifiers};
use glasnost_sim::world::{HouseId, Layer, ObjectId, Target, World};
use glasnost_ui::geom::Rect;
use glasnost_ui::menu::MenuHit;
use glasnost_ui::setup::SetupHit;
use glasnost_ui::shell::Screen;
use glasnost_vfs::source::{FileSource, Unopened};
use glasnost_vfs::{ByteSource, MountOptions, PickedFile};

/// A scene: its key for `GLASNOST_LANE_SCENE` and the function that plays
/// it on a freshly started skirmish.
type Scene = (&'static str, fn(&mut Lane));

/// The scenes, in the order they are played.
const SCENES: &[Scene] = &[
    ("berserk", berserk_spreads),
    ("berserk-mask", berserk_mask),
    ("sensor", sensor_shows_intent),
    ("reveal", psychic_reveal),
    ("mind", mind_control),
    ("overload", mind_overload),
    ("gattling", gattling_spin),
    ("gas", gas_clouds),
    ("absorber", absorber),
    ("grinder", grinder),
    ("bunker", bunker),
    ("lift", magnetic_lift),
];

#[test]
#[ignore = "local lane: needs GLASNOST_GAME_DIR"]
fn m7a_scenes() {
    if std::env::var_os("GLASNOST_GAME_DIR").is_none() {
        eprintln!("GLASNOST_GAME_DIR not set: skipped");
        return;
    }
    let only: Vec<String> = std::env::var("GLASNOST_LANE_SCENE")
        .map(|v| v.split(',').map(|s| s.trim().to_string()).collect())
        .unwrap_or_default();
    for (key, play) in SCENES {
        if !only.is_empty() && !only.iter().any(|o| o == key) {
            continue;
        }
        println!("== scene {key}");
        let mut lane = Lane::start().expect("the install is set");
        play(&mut lane);
    }
}

// ---------------------------------------------------------------------------
// Scenes

/// An armed vehicle type that is neither psionics-immune nor a harvester,
/// for the enemy group.
fn plain_vehicle(w: &World) -> Option<String> {
    let r = w.rules();
    (0..r.vehicles.len())
        .find(|&i| {
            r.unit(Kind::Vehicle, i as u32).is_some_and(|u| {
                u.combat.slots[0].is_some()
                    && !u.harvester
                    && u.strength > 0
                    && !u.combat.immune_psionics
                    && u.passengers == 0
            })
        })
        .map(|i| r.vehicles[i].name.clone())
}

/// S-0610 claims 3 to 6: a unit whose first weapon has a berserk-causing
/// warhead is ordered (by the click a person gives) at one of three enemy
/// vehicles standing together; the hit vehicle goes berserk, turns on its
/// fellows and ends berserk when its time runs out.
fn berserk_spreads(lane: &mut Lane) {
    let w = lane.world();
    let r = w.rules();
    let shooter = (0..r.infantry.len())
        .map(|i| (Kind::Infantry, i as u32))
        .chain((0..r.vehicles.len()).map(|i| (Kind::Vehicle, i as u32)))
        .find(|&(k, i)| {
            r.unit(k, i).is_some_and(|u| {
                u.combat.slots[0]
                    .and_then(|p| r.combat.weapon(p).warhead)
                    .is_some_and(|h| r.combat.warhead(h).psychedelic)
            })
        })
        .map(|(k, i)| (k, r.unit(k, i).map(|u| u.name.clone()).unwrap_or_default()));
    let victim = plain_vehicle(w);
    println!(
        "berserk-weapon type found: {}, enemy vehicle type found: {}",
        shooter.is_some(),
        victim.is_some()
    );
    let (Some((kind, sname)), Some(vname)) = (shooter, victim) else {
        return;
    };
    let local = lane.local();
    let Some(enemy) = lane.enemy() else { return };
    let home = lane.home();
    let spot = lane.open_land(CellXY::new(home.x + 8, home.y + 8));
    let s = lane.spawn(kind, &sname, local, CellXY::new(spot.x - 3, spot.y));
    let vs: Vec<ObjectId> = [(1, 0), (2, 0), (1, 1)]
        .iter()
        .map(|&(dx, dy)| {
            lane.spawn(
                Kind::Vehicle,
                &vname,
                enemy,
                CellXY::new(spot.x + dx, spot.y + dy),
            )
        })
        .collect();
    lane.look_at(spot);
    lane.ticks(4);
    lane.save("berserk-before");
    let sent = lane.order(local, s, ClickOn::Object(vs[0]));
    println!("click on the target: {sent} orders");
    let mut went = None;
    let mut shot = false;
    for t in 0..3000 {
        lane.ticks(1);
        let w = lane.world();
        let mad: Vec<ObjectId> = vs
            .iter()
            .copied()
            .filter(|&v| glasnost_sim::psychic::berserk(w, v))
            .collect();
        if went.is_none() && !mad.is_empty() {
            went = Some(t);
            println!("first enemy berserk at tick {t}; berserk now {}", mad.len());
            // A person stops the shooter so the berserk vehicle can be watched.
            let _ = lane.order(local, s, ClickOn::Cell(CellXY::new(spot.x - 6, spot.y)));
        }
        if let Some(g) = went
            && !shot
            && t >= g + 20
        {
            shot = true;
            let w = lane.world();
            let targets = vs
                .iter()
                .filter(|&&v| {
                    w.obj(v).is_some_and(|o| {
                        matches!(o.target, Some(Target::Object(x)) if w.obj(x).is_some_and(|xo| xo.owner == Some(enemy)))
                    })
                })
                .count();
            println!("20 ticks later: enemy vehicles aiming at their own side {targets}");
            lane.save("berserk-turned");
        }
        if went.is_some()
            && vs
                .iter()
                .all(|&v| !glasnost_sim::psychic::berserk(lane.world(), v))
        {
            println!("berserk over at tick {t}");
            break;
        }
    }
    lane.ticks(3);
    lane.save("berserk-after");
}

/// S-0610 claim 7: the berserk colour mask side by side. Two enemy
/// vehicles and two enemy infantry of one type each are placed in a row;
/// one of each pair is made berserk directly (S-0610 claim 4's state
/// start), so the berserk unit and its plain twin are drawn in one shot.
fn berserk_mask(lane: &mut Lane) {
    let w = lane.world();
    let vname = plain_vehicle(w);
    let iname = human_infantry(w);
    println!(
        "vehicle type found: {}, infantry type found: {}",
        vname.is_some(),
        iname.is_some()
    );
    let Some(enemy) = lane.enemy() else { return };
    let home = lane.home();
    let spot = lane.open_land(CellXY::new(home.x + 8, home.y + 8));
    let mut pairs = Vec::new();
    if let Some(v) = vname {
        let a = lane.spawn(Kind::Vehicle, &v, enemy, CellXY::new(spot.x, spot.y));
        let b = lane.spawn(Kind::Vehicle, &v, enemy, CellXY::new(spot.x + 2, spot.y));
        pairs.push((a, b));
    }
    if let Some(i) = iname {
        let a = lane.spawn(Kind::Infantry, &i, enemy, CellXY::new(spot.x, spot.y + 2));
        let b = lane.spawn(
            Kind::Infantry,
            &i,
            enemy,
            CellXY::new(spot.x + 2, spot.y + 2),
        );
        pairs.push((a, b));
    }
    lane.look_at(CellXY::new(spot.x + 1, spot.y + 1));
    lane.ticks(2);
    for &(_, b) in &pairs {
        let took = glasnost_sim::psychic::enter(lane.world_mut(), b, 900);
        println!("berserk started: {took}");
    }
    lane.ticks(1);
    let w = lane.world();
    println!(
        "berserk now: {:?}",
        pairs
            .iter()
            .map(|&(a, b)| (
                glasnost_sim::psychic::berserk(w, a),
                glasnost_sim::psychic::berserk(w, b)
            ))
            .collect::<Vec<_>>()
    );
    lane.save("berserk-mask");
}

/// S-0610 claims 9 to 12: a psychic sensor (a building type with a
/// psychic-detection radius) is placed for the local house; an enemy
/// vehicle far outside its reach is sent to a cell inside it; the
/// hostile-intent line is drawn with its end squares.
fn sensor_shows_intent(lane: &mut Lane) {
    let w = lane.world();
    let r = w.rules();
    let sensor = (0..r.buildings.len())
        .filter(|&b| {
            r.buildings[b]
                .raw
                .econ
                .psychic_detection_radius
                .unwrap_or(0)
                > 0
        })
        .min_by_key(|&b| (r.buildings[b].foundation.0 * r.buildings[b].foundation.1, b))
        .map(|b| (r.buildings[b].name.clone(), r.buildings[b].foundation));
    let mover = plain_vehicle(w);
    println!(
        "sensor type found: {}, enemy vehicle type found: {}",
        sensor.is_some(),
        mover.is_some()
    );
    let (Some((bname, size)), Some(vname)) = (sensor, mover) else {
        return;
    };
    let local = lane.local();
    let Some(enemy) = lane.enemy() else { return };
    let home = lane.home();
    let Some(site) = lane.area(CellXY::new(home.x + 6, home.y + 6), size, 2, |l| {
        l != land::WATER && l != land::ROCK
    }) else {
        println!("no open ground");
        return;
    };
    let b = lane.spawn(Kind::Building, &bname, local, site);
    // Power for the sensor: the local house's strongest power plant type, beside it.
    let plant = {
        let r = lane.world().rules();
        (0..r.buildings.len())
            .filter(|&p| r.buildings[p].raw.econ.power.unwrap_or(0) > 0)
            .max_by_key(|&p| {
                (
                    r.buildings[p].raw.econ.power.unwrap_or(0),
                    std::cmp::Reverse(p),
                )
            })
            .map(|p| (r.buildings[p].name.clone(), r.buildings[p].foundation))
    };
    if let Some((pname, psize)) = plant
        && let Some(at) = lane.area(CellXY::new(site.x - 4, site.y), psize, 1, |l| {
            l != land::WATER && l != land::ROCK
        })
    {
        lane.spawn(Kind::Building, &pname, local, at);
    }
    lane.ticks(2);
    let radius = glasnost_sim::psychic::sensor_radius(lane.world(), b);
    println!(
        "sensor placed at {site:?}; in operation {}; shows a ring {}",
        glasnost_sim::econ::power::in_operation(lane.world(), b),
        glasnost_sim::psychic::range_ring(lane.world(), b).is_some()
    );
    let reach = radius.unwrap_or(0);
    let far = lane.open_land(CellXY::new(site.x + reach + 6, site.y + 2));
    let v = lane.spawn(Kind::Vehicle, &vname, enemy, far);
    let goal = lane.open_land(CellXY::new(site.x + 1, site.y + size.1 + 1));
    let w = lane.world();
    let r = targeting::click(w, enemy, &[v], ClickOn::Cell(goal), Modifiers::default());
    let mut n = 0;
    for o in r.orders {
        n += usize::from(glasnost_sim::orders::issue(lane.world_mut(), enemy, o));
    }
    println!("enemy move order: {n}");
    lane.look_at(CellXY::new((site.x + far.x) / 2, (site.y + far.y) / 2));
    lane.ticks(3);
    let lines = glasnost_sim::psychic::intent_lines(lane.world(), local).len();
    println!(
        "intent lines for the local house: {lines}; the enemy vehicle has a destination {}",
        lane.world().obj(v).is_some_and(|o| o.dest.is_some())
    );
    lane.save("sensor-intent");
    lane.ticks(40);
    let lines = glasnost_sim::psychic::intent_lines(lane.world(), local).len();
    println!("40 ticks later: intent lines {lines}");
    lane.save("sensor-intent-later");
}

/// S-0610 claims 13 to 15: the local house's map is shrouded again; the
/// psychic reveal's effect (with the install's radius) is made at a cell
/// near an enemy start; the area is explored with no lasting sight.
fn psychic_reveal(lane: &mut Lane) {
    let local = lane.local();
    let w = lane.world();
    let sw = w
        .rules()
        .superweapons
        .types
        .iter()
        .position(|t| t.kind == Some(glasnost_sim::superweapons::SwKind::PsychicReveal));
    println!("psychic reveal type found: {}", sw.is_some());
    let Some(i) = sw else { return };
    let target = lane
        .enemy()
        .and_then(|e| {
            let w = lane.world();
            w.objects
                .values()
                .find(|o| o.on_map && o.owner == Some(e))
                .map(|o| o.pos.cell())
        })
        .unwrap_or_else(|| {
            let h = lane.home();
            CellXY::new(h.x + 12, h.y + 12)
        });
    glasnost_sim::econ::shroud::crate_darkness(lane.world_mut(), local);
    lane.ticks(2);
    lane.look_at(target);
    lane.save("reveal-before");
    glasnost_sim::psychic::reveal(lane.world_mut(), local, i as u32, target);
    let explored = |w: &World| {
        w.cells
            .grid
            .iter()
            .filter(|(c, _)| glasnost_sim::econ::shroud::explored(w, local, *c))
            .count()
    };
    println!(
        "explored cells after the reveal: {}",
        explored(lane.world())
    );
    lane.ticks(2);
    lane.save("reveal-after");
    lane.ticks(60);
    println!("explored cells 60 ticks later: {}", explored(lane.world()));
}

/// The first armed type with the gattling flag of `kind` (vehicles or
/// buildings), by index: its name and, for a building, its foundation.
fn gattling_type(w: &World, kind: Kind) -> Option<(String, (i32, i32))> {
    let r = w.rules();
    if kind == Kind::Building {
        return (0..r.buildings.len())
            .find(|&b| r.buildings[b].combat.gattling && r.buildings[b].combat.slots[0].is_some())
            .map(|b| (r.buildings[b].name.clone(), r.buildings[b].foundation));
    }
    (0..r.vehicles.len())
        .find(|&i| {
            r.unit(Kind::Vehicle, i as u32)
                .is_some_and(|u| u.combat.gattling && u.combat.slots[0].is_some())
        })
        .map(|i| (r.vehicles[i].name.clone(), (1, 1)))
}

/// Follows `g`'s spin for up to `ticks` ticks, keeping `g` and `targets`
/// at full health (so the scene shows the climb, not a kill); saves a screen the
/// first time each stage is reached as `{name}-stage{n}`. Returns the
/// (tick, stage) changes.
fn follow_spin(
    lane: &mut Lane,
    g: ObjectId,
    targets: &[ObjectId],
    ticks: usize,
    name: &str,
) -> Vec<(i32, i32)> {
    let mut changes = Vec::new();
    let mut last = glasnost_sim::gattling::spin(lane.world(), g).stage;
    let mut saved = std::collections::BTreeSet::new();
    // GLASNOST_TRACE_SHOTS=n: also saves the first n frames that show a
    // shot animation of `g` at one of its fire points (S-0301 claim 22).
    let mut shots = std::env::var("GLASNOST_TRACE_SHOTS")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(0);
    let mut shot_n = 0;
    for _ in 0..ticks {
        for &t in targets.iter().chain([&g]) {
            let full = lane.full_health(t);
            if let Some(o) = lane.world_mut().obj_mut(t) {
                o.health = full;
            }
        }
        lane.ticks(1);
        let Some(o) = lane.world().obj(g) else {
            println!("{name}: gone at tick {}", lane.world().tick);

            break;
        };
        let aim = o.target;
        if shots > 0 {
            let w = lane.world();
            let own = (o.pos.x, o.pos.y, o.pos.z);
            let points: Vec<_> = (0..18)
                .flat_map(|slot| {
                    [0, 1].map(|k| glasnost_sim::combat::position::fire_point(w, g, slot, k))
                })
                .map(|p| (p.x, p.y, p.z))
                .filter(|&p| p != own)
                .collect();
            let s = lane.c.session.as_ref().unwrap();
            let live = s.loaded.live.as_ref().expect("a live game");
            if live.playing_anims().iter().any(|(_, p)| points.contains(p)) {
                shots -= 1;
                shot_n += 1;
                lane.save(&format!("{name}-shot{shot_n}"));
            }
        }
        let s = glasnost_sim::gattling::spin(lane.world(), g);
        if lane.world().tick % 300 == 0 {
            println!(
                "{name} tick {}: value {} stage {} target set {} targets alive {}",
                lane.world().tick,
                s.value,
                s.stage,
                aim.is_some(),
                targets
                    .iter()
                    .filter(|&&t| lane.world().obj(t).is_some())
                    .count()
            );
        }
        if s.stage != last {
            changes.push((lane.world().tick, s.stage));
            last = s.stage;
            if saved.insert(s.stage) {
                lane.save(&format!("{name}-stage{}", s.stage));
            }
        }
        if glasnost_sim::gattling::at_top(lane.world(), g) && saved.contains(&s.stage) {
            lane.ticks(2);
            break;
        }
    }
    changes
}

/// S-0609 claims 2 to 5, 7, 9 and 14: a gattling vehicle of the local house
/// is ordered (by the click a person gives) at an enemy vehicle beside it
/// and climbs its stages while it fires; with the enemy removed it spins
/// down over the passive spin-down. A gattling building beside an enemy
/// vehicle climbs its stages by itself (guard, then attack).
fn gattling_spin(lane: &mut Lane) {
    let w = lane.world();
    let unit = gattling_type(w, Kind::Vehicle);
    let tower = gattling_type(w, Kind::Building);
    let victim = plain_vehicle(w);
    println!(
        "gattling vehicle type found: {}, gattling building type found: {}, enemy vehicle type found: {}",
        unit.is_some(),
        tower.is_some(),
        victim.is_some()
    );
    let Some(vname) = victim else { return };
    let local = lane.local();
    let Some(enemy) = lane.enemy() else { return };
    let home = lane.home();
    if let Some((gname, _)) = unit {
        let spot = lane.open_land(CellXY::new(home.x + 8, home.y + 8));
        let g = lane.spawn(
            Kind::Vehicle,
            &gname,
            local,
            CellXY::new(spot.x - 2, spot.y),
        );
        let v = lane.spawn(
            Kind::Vehicle,
            &vname,
            enemy,
            CellXY::new(spot.x + 1, spot.y),
        );
        lane.look_at(spot);
        lane.ticks(2);
        lane.save("gattling-before");
        let sent = lane.order(local, g, ClickOn::Object(v));
        println!("click on the enemy vehicle: {sent} orders");
        let changes = follow_spin(lane, g, &[v], 2400, "gattling-vehicle");
        let s = glasnost_sim::gattling::spin(lane.world(), g);
        println!(
            "vehicle stage changes (tick, stage): {changes:?}; now value {} stage {}, at top {}",
            s.value,
            s.stage,
            glasnost_sim::gattling::at_top(lane.world(), g)
        );
        glasnost_sim::objects::remove(lane.world_mut(), v);
        let start = lane.world().tick;
        let mut down = Vec::new();
        let mut last = s.stage;
        for _ in 0..600 {
            lane.ticks(1);
            let s = glasnost_sim::gattling::spin(lane.world(), g);
            if s.stage != last {
                down.push((lane.world().tick - start, s.stage));
                last = s.stage;
            }
            if s.value == 0 {
                break;
            }
        }
        let s = glasnost_sim::gattling::spin(lane.world(), g);
        println!(
            "target removed: stage falls (ticks after, stage) {down:?}; value {} after {} ticks",
            s.value,
            lane.world().tick - start
        );
        lane.save("gattling-spun-down");
        glasnost_sim::objects::remove(lane.world_mut(), g);
    }
    if let Some((bname, size)) = tower {
        let Some(site) = lane.area(CellXY::new(home.x + 6, home.y - 6), size, 2, |l| {
            l != land::WATER && l != land::ROCK
        }) else {
            println!("no open ground for the building");
            return;
        };
        let b = lane.spawn(Kind::Building, &bname, local, site);
        // Power for the building: the strongest power plant type, beside it.
        let plant = {
            let r = lane.world().rules();
            (0..r.buildings.len())
                .filter(|&p| r.buildings[p].raw.econ.power.unwrap_or(0) > 0)
                .max_by_key(|&p| {
                    (
                        r.buildings[p].raw.econ.power.unwrap_or(0),
                        std::cmp::Reverse(p),
                    )
                })
                .map(|p| (r.buildings[p].name.clone(), r.buildings[p].foundation))
        };
        if let Some((pname, psize)) = plant
            && let Some(at) = lane.area(CellXY::new(site.x - 5, site.y), psize, 1, |l| {
                l != land::WATER && l != land::ROCK
            })
        {
            lane.spawn(Kind::Building, &pname, local, at);
        }
        let at = lane.open_land(CellXY::new(site.x + size.0 + 3, site.y));
        let v = lane.spawn(Kind::Vehicle, &vname, enemy, at);
        lane.look_at(site);
        lane.ticks(2);
        let changes = follow_spin(lane, b, &[v], 3600, "gattling-building");
        let s = glasnost_sim::gattling::spin(lane.world(), b);
        println!(
            "building stage changes (tick, stage): {changes:?}; now value {} stage {}, at top {}",
            s.value,
            s.stage,
            glasnost_sim::gattling::at_top(lane.world(), b)
        );
        lane.save("gattling-building");
    }
}

/// The first infantry type whose primary weapon's warhead kills with the
/// virus death (infantry-death kind 8, S-0107 claim 17), and whether that
/// warhead also names a particle system (S-0613 claim 4).
fn virus_type(w: &World) -> Option<(String, bool)> {
    let r = w.rules();
    (0..r.infantry.len() as u32).find_map(|i| {
        let u = r.unit(Kind::Infantry, i)?;
        let p = u.combat.slots[0]?;
        let h = r.combat.weapon(p).warhead?;
        let wh = r.combat.warhead(h);
        // spec: S-0107 claim 17 (infantry-death kind 8, the virus death)
        (wh.inf_death == 8).then(|| (u.name.clone(), wh.particle.is_some()))
    })
}

/// A human, armed infantry type that the virus death applies to (S-0613
/// claim 2), for the enemy group.
fn human_infantry(w: &World) -> Option<String> {
    let r = w.rules();
    (0..r.infantry.len() as u32)
        .find(|&i| {
            r.unit(Kind::Infantry, i).is_some_and(|u| {
                u.combat.slots[0].is_some()
                    && u.strength > 0
                    && !u.raw.not_human.unwrap_or(false)
                    && r.combat
                        .weapon(u.combat.slots[0].unwrap())
                        .warhead
                        // spec: S-0107 claim 17 (infantry-death kind 8, the virus death)
                        .is_none_or(|h| r.combat.warhead(h).inf_death != 8)
            })
        })
        .map(|i| r.infantry[i as usize].name.clone())
}

/// An infantry absorber type (S-0605 Terms) with room, and its foundation.
fn absorber_type(w: &World) -> Option<(String, (i32, i32))> {
    w.rules()
        .buildings
        .iter()
        .find(|b| b.absorb.infantry && b.absorb.capacity > 0)
        .map(|b| (b.name.clone(), b.foundation))
}

/// S-0605 claims 1, 3 to 7, 10 to 13: an infantry absorber of the local
/// house; three infantry are ordered in by the click a person gives (one
/// link place: they go in one at a time) and the house's power rises with
/// each; the building is ordered to deploy and lets them out at its exits.
/// Counts and four screens.
fn absorber(lane: &mut Lane) {
    let w = lane.world();
    let ty = absorber_type(w);
    let inf = human_infantry(w);
    println!(
        "absorber type found: {}, infantry type found: {}",
        ty.is_some(),
        inf.is_some()
    );
    let (Some((bname, size)), Some(iname)) = (ty, inf) else {
        return;
    };
    let local = lane.local();
    let home = lane.home();
    let near = CellXY::new(home.x + 8, home.y + 8);
    let Some(at) = lane.area(near, size, 3, |l| l != land::WATER && l != land::ROCK) else {
        println!("no room for the absorber");
        return;
    };
    let b = lane.spawn(Kind::Building, &bname, local, at);
    lane.ticks(4);
    let power = |lane: &Lane| glasnost_sim::econ::power::totals(lane.world(), local);
    let start = power(lane);
    println!("absorber placed");
    let row = at.y + size.1 + 3;
    let units: Vec<ObjectId> = (0..3)
        .map(|i| lane.spawn(Kind::Infantry, &iname, local, CellXY::new(at.x + i, row)))
        .collect();
    lane.look_at(CellXY::new(at.x + 1, at.y + 2));
    lane.ticks(2);
    lane.save("absorber-before");
    lane.print_slots(b);
    for &u in &units {
        let n = lane.order(local, u, ClickOn::Object(b));
        let wait = glasnost_sim::structures::absorb::waiting_for(lane.world(), u).is_some();
        println!("click on the absorber: {n} orders");
        lane.ticks(1);
        println!(
            "  contact {}, waiting in line {}",
            lane.world().obj(u).and_then(|o| o.contact) == Some(b),
            wait || glasnost_sim::structures::absorb::waiting_for(lane.world(), u).is_some()
        );
    }
    let mut most_contacts = 0;
    let mut shot = false;
    for t in 0..1200 {
        lane.ticks(1);
        let w = lane.world();
        most_contacts = most_contacts.max(glasnost_sim::structures::absorb::contacts(w, b).len());
        let n = glasnost_sim::structures::absorb::count(w, b);
        if n == 1 && !shot {
            println!(
                "first inside after {t} ticks; output rose {}",
                power(lane).0 > start.0
            );
            lane.save("absorber-first-in");
            lane.print_slots(b);
            shot = true;
        }
        if n == 3 {
            println!("all three inside after {t} ticks");
            break;
        }
    }
    println!(
        "passengers {}; most contacts at once {most_contacts}; output rose {}",
        glasnost_sim::structures::absorb::count(lane.world(), b),
        power(lane).0 > start.0
    );
    lane.c.session.as_mut().unwrap().play.selection = vec![b];
    lane.ticks(1);
    lane.save("absorber-full-selected");
    lane.print_slots(b);
    let n = lane.order(local, b, ClickOn::Object(b));
    println!("deploy click on the absorber: {n} orders");
    for t in 0..600 {
        lane.ticks(1);
        if glasnost_sim::structures::absorb::count(lane.world(), b) == 0 {
            println!("empty after {t} ticks");
            break;
        }
    }
    lane.ticks(20);
    let out: Vec<_> = units
        .iter()
        .filter_map(|&u| {
            lane.world()
                .obj(u)
                .filter(|o| o.on_map)
                .map(|o| o.pos.cell())
        })
        .collect();
    println!(
        "released {} of 3 at {out:?} (building at {at:?}, size {size:?}); output back {}",
        out.len(),
        power(lane).0 == start.0
    );
    lane.save("absorber-released");
    lane.print_slots(b);
}

/// A recycling building type (S-0603 Terms: the grinding flag) and its
/// foundation.
fn grinder_type(w: &World) -> Option<(String, (i32, i32))> {
    w.rules()
        .buildings
        .iter()
        .find(|b| b.raw.grinding.unwrap_or(false))
        .map(|b| (b.name.clone(), b.foundation))
}

/// S-0603 claims 1, 3 to 5, 8 and 9: a recycling building of the local
/// house; three infantry, an own vehicle and an enemy vehicle taken over by
/// a controller (set up directly) are sent in by the click a person gives
/// (the recycle cursor); each is consumed on arriving in a foundation cell
/// and the house is paid. Counts, ticks and three screens; no money values.
fn grinder(lane: &mut Lane) {
    let w = lane.world();
    let ty = grinder_type(w);
    let inf = human_infantry(w);
    let veh = plain_vehicle(w);
    let ctl = controller_type(w, false);
    println!(
        "grinder type found: {}, infantry type found: {}, vehicle type found: {}, controller type found: {}",
        ty.is_some(),
        inf.is_some(),
        veh.is_some(),
        ctl.is_some()
    );
    let (Some((gname, size)), Some(iname), Some(vname)) = (ty, inf, veh) else {
        return;
    };
    let local = lane.local();
    let home = lane.home();
    let near = CellXY::new(home.x + 8, home.y + 8);
    let Some(at) = lane.area(near, size, 4, |l| l != land::WATER && l != land::ROCK) else {
        println!("no room for the grinder");
        return;
    };
    let b = lane.spawn(Kind::Building, &gname, local, at);
    lane.ticks(4);
    println!("grinder placed");
    let row = at.y + size.1 + 3;
    let mut units: Vec<ObjectId> = (0..3)
        .map(|i| lane.spawn(Kind::Infantry, &iname, local, CellXY::new(at.x + i, row)))
        .collect();
    units.push(lane.spawn(Kind::Vehicle, &vname, local, CellXY::new(at.x + 4, row)));
    let mut taken = false;
    if let (Some((ck, cname)), Some(enemy)) = (ctl, lane.enemy()) {
        let c = lane.spawn(ck, &cname, local, CellXY::new(at.x + 4, row + 3));
        let foe = lane.spawn(Kind::Vehicle, &vname, enemy, CellXY::new(at.x + 6, row + 1));
        taken = glasnost_sim::mind::take(lane.world_mut(), c, foe);
        if taken {
            units.push(foe);
        }
    }
    println!("controlled enemy vehicle among them: {taken}");
    lane.look_at(CellXY::new(at.x + 1, at.y + 2));
    lane.ticks(2);
    lane.save("grinder-before");
    let credits = |lane: &Lane| lane.world().houses[usize::from(local.0)].credits;
    let start = credits(lane);
    for &u in &units {
        let cursor = targeting::cursor(
            lane.world(),
            local,
            &[u],
            ClickOn::Object(b),
            Modifiers::default(),
        );
        let n = lane.order(local, u, ClickOn::Object(b));
        println!("click on the grinder: cursor {cursor:?}, {n} orders");
    }
    let mut shot = false;
    let mut left = units.len();
    for t in 0..1500 {
        lane.ticks(1);
        let now = units
            .iter()
            .filter(|&&u| lane.world().obj(u).is_some_and(|o| o.on_map))
            .count();
        if now < left {
            println!(
                "{} consumed after {t} ticks; house paid {}",
                left - now,
                credits(lane) > start
            );
            left = now;
        }
        if t == 20 && !shot {
            lane.save("grinder-bound");
            shot = true;
        }
        if left == 0 {
            break;
        }
    }
    println!(
        "consumed {} of {}; the house was paid {}; grinder still standing {}",
        units.len() - left,
        units.len(),
        credits(lane) > start,
        lane.world().obj(b).is_some_and(|o| o.on_map)
    );
    lane.ticks(10);
    lane.save("grinder-after");
}

fn bunker_type(w: &World) -> Option<(String, (i32, i32))> {
    w.rules()
        .buildings
        .iter()
        .find(|b| b.raw.bunker.unwrap_or(false))
        .map(|b| (b.name.clone(), b.foundation))
}

/// A vehicle type that may go into a tank bunker by its type's flags (S-0606
/// claim 1: bunkerable, turret, a first weapon, not hover), not a harvester.
fn bunker_vehicle(w: &World) -> Option<String> {
    let r = w.rules();
    (0..r.vehicles.len())
        .find(|&i| {
            r.unit(Kind::Vehicle, i as u32).is_some_and(|u| {
                u.raw.bunkerable.unwrap_or(true)
                    && u.turret
                    && u.combat.slots[0].is_some()
                    && u.class != Some(glasnost_rules::sim::LocomotionClass::Hover)
                    && !u.harvester
                    && u.strength > 0
            })
        })
        .map(|i| r.vehicles[i].name.clone())
}

/// S-0606 claims 3 to 7, 9, 11 and 12: a tank bunker of the local house and
/// a vehicle it accepts; the enter click (cursor *enter*) sends it in, the
/// bunker clears, turns it and drives it in along its track, and it is
/// bunkered facing south; an enemy vehicle comes into range and is fired at
/// from inside; the deploy click on the vehicle (cursor *deploy*) lets it
/// out along the release track. Stages with their ticks; five screens.
fn bunker(lane: &mut Lane) {
    use glasnost_sim::structures::bunker::{self, Stage};
    let w = lane.world();
    let ty = bunker_type(w);
    let veh = bunker_vehicle(w);
    let foe_ty = plain_vehicle(w);
    println!(
        "bunker type found: {}, accepted vehicle type found: {}",
        ty.is_some(),
        veh.is_some()
    );
    let (Some((bname, size)), Some(vname)) = (ty, veh) else {
        return;
    };
    let local = lane.local();
    let home = lane.home();
    let near = CellXY::new(home.x + 8, home.y + 8);
    let Some(at) = lane.area(near, size, 5, |l| l != land::WATER && l != land::ROCK) else {
        println!("no room for the bunker");
        return;
    };
    let b = lane.spawn(Kind::Building, &bname, local, at);
    lane.ticks(4);
    let v = lane.spawn(
        Kind::Vehicle,
        &vname,
        local,
        CellXY::new(at.x + size.0 + 3, at.y + 1),
    );
    lane.ticks(2);
    lane.look_at(CellXY::new(at.x + 1, at.y + 1));
    lane.ticks(2);
    lane.save("bunker-before");
    lane.print_slots(b);
    let cursor = targeting::cursor(
        lane.world(),
        local,
        &[v],
        ClickOn::Object(b),
        Modifiers::default(),
    );
    let n = lane.order(local, v, ClickOn::Object(b));
    println!("click on the bunker: cursor {cursor:?}, {n} orders");
    let stage = |lane: &Lane| lane.world().obj(b).map(|o| o.structure.bunker_stage);
    let mut last = stage(lane);
    let mut driving_shot = false;
    let mut inside = false;
    for t in 0..1500 {
        lane.ticks(1);
        let s = stage(lane);
        if s != last {
            println!("bunker stage {s:?} after {t} ticks");
            last = s;
        }
        if s == Some(Stage::DrivingIn) && !driving_shot {
            lane.save("bunker-driving-in");
            lane.print_slots(b);
            driving_shot = true;
        }
        if bunker::bunkered_in(lane.world(), v) == Some(b) {
            println!("bunkered after {t} ticks");
            inside = true;
            break;
        }
    }
    if !inside {
        println!("not bunkered");
        return;
    }
    lane.ticks(10);
    let facing = lane.world().obj(v).map(|o| o.body.target);
    println!("bunkered vehicle body facing {facing:?}");
    // The walls rise as special-1 and special-2 play (S-0606 claim 6, S-0109): wait until
    // their slot animations have held still for 60 ticks (or gone), at most 900 ticks.
    let (mut last, mut still) = (lane.slot_list(b), 0);
    for t in 0..900 {
        lane.ticks(1);
        let now = lane.slot_list(b);
        still = if now == last { still + 1 } else { 0 };
        last = now;
        if still >= 60 {
            println!("wall slots settled after {t} more ticks");
            break;
        }
    }
    lane.save("bunker-occupied");
    lane.print_slots(b);
    if let (Some(fname), Some(enemy)) = (foe_ty, lane.enemy()) {
        let foe = lane.spawn(
            Kind::Vehicle,
            &fname,
            enemy,
            CellXY::new(at.x - 4, at.y + 1),
        );
        let full = lane.full_health(foe);
        let mut shot = false;
        for t in 0..600 {
            lane.ticks(1);
            let hp = lane.world().obj(foe).map_or(0, |o| o.health);
            if hp < full && !shot {
                println!("the enemy vehicle is hit from inside after {t} ticks");
                lane.ticks(4);
                lane.save("bunker-firing");
                lane.print_slots(b);
                shot = true;
            }
            if hp <= 0 {
                break;
            }
        }
        println!("enemy vehicle hit from the bunker: {shot}");
    }
    let cursor = targeting::cursor(
        lane.world(),
        local,
        &[v],
        ClickOn::Object(v),
        Modifiers::default(),
    );
    let n = lane.order(local, v, ClickOn::Object(v));
    println!("click on the bunkered vehicle: cursor {cursor:?}, {n} orders");
    for t in 0..600 {
        lane.ticks(1);
        if !bunker::bunkered(lane.world(), v) {
            println!("let out after {t} ticks; bunker stage {:?}", stage(lane));
            lane.ticks(5);
            lane.save("bunker-walls-down");
            lane.print_slots(b);
            break;
        }
    }
    lane.ticks(60);
    lane.save("bunker-released");
    lane.print_slots(b);
}

/// A lifter type by its flags (S-0607 Terms: a vehicle whose first weapon's
/// warhead is a seizing warhead), with that weapon's damage.
fn lifter_type(w: &World) -> Option<(String, i32)> {
    let r = w.rules();
    (0..r.vehicles.len()).find_map(|i| {
        let u = r.unit(Kind::Vehicle, i as u32)?;
        let wp = u.combat.slots[0]?;
        let weapon = r.combat.weapon(wp);
        let h = weapon.warhead?;
        r.combat
            .warhead(h)
            .is_locomotor
            .then(|| (r.vehicles[i].name.clone(), weapon.damage))
    })
}

/// A ground vehicle type a lifter of weapon damage `damage` may seize
/// (S-0607 claim 3 and F1: size below the damage), not a harvester, not
/// hover, not one that deploys into a building (the enemy would deploy it
/// out of the shot), with no seizing weapon of its own; an unarmed type first, so that
/// the scene's enemies do not shoot the lifter down.
fn liftable_type(w: &World, damage: i32) -> Option<String> {
    let r = w.rules();
    let d = glasnost_num::F64::from_i64(i64::from(damage), r.mode);
    let ok = |i: usize, armed: bool| {
        r.unit(Kind::Vehicle, i as u32).is_some_and(|u| {
            u.combat.slots[0].is_some() == armed
                && u.combat.slots[0].is_none_or(|wp| {
                    r.combat
                        .weapon(wp)
                        .warhead
                        .is_none_or(|h| !r.combat.warhead(h).is_locomotor)
                })
                && !u.harvester
                && !u.raw.deploys_into.as_deref().is_some_and(|d| !d.is_empty())
                && u.strength > 0
                && u.size.lt(d)
                && u.class != Some(glasnost_rules::sim::LocomotionClass::Hover)
        })
    };
    (0..r.vehicles.len())
        .find(|&i| ok(i, false))
        .or_else(|| (0..r.vehicles.len()).find(|&i| ok(i, true)))
        .map(|i| r.vehicles[i].name.clone())
}

/// S-0607 claims 1, 5, 7, 12, 13 and 15 to 18: a lifter of the local house
/// is clicked onto an enemy vehicle (cursor *attack*); the hit seizes it
/// unhurt and pulls it up towards the lifter. When it is over a second
/// enemy vehicle, a move click on the lifter ends the grip: the unit is
/// dropped, falls onto the occupied cell and is destroyed by its impact,
/// hurting the vehicle underneath. Ticks and health in per-mille; four
/// screens.
fn magnetic_lift(lane: &mut Lane) {
    use glasnost_sim::lift;
    let w = lane.world();
    let lt = lifter_type(w);
    let victim_ty = lt.as_ref().and_then(|(_, d)| liftable_type(w, *d));
    let enemy = lane.enemy();
    println!(
        "lifter type found: {}, liftable type found: {}",
        lt.is_some(),
        victim_ty.is_some()
    );
    let (Some((lname, _)), Some(vname), Some(enemy)) = (lt, victim_ty, enemy) else {
        return;
    };
    let local = lane.local();
    let home = lane.home();
    let Some(at) = lane.area(CellXY::new(home.x + 8, home.y + 8), (9, 1), 3, |l| {
        l != land::WATER && l != land::ROCK
    }) else {
        println!("no room for the scene");
        return;
    };
    let l = lane.spawn(Kind::Vehicle, &lname, local, at);
    let under = lane.spawn(Kind::Vehicle, &vname, enemy, CellXY::new(at.x + 3, at.y));
    let v = lane.spawn(Kind::Vehicle, &vname, enemy, CellXY::new(at.x + 6, at.y));
    lane.ticks(2);
    lane.look_at(CellXY::new(at.x + 3, at.y));
    lane.ticks(2);
    lane.save("lift-before");
    let cursor = targeting::cursor(
        lane.world(),
        local,
        &[l],
        ClickOn::Object(v),
        Modifiers::default(),
    );
    let n = lane.order(local, l, ClickOn::Object(v));
    println!("click on the enemy vehicle: cursor {cursor:?}, {n} orders");
    let full = lane.full_health(v);
    let mut seized_at = None;
    let mut over = false;
    for t in 0..900 {
        lane.ticks(1);
        let w = lane.world();
        if seized_at.is_none() && lift::seized(w, v) {
            seized_at = Some(t);
            println!(
                "seized after {t} ticks, health {} per mille",
                lane.permille(v)
            );
        }
        if seized_at.is_some() && w.obj(v).is_none() {
            println!("the seized unit left play before the drop");
            break;
        }
        let pos = w.obj(v).map(|o| o.pos.cell());
        if lift::grips(w, l, v)
            && pos == Some(CellXY::new(at.x + 3, at.y))
            && w.height_above_surface(v) > 200
        {
            println!(
                "over the second vehicle after {t} ticks at height {}",
                w.height_above_surface(v)
            );
            over = true;
            break;
        }
    }
    let _ = full;
    if seized_at.is_none() || !over {
        println!("not lifted over the second vehicle");
        lane.save("lift-seized");
        return;
    }
    lane.save("lift-seized");
    let cursor = targeting::cursor(
        lane.world(),
        local,
        &[l],
        ClickOn::Cell(CellXY::new(at.x - 1, at.y)),
        Modifiers::default(),
    );
    let n = lane.order(local, l, ClickOn::Cell(CellXY::new(at.x - 1, at.y)));
    println!("move click for the lifter: cursor {cursor:?}, {n} orders");
    for t in 1..=10 {
        lane.ticks(1);
        if !lift::grips(lane.world(), l, v) {
            println!(
                "grip ended after {t} ticks, dropped: {}",
                lift::dropped(lane.world(), v)
            );
            break;
        }
    }
    lane.ticks(5);
    lane.save("lift-drop");
    for t in 0..400 {
        lane.ticks(1);
        if lane.world().obj(v).is_none() {
            println!(
                "impact after {t} ticks: the dropped unit left play; the vehicle underneath at {} per mille",
                lane.permille(under)
            );
            break;
        }
    }
    lane.ticks(3);
    lane.save("lift-impact");
}

/// S-0613 claims 2, 4, 5, 8 to 13: a virus unit of the local house is
/// ordered (by the click a person gives) at enemy infantry standing in a
/// row; each kill plays the virus death and its middle event puts clouds
/// into the shared gas system; the clouds drift, sink to the ground and
/// hit what stands in them. Counts clouds, hits and kills; three screens.
fn gas_clouds(lane: &mut Lane) {
    let w = lane.world();
    let virus = virus_type(w);
    let victim = human_infantry(w);
    println!(
        "virus type found: {}, its warhead names a particle system: {}, enemy infantry type found: {}, shared gas system: {}",
        virus.is_some(),
        virus.as_ref().is_some_and(|v| v.1),
        victim.is_some(),
        w.gas.shared.is_some()
    );
    let (Some((vname, _)), Some(iname)) = (virus, victim) else {
        return;
    };
    let local = lane.local();
    let Some(enemy) = lane.enemy() else { return };
    let home = lane.home();
    let spot = lane.open_land(CellXY::new(home.x + 8, home.y + 8));
    let v = lane.spawn(
        Kind::Infantry,
        &vname,
        local,
        CellXY::new(spot.x - 3, spot.y),
    );
    let foes: Vec<ObjectId> = (0..4)
        .map(|i| {
            lane.spawn(
                Kind::Infantry,
                &iname,
                enemy,
                CellXY::new(spot.x + 1, spot.y - 1 + i),
            )
        })
        .collect();
    lane.look_at(spot);
    lane.ticks(4);
    lane.save("gas-before");
    let particles = |w: &World| glasnost_sim::gas::particles(w).count();
    let mut most = 0;
    let mut shot_cloud = false;
    for (n, &f) in foes.iter().enumerate() {
        if lane.world().obj(f).is_none_or(|o| !o.on_map) {
            continue;
        }
        let sent = lane.order(local, v, ClickOn::Object(f));
        println!("click on enemy infantry {n}: {sent} orders");
        for _ in 0..400 {
            lane.ticks(1);
            most = most.max(particles(lane.world()));
            if !shot_cloud && particles(lane.world()) >= 2 {
                lane.ticks(6);
                lane.save("gas-cloud");
                shot_cloud = true;
            }
            if lane.world().obj(f).is_none_or(|o| !o.on_map) {
                println!("enemy infantry {n} gone at tick {}", lane.world().tick);
                break;
            }
        }
    }
    lane.ticks(60);
    most = most.max(particles(lane.world()));
    lane.save("gas-later");
    let w = lane.world();
    let s = lane.c.session.as_ref().unwrap();
    let k = &s.loaded.live.as_ref().expect("a live game").counts;
    println!(
        "gas: particles now {}, most at once {most} (view {}), systems {}, virus animations playing {}, gas hits {}, kills {}; virus unit alive {}",
        particles(w),
        k.most_gas,
        w.gas.systems.len(),
        w.gas.anims.len(),
        k.gas_hits,
        k.kills,
        w.obj(v).is_some_and(|o| o.on_map)
    );
}

/// The first unit type (infantry, then vehicles) whose primary weapon has a
/// takeover warhead and whose unlimited-control flag is `unlimited`.
fn controller_type(w: &World, unlimited: bool) -> Option<(Kind, String)> {
    let r = w.rules();
    (0..r.infantry.len())
        .map(|i| (Kind::Infantry, i as u32))
        .chain((0..r.vehicles.len()).map(|i| (Kind::Vehicle, i as u32)))
        .find(|&(k, i)| {
            r.unit(k, i).is_some_and(|u| {
                u.combat.slots[0].is_some_and(|p| {
                    let wr = r.combat.weapon(p);
                    wr.infinite_mind_control == unlimited
                        && wr.warhead.is_some_and(|h| r.combat.warhead(h).mind_control)
                })
            })
        })
        .map(|(k, i)| (k, r.unit(k, i).map(|u| u.name.clone()).unwrap_or_default()))
}

/// S-0601 claims 1 to 7, 9, 17 and 18: a controller of the local house is
/// ordered (by the click a person gives) at an enemy vehicle; the vehicle
/// changes sides and shows its ring; a second enemy vehicle is put beside
/// it and taken (a capacity-1 controller lets the first go, claim 7); with
/// the controller selected the link line is drawn; the controller is
/// removed and its victim goes back to its owner.
fn mind_control(lane: &mut Lane) {
    let w = lane.world();
    let ctrl = controller_type(w, false);
    let victim = plain_vehicle(w);
    println!(
        "controller type found: {}, enemy vehicle type found: {}",
        ctrl.is_some(),
        victim.is_some()
    );
    let (Some((kind, cname)), Some(vname)) = (ctrl, victim) else {
        return;
    };
    let local = lane.local();
    let Some(enemy) = lane.enemy() else { return };
    let home = lane.home();
    let spot = lane.open_land(CellXY::new(home.x + 8, home.y + 8));
    let c = lane.spawn(kind, &cname, local, CellXY::new(spot.x - 3, spot.y));
    // One enemy vehicle at a time, so the controller lives to take the next.
    let mut vs: Vec<ObjectId> = vec![lane.spawn(
        Kind::Vehicle,
        &vname,
        enemy,
        CellXY::new(spot.x + 1, spot.y),
    )];
    let rec = glasnost_sim::mind::controller(lane.world(), c).map(|r| (r.capacity, r.unlimited));
    println!("controller record (capacity, unlimited): {rec:?}");
    lane.look_at(spot);
    lane.ticks(4);
    lane.save("mind-before");
    let mut taken = Vec::new();
    for n in 0..2 {
        if n == 1 {
            let v = lane.spawn(
                Kind::Vehicle,
                &vname,
                enemy,
                CellXY::new(spot.x + 1, spot.y + 2),
            );
            vs.push(v);
        }
        let v = vs[n];
        let sent = lane.order(local, c, ClickOn::Object(v));
        println!("click on enemy vehicle {n}: {sent} orders");
        for t in 0..600 {
            lane.ticks(1);
            if glasnost_sim::mind::controlled_by(lane.world(), v) == Some(c) {
                println!(
                    "vehicle {n} taken after {t} ticks; owner is the local house {}",
                    lane.world().obj(v).is_some_and(|o| o.owner == Some(local))
                );
                taken.push(v);
                break;
            }
        }
        lane.ticks(3);
        if let Some(at) = lane.world().obj(v).map(|o| o.pos.cell()) {
            lane.look_at(at);
            lane.ticks(1);
        }
        if n == 0 {
            lane.save("mind-taken");
        } else {
            lane.save("mind-second");
        }
    }
    println!(
        "controller alive {} (health per mille {})",
        lane.world().obj(c).is_some(),
        lane.permille(c)
    );
    for (n, &v) in vs.iter().enumerate() {
        let w = lane.world();
        println!(
            "vehicle {n}: exists {}, local {}, controlled by the controller {}",
            w.obj(v).is_some(),
            w.obj(v).is_some_and(|o| o.owner == Some(local)),
            glasnost_sim::mind::controlled_by(w, v) == Some(c)
        );
    }
    let lines = glasnost_sim::mind::link_lines(lane.world()).len();
    let rings = vs
        .iter()
        .filter(|&&v| glasnost_sim::mind::ring(lane.world(), v).is_some())
        .count();
    println!(
        "link lines {lines}; rings {rings}; taken in turn {}",
        taken.len()
    );
    lane.c.session.as_mut().unwrap().play.selection = vec![c];
    if let Some(at) = lane.world().obj(c).map(|o| o.pos.cell()) {
        lane.look_at(at);
    }
    lane.ticks(1);
    lane.save("mind-link-selected");
    glasnost_sim::objects::remove(lane.world_mut(), c);
    lane.ticks(3);
    let back = taken
        .iter()
        .filter(|&&v| lane.world().obj(v).is_some_and(|o| o.owner == Some(enemy)))
        .count();
    println!(
        "controller removed: {back} of {} back with their owner",
        taken.len()
    );
    lane.save("mind-released");
}

/// S-0601 claim 16: an unlimited controller is given (by the takeover
/// itself) more victims than its overload counts allow; its health is
/// followed until it dies or 300 ticks pass.
fn mind_overload(lane: &mut Lane) {
    let w = lane.world();
    let ctrl = controller_type(w, true);
    let victim = plain_vehicle(w);
    println!(
        "unlimited controller type found: {}, enemy vehicle type found: {}",
        ctrl.is_some(),
        victim.is_some()
    );
    let (Some((kind, cname)), Some(vname)) = (ctrl, victim) else {
        return;
    };
    let local = lane.local();
    let Some(enemy) = lane.enemy() else { return };
    let home = lane.home();
    let spot = lane.open_land(CellXY::new(home.x + 8, home.y + 8));
    let c = lane.spawn(kind, &cname, local, CellXY::new(spot.x - 4, spot.y));
    // The smallest victim count whose overload level hurts the controller
    // without killing it with one pulse (S-0601 claim 16, F2), so that the
    // picture shows the pulse's damage, sparks and highlight; else the
    // smallest count that hurts at all.
    let (n, lethal) = {
        let w = lane.world();
        let m = &w.rules().combat.general.mind;
        let strength = lane.strength(c);
        let dmg = |n: usize| {
            glasnost_sim::mind::overload_level(&m.overload_count, n as i64)
                .and_then(|i| m.overload_damage.get(i).copied())
                .unwrap_or(0)
        };
        let hurts = (1..=12).filter(|&n| dmg(n) >= 1);
        hurts
            .clone()
            .find(|&n| dmg(n) < strength)
            .map(|n| (n, false))
            .or_else(|| hurts.clone().next().map(|n| (n, true)))
            .unwrap_or((12, false))
    };
    println!("victims for a pulse: {n}; one pulse kills: {lethal}");
    let mut held = 0;
    for i in 0..n {
        let at = lane.open_land(CellXY::new(
            spot.x + (i % 4) as i32 * 2,
            spot.y + (i / 4) as i32 * 2,
        ));
        let v = lane.spawn(Kind::Vehicle, &vname, enemy, at);
        held += usize::from(glasnost_sim::mind::take(lane.world_mut(), c, v));
    }
    println!("victims taken directly: {held}");
    lane.c.session.as_mut().unwrap().play.selection = vec![c];
    lane.look_at(spot);
    lane.ticks(2);
    let start = lane.permille(c);
    lane.save("mind-overload-before");
    // The first pulse check is the 31st overload update (S-0601 claim 16):
    // wait for the first hit, then a few ticks for its sparks to show.
    let mut low = start;
    let mut pulsed = false;
    for _ in 0..300 {
        lane.ticks(1);
        if lane.world().obj(c).is_none() {
            println!(
                "the controller died of overload at tick {}",
                lane.world().tick
            );
            if !pulsed {
                lane.save("mind-overload");
            }
            break;
        }
        low = low.min(lane.permille(c));
        if !pulsed && low < start {
            pulsed = true;
            println!("first damaging pulse at tick {}", lane.world().tick);
            lane.ticks(3);
            lane.save("mind-overload");
        }
    }
    println!("controller health per mille: start {start}, lowest {low}");
    lane.save("mind-overload-later");
}

// ---------------------------------------------------------------------------
// The lane

fn files(dir: &std::path::Path) -> Vec<PickedFile> {
    let ext = MountOptions::default().archive_extensions;
    let mut out = Vec::new();
    for e in std::fs::read_dir(dir).unwrap().flatten() {
        let Ok(meta) = e.metadata() else { continue };
        if !meta.is_file() {
            continue;
        }
        let name = e.file_name().to_string_lossy().into_owned();
        let x = name
            .rsplit_once('.')
            .map(|(_, x)| x.to_ascii_lowercase())
            .unwrap_or_default();
        let source: Rc<dyn ByteSource> = if ext.contains(&x) {
            match FileSource::open(&e.path()) {
                Ok(f) => Rc::new(f),
                Err(_) => continue,
            }
        } else {
            Rc::new(Unopened(meta.len()))
        };
        out.push(PickedFile { path: name, source });
    }
    out
}

fn env_num(name: &str, default: usize) -> usize {
    std::env::var(name)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn run_jobs(c: &mut Client) {
    while let Some(job) = c.job.take() {
        let done = pollster::block_on(run_job(job));
        c.done(done);
    }
}

fn click(c: &mut Client, r: Rect) {
    use glasnost_app::game::play::Button;
    use glasnost_ui::geom::Point;
    let p = Point::new(r.x + r.w / 2, r.y + r.h / 2);
    let _ = c.frame();
    c.pointer_moved(p);
    c.pointer_down(Button::Left, p);
    c.pointer_up(Button::Left, p, Modifiers::default());
}

struct Lane {
    c: Client,
    clock: i64,
    dir: std::path::PathBuf,
}

impl Lane {
    /// Starts a skirmish from the menu, the map revealed to the local house.
    fn start() -> Option<Lane> {
        Lane::start_on(env_num("GLASNOST_UI_MAP", 0))
    }

    /// The number of maps the install offers.
    fn map_count() -> usize {
        let Some(dir) = std::env::var_os("GLASNOST_GAME_DIR") else {
            return 0;
        };
        let mut c = Client::new();
        c.mount(files(std::path::Path::new(&dir)));
        run_jobs(&mut c);
        c.install.as_ref().map_or(0, |i| i.maps.len())
    }

    /// Starts a skirmish on the first offered map whose world satisfies
    /// `fits`, chosen by content, never by name. Prints the map's index.
    fn start_where(fits: impl Fn(&World) -> bool) -> Option<Lane> {
        for i in 0..Lane::map_count() {
            let lane = Lane::start_on(i)?;
            if fits(lane.world()) {
                println!("map index {i} fits");
                return Some(lane);
            }
        }
        None
    }

    /// Starts a skirmish on the `map`-th offered map.
    fn start_on(map: usize) -> Option<Lane> {
        let dir = std::env::var_os("GLASNOST_GAME_DIR")?;
        let out = png::lane_out("m7a");
        std::fs::create_dir_all(&out).unwrap();
        let mut c = Client::new();
        c.resize((1280, 800));
        c.mount(files(std::path::Path::new(&dir)));
        run_jobs(&mut c);
        let maps = c.install.as_ref().expect("the install reads").maps.len();
        let map = map.min(maps - 1);
        let _ = c.frame();
        let r = c.menu_targets.rect_of(&MenuHit::Skirmish).unwrap();
        click(&mut c, r);
        for _ in 0..map {
            let _ = c.frame();
            let r = c.setup_targets.rect_of(&SetupHit::Map(1)).unwrap();
            click(&mut c, r);
        }
        let _ = c.frame();
        let r = c.setup_targets.rect_of(&SetupHit::Start_).unwrap();
        c.next_seed = 12345;
        click(&mut c, r);
        run_jobs(&mut c);
        assert_eq!(
            c.shell.screen,
            Screen::Game,
            "the game starts: {}",
            c.status
        );
        let mut lane = Lane {
            c,
            clock: 0,
            dir: out,
        };
        let s = lane.c.session.as_mut().unwrap();
        let local = s.play.me();
        glasnost_sim::econ::shroud::defeat_reveal(&mut s.play.game.world, local);
        lane.ticks(5);
        Some(lane)
    }

    fn ticks(&mut self, n: usize) {
        for _ in 0..n {
            self.clock += 1000;
            self.c.update(self.clock);
            if self.c.shell.screen != Screen::Game {
                break;
            }
        }
    }

    fn world(&self) -> &World {
        self.c.session.as_ref().unwrap().play.world()
    }

    fn world_mut(&mut self) -> &mut World {
        &mut self.c.session.as_mut().unwrap().play.game.world
    }

    fn local(&self) -> HouseId {
        self.c.session.as_ref().unwrap().play.me()
    }

    /// The cell of the local house's first object on the map (its start).
    fn home(&self) -> CellXY {
        let w = self.world();
        let local = self.local();
        w.objects
            .values()
            .find(|o| o.on_map && o.owner == Some(local))
            .map(|o| o.pos.cell())
            .expect("a local object")
    }

    /// Centres the view on `cell`.
    fn look_at(&mut self, cell: CellXY) {
        let s = self.c.session.as_mut().unwrap();
        let (x, y) = glasnost_render::projection::cell_centre(cell.x, cell.y);
        s.set_centre(glasnost_render::projection::project_lifted(x, y, 0));
    }

    /// The live view's animation counts: created, with art, without a
    /// sprite, drawn in the last frame.
    fn counts(&self) -> (u64, u64, u64, usize) {
        let s = self.c.session.as_ref().unwrap();
        let k = &s.loaded.live.as_ref().expect("a live game").counts;
        (
            k.anims,
            k.anims_with_art,
            k.anims_without_sprite,
            k.anims_drawn,
        )
    }

    /// A building's running slot animations (S-0108) with their frames.
    fn slot_list(&self, id: ObjectId) -> Vec<String> {
        let s = self.c.session.as_ref().unwrap();
        s.loaded.live.as_ref().expect("a live game").slot_names(id)
    }

    /// A building's running slot animations (S-0108), printed.
    fn print_slots(&self, id: ObjectId) {
        println!("  slot animations: {:?}", self.slot_list(id));
    }

    fn save(&mut self, name: &str) {
        let rgba = self.c.frame_rgba();
        let (w, h) = self.c.size;
        std::fs::write(
            self.dir.join(format!("{name}.png")),
            png::png(w as u32, h as u32, &rgba),
        )
        .unwrap();
        let (made, art, none, drawn) = self.counts();
        println!(
            "wrote {name}.png (tick {}): animations created {made}, \
             with art {art}, without sprite {none}, drawn now {drawn}",
            self.world().tick
        );
        // GLASNOST_TRACE_ANIMS=1: the animations playing in this shot, and
        // the armed objects on screen, by position and screen point.
        if std::env::var_os("GLASNOST_TRACE_ANIMS").is_some() {
            let s = self.c.session.as_ref().unwrap();
            let live = s.loaded.live.as_ref().expect("a live game");
            for (i, (name, (x, y, z))) in live.playing_anims().into_iter().enumerate() {
                let p = self.screen_of(x, y, z);
                println!("  animation {i} {name} at position ({x}, {y}, {z}), screen {p:?}");
            }
            let w = self.world();
            let t = w.tick;
            for (id, o) in w.objects.iter() {
                let p = self.screen_of(o.pos.x, o.pos.y, o.pos.z);
                let m = s.map_rect();
                if !o.on_map || p.0 < m.x || p.1 < m.y || p.0 > m.x + m.w || p.1 > m.y + m.h {
                    continue;
                }
                if glasnost_sim::combat::veterancy::type_combat(w, *id).is_none() {
                    continue;
                }
                let fp0 = glasnost_sim::combat::position::fire_point(w, *id, 0, 0);
                let fp1 = glasnost_sim::combat::position::fire_point(w, *id, 0, 1);
                println!(
                    "  object {id:?} {:?} at {:?} screen {p:?}, body {} turret {}, \
                     fire point (slot 0, k 0) {:?} screen {:?}, (k 1) {:?} screen {:?}",
                    o.kind,
                    (o.pos.x, o.pos.y, o.pos.z),
                    o.body.value(t),
                    o.turret.value(t),
                    (fp0.x, fp0.y, fp0.z),
                    self.screen_of(fp0.x, fp0.y, fp0.z),
                    (fp1.x, fp1.y, fp1.z),
                    self.screen_of(fp1.x, fp1.y, fp1.z),
                );
            }
        }
        // GLASNOST_TRACE_BOX="x0,y0,x1,y1,step": the distinct items over a grid of screen points.
        if let Ok(b) = std::env::var("GLASNOST_TRACE_BOX") {
            let n: Vec<i32> = b.split(',').filter_map(|v| v.trim().parse().ok()).collect();
            if let [x0, y0, x1, y1, step] = n[..] {
                let mut seen = std::collections::BTreeSet::new();
                for x in (x0..x1).step_by(step.max(1) as usize) {
                    for y in (y0..y1).step_by(step.max(1) as usize) {
                        let s = self.c.session.as_mut().unwrap();
                        seen.extend(s.trace_at(glasnost_ui::geom::Point::new(x, y)));
                    }
                }
                println!("trace box {name}: {} distinct items", seen.len());
                for l in seen {
                    println!("  {l}");
                }
            }
        }
    }

    /// The screen point of the world position (x, y, z): S-0101's
    /// projection with the height lift, less the view origin.
    fn screen_of(&self, x: i32, y: i32, z: i32) -> (i32, i32) {
        let s = self.c.session.as_ref().unwrap();
        let (wx, wy) = glasnost_render::projection::project_lifted(x, y, z);
        let v = s.view();
        let m = s.map_rect();
        (wx - v.origin.0 + m.x, wy - v.origin.1 + m.y)
    }

    fn spawn(&mut self, kind: Kind, name: &str, house: HouseId, at: CellXY) -> ObjectId {
        glasnost_sim::testkit::spawn(self.world_mut(), kind, name, house.0, at, 0)
    }

    /// Orders `actor` of `house` by a click on `target`, as a person would.
    fn order(&mut self, house: HouseId, actor: ObjectId, target: ClickOn) -> usize {
        let w = self.world();
        let r = targeting::click(w, house, &[actor], target, Modifiers::default());
        let mut n = 0;
        for o in r.orders {
            n += usize::from(glasnost_sim::orders::issue(self.world_mut(), house, o));
        }
        n
    }

    /// A player house that is not allied with the local house.
    fn enemy(&self) -> Option<HouseId> {
        let w = self.world();
        let local = self.local();
        (0..w.houses.len()).map(|i| HouseId(i as u8)).find(|&h| {
            h != local
                && !w.allied(Some(local), Some(h))
                && w.houses[usize::from(h.0)].role == glasnost_sim::world::HouseRole::Player
        })
    }

    /// The strength of `id`'s type.
    fn strength(&self, id: ObjectId) -> i32 {
        let w = self.world();
        w.obj(id)
            .and_then(|o| w.rules().unit(o.kind, o.type_index))
            .map_or(1, |u| u.strength.max(1))
    }

    /// The strength of `id`'s type, buildings included.
    fn full_health(&self, id: ObjectId) -> i32 {
        let w = self.world();
        match w.obj(id) {
            Some(o) if o.kind == Kind::Building => {
                w.rules().buildings[o.type_index as usize].strength.max(1)
            }
            _ => self.strength(id),
        }
    }

    /// `id`'s health in per-mille of its type's strength.
    fn permille(&self, id: ObjectId) -> i32 {
        let h = self.world().obj(id).map_or(0, |o| o.health);
        (i64::from(h) * 1000 / i64::from(self.strength(id))) as i32
    }

    /// The nearest cell to `near` whose 5 × 5 neighbourhood is legal land
    /// that is not water, so that units can stand there.
    fn open_land(&self, near: CellXY) -> CellXY {
        self.area(near, (1, 1), 2, |l| l != land::WATER && l != land::ROCK)
            .unwrap_or(near)
    }

    /// The nearest top-left cell to `near` of a `size` area that, with a
    /// `margin` of cells around it, is usable, of land types `ok` and free of
    /// ground objects.
    fn area(
        &self,
        near: CellXY,
        size: (i32, i32),
        margin: i32,
        ok: impl Fn(u8) -> bool,
    ) -> Option<CellXY> {
        let w = self.world();
        let good = |c: CellXY| {
            (-margin..size.0 + margin).all(|dx| {
                (-margin..size.1 + margin).all(|dy| {
                    let k = CellXY::new(c.x + dx, c.y + dy);
                    w.cells.usable(k)
                        && w.cells.cell(k).is_some_and(|x| ok(x.land))
                        && w.cells.list(k, Layer::Ground).is_empty()
                })
            })
        };
        w.cells
            .grid
            .iter()
            .map(|(k, _)| k)
            .filter(|&k| good(k))
            .min_by_key(|k| {
                let (dx, dy) = (i64::from(k.x - near.x), i64::from(k.y - near.y));
                (dx * dx + dy * dy, k.y, k.x)
            })
    }
}
