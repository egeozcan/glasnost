//! Local lane for the M6c features (never in CI): with `GLASNOST_GAME_DIR`
//! set, every scene of [`SCENES`] starts its own skirmish through the
//! windowless client, puts the objects it needs on the map by their rules
//! flags (never by type name), plays the scene and writes its screens to
//! `local-lane/m6c/` (git-ignored). Prints counts, cells and ticks only,
//! never type names or rules values.
//!
//! Scenes now: infantry garrisoning a building and put out again (S-0508),
//! an engineer capturing an enemy building (S-0509) and a spy infiltrating
//! one (S-0510). A new scene is one more function and one more row of
//! [`SCENES`].
//!
//! Run with `cargo test --release -p glasnost-app --test m6c_lane --
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
use glasnost_sim::world::{HouseId, Layer, ObjectId, World};
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
    ("garrison", garrison_and_evict),
    ("capture", engineer_captures),
    ("infiltrate", spy_infiltrates),
    ("tech", engineer_takes_tech_building),
    ("jumpjet", jumpjet_flies),
    ("jumpjet-factory", jumpjet_leaves_its_factory),
    ("low-release-ahead", aircraft_low_release_ahead),
    ("low-release-abeam", aircraft_low_release_abeam),
    ("deploy", unit_deploys),
    ("radiation", radiation_field),
    ("chrono", chrono_erases),
    ("prism", prism_supports),
    ("terror", terror_attacks),
    ("crates", crate_pickup),
];

#[test]
#[ignore = "local lane: needs GLASNOST_GAME_DIR"]
fn m6c_scenes() {
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

/// S-0508: three infantry of an occupier type are sent into a garrisonable
/// building with the click a person gives (S-0214 row I8); the building is
/// then drawn with its occupied frame (S-0108 claim 2) and finally emptied
/// by a deploy order (S-0508 claim 13).
fn garrison_and_evict(lane: &mut Lane) {
    let w = lane.world();
    let r = w.rules();
    let building = (0..r.buildings.len())
        .filter(|&b| {
            r.buildings[b].building.can_be_occupied && r.buildings[b].building.max_occupants >= 2
        })
        .min_by_key(|&b| (r.buildings[b].foundation.0 * r.buildings[b].foundation.1, b))
        .map(|b| (r.buildings[b].name.clone(), r.buildings[b].foundation));
    let man = (0..r.infantry.len())
        .find(|&i| r.infantry[i].raw.occupier.unwrap_or(false))
        .map(|i| r.infantry[i].name.clone());
    println!(
        "garrison type found: {}, occupier found: {}",
        building.is_some(),
        man.is_some()
    );
    let (Some((bname, size)), Some(man)) = (building, man) else {
        return;
    };
    let local = lane.local();
    let neutral = (0..lane.world().houses.len())
        .map(|i| HouseId(i as u8))
        .find(|&h| {
            lane.world().houses[usize::from(h.0)].role == glasnost_sim::world::HouseRole::Neutral
        })
        .unwrap_or(local);
    let home = lane.home();
    let Some(site) = lane.area(CellXY::new(home.x + 6, home.y + 6), size, 2, |l| {
        l != land::WATER && l != land::ROCK
    }) else {
        println!("no open ground");
        return;
    };
    // A garrisonable building the map placed, as a person meets one; else one of ours.
    let placed = lane
        .world()
        .objects
        .values()
        .filter(|o| {
            o.kind == Kind::Building
                && o.on_map
                && glasnost_sim::structures::garrison::garrisonable(lane.world(), o.id)
                && glasnost_sim::structures::garrison::capacity(lane.world(), o.id) >= 2
        })
        .map(|o| o.id)
        .min_by_key(|&id| {
            let c = lane.world().obj(id).map_or(home, |o| o.pos.cell());
            ((c.x - home.x).pow(2) + (c.y - home.y).pow(2), id)
        });
    println!("a map-placed garrison building found: {}", placed.is_some());
    let b = placed.unwrap_or_else(|| lane.spawn(Kind::Building, &bname, neutral, site));
    let site = lane.world().obj(b).map_or(site, |o| o.pos.cell());
    println!("building at {site:?}, home {home:?}, size {size:?}");
    lane.look_at(site);
    lane.ticks(3);
    lane.save("garrison-empty");
    let mut men = Vec::new();
    for i in 0..3 {
        let at = lane.open_land(CellXY::new(site.x - 3, site.y + i * 2));
        men.push(lane.spawn(Kind::Infantry, &man, local, CellXY::new(at.x, at.y + i)));
    }
    let sent: usize = men
        .iter()
        .map(|&m| lane.order(local, m, ClickOn::Object(b)))
        .sum();
    println!("{} infantry sent, {sent} orders", men.len());
    let capacity = glasnost_sim::structures::garrison::capacity(lane.world(), b).max(0) as usize;
    for t in 0..900 {
        lane.ticks(1);
        let inside = glasnost_sim::structures::garrison::count(lane.world(), b);
        if t % 60 == 0 {
            println!("tick {}: {inside} inside", lane.world().tick);
        }
        if inside as usize == men.len().min(capacity) {
            break;
        }
    }
    lane.ticks(3);
    let owner = lane.world().obj(b).map(|o| o.owner);
    println!(
        "{} inside, owner is the local house: {}",
        glasnost_sim::structures::garrison::count(lane.world(), b),
        owner == Some(Some(local))
    );
    lane.save("garrison-occupied");
    glasnost_sim::orders::issue(
        lane.world_mut(),
        local,
        glasnost_sim::orders::Order::Deploy { actor: b },
    );
    for _ in 0..200 {
        lane.ticks(1);
        if glasnost_sim::structures::garrison::count(lane.world(), b) == 0 {
            break;
        }
    }
    lane.ticks(8);
    let on_map = men
        .iter()
        .filter(|&&m| lane.world().obj(m).is_some_and(|o| o.on_map))
        .count();
    println!(
        "after the deploy order: {} inside, {on_map} on the map",
        glasnost_sim::structures::garrison::count(lane.world(), b),
    );
    lane.save("garrison-evicted");
}

/// S-0509: an engineer of the local house is sent into a capturable building
/// of an enemy house with the click a person gives (S-0214 row I7): it walks
/// there, captures it and is used up; the building is drawn in the local
/// house's colour afterwards.
fn engineer_captures(lane: &mut Lane) {
    let w = lane.world();
    let r = w.rules();
    let man = (0..r.infantry.len())
        .find(|&i| r.infantry[i].engineer)
        .map(|i| r.infantry[i].name.clone());
    println!("engineer type found: {}", man.is_some());
    let Some(man) = man else { return };
    let local = lane.local();
    let Some(enemy) = lane.enemy() else {
        println!("no enemy house");
        return;
    };
    let target = lane
        .world()
        .objects
        .values()
        .filter(|o| {
            o.kind == Kind::Building
                && o.on_map
                && o.owner == Some(enemy)
                && lane.world().rules().buildings[o.type_index as usize]
                    .raw
                    .capturable
                    .unwrap_or(false)
        })
        .map(|o| o.id)
        .min_by_key(|&id| id);
    println!("a capturable enemy building found: {}", target.is_some());
    // None at the start of a skirmish: the enemy gets one near the local base.
    let target = target.or_else(|| {
        let w = lane.world();
        let r = w.rules();
        let (name, size) = (0..r.buildings.len())
            .filter(|&b| {
                r.buildings[b].raw.capturable.unwrap_or(false) && r.buildings[b].foundation.0 > 0
            })
            .min_by_key(|&b| (r.buildings[b].foundation.0 * r.buildings[b].foundation.1, b))
            .map(|b| (r.buildings[b].name.clone(), r.buildings[b].foundation))?;
        let home = lane.home();
        let at = lane.area(CellXY::new(home.x + 8, home.y + 8), size, 2, |l| {
            l != land::WATER && l != land::ROCK
        })?;
        Some(lane.spawn(Kind::Building, &name, enemy, at))
    });
    let Some(b) = target else {
        println!("no capturable building type");
        return;
    };
    let site = lane.world().obj(b).map(|o| o.pos.cell()).unwrap();
    let start = lane.open_land(CellXY::new(site.x - 6, site.y));
    let e = lane.spawn(Kind::Infantry, &man, local, start);
    lane.look_at(site);
    lane.ticks(3);
    lane.save("capture-before");
    let sent = lane.order(local, e, ClickOn::Object(b));
    println!("engineer at {start:?} sent at {site:?}, {sent} orders");
    for t in 0..1500 {
        lane.ticks(1);
        if lane.world().obj(e).is_none() {
            println!("engineer used up at tick {}", lane.world().tick);
            break;
        }
        if t % 100 == 0 {
            let p = lane.world().obj(e).map(|o| o.pos.cell());
            println!("tick {}: engineer at {p:?}", lane.world().tick);
        }
    }
    lane.ticks(3);
    let owner = lane.world().obj(b).and_then(|o| o.owner);
    println!(
        "building owner is the local house: {}; the enemy's lost-to-capture mark: {}",
        owner == Some(local),
        glasnost_sim::structures::engineer::lost_to_capture(lane.world(), enemy)
    );
    lane.save("capture-after");
}

/// S-0510: a spy of the local house is sent into an enemy building whose
/// type has an infiltration effect (found by what `structures::spy::rows`
/// says, never by name) with the click a person gives (S-0214 row I24); it
/// walks there, is used up, and the effect shows (a blackout, a theft, a
/// flag). Prints the effect rows by their own names.
fn spy_infiltrates(lane: &mut Lane) {
    let w = lane.world();
    let r = w.rules();
    let man = (0..r.infantry.len())
        .find(|&i| r.infantry[i].raw.agent.unwrap_or(false) && !r.infantry[i].engineer)
        .map(|i| r.infantry[i].name.clone());
    println!("spy type found: {}", man.is_some());
    let Some(man) = man else { return };
    let local = lane.local();
    let Some(enemy) = lane.enemy() else {
        println!("no enemy house");
        return;
    };
    let before_credits = (
        lane.world().houses[usize::from(local.0)].credits,
        lane.world().houses[usize::from(enemy.0)].credits,
    );
    let target = lane
        .world()
        .objects
        .values()
        .filter(|o| o.kind == Kind::Building && o.on_map && o.owner == Some(enemy))
        .map(|o| o.id)
        .filter(|&b| !glasnost_sim::structures::spy::rows(lane.world(), b).is_empty())
        .min_by_key(|&b| {
            let rows = glasnost_sim::structures::spy::rows(lane.world(), b);
            // Money, then power, then anything else: what shows on a screenshot.
            let first = rows.first().map_or(9, |r| match r {
                glasnost_sim::combat::effects::InfiltrationRow::Power => 0,
                glasnost_sim::combat::effects::InfiltrationRow::Cash => 1,
                _ => 2,
            });
            (first, b)
        });
    println!(
        "an enemy building with an infiltration effect found: {}",
        target.is_some()
    );
    // None at the start of a skirmish: the enemy gets a spyable power plant near the local base.
    let target = target.or_else(|| {
        let w = lane.world();
        let r = w.rules();
        let i = (0..r.buildings.len())
            .filter(|&i| {
                r.buildings[i].raw.spyable.unwrap_or(false)
                    && r.econ.power.buildings.get(i).is_some_and(|p| p.output > 0)
                    && r.buildings[i].foundation.0 > 0
            })
            .min_by_key(|&i| (r.buildings[i].foundation.0 * r.buildings[i].foundation.1, i))?;
        let (name, size) = (r.buildings[i].name.clone(), r.buildings[i].foundation);
        let home = lane.home();
        let at = lane.area(CellXY::new(home.x + 8, home.y + 8), size, 2, |l| {
            l != land::WATER && l != land::ROCK
        })?;
        Some(lane.spawn(Kind::Building, &name, enemy, at))
    });
    let Some(b) = target else {
        println!("no spyable power building type");
        return;
    };
    let rows = glasnost_sim::structures::spy::rows(lane.world(), b);
    println!("effect rows: {rows:?}");
    let site = lane.world().obj(b).map(|o| o.pos.cell()).unwrap();
    let start = lane.open_land(CellXY::new(site.x - 6, site.y));
    let s = lane.spawn(Kind::Infantry, &man, local, start);
    lane.look_at(site);
    lane.ticks(3);
    lane.save("infiltrate-before");
    let sent = lane.order(local, s, ClickOn::Object(b));
    println!("spy at {start:?} sent at {site:?}, {sent} orders");
    for t in 0..2500 {
        lane.ticks(1);
        if lane.world().obj(s).is_none() {
            println!("spy used up at tick {}", lane.world().tick);
            break;
        }
        if t % 200 == 0 {
            let p = lane.world().obj(s).map(|o| o.pos.cell());
            println!("tick {}: spy at {p:?}", lane.world().tick);
        }
    }
    lane.ticks(3);
    let w = lane.world();
    println!(
        "credits (local, enemy) before {before_credits:?}, after ({}, {}); enemy blackout running: {}; \
         local factory flags (vehicle, infantry): ({}, {})",
        w.houses[usize::from(local.0)].credits,
        w.houses[usize::from(enemy.0)].credits,
        glasnost_sim::econ::power::blackout_running(w, enemy),
        w.houses[usize::from(local.0)].infiltrated_vehicle_factory,
        w.houses[usize::from(local.0)].infiltrated_infantry_factory,
    );
    lane.save("infiltrate-after");
}

/// S-0515: an engineer of the local house is sent into a cash-producing tech
/// building of the neutral house (found by its rules fields, never by name:
/// the needs-engineer flag with an income amount and delay) with the click a
/// person gives; the capture pays the acquisition bonus, and the income
/// follows at the building's cycles. Prints credits and ticks only.
fn engineer_takes_tech_building(lane: &mut Lane) {
    let w = lane.world();
    let r = w.rules();
    let man = (0..r.infantry.len())
        .find(|&i| r.infantry[i].engineer)
        .map(|i| r.infantry[i].name.clone());
    let tech = (0..r.buildings.len())
        .filter(|&b| {
            let x = &r.buildings[b].raw;
            x.needs_engineer.unwrap_or(false)
                && x.capturable.unwrap_or(false)
                && x.produce_cash_amount.unwrap_or(0) > 0
                && x.produce_cash_delay.unwrap_or(0) > 0
                && r.buildings[b].foundation.0 > 0
        })
        .min_by_key(|&b| (r.buildings[b].foundation.0 * r.buildings[b].foundation.1, b))
        .map(|b| {
            let x = &r.buildings[b].raw;
            (
                r.buildings[b].name.clone(),
                r.buildings[b].foundation,
                (
                    x.produce_cash_startup.unwrap_or(0),
                    x.produce_cash_amount.unwrap_or(0),
                    x.produce_cash_delay.unwrap_or(0),
                ),
            )
        });
    println!(
        "engineer type found: {}, cash-producing tech type found: {}",
        man.is_some(),
        tech.is_some()
    );
    let (Some(man), Some((name, size, (bonus, amount, delay)))) = (man, tech) else {
        return;
    };
    println!("bonus {bonus}, amount {amount}, delay {delay}");
    let local = lane.local();
    let neutral = (0..lane.world().houses.len())
        .map(|i| HouseId(i as u8))
        .find(|&h| {
            lane.world().houses[usize::from(h.0)].role == glasnost_sim::world::HouseRole::Neutral
        });
    let Some(neutral) = neutral else {
        println!("no neutral house");
        return;
    };
    let home = lane.home();
    let Some(at) = lane.area(CellXY::new(home.x + 8, home.y + 8), size, 2, |l| {
        l != land::WATER && l != land::ROCK
    }) else {
        println!("no open ground");
        return;
    };
    let b = lane.spawn(Kind::Building, &name, neutral, at);
    let start = lane.open_land(CellXY::new(at.x - 6, at.y));
    let e = lane.spawn(Kind::Infantry, &man, local, start);
    lane.look_at(at);
    lane.ticks(3);
    let credits = |l: &Lane| l.world().houses[usize::from(l.local().0)].credits;
    println!(
        "credits before {}, tech building manned: {}",
        credits(lane),
        glasnost_sim::econ::power::manned(lane.world(), b)
    );
    lane.save("tech-before");
    let sent = lane.order(local, e, ClickOn::Object(b));
    println!("engineer at {start:?} sent at {at:?}, {sent} orders");
    for _ in 0..1500 {
        lane.ticks(1);
        if lane.world().obj(e).is_none() {
            break;
        }
    }
    let captured = lane.world().tick;
    lane.ticks(1);
    println!(
        "captured at tick {captured}: owner is the local house {}, manned {}, credits {}",
        lane.world().obj(b).and_then(|o| o.owner) == Some(local),
        glasnost_sim::econ::power::manned(lane.world(), b),
        credits(lane)
    );
    lane.save("tech-captured");
    let mut paid = Vec::new();
    let mut last = credits(lane);
    // Three income cycles after the capture.
    for _ in 0..(delay.max(2) * 3 + 5) {
        lane.ticks(1);
        let now = credits(lane);
        if now > last {
            paid.push((lane.world().tick, now - last));
        }
        last = now;
    }
    println!("income payments (tick, credits): {paid:?}");
    lane.save("tech-income");
}

/// S-0502: a unit of a jumpjet type (found by its locomotor, never by name)
/// is ordered by a click to a cell its walkers cannot reach from where it
/// stands (another reach region of the ground: across water or a cliff,
/// found from the map) and flies there: it takes off, cruises and lands.
/// Prints the states and heights by tick only.
fn jumpjet_flies(lane: &mut Lane) {
    use glasnost_sim::jump::Phase;
    let w = lane.world();
    let r = w.rules();
    let found = [Kind::Infantry, Kind::Vehicle].into_iter().find_map(|k| {
        let n = if k == Kind::Infantry {
            r.infantry.len()
        } else {
            r.vehicles.len()
        };
        (0..n).find_map(|i| {
            r.unit(k, i as u32)
                .filter(|u| {
                    u.jumpjet_locomotor
                        && !u.raw.balloon_hover.unwrap_or(false)
                        && !u.simple_deployer
                })
                .map(|u| (k, u.name.clone()))
        })
    });
    println!("jumpjet type found: {}", found.is_some());
    let Some((kind, name)) = found else { return };
    let local = lane.local();
    let home = lane.home();
    let start = lane.open_land(CellXY::new(home.x + 3, home.y + 3));
    let unit = lane.spawn(kind, &name, local, start);
    lane.look_at(start);
    lane.ticks(4);
    // A cell 8 to 16 cells away that the ground does not reach from the start, else any open one.
    let zone = glasnost_rules::sim::MovementZone::Normal;
    let here = glasnost_sim::regions::reach_in(lane.world(), start, zone);
    let across = lane
        .world()
        .cells
        .grid
        .iter()
        .map(|(c, _)| c)
        .filter(|&c| {
            let d2 = (c.x - start.x).pow(2) + (c.y - start.y).pow(2);
            (64..=256).contains(&d2)
                && lane.world().cells.usable(c)
                && glasnost_sim::regions::reach_in(lane.world(), c, zone) != here
                && lane.world().cells.cell(c).is_some_and(|x| {
                    x.land != land::WATER && x.land != land::ROCK && x.land != land::BEACH
                })
                && lane.world().cells.list(c, Layer::Ground).is_empty()
                // Open, level land two cells around it, so that the landing is not hidden at a
                // cliff foot or behind trees (review 2026-10-19).
                && (-2..=2).all(|dx| {
                    (-2..=2).all(|dy| {
                        let k = CellXY::new(c.x + dx, c.y + dy);
                        let (Some(a), Some(b)) =
                            (lane.world().cells.cell(k), lane.world().cells.cell(c))
                        else {
                            return false;
                        };
                        a.level == b.level
                            && a.slope == 0
                            && a.land != land::WATER
                            && a.land != land::ROCK
                            && lane.world().cells.list(k, Layer::Ground).is_empty()
                    })
                })
        })
        .min_by_key(|c| ((c.x - start.x).pow(2) + (c.y - start.y).pow(2), c.y, c.x));
    println!("a cell across an obstacle found: {}", across.is_some());
    let goal = across.unwrap_or_else(|| lane.open_land(CellXY::new(start.x + 12, start.y)));
    lane.save("jumpjet-before");
    let sent = lane.order(local, unit, ClickOn::Cell(goal));
    println!("jumpjet at {start:?} sent to {goal:?}, {sent} orders");
    let mut phase = None;
    let mut shot = false;
    let mut cruise_ticks = 0;
    let mut changes = 0;
    for t in 0..2000 {
        lane.ticks(1);
        let Some(o) = lane.world().obj(unit) else {
            println!("the jumpjet is gone");
            break;
        };
        let p = o.jump.as_ref().map(|j| j.phase);
        if p != phase && changes < 12 {
            changes += 1;
            println!(
                "tick {}: {:?}, height {}, cell {:?}",
                lane.world().tick,
                p,
                lane.world().height_above_surface(unit),
                o.pos.cell()
            );
            phase = p;
        }
        if p == Some(Phase::Cruising) {
            cruise_ticks += 1;
            if cruise_ticks == 30 && !shot {
                let c = o.pos.cell();
                lane.look_at(c);
                lane.ticks(1);
                lane.save("jumpjet-flight");
                shot = true;
            }
        }
        if p == Some(Phase::Grounded) && t > 20 {
            break;
        }
    }
    let end = lane.world().obj(unit).map(|o| o.pos.cell());
    if let Some(o) = lane.world().obj(unit) {
        let u = lane.world().rules().unit(o.kind, o.type_index);
        println!(
            "end state {:?}; has a target: {}; balloon-hover: {}; behaviour {:?}",
            o.jump.as_ref().map(|j| (j.phase, j.moving)),
            o.target.is_some(),
            u.is_some_and(|u| u.raw.balloon_hover.unwrap_or(false)),
            o.beh.current
        );
    }
    println!("landed at {end:?}, goal {goal:?}");
    if let Some(c) = end {
        lane.look_at(c);
    }
    lane.ticks(3);
    lane.save("jumpjet-landed");
}

/// S-0417 claim 15: a jumpjet unit (infantry from a barracks with a door
/// kind, else a vehicle from a war factory) released from its factory rises;
/// the factory contact and the tether end in the ascending update whose
/// height first lies inside the window. Screens: at the release, in the
/// climb, and 20 ticks after the report.
fn jumpjet_leaves_its_factory(lane: &mut Lane) {
    use glasnost_sim::econ::release;
    use glasnost_sim::rules::FactoryKind;
    let w = lane.world();
    let r = w.rules();
    // A jumpjet unit (infantry first) and the smallest factory of its kind; a barracks needs a door kind.
    let jumper = [Kind::Infantry, Kind::Vehicle].into_iter().find_map(|k| {
        let n = if k == Kind::Infantry {
            r.infantry.len()
        } else {
            r.vehicles.len()
        };
        (0..n as u32)
            .find(|&i| {
                r.unit(k, i).is_some_and(|u| {
                    u.jumpjet_locomotor
                        && !u.raw.balloon_hover.unwrap_or(false)
                        && !u.simple_deployer
                })
            })
            .map(|i| (k, i))
    });
    let factory = jumper.and_then(|(k, _)| {
        let want = if k == Kind::Infantry {
            FactoryKind::Infantry
        } else {
            FactoryKind::Vehicle
        };
        (0..r.buildings.len())
            .filter(|&b| {
                let x = &r.buildings[b].raw.econ;
                r.buildings[b].factory == Some(want)
                    && (want == FactoryKind::Vehicle
                        || x.door_first == Some(true)
                        || x.door_second == Some(true)
                        || x.door_third == Some(true))
            })
            .min_by_key(|&b| (r.buildings[b].foundation.0 * r.buildings[b].foundation.1, b))
            .map(|b| (r.buildings[b].name.clone(), r.buildings[b].foundation))
    });
    println!(
        "jumpjet unit found: {:?}, its factory found: {}",
        jumper.map(|j| j.0),
        factory.is_some()
    );
    let (Some((bname, (fw, fh))), Some((jk, ji))) = (factory, jumper) else {
        return;
    };
    let local = lane.local();
    let home = lane.home();
    let Some(tl) = lane.area(CellXY::new(home.x + 4, home.y + 4), (fw, fh), 2, |l| {
        l != land::WATER && l != land::ROCK
    }) else {
        println!("no room for the barracks");
        return;
    };
    let k = lane.spawn(Kind::Building, &bname, local, tl);
    lane.ticks(2);
    let u = glasnost_sim::objects::create(lane.world_mut(), jk, ji, Some(local));
    let out = release::release(lane.world_mut(), k, u);
    println!("released: {out:?}");
    let c = lane.world().obj(k).map_or(tl, |o| o.pos.cell());
    lane.look_at(c);
    lane.ticks(1);
    lane.save("jumpjet-factory-release");
    let mut climbing_shot = false;
    let mut reported = None;
    for _ in 0..600 {
        lane.ticks(1);
        let w = lane.world();
        let Some(o) = w.obj(u) else { break };
        let h = w.height_above_surface(u);
        if !climbing_shot && h > 0 && o.contact.is_some() {
            lane.save("jumpjet-factory-climb");
            climbing_shot = true;
            continue;
        }
        if reported.is_none() && o.contact.is_none() {
            reported = Some((w.tick, h, release::tethered(w, u)));
            println!(
                "contact ended at tick {} at height {h}, still tethered: {}",
                w.tick,
                release::tethered(w, u)
            );
            lane.ticks(20);
            lane.save("jumpjet-factory-away");
            break;
        }
    }
    println!(
        "climb shot: {climbing_shot}, contact ended: {}",
        reported.is_some()
    );
}

/// S-0501 claim 48 (c), *Low release on a move* (r62, Q-1942): an aircraft
/// released low from its pad. The lane gives a local copy of the loaded
/// rules a pad whose docking displacements sit 10 leptons up at a cell's
/// centre (AC30 part k (7)'s second pad) and the aircraft type a slowdown
/// distance of 2000, so that the release rests above the terrain, then
/// gives a move order as a person would: `ahead` along the released facing,
/// else at a right angle to it. In both runs it should reach height 0 in its
/// first movement update, take off at the move's first processing and climb
/// in its second, unturned up to half its cruise height; at a right angle it
/// then turns toward the destination. Writes a per-update trace
/// (`low-release-<run>.txt`: tick, cell, height, facing relative to the
/// release, speed fractions, destination, behaviour, contact) and screens.
/// Prints no type names or rules values.
fn aircraft_low_release(lane: &mut Lane, ahead: bool) {
    use glasnost_sim::econ::release;
    use std::fmt::Write as _;
    let run = if ahead { "ahead" } else { "abeam" };
    let w = lane.world();
    let r = w.rules();
    // A pad-bound aircraft type whose first dock type is a pad building, by flags only.
    let found = (0..r.aircraft.len() as u32).find_map(|i| {
        let u = r.unit(Kind::Aircraft, i)?;
        if u.locomotor != glasnost_sim::rules::Locomotor::Fly
            || !u.raw.econ.airport_bound.unwrap_or(false)
        {
            return None;
        }
        let first = u.raw.econ.dock.as_ref()?.first()?;
        let b = *r.names.get(&(Kind::Building, first.clone()))?;
        r.buildings[b as usize].building.helipad.then_some((i, b))
    });
    println!(
        "pad-bound aircraft with a pad found: {}; quirks.aircraft_released_above_pad: {}",
        found.is_some(),
        r.quirks.aircraft_released_above_pad
    );
    let Some((ai, bi)) = found else { return };
    let (bname, size) = (
        r.buildings[bi as usize].name.clone(),
        r.buildings[bi as usize].foundation,
    );
    // The destination lies off the pad, in whole cells from the docking cell,
    // and inside the slowdown distance S, so that the target fraction stays
    // below 1 at the start. The local copy sets S to AC30 part k (7)'s
    // pad-bound test type's 2000 leptons (the synthetic NEAR of
    // claims_s0501_released.rs), whatever the loaded type's is.
    let s_dist = 2000; // spec: S-0501 AC30 part k (7) (a slowdown distance large enough that the target fraction stays below 1)
    let cells = size.0.max(size.1) / 2 + 2;
    let local = lane.local();
    let home = lane.home();
    let Some(tl) = lane.area(CellXY::new(home.x + 6, home.y + 6), size, 8, |l| {
        l != land::WATER && l != land::ROCK
    }) else {
        println!("no room for the pad");
        return;
    };
    let k = lane.spawn(Kind::Building, &bname, local, tl);
    lane.ticks(2);
    // The local copy: every docking displacement of the pad type put at the
    // centre of the cell holding the pad's centre, 10 leptons up, so that a
    // cell straight ahead has its centre on the line of the run.
    let w = lane.world();
    let c = w.reference_point(k);
    let mid = glasnost_sim::world::Pos::centre(c.cell(), 0);
    let mut m = (*w.rules_arc()).clone();
    let econ = &mut m.buildings[bi as usize].raw.econ;
    let mut offs = econ.docking_offsets.clone().unwrap_or_default();
    if offs.is_empty() {
        offs.push([0; 3]);
    }
    for d in &mut offs {
        // spec: S-0501 AC30 part k (7) (the second pad's docking position 10 leptons above level ground, in a foundation cell)
        *d = [mid.x - c.x, mid.y - c.y, 10];
    }
    econ.docking_offsets = Some(offs);
    m.aircraft[ai as usize].slowdown_distance = s_dist;
    lane.world_mut().rules = Some(std::sync::Arc::new(m));
    let a = glasnost_sim::objects::create(lane.world_mut(), Kind::Aircraft, ai, Some(local));
    let out = release::release(lane.world_mut(), k, a);
    let w = lane.world();
    let Some(o) = w.obj(a) else { return };
    let f0 = o.body.value(w.tick);
    println!(
        "{run}: release {out:?}, height {} at cell {:?}",
        glasnost_sim::air::height(w, a),
        o.pos.cell()
    );
    lane.look_at(o.pos.cell());
    lane.ticks(1);
    lane.save(&format!("low-release-{run}-0-released"));
    // The destination: `cells` cells along the released facing, or a
    // quarter turn from it.
    let w = lane.world();
    let o = w.obj(a).unwrap();
    let f = if ahead { f0 } else { f0.wrapping_add(16384) };
    let dist = cells * 256;
    let (x, y) = glasnost_sim::walk::step_point(o.pos.x, o.pos.y, f, dist, true, w.rules().mode);
    let to = glasnost_sim::world::Pos::new(x, y, 0).cell();
    let n = lane.order(local, a, ClickOn::Cell(to));
    println!("{run}: destination inside S: {}, {n} orders", dist < s_dist);
    let mut trace = String::new();
    let mut climbed = None;
    let mut rest = 0;
    let mut turned_at_0 = false;
    let mut mid_shot = false;
    // The first traced update can still show the release height; a climb
    // counts only after the run has been at height 0.
    let mut been_at_0 = false;
    // Updates that begin at height 0 and end with a destination and no
    // take-off, and the update (counted from the order) whose end first
    // shows a take-off.
    let mut at_0_unsteered = 0;
    let mut began_at_0 = false;
    let mut take_off = None;
    let mut updates = 0;
    let mut turned_low = false;
    for _ in 0..400 {
        lane.ticks(1);
        let w = lane.world();
        let Some(o) = w.obj(a) else { break };
        let s = o.air.clone().unwrap_or_default();
        let h = glasnost_sim::air::height(w, a);
        let df = o.body.value(w.tick).wrapping_sub(f0) as i16;
        let _ = writeln!(
            trace,
            "tick {} cell {:?} x {} y {} h {h} held {} facing-delta {df} cur {:?} tgt {:?} dest {} beh {:?} contact {} released {} taking-off {} landing {} holding {}",
            w.tick,
            o.pos.cell(),
            o.pos.x,
            o.pos.y,
            s.held,
            s.cur,
            s.tgt,
            o.dest.is_some(),
            o.beh.current,
            o.contact.is_some(),
            s.released,
            s.taking_off,
            s.landing,
            s.holding,
        );
        been_at_0 |= h == 0;
        updates += 1;
        if h == 0 && climbed.is_none() && df != 0 {
            turned_at_0 = true;
        }
        if began_at_0 && o.dest.is_some() && !s.taking_off {
            at_0_unsteered += 1;
        }
        began_at_0 = h == 0;
        if take_off.is_none() && s.taking_off {
            take_off = Some(updates);
        }
        let half = glasnost_sim::air::cruise_height(w, a) / 2;
        if df != 0 && h <= half && climbed.is_some() {
            turned_low = true;
        }
        if climbed.is_none() && been_at_0 && h > 0 {
            climbed = Some(w.tick);
        }
        let (tick, cell, has_dest) = (w.tick, o.pos.cell(), o.dest.is_some());
        if !mid_shot && h == 0 && has_dest {
            // The screen of the run at height 0, without skipping a traced update.
            mid_shot = true;
            lane.look_at(cell);
            lane.save(&format!("low-release-{run}-1-at-height-0"));
        }
        if !has_dest && h == 0 {
            rest += 1;
            if rest == 30 {
                break;
            }
        }
        if let Some(t) = climbed
            && tick >= t + 60
        {
            break;
        }
    }
    let w = lane.world();
    let c = w.obj(a).map_or(to, |o| o.pos.cell());
    println!(
        "{run}: turned at height 0: {turned_at_0}, climbed: {}, resting with no destination at height 0 for {rest} updates, ends in the destination cell: {}",
        climbed.is_some(),
        c == to
    );
    println!(
        "{run}: updates begun at height 0 that end with a destination and no take-off: {at_0_unsteered}, take-off first seen at the end of update {take_off:?} after the order, turned at or below half its cruise height: {turned_low}"
    );
    lane.look_at(c);
    lane.ticks(1);
    lane.save(&format!("low-release-{run}-2-end"));
    let path = lane.dir.join(format!("low-release-{run}.txt"));
    std::fs::write(&path, trace).unwrap();
    println!("wrote {}", path.display());
}

fn aircraft_low_release_ahead(lane: &mut Lane) {
    aircraft_low_release(lane, true);
}

fn aircraft_low_release_abeam(lane: &mut Lane) {
    aircraft_low_release(lane, false);
}

/// S-0517: a vehicle that deploys in place is sent the click a person gives
/// on it (S-0214); it is drawn with its deploying presentation while it
/// changes (its own body hidden), with the deployed image when its type
/// names one, and a second click packs it up again.
fn unit_deploys(lane: &mut Lane) {
    use glasnost_sim::econ::inplace::{self, Form};
    let w = lane.world();
    let r = w.rules();
    let pick = |animated: bool| {
        (0..r.vehicles.len()).find_map(|i| {
            r.unit(Kind::Vehicle, i as u32)
                .filter(|u| {
                    u.simple_deployer
                        && u.passengers == 0
                        && !u.harvester
                        && u.deploying_anim.is_some() == animated
                        && !u.raw.deploys_into.as_deref().is_some_and(|d| !d.is_empty())
                })
                .map(|u| (u.name.clone(), u.jumpjet_locomotor))
        })
    };
    let found = pick(true).or_else(|| pick(false));
    let count = |f: &dyn Fn(&glasnost_sim::rules::UnitRules) -> bool| {
        (0..r.vehicles.len())
            .filter(|&i| r.unit(Kind::Vehicle, i as u32).is_some_and(f))
            .count()
    };
    println!(
        "vehicle types {}: simple deployers {}, with a deploying animation {}, deploying into a building {}",
        r.vehicles.len(),
        count(&|u| u.simple_deployer),
        count(&|u| u.deploying_anim.is_some()),
        count(&|u| u.raw.deploys_into.is_some())
    );
    println!("in-place deploying vehicle type found: {}", found.is_some());
    let Some((name, flier)) = found else { return };
    println!("a jumpjet type (landing first): {flier}");
    let local = lane.local();
    let home = lane.home();
    let at = lane.open_land(CellXY::new(home.x + 3, home.y + 3));
    let unit = lane.spawn(Kind::Vehicle, &name, local, at);
    lane.look_at(at);
    lane.ticks(4);
    lane.save("deploy-before");
    let sent = lane.order(local, unit, ClickOn::Object(unit));
    println!("click on the unit: {sent} orders");
    if sent == 0 {
        glasnost_sim::orders::issue(
            lane.world_mut(),
            local,
            glasnost_sim::orders::Order::Deploy { actor: unit },
        );
    }
    let mut shot = false;
    for _ in 0..600 {
        lane.ticks(1);
        let f = inplace::form(lane.world(), unit);
        if matches!(f, Form::Deploying { .. }) && !shot {
            lane.ticks(2);
            lane.save("deploy-transition");
            shot = true;
        }
        if f == Form::Deployed {
            break;
        }
    }
    println!(
        "form {:?}, body hidden {}, deployed image {:?}, behaviour {:?}",
        inplace::form(lane.world(), unit),
        inplace::body_hidden(lane.world(), unit),
        inplace::deployed_image(lane.world(), unit),
        lane.world().obj(unit).map(|o| o.beh.current)
    );
    lane.ticks(6);
    lane.save("deploy-deployed");
    let again = lane.order(local, unit, ClickOn::Object(unit));
    println!("second click: {again} orders");
    if again == 0 {
        glasnost_sim::orders::issue(
            lane.world_mut(),
            local,
            glasnost_sim::orders::Order::Deploy { actor: unit },
        );
    }
    for _ in 0..600 {
        lane.ticks(1);
        if inplace::form(lane.world(), unit) == Form::Mobile {
            break;
        }
    }
    lane.ticks(6);
    println!("form after: {:?}", inplace::form(lane.world(), unit));
    lane.save("deploy-packed");
}

/// S-0525: a radiation weapon of the install detonates on open ground (as a
/// shot would); infantry stand at the centre, one cell off and outside the
/// reach, and the field's ring is drawn. Prints the exposure and the health
/// lost, never a type name or a rules value.
fn radiation_field(lane: &mut Lane) {
    let w = lane.world();
    let r = w.rules();
    let weapon = (0..r.combat.weapons.len() as u32).find(|&i| r.combat.weapon(i).rad_level > 0);
    let man = (0..r.infantry.len())
        .find(|&i| {
            r.unit(Kind::Infantry, i as u32)
                .is_some_and(|u| !u.raw.immune_to_radiation.unwrap_or(false) && u.strength > 0)
        })
        .map(|i| r.infantry[i].name.clone());
    println!(
        "radiation weapon found: {}, infantry type found: {}",
        weapon.is_some(),
        man.is_some()
    );
    let (Some(weapon), Some(man)) = (weapon, man) else {
        return;
    };
    let local = lane.local();
    let home = lane.home();
    let centre = lane.open_land(CellXY::new(home.x + 8, home.y + 8));
    let spread = lane
        .world()
        .rules()
        .combat
        .warhead_of(weapon)
        .map_or(0, |h| h.spread.to_i32_common());
    let side = CellXY::new(centre.x + 1, centre.y);
    let far = CellXY::new(centre.x + spread + 3, centre.y);
    let units: Vec<ObjectId> = [centre, side, far]
        .iter()
        .map(|&c| lane.spawn(Kind::Infantry, &man, local, c))
        .collect();
    lane.look_at(centre);
    lane.ticks(4);
    lane.save("radiation-before");
    glasnost_sim::radiation::detonation(lane.world_mut(), weapon, centre);
    lane.ticks(2);
    println!(
        "fields {}, spread {spread}, exposure at the centre {}, one cell off {}, outside {}",
        lane.world().radiation.fields.len(),
        glasnost_sim::radiation::exposure(lane.world(), centre),
        glasnost_sim::radiation::exposure(lane.world(), side),
        glasnost_sim::radiation::exposure(lane.world(), far)
    );
    lane.save("radiation-field");
    let before: Vec<i32> = units
        .iter()
        .map(|&u| lane.world().obj(u).map_or(0, |o| o.health))
        .collect();
    lane.ticks(90);
    let after: Vec<i32> = units
        .iter()
        .map(|&u| lane.world().obj(u).map_or(0, |o| o.health))
        .collect();
    println!(
        "health lost after 90 ticks: {:?}",
        before
            .iter()
            .zip(&after)
            .map(|(b, a)| b - a)
            .collect::<Vec<_>>()
    );
    lane.save("radiation-damage");
}

/// S-0514: a unit with a temporal beam (found by its primary weapon's
/// warhead flag) is sent the attack click against an enemy vehicle; the
/// target is locked, sparkles, and is removed when its erasure amount has
/// run out. Prints counts and ticks only.
fn chrono_erases(lane: &mut Lane) {
    let w = lane.world();
    let r = w.rules();
    let eraser = (0..r.infantry.len())
        .map(|i| (Kind::Infantry, i as u32))
        .chain((0..r.vehicles.len()).map(|i| (Kind::Vehicle, i as u32)))
        .find(|&(k, i)| {
            r.unit(k, i).is_some_and(|u| {
                u.combat.slots[0]
                    .and_then(|p| r.combat.weapon(p).warhead)
                    .is_some_and(|h| r.combat.warhead(h).temporal)
            })
        })
        .map(|(k, i)| (k, r.unit(k, i).map(|u| u.name.clone()).unwrap_or_default()));
    let victim = (0..r.vehicles.len())
        .find(|&i| {
            r.unit(Kind::Vehicle, i as u32).is_some_and(|u| {
                u.combat.slots[0].is_none() && !u.harvester && u.strength > 0 && u.passengers == 0
            })
        })
        .map(|i| r.vehicles[i].name.clone());
    println!(
        "erasing type found: {}, target vehicle type found: {}, sparkle animation {}, warp-away {}",
        eraser.is_some(),
        victim.is_some(),
        r.combat.general.chrono_sparkle.is_some(),
        r.combat.general.warp_away.is_some()
    );
    let (Some((kind, ename)), Some(vname)) = (eraser, victim) else {
        return;
    };
    let local = lane.local();
    let enemy = (0..lane.world().houses.len())
        .map(|i| HouseId(i as u8))
        .find(|&h| {
            h != local
                && lane.world().houses[usize::from(h.0)].role
                    == glasnost_sim::world::HouseRole::Player
        })
        .unwrap_or(local);
    let home = lane.home();
    let spot = lane.open_land(CellXY::new(home.x + 8, home.y + 8));
    let at_e = CellXY::new(spot.x, spot.y + 1);
    let at_v = CellXY::new(spot.x + 3, spot.y);
    let e = lane.spawn(kind, &ename, local, at_e);
    let v = lane.spawn(Kind::Vehicle, &vname, enemy, at_v);
    // The victim's house keeps a second vehicle out of view, so erasing the victim does not
    // defeat it and end the game before "chrono-erased" (review 2026-10-19).
    let keep = lane.open_land(CellXY::new(home.x - 12, home.y - 12));
    lane.spawn(Kind::Vehicle, &vname, enemy, keep);
    lane.look_at(spot);
    lane.ticks(4);
    lane.save("chrono-before");
    let sent = lane.order(local, e, ClickOn::Object(v));
    println!("click on the target: {sent} orders");
    let mut locked_at = None;
    let mut shot = false;
    for t in 0..2400 {
        lane.ticks(1);
        let w = lane.world();
        if locked_at.is_none() && glasnost_sim::temporal::erased(w, v) {
            locked_at = Some(t);
        }
        if let Some(l) = locked_at
            && !shot
            && t >= l + 30
        {
            shot = true;
            let amount = w.temporal.locks.get(&v).map(|l| l.amount);
            println!("locked at {l}; amount after 30 updates {amount:?}");
            lane.save("chrono-erasing");
        }
        if lane.world().obj(v).is_none() {
            println!("target removed at tick {t}");
            break;
        }
    }
    lane.ticks(3);
    lane.save("chrono-erased");
}

/// S-0518: the prism type of the install (found by the game-wide setting)
/// is built three times near each other for the local house; the first is
/// ordered to attack an enemy vehicle, recruits the others, and fires one
/// stronger shot; the support beams are drawn as marks. Prints counts and
/// ticks only.
fn prism_supports(lane: &mut Lane) {
    let w = lane.world();
    let r = w.rules();
    let prism = r.combat.general.prism.building;
    let victim = (0..r.vehicles.len())
        .find(|&i| {
            r.unit(Kind::Vehicle, i as u32)
                .is_some_and(|u| u.combat.slots[0].is_none() && !u.harvester && u.strength > 0)
        })
        .map(|i| r.vehicles[i].name.clone());
    println!(
        "prism type named: {}, modifier {}, cap {}, target vehicle found: {}",
        prism.is_some(),
        r.combat.general.prism.modifier,
        r.combat.general.prism.max,
        victim.is_some()
    );
    let (Some(p), Some(vname)) = (prism, victim) else {
        return;
    };
    let pname = r.buildings[p as usize].name.clone();
    let local = lane.local();
    let enemy = (0..lane.world().houses.len())
        .map(|i| HouseId(i as u8))
        .find(|&h| {
            h != local
                && lane.world().houses[usize::from(h.0)].role
                    == glasnost_sim::world::HouseRole::Player
        })
        .unwrap_or(local);
    let home = lane.home();
    let spot = lane.open_land(CellXY::new(home.x + 8, home.y + 8));
    let sites = [(0, 0), (-3, 0), (-3, 3)];
    let prisms: Vec<ObjectId> = sites
        .iter()
        .map(|&(dx, dy)| {
            lane.spawn(
                Kind::Building,
                &pname,
                local,
                CellXY::new(spot.x + dx, spot.y + dy),
            )
        })
        .collect();
    let target = lane.spawn(
        Kind::Vehicle,
        &vname,
        enemy,
        CellXY::new(spot.x + 5, spot.y),
    );
    lane.look_at(spot);
    lane.ticks(4);
    lane.save("prism-before");
    let sent = lane.order(local, prisms[0], ClickOn::Object(target));
    println!("click on the target: {sent} orders");
    let hp = lane.world().obj(target).map_or(0, |o| o.health);
    let mut shot = false;
    let mut pending = false;
    let mut recruited = None;
    let mut peak = 0;
    for t in 0..400 {
        // The lane keeps the house at full power, as a person with enough plants would.
        lane.world_mut().houses[usize::from(local.0)]
            .econ
            .power
            .output = 100_000;
        lane.ticks(1);
        let n = lane.world().obj(prisms[0]).map_or(0, |o| o.combat.support);
        peak = peak.max(n);
        if n > 0 && recruited.is_none() {
            recruited = Some(t);
            println!("first recruitment at tick {t}");
        }
        // S-0518 claim 6: a supporter's beam is drawn max(d − 1, 0) ticks after the
        // recruitment, in the tick its pending beam is taken.
        let waiting = prisms.iter().any(|&p| {
            lane.world()
                .obj(p)
                .is_some_and(|o| o.combat.support_beam.is_some())
        });
        if pending && !waiting && !shot {
            shot = true;
            lane.save("prism-beams");
            println!("first support beam at tick {t}");
        }
        pending = waiting;
        if lane.world().obj(target).is_none() {
            break;
        }
    }
    println!(
        "support count peak {peak}, target health {hp} -> {:?}",
        lane.world().obj(target).map(|o| o.health)
    );
    lane.save("prism-after");
}

/// S-0519: a bomb planter plants on an enemy vehicle and the bomb goes off;
/// a unit with explosive charges sabotages an enemy building; a designator
/// calls an air strike onto one. Each part is played only when the install
/// has a type with the needed flag.
fn terror_attacks(lane: &mut Lane) {
    let w = lane.world();
    let r = w.rules();
    let planter = (0..r.infantry.len())
        .find(|&i| {
            r.unit(Kind::Infantry, i as u32)
                .is_some_and(|u| u.raw.ivan.unwrap_or(false))
        })
        .map(|i| r.infantry[i].name.clone());
    let charger = (0..r.infantry.len())
        .find(|&i| {
            r.unit(Kind::Infantry, i as u32)
                .is_some_and(|u| u.raw.c4.unwrap_or(false))
        })
        .map(|i| r.infantry[i].name.clone());
    let victim = (0..r.vehicles.len())
        .find(|&i| {
            r.unit(Kind::Vehicle, i as u32)
                .is_some_and(|u| u.combat.slots[0].is_none() && !u.harvester && u.strength > 0)
        })
        .map(|i| r.vehicles[i].name.clone());
    let wall_target = (0..r.buildings.len())
        .find(|&i| {
            let b = &r.buildings[i];
            b.raw.can_c4.unwrap_or(true)
                && b.raw.foundation.is_none_or(|f| f == [1, 1] || f == [2, 2])
                && b.strength > 0
        })
        .map(|i| r.buildings[i].name.clone());
    println!(
        "planter found: {}, charger found: {}, victim found: {}, building found: {}",
        planter.is_some(),
        charger.is_some(),
        victim.is_some(),
        wall_target.is_some()
    );
    let local = lane.local();
    let enemy = (0..lane.world().houses.len())
        .map(|i| HouseId(i as u8))
        .find(|&h| {
            h != local
                && lane.world().houses[usize::from(h.0)].role
                    == glasnost_sim::world::HouseRole::Player
        })
        .unwrap_or(local);
    let home = lane.home();
    let spot = lane.open_land(CellXY::new(home.x + 8, home.y + 8));
    lane.look_at(spot);
    if let (Some(p), Some(v)) = (planter, victim) {
        let bomber = lane.spawn(Kind::Infantry, &p, local, CellXY::new(spot.x - 2, spot.y));
        let target = lane.spawn(Kind::Vehicle, &v, enemy, CellXY::new(spot.x + 2, spot.y));
        lane.ticks(4);
        lane.save("terror-before");
        let sent = lane.order(local, bomber, ClickOn::Object(target));
        println!("click on the vehicle: {sent} orders");
        let hp = lane.world().obj(target).map_or(0, |o| o.health);
        let (mut planted, mut gone) = (None, None);
        for t in 0..2400 {
            lane.ticks(1);
            let has = lane
                .world()
                .obj(target)
                .is_some_and(|o| o.combat.bomb.is_some());
            if has && planted.is_none() {
                planted = Some(t);
                lane.save("terror-bomb");
            }
            // S-0519 claim 24, F3: the bomb icon a while after planting, with
            // its frame advanced.
            if has && planted.is_some_and(|p| t == p + 40) {
                let (sprite, palette) = {
                    let s = lane.c.session.as_ref().unwrap();
                    let live = s.loaded.live.as_ref().expect("a live game");
                    let has = |t: &str| live.notes.iter().any(|n| n.contains(t));
                    // S-0111 claim 19a: the cursor palette MOUSEPAL.PAL
                    (has("BOMBCURS"), has("MOUSEPAL"))
                };
                println!("bomb icon note present: sprite {sprite}, cursor palette {palette}");
                lane.save("terror-bomb-later");
            }
            if planted.is_some() && !has {
                gone = Some(t);
                lane.save("terror-boom");
                break;
            }
        }
        println!(
            "bomb planted at tick {planted:?}, ended at {gone:?}, target health {hp} -> {:?}",
            lane.world().obj(target).map(|o| o.health)
        );
    }
    if let (Some(c), Some(b)) = (charger, wall_target.clone()) {
        let size = lane
            .world()
            .rules()
            .buildings
            .iter()
            .find(|x| x.name == b)
            .map_or((1, 1), |x| x.foundation);
        let at = lane
            .area(CellXY::new(spot.x + 6, spot.y + 6), size, 2, |l| {
                l != land::WATER && l != land::ROCK
            })
            .unwrap_or(spot);
        let building = lane.spawn(Kind::Building, &b, enemy, at);
        let start = lane.open_land(CellXY::new(at.x - 5, at.y));
        let sabo = lane.spawn(Kind::Infantry, &c, local, start);
        lane.look_at(at);
        lane.ticks(4);
        let sent = lane.order(local, sabo, ClickOn::Object(building));
        println!("click on the building: {sent} orders");
        let (mut planted, mut gone) = (None, None);
        let (mut lit, mut unlit) = (false, false);
        for t in 0..1800 {
            lane.ticks(1);
            // S-0519 claim 25, F6: one shot with the flash on (c mod 4 in
            // {2, 3}) and one with it off while the count still runs.
            let flash = lane
                .world()
                .obj(building)
                .map_or(0, |o| o.combat.look.flash);
            if flash > 0 && glasnost_sim::mark::flash_on(flash) && !lit {
                lit = true;
                println!("flash count {flash}: on");
                lane.save("terror-charge-flash-on");
            }
            if flash > 0 && !glasnost_sim::mark::flash_on(flash) && lit && !unlit {
                unlit = true;
                println!("flash count {flash}: off");
                lane.save("terror-charge-flash-off");
            }
            if t % 300 == 0 {
                let o = lane.world().obj(sabo);
                println!(
                    "t {t}: charger {:?} at {:?} dest {:?} target {:?}",
                    o.map(|o| o.beh.current),
                    o.map(|o| o.pos.cell()),
                    o.map(|o| o.dest),
                    o.map(|o| o.target)
                );
            }
            let running = lane
                .world()
                .obj(building)
                .is_some_and(|o| o.combat.charge_planter.is_some());
            if running && planted.is_none() {
                planted = Some(t);
                lane.save("terror-charge");
            }
            if lane.world().obj(building).is_none() {
                gone = Some(t);
                break;
            }
        }
        println!("charge planted at tick {planted:?}, building gone at {gone:?}");
    }
    let rr = lane.world().rules();
    let designator = (0..rr.infantry.len())
        .find(|&i| {
            rr.unit(Kind::Infantry, i as u32)
                .is_some_and(|u| u.combat.airstrike_team > 0)
        })
        .map(|i| (Kind::Infantry, rr.infantry[i].name.clone()))
        .or_else(|| {
            (0..rr.vehicles.len())
                .find(|&i| {
                    rr.unit(Kind::Vehicle, i as u32)
                        .is_some_and(|u| u.combat.airstrike_team > 0)
                })
                .map(|i| (Kind::Vehicle, rr.vehicles[i].name.clone()))
        })
        .or_else(|| {
            (0..rr.buildings.len())
                .find(|&i| rr.buildings[i].combat.airstrike_team > 0)
                .map(|i| (Kind::Building, rr.buildings[i].name.clone()))
        });
    println!("designator found: {}", designator.is_some());
    if let (Some((dk, d)), Some(b)) = (designator, wall_target) {
        let size = lane
            .world()
            .rules()
            .buildings
            .iter()
            .find(|x| x.name == b)
            .map_or((1, 1), |x| x.foundation);
        let at = lane
            .area(CellXY::new(spot.x + 14, spot.y), size, 2, |l| {
                l != land::WATER && l != land::ROCK
            })
            .unwrap_or(spot);
        let target = lane.spawn(Kind::Building, &b, enemy, at);
        let start = lane.open_land(CellXY::new(at.x - 5, at.y + 3));
        let des = lane.spawn(dk, &d, local, start);
        lane.look_at(at);
        {
            let w = lane.world();
            let (slot, wp) = glasnost_sim::combat::weapon::would_use(
                w,
                des,
                Some(glasnost_sim::world::Target::Object(target)),
            );
            let flags = wp.map(|x| {
                let h = w.rules().combat.warhead_of(x);
                (h.map(|h| h.airstrike), w.rules().combat.weapon(x).range)
            });
            let tc = glasnost_sim::combat::veterancy::type_combat(w, des).unwrap();
            println!(
                "slot {slot:?} weapon found {} (airstrike flag, range) {flags:?}; team {} type {:?} designator {}",
                wp.is_some(),
                tc.airstrike_team,
                tc.airstrike_team_type[0].is_some(),
                glasnost_sim::strike::designator(w, des)
            );
        }
        println!(
            "cursor over the building: {:?}",
            targeting::cursor(
                lane.world(),
                local,
                &[des],
                ClickOn::Object(target),
                Modifiers::default()
            )
        );
        lane.ticks(4);
        let sent = lane.order(local, des, ClickOn::Object(target));
        println!("click with the designator: {sent} orders");
        let mut called = None;
        let (mut held, mut pulsed, mut fallen) = (false, false, false);
        for t in 0..1400 {
            lane.ticks(1);
            // S-0519 claim 26: shots in Bright hold, near the top of a Pulse
            // rise and near the bottom of a Pulse fall.
            if let Some(o) = lane.world().obj(target) {
                use glasnost_sim::mark::Phase;
                let tick = lane.world().tick;
                let phase = o.combat.look.phase;
                let m = glasnost_sim::mark::factor(phase, tick);
                let shot = match phase {
                    Phase::BrightHold(_) if !held => {
                        held = true;
                        Some("terror-strike-bright-hold")
                    }
                    Phase::PulseRise(_) if !pulsed && m.is_some_and(|m| m >= 480) => {
                        pulsed = true;
                        Some("terror-strike-pulse-rise")
                    }
                    Phase::PulseFall(_) if !fallen && m.is_some_and(|m| m <= 288) => {
                        fallen = true;
                        Some("terror-strike-pulse-fall")
                    }
                    _ => None,
                };
                if let Some(name) = shot {
                    println!("mark factor {m:?} at tick {tick}");
                    lane.save(name);
                }
            }
            if pulsed && fallen {
                break;
            }
            for pr in lane
                .world()
                .projectiles
                .values()
                .filter(|p| p.attacker == Some(des))
            {
                let h = lane.world().rules().combat.warhead_of(pr.weapon);
                println!(
                    "t {t}: projectile airstrike {:?} target {:?} attacker is designator {}",
                    h.map(|h| h.airstrike),
                    pr.target,
                    pr.attacker == Some(des)
                );
            }
            if t % 20 == 0 && t < 200 {
                let o = lane.world().obj(des);
                let ck = glasnost_sim::combat::check::check(
                    lane.world(),
                    des,
                    Some(glasnost_sim::world::Target::Object(target)),
                    1,
                    false,
                );
                println!("check slot 1: {ck:?}");
                println!(
                    "t {t}: designator {dk:?} {:?} at {:?} target {:?} strike {:?} projectiles {}",
                    o.map(|o| o.beh.current),
                    o.map(|o| o.pos.cell()),
                    o.map(|o| o.target),
                    lane.world().strike.strikes.len(),
                    lane.world().projectiles.len()
                );
            }
            if called.is_none() && glasnost_sim::strike::marked(lane.world(), target) {
                called = Some(t);
                lane.save("terror-strike");
            }
        }
        println!("strike called at tick {called:?}");
    }
}

/// S-0520: a trigger crate of fixed money contents is put on open land with
/// the rules' own overlays, a unit of the local house is ordered over it, and
/// the credits, the crate and its replacement (crates on) are watched; then a
/// heal-all crate and a reveal crate are picked up the same way.
fn crate_pickup(lane: &mut Lane) {
    let local = lane.local();
    let home = lane.home();
    let spot = lane.open_land(CellXY::new(home.x + 6, home.y + 6));
    let man = {
        let r = lane.world().rules();
        (0..r.infantry.len())
            .find(|&i| {
                r.unit(Kind::Infantry, i as u32)
                    .is_some_and(|u| !u.engineer && u.strength > 0)
            })
            .map(|i| r.infantry[i].name.clone())
    };
    println!("infantry type found: {}", man.is_some());
    let Some(man) = man else { return };
    lane.world_mut().econ.settings.crates = Some(true);
    let start = lane.open_land(CellXY::new(spot.x - 4, spot.y));
    let unit = lane.spawn(Kind::Infantry, &man, local, start);
    lane.look_at(spot);
    for (kind, label) in [(0, "money"), (2, "heal"), (8, "reveal")] {
        // The previous crate's animation ends first, so that no shot shows another crate's
        // animation (review 2026-10-19: "dollar signs for heal").
        for _ in 0..600 {
            let s = lane.c.session.as_ref().unwrap();
            if s.loaded
                .live
                .as_ref()
                .is_none_or(|l| l.playing_anims().is_empty())
            {
                break;
            }
            lane.ticks(1);
        }
        let at = CellXY::new(spot.x + 2 * (kind / 2), spot.y + 2);
        let placed = glasnost_sim::crates::place_trigger(lane.world_mut(), at, kind);
        let seen = lane
            .world()
            .crates
            .places
            .values()
            .filter(|p| p.trigger)
            .count();
        println!("{label} crate placed: {placed}, crates on the map: {seen}");
        lane.ticks(3);
        if kind == 0 {
            lane.save("crate-before");
        } else {
            // Each crate is shown before it is picked up (review 2026-10-19: "should there have
            // been a crate here?").
            lane.save(&format!("crate-{label}-before"));
        }
        let credits = lane.world().houses[usize::from(local.0)].credits;
        let target = lane
            .world()
            .crates
            .places
            .values()
            .filter(|p| p.trigger)
            .map(|p| p.cell)
            .next_back();
        let Some(target) = target else { continue };
        let orders = lane.order(local, unit, glasnost_sim::targeting::ClickOn::Cell(target));
        println!("click on the crate cell: {orders} orders");
        let mut got = None;
        for t in 0..400 {
            lane.ticks(1);
            if !glasnost_sim::crates::is_crate(lane.world(), target) {
                got = Some(t);
                break;
            }
        }
        let after = lane.world().houses[usize::from(local.0)].credits;
        println!(
            "{label} crate picked up at tick {got:?}, credits {credits} -> {after}, crates left {}",
            lane.world().crates.places.len()
        );
        // The pickup animation is created in the pickup tick and drawn from the next frame on.
        lane.ticks(4);
        lane.save(&format!("crate-{label}"));
    }
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
        let out = png::lane_out("m6c");
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
        // GLASNOST_TRACE_ANIMS=1: the animations playing in this shot, by name and world pixel.
        if std::env::var_os("GLASNOST_TRACE_ANIMS").is_some() {
            let s = self.c.session.as_ref().unwrap();
            let live = s.loaded.live.as_ref().expect("a live game");
            for (n, (x, y, z)) in live.playing_anims() {
                let (wx, wy) = glasnost_render::projection::project_lifted(x, y, z);
                println!("  anim {n} at world pixel ({wx}, {wy})");
            }
        }
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
