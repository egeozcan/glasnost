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
use super::beams;
use super::build::{
    AnimDraw, Attitude, Built, GameMode, GroundEntry, InfantryFrame, Inputs, LiveBuilding, Mobile,
    Painter, ShadowAnchor, find_type, rule_int,
};
use super::combat::{self, Cache, Counts, Marker, Ratios, Transient, TransientKind};
use super::pip_scale;
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

/// Two translucency codes combined (S-0109 Terms): the larger of the two,
/// except that 1 and 2 combine to 3.
pub fn combine_codes(
    a: Option<glasnost_render::compose::Translucency>,
    b: Option<glasnost_render::compose::Translucency>,
) -> Option<glasnost_render::compose::Translucency> {
    use glasnost_render::compose::Translucency;
    let code = |t: Option<Translucency>| t.map_or(0, Translucency::code);
    let (a, b) = (code(a), code(b));
    // spec: S-0109 Terms (two codes combine to the larger, except that 1 and 2 combine to 3)
    let c = if a.min(b) == 1 && a.max(b) == 2 {
        3
    } else {
        a.max(b)
    };
    Translucency::from_code(c as i32)
}

/// The fixed seed of the viewer's sandbox games (our own choice: the
/// viewer has no lobby).
pub const SANDBOX_SEED: u32 = 1;
/// The ticks a target line is shown after its starting shot (S-0301 claim 24 b).
const TARGET_LINE_TICKS: i32 = 15; // spec: S-0301 claim 24 b (drawn while 0 ≤ t − t₀ < 15, for 15 ticks)

// ---------------------------------------------------------------------------
// Pacing (S-0201 claims 2 to 5)
// ---------------------------------------------------------------------------

/// The paced period of game-speed index `speed` in milliseconds (S-0201
/// claims 4 and 5), and that of a network game (claim 17), from which the
/// relay works out the drop grace period in ticks (M9 N5); both kept with
/// the setup.
pub use glasnost_ui::setup::{net_period_ms, period_ms};

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
        self.due_after(now_ms, period_ms(speed))
    }

    /// Whether a tick is due at host time `now_ms` for cycles of at least
    /// `period` milliseconds (a network game's, S-0201 claim 17); as
    /// [`Pacer::due`] otherwise.
    pub fn due_after(&mut self, now_ms: i64, period: i64) -> bool {
        let due = self.cycle_start.is_none_or(|s| now_ms - s >= period);
        if due {
            // spec: S-0201 claim 3 (each cycle measures its paced period from its own start) and claim 17 (so does a network game's, with no catch-up)
            self.cycle_start = Some(now_ms);
        }
        due
    }

    /// Tells the pacer of a drawn frame at host time `now_ms` that ran no
    /// tick without asking [`Pacer::due_after`] (a network game waiting on
    /// its period or on the other peers).
    pub fn waited(&mut self, _now_ms: i64) {}

    /// Milliseconds until the next tick is due (0 when it is).
    pub fn remaining(&self, now_ms: i64, speed: u8) -> i64 {
        self.remaining_for(now_ms, period_ms(speed))
    }

    /// Milliseconds until the next tick is due for cycles of at least
    /// `period` milliseconds (0 when it is).
    pub fn remaining_for(&self, now_ms: i64, period: i64) -> i64 {
        self.cycle_start
            .map_or(0, |s| (period - (now_ms - s)).max(0))
    }

    /// Forgets the current cycle (after a pause: S-0201 claims 3 and 8, the
    /// game resumes without catching up).
    pub fn reset(&mut self) {
        self.cycle_start = None;
    }
}

/// Where each magnetic wave goes in an air-layer list: the index of the
/// entrant it is drawn before, or the list's length for its end. `afters`
/// gives each wave's next animation serial at its creation, in creation
/// order.
///
/// S-0607 claim 26 places a wave after every animation made before it
/// (serial below its `after`). Past the last of those, our interim puts it
/// before the first particle or newer animation, otherwise at the end; its
/// place against mobile objects and static entries is open (claim 26 orders
/// it only against animations).
pub(crate) fn wave_slots(list: &[Entrant], afters: impl Iterator<Item = u32>) -> Vec<usize> {
    afters
        .map(|after| {
            // spec: S-0607 claim 26 (after the animations made before the wave)
            let start = list
                .iter()
                .rposition(|e| matches!(*e, Entrant::Anim(s) if s < after))
                .map_or(0, |i| i + 1);
            // Our interim: before the next particle or newer animation, else at the end.
            list[start..]
                .iter()
                .position(|e| match *e {
                    Entrant::Anim(s) => s >= after,
                    Entrant::Particle(_) => true,
                    _ => false,
                })
                .map_or(list.len(), |j| start + j)
        })
        .collect()
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
    /// A projectile in flight (S-0306 claim 28), by its object number.
    Projectile(ObjectId),
    /// A debris piece in play (S-0317), by its object number.
    Piece(ObjectId),
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
    /// The owner and health of each of the map's buildings
    /// ([`Built::objects`] indices) as the game started: what the cell
    /// pass drew it with.
    map_start: BTreeMap<u32, (Option<HouseId>, i32)>,
    /// Each cell's static overlay objects ([`Built::objects`] indices).
    overlay_codes: BTreeMap<CellXY, Vec<u32>>,
    /// Each cell's overlay and overlay data value when the game started, by
    /// the grid's dense index (a lookup per cell and tick in a map was a
    /// large part of a late game's redraw, M8 step C4).
    start_overlays: Vec<(u8, u8)>,
    /// The grid's legal cells with their dense indices, in the grid's
    /// order: the overlay comparison walks these instead of the whole
    /// coordinate square (M8 step C4 part 4).
    legal_cells: Vec<(CellXY, usize)>,
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
    /// The tick's layer moments of S-0102 claim 4 the simulation reported,
    /// in order, until the layers take them.
    layer_moments: Vec<super::layers::Moment>,
    /// The drawing's quirk switches (S-0102 Edge cases).
    view_quirks: super::quirks::ViewQuirks,
    /// Images and rows the combat drawing added.
    cache: Cache,
    /// Shots, detonations and animations still shown.
    transients: Vec<Transient>,
    /// The beams, beam flashes and magnetic waves shown (S-0518 claims 17
    /// to 23, S-0514 claims 23 to 26, S-0607 claims 25 and 26).
    lines: beams::Lines,
    /// The building types (import indices) whose art has the double-thick
    /// flag (S-0514 claim 27).
    double_thick: BTreeSet<usize>,
    /// Counts of the combat events seen.
    pub counts: Counts,
    /// The selection whose status extras are drawn (presentation state).
    marked: Vec<ObjectId>,
    /// The object under the pointer (presentation state).
    hovered: Option<ObjectId>,
    /// The digit each member of the local player's control groups shows
    /// (S-0111 claim 17; presentation state, the client's groups).
    group_digits: BTreeMap<ObjectId, u8>,
    /// Each house's display RGB, in house order (S-0103 claim 15), for the
    /// control-group number's house colour.
    house_colours: Vec<[u8; 3]>,
    /// The game font's digits the control-group number is drawn with
    /// (S-0111 claim 17); `None` without a usable font: no number.
    group_font: Option<combat::GroupFont>,
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
    /// Every projectile type of the simulation's rules, by its index: its
    /// import registration index and drawing fields (S-0306 Terms), `None`
    /// for a name the import does not list.
    projectile_table: Vec<Option<(usize, glasnost_convert::art::ProjectileLook)>>,
    /// The projectiles that drew items in the last redraw.
    projectiles_drawn: BTreeSet<ObjectId>,
    /// Every debris-piece type of the simulation's rules, by its index:
    /// its import registration index as a voxel animation, `None` for a
    /// name the import does not list.
    piece_table: Vec<Option<usize>>,
    /// The debris pieces that drew items in the last redraw.
    pieces_drawn: BTreeSet<ObjectId>,
    /// Each vehicle's target-line timer: the tick number *t*₀ of its latest
    /// starting shot (S-0301 claim 24 b).
    target_lines: BTreeMap<ObjectId, i32>,
    /// The drawing's cosmetic stream, for the playback draws the
    /// simulation does not take (a seam: S-0109 claim 28 puts them on the
    /// scenario stream once playback is in the simulation).
    cosmetic: glasnost_sim::rng::Rng,
    /// The battlefield view's origin and size at the latest redraw, for the
    /// electric bolts' view test (S-0316 claim 5); `None` (no view known):
    /// every bolt counts as in view.
    pub battlefield: Option<((i32, i32), (u32, u32))>,
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
    /// The set-up superweapon buildings' charge readings at the end of the
    /// set-up (S-0108 claim 11), used at their grand opening.
    setup_charges: BTreeMap<ObjectId, Option<i32>>,
    /// Each building type's charged-animation time (S-0108 claim 18), by
    /// the simulation's type index, read from the import at the start.
    charged_anim_times: Vec<Option<glasnost_num::F64>>,
    /// The walls' slot starts and stops the last tick reported (S-0606
    /// claims 6, 12 and 16): building, slots, start, damaged variant.
    special_events: Vec<(ObjectId, [u8; 2], bool, bool)>,
    /// The repair depots' *idle* and *active-1* starts and stops the last
    /// tick reported (S-0108 claim 20): building, active-1 (else idle),
    /// start, damaged variant.
    depot_events: Vec<(ObjectId, bool, bool, bool)>,
    /// The buildings whose ownership change the last tick reported with an
    /// announcement (S-0509 claim 12): a change in the turn pass, such as an
    /// engineer that stands after the slots, not a trigger's (S-0108 claim 26; Q-2186).
    turn_captures: BTreeSet<ObjectId>,
    /// The bodies that rock (S-0113): the view's copy, moved once per game
    /// tick, until the angles and rates move into the game state (claim 19,
    /// a seam, docs/m8-claims/intake53-S-0107-S-0108-S-0113.md).
    rocking: super::rocking::Rocking,
    /// The vehicles' walk counters (S-0102 claim 22, S-0105 claim 16): the
    /// view's own, moved once per game tick.
    walks: super::walk::Counters,
    /// Each vehicle type's walk and idle rates (S-0102 Import mapping), by
    /// the simulation's vehicle type index.
    walk_rates: Vec<super::walk::Rates>,
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
        // spec: S-0109 Edge cases (the clock's switches are the ruleset's quirks)
        let aq = AnimQuirks::from(&self.env.rules.quirks);
        let mut player = Player::new(name, play, self.now, None, &mut *self.env, &aq);
        if let Some(old) = from {
            // spec: S-0108 claim 25 (the new animation keeps the replaced one's stage, not its image frame, and takes its passes afresh)
            player.keep_stage(old.player.stage());
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
    /// The ordinary animation in play it shows (S-0109 claim 28: the
    /// simulation plays it with the scenario stream, so its type, stage and
    /// position are the simulation's); one the simulation has removed
    /// disappears.
    sim: Option<ObjectId>,
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
        let play = anims::playback_of(&e.t, e.n, delay, random, &AnimQuirks::from(q));
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
/// speed fraction and unfinished aiming-facing turn; or, for a vehicle the
/// simulation marks sinking, its pitch and roll (S-0504 claims 15, 16).
pub fn attitude(w: &World, o: &Object) -> Attitude {
    if o.kind == SimKind::Vehicle && glasnost_sim::naval::sinking(w, o.id) {
        let drawn = draws_rocking(w, o);
        return Attitude::Sinking {
            // spec: S-0504 claim 15 and F4 (the pitch the simulation steps at each descent)
            pitch: glasnost_sim::naval::pitch(w, o.id).0,
            // spec: S-0504 claim 16 and F5 (r: the roll; rocking, S-0113, is the view's copy until claim 19 is built, which attitude_with adds; the game state alone gives a level roll)
            roll: 0,
            drawn,
        };
    }
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
        // spec: S-0113 claim 16 (the tumble angles are the view's copy until claim 19 is built, which attitude_with adds; the game state alone gives none)
        tumble: (0, 0),
    }
}

/// Whether a vehicle's locomotor draws rocking: its type's locomotor is
/// the driving, ship, hovering or jumpjet one (S-0504 claim 16; S-0106
/// claims 5 and 9).
pub fn draws_rocking(w: &World, o: &Object) -> bool {
    use glasnost_rules::sim::Locomotor as L;
    // spec: S-0504 claim 16 (the lean is drawn for a type whose locomotor is the driving, ship, hovering or jumpjet one; S-0106 claims 5 and 9)
    // design: the type's own locomotor (S-0207 claim 14), not a carrier's: a vehicle sinks only from its own movement, and no spec gives a lift or teleport carrier's drawing a lean
    w.rules().unit(o.kind, o.type_index).is_some_and(|u| {
        matches!(
            u.type_locomotor(),
            L::Drive | L::Ship | L::Hover | L::Jumpjet
        )
    })
}

/// [`attitude`] with the view's rocking (`super::rocking`): a sinking
/// vehicle's roll, and its lean when it was rocked as it began to sink
/// (S-0504 claim 15, S-0113 claim 5); a vehicle that rocks and does not
/// sink is drawn with its pitch and roll (S-0113 claim 18, S-0106 claim 9);
/// a crashing aircraft with its tumble (S-0113 claims 16 and 18).
pub fn attitude_with(w: &World, o: &Object, rocking: &super::rocking::Rocking) -> Attitude {
    let angles = rocking.angles(o.id);
    match attitude(w, o) {
        Attitude::Sinking { pitch, drawn, .. } => Attitude::Sinking {
            // spec: S-0504 claim 15 (Starting value: the lean steps from the rocked pitch the vehicle had when sinking began)
            pitch: rocking.lean(o.id).map_or(pitch, |p| p.0),
            // spec: S-0113 claim 5 (a sinking body's roll takes table R's roll step: a push during descent rolls the hull) and S-0504 claim 16 (drawn as S-0106 claim 9's rocking)
            roll: angles.map_or(0, |(_, r)| r.to_bits()),
            drawn,
        },
        Attitude::Flying {
            height,
            speed,
            turn,
            crashing: true,
            ..
        } => Attitude::Flying {
            height,
            speed,
            turn,
            crashing: true,
            // spec: S-0113 claims 18 and 20 (the flying locomotor draws the pitch and roll only while the aircraft is crashing; the body at positive height is drawn fresh with claim 16's angles, so they are not held to the drawing step)
            tumble: rocking
                .exact_angles(o.id)
                .map_or((0, 0), |(p, r)| (p.to_bits(), r.to_bits())),
        },
        Attitude::None if o.kind == SimKind::Vehicle => match angles {
            Some((pitch, roll)) if pitch != 0.0 || roll != 0.0 => Attitude::Rocking {
                pitch: pitch.to_bits(),
                roll: roll.to_bits(),
                drawn: draws_rocking(w, o),
            },
            _ => Attitude::None,
        },
        other => other,
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

/// Building type `name`'s charged-animation time *t_c* in game minutes
/// (S-0108 claim 18, Import mapping): the rules key `ChargedAnimTime` of
/// the building's section, a decimal held in single precision and widened
/// exactly to binary64, read under rounding `mode`; `None` when the import
/// does not set it (its default, 999, turns claim 18 off). No typed field
/// carries the key, so it is read from the import's stored INI text of the
/// type.
pub fn charged_anim_time(
    import: &glasnost_convert::Import,
    name: &str,
    mode: glasnost_num::Rounding,
) -> Option<glasnost_num::F64> {
    // spec: S-0108 Import mapping (ChargedAnimTime: building, decimal, held in single precision, game minutes)
    let text = import
        .tree
        .get("types")?
        .get(Kind::Buildings.key())?
        .get(name)?
        .get("ini")?
        .get("ChargedAnimTime")?
        .as_str()?;
    // spec: S-0108 F6 (t_c is the single-precision field widened exactly to binary64); S-0003 claim 7 (a single-precision field's value)
    glasnost_num::decimal::parse(text, glasnost_num::decimal::Field::Single, mode).ok()
}

/// The remaining charge in ticks that building `o`'s superweapon slots read
/// at tick `now` from its owner's record of its first superweapon
/// `first_sw` (S-0108 claims 11 and 18, S-0512 F2); `None` when there is no
/// reading (no superweapon or no owner, or, with
/// `quirks.ungranted_super_reads_clock` off, a record that is not granted).
pub fn charge_reading(w: &World, o: &Object, first_sw: Option<u32>, now: i32) -> Option<i32> {
    let reads_ungranted = w.rules().quirks.ungranted_super_reads_clock;
    let (sw, h) = first_sw.zip(o.owner)?;
    match glasnost_sim::superweapons::record(w, h, sw) {
        // spec: S-0108 claims 11 and 18 (quirks.ungranted_super_reads_clock on: the clock is read whether or not the record is granted; off: a record that is not granted counts as not charged)
        Some(rec) if rec.granted || reads_ungranted => Some(rec.remaining(now)),
        Some(_) => None,
        // spec: S-0108 claim 11 (each house holds one record of each type from its creation, S-0512 claim 1; a record never granted and never reset reads 0); the simulation allocates a house's records when it first writes one
        None => reads_ungranted.then_some(0),
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
        let charged_anim_times = rules
            .buildings
            .iter()
            .map(|b| charged_anim_time(inputs.import, &b.name, rules.mode))
            .collect();
        // spec: S-0103 claim 15 (a house's display RGB, sampled once at colour setup; 8-bit channels)
        let house_rgb: Vec<[u8; 3]> = world
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
        // spec: S-0102 Import mapping (WalkRate, IdleRate of the vehicle type: 1 and 0 when absent)
        let walk_rates = rules
            .vehicles
            .iter()
            .map(|v| {
                let d = super::walk::Rates::default();
                find_type(inputs.import, Kind::Vehicles, v.name.as_bytes()).map_or(d, |i| {
                    super::walk::Rates {
                        walk: rule_int(inputs.import, Kind::Vehicles, i, b"WalkRate", d.walk),
                        idle: rule_int(inputs.import, Kind::Vehicles, i, b"IdleRate", d.idle),
                        // spec: S-0102 Import mapping (HoverAttack and DeployToLand: boolean, no when absent)
                        hover_attack: v.raw.hover_attack.unwrap_or(d.hover_attack),
                        deploy_to_land: v.deploy_to_land,
                    }
                })
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
        let map_start: BTreeMap<u32, (Option<HouseId>, i32)> = building_ids
            .iter()
            .filter_map(|(code, id)| Some((*code, world.obj(*id).map(|o| (o.owner, o.health))?)))
            .collect();
        let mut overlay_codes: BTreeMap<CellXY, Vec<u32>> = BTreeMap::new();
        for (i, info) in built.objects.iter().enumerate() {
            if info.kind == "overlay" {
                overlay_codes.entry(info.cell).or_default().push(i as u32);
            }
        }
        let grid = &world.cells.grid;
        let mut start_overlays = vec![(0, 0); grid.len()];
        let mut legal_cells = Vec::new();
        for (c, cell) in grid.iter() {
            if let Some(i) = grid.index(c) {
                start_overlays[i] = (cell.overlay, cell.overlay_data);
                legal_cells.push((c, i));
            }
        }
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
        let names: Vec<Vec<u8>> = rules
            .combat
            .projectiles
            .iter()
            .map(|t| name_bytes(&t.name))
            .collect();
        let projectile_table = super::projectiles::entries(inputs.import, &names)
            .into_iter()
            .map(|e| {
                e.and_then(|idx| {
                    let look = inputs
                        .import
                        .arts
                        .get(&(Kind::Projectiles, idx))?
                        .projectile
                        .clone()?;
                    Some((idx, look))
                })
            })
            .collect();
        let mut piece_index: BTreeMap<Vec<u8>, usize> = BTreeMap::new();
        if let Some(reg) = inputs.import.outcome.registries.get(&Kind::VoxelAnimations) {
            for (idx, t) in reg.iter().enumerate() {
                // spec: S-0003 claim 14 (names match with ASCII case folding; the first registered wins)
                piece_index
                    .entry(t.name.to_ascii_uppercase())
                    .or_insert(idx);
            }
        }
        let piece_table = rules
            .pieces
            .types
            .iter()
            .map(|t| {
                piece_index
                    .get(&name_bytes(&t.name).to_ascii_uppercase())
                    .copied()
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
        let mut settings = beams::Settings::read(inputs.import, &rules, &house_rgb);
        // spec: S-0316 claim 10 and Import mapping (the bolt colours from the general palette, PALETTE.PAL)
        settings.bolt = inputs
            .assets
            .get(combat::OVERLAY_PALETTE)
            .and_then(super::bolts::Colours::from_palette_file);
        let lines = beams::Lines::new(settings);
        // spec: S-0514 Import mapping (DoubleThick, building art: boolean, absent no)
        let double_thick = (0..inputs
            .import
            .outcome
            .registries
            .get(&Kind::Buildings)
            .map_or(0, |r| r.len()))
            .filter(|&i| {
                painter
                    .art_bool(inputs.import, Kind::Buildings, i, b"DoubleThick")
                    .unwrap_or(false)
            })
            .collect();
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
            map_start,
            overlay_codes,
            start_overlays,
            legal_cells,
            changed_overlays: BTreeMap::new(),
            live_overlays: Vec::new(),
            objects_end: built.scene.items.len(),
            effects_end: built.scene.items.len(),
            ground: GroundLayer::default(),
            layers: super::layers::Layers::default(),
            layer_moments: Vec::new(),
            view_quirks: super::quirks::ViewQuirks::default(),
            cache: Cache::default(),
            transients: Vec::new(),
            lines,
            double_thick,
            counts: Counts::default(),
            marked: Vec::new(),
            hovered: None,
            group_digits: BTreeMap::new(),
            house_colours: Vec::new(),
            group_font: None,
            notes: BTreeSet::new(),
            undrawn_types: undrawn,
            failed: None,
            scripts: Vec::new(),
            anims: Vec::new(),
            anim_serial: 0,
            anim_table,
            particle_table,
            projectile_table,
            projectiles_drawn: BTreeSet::new(),
            piece_table,
            pieces_drawn: BTreeSet::new(),
            target_lines: BTreeMap::new(),
            cosmetic,
            battlefield: None,
            debug_markers: true,
            move_flash,
            viewer: None,
            source_marker: None,
            slot_anims: BTreeMap::new(),
            slot_defs: BTreeMap::new(),
            charged_anim_times,
            special_events: Vec::new(),
            depot_events: Vec::new(),
            turn_captures: BTreeSet::new(),
            rocking: Default::default(),
            walks: Default::default(),
            walk_rates,
            setup_charges: BTreeMap::new(),
        };
        // spec: S-0108 claim 11 (a building placed during the game's set-up opens during the set-up: the record it meets is the one the set-up left, before the first tick's house updates grant it)
        live.setup_charges = live
            .building_ids
            .values()
            .filter_map(|id| live.world.obj(*id))
            .filter_map(|o| {
                let first_sw = live
                    .rules
                    .superweapons
                    .building_sw
                    .get(o.type_index as usize)
                    .and_then(|s| s[0]);
                first_sw.map(|_| {
                    (
                        o.id,
                        charge_reading(&live.world, o, first_sw, live.world.tick),
                    )
                })
            })
            .collect();
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
        live.layers.evaluate_mobiles(&live.world, false, true, &[]);
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
            if super::layers::listed(&live.world, o.id)
                && live.layers.of(o.id) == Some(super::slots::Layer::Ground)
            {
                entrants.push((o.id.0, Entrant::Object(o.id), mobile_key(o)));
            }
        }
        entrants.sort_by_key(|(rank, e, _)| (*rank, *e));
        for (_, e, key) in entrants {
            live.ground.enter(e, key);
        }
        // spec: S-0519 Import mapping (a missing bomb-icon image draws no icon, and the importer warns)
        if inputs.assets.get(combat::BOMB_SPRITE).is_none() {
            live.notes.insert(BOMB_ICON_MISSING.to_owned());
        }
        // spec: S-0111 Import mapping (a missing wrench sprite draws nothing, and the importer warns)
        if inputs.assets.get(combat::WRENCH_SPRITE).is_none() {
            live.notes.insert(WRENCH_MISSING.to_owned());
        }
        // spec: S-0111 Import mapping (a missing overlay sprite draws nothing, with an importer warning)
        for (file, note) in [
            (combat::PIP_SPRITE, PIPS_MISSING),
            (combat::PIP2_SPRITE, PIPS2_MISSING),
        ] {
            if inputs.assets.get(file).is_none() {
                live.notes.insert(note.to_owned());
            }
        }
        live.note_cursor_palette(inputs.assets);
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

    /// The bodies that rock (S-0113; the view's copy until claim 19 is built).
    pub fn rocking(&self) -> &super::rocking::Rocking {
        &self.rocking
    }

    /// The walk counter of vehicle `id` and whether it moved in the shown
    /// tick, as the view keeps them (S-0102 claim 22).
    pub fn walk(&self, id: ObjectId) -> super::walk::Walk {
        self.walks.of(id)
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
        self.layer_moments = self
            .world
            .effects
            .iter()
            .filter_map(|e| match e {
                Effect::LayerMoment { object, check } => Some((*object, *check)),
                _ => None,
            })
            .collect();
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
        let moments = std::mem::take(&mut self.layer_moments);
        self.layers
            .evaluate_mobiles(&self.world, tick, events_only, &moments);
        let anims: BTreeMap<u32, Layer> = self.anims.iter().map(|a| (a.serial, a.layer)).collect();
        let particles: BTreeSet<ObjectId> = self
            .world
            .gas
            .systems
            .values()
            .flat_map(|s| s.particles.iter().map(|p| p.id))
            .collect();
        let projectiles: Vec<(ObjectId, bool)> = self
            .world
            .projectiles
            .values()
            .map(|p| {
                (
                    p.id,
                    self.projectile_look(p.weapon).is_some_and(|(_, l)| l.flat),
                )
            })
            .collect();
        let in_play: BTreeSet<ObjectId> = projectiles.iter().map(|(id, _)| *id).collect();
        let pieces: Vec<ObjectId> = self.world.pieces.keys().copied().collect();
        let world = &self.world;
        self.layers.retain(|e| match e {
            Entrant::Anim(s) => anims.get(&s).is_some_and(|l| *l != Layer::Ground),
            Entrant::Particle(id) => particles.contains(&id),
            Entrant::Projectile(id) => in_play.contains(&id),
            Entrant::Piece(id) => world.pieces.contains_key(&id),
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
        for (id, flat) in projectiles {
            if !self.layers.contains(Entrant::Projectile(id)) {
                // spec: S-0102 claims 3, 4 and 7 (a projectile is in the air layer, or the surface layer when its image is flat; placed at the layer's end)
                let layer = if flat { Layer::Surface } else { Layer::Air };
                self.layers.place(Entrant::Projectile(id), layer);
            }
        }
        for id in pieces {
            if !self.layers.contains(Entrant::Piece(id)) {
                // spec: S-0102 claims 3, 4 and 7 (a voxel animation is in the air layer; placed at the layer's end)
                self.layers.place(Entrant::Piece(id), Layer::Air);
            }
        }
    }

    /// A projectile type's import index and drawing fields, by the weapon
    /// that fired it.
    fn projectile_look(
        &self,
        weapon: glasnost_sim::combat::data::WeaponId,
    ) -> Option<&(usize, glasnost_convert::art::ProjectileLook)> {
        let p = self.rules.combat.weapon(weapon).projectile?;
        self.projectile_table.get(p as usize)?.as_ref()
    }

    /// The buildings' slot animations after a tick (S-0108 claims 11, 12,
    /// 13, 17, 20 and 26; S-0606 claims 6, 12 and 16): each running one
    /// plays (S-0109), or is held while it is frozen or its building is
    /// being erased (S-0109 claim 23), those that finished on their own
    /// leave their slot, then the building's update starts, stops, freezes
    /// and unfreezes slots and the walls' events of the tick apply. A
    /// building whose type has not been drawn yet waits for its slot
    /// definitions.
    fn update_slots(&mut self) {
        let now = self.shown_tick();
        let events = std::mem::take(&mut self.special_events);
        let depot_events = std::mem::take(&mut self.depot_events);
        let turn_captures = std::mem::take(&mut self.turn_captures);
        let enable_unfreezes = self.view_quirks.enable_unfreezes_unpowered;
        let w = &self.world;
        let mode = self.rules.mode;
        let present: Vec<(ObjectId, usize, slot_play::TypeKind, slot_play::State, bool)> = w
            .objects
            .values()
            .filter(|o| o.kind == SimKind::Building && o.on_map)
            .filter_map(|o| {
                let idx = (*self.types.get(&(o.kind, o.type_index))?)?;
                let r = self.rules.buildings.get(o.type_index as usize)?;
                let power = self.rules.econ.power.buildings.get(o.type_index as usize);
                let first_sw = self
                    .rules
                    .superweapons
                    .building_sw
                    .get(o.type_index as usize)
                    .and_then(|s| s[0]);
                let extra = power.map_or(0, |p| p.extra_output);
                let needs_power = power.is_some_and(|p| p.needs_power);
                let kind = slot_play::TypeKind {
                    refinery: r.raw.econ.refinery.unwrap_or(false),
                    // spec: S-0108 claim 17 (absorbs infantry and gives a positive extra power per held infantry)
                    absorbs: r.absorb.infantry && extra > 0,
                    depot: r.building.unit_repair,
                    delayed_fire: r.building.anim_delayed_fire,
                    // spec: S-0108 claim 13 and Import mapping (Powered: needs power)
                    needs_power,
                    // spec: S-0108 Terms (needs and draws power: the needs-power flag and a negative power field, a drain above 0)
                    draws_power: needs_power && power.is_some_and(|p| p.drain > 0),
                    // spec: S-0108 claims 13 and 26 (the needs-engineer flag: only such a type's ownership change re-enables it)
                    needs_engineer: power.is_some_and(|p| p.needs_engineer),
                    // spec: S-0108 claims 11 and 18 (a building with a superweapon: its first superweapon field)
                    superweapon: first_sw.is_some(),
                    charged_anim_time: self
                        .charged_anim_times
                        .get(o.type_index as usize)
                        .copied()
                        .flatten(),
                };
                // spec: S-0108 claim 11 (a set-up building's grand opening is during the set-up, so it meets the record as the set-up left it)
                let charge_remaining = match self.setup_charges.get(&o.id) {
                    Some(c) if !self.slot_anims.contains_key(&o.id) => *c,
                    _ => charge_reading(w, o, first_sw, now),
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
                let st = slot_play::State {
                    // spec: S-0108 claims 11 and 17 (completed and not in the construction state; a building placed in play, or a code-130 replacement, enters with construction queued and opens at its completion) and S-0406 claim 17 (while none is current, the queued behaviour is the one seen)
                    // spec: S-0309 claim 31 (an owner change during the build-up makes the grand opening at once, starting the slot animations)
                    completed: glasnost_sim::econ::placement::seen_behaviour(o)
                        != Some(Behaviour::Construction)
                        || o.econ.construction.opened,
                    // spec: S-0108 claim 11 (a building of the map was placed during the game's set-up)
                    setup: self.building_ids.values().any(|b| *b == o.id),
                    damaged: super::build::condition_of(&self.rules, o.health, r.strength).0,
                    // spec: S-0108 Terms (occupied: at least one occupant; held infantry are not occupants)
                    occupied: !o.structure.occupants.is_empty(),
                    // spec: S-0108 claim 17, F9 (the held infantry's total size)
                    held_size: slot_play::held_size(&sizes, mode),
                    owner: o.owner.map(|h| h.0),
                    // spec: S-0108 Terms and claim 13 (powered: the in-operation test of S-0405 claim 8, every row)
                    in_operation: glasnost_sim::econ::power::in_operation(w, o.id),
                    // spec: S-0108 claim 28 (the switched-on row of S-0405 claim 8)
                    switched_off: o.econ.building.switched_off,
                    // spec: S-0108 claim 28 (the not-under-a-pulse row of S-0405 claim 8)
                    pulsed: glasnost_sim::econ::power::under_pulse(w, o.id),
                    // spec: S-0108 claim 26 (Q-2186: a change made in the turn of an object that stands after the building's slots comes after their turns of the tick; one made before the turn pass, or in the turn of an object before the slots, comes before them)
                    change_after_slots: turn_captures.contains(&o.id),
                    // spec: S-0108 claim 26 (the needs-engineer flag and not manned, S-0515 claim 2)
                    unmanned: power.is_some_and(|p| p.needs_engineer) && !o.econ.building.manned,
                    // spec: S-0108 claim 18 (not checked while the building is being sold)
                    selling: o.econ.sale.is_some(),
                    charge_remaining,
                    // spec: S-0108 F6 (t_r = r × κ)
                    charge_minutes: charge_remaining.map(|r| slot_play::charge_minutes(r, mode)),
                };
                // spec: S-0109 claims 21 and 23 (a building being erased holds its slot animations, a hold independent of claim 13's freeze)
                let erased = glasnost_sim::superweapons::being_erased(w, o.id);
                Some((o.id, idx, kind, st, erased))
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
        let q = AnimQuirks::from(&env.rules.quirks); // spec: S-0109 Edge cases
        for (id, idx, kind, st, erased) in present {
            let Some(defs) = self.slot_defs.get(&idx) else {
                continue;
            };
            let slots = self.slot_anims.entry(id).or_default();
            slots.prepare(&kind, &st, enable_unfreezes);
            let mut finished = Vec::new();
            for (slot, r) in &mut slots.running {
                let before = r.anim.player.name.clone();
                // spec: S-0109 claim 23 (held while frozen by S-0108 claim 13 or while its building is being erased; the two causes are independent)
                let held = r.frozen || erased;
                // spec: S-0109 claims 5 to 16 (each running slot animation plays; claim 16: it ends) and claim 23 (a held one does not advance)
                if !r.anim.player.update_held(now, &mut env, &q, held) {
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
            for (_, active1, start, damaged) in depot_events.iter().filter(|(b, ..)| *b == id) {
                slots.depot(*active1, *start, *damaged, defs, &mut mk);
            }
        }
    }

    /// A building's running slot animations as the drawing takes them.
    fn slot_draws(&self, id: ObjectId, idx: usize) -> Vec<super::build::LiveSlot> {
        let Some(defs) = self.slot_defs.get(&idx) else {
            return Vec::new();
        };
        // spec: S-0405 claim 9 and Changes for other specs (revision 36, S-0108 claim 10: a building being sold, or packing up, S-0415 claim 9, keeps its slot animations drawn until the first tick of its build-down, S-0408 claim 6, and hides them from then on)
        if self.world.obj(id).is_some_and(|o| {
            matches!(
                o.econ.sale,
                Some(glasnost_sim::econ::sell::Sale {
                    stage: glasnost_sim::econ::sell::Stage::BuildDown(_),
                    ..
                })
            )
        }) {
            return Vec::new();
        }
        let mut out: Vec<(slot_play::Slot, super::build::LiveSlot)> = self
            .slot_anims
            .get(&id)
            .into_iter()
            .flat_map(|slots| slots.running.iter())
            .filter(|(_, r)| r.anim.entry.sprite)
            .filter_map(|(slot, r)| {
                Some((
                    *slot,
                    super::build::LiveSlot {
                        def: defs.get(slot)?.clone(),
                        idx: r.anim.entry.idx,
                        frame: r.anim.player.frame(),
                        stage: r.anim.player.stage(),
                        end: r.anim.player.play.end,
                    },
                ))
            })
            .collect();
        // spec: S-0108 claim 20 (a repair depot's production and special-1 to special-3 slot animations are simulation state; the drawing shows them as the simulation plays them)
        for d in glasnost_sim::depot_anim::draws(&self.world, id) {
            let slot = match d.slot {
                glasnost_sim::depot::DepotSlot::Production => slot_play::Slot::Production,
                glasnost_sim::depot::DepotSlot::Special1 => slot_play::Slot::Special1,
                glasnost_sim::depot::DepotSlot::Special2 => slot_play::Slot::Special2,
                glasnost_sim::depot::DepotSlot::Special3 => slot_play::Slot::Special3,
            };
            let Some(entry) = self.anim_table.get(&d.name.as_bytes().to_ascii_uppercase()) else {
                continue;
            };
            let Some(def) = defs.get(&slot) else { continue };
            if !entry.sprite {
                continue;
            }
            out.push((
                slot,
                super::build::LiveSlot {
                    def: def.clone(),
                    idx: entry.idx,
                    frame: d.frame,
                    stage: d.stage,
                    end: d.end,
                },
            ));
        }
        out.sort_by_key(|(s, _)| *s);
        out.into_iter().map(|(_, l)| l).collect()
    }

    /// Whether a building is drawn live, as of the last redraw: always for
    /// one the game placed, and for one of the map's when it has left the
    /// cell pass (tests and lanes).
    pub fn drawn_live(&self, id: ObjectId) -> bool {
        self.building_ids
            .iter()
            .find(|(_, b)| **b == id)
            .is_none_or(|(code, _)| self.gone.contains(code))
    }

    /// The import registration index of a simulation type, once a redraw
    /// has met it (tests and lanes).
    pub fn import_index(&self, kind: SimKind, index: u32) -> Option<usize> {
        self.types.get(&(kind, index)).copied().flatten()
    }

    /// How many of a building's running slot animations are frozen by
    /// S-0108 claim 13 (tests and lanes).
    pub fn frozen_slots(&self, id: ObjectId) -> usize {
        self.slot_anims
            .get(&id)
            .map_or(0, |s| s.running.values().filter(|r| r.frozen).count())
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

    /// The protection look of object `id` as drawn (tests).
    #[cfg(test)]
    pub(super) fn protection_look_of(&self, id: ObjectId) -> Option<super::protection::Look> {
        self.protection_look(self.world.obj(id)?)
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

    /// Gives or withdraws the missing-cursor-palette note for the game's
    /// cursor palette in `assets` (S-0111 claim 19a: with no cursor
    /// palette, neither the wrench nor the bomb icon is drawn, and the
    /// importer warns once, naming the file). The game client calls it
    /// again when it puts the session's kept copy in place.
    pub fn note_cursor_palette(&mut self, assets: &super::build::Assets) {
        // spec: S-0111 claim 19a and Edge cases (missing cursor palette: a departure; one warning naming the file)
        if assets.get(combat::CURSOR_PALETTE).is_none() {
            self.notes.insert(CURSOR_PALETTE_MISSING.to_owned());
        } else {
            self.notes.remove(CURSOR_PALETTE_MISSING);
        }
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
        use glasnost_sim::cloak::view;
        let w = &self.world;
        let v = self.viewer.or_else(|| w.obj(id).and_then(|o| o.owner));
        cloak_items(
            view::look(w, id, v),
            view::shadow(w, id),
            self.erased_code(id),
            items,
        );
    }

    /// The translucency of an object being erased or phasing in (S-0514
    /// claim 27): code 2, or 3 for a building whose art has the
    /// double-thick flag; `None` otherwise.
    pub(super) fn erased_code(
        &self,
        id: ObjectId,
    ) -> Option<glasnost_render::compose::Translucency> {
        use glasnost_render::compose::Translucency;
        let w = &self.world;
        // spec: S-0514 claim 27 (while being erased, the leaving stage included, or phasing in, S-0513 claim 40)
        if !(glasnost_sim::superweapons::being_erased(w, id)
            || glasnost_sim::superweapons::warp::phasing(w, id))
        {
            return None;
        }
        let o = w.obj(id)?;
        let thick = o.kind == SimKind::Building
            && self
                .types
                .get(&(o.kind, o.type_index))
                .copied()
                .flatten()
                .is_some_and(|idx| self.double_thick.contains(&idx));
        // spec: S-0514 claim 27 (code 2; code 3 for a building whose art has the double-thick flag)
        Some(if thick {
            Translucency::ThreeQuarter
        } else {
            Translucency::Half
        })
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
        let aq = AnimQuirks::from(&env.rules.quirks); // spec: S-0109 Edge cases
        let player = Player::new(name, play, first_update, drawn, &mut env, &aq);
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
            sim: None,
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

    /// The ordinary animations the simulation plays (S-0109 claims 28 and
    /// 29): each drawn animation shows its simulated animation's type,
    /// stage and position; one the simulation has removed disappears.
    fn follow_sim_anims(&mut self) {
        let world = &self.world;
        let table = &self.anim_table;
        let painter = &self.painter;
        self.anims.retain_mut(|a| {
            let Some(id) = a.sim else { return true };
            let Some(s) = world.anims.anims.get(&id) else {
                // spec: S-0109 claims 16 and 22 (removed in the simulation: no longer drawn)
                return false;
            };
            if a.player.name != s.player.name {
                // spec: S-0109 claim 15 (the same animation continues as the follow-up type)
                a.entry = table.get(&s.player.name.to_ascii_uppercase()).cloned();
            }
            // spec: S-0109 claim 28 (no viewer-dependent cosmetic stream substitutes for the playback draws: the stage shown is the simulation's)
            a.player.clone_from(&s.player);
            if let Some(p) = s.pos {
                a.pos = p;
            }
            if let glasnost_sim::anims::Role::Grab { house, cell, .. } = s.role {
                // spec: S-0614 claim 25 (attached to its host: in the ground render layer, S-0109 claim 22)
                a.layer = super::slots::Layer::Ground;
                // spec: S-0614 claim 25 and S-0103 claim 8 row 3 (drawn with the parasite's house-colour palette at the unscaled ground level of the host's cell, both as the parasite last gave them; the normal-light flag and the earlier rows and tiled copies stay S-0103's)
                a.carrier = house
                    .and_then(|h| world.houses.get(usize::from(h.0)))
                    .map(|h| {
                        // design: no cell given yet (never in play: the host is on the map through the hold, claim 15) draws at 1000
                        let light = cell.map_or(NEUTRAL_LIGHT, |c| painter.ground_light(c));
                        (h.name.as_bytes().to_vec(), light)
                    });
            }
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
        let q = AnimQuirks::from(&env.rules.quirks); // spec: S-0109 Edge cases
        self.anims.retain_mut(|a| {
            // spec: S-0615 claim 4 (a burning walker replaces S-0109's playback)
            if now < a.first_update
                || a.walker.is_some()
                || a.maker.is_some()
                || a.dominating.is_some()
                || a.sim.is_some()
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
        // spec: S-0113 claim 19 and State and ordering (the rocking step is the first part of a body's turn, once per game tick and not per drawn frame; the tick's pushes then set the rates, which show from the next step)
        self.rocking.advance(&self.world);
        self.rocking.take(&self.world, &effects);
        // spec: S-0102 claim 22 (the walk counter grows in a tick, once per game tick and not per drawn frame)
        self.walks.advance(&self.world, &self.walk_rates);
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
                event:
                    glasnost_sim::superweapons::SwEvent::Parachute {
                        trooper,
                        delay,
                        anim,
                    },
                pos: Some(pos),
                ..
            } = e
            {
                let serial = self.anim_serial;
                self.attach_parachute(*trooper, *pos, now, *delay);
                if let Some(a) = self.anims.iter_mut().find(|a| a.serial == serial) {
                    a.sim = *anim;
                }
            }
            if let Effect::ShotAnim {
                name,
                pos,
                delay,
                attached,
                z_adjust,
                id,
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
                    a.sim = *id;
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
                z_adjust,
                id,
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
                if z_adjust.is_some()
                    && let Some(a) = self.anims.iter_mut().find(|a| a.serial == serial)
                {
                    // spec: S-0102 claim 13 and S-0506 claim 19 (its creator replaces the animation's depth adjustment: −15 for a rocket detonation's explosion)
                    a.z_adjust = *z_adjust;
                }
                if let Some(a) = self.anims.iter_mut().find(|a| a.serial == serial) {
                    a.sim = *id;
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
        self.follow_sim_anims();
        self.update_anims(now);
        let next_anim = self.anim_serial;
        for e in &effects {
            // spec: S-0518 claims 6 and 18, S-0615 claims 15 and 16, S-0514 claim 23, S-0607 claim 25 (the line effects of the tick)
            self.lines.take(&self.world, e, now, next_anim);
            let quirk = self.rules.quirks.ranged_draw_half_span;
            let cosmetic = &mut self.cosmetic;
            // spec: S-0316 claim 13 (the bolt's draws come from the unsynchronised stream: our cosmetic stream)
            let mut draw = |a: i32, b: i32| cosmetic.ranged(a, b, quirk);
            // spec: S-0316 claims 3 and 4 (the tick's electric bolts, made after the shot's report)
            self.lines.take_bolt(&self.world, e, now, &mut draw);
            if matches!(e, Effect::Bolt { .. }) {
                self.counts.bolts += 1;
            }
        }
        for e in effects {
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
                Effect::TargetLine { firer, house } => {
                    // spec: S-0301 claim 24 a and b (started only for the local player who owns the vehicle at the shot; a new start restarts the 15 ticks; our viewer with no local player shows every line)
                    if self.viewer.is_none_or(|v| house == Some(v)) {
                        self.target_lines.insert(firer, now);
                    }
                    continue;
                }
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
                Effect::DepotSlot {
                    building,
                    active1,
                    start,
                    damaged,
                } => {
                    // spec: S-0108 claim 20 (a repair depot's idle or active-1 starts or stops)
                    self.depot_events.push((building, active1, start, damaged));
                    continue;
                }
                Effect::Captured {
                    building,
                    announce: true,
                    after_slots,
                    ..
                } => {
                    // spec: S-0108 claim 26 (Q-2186: the change comes after the building's slots' turns when the object that made it stands after them in the update order); S-0509 claim 12 (the announcing causes)
                    if after_slots {
                        self.turn_captures.insert(building);
                    }
                    continue;
                }
                Effect::Sound { .. }
                | Effect::Mind { .. }
                | Effect::Consumed { .. }
                | Effect::Spin { .. }
                | Effect::Trigger { .. }
                | Effect::Hit { .. }
                | Effect::LeftoverHit { .. }
                | Effect::Rock { .. }
                | Effect::DroneRock { .. }
                | Effect::GrapplePush { .. }
                | Effect::OverloadRock { .. }
                | Effect::CrashTumble { .. }
                | Effect::OverlayCrushed { .. }
                | Effect::PieceSound { .. }
                | Effect::Superweapon { .. }
                | Effect::SupportBeam { .. }
                | Effect::Captured { .. }
                | Effect::Infiltrated { .. }
                | Effect::Cloak { .. }
                | Effect::Bunkered { .. }
                | Effect::Converted { .. }
                | Effect::Seized { .. }
                | Effect::Route { .. }
                | Effect::Deactivated { .. }
                | Effect::Dropped { .. }
                | Effect::Disc { .. }
                | Effect::Beam { .. }
                | Effect::Line { .. }
                | Effect::FragmentLine { .. }
                | Effect::Bolt { .. }
                | Effect::Wave { .. }
                | Effect::RailLine { .. }
                | Effect::Drain { .. }
                | Effect::Maker { .. }
                | Effect::Domination { .. }
                | Effect::LayerMoment { .. }
                | Effect::Walker { .. } => continue,
            };
            self.transients.push(Transient {
                kind,
                at: at(pos),
                born: now,
            });
        }
        self.transients.retain(|t| t.marker(now).is_some());
        self.lines
            .after_tick(&self.world, now, self.world.game_speed);
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
                    // spec: S-0102 claims 3, 4 and 6 (a warp member keeps its place in the ground list while it is away, with the key of its position at departure, S-0513 claim 39; it is not drawn then)
                    super::layers::listed(&self.world, o.id)
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

    /// The protection look of `o` in the shown tick (S-0513 claim 29),
    /// with the centre-cell shroud test of claim 29b for the local player:
    /// a building's centre cell is the cell under the centre of its
    /// foundation (S-0104 claim 13), a unit's the cell that holds its
    /// position. Without a local player nothing counts as shrouded.
    fn protection_look(&self, o: &Object) -> Option<super::protection::Look> {
        // spec: S-0513 claim 29 (the envelope's phase as the last tick's turns left it; F7 in that tick)
        let look = super::protection::Look::of(o, self.shown_tick())?;
        let w = &self.world;
        let centre = if o.kind == SimKind::Building {
            let size = w
                .rules()
                .buildings
                .get(o.type_index as usize)
                .map_or((1, 1), |b| b.foundation);
            let top_left = o.pos.cell();
            // spec: S-0513 claim 29b, S-0104 claim 13 (the cell under the foundation's centre, the larger coordinate on an even side)
            let (x, y) =
                glasnost_render::light::building_centre_cell((top_left.x, top_left.y), size);
            CellXY::new(x, y)
        } else {
            // spec: S-0513 claim 29b (a unit's centre cell: the cell that holds its position)
            o.pos.cell()
        };
        // spec: S-0513 claim 29b (centre cell under shroud for the local player: envelope, no mask)
        let shrouded = self
            .viewer
            .is_some_and(|v| !glasnost_sim::econ::shroud::explored(w, v, centre));
        Some(look.shrouded(shrouded))
    }

    /// The look of a building in the shown tick: its protection look, and
    /// its charge flash count and air-strike mark (S-0519 F7).
    fn building_look(&self, o: &Object) -> super::protection::BuildingLook {
        super::protection::BuildingLook {
            protection: self.protection_look(o),
            // spec: S-0519 claims 25 and 26 (the flash count and the mark's phase as the last tick's turns left them; the factor in the shown tick)
            adjust: glasnost_sim::mark::Adjust::of(&self.world, o.id, self.shown_tick()),
        }
    }

    /// The look of the building occupying each cell and the cell's own
    /// shroud answer for the local player, for the slot animations' mask
    /// (S-0102 claim 34, animation rule; S-0519 claim 26; S-0513 claim
    /// 29b). Only cells a protected, marked or flashing building occupies
    /// are listed: an animation over any other cell takes no mask.
    pub(super) fn protected_cells(&self) -> BTreeMap<CellXY, super::protection::SlotCell> {
        let mut cells = BTreeMap::new();
        for o in self.world.objects.values() {
            if o.kind != SimKind::Building || !o.on_map {
                continue;
            }
            let look = self.building_look(o);
            if !look.active() {
                continue;
            }
            // spec: S-0406 claim 2 (the cells a building occupies: its foundation cells)
            for c in glasnost_sim::econ::placement::foundation_cells(
                &self.world,
                o.type_index,
                o.pos.cell(),
            ) {
                let shrouded = self
                    .viewer
                    .is_some_and(|v| slot_cell_shrouded(&self.world, v, c));
                cells.insert(c, super::protection::SlotCell { look, shrouded });
            }
        }
        cells
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

    /// The objects whose status extras are drawn, in the object order of
    /// S-0102 claims 1, 6 and 7 (S-0111 claim 17 and State and ordering):
    /// the underground and surface lists, the ground layer, then the air
    /// and top lists. The ground layer's mobiles keep its order; each
    /// building on the map joins them by its sort key (S-0102 claim 5, F4).
    fn status_order(&self, inputs: &Inputs) -> Vec<ObjectId> {
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
        let mut buildings: Vec<(i64, ObjectId)> = self
            .world
            .objects
            .values()
            .filter(|o| o.kind == SimKind::Building && o.on_map)
            .map(|o| {
                let c = o.pos.cell();
                let voxel = self
                    .types
                    .get(&(o.kind, o.type_index))
                    .copied()
                    .flatten()
                    .is_some_and(|i| {
                        super::build::rule_bool(
                            inputs.import,
                            Kind::Buildings,
                            i,
                            b"TurretAnimIsVoxel",
                            false,
                        )
                    });
                let gate = self
                    .rules
                    .buildings
                    .get(o.type_index as usize)
                    .is_some_and(|b| b.gate);
                // spec: S-0102 claim 5 and F4 (a building's key: X + Y of its top-left foundation cell's corner, + 32 for a voxel turret, − 16 for a gate)
                let key = i64::from(256 * (c.x + c.y)) + if voxel { 32 } else { 0 }
                    - if gate { 16 } else { 0 };
                (key, o.id)
            })
            .collect();
        buildings.sort();
        let mut v = upper(&|l| l < Layer::Ground);
        let mut b = buildings.into_iter().peekable();
        // spec: S-0102 claim 6 (the ground layer in increasing sort key); design: at equal keys a building goes before the layer's mobiles, buildings among themselves in object ID order (our engine keeps no entering order for buildings)
        for (e, key) in &self.ground.list {
            let Entrant::Object(id) = *e else {
                continue;
            };
            while let Some((_, bid)) = b.next_if(|(k, _)| k <= key) {
                v.push(bid);
            }
            v.push(id);
        }
        v.extend(b.map(|(_, id)| id));
        v.extend(upper(&|l| l > Layer::Ground));
        // Objects of no list yet (none in a synced game): last, in object ID order.
        let listed: BTreeSet<ObjectId> = v.iter().copied().collect();
        let rest: Vec<ObjectId> = self
            .world
            .objects
            .keys()
            .copied()
            .filter(|id| !listed.contains(id))
            .collect();
        v.extend(rest);
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
            // spec: S-0402 claim 5a, F8 (ν: the walk-animation counter of S-0105 claim 16, the
            // view's walk counter of S-0102 claim 22, design Q-2176)
            harvesting: super::harvesting::of(w, id, self.walks.of(id).counter as i32),
            // spec: S-0102 claim 22, S-0105 claim 16 (the walk counter and the moving state of the shown tick)
            walk: self.walks.of(id),
            // spec: S-0105 claims 40 and 44 (an aircraft's anchor; a hunter-seeker vehicle type's is claim 40's P too)
            shadow: (kind == Kind::Aircraft || kind == Kind::Vehicles).then(|| shadow_anchor(w, o)),
            // spec: S-0105 claim 12 (alternative body: a spawner none of whose spawned aircraft is docked)
            alternative: glasnost_sim::spawn::any_docked(w, id) == Some(false),
            attitude: attitude_with(w, o, &self.rocking),
            // spec: S-0104 claim 13 (Deactivated voxel vehicles) and S-0405 claims 13 and 20
            disabled: kind == Kind::Vehicles && glasnost_sim::econ::power::unit_disabled(w, id),
            // spec: S-0107 claim 19 and Terms (height above the surface; 0 on a bridge deck)
            above: w.height_above_surface(id),
            // spec: S-0513 claims 29 and 29b (the envelope's phase as the last tick's turns left it, F7 in that tick; the centre-cell shroud test)
            protection: self.protection_look(o),
            generic_shadow: (kind == Kind::Infantry)
                .then(|| generic_shadow(w, o))
                .flatten(),
            // spec: S-0610 claim 7 (berserk vehicles and infantry; the infantry's cell centre shrouded for the viewer)
            berserk: glasnost_sim::psychic::berserk(w, id).then(|| {
                self.viewer
                    .is_some_and(|v| !glasnost_sim::econ::shroud::explored(w, v, o.pos.cell()))
            }),
            // spec: S-0105 claim 43 and S3 (the selected turret index as the last tick's turns left it, from the type's imported turret table)
            turret_index: super::build::selected_turret_index(
                w,
                id,
                self.shown_tick(),
                (kind == Kind::Vehicles)
                    .then(|| inputs.import.turret_tables.get(&idx))
                    .flatten(),
            ),
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
        self.draw_target_lines(inputs, scene, &mut by_code);
        // Buildings the game removed leave the cell pass and the ground
        // layer; so do the map's gates, which are drawn live.
        let mut gone: BTreeSet<u32> = self
            .building_ids
            .iter()
            .filter(|(_, id)| self.world.obj(**id).is_none_or(|o| !o.on_map))
            .map(|(i, _)| *i)
            .collect();
        gone.extend(self.live_gates.iter().copied());
        // The map's buildings while protected, flashing or marked are drawn
        // live too, so that their bodies show the protection appearance
        // (S-0102 claim 34), the charge flash (S-0519 claim 25: each on/off
        // change repaints the body) and the mark (claim 26: every frame);
        // our placeholder order, as for the gates. So are those being
        // erased, so that their bodies show half see-through (S-0514 claim
        // 27).
        let protected_map: BTreeSet<u32> = self
            .building_ids
            .iter()
            .filter(|(_, id)| {
                self.world.obj(**id).is_some_and(|o| {
                    o.on_map && (self.building_look(o).active() || self.erased_code(o.id).is_some())
                })
            })
            .map(|(code, _)| *code)
            .collect();
        gone.extend(protected_map.iter().copied());
        // The map's buildings that no longer look as the cell pass drew
        // them are drawn live too (our placeholder order, as for the
        // gates): one whose owner changed, so that it shows its owner's
        // house colour (S-0103 claim 8; an engineer capture's ownership
        // change, S-0509 claim 8), one whose health changed, so that its
        // body frame follows its damage (S-0108 claim 2), and one with a
        // running slot animation, so that the animation plays (S-0108
        // claim 11, S-0109 claims 1 and 5) instead of standing at its
        // first image frame. That includes a building that needs an
        // engineer and is not manned: its *freeze-unpowered* slots are held
        // at the image frame they were created with (S-0108 claim 26) by
        // the slot player, and its other slots play.
        let changed_map: BTreeSet<u32> = self
            .building_ids
            .iter()
            .filter(|(code, id)| {
                self.world.obj(**id).is_some_and(|o| {
                    o.on_map
                        && (self.map_start.get(code) != Some(&(o.owner, o.health))
                            || self
                                .slot_anims
                                .get(id)
                                .is_some_and(|s| !s.running.is_empty()))
                })
            })
            .map(|(code, _)| *code)
            .collect();
        gone.extend(changed_map.iter().copied());
        // Overlays that changed during play are drawn again from the game's
        // cells (S-0112 claims 1 to 9); their static items leave the cell pass.
        let grid = &self.world.cells.grid;
        let mut changed = self.changed_overlays.clone();
        // The cells changed before keep their place with today's values;
        // the others join when they differ from the start.
        for (c, v) in changed.iter_mut() {
            if let Some(cell) = grid.get(*c) {
                *v = (cell.overlay, cell.overlay_data);
            }
        }
        for &(c, i) in &self.legal_cells {
            let Some(cell) = grid.get(c) else { continue };
            let now = (cell.overlay, cell.overlay_data);
            if self.start_overlays.get(i) != Some(&now) {
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
        // The game's own buildings (deployed or placed), the map's gates and
        // the map's buildings drawn live (protected, changed or animated)
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
        // The map's buildings that keep their cell-pass drawing.
        let kept_map: BTreeSet<ObjectId> = self
            .building_ids
            .iter()
            .filter(|(code, _)| {
                !self.live_gates.contains(code)
                    && !protected_map.contains(code)
                    && !changed_map.contains(code)
            })
            .map(|(_, id)| *id)
            .collect();
        let own: Vec<(ObjectId, LiveBuilding)> = self
            .world
            .objects
            .values()
            .filter(|o| o.kind == SimKind::Building && o.on_map && !kept_map.contains(&o.id))
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
                        // spec: S-0513 claims 29 and 29b (the envelope's phase as the last tick's turns left it, F7 in that tick; the centre-cell shroud test)
                        protection: self.protection_look(o),
                        // spec: S-0519 claims 25 and 26, F7 (the flash count and the mark)
                        adjust: self.building_look(o).adjust,
                    },
                ))
            })
            .collect();
        if !own.is_empty() {
            let (ids, own): (Vec<ObjectId>, Vec<LiveBuilding>) = own.into_iter().unzip();
            let cells = self.protected_cells();
            self.painter.set_protected_cells(cells);
            let (drawn, notes) = self.painter.buildings(inputs, scene, &own);
            self.painter.set_protected_cells(BTreeMap::new());
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
        let mut particle_items = self.particle_items(inputs, scene);
        particle_items.extend(self.projectile_items(inputs, scene));
        particle_items.extend(self.piece_items(inputs, scene));
        scene.spots.clear();
        let waves = self.lines.wave_spots(self.shown_tick(), self.rules.mode);
        // spec: S-0102 claims 1 and 7 (underground and surface before the ground layer, air and
        // top after it; each unsorted)
        self.draw_layers(
            scene,
            &anim_items,
            &particle_items,
            &mut by_code,
            &[],
            |l| l < super::slots::Layer::Ground,
        );
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
                Entrant::Layered(_)
                | Entrant::Particle(_)
                | Entrant::Projectile(_)
                | Entrant::Piece(_) => {}
            }
        }
        self.draw_layers(
            scene,
            &anim_items,
            &particle_items,
            &mut by_code,
            &waves,
            |l| l > super::slots::Layer::Ground,
        );
        self.objects_end = scene.items.len();
        self.draw_effects(scene);
        self.draw_lines(inputs, scene);
        self.draw_status(inputs, scene);
    }

    /// The beams and beam flashes of the shown tick (S-0518 claims 19 to
    /// 23, S-0514 claims 24 to 26), after all layers and the status extras
    /// (S-0102 claim 2): lasers first, then beam flashes (S-0514 claim 26).
    fn draw_lines(&mut self, inputs: &Inputs, scene: &mut Scene) {
        use glasnost_render::light::DetailLevel;
        use glasnost_render::spots::SpotGroup;
        // spec: S-0518 Import mapping (the detail level of S-0104: 0, 1 or 2)
        let detail = match inputs.options.detail {
            DetailLevel::Low => 0,
            DetailLevel::Medium => 1,
            DetailLevel::High => 2,
        };
        let now = self.shown_tick();
        let quirk = self.rules.quirks.ranged_draw_half_span;
        let cosmetic = &mut self.cosmetic;
        // spec: S-0518 claim 21 (spread draws from the unsynchronised stream: our cosmetic stream)
        let mut draw = |a: i32, b: i32| cosmetic.ranged(a, b, quirk);
        let (beams, flashes) = self.lines.spots(now, detail, self.rules.mode, &mut draw);
        // spec: S-0316 claims 12 and 13 (the bolts after the lasers' draws and before the beam flashes)
        let bolts = self.lines.bolt_spots(
            now,
            self.battlefield,
            self.rules.quirks.table_root_distance,
            self.rules.mode,
            &mut draw,
        );
        self.counts.bolt_pixels = bolts.len();
        // spec: S-0316 claim 12 (in the effects group after lasers, before trails and beam flashes)
        for spots in [beams, bolts, flashes] {
            if !spots.is_empty() {
                // spec: S-0102 claim 2 (beam effects after all five layers and after the status extras)
                scene.spots.push(SpotGroup {
                    at: usize::MAX,
                    group: glasnost_render::scene::Group::Effects,
                    provisional: false,
                    spots,
                });
            }
        }
    }

    /// The items of every particle with a sprite (S-0613, S-0616; the
    /// interim rule of Q-1520), by system and then in each system's
    /// particle order (Q-1520 question 7 is open).
    fn particle_items(
        &mut self,
        inputs: &Inputs,
        scene: &mut Scene,
    ) -> BTreeMap<ObjectId, Vec<Item>> {
        use super::build::ParticleDraw;
        use super::particles as look;
        let detail = inputs.options.detail;
        let m = self.world.rules().mode;
        let mut list = Vec::new();
        let mut ids = Vec::new();
        for s in self.world.gas.systems.values() {
            for p in &s.particles {
                let Some(t) = self.rules.gas.particles.get(p.type_index as usize) else {
                    continue;
                };
                // spec: S-0616 claim 23 (every particle in play, the dead-marked too, unless the lowest detail hides its kind; by its own kind, not its system's)
                if !look::shown(t.kind, detail) {
                    continue;
                }
                let pos = (p.pos.x, p.pos.y, p.pos.z);
                let draw = if look::is_pixel(t.kind) {
                    // spec: S-0616 claims 28 and 29, F16 (a pixel kind draws its current colour; never its sprite, translucency or stage)
                    let rgb = p.colour.drawn(&t.look, m);
                    ParticleDraw::Pixel { rgb, pos }
                } else {
                    // spec: S-0616 claim 23 (a sprite kind whose type has no sprite draws nothing)
                    let Some(Some((e, true))) = self.particle_table.get(p.type_index as usize)
                    else {
                        continue;
                    };
                    // spec: S-0616 F15 (the screen pixels of the creation position and the target point, height lift included)
                    let group = if t.kind == Some(glasnost_sim::gas::Behaviour::Fire) {
                        look::fire_group(
                            project_lifted(p.origin.x, p.origin.y, p.origin.z),
                            project_lifted(p.aim.x, p.aim.y, p.aim.z),
                            m,
                        )
                    } else {
                        0
                    };
                    ParticleDraw::Sprite {
                        idx: e.idx,
                        frame: look::frame(t.kind, p.stage, t.end_stage, group),
                        // spec: S-0616 claim 26 (the particle's own translucency value)
                        code: look::translucency(p.translucency, detail),
                        pos,
                    }
                };
                ids.push(p.id);
                list.push(draw);
            }
        }
        let (drawn, notes) = self.painter.particles(inputs, scene, &list);
        self.notes.extend(notes);
        self.counts.particles_drawn = drawn.iter().filter(|i| !i.is_empty()).count();
        ids.into_iter().zip(drawn).collect()
    }

    /// The items of every projectile in flight (S-0306 claims 28 to 32), by
    /// object number; empty for one that draws nothing.
    fn projectile_items(
        &mut self,
        inputs: &Inputs,
        scene: &mut Scene,
    ) -> BTreeMap<ObjectId, Vec<Item>> {
        use super::build::{ProjectileDraw, ProjectilePalette};
        use super::projectiles as look;
        let house_name = |w: &World, h: Option<HouseId>| {
            h.and_then(|h| w.houses.get(usize::from(h.0)))
                .map(|h| h.name.as_bytes().to_vec())
                .unwrap_or_default()
        };
        let mut list = Vec::new();
        let mut ids = Vec::new();
        for p in self.world.projectiles.values() {
            // spec: S-0306 claim 28 (nothing for a projectile waiting for its detonation animation)
            if p.nuke_wait.is_some() {
                continue;
            }
            let Some((idx, l)) = self.projectile_look(p.weapon).cloned() else {
                continue;
            };
            // spec: S-0306 claim 28 (nothing at all for an invisible projectile)
            if l.inviso {
                continue;
            }
            // spec: S-0306 F14 (ψ and e from the velocity after its latest turn)
            let facings = glasnost_sim::combat::projectile::drawn_facings(&self.world, p.vel);
            let palette = if l.anim_palette {
                // spec: S-0306 claim 31 case 1 (the anim-palette flag first)
                ProjectilePalette::Anim
            } else if l.firers_palette {
                // spec: S-0306 claim 31 case 2 (the house that owned the firer at creation; without a
                // firer the local player's house at each drawing; our viewer with no local player
                // takes the projectile's attacking house)
                let house = if p.firer.is_some() {
                    p.firer_house
                } else {
                    self.viewer.or(p.house)
                };
                ProjectilePalette::House(house_name(&self.world, house))
            } else {
                // spec: S-0306 claim 31 case 3 (the general palette)
                ProjectilePalette::General
            };
            // spec: S-0306 claims 30 and 32 (a sprite projectile with the shadow flag while η > 0; a voxel one never)
            let shadow = (l.shadow && !l.voxel)
                .then(|| look::shadow(&self.world, p.pos))
                .flatten();
            list.push(ProjectileDraw {
                idx,
                pos: (p.pos.x, p.pos.y, p.pos.z),
                facings,
                // spec: S-0306 claim 29 (chosen afresh in every drawn frame from the state after its latest turn)
                frame: look::sprite_frame(&l, facings.0, p.turns),
                palette,
                shadow,
            });
            ids.push(p.id);
        }
        let (drawn, notes) = self.painter.projectiles(inputs, scene, &list);
        self.notes.extend(notes);
        self.projectiles_drawn = ids
            .iter()
            .zip(&drawn)
            .filter(|(_, i)| !i.is_empty())
            .map(|(id, _)| *id)
            .collect();
        ids.into_iter().zip(drawn).collect()
    }

    /// The items of every debris piece in play (S-0317 claims 15 and 22 to
    /// 28), by object number; empty for one that draws nothing.
    fn piece_items(&mut self, inputs: &Inputs, scene: &mut Scene) -> BTreeMap<ObjectId, Vec<Item>> {
        use super::build::PieceDraw;
        let mut list = Vec::new();
        let mut ids = Vec::new();
        for (id, p) in &self.world.pieces {
            let Some(idx) = self
                .piece_table
                .get(p.type_index as usize)
                .copied()
                .flatten()
            else {
                continue;
            };
            let owner = p
                .house
                .and_then(|h| self.world.houses.get(usize::from(h.0)))
                .map(|h| h.name.as_bytes().to_vec())
                .unwrap_or_default();
            // spec: S-0317 Terms (the cell of a position: (trunc(x / 256), trunc(y / 256)))
            let cell = p.pos.cell();
            list.push(PieceDraw {
                id: sim_code(*id),
                idx,
                pos: (p.pos.x, p.pos.y, p.pos.z),
                axis: p.axis,
                spin: p.spin.0,
                turns: p.turns,
                owner,
                // spec: S-0317 claims 20 and 27 (the ground height at the position, 0 off the legal map)
                ground: self.world.surface_at(p.pos.x, p.pos.y),
                bridge: self.world.high_bridge(cell),
                on_map: self.world.cells.cell(cell).is_some(),
            });
            ids.push(*id);
        }
        let (drawn, notes) = self.painter.pieces(inputs, scene, &list);
        self.notes.extend(notes);
        self.pieces_drawn = ids
            .iter()
            .zip(&drawn)
            .filter(|(_, i)| !i.is_empty())
            .map(|(id, _)| *id)
            .collect();
        ids.into_iter().zip(drawn).collect()
    }

    /// For the lanes: the debris pieces in play and those that drew a
    /// model in the last frame.
    pub fn piece_counts(&self) -> (usize, usize) {
        (self.world.pieces.len(), self.pieces_drawn.len())
    }

    /// For the lanes: the electric bolts still shown and how many of them
    /// are attached (S-0316 claims 4 and 5).
    pub fn bolt_counts(&self) -> (usize, usize) {
        let b = &self.lines.bolts;
        (b.len(), b.iter().filter(|b| b.attached).count())
    }

    /// For the lanes: the projectiles that drew a body in the last frame and
    /// the vehicles whose target-line timer runs.
    pub fn projectile_counts(&self) -> (usize, usize) {
        (self.projectiles_drawn.len(), self.target_lines.len())
    }

    /// Appends each vehicle's target line to its picture (S-0301 claim 24 b
    /// to h): while 0 ≤ *t* − *t*₀ < 15, the vehicle is drawn and it has a
    /// target, a dotted line from its turret pivot to the led aim point of
    /// its current target, after everything else its drawing step draws.
    fn draw_target_lines(
        &mut self,
        inputs: &Inputs,
        scene: &mut Scene,
        by_code: &mut BTreeMap<u32, GroundEntry>,
    ) {
        let t = self.shown_tick();
        let world = &self.world;
        // A timer past its 15 ticks, or of a vehicle no longer in the game, has nothing more to draw.
        self.target_lines
            .retain(|id, t0| t - *t0 < TARGET_LINE_TICKS && world.obj(*id).is_some());
        let lines: Vec<(ObjectId, i32)> = self.target_lines.iter().map(|(k, v)| (*k, *v)).collect();
        for (id, t0) in lines {
            // spec: S-0301 claim 24 b (drawn in every drawn frame with 0 ≤ t − t₀ < 15)
            if !(0..TARGET_LINE_TICKS).contains(&(t - t0)) {
                continue;
            }
            let Some(o) = self.world.obj(id) else {
                continue;
            };
            // spec: S-0301 claim 24 c (only while the vehicle has a target at the moment of drawing)
            let Some(target) = o.target else {
                continue;
            };
            // spec: S-0301 claim 24 c (no line when the vehicle's own drawing step does not run)
            let Some(entry) = by_code
                .get_mut(&sim_code(id))
                .filter(|e| !e.items.is_empty())
            else {
                continue;
            };
            // spec: S-0301 claim 24 d (the turret pivot, recomputed in every drawn frame)
            let a = glasnost_sim::combat::position::turret_pivot(&self.world, id);
            // spec: S-0301 claim 24 e (the led aim point of the current target, recomputed in every drawn frame)
            let b = glasnost_sim::combat::projectile::led_aim_point(&self.world, id, target);
            // spec: S-0301 claim 24 f (both projected as every object's position, height lift included)
            let start = project_lifted(a.x, a.y, a.z);
            let end = project_lifted(b.x, b.y, b.z);
            // spec: S-0301 claim 24 g (the dots swap from each tick to the next; which phase a tick
            // draws is low confidence there, ours: the tick number's parity)
            let phase = t.rem_euclid(2) == 1;
            let items = self.painter.target_line(inputs, scene, start, end, phase);
            // spec: S-0301 claim 24 h (right after its vehicle's picture, in its place in the drawing order)
            entry.items.extend(items);
        }
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
        for p in self.world.projectiles.values() {
            // spec: S-0102 claim 5 (X + Y of the position)
            keys.insert(
                Entrant::Projectile(p.id),
                i64::from(p.pos.x) + i64::from(p.pos.y),
            );
        }
        for (id, p) in &self.world.pieces {
            // spec: S-0102 claim 5 (X + Y of the position)
            keys.insert(Entrant::Piece(*id), i64::from(p.pos.x) + i64::from(p.pos.y));
        }
        for (i, (_, g)) in self.static_layered.iter().enumerate() {
            keys.insert(Entrant::Layered(i), g.key);
        }
        keys
    }

    /// Draws the layers other than ground that `which` picks, layer by
    /// layer in drawing order, each in its list's order (S-0102 claims 1,
    /// 3, 4 and 7; S-0616 claim 30): the static entries, the playing
    /// animations (`anims`, by serial number), the particles and projectiles
    /// (`particles`, by object number) and the mobile objects (`mobiles`, taken out as drawn); the
    /// magnetic waves (`waves`, with the next animation serial at their
    /// creation) in the air layer (S-0607 claim 26).
    fn draw_layers(
        &self,
        scene: &mut Scene,
        anims: &BTreeMap<u32, (super::slots::Layer, Vec<Item>)>,
        particles: &BTreeMap<ObjectId, Vec<Item>>,
        mobiles: &mut BTreeMap<u32, GroundEntry>,
        waves: &[(u32, Vec<glasnost_render::spots::Spot>)],
        which: impl Fn(super::slots::Layer) -> bool,
    ) {
        use super::slots::Layer;
        use glasnost_render::spots::SpotGroup;
        let mut layers: Vec<Layer> = self.layers.upper().filter(|l| which(*l)).collect();
        // spec: S-0607 claim 26 (a magnetic wave is an object of the air layer)
        let wave_layer = (!waves.is_empty() && which(Layer::Air)).then_some(Layer::Air);
        if let Some(l) = wave_layer
            && !layers.contains(&l)
        {
            layers.push(l);
            layers.sort();
        }
        let wave_group = |scene: &mut Scene, spots: &Vec<glasnost_render::spots::Spot>| {
            scene.spots.push(SpotGroup {
                at: scene.items.len(),
                group: glasnost_render::scene::Group::Effects,
                provisional: false,
                spots: spots.clone(),
            });
        };
        for layer in layers {
            let list = self.layers.list(layer);
            let slots = if wave_layer == Some(layer) {
                wave_slots(list, waves.iter().map(|(after, _)| *after))
            } else {
                Vec::new()
            };
            for (i, e) in list.iter().enumerate() {
                for ((_, spots), _) in waves.iter().zip(&slots).filter(|(_, s)| **s == i) {
                    wave_group(scene, spots);
                }
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
                    Entrant::Particle(id) | Entrant::Projectile(id) | Entrant::Piece(id) => {
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
            for ((_, spots), _) in waves.iter().zip(&slots).filter(|(_, s)| **s == list.len()) {
                wave_group(scene, spots);
            }
        }
    }

    /// Draws the combat placeholders after the object pass: projectiles in
    /// flight (invisible ones are not drawn) and the shots, detonations and
    /// animations still shown. Our own presentation, provisional.
    fn draw_effects(&mut self, scene: &mut Scene) {
        scene.items.truncate(self.objects_end);
        let rules = self.rules.clone();
        // S-0306 claim 28: the game view draws only the projectile's art; the debug-markers view
        // marks a projectile that drew nothing (no stand-in picture in the game view).
        for p in self.world.projectiles.values() {
            let kind = rules.combat.projectile_of(p.weapon);
            if kind.is_some_and(|k| k.inviso)
                || !self.debug_markers
                || self.projectiles_drawn.contains(&p.id)
            {
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
        // S-0616 claims 23 to 28: the game view draws particles in the air layer; the
        // debug-markers view marks a sprite-kind particle whose type has no sprite, a placeholder
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
                    let pixel = rules
                        .gas
                        .particles
                        .get(p.type_index as usize)
                        .is_some_and(|t| super::particles::is_pixel(t.kind));
                    if sprite || pixel {
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

    /// Sets the digit each control-group member shows (S-0111 claim 17).
    /// Returns whether it changed; then call [`Live::redraw_status`].
    pub fn set_group_digits(&mut self, digits: BTreeMap<ObjectId, u8>) -> bool {
        if self.group_digits == digits {
            return false;
        }
        self.group_digits = digits;
        true
    }

    /// Sets each house's display RGB, in house order (S-0103 claim 15),
    /// which the control-group number is drawn in.
    pub fn set_house_colours(&mut self, colours: Vec<[u8; 3]>) {
        self.house_colours = colours;
    }

    /// Sets the game font the control-group number is drawn with (S-0111
    /// claim 17 and Import mapping); `None` for a missing or rejected font
    /// (S-0014 claim 6), and then no number and no box is drawn.
    pub fn set_group_font(&mut self, font: Option<combat::GroupFont>) {
        if self.group_font != font {
            self.group_font = font;
            self.cache.forget_group_digits();
        }
    }

    /// Whether the pip scale of object `o` is drawn with its health bar
    /// for the viewer (S-0111 claims 9 and 11): an owner that is the
    /// viewer's house or allied with it, or for a building whose type draws
    /// its pips for all or can be occupied; never for a building whose
    /// pip-scale kind is resource while `quirks.building_resource_pips_hidden`
    /// is on. With no viewer every owner passes.
    fn pip_scale_shown(&self, inputs: &Inputs, o: &Object) -> bool {
        let building = o.kind == SimKind::Building;
        let import_idx = self.types.get(&(o.kind, o.type_index)).copied().flatten();
        let key = |k: &[u8]| -> Option<&[u8]> {
            let (kind, i) = (import_kind(o.kind)?, import_idx?);
            super::build::rule(inputs.import, kind, i, k)
        };
        let flag = |k: &[u8]| key(k).and_then(glasnost_ini::value::boolean);
        // spec: S-0111 claim 11 and Edge cases (quirks.building_resource_pips_hidden: a building whose pip-scale kind is resource shows no pip scale at all, no group number included; Import mapping PipScale, absent none)
        if building
            && self.view_quirks.building_resource_pips_hidden
            && pip_scale::pip_kind(key(b"PipScale")) == pip_scale::PipKind::Resource
        {
            return false;
        }
        let Some(v) = self.viewer else {
            return true;
        };
        // spec: S-0111 claim 9 (the owner is the local player's house or allied with it)
        if o.owner == Some(v) || self.world.allied(Some(v), o.owner) {
            return true;
        }
        // spec: S-0111 claim 9 and Import mapping (a building only: PipsDrawForAll or CanBeOccupied, both absent no)
        building && (flag(b"PipsDrawForAll") == Some(true) || flag(b"CanBeOccupied") == Some(true))
    }

    /// The rows of an object's pip scale and its self-heal marker (S-0111
    /// claims 10 to 16), in drawing order: the rows in the order of claim
    /// 11's table, each covering the rows above it, then the marker.
    fn pip_rows(&self, inputs: &Inputs, o: &Object) -> Vec<combat::Pip> {
        use pip_scale::{Body, Heal, Passenger, PipKind};
        let w = &self.world;
        let r = w.rules();
        let mode = r.mode;
        let import_of = |kind: SimKind, index: u32| {
            let i = self.types.get(&(kind, index)).copied().flatten()?;
            Some((import_kind(kind)?, i))
        };
        let key_of = |kind: SimKind, index: u32, k: &[u8]| -> Option<&[u8]> {
            let (ik, i) = import_of(kind, index)?;
            super::build::rule(inputs.import, ik, i, k)
        };
        let key = |k: &[u8]| key_of(o.kind, o.type_index, k);
        let body = match o.kind {
            SimKind::Building => Body::Building,
            SimKind::Infantry => Body::Infantry,
            _ => Body::Vehicle,
        };
        let infantry = body == Body::Infantry;
        let building = (body == Body::Building).then_some(o.foundation);
        let origin = combat::pip_origin(building);
        let tc = glasnost_sim::combat::veterancy::type_combat(w, o.id);
        let unit = r.unit(o.kind, o.type_index);
        let bt = building.and_then(|_| r.buildings.get(o.type_index as usize));
        let kind = pip_scale::pip_kind(key(b"PipScale"));
        // spec: S-0111 Import mapping (Passengers: the passenger capacity, as the transport specs give; a building's is its absorbing capacity, S-0605)
        let capacity = match bt {
            Some(b) => b.absorb.capacity,
            None => unit.map_or(0, |u| u.passengers),
        };
        // spec: S-0111 claim 13 and Import mapping (M: the damage of the type's primary weapon; none without one)
        let primary = tc
            .and_then(|t| t.slots[0])
            .map(|wp| r.combat.weapon(wp).damage);
        let ammo_cap = tc.map_or(0, |t| t.ammo);
        let len = pip_scale::row_length(kind, infantry, ammo_cap, capacity, primary);
        let mut out = Vec::new();
        // spec: S-0111 claim 11 (spawned aircraft: a type that spawns aircraft, whatever else applies)
        if let Some(spawn) = o.spawn.as_ref() {
            use glasnost_sim::spawn::SlotState;
            // spec: S-0111 claim 11 (a slot counts as full while docked or reloading, S-0506 claim 4's slot states; on the rail, preparing, attacking, returning and empty do not)
            let full: Vec<bool> = spawn
                .slots
                .iter()
                .map(|sl| matches!(sl.state, SlotState::Docked | SlotState::Reloading))
                .collect();
            out.extend(pip_scale::places(
                body,
                origin,
                &pip_scale::spawn_row(&full),
            ));
        }
        // spec: S-0111 claim 11 and Import mapping (occupants: a building type that shows its occupant pips, ShowOccupantPips absent yes, with at least one occupant slot)
        let slots = bt.map_or(0, |b| b.building.max_occupants);
        if body == Body::Building
            && key(b"ShowOccupantPips")
                .and_then(glasnost_ini::value::boolean)
                .unwrap_or(true)
            && slots >= 1
        {
            let frames: Vec<usize> = o
                .structure
                .occupants
                .iter()
                .filter_map(|&id| w.obj(id))
                .map(|p| {
                    // spec: S-0111 claim 12 and Import mapping (the occupying infantry type's OccupyPip, the held value, initially 7)
                    pip_scale::pip_frame(
                        key_of(p.kind, p.type_index, b"OccupyPip"),
                        pip_scale::OCCUPY_PIP_INITIAL,
                    )
                })
                .collect();
            out.extend(pip_scale::places(
                body,
                origin,
                &pip_scale::occupant_row(slots, &frames),
            ));
        }
        let passenger = |id: ObjectId| -> Option<(Passenger, glasnost_num::F64)> {
            let p = w.obj(id)?;
            let size = r.unit(p.kind, p.type_index).map(|u| u.size);
            let as_pip = match p.kind {
                // spec: S-0111 claim 12 and Import mapping (an infantry passenger's colour: its type's Pip, the held value, initially 1)
                SimKind::Infantry => Passenger::Infantry(pip_scale::pip_frame(
                    key_of(p.kind, p.type_index, b"Pip"),
                    pip_scale::PIP_INITIAL,
                )),
                SimKind::Vehicle => Passenger::Vehicle,
                _ => Passenger::Other,
            };
            Some((as_pip, size.unwrap_or(glasnost_num::F64::ZERO)))
        };
        if capacity > 0 {
            // spec: S-0111 claim 11 (passengers: a type with a passenger capacity above 0, whatever its pip-scale kind)
            let frames = if bt.is_some_and(|b| b.absorb.absorbing()) {
                // spec: S-0111 claim 12 (absorbing buildings, the front the most recent, S-0605 Terms)
                let recent: Vec<Passenger> = o
                    .structure
                    .absorbed
                    .iter()
                    .filter_map(|&id| passenger(id).map(|(p, _)| p))
                    .collect();
                pip_scale::absorbed_row(len, &recent)
            } else {
                // spec: S-0111 claim 12 and S-0507 claim 3 (the order of entry: position 1 is the most recent, so the passenger order reversed)
                let entered: Vec<(Passenger, glasnost_num::F64)> = o
                    .transport
                    .passengers
                    .iter()
                    .rev()
                    .filter_map(|&id| passenger(id))
                    .collect();
                pip_scale::passenger_row(len, &entered, mode)
            };
            let gunner = tc.is_some_and(|t| t.gunner);
            out.extend(frames.iter().zip(0..).map(|(&f, j)| {
                pip_scale::place(
                    body,
                    origin,
                    pip_scale::passenger_place(gunner, j),
                    f,
                    false,
                )
            }));
        } else {
            // spec: S-0111 claim 11 (by pip-scale kind, for a passenger capacity of 0 or below: mind control claim 13, resource claim 14, ammunition claim 15, none and power no row)
            match kind {
                PipKind::MindControl => {
                    let c = glasnost_sim::mind::controller(w, o.id);
                    let count = c.map_or(0, |c| c.victims.len());
                    // spec: S-0111 claim 13 (a type that may control without limit and controls more than M shows one more pip; highlighted while the controller's pip highlight runs, S-0601 claim 16)
                    let overload = c
                        .filter(|c| {
                            c.unlimited && i64::try_from(count).unwrap_or(i64::MAX) > i64::from(len)
                        })
                        .map(|_| glasnost_sim::mind::highlight(w, o.id) > 0);
                    out.extend(
                        pip_scale::mind_row(len, count, overload)
                            .into_iter()
                            .map(|(j, f)| pip_scale::place(body, origin, j, f, false)),
                    );
                }
                PipKind::Resource => {
                    let load = o
                        .econ
                        .harvest
                        .as_ref()
                        .map(|h| h.load.as_slice())
                        .or_else(|| o.econ.slave.as_ref().map(|s| s.load.as_slice()))
                        .unwrap_or(&[]);
                    let h = &r.econ.harvest;
                    // spec: S-0111 claim 14 and Import mapping (C: the type's Storage, as the economy specs give: absent 0, S-0402 and S-0309 Import mapping)
                    let storage = match o.kind {
                        SimKind::Vehicle => {
                            h.vehicles.get(o.type_index as usize).map(|u| u.capacity)
                        }
                        SimKind::Infantry => {
                            h.infantry.get(o.type_index as usize).map(|u| u.capacity)
                        }
                        _ => None,
                    }
                    .unwrap_or_else(|| {
                        key(b"Storage")
                            .and_then(glasnost_ini::value::integer)
                            .unwrap_or(0)
                    });
                    out.extend(pip_scale::places(
                        body,
                        origin,
                        &pip_scale::resource_row(len, load, storage, mode),
                    ));
                }
                PipKind::Ammo => {
                    let wrap = tc.map_or(0, |t| t.pip_wrap);
                    out.extend(
                        pip_scale::ammo_row(ammo_cap, o.combat.ammo, wrap)
                            .into_iter()
                            .map(|(j, f)| pip_scale::place(body, origin, j, f, true)),
                    );
                }
                PipKind::None | PipKind::Passengers | PipKind::Power => {}
            }
        }
        // spec: S-0111 claim 16 (vehicles and infantry only; infantry and organic vehicles the infantry healing ability, other vehicles unit repair; aircraft and buildings never)
        let heal = match o.kind {
            SimKind::Infantry => Some(Heal::Infantry),
            SimKind::Vehicle if tc.is_some_and(|t| t.organic) => Some(Heal::Infantry),
            SimKind::Vehicle => Some(Heal::Unit),
            _ => None,
        };
        if let Some(heal) = heal {
            let i = match heal {
                Heal::Infantry => 0,
                Heal::Unit => 1,
            };
            // spec: S-0111 claim 16 and S-0314 claim 10 (the owner has the ability: its running self-heal total of that kind is above 0)
            let has = o
                .owner
                .and_then(|h| w.houses.get(usize::from(h.0)))
                .is_some_and(|h| h.combat.self_heal[i] > 0);
            if has {
                let strength = unit.map_or(0, |u| u.strength);
                // spec: S-0111 claim 16 (T: the game-wide self-heal interval of the ability, Import mapping SelfHealInfantryFrames and SelfHealUnitFrames)
                let period = r.combat.general.self_heal_frames[i];
                out.push(pip_scale::self_heal(
                    infantry,
                    heal,
                    origin,
                    o.health < strength,
                    self.shown_tick(),
                    period,
                ));
            }
        }
        out
    }

    /// Redraws only the status extras (after [`Live::set_marks`]).
    pub fn redraw_status(&mut self, inputs: &Inputs, scene: &mut Scene) {
        self.draw_status(inputs, scene);
    }

    /// S-0111's status extras after the effects: every object's bomb icon
    /// (S-0519 claim 24) and repair wrench (claim 19), its veterancy
    /// insignia (claim 18), then the health bars of the selected objects
    /// (with the unit bracket) and of the hovered one (claims 3 to 7). Within
    /// an object the bar covers the insignia, and the pip scale's rows, the
    /// self-heal marker and the control-group number cover the bar (State
    /// and ordering). Each object's extras are complete before the next
    /// object's begin, in the object order of S-0102 (claim 17).
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
        // spec: S-0111 claim 17 and State and ordering (each object's status extras, the number included, are complete before the next object's begin, in the object order of S-0102 claims 1, 6 and 7)
        let order = self.status_order(inputs);
        for id in order {
            let Some(o) = self.world.obj(id) else {
                continue;
            };
            if !o.on_map || o.kind == SimKind::Terrain {
                continue;
            }
            let infantry = o.kind == SimKind::Infantry;
            // spec: S-0111 Terms (status anchor: the centre at its world height, projected; S-0204 claim 4 for a building's centre)
            let p = self.world.reference_point(o.id);
            let anchor = project_lifted(p.x, p.y, p.z);
            // spec: S-0111 claim 1 (status extras, the bomb icon too, are drawn only while the status anchor lies within the battlefield view widened by 360 pixels left and right and 180 above and below; fog does not hide them)
            if !combat::in_status_view(anchor, self.battlefield) {
                continue;
            }
            // spec: S-0519 claim 24 (the bomb icon is the first of its carrier's status extras, so its own extras paint over it)
            if let Some(b) = o.combat.bomb
                && self
                    .viewer
                    .is_none_or(|v| glasnost_sim::bomb::visible(&self.world, v, o.id))
            {
                let g = &r.combat.general.ivan;
                // spec: S-0519 F3 (the icon's image frame in the tick the picture shows, S-0201 claim 6)
                let frame = combat::bomb_frame(self.shown_tick(), b.planted, g.delay, g.flicker);
                let at = if o.kind == SimKind::Building {
                    // spec: S-0519 claim 24, S-0108 claim 8 (a building: the projected top corner of its foundation)
                    let c = o.pos.cell();
                    let level = self.world.cells.grid.get(c).map_or(0, |c| c.level);
                    project_lifted(
                        256 * c.x,
                        256 * c.y,
                        glasnost_render::projection::LEVEL_LEPTONS * i32::from(level),
                    )
                } else {
                    // spec: S-0519 claim 24 (a unit: the screen pixel of its position, height included)
                    project_lifted(o.pos.x, o.pos.y, o.pos.z)
                };
                match self
                    .cache
                    .bomb_item(scene, inputs.assets, at, frame, Some(sim_code(o.id)))
                {
                    Some(item) => items.push(item),
                    // spec: S-0519 Import mapping (missing bomb-icon image: no icon is drawn; the importer warns); S-0111 claim 19a (missing cursor palette: likewise, its own warning)
                    None => {
                        let note = if inputs.assets.get(combat::CURSOR_PALETTE).is_some() {
                            BOMB_ICON_MISSING
                        } else {
                            CURSOR_PALETTE_MISSING
                        };
                        self.notes.insert(note.to_owned());
                    }
                }
            }
            // spec: S-0111 claim 19 (a building under repair, its repair switched on, S-0408 claim 13 and S-0314 claim 8, shows the wrench, unless the cell of its position, the top-left cell of its foundation, is under shroud by S-0110 claim 8's point test at the centre of that cell at its surface height)
            if o.kind == SimKind::Building
                && o.econ.building.repairing
                && self
                    .viewer
                    .is_none_or(|v| !slot_cell_shrouded(&self.world, v, o.pos.cell()))
            {
                // spec: S-0111 F4 (the image frame in the tick the picture shows, S-0201 claim 6)
                let frame = combat::wrench_frame(
                    self.shown_tick(),
                    self.world.game_speed,
                    r.quirks.speed_normalised_delays,
                );
                // design: the wrench follows the bomb icon and precedes the insignia and health bar among the object's extras, so the bar and its marks cover it as they cover the bomb icon (S-0111 gives no order for the wrench)
                match self.cache.wrench_item(
                    scene,
                    inputs.assets,
                    anchor,
                    frame,
                    Some(sim_code(o.id)),
                ) {
                    Some(item) => items.push(item),
                    // spec: S-0111 Import mapping (a missing sprite draws nothing; the importer warns); claim 19a (missing cursor palette: likewise, its own warning)
                    None => {
                        let note = if inputs.assets.get(combat::CURSOR_PALETTE).is_some() {
                            WRENCH_MISSING
                        } else {
                            CURSOR_PALETTE_MISSING
                        };
                        self.notes.insert(note.to_owned());
                    }
                }
            }
            let bar = shows.iter().find(|(id, _)| *id == o.id).map(|(_, s)| *s);
            // spec: S-0111 claim 18 (no insignia while the unit appears disguised to the local player's house, S-0516 claims 35–36, even when drawn as its real type)
            let disguised = self.viewer.is_some()
                && glasnost_sim::cloak::disguise::appears_disguised(&self.world, o.id, self.viewer);
            // spec: S-0111 claim 18 (nor while cloak presentation hides the object from the viewer, S-0516 claim 28: a building at fade levels 11 to 15 by the fully cloaked rule, at 0 to 10 not suppressed), the viewer as for the object's own picture (apply_cloak)
            let hidden =
                glasnost_sim::cloak::view::look(&self.world, o.id, self.viewer.or(o.owner))
                    == glasnost_sim::cloak::view::Look::Hidden;
            let insignia =
                combat::insignia(o.combat.experience, infantry).filter(|_| !disguised && !hidden);
            if bar.is_none() && insignia.is_none() {
                continue;
            }
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
                    if selected {
                        // spec: S-0111 claim 8 (a selected building gets corner brackets instead of a bracket image: palette index 15, index 12 when its height above the terrain is below -4 leptons; design: the terrain is its cell's level height)
                        let level = self
                            .world
                            .cells
                            .grid
                            .get(o.pos.cell())
                            .map_or(0, |c| c.level);
                        let above =
                            p.z - glasnost_render::projection::LEVEL_LEPTONS * i32::from(level);
                        let index = if above < combat::CORNER_LOW {
                            combat::CORNER_INDEX_LOW
                        } else {
                            combat::CORNER_INDEX
                        };
                        items.extend(self.cache.corner_item(
                            scene,
                            inputs.assets,
                            anchor,
                            (w, d),
                            levels * combat::LEVEL_HEIGHT,
                            index,
                            Some(sim_code(o.id)),
                        ));
                    }
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
            // spec: S-0111 claims 9 and 11 and State and ordering (the pip scale is drawn with the health bar when claim 9 lets the viewer see it, after the bar, covering it)
            let pip_scale = bar.is_some() && self.pip_scale_shown(inputs, o);
            if pip_scale {
                pips.extend(self.pip_rows(inputs, o));
            }
            items.extend(self.cache.pip_items(
                scene,
                inputs.assets,
                anchor,
                &pips,
                Some(sim_code(o.id)),
            ));
            // spec: S-0111 claim 17 and State and ordering (the group number is drawn with the pip scale, so with the health bar, when claim 9 lets the viewer see the pip scale; drawn last among the object's own status extras, covering them)
            if pip_scale
                && let Some(&digit) = self.group_digits.get(&o.id)
                && let Some(font) = self.group_font.as_ref()
                // spec: S-0111 claim 17 (the owner's house display RGB of S-0103 claim 15; every object has an owning house in skirmish, so no ownerless case arises)
                && let Some(&colour) = o.owner.and_then(|h| self.house_colours.get(usize::from(h.0)))
            {
                let building = (o.kind == SimKind::Building).then_some(o.foundation);
                let at = combat::group_number_at(infantry, building);
                items.extend(self.cache.group_number_item(
                    scene,
                    font,
                    anchor,
                    at,
                    digit,
                    colour,
                    Some(sim_code(o.id)),
                ));
            }
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
                    Paint::Pixel(_) => "pixel",
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
                    "item {i} {:?} {paint} {}x{} at ({}, {}) depth {:?} test {:?}: {what}; path: {path}",
                    it.group,
                    img.width(),
                    img.height(),
                    it.x,
                    it.y,
                    it.depth,
                    it.test
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

/// Applies the cloak look `look` to an object's drawn items (S-0516
/// claims 28 and 29): not drawn, or blended 25 % or 50 %, and its shadow
/// only when `shadow`.
pub(super) fn cloak_items(
    look: glasnost_sim::cloak::view::Look,
    shadow: bool,
    erased: Option<glasnost_render::compose::Translucency>,
    items: &mut Vec<Item>,
) {
    use glasnost_render::compose::{Paint, Translucency};
    use glasnost_sim::cloak::view::Look;
    let blend = match look {
        Look::Normal => None,
        // spec: S-0516 claim 28 (25 % translucent: S-0102 code 1)
        Look::Translucent25 => Some(Translucency::Quarter),
        // spec: S-0516 claim 28 (50 % translucent: S-0102 code 2)
        Look::Translucent50 | Look::Rippling => Some(Translucency::Half),
        // spec: S-0516 claim 28 (not drawn); S-0513 claim 29b (a fully suppressed drawing: nothing)
        Look::Hidden => {
            items.clear();
            return;
        }
    };
    // spec: S-0516 claim 29 (no shadow while not uncloaked)
    if !shadow {
        items.retain(|i| i.paint != Paint::Shadow);
    }
    // spec: S-0514 claim 27 (an erased or phasing body's code combines with the cloak's, S-0109 Terms)
    let blend = combine_codes(blend, erased);
    if let Some(t) = blend {
        for i in items.iter_mut() {
            if let Paint::Lit(s) = i.paint {
                // spec: S-0513 claim 29b, S-0516 claim 28 (a protected object drawn translucent or
                // rippling keeps the protection brightness of its source pixels before the F8
                // blend and takes no mask; drawn opaque again, it takes its mask again)
                i.paint = Paint::Translucent(s.with_mask(0), t);
            }
        }
    }
}

/// A mobile object's ground-layer sort key, X + Y of its position (S-0102
/// claim 5).
pub fn mobile_key(o: &glasnost_sim::world::Object) -> i64 {
    // spec: S-0102 claim 5 (sort key X + Y in leptons; height plays no part)
    i64::from(o.pos.x) + i64::from(o.pos.y)
}

/// The note for a bomb icon whose image cannot be drawn (S-0519 Import
/// mapping).
pub const BOMB_ICON_MISSING: &str = "the bomb icon sprite BOMBCURS.SHP or its image frame is missing: no bomb icon is drawn (S-0519 claim 24, Import mapping)";

/// The note for a missing pip sprite (S-0111 Import mapping).
pub const PIPS_MISSING: &str = "the pip sprite PIPS.SHP is missing: no health pips, insignia, building pip rows or self-heal marker are drawn (S-0111 Import mapping)";

/// The note for a missing second pip sprite (S-0111 Import mapping).
pub const PIPS2_MISSING: &str = "the second pip sprite PIPS2.SHP is missing: no unit pip rows or ammunition pips are drawn (S-0111 claim 10, Import mapping)";

/// The note for a wrench that cannot be drawn (S-0111 Import mapping).
pub const WRENCH_MISSING: &str = "the wrench sprite WRENCH.SHP or its image frame is missing: no repair wrench is drawn (S-0111 claim 19, Import mapping)";

/// The note for a missing cursor palette, given once (S-0111 claim 19a and
/// Edge cases: our engine's departure, no switch).
pub const CURSOR_PALETTE_MISSING: &str = "the cursor palette MOUSEPAL.PAL is missing: neither the repair wrench nor the bomb icon is drawn (S-0111 claim 19a)";

/// The person's house of a setup: the first player house without an AI.
pub fn person_house(setup: &GameSetup) -> Option<HouseId> {
    setup
        .houses
        .iter()
        .position(|h| !h.ai && h.role == glasnost_sim::world::HouseRole::Player)
        .map(|i| HouseId(i as u8))
}

/// The local house of a client game on `setup` chosen as `choice`: the
/// house asked for (a person's house unless the game is watched), else in
/// a watched game the person's house or the first player house, else the
/// person's house, `None` when the setup has none (a watch game from no
/// house). The session and the loader's wall colour (#94) both use it.
pub fn local_of(
    setup: &GameSetup,
    choice: super::build::LocalChoice,
) -> Result<Option<HouseId>, String> {
    Ok(match choice.house {
        Some(h)
            if setup
                .houses
                .get(usize::from(h.0))
                .is_some_and(|x| choice.watch || !x.ai) =>
        {
            Some(h)
        }
        Some(h) => return Err(format!("house {} is no person's house", h.0)),
        None if choice.watch => {
            // A watched game's first player house (M8 step C4).
            let first = setup
                .houses
                .iter()
                .position(|h| h.role == glasnost_sim::world::HouseRole::Player);
            Some(
                person_house(setup)
                    .or(first.map(|i| HouseId(i as u8)))
                    .ok_or("the setup has no player house")?,
            )
        }
        // spec: S-0412 Terms (human participant: our setup may start a skirmish with none, every player house an AI house)
        // design: watch mode (lead request 2026-11-02, an extra not in the original): a setup with no person's house is watched from no house at all
        None => person_house(setup),
    })
}

/// Whether object `id` of world `w` has its selection drawn by S-0111's
/// marks: a unit's bracket image (the bracket sprite and the overlay palette
/// are there), or a building's corner brackets (claim 8; the overlay palette
/// is there). `w` is the world the marks are drawn from: in a game, the
/// canonical world, which holds the objects made since the game began (#67).
pub fn bracket_drawn(w: &World, assets: &super::build::Assets, id: ObjectId) -> bool {
    let Some(o) = w.obj(id) else {
        return false;
    };
    let palette = assets.get(combat::OVERLAY_PALETTE).is_some();
    if o.kind == SimKind::Building {
        return palette;
    }
    o.kind.is_mobile() && palette && assets.get(combat::BRACKET_SPRITE).is_some()
}

/// Whether cell `c` counts as shrouded for house `viewer` for the slot
/// animations drawn over it (S-0513 claim 29b): S-0110 claim 8's point
/// test at the centre of the cell, at its surface height, which on raised
/// ground reads the explored state of a displaced cell. The height is the
/// terrain's surface height at the centre (S-0106 claims 2 and 3), as S-0410
/// claim 6 gives the radar's test; on a sloped cell that is the slope's
/// height at the centre, not the flat height of the cell's level (S-0513
/// revision 34, Q-1740 answered).
pub(super) fn slot_cell_shrouded(w: &World, viewer: HouseId, c: CellXY) -> bool {
    // spec: S-0513 claim 29b (the centre of the cell); S-0110 claim 8
    let centre = glasnost_sim::world::Pos::centre(c, 0);
    // spec: S-0513 claim 29b (revision 34: at the terrain's surface height there, on a sloped cell the slope's height at the centre; Q-1740 answered); S-0106 claims 2 and 3
    let z = w.surface_at(centre.x, centre.y);
    // spec: S-0110 claim 8 and Edge cases (the point test, with the renderer's quirks.shroud_on_flat_grid, whose direct-cell test takes precedence when off)
    glasnost_sim::econ::shroud::point_shrouded_on(
        w,
        viewer,
        glasnost_sim::world::Pos::new(centre.x, centre.y, z),
        glasnost_render::shroud::ShroudQuirks::default().shroud_on_flat_grid,
    )
}

/// The shadow height *h*ₛ of infantry unit `o`'s generic shadow, or `None`
/// when it draws none (S-0107 claim 19 b and c): only off the surface, and
/// never for a type with drop-pod movement.
pub(super) fn generic_shadow(w: &World, o: &Object) -> Option<i32> {
    // spec: S-0107 claim 19 and Terms (h: the height above the surface; 0 on a bridge deck)
    let h = w.height_above_surface(o.id);
    // spec: S-0107 claim 19 (a) (h = 0: the own shadow instead)
    if h == 0 {
        return None;
    }
    // spec: S-0107 claim 19 (c) (a unit with drop-pod movement never draws the generic shadow)
    if w.rules()
        .unit(o.kind, o.type_index)
        .is_some_and(|u| u.raw.locomotor == Some(glasnost_rules::sim::Locomotor::DropPod))
    {
        return None;
    }
    let c = o.pos.cell();
    // spec: S-0107 claim 19 (b) (the neighbour one step across the bridge towards lower coordinates: (x, y − 1) for an axis along x, (x − 1, y) along y)
    let neighbour = match w.cells.axis(c) {
        glasnost_sim::cells::Axis::X => Some(CellXY::new(c.x, c.y - 1)),
        glasnost_sim::cells::Axis::Y => Some(CellXY::new(c.x - 1, c.y)),
        glasnost_sim::cells::Axis::None => None,
    };
    // spec: S-0107 claim 19 (b) (h − 416, the deck height, over a high-bridge cell with h ≥ 416 whose neighbour is also a high-bridge cell; h otherwise)
    let deck = glasnost_sim::hover::DECK;
    let hs = if w.high_bridge(c) && h >= deck && neighbour.is_some_and(|n| w.high_bridge(n)) {
        h - deck
    } else {
        h
    };
    // spec: S-0107 claim 19 (c) (drawn exactly when hₛ > 0; nothing while h < 0)
    (h > 0 && hs > 0).then_some(hs)
}
