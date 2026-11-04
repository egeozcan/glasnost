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

/// spec: S-0405 claim 18 (a switched-on field of 0 starts a building off only with a base drain above 0 or the needs-power flag; the firestorm-wall flag does not decide it; field 1 or absent: on)
#[test]
fn s0405_claim_18_map_switched_off() {
    // (name, power, needs power, firestorm wall, starts off with field 0)
    let rows = [
        ("PP", 100, false, false, false),
        ("CC", -40, false, false, true),
        ("NN", 10, true, false, true),
        ("FF", -40, false, true, true),
        ("GG", 100, false, true, false),
    ];
    for field in [Some(false), Some(true), None] {
        let mut s = scene();
        for (i, &(name, power, needs, fire, _)) in rows.iter().enumerate() {
            s.register("buildings", name);
            let mut b = BuildingType {
                strength: Some(100),
                foundation: Some([1, 1]),
                powered: Some(needs),
                firestorm_wall: Some(fire),
                ..Default::default()
            };
            b.econ.power = Some(power);
            s.ruleset.sim.buildings.insert(name.into(), b);
            s.map.objects.structures.push(Structure {
                placed: placed("Alpha", name, cell(2 * i as i32, 0), 256, 0),
            });
            s.map.objects.structure_fields.push(EntryFields {
                switched_on: field,
                ..Default::default()
            });
        }
        let w = s.build();
        for &(name, _, _, _, off) in &rows {
            let o = w
                .objects
                .values()
                .find(|o| w.rules().buildings[o.type_index as usize].name == name)
                .unwrap();
            let want = off && field == Some(false);
            assert_eq!(o.econ.building.switched_off, want, "{name} {field:?}");
        }
    }
}

/// spec: S-0314 claim 13 and Import mapping, S-0009 claim 40 (a map building's computer-repair field, nonzero, sets its computer-repair flag; zero or absent leaves it clear)
#[test]
fn s0314_claim_13_map_computer_repair_flag() {
    let mut s = scene();
    s.register("buildings", "RR");
    s.ruleset.sim.buildings.insert(
        "RR".into(),
        BuildingType {
            strength: Some(100),
            foundation: Some([1, 1]),
            ..Default::default()
        },
    );
    let fields = [Some(true), Some(false), None];
    for (i, f) in fields.iter().enumerate() {
        s.map.objects.structures.push(Structure {
            placed: placed("Alpha", "RR", cell(2 * i as i32, 0), 256, 0),
        });
        s.map.objects.structure_fields.push(EntryFields {
            ai_repair: *f,
            ..Default::default()
        });
    }
    let w = s.build();
    let mut flags: Vec<(i32, bool)> = w
        .objects
        .values()
        .filter(|o| o.kind == Kind::Building)
        .map(|o| (o.pos.cell().x, o.combat.computer_repair))
        .collect();
    flags.sort();
    let got: Vec<bool> = flags.iter().map(|f| f.1).collect();
    assert_eq!(got, [true, false, false]);
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
    // spec: S-0215 claim 1 (at game set-up: guard queued, no current behaviour, the idle body)
    assert!(
        b.iter()
            .all(|o| o.beh.current.is_none() && o.beh.queued == Some(Behaviour::Guard))
    );
    assert!(b.iter().all(|o| o.econ.idle_body.is_some()));
    // quirks.upgrade_fields_carry_over off: the second building gets no upgrade.
    s.ruleset.globals.quirks.upgrade_fields_carry_over = false;
    let w = s.build();
    assert_eq!(w.objects.len(), 3);
}

/// spec: S-0009 claim 38 (a pre-placed building's bounding rectangle starts at its cell; its occupied cells are S-0406 claim 2's: the 8-cell 3 × 3 word lacks (2, 1), the zero-size word has none)
#[test]
fn claim_38_building_foundation_cells() {
    let mut s = scene();
    for (name, size, refinery) in [("REFY", [3, 3], true), ("NIL", [0, 0], false)] {
        s.register("buildings", name);
        s.ruleset.sim.buildings.insert(
            name.into(),
            BuildingType {
                strength: Some(100),
                foundation: Some(size),
                foundation_refinery: Some(refinery),
                ..Default::default()
            },
        );
    }
    for (ty, c) in [("REFY", cell(2, 2)), ("NIL", cell(8, 8))] {
        s.map.objects.structures.push(Structure {
            placed: placed("Alpha", ty, c, 256, 0),
        });
        s.map.objects.structure_fields.push(EntryFields::default());
    }
    let w = s.build();
    let held = |x: i32, y: i32| {
        w.cells
            .list(cell(x, y), glasnost_sim::world::Layer::Ground)
            .iter()
            .filter(|&&id| w.obj(id).is_some_and(|o| o.kind == Kind::Building))
            .count()
    };
    let mut occupied = Vec::new();
    for y in 0..12 {
        for x in 0..12 {
            if held(x, y) > 0 {
                occupied.push((x - 2, y - 2));
            }
        }
    }
    let expected: Vec<(i32, i32)> = (0..3)
        .flat_map(|j| (0..3).map(move |i| (i, j)))
        .filter(|&c| c != (2, 1))
        .collect();
    assert_eq!(occupied, expected);
    assert_eq!(
        w.objects
            .values()
            .filter(|o| o.kind == Kind::Building)
            .count(),
        2
    );
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
    // spec: S-0412 claim 2 (a credits setting of 0, so the balance is the left-over budget alone)
    s.setup.skirmish.credits = Some(0);
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
    // An AI house, a computer player, so that the cleanup of claim 28 does not run (S-0412 claim 12).
    s.setup.houses[0].ai = true;
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

/// spec: S-0009 claim 45 (the starting units of a house that is not human-controlled start in area guard) and glossary (a test-only computer-controlled player house)
#[test]
fn claim_45_test_house_area_guard() {
    let mut s = starting_scene();
    // Beta stays a player house but is marked not human-controlled.
    s.setup.houses[1].human = false;
    s.setup.houses[0].ai = true;
    let w = load(&s).expect("loads");
    let behaviours = |h: u8| -> Vec<Option<Behaviour>> {
        w.objects
            .values()
            .filter(|o| o.owner == Some(glasnost_sim::world::HouseId(h)))
            .map(|o| o.beh.current)
            .collect()
    };
    let test = behaviours(1);
    assert!(!test.is_empty());
    assert!(test.iter().all(|b| *b == Some(Behaviour::AreaGuard)));
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

/// The raw values that `rounds` house-creation rounds of `houses` houses
/// take from the start of the stream of `seed` (S-0202 claim 14: one
/// R(450, 1800) per house and round, by claim 4's range rule).
fn house_rounds(seed: u32, houses: usize, rounds: usize, half: bool) -> usize {
    let mut r = glasnost_sim::rng::Rng::new(seed);
    for _ in 0..houses * rounds {
        // spec: S-0202 claim 14 (house creation: one ranged draw R(450, 1800) per house)
        let _ = r.ranged(450, 1800, half);
    }
    r.taken() as usize
}

/// spec: S-0009 claim 20 (the early phase's random-position draw comes before the initial fill's draws; quirks.double_start_resolution off: one assignment after the terrain); S-0202 claim 14 (the first house-creation round before the early phase, the second after it and before the fill)
#[test]
fn claim_20_early_draws_before_the_fill() {
    let mut s = skirmish();
    s.map.fill = glasnost_map::Fill::Water;
    s.map.waypoints.insert(0, (30, 40));
    s.map.waypoints.insert(1, (50, 40));
    let half = s.ruleset.globals.quirks.ranged_draw_half_span;
    let h = s.setup.houses.len();
    // N = 2 and no random entry: the position draw R(0, 1) takes one raw value.
    let mut r = glasnost_sim::rng::Rng::new(s.setup.seed);
    let raws: Vec<u32> = (0..4000).map(|_| r.raw()).collect();
    let fill = 40 * (2 * 40 - 1); // H · (2W − 1) visited cells, one draw each
    let w = load(&s).expect("loads");
    // spec: S-0202 claim 14 (the position draw directly after the first round's raw values)
    let first = house_rounds(s.setup.seed, h, 1, half);
    assert_eq!(w.houses[0].start, Some(raws[first] % 2));
    s.ruleset.globals.quirks.double_start_resolution = false;
    let w = load(&s).expect("loads");
    // spec: S-0202 claim 14 (with no early phase, both rounds come before the fill)
    let both = house_rounds(s.setup.seed, h, 2, half);
    assert_eq!(w.houses[0].start, Some(raws[both + fill] % 2));
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

/// spec: S-0009 claim 12 (Cell animations: an overlay whose type has a cell
/// animation is created even without artwork, and its animation's creation
/// draws, S-0202 claim 10, come before those of the tile animation that the
/// same placement's recalculation of the cell starts)
#[test]
fn claim_12_cell_animation_draws_before_the_tile_animation() {
    use glasnost_map::terrain::TileAnim;
    use glasnost_rules::sim::{AnimationType, OverlayType};
    use glasnost_sim::rng::Rng;
    let mut s = scene();
    s.register("overlays", "GLOW");
    s.ruleset.sim.overlays.insert(
        "GLOW".into(),
        OverlayType {
            has_art: Some(false),
            cell_anim: Some("SPARK".into()),
            ..Default::default()
        },
    );
    s.register("animations", "SPARK");
    s.register("animations", "ROCK");
    // SPARK: a random stage delay (one ranged draw over 450 … 900); ROCK: a bouncing type.
    s.ruleset.sim.animations.insert(
        "SPARK".into(),
        AnimationType {
            random_rate: Some([2, 1]),
            ..Default::default()
        },
    );
    s.ruleset.sim.animations.insert(
        "ROCK".into(),
        AnimationType {
            bouncer: Some(true),
            ..Default::default()
        },
    );
    s.catalogue.tiles[1].anim = Some(TileAnim {
        name: "ROCK".into(),
        attaches_to: 0,
        ..TileAnim::default()
    });
    s.set_cell(cell(0, 0), 1, 0);
    s.cell_mut(cell(0, 0)).overlay = 0;
    let w = s.build();
    assert_eq!(
        w.cells.cell(cell(0, 0)).unwrap().overlay,
        0,
        "created without artwork"
    );
    let bounce = |r: &mut Rng| {
        for _ in 0..3 {
            let _ = r.raw();
        }
        for _ in 0..3 {
            let _ = r.ranged(-65535, 65535, true);
        }
    };
    let mut r = Rng::new(s.setup.seed);
    // The overlay stream: the cell animation, then the tile animation of the placement's recalculation.
    let _ = r.ranged(450, 900, true);
    bounce(&mut r);
    // The second pass's tile animation, then the final draw.
    bounce(&mut r);
    let _ = r.ranged(0, 65535, true);
    assert_eq!(w.rng, r);

    // No cell animation: no creation draw of its own.
    let glow = s.ruleset.sim.overlays.get_mut("GLOW").unwrap();
    glow.cell_anim = None;
    glow.has_art = Some(true);
    let w = s.build();
    let mut r = Rng::new(s.setup.seed);
    bounce(&mut r);
    bounce(&mut r);
    let _ = r.ranged(0, 65535, true);
    assert_eq!(w.rng, r);
}

/// spec: S-0009 claim 42 (a terrain object starts with its type's strength as its health; a type whose strength is unset starts with health −1, S-0102 claim 23)
#[test]
fn claim_42_terrain_object_starting_health() {
    let mut s = scene();
    s.register("terrain_objects", "OAK");
    s.ruleset.sim.terrain_objects.insert(
        "OAK".into(),
        TerrainObjectType {
            strength: Some(7),
            ..Default::default()
        },
    );
    s.place_terrain("TREE", cell(0, 0));
    s.place_terrain("OAK", cell(2, 0));
    let w = s.build();
    let health: Vec<i32> = w
        .objects
        .values()
        .filter(|o| o.kind == Kind::Terrain)
        .map(|o| o.health)
        .collect();
    assert_eq!(health, [-1, 7]);
}

/// The starting scene with only the infantry pool (MAN), no base units,
/// Alpha on position 0 at cell (40, 40) and Beta on position 1.
fn infantry_start_scene() -> Scene {
    let mut s = starting_scene();
    s.ruleset.sim.vehicles.get_mut("TANK").unwrap().owner = Some(Vec::new());
    s.setup.bases = false;
    s.setup.houses[0].start = Some(0);
    s.setup.houses[1].start = Some(1);
    s
}

/// The cells on which house `h`'s starting units stand.
fn unit_cells(w: &World, h: u8) -> Vec<glasnost_map::CellXY> {
    w.objects
        .values()
        .filter(|o| o.owner == Some(glasnost_sim::world::HouseId(h)) && o.kind != Kind::Building)
        .map(|o| o.pos.cell())
        .collect()
}

/// spec: S-0009 claim 46 and Formulas (the nearest ground occupant is measured to the cell corner
/// with the table root of a² + b², not to the centre; on a tie the occupant that came into the cell
/// last wins among infantry, vehicles and aircraft) (scenario M26 variant J)
#[test]
fn claim_46_nearest_ground_occupant_from_the_corner() {
    let base = glasnost_map::CellXY::new(40, 40);
    let run = |sub: i32| {
        let mut s = infantry_start_scene();
        // Beta's vehicle at the base cell's centre, then Beta's infantry unit at position `sub`.
        s.place_unit(1, "TANK", base, 0);
        s.place_infantry(1, "MAN", base, sub);
        let w = load(&s).expect("loads");
        let cells = unit_cells(&w, 0);
        assert!(!cells.is_empty());
        cells.iter().filter(|&&c| c == base).count() == cells.len()
    };
    // J1: the top position (64, 64) is nearer the corner than the centre: infantry is nearest.
    assert!(run(1), "J1");
    // J2: the bottom position (192, 192) is farther: the vehicle is nearest, none on the base cell.
    let mut s = infantry_start_scene();
    s.place_unit(1, "TANK", base, 0);
    s.place_infantry(1, "MAN", base, 4);
    let w = load(&s).expect("loads");
    assert!(!unit_cells(&w, 0).contains(&base), "J2");
    // J3: both at the centre, a tie: the infantry unit came in last and wins.
    assert!(run(0), "J3");
}

/// spec: S-0009 claim 46 and Terms (objects on a bridge deck are not ground occupants: a cell whose
/// deck holds a vehicle accepts a starting unit, which stands on the ground) (scenario M26 variant K)
#[test]
fn claim_46_deck_objects_do_not_count() {
    let base = glasnost_map::CellXY::new(40, 40);
    let run = |flag: bool| {
        let mut s = starting_scene();
        s.setup.bases = false;
        s.setup.houses[0].start = Some(0);
        s.setup.houses[1].start = Some(1);
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
        // A high-bridge overlay (index 24, S-0211 claim 15) on the base cell.
        s.cell_mut(base).overlay = 24;
        s.place_unit(1, "TANK", base, 0);
        s.map.objects.unit_fields.push(EntryFields {
            bridge: Some(flag),
            ..Default::default()
        });
        load(&s).expect("loads")
    };
    let w = run(true);
    let first = w
        .objects
        .values()
        .find(|o| o.owner == Some(glasnost_sim::world::HouseId(0)))
        .expect("a starting unit");
    assert_eq!(first.pos.cell(), base, "K1");
    assert_eq!(first.layer, glasnost_sim::world::Layer::Ground);
    let w = run(false);
    let first = w
        .objects
        .values()
        .find(|o| o.owner == Some(glasnost_sim::world::HouseId(0)) && o.kind == Kind::Vehicle)
        .expect("a starting vehicle");
    assert_ne!(first.pos.cell(), base, "K2");
}

/// spec: S-0009 Edge cases (quirks.starting_infantry_share_centre off: testing a cell takes no draw;
/// one draw R(0, 3) on the accepting cell gives the standing position of S-0208 claim 3's row)
#[test]
fn claim_46_switch_off_standing_position() {
    use glasnost_sim::rng::Rng;
    let mut s = infantry_start_scene();
    s.setup.houses.truncate(1);
    s.setup.unit_count = 1;
    s.ruleset.globals.quirks.starting_infantry_share_centre = false;
    let w = load(&s).expect("loads");
    let men: Vec<_> = w
        .objects
        .values()
        .filter(|o| o.kind == Kind::Infantry)
        .collect();
    // n = 1 (MAN), C = 50: B = trunc(50 / 1) × 1 = 50: one MAN.
    assert_eq!(men.len(), 1);
    // Replay the draws: the two house-creation rounds of the one house around the early lists
    // (position 0 defined, no random entry), then the starting unit:
    // its type R(0, 0) (no draw), its creation draw, the base cell accepts with no search draw, R(0, 3).
    let mut r = Rng::new(s.setup.seed);
    for _ in 0..2 {
        // spec: S-0202 claim 14 (house creation: one R(450, 1800) per house in each of the two rounds)
        let _ = r.ranged(450, 1800, true);
    }
    let _ = r.raw();
    let _ = r.ranged(0, 3, true);
    let _ = r.ranged(0, 65535, true);
    assert_eq!(w.rng, r);
    // The position is a side position of S-0208 table O, never the centre.
    let at = (men[0].pos.x.rem_euclid(256), men[0].pos.y.rem_euclid(256));
    let sides: Vec<_> = glasnost_sim::cells::position::SIDES
        .iter()
        .map(|&p| glasnost_sim::walk::standing_point(p))
        .collect();
    assert!(sides.contains(&at), "{at:?}");
}

/// A skirmish with Alpha (a human player) and the neutral and special
/// houses, both countries not passive, each owning a pre-placed vehicle.
fn cleanup_scene() -> Scene {
    let mut s = starting_scene();
    s.setup.bases = false;
    s.setup.unit_count = 0;
    s.setup.houses.truncate(1);
    for (name, role) in [
        ("Neutral", glasnost_sim::world::HouseRole::Neutral),
        ("Special", glasnost_sim::world::HouseRole::Special),
    ] {
        s.register("countries", name);
        s.ruleset.sim.countries.insert(
            name.into(),
            glasnost_rules::sim::Country {
                multiplay_passive: Some(false),
                ..Default::default()
            },
        );
        s.setup.houses.push(glasnost_sim::world::HouseSetup {
            name: name.into(),
            country: name.into(),
            human: false,
            start: None,
            role,
            ai: false,
        });
    }
    s.map.objects.units.push(Unit {
        placed: placed("Neutral", "TANK", cell(-6, 0), 256, 0),
    });
    s.map.objects.units.push(Unit {
        placed: placed("Special", "TANK", cell(-6, 4), 256, 0),
    });
    s
}

fn owned(w: &World, h: u8) -> usize {
    w.objects
        .values()
        .filter(|o| o.owner == Some(glasnost_sim::world::HouseId(h)))
        .count()
}

/// spec: S-0009 claim 28 (no computer player and fewer human slots than the maximum player count:
/// eligible houses are removed with their objects, the house after a removed one kept,
/// quirks.cleanup_skips_next_house; an AI house is a computer player, S-0412 claim 12)
#[test]
fn claim_28_unused_house_cleanup() {
    let s = cleanup_scene();
    let w = load(&s).expect("loads");
    assert!(w.houses[1].removed && !w.houses[2].removed);
    assert_eq!((owned(&w, 1), owned(&w, 2)), (0, 1));
    assert_eq!(w.houses[1].start, None);
    // Switch off: every eligible house is removed.
    let mut off = cleanup_scene();
    off.ruleset.globals.quirks.cleanup_skips_next_house = false;
    let w = load(&off).expect("loads");
    assert!(w.houses[1].removed && w.houses[2].removed);
    assert_eq!((owned(&w, 1), owned(&w, 2)), (0, 0));
    // An AI house: a computer player, no cleanup.
    let mut ai = cleanup_scene();
    ai.setup.houses[0].ai = true;
    let w = load(&ai).expect("loads");
    assert!(w.houses.iter().all(|h| !h.removed));
    // As many human slots as the maximum player count: no cleanup.
    let mut full = cleanup_scene();
    full.ruleset.sim.general.max_players = Some(1);
    let w = load(&full).expect("loads");
    assert!(w.houses.iter().all(|h| !h.removed));
    // A passive neutral country: not eligible; the special house is then removed.
    let mut passive = cleanup_scene();
    passive
        .ruleset
        .sim
        .countries
        .get_mut("Neutral")
        .unwrap()
        .multiplay_passive = Some(true);
    let w = load(&passive).expect("loads");
    assert!(!w.houses[1].removed && w.houses[2].removed);
}

/// spec: S-0009 claim 28 and S-0202 claim 14 (the cleanup runs after the second pass; removing a
/// ground occupant recalculates its cell, and a cell whose tile animation started in the second
/// pass takes no new creation draw)
#[test]
fn claim_28_cleanup_recalculation_after_the_second_pass() {
    use glasnost_map::terrain::TileAnim;
    use glasnost_rules::sim::AnimationType;
    let with_anim = |removed_on_anim: bool| {
        let mut s = cleanup_scene();
        s.register("animations", "SPARK");
        // A random delay: one ranged draw per creation.
        s.ruleset.sim.animations.insert(
            "SPARK".into(),
            AnimationType {
                random_rate: Some([2, 1]),
                ..Default::default()
            },
        );
        s.catalogue.tiles[1].anim = Some(TileAnim {
            name: "SPARK".into(),
            attaches_to: 0,
            ..TileAnim::default()
        });
        s.set_cell(cell(-6, 0), 1, 0);
        if !removed_on_anim {
            // The removed house's vehicle elsewhere, the kept house's on the animation's cell.
            s.map.objects.units[0].placed.cell = cell(-6, 4);
            s.map.objects.units[1].placed.cell = cell(-6, 0);
        }
        load(&s).expect("loads")
    };
    let on = with_anim(true);
    let off = with_anim(false);
    assert!(on.houses[1].removed && off.houses[1].removed);
    let i = on.cells.idx(cell(-6, 0)).unwrap();
    assert!(on.cells.anim_started[i], "started in the second pass");
    // The removal's recalculation of the started cell took no draw.
    assert_eq!(on.rng, off.rng);
}

/// spec: S-0010 claim 12 and claim 19 (the game-start pass after the starting units: a cell that has
/// started no tile animation since the second pass starts that of the tile the second pass left, with
/// its creation draws; at most one per cell)
#[test]
fn s0010_claim_12_game_start_pass() {
    use glasnost_map::terrain::TileAnim;
    use glasnost_rules::sim::AnimationType;
    use glasnost_sim::rng::Rng;
    let mut s = scene();
    s.register("animations", "SPARK");
    s.ruleset.sim.animations.insert(
        "SPARK".into(),
        AnimationType {
            random_rate: Some([2, 1]),
            ..Default::default()
        },
    );
    s.catalogue.tiles[1].anim = Some(TileAnim {
        name: "SPARK".into(),
        attaches_to: 0,
        ..TileAnim::default()
    });
    s.set_cell(cell(0, 0), 1, 0);
    let w = s.build();
    let mut r = Rng::new(s.setup.seed);
    let _ = r.ranged(450, 900, true);
    let _ = r.ranged(450, 900, true);
    let _ = r.ranged(0, 65535, true);
    assert_eq!(w.rng, r, "the game-start pass starts no second animation");
    // A cell the second pass did not start: mark it unstarted and run the pass again as game start does.
    let mut w = w;
    let i = w.cells.idx(cell(0, 0)).unwrap();
    w.cells.anim_started[i] = false;
    let order = w.cells.grid.visiting_order(true);
    glasnost_sim::load::recalc_cells(&mut w, &order);
    let _ = r.ranged(450, 900, true);
    assert_eq!(w.rng, r);
}

/// spec: S-0010 claim 25 (at game start the bands of low bridge spans carry the bridge-body mark;
/// the end cells and the cells beyond the band stay clear; the mark is game state)
#[test]
fn s0010_claim_25_bridge_body_marks_at_game_start() {
    let mut s = scene();
    // The wooden-bridge role starts at tile 5, so tile 11 is its i = 6 (start sub-tile 4, east; end
    // sub-tile 4) and the flat tile 0 between the ends lies outside it; tile 11 is made five cells wide.
    s.catalogue.roles.wooden_bridge = 5;
    let mut wide = s.catalogue.tiles[11].clone();
    wide.width = 5;
    wide.height = 1;
    wide.cells = vec![wide.cells[0]; 5];
    s.catalogue.tiles[11] = wide;
    for x in [0, 10] {
        let c = s.cell_mut(cell(x, 0));
        c.tile = 11;
        c.sub_tile = 4;
    }
    let w = s.build();
    let marked = |dx: i32, dy: i32| {
        let i = w.cells.idx(cell(dx, dy)).unwrap();
        w.cells.bridge_body[i]
    };
    for dy in [-2, -1, 0, 1] {
        assert!(marked(5, dy), "row {dy}");
    }
    assert!(!marked(5, -3) && !marked(5, 2));
    assert!(!marked(0, 0) && !marked(10, 0));
    let l = glasnost_sim::save::load(
        &glasnost_sim::save::save(&w),
        s.sim_rules(),
        std::sync::Arc::new(s.catalogue.clone()),
    )
    .expect("loads back");
    assert_eq!(
        glasnost_sim::save::state_hash(&w),
        glasnost_sim::save::state_hash(&l)
    );
}
