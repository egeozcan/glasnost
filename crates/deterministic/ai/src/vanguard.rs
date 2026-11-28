//! The vanguard script (#449, registry id 3): a rules-adaptive, fair-play AI
//! script designed to beat both `default` and `siege` in tournaments across
//! synthetic and stock/mod rulesets (`design: own AI (Vanguard)`).
//!
//! **What it does.**
//! - **Rules-adaptive economy:** detects at game start whether refineries bring
//!   a free harvester (`production::free_unit`) and whether extra factories
//!   accelerate production (`0 < multiple_factory < 1`). On free-harvester
//!   rulesets it builds refineries on the yard early so the war factory trains
//!   tanks sooner; on paid-harvester rulesets it skips redundant refineries and
//!   redundant factories when `multiple_factory` gives no speedup. Drops
//!   superweapon buildings (normal/hard), drops navy and unarmed land ferries
//!   when every enemy start is reached by land, gates non-essential tech climb
//!   behind a frontline army floor, pauses extra economy while outmatched
//!   (`wary`), repairs damaged buildings with credit hysteresis, and uses the
//!   parallel defence line when armed defences are buildable and needed.
//! - **Combat unit scoring & infantry throttling:** scores vehicle and aircraft
//!   candidates on free lines (`line.item.is_none()`, never breaking
//!   `Hold::Player` resumes) from `SimRules` (handling standard, gattling and
//!   spawner/rocket weapons, warhead versus tables, splash, range, speed, and
//!   anti-air needs), switching from credit efficiency to build-time throughput
//!   when rich; throttles barracks spend once a war factory stands so infantry
//!   never starve tank production.
//! - **Active scouting & smart aircraft targeting:** sends one fast/cheap unit
//!   to explore the nearest unexplored enemy start for live vision (fog of war
//!   is off in skirmish), and aims idle rearmed aircraft at high-ROI targets
//!   (harvesters, damaged high-value vehicles, key base buildings).
//! - **Tactical army controller & sim-safe micro:** rallies non-guard units at
//!   a forward staging cell, launches decisive strikes based on ground-only
//!   force ratios with local-threat recall, applies in-range-only focus fire
//!   with overkill splitting and switch hysteresis, pulls back actively
//!   targeted wounded high-value vehicles behind healthy allies or to repair
//!   depots, kites reloading standoff artillery, guards active micro units from
//!   `Brain::defend` on every tick, and spreads units across unseen grid cells
//!   in the endgame.
//!
//! **Fair play & determinism.** Reads only `Script::seen_enemies` (shroud,
//! cloak, disguise and alliance filtered), `in_view`-filtered occupancy,
//! explored resource cells, map start waypoints, and the house's own state and
//! rules. Pure integer and `F64` arithmetic; zero floats; deterministic
//! `BTreeMap`/`BTreeSet` containers; passes `Brain::filter_orders` and respects
//! `PENDING_LIMIT`.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use glasnost_map::{CellXY, Map};
use glasnost_num::F64;
use glasnost_rules::sim::MovementZone;
use glasnost_sim::approach;
use glasnost_sim::combat::{damage, veterancy, weapon};
use glasnost_sim::econ::{TypeRef, power, production, sell, shroud, tech};
use glasnost_sim::orders::{self, Order};
use glasnost_sim::probe::Probe;
use glasnost_sim::regions::{self, Reach};
use glasnost_sim::rules::{Behaviour, FactoryKind, Kind};
use glasnost_sim::spawn::SlotState;
use glasnost_sim::transport;
use glasnost_sim::world::{AiDifficulty, HouseId, Object, ObjectId, Target, World};
use serde::{Deserialize, Serialize};

use crate::brain::Brain;
use crate::scripted::{self, Plan, Role, Script, all_types, can_attack, dist2, idle};
use crate::scripts::{AiScript, Schedule, View};
use crate::siege::{HELD, gives_superweapon, naval, worth};

/// The vanguard script's registry id.
pub const ID: u8 = 3; // nocite: design: own AI (Vanguard): the registry's next id

/// Cells within which a unit counts as arrived at its goal.
const ARRIVED: i64 = 2; // nocite: design: own AI (Vanguard)

/// Cells within which a moved goal counts as the same goal.
const SAME_GOAL: i64 = 4; // nocite: design: own AI (Vanguard)

/// Cells around an enemy start within which an enemy building means the start
/// still holds an enemy base.
const SITE: i64 = 12; // nocite: design: own AI (Vanguard)

/// Grid spacing of the endgame hunt search.
const SEARCH_STEP: i32 = 6; // nocite: design: own AI (Vanguard)

/// Ticks after which an unseen enemy bordering unexplored shroud is forgotten.
const MEMORY: i32 = 1800; // nocite: design: own AI (Vanguard)

/// Border radius in cells around an enemy's last cell checked for unexplored
/// shroud before purging it from memory.
const SHROUD_BORDER: i32 = 2; // nocite: design: own AI (Vanguard)

/// Maximum factories of each army kind when `multiple_factory` speeds builds.
const MOST_FACTORIES: usize = 3; // nocite: design: own AI (Vanguard)

/// Maximum early infantry trained beside harvesters before vehicles start.
const BESIDE_CAP: usize = 14; // nocite: design: own AI (Vanguard)

/// Credits above which damaged buildings turn repair ON.
const REPAIR_ON_CREDITS: i32 = 600; // nocite: design: own AI (Vanguard)

/// Credits below which active building repair turns OFF (hysteresis).
const REPAIR_OFF_CREDITS: i32 = 300; // nocite: design: own AI (Vanguard)

/// Credit buffer required before building a paid extra refinery on congestion.
const PAID_REF_BUFFER: i32 = 500; // nocite: design: own AI (Vanguard)

/// Harvesters per refinery ratio that triggers an extra paid refinery.
const HARVESTERS_PER_PAID_REF: usize = 3; // nocite: design: own AI (Vanguard)

/// Credit threshold below which infantry production is throttled to feed tanks.
const INFANTRY_THROTTLE_CREDITS: i32 = 1800; // nocite: design: own AI (Vanguard)

/// Standing infantry count past which infantry production throttles for tanks.
const INFANTRY_THROTTLE_MIN: usize = 4; // nocite: design: own AI (Vanguard)

/// Standing vehicles count past which infantry production throttles for tanks.
const VEHICLE_THROTTLE_MIN: usize = 2; // nocite: design: own AI (Vanguard)

/// Credit threshold at which unit scoring shifts from credit efficiency to
/// build-time throughput.
const THROUGHPUT_CREDITS: i32 = 3000; // nocite: design: own AI (Vanguard)

/// Minimum weapon range in cells to classify a unit as standoff artillery.
const ARTILLERY_RANGE_CELLS: i32 = 10; // nocite: design: own AI (Vanguard)

/// Maximum strength for a long-range unit to be capped as a glass cannon.
const GLASS_CANNON_HP: i32 = 350; // nocite: design: own AI (Vanguard)

/// Maximum fraction numerator/denominator (1/4 = 25%) of vehicles for fragile
/// long-range artillery.
const GLASS_CANNON_DEN: usize = 4; // nocite: design: own AI (Vanguard)

/// Forward staging distance from home toward the enemy start, in cells.
const STAGE_DIST: i32 = 8; // nocite: design: own AI (Vanguard)

/// Deadband radius around the staging point so rallied units do not oscillate.
const STAGE_DEADBAND: i64 = 3; // nocite: design: own AI (Vanguard)

/// Local radius (cells) around strike units used for local-threat recall.
const LOCAL_THREAT_RADIUS: i64 = 18; // nocite: design: own AI (Vanguard)

/// Local enemy worth percent above which an engaged strike recalls (150%).
const LOCAL_RECALL_PCT: i64 = 150; // nocite: design: own AI (Vanguard)

/// Minimum cost of a vehicle eligible for wounded pullback micro.
const PULLBACK_MIN_COST: i32 = 600; // nocite: design: own AI (Vanguard)

/// Minimum speed of a vehicle eligible for wounded pullback micro.
const PULLBACK_MIN_SPEED: i32 = 6; // nocite: design: own AI (Vanguard)

/// Health percentage below which a targeted high-value vehicle pulls back.
const PULLBACK_HP_PCT: i64 = 35; // nocite: design: own AI (Vanguard)

/// Healthy ally health percentage needed to screen a retreating vehicle.
const SCREEN_ALLY_HP_PCT: i64 = 55; // nocite: design: own AI (Vanguard)

/// Radius in cells for healthy screening allies around a wounded vehicle.
const SCREEN_RADIUS: i64 = 6; // nocite: design: own AI (Vanguard)

/// Minimum healthy nearby allies required to pull back without a repair depot.
const SCREEN_MIN_ALLIES: usize = 2; // nocite: design: own AI (Vanguard)

/// Distance in cells a wounded vehicle retreats toward home.
const PULLBACK_STEP: i32 = 6; // nocite: design: own AI (Vanguard)

/// Ticks a retreating wounded vehicle is protected from `Brain::defend`.
const PULLBACK_GUARD_TICKS: i32 = 45; // nocite: design: own AI (Vanguard)

/// Distance in cells within which a threat triggers reloading artillery kiting.
const KITE_THREAT_RADIUS: i64 = 7; // nocite: design: own AI (Vanguard)

/// Distance in cells reloading artillery steps backward.
const KITE_STEP: i32 = 5; // nocite: design: own AI (Vanguard)

/// Ticks a kiting artillery unit is protected from `Brain::defend`.
const KITE_GUARD_TICKS: i32 = 30; // nocite: design: own AI (Vanguard)

/// Ticks a scout order is protected from `Brain::defend`.
const SCOUT_GUARD_TICKS: i32 = 120; // nocite: design: own AI (Vanguard)

/// Scale factor for fixed-point integer ratios (1000 = 1.0).
const SCALE: i128 = 1000; // nocite: design: own AI (Vanguard)

/// Lepton width of one cell (S-0204 claim 2).
const LEPTONS_PER_CELL: i32 = 256; // spec: S-0204 claim 2 (a cell is 256 leptons across)

/// Number of armour classes in `WarheadRules::versus`.
const ARMOUR_CLASSES: usize = 11; // nocite: design: own AI (Vanguard): S-0305 eleven armour classes

/// Maximum parallel anti-rush defensive buildings built early.
const MAX_DEFENCES: usize = 2; // nocite: design: own AI (Vanguard)

/// Credits above which an anti-rush defence may be built proactively.
const DEFENCE_RICH: i32 = 2500; // nocite: design: own AI (Vanguard)

/// Percentage scale constant.
const PCT: i64 = 100; // nocite: design: own AI (Vanguard)

/// Tuning parameters per difficulty level (`design: own AI (Vanguard)`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Tuning {
    /// Harvesters kept when refineries do not bring a free harvester.
    pub paid_harvesters: u32,
    /// Harvesters kept when refineries bring a free harvester.
    pub free_harvesters: u32,
    /// Whether superweapon buildings are built.
    pub superweapons: bool,
    /// Whether naval units/buildings are built on land-connected maps.
    pub navy: bool,
    /// Idle army units needed to launch a new strike.
    pub strike: usize,
    /// Idle army units needed to reinforce an active strike.
    pub join: usize,
    /// Home guard units kept nearest home when launching a strike.
    pub guard: usize,
    /// Army acts on one of every `every` think ticks.
    pub every: i32,
    /// Minimum worth of goers for a new strike.
    pub floor: i64,
    /// Required goers worth as percent of known enemy ground army (0: off).
    pub edge: i64,
    /// Global fallback recall percent of known enemy ground army (0: off).
    pub recall: i64,
    /// Ticks without a strike before edge relaxes to 80%.
    pub patience: i32,
    /// Target refinery count on free-harvester rulesets (0: brain default).
    pub free_refineries: usize,
    /// Credits from which extra factories are added when `multiple_factory < 1`.
    pub rich: i32,
    /// Minimum ground army worth before non-essential tech climb proceeds.
    pub tech_gate: i64,
    /// Whether rules-driven vehicle/aircraft scoring runs on free lines.
    pub unit_scoring: bool,
    /// Whether infantry production throttles to avoid starving the war factory.
    pub infantry_throttle: bool,
    /// Whether early infantry train beside harvesters.
    pub beside: bool,
    /// Whether active building repair is enabled.
    pub repair: bool,
    /// Whether an early scout explores enemy starts.
    pub scout: bool,
    /// Whether smart aircraft targeting is enabled.
    pub air_snipe: bool,
    /// Whether forward staging is enabled.
    pub staging: bool,
    /// Whether in-range focus fire is enabled.
    pub focus_fire: bool,
    /// Whether targeted wounded pullback and reload-window artillery kiting run.
    pub advanced_micro: bool,
    /// Whether economy-first targeting and endgame spread hunt run.
    pub hunts: bool,
    /// Whether extra economy pauses while outmatched (`wary`).
    pub wary: bool,
    /// Whether parallel anti-rush defences are built when available.
    pub defences: bool,
}

/// The tuning of difficulty level `level`.
pub fn tuning(level: AiDifficulty) -> Tuning {
    match level {
        // nocite: design: own AI (Vanguard): easy plays slowly with one harvester, small strikes and no micro
        AiDifficulty::Easy => Tuning {
            paid_harvesters: 1,
            free_harvesters: 1,
            superweapons: true,
            navy: true,
            strike: 6,
            join: 5,
            guard: 1,
            every: 3,
            floor: 0,
            edge: 0,
            recall: 0,
            patience: 0,
            free_refineries: 0,
            rich: 0,
            tech_gate: 0,
            unit_scoring: false,
            infantry_throttle: false,
            beside: false,
            repair: false,
            scout: false,
            air_snipe: false,
            staging: false,
            focus_fire: false,
            advanced_micro: false,
            hunts: false,
            wary: false,
            defences: false,
        },
        // nocite: design: own AI (Vanguard): normal uses rules-adaptive economy, vehicle scoring, repair and focus fire
        AiDifficulty::Normal => Tuning {
            paid_harvesters: 2,
            free_harvesters: 2,
            superweapons: false,
            navy: false,
            strike: 10,
            join: 4,
            guard: 2,
            every: 1,
            floor: 3200,
            edge: 100,
            recall: 40,
            patience: 2800,
            free_refineries: 3,
            rich: 3500,
            tech_gate: 2200,
            unit_scoring: true,
            infantry_throttle: true,
            beside: false,
            repair: true,
            scout: false,
            air_snipe: true,
            staging: false,
            focus_fire: true,
            advanced_micro: false,
            hunts: true,
            wary: true,
            defences: false,
        },
        // nocite: design: own AI (Vanguard): hard enables all four pillars with early vehicle interleaving and decisive strike timing
        AiDifficulty::Hard => Tuning {
            paid_harvesters: 3,
            free_harvesters: 2,
            superweapons: false,
            navy: false,
            strike: 8,
            join: 2,
            guard: 2,
            every: 1,
            floor: 3000,
            edge: 115,
            recall: 45,
            patience: 2000,
            free_refineries: 4,
            rich: 3000,
            tech_gate: 3200,
            unit_scoring: true,
            infantry_throttle: true,
            beside: true,
            repair: true,
            scout: false,
            air_snipe: true,
            staging: false,
            focus_fire: true,
            advanced_micro: true,
            hunts: true,
            wary: true,
            defences: true,
        },
    }
}

/// Detects directly from `w.rules()` whether house `house`'s country has a
/// buildable non-slave refinery that grants a free harvester (S-0404 claim 2).
/// Safe to call at tick 0 before the MCV deploys.
pub fn refinery_brings_harvester(w: &World, house: HouseId) -> bool {
    let r = w.rules();
    let Some(country) = w.houses.get(usize::from(house.0)).map(|h| h.country) else {
        return false;
    };
    let house_tech = tech::house_tech_level(w, house);
    (0..r.buildings.len() as u32).any(|index| {
        let t = TypeRef {
            kind: Kind::Building,
            index,
        };
        if scripted::role_of(w, t) != Some(Role::Refinery) || scripted::keeps_slaves(w, t) {
            return false;
        }
        let Some(tt) = r.econ.tech.of(t) else {
            return false;
        };
        if tt.tech_level < 0 || tt.tech_level > house_tech {
            return false;
        }
        if !tt.owners.is_empty() && !tt.owners.contains(&country) {
            return false;
        }
        if tt
            .required
            .as_ref()
            .is_some_and(|req| !req.contains(&country))
        {
            return false;
        }
        if tt
            .forbidden
            .as_ref()
            .is_some_and(|forb| forb.contains(&country))
        {
            return false;
        }
        production::free_unit(w, index)
            .is_some_and(|f| scripted::role_of(w, f) == Some(Role::Harvester))
    })
}

/// Whether `multiple_factory` genuinely shortens build times (S-0403 claim 6:
/// strictly between 0 and 1).
pub fn multiple_factory_helps(w: &World) -> bool {
    let mf = w.rules().econ.production.multiple_factory;
    let one = F64::from_i64(1, w.rules().mode);
    mf.gt(F64::ZERO) && mf.lt(one)
}

/// The brain's plan at level `level` for `house` in `w`: holds the brain's
/// waves so Vanguard owns army control, and sets the harvester plan according to
/// whether refineries bring a free harvester.
pub fn plan_for(w: &World, house: HouseId, level: AiDifficulty) -> Plan {
    let t = tuning(level);
    let harvesters = if refinery_brings_harvester(w, house) {
        t.free_harvesters
    } else {
        t.paid_harvesters
    };
    Plan {
        harvesters,
        wave: HELD,
        ..crate::brain::default_plan()
    }
}

/// Cached sighting data for one enemy object, recorded only while it is in
/// `Script::seen_enemies` so forgotten IDs never touch `w.obj(id)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Sighting {
    /// Object cost worth.
    worth: i64,
    /// Armour class index (0..10).
    armour: u8,
    /// Whether this is a ground combat army unit (excludes buildings & aircraft).
    ground_army: bool,
    /// Whether this is an airborne/aircraft combat threat.
    air_threat: bool,
    /// Whether this unit is crushable infantry.
    crushable: bool,
    /// Last seen cell.
    cell: CellXY,
    /// Tick last seen.
    tick: i32,
}

/// The vanguard's army and intelligence state.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct Army {
    /// Units currently committed to the active strike.
    out: BTreeSet<ObjectId>,
    /// Whether a strike is under way.
    striking: bool,
    /// Number of strikes launched.
    strikes: u32,
    /// Number of strikes recalled.
    recalls: u32,
    /// Tick the last strike ended (0 before first).
    ended: i32,
    /// Remembered enemy combat units, cached at sighting time.
    seen: BTreeMap<ObjectId, Sighting>,
}

/// The vanguard script's state.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Vanguard {
    /// Base brain with waves held.
    brain: Brain,
    /// House view helper (derived per-think queries only).
    eyes: Script,
    /// Army & enemy intelligence state.
    army: Army,
    /// Actors under active Vanguard micro protection (`actor -> expire_tick`)
    /// so `Brain::defend` cannot clobber them on non-think ticks.
    micro_guard: BTreeMap<ObjectId, i32>,
    /// Active early scout exploring enemy start positions.
    scout: Option<ObjectId>,
    /// Cached ruleset fact: whether this house's refinery grants a free harvester.
    free_harv_refinery: bool,
}

impl Army {
    /// Updates remembered enemy combat units strictly from `enemies`
    /// (`Script::seen_enemies`) and purges absent units whose last cell is
    /// explored without bordering unexplored shroud, or whose memory expired.
    /// Never inspects `w.obj(e)` for any ID not in `enemies`.
    fn remember(&mut self, w: &World, house: HouseId, enemies: &[(ObjectId, CellXY)]) -> i64 {
        let now: BTreeSet<ObjectId> = enemies.iter().map(|&(e, _)| e).collect();
        let whole_map = shroud::whole_map_revealed(w, house);
        self.seen.retain(|e, s| {
            if now.contains(e) {
                return true;
            }
            if whole_map || w.tick - s.tick >= MEMORY {
                return false;
            }
            if !shroud::explored(w, house, s.cell) {
                return true;
            }
            // Keep brief memory only if the last cell borders unexplored shroud
            // (reads only our own shroud).
            borders_unexplored(w, house, s.cell)
        });
        for &(e, c) in enemies {
            let Some(o) = w.obj(e) else { continue };
            if o.kind == Kind::Building {
                continue;
            }
            let tr = tech::type_of(o);
            if scripted::role_of(w, tr) != Some(Role::Army) {
                continue;
            }
            let armour = veterancy::type_combat(w, e).map_or(0, |tc| tc.armour) as u8;
            let air_threat = o.kind == Kind::Aircraft || weapon::airborne_by_height(w, e);
            let ground_army = o.kind != Kind::Aircraft;
            let crushable = w
                .rules()
                .unit(o.kind, o.type_index)
                .is_some_and(|u| u.crushable);
            self.seen.insert(
                e,
                Sighting {
                    worth: worth(w, e),
                    armour,
                    ground_army,
                    air_threat,
                    crushable,
                    cell: c,
                    tick: w.tick,
                },
            );
        }
        self.ground_worth()
    }

    /// Worth of remembered enemy ground combat army units.
    fn ground_worth(&self) -> i64 {
        self.seen
            .values()
            .filter(|s| s.ground_army)
            .map(|s| s.worth)
            .sum()
    }

    /// Count of remembered enemy airborne combat threats.
    fn air_count(&self) -> usize {
        self.seen.values().filter(|s| s.air_threat).count()
    }

    /// Whether remembered enemy army includes crushable infantry.
    fn has_crushable(&self) -> bool {
        self.seen.values().any(|s| s.crushable)
    }

    /// Observed enemy ground armor-class weights (by worth).
    fn armour_weights(&self) -> [i64; ARMOUR_CLASSES] {
        let mut weights = [0i64; ARMOUR_CLASSES];
        for s in self.seen.values().filter(|s| s.ground_army) {
            let idx = usize::from(s.armour).min(ARMOUR_CLASSES - 1);
            weights[idx] = weights[idx].saturating_add(s.worth.max(1));
        }
        weights
    }
}

/// Whether cell `c` has an unexplored usable neighbor within [`SHROUD_BORDER`]
/// cells (reads only the house's own shroud).
fn borders_unexplored(w: &World, house: HouseId, c: CellXY) -> bool {
    (-SHROUD_BORDER..=SHROUD_BORDER).any(|dy| {
        (-SHROUD_BORDER..=SHROUD_BORDER).any(|dx| {
            let n = CellXY::new(c.x + dx, c.y + dy);
            w.cells.usable(n) && !shroud::explored(w, house, n)
        })
    })
}

impl Vanguard {
    /// Starts a Vanguard script for `house` at `level`.
    pub fn start(w: &World, map: &Map, house: HouseId, level: AiDifficulty) -> Self {
        let free_harv_refinery = refinery_brings_harvester(w, house);
        let plan = plan_for(w, house, level);
        let starts = crate::skirmish::start_cells(map);
        let mut eyes = Script::new(house, plan.clone());
        eyes.set_starts(w, &starts);
        Self {
            brain: Brain::with_starts(w, &starts, house, plan),
            eyes,
            army: Army::default(),
            micro_guard: BTreeMap::new(),
            scout: None,
            free_harv_refinery,
        }
    }

    /// Strikes launched so far.
    pub fn strikes(&self) -> u32 {
        self.army.strikes
    }

    /// Strikes recalled so far.
    pub fn recalls(&self) -> u32 {
        self.army.recalls
    }

    /// Remembered enemy ground army worth.
    pub fn remembered_ground(&self) -> i64 {
        self.army.ground_worth()
    }

    /// Units currently committed to the strike.
    pub fn out(&self) -> &BTreeSet<ObjectId> {
        &self.army.out
    }

    /// Whether every enemy start lies in the reach region of our first ground
    /// vehicle (every enemy is reachable by land).
    fn by_land(&self, w: &World) -> bool {
        let eyes = &self.eyes;
        let ground = eyes.own(w).iter().find_map(|&id| {
            let o = w.obj(id)?;
            if o.kind != Kind::Vehicle {
                return None;
            }
            let zone = w.rules().unit(o.kind, o.type_index)?.zone;
            if zone == MovementZone::Water {
                return None;
            }
            eyes.reach_region(w, id).map(|r| (zone, r))
        });
        let Some((zone, r)) = ground else {
            return false;
        };
        let starts = eyes.enemy_starts(w);
        !starts.is_empty()
            && starts
                .iter()
                .all(|&c| regions::reach_in(w, c, zone) == Reach::Region(r))
    }

    /// Worth of our living ground combat army units.
    fn own_ground_worth(&self, w: &World) -> i64 {
        self.eyes
            .of_role(w, Role::Army)
            .into_iter()
            .filter(|&id| w.obj(id).is_some_and(|o| o.kind != Kind::Aircraft))
            .map(|id| worth(w, id))
            .sum()
    }

    /// Whether we are currently outmatched by remembered enemy ground forces
    /// (AS6 R3 `wary`: pause extra economy to fund army first).
    fn is_wary(&self, w: &World, t: Tuning) -> bool {
        if !t.wary || t.floor <= 0 {
            return false;
        }
        let enemy = self.army.ground_worth();
        enemy >= t.floor && enemy > self.own_ground_worth(w)
    }

    /// Drops superweapon buildings when disabled by tuning, substituting a
    /// non-superweapon climb building only if the tech gate allows it.
    fn no_superweapons(&self, w: &World, tech_open: bool, out: &mut Vec<Order>) {
        let before = out.len();
        out.retain(|o| !matches!(o, Order::Build { what } if gives_superweapon(w, *what)));
        if out.len() == before || !tech_open {
            return;
        }
        let has_building = out
            .iter()
            .any(|o| matches!(o, Order::Build { what } if what.kind == Kind::Building));
        if let Some(c) = self.eyes.climb(w)
            && !has_building
            && !gives_superweapon(w, c)
            && !naval(w, c)
            && !self.eyes.busy(w, c)
        {
            out.push(Order::Build { what: c });
        }
    }

    /// Drops naval buildings and water units when every enemy start is reached
    /// by land.
    fn no_navy(&self, w: &World, by_land: bool, tech_open: bool, out: &mut Vec<Order>) {
        if !by_land {
            return;
        }
        let is_nav = |o: &Order| matches!(o, Order::Build { what } if naval(w, *what));
        if !out.iter().any(is_nav) {
            return;
        }
        let dropped_building = out.iter().any(|o| {
            matches!(o, Order::Build { what } if what.kind == Kind::Building && naval(w, *what))
        });
        out.retain(|o| !is_nav(o));
        if !dropped_building || !tech_open {
            return;
        }
        let has_building = out
            .iter()
            .any(|o| matches!(o, Order::Build { what } if what.kind == Kind::Building));
        if let Some(c) = self.eyes.climb(w)
            && !has_building
            && !naval(w, c)
            && !gives_superweapon(w, c)
            && !self.eyes.busy(w, c)
        {
            out.push(Order::Build { what: c });
        }
    }

    /// Drops unarmed ground ferry build orders and boarding orders onto unarmed
    /// vehicle transports when every enemy start is reached by land (never
    /// touching repair depots, bunkers, garrisons, or water crossings).
    fn no_land_ferries(&self, w: &World, by_land: bool, out: &mut Vec<Order>) {
        if !by_land {
            return;
        }
        let house = self.eyes.house;
        out.retain(|o| match o {
            Order::Build { what } if what.kind == Kind::Vehicle => {
                let unarmed_transport = w.rules().unit(what.kind, what.index).is_some_and(|u| {
                    u.passengers > 0
                        && !scripted::armed(u)
                        && scripted::role_of(w, *what).is_none()
                        && !scripted::deploys_into_yard(w, what.kind, what.index)
                });
                // Drop even when Brain::ferry appends to a busy vehicle queue,
                // while never dropping a Hold::Player resume.
                if unarmed_transport && !is_held_resume(w, house, *what) {
                    return false;
                }
                true
            }
            Order::Behaviour {
                behaviour: Behaviour::Enter,
                dest: Some(Target::Object(target)),
                ..
            } => {
                let is_ferry = w.obj(*target).is_some_and(|to| {
                    transport::vehicle_transport(w, *target)
                        && scripted::role_of(w, tech::type_of(to)).is_none()
                });
                !is_ferry
            }
            _ => true,
        });
    }

    /// Gates non-essential tech-climb buildings (`RADR`, `LAB`, `HPAD`, etc.)
    /// behind frontline army readiness or surplus credits, and pauses extra
    /// refineries while wary, only on free lines and never touching core base
    /// roles or defences.
    fn gate_tech_climb(&self, w: &World, tech_open: bool, wary: bool, out: &mut Vec<Order>) {
        let house = self.eyes.house;
        let has_ref = self.eyes.has_refinery(w);
        out.retain(|o| match o {
            Order::Build { what } if what.kind == Kind::Building => {
                if !line_is_free(w, house, *what) {
                    return true;
                }
                let role = scripted::role_of(w, *what);
                if wary && has_ref && role == Some(Role::Refinery) {
                    return false;
                }
                if tech_open {
                    return true;
                }
                let is_core = matches!(
                    role,
                    Some(Role::Yard | Role::Power | Role::Refinery | Role::Factory(_))
                );
                let is_defence =
                    production::line_kind(w, *what) == Some(production::LineKind::Defence);
                is_core || is_defence
            }
            _ => true,
        });
    }

    /// Rules-adaptive extra refineries:
    /// - Free-harvester rulesets: build up to `t.free_refineries` on the yard
    ///   once a war factory (or harvesters) stands and we are not `wary`.
    /// - Paid-harvester rulesets (`fixture`): only build an extra refinery when
    ///   docks are congested (`harvesters >= refineries * 3`) with spare cash,
    ///   leaving `Script::field_refinery` to expand when home ore runs low.
    fn manage_refineries(&self, w: &World, t: Tuning, wary: bool, out: &mut Vec<Order>) {
        let eyes = &self.eyes;
        let house = eyes.house;
        if eyes.selling() || eyes.slave_miners(w) || wary || base_ordered(w, out) {
            return;
        }
        let refineries = eyes.of_role(w, Role::Refinery).len();
        if refineries == 0 {
            return;
        }
        let harvesters = eyes.of_role(w, Role::Harvester).len();
        let has_wf = !eyes.of_role(w, Role::Factory(FactoryKind::Vehicle)).is_empty();
        let credits = w.houses[usize::from(house.0)].credits;

        let want = if self.free_harv_refinery {
            t.free_refineries > 0
                && refineries < t.free_refineries
                && (has_wf || eyes.harvesters_kept(w))
        } else {
            eyes.harvesters_kept(w) && harvesters >= refineries * HARVESTERS_PER_PAID_REF
        };
        if !want {
            return;
        }
        let types: Vec<TypeRef> = eyes
            .candidates(w, Role::Refinery)
            .into_iter()
            .filter(|&ty| !scripted::keeps_slaves(w, ty))
            .collect();
        if types.iter().any(|&ty| eyes.busy(w, ty)) {
            return;
        }
        let Some(ref_type) = types.into_iter().find(|&ty| {
            let affordable = self.free_harv_refinery
                || credits >= production::price(w, house, ty).saturating_add(PAID_REF_BUFFER);
            affordable && eyes.place_refinery(w, ty).is_some()
        }) else {
            return;
        };
        let pick = if crate::power::short_after(w, house, ref_type) {
            match eyes.plant_type(w) {
                Some(p) => p,
                None => return,
            }
        } else {
            ref_type
        };
        if !eyes.busy(w, pick) && line_is_free(w, house, pick) {
            out.retain(|o| !matches!(o, Order::Build { what } if what.kind == Kind::Building && line_is_free(w, house, *what)));
            out.push(Order::Build { what: pick });
        }
    }

    /// Adds extra factories ONLY when `0 < multiple_factory < 1`, economy
    /// stands, not `wary`, and credits >= `t.rich`. Prioritizes vehicle
    /// factories up to [`MOST_FACTORIES`] first before adding barracks.
    fn manage_factories(&self, w: &World, t: Tuning, wary: bool, out: &mut Vec<Order>) {
        if t.rich <= 0 || wary || !multiple_factory_helps(w) || base_ordered(w, out) {
            return;
        }
        let eyes = &self.eyes;
        let house = eyes.house;
        if eyes.selling() || !eyes.harvesters_kept(w) {
            return;
        }
        let refs = eyes.of_role(w, Role::Refinery).len();
        // nocite: design: own AI (Vanguard): build at least 2 refineries before extra factories
        if !eyes.slave_miners(w) && refs < 2 {
            return;
        }
        let credits = w.houses[usize::from(house.0)].credits;
        if credits < t.rich {
            return;
        }
        let owned = |k: FactoryKind| eyes.of_role(w, Role::Factory(k)).len();
        let (veh, inf) = (owned(FactoryKind::Vehicle), owned(FactoryKind::Infantry));
        // Prioritize vehicle factories first up to MOST_FACTORIES, then infantry.
        let kinds = [(FactoryKind::Vehicle, veh), (FactoryKind::Infantry, inf)];
        for (k, n) in kinds {
            if n == 0 || n >= MOST_FACTORIES {
                continue;
            }
            let Some(fac) = eyes.candidates(w, Role::Factory(k)).into_iter().find(|&ty| {
                production::price(w, house, ty) <= credits && eyes.place_cell(w, ty).is_some()
            }) else {
                continue;
            };
            let pick = if crate::power::short_after(w, house, fac) {
                match eyes.plant_type(w) {
                    Some(p) => p,
                    None => return,
                }
            } else {
                fac
            };
            if !eyes.busy(w, pick) && line_is_free(w, house, pick) {
                out.retain(|o| !matches!(o, Order::Build { what } if what.kind == Kind::Building && line_is_free(w, house, *what)));
                out.push(Order::Build { what: pick });
            }
            return;
        }
    }

    /// Trains early infantry beside initial harvesters (capped at [`BESIDE_CAP`])
    /// only when spare cash covers both the next harvester and the soldier.
    fn beside_soldiers(&self, w: &World, out: &mut Vec<Order>) {
        let eyes = &self.eyes;
        let house = eyes.house;
        if eyes.selling() || eyes.harvesters_kept(w) {
            return;
        }
        let inf_count = eyes
            .of_role(w, Role::Army)
            .into_iter()
            .filter(|&id| w.obj(id).is_some_and(|o| o.kind == Kind::Infantry))
            .count();
        if inf_count >= BESIDE_CAP {
            return;
        }
        if out
            .iter()
            .any(|o| matches!(o, Order::Build { what } if what.kind == Kind::Infantry))
        {
            return;
        }
        let price = |ty: TypeRef| i64::from(production::price(w, house, ty));
        let reserve = eyes
            .candidates(w, Role::Harvester)
            .first()
            .copied()
            .or_else(|| {
                eyes.candidates(w, Role::Factory(FactoryKind::Vehicle))
                    .first()
                    .copied()
            })
            .map_or(0, price);
        let owed: i64 = w
            .econ
            .lines
            .lines
            .iter()
            .filter(|l| l.house == house)
            .filter_map(|l| l.item.as_ref())
            .filter(|i| !i.done() && i.hold == production::Hold::None)
            .map(|i| i64::from(i.owed))
            .sum();
        let ordered: i64 = out
            .iter()
            .filter_map(|o| match o {
                Order::Build { what } => Some(price(*what)),
                _ => None,
            })
            .sum();
        let spare = i64::from(w.houses[usize::from(house.0)].credits) - owed - ordered;
        let own = eyes.own(w);
        let living = |ty: TypeRef| {
            own.iter()
                .filter(|&&id| w.obj(id).is_some_and(|o| tech::type_of(o) == ty))
                .count()
        };
        let pick = eyes
            .candidates(w, Role::Army)
            .into_iter()
            .filter(|&ty| {
                ty.kind == Kind::Infantry
                    && !scripted::spent_alone(w, ty)
                    && !crate::lifter::lifter_type(w, ty)
                    && !crate::drain::drainer_type(w, ty)
                    && !eyes.busy(w, ty)
                    && line_is_free(w, house, ty)
                    && crate::startable(w, house, ty)
            })
            .min_by_key(|&ty| {
                (
                    living(ty),
                    tech::ever_built(w, house, ty),
                    production::cost_field(w, ty),
                    ty.index,
                )
            });
        if let Some(ty) = pick
            && spare >= reserve + price(ty)
        {
            out.push(Order::Build { what: ty });
        }
    }

    /// Parallel `LineKind::Defence` anti-rush turrets when buildable, lit when
    /// placed, power-safe, placeable, and either threatened/wary or rich.
    fn manage_defences(&self, w: &World, wary: bool, out: &mut Vec<Order>) {
        let eyes = &self.eyes;
        let house = eyes.house;
        if eyes.selling() || !eyes.has_refinery(w) {
            return;
        }
        let credits = w.houses[usize::from(house.0)].credits;
        let threatened = wary || credits >= DEFENCE_RICH;
        if !threatened {
            return;
        }
        let already_ordered = out.iter().any(|o| {
            matches!(o, Order::Build { what } if production::line_kind(w, *what) == Some(production::LineKind::Defence))
        });
        if already_ordered {
            return;
        }
        let existing = eyes
            .own(w)
            .into_iter()
            .filter(|&id| {
                w.obj(id).is_some_and(|o| {
                    o.kind == Kind::Building
                        && production::line_kind(w, tech::type_of(o))
                            == Some(production::LineKind::Defence)
                        && weapon::armed(w, id)
                })
            })
            .count();
        if existing >= MAX_DEFENCES {
            return;
        }
        let pick = all_types(w)
            .into_iter()
            .filter(|&ty| {
                ty.kind == Kind::Building
                    && production::line_kind(w, ty) == Some(production::LineKind::Defence)
                    && !gives_superweapon(w, ty)
                    && w.rules()
                        .buildings
                        .get(ty.index as usize)
                        .is_some_and(|b| b.combat.slots[0].is_some())
                    && crate::antiair::lit_when_placed(w, house, ty)
                    && crate::startable(w, house, ty)
                    && !eyes.busy(w, ty)
                    && line_is_free(w, house, ty)
                    && !crate::power::short_after(w, house, ty)
                    && eyes.place_cell(w, ty).is_some()
            })
            .min_by_key(|&ty| (production::cost_field(w, ty), ty.index));
        if let Some(ty) = pick {
            out.push(Order::Build { what: ty });
        }
    }

    /// Active building repair with strict safety guards and credit hysteresis.
    fn manage_repair(&self, w: &World, out: &mut Vec<Order>) {
        let eyes = &self.eyes;
        let house = eyes.house;
        if eyes.selling() {
            return;
        }
        let Some(h) = w.houses.get(usize::from(house.0)) else {
            return;
        };
        let credits = h.credits;
        let auto_repairs = h.skill >= w.rules().combat.general.iq_repair_sell
            && credits >= w.rules().combat.general.credit_reserve;
        let mut toggled = BTreeSet::new();
        for id in eyes.own(w) {
            let Some(o) = w.obj(id) else { continue };
            if o.kind != Kind::Building || toggled.contains(&id) {
                continue;
            }
            let in_progress = o.econ.construction.building_up
                || matches!(
                    glasnost_sim::econ::placement::seen_behaviour(o),
                    Some(Behaviour::Construction | Behaviour::Selling)
                );
            let blocked = glasnost_sim::temporal::building_erased(w, id)
                || glasnost_sim::teleport::phasing_in(w, id);
            if in_progress || blocked {
                continue;
            }
            if !o.econ.building.repairing {
                if credits >= REPAIR_ON_CREDITS && sell::repairable(w, id) {
                    out.push(Order::Repair { actor: id });
                    toggled.insert(id);
                }
            } else if credits < REPAIR_OFF_CREDITS && !auto_repairs {
                out.push(Order::Repair { actor: id });
                toggled.insert(id);
            }
        }
    }

    /// Pillar 2: replaces round-robin Vehicle/Aircraft army build orders on
    /// free lines (`line.item.is_none()`) with the highest combat-value unit,
    /// interleaves frontline combat vehicles after the first two harvesters,
    /// caps landable aircraft to owned reload docks, and throttles Infantry on
    /// free lines when a War Factory is active and credits are tight.
    fn optimize_production(
        &self,
        w: &World,
        t: Tuning,
        by_land: bool,
        wary: bool,
        out: &mut Vec<Order>,
    ) {
        let eyes = &self.eyes;
        let house = eyes.house;
        let credits = w.houses[usize::from(house.0)].credits;
        let armour_weights = self.army.armour_weights();
        let enemy_air = self.army.air_count();
        let enemy_crushable = self.army.has_crushable();

        // Count own AA army units, vehicles, infantry, harvesters and landable aircraft.
        let own_army = eyes.of_role(w, Role::Army);
        let mut own_aa: usize = 0;
        let mut own_vehicles: usize = 0;
        let mut own_glass_artillery: usize = 0;
        let mut own_infantry: usize = 0;
        let mut own_landable_air: usize = 0;
        for &id in &own_army {
            let Some(o) = w.obj(id) else { continue };
            let tr = tech::type_of(o);
            if can_hit_any_air(w, tr) {
                own_aa += 1;
            }
            match o.kind {
                Kind::Vehicle => {
                    own_vehicles += 1;
                    if is_glass_artillery(w, tr) {
                        own_glass_artillery += 1;
                    }
                }
                Kind::Infantry => own_infantry += 1,
                Kind::Aircraft
                    if w.rules()
                        .unit(Kind::Aircraft, o.type_index)
                        .is_some_and(|u| u.raw.econ.landable.unwrap_or(false)) =>
                {
                    own_landable_air += 1;
                }
                _ => {}
            }
        }
        let own_harvesters = eyes.of_role(w, Role::Harvester).len();
        let total_air_docks: usize = eyes
            .of_role(w, Role::Factory(FactoryKind::Aircraft))
            .into_iter()
            .filter_map(|id| {
                let o = w.obj(id)?;
                let b = w.rules().buildings.get(o.type_index as usize)?;
                Some(b.raw.econ.number_of_docks.unwrap_or(1).max(1) as usize)
            })
            .sum();
        // nocite: design: own AI (Vanguard): need AA when enemy air * 2 exceeds own AA
        let need_aa = enemy_air > 0 && own_aa < enemy_air * 2;
        let has_wf = !eyes.of_role(w, Role::Factory(FactoryKind::Vehicle)).is_empty();

        // Compare best buildable vehicle score vs best infantry score to decide
        // whether infantry should be throttled when credits are tight.
        let pick_best_army = |kind: Kind, line_k: Option<production::LineKind>| {
            eyes.candidates(w, Role::Army)
                .into_iter()
                .filter(|&ty| {
                    ty.kind == kind
                        && production::line_kind(w, ty) == line_k
                        && (!by_land || !naval(w, ty))
                        && !scripted::spent_alone(w, ty)
                        && !crate::lifter::lifter_type(w, ty)
                        && !crate::drain::drainer_type(w, ty)
                        && !eyes.busy(w, ty)
                        && crate::startable(w, house, ty)
                        && !(kind == Kind::Vehicle
                            && is_glass_artillery(w, ty)
                            && own_glass_artillery * GLASS_CANNON_DEN >= own_vehicles.max(1))
                })
                .max_by_key(|&ty| {
                    (
                        score_unit_type(
                            w,
                            house,
                            ty,
                            credits,
                            &armour_weights,
                            need_aa,
                            enemy_crushable,
                        ),
                        std::cmp::Reverse(production::cost_field(w, ty)),
                        std::cmp::Reverse(ty.index),
                    )
                })
        };
        let best_veh_score = eyes
            .candidates(w, Role::Army)
            .into_iter()
            .filter(|&ty| {
                ty.kind == Kind::Vehicle
                    && (!by_land || !naval(w, ty))
                    && !scripted::spent_alone(w, ty)
                    && !crate::lifter::lifter_type(w, ty)
                    && !crate::drain::drainer_type(w, ty)
                    && crate::startable(w, house, ty)
            })
            .map(|ty| {
                score_unit_type(
                    w,
                    house,
                    ty,
                    credits,
                    &armour_weights,
                    need_aa,
                    enemy_crushable,
                )
            })
            .max()
            .unwrap_or(0);
        let best_inf_score = eyes
            .candidates(w, Role::Army)
            .into_iter()
            .filter(|&ty| {
                ty.kind == Kind::Infantry
                    && !scripted::spent_alone(w, ty)
                    && crate::startable(w, house, ty)
            })
            .map(|ty| {
                score_unit_type(
                    w,
                    house,
                    ty,
                    credits,
                    &armour_weights,
                    need_aa,
                    enemy_crushable,
                )
            })
            .max()
            .unwrap_or(0);

        let throttle_inf = t.infantry_throttle
            && has_wf
            && (eyes.harvesters_kept(w) || own_vehicles >= VEHICLE_THROTTLE_MIN)
            && credits < INFANTRY_THROTTLE_CREDITS
            && (own_infantry >= INFANTRY_THROTTLE_MIN || own_vehicles >= VEHICLE_THROTTLE_MIN)
            && best_veh_score > best_inf_score;

        let mut i = 0;
        while i < out.len() {
            let Order::Build { what } = out[i] else {
                i += 1;
                continue;
            };
            // Harvester-vehicle interleaving: once 2 harvesters stand, build a
            // frontline of 4 combat vehicles before harvester #3, and 8 combat
            // vehicles before harvester #4 (unless rich).
            if t.unit_scoring
                && what.kind == Kind::Vehicle
                && scripted::role_of(w, what) == Some(Role::Harvester)
                && !eyes.slave_miners(w)
                && line_is_free(w, house, what)
                && !is_held_resume(w, house, what)
            {
                // nocite: design: own AI (Vanguard): interleave 4 vehicles after 2 harvesters, 8 after 3
                let need_frontline = (own_harvesters >= 2 && (own_vehicles < 4 || wary))
                    || (own_harvesters >= 3 && own_vehicles < 8 && credits < 2000); // nocite: design: own AI (Vanguard)
                if need_frontline
                    && let Some(best) = pick_best_army(Kind::Vehicle, production::line_kind(w, what))
                {
                    out[i] = Order::Build { what: best };
                    i += 1;
                    continue;
                }
            }
            if what.kind == Kind::Building
                || scripted::role_of(w, what) != Some(Role::Army)
                || !line_is_free(w, house, what)
                || crate::lifter::lifter_type(w, what)
                || crate::drain::drainer_type(w, what)
            {
                i += 1;
                continue;
            }
            // Infantry throttling on free line only (never drop a Hold::Player resume).
            if what.kind == Kind::Infantry {
                if throttle_inf && !(need_aa && can_hit_any_air(w, what)) {
                    out.remove(i);
                    continue;
                }
                i += 1;
                continue;
            }
            // Cap landable aircraft to owned reload docks so helipads do not
            // starve ground vehicle production.
            if what.kind == Kind::Aircraft
                && t.unit_scoring
                && w.rules()
                    .unit(Kind::Aircraft, what.index)
                    .is_some_and(|u| u.raw.econ.landable.unwrap_or(false))
            {
                // nocite: design: own AI (Vanguard): cap landable aircraft to docks (max docks + 1 when rich)
                let dock_cap = if credits >= THROUGHPUT_CREDITS {
                    total_air_docks.max(1) + 1
                } else {
                    total_air_docks.max(1)
                };
                // nocite: design: own AI (Vanguard): hold air expansion until 3 ground vehicles stand unless rich
                let air_starving_ground =
                    has_wf && own_vehicles < 3 && credits < INFANTRY_THROTTLE_CREDITS;
                if own_landable_air >= dock_cap || air_starving_ground {
                    out.remove(i);
                    continue;
                }
            }
            // Vehicle / Aircraft scoring on free line only.
            if t.unit_scoring
                && matches!(what.kind, Kind::Vehicle | Kind::Aircraft)
                && let Some(best) = pick_best_army(what.kind, production::line_kind(w, what))
            {
                out[i] = Order::Build { what: best };
            }
            i += 1;
        }
    }

    /// Pillar 3.1: Early active scout to uncover unexplored enemy start
    /// waypoints (and thus gain permanent live vision since fog of war is off).
    fn manage_scout(
        &mut self,
        w: &World,
        home: CellXY,
        army: &[ObjectId],
        busy: &BTreeSet<ObjectId>,
        out: &mut Vec<Order>,
    ) {
        let house = self.eyes.house;
        // Validate existing scout.
        self.scout = self
            .scout
            .filter(|&id| w.obj(id).is_some_and(|o| o.on_map && o.health > 0));
        let unexplored_start = self
            .eyes
            .enemy_starts(w)
            .into_iter()
            .filter(|&s| !shroud::explored(w, house, s))
            .min_by_key(|&s| (dist2(s, home), s.y, s.x));
        let Some(target_start) = unexplored_start else {
            self.scout = None;
            return;
        };
        if let Some(s) = self.scout {
            if !self.army.out.contains(&s) && !busy.contains(&s) {
                self.micro_guard
                    .insert(s, w.tick.saturating_add(SCOUT_GUARD_TICKS));
                if idle(w, s) {
                    out.push(Order::Behaviour {
                        actor: s,
                        behaviour: Behaviour::Move,
                        attack_move: false,
                        target: None,
                        dest: Some(Target::Cell(target_start)),
                    });
                }
            }
            return;
        }
        // Designate one cheap/fast idle unit once we have at least 2 army units.
        // nocite: design: own AI (Vanguard): wait for at least 2 units before scouting
        if army.len() < 2 {
            return;
        }
        let pick = army
            .iter()
            .copied()
            .filter(|&id| !self.army.out.contains(&id) && !busy.contains(&id) && idle(w, id))
            .filter(|&id| {
                let Some(o) = w.obj(id) else { return false };
                let Some(u) = w.rules().unit(o.kind, o.type_index) else {
                    return false;
                };
                match self.eyes.reach_region(w, id) {
                    Some(r) => regions::reach_in(w, target_start, u.zone) == Reach::Region(r),
                    None => true,
                }
            })
            .min_by_key(|&id| {
                let Some(o) = w.obj(id) else {
                    return (i32::MAX, std::cmp::Reverse(0), id);
                };
                let speed = w
                    .rules()
                    .unit(o.kind, o.type_index)
                    .map_or(0, |u| u.speed);
                (
                    production::cost_field(w, tech::type_of(o)),
                    std::cmp::Reverse(speed),
                    id,
                )
            });
        if let Some(s) = pick {
            self.scout = Some(s);
            self.micro_guard
                .insert(s, w.tick.saturating_add(SCOUT_GUARD_TICKS));
            out.push(Order::Behaviour {
                actor: s,
                behaviour: Behaviour::Move,
                attack_move: false,
                target: None,
                dest: Some(Target::Cell(target_start)),
            });
        }
    }

    /// Pillar 3.2: Smart aircraft targeting — replaces idle rearmed aircraft
    /// attack orders in-place to prioritize harvesters, wounded high-cost
    /// vehicles/artillery, and key production/economy buildings.
    fn smart_aircraft(&self, w: &World, enemies: &[(ObjectId, CellXY)], out: &mut Vec<Order>) {
        if enemies.is_empty() {
            return;
        }
        let air_units: Vec<ObjectId> = self
            .eyes
            .of_role(w, Role::Army)
            .into_iter()
            .filter(|&id| {
                w.obj(id).is_some_and(|o| o.kind == Kind::Aircraft)
                    && idle(w, id)
                    && glasnost_sim::air::ammo_left(w, id)
            })
            .collect();
        if air_units.is_empty() {
            return;
        }
        // Precompute enemy AA positions from seen_enemies only.
        let aa_cells: Vec<CellXY> = enemies
            .iter()
            .filter_map(|&(e, c)| {
                let o = w.obj(e)?;
                can_hit_any_air(w, tech::type_of(o)).then_some(c)
            })
            .collect();
        for id in air_units {
            let Some(at) = w.obj(id).map(|o| o.pos.cell()) else {
                continue;
            };
            let best = enemies
                .iter()
                .copied()
                .filter(|&(e, _)| can_attack(w, id, e))
                .max_by_key(|&(e, c)| {
                    let Some(eo) = w.obj(e) else {
                        return (i64::MIN, std::cmp::Reverse(e));
                    };
                    let tr = tech::type_of(eo);
                    let role = scripted::role_of(w, tr);
                    let base_pri: i64 = match (eo.kind, role) {
                        // nocite: design: own AI (Vanguard): aircraft ROI target tiers
                        (_, Some(Role::Harvester)) => 9000,
                        (Kind::Vehicle, Some(Role::Army)) => {
                            let str_hp = damage::strength(w, e).max(1);
                            let missing_pct =
                                i64::from(str_hp.saturating_sub(eo.health)) * PCT / i64::from(str_hp);
                            // nocite: design: own AI (Vanguard): wounded high-value vehicle bonus
                            4500 + worth(w, e) * 2 + missing_pct * 35
                        }
                        // nocite: design: own AI (Vanguard): key production/economy building priority
                        (Kind::Building, Some(Role::Factory(_) | Role::Yard | Role::Refinery)) => {
                            6500
                        }
                        // nocite: design: own AI (Vanguard): power/other building/infantry priority
                        (Kind::Building, Some(Role::Power)) => 5000, // nocite: design: own AI (Vanguard)
                        (Kind::Building, _) => 3000,                 // nocite: design: own AI (Vanguard)
                        _ => 1500 + worth(w, e),                     // nocite: design: own AI (Vanguard)
                    };
                    // Penalize targets surrounded by seen AA within 8 cells.
                    // nocite: design: own AI (Vanguard): AA avoidance radius 8
                    let aa_near = aa_cells
                        .iter()
                        .filter(|&&ac| dist2(ac, c) <= 64)
                        .count() as i64;
                    let dist = dist2(c, at).isqrt();
                    // nocite: design: own AI (Vanguard): AA penalty 900, travel distance penalty 40/cell
                    let score = base_pri - aa_near * 900 - dist * 40;
                    (score, std::cmp::Reverse(e))
                });
            let Some((target, _)) = best else { continue };
            let order = Order::Behaviour {
                actor: id,
                behaviour: Behaviour::Attack,
                attack_move: false,
                target: Some(Target::Object(target)),
                dest: None,
            };
            if let Some(slot) = out
                .iter_mut()
                .find(|o| o.actor() == Some(id) && matches!(o, Order::Behaviour { .. }))
            {
                *slot = order;
            } else {
                out.push(order);
            }
        }
    }

    /// Pillar 4: Forward staging, strike launch/reinforcement, local-threat
    /// recall, sim-safe focus fire, targeted wounded pullback, reload-window
    /// artillery kiting, and spread endgame cleanup.
    fn command(
        &mut self,
        w: &World,
        t: Tuning,
        enemies: &[(ObjectId, CellXY)],
        busy: &BTreeSet<ObjectId>,
        out: &mut Vec<Order>,
    ) {
        let Some(home) = self.eyes.home(w) else {
            return;
        };
        let army: Vec<ObjectId> = self
            .eyes
            .of_role(w, Role::Army)
            .into_iter()
            .filter(|&id| free_for_army(w, id))
            .collect();
        let alive: BTreeSet<ObjectId> = army.iter().copied().collect();
        self.army.out.retain(|id| alive.contains(id));
        if self.army.striking && self.army.out.is_empty() {
            self.army.striking = false;
            self.army.ended = w.tick;
        }

        if t.scout {
            self.manage_scout(w, home, &army, busy, out);
        }

        let enemy_ground_worth = self.army.ground_worth();
        let out_worth: i64 = self.army.out.iter().map(|&id| worth(w, id)).sum();

        // Local-threat recall (Pillar 4.2): evaluate local enemy ground force
        // within LOCAL_THREAT_RADIUS of our strike, plus fallback global recall
        // when not actively killing enemy buildings unopposed.
        if self.army.striking && t.recall > 0 {
            let strike_cells: Vec<CellXY> = self
                .army
                .out
                .iter()
                .filter_map(|&id| w.obj(id).map(|o| o.pos.cell()))
                .collect();
            let local_enemy: i64 = enemies
                .iter()
                .filter_map(|&(e, ec)| {
                    let o = w.obj(e)?;
                    if o.kind == Kind::Building
                        || o.kind == Kind::Aircraft
                        || scripted::role_of(w, tech::type_of(o)) != Some(Role::Army)
                    {
                        return None;
                    }
                    let near = strike_cells
                        .iter()
                        .any(|&sc| dist2(sc, ec) <= LOCAL_THREAT_RADIUS * LOCAL_THREAT_RADIUS);
                    near.then(|| worth(w, e))
                })
                .sum();
            let attacking_base = enemies.iter().any(|&(e, ec)| {
                w.obj(e).is_some_and(|o| o.kind == Kind::Building)
                    && strike_cells
                        .iter()
                        .any(|&sc| dist2(sc, ec) <= LOCAL_THREAT_RADIUS * LOCAL_THREAT_RADIUS)
            });
            let locally_outmatched =
                local_enemy > 0 && local_enemy * PCT > out_worth * LOCAL_RECALL_PCT;
            let globally_weak = !attacking_base
                && enemy_ground_worth > 0
                && out_worth * PCT < t.recall * enemy_ground_worth;
            if locally_outmatched || globally_weak {
                for &id in &self.army.out {
                    if !busy.contains(&id) {
                        out.push(Order::Behaviour {
                            actor: id,
                            behaviour: Behaviour::Move,
                            attack_move: false,
                            target: None,
                            dest: Some(Target::Cell(home)),
                        });
                    }
                }
                self.army.out.clear();
                self.army.striking = false;
                self.army.recalls += 1;
                self.army.ended = w.tick;
                return;
            }
        }

        // Gather idle home units (excluding scout and busy).
        let mut waiting: Vec<(i64, ObjectId)> = army
            .iter()
            .copied()
            .filter(|&id| {
                !self.army.out.contains(&id)
                    && Some(id) != self.scout
                    && !self.micro_guard.contains_key(&id)
                    && idle(w, id)
                    && !busy.contains(&id)
            })
            .filter_map(|id| w.obj(id).map(|o| (dist2(o.pos.cell(), home), id)))
            .collect();
        waiting.sort_unstable();
        let guard = t.guard.min(waiting.len());
        let goers_worth: i64 = waiting.iter().skip(guard).map(|&(_, id)| worth(w, id)).sum();
        let waited = t.patience > 0 && w.tick - self.army.ended >= t.patience;
        // nocite: design: own AI (Vanguard): after patience, edge relaxes to 80%
        let want_edge = if waited { t.edge.min(80) } else { t.edge };
        let edge_ok = want_edge == 0 || goers_worth * PCT >= want_edge * enemy_ground_worth;
        let floor_ok = goers_worth >= t.floor;
        let free_n = army.len().saturating_sub(self.army.out.len());

        // Allow heavy vehicle armies meeting the worth floor to strike without
        // waiting for cheap infantry filler to reach `t.strike`.
        // nocite: design: own AI (Vanguard): heavy force needs at least guard + 3 units
        let enough_force = waiting.len() >= t.strike
            || (t.floor > 0 && waiting.len() > guard + 2 && goers_worth >= t.floor);
        let goes = if self.army.striking {
            waiting.len() > guard && waiting.len() - guard >= t.join
        } else {
            enough_force && free_n > guard && edge_ok && floor_ok
        };

        // Forward staging for idle non-guard home units when not launching.
        if t.staging
            && !goes
            && waiting.len() > guard
            && let Some(stage_cell) = self.staging_cell(w, home)
        {
            for &(_, id) in waiting.iter().skip(guard) {
                let Some(o) = w.obj(id) else { continue };
                let at = o.pos.cell();
                if dist2(at, stage_cell) > STAGE_DEADBAND * STAGE_DEADBAND
                    && same_region(w, id, stage_cell)
                {
                    out.push(Order::Behaviour {
                        actor: id,
                        behaviour: Behaviour::Move,
                        attack_move: true,
                        target: None,
                        dest: Some(Target::Cell(stage_cell)),
                    });
                }
            }
        }

        // Launch strike or reinforce existing strike.
        let mut goals = Goals::new(&self.eyes, w, home, enemies, t.hunts);
        let mut send: Vec<ObjectId> = Vec::new();
        if goes {
            send.extend(waiting.iter().skip(guard).map(|&(_, id)| id));
        }
        for &id in &self.army.out {
            if !busy.contains(&id) && !self.micro_guard.contains_key(&id) {
                send.push(id);
            }
        }
        let mut sent_any = false;
        for id in send {
            let Some(o) = w.obj(id) else { continue };
            let at = o.pos.cell();
            let Some(goal) = goals.for_unit(w, id) else {
                continue;
            };
            let out_already = self.army.out.contains(&id);
            if out_already {
                let fighting = o.target.is_some();
                let toward = match o.dest {
                    Some(Target::Cell(c)) => dist2(c, goal) <= SAME_GOAL * SAME_GOAL,
                    _ => false,
                };
                if fighting || (toward && !idle(w, id)) {
                    continue;
                }
            }
            if dist2(goal, at) <= ARRIVED * ARRIVED {
                continue;
            }
            out.push(Order::Behaviour {
                actor: id,
                behaviour: Behaviour::Move,
                attack_move: true,
                target: None,
                dest: Some(Target::Cell(goal)),
            });
            if !out_already {
                self.army.out.insert(id);
                sent_any = true;
            }
        }
        if sent_any && !self.army.striking {
            self.army.striking = true;
            self.army.strikes += 1;
        }

        // Pillar 4.4 & 4.5: Targeted wounded pullback & reload-window artillery kiting.
        let mut micro_acted = BTreeSet::new();
        if t.advanced_micro && !enemies.is_empty() {
            self.wounded_pullback(w, home, &army, enemies, busy, &mut micro_acted, out);
            self.artillery_kite(w, home, &army, enemies, busy, &mut micro_acted, out);
        }

        // Pillar 4.3: Sim-safe in-range focus fire with overkill splitting & hysteresis.
        if t.focus_fire && !enemies.is_empty() {
            self.focus_fire(w, &army, enemies, busy, &micro_acted, out);
        }
    }

    /// Computes a passable forward staging cell [`STAGE_DIST`] cells from
    /// `home` toward the nearest enemy start (using only own shroud/in_view).
    fn staging_cell(&self, w: &World, home: CellXY) -> Option<CellXY> {
        let enemy_start = self
            .eyes
            .enemy_starts(w)
            .into_iter()
            .min_by_key(|&s| (dist2(s, home), s.y, s.x))?;
        let dx = i64::from(enemy_start.x - home.x);
        let dy = i64::from(enemy_start.y - home.y);
        let len = (dx * dx + dy * dy).isqrt().max(1);
        let target = CellXY::new(
            home.x + (dx * i64::from(STAGE_DIST) / len) as i32,
            home.y + (dy * i64::from(STAGE_DIST) / len) as i32,
        );
        let house = self.eyes.house;
        let taken: BTreeSet<CellXY> = w
            .objects
            .values()
            .filter(|o| o.on_map && o.kind == Kind::Building && crate::in_view(w, house, o.id))
            .flat_map(|o| crate::cells_of(w, o))
            .collect();
        // nocite: design: own AI (Vanguard): spiral search up to 4 rings for clear staging cell
        (0_i32..=4).find_map(|r| {
            (-r..=r)
                .flat_map(|oy| (-r..=r).map(move |ox| (ox, oy)))
                .filter(|&(ox, oy)| ox.abs().max(oy.abs()) == r)
                .map(|(ox, oy)| CellXY::new(target.x + ox, target.y + oy))
                .find(|&c| w.cells.usable(c) && !taken.contains(&c))
        })
    }

    /// Targeted wounded pullback: pulls back high-value fast vehicles (<35% HP)
    /// ONLY when actively targeted by a seen enemy AND either a friendly repair
    /// depot exists or >= 2 healthy nearby allies can take aggro.
    #[allow(clippy::too_many_arguments)]
    fn wounded_pullback(
        &mut self,
        w: &World,
        home: CellXY,
        army: &[ObjectId],
        enemies: &[(ObjectId, CellXY)],
        busy: &BTreeSet<ObjectId>,
        micro_acted: &mut BTreeSet<ObjectId>,
        out: &mut Vec<Order>,
    ) {
        let targeted: BTreeSet<ObjectId> = enemies
            .iter()
            .filter_map(|&(e, _)| match w.obj(e)?.target {
                Some(Target::Object(t)) => Some(t),
                _ => None,
            })
            .collect();
        if targeted.is_empty() {
            return;
        }
        let depots: Vec<ObjectId> = self
            .eyes
            .own(w)
            .into_iter()
            .filter(|&id| glasnost_sim::depot::is_depot(w, id))
            .collect();
        let healthy_allies: Vec<(ObjectId, CellXY)> = army
            .iter()
            .copied()
            .filter_map(|id| {
                let o = w.obj(id)?;
                let max_hp = i64::from(damage::strength(w, id).max(1));
                (i64::from(o.health) * PCT >= max_hp * SCREEN_ALLY_HP_PCT)
                    .then_some((id, o.pos.cell()))
            })
            .collect();

        for &id in army {
            if !targeted.contains(&id)
                || busy.contains(&id)
                || self.micro_guard.contains_key(&id)
                || Some(id) == self.scout
            {
                continue;
            }
            let Some(o) = w.obj(id) else { continue };
            if o.kind != Kind::Vehicle || o.contact.is_some() {
                continue;
            }
            let tr = tech::type_of(o);
            if production::cost_field(w, tr) < PULLBACK_MIN_COST {
                continue;
            }
            let speed = w.rules().unit(o.kind, o.type_index).map_or(0, |u| u.speed);
            if speed < PULLBACK_MIN_SPEED {
                continue;
            }
            let max_hp = i64::from(damage::strength(w, id).max(1));
            if i64::from(o.health) * PCT >= max_hp * PULLBACK_HP_PCT {
                continue;
            }
            let at = o.pos.cell();
            let depot = depots
                .iter()
                .copied()
                .filter(|&d| glasnost_sim::depot::accepts(w, d, id, true))
                .min_by_key(|&d| (w.obj(d).map_or(i64::MAX, |b| dist2(b.pos.cell(), at)), d));
            let screens = healthy_allies
                .iter()
                .filter(|&&(aid, ac)| aid != id && dist2(ac, at) <= SCREEN_RADIUS * SCREEN_RADIUS)
                .count();
            if depot.is_none() && screens < SCREEN_MIN_ALLIES {
                continue;
            }
            if let Some(d) = depot {
                out.push(Order::Behaviour {
                    actor: id,
                    behaviour: Behaviour::Enter,
                    attack_move: false,
                    target: None,
                    dest: Some(Target::Object(d)),
                });
                self.micro_guard
                    .insert(id, w.tick.saturating_add(PULLBACK_GUARD_TICKS));
                micro_acted.insert(id);
            } else if let Some(back) = step_toward(w, id, at, home, PULLBACK_STEP) {
                out.push(Order::Behaviour {
                    actor: id,
                    behaviour: Behaviour::Move,
                    attack_move: false,
                    target: None,
                    dest: Some(Target::Cell(back)),
                });
                self.micro_guard
                    .insert(id, w.tick.saturating_add(PULLBACK_GUARD_TICKS));
                micro_acted.insert(id);
            }
        }
    }

    /// Reload-window artillery kiting: steps long-range units backward ONLY
    /// while reloading and an enemy ground threat closes within
    /// [`KITE_THREAT_RADIUS`] cells.
    #[allow(clippy::too_many_arguments)]
    fn artillery_kite(
        &mut self,
        w: &World,
        home: CellXY,
        army: &[ObjectId],
        enemies: &[(ObjectId, CellXY)],
        busy: &BTreeSet<ObjectId>,
        micro_acted: &mut BTreeSet<ObjectId>,
        out: &mut Vec<Order>,
    ) {
        let threats: Vec<CellXY> = enemies
            .iter()
            .filter_map(|&(e, ec)| {
                let o = w.obj(e)?;
                (o.kind != Kind::Building
                    && o.kind != Kind::Aircraft
                    && scripted::role_of(w, tech::type_of(o)) == Some(Role::Army))
                .then_some(ec)
            })
            .collect();
        if threats.is_empty() {
            return;
        }
        for &id in army {
            if busy.contains(&id)
                || micro_acted.contains(&id)
                || self.micro_guard.contains_key(&id)
            {
                continue;
            }
            let Some(o) = w.obj(id) else { continue };
            let tr = tech::type_of(o);
            if unit_max_range_cells(w, tr) < ARTILLERY_RANGE_CELLS {
                continue;
            }
            // Only kite while reloading (or all spawner slots busy/empty).
            let reloading = w.tick < o.combat.rearm_until
                || o.spawn.as_ref().is_some_and(|sc| {
                    !sc.slots.is_empty() && sc.slots.iter().all(|s| s.state != SlotState::Docked)
                });
            if !reloading {
                continue;
            }
            let at = o.pos.cell();
            let near_threat = threats
                .iter()
                .any(|&tc| dist2(tc, at) <= KITE_THREAT_RADIUS * KITE_THREAT_RADIUS);
            if !near_threat {
                continue;
            }
            if let Some(back) = step_toward(w, id, at, home, KITE_STEP) {
                out.push(Order::Behaviour {
                    actor: id,
                    behaviour: Behaviour::Move,
                    attack_move: false,
                    target: None,
                    dest: Some(Target::Cell(back)),
                });
                self.micro_guard
                    .insert(id, w.tick.saturating_add(KITE_GUARD_TICKS));
                micro_acted.insert(id);
            }
        }
    }

    /// Sim-safe in-range focus fire: operates ONLY on seen enemies already
    /// inside each attacker's weapon range (`approach::in_range`), suppresses
    /// redundant orders, applies 1.5x switch hysteresis against current valid
    /// combat targets, and splits overkill damage (`5/4` of remaining HP).
    fn focus_fire(
        &self,
        w: &World,
        army: &[ObjectId],
        enemies: &[(ObjectId, CellXY)],
        busy: &BTreeSet<ObjectId>,
        micro_acted: &BTreeSet<ObjectId>,
        out: &mut Vec<Order>,
    ) {
        let mut assigned_dmg: BTreeMap<ObjectId, i64> = BTreeMap::new();
        for &id in army {
            if busy.contains(&id)
                || micro_acted.contains(&id)
                || self.micro_guard.contains_key(&id)
                || Some(id) == self.scout
            {
                continue;
            }
            let Some(o) = w.obj(id) else { continue };
            if crate::gattling::engaged(w, id) {
                continue;
            }
            let mut best: Option<(i64, ObjectId)> = None;
            let mut cur_score: Option<(i64, bool)> = None;
            let cur_target = match o.target {
                Some(Target::Object(e)) => Some(e),
                _ => None,
            };

            for &(e, _) in enemies {
                let Some(eo) = w.obj(e) else { continue };
                if !approach::in_range(w, id, Target::Object(e)) || !can_attack(w, id, e) {
                    continue;
                }
                let (score, is_combat) = focus_target_score(w, eo);
                if cur_target == Some(e) {
                    cur_score = Some((score, is_combat));
                }
                // Overkill guard: once assigned damage >= 5/4 of target HP,
                // demote heavily so remaining attackers finish target #2.
                let already = assigned_dmg.get(&e).copied().unwrap_or(0);
                // nocite: design: own AI (Vanguard): 5/4 overkill buffer, 1/8 demotion
                let eff_score = if already * 4 >= i64::from(eo.health) * 5 {
                    score / 8 // nocite: design: own AI (Vanguard)
                } else {
                    score
                };
                if best.is_none_or(|(bs, bid)| (eff_score, std::cmp::Reverse(e)) > (bs, std::cmp::Reverse(bid))) {
                    best = Some((eff_score, e));
                }
            }
            let Some((best_score, best_e)) = best else {
                continue;
            };
            // Record volley damage whether we keep cur_target or switch.
            let chosen = if cur_target == Some(best_e) {
                best_e
            } else if let Some((cs, cur_is_combat)) = cur_score {
                let best_is_combat = w
                    .obj(best_e)
                    .is_some_and(|eo| focus_target_score(w, eo).1);
                // Switch hysteresis: if already shooting a combat threat,
                // switch only when best is at least 3/2 better.
                // nocite: design: own AI (Vanguard): 3/2 switch hysteresis
                let worth_switch =
                    (!cur_is_combat && best_is_combat) || best_score * 2 >= cs * 3;
                if !worth_switch {
                    cur_target.unwrap_or(best_e)
                } else {
                    best_e
                }
            } else {
                best_e
            };

            let volley = estimate_volley_damage(w, id, chosen);
            *assigned_dmg.entry(chosen).or_default() += volley;

            // Zero redundant orders: skip if already attacking chosen.
            if o.beh.current == Some(Behaviour::Attack)
                && (o.target == Some(Target::Object(chosen))
                    || o.dest == Some(Target::Object(chosen)))
            {
                continue;
            }
            out.push(Order::Behaviour {
                actor: id,
                behaviour: Behaviour::Attack,
                attack_move: false,
                target: Some(Target::Object(chosen)),
                dest: None,
            });
        }
    }

    /// Strips `Brain::defend` orders targeting units currently protected by
    /// `micro_guard` (runs on BOTH think and non-think ticks!).
    fn protect_micro_units(&mut self, w: &World, out: &mut Vec<Order>) {
        self.micro_guard.retain(|&id, &mut until| {
            until > w.tick && w.obj(id).is_some_and(|o| o.on_map && o.health > 0)
        });
        if self.micro_guard.is_empty() {
            return;
        }
        out.retain(|o| match o {
            Order::Behaviour {
                actor,
                behaviour: Behaviour::Attack,
                ..
            } => !self.micro_guard.contains_key(actor),
            _ => true,
        });
    }
}

/// Priority score and combat-threat flag for focus-fire target `eo`.
fn focus_target_score(w: &World, eo: &Object) -> (i64, bool) {
    let tr = tech::type_of(eo);
    let role = scripted::role_of(w, tr);
    let hp = i64::from(eo.health).max(1);
    let cost = i64::from(production::cost_field(w, tr)).max(100); // nocite: design: own AI (Vanguard)
    let armed_bldg = eo.kind == Kind::Building && weapon::armed(w, eo.id);
    // nocite: design: own AI (Vanguard): priority tiers for focus fire
    let (mult, combat) = match (eo.kind, role) {
        (_, Some(Role::Army)) => (400, true),
        (Kind::Building, _) if armed_bldg => (350, true),
        (_, Some(Role::Harvester)) => (280, false),
        (Kind::Building, Some(Role::Factory(_) | Role::Yard)) => (180, false),
        (Kind::Building, Some(Role::Refinery | Role::Power)) => (140, false),
        _ => (80, false),
    };
    let score = (cost * mult) / hp;
    (score, combat)
}

/// Armor-adjusted single-volley damage estimate from `attacker` against
/// `target`, used for focus-fire overkill splitting.
fn estimate_volley_damage(w: &World, attacker: ObjectId, target: ObjectId) -> i64 {
    let r = w.rules();
    let mode = r.mode;
    let t = Some(Target::Object(target));
    let (_, Some(wp)) = weapon::would_use(w, attacker, t) else {
        return 1;
    };
    let wr = r.combat.weapon(wp);
    let armour = weapon::armour_class(w, Target::Object(target)).unwrap_or(0);
    if wr.spawner
        && let Some(ao) = w.obj(attacker)
        && let Some(sr) = r.missiles.spawners.get(&(ao.kind, ao.type_index))
        && let Some(set) = r
            .missiles
            .sets
            .iter()
            .find(|s| s.type_index == Some(sr.spawned))
    {
        let v = set
            .warhead
            .map_or(F64::from_i64(1, mode), |wh| r.combat.warhead(wh).versus(armour));
        let d = F64::from_i64(i64::from(set.damage.max(1)), mode)
            .mul(v, mode)
            .to_i32_common();
        return i64::from(d.max(1));
    }
    let burst = i64::from(wr.burst.max(1));
    let raw_dmg = i64::from(wr.damage.max(1));
    let v = r
        .combat
        .warhead_of(wp)
        .map_or(F64::from_i64(1, mode), |h| h.versus(armour));
    let eff = F64::from_i64(raw_dmg, mode).mul(v, mode).to_i32_common();
    i64::from(eff.max(1)) * burst
}

/// Whether the production line for `t` is currently idle (`item.is_none()`),
/// guaranteeing we never overwrite or drop a `Hold::Player` resume order.
fn line_is_free(w: &World, house: HouseId, t: TypeRef) -> bool {
    production::line_of(w, house, t).is_none_or(|l| l.item.is_none())
}

/// Whether `Order::Build { what: t }` would resume a `Hold::Player` item on its line.
fn is_held_resume(w: &World, house: HouseId, t: TypeRef) -> bool {
    production::line_of(w, house, t).is_some_and(|l| {
        l.item
            .as_ref()
            .is_some_and(|i| i.t == t && i.hold == production::Hold::Player)
    })
}

/// Whether `out` already orders a core base building.
fn base_ordered(w: &World, out: &[Order]) -> bool {
    out.iter().any(|o| {
        matches!(o, Order::Build { what } if what.kind == Kind::Building
            && matches!(
                scripted::role_of(w, *what),
                Some(Role::Yard | Role::Power | Role::Refinery | Role::Factory(_))
            ))
    })
}

/// Whether type `t` has an ordinary weapon that can hit airborne targets
/// (checked against common aircraft armour classes).
fn can_hit_any_air(w: &World, t: TypeRef) -> bool {
    // nocite: design: own AI (Vanguard): check standard light/none/medium armour classes 0..=4
    (0..=4).any(|a| crate::antiair::hits_air(w, t, a))
}

/// Maximum weapon range of type `t` in whole cells.
fn unit_max_range_cells(w: &World, t: TypeRef) -> i32 {
    let r = w.rules();
    let Some(tc) = veterancy::type_combat_in(r, t.kind, t.index) else {
        return 0;
    };
    let max_lep = tc
        .slots
        .iter()
        .flatten()
        .map(|&wp| r.combat.weapon(wp).range)
        .max()
        .unwrap_or(0);
    if max_lep < 0 {
        return 0;
    }
    max_lep / LEPTONS_PER_CELL
}

/// Whether `t` is a fragile long-range artillery/spawner unit subject to the
/// glass-cannon cap.
fn is_glass_artillery(w: &World, t: TypeRef) -> bool {
    let hp = w.rules().unit(t.kind, t.index).map_or(0, |u| u.strength);
    unit_max_range_cells(w, t) >= ARTILLERY_RANGE_CELLS && hp <= GLASS_CANNON_HP
}

/// Computes combat value score for unit type `t` in pure `i128` arithmetic
/// (zero floats). Covers standard weapons, gattling slots, and spawner/rocket
/// units (`LNCH`/V3/Dreadnought/Carrier).
fn score_unit_type(
    w: &World,
    house: HouseId,
    t: TypeRef,
    credits: i32,
    armour_weights: &[i64; ARMOUR_CLASSES],
    need_aa: bool,
    enemy_crushable: bool,
) -> i128 {
    let r = w.rules();
    let mode = r.mode;
    let Some(u) = r.unit(t.kind, t.index) else {
        return 0;
    };
    let Some(tc) = veterancy::type_combat_in(r, t.kind, t.index) else {
        return 0;
    };
    let cost = i128::from(production::price(w, house, t).max(1));
    let hp = i128::from(u.strength.max(1));

    // Build effective armor target weights: observed enemy mix or balanced
    // default across None(0), Flak(1), Plate(2), Light(3), Medium(4), Heavy(5).
    let mut weights = *armour_weights;
    let sum_w: i64 = weights.iter().sum();
    if sum_w == 0 {
        // nocite: design: own AI (Vanguard): balanced default armor mix before contact
        weights[0] = 30;
        weights[3] = 25; // nocite: design: own AI (Vanguard)
        weights[4] = 25; // nocite: design: own AI (Vanguard)
        weights[5] = 20; // nocite: design: own AI (Vanguard)
    }
    let total_w = i128::from(weights.iter().sum::<i64>().max(1));

    // Evaluate best weapon slot (including gattling slots and spawners).
    let mut best_dps_x1000: i128 = 0;
    let mut best_range_cells: i128 = 0;
    for &wp in tc.slots.iter().flatten() {
        let wr = r.combat.weapon(wp);
        if wr.never_use || wr.suicide || wr.drain {
            continue;
        }
        let range_cells = i128::from((wr.range / LEPTONS_PER_CELL).max(1));
        if wr.spawner
            && let Some(sr) = r.missiles.spawners.get(&(t.kind, t.index))
        {
            // Spawner vehicle (e.g. V3 / LNCH / Dreadnought / Carrier).
            let (dmg, wh_opt, period) = if let Some(set) = r
                .missiles
                .sets
                .iter()
                .find(|s| s.type_index == Some(sr.spawned))
            {
                let p = sr.regen.max(wr.rof).max(60); // nocite: design: own AI (Vanguard)
                (set.damage.max(0), set.warhead, p)
            } else if let Some(au) = r.aircraft.get(sr.spawned as usize)
                && let Some(awp) = au.combat.slots[0]
            {
                let awr = r.combat.weapon(awp);
                let p = sr.reload.max(awr.rof).max(60); // nocite: design: own AI (Vanguard)
                (awr.damage.max(0) * sr.count.max(1), awr.warhead, p)
            } else {
                (0, None, 100) // nocite: design: own AI (Vanguard)
            };
            if dmg > 0 {
                let mut weighted_v: i128 = 0;
                for (a, &wt) in weights.iter().enumerate() {
                    if wt <= 0 {
                        continue;
                    }
                    let v = wh_opt.map_or(SCALE, |wh| {
                        i128::from(
                            r.combat
                                .warhead(wh)
                                .versus(a)
                                .mul(F64::from_i64(SCALE as i64, mode), mode)
                                .to_i32_common()
                                .max(0),
                        )
                    });
                    weighted_v += v * i128::from(wt);
                }
                let avg_v = weighted_v / total_w;
                let splash_mult = wh_opt.map_or(SCALE, |wh| {
                    let spread_x100 = r
                        .combat
                        .warhead(wh)
                        .spread
                        .mul(F64::from_i64(100, mode), mode) // nocite: design: own AI (Vanguard)
                        .to_i32_common()
                        .max(0);
                    // nocite: design: own AI (Vanguard): modest splash bonus up to 1.6x
                    SCALE + i128::from(spread_x100.min(200)) * 3
                });
                let dps = i128::from(dmg) * avg_v * splash_mult / (SCALE * i128::from(period));
                if dps > best_dps_x1000 {
                    best_dps_x1000 = dps;
                    best_range_cells = range_cells;
                }
            }
            continue;
        }
        if wr.damage <= 0 {
            continue;
        }
        // Skip AA-only weapons for ground DPS calculation.
        let hits_ground = r.combat.projectile_of(wp).is_none_or(|p| p.ag);
        if !hits_ground {
            continue;
        }
        let burst = i128::from(wr.burst.max(1));
        let rof = i128::from(wr.rof.max(1));
        let wh_opt = wr.warhead;
        let mut weighted_v: i128 = 0;
        for (a, &wt) in weights.iter().enumerate() {
            if wt <= 0 {
                continue;
            }
            let v = wh_opt.map_or(SCALE, |wh| {
                i128::from(
                    r.combat
                        .warhead(wh)
                        .versus(a)
                        .mul(F64::from_i64(SCALE as i64, mode), mode)
                        .to_i32_common()
                        .max(0),
                )
            });
            weighted_v += v * i128::from(wt);
        }
        let avg_v = weighted_v / total_w;
        let splash_mult = wh_opt.map_or(SCALE, |wh| {
            let spread_x100 = r
                .combat
                .warhead(wh)
                .spread
                .mul(F64::from_i64(100, mode), mode) // nocite: design: own AI (Vanguard)
                .to_i32_common()
                .max(0);
            // nocite: design: own AI (Vanguard): splash bonus up to 1.6x
            SCALE + i128::from(spread_x100.min(200)) * 3
        });
        let dps = i128::from(wr.damage) * burst * avg_v * splash_mult / (SCALE * rof);
        if dps > best_dps_x1000 {
            best_dps_x1000 = dps;
            best_range_cells = range_cells;
        }
    }
    if best_dps_x1000 == 0 && !(need_aa && can_hit_any_air(w, t)) {
        return 0;
    }
    // Range and speed bonuses (pure integer, capped so high-range doesn't break balance).
    // nocite: design: own AI (Vanguard): range & speed scaling factors
    let range_factor = SCALE + best_range_cells.min(12) * 35;
    let speed_factor = SCALE + i128::from(u.speed.clamp(0, 14)) * 20; // nocite: design: own AI (Vanguard)
    let crusher_factor = if enemy_crushable && u.crusher {
        1150 // nocite: design: own AI (Vanguard): 1.15x crusher bonus
    } else {
        SCALE
    };
    let aa_factor = if need_aa && can_hit_any_air(w, t) {
        2500 // nocite: design: own AI (Vanguard): 2.5x anti-air priority boost
    } else {
        SCALE
    };
    let combat_power = best_dps_x1000.max(100) // nocite: design: own AI (Vanguard)
        * hp
        / SCALE
        * range_factor
        / SCALE
        * speed_factor
        / SCALE
        * crusher_factor
        / SCALE
        * aa_factor
        / SCALE;

    // Regime switch: when credits < THROUGHPUT_CREDITS, optimize per credit;
    // when rich, shift toward per build-tick throughput so heavy tier-3 units win.
    if credits >= THROUGHPUT_CREDITS {
        let bt = i128::from(production::build_time(w, house, t).max(1));
        combat_power * SCALE / bt
    } else {
        combat_power * SCALE / cost
    }
}

/// Whether army unit `id` is free for Vanguard's army & micro control.
fn free_for_army(w: &World, id: ObjectId) -> bool {
    let Some(o) = w.obj(id) else { return false };
    let non_combat_job = matches!(
        o.beh.current,
        Some(
            Behaviour::Eaten
                | Behaviour::Enter
                | Behaviour::Unload
                | Behaviour::Capture
                | Behaviour::Sabotage
                | Behaviour::Harvest
                | Behaviour::Return
                | Behaviour::Construction
                | Behaviour::Selling
        )
    ) || matches!(
        o.beh.queued,
        Some(
            Behaviour::Eaten
                | Behaviour::Enter
                | Behaviour::Unload
                | Behaviour::Capture
                | Behaviour::Sabotage
        )
    );
    let bunkered = glasnost_sim::structures::bunker::bunkered(w, id);
    let berserk = glasnost_sim::psychic::berserk(w, id);
    let disabled = o.kind == Kind::Vehicle && power::unit_disabled(w, id);
    o.kind != Kind::Aircraft
        && o.contact.is_none()
        && !non_combat_job
        && !bunkered
        && !berserk
        && !disabled
        && !scripted::spent_alone(w, tech::type_of(o))
        && !crate::lifter::is_lifter(w, id)
        && !crate::drain::is_drainer(w, id)
        && !crate::deployers::infantry_deployed(w, id)
        && !glasnost_sim::econ::inplace::blocks_destination(w, id)
}

/// Whether cell `c` lies in unit `id`'s movement reach region.
fn same_region(w: &World, id: ObjectId, c: CellXY) -> bool {
    let Some(o) = w.obj(id) else { return false };
    if crate::jumpjets::flies(w, id) {
        return true;
    }
    let Some(zone) = w.rules().unit(o.kind, o.type_index).map(|u| u.zone) else {
        return true;
    };
    match regions::reach_in(w, o.pos.cell(), zone) {
        Reach::Region(r) => regions::reach_in(w, c, zone) == Reach::Region(r),
        _ => true,
    }
}

/// Finds a usable reachable cell roughly `steps` cells from `at` toward `goal`.
fn step_toward(w: &World, id: ObjectId, at: CellXY, goal: CellXY, steps: i32) -> Option<CellXY> {
    let dx = i64::from(goal.x - at.x);
    let dy = i64::from(goal.y - at.y);
    let len = (dx * dx + dy * dy).isqrt();
    if len == 0 {
        return None;
    }
    let target = CellXY::new(
        at.x + (dx * i64::from(steps) / len) as i32,
        at.y + (dy * i64::from(steps) / len) as i32,
    );
    // nocite: design: own AI (Vanguard): search up to 2 cells around step target
    (0_i32..=2).find_map(|r| {
        (-r..=r)
            .flat_map(|oy| (-r..=r).map(move |ox| (ox, oy)))
            .filter(|&(ox, oy)| ox.abs().max(oy.abs()) == r)
            .map(|(ox, oy)| CellXY::new(target.x + ox, target.y + oy))
            .find(|&c| c != at && w.cells.usable(c) && same_region(w, id, c))
    })
}

/// Deduplicates per-actor `Order::Behaviour` and per-building `Order::Repair`,
/// orders macro orders before micro orders, and enforces `PENDING_LIMIT`.
fn finalize_orders(w: &World, house: HouseId, out: &mut Vec<Order>) {
    // Keep only the LAST Behaviour order per actor, and at most ONE Repair per actor.
    let mut last_beh: BTreeMap<ObjectId, usize> = BTreeMap::new();
    for (idx, o) in out.iter().enumerate() {
        if let Order::Behaviour { actor, .. } = o {
            last_beh.insert(*actor, idx);
        }
    }
    let mut seen_repair: BTreeSet<ObjectId> = BTreeSet::new();
    let mut macro_orders = Vec::new();
    let mut micro_orders = Vec::new();
    for (idx, o) in out.drain(..).enumerate() {
        match o {
            Order::Behaviour { actor, .. } => {
                if last_beh.get(&actor) == Some(&idx) {
                    micro_orders.push(o);
                }
            }
            Order::Repair { actor } => {
                if seen_repair.insert(actor) {
                    macro_orders.push(o);
                }
            }
            _ => macro_orders.push(o),
        }
    }
    macro_orders.extend(micro_orders);
    if w.rules().quirks.pending_order_limit {
        let pending = w
            .houses
            .get(usize::from(house.0))
            .map_or(0, |h| h.pending.len());
        let room = orders::PENDING_LIMIT.saturating_sub(pending);
        macro_orders.truncate(room);
    }
    *out = macro_orders;
}

/// Strike & spread-hunt goal resolver for one think.
struct Goals<'a> {
    eyes: &'a Script,
    home: CellXY,
    enemies: &'a [(ObjectId, CellXY)],
    hunts: bool,
    focus: Option<HouseId>,
    kept: BTreeMap<Option<(u8, u32)>, Option<CellXY>>,
    open_starts: Vec<CellXY>,
    claimed_hunt: BTreeSet<CellXY>,
}

impl<'a> Goals<'a> {
    fn new(
        eyes: &'a Script,
        w: &World,
        home: CellXY,
        enemies: &'a [(ObjectId, CellXY)],
        hunts: bool,
    ) -> Self {
        let focus = enemies
            .iter()
            .filter_map(|&(e, c)| {
                let o = w.obj(e)?;
                (o.kind == Kind::Building).then_some((dist2(c, home), e, o.owner))
            })
            .min()
            .and_then(|(_, _, owner)| owner);
        let house = eyes.house;
        let open_starts = eyes
            .enemy_starts(w)
            .into_iter()
            .filter(|&s| {
                !shroud::explored(w, house, s)
                    || enemies.iter().any(|&(e, c)| {
                        dist2(c, s) <= SITE * SITE
                            && w.obj(e).is_some_and(|o| o.kind == Kind::Building)
                    })
            })
            .collect();
        Self {
            eyes,
            home,
            enemies,
            hunts,
            focus,
            kept: BTreeMap::new(),
            open_starts,
            claimed_hunt: BTreeSet::new(),
        }
    }

    /// Goal cell for unit `id`: enemy harvester/refinery nearest home, then
    /// focus-house buildings, then any enemy building, then open enemy starts;
    /// during endgame hunt, nearest seen reachable enemy, else distinct unseen
    /// search-grid cells so units fan out cleanly.
    fn for_unit(&mut self, w: &World, id: ObjectId) -> Option<CellXY> {
        let o = w.obj(id)?;
        let zone = w.rules().unit(o.kind, o.type_index).map(|u| u.zone);
        let key = match (zone, self.eyes.reach_region(w, id)) {
            (Some(z), Some(r)) => Some((z as u8, r)),
            _ => None,
        };
        let in_region = |c: CellXY| match (zone, key) {
            (Some(z), Some((_, r))) => regions::reach_in(w, c, z) == Reach::Region(r),
            _ => true,
        };
        let home = self.home;
        let focus = self.focus;
        let enemies = self.enemies;
        let open_starts = &self.open_starts;
        let base = *self.kept.entry(key).or_insert_with(|| {
            let role = |e: ObjectId| {
                w.obj(e)
                    .and_then(|x| scripted::role_of(w, tech::type_of(x)))
            };
            let economy = || {
                enemies
                    .iter()
                    .copied()
                    .filter(|&(e, c)| {
                        matches!(role(e), Some(Role::Harvester | Role::Refinery)) && in_region(c)
                    })
                    .min_by_key(|&(e, c)| (dist2(c, home), e))
                    .map(|(_, c)| c)
            };
            let building = |any: bool| {
                enemies
                    .iter()
                    .copied()
                    .filter(|&(e, c)| {
                        w.obj(e)
                            .is_some_and(|x| x.kind == Kind::Building && (any || x.owner == focus))
                            && in_region(c)
                    })
                    .min_by_key(|&(e, c)| (dist2(c, home), e))
                    .map(|(_, c)| c)
            };
            economy()
                .or_else(|| building(false))
                .or_else(|| building(true))
                .or_else(|| {
                    open_starts
                        .iter()
                        .copied()
                        .filter(|&s| in_region(s))
                        .min_by_key(|&s| (dist2(s, home), s.y, s.x))
                })
        });
        if base.is_some() || !self.hunts {
            return base;
        }
        let at = o.pos.cell();
        let house = self.eyes.house;
        if let Some((_, c)) = enemies
            .iter()
            .copied()
            .filter(|&(e, c)| in_region(c) && can_attack(w, id, e))
            .min_by_key(|&(e, c)| (dist2(c, at), e))
        {
            return Some(c);
        }
        // Spread hunt: pick nearest unclaimed unseen grid cell first, falling
        // back to nearest unseen grid cell if all are claimed.
        let pick = w
            .cells
            .grid
            .iter()
            .map(|(c, _)| c)
            .filter(|c| c.x % SEARCH_STEP == 0 && c.y % SEARCH_STEP == 0)
            .filter(|&c| w.cells.usable(c) && !shroud::seen(w, house, c) && in_region(c))
            .min_by_key(|&c| (self.claimed_hunt.contains(&c), dist2(c, at), c.y, c.x));
        if let Some(c) = pick {
            self.claimed_hunt.insert(c);
        }
        pick
    }
}

impl AiScript for Vanguard {
    fn house(&self) -> HouseId {
        self.brain.house()
    }

    fn think<P: Probe>(
        &mut self,
        view: &View<'_>,
        schedule: Schedule,
        level: AiDifficulty,
        own: Option<Arc<[ObjectId]>>,
        p: &mut P,
    ) -> Vec<Order> {
        let w = view.world();
        let house = self.house();
        let mut out = AiScript::think(&mut self.brain, view, schedule, level, own.clone(), p);
        // On EVERY tick (think and non-think), protect active micro/scout units
        // from being hijacked by Brain::defend.
        self.protect_micro_units(w, &mut out);
        if !schedule.thinks {
            return out;
        }

        let t = tuning(level);
        let acts = w
            .tick
            .div_euclid(scripted::THINK_PERIOD)
            .rem_euclid(t.every)
            == 0;
        let busy: BTreeSet<ObjectId> = out.iter().filter_map(Order::actor).collect();

        match own {
            Some(own) => self.eyes.begin_think_with_own(own),
            None => self.eyes.begin_think(),
        }

        // Single exclusive fair-play enemy vision read for this think.
        let enemies = self.eyes.seen_enemies(w);
        self.army.remember(w, house, &enemies);

        let by_land = self.by_land(w);
        let wary = self.is_wary(w, t);
        let credits = w.houses[usize::from(house.0)].credits;
        let tech_open = t.tech_gate <= 0
            || self.own_ground_worth(w) >= t.tech_gate
            || credits >= THROUGHPUT_CREDITS;

        // Pillar 1: Rules-adaptive economy & base management.
        if !t.superweapons {
            self.no_superweapons(w, tech_open, &mut out);
        }
        if !t.navy {
            self.no_navy(w, by_land, tech_open, &mut out);
        }
        self.no_land_ferries(w, by_land, &mut out);
        self.gate_tech_climb(w, tech_open, wary, &mut out);
        self.manage_refineries(w, t, wary, &mut out);
        self.manage_factories(w, t, wary, &mut out);
        if t.beside {
            self.beside_soldiers(w, &mut out);
        }
        if t.defences {
            self.manage_defences(w, wary, &mut out);
        }
        if t.repair {
            self.manage_repair(w, &mut out);
        }

        // Pillar 2: Rules-driven vehicle/aircraft scoring & infantry throttling.
        self.optimize_production(w, t, by_land, wary, &mut out);

        // Pillar 3.2: Smart aircraft sniping.
        if t.air_snipe {
            self.smart_aircraft(w, &enemies, &mut out);
        }

        // Pillar 3.1 & Pillar 4: Scouting, staging, strike, focus fire & micro.
        if acts {
            self.command(w, t, &enemies, &busy, &mut out);
        }

        self.eyes.end_think();

        // Enforce all 10 Brain unit-safety filters, macro-first priority, dedup
        // and PENDING_LIMIT.
        self.brain.filter_orders(w, &mut out);
        finalize_orders(w, house, &mut out);
        out
    }
}
