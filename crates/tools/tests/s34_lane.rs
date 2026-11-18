//! Local-only lane for sim issues batch S34 (opt-in with `--ignored` and
//! `GLASNOST_GAME_DIR`, never in CI). It reads the install's rules and art
//! and checks, for the issues the batch traces, what the install's data
//! give the simulation and the drawing:
//! - #444: every refinery-flag dock-unload building type's docking,
//!   payment and unloading slot names (S-0402 Terms, S-0108 Import
//!   mapping), whether the simulation and the drawing find each as an
//!   animation type with a sprite, and a whole harvest cycle of each plain
//!   harvester type at its first dock type, run headless on a synthetic
//!   flat map, with the refinery's running modelled slots per tick;
//! - #446: every balloon-hover type's kind, locomotor and crash flag;
//! - #448: every designator type's S-0308 row 6 weapon: the simulation's
//!   laser flag and laser duration, and the drawing's laser, electric-bolt
//!   and radiation-beam flags for it (`beams::Settings`).
//!
//! It prints type positions, slot kinds, counts and booleans only: no type
//! or file names, no values from the data. Nothing is written to a tracked
//! file and nothing is asserted about the data.
//!
//! Run with `cargo test -p glasnost-tools --test s34_lane -- --ignored
//! --nocapture`.

use std::path::Path;
use std::sync::Arc;

use glasnost_app::mapview::{beams, build, slot_play};
use glasnost_convert::kinds::Kind as ImportKind;
use glasnost_map::CellXY;
use glasnost_sim::depot::DepotSlot;
use glasnost_sim::econ::harvest;
use glasnost_sim::rules::{Behaviour, Kind, SimRules};
use glasnost_sim::testkit;
use glasnost_sim::world::{GameSetup, HouseRole, HouseSetup, World};

const SIZE: i32 = 48;

/// The import (with the temperate theater's art) and the resolved rules.
fn install(dir: &Path) -> (glasnost_convert::Import, Arc<SimRules>, glasnost_vfs::Vfs) {
    pollster::block_on(async {
        let mut vfs = glasnost_tools::local_lane::mount(dir)
            .await
            .expect("the install mounts");
        let options = glasnost_convert::ImportOptions {
            theater: Some(glasnost_vfs::theater::Theater::Temperate),
            ..Default::default()
        };
        let import = glasnost_convert::import(&mut vfs, &options).await;
        let r = glasnost_rules::resolve_ruleset(
            Some((import.tree.clone(), import.provenance.clone())),
            &[],
        )
        .expect("the install's rules resolve");
        let rules = Arc::new(SimRules::new(&r.ruleset, &r.sim_rules_hash));
        (import, rules, vfs)
    })
}

fn art_document(import: &glasnost_convert::Import) -> glasnost_ini::Document {
    match &import.art_file {
        Some(f) => glasnost_ini::Document::parse(&f.bytes, Default::default(), &mut Vec::new()),
        None => glasnost_ini::Document::empty(Default::default()),
    }
}

/// Whether the drawing finds `name` as an animation type whose art names
/// a sprite that the install holds.
fn drawn_anim(
    import: &glasnost_convert::Import,
    vfs: &glasnost_vfs::Vfs,
    name: &str,
) -> (bool, bool, bool) {
    let Some(a) = build::find_type(import, ImportKind::Animations, name.as_bytes()) else {
        return (false, false, false);
    };
    let sprite = import
        .arts
        .get(&(ImportKind::Animations, a))
        .and_then(|t| t.sprite.clone());
    let held = sprite
        .as_ref()
        .is_some_and(|s| vfs.find(&String::from_utf8_lossy(s)).is_some());
    (true, sprite.is_some(), held)
}

const BASES: [(&str, DepotSlot); 3] = [
    ("PreProductionAnim", DepotSlot::PreProduction),
    ("SpecialAnim", DepotSlot::Special1),
    ("ProductionAnim", DepotSlot::Production),
];

fn issue_444_names(import: &glasnost_convert::Import, rules: &SimRules, vfs: &glasnost_vfs::Vfs) {
    let art = art_document(import);
    for (i, b) in rules.buildings.iter().enumerate() {
        let e = &b.raw.econ;
        if e.refinery != Some(true) || e.dock_unload != Some(true) {
            continue;
        }
        let image = import
            .arts
            .get(&(
                ImportKind::Buildings,
                build::find_type(import, ImportKind::Buildings, b.name.as_bytes())
                    .unwrap_or(usize::MAX),
            ))
            .map(|a| a.image.clone());
        let defs = image.as_ref().map(|im| slot_play::read_defs(&art, im));
        println!(
            "#444 refinery type {i}: slot names given {}",
            e.slot_names.is_some()
        );
        // Every S-0108 slot whose normal name the type's art gives.
        let named: Vec<slot_play::Slot> = defs
            .iter()
            .flatten()
            .filter(|(_, def)| !def.normal.is_empty())
            .map(|(s, _)| *s)
            .collect();
        println!("  the drawing's named slots: {named:?}");
        for (base, slot) in BASES {
            let pair = e.slot_names.as_ref().and_then(|m| m.get(base));
            for (v, variant) in ["normal", "damaged"].into_iter().enumerate() {
                let name = pair.map(|p| p[v].clone()).unwrap_or_default();
                let sim = !name.is_empty() && rules.anim_index(name.as_bytes()).is_some();
                let (found, sprite, held) = drawn_anim(import, vfs, &name);
                println!(
                    "  {slot:?} {variant}: named {} sim finds {sim} drawing finds {found} sprite {sprite} held {held}",
                    !name.is_empty()
                );
            }
            let client = defs.as_ref().and_then(|d| {
                let s = slot_play::Slot::ALL
                    .iter()
                    .find(|s| s.base() == base.as_bytes())?;
                d.get(s).map(|def| !def.normal.is_empty())
            });
            println!("  {slot:?}: the drawing's slot definition has a name {client:?}");
        }
    }
}

fn default_country(rules: &SimRules) -> String {
    rules
        .country_names
        .iter()
        .zip(&rules.countries)
        .find(|(n, c)| {
            !c.passive
                && n.as_str() != glasnost_sim::load::NEUTRAL
                && n.as_str() != glasnost_sim::load::SPECIAL
        })
        .map(|(n, _)| n.clone())
        .expect("a player country")
}

fn game(rules: &Arc<SimRules>) -> World {
    let country = default_country(rules);
    let house = |name: &str| HouseSetup {
        name: name.into(),
        country: country.clone(),
        human: true,
        start: None,
        role: HouseRole::Player,
        ai: false,
    };
    let setup = GameSetup {
        seed: 7,
        game_speed: rules.general.lobby_game_speed,
        houses: vec![house("A"), house("B")],
        alliances: Vec::new(),
        sandbox: true,
        unit_count: 0,
        bases: false,
        skirmish: Default::default(),
    };
    let map = testkit::empty_map(SIZE);
    glasnost_sim::load::new_game(rules.clone(), &map, testkit::catalogue(), &setup)
        .expect("the lane's game starts")
}

fn issue_444_cycle(rules: &Arc<SimRules>) {
    let Some(run) = rules.econ.resource.tiberiums.iter().find_map(|t| t.run) else {
        println!("#444 cycle: no tiberium run");
        return;
    };
    for (ti, u) in rules.econ.harvest.vehicles.iter().enumerate() {
        if !u.harvester || u.slave_miner || u.teleporter || u.docks.is_empty() {
            continue;
        }
        let dock = u.docks[0];
        let mut w = game(rules);
        let mid = |dx, dy| testkit::mid(SIZE, dx, dy);
        for dy in 0..3 {
            for dx in 0..3 {
                let c: CellXY = mid(6 + dx, -1 + dy);
                let cell = w.cells.grid.get_mut(c).unwrap();
                cell.overlay = u8::try_from(run[0]).unwrap_or(0);
                cell.overlay_data = 3;
                glasnost_sim::load::recalc_cells(&mut w, &[c]);
                glasnost_sim::regions::cell_changed(&mut w, c);
            }
        }
        let bname = rules.buildings[dock as usize].name.clone();
        let vname = rules.vehicles[ti].name.clone();
        let b = testkit::spawn(&mut w, Kind::Building, &bname, 0, mid(-6, -1), 0);
        let Some(v) = testkit::spawn_in_play(
            &mut w,
            Kind::Vehicle,
            &vname,
            0,
            mid(0, 2),
            0,
            Some(Behaviour::Harvest),
        ) else {
            println!("#444 cycle, harvester type {ti}: not placed");
            continue;
        };
        let credits0 = w.houses[0].credits;
        let mut seen_contact = false;
        let mut last = String::new();
        for _ in 0..6000 {
            testkit::run(&mut w, 1);
            let Some(o) = w.obj(v) else { break };
            let contact = o.contact == Some(b);
            let s = harvest::state_of(&w, v);
            let facing = o.body.value(w.tick);
            let slots: Vec<DepotSlot> = glasnost_sim::depot_anim::draws(&w, b)
                .into_iter()
                .map(|d| d.slot)
                .collect();
            // spec: S-0402 claim 11 (facing map east: 16256 ≤ f ≤ 16511)
            let line = format!(
                "contact {contact} unloading {} finishing {} east {} load bales {} paid {} slots {slots:?}",
                s.unloading,
                s.finishing,
                (16256..=16511).contains(&facing),
                s.load.iter().sum::<i32>(),
                w.houses[0].credits != credits0
            );
            if contact {
                seen_contact = true;
            }
            if contact && line != last {
                println!(
                    "#444 cycle, harvester type {ti} at dock type {dock}, tick {}: {line}",
                    w.tick
                );
                last = line;
            }
            if seen_contact && !contact {
                println!(
                    "#444 cycle, harvester type {ti}: contact broken at tick {}",
                    w.tick
                );
                break;
            }
        }
        if !seen_contact {
            println!("#444 cycle, harvester type {ti}: no contact in 6000 ticks");
        }
    }
}

fn issue_446(rules: &SimRules) {
    for (kind, list) in [
        (Kind::Infantry, &rules.infantry),
        (Kind::Vehicle, &rules.vehicles),
        (Kind::Aircraft, &rules.aircraft),
    ] {
        for (i, u) in list.iter().enumerate() {
            if u.raw.balloon_hover != Some(true) {
                continue;
            }
            println!(
                "#446 balloon-hover {kind:?} {i}: jumpjet locomotor {} crashable {}",
                u.raw.locomotor == Some(glasnost_rules::sim::Locomotor::Jumpjet),
                u.vehicle.crashable
            );
        }
    }
}

fn issue_448(import: &glasnost_convert::Import, rules: &SimRules) {
    let s = beams::Settings::read(import, rules, &[]);
    for (kind, list) in [
        (Kind::Infantry, &rules.infantry),
        (Kind::Vehicle, &rules.vehicles),
        (Kind::Aircraft, &rules.aircraft),
    ] {
        for (i, u) in list.iter().enumerate() {
            let c = &u.combat;
            if c.airstrike_team <= 0 {
                continue;
            }
            for (rank, slots) in [("normal", &c.slots), ("elite", &c.elite_slots)] {
                let Some(wi) = slots[1] else {
                    println!("#448 designator {kind:?} {i} {rank}: no secondary");
                    continue;
                };
                let wr = rules.combat.weapon(wi);
                let airstrike = wr
                    .warhead
                    .is_some_and(|h| rules.combat.warhead(h).airstrike);
                let zero = wr.laser_colours.iter().all(|c| c.iter().all(|&x| x == 0));
                println!(
                    "#448 designator {kind:?} {i} {rank}: secondary airstrike {airstrike} sim laser {} duration > 0 {} house colour {} colours all 0 {zero} drawing flags {:?}",
                    wr.laser,
                    wr.laser_duration > 0,
                    wr.house_colour,
                    s.flags.get(wi as usize)
                );
            }
        }
    }
}

#[test]
#[ignore = "local lane: reads the install named by GLASNOST_GAME_DIR"]
fn s34_install_checks() {
    let Some(dir) = std::env::var_os("GLASNOST_GAME_DIR") else {
        eprintln!("s34_lane: GLASNOST_GAME_DIR is not set; nothing to do");
        return;
    };
    let (import, rules, vfs) = install(Path::new(&dir));
    issue_446(&rules);
    issue_448(&import, &rules);
    issue_444_names(&import, &rules, &vfs);
    issue_444_cycle(&rules);
}
