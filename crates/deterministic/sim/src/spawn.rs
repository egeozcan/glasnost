//! The spawn controller and its slots (S-0506 claims 1 to 13): an object
//! whose type names a spawned type keeps, launches, regenerates and reloads
//! its slot objects (V3 and Dreadnought missiles, a carrier's aircraft).
//!
//! The flight of a non-missile slot aircraft is S-0501 (claims 35 and 36,
//! table T4, in [`crate::air`]). This module drives its slot states and
//! calls the hooks [`aircraft_launched`], [`aircraft_attack`] and
//! [`aircraft_return`], which hand a flying aircraft to the flight code; a
//! slot object of another locomotor only receives its destination or
//! target. Docking back into the slot is the returning row of the Slot
//! table, tested here in every pass ([`docks`]).

use std::collections::BTreeMap;

use glasnost_map::CellXY;
use glasnost_num::{F32, Rounding};
use glasnost_rules::sim::{Decimal, SimData};
use serde::{Deserialize, Serialize};

use crate::combat::data::WarheadId;
use crate::rules::Kind;
use crate::world::{HouseId, ObjectId, Pos, Target, World};

/// A slot's state (S-0506 claim 4, Slot table).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SlotState {
    /// No slot object; regenerating.
    #[default]
    Empty,
    /// The slot object is held, off the map.
    Docked,
    /// Launched and waiting for the release (claims 7 and 8).
    Preparing,
    /// A released rocket's slot, until its rail time ends (claim 8).
    OnRail,
    /// A non-missile slot aircraft attacking the controller target.
    Attacking,
    /// A non-missile slot aircraft flying back to the spawner.
    Returning,
    /// A docked-back non-missile slot aircraft, until its reload time ends.
    Reloading,
}

/// One spawn slot (S-0506 Terms).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Slot {
    /// The slot state.
    pub state: SlotState,
    /// The slot object, while the slot is not empty.
    pub object: Option<ObjectId>,
    /// The tick at which the slot's countdown (rail, regeneration or
    /// reload) has run out; only looked at in passes (claim 2).
    pub until: i32,
}

/// The controller's state (S-0506 claim 5).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ControllerState {
    /// Waiting for a target.
    #[default]
    Idle,
    /// Launching the slots.
    Launching,
    /// After a release, until no slot is attacking or returning.
    CoolingDown,
}

/// A spawner's spawn controller (S-0506 Terms).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpawnController {
    /// The slots, by slot index.
    pub slots: Vec<Slot>,
    /// The controller state.
    pub state: ControllerState,
    /// The controller target.
    pub target: Option<Target>,
    /// The pending target (claim 3).
    pub pending: Option<Target>,
    /// The tick of the next launch pass (claim 2).
    pub next_pass: i32,
    /// The tick of the controller's last launch (claim 6), if any.
    pub last_launch: Option<i32>,
    /// A pending-target clear requested by an own-target change and not
    /// yet settled (claim 25): `Some(true)` for an actual change of a
    /// nonempty own target to empty (or a nonempty assignment redirected
    /// to none), `Some(false)` for an explicit clear of an already empty
    /// one. See [`clear_own_target`] and [`settle_clear`].
    #[serde(default)]
    pub clear_request: Option<bool>,
}

/// The world's record of spawned objects that outlive their slots
/// (S-0506 claims 8, 12, 19 and 26; S-0311 claim 8).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpawnedObjects {
    /// Each slot object's spawner (its launcher), kept until the object
    /// leaves play.
    pub launcher: BTreeMap<ObjectId, ObjectId>,
    /// The enrolled kamikazes with their release cells (claim 12), in
    /// enrollment order: removal keeps the others' relative order and a new
    /// enrollment goes last (claim 26).
    pub kamikaze: Vec<(ObjectId, CellXY)>,
    /// The tick number on which the shared re-order cycle is next due
    /// (claim 26).
    pub next_reorder: i32,
}

impl Default for SpawnedObjects {
    fn default() -> Self {
        Self {
            launcher: BTreeMap::new(),
            kamikaze: Vec::new(),
            // spec: S-0506 claim 26 and Constants (in a newly loaded skirmish the first shared re-order cycle is due at tick number 1, not 0)
            next_reorder: 1,
        }
    }
}

/// A type's spawn fields (S-0506 Import mapping).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SpawnRules {
    /// The spawned aircraft type (registration index).
    pub spawned: u32,
    /// The spawn count N (at least 1).
    pub count: i32,
    /// The regeneration time in ticks.
    pub regen: i32,
    /// The reload time in ticks.
    pub reload: i32,
}

/// One rocket parameter set (S-0506 Import mapping).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RocketSet {
    /// The set's rocket type (aircraft registration index).
    pub type_index: Option<u32>,
    /// Pause time, ticks.
    pub pause: i32,
    /// Tilt time (raise time for set C), ticks.
    pub tilt: i32,
    /// Pᵢ, quarter turns.
    pub pitch_initial: F32,
    /// P_f, quarter turns.
    pub pitch_final: F32,
    /// τ, radians per tick.
    pub turn_rate: F32,
    /// Raise rate, leptons per tick.
    pub raise_rate: i32,
    /// a, leptons per tick per tick.
    pub acceleration: F32,
    /// Cruise altitude, leptons above the terrain.
    pub altitude: i32,
    /// Damage.
    pub damage: i32,
    /// Elite damage.
    pub elite_damage: i32,
    /// Body length L, leptons.
    pub body_length: i32,
    /// Lazy-curve flag.
    pub lazy: bool,
    /// Warhead.
    pub warhead: Option<WarheadId>,
    /// Elite warhead.
    pub elite_warhead: Option<WarheadId>,
}

/// The parameter sets A, B and C (S-0506 Terms).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub enum RocketSetId {
    /// Set A (the V3-rocket settings).
    #[default]
    A,
    /// Set B (the dreadnought-missile settings).
    B,
    /// Set C (the third settings).
    C,
}

impl RocketSetId {
    /// The index into [`MissileRules::sets`].
    pub fn index(self) -> usize {
        match self {
            RocketSetId::A => 0,
            RocketSetId::B => 1,
            RocketSetId::C => 2, // spec: S-0506 Terms (sets A, B and C)
        }
    }
}

/// The missile rules of one game (S-0506 Import mapping).
#[derive(Clone, Debug, Default)]
pub struct MissileRules {
    /// Spawn fields of every type that names a spawned type and a positive
    /// spawn count, by kind and registration index.
    pub spawners: BTreeMap<(Kind, u32), SpawnRules>,
    /// Parameter sets A, B and C.
    pub sets: [RocketSet; 3], // spec: S-0506 Terms (three parameter sets)
    /// The importer's warnings about cycles of spawned and slave types
    /// (S-0611 F5).
    pub warnings: Vec<String>,
}

impl MissileRules {
    /// Builds the missile rules from the typed data.
    pub fn new(
        s: &SimData,
        names: &BTreeMap<(Kind, String), u32>,
        warheads: &[crate::combat::data::WarheadRules],
        mode: Rounding,
    ) -> Self {
        let aircraft = |n: &Option<String>| -> Option<u32> {
            n.as_ref()
                .and_then(|n| names.get(&(Kind::Aircraft, n.clone())).copied())
        };
        let warhead = |n: &Option<String>| -> Option<WarheadId> {
            n.as_ref()
                .and_then(|n| warheads.iter().position(|h| &h.name == n))
                .map(|i| i as u32)
        };
        let mut spawners = BTreeMap::new();
        let mut add = |kind: Kind, name: &str, c: &glasnost_rules::sim::CombatFields| {
            // spec: S-0506 claim 1 and Import mapping (a spawned type and a spawn count N ≥ 1; SpawnsNumber, SpawnRegenRate and SpawnReloadRate absent: 0)
            let count = c.spawns_number.unwrap_or(0);
            if let (Some(spawned), true, Some(&i)) = (
                aircraft(&c.spawns),
                count >= 1,
                names.get(&(kind, name.to_owned())),
            ) {
                spawners.insert(
                    (kind, i),
                    SpawnRules {
                        spawned,
                        count,
                        regen: c.spawn_regen_rate.unwrap_or(0),
                        reload: c.spawn_reload_rate.unwrap_or(0),
                    },
                );
            }
        };
        for (n, u) in &s.infantry {
            add(Kind::Infantry, n, &u.combat);
        }
        for (n, u) in &s.vehicles {
            add(Kind::Vehicle, n, &u.combat);
        }
        for (n, u) in &s.aircraft {
            add(Kind::Aircraft, n, &u.combat);
        }
        for (n, b) in &s.buildings {
            add(Kind::Building, n, &b.combat);
        }
        let g = &s.general;
        let dbl = |d: &Option<Decimal>, def: &str| -> F32 {
            // spec: S-0506 Import mapping (pitches: real, read in double and stored in single precision)
            d.clone()
                .unwrap_or_else(|| Decimal(def.into()))
                .value(mode)
                .to_f32(mode)
        };
        let sgl = |d: &Option<Decimal>, def: &str| -> F32 {
            // spec: S-0506 Import mapping (turn rate and acceleration: real, single precision)
            d.clone()
                .unwrap_or_else(|| Decimal(def.into()))
                .single_value(mode)
                .to_f32(mode)
        };
        let raise = |d: &Option<Decimal>| -> i32 {
            // spec: S-0506 Import mapping (raise rate: real, truncated to an integer; absent 1)
            d.as_ref().map_or(1, |d| d.value(mode).to_i32_common())
        };
        let sets = [
            RocketSet {
                type_index: aircraft(&g.v3_rocket_type),
                // spec: S-0506 Import mapping (set A defaults: pause 0, tilt 60, Pᵢ 0.0, P_f 1.0, τ 0.05, a 0.4, altitude 768, damage 1000, elite 1000, body 256, lazy curve yes)
                pause: g.v3_rocket_pause_frames.unwrap_or(0),
                tilt: g.v3_rocket_tilt_frames.unwrap_or(60), // spec: S-0506 Import mapping (absent-key default)
                pitch_initial: dbl(&g.v3_rocket_pitch_initial, "0.0"),
                pitch_final: dbl(&g.v3_rocket_pitch_final, "1.0"),
                turn_rate: sgl(&g.v3_rocket_turn_rate, "0.05"),
                raise_rate: raise(&g.v3_rocket_raise_rate),
                acceleration: sgl(&g.v3_rocket_acceleration, "0.4"),
                altitude: g.v3_rocket_altitude.unwrap_or(768), // spec: S-0506 Import mapping (absent-key default)
                damage: g.v3_rocket_damage.unwrap_or(1000), // spec: S-0506 Import mapping (absent-key default)
                elite_damage: g.v3_rocket_elite_damage.unwrap_or(1000), // spec: S-0506 Import mapping (absent-key default)
                body_length: g.v3_rocket_body_length.unwrap_or(256), // spec: S-0506 Import mapping (absent-key default)
                lazy: g.v3_rocket_lazy_curve.unwrap_or(true),
                warhead: warhead(&g.v3_warhead),
                elite_warhead: warhead(&g.v3_elite_warhead),
            },
            RocketSet {
                type_index: aircraft(&g.d_misl_type),
                // spec: S-0506 Import mapping (set B defaults: pause 10, tilt 60, Pᵢ 0.0, P_f 1.0, τ 0.05, a 0.4, altitude 768, damage 1000, elite 1000, body 128, lazy curve no)
                pause: g.d_misl_pause_frames.unwrap_or(10),
                tilt: g.d_misl_tilt_frames.unwrap_or(60), // spec: S-0506 Import mapping (absent-key default)
                pitch_initial: dbl(&g.d_misl_pitch_initial, "0.0"),
                pitch_final: dbl(&g.d_misl_pitch_final, "1.0"),
                turn_rate: sgl(&g.d_misl_turn_rate, "0.05"),
                raise_rate: raise(&g.d_misl_raise_rate),
                acceleration: sgl(&g.d_misl_acceleration, "0.4"),
                altitude: g.d_misl_altitude.unwrap_or(768), // spec: S-0506 Import mapping (absent-key default)
                damage: g.d_misl_damage.unwrap_or(1000), // spec: S-0506 Import mapping (absent-key default)
                elite_damage: g.d_misl_elite_damage.unwrap_or(1000), // spec: S-0506 Import mapping (absent-key default)
                body_length: g.d_misl_body_length.unwrap_or(128), // spec: S-0506 Import mapping (absent-key default)
                lazy: g.d_misl_lazy_curve.unwrap_or(false),
                warhead: warhead(&g.d_misl_warhead),
                elite_warhead: warhead(&g.d_misl_elite_warhead),
            },
            RocketSet {
                type_index: aircraft(&g.c_misl_type),
                // spec: S-0506 Import mapping (set C defaults: pause 10, tilt 30, Pᵢ 1.0, P_f 1.0, τ 1.0, a 0.6, altitude 768, damage 500, elite 500, body 128, lazy curve no)
                pause: g.c_misl_pause_frames.unwrap_or(10),
                tilt: g.c_misl_tilt_frames.unwrap_or(30), // spec: S-0506 Import mapping (absent-key default)
                pitch_initial: dbl(&g.c_misl_pitch_initial, "1.0"),
                pitch_final: dbl(&g.c_misl_pitch_final, "1.0"),
                turn_rate: sgl(&g.c_misl_turn_rate, "1.0"),
                raise_rate: raise(&g.c_misl_raise_rate),
                acceleration: sgl(&g.c_misl_acceleration, "0.6"),
                altitude: g.c_misl_altitude.unwrap_or(768), // spec: S-0506 Import mapping (absent-key default)
                damage: g.c_misl_damage.unwrap_or(500), // spec: S-0506 Import mapping (absent-key default)
                elite_damage: g.c_misl_elite_damage.unwrap_or(500), // spec: S-0506 Import mapping (absent-key default)
                body_length: g.c_misl_body_length.unwrap_or(128), // spec: S-0506 Import mapping (absent-key default)
                lazy: g.c_misl_lazy_curve.unwrap_or(false),
                warhead: warhead(&g.c_misl_warhead),
                elite_warhead: warhead(&g.c_misl_elite_warhead),
            },
        ];
        Self {
            spawners,
            sets,
            warnings: Vec::new(),
        }
    }

    /// The parameter set of a rocket type, if it is one of the three
    /// (S-0506 Terms: set A for the set-A type, set C for the set-C type,
    /// set B otherwise).
    pub fn set_of(&self, aircraft: u32) -> Option<RocketSetId> {
        let [a, b, c] = &self.sets;
        if a.type_index == Some(aircraft) {
            Some(RocketSetId::A)
        } else if c.type_index == Some(aircraft) {
            Some(RocketSetId::C)
        } else if b.type_index == Some(aircraft) {
            Some(RocketSetId::B)
        } else {
            None
        }
    }

    /// A parameter set.
    pub fn set(&self, s: RocketSetId) -> &RocketSet {
        &self.sets[s.index()]
    }
}

// ---------------------------------------------------------------------
// Queries
// ---------------------------------------------------------------------

/// Whether an object is of a rocket type (S-0506 Terms).
pub fn rocket_type(w: &World, id: ObjectId) -> Option<RocketSetId> {
    let o = w.obj(id)?;
    if o.kind != Kind::Aircraft {
        return None;
    }
    w.rules().missiles.set_of(o.type_index)
}

/// Whether an object's type has the missile-spawn flag (S-0311 Terms).
pub fn missile_type(w: &World, id: ObjectId) -> bool {
    w.obj(id)
        .is_some_and(|o| type_missile_flag(w, o.kind, o.type_index))
}

/// Whether a type has the missile-spawn flag.
/// A building type has no missile-spawn field (S-0506 Import mapping: an
/// object type's `MissileSpawn`, imported for mobile types).
fn type_missile_flag(w: &World, kind: Kind, ti: u32) -> bool {
    w.rules()
        .unit(kind, ti)
        .is_some_and(|u| u.vehicle.missile_spawn)
}

/// Whether slot `i` of a controller is a missile slot (S-0506 Terms: its
/// spawned type is one of the three rocket types).
fn missile_slot(w: &World, spawner: ObjectId) -> bool {
    spawn_rules(w, spawner).is_some_and(|r| w.rules().missiles.set_of(r.spawned).is_some())
}

/// A spawner's spawn fields.
pub fn spawn_rules(w: &World, spawner: ObjectId) -> Option<SpawnRules> {
    let o = w.obj(spawner)?;
    w.rules()
        .missiles
        .spawners
        .get(&(o.kind, o.type_index))
        .copied()
}

/// The launching count (S-0506 Terms): the slots on the rail plus the
/// preparing slots whose object is of a missile type and on the map.
pub fn launching_count(w: &World, spawner: ObjectId) -> i32 {
    let Some(c) = w.obj(spawner).and_then(|o| o.spawn.as_ref()) else {
        return 0;
    };
    c.slots
        .iter()
        .filter(|s| match s.state {
            SlotState::OnRail => true,
            SlotState::Preparing => s
                .object
                .is_some_and(|x| missile_type(w, x) && w.obj(x).is_some_and(|o| o.on_map)),
            _ => false,
        })
        .count() as i32
}

/// Whether any of a spawner's slot objects is alive (S-0506 Terms: its
/// slot is not empty).
pub fn any_alive(w: &World, spawner: ObjectId) -> bool {
    w.obj(spawner)
        .and_then(|o| o.spawn.as_ref())
        .is_some_and(|c| c.slots.iter().any(|s| s.state != SlotState::Empty))
}

/// For a spawner with a spawn controller: whether any of its spawned
/// aircraft is docked (S-0105 Terms *docked*: inside its spawner, waiting or
/// reloading; one taking off, attacking, returning, or destroyed and not yet
/// replaced is not). `None` without a controller. Read-only; the drawing's
/// alternative-body state (S-0105 claim 12) reads it.
pub fn any_docked(w: &World, spawner: ObjectId) -> Option<bool> {
    let c = w.obj(spawner)?.spawn.as_ref()?;
    // spec: S-0105 Terms (docked: a slot that is docked or reloading in its spawner)
    Some(
        c.slots
            .iter()
            .any(|s| matches!(s.state, SlotState::Docked | SlotState::Reloading)),
    )
}

/// Whether the launching count lies strictly between 0 and the spawn
/// count, which makes the spawner not movable (S-0506 claim 10, S-0214
/// Terms *movable*).
pub fn partly_launched(w: &World, spawner: ObjectId) -> bool {
    let n = w
        .obj(spawner)
        .and_then(|o| o.spawn.as_ref())
        .map_or(0, |c| c.slots.len() as i32);
    let k = launching_count(w, spawner);
    // spec: S-0506 claim 10 (0 < launching count < N: not movable)
    0 < k && k < n
}

/// Whether a spawner may not begin a new track (S-0506 claim 10, S-0504
/// claim 3, S-0207 claim 6): its launching count is above 0.
pub fn track_held(w: &World, spawner: ObjectId) -> bool {
    launching_count(w, spawner) > 0
}

/// A mobile spawner that is seized (S-0506 claims 6 and 13, revision 8):
/// the magnetic-lift state of S-0607 claims 5 and 20, which lasts past the
/// grip until the unit's own locomotor returns.
pub fn magnet_held(w: &World, spawner: ObjectId) -> bool {
    crate::lift::seized(w, spawner)
}

/// The launcher of a spawned object, if it still exists (S-0311 claim 8).
pub fn launcher_of(w: &World, id: ObjectId) -> Option<ObjectId> {
    w.spawned
        .launcher
        .get(&id)
        .copied()
        .filter(|&l| w.obj(l).is_some())
}

/// Whether an object is a kamikaze (S-0506 Terms).
pub fn kamikaze(w: &World, id: ObjectId) -> bool {
    w.spawned.kamikaze.iter().any(|&(k, _)| k == id)
}

// ---------------------------------------------------------------------
// Creation and slot objects
// ---------------------------------------------------------------------

/// Gives a newly created object its spawn controller (S-0506 claim 1):
/// N slots, each with a new docked slot object of the spawned type, owned
/// by the spawner's house and off the map; the first pass 20 ticks later
/// (claim 2). `chain` holds the creations in progress, the new spawner's
/// own type last (S-0611 claim 14 and F5).
pub(crate) fn created_in(w: &mut World, id: ObjectId, chain: &mut crate::objects::Chain) {
    let Some(r) = spawn_rules(w, id) else { return };
    let t = w.tick;
    let mut c = SpawnController {
        slots: vec![Slot::default(); r.count as usize],
        // spec: S-0506 claim 2 (the first pass 20 ticks after the controller is created)
        next_pass: t.wrapping_add(20),
        ..Default::default()
    };
    if let Some(o) = w.obj_mut(id) {
        o.spawn = Some(c.clone());
    }
    let owner = w.obj(id).and_then(|o| o.owner);
    for i in 0..c.slots.len() {
        // spec: S-0506 claim 1 (each slot at once receives a new docked slot object; no regeneration time on the first fill)
        // spec: S-0506 claim 1 and S-0611 F5 (in increasing slot index, each slot object's own creation set before the next slot object)
        let x = new_slot_object(w, id, r.spawned, owner, chain);
        c.slots[i] = Slot {
            state: SlotState::Docked,
            object: Some(x),
            until: t,
        };
        if let Some(o) = w.obj_mut(id)
            && let Some(sc) = o.spawn.as_mut()
        {
            sc.slots[i] = c.slots[i].clone();
        }
    }
}

/// A new slot object of the spawned type, off the map (S-0506 claims 1
/// and 9).
fn new_slot_object(
    w: &mut World,
    spawner: ObjectId,
    spawned: u32,
    owner: Option<HouseId>,
    chain: &mut crate::objects::Chain,
) -> ObjectId {
    let x = crate::objects::create_in(w, Kind::Aircraft, spawned, owner, chain);
    w.spawned.launcher.insert(x, spawner);
    x
}

// ---------------------------------------------------------------------
// Shots (claim 3)
// ---------------------------------------------------------------------

/// A shot of a spawner weapon (S-0506 claim 3, S-0301 Terms): no
/// projectile; the shot's target becomes the pending target unless it is
/// already the controller target.
pub fn shot(w: &mut World, spawner: ObjectId, target: Target) {
    settle_clear(w, spawner);
    let Some(c) = w.obj_mut(spawner).and_then(|o| o.spawn.as_mut()) else {
        return;
    };
    // spec: S-0506 claim 3 (the controller target again: nothing changes)
    if c.target == Some(target) {
        return;
    }
    c.pending = Some(target);
}

/// The spawner's held target actually changed to empty (S-0506 claim
/// 25): a pending clear is requested. It succeeds only when the
/// controller target is nonempty
/// (`quirks.spawner_pending_target_survives_idle_clear`; off: it always
/// succeeds).
pub fn target_cleared(w: &mut World, spawner: ObjectId) {
    let survives = w.rules().quirks.spawner_pending_target_survives_idle_clear;
    if let Some(c) = w.obj_mut(spawner).and_then(|o| o.spawn.as_mut()) {
        // spec: S-0506 claim 25 and Edge cases (the clear succeeds only with a nonempty controller target; off: every explicit own-target clear empties pending)
        if !survives || c.target.is_some() {
            c.pending = None;
        }
    }
}

/// Sets an object's own target to `t` (S-0506 claim 25; S-0215 claim 12;
/// S-0309 claims 4 and 24). When this actually changes a nonempty own
/// target to empty, a spawner requests the conditional pending-target
/// clear; direct replacement by a nonempty target does not touch pending.
/// The request is recorded on the controller and settled by
/// [`settle_clear`] before anything reads the pending or controller
/// target, so its outcome is the same as an immediate clear.
pub fn set_own_target(ob: &mut crate::world::Object, t: Option<Target>) {
    // spec: S-0518 claim 15 (the charging mark clears whenever the unit is given a target or its target changes)
    ob.combat.charging = false;
    let had = ob.target.is_some();
    if ob.target != t {
        // spec: S-0519 claim 19 (every target change is counted, so that a change to B and back to A is two changes)
        ob.combat.target_epoch = ob.combat.target_epoch.wrapping_add(1);
    }
    ob.target = t;
    // A nonempty target's retarget let-go needs the world: the caller makes it (`crate::emitters::retarget_let_go`).
    crate::emitters::cleared(ob);
    if t.is_none() && had {
        request_clear(ob, true);
    }
}

/// Clears an object's own target ([`set_own_target`] with none).
pub fn clear_own_target(ob: &mut crate::world::Object) {
    set_own_target(ob, None);
}

/// An explicit own-target clear by an order (S-0213 claims 8 and 10, table
/// T3; S-0215 claim 12): as [`clear_own_target`], but a clear of an already
/// empty own target is also recorded, for
/// `quirks.spawner_pending_target_survives_idle_clear` off.
pub fn explicit_clear_own_target(ob: &mut crate::world::Object) {
    let had = ob.target.is_some();
    if had {
        ob.combat.target_epoch = ob.combat.target_epoch.wrapping_add(1); // spec: S-0519 claim 19 (a target change)
    }
    ob.target = None;
    crate::emitters::cleared(ob);
    request_clear(ob, had);
}

/// A nonempty target assigned to the object is redirected to no target
/// (S-0309 claim 24): the own target becomes empty and a spawner requests
/// the conditional pending clear even when its old own target was already
/// empty.
pub fn redirected_to_none(ob: &mut crate::world::Object) {
    if ob.target.is_some() {
        ob.combat.target_epoch = ob.combat.target_epoch.wrapping_add(1); // spec: S-0519 claim 19 (a target change)
    }
    ob.target = None;
    crate::emitters::cleared(ob);
    // spec: S-0309 claim 24 (assigning a nonempty target redirected to no target requests the conditional clear, even if the old own target was empty)
    request_clear(ob, true);
}

fn request_clear(ob: &mut crate::world::Object, counts: bool) {
    if let Some(c) = ob.spawn.as_mut() {
        c.clear_request = Some(c.clear_request.unwrap_or(false) || counts);
    }
}

/// Settles a spawner's recorded pending-target clear request (S-0506 claim
/// 25): with `quirks.spawner_pending_target_survives_idle_clear` on, a
/// request from an already empty own target changes nothing, and any other
/// clears pending only when the controller target is nonempty; off, every
/// request clears pending.
pub fn settle_clear(w: &mut World, spawner: ObjectId) {
    let survives = w.rules().quirks.spawner_pending_target_survives_idle_clear;
    let Some(c) = w.obj_mut(spawner).and_then(|o| o.spawn.as_mut()) else {
        return;
    };
    let Some(counts) = c.clear_request.take() else {
        return;
    };
    // spec: S-0506 claim 25 and Edge cases (asking to clear an already empty own target leaves pending unchanged; off: every explicit own-target clear empties pending)
    if survives && !counts {
        return;
    }
    target_cleared(w, spawner);
}

/// The spawner's firing check answered *impossible* to an attempt with
/// the spawner weapon (S-0506 claim 11, S-0309 claim 4): the controller
/// resets; the spawner keeps its target.
pub fn impossible(w: &mut World, spawner: ObjectId) {
    if w.obj(spawner).is_some_and(|o| o.spawn.is_some()) {
        reset(w, spawner);
    }
}

/// A firing check's answer to an attempt with the weapon in `slot`
/// (S-0506 claim 11): *impossible* for the spawner weapon resets the
/// controller.
pub fn firing_answer(
    w: &mut World,
    spawner: ObjectId,
    slot: usize,
    answer: crate::combat::check::Answer,
) {
    if answer != crate::combat::check::Answer::Impossible {
        return;
    }
    if spawner_weapon(w, spawner).is_some_and(|(s, _)| s == slot) {
        impossible(w, spawner);
    }
}

// ---------------------------------------------------------------------
// Launch passes (claims 2 to 9)
// ---------------------------------------------------------------------

/// The spawner's part of its turn: a launch pass when one is due
/// (S-0506 claim 2).
pub fn turn(w: &mut World, spawner: ObjectId) {
    settle_clear(w, spawner);
    let t = w.tick;
    let due = w
        .obj(spawner)
        .and_then(|o| o.spawn.as_ref())
        .is_some_and(|c| t >= c.next_pass);
    if !due {
        return;
    }
    if let Some(c) = w.obj_mut(spawner).and_then(|o| o.spawn.as_mut()) {
        // spec: S-0506 claim 2 (each later pass comes 10 ticks after the previous one)
        c.next_pass = c.next_pass.wrapping_add(10).max(t.wrapping_add(1));
    }
    pass(w, spawner);
}

/// One launch pass (S-0506 claim 2, Slot and Controller tables).
pub fn pass(w: &mut World, spawner: ObjectId) {
    settle_clear(w, spawner);
    let Some(n) = w
        .obj(spawner)
        .and_then(|o| o.spawn.as_ref())
        .map(|c| c.slots.len())
    else {
        return;
    };
    // spec: S-0506 claim 25 (no launch pass drops pending merely because the spawner has no own target)
    for i in 0..n {
        slot_step(w, spawner, i);
        if w.obj(spawner).is_none() {
            return;
        }
    }
    controller_step(w, spawner);
}

fn controller(w: &World, spawner: ObjectId) -> Option<&SpawnController> {
    w.obj(spawner).and_then(|o| o.spawn.as_ref())
}

fn controller_mut(w: &mut World, spawner: ObjectId) -> Option<&mut SpawnController> {
    w.obj_mut(spawner).and_then(|o| o.spawn.as_mut())
}

/// Adopts a nonempty pending target as the controller target (S-0506
/// claim 3).
fn adopt_pending(w: &mut World, spawner: ObjectId) {
    if let Some(c) = controller_mut(w, spawner)
        && let Some(p) = c.pending.take()
    {
        c.target = Some(p);
    }
}

/// The Controller table (S-0506 claim 5).
fn controller_step(w: &mut World, spawner: ObjectId) {
    let Some(c) = controller(w, spawner).cloned() else {
        return;
    };
    match c.state {
        ControllerState::Idle => {
            // spec: S-0506 claim 5 (idle: a pending target becomes the controller target)
            adopt_pending(w, spawner);
            let Some(target) = controller(w, spawner).and_then(|c| c.target) else {
                // spec: S-0506 claim 11 (an idle controller without a target simply waits)
                return;
            };
            // spec: S-0506 claim 5 (the weapon range test of S-0307 for the spawner weapon, from where it stands)
            let weapon = spawner_weapon(w, spawner).map(|(_, wp)| wp);
            if crate::combat::range::in_range(w, spawner, weapon, Some(target), None) {
                if let Some(c) = controller_mut(w, spawner) {
                    c.state = ControllerState::Launching;
                }
            } else {
                reset(w, spawner);
            }
        }
        ControllerState::Launching => {
            if c.target.is_none() {
                // spec: S-0506 claim 5 (launching without a controller target: reset)
                reset(w, spawner);
                return;
            }
            // spec: S-0506 claim 5 (wait until every slot is preparing or empty)
            let ready = c
                .slots
                .iter()
                .all(|s| matches!(s.state, SlotState::Preparing | SlotState::Empty));
            if ready {
                let any_missile = release(w, spawner);
                if any_missile {
                    // spec: S-0506 claim 8 (a missile released: the controller resets, then cools down)
                    reset(w, spawner);
                }
                if let Some(c) = controller_mut(w, spawner) {
                    c.state = ControllerState::CoolingDown;
                }
            }
        }
        ControllerState::CoolingDown => {
            // spec: S-0506 claim 5 (cooling down: idle once no slot is attacking or returning)
            let busy = c
                .slots
                .iter()
                .any(|s| matches!(s.state, SlotState::Attacking | SlotState::Returning));
            if !busy && let Some(c) = controller_mut(w, spawner) {
                c.state = ControllerState::Idle;
            }
        }
    }
}

/// The weapon slot that holds the spawner weapon, and the weapon (S-0506
/// claim 7: the primary slot if it does, the secondary otherwise).
pub fn spawner_weapon(
    w: &World,
    spawner: ObjectId,
) -> Option<(usize, crate::combat::data::WeaponId)> {
    let r = w.rules();
    for slot in [0, 1] {
        if let Some(wp) = crate::combat::weapon::slot_weapon(w, spawner, slot)
            && r.combat.weapon(wp).spawner
        {
            return Some((slot, wp));
        }
    }
    None
}

/// The slot-table transitions of one slot in a pass (S-0506 claim 4).
fn slot_step(w: &mut World, spawner: ObjectId, i: usize) {
    let t = w.tick;
    let Some(c) = controller(w, spawner).cloned() else {
        return;
    };
    let Some(slot) = c.slots.get(i).cloned() else {
        return;
    };
    let r = spawn_rules(w, spawner).unwrap_or_default();
    let missile = missile_slot(w, spawner);
    match slot.state {
        SlotState::Docked => {
            if can_launch(w, spawner, &c, missile) {
                launch(w, spawner, i);
            }
        }
        SlotState::Preparing if missile => {
            // spec: S-0506 Slot table (preparing, object of a missile type: a rocket slot's object is of a rocket type; none, claim 8 releases it)
        }
        SlotState::Preparing => {
            // spec: S-0506 claim 3 (a non-missile slot preparing, attacking or returning adopts the pending target)
            adopt_pending(w, spawner);
            let target = controller(w, spawner).and_then(|c| c.target);
            let Some(x) = slot.object else { return };
            if missile_type(w, x) {
                // spec: S-0506 Terms (missile type) and Slot table (preparing, object of a missile type, here a flagged type in a non-missile slot: none; claim 8 releases it)
            } else if target.is_none() {
                // spec: S-0506 Slot table (preparing, object not of a missile type, no controller target: returning)
                set_slot(w, spawner, i, SlotState::Returning, slot.until);
                aircraft_return(w, x, spawner);
            } else if let Some(s) = w.obj(spawner) {
                // spec: S-0506 Slot table (re-sent to the step direction 4 neighbour of the spawner's cell)
                let (dx, dy) = crate::regions::STEPS[4]; // spec: S-0204 Constants (step direction 4)
                let cell = s.pos.cell();
                aircraft_launched(w, x, CellXY::new(cell.x + dx, cell.y + dy));
            }
        }
        SlotState::OnRail => {
            if t >= slot.until {
                // spec: S-0506 claim 9 (rail time over: empty, regeneration countdown of the spawner type)
                empty_slot(w, spawner, i, t.wrapping_add(r.regen));
            }
        }
        SlotState::Attacking => {
            adopt_pending(w, spawner);
            let target = controller(w, spawner).and_then(|c| c.target);
            let Some(x) = slot.object else { return };
            let out = w.obj(x).is_some_and(|o| o.combat.ammo == 0);
            match target {
                Some(tg) if !out => {
                    // spec: S-0506 Slot table (attacking: keeps attacking the controller target)
                    aircraft_attack(w, x, tg);
                }
                _ => {
                    // spec: S-0506 Slot table (out of ammunition or no controller target: returning)
                    set_slot(w, spawner, i, SlotState::Returning, slot.until);
                    aircraft_return(w, x, spawner);
                }
            }
        }
        SlotState::Returning => {
            adopt_pending(w, spawner);
            let target = controller(w, spawner).and_then(|c| c.target);
            let Some(x) = slot.object else { return };
            let ammo_left = w.obj(x).is_some_and(|o| o.combat.ammo != 0);
            if let Some(tg) = target
                && ammo_left
            {
                // spec: S-0506 Slot table (returning with ammunition left and a controller target: attacking)
                set_slot(w, spawner, i, SlotState::Attacking, slot.until);
                aircraft_attack(w, x, tg);
            } else if docks(w, x, spawner) {
                // spec: S-0506 Slot table (docking: taken off the map; reloading, reload countdown starts)
                crate::objects::take_inside(w, x);
                set_slot(
                    w,
                    spawner,
                    i,
                    SlotState::Reloading,
                    t.wrapping_add(r.reload),
                );
            } else {
                aircraft_return(w, x, spawner);
            }
        }
        SlotState::Reloading => {
            if t >= slot.until {
                // spec: S-0506 claim 9 (reload over: docked; ammunition set to the type's, health to its strength)
                if let Some(x) = slot.object {
                    let tc_ammo =
                        crate::combat::veterancy::type_combat(w, x).map_or(0, |tc| tc.ammo);
                    let strength = w
                        .obj(x)
                        .and_then(|o| w.rules().unit(o.kind, o.type_index))
                        .map_or(0, |u| u.strength);
                    if let Some(o) = w.obj_mut(x) {
                        o.combat.ammo = tc_ammo;
                        o.health = strength;
                    }
                }
                set_slot(w, spawner, i, SlotState::Docked, slot.until);
            }
        }
        SlotState::Empty => {
            if t >= slot.until {
                // spec: S-0506 claim 9 (regeneration over: a new object for the spawner's current house, off the map, docked)
                let owner = w.obj(spawner).and_then(|o| o.owner);
                // A fresh creation, outside any other creation in progress (an empty chain).
                let x = new_slot_object(w, spawner, r.spawned, owner, &mut Vec::new());
                if let Some(c) = controller_mut(w, spawner) {
                    c.slots[i] = Slot {
                        state: SlotState::Docked,
                        object: Some(x),
                        until: t,
                    };
                }
            }
        }
    }
}

/// Whether a returning slot aircraft docks (S-0506 Slot table): it is in
/// the spawner's cell and its height minus the spawner's is below 20
/// leptons (signed).
pub fn docks(w: &World, x: ObjectId, spawner: ObjectId) -> bool {
    let (Some(a), Some(s)) = (w.obj(x), w.obj(spawner)) else {
        return false;
    };
    // spec: S-0506 Slot table and Constants (return height margin 20 leptons, signed)
    a.on_map && a.pos.cell() == s.pos.cell() && a.pos.z - s.pos.z < 20
}

fn set_slot(w: &mut World, spawner: ObjectId, i: usize, state: SlotState, until: i32) {
    if let Some(c) = controller_mut(w, spawner)
        && let Some(s) = c.slots.get_mut(i)
    {
        s.state = state;
        s.until = until;
    }
}

/// Makes a slot empty with its countdown ending at `until`.
fn empty_slot(w: &mut World, spawner: ObjectId, i: usize, until: i32) {
    if let Some(c) = controller_mut(w, spawner)
        && let Some(s) = c.slots.get_mut(i)
    {
        *s = Slot {
            state: SlotState::Empty,
            object: None,
            until,
        };
    }
}

/// The launch conditions of S-0506 claim 6.
fn can_launch(w: &World, spawner: ObjectId, c: &SpawnController, missile: bool) -> bool {
    let t = w.tick;
    let q = w.rules().quirks;
    let Some(o) = w.obj(spawner) else {
        return false;
    };
    // spec: S-0506 claim 6 (a controller target)
    if c.target.is_none() {
        return false;
    }
    // spec: S-0506 claim 6 (s = 9 when the spawner's own type has the missile-spawn flag, else 20; quirks.launch_spacing_from_spawner_type off: the spawned type's flag)
    let flag = if q.launch_spacing_from_spawner_type {
        type_missile_flag(w, o.kind, o.type_index)
    } else {
        spawn_rules(w, spawner).is_some_and(|r| type_missile_flag(w, Kind::Aircraft, r.spawned))
    };
    let s = if flag { 9 } else { 20 }; // spec: S-0506 claim 6 and Constants (launch spacing 9 and 20 ticks)
    if let Some(last) = c.last_launch
        && t.wrapping_sub(last) < s
    {
        return false;
    }
    // spec: S-0506 claim 6 (not cooling down)
    if c.state == ControllerState::CoolingDown {
        return false;
    }
    // spec: S-0506 claim 6 (a mobile spawner that is not seized, S-0607 claims 5 and 20)
    if o.kind.is_mobile() && magnet_held(w, spawner) {
        return false;
    }
    // spec: S-0506 claim 6 (a missile slot only: the spawner's locomotor reports that it is not moving)
    if missile && crate::occupancy::in_progress(w, spawner) {
        return false;
    }
    true
}

/// The placement facing of S-0506 Formulas F2.
pub fn placement_facing(f: u16) -> u16 {
    // spec: S-0506 F2 (f = ((⌊F / 128⌋ + 1) div 2) mod 256; facing = 256 × f)
    let f = ((i32::from(f) / 128 + 1) / 2).rem_euclid(256);
    (256 * f) as u16 // spec: S-0506 F2 (facing = 256 × f)
}

/// Launches slot `i`'s object (S-0506 claim 7).
fn launch(w: &mut World, spawner: ObjectId, i: usize) {
    let t = w.tick;
    let missile = missile_slot(w, spawner);
    if missile {
        // spec: S-0506 claim 3 (a rocket slot launching adopts the pending target)
        adopt_pending(w, spawner);
    }
    let Some(c) = controller(w, spawner).cloned() else {
        return;
    };
    let Some(x) = c.slots[i].object else { return };
    let Some(s) = w.obj(spawner).cloned() else {
        return;
    };
    let rules = w.rules_arc();
    let (wslot, _) = spawner_weapon(w, spawner).unwrap_or((0, 0));
    // spec: S-0506 claim 7 (the primary weapon's burst above 1: a missile slot alternates by k mod 2, k the slot index)
    let primary_burst = crate::combat::weapon::slot_weapon(w, spawner, 0)
        .map_or(1, |wp| rules.combat.weapon(wp).burst);
    let k = if missile && primary_burst > 1 {
        (i % 2) as i32 // spec: S-0506 claim 7 (shot number k mod 2)
    } else {
        0
    };
    let fp = crate::combat::position::fire_point(w, spawner, wslot, k);
    // spec: S-0506 claim 7 and Constants (raised by 10 leptons)
    let mut pos = Pos::new(fp.x, fp.y, fp.z + 10);
    let set = rocket_type(w, x);
    if set == Some(RocketSetId::C) {
        // spec: S-0506 claim 7 and Constants (the set-C type: 40 leptons towards map north and map west)
        pos = Pos::new(pos.x - 40, pos.y - 40, pos.z);
        // The take-off animation here is cosmetic and no import key names it (S-0506 Import mapping).
    }
    let facing = placement_facing(s.body.value(t));
    let target = controller(w, spawner).and_then(|c| c.target);
    if missile {
        crate::rocket::place(w, x, pos, facing, spawner);
        let stale = rules.quirks.stale_suspended_revert; // spec: S-0215 Edge cases (quirks.stale_suspended_revert)
        // spec: S-0506 claim 7 (r37: a controller target whose in-the-air answer is true gives the missile no destination at all, neither that target nor any point of its own)
        let dest = target.filter(|&d| match d {
            Target::Cell(_) => true,
            Target::Object(d) => !crate::combat::weapon::in_air(w, d),
        });
        if let Some(o) = w.obj_mut(x) {
            // spec: S-0506 claim 7 (a missile slot's object: the controller target as destination and the move behaviour)
            o.dest = dest;
            o.beh.force(crate::rules::Behaviour::Move, t, stale);
        }
        // spec: S-0506 claim 14 (r34: a destination whose in-the-air answer is false at the launch: the aim point is fixed at that launch, in the launch pass, in the tick of the placement; r37: with no destination the missile waits with no flight)
        if let Some(dest) = dest {
            crate::rocket::fix_at_launch(w, x, dest);
        }
    } else {
        crate::objects::place(w, x, pos, facing);
        // spec: S-0506 claim 7 (a non-missile slot's object: sent to the step direction 0 neighbour of the spawner's cell)
        let (dx, dy) = crate::regions::STEPS[0];
        let cell = s.pos.cell();
        aircraft_launched(w, x, CellXY::new(cell.x + dx, cell.y + dy));
    }
    if let Some(c) = controller_mut(w, spawner) {
        // spec: S-0506 claim 7 (the slot becomes preparing); claim 6 (the spacing restarts)
        c.slots[i].state = SlotState::Preparing;
        c.last_launch = Some(t);
    }
}

/// The release of S-0506 claim 8; whether a missile was released.
fn release(w: &mut World, spawner: ObjectId) -> bool {
    let t = w.tick;
    let q = w.rules().quirks;
    let Some(c) = controller(w, spawner).cloned() else {
        return false;
    };
    let r = spawn_rules(w, spawner).unwrap_or_default();
    let mut any = false;
    for (i, slot) in c.slots.iter().enumerate() {
        if slot.state != SlotState::Preparing {
            continue;
        }
        let Some(x) = slot.object else { continue };
        if missile_type(w, x) {
            any = true;
            // spec: S-0506 claim 8 (an object with the flag is released with the controller target, claim 12)
            release_object(w, x, c.target);
            match rocket_type(w, x) {
                Some(set) => {
                    // spec: S-0506 claim 8 (the rail time: pause + tilt of set A for the set-A type, set B otherwise; quirks.raised_missile_rail_time_from_set_b off: the set's own)
                    let rail_set = match set {
                        RocketSetId::A => RocketSetId::A,
                        RocketSetId::C if !q.raised_missile_rail_time_from_set_b => RocketSetId::C,
                        _ => RocketSetId::B,
                    };
                    let p = *w.rules().missiles.set(rail_set);
                    set_slot(
                        w,
                        spawner,
                        i,
                        SlotState::OnRail,
                        t.wrapping_add(p.pause).wrapping_add(p.tilt),
                    );
                }
                None => {
                    // spec: S-0506 claim 8 (a flagged object outside the rocket types: detached at once; its slot starts regenerating)
                    empty_slot(w, spawner, i, t.wrapping_add(r.regen));
                }
            }
        } else {
            // spec: S-0506 claim 8 (an object without the flag is set on the controller target and becomes attacking)
            set_slot(w, spawner, i, SlotState::Attacking, slot.until);
            if let Some(tg) = c.target {
                aircraft_attack(w, x, tg);
            }
        }
    }
    if any {
        // spec: S-0506 claim 26 (a normal salvo release replaces the shared deadline with the release tick + 2: a replacement, not a minimum)
        w.spawned.next_reorder = t.wrapping_add(REORDER_DELAY);
    }
    any
}

// spec: S-0506 claim 26 and Constants (salvo/reset re-order delay: 2 ticks after release)
const REORDER_DELAY: i32 = 2;
// spec: S-0506 claims 12 and 26 and Constants (release re-order interval: 30 ticks)
const REORDER_INTERVAL: i32 = 30;

/// Releases a slot object (S-0506 claim 12): a missile becomes a kamikaze
/// with its release cell; an object without the missile-spawn flag starts
/// to crash at once.
fn release_object(w: &mut World, x: ObjectId, target: Option<Target>) {
    if !missile_type(w, x) {
        // spec: S-0506 claim 12 (a released object without the missile-spawn flag starts to crash at once, S-0312 claim 16)
        start_crash(w, x);
        return;
    }
    let Some(o) = w.obj(x) else { return };
    // spec: S-0506 claim 12 (the release cell: the cell under the target's position, else the F8 neighbour of its own cell)
    let target_cell = target.and_then(|t| match t {
        Target::Cell(c) => Some(c),
        Target::Object(y) => w.obj(y).map(|yo| yo.pos.cell()),
    });
    let cell = target_cell.unwrap_or_else(|| {
        let k = release_direction(o.body.value(w.tick));
        let (dx, dy) = crate::regions::STEPS[k];
        let c = o.pos.cell();
        CellXY::new(c.x + dx, c.y + dy)
    });
    // spec: S-0506 claim 26 (enrollment order: a new enrollment follows the earlier ones)
    if let Some(e) = w.spawned.kamikaze.iter_mut().find(|e| e.0 == x) {
        e.1 = cell;
    } else {
        w.spawned.kamikaze.push((x, cell));
    }
}

/// The step direction of S-0506 Formulas F8.
pub fn release_direction(f: u16) -> usize {
    // spec: S-0506 F8 (k = ((⌊F / 4096⌋ + 1) div 2) mod 8)
    ((i32::from(f) / 4096 + 1) / 2).rem_euclid(8) as usize
}

/// Starts a released aircraft's crash (S-0312 claim 16): a flying aircraft
/// above the terrain gets health 0 and falls as a dead aircraft (S-0501
/// claims 9 and 36); any other is destroyed at once with no attacker.
fn start_crash(w: &mut World, x: ObjectId) {
    let Some(o) = w.obj(x) else { return };
    if crate::combat::damage::dead(o) {
        return;
    }
    if crate::air::is_aircraft(w, x) && crate::air::height(w, x) > 0 {
        // spec: S-0501 claim 36 (starting this crash sets health to 0, not a living disabled fall; its unfinished flight and speed fraction are kept and ordinary movement continues)
        // spec: S-0501 claim 9 (it falls as a dead aircraft, and its impact effects and removal follow S-0312 claim 16 for one shot down): our engine's dead state is killed with no health left (`damage::dead`), with none of a killing hit's other effects, which S-0501 claim 36 does not give the release
        if let Some(ob) = w.obj_mut(x) {
            ob.health = 0;
            ob.combat.killed = true;
        }
        if crate::air::start_crash(w, x, false) {
            return;
        }
    }
    if let Some(ob) = w.obj_mut(x) {
        ob.health = 0;
    }
    crate::combat::death::kill(
        w,
        x,
        &crate::combat::damage::Hit {
            amount: 0,
            warhead: None,
            distance: 0,
            attacker: None,
            house: None,
            ignore_defences: true,
            no_escape: true,
        },
    );
}

// ---------------------------------------------------------------------
// Reset and clearing (claims 11 and 13)
// ---------------------------------------------------------------------

/// The controller resets (S-0506 claim 11): every preparing object of a
/// missile type is released with the controller target and its slot
/// becomes empty with a regeneration countdown; the controller target and
/// the pending target are cleared and the controller becomes idle.
pub fn reset(w: &mut World, spawner: ObjectId) {
    let t = w.tick;
    let Some(c) = controller(w, spawner).cloned() else {
        return;
    };
    let r = spawn_rules(w, spawner).unwrap_or_default();
    let mut any = false;
    // spec: S-0506 claim 26 (within a reset, enrollment follows increasing slot index)
    for (i, slot) in c.slots.iter().enumerate() {
        if slot.state != SlotState::Preparing {
            continue;
        }
        let Some(x) = slot.object else { continue };
        // spec: S-0506 claim 11 (a non-missile preparing object is not affected)
        if !missile_type(w, x) {
            continue;
        }
        any = true;
        release_object(w, x, c.target);
        empty_slot(w, spawner, i, t.wrapping_add(r.regen));
    }
    if any {
        // spec: S-0506 claim 26 (a reset that actually releases a preparing flagged aircraft replaces the shared deadline with the release tick + 2; one that releases none changes no deadline)
        w.spawned.next_reorder = t.wrapping_add(REORDER_DELAY);
    }
    if let Some(c) = controller_mut(w, spawner) {
        c.target = None;
        c.pending = None;
        c.state = ControllerState::Idle;
    }
}

/// The controller clears its slots (S-0506 claim 13), in descending slot
/// index. The controller target is left as it is (S-0506 r20 claim 13:
/// clearing the slots does not change it, and the controller's passes go
/// on while the spawner is seized).
pub fn clear_slots(w: &mut World, spawner: ObjectId) {
    let t = w.tick;
    let q = w.rules().quirks;
    let Some(c) = controller(w, spawner).cloned() else {
        return;
    };
    let r = spawn_rules(w, spawner).unwrap_or_default();
    // spec: S-0506 claim 13 (regeneration time when the spawner has health below 1 or is out of play, else 0)
    let out = w.obj(spawner).is_none_or(|o| o.health < 1 || !o.on_map);
    let until = if out { t.wrapping_add(r.regen) } else { t };
    // spec: S-0506 claim 13 (crash-start draws in descending slot index); claim 26 (enrollment in descending slot index, without replacing the shared deadline)
    for i in (0..c.slots.len()).rev() {
        let slot = c.slots[i].clone();
        let Some(x) = slot.object else { continue };
        // The slot is emptied first, so the removals below find no slot.
        empty_slot(w, spawner, i, until);
        match slot.state {
            SlotState::Empty => {}
            SlotState::Docked | SlotState::Reloading => {
                // spec: S-0506 claim 13 (docked, reloading: removed silently)
                remove_silently(w, x);
            }
            SlotState::OnRail if q.rail_missile_vanishes_with_spawner => {
                // spec: S-0506 claim 13 (on the rail: removed silently, no detonation; quirks.rail_missile_vanishes_with_spawner)
                remove_silently(w, x);
            }
            SlotState::OnRail
            | SlotState::Preparing
            | SlotState::Attacking
            | SlotState::Returning => {
                // spec: S-0506 claim 13 (preparing, attacking, returning, and on the rail with the quirk off: released with the controller target)
                release_object(w, x, c.target);
            }
        }
    }
}

/// Removes a slot object silently (S-0506 claim 13).
fn remove_silently(w: &mut World, x: ObjectId) {
    crate::combat::flying::unregister(w, x);
    crate::objects::remove(w, x);
}

/// The spawner changed house (S-0506 claim 13: mind control, capture).
/// Called by `mind` on a takeover, a domination or a release and by
/// `structures::ownership` on a capture.
pub fn owner_changed(w: &mut World, spawner: ObjectId) {
    clear_slots(w, spawner);
}

/// A killing hit on an object (S-0506 claims 9, 11 and 13; S-0312): a slot
/// object is announced as lost; a spawner clears its slots unless it is
/// seized (S-0607 claims 5 and 20); the object is lost as a target.
pub fn killed(w: &mut World, id: ObjectId) {
    lost(w, id);
    if w.obj(id).is_some_and(|o| o.spawn.is_some()) && !magnet_held(w, id) {
        // spec: S-0506 claim 13 (the spawner destroyed or starting to crash, unless it is seized)
        clear_slots(w, id);
    }
    // spec: S-0506 claim 11 (r37: the controller target and a pending target are lost at the hit that destroys them, even while the target then stays in play: a death sequence, a sinking or crashing object)
    target_lost(w, id);
}

/// An object is about to be removed from play (S-0506 claims 9, 11 and
/// 13): a spawner clears its slots; a slot object is announced as lost;
/// controllers whose target it is reset unless a pending target waits.
pub fn removing(w: &mut World, id: ObjectId) {
    settle_waiting(w);
    if w.obj(id).is_some_and(|o| o.spawn.is_some()) {
        // spec: S-0506 claim 13 (the spawner removed)
        clear_slots(w, id);
    }
    lost(w, id);
    w.spawned.launcher.remove(&id);
    // spec: S-0506 claim 26 (removing an enrolled aircraft keeps the others' relative order)
    w.spawned.kamikaze.retain(|&(k, _)| k != id);
    crate::combat::flying::unregister(w, id);
    target_gone(w, id);
}

/// An object leaves play by going inside a building or a transport (S-0506
/// claim 14, r34): for the controllers, a target leaving play as at a
/// removal. A pending target that goes inside is cleared (claim 25's loss
/// notification); a controller whose target goes inside resets unless a
/// pending target waits (claim 11). The object's own slots and its slot are
/// not touched: it stays in existence.
pub fn target_leaves_play(w: &mut World, id: ObjectId) {
    settle_waiting(w);
    // spec: S-0506 claim 14 (r34: a destination that goes inside a building or a transport leaves play; when it is the controller target and no pending target is waiting, claim 11's reset follows when it leaves play)
    target_gone(w, id);
}

/// Clear requests recorded earlier are settled before the loss rules read
/// pending and controller targets.
fn settle_waiting(w: &mut World) {
    let waiting: Vec<ObjectId> = w
        .objects
        .iter()
        .filter(|(_, o)| o.spawn.as_ref().is_some_and(|c| c.clear_request.is_some()))
        .map(|(&k, _)| k)
        .collect();
    for s in waiting {
        settle_clear(w, s);
    }
}

/// The loss of `id` as a target (S-0506 claims 11 and 25, r37) at an event
/// that keeps it in existence: the destroying hit (also of a target that
/// then stays in play: a death sequence, a sinking, a crash) and the start
/// of cloaking, whatever the spawner's house and sensors. Clear requests
/// recorded earlier are settled first. Going inside and removal have their
/// own hooks ([`target_leaves_play`], [`removing`]).
pub fn target_lost(w: &mut World, id: ObjectId) {
    // design: the clear requests recorded earlier (S-0506 claim 25) are settled before the loss reads the pending and controller targets, as `removing` and `target_leaves_play` place it; no spec orders a settle at the other loss events
    settle_waiting(w);
    target_gone(w, id);
}

/// The controllers' side of the loss of `id` (S-0506 claims 11 and 25).
fn target_gone(w: &mut World, id: ObjectId) {
    let gone = Some(Target::Object(id));
    // spec: S-0506 claim 25 (loss notification for the pending target clears it at once, whatever the own target)
    let holders: Vec<ObjectId> = w
        .objects
        .iter()
        .filter(|(_, o)| o.spawn.as_ref().is_some_and(|c| c.pending == gone))
        .map(|(&k, _)| k)
        .collect();
    for s in holders {
        if let Some(c) = w.obj_mut(s).and_then(|o| o.spawn.as_mut()) {
            c.pending = None;
        }
    }
    // spec: S-0506 claim 11 (r37: the controller target is lost and no pending target is waiting: reset); design: Q-2811 interim (with a pending target waiting, the controller target is left as it is and nothing resets)
    let resets: Vec<ObjectId> = w
        .objects
        .iter()
        .filter(|(_, o)| {
            o.spawn
                .as_ref()
                .is_some_and(|c| c.target == gone && c.pending.is_none())
        })
        .map(|(&k, _)| k)
        .collect();
    for s in resets {
        // spec: S-0506 claims 11 and 12 (r37: the controller target is cleared as it is lost, before the reset, so the reset releases with no target and its missiles get F8's cell)
        if let Some(c) = controller_mut(w, s) {
            c.target = None;
        }
        reset(w, s);
    }
}

/// A slot object is announced as lost to its controller (S-0506 claim 9):
/// its slot becomes empty with the regeneration countdown when its health
/// is below 1, it is a kamikaze, or it is of a rocket type; otherwise the
/// slot is unchanged.
pub fn lost(w: &mut World, x: ObjectId) {
    let Some(spawner) = w.spawned.launcher.get(&x).copied() else {
        return;
    };
    let Some(c) = controller(w, spawner) else {
        return;
    };
    let Some(i) = c
        .slots
        .iter()
        .position(|s| s.object == Some(x) && s.state != SlotState::Empty)
    else {
        return;
    };
    let Some(o) = w.obj(x) else { return };
    // spec: S-0506 claim 9 (health below 1, a kamikaze, or a rocket type: empty with the regeneration countdown; otherwise unchanged)
    if o.health < 1 || kamikaze(w, x) || rocket_type(w, x).is_some() {
        let r = spawn_rules(w, spawner).unwrap_or_default();
        empty_slot(w, spawner, i, w.tick.wrapping_add(r.regen));
    }
}

// ---------------------------------------------------------------------
// Hooks into the aircraft flight (S-0501 claims 35 and 36, table T4)
// ---------------------------------------------------------------------

/// A non-missile slot aircraft is launched (or re-sent while preparing)
/// towards `cell` (S-0506 claim 7 and Slot table): a flying aircraft gets
/// the destination with *move* queued (S-0501 T4).
pub fn aircraft_launched(w: &mut World, x: ObjectId, cell: CellXY) {
    if crate::air::is_aircraft(w, x) {
        crate::air::carrier_move(w, x, cell);
        return;
    }
    if let Some(o) = w.obj_mut(x) {
        o.dest = Some(Target::Cell(cell));
    }
}

/// A non-missile slot aircraft is set on the controller target (S-0506
/// claim 8 and Slot table): a flying aircraft receives it and queues
/// *attack* (S-0501 T4).
pub fn aircraft_attack(w: &mut World, x: ObjectId, target: Target) {
    if crate::air::is_aircraft(w, x) {
        crate::air::carrier_attack(w, x, target);
        return;
    }
    if let Some(o) = w.obj_mut(x) {
        o.target = Some(target);
    }
    // spec: S-0301 claim 5, Moment of a retarget's let-go (revision 67, Q-2246: at the change of the firer's target)
    crate::emitters::retarget_let_go(w, x);
}

/// A non-missile slot aircraft flies back to its spawner (S-0506 Slot
/// table): it loses its target and gets the spawner as its destination,
/// with *move* for a flying aircraft (S-0501 T4). Docking is tested in each
/// pass by [`docks`].
pub fn aircraft_return(w: &mut World, x: ObjectId, spawner: ObjectId) {
    if crate::air::is_aircraft(w, x) {
        crate::air::carrier_return(w, x, spawner);
        return;
    }
    if let Some(o) = w.obj_mut(x) {
        o.target = None;
        crate::emitters::cleared(o);
        o.dest = Some(Target::Object(spawner));
    }
}

/// The shared re-order cycle of released missiles (S-0506 claims 12 and
/// 26), once per tick from the tick pipeline, outside object turns. When
/// due, it schedules the next cycle for its own tick + 30 (even with no
/// kamikaze), then gives every enrolled aircraft, in enrollment order,
/// ammunition of exactly 1 round, its release cell as target and *attack*
/// queued without an immediate start. It takes no random draws. A rocket
/// missile receives the same changes; they do not alter a flight whose aim
/// point is already fixed (claim 14). A released rocket still waiting with
/// no flight (a controller target in the air at its launch, claim 14 r37)
/// is given its release cell as a destination (Q-2748 interim).
pub fn kamikaze_orders(w: &mut World) {
    let t = w.tick;
    // spec: S-0506 claim 26 (the cycle is due on its deadline tick; tick numbers only grow)
    if t < w.spawned.next_reorder {
        return;
    }
    // spec: S-0506 claim 26 (a cycle schedules the next for its own tick + 30, even when no kamikaze exists)
    w.spawned.next_reorder = t.wrapping_add(REORDER_INTERVAL);
    let list = w.spawned.kamikaze.clone();
    // spec: S-0506 claim 26 (orders in enrollment order)
    for (x, cell) in list {
        let Some(o) = w.obj_mut(x) else { continue };
        // spec: S-0506 claim 26 (ammunition set to exactly 1 round)
        o.combat.ammo = 1;
        // spec: S-0506 claims 12 and 26 (target: the release cell)
        o.target = Some(Target::Cell(cell));
        // spec: S-0506 claim 26 (attack queued, without forcing an immediate behaviour start; no draws)
        o.beh.queue(Some(crate::rules::Behaviour::Attack));
        let waiting = o.rocket.as_ref().is_some_and(|rs| rs.phase.is_none());
        // spec: S-0301 claim 5, Moment of a retarget's let-go (revision 67, Q-2246: at the change of the firer's target)
        crate::emitters::retarget_let_go(w, x);
        if waiting {
            // design: Q-2748 interim (questions 2 and 3 still open: a released rocket waiting with no flight is given its release cell, at this attack on it, as the first destination not in the air; S-0506 claim 14 r37 then fixes its aim)
            crate::rocket::give_destination(w, x, Target::Cell(cell));
        }
    }
}

/// The launch elite test of S-0506 claim 19: the spawner exists and has
/// the elite rank.
pub fn launcher_elite(w: &World, spawner: Option<ObjectId>) -> bool {
    spawner.is_some_and(|s| {
        w.obj(s).is_some()
            && crate::combat::veterancy::rank(w, s) == crate::combat::veterancy::Rank::Elite
    })
}
