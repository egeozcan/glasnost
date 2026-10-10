//! Claim tests for S-0009's pre-placed objects (claims 36 to 43) on
//! synthetic data: starting health, facing, initial state, upgrades,
//! behaviour names, dropped entries and their draws, and placement order.

mod common;

use common::{Scene, TRACK, cell, unit_type};
use glasnost_map::objects::{EntryFields, Placed};
use glasnost_map::{Aircraft, Infantry, Structure, Unit};
use glasnost_rules::sim::{BuildingType, TerrainObjectType};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::world::World;

fn placed(owner: &str, ty: &str, c: glasnost_map::CellXY, health: i32, facing: i32) -> Placed {
    Placed {
        key: b"k".to_vec(),
        owner: owner.as_bytes().to_vec(),
        type_name: ty.as_bytes().to_vec(),
        health,
        cell: c,
        facing,
        raw: Vec::new(),
    }
}

/// A scene with a vehicle type TANK (strength 100), an infantry type MAN
/// (strength 100), an aircraft type JET, building types HUT (strength 300)
/// and ADDON, and a terrain object type TREE.
fn scene() -> Scene {
    let mut s = Scene::new();
    s.vehicle("TANK", unit_type(TRACK, 4, 5));
    s.infantry("MAN", unit_type(common::FOOT, 4, 5));
    s.register("aircraft", "JET");
    s.ruleset
        .sim
        .aircraft
        .insert("JET".into(), unit_type(TRACK, 4, 5));
    for (name, strength) in [("HUT", 300), ("ADDON", 50)] {
        s.register("buildings", name);
        s.ruleset.sim.buildings.insert(
            name.into(),
            BuildingType {
                strength: Some(strength),
                foundation: Some([1, 1]),
                ..Default::default()
            },
        );
    }
    s.register("terrain_objects", "TREE");
    s.ruleset
        .sim
        .terrain_objects
        .insert("TREE".into(), TerrainObjectType::default());
    s
}

fn only(w: &World, kind: Kind) -> &glasnost_sim::world::Object {
    let v: Vec<_> = w.objects.values().filter(|o| o.kind == kind).collect();
    assert_eq!(v.len(), 1, "{kind:?}");
    v[0]
}

/// spec: S-0009 claim 37 (starting health from the health value; near-full rule; low ends)
#[test]
fn claim_37_starting_health() {
    let mut s = scene();
    s.map.objects.units.push(Unit {
        placed: placed("Alpha", "TANK", cell(0, 0), 128, 0),
    });
    s.map.objects.infantry.push(Infantry {
        placed: placed("Alpha", "MAN", cell(2, 0), 0, 0),
        sub_cell: 0,
    });
    s.map.objects.structures.push(Structure {
        placed: placed("Alpha", "HUT", cell(4, 0), 400, 0),
    });
    let w = s.build();
    assert_eq!(only(&w, Kind::Vehicle).health, 50);
    assert_eq!(only(&w, Kind::Infantry).health, 1);
    assert_eq!(only(&w, Kind::Building).health, 300);

    // 254 of 100 gives 99.2 → 99 > 97: full strength.
    let mut s = scene();
    s.map.objects.units.push(Unit {
        placed: placed("Alpha", "TANK", cell(0, 0), 254, 0),
    });
    assert_eq!(only(&s.build(), Kind::Vehicle).health, 100);
    // quirks.near_full_health_rounds_up off: the computed value stays.
    s.ruleset.globals.quirks.near_full_health_rounds_up = false;
    assert_eq!(only(&s.build(), Kind::Vehicle).health, 99);
}

/// spec: S-0009 claim 39 (facing 256 × (v mod 256); turrets start at the body facing; infantry field 8)
#[test]
fn claim_39_facing() {
    let mut s = scene();
    s.map.objects.units.push(Unit {
        placed: placed("Alpha", "TANK", cell(0, 0), 256, 300),
    });
    s.map.objects.infantry.push(Infantry {
        placed: placed("Alpha", "MAN", cell(2, 0), 256, -64),
        sub_cell: 0,
    });
    let w = s.build();
    let v = only(&w, Kind::Vehicle);
    assert_eq!(v.body.value(0), 44 * 256);
    assert_eq!(v.turret.value(0), 44 * 256);
    assert_eq!(only(&w, Kind::Infantry).body.value(0), 192 * 256);
}

/// spec: S-0009 claim 40 (a pre-placed building starts in guard; upgrades placed right after it, carried over)
#[test]
fn claim_40_building_initial_state_and_upgrades() {
    let mut s = scene();
    for (i, c) in [cell(0, 0), cell(3, 0)].into_iter().enumerate() {
        s.map.objects.structures.push(Structure {
            placed: placed("Alpha", "HUT", c, 256, 0),
        });
        let mut f = EntryFields {
            upgrade_count: Some(1),
            ..Default::default()
        };
        if i == 0 {
            f.upgrades[0] = Some(b"addon".to_vec());
        }
        s.map.objects.structure_fields.push(f);
    }
    let w = s.build();
    let b: Vec<_> = w
        .objects
        .values()
        .filter(|o| o.kind == Kind::Building)
        .collect();
    // HUT, ADDON, HUT, ADDON (the second carried over), in creation order.
    let names: Vec<_> = b
        .iter()
        .map(|o| w.rules().buildings[o.type_index as usize].name.clone())
        .collect();
    assert_eq!(names, ["HUT", "ADDON", "HUT", "ADDON"]);
    assert_eq!(b[1].pos, b[0].pos);
    assert_eq!(b[1].health, 50, "upgrades start at full health");
    assert!(b.iter().all(|o| o.beh.current == Some(Behaviour::Guard)));
    // quirks.upgrade_fields_carry_over off: the second building gets no upgrade.
    s.ruleset.globals.quirks.upgrade_fields_carry_over = false;
    let w = s.build();
    assert_eq!(w.objects.len(), 3);
}

/// spec: S-0009 claim 41 (the behaviour name; a name that matches none means none)
#[test]
fn claim_41_behaviour_name() {
    let mut s = scene();
    for (c, name) in [
        (cell(0, 0), "sleep"),
        (cell(2, 0), "Area Guard"),
        (cell(4, 0), "Dance"),
    ] {
        s.map.objects.units.push(Unit {
            placed: placed("Alpha", "TANK", c, 256, 0),
        });
        s.map.objects.unit_fields.push(EntryFields {
            behaviour: Some(name.as_bytes().to_vec()),
            ..Default::default()
        });
    }
    let w = s.build();
    let cur: Vec<_> = w
        .objects
        .values()
        .filter(|o| o.kind == Kind::Vehicle)
        .map(|o| o.beh.current)
        .collect();
    assert_eq!(
        cur,
        [Some(Behaviour::Sleep), Some(Behaviour::AreaGuard), None]
    );
}

/// spec: S-0009 claims 36 and 38 (dropped entries and the creation draws they take)
#[test]
fn claims_36_38_dropped_entries_draws() {
    let reference = {
        let mut s = scene();
        s.map.objects.units.push(Unit {
            placed: placed("Alpha", "TANK", cell(0, 0), 256, 0),
        });
        s.build()
    };
    // A short vehicle entry takes one creation draw and is dropped.
    let mut s = scene();
    s.map.objects.units.push(Unit {
        placed: placed("Alpha", "TANK", cell(0, 0), 256, 0),
    });
    s.map.objects.unit_fields.push(EntryFields {
        short: true,
        ..Default::default()
    });
    // A short building entry takes none.
    s.map.objects.structures.push(Structure {
        placed: placed("Alpha", "HUT", cell(2, 0), 256, 0),
    });
    s.map.objects.structure_fields.push(EntryFields {
        short: true,
        ..Default::default()
    });
    let w = s.build();
    assert!(w.objects.is_empty());
    assert_eq!(w.rng, reference.rng);
    // A vehicle on a non-legal cell takes its creation draw and is dropped.
    let mut s = scene();
    s.map.objects.units.push(Unit {
        placed: placed("Alpha", "TANK", glasnost_map::CellXY::new(0, 0), 256, 0),
    });
    let w = s.build();
    assert!(w.objects.is_empty());
    assert_eq!(w.rng, reference.rng);
    // An unknown type or owner takes no draw; the type matches without regard to letter case.
    let mut s = scene();
    s.map.objects.units.push(Unit {
        placed: placed("Alpha", "NOPE", cell(0, 0), 256, 0),
    });
    s.map.objects.units.push(Unit {
        placed: placed("Gamma", "TANK", cell(0, 0), 256, 0),
    });
    s.map.objects.units.push(Unit {
        placed: placed("Alpha", "tank", cell(0, 0), 256, 0),
    });
    let w = s.build();
    assert_eq!(w.objects.len(), 1);
    assert_eq!(w.rng, reference.rng);
}

/// spec: S-0009 claims 42 and 43 (terrain objects first at their cell centre; then vehicles, aircraft, infantry, buildings)
#[test]
fn claims_42_43_placement_order() {
    let mut s = scene();
    s.map.objects.structures.push(Structure {
        placed: placed("Alpha", "HUT", cell(0, 0), 256, 0),
    });
    s.map.objects.infantry.push(Infantry {
        placed: placed("Alpha", "MAN", cell(2, 0), 256, 0),
        sub_cell: 0,
    });
    s.map.objects.aircraft.push(Aircraft {
        placed: placed("Alpha", "JET", cell(4, 0), 256, 0),
    });
    s.map.objects.units.push(Unit {
        placed: placed("Alpha", "TANK", cell(6, 0), 256, 0),
    });
    s.place_terrain("tree", cell(8, 0));
    let w = s.build();
    let kinds: Vec<_> = w.objects.values().map(|o| o.kind).collect();
    assert_eq!(
        kinds,
        [
            Kind::Terrain,
            Kind::Vehicle,
            Kind::Aircraft,
            Kind::Infantry,
            Kind::Building
        ]
    );
    let t = only(&w, Kind::Terrain);
    let c = cell(8, 0);
    assert_eq!((t.pos.x, t.pos.y), (256 * c.x + 128, 256 * c.y + 128));
}

/// A scene set up as a skirmish: houses Alpha and Beta are players on
/// random positions; no sandbox.
fn skirmish() -> Scene {
    let mut s = scene();
    s.setup.sandbox = false;
    s
}

fn load(s: &Scene) -> Result<World, glasnost_sim::load::LoadError> {
    glasnost_sim::load::new_game(
        s.sim_rules(),
        &s.map,
        std::sync::Arc::new(s.catalogue.clone()),
        &s.setup,
    )
}

/// spec: S-0009 claim 18 (random start cells: candidates (v + 1, u + 11) from two ranged draws; on blank cells the candidate itself when usable)
#[test]
fn claim_18_random_start_on_blank_cells() {
    let s = scene();
    let mut w = s.build();
    // The early phase's view: every legal cell a blank cell.
    w.cells = glasnost_sim::cells::Cells::new(glasnost_map::CellGrid::new(s.map.footprint));
    let mut probe = w.rng.clone();
    let span = 2 * common::SIZE;
    let u = probe.ranged(0, span - 11, true);
    let v = probe.ranged(10, span - 11, true);
    let candidate = glasnost_map::CellXY::new(v + 1, u + 11);
    let list = glasnost_sim::starts::build_list(&mut w, &s.map, 1).expect("a start cell");
    if w.cells.usable(candidate) {
        assert_eq!(list, vec![candidate]);
        assert_eq!(w.rng, probe, "the search takes no draw");
    }
    assert!(w.cells.usable(list[0]));
}

/// spec: S-0009 claims 17 and 18 (defined start waypoints need no random cell; a map without a clear 8 × 8 area is refused by our engine)
#[test]
fn claim_18_no_start_cell_refused() {
    let mut s = skirmish();
    let all: Vec<_> = s.map.cells.iter().map(|(c, _)| c).collect();
    for c in all {
        // Water (terrain code 9), which the start-up land row closes to tracks.
        s.set_cell(c, common::code_tile(9), 0);
    }
    assert_eq!(
        load(&s).err(),
        Some(glasnost_sim::load::LoadError::NoStartCell)
    );
    s.map.waypoints.insert(0, (40, 40));
    s.map.waypoints.insert(1, (50, 40));
    let w = load(&s).expect("start waypoints for both players");
    let mut starts: Vec<_> = w.houses.iter().map(|h| h.start).collect();
    starts.sort();
    assert_eq!(starts, [Some(0), Some(1)]);
}

/// The starting-unit rules of claims 45 and 46: base unit MCV, TANK (cost
/// 100) and MAN (cost 50), every type owned by both countries.
fn starting_scene() -> Scene {
    let mut s = skirmish();
    for (name, cost) in [("MCV", 500), ("TANK", 100)] {
        let mut t = unit_type(TRACK, 4, 5);
        t.cost = Some(cost);
        t.tech_level = Some(1);
        t.owner = Some(vec!["Alphaland".into(), "Betaland".into()]);
        s.vehicle(name, t);
    }
    let mut man = unit_type(common::FOOT, 4, 5);
    man.cost = Some(50);
    man.tech_level = Some(1);
    man.owner = Some(vec!["Alphaland".into(), "Betaland".into()]);
    s.infantry("MAN", man);
    s.ruleset.sim.general.base_unit = Some(vec!["MCV".into()]);
    s.map.waypoints.insert(0, (40, 40));
    s.map.waypoints.insert(1, (52, 40));
    s.setup.unit_count = 2;
    s.setup.bases = true;
    s
}

/// spec: S-0009 claim 45 (base unit, budget B = trunc((trunc(n / 2) + C) / n) × U, vehicles while R > trunc(B / 3), then infantry); claim 46 (the base unit at its base cell, the others by the placement search, facing 0, guard)
#[test]
fn claims_45_46_starting_units() {
    let s = starting_scene();
    let w = load(&s).expect("loads");
    let r = w.rules_arc();
    // n = 2 (TANK, MAN; MCV is a base unit), C = 150: B = trunc(151 / 2) × 2 = 150.
    for h in 0..2u8 {
        let mut names: Vec<String> = w
            .objects
            .values()
            .filter(|o| o.owner == Some(glasnost_sim::world::HouseId(h)))
            .map(|o| r.unit(o.kind, o.type_index).unwrap().name.clone())
            .collect();
        names.sort();
        assert_eq!(names, ["MAN", "MCV", "TANK"], "house {h}");
        assert_eq!(w.houses[usize::from(h)].credits, 0);
    }
    let base_cells: Vec<_> = w
        .objects
        .values()
        .filter(|o| r.unit(o.kind, o.type_index).unwrap().name == "MCV")
        .map(|o| o.pos.cell())
        .collect();
    assert!(base_cells.contains(&glasnost_map::CellXY::new(40, 40)));
    assert!(base_cells.contains(&glasnost_map::CellXY::new(52, 40)));
    for o in w.objects.values() {
        assert_eq!(o.beh.current, Some(Behaviour::Guard));
        assert_eq!(o.body.value(0), 0);
    }
}

/// spec: S-0009 claim 45 (early stop: no base-unit type qualifies for the house; no units for it or later houses, and no final draw)
#[test]
fn claim_45_early_stop() {
    let mut s = starting_scene();
    s.ruleset.sim.vehicles.get_mut("MCV").unwrap().owner = Some(vec!["Betaland".into()]);
    let stopped = load(&s).expect("loads");
    assert!(stopped.objects.is_empty());
    // Without bases the step runs to the end, with the final draw.
    s.setup.bases = false;
    let full = load(&s).expect("loads");
    assert!(!full.objects.is_empty());
}

/// spec: S-0009 claim 41 (follow values make placed vehicles follower cars of the placed vehicle they name)
#[test]
fn claim_41_follower_links_stored() {
    let mut s = scene();
    for (k, follow) in [(0, -1), (2, 0)] {
        s.map.objects.units.push(Unit {
            placed: placed("Alpha", "TANK", cell(k, 0), 256, 0),
        });
        s.map.objects.unit_fields.push(EntryFields {
            follow: Some(follow),
            ..Default::default()
        });
    }
    let w = s.build();
    let v: Vec<_> = w
        .objects
        .values()
        .filter(|o| o.kind == Kind::Vehicle)
        .collect();
    assert_eq!(v[0].leader, None);
    assert_eq!(v[1].leader, Some(v[0].id));
}

/// spec: S-0009 claim 27 and Import mapping (the neutral and special houses answer to their country names); S-0208 Terms (they are not human-controlled)
#[test]
fn claim_27_sandbox_neutral_house() {
    let mut s = scene();
    s.register("countries", "Neutral");
    s.ruleset
        .sim
        .countries
        .insert("Neutral".into(), Default::default());
    s.map.objects.units.push(Unit {
        placed: placed("Neutral", "TANK", cell(0, 0), 256, 0),
    });
    s.map.objects.units.push(Unit {
        placed: placed("Alpha", "TANK", cell(2, 0), 256, 0),
    });
    let (setup, _) = glasnost_sim::load::sandbox_setup(&s.map, &s.sim_rules(), 1);
    assert_eq!(setup.houses[0].name, "Neutral");
    assert!(!setup.houses[0].human);
    assert_eq!(
        setup.houses[0].role,
        glasnost_sim::world::HouseRole::Neutral
    );
    assert!(setup.houses[1].human);
}

/// spec: S-0009 claim 12 (an overlay is created only where the cell's slope shape as it stands is at most 4, or for the type at position 178; only an earlier placement's recalculation can make it stand above 4)
#[test]
fn claim_12_slope_condition() {
    let build = |slope: u8| {
        let mut s = scene();
        for i in 0..200 {
            let name = format!("O{i}");
            s.register("overlays", &name);
            s.ruleset.sim.overlays.insert(
                name,
                glasnost_rules::sim::OverlayType {
                    has_art: Some(true),
                    ..Default::default()
                },
            );
        }
        // A marker of position 124 (its end row runs along x) west of a sloped cell holding type 7.
        s.cell_mut(cell(0, 0)).overlay = 124;
        s.set_cell(cell(1, 0), common::slope_tile(slope), 0);
        s.cell_mut(cell(1, 0)).overlay = 7;
        // A sloped cell no placement recalculated before: its overlay is created.
        s.set_cell(cell(5, 0), common::slope_tile(9), 0);
        s.cell_mut(cell(5, 0)).overlay = 7;
        s.build()
    };
    let w = build(9);
    assert_eq!(w.cells.cell(cell(1, 0)).unwrap().overlay, 96, "refused");
    assert_eq!(w.cells.cell(cell(5, 0)).unwrap().overlay, 7);
    let w = build(1);
    assert_eq!(w.cells.cell(cell(1, 0)).unwrap().overlay, 7);
}

/// spec: S-0009 claims 41 and 45 (starting units, credits, house roles and follower-car links are game state: the state hash survives a save/load round trip)
#[test]
fn claims_41_45_state_round_trip() {
    let mut s = starting_scene();
    s.map.objects.units.push(Unit {
        placed: placed("Alpha", "TANK", cell(-8, 6), 256, 0),
    });
    s.map.objects.units.push(Unit {
        placed: placed("Alpha", "TANK", cell(-6, 6), 256, 0),
    });
    s.map.objects.unit_fields.push(EntryFields::default());
    s.map.objects.unit_fields.push(EntryFields {
        follow: Some(0),
        ..Default::default()
    });
    s.setup.unit_count = 1;
    let w = load(&s).expect("loads");
    let bytes = glasnost_sim::save::save(&w);
    let l = glasnost_sim::save::load(
        &bytes,
        s.sim_rules(),
        std::sync::Arc::new(s.catalogue.clone()),
    )
    .expect("loads back");
    assert_eq!(
        glasnost_sim::save::state_hash(&w),
        glasnost_sim::save::state_hash(&l)
    );
    assert!(l.objects.values().any(|o| o.leader.is_some()));
}

/// spec: S-0009 claims 28 and 45 (a neutral house that is not passive takes a position and receives starting units with tech level 1, in area guard; a player's in guard)
#[test]
fn claim_45_neutral_house_area_guard() {
    let mut s = starting_scene();
    s.register("countries", "Neutral");
    s.ruleset
        .sim
        .countries
        .insert("Neutral".into(), Default::default());
    for t in ["MCV", "TANK"] {
        let v = s.ruleset.sim.vehicles.get_mut(t).unwrap();
        v.owner.as_mut().unwrap().push("Neutral".into());
    }
    // Beta becomes the neutral house.
    s.setup.houses[1].name = "Neutral".into();
    s.setup.houses[1].country = "Neutral".into();
    s.setup.houses[1].human = false;
    s.setup.houses[1].role = glasnost_sim::world::HouseRole::Neutral;
    let w = load(&s).expect("loads");
    let behaviours = |h: u8| -> Vec<Option<Behaviour>> {
        w.objects
            .values()
            .filter(|o| o.owner == Some(glasnost_sim::world::HouseId(h)))
            .map(|o| o.beh.current)
            .collect()
    };
    let neutral = behaviours(1);
    assert!(!neutral.is_empty());
    assert!(neutral.iter().all(|b| *b == Some(Behaviour::AreaGuard)));
    assert!(behaviours(0).iter().all(|b| *b == Some(Behaviour::Guard)));
}

/// spec: S-0009 claim 19 (the first house on random takes R(0, N − 1); a later one the free position whose distance sum is largest)
#[test]
fn claim_19_farthest_sum() {
    let mut s = skirmish();
    for (k, x) in [(0u16, 30i16), (1, 40), (2, 60)] {
        s.map.waypoints.insert(k, (x, 40));
    }
    let w = load(&s).expect("loads");
    let first = w.houses[0].start.expect("a position");
    let second = w.houses[1].start.expect("a position");
    // Positions 0, 1, 2 lie at x = 30, 40, 60: the farthest from 0 or 1 is 2, from 2 it is 0.
    let want = if first == 2 { 0 } else { 2 };
    assert_eq!(second, want, "first {first}");
}

/// spec: S-0009 claim 20 (the early phase's random-position draw comes before the initial fill's draws; quirks.double_start_resolution off: one assignment after the terrain)
#[test]
fn claim_20_early_draws_before_the_fill() {
    let mut s = skirmish();
    s.map.fill = glasnost_map::Fill::Water;
    s.map.waypoints.insert(0, (30, 40));
    s.map.waypoints.insert(1, (50, 40));
    // N = 2 and no random entry: the position draw R(0, 1) takes one raw value.
    let mut r = glasnost_sim::rng::Rng::new(s.setup.seed);
    let raws: Vec<u32> = (0..3200).map(|_| r.raw()).collect();
    let fill = 40 * (2 * 40 - 1); // H · (2W − 1) visited cells, one draw each
    let w = load(&s).expect("loads");
    assert_eq!(w.houses[0].start, Some(raws[0] % 2));
    s.ruleset.globals.quirks.double_start_resolution = false;
    let w = load(&s).expect("loads");
    assert_eq!(w.houses[0].start, Some(raws[fill] % 2));
}

/// spec: S-0009 claim 33 (the early phase judges passivity by the base rules' country data, quirks.early_phase_stale_country_data; a house passive in the late phase receives no starting units)
#[test]
fn claim_33_early_phase_country_data() {
    let mut s = starting_scene();
    let beta = s.ruleset.sim.countries.get_mut("Betaland").unwrap();
    beta.multiplay_passive = Some(true);
    beta.multiplay_passive_base = Some(false);
    let w = load(&s).expect("loads");
    assert!(
        w.houses[1].start.is_some(),
        "a position taken in the early phase"
    );
    assert!(
        !w.objects
            .values()
            .any(|o| o.owner == Some(glasnost_sim::world::HouseId(1))),
        "passive in the late phase: no starting units"
    );
    s.ruleset.globals.quirks.early_phase_stale_country_data = false;
    let w = load(&s).expect("loads");
    assert_eq!(w.houses[1].start, None);
}

/// spec: S-0009 claim 27 (an object owned by a country name other than the neutral and special houses' is dropped; quirks.skirmish_drops_country_owned_objects off: it goes to the first player of that country, and "player at position X" to that player)
#[test]
fn claim_27_country_owned_objects() {
    let mut s = skirmish();
    s.map.waypoints.insert(0, (30, 40));
    s.map.waypoints.insert(1, (50, 40));
    for owner in ["Betaland", "<Player @ A>"] {
        s.map.objects.units.push(Unit {
            placed: placed(owner, "TANK", cell(0, 0), 256, 0),
        });
    }
    assert!(load(&s).expect("loads").objects.is_empty());
    s.ruleset
        .globals
        .quirks
        .skirmish_drops_country_owned_objects = false;
    let w = load(&s).expect("loads");
    let owners: Vec<_> = w.objects.values().map(|o| o.owner).collect();
    let at_a = w
        .houses
        .iter()
        .position(|h| h.start == Some(0))
        .map(|i| glasnost_sim::world::HouseId(i as u8));
    assert_eq!(owners, [Some(glasnost_sim::world::HouseId(1)), at_a]);
}

// spec: S-0009 claim 38 (a vehicle whose bridge flag is set stands on the bridge deck, else on the ground)
// spec: S-0207 claim 5 (on a deck: the surface height plus 416 leptons)
#[test]
fn claim_38_bridge_flag_puts_a_vehicle_on_the_deck() {
    let mut s = scene();
    // Invented overlay types up to list index 24, a high-bridge index
    // (S-0211 claim 15), with artwork so loading keeps them.
    for i in 0..=24 {
        let name = format!("OV{i}");
        s.register("overlays", &name);
        s.ruleset.sim.overlays.insert(
            name,
            glasnost_rules::sim::OverlayType {
                has_art: Some(true),
                ..Default::default()
            },
        );
    }
    s.cell_mut(cell(0, 0)).overlay = 24;
    for (c, flag) in [(cell(0, 0), true), (cell(0, 6), true), (cell(0, 8), false)] {
        s.map.objects.units.push(Unit {
            placed: placed("Alpha", "TANK", c, 256, 0),
        });
        s.map.objects.unit_fields.push(EntryFields {
            bridge: Some(flag),
            ..Default::default()
        });
    }
    let w = s.build();
    let at = |c: glasnost_map::CellXY| {
        w.objects
            .values()
            .find(|o| o.kind == Kind::Vehicle && o.pos.cell() == c)
            .expect("vehicle")
    };
    let on_deck = at(cell(0, 0));
    assert_eq!(on_deck.layer, glasnost_sim::world::Layer::Deck);
    assert_eq!(
        on_deck.pos.z,
        w.surface_at(on_deck.pos.x, on_deck.pos.y) + 416
    );
    // The flag on a cell without a deck, and no flag: on the ground.
    for c in [cell(0, 6), cell(0, 8)] {
        let o = at(c);
        assert_eq!(o.layer, glasnost_sim::world::Layer::Ground);
        assert_eq!(o.pos.z, w.surface_at(o.pos.x, o.pos.y));
    }
}
