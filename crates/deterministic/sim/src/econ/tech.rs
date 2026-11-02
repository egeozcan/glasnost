//! The tech tree (S-0407): what a house may build. The tech-tree answer of
//! a type for a house (claims 2 to 10), the counts it reads, the taken-up
//! set and its rechecks (claims 1 and 12), stolen tech (claim 6), the
//! queue-time limit (claim 16) and the secret labs at game start (claim 14).

use std::collections::{BTreeMap, BTreeSet};

use glasnost_num::Rounding;
use glasnost_rules::schema::Ruleset;
use serde::{Deserialize, Serialize};

use super::TypeRef;
use crate::rules::{Behaviour, Kind};
use crate::world::{HouseId, HouseRole, Object, ObjectId, World};

/// A house's offer, counts and flags (S-0407).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HouseTech {
    /// The taken-up set (S-0407 Terms, claim 1 (c)): the types offered to
    /// the house, as of its last recheck. It is also the set of the house's
    /// build icons (S-0407 Terms, S-0411 claim 2); for a house other than
    /// the local player no icon is drawn.
    pub taken_up: BTreeSet<TypeRef>,
    /// A recheck is due at the house's next house update (S-0407 claim 12).
    pub recheck_due: bool,
    /// The first, second and third stolen-tech flags (S-0407 claim 6).
    pub stolen: [bool; 3], // spec: S-0407 Terms (three stolen-tech flags)
    /// Ever-built counts by type (S-0407 Terms, claim 10).
    pub ever_built: BTreeMap<TypeRef, i32>,
    /// The icons of the taken-up set by sidebar tab, first to fourth, each
    /// in its icon order (S-0004 claim 11, S-0411 claim 13). Kept for every
    /// player house, as if it were the local player (S-0411 claim 13).
    pub icons: [Vec<Icon>; 4], // spec: S-0004 claim 11 (four sidebar tabs)
    /// The pad total (S-0501 claim 30, S-0407 claim 16): a saved running
    /// total of the docking-place counts of the house's placed helipad
    /// buildings, never below 0.
    #[serde(default)]
    pub pad_total: i64,
}

/// An icon in a sidebar tab (S-0004 claim 11): its type and the cost it was
/// sorted by when it was added (items are not re-sorted later).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Icon {
    /// The type.
    pub t: TypeRef,
    /// Its price to the house when it was added (S-0004 claim 11 rule 5).
    pub cost: i32,
}

/// One of the six generic groups (S-0407 Terms, Import mapping).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Group {
    /// `POWER`.
    Power,
    /// `FACTORY`.
    Factory,
    /// `BARRACKS`.
    Barracks,
    /// `RADAR`.
    Radar,
    /// `TECH`.
    Tech,
    /// `PROC`, the refinery group.
    Proc,
}

impl Group {
    /// The group a prerequisite word names, compared ignoring case (S-0407
    /// Import mapping).
    pub fn from_word(w: &str) -> Option<Self> {
        // spec: S-0407 Import mapping (POWER, FACTORY, BARRACKS, RADAR, TECH, PROC, compared ignoring case)
        [
            ("POWER", Group::Power),
            ("FACTORY", Group::Factory),
            ("BARRACKS", Group::Barracks),
            ("RADAR", Group::Radar),
            ("TECH", Group::Tech),
            ("PROC", Group::Proc),
        ]
        .into_iter()
        .find(|(n, _)| n.eq_ignore_ascii_case(w))
        .map(|(_, g)| g)
    }

    fn index(self) -> usize {
        self as usize
    }
}

/// An entry of a prerequisite list (S-0407 claim 8).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Prereq {
    /// A building type (claim 8 (a)).
    Building(u32),
    /// A generic group (claim 8 (b)).
    Group(Group),
    /// An upgrade type (claim 8 (c)).
    Upgrade(u32),
    /// An unrecognised name kept as an unmet entry
    /// (`quirks.prerequisite_names_dropped` off, claim 8 (d)).
    Unmet,
}

/// The tech-tree fields of one type.
#[derive(Clone, Debug, Default)]
pub struct TypeTech {
    /// Tech level (claim 5 (a)).
    pub tech_level: i32,
    /// Required countries: `None` no restriction, else the registration
    /// indices of the known countries named (claim 5).
    pub required: Option<Vec<u32>>,
    /// Forbidden countries, as `required`.
    pub forbidden: Option<Vec<u32>>,
    /// Needs the first, second, third stolen-tech flag (claim 6).
    pub stolen: [bool; 3], // spec: S-0407 claim 6 (a type can need any of the three)
    /// The prerequisite list (claim 8).
    pub prereq: Vec<Prereq>,
    /// The override list: building types (claim 3).
    pub overrides: Vec<u32>,
    /// The build limit *L* (claim 9).
    pub limit: i32,
    /// The don't-score flag (Terms).
    pub dont_score: bool,
    /// The insignificant flag (Terms).
    pub insignificant: bool,
    /// Owner countries (claim 1 (b), claim 11).
    pub owners: Vec<u32>,
    /// A building type whose first superweapon may be disabled from the
    /// setup screen (claim 7).
    pub disableable_sw: bool,
    /// A building type whose second superweapon may be disabled from the
    /// setup screen (claim 7, `quirks.superweapon_setting_tests_first_only` off).
    pub disableable_sw2: bool,
    /// A building type on the tech-lab list (claims 6 and 7).
    pub tech_lab: bool,
    /// A building type with the radar flag (claim 6).
    pub radar: bool,
    /// A building type's power field (claim 6: no power output when 0 or
    /// below).
    pub power: i32,
    /// A building type's base-planning-side field (claim 6).
    pub side: i32,
    /// A building type with the secret-lab flag (claims 4 and 14).
    pub secret_lab: bool,
    /// A lab type's fixed secret type (claim 14).
    pub fixed_secret: Option<TypeRef>,
    /// An aircraft type that must return to a dock (claim 16).
    pub airport_bound: bool,
    /// Docks a building adds to the house's pad total (claim 16): its
    /// docking-place count when it has the helipad flag, else 0.
    pub docks: i32,
    /// A building type that is an upgrade type (S-0406 Terms).
    pub upgrade: bool,
    /// The sidebar tab, 0 to 3 for the first to fourth (S-0004 claim 11).
    pub tab: u8,
    /// The counts-as-aircraft flag (S-0004 claim 11 rule 3; tracking from a
    /// placement in the air, S-0502 claim 29 and S-0501 claim 47).
    pub considered_aircraft: bool,
    /// The naval flag (S-0004 claim 11 rule 3).
    pub naval: bool,
}

/// The typed tech-tree rules (S-0407 Import mapping).
#[derive(Clone, Debug, Default)]
pub struct Rules {
    /// Infantry types.
    pub infantry: Vec<TypeTech>,
    /// Vehicle types.
    pub vehicles: Vec<TypeTech>,
    /// Aircraft types.
    pub aircraft: Vec<TypeTech>,
    /// Building types.
    pub buildings: Vec<TypeTech>,
    /// Members of the six generic groups, in [`Group`] order.
    pub groups: [Vec<u32>; 6], // spec: S-0407 Constants (six generic groups)
    /// The refinery-alternative vehicle type (claim 8 (b)).
    pub proc_alternate: Option<u32>,
    /// The secret infantry, vehicle and building lists, concatenated in that
    /// order (claim 14).
    pub secrets: Vec<TypeRef>,
    /// The pad aircraft types (claim 16).
    pub pad_aircraft: Vec<u32>,
    /// Per country (registration index): the index of its own side, if it
    /// has one (S-0004 claim 9), for the sidebar order (S-0004 claim 11
    /// rule 2).
    pub country_side: Vec<Option<i32>>,
}

impl Rules {
    /// The tech-tree rules of a resolved ruleset.
    pub fn new(r: &Ruleset, mode: Rounding) -> Self {
        let _ = mode;
        let s = &r.sim;
        let g = &s.general.economy;
        let reg = |k: &str| r.registries.get(k).cloned().unwrap_or_default();
        let countries = reg("countries");
        let buildings = reg("buildings");
        let infantry = reg("infantry");
        let vehicles = reg("vehicles");
        let aircraft = reg("aircraft");
        let find = |list: &[String], n: &str| -> Option<u32> {
            // spec: S-0003 claim 14 (names compared with ASCII case folding)
            list.iter()
                .position(|x| x.eq_ignore_ascii_case(n))
                .map(|i| i as u32)
        };
        let country_list = |v: &Option<Vec<String>>| -> Option<Vec<u32>> {
            // spec: S-0407 claim 5 (c) (absent or empty: no restriction; only unknown names: an empty set)
            match v {
                Some(l) if !l.is_empty() => {
                    Some(l.iter().filter_map(|n| find(&countries, n)).collect())
                }
                _ => None,
            }
        };
        let owners = |v: &Option<Vec<String>>| -> Vec<u32> {
            // spec: S-0407 Import mapping (Owner absent: none, never offered)
            v.iter()
                .flatten()
                .filter_map(|n| find(&countries, n))
                .collect()
        };
        let upgrade_of = |b: &str| -> bool {
            s.buildings
                .get(b)
                .is_some_and(|t| t.econ.powers_up_building.is_some())
        };
        let prereqs = |v: &Option<Vec<String>>| -> Vec<Prereq> {
            v.iter()
                .flatten()
                .filter_map(|n| {
                    if let Some(g) = Group::from_word(n) {
                        return Some(Prereq::Group(g));
                    }
                    let Some(b) = find(&buildings, n) else {
                        // spec: S-0407 claim 8 (d) (a name that is neither a group word nor a building type is dropped; quirks.prerequisite_names_dropped off: it remains an unmet entry)
                        return (!r.globals.quirks.prerequisite_names_dropped)
                            .then_some(Prereq::Unmet);
                    };
                    if upgrade_of(&buildings[b as usize]) {
                        Some(Prereq::Upgrade(b))
                    } else {
                        Some(Prereq::Building(b))
                    }
                })
                .collect()
        };
        let overrides = |v: &Option<Vec<String>>| -> Vec<u32> {
            // spec: S-0407 claim 3 (only building type names count; a group word is never met, a departure)
            v.iter()
                .flatten()
                .filter_map(|n| find(&buildings, n))
                .collect()
        };
        let names = |v: &Option<Vec<String>>, list: &[String]| -> Vec<u32> {
            v.iter().flatten().filter_map(|n| find(list, n)).collect()
        };
        let tech_labs = names(&g.build_tech, &buildings);
        let econ_fields = |e: &glasnost_rules::sim_econ::EconFields, t: &mut TypeTech| {
            t.required = country_list(&e.required_houses);
            t.forbidden = country_list(&e.forbidden_houses);
            // spec: S-0407 Import mapping (RequiresStolenAlliedTech, RequiresStolenSovietTech, RequiresStolenThirdTech absent: no)
            t.stolen = [
                e.requires_stolen_allied_tech.unwrap_or(false),
                e.requires_stolen_soviet_tech.unwrap_or(false),
                e.requires_stolen_third_tech.unwrap_or(false),
            ];
            t.prereq = prereqs(&e.prerequisite);
            t.overrides = overrides(&e.prerequisite_override);
            // spec: S-0407 Import mapping (BuildLimit absent: 2³¹ − 1)
            t.limit = e.build_limit.unwrap_or(i32::MAX);
            // spec: S-0004 Import mapping (base-planning side = AIBasePlanningSide) and S-0407 Import mapping (absent: −1)
            t.side = e.ai_base_planning_side.unwrap_or(-1);
        };
        let units = |list: &[String],
                     map: &std::collections::BTreeMap<String, glasnost_rules::sim::UnitType>,
                     tab: u8,
                     air_default: bool|
         -> Vec<TypeTech> {
            list.iter()
                .map(|n| {
                    let u = map.get(n).cloned().unwrap_or_default();
                    let mut t = TypeTech {
                        // spec: S-0407 Import mapping (TechLevel absent: 255)
                        tech_level: u.tech_level.unwrap_or(255),
                        // spec: S-0407 Import mapping (DontScore, Insignificant absent: no)
                        dont_score: u.combat.dont_score.unwrap_or(false),
                        insignificant: u.combat.insignificant.unwrap_or(false),
                        owners: owners(&u.owner),
                        // spec: S-0407 Import mapping (AirportBound absent: no)
                        airport_bound: u.econ.airport_bound.unwrap_or(false),
                        tab,
                        // spec: S-0004 Import mapping (naval = Naval; absent: no)
                        naval: u.combat.naval.unwrap_or(false),
                        // spec: S-0004 Import mapping (counts as aircraft = ConsideredAircraft) and S-0501 Import mapping (absent: yes for an aircraft type, no for other types) and S-0004 r20 Import mapping (the same default; open question 2 resolved by S-0501)
                        considered_aircraft: u.econ.considered_aircraft.unwrap_or(air_default),
                        ..Default::default()
                    };
                    econ_fields(&u.econ, &mut t);
                    t
                })
                .collect()
        };
        // spec: S-0004 claim 11 (infantry: the third tab; vehicles and aircraft: the fourth)
        let infantry_t = units(&infantry, &s.infantry, 2, false);
        // spec: S-0004 claim 11 (vehicles: the fourth tab)
        let vehicles_t = units(&vehicles, &s.vehicles, 3, false);
        // spec: S-0004 claim 11 (aircraft: the fourth tab)
        let aircraft_t = units(&aircraft, &s.aircraft, 3, true);
        let buildings_t = buildings
            .iter()
            .map(|n| {
                let b = s.buildings.get(n).cloned().unwrap_or_default();
                let e = &b.econ;
                let disableable = |f: &Option<String>| {
                    f.as_ref().is_some_and(|sw| {
                        // spec: S-0407 Import mapping (DisableableFromShell absent: no)
                        s.superweapons
                            .iter()
                            .find(|(k, _)| k.eq_ignore_ascii_case(sw))
                            .is_some_and(|(_, v)| v.disableable_from_shell.unwrap_or(false))
                    })
                };
                let disableable_sw = disableable(&e.super_weapon);
                let disableable_sw2 = disableable(&e.super_weapon2);
                let fixed_secret = [
                    (Kind::Infantry, &e.secret_infantry, &infantry),
                    (Kind::Vehicle, &e.secret_unit, &vehicles),
                    (Kind::Building, &e.secret_building, &buildings),
                ]
                .into_iter()
                // spec: S-0407 claim 14 (the fixed secret type: the first of infantry, vehicle and building)
                .find_map(|(k, n, list)| {
                    n.as_ref()
                        .and_then(|n| find(list, n))
                        .map(|index| TypeRef { kind: k, index })
                });
                let i = find(&buildings, n).unwrap_or(0);
                let mut t = TypeTech {
                    // spec: S-0407 Import mapping (TechLevel absent: 255)
                    tech_level: b.tech_level.unwrap_or(255),
                    // spec: S-0407 Import mapping (DontScore, Insignificant absent: no)
                    dont_score: b.combat.dont_score.unwrap_or(false),
                    insignificant: b.combat.insignificant.unwrap_or(false),
                    owners: owners(&b.owner),
                    disableable_sw,
                    disableable_sw2,
                    tech_lab: tech_labs.contains(&i),
                    // spec: S-0407 claim 6 (radar flag; power field, absent 0)
                    radar: e.radar.unwrap_or(false),
                    power: e.power.unwrap_or(0),
                    // spec: S-0407 Import mapping (AIBasePlanningSide absent: −1)
                    side: e.ai_base_planning_side.unwrap_or(-1),
                    // spec: S-0407 Import mapping (SecretLab absent: no)
                    secret_lab: e.secret_lab.unwrap_or(false),
                    fixed_secret,
                    // spec: S-0407 claim 16 and S-0501 claim 30 (only helipad buildings count; the docking-place count's absent default is 1, an explicit 0 adds 0)
                    docks: if b.helipad.unwrap_or(false) {
                        e.number_of_docks.unwrap_or(1)
                    } else {
                        0
                    },
                    upgrade: e.powers_up_building.is_some(),
                    // spec: S-0004 claim 11 (build category combat: the second tab; other buildings: the first; the category's import is S-0302 claim 5's)
                    tab: u8::from(b.defense.unwrap_or(false)),
                    // spec: S-0004 Import mapping (naval = Naval; absent: no)
                    naval: b.naval.unwrap_or(false),
                    ..Default::default()
                };
                econ_fields(e, &mut t);
                t
            })
            .collect();
        let groups = [
            names(&g.prerequisite_power, &buildings),
            names(&g.prerequisite_factory, &buildings),
            names(&g.prerequisite_barracks, &buildings),
            names(&g.prerequisite_radar, &buildings),
            names(&g.prerequisite_tech, &buildings),
            names(&g.prerequisite_proc, &buildings),
        ];
        let proc_alternate = g
            .prerequisite_proc_alternate
            .as_ref()
            .and_then(|n| find(&vehicles, n));
        let mut secrets = Vec::new();
        // spec: S-0407 claim 14 (the secret infantry, vehicle and building lists, concatenated in that order)
        for (k, v, list) in [
            (Kind::Infantry, &g.secret_infantry, &infantry),
            (Kind::Vehicle, &g.secret_units, &vehicles),
            (Kind::Building, &g.secret_buildings, &buildings),
        ] {
            secrets.extend(
                names(v, list)
                    .into_iter()
                    .map(|index| TypeRef { kind: k, index }),
            );
        }
        Self {
            infantry: infantry_t,
            vehicles: vehicles_t,
            aircraft: aircraft_t,
            buildings: buildings_t,
            groups,
            proc_alternate,
            secrets,
            pad_aircraft: names(&g.pad_aircraft, &aircraft),
            country_side: countries
                .iter()
                .map(|c| {
                    // spec: S-0004 claim 9 (a country's own side; side names compared without regard to case; a side's index is its position in the imported side list)
                    let side = s.countries.get(c).and_then(|c| c.econ.side.as_deref())?;
                    r.sides
                        .iter()
                        .position(|x| x.name.eq_ignore_ascii_case(side))
                        .map(|i| i as i32)
                })
                .collect(),
        }
    }

    /// A type's tech-tree fields.
    pub fn of(&self, t: TypeRef) -> Option<&TypeTech> {
        let v = match t.kind {
            Kind::Infantry => &self.infantry,
            Kind::Vehicle => &self.vehicles,
            Kind::Aircraft => &self.aircraft,
            Kind::Building => &self.buildings,
            Kind::Terrain => return None,
        };
        v.get(t.index as usize)
    }

    /// Every type of the four production kinds, in kind and registration
    /// order.
    pub fn all_types(&self) -> Vec<TypeRef> {
        let mut out = Vec::new();
        for (kind, v) in [
            (Kind::Infantry, &self.infantry),
            (Kind::Vehicle, &self.vehicles),
            (Kind::Aircraft, &self.aircraft),
            (Kind::Building, &self.buildings),
        ] {
            out.extend((0..v.len()).map(|i| TypeRef {
                kind,
                index: i as u32,
            }));
        }
        out
    }
}

/// A tech-tree answer (S-0407 Terms).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Answer {
    /// Allowed.
    Allowed,
    /// At limit.
    AtLimit,
    /// Refused.
    Refused,
}

fn tt(w: &World, t: TypeRef) -> Option<&TypeTech> {
    w.rules().econ.tech.of(t)
}

/// An object's type.
pub fn type_of(o: &Object) -> TypeRef {
    TypeRef {
        kind: o.kind,
        index: o.type_index,
    }
}

/// A building whose current behaviour is *selling* (S-0407 Terms, *being
/// sold*).
pub fn being_sold(o: &Object) -> bool {
    o.beh.seen() == Some(Behaviour::Selling)
}

/// A building whose queued behaviour is *selling* (S-0407 Terms, *queued
/// to be sold*).
pub fn queued_to_be_sold(o: &Object) -> bool {
    o.beh.queued == Some(Behaviour::Selling)
}

/// The house tech level (S-0407 Terms, S-0009 Terms): the game's tech level
/// for a player house, 1 for the neutral and special houses.
pub fn house_tech_level(w: &World, house: HouseId) -> i32 {
    match w.houses.get(usize::from(house.0)).map(|h| h.role) {
        Some(HouseRole::Player) => w.rules().game_tech_level,
        // spec: S-0009 Constants (tech level of the neutral and special houses: 1)
        _ => 1,
    }
}

/// The present count of a type for a house (S-0407 Terms): its objects of
/// the type on the map. Don't-score buildings, aircraft and infantry never
/// count; a don't-score vehicle type counts by S-0414's rule (claim 4).
pub fn present_count(w: &World, house: HouseId, t: TypeRef) -> i32 {
    let Some(f) = tt(w, t) else { return 0 };
    if f.dont_score {
        if t.kind == Kind::Vehicle {
            // spec: S-0407 Terms and S-0414 claim 4 (a don't-score vehicle type: its entries since game start; quirks.unscored_vehicle_stays_counted)
            return crate::econ::victory::unscored_vehicle_entries(w, house, t.index);
        }
        // spec: S-0407 Terms (don't-score buildings, aircraft and infantry are never counted)
        return 0;
    }
    w.objects
        .values()
        .filter(|o| o.on_map && o.owner == Some(house) && type_of(o) == t)
        .count() as i32
}

/// The owned count of a type for a house (S-0407 Terms): its objects of the
/// type, on the map or not (an item's object included). Don't-score and
/// insignificant types are never counted. For a vehicle-thief infantry
/// type, the vehicles holding such a thief count too (claim 9): a seam
/// until the special-abilities specs give take-overs.
pub fn owned_count(w: &World, house: HouseId, t: TypeRef) -> i32 {
    let Some(f) = tt(w, t) else { return 0 };
    // spec: S-0407 Terms (don't-score and insignificant types are never counted in the owned count)
    if f.dont_score || f.insignificant {
        return 0;
    }
    let n = w
        .objects
        .values()
        .filter(|o| o.owner == Some(house) && type_of(o) == t)
        .count() as i32;
    // spec: S-0407 claim 9 (a vehicle thief's type also counts the vehicles it took over: a seam, no take-overs yet)
    n + thief_vehicles(w, house, t)
}

/// The vehicles of a house holding a thief of infantry type `t` after a
/// take-over (S-0407 claim 9). Take-overs belong to the special-abilities
/// specs (batch 5 and later): none happen yet, so the count is 0.
fn thief_vehicles(w: &World, house: HouseId, t: TypeRef) -> i32 {
    let _ = (w, house, t);
    0
}

/// The ever-built count of a type for a house (S-0407 Terms, claim 10).
pub fn ever_built(w: &World, house: HouseId, t: TypeRef) -> i32 {
    w.houses
        .get(usize::from(house.0))
        .and_then(|h| h.econ.tech.ever_built.get(&t).copied())
        .unwrap_or(0)
}

/// Raises the ever-built count (S-0407 claim 10). Don't-score types are
/// never counted.
pub fn count_built(w: &mut World, house: HouseId, t: TypeRef) {
    if tt(w, t).is_some_and(|f| f.dont_score) {
        // spec: S-0407 Terms (don't-score types are never counted as ever built)
        return;
    }
    if let Some(h) = w.houses.get_mut(usize::from(house.0)) {
        *h.econ.tech.ever_built.entry(t).or_insert(0) += 1;
    }
}

/// Whether one of the house's lines has an item of the type (S-0407 claim 9,
/// F2's *P*).
fn in_production(w: &World, house: HouseId, t: TypeRef) -> bool {
    w.econ
        .lines
        .lines
        .iter()
        .any(|l| l.house == house && l.item.as_ref().is_some_and(|i| i.t == t))
}

/// Whether a house's country is in a country list (S-0407 F1).
fn in_list(w: &World, country: u32, list: &[u32]) -> bool {
    if w.rules().quirks.country_index_wraps {
        // spec: S-0407 F1 (i mod 32 = j mod 32; quirks.country_index_wraps)
        list.iter().any(|&j| country % 32 == j % 32)
    } else {
        list.contains(&country)
    }
}

/// The build-limit answer (S-0407 claim 9, F2).
pub fn limit_answer(w: &World, house: HouseId, t: TypeRef, production_exception: bool) -> Answer {
    let Some(f) = tt(w, t) else {
        return Answer::Refused;
    };
    let l = f.limit;
    if l > 0 {
        let o = owned_count(w, house, t);
        if o < l {
            return Answer::Allowed;
        }
        let p = production_exception && in_production(w, house, t);
        // spec: S-0407 F2 (with an item in production: allowed at O = L, or above L for a non-infantry type; quirks.infantry_limit_exception_exact)
        let exact = t.kind == Kind::Infantry && w.rules().quirks.infantry_limit_exception_exact;
        if p && (o == l || !exact) {
            Answer::Allowed
        } else {
            Answer::AtLimit
        }
    } else {
        // spec: S-0407 F2 (L ≤ 0: refused when L = −2³¹ or E ≥ −L)
        if l == i32::MIN || ever_built(w, house, t) >= -l {
            Answer::Refused
        } else {
            Answer::Allowed
        }
    }
}

/// Whether a type is lifted for a house (S-0407 claims 3 and 4).
pub fn lifted(w: &World, house: HouseId, t: TypeRef) -> bool {
    let Some(f) = tt(w, t) else { return false };
    // spec: S-0407 claim 3 (an override building present lifts the type)
    if f.overrides.iter().any(|&b| {
        present_count(
            w,
            house,
            TypeRef {
                kind: Kind::Building,
                index: b,
            },
        ) > 0
    }) {
        return true;
    }
    // spec: S-0407 claim 4 (a secret lab of the house on the map with the type as its secret type)
    w.objects.values().any(|o| {
        o.on_map && o.owner == Some(house) && o.kind == Kind::Building && o.econ.secret == Some(t)
    })
}

/// Whether a condition of claims 5 to 8 refuses a type for a house.
pub fn conditions_refuse(w: &World, house: HouseId, t: TypeRef) -> bool {
    let Some(f) = tt(w, t) else { return true };
    if other_conditions_refuse(w, house, t, f) {
        return true;
    }
    // spec: S-0407 claim 8 (every entry of the prerequisite list must be met; an empty list is met)
    !f.prereq.iter().all(|&p| prereq_met(w, house, p))
}

/// The entries of a type's prerequisite list (claim 8) that a house does
/// not meet now, or `None` when a condition of claims 5 to 7 refuses the
/// type anyway: what a player reads off the tech tree to know what to
/// build next.
pub fn unmet_prerequisites(w: &World, house: HouseId, t: TypeRef) -> Option<Vec<Prereq>> {
    let f = tt(w, t)?;
    if other_conditions_refuse(w, house, t, f) {
        return None;
    }
    Some(
        f.prereq
            .iter()
            .copied()
            .filter(|&p| !prereq_met(w, house, p))
            .collect(),
    )
}

/// Whether a condition of claims 5 to 7 refuses a type for a house.
fn other_conditions_refuse(w: &World, house: HouseId, t: TypeRef, f: &TypeTech) -> bool {
    let tech = house_tech_level(w, house);
    // spec: S-0407 claim 5 (a) (tech level −1 or above the house tech level)
    if f.tech_level == -1 || f.tech_level > tech {
        return true;
    }
    let country = w.houses.get(usize::from(house.0)).map_or(0, |h| h.country);
    // spec: S-0407 claim 5 (b) (required list: refused unless the country is in it; forbidden list: refused when it is)
    if let Some(l) = &f.required
        && !in_list(w, country, l)
    {
        return true;
    }
    if let Some(l) = &f.forbidden
        && in_list(w, country, l)
    {
        return true;
    }
    let flags = w
        .houses
        .get(usize::from(house.0))
        .map_or([false; 3], |h| h.econ.tech.stolen); // spec: S-0407 Terms (three stolen-tech flags)
    // spec: S-0407 claim 6 (a type that needs a side's stolen tech is refused without that flag)
    if f.stolen.iter().zip(flags).any(|(&need, has)| need && !has) {
        return true;
    }
    // spec: S-0407 claim 7 (superweapons off: a disableable first superweapon refuses, unless on the tech-lab list; quirks.superweapon_setting_tests_first_only off: the second field too)
    let second = !w.rules().quirks.superweapon_setting_tests_first_only && f.disableable_sw2;
    if t.kind == Kind::Building
        && (f.disableable_sw || second)
        && !f.tech_lab
        && !crate::econ::setup::superweapons_allowed(w)
    {
        return true;
    }
    false
}

fn prereq_met(w: &World, house: HouseId, p: Prereq) -> bool {
    let b = |index| TypeRef {
        kind: Kind::Building,
        index,
    };
    match p {
        // spec: S-0407 claim 8 (a) (present count above 0)
        Prereq::Building(i) => present_count(w, house, b(i)) > 0,
        Prereq::Group(g) => {
            let r = &w.rules().econ.tech;
            // spec: S-0407 claim 8 (b) (any member of the group present)
            if r.groups[g.index()]
                .iter()
                .any(|&i| present_count(w, house, b(i)) > 0)
            {
                return true;
            }
            // spec: S-0407 claim 8 (b) (the refinery group: also the refinery-alternative vehicle present)
            g == Group::Proc
                && r.proc_alternate.is_some_and(|v| {
                    present_count(
                        w,
                        house,
                        TypeRef {
                            kind: Kind::Vehicle,
                            index: v,
                        },
                    ) > 0
                })
        }
        Prereq::Upgrade(u) => upgrade_met(w, house, u),
        // spec: S-0407 claim 8 (d) (quirks.prerequisite_names_dropped off: never met)
        Prereq::Unmet => false,
    }
}

/// Claim 8 (c): of the house's eligible buildings, the most recently
/// acquired decides (`quirks.upgrade_prerequisite_last_building`).
fn upgrade_met(w: &World, house: HouseId, u: u32) -> bool {
    let Some(h) = w.houses.get(usize::from(house.0)) else {
        return false;
    };
    // spec: S-0407 claim 8 (c) (eligible: on the map, switched on, not being sold, not queued to be sold)
    let eligible = h
        .econ
        .acquired
        .iter()
        .filter_map(|&id| w.obj(id))
        .filter(|o| {
            o.on_map
                && o.owner == Some(house)
                && !o.econ.building.switched_off
                && !being_sold(o)
                && !queued_to_be_sold(o)
        });
    let holds = |o: &Object| o.econ.building.held().any(|x| x == u);
    if w.rules().quirks.upgrade_prerequisite_last_building {
        // spec: S-0407 claim 8 (c) (only the most recently acquired eligible building decides)
        eligible.last().is_some_and(holds)
    } else {
        let mut e = eligible;
        e.any(holds)
    }
}

/// The tech-tree answer of a type for a house (S-0407 claim 2).
pub fn answer(w: &World, house: HouseId, t: TypeRef) -> Answer {
    answer_with(w, house, t, true)
}

fn answer_with(w: &World, house: HouseId, t: TypeRef, production_exception: bool) -> Answer {
    // spec: S-0407 claim 2 (lifted: the build-limit answer; else refused when a condition refuses)
    if lifted(w, house, t) || !conditions_refuse(w, house, t) {
        limit_answer(w, house, t, production_exception)
    } else {
        Answer::Refused
    }
}

/// Whether a building is a factory building that could make type `t` for
/// its owner *without the tech-tree rule* (S-0404 Terms), ignoring the power
/// switch and docks. S-0404's own *could make*, with the build-limit answer,
/// is `could_make_limit_only`.
pub fn could_make(w: &World, b: &Object, t: TypeRef) -> bool {
    let r = w.rules();
    // spec: S-0404 Terms (a factory building on the map, neither being sold nor queued to be sold, S-0407 Terms)
    if b.kind != Kind::Building || !b.on_map || being_sold(b) || queued_to_be_sold(b) {
        return false;
    }
    let Some(br) = r.buildings.get(b.type_index as usize) else {
        return false;
    };
    let Some(fk) = br.factory else { return false };
    if fk.kind() != t.kind {
        return false;
    }
    // spec: S-0404 Terms (a naval factory for a naval vehicle, a non-naval one for every other item)
    let naval_item = t.kind == Kind::Vehicle && crate::econ::production::naval(w, t);
    if br.naval != naval_item {
        return false;
    }
    let (Some(bt), Some(it)) = (
        tt(
            w,
            TypeRef {
                kind: Kind::Building,
                index: b.type_index,
            },
        ),
        tt(w, t),
    ) else {
        return false;
    };
    // spec: S-0404 Terms and S-0407 claim 11 (the factory type's owner countries share one with the type's)
    if !bt.owners.iter().any(|c| it.owners.contains(c)) {
        return false;
    }
    b.owner.is_some()
}

/// Whether the house has a factory building that could make `t` (claim 1
/// (b): tech-tree condition, power and docks left out; not being sold or
/// queued to be sold).
fn factory_condition(w: &World, house: HouseId, t: TypeRef) -> bool {
    // spec: S-0407 claim 1 (b) (a factory not being sold or queued to be sold: S-0404's could make)
    w.objects
        .values()
        .any(|b| b.owner == Some(house) && could_make(w, b, t))
}

/// A qualifying factory for a type (S-0407 Terms): a factory building of
/// the type's kind, on the map, switched on, alive, not being sold and not
/// queued to be sold.
fn qualifying_factory(w: &World, house: HouseId, t: TypeRef) -> bool {
    w.objects
        .values()
        .any(|b| b.owner == Some(house) && qualifies(w, b, t.kind))
}

/// Whether building `b` is a qualifying factory for types of `kind`
/// (S-0407 Terms), whoever owns it.
fn qualifies(w: &World, b: &Object, kind: Kind) -> bool {
    b.kind == Kind::Building
        && b.on_map
        && b.health > 0
        && !b.econ.building.switched_off
        && !being_sold(b)
        && !queued_to_be_sold(b)
        && w.rules()
            .buildings
            .get(b.type_index as usize)
            .and_then(|x| x.factory)
            .is_some_and(|f| f.kind() == kind)
}

/// The admission rank of a kind at a recheck (S-0407 claim 17): the place
/// in the house's acquisition order of its earliest still-owned qualifying
/// factory of that kind; none gives the last rank.
fn admission_rank(w: &World, house: HouseId, kind: Kind) -> usize {
    let Some(h) = w.houses.get(usize::from(house.0)) else {
        return usize::MAX;
    };
    h.econ
        .acquired
        .iter()
        .position(|&id| {
            w.obj(id)
                .is_some_and(|b| b.owner == Some(house) && qualifies(w, b, kind))
        })
        .unwrap_or(usize::MAX)
}

/// Items per sidebar tab (S-0004 claim 11 and Constants). The simulation
/// has no superweapon items yet (a seam), so only production icons count.
// spec: S-0004 Constants (items per sidebar tab: 76)
const TAB_CAPACITY: usize = 76;

/// Marks a recheck due for a house (S-0407 claim 12): every event of the
/// table calls this; the recheck runs in the house's next house update.
pub fn recheck_due(w: &mut World, house: HouseId) {
    if let Some(h) = w.houses.get_mut(usize::from(house.0)) {
        h.econ.tech.recheck_due = true;
    }
}

/// The recheck (S-0407 claims 1 and 12): the whole taken-up set worked out
/// again from the state at this moment, and the icons kept with it (S-0004
/// claim 11). Returns the types that left the set in S-0411 claim 13's
/// order, for the removal orders of S-0404 claim 9 and S-0411 claim 2.
pub fn recheck(w: &mut World, house: HouseId) -> Vec<TypeRef> {
    let Some(h) = w.houses.get(usize::from(house.0)) else {
        return Vec::new();
    };
    if h.role != HouseRole::Player {
        // spec: S-0407 claim 1 (the neutral and special houses build nothing)
        if let Some(h) = w.houses.get_mut(usize::from(house.0)) {
            h.econ.tech.recheck_due = false;
        }
        return Vec::new();
    }
    let before = h.econ.tech.taken_up.clone();
    let mut after = BTreeSet::new();
    let mut candidates = Vec::new();
    for t in w.rules().econ.tech.all_types() {
        // spec: S-0407 claim 1 (a) (answer allowed or at limit)
        if answer(w, house, t) == Answer::Refused {
            continue;
        }
        if before.contains(&t) {
            // spec: S-0407 claim 1 (b) and (c) (in the set before; a factory could make it)
            if factory_condition(w, house, t) {
                after.insert(t);
            }
        } else if qualifying_factory(w, house, t) {
            // spec: S-0411 claim 15 (the candidates for admission: an allowed or at-limit answer presented by a qualifying factory, S-0407 claim 1 (c))
            candidates.push(t);
        }
    }
    // spec: S-0407 claim 17 and S-0411 claim 15 (admission order: kinds in the acquisition order of their earliest still-owned qualifying factory, then increasing registration index)
    candidates.sort_by_cached_key(|t| (admission_rank(w, house, t.kind), t.kind, t.index));
    // spec: S-0411 claim 15 (capacity is judged before the removals of the recheck: every icon present at its start occupies a place; each tab of each player house separately)
    let mut used: Vec<usize> = h.econ.tech.icons.iter().map(Vec::len).collect();
    for t in candidates {
        let tab = usize::from(tt(w, t).map_or(0, |f| f.tab));
        let Some(slot) = used.get_mut(tab) else {
            continue;
        };
        // spec: S-0004 claim 11 and Constants (a tab holds at most 76 items; a candidate for a full tab is suppressed: no icon, not in the set) and S-0411 claim 15
        if *slot >= TAB_CAPACITY {
            continue;
        }
        // spec: S-0411 claim 15 (an admitted candidate occupies a place against later candidates even when claim 1 (b) removes it in the same recheck)
        *slot += 1;
        // spec: S-0407 claim 1 (b) (a factory could make it)
        if factory_condition(w, house, t) {
            after.insert(t);
        }
    }
    // spec: S-0411 claim 2 (an icon is added when its type becomes offered and removed only when a recheck removes it from the taken-up set)
    let gone: BTreeSet<TypeRef> = before.difference(&after).copied().collect();
    // spec: S-0411 claim 13 (the removed types in tab order, first to fourth, and within a tab in the icon order immediately before removal)
    let mut removed: Vec<TypeRef> = h
        .econ
        .tech
        .icons
        .iter()
        .flatten()
        .map(|i| i.t)
        .filter(|t| gone.contains(t))
        .collect();
    // A type of the set without an icon cannot occur; were it to, it goes
    // last, in set order.
    let rest: Vec<TypeRef> = gone
        .iter()
        .copied()
        .filter(|t| !removed.contains(t))
        .collect();
    removed.extend(rest);
    let side = w
        .rules()
        .econ
        .tech
        .country_side
        .get(h.country as usize)
        .copied()
        .flatten();
    // spec: S-0407 claim 17 (new offers of one recheck: kinds in the acquisition order of their earliest still-owned qualifying factory, then increasing registration index)
    let mut fresh: Vec<TypeRef> = after.difference(&before).copied().collect();
    fresh.sort_by_cached_key(|t| (admission_rank(w, house, t.kind), t.kind, t.index));
    let added: Vec<(Icon, IconKey)> = fresh
        .iter()
        .map(|&t| {
            let cost = super::production::price(w, house, t);
            (Icon { t, cost }, icon_key(w, t, side))
        })
        .collect();
    let mut keys: BTreeMap<TypeRef, IconKey> = h
        .econ
        .tech
        .icons
        .iter()
        .flatten()
        .filter(|i| !gone.contains(&i.t))
        .map(|i| (i.t, icon_key(w, i.t, side)))
        .collect();
    let tie_newest_first = w.rules().quirks.sidebar_tie_newest_first;
    let both_flags = w.rules().quirks.sidebar_both_flags_rank;
    let hh = &mut w.houses[usize::from(house.0)].econ.tech;
    hh.recheck_due = false;
    hh.taken_up = after;
    for tab in hh.icons.iter_mut() {
        tab.retain(|i| !gone.contains(&i.t));
    }
    for (icon, key) in added {
        let tab = &mut hh.icons[usize::from(key.tab)];
        // spec: S-0004 claim 11 (an item already shown is not added twice)
        if tab.iter().any(|i| i.t == icon.t) {
            continue;
        }
        // spec: S-0004 claim 11 (placed in front of the first item it sorts before, else after all of them)
        let at = tab
            .iter()
            .position(|old| {
                keys.get(&old.t).is_some_and(|ok| {
                    sorts_before(&key, icon.cost, ok, old.cost, both_flags, tie_newest_first)
                })
            })
            .unwrap_or(tab.len());
        tab.insert(at, icon);
        keys.insert(icon.t, key);
    }
    removed
}

/// The fixed sort keys of an item (S-0004 claim 11).
#[derive(Clone, Copy, Debug)]
struct IconKey {
    tab: u8,
    /// Rule 2: the base-planning side equals the house's side.
    own_side: bool,
    /// Rule 3: the counts-as-aircraft flag, for vehicle and aircraft types
    /// only.
    air: bool,
    /// Rule 3: the naval flag, for vehicle and aircraft types only.
    naval: bool,
    /// Rule 4.
    tech_level: i32,
}

fn icon_key(w: &World, t: TypeRef, side: Option<i32>) -> IconKey {
    let f = tt(w, t).cloned().unwrap_or_default();
    // spec: S-0004 claim 11 rule 3 (the flags count only for vehicle and aircraft types)
    let unit = matches!(t.kind, Kind::Vehicle | Kind::Aircraft);
    IconKey {
        tab: f.tab,
        // spec: S-0004 claim 11 rule 2 (the base-planning side equals the house's side)
        own_side: side.is_some_and(|s| s == f.side),
        air: unit && f.considered_aircraft,
        naval: unit && f.naval,
        tech_level: f.tech_level,
    }
}

/// Category rank (S-0004 claim 11 rule 3): ground 0, air 1, both flags 1
/// as the new item and 2 as an item already in the tab
/// (`quirks.sidebar_both_flags_rank`; off: 1), naval 3.
fn rank(k: &IconKey, new: bool, both_flags: bool) -> i32 {
    // spec: S-0004 claim 11 rule 3 (the rank table: ground 0, air 1, both 1 new or 2 existing, naval 3)
    match (k.air, k.naval) {
        (false, false) => 0,
        (true, false) => 1,
        (true, true) if new || !both_flags => 1,
        (true, true) => 2,
        (false, true) => 3,
    }
}

/// Whether a new item sorts before an item already in the tab (S-0004
/// claim 11, rules 1 to 6).
fn sorts_before(
    new: &IconKey,
    new_cost: i32,
    old: &IconKey,
    old_cost: i32,
    both_flags: bool,
    tie_newest_first: bool,
) -> bool {
    // Rule 1 never applies: the simulation has no superweapon items yet.
    let a = (
        // spec: S-0004 claim 11 rule 2 (the item whose base-planning side equals the house's side first)
        !new.own_side,
        // spec: S-0004 claim 11 rule 3 (category rank, lowest first)
        rank(new, true, both_flags),
        // spec: S-0004 claim 11 rule 4 (tech level, lowest first)
        new.tech_level,
        // spec: S-0004 claim 11 rule 5 (cost to the house at the time it is added, lowest first)
        new_cost,
    );
    let b = (
        !old.own_side,
        rank(old, false, both_flags),
        old.tech_level,
        old_cost,
    );
    match a.cmp(&b) {
        std::cmp::Ordering::Less => true,
        std::cmp::Ordering::Greater => false,
        // Seam: display names (S-0004 claim 11 rule 6) are text the simulation does not import yet, so they compare equal.
        // spec: S-0004 claim 11 rule 6 (equal names: the newly added item first; quirks.sidebar_tie_newest_first)
        std::cmp::Ordering::Equal => tie_newest_first,
    }
}

/// Whether a type is offered to a house (S-0407 claim 1).
pub fn offered(w: &World, house: HouseId, t: TypeRef) -> bool {
    w.houses
        .get(usize::from(house.0))
        .is_some_and(|h| h.econ.tech.taken_up.contains(&t))
}

/// Buildable (S-0411 Terms): the tech-tree answer is allowed or at limit
/// and some factory building of the house could make the type without the
/// tech-tree rule (one being sold does not count). It describes current
/// conditions only: no icon is removed and no order given by it (S-0411
/// claim 2).
pub fn buildable(w: &World, house: HouseId, t: TypeRef) -> bool {
    answer(w, house, t) != Answer::Refused && factory_condition(w, house, t)
}

/// Whether an offered type is at limit for a house (S-0407 Terms, *offered*:
/// judged from the current counts at every moment, claims 9 and 13): its
/// build-limit answer, with the allowance for an item in production. A type
/// whose other conditions have come to refuse it since the last recheck is
/// still judged by its limit alone (S-0407 claim 13, S-0411 claim 4).
pub fn at_limit(w: &World, house: HouseId, t: TypeRef) -> bool {
    // spec: S-0407 claims 9 and 13 (at limit: the build-limit answer from the current counts)
    limit_answer(w, house, t, true) == Answer::AtLimit
}

/// The test made when an order to start or queue an item is issued (S-0407
/// claims 13 and 16, *At issue*): the type is at limit (claim 9's answer,
/// [`at_limit`]), or claim 16's test refuses it with the line's copies as
/// they are at that moment, none for an idle line ([`queue_refused`]). For a
/// dock-bound aircraft type that is the house-wide pad capacity *D* ≥ *T*,
/// so an aircraft in flight counts like one on its pad. A resume of a held
/// item is exempt; the caller tests that first.
pub fn refused_at_issue(w: &World, house: HouseId, t: TypeRef) -> bool {
    // spec: S-0407 claim 13 (our engine: not at limit by claim 9's answer when issued, also for a dock-bound aircraft type at its own limit; a design rule with no switch)
    at_limit(w, house, t)
        // spec: S-0407 claim 16 *At issue* and F3 (the same test made at issue, with Q the line's copies then, 0 for an idle line; a dock-bound aircraft: D, present listed aircraft in flight or on a pad plus line copies, refused when D ≥ T)
        || queue_refused(
            w,
            house,
            t,
            crate::econ::production::copies_in_line(w, house, t),
        )
}

/// Whether a factory building *could make* `t` (S-0404 Terms, revision 3):
/// the factory conditions and the build-limit answer with the allowance
/// for an item in production (S-0407 claim 9), not the full tech tree. The
/// factory check of S-0404 claim 8, the darkening of S-0411 claim 3 and,
/// through `could_start`, the application of build orders and the forced
/// start of S-0404 claims 3 and 10 use it.
pub fn could_make_limit_only(w: &World, b: &Object, t: TypeRef) -> bool {
    // spec: S-0404 Terms (could make: the factory conditions and the build-limit answer with the production allowance, no full tech-tree test) and S-0407 claim 13
    could_make(w, b, t)
        && b.owner
            .is_some_and(|h| limit_answer(w, h, t, true) == Answer::Allowed)
}

/// Whether a factory building may serve a new start, a queue request, a
/// resume or a release (S-0411 claim 14): it could make `t` (S-0404 Terms,
/// which already leaves out a building queued to be sold).
pub fn could_start(w: &World, b: &Object, t: TypeRef) -> bool {
    // spec: S-0411 claim 14 (a factory queued to be sold is excluded from new starts, queue requests, resumes and releases) and S-0404 Terms
    could_make_limit_only(w, b, t)
}

/// The house's part of the house update for the tech tree (S-0407 claim 12):
/// a recheck when one is due, and, for the types it removes from the
/// taken-up set, the removal orders (S-0404 claim 9, S-0411 claim 2).
pub fn house_update(w: &mut World, house: HouseId) {
    if w.houses
        .get(usize::from(house.0))
        .is_some_and(|h| h.econ.tech.recheck_due)
    {
        let removed = recheck(w, house);
        // spec: S-0404 claim 9 and S-0411 claim 2 (the orders are issued in the house update of the recheck)
        crate::econ::sidebar::removal_orders(w, house, &removed);
    }
}

/// Event (g) of claim 12: an object of a type that is at limit for its
/// owner is about to stop existing, judged with the object counted and the
/// production exception left out. Non-event (o) with
/// `quirks.unit_changes_no_recheck` off.
pub fn object_ends(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id) else { return };
    let (Some(house), t, mobile) = (o.owner, type_of(o), o.kind.is_mobile()) else {
        return;
    };
    // spec: S-0407 claim 12 (g) (only the answer at limit, with the production exception left out, is an event)
    let at_limit = answer_with(w, house, t, false) == Answer::AtLimit;
    // spec: S-0407 claim 12 (o) and quirks.unit_changes_no_recheck (off: a unit's end is always an event)
    if at_limit || (mobile && !w.rules().quirks.unit_changes_no_recheck) {
        recheck_due(w, house);
    }
}

/// Stolen tech (S-0407 claim 6): a spy of `house` infiltrated building
/// `b`. Infiltration itself belongs to the special-abilities specs, which
/// call this.
pub fn infiltrated(w: &mut World, house: HouseId, b: ObjectId) {
    infiltrated_with(w, house, b, true);
}

/// [`infiltrated`] with the radar and power exclusions of S-0407 claim 6
/// applied or not: S-0510's single-effect switch off (`quirks.
/// infiltration_single_effect`) requires only tech-lab-list membership.
pub fn infiltrated_with(w: &mut World, house: HouseId, b: ObjectId, exclusions: bool) {
    let Some(o) = w.obj(b) else { return };
    if o.kind != Kind::Building {
        return;
    }
    let f = tt(w, type_of(o)).cloned().unwrap_or_default();
    // spec: S-0407 claim 6 (a tech-lab building that is not a radar building and has no power output; S-0510 Edge cases: with the single-effect switch off only the list membership)
    if f.tech_lab && (!exclusions || (!f.radar && f.power <= 0)) {
        // spec: S-0407 claim 6 (side 0: the first flag, 1: the second, any other value: the third)
        let flag = match f.side {
            0 => 0,
            1 => 1,
            _ => 2, // spec: S-0407 Constants (stolen-tech side values 0, 1, other)
        };
        if let Some(h) = w.houses.get_mut(usize::from(house.0)) {
            h.econ.tech.stolen[flag] = true;
        }
    }
    // spec: S-0407 claim 12 (h) (a spy of the house infiltrates a building)
    recheck_due(w, house);
}

/// The queue-time limit (S-0407 claim 16, F3): whether a type queued behind
/// an item in its line is refused. `copies`: the type's copies in the line
/// (item and queue).
pub fn queue_refused(w: &World, house: HouseId, t: TypeRef, copies: i32) -> bool {
    let Some(f) = tt(w, t) else { return true };
    let q = w.rules().quirks;
    let l = f.limit;
    let limit_test = || {
        if l > 0 {
            // spec: S-0407 claim 16 (the item in production counted in the owned count and in the line; quirks.queue_limit_counts_item_twice)
            let item = !q.queue_limit_counts_item_twice && in_production(w, house, t);
            // spec: S-0407 F3 (L > 0: refused when O + Q ≥ L)
            // spec: S-0407 F3 (signed 32-bit sums, wrapping modulo 2³², S-0205 claim 1)
            owned_count(w, house, t).wrapping_add(copies.wrapping_sub(i32::from(item))) >= l
        } else {
            // spec: S-0407 F3 (L ≤ 0: refused when L = −2³¹ or E + Q ≥ −L, the sum wrapping)
            l == i32::MIN || ever_built(w, house, t).wrapping_add(copies) >= -l
        }
    };
    if t.kind == Kind::Aircraft && f.airport_bound {
        let r = &w.rules().econ.tech;
        // spec: S-0407 F3 and S-0501 F12 (D and T: exact non-negative integer count arithmetic, no wrapping)
        let count = |pt: TypeRef| {
            i64::from(present_count(w, house, pt).max(0))
                + i64::from(crate::econ::production::copies_in_line(w, house, pt).max(0))
        };
        // spec: S-0407 claim 16 (D: present pad aircraft plus their copies in the aircraft line)
        let mut d = 0i64;
        for &p in &r.pad_aircraft {
            d += count(TypeRef {
                kind: Kind::Aircraft,
                index: p,
            });
        }
        if !q.pad_limit_counts_list_only && !r.pad_aircraft.contains(&t.index) {
            // spec: S-0501 claim 30 and S-0407 claim 16 (quirks.pad_limit_counts_list_only off: the asked-for type outside the list is counted too, never twice)
            d += count(t);
        }
        // spec: S-0407 claim 16 and S-0501 claim 30 (T: the house's saved pad total)
        let total = w
            .houses
            .get(usize::from(house.0))
            .map_or(0, |h| h.econ.tech.pad_total);
        // spec: S-0501 F12 (refused when n ≥ the pad total)
        let dock = d >= total;
        // spec: S-0407 F3 (a dock-bound aircraft: refused when D ≥ T; quirks.dock_bound_aircraft_ignores_limit)
        return if q.dock_bound_aircraft_ignores_limit {
            dock
        } else {
            dock || limit_test()
        };
    }
    limit_test()
}

/// The docking-place count a building adds to its house's pad total
/// (S-0501 claim 30): 0 for a building that is not a helipad building.
fn pad_docks(w: &World, id: ObjectId) -> i32 {
    w.obj(id)
        .filter(|o| o.kind == Kind::Building)
        .and_then(|o| w.rules().econ.tech.buildings.get(o.type_index as usize))
        .map_or(0, |b| b.docks)
}

/// Adds `n` to a house's pad total, never below 0 (S-0501 claim 30).
fn add_pads(w: &mut World, house: Option<HouseId>, n: i32) {
    if n == 0 {
        return;
    }
    if let Some(h) = house
        && let Some(x) = w.houses.get_mut(usize::from(h.0))
    {
        // spec: S-0501 claim 30 (the pad total never goes below 0) and S-0407 F3 (T: exact count arithmetic, no wrapping)
        x.econ.tech.pad_total = (x.econ.tech.pad_total + i64::from(n)).max(0);
    }
}

/// A building was placed (S-0501 claim 30): a helipad building adds its
/// docking-place count to its house's pad total.
pub fn pad_placed(w: &mut World, id: ObjectId) {
    let n = pad_docks(w, id);
    let owner = w.obj(id).and_then(|o| o.owner);
    // spec: S-0501 claim 30 (the pad total grows by d when the building is placed)
    add_pads(w, owner, n);
}

/// A building leaves the map (S-0501 claim 30): its count is taken from
/// its house's pad total, never below 0.
pub fn pad_removed(w: &mut World, id: ObjectId) {
    let n = pad_docks(w, id);
    let owner = w.obj(id).filter(|o| o.on_map).and_then(|o| o.owner);
    // spec: S-0501 claim 30 (it shrinks by d when the building is removed from the map, never below 0)
    add_pads(w, owner, -n);
}

/// A building on the map changed owner from `old` (S-0501 claim 30): its
/// count moves with it.
pub fn pad_owner_changed(w: &mut World, id: ObjectId, old: Option<HouseId>) {
    let Some(o) = w.obj(id) else { return };
    let new = o.owner;
    if !o.on_map || old == new {
        return;
    }
    let n = pad_docks(w, id);
    // spec: S-0501 claim 30 (the count moves with the building when it changes owner)
    add_pads(w, old, -n);
    add_pads(w, new, n);
}

/// Secret labs at game start (S-0407 claim 14): S-0412's game-start setup
/// calls this after the starting units are placed.
pub fn assign_secret_labs(w: &mut World) {
    let r = w.rules_arc();
    let list = &r.econ.tech.secrets;
    let n_entries = list.len() as i32;
    // spec: S-0407 claim 14 (the labs in creation order: object number order)
    let labs: Vec<(ObjectId, Option<TypeRef>)> = w
        .objects
        .values()
        .filter(|o| o.kind == Kind::Building && o.on_map)
        .filter_map(|o| {
            let f = r.econ.tech.buildings.get(o.type_index as usize)?;
            f.secret_lab.then_some((o.id, f.fixed_secret))
        })
        .collect();
    let n = labs.len() as i32;
    let draw = n >= 1 && n <= n_entries;
    let mut given: Vec<bool> = vec![false; list.len()];
    for (k, (id, fixed)) in labs.into_iter().enumerate() {
        let mut secret = fixed;
        if draw {
            // spec: S-0407 F4 (one ranged draw over 0 … N − 1 − k for every lab, one with a fixed type included)
            let rk = w
                .rng
                .ranged(0, n_entries - 1 - k as i32, r.quirks.ranged_draw_half_span);
            let entry = if r.quirks.secret_lab_pick_by_position {
                // spec: S-0407 claim 14 (entry r_k by position; quirks.secret_lab_pick_by_position)
                Some(rk as usize)
            } else {
                // spec: S-0407 Edge cases (off: the entry at position r_k among those not yet given)
                (0..list.len()).filter(|&i| !given[i]).nth(rk as usize)
            };
            if let Some(e) = entry {
                if let Some(g) = given.get_mut(e) {
                    *g = true;
                }
                if secret.is_none() {
                    secret = list.get(e).copied();
                }
            }
        }
        if let Some(o) = w.obj_mut(id) {
            o.econ.secret = secret;
        }
    }
}
