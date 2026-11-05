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
//! systems (S-0107 claim 17; S-0109; S-0317; S-0616). `debris-pieces`
//! destroys six enemy vehicles and follows their debris pieces tick by
//! tick, with screens of them flying, bouncing and drawn with their shadows
//! (S-0317 claims 10, 17 to 22 and 27). `bouncing-anims` destroys six
//! enemy vehicles and follows the bouncing animations they leave tick by
//! tick (in play, heights, how many have left play), with screens of them
//! falling, with the view on the first one in flight (S-0319 claims 4 to
//! 8).
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
//! S-0108 claim 26, which answers Q-1772), and how many slots are frozen.
//!
//! M8 step A4, note 18 reopened: `derrick-paths` counts, map by map, the
//! map buildings that need an engineer and the non-player buildings that
//! need none but share a body image or a slot animation with a type that
//! does; on the first map with such a building it counts the art of its
//! slots (the drawing-with-object, alternative-palette and tiled flags,
//! and the pixels in the remap range, S-0103 claim 8 and Terms) and
//! whether the drawing animates it after each reason that draws it live
//! (a 1-point hit, protection, a charge flash; S-0108 claim 2, S-0102
//! claim 34, S-0519 claim 25); then, on a map with four starts and four
//! houses, a capture by the last house (S-0103 claim 10).
//!
//! M8 step A3 intake 19 (S-0105 claim 43): `ifv-turrets` counts the
//! multi-turret vehicle types by flag and imported turret table, builds the
//! first gunner one with a filled table empty and with a first passenger of
//! each mode, and lets the first charge-turret one attack, counting the
//! selected turret indices it shows (never which mode shows which).
//!
//! M8 step A3 intake 26 (S-0504 r26 claims 15 and 16, F4 and F5; Q-1871
//! answered): `sinking-list` counts the naval vehicle types by voxel or
//! sprite body, underwater flag and whether a killing hit sinks them, then
//! puts two vessels of one type a killing hit sinks (a turreted one when
//! there is one), with body facings 0 and 16384, and an underwater one on
//! open water; it kills the first two and sets the third sinking directly,
//! as a cull would (S-0614 claim 11; an underwater type never sinks at a
//! killing hit, S-0504 claim 6), and saves the view and a crop centred on
//! each vessel before the hit and after descents 1, 40, 79 and 80,
//! printing each vessel's id, facing, the simulation's pitch and lean, and
//! whether its locomotor draws the lean. Ids, cells, counts, ticks and
//! pitches only.
//!
//! M8 step A3 intake 27 (S-0105 r39 claim 12, Q-1885 answered):
//! `sprite-barrel` puts a sprite-body vehicle with the turret flag and a
//! barrel part (found by flags; when no such type exists, the lane's copy
//! of the import loads a voxel type as a sprite body) on the four sides of
//! a small building and on a high bridge's deck, its turret turned through
//! the four 4-step facings, one screen each.
//!
//! M8 step A3 intake 29 (S-0108 r81 claims 13 and 28, Q-1890 answered):
//! `placed-off` counts, map by map, the map's building entries with the
//! switched-on field 0 (S-0009 claim 40) and the non-player map buildings
//! whose type the map may switch off (S-0405 claim 18). When no entry has
//! the field 0, it switches those buildings off before the first tick, as
//! the field would (the view's first update is their placement). On the
//! first map with such a building that has a running slot animation
//! counting *freeze-unpowered* (S-0108 claim 13), it saves the view fresh,
//! 20 ticks later, after a trigger-style ownership change to the local
//! house (S-0011 claim 24) and after the local house's power-on order,
//! with the house first made short of power by a building of its own that
//! needs and draws power (as S-0108 B22 part (m) sets it up), printing the
//! running slots with their frames and how many are frozen, tick by tick
//! around the order. Counts, frames and ticks only.
//!
//! M8 step A3 intake 53 (S-0113, Q-2081 answered; S-0108 r102 claims 11
//! and 18, Q-2001 answered): `body-rocking` puts three voxel vehicles of
//! the driving locomotor around a point and sets off a rocking warhead
//! there, saving the view and a crop of each over the rock with the view's
//! angles and rates; `super-ungranted` places one building of each
//! disableable superweapon type with the setting off, in both positions of
//! `quirks.ungranted_super_reads_clock`, printing their running slots.
//!
//! M8 step A4 round 8 (the lead's notes 44, 50 and 53): `r8-carrier-squid`
//! puts a carrier ship of the local house on open water, sends its slot
//! aircraft at an enemy vehicle and has an enemy grappling parasite attack
//! the carrier (and, in a second run, kill it), following each slot
//! aircraft tick by tick: its slot state, behaviour, destination, whether
//! it leaves play alive outside the usable area (S-0506 claims 9 to 13;
//! S-0501 claims 10, 17, 34, 35, 45; S-0614 claims 7 and 8).
//! `r8-pad-no-ammo` gives a pad-bound aircraft with no ammunition an attack
//! order, on its pad and while it flies home, and counts its heights,
//! behaviours and the ticks to full ammunition against a control with no
//! order (S-0214 claim 6 part e; S-0501 claims 8, 17, 24, 26, 28, T1 to
//! T3). `r8-garrison-fire` garrisons a civilian building that lets its
//! occupants fire, with an enemy in range before and after the entry, and
//! counts the ticks to the building's first hit (S-0309 claims 1 to 3;
//! S-0508 claim 5; S-0308 claim 13). Ids, cells, counts and ticks only.
//!
//! Run with `cargo test --release -p glasnost-app --test m7b_lane --
//! --ignored --nocapture`. `GLASNOST_LANE_SCENE=a,b` plays only the scenes
//! with those keys; `GLASNOST_UI_MAP=n` picks the n-th offered map.

#![allow(missing_docs)]

use std::collections::BTreeMap;
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
    ("debris-pieces", debris_pieces),
    ("bouncing-anims", bouncing_anims),
    ("carrier-death", carrier_death),
    ("mirage-click", mirage_click),
    ("derrick-look", derrick_look),
    ("derrick-paths", derrick_paths),
    ("ifv-turrets", ifv_turrets),
    ("sprite-barrel", sprite_barrel),
    ("sinking-list", sinking_list),
    ("placed-off", placed_off),
    ("barracks-no-rally", barracks_no_rally),
    ("body-rocking", body_rocking),
    ("super-ungranted", super_ungranted),
    ("r8-carrier-squid", r8_carrier_squid),
    ("r8-pad-no-ammo", r8_pad_no_ammo),
    ("r8-garrison-fire", r8_garrison_fire),
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

/// S-0317 claims 10, 17 to 22 and 27: enemy armed vehicles destroyed, then
/// their debris pieces watched tick by tick (count, heights, contacts as
/// reversals of the turn's sense, net turns) with screens of the pieces
/// flying, bouncing and drawn with their shadows.
fn debris_pieces(lane: &mut Lane) {
    let Some(enemy) = lane.enemy() else { return };
    let home = lane.home();
    let at = lane.open_land(CellXY::new(home.x + 10, home.y + 10));
    lane.look_at(at);
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
    let ids: Vec<ObjectId> = vehicles
        .iter()
        .enumerate()
        .map(|(j, v)| {
            let x = at.x + 2 * (j as i32 % 3);
            let y = at.y + 2 * (j as i32 / 3);
            lane.spawn(Kind::Vehicle, v, enemy, CellXY::new(x, y))
        })
        .collect();
    lane.ticks(2);
    for id in ids {
        kill_with(lane, id, h);
    }
    let mut flips: std::collections::BTreeMap<ObjectId, (bool, u32)> = Default::default();
    let mut most = 0;
    let mut first_contact: Option<usize> = None;
    for t in 1..=600 {
        lane.ticks(1);
        let w = lane.world();
        let mut zs = Vec::new();
        let mut net = Vec::new();
        for (id, p) in &w.pieces {
            let e = flips.entry(*id).or_insert((p.reversed, 0));
            if e.0 != p.reversed {
                *e = (p.reversed, e.1 + 1);
            }
            zs.push(p.pos.z);
            net.push(p.turns);
        }
        let live = w.pieces.len();
        most = most.max(live);
        let bounced = flips.values().filter(|f| f.1 > 0).count();
        println!(
            "debris-pieces +{t}: in play {live}, height min {:?} max {:?}, net turns min {:?} max {:?}, pieces seen {}, with a contact so far {bounced}",
            zs.iter().min(),
            zs.iter().max(),
            net.iter().min(),
            net.iter().max(),
            flips.len(),
        );
        // The first contact: centre the view on that piece, then screens
        // of it and of the ticks after the bounce.
        if first_contact.is_none() && bounced > 0 {
            first_contact = Some(t);
            let at = flips
                .iter()
                .find(|(_, f)| f.1 > 0)
                .and_then(|(id, _)| w.pieces.get(id))
                .map(|p| CellXY::new(p.pos.x / 256, p.pos.y / 256));
            if let Some(c) = at {
                println!("debris-pieces: first contact at +{t}, view centred on cell {c:?}");
                lane.look_at(c);
            }
        }
        let after = first_contact.map(|f| t - f);
        if [2, 5, 9, 14, 20, 30].contains(&t) || matches!(after, Some(0 | 2 | 4 | 8)) {
            lane.save(&format!("debris-pieces-{t:03}"));
        }
        if t > 5 && live == 0 {
            break;
        }
    }
    lane.save("debris-pieces-end");
    let contacts: u32 = flips.values().map(|f| f.1).sum();
    println!(
        "debris-pieces: most in play {most}, pieces {}, reversals (contacts) {contacts}",
        flips.len()
    );
}

/// Play note 7 (S-0417 r64 claim 17, Q-1850): an infantry factory of the
/// local house with no rally point releases five walking infantry units,
/// one every 60 ticks (one line finishes its items at least 54 ticks
/// apart, S-0404 claim 7). Each is sent off the factory's cell in its next
/// turn; the scene prints every release, when each unit's contact ends and
/// where each one ends up.
fn barracks_no_rally(lane: &mut Lane) {
    use glasnost_sim::econ::release::{self, Outcome};
    let local = lane.local();
    let (factory, foot, unit) = {
        let r = lane.world().rules();
        let Some(b) = r.buildings.iter().find(|b| {
            b.factory == Some(glasnost_sim::rules::FactoryKind::Infantry)
                && b.raw.econ.cloning != Some(true)
        }) else {
            println!("barracks-no-rally: no infantry factory type");
            return;
        };
        let Some(u) = (0..r.infantry.len()).find(|&i| {
            r.unit(Kind::Infantry, i as u32).is_some_and(|u| {
                u.strength > 0
                    && u.locomotor == glasnost_sim::rules::Locomotor::Walk
                    && !u.jumpjet
                    && !u.raw.teleporter.unwrap_or(false)
            })
        }) else {
            println!("barracks-no-rally: no walking infantry type");
            return;
        };
        (b.name.clone(), b.foundation, u as u32)
    };
    let home = lane.home();
    let Some(at) = lane.area(CellXY::new(home.x + 8, home.y + 8), foot, 3, |l| {
        l != land::WATER && l != land::ROCK
    }) else {
        println!("barracks-no-rally: no open area");
        return;
    };
    let k = lane.spawn(Kind::Building, &factory, local, at);
    println!(
        "barracks-no-rally: factory {factory} ({}x{}) at {at:?}, rally point {:?}",
        foot.0,
        foot.1,
        lane.world().obj(k).and_then(|o| o.guard_spot)
    );
    lane.look_at(at);
    let holds = |w: &World, c: CellXY| w.cells.list(c, Layer::Ground).contains(&k);
    let mut units: Vec<ObjectId> = Vec::new();
    let mut freed: Vec<Option<i32>> = Vec::new();
    for t in 0..600 {
        if t % 60 == 0 && units.len() < 5 {
            let w = lane.world_mut();
            let u = glasnost_sim::objects::create(w, Kind::Infantry, unit, Some(local));
            let out = release::release(w, k, u);
            let c = w.obj(u).map(|o| o.pos.cell());
            println!(
                "barracks-no-rally +{t}: release {} {out:?}, at {c:?}",
                units.len() + 1
            );
            if out == Outcome::Released {
                units.push(u);
                freed.push(None);
            }
        }
        lane.ticks(1);
        let w = lane.world();
        for (i, &u) in units.iter().enumerate() {
            if freed[i].is_none() && w.obj(u).is_some_and(|o| o.contact.is_none()) {
                freed[i] = Some(t);
                let o = w.obj(u).unwrap();
                println!(
                    "barracks-no-rally +{t}: unit {} out of contact at {:?}, destination {:?}",
                    i + 1,
                    o.pos.cell(),
                    o.dest
                );
            }
        }
        if [1, 30, 120, 300].contains(&t) {
            lane.save(&format!("barracks-no-rally-{t:03}"));
        }
    }
    lane.ticks(200);
    lane.save("barracks-no-rally-end");
    let w = lane.world();
    let mut cells = Vec::new();
    for (i, &u) in units.iter().enumerate() {
        let Some(o) = w.obj(u) else { continue };
        let c = o.pos.cell();
        cells.push(c);
        println!(
            "barracks-no-rally: unit {} rests at {c:?}, on the factory {}, behaviour {:?}, contact {:?}",
            i + 1,
            holds(w, c),
            o.beh.current,
            o.contact
        );
    }
    cells.sort_by_key(|c| (c.y, c.x));
    cells.dedup();
    println!(
        "barracks-no-rally: {} released, {} different cells",
        units.len(),
        cells.len()
    );
}

/// S-0319 claims 4 to 8: enemy armed vehicles destroyed, then the
/// bouncing animations (sprite debris) they leave watched tick by tick:
/// how many are in play, their heights, how many have left play, with
/// screens of them in flight and after their landings.
fn bouncing_anims(lane: &mut Lane) {
    let Some(enemy) = lane.enemy() else { return };
    let home = lane.home();
    let at = lane.open_land(CellXY::new(home.x + 10, home.y + 10));
    lane.look_at(at);
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
    let ids: Vec<ObjectId> = vehicles
        .iter()
        .enumerate()
        .map(|(j, v)| {
            let x = at.x + 2 * (j as i32 % 3);
            let y = at.y + 2 * (j as i32 / 3);
            lane.spawn(Kind::Vehicle, v, enemy, CellXY::new(x, y))
        })
        .collect();
    lane.ticks(2);
    for id in ids {
        kill_with(lane, id, h);
    }
    let mut seen: std::collections::BTreeSet<ObjectId> = Default::default();
    let mut most = 0;
    let mut centred = false;
    let mut last_landing = None;
    for t in 1..=400 {
        lane.ticks(1);
        let w = lane.world();
        let flying: Vec<(ObjectId, glasnost_sim::world::Pos)> = w
            .anims
            .anims
            .iter()
            .filter(|(_, a)| a.flight.is_some())
            .filter_map(|(id, a)| Some((*id, a.pos?)))
            .collect();
        let before = seen.len();
        seen.extend(flying.iter().map(|f| f.0));
        let live = flying.len();
        let ended = seen.len() - live;
        most = most.max(live);
        let zs: Vec<i32> = flying.iter().map(|f| f.1.z).collect();
        println!(
            "bouncing-anims +{t}: in play {live}, height min {:?} max {:?}, seen {}, left play {ended}",
            zs.iter().min(),
            zs.iter().max(),
            seen.len(),
        );
        if seen.len() > before && !centred {
            centred = true;
            let c = flying[0].1;
            let c = CellXY::new(c.x / 256, c.y / 256);
            println!("bouncing-anims: first in flight at +{t}, view centred on cell {c:?}");
            lane.look_at(c);
        }
        if ended > 0 && last_landing.is_none() {
            last_landing = Some(t);
            println!("bouncing-anims: first left play at +{t}");
        }
        if [1, 3, 6, 10, 15, 20, 30, 45].contains(&t) {
            lane.save(&format!("bouncing-anims-{t:03}"));
        }
        if t > 5 && live == 0 {
            println!("bouncing-anims: none in play from +{t}");
            break;
        }
    }
    lane.save("bouncing-anims-end");
    println!("bouncing-anims: most in play {most}, seen {}", seen.len());
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

/// The selected turret index `id` shows now, as the live view derives it
/// (S-0105 claim 43, S3).
fn shown_index(l: &Lane, id: ObjectId) -> Option<i32> {
    let s = l.c.session.as_ref().unwrap();
    let live = s.loaded.live.as_ref()?;
    let w = l.world();
    let o = w.obj(id)?;
    let table = live
        .import_index(o.kind, o.type_index)
        .and_then(|i| s.loaded.import.turret_tables.get(&i));
    glasnost_app::mapview::build::selected_turret_index(w, id, (w.tick - 1).max(0), table)
}

/// M8 step A3 intake 19 (S-0105 r35 claim 43, Q-1760; the lead's play notes
/// 3 and 14): counts of the multi-turret vehicle types by flag and turret
/// table after import; the first gunner multi-turret type with a filled
/// table built empty and then with one first passenger of each mode some
/// infantry type has (a fresh vehicle per passenger), its shown index
/// counted (distinct indices, passengers showing none), one screen each;
/// the first charge-turret multi-turret type attacking an enemy vehicle,
/// the ticks it shows each turret number. Counts, cells and ticks only:
/// which mode shows which turret is never printed.
fn ifv_turrets(lane: &mut Lane) {
    let (multi, gunners, charge, neither, filled) = {
        let s = lane.c.session.as_ref().unwrap();
        let import = &s.loaded.import;
        let registry = &import.outcome.registries[&glasnost_convert::kinds::Kind::Vehicles];
        let w = lane.world();
        let r = w.rules();
        let mut k = (Vec::new(), Vec::new(), Vec::new(), 0usize, 0usize);
        for i in 0..r.vehicles.len() {
            let Some(u) = r.unit(Kind::Vehicle, i as u32) else {
                continue;
            };
            let c = &u.combat;
            if c.turret_count <= 0 || c.gattling {
                continue;
            }
            k.0.push(i);
            let name = r.vehicles[i].name.as_bytes();
            let table = registry
                .iter()
                .position(|t| t.name.eq_ignore_ascii_case(name))
                .and_then(|x| import.turret_tables.get(&x));
            if table.is_some() {
                k.4 += 1;
            }
            if c.charge_turret {
                k.2.push(i);
            } else if c.gunner {
                k.1.push((i, table.is_some()));
            } else {
                k.3 += 1;
            }
        }
        k
    };
    println!(
        "multi-turret vehicle types {}: gunner (not charge-turret) {}, charge-turret {}, neither {}; \
         with a filled turret table {}; gunner types with a filled table {}",
        multi.len(),
        gunners.len(),
        charge.len(),
        neither,
        filled,
        gunners.iter().filter(|(_, f)| *f).count()
    );
    let local = lane.local();
    let home = lane.home();
    let at = lane.open_land(CellXY::new(home.x + 8, home.y + 8));
    if let Some(&(gi, _)) = gunners.iter().find(|(_, f)| *f) {
        let name = lane.world().rules().vehicles[gi].name.clone();
        let empty = lane.spawn(Kind::Vehicle, &name, local, at);
        lane.ticks(2);
        lane.look_at(at);
        let none_empty = shown_index(lane, empty).is_none();
        println!("gunner type, no passenger: shows none {none_empty}");
        lane.save("ifv-empty");
        // One infantry type per mode index (the first in the list).
        let modes: std::collections::BTreeMap<i32, String> = {
            let r = lane.world().rules();
            let mut m = std::collections::BTreeMap::new();
            for i in 0..r.infantry.len() {
                if let Some(u) = r.unit(Kind::Infantry, i as u32)
                    && u.strength > 0
                {
                    m.entry(u.combat.ifv_mode)
                        .or_insert_with(|| r.infantry[i].name.clone());
                }
            }
            m
        };
        let mut seen = std::collections::BTreeSet::new();
        let mut none = 0;
        for (n, pname) in modes.values().enumerate() {
            let cell = CellXY::new(at.x + 3 * (n as i32 % 6), at.y + 3 + 3 * (n as i32 / 6));
            let cell = lane.open_land(cell);
            let v = lane.spawn(Kind::Vehicle, &name, local, cell);
            let p = lane.spawn(
                Kind::Infantry,
                pname,
                local,
                CellXY::new(cell.x, cell.y + 1),
            );
            glasnost_sim::transport::board(lane.world_mut(), p, v);
            lane.ticks(2);
            match shown_index(lane, v) {
                Some(k) => {
                    seen.insert(k);
                }
                None => none += 1,
            }
            lane.look_at(cell);
            lane.save(&format!("ifv-passenger-{n}"));
        }
        println!(
            "gunner type: {} passenger modes tried; distinct turret indices shown {}; passengers showing none {none}",
            modes.len(),
            seen.len()
        );
    }
    let (Some(&ci), Some(enemy)) = (charge.first(), lane.enemy()) else {
        println!("no charge-turret multi-turret type or no enemy: charge part skipped");
        return;
    };
    // An unarmed enemy vehicle type, so that the charge-turret vehicle is not shot back at.
    let Some(target) = ({
        let r = lane.world().rules();
        (0..r.vehicles.len())
            .find(|&i| {
                r.unit(Kind::Vehicle, i as u32)
                    .is_some_and(|u| u.strength > 0 && u.combat.slots.iter().all(Option::is_none))
            })
            .map(|i| r.vehicles[i].name.clone())
    }) else {
        println!("no unarmed vehicle type: charge part skipped");
        return;
    };
    let name = lane.world().rules().vehicles[ci].name.clone();
    let cell = lane.open_land(CellXY::new(at.x - 10, at.y));
    let c = lane.spawn(Kind::Vehicle, &name, local, cell);
    let t = lane.spawn(
        Kind::Vehicle,
        &target,
        enemy,
        CellXY::new(cell.x + 3, cell.y),
    );
    lane.ticks(2);
    lane.look_at(cell);
    println!(
        "charge-turret type: index at creation {:?}",
        shown_index(lane, c)
    );
    lane.save("charge-start");
    let n = lane.order(local, c, ClickOn::Object(t));
    // The shown index run by run, (index, ticks), and the shots, for 300 ticks: the target may
    // die early; the countdown is followed to its end.
    let mut runs: Vec<(Option<i32>, usize)> = Vec::new();
    let mut shots = 0;
    let mut last_until = lane.world().obj(c).map(|o| o.combat.rearm_until);
    for _ in 0..300 {
        lane.ticks(1);
        let k = shown_index(lane, c);
        let until = lane.world().obj(c).map(|o| o.combat.rearm_until);
        if until != last_until {
            shots += 1;
            last_until = until;
        }
        match runs.last_mut() {
            Some((x, n)) if *x == k => *n += 1,
            _ => {
                runs.push((k, 1));
                if runs.len() <= 6 {
                    lane.save(&format!("charge-step-{}", runs.len()));
                }
            }
        }
    }
    println!(
        "charge-turret type: {n} orders; {shots} shots; shown index runs (index, ticks) {runs:?}; \
         alive {}, target alive {}",
        lane.world().obj(c).is_some(),
        lane.world().obj(t).is_some()
    );
}

/// The subject of `sprite-barrel` (S-0105 claim 12, *Sprite body*): the
/// first vehicle type with the turret flag, a barrel part and no voxel body
/// (a sprite body), found by its art and rules flags. Prints the counts of
/// turret-flag vehicle types by body and barrel part. Without such a type,
/// MS-33 part k's way is taken in this lane's own copy of the import: the
/// first IFV-class type (a gunner multi-turret type with a filled turret
/// table) with a barrel part, else the first turret-flag type with one, is
/// loaded as a sprite body would be (S-0007 claim 19): its voxel body and
/// numbered parts are cleared, and its single turret and barrel parts are
/// the files a sprite body reads, the image name with the turret and barrel
/// suffixes, which a numbered type holds as its parts 0 (S-0007 claim 16).
/// Where it has no sprite file it borrows the one of the first vehicle type
/// drawn as a sprite. The scene says so.
fn sprite_barrel_subject(lane: &mut Lane, print: bool) -> Option<String> {
    use glasnost_convert::kinds::Kind as CKind;
    let s = lane.c.session.as_mut().unwrap();
    let r = s.play.world().rules();
    let import = &s.loaded.import;
    let registry = &import.outcome.registries[&CKind::Vehicles];
    // (sim type name, import index, voxel body, barrel part, sprite file, IFV class)
    let mut types = Vec::new();
    let mut donor = None;
    for (i, t) in r.vehicles.iter().enumerate() {
        let Some(u) = r.unit(Kind::Vehicle, i as u32).filter(|u| u.strength > 0) else {
            continue;
        };
        let Some(x) = registry
            .iter()
            .position(|e| e.name.eq_ignore_ascii_case(t.name.as_bytes()))
        else {
            continue;
        };
        let Some(art) = import.arts.get(&(CKind::Vehicles, x)) else {
            continue;
        };
        let body = art.voxels.contains_key("body");
        if !body && donor.is_none() {
            donor = art.sprite.clone();
        }
        if !u.turret {
            continue;
        }
        let c = &u.combat;
        let ifv = c.gunner
            && c.turret_count > 0
            && !c.gattling
            && !c.charge_turret
            && import.turret_tables.contains_key(&x);
        // The single barrel a sprite body reads, or a numbered type's part 0 (the same file).
        let barrel = art.voxels.contains_key("barrel") || art.voxels.contains_key("barrel0");
        if ifv && print {
            let parts = |p: &str| art.voxels.keys().filter(|k| k.starts_with(p)).count();
            println!(
                "IFV-class type: turret count {}, voxel turret parts {}, voxel barrel parts {}",
                c.turret_count,
                parts("turret"),
                parts("barrel")
            );
        }
        types.push((t.name.clone(), x, body, barrel, art.sprite.is_some(), ifv));
    }
    type T = (String, usize, bool, bool, bool, bool);
    let count = |f: &dyn Fn(&T) -> bool| types.iter().filter(|t| f(t)).count();
    if print {
        println!(
            "turret-flag vehicle types {}: sprite body with a barrel part {}, sprite body without {}, \
             voxel body with a barrel part {} (with a sprite file {}, IFV class {}); \
             a sprite-body vehicle type to borrow a sprite file from {}; IFV-class types {} (with a barrel part {})",
            types.len(),
            count(&|t| !t.2 && t.3 && t.4),
            count(&|t| !t.2 && !t.3 && t.4),
            count(&|t| t.2 && t.3),
            count(&|t| t.2 && t.3 && t.4),
            count(&|t| t.2 && t.3 && t.5),
            donor.is_some(),
            count(&|t| t.5),
            count(&|t| t.5 && t.3)
        );
    }
    if let Some(t) = types.iter().find(|t| !t.2 && t.3 && t.4) {
        return Some(t.0.clone());
    }
    let t = types
        .iter()
        .find(|t| t.2 && t.3 && t.5)
        .or_else(|| types.iter().find(|t| t.2 && t.3))?
        .clone();
    let art = s.loaded.import.arts.get_mut(&(CKind::Vehicles, t.1))?;
    let numbered = !art.voxels.contains_key("barrel");
    let single: Vec<_> = [("turret", "turret0"), ("barrel", "barrel0")]
        .into_iter()
        .filter_map(|(role, part0)| {
            let file = art.voxels.get(role).or_else(|| art.voxels.get(part0))?;
            Some((role.to_string(), file.clone()))
        })
        .collect();
    art.voxels.clear();
    art.voxels.extend(single);
    let borrowed = art.sprite.is_none();
    if borrowed {
        art.sprite = Some(donor?);
    }
    if print {
        println!(
            "lane copy: the subject (IFV class {}) loaded as a sprite body: voxel body cleared, \
             single parts from numbered parts 0 {numbered}, sprite file borrowed {borrowed}",
            t.5
        );
    }
    Some(t.0)
}

/// M8 step A3 intake 27 (S-0105 r39 claim 12, *One image* and *Depth*; Q-1885
/// answered): a sprite-body vehicle with the turret flag and a barrel part
/// ([`sprite_barrel_subject`]) placed round a small building of the local
/// house spawned near its start, behind it and in front of it, its turret
/// turned through the
/// four 4-step facings, one screen each; then on the deck of the first high
/// bridge with banks at deck level of the maps tried (`GLASNOST_LANE_MAPS`,
/// default 8), driven there by a click, and turned through the same
/// facings. Counts, cells, layers and ticks only.
fn sprite_barrel(lane: &mut Lane) {
    let Some(name) = sprite_barrel_subject(lane, true) else {
        println!("sprite-barrel: no turret-flag vehicle type with a barrel part");
        return;
    };
    let local = lane.local();
    // A small building of the local house near its start (our pick), with room round it.
    let Some(bi) = small_building(lane.world()) else {
        println!("sprite-barrel: no small building type");
        return;
    };
    let (bname, (fw, fh)) = {
        let r = lane.world().rules();
        (r.buildings[bi].name.clone(), r.buildings[bi].foundation)
    };
    let home = lane.home();
    let Some(at) = lane.area(CellXY::new(home.x + 8, home.y + 8), (fw, fh), 3, |l| {
        l != land::WATER && l != land::ROCK
    }) else {
        println!("sprite-barrel: no room for the building");
        return;
    };
    lane.spawn(Kind::Building, &bname, local, at);
    let (x0, y0, x1, y1) = (at.x, at.y, at.x + fw - 1, at.y + fh - 1);
    println!("sprite-barrel: building of {fw}x{fh} cells at {at:?}");
    // Behind the building (map north and west of it) and in front of it (south and east).
    let places = [
        ("north", CellXY::new((x0 + x1) / 2, y0 - 1)),
        ("west", CellXY::new(x0 - 1, (y0 + y1) / 2)),
        ("south", CellXY::new((x0 + x1) / 2, y1 + 1)),
        ("east", CellXY::new(x1 + 1, (y0 + y1) / 2)),
    ];
    let mut cars = Vec::new();
    for (side, at) in places {
        let v = lane.spawn(Kind::Vehicle, &name, local, at);
        let placed = lane.world().obj(v).map(|o| o.pos.cell());
        println!("sprite-barrel: vehicle {v:?} {side} of the building at {placed:?}");
        cars.push(v);
    }
    lane.ticks(2);
    lane.look_at(CellXY::new((x0 + x1) / 2, (y0 + y1) / 2));
    turn_and_save(lane, &cars, "building");
    // On a high bridge's deck.
    let mut found = None;
    for map in 0..env_num("GLASNOST_LANE_MAPS", 8) {
        let mut l = Lane::start_on(map).expect("the install is set");
        if let Some(&ab) = lane_bridges(l.world()).first() {
            let Some(name) = sprite_barrel_subject(&mut l, false) else {
                return;
            };
            println!(
                "sprite-barrel: bridge on map {map} from {:?} to {:?}",
                ab.0, ab.1
            );
            found = Some((l, ab, name));
            break;
        }
    }
    let Some((mut l, (a, b), name)) = found else {
        println!("sprite-barrel: no high bridge with banks at deck level on the maps tried");
        return;
    };
    let local = l.local();
    let mid = CellXY::new((a.x + b.x) / 2, (a.y + b.y) / 2);
    let v = l.spawn(Kind::Vehicle, &name, local, a);
    l.ticks(2);
    let n = l.order(local, v, ClickOn::Cell(mid));
    let mut on_deck = None;
    for _ in 0..1500 {
        l.ticks(1);
        let w = l.world();
        let Some(o) = w.obj(v) else { break };
        if o.layer == Layer::Deck && o.pos.cell() == mid && o.dest.is_none() {
            on_deck = Some((w.tick, o.pos.z - w.surface_at(o.pos.x, o.pos.y)));
            break;
        }
    }
    println!(
        "sprite-barrel: {n} orders; on the deck's middle at (tick, height above ground) {on_deck:?}"
    );
    let o = l.world().obj(v).map(|o| (o.pos.cell(), o.layer));
    println!("sprite-barrel: vehicle at {o:?}");
    l.look_at(mid);
    turn_and_save(&mut l, &[v], "bridge");
}

/// Sets each of `cars`' turret to the four 4-step facings (north, east,
/// south, west) in turn, saving a screen at each.
fn turn_and_save(lane: &mut Lane, cars: &[ObjectId], what: &str) {
    for (k, facing) in [0u16, 16384, 32768, 49152].into_iter().enumerate() {
        for &v in cars {
            if let Some(o) = lane.world_mut().obj_mut(v) {
                o.turret = glasnost_sim::turning::Turning::at(facing);
            }
        }
        lane.ticks(1);
        lane.save(&format!("sprite-barrel-{what}-{k}"));
    }
}

/// High bridges of the world whose both ends are banks at deck level: the
/// (bank, bank) pairs along each deck run (as `fix_lane.rs`).
fn lane_bridges(w: &World) -> Vec<(CellXY, CellXY)> {
    use glasnost_sim::cells::Axis;
    let cells = &w.cells;
    let level = |c: CellXY| cells.cell(c).map(|c| i32::from(c.level));
    let deck = |c: CellXY| cells.idx(c).is_some_and(|i| cells.deck[i]);
    let mut out = Vec::new();
    for y in 0..512 {
        for x in 0..512 {
            let start = CellXY::new(x, y);
            let Some(i) = cells.idx(start) else { continue };
            if !cells.deck[i] {
                continue;
            }
            let (dx, dy) = match cells.bridge_axis[i] {
                Axis::X => (1, 0),
                Axis::Y => (0, 1),
                Axis::None => continue,
            };
            if deck(CellXY::new(x - dx, y - dy)) {
                continue;
            }
            let mut end = start;
            while deck(CellXY::new(end.x + dx, end.y + dy)) {
                end = CellXY::new(end.x + dx, end.y + dy);
            }
            let (a, b) = (
                CellXY::new(x - dx, y - dy),
                CellXY::new(end.x + dx, end.y + dy),
            );
            let ground = level(start).unwrap_or(0);
            if level(a) == Some(ground + 4) && level(b) == Some(ground + 4) && end != start {
                out.push((a, b));
            }
        }
    }
    out
}

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
        Lane::start_with(map, 0)
    }

    /// Starts a skirmish on the `map`-th offered map, runs `before` on the
    /// lane before the first tick, then plays the first ticks.
    fn start_hooked(map: usize, before: impl FnOnce(&mut Lane)) -> Option<Lane> {
        Lane::start_full(map, 0, before)
    }

    /// Starts a skirmish on the `map`-th offered map with `extra` more
    /// computer houses in the lobby's third and later rows; `None` when the
    /// map has too few starts for them.
    fn start_with(map: usize, extra: usize) -> Option<Lane> {
        Lane::start_full(map, extra, |_| {})
    }

    /// [`Lane::start_with`], running `before` before the first tick.
    fn start_full(map: usize, extra: usize, before: impl FnOnce(&mut Lane)) -> Option<Lane> {
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
        if extra > 0 {
            let starts = c.install.as_ref().unwrap().maps[map].1.starts as usize;
            if starts < 2 + extra {
                return None;
            }
            for row in 2..2 + extra {
                let _ = c.frame();
                let r = c.setup_targets.rect_of(&SetupHit::Kind(row, 1)).unwrap();
                click(&mut c, r);
            }
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
        before(&mut lane);
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
/// claims 8 and 9; S-0509 claim 8; S-0515 claim 2; S-0108 claim 26): the map's
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
        "derrick-look map {map} {what}: owner role {:?}, manned {}, health per mille {permille}, drawn live {}, running slots {}, frozen {}, owner colour the non-player grey {}, ours {ours}",
        house.map(|h| h.role),
        o.econ.building.manned,
        live.drawn_live(b),
        live.slot_names(b).len(),
        live.frozen_slots(b),
        hsv == grey,
    );
}

/// Note 18 reopened (S-0103 claims 8 and 10, Terms *remap range*; S-0108
/// claims 2 and 11; S-0102 claim 34; S-0519 claim 25; S-0515 claim 2;
/// S-0108 claims 13 and 26): over the first maps (of `GLASNOST_LANE_MAPS`, default 8) the
/// buildings that need an engineer and the lookalikes that do not; on the
/// first map with one, its art and each reason it is drawn live; then a
/// capture by a fourth house. Counts only.
fn derrick_paths(_: &mut Lane) {
    let maps = env_num("GLASNOST_LANE_MAPS", 8);
    let mut done = false;
    for map in 0..maps {
        let Some(mut l) = Lane::start_on(map) else {
            return;
        };
        l.ticks(3);
        let _ = l.c.frame_rgba();
        lookalikes(&l, map);
        if done {
            continue;
        }
        let found = derricks(&l);
        let Some(&b) = found.first() else { continue };
        done = true;
        derrick_art(&l, map, &found);
        derrick_reasons(&mut l, map, b);
    }
    for map in 0..maps {
        let Some(mut l) = Lane::start_with(map, 2) else {
            continue;
        };
        l.ticks(3);
        let _ = l.c.frame_rgba();
        let Some(&b) = derricks(&l).first() else {
            continue;
        };
        derrick_fourth_house(&mut l, map, b);
        return;
    }
    println!("derrick-paths: no map with four starts and a derrick");
}

/// Whether type `t` needs an engineer (S-0515 Terms).
fn needs_engineer(w: &World, t: u32) -> bool {
    w.rules()
        .econ
        .power
        .buildings
        .get(t as usize)
        .is_some_and(|p| p.needs_engineer)
}

/// The art document of the lane's import.
fn art_doc(l: &Lane) -> glasnost_ini::Document {
    let s = l.c.session.as_ref().unwrap();
    let f = s.loaded.import.art_file.as_ref().expect("an art file");
    glasnost_ini::Document::parse(&f.bytes, Default::default(), &mut Vec::new())
}

/// A building type's art image and the names of its five set-up slots'
/// normal and damaged animations (S-0108 claim 11 and Import mapping).
fn type_look(l: &Lane, doc: &glasnost_ini::Document, t: u32) -> Option<(Vec<u8>, Vec<Vec<u8>>)> {
    use glasnost_app::mapview::{build, live, slots};
    let s = l.c.session.as_ref().unwrap();
    let import = &s.loaded.import;
    let name = live::name_bytes(&l.world().rules().buildings.get(t as usize)?.name);
    let idx = build::find_type(import, glasnost_convert::kinds::Kind::Buildings, &name)?;
    let image = import
        .arts
        .get(&(glasnost_convert::kinds::Kind::Buildings, idx))?
        .image
        .clone();
    let names = slots::SETUP_SLOTS
        .iter()
        .map(|base| slots::read_slot(doc, &image, base))
        .flat_map(|d| [d.normal, d.damaged])
        .filter(|n| !n.is_empty())
        .collect();
    Some((image, names))
}

/// Per map: the map buildings that need an engineer (with running slots,
/// and gates or occupiable among them), and the non-player map buildings
/// that need none, have running slots and share a body image or a slot
/// animation with a type that needs one.
fn lookalikes(l: &Lane, map: usize) {
    let doc = art_doc(l);
    let w = l.world();
    let r = w.rules();
    let tech: Vec<(Vec<u8>, Vec<Vec<u8>>)> = (0..r.buildings.len() as u32)
        .filter(|&t| needs_engineer(w, t))
        .filter_map(|t| type_look(l, &doc, t))
        .collect();
    let s = l.c.session.as_ref().unwrap();
    let live = s.loaded.live.as_ref().expect("a live game");
    let (mut n_tech, mut n_tech_slots, mut n_tech_live_from_start) = (0, 0, 0);
    let (mut n_other, mut same_image, mut shared_anim) = (0, 0, 0);
    for &id in live.building_ids.values() {
        let Some(o) = w.obj(id).filter(|o| o.on_map) else {
            continue;
        };
        let running = !live.slot_names(id).is_empty();
        if needs_engineer(w, o.type_index) {
            n_tech += 1;
            n_tech_slots += usize::from(running);
            let b = r.buildings.get(o.type_index as usize);
            n_tech_live_from_start +=
                usize::from(b.is_some_and(|b| b.gate || b.building.can_be_occupied));
            continue;
        }
        let player = o
            .owner
            .and_then(|h| w.houses.get(usize::from(h.0)))
            .is_some_and(|h| h.role == glasnost_sim::world::HouseRole::Player);
        if player || !running {
            continue;
        }
        n_other += 1;
        let Some((image, names)) = type_look(l, &doc, o.type_index) else {
            continue;
        };
        same_image += usize::from(tech.iter().any(|(i, _)| *i == image));
        shared_anim += usize::from(
            tech.iter()
                .any(|(_, t)| names.iter().any(|n| t.contains(n))),
        );
    }
    println!(
        "derrick-paths map {map}: map buildings needing an engineer {n_tech}, with running slots {n_tech_slots}, gate or occupiable {n_tech_live_from_start}; non-player map buildings needing none with running slots {n_other}, same body image as an engineer type {same_image}, sharing a slot animation with one {shared_anim}"
    );
}

/// The opaque pixels and those in the remap range 16 … 31 (S-0103 Terms)
/// of image frame `f` of sprite file `file`.
fn remap_count(l: &Lane, file: &[u8], f: usize) -> Option<(usize, usize)> {
    let s = l.c.session.as_ref().unwrap();
    let bytes = s.loaded.assets.get(file)?;
    let shp = glasnost_fmt::shp::Shp::parse(bytes).ok()?;
    let img = shp.decode_frame(f).ok()?;
    let px = img.pixels();
    Some((
        px.iter().filter(|&&p| p != 0).count(),
        px.iter().filter(|&&p| (16..=31).contains(&p)).count(),
    ))
}

/// The frame count of sprite file `file`.
fn frame_count(l: &Lane, file: &[u8]) -> usize {
    let s = l.c.session.as_ref().unwrap();
    s.loaded
        .assets
        .get(file)
        .and_then(|b| glasnost_fmt::shp::Shp::parse(b).ok())
        .map_or(0, |shp| shp.frames.len())
}

/// The art of each type among `found`, numbered (no names): the body's
/// image frames with their opaque and remap-range pixels, and for each
/// set-up slot animation its palette flags (S-0103 claim 8 rows 3 to 5)
/// and the remap-range pixels over its image frames.
fn derrick_art(l: &Lane, map: usize, found: &[ObjectId]) {
    use glasnost_app::mapview::{build, slots};
    use glasnost_convert::kinds::Kind as CKind;
    let doc = art_doc(l);
    let w = l.world();
    let types: std::collections::BTreeSet<u32> = found
        .iter()
        .filter_map(|&id| w.obj(id).map(|o| o.type_index))
        .collect();
    let s = l.c.session.as_ref().unwrap();
    let import = &s.loaded.import;
    for (k, &t) in types.iter().enumerate() {
        let Some((image, _)) = type_look(l, &doc, t) else {
            continue;
        };
        let name = glasnost_app::mapview::live::name_bytes(&w.rules().buildings[t as usize].name);
        let Some(idx) = build::find_type(import, CKind::Buildings, &name) else {
            continue;
        };
        if let Some(body) = import
            .arts
            .get(&(CKind::Buildings, idx))
            .and_then(|a| a.sprite.clone())
        {
            let n = frame_count(l, &body);
            let per: Vec<(usize, usize)> = (0..n / 2)
                .filter_map(|f| remap_count(l, &body, f))
                .collect();
            println!(
                "derrick-paths map {map} type {k}: body frames {n}, image frames (opaque, remap-range) pixels {per:?}"
            );
        }
        for (si, base) in slots::SETUP_SLOTS.iter().enumerate() {
            let def = slots::read_slot(&doc, &image, base);
            if !def.normal.is_empty() {
                println!(
                    "derrick-paths map {map} type {k} slot {si}: counting power flag {:?} (S-0108 claim 13)",
                    def.power.counting()
                );
            }
            for (variant, anim) in [("normal", &def.normal), ("damaged", &def.damaged)] {
                if anim.is_empty() {
                    continue;
                }
                let Some(a) = build::find_type(import, CKind::Animations, anim) else {
                    println!("derrick-paths map {map} type {k} slot {si} {variant}: unknown type");
                    continue;
                };
                let at = slots::read_anim(&doc, build::anim_own_section(import, a));
                let sprite = import
                    .arts
                    .get(&(CKind::Animations, a))
                    .and_then(|x| x.sprite.clone());
                let (n, opaque, remap, with) = sprite.as_ref().map_or((0, 0, 0, 0), |f| {
                    let n = frame_count(l, f);
                    let images = if at.shadow { n / 2 } else { n };
                    let c: Vec<(usize, usize)> =
                        (0..images).filter_map(|i| remap_count(l, f, i)).collect();
                    (
                        n,
                        c.iter().map(|x| x.0).sum::<usize>(),
                        c.iter().map(|x| x.1).sum::<usize>(),
                        c.iter().filter(|x| x.1 > 0).count(),
                    )
                });
                println!(
                    "derrick-paths map {map} type {k} slot {si} {variant}: drawn with its building {}, alternative palette {}, tiled {}, normal light {}, sprite {}, frames {n}, opaque pixels {opaque}, remap-range pixels {remap}, image frames with any {with}",
                    at.cell_drawer,
                    at.alt_palette,
                    at.tiled,
                    at.normal_light,
                    sprite.is_some(),
                );
            }
        }
    }
}

/// The image ids the scene draws for building `b` (its cell-pass code or
/// its simulation code), sorted.
fn drawn_images(l: &Lane, b: ObjectId) -> Vec<u32> {
    let s = l.c.session.as_ref().unwrap();
    let live = s.loaded.live.as_ref().expect("a live game");
    let code = live
        .building_ids
        .iter()
        .find(|(_, id)| **id == b)
        .map(|(c, _)| *c);
    let sim = glasnost_app::mapview::live::sim_code(b);
    let mut v: Vec<u32> = s
        .scene
        .items
        .iter()
        .filter(|i| i.object == Some(sim) || (code.is_some() && i.object == code))
        .map(|i| i.image)
        .collect();
    v.sort_unstable();
    v
}

/// Over `n` ticks: whether `b` was drawn live and how many different image
/// sets the scene drew for it (more than 1: the drawing animates it).
fn watch_drawing(l: &mut Lane, b: ObjectId, n: usize) -> (bool, usize) {
    let mut seen = std::collections::BTreeSet::new();
    let mut live_drawn = false;
    for _ in 0..n {
        l.ticks(1);
        seen.insert(drawn_images(l, b));
        let s = l.c.session.as_ref().unwrap();
        live_drawn |= s.loaded.live.as_ref().unwrap().drawn_live(b);
    }
    (live_drawn, seen.len())
}

/// Each reason that draws a never-captured map building live, one at a
/// time from its fresh state: nothing, a 1-point hit (no damage frame, a
/// full-looking health bar), protection (S-0102 claim 34) and a charge
/// flash (S-0519 claim 25). Prints whether the drawing animates it.
fn derrick_reasons(l: &mut Lane, map: usize, b: ObjectId) {
    use glasnost_sim::superweapons::{ProtectKind, Protection, Tint};
    let cell = l.world().obj(b).map_or(CellXY::new(0, 0), |o| o.pos.cell());
    l.look_at(cell);
    let full = l.world().obj(b).map_or(0, |o| o.health);
    for reason in ["nothing", "1-point hit", "protection", "charge flash"] {
        let tick = l.world().tick;
        if let Some(o) = l.world_mut().obj_mut(b) {
            match reason {
                "1-point hit" => o.health = full - 1,
                "protection" => {
                    o.sw.protection = Some(Protection {
                        start: tick - 1,
                        duration: 200,
                        kind: ProtectKind::Curtain,
                        tint: Tint::Rise(tick - 1),
                    })
                }
                "charge flash" => o.combat.look.flash = 200,
                _ => {}
            }
        }
        l.ticks(2);
        let lost = full - l.world().obj(b).map_or(0, |o| o.health);
        let (drawn_live, sets) = watch_drawing(l, b, 24);
        let manned = l.world().obj(b).is_some_and(|o| o.econ.building.manned);
        println!(
            "derrick-paths map {map} {reason}: manned {manned}, health points lost {lost}, drawn live {drawn_live}, image sets drawn over 24 ticks {sets} (more than 1: animates)"
        );
        l.save(&format!("derrick-paths-{map}-{}", reason.replace(' ', "-")));
        if let Some(o) = l.world_mut().obj_mut(b) {
            o.health = full;
            o.sw.protection = None;
            o.combat.look.flash = 0;
        }
        l.ticks(2);
    }
}

/// A capture by the last player house of a four-house skirmish, through
/// a click of its engineer: its colour slot and whether its colour is the
/// non-player grey (S-0103 claim 10).
fn derrick_fourth_house(l: &mut Lane, map: usize, b: ObjectId) {
    let w = l.world();
    let Some(house) = (0..w.houses.len())
        .rev()
        .map(|i| HouseId(i as u8))
        .find(|h| w.houses[usize::from(h.0)].role == glasnost_sim::world::HouseRole::Player)
    else {
        return;
    };
    let players = w
        .houses
        .iter()
        .filter(|h| h.role == glasnost_sim::world::HouseRole::Player)
        .count();
    let name = live_name(w, house);
    let slot = {
        let s = l.c.session.as_ref().unwrap();
        s.loaded
            .options
            .slots
            .iter()
            .find(|(n, _)| *n == name)
            .map(|(_, s)| *s)
    };
    let Some(eng) = engineer(l) else { return };
    let cell = l.world().obj(b).map_or(CellXY::new(0, 0), |o| o.pos.cell());
    l.look_at(cell);
    let spot = l.open_land(CellXY::new(cell.x - 4, cell.y + 4));
    let e = l.spawn(Kind::Infantry, &eng, house, spot);
    l.ticks(2);
    let orders = l.order(house, e, ClickOn::Object(b));
    let took = until(l, 1500, |w| {
        w.obj(b).is_some_and(|o| o.owner == Some(house))
    });
    l.ticks(20);
    println!(
        "derrick-paths map {map} four houses: player houses {players}, capturing house index {}, its lobby colour slot {slot:?}, order {orders}, ticks to the ownership change {took:?}",
        house.0
    );
    derrick_state(l, map, "captured by the last house", b);
    l.save(&format!("derrick-paths-{map}-last-house"));
}

/// A house's name as the drawing keys it.
fn live_name(w: &World, h: HouseId) -> Vec<u8> {
    w.houses
        .get(usize::from(h.0))
        .map(|h| h.name.as_bytes().to_vec())
        .unwrap_or_default()
}

// ---------------------------------------------------------------------------
// M8 step A3 intake 26 (S-0504 r26 claims 15, 16: a sinking vessel's lean)

/// A naval vehicle type: index, voxel body, turret flag, underwater, sinks
/// at a killing hit.
type NavalType = (u32, bool, bool, bool, bool);

/// `w` × `h` pixels of the frame centred on `at` (kept inside the frame),
/// scaled up three times.
fn crop_at(
    rgba: &[u8],
    size: (usize, usize),
    at: (i32, i32),
    w: usize,
    h: usize,
) -> (u32, u32, Vec<u8>) {
    let (fw, fh) = size;
    let x0 = (at.0 - w as i32 / 2).clamp(0, (fw - w) as i32) as usize;
    let y0 = (at.1 - h as i32 / 2).clamp(0, (fh - h) as i32) as usize;
    let mut out = Vec::with_capacity(w * h * 36);
    for y in 0..h * 3 {
        for x in 0..w * 3 {
            let i = ((y0 + y / 3) * fw + x0 + x / 3) * 4;
            out.extend_from_slice(&rgba[i..i + 4]);
        }
    }
    ((w * 3) as u32, (h * 3) as u32, out)
}

/// S-0504 claims 15 and 16 (F4, F5; S-0312 claim 10; S-0614 claim 11): the
/// naval vehicle types counted, then two vessels of one type that a killing
/// hit sinks, with body facings 0 and 16384 (8-directions 0 and 2: stern
/// down and bow down), and an underwater one set sinking directly, as a
/// cull would. Frames of the whole view and a crop centred on each vessel
/// before the hit and after descents 1, 40, 79 and 80. Counts, ids, ticks
/// and the simulation's pitch only.
fn sinking_list(lane: &mut Lane) {
    use glasnost_app::mapview::build::Attitude;
    let types: Vec<NavalType> = {
        let s = lane.c.session.as_ref().unwrap();
        let import = &s.loaded.import;
        let registry = &import.outcome.registries[&glasnost_convert::kinds::Kind::Vehicles];
        let r = lane.world().rules();
        let threshold = r.combat.general.sinking_weight;
        let mode = r.mode;
        (0..r.vehicles.len())
            .filter_map(|i| {
                let u = r.unit(Kind::Vehicle, i as u32)?;
                if !u.combat.naval {
                    return None;
                }
                let name = r.vehicles[i].name.as_bytes();
                let voxel = registry
                    .iter()
                    .position(|t| t.name.eq_ignore_ascii_case(name))
                    .and_then(|x| {
                        import
                            .arts
                            .get(&(glasnost_convert::kinds::Kind::Vehicles, x))
                    })
                    .is_some_and(|a| a.voxels.contains_key("body"));
                let heavy = u
                    .vehicle
                    .weight
                    .unwrap_or_else(|| glasnost_num::F64::from_i64(1, mode))
                    .ge(threshold);
                let sinks = !u.combat.underwater && !u.combat.organic && heavy;
                Some((i as u32, voxel, u.turret, u.combat.underwater, sinks))
            })
            .collect()
    };
    let count = |f: &dyn Fn(&NavalType) -> bool| types.iter().filter(|t| f(t)).count();
    println!(
        "sinking-list: naval vehicle types {}: voxel body {}, sprite body {}; underwater {} (voxel {}); sink at a killing hit {} (voxel {}, with a turret {})",
        types.len(),
        count(&|t| t.1),
        count(&|t| !t.1),
        count(&|t| t.3),
        count(&|t| t.3 && t.1),
        count(&|t| t.4),
        count(&|t| t.4 && t.1),
        count(&|t| t.4 && t.1 && t.2),
    );
    let ship = types
        .iter()
        .find(|t| t.4 && t.1 && t.2)
        .or_else(|| types.iter().find(|t| t.4 && t.1))
        .map(|t| t.0);
    let sub = types.iter().find(|t| t.3 && t.1).map(|t| t.0);
    let (Some(ship), Some(sub)) = (ship, sub) else {
        println!("sinking-list: no voxel vessel that sinks at a hit, or no voxel underwater one");
        return;
    };
    let mut found = None;
    for map in 0..env_num("GLASNOST_LANE_MAPS", 8) {
        let l = Lane::start_on(map).expect("the install is set");
        let home = l.home();
        if let Some(at) = l.area(home, (7, 2), 2, |x| x == land::WATER) {
            println!("sinking-list: map {map}, water at ({}, {})", at.x, at.y);
            found = Some((l, at));
            break;
        }
    }
    let Some((mut l, at)) = found else {
        println!("sinking-list: no open water on the maps tried");
        return;
    };
    let local = l.local();
    let ship_name = type_name_of(l.world(), Kind::Vehicle, ship as usize);
    let sub_name = type_name_of(l.world(), Kind::Vehicle, sub as usize);
    let cells = [
        at,
        CellXY::new(at.x + 3, at.y),
        CellXY::new(at.x + 6, at.y + 1),
    ];
    let stern = glasnost_sim::testkit::spawn(
        l.world_mut(),
        Kind::Vehicle,
        &ship_name,
        local.0,
        cells[0],
        0,
    );
    let bow = glasnost_sim::testkit::spawn(
        l.world_mut(),
        Kind::Vehicle,
        &ship_name,
        local.0,
        cells[1],
        16384,
    );
    let under = glasnost_sim::testkit::spawn(
        l.world_mut(),
        Kind::Vehicle,
        &sub_name,
        local.0,
        cells[2],
        40960,
    );
    l.look_at(CellXY::new(at.x + 3, at.y + 1));
    l.ticks(3);
    let vessels = [
        ("facing-0", stern),
        ("facing-16384", bow),
        ("underwater", under),
    ];
    let report = |l: &Lane, what: &str| {
        let w = l.world();
        for (name, id) in vessels {
            let Some(o) = w.obj(id).filter(|o| o.on_map) else {
                println!("sinking-list {what}: {name} id {} out of play", id.0);
                continue;
            };
            let height = o.pos.z - w.surface_at(o.pos.x, o.pos.y);
            let lean = match glasnost_app::mapview::live::attitude(w, o) {
                Attitude::Sinking { pitch, roll, drawn } => format!(
                    "pitch {} (bits {pitch:#010x}), roll {}, lean {:?}, drawn {drawn}",
                    f32::from_bits(pitch),
                    f32::from_bits(roll),
                    o.naval.as_ref().and_then(|n| n.lean),
                ),
                _ => "level (not sinking)".to_string(),
            };
            println!(
                "sinking-list {what} (tick {}): {name} id {} type {} cell ({}, {}) facing {} (8-direction {}) sinking {} submerged {} height {height}: {lean}",
                w.tick,
                id.0,
                o.type_index,
                o.pos.cell().x,
                o.pos.cell().y,
                o.body.value(w.tick),
                glasnost_num::geom::eight_direction(o.body.value(w.tick)),
                glasnost_sim::naval::sinking(w, id),
                glasnost_sim::naval::submerged(w, id),
            );
        }
    };
    // The view stays put; each crop is centred on the vessel's own drawn
    // point (its projected position, lifted by its height).
    let frames = |l: &mut Lane, mark: &str| {
        l.save(&format!("sinking-list-{mark}"));
        let rgba = l.c.frame_rgba();
        let size = (l.c.size.0 as usize, l.c.size.1 as usize);
        let s = l.c.session.as_ref().unwrap();
        let (r, c) = (s.map_rect(), s.centre());
        for (name, id) in vessels {
            let Some(o) = l.world().obj(id).filter(|o| o.on_map) else {
                continue;
            };
            let (px, py) = glasnost_render::projection::project_lifted(o.pos.x, o.pos.y, o.pos.z);
            let at = (r.w / 2 + px - c.0, r.h / 2 + py - c.1);
            let (cw, ch, crop) = crop_at(&rgba, size, at, 120, 100);
            let file = format!("sinking-list-{mark}-{name}.png");
            std::fs::write(l.dir.join(&file), png::png(cw, ch, &crop)).unwrap();
            println!("wrote {file} (centred on {}, {})", at.0, at.1);
        }
    };
    report(&l, "before");
    frames(&mut l, "0-before");
    let h = warhead_of_kind(l.world(), 0).unwrap_or(0);
    kill_with(&mut l, stern, h);
    kill_with(&mut l, bow, h);
    glasnost_sim::naval::start_sinking(l.world_mut(), under);
    let t0 = l.world().tick;
    report(&l, "hit");
    // Frames after descents 1, 40, 79 (the full lean, F4) and 80 (held); a
    // vessel from height 0 leaves play at the 81st (S-0504 claim 15).
    let marks = [(1, "1"), (40, "2"), (79, "3"), (80, "4")];
    let mut next = 0;
    let mut descents = 0;
    let mut z = l.world().obj(bow).map(|o| o.pos.z);
    for _ in 0..200 {
        l.ticks(1);
        let now = l.world().obj(bow).filter(|o| o.on_map).map(|o| o.pos.z);
        let Some(now) = now else {
            println!(
                "sinking-list: the facing-16384 vessel left play {} ticks after the hit, after {descents} descents",
                l.world().tick - t0
            );
            break;
        };
        if z.is_some_and(|z| now < z) {
            descents += 1;
        }
        z = Some(now);
        if next < marks.len() && descents == marks[next].0 {
            let what = format!(
                "descent {} ({} ticks after the hit)",
                marks[next].0,
                l.world().tick - t0
            );
            report(&l, &what);
            frames(
                &mut l,
                &format!("{}-descent-{}", marks[next].1, marks[next].0),
            );
            next += 1;
        }
    }
    let w = l.world();
    println!(
        "sinking-list: after the descent, in play: facing-0 {}, facing-16384 {}, underwater {}",
        w.obj(stern).is_some_and(|o| o.on_map),
        w.obj(bow).is_some_and(|o| o.on_map),
        w.obj(under).is_some_and(|o| o.on_map)
    );
}

/// S-0108 claims 13 and 28 (Q-1890 answered by r81): a map building placed
/// switched off, given to the local house and switched on. See the module
/// docs. Counts, frames and ticks only.
fn placed_off(_: &mut Lane) {
    let maps = env_num("GLASNOST_LANE_MAPS", 8);
    // First pass: a building with a slot counting freeze-unpowered; else, the first with any
    // running slot.
    let mut fallback = None;
    for map in 0..maps {
        let Some((l, freeze, any)) = placed_off_start(map, true) else {
            break;
        };
        if let Some(b) = freeze {
            return placed_off_play(l, map, b);
        }
        if fallback.is_none() && any.is_some() {
            fallback = Some(map);
        }
    }
    let Some(map) = fallback else {
        println!("placed-off: no map with such a building that has slot animations");
        return;
    };
    println!(
        "placed-off: no such building with a slot counting freeze-unpowered; map {map}'s first with running slots"
    );
    if let Some((l, _, Some(b))) = placed_off_start(map, false) {
        placed_off_play(l, map, b);
    }
}

/// Starts map `map` with its non-player map buildings that the map may
/// switch off switched off before the first tick (when no entry has the
/// switched-on field 0; else as the map leaves them), printing the counts
/// when `print`. Returns the lane, the first such building with a running
/// slot counting freeze-unpowered, and the first with any running slot;
/// `None` past the last offered map.
fn placed_off_start(map: usize, print: bool) -> Option<(Lane, Option<ObjectId>, Option<ObjectId>)> {
    let mut natural = 0;
    let mut forced: Vec<ObjectId> = Vec::new();
    let l = Lane::start_hooked(map, |l| {
        let s = l.c.session.as_ref().unwrap();
        natural = s
            .loaded
            .map
            .objects
            .structure_fields
            .iter()
            .filter(|f| !f.switched_on())
            .count();
        let live = s.loaded.live.as_ref().expect("a live game");
        let ids: Vec<ObjectId> = live.building_ids.values().copied().collect();
        let w = l.world();
        let mut fit: Vec<(bool, ObjectId)> = ids
            .into_iter()
            .filter_map(|id| {
                let o = w.obj(id)?;
                let role = o.owner.and_then(|h| w.houses.get(usize::from(h.0)))?.role;
                let t = w.rules().econ.power.buildings.get(o.type_index as usize)?;
                (o.on_map
                    && role != glasnost_sim::world::HouseRole::Player
                    && glasnost_sim::econ::power::map_off_applies(w, o.type_index))
                .then_some((t.needs_engineer, id))
            })
            .collect();
        fit.sort();
        if natural == 0 {
            for (_, id) in &fit {
                if let Some(o) = l.world_mut().obj_mut(*id) {
                    o.econ.building.switched_off = true;
                }
            }
        }
        forced = fit.into_iter().map(|(_, id)| id).collect();
    })?;
    if map >= l.c.install.as_ref().map_or(0, |i| i.maps.len()) {
        return None;
    }
    let doc = art_doc(&l);
    let mut seen = (0, 0, 0);
    let (mut freeze_pick, mut any_pick) = (None, None);
    for &id in &forced {
        let Some(o) = l.world().obj(id) else { continue };
        let (off, t) = (o.econ.building.switched_off, o.type_index);
        let running = {
            let s = l.c.session.as_ref().unwrap();
            !s.loaded.live.as_ref().unwrap().slot_names(id).is_empty()
        };
        let freeze =
            slot_counting(&l, &doc, t).contains(&glasnost_app::mapview::slots::Counting::Freeze);
        seen.0 += usize::from(off);
        seen.1 += usize::from(off && running);
        seen.2 += usize::from(off && running && freeze);
        if off && running {
            any_pick = any_pick.or(Some(id));
            if freeze {
                freeze_pick = freeze_pick.or(Some(id));
            }
        }
    }
    if print {
        println!(
            "placed-off map {map}: entries with the switched-on field 0 {natural}, non-player map buildings the map may switch off {} ({}); switched off {}, with running slots {}, a slot counting freeze-unpowered {}",
            forced.len(),
            if natural == 0 {
                "switched off by the lane"
            } else {
                "as the map leaves them"
            },
            seen.0,
            seen.1,
            seen.2
        );
    }
    Some((l, freeze_pick, any_pick))
}

/// The scene on building `b` of map `map`: fresh, held, given to the local
/// house, the house made short of power, switched on.
fn placed_off_play(mut l: Lane, map: usize, b: ObjectId) {
    use glasnost_sim::structures::ownership::{Cause, change};
    let doc = art_doc(&l);
    if let Some(t) = l.world().obj(b).map(|o| o.type_index) {
        println!(
            "placed-off map {map}: its set-up slots' counting power flags {:?} (S-0108 claim 13)",
            slot_counting(&l, &doc, t)
        );
    }
    let cell = l.world().obj(b).map_or(CellXY::new(0, 0), |o| o.pos.cell());
    l.look_at(cell);
    placed_off_state(&mut l, map, "fresh", b);
    l.save(&format!("placed-off-{map}-fresh"));
    l.ticks(20);
    placed_off_state(&mut l, map, "20 ticks later", b);
    l.save(&format!("placed-off-{map}-held"));
    let local = l.local();
    change(l.world_mut(), b, Some(local), Cause::Trigger);
    l.ticks(20);
    placed_off_state(&mut l, map, "given to the local house", b);
    l.save(&format!("placed-off-{map}-given"));
    // The local house short of power: a building of its own that needs and draws power.
    let drawer =
        {
            let r = l.world().rules();
            (0..r.buildings.len() as u32).find(|&t| {
                r.econ.power.buildings.get(t as usize).is_some_and(|p| {
                    p.needs_power && p.drain > 0 && p.output == 0 && !p.needs_engineer
                })
            })
        };
    if let Some(t) = drawer {
        let name = l.world().rules().buildings[t as usize].name.clone();
        let home = l.home();
        let spot = l.open_land(CellXY::new(home.x + 6, home.y + 6));
        l.spawn(Kind::Building, &name, local, spot);
        glasnost_sim::econ::power::make_due(l.world_mut(), local);
        l.ticks(5);
    }
    placed_off_state(&mut l, map, "the house made short of power", b);
    let ok = glasnost_sim::orders::issue(
        l.world_mut(),
        local,
        glasnost_sim::orders::Order::PowerOn { actor: b },
    );
    println!("placed-off map {map}: power-on order accepted {ok}");
    for t in 0..4 {
        placed_off_state(
            &mut l,
            map,
            &format!("switch-on, tick {t} from the order"),
            b,
        );
    }
    l.ticks(20);
    placed_off_state(&mut l, map, "switched on, 20 ticks later", b);
    l.save(&format!("placed-off-{map}-switched-on"));
}

/// One tick, then one line: owner role, switched off, in operation, the
/// house's power (output, drain), running slots with their frames, frozen.
fn placed_off_state(l: &mut Lane, map: usize, what: &str, b: ObjectId) {
    l.ticks(1);
    let _ = l.c.frame_rgba();
    let s = l.c.session.as_ref().unwrap();
    let live = s.loaded.live.as_ref().expect("a live game");
    let w = l.world();
    let Some(o) = w.obj(b) else {
        println!("placed-off map {map} {what}: gone");
        return;
    };
    let role = o
        .owner
        .and_then(|h| w.houses.get(usize::from(h.0)))
        .map(|h| h.role);
    let power = o.owner.map(|h| glasnost_sim::econ::power::totals(w, h));
    println!(
        "placed-off map {map} {what} (tick {}): owner role {role:?}, needs engineer {}, switched off {}, in operation {}, house power {power:?}, slots {:?}, frozen {}",
        w.tick,
        w.rules()
            .econ
            .power
            .buildings
            .get(o.type_index as usize)
            .is_some_and(|t| t.needs_engineer),
        o.econ.building.switched_off,
        glasnost_sim::econ::power::in_operation(w, b),
        live.slot_names(b),
        live.frozen_slots(b),
    );
}

/// The counting power flags of a building type's set-up slots that name an
/// animation (S-0108 claim 13 and Import mapping).
fn slot_counting(
    l: &Lane,
    doc: &glasnost_ini::Document,
    t: u32,
) -> Vec<glasnost_app::mapview::slots::Counting> {
    use glasnost_app::mapview::slots;
    let Some((image, _)) = type_look(l, doc, t) else {
        return Vec::new();
    };
    slots::SETUP_SLOTS
        .iter()
        .map(|base| slots::read_slot(doc, &image, base))
        .filter(|d| !d.normal.is_empty())
        .map(|d| d.power.counting())
        .collect()
}

// ---------------------------------------------------------------------------
// M8 step A3 intake 53 (S-0113 body rocking; S-0108 r102 claims 11 and 18)

/// A vehicle type with a voxel body whose locomotor draws rocking and that
/// is not naval: its index.
fn rocking_vehicle(l: &Lane) -> Option<usize> {
    let s = l.c.session.as_ref().unwrap();
    let import = &s.loaded.import;
    let registry = &import.outcome.registries[&glasnost_convert::kinds::Kind::Vehicles];
    let r = l.world().rules();
    (0..r.vehicles.len()).find(|&i| {
        let Some(u) = r.unit(Kind::Vehicle, i as u32) else {
            return false;
        };
        let name = r.vehicles[i].name.as_bytes();
        let voxel = registry
            .iter()
            .position(|t| t.name.eq_ignore_ascii_case(name))
            .and_then(|x| {
                import
                    .arts
                    .get(&(glasnost_convert::kinds::Kind::Vehicles, x))
            })
            .is_some_and(|a| a.voxels.contains_key("body"));
        voxel
            && !u.combat.naval
            && u.strength > 0
            && matches!(u.type_locomotor(), glasnost_rules::sim::Locomotor::Drive)
    })
}

/// A crop of the current frame centred on each object's drawn point,
/// written as `{key}-{mark}-{name}.png`.
fn crops(l: &mut Lane, key: &str, mark: &str, objects: &[(&str, ObjectId)]) {
    let rgba = l.c.frame_rgba();
    let size = (l.c.size.0 as usize, l.c.size.1 as usize);
    let s = l.c.session.as_ref().unwrap();
    let (r, c) = (s.map_rect(), s.centre());
    for &(name, id) in objects {
        let Some(o) = l.world().obj(id).filter(|o| o.on_map) else {
            continue;
        };
        let (px, py) = glasnost_render::projection::project_lifted(o.pos.x, o.pos.y, o.pos.z);
        let at = (r.w / 2 + px - c.0, r.h / 2 + py - c.1);
        let (cw, ch, crop) = crop_at(&rgba, size, at, 120, 100);
        let file = format!("{key}-{mark}-{name}.png");
        std::fs::write(l.dir.join(&file), png::png(cw, ch, &crop)).unwrap();
        println!("wrote {file}");
    }
}

/// S-0113 (claims 1, 5 to 7, 11 and 18; F1, F2): three vehicles of a voxel
/// type with the driving locomotor, all facing 0 (map north), one cell
/// north, east and west of a point *P*, and a detonation of a warhead with
/// the rocking flag and a spread above 0 at *P* with no source object
/// (found by its flags; when no stock warhead has both, the lane's copy of
/// the rules sets the flag on one with a spread). *P* is behind the north
/// one (its bow goes down), on the east one's left and on the west one's
/// right (the nearer side rises). Frames of the view and a crop of each
/// vehicle after 1, 3, 6, 12, 23, 30 and 45 ticks; the view's angles and
/// rates per vehicle at each, as bits and values. Ids, ticks and angles
/// only.
fn body_rocking(l: &mut Lane) {
    let Some(v) = rocking_vehicle(l) else {
        println!("body-rocking: no voxel vehicle type with the driving locomotor");
        return;
    };
    let warhead = {
        let r = l.world().rules();
        let both = r
            .combat
            .warheads
            .iter()
            .position(|h| h.rocker && h.spread.gt(glasnost_num::F64::ZERO));
        let spread = r
            .combat
            .warheads
            .iter()
            .position(|h| h.spread.gt(glasnost_num::F64::ZERO));
        println!(
            "body-rocking: a stock warhead with the rocking flag and a spread: {}",
            both.is_some()
        );
        both.map(|i| (i, false)).or(spread.map(|i| (i, true)))
    };
    let Some((h, patched)) = warhead else {
        println!("body-rocking: no warhead with a spread");
        return;
    };
    if patched {
        patch_rules(l, |r| r.combat.warheads[h].rocker = true);
        println!("body-rocking: the lane's copy of the rules sets the rocking flag on one warhead");
    }
    let local = l.local();
    let home = l.home();
    let Some(at) = l.area(home, (3, 3), 2, |x| x != land::WATER && x != land::ROCK) else {
        println!("body-rocking: no open land");
        return;
    };
    let p_cell = CellXY::new(at.x + 1, at.y + 1);
    let name = type_name_of(l.world(), Kind::Vehicle, v);
    let place = |l: &mut Lane, dx: i32, dy: i32| {
        glasnost_sim::testkit::spawn(
            l.world_mut(),
            Kind::Vehicle,
            &name,
            local.0,
            CellXY::new(p_cell.x + dx, p_cell.y + dy),
            0,
        )
    };
    let north = place(l, 0, -1);
    let east = place(l, 1, 0);
    let west = place(l, -1, 0);
    let bodies = [("p-behind", north), ("p-left", east), ("p-right", west)];
    l.look_at(p_cell);
    l.ticks(3);
    crops(l, "body-rocking", "0-before", &bodies);
    l.save("body-rocking-0-before");
    let (x, y) = glasnost_render::projection::cell_centre(p_cell.x, p_cell.y);
    let z = l.world().surface_at(x, y);
    let point = glasnost_sim::world::Pos::new(x, y, z);
    glasnost_sim::combat::area::detonate(
        l.world_mut(),
        glasnost_sim::combat::area::Detonation {
            point,
            damage: 200,
            warhead: Some(h as u32),
            source: None,
            house: None,
            resources: false,
        },
    );
    let t0 = l.world().tick;
    let report = |l: &Lane, after: i32| {
        let s = l.c.session.as_ref().unwrap();
        let rocking = s.loaded.live.as_ref().expect("a live game").rocking();
        for (name, id) in bodies {
            match rocking.body(id) {
                Some(b) => println!(
                    "body-rocking +{after} (tick {}): {name} id {}: pitch {} ({:#010x}) roll {} ({:#010x}) pitch rate {} roll rate {}",
                    l.world().tick,
                    id.0,
                    f32::from_bits(b.pitch.0),
                    b.pitch.0,
                    f32::from_bits(b.roll.0),
                    b.roll.0,
                    f32::from_bits(b.pitch_rate.0),
                    f32::from_bits(b.roll_rate.0),
                ),
                None => println!(
                    "body-rocking +{after} (tick {}): {name} id {}: level",
                    l.world().tick,
                    id.0
                ),
            }
        }
    };
    for after in [1, 3, 6, 12, 23, 30, 45] {
        while l.world().tick - t0 < after {
            l.ticks(1);
        }
        report(l, after);
        let mark = format!("{after:02}");
        crops(l, "body-rocking", &mark, &bodies);
        l.save(&format!("body-rocking-{mark}"));
    }
}

/// S-0108 r102 claims 11 and 18 (`quirks.ungranted_super_reads_clock`): the
/// superweapon setting off, so a disableable superweapon type is never
/// granted (S-0512 claims 3 and 7), and one building of each type that gives one placed for
/// the local house with power: its slots 60 ticks after its opening, with
/// the switch on (the original: the clock of a record never granted reads
/// 0, so *super-3*) and off (the play-fixes layer: *super-1*). The lane
/// plays with the layer on, so each run sets the switch in the lane's copy
/// of the rules. Slot names, counts and ticks only.
fn super_ungranted(_: &mut Lane) {
    for reads in [true, false] {
        let Some(mut l) = Lane::start() else { return };
        let local = l.local();
        let picked = {
            let r = l.world().rules();
            let sw: Vec<(usize, u32)> = (0..r.buildings.len())
                .filter_map(|i| {
                    let s = r.superweapons.building_sw.get(i)?[0]?;
                    r.superweapons
                        .types
                        .get(s as usize)
                        .is_some_and(|t| t.disableable)
                        .then_some((i, s))
                })
                .collect();
            let power = (0..r.buildings.len())
                .filter(|&i| {
                    r.superweapons
                        .building_sw
                        .get(i)
                        .is_none_or(|s| s[0].is_none())
                        && r.buildings[i].raw.econ.power.unwrap_or(0) > 0
                })
                .max_by_key(|&i| r.buildings[i].raw.econ.power.unwrap_or(0));
            power.filter(|_| !sw.is_empty()).map(|p| (sw, p))
        };
        let Some((sws, p)) = picked else {
            println!(
                "super-ungranted: no disableable superweapon building type, or no power plant"
            );
            return;
        };
        patch_rules(&mut l, |r| r.quirks.ungranted_super_reads_clock = reads);
        l.world_mut().econ.settings.superweapons = Some(false);
        let home = l.home();
        let power_name = type_name_of(l.world(), Kind::Building, p);
        let fp = l.world().rules().buildings[p].foundation;
        for _ in 0..(4 * sws.len()) {
            if let Some(at) = l.area(home, fp, 1, |x| x != land::WATER && x != land::ROCK) {
                l.spawn(Kind::Building, &power_name, local, at);
            }
        }
        let mut placed = Vec::new();
        for &(b, sw) in &sws {
            let name = type_name_of(l.world(), Kind::Building, b);
            let fb = l.world().rules().buildings[b].foundation;
            let Some(at) = l.area(home, fb, 1, |x| x != land::WATER && x != land::ROCK) else {
                println!("super-ungranted: no free place for {name}");
                continue;
            };
            placed.push((l.spawn(Kind::Building, &name, local, at), sw, at));
        }
        l.ticks(60);
        let on = if reads { "on" } else { "off" };
        for (k, &(id, sw, at)) in placed.iter().enumerate() {
            let names = {
                let s = l.c.session.as_ref().unwrap();
                s.loaded.live.as_ref().expect("a live game").slot_names(id)
            };
            let slots: Vec<&str> = names
                .iter()
                .map(|n| n.split('=').next().unwrap_or(""))
                .collect();
            let granted =
                glasnost_sim::superweapons::record(l.world(), local, sw).map(|r| r.granted);
            println!(
                "super-ungranted (switch {on}): building {k} id {}, tick {}, record granted {granted:?}, running slots {slots:?}",
                id.0,
                l.world().tick
            );
            let key = format!("super-ungranted-{on}-sw{k}");
            l.look_at(at);
            l.ticks(1);
            crops(&mut l, &key, "60", &[("building", id)]);
            l.save(&key);
        }
    }
}

// ---------------------------------------------------------------------------
// M8 step A4 round 8 (the lead's notes 44, 50 and 53)

/// An armed vehicle type that grapples: naval and organic (S-0614 Terms).
fn grappler(w: &World) -> Option<String> {
    let r = w.rules();
    (0..r.vehicles.len())
        .find(|&i| {
            r.unit(Kind::Vehicle, i as u32).is_some_and(|u| {
                u.combat.naval && u.combat.organic && u.combat.slots[0].is_some() && u.strength > 0
            })
        })
        .map(|i| r.vehicles[i].name.clone())
}

/// A short word for a destination or target.
fn target_word(w: &World, t: Option<glasnost_sim::world::Target>) -> String {
    use glasnost_sim::world::Target;
    match t {
        None => "none".into(),
        Some(Target::Cell(c)) => {
            let usable = w.cells.usable(c);
            format!("cell ({}, {}) usable {usable}", c.x, c.y)
        }
        Some(Target::Object(o)) => format!("object #{}", o.0),
    }
}

/// Note 44: a carrier ship's slot aircraft while an enemy grappling
/// parasite holds the carrier (run 1) and kills it (run 2); run 0 is the
/// control without the parasite. In runs 3 and 4 the grappler is itself the
/// carrier's target: it grapples once a slot aircraft attacks it, and in
/// run 4 the carrier is killed. In run 5 the grappler comes once a slot
/// aircraft is 15 cells out. Every slot aircraft is followed for 1500
/// ticks: the changes of its slot state, behaviour and destination, and
/// how it leaves play (docked, dead, or alive outside the usable area).
fn r8_carrier_squid(_lane: &mut Lane) {
    let probe = Lane::start_on(0).expect("the install is set");
    let pick = {
        let w = probe.world();
        let r = w.rules();
        r.missiles
            .spawners
            .iter()
            .find(|((k, ti), sr)| {
                *k == Kind::Vehicle
                    && r.missiles.set_of(sr.spawned).is_none()
                    && r.unit(*k, *ti)
                        .is_some_and(|u| u.locomotor == glasnost_sim::rules::Locomotor::Ship)
            })
            .map(|(k, v)| (*k, *v))
    };
    let squid = grappler(probe.world());
    let victim_name = plain_vehicle(probe.world());
    drop(probe);
    let (Some(((kind, ti), sr)), Some(squid), Some(victim_name)) = (pick, squid, victim_name)
    else {
        println!("r8-carrier-squid: no carrier ship, grappler or plain vehicle type");
        return;
    };
    for run in 0..6 {
        let mut found = None;
        for map in 0..env_num("GLASNOST_LANE_MAPS", 8) {
            let l = Lane::start_on(map).expect("the install is set");
            let home = l.home();
            if let Some(at) = l.area(home, (1, 1), 3, |x| x == land::WATER) {
                found = Some((l, at, map));
                break;
            }
        }
        let Some((mut l, at, map)) = found else {
            println!("r8-carrier-squid: no open water on the maps tried");
            return;
        };
        let Some(enemy) = l.enemy() else { return };
        let local = l.local();
        let name = type_name_of(l.world(), kind, ti as usize);
        let car = l.spawn(kind, &name, local, at);
        // Runs 3 and 4: the carrier's own target is the grappler, on water
        // beside it (a person's carrier that answers the parasite).
        let wet = l
            .area(CellXY::new(at.x + 3, at.y), (1, 1), 0, |x| x == land::WATER)
            .unwrap_or(CellXY::new(at.x + 3, at.y));
        let near = if run >= 3 {
            wet
        } else {
            l.open_land(CellXY::new(at.x + 6, at.y))
        };
        let victim = if run >= 3 {
            l.spawn(Kind::Vehicle, &squid, enemy, near)
        } else {
            l.spawn(Kind::Vehicle, &victim_name, enemy, near)
        };
        if let Some(o) = l.world_mut().obj_mut(victim) {
            o.health = i32::MAX / 4;
        }
        l.ticks(2);
        let orders = l.order(local, car, ClickOn::Object(victim));
        let attacking = |w: &World| {
            w.obj(car).and_then(|o| o.spawn.as_ref()).is_some_and(|c| {
                c.slots
                    .iter()
                    .any(|s| s.state == glasnost_sim::spawn::SlotState::Attacking)
            })
        };
        let waited = until(&mut l, 1500, attacking);
        println!(
            "r8-carrier-squid run {run}: map {map}, carrier #{} at ({}, {}), victim #{} at ({}, {}), order {orders}, ticks to a slot attacking {waited:?}",
            car.0, at.x, at.y, victim.0, near.x, near.y
        );
        let mut sq = None;
        if run >= 3 {
            let n = l.order(enemy, victim, ClickOn::Object(car));
            println!("r8-carrier-squid run {run}: the grappler is the carrier's target, order {n}");
            sq = Some(victim);
        } else if run == 1 || run == 2 {
            let s = l.spawn(Kind::Vehicle, &squid, enemy, wet);
            let n = l.order(enemy, s, ClickOn::Object(car));
            println!(
                "r8-carrier-squid run {run}: grappler #{} at ({}, {}), order {n}",
                s.0, wet.x, wet.y
            );
            sq = Some(s);
        }
        let t0 = l.world().tick;
        let mut held_at = None;
        let mut killed_at = None;
        // Per aircraft of the spawned type: the last state line, how it left.
        let mut last: BTreeMap<ObjectId, String> = BTreeMap::new();
        let mut gone: BTreeMap<ObjectId, String> = BTreeMap::new();
        let mut far: BTreeMap<ObjectId, i32> = BTreeMap::new();
        let mut lines = 0usize;
        let mut seen: BTreeMap<ObjectId, (bool, i32, CellXY)> = BTreeMap::new();
        for _ in 0..1500 {
            l.ticks(1);
            let w = l.world();
            let dt = w.tick - t0;
            if held_at.is_none() && glasnost_sim::parasite::held(w, car) {
                held_at = Some(dt);
                println!("r8-carrier-squid run {run}: +{dt} the carrier is held");
            }
            // Run 5: the grappler comes once a slot aircraft is far out.
            if run == 5 && sq.is_none() && far.values().any(|&d| d >= 15) {
                let s = l.spawn(Kind::Vehicle, &squid, enemy, wet);
                let n = l.order(enemy, s, ClickOn::Object(car));
                println!(
                    "r8-carrier-squid run {run}: +{dt} grappler #{} at ({}, {}) with the aircraft far out, order {n}",
                    s.0, wet.x, wet.y
                );
                sq = Some(s);
            }
            if (run == 2 || run == 4)
                && killed_at.is_none()
                && held_at.is_some_and(|h| dt >= h + 40)
            {
                killed_at = Some(dt);
                let h = warhead_of_kind(l.world(), 0).unwrap_or(0);
                kill_with(&mut l, car, h);
                println!("r8-carrier-squid run {run}: +{dt} the carrier is killed");
            }
            let w = l.world();
            let slots: Vec<(Option<ObjectId>, glasnost_sim::spawn::SlotState)> = w
                .obj(car)
                .and_then(|o| o.spawn.as_ref())
                .map(|c| c.slots.iter().map(|s| (s.object, s.state)).collect())
                .unwrap_or_default();
            let carrier_cell = w.obj(car).map(|o| o.pos.cell());
            for o in w.objects.values() {
                if o.kind != Kind::Aircraft || o.type_index != sr.spawned || !o.on_map {
                    continue;
                }
                let slot = slots
                    .iter()
                    .find(|(x, _)| *x == Some(o.id))
                    .map_or("detached", |(_, s)| slot_word(*s));
                let visiting = o.air.as_ref().is_some_and(|a| a.visiting);
                let line = format!(
                    "slot {slot}, behaviour {:?} (queued {:?}), destination {}, target {}, ammunition {}, health {}, visiting {visiting}",
                    o.beh.current,
                    o.beh.queued,
                    target_word(w, o.dest),
                    target_word(w, o.target),
                    o.combat.ammo,
                    o.health
                );
                let c = o.pos.cell();
                seen.insert(o.id, (o.health > 0, o.health, c));
                if let Some(cc) = carrier_cell {
                    let d = (c.x - cc.x).abs().max((c.y - cc.y).abs());
                    let e = far.entry(o.id).or_insert(0);
                    *e = (*e).max(d);
                }
                if last.get(&o.id) != Some(&line) {
                    if lines < 120 {
                        println!(
                            "r8-carrier-squid run {run}: +{dt} aircraft #{} at ({}, {}) usable {}: {line}",
                            o.id.0,
                            c.x,
                            c.y,
                            w.cells.usable(c)
                        );
                    }
                    lines += 1;
                    last.insert(o.id, line);
                }
            }
            let ids: Vec<ObjectId> = seen.keys().copied().collect();
            for id in ids {
                if gone.contains_key(&id) {
                    continue;
                }
                let here = w.obj(id).is_some_and(|o| o.on_map);
                if here {
                    continue;
                }
                let (alive, _, c) = seen[&id];
                let docked = slots.iter().any(|(x, s)| {
                    *x == Some(id) && *s == glasnost_sim::spawn::SlotState::Reloading
                });
                let how = if docked {
                    "docked in the carrier".to_string()
                } else if alive && !w.cells.usable(c) {
                    format!(
                        "left play alive outside the usable area at ({}, {})",
                        c.x, c.y
                    )
                } else if alive {
                    format!(
                        "left play alive inside the usable area at ({}, {})",
                        c.x, c.y
                    )
                } else {
                    "left play dead".to_string()
                };
                println!("r8-carrier-squid run {run}: +{dt} aircraft #{} {how}", id.0);
                gone.insert(id, how);
            }
            // A docked aircraft comes back as the same object: forget it.
            gone.retain(|id, how| {
                !(how.starts_with("docked") && w.obj(*id).is_some_and(|o| o.on_map))
            });
        }
        let alive_off = gone
            .values()
            .filter(|h| h.starts_with("left play alive outside"))
            .count();
        println!(
            "r8-carrier-squid run {run}: carrier in play {}, held from {held_at:?}, killed at {killed_at:?}, grappler {:?}; aircraft followed {}, left play alive outside the usable area {alive_off}, furthest from the carrier in cells {far:?}; state lines {lines}",
            l.world().obj(car).is_some_and(|o| o.on_map),
            sq.map(|s| s.0),
            seen.len()
        );
    }
}

/// A pad-bound, armed aircraft type with ammunition, and its first pad type.
fn pad_aircraft(w: &World) -> Option<(String, String)> {
    let r = w.rules();
    (0..r.aircraft.len()).find_map(|i| {
        let u = r.unit(Kind::Aircraft, i as u32)?;
        if !u.raw.econ.airport_bound.unwrap_or(false)
            || u.combat.slots[0].is_none()
            || u.combat.ammo <= 0
            || u.strength <= 0
        {
            return None;
        }
        let pad = u.raw.econ.dock.as_ref()?.first()?.clone();
        r.index_of(Kind::Building, &pad)?;
        Some((r.aircraft[i].name.clone(), pad))
    })
}

/// Note 50: a pad-bound aircraft released at its pad with no ammunition;
/// run 0 is the control (no order), run 1 gets an attack order on its pad
/// as soon as it sleeps there; in run 2 it spends its rounds on a first
/// attack and gets a second order while it flies home spent. Counts
/// the heights, behaviours, take-offs and landings, and the ticks to half
/// and full ammunition.
fn r8_pad_no_ammo(_lane: &mut Lane) {
    for run in 0..3 {
        let mut l = Lane::start_on(0).expect("the install is set");
        let Some((plane_name, pad_name)) = pad_aircraft(l.world()) else {
            println!("r8-pad-no-ammo: no pad-bound armed aircraft type");
            return;
        };
        let Some(victim_name) = plain_vehicle(l.world()) else {
            return;
        };
        let Some(enemy) = l.enemy() else { return };
        let local = l.local();
        let home = l.home();
        let pi = l
            .world()
            .rules()
            .index_of(Kind::Building, &pad_name)
            .unwrap() as usize;
        let fp = l.world().rules().buildings[pi].foundation;
        let Some(pad_at) = l.area(CellXY::new(home.x + 8, home.y + 4), fp, 1, |x| {
            x != land::WATER && x != land::ROCK
        }) else {
            println!("r8-pad-no-ammo: no place for the pad");
            return;
        };
        let pad = l.spawn(Kind::Building, &pad_name, local, pad_at);
        l.ticks(2);
        // Made at its pad, as its factory releases it (S-0501 claim 24).
        let ai = l
            .world()
            .rules()
            .index_of(Kind::Aircraft, &plane_name)
            .unwrap();
        let plane = glasnost_sim::objects::create(l.world_mut(), Kind::Aircraft, ai, Some(local));
        let released = glasnost_sim::econ::release::release(l.world_mut(), pad, plane);
        let start = l.world().obj(plane).map_or(pad_at, |o| o.pos.cell());
        let max = l
            .world()
            .rules()
            .unit(Kind::Aircraft, ai)
            .map_or(0, |u| u.combat.ammo);
        // Every run spends its rounds on a first attack run; run 0 is then
        // ordered to attack again once it sleeps on its pad, run 2 while it
        // flies home with none, and run 1 is the control with no second
        // order.
        let vat = l.open_land(CellXY::new(pad_at.x - 14, pad_at.y + 10));
        let victim = l.spawn(Kind::Vehicle, &victim_name, enemy, vat);
        if let Some(o) = l.world_mut().obj_mut(victim) {
            o.health = i32::MAX / 4;
        }
        let first = l.order(local, plane, ClickOn::Object(victim));
        println!(
            "r8-pad-no-ammo run {run}: pad #{} at ({}, {}), aircraft #{} at ({}, {}) released {released:?}, half-reload switch {}, its full ammunition {max}, victim #{} at ({}, {}), first attack orders {first}",
            pad.0,
            pad_at.x,
            pad_at.y,
            plane.0,
            start.x,
            start.y,
            l.world().rules().quirks.aircraft_half_reload_with_target,
            victim.0,
            vat.x,
            vat.y
        );
        let t0 = l.world().tick;
        let mut ordered = None;
        let mut last = String::new();
        let mut spent_at = None;
        let mut full_at = None;
        let mut took_off = 0;
        let mut landed = 0;
        let mut was_ground = glasnost_sim::air::on_ground(l.world(), plane);
        let mut peak_after = 0;
        let mut slept_at = None;
        for _ in 0..4000 {
            l.ticks(1);
            let w = l.world();
            let dt = w.tick - t0;
            let Some(o) = w.obj(plane).filter(|o| o.on_map) else {
                println!(
                    "r8-pad-no-ammo run {run}: +{dt} the aircraft left play (object kept {})",
                    w.obj(plane).is_some()
                );
                break;
            };
            let h = glasnost_sim::air::height(w, plane);
            let ground = glasnost_sim::air::on_ground(w, plane);
            if ground != was_ground {
                if ground {
                    landed += 1;
                } else {
                    took_off += 1;
                }
                was_ground = ground;
            }
            if ordered.is_some() {
                peak_after = peak_after.max(h);
            }
            let ammo = o.combat.ammo;
            if spent_at.is_none() && ammo == 0 {
                spent_at = Some(dt);
            }
            if spent_at.is_some() && full_at.is_none() && ammo >= max {
                full_at = Some(dt);
            }
            let sleeping = o.beh.current == Some(glasnost_sim::rules::Behaviour::Sleep);
            if slept_at.is_none() && sleeping && spent_at.is_some() {
                slept_at = Some(dt);
            }
            let enter = o.beh.current == Some(glasnost_sim::rules::Behaviour::Enter);
            let now = match run {
                0 => slept_at.is_some() && ground && ammo < max && ordered.is_none(),
                2 => enter && !ground && h > 200 && ammo == 0 && ordered.is_none(),
                _ => false,
            };
            let line = format!(
                "behaviour {:?} (queued {:?}), on the ground {ground}, height band {}, ammunition {ammo}, target {}, destination {}",
                o.beh.current,
                o.beh.queued,
                h / 100,
                target_word(w, o.target),
                target_word(w, o.dest)
            );
            if line != last {
                println!("r8-pad-no-ammo run {run}: +{dt} {line}");
                last = line;
            }
            if now {
                let n = l.order(local, plane, ClickOn::Object(victim));
                println!(
                    "r8-pad-no-ammo run {run}: +{dt} attack order on the victim with ammunition {ammo}: orders {n}"
                );
                ordered = Some(dt);
            }
            if full_at.is_some() && dt > full_at.unwrap() + 60 {
                break;
            }
        }
        println!(
            "r8-pad-no-ammo run {run}: spent +{spent_at:?}, first sleep after +{slept_at:?}, order +{ordered:?}, take-offs {took_off}, landings {landed}, peak height after the order {peak_after}, full ammunition again +{full_at:?}"
        );
    }
}

/// A civilian building type that infantry can occupy and fire from, and
/// an armed occupier infantry type.
fn garrison_types(w: &World) -> Option<(String, String)> {
    let r = w.rules();
    let b = (0..r.buildings.len()).find(|&i| {
        let t = &r.buildings[i];
        t.building.can_be_occupied
            && t.building.can_occupy_fire
            && t.building.max_occupants >= 3
            && t.raw.tech_level == Some(-1)
            && t.foundation.0 <= 3
            && t.foundation.1 <= 3
    })?;
    let inf = (0..r.infantry.len()).find(|&i| {
        r.unit(Kind::Infantry, i as u32).is_some_and(|u| {
            u.raw.occupier.unwrap_or(false) && u.combat.slots[0].is_some() && u.strength > 0
        })
    })?;
    Some((r.buildings[b].name.clone(), r.infantry[inf].name.clone()))
}

/// An unarmed infantry type (an enemy in range that cannot shoot back).
fn unarmed_infantry(w: &World) -> Option<String> {
    let r = w.rules();
    (0..r.infantry.len())
        .find(|&i| {
            r.unit(Kind::Infantry, i as u32).is_some_and(|u| {
                u.combat.slots[0].is_none() && u.combat.slots[1].is_none() && u.strength > 0
            })
        })
        .map(|i| r.infantry[i].name.clone())
}

/// Note 53: three occupiers enter a civilian building; an enemy stands in
/// range before the entry (runs 0 and 2) or arrives after it (runs 1 and
/// 3); the enemy is unarmed (runs 0 and 1) or a plain armed vehicle (runs
/// 2 and 3). Counts the ticks from the last entry to the building's first
/// target and first hit, and whether the enemy hit the building first.
fn r8_garrison_fire(_lane: &mut Lane) {
    for run in 0..4 {
        let mut l = Lane::start_on(0).expect("the install is set");
        let Some((bname, iname)) = garrison_types(l.world()) else {
            println!("r8-garrison-fire: no garrison building or occupier type");
            return;
        };
        let armed_enemy = run >= 2;
        let enemy_name = if armed_enemy {
            plain_vehicle(l.world())
        } else {
            unarmed_infantry(l.world())
        };
        let Some(enemy_name) = enemy_name else {
            println!("r8-garrison-fire: no enemy type");
            return;
        };
        let enemy_kind = if armed_enemy {
            Kind::Vehicle
        } else {
            Kind::Infantry
        };
        let Some(enemy) = l.enemy() else { return };
        let Some(civ) = glasnost_sim::structures::garrison::civilian_house(l.world()) else {
            println!("r8-garrison-fire: no civilian house");
            return;
        };
        let local = l.local();
        let home = l.home();
        let bi = l.world().rules().index_of(Kind::Building, &bname).unwrap() as usize;
        let fp = l.world().rules().buildings[bi].foundation;
        let Some(tl) = l.area(CellXY::new(home.x + 10, home.y), fp, 3, |x| {
            x != land::WATER && x != land::ROCK
        }) else {
            println!("r8-garrison-fire: no place for the building");
            return;
        };
        let b = l.spawn(Kind::Building, &bname, civ, tl);
        let mut men = Vec::new();
        for k in 0..3 {
            let c = CellXY::new(tl.x - 2, tl.y + k);
            men.push(l.spawn(Kind::Infantry, &iname, local, c));
        }
        let ecell = CellXY::new(tl.x + fp.0 + 1, tl.y);
        let place_enemy = |l: &mut Lane| {
            let e = l.spawn(enemy_kind, &enemy_name, enemy, ecell);
            if let Some(o) = l.world_mut().obj_mut(e) {
                o.health = i32::MAX / 4;
            }
            e
        };
        let before = run % 2 == 0;
        let mut e = if before {
            Some(place_enemy(&mut l))
        } else {
            None
        };
        l.ticks(2);
        let mut orders = 0;
        for &m in &men {
            orders += l.order(local, m, ClickOn::Object(b));
        }
        let t0 = l.world().tick;
        let all_in = until(&mut l, 900, |w| {
            glasnost_sim::structures::garrison::count(w, b) == 3
        });
        let t_in = l.world().tick;
        if e.is_none() {
            e = Some(place_enemy(&mut l));
        }
        let e = e.unwrap();
        let eh0 = l.world().obj(e).map_or(0, |o| o.health);
        let bh0 = l.world().obj(b).map_or(0, |o| o.health);
        let mut first_target = None;
        let mut first_hit = None;
        let mut building_hit = None;
        for _ in 0..1200 {
            l.ticks(1);
            let w = l.world();
            let dt = w.tick - t_in;
            let Some(o) = w.obj(b) else { break };
            if first_target.is_none() && o.target.is_some() {
                first_target = Some(dt);
            }
            if building_hit.is_none() && o.health < bh0 {
                building_hit = Some(dt);
            }
            if first_hit.is_none() && w.obj(e).is_some_and(|x| x.health < eh0) {
                first_hit = Some(dt);
                break;
            }
        }
        let w = l.world();
        let o = w.obj(b);
        println!(
            "r8-garrison-fire run {run}: enemy {} {}, entry orders {orders}, ticks to three inside {all_in:?} (from +{}), occupants {}, building owner is the local house {}, behaviour {:?}; after the entry: first target +{first_target:?}, first hit on the enemy +{first_hit:?}, building hit +{building_hit:?}",
            if armed_enemy { "armed" } else { "unarmed" },
            if before {
                "before the entry"
            } else {
                "after the entry"
            },
            t_in - t0,
            glasnost_sim::structures::garrison::count(w, b),
            o.and_then(|o| o.owner) == Some(local),
            o.and_then(|o| o.beh.current)
        );
    }
}
