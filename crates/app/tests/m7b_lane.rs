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
//! S-0615 (special-hit effects): one human infantry unit of the enemy house
//! killed by a stock warhead of each death kind with an animation (3 to 7
//! and 10), each death's effect on its own screen; a burning walker running
//! from a kind-4 death and expiring; a disc-laser carrier clicked onto an
//! enemy vehicle, its ring of beams closing and the final beam with the hit.
//!
//! S-0608 (drain): a drainer of the local house clicked onto an enemy
//! drainable power plant, and onto an enemy drainable refinery (the credits
//! it moves); an enemy drainer clicked onto a local power plant, and the
//! local house's blackout.
//!
//! S-0611 (mutation): the building that gives the genetic mutation
//! superweapon placed for the local house, its record charged (the charge
//! time cut short in the lane's own copy of the rules), and a launch by a
//! fire order at a group of enemy human infantry: the launch, the
//! conversion animations and the new infantry of the local house.
//!
//! S-0306 claims 28 to 32 and S-0301 claim 24 (projectiles and the target
//! line): a vehicle whose weapon fires an arcing sprite projectile, with
//! the target-laser flag (given in the lane's copy of the rules when the
//! stock rules lack it), and the same vehicle with its weapon's projectile
//! replaced in the lane's copy of the rules by one drawn as a voxel model,
//! each clicked onto an enemy of its type; frames while the projectiles fly.
//!
//! S-0306 claim 23 and S-0309 claim 18 (M8 step A3 intake 5, the lead's
//! play note 4): every building type whose weapons' projectiles are anti-air
//! and not anti-ground, of the local (human) house and of an enemy
//! (computer) house, attacked by an enemy ground vehicle and an enemy
//! infantry unit from inside its range, and in a second place by an enemy
//! aircraft: whether its type has land-targeting mode 1 or the SAM flag
//! (counts only), and the building's targets and shots at each attacker. The
//! lane's copy of the rules lets the building need no power; the computer
//! run clears the owner's human-controlled flag.
//!
//! S-0316 (electric bolts, M8 step C5): a vehicle, an infantry unit and a
//! building whose weapon has the electric-bolt flag (found by the flag;
//! where no stock type of that kind carries such a weapon, the lane's copy
//! of the rules gives one the first electric-bolt weapon), each firing at
//! an enemy vehicle; frames while bolts are shown, with the bolt, attached
//! bolt, pixel and spark-system counts.
//!
//! M8 step A4 round 3 (the lead's play notes 13 and 15): `aa-auto` puts
//! every building, vehicle and infantry type with an anti-air weapon in
//! slots 0 or 1 (any slot for a gunner type) on open land, of the local
//! (human) house and of a test-only computer house, and flies an enemy
//! aircraft across it on a move order (never an attack order, so that no
//! retaliation decides): the tick it comes into the weapon's range in
//! flight, the tick the searcher takes it as its target and the tick of the
//! first shot at it (S-0309 claims 1 to 9; S-0303 Terms). `debris-left`
//! kills enemy infantry by each death kind and enemy vehicles and counts,
//! 300 and 1000 ticks later, what stays on the map: animations by kind
//! (bouncing, corpse, endless, other), voxel debris pieces and particle
//! systems (S-0107 claim 17; S-0109; S-0317; S-0616).
//!
//! M8 step A4, round 4 (the lead's notes 16 to 18): `carrier-death` puts
//! each spawner type that launches aircraft on open water of the first map
//! that has some, orders it to attack an enemy vehicle, kills it once a slot
//! attacks or returns and counts, per slot aircraft, the ticks to health 0
//! and to leaving play (S-0506 claims 12 and 13; S-0501 claims 9 and 36;
//! S-0312 claim 16). `mirage-click` lets an enemy disguise-when-still
//! vehicle take its disguise and counts the cursor every armed own vehicle
//! and infantry type shows over it, plain and with force-fire, with the
//! clicked thing chosen as the client chooses it: disguised, with its fake
//! blink running, and without the disguise as the control (S-0214 claim 6,
//! Terms; S-0516 claims 35, 36 and 39). `derrick-look` finds the map's
//! buildings that need an engineer and have slot animations, and reports
//! one fresh, damaged, captured by the local house and captured by the
//! enemy: owner, manned state, drawn live, running slots and whether its
//! colour is the non-player grey (S-0103 claims 8 and 10; S-0515 claim 2;
//! Q-1772).
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
    ("deaths", death_kinds),
    ("walker", burning_walker),
    ("disc", disc_laser),
    ("drain-power", |l| drain(l, Drained::Power)),
    ("drain-refinery", |l| drain(l, Drained::Refinery)),
    ("drain-blackout", |l| drain(l, Drained::Blackout)),
    ("mutator", mutator),
    ("dominator", dominator),
    ("shot-sprite", sprite_shots),
    ("shot-voxel", voxel_shots),
    ("aa-only", aa_only),
    ("bolt-vehicle", |l| bolts(l, Kind::Vehicle)),
    ("bolt-infantry", |l| bolts(l, Kind::Infantry)),
    ("bolt-building", |l| bolts(l, Kind::Building)),
    ("aa-auto", aa_auto),
    ("debris-left", debris_left),
    ("carrier-death", carrier_death),
    ("mirage-click", mirage_click),
    ("derrick-look", derrick_look),
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

/// A human infantry type with strength that is not a cyborg, a jumpjet or
/// marked not human (S-0107 claim 17's ordinary human rows).
fn human_infantry(w: &World) -> Option<String> {
    let r = w.rules();
    (0..r.infantry.len())
        .find(|&i| {
            r.unit(Kind::Infantry, i as u32).is_some_and(|u| {
                u.strength > 0 && !u.cyborg && !u.jumpjet && !u.raw.not_human.unwrap_or(false)
            })
        })
        .map(|i| r.infantry[i].name.clone())
}

/// The first stock warhead of infantry-death kind `k`.
fn warhead_of_kind(w: &World, k: i32) -> Option<u32> {
    w.rules()
        .combat
        .warheads
        .iter()
        .position(|h| h.inf_death == k)
        .map(|i| i as u32)
}

/// Kills `id` with a hit of warhead `h` that ignores defences.
fn kill_with(lane: &mut Lane, id: ObjectId, h: u32) {
    let amount = lane.strength(id) * 10;
    glasnost_sim::combat::damage::apply(
        lane.world_mut(),
        id,
        glasnost_sim::combat::damage::Hit {
            amount,
            warhead: Some(h),
            distance: 0,
            attacker: None,
            house: None,
            ignore_defences: true,
            no_escape: false,
        },
    );
}

/// S-0615 claims 1 to 3: for each death kind with an animation (3 to 7
/// and 10), an enemy human infantry unit is killed by a stock warhead of
/// that kind; the screen shows the effect a few ticks later. A kind no stock
/// warhead has is made directly at the unit's position (lane only).
fn death_kinds(lane: &mut Lane) {
    let Some(name) = human_infantry(lane.world()) else {
        println!("human infantry type found: false");
        return;
    };
    let Some(enemy) = lane.enemy() else { return };
    let home = lane.home();
    let at = lane.open_land(CellXY::new(home.x + 8, home.y + 8));
    lane.look_at(at);
    for k in [3, 4, 5, 6, 7, 10] {
        let id = lane.spawn(Kind::Infantry, &name, enemy, at);
        lane.ticks(2);
        let pos = lane.world().obj(id).map(|o| o.pos);
        let before = lane.counts().0;
        match warhead_of_kind(lane.world(), k) {
            Some(h) => kill_with(lane, id, h),
            None => {
                println!("kind {k}: no stock warhead, the effect made directly");
                glasnost_sim::objects::remove(lane.world_mut(), id);
                if let Some(p) = pos {
                    glasnost_sim::combat::death::death_kind_effect(lane.world_mut(), k, p);
                }
            }
        }
        let gone = lane.world().obj(id).is_none();
        let walkers = lane.world().walkers.len();
        lane.ticks(4);
        println!(
            "kind {k}: unit removed {gone}, walkers {walkers}, animations created {}",
            lane.counts().0 - before
        );
        lane.save(&format!("death-kind-{k}"));
        lane.ticks(60);
    }
}

/// S-0615 claims 4 to 11: three enemy human infantry units killed by a
/// stock kind-4 warhead become burning walkers that run towards water (or
/// stumble about), then expire with damage over time and are removed.
fn burning_walker(lane: &mut Lane) {
    let w = lane.world();
    let (Some(name), Some(h)) = (human_infantry(w), warhead_of_kind(w, 4)) else {
        println!("human infantry and a kind-4 warhead found: false");
        return;
    };
    let flaming = w
        .rules()
        .combat
        .general
        .flaming_infantry
        .as_ref()
        .and_then(|n| w.rules().animation(n))
        .is_some_and(|a| a.flaming);
    println!("the burning death type is a burning walker: {flaming}");
    let Some(enemy) = lane.enemy() else { return };
    let home = lane.home();
    let at = lane.open_land(CellXY::new(home.x + 8, home.y + 8));
    lane.look_at(at);
    for k in 0..3 {
        let id = lane.spawn(Kind::Infantry, &name, enemy, CellXY::new(at.x + k, at.y));
        lane.ticks(1);
        kill_with(lane, id, h);
    }
    println!("walkers made: {}", lane.world().walkers.len());
    lane.ticks(6);
    lane.save("walker-running");
    lane.ticks(20);
    lane.save("walker-later");
    let expiring = until(lane, 400, |w| w.walkers.values().any(|k| k.expiring));
    println!("first expiry after {expiring:?} more ticks");
    lane.ticks(3);
    lane.save("walker-expiring");
    let gone = until(lane, 600, |w| w.walkers.is_empty());
    println!("all walkers gone after {gone:?} more ticks");
}

/// A stock unit type (vehicle or aircraft) whose first weapon is a disc
/// laser, and its kind.
fn disc_carrier(w: &World) -> Option<(Kind, String)> {
    let r = w.rules();
    for (kind, names) in [(Kind::Vehicle, &r.vehicles), (Kind::Aircraft, &r.aircraft)] {
        for (i, t) in names.iter().enumerate() {
            let disc = r
                .unit(kind, i as u32)
                .is_some_and(|u| u.combat.slots[0].is_some_and(|x| r.combat.weapon(x).disk_laser));
            if disc {
                return Some((kind, t.name.clone()));
            }
        }
    }
    None
}

/// S-0615 claims 12 to 17: a disc-laser carrier of the local house is
/// clicked onto an enemy vehicle; the ring of beams closes over 16 ticks and
/// the final beam lands with the damage at tick T + 17.
fn disc_laser(lane: &mut Lane) {
    let w = lane.world();
    let c = disc_carrier(w);
    let v = plain_vehicle(w);
    println!(
        "disc carrier found: {}, target type found: {}",
        c.is_some(),
        v.is_some()
    );
    let (Some((kind, cname)), Some(vname), Some(enemy)) = (c, v, lane.enemy()) else {
        return;
    };
    let local = lane.local();
    let home = lane.home();
    let at = lane.open_land(CellXY::new(home.x + 8, home.y + 8));
    let disc = lane.spawn(kind, &cname, local, at);
    let target = lane.spawn(Kind::Vehicle, &vname, enemy, CellXY::new(at.x + 3, at.y));
    lane.ticks(2);
    lane.look_at(CellXY::new(at.x + 1, at.y));
    let n = lane.order(local, disc, ClickOn::Object(target));
    let h0 = lane.permille(target);
    println!("click on the enemy vehicle: {n} orders; target at {h0} per mille");
    let shot = until(lane, 400, |w| !w.discs.is_empty());
    println!("disc laser shot after {shot:?} ticks");
    if shot.is_none() {
        return;
    }
    lane.ticks(7);
    lane.save("disc-ring");
    lane.ticks(6);
    lane.save("disc-ring-closing");
    // The client drains the effects each frame, so the hit is seen by the target's health.
    let health = lane.world().obj(target).map_or(0, |o| o.health);
    let hit = until(lane, 20, |w| {
        w.obj(target).is_none_or(|o| o.health < health)
    });
    println!(
        "final step after {hit:?} more ticks; target at {} per mille",
        lane.permille(target)
    );
    lane.save("disc-hit");
    lane.ticks(2);
    lane.save("disc-after");
}

/// What a drain scene drains.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Drained {
    /// An enemy power plant, drained by a local drainer.
    Power,
    /// An enemy refinery (a resource destination), drained by a local
    /// drainer.
    Refinery,
    /// A local power plant, drained by an enemy drainer.
    Blackout,
}

/// A stock unit type with a draining weapon in some slot, and its kind.
fn drainer(w: &World) -> Option<(Kind, String)> {
    let r = w.rules();
    for (kind, names) in [
        (Kind::Vehicle, &r.vehicles),
        (Kind::Aircraft, &r.aircraft),
        (Kind::Infantry, &r.infantry),
    ] {
        for (i, t) in names.iter().enumerate() {
            let drains = r.unit(kind, i as u32).is_some_and(|u| {
                u.strength > 0
                    && u.combat
                        .slots
                        .iter()
                        .flatten()
                        .any(|&x| r.combat.weapon(x).drain)
            });
            if drains {
                return Some((kind, t.name.clone()));
            }
        }
    }
    None
}

/// A stock drainable building type of at most 4 × 4 cells: one with
/// output (a power plant) or a resource destination (a refinery).
fn drainable_building(w: &World, refinery: bool) -> Option<(String, (i32, i32))> {
    let r = w.rules();
    r.buildings
        .iter()
        .find(|b| {
            b.strength > 0
                && b.combat.drainable
                && b.foundation.0 <= 4
                && b.foundation.1 <= 4
                && if refinery {
                    b.raw.econ.resource_destination.unwrap_or(false)
                } else {
                    b.raw.econ.power.unwrap_or(0) > 0
                }
        })
        .map(|b| (b.name.clone(), b.foundation))
}

/// S-0608 claims 2, 5, 7 and S-0405 claim 5: a drainer clicked onto a
/// drainable building flies over it and links; a refinery's house pays the
/// drainer's house; a drained power plant blacks its house out.
fn drain(lane: &mut Lane, which: Drained) {
    let w = lane.world();
    let d = drainer(w);
    let b = drainable_building(w, which == Drained::Refinery);
    println!(
        "drainer type found: {}, drainable building type found: {}",
        d.is_some(),
        b.is_some()
    );
    let (Some((kind, dname)), Some((bname, size)), Some(enemy)) = (d, b, lane.enemy()) else {
        return;
    };
    let local = lane.local();
    let (owner, attacker) = match which {
        Drained::Blackout => (local, enemy),
        _ => (enemy, local),
    };
    let home = lane.home();
    let Some(top) = lane.area(CellXY::new(home.x + 10, home.y + 4), size, 2, |l| {
        l != land::WATER && l != land::ROCK
    }) else {
        println!("no free area for the building");
        return;
    };
    let b = lane.spawn(Kind::Building, &bname, owner, top);
    let at = lane.open_land(CellXY::new(top.x - 6, top.y));
    let d = lane.spawn(kind, &dname, attacker, at);
    lane.ticks(3);
    let output = |l: &Lane| l.world().houses[usize::from(owner.0)].econ.power.output;
    let credits = |l: &Lane, h: HouseId| l.world().houses[usize::from(h.0)].credits;
    let out0 = output(lane);
    lane.look_at(CellXY::new(top.x + size.0 / 2, top.y + size.1 / 2));
    let cursor = targeting::cursor(
        lane.world(),
        attacker,
        &[d],
        ClickOn::Object(b),
        Modifiers::default(),
    );
    let n = lane.order(attacker, d, ClickOn::Object(b));
    println!("{which:?}: click on the building shows {cursor:?}; {n} orders");
    lane.save(&format!("drain-{}-before", key(which)));
    let mut linked = until(lane, 2000, |w| glasnost_sim::drain::being_drained(w, b));
    if linked.is_none() && lane.world().obj(d).is_some_and(|o| o.target.is_none()) {
        // The attempt came from outside the foundation (claim 2): the
        // drainer lost its target; a person clicks again.
        lane.save(&format!("drain-{}-first-attempt", key(which)));
        let n = lane.order(attacker, d, ClickOn::Object(b));
        println!("no link from the first click; a second click: {n} orders");
        linked = until(lane, 2000, |w| glasnost_sim::drain::being_drained(w, b));
    }
    let Some(t) = linked else {
        let w = lane.world();
        let o = w.obj(d);
        let over = o.is_some_and(|o| {
            let bo = w.obj(b).unwrap();
            glasnost_sim::econ::placement::foundation_cells(w, bo.type_index, bo.pos.cell())
                .contains(&o.pos.cell())
        });
        println!(
            "no link within 2 000 ticks: drainer alive {}, over the foundation {over}, \
             has a target {}, has a destination {}",
            o.is_some(),
            o.is_some_and(|o| o.target.is_some()),
            o.is_some_and(|o| o.dest.is_some())
        );
        lane.save(&format!("drain-{}-no-link", key(which)));
        return;
    };
    println!("linked after {t} ticks");
    let c0 = (credits(lane, owner), credits(lane, attacker));
    lane.ticks(30);
    lane.save(&format!("drain-{}", key(which)));
    lane.ticks(270);
    let c1 = (credits(lane, owner), credits(lane, attacker));
    let out1 = output(lane);
    println!(
        "after 300 ticks: still linked {}; the building's house's credits fell {}, \
         the drainer's house's rose {}; the building's house's output was above 0 {}, is 0 now {}",
        glasnost_sim::drain::being_drained(lane.world(), b),
        c1.0 < c0.0,
        c1.1 > c0.1,
        out0 > 0,
        out1 == 0
    );
    if which == Drained::Blackout {
        lane.save("drain-blackout-later");
    }
}

/// The genetic mutation superweapon type and a building type that gives
/// it, by the rules' kind and the buildings' superweapon fields.
fn mutator_types(w: &World) -> Option<(u32, String)> {
    use glasnost_sim::superweapons::SwKind;
    let r = w.rules();
    let sw = r
        .superweapons
        .types
        .iter()
        .position(|t| t.kind == Some(SwKind::Genetic))? as u32;
    let b = r
        .superweapons
        .building_sw
        .iter()
        .position(|f| f.iter().flatten().any(|&i| i == sw))?;
    Some((sw, r.buildings.get(b)?.name.clone()))
}

/// S-0611 claims 2 to 12: the local house's mutator, charged, fired at a
/// group of enemy human infantry; the screens show the launch, the
/// conversion animations and the new units.
fn mutator(lane: &mut Lane) {
    let w = lane.world();
    let types = mutator_types(w);
    let inf = human_infantry(w);
    println!(
        "mutator type and giving building found: {}, human infantry found: {}, explosion mode: {}",
        types.is_some(),
        inf.is_some(),
        w.rules().superweapons.mutation.explosion
    );
    let (Some((sw, building)), Some(name), Some(enemy)) = (types, inf, lane.enemy()) else {
        return;
    };
    let local = lane.local();
    // The lane's own copy: a short charge, and a building that needs and
    // draws no power (the lane's house has no power plant).
    let bi = w.rules().index_of(Kind::Building, &building);
    patch_rules(lane, |r| {
        let t = &mut r.superweapons.types[sw as usize];
        t.charge = 30;
        t.powered = false;
        if let Some(b) = bi.and_then(|i| r.econ.power.buildings.get_mut(i as usize)) {
            b.needs_power = false;
            b.drain = 0;
        }
    });
    let home = lane.home();
    let Some(top) = lane.area(CellXY::new(home.x - 8, home.y + 4), (4, 4), 1, |l| {
        l != land::WATER && l != land::ROCK
    }) else {
        println!("no free area for the building");
        return;
    };
    lane.spawn(Kind::Building, &building, local, top);
    let ready = until(lane, 600, move |w| {
        glasnost_sim::superweapons::charge::get(w, local, sw).is_some_and(|r| r.ready)
    });
    println!("the record is ready after {ready:?} ticks");
    if ready.is_none() {
        let w = lane.world();
        let b = w.objects.values().find(|o| {
            o.kind == Kind::Building
                && o.owner == Some(local)
                && w.rules().buildings[o.type_index as usize].name == building
        });
        println!(
            "record: {:?}; building on the map {}, health {:?}",
            glasnost_sim::superweapons::charge::get(w, local, sw),
            b.is_some_and(|o| o.on_map),
            b.map(|o| o.health)
        );
    }
    let Some(_) = ready else { return };
    // The group, well away from the local base.
    let at = lane.open_land(CellXY::new(home.x + 12, home.y + 12));
    let mut group = Vec::new();
    for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1), (-1, 0)] {
        let c = CellXY::new(at.x + dx, at.y + dy);
        group.push(lane.spawn(Kind::Infantry, &name, enemy, c));
    }
    lane.look_at(at);
    lane.ticks(3);
    let mine = |l: &Lane| {
        l.world()
            .objects
            .values()
            .filter(|o| o.on_map && o.kind == Kind::Infantry && o.owner == Some(local))
            .count()
    };
    let before = mine(lane);
    let ok = glasnost_sim::orders::issue(
        lane.world_mut(),
        local,
        glasnost_sim::orders::Order::FireSuperweapon { sw, cell: at },
    );
    println!("fire order taken: {ok}");
    lane.ticks(2);
    let gone = group
        .iter()
        .filter(|&&id| lane.world().obj(id).is_none())
        .count();
    println!(
        "launch: {gone} of {} converted at once, infantry-making animations {}",
        group.len(),
        lane.world().mutation.makers.len()
    );
    lane.save("mutator-launch");
    lane.ticks(6);
    println!(
        "conversion animations playing: {}",
        lane.world().mutation.makers.len()
    );
    lane.save("mutator-conversion");
    let done = until(lane, 600, |w| w.mutation.makers.is_empty());
    lane.ticks(2);
    println!(
        "animations ended after {done:?} more ticks; the local house's infantry rose by {}",
        mine(lane) as i64 - before as i64
    );
    lane.save("mutator-new-infantry");
}

/// The psychic dominator superweapon type and a building type that gives
/// it, by the rules' kind and the buildings' superweapon fields.
fn dominator_types(w: &World) -> Option<(u32, String)> {
    use glasnost_sim::superweapons::SwKind;
    let r = w.rules();
    let sw = r
        .superweapons
        .types
        .iter()
        .position(|t| t.kind == Some(SwKind::Dominator))? as u32;
    let b = r
        .superweapons
        .building_sw
        .iter()
        .position(|f| f.iter().flatten().any(|&i| i == sw))?;
    Some((sw, r.buildings.get(b)?.name.clone()))
}

/// S-0612 claims 1 to 14 and S-0104 (the dominator lighting): the local
/// house's dominator, charged, fired at a group of enemy vehicles and
/// infantry; the screens show the launch, the dominator lighting and the
/// taken-over units.
fn dominator(lane: &mut Lane) {
    let w = lane.world();
    let types = dominator_types(w);
    let inf = human_infantry(w);
    let veh = plain_vehicle(w);
    println!(
        "dominator type and giving building found: {}, infantry found: {}, vehicle found: {}",
        types.is_some(),
        inf.is_some(),
        veh.is_some()
    );
    let (Some((sw, building)), Some(inf), Some(veh), Some(enemy)) = (types, inf, veh, lane.enemy())
    else {
        return;
    };
    let local = lane.local();
    // The lane's own copy: a short charge, and a building that needs and
    // draws no power (the lane's house has no power plant).
    let bi = w.rules().index_of(Kind::Building, &building);
    patch_rules(lane, |r| {
        let t = &mut r.superweapons.types[sw as usize];
        t.charge = 30;
        t.powered = false;
        if let Some(b) = bi.and_then(|i| r.econ.power.buildings.get_mut(i as usize)) {
            b.needs_power = false;
            b.drain = 0;
        }
    });
    let home = lane.home();
    let Some(top) = lane.area(CellXY::new(home.x - 8, home.y + 4), (4, 4), 1, |l| {
        l != land::WATER && l != land::ROCK
    }) else {
        println!("no free area for the building");
        return;
    };
    lane.spawn(Kind::Building, &building, local, top);
    let ready = until(lane, 600, move |w| {
        glasnost_sim::superweapons::charge::get(w, local, sw).is_some_and(|r| r.ready)
    });
    println!("the record is ready after {ready:?} ticks");
    let Some(_) = ready else { return };
    // The group, well away from the local base.
    let at = lane.open_land(CellXY::new(home.x + 12, home.y + 12));
    let mut group = Vec::new();
    for (dx, dy) in [(0, 0), (1, 0), (-1, 0)] {
        let c = CellXY::new(at.x + dx, at.y + dy);
        group.push(lane.spawn(Kind::Infantry, &inf, enemy, c));
    }
    for (dx, dy) in [(0, 1), (-1, -1)] {
        let c = CellXY::new(at.x + dx, at.y + dy);
        group.push(lane.spawn(Kind::Vehicle, &veh, enemy, c));
    }
    lane.look_at(at);
    lane.ticks(3);
    let light = |l: &Lane| {
        let g = &l.world().lighting;
        format!(
            "ambient {} target {} choice {:?}",
            g.current, g.target, g.choice
        )
    };
    println!("before the launch: {}", light(lane));
    {
        let w = lane.world();
        let reach = glasnost_sim::domination::swept(w, at);
        println!(
            "capture range {}, cells swept {}",
            w.rules().superweapons.domination.range,
            reach.len()
        );
        for &id in &group {
            let Some(o) = w.obj(id) else { continue };
            let c = o.pos.cell();
            println!(
                "  {:?} {} at {:?}: in the sweep {}, dominable {}",
                o.kind,
                o.type_index,
                (c.x - at.x, c.y - at.y),
                reach.iter().any(|&(rc, _)| rc == c),
                glasnost_sim::mind::dominable(w, id)
            );
        }
    }
    let ok = glasnost_sim::orders::issue(
        lane.world_mut(),
        local,
        glasnost_sim::orders::Order::FireSuperweapon { sw, cell: at },
    );
    println!("fire order taken: {ok}");
    lane.ticks(2);
    println!(
        "launch: domination in force {}, {}",
        glasnost_sim::domination::in_force(lane.world()),
        light(lane)
    );
    lane.save("dominator-launch");
    let swept = until(lane, 600, |w| {
        w.domination
            .current
            .as_ref()
            .is_none_or(|d| !matches!(d.phase, glasnost_sim::domination::Phase::LeadIn { .. }))
    });
    lane.ticks(2);
    println!("the sweep after {swept:?} more ticks; {}", light(lane));
    lane.save("dominator-lighting");
    for &id in &group {
        let w = lane.world();
        if let Some(o) = w.obj(id) {
            let c = o.pos.cell();
            println!(
                "  after the sweep: {:?} at {:?}, owner {:?}, layer {:?}, health {}",
                o.kind,
                (c.x - at.x, c.y - at.y),
                o.owner,
                o.layer,
                o.health
            );
        }
    }
    let over = until(lane, 2000, |w| w.domination.current.is_none());
    lane.ticks(2);
    let taken = group
        .iter()
        .filter(|&&id| lane.world().obj(id).is_some_and(|o| o.owner == Some(local)))
        .count();
    let alive = group
        .iter()
        .filter(|&&id| lane.world().obj(id).is_some())
        .count();
    println!(
        "over after {over:?} more ticks; {taken} of {} taken over, {alive} alive; {}",
        group.len(),
        light(lane)
    );
    lane.save("dominator-taken");
}

/// An armed vehicle type (neither a harvester, a transport nor a gunner)
/// with a slot whose projectile `fits`: its name and that slot.
fn shooter(
    w: &World,
    fits: impl Fn(&glasnost_sim::combat::data::ProjectileRules) -> bool,
) -> Option<(String, usize)> {
    let r = w.rules();
    (0..r.vehicles.len()).find_map(|i| {
        let u = r.unit(Kind::Vehicle, i as u32)?;
        if u.harvester || u.strength <= 0 || u.passengers > 0 || u.combat.gunner {
            return None;
        }
        let s = u.combat.slots.iter().position(|x| {
            x.and_then(|x| r.combat.weapon(x).projectile)
                .is_some_and(|p| fits(&r.combat.projectiles[p as usize]))
        })?;
        Some((r.vehicles[i].name.clone(), s))
    })
}

/// The live view's projectile counts: bodies drawn in the last frame and
/// running target-line timers.
fn projectile_counts(lane: &Lane) -> (usize, usize) {
    let s = lane.c.session.as_ref().unwrap();
    s.loaded
        .live
        .as_ref()
        .expect("a live game")
        .projectile_counts()
}

/// The arcing sprite projectile scene: a vehicle type whose weapon fires
/// one, with the target-laser flag (given in the lane's copy of the rules
/// when the stock rules lack it).
fn sprite_shots(lane: &mut Lane) {
    let found = shooter(lane.world(), |p| !p.voxel && !p.inviso && p.arcing);
    shots(lane, "shot-sprite", found, true);
}

/// The voxel projectile scene: the sprite scene's vehicle type, its
/// weapon's projectile replaced in the lane's copy of the rules by the
/// first projectile type that is drawn as a voxel model and hits the
/// ground.
fn voxel_shots(lane: &mut Lane) {
    let w = lane.world();
    let r = w.rules();
    let pv = r
        .combat
        .projectiles
        .iter()
        .position(|p| p.voxel && !p.inviso && p.ag);
    let found = shooter(w, |p| !p.voxel && !p.inviso && p.arcing);
    println!("shot-voxel: voxel projectile type found: {}", pv.is_some());
    let (Some(pv), Some((vname, slot))) = (pv, found) else {
        return;
    };
    let ti = r.index_of(Kind::Vehicle, &vname).expect("the type");
    let Some(wid) = r.unit(Kind::Vehicle, ti).and_then(|u| u.combat.slots[slot]) else {
        return;
    };
    patch_rules(lane, |r| {
        r.combat.weapons[wid as usize].projectile = Some(pv as u32);
    });
    shots(lane, "shot-voxel", Some((vname, slot)), false);
}

/// S-0306 claims 28 to 32 (projectile drawing) and S-0301 claim 24 (the
/// target line): the vehicle type `found` (with the slot it fires) is
/// clicked onto an enemy of its type within its weapon's range; frames are
/// saved while its projectiles fly. With `laser` the lane's copy of the
/// rules gives the type the target-laser flag, so that its shots also start
/// the target line (the local house is the viewer).
fn shots(lane: &mut Lane, key: &str, found: Option<(String, usize)>, laser: bool) {
    let w = lane.world();
    let enemy = lane.enemy();
    println!("{key}: vehicle type found: {}", found.is_some());
    let (Some((vname, slot)), Some(enemy)) = (found, enemy) else {
        return;
    };
    let ti = w.rules().index_of(Kind::Vehicle, &vname).expect("the type");
    let already = w.rules().vehicles[ti as usize].vehicle.target_laser;
    println!(
        "{key}: slot {slot}; the type has the target-laser flag in the stock rules: {already}"
    );
    if laser && !already {
        patch_rules(lane, |r| {
            r.vehicles[ti as usize].vehicle.target_laser = true
        });
    }
    // The enemy within the weapon's range: up to 5 cells east, at least 2.
    let range = {
        let r = lane.world().rules();
        r.unit(Kind::Vehicle, ti)
            .and_then(|u| u.combat.slots[slot])
            .map_or(256, |x| r.combat.weapon(x).range)
    };
    let dx = (range / 256).clamp(2, 5);
    let local = lane.local();
    let home = lane.home();
    let at = lane.open_land(CellXY::new(home.x + 8, home.y + 8));
    let a = lane.spawn(Kind::Vehicle, &vname, local, at);
    let t = lane.spawn(Kind::Vehicle, &vname, enemy, CellXY::new(at.x + dx, at.y));
    lane.ticks(2);
    lane.look_at(CellXY::new(at.x + 2, at.y));
    let n = lane.order(local, a, ClickOn::Object(t));
    println!("{key}: enemy {dx} cells east; click on it: {n} orders");
    let mut saved = 0;
    for _ in 0..400 {
        lane.ticks(1);
        let flying = lane
            .world()
            .projectiles
            .values()
            .filter(|p| p.firer == Some(a))
            .count();
        let (drawn, lines) = projectile_counts(lane);
        if flying > 0 {
            println!(
                "{key}: tick {}: projectiles of the firer {flying}, bodies drawn {drawn}, target-line timers {lines}",
                lane.world().tick
            );
            // Where each of the firer's projectiles is drawn, from the firer's draw point.
            let w = lane.world();
            let lift = |p: glasnost_sim::world::Pos| {
                glasnost_render::projection::project_lifted(p.x, p.y, p.z)
            };
            if let Some(o) = w.obj(a) {
                let base = lift(o.pos);
                for p in w.projectiles.values().filter(|p| p.firer == Some(a)) {
                    let s = lift(p.pos);
                    println!(
                        "{key}: a projectile {} px right, {} px down of the firer, {} leptons up",
                        s.0 - base.0,
                        s.1 - base.1,
                        p.pos.z - o.pos.z
                    );
                }
            }
            lane.save(&format!("{key}-{saved}"));
            saved += 1;
            if saved == 3 {
                break;
            }
            lane.ticks(2);
        }
    }
    if saved == 0 {
        let w = lane.world();
        let r = w.rules();
        let wid = r.unit(Kind::Vehicle, ti).and_then(|u| u.combat.slots[slot]);
        let wr = wid.map(|x| r.combat.weapon(x));
        let p = wr
            .and_then(|x| x.projectile)
            .map(|p| &r.combat.projectiles[p as usize]);
        println!(
            "{key}: no frame with a projectile of the firer in flight; weapon range {:?}, minimum {:?}; projectile against ground {:?}, against air {:?}; firer target {:?}; enemy alive {}",
            wr.map(|x| x.range),
            wr.map(|x| x.min_range),
            p.map(|p| p.ag),
            p.map(|p| p.aa),
            w.obj(a).map(|o| o.target.is_some()),
            w.obj(t).is_some()
        );
    }
}

/// The building types whose weapons in slots 0 and 1 all have projectiles
/// that are anti-air and not anti-ground (at least one weapon), with the
/// range of slot 0.
fn aa_only_buildings(w: &World) -> Vec<(usize, i32)> {
    let r = w.rules();
    (0..r.buildings.len())
        .filter_map(|i| {
            let b = &r.buildings[i];
            if b.strength <= 0 {
                return None;
            }
            let ws: Vec<_> = b.combat.slots[..2].iter().flatten().collect();
            let aa_only = !ws.is_empty()
                && ws.iter().all(|&&x| {
                    r.combat.weapon(x).projectile.is_some_and(|p| {
                        r.combat.projectiles[p as usize].aa && !r.combat.projectiles[p as usize].ag
                    })
                });
            let range = b.combat.slots[0].map(|x| r.combat.weapon(x).range)?;
            aa_only.then_some((i, range))
        })
        .collect()
}

/// An armed aircraft type whose slot-0 projectile is anti-ground.
fn ground_attack_aircraft(w: &World) -> Option<String> {
    let r = w.rules();
    (0..r.aircraft.len())
        .find(|&i| {
            r.unit(Kind::Aircraft, i as u32).is_some_and(|u| {
                u.strength > 0
                    && u.combat.slots[0]
                        .and_then(|x| r.combat.weapon(x).projectile)
                        .is_some_and(|p| r.combat.projectiles[p as usize].ag)
            })
        })
        .map(|i| r.aircraft[i].name.clone())
}

/// One attack on building `b` for 600 ticks: per attacker, the ticks the
/// building held it as its target, the building's shots at it (new
/// projectiles of the building aimed at it), and the shots at it while it
/// was airborne (S-0307 Terms).
fn watch_attack(lane: &mut Lane, key: &str, b: ObjectId, attackers: &[(&str, ObjectId)]) {
    let mut seen = std::collections::BTreeSet::new();
    let mut held = vec![0usize; attackers.len()];
    let mut shots = vec![0usize; attackers.len()];
    let mut air_shots = vec![0usize; attackers.len()];
    let mut first = vec![None; attackers.len()];
    // The firing check's answers (with the range test) while held, and the
    // ticks the attacker was airborne then (S-0307 Terms).
    let mut answers = vec![std::collections::BTreeMap::<String, usize>::new(); attackers.len()];
    let mut held_air = vec![0usize; attackers.len()];
    let h0 = lane.permille(b);
    let rearm = |w: &World| w.obj(b).map(|o| o.combat.rearm_until);
    let mut last = rearm(lane.world());
    let mut restarts = 0;
    for _ in 0..600 {
        lane.ticks(1);
        let w = lane.world();
        let now = rearm(w);
        if now.is_some() && now != last {
            restarts += 1;
        }
        last = now;
        let target = w.obj(b).and_then(|o| o.target);
        for (k, (_, a)) in attackers.iter().enumerate() {
            if target == Some(glasnost_sim::world::Target::Object(*a)) {
                held[k] += 1;
                first[k].get_or_insert(w.tick);
                let answer = glasnost_sim::combat::check::check(w, b, target, 0, true);
                *answers[k].entry(format!("{answer:?}")).or_default() += 1;
                if glasnost_sim::combat::weapon::airborne_by_height(w, *a) {
                    held_air[k] += 1;
                }
            }
        }
        for p in w.projectiles.values().filter(|p| p.firer == Some(b)) {
            if !seen.insert(p.id) {
                continue;
            }
            for (k, (_, a)) in attackers.iter().enumerate() {
                if p.target == Some(glasnost_sim::world::Target::Object(*a)) {
                    shots[k] += 1;
                    if glasnost_sim::combat::weapon::airborne_by_height(w, *a) {
                        air_shots[k] += 1;
                    }
                }
            }
        }
        if w.obj(b).is_none() {
            break;
        }
    }
    let alive = lane.world().obj(b).is_some();
    let h1 = if alive { lane.permille(b) } else { 0 };
    println!(
        "{key}: building health {h0} -> {h1} per mille, alive {alive}, building projectiles {}, rearm restarts {restarts}",
        seen.len()
    );
    for (k, (what, a)) in attackers.iter().enumerate() {
        println!(
            "{key}: {what}: held as target {} ticks (first tick {:?}; airborne {} of them; check answers {:?}), shots at it {}, of them while it was airborne {}, alive at the end {}",
            held[k],
            first[k],
            held_air[k],
            answers[k],
            shots[k],
            air_shots[k],
            lane.world().obj(*a).is_some()
        );
    }
}

/// S-0306 claim 23 and S-0309 claim 18 on stock rules (M8 step A3 intake
/// 5, play note 4): each anti-air-only building type, of the local house and
/// of an enemy computer house, attacked by a ground vehicle and an infantry
/// unit of the other house from inside its range; then, in another place, a
/// copy attacked by an aircraft. Counts only.
fn aa_only(lane: &mut Lane) {
    let found = aa_only_buildings(lane.world());
    println!("aa-only: anti-air-only building types: {}", found.len());
    let (Some(veh), Some(inf), air) = (
        plain_vehicle(lane.world()),
        human_infantry(lane.world()),
        ground_attack_aircraft(lane.world()),
    ) else {
        println!("aa-only: no plain vehicle or human infantry type");
        return;
    };
    println!("aa-only: aircraft attacker type found: {}", air.is_some());
    for (n, &(bi, range)) in found.iter().enumerate() {
        {
            let r = lane.world().rules();
            let b = &r.buildings[bi];
            println!(
                "aa-only: building {n}: land-targeting mode 1: {}; SAM flag: {}; naval-targeting mode gives none on water: {}; retaliate flag: {}",
                b.combat.land_targeting == 1,
                b.building.sam,
                matches!(b.combat.naval_targeting, 2 | 6),
                b.combat.retaliate
            );
        }
        for computer in [false, true] {
            let mut l = Lane::start().expect("the install is set");
            let Some(enemy) = l.enemy() else {
                println!("aa-only: no enemy house");
                return;
            };
            let local = l.local();
            let (owner, other) = if computer {
                (enemy, local)
            } else {
                (local, enemy)
            };
            let bname = l.world().rules().buildings[bi].name.clone();
            // The lane's own copy: the building needs and draws no power
            // (the lane's houses have no power plant), and the computer run
            // makes the owner a test-only computer-controlled player house.
            patch_rules(&mut l, |r| {
                if let Some(p) = r.econ.power.buildings.get_mut(bi) {
                    p.needs_power = false;
                    p.drain = 0;
                }
            });
            l.world_mut().houses[usize::from(owner.0)].human = !computer;
            let home = l.home();
            let key = format!(
                "aa-only {n} {}",
                if computer { "computer" } else { "human" }
            );
            println!(
                "{key}: owner human {}",
                l.world().houses[usize::from(owner.0)].human
            );
            // Ground attackers inside the building's range.
            let at = l.open_land(CellXY::new(home.x + 12, home.y + 12));
            let b = l.spawn(Kind::Building, &bname, owner, at);
            let d = (range / 256 - 1).clamp(2, 6);
            let v = l.spawn(Kind::Vehicle, &veh, other, CellXY::new(at.x + d, at.y + 1));
            let i = l.spawn(Kind::Infantry, &inf, other, CellXY::new(at.x + 1, at.y + d));
            l.ticks(2);
            let nv = l.order(other, v, ClickOn::Object(b));
            let ni = l.order(other, i, ClickOn::Object(b));
            println!("{key}: ground attackers {d} cells off; orders {nv} and {ni}");
            watch_attack(
                &mut l,
                &format!("{key} ground"),
                b,
                &[("vehicle", v), ("infantry", i)],
            );
            // An aircraft attacker, in a fresh lane.
            let Some(air) = air.as_deref() else { continue };
            let mut l = Lane::start().expect("the install is set");
            patch_rules(&mut l, |r| {
                if let Some(p) = r.econ.power.buildings.get_mut(bi) {
                    p.needs_power = false;
                    p.drain = 0;
                }
            });
            l.world_mut().houses[usize::from(owner.0)].human = !computer;
            let at2 = l.open_land(CellXY::new(home.x - 12, home.y + 12));
            let b2 = l.spawn(Kind::Building, &bname, owner, at2);
            let a = l.spawn(Kind::Aircraft, air, other, CellXY::new(at2.x + 10, at2.y));
            l.ticks(2);
            let na = l.order(other, a, ClickOn::Object(b2));
            println!("{key}: aircraft 10 cells off; orders {na}");
            watch_attack(&mut l, &format!("{key} air"), b2, &[("aircraft", a)]);
        }
    }
}

// ---------------------------------------------------------------------------
// M8 step A4 round 3

/// Whether weapon `x`'s projectile is anti-air and whether it is
/// anti-ground.
fn aa_ag(w: &World, x: u32) -> (bool, bool) {
    let r = w.rules();
    r.combat.weapon(x).projectile.map_or((false, false), |p| {
        let p = &r.combat.projectiles[p as usize];
        (p.aa, p.ag)
    })
}

/// The types with an anti-air weapon in slot 0 or 1, or, for a gunner
/// type, in any slot (its weapon number picks the slot, S-0308 claim 6).
fn aa_types(w: &World) -> Vec<(Kind, usize)> {
    let r = w.rules();
    let mut out = Vec::new();
    for i in 0..r.buildings.len() {
        let b = &r.buildings[i];
        if b.strength > 0 && b.combat.slots[..2].iter().flatten().any(|&x| aa_ag(w, x).0) {
            out.push((Kind::Building, i));
        }
    }
    for kind in [Kind::Vehicle, Kind::Infantry] {
        for i in 0..type_count(w, kind) {
            let Some(u) = r.unit(kind, i as u32) else {
                continue;
            };
            let n = if u.combat.gunner {
                u.combat.slots.len()
            } else {
                2
            };
            if u.strength > 0 && u.combat.slots[..n].iter().flatten().any(|&x| aa_ag(w, x).0) {
                out.push((kind, i));
            }
        }
    }
    out
}

/// A landable aircraft type placed on the ground (not visiting) at `at`,
/// for the flight across a searcher.
fn crossing_aircraft(lane: &mut Lane, house: HouseId, at: CellXY) -> Option<ObjectId> {
    let n = lane.world().rules().aircraft.len();
    for i in 0..n {
        let r = lane.world().rules();
        if r.unit(Kind::Aircraft, i as u32)
            .is_none_or(|u| u.strength <= 0)
        {
            continue;
        }
        let name = r.aircraft[i].name.clone();
        let id = lane.spawn(Kind::Aircraft, &name, house, at);
        let ok = lane
            .world()
            .obj(id)
            .and_then(|o| o.air.as_ref())
            .is_some_and(|a| !a.visiting)
            && glasnost_sim::air::on_ground(lane.world(), id);
        if ok {
            return Some(id);
        }
        glasnost_sim::objects::remove(lane.world_mut(), id);
    }
    None
}

/// Note 15: every anti-air type of [`aa_types`], human and computer owner,
/// with an enemy aircraft flying across it on a move order. Counts and
/// ticks only.
fn aa_auto(lane: &mut Lane) {
    let found = aa_types(lane.world());
    println!("aa-auto: anti-air types: {}", found.len());
    // GLASNOST_LANE_AA_INDEX=n plays only the n-th type.
    let pick: Option<usize> = std::env::var("GLASNOST_LANE_AA_INDEX")
        .ok()
        .and_then(|x| x.parse().ok());
    for (n, &(kind, ti)) in found.iter().enumerate() {
        if pick.is_some_and(|p| p != n) {
            continue;
        }
        for computer in [false, true] {
            let mut l = Lane::start().expect("the install is set");
            let Some(enemy) = l.enemy() else { return };
            let local = l.local();
            let (owner, other) = if computer {
                (enemy, local)
            } else {
                (local, enemy)
            };
            let name = type_name_of(l.world(), kind, ti);
            if kind == Kind::Building {
                // The lane's own copy: the building needs and draws no power.
                patch_rules(&mut l, |r| {
                    if let Some(p) = r.econ.power.buildings.get_mut(ti) {
                        p.needs_power = false;
                        p.drain = 0;
                    }
                });
            }
            l.world_mut().houses[usize::from(owner.0)].human = !computer;
            let home = l.home();
            let key = format!(
                "aa-auto {n} {:?} {}",
                kind,
                if computer { "computer" } else { "human" }
            );
            let at = l.open_land(CellXY::new(home.x + 14, home.y + 14));
            let s = l.spawn(kind, &name, owner, at);
            let west = l.open_land(CellXY::new(at.x - 16, at.y));
            let Some(a) = crossing_aircraft(&mut l, other, west) else {
                println!("{key}: no aircraft type");
                return;
            };
            let target = Some(glasnost_sim::world::Target::Object(a));
            {
                let w = l.world();
                let tc = if kind == Kind::Building {
                    w.rules().buildings[ti].combat.clone()
                } else {
                    w.rules()
                        .unit(kind, ti as u32)
                        .map(|u| u.combat.clone())
                        .unwrap_or_default()
                };
                let wn = glasnost_sim::combat::weapon::weapon_number(w, s);
                let (slot, wp) = glasnost_sim::combat::weapon::would_use(w, s, target);
                let c = glasnost_sim::combat::acquire::classes(w, s);
                println!(
                    "{key}: gunner {} turret count {} switching {} weapon number {wn}; slot 0 aa/ag {:?}, slot 1 aa/ag {:?}; against the aircraft slot {slot} aa/ag {:?}; search classes aircraft {} ground {}; passive {}; guard range {}; scan mode {:?}",
                    tc.gunner,
                    tc.turret_count,
                    glasnost_sim::combat::weapon::switching(w, s),
                    tc.slots[0].map(|x| aa_ag(w, x)),
                    tc.slots[1].map(|x| aa_ag(w, x)),
                    wp.map(|x| aa_ag(w, x)),
                    c.aircraft,
                    c.infantry || c.vehicles || c.buildings,
                    glasnost_sim::combat::acquire::may_search_passively(w, s),
                    w.rules().unit(kind, ti as u32).map_or(0, |u| u.guard_range),
                    tc.scan_mode,
                );
            }
            l.ticks(2);
            let east = CellXY::new(at.x + 16, at.y);
            let na = l.order(other, a, ClickOn::Cell(east));
            let (mut first_range, mut first_target, mut first_shot) = (None, None, None);
            let mut first_hit = None;
            let mut in_range_beh = std::collections::BTreeMap::<String, usize>::new();
            let mut last_timer = l.world().obj(s).map_or(0, |o| o.targeting_timer);
            let mut shots = std::collections::BTreeMap::<&str, usize>::new();
            let mut last = l.world().obj(s).map_or(0, |o| o.last_fired);
            let full = l.world().obj(a).map_or(0, |o| o.health);
            let (mut in_range_ticks, mut tracked_ticks, mut flight_ticks) = (0, 0, 0);
            for _ in 0..700 {
                l.ticks(1);
                let w = l.world();
                if w.obj(a).is_none() || w.obj(s).is_none() {
                    break;
                }
                let flying = !glasnost_sim::air::on_ground(w, a);
                flight_ticks += usize::from(flying);
                tracked_ticks += usize::from(glasnost_sim::combat::flying::tracked(w, a));
                let (_, wp) = glasnost_sim::combat::weapon::would_use(w, s, target);
                let inr = flying && glasnost_sim::combat::range::in_range(w, s, wp, target, None);
                if inr {
                    in_range_ticks += 1;
                    first_range.get_or_insert(w.tick);
                    // The searcher's behaviour and target kind while the
                    // aircraft is in range (only behaviours that search
                    // passively look at it, S-0215 claim 33).
                    let o = w.obj(s).expect("checked above");
                    let what = match o.target {
                        None => "none",
                        x if x == target => "the aircraft",
                        _ => "another",
                    };
                    // With no target: the firing check without range and
                    // the versus test of S-0309 table C, and whether a
                    // search restarted the targeting timer this tick.
                    let why = if o.target.is_none() {
                        let (slot, wp) = glasnost_sim::combat::weapon::would_use(w, s, target);
                        let ans = glasnost_sim::combat::check::check(w, s, target, slot, false);
                        let vs = wp.and_then(|x| {
                            target.and_then(|t| glasnost_sim::combat::weapon::versus(w, x, t))
                        });
                        let vs_ok =
                            vs.is_some_and(|v| v.gt(glasnost_sim::combat::acquire::tau_a()));
                        let expected = w.obj(a).map_or(0, |x| x.combat.expected_health);
                        format!(
                            "; slot {slot} check {ans:?} versus above τ {vs_ok} searched {} expected health above 0 {}",
                            o.targeting_timer != last_timer,
                            expected > 0
                        )
                    } else {
                        String::new()
                    };
                    *in_range_beh
                        .entry(format!("{:?} target {what}{why}", o.beh.current))
                        .or_insert(0) += 1;
                }
                last_timer = w.obj(s).map_or(0, |o| o.targeting_timer);
                if w.obj(s).and_then(|o| o.target) == target {
                    first_target.get_or_insert(w.tick);
                }
                // A shot at the aircraft: the searcher's tick of the last
                // shot (S-0206 claim 9) moved while it held the aircraft.
                if let Some(o) = w.obj(s) {
                    if o.last_fired != last {
                        // Every shot, by what the searcher holds after it. A
                        // distributed-fire type outside attack drops the
                        // target it fired at onto its list (S-0309 claim 20).
                        let listed = o.combat.fired_at.contains(&a);
                        let what = match o.target {
                            x if x == target => "the aircraft",
                            None if listed => "the aircraft, dropped onto the list",
                            None => "no target",
                            _ => "another target",
                        };
                        *shots.entry(what).or_insert(0) += 1;
                        if o.target == target || listed {
                            first_shot.get_or_insert(w.tick);
                        }
                    }
                    last = o.last_fired;
                }
                if w.obj(a).is_some_and(|o| o.health < full) {
                    first_hit.get_or_insert(w.tick);
                }
            }
            let alive = l.world().obj(a).is_some();
            let delta = |x: Option<i32>| x.zip(first_range).map(|(x, r)| x - r);
            println!(
                "{key}: order {na}; aircraft in flight {flight_ticks} ticks, tracked {tracked_ticks}, in range in flight {in_range_ticks}; first in range {first_range:?}, first target {first_target:?} ({:?} after), first shot {first_shot:?} ({:?} after), first damage {first_hit:?} ({:?} after); aircraft alive {alive}; shots {shots:?}; in range: {in_range_beh:?}",
                delta(first_target),
                delta(first_shot),
                delta(first_hit),
            );
        }
    }
}

/// Animation types named by a corpse list (the general list or any
/// infantry type's own), lower case.
fn corpse_names(w: &World) -> std::collections::BTreeSet<String> {
    let r = w.rules();
    let mut out: std::collections::BTreeSet<String> = r
        .combat
        .general
        .dead_bodies
        .iter()
        .map(|x| x.to_ascii_lowercase())
        .collect();
    for i in 0..r.infantry.len() {
        if let Some(u) = r.unit(Kind::Infantry, i as u32) {
            for x in u.raw.dead_bodies.as_deref().unwrap_or_default() {
                out.insert(x.to_ascii_lowercase());
            }
        }
    }
    out
}

/// What stays on the map of the animations created after `before`, the
/// voxel debris pieces and the particle systems.
fn leftovers(lane: &Lane, before: &std::collections::BTreeSet<ObjectId>, what: &str) {
    let w = lane.world();
    let corpses = corpse_names(w);
    let (mut bouncing, mut corpse, mut endless, mut other, mut ended) = (0, 0, 0, 0, 0);
    let mut corpse_loops = std::collections::BTreeMap::<i32, usize>::new();
    for (id, a) in &w.anims.anims {
        if before.contains(id) {
            continue;
        }
        let ty = a.anim.and_then(|i| w.rules().animations.get(i as usize));
        let name = a.player.name_text().to_ascii_lowercase();
        if a.player.ended() {
            ended += 1;
        }
        if ty.is_some_and(|t| t.bouncing) {
            bouncing += 1;
        } else if corpses.contains(&name) {
            corpse += 1;
            *corpse_loops.entry(a.player.play.loop_count).or_default() += 1;
        } else if a.player.remaining_passes().is_none() {
            endless += 1;
        } else {
            other += 1;
        }
    }
    let (k, n) = census(w);
    println!(
        "{what}: new animations: bouncing {bouncing}, corpse {corpse} (by loop count {corpse_loops:?}), endless other {endless}, other {other}, ended but listed {ended}; voxel debris pieces {}; walkers {}; particle systems {} with {n} particles",
        w.pieces.len(),
        w.walkers.len(),
        k.iter().sum::<usize>()
    );
}

/// Note 13: enemy infantry killed by each death kind 1 to 7 and 10 and
/// enemy armed vehicles destroyed, then what stays on the map at +300 and
/// +1000 ticks.
fn debris_left(lane: &mut Lane) {
    let Some(inf) = human_infantry(lane.world()) else {
        println!("debris-left: no human infantry type");
        return;
    };
    let Some(enemy) = lane.enemy() else { return };
    let before: std::collections::BTreeSet<ObjectId> =
        lane.world().anims.anims.keys().copied().collect();
    let home = lane.home();
    let at = lane.open_land(CellXY::new(home.x + 10, home.y + 10));
    lane.look_at(at);
    let mut killed = 0;
    for (j, k) in [1, 2, 3, 4, 5, 6, 7, 10].into_iter().enumerate() {
        let Some(h) = warhead_of_kind(lane.world(), k) else {
            continue;
        };
        let id = lane.spawn(
            Kind::Infantry,
            &inf,
            enemy,
            CellXY::new(at.x + j as i32, at.y),
        );
        lane.ticks(2);
        kill_with(lane, id, h);
        killed += 1;
    }
    let vehicles: Vec<String> = {
        let r = lane.world().rules();
        (0..r.vehicles.len())
            .filter(|&i| {
                r.unit(Kind::Vehicle, i as u32).is_some_and(|u| {
                    u.strength > 0
                        && u.combat.slots[0].is_some()
                        && u.locomotor != glasnost_sim::rules::Locomotor::Ship
                })
            })
            .take(6)
            .map(|i| r.vehicles[i].name.clone())
            .collect()
    };
    let h = warhead_of_kind(lane.world(), 0).unwrap_or(0);
    for (j, v) in vehicles.iter().enumerate() {
        let id = lane.spawn(
            Kind::Vehicle,
            v,
            enemy,
            CellXY::new(at.x + 2 * j as i32, at.y + 4),
        );
        lane.ticks(2);
        kill_with(lane, id, h);
        killed += 1;
    }
    println!("debris-left: killed {killed} ({} vehicles)", vehicles.len());
    for t in (50..300).step_by(50) {
        lane.ticks(50);
        leftovers(lane, &before, &format!("debris-left +{t}"));
    }
    lane.ticks(50);
    leftovers(lane, &before, "debris-left +300");
    lane.save("debris-left-300");
    for t in (350..1000).step_by(50) {
        lane.ticks(50);
        leftovers(lane, &before, &format!("debris-left +{t}"));
    }
    lane.ticks(50);
    leftovers(lane, &before, "debris-left +1000");
    lane.save("debris-left-1000");
}

/// The first weapon with the electric-bolt flag that makes a bolt (no
/// laser flag, a projectile), by index.
fn bolt_weapon(w: &World) -> Option<u32> {
    let r = w.rules();
    r.combat
        .weapons
        .iter()
        .position(|x| x.electric_bolt && !x.laser && x.projectile.is_some())
        .map(|i| i as u32)
}

/// The slots of the type `i` of `kind`.
fn slots_of(
    w: &World,
    kind: Kind,
    i: usize,
) -> Option<[Option<glasnost_sim::combat::data::WeaponId>; glasnost_sim::combat::data::SLOTS]> {
    let r = w.rules();
    match kind {
        Kind::Building => r.buildings.get(i).map(|b| b.combat.slots),
        _ => r.unit(kind, i as u32).map(|u| u.combat.slots),
    }
}

/// The number of types of `kind` and the name of type `i`.
fn type_count(w: &World, kind: Kind) -> usize {
    let r = w.rules();
    match kind {
        Kind::Building => r.buildings.len(),
        Kind::Infantry => r.infantry.len(),
        _ => r.vehicles.len(),
    }
}

fn type_name_of(w: &World, kind: Kind, i: usize) -> String {
    let r = w.rules();
    match kind {
        Kind::Building => r.buildings[i].name.clone(),
        Kind::Infantry => r.infantry[i].name.clone(),
        _ => r.vehicles[i].name.clone(),
    }
}

/// An armed type of `kind` (a small, not wall building; a mobile type that
/// is neither a harvester nor a transport nor a gunner) and its first armed
/// slot, preferring one whose slot holds an electric-bolt weapon.
fn bolt_firer(w: &World, kind: Kind) -> Option<(usize, usize, bool)> {
    let r = w.rules();
    let fits = |i: usize| match kind {
        Kind::Building => {
            let b = &r.buildings[i];
            b.strength > 0
                && b.foundation.0 <= 2
                && b.foundation.1 <= 2
                && !b.raw.econ.wall.unwrap_or(false)
        }
        _ => r.unit(kind, i as u32).is_some_and(|u| {
            u.strength > 0 && !u.harvester && u.passengers == 0 && !u.combat.gunner
        }),
    };
    let is_bolt = |x: glasnost_sim::combat::data::WeaponId| {
        let wr = r.combat.weapon(x);
        wr.electric_bolt && !wr.laser && wr.projectile.is_some()
    };
    let n = type_count(w, kind);
    let found = (0..n).filter(|&i| fits(i)).find_map(|i| {
        let s = slots_of(w, kind, i)?;
        let k = s.iter().position(|x| x.is_some_and(is_bolt))?;
        Some((i, k, true))
    });
    found.or_else(|| {
        (0..n).filter(|&i| fits(i)).find_map(|i| {
            let s = slots_of(w, kind, i)?;
            let k = s
                .iter()
                .position(|x| x.is_some_and(|x| r.combat.weapon(x).projectile.is_some()))?;
            Some((i, k, false))
        })
    })
}

/// The systems in play of the game-wide spark type.
fn default_sparks(w: &World) -> usize {
    let Some(t) = w.rules().gas.default_spark else {
        return 0;
    };
    w.gas.systems.values().filter(|s| s.type_index == t).count()
}

/// S-0316 (electric bolts): a firer of `kind` whose weapon has the
/// electric-bolt flag fires at an enemy vehicle; frames while bolts are
/// shown, with counts.
fn bolts(lane: &mut Lane, kind: Kind) {
    let key = format!("bolt-{kind:?}").to_lowercase();
    let w = lane.world();
    let weapon = bolt_weapon(w);
    let enemy = lane.enemy();
    let target_type = plain_vehicle(w);
    let found = bolt_firer(w, kind);
    println!(
        "{key}: electric-bolt weapons {}; a type of this kind found: {}; with the flag in the stock rules: {}",
        lane.world()
            .rules()
            .combat
            .weapons
            .iter()
            .filter(|x| x.electric_bolt)
            .count(),
        found.is_some(),
        found.is_some_and(|f| f.2)
    );
    let (Some(weapon), Some((ti, slot, stock)), Some(enemy), Some(target_type)) =
        (weapon, found, enemy, target_type)
    else {
        return;
    };
    if !stock {
        println!(
            "{key}: patched: the type's slot {slot} gets the electric-bolt weapon (lane copy only)"
        );
        patch_rules(lane, |r| match kind {
            Kind::Building => r.buildings[ti].combat.slots[slot] = Some(weapon),
            Kind::Infantry => r.infantry[ti].combat.slots[slot] = Some(weapon),
            _ => r.vehicles[ti].combat.slots[slot] = Some(weapon),
        });
    }
    let name = type_name_of(lane.world(), kind, ti);
    let local = lane.local();
    let home = lane.home();
    let at = lane
        .area(CellXY::new(home.x + 8, home.y + 8), (2, 2), 3, |l| {
            l != land::WATER && l != land::ROCK
        })
        .unwrap_or_else(|| lane.open_land(CellXY::new(home.x + 8, home.y + 8)));
    let a = lane.spawn(kind, &name, local, at);
    let t = lane.spawn(
        Kind::Vehicle,
        &target_type,
        enemy,
        CellXY::new(at.x + 4, at.y),
    );
    if kind == Kind::Building {
        // Power for the building: the strongest power plant type, west of it.
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
            && let Some(p) = lane.area(CellXY::new(at.x - 5, at.y), psize, 2, |l| {
                l != land::WATER && l != land::ROCK
            })
        {
            lane.spawn(Kind::Building, &pname, local, p);
        }
    }
    lane.ticks(2);
    if kind == Kind::Building {
        println!(
            "{key}: building {name}; in operation {}",
            glasnost_sim::econ::power::in_operation(lane.world(), a)
        );
    }
    lane.look_at(CellXY::new(at.x + 2, at.y));
    let n = if kind == Kind::Building {
        lane.world_mut().obj_mut(a).unwrap().target = Some(glasnost_sim::world::Target::Object(t));
        1
    } else {
        lane.order(local, a, ClickOn::Object(t))
    };
    println!("{key}: enemy vehicle 4 cells east; orders {n}");
    let sparks0 = default_sparks(lane.world());
    let mut saved = 0;
    let mut most = (0, 0);
    for _ in 0..600 {
        lane.ticks(1);
        let (shown, attached) = live_bolts(lane);
        most = (most.0.max(shown), most.1.max(attached));
        if shown > 0 && saved < 4 {
            let pixels = lane
                .c
                .session
                .as_ref()
                .and_then(|s| s.loaded.live.as_ref())
                .map_or(0, |l| l.counts.bolt_pixels);
            println!(
                "{key}: tick {}: bolts shown {shown}, attached {attached}, bolt pixels {pixels}, spark systems of the game-wide type {}",
                lane.world().tick,
                default_sparks(lane.world())
            );
            lane.save(&format!("{key}-{saved}"));
            saved += 1;
            lane.ticks(3);
        }
        if saved >= 4 {
            break;
        }
    }
    let made = lane
        .c
        .session
        .as_ref()
        .and_then(|s| s.loaded.live.as_ref())
        .map_or(0, |l| l.counts.bolts);
    println!(
        "{key}: bolts made {made}; most shown at once {}, most attached {}; spark systems of the game-wide type before {sparks0}, now {}; frames {saved}",
        most.0,
        most.1,
        default_sparks(lane.world())
    );
}

/// The live view's bolts: shown and attached.
fn live_bolts(lane: &Lane) -> (usize, usize) {
    lane.c
        .session
        .as_ref()
        .and_then(|s| s.loaded.live.as_ref())
        .map_or((0, 0), |l| l.bolt_counts())
}

/// The screenshot key of a drain scene.
fn key(which: Drained) -> &'static str {
    match which {
        Drained::Power => "power",
        Drained::Refinery => "refinery",
        Drained::Blackout => "blackout",
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

    /// Starts a skirmish on the `map`-th offered map.
    fn start_on(map: usize) -> Option<Lane> {
        let dir = std::env::var_os("GLASNOST_GAME_DIR")?;
        let out = png::lane_out("m7b");
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

// ---------------------------------------------------------------------------
// M8 step A4 round 4 (the lead's play notes 16 to 18)

/// A slot state's short name.
fn slot_word(s: glasnost_sim::spawn::SlotState) -> &'static str {
    use glasnost_sim::spawn::SlotState as S;
    match s {
        S::Empty => "empty",
        S::Docked => "docked",
        S::Preparing => "preparing",
        S::OnRail => "on the rail",
        S::Attacking => "attacking",
        S::Returning => "returning",
        S::Reloading => "reloading",
    }
}

/// Note 16 (S-0506 claims 8, 12 and 13; S-0312 claim 16; S-0501 claim
/// 36): each spawner type whose spawned type is not a rocket type (a
/// carrier of aircraft), of the local house, clicked onto an enemy vehicle;
/// once a slot aircraft attacks or returns, the carrier is killed by a hit,
/// and every aircraft of the spawned type that was in play is followed:
/// its health, the tick it falls and the tick it leaves play. Counts and
/// ticks only.
fn carrier_death(lane: &mut Lane) {
    let spawners: Vec<((Kind, u32), glasnost_sim::spawn::SpawnRules)> = lane
        .world()
        .rules()
        .missiles
        .spawners
        .iter()
        .map(|(k, v)| (*k, *v))
        .collect();
    println!("carrier-death: spawner types {}", spawners.len());
    let Some(target_name) = plain_vehicle(lane.world()) else {
        println!("carrier-death: no plain vehicle type");
        return;
    };
    for (n, &((kind, ti), sr)) in spawners.iter().enumerate() {
        let (rocket, flagged, ship) = {
            let r = lane.world().rules();
            (
                r.missiles.set_of(sr.spawned).is_some(),
                r.unit(Kind::Aircraft, sr.spawned)
                    .is_some_and(|u| u.vehicle.missile_spawn),
                r.unit(kind, ti)
                    .is_some_and(|u| u.locomotor == glasnost_sim::rules::Locomotor::Ship),
            )
        };
        println!(
            "carrier-death {n}: kind {kind:?}, ship {ship}, spawn count {}, spawned type a rocket type {rocket}, spawned type with the missile-spawn flag {flagged}",
            sr.count
        );
        if rocket || kind == Kind::Building {
            continue;
        }
        // The first map (of `GLASNOST_LANE_MAPS`, default 8) with open
        // water near the local start, for a ship.
        let mut found = None;
        for map in 0..env_num("GLASNOST_LANE_MAPS", 8) {
            let l = Lane::start_on(map).expect("the install is set");
            let home = l.home();
            let at = if ship {
                l.area(home, (1, 1), 2, |x| x == land::WATER)
            } else {
                Some(l.open_land(CellXY::new(home.x + 10, home.y + 10)))
            };
            if let Some(at) = at {
                println!("carrier-death {n}: map {map}");
                found = Some((l, at));
                break;
            }
        }
        let Some((mut l, at)) = found else {
            println!("carrier-death {n}: no open water on the maps tried");
            continue;
        };
        let Some(enemy) = l.enemy() else { return };
        let local = l.local();
        let name = type_name_of(l.world(), kind, ti as usize);
        let car = l.spawn(kind, &name, local, at);
        let near = l.open_land(CellXY::new(at.x + 6, at.y));
        let victim = l.spawn(Kind::Vehicle, &target_name, enemy, near);
        // The victim lasts: the scene is about the carrier's death.
        if let Some(o) = l.world_mut().obj_mut(victim) {
            o.health = i32::MAX / 4;
        }
        l.look_at(at);
        l.ticks(2);
        let orders = l.order(local, car, ClickOn::Object(victim));
        let slots = |w: &World| {
            w.obj(car)
                .and_then(|o| o.spawn.as_ref())
                .map(|c| c.slots.clone())
                .unwrap_or_default()
        };
        let flying = |w: &World| {
            w.obj(car).and_then(|o| o.spawn.as_ref()).is_some_and(|c| {
                c.slots.iter().any(|s| {
                    matches!(
                        s.state,
                        glasnost_sim::spawn::SlotState::Attacking
                            | glasnost_sim::spawn::SlotState::Returning
                    )
                })
            })
        };
        let waited = until(&mut l, 1500, flying);
        let at_death: Vec<String> = slots(l.world())
            .iter()
            .map(|s| slot_word(s.state).to_string())
            .collect();
        // Every aircraft of the spawned type in play (slot objects or detached).
        let planes: Vec<ObjectId> = l
            .world()
            .objects
            .values()
            .filter(|o| o.on_map && o.kind == Kind::Aircraft && o.type_index == sr.spawned)
            .map(|o| o.id)
            .collect();
        let in_slot = planes
            .iter()
            .filter(|&&p| slots(l.world()).iter().any(|s| s.object == Some(p)))
            .count();
        println!(
            "carrier-death {n}: order {orders}; ticks until a slot attacks or returns {waited:?}; slots at the death {at_death:?}; aircraft of the type in play {}, in a slot {in_slot}",
            planes.len()
        );
        l.save(&format!("carrier-death-{n}-before"));
        let h = warhead_of_kind(l.world(), 0).unwrap_or(0);
        kill_with(&mut l, car, h);
        let t0 = l.world().tick;
        let mut fell: Vec<Option<i32>> = vec![None; planes.len()];
        let mut gone: Vec<Option<i32>> = vec![None; planes.len()];
        for _ in 0..1500 {
            l.ticks(1);
            let w = l.world();
            for (i, &p) in planes.iter().enumerate() {
                match w.obj(p) {
                    Some(o) if o.on_map => {
                        if o.health <= 0 {
                            fell[i].get_or_insert(w.tick - t0);
                        }
                    }
                    _ => {
                        gone[i].get_or_insert(w.tick - t0);
                    }
                }
            }
            if gone.iter().all(Option::is_some) {
                break;
            }
            if w.tick - t0 == 30 {
                l.save(&format!("carrier-death-{n}-after"));
            }
        }
        let w = l.world();
        let left: Vec<String> = planes
            .iter()
            .filter_map(|&p| w.obj(p))
            .map(|o| {
                format!(
                    "health {} height {} behaviour {:?} kamikaze {}",
                    o.health,
                    glasnost_sim::air::height(w, o.id),
                    o.beh.current,
                    glasnost_sim::spawn::kamikaze(w, o.id)
                )
            })
            .collect();
        println!(
            "carrier-death {n}: carrier in play {}; per aircraft, ticks after the hit to health 0 {fell:?}, to leaving play {gone:?}; still in play {left:?}",
            w.obj(car).is_some_and(|o| o.on_map)
        );
    }
}

/// The weapon names whose `TerrainFire` key reads as yes in the import
/// (S-0214 Import mapping; our engine does not import the key yet).
fn terrain_fire_weapons(lane: &Lane) -> std::collections::BTreeSet<String> {
    let s = lane.c.session.as_ref().unwrap();
    s.loaded
        .import
        .outcome
        .registries
        .get(&glasnost_convert::kinds::Kind::Weapons)
        .map(|v| {
            v.iter()
                .filter(|t| {
                    t.keys
                        .get(b"TerrainFire".as_slice())
                        .and_then(|k| k.value.as_deref())
                        .and_then(glasnost_ini::value::boolean)
                        .unwrap_or(false)
                })
                .map(|t| String::from_utf8_lossy(&t.name).to_lowercase())
                .collect()
        })
        .unwrap_or_default()
}

/// Note 17 (S-0214 Terms *clicked object*, *can be selected*, claim 6 and
/// table G; S-0516 claims 33, 35 to 37 and 39): each vehicle type that
/// disguises when still, of an enemy house, standing still until it takes a
/// disguise; every armed vehicle and infantry type of the local house, five
/// cells away, is asked what a plain click and a force-fire click on it
/// give, through the client's own choice of the clicked thing; then the
/// same while its fake blink runs. Counts only.
fn mirage_click(lane: &mut Lane) {
    let types: Vec<usize> = {
        let r = lane.world().rules();
        (0..r.vehicles.len())
            .filter(|&i| {
                r.unit(Kind::Vehicle, i as u32).is_some_and(|u| {
                    u.strength > 0
                        && u.combat.can_disguise
                        && u.combat.disguise_when_still
                        && !u.raw.perma_disguise.unwrap_or(false)
                })
            })
            .collect()
    };
    let terrain_fire = terrain_fire_weapons(lane);
    println!(
        "mirage-click: disguise-when-still vehicle types {}; general attack cursor on disguised units {}; weapons whose terrain-fire key reads yes {}",
        types.len(),
        lane.world().rules().cloak.attack_cursor_on_disguise,
        terrain_fire.len()
    );
    let Some(enemy) = lane.enemy() else { return };
    let local = lane.local();
    for (n, &ti) in types.iter().enumerate() {
        let name = lane.world().rules().vehicles[ti].name.clone();
        let home = lane.home();
        let at = lane.open_land(CellXY::new(home.x + 12, home.y + 12));
        let m = lane.spawn(Kind::Vehicle, &name, enemy, at);
        let took = until(lane, 600, |w| {
            w.obj(m).is_some_and(|o| o.cloak.disguise.is_some())
        });
        let what = lane.world().obj(m).and_then(|o| o.cloak.disguise).map(|d| {
            let kind = match d.what {
                Some(glasnost_sim::cloak::disguise::DisguiseAs::Terrain(_)) => "terrain object",
                Some(glasnost_sim::cloak::disguise::DisguiseAs::Overlay(_)) => "overlay",
                Some(_) => "unit",
                None => "no type",
            };
            format!("{kind}, house {:?}", d.house)
        });
        let w = lane.world();
        println!(
            "mirage-click {n}: ticks to its disguise {took:?}: {what:?}; appears disguised to the local house {}; selectable by it {}",
            glasnost_sim::cloak::appears_disguised(w, m, Some(local)),
            glasnost_sim::targeting::selectable_by(w, m, Some(local))
        );
        lane.look_at(at);
        lane.save(&format!("mirage-click-{n}-disguised"));
        mirage_actors(lane, n, "disguised", m, &terrain_fire);
        // The fake blink (S-0516 claim 39): it runs while the enemy sees
        // the real unit; set as a shot would set it.
        let blink_end = lane.world().tick + 60;
        if let Some(o) = lane.world_mut().obj_mut(m) {
            o.cloak.blink_until = blink_end;
        }
        let w = lane.world();
        println!(
            "mirage-click {n}: blinking {}, still disguised {}",
            glasnost_sim::cloak::disguise::blinking(w, m),
            w.obj(m).is_some_and(|o| o.cloak.disguise.is_some())
        );
        mirage_actors(lane, n, "blinking", m, &terrain_fire);
        // The control: the same unit without its disguise, at once (no
        // tick runs, so it does not take it again).
        if let Some(o) = lane.world_mut().obj_mut(m) {
            o.cloak.blink_until = 0;
            o.cloak.disguise = None;
        }
        mirage_actors(lane, n, "undisguised (control)", m, &terrain_fire);
        glasnost_sim::objects::remove(lane.world_mut(), m);
    }
}

/// Every armed vehicle and infantry type of the local house, five cells
/// from `m`, clicked onto it plain and with force-fire, the clicked thing
/// chosen as the client chooses it; the actions by count.
fn mirage_actors(
    lane: &mut Lane,
    n: usize,
    what: &str,
    m: ObjectId,
    terrain_fire: &std::collections::BTreeSet<String>,
) {
    let local = lane.local();
    let at = lane
        .world()
        .obj(m)
        .map_or(CellXY::new(0, 0), |o| o.pos.cell());
    let mut plain = std::collections::BTreeMap::<String, usize>::new();
    let mut force = std::collections::BTreeMap::<String, usize>::new();
    let mut tf = 0;
    let mut actors = 0;
    for kind in [Kind::Vehicle, Kind::Infantry] {
        for i in 0..type_count(lane.world(), kind) {
            let ok =
                lane.world().rules().unit(kind, i as u32).is_some_and(|u| {
                    u.strength > 0 && u.combat.slots[0].is_some() && !u.combat.gunner
                });
            if !ok {
                continue;
            }
            let name = type_name_of(lane.world(), kind, i);
            let a = lane.spawn(kind, &name, local, CellXY::new(at.x - 5, at.y));
            actors += 1;
            let on = lane.c.session.as_ref().unwrap().play.target(Some(m), at);
            let w = lane.world();
            let (_, wp) = glasnost_sim::combat::weapon::would_use(
                w,
                a,
                Some(glasnost_sim::world::Target::Object(m)),
            );
            if wp.is_some_and(|x| {
                terrain_fire.contains(&w.rules().combat.weapon(x).name.to_lowercase())
            }) {
                tf += 1;
            }
            let p = targeting::cursor(w, local, &[a], on, Modifiers::default());
            let f = targeting::cursor(
                w,
                local,
                &[a],
                on,
                Modifiers {
                    force_fire: true,
                    ..Modifiers::default()
                },
            );
            let key = |x: targeting::Action| {
                format!(
                    "{x:?} on {}",
                    if matches!(on, ClickOn::Object(_)) {
                        "the unit"
                    } else {
                        "its cell"
                    }
                )
            };
            *plain.entry(key(p)).or_insert(0) += 1;
            *force.entry(key(f)).or_insert(0) += 1;
            glasnost_sim::objects::remove(lane.world_mut(), a);
        }
    }
    println!(
        "mirage-click {n} {what}: actors {actors} (the weapon each would use is a terrain-fire weapon for {tf}); plain click {plain:?}; force-fire click {force:?}"
    );
}

/// Note 18 (S-0103 claims 8 and 10; S-0108 claims 2 and 11; S-0405
/// claims 8 and 9; S-0509 claim 8; S-0515 claim 2; Q-1772): the map's
/// buildings whose type needs an engineer and has slot animations, on the
/// first map (of `GLASNOST_LANE_MAPS`, default 8) that has one: the art's
/// terrain-palette flag; then one of them fresh, damaged, captured by the
/// local house's engineer through a click and captured by an enemy
/// engineer: whether it is drawn live, its running slots, and whether its
/// owner's colour definition is the non-player grey. Counts only.
fn derrick_look(_: &mut Lane) {
    let maps = env_num("GLASNOST_LANE_MAPS", 8);
    for map in 0..maps {
        let Some(mut l) = Lane::start_on(map) else {
            return;
        };
        l.ticks(3);
        let _ = l.c.frame_rgba();
        let found = derricks(&l);
        println!(
            "derrick-look map {map}: needs-engineer map buildings with running slot animations {}",
            found.len()
        );
        let Some(&b) = found.first() else { continue };
        let flags: std::collections::BTreeMap<Option<bool>, usize> =
            found.iter().fold(Default::default(), |mut m, &id| {
                *m.entry(terrain_palette(&l, id)).or_insert(0) += 1;
                m
            });
        println!("derrick-look map {map}: terrain-palette art flag over them {flags:?}");
        let cell = l.world().obj(b).map_or(CellXY::new(0, 0), |o| o.pos.cell());
        l.look_at(cell);
        derrick_state(&mut l, map, "fresh", b);
        l.save(&format!("derrick-{map}-fresh"));
        // Damaged, never captured.
        let full = l.world().obj(b).map_or(0, |o| o.health);
        if let Some(o) = l.world_mut().obj_mut(b) {
            o.health = full / 2;
        }
        l.ticks(20);
        derrick_state(&mut l, map, "damaged", b);
        l.save(&format!("derrick-{map}-damaged"));
        if let Some(o) = l.world_mut().obj_mut(b) {
            o.health = full;
        }
        // Captured by the local house, then by an enemy, each by a click.
        let local = l.local();
        let enemy = l.enemy();
        for (who, house) in [("local", Some(local)), ("enemy", enemy)] {
            let Some(house) = house else { continue };
            let Some(eng) = engineer(&l) else {
                println!("derrick-look: no engineer type");
                return;
            };
            let spot = l.open_land(CellXY::new(cell.x - 4, cell.y + 4));
            let e = l.spawn(Kind::Infantry, &eng, house, spot);
            l.ticks(2);
            let orders = l.order(house, e, ClickOn::Object(b));
            let took = until(&mut l, 1500, |w| {
                w.obj(b).is_some_and(|o| o.owner == Some(house))
            });
            l.ticks(20);
            println!(
                "derrick-look map {map} {who}: capture order {orders}, ticks to the ownership change {took:?}"
            );
            derrick_state(&mut l, map, &format!("captured by the {who} house"), b);
            l.save(&format!("derrick-{map}-{who}"));
        }
        return;
    }
}

/// The map's buildings whose type needs an engineer and that have running
/// slot animations (the live view's slot player).
fn derricks(l: &Lane) -> Vec<ObjectId> {
    let s = l.c.session.as_ref().unwrap();
    let live = s.loaded.live.as_ref().expect("a live game");
    let w = l.world();
    live.building_ids
        .values()
        .filter(|&&id| {
            w.obj(id).is_some_and(|o| {
                let tech = w
                    .rules()
                    .econ
                    .power
                    .buildings
                    .get(o.type_index as usize)
                    .is_some_and(|t| t.needs_engineer);
                o.on_map && tech && !live.slot_names(id).is_empty()
            })
        })
        .copied()
        .collect()
}

/// The `TerrainPalette` art flag of a building's type (S-0103 claim 8).
fn terrain_palette(l: &Lane, id: ObjectId) -> Option<bool> {
    let s = l.c.session.as_ref().unwrap();
    let live = s.loaded.live.as_ref()?;
    let o = l.world().obj(id)?;
    let idx = live.import_index(o.kind, o.type_index)?;
    let import = &s.loaded.import;
    let image = import
        .arts
        .get(&(glasnost_convert::kinds::Kind::Buildings, idx))?
        .image
        .clone();
    let f = import.art_file.as_ref()?;
    let doc = glasnost_ini::Document::parse(&f.bytes, Default::default(), &mut Vec::new());
    Some(
        doc.value(&image, b"TerrainPalette")
            .and_then(glasnost_ini::value::boolean)
            .unwrap_or(false),
    )
}

/// An engineer infantry type.
fn engineer(l: &Lane) -> Option<String> {
    let r = l.world().rules();
    (0..r.infantry.len())
        .find(|&i| {
            r.unit(Kind::Infantry, i as u32)
                .is_some_and(|u| u.engineer && u.strength > 0)
        })
        .map(|i| r.infantry[i].name.clone())
}

/// One line on the building's look: owner role, manned, drawn live,
/// running slots, and whether its owner's colour definition is the
/// non-player grey or ours.
fn derrick_state(l: &mut Lane, map: usize, what: &str, b: ObjectId) {
    l.ticks(1);
    let _ = l.c.frame_rgba();
    let permille = l.permille(b);
    let s = l.c.session.as_ref().unwrap();
    let live = s.loaded.live.as_ref().expect("a live game");
    let w = l.world();
    let Some(o) = w.obj(b) else {
        println!("derrick-look map {map} {what}: gone");
        return;
    };
    let house = o.owner.and_then(|h| w.houses.get(usize::from(h.0)));
    let name = house
        .map(|h| h.name.as_bytes().to_vec())
        .unwrap_or_default();
    let import = &s.loaded.import;
    let options = &s.loaded.options;
    let (hsv, ours) = glasnost_app::mapview::build::owner_definition(import, options, &name);
    let (grey, _) = glasnost_app::mapview::build::owner_definition(import, options, b"Neutral");
    println!(
        "derrick-look map {map} {what}: owner role {:?}, manned {}, health per mille {permille}, drawn live {}, running slots {}, owner colour the non-player grey {}, ours {ours}",
        house.map(|h| h.role),
        o.econ.building.manned,
        live.drawn_live(b),
        live.slot_names(b).len(),
        hsv == grey,
    );
}
