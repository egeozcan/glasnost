//! A live game on a loaded map: the simulation's sandbox game (one
//! human-controlled house per object owner of the map), its pacing
//! (S-0201), the object pass's ground layer (S-0102 claims 5 and 6), the
//! mobile objects drawn from the simulation each time a tick has run, and
//! the selection and orders of the controlled house (S-0213, S-0214).
//!
//! Nothing here feeds the simulation except orders, issued between ticks
//! through `glasnost_sim::orders::issue`, and the ticks themselves. The
//! selection, the ground layer's order and the drawing are presentation
//! state the simulation never reads.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use glasnost_convert::kinds::Kind;
use glasnost_map::terrain::land;
use glasnost_render::scene::Scene;
use glasnost_render::voxel;
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Kind as SimKind, SimRules};
use glasnost_sim::targeting::{self, ClickOn, KeyCommand, Modifiers};
use glasnost_sim::world::{GameSetup, HouseId, Layer, ObjectId, World};

use super::build::{Built, GroundEntry, InfantryFrame, Inputs, Mobile, Painter, find_type};

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
    /// before this tick's movement.
    pub fn pass(&mut self) {
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
    cell_pass_len: usize,
    ground: GroundLayer,
    /// Notes the drawing of the live objects met.
    pub notes: BTreeSet<String>,
    /// Simulation types with no import type of the same name (not drawn).
    pub undrawn_types: BTreeSet<String>,
    /// Why the game stopped, when a tick failed.
    pub failed: Option<String>,
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

/// Starts the sandbox game of a map: setup, then a new game.
pub fn start(
    map: &glasnost_map::Map,
    resolved: &glasnost_rules::Resolved,
    tiles: glasnost_map::terrain::TileCatalogue,
) -> Result<(World, Arc<SimRules>, GameSetup, Vec<String>), String> {
    let rules = Arc::new(SimRules::new(&resolved.ruleset, &resolved.sim_rules_hash));
    let (setup, substituted) = glasnost_sim::load::sandbox_setup(map, &rules, SANDBOX_SEED);
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
            cell_pass_len: built.scene.cell_pass_len,
            ground: GroundLayer::default(),
            notes: BTreeSet::new(),
            undrawn_types: undrawn,
            failed: None,
        };
        // Objects enter the ground layer in the order they are put on the
        // map, which is object ID order (S-0203 claim 3); a static entry
        // enters with its building, and entries whose building the game did
        // not place enter last.
        let mut entrants: Vec<(u32, Entrant, i64)> = Vec::new();
        for (i, g) in live.static_ground.iter().enumerate() {
            let rank = g
                .object
                .and_then(|o| live.building_ids.get(&o))
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
        let world = &mut self.world;
        if let Err(p) = guarded(|| glasnost_sim::tick::step(world)) {
            self.failed = Some(format!("the simulation failed in a tick: {p}"));
            return;
        }
        self.ground.pass();
        self.sync_ground();
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
        scene.items.truncate(self.cell_pass_len);
        for e in self.ground.order() {
            match e {
                Entrant::Static(i) => {
                    if let Some(g) = self.static_ground.get(i) {
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
