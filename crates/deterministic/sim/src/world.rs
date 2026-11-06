//! The game state: houses, objects, cells and the scenario random stream.
//!
//! Everything here is plain data that serializes deterministically: ordered
//! containers only, object IDs from a counter (never from addresses). The
//! rules ([`SimRules`]) are held beside the state and are not saved.

use std::collections::BTreeMap;
use std::sync::Arc;

use glasnost_map::terrain::TileCatalogue;
use glasnost_map::{CellXY, Theater};
use glasnost_num::F64;
use glasnost_num::geom::Facing;
use serde::{Deserialize, Serialize};

use crate::behaviour::Behaviours;
use crate::cells::Cells;
use crate::drive::DriveState;
use crate::infantry::InfantryState;
use crate::orders::Order;
use crate::path::{PathState, PathfinderChoice};
use crate::regions::Regions;
use crate::rng::Rng;
use crate::rules::{Kind, SimRules};
use crate::turning::Turning;
use crate::update::UpdateOrder;
use crate::walk::WalkState;

/// An object's stable ID, assigned from a counter at creation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ObjectId(pub u32);

/// A house's index in house order (S-0009 claim 19).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct HouseId(pub u8);

/// A position in leptons (S-0204 claim 1).
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct Pos {
    /// Map x.
    pub x: i32,
    /// Map y.
    pub y: i32,
    /// World height.
    pub z: i32,
}

impl Pos {
    /// A position.
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    /// The cell that contains the position (S-0204 claim 2: truncation
    /// toward zero).
    pub fn cell(self) -> CellXY {
        // spec: S-0204 claim 2 (cell = (trunc(x / 256), trunc(y / 256)))
        CellXY::new(self.x / 256, self.y / 256)
    }

    /// The centre of a cell at height `z` (S-0204 claim 3).
    pub fn centre(c: CellXY, z: i32) -> Self {
        // spec: S-0204 claim 3 (centre = (256x + 128, 256y + 128))
        Self::new(256 * c.x + 128, 256 * c.y + 128, z)
    }
}

/// What an attack-move memory holds (S-0213 claim 6): a destination, or a
/// target only. A memory that would hold neither is removed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttackMoveMemory {
    /// A destination (the order had one).
    Dest(Target),
    /// A target only.
    Target(Target),
}

/// What a target, destination or guard spot names (S-0215 Terms).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Target {
    /// An object.
    Object(ObjectId),
    /// A cell.
    Cell(CellXY),
}

/// Which layer of a cell an object is in (S-0209 Terms, *cell layer*).
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub enum Layer {
    /// The ground layer.
    #[default]
    Ground,
    /// The bridge-deck layer.
    Deck,
}

/// A house.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct House {
    /// The name pre-placed objects match (S-0009 claim 27).
    pub name: String,
    /// Country registration index.
    pub country: u32,
    /// Houses this house counts as allies, itself included (S-0209 Terms).
    pub allies: Vec<HouseId>,
    /// Human-controlled (an AI house of our engine counts as one, glossary
    /// *AI house*). A player house with this clear is a test-only
    /// computer-controlled player house (glossary; S-0215 Terms): it follows
    /// the original's computer-controlled-owner rules, as the neutral and
    /// special houses do.
    pub human: bool,
    /// House skill level (S-0209 Terms; 0 for every house of our engine in
    /// a skirmish).
    pub skill: i32,
    /// Start position index, if it has one.
    pub start: Option<u32>,
    /// The house's role in the game (S-0009 claim 19).
    pub role: HouseRole,
    /// Removed from the game by the unused-house cleanup (S-0009 claim
    /// 28): it takes no position, receives no starting units and owns
    /// nothing.
    #[serde(default)]
    pub removed: bool,
    /// Credits (S-0009 claim 45: a left-over starting-unit budget).
    pub credits: i32,
    /// Orders waiting to be applied (S-0213 claim 2).
    pub pending: Vec<Order>,
    /// Combat state (M4).
    pub combat: crate::combat::state::HouseCombat,
    /// Economy state (M5a).
    pub econ: crate::econ::HouseEcon,
    /// The base records (S-0215 claim 38).
    #[serde(default)]
    pub base: crate::base::BaseRecord,
    /// The computed arrival edge (S-0522 claim 5): 0 north, 1 east,
    /// 2 south, 3 west; it starts as north.
    #[serde(default)]
    pub computed_edge: u8,
    /// The lost-to-capture mark (S-0509 claim 11): an engineer has captured
    /// one of this house's buildings. Nothing else sets it and nothing
    /// clears it.
    #[serde(default)]
    pub lost_to_capture: bool,
    /// The infiltrated-vehicle-factory flag (S-0510 claim 5): set when one
    /// of this house's spies infiltrates another house's vehicle factory
    /// (naval yards included); clear at the start of a game and never
    /// cleared.
    #[serde(default)]
    pub infiltrated_vehicle_factory: bool,
    /// The infiltrated-infantry-factory flag (S-0510 claim 5).
    #[serde(default)]
    pub infiltrated_infantry_factory: bool,
    /// An idle test house (S-0216 claim 8, *Removal before the groups*,
    /// design): no house treats it as an enemy, so no object's target
    /// search takes its objects and no hit damages them. Only our arena's
    /// human player house sets it; every house of a game the loader starts
    /// has it clear.
    #[serde(default)]
    pub idle_test_house: bool,
}

/// A house's role (S-0009 claim 19, glossary *house*).
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub enum HouseRole {
    /// A player: a human player's house or a computer player's (an AI house
    /// of our engine).
    #[default]
    Player,
    /// The neutral house.
    Neutral,
    /// The special house.
    Special,
}

impl House {
    /// Whether this house counts `other` as allied (this house's view).
    pub fn allied_with(&self, other: HouseId) -> bool {
        self.allies.binary_search(&other).is_ok()
    }
}

/// An object in the world.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Object {
    /// Stable ID.
    pub id: ObjectId,
    /// Kind.
    pub kind: Kind,
    /// Type registration index within its kind.
    pub type_index: u32,
    /// Owner (terrain objects have none).
    pub owner: Option<HouseId>,
    /// Position in leptons.
    pub pos: Pos,
    /// Cell layer.
    pub layer: Layer,
    /// Hit points.
    pub health: i32,
    /// On the map (S-0213 Terms).
    pub on_map: bool,
    /// Body facing.
    pub body: Turning,
    /// Turret facing.
    pub turret: Turning,
    /// Tick of the last shot (−100 for one that never fired, S-0206 claim 9).
    pub last_fired: i32,
    /// The creation-derived report value r = x mod 65536 of the creation
    /// draw (S-0202 claim 10, Q-0327); 0 for terrain objects, which take no
    /// draw. Presentation reads it; gameplay never does.
    #[serde(default)]
    pub report_value: u16,
    /// A turreted vehicle's latest turret rotation sample (S-0206 claim
    /// 17): whether its turret was turning, recorded after all the turret
    /// rows of the last tick that processed them.
    #[serde(default)]
    pub rotation_sample: bool,
    /// Behaviour slots and processing (S-0215).
    pub beh: Behaviours,
    /// Current target.
    pub target: Option<Target>,
    /// Current destination.
    pub dest: Option<Target>,
    /// Guard spot.
    pub guard_spot: Option<Target>,
    /// Queued destinations.
    pub queued_dests: Vec<Target>,
    /// Attack-move memory (S-0213 claim 6).
    pub attack_move: Option<AttackMoveMemory>,
    /// The object it is in contact with (S-0215 Terms).
    pub contact: Option<ObjectId>,
    /// Path state (S-0212).
    pub path: PathState,
    /// Drive locomotor state (S-0207).
    pub drive: Option<DriveState>,
    /// Walk locomotor state (S-0208).
    pub walk: Option<WalkState>,
    /// Infantry state (S-0107).
    pub infantry: Option<InfantryState>,
    /// Depot-cycle state (S-0314 claims 6 and 16 to 24).
    #[serde(default)]
    pub depot: crate::depot::DepotState,
    /// Teleporting locomotor state (S-0207 claim 20).
    #[serde(default)]
    pub teleport: Option<crate::teleport::TeleportState>,
    /// The spawn controller of a spawner (S-0506 claim 1).
    #[serde(default)]
    pub spawn: Option<crate::spawn::SpawnController>,
    /// Rocket flight state of a rocket missile (S-0506 claims 14 to 21).
    #[serde(default)]
    pub rocket: Option<crate::rocket::RocketState>,
    /// Parasite links (S-0614).
    #[serde(default)]
    pub parasite: Option<crate::parasite::ParasiteState>,
    /// Flight state of an aircraft with the flying locomotor (S-0501).
    #[serde(default)]
    pub air: Option<crate::air::AirState>,
    /// Naval state of a vessel (S-0504).
    #[serde(default)]
    pub naval: Option<crate::naval::NavalState>,
    /// Hover state of a vehicle with the hovering locomotor (S-0503).
    #[serde(default)]
    pub hover: Option<crate::hover::HoverState>,
    /// Flight state of a unit with the jumpjet locomotor (S-0502).
    #[serde(default)]
    pub jump: Option<crate::jump::JumpState>,
    /// Transport state: a transport's passengers, a passenger's transport
    /// (S-0507).
    #[serde(default)]
    pub transport: crate::transport::TransportState,
    /// Falling: the fall rate, whether the object is falling and the bomb
    /// mark (S-0511 claim 10, S-0312 Terms).
    #[serde(default)]
    pub fall: crate::fall::FallState,
    /// The crush-slowdown state (S-0210 claim 9).
    pub crush_slowdown: bool,
    /// Tick at which the targeting timer has run out (S-0215 claim 33).
    pub targeting_timer: i32,
    /// Tick of the object's last turn (S-0203 claim 11).
    pub last_turn: i32,
    /// For a building: its foundation size.
    pub foundation: (i32, i32),
    /// A follower car's leader (S-0009 claim 41, glossary *follower car*).
    pub leader: Option<ObjectId>,
    /// Speed multiplier (S-0208 Terms) or speed bonus factor (S-0207
    /// claim 13), a double; 1.0 at creation.
    pub speed_bonus: F64,
    /// Combat state (M4).
    pub combat: crate::combat::state::CombatState,
    /// Economy state (M5a).
    pub econ: crate::econ::ObjectEcon,
    /// Cloak and disguise state (S-0516).
    #[serde(default)]
    pub cloak: crate::cloak::ObjCloak,
    /// Superweapon state: protection, warp, silo launch (S-0513).
    #[serde(default)]
    pub sw: crate::superweapons::ObjSw,
    /// Garrison state: a building's occupants and shared firing index, an
    /// occupant's building (S-0508).
    #[serde(default)]
    pub structure: crate::structures::StructState,
    /// Magnetic-lift state: seized, grip, drop, wave (S-0607).
    #[serde(default)]
    pub lift: crate::lift::LiftState,
}

/// One house in the game setup.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HouseSetup {
    /// Name pre-placed objects match.
    pub name: String,
    /// Country type name.
    pub country: String,
    /// Human-controlled (AI houses count as human, glossary *AI house*).
    pub human: bool,
    /// Chosen start position, or `None` for random.
    pub start: Option<u32>,
    /// The house's role (S-0009 claim 19).
    pub role: HouseRole,
    /// An AI house of our engine filling a lobby computer slot (glossary
    /// *AI house*; S-0412 claim 12): it counts as human-controlled for the
    /// unit rules, and as a computer player for the unused-house cleanup
    /// (S-0009 claim 28).
    #[serde(default)]
    pub ai: bool,
}

/// How a game is set up: everything the simulation needs besides rules and
/// map. Our engine takes the seed from the setup (S-0202 claim 8,
/// departure).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameSetup {
    /// The game seed (S-0202 claim 8).
    pub seed: u32,
    /// Game-speed index 0 … 6 (S-0201).
    pub game_speed: u8,
    /// Houses in house order (S-0009 claim 19).
    pub houses: Vec<HouseSetup>,
    /// Alliances: pairs of house indices allied both ways.
    pub alliances: Vec<(u8, u8)>,
    /// Our viewer's sandbox: skip the start positions of S-0009 claims 17 to
    /// 20 and starting units, so that a map can be watched without players.
    pub sandbox: bool,
    /// The unit-count setting U (S-0009 claim 45).
    pub unit_count: i32,
    /// The bases setting (S-0009 claim 45).
    pub bases: bool,
    /// The other setup values of a skirmish (S-0412 claim 1).
    #[serde(default)]
    pub skirmish: crate::econ::setup::Settings,
}

/// The map's special flags that the simulation consults (S-0303 Import
/// mapping). `DestroyableBridges` is not here: a skirmish ignores it
/// (S-0511 claim 1).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MapFlags {
    /// The inert flag (S-0303 claim 2; S-0302 claim 6).
    pub inert: bool,
    /// Harvester immunity (S-0303 claim 9; S-0309 table C).
    pub harvester_immune: bool,
}

/// The game state.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct World {
    /// Tick number (S-0201 claim 1).
    pub tick: i32,
    /// The scenario random stream (S-0202).
    pub rng: Rng,
    /// The map-generator stream (S-0202 claim 13), drawn by low-bridge
    /// repair (S-0511 claim 20, F4). Seeded with 0 at session start, which is game start in our engine (S-0202 claim 13).
    #[serde(default)]
    pub map_rng: Option<Rng>,
    /// Game-speed index.
    pub game_speed: u8,
    /// Houses in house order.
    pub houses: Vec<House>,
    /// Objects by ID.
    pub objects: BTreeMap<ObjectId, Object>,
    /// Next object ID.
    pub next_id: u32,
    /// The update order (S-0203 claim 3).
    pub update: UpdateOrder,
    /// Cells.
    pub cells: Cells,
    /// Orders are blocked up to and including this tick (S-0201 claim 14).
    pub orders_blocked_through: i32,
    /// Projectiles in flight by object number (S-0306; in the update order).
    pub projectiles: BTreeMap<ObjectId, crate::combat::state::Projectile>,
    /// The raised effective spawn delay of each scalable projectile type
    /// a vehicle has fired (S-0306 claim 16), by projectile type.
    pub scaled_spawn_delay: BTreeMap<crate::combat::data::ProjectileId, i32>,
    /// The tracked flying objects (S-0303 Terms, F7).
    #[serde(default)]
    pub flying: crate::combat::flying::Flying,
    /// Spawned objects' launchers and released missiles (S-0506).
    #[serde(default)]
    pub spawned: crate::spawn::SpawnedObjects,
    /// Effects created since the drawing last took them (explosions,
    /// launches, deaths): presentation only, neither saved nor hashed.
    #[serde(skip)]
    pub effects: Vec<crate::combat::effects::Effect>,
    /// Provisional rules and requests met so far (diagnostics, not game
    /// state: neither saved nor hashed).
    #[serde(skip)]
    pub notes: Vec<String>,
    /// Economy, production, structures and visibility (M5a).
    pub econ: crate::econ::WorldEcon,
    /// Cloak fields, sensor and disguise-detector coverage (S-0516).
    #[serde(default)]
    pub cloak: crate::cloak::CloakWorld,
    /// Superweapon records, the storm and its clouds (S-0512, S-0513).
    #[serde(default)]
    pub sw: crate::superweapons::SwWorld,
    /// Temporal beams and erasures (S-0514).
    #[serde(default)]
    pub temporal: crate::temporal::TemporalWorld,
    /// Mind control: controllers, victims and permanent control (S-0601).
    #[serde(default)]
    pub mind: crate::mind::MindWorld,
    /// The number of building acquisitions so far (S-0518 claim 3): each
    /// placement of a building into play and each ownership change of one
    /// takes the next number as that building's acquisition rank.
    #[serde(default)]
    pub acquisitions: u64,
    /// Crate places and the any-crate mark (S-0520).
    #[serde(default)]
    pub crates: crate::crates::CrateWorld,
    /// Air-strike designators' strikes (S-0519 claims 16 to 21).
    #[serde(default)]
    pub strike: crate::strike::StrikeWorld,
    /// Radiation fields and cell radiation (S-0525).
    #[serde(default)]
    pub radiation: crate::radiation::RadWorld,
    /// Particle systems and virus-death animations (S-0613).
    #[serde(default)]
    pub gas: crate::gas::GasWorld,
    /// The magnetic waves by their number in the update order (S-0607
    /// claim 21).
    #[serde(default)]
    pub waves: BTreeMap<ObjectId, crate::lift::Wave>,
    /// The debris pieces in play by their number in the update order
    /// (S-0317 claims 1 and 6).
    #[serde(default)]
    pub pieces: BTreeMap<ObjectId, crate::pieces::Piece>,
    /// Disc lasers in play, the most recently fired first (S-0615 claim 12,
    /// S-0203 row 12).
    #[serde(default)]
    pub discs: Vec<crate::disc::Disc>,
    /// Burning walkers in play, by object number (S-0615 claims 4 to 11).
    #[serde(default)]
    pub walkers: BTreeMap<ObjectId, crate::walker::Walker>,
    /// Infantry-making animations and refused mutation victims (S-0611).
    #[serde(default)]
    pub mutation: crate::mutation::MutationWorld,
    /// Drain links (S-0608): the drained buildings and each drainer's
    /// partner.
    #[serde(default)]
    pub drain: crate::drain::DrainWorld,
    /// Route mode's points and routes (S-0213 claims 16 to 17).
    pub routes: crate::routes::RouteWorld,
    /// The psychic domination (S-0612).
    #[serde(default)]
    pub domination: crate::domination::DominationWorld,
    /// The repair depots' modelled slot animations (S-0108 claim 20).
    #[serde(default)]
    pub depot_anims: crate::depot_anim::DepotAnimWorld,
    /// The ordinary animations in play (S-0109, S-0203 claim 2).
    #[serde(default)]
    pub anims: crate::anims::AnimWorld,
    /// The game-wide lighting state (S-0104).
    #[serde(default)]
    pub lighting: crate::lighting::LightWorld,
    /// Map triggers (S-0011).
    pub triggers: crate::triggers::TriggerState,
    /// The map's theater.
    pub theater: Theater,
    /// The map's special flags that the simulation consults (S-0303
    /// Import mapping).
    #[serde(default)]
    pub map_flags: MapFlags,
    /// The hover slots (S-0502 Terms): the jumpjet that holds the slot of a
    /// cell, at most one per cell.
    #[serde(default)]
    pub jump_slots: BTreeMap<CellXY, ObjectId>,
    /// The corridor exemption of buildings and aircraft (S-0212 claim 13).
    #[serde(default)]
    pub threat: crate::threat::ThreatWorld,
    /// The leftovers of the tick that has run last (S-0203 claim 13): the
    /// objects of the four kinds removed in it, kept to its end and gone
    /// at the start of the next tick. Only the defeat sweep reads them
    /// (`crate::leftover`).
    #[serde(default)]
    pub leftovers: BTreeMap<ObjectId, crate::leftover::Leftover>,
    /// The pathfinder in use (S-0212, behind a swappable interface).
    pub pathfinder: PathfinderChoice,
    /// The rules (not saved).
    #[serde(skip)]
    pub rules: Option<Arc<SimRules>>,
    /// The theater's tile catalogue (not saved; given again on load).
    #[serde(skip)]
    pub catalogue: Option<Arc<TileCatalogue>>,
    /// Regions and reach regions (derived; rebuilt on load).
    #[serde(skip)]
    pub regions: Regions,
    /// Working memory of the path search (not state).
    #[serde(skip)]
    pub scratch: std::cell::RefCell<crate::path::SearchScratch>,
    /// The movement trace of the pathing-report tool (M8 A4): off unless
    /// switched on, never saved, hashed, cloned or read by the simulation.
    #[serde(skip)]
    pub trace: crate::trace::Slot,
}

impl World {
    /// The rules.
    pub fn rules(&self) -> &SimRules {
        self.rules.as_deref().expect("world has rules")
    }

    /// A shared handle on the rules.
    pub fn rules_arc(&self) -> Arc<SimRules> {
        self.rules.clone().expect("world has rules")
    }

    /// An object.
    pub fn obj(&self, id: ObjectId) -> Option<&Object> {
        self.objects.get(&id)
    }

    /// An object, for writing.
    pub fn obj_mut(&mut self, id: ObjectId) -> Option<&mut Object> {
        self.objects.get_mut(&id)
    }

    /// Whether house `a` counts house `b` as allied (`a`'s view; S-0209
    /// Terms). An object without an owner is allied with no house.
    pub fn allied(&self, a: Option<HouseId>, b: Option<HouseId>) -> bool {
        match (a, b) {
            (Some(a), Some(b)) => self
                .houses
                .get(usize::from(a.0))
                .is_some_and(|h| h.allied_with(b)),
            _ => false,
        }
    }

    /// Whether object `id` belongs to an idle test house
    /// ([`House::idle_test_house`]; S-0216 claim 8, *Removal before the
    /// groups*).
    pub fn of_idle_house(&self, id: ObjectId) -> bool {
        self.obj(id)
            .and_then(|o| o.owner)
            .and_then(|h| self.houses.get(usize::from(h.0)))
            .is_some_and(|h| h.idle_test_house)
    }

    /// The facing an object's body has now.
    pub fn body_facing(&self, id: ObjectId) -> Facing {
        self.obj(id).map_or(0, |o| o.body.value(self.tick))
    }

    /// A new object ID.
    pub fn new_id(&mut self) -> ObjectId {
        let id = ObjectId(self.next_id);
        self.next_id += 1;
        id
    }

    /// Records a note (a provisional rule met or a request that applies).
    pub fn note(&mut self, text: impl Into<String>) {
        let t = text.into();
        if !self.notes.contains(&t) {
            self.notes.push(t);
        }
    }
}

impl World {
    /// An object's reference point (S-0204 claim 4).
    pub fn reference_point(&self, id: ObjectId) -> Pos {
        let Some(o) = self.obj(id) else {
            return Pos::default();
        };
        if o.kind == Kind::Building {
            let (fw, fh) = o.foundation;
            // spec: S-0204 claim 4 (a building: its position plus (128 × (W − 1), 128 × (H − 1), 0))
            Pos::new(o.pos.x + 128 * (fw - 1), o.pos.y + 128 * (fh - 1), o.pos.z)
        } else {
            o.pos
        }
    }
}
