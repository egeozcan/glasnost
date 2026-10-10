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
