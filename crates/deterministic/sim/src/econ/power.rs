//! Power (S-0405): output and drain, low power and the power fraction,
//! blackouts, when a house works out its power again, the in-operation test
//! and what stops, the power switch, and the units that depend on a control
//! centre.
//!
//! Seams (states that belong to topics not written yet, each a small
//! function below that answers "no" or 0 until its topic arrives): pulses and
//! electromagnetic locks, being erased, being drained, chargers, absorbed
//! passengers, engineers inside a building, occupants of a garrisonable
//! building, controlled and held units, superweapon charge (claim 15) and the
//! super-gap's deploy behaviour (claim 16).

use std::collections::BTreeMap;

use glasnost_num::{F64, Rounding};
use glasnost_rules::schema::Ruleset;
use serde::{Deserialize, Serialize};

use crate::combat::check::Answer;
use crate::rules::{Behaviour, Kind};
use crate::world::{HouseId, Layer, ObjectId, World};

/// The power fields of a building type (S-0405 Terms and Import mapping).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BuildingPower {
    /// Base output (power field ≥ 0).
    pub output: i32,
    /// Base drain (−power field when it is negative).
    pub drain: i32,
    /// Extra output (extra power field ≥ 0).
    pub extra_output: i32,
    /// Extra drain (−extra power field when it is negative).
    pub extra_drain: i32,
    /// Absorbs infantry or vehicles as passengers (S-0312 Terms).
    pub absorbing: bool,
    /// The needs-power flag.
    pub needs_power: bool,
    /// The special-power flag.
    pub special_power: bool,
    /// The needs-engineer flag.
    pub needs_engineer: bool,
    /// The control centre's powered unit (a vehicle type index).
    pub powers_unit: Option<u32>,
    /// The super-gap radius in cells (claim 16).
    pub super_gap_radius: i32,
    /// The firestorm-wall flag (S-0213 claim 13: power orders do nothing).
    pub firestorm: bool,
    /// A pulse cannon (S-0214's type flag; T1).
    pub pulse_cannon: bool,
    /// Infantry may occupy it (S-0215; T1).
    pub occupiable: bool,
    /// Occupants may fire from it (glossary *garrisoned building*; T1).
    pub occupants_fire: bool,
    /// An upgrade type (S-0406 Terms): added to a building, not counted on
    /// its own (claim 3, Terms *upgrade positions*).
    pub upgrade: bool,
}

/// The power rules: per building type, per vehicle type, and game-wide.
#[derive(Clone, Debug, Default)]
pub struct Rules {
    /// Building types, by registration index.
    pub buildings: Vec<BuildingPower>,
    /// Vehicle types' powered-unit flag, by registration index (claim 13).
    pub powered_unit: Vec<bool>,
    /// The spy blackout length *d* in ticks (claim 6).
    pub spy_blackout: i32,
    /// The spy's steal fraction *f*, a single widened exactly (S-0510
    /// claim 6, F1).
    pub spy_steal: F64,
}

/// Splits a signed power field into (output, drain) (S-0405 Terms).
fn split(v: i32) -> (i32, i32) {
    // spec: S-0405 Terms (a value of 0 or more is the output; a negative value v gives a drain of −v)
    if v >= 0 {
        (v, 0)
    } else {
        (0, v.saturating_neg())
    }
}

impl Rules {
    /// The power rules of a resolved ruleset.
    pub fn new(r: &Ruleset, mode: Rounding) -> Self {
        let reg = |k: &str| r.registries.get(k).cloned().unwrap_or_default();
        let vehicles = reg("vehicles");
        let buildings = reg("buildings")
            .iter()
            .map(|n| {
                let b = r.sim.buildings.get(n).cloned().unwrap_or_default();
                let e = &b.econ;
                // spec: S-0405 Import mapping (Power and ExtraPower absent: 0)
                let (output, drain) = split(e.power.unwrap_or(0));
                let (extra_output, extra_drain) = split(e.extra_power.unwrap_or(0));
                BuildingPower {
                    output,
                    drain,
                    extra_output,
                    extra_drain,
                    // spec: S-0405 Import mapping (InfantryAbsorb, UnitAbsorb absent: no)
                    absorbing: b.infantry_absorb.unwrap_or(false) || b.unit_absorb.unwrap_or(false),
                    // spec: S-0405 Constants (needs-power, special-power, needs-engineer flags absent: no)
                    needs_power: b.powered.unwrap_or(false),
                    special_power: e.powered_special.unwrap_or(false),
                    needs_engineer: b.needs_engineer.unwrap_or(false),
                    // spec: S-0405 Import mapping (PowersUnit absent: none)
                    powers_unit: e
                        .powers_unit
                        .as_ref()
                        .and_then(|u| vehicles.iter().position(|v| v == u))
                        .map(|i| i as u32),
                    // spec: S-0405 Import mapping (SuperGapRadiusInCells absent: 0)
                    super_gap_radius: e.super_gap_radius_in_cells.unwrap_or(0),
                    // spec: S-0213 claim 13 (the firestorm-wall flag; absent: no)
                    firestorm: b.firestorm_wall.unwrap_or(false),
                    // spec: S-0405 T1 (the pulse cannon flag; absent: no)
                    pulse_cannon: b.emp_cannon.unwrap_or(false),
                    // spec: S-0405 T1 (occupiable and occupants-fire flags; absent: no)
                    occupiable: b.can_be_occupied.unwrap_or(false),
                    occupants_fire: b.can_occupy_fire.unwrap_or(false),
                    // spec: S-0405 Terms (an upgrade type: S-0406's, the type names the building it is added to)
                    upgrade: e.powers_up_building.is_some(),
                }
            })
            .collect();
        let powered_unit = vehicles
            .iter()
            .map(|n| {
                r.sim
                    .vehicles
                    .get(n)
                    // spec: S-0405 Import mapping (PoweredUnit absent: no)
                    .and_then(|v| v.econ.powered_unit)
                    .unwrap_or(false)
            })
            .collect();
        Self {
            buildings,
            powered_unit,
            // spec: S-0405 Constants (spy blackout length absent: 0)
            spy_blackout: r.sim.general.economy.spy_power_blackout.unwrap_or(0),
            // spec: S-0510 Import mapping and Constants (SpyMoneyStealPercent: a percentage field of S-0003 claim 7 kept in single precision, a % product narrowed under the rounding mode of S-0205; absent 0.5; a later layer that omits it keeps the earlier value)
            spy_steal: r
                .sim
                .general
                .economy
                .spy_money_steal_percent
                .as_ref()
                .map_or_else(
                    || F64::from_i64(1, mode).div(F64::from_i64(2, mode), mode),
                    |d| d.single_value(mode).to_f32(mode).to_f64(),
                ),
        }
    }
}

/// A house's power totals (S-0405 claim 4) and blackout (claims 5, 6).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HousePower {
    /// The house output *O* as last worked out (claim 4).
    pub output: i32,
    /// The house drain *D* as last worked out (claim 4).
    pub drain: i32,
    /// An event since the last working-out has made the house due (claim 7).
    pub due: bool,
    /// A running timed blackout: the tick at whose house update it ends
    /// (claim 6).
    pub blackout_end: Option<i32>,
    /// A drain blackout at the last working-out (claim 8, the no-blackout row).
    pub drain_blackout: bool,
    /// The tick of the house's last update (to tell whether an event of
    /// this tick came before or after it).
    pub updated: i32,
    /// The count of active control centres (claim 13).
    pub centres: i32,
    /// With `quirks.shared_control_centre_count` off: the count per powered
    /// vehicle type.
    pub centres_by_unit: BTreeMap<u32, i32>,
}

impl Default for HousePower {
    fn default() -> Self {
        Self {
            output: 0,
            drain: 0,
            due: false,
            blackout_end: None,
            drain_blackout: false,
            updated: -1, // nocite: before the first tick, no house update has run
            centres: 0,
            centres_by_unit: BTreeMap::new(),
        }
    }
}

/// A building's switches: the power switch (S-0405 Terms, claim 10), the
/// primary-factory mark (S-0213 claim 13) and its upgrades (S-0406 claim 12);
/// and the power topic's per-object state.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildingSwitches {
    /// Switched off by the power switch (S-0405 Terms).
    pub switched_off: bool,
    /// The primary factory of its kind (S-0213 claim 13).
    pub primary: bool,
    /// The building's three upgrade positions, 1 to 3 at indices 0 to 2
    /// (S-0405 Terms, S-0406 F5, S-0408 claim 5): the upgrade type recorded
    /// in each, or none.
    pub upgrades: [Option<u32>; 3], // spec: S-0405 Constants (three upgrade positions), S-0406 Terms (upgrade positions 1 to 3)
    /// The building's upgrade count: the room it has used (S-0406 claim 12,
    /// F5).
    #[serde(default)]
    pub upgrade_count: i32,
    /// Repair switched on (S-0213 claim 13, S-0314).
    pub repairing: bool,
    /// The health the building last noticed in its own turn (claim 7).
    pub noticed_health: i32,
    /// The in-operation result at the building's previous turn (claim 9);
    /// `None` before its first turn on the map (or under a new owner).
    pub operating: Option<bool>,
    /// An active control centre (Terms, claim 13).
    pub centre: bool,
    /// The super-gap is active (claim 16).
    pub super_gap: bool,
    /// Its sale was seen starting (S-0410 claim 2).
    pub selling_seen: bool,
    /// Manned (glossary; S-0515 claim 2, S-0405 Terms): set at the
    /// building's first ownership change of any cause and never cleared.
    /// A building put on the map, by any means, is not manned.
    pub manned: bool,
    /// The tick in which the building's income countdown was last restarted
    /// (S-0515 claim 4, Terms); `None` before the building was first put on
    /// the map, a countdown that has run out (the length itself is the
    /// type's income delay).
    #[serde(default)]
    pub income_restart: Option<i32>,
    /// A vehicle disabled by the loss of its control centre (claim 13).
    pub unit_disabled: bool,
}

// ---------------------------------------------------------------------------
// Seams: states other topics own (each answers "no" or 0 until it exists).

/// Under a pulse (S-0405 Terms; the combat and superweapon specs start and
/// end it). Seam: no pulse exists yet.
pub fn under_pulse(w: &World, id: ObjectId) -> bool {
    let _ = (w, id);
    false
}

/// Being erased (glossary): a superweapon warp member in its leaving stage
/// (S-0513 claim 39) or an object a temporal beam is attached to (S-0514),
/// both read by [`crate::superweapons::being_erased`].
pub fn being_erased(w: &World, id: ObjectId) -> bool {
    crate::superweapons::being_erased(w, id)
}

/// Being drained (S-0405 Terms): attached to a drainer by a drain link
/// (S-0608, `crate::drain`).
pub fn being_drained(w: &World, id: ObjectId) -> bool {
    // spec: S-0405 Terms and S-0608 Terms (being drained: the building of a drain link)
    crate::drain::being_drained(w, id)
}

/// *n*<sub>c</sub>, the chargers a building has (S-0405 Terms): the
/// registered chargers of S-0518 claim 12.
pub fn chargers(w: &World, id: ObjectId) -> i32 {
    crate::linked::chargers(w, id)
}

/// *n*<sub>a</sub>, the passengers an absorbing building holds (S-0312
/// Terms, S-0605 Terms).
pub fn absorbed(w: &World, id: ObjectId) -> i32 {
    // spec: S-0605 Terms and claim 9 (the passenger count n is S-0405's n_a)
    crate::structures::absorb::count(w, id)
}

/// Whether a building is manned (glossary; S-0515 claim 2): it has had
/// at least one ownership change, whatever its cause. Not physical
/// engineer presence (S-0405 Terms).
pub fn manned(w: &World, id: ObjectId) -> bool {
    w.obj(id).is_some_and(|o| o.econ.building.manned)
}

/// Occupants who are inside an occupiable building (S-0215, S-0508).
pub fn occupants(w: &World, id: ObjectId) -> i32 {
    crate::structures::garrison::count(w, id)
}

// ---------------------------------------------------------------------------

fn bp(w: &World, type_index: u32) -> Option<&BuildingPower> {
    w.rules().econ.power.buildings.get(type_index as usize)
}

/// The upgrades a building holds that count: those in its three upgrade
/// positions (Terms, *upgrade positions*).
fn held_upgrades(w: &World, id: ObjectId) -> Vec<u32> {
    w.obj(id).map_or_else(Vec::new, |o| {
        // spec: S-0405 claims 1 and 2, F1 and F2 (recorded upgrades count only while the upgrade count is positive)
        if o.econ.building.upgrade_count == 0 {
            return Vec::new();
        }
        // spec: S-0405 Constants (3 upgrade positions count in power); S-0405 claims 1 and 2 (with a positive count every recorded position counts, those above the count included)
        o.econ.building.held().collect()
    })
}

/// A building's output (claim 1, F1) with its health as it is now.
pub fn building_output(w: &World, id: ObjectId) -> i32 {
    let Some(o) = w.obj(id) else { return 0 };
    let Some(t) = bp(w, o.type_index) else {
        return 0;
    };
    // spec: S-0405 claim 1 (0 when switched off or being erased)
    if o.kind != Kind::Building || o.econ.building.switched_off || being_erased(w, id) {
        return 0;
    }
    // spec: S-0405 F1 (P = base output + [absorbing and extra output > 0] × extra output × n_a + Σ base output of the held upgrades)
    let mut p = t.output;
    if t.absorbing && t.extra_output > 0 {
        p += t.extra_output * absorbed(w, id);
    }
    for u in held_upgrades(w, id) {
        p += bp(w, u).map_or(0, |b| b.output);
    }
    if p == 0 {
        return 0;
    }
    let mode = w.rules().mode;
    let s = crate::combat::damage::strength(w, id);
    if s <= 0 {
        return 0;
    }
    // spec: S-0405 F1 (output = trunc(RZ₅₃(RZ₅₃(H / S) × P)))
    F64::from_i64(i64::from(o.health), mode)
        .div(F64::from_i64(i64::from(s), mode), mode)
        .mul(F64::from_i64(i64::from(p), mode), mode)
        .to_i32_common()
}

/// A building's drain (claim 2, F2).
pub fn building_drain(w: &World, id: ObjectId) -> i32 {
    let Some(o) = w.obj(id) else { return 0 };
    let Some(t) = bp(w, o.type_index) else {
        return 0;
    };
    // spec: S-0405 claim 2 (0 when switched off or being erased)
    if o.kind != Kind::Building || o.econ.building.switched_off || being_erased(w, id) {
        return 0;
    }
    // spec: S-0405 F2 (base drain + Σ base drain of the held upgrades + [super-gap active] × extra drain)
    let mut d = t.drain;
    for u in held_upgrades(w, id) {
        d += bp(w, u).map_or(0, |b| b.drain);
    }
    if o.econ.building.super_gap {
        d += t.extra_drain;
    }
    d
}

/// The counted buildings of a house (claim 3): on the map, whatever their
/// state. Upgrades count through the building that holds them (Terms).
fn counted(w: &World, h: HouseId) -> Vec<ObjectId> {
    w.objects
        .values()
        // spec: S-0405 claim 3 (every building of the house that is on the map)
        .filter(|o| o.kind == Kind::Building && o.owner == Some(h) && o.on_map)
        .filter(|o| bp(w, o.type_index).is_none_or(|b| !b.upgrade))
        .map(|o| o.id)
        .collect()
}

/// Makes a house's power due (claim 7); its radar is due too (S-0410
/// claim 2, a power balance change follows).
pub fn make_due(w: &mut World, h: HouseId) {
    if let Some(house) = w.houses.get_mut(usize::from(h.0)) {
        house.econ.power.due = true;
    }
}

/// Whether a timed blackout of the house is running at this moment
/// (claim 6, the live reading of claim 8).
pub fn blackout_running(w: &World, h: HouseId) -> bool {
    w.houses
        .get(usize::from(h.0))
        .and_then(|x| x.econ.power.blackout_end)
        // spec: S-0405 claim 6 (running until the house update of its end tick; a late start of length 1 runs to the end of t₀)
        .is_some_and(|end| w.tick <= end)
}

/// Starts a timed blackout of length `d` ticks for a house (claim 6): a
/// spy's entry only when S-0510's single-effect selection picks the
/// blackout effect (S-0510 claims 1 and 3, row E2; *d* the game-wide spy
/// blackout length, [`Rules::spy_blackout`]), or a force-shield launch
/// (S-0513 claim 30).
pub fn start_blackout(w: &mut World, h: HouseId, d: i32) {
    let t0 = w.tick;
    let one_short = w.rules().quirks.blackout_one_short;
    let Some(house) = w.houses.get_mut(usize::from(h.0)) else {
        return;
    };
    let p = &mut house.econ.power;
    // spec: S-0405 claim 6 (starting a blackout, whatever its length, makes the house due)
    p.due = true;
    // spec: S-0405 claim 6 (a length of 0 or less runs no blackout and ends a running one at once; a new start replaces it)
    if d <= 0 {
        p.blackout_end = None;
        return;
    }
    let after = p.updated == t0;
    p.blackout_end = Some(if one_short {
        // spec: S-0405 claim 6 (it ends at the beginning of the house update of tick t₀ + d − 1, quirks.blackout_one_short)
        t0 + d - 1
    } else {
        // spec: S-0405 Edge cases (switch off: exactly d house updates from the first one after the start)
        let first = if after { t0 + 1 } else { t0 };
        first + d
    });
}

/// Whether the house has low power (claim 4): *D* > 0 and *O* < *D* as last
/// worked out.
pub fn low_power(w: &World, house: HouseId) -> bool {
    w.houses.get(usize::from(house.0)).is_some_and(|h| {
        let p = &h.econ.power;
        // spec: S-0405 Terms (low power: D > 0 and O < D)
        p.drain > 0 && p.output < p.drain
    })
}

/// The house's power fraction *p* (claim 4, F3), a double from 0 to 1, as
/// S-0403 claim 5 reads it.
pub fn fraction(w: &World, house: HouseId) -> F64 {
    let mode = w.rules().mode;
    let (o, d) = w
        .houses
        .get(usize::from(house.0))
        .map_or((0, 0), |h| (h.econ.power.output, h.econ.power.drain));
    // spec: S-0405 F3 (p = 1 when D = 0 or O ≥ D; 0 when O = 0 < D; otherwise RZ₅₃(O / D))
    if d == 0 || o >= d {
        F64::from_i64(1, mode)
    } else if o <= 0 {
        F64::ZERO
    } else {
        F64::from_i64(i64::from(o), mode).div(F64::from_i64(i64::from(d), mode), mode)
    }
}

/// The house's totals (*O*, *D*) as last worked out.
pub fn totals(w: &World, house: HouseId) -> (i32, i32) {
    w.houses
        .get(usize::from(house.0))
        .map_or((0, 0), |h| (h.econ.power.output, h.econ.power.drain))
}

/// The meter's scale *B* (claim 19, F4): the sum of base output and base
/// drain of the house's buildings, without health scaling, upgrades,
/// passengers, switching or blackouts.
pub fn meter_base(w: &World, house: HouseId) -> i32 {
    w.objects
        .values()
        .filter(|o| o.kind == Kind::Building && o.owner == Some(house) && o.on_map && o.health > 0)
        .filter_map(|o| bp(w, o.type_index))
        // spec: S-0405 claim 19 (B: base output plus base drain, nothing else)
        .fold(0i32, |a, b| a.wrapping_add(b.output).wrapping_add(b.drain))
        .max(0)
}

/// The settled power meter (claim 19, F4) for a display capacity of `n`
/// bands (*n* > 1), scale `b` and totals `o` and `d`: the occupied capacity
/// *A* and the supplied, near-shortage and further-shortage extents in
/// bands, as doubles before any quantisation.
// spec: S-0405 F4 (three extents: supplied, near shortage, further shortage)
pub fn meter(b: i32, o: i32, d: i32, n: i32, mode: Rounding) -> (i32, [F64; 3]) {
    let f = |x: i64| F64::from_i64(x, mode);
    // spec: S-0405 F4 (q = trunc(RZ₅₃(RZ₅₃(400 / RZ₅₃(B + 400)) × N)); Constants: meter scale constant 400)
    let q = f(400)
        .div(f(i64::from(b)).add(f(400), mode), mode)
        .mul(f(i64::from(n)), mode)
        .to_i32_common();
    // spec: S-0405 F4 (A = N − min(N − 1, max(0, q)), integer clamps)
    let a = n - (n - 1).min(q.max(0));
    // spec: S-0405 F4 (U = max(0, D − O), V = min(U, 100), W = U − V, T = max(O, D)); Constants: near-shortage interval 100
    let u = (i64::from(d) - i64::from(o)).max(0);
    let v = u.min(100); // spec: S-0405 Constants (near-shortage interval 100)
    let wv = u - v;
    let t = i64::from(o.max(d));
    if t <= 0 {
        // spec: S-0405 F4 (T = 0: the supplied extent is A, both shortage extents 0)
        return (a, [f(i64::from(a)), F64::ZERO, F64::ZERO]);
    }
    let ext = |x: i64| f(i64::from(a)).mul(f(x).div(f(t), mode), mode);
    // spec: S-0405 F4 (supplied RZ₅₃(A × RZ₅₃(O / T)), near RZ₅₃(A × RZ₅₃(V / T)), further RZ₅₃(A × RZ₅₃(W / T)))
    (a, [ext(i64::from(o.max(0))), ext(v), ext(wv)])
}

/// The power part of a house's update (claims 4 to 7 and 11): a running
/// timed blackout that ends here ends first; then a due house works out its
/// power. Returns whether the totals changed (S-0410 claim 2).
pub fn house_update(w: &mut World, h: HouseId) -> bool {
    let t = w.tick;
    let hi = usize::from(h.0);
    let Some(house) = w.houses.get_mut(hi) else {
        return false;
    };
    let p = &mut house.econ.power;
    p.updated = t;
    if let Some(end) = p.blackout_end
        && t >= end
    {
        // spec: S-0405 claim 6 (the blackout ends at the beginning of the house update, and the house works out its power again)
        p.blackout_end = None;
        p.due = true;
    }
    // spec: S-0405 claim 7 (only when an event since its last working-out has made it due)
    if !p.due {
        return false;
    }
    p.due = false;
    let timed = p.blackout_end.is_some_and(|end| t < end);
    let before = (p.output, p.drain);
    let ids = counted(w, h);
    let mut o = 0i32;
    let mut d = 0i32;
    let mut drained = false;
    for &id in &ids {
        // spec: S-0405 claim 7 and F1 (each building's output uses its health at this moment)
        let out = building_output(w, id);
        o = o.saturating_add(out);
        d = d.saturating_add(building_drain(w, id));
        // spec: S-0405 claim 5 (a counted building whose output is above 0 and that is being drained)
        if out > 0 && being_drained(w, id) {
            drained = true;
        }
    }
    // spec: S-0405 claim 5 (during a blackout the output O is 0; the drain is unchanged)
    if timed || drained {
        o = 0;
    }
    let p = &mut w.houses[hi].econ.power;
    p.output = o;
    p.drain = d;
    p.drain_blackout = drained;
    let changed = before != (o, d);
    // spec: S-0405 claim 11 (the step period of each production line is worked out again at once, S-0403 claim 8)
    crate::econ::production::power_changed(w, h);
    // S-0405 claim 15 (the charge hold of superweapons that depend on power) is a seam: superweapons are not written yet.
    changed
}

/// The in-operation test (claim 8), read live.
pub fn in_operation(w: &World, id: ObjectId) -> bool {
    let Some(o) = w.obj(id) else { return false };
    if o.kind != Kind::Building {
        return false;
    }
    let Some(t) = bp(w, o.type_index) else {
        return false;
    };
    let nc = chargers(w, id);
    // spec: S-0405 Constants (2 chargers keep a building in operation)
    let charged = nc >= 2;
    // spec: S-0405 claim 8 (switched on, or charged)
    if o.econ.building.switched_off && !charged {
        return false;
    }
    // spec: S-0405 claim 8 (not under a pulse)
    if under_pulse(w, id) {
        return false;
    }
    // spec: S-0405 claim 8 (alive)
    if o.health <= 0 {
        return false;
    }
    let house = o.owner;
    // spec: S-0405 claim 8 (powered: needs-power flag, base drain above 0, low power, fewer than 2 chargers)
    if t.needs_power && t.drain > 0 && !charged && house.is_some_and(|h| low_power(w, h)) {
        return false;
    }
    // spec: S-0405 claim 8 (no blackout: special-power flag and a timed blackout running, or a drain blackout at the last working-out)
    if t.special_power
        && house.is_some_and(|h| {
            blackout_running(w, h)
                || w.houses
                    .get(usize::from(h.0))
                    .is_some_and(|x| x.econ.power.drain_blackout)
        })
    {
        return false;
    }
    // spec: S-0405 claim 8 (manned when required: the needs-engineer flag and not manned, S-0515 claim 2)
    if t.needs_engineer && !manned(w, id) {
        return false;
    }
    // spec: S-0405 claim 8 (not building or selling: the current behaviour only) and S-0406 claim 17 (the queued construction is seen while none is current)
    !matches!(
        super::placement::seen_behaviour(o),
        Some(Behaviour::Construction | Behaviour::Selling)
    )
}

/// A building's own answer in the firing check (claim 9, table T1), at
/// S-0301 claim 9 rank 4; `None` when neither row applies.
pub fn firing_answer(w: &World, id: ObjectId) -> Option<Answer> {
    let o = w.obj(id)?;
    if o.kind != Kind::Building {
        return None;
    }
    let t = bp(w, o.type_index)?;
    // spec: S-0406 claim 17 (the queued construction is seen while none is current)
    let building_or_selling = matches!(
        super::placement::seen_behaviour(o),
        Some(Behaviour::Construction | Behaviour::Selling)
    );
    // spec: S-0405 T1 (illegal: occupiable without firing occupants; being drained; building or selling and not a pulse cannon)
    if (t.occupiable && (occupants(w, id) == 0 || !t.occupants_fire))
        || being_drained(w, id)
        || (building_or_selling && !t.pulse_cannon)
    {
        return Some(Answer::Illegal);
    }
    // spec: S-0405 T1 (impossible: a pulse cannon, or not in operation)
    if t.pulse_cannon || !in_operation(w, id) {
        return Some(Answer::Impossible);
    }
    None
}

impl BuildingSwitches {
    /// The upgrade types recorded in the building's positions, position 1
    /// first (S-0405 Terms).
    pub fn held(&self) -> impl Iterator<Item = u32> + '_ {
        self.upgrades.iter().flatten().copied()
    }
}

/// Whether a map's switched-on field of 0 starts a pre-placed building of
/// type `type_index` switched off (S-0405 claim 18).
pub fn map_off_applies(w: &World, type_index: u32) -> bool {
    // spec: S-0405 claim 18 (only with a base drain above 0 or the needs-power flag; the firestorm-wall and power-mode flags do not decide it)
    bp(w, type_index).is_some_and(|t| t.drain > 0 || t.needs_power)
}

/// Applies a power-off or power-on order (S-0213 claim 13, S-0405 claim 10).
pub fn switch(w: &mut World, actor: ObjectId, on: bool) {
    let Some(o) = w.obj(actor) else { return };
    // spec: S-0405 claim 10 and S-0213 claim 13 (a building that is live and on the map)
    if o.kind != Kind::Building || !o.on_map || o.health <= 0 {
        return;
    }
    let Some(t) = bp(w, o.type_index).cloned() else {
        return;
    };
    // spec: S-0213 claim 13 (nothing for a building whose type carries the firestorm-wall flag)
    if t.firestorm {
        return;
    }
    let off = o.econ.building.switched_off;
    if on {
        // spec: S-0405 claim 10 (power on only when not under a pulse)
        if !off || under_pulse(w, actor) {
            return;
        }
    } else {
        // spec: S-0405 claim 10 (power off only with a base drain above 0 or the needs-power flag)
        if off || !(t.drain > 0 || t.needs_power) {
            return;
        }
    }
    if let Some(ob) = w.obj_mut(actor) {
        ob.econ.building.switched_off = !on;
    }
    // spec: S-0405 claim 10 (an accepted switch-off ends the gap field immediately; switching on lets it start again in the next building turn in operation, claim 9)
    // The lights and working animations of claim 10 are presentation, derived from `switched_off` and the in-operation result; no simulation state.
    if !on {
        crate::econ::shroud::gap_switched_off(w, actor);
        // spec: S-0516 claim 20 (an accepted manual power-off makes a cloak generator's shrinking pending at once, also at radius 0)
        crate::cloak::field::switched_off(w, actor);
    }
    crate::econ::events::switched(w, actor);
}

/// The power topic's reaction to a building put on the map (claims 3 and 7).
pub fn entered_map(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id) else { return };
    if o.kind != Kind::Building {
        return;
    }
    let (owner, health) = (o.owner, o.health);
    if let Some(ob) = w.obj_mut(id) {
        ob.econ.building.noticed_health = health;
        ob.econ.building.operating = None;
    }
    // spec: S-0405 claim 7 (a building placed or created on the map makes its house due)
    if let Some(h) = owner {
        make_due(w, h);
    }
}

/// The power topic's reaction to a building leaving the map (claims 7 and
/// 13): its house is due, and an active control centre is lost.
pub fn leaving_map(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id) else { return };
    if o.kind != Kind::Building {
        return;
    }
    let owner = o.owner;
    lose_centre(w, id, owner);
    // spec: S-0405 claim 7 (a building removed or destroyed makes its house due)
    if let Some(h) = owner {
        make_due(w, h);
    }
}

/// The power topic's reaction to a change of owner (claims 7 and 13, and
/// the manned state of S-0515 claim 2).
pub fn owner_changed(w: &mut World, id: ObjectId, old: Option<HouseId>) {
    let Some(o) = w.obj(id) else { return };
    if o.kind != Kind::Building {
        return;
    }
    let new = o.owner;
    lose_centre(w, id, old);
    if let Some(ob) = w.obj_mut(id) {
        // Under its new owner the building comes into operation afresh in its next turn (Terms, *active control centre*).
        ob.econ.building.operating = None;
        // spec: S-0515 claim 2 and S-0405 Terms (manned from the first ownership change of any cause, whatever the old and new owners; never cleared; glossary, *building ownership change*: a change of the owning house)
        if old != new {
            ob.econ.building.manned = true;
        }
    }
    // spec: S-0405 claim 7 (a building that changes owner makes the house due)
    for h in [old, new].into_iter().flatten() {
        make_due(w, h);
    }
}

/// A building's own turn (claims 7, 9, 13 and 16): it notices a health
/// change, and the one-off effects follow its in-operation result.
pub fn building_turn(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id) else { return };
    if o.kind != Kind::Building || !o.on_map {
        return;
    }
    let owner = o.owner;
    let (health, noticed, prev) = (
        o.health,
        o.econ.building.noticed_health,
        o.econ.building.operating,
    );
    if health != noticed {
        if let Some(ob) = w.obj_mut(id) {
            ob.econ.building.noticed_health = health;
        }
        // spec: S-0405 claim 7 (the building notices a health change in its own turn; the house works out its power in its update of the same tick)
        if let Some(h) = owner {
            make_due(w, h);
        }
    }
    let selling = w.obj(id).is_some_and(|o| {
        o.beh.current == Some(Behaviour::Selling) || o.beh.queued == Some(Behaviour::Selling)
    });
    if selling && !w.obj(id).is_some_and(|o| o.econ.building.selling_seen) {
        if let Some(ob) = w.obj_mut(id) {
            ob.econ.building.selling_seen = true;
        }
        // spec: S-0410 claim 2 (a house that starts selling a building is rechecked)
        if let Some(h) = owner {
            crate::econ::radar::mark_due(w, h);
        }
    }
    let op = in_operation(w, id);
    // spec: S-0405 claim 9 and S-0601 claim 10 (a controller building not in operation releases its victims in its own turn; reconciled on every turn)
    crate::mind::building_turn(w, id, op);
    // spec: S-0405 claim 9 and S-0607 claim 12 (row 4: a building lifter not in operation lets go of its held unit in its own turn)
    crate::lift::building_turn(w, id, op);
    if prev == Some(op) {
        // spec: S-0405 claim 9 (the effects are reconciled on a turn even without an in-operation transition: a charged gap building switched off can generate again)
        crate::econ::shroud::gap_refresh(w, id);
        return;
    }
    if let Some(ob) = w.obj_mut(id) {
        ob.econ.building.operating = Some(op);
    }
    // spec: S-0405 claim 9 (the one-off effects happen in the first turn whose in-operation result differs from the previous turn's)
    if op {
        gain_centre(w, id);
    } else {
        lose_centre(w, id, owner);
        // spec: S-0405 claim 16 (the building ceasing to be in operation ends the super-gap and removes the extra drain)
        if w.obj(id).is_some_and(|o| o.econ.building.super_gap) {
            if let Some(ob) = w.obj_mut(id) {
                ob.econ.building.super_gap = false;
            }
            if let Some(h) = owner {
                make_due(w, h);
            }
        }
        // S-0405 claim 9: the release of controlled units is `mind::building_turn` above; a held unit's, `lift::building_turn`.
    }
    // spec: S-0405 claim 9 and S-0409 claim 11 (the gap starts and stops with the building's operation)
    crate::econ::shroud::gap_refresh(w, id);
}

/// The count key of a control centre: its powered unit, if its type is a
/// control-centre type.
fn centre_unit(w: &World, id: ObjectId) -> Option<u32> {
    let o = w.obj(id)?;
    if o.kind != Kind::Building {
        return None;
    }
    bp(w, o.type_index)?.powers_unit
}

/// A control centre comes into operation (claim 13).
fn gain_centre(w: &mut World, id: ObjectId) {
    let Some(unit) = centre_unit(w, id) else {
        return;
    };
    let Some(h) = w.obj(id).and_then(|o| o.owner) else {
        return;
    };
    if w.obj(id).is_some_and(|o| o.econ.building.centre) {
        return;
    }
    if let Some(ob) = w.obj_mut(id) {
        ob.econ.building.centre = true;
    }
    let shared = w.rules().quirks.shared_control_centre_count;
    let p = &mut w.houses[usize::from(h.0)].econ.power;
    // spec: S-0405 claim 13 (one count for all control-centre types, quirks.shared_control_centre_count; off: per powered-unit type)
    let n = if shared {
        p.centres += 1;
        p.centres
    } else {
        let c = p.centres_by_unit.entry(unit).or_insert(0);
        *c += 1;
        *c
    };
    // spec: S-0405 claim 13 (count from 0 to 1: the vehicles of the powered unit of the building that came into operation are enabled)
    if n == 1 {
        set_disabled(w, h, unit, false);
    }
}

/// A control centre stops being active (claim 13): it left operation or
/// was lost.
fn lose_centre(w: &mut World, id: ObjectId, owner: Option<HouseId>) {
    let Some(unit) = centre_unit(w, id) else {
        return;
    };
    if !w.obj(id).is_some_and(|o| o.econ.building.centre) {
        return;
    }
    if let Some(ob) = w.obj_mut(id) {
        ob.econ.building.centre = false;
    }
    let Some(h) = owner else { return };
    let shared = w.rules().quirks.shared_control_centre_count;
    let p = &mut w.houses[usize::from(h.0)].econ.power;
    let n = if shared {
        p.centres -= 1;
        p.centres
    } else {
        let c = p.centres_by_unit.entry(unit).or_insert(0);
        *c -= 1;
        *c
    };
    // spec: S-0405 claim 13 (count from 1 to 0: the vehicles of the powered unit of the building type that just stopped, not off the map, are disabled)
    if n == 0 {
        set_disabled(w, h, unit, true);
    }
}

/// Disables or enables every vehicle of a house of one type (claim 13) and
/// gives each the one-off effects of claim 20. Vehicles off the map are
/// passed over at both transitions; at disabling, so is a vehicle tethered
/// to the building standing in its own cell.
fn set_disabled(w: &mut World, h: HouseId, unit: u32, disabled: bool) {
    let ids: Vec<ObjectId> = w
        .objects
        .values()
        .filter(|o| {
            // spec: S-0405 claim 13 (revision 20, Q-1503 answered: a vehicle off the map at either transition is passed over and keeps its state)
            o.kind == Kind::Vehicle && o.owner == Some(h) && o.type_index == unit && o.on_map
        })
        .map(|o| o.id)
        .collect();
    for id in ids {
        if disabled && tethered_in_own_cell(w, id) {
            // spec: S-0405 claim 13 (at the 1-to-0 transition a vehicle tethered to the building standing in its own cell is passed over and stays enabled, for example a new vehicle still in its factory; revision 21, Q-1505: on its own house's repair depot only when tethered to that depot and the depot stands in its cell, reading neither the pad flag nor the repair phase; S-0314 claim 17 owns when depot entry tethers)
            continue;
        }
        if disabled {
            disable(w, id);
        } else {
            enable(w, id);
        }
    }
}

/// Whether a unit is tethered to a building that stands on its own cell
/// (glossary *tether link*; S-0214 claim 19, S-0405 claim 13).
pub fn tethered_in_own_cell(w: &World, id: ObjectId) -> bool {
    let Some(o) = w.obj(id) else { return false };
    super::tethered(w, id)
        && o.contact.and_then(|k| w.obj(k)).is_some_and(|b| {
            b.kind == Kind::Building
                && crate::objects::foundation_cells(b.pos.cell(), b.foundation)
                    .contains(&o.pos.cell())
        })
}

/// Disables a vehicle (claims 13 and 20): the guard reset, deselection and
/// the locomotor switched off, also for one that is already disabled; the
/// deactivation sound only for one that was enabled.
fn disable(w: &mut World, id: ObjectId) {
    let was = unit_disabled(w, id);
    set_flag(w, id, true);
    // spec: S-0405 claim 20 (revision 20, Q-1503 answered: at disabling, the guard reset of S-0215 claim 9, removal from the selection, the locomotor switched off, in that order; the 1-to-0 transition does this also to a vehicle already disabled)
    guard_reset(w, id);
    // spec: S-0405 claim 20 and S-0503 claim 10 (the locomotor switched off after the guard reset, whose cleared destination can first have switched a settled vehicle on, S-0314 claim 21; powered is judged then)
    crate::hover::switched(w, id, true);
    // spec: S-0314 Terms and claim 21 (the disabling's switch-off makes the vehicle settled, whether or not it was settled or switched on before)
    crate::depot::disabled(w, id);
    w.effects
        .push(crate::combat::effects::Effect::Deactivated { unit: id });
    if !was {
        // spec: S-0405 claim 20 (a vehicle that becomes disabled plays its type's deactivation sound at its position; one already disabled plays nothing)
        activation_sound(w, id, crate::combat::effects::Sound::Deactivate);
    }
}

/// Enables a vehicle (claims 13 and 20): the guard reset and the
/// locomotor switched on, whether or not it was disabled; the activation
/// sound only for one that was disabled.
fn enable(w: &mut World, id: ObjectId) {
    let was = unit_disabled(w, id);
    set_flag(w, id, false);
    // spec: S-0405 claim 20 (at enabling, each vehicle claim 13 enables gets the guard reset and its locomotor switched on again, whether or not it was disabled)
    guard_reset(w, id);
    crate::hover::switched(w, id, false);
    // spec: S-0314 Terms and claim 21 (an enabling ends a vehicle's being settled)
    crate::depot::enabled(w, id);
    if was {
        // spec: S-0405 claim 20 (a disabled vehicle that becomes enabled plays its type's activation sound)
        activation_sound(w, id, crate::combat::effects::Sound::Activate);
    }
}

/// The deactivation or activation sound of a vehicle at its position
/// (claim 20, Import mapping; playback is S-0701's).
fn activation_sound(w: &mut World, id: ObjectId, sound: crate::combat::effects::Sound) {
    let Some(o) = w.obj(id) else { return };
    let (of, pos) = (crate::econ::tech::type_of(o), o.pos);
    w.effects.push(crate::combat::effects::Effect::Sound {
        sound,
        of: Some(of),
        house: None,
        pos: Some(pos),
    });
}

/// The guard reset (S-0215 claim 9, Terms): *guard* is forced, and the
/// target, the destination and the guard spot are cleared.
pub(crate) fn guard_reset(w: &mut World, id: ObjectId) {
    let t = w.tick;
    let stale = w.rules().quirks.stale_suspended_revert; // spec: S-0215 Edge cases (quirks.stale_suspended_revert)
    if let Some(ob) = w.obj_mut(id) {
        crate::spawn::clear_own_target(ob);
        ob.combat.burst = 0; // spec: S-0301 claim 4 (a cleared target resets the burst counter)
        ob.guard_spot = None;
        ob.beh.force(crate::rules::Behaviour::Guard, t, stale);
    }
    // spec: S-0107 r46 claim 11, Changes of target (the loss of the target at a guard reset, mind control's included, S-0601 claim 6, ends an infantry unit's attack state and makes its request at once, before the destination's clear)
    crate::infantry::watch_test(w, id);
    super::assign_destination(w, id, None);
}

/// Sets a vehicle's disabled flag.
fn set_flag(w: &mut World, id: ObjectId, disabled: bool) {
    if let Some(o) = w.obj_mut(id) {
        o.econ.building.unit_disabled = disabled;
    }
}

/// Sets a vehicle's disabled flag and switches a hovering vehicle off or
/// on with it (S-0503 claim 10), without claim 20's other effects.
pub fn apply_disabled(w: &mut World, id: ObjectId, disabled: bool) {
    set_flag(w, id, disabled);
    // spec: S-0503 claim 10 (a hovering vehicle is switched off by the disabling of S-0405 claim 13, and switched on again by its re-enabling)
    crate::hover::switched(w, id, disabled);
}

/// Whether a vehicle is disabled by the loss of its control centre
/// (claim 13), the *deactivated* state of S-0616 claim 1; claim 20 gives
/// what it cannot do.
pub fn unit_disabled(w: &World, id: ObjectId) -> bool {
    w.obj(id).is_some_and(|o| o.econ.building.unit_disabled)
}

/// Whether the house has an active control centre for a vehicle type
/// (claim 13; the per-type count with `quirks.shared_control_centre_count`
/// off).
pub(crate) fn has_centre(w: &World, h: HouseId, unit: u32) -> bool {
    let p = &w.houses[usize::from(h.0)].econ.power;
    if w.rules().quirks.shared_control_centre_count {
        p.centres > 0
    } else {
        p.centres_by_unit.get(&unit).is_some_and(|&c| c > 0)
    }
}

/// A completed arrival in a cell (claim 13, the cell-entry rule): a
/// vehicle whose type has the powered-unit flag, completing an arrival in
/// a cell while its house has no active control centre, is disabled,
/// unless the cell holds a building. One already disabled gets the
/// disabling again, with no sound (claim 20). An arrival is the end of a
/// step of its own movement into the cell; placement and creation are not
/// arrivals. Called by the locomotors at their cell-arrival updates
/// (`cloak::machine::cell_movement_completed`).
pub fn arrival(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id) else { return };
    // spec: S-0405 claim 20 (revision 36: a disabled vehicle's arrival is an arrival in claim 13's sense too, so it is not passed over here)
    if o.kind != Kind::Vehicle || !o.on_map {
        return;
    }
    let (cell, owner, ti) = (o.pos.cell(), o.owner, o.type_index);
    let flag = w
        .rules()
        .econ
        .power
        .powered_unit
        .get(ti as usize)
        .copied()
        .unwrap_or(false);
    let Some(h) = owner else { return };
    if !flag || has_centre(w, h, ti) {
        return;
    }
    // spec: S-0405 claim 13 (unless the cell holds a building: a vehicle in a factory or on a repair depot stays enabled)
    let building = w
        .cells
        .occ(cell, Layer::Ground)
        .is_some_and(|r| r.buildings > 0);
    if !building {
        // spec: S-0405 claims 13 and 20 (revision 20, Q-1503 answered: disabled on a completed arrival, not on placement or creation: guard reset, deselected, locomotor off; revision 36: a vehicle already disabled gets the guard reset and locomotor switch-off again, with no sound, and comes down as S-0503 states)
        disable(w, id);
    }
}
