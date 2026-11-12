//! Crates (S-0520): the crate places and their random placement, expiry and
//! replacement, pickup by ground movers, jumpjets and teleporters, the choice
//! of the power-up and its conversions to money, and the nineteen effects.
//!
//! A crate is a cell holding an overlay of a type with the crate flag; its
//! contents are the cell's overlay data value (0 to 18 a fixed power-up,
//! anything else random). The crate places record each placed crate's cell,
//! start tick and lifetime; a cell that holds a crate overlay with no place
//! (a map-placed one) never expires. Sounds and announcements are local-player
//! feedback and not state (claim 26).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use glasnost_map::terrain::land;
use glasnost_map::{CellXY, NO_OVERLAY};
use glasnost_num::{F64, Rounding};
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::LocomotionClass;

use crate::combat::damage::{self, Hit};
use crate::combat::data::{CombatRules, WarheadId};
use crate::combat::veterancy::{self, Rank};
use crate::econ::{TypeRef, production, tech};
use crate::rules::{Kind, OverlayRules};
use crate::world::{HouseId, ObjectId, Pos, World};

/// The number of crate places (S-0520 Constants, claim 1).
pub const PLACES: usize = 256; // spec: S-0520 Constants (crate capacity 256)

/// The number of power-ups (S-0520 Constants).
pub const POWERUPS: usize = 19; // spec: S-0520 Constants (power-up count 19)

/// The contents value our engine stores for a random crate (S-0520 Terms: any
/// value above 18 means random contents).
const RANDOM: u8 = 255; // spec: S-0520 Terms (a value other than 0 … 18 is random contents; our choice 255)

/// The blasts of the explosion power-up (S-0520 Constants).
const BLASTS: i32 = 5; // spec: S-0520 Constants (explosion blasts 5)

/// The neighbour cells of the gas power-up (S-0520 claim 23).
const NEIGHBOURS: u8 = 8; // spec: S-0520 claim 23 (the eight neighbours)

/// Placement attempts of one random placement (S-0520 Constants).
const ATTEMPTS: i32 = 1000; // spec: S-0520 Constants (placement attempts)

const MONEY: usize = 0; // spec: S-0520 claim 8 (the power-up numbering of the table)
const UNIT: usize = 1; // spec: S-0520 claim 8 (the power-up numbering of the table)
const HEAL: usize = 2; // spec: S-0520 claim 8 (the power-up numbering of the table)
const CLOAK: usize = 3; // spec: S-0520 claim 8 (the power-up numbering of the table)
const EXPLOSION: usize = 4; // spec: S-0520 claim 8 (the power-up numbering of the table)
const NAPALM: usize = 5; // spec: S-0520 claim 8 (the power-up numbering of the table)
const SQUAD: usize = 6; // spec: S-0520 claim 8 (the power-up numbering of the table)
const DARKNESS: usize = 7; // spec: S-0520 claim 8 (the power-up numbering of the table)
const REVEAL: usize = 8; // spec: S-0520 claim 8 (the power-up numbering of the table)
const ARMOUR: usize = 9; // spec: S-0520 claim 8 (the power-up numbering of the table)
const SPEED: usize = 10; // spec: S-0520 claim 8 (the power-up numbering of the table)
const FIREPOWER: usize = 11; // spec: S-0520 claim 8 (the power-up numbering of the table)
const NUKE: usize = 12; // spec: S-0520 claim 8 (the power-up numbering of the table)
const VETERAN: usize = 14; // spec: S-0520 claim 8 (the power-up numbering of the table)
const GAS: usize = 16; // spec: S-0520 claim 8 (the power-up numbering of the table)
const RESOURCE: usize = 17; // spec: S-0520 claim 8 (the power-up numbering of the table)

/// A placed crate (S-0520 Terms, *crate places*).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Place {
    /// The crate's cell.
    pub cell: CellXY,
    /// The tick the lifetime began.
    pub start: i32,
    /// The lifetime in ticks.
    pub life: i32,
    /// Placed by a trigger action (S-0524 claim 4): it may outlive its
    /// lifetime while crates are off.
    pub trigger: bool,
}

/// All crate state of a game.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CrateWorld {
    /// The occupied crate places by priority number 0 … 255.
    pub places: BTreeMap<u16, Place>,
    /// The scenario's any-crate-picked-up mark (claim 7, event 50).
    pub any_picked: bool,
}

/// The crate rules of a ruleset (S-0520 Import mapping).
#[derive(Clone, Debug)]
pub struct CrateRules {
    /// Weights by power-up.
    pub weight: [i32; POWERUPS],
    /// Animation type names by power-up.
    pub anim: [Option<String>; POWERUPS],
    /// Water flags by power-up.
    pub water: [bool; POWERUPS],
    /// Arguments by power-up, doubles.
    pub arg: [F64; POWERUPS],
    /// The crate radius ρ in leptons (F3).
    pub radius: i32,
    /// The regeneration field *C* in game minutes, a double.
    pub regen: F64,
    /// The unit-crate type (a vehicle registration index).
    pub unit_type: Option<u32>,
    /// The wooden-crate overlay (an overlay registration index).
    pub wood: Option<u8>,
    /// The water-crate overlay.
    pub water_overlay: Option<u8>,
    /// The flame warhead.
    pub flame: Option<WarheadId>,
    /// The gas warhead (the one named `GAS`; the table always holds one).
    pub gas: Option<WarheadId>,
    /// The harvester list (vehicle registration indices).
    pub harvesters: Vec<u32>,
    /// The refinery list (building registration indices).
    pub refineries: Vec<u32>,
    /// Per vehicle type: the crate-goodie flag.
    pub goodie: Vec<bool>,
}

impl CrateRules {
    /// The crate rules of a resolved ruleset.
    pub fn new(
        r: &Ruleset,
        names: &BTreeMap<(Kind, String), u32>,
        overlays: &[OverlayRules],
        combat: &CombatRules,
        mode: Rounding,
    ) -> Self {
        let s = &r.sim;
        let g = &s.general.economy;
        let z = F64::from_i64(0, mode);
        // spec: S-0520 Import mapping (the built-in table at a fresh start: weights 50, 20, 1, 3, 5, 5, 20, 1, 1, 10, 10, 10, 1, 3, 1, 1, 1, 1, 1; no animations, water flags off, arguments 0.0)
        let defaults: [i32; POWERUPS] = [
            50, 20, 1, 3, 5, 5, 20, 1, 1, 10, 10, 10, 1, 3, 1, 1, 1, 1, 1,
        ];
        let mut weight = defaults;
        let mut anim: [Option<String>; POWERUPS] = Default::default();
        let mut water = [false; POWERUPS];
        let mut arg = [z; POWERUPS];
        if let Some(t) = &g.powerups {
            for (i, p) in t.iter().take(POWERUPS).enumerate() {
                weight[i] = p.weight;
                anim[i] = p.animation.clone();
                water[i] = p.water;
                arg[i] = p.argument.as_ref().map_or(z, |d| d.value(mode));
            }
        }
        // spec: S-0520 F3 (ρ = trunc(v × 256) leptons; absent: 640)
        let radius = g.crate_radius.as_ref().map_or(640, |d| {
            d.value(mode)
                .mul(F64::from_i64(256, mode), mode)
                .to_i32_common()
        });
        // spec: S-0520 Import mapping (CrateRegen absent: 10 game minutes)
        let regen = g
            .crate_regen
            .as_ref()
            .map_or(F64::from_i64(10, mode), |d| d.value(mode));
        let vehicle = |n: &String| names.get(&(Kind::Vehicle, n.clone())).copied();
        let overlay = |n: &Option<String>| -> Option<u8> {
            let n = n.as_ref()?;
            overlays
                .iter()
                .position(|o| &o.name == n)
                .and_then(|i| u8::try_from(i).ok())
        };
        let warhead = |n: &Option<String>| -> Option<WarheadId> {
            let n = n.as_ref()?;
            combat
                .warheads
                .iter()
                .position(|h| &h.name == n)
                .map(|i| i as u32)
        };
        let goodie = (0..r.registries.get("vehicles").map_or(0, Vec::len))
            .map(|i| {
                r.registries
                    .get("vehicles")
                    .and_then(|v| v.get(i))
                    .and_then(|n| s.vehicles.get(n))
                    .is_some_and(|u| u.crate_goodie.unwrap_or(false))
            })
            .collect();
        Self {
            weight,
            anim,
            water,
            arg,
            radius,
            regen,
            unit_type: g.unit_crate_type.as_ref().and_then(vehicle),
            wood: overlay(&g.wood_crate_img),
            water_overlay: overlay(&g.water_crate_img),
            flame: warhead(&g.flame_damage),
            // spec: S-0520 Edge cases (the gas warhead is the one registered under the fixed name; the table adds a default one when none exists)
            gas: combat
                .warheads
                .iter()
                .position(|h| h.name.eq_ignore_ascii_case("GAS"))
                .map(|i| i as u32),
            harvesters: s
                .general
                .harvester_unit
                .iter()
                .flatten()
                .filter_map(vehicle)
                .collect(),
            refineries: g
                .build_refinery
                .iter()
                .flatten()
                .filter_map(|n| names.get(&(Kind::Building, n.clone())).copied())
                .collect(),
            goodie,
        }
    }
}

// ---------------------------------------------------------------------------
// The crate point and the crate area

/// The crate point *P* of a cell (S-0520 Terms).
fn point(w: &World, c: CellXY) -> Pos {
    let (x, y) = (256 * c.x + 128, 256 * c.y + 128); // spec: S-0520 Terms (the crate point: the cell centre at its floor height)
    Pos::new(x, y, w.surface_at(x, y))
}

/// Whether an object is inside the crate area of `p` (F4): the table square
/// root of the sum of squares, truncated, below ρ.
fn inside(w: &World, id: ObjectId, p: Pos) -> bool {
    let q = w.reference_point(id); // spec: S-0520 Terms (a building is measured to its reference point, any other object to its position, which `reference_point` gives)
    let sq = |a: i32| i64::from(a) * i64::from(a);
    let n = sq(q.x.wrapping_sub(p.x)) + sq(q.y.wrapping_sub(p.y)) + sq(q.z.wrapping_sub(p.z));
    let r = w.rules();
    let d = glasnost_num::geom::root_floor(n as u64, r.quirks.table_root_distance, r.mode) as i64;
    d < i64::from(r.crates.radius) // spec: S-0520 F4 (inside means d < ρ, a strict comparison in leptons)
}

/// The actor kinds of a crate effect (claims 15, 19, 20, 22).
fn actor(k: Kind) -> bool {
    matches!(
        k,
        Kind::Building | Kind::Vehicle | Kind::Infantry | Kind::Aircraft
    )
}

/// The objects inside the crate area, by object number; `on_map` restricts
/// them to those on the map.
fn in_area(w: &World, p: Pos, on_map: bool) -> Vec<ObjectId> {
    w.objects
        .values()
        .filter(|o| actor(o.kind) && (o.on_map || !on_map) && inside(w, o.id, p))
        .map(|o| o.id)
        .collect()
}

// ---------------------------------------------------------------------------
// Placement and expiry (claims 1 to 5)

/// Whether a cell holds a crate overlay.
pub fn is_crate(w: &World, c: CellXY) -> bool {
    w.cells.cell(c).is_some_and(|x| {
        x.overlay != NO_OVERLAY
            && w.rules()
                .overlays
                .get(usize::from(x.overlay))
                .is_some_and(|o| o.crate_)
    })
}

/// The lowest free crate place (claim 1).
fn free_place(w: &World) -> Option<u16> {
    (0..PLACES as u16).find(|p| !w.crates.places.contains_key(p))
}

/// The lifetime of S-0524 claim 4 in ticks.
fn lifetime(w: &mut World) -> i32 {
    let r = w.rules_arc();
    let mode = r.mode;
    let c = r.crates.regen;
    let f = |n: i64| F64::from_i64(n, mode);
    let l = f(450).mul(c, mode); // spec: S-0524 Formulas (L = RZ(450 C); 900 ticks per game minute)
    let h = f(1800).mul(c, mode); // spec: S-0524 Formulas (H = RZ(1800 C); T = trunc(RZ(L + RZ(RZ(H − L) × u))))
    let u = w.rng.unit(mode);
    l.add(h.sub(l, mode).mul(u, mode), mode).to_i32_common()
}

/// The search of a candidate or trigger cell (claims 3 and S-0524 claim 4):
/// *float* class for water, *track* otherwise, *normal* zone, no reach, no
/// deck, no height condition, bridges allowed, no reference cell.
fn search(w: &mut World, centre: CellXY) -> Option<CellXY> {
    crate::regions::ensure(w);
    // spec: S-0520 claim 3 (a candidate outside the cell array is treated as ordinary land)
    let wet = w.cells.cell(centre).is_some_and(|c| c.land == land::WATER);
    let settings = crate::path::NearbySettings {
        class: Some(if wet {
            LocomotionClass::Float
        } else {
            LocomotionClass::Track
        }),
        zone: glasnost_rules::sim::MovementZone::Normal,
        reach: None,
        deck: false,
        height: false,
        reference: None,
        no_bridges: false,
        no_overlay: false,
    };
    crate::path::nearby(w, centre, crate::path::NearbyUse::Settings(settings))
}

/// Puts a crate on the found cell with `contents` when the cell passes the
/// tests of claim 3; the lifetime draw follows any overlay draws.
fn finish(w: &mut World, found: Option<CellXY>, contents: u8, trigger: bool) -> bool {
    let Some(c) = found else { return false };
    // spec: S-0520 claim 3 (the selected cell must be in the height-aware usable area and hold no overlay)
    let Some(cell) = w.cells.cell(c) else {
        return false;
    };
    if !w.cells.usable(c) || cell.overlay != NO_OVERLAY {
        return false;
    }
    let Some(place) = free_place(w) else {
        return false;
    };
    let r = w.rules_arc();
    let wet = cell.land == land::WATER;
    // spec: S-0520 claim 3 (the water-crate overlay on water, the wooden-crate overlay otherwise; a missing overlay is unsafe data: no crate)
    let Some(o) = (if wet {
        r.crates.water_overlay
    } else {
        r.crates.wood
    }) else {
        w.note("S-0520 claim 3: no crate overlay is named for this land: no crate is placed");
        return false;
    };
    crate::overlays::place_overlay(w, c, o);
    if let Some(x) = w.cells.grid.get_mut(c) {
        x.overlay_data = contents; // spec: S-0520 Terms (the overlay data value is the contents)
    }
    // spec: S-0520 claim 3 (any overlay draws precede the lifetime request, which begins at the current tick)
    let life = lifetime(w);
    let start = w.tick;
    w.crates.places.insert(
        place,
        Place {
            cell: c,
            start,
            life,
            trigger,
        },
    );
    true
}

/// Whether the rules name both crate overlays. A missing one is unsafe
/// original data: our engine warns and gives every placement no effect, with
/// no draw (S-0524 Edge cases, S-0520 claim 3).
fn overlays_named(w: &mut World) -> bool {
    let r = w.rules_arc();
    if r.crates.wood.is_some() && r.crates.water_overlay.is_some() {
        return true;
    }
    w.note(
        "S-0520 claim 3: the rules name no wooden-crate or water-crate overlay: no crate is placed",
    );
    false
}

/// One random placement (claim 3): at most 1000 attempts, each two ranged
/// requests (F5) and a single-cell search. A placement with no free place
/// takes no draw.
pub fn place_random(w: &mut World) -> bool {
    if free_place(w).is_none() || !overlays_named(w) {
        // spec: S-0520 claim 1 (every place occupied: nothing and no draw)
        return false;
    }
    let half = w.rules().quirks.ranged_draw_half_span;
    let fp = *w.cells.footprint();
    // spec: S-0520 F5 (x = 1 + R(0, W + H − 2), then y likewise)
    let span = fp.width + fp.height - 2;
    for _ in 0..ATTEMPTS {
        let x = 1 + w.rng.ranged(0, span, half);
        let y = 1 + w.rng.ranged(0, span, half);
        let found = search(w, CellXY::new(x, y));
        // spec: S-0520 claim 3 (the first attempt that meets every condition ends the placement)
        if finish(w, found, RANDOM, false) {
            return true;
        }
    }
    false
}

/// A crate at a trigger action's cell (S-0524 claim 4): one search, no retry.
/// `kind` is the number slot's value.
pub fn place_trigger(w: &mut World, at: CellXY, kind: i32) -> bool {
    if w.rules().quirks.trigger_crate_respects_setting && !crate::econ::setup::crates(w) {
        // spec: S-0524 Edge cases (quirks.trigger_crate_respects_setting: no effect with crates off)
        return false;
    }
    if free_place(w).is_none() || !overlays_named(w) {
        // spec: S-0520 claim 1 (capacity: no draw)
        return false;
    }
    // spec: S-0524 claim 4 (the value 20 leaves the crate random; any other value is reduced modulo 256: 0 … 18 fixed, 19 … 255 random)
    let contents = if kind == 20 {
        RANDOM
    } else {
        u8::try_from(kind.rem_euclid(256)).unwrap_or(RANDOM)
    };
    let found = search(w, at);
    finish(w, found, contents, true)
}

/// A crate from a death (claims 5): random contents, or the money power-up.
pub fn place_death(w: &mut World, at: CellXY, money: bool) -> bool {
    if free_place(w).is_none() || !overlays_named(w) {
        return false;
    }
    let found = search(w, at);
    finish(w, found, if money { 0 } else { RANDOM }, false) // spec: S-0520 claim 5 (a building's crate beneath holds the money power-up (0) when the type says so; every other death crate is random)
}

/// The game-start placements (claim 2): `n` random placements in order.
pub fn place_at_start(w: &mut World, n: i32) {
    for _ in 0..n {
        place_random(w);
    }
}

/// Removes the crate of a cell with its place.
fn remove_crate(w: &mut World, c: CellXY) {
    w.crates.places.retain(|_, p| p.cell != c);
    crate::crush::remove_overlay(w, c);
}

/// The crate-update part of the tick (claim 4, S-0203 claim 1 row 16):
/// crates whose lifetime has run out expire, each replaced by one random
/// placement, in increasing place-priority order.
pub fn update(w: &mut World) {
    let on = crate::econ::setup::crates(w);
    let keep_trigger = !w.rules().quirks.trigger_crate_expiry_while_disabled;
    if !on && w.crates.places.values().all(|p| !p.trigger || keep_trigger) {
        // spec: S-0520 claim 4 (crates off: the update does nothing, trigger crates included)
        return;
    }
    let now = w.tick;
    let due: Vec<(u16, Place)> = w
        .crates
        .places
        .iter()
        .filter(|(_, p)| now.wrapping_sub(p.start) >= p.life) // spec: S-0520 claim 4 (expiry when the elapsed time reaches the lifetime)
        .map(|(&k, &p)| (k, p))
        .filter(|(_, p)| on || (p.trigger && !keep_trigger))
        .collect();
    for (k, p) in due {
        // spec: S-0520 claim 4 (each expiry frees its place and its crate; the replacement sees it free)
        w.crates.places.remove(&k);
        crate::crush::remove_overlay(w, p.cell);
        if on {
            // spec: S-0520 claim 4 (one random placement per expiry, which can fail)
            place_random(w);
        }
    }
}

// ---------------------------------------------------------------------------
// Pickup (claims 6 to 11)

/// A ground mover commits to a step into `cell`, a jumpjet or teleporter has
/// landed on it (claim 6). Returns whether a ground step is blocked
/// (claims 7 and 13); a landed object is never moved back.
pub fn pickup(w: &mut World, collector: ObjectId, cell: CellXY) -> bool {
    if !is_crate(w, cell) {
        // spec: S-0520 claim 7 (the cell's overlay lacks the crate flag: no pickup)
        return false;
    }
    let Some(o) = w.obj(collector) else {
        return false;
    };
    let Some(house) = o.owner else { return false };
    // spec: S-0520 claim 7 (the collecting house's country has the passive flag: no pickup, the crate stays)
    if crate::econ::setup::passive(w, house) {
        return false;
    }
    let r = w.rules_arc();
    let overlay = w.cells.cell(cell).map_or(0, |c| c.overlay);
    let trigger_flag = r
        .overlays
        .get(usize::from(overlay))
        .is_some_and(|o| o.crate_trigger);
    if trigger_flag {
        // spec: S-0520 claim 7 (a collector with a tag raises event 49 with itself as the causing object; a collector that no longer exists afterwards: the crate stays and a ground step is blocked)
        crate::triggers::crate_picked_up(w, collector);
        if w.obj(collector).is_none() {
            return true;
        }
        // spec: S-0520 claim 7 (the pickup continues: the any-crate mark is set)
        w.crates.any_picked = true;
    }
    let contents = w.cells.cell(cell).map_or(RANDOM, |c| c.overlay_data);
    let (mut p, forced) = choose(w, contents, collector, house, cell);
    p = convert(w, p, collector, house, cell);
    // spec: S-0520 claim 11 (the crate is removed and its place freed; with crates on one random replacement is placed before any effect draw)
    remove_crate(w, cell);
    if crate::econ::setup::crates(w) {
        place_random(w);
    }
    let (shown, blocked) = effect(w, p, forced, collector, house, cell);
    if let Some(name) = r.crates.anim[shown].clone() {
        // spec: S-0520 claim 11 (the animation appears 200 leptons above the crate point's floor height; a successful unit has none, and its creation draws follow every effect draw)
        let mut at = point(w, cell);
        at.z += 200; // spec: S-0520 Constants (animation height 200 leptons)
        crate::combat::effects::anim(w, &name, at);
    }
    blocked
}

/// Claims 8 and 9: the power-up and a forced base unit.
fn choose(
    w: &mut World,
    contents: u8,
    collector: ObjectId,
    house: HouseId,
    cell: CellXY,
) -> (usize, Option<u32>) {
    let _ = (collector, cell);
    let r = w.rules_arc();
    let mut p = if usize::from(contents) < POWERUPS {
        // spec: S-0520 claim 8 (contents 0 … 18 are the power-up)
        usize::from(contents)
    } else {
        // spec: S-0520 F1 (the sum of the weights in signed 32-bit arithmetic; a non-positive total is unsafe data and gives money)
        let total = r.crates.weight.iter().fold(0i32, |a, &x| a.wrapping_add(x));
        if total <= 0 {
            w.note("S-0520 Edge cases: the power-up weights add up to 0 or less: the crate gives money");
            MONEY
        } else {
            let half = r.quirks.ranged_draw_half_span;
            // spec: S-0520 claim 8 and F1 (r = R(1, total); the first power-up whose running total reaches r)
            let pick = w.rng.ranged(1, total, half);
            let mut acc = 0i32;
            let mut chosen = MONEY;
            for (i, &x) in r.crates.weight.iter().enumerate() {
                acc = acc.wrapping_add(x);
                if acc >= pick {
                    chosen = i;
                    break;
                }
            }
            chosen
        }
    };
    // spec: S-0520 claim 9 (a forced base unit: no crate building, more than 1500 credits, no unit of the first ownable base-unit type, bases setting on)
    let mut forced = None;
    if crate::econ::setup::bases(w)
        && building_count(w, house) == 0
        && production::available_credits(w, house) > 1500 // spec: S-0520 Constants (forced-base-unit credit threshold, strictly more than)
        && let Some(t) = base_unit_type(w, house)
        && tech::owned_count(
            w,
            house,
            TypeRef {
                kind: Kind::Vehicle,
                index: t,
            },
        ) == 0
    {
        p = UNIT;
        forced = Some(t);
    }
    (p, forced)
}

/// The first entry of the base-unit list that the house's country may own.
fn base_unit_type(w: &World, house: HouseId) -> Option<u32> {
    let country = w.houses.get(usize::from(house.0))?.country;
    w.rules().base_units.iter().copied().find(|&t| {
        w.rules()
            .econ
            .tech
            .of(TypeRef {
                kind: Kind::Vehicle,
                index: t,
            })
            .is_some_and(|f| f.owners.contains(&country))
    })
}

/// The crate counts of a house (Terms).
fn building_count(w: &World, h: HouseId) -> i32 {
    crate::econ::victory::counted_buildings(w, h)
}

/// The crate infantry count of a house (Terms): its infantry in existence,
/// off the map included, without insignificant or unscored types, nominal
/// survivors and absorbed infantry (S-0605 claims 7, 13 and 14). No crate
/// rule reads it yet (the squad crate gives money whatever the count).
pub fn infantry_count(w: &World, h: HouseId) -> i32 {
    w.objects
        .values()
        .filter(|o| o.owner == Some(h) && o.kind == Kind::Infantry)
        .filter(|o| {
            let (ds, ins) = crate::econ::victory::flags(w, o.kind, o.type_index);
            // spec: S-0520 Terms (the infantry count excludes nominal survivors and absorbed infantry, S-0312 claim 24, S-0605 claim 7)
            !(ds || ins || o.econ.counted.nominal_survivor || o.structure.absorbed_in.is_some())
        })
        .count() as i32
}

fn vehicle_count(w: &World, h: HouseId) -> i32 {
    let r = &w.rules().econ.victory;
    w.objects
        .values()
        .filter(|o| o.owner == Some(h))
        .filter(|o| {
            let (ds, ins) = crate::econ::victory::flags(w, o.kind, o.type_index);
            if ds || ins {
                return false;
            }
            match o.kind {
                Kind::Vehicle => true,
                Kind::Building => r
                    .vehicle_building
                    .get(o.type_index as usize)
                    .copied()
                    .unwrap_or(false),
                _ => false,
            }
        })
        .count() as i32
}

/// Claim 10: the conversions to money.
fn convert(w: &World, p: usize, collector: ObjectId, house: HouseId, cell: CellXY) -> usize {
    let r = w.rules();
    let Some(o) = w.obj(collector) else {
        return p;
    };
    let land_of = w.cells.cell(cell).map_or(land::CLEAR, |c| c.land);
    let wet = land_of == land::WATER;
    let shore = wet || land_of == land::BEACH;
    let one = F64::from_i64(1, r.mode);
    let tc = veterancy::type_combat(w, collector);
    let mut p = p;
    // spec: S-0520 claim 10 (each row converts its power-up to money when the collector or the house cannot use it)
    let money = match p {
        UNIT => vehicle_count(w, house) > 50 || shore, // spec: S-0520 Constants (unit power-up vehicle limit)
        SQUAD => true, // spec: S-0520 claims 10 and 11 (squad gives the money effect whatever the infantry count and the land)
        CLOAK => o.cloak.own,
        ARMOUR => !o.combat.armour_mult.eq_num(one),
        SPEED => !o.speed_bonus.eq_num(one) || o.kind == Kind::Aircraft,
        FIREPOWER => {
            !o.combat.firepower_mult.eq_num(one)
                || crate::combat::weapon::slot_weapon(w, collector, 0).is_none()
        }
        VETERAN => !tc.is_some_and(|t| t.trainable) || veterancy::rank(w, collector) == Rank::Elite,
        _ => false,
    };
    if money {
        p = MONEY;
    }
    // spec: S-0520 claim 10 (the water gate: a power-up whose water flag is off becomes money on a water cell; money stays money)
    if wet && !r.crates.water[p] {
        p = MONEY;
    }
    p
}

// ---------------------------------------------------------------------------
// Effects (claims 12 to 25)

/// Plays a power-up: returns the power-up whose animation shows and whether
/// a ground collector's step is blocked.
fn effect(
    w: &mut World,
    p: usize,
    forced: Option<u32>,
    collector: ObjectId,
    house: HouseId,
    cell: CellXY,
) -> (usize, bool) {
    let r = w.rules_arc();
    let half = r.quirks.ranged_draw_half_span;
    let at = point(w, cell);
    // spec: S-0520 F2 (b = trunc(A) of the power-up that gives the effect)
    let arg = |i: usize| r.crates.arg[i].to_i32_common();
    match p {
        MONEY | SQUAD => {
            let b = arg(MONEY);
            give_money(w, house, b);
            (MONEY, false)
        }
        UNIT => match unit(w, forced, collector, house, cell) {
            true => (UNIT, true),
            false => {
                // spec: S-0520 claim 13 and quirks.crate_unit_fallback_unit_argument (no unit could be placed: money with the money animation, based on the unit power-up's argument; off: the money argument)
                let b = if r.quirks.crate_unit_fallback_unit_argument {
                    arg(UNIT)
                } else {
                    arg(MONEY)
                };
                give_money(w, house, b);
                (MONEY, false)
            }
        },
        HEAL => {
            heal_all(w, house);
            (p, false)
        }
        CLOAK => {
            for id in in_area(w, at, true) {
                // spec: S-0520 claim 15 (every building, vehicle, infantry unit and aircraft on the map inside the crate area, every owner, gets its own cloak flag)
                crate::cloak::machine::grant_own_flag(w, id);
            }
            (p, false)
        }
        EXPLOSION => {
            explosion(w, collector, at, arg(EXPLOSION), half);
            (p, false)
        }
        NAPALM => {
            napalm(w, collector, at, arg(NAPALM));
            (p, false)
        }
        DARKNESS => {
            // spec: S-0520 claim 18 (the whole map is re-covered for the collecting house, S-0409 claim 15)
            crate::econ::shroud::crate_darkness(w, house);
            (p, false)
        }
        REVEAL => {
            // spec: S-0520 claim 18 (the whole map is revealed for the collecting house, S-0409 claim 15)
            crate::econ::shroud::crate_reveal(w, house);
            (p, false)
        }
        ARMOUR | FIREPOWER | SPEED => {
            bonus(w, p, at, r.crates.arg[p]);
            (p, false)
        }
        NUKE => {
            nuke(w, house);
            (p, false)
        }
        VETERAN => {
            veteran(w, at, r.crates.arg[VETERAN]);
            (p, false)
        }
        GAS => {
            gas(w, cell, arg(GAS));
            (p, false)
        }
        RESOURCE => {
            resource(w, at, half);
            (p, false)
        }
        // spec: S-0520 claim 25 (invulnerability, weather storm and drop pod only create their animation)
        _ => (p, false),
    }
}

/// Claim 12: money R(b, b + 900).
fn give_money(w: &mut World, house: HouseId, b: i32) {
    let half = w.rules().quirks.ranged_draw_half_span;
    // spec: S-0520 F2 (amount = R(b, b + 900), with b + 900 in signed 32-bit arithmetic)
    let amount = w.rng.ranged(b, b.wrapping_add(900), half);
    if let Some(h) = w.houses.get_mut(usize::from(house.0)) {
        h.credits = h.credits.wrapping_add(amount);
    }
}

/// Claim 13: the unit power-up. Returns whether a unit was placed.
fn unit(
    w: &mut World,
    forced: Option<u32>,
    collector: ObjectId,
    house: HouseId,
    cell: CellXY,
) -> bool {
    let r = w.rules_arc();
    let _ = collector;
    let half = r.quirks.ranged_draw_half_span;
    let ti = if let Some(t) = r.crates.unit_type {
        // spec: S-0520 claim 13 (the unit-crate type takes precedence, even over a forced base unit)
        Some(t)
    } else if let Some(t) = forced {
        // spec: S-0520 claim 13 (otherwise a forced base unit uses the house's base-unit type)
        Some(t)
    } else if let Some(t) = harvester_rule(w, house) {
        Some(t)
    } else {
        random_vehicle(w, house, half)
    };
    let Some(ti) = ti else { return false };
    // spec: S-0520 claim 13 (created for the collecting house with its creation draw, facing 0)
    let id = crate::objects::create(w, Kind::Vehicle, ti, Some(house));
    let Some(u) = r.unit(Kind::Vehicle, ti) else {
        crate::objects::remove(w, id);
        return false;
    };
    crate::regions::ensure(w);
    // spec: S-0520 claim 13 (revision 3, Q-0987 answered: first a direct placement attempt at the crate cell centre at ground height, which is not the first candidate of the search; it accepts only the clear entry outcome of the awarded unit)
    if clear_for(w, id, cell) {
        let at = point(w, cell);
        crate::objects::place(w, id, at, 0);
        return true;
    }
    // spec: S-0520 claim 13 (then the nearby search around the crate cell: the type's class, normal zone (not the type's own), no reach, no deck, no height condition, no further test, no no-overlay condition, bridges allowed, no reference cell; it requires the empty occupancy record of S-0212 claim 25)
    let settings = crate::path::NearbySettings {
        class: u.class,
        zone: glasnost_rules::sim::MovementZone::Normal,
        reach: None,
        deck: false,
        height: false,
        reference: None,
        no_bridges: false,
        no_overlay: false,
    };
    let found = crate::path::nearby(w, cell, crate::path::NearbyUse::Settings(settings));
    // spec: S-0520 claim 13 (a search result is offered the same direct placement test, once, at its cell centre; a refused result does not start another search)
    let Some(found) = found.filter(|&c| clear_for(w, id, c)) else {
        // spec: S-0520 claim 13 (no cell, or the placement fails: the unit is deleted, its creation draw stays taken)
        crate::objects::remove(w, id);
        return false;
    };
    let at = point(w, found);
    crate::objects::place(w, id, at, 0);
    true
}

/// The direct placement test of claim 13: the awarded unit's entry outcome
/// for the cell is clear (S-0209 claim 5, S-0211 claim 10), with no approach
/// direction or source elevation, the ground layer's occupants and ground
/// occupancy record, and no forced-placement exemption. Every other outcome
/// refuses; the collector counts by its actual listing and reservations.
fn clear_for(w: &World, unit: ObjectId, cell: CellXY) -> bool {
    // spec: S-0520 claim 13 (revision 3: the reference level of the asked cell itself, so the ground layer's objects and record are read)
    let level = w.cells.cell(cell).map_or(0, |c| i32::from(c.level));
    crate::occupancy::vehicle_outcome(w, unit, cell, level) == crate::occupancy::Outcome::Clear
}

/// The harvester rule of claim 13.
fn harvester_rule(w: &World, house: HouseId) -> Option<u32> {
    let r = &w.rules().crates;
    let prefix = w.rules().quirks.crate_harvester_list_prefix;
    let n = |list: &[u32], kind: Kind| -> Vec<u32> {
        let take = if prefix { 2 } else { list.len() }; // spec: S-0520 claim 13 and quirks.crate_harvester_list_prefix (only the first two entries of each list; off: all)
        let _ = kind;
        list.iter().take(take).copied().collect()
    };
    let count = |kind: Kind, i: u32| tech::owned_count(w, house, TypeRef { kind, index: i });
    let refineries = n(&r.refineries, Kind::Building);
    let harvesters = n(&r.harvesters, Kind::Vehicle);
    let has_refinery = refineries.iter().any(|&i| count(Kind::Building, i) > 0);
    let has_harvester = harvesters.iter().any(|&i| count(Kind::Vehicle, i) > 0);
    if !(has_refinery && !has_harvester) {
        return None;
    }
    let country = w.houses.get(usize::from(house.0))?.country;
    // spec: S-0520 claim 13 (the first harvester-list type its country may own; none ownable: the random rule)
    r.harvesters.iter().copied().find(|&t| {
        w.rules()
            .econ
            .tech
            .of(TypeRef {
                kind: Kind::Vehicle,
                index: t,
            })
            .is_some_and(|f| f.owners.contains(&country))
    })
}

/// The random vehicle of claim 13: requests R(0, n − 1) until a type with the
/// crate-goodie flag is accepted (a base-unit type is refused with the bases
/// setting off).
fn random_vehicle(w: &mut World, _house: HouseId, half: bool) -> Option<u32> {
    let r = w.rules_arc();
    let n = r.vehicles.len() as i32;
    let bases = crate::econ::setup::bases(w);
    // spec: S-0520 Edge cases (no vehicle type can ever be accepted: the original never stops redrawing; our engine gives money after a bounded search with no draw)
    let acceptable = |i: usize| {
        r.crates.goodie.get(i).copied().unwrap_or(false)
            && (bases || !r.base_units.contains(&(i as u32)))
    };
    if n <= 0 || !(0..n as usize).any(acceptable) {
        w.note("S-0520 Edge cases: no vehicle type is accepted by the unit power-up: the crate gives money");
        return None;
    }
    loop {
        // spec: S-0520 claim 13 (each request R(0, n − 1) among the vehicle types in rules order)
        let i = w.rng.ranged(0, n - 1, half) as usize;
        if acceptable(i) {
            return Some(i as u32);
        }
    }
}

/// Claim 14: every object in play that belongs to the house is offered a
/// healing hit (health − strength).
fn heal_all(w: &mut World, house: HouseId) {
    let warhead = w.rules().combat.general.c4_warhead;
    let ids: Vec<ObjectId> = w
        .objects
        .values()
        .filter(|o| o.owner == Some(house) && actor(o.kind) && w.update.contains(o.id))
        .map(|o| o.id)
        .collect();
    for id in ids {
        let Some(o) = w.obj(id) else { continue };
        let amount = o.health.wrapping_sub(damage::strength(w, id)); // spec: S-0520 claim 14 (a hit of health − strength, signed)
        damage::apply(
            w,
            id,
            Hit {
                amount,
                warhead,
                distance: 0,
                attacker: None,
                house: None,
                ignore_defences: true,
                no_escape: true,
            },
        );
    }
}

/// Claim 16: explosion.
fn explosion(w: &mut World, collector: ObjectId, p: Pos, d: i32, half: bool) {
    let Some(wh) = w.rules().combat.general.c4_warhead else {
        return;
    };
    damage::apply(
        w,
        collector,
        Hit {
            amount: d,
            warhead: Some(wh),
            distance: 0,
            attacker: None,
            house: None,
            ignore_defences: true,
            no_escape: false,
        },
    );
    for _ in 0..BLASTS {
        // spec: S-0520 Constants (five blasts) and claim 16 (r = R(0, 512), then the nearby point of S-0312 F2 with one raw draw, no snapping)
        let rr = w.rng.ranged(0, 512, half);
        let b = w.rng.raw();
        let q = crate::combat::death::nearby_point(w, p, rr, b, false);
        crate::combat::area::detonate(
            w,
            crate::combat::area::Detonation {
                point: q,
                damage: d,
                warhead: Some(wh),
                source: None,
                house: None,
                resources: true,
            },
        );
        // spec: S-0520 claim 16 (the explosion animation of S-0303 F5 as if the land were not water)
        if let Some(name) = crate::combat::projectile::explosion_anim_dry(w, wh, d, q) {
            crate::combat::effects::anim(w, &name, q);
        }
    }
}

/// Claim 17: napalm.
fn napalm(w: &mut World, collector: ObjectId, p: Pos, d: i32) {
    let Some(flame) = w.rules().crates.flame else {
        w.note("S-0520 Edge cases: no flame warhead: napalm skips its hit and its area damage");
        return;
    };
    let c = w.reference_point(collector);
    // spec: S-0520 F6 (the midpoint, each coordinate truncated toward zero)
    let m = Pos::new((p.x + c.x) / 2, (p.y + c.y) / 2, (p.z + c.z) / 2);
    // spec: S-0520 claim 17 (the animation at position 0 of the animation list appears at the midpoint, before the hits)
    if let Some(name) = w.rules().animations.first().map(|a| a.name.clone()) {
        crate::combat::effects::anim(w, &name, m);
    }
    damage::apply(
        w,
        collector,
        Hit {
            amount: d,
            warhead: Some(flame),
            distance: 0,
            attacker: None,
            house: None,
            ignore_defences: true,
            no_escape: false,
        },
    );
    crate::combat::area::detonate(
        w,
        crate::combat::area::Detonation {
            point: m,
            damage: d,
            warhead: Some(flame),
            source: None,
            house: None,
            resources: true,
        },
    );
}

/// Claims 19 and 20: armour, firepower and speed bonuses.
fn bonus(w: &mut World, p: usize, at: Pos, a: F64) {
    let limbo = w.rules().quirks.crate_bonus_reaches_limbo;
    let one = F64::from_i64(1, w.rules().mode);
    let mode = w.rules().mode;
    for id in in_area(w, at, !limbo) {
        let Some(o) = w.obj_mut(id) else { continue };
        match p {
            ARMOUR if o.combat.armour_mult.eq_num(one) => {
                o.combat.armour_mult = o.combat.armour_mult.mul(a, mode); // spec: S-0520 F8 (a multiplier of exactly 1.0 becomes A)
            }
            FIREPOWER if o.combat.firepower_mult.eq_num(one) => {
                o.combat.firepower_mult = o.combat.firepower_mult.mul(a, mode);
            }
            // spec: S-0520 claim 20 (speed: vehicles, infantry and ships only; never aircraft or buildings)
            SPEED
                if matches!(o.kind, Kind::Vehicle | Kind::Infantry)
                    && o.speed_bonus.eq_num(one) =>
            {
                o.speed_bonus = o.speed_bonus.mul(a, mode);
            }
            _ => {}
        }
    }
}

/// Claim 21: the one-time nuclear-missile grant.
fn nuke(w: &mut World, house: HouseId) {
    let sw = &w.rules().superweapons;
    // spec: S-0520 claim 21 (a type of the nuclear-missile kind must exist; the grant is the first type, in rules order, whose action is the nuke action)
    if sw
        .first_of_kind(crate::superweapons::SwKind::Missile)
        .is_none()
    {
        return;
    }
    let Some(i) = sw.types.iter().position(|t| {
        t.action
            .as_deref()
            .is_some_and(|a| a.eq_ignore_ascii_case("nuke"))
    }) else {
        w.note("S-0520 Edge cases: a nuclear-missile type exists but no type has the nuke action: no grant");
        return;
    };
    crate::superweapons::charge::one_time_grant(w, house, i as u32);
}

/// Claim 22: veteran steps.
fn veteran(w: &mut World, at: Pos, a: F64) {
    let mode = w.rules().mode;
    let zero = F64::from_i64(0, mode);
    // spec: S-0520 claim 22 and F7 (only for A > 0: ⌈A⌉ steps, counted as the integers k ≥ 0 with k < A in double precision)
    if !zero.lt(a) {
        return;
    }
    let mut steps = 0i64;
    // spec: S-0520 F7 (one step maps e < 0.0 to 0.0, 0.0 ≤ e < 1.0 to 1.0, 1.0 ≤ e < 2.0 to 2.0, and leaves elite unchanged: three steps reach elite from any experience, so counting past three changes nothing; a huge A no longer loops once per step, #194)
    while steps < 3 && F64::from_i64(steps, mode).lt(a) {
        steps += 1;
    }
    for id in in_area(w, at, true) {
        if !veterancy::type_combat(w, id).is_some_and(|t| t.trainable) {
            continue;
        }
        for _ in 0..steps {
            let Some(o) = w.obj_mut(id) else { break };
            let e = o.combat.experience.to_f64();
            let f = |n| F64::from_i64(n, Rounding::TowardZero);
            // spec: S-0520 F7 (one step: 1.0 ≤ e < 2.0 becomes 2.0; 0.0 ≤ e < 1.0 becomes 1.0; e < 0.0 becomes 0.0; elite is unchanged)
            let next = if e.ge(f(2)) {
                break;
            } else if e.ge(f(1)) {
                f(2)
            } else if e.ge(f(0)) {
                f(1)
            } else {
                f(0)
            };
            o.combat.experience = next.to_f32(Rounding::TowardZero);
        }
    }
}

/// Claim 23: nine area damages with the gas warhead.
fn gas(w: &mut World, cell: CellXY, d: i32) {
    let Some(g) = w.rules().crates.gas else {
        return;
    };
    let mut cells = vec![cell];
    for dir in 0..NEIGHBOURS {
        // spec: S-0520 claim 23 (the eight neighbours in step-direction order 0 through 7)
        cells.push(crate::drive::neighbour(cell, dir));
    }
    for c in cells {
        let p = point(w, c);
        crate::combat::area::detonate(
            w,
            crate::combat::area::Detonation {
                point: p,
                damage: d,
                warhead: Some(g),
                source: None,
                house: None,
                resources: false, // spec: S-0520 claim 23 (resource reduction forbidden)
            },
        );
    }
}

/// Claim 24: resource.
fn resource(w: &mut World, p: Pos, half: bool) {
    let k = w.rules().econ.resource.tiberiums.len() as i32;
    if k <= 0 {
        return;
    }
    // spec: S-0520 claim 24 (a ranged draw selects the resource type; position 1 is treated as position 0)
    let mut t = w.rng.ranged(0, k - 1, half) as usize;
    if t == 1 {
        t = 0;
    }
    crate::econ::resource::crate_stage(w, p.cell(), t);
    // spec: S-0520 claim 24 (n = R(10, 20) further stage-increase attempts)
    let n = w.rng.ranged(10, 20, half);
    for _ in 0..n {
        let r = w.rng.ranged(0, 768, half); // spec: S-0520 Constants (resource spread 0 … 768 leptons)
        let b = w.rng.raw();
        let q = crate::combat::death::nearby_point(w, p, r, b, true);
        crate::econ::resource::crate_stage(w, q.cell(), t);
    }
}

/// A destroyed crate-carrying vehicle or building with a crate beneath
/// leaves a crate (S-0312 claims 14 and 20, S-0520 claim 5) when the game's
/// crate setting allows crates.
pub fn death_crate(w: &mut World, id: ObjectId) {
    if let Some((at, money)) = death_crate_plan(w, id) {
        place_death(w, at, money);
    }
}

/// The crate a killed object leaves (S-0312 claims 14 and 20): the cell
/// it is placed near and whether it is a money crate, or `None`. Reads
/// only.
pub fn death_crate_plan(w: &World, id: ObjectId) -> Option<(CellXY, bool)> {
    let o = w.obj(id)?;
    if !o.combat.killed {
        return None;
    }
    let r = w.rules();
    let (leaves, money) = match o.kind {
        Kind::Vehicle => (
            r.unit(Kind::Vehicle, o.type_index)
                .is_some_and(|u| u.vehicle.carries_crate),
            false,
        ),
        Kind::Building => r
            .buildings
            .get(o.type_index as usize)
            .map_or((false, false), |b| {
                (
                    b.raw.crate_beneath.unwrap_or(false),
                    b.raw.crate_beneath_is_money.unwrap_or(false),
                )
            }),
        _ => (false, false),
    };
    if !leaves || !crate::econ::setup::crates(w) {
        return None;
    }
    // spec: S-0312 claims 14 and 20 (a free cell near the vehicle's position; the building's top-left foundation cell)
    Some((o.pos.cell(), money))
}
