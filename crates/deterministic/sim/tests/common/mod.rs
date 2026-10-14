//! Shared builders for the claim tests: a small synthetic world with
//! invented rules, a flat map and a tile catalogue (no game data).
//!
//! Each `tests/claims_*.rs` file includes this module with `mod common;`.
//! Not every test file uses every helper.

#![allow(dead_code)]

use std::collections::BTreeMap;
use std::sync::Arc;

use glasnost_map::objects::{Placed, Structure, TerrainObject};
use glasnost_map::terrain::{Roles, TileCatalogue, TileCell, TileInfo};
use glasnost_map::{
    Basic, Cell, CellGrid, CellXY, DEFAULT_HOME_WAYPOINT, Fill, Footprint, Header, Infantry,
    Lighting, Map, Objects, Theater, Unit,
};
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{Country, LandRow, LandType, LocomotionClass, Locomotor, UnitType};
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind, SimRules};
use glasnost_sim::tick;
use glasnost_sim::world::{GameSetup, HouseId, HouseRole, HouseSetup, ObjectId, Target, World};

/// Map size of a scene (width and height of the footprint).
pub const SIZE: i32 = 40;

/// Tile number of a flat 1×1 tile whose cell has terrain code `code`
/// (0 … 15). Tiles 0 … 4 are the clear tiles of the testkit (0 flat, 1 … 4
/// ramps with slope shapes 1 … 4).
pub fn code_tile(code: u8) -> i32 {
    5 + i32::from(code)
}

/// Tile number of a 1×1 tile with terrain code 0 and slope shape `slope`
/// (0 … 20).
pub fn slope_tile(slope: u8) -> i32 {
    21 + i32::from(slope)
}

/// The middle cell of the map, offset by (dx, dy).
pub fn cell(dx: i32, dy: i32) -> CellXY {
    CellXY::new(SIZE + dx, SIZE + dy)
}

/// A unit type with the given locomotion class, speed and turn rate.
pub fn unit_type(class: LocomotionClass, speed: i32, rot: i32) -> UnitType {
    UnitType {
        strength: Some(100),
        speed: Some(speed),
        rot: Some(rot),
        locomotion_class: Some(class),
        ..Default::default()
    }
}

/// A scene under construction. Every field is public: change the ruleset,
/// map cells, catalogue or setup directly before [`Scene::build`].
pub struct Scene {
    /// The rules; registries "countries", "vehicles" and "infantry" are kept
    /// in step with the typed sections by the `vehicle` and `infantry`
    /// helpers.
    pub ruleset: Ruleset,
    /// The map: every cell has `stream: true`.
    pub map: Map,
    /// The tile catalogue (see [`code_tile`] and [`slope_tile`]).
    pub catalogue: TileCatalogue,
    /// Two human houses, "Alpha" (0) and "Beta" (1), not allied; sandbox.
    pub setup: GameSetup,
    next_key: usize,
}

impl Default for Scene {
    fn default() -> Self {
        Self::new()
    }
}

impl Scene {
    /// A flat clear 40×40 map with two countries, land rows for clear and
    /// road (all multipliers 1.0), and no object types.
    pub fn new() -> Self {
        let mut r = Ruleset::default();
        r.registries.insert(
            "countries".into(),
            vec!["Alphaland".to_string(), "Betaland".to_string()],
        );
        r.registries.insert("vehicles".into(), Vec::new());
        r.registries.insert("infantry".into(), Vec::new());
        r.sim
            .countries
            .insert("Alphaland".into(), Country::default());
        r.sim
            .countries
            .insert("Betaland".into(), Country::default());
        r.sim.land_types.insert(LandType::Clear, LandRow::default());
        r.sim.land_types.insert(LandType::Road, LandRow::default());

        let fp = Footprint::new(SIZE, SIZE, [0, 0, SIZE, SIZE]);
        let mut cells = CellGrid::new(fp);
        let all: Vec<CellXY> = cells.iter().map(|(c, _)| c).collect();
        for c in all {
            if let Some(cell) = cells.get_mut(c) {
                *cell = Cell {
                    tile: 0,
                    stream: true,
                    ..Cell::BLANK
                };
            }
        }
        let map = Map {
            format_version: 4,
            theater: Theater::Temperate,
            base_level: 0,
            fill: Fill::Clear,
            footprint: fp,
            cells,
            has_overlay_data: true,
            waypoints: BTreeMap::new(),
            home_waypoint: DEFAULT_HOME_WAYPOINT,
            header: Header::default(),
            preview: None,
            lighting: Lighting::default(),
            basic: Basic::default(),
            objects: Objects::default(),
            cell_tags: BTreeMap::new(),
            fill_draws: 0,
            triggers: Default::default(),
        };

        let tile = |code: u8, slope: u8| TileInfo {
            width: 1,
            height: 1,
            has_image: true,
            cells: vec![Some(TileCell {
                terrain_code: code,
                slope,
            })],
            variants: 0,
            anim: None,
            no_burrowing: false,
        };
        let mut tiles: Vec<TileInfo> = (0..=4).map(|s| tile(0, s)).collect();
        tiles.extend((0..16).map(|k| tile(k, 0)));
        tiles.extend((0..=20).map(|s| tile(0, s)));
        let catalogue = TileCatalogue {
            tiles,
            roles: Roles::default(),
            allow_tiberium: Vec::new(),
        };

        let setup = GameSetup {
            seed: 1,
            game_speed: 4,
            houses: vec![
                HouseSetup {
                    name: "Alpha".into(),
                    country: "Alphaland".into(),
                    human: true,
                    start: None,
                    role: HouseRole::Player,
                    ai: false,
                },
                HouseSetup {
                    name: "Beta".into(),
                    country: "Betaland".into(),
                    human: true,
                    start: None,
                    role: HouseRole::Player,
                    ai: false,
                },
            ],
            alliances: Vec::new(),
            sandbox: true,
            unit_count: 0,
            bases: false,
            skirmish: Default::default(),
        };
        Scene {
            ruleset: r,
            map,
            catalogue,
            setup,
            next_key: 0,
        }
    }

    /// Registers a vehicle type; one without a locomotor field gets the
    /// driving locomotor (S-0207 claim 14 would otherwise give teleporting).
    pub fn vehicle(&mut self, name: &str, mut t: UnitType) -> &mut Self {
        t.locomotor.get_or_insert(Locomotor::Drive);
        self.register("vehicles", name);
        self.ruleset.sim.vehicles.insert(name.into(), t);
        self
    }

    /// Registers an infantry type; one without a locomotor field gets the
    /// walking locomotor.
    pub fn infantry(&mut self, name: &str, mut t: UnitType) -> &mut Self {
        t.locomotor.get_or_insert(Locomotor::Walk);
        self.register("infantry", name);
        self.ruleset.sim.infantry.insert(name.into(), t);
        self
    }

    /// Registers a type of `registry` exactly as given (no locomotor
    /// default), for tests of the locomotor field itself.
    pub fn raw_unit(&mut self, registry: &str, name: &str, t: UnitType) -> &mut Self {
        self.register(registry, name);
        let map = match registry {
            "vehicles" => &mut self.ruleset.sim.vehicles,
            "infantry" => &mut self.ruleset.sim.infantry,
            _ => &mut self.ruleset.sim.aircraft,
        };
        map.insert(name.into(), t);
        self
    }

    /// Appends `name` to a registry if it is not there yet.
    pub fn register(&mut self, registry: &str, name: &str) {
        let list = self.ruleset.registries.entry(registry.into()).or_default();
        if !list.iter().any(|n| n == name) {
            list.push(name.to_string());
        }
    }

    /// The map cell at `c` (panics outside the map).
    pub fn cell_mut(&mut self, c: CellXY) -> &mut Cell {
        self.map.cells.get_mut(c).expect("cell on the map")
    }

    /// Sets the tile and height level of a cell.
    pub fn set_cell(&mut self, c: CellXY, tile: i32, level: i8) -> &mut Self {
        let cell = self.cell_mut(c);
        cell.tile = tile;
        cell.sub_tile = 0;
        cell.level = level;
        cell.stream = true;
        self
    }

    fn placed(&mut self, house: u8, ty: &str, c: CellXY, facing: i32) -> Placed {
        self.next_key += 1;
        let owner = if house == 0 { "Alpha" } else { "Beta" };
        Placed {
            key: self.next_key.to_string().into_bytes(),
            owner: owner.as_bytes().to_vec(),
            type_name: ty.as_bytes().to_vec(),
            health: 256,
            cell: c,
            facing,
            raw: Vec::new(),
        }
    }

    /// Places a vehicle of house 0 ("Alpha") or 1 ("Beta").
    pub fn place_unit(&mut self, house: u8, ty: &str, c: CellXY, facing: i32) -> &mut Self {
        let placed = self.placed(house, ty, c, facing);
        self.map.objects.units.push(Unit { placed });
        self
    }

    /// Places an infantry unit in sub-cell `sub` (1 … 4).
    pub fn place_infantry(&mut self, house: u8, ty: &str, c: CellXY, sub: i32) -> &mut Self {
        let placed = self.placed(house, ty, c, 0);
        self.map.objects.infantry.push(Infantry {
            placed,
            sub_cell: sub,
        });
        self
    }

    /// Places a structure (its type must be registered by the caller).
    pub fn place_structure(&mut self, house: u8, ty: &str, c: CellXY) -> &mut Self {
        let placed = self.placed(house, ty, c, 0);
        self.map.objects.structures.push(Structure { placed });
        self
    }

    /// Places a terrain object (its type must be registered by the caller).
    pub fn place_terrain(&mut self, ty: &str, c: CellXY) -> &mut Self {
        self.map.objects.terrain.push(TerrainObject {
            cell: c,
            type_name: ty.as_bytes().to_vec(),
        });
        self
    }

    /// The rules this scene would load with.
    pub fn sim_rules(&self) -> Arc<SimRules> {
        Arc::new(SimRules::new(&self.ruleset, "claims"))
    }

    /// Loads the scene into a new game.
    pub fn build(&self) -> World {
        glasnost_sim::load::new_game(
            self.sim_rules(),
            &self.map,
            Arc::new(self.catalogue.clone()),
            &self.setup,
        )
        .expect("synthetic scene loads")
    }
}

/// Runs ticks until orders are accepted (S-0201 claim 14: ticks 0 to 3 are
/// blocked).
pub fn open_orders(w: &mut World) {
    while w.tick < 4 {
        tick::step(w);
    }
}

/// Runs `n` ticks.
pub fn run(w: &mut World, n: i32) {
    for _ in 0..n {
        tick::step(w);
    }
}

/// Runs ticks until `done` holds or `max` ticks have passed; returns the
/// number of ticks run.
pub fn run_until(w: &mut World, max: i32, mut done: impl FnMut(&World) -> bool) -> i32 {
    for i in 0..max {
        if done(w) {
            return i;
        }
        tick::step(w);
    }
    max
}

/// A move order to a cell.
pub fn move_to(actor: ObjectId, c: CellXY) -> Order {
    Order::Behaviour {
        actor,
        behaviour: Behaviour::Move,
        attack_move: false,
        target: None,
        dest: Some(Target::Cell(c)),
    }
}

/// Issues a move order for the actor's owner; panics if refused.
pub fn order_move(w: &mut World, actor: ObjectId, c: CellXY) {
    let owner = w.obj(actor).and_then(|o| o.owner).unwrap_or(HouseId(0));
    assert!(
        orders::issue(w, owner, move_to(actor, c)),
        "move order refused"
    );
}

/// The objects of a kind, in ID order.
pub fn objects_of(w: &World, kind: Kind) -> Vec<ObjectId> {
    w.objects
        .values()
        .filter(|o| o.kind == kind)
        .map(|o| o.id)
        .collect()
}

/// The first object standing in cell `c`, if any.
pub fn object_in(w: &World, c: CellXY) -> Option<ObjectId> {
    w.objects.values().find(|o| o.pos.cell() == c).map(|o| o.id)
}

/// Vehicle class shorthand.
pub const WHEEL: LocomotionClass = LocomotionClass::Wheel;
/// Track class shorthand.
pub const TRACK: LocomotionClass = LocomotionClass::Track;
/// Foot class shorthand.
pub const FOOT: LocomotionClass = LocomotionClass::Foot;

impl Scene {
    /// Registers an aircraft type.
    pub fn aircraft(&mut self, name: &str, t: UnitType) -> &mut Self {
        self.register("aircraft", name);
        self.ruleset.sim.aircraft.insert(name.into(), t);
        self
    }

    /// Registers a building type.
    pub fn building(&mut self, name: &str, t: glasnost_rules::sim::BuildingType) -> &mut Self {
        self.register("buildings", name);
        self.ruleset.sim.buildings.insert(name.into(), t);
        self
    }

    /// Places an aircraft of house 0 ("Alpha") or 1 ("Beta").
    pub fn place_aircraft(&mut self, house: u8, ty: &str, c: CellXY, facing: i32) -> &mut Self {
        let placed = self.placed(house, ty, c, facing);
        self.map
            .objects
            .aircraft
            .push(glasnost_map::objects::Aircraft { placed });
        self
    }
}
