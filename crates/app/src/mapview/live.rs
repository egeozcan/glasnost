//! A live game on a loaded map: the simulation's sandbox game (one
//! human-controlled house per object owner of the map), its pacing
//! (S-0201), the object pass's ground layer (S-0102 claims 5 and 6), the
//! mobile objects drawn from the simulation each time a tick has run, and
//! the selection and orders of the controlled house (S-0213, S-0214).
//!
//! Combat is drawn after the object pass: placeholders of our own for
//! projectiles in flight and for the shots, detonations and animations the
//! simulation reports in `World::effects` (drained after each tick), then
//! S-0111's status extras (health bars of the selected and hovered objects,
//! unit brackets, veterancy insignia; [`super::combat`]). Buildings the game
//! has removed leave the picture.
//!
//! Nothing here feeds the simulation except orders, issued between ticks
//! through `glasnost_sim::orders::issue`, and the ticks themselves. The
//! selection, the ground layer's order and the drawing are presentation
//! state the simulation never reads.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use glasnost_convert::kinds::Kind;
use glasnost_map::terrain::land;
use glasnost_render::projection::project_lifted;
use glasnost_render::scene::{Item, Scene};
use glasnost_render::voxel;
use glasnost_sim::combat::effects::Effect;
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Kind as SimKind, SimRules};
use glasnost_sim::targeting::{self, ClickOn, KeyCommand, Modifiers};
use glasnost_sim::world::{GameSetup, HouseId, Layer, ObjectId, World};

use super::build::{
    Built, GameMode, GroundEntry, InfantryFrame, Inputs, LiveBuilding, Mobile, Painter, find_type,
    rule_int,
};
use super::combat::{self, Cache, Counts, Marker, Ratios, Transient, TransientKind};
use super::quirks::ViewQuirks;

/// The flag that marks an [`glasnost_render::scene::Item::object`] code as
/// a simulation object ID (the low 31 bits); codes without it index the
/// static object table ([`Built::objects`]). Our own encoding.
pub const SIM_OBJECT: u32 = 1 << 31;

/// The item object code of a simulation object.
pub fn sim_code(id: ObjectId) -> u32 {
    SIM_OBJECT | id.0
}

/// The fixed seed of the viewer's sandbox games (our own choice: the
/// viewer has no lobby).
pub const SANDBOX_SEED: u32 = 1;

// ---------------------------------------------------------------------------
// Pacing (S-0201 claims 2 to 5)
// ---------------------------------------------------------------------------

/// The paced period of game-speed index `speed` in milliseconds: 16 × s for
/// s from 1 to 6, 8 at index 0 (S-0201 claims 4 and 5).
pub fn period_ms(speed: u8) -> i64 {
    match speed {
        // spec: S-0201 claim 5 (our engine paces index 0 at a fixed 8 ms)
        0 => 8,
        // spec: S-0201 claims 2 and 4 (s pacing units of 16 ms)
        s => 16 * i64::from(s.min(6)),
    }
}

/// The tick pacer: a cycle runs one tick and lasts at least its paced period
/// measured from its own start, with no catch-up (S-0201 claims 2 to 4).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Pacer {
    /// Start of the current cycle, in host milliseconds.
    cycle_start: Option<i64>,
}

impl Pacer {
    /// Whether a tick is due at host time `now_ms` at game speed `speed`.
    /// At index 0 the display rate limits the ticks (one per drawn frame),
    /// below S-0201 claim 5's 125 per second.
    /// When it is, a new cycle starts at `now_ms`: time lost to a late call
    /// is not made up (S-0201 claim 3). Call once per drawn frame, so at most
    /// one tick runs per frame.
    pub fn due(&mut self, now_ms: i64, speed: u8) -> bool {
        let due = self
            .cycle_start
            .is_none_or(|s| now_ms - s >= period_ms(speed));
        if due {
            // spec: S-0201 claim 3 (each cycle measures its paced period from its own start)
            self.cycle_start = Some(now_ms);
        }
        due
    }

    /// Milliseconds until the next tick is due (0 when it is).
    pub fn remaining(&self, now_ms: i64, speed: u8) -> i64 {
        self.cycle_start
            .map_or(0, |s| (period_ms(speed) - (now_ms - s)).max(0))
    }

    /// Forgets the current cycle (after a pause: S-0201 claims 3 and 8, the
    /// game resumes without catching up).
    pub fn reset(&mut self) {
        self.cycle_start = None;
    }
}

// ---------------------------------------------------------------------------
// The ground layer (S-0102 claim 6)
// ---------------------------------------------------------------------------

/// Something in the ground layer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Entrant {
    /// A static entry of the built scene ([`Built::ground`]).
    Static(usize),
    /// A simulation object.
    Object(ObjectId),
}

/// The object pass's ground layer: a list kept nearly sorted by sort key
/// (S-0102 claim 6).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GroundLayer {
    list: Vec<(Entrant, i64)>,
}

impl GroundLayer {
    /// The entrants in drawing order.
    pub fn order(&self) -> impl Iterator<Item = Entrant> + '_ {
        self.list.iter().map(|(e, _)| *e)
    }

    /// Whether `e` is in the layer.
    pub fn contains(&self, e: Entrant) -> bool {
        self.list.iter().any(|(x, _)| *x == e)
    }

    /// An entrant enters with sort key `key`: it goes immediately before the
    /// first entry whose key is strictly greater, or at the end.
    pub fn enter(&mut self, e: Entrant, key: i64) {
        // spec: S-0102 claim 6 (placed before the first object whose key is strictly greater)
        let at = self
            .list
            .iter()
            .position(|(_, k)| *k > key)
            .unwrap_or(self.list.len());
        self.list.insert(at, (e, key));
    }

    /// An entrant leaves.
    pub fn leave(&mut self, e: Entrant) {
        self.list.retain(|(x, _)| *x != e);
    }

    /// The once-per-tick pass: adjacent swaps from front to back where the
    /// later entry's key is strictly smaller, on the keys as they stood
    /// before this tick's movement. With
    /// `quirks.ground_layer_single_pass_sort` off, a full stable sort by key.
    pub fn pass(&mut self, q: &ViewQuirks) {
        if !q.ground_layer_single_pass_sort {
            // spec: S-0102 Edge cases (quirks.ground_layer_single_pass_sort off: fully sorted every tick, stable, by key)
            self.list.sort_by_key(|(_, k)| *k);
            return;
        }
        // spec: S-0102 claim 6 (one pass of adjacent swaps per tick, front to back)
        for i in 1..self.list.len() {
            if self.list[i].1 < self.list[i - 1].1 {
                self.list.swap(i, i - 1);
            }
        }
    }

    /// Appends an entrant at the end, unsorted (tests of claim 6's unsorted
    /// case).
    #[cfg(test)]
    pub fn list_for_test_push(&mut self, e: Entrant, key: i64) {
        self.list.push((e, key));
    }

    /// Records an entrant's new key (it takes effect in the next pass).
    pub fn set_key(&mut self, e: Entrant, key: i64) {
        if let Some(x) = self.list.iter_mut().find(|(x, _)| *x == e) {
            x.1 = key;
        }
    }
}

// ---------------------------------------------------------------------------
// The live game
// ---------------------------------------------------------------------------

/// What an input did.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct InputReport {
    /// Orders the simulation took.
    pub issued: usize,
    /// Orders it refused (S-0213 claims 1 and 2: the order block of
    /// S-0201 claim 14, or a full pending list).
    pub refused: usize,
    /// Whether the selection changed.
    pub selection_changed: bool,
}

/// A live sandbox game on a loaded map.
pub struct Live {
    /// The game state.
    pub world: World,
    /// The rules.
    pub rules: Arc<SimRules>,
    /// The setup it started from.
    pub setup: GameSetup,
    /// Owner names whose country was substituted (sandbox setup).
    pub substituted: Vec<String>,
    /// Import registration index of each simulation type (by name).
    types: BTreeMap<(SimKind, u32), Option<usize>>,
    /// Static object index (buildings of the built scene) → simulation
    /// object.
    pub building_ids: BTreeMap<u32, ObjectId>,
    painter: Painter,
    static_ground: Vec<GroundEntry>,
    /// The cell pass as built, before any building left it.
    cell_pass: Vec<Item>,
    cell_pass_len: usize,
    /// Static building objects ([`Built::objects`] indices) whose
    /// simulation object is gone, as last drawn.
    gone: BTreeSet<u32>,
    /// End of the object pass in the scene's items; the combat tail follows.
    objects_end: usize,
    /// End of the effects in the scene's items; the status extras follow.
    effects_end: usize,
    ground: GroundLayer,
    /// Images and rows the combat drawing added.
    cache: Cache,
    /// Shots, detonations and animations still shown.
    transients: Vec<Transient>,
    /// Counts of the combat events seen.
    pub counts: Counts,
    /// The selection whose status extras are drawn (presentation state).
    marked: Vec<ObjectId>,
    /// The object under the pointer (presentation state).
    hovered: Option<ObjectId>,
    /// Notes the drawing of the live objects met.
    pub notes: BTreeSet<String>,
    /// Simulation types with no import type of the same name (not drawn).
    pub undrawn_types: BTreeSet<String>,
    /// Why the game stopped, when a tick failed.
    pub failed: Option<String>,
    /// The computer houses' scripts (`glasnost_ai`), one per AI house of a
    /// skirmish.
    scripts: Vec<glasnost_ai::scripted::Script>,
}

/// The import kind of a simulation kind.
pub fn import_kind(k: SimKind) -> Option<Kind> {
    match k {
        SimKind::Infantry => Some(Kind::Infantry),
        SimKind::Vehicle => Some(Kind::Vehicles),
        SimKind::Aircraft => Some(Kind::Aircraft),
        SimKind::Building => Some(Kind::Buildings),
        SimKind::Terrain => Some(Kind::TerrainObjects),
    }
}

/// The bytes of a type name as our rules format writes it (bytes 128 to 255
/// as U+F780 to U+F7FF, docs/development.md "Import").
pub fn name_bytes(s: &str) -> Vec<u8> {
    let mut out = Vec::with_capacity(s.len());
    for c in s.chars() {
        let u = c as u32;
        if (0xF780..=0xF7FF).contains(&u) {
            out.push((u - 0xF780 + 128) as u8);
        } else {
            let mut b = [0; 4];
            out.extend_from_slice(c.encode_utf8(&mut b).as_bytes());
        }
    }
    out
}

/// A simulation type's name.
pub fn type_name(rules: &SimRules, kind: SimKind, index: u32) -> String {
    match kind {
        SimKind::Building => rules
            .buildings
            .get(index as usize)
            .map(|b| b.name.clone())
            .unwrap_or_default(),
        SimKind::Terrain => rules
            .terrain
            .get(index as usize)
            .map(|t| t.name.clone())
            .unwrap_or_default(),
        _ => rules
            .unit(kind, index)
            .map(|u| u.name.clone())
            .unwrap_or_default(),
    }
}

/// Runs `f`, turning a panic into an error on hosts that can unwind.
fn guarded<T>(f: impl FnOnce() -> T) -> Result<T, String> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)).map_err(|p| {
            p.downcast_ref::<String>()
                .cloned()
                .or_else(|| p.downcast_ref::<&str>().map(|s| (*s).to_owned()))
                .unwrap_or_else(|| "a panic".into())
        })
    }
    #[cfg(target_arch = "wasm32")]
    {
        Ok(f())
    }
}

/// Starts the game of a map: setup, then a new game. The sandbox has one
/// house per object owner; the skirmish has the user's house and a
/// computer house (the house names of the map format's placeholders, so the
/// colour slots apply), the rules' first two countries that are not
/// passive, random start positions and the setup's default values.
pub fn start(
    map: &glasnost_map::Map,
    resolved: &glasnost_rules::Resolved,
    tiles: glasnost_map::terrain::TileCatalogue,
    mode: GameMode,
) -> Result<(World, Arc<SimRules>, GameSetup, Vec<String>), String> {
    let rules = Arc::new(SimRules::new(&resolved.ruleset, &resolved.sim_rules_hash));
    let (setup, substituted) = match mode {
        GameMode::Sandbox => glasnost_sim::load::sandbox_setup(map, &rules, SANDBOX_SEED),
        GameMode::Skirmish => (skirmish_setup(&rules)?, Vec::new()),
    };
    if setup.houses.is_empty() {
        return Err("the map places no owned objects: the sandbox game has no house".into());
    }
    let world = guarded(|| {
        glasnost_sim::load::new_game(rules.clone(), map, Arc::new(tiles), &setup)
            .map_err(|e| e.to_string())
    })
    .map_err(|p| format!("the simulation failed while setting up the game: {p}"))??;
    Ok((world, rules, setup, substituted))
}

/// The viewer's skirmish setup (our own viewer choice).
fn skirmish_setup(rules: &SimRules) -> Result<GameSetup, String> {
    use glasnost_sim::econ::setup::{self, Player, Settings};
    let countries: Vec<String> = rules
        .country_names
        .iter()
        .zip(&rules.countries)
        .filter(|(n, c)| {
            !c.passive
                && n.as_str() != glasnost_sim::load::NEUTRAL
                && n.as_str() != glasnost_sim::load::SPECIAL
        })
        .map(|(n, _)| n.clone())
        .take(2)
        .collect();
    if countries.len() < 2 {
        return Err("the rules have fewer than two countries that are not passive".into());
    }
    let players = [
        Player {
            name: HUMAN_HOUSE.into(),
            country: countries[0].clone(),
            ..Default::default()
        },
        Player {
            name: COMPUTER_HOUSE.into(),
            country: countries[1].clone(),
            ai: true,
            ..Default::default()
        },
    ];
    Ok(setup::skirmish(
        rules,
        &players,
        Settings::default(),
        0,
        SANDBOX_SEED,
    ))
}

/// The house names of the map format's human and computer placeholders
/// (S-0009 Import mapping), given to the skirmish's two houses.
pub const HUMAN_HOUSE: &str = "<human player>"; // spec: S-0009 Import mapping (house names)
/// See [`HUMAN_HOUSE`].
pub const COMPUTER_HOUSE: &str = "Computer"; // spec: S-0009 Import mapping (house names)

impl Live {
    /// Wraps a started game around a scene built with `live` on: takes the
    /// static ground entries out of `built`, fills the ground layer and
    /// draws the mobile objects into `built.scene`.
    pub fn new(
        inputs: &Inputs,
        built: &mut Built,
        painter: Painter,
        started: (World, Arc<SimRules>, GameSetup, Vec<String>),
    ) -> Self {
        let (world, rules, setup, substituted) = started;
        let mut types = BTreeMap::new();
        let mut undrawn = BTreeSet::new();
        for o in world.objects.values() {
            types.entry((o.kind, o.type_index)).or_insert_with(|| {
                let name = type_name(&rules, o.kind, o.type_index);
                let found = import_kind(o.kind)
                    .and_then(|k| find_type(inputs.import, k, &name_bytes(&name)));
                if found.is_none() {
                    undrawn.insert(name);
                }
                found
            });
        }
        // Static buildings and their simulation objects: same type name and
        // top-left cell.
        let mut building_ids = BTreeMap::new();
        for (i, info) in built.objects.iter().enumerate() {
            if info.kind != "building" {
                continue;
            }
            let id = world.objects.values().find(|o| {
                o.kind == SimKind::Building
                    && o.pos.cell() == info.cell
                    && type_name(&rules, o.kind, o.type_index).as_bytes()
                        == info.type_name.as_bytes()
            });
            if let Some(o) = id {
                building_ids.insert(i as u32, o.id);
            }
        }
        // Static terrain objects (the animated ones are ground entries) and
        // their simulation objects: same type name and cell.
        let mut terrain_ids: BTreeMap<u32, ObjectId> = BTreeMap::new();
        for (i, info) in built.objects.iter().enumerate() {
            if info.kind != "terrain object" {
                continue;
            }
            let id = world.objects.values().find(|o| {
                o.kind == SimKind::Terrain
                    && o.pos.cell() == info.cell
                    && type_name(&rules, o.kind, o.type_index).as_bytes()
                        == info.type_name.as_bytes()
            });
            if let Some(o) = id {
                terrain_ids.insert(i as u32, o.id);
            }
        }
        let static_ground = std::mem::take(&mut built.ground);
        let mut live = Self {
            world,
            rules,
            setup,
            substituted,
            types,
            building_ids,
            painter,
            static_ground,
            cell_pass: built.scene.items[..built.scene.cell_pass_len].to_vec(),
            cell_pass_len: built.scene.cell_pass_len,
            gone: BTreeSet::new(),
            objects_end: built.scene.items.len(),
            effects_end: built.scene.items.len(),
            ground: GroundLayer::default(),
            cache: Cache::default(),
            transients: Vec::new(),
            counts: Counts::default(),
            marked: Vec::new(),
            hovered: None,
            notes: BTreeSet::new(),
            undrawn_types: undrawn,
            failed: None,
            scripts: Vec::new(),
        };
        // The skirmish's AI houses are played by the script (S-0412
        // claim 12: AI houses; the script is our own opponent).
        if !live.setup.sandbox {
            let starts = glasnost_ai::skirmish::start_cells(inputs.map);
            for (i, h) in live.setup.houses.iter().enumerate() {
                if h.ai && h.role == glasnost_sim::world::HouseRole::Player {
                    let mut s = glasnost_ai::scripted::Script::new(
                        HouseId(i as u8),
                        glasnost_ai::fixture::plan(),
                    );
                    s.set_starts(&live.world, &starts);
                    live.scripts.push(s);
                }
            }
        }
        // Objects enter the ground layer in the order they are put on the
        // map, which is object ID order (S-0203 claim 3, S-0009 claim 43); a
        // static entry enters with its building or animated terrain object,
        // and entries whose object the game did not place enter last.
        let mut entrants: Vec<(u32, Entrant, i64)> = Vec::new();
        for (i, g) in live.static_ground.iter().enumerate() {
            let rank = g
                .object
                .and_then(|o| live.building_ids.get(&o).or_else(|| terrain_ids.get(&o)))
                .map_or(u32::MAX, |id| id.0);
            entrants.push((rank, Entrant::Static(i), g.key));
        }
        for o in live.world.objects.values() {
            if o.kind.is_mobile() && o.on_map {
                entrants.push((o.id.0, Entrant::Object(o.id), mobile_key(o)));
            }
        }
        entrants.sort_by_key(|(rank, e, _)| (*rank, *e));
        for (_, e, key) in entrants {
            live.ground.enter(e, key);
        }
        live.redraw(inputs, &mut built.scene);
        live
    }

    /// Finds the import type of every simulation type on the map that has
    /// none yet (types the game created after the start).
    fn learn_types(&mut self, inputs: &Inputs) {
        let missing: Vec<(SimKind, u32)> = self
            .world
            .objects
            .values()
            .map(|o| (o.kind, o.type_index))
            .filter(|k| !self.types.contains_key(k))
            .collect();
        for (kind, index) in missing {
            let name = type_name(&self.rules, kind, index);
            let found =
                import_kind(kind).and_then(|k| find_type(inputs.import, k, &name_bytes(&name)));
            if found.is_none() {
                self.undrawn_types.insert(name);
            }
            self.types.insert((kind, index), found);
        }
    }

    /// The voxel buffer the game's fresh voxel draws left (S-0105 claim 34,
    /// S2).
    pub fn voxel_canvas(&self) -> &glasnost_render::voxel::VoxelCanvas {
        self.painter.canvas()
    }

    /// The ground layer.
    pub fn ground(&self) -> &GroundLayer {
        &self.ground
    }

    /// Runs one tick, then the ground layer's pass on the keys as they
    /// stood before the tick, then records the new keys and the objects
    /// that entered or left (S-0102 claim 6). Call [`Live::redraw`] after.
    pub fn step(&mut self) {
        if self.failed.is_some() {
            return;
        }
        if self.world.tick % glasnost_ai::scripted::THINK_PERIOD == 0 {
            for s in &mut self.scripts {
                for o in s.think(&self.world) {
                    orders::issue(&mut self.world, s.house, o);
                }
            }
        }
        let world = &mut self.world;
        if let Err(p) = guarded(|| glasnost_sim::tick::step(world)) {
            self.failed = Some(format!("the simulation failed in a tick: {p}"));
            return;
        }
        self.ground.pass(&ViewQuirks::default());
        self.sync_ground();
        self.take_effects();
    }

    /// Takes the tick's presentation events out of the game (they are not
    /// game state) and turns them into short-lived markers.
    fn take_effects(&mut self) {
        let now = self.shown_tick();
        let at = |p: glasnost_sim::world::Pos| project_lifted(p.x, p.y, p.z);
        for e in std::mem::take(&mut self.world.effects) {
            let (kind, pos) = match e {
                Effect::Shot { from, .. } => {
                    self.counts.shots += 1;
                    (TransientKind::Muzzle, from)
                }
                Effect::Detonation { pos, radius, .. } => {
                    self.counts.detonations += 1;
                    (TransientKind::Burst(combat::burst_half_width(radius)), pos)
                }
                Effect::Anim { pos, .. } => {
                    self.counts.anims += 1;
                    (TransientKind::Anim, pos)
                }
                Effect::Killed { .. } => {
                    self.counts.kills += 1;
                    continue;
                }
                Effect::Sound { .. } => continue,
            };
            self.transients.push(Transient {
                kind,
                at: at(pos),
                born: now,
            });
        }
        self.transients.retain(|t| t.marker(now).is_some());
        self.counts.most_in_flight = self.counts.most_in_flight.max(self.world.projectiles.len());
    }

    /// Brings the ground layer's members and keys up to date with the game.
    fn sync_ground(&mut self) {
        let gone: Vec<Entrant> = self
            .ground
            .order()
            .filter(|e| match e {
                Entrant::Object(id) => self.world.obj(*id).is_none_or(|o| !o.on_map),
                Entrant::Static(_) => false,
            })
            .collect();
        for e in gone {
            self.ground.leave(e);
        }
        let mobiles: Vec<(ObjectId, i64)> = self
            .world
            .objects
            .values()
            .filter(|o| o.kind.is_mobile() && o.on_map)
            .map(|o| (o.id, mobile_key(o)))
            .collect();
        for (id, key) in mobiles {
            let e = Entrant::Object(id);
            if self.ground.contains(e) {
                self.ground.set_key(e, key);
            } else {
                self.ground.enter(e, key);
            }
        }
    }

    /// The tick whose updates the picture shows: the last one that ran
    /// (S-0201 claim 6).
    fn shown_tick(&self) -> i32 {
        (self.world.tick - 1).max(0)
    }

    /// What one simulation object looks like now, if it is drawn.
    pub fn mobile(&self, inputs: &Inputs, id: ObjectId) -> Option<Mobile> {
        let w = &self.world;
        let o = w.obj(id)?;
        if !o.kind.is_mobile() || !o.on_map {
            return None;
        }
        let kind = import_kind(o.kind)?;
        let idx = (*self.types.get(&(o.kind, o.type_index))?)?;
        let owner = o
            .owner
            .and_then(|h| w.houses.get(usize::from(h.0)))
            .map(|h| h.name.as_bytes().to_vec())
            .unwrap_or_default();
        let unit = self.rules.unit(o.kind, o.type_index);
        let infantry = o.infantry.as_ref().map(|s| {
            let wet = || {
                o.layer == Layer::Ground
                    && w.cells.cell(o.pos.cell()).is_some_and(|c| {
                        // spec: S-0107 claim 2 (water; beach too with quirks.tread_default_water_only off)
                        c.land == land::WATER
                            || (!self.rules.quirks.tread_default_water_only
                                && c.land == land::BEACH)
                    })
            };
            // spec: S-0107 claim 2 (the current sequence's entry, else tread on water, else stand)
            let key = match s.seq {
                Some(q) => q.key(),
                None if wet() => "tread",
                None => "stand",
            };
            let e = unit.and_then(|u| u.sequences.get(key));
            InfantryFrame {
                start: e.map_or(0, |e| e.start),
                count: e.map_or(0, |e| e.count),
                multiplier: e.map_or(0, |e| e.multiplier),
                // spec: S-0107 claim 4 (the stage seen after the updates of the last tick)
                stage: if s.seq.is_some() {
                    s.stage(self.shown_tick())
                } else {
                    0
                },
            }
        });
        let fire_angle = super::build::fire_angle(inputs, kind, idx);
        Some(Mobile {
            kind,
            idx,
            owner,
            pos: (o.pos.x, o.pos.y, o.pos.z),
            // spec: S-0206 F1 (the facings' values at the current tick number)
            body: o.body.value(w.tick),
            turret: o.turret.value(w.tick),
            // spec: S-0105 claim 6 (placed level, then turned toward the fire angle since placement at tick 0)
            elevation: voxel::barrel_elevation_after(fire_angle, w.tick.max(0) as u32),
            infantry,
            object: sim_code(id),
        })
    }

    /// Rebuilds the object pass of `scene`: the cell pass stays, the static
    /// ground entries are reused and the mobile objects are drawn from the
    /// game, all in ground-layer order.
    pub fn redraw(&mut self, inputs: &Inputs, scene: &mut Scene) {
        self.learn_types(inputs);
        let ids: Vec<ObjectId> = self
            .ground
            .order()
            .filter_map(|e| match e {
                Entrant::Object(id) => Some(id),
                Entrant::Static(_) => None,
            })
            .collect();
        let mobiles: Vec<Mobile> = ids
            .iter()
            .filter_map(|&id| self.mobile(inputs, id))
            .collect();
        let (entries, notes) = self.painter.mobiles(inputs, scene, &mobiles);
        self.notes.extend(notes);
        let mut by_code: BTreeMap<u32, GroundEntry> = mobiles
            .iter()
            .zip(entries)
            .map(|(m, e)| (m.object, e))
            .collect();
        // Buildings the game removed leave the cell pass and the ground
        // layer.
        let gone: BTreeSet<u32> = self
            .building_ids
            .iter()
            .filter(|(_, id)| self.world.obj(**id).is_none_or(|o| !o.on_map))
            .map(|(i, _)| *i)
            .collect();
        if gone != self.gone {
            scene.items.clear();
            scene.items.extend(
                self.cell_pass
                    .iter()
                    .filter(|i| i.object.is_none_or(|o| !gone.contains(&o))),
            );
            self.cell_pass_len = scene.items.len();
            scene.cell_pass_len = self.cell_pass_len;
            self.gone = gone;
        }
        scene.items.truncate(self.cell_pass_len);
        // The game's own buildings (deployed or placed) follow the cell
        // pass, in object ID order (our placeholder order; the map's
        // buildings keep their place in the cell pass).
        let own: Vec<LiveBuilding> = self
            .world
            .objects
            .values()
            .filter(|o| {
                o.kind == SimKind::Building
                    && o.on_map
                    && !self.building_ids.values().any(|id| *id == o.id)
            })
            .filter_map(|o| {
                let idx = (*self.types.get(&(o.kind, o.type_index))?)?;
                let owner = o
                    .owner
                    .and_then(|h| self.world.houses.get(usize::from(h.0)))
                    .map(|h| h.name.as_bytes().to_vec())
                    .unwrap_or_default();
                let strength = self
                    .rules
                    .buildings
                    .get(o.type_index as usize)
                    .map_or(0, |b| b.strength);
                Some(LiveBuilding {
                    idx,
                    cell: o.pos.cell(),
                    owner,
                    health: (o.health, strength),
                    facing: o.turret.value(self.world.tick),
                    object: sim_code(o.id),
                })
            })
            .collect();
        if !own.is_empty() {
            let (drawn, notes) = self.painter.buildings(inputs, scene, &own);
            self.notes.extend(notes);
            for items in drawn {
                scene.items.extend(items);
            }
        }
        for e in self.ground.order() {
            match e {
                Entrant::Static(i) => {
                    if let Some(g) = self.static_ground.get(i)
                        && g.object.is_none_or(|o| !self.gone.contains(&o))
                    {
                        scene.items.extend_from_slice(&g.items);
                    }
                }
                Entrant::Object(id) => {
                    if let Some(g) = by_code.remove(&sim_code(id)) {
                        scene.items.extend(g.items);
                    }
                }
            }
        }
        self.objects_end = scene.items.len();
        self.draw_effects(scene);
        self.draw_status(inputs, scene);
    }

    /// Draws the combat placeholders after the object pass: projectiles in
    /// flight (invisible ones are not drawn) and the shots, detonations and
    /// animations still shown. Our own presentation, provisional.
    fn draw_effects(&mut self, scene: &mut Scene) {
        scene.items.truncate(self.objects_end);
        let rules = self.rules.clone();
        for p in self.world.projectiles.values() {
            let kind = rules.combat.projectile_of(p.weapon);
            if kind.is_some_and(|k| k.inviso) {
                continue;
            }
            let m = Marker::Projectile {
                arcing: kind.is_some_and(|k| k.arcing),
                homing: kind.is_some_and(|k| k.rot > 0),
            };
            // spec: S-0101 F1, F2 (the projected point of the position with its height)
            let at = project_lifted(p.pos.x, p.pos.y, p.pos.z);
            let item = self.cache.marker_item(scene, m, at);
            scene.items.push(item);
        }
        let now = self.shown_tick();
        for t in &self.transients {
            if let Some(m) = t.marker(now) {
                let item = self.cache.marker_item(scene, m, t.at);
                scene.items.push(item);
            }
        }
        self.effects_end = scene.items.len();
    }

    /// Sets the objects whose status extras show: the selection and the
    /// object under the pointer (S-0111 claim 3). Returns whether they
    /// changed; then call [`Live::redraw_status`].
    pub fn set_marks(&mut self, selected: &[ObjectId], hovered: Option<ObjectId>) -> bool {
        // spec: S-0111 Terms (hovered: under the pointer while not selected)
        let hovered = hovered.filter(|h| !selected.contains(h));
        if self.marked == selected && self.hovered == hovered {
            return false;
        }
        self.marked = selected.to_vec();
        self.hovered = hovered;
        true
    }

    /// Redraws only the status extras (after [`Live::set_marks`]).
    pub fn redraw_status(&mut self, inputs: &Inputs, scene: &mut Scene) {
        self.draw_status(inputs, scene);
    }

    /// Whether a unit's selection is drawn by S-0111's bracket image (the
    /// bracket sprite and the overlay palette are there); buildings get no
    /// bracket image (claim 8's corner brackets are not drawn yet).
    pub fn bracket_drawn(&self, assets: &super::build::Assets, id: ObjectId) -> bool {
        self.world.obj(id).is_some_and(|o| o.kind.is_mobile())
            && assets.get(combat::BRACKET_SPRITE).is_some()
            && assets.get(combat::OVERLAY_PALETTE).is_some()
    }

    /// S-0111's status extras after the effects: every object's veterancy
    /// insignia (claim 18), then the health bars of the selected objects
    /// (with the unit bracket) and of the hovered one (claims 3 to 7). Within
    /// an object the bar covers the insignia (State and ordering).
    fn draw_status(&mut self, inputs: &Inputs, scene: &mut Scene) {
        scene.items.truncate(self.effects_end);
        let r = self.rules.clone();
        let ratios = Ratios {
            yellow: r.general.condition_yellow,
            red: r.general.condition_red,
            mode: r.mode,
        };
        let shows: Vec<(ObjectId, bool)> = self
            .marked
            .iter()
            .map(|&id| (id, true))
            .chain(self.hovered.map(|h| (h, false)))
            .collect();
        let mut items = Vec::new();
        for o in self.world.objects.values() {
            if !o.on_map || o.kind == SimKind::Terrain {
                continue;
            }
            let infantry = o.kind == SimKind::Infantry;
            let bar = shows.iter().find(|(id, _)| *id == o.id).map(|(_, s)| *s);
            let insignia = combat::insignia(o.combat.experience, infantry);
            if bar.is_none() && insignia.is_none() {
                continue;
            }
            // spec: S-0111 Terms (status anchor: the centre at its world height, projected; S-0204 claim 4 for a building's centre)
            let p = self.world.reference_point(o.id);
            let anchor = project_lifted(p.x, p.y, p.z);
            let import_idx = self.types.get(&(o.kind, o.type_index)).copied().flatten();
            let mut pips: Vec<combat::Pip> = insignia.into_iter().collect();
            if let Some(selected) = bar {
                if o.kind == SimKind::Building {
                    let strength = r
                        .buildings
                        .get(o.type_index as usize)
                        .map_or(0, |b| b.strength);
                    // spec: S-0111 Import mapping (the building art's height field, absent 2)
                    let levels = import_idx
                        .and_then(|i| {
                            self.painter
                                .art_int(inputs.import, Kind::Buildings, i, b"Height")
                        })
                        .unwrap_or(combat::DEFAULT_ART_HEIGHT);
                    let (w, d) = o.foundation;
                    pips.extend(combat::building_bar(
                        w, d, levels, o.health, strength, &ratios,
                    ));
                } else {
                    let strength = r.unit(o.kind, o.type_index).map_or(0, |u| u.strength);
                    // spec: S-0111 Import mapping (PixelSelectionBracketDelta, absent 0)
                    let delta = match (import_kind(o.kind), import_idx) {
                        (Some(k), Some(i)) => {
                            rule_int(inputs.import, k, i, b"PixelSelectionBracketDelta", 0)
                        }
                        _ => 0,
                    };
                    pips.extend(combat::unit_bar(
                        infantry, delta, o.health, strength, selected, &ratios,
                    ));
                }
            }
            items.extend(self.cache.pip_items(
                scene,
                inputs.assets,
                anchor,
                &pips,
                Some(sim_code(o.id)),
            ));
        }
        scene.items.extend(items);
    }

    /// The simulation object an item code names: a live object, or a static
    /// building's simulation object.
    pub fn object_of_code(&self, code: u32) -> Option<ObjectId> {
        if code & SIM_OBJECT != 0 {
            Some(ObjectId(code & !SIM_OBJECT))
        } else {
            self.building_ids.get(&code).copied()
        }
    }

    /// The first house in house order that owns a mobile object on the
    /// map, else the first house.
    pub fn default_house(&self) -> HouseId {
        (0..self.world.houses.len())
            .map(|i| HouseId(i as u8))
            .find(|&h| {
                self.world
                    .objects
                    .values()
                    .any(|o| o.kind.is_mobile() && o.on_map && o.owner == Some(h))
            })
            .unwrap_or(HouseId(0))
    }

    /// Issues orders in `house`'s name (between ticks).
    fn issue(&mut self, house: HouseId, list: Vec<Order>, report: &mut InputReport) {
        for o in list {
            if orders::issue(&mut self.world, house, o) {
                report.issued += 1;
            } else {
                report.refused += 1;
            }
        }
    }

    /// Issues house orders made elsewhere (the cheer command) in `house`'s
    /// name.
    pub fn issue_orders(&mut self, house: HouseId, list: Vec<Order>) -> InputReport {
        let mut report = InputReport::default();
        self.issue(house, list, &mut report);
        report
    }

    /// Drops selected objects that no longer exist or can't be selected.
    pub fn prune(&self, selection: &mut Vec<ObjectId>) {
        selection.retain(|&id| targeting::selectable(&self.world, id));
    }

    /// A click for `house` through the input rules (S-0214). The orders go
    /// out in `house`'s name; the selection is the caller's.
    pub fn click(
        &mut self,
        house: HouseId,
        selection: &mut Vec<ObjectId>,
        on: ClickOn,
        mods: Modifiers,
    ) -> InputReport {
        self.prune(selection);
        let r = targeting::click(&self.world, house, selection, on, mods);
        let mut report = InputReport::default();
        if let Some(s) = r.selection {
            report.selection_changed = s != *selection;
            *selection = s;
        }
        self.issue(house, r.orders, &mut report);
        report
    }

    /// A key command for `house`'s selection (S-0213 table T2).
    pub fn key(
        &mut self,
        house: HouseId,
        selection: &mut Vec<ObjectId>,
        cmd: KeyCommand,
    ) -> InputReport {
        self.prune(selection);
        let list = targeting::key_command(&self.world, house, selection, cmd);
        let mut report = InputReport::default();
        self.issue(house, list, &mut report);
        report
    }
}

/// A mobile object's ground-layer sort key, X + Y of its position (S-0102
/// claim 5).
pub fn mobile_key(o: &glasnost_sim::world::Object) -> i64 {
    // spec: S-0102 claim 5 (sort key X + Y in leptons; height plays no part)
    i64::from(o.pos.x) + i64::from(o.pos.y)
}
