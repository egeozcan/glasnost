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
use crate::world::{GameSetup, HouseId, HouseRole, HouseSetup, ObjectId, Target, World};

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
    // The scenario's data gives an area-guard threshold of 0, so that its armed vehicles rest in
    // area guard (S-0215 claim 15 row 12); the engine default of claim 36 (a) would give guard.
    s.general.iq_guard_area = Some(0);
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
        anim: None,
        no_burrowing: false,
    };
    Arc::new(TileCatalogue {
        tiles: (0..=4).map(tile).collect(),
        roles: Roles::default(),
        // Every tile's tileset allows new resource (S-0010 claim 16).
        allow_tiberium: vec![true; 5],
        bridge_positions: Default::default(),
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
        has_overlay_data: true,
        waypoints: BTreeMap::new(),
        home_waypoint: DEFAULT_HOME_WAYPOINT,
        header: Header::default(),
        preview: None,
        lighting: Lighting::default(),
        basic: Basic::default(),
        objects,
        cell_tags: BTreeMap::new(),
        fill_draws: 0,
        triggers: Default::default(),
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

/// The synthetic combat ruleset: [`ruleset`] with invented weapons,
/// projectiles and warheads for its types (no stock values), and a guard
/// tower building type.
pub fn combat_ruleset() -> Ruleset {
    use glasnost_rules::sim::{
        ArmourClass, BuildingType, Decimal, ProjectileType, WarheadType, WeaponType,
    };
    let mut r = ruleset();
    let d = |s: &str| Decimal(s.into());
    r.registries
        .insert("buildings".into(), vec!["TOWER".into()]);
    let s = &mut r.sim;
    s.weapons.insert(
        "Gun".into(),
        WeaponType {
            damage: Some(30),
            rof: Some(40),
            range: Some(d("5.5")),
            speed: Some(40),
            projectile: Some("Shell".into()),
            warhead: Some("HE".into()),
            ..Default::default()
        },
    );
    s.weapons.insert(
        "Rifle".into(),
        WeaponType {
            damage: Some(12),
            rof: Some(25),
            range: Some(d("4")),
            speed: Some(100),
            projectile: Some("Bullet".into()),
            warhead: Some("SA".into()),
            ..Default::default()
        },
    );
    s.weapons.insert(
        "Cannon".into(),
        WeaponType {
            damage: Some(55),
            rof: Some(60),
            range: Some(d("7.25")),
            speed: Some(30),
            projectile: Some("Rocket".into()),
            warhead: Some("HE".into()),
            ..Default::default()
        },
    );
    s.projectiles.insert(
        "Shell".into(),
        ProjectileType {
            arcing: Some(true),
            ..Default::default()
        },
    );
    s.projectiles.insert(
        "Rocket".into(),
        ProjectileType {
            rot: Some(12),
            acceleration: Some(4),
            ..Default::default()
        },
    );
    s.projectiles.insert(
        "Bullet".into(),
        ProjectileType {
            inviso: Some(true),
            ..Default::default()
        },
    );
    s.projectiles.insert(
        "Lob".into(),
        ProjectileType {
            arcing: Some(true),
            ..Default::default()
        },
    );
    let verses = |v: [&str; 11]| Some(v.iter().map(|x| Decimal((*x).into())).collect());
    s.warheads.insert(
        "HE".into(),
        WarheadType {
            cell_spread: Some(d("0.75")),
            percent_at_max: Some(d("0.5")),
            verses: verses([
                "0.9", "0.8", "0.7", "0.6", "0.5", "0.45", "0.6", "0.4", "0.35", "1", "1",
            ]),
            wall: Some(true),
            ..Default::default()
        },
    );
    s.warheads.insert(
        "SA".into(),
        WarheadType {
            verses: verses([
                "1", "0.7", "0.5", "0.3", "0.2", "0.1", "0.4", "0.05", "0.05", "1", "1",
            ]),
            ..Default::default()
        },
    );
    for (name, class) in [
        ("CART", ArmourClass::Light),
        ("CRUSHER", ArmourClass::Heavy),
        ("BUGGY", ArmourClass::Light),
    ] {
        if let Some(v) = s.vehicles.get_mut(name) {
            v.combat.armor = Some(class);
            v.turret = Some(true);
            v.combat.primary_fire_flh = Some([90, 0, 70]);
        }
    }
    if let Some(v) = s.vehicles.get_mut("CRUSHER") {
        v.primary = Some("Gun".into());
    }
    if let Some(i) = s.infantry.get_mut("WALKER") {
        i.combat.primary_fire_flh = Some([30, 0, 50]);
    }
    s.buildings.insert(
        "TOWER".into(),
        BuildingType {
            strength: Some(800),
            foundation: Some([1, 1]),
            primary: Some("Cannon".into()),
            turret: Some(true),
            rot: Some(10),
            combat: glasnost_rules::sim::CombatFields {
                armor: Some(ArmourClass::Concrete),
                primary_fire_flh: Some([60, 0, 220]),
                ..Default::default()
            },
            ..Default::default()
        },
    );
    r
}

/// The synthetic combat rules.
pub fn combat_rules() -> Arc<SimRules> {
    Arc::new(SimRules::new(&combat_ruleset(), "synthetic-combat"))
}

/// A new game of the scenario with the combat rules.
pub fn combat_game(seed: u32) -> World {
    crate::load::new_game(combat_rules(), &map(), catalogue(), &setup(seed))
        .expect("synthetic combat scenario loads")
}

/// A flat map of `size` × `size` clear cells at level 0 with no objects,
/// for tests that place their own objects.
pub fn empty_map(size: i32) -> Map {
    let fp = Footprint::new(size, size, [0, 0, size, size]);
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
    Map {
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
    }
}

/// A new sandbox game of houses Alpha and Beta (not allied) on an empty
/// map of `size`, with the given ruleset.
pub fn game_with(ruleset: &Ruleset, size: i32, seed: u32) -> World {
    let rules = Arc::new(SimRules::new(ruleset, "synthetic-test"));
    crate::load::new_game(rules, &empty_map(size), catalogue(), &setup(seed))
        .expect("synthetic test game loads")
}

/// Triggers for object-tag notification tests (S-0524 claims 10 to 14),
/// every tag repeating: `ENT` fires on every entered-by raise (any house)
/// and sets variable 0, `DIE` fires on a destroyed-by-anything raise and
/// sets variable 1, and `DIE_CARRY` is like `DIE` with the carry-over flag
/// and sets variable 2. Tag them with [`crate::triggers::object_tag`].
pub fn entry_triggers() -> glasnost_map::triggers::Triggers {
    use glasnost_map::triggers::{Action, Event, Owner, TagDef, TriggerDef, Triggers};
    let def = |id: &str, code: i32, value: i32, var: i32, carry: bool| TriggerDef {
        owner: Owner::None,
        events: vec![Event {
            code,
            kind: 0,
            value,
            type_name: None,
        }],
        actions: vec![Action {
            code: 56,
            number: var,
            ..Default::default()
        }],
        carry_over: carry,
        ..TriggerDef::created(id.as_bytes().to_vec())
    };
    let defs = vec![
        def("ENT", 1, -1, 0, false),
        def("DIE", 48, 0, 1, false),
        def("DIE_CARRY", 48, 0, 2, true),
    ];
    let tags = defs
        .iter()
        .enumerate()
        .map(|(i, t)| TagDef {
            id: t.id.clone(),
            repeat: 2,
            name: t.id.clone(),
            first: Some(i as u32),
            chain: vec![i as u32],
        })
        .collect();
    Triggers {
        triggers: defs,
        tags,
        ..Default::default()
    }
}

/// A new sandbox game like [`game_with`], on a map holding `triggers`.
pub fn game_with_triggers(
    ruleset: &Ruleset,
    size: i32,
    seed: u32,
    triggers: glasnost_map::triggers::Triggers,
) -> World {
    let rules = Arc::new(SimRules::new(ruleset, "synthetic-test"));
    let mut m = empty_map(size);
    m.triggers = triggers;
    crate::load::new_game(rules, &m, catalogue(), &setup(seed)).expect("synthetic test game loads")
}

/// A new sandbox game like [`game_with`], with two more houses that are
/// not player houses: house 2 is the neutral house and house 3 the special
/// house (S-0009 claim 19), neither human-controlled, of the countries
/// Alphaland and Betaland.
pub fn game_with_non_players(ruleset: &Ruleset, size: i32, seed: u32) -> World {
    let rules = Arc::new(SimRules::new(ruleset, "synthetic-test"));
    let mut s = setup(seed);
    for (name, country, role) in [
        ("Neutral", "Alphaland", HouseRole::Neutral),
        ("Special", "Betaland", HouseRole::Special),
    ] {
        s.houses.push(HouseSetup {
            name: name.into(),
            country: country.into(),
            human: false,
            start: None,
            role,
            ai: false,
        });
    }
    crate::load::new_game(rules, &empty_map(size), catalogue(), &s)
        .expect("synthetic test game loads")
}

/// The middle cell of a map of `size` (s = 2W, d = 0), offset by (dx, dy).
pub fn mid(size: i32, dx: i32, dy: i32) -> CellXY {
    CellXY::new(size + dx, size + dy)
}

/// Creates an object of type `name` for house `house` and places it at the
/// centre of `cell` (a building's top-left cell), facing `facing`, in
/// *guard*; a building also in its idle body state. Infantry stand at the
/// cell's centre position. Takes the creation draw like any
/// object creation (S-0202 claim 10).
pub fn spawn(
    w: &mut World,
    kind: crate::rules::Kind,
    name: &str,
    house: u8,
    cell: CellXY,
    facing: u16,
) -> ObjectId {
    let stale = w.rules().quirks.stale_suspended_revert; // spec: S-0215 Edge cases (quirks.stale_suspended_revert)
    let ti = w
        .rules()
        .index_of(kind, name)
        .unwrap_or_else(|| panic!("no {kind:?} type {name}"));
    let id = crate::objects::create(w, kind, ti, Some(HouseId(house)));
    let p = crate::world::Pos::centre(cell, 0);
    let z = w.surface_at(p.x, p.y);
    crate::objects::place(w, id, crate::world::Pos::new(p.x, p.y, z), facing);
    let tick = w.tick;
    if let Some(o) = w.obj_mut(id) {
        o.beh.force(Behaviour::Guard, tick, stale);
    }
    if kind == crate::rules::Kind::Building {
        // A complete building runs its idle body range (S-0108 claim 6),
        // whose stage updates set the ready mark (S-0215 claim 11).
        crate::econ::body::enter_idle(w, id, true);
    }
    id
}

/// A placing step of S-0216's scenarios: creates an infantry unit or a
/// vehicle of type `name` for house `house` (its creation draw, S-0202
/// claim 10) and places it at play time at the centre of `cell` with
/// facing value `v` ([`crate::objects::place_in_play`]); then, when given,
/// queues `behaviour` without starting it (S-0216 claim 4, S-0215 claim 7).
/// A refused unit is discarded with its creation set (our harness's choice,
/// S-0216 Open questions *A failed placement*): it never enters play, and
/// its draws and creation numbers stay used. Returns the placed unit.
pub fn spawn_in_play(
    w: &mut World,
    kind: crate::rules::Kind,
    name: &str,
    house: u8,
    cell: CellXY,
    v: u8,
    behaviour: Option<Behaviour>,
) -> Option<ObjectId> {
    let ti = w
        .rules()
        .index_of(kind, name)
        .unwrap_or_else(|| panic!("no {kind:?} type {name}"));
    let id = crate::objects::create(w, kind, ti, Some(HouseId(house)));
    if !crate::objects::place_in_play(w, id, cell, v) {
        crate::objects::discard_unplaced(w, id);
        return None;
    }
    if let Some(o) = w.obj_mut(id) {
        // spec: S-0216 claim 4 (a behaviour the placing step gives is queued by S-0215 claim 7 without being started)
        o.beh.queue(behaviour);
    }
    Some(id)
}

/// Runs `n` ticks.
pub fn run(w: &mut World, n: i32) {
    for _ in 0..n {
        crate::tick::step(w);
    }
}

/// The combat replay of the determinism gate: the scenario's map under the
/// synthetic combat rules; both sides attack-move into each other, a few
/// explicit attacks and hunts follow, and the survivors return home.
pub fn combat_replay(seed: u32, ticks: i32) -> Replay {
    let w = combat_game(seed);
    let mut rep = Replay::new(w.rules(), "synthetic-combat", setup(seed));
    let a = house_objects(&w, 0);
    let b = house_objects(&w, 1);
    let mut push = |t: i32, h: u8, o: Order| rep.record(t, HouseId(h), o);
    let attack_move = |actor: ObjectId, c: CellXY| Order::Behaviour {
        actor,
        behaviour: Behaviour::Move,
        attack_move: true,
        target: None,
        dest: Some(Target::Cell(c)),
    };
    let attack = |actor: ObjectId, t: ObjectId| Order::Behaviour {
        actor,
        behaviour: Behaviour::Attack,
        attack_move: false,
        target: Some(Target::Object(t)),
        dest: None,
    };
    for (i, &id) in a.iter().enumerate() {
        push(5, 0, attack_move(id, cell(6, i as i32 % 7 - 3)));
    }
    for (i, &id) in b.iter().enumerate() {
        push(7, 1, attack_move(id, cell(-6, i as i32 % 7 - 3)));
    }
    // Focused fire: the first two of each side attack the other's first.
    if let (Some(&a0), Some(&b0)) = (a.first(), b.first()) {
        for &id in a.iter().take(2) {
            push(120, 0, attack(id, b0));
        }
        for &id in b.iter().take(2) {
            push(125, 1, attack(id, a0));
        }
    }
    // Beta's infantry hunt; Alpha's hold an area.
    for &id in b.iter().filter(|&&id| {
        w.obj(id)
            .is_some_and(|o| o.kind == crate::rules::Kind::Infantry)
    }) {
        push(
            300,
            1,
            Order::Behaviour {
                actor: id,
                behaviour: Behaviour::Hunt,
                attack_move: false,
                target: None,
                dest: None,
            },
        );
    }
    for &id in a.iter().filter(|&&id| {
        w.obj(id)
            .is_some_and(|o| o.kind == crate::rules::Kind::Infantry)
    }) {
        push(
            310,
            0,
            Order::Behaviour {
                actor: id,
                behaviour: Behaviour::AreaGuard,
                attack_move: false,
                target: Some(Target::Cell(cell(0, 0))),
                dest: None,
            },
        );
    }
    // The survivors return home late in the run.
    for (i, &id) in a.iter().enumerate() {
        push(1500, 0, mv(id, cell(-6, i as i32 % 7 - 3)));
    }
    for (i, &id) in b.iter().enumerate() {
        push(1520, 1, mv(id, cell(6, i as i32 % 7 - 3)));
    }
    rep.ticks = ticks;
    rep
}
