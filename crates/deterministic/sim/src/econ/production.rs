//! Production lines, payment and build time (S-0403, S-0404).
//!
//! Every house has at most one line of each of six kinds (S-0404 claim 1),
//! all kept in [`Lines`] in creation order. A line builds its item in 54
//! steps of `period` ticks each (S-0403 claim 1), paying each step when it
//! is made (S-0404 claim 4). The orders of S-0404 (build, hold, cancel,
//! cancel-all, place) arrive through `crate::orders`; the factory checks of
//! claim 8 arrive through `crate::econ::events`.

use glasnost_map::CellXY;
use glasnost_num::{F64, Rounding};
use glasnost_rules::schema::Ruleset;
use serde::{Deserialize, Serialize};

use super::TypeRef;
use super::tech::{self, Answer};
use crate::rules::Kind;
use crate::world::{HouseId, HouseRole, Object, ObjectId, World};

/// Progress steps per item (S-0403 claim 1).
// spec: S-0403 Constants (54 progress steps per item)
pub const STEPS: i32 = 54;

/// The upper limit of the step period (S-0403 claim 1).
// spec: S-0403 Constants (step period limits 1 to 255 ticks)
pub const MAX_PERIOD: i32 = 255;

/// The last step that pays (S-0404 claim 4).
// spec: S-0404 Constants (last paying step: 53)
pub const LAST_PAYING_STEP: i32 = 53;

/// The six line kinds (S-0403 Terms, S-0404 claim 1).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum LineKind {
    /// Infantry.
    Infantry,
    /// Ground vehicles.
    Ground,
    /// Naval vehicles.
    Naval,
    /// Aircraft.
    Aircraft,
    /// Ordinary buildings.
    Building,
    /// Defences.
    Defence,
}

impl LineKind {
    /// The kind of factory building that serves the line (S-0403 Terms).
    pub fn factory_kind(self) -> Kind {
        match self {
            LineKind::Infantry => Kind::Infantry,
            LineKind::Ground | LineKind::Naval => Kind::Vehicle,
            LineKind::Aircraft => Kind::Aircraft,
            LineKind::Building | LineKind::Defence => Kind::Building,
        }
    }

    /// Whether items of the line are units (S-0404 claim 7).
    pub fn unit(self) -> bool {
        !matches!(self, LineKind::Building | LineKind::Defence)
    }

    /// The index of the country's and the cost bonus's factor for the kind:
    /// infantry, vehicles, aircraft, ordinary buildings, defences (S-0403
    /// claim 3, S-0404 claim 2).
    fn factor(self) -> usize {
        // spec: S-0403 claim 3 (five kinds: infantry, vehicles, aircraft, ordinary buildings, defences)
        match self {
            LineKind::Infantry => 0,
            LineKind::Ground | LineKind::Naval => 1,
            LineKind::Aircraft => 2,
            LineKind::Building => 3,
            LineKind::Defence => 4,
        }
    }
}

/// A line's hold (S-0404 Terms): at most one at a time.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Hold {
    /// Not held.
    #[default]
    None,
    /// A player hold (S-0404 claim 5; also the forced start of claim 10).
    Player,
    /// An automatic hold (S-0404 claim 8).
    Auto,
}

/// The item a line is building (S-0403 Terms, S-0404 claim 3).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Item {
    /// The type.
    pub t: TypeRef,
    /// Its object: it exists from the start, off the map (S-0404 claim 3).
    pub object: ObjectId,
    /// Progress, 0 to 54 (S-0403 Terms).
    pub progress: i32,
    /// The balance owed *B* (S-0404 Terms).
    pub owed: i32,
    /// The price at the start (S-0404 Edge cases: needed only with
    /// `quirks.refund_at_current_price` off).
    pub start_price: i32,
    /// The step period *R* in force (S-0403 claims 1 and 8).
    pub period: i32,
    /// The tick of the next step or retry (S-0403 State and ordering).
    pub due: i32,
    /// The hold.
    pub hold: Hold,
    /// Waiting for money (S-0404 Terms).
    pub waiting: bool,
    /// The ticks that remained of the step under way when the line was held
    /// (used with `quirks.resume_restarts_step` off).
    pub remaining: i32,
    /// The tick the item became done (S-0411 claim 9).
    pub done_tick: Option<i32>,
}

impl Item {
    /// Done (S-0403 Terms).
    pub fn done(&self) -> bool {
        self.progress >= STEPS
    }

    /// Running or waiting for money: not done, not held.
    pub fn running(&self) -> bool {
        !self.done() && self.hold == Hold::None
    }
}

/// One production line (S-0404 claim 1).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Line {
    /// The house.
    pub house: HouseId,
    /// The kind.
    pub kind: LineKind,
    /// The item, if any.
    pub item: Option<Item>,
    /// The queue (S-0404 Terms).
    pub queue: Vec<TypeRef>,
}

/// Every production line of every house, in the order the lines were
/// created (S-0404 claim 1).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Lines {
    /// The lines in creation order.
    pub lines: Vec<Line>,
}

/// Production state of a house that is not a line (S-0404 claim 2). The
/// cost bonus product is worked out from the house's buildings whenever a
/// price is asked, which is the same as working it out again at every gain
/// or loss of a bonus building.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HouseProduction {}

/// The production fields of one type.
#[derive(Clone, Debug, Default)]
pub struct TypeProd {
    /// The cost field *C* (S-0403 claim 2).
    pub cost: i32,
    /// The type's build-time multiplier *m*, a single (S-0403 claim 4).
    pub multiplier: F64,
    /// A building type with the wall flag (S-0403 claim 7).
    pub wall: bool,
    /// A building type of the combat category (S-0403 Terms).
    pub defence: bool,
    /// A vehicle type with the naval flag (S-0403 Terms).
    pub naval: bool,
    /// A building type that grants cost bonuses: its bonuses per kind
    /// (S-0404 claim 2).
    pub bonus: Option<[F64; 5]>, // spec: S-0404 claim 2 (a bonus for each of five kinds)
    /// A building type's free unit type (S-0404 claim 2).
    pub free_unit: Option<TypeRef>,
    /// An aircraft type's first dock type (S-0404 Terms).
    pub first_dock: Option<u32>,
}

/// A country's production factors (S-0403 claim 3, S-0404 claim 2).
#[derive(Clone, Debug, Default)]
pub struct CountryProd {
    /// The cost factors *a* per kind (singles).
    pub cost: [F64; 5], // spec: S-0404 claim 2 (a cost factor for each of five kinds)
    /// The build-time factors *c* per kind (singles).
    pub time: [F64; 5], // spec: S-0403 claim 3 (a build-time factor for each of five kinds)
}

/// The typed production rules (S-0403 and S-0404 Import mapping).
#[derive(Clone, Debug, Default)]
pub struct Rules {
    /// Build speed *S* (S-0403 claim 2).
    pub build_speed: F64,
    /// *k*₀, the double nearest 0.9 (S-0403 claim 2).
    pub k0: F64,
    /// Low-power penalty *q* (S-0403 claim 5).
    pub penalty: F64,
    /// *f*ₘᵢₙ.
    pub min_speed: F64,
    /// *f*ₘₐₓ.
    pub max_speed: F64,
    /// The zero-speed replacement (S-0403 claim 5).
    pub zero_speed: F64,
    /// The multiple-factory factor *g* (S-0403 claim 6).
    pub multiple_factory: F64,
    /// The wall coefficient *w* (S-0403 claim 7).
    pub wall: F64,
    /// The maximum number of queued types per line (S-0404 claim 3).
    pub max_queued: i32,
    /// The pose direction of an aircraft released onto a dock, 1/256 turns
    /// (S-0404 claim 13).
    pub pose_dir: i32,
    /// The game-wide deploy direction, an 8-step direction (S-0502 claim 15
    /// and Import mapping; absent 0).
    pub deploy_dir: i32,
    /// The separate-aircraft flag (S-0404 claim 2).
    pub separate_aircraft: bool,
    /// The pad aircraft types (S-0404 claim 2).
    pub pad_aircraft: Vec<u32>,
    /// Countries by registration index.
    pub countries: Vec<CountryProd>,
    /// Infantry types.
    pub infantry: Vec<TypeProd>,
    /// Vehicle types.
    pub vehicles: Vec<TypeProd>,
    /// Aircraft types.
    pub aircraft: Vec<TypeProd>,
    /// Building types.
    pub buildings: Vec<TypeProd>,
}

/// The single nearest to a decimal (an engine constant that a spec gives as
/// "the single nearest"), widened exactly.
fn single(text: &str) -> F64 {
    glasnost_num::decimal::parse(
        text,
        glasnost_num::decimal::Field::Single,
        Rounding::TowardZero,
    )
    .unwrap_or_default()
}

impl Rules {
    /// The production rules of a resolved ruleset.
    pub fn new(r: &Ruleset, mode: Rounding) -> Self {
        let s = &r.sim;
        let g = &s.general.economy;
        let one = F64::from_i64(1, mode);
        let reg = |k: &str| r.registries.get(k).cloned().unwrap_or_default();
        let (infantry, vehicles, aircraft, buildings) = (
            reg("infantry"),
            reg("vehicles"),
            reg("aircraft"),
            reg("buildings"),
        );
        let find = |list: &[String], n: &str| -> Option<u32> {
            // spec: S-0003 claim 14 (names compared with ASCII case folding)
            list.iter()
                .position(|x| x.eq_ignore_ascii_case(n))
                .map(|i| i as u32)
        };
        let sv = |d: &Option<glasnost_rules::sim::Decimal>, default: F64| -> F64 {
            d.as_ref().map_or(default, |d| d.single_value(mode))
        };
        let dv = |d: &Option<glasnost_rules::sim::Decimal>, default: F64| -> F64 {
            d.as_ref().map_or(default, |d| d.value(mode))
        };
        let countries = reg("countries")
            .iter()
            .map(|n| {
                let c = s.countries.get(n).cloned().unwrap_or_default().econ;
                // spec: S-0404 Import mapping (CostInfantryMult … CostDefensesMult, singles, absent 1.0)
                let cost = [
                    sv(&c.cost_infantry_mult, one),
                    sv(&c.cost_units_mult, one),
                    sv(&c.cost_aircraft_mult, one),
                    sv(&c.cost_buildings_mult, one),
                    sv(&c.cost_defenses_mult, one),
                ];
                // spec: S-0403 Import mapping (BuildTimeInfantryMult … BuildTimeDefensesMult, singles, absent 1.0)
                let time = [
                    sv(&c.build_time_infantry_mult, one),
                    sv(&c.build_time_units_mult, one),
                    sv(&c.build_time_aircraft_mult, one),
                    sv(&c.build_time_buildings_mult, one),
                    sv(&c.build_time_defenses_mult, one),
                ];
                CountryProd { cost, time }
            })
            .collect();
        let unit = |u: &glasnost_rules::sim::UnitType| TypeProd {
            // spec: S-0403 Import mapping (Cost absent: 0)
            cost: u.cost.unwrap_or(0),
            // spec: S-0403 Import mapping (BuildTimeMultiplier, a single, absent 1.0)
            multiplier: sv(&u.econ.build_time_multiplier, one),
            // spec: S-0403 Import mapping (Naval absent: no)
            naval: u.combat.naval.unwrap_or(false),
            first_dock: u
                .econ
                .dock
                .iter()
                .flatten()
                .find_map(|n| find(&buildings, n)),
            ..Default::default()
        };
        let units =
            |list: &[String],
             map: &std::collections::BTreeMap<String, glasnost_rules::sim::UnitType>| {
                list.iter()
                    .map(|n| unit(&map.get(n).cloned().unwrap_or_default()))
                    .collect::<Vec<_>>()
            };
        let free_unit = |n: &str| -> Option<TypeRef> {
            [
                (Kind::Vehicle, &vehicles),
                (Kind::Infantry, &infantry),
                (Kind::Aircraft, &aircraft),
            ]
            .into_iter()
            .find_map(|(kind, list)| find(list, n).map(|index| TypeRef { kind, index }))
        };
        let buildings_p = buildings
            .iter()
            .map(|n| {
                let b = s.buildings.get(n).cloned().unwrap_or_default();
                let e = &b.econ;
                // spec: S-0404 Import mapping (FactoryPlant absent: no; the five cost bonuses, singles, absent 1.0)
                let bonus = e.factory_plant.unwrap_or(false).then(|| {
                    [
                        sv(&e.infantry_cost_bonus, one),
                        sv(&e.units_cost_bonus, one),
                        sv(&e.aircraft_cost_bonus, one),
                        sv(&e.buildings_cost_bonus, one),
                        sv(&e.defenses_cost_bonus, one),
                    ]
                });
                TypeProd {
                    // spec: S-0403 Import mapping (Cost absent: 0)
                    cost: b.cost.unwrap_or(0),
                    // spec: S-0403 Import mapping (BuildTimeMultiplier, a single, absent 1.0)
                    multiplier: sv(&e.build_time_multiplier, one),
                    // spec: S-0403 Import mapping (Wall absent: no)
                    wall: e.wall.unwrap_or(false),
                    // spec: S-0403 Import mapping (BuildCat absent: not combat)
                    defence: b.defense.unwrap_or(false),
                    naval: b.naval.unwrap_or(false),
                    bonus,
                    free_unit: e.free_unit.as_deref().and_then(free_unit),
                    first_dock: None,
                }
            })
            .collect();
        Self {
            // spec: S-0403 Import mapping (BuildSpeed, a double, absent 1.0)
            build_speed: dv(&g.build_speed, one),
            // spec: S-0403 claim 2 (k₀: the double nearest 0.9, 900 ticks per game minute ÷ 1000)
            k0: crate::rules::nearest_double("0.9"),
            // spec: S-0403 Import mapping (LowPowerPenaltyModifier, a single, absent 1.0)
            penalty: sv(&g.low_power_penalty_modifier, one),
            // spec: S-0403 Import mapping (MinLowPowerProductionSpeed, a single, absent 0.5)
            min_speed: sv(&g.min_low_power_production_speed, single("0.5")),
            // spec: S-0403 Import mapping (MaxLowPowerProductionSpeed, a single, absent the single nearest 0.9)
            max_speed: sv(&g.max_low_power_production_speed, single("0.9")),
            // spec: S-0403 claim 5 (the zero-speed replacement: the single nearest 0.01)
            zero_speed: single("0.01"),
            // spec: S-0403 Import mapping (MultipleFactory, a single, absent 1.0)
            multiple_factory: sv(&g.multiple_factory, one),
            // spec: S-0403 Import mapping (WallBuildSpeedCoefficient, a double, absent 0.5)
            wall: dv(&g.wall_build_speed_coefficient, single("0.5")),
            // spec: S-0404 Constants (maximum queued types absent: 5)
            max_queued: g.maximum_queued_objects.unwrap_or(5),
            // spec: S-0404 Constants (pose direction absent: 0)
            pose_dir: g.pose_dir.unwrap_or(0),
            // spec: S-0502 Import mapping ([AudioVisual] DeployDir absent: 0 at program start)
            deploy_dir: g.deploy_dir.unwrap_or(0),
            // spec: S-0404 Import mapping (SeparateAircraft absent: no)
            separate_aircraft: g.separate_aircraft.unwrap_or(false),
            pad_aircraft: g
                .pad_aircraft
                .iter()
                .flatten()
                .filter_map(|n| find(&aircraft, n))
                .collect(),
            countries,
            infantry: units(&infantry, &s.infantry),
            vehicles: units(&vehicles, &s.vehicles),
            aircraft: units(&aircraft, &s.aircraft),
            buildings: buildings_p,
        }
    }

    /// A type's production fields.
    pub fn of(&self, t: TypeRef) -> Option<&TypeProd> {
        let v = match t.kind {
            Kind::Infantry => &self.infantry,
            Kind::Vehicle => &self.vehicles,
            Kind::Aircraft => &self.aircraft,
            Kind::Building => &self.buildings,
            Kind::Terrain => return None,
        };
        v.get(t.index as usize)
    }
}

fn tp(w: &World, t: TypeRef) -> TypeProd {
    w.rules().econ.production.of(t).cloned().unwrap_or_default()
}

/// Whether a vehicle type is naval (S-0403 Terms).
pub fn naval(w: &World, t: TypeRef) -> bool {
    w.rules().econ.production.of(t).is_some_and(|p| p.naval)
}

/// The line kind of a type (S-0403 Terms, S-0404 claim 1).
pub fn line_kind(w: &World, t: TypeRef) -> Option<LineKind> {
    let p = w.rules().econ.production.of(t)?;
    Some(match t.kind {
        Kind::Infantry => LineKind::Infantry,
        Kind::Vehicle if p.naval => LineKind::Naval,
        Kind::Vehicle => LineKind::Ground,
        Kind::Aircraft => LineKind::Aircraft,
        Kind::Building if p.defence => LineKind::Defence,
        Kind::Building => LineKind::Building,
        Kind::Terrain => return None,
    })
}

// ---------------------------------------------------------------------
// S-0403: build time
// ---------------------------------------------------------------------

/// The inputs of the build time of one item at one moment (S-0403 claims 2
/// to 7), for [`build_time_of`].
#[derive(Clone, Copy, Debug)]
pub struct TimeInputs {
    /// The cost field *C*.
    pub cost: i32,
    /// The country's factor *c* for the kind.
    pub country: F64,
    /// The type's multiplier *m*.
    pub multiplier: F64,
    /// The owner's power fraction *p* (S-0405 F3), a double.
    pub power: F64,
    /// The number *n* of factory buildings of the kind (claim 6).
    pub factories: i32,
    /// The wall flag.
    pub wall: bool,
}

/// The build time *T* (S-0403 F1 to F6).
pub fn build_time_of(r: &Rules, x: TimeInputs, mode: Rounding) -> i32 {
    let f = |v: i32| F64::from_i64(i64::from(v), mode);
    let one = f(1);
    // spec: S-0403 F1 (T₀ = trunc(RZ₅₃(RZ₅₃(C × S) × k₀)))
    let t0 = f(x.cost)
        .mul(r.build_speed, mode)
        .mul(r.k0, mode)
        .to_i32_common();
    // spec: S-0403 F2 (T₁ = trunc(RZ₅₃(c × T₀)))
    let t1 = x.country.mul(f(t0), mode).to_i32_common();
    // spec: S-0403 F3 (T₂ = trunc(RZ₅₃(T₁ × m)))
    let t2 = f(t1).mul(x.multiplier, mode).to_i32_common();
    // spec: S-0403 F4 (pₛ: p narrowed to a single by S-0205's narrowing rule)
    let ps = x.power.to_f32(mode).to_f64();
    // spec: S-0403 F4 (f₁ = RZ₅₃(1 − RZ₅₃(RZ₅₃(1 − pₛ) × q)))
    let f1 = one.sub(one.sub(ps, mode).mul(r.penalty, mode), mode);
    // spec: S-0403 F4 (f₂: both limits under a deficit, the lower one only at full power)
    let f2 = if ps.lt(one) {
        f1.max(r.min_speed).min(r.max_speed)
    } else {
        f1.max(r.min_speed)
    };
    // spec: S-0403 F4 (f = f₂, or the single nearest 0.01 when f₂ = 0)
    let speed = if f2.eq_num(F64::ZERO) {
        r.zero_speed
    } else {
        f2
    };
    // spec: S-0403 F4 (T₃ = trunc(RZ₅₃(T₂ ÷ f)))
    let t3 = f(t2).div(speed, mode).to_i32_common();
    // spec: S-0403 F5 (T₄: × g once per factory beyond the first, truncating each time, when g > 0 and n ≥ 2)
    let mut t4 = t3;
    // spec: S-0403 F5 (only when n ≥ 2)
    let several = x.factories >= 2;
    if r.multiple_factory.gt(F64::ZERO) && several {
        for _ in 1..x.factories {
            t4 = f(t4).mul(r.multiple_factory, mode).to_i32_common();
        }
    }
    // spec: S-0403 F6 (a wall: T = trunc(RZ₅₃(T₄ × w)))
    if x.wall {
        f(t4).mul(r.wall, mode).to_i32_common()
    } else {
        t4
    }
}

/// The step period *R* of a build time (S-0403 F7).
pub fn period_of(t: i32, cap: bool) -> i32 {
    // spec: S-0403 F7 (R = min(255, max(1, T ÷ 54)), integer quotient; a negative T gives 1)
    let q = (t / STEPS).max(1);
    // spec: S-0403 Edge cases (quirks.build_step_period_cap: off, no upper limit)
    if cap { q.min(MAX_PERIOD) } else { q }
}

/// The number of factory buildings of the item's kind that the house owns
/// on the map (S-0403 claim 6).
pub fn factory_count(w: &World, house: HouseId, t: TypeRef) -> i32 {
    let r = w.rules();
    let naval_item = naval(w, t);
    let all = r.quirks.multiple_factory_counts_all;
    w.objects
        .values()
        .filter(|b| b.kind == Kind::Building && b.on_map && b.owner == Some(house))
        .filter(|b| {
            let Some(br) = r.buildings.get(b.type_index as usize) else {
                return false;
            };
            if br.factory.map(|f| f.kind()) != Some(t.kind) {
                return false;
            }
            if all {
                // spec: S-0403 claim 6 (naval and ground vehicle factories counted apart; every other kind counts all its factories)
                t.kind != Kind::Vehicle || br.naval == naval_item
            } else {
                // spec: S-0403 Edge cases (quirks.multiple_factory_counts_all off: only factories that could make the item in S-0404's sense, power and docks ignored: not sold or queued to be sold, owner-country overlap, naval or ground kind and the build-limit answer with its production allowance, not the full tech-tree test)
                tech::could_make_limit_only(w, b, t)
            }
        })
        .count() as i32
}

/// The build time *T* of type `t` for a house at this moment (S-0403
/// claims 2 to 7).
pub fn build_time(w: &World, house: HouseId, t: TypeRef) -> i32 {
    let r = w.rules();
    let p = tp(w, t);
    let lk = line_kind(w, t).unwrap_or(LineKind::Infantry);
    let country = w
        .houses
        .get(usize::from(house.0))
        .and_then(|h| r.econ.production.countries.get(h.country as usize))
        .map_or(F64::from_i64(1, r.mode), |c| c.time[lk.factor()]);
    build_time_of(
        &r.econ.production,
        TimeInputs {
            cost: p.cost,
            country,
            multiplier: p.multiplier,
            power: super::power::fraction(w, house),
            factories: factory_count(w, house, t),
            wall: t.kind == Kind::Building && p.wall,
        },
        r.mode,
    )
}

/// The step period *R* of type `t` for a house at this moment (S-0403
/// claims 1 and 8).
pub fn period(w: &World, house: HouseId, t: TypeRef) -> i32 {
    period_of(
        build_time(w, house, t),
        w.rules().quirks.build_step_period_cap,
    )
}

/// The house has worked out its power again (S-0405 claim 11, called from
/// `crate::econ::power::house_update`): every line of the house works out
/// its step period again from the power fraction `crate::econ::power::fraction`
/// (S-0403 claim 8); the step under way keeps its period.
pub fn power_changed(w: &mut World, house: HouseId) {
    for li in 0..w.econ.lines.lines.len() {
        let l = &w.econ.lines.lines[li];
        if l.house != house {
            continue;
        }
        let Some(t) = l.item.as_ref().map(|i| i.t) else {
            continue;
        };
        let r = period(w, house, t);
        if let Some(i) = w.econ.lines.lines[li].item.as_mut() {
            // spec: S-0403 claim 8 (the new R applies from the next step on)
            i.period = r;
        }
    }
}

// ---------------------------------------------------------------------
// S-0404: price
// ---------------------------------------------------------------------

/// The cost bonus product *b* for a kind (S-0404 claim 2, F1): over the
/// house's bonus buildings in the order it acquired them, each partial
/// product narrowed to a single.
fn bonus_product(w: &World, house: HouseId, factor: usize) -> F64 {
    let r = w.rules();
    let mode = r.mode;
    let mut b = F64::from_i64(1, mode);
    let Some(h) = w.houses.get(usize::from(house.0)) else {
        return b;
    };
    for &id in &h.econ.acquired {
        let Some(o) = w.obj(id) else { continue };
        if o.owner != Some(house) || o.kind != Kind::Building {
            continue;
        }
        let bonus = r
            .econ
            .production
            .buildings
            .get(o.type_index as usize)
            .and_then(|p| p.bonus);
        if let Some(beta) = bonus {
            // spec: S-0404 F1 (b = ρ₂₄(… ρ₂₄(ρ₂₄(1 × β₁) × β₂) … × βⱼ); ρ₂₄ is RZ₂₄ with quirks.chop_rounding on, RN₂₄ off, through the rules rounding mode)
            b = b.mul(beta[factor], mode).to_f32(mode).to_f64();
        }
    }
    b
}

/// The country cost factor *a* and the cost bonus product *b* of a house
/// for a kind (S-0404 claim 2, F1).
fn factors_of(w: &World, house: HouseId, lk: LineKind) -> (F64, F64) {
    let r = w.rules();
    let mode = r.mode;
    let a = w
        .houses
        .get(usize::from(house.0))
        .and_then(|h| r.econ.production.countries.get(h.country as usize))
        .map_or(F64::from_i64(1, mode), |c| c.cost[lk.factor()]);
    (a, bonus_product(w, house, lk.factor()))
}

/// The factors *a* and *b* (S-0404 claim 2) of a house for type `t`'s
/// kind, as they stand now, which S-0408's refund (claim 1, F1) uses; 1 and
/// 1 for a type with no line kind.
pub fn cost_factors(w: &World, house: HouseId, t: TypeRef) -> (F64, F64) {
    match line_kind(w, t) {
        Some(lk) => factors_of(w, house, lk),
        None => {
            let one = F64::from_i64(1, w.rules().mode);
            (one, one)
        }
    }
}

/// trunc(ρ₅₃(ρ₅₃(C × b) × a)) for a cost and a type's kind (S-0404 F1).
fn base_price(w: &World, house: HouseId, cost: i32, lk: LineKind) -> i32 {
    let mode = w.rules().mode;
    let (a, b) = factors_of(w, house, lk);
    // spec: S-0404 F1 (P = trunc(ρ₅₃(ρ₅₃(C × b) × a)); ρ₅₃ is RZ₅₃ with quirks.chop_rounding on, RN₅₃ off; trunc toward zero either way)
    F64::from_i64(i64::from(cost), mode)
        .mul(b, mode)
        .mul(a, mode)
        .to_i32_common()
}

/// Whether S-0408's pad deduction applies to a building type (S-0404 claim
/// 2): the separate-aircraft flag off and the type the first dock type of
/// the first pad aircraft type. An empty list, or a first pad aircraft with
/// no dock, gives none (S-0404 Edge cases, the departure).
pub fn pad_term_applies(w: &World, b: u32) -> bool {
    let p = &w.rules().econ.production;
    if p.separate_aircraft {
        return false;
    }
    p.pad_aircraft
        .first()
        .and_then(|&a| p.aircraft.get(a as usize))
        .and_then(|a| a.first_dock)
        == Some(b)
}

/// The pad aircraft at position `i` (0 or 1), if the list has one.
pub fn pad(w: &World, i: usize) -> Option<TypeRef> {
    w.rules()
        .econ
        .production
        .pad_aircraft
        .get(i)
        .map(|&index| TypeRef {
            kind: Kind::Aircraft,
            index,
        })
}

/// A type's cost field *C* (S-0403 claim 2).
pub fn cost_field(w: &World, t: TypeRef) -> i32 {
    tp(w, t).cost
}

/// A building type's free unit type (S-0404 claim 2).
pub fn free_unit(w: &World, b: u32) -> Option<TypeRef> {
    tp(
        w,
        TypeRef {
            kind: Kind::Building,
            index: b,
        },
    )
    .free_unit
}

/// The price *P* of an item of type `t` for a house (S-0404 claim 2, F1).
pub fn price(w: &World, house: HouseId, t: TypeRef) -> i32 {
    let Some(lk) = line_kind(w, t) else { return 0 };
    let p = tp(w, t);
    if t.kind != Kind::Building {
        return base_price(w, house, p.cost, lk);
    }
    let own = base_price(w, house, super::sell::adjusted_cost(w, t.index), lk);
    // spec: S-0404 claim 2 (the pad term (P₁ + P₂) ÷ 2 when the pad deduction applies; a missing entry counts as price 0)
    let pad_term = if pad_term_applies(w, t.index) {
        let pp = |i| pad(w, i).map_or(0, |a| price(w, house, a));
        (pp(0) + pp(1)) / 2
    } else {
        0
    };
    // spec: S-0404 claim 2 (Pₑ, the price of the free unit type for the same house)
    let free = p.free_unit.map_or(0, |f| price(w, house, f));
    let total = own + pad_term + free;
    // spec: S-0404 claim 2 (limited below by 0 only when the type names a free unit type)
    if p.free_unit.is_some() {
        total.max(0)
    } else {
        total
    }
}

/// A house's available credits *A* (S-0404 claim 4, F3): in a skirmish
/// nothing fills the house storage, so *A* = *Y*, the credit balance.
pub fn available_credits(w: &World, house: HouseId) -> i32 {
    // spec: S-0404 claim 4 (A = Y in a skirmish; our engine keeps the credit balance only)
    w.houses.get(usize::from(house.0)).map_or(0, |h| h.credits)
}

/// A house pays `amount` credits (S-0404 claim 4): the credit balance drops
/// by it, with no lower limit (claim 6).
pub fn pay(w: &mut World, house: HouseId, amount: i32) {
    if let Some(h) = w.houses.get_mut(usize::from(house.0)) {
        // spec: S-0404 claim 4 (a paid step reduces the credit balance by its payment)
        h.credits = h.credits.saturating_sub(amount);
    }
}

/// The step payment *x*ₖ for step *k* with balance owed *B* (S-0404 F2).
pub fn payment(k: i32, owed: i32) -> i32 {
    if k <= LAST_PAYING_STEP {
        // spec: S-0404 F2 (xₖ = min(B, B ÷ (54 − k)) for k = 1 … 53)
        owed.min(owed / (STEPS - k))
    } else {
        // spec: S-0404 claim 4 (x₅₄ = B)
        owed
    }
}

// ---------------------------------------------------------------------
// Lines
// ---------------------------------------------------------------------

/// The index of a house's line of a kind, if it exists.
pub fn find_line(w: &World, house: HouseId, kind: LineKind) -> Option<usize> {
    w.econ
        .lines
        .lines
        .iter()
        .position(|l| l.house == house && l.kind == kind)
}

/// A house's line of a type's kind.
pub fn line_of(w: &World, house: HouseId, t: TypeRef) -> Option<&Line> {
    let lk = line_kind(w, t)?;
    find_line(w, house, lk).map(|i| &w.econ.lines.lines[i])
}

/// The copies of a type in its line: the item and the queue (S-0407 claim
/// 16, S-0411 claim 7).
pub fn copies_in_line(w: &World, house: HouseId, t: TypeRef) -> i32 {
    line_of(w, house, t).map_or(0, |l| {
        l.queue.iter().filter(|&&q| q == t).count() as i32
            + i32::from(l.item.as_ref().is_some_and(|i| i.t == t))
    })
}

/// The object of a house's done item of type `t`, which a place order puts
/// on the map (S-0404 claim 7; the placement of S-0406 places this object
/// and creates none).
pub fn item_object(w: &World, house: HouseId, t: TypeRef) -> Option<ObjectId> {
    line_of(w, house, t)
        .and_then(|l| l.item.as_ref())
        .filter(|i| i.t == t && i.done())
        .map(|i| i.object)
}

/// Whether an object is the object of a line's item (S-0404 claim 11).
pub fn is_item_object(w: &World, id: ObjectId) -> bool {
    w.econ
        .lines
        .lines
        .iter()
        .any(|l| l.item.as_ref().is_some_and(|i| i.object == id))
}

/// Whether an item's object still exists and is alive: a defeat's hit can
/// kill it in production (S-0414 claim 15).
fn item_alive(w: &World, id: ObjectId) -> bool {
    w.obj(id).is_some_and(|o| o.health > 0)
}

/// Removes the lines with neither an item nor a queued type (S-0404 claim
/// 1: such a line ceases to exist).
fn cleanup(w: &mut World) {
    w.econ
        .lines
        .lines
        .retain(|l| l.item.is_some() || !l.queue.is_empty());
}

/// A free dock (S-0404 Terms): at least one of the building's contact
/// places is empty, or no aircraft is docked with it at all. The places
/// held are the aircraft in contact with it, by release or by pad
/// reservation (S-0501 claim 23, free-dock accounting).
pub fn dock_free(w: &World, b: ObjectId) -> bool {
    let held = w
        .objects
        .values()
        .filter(|o| o.kind == Kind::Aircraft && o.contact == Some(b))
        .count() as u32;
    // spec: S-0404 Terms (a free dock: a docking place empty, or no aircraft docked with it at all)
    held == 0 || held < super::release::contact_places(w, b)
}

fn switched_on(b: &Object) -> bool {
    !b.econ.building.switched_off
}

/// The house's factory buildings, in object order, that pass `f`.
fn factories(w: &World, house: HouseId, f: impl Fn(&Object) -> bool) -> Vec<ObjectId> {
    w.objects
        .values()
        .filter(|b| b.kind == Kind::Building && b.owner == Some(house) && f(b))
        .map(|b| b.id)
        .collect()
}

/// Starts an item of `t` on line `li` (S-0404 claim 3; claim 10 when
/// `forced`): its object is created (one creation draw, claim 11).
fn start_item(w: &mut World, li: usize, t: TypeRef, forced: bool) {
    let house = w.econ.lines.lines[li].house;
    // spec: S-0404 claim 11 (starting an item creates its object: one creation draw, taken at the start's causing action, so same-tick starts draw in order of application, no value reserved ahead)
    let object = crate::objects::create(w, t.kind, t.index, Some(house));
    let p = price(w, house, t);
    let r = period(w, house, t);
    let credits = available_credits(w, house);
    // spec: S-0404 claim 10 and quirks.forced_start_player_hold (a forced start is held when A ≥ P ÷ 54)
    let hold = if forced && w.rules().quirks.forced_start_player_hold && credits >= p / STEPS {
        Hold::Player
    } else {
        Hold::None
    };
    if let Some(o) = w.obj_mut(object) {
        // spec: S-0402 Terms (the start price, recorded on the produced object)
        o.econ.start_price = Some(p);
    }
    let tick = w.tick;
    w.econ.lines.lines[li].item = Some(Item {
        t,
        object,
        progress: 0,
        // spec: S-0404 claim 3 (progress 0, B = P at the start)
        owed: p,
        start_price: p,
        period: r,
        // spec: S-0403 claim 1 (the first step R ticks after the start)
        due: tick + r,
        hold,
        waiting: false,
        remaining: r,
        done_tick: None,
    });
}

/// Starts the first queued type after a cancel or a release (S-0404 claims
/// 6, 7 and 10), with the forced start of claim 10. `exclude`: a building
/// about to leave the map.
fn start_next(w: &mut World, li: usize, exclude: Option<ObjectId>) {
    let house = w.econ.lines.lines[li].house;
    while !w.econ.lines.lines[li].queue.is_empty() {
        let head = w.econ.lines.lines[li].queue.remove(0);
        // spec: S-0404 claim 10 (the forced start tests the factory conditions and the build-limit answer with the production allowance, not the full tech tree or S-0407 claim 16 again) and S-0411 claim 14 (not a factory queued to be sold)
        let can = factories(w, house, |b| {
            Some(b.id) != exclude && tech::could_start(w, b, head)
        });
        if !can.is_empty() {
            // spec: S-0404 claim 10 (started even when every factory that could make it is switched off or has no free dock)
            let ready = can.iter().any(|&b| {
                w.obj(b).is_some_and(switched_on)
                    && (head.kind != Kind::Aircraft || dock_free(w, b))
            });
            start_item(w, li, head, !ready);
            return;
        }
        // spec: S-0404 Edge cases (quirks.unstartable_queue_head_stalls: the head that cannot start leaves the queue, nothing starts)
        if w.rules().quirks.unstartable_queue_head_stalls {
            return;
        }
    }
}

/// Cancels the item of line `li` with the refund of S-0404 claim 6; its
/// object is deleted.
fn cancel_item(w: &mut World, li: usize) {
    let house = w.econ.lines.lines[li].house;
    let Some(item) = w.econ.lines.lines[li].item.take() else {
        return;
    };
    // spec: S-0404 F4 (refund = P′ − B; quirks.refund_at_current_price off: P − B, the amount paid)
    let refund = if w.rules().quirks.refund_at_current_price {
        price(w, house, item.t) - item.owed
    } else {
        item.start_price - item.owed
    };
    let h = &mut w.houses[usize::from(house.0)];
    // spec: S-0404 claim 6 (the refund is added with no upper or lower limit)
    h.credits = h.credits.saturating_add(refund);
    delete_item_object(w, item.object);
}

/// Deletes an item's object, which is off the map (S-0404 claim 6), after
/// S-0407's event (g).
pub(crate) fn delete_item_object(w: &mut World, id: ObjectId) {
    if w.obj(id).is_some_and(|o| !o.on_map) {
        tech::object_ends(w, id);
        drop_creation_set(w, id);
        w.objects.remove(&id);
        // spec: S-0414 claim 15 (an object that ceases to exist at once, for the cancellation exception)
        super::victory::object_deleted(w, id);
    }
}

/// What an item's object brought with it at its creation goes with it
/// when it is deleted unplaced (#156): its docked slot objects, its
/// hidden slaves and its mind-control record.
fn drop_creation_set(w: &mut World, id: ObjectId) {
    // spec: S-0506 claim 13 (the spawner removed: its docked slot objects are removed silently)
    crate::spawn::removing(w, id);
    // spec: S-0402 claim 19 (the slaves were created with the master) and S-0312 r71 claim 31 (a cancelled production master: the group ends as a removal without a killing hit; every slave is destroyed and removed at the cancellation, a loss each to its house, no kill and no score)
    super::slaves::removing(w, id);
    // spec: S-0601 claims 1 and 10 (a controller has its record from its creation; a controller removed from the game releases its victims and its record goes)
    crate::mind::removing(w, id);
}

/// Resumes a held item (S-0404 claim 5; the lift of an automatic hold,
/// claim 8): the step period is worked out again and the next step comes a
/// full *R* later (S-0403 claim 8).
fn resume(w: &mut World, li: usize) {
    let house = w.econ.lines.lines[li].house;
    let Some(t) = w.econ.lines.lines[li].item.as_ref().map(|i| i.t) else {
        return;
    };
    let r = period(w, house, t);
    let tick = w.tick;
    let restart = w.rules().quirks.resume_restarts_step;
    if let Some(i) = w.econ.lines.lines[li].item.as_mut() {
        i.hold = Hold::None;
        i.period = r;
        // spec: S-0403 claim 8 and quirks.resume_restarts_step (the next step a full R later; off: the ticks that remained)
        i.due = tick + if restart { r } else { i.remaining };
    }
}

fn put_on_hold(w: &mut World, li: usize, hold: Hold) {
    let tick = w.tick;
    if let Some(i) = w.econ.lines.lines[li].item.as_mut() {
        i.hold = hold;
        i.remaining = (i.due - tick).max(1);
    }
}

/// Row 6 of S-0203: every production line, in creation order (S-0404
/// claim 1), makes its step when one is due (S-0403 claim 1, S-0404
/// claim 4).
pub fn part(w: &mut World) {
    let tick = w.tick;
    // spec: S-0404 claim 1 (all lines of all houses in creation order)
    for li in 0..w.econ.lines.lines.len() {
        let house = w.econ.lines.lines[li].house;
        let Some(item) = w.econ.lines.lines[li].item.as_ref() else {
            continue;
        };
        if !item.running() || tick < item.due {
            continue;
        }
        let k = item.progress + 1;
        let x = payment(k, item.owed);
        let credits = available_credits(w, house);
        let Some(item) = w.econ.lines.lines[li].item.as_mut() else {
            continue;
        };
        // spec: S-0403 State and ordering (the next step or retry is due R ticks later, with the R in force)
        item.due = tick + item.period;
        // spec: S-0404 claim 4 (the step is refused when xₖ > A, a signed comparison; no partial payment)
        if x > credits {
            item.waiting = true;
            continue;
        }
        item.waiting = false;
        item.owed -= x;
        item.progress = k;
        if item.done() {
            // spec: S-0404 claim 7 (at step 54 the item is done)
            item.done_tick = Some(tick);
        }
        pay(w, house, x);
    }
}

/// Applies a build order (S-0404 claim 3).
pub fn build(w: &mut World, house: HouseId, t: TypeRef) {
    let Some(lk) = line_kind(w, t) else { return };
    // spec: S-0404 claim 3 (applied only with a switched-on factory that could make it without the tech-tree rule and, for an aircraft, a free dock, and the build-limit answer with the production allowance; no full tech-tree test, S-0407 claim 13; a resume too) and S-0411 claim 14 (not a factory queued to be sold)
    let ok = !factories(w, house, |b| {
        switched_on(b)
            && tech::could_start(w, b, t)
            && (t.kind != Kind::Aircraft || dock_free(w, b.id))
    })
    .is_empty();
    if !ok {
        return;
    }
    let Some(li) = find_line(w, house, lk) else {
        // spec: S-0404 claim 3 (idle line: the item starts at once) and claim 1 (a line created again counts as new)
        w.econ.lines.lines.push(Line {
            house,
            kind: lk,
            item: None,
            queue: Vec::new(),
        });
        let li = w.econ.lines.lines.len() - 1;
        start_item(w, li, t, false);
        return;
    };
    let line = &w.econ.lines.lines[li];
    if let Some(i) = line.item.as_ref().filter(|i| i.t == t) {
        if i.hold == Hold::Player && !i.done() {
            // spec: S-0404 claim 3 (a line on a player hold whose item is of the ordered type: resumed)
            resume(w, li);
            return;
        }
        if i.done() {
            // spec: S-0404 claim 3 (a done item of the ordered type: nothing added)
            return;
        }
    }
    let max = w.rules().econ.production.max_queued;
    // spec: S-0404 claim 3 (the queue limit counts queued types; 0 or below refuses every queued type)
    if line.queue.len() as i32 >= max {
        queue_refused_cue(w, house, t);
        return;
    }
    // spec: S-0404 claim 3 and S-0407 claim 16 (the queue-time limit)
    if tech::queue_refused(w, house, t, copies_in_line(w, house, t)) {
        queue_refused_cue(w, house, t);
        return;
    }
    w.econ.lines.lines[li].queue.push(t);
}

/// The cue of a queue request refused as it is applied (S-0404 claim 3):
/// the presentation plays the game-wide refusal sound for the house's own
/// player only.
fn queue_refused_cue(w: &mut World, house: HouseId, t: TypeRef) {
    // spec: S-0404 claim 3; S-0701 claim 24 (a production request refused at application by the queue limit or the build limit: the game-wide refusal sound, the local player's house only)
    w.effects.push(crate::combat::effects::Effect::Sound {
        sound: crate::combat::effects::Sound::QueueRefused,
        of: Some(t),
        house: Some(house),
        pos: None,
    });
}

/// Applies a hold order (S-0404 claim 5).
pub fn hold(w: &mut World, house: HouseId, t: TypeRef) {
    let Some(lk) = line_kind(w, t) else { return };
    let Some(li) = find_line(w, house, lk) else {
        return;
    };
    // spec: S-0404 claim 5 (a running or waiting line goes on a player hold; a held line or a done item: nothing)
    if w.econ.lines.lines[li]
        .item
        .as_ref()
        .is_some_and(Item::running)
    {
        put_on_hold(w, li, Hold::Player);
    }
}

/// Applies a cancel order (S-0404 claim 6); `all`: a cancel-all order.
pub fn cancel(w: &mut World, house: HouseId, t: TypeRef, all: bool) {
    let Some(lk) = line_kind(w, t) else { return };
    let Some(li) = find_line(w, house, lk) else {
        return;
    };
    let line = &mut w.econ.lines.lines[li];
    let item_is = line.item.as_ref().is_some_and(|i| i.t == t);
    if all {
        // spec: S-0404 claim 6 (cancel-all: every queued copy removed, an item of the type cancelled)
        line.queue.retain(|&q| q != t);
        if item_is {
            cancel_item(w, li);
            start_next(w, li, None);
        }
    } else if let Some(p) = line.queue.iter().position(|&q| q == t) {
        // spec: S-0404 claim 6 (a queued copy: the first one removed, nothing else)
        line.queue.remove(p);
    } else if item_is {
        // spec: S-0404 claim 6 (the item cancelled with the refund; the first queued type starts)
        cancel_item(w, li);
        start_next(w, li, None);
    }
    cleanup(w);
}

/// Applies a place order (S-0404 claim 7, S-0406): a done unit is released
/// (no cell), a done building is put on the map at `cell`.
pub fn place(w: &mut World, house: HouseId, t: TypeRef, cell: Option<CellXY>) {
    let Some(lk) = line_kind(w, t) else { return };
    let Some(li) = find_line(w, house, lk) else {
        return;
    };
    let done = w.econ.lines.lines[li]
        .item
        .as_ref()
        .is_some_and(|i| i.t == t && i.done());
    if !done {
        return;
    }
    if lk.unit() {
        release(w, house, t);
        return;
    }
    let Some(cell) = cell else { return };
    let alive = w.econ.lines.lines[li]
        .item
        .as_ref()
        .is_some_and(|i| item_alive(w, i.object));
    if !alive {
        // Q-0193 (interim: a building item killed in production, S-0414 claim 15, waits for its line's deletion)
        return;
    }
    // spec: S-0404 claim 7 (a done building waits for a place order with a cell; S-0406 places it)
    if !super::placement::place_building(w, house, t, cell) {
        return;
    }
    let Some(li) = find_line(w, house, lk) else {
        return;
    };
    let Some(item) = w.econ.lines.lines[li].item.take() else {
        return;
    };
    if !w.rules().quirks.placed_buildings_not_ever_built {
        // spec: S-0407 claim 10 (quirks.placed_buildings_not_ever_built off: a placed building raises its ever-built count)
        tech::count_built(w, house, t);
    }
    // An upgrade enters no play as a building (S-0406 claim 12): its object goes.
    if w.obj(item.object).is_some_and(|o| !o.on_map) {
        drop_creation_set(w, item.object);
        w.objects.remove(&item.object);
    }
    // spec: S-0404 claim 7 (when the placement succeeds, the line ceases to exist)
    start_next(w, li, None);
    cleanup(w);
}

/// The factory building that releases a done unit (S-0404 claim 7): among
/// the switched-on buildings that could make it without the tech-tree rule
/// (with a free dock, for an aircraft; the fallback without it), the one
/// marked primary, else the last the house acquired.
pub fn releasing_building(w: &World, house: HouseId, t: TypeRef) -> Option<ObjectId> {
    // spec: S-0404 Terms (could release: switched on, could make without the tech-tree rule) and S-0411 claim 14 (not a factory queued to be sold)
    let base = |b: &Object| switched_on(b) && tech::could_make(w, b, t);
    let mut cands = factories(w, house, |b| {
        base(b) && (t.kind != Kind::Aircraft || dock_free(w, b.id))
    });
    if cands.is_empty() && t.kind == Kind::Aircraft {
        // spec: S-0404 claim 7 (an aircraft without a free dock anywhere: the same choice without the dock condition)
        cands = factories(w, house, base);
    }
    // spec: S-0404 claim 7 (the one marked primary)
    if let Some(&p) = cands
        .iter()
        .find(|&&b| w.obj(b).is_some_and(|o| o.econ.building.primary))
    {
        return Some(p);
    }
    // spec: S-0404 claim 7 (otherwise the last in the order the house acquired its buildings)
    let acquired = &w.houses.get(usize::from(house.0))?.econ.acquired;
    acquired.iter().rev().find(|b| cands.contains(b)).copied()
}

/// The release of a done unit (S-0404 claim 7).
fn release(w: &mut World, house: HouseId, t: TypeRef) {
    let Some(lk) = line_kind(w, t) else { return };
    let Some(li) = find_line(w, house, lk) else {
        return;
    };
    let Some(object) = w.econ.lines.lines[li]
        .item
        .as_ref()
        .filter(|i| i.t == t && i.done())
        .map(|i| i.object)
    else {
        return;
    };
    if !item_alive(w, object) {
        // spec: S-0404 claim 13 (a unit that cannot be put on the map: failed, cancelled with the refund as claim 7 says)
        // Q-0193 (interim: an item killed by a destruction hit counts as such a unit)
        cancel_item(w, li);
        start_next(w, li, None);
        cleanup(w);
        return;
    }
    // spec: S-0404 claim 7 (no releasing building: the order does nothing and the item stays done)
    let Some(factory) = releasing_building(w, house, t) else {
        return;
    };
    // spec: S-0404 claim 7 (the chosen building releases the unit by claim 13)
    let outcome = super::release::release(w, factory, object);
    let Some(li) = find_line(w, house, lk) else {
        return;
    };
    if outcome != super::release::Outcome::Released {
        // spec: S-0404 claim 7 (busy or failed: the item is cancelled with the refund of claim 6 and the line goes on as after that cancel; nothing is retried)
        cancel_item(w, li);
        start_next(w, li, None);
        cleanup(w);
        return;
    }
    w.econ.lines.lines[li].item = None;
    // spec: S-0407 claim 10 (a unit released by a factory raises its type's ever-built count)
    tech::count_built(w, house, t);
    // spec: S-0407 claim 12 (f) (a unit released by a factory calls for a recheck)
    tech::recheck_due(w, house);
    // spec: S-0404 claim 7 (the line's next queued type starts at once, with claim 10's start conditions)
    start_next(w, li, None);
    cleanup(w);
}

/// The factory check of S-0404 claim 8 for a house's line of `lk`.
/// `exclude`: a building about to leave the map.
pub fn check_line(w: &mut World, house: HouseId, lk: LineKind, exclude: Option<ObjectId>) {
    let Some(li) = find_line(w, house, lk) else {
        return;
    };
    let can = |w: &World, t: TypeRef, on: bool| {
        !factories(w, house, |b| {
            // spec: S-0404 claim 8 (could make: the build-limit answer with the production allowance, not the full tech-tree; a factory building being sold or queued to be sold counts in no test, Terms)
            Some(b.id) != exclude && (!on || switched_on(b)) && tech::could_make_limit_only(w, b, t)
        })
        .is_empty()
    };
    // spec: S-0404 claim 8 (queue: no queued type that no factory building could make)
    let queue = w.econ.lines.lines[li].queue.clone();
    let kept: Vec<TypeRef> = queue.into_iter().filter(|&t| can(w, t, false)).collect();
    w.econ.lines.lines[li].queue = kept;
    let Some(item) = w.econ.lines.lines[li].item.clone() else {
        cleanup(w);
        return;
    };
    if !can(w, item.t, false) {
        // spec: S-0404 claim 8 (an item no factory could make: cancelled with the refund, the line goes on with claim 10's forced start)
        cancel_item(w, li);
        start_next(w, li, exclude);
    } else if item.running() && !can(w, item.t, true) {
        // spec: S-0404 claim 8 (running or waiting, none switched on: an automatic hold)
        put_on_hold(w, li, Hold::Auto);
    } else if item.hold == Hold::Auto && can(w, item.t, true) {
        // spec: S-0404 claim 8 (an automatic hold is lifted, with a full step period as for a resume)
        resume(w, li);
    }
    cleanup(w);
}

/// The lines of a house that a factory building's kind concerns (S-0404
/// claim 8): for a vehicle factory the naval or the ground line by its own
/// naval flag; for a factory of buildings the ordinary-building line, and
/// the defence line with `quirks.defence_line_partial_factory_check` off.
fn lines_for(w: &World, b: &Object) -> Vec<LineKind> {
    let r = w.rules();
    let Some(br) = r.buildings.get(b.type_index as usize) else {
        return Vec::new();
    };
    match br.factory.map(|f| f.kind()) {
        Some(Kind::Infantry) => vec![LineKind::Infantry],
        Some(Kind::Vehicle) if br.naval => vec![LineKind::Naval],
        Some(Kind::Vehicle) => vec![LineKind::Ground],
        Some(Kind::Aircraft) => vec![LineKind::Aircraft],
        // spec: S-0404 claim 8 and quirks.defence_line_partial_factory_check (only the ordinary-building line)
        Some(Kind::Building) if r.quirks.defence_line_partial_factory_check => {
            vec![LineKind::Building]
        }
        Some(Kind::Building) => vec![LineKind::Building, LineKind::Defence],
        _ => Vec::new(),
    }
}

/// S-0404 claim 8: one of a house's factory buildings was placed, is about
/// to leave the map (`leaving`), changed owner or was switched: the lines
/// of its kind are checked at once, for the house given.
pub fn factory_event(w: &mut World, house: HouseId, b: ObjectId, leaving: bool) {
    let Some(o) = w.obj(b).cloned() else { return };
    let exclude = leaving.then_some(b);
    for lk in lines_for(w, &o) {
        check_line(w, house, lk, exclude);
    }
}

/// S-0404 claim 8, last rule: a factory building removed for good cancels
/// every line of its kind (the defence line included) whose item no other
/// factory building could make without the tech-tree rule, whatever its
/// power switch and docks.
pub fn factory_removed(w: &mut World, house: HouseId, b: ObjectId) {
    let Some(kind) = w
        .obj(b)
        .and_then(|o| w.rules().buildings.get(o.type_index as usize))
        .and_then(|br| br.factory)
        .map(|f| f.kind())
    else {
        return;
    };
    let mut li = 0;
    while li < w.econ.lines.lines.len() {
        let l = &w.econ.lines.lines[li];
        if l.house == house
            && l.kind.factory_kind() == kind
            && let Some(t) = l.item.as_ref().map(|i| i.t)
            // spec: S-0404 claim 8 and Terms (a factory building being sold or queued to be sold counts in no test)
            && factories(w, house, |x| x.id != b && tech::could_make(w, x, t)).is_empty()
        {
            // spec: S-0404 claim 8 (removed for good: no other factory could make the item: cancelled as after a cancel)
            cancel_item(w, li);
            start_next(w, li, Some(b));
        }
        li += 1;
    }
    cleanup(w);
}

/// Whether a house is a player house (the neutral and special houses never
/// build, S-0404 claim 9).
pub fn player(w: &World, house: HouseId) -> bool {
    w.houses
        .get(usize::from(house.0))
        .is_some_and(|h| h.role == HouseRole::Player)
}

/// The tech-tree answer of a type for a house, re-exported for the sidebar.
pub fn answer(w: &World, house: HouseId, t: TypeRef) -> Answer {
    tech::answer(w, house, t)
}
