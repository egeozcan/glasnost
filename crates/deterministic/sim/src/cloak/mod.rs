//! Cloak and stealth (S-0516): the four cloak states and their stages,
//! cloak fields of generator buildings, per-house sensor and
//! disguise-detector coverage, disguise, and the submerged state of
//! underwater vessels that falls out of them (claim 41).
//!
//! Game state (what a house may target or fire on) never depends on a
//! viewer. The presentation tests of claims 26 to 29, 37 and 38 are the
//! read-only queries of [`view`].
//!
//! The per-object state is [`ObjCloak`] (`Object::cloak`); the per-house,
//! per-cell coverage is [`CloakWorld`] (`World::cloak`). The machine runs in
//! the object's own turn ([`turn`], called from `econ::events::turn`), and
//! the event hooks of `econ::events` call [`entered_map`], [`leaving_map`]
//! and [`owner_changed`].
//!
//! Seams to specs that are not built (each named where it is used):
//! - S-0607 (seized), S-0214 (paralysed), the special-weapons specs (EMP
//!   through `econ::power::under_pulse`, warping through
//!   `econ::power::being_erased`; chrono warp is S-0514's);
//! - S-0519 (C4 planting: [`machine::uncloak`] is the call it makes);
//! - S-0520 claim 15 (cloaking crates: [`machine::grant_own_flag`]);
//! - S-0517 claim 6 (undeploy disguise loss: [`disguise::clear`]);
//! - S-0510 claim 2 (infiltration uses the spy up);
//! - S-0309 claim 25 (g) (the neutral and special houses' blinking-disguise
//!   draw: [`disguise::neutral_sees_through`]);
//! - S-0210 claim 11 (automatic crush choice: [`appears_disguised`]);
//! - S-0414 (a house leaving the game: [`disguise::house_left`]);
//! - S-0518, S-0525 and S-06xx (psychic detection and the special-weapon
//!   effects that would call [`machine::uncloak`]);
//! - S-0502 (jumpjets: the height test of claim 4 already covers them);
//! - S-0508 and S-0509 (occupied buildings and tank bunkers: a hit stopped
//!   by a bunker does not uncloak, claim 12);
//! - S-0515 (mind control: an owner change, [`owner_changed`]).

pub mod disguise;
pub mod field;
pub mod machine;
pub mod sensor;
pub mod view;

use std::collections::{BTreeMap, BTreeSet};

use glasnost_map::CellXY;
use glasnost_num::{F64, Rounding};
use serde::{Deserialize, Serialize};

use crate::rules::Kind;
use crate::world::{HouseId, ObjectId, World};

pub use disguise::{Disguise, DisguiseAs, appears_disguised};
pub use machine::{entered_map, leaving_map, owner_changed, turn, uncloak};

/// The cloak states (S-0516 Terms, claim 1).
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub enum CloakState {
    /// Fully visible.
    #[default]
    Uncloaked,
    /// Fading out.
    Cloaking,
    /// Fully cloaked.
    Cloaked,
    /// Fading in.
    Uncloaking,
}

/// One building's added coverage (S-0516 claims 23 and 40): the house it was
/// added for, the centre cell and the radius.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cover {
    /// The house the coverage was added for.
    pub house: HouseId,
    /// The disc's centre cell.
    pub centre: CellXY,
    /// The disc's radius.
    pub radius: i32,
}

/// A cloak generator's field (S-0516 claims 19 and 20).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Generator {
    /// The house the field covers.
    pub house: HouseId,
    /// The centre cell.
    pub centre: CellXY,
    /// The applied radius, 0 for no field (S-0516 claim 20: radius 0 is an
    /// empty field).
    pub radius: i32,
    /// Whether growth or shrinking is pending (S-0516 claim 20, Terms
    /// *idle generator*).
    #[serde(default)]
    pub phase: FieldPhase,
    /// Whether the building was in operation at its last turn (S-0516 claim
    /// 20: the field reacts to changes in operation, not to its level). A
    /// new generator records out of operation, so its first turn in
    /// operation is a change.
    #[serde(default)]
    pub was_on: bool,
}

/// A cloak generator's field state (S-0516 Terms, claim 20): idle means
/// neither growth nor shrinking is pending, independently of the radius and
/// of being in operation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum FieldPhase {
    /// Neither growth nor shrinking pending.
    #[default]
    Idle,
    /// Growth pending: the next growth update adds a radius, or ends growth
    /// when the radius is already full.
    Growing,
    /// Shrinking pending: the next shrink update removes a radius, or ends
    /// shrinking at radius 0.
    Shrinking,
}

/// An object's cloak and disguise state (S-0516).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObjCloak {
    /// The cloak state (claim 1).
    pub state: CloakState,
    /// The stage *k* of a fading mobile object, or a building's fade level
    /// (claims 1 and 18).
    pub stage: i32,
    /// The tick of the last stage step, or of the fade's start.
    pub stepped: i32,
    /// The tick at which the cloak delay ends (claim 7).
    pub delay_until: i32,
    /// The own cloak flag (claim 2 (a), (d)).
    pub own: bool,
    /// The cell under its centre at its last turn or entry (claims 6, 13
    /// and 22).
    pub cell: Option<CellXY>,
    /// A mobile sensor's added disc: house and centre (claim 22).
    pub disc: Option<(HouseId, CellXY)>,
    /// A building has had its first placement (claim 23).
    pub placed: bool,
    /// A sensor array's added coverage (claim 23).
    pub array: Option<Cover>,
    /// A disguise detector's added coverage (claim 40).
    pub detector: Option<Cover>,
    /// A cloak generator's field (claims 19 and 20).
    pub field: Option<Generator>,
    /// The disguise (claim 30).
    pub disguise: Option<Disguise>,
    /// The tick at which the fake blink ends (claim 39).
    pub blink_until: i32,
    /// The tick at which the re-disguise lockout ends (claim 33).
    pub lockout_until: i32,
}

/// The per-house, per-cell coverage (S-0516 Terms, claims 20 to 23, 40).
/// Keys are (house index, cell).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CloakWorld {
    /// Cloak-field flags (claim 20: a flag, not a count).
    pub fields: BTreeSet<(u8, CellXY)>,
    /// Sensor coverage counts, zero entries dropped (claim 21).
    pub sensors: BTreeMap<(u8, CellXY), i32>,
    /// Disguise-detector coverage counts, zero entries dropped (claim 40).
    pub detectors: BTreeMap<(u8, CellXY), i32>,
}

/// The game-wide cloak and disguise values (S-0516 Import mapping).
#[derive(Clone, Debug, Default)]
pub struct CloakRules {
    /// *N*, the number of cloaking stages.
    pub stages: i32,
    /// *D*, the cloak delay in ticks (F3).
    pub delay: i32,
    /// The game-wide cloak sound.
    pub sound: Option<String>,
    /// The side default disguises: allied, soviet, third (infantry types).
    pub side_disguise: [Option<u32>; 3], // spec: S-0516 claim 32 (three side defaults)
    /// The default-disguise list (terrain-object types).
    pub mirage: Vec<u32>,
    /// The infantry-blink lockout, ticks.
    pub blink: i32,
    /// The attack cursor on disguised units (S-0214 claim 6).
    pub attack_cursor_on_disguise: bool,
}

impl CloakRules {
    /// The cloak rules of a ruleset.
    pub fn new(
        g: &glasnost_rules::sim::General,
        names: &BTreeMap<(Kind, String), u32>,
        mode: Rounding,
    ) -> Self {
        let idx = |k: Kind, n: &Option<String>| -> Option<u32> {
            n.as_ref().and_then(|n| names.get(&(k, n.clone())).copied())
        };
        // spec: S-0516 Import mapping (CloakDelay absent: 0.0 minutes, a decimal per S-0003 claims 6 and 7)
        let d = g.cloak_delay.as_ref().map_or(F64::ZERO, |d| d.value(mode));
        Self {
            // spec: S-0516 Import mapping (CloakingStages absent: 9) and claim 43 (a value of 0 or below is taken as 1)
            stages: g.cloaking_stages.unwrap_or(9).max(1),
            delay: delay_ticks(d, mode),
            // spec: S-0516 Import mapping (CloakSound absent: none)
            sound: g.cloak_sound.clone(),
            // spec: S-0516 Import mapping (AlliedDisguise, SovietDisguise, ThirdDisguise absent: none)
            side_disguise: [
                idx(Kind::Infantry, &g.allied_disguise),
                idx(Kind::Infantry, &g.soviet_disguise),
                idx(Kind::Infantry, &g.third_disguise),
            ],
            // spec: S-0516 Import mapping (DefaultMirageDisguises absent: empty)
            mirage: g
                .default_mirage_disguises
                .iter()
                .flatten()
                .filter_map(|n| names.get(&(Kind::Terrain, n.clone())).copied())
                .collect(),
            // spec: S-0516 Import mapping (InfantryBlinkDisguiseTime absent: 0 ticks)
            blink: g.infantry_blink_disguise_time.unwrap_or(0),
            // spec: S-0516 Import mapping (AttackCursorOnDisguise absent: no)
            attack_cursor_on_disguise: g.attack_cursor_on_disguise.unwrap_or(false),
        }
    }
}

/// F3: *D* = trunc(*d* × 900.0), the product rounded to 53 bits in the
/// game's mode and truncated toward zero.
pub fn delay_ticks(d: F64, mode: Rounding) -> i32 {
    // spec: S-0516 F3 (D = trunc(d × 900.0), 900 ticks per minute)
    d.mul(F64::from_i64(900, mode), mode).to_i32_common()
}

/// The fade fraction *x* = ⌊256 *k* / *N*⌋ of stage *k* (S-0516 Terms, F1).
pub fn fade_fraction(k: i32, n: i32) -> i32 {
    if n <= 0 {
        return 0;
    }
    // spec: S-0516 Terms and F1 (x = ⌊256 k / N⌋, the fade-fraction scale 256)
    ((256 * i64::from(k)) / i64::from(n)) as i32
}

/// *c* = ⌈*N* / 2⌉, the stage at which cloaking completes (S-0516 claim 7,
/// F1).
pub fn cloaked_stage(n: i32) -> i32 {
    // spec: S-0516 claim 7 and F1 (c = ⌈N / 2⌉)
    (n + 1) / 2
}

/// F1: the cloaking time *c* · *S*.
pub fn cloak_time(n: i32, s: i32) -> i32 {
    // spec: S-0516 F1 (T_cloak = c × S)
    cloaked_stage(n) * s
}

/// F2: the uncloaking time (*N* − 1) · *S*.
pub fn uncloak_time(n: i32, s: i32) -> i32 {
    // spec: S-0516 F2 (T_uncloak = (N − 1) × S)
    (n - 1) * s
}

/// The displacements of the sensor disc of radius `r` (S-0516 Terms):
/// −R ≤ dx, dy ≤ R − 1 and dx² + dy² < R²; empty for R ≤ 0.
pub fn sensor_disc(r: i32) -> Vec<(i32, i32)> {
    let mut out = Vec::new();
    if r <= 0 {
        return out;
    }
    let r2 = i64::from(r) * i64::from(r);
    // spec: S-0516 Terms (sensor disc: −R ≤ dx ≤ R − 1, −R ≤ dy ≤ R − 1, dx² + dy² < R²)
    for dy in -r..r {
        for dx in -r..r {
            if i64::from(dx) * i64::from(dx) + i64::from(dy) * i64::from(dy) < r2 {
                out.push((dx, dy));
            }
        }
    }
    out
}

/// The cloak-field circle of radius `r` (S-0516 claim 19, F5): the union of
/// the four spans of every midpoint step with *x* ≤ *y*. Empty for `r` ≤ 0
/// (S-0516 claim 20: radius 0 is an empty field, not F5's radius-zero circle).
pub fn field_circle(r: i32) -> BTreeSet<(i32, i32)> {
    let mut out = BTreeSet::new();
    if r <= 0 {
        return out;
    }
    // spec: S-0516 F5 (x₀ = 0, y₀ = r, e₀ = 3 − 2r)
    let (mut x, mut y, mut e) = (0i32, r, 3 - 2 * r);
    while x <= y {
        // spec: S-0516 F5 (rows dy = ±x for |dx| ≤ y, rows dy = ±y for |dx| ≤ x)
        for dx in -y..=y {
            out.insert((dx, x));
            out.insert((dx, -x));
        }
        for dx in -x..=x {
            out.insert((dx, y));
            out.insert((dx, -y));
        }
        if e < 0 {
            // spec: S-0516 F5 (e < 0: y kept, e + 4x + 6)
            e += 4 * x + 6;
        } else {
            // spec: S-0516 F5 (e ≥ 0: y − 1, e + 4(x − y) + 10)
            e += 4 * (x - y) + 10;
            y -= 1;
        }
        x += 1;
    }
    out
}

/// The cell under an object's centre (S-0516 Terms): its reference point's
/// cell (a building's foundation centre, S-0204 claim 4).
pub fn centre_cell(w: &World, id: ObjectId) -> CellXY {
    w.reference_point(id).cell()
}

/// An object's cloak state.
pub fn state(w: &World, id: ObjectId) -> CloakState {
    w.obj(id).map_or(CloakState::Uncloaked, |o| o.cloak.state)
}

/// Whether an object is cloaking, cloaked or uncloaking.
pub fn not_uncloaked(w: &World, id: ObjectId) -> bool {
    state(w, id) != CloakState::Uncloaked
}

/// Whether an object is fully cloaked (S-0516 Terms).
pub fn fully_cloaked(w: &World, id: ObjectId) -> bool {
    state(w, id) == CloakState::Cloaked
}

/// Whether house `h` senses cell `c` (S-0516 claim 21: its count above 0).
pub fn sensed(w: &World, h: Option<HouseId>, c: CellXY) -> bool {
    h.is_some_and(|h| w.cloak.sensors.get(&(h.0, c)).is_some_and(|&n| n > 0))
}

/// Whether house `h` senses an object (S-0516 Terms: the cell under its
/// centre).
pub fn senses(w: &World, h: Option<HouseId>, id: ObjectId) -> bool {
    sensed(w, h, centre_cell(w, id))
}

/// Whether cell `c` is in house `h`'s cloak field (S-0516 Terms).
pub fn in_field(w: &World, h: Option<HouseId>, c: CellXY) -> bool {
    h.is_some_and(|h| w.cloak.fields.contains(&(h.0, c)))
}

/// Whether the cell under an object's centre is in its owner's cloak field
/// (S-0516 claim 2 (c)).
pub fn in_own_field(w: &World, id: ObjectId) -> bool {
    w.obj(id)
        .is_some_and(|o| in_field(w, o.owner, centre_cell(w, id)))
}

/// Whether house `h` has disguise-detector coverage on cell `c` (S-0516
/// Terms, claim 40).
pub fn detected(w: &World, h: Option<HouseId>, c: CellXY) -> bool {
    h.is_some_and(|h| w.cloak.detectors.get(&(h.0, c)).is_some_and(|&n| n > 0))
}

/// Whether a building counts as hidden by its fade level (S-0516 Terms:
/// levels 11 to 15).
fn building_hidden_level(o: &crate::world::Object) -> bool {
    // spec: S-0516 Terms and claim 18 (a building at fade levels 11 to 15)
    o.cloak.state != CloakState::Uncloaked && o.cloak.stage >= 11
}

/// Whether an object is hidden from house `h` (S-0516 Terms): fully cloaked
/// (a building also at fade levels 11 to 15) and not sensed by `h`. It
/// holds for the owner too.
pub fn hidden_from(w: &World, id: ObjectId, h: Option<HouseId>) -> bool {
    let Some(o) = w.obj(id) else { return false };
    let cloaked = if o.kind == Kind::Building {
        building_hidden_level(o)
    } else {
        o.cloak.state == CloakState::Cloaked
    };
    cloaked && !senses(w, h, id)
}

/// The cloak-field radius of a building type (S-0516 Import mapping: the
/// cloak radius, kept as a signed 8-bit value).
pub fn type_radius(w: &World, building_type: u32) -> i32 {
    // spec: S-0516 Import mapping (CloakRadiusInCells absent: 20, kept as signed 8-bit)
    let absent = 20;
    w.rules()
        .buildings
        .get(building_type as usize)
        .map_or(absent, |b| {
            i32::from(b.raw.cloak_radius_in_cells.unwrap_or(absent) as i8)
        })
}
