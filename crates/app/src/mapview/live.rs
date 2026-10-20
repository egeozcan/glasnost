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
use glasnost_map::CellXY;
use glasnost_map::terrain::land;
use glasnost_render::projection::project_lifted;
use glasnost_render::scene::{Item, Scene};
use glasnost_render::voxel;
use glasnost_sim::combat::effects::Effect;
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind as SimKind, SimRules};
use glasnost_sim::targeting::{self, ClickOn, KeyCommand, Modifiers};
use glasnost_sim::world::{GameSetup, HouseId, Layer, Object, ObjectId, World};

use super::anims::{self, AnimQuirks, Playback, Player};
use super::build::{
    AnimDraw, Attitude, Built, GameMode, GroundEntry, InfantryFrame, Inputs, LiveBuilding, Mobile,
    Painter, ShadowAnchor, find_type, rule_int,
};
use super::combat::{self, Cache, Counts, Marker, Ratios, Transient, TransientKind};
use super::slot_play;
use super::unload::{self, Unloading};

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
    /// A playing animation of the drawing (by its serial number).
    Anim(u32),
    /// A static entry of a layer other than ground ([`Built::layered`]).
    Layered(usize),
    /// A particle (S-0616 claim 30), by its object number.
    Particle(ObjectId),
}

/// The object pass's ground layer: a list fully sorted by sort key for
/// every drawn frame (S-0102 claim 6).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GroundLayer {
    list: Vec<(Entrant, i64)>,
}

impl GroundLayer {
    /// The entrants in drawing order.
    pub fn order(&self) -> impl Iterator<Item = Entrant> + '_ {
        self.list.iter().map(|(e, _)| *e)
    }

    /// The simulation objects of the layer in drawing order; static
    /// entries consume no place (S-0214 claim 25).
    pub fn objects(&self) -> impl Iterator<Item = ObjectId> + '_ {
        self.order().filter_map(|e| match e {
            Entrant::Object(id) => Some(id),
            _ => None,
        })
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

    /// Records an entrant's new key (it takes effect at the next
    /// [`GroundLayer::sort`]).
    pub fn set_key(&mut self, e: Entrant, key: i64) {
        if let Some(x) = self.list.iter_mut().find(|(x, _)| *x == e) {
            x.1 = key;
        }
    }

    /// Brings the layer up to date for a drawn frame: `members` are the
    /// entrants of the frame with their current keys, in the order the
    /// newcomers among them entered. Leavers are dropped, every member
    /// takes its current key, newcomers go after the existing members, and
    /// the whole list is sorted by key, stably.
    pub fn update(&mut self, members: &[(Entrant, i64)]) {
        self.list
            .retain(|(e, _)| members.iter().any(|(m, _)| m == e));
        for &(e, key) in members {
            if self.contains(e) {
                self.set_key(e, key);
            } else {
                // spec: S-0102 claim 6 (a newcomer follows existing equal-key objects)
                self.list.push((e, key));
            }
        }
        self.sort();
    }

    /// Fully orders the layer by key (S-0102 claim 6): a stable sort, so
    /// equal-key entrants keep their previous relative order.
    pub fn sort(&mut self) {
        // spec: S-0102 claim 6 (fully ordered by increasing sort key in every drawn frame, using the keys
        // of that frame; equal-key objects retain their previous relative order; a departure)
        self.list.sort_by_key(|(_, k)| *k);
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
    /// The static objects' entries of the other layers, in entering order
    /// (S-0102 claim 7).
    static_layered: Vec<(super::slots::Layer, GroundEntry)>,
    /// The cell pass as built, before any building left it.
    cell_pass: Vec<Item>,
    cell_pass_len: usize,
    /// The game's global lighting the cell pass was last lit with: the
    /// current ambient and the chosen set (S-0104 claims 4 and 5).
    lit: (i32, glasnost_sim::lighting::Choice),
    /// Static objects ([`Built::objects`] indices) left out of the cell
    /// pass, as last drawn: buildings whose simulation object is gone, the
    /// map's gates (drawn live) and the overlays of cells that changed.
    gone: BTreeSet<u32>,
    /// The map's gates ([`Built::objects`] indices), drawn with the game's
    /// own buildings so that their door shows (S-0108 claim 4).
    live_gates: BTreeSet<u32>,
    /// Each cell's static overlay objects ([`Built::objects`] indices).
    overlay_codes: BTreeMap<CellXY, Vec<u32>>,
    /// Each cell's overlay and overlay data value when the game started.
    start_overlays: BTreeMap<CellXY, (u8, u8)>,
    /// The cells whose overlay changed since the start (S-0112 claim 7,
    /// S-0210 claim 8, S-0406 claim 10, S-0408 claim 12, harvesting), with
    /// their overlay and data value as last drawn.
    changed_overlays: BTreeMap<CellXY, (u8, u8)>,
    /// The overlay items of the changed cells, as last drawn.
    live_overlays: Vec<Item>,
    /// End of the object pass in the scene's items; the combat tail follows.
    objects_end: usize,
    /// End of the effects in the scene's items; the status extras follow.
    effects_end: usize,
    ground: GroundLayer,
    /// The mobile objects' layers and the lists of the layers other than
    /// ground (S-0102 claims 3, 4 and 7; presentation state).
    layers: super::layers::Layers,
    /// The drawing's quirk switches (S-0102 Edge cases).
    view_quirks: super::quirks::ViewQuirks,
    /// Images and rows the combat drawing added.
    cache: Cache,
    /// Shots, detonations and animations still shown.
    transients: Vec<Transient>,
    /// Laser beams still shown (S-0518 claim 6, S-0615 claims 15 and 16).
    beams: Vec<combat::Beam>,
    /// Each house's display RGB by house index (S-0103 claim 15), the
    /// stand-in for its laser colour (Q-1521).
    house_rgb: Vec<[u8; 3]>,
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
    /// The playing animations (S-0109), in creation order.
    anims: Vec<LiveAnim>,
    /// The serial number of the next animation.
    anim_serial: u32,
    /// Every animation type of the import by upper-case name (the first
    /// registered of a name, S-0003 claim 14): its index, art fields and
    /// sprite image-frame count.
    anim_table: BTreeMap<Vec<u8>, AnimEntry>,
    /// Every particle type of the simulation's rules, by its type index:
    /// its import entry and whether its art has a sprite (S-0007 claim
    /// 28), `None` for a name the import does not list.
    particle_table: Vec<Option<(super::particles::ParticleEntry, bool)>>,
    /// The drawing's cosmetic stream, for the playback draws the
    /// simulation does not take (a seam: S-0109 claim 28 puts them on the
    /// scenario stream once playback is in the simulation).
    cosmetic: glasnost_sim::rng::Rng,
    /// Whether our own debug markers are drawn for what has no art (the
    /// viewer); the game client draws only art.
    debug_markers: bool,
    /// The move-feedback animation's type name (S-0214 claim 30).
    move_flash: Option<Vec<u8>>,
    /// The house whose eyes the cloak drawing uses (S-0516 claims 28 and
    /// 29); `None` (the viewer) draws every object as its owner sees it.
    viewer: Option<HouseId>,
    /// The serial of the teleport source marker shown to the viewer
    /// (S-0513 claim 33), while it plays.
    source_marker: Option<u32>,
    /// Each building's running slot animations (S-0108; presentation
    /// state).
    slot_anims: BTreeMap<ObjectId, slot_play::Slots<SlotAnim>>,
    /// Each building type's slot definitions, by import registration
    /// index, read when the type is first drawn.
    slot_defs: BTreeMap<usize, slot_play::Defs>,
    /// The walls' slot starts and stops the last tick reported (S-0606
    /// claims 6, 12 and 16): building, slots, start, damaged variant.
    special_events: Vec<(ObjectId, [u8; 2], bool, bool)>,
}

/// A running slot animation of a building (S-0108 Terms), played by
/// S-0109's playback.
#[derive(Clone, Debug)]
pub struct SlotAnim {
    player: Player,
    /// Its current type's entry.
    entry: AnimEntry,
}

/// Starts slot animations for [`slot_play`]: an animation type of the
/// import by name, created in tick `now`.
struct SlotMaker<'a, 'b> {
    env: &'a mut AnimEnv<'b>,
    now: i32,
}

impl slot_play::Make<SlotAnim> for SlotMaker<'_, '_> {
    fn make(&mut self, name: &[u8], from: Option<&SlotAnim>) -> Option<SlotAnim> {
        // spec: S-0108 Terms (a name that names no animation type starts nothing)
        let (entry, play) = self.env.lookup(name);
        let entry = entry?;
        let mut player = Player::new(
            name,
            play,
            self.now,
            None,
            &mut *self.env,
            &AnimQuirks::default(),
        );
        if let Some(old) = from {
            // spec: S-0108 Terms (the new animation continues from the old one's image frame)
            player.continue_from(old.player.frame());
        }
        Some(SlotAnim { player, entry })
    }
}

/// An animation type of the import, for playback.
#[derive(Clone, Debug)]
struct AnimEntry {
    idx: usize,
    t: super::slots::AnimType,
    n: i32,
    sprite: bool,
}

/// A playing animation of the drawing (S-0109), unattached (claim 22: the
/// simulation reports no owner for the animations it creates).
#[derive(Clone, Debug)]
struct LiveAnim {
    serial: u32,
    player: Player,
    /// Its current type's entry, `None` for a name the import does not
    /// list.
    entry: Option<AnimEntry>,
    pos: glasnost_sim::world::Pos,
    /// Its layer, evaluated when it was placed (S-0102 claims 3 and 4).
    layer: super::slots::Layer,
    /// Tick of its first update (S-0109 claim 5).
    first_update: i32,
    /// The object it is attached to (S-0109 claim 22), if any: a
    /// paratrooper's parachute (S-0522 claim 13).
    owner: Option<ObjectId>,
    /// Its carrier's house name and the cell light sampled when it was
    /// attached (S-0522 claim 18).
    carrier: Option<(Vec<u8>, i32)>,
    /// The burning walker it shows (S-0615 claims 4 to 11), if any: its
    /// position and image frame are the simulation's walker's.
    walker: Option<ObjectId>,
    /// The drainer it is attached to (S-0608 claim 5), if it is a drain
    /// animation: it follows the drainer and is removed when the drainer's
    /// link ends (claim 14).
    drainer: Option<ObjectId>,
    /// The infantry-making animation it shows (S-0611 claims 8 to 12), if
    /// any: the simulation plays it, so its type, image frame and owner
    /// colour are the simulation's.
    maker: Option<ObjectId>,
    /// The domination animation it shows (S-0612 claims 1 and 8), if any:
    /// the simulation plays it, so its type and image frame are the
    /// simulation's.
    dominating: Option<ObjectId>,
    /// The image frame a burning walker shows (S-0615 F1, or its expiring
    /// stage), in place of the playback's.
    frame: Option<i32>,
    /// The firer a shot animation is attached to and its displacement from
    /// the firer's position at creation (S-0301 claim 22, S-0109 claim 22).
    shooter: Option<(ObjectId, glasnost_sim::world::Pos)>,
    /// The depth adjustment its creator gave in place of its type's (a
    /// building's shot animation, S-0301 F8).
    z_adjust: Option<i32>,
}

/// What playback asks of the live game: the follow-up types, the
/// cosmetic stream, the game speed and the speed switch.
struct AnimEnv<'a> {
    table: &'a BTreeMap<Vec<u8>, AnimEntry>,
    rules: &'a SimRules,
    rng: &'a mut glasnost_sim::rng::Rng,
    speed: u8,
}

impl AnimEnv<'_> {
    /// The entry and fields in play of type `name`.
    fn lookup(&self, name: &[u8]) -> (Option<AnimEntry>, Playback) {
        let Some(e) = self.table.get(&name.to_ascii_uppercase()) else {
            // spec: S-0109 Edge cases (a name that names no type: absent-key defaults, no sprite)
            return (None, Playback::missing());
        };
        let q = &self.rules.quirks;
        let delay = anims::rate_delay(e.t.rate, q.anim_fast_rate_freezes);
        // spec: S-0109 claim 3, F1 (the range [d₁′, d₂′], as the simulation converted it)
        let random = self
            .rules
            .animation(&glasnost_convert::names::encode(name))
            .map_or((0, 0), |a| a.random_delay);
        let play = anims::playback_of(&e.t, e.n, delay, random, &AnimQuirks::default());
        (Some(e.clone()), play)
    }
}

impl anims::Env for AnimEnv<'_> {
    fn follow_up(&mut self, name: &[u8]) -> Playback {
        self.lookup(name).1
    }
    fn draw(&mut self, lo: i32, hi: i32) -> i32 {
        self.rng
            .ranged(lo, hi, self.rules.quirks.ranged_draw_half_span)
    }
    fn speed(&self) -> u8 {
        self.speed
    }
    fn normalise(&self) -> bool {
        self.rules.quirks.speed_normalised_delays
    }
}

/// The surface anchor of an aircraft's voxel shadow (S-0105 claim 40): the
/// terrain height at its exact position, and whether a qualifying high
/// bridge raises the anchor.
pub fn shadow_anchor(w: &World, o: &Object) -> ShadowAnchor {
    use glasnost_sim::cells::Axis;
    let ground = w.surface_at(o.pos.x, o.pos.y);
    // spec: S-0105 claim 40 and Terms (marked on a bridge deck: S-0501 claim 43's aircraft deck mark, not the render layer or an occupancy inference; landing sets it, ordinary climbing clears it, horizontal flight and hovering leave it)
    let marked = o.air.as_ref().is_some_and(|a| a.deck);
    // spec: S-0105 Terms (a = z − H − b, b = 416 while marked on a bridge deck, otherwise 0)
    let a = o.pos.z - ground - if marked { 416 } else { 0 };
    let c = o.pos.cell();
    // spec: S-0105 claim 40 (the neighbour one cell towards y − 1 for an axis along cell x, towards x − 1 for an axis along cell y)
    let neighbour = match w.cells.axis(c) {
        Axis::X => Some(CellXY::new(c.x, c.y - 1)),
        Axis::Y => Some(CellXY::new(c.x - 1, c.y)),
        Axis::None => None,
    };
    // spec: S-0105 claim 40 (a high-bridge cell and neighbour, and a ≥ 416 when not marked on the deck, a ≥ 0 when marked)
    let deck = w.cells.is_bridge(c)
        && neighbour.is_some_and(|n| w.cells.is_bridge(n))
        && a >= if marked { 0 } else { 416 };
    ShadowAnchor { ground, deck }
}

/// How an aircraft body is tilted, read-only from the game state (S-0105
/// claims 41, 42): a rocket missile's current nose pitch (S-0506) and its set's
/// pitch fractions, or a flying-locomotor aircraft's signed height, current
/// speed fraction and unfinished aiming-facing turn.
pub fn attitude(w: &World, o: &Object) -> Attitude {
    if let Some(d) = glasnost_sim::rocket::drawing(w, o.id) {
        return Attitude::Rocket {
            pitch: d.pitch,
            initial: d.initial,
            final_: d.final_,
        };
    }
    if !glasnost_sim::air::is_aircraft(w, o.id) {
        return Attitude::None;
    }
    let air = o.air.as_ref();
    // spec: S-0105 claim 42 (the signed height used in claim 40: a = z − H − b, b = 416 while marked on a bridge deck)
    let marked = air.is_some_and(|a| a.deck);
    let height = o.pos.z - w.surface_at(o.pos.x, o.pos.y) - if marked { 416 } else { 0 };
    Attitude::Flying {
        height,
        speed: air.map_or(0, |a| a.cur.0),
        // spec: S-0105 claim 42 (S-0206's signed shorter turn of the aiming facing, positive clockwise; an unfinished turn only)
        turn: if o.turret.turning(w.tick) {
            o.turret.step.signum()
        } else {
            0
        },
        crashing: air.is_some_and(|a| a.crash.is_some()),
    }
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
pub fn guarded<T>(f: impl FnOnce() -> T) -> Result<T, String> {
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
    start_with(map, resolved, tiles, mode, None)
}

/// Makes a game setup from the map's typed rules (the game client's lobby).
pub type MakeSetup<'a> = &'a dyn Fn(&SimRules) -> Result<GameSetup, String>;

/// [`start`] with the setup made by `make` when given (the game client's
/// skirmish of S-0412), else by `mode`.
pub fn start_with(
    map: &glasnost_map::Map,
    resolved: &glasnost_rules::Resolved,
    tiles: glasnost_map::terrain::TileCatalogue,
    mode: GameMode,
    make: Option<MakeSetup<'_>>,
) -> Result<(World, Arc<SimRules>, GameSetup, Vec<String>), String> {
    let rules = Arc::new(SimRules::new(&resolved.ruleset, &resolved.sim_rules_hash));
    let (setup, substituted) = match (make, mode) {
        (Some(f), _) => (f(&rules)?, Vec::new()),
        (None, GameMode::Sandbox) => glasnost_sim::load::sandbox_setup(map, &rules, SANDBOX_SEED),
        (None, GameMode::Skirmish) => (skirmish_setup(&rules)?, Vec::new()),
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

/// Neutral brightness, ‰ (S-0103 claim 8: a deploy or undeploy animation is
/// drawn at brightness 1000).
const NEUTRAL_LIGHT: i32 = 1000; // spec: S-0103 claim 8 (brightness of row 3 for deploy and undeploy animations)

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
        // spec: S-0103 claim 15 (a house's display RGB, sampled once at colour setup; 8-bit channels)
        let house_rgb = world
            .houses
            .iter()
            .map(|h| {
                super::build::owner_radar_colour(
                    inputs.import,
                    inputs.options,
                    h.name.as_bytes(),
                    [8, 8, 8], // nocite: 8-bit channels, no display narrowing
                )
            })
            .collect();
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
        // The map's gates are drawn with the game's own buildings, so that
        // their door shows (S-0108 claim 4); so are its occupiable buildings,
        // so that their occupied frame shows (S-0108 claim 2).
        let live_gates: BTreeSet<u32> = building_ids
            .iter()
            .filter(|(_, id)| {
                world
                    .obj(**id)
                    .and_then(|o| rules.buildings.get(o.type_index as usize))
                    .is_some_and(|b| b.gate || b.building.can_be_occupied)
            })
            .map(|(code, _)| *code)
            .collect();
        let mut overlay_codes: BTreeMap<CellXY, Vec<u32>> = BTreeMap::new();
        for (i, info) in built.objects.iter().enumerate() {
            if info.kind == "overlay" {
                overlay_codes.entry(info.cell).or_default().push(i as u32);
            }
        }
        let start_overlays = world
            .cells
            .grid
            .iter()
            .map(|(c, cell)| (c, (cell.overlay, cell.overlay_data)))
            .collect();
        let static_ground = std::mem::take(&mut built.ground);
        let static_layered = std::mem::take(&mut built.layered);
        let mut anim_table = BTreeMap::new();
        if let Some(reg) = inputs.import.outcome.registries.get(&Kind::Animations) {
            for (idx, ty) in reg.iter().enumerate() {
                let Some((t, n)) = painter.anim_type(inputs, idx) else {
                    continue;
                };
                let sprite = n > 0;
                // spec: S-0003 claim 14 (names match with ASCII case folding; the first registered wins, as find_type)
                anim_table
                    .entry(ty.name.to_ascii_uppercase())
                    .or_insert(AnimEntry { idx, t, n, sprite });
            }
        }
        let names: Vec<Vec<u8>> = rules
            .gas
            .particles
            .iter()
            .map(|t| name_bytes(&t.name))
            .collect();
        let particle_table = super::particles::entries(inputs.import, &names)
            .into_iter()
            .map(|e| {
                e.map(|e| {
                    let sprite = inputs
                        .import
                        .arts
                        .get(&(Kind::Particles, e.idx))
                        .is_some_and(|a| a.sprite.is_some());
                    (e, sprite)
                })
            })
            .collect();
        // Our own choice: the cosmetic stream takes the game's seed, like the
        // unsynchronised stream of S-0202 claim 8.
        let cosmetic = glasnost_sim::rng::Rng::new(setup.seed);
        // spec: S-0214 Import mapping (rules [General] MoveFlash, an animation type name; none initially)
        let move_flash = inputs
            .import
            .outcome
            .globals
            .get(&b"General"[..])
            .and_then(|s| s.get(&b"MoveFlash"[..]))
            .and_then(|k| k.value.as_deref())
            .map(|v| glasnost_ini::value::text(v, usize::MAX).to_vec())
            .filter(|n| !n.is_empty() && !glasnost_ini::value::is_no_type(n));
        let mut live = Self {
            world,
            rules,
            setup,
            substituted,
            types,
            building_ids,
            painter,
            static_ground,
            static_layered,
            cell_pass: built.scene.items[..built.scene.cell_pass_len].to_vec(),
            cell_pass_len: built.scene.cell_pass_len,
            // spec: S-0104 claim 1 (the scene was lit with the map's normal set)
            lit: (
                inputs.map.lighting.normal.ambient,
                glasnost_sim::lighting::Choice::Normal,
            ),
            gone: BTreeSet::new(),
            live_gates,
            overlay_codes,
            start_overlays,
            changed_overlays: BTreeMap::new(),
            live_overlays: Vec::new(),
            objects_end: built.scene.items.len(),
            effects_end: built.scene.items.len(),
            ground: GroundLayer::default(),
            layers: super::layers::Layers::default(),
            view_quirks: super::quirks::ViewQuirks::default(),
            cache: Cache::default(),
            transients: Vec::new(),
            beams: Vec::new(),
            house_rgb,
            counts: Counts::default(),
            marked: Vec::new(),
            hovered: None,
            notes: BTreeSet::new(),
            undrawn_types: undrawn,
            failed: None,
            scripts: Vec::new(),
            anims: Vec::new(),
            anim_serial: 0,
            anim_table,
            particle_table,
            cosmetic,
            debug_markers: true,
            move_flash,
            viewer: None,
            source_marker: None,
            slot_anims: BTreeMap::new(),
            slot_defs: BTreeMap::new(),
            special_events: Vec::new(),
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
        // The static entries of the other layers enter at the start, in
        // their order (S-0102 claim 7); then every mobile object is placed
        // in the layer it evaluates to (claim 4).
        for i in 0..live.static_layered.len() {
            let layer = live.static_layered[i].0;
            live.layers.place(Entrant::Layered(i), layer);
        }
        live.layers.evaluate_mobiles(&live.world, false, true);
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
            if o.kind.is_mobile()
                && o.on_map
                && live.layers.of(o.id) == Some(super::slots::Layer::Ground)
            {
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
            .flat_map(|o| {
                // The deployed image type is drawn in place of its vehicle (S-0517 claim 12).
                let image = glasnost_sim::econ::inplace::deployed_image(&self.world, o.id)
                    .map(|i| (o.kind, i));
                std::iter::once((o.kind, o.type_index)).chain(image)
            })
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

    /// Runs one tick, then brings the ground layer up to date: the objects
    /// that entered or left, the new keys, and a full sort on them (S-0102
    /// claim 6). Call [`Live::redraw`] after.
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
        self.after_tick();
    }

    /// What [`Live::step`] does after its tick, for a tick run elsewhere
    /// (the game client's `glasnost_ai::game::Game`): the ground layer's
    /// members, keys and order, and the tick's presentation events
    /// (taken out of the world: read them before).
    pub fn after_tick(&mut self) {
        self.take_effects();
        self.update_slots();
        self.sync_layers(true);
        self.sync_ground();
    }

    /// Sets the drawing's quirk switches (S-0102 Edge cases).
    pub fn set_view_quirks(&mut self, q: super::quirks::ViewQuirks) {
        self.view_quirks = q;
    }

    /// The layer a mobile object was last put into (S-0102 claims 3 and 4).
    pub fn layer_of(&self, id: ObjectId) -> Option<super::slots::Layer> {
        self.layers.of(id)
    }

    /// The list of a layer other than ground, in drawing order (S-0102
    /// claim 7).
    pub fn layer_list(&self, layer: super::slots::Layer) -> Vec<Entrant> {
        self.layers.list(layer).to_vec()
    }

    /// Brings the layers' membership up to date (S-0102 claims 3, 4 and
    /// 7): the mobile objects at claim 4's moments (`tick`: after a tick),
    /// the animations of a layer other than ground and the particles (in
    /// the air layer, S-0616 claim 30) entering at the end of their list
    /// when the drawing first sees them and leaving when they are gone.
    /// Newcomers of one tick enter as mobile objects, then animations by
    /// creation, then particles by object number (our order within a tick).
    fn sync_layers(&mut self, tick: bool) {
        use super::slots::Layer;
        let events_only = self.view_quirks.layer_change_on_events_only;
        self.layers.evaluate_mobiles(&self.world, tick, events_only);
        let anims: BTreeMap<u32, Layer> = self.anims.iter().map(|a| (a.serial, a.layer)).collect();
        let particles: BTreeSet<ObjectId> = self
            .world
            .gas
            .systems
            .values()
            .flat_map(|s| s.particles.iter().map(|p| p.id))
            .collect();
        self.layers.retain(|e| match e {
            Entrant::Anim(s) => anims.get(&s).is_some_and(|l| *l != Layer::Ground),
            Entrant::Particle(id) => particles.contains(&id),
            _ => true,
        });
        for (serial, layer) in anims {
            if layer != Layer::Ground && !self.layers.contains(Entrant::Anim(serial)) {
                // spec: S-0102 claims 4 and 7 (an animation placed in a layer other than ground becomes its last)
                self.layers.place(Entrant::Anim(serial), layer);
            }
        }
        for id in particles {
            if !self.layers.contains(Entrant::Particle(id)) {
                // spec: S-0616 claim 30 and S-0102 claim 3 (a particle enters the air layer when it is made, at its end, and keeps its place until removed)
                self.layers.place(Entrant::Particle(id), Layer::Air);
            }
        }
    }

    /// The buildings' slot animations after a tick (S-0108 claims 11, 12,
    /// 17 and 20; S-0606 claims 6, 12 and 16): each running one plays
    /// (S-0109), those that finished on their own leave their slot, then
    /// the building's update starts and stops slots and the walls' events
    /// of the tick apply. A building whose type has not been drawn yet
    /// waits for its slot definitions.
    fn update_slots(&mut self) {
        let now = self.shown_tick();
        let events = std::mem::take(&mut self.special_events);
        let w = &self.world;
        let mode = self.rules.mode;
        let present: Vec<(ObjectId, usize, slot_play::TypeKind, slot_play::State)> = w
            .objects
            .values()
            .filter(|o| o.kind == SimKind::Building && o.on_map)
            .filter_map(|o| {
                let idx = (*self.types.get(&(o.kind, o.type_index))?)?;
                let r = self.rules.buildings.get(o.type_index as usize)?;
                let extra = self
                    .rules
                    .econ
                    .power
                    .buildings
                    .get(o.type_index as usize)
                    .map_or(0, |p| p.extra_output);
                let kind = slot_play::TypeKind {
                    refinery: r.raw.econ.refinery.unwrap_or(false),
                    // spec: S-0108 claim 17 (absorbs infantry and gives a positive extra power per held infantry)
                    absorbs: r.absorb.infantry && extra > 0,
                    depot: r.building.unit_repair,
                    naval: r.naval,
                    delayed_fire: r.building.anim_delayed_fire,
                };
                let sizes: Vec<glasnost_num::F64> =
                    glasnost_sim::structures::absorb::passengers(w, o.id)
                        .iter()
                        .filter_map(|p| w.obj(*p))
                        .map(|p| {
                            w.rules()
                                .unit(p.kind, p.type_index)
                                // spec: S-0108 Import mapping (Size absent: 1)
                                .map_or_else(|| glasnost_num::F64::from_i64(1, mode), |t| t.size)
                        })
                        .collect();
                let in_repair = o.beh.current == Some(Behaviour::Repair);
                let st = slot_play::State {
                    // spec: S-0108 claims 11 and 17 (completed and not in the construction state)
                    completed: o.beh.current != Some(Behaviour::Construction),
                    // spec: S-0108 claim 11 (a building of the map was placed during the game's set-up)
                    setup: self.building_ids.values().any(|b| *b == o.id),
                    damaged: super::build::condition_of(&self.rules, o.health, r.strength).0,
                    // spec: S-0108 Terms (occupied: at least one occupant; held infantry are not occupants)
                    occupied: !o.structure.occupants.is_empty(),
                    // spec: S-0108 claim 17, F9 (the held infantry's total size)
                    held_size: slot_play::held_size(&sizes, mode),
                    in_repair,
                    // spec: S-0108 claim 20 and S-0314 claim 6 (repairing a docked unit: in its repair
                    // behaviour with a serviced vehicle)
                    repairing: in_repair && o.depot.serviced.is_some(),
                };
                Some((o.id, idx, kind, st))
            })
            .collect();
        self.slot_anims
            .retain(|id, _| present.iter().any(|(p, ..)| p == id));
        let mut env = AnimEnv {
            table: &self.anim_table,
            rules: &self.rules,
            rng: &mut self.cosmetic,
            speed: self.world.game_speed,
        };
        let q = AnimQuirks::default();
        for (id, idx, kind, st) in present {
            let Some(defs) = self.slot_defs.get(&idx) else {
                continue;
            };
            let slots = self.slot_anims.entry(id).or_default();
            let mut finished = Vec::new();
            for (slot, r) in &mut slots.running {
                let before = r.anim.player.name.clone();
                // spec: S-0109 claims 5 to 16 (each running slot animation plays; claim 16: it ends)
                if !r.anim.player.update(now, &mut env, &q) {
                    finished.push(*slot);
                } else if r.anim.player.name != before
                    && let Some(e) = env.lookup(&r.anim.player.name).0
                {
                    // spec: S-0109 claim 15 (the same animation continues as the follow-up type)
                    r.anim.entry = e;
                }
            }
            let mut mk = SlotMaker { env: &mut env, now };
            for slot in finished {
                slots.finished(slot, &kind, defs, &st, &mut mk);
            }
            slots.update(&kind, defs, &st, &mut mk);
            for (_, pair, start, damaged) in events.iter().filter(|(b, ..)| *b == id) {
                slots.walls(*pair, *start, *damaged, defs, &mut mk);
            }
        }
    }

    /// A building's running slot animations as the drawing takes them.
    fn slot_draws(&self, id: ObjectId, idx: usize) -> Vec<super::build::LiveSlot> {
        let (Some(slots), Some(defs)) = (self.slot_anims.get(&id), self.slot_defs.get(&idx)) else {
            return Vec::new();
        };
        slots
            .running
            .iter()
            .filter(|(_, r)| r.anim.entry.sprite)
            .filter_map(|(slot, r)| {
                Some(super::build::LiveSlot {
                    def: defs.get(slot)?.clone(),
                    idx: r.anim.entry.idx,
                    frame: r.anim.player.frame(),
                    stage: r.anim.player.stage(),
                    end: r.anim.player.play.end,
                })
            })
            .collect()
    }

    /// The names of a building's running slot animations, in slot order
    /// (tests and lanes).
    pub fn slot_names(&self, id: ObjectId) -> Vec<String> {
        self.slot_anims.get(&id).map_or_else(Vec::new, |s| {
            s.running
                .iter()
                .map(|(slot, r)| {
                    format!(
                        "{slot:?}={} frame {}",
                        String::from_utf8_lossy(&r.name),
                        r.anim.player.frame()
                    )
                })
                .collect()
        })
    }

    /// The attached animations' owners and positions (tests).
    #[cfg(test)]
    pub(super) fn attached(&self) -> Vec<(ObjectId, glasnost_sim::world::Pos)> {
        self.anims
            .iter()
            .filter_map(|a| a.owner.map(|o| (o, a.pos)))
            .collect()
    }

    /// The shot animations: firer (if attached), position, layer and depth
    /// adjustment (tests).
    #[cfg(test)]
    pub(super) fn shot_anims(
        &self,
    ) -> Vec<(
        Option<ObjectId>,
        glasnost_sim::world::Pos,
        super::slots::Layer,
        Option<i32>,
    )> {
        self.anims
            .iter()
            .filter(|a| a.shooter.is_some() || a.z_adjust.is_some())
            .map(|a| (a.shooter.map(|s| s.0), a.pos, a.layer, a.z_adjust))
            .collect()
    }

    /// Every playing animation's layer (tests).
    #[cfg(test)]
    pub(super) fn anim_layers(&self) -> Vec<super::slots::Layer> {
        self.anims.iter().map(|a| a.layer).collect()
    }

    /// Every playing animation's carrier house name and brightness (tests).
    #[cfg(test)]
    pub(super) fn anim_colours(&self) -> Vec<Option<(Vec<u8>, i32)>> {
        self.anims.iter().map(|a| a.carrier.clone()).collect()
    }

    /// The attached animations' owners and carrier house names (tests).
    #[cfg(test)]
    pub(super) fn carriers(&self) -> Vec<(ObjectId, Vec<u8>)> {
        self.anims
            .iter()
            .filter_map(|a| Some((a.owner?, a.carrier.as_ref()?.0.clone())))
            .collect()
    }

    /// Drops the computer houses' scripts: their houses are played
    /// elsewhere (the game client's `Game`).
    pub fn clear_scripts(&mut self) {
        self.scripts.clear();
    }

    /// The animations playing now, by type name and position, in creation
    /// order: a lane's way to name what an effect pixel shows.
    pub fn playing_anims(&self) -> Vec<(String, (i32, i32, i32))> {
        self.anims
            .iter()
            .map(|a| {
                (
                    String::from_utf8_lossy(&a.player.name).into_owned(),
                    (a.pos.x, a.pos.y, a.pos.z),
                )
            })
            .collect()
    }

    /// Draws our own debug markers for what has no art (`true`, the
    /// viewer's default) or only art (`false`, the game client).
    pub fn set_debug_markers(&mut self, on: bool) {
        self.debug_markers = on;
    }

    /// Sets the house whose eyes the cloak drawing uses (S-0516 claims 28
    /// and 29); `None` draws every object as its owner sees it.
    pub fn set_viewer(&mut self, viewer: Option<HouseId>) {
        self.viewer = viewer;
    }

    /// Applies how the viewer sees an object's cloak to its drawn items
    /// (S-0516 claims 28 and 29): not drawn, or blended 25 % or 50 %, and no
    /// shadow unless uncloaked. The rippling image has no warp effect here
    /// yet; it is drawn 50 % translucent (our placeholder, presentation only).
    /// A disguise is not drawn as its disguise yet (S-0516 claim 37).
    fn apply_cloak(&self, id: ObjectId, items: &mut Vec<Item>) {
        use glasnost_render::compose::{Paint, Translucency};
        use glasnost_sim::cloak::view::{self, Look};
        let w = &self.world;
        let v = self.viewer.or_else(|| w.obj(id).and_then(|o| o.owner));
        let blend = match view::look(w, id, v) {
            Look::Normal => None,
            // spec: S-0516 claim 28 (25 % translucent: S-0102 code 1)
            Look::Translucent25 => Some(Translucency::Quarter),
            // spec: S-0516 claim 28 (50 % translucent: S-0102 code 2)
            Look::Translucent50 | Look::Rippling => Some(Translucency::Half),
            // spec: S-0516 claim 28 (not drawn)
            Look::Hidden => {
                items.clear();
                return;
            }
        };
        // spec: S-0516 claim 29 (no shadow while not uncloaked)
        if !view::shadow(w, id) {
            items.retain(|i| i.paint != Paint::Shadow);
        }
        if let Some(t) = blend {
            for i in items.iter_mut() {
                if let Paint::Lit(s) = i.paint {
                    i.paint = Paint::Translucent(s, t);
                }
            }
        }
    }

    /// Shows a move-target marker (S-0214 claim 30): the imported
    /// move-feedback animation, unattached at `at`, with no loop delay and
    /// repeat factor 1. It is created between ticks, after the turn pass,
    /// so its first update is in the next tick (S-0109 claim 5). Drawing
    /// only: its draws come from the cosmetic stream (S-0214 claim 14).
    /// Without the animation's art the viewer shows its animation marker.
    pub fn move_marker(&mut self, at: glasnost_sim::world::Pos) {
        // spec: S-0214 Import mapping (rules [General] MoveFlash: the move-feedback animation)
        let name: Option<Vec<u8>> = self.move_flash.clone();
        // spec: S-0109 claim 5 (created after the turn pass: first update in the next tick)
        let next = self.world.tick;
        let drawn = name.is_some_and(|n| self.create_anim(&n, at, next, None, false));
        if !drawn && self.debug_markers {
            self.transients.push(Transient {
                kind: TransientKind::Anim,
                at: project_lifted(at.x, at.y, at.z),
                born: self.shown_tick(),
            });
        }
        self.sync_ground();
    }

    /// Starts a playing animation of type `name` at `pos` whose first
    /// update is in tick `first_update`, with the stage delay `drawn` the
    /// simulation's creation draw gave (S-0109 claims 1 to 5). Returns
    /// whether its type has a sprite.
    fn create_anim(
        &mut self,
        name: &[u8],
        pos: glasnost_sim::world::Pos,
        first_update: i32,
        drawn: Option<i32>,
        reverse: bool,
    ) -> bool {
        let speed = self.world.game_speed;
        let mut env = AnimEnv {
            table: &self.anim_table,
            rules: &self.rules,
            rng: &mut self.cosmetic,
            speed,
        };
        let (entry, mut play) = env.lookup(name);
        if reverse {
            // spec: S-0517 claim 6 (the deploying presentation of an undeploy plays in reverse, S-0109 claim 8)
            play.reverse = true;
        }
        let player = Player::new(
            name,
            play,
            first_update,
            drawn,
            &mut env,
            &AnimQuirks::default(),
        );
        let sprite = entry.as_ref().is_some_and(|e| e.sprite);
        // spec: S-0102 claim 3 (an unattached animation is in its type's layer, default air)
        let layer = entry
            .as_ref()
            .map_or(super::slots::Layer::Air, |e| e.t.layer);
        if sprite {
            self.counts.anims_with_art += 1;
        } else {
            self.counts.anims_without_sprite += 1;
        }
        self.anims.push(LiveAnim {
            serial: self.anim_serial,
            player,
            entry,
            pos,
            layer,
            first_update,
            owner: None,
            carrier: None,
            walker: None,
            drainer: None,
            maker: None,
            dominating: None,
            frame: None,
            shooter: None,
            z_adjust: None,
        });
        self.anim_serial = self.anim_serial.wrapping_add(1);
        sprite
    }

    /// The owner's house name of the vehicle at `pos` whose deploying
    /// presentation (S-0517 claims 4 and 6) is the animation `name`, if one
    /// is there: the animation is that unit's own deploy or undeploy
    /// animation (S-0103 claim 8 row 3).
    fn deploying_owner(&self, name: &str, pos: glasnost_sim::world::Pos) -> Option<Vec<u8>> {
        let w = &self.world;
        w.objects
            .values()
            .filter(|o| o.pos == pos && o.kind == SimKind::Vehicle)
            .find(|o| {
                self.rules
                    .unit(o.kind, o.type_index)
                    .and_then(|u| u.deploying_anim.as_deref())
                    .is_some_and(|d| d.eq_ignore_ascii_case(name))
            })
            .and_then(|o| o.owner)
            .and_then(|h| w.houses.get(usize::from(h.0)))
            .map(|h| h.name.as_bytes().to_vec())
    }

    /// A paratrooper's parachute (S-0522 claim 13): the parachute type
    /// created at the paratrooper's position and attached to it (S-0109
    /// claim 22), so it follows the unit down in the ground layer.
    fn attach_parachute(
        &mut self,
        trooper: ObjectId,
        pos: glasnost_sim::world::Pos,
        now: i32,
        drawn: Option<i32>,
    ) {
        let Some(name) = self.rules.superweapons.parachute.clone() else {
            return;
        };
        self.counts.anims += 1;
        let serial = self.anim_serial;
        self.create_anim(&name_bytes(&name), pos, now, drawn, false);
        if let Some(a) = self.anims.iter_mut().find(|a| a.serial == serial) {
            a.owner = Some(trooper);
            // spec: S-0109 claim 22 (an attached animation is in the ground render layer, S-0102)
            a.layer = super::slots::Layer::Ground;
            // spec: S-0522 claim 18 (the parachute is in the ground render layer and orders against the unit by S-0102 claims 5 and 6 with its sort adjustment; its palette is the carrier's house-colour unit palette, its brightness the unscaled ground light of the carrier's cell sampled now, at creation)
            let house = self
                .world
                .obj(trooper)
                .and_then(|o| o.owner)
                .and_then(|h| self.world.houses.get(usize::from(h.0)))
                .map(|h| h.name.as_bytes().to_vec());
            let light = self.painter.ground_light(pos.cell());
            a.carrier = house.map(|h| (h, light));
        }
    }

    /// The attached animations follow their owners (S-0109 claim 22, at
    /// the displacement 0 they were attached with); an owner that left the
    /// game takes its animation with it. A parachute whose paratrooper has
    /// landed plays its current pass as its last (S-0522 claim 13).
    fn follow_owners(&mut self) {
        let world = &self.world;
        self.anims.retain_mut(|a| {
            let Some(id) = a.owner else { return true };
            // spec: S-0109 claim 22 (an owner removed from the game: the animation disappears at once)
            let Some(o) = world.obj(id).filter(|o| o.on_map) else {
                return false;
            };
            // spec: S-0109 claim 22 (its position is the owner's plus its displacement at attachment, here 0)
            a.pos = o.pos;
            if !o.fall.chute {
                // spec: S-0522 claim 13 (still present on landing: the remaining passes become 0); the parachute stays attached
                a.player.last_pass();
            }
            true
        });
    }

    /// Shot animations follow their firers (S-0301 claim 22, S-0109 claim
    /// 22): the firer's position plus the displacement at creation, also
    /// for a passenger firing out, whose position is its transport's (S-0507
    /// claim 15). A firer removed from the game takes its animation with it.
    fn follow_shooters(&mut self) {
        let world = &self.world;
        self.anims.retain_mut(|a| {
            let Some((id, d)) = a.shooter else {
                return true;
            };
            // spec: S-0109 claim 22 (an owner removed from the game: the animation disappears at once); a passenger inside its transport is still in the game
            let Some(o) = world
                .obj(id)
                .filter(|o| o.on_map || glasnost_sim::transport::riding(world, id).is_some())
            else {
                return false;
            };
            // spec: S-0109 claim 22 (its position is the owner's plus its displacement at attachment, height included)
            a.pos = glasnost_sim::world::Pos::new(o.pos.x + d.x, o.pos.y + d.y, o.pos.z + d.z);
            true
        });
    }

    /// The viewer's teleport source marker (S-0513 claims 33 and 34): a
    /// source pick places the marker animation over the picked cell,
    /// replacing an earlier one, and the launch removes it. Presentation
    /// only, with cosmetic draws (the scenario draws of its creation are the
    /// simulation's, taken at the pick: S-0513 claim 33).
    fn source_marker_event(
        &mut self,
        event: glasnost_sim::superweapons::SwEvent,
        pos: Option<glasnost_sim::world::Pos>,
        now: i32,
    ) {
        use glasnost_sim::superweapons::SwEvent;
        match event {
            SwEvent::SourceMarker => {
                // spec: S-0513 claim 33 (the source marker over the cell, seen by the launching player only; a new pick replaces it)
                self.drop_source_marker();
                let name = self.rules.superweapons.chrono_placement.clone();
                if let (Some(name), Some(pos)) = (name, pos) {
                    let serial = self.anim_serial;
                    self.counts.anims += 1;
                    self.create_anim(&name_bytes(&name), pos, now, None, false);
                    self.source_marker = Some(serial);
                }
            }
            // spec: S-0513 claim 34 (the launch ends the source marker)
            SwEvent::MarkerEnd => self.drop_source_marker(),
            _ => {}
        }
    }

    /// Removes the viewer's source marker, if it still plays.
    fn drop_source_marker(&mut self) {
        if let Some(s) = self.source_marker.take() {
            self.anims.retain(|a| a.serial != s);
        }
    }

    /// Every playing animation's update of tick `now` (S-0109 claim 5: one
    /// update per tick from its first); ended ones leave.
    /// The burning walkers (S-0615 claims 4 to 11): each drawn animation
    /// takes its walker's position and image frame; one whose walker has
    /// been removed disappears.
    /// Drain animations follow their drainers (S-0608 claim 5) and go when
    /// the drainer is no longer draining (claim 14): a relinked drainer
    /// keeps no old animation (Edge cases, a display defect fixed).
    fn follow_drainers(&mut self) {
        let world = &self.world;
        let mut seen = std::collections::BTreeSet::new();
        // The newest animation of a drainer is the one of its newest link.
        for a in self.anims.iter_mut().rev() {
            if let Some(d) = a.drainer
                && !seen.insert(d)
            {
                a.drainer = Some(ObjectId(u32::MAX));
            }
        }
        self.anims.retain_mut(|a| {
            let Some(id) = a.drainer else { return true };
            // spec: S-0608 claim 14 (the drain animation is removed when the link ends)
            let Some(o) = world
                .obj(id)
                .filter(|_| glasnost_sim::drain::draining(world, id))
            else {
                return false;
            };
            a.pos = o.pos;
            true
        });
    }

    /// The infantry-making animations (S-0611 claims 8 to 12): each drawn
    /// animation shows its simulated animation's type and image frame, in
    /// its owner's colour (claim 8, also after claim 9's change of owner);
    /// one whose animation has ended disappears.
    fn follow_makers(&mut self) {
        let world = &self.world;
        let table = &self.anim_table;
        let mut colours = Vec::new();
        self.anims.retain_mut(|a| {
            let Some(id) = a.maker else { return true };
            let Some(m) = world.mutation.makers.get(&id) else {
                return false;
            };
            if a.player.name != m.player.name {
                a.player = m.player.clone();
                a.entry = table.get(&m.player.name.to_ascii_uppercase()).cloned();
            }
            a.frame = Some(m.player.frame());
            let house = m
                .owner
                .and_then(|h| world.houses.get(usize::from(h.0)))
                .map(|h| h.name.as_bytes().to_vec());
            if a.carrier.as_ref().map(|c| &c.0) != house.as_ref() {
                colours.push((a.serial, house, m.pos.cell()));
            }
            true
        });
        for (serial, house, cell) in colours {
            let light = self.painter.ground_light(cell);
            if let Some(a) = self.anims.iter_mut().find(|a| a.serial == serial) {
                // spec: S-0611 claim 8 (drawn in its owner's colour, including after claim 9's fallback; without an owner its ordinary palette)
                a.carrier = house.map(|h| (h, light));
            }
        }
    }

    /// The domination animations (S-0612 claims 1 and 8: the lead-in and
    /// the sweep animation): each simulated one is drawn with its type and
    /// image frame; one the simulation has removed disappears.
    fn follow_dominations(&mut self, now: i32) {
        let shown: std::collections::BTreeSet<ObjectId> =
            self.anims.iter().filter_map(|a| a.dominating).collect();
        let new: Vec<(ObjectId, Vec<u8>, glasnost_sim::world::Pos)> = self
            .world
            .domination
            .anims
            .iter()
            .filter(|(id, _)| !shown.contains(id))
            .map(|(&id, d)| (id, d.player.name.clone(), d.pos))
            .collect();
        for (id, name, pos) in new {
            self.counts.anims += 1;
            let serial = self.anim_serial;
            self.create_anim(&name, pos, now, None, false);
            if let Some(a) = self.anims.iter_mut().find(|a| a.serial == serial) {
                a.dominating = Some(id);
            }
        }
        let world = &self.world;
        let table = &self.anim_table;
        self.anims.retain_mut(|a| {
            let Some(id) = a.dominating else { return true };
            let Some(d) = world.domination.anims.get(&id) else {
                return false;
            };
            if a.player.name != d.player.name {
                a.player = d.player.clone();
                a.entry = table.get(&d.player.name.to_ascii_uppercase()).cloned();
            }
            a.frame = Some(d.player.frame());
            a.pos = d.pos;
            true
        });
    }

    fn follow_walkers(&mut self) {
        let world = &self.world;
        let rules = &self.rules;
        let table = rules.quirks.table_direction;
        self.anims.retain_mut(|a| {
            let Some(id) = a.walker else { return true };
            let Some(wk) = world.walkers.get(&id) else {
                return false;
            };
            a.pos = wk.pos;
            let r = rules
                .animations
                .get(wk.anim as usize)
                .map_or(0, |t| t.running_frames);
            // spec: S-0615 claim 4 and F1 (the running frame; claim 11: the expiring stage)
            a.frame = Some(glasnost_sim::walker::frame(wk, r, world.tick, table));
            true
        });
    }

    fn update_anims(&mut self, now: i32) {
        let speed = self.world.game_speed;
        let mut env = AnimEnv {
            table: &self.anim_table,
            rules: &self.rules,
            rng: &mut self.cosmetic,
            speed,
        };
        let q = AnimQuirks::default();
        self.anims.retain_mut(|a| {
            // spec: S-0615 claim 4 (a burning walker replaces S-0109's playback)
            if now < a.first_update
                || a.walker.is_some()
                || a.maker.is_some()
                || a.dominating.is_some()
            {
                return true;
            }
            let before = a.player.name.clone();
            let alive = a.player.update(now, &mut env, &q);
            if alive && a.player.name != before {
                // spec: S-0109 claim 15 (the same animation continues as the follow-up type)
                a.entry = env.lookup(&a.player.name).0;
            }
            alive
        });
    }

    /// Takes the tick's presentation events out of the game (they are not
    /// game state): animations start playing; shots, detonations and
    /// animations without art become short-lived markers.
    fn take_effects(&mut self) {
        let now = self.shown_tick();
        let at = |p: glasnost_sim::world::Pos| project_lifted(p.x, p.y, p.z);
        let effects = std::mem::take(&mut self.world.effects);
        for e in &effects {
            if let Effect::Superweapon {
                event, house, pos, ..
            } = e
                && house.is_some()
                && *house == self.viewer
            {
                self.source_marker_event(*event, *pos, now);
            }
            if let Effect::Superweapon {
                event: glasnost_sim::superweapons::SwEvent::Parachute { trooper, delay },
                pos: Some(pos),
                ..
            } = e
            {
                self.attach_parachute(*trooper, *pos, now, *delay);
            }
            if let Effect::ShotAnim {
                name,
                pos,
                delay,
                attached,
                z_adjust,
            } = e
            {
                self.counts.anims += 1;
                let serial = self.anim_serial;
                // spec: S-0301 claim 22 and S-0109 claim 29 (created in the firer's turn: first update in the creation tick; no loop delay, repeat factor 1, no reverse request)
                let sprite = self.create_anim(&name_bytes(name), *pos, now, *delay, false);
                if let Some(a) = self.anims.iter_mut().find(|a| a.serial == serial) {
                    if attached.is_some() {
                        // spec: S-0301 claim 22 and S-0102 claim 3 (attached to a firer that is not a building: in the ground layer, even for an aircraft in flight)
                        a.layer = super::slots::Layer::Ground;
                    }
                    a.shooter = *attached;
                    // spec: S-0301 claim 22 and F8 (a building's: its type's layer, with the creator's depth adjustment)
                    a.z_adjust = *z_adjust;
                    // spec: S-0301 claim 22 (Palette: no palette of its object; row 4 or 5 of S-0103 claim 8 by its own art flags, also when attached)
                    a.carrier = None;
                }
                // spec: S-0301 claim 23 (no default or placeholder picture: a type without art draws nothing; the viewer's debug layer marks it)
                if !sprite && self.debug_markers {
                    self.transients.push(Transient {
                        kind: TransientKind::Anim,
                        at: at(*pos),
                        born: now,
                    });
                }
            }
            if let Effect::Anim {
                name,
                pos,
                delay,
                reverse,
            } = e
            {
                self.counts.anims += 1;
                let serial = self.anim_serial;
                // spec: S-0109 claims 5 and 29 (created before or during the turn pass: first update
                // in the creation tick; the lost update of claim 29 needs playback in the simulation)
                let sprite = self.create_anim(&name_bytes(name), *pos, now, *delay, *reverse);
                if let Some(house) = self.deploying_owner(name, *pos)
                    && let Some(a) = self.anims.iter_mut().find(|a| a.serial == serial)
                {
                    // spec: S-0103 claim 8 row 3 (a unit's deploy or undeploy animation is drawn with
                    // its object: the owner's house-colour palette, at brightness 1000)
                    a.carrier = Some((house, NEUTRAL_LIGHT));
                }
                if !sprite && self.debug_markers {
                    self.transients.push(Transient {
                        kind: TransientKind::Anim,
                        at: at(*pos),
                        born: now,
                    });
                }
            }
        }
        for e in &effects {
            if let Effect::Walker {
                id,
                name,
                pos,
                made: true,
            } = e
            {
                self.counts.anims += 1;
                let serial = self.anim_serial;
                self.create_anim(&name_bytes(name), *pos, now, None, false);
                if let Some(a) = self.anims.iter_mut().find(|a| a.serial == serial) {
                    a.walker = Some(*id);
                }
            }
        }
        for e in &effects {
            if let Effect::Maker {
                id,
                name,
                pos,
                event: glasnost_sim::mutation::MakerEvent::Made,
                ..
            } = e
            {
                self.counts.anims += 1;
                let serial = self.anim_serial;
                self.create_anim(&name_bytes(name), *pos, now, None, false);
                if let Some(a) = self.anims.iter_mut().find(|a| a.serial == serial) {
                    a.maker = Some(*id);
                }
            }
        }
        for e in &effects {
            if let Effect::Drain {
                drainer,
                event:
                    glasnost_sim::drain::DrainEvent::Formed {
                        anim: Some(name),
                        delay,
                    },
                ..
            } = e
                && let Some(o) = self.world.obj(*drainer)
            {
                // spec: S-0608 claim 5 (the drain animation starts attached to the drainer and follows it while it remains alive)
                self.counts.anims += 1;
                let serial = self.anim_serial;
                let pos = o.pos;
                self.create_anim(&name_bytes(name), pos, now, *delay, false);
                if let Some(a) = self.anims.iter_mut().find(|a| a.serial == serial) {
                    a.drainer = Some(*drainer);
                }
            }
        }
        self.follow_owners();
        self.follow_shooters();
        self.follow_walkers();
        self.follow_makers();
        self.follow_dominations(now);
        self.follow_drainers();
        self.update_anims(now);
        for e in effects {
            if let Effect::SupportBeam {
                from,
                to,
                duration,
                house,
            } = e
            {
                // spec: S-0518 claim 6 (the beam lasts L_s ticks, is 3 pixels thick and has the house's laser colour)
                const SUPPORT_BEAM_WIDTH: i32 = 3;
                // The house's laser colour is not defined by any spec (Q-1521 question 1): the
                // house display RGB of S-0103 claim 15 stands in for it.
                let colour = self.house_colour(house);
                self.beams.push(combat::Beam {
                    a: at(from),
                    b: at(to),
                    born: now,
                    life: duration,
                    width: SUPPORT_BEAM_WIDTH,
                    inner: colour,
                    outer: None,
                    fade: combat::Fade::None,
                });
                continue;
            }
            if let Effect::Beam {
                from,
                to,
                duration,
                fade_out,
                colours,
                house,
            } = e
            {
                // spec: S-0615 claims 15 and 16 (inner, outer and spread colours; with the house-colour
                // flag the inner is the house laser colour and the outer that colour halved, toward zero)
                let channel = |c: [i32; 3]| c.map(|v| v.clamp(0, 255) as u8); // nocite: 8-bit colour channels
                let [inner, outer, spread] = colours;
                let (inner, outer) = match house {
                    Some(_) => {
                        let h = self.house_colour(house).map(i32::from);
                        let [i, o, _] = glasnost_sim::disc::house_colours(h, spread);
                        (channel(i), channel(o))
                    }
                    None => (channel(inner), channel(outer)),
                };
                self.beams.push(combat::Beam {
                    a: at(from),
                    b: at(to),
                    born: now,
                    life: duration,
                    // Q-1521 question 2: a one-pixel core with a one-pixel edge (our interim rule).
                    width: 1,
                    inner,
                    outer: Some(outer),
                    fade: if fade_out {
                        combat::Fade::ToNone
                    } else {
                        combat::Fade::ToHalf
                    },
                });
                continue;
            }
            let (kind, pos) = match e {
                Effect::Shot { from, .. } => {
                    self.counts.shots += 1;
                    // spec: S-0301 claim 23 (the shot's own picture is its shot animation, Effect::ShotAnim; no
                    // placeholder at the launch point: only the viewer's debug layer marks the shot)
                    if !self.debug_markers {
                        continue;
                    }
                    (TransientKind::Muzzle, from)
                }
                Effect::Detonation { pos, radius, .. } => {
                    self.counts.detonations += 1;
                    // The game client draws only the detonation's animation art (S-0303 claim 21);
                    // the viewer's debug layer marks every detonation.
                    if !self.debug_markers {
                        continue;
                    }
                    (TransientKind::Burst(combat::burst_half_width(radius)), pos)
                }
                Effect::Anim { .. } | Effect::ShotAnim { .. } => continue,
                Effect::Killed { .. } => {
                    self.counts.kills += 1;
                    continue;
                }
                Effect::GasHit { .. } => {
                    self.counts.gas_hits += 1;
                    continue;
                }
                Effect::SpecialAnims {
                    building,
                    slots,
                    start,
                    damaged,
                } => {
                    // spec: S-0606 claims 6, 12 and 16 (the walls' special slots start or stop)
                    self.special_events.push((building, slots, start, damaged));
                    continue;
                }
                Effect::Sound { .. }
                | Effect::Mind { .. }
                | Effect::Consumed { .. }
                | Effect::Spin { .. }
                | Effect::Trigger { .. }
                | Effect::Hit { .. }
                | Effect::OverlayCrushed { .. }
                | Effect::Superweapon { .. }
                | Effect::SupportBeam { .. }
                | Effect::Captured { .. }
                | Effect::Infiltrated { .. }
                | Effect::Cloak { .. }
                | Effect::Bunkered { .. }
                | Effect::Seized { .. }
                | Effect::Deactivated { .. }
                | Effect::Dropped { .. }
                | Effect::Disc { .. }
                | Effect::Beam { .. }
                | Effect::Drain { .. }
                | Effect::Maker { .. }
                | Effect::Domination { .. }
                | Effect::Walker { .. } => continue,
            };
            self.transients.push(Transient {
                kind,
                at: at(pos),
                born: now,
            });
        }
        self.transients.retain(|t| t.marker(now).is_some());
        self.beams.retain(|b| b.shown(now));
        self.counts.most_in_flight = self.counts.most_in_flight.max(self.world.projectiles.len());
        self.counts.most_gas = self
            .counts
            .most_gas
            .max(glasnost_sim::gas::particles(&self.world).count());
    }

    /// Brings the ground layer's members and keys up to date with the game
    /// and orders it for the next drawn frame (S-0102 claim 6).
    fn sync_ground(&mut self) {
        let mut members: Vec<(Entrant, i64)> = self
            .ground
            .list
            .iter()
            .filter(|(e, _)| matches!(e, Entrant::Static(_)))
            .copied()
            .collect();
        members.extend(
            self.world
                .objects
                .values()
                .filter(|o| {
                    o.kind.is_mobile()
                        && o.on_map
                        && self.layers.of(o.id) == Some(super::slots::Layer::Ground)
                })
                .map(|o| (Entrant::Object(o.id), mobile_key(o))),
        );
        // spec: S-0102 claims 3 and 5 (a ground-layer animation's key: X + Y plus its sort adjustment)
        members.extend(
            self.anims
                .iter()
                .filter(|a| a.layer == super::slots::Layer::Ground)
                .map(|a| {
                    let adjust = a.entry.as_ref().map_or(0, |e| e.t.y_sort_adjust);
                    (
                        Entrant::Anim(a.serial),
                        i64::from(a.pos.x) + i64::from(a.pos.y) + i64::from(adjust),
                    )
                }),
        );
        self.ground.update(&members);
    }

    /// The unloading state of building `o`, while it unloads a unit
    /// (S-0108 claims 22 and 23).
    pub(super) fn unloading_of(&self, o: &Object) -> Option<Unloading> {
        // spec: S-0417 Terms (unloading: K's current behaviour is unload; a queued one is not yet)
        if o.beh.current != Some(Behaviour::Unload) {
            return None;
        }
        // spec: S-0417 claim 2 (the unit being released is the one in contact with K); S-0108
        // claim 22, Import mapping (JumpJet, BalloonHover of the unloaded unit's type, absent false;
        // no unit in contact counts as unmarked)
        let roof = self
            .world
            .objects
            .values()
            .find(|u| u.contact == Some(o.id))
            .and_then(|u| self.rules.unit(u.kind, u.type_index))
            .is_some_and(|t| t.jumpjet || t.raw.balloon_hover.unwrap_or(false));
        Some(Unloading {
            roof,
            door: unload::door_view(o.beh.gate.door, self.shown_tick()),
        })
    }

    /// The tick whose updates the picture shows: the last one that ran
    /// (S-0201 claim 6).
    fn shown_tick(&self) -> i32 {
        (self.world.tick - 1).max(0)
    }

    /// The mobile objects that take part in the object pass's body drawing,
    /// in their current drawing order (S-0102 claims 4 to 7, S-0214 claims
    /// 24 and 25): the simulation objects of the five layers that are
    /// drawn, layer by layer in drawing order.
    pub fn drawn_mobiles(&self, inputs: &Inputs) -> Vec<ObjectId> {
        self.mobile_order()
            .into_iter()
            .filter(|&id| self.mobile(inputs, id).is_some())
            .collect()
    }

    /// The mobile objects in drawing order (S-0102 claims 1, 6 and 7):
    /// the underground and surface lists, the ground layer, then the air
    /// and top lists.
    pub(super) fn mobile_order(&self) -> Vec<ObjectId> {
        use super::slots::Layer;
        let upper = |which: &dyn Fn(Layer) -> bool| -> Vec<ObjectId> {
            self.layers
                .upper()
                .filter(|l| which(*l))
                .flat_map(|l| self.layers.list(l).iter())
                .filter_map(|e| match e {
                    Entrant::Object(id) => Some(*id),
                    _ => None,
                })
                .collect()
        };
        let mut v = upper(&|l| l < Layer::Ground);
        v.extend(self.ground.objects());
        v.extend(upper(&|l| l > Layer::Ground));
        v
    }

    /// What one simulation object looks like now, if it is drawn.
    pub fn mobile(&self, inputs: &Inputs, id: ObjectId) -> Option<Mobile> {
        let w = &self.world;
        let o = w.obj(id)?;
        if !o.kind.is_mobile() || !o.on_map {
            return None;
        }
        // spec: S-0517 claim 12 (while a vehicle is in transition its own body is not drawn: the deploying presentation stands in for it)
        if glasnost_sim::econ::inplace::body_hidden(w, id) {
            return None;
        }
        let kind = import_kind(o.kind)?;
        // spec: S-0517 claim 12 (while deployed, the image of the deployed image type when the type names one; the vehicle keeps its own type for every rule)
        let drawn_type = glasnost_sim::econ::inplace::deployed_image(w, id).unwrap_or(o.type_index);
        let idx = (*self.types.get(&(o.kind, drawn_type))?)?;
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
            // spec: S-0105 claim 16 and S-0609 claim 14 (the animation-frame counter as the simulation keeps it; the bit pattern, taken modulo the frame count)
            anim: o.combat.anim_frame as u32,
            infantry,
            object: sim_code(id),
            // spec: S-0402 claim 5a, F8 (ν: the walk-animation counter of S-0105 claim 16). The
            // simulation does not model that counter yet, so 0, as for the body's HVA frame.
            harvesting: super::harvesting::of(w, id, 0),
            shadow: (kind == Kind::Aircraft).then(|| shadow_anchor(w, o)),
            // spec: S-0105 claim 12 (alternative body: a spawner none of whose spawned aircraft is docked)
            alternative: glasnost_sim::spawn::any_docked(w, id) == Some(false),
            attitude: attitude(w, o),
            // spec: S-0104 claim 13 (Deactivated voxel vehicles) and S-0405 claims 13 and 20
            disabled: kind == Kind::Vehicles && glasnost_sim::econ::power::unit_disabled(w, id),
            // spec: S-0107 claim 19 and Terms (height above the surface; 0 on a bridge deck)
            above: w.height_above_surface(id),
            // spec: S-0513 claim 29 (the envelope's phase as the last tick's turns left it; F7 in that tick)
            protection: super::protection::Look::of(o, self.shown_tick()),
        })
    }

    /// Rebuilds the object pass of `scene`: the cell pass stays, the static
    /// ground entries are reused and the mobile objects are drawn from the
    /// game, all in ground-layer order.
    pub fn redraw(&mut self, inputs: &Inputs, scene: &mut Scene) {
        self.learn_types(inputs);
        self.sync_layers(false);
        if !self.view_quirks.unsorted_upper_layers {
            // spec: S-0102 Edge cases (quirks.unsorted_upper_layers off: the other layers are sorted like the ground layer, by the keys of claim 5)
            let keys = self.upper_keys();
            self.layers
                .sort_by_key(|e| keys.get(&e).copied().unwrap_or(i64::MIN));
        }
        let ids: Vec<ObjectId> = self.mobile_order();
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
        for &id in &ids {
            if let Some(e) = by_code.get_mut(&sim_code(id)) {
                self.apply_cloak(id, &mut e.items);
            }
        }
        // Buildings the game removed leave the cell pass and the ground
        // layer; so do the map's gates, which are drawn live.
        let mut gone: BTreeSet<u32> = self
            .building_ids
            .iter()
            .filter(|(_, id)| self.world.obj(**id).is_none_or(|o| !o.on_map))
            .map(|(i, _)| *i)
            .collect();
        gone.extend(self.live_gates.iter().copied());
        // The map's buildings while protected are drawn live too, so that
        // their bodies show the protection appearance (S-0102 claim 34; our
        // placeholder order, as for the gates).
        let shown = self.shown_tick();
        let protected_map: BTreeSet<u32> = self
            .building_ids
            .iter()
            .filter(|(_, id)| {
                self.world
                    .obj(**id)
                    .is_some_and(|o| o.on_map && super::protection::Look::of(o, shown).is_some())
            })
            .map(|(code, _)| *code)
            .collect();
        gone.extend(protected_map.iter().copied());
        // Overlays that changed during play are drawn again from the game's
        // cells (S-0112 claims 1 to 9); their static items leave the cell pass.
        let mut changed = self.changed_overlays.clone();
        for (c, cell) in self.world.cells.grid.iter() {
            let now = (cell.overlay, cell.overlay_data);
            if changed.contains_key(&c) || self.start_overlays.get(&c) != Some(&now) {
                changed.insert(c, now);
            }
        }
        if changed != self.changed_overlays {
            let cells: Vec<(CellXY, glasnost_map::Cell)> = changed
                .keys()
                .filter_map(|&c| Some((c, *self.world.cells.grid.get(c)?)))
                .collect();
            let (items, notes) = self.painter.live_overlays(inputs, scene, cells);
            self.notes.extend(notes);
            self.live_overlays = items;
            self.changed_overlays = changed;
        }
        for c in self.changed_overlays.keys() {
            if let Some(codes) = self.overlay_codes.get(c) {
                gone.extend(codes.iter().copied());
            }
        }
        // The game's lighting changed (a fade step or a re-evaluation,
        // S-0104 claims 4 and 5; the psychic domination's and the storm's
        // lighting among them): the cells and the terrain are lit again.
        let l = &self.world.lighting;
        let key = (l.current, l.choice);
        // spec: S-0525 claim 13 (each field is a light source; every change of it relights the cells it reaches)
        let fields = glasnost_sim::radiation::lights(&self.world)
            .iter()
            .map(|l| {
                glasnost_render::light::LightSource::field(
                    (l.centre.x, l.centre.y),
                    l.radius,
                    l.intensity,
                    l.tint,
                )
            })
            .collect();
        let fields_changed = self.painter.set_field_lights(fields);
        let relit = key != self.lit || fields_changed;
        if relit {
            let set =
                glasnost_sim::lighting::set_of(l, key.1).unwrap_or(inputs.map.lighting.normal);
            self.painter
                .relight(inputs, scene, &mut self.cell_pass, key.0, set);
            self.lit = key;
        }
        if gone != self.gone || relit {
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
        // The changed overlays follow the cell pass (our placeholder place
        // for them: the depth test orders them against the rest).
        scene.items.extend_from_slice(&self.live_overlays);
        // The game's own buildings (deployed or placed) and the map's gates
        // follow, in object ID order (our placeholder order; the map's
        // other buildings keep their place in the cell pass).
        // Each drawn type's slot definitions, read once (S-0108 Import mapping).
        let drawn_types: Vec<usize> = self
            .world
            .objects
            .values()
            .filter(|o| o.kind == SimKind::Building && o.on_map)
            .filter_map(|o| *self.types.get(&(o.kind, o.type_index))?)
            .collect();
        for idx in drawn_types {
            if !self.slot_defs.contains_key(&idx) {
                let defs = self.painter.slot_defs(inputs, idx);
                self.slot_defs.insert(idx, defs);
            }
        }
        let own: Vec<(ObjectId, LiveBuilding)> = self
            .world
            .objects
            .values()
            .filter(|o| {
                o.kind == SimKind::Building
                    && o.on_map
                    && !self.building_ids.iter().any(|(code, id)| {
                        *id == o.id
                            && !self.live_gates.contains(code)
                            && !protected_map.contains(code)
                    })
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
                Some((
                    o.id,
                    LiveBuilding {
                        idx,
                        cell: o.pos.cell(),
                        owner,
                        health: (o.health, strength),
                        facing: o.turret.value(self.world.tick),
                        // spec: S-0105 claim 16 and S-0609 claim 14 (the animation-frame counter as the simulation keeps it)
                        anim: o.combat.anim_frame as u32,
                        // spec: S-0108 Terms and claim 2 (occupied: at least one occupant)
                        occupied: !o.structure.occupants.is_empty(),
                        object: sim_code(o.id),
                        unloading: self.unloading_of(o),
                        // spec: S-0108 claim 4 (a gate's door, S-0215 claim 35: its special image only while the current behaviour is open; otherwise the stationary rows)
                        slots: self.slot_draws(o.id, idx),
                        gate_door: (self
                            .rules
                            .buildings
                            .get(o.type_index as usize)
                            .is_some_and(|b| b.gate)
                            && o.beh.current == Some(Behaviour::Open))
                        .then(|| unload::door_view(o.beh.gate.door, self.shown_tick())),
                        // spec: S-0513 claim 29 (the envelope's phase as the last tick's turns left it; F7 in that tick)
                        protection: super::protection::Look::of(o, self.shown_tick()),
                    },
                ))
            })
            .collect();
        if !own.is_empty() {
            let (ids, own): (Vec<ObjectId>, Vec<LiveBuilding>) = own.into_iter().unzip();
            let (drawn, notes) = self.painter.buildings(inputs, scene, &own);
            self.notes.extend(notes);
            for (id, mut items) in ids.into_iter().zip(drawn) {
                self.apply_cloak(id, &mut items);
                scene.items.extend(items);
            }
        }
        // The playing animations with a sprite (S-0109), by serial number.
        let playing: Vec<(u32, super::slots::Layer, AnimDraw)> = self
            .anims
            .iter()
            .filter_map(|a| {
                let e = a.entry.as_ref().filter(|e| e.sprite)?;
                Some((
                    a.serial,
                    a.layer,
                    AnimDraw {
                        idx: e.idx,
                        frame: a.frame.unwrap_or_else(|| a.player.frame()),
                        stage: a.player.stage(),
                        end: a.player.play.end,
                        pos: (a.pos.x, a.pos.y, a.pos.z),
                        carrier: a.carrier.clone(),
                        z_adjust: a.z_adjust,
                    },
                ))
            })
            .collect();
        let list: Vec<AnimDraw> = playing.iter().map(|(_, _, d)| d.clone()).collect();
        let (drawn, notes) = self.painter.anims(inputs, scene, &list);
        self.notes.extend(notes);
        let mut anim_items: BTreeMap<u32, (super::slots::Layer, Vec<Item>)> = BTreeMap::new();
        for ((serial, layer, _), d) in playing.iter().zip(drawn) {
            if let Some((_, _, items)) = d
                && !items.is_empty()
            {
                anim_items.insert(*serial, (*layer, items));
            }
        }
        self.counts.anims_drawn = anim_items.len();
        self.counts.most_anims_drawn = self.counts.most_anims_drawn.max(anim_items.len());
        let particle_items = self.particle_items(inputs, scene);
        // spec: S-0102 claims 1 and 7 (underground and surface before the ground layer, air and
        // top after it; each unsorted)
        self.draw_layers(scene, &anim_items, &particle_items, &mut by_code, |l| {
            l < super::slots::Layer::Ground
        });
        for e in self.ground.order() {
            match e {
                Entrant::Anim(serial) => {
                    if let Some((_, items)) = anim_items.get(&serial) {
                        scene.items.extend_from_slice(items);
                    }
                }
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
                Entrant::Layered(_) | Entrant::Particle(_) => {}
            }
        }
        self.draw_layers(scene, &anim_items, &particle_items, &mut by_code, |l| {
            l > super::slots::Layer::Ground
        });
        self.objects_end = scene.items.len();
        self.draw_effects(scene);
        self.draw_status(inputs, scene);
    }

    /// The items of every particle with a sprite (S-0613, S-0616; the
    /// interim rule of Q-1520), by system and then in each system's
    /// particle order (Q-1520 question 7 is open).
    fn particle_items(
        &mut self,
        inputs: &Inputs,
        scene: &mut Scene,
    ) -> BTreeMap<ObjectId, Vec<Item>> {
        let detail = inputs.options.detail;
        let mut list = Vec::new();
        let mut ids = Vec::new();
        for s in self.world.gas.systems.values() {
            for p in &s.particles {
                let Some(t) = self.rules.gas.particles.get(p.type_index as usize) else {
                    continue;
                };
                let Some(Some((e, true))) = self.particle_table.get(p.type_index as usize) else {
                    continue;
                };
                ids.push(p.id);
                list.push(super::build::ParticleDraw {
                    idx: e.idx,
                    frame: super::particles::frame(t.kind, p.stage, e.start_frame),
                    // spec: S-0613 claim 16 (from the type's translucency only; Q-1520 interim rule)
                    code: super::particles::translucency(t.translucency, detail),
                    pos: (p.pos.x, p.pos.y, p.pos.z),
                });
            }
        }
        let (drawn, notes) = self.painter.particles(inputs, scene, &list);
        self.notes.extend(notes);
        self.counts.particles_drawn = drawn.iter().filter(|i| !i.is_empty()).count();
        ids.into_iter().zip(drawn).collect()
    }

    /// The sort keys of the entrants of the layers other than ground
    /// (S-0102 claim 5), for `quirks.unsorted_upper_layers` off.
    fn upper_keys(&self) -> BTreeMap<Entrant, i64> {
        let mut keys = BTreeMap::new();
        for o in self.world.objects.values() {
            if o.kind.is_mobile() {
                keys.insert(Entrant::Object(o.id), mobile_key(o));
            }
        }
        for a in &self.anims {
            let adjust = a.entry.as_ref().map_or(0, |e| e.t.y_sort_adjust);
            keys.insert(
                Entrant::Anim(a.serial),
                i64::from(a.pos.x) + i64::from(a.pos.y) + i64::from(adjust),
            );
        }
        for s in self.world.gas.systems.values() {
            for p in &s.particles {
                keys.insert(
                    Entrant::Particle(p.id),
                    i64::from(p.pos.x) + i64::from(p.pos.y),
                );
            }
        }
        for (i, (_, g)) in self.static_layered.iter().enumerate() {
            keys.insert(Entrant::Layered(i), g.key);
        }
        keys
    }

    /// Draws the layers other than ground that `which` picks, layer by
    /// layer in drawing order, each in its list's order (S-0102 claims 1,
    /// 3, 4 and 7; S-0616 claim 30): the static entries, the playing
    /// animations (`anims`, by serial number), the particles (`particles`)
    /// and the mobile objects (`mobiles`, taken out as drawn).
    fn draw_layers(
        &self,
        scene: &mut Scene,
        anims: &BTreeMap<u32, (super::slots::Layer, Vec<Item>)>,
        particles: &BTreeMap<ObjectId, Vec<Item>>,
        mobiles: &mut BTreeMap<u32, GroundEntry>,
        which: impl Fn(super::slots::Layer) -> bool,
    ) {
        let layers: Vec<super::slots::Layer> = self.layers.upper().filter(|l| which(*l)).collect();
        for layer in layers {
            for e in self.layers.list(layer) {
                match *e {
                    Entrant::Layered(i) => {
                        if let Some((_, g)) = self.static_layered.get(i)
                            && g.object.is_none_or(|o| !self.gone.contains(&o))
                        {
                            scene.items.extend_from_slice(&g.items);
                        }
                    }
                    Entrant::Anim(serial) => {
                        if let Some((_, items)) = anims.get(&serial) {
                            scene.items.extend_from_slice(items);
                        }
                    }
                    Entrant::Particle(id) => {
                        if let Some(items) = particles.get(&id) {
                            scene.items.extend_from_slice(items);
                        }
                    }
                    Entrant::Object(id) => {
                        if let Some(g) = mobiles.remove(&sim_code(id)) {
                            scene.items.extend(g.items);
                        }
                    }
                    Entrant::Static(_) => {}
                }
            }
        }
    }

    /// The display RGB of house `house` (S-0103 claim 15), white for no
    /// house (our choice).
    fn house_colour(&self, house: Option<HouseId>) -> [u8; 3] {
        house
            .and_then(|h| self.house_rgb.get(h.0 as usize))
            .copied()
            .unwrap_or([255, 255, 255]) // nocite: white, our stand-in for no house
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
        // S-0613, S-0616: the game view draws particles with their sprites (Q-1520 interim
        // rule); the debug-markers view marks a particle whose type has no sprite, a placeholder
        // by its system's kind (our own pictures).
        if self.debug_markers {
            for s in self.world.gas.systems.values() {
                use glasnost_sim::gas::Behaviour;
                let m = match rules
                    .gas
                    .systems
                    .get(s.type_index as usize)
                    .and_then(|t| t.kind)
                {
                    Some(Behaviour::Smoke) => Marker::Smoke,
                    Some(Behaviour::Fire) => Marker::Fire,
                    Some(Behaviour::Spark) => Marker::Spark,
                    Some(Behaviour::Railgun) => Marker::Rail,
                    Some(Behaviour::Gas) | None => Marker::Gas,
                };
                for p in &s.particles {
                    let sprite = matches!(
                        self.particle_table.get(p.type_index as usize),
                        Some(Some((_, true)))
                    );
                    if sprite {
                        continue;
                    }
                    // spec: S-0101 F1, F2 (the projected point of the position with its height)
                    let at = project_lifted(p.pos.x, p.pos.y, p.pos.z);
                    let item = self.cache.marker_item(scene, m, at);
                    scene.items.push(item);
                }
            }
        }
        // S-0525: the reach of every radiation field, as a ring, in the viewer's debug layer only (the
        // game client draws the field's light, claim 13).
        for f in &self.world.radiation.fields {
            if f.spread < 0 || !self.debug_markers {
                continue;
            }
            let p = glasnost_sim::world::Pos::centre(f.centre, 0);
            let z = self.world.surface_at(p.x, p.y);
            let m = Marker::Field {
                half: combat::burst_half_width(f.reach),
            };
            let item = self
                .cache
                .marker_item(scene, m, project_lifted(p.x, p.y, z));
            scene.items.push(item);
        }
        // S-0520: every crate place, a box on its cell, in the viewer's debug layer only (the game
        // client draws the crate as its cell's overlay, S-0520 Terms, S-0112).
        for place in self
            .world
            .crates
            .places
            .values()
            .filter(|_| self.debug_markers)
        {
            let p = glasnost_sim::world::Pos::centre(place.cell, 0);
            let z = self.world.surface_at(p.x, p.y);
            let item =
                self.cache
                    .marker_item(scene, Marker::Crate, project_lifted(p.x, p.y, z + 40));
            scene.items.push(item);
        }
        // S-0519: planted bombs the viewer sees, charges counting down and buildings marked by a strike,
        // in the viewer's debug layer only: how the game draws them is open (Q-1524).
        let viewer = self.viewer;
        let mut extra: Vec<(Marker, (i32, i32))> = Vec::new();
        let debug = self.debug_markers;
        for o in self.world.objects.values().filter(|o| o.on_map && debug) {
            let z = self.world.surface_at(o.pos.x, o.pos.y);
            let at = project_lifted(o.pos.x, o.pos.y, z + 200);
            if let Some(b) = o.combat.bomb
                && viewer.is_none_or(|v| glasnost_sim::bomb::visible(&self.world, v, o.id))
            {
                let g = &self.rules.combat.general.ivan;
                let q = (g.delay / 6).max(1); // spec: S-0519 F3 (q = floor(D / 6); 0 becomes 1)
                let big_f = g.flicker.max(1);
                let n = self.world.tick;
                let bit = i32::from(n.rem_euclid(2 * big_f) >= big_f);
                let frame = (2 * ((n - b.planted) / q) + bit).clamp(0, 11); // spec: S-0519 F3 (frame = min(11, 2 floor((n - t) / q) + b))
                extra.push((Marker::Bomb { frame }, at));
            }
            if o.combat.charge_planter.is_some() && o.combat.delay_kill_until.is_some() {
                extra.push((Marker::Charge, at));
            }
            if glasnost_sim::strike::marked(&self.world, o.id) {
                extra.push((Marker::Marked, project_lifted(o.pos.x, o.pos.y, z)));
            }
        }
        for (m, at) in extra {
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
        // spec: S-0102 claim 2 (beam effects after all layers)
        for b in &self.beams {
            if b.shown(now) {
                let item = self.cache.beam_item(scene, b, now);
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

    /// Debug diagnostic: one line for every drawn item that covers world
    /// pixel `(x, y)` (`top` is the view's top world row), in draw order:
    /// the item's place in the scene, group, paint, image size, the object
    /// it belongs to (kind, type, owner, cell, rocket phase) and the draw
    /// path that produced it. Presentation only; nothing reads it.
    pub fn trace_point(
        &self,
        scene: &Scene,
        statics: &[super::build::ObjectInfo],
        x: i32,
        y: i32,
        top: i32,
    ) -> Vec<String> {
        use glasnost_render::compose::Paint;
        scene
            .items_at(x, y, top)
            .into_iter()
            .map(|i| {
                let it = &scene.items[i];
                let img = &scene.images[it.image as usize];
                let paint = match it.paint {
                    Paint::Shadow => "shadow",
                    Paint::Row(_) => "row",
                    Paint::Diamond(_) => "diamond",
                    Paint::Lit(_) => "lit",
                    Paint::LitDiamond(_) => "lit-diamond",
                    Paint::Translucent(..) => "translucent",
                    #[allow(unreachable_patterns)]
                    _ => "other",
                };
                let what = match it.object {
                    Some(code) => match self.object_of_code(code).and_then(|id| self.world.obj(id))
                    {
                        Some(o) => format!(
                            "object #{} {:?} {} owner {:?} cell {:?} on_map {} rocket {:?}",
                            o.id.0,
                            o.kind,
                            type_name(&self.rules, o.kind, o.type_index),
                            o.owner.map(|h| h.0),
                            o.pos.cell(),
                            o.on_map,
                            o.rocket.as_ref().map(|r| r.phase),
                        ),
                        None => match statics.get(code as usize) {
                            Some(s) => format!(
                                "static {} {} owner {} cell {:?} ({})",
                                s.kind, s.type_name, s.owner, s.cell, s.entry
                            ),
                            None => format!("static object code {code:#x}"),
                        },
                    },
                    None => "no object".to_string(),
                };
                let path = match (it.object, it.group, it.paint == Paint::Shadow) {
                    (_, glasnost_render::scene::Group::Effects, _) => {
                        "effect or projectile placeholder"
                    }
                    (Some(c), _, true) if c & SIM_OBJECT != 0 => "object shadow",
                    (Some(c), _, false) if c & SIM_OBJECT != 0 => "object body, turret or part",
                    (Some(_), _, _) => "static building draw (body, slot animation or bib)",
                    (None, _, _) => "unattached animation or overlay",
                };
                format!(
                    "item {i} {:?} {paint} {}x{} at ({}, {}) depth {:?}: {what}; path: {path}",
                    it.group,
                    img.width(),
                    img.height(),
                    it.x,
                    it.y,
                    it.depth
                )
            })
            .collect()
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
