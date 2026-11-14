//! The harvest cycle (S-0402): loading, field and refinery choice, docking,
//! unloading and income.
//!
//! A harvester's processings of *harvest*, *enter* and *unload* are
//! [`process_harvest`], [`process_enter`] and [`process_unload`], which
//! `crate::process` calls. The phases, the step counter (F6), the ring
//! search (claim 4), the refinery choice (claims 8 and 9), unloading (claim
//! 11) and payment (claim 12, F3) are here; the guard rules of claims 13
//! and 17 and the slave miner's tests of claim 14 answer through
//! `crate::seams`.
//!
//! Seams: the building animations of claim 11 (no building animation is
//! modelled, so none is ever playing and none delays the harvester), a
//! second-route arrival report to a building other than a dock-unload
//! refinery (claim 10; the entry-exchange specs own it). A chrono miner
//! with an active teleporting locomotor drops its destination (a remembered
//! cell included) at every seeking processing past the *full* test and
//! searches from where it stands (claim 17, the seeking drop, Q-1130). A
//! chrono miner travels by its active locomotion: teleporting
//! (S-0207 claim 20), or the temporary driving of a factory exit (S-0417
//! claim 16), which drives its first leg to the ore. The slave cycle of claim 14 and the slave groups of
//! claims 19 to 25 are `crate::econ::slaves`; the wake-up rule is
//! [`wake_rule`].

use std::collections::BTreeSet;

use glasnost_map::CellXY;
use glasnost_map::terrain::land;
use glasnost_num::geom::root_floor;
use glasnost_num::{F64, Rounding};
use glasnost_rules::schema::Ruleset;
use serde::{Deserialize, Serialize};

use crate::rules::{Behaviour, Kind, nearest_double};
use crate::world::{HouseId, HouseRole, Layer, ObjectId, Target, World};

use super::resource;

/// A harvester type's fields (S-0402 Import mapping).
#[derive(Clone, Debug, Default)]
pub struct UnitHarvest {
    /// The harvester flag.
    pub harvester: bool,
    /// Capacity *C* (storage).
    pub capacity: i32,
    /// Dock types: building type indices in dock-list order.
    pub docks: Vec<u32>,
    /// The teleporter flag (a chrono miner, claim 17).
    pub teleporter: bool,
    /// A slave miner (claim 14): gatherer, destination and an enslaves field.
    pub slave_miner: bool,
    /// The slave group its objects create (claim 19), if its type names a
    /// slave type.
    pub group: Option<GroupType>,
}

/// A type's slave-group fields (S-0402 claims 14, 19 and 21; Import
/// mapping).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GroupType {
    /// The slave infantry type, by registration index.
    pub slave: u32,
    /// The slave count *N*.
    pub count: i32,
    /// The regeneration delay, ticks.
    pub regen: i32,
    /// The reload delay, ticks.
    pub reload: i32,
}

/// An infantry type's slave fields (S-0402 claim 14; Import mapping).
#[derive(Clone, Copy, Debug, Default)]
pub struct InfantryHarvest {
    /// The slave group its objects create (claim 19; S-0611 claim 14), if
    /// its type names a slave type.
    pub group: Option<GroupType>,
    /// The interval between bales *h*, ticks (`HarvestRate`).
    pub harvest_rate: i32,
    /// The capacity (`Storage`), bales.
    pub capacity: i32,
}

/// A building type's fields (S-0402 Import mapping).
#[derive(Clone, Debug, Default)]
pub struct BuildingHarvest {
    /// The refinery flag.
    pub refinery: bool,
    /// The ore-purifier flag.
    pub purifier: bool,
    /// Dock slots (at least 1).
    pub slots: i32,
    /// The queueing shift (*x*, *y*), cells.
    pub queueing: (i32, i32),
    /// A dock-unload refinery (Terms, claim 10).
    pub dock_unload: bool,
    /// The weed flag (claim 13's idle step-off).
    pub weeder: bool,
    /// The slave group its objects create (claim 19), if its type names a
    /// slave type.
    pub group: Option<GroupType>,
}

/// The harvest rules.
#[derive(Clone, Debug, Default)]
pub struct Rules {
    /// Load step period *L*, ticks.
    pub load_rate: i32,
    /// The dump threshold 900 × *D*, a double (F2).
    pub dump_threshold: F64,
    /// Long and short scan ranges, cells.
    pub long_scan: i32,
    /// Short scan range, cells.
    pub short_scan: i32,
    /// Too-far distance of ordinary harvesters, cells.
    pub too_far: i32,
    /// Too-far distance of chrono miners, cells.
    pub chrono_too_far: i32,
    /// The slave-miner short scan range, cells (claim 14).
    pub slave_short_scan: i32,
    /// The slave-miner kick delay *k*, ticks (claim 14).
    pub kick_delay: i32,
    /// The slave scan range, cells (claim 14).
    pub slave_scan: i32,
    /// The slave-miner long scan range, cells (claim 22).
    pub slave_long_scan: i32,
    /// The scan correction *c*, whole cells (claim 22, F7).
    pub scan_correction: i32,
    /// Infantry types by index.
    pub infantry: Vec<InfantryHarvest>,
    /// Purifier bonus *b*, a single widened to double.
    pub purifier_bonus: F64,
    /// Virtual purifiers per difficulty row (the original's computer player).
    pub virtual_purifiers: Vec<i32>,
    /// Vehicle types by index.
    pub vehicles: Vec<UnitHarvest>,
    /// The slave group of each aircraft type's objects, by index (claim
    /// 19; S-0611 claim 14 and S-0506 scenario R1).
    pub aircraft_groups: Vec<Option<GroupType>>,
    /// Building types by index.
    pub buildings: Vec<BuildingHarvest>,
    /// Income factor *M* per country, a single widened to double.
    pub income: Vec<F64>,
}

/// A lepton-form field in whole cells (S-0402 Terms, *scan range*): the
/// decimal cells × 256 leptons, divided by 256 and truncated toward zero.
fn scan_cells(
    d: &Option<glasnost_rules::sim::Decimal>,
    absent_leptons: i32,
    mode: Rounding,
) -> i32 {
    let leptons = d.as_ref().map_or(absent_leptons, |d| {
        // spec: S-0402 Import mapping (lepton form: decimal cells × 256)
        d.value(mode)
            .mul(F64::from_i64(256, mode), mode)
            .to_i32_common()
    });
    // spec: S-0402 Terms (the field divided by 256, truncated toward zero)
    leptons / 256
}

impl Rules {
    /// The slave group that objects of a type create (claim 19), if the
    /// type names a slave type.
    pub fn group_of(&self, kind: Kind, index: u32) -> Option<GroupType> {
        let i = index as usize;
        match kind {
            Kind::Infantry => self.infantry.get(i)?.group,
            Kind::Vehicle => self.vehicles.get(i)?.group,
            Kind::Aircraft => *self.aircraft_groups.get(i)?,
            Kind::Building => self.buildings.get(i)?.group,
            _ => None,
        }
    }

    /// Every type that names a slave type, by kind and index.
    pub fn masters(&self) -> Vec<(Kind, u32)> {
        let mut v = Vec::new();
        let mut add = |k: Kind, gs: &mut dyn Iterator<Item = Option<GroupType>>| {
            for (i, g) in gs.enumerate() {
                if g.is_some() {
                    v.push((k, i as u32));
                }
            }
        };
        add(Kind::Infantry, &mut self.infantry.iter().map(|x| x.group));
        add(Kind::Vehicle, &mut self.vehicles.iter().map(|x| x.group));
        add(Kind::Aircraft, &mut self.aircraft_groups.iter().copied());
        add(Kind::Building, &mut self.buildings.iter().map(|x| x.group));
        v
    }

    /// The harvest rules of a resolved ruleset.
    pub fn new(r: &Ruleset, mode: Rounding) -> Self {
        let g = &r.sim.general.economy;
        let reg = |k: &str| r.registries.get(k).cloned().unwrap_or_default();
        let buildings_reg = reg("buildings");
        let find_building = |n: &str| {
            // spec: S-0003 claim 14 (names compared with ASCII case folding)
            buildings_reg
                .iter()
                .position(|x| x.eq_ignore_ascii_case(n))
                .map(|i| i as u32)
        };
        let infantry_reg = reg("infantry");
        // spec: S-0402 Import mapping (Enslaves: an infantry type name; SlavesNumber, SlaveRegenRate and SlaveReloadRate absent: 0)
        let group = |e: &glasnost_rules::sim_econ::EconFields| {
            let slave = e.enslaves.as_deref().and_then(|n| {
                // spec: S-0003 claim 14 (names compared with ASCII case folding)
                infantry_reg
                    .iter()
                    .position(|x| x.eq_ignore_ascii_case(n))
                    .map(|i| i as u32)
            })?;
            Some(GroupType {
                slave,
                count: e.slaves_number.unwrap_or(0),
                regen: e.slave_regen_rate.unwrap_or(0),
                reload: e.slave_reload_rate.unwrap_or(0),
            })
        };
        let infantry = infantry_reg
            .iter()
            .map(|n| {
                let u = r.sim.infantry.get(n).cloned().unwrap_or_default();
                InfantryHarvest {
                    // spec: S-0402 claim 19 and S-0611 claim 14 (any created object whose type names a slave type creates its group, a made infantry type included)
                    group: group(&u.econ),
                    // spec: S-0402 Import mapping (HarvestRate absent: 1; Storage absent: 0)
                    harvest_rate: u.econ.harvest_rate.unwrap_or(1),
                    capacity: u.storage.unwrap_or(0),
                }
            })
            .collect();
        let vehicles = reg("vehicles")
            .iter()
            .map(|n| {
                let u = r.sim.vehicles.get(n).cloned().unwrap_or_default();
                let e = &u.econ;
                UnitHarvest {
                    group: group(e),
                    // spec: S-0402 Import mapping (Harvester, Teleporter absent: no; Storage absent: 0; Dock absent: empty)
                    harvester: u.harvester.unwrap_or(false),
                    capacity: u.storage.unwrap_or(0),
                    docks: e
                        .dock
                        .iter()
                        .flatten()
                        .filter_map(|d| find_building(d))
                        .collect(),
                    teleporter: u.teleporter.unwrap_or(false),
                    // spec: S-0402 Terms and Import mapping (ResourceGatherer and ResourceDestination together with an enslaves field: a slave miner)
                    slave_miner: e.resource_gatherer.unwrap_or(false)
                        && e.resource_destination.unwrap_or(false)
                        && e.enslaves.is_some(),
                }
            })
            .collect();
        let aircraft_groups = reg("aircraft")
            .iter()
            .map(|n| {
                // spec: S-0402 claim 19 and S-0611 F5 (an aircraft type, a slot object's type included, that names a slave type creates its group)
                r.sim.aircraft.get(n).and_then(|u| group(&u.econ))
            })
            .collect();
        let buildings = buildings_reg
            .iter()
            .map(|n| {
                let b = r.sim.buildings.get(n).cloned().unwrap_or_default();
                let e = &b.econ;
                BuildingHarvest {
                    group: group(e),
                    // spec: S-0402 Import mapping (Refinery, OrePurifier absent: no)
                    refinery: e.refinery.unwrap_or(false),
                    purifier: e.ore_purifier.unwrap_or(false),
                    // spec: S-0402 Import mapping (NumberOfDocks absent: 1; values below 1 give 1)
                    slots: e.art_number_of_docks.unwrap_or(1).max(1),
                    // spec: S-0402 Import mapping (QueueingCell absent: 0, 0)
                    queueing: e.queueing_cell.map_or((0, 0), |[x, y]| (x, y)),
                    // spec: S-0402 Import mapping (DockUnload, Weeder absent: no)
                    dock_unload: e.dock_unload.unwrap_or(false),
                    weeder: e.weeder.unwrap_or(false),
                }
            })
            .collect();
        let one = F64::from_i64(1, mode);
        let income = reg("countries")
            .iter()
            .map(|n| {
                // spec: S-0402 Import mapping (IncomeMult, a single; absent: 1.0)
                r.sim
                    .countries
                    .get(n)
                    .and_then(|c| c.econ.income_mult.as_ref())
                    .map_or(one, |d| d.single_value(mode))
            })
            .collect();
        // spec: S-0402 Import mapping (HarvesterDumpRate: a single widened to double; absent: the double nearest 0.016)
        let dump = g
            .harvester_dump_rate
            .as_ref()
            .map_or(nearest_double("0.016"), |d| d.single_value(mode));
        Self {
            // spec: S-0402 Import mapping (HarvesterLoadRate absent: 2)
            load_rate: g.harvester_load_rate.unwrap_or(2),
            // spec: S-0402 claim 11 and F2 (900 × D, a double product of the double nearest 900 and D)
            dump_threshold: F64::from_i64(900, mode).mul(dump, mode),
            // spec: S-0402 Import mapping (TiberiumLongScan, TiberiumShortScan absent: 8192 and 1536 leptons)
            long_scan: scan_cells(&g.tiberium_long_scan, 8192, mode),
            short_scan: scan_cells(&g.tiberium_short_scan, 1536, mode), // spec: S-0402 Import mapping (TiberiumShortScan absent: 1536 leptons)
            // spec: S-0402 Import mapping (HarvesterTooFarDistance absent: 5; ChronoHarvTooFarDistance absent: 50)
            too_far: g.harvester_too_far_distance.unwrap_or(5),
            chrono_too_far: g.chrono_harv_too_far_distance.unwrap_or(50), // spec: S-0402 Import mapping (ChronoHarvTooFarDistance absent: 50)
            // spec: S-0402 Import mapping (SlaveMinerShortScan absent: 1280 leptons)
            slave_short_scan: scan_cells(&g.slave_miner_short_scan, 1280, mode),
            // spec: S-0402 Import mapping (SlaveMinerKickFrameDelay absent: 2147483647)
            kick_delay: g.slave_miner_kick_frame_delay.unwrap_or(i32::MAX),
            // spec: S-0402 Import mapping (SlaveMinerSlaveScan absent: 4096 leptons)
            slave_scan: scan_cells(&g.slave_miner_slave_scan, 4096, mode),
            // spec: S-0402 Import mapping (SlaveMinerLongScan absent: 20480 leptons)
            slave_long_scan: scan_cells(&g.slave_miner_long_scan, 20480, mode),
            // spec: S-0402 Import mapping and F7 (SlaveMinerScanCorrection absent: 768 leptons; c = trunc(leptons / 256))
            scan_correction: scan_cells(&g.slave_miner_scan_correction, 768, mode),
            infantry,
            // spec: S-0402 Import mapping (PurifierBonus, a single; absent: the single nearest 0.25)
            purifier_bonus: g.purifier_bonus.as_ref().map_or_else(
                || {
                    glasnost_num::decimal::parse("0.25", glasnost_num::decimal::Field::Single, mode)
                        .unwrap_or_default()
                },
                |d| d.single_value(mode),
            ),
            // spec: S-0402 Import mapping (AIVirtualPurifiers absent: empty)
            virtual_purifiers: g.ai_virtual_purifiers.clone().unwrap_or_default(),
            vehicles,
            aircraft_groups,
            buildings,
            income,
        }
    }
}

/// A harvest phase (S-0402 claim 3).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Phase {
    /// Seeking a field cell.
    #[default]
    Seeking,
    /// Loading bales.
    Loading,
    /// Returning to a refinery.
    Returning,
    /// A refinery is reserved.
    Requesting,
    /// No field was found.
    Idle,
}

/// The step counter and its pacing period (S-0402 claim 5, F6).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Counter {
    /// The tick of the latest processing that set the counter to 0 (*r*).
    pub reset: i32,
    /// Rises counted from `reset` up to the start of the current period.
    pub base: i32,
    /// The current pacing period *p*.
    pub period: i32,
    /// The tick the current period was started (*a*).
    pub start: i32,
}

impl Counter {
    /// The rises of period `p` started at `a` in the ticks `from` … `to`.
    fn rises(p: i32, a: i32, from: i32, to: i32, zero_stalls: bool) -> i32 {
        let from = i64::from(from.max(a));
        let to = i64::from(to);
        if to < from {
            return 0;
        }
        // spec: S-0402 F6 (p = 0: none; quirks.zero_load_period_stalls off: a period of 0 is treated like a negative one)
        if p < 0 || (p == 0 && !zero_stalls) {
            // spec: S-0402 F6 (p < 0: every tick from the start on)
            return (to - from + 1) as i32;
        }
        if p == 0 {
            return 0;
        }
        let (a, p) = (i64::from(a), i64::from(p));
        // spec: S-0402 F6 (p > 0: τ = a + kp, k ≥ 1)
        let k_hi = (to - a).div_euclid(p);
        let k_lo = ((from - a) + p - 1).div_euclid(p).max(1);
        (k_hi - k_lo + 1).max(0) as i32
    }

    /// The counter a processing in tick `t` sees (F6).
    pub fn value(&self, t: i32, zero_stalls: bool) -> i32 {
        // spec: S-0402 F6 (the rise ticks τ with r ≤ τ ≤ t − 1)
        self.base
            + Self::rises(
                self.period,
                self.start,
                self.reset,
                t.wrapping_sub(1),
                zero_stalls,
            )
    }

    /// Sets the counter to 0 in tick `t`.
    fn zero(&mut self, t: i32) {
        self.reset = t;
        self.base = 0;
    }

    /// Starts a pacing period `p` in tick `t`, replacing the schedule of the
    /// one before.
    fn start_period(&mut self, p: i32, t: i32, zero_stalls: bool) {
        self.base += Self::rises(
            self.period,
            self.start,
            self.reset,
            t.wrapping_sub(1),
            zero_stalls,
        );
        self.period = p;
        self.start = t;
    }
}

/// A harvester's load and cycle state (S-0402 claims 1 to 13).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HarvestState {
    /// The load: bales per tiberium, in registration order (claims 1, 15).
    pub load: Vec<i32>,
    /// The harvest phase.
    pub phase: Phase,
    /// The start tick of the *harvest* behaviour the phase belongs to.
    pub harvest_start: Option<i32>,
    /// The remembered cell (claims 6 and 12).
    pub remembered: Option<CellXY>,
    /// The step counter.
    pub counter: Counter,
    /// The period *L* is started at the next processing in *loading*
    /// (claim 5 table, the full or off-field failure).
    pub pending_load_period: bool,
    /// The start tick of the *unload* behaviour the unloading state belongs to.
    pub unload_start: Option<i32>,
    /// Unloading (the start row has been done, claim 11).
    pub unloading: bool,
    /// Finishing (claim 11).
    pub finishing: bool,
    /// Tethered to the refinery it holds contact with (claim 10, glossary
    /// *tether link*); it counts only while that contact lasts.
    #[serde(default)]
    pub tethered: bool,
}

impl HarvestState {
    /// The total of the load (F4).
    pub fn total(&self) -> i32 {
        self.load.iter().sum()
    }
}

/// A refinery's docks: the harvesters holding contact with it (S-0402
/// claims 9 and 10).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DockState {
    /// The harvesters that hold a dock slot, in the order they got it.
    pub holders: Vec<ObjectId>,
}

/// A house's harvest record (S-0402 claim 13): a harvester of the house
/// went idle for lack of a field.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HouseHarvest {
    /// The no-field record; never cleared during the game.
    pub no_field: bool,
}

fn unit(w: &World, id: ObjectId) -> Option<&UnitHarvest> {
    let o = w.obj(id)?;
    if o.kind != Kind::Vehicle {
        return None;
    }
    w.rules().econ.harvest.vehicles.get(o.type_index as usize)
}

/// Whether an object is a harvester (S-0402 Terms).
pub fn is_harvester(w: &World, id: ObjectId) -> bool {
    unit(w, id).is_some_and(|u| u.harvester)
}

fn building(w: &World, id: ObjectId) -> Option<&BuildingHarvest> {
    let o = w.obj(id)?;
    if o.kind != Kind::Building {
        return None;
    }
    w.rules().econ.harvest.buildings.get(o.type_index as usize)
}

/// Whether building `b` permits dock unloading (its type's `DockUnload`
/// flag; S-0207 claim 27, S-0402 Terms).
pub fn permits_dock_unload(w: &World, b: ObjectId) -> bool {
    building(w, b).is_some_and(|x| x.dock_unload)
}

/// The harvest state of a harvester, created on first use.
fn state(w: &mut World, id: ObjectId) -> Option<&mut HarvestState> {
    let n = w.rules().econ.resource.tiberiums.len();
    let o = w.obj_mut(id)?;
    let s = o.econ.harvest.get_or_insert_with(HarvestState::default);
    if s.load.len() < n {
        // spec: S-0402 claim 15 (our engine holds an amount for every tiberium, a departure)
        s.load.resize(n, 0);
    }
    Some(s)
}

/// A copy of a harvester's harvest state (the default when it has none).
pub fn state_of(w: &World, id: ObjectId) -> HarvestState {
    st(w, id)
}

/// A harvester forgets its remembered cell (claim 3), as an owner change
/// makes it (S-0309 claim 31). An object with no harvest state is left
/// without one.
pub(crate) fn forget_remembered(w: &mut World, id: ObjectId) {
    if let Some(s) = w.obj_mut(id).and_then(|o| o.econ.harvest.as_mut()) {
        // spec: S-0309 claim 31 (a harvester forgets its remembered cell, S-0402 claim 3)
        s.remembered = None;
    }
}

fn st(w: &World, id: ObjectId) -> HarvestState {
    w.obj(id)
        .and_then(|o| o.econ.harvest.clone())
        .unwrap_or_default()
}

/// The effective capacity *C* of a vehicle harvester (S-0402 Terms): the
/// imported capacity, except that with
/// `quirks.zero_capacity_harvester_cycles` off an imported 0 counts as 1.
pub fn capacity(w: &World, id: ObjectId) -> i32 {
    let c = unit(w, id).map_or(0, |u| u.capacity);
    // spec: S-0402 Terms and claim 1 (quirks.zero_capacity_harvester_cycles off: imported capacity 0 is effective C = 1 bale)
    if c == 0 && !w.rules().quirks.zero_capacity_harvester_cycles {
        return 1;
    }
    c
}

fn zero_stalls(w: &World) -> bool {
    w.rules().quirks.zero_load_period_stalls
}

/// Whether the harvester is full: the fill ratio total / *C* equals 1.0
/// (claim 16, F4). For *C* = 0 the ratio is not a number and never equals
/// 1.0.
pub fn full(w: &World, id: ObjectId) -> bool {
    let c = capacity(w, id);
    let total = st(w, id).total();
    let m = w.rules().mode;
    // spec: S-0402 F4 (ratio = total / C, a double quotient; full when it equals 1.0)
    F64::from_i64(i64::from(total), m)
        .div(F64::from_i64(i64::from(c), m), m)
        .eq_num(F64::from_i64(1, m))
}

/// The step counter a processing in this tick sees.
pub fn counter(w: &World, id: ObjectId) -> i32 {
    st(w, id).counter.value(w.tick, zero_stalls(w))
}

/// The load of a harvester, bales per tiberium.
pub fn load(w: &World, id: ObjectId) -> Vec<i32> {
    st(w, id).load
}

/// The harvest phase of a harvester.
pub fn phase(w: &World, id: ObjectId) -> Phase {
    st(w, id).phase
}

/// Whether a vehicle harvester's harvesting display is eligible (S-0402
/// claim 5a, start and stop): it is in *harvest* and its harvest is in
/// *loading*. Loading starts in the processing that starts it, before the
/// first bale; it lasts while the harvester is full until the failing load
/// step, and while it drives on to another cell after an exhausted one. A
/// slave miner on claim 22's group route never loads itself. Read-only:
/// presentation asks, the simulation never does.
pub fn display_eligible(w: &World, id: ObjectId) -> bool {
    let Some(o) = w.obj(id) else {
        return false;
    };
    // spec: S-0402 claim 5a and Import mapping (a vehicle harvester: the vehicle type's `Harvester` flag)
    if !is_harvester(w, id) {
        return false;
    }
    // spec: S-0402 claim 5a (leaving harvest ends it by the vehicle's next turn; a queued replacement behaviour does not end loading)
    if o.beh.current != Some(Behaviour::Harvest) {
        return false;
    }
    // spec: S-0402 Terms (a harvester enters harvest in the seeking phase: the phase of an earlier harvest does not count)
    // and claim 5a (eligible from the processing that starts loading until the phase changes)
    o.econ
        .harvest
        .as_ref()
        .is_some_and(|s| s.harvest_start == Some(o.beh.start_tick) && s.phase == Phase::Loading)
}

/// Whether an eligible harvesting display is suppressed now (S-0402 claim
/// 5a, visibility conditions): while the body turns; for a ground driver
/// while it is on a track with a positive integer progress step (a destination
/// alone does not suppress it); for a teleporting harvester while a
/// journey is in progress (a landing point fixed and not yet jumped to);
/// and while phasing in. Being removed by an erasing weapon also
/// suppresses it, but our engine models no erasing weapon yet.
pub fn display_suppressed(w: &World, id: ObjectId) -> bool {
    let Some(o) = w.obj(id) else {
        return true;
    };
    // spec: S-0402 claim 5a (suppressed during body turning)
    if o.body.turning(w.tick) {
        return true;
    }
    // spec: S-0402 claim 5a (suppressed during its teleport disappearance or reappearance state; S-0207 claim 20 f, phasing in)
    if crate::teleport::phasing_in(w, id) {
        return true;
    }
    if crate::teleport::is_teleporter(w, id) {
        // spec: S-0402 claim 5a (a teleporting harvester: a teleport journey still in progress, S-0207 claim 20 a)
        return o.teleport.as_ref().is_some_and(|t| t.landing.is_some());
    }
    // spec: S-0402 claim 5a (a ground driver: an active track with a positive integer per-tick progress
    // step s = trunc(s₂ × ρ) of S-0207 claim 2, not merely ρ > 0; a destination alone does not suppress it).
    // The step is the one the latest drive tick earned (a tick on a track always earns one), not a
    // re-estimate from ρ, which a second update at a track boundary may change (S-0207 claim 12).
    // Erasing (also suppressing) is not modelled: no erasing weapon yet.
    o.drive
        .as_ref()
        .is_some_and(|d| d.track.is_some() && d.step > 0)
}

/// Whether a vehicle harvester shows its harvesting sprite now (S-0402
/// claim 5a): eligible and not suppressed. Ordinary object visibility is
/// the drawing's own.
pub fn display_shown(w: &World, id: ObjectId) -> bool {
    display_eligible(w, id) && !display_suppressed(w, id)
}

fn set_phase(w: &mut World, id: ObjectId, p: Phase) {
    if let Some(s) = state(w, id) {
        s.phase = p;
    }
}

/// Gives a mobile unit a cell to drive to.
fn drive_to(w: &mut World, id: ObjectId, dest: Target) {
    if let Some(o) = w.obj_mut(id) {
        // A new destination starts a new path search (S-0212 claim 20).
        o.path.steps.clear();
        // spec: S-0212 claim 21 and its state table, row "destination changes" (r97: for the driving and walk locomotors a different destination, or none, ends a running retry delay, path::destination_changed and path::waits; the ship and the other locomotors keep it, low confidence; the hovering locomotor's is S-0503's)
    }
    // spec: S-0207 claim 20 (a) (a teleporting harvester's landing point and mark move at once)
    super::assign_destination(w, id, Some(dest));
}

/// Clears a mobile unit's destination.
fn drive_to_none(w: &mut World, id: ObjectId) {
    if let Some(o) = w.obj_mut(id) {
        o.path.steps.clear();
        // spec: S-0212 claim 21 and its state table, row "destination changes" (r97: for the driving and walk locomotors a different destination, or none, ends a running retry delay, path::destination_changed and path::waits; the ship and the other locomotors keep it, low confidence; the hovering locomotor's is S-0503's)
    }
    super::assign_destination(w, id, None);
}

fn zone(w: &World, id: ObjectId) -> Option<glasnost_rules::sim::MovementZone> {
    let o = w.obj(id)?;
    w.rules().unit(o.kind, o.type_index).map(|u| u.zone)
}

/// Whether the movement zone of `id` connects cell `a` with cell `b`
/// (S-0212 claim 4).
fn cells_connected(w: &mut World, id: ObjectId, a: CellXY, b: CellXY) -> bool {
    let Some(z) = zone(w, id) else { return false };
    crate::regions::ensure(w);
    matches!(
        (crate::regions::reach_in(w, a, z), crate::regions::reach_in(w, b, z)),
        (crate::regions::Reach::Region(x), crate::regions::Reach::Region(y)) if x == y
    )
}

/// Whether a harvester is connected to a refinery (S-0402 claim 8): the
/// cell of its heading point (its own position when it is not moving) and
/// the cell of the refinery's reference point lie in the same reach region
/// for the movement zone of its type, read for the deck when it is on a
/// bridge deck. A heading-point cell outside the usable area counts as
/// connected to every refinery.
fn connected(w: &mut World, id: ObjectId, refinery: ObjectId) -> bool {
    let Some(z) = zone(w, id) else { return false };
    let Some(o) = w.obj(id) else { return false };
    let deck = o.layer == Layer::Deck;
    // spec: S-0402 claim 8 (the cell of the heading point, S-0212 Terms; its own position when it is not moving)
    let from = if crate::occupancy::in_progress(w, id) {
        crate::path::heading_point(w, id).unwrap_or(o.pos).cell()
    } else {
        o.pos.cell()
    };
    // spec: S-0402 claim 8 (a heading-point cell outside the usable area: connected to every refinery)
    if !w.cells.usable(from) {
        return true;
    }
    let to = w.reference_point(refinery).cell();
    crate::regions::ensure(w);
    use crate::regions::Reach;
    // spec: S-0402 claim 8 (the same reach region, S-0212 claim 4, read for the deck when the harvester is on a bridge deck)
    let a = crate::regions::reach_read(w, from, z, deck);
    let b = crate::regions::reach_read(w, to, z, deck);
    a == b && matches!(a, Reach::Region(_) | Reach::LoneDeck)
}

/// The ring search with range `range` from cell `from` (S-0402 claim 4).
pub fn ring_search(w: &mut World, id: ObjectId, from: CellXY, range: i32) -> Option<CellXY> {
    let reference = crate::occupancy::reference_level(w, id);
    // design: C7-P5 field cells (quirks.harvest_cell_shared off: the cells other units of the harvester's house are heading for)
    let taken: BTreeSet<CellXY> = if w.rules().quirks.harvest_cell_shared {
        BTreeSet::new()
    } else {
        heading_for(w, id)
    };
    ring_search_with(w, from, range, &mut |w, c| {
        acceptable(w, id, from, c, reference) && !taken.contains(&c)
    })
}

/// The cells the other units of `id`'s house on the map are heading for
/// (their cell destinations; our design, C7 part 5, field cells).
fn heading_for(w: &World, id: ObjectId) -> BTreeSet<CellXY> {
    let owner = w.obj(id).and_then(|o| o.owner);
    w.objects
        .iter()
        .filter(|(x, o)| **x != id && o.on_map && o.owner == owner)
        .filter_map(|(_, o)| match o.dest {
            Some(Target::Cell(c)) => Some(c),
            _ => None,
        })
        .collect()
}

/// The ring search of S-0402 claim 4 from cell `from` with range `range`,
/// with the acceptability test `accept` (the searcher's own, claim 4's
/// for a harvester).
pub(crate) fn ring_search_with(
    w: &mut World,
    from: CellXY,
    range: i32,
    accept: &mut dyn FnMut(&mut World, CellXY) -> bool,
) -> Option<CellXY> {
    // spec: S-0402 claim 4 (the harvester's own cell, when its land type is tiberium, whatever the cell holds)
    if w.cells.cell(from).is_some_and(|c| c.land == land::TIBERIUM) {
        return Some(from);
    }
    // spec: S-0402 claim 4 (rings k = 1 … R − 1; nothing when R ≤ 1)
    for k in 1..range {
        let mut best: Option<(i64, (i32, i32), CellXY)> = None;
        for b in -k..=k {
            for a in -k..=k {
                if a.abs() != k && b.abs() != k {
                    continue;
                }
                let c = CellXY::new(from.x + a, from.y + b);
                if !accept(w, c) {
                    continue;
                }
                let worth = resource::worth(w, c);
                // spec: S-0402 claim 4 (tie key: top (a, 0), bottom (a, 1), left (b, 2), right (b, 3); the smallest pair)
                let mut key = (i32::MAX, i32::MAX);
                if b == -k {
                    key = key.min((a, 0));
                }
                if b == k {
                    key = key.min((a, 1));
                }
                if a == -k {
                    key = key.min((b, 2));
                }
                if a == k {
                    key = key.min((b, 3));
                }
                let better = match best {
                    None => true,
                    // spec: S-0402 claim 4 (the acceptable cell of greatest worth; ties to the smallest tie key)
                    Some((bw, bk, _)) => worth > bw || (worth == bw && key < bk),
                };
                if better {
                    best = Some((worth, key, c));
                }
            }
        }
        if let Some((_, _, c)) = best {
            // spec: S-0402 claim 4 (it stops at the first ring that holds an acceptable cell)
            return Some(c);
        }
    }
    None
}

/// An acceptable cell of the ring search (S-0402 claim 4).
fn acceptable(w: &mut World, id: ObjectId, from: CellXY, c: CellXY, reference: i32) -> bool {
    // spec: S-0402 claim 4 (in the usable area, height-aware; land type tiberium)
    if !w.cells.usable(c) || w.cells.cell(c).is_none_or(|x| x.land != land::TIBERIUM) {
        return false;
    }
    // spec: S-0402 claim 4 (the movement zone connects it with the harvester's cell)
    if !cells_connected(w, id, from, c) {
        return false;
    }
    // spec: S-0402 claim 4 (the entry outcome clear for a ground vehicle mover, S-0209 claim 5)
    crate::occupancy::vehicle_outcome(w, id, c, reference) == crate::occupancy::Outcome::Clear
}

/// Whether the owner owns a building of one of the harvester's dock types
/// (S-0402 claim 13).
fn owns_dock(w: &World, id: ObjectId) -> bool {
    let Some(o) = w.obj(id) else { return false };
    let Some(owner) = o.owner else { return false };
    let docks: &[u32] = unit(w, id).map_or(&[], |u| u.docks.as_slice());
    let dock = |b: &crate::world::Object| {
        b.kind == Kind::Building
            && b.owner == Some(owner)
            && b.health > 0
            && docks.contains(&b.type_index)
    };
    // The house's acquired buildings are looked at first: one of them that
    // passes the test answers yes, as the walk of every object below would.
    // Only when none passes is every object walked, so the answer is the
    // walk's whatever the list holds.
    let acquired = w
        .houses
        .get(usize::from(owner.0))
        .map_or(&[][..], |h| h.econ.acquired.as_slice());
    if acquired.iter().any(|&b| w.obj(b).is_some_and(dock)) {
        return true;
    }
    w.objects.values().any(dock)
}

/// The docking cell of a dock-unload refinery (S-0402 Terms, claim 10):
/// (*x*₀ + 3, *y*₀ + 1) from its top-left cell, whatever the size of its
/// foundation. A refinery that is not a dock-unload refinery has none.
pub fn docking_cell(w: &World, refinery: ObjectId) -> Option<CellXY> {
    // spec: S-0402 Terms (only a dock-unload refinery directs a harvester to its docking cell)
    if !building(w, refinery).is_some_and(|b| b.dock_unload) {
        return None;
    }
    let tl = w.obj(refinery)?.pos.cell();
    // spec: S-0402 Terms and Constants (the docking cell (x₀ + 3, y₀ + 1), independent of the foundation size and the art)
    Some(CellXY::new(tl.x + 3, tl.y + 1))
}

/// Whether a harvester is tethered to the refinery it holds contact with
/// (S-0402 claim 10). The tether link is with that refinery (glossary
/// *tether link*): a contact with anything else, such as a depot, is not
/// this tether (#185).
pub(crate) fn tethered(w: &World, id: ObjectId) -> bool {
    // spec: S-0402 claims 10 and 11 and glossary tether link (the harvester and the refinery it docks with are tethered; S-0213 table T3 (b): an order that breaks off that contact ends the tether)
    w.obj(id)
        .and_then(|o| o.contact)
        .is_some_and(|c| live_holders(w, c).contains(&id))
        && st(w, id).tethered
}

/// The harvesters docked with a refinery (S-0408 Terms): those holding a
/// dock slot and contact with it.
pub fn docked(w: &World, refinery: ObjectId) -> Vec<ObjectId> {
    live_holders(w, refinery)
}

/// Whether a refinery accepts a harvester (S-0402 claim 10); `waiting`: the
/// waiting choice, which does not look at slots.
fn accepts(w: &World, refinery: ObjectId, harvester: ObjectId, waiting: bool) -> bool {
    let Some(b) = w.obj(refinery) else {
        return false;
    };
    // spec: S-0402 claim 10 (not in its construction state, not being sold, not switched off)
    // spec: S-0406 claim 17 (the queued construction is seen while none is current)
    if matches!(
        super::placement::seen_behaviour(b),
        Some(Behaviour::Construction | Behaviour::Selling)
    ) || b.econ.sale.is_some()
        || b.econ.building.switched_off
    {
        return false;
    }
    // spec: S-0402 claims 8 and 10 (quirks.non_unloading_refinery_accepts off: a building without the
    // dock-unload flag is excluded from both choices and refuses refinery contact, an enter order included)
    if !w.rules().quirks.non_unloading_refinery_accepts
        && !building(w, refinery).is_some_and(|x| x.dock_unload)
    {
        return false;
    }
    if waiting {
        return true;
    }
    let slots = building(w, refinery).map_or(1, |x| x.slots);
    let holders = live_holders(w, refinery);
    // spec: S-0402 claim 10 (a dock slot free or already held by that harvester)
    holders.contains(&harvester) || (holders.len() as i32) < slots
}

/// The harvesters that hold a slot of a refinery and still hold contact.
fn live_holders(w: &World, refinery: ObjectId) -> Vec<ObjectId> {
    w.obj(refinery)
        .and_then(|b| b.econ.dock.as_ref())
        .map(|d| {
            d.holders
                .iter()
                .copied()
                .filter(|&h| w.obj(h).is_some_and(|x| x.contact == Some(refinery)))
                .collect()
        })
        .unwrap_or_default()
}

/// The harvester takes contact with a refinery and holds a dock slot.
fn take_contact(w: &mut World, harvester: ObjectId, refinery: ObjectId) {
    let mut holders = live_holders(w, refinery);
    if !holders.contains(&harvester) {
        holders.push(harvester);
    }
    if let Some(b) = w.obj_mut(refinery) {
        b.econ.dock = Some(DockState { holders });
    }
    if let Some(o) = w.obj_mut(harvester) {
        o.contact = Some(refinery);
    }
    if let Some(s) = state(w, harvester) {
        s.tethered = false;
    }
}

/// The harvester breaks off contact and frees its dock slot.
pub(crate) fn break_contact(w: &mut World, harvester: ObjectId) {
    let Some(r) = w.obj(harvester).and_then(|o| o.contact) else {
        return;
    };
    if let Some(o) = w.obj_mut(harvester) {
        o.contact = None;
    }
    if let Some(s) = state(w, harvester) {
        // spec: glossary (tether link: lost when the link is broken off)
        s.tethered = false;
    }
    if let Some(d) = w.obj_mut(r).and_then(|b| b.econ.dock.as_mut()) {
        d.holders.retain(|&h| h != harvester);
    }
}

/// An object leaves the map: a harvester frees its dock slot.
pub fn leaving_map(w: &mut World, id: ObjectId) {
    if is_harvester(w, id) {
        break_contact(w, id);
    }
}

/// The squared 2-D distance between two reference points (S-0402 claim 8,
/// F5), wrapping as a signed 32-bit integer under
/// `quirks.refinery_distance_wraps`.
fn distance_sq(w: &World, a: ObjectId, b: ObjectId) -> i64 {
    let pa = w.reference_point(a);
    let pb = w.reference_point(b);
    squared_distance(
        i64::from(pa.x) - i64::from(pb.x),
        i64::from(pa.y) - i64::from(pb.y),
        w.rules().quirks.refinery_distance_wraps,
    )
}

/// *dx*² + *dy*² of S-0402 claim 8 and F5: a signed 32-bit integer that
/// wraps when `wraps` (`quirks.refinery_distance_wraps`), else exact.
pub fn squared_distance(dx: i64, dy: i64, wraps: bool) -> i64 {
    let exact = dx * dx + dy * dy;
    if wraps {
        // spec: S-0402 F5 (dx² + dy² as a signed 32-bit integer that wraps; quirks.refinery_distance_wraps)
        i64::from(exact as i32)
    } else {
        exact
    }
}

/// The 3-D distance of S-0204's table square root, truncated (S-0402
/// claim 9).
fn distance_3d(w: &World, a: ObjectId, b: ObjectId) -> i64 {
    let pa = w.reference_point(a);
    let pb = w.reference_point(b);
    let dx = i64::from(pa.x) - i64::from(pb.x);
    let dy = i64::from(pa.y) - i64::from(pb.y);
    let dz = i64::from(pa.z) - i64::from(pb.z);
    let r = w.rules();
    // spec: S-0402 claim 9 (the 3-D distance between the reference points by S-0204's table square root, truncated)
    root_floor(
        (dx * dx + dy * dy + dz * dz) as u64,
        r.quirks.table_root_distance,
        r.mode,
    ) as i64
}

/// One level of the choice of claim 8: candidates (id, distance, primary)
/// in order.
fn pick(c: &[(ObjectId, i64, bool)]) -> Option<ObjectId> {
    // spec: S-0402 claim 8 (P, the last primary building of the level; the nearest among P and everything after it; no primary: the nearest; ties to the earliest)
    let from = c.iter().rposition(|x| x.2).unwrap_or(0);
    let mut best: Option<(ObjectId, i64)> = None;
    for &(id, d, _) in &c[from..] {
        if best.is_none_or(|(_, bd)| d < bd) {
            best = Some((id, d));
        }
    }
    best.map(|b| b.0)
}

/// The refinery choice of S-0402 claim 8 (`waiting`: the waiting choice of
/// claim 9, which does not look at slots).
pub fn choose_refinery(w: &mut World, id: ObjectId, waiting: bool) -> Option<ObjectId> {
    let o = w.obj(id)?.clone();
    let owner = o.owner?;
    let docks = unit(w, id)?.docks.clone();
    // spec: S-0402 Terms (the holding order: joined when put on the map for the owner, left when it ceases to exist or changes owner; econ::events keeps it)
    let order = w.houses[usize::from(owner.0)].econ.acquired.clone();
    let mut bests = Vec::new();
    for &ty in &docks {
        let mut cands = Vec::new();
        // spec: S-0402 claim 8 (within each dock type, the owner's candidates by holding order)
        for &b in &order {
            let Some(bo) = w.obj(b) else { continue };
            if bo.kind != Kind::Building
                || bo.type_index != ty
                || bo.owner != Some(owner)
                || !bo.on_map
            {
                continue;
            }
            // spec: S-0402 claim 8 (connected by the movement zone, and accepting it)
            if !connected(w, id, b) || !accepts(w, b, id, waiting) {
                continue;
            }
            let primary = w.obj(b).is_some_and(|x| x.econ.building.primary);
            cands.push((b, distance_sq(w, id, b), primary));
        }
        if let Some(best) = pick(&cands) {
            let primary = w.obj(best).is_some_and(|x| x.econ.building.primary);
            bests.push((best, distance_sq(w, id, best), primary));
        }
    }
    // spec: S-0402 claim 8 (across types, the bests by the dock list; a best counts as primary only if it is one)
    pick(&bests)
}

/// The standard delay of the current behaviour (S-0215 claim 6).
fn standard(w: &mut World, id: ObjectId) -> i32 {
    crate::process::standard_delay(w, id)
}

/// The processing of *harvest* for a harvester (S-0402 claims 3 to 9 and
/// 13; State and ordering).
pub fn process_harvest(w: &mut World, id: ObjectId) -> i32 {
    let Some(start) = w.obj(id).map(|o| o.beh.start_tick) else {
        return 1;
    };
    if let Some(s) = state(w, id)
        && s.harvest_start != Some(start)
    {
        // spec: S-0402 Terms (a harvester enters harvest in the seeking phase)
        s.harvest_start = Some(start);
        s.phase = Phase::Seeking;
        s.pending_load_period = false;
    }
    // spec: S-0402 claim 22 (a miner with gatherer and destination fields and a slave group processes harvest through its group rule, not claims 3 to 10)
    if let Some(d) = super::slaves::process_harvest(w, id) {
        return d;
    }
    // spec: S-0402 claim 3 and claim 13 (in every phase: no building of a dock type, guard queued, 1 tick)
    if !owns_dock(w, id) {
        if let Some(ob) = w.obj_mut(id) {
            ob.beh.queue(Some(Behaviour::Guard));
        }
        return 1;
    }
    match st(w, id).phase {
        Phase::Seeking => seeking(w, id),
        Phase::Loading => loading(w, id),
        Phase::Returning => returning(w, id),
        Phase::Requesting => {
            // spec: S-0402 State and ordering (requesting: queues enter, 1 tick)
            if let Some(ob) = w.obj_mut(id) {
                ob.beh.queue(Some(Behaviour::Enter));
            }
            1
        }
        Phase::Idle => {
            step_off_refinery(w, id);
            // spec: S-0402 claim 13 (105 ticks later it queues guard; State and ordering: the standard delay)
            if let Some(ob) = w.obj_mut(id) {
                ob.beh.queue(Some(Behaviour::Guard));
            }
            standard(w, id)
        }
    }
}

/// The idle step-off (S-0402 claim 13): off the cell of a refinery-flag
/// or weed-flag building, to the cell of S-0212's nearby-cell search with
/// claim 13's settings.
fn step_off_refinery(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id).cloned() else {
        return;
    };
    let c = o.pos.cell();
    // spec: S-0402 claim 13 (a refinery-flag building, or a weed-flag building, Import mapping)
    let Some(b) = w
        .cells
        .list(c, Layer::Ground)
        .iter()
        .copied()
        .find(|&b| building(w, b).is_some_and(|x| x.refinery || x.weeder))
    else {
        return;
    };
    // spec: S-0402 claim 13 (the centre: the cell of that building's reference point)
    let centre = w.reference_point(b).cell();
    crate::regions::ensure(w);
    // spec: S-0402 claim 13 and S-0212 table T5 (the step-off row: the harvester's own class, winged as track, its own zone and the centre's reach region read for its deck, deck mode on a bridge deck, no height condition, no further test, no reference cell: the preferred cell at position N mod n)
    let Some(settings) = crate::path::NearbySettings::own_class(w, id, centre) else {
        return;
    };
    match crate::path::nearby(w, centre, crate::path::NearbyUse::Settings(settings)) {
        Some(n) => drive_to(w, id, Target::Cell(n)),
        None => {
            if w.rules().quirks.harvester_failed_step_off_origin {
                // spec: S-0402 claim 13 and Edge cases (no result: the destination is nevertheless replaced by the off-map cell (0, 0), quirks.harvester_failed_step_off_origin; no draw, no retry)
                drive_to(w, id, Target::Cell(CellXY::new(0, 0)));
            } else {
                // spec: S-0402 Edge cases (quirks.harvester_failed_step_off_origin off: the failure clears the destination)
                drive_to_none(w, id);
            }
        }
    }
}

fn seeking(w: &mut World, id: ObjectId) -> i32 {
    // spec: S-0402 State and ordering (seeking, full: returning, 1 tick)
    if full(w, id) {
        set_phase(w, id, Phase::Returning);
        return 1;
    }
    // spec: S-0402 claim 17 (the seeking drop holds for a harvester whose current locomotor is the teleport locomotor at the test, whether or not its type has the teleporter flag; not for a flagged type moving by another locomotor, S-0417 claim 16)
    let drops = |w: &World| crate::teleport::is_teleporter(w, id);
    let s = st(w, id);
    if let Some(c) = s.remembered {
        // spec: S-0402 claim 3 (a remembered cell: it drives there and forgets it; no search)
        if let Some(x) = state(w, id) {
            x.remembered = None;
        }
        drive_to(w, id, Target::Cell(c));
        // spec: S-0402 claim 17 (Who reaches the drop: the adoption is an ordinary assignment, which installs driving before the drop's test outside a qualified docking assignment, S-0207 claim 28; then the cell is not dropped)
        if !drops(w) {
            return standard(w, id);
        }
        // spec: S-0402 claim 17 (the seeking drop: the remembered cell, adopted by an ordinary destination assignment, is removed at once, so the miner never jumps to it; the processing then takes the ring-search rows of claim 3)
        drive_to_none(w, id);
    } else if w.obj(id).is_some_and(|o| o.dest.is_some()) {
        if !drops(w) {
            // spec: S-0402 claim 3 (a destination: stays seeking, no search)
            return standard(w, id);
        }
        // spec: S-0402 claim 17 (the seeking drop at every seeking processing past the full test: the order-level destination is removed and the pending jump cancelled, S-0207 claim 20 h; the landing point and its reservation stay; the ring search then runs from the miner's own cell)
        drive_to_none(w, id);
    }
    let Some(here) = w.obj(id).map(|o| o.pos.cell()) else {
        return 1;
    };
    let range = w.rules().econ.harvest.long_scan;
    match ring_search(w, id, here, range) {
        Some(c) if c == here => {
            let t = w.tick;
            let zs = zero_stalls(w);
            if let Some(x) = state(w, id) {
                // spec: S-0402 claim 5 (loading starts: counter 0, pacing period 2 started now)
                x.counter.zero(t);
                x.counter.start_period(2, t, zs); // spec: S-0402 Constants (pacing period at loading start: 2 ticks)
                x.pending_load_period = false;
                x.phase = Phase::Loading;
            }
            1
        }
        Some(c) => {
            // spec: S-0402 claim 3 (another cell: it drives there and stays seeking)
            drive_to(w, id, Target::Cell(c));
            standard(w, id)
        }
        None => {
            // spec: S-0402 claim 13 (no field: idle; the house's no-field record is set)
            set_phase(w, id, Phase::Idle);
            if let Some(h) = w.obj(id).and_then(|o| o.owner) {
                w.houses[usize::from(h.0)].econ.harvest.no_field = true;
            }
            105 // spec: S-0402 Constants (idle interval after no field is found: 105 ticks)
        }
    }
}

fn loading(w: &mut World, id: ObjectId) -> i32 {
    let t = w.tick;
    let zs = zero_stalls(w);
    let l = w.rules().econ.harvest.load_rate;
    if let Some(x) = state(w, id)
        && x.pending_load_period
    {
        // spec: S-0402 claim 5 table (the next processing in loading starts the period L)
        x.pending_load_period = false;
        x.counter.start_period(l, t, zs);
    }
    // spec: S-0402 claim 5 (a load step only when the counter is at least 9)
    if counter(w, id) < 9 {
        return 1;
    }
    let Some(o) = w.obj(id).cloned() else {
        return 1;
    };
    // spec: S-0402 claim 5 table (a destination: no step, it stays in loading)
    if o.dest.is_some() {
        return 1;
    }
    let cell = o.pos.cell();
    if capacity(w, id) == 0 {
        // spec: S-0402 claim 5 table and Edge cases (capacity 0: no bale, counter unchanged; then counted as full)
        return now_full(w, id, cell);
    }
    if full(w, id) || w.cells.cell(cell).is_none_or(|c| c.land != land::TIBERIUM) {
        let was_full = full(w, id);
        if let Some(x) = state(w, id) {
            // spec: S-0402 claim 5 table (a failure because full or off the field: counter 0, period 0 until the next processing in loading)
            x.counter.zero(t);
            x.counter.start_period(0, t, zs);
            x.pending_load_period = true;
        }
        return if was_full {
            now_full(w, id, cell)
        } else {
            exhausted(w, id, cell)
        };
    }
    let tib = resource::resource_at(w, cell).map(|(tt, _)| tt);
    // spec: S-0402 claim 5 (a load step takes one bale from the harvester's cell, S-0401 claim 3)
    let got = resource::take_bales(w, cell, 1);
    if got > 0
        && let Some(tt) = tib
    {
        if let Some(x) = state(w, id) {
            // spec: S-0402 claim 1 (each bale is added to the amount of the tiberium of the cell it came from)
            x.load[tt] += got;
            // spec: S-0402 claim 5 table (a load step succeeds: counter 0, period L started)
            x.counter.zero(t);
            x.counter.start_period(l, t, zs);
        }
        return 1;
    }
    // spec: S-0402 claim 5 table (a failure only because the last bale gave nothing: counter and period unchanged)
    exhausted(w, id, cell)
}

/// Claim 6: the harvester is full.
fn now_full(w: &mut World, id: ObjectId, cell: CellXY) -> i32 {
    let range = w.rules().econ.harvest.short_scan;
    // spec: S-0402 claim 6 (ring search with the short scan range, remembered or forgotten; returning)
    let found = ring_search(w, id, cell, range);
    if let Some(x) = state(w, id) {
        x.remembered = found;
        x.phase = Phase::Returning;
    }
    1
}

/// Claim 7: the cell is exhausted and the harvester is not full.
fn exhausted(w: &mut World, id: ObjectId, cell: CellXY) -> i32 {
    let range = w.rules().econ.harvest.short_scan;
    // spec: S-0402 claim 7 (ring search with the short scan range from its cell)
    match ring_search(w, id, cell, range) {
        Some(c) => {
            if c != cell {
                // spec: S-0402 claim 7 (it drives there, or stays for its own cell, and keeps loading)
                drive_to(w, id, Target::Cell(c));
            }
        }
        None => {
            if w.obj(id).is_some_and(|o| o.dest.is_none()) {
                // spec: S-0402 claim 7 (none found and no destination: forgets, returning with whatever it carries)
                if let Some(x) = state(w, id) {
                    x.remembered = None;
                    x.phase = Phase::Returning;
                }
            }
        }
    }
    1
}

fn returning(w: &mut World, id: ObjectId) -> i32 {
    let chrono = unit(w, id).is_some_and(|u| u.teleporter);
    if w.obj(id).is_some_and(|o| o.dest.is_some()) {
        if chrono && choose_refinery(w, id, false).is_some() {
            // spec: S-0402 claim 17 (a chrono miner moving in returning drops its movement when the normal choice finds a refinery; its next processing reserves it)
            crate::drive::stop(w, id);
            super::assign_destination(w, id, None);
        }
        // spec: S-0402 State and ordering (returning with a destination: keeps driving)
        return standard(w, id);
    }
    let hr = &w.rules().econ.harvest;
    // spec: S-0402 claim 9 (the too-far field, chrono miners' field for a chrono miner, in cells × 256 leptons)
    let limit = i64::from(if chrono {
        hr.chrono_too_far
    } else {
        hr.too_far
    }) * 256;
    if let Some(r) = choose_refinery(w, id, false)
        && distance_3d(w, id, r) <= limit
    {
        // spec: S-0402 claim 9 (within the too-far distance, equality included: it asks for contact; accepted: requesting)
        take_contact(w, id, r);
        set_phase(w, id, Phase::Requesting);
        return standard(w, id);
    }
    // spec: S-0402 claim 9 (otherwise the waiting choice)
    if let Some(r) = choose_refinery(w, id, true) {
        // spec: S-0402 claim 9 (an ordinary harvester at most 768 leptons from it stays; a chrono miner always moves)
        if !chrono && distance_3d(w, id, r) <= 768 {
            return standard(w, id);
        }
        // spec: S-0402 claim 9 (it drives to the search's cell; with none, its destination is the refinery building itself)
        let dest = queueing_cell(w, r).map_or(Target::Object(r), Target::Cell);
        drive_to(w, id, dest);
    }
    standard(w, id)
}

/// Where a harvester queues at a refinery (S-0402 claim 9): the result of
/// S-0212's nearby-cell search around the queueing cell (the refinery's
/// top-left cell moved by its queueing shift) with claim 9's settings, or
/// none.
fn queueing_cell(w: &mut World, refinery: ObjectId) -> Option<CellXY> {
    use glasnost_rules::sim::{LocomotionClass, MovementZone};
    let tl = w.obj(refinery)?.pos.cell();
    let (qx, qy) = building(w, refinery).map_or((0, 0), |x| x.queueing);
    // spec: S-0402 Terms (the queueing cell: the top-left cell moved by the queueing shift)
    let q = CellXY::new(tl.x + qx, tl.y + qy);
    crate::regions::ensure(w);
    let settings = crate::path::NearbySettings {
        // spec: S-0402 claim 9 and Constants (locomotion class wheel, movement zone normal, whatever the harvester's own)
        class: Some(LocomotionClass::Wheel),
        zone: MovementZone::Normal,
        // spec: S-0402 claim 9 (no reach region; never in deck mode; no height condition; no further test)
        reach: None,
        deck: false,
        height: false,
        // spec: S-0402 claim 9 (the reference cell: the refinery's top-left cell, without the queueing shift)
        reference: Some(tl),
        no_bridges: false,
        no_overlay: false,
    };
    crate::path::nearby(w, q, crate::path::NearbyUse::Settings(settings))
}

/// The processing of *enter* for a harvester (S-0402 claim 10): in
/// contact with a dock-unload refinery, it asks to dock, and the first
/// matching row of claim 10's table applies. Every processing is followed
/// by the next after the standard delay of *enter*.
pub fn process_enter(w: &mut World, id: ObjectId) -> i32 {
    let Some(o) = w.obj(id).cloned() else {
        return 1;
    };
    let refinery = match o.contact {
        Some(r) if w.obj(r).is_some() => Some(r),
        _ => match o.dest {
            // spec: S-0402 claim 2 (an enter order on a refinery starts docking directly)
            Some(Target::Object(r))
                if accepts(w, r, id, false)
                    && w.obj(r).is_some_and(|b| b.kind == Kind::Building) =>
            {
                take_contact(w, id, r);
                Some(r)
            }
            _ => None,
        },
    };
    let Some(r) = refinery else {
        crate::process::fall_back(w, id, false);
        return 1;
    };
    // spec: S-0402 claim 10 row 1 (the refinery refuses, it does when switched off, and the harvester is not tethered: it breaks off contact and falls back to rest)
    if !tethered(w, id) && !accepts(w, r, id, false) {
        break_contact(w, id);
        crate::process::fall_back(w, id, false);
        return standard(w, id);
    }
    // spec: S-0402 claim 10 and Edge cases (a refinery that is not a dock-unload refinery never directs the harvester: it stays in enter)
    let Some(dc) = docking_cell(w, r) else {
        repeated_dock_test(w, id);
        return standard(w, id);
    };
    if o.pos.cell() != dc {
        // spec: S-0402 claim 10 row 2 (not on the docking cell: its destination is set to it, an ordinary move)
        // spec: S-0402 claim 10 row 2 (revision 26: the docking cell is offered at each such processing; an unchanged offer is ignored unless pending destination refresh makes it effective, S-0207 claim 27, so it reaches the dock test again after a fallback)
        if o.dest != Some(Target::Cell(dc)) || crate::refresh::pending(w, id) {
            // spec: S-0402 claim 10a (the path computation on an offer of the docking cell waits for a running retry delay like any fresh search) and S-0212 claim 21 (r97: those offers are left to S-0402 claim 10a, so the change to the docking cell does not end the delay)
            let held = crate::path::running_delay(w, id);
            drive_to(w, id, Target::Cell(dc));
            crate::path::keep_delay(w, id, held);
        }
        // spec: S-0207 claim 28 (One jump: when the offer and the repeated dock test both act in one processing, the offer comes first)
        repeated_dock_test(w, id);
        return standard(w, id);
    }
    repeated_dock_test(w, id);
    // spec: S-0402 claim 10 rows 3 to 5 (on the docking cell: it and the refinery become tethered)
    if let Some(s) = state(w, id) {
        s.tethered = true;
    }
    let t = w.tick;
    // spec: S-0402 claim 10 and Constants (the facing aimed at on the docking cell: exactly 16384)
    const DOCK_FACING: u16 = 16384;
    if w.body_facing(id) != DOCK_FACING {
        let rate = w
            .rules()
            .unit(o.kind, o.type_index)
            .map_or(0, |u| u.turn_rate);
        if let Some(ob) = w.obj_mut(id) {
            // spec: S-0402 claim 10 row 3 (facing not exactly 16384: it turns its body on the spot towards 16384)
            ob.body.turn_to(DOCK_FACING, t, rate);
        }
    } else if !crate::occupancy::in_progress(w, id) {
        let sold = w
            .obj(r)
            .is_some_and(|b| b.econ.sale.is_some() || b.beh.seen() == Some(Behaviour::Selling));
        // spec: S-0402 claim 10 row 4 (facing 16384 and not moving: the refinery queues unload on it, without an immediate start, unless it is being sold)
        if !sold && let Some(ob) = w.obj_mut(id) {
            ob.beh.queue(Some(Behaviour::Unload));
        }
    }
    // spec: S-0402 claim 10 row 5 (still moving: tethered, nothing else) and the standard delay of enter
    standard(w, id)
}

/// S-0207 claim 28's repeated dock test in *enter* (S-0402 claim 10): at a
/// processing that the refinery accepts, or at which the harvester is
/// tethered, a vehicle whose type has the teleport-capable flag and holds
/// a destination has it treated as a new assignment, even when unchanged.
/// The caller has passed row 1.
fn repeated_dock_test(w: &mut World, id: ObjectId) {
    // spec: S-0207 claim 28 (the repeated dock test: a teleport-capable vehicle holding a destination in enter, while its contact accepts it or it is tethered; a vehicle with no destination makes no such jump)
    if crate::refresh::flagged(w, id) && w.obj(id).is_some_and(|o| o.dest.is_some()) {
        crate::refresh::reassign(w, id);
    }
}

/// The second route to unloading (S-0402 claim 10): a driving harvester
/// has just reached its track target, its arrival position applied and the
/// finished track cleared, before cell-centre crushing and any next track.
/// A tethered harvester currently in *enter*, in contact with a building
/// occupying the step-direction-0 neighbour of its arrival cell, reports
/// arrival to it. A dock-unload refinery not in *selling* queues *unload*
/// without a start; one in *selling* refuses, and the harvester receives a
/// forced vehicle scatter request without a threat point, not move-anywhere.
/// No draw, no processing, no new delay. A report to any other building
/// belongs to that building's entry-exchange spec (a seam: none is modelled).
pub fn track_arrival(w: &mut World, id: ObjectId) {
    if !is_harvester(w, id) {
        return;
    }
    let Some(o) = w.obj(id) else { return };
    // spec: S-0402 claim 10 (second route: tethered, currently in enter, in contact with a building)
    if o.beh.current != Some(Behaviour::Enter) || !tethered(w, id) {
        return;
    }
    let Some(b) = o.contact else { return };
    let c = o.pos.cell();
    // spec: S-0402 claim 10 (the building occupies the neighbour in step direction 0, map north, of the arrival cell)
    let (dx, dy) = crate::regions::STEPS[0];
    let n = CellXY::new(c.x + dx, c.y + dy);
    if !w.cells.list(n, Layer::Ground).contains(&b) {
        return;
    }
    let docks = unit(w, id).map(|u| u.docks.clone()).unwrap_or_default();
    let refinery = w
        .obj(b)
        .is_some_and(|x| x.kind == Kind::Building && docks.contains(&x.type_index));
    // spec: S-0402 claim 10 (a dock-unload refinery; no facing, stopped or docking-cell test, and claim 8's acceptance test is not repeated)
    if !refinery || !building(w, b).is_some_and(|x| x.dock_unload) {
        return;
    }
    let selling = w
        .obj(b)
        .is_some_and(|x| x.beh.current == Some(Behaviour::Selling));
    if selling {
        // spec: S-0402 claim 10 (refused: a forced vehicle scatter request, S-0207 claim 16, without a threat point and not move-anywhere; no draw; contact, tether and behaviours kept)
        crate::scatter::request(
            w,
            id,
            crate::scatter::ScatterRequest {
                threat: None,
                forced: true,
                anywhere: false,
            },
        );
    } else if let Some(ob) = w.obj_mut(id) {
        // spec: S-0402 claim 10 (accepted: unload queued without an immediate start)
        ob.beh.queue(Some(Behaviour::Unload));
    }
}

/// Whether the body faces map east (S-0402 claim 11).
fn facing_east(w: &World, id: ObjectId) -> bool {
    // spec: S-0402 claim 11 (facing map east: 16256 ≤ f ≤ 16511)
    (16256..=16511).contains(&w.body_facing(id))
}

/// The building in the neighbouring cell (step direction 6 of the
/// harvester's cell, S-0402 claim 11).
fn west_building(w: &World, id: ObjectId) -> Option<ObjectId> {
    let c = w.obj(id)?.pos.cell();
    // spec: S-0402 Terms and claim 11 (the neighbouring cell: step direction 6, map west)
    let (dx, dy) = crate::regions::STEPS[6];
    let n = CellXY::new(c.x + dx, c.y + dy);
    w.cells.list(n, Layer::Ground).iter().copied().find(|&b| {
        w.obj(b)
            .is_some_and(|x| x.kind == Kind::Building && x.health > 0)
    })
}

/// The processing of *unload* for a harvester (S-0402 claim 11).
pub fn process_unload(w: &mut World, id: ObjectId) -> i32 {
    let stale = w.rules().quirks.stale_suspended_revert; // spec: S-0215 Edge cases (quirks.stale_suspended_revert)
    let t = w.tick;
    let zs = zero_stalls(w);
    let Some(o) = w.obj(id).cloned() else {
        return 1;
    };
    let start = o.beh.start_tick;
    if let Some(s) = state(w, id)
        && s.unload_start != Some(start)
    {
        s.unload_start = Some(start);
        s.unloading = false;
        s.finishing = false;
    }
    if o.contact.is_none() {
        return no_contact(w, id);
    }
    let s = st(w, id);
    if !s.unloading {
        if !facing_east(w, id) {
            let rate = w
                .obj(id)
                .and_then(|x| w.rules().unit(x.kind, x.type_index))
                .map_or(0, |u| u.turn_rate);
            if let Some(ob) = w.obj_mut(id) {
                // spec: S-0402 claim 11 (it turns its body towards map east: the facing of step direction 2, 8192 × 2, S-0207 claim 6)
                ob.body.turn_to(16384, t, rate);
            }
            // spec: S-0402 Constants (processing interval while turning to dock: 5 ticks)
            return 5;
        }
        if let Some(x) = state(w, id) {
            // spec: S-0402 claim 11 (unloading starts: counter 0, a pacing period of 1 tick started now)
            x.counter.zero(t);
            x.counter.start_period(1, t, zs); // spec: S-0402 Constants (pacing period while unloading: 1 tick)
            x.unloading = true;
        }
        if let Some(b) = west_building(w, id) {
            // spec: S-0402 claim 11 (the building in the neighbouring cell, if any, plays its docking animation)
            crate::depot_anim::docking(w, b);
        }
        return standard(w, id);
    }
    let Some(b) = west_building(w, id) else {
        if !s.finishing {
            // spec: S-0402 claim 11 (unloading, no building in the neighbouring cell: breaks off contact, harvest with an immediate start)
            leave_unloading(w, id);
            break_contact(w, id);
            if let Some(ob) = w.obj_mut(id) {
                ob.beh.queue_and_start_at_once(Behaviour::Harvest, t, stale);
            }
            return 1;
        }
        return finish(w, id);
    };
    let refinery = building(w, b).is_some_and(|x| x.refinery);
    if s.finishing {
        if refinery && crate::depot_anim::unloading_playing(w, b) {
            // spec: S-0402 claim 11 (finishing, and the building in the neighbouring cell is a refinery-flag building still playing its unloading animation: nothing, 1 tick)
            return 1;
        }
        return finish(w, id);
    }
    let thr = w.rules().econ.harvest.dump_threshold;
    let m = w.rules().mode;
    let c = s.counter.value(t, zs);
    // spec: S-0402 claim 11 (counter < 900 × D, the integer compared with the double product: nothing, 1 tick)
    if F64::from_i64(i64::from(c), m).lt(thr) {
        return 1;
    }
    // spec: S-0402 claim 11 (the tiberium it holds with the lowest registration index, the whole amount at once)
    if let Some((tib, &amount)) = s.load.iter().enumerate().find(|(_, a)| **a > 0) {
        if let Some(x) = state(w, id) {
            x.load[tib] = 0;
            // spec: S-0402 claim 11 (counter set to 0; its period is not restarted)
            x.counter.zero(t);
        }
        // spec: S-0402 claim 11 (the building plays its payment animation unless that is already playing; before the smoke event, Q-3062's interim)
        crate::depot_anim::payment(w, b);
        // spec: S-0402 claim 11 and S-0616 claim 1 (each payment row makes one smoke event on the building in the neighbouring cell, before the payment of its processing)
        crate::emitters::refinery_smoke(w, b);
        pay(w, b, tib, amount);
        return 1;
    }
    // spec: S-0402 claim 11 (load empty: the payment animation, if playing, is stopped, and a refinery-flag building plays its unloading animation; before the smoke event, Q-3062's interim)
    crate::depot_anim::load_emptied(w, b, refinery);
    // spec: S-0402 claim 11 and S-0616 claim 1 (the empty-load row that makes the harvester finishing also makes one smoke event, so k + 1 events for k tiberiums)
    crate::emitters::refinery_smoke(w, b);
    // spec: S-0402 claim 11 (load empty and the counter allows: finishing, the counter left as it is)
    if let Some(x) = state(w, id) {
        x.finishing = true;
    }
    1
}

/// Leaves the unloading state (S-0402 claim 11).
fn leave_unloading(w: &mut World, id: ObjectId) {
    if let Some(x) = state(w, id) {
        x.unloading = false;
        x.finishing = false;
    }
}

/// Whether a harvester is in the unloading state of S-0402 claim 11: from
/// the start row of *unload* until it leaves. A vehicle in that state is
/// not ready to change (S-0215 claim 11, the letting-out transition).
pub fn in_unloading_state(w: &World, id: ObjectId) -> bool {
    w.obj(id).is_some_and(|o| {
        o.beh.current == Some(Behaviour::Unload)
            && o.econ.harvest.as_ref().is_some_and(|s| s.unloading)
    })
}

/// The first row of claim 11: a processing of *unload* not in contact with
/// anything.
fn no_contact(w: &mut World, id: ObjectId) -> i32 {
    let stale = w.rules().quirks.stale_suspended_revert; // spec: S-0215 Edge cases (quirks.stale_suspended_revert)
    let t = w.tick;
    let stalls = w.rules().quirks.unload_without_contact_stalls;
    if stalls {
        // spec: S-0402 claim 11 row 1 (falls back to rest, S-0215 claim 15, which queues nothing while unload is current; quirks.unload_without_contact_stalls)
        crate::process::fall_back(w, id, false);
    } else {
        // spec: S-0402 Edge cases (quirks.unload_without_contact_stalls off: the fall-back as if unload were not current)
        let cur = w.obj_mut(id).map(|o| o.beh.current.take());
        crate::process::fall_back(w, id, false);
        if let (Some(cur), Some(o)) = (cur, w.obj_mut(id)) {
            o.beh.current = cur;
        }
    }
    // spec: S-0402 claim 11 row 1 (leaves the unloading state; stops if it is moving)
    leave_unloading(w, id);
    if crate::occupancy::in_progress(w, id) {
        crate::drive::stop(w, id);
    }
    // spec: S-0402 claim 11 row 1 (then a start-next if it has a queued behaviour and is ready to change; with the switch off it starts at once)
    let ready = !stalls || crate::process::ready(w, id);
    if ready && let Some(ob) = w.obj_mut(id) {
        ob.beh.start_next(t, stale);
    }
    // spec: S-0402 Constants (processing interval of unload without contact: 1 tick)
    1
}

/// The two finishing rows of claim 11.
fn finish(w: &mut World, id: ObjectId) -> i32 {
    let stale = w.rules().quirks.stale_suspended_revert; // spec: S-0215 Edge cases (quirks.stale_suspended_revert)
    let t = w.tick;
    let Some(o) = w.obj(id).cloned() else {
        return 1;
    };
    leave_unloading(w, id);
    if o.dest.is_some() && o.beh.queued.is_some_and(|b| b != Behaviour::Harvest) {
        // spec: S-0402 claim 11 (finishing with a destination and a queued behaviour other than harvest: leaves the unloading state, stops if moving, starts the queued behaviour if ready; contact kept)
        if crate::occupancy::in_progress(w, id) {
            crate::drive::stop(w, id);
        }
        if crate::process::ready(w, id)
            && let Some(ob) = w.obj_mut(id)
        {
            ob.beh.start_next(t, stale);
        }
        // spec: S-0402 claim 11 (the standard delay of the behaviour then current, one draw)
        return standard(w, id);
    }
    // spec: S-0402 claim 11 (finishing, otherwise: queues harvest without an immediate start; if ready to change, breaks off contact and starts the queued behaviour)
    if let Some(ob) = w.obj_mut(id) {
        ob.beh.queue(Some(Behaviour::Harvest));
    }
    if crate::process::ready(w, id) {
        break_contact(w, id);
        if let Some(ob) = w.obj_mut(id) {
            ob.beh.start_next(t, stale);
        }
    }
    // spec: S-0402 claim 11 (the standard delay of the behaviour then current, one draw)
    standard(w, id)
}

/// A refinery's sale reaches its first processing (S-0402 claim 11, S-0408
/// claim 6): every harvester docked with it is told to leave. One in the
/// unloading state leaves it, queues *harvest* and starts it, keeping the
/// unpaid part of its load, then scatters; one still in *enter* queues
/// *guard*, loses a destination naming the refinery and scatters. The
/// contact holds until the sale's next processing
/// ([`sale_breaks_contact`]).
pub fn refinery_sold(w: &mut World, refinery: ObjectId) {
    let stale = w.rules().quirks.stale_suspended_revert; // spec: S-0215 Edge cases (quirks.stale_suspended_revert)
    let t = w.tick;
    for h in docked(w, refinery) {
        if in_unloading_state(w, h) {
            // spec: S-0402 claim 11 (refinery sold: a harvester in the unloading state leaves it at once, queues harvest and starts it if it is ready to change, keeping its unpaid load)
            leave_unloading(w, h);
            let ready = crate::process::ready(w, h);
            if let Some(ob) = w.obj_mut(h) {
                ob.beh
                    .queue_and_start(Some(Behaviour::Harvest), ready, t, stale);
            }
            // spec: S-0402 claim 11 (in the same processing a forced vehicle scatter request without a threat point, only while it has no destination; refused, by S-0207 claim 16, when harvest could not start and unload is still current)
            sale_scatter(w, h);
        } else if w
            .obj(h)
            .is_some_and(|o| o.beh.seen() == Some(Behaviour::Enter))
        {
            // spec: S-0402 claim 11 (refinery sold: a harvester still in enter, in contact with the refinery, queues guard instead)
            if let Some(ob) = w.obj_mut(h) {
                ob.beh.queue(Some(Behaviour::Guard));
                // spec: S-0402 claim 11 (a destination naming the refinery building itself is removed first; a cell destination, such as the docking cell, is kept)
                if ob.dest == Some(Target::Object(refinery)) {
                    ob.dest = None;
                }
            }
            // spec: S-0402 claim 11 (then, only with no destination, the same vehicle scatter request, which a turning body still refuses: S-0207 claim 16)
            sale_scatter(w, h);
        }
        // spec: S-0402 claim 11 (the contact holds until the sale's next processing: sale_breaks_contact)
    }
}

/// The sale's second processing breaks off every contact with the
/// refinery (S-0402 claim 11 and its change for S-0408 claim 6), whether or
/// not the harvester is still tethered. Returns whether a tethered
/// harvester was docked when it came, which holds back the releases and
/// the build-down, not the end of its contact.
pub fn sale_breaks_contact(w: &mut World, refinery: ObjectId) -> bool {
    let holders = docked(w, refinery);
    // spec: S-0402 Changes for other specs (S-0408 claim 6: a tethered unit holds up only the releases and the build-down; a contact without a tether does not)
    let held = holders.iter().any(|&h| tethered(w, h));
    for h in holders {
        // spec: S-0402 claim 11 (the sale breaks off the contact at its next processing, 1 tick after the first)
        break_contact(w, h);
    }
    held
}

/// The scatter a refinery's sale sends a docked harvester (S-0402 claim
/// 11): a forced vehicle scatter request (S-0207 claim 16) without a
/// threat point and not move-anywhere, with the usual refusals; no draw.
fn sale_scatter(w: &mut World, id: ObjectId) {
    // spec: S-0402 claim 11 (forced, no threat point, not move-anywhere)
    crate::scatter::request(
        w,
        id,
        crate::scatter::ScatterRequest {
            threat: None,
            forced: true,
            anywhere: false,
        },
    );
}

/// A building's grand opening (S-0402 claim 18): its free unit, if its
/// type has one, unless claim 18 withholds it.
pub fn grand_opening(w: &mut World, building: ObjectId) {
    use glasnost_rules::sim::LocomotionClass;
    let stale = w.rules().quirks.stale_suspended_revert; // spec: S-0215 Edge cases (quirks.stale_suspended_revert)
    let Some(b) = w.obj(building).cloned() else {
        return;
    };
    let Some(owner) = b.owner else { return };
    let Some(free) = super::production::free_unit(w, b.type_index) else {
        return;
    };
    let player = w
        .houses
        .get(usize::from(owner.0))
        .is_some_and(|h| h.role == HouseRole::Player);
    if player
        && let Some(sp) = b.econ.start_price
        && sp != 0
        && sp <= super::sell::adjusted_cost(w, b.type_index)
    {
        // spec: S-0402 claim 18 (a produced building of a player house whose start price is not 0 and not above the adjusted cost C gets none)
        return;
    }
    let Some(zone) = w.rules().unit(free.kind, free.index).map(|u| u.zone) else {
        return;
    };
    // spec: S-0402 claim 18 (a new vehicle of the free unit's type, owned by the building's owner; the object creation draw)
    let unit = crate::objects::create(w, free.kind, free.index, Some(owner));
    let tl = b.pos.cell();
    let refc = w.reference_point(building).cell();
    // spec: S-0402 claim 18 and Constants (the first place: the cell in step direction 4, map south, of the reference point's cell)
    let (dx, dy) = crate::regions::STEPS[4];
    let first = CellXY::new(refc.x + dx, refc.y + dy);
    let reference = crate::occupancy::reference_level(w, unit);
    let accepts = |w: &World, c: CellXY| {
        // spec: S-0402 claim 18 (a place accepts the unit when its entry outcome there is clear, S-0209)
        w.cells.cell(c).is_some()
            && crate::occupancy::vehicle_outcome(w, unit, c, reference)
                == crate::occupancy::Outcome::Clear
    };
    crate::regions::ensure(w);
    let search = |w: &World, no_overlay: bool| {
        let settings = crate::path::NearbySettings {
            // spec: S-0402 claim 18 and Constants (locomotion class wheel, whatever the free unit's own; its type's movement zone)
            class: Some(LocomotionClass::Wheel),
            zone,
            // spec: S-0402 claim 18 (the reach region of the centre cell for that zone, read for the ground; never in deck mode; the height condition; bridge cells never qualify)
            reach: Some(crate::regions::reach_read(w, tl, zone, false)),
            deck: false,
            height: true,
            // spec: S-0402 claim 18 (the reference cell: the top-left cell)
            reference: Some(tl),
            no_bridges: true,
            no_overlay,
        };
        // spec: S-0402 claim 18 (the centre: the building's top-left cell)
        crate::path::nearby(w, tl, crate::path::NearbyUse::Settings(settings))
    };
    // spec: S-0402 claim 18 and Constants (facing 49152 at the first place, 40960 at a searched place)
    let mut place = accepts(w, first).then_some((first, 49152u16));
    if place.is_none() {
        // spec: S-0402 claim 18 (the second place: the search with the no-overlay condition)
        place = search(w, true)
            .filter(|&c| accepts(w, c))
            .map(|c| (c, 40960));
    }
    if place.is_none() {
        // spec: S-0402 claim 18 (the third place: the same search without the overlay condition)
        place = search(w, false)
            .filter(|&c| accepts(w, c))
            .map(|c| (c, 40960));
    }
    let Some((cell, facing)) = place else {
        // spec: S-0402 claim 18 (no place accepts it: the unit is removed again and the owner receives the refund with the percentage taken as 1)
        crate::objects::remove(w, unit);
        let refund = super::sell::refund_whole(w, owner, free);
        if let Some(h) = w.houses.get_mut(usize::from(owner.0)) {
            h.credits = h.credits.wrapping_add(refund);
        }
        return;
    };
    let p = crate::world::Pos::centre(cell, 0);
    let z = w.surface_at(p.x, p.y);
    crate::objects::place(w, unit, crate::world::Pos::new(p.x, p.y, z), facing);
    let t = w.tick;
    if let Some(ob) = w.obj_mut(unit) {
        // spec: S-0402 claim 18 (a placed free unit queues harvest with an immediate start, whatever its type)
        ob.beh.queue_and_start_at_once(Behaviour::Harvest, t, stale);
    }
}

/// The purifier count *n*ₚ of a house (S-0402 Terms, claim 12): its
/// ore-purifier buildings outside their construction state, plus the
/// virtual purifiers of a house that is not human-controlled (claim 12).
pub fn purifier_count(w: &World, house: HouseId) -> i64 {
    let r = w.rules();
    let real = w
        .objects
        .values()
        .filter(|b| {
            b.kind == Kind::Building
                && b.owner == Some(house)
                && b.on_map
                && b.health > 0
                // spec: S-0406 claim 17 (the queued construction is seen while none is current)
                && super::placement::seen_behaviour(b) != Some(Behaviour::Construction)
                && r.econ
                    .harvest
                    .buildings
                    .get(b.type_index as usize)
                    .is_some_and(|x| x.purifier)
        })
        .count() as i64;
    let h = &w.houses[usize::from(house.0)];
    let virtual_ = if h.human {
        // spec: S-0402 claim 12 (virtual purifiers: never for a player house of our engine, AI houses included)
        0
    } else {
        // spec: S-0402 claim 12 (the neutral and special houses: the list entry their difficulty index selects, index 1 in every skirmish; a missing list or entry counts 0) and glossary (a test-only computer-controlled player house by its oracle difficulty index)
        usize::try_from(h.econ.setup.difficulty)
            .ok()
            .and_then(|i| r.econ.harvest.virtual_purifiers.get(i))
            .map_or(0, |&v| i64::from(v))
    };
    real + virtual_
}

/// The main payment of F3: trunc(RZ₅₃(RZ₅₃(RZ₅₃(*V* × *M*) × *a*) + *B*)).
fn paid(v: i32, mult: F64, a: F64, balance: i32, mode: Rounding) -> i32 {
    // spec: S-0402 F3 (B′ = trunc(RZ₅₃(RZ₅₃(RZ₅₃(V × M) × a) + B)))
    F64::from_i64(i64::from(v), mode)
        .mul(mult, mode)
        .mul(a, mode)
        .add(F64::from_i64(i64::from(balance), mode), mode)
        .to_i32_common()
}

/// A payment for `amount` bales of tiberium `tib` to the owner of the
/// refinery `refinery` (S-0402 claim 12, F3): the main payment, then the
/// purifier bonus when *x* > 0.
pub fn pay(w: &mut World, refinery: ObjectId, tib: usize, amount: i32) {
    let Some(owner) = w.obj(refinery).and_then(|b| b.owner) else {
        return;
    };
    let r = w.rules_arc();
    let m = r.mode;
    let v = r.econ.resource.tiberiums.get(tib).map_or(0, |t| t.value);
    let country = w.houses[usize::from(owner.0)].country as usize;
    let mult = r
        .econ
        .harvest
        .income
        .get(country)
        .copied()
        .unwrap_or_else(|| F64::from_i64(1, m));
    // spec: S-0402 F3 (a, a whole number held as a single)
    let a = F64::from_i64(i64::from(amount), m).to_f32(m).to_f64();
    let np = purifier_count(w, owner);
    // spec: S-0414 claim 22 (the incoming resource amount a adds to the running score)
    super::stats::resource(w, owner, a);
    let house = &mut w.houses[usize::from(owner.0)];
    // spec: S-0402 claim 12 (credits go straight into the balance; no storage caps them)
    house.credits = paid(v, mult, a, house.credits, m);
    // spec: S-0402 F3 (x = RZ₂₄(RZ₅₃(nₚ × b) × a))
    let x = F64::from_i64(np, m)
        .mul(r.econ.harvest.purifier_bonus, m)
        .mul(a, m)
        .to_f32(m)
        .to_f64();
    if x.gt(F64::ZERO) {
        // spec: S-0402 F3 (when x > 0, a second payment with x in place of a)
        house.credits = paid(v, mult, x, house.credits, m);
        // spec: S-0414 claim 22 (a purifier bonus amount is a separate score update)
        super::stats::resource(w, owner, x);
    }
}

/// The guard rules of S-0402 that queue *harvest* again for a harvester in
/// *guard* (claim 13 for the neutral and special houses, claim 17 for a
/// chrono miner of a player house). Returns whether one applied.
pub fn guard_rule(w: &mut World, id: ObjectId) -> bool {
    let Some(u) = unit(w, id).cloned() else {
        return false;
    };
    // spec: S-0402 claim 14 and Import mapping (Harvester: claim 13's guard rows apply to a slave miner only when its type has the harvester flag, a field separate from the enslaves field; the weeder's row is a later revision's)
    if !u.harvester {
        return false;
    }
    let Some(owner) = w.obj(id).and_then(|o| o.owner) else {
        return false;
    };
    let h = &w.houses[usize::from(owner.0)];
    let applies = if !h.human {
        // spec: S-0402 claim 13 (the computer-controlled rule, kept by the neutral and special houses and a test-only computer-controlled player house, glossary: a refinery of its dock types and no no-field record)
        owns_dock(w, id) && !h.econ.harvest.no_field
    } else if u.teleporter {
        // spec: S-0402 claim 17 (a chrono miner: a neighbouring building of its owner with the refinery flag, or full and still moving; every player house)
        let Some(c) = w.obj(id).map(|o| o.pos.cell()) else {
            return false;
        };
        let next_to = crate::regions::STEPS.iter().any(|&(dx, dy)| {
            let n = CellXY::new(c.x + dx, c.y + dy);
            w.cells.list(n, Layer::Ground).iter().any(|&b| {
                w.obj(b).is_some_and(|x| x.owner == Some(owner))
                    && building(w, b).is_some_and(|x| x.refinery)
            })
        });
        next_to || (full(w, id) && crate::occupancy::in_progress(w, id))
    } else {
        false
    };
    if applies {
        // spec: S-0402 claims 13 and 17 (queues harvest again; a row 2 case of S-0215 claim 24's vehicle table)
        if let Some(ob) = w.obj_mut(id) {
            ob.beh.queue(Some(Behaviour::Harvest));
        }
    }
    applies
}

/// Whether a vehicle is a slave miner (S-0402 claim 14; S-0215's slave
/// master).
pub fn is_slave_miner(w: &World, id: ObjectId) -> bool {
    unit(w, id).is_some_and(|u| u.slave_miner)
}

/// The wake-up rule of S-0402 claim 14, given its inputs: the kick delay
/// `k`, the start tick `s_m` of the miner's current behaviour, the tick
/// `t`, whether the group is at rest, whether the owner is
/// computer-controlled, whether the miner's cell has land type tiberium,
/// the tick `s_g` the group last came to rest and whether the slave-miner
/// short-scan search finds a cell (asked only when needed).
#[allow(clippy::too_many_arguments)]
pub fn wake_rule(
    k: i32,
    s_m: i32,
    t: i32,
    at_rest: bool,
    computer: bool,
    on_ore: bool,
    s_g: i32,
    wraps: bool,
    scan: impl FnOnce() -> bool,
) -> bool {
    // spec: S-0402 F5 (k + s < t, signed 32-bit sums that wrap; quirks.slave_kick_delay_wraps off: exact)
    let before = |s: i32| {
        if wraps {
            k.wrapping_add(s) < t
        } else {
            i64::from(k) + i64::from(s) < i64::from(t)
        }
    };
    // spec: S-0402 claim 14 (k + s_m < t, the group at rest, and one of: computer-controlled, on tiberium land, or k + s_g < t with a short-scan cell)
    before(s_m) && at_rest && (computer || on_ore || (before(s_g) && scan()))
}

/// Whether S-0402 claim 14's wake-up rule holds for a slave master in play
/// ([`wake_rule`] is the rule itself).
pub fn slave_wake_rule_holds(w: &mut World, id: ObjectId) -> bool {
    let Some(g) = w.obj(id).and_then(|o| o.econ.group.clone()) else {
        return false;
    };
    let Some(o) = w.obj(id).cloned() else {
        return false;
    };
    let r = w.rules_arc();
    // spec: S-0402 claim 14 (the group at rest: no slave sent out since the group last came together)
    let at_rest = g.state == super::slaves::GroupState::AtRest;
    // spec: S-0402 claim 14 (who takes the owner alternative, Q-1990: in our engine the neutral and special houses and a test-only computer-controlled player house; a person's house and an AI house take the human-controlled alternatives, glossary *AI house*)
    let computer = o.owner.is_some_and(|h| !w.houses[usize::from(h.0)].human);
    let here = o.pos.cell();
    let on_ore = w.cells.cell(here).is_some_and(|c| c.land == land::TIBERIUM);
    let range = r.econ.harvest.slave_short_scan;
    wake_rule(
        r.econ.harvest.kick_delay,
        o.beh.start_tick,
        w.tick,
        at_rest,
        computer,
        on_ore,
        // spec: S-0402 claim 14 (s_g, the group's last-rest tick; initially 0, claim 21)
        g.last_rest,
        r.quirks.slave_kick_delay_wraps,
        // spec: S-0402 claim 14 (the short-scan test: the ring search from the miner with the slave-miner short scan range)
        || ring_search(w, id, here, range).is_some(),
    )
}

/// Whether a vehicle's *unload* follows the harvesting rules: rank 3 of
/// S-0415 claim 1, after the transport and deploy-fire ranks.
pub fn unload_is_harvesting(w: &World, id: ObjectId) -> bool {
    let Some(o) = w.obj(id) else { return false };
    let Some(u) = w.rules().unit(o.kind, o.type_index) else {
        return false;
    };
    // spec: S-0415 claim 1 ranks 1 and 2 (a passenger capacity, or deploy fire without the simple-deployer flag, take precedence)
    if u.passengers > 0 || (u.combat.deploy_fire && !u.simple_deployer) {
        return false;
    }
    // spec: S-0415 claim 1 rank 3 (the harvester flag: the harvesting rules of S-0402)
    is_harvester(w, id)
}
