//! Local lane for the M7b features (never in CI): with `GLASNOST_GAME_DIR`
//! set, every scene of [`SCENES`] starts its own skirmish through the
//! windowless client, puts the objects it needs on the map by their rules
//! flags (never by type name), plays the scene and writes its screens to
//! `local-lane/m7b/` (git-ignored; `GLASNOST_LANE_OUT` overrides the
//! folder). Prints counts, cells and ticks only, never type names or rules
//! values.
//!
//! Scenes now (S-0616, particle systems; the view draws each particle as a
//! placeholder by its system's kind: a white puff for smoke, an orange dot
//! for fire, a yellow dot for sparks, a cyan dot for railgun, a green cloud
//! for gas, the railgun line spread from the system to its target point): a
//! gunner vehicle carrying a passenger whose mode selects its spark weapon
//! is clicked onto a damaged own vehicle and repairs it with its spark
//! emitter; fire and railgun emitters on an armed vehicle firing at an enemy
//! vehicle; a building's natural smoke; a refinery's smoke at an unloading
//! step; the destruction smoke of an enemy building destroyed by a clicked
//! attacker; the damage sparks of hurt cyborg infantry. Where no stock type
//! carries a creator (fire and railgun weapons, natural smoke, destruction
//! smoke, cyborg infantry), the scene says so and flags a stock type in the
//! lane's own copy of the rules (never in a file) so that the screen can
//! show it.
//!
//! Run with `cargo test --release -p glasnost-app --test m7b_lane --
//! --ignored --nocapture`. `GLASNOST_LANE_SCENE=a,b` plays only the scenes
//! with those keys; `GLASNOST_UI_MAP=n` picks the n-th offered map.

#![allow(missing_docs)]

use std::rc::Rc;
use std::sync::Arc;

use glasnost_app::game::client::{Client, run_job};
use glasnost_app::mapview::png;
use glasnost_map::CellXY;
use glasnost_map::terrain::land;
use glasnost_sim::gas::Behaviour;
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
    ("spark", spark_repair),
    ("fire", |l| flagged_emitter(l, Behaviour::Fire)),
    ("rail", |l| flagged_emitter(l, Behaviour::Railgun)),
    ("natural", natural_smoke),
    ("refinery", refinery_smoke),
    ("destroy", destruction_smoke),
    ("cyborg", damage_sparks),
];

#[test]
#[ignore = "local lane: needs GLASNOST_GAME_DIR"]
fn m7b_scenes() {
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

/// The kind of the particle-system type called `name`.
fn system_kind(w: &World, name: &str) -> Option<Behaviour> {
    w.rules()
        .gas
        .systems
        .iter()
        .find(|s| s.name.eq_ignore_ascii_case(name))
        .and_then(|s| s.kind)
}

/// Particle systems in play by kind (gas, smoke, fire, spark, railgun) and
/// their particles.
fn census(w: &World) -> ([usize; 5], usize) {
    let mut k = [0usize; 5];
    let mut n = 0;
    for s in w.gas.systems.values() {
        let i = match w
            .rules()
            .gas
            .systems
            .get(s.type_index as usize)
            .and_then(|t| t.kind)
        {
            Some(Behaviour::Smoke) => 1,
            Some(Behaviour::Fire) => 2,
            Some(Behaviour::Spark) => 3,
            Some(Behaviour::Railgun) => 4,
            _ => 0,
        };
        k[i] += 1;
        n += s.particles.len();
    }
    (k, n)
}

fn print_census(lane: &Lane, what: &str) {
    let (k, n) = census(lane.world());
    println!(
        "{what}: systems gas {} smoke {} fire {} spark {} railgun {}; particles {n}",
        k[0], k[1], k[2], k[3], k[4]
    );
}

/// The first stock particle-system type of `kind`.
fn system_of(w: &World, kind: Behaviour) -> Option<String> {
    w.rules()
        .gas
        .systems
        .iter()
        .find(|s| s.kind == Some(kind))
        .map(|s| s.name.clone())
}

/// Changes the lane's own copy of the rules (never a file): for creators
/// that no stock type carries, so that the screen can show them.
fn patch_rules(lane: &mut Lane, f: impl FnOnce(&mut glasnost_sim::rules::SimRules)) {
    let mut rules = (*lane.world().rules_arc()).clone();
    f(&mut rules);
    lane.world_mut().rules = Some(Arc::new(rules));
}

/// The first building type of at most 3 × 3 cells with strength, not a
/// wall or a gate (our pick for the patched scenes).
fn small_building(w: &World) -> Option<usize> {
    let r = w.rules();
    (0..r.buildings.len()).find(|&i| {
        let b = &r.buildings[i];
        b.strength > 0
            && (2..=3).contains(&b.foundation.0)
            && (2..=3).contains(&b.foundation.1)
            && !b.raw.econ.wall.unwrap_or(false)
    })
}

/// An armed vehicle type that is neither a harvester nor a transport.
fn plain_vehicle(w: &World) -> Option<String> {
    let r = w.rules();
    (0..r.vehicles.len())
        .find(|&i| {
            r.unit(Kind::Vehicle, i as u32).is_some_and(|u| {
                u.combat.slots[0].is_some()
                    && !u.harvester
                    && u.strength > 0
                    && u.passengers == 0
                    && !u.combat.gunner
            })
        })
        .map(|i| r.vehicles[i].name.clone())
}

/// A gunner vehicle type with a spark weapon in some slot, and that slot.
fn spark_gunner(w: &World) -> Option<(String, usize)> {
    let r = w.rules();
    (0..r.vehicles.len()).find_map(|i| {
        let u = r.unit(Kind::Vehicle, i as u32)?;
        if !u.combat.gunner {
            return None;
        }
        let s = u
            .combat
            .slots
            .iter()
            .position(|x| x.is_some_and(|x| r.combat.weapon(x).spark))?;
        Some((r.vehicles[i].name.clone(), s))
    })
}

/// An infantry type whose gunner mode is `mode`.
fn mode_infantry(w: &World, mode: usize) -> Option<String> {
    let r = w.rules();
    (0..r.infantry.len())
        .find(|&i| {
            r.unit(Kind::Infantry, i as u32)
                .is_some_and(|u| u.combat.ifv_mode == mode as i32 && u.strength > 0)
        })
        .map(|i| r.infantry[i].name.clone())
}

/// Ticks up to `n`, stopping when `done` holds; the ticks taken.
fn until(lane: &mut Lane, n: usize, done: impl Fn(&World) -> bool) -> Option<usize> {
    for t in 0..n {
        if done(lane.world()) {
            return Some(t);
        }
        lane.ticks(1);
    }
    None
}

/// S-0616 claim 1 (effect weapons, spark) on stock rules: a gunner vehicle
/// takes a passenger whose mode selects the slot with the spark weapon
/// (S-0308 claim 6) and is clicked onto a damaged own vehicle; the shot
/// makes a spark emitter at the fire point, which emits while it lives and
/// keeps the weapon rearming (S-0301 claim 5).
fn spark_repair(lane: &mut Lane) {
    let w = lane.world();
    let g = spark_gunner(w);
    let p = g.as_ref().and_then(|(_, s)| mode_infantry(w, *s));
    let v = plain_vehicle(w);
    println!(
        "spark gunner found: {}, passenger type found: {}, patient type found: {}",
        g.is_some(),
        p.is_some(),
        v.is_some()
    );
    let (Some((gname, _)), Some(pname), Some(vname)) = (g, p, v) else {
        return;
    };
    let local = lane.local();
    let home = lane.home();
    let at = lane.open_land(glasnost_map::CellXY::new(home.x + 8, home.y + 8));
    let ifv = lane.spawn(Kind::Vehicle, &gname, local, at);
    let pass = lane.spawn(Kind::Infantry, &pname, local, CellXY::new(at.x, at.y + 1));
    let patient = lane.spawn(Kind::Vehicle, &vname, local, CellXY::new(at.x + 3, at.y));
    let half = lane.strength(patient) * 3 / 10;
    if let Some(o) = lane.world_mut().obj_mut(patient) {
        o.health = half;
    }
    glasnost_sim::transport::board(lane.world_mut(), pass, ifv);
    lane.ticks(2);
    lane.look_at(CellXY::new(at.x + 1, at.y));
    let cursor = targeting::cursor(
        lane.world(),
        local,
        &[ifv],
        ClickOn::Object(patient),
        Modifiers::default(),
    );
    let n = lane.order(local, ifv, ClickOn::Object(patient));
    println!(
        "passenger aboard: {}; click on the damaged vehicle: cursor {cursor:?}, {n} orders; health {} per mille",
        lane.world()
            .obj(pass)
            .is_some_and(|o| o.transport.riding == Some(ifv)),
        lane.permille(patient)
    );
    let made = until(lane, 300, |w| {
        w.obj(ifv).is_some_and(|o| o.combat.spark_emitter.is_some())
    });
    println!("spark emitter made after {made:?} ticks");
    lane.ticks(6);
    print_census(lane, "spark emitter");
    lane.save("spark-emitter");
    let gone = until(lane, 600, |w| {
        w.obj(ifv).is_some_and(|o| o.combat.spark_emitter.is_none())
    });
    println!(
        "spark emitter gone after {gone:?} more ticks; patient at {} per mille",
        lane.permille(patient)
    );
    lane.ticks(2);
    lane.save("spark-after");
}

/// S-0616 claims 1, 10, 11 (fire) or claim 1 (railgun): no stock weapon is
/// a fire or railgun weapon, so the lane's own copy of the rules flags an
/// armed vehicle's first weapon so and attaches a stock system of that kind
/// to it; the vehicle is clicked onto an enemy vehicle. Fire: the emitter
/// follows its owner and sweeps when it turns (F5 to F7); railgun: the beam
/// system at the shot.
fn flagged_emitter(lane: &mut Lane, kind: Behaviour) {
    let key = if kind == Behaviour::Fire {
        "fire"
    } else {
        "rail"
    };
    let w = lane.world();
    let sys = system_of(w, kind);
    let v = plain_vehicle(w);
    let enemy = lane.enemy();
    let stock = w.rules().combat.weapons.iter().filter(|x| match kind {
        Behaviour::Fire => x.fire_particles,
        _ => x.railgun,
    });
    println!(
        "stock {key} weapons: {}; stock {key} system types: {}; vehicle type found: {}",
        stock.count(),
        w.rules()
            .gas
            .systems
            .iter()
            .filter(|s| s.kind == Some(kind))
            .count(),
        v.is_some()
    );
    let (Some(sys), Some(vname), Some(enemy)) = (sys, v, enemy) else {
        return;
    };
    let r = lane.world().rules();
    let ti = r.index_of(Kind::Vehicle, &vname).expect("the type");
    let Some(wid) = r.unit(Kind::Vehicle, ti).and_then(|u| u.combat.slots[0]) else {
        return;
    };
    patch_rules(lane, |r| {
        let wr = &mut r.combat.weapons[wid as usize];
        match kind {
            Behaviour::Fire => wr.fire_particles = true,
            _ => wr.railgun = true,
        }
        wr.attached_system = Some(sys);
    });
    let local = lane.local();
    let home = lane.home();
    let at = lane.open_land(CellXY::new(home.x + 8, home.y + 8));
    let a = lane.spawn(Kind::Vehicle, &vname, local, at);
    let t = lane.spawn(
        Kind::Vehicle,
        &vname,
        enemy,
        CellXY::new(at.x + 3, at.y + 1),
    );
    lane.ticks(2);
    lane.look_at(CellXY::new(at.x + 1, at.y));
    let n = lane.order(local, a, ClickOn::Object(t));
    println!("click on the enemy vehicle: {n} orders");
    let made = until(lane, 300, |w| {
        w.obj(a).is_some_and(|o| match kind {
            Behaviour::Fire => o.combat.fire_emitter.is_some(),
            _ => o.combat.railgun_emitter.is_some(),
        })
    });
    println!("{key} emitter made after {made:?} ticks");
    let mut seen = Vec::new();
    for _ in 0..if kind == Behaviour::Fire { 12 } else { 3 } {
        let e = lane.world().obj(a).and_then(|o| match kind {
            Behaviour::Fire => o.combat.fire_emitter.as_ref().map(|e| e.system),
            _ => o.combat.railgun_emitter,
        });
        if seen.last() != Some(&e) {
            seen.push(e);
        }
        lane.ticks(1);
    }
    println!(
        "{key} place over the next ticks (distinct holders in order): {}; target alive {}",
        seen.len(),
        lane.world().obj(t).is_some()
    );
    print_census(lane, key);
    for s in lane.world().gas.systems.values() {
        if s.owner_object != Some(a) {
            continue;
        }
        let xs = s.particles.iter().map(|p| p.pos.x);
        let ys = s.particles.iter().map(|p| p.pos.y);
        println!(
            "{key} system of the firer: from {:?} to {:?} (firer {:?}, target {:?}); particle x {:?}..{:?}, y {:?}..{:?}",
            (s.pos.x, s.pos.y, s.pos.z),
            (s.target.x, s.target.y, s.target.z),
            lane.world().obj(a).map(|o| (o.pos.x, o.pos.y)),
            lane.world().obj(t).map(|o| (o.pos.x, o.pos.y)),
            xs.clone().min(),
            xs.max(),
            ys.clone().min(),
            ys.max()
        );
    }
    lane.save(&format!("{key}-emitter"));
    if kind == Behaviour::Fire {
        // A second enemy beside: the fire turn lets go of the old emitter when the
        // remembered target leaves reach, and a turning firer sweeps (F5).
        let t2 = lane.spawn(
            Kind::Vehicle,
            &vname,
            enemy,
            CellXY::new(at.x - 1, at.y + 3),
        );
        let _ = lane.order(local, a, ClickOn::Object(t2));
        lane.ticks(10);
        print_census(lane, "fire, retargeted");
        lane.save("fire-sweep");
    }
}

/// S-0616 claim 1 (natural smoke): a building type with a natural system
/// placed near the start makes its system at placement.
fn natural_smoke(lane: &mut Lane) {
    let w = lane.world();
    let r = w.rules();
    let b = (0..r.buildings.len()).find(|&i| {
        let b = &r.buildings[i];
        b.combat.natural_system.is_some() && b.foundation.0 <= 3 && b.foundation.1 <= 3
    });
    println!(
        "building types with a natural system: {}",
        r.buildings
            .iter()
            .filter(|b| b.combat.natural_system.is_some())
            .count()
    );
    let bi = match b {
        Some(bi) => bi,
        None => {
            // None in the stock rules: the lane's copy gives a small building a smoke system.
            let (Some(bi), Some(sys)) = (small_building(w), system_of(w, Behaviour::Smoke)) else {
                return;
            };
            println!("patched: a building type gets a natural smoke system (lane copy only)");
            patch_rules(lane, |r| {
                r.buildings[bi].combat.natural_system = Some(sys);
                r.buildings[bi].combat.natural_offset = [0, 0, 100];
            });
            bi
        }
    };
    let r = lane.world().rules();
    let name = r.buildings[bi].name.clone();
    let local = lane.local();
    let home = lane.home();
    let Some(at) = lane.area(CellXY::new(home.x + 8, home.y + 8), (3, 3), 1, |l| {
        l != land::WATER && l != land::ROCK
    }) else {
        println!("no room for the scene");
        return;
    };
    let id = lane.spawn(Kind::Building, &name, local, at);
    lane.look_at(CellXY::new(at.x + 1, at.y + 1));
    lane.ticks(40);
    println!(
        "natural smoke held: {}",
        lane.world()
            .obj(id)
            .is_some_and(|o| o.combat.natural_smoke.is_some())
    );
    print_census(lane, "natural smoke");
    lane.save("natural-smoke");
}

/// S-0616 claim 1 (refinery smoke): a refinery type with a smoke system and
/// set points; the creator runs as at an unloading step (called by the
/// lane, as `econ::harvest` calls it after each payment row).
fn refinery_smoke(lane: &mut Lane) {
    let w = lane.world();
    let r = w.rules();
    let b = (0..r.buildings.len()).find(|&i| {
        r.buildings[i]
            .raw
            .refinery_smoke_particle_system
            .as_deref()
            .is_some_and(|s| system_kind(w, s).is_some())
    });
    println!("refinery type with a smoke system found: {}", b.is_some());
    let Some(bi) = b else { return };
    let name = r.buildings[bi].name.clone();
    let (fw, fh) = r.buildings[bi].foundation;
    let local = lane.local();
    let home = lane.home();
    let Some(at) = lane.area(CellXY::new(home.x + 8, home.y + 8), (fw, fh), 1, |l| {
        l != land::WATER && l != land::ROCK
    }) else {
        println!("no room for the scene");
        return;
    };
    let id = lane.spawn(Kind::Building, &name, local, at);
    lane.look_at(CellXY::new(at.x + fw / 2, at.y + fh / 2));
    lane.ticks(2);
    glasnost_sim::emitters::refinery_smoke(lane.world_mut(), id);
    lane.ticks(15);
    print_census(lane, "refinery smoke");
    lane.save("refinery-smoke");
}

/// S-0616 claim 1 (destruction smoke, S-0312 claim 19 f): an enemy building
/// type whose destruction list has a smoke system, at low health, is
/// destroyed by clicked attackers; the smoke rises where it stood.
fn destruction_smoke(lane: &mut Lane) {
    let w = lane.world();
    let r = w.rules();
    let b = (0..r.buildings.len()).find(|&i| {
        let b = &r.buildings[i];
        b.strength > 0
            && b.foundation.0 <= 3
            && b.foundation.1 <= 3
            && b.combat
                .destroy_systems
                .iter()
                .any(|s| system_kind(w, s) == Some(Behaviour::Smoke))
    });
    let v = plain_vehicle(w);
    let enemy = lane.enemy();
    let mut kinds = std::collections::BTreeMap::new();
    for b in &r.buildings {
        for s in &b.combat.destroy_systems {
            *kinds
                .entry(format!("{:?}", system_kind(w, s)))
                .or_insert(0usize) += 1;
        }
    }
    println!("destruction list entries by system kind: {kinds:?}");
    println!(
        "building type with destruction smoke found: {}, attacker found: {}",
        b.is_some(),
        v.is_some()
    );
    let (Some(vname), Some(enemy)) = (v, enemy) else {
        return;
    };
    let bi = match b {
        Some(bi) => bi,
        None => {
            // None in the stock rules: the lane's copy gives a small building a smoke system in its list.
            let (Some(bi), Some(sys)) = (small_building(w), system_of(w, Behaviour::Smoke)) else {
                return;
            };
            println!("patched: a building type gets destruction smoke (lane copy only)");
            patch_rules(lane, |r| r.buildings[bi].combat.destroy_systems = vec![sys]);
            bi
        }
    };
    let r = lane.world().rules();
    let name = r.buildings[bi].name.clone();
    let local = lane.local();
    let home = lane.home();
    let Some(at) = lane.area(CellXY::new(home.x + 8, home.y + 8), (3, 3), 3, |l| {
        l != land::WATER && l != land::ROCK
    }) else {
        println!("no room for the scene");
        return;
    };
    let id = lane.spawn(Kind::Building, &name, enemy, at);
    if let Some(o) = lane.world_mut().obj_mut(id) {
        o.health = 1;
    }
    let a = lane.spawn(
        Kind::Vehicle,
        &vname,
        local,
        CellXY::new(at.x - 3, at.y + 1),
    );
    lane.look_at(CellXY::new(at.x + 1, at.y + 1));
    lane.ticks(2);
    let n = lane.order(local, a, ClickOn::Object(id));
    println!("click on the enemy building: {n} orders");
    let before = lane.world().gas.systems.len();
    let mut gone = None;
    let mut most = 0;
    for t in 0..400 {
        lane.ticks(1);
        let w = lane.world();
        most = most.max(census(w).0[1]);
        if gone.is_none() && w.obj(id).is_none_or(|o| o.health <= 0 || !o.on_map) {
            gone = Some(t);
        }
        if gone.is_some_and(|g| t >= g + 10) {
            break;
        }
    }
    println!(
        "building destroyed after {gone:?} ticks; systems before {before}, most smoke systems seen at once {most} \
         (S-0616 claim 1: a building leaving play at once marks its smoke by claim 4 before it emits; an exploding one lets it emit until its removal)"
    );
    print_census(lane, "destruction smoke");
    lane.save("destroy-smoke");
}

/// S-0616 claim 1 (damage sparks): cyborg infantry with spark entries in
/// its damage list, hurt below the yellow line, spark now and then.
fn damage_sparks(lane: &mut Lane) {
    let w = lane.world();
    let r = w.rules();
    let t = (0..r.infantry.len()).find(|&i| {
        r.unit(Kind::Infantry, i as u32)
            .is_some_and(|u| u.cyborg && !u.combat.spark_systems.is_empty() && u.strength > 0)
    });
    let n = |f: &dyn Fn(&glasnost_sim::rules::UnitRules) -> bool| {
        (0..r.infantry.len())
            .filter(|&i| r.unit(Kind::Infantry, i as u32).is_some_and(f))
            .count()
    };
    println!(
        "cyborg infantry types {}, infantry types with spark entries {}, vehicle types with spark entries {}",
        n(&|u| u.cyborg),
        n(&|u| !u.combat.spark_systems.is_empty()),
        (0..r.vehicles.len())
            .filter(|&i| r
                .unit(Kind::Vehicle, i as u32)
                .is_some_and(|u| !u.combat.spark_systems.is_empty()))
            .count()
    );
    println!("cyborg infantry type with sparks found: {}", t.is_some());
    let ti = match t {
        Some(ti) => ti,
        None => {
            // None in the stock rules: the lane's copy makes an infantry type a cyborg with a spark entry.
            let sys = r
                .vehicles
                .iter()
                .find_map(|v| v.combat.spark_systems.first().cloned());
            let ti = (0..r.infantry.len()).find(|&i| {
                r.unit(Kind::Infantry, i as u32)
                    .is_some_and(|u| u.strength > 0)
            });
            let (Some(ti), Some(sys)) = (ti, sys) else {
                return;
            };
            println!(
                "patched: an infantry type becomes a cyborg with a spark entry (lane copy only)"
            );
            patch_rules(lane, |r| {
                r.infantry[ti].cyborg = true;
                r.infantry[ti].combat.spark_systems = vec![sys];
            });
            ti
        }
    };
    let r = lane.world().rules();
    let name = r.infantry[ti].name.clone();
    let local = lane.local();
    let home = lane.home();
    let at = lane.open_land(CellXY::new(home.x + 8, home.y + 8));
    let mut ids = Vec::new();
    for k in 0..4 {
        let id = lane.spawn(Kind::Infantry, &name, local, CellXY::new(at.x + k, at.y));
        let h = lane.strength(id) / 5;
        if let Some(o) = lane.world_mut().obj_mut(id) {
            o.health = h.max(1);
        }
        ids.push(id);
    }
    lane.look_at(CellXY::new(at.x + 2, at.y));
    let first = until(lane, 1500, |w| {
        ids.iter()
            .any(|&i| w.obj(i).is_some_and(|o| o.combat.spark_emitter.is_some()))
    });
    println!("first damage sparks after {first:?} ticks");
    lane.ticks(2);
    print_census(lane, "damage sparks");
    lane.save("cyborg-sparks");
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

    /// Starts a skirmish on the `map`-th offered map.
    fn start_on(map: usize) -> Option<Lane> {
        let dir = std::env::var_os("GLASNOST_GAME_DIR")?;
        let out = std::env::var_os("GLASNOST_LANE_OUT")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| {
                std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../local-lane/m7b")
            });
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
        let local = s.play.local;
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
        self.c.session.as_ref().unwrap().play.local
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
