//! Local lane for the M6b features (never in CI): with `GLASNOST_GAME_DIR`
//! set, every scene of [`SCENES`] starts its own skirmish through the
//! windowless client, puts the objects it needs on the map by their rules
//! flags (never by type name), plays the scene and writes its screens to
//! `local-lane/m6b/` (git-ignored). Prints counts, cells, ticks and
//! per-mille health only, never type names or rules values.
//!
//! Scenes now: a hover crossing water (S-0503), an own unit passing an own
//! gate (S-0215 claim 35, S-0108 claim 4), a teleporting unit's jump
//! (S-0207 claims 20 to 22, S-0208 claims 19 and 20), a damaged ship
//! repaired at its naval yard (S-0314 claims 16 to 24, S-0504 claim 14) and
//! a vehicle repaired at a repair depot (S-0314), and a bridge demolished
//! from its hut and repaired by an engineer (S-0511; the map is the first
//! offered map with a high bridge and its hut), and a cloaked vehicle seen
//! by its owner and by an enemy (S-0516 claims 16 and 28), and the local
//! house's superweapons launched through the client's aim (S-0512, S-0513:
//! the nuclear strike, the weather storm, the iron curtain, the teleport, the
//! force shield, the paradrop plane and the spy plane; the type is found by
//! its kind, never by name), and paratroopers dropping under their
//! parachutes (S-0522; sent through `planes::paradrop` only when no type has the
//! paradrop kind), and the trigger events and actions of every
//! offered map counted by how this engine covers them (S-0011, S-0524;
//! counts and codes only). A new scene is one more function and one more
//! row of [`SCENES`].
//!
//! Run with `cargo test --release -p glasnost-app --test m6b_lane --
//! --ignored --nocapture`. `GLASNOST_LANE_SCENE=a,b` plays only the scenes
//! with those keys; `GLASNOST_UI_MAP=n` picks the n-th offered map (default
//! 0; the hover and yard scenes need a map with open water).

#![cfg(not(target_arch = "wasm32"))]

use std::rc::Rc;

use glasnost_app::game::client::{Client, run_job};
use glasnost_app::game::play::{Button, Tool};
use glasnost_app::mapview::png;
use glasnost_map::CellXY;
use glasnost_map::terrain::land;
use glasnost_sim::behaviour::Door;
use glasnost_sim::combat::effects::Effect;
use glasnost_sim::orders::Order;
use glasnost_sim::rules::{Behaviour, Kind, Locomotor, UnitRules};
use glasnost_sim::superweapons::{SwEvent, SwKind, charge};
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
    ("hover", hover_crossing_water),
    ("gate", gate_opening),
    ("teleport", unit_teleporting),
    ("yard", ship_repaired_at_yard),
    ("depot", vehicle_repaired_at_depot),
    ("bridge-destroyed", bridge_destroyed),
    ("bridge-repaired", bridge_repaired),
    ("cloaked-as-owner", cloaked_as_owner),
    ("cloaked-as-enemy", cloaked_as_enemy),
    ("sw-nuke", sw_nuke),
    ("sw-storm", sw_storm),
    ("sw-iron-curtain", sw_iron_curtain),
    ("sw-chronosphere", sw_chronosphere),
    ("sw-force-shield", sw_force_shield),
    ("sw-paradrop-plane", sw_paradrop_plane),
    ("sw-spy-plane", sw_spy_plane),
    ("paratroopers-dropping", paratroopers_dropping),
    ("transport-unloading", transport_unloading),
    ("trigger-coverage", trigger_coverage),
];

#[test]
#[ignore = "local lane: needs GLASNOST_GAME_DIR"]
fn m6b_scenes() {
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

/// S-0503: a hover-locomotor vehicle driven from the shore out across open
/// water, shot while it is moving over water cells away from the shore.
fn hover_crossing_water(lane: &mut Lane) {
    let w = lane.world();
    let hover = find_type(w, Kind::Vehicle, |_, u| u.locomotor == Locomotor::Hover);
    println!("hover type found: {}", hover.is_some());
    let Some(name) = hover else { return };
    let home = lane.home();
    let Some(sea) = lane.area(home, (1, 1), 2, |l| l == land::WATER) else {
        println!("no open water");
        return;
    };
    // The nearest standing place on land to that water, and a point across it.
    let shore = lane.open_land(sea);
    let (dx, dy) = (sea.x - shore.x, sea.y - shore.y);
    let far = CellXY::new(sea.x + dx.signum() * 8, sea.y + dy.signum() * 8);
    let dest = lane
        .area(far, (1, 1), 1, |l| l == land::WATER)
        .unwrap_or(sea);
    let local = lane.local();
    let id = lane.spawn(Kind::Vehicle, &name, local, shore);
    let issued = lane.order(local, id, ClickOn::Cell(dest));
    println!("hover from {shore:?} to {dest:?}, {issued} orders");
    let mut shot = false;
    for _ in 0..1500 {
        lane.ticks(1);
        let w = lane.world();
        let Some(o) = w.obj(id) else { break };
        let c = o.pos.cell();
        // Open water two cells round, so that no shore or tree hides the craft.
        let open = (-2..=2).all(|dy| {
            (-2..=2).all(|dx| {
                w.cells
                    .cell(CellXY::new(c.x + dx, c.y + dy))
                    .is_some_and(|x| x.land == land::WATER)
            })
        });
        let from_shore = (c.x - shore.x).abs().max((c.y - shore.y).abs());
        if open && from_shore >= 3 && glasnost_sim::hover::moving_now(w, id) {
            let above = glasnost_sim::hover::hover_height(w, id) > 0;
            println!(
                "hover over water at {c:?}, {from_shore} cells out, above the surface: {above}"
            );
            lane.look_at(c);
            lane.ticks(1);
            lane.save("hover-crossing-water");
            shot = true;
            break;
        }
    }
    println!("hover seen moving over water: {shot}");
}

/// S-0215 claim 35, S-0108 claim 4: an own vehicle ordered straight through
/// an own gate, shot while the door is half open. The door frame of a
/// moving gate is S-0215 claim 35 d1's.
fn gate_opening(lane: &mut Lane) {
    let w = lane.world();
    let gate = w
        .rules()
        .buildings
        .iter()
        .find(|b| b.gate)
        .map(|b| (b.name.clone(), b.foundation));
    let unit = find_type(w, Kind::Vehicle, |_, u| plain_vehicle(u));
    println!(
        "gate type found: {}, vehicle type found: {}",
        gate.is_some(),
        unit.is_some()
    );
    let (Some((gn, (fw, fh))), Some(un)) = (gate, unit) else {
        return;
    };
    let home = lane.home();
    let Some(at) = lane.area(home, (fw, fh), 4, |l| l != land::WATER && l != land::ROCK) else {
        println!("no open land for the gate");
        return;
    };
    let local = lane.local();
    let g = lane.spawn(Kind::Building, &gn, local, at);
    // Across the gate's thin axis, through its middle cell.
    let (from, to) = if fw >= fh {
        let x = at.x + fw / 2;
        (CellXY::new(x, at.y - 4), CellXY::new(x, at.y + fh + 3))
    } else {
        let y = at.y + fh / 2;
        (CellXY::new(at.x - 4, y), CellXY::new(at.x + fw + 3, y))
    };
    let v = lane.spawn(Kind::Vehicle, &un, local, from);
    lane.ticks(2);
    let issued = lane.order(local, v, ClickOn::Cell(to));
    println!("gate at {at:?} foundation {fw}x{fh}, vehicle {from:?} to {to:?}, {issued} orders");
    let cells: Vec<CellXY> = (0..fw)
        .flat_map(|dx| (0..fh).map(move |dy| CellXY::new(at.x + dx, at.y + dy)))
        .collect();
    let (mut shot, mut inside, mut opened) = (false, false, None);
    for _ in 0..1500 {
        lane.ticks(1);
        let w = lane.world();
        let Some(o) = w.obj(v) else { break };
        inside |= cells.contains(&o.pos.cell());
        let t = w.tick;
        if let Some(Door::Opening { since, ticks }) = w.obj(g).map(|x| x.beh.gate.door) {
            opened.get_or_insert(since);
            if !shot && t - since >= ticks / 2 {
                println!(
                    "door opening since tick {since} over {ticks} ticks; vehicle at {:?}",
                    o.pos.cell()
                );
                lane.look_at(at);
                lane.ticks(1);
                lane.save("gate-opening");
                shot = true;
            }
        }
        if shot && inside {
            break;
        }
    }
    let end = lane.world().obj(v).map(|o| o.pos.cell());
    println!(
        "door began opening at tick {opened:?}; shot: {shot}; vehicle stood in the gate: {inside}; vehicle now at {end:?}"
    );
}

/// S-0207 claims 20 to 22, S-0208 claims 19 and 20: a teleporting unit
/// ordered a few cells away; shot just after its jump (the warp animations
/// at the old point and the landing point) and again after it has phased in.
fn unit_teleporting(lane: &mut Lane) {
    let w = lane.world();
    // An explicit teleporting locomotor: a type with none defaults to it (S-0207 claim 14).
    let real = |u: &UnitRules| u.raw.locomotor.is_some() && u.locomotor == Locomotor::Teleport;
    let unit = find_type(w, Kind::Vehicle, |_, u| real(u) && !u.harvester)
        .map(|n| (Kind::Vehicle, n))
        .or_else(|| find_type(w, Kind::Infantry, |_, u| real(u)).map(|n| (Kind::Infantry, n)));
    println!(
        "teleporting type found: {}",
        unit.as_ref()
            .map(|(k, _)| format!("{k:?}"))
            .unwrap_or("none".into())
    );
    let Some((kind, name)) = unit else { return };
    let home = lane.home();
    let Some(from) = lane.area(CellXY::new(home.x + 6, home.y + 6), (1, 1), 2, |l| {
        l != land::WATER && l != land::ROCK
    }) else {
        return;
    };
    let Some(to) = lane.area(CellXY::new(from.x + 6, from.y - 3), (1, 1), 1, |l| {
        l != land::WATER && l != land::ROCK
    }) else {
        return;
    };
    let local = lane.local();
    let id = lane.spawn(kind, &name, local, from);
    lane.ticks(2);
    println!(
        "teleporter: {}",
        glasnost_sim::teleport::is_teleporter(lane.world(), id)
    );
    let issued = lane.order(local, id, ClickOn::Cell(to));
    println!("teleport from {from:?} to {to:?}, {issued} orders");
    let mut last = lane.world().obj(id).map(|o| o.pos.cell());
    let mut jumped = None;
    for _ in 0..600 {
        lane.ticks(1);
        let w = lane.world();
        let now = w.obj(id).map(|o| o.pos.cell());
        if let (Some(a), Some(b)) = (last, now)
            && (a.x - b.x).abs().max((a.y - b.y).abs()) > 1
        {
            jumped = Some((w.tick, a, b));
            break;
        }
        last = now;
    }
    println!("jump (tick, from, to): {jumped:?}");
    let Some((_, a, b)) = jumped else { return };
    lane.look_at(CellXY::new((a.x + b.x) / 2, (a.y + b.y) / 2));
    lane.ticks(1);
    let phasing = glasnost_sim::teleport::phasing_in(lane.world(), id);
    println!("phasing in after the jump: {phasing}");
    lane.save("unit-teleporting");
    // The warp animations a few frames on.
    lane.ticks(5);
    lane.save("unit-teleporting-2");
    for _ in 0..600 {
        lane.ticks(1);
        if !glasnost_sim::teleport::phasing_in(lane.world(), id) {
            break;
        }
    }
    lane.look_at(b);
    lane.ticks(1);
    lane.save("unit-teleported");
}

/// S-0314 claims 16 to 24 and S-0504 claim 14: a damaged ship clicked onto
/// an own naval yard with the repair flag docks and is repaired.
fn ship_repaired_at_yard(lane: &mut Lane) {
    let w = lane.world();
    let yard = w
        .rules()
        .buildings
        .iter()
        .find(|b| b.naval && b.factory.is_some() && b.building.unit_repair)
        .map(|b| (b.name.clone(), b.foundation));
    let ship = find_type(w, Kind::Vehicle, |_, u| {
        u.locomotor == Locomotor::Ship && u.combat.naval && !u.combat.underwater
    });
    println!(
        "repairing yard type found: {}, ship type found: {}",
        yard.is_some(),
        ship.is_some()
    );
    let (Some((yn, (fw, fh))), Some(sn)) = (yard, ship) else {
        return;
    };
    let home = lane.home();
    let water = |l| l == land::WATER;
    let Some(at) = lane
        .area(home, (fw, fh), 1, water)
        .or_else(|| lane.area(home, (fw, fh), 0, water))
    else {
        println!("no open water for the yard");
        return;
    };
    let local = lane.local();
    let k = lane.spawn(Kind::Building, &yn, local, at);
    let Some(sea) = lane.area(CellXY::new(at.x - 5, at.y + 1), (1, 1), 1, water) else {
        return;
    };
    let s = lane.spawn(Kind::Vehicle, &sn, local, sea);
    println!("yard at {at:?} foundation {fw}x{fh}, ship at {sea:?}");
    repair_scene(lane, k, s, at, "ship-repaired-at-yard");
}

/// S-0314 claims 6 and 16 to 24: a damaged vehicle clicked onto an own
/// repair depot docks and is repaired.
fn vehicle_repaired_at_depot(lane: &mut Lane) {
    let w = lane.world();
    let depot = w
        .rules()
        .buildings
        .iter()
        .find(|b| b.building.unit_repair && !b.naval && b.factory.is_none())
        .map(|b| (b.name.clone(), b.foundation));
    let unit = find_type(w, Kind::Vehicle, |_, u| plain_vehicle(u));
    println!(
        "depot type found: {}, vehicle type found: {}",
        depot.is_some(),
        unit.is_some()
    );
    let (Some((dn, (fw, fh))), Some(un)) = (depot, unit) else {
        return;
    };
    let home = lane.home();
    let Some(at) = lane.area(home, (fw, fh), 3, |l| l != land::WATER && l != land::ROCK) else {
        println!("no open land for the depot");
        return;
    };
    let local = lane.local();
    let k = lane.spawn(Kind::Building, &dn, local, at);
    let Some(from) = lane.area(CellXY::new(at.x + fw + 5, at.y), (1, 1), 1, |l| {
        l != land::WATER && l != land::ROCK
    }) else {
        return;
    };
    let v = lane.spawn(Kind::Vehicle, &un, local, from);
    println!("depot at {at:?} foundation {fw}x{fh}, vehicle at {from:?}");
    repair_scene(lane, k, v, at, "vehicle-repaired-at-depot");
}

/// The shared repair scene: `unit` is damaged to a quarter, clicked onto
/// `depot`, and run until it is docked and repaired; shot once it has been
/// docked and repaired for a while. Prints health in per-mille of the type's
/// strength with the ticks.
fn repair_scene(lane: &mut Lane, depot: ObjectId, unit: ObjectId, at: CellXY, shot: &str) {
    let local = lane.local();
    lane.world_mut().houses[usize::from(local.0)].credits += 10_000;
    let full = lane.strength(unit);
    lane.world_mut().obj_mut(unit).unwrap().health = full / 4;
    lane.ticks(2);
    let orders = targeting::click(
        lane.world(),
        local,
        &[unit],
        ClickOn::Object(depot),
        Modifiers::default(),
    )
    .orders;
    let enter = orders.iter().any(|o| {
        matches!(
            o,
            Order::Behaviour {
                behaviour: Behaviour::Enter,
                ..
            }
        )
    });
    let mut issued = 0;
    for o in orders {
        issued += usize::from(glasnost_sim::orders::issue(lane.world_mut(), local, o));
    }
    let t0 = lane.world().tick;
    println!(
        "click onto the depot: enter order {enter}, {issued} issued at tick {t0}; health {} per mille",
        lane.permille(unit)
    );
    let mut docked = None;
    let mut first_rise = None;
    let mut saved = false;
    let mut start = 0;
    for _ in 0..6000 {
        lane.ticks(1);
        let w = lane.world();
        let Some(o) = w.obj(unit) else { break };
        let t = w.tick;
        let tethered = glasnost_sim::depot::tethered(w, unit);
        if docked.is_none() && (t - t0) % 250 == 0 {
            println!(
                "  tick {t}: cell {:?}, behaviour {:?}, depot link {:?}",
                o.pos.cell(),
                o.beh.current,
                glasnost_sim::depot::link(w, unit).map(|l| l.tethered)
            );
        }
        if tethered && docked.is_none() {
            docked = Some(t);
            start = lane.permille(unit);
            println!(
                "docked at tick {t} (cell {:?}); health {start} per mille",
                o.pos.cell()
            );
        }
        let pm = lane.permille(unit);
        if docked.is_some() && pm > start && first_rise.is_none() {
            first_rise = Some(t);
            println!("first repair step at tick {t}; health {pm} per mille");
        }
        if !saved && first_rise.is_some() && pm >= (start + 1000) / 2 {
            lane.look_at(at);
            lane.ticks(1);
            lane.save(shot);
            saved = true;
        }
        if pm >= 1000 || (docked.is_some() && !tethered) {
            break;
        }
    }
    let w = lane.world();
    println!(
        "after tick {}: health {} per mille, still docked: {}; docked at {docked:?}, first repair step at {first_rise:?}",
        w.tick,
        lane.permille(unit),
        glasnost_sim::depot::tethered(w, unit)
    );
    if !saved {
        lane.look_at(at);
        lane.ticks(1);
        lane.save(shot);
    }
}

/// A bridge repair hut on the map with a high-bridge cell within 3 cells
/// of its cell, if any.
fn bridge_hut(w: &World) -> Option<ObjectId> {
    let r = w.rules();
    w.objects.values().find_map(|o| {
        let hut = o.on_map
            && o.kind == Kind::Building
            && r.buildings
                .get(o.type_index as usize)
                .is_some_and(|b| b.building.bridge_repair_hut);
        let c = o.pos.cell();
        let near = (-3..=3)
            .any(|dx| (-3..=3).any(|dy| w.cells.is_bridge(CellXY::new(c.x + dx, c.y + dy))));
        (hut && near).then_some(o.id)
    })
}

/// The number of high-bridge sections and of enabled bridge links.
fn bridge_counts(w: &World) -> (usize, usize) {
    let sections = w
        .cells
        .grid
        .iter()
        .filter(|(_, c)| glasnost_sim::bridges::pieces::high_of(c.overlay).is_some())
        .count();
    let enabled = w.cells.links.iter().filter(|l| l.enabled).count();
    (sections, enabled)
}

/// Starts on the first map with a high bridge and its hut, and demolishes
/// that bridge from the hut (S-0511 claim 16). Returns the hut.
fn demolish_a_bridge(lane: &mut Lane) -> Option<ObjectId> {
    let Some(l) = Lane::start_where(|w| bridge_hut(w).is_some()) else {
        println!("no offered map has a high bridge with its hut");
        return None;
    };
    *lane = l;
    let hut = bridge_hut(lane.world())?;
    let at = lane.world().obj(hut)?.pos.cell();
    let (s0, e0) = bridge_counts(lane.world());
    let t0 = lane.world().tick;
    glasnost_sim::bridges::hut::demolish(lane.world_mut(), hut);
    let (s1, e1) = bridge_counts(lane.world());
    let offered = glasnost_sim::bridges::hut::repair_offered(lane.world(), hut);
    println!(
        "demolition at tick {t0}: sections {s0} -> {s1}, enabled links {e0} -> {e1}, repair offered {offered}"
    );
    lane.look_at(at);
    Some(hut)
}

/// S-0511 claims 5, 9 and 16: a high bridge demolished from its hut; shot
/// once the debris has fallen.
fn bridge_destroyed(lane: &mut Lane) {
    if demolish_a_bridge(lane).is_none() {
        return;
    }
    lane.ticks(30);
    lane.save("bridge-destroyed");
}

/// S-0511 claims 17 to 19: the demolished bridge repaired by an engineer
/// entering its hut (the entry itself is S-0509's, not built: the seam is
/// called at the hut's cell); shot after the repair.
fn bridge_repaired(lane: &mut Lane) {
    let Some(hut) = demolish_a_bridge(lane) else {
        return;
    };
    lane.ticks(30);
    let w = lane.world();
    let Some(name) = find_type(w, Kind::Infantry, |_, u| u.engineer) else {
        println!("no engineer type");
        return;
    };
    let at = w.obj(hut).map(|o| o.pos.cell()).unwrap();
    let local = lane.local();
    let e = lane.spawn(Kind::Infantry, &name, local, at);
    let t0 = lane.world().tick;
    let accepted = glasnost_sim::bridges::hut::engineer_entered_hut(lane.world_mut(), e, hut);
    let (s, en) = bridge_counts(lane.world());
    let offered = glasnost_sim::bridges::hut::repair_offered(lane.world(), hut);
    println!(
        "engineer entry at tick {t0} accepted {accepted}: sections {s}, enabled links {en}, repair offered {offered}"
    );
    lane.ticks(10);
    lane.save("bridge-repaired");
}

/// A driving vehicle type with the cloak flag, else (printed) a plain
/// vehicle that is then given its own cloak flag through the crate seam.
fn cloak_vehicle(lane: &Lane) -> Option<(String, bool)> {
    let w = lane.world();
    let own = find_type(w, Kind::Vehicle, |_, u| {
        u.locomotor == Locomotor::Drive && u.combat.cloakable && !u.combat.naval && !u.harvester
    });
    println!("vehicle type with the cloak flag found: {}", own.is_some());
    own.map(|n| (n, true))
        .or_else(|| find_type(w, Kind::Vehicle, |_, u| plain_vehicle(u)).map(|n| (n, false)))
}

/// Places a cloaking vehicle for `house` and a plain one beside it, near
/// the local start; returns the cloaking one and its cell.
fn place_cloaked(lane: &mut Lane, house: HouseId) -> Option<(ObjectId, CellXY)> {
    let (name, flag) = cloak_vehicle(lane)?;
    let home = lane.home();
    let at = lane.open_land(CellXY::new(home.x + 6, home.y + 6));
    let id = lane.spawn(Kind::Vehicle, &name, house, at);
    if !flag {
        glasnost_sim::cloak::machine::grant_own_flag(lane.world_mut(), id);
    }
    if let Some(plain) = find_type(lane.world(), Kind::Vehicle, |_, u| plain_vehicle(u)) {
        let beside = lane.open_land(CellXY::new(at.x + 2, at.y));
        lane.spawn(Kind::Vehicle, &plain, house, beside);
    }
    lane.look_at(at);
    Some((id, at))
}

/// Runs until `id` is fully cloaked (at most 600 ticks); prints the ticks.
fn wait_cloaked(lane: &mut Lane, id: ObjectId) -> bool {
    for n in 0..600 {
        if glasnost_sim::cloak::fully_cloaked(lane.world(), id) {
            println!("fully cloaked after {n} ticks");
            return true;
        }
        lane.ticks(1);
    }
    println!("not cloaked after 600 ticks");
    false
}

/// S-0516 claims 16, 28 and 29: the local house's own cloaked vehicle, drawn
/// 50 % translucent for its owner, without its shadow.
fn cloaked_as_owner(lane: &mut Lane) {
    let local = lane.local();
    let Some((id, at)) = place_cloaked(lane, local) else {
        println!("no vehicle type");
        return;
    };
    if !wait_cloaked(lane, id) {
        return;
    }
    let look = glasnost_sim::cloak::view::look(lane.world(), id, Some(local));
    println!("at {at:?}: the owner's look {look:?}");
    lane.ticks(1);
    lane.save("cloaked-as-owner");
}

/// S-0516 claim 28: an enemy's cloaking vehicle shot mid-fade (drawn
/// translucent for every viewer), then fully cloaked (not drawn for a house
/// that does not sense it).
fn cloaked_as_enemy(lane: &mut Lane) {
    let local = lane.local();
    let w = lane.world();
    let enemy = (0..w.houses.len()).map(|i| HouseId(i as u8)).find(|&h| {
        h != local
            && !w.allied(Some(local), Some(h))
            && w.houses[usize::from(h.0)].role == glasnost_sim::world::HouseRole::Player
    });
    let Some(enemy) = enemy else {
        println!("no enemy house");
        return;
    };
    let Some((id, at)) = place_cloaked(lane, enemy) else {
        println!("no vehicle type");
        return;
    };
    // Make it fade in again from the start, then shoot the fade.
    glasnost_sim::cloak::machine::uncloak(lane.world_mut(), id, false);
    let mut shot = false;
    for n in 0..1200 {
        lane.ticks(1);
        let w = lane.world();
        let look = glasnost_sim::cloak::view::look(w, id, Some(local));
        if glasnost_sim::cloak::state(w, id) == glasnost_sim::cloak::CloakState::Cloaking
            && look == glasnost_sim::cloak::view::Look::Translucent50
        {
            println!("at {at:?}, after {n} ticks: the enemy's fading look {look:?}");
            lane.save("cloaking-as-enemy");
            shot = true;
            break;
        }
    }
    println!("fade shot: {shot}");
    if !wait_cloaked(lane, id) {
        return;
    }
    let look = glasnost_sim::cloak::view::look(lane.world(), id, Some(local));
    let hidden = glasnost_sim::cloak::hidden_from(lane.world(), id, Some(local));
    println!("fully cloaked: the enemy's look {look:?}, hidden from the local house {hidden}");
    lane.save("cloaked-as-enemy");
}

// ---------------------------------------------------------------------------
// Superweapons (S-0512, S-0513)

/// Land a building or a vehicle can stand on.
fn standing(l: u8) -> bool {
    l != land::WATER && l != land::ROCK
}

/// Readies the local house's first superweapon type of `kind`: the
/// building that gives it is placed (with power plants until the house has
/// enough power), else the record is granted permanently; then the charge
/// is set full. Returns the type's list position.
fn ready_sw(lane: &mut Lane, kind: SwKind) -> Option<u32> {
    let w = lane.world();
    let types = &w.rules().superweapons.types;
    let unknown: Vec<usize> = (0..types.len())
        .filter(|&i| types[i].kind.is_none())
        .collect();
    let Some(sw) = w.rules().superweapons.first_of_kind(kind) else {
        println!(
            "no superweapon type of this kind ({} types; list positions without a recognised kind: {unknown:?})",
            types.len()
        );
        return None;
    };
    let credits = w.houses[usize::from(lane.local().0)].credits;
    let local = lane.local();
    let giver = w
        .rules()
        .superweapons
        .building_sw
        .iter()
        .position(|f| f.iter().flatten().any(|&i| i == sw));
    let giver = giver.map(|b| {
        let r = &w.rules().buildings[b];
        (r.name.clone(), r.foundation)
    });
    let home = lane.home();
    let mut placed = false;
    if let Some((name, size)) = giver
        && let Some(at) = lane.area(CellXY::new(home.x - 8, home.y + 4), size, 1, standing)
    {
        lane.spawn(Kind::Building, &name, local, at);
        placed = true;
    }
    lane.power_up();
    lane.ticks(3);
    let granted = charge::get(lane.world(), local, sw).is_some_and(|r| r.granted);
    if !granted {
        charge::permanent_grant(lane.world_mut(), local, sw);
    }
    let now = lane.world().houses[usize::from(local.0)].credits;
    println!(
        "superweapon at list position {sw}: giver building placed {placed}, granted by it {granted}; credits changed by {}",
        now - credits
    );
    charge::set_charge(lane.world_mut(), local, sw, 100);
    for n in 0..30 {
        if charge::offers_targeting(lane.world(), local, sw) {
            println!("ready {n} ticks after the full charge was set");
            return Some(sw);
        }
        lane.ticks(1);
    }
    println!("not ready 30 ticks after the full charge was set");
    None
}

/// S-0513 claims 7 to 13: a nuclear strike at three enemy vehicles, shot
/// when the first of them is hit.
fn sw_nuke(lane: &mut Lane) {
    let Some(sw) = ready_sw(lane, SwKind::Missile) else {
        return;
    };
    let Some(enemy) = lane.enemy() else {
        println!("no enemy house");
        return;
    };
    let home = lane.home();
    let target = lane.open_land(CellXY::new(home.x + 14, home.y + 14));
    let victims = lane.vehicles(enemy, target, 3);
    lane.look_at(target);
    if !lane.aim(sw, None, target) {
        return;
    }
    let t0 = lane.world().tick;
    let mut launch = None;
    let mut before: Vec<i32> = victims.iter().map(|&v| lane.permille(v)).collect();
    for _ in 0..2000 {
        lane.ticks(1);
        let t = lane.world().tick;
        if launch.is_none() && lane.sw_event(|e| e == SwEvent::Launch) {
            launch = Some(t - t0);
        }
        let now: Vec<i32> = victims.iter().map(|&v| lane.permille(v)).collect();
        // The strike: a vehicle losing at least 300 per mille in one tick.
        let struck = before.iter().zip(&now).any(|(b, n)| b - n >= 300);
        before = now;
        if struck {
            let dead = before.iter().filter(|&&n| n == 0).count();
            println!(
                "launch event {launch:?} ticks after the order; strike {} ticks after it, per mille now {before:?}, {dead} of {} destroyed",
                t - t0,
                victims.len()
            );
            lane.ticks(2);
            lane.save("sw-nuke");
            return;
        }
    }
    println!("launch event {launch:?}; no strike within 2000 ticks");
    lane.save("sw-nuke");
}

/// S-0513 claims 14 to 21: a weather storm over three enemy vehicles, shot
/// just after its first lightning strike.
fn sw_storm(lane: &mut Lane) {
    let Some(sw) = ready_sw(lane, SwKind::Storm) else {
        return;
    };
    let Some(enemy) = lane.enemy() else {
        println!("no enemy house");
        return;
    };
    let home = lane.home();
    let target = lane.open_land(CellXY::new(home.x + 14, home.y + 14));
    lane.vehicles(enemy, target, 3);
    lane.look_at(target);
    if !lane.aim(sw, None, target) {
        return;
    }
    let t0 = lane.world().tick;
    let mut start = None;
    for _ in 0..2000 {
        lane.ticks(1);
        let t = lane.world().tick - t0;
        if start.is_none() && lane.sw_event(|e| e == SwEvent::StormStart) {
            start = Some(t);
        }
        if lane.sw_event(|e| matches!(e, SwEvent::Lightning(_))) {
            println!(
                "storm start {start:?} ticks after the order; first lightning {t} ticks after it"
            );
            lane.ticks(1);
            lane.save("sw-storm");
            return;
        }
    }
    println!("storm start {start:?}; no lightning within 2000 ticks");
    lane.save("sw-storm");
}

/// S-0513 claims 22 to 29: the iron curtain on three own vehicles, shot a
/// few ticks after their protection starts.
fn sw_iron_curtain(lane: &mut Lane) {
    let Some(sw) = ready_sw(lane, SwKind::IronCurtain) else {
        return;
    };
    let local = lane.local();
    let home = lane.home();
    let target = lane.open_land(CellXY::new(home.x + 6, home.y + 6));
    let own = lane.vehicles(local, target, 3);
    lane.look_at(target);
    if !lane.aim(sw, None, target) {
        return;
    }
    let t0 = lane.world().tick;
    for _ in 0..300 {
        lane.ticks(1);
        let n = lane.protected(&own);
        if n > 0 {
            println!(
                "protection {} ticks after the order: {n} of {} vehicles",
                lane.world().tick - t0,
                own.len()
            );
            lane.ticks(5);
            lane.save("sw-iron-curtain");
            return;
        }
    }
    println!("no protection within 300 ticks");
    lane.save("sw-iron-curtain");
}

/// S-0513 claims 33 to 41: the teleport, its source picked through the aim
/// and its destination through the aim the pick armed (claim 33), shot as
/// the group lands.
fn sw_chronosphere(lane: &mut Lane) {
    let Some(sw) = ready_sw(lane, SwKind::Chrono) else {
        return;
    };
    let local = lane.local();
    let home = lane.home();
    let source = lane.open_land(CellXY::new(home.x + 6, home.y + 6));
    let group = lane.vehicles(local, source, 3);
    let dest = lane.open_land(CellXY::new(source.x + 10, source.y - 10));
    println!(
        "source {source:?}, destination {} cells away",
        (dest.x - source.x).abs().max((dest.y - source.y).abs())
    );
    if !lane.aim(sw, None, source) {
        return;
    }
    lane.ticks(1);
    let tool = lane.c.session.as_ref().unwrap().play.tool;
    let warp = lane
        .world()
        .rules()
        .superweapons
        .first_of_kind(SwKind::ChronoWarp);
    let Tool::Superweapon {
        sw: armed,
        from_source: true,
    } = tool
    else {
        println!("the source pick armed no warp type");
        return;
    };
    println!("the pick armed list position {armed}; the first warp-kind type is at {warp:?}");
    let t0 = lane.world().tick;
    let s = lane.c.session.as_mut().unwrap();
    s.play
        .click_map(Button::Left, None, dest, Modifiers::default());
    lane.look_at(dest);
    let mut joined = None;
    for _ in 0..600 {
        lane.ticks(1);
        let t = lane.world().tick - t0;
        let w = lane.world();
        let members = group
            .iter()
            .filter(|&&v| w.obj(v).is_some_and(|o| o.sw.warp.is_some()))
            .count();
        if joined.is_none() && members > 0 {
            joined = Some(t);
        }
        let near = group
            .iter()
            .filter(|&&v| {
                w.obj(v).is_some_and(|o| {
                    let c = o.pos.cell();
                    o.on_map
                        && o.sw.warp.is_none()
                        && (c.x - dest.x).abs().max((c.y - dest.y).abs()) <= 3
                })
            })
            .count();
        if near > 0 {
            println!(
                "members from {joined:?} ticks after the warp order; {near} of {} at the destination {t} ticks after it",
                group.len()
            );
            lane.ticks(2);
            lane.save("sw-chronosphere");
            return;
        }
    }
    println!("members from {joined:?}; none at the destination within 600 ticks");
    lane.save("sw-chronosphere");
}

/// S-0513 claims 30 to 32: the force shield aimed at an own building, shot a
/// few ticks after the shield starts.
fn sw_force_shield(lane: &mut Lane) {
    let Some(sw) = ready_sw(lane, SwKind::ForceShield) else {
        return;
    };
    let local = lane.local();
    let w = lane.world();
    let Some((b, at)) = w
        .objects
        .values()
        .find(|o| o.on_map && o.owner == Some(local) && o.kind == Kind::Building)
        .map(|o| (o.id, o.pos.cell()))
    else {
        println!("no own building");
        return;
    };
    lane.look_at(at);
    if !lane.aim(sw, Some(b), at) {
        return;
    }
    let t0 = lane.world().tick;
    for _ in 0..300 {
        lane.ticks(1);
        if lane.sw_event(|e| e == SwEvent::ShieldStart) {
            let w = lane.world();
            let shielded = w
                .objects
                .values()
                .filter(|o| o.on_map && o.sw.protection.is_some())
                .count();
            println!(
                "shield start {} ticks after the order; {shielded} objects protected",
                lane.world().tick - t0
            );
            lane.ticks(5);
            lane.save("sw-force-shield");
            return;
        }
    }
    println!("no shield start within 300 ticks");
    lane.save("sw-force-shield");
}

/// S-0513 claims 42 to 45: a plane sent to a cell, followed and shot when it
/// is within 6 cells of the target.
/// The plane comes through the aim when the type was found by its kind, else
/// straight through `launch` (the plane functions of `superweapons::planes`;
/// a type of that kind is not found, S-0512 claim 24).
fn plane_scene(
    lane: &mut Lane,
    sw: Option<u32>,
    launch: fn(&mut World, HouseId, CellXY) -> Vec<ObjectId>,
    shot: &str,
) {
    let local = lane.local();
    let home = lane.home();
    let target = lane.open_land(CellXY::new(home.x + 14, home.y + 14));
    let before = lane.aircraft(local);
    let through_aim = match sw {
        Some(sw) => lane.aim(sw, None, target),
        None => false,
    };
    if !through_aim {
        let n = launch(lane.world_mut(), local, target).len();
        println!("no type took the order: {n} plane(s) sent through planes directly");
    }
    let t0 = lane.world().tick;
    let mut plane = None;
    for _ in 0..2000 {
        lane.ticks(1);
        let t = lane.world().tick - t0;
        if plane.is_none() {
            plane = lane
                .aircraft(local)
                .into_iter()
                .find(|id| !before.contains(id));
            if plane.is_some() {
                println!("plane on the map {t} ticks after the order");
            }
        }
        let Some(id) = plane else { continue };
        let Some(c) = lane.world().obj(id).map(|o| o.pos.cell()) else {
            println!("the plane left the map {t} ticks after the order");
            break;
        };
        lane.look_at(c);
        if (c.x - target.x).abs().max((c.y - target.y).abs()) <= 6 {
            println!(
                "plane within 6 cells of the target {t} ticks after the order, {} leptons above the ground",
                glasnost_sim::air::height(lane.world(), id)
            );
            lane.ticks(1);
            lane.save(shot);
            return;
        }
    }
    println!("plane found {}; not near the target", plane.is_some());
    lane.save(shot);
}

fn sw_paradrop_plane(lane: &mut Lane) {
    let w = lane.world();
    let kind = if w
        .rules()
        .superweapons
        .first_of_kind(SwKind::Paradrop)
        .is_some()
    {
        SwKind::Paradrop
    } else {
        SwKind::AmerParadrop
    };
    let sw = ready_sw(lane, kind);
    plane_scene(
        lane,
        sw,
        |w, h, c| glasnost_sim::superweapons::planes::paradrop(w, h, c, false),
        "sw-paradrop-plane",
    );
}

fn sw_spy_plane(lane: &mut Lane) {
    let sw = ready_sw(lane, SwKind::SpyPlane);
    plane_scene(
        lane,
        sw,
        glasnost_sim::superweapons::planes::spy_plane,
        "sw-spy-plane",
    );
}

/// S-0011 and S-0524: on every offered map, the trigger events and actions
/// counted as implemented, seamed or outside the subset
/// (`mp_triggers::event_coverage`, `mp_triggers::action_coverage`). Prints
/// counts and code numbers only.
fn trigger_coverage(_lane: &mut Lane) {
    use glasnost_sim::mp_triggers::{Coverage, action_coverage, event_coverage};
    use std::collections::BTreeMap;
    let n = Lane::map_count();
    let (mut with, mut events, mut actions) = (0, [0usize; 3], [0usize; 3]);
    let mut other: BTreeMap<(&str, i32), usize> = BTreeMap::new();
    let slot = |c: Coverage| match c {
        Coverage::Implemented => 0,
        Coverage::Seam => 1,
        Coverage::Unknown => 2,
    };
    for i in 0..n {
        let Some(lane) = Lane::start_on(i) else {
            return;
        };
        let t = &lane.world().triggers.map;
        println!("map {i}: {} triggers", t.triggers.len());
        with += usize::from(!t.triggers.is_empty());
        for d in &t.triggers {
            for e in &d.events {
                let k = slot(event_coverage(e.code));
                events[k] += 1;
                if k != 0 {
                    *other.entry(("event", e.code)).or_default() += 1;
                }
            }
            for a in &d.actions {
                let k = slot(action_coverage(a.code));
                actions[k] += 1;
                if k != 0 {
                    *other
                        .entry((if k == 1 { "seam action" } else { "action" }, a.code))
                        .or_default() += 1;
                }
            }
        }
    }
    println!("maps: {n}, with triggers: {with}");
    println!(
        "events: {} implemented, {} seamed, {} outside",
        events[0], events[1], events[2]
    );
    println!(
        "actions: {} implemented, {} seamed, {} outside",
        actions[0], actions[1], actions[2]
    );
    for ((what, code), k) in other {
        println!("  {what} {code}: {k}");
    }
}

/// S-0522 claims 6 to 13: a paradrop plane sent to a cell near home; shot
/// while its paratroopers are descending under their parachutes, then
/// followed until they have all landed. The plane comes through the aim when
/// a superweapon type has a paradrop kind, else straight through
/// `planes::paradrop` (S-0512 claim 24: the kind words are fixed, so a type is normally found).
fn paratroopers_dropping(lane: &mut Lane) {
    let local = lane.local();
    let home = lane.home();
    let target = lane.open_land(CellXY::new(home.x + 10, home.y + 10));
    let w = lane.world();
    let kind = [SwKind::Paradrop, SwKind::AmerParadrop]
        .into_iter()
        .find(|&k| w.rules().superweapons.first_of_kind(k).is_some());
    let through_aim = match kind.and_then(|k| ready_sw(lane, k)) {
        Some(sw) => lane.aim(sw, None, target),
        None => false,
    };
    if !through_aim {
        let american = kind == Some(SwKind::AmerParadrop);
        let planes =
            glasnost_sim::superweapons::planes::paradrop(lane.world_mut(), local, target, american);
        println!(
            "no paradrop-kind type took the order: {} plane(s) sent through planes::paradrop",
            planes.len()
        );
    }
    lane.look_at(target);
    let t0 = lane.world().tick;
    let mut first = None;
    let mut dropped = 0usize;
    let mut shot = false;
    let mut later = None;
    let mut troopers: Vec<ObjectId> = Vec::new();
    for _ in 0..3000 {
        lane.ticks(1);
        let t = lane.world().tick - t0;
        let new: Vec<ObjectId> = lane
            .c
            .session
            .as_ref()
            .unwrap()
            .play
            .effects
            .iter()
            .filter_map(|e| match e {
                Effect::Superweapon {
                    event: SwEvent::Parachute { trooper, .. },
                    ..
                } => Some(*trooper),
                _ => None,
            })
            .collect();
        if !new.is_empty() && first.is_none() {
            first = Some(t);
            println!("first drop {t} ticks after the order");
        }
        dropped += new.len();
        troopers.extend(new);
        let w = lane.world();
        let descending = troopers
            .iter()
            .filter(|&&id| w.obj(id).is_some_and(|o| o.on_map && o.fall.chute))
            .count();
        if let Some(p) = troopers.first().and_then(|&id| w.obj(id)).map(|o| o.pos) {
            // Centred on the first paratrooper at its height.
            let s = lane.c.session.as_mut().unwrap();
            s.set_centre(glasnost_render::projection::project_lifted(p.x, p.y, p.z));
        }
        if !shot && descending >= 3 {
            let heights: Vec<i32> = lane
                .aircraft(local)
                .into_iter()
                .map(|id| glasnost_sim::air::height(lane.world(), id))
                .collect();
            println!(
                "{descending} descending {t} ticks after the order; aircraft heights {heights:?}"
            );
            lane.save("paratroopers-dropping");
            shot = true;
            later = Some(t + 60);
        }
        if later == Some(t) {
            println!("{descending} descending {t} ticks after the order");
            lane.save("paratroopers-descending");
        }
        if first.is_some_and(|f| t > f + 30) && descending == 0 {
            let w = lane.world();
            let standing = troopers
                .iter()
                .filter(|&&id| w.obj(id).is_some_and(|o| o.on_map))
                .count();
            println!(
                "{dropped} dropped; none descending {t} ticks after the order; {standing} on the ground"
            );
            if !shot {
                lane.save("paratroopers-dropping");
            }
            return;
        }
    }
    println!("first drop {first:?}; {dropped} dropped within 3000 ticks");
    if !shot {
        lane.save("paratroopers-dropping");
    }
}

/// S-0507 claims 3 to 12: infantry clicked onto an own ground transport
/// board it (shot with the load aboard), then a click on the transport lets
/// them out one by one (shot while some stand beside it and some are still
/// inside). The types are found by their flags: a vehicle with passenger
/// room that walks or drives on land, and plain infantry.
fn transport_unloading(lane: &mut Lane) {
    let w = lane.world();
    let truck = find_type(w, Kind::Vehicle, |_, u| {
        u.passengers >= 3 && u.locomotor == Locomotor::Drive && !u.combat.naval
    });
    let foot = find_type(w, Kind::Infantry, |_, u| u.locomotor == Locomotor::Walk);
    println!(
        "transport type found: {}, infantry type found: {}",
        truck.is_some(),
        foot.is_some()
    );
    let (Some(tn), Some(fnm)) = (truck, foot) else {
        return;
    };
    let home = lane.home();
    let Some(at) = lane.area(CellXY::new(home.x + 6, home.y + 6), (3, 3), 2, |l| {
        l != land::WATER && l != land::ROCK
    }) else {
        println!("no open land");
        return;
    };
    let local = lane.local();
    let t = lane.spawn(Kind::Vehicle, &tn, local, CellXY::new(at.x + 1, at.y + 1));
    let riders: Vec<ObjectId> = [(0, 0), (2, 0), (0, 2)]
        .iter()
        .map(|&(dx, dy)| {
            lane.spawn(
                Kind::Infantry,
                &fnm,
                local,
                CellXY::new(at.x + dx, at.y + dy),
            )
        })
        .collect();
    lane.ticks(2);
    lane.look_at(at);
    let orders: usize = riders
        .iter()
        .map(|&p| lane.order(local, p, ClickOn::Object(t)))
        .sum();
    println!("{} riders ordered aboard, {orders} orders", riders.len());
    let mut loaded_at = None;
    for i in 0..900 {
        lane.ticks(1);
        let n = glasnost_sim::transport::count(lane.world(), t);
        if n as usize == riders.len() {
            loaded_at = Some(i);
            break;
        }
    }
    println!(
        "loaded after {loaded_at:?} ticks: {} aboard",
        glasnost_sim::transport::count(lane.world(), t)
    );
    lane.ticks(2);
    lane.save("transport-loaded");
    let issued = lane.order(local, t, ClickOn::Object(t));
    println!("unload clicked: {issued} orders");
    let mut shot = false;
    for i in 0..900 {
        lane.ticks(1);
        let w = lane.world();
        let aboard = glasnost_sim::transport::count(w, t);
        let out = riders
            .iter()
            .filter(|&&p| {
                w.obj(p).is_some_and(|o| o.on_map)
                    && glasnost_sim::transport::riding(w, p).is_none()
            })
            .count();
        if !shot && out >= 1 && aboard >= 1 {
            println!("{out} out, {aboard} still inside, {i} ticks after the click");
            lane.save("transport-unloading");
            shot = true;
        }
        if out == riders.len() {
            println!("all {out} out {i} ticks after the click");
            if !shot {
                lane.save("transport-unloading");
            }
            lane.ticks(20);
            lane.save("transport-unloaded");
            return;
        }
    }
    println!("unloading incomplete; shot {shot}");
    if !shot {
        lane.save("transport-unloading");
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
        let out = png::lane_out("m6b");
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

    /// Places `n` plain vehicles of `house` on open land near `near`.
    fn vehicles(&mut self, house: HouseId, near: CellXY, n: i32) -> Vec<ObjectId> {
        let Some(name) = find_type(self.world(), Kind::Vehicle, |_, u| plain_vehicle(u)) else {
            println!("no plain vehicle type");
            return Vec::new();
        };
        // `near` from `open_land` has two free cells on each side.
        let at = self.open_land(near);
        (0..n.min(3))
            .map(|i| self.spawn(Kind::Vehicle, &name, house, CellXY::new(at.x - 1 + i, at.y)))
            .collect()
    }

    /// Places the local house's strongest plain power plant until it has
    /// enough power (at most 6); prints how many.
    fn power_up(&mut self) {
        let local = self.local();
        let w = self.world();
        let t = &w.rules().econ.tech.buildings;
        let plant = (0..t.len())
            .filter(|&b| {
                t[b].power > 0
                    && w.rules()
                        .superweapons
                        .building_sw
                        .get(b)
                        .is_none_or(|f| f.iter().all(Option::is_none))
            })
            .max_by_key(|&b| (t[b].power, std::cmp::Reverse(b)))
            .map(|b| {
                (
                    w.rules().buildings[b].name.clone(),
                    w.rules().buildings[b].foundation,
                )
            });
        let Some((name, size)) = plant else {
            println!("no power plant type");
            return;
        };
        for i in 0..6 {
            self.ticks(1);
            if !glasnost_sim::econ::power::low_power(self.world(), local) {
                println!("{i} power plants placed");
                return;
            }
            let home = self.home();
            if let Some(at) = self.area(
                CellXY::new(home.x + 8, home.y - 8 + 4 * i),
                size,
                1,
                standing,
            ) {
                self.spawn(Kind::Building, &name, local, at);
            }
        }
        println!("still low on power after 6 power plants");
    }

    /// Aims superweapon `sw` through the client (S-0513 claim 1): the tool
    /// armed as the icon arms it, then a left click on `cell` with no units
    /// selected. Returns whether the order was taken; prints it.
    fn aim(&mut self, sw: u32, on: Option<ObjectId>, cell: CellXY) -> bool {
        let p = &mut self.c.session.as_mut().unwrap().play;
        p.selection.clear();
        p.tool = Tool::Superweapon {
            sw,
            from_source: false,
        };
        let before = p.game.replay().orders.len();
        p.click_map(Button::Left, on, cell, Modifiers::default());
        let taken = p.game.replay().orders.len() > before;
        println!("aimed at {cell:?}: order taken {taken}");
        taken
    }

    /// Whether the last tick had a superweapon event matching `f`.
    fn sw_event(&self, f: impl Fn(SwEvent) -> bool) -> bool {
        self.c
            .session
            .as_ref()
            .unwrap()
            .play
            .effects
            .iter()
            .any(|e| matches!(e, Effect::Superweapon { event, .. } if f(*event)))
    }

    /// How many of `ids` are protected.
    fn protected(&self, ids: &[ObjectId]) -> usize {
        let w = self.world();
        ids.iter()
            .filter(|&&id| w.obj(id).is_some_and(|o| o.sw.protection.is_some()))
            .count()
    }

    /// The aircraft of `house` on the map.
    fn aircraft(&self, house: HouseId) -> Vec<ObjectId> {
        self.world()
            .objects
            .values()
            .filter(|o| o.on_map && o.owner == Some(house) && o.kind == Kind::Aircraft)
            .map(|o| o.id)
            .collect()
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

/// An armed driving vehicle that is not a harvester.
fn plain_vehicle(u: &UnitRules) -> bool {
    u.locomotor == Locomotor::Drive
        && !u.harvester
        && u.combat.slots[0].is_some()
        && !u.combat.naval
}

/// The name of the first type of `kind` matching `f`.
fn find_type(w: &World, kind: Kind, f: impl Fn(u32, &UnitRules) -> bool) -> Option<String> {
    let r = w.rules();
    let list = match kind {
        Kind::Infantry => &r.infantry,
        Kind::Vehicle => &r.vehicles,
        _ => &r.aircraft,
    };
    list.iter()
        .enumerate()
        .find(|(i, u)| f(*i as u32, u))
        .map(|(_, u)| u.name.clone())
}
