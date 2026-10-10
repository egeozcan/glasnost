//! A synthetic scenario for tests, the headless runner and the determinism
//! gate: invented rules (no stock values) on a small flat map, two houses
//! with vehicles and infantry, and a scripted list of orders.

use std::collections::BTreeMap;
use std::sync::Arc;

use glasnost_map::objects::Placed;
use glasnost_map::terrain::{Roles, TileCatalogue, TileCell, TileInfo};
use glasnost_map::{
    Basic, Cell, CellGrid, CellXY, DEFAULT_HOME_WAYPOINT, Fill, Footprint, Header, Infantry,
    Lighting, Map, Objects, Theater, Unit,
};
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{Country, LandRow, LandType, LocomotionClass, Locomotor, UnitType};

use crate::orders::Order;
use crate::rules::{Behaviour, SimRules};
use crate::save::Replay;
use crate::world::{GameSetup, HouseId, HouseSetup, ObjectId, Target, World};

/// Map size of the scenario.
pub const SIZE: i32 = 40;

/// The synthetic rules.
pub fn rules() -> Arc<SimRules> {
    Arc::new(SimRules::new(&ruleset(), "synthetic"))
}

/// The synthetic ruleset that [`rules`] is built from, for tests that add
/// types of their own.
pub fn ruleset() -> Ruleset {
    let mut r = Ruleset::default();
    let reg = |v: &[&str]| v.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    r.registries
        .insert("countries".into(), reg(&["Alphaland", "Betaland"]));
    r.registries
        .insert("vehicles".into(), reg(&["CART", "CRUSHER", "BUGGY"]));
    r.registries.insert("infantry".into(), reg(&["WALKER"]));
    let s = &mut r.sim;
    s.countries.insert("Alphaland".into(), Country::default());
    s.countries.insert("Betaland".into(), Country::default());
    s.land_types.insert(LandType::Clear, LandRow::default());
    s.land_types.insert(LandType::Road, LandRow::default());
    let cart = UnitType {
        strength: Some(300),
        speed: Some(7),
        rot: Some(6),
        locomotion_class: Some(LocomotionClass::Wheel),
        locomotor: Some(Locomotor::Drive),
        primary: Some("Gun".into()),
        ..Default::default()
    };
    let crusher = UnitType {
        strength: Some(500),
        speed: Some(5),
        rot: Some(4),
        locomotion_class: Some(LocomotionClass::Track),
        locomotor: Some(Locomotor::Drive),
        crusher: Some(true),
        ..Default::default()
    };
    let walker = UnitType {
        strength: Some(100),
        speed: Some(4),
        locomotion_class: Some(LocomotionClass::Foot),
        locomotor: Some(Locomotor::Walk),
        crushable: Some(true),
        primary: Some("Rifle".into()),
        ..Default::default()
    };
    let buggy = UnitType {
        strength: Some(120),
        speed: Some(9),
        rot: Some(8),
        locomotion_class: Some(LocomotionClass::Wheel),
        locomotor: Some(Locomotor::Drive),
        crushable: Some(true),
        ..Default::default()
    };
    s.vehicles.insert("BUGGY".into(), buggy);
    s.vehicles.insert("CART".into(), cart);
    s.vehicles.insert("CRUSHER".into(), crusher);
    s.infantry.insert("WALKER".into(), walker);
    r
}

/// A small catalogue: tile 0 is clear and flat, tiles 1 to 4 are clear
/// ramps with slope shapes 1 to 4.
pub fn catalogue() -> Arc<TileCatalogue> {
    let tile = |slope: u8| TileInfo {
        width: 1,
        height: 1,
        has_image: true,
        cells: vec![Some(TileCell {
            terrain_code: 0,
            slope,
        })],
        variants: 0,
    };
    Arc::new(TileCatalogue {
        tiles: (0..=4).map(tile).collect(),
        roles: Roles::default(),
    })
}

fn placed(owner: &str, ty: &str, c: CellXY, facing: i32, key: usize) -> Placed {
    Placed {
        key: key.to_string().into_bytes(),
        owner: owner.as_bytes().to_vec(),
        type_name: ty.as_bytes().to_vec(),
        health: 256,
        cell: c,
        facing,
        raw: Vec::new(),
    }
}

/// The middle cell of the map (s = 2W, d = 0), offset by (dx, dy).
pub fn cell(dx: i32, dy: i32) -> CellXY {
    CellXY::new(SIZE + dx, SIZE + dy)
}

/// The synthetic map: clear ground with a raised plateau and ramps across
/// the middle; house Alpha's units on the left, house Beta's on the right.
pub fn map() -> Map {
    let fp = Footprint::new(SIZE, SIZE, [0, 0, SIZE, SIZE]);
    let mut cells = CellGrid::new(fp);
    let all: Vec<CellXY> = cells.iter().map(|(c, _)| c).collect();
    for c in all {
        if let Some(cell) = cells.get_mut(c) {
            // Every cell comes from the terrain stream, so the initial fill keeps it.
            *cell = Cell {
                tile: 0,
                stream: true,
                ..Cell::BLANK
            };
        }
    }
    // A plateau one level up, with a ramp tile on each side.
    for y in -3..=3 {
        for (dx, tile, level) in [(-2, 1, 0), (-1, 0, 1), (0, 0, 1), (1, 0, 1), (2, 3, 0)] {
            if let Some(cell) = cells.get_mut(cell(dx, y - 6)) {
                *cell = Cell {
                    tile,
                    level,
                    stream: true,
                    ..Cell::BLANK
                };
            }
        }
    }
    let mut objects = Objects::default();
    let mut k = 0;
    let mut unit = |owner: &str, ty: &str, c: CellXY, f: i32, objects: &mut Objects| {
        objects.units.push(Unit {
            placed: placed(owner, ty, c, f, k),
        });
        k += 1;
    };
    for i in 0..5 {
        unit("Alpha", "CART", cell(-6, i * 2 - 4), 64, &mut objects);
        unit("Beta", "CART", cell(6, i * 2 - 4), 192, &mut objects);
    }
    unit("Alpha", "CRUSHER", cell(-8, 0), 64, &mut objects);
    unit("Beta", "BUGGY", cell(-2, 8), 128, &mut objects);
    for i in 0..6 {
        objects.infantry.push(Infantry {
            placed: placed("Beta", "WALKER", cell(2, i - 3), 0, 100 + i as usize),
            sub_cell: 1 + (i % 3),
        });
    }
    for i in 0..4 {
        objects.infantry.push(Infantry {
            placed: placed("Alpha", "WALKER", cell(-4, i - 2), 128, 200 + i as usize),
            sub_cell: 2 + (i % 2),
        });
    }
    Map {
        format_version: 4,
        theater: Theater::Temperate,
        base_level: 0,
        fill: Fill::Clear,
        footprint: fp,
        cells,
        has_overlay_data: false,
        waypoints: BTreeMap::new(),
        home_waypoint: DEFAULT_HOME_WAYPOINT,
        header: Header::default(),
        preview: None,
        lighting: Lighting::default(),
        basic: Basic::default(),
        objects,
        cell_tags: BTreeMap::new(),
        fill_draws: 0,
    }
}

/// The setup: two houses, sandbox (no start positions), seed given.
pub fn setup(seed: u32) -> GameSetup {
    GameSetup {
        seed,
        game_speed: 4,
        houses: vec![
            HouseSetup {
                name: "Alpha".into(),
                country: "Alphaland".into(),
                human: true,
                start: None,
            },
            HouseSetup {
                name: "Beta".into(),
                country: "Betaland".into(),
                human: true,
                start: None,
            },
        ],
        alliances: Vec::new(),
        sandbox: true,
    }
}

/// A new game of the scenario.
pub fn new_game(seed: u32) -> World {
    crate::load::new_game(rules(), &map(), catalogue(), &setup(seed))
        .expect("synthetic scenario loads")
}

/// The objects of a house, by ID.
pub fn house_objects(w: &World, h: u8) -> Vec<ObjectId> {
    w.objects
        .values()
        .filter(|o| o.owner == Some(HouseId(h)))
        .map(|o| o.id)
        .collect()
}

fn mv(actor: ObjectId, c: CellXY) -> Order {
    Order::Behaviour {
        actor,
        behaviour: Behaviour::Move,
        attack_move: false,
        target: None,
        dest: Some(Target::Cell(c)),
    }
}

/// The scripted replay of the determinism gate: group moves, crossing
/// paths, a stop, scatters and area guards over `ticks` ticks.
pub fn scripted_replay(seed: u32, ticks: i32) -> Replay {
    let w = new_game(seed);
    let mut rep = Replay::new(w.rules(), "synthetic", setup(seed));
    let a = house_objects(&w, 0);
    // Beta's buggy stays where it is, for the crusher.
    let b: Vec<ObjectId> = house_objects(&w, 1)
        .into_iter()
        .filter(|&id| {
            w.obj(id)
                .is_some_and(|o| !(o.kind == crate::rules::Kind::Vehicle && o.type_index == 2))
        })
        .collect();
    let mut push = |t: i32, h: u8, o: Order| rep.record(t, HouseId(h), o);
    // Alpha moves as a group to the right, Beta to the left: paths cross.
    for &id in &a {
        push(10, 0, mv(id, cell(8, 0)));
    }
    for &id in &b {
        push(12, 1, mv(id, cell(-8, 1)));
    }
    // The crusher drives onto Beta's crushable buggy.
    if let Some(&id) = a.iter().find(|&&id| {
        w.obj(id)
            .is_some_and(|o| o.type_index == 1 && o.kind == crate::rules::Kind::Vehicle)
    }) {
        push(150, 0, mv(id, cell(-2, 8)));
    }
    // Two units climb the plateau: a vehicle and an infantry unit.
    if let Some(&id) = a.get(1) {
        push(400, 0, mv(id, cell(0, -6)));
    }
    if let Some(&id) = a.iter().rev().find(|&&id| {
        w.obj(id)
            .is_some_and(|o| o.kind == crate::rules::Kind::Infantry)
    }) {
        push(410, 0, mv(id, cell(1, -5)));
    }
    // A stop, then a new move, for one Alpha unit.
    if let Some(&id) = a.first() {
        push(60, 0, Order::Stop { actor: id });
        push(61, 0, mv(id, cell(0, 8)));
    }
    // Beta's infantry scatter, then guard an area.
    for &id in b.iter().filter(|&&id| {
        w.obj(id)
            .is_some_and(|o| o.kind == crate::rules::Kind::Infantry)
    }) {
        push(90, 1, Order::Scatter { actor: id });
        push(
            300,
            1,
            Order::Behaviour {
                actor: id,
                behaviour: Behaviour::AreaGuard,
                attack_move: false,
                target: Some(Target::Cell(cell(0, -6))),
                dest: None,
            },
        );
    }
    // Everyone returns home late in the run.
    for (i, &id) in a.iter().enumerate() {
        push(800, 0, mv(id, cell(-6, i as i32 - 2)));
    }
    for (i, &id) in b.iter().enumerate() {
        push(820, 1, mv(id, cell(6, i as i32 - 3)));
    }
    rep.ticks = ticks;
    rep
}
