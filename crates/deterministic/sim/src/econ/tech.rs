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
    /// Docks a building adds to the house total (claim 16): its number of
    /// docks when it has the dock flag, else 0.
    pub docks: i32,
    /// A building type that is an upgrade type (S-0406 Terms).
    pub upgrade: bool,
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
                    // spec: S-0407 claim 8 (d) (a name that is neither a group word nor a building type is dropped)
                    let b = find(&buildings, n)?;
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
        };
        let units = |list: &[String],
                     map: &std::collections::BTreeMap<String, glasnost_rules::sim::UnitType>|
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
                        ..Default::default()
                    };
                    econ_fields(&u.econ, &mut t);
                    t
                })
                .collect()
        };
        let infantry_t = units(&infantry, &s.infantry);
        let vehicles_t = units(&vehicles, &s.vehicles);
        let aircraft_t = units(&aircraft, &s.aircraft);
        let buildings_t = buildings
            .iter()
            .map(|n| {
                let b = s.buildings.get(n).cloned().unwrap_or_default();
                let e = &b.econ;
                let disableable_sw = e.super_weapon.as_ref().is_some_and(|sw| {
                    // spec: S-0407 Import mapping (DisableableFromShell absent: no)
                    s.superweapons
                        .iter()
                        .find(|(k, _)| k.eq_ignore_ascii_case(sw))
                        .is_some_and(|(_, v)| v.disableable_from_shell.unwrap_or(false))
                });
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
                    tech_lab: tech_labs.contains(&i),
                    // spec: S-0407 claim 6 (radar flag; power field, absent 0)
                    radar: e.radar.unwrap_or(false),
                    power: e.power.unwrap_or(0),
                    // spec: S-0407 Import mapping (AIBasePlanningSide absent: −1)
                    side: e.ai_base_planning_side.unwrap_or(-1),
                    // spec: S-0407 Import mapping (SecretLab absent: no)
                    secret_lab: e.secret_lab.unwrap_or(false),
                    fixed_secret,
                    // spec: S-0407 Import mapping (NumberOfDocks absent: 0; counted with the Helipad flag, absent no)
                    docks: if b.helipad.unwrap_or(false) {
                        e.number_of_docks.unwrap_or(0)
                    } else {
                        0
                    },
                    upgrade: e.powers_up_building.is_some(),
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
    o.beh.current == Some(Behaviour::Selling)
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
    // spec: S-0407 claim 7 (superweapons off: a disableable first superweapon refuses, unless on the tech-lab list)
    if t.kind == Kind::Building
        && f.disableable_sw
        && !f.tech_lab
        && !crate::econ::setup::superweapons_allowed(w)
    {
        return true;
    }
    // spec: S-0407 claim 8 (every entry of the prerequisite list must be met; an empty list is met)
    !f.prereq.iter().all(|&p| prereq_met(w, house, p))
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
    let holds = |o: &Object| o.econ.building.upgrades.contains(&u);
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

/// Whether a building is a factory building that *could make* type `t` for
/// its owner (S-0404 Terms), with the tech-tree condition when `tech`, and
/// ignoring the power switch and docks. `exclude`: a building about to
/// leave the map.
pub fn could_make(w: &World, b: &Object, t: TypeRef, tech: bool) -> bool {
    let r = w.rules();
    if b.kind != Kind::Building || !b.on_map || being_sold(b) {
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
    let Some(owner) = b.owner else { return false };
    !tech || answer(w, owner, t) != Answer::Refused
}

/// Whether the house has a factory building that could make `t` (claim 1
/// (b): tech-tree condition, power and docks left out).
fn factory_condition(w: &World, house: HouseId, t: TypeRef) -> bool {
    w.objects
        .values()
        .any(|b| b.owner == Some(house) && could_make(w, b, t, false))
}

/// A qualifying factory for a type (S-0407 Terms): a factory building of
/// the type's kind, on the map, switched on, alive, not being sold and not
/// queued to be sold.
fn qualifying_factory(w: &World, house: HouseId, t: TypeRef) -> bool {
    let r = w.rules();
    w.objects.values().any(|b| {
        b.kind == Kind::Building
            && b.owner == Some(house)
            && b.on_map
            && b.health > 0
            && !b.econ.building.switched_off
            && !being_sold(b)
            && !queued_to_be_sold(b)
            && r.buildings
                .get(b.type_index as usize)
                .and_then(|x| x.factory)
                .is_some_and(|f| f.kind() == t.kind)
    })
}

/// Marks a recheck due for a house (S-0407 claim 12): every event of the
/// table calls this; the recheck runs in the house's next house update.
pub fn recheck_due(w: &mut World, house: HouseId) {
    if let Some(h) = w.houses.get_mut(usize::from(house.0)) {
        h.econ.tech.recheck_due = true;
    }
}

/// The recheck (S-0407 claims 1 and 12): the whole taken-up set worked out
/// again from the state at this moment. Returns the types that left the set,
/// in set order, for the removal orders of S-0404 claim 9 and S-0411 claim 2.
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
    for t in w.rules().econ.tech.all_types() {
        // spec: S-0407 claim 1 (a) and (b) (answer allowed or at limit; a factory could make it)
        let a = answer(w, house, t) != Answer::Refused;
        if !a || !factory_condition(w, house, t) {
            continue;
        }
        // spec: S-0407 claim 1 (c) (in the set before, or a qualifying factory at the recheck)
        if before.contains(&t) || qualifying_factory(w, house, t) {
            after.insert(t);
        }
    }
    // spec: S-0411 claim 2 (an icon is added when its type becomes offered and removed only when a recheck removes it from the taken-up set)
    let removed: Vec<TypeRef> = before.difference(&after).copied().collect();
    let hh = &mut w.houses[usize::from(house.0)].econ.tech;
    hh.recheck_due = false;
    hh.taken_up = after;
    removed
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

/// Whether a factory building could make `t` when a build order is applied
/// or for an icon's darkening (S-0407 claim 13, S-0411 claims 3 and 4):
/// could make without the tech-tree rule, with the build-limit test kept,
/// including its allowance for an item in production (S-0407 claim 9).
/// Interim rule of request Q-0122: S-0404 claim 3's wording names the full
/// tech-tree condition.
pub fn could_make_limit_only(w: &World, b: &Object, t: TypeRef) -> bool {
    // spec: S-0407 claim 13 (the build order checks only the factory conditions and the build limit when applied) and S-0411 claim 3 (the build-limit test retained); Q-0122
    could_make(w, b, t, false)
        && b.owner
            .is_some_and(|h| limit_answer(w, h, t, true) == Answer::Allowed)
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
    let Some(o) = w.obj(b) else { return };
    if o.kind != Kind::Building {
        return;
    }
    let f = tt(w, type_of(o)).cloned().unwrap_or_default();
    // spec: S-0407 claim 6 (a tech-lab building that is not a radar building and has no power output)
    if f.tech_lab && !f.radar && f.power <= 0 {
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
            owned_count(w, house, t).saturating_add(copies - i32::from(item)) >= l
        } else {
            // spec: S-0407 F3 (L ≤ 0: refused when L = −2³¹ or E + Q ≥ −L)
            l == i32::MIN || ever_built(w, house, t).saturating_add(copies) >= -l
        }
    };
    if t.kind == Kind::Aircraft && f.airport_bound {
        let r = &w.rules().econ.tech;
        // spec: S-0407 claim 16 (D: present pad aircraft plus their copies in the aircraft line)
        let mut d = 0i32;
        for &p in &r.pad_aircraft {
            let pt = TypeRef {
                kind: Kind::Aircraft,
                index: p,
            };
            d += present_count(w, house, pt);
            d += crate::econ::production::copies_in_line(w, house, pt);
        }
        // spec: S-0407 claim 16 (T: the docks of the house's dock buildings)
        let total: i32 = w
            .objects
            .values()
            .filter(|b| b.kind == Kind::Building && b.on_map && b.owner == Some(house))
            .filter_map(|b| r.buildings.get(b.type_index as usize))
            .map(|b| b.docks)
            .sum();
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
