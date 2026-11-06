//! Victory and defeat (S-0414): the defeat tests of the normal and the
//! short game, what defeat does, the end of the game with its end delay and
//! results, and the surrender and alliance orders.
//!
//! # Querying the outcome
//!
//! [`outcome`] answers `None` while the game is not over, and the end tick,
//! the tick in which it is over and every player house's [`Verdict`] once it
//! is. [`ended`] answers whether the end test has succeeded (results
//! pending). [`defeated`] answers per house.
//!
//! The tests run only in a skirmish ([`GameEnd::active`], set by
//! [`crate::econ::setup::start_game`]); the counts of claim 4 are kept in
//! every game. In each house update the power and radar work comes first,
//! then ([`house_update`], called by `econ::houses`) a pending surrender's
//! destruction and the defeat test, then the tech tree (claim 16). The
//! results are committed at the start of the houses' part of the final
//! tick ([`commit`]).

use std::collections::BTreeMap;

use glasnost_num::{F64, Rounding};
use glasnost_rules::schema::Ruleset;
use serde::{Deserialize, Serialize};

use crate::rules::Kind;
use crate::world::{HouseId, HouseRole, ObjectId, Target, World};

/// A house's result (S-0414 claim 7).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Verdict {
    /// The house won.
    Win,
    /// The house lost.
    Loss,
}

/// A house's defeat, result and counts (S-0414 claims 4, 5, 7 and 11).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HouseResult {
    /// Defeated (claim 5), for the rest of the game.
    pub defeated: bool,
    /// Marked to die by a surrender order (claim 11).
    pub doomed: bool,
    /// The result, fixed at the end of the game (claims 7 and 9).
    pub verdict: Option<Verdict>,
    /// *V*, *I*, *A*, *R*: the on-map counts of claim 4.
    pub on_map: [i32; 4], // spec: S-0414 claim 4 (the four on-map counts V, I, A, R)
    /// Entries onto the map of each not-scored vehicle type as this
    /// house's (S-0407 Terms reads it; S-0414 claim 4).
    pub unscored: BTreeMap<u32, i32>,
    /// The result screen's counters (claims 21 to 23).
    pub stats: super::stats::Stats,
}

/// An object's part in the counts (S-0414 claim 4).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Counted {
    /// A nominal survivor (Terms). Set by [`mark_nominal_survivor`], which
    /// the survivor rules call; which buildings' survivors are nominal is
    /// not identified (S-0414 Terms, low confidence), so nothing sets it yet.
    pub nominal_survivor: bool,
}

/// The end of the game (S-0414 claims 6 to 10).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameEnd {
    /// The game is a skirmish whose houses are tested (claim 1).
    pub active: bool,
    /// The tick of the end of the game (claim 6).
    pub ended: Option<i32>,
    /// The tick in which the game is over (claims 8 and 9).
    pub over: Option<i32>,
    /// The results are committed (claim 16): later defeats change neither
    /// a result nor the tick in which the game is over.
    #[serde(default)]
    pub committed: bool,
    /// The final tick has run (claim 8): the tick number stays at it
    /// (S-0201 claim 1).
    #[serde(default)]
    pub finished: bool,
    /// Production objects deleted at once (S-0404 claim 6) during the
    /// current destruction hit (claim 15); `None` outside a hit.
    #[serde(skip)]
    pub deleted: Option<Vec<ObjectId>>,
}

/// The game's outcome once it is over.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Outcome {
    /// The tick of the end of the game (claim 6).
    pub ended: i32,
    /// The tick in which it is over (claims 8 and 9).
    pub over: i32,
    /// Every player house's result, in house order (claim 7): no neutral,
    /// special, removed or observer's house.
    pub results: Vec<(HouseId, Verdict)>,
}

/// Which on-map count an object is on (S-0414 claim 4).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Slot {
    V,
    I,
    A,
    R,
}

/// The victory rules (S-0414 Import mapping).
#[derive(Clone, Debug, Default)]
pub struct Rules {
    /// The end delay *D* in ticks (Formulas).
    pub delay: i32,
    /// The allies-allowed and ally-change-allowed flags (claim 14).
    pub allies_allowed: bool,
    /// See `allies_allowed`.
    pub ally_change_allowed: bool,
    /// The vehicle types of the short-game test (claim 2).
    pub short_types: Vec<u32>,
    /// The refinery-list building type (claim 3).
    pub refinery: Option<u32>,
    /// Per building type: counts as a vehicle (Terms).
    pub vehicle_building: Vec<bool>,
    /// Per kind and type: marked not scored, marked insignificant.
    pub flags: BTreeMap<(Kind, u32), (bool, bool)>,
}

impl Rules {
    /// The victory rules of a resolved ruleset.
    pub fn new(r: &Ruleset, mode: Rounding) -> Self {
        let s = &r.sim;
        let g = &s.general.economy;
        let reg = |k: &str| r.registries.get(k).cloned().unwrap_or_default();
        let vehicles = reg("vehicles");
        let buildings = reg("buildings");
        let find = |list: &[String], n: &str| list.iter().position(|x| x == n).map(|p| p as u32);
        // spec: S-0414 Formulas (D = trunc(RZ₅₃(d × 900)); d absent: the double nearest 0.03)
        let d = g
            .savour_delay
            .as_ref()
            .map_or_else(|| crate::rules::nearest_double("0.03"), |x| x.value(mode));
        // spec: S-0414 Formulas (900 ticks per game minute, glossary)
        let delay = d.mul(F64::from_i64(900, mode), mode).to_i32_common();
        let base: Vec<Option<u32>> = s
            .general
            .base_unit
            .iter()
            .flatten()
            .map(|n| find(&vehicles, n))
            .collect();
        let short_types = if r.globals.quirks.short_game_first_three_base_units {
            // spec: S-0414 claim 2 and Edge cases (only the first three positions; a missing position holds no type)
            base.iter().take(3).flatten().copied().collect()
        } else {
            // spec: S-0414 Edge cases (quirks.short_game_first_three_base_units off: every position)
            base.iter().flatten().copied().collect()
        };
        // spec: S-0414 claim 3 and Edge cases (the third position of the refinery list; none when shorter)
        let refinery = g
            .build_refinery
            .as_ref()
            .and_then(|l| l.get(2))
            .and_then(|n| find(&buildings, n));
        let vehicle_building = buildings
            .iter()
            .map(|n| {
                let b = s.buildings.get(n).cloned().unwrap_or_default();
                let Some(u) = b.undeploys_into.as_deref() else {
                    return false;
                };
                // spec: S-0108 Import mapping (Foundation absent: 1x1)
                let f = b.foundation.unwrap_or([1, 1]);
                let one_cell = f[0].max(1) == 1 && f[1].max(1) == 1;
                // spec: S-0414 Terms (the resource-gatherer flag is the undeploy vehicle type's)
                let gatherer = s
                    .vehicles
                    .get(u)
                    .is_some_and(|v| v.econ.resource_gatherer.unwrap_or(false));
                // spec: S-0414 Terms (an undeploy type, and a one-cell foundation or a resource-gathering undeploy type)
                one_cell || gatherer
            })
            .collect();
        let mut flags = BTreeMap::new();
        for (kind, key) in [
            (Kind::Infantry, "infantry"),
            (Kind::Vehicle, "vehicles"),
            (Kind::Aircraft, "aircraft"),
        ] {
            let map = match kind {
                Kind::Infantry => &s.infantry,
                Kind::Vehicle => &s.vehicles,
                _ => &s.aircraft,
            };
            for (i, n) in reg(key).iter().enumerate() {
                let u = map.get(n).cloned().unwrap_or_default();
                // spec: S-0414 Import mapping (DontScore, Insignificant absent: no)
                let f = (
                    u.combat.dont_score.unwrap_or(false),
                    u.combat.insignificant.unwrap_or(false),
                );
                flags.insert((kind, i as u32), f);
            }
        }
        for (i, n) in buildings.iter().enumerate() {
            let b = s.buildings.get(n).cloned().unwrap_or_default();
            // spec: S-0414 Import mapping (DontScore, Insignificant absent: no)
            let f = (
                b.combat.dont_score.unwrap_or(false),
                b.combat.insignificant.unwrap_or(false),
            );
            flags.insert((Kind::Building, i as u32), f);
        }
        Self {
            delay,
            // spec: S-0414 Import mapping (AlliesAllowed absent: no)
            allies_allowed: g.allies_allowed.unwrap_or(false),
            // spec: S-0414 Import mapping (AllyChangeAllowed absent: yes)
            ally_change_allowed: g.ally_change_allowed.unwrap_or(true),
            short_types,
            refinery,
            vehicle_building,
            flags,
        }
    }
}

fn rules(w: &World) -> &Rules {
    &w.rules().econ.victory
}

fn result(w: &World, h: HouseId) -> Option<&HouseResult> {
    w.houses.get(usize::from(h.0)).map(|x| &x.econ.result)
}

fn result_mut(w: &mut World, h: HouseId) -> Option<&mut HouseResult> {
    w.houses
        .get_mut(usize::from(h.0))
        .map(|x| &mut x.econ.result)
}

// ---------------------------------------------------------------------------
// Queries

/// Whether the house has been defeated (S-0414 claim 5). S-0409 claim 11,
/// S-0410 claim 1 and S-0312 claim 18 (through `seams::game_over`) read it.
pub fn defeated(w: &World, house: HouseId) -> bool {
    result(w, house).is_some_and(|r| r.defeated)
}

/// Whether the game has ended (S-0414 claim 6): the results are pending
/// or fixed.
pub fn ended(w: &World) -> bool {
    w.econ.end.ended.is_some()
}

/// The house's result, once the game has ended (S-0414 claims 7 and 9).
pub fn verdict(w: &World, house: HouseId) -> Option<Verdict> {
    result(w, house).and_then(|r| r.verdict)
}

/// The outcome once the game is over (S-0414 claims 7, 8 and 16): after
/// tick `over`, which runs in full, has run. The tick number then stays at
/// `over` (S-0201 claim 1).
pub fn outcome(w: &World) -> Option<Outcome> {
    let e = &w.econ.end;
    let (ended, over) = (e.ended?, e.over?);
    if !e.finished {
        return None;
    }
    let results = w
        .houses
        .iter()
        .enumerate()
        .filter_map(|(i, h)| h.econ.result.verdict.map(|v| (HouseId(i as u8), v)))
        .collect();
    Some(Outcome {
        ended,
        over,
        results,
    })
}

/// The present count of a don't-score vehicle type for a house (S-0414
/// claim 4, `quirks.unscored_vehicle_stays_counted`; S-0407 Terms reads
/// it): the times a vehicle of the type has entered the map as the house's
/// since game start, or 0 with the switch off.
pub fn unscored_vehicle_entries(w: &World, house: HouseId, vehicle: u32) -> i32 {
    if !w.rules().quirks.unscored_vehicle_stays_counted {
        // spec: S-0414 Edge cases (quirks.unscored_vehicle_stays_counted off: never counted)
        return 0;
    }
    result(w, house)
        .and_then(|r| r.unscored.get(&vehicle).copied())
        .unwrap_or(0)
}

/// Marks an object as a nominal survivor (S-0414 Terms): for the survivor
/// rules of S-0312, once the building condition is identified.
pub fn mark_nominal_survivor(w: &mut World, id: ObjectId) {
    let nominal = w.obj(id).is_some_and(|o| {
        o.kind == Kind::Infantry
            && w.rules()
                .infantry
                .get(o.type_index as usize)
                .is_some_and(|u| u.raw.econ.nominal.unwrap_or(false))
    });
    if nominal && let Some(o) = w.obj_mut(id) {
        o.econ.counted.nominal_survivor = true;
    }
}

/// Whether a house is a contender (S-0414 Terms): undefeated, not passive,
/// in our engine not an observer's house; a house the unused-house cleanup
/// removed is no house of the game.
pub fn contender(w: &World, h: HouseId) -> bool {
    let Some(house) = w.houses.get(usize::from(h.0)) else {
        return false;
    };
    !house.removed
        && !house.econ.result.defeated
        && !crate::econ::setup::passive(w, h)
        // spec: S-0414 Terms (an observer's house is set aside)
        && !house.econ.setup.observer
}

// ---------------------------------------------------------------------------
// The counts (claim 4)

pub(crate) fn flags(w: &World, kind: Kind, t: u32) -> (bool, bool) {
    rules(w)
        .flags
        .get(&(kind, t))
        .copied()
        .unwrap_or((false, false))
}

/// The on-map count an object of this type is on, with whether it is added
/// at an entry and taken off at a leaving (claim 4 and its quirks).
fn slot(w: &World, id: ObjectId) -> Option<(Slot, bool, bool)> {
    let o = w.obj(id)?;
    let q = w.rules().quirks;
    let (dont_score, _insignificant) = flags(w, o.kind, o.type_index);
    // spec: S-0414 claim 4 (marked insignificant: counted on V, I, A and R)
    match o.kind {
        Kind::Vehicle => {
            if dont_score {
                // spec: S-0414 claim 4 (not scored: added but never taken off, quirks.unscored_vehicle_stays_counted; off: never counted)
                return q
                    .unscored_vehicle_stays_counted
                    .then_some((Slot::V, true, false));
            }
            Some((Slot::V, true, true))
        }
        Kind::Infantry => {
            if dont_score {
                // spec: S-0414 claim 4 (not scored: never counted)
                return None;
            }
            // spec: S-0414 claim 4 (a nominal survivor is not added at its entry, quirks.nominal_survivor_count)
            let add = !(o.econ.counted.nominal_survivor && q.nominal_survivor_count);
            Some((Slot::I, add, true))
        }
        Kind::Aircraft => (!dont_score).then_some((Slot::A, true, true)),
        Kind::Building => {
            if dont_score {
                return None;
            }
            let r = rules(w);
            let refinery = r.refinery == Some(o.type_index);
            let vehicle_building = r
                .vehicle_building
                .get(o.type_index as usize)
                .copied()
                .unwrap_or(false);
            // spec: S-0414 claim 3 and Edge cases (R: refinery-list buildings; quirks.vehicle_buildings_keep_nobody_alive off: every building that counts as a vehicle)
            let on_r = refinery || (vehicle_building && !q.vehicle_buildings_keep_nobody_alive);
            on_r.then_some((Slot::R, true, true))
        }
        _ => None,
    }
}

fn count_entry(w: &mut World, id: ObjectId, owner: HouseId) {
    let Some((s, add, _)) = slot(w, id) else {
        return;
    };
    let unscored = w.obj(id).and_then(|o| {
        (o.kind == Kind::Vehicle && flags(w, o.kind, o.type_index).0).then_some(o.type_index)
    });
    if let Some(r) = result_mut(w, owner) {
        if add {
            r.on_map[s as usize] = r.on_map[s as usize].wrapping_add(1);
        }
        if let Some(t) = unscored {
            let e = r.unscored.entry(t).or_insert(0);
            *e = e.wrapping_add(1);
        }
    }
}

fn count_leaving(w: &mut World, id: ObjectId, owner: HouseId) {
    let Some((s, _, take)) = slot(w, id) else {
        return;
    };
    if take && let Some(r) = result_mut(w, owner) {
        r.on_map[s as usize] = r.on_map[s as usize].wrapping_sub(1);
    }
}

/// An object entered the map (claim 4).
pub fn entered_map(w: &mut World, id: ObjectId) {
    if let Some(h) = w.obj(id).and_then(|o| o.owner) {
        count_entry(w, id, h);
    }
}

/// An object is about to leave the map (claim 4).
pub fn leaving_map(w: &mut World, id: ObjectId) {
    if let Some(o) = w.obj(id)
        && o.on_map
        && let Some(h) = o.owner
    {
        count_leaving(w, id, h);
    }
}

/// An object changed owner from `old` (claim 4, *change of owner*): on the
/// map, it leaves the old owner's counts and enters the new owner's; off
/// the map, no on-map count changes. *B* and the short-game count follow
/// the owner by construction.
pub fn owner_changed(w: &mut World, id: ObjectId, old: Option<HouseId>) {
    let Some(o) = w.obj(id) else { return };
    if !o.on_map {
        return;
    }
    let new = o.owner;
    if let Some(h) = old {
        count_leaving(w, id, h);
    }
    if let Some(h) = new {
        count_entry(w, id, h);
    }
}

/// *B*: the house's counted buildings in existence (Terms; claim 4).
pub(crate) fn counted_buildings(w: &World, h: HouseId) -> i32 {
    let r = rules(w);
    w.objects
        .values()
        .filter(|o| o.owner == Some(h) && o.kind == Kind::Building)
        .filter(|o| {
            let (ds, ins) = flags(w, Kind::Building, o.type_index);
            let vb = r
                .vehicle_building
                .get(o.type_index as usize)
                .copied()
                .unwrap_or(false);
            // spec: S-0414 Terms (a counted building: not counting as a vehicle, neither insignificant nor not scored)
            !vb && !ds && !ins
        })
        .count() as i32
}

/// The short-game vehicle count (claim 2).
fn short_vehicles(w: &World, h: HouseId) -> i32 {
    let r = rules(w);
    w.objects
        .values()
        .filter(|o| o.owner == Some(h) && o.kind == Kind::Vehicle)
        .filter(|o| {
            let (ds, ins) = flags(w, Kind::Vehicle, o.type_index);
            // spec: S-0414 claim 2 (a vehicle of a base-unit type, not insignificant and not scored)
            r.short_types.contains(&o.type_index) && !ds && !ins
        })
        .count() as i32
}

/// *S* of the normal test (claim 3).
pub fn normal_sum(w: &World, h: HouseId) -> i32 {
    let m = result(w, h).map_or([0; 4], |r| r.on_map); // nocite: four zero counts
    // spec: S-0414 claim 3 (S = B + V + I + A + R)
    m.iter()
        .fold(counted_buildings(w, h), |s, &x| s.wrapping_add(x))
}

/// Whether the house passes its test (claims 2 and 3).
pub fn passes(w: &World, h: HouseId) -> bool {
    if crate::econ::setup::short_game(w) {
        // spec: S-0414 claim 2 (a counted building or a vehicle of the first base-unit types)
        counted_buildings(w, h) > 0 || short_vehicles(w, h) > 0
    } else {
        // spec: S-0414 claim 3 (S ≠ 0; a negative S passes)
        normal_sum(w, h) != 0
    }
}

// ---------------------------------------------------------------------------
// The house update (claims 1, 5 and 11)

/// The victory part of a house's update: a pending surrender's
/// destruction (claim 11), then the defeat test (claim 1).
pub fn house_update(w: &mut World, h: HouseId) {
    if !w.econ.end.active {
        return;
    }
    if result(w, h).is_some_and(|r| r.doomed) {
        if let Some(r) = result_mut(w, h) {
            r.doomed = false;
        }
        // spec: S-0414 claim 11 (in the next house update every object of the house is destroyed as in claim 5)
        destroy_all(w, h);
    }
    // spec: S-0414 claim 1 (no test in tick 0)
    if w.tick < 1 {
        return;
    }
    // spec: S-0414 claim 8 (no tick runs after the final one; a caller that runs on tests nothing)
    if w.econ.end.finished {
        return;
    }
    // spec: S-0414 claim 1 (only contenders are tested; an observer's house never)
    if !contender(w, h) {
        return;
    }
    if !passes(w, h) {
        defeat(w, h);
    }
}

/// The start of the houses' part of a tick (S-0414 claim 16): in the final
/// tick the pending results are committed, before any surrender
/// destruction and defeat test of that tick.
pub fn commit(w: &mut World) {
    let e = &mut w.econ.end;
    if e.over == Some(w.tick) {
        // spec: S-0414 claims 8 and 16 (the result and expiry committed at the start of the houses' part of the final tick)
        e.committed = true;
    }
}

/// The defeat of a house (claim 5): its objects destroyed, the house marked
/// defeated, its reveal and radar, then the end test.
pub fn defeat(w: &mut World, h: HouseId) {
    destroy_all(w, h);
    let tick = w.tick;
    let q = w.rules().quirks;
    let delay = rules(w).delay;
    let pending_win = verdict(w, h) == Some(Verdict::Win);
    let person = crate::econ::setup::person_played(w, h);
    if let Some(r) = result_mut(w, h) {
        // spec: S-0414 claim 5 (the house is marked defeated, whatever happens to its objects afterwards)
        r.defeated = true;
    }
    // spec: S-0414 claim 5, S-0409 claim 15 and S-0410 claim 1 (the whole map explored and radar on, at once)
    crate::econ::radar::on_defeat(w, h);
    if ended(w) {
        // spec: S-0414 claim 16 (a defeat tested in the final tick changes neither the committed result nor the expiry, whatever quirks.defeat_during_end_delay)
        if w.econ.end.committed {
            return;
        }
        if q.defeat_during_end_delay && pending_win {
            // spec: S-0414 claim 9 (a player house defeated during the end delay loses, quirks.defeat_during_end_delay)
            if let Some(r) = result_mut(w, h) {
                r.verdict = Some(Verdict::Loss);
            }
            if person {
                // spec: S-0414 claim 9 (the delay restarts for a person-played house with a win pending)
                w.econ.end.over = Some(tick.wrapping_add(delay.max(1)));
            }
        }
        return;
    }
    // spec: S-0414 claims 5 and 6 (the end test of the same defeat)
    if no_hostile_pair(w) {
        end(w);
    }
}

/// The end test (claim 6, Formulas *K*): no two contenders of which one
/// does not ally the other.
pub fn no_hostile_pair(w: &World) -> bool {
    let k: Vec<HouseId> = (0..w.houses.len())
        .map(|i| HouseId(i as u8))
        .filter(|&h| contender(w, h))
        .collect();
    // spec: S-0414 Formulas (a(x, y) for every ordered pair of different members of K)
    k.iter()
        .all(|&x| k.iter().all(|&y| x == y || w.allied(Some(x), Some(y))))
}

/// The end of the game (claims 6 to 8): results fixed, the end delay
/// started.
fn end(w: &mut World) {
    let tick = w.tick;
    let delay = rules(w).delay;
    w.econ.end.ended = Some(tick);
    // spec: S-0414 Formulas (over in tick t + max(D, 1))
    w.econ.end.over = Some(tick.wrapping_add(delay.max(1)));
    for h in &mut w.houses {
        // spec: S-0414 claim 7 (every player house gets a result; no neutral, special or observer's house)
        if h.role != HouseRole::Player || h.removed || h.econ.setup.observer {
            continue;
        }
        h.econ.result.verdict = Some(if h.econ.result.defeated {
            Verdict::Loss
        } else {
            Verdict::Win
        });
    }
}

/// Whether an object is one of the four kinds a destruction sweep counts
/// (S-0414 claim 15).
fn swept_kind(k: Kind) -> bool {
    matches!(
        k,
        Kind::Infantry | Kind::Vehicle | Kind::Aircraft | Kind::Building
    )
}

/// Destroys every infantry unit, vehicle, aircraft and building of the
/// house (claims 5 and 15): one hit each of its current health with the
/// demolition warhead, ignoring defences, with the no-escape marker and no
/// attacker, in creation order across the four kinds, on the map or off it
/// (production items included). Each object's eligibility is judged at its
/// place in that order, so an object an earlier death removed is not hit
/// and one an earlier death created is. Under
/// `quirks.defeat_cancellation_skip` a hit whose effects delete k objects
/// at once (production cancellations) at or before its place skips the
/// next max(k − 1, 0) objects of the four kinds, whoever owns them.
/// Controlled objects are judged by their original house (S-0601 claim
/// 14, claim 5's table); an erased victim's beams are detached first
/// (S-0514 claim 15).
fn destroy_all(w: &mut World, h: HouseId) {
    let warhead = w.rules().combat.general.c4_warhead;
    let skip_quirk = w.rules().quirks.defeat_cancellation_skip;
    let mut place: Option<ObjectId> = None;
    let mut skip: usize = 0;
    loop {
        let from = place.map_or(ObjectId(0), |p| ObjectId(p.0 + 1));
        // spec: S-0414 claim 15 (creation order across all four kinds; object numbers are given in creation order)
        let next = w
            .objects
            .range(from..)
            .map(|(_, o)| o)
            .find(|o| swept_kind(o.kind))
            .map(|o| (o.id, o.owner, o.health));
        // spec: S-0203 claim 13 (a) and (c) (the sweep finds the tick's leftovers of the four kinds at their places in creation order, those of objects removed earlier in the sweep itself included)
        let left = crate::leftover::next(w, from);
        if let Some(l) = left
            && next.is_none_or(|(id, _, _)| l < id)
        {
            place = Some(l);
            if skip > 0 {
                // spec: S-0414 claim 15 (a skipped place, whoever owns the object) and S-0203 claim 13 (c) (a leftover has its place in creation order); Q-2693 (interim: every leftover of the four kinds holds its place, in both positions of quirks.defeat_sweep_leftovers)
                skip -= 1;
                continue;
            }
            w.econ.end.deleted = Some(Vec::new());
            crate::leftover::sweep_hit(w, l, h);
            let k = w
                .econ
                .end
                .deleted
                .take()
                .unwrap_or_default()
                .into_iter()
                .filter(|&d| d <= l)
                .count();
            if skip_quirk {
                // spec: S-0414 claim 15 (quirks.defeat_cancellation_skip: max(k − 1, 0) following places skipped)
                skip = k.saturating_sub(1);
            }
            continue;
        }
        let Some((id, owner, health)) = next else {
            break;
        };
        place = Some(id);
        if skip > 0 {
            // spec: S-0414 claim 15 (a skipped place, whoever owns the object)
            skip -= 1;
            continue;
        }
        // spec: S-0414 claim 15 (eligibility at the object's place; an already dead object gets no second death)
        if health <= 0 {
            continue;
        }
        // spec: S-0414 claim 5 table and S-0601 claim 14 (objects under mind control are treated by their original house: one of another house held by this house is not hit; a permanently controlled one by its current owner)
        if crate::mind::original_owner(w, id) != Some(h) {
            continue;
        }
        if owner != Some(h) && crate::mind::controlled_by(w, id).is_some() {
            // spec: S-0414 claim 5 table (of this house, held by another house through a controlling unit: with a civilian-side house in the game, not hit and the first such house takes this house's place as its original house; otherwise hit)
            let civ = (0..w.houses.len())
                .map(|i| HouseId(i as u8))
                .find(|&x| crate::econ::setup::civilian(w, x));
            if let Some(c) = civ {
                crate::mind::redirect_original_owner(w, id, Some(c));
                continue;
            }
        }
        // spec: S-0514 claim 15 (a destroy-all detaches the beams on each erased victim and idles their attackers before the destructive hit)
        crate::temporal::detach_for_destruction(w, id);
        w.econ.end.deleted = Some(Vec::new());
        // spec: S-0414 claim 5 (amount: current health; the demolition warhead; ignores defences; no-escape marker; no attacker, no attacking house)
        crate::combat::damage::apply(
            w,
            id,
            crate::combat::damage::Hit {
                amount: health,
                warhead,
                distance: 0,
                attacker: None,
                house: None,
                ignore_defences: true,
                no_escape: true,
            },
        );
        let k = w
            .econ
            .end
            .deleted
            .take()
            .unwrap_or_default()
            .into_iter()
            .filter(|&d| d <= id)
            .count();
        if skip_quirk {
            // spec: S-0414 claim 15 (quirks.defeat_cancellation_skip: max(k − 1, 0) following places skipped)
            skip = k.saturating_sub(1);
        }
    }
}

/// Records a production object deleted at once (S-0404 claim 6), for the
/// cancellation exception of S-0414 claim 15.
pub fn object_deleted(w: &mut World, id: ObjectId) {
    if let Some(d) = w.econ.end.deleted.as_mut() {
        d.push(id);
    }
}

// ---------------------------------------------------------------------------
// Orders (claims 11 to 14)

/// Applies a surrender order (S-0414 claim 11): the house is marked to die
/// when it is undefeated and has no result pending; an observer's house's
/// order does nothing.
pub fn surrender(w: &mut World, house: HouseId) {
    if !w.econ.end.active
        || ended(w)
        || defeated(w, house)
        || crate::econ::setup::observer(w, house)
    {
        // spec: S-0414 claim 11 and Edge cases (a defeated house, a pending result or an observer's house: nothing)
        return;
    }
    if let Some(r) = result_mut(w, house) {
        r.doomed = true;
    }
}

/// Applies an alliance order from `house` naming `other` (S-0414 claims 12
/// to 14).
pub fn alliance(w: &mut World, house: HouseId, other: HouseId) {
    let r = rules(w);
    // spec: S-0414 claim 14 (both flags on, checked when the order is applied)
    if !(r.allies_allowed && r.ally_change_allowed) {
        return;
    }
    // spec: S-0414 Edge cases (an observer's house's alliance orders do nothing)
    if crate::econ::setup::observer(w, house) || usize::from(other.0) >= w.houses.len() {
        return;
    }
    if w.allied(Some(house), Some(other)) {
        // spec: S-0414 claim 13 (naming itself counts as allied and changes nothing; a passive A or B: nothing)
        if house == other
            || crate::econ::setup::passive(w, house)
            || crate::econ::setup::passive(w, other)
        {
            return;
        }
        // spec: S-0414 claim 13 (the alliance ends in both directions, also for a defeated issuer)
        for (a, b) in [(house, other), (other, house)] {
            let was = w.allied(Some(a), Some(b));
            w.houses[usize::from(a.0)].allies.retain(|&x| x != b);
            // spec: S-0309 claim 16 (a house made an enemy of another, here one whose alliance ended: its anger towards it grows by 1)
            if was {
                crate::combat::acquire::alliance_anger(w, a, b, false);
                // spec: S-0212 claim 16a (a house that breaks an alliance has its records rebuilt; the order ends it in both directions, so each house whose alliance ended)
                crate::threat::rebuild(w, a);
            }
        }
    } else {
        // spec: S-0414 claim 12 (B is not A and not of the civilian side; A is undefeated)
        if other == house || crate::econ::setup::civilian(w, other) || defeated(w, house) {
            return;
        }
        let enemies = (0..w.houses.len())
            .map(|i| HouseId(i as u8))
            .filter(|&c| c != house && contender(w, c) && !w.allied(Some(house), Some(c)))
            .count();
        // spec: S-0414 claim 12 (refused when exactly one contender other than A is not allied by A)
        if enemies == 1 {
            return;
        }
        let allies = &mut w.houses[usize::from(house.0)].allies;
        if let Err(p) = allies.binary_search(&other) {
            allies.insert(p, other);
        }
        // spec: S-0309 claim 16 (made an ally: the anger towards it becomes 0, the enemy house worked out again)
        crate::combat::acquire::alliance_anger(w, house, other, true);
        drop_allied_targets(w, house);
        // spec: S-0212 claim 16a (a house that makes an alliance has its records rebuilt)
        crate::threat::rebuild(w, house);
    }
    // spec: S-0409 claim 2 and Edge cases (the sights follow the new alliances)
    crate::econ::shroud::alliance_changed(w);
}

/// Claim 12: A's objects on the map whose current target belongs to a
/// house A allies drop that target.
fn drop_allied_targets(w: &mut World, house: HouseId) {
    let ids: Vec<ObjectId> = w
        .objects
        .values()
        .filter(|o| o.owner == Some(house) && o.on_map)
        .map(|o| o.id)
        .collect();
    for id in ids {
        let Some(Target::Object(t)) = w.obj(id).and_then(|o| o.target) else {
            continue;
        };
        let owner = w.obj(t).and_then(|o| o.owner);
        if w.allied(Some(house), owner)
            && let Some(o) = w.obj_mut(id)
        {
            // spec: S-0414 claim 12 (the target of a house A now allies is dropped)
            crate::spawn::clear_own_target(o);
        }
    }
}
