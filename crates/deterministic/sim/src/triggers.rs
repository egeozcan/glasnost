//! Map triggers in a skirmish (S-0011): tags and their trigger instances,
//! the per-tick poll (S-0203 row 1), the owner-house raise every 8 ticks,
//! trigger timers with their draws, latches, repeat types, map trigger
//! variables, the mission timer and the actions of the subset this engine
//! covers. The map's definitions arrive resolved by the importer
//! ([`glasnost_map::triggers`]) and are kept in the state, so a saved game
//! carries them.
//!
//! Covered: the condition events (any event, elapsed time, random delay,
//! game time, variables, type exists / does not exist), the code-matched
//! evaluation and latches (for the raises other parts of the engine make),
//! enable and disable, set and clear variable, the mission timer actions,
//! reveal around a waypoint, and the presentation-only actions as
//! [`Effect::Trigger`]. The actions that hand over to specs or parts of the
//! engine that do not exist yet are recognised and leave a note (seams); so
//! does the bridge-destroyed raise. The carrier raises of a unit's arrival
//! ([`cell_entered`], S-0207 and S-0208 claim 22) and of a killing hit
//! ([`destruction_raises`], S-0312 claim 29) are entry points that the
//! movement and combat code call at the moments their specs give.

use std::collections::{BTreeMap, BTreeSet};

use glasnost_map::CellXY;
use glasnost_map::triggers::{Owner, Triggers, VARIABLES};
use serde::{Deserialize, Serialize};

use crate::combat::effects::Effect;
use crate::rules::Kind;
use crate::world::{HouseId, HouseRole, ObjectId, Pos, World};

/// Ticks in one trigger time unit (S-0011 Terms).
const UNIT: i32 = 15; // spec: S-0011 Constants (trigger time unit: 15 ticks)
/// Owner-house raise interval (S-0011 claim 35).
const HOUSE_RAISE: i32 = 8; // spec: S-0011 Constants (owner-house raise interval: 8 ticks)
/// Latch positions per trigger instance (S-0011 claim 9).
const LATCHES: usize = 32; // spec: S-0011 Constants (latch positions per trigger instance: 32)
/// Largest waypoint number of the reveal action (S-0011 claim 33).
const LAST_WAYPOINT: i32 = 701; // spec: S-0011 claim 33 (a reveal waypoint outside 0 … 701 is unspecified)
/// The high-bridge lift in height levels (S-0011 Constants).
const BRIDGE_LIFT: i32 = 4; // spec: S-0011 Constants (high-bridge lift: 4 height levels)
/// Leptons per height level (glossary).
const LEVEL: i32 = 104; // spec: S-0011 Constants (height level: 104 leptons)

// Event codes (S-0011 I3).
const ENTERED_BY: i32 = 1; // spec: S-0011 I3 (1: entered-by)
const ATTACKED: i32 = 6; // spec: S-0011 I3 (6: attacked, tagged object)
const ATTACKED_BY_HOUSE: i32 = 44; // spec: S-0011 I3 (44: attacked by house)
const CRATE_PICKED_UP: i32 = 49; // spec: S-0520 claim 7 (event 49: a crate is picked up)
const ANY_EVENT: i32 = 8; // spec: S-0011 I3 (8: any event)
const ELAPSED: i32 = 13; // spec: S-0011 I3 (13: elapsed time)
const VAR_SET: i32 = 36; // spec: S-0011 I3 (36: variable set)
const VAR_CLEAR: i32 = 37; // spec: S-0011 I3 (37: variable clear)
const GAME_TIME: i32 = 47; // spec: S-0011 I3 (47: game time)
const RANDOM_DELAY: i32 = 51; // spec: S-0011 I3 (51: random delay)
const TYPE_EXISTS: i32 = 60; // spec: S-0011 I3 (60: type exists)
const TYPE_ABSENT: i32 = 61; // spec: S-0011 I3 (61: type does not exist)
const DESTROYED_BY: i32 = 48; // spec: S-0011 I3 (48: destroyed by anything)
const BRIDGE_DESTROYED: i32 = 31; // spec: S-0011 I3 (31: bridge destroyed in tagged cell)

/// The polled set (S-0011 Constants).
const POLLED: [i32; 15] = [8, 13, 14, 23, 27, 28, 36, 37, 45, 46, 47, 50, 51, 60, 61]; // spec: S-0011 Constants (polled set)

/// Whether a code is in the code-matched set (S-0011 Constants).
fn code_matched(c: i32) -> bool {
    // spec: S-0011 Constants (code-matched set: 1, 2, 3, 4, 6, 7, 18 … 26, 29, 31, 33, 34, 35, 38 … 44, 48, 49, 50, 53, 54, 59)
    matches!(c, 1..=4 | 6 | 7 | 18..=26 | 29 | 31 | 33..=35 | 38..=44 | 48..=50 | 53 | 54 | 59)
}

/// Whether a code is latchable (S-0011 Constants).
fn latchable(c: i32) -> bool {
    // spec: S-0011 Constants (latchable set: the code-matched set without 1, 6, 31, 44 and 59)
    code_matched(c) && !matches!(c, 1 | 6 | 31 | 44 | 59)
}

/// The rules of the triggers.
#[derive(Clone, Debug, Default)]
pub struct Rules {
    /// The initial session computer difficulty (S-0011 claim 4).
    pub difficulty: i32,
    /// *R* of the reveal action, cells (S-0011 claim 26).
    pub reveal_radius: i32,
}

impl Rules {
    /// The trigger rules of a resolved ruleset.
    pub fn new(r: &glasnost_rules::schema::Ruleset) -> Self {
        let g = &r.sim.general.economy;
        Self {
            // spec: S-0011 Import mapping (AIDifficulty absent: 1)
            difficulty: g.ai_difficulty.unwrap_or(1),
            // spec: S-0011 Import mapping (RevealTriggerRadius absent: 5)
            reveal_radius: g.reveal_trigger_radius.unwrap_or(5),
        }
    }
}

/// One trigger instance (S-0011 Terms).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Instance {
    /// The trigger definition.
    pub trigger: u32,
    /// Enabled flag.
    pub enabled: bool,
    /// Removed flag.
    pub removed: bool,
    /// Timer arming tick *s*.
    pub start: i32,
    /// Timer length *D*, ticks.
    pub length: i32,
    /// Latched event positions (1-based, after aliasing).
    pub latches: BTreeSet<u32>,
    /// Triggering house (claim 18).
    pub triggering: Option<HouseId>,
}

/// One tag (S-0011 Terms).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tag {
    /// Carrier count.
    pub carriers: i32,
    /// Instances in chain order.
    pub instances: Vec<Instance>,
    /// Polled (decided at the end of loading, claim 6).
    pub polled: bool,
    /// A one-time tag that fired: deleted at the end of the tick (claim 11).
    pub dying: bool,
}

/// The mission timer (S-0011 claim 31).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MissionTimer {
    /// Ever started (shown).
    pub shown: bool,
    /// Running.
    pub running: bool,
    /// Tick at which `length` was set.
    pub since: i32,
    /// Remaining length at `since`, ticks.
    pub length: i32,
    /// The label; `None` is the default label.
    pub label: Option<Vec<u8>>,
}

impl MissionTimer {
    /// Remaining ticks at tick `t`.
    pub fn remaining(&self, t: i32) -> i32 {
        if self.running {
            // spec: S-0011 claim 31 (a running mission timer reaching 0 stops)
            self.length.wrapping_sub(t.wrapping_sub(self.since)).max(0)
        } else {
            self.length
        }
    }
}

/// The trigger state of a game.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TriggerState {
    /// The map's definitions.
    pub map: Triggers,
    /// Live tags by tag definition index (registration order).
    pub tags: BTreeMap<u32, Tag>,
    /// The cells that carry a tag (S-0009 claim 31).
    pub cells: BTreeMap<(i16, i16), u32>,
    /// The objects that carry a tag (S-0011 claim 2).
    pub objects: BTreeMap<ObjectId, u32>,
    /// Tags assigned to each house at the end of loading (claim 35), in
    /// registration order.
    pub house_tags: Vec<Vec<u32>>,
    /// Map trigger variables 0 … 99 (claim 22).
    pub variables: Vec<bool>,
    /// The mission timer.
    pub mission: MissionTimer,
    /// The session difficulty position (claim 4).
    pub difficulty: i32,
    /// The map's waypoint markers (S-0009 claim 15), for actions.
    pub waypoints: BTreeMap<u16, (i16, i16)>,
    /// Triggers fired so far (a statistic for reports, not behaviour).
    pub fired: u64,
    /// The multiplayer completion's state (S-0524).
    pub mp: crate::mp_triggers::MpState,
    /// The tags being raised (claim 12); empty between raises.
    #[serde(skip)]
    pub raising: BTreeSet<u32>,
}

/// A raise (S-0011 Terms): its code, and optionally the causing object.
#[derive(Clone, Copy, Debug)]
pub struct Raise {
    /// The raise's code; `None` for a poll, in which no code-matched event
    /// is judged true (claim 6).
    pub code: Option<i32>,
    /// The causing object.
    pub causer: Option<ObjectId>,
    /// The raising carrier.
    pub carrier: Option<Carrier>,
}

/// The carrier that raised a tag.
#[derive(Clone, Copy, Debug)]
pub enum Carrier {
    /// A cell.
    Cell((i16, i16)),
    /// An object.
    Object(ObjectId),
}

/// The owner country of a trigger definition (S-0011 claim 5), as a
/// country registration index.
fn owner_country(w: &World, t: u32) -> Option<u32> {
    let def = w.triggers.map.triggers.get(t as usize)?;
    let r = w.rules();
    match &def.owner {
        Owner::None => None,
        // spec: S-0011 claim 5 (`<none>`: the first country of the country list, quirks.none_owner_is_first_country)
        Owner::NoneMarker => {
            (r.quirks.none_owner_is_first_country && !r.country_names.is_empty()).then_some(0)
        }
        // spec: S-0011 claim 5 (the country whose type name equals the text without regard to letter case)
        Owner::Name(n) => r
            .country_names
            .iter()
            .position(|c| c.as_bytes().eq_ignore_ascii_case(n))
            .map(|i| i as u32),
    }
}

/// The first house in house order that plays country `c` (S-0011 Terms,
/// *Owner house*), among the houses that exist.
pub(crate) fn house_of_country(w: &World, c: u32) -> Option<HouseId> {
    w.houses
        .iter()
        .position(|h| !h.removed && h.country == c)
        .map(|i| HouseId(i as u8))
}

/// Sets up the trigger state as a map's loading begins: the definitions,
/// the waypoints, the variables and the difficulty (S-0011 claim 4; S-0009
/// claim 32: every variable state cleared at every game start, then the
/// map's initial states).
pub fn begin(w: &mut World, map: &glasnost_map::Map) {
    let difficulty = w.rules().triggers.difficulty;
    let t = &mut w.triggers;
    t.map = map.triggers.clone();
    t.waypoints = map.waypoints.clone();
    t.difficulty = difficulty;
    // spec: S-0009 claim 32 (our engine clears every variable state at every game start, then the map's initial states apply)
    t.variables = (0..VARIABLES)
        .map(|i| {
            map.triggers
                .variables
                .get(i)
                .copied()
                .flatten()
                .unwrap_or(false)
        })
        .collect();
    // spec: S-0011 claim 4 (difficulty positions 0 … 2)
    if !(0..=2).contains(&difficulty) {
        // spec: S-0011 claim 4 (a setting outside 0 … 2 selects no difficulty flag; the importer warns)
        w.note("S-0011 claim 4: the trigger difficulty lies outside 0 … 2; no difficulty flag gates the triggers");
    }
}

/// Whether trigger `t` passes the difficulty gate (S-0011 claims 3, 4, 25).
fn difficulty_ok(w: &World, t: u32) -> bool {
    let d = w.triggers.difficulty;
    let def = &w.triggers.map.triggers[t as usize];
    // spec: S-0011 claim 4 (a setting outside 0 … 2 selects no difficulty flag)
    usize::try_from(d)
        .ok()
        .and_then(|i| def.difficulty.get(i).copied())
        .unwrap_or(true)
}

/// Creates the tag of definition `tag` if it does not exist yet (S-0011
/// claim 34), arming its instances in chain order (claims 13, 14).
fn create(w: &mut World, tag: u32) {
    if w.triggers.tags.contains_key(&tag) {
        return;
    }
    let chain = w.triggers.map.tags[tag as usize].chain.clone();
    let mut instances = Vec::with_capacity(chain.len());
    for &t in &chain {
        let def = &w.triggers.map.triggers[t as usize];
        // spec: S-0011 claim 3 (enabled at start: the disabled flag clear and the difficulty flag set)
        let enabled = !def.disabled && difficulty_ok(w, t);
        let mut i = Instance {
            trigger: t,
            enabled,
            removed: false,
            start: 0,
            length: 0,
            latches: BTreeSet::new(),
            triggering: None,
        };
        arm(w, &mut i);
        instances.push(i);
    }
    w.triggers.tags.insert(
        tag,
        Tag {
            carriers: 0,
            instances,
            polled: false,
            dying: false,
        },
    );
}

/// The latch position of event position `p` (1-based), S-0011 claim 9.
fn latch_pos(w: &World, p: usize) -> u32 {
    if w.rules().quirks.event_latch_aliasing {
        // spec: S-0011 claim 9 (positions from 33 on share the latch of the position 32 lower, quirks.event_latch_aliasing)
        ((p - 1) % LATCHES + 1) as u32
    } else {
        p as u32
    }
}

/// Arms an instance's trigger timer at the current tick (S-0011 claims 13
/// to 15, Formulas F1 and F2), taking the random-delay draws in reverse
/// event order, and clears the latches at the timer events' positions
/// (claim 9).
fn arm(w: &mut World, i: &mut Instance) {
    let events = w.triggers.map.triggers[i.trigger as usize].events.clone();
    let half = w.rules().quirks.ranged_draw_half_span;
    let mut draws = vec![0i32; events.len()];
    // spec: S-0011 claim 14 (one ranged draw per random-delay event, none when v = 0, in reverse order of the event list)
    for (p, e) in events.iter().enumerate().rev() {
        if e.code == RANDOM_DELAY && e.value != 0 {
            // spec: S-0011 F2 (r: a ranged draw over the inclusive range between 0 and v)
            draws[p] = w.rng.ranged(0, e.value, half);
        }
    }
    // spec: S-0011 claim 13 (the timer event written first sets the timer)
    if let Some((p, e)) = events
        .iter()
        .enumerate()
        .find(|(_, e)| e.code == ELAPSED || e.code == RANDOM_DELAY)
    {
        i.length = if e.code == ELAPSED {
            // spec: S-0011 F1 (D = 15 × v, wrapped to signed 32 bits)
            UNIT.wrapping_mul(e.value)
        } else {
            // spec: S-0011 F2 (D = 15 × (v quo 2 + r), each step wrapped)
            UNIT.wrapping_mul((e.value / 2).wrapping_add(draws[p]))
        };
    }
    i.start = w.tick;
    // spec: S-0011 claim 9 (arming a timer clears the latch at each timer event's position)
    for (p, e) in events.iter().enumerate() {
        if e.code == ELAPSED || e.code == RANDOM_DELAY {
            let l = latch_pos(w, p + 1);
            i.latches.remove(&l);
        }
    }
}

/// Attaches the map's cell tags as loading reads them (S-0009 claim 31;
/// S-0011 claim 34): each creates its tag if needed, with the draws.
pub fn cell_tags(w: &mut World) {
    for (c, tag) in w.triggers.map.cell_tags.clone() {
        create(w, tag);
        w.triggers.cells.insert(c, tag);
        if let Some(t) = w.triggers.tags.get_mut(&tag) {
            t.carriers += 1;
        }
    }
}

/// Attaches the tag a pre-placed object's tag field names (S-0011 claims 2
/// and 34), right after the object's creation draw.
pub fn object_tag(w: &mut World, id: ObjectId, field: Option<&[u8]>) {
    let Some(field) = field else { return };
    let Some(tag) = w.triggers.map.object_tag(field) else {
        if !field.eq_ignore_ascii_case(b"none") {
            // spec: S-0011 claim 2 (a tag field that matches no tag definition gives no tag; the importer warns)
            w.note("S-0011 claim 2: a pre-placed object's tag field matches no tag definition");
        }
        return;
    };
    create(w, tag);
    w.triggers.objects.insert(id, tag);
    if let Some(t) = w.triggers.tags.get_mut(&tag) {
        t.carriers += 1;
    }
}

/// Detaches an object's tag at its removal, with
/// `quirks.removed_carrier_keeps_tag` off (S-0011 Edge cases): the tag's
/// carrier count drops by one. With the switch on, nothing detaches it.
pub fn detach_object(w: &mut World, id: ObjectId) {
    if let Some(tag) = w.triggers.objects.remove(&id)
        && let Some(t) = w.triggers.tags.get_mut(&tag)
    {
        t.carriers -= 1;
    }
}

/// The end of loading (S-0011 Terms, claims 6, 34 and 35): the remaining
/// polled tags are created in registration order, every tag's polling is
/// decided, and the house-scope tags are assigned to their owner houses.
pub fn end_of_loading(w: &mut World) {
    let n = w.triggers.map.tags.len() as u32;
    let has = |w: &World, tag: u32, f: &dyn Fn(i32) -> bool| {
        w.triggers.map.tags[tag as usize].chain.iter().any(|&t| {
            w.triggers.map.triggers[t as usize]
                .events
                .iter()
                .any(|e| f(e.code))
        })
    };
    // spec: S-0011 claim 34 (every remaining polled tag definition gets its tag, in registration order)
    for tag in 0..n {
        if has(w, tag, &|c| POLLED.contains(&c)) {
            create(w, tag);
        }
    }
    // spec: S-0011 claim 6 (whether a tag is polled is decided once, at the end of loading, for every tag that exists then)
    for tag in 0..n {
        let polled = has(w, tag, &|c| POLLED.contains(&c));
        if let Some(t) = w.triggers.tags.get_mut(&tag) {
            t.polled = polled;
        }
    }
    // spec: S-0011 claim 35 (a tag whose chain holds an any-event is assigned to the owner house of its chain's first trigger)
    let mut house_tags = vec![Vec::new(); w.houses.len()];
    for tag in 0..n {
        if !w.triggers.tags.contains_key(&tag) || !has(w, tag, &|c| c == ANY_EVENT) {
            continue;
        }
        let Some(&first) = w.triggers.map.tags[tag as usize].chain.first() else {
            continue;
        };
        if let Some(h) = owner_country(w, first).and_then(|c| house_of_country(w, c)) {
            house_tags[usize::from(h.0)].push(tag);
        }
    }
    w.triggers.house_tags = house_tags;
}

/// Row 1 of the tick (S-0203; S-0011 claim 6): every polled tag receives
/// one poll, in registration order.
pub fn poll(w: &mut World) {
    let polled: Vec<u32> = w
        .triggers
        .tags
        .iter()
        .filter(|(_, t)| t.polled)
        .map(|(&i, _)| i)
        .collect();
    for tag in polled {
        raise(
            w,
            tag,
            Raise {
                code: None,
                causer: None,
                carrier: None,
            },
        );
    }
}

/// A house's raise of its assigned tags in every eighth tick (S-0011 claim
/// 35), in reverse registration order, with the any-event code.
pub fn house_raise(w: &mut World, h: HouseId) {
    // spec: S-0011 claim 35 (in every tick whose number is a multiple of 8, tick 0 included)
    if w.tick.rem_euclid(HOUSE_RAISE) != 0 {
        return;
    }
    let tags = w
        .triggers
        .house_tags
        .get(usize::from(h.0))
        .cloned()
        .unwrap_or_default();
    // spec: S-0011 claim 35 (for tags assigned to one house, the effects follow reverse registration order)
    for tag in tags.into_iter().rev() {
        raise(
            w,
            tag,
            Raise {
                code: Some(ANY_EVENT),
                causer: None,
                carrier: None,
            },
        );
    }
}

/// The end of the tick: one-time tags that fired are deleted (S-0011 claim
/// 11), with their carriers.
pub fn end_tick(w: &mut World) {
    let dying: Vec<u32> = w
        .triggers
        .tags
        .iter()
        .filter(|(_, t)| t.dying)
        .map(|(&i, _)| i)
        .collect();
    for tag in dying {
        w.triggers.tags.remove(&tag);
        w.triggers.cells.retain(|_, t| *t != tag);
        w.triggers.objects.retain(|_, t| *t != tag);
        for list in &mut w.triggers.house_tags {
            list.retain(|&t| t != tag);
        }
    }
}

/// Raises tag `tag` (S-0011 claims 8 to 12).
pub fn raise(w: &mut World, tag: u32, r: Raise) {
    if !w.triggers.tags.contains_key(&tag) {
        return;
    }
    // spec: S-0011 claim 12 (a raise of a tag while that same tag is being raised has no effect)
    if !w.triggers.raising.insert(tag) {
        return;
    }
    let n = w.triggers.tags[&tag].instances.len();
    let repeat = w.triggers.map.tags[tag as usize].repeat;
    let mut leaves = false;
    // spec: S-0011 claim 11 (the observable effects of chain members follow reverse chain order; each member's eligibility sees the effects before its own)
    for k in (0..n).rev() {
        let Some(inst) = w.triggers.tags.get(&tag).map(|t| t.instances[k].clone()) else {
            break;
        };
        // spec: S-0011 claim 8 (each enabled, not removed instance is evaluated on its own)
        if !inst.enabled || inst.removed {
            continue;
        }
        let (satisfied, entered, mut inst) = evaluate(w, inst, repeat, &r);
        if satisfied {
            // spec: S-0011 claim 15 (satisfied, and the tag repeating or an entered-by event judged true: re-armed at the current tick, all its draws before any action of this instance; an effective self-enable action arms it again and its length replaces this one)
            if repeat == 2 || entered {
                arm(w, &mut inst);
            }
        }
        let carriers = w.triggers.tags.get(&tag).map_or(0, |t| t.carriers);
        let fire = satisfied
            && match repeat {
                // spec: S-0011 State and ordering (one-time: the trigger fires and is removed)
                0 => true,
                // spec: S-0011 claim 10 (all-carriers fires only with exactly one carrier left; with none, quirks.all_carriers_without_carrier)
                1 => {
                    carriers == 1
                        || (carriers == 0 && !w.rules().quirks.all_carriers_without_carrier)
                }
                // spec: S-0011 claim 10 (repeating fires every time and stays)
                2 => true,
                // spec: S-0011 claim 10 (any other repeat value never fires)
                _ => false,
            };
        if satisfied && (repeat == 0 || repeat == 1) {
            // spec: S-0011 State and ordering (one-time, and all-carriers: the raising carrier loses the tag after the raise)
            leaves = true;
        }
        let one_time = fire && (repeat == 0 || repeat == 1);
        if one_time {
            // spec: S-0011 claim 11 (a fired member of a one-time tag is removed at once)
            inst.removed = true;
        }
        if fire && !w.rules().quirks.persistent_event_latch {
            // spec: S-0011 Edge cases (quirks.persistent_event_latch off: the latches are cleared each time the instance fires)
            inst.latches.clear();
        }
        if let Some(t) = w.triggers.tags.get_mut(&tag) {
            t.instances[k] = inst.clone();
            if one_time {
                // spec: S-0011 State and ordering (every carrier loses the tag, and the tag is deleted at the end of the tick)
                t.dying = true;
            }
        }
        if fire {
            fire_actions(w, tag, &inst);
        }
    }
    if leaves && let Some(c) = r.carrier {
        detach(w, tag, c);
    }
    w.triggers.raising.remove(&tag);
}

/// The raise of a cell's tag when a unit arrives in the cell (S-0011
/// claim 18), at the arrival moment of S-0207 and S-0208 claim 22. The
/// caller leaves out ground passage under a high bridge, where only units
/// on the deck raise the tag (claim 18).
pub fn cell_entered(w: &mut World, unit: ObjectId, cell: (i16, i16)) {
    let Some(&tag) = w.triggers.cells.get(&cell) else {
        // spec: S-0207 Import mapping (no arrival raise without a cell tag)
        return;
    };
    // spec: S-0011 claim 18 (entered-by code, the arriving object as causing object)
    raise(
        w,
        tag,
        Raise {
            code: Some(ENTERED_BY),
            causer: Some(unit),
            carrier: Some(Carrier::Cell(cell)),
        },
    );
}

/// The bridge-destroyed raise of a tagged cell (S-0011 claim 20; S-0511
/// claim 8): the cell's tag is raised with code 31, with no object and no
/// cell.
pub fn bridge_destroyed(w: &mut World, cell: CellXY) {
    let Some(&tag) = w.triggers.cells.get(&(cell.x as i16, cell.y as i16)) else {
        return;
    };
    // spec: S-0511 claim 8 (the cell's tag is raised with the bridge-destroyed code, with no object and no cell)
    raise(
        w,
        tag,
        Raise {
            code: Some(BRIDGE_DESTROYED),
            causer: None,
            carrier: None,
        },
    );
}

/// The entered-by raise of a building that an infantry unit enters
/// (S-0011 claim 18; S-0511 claim 17): the building's tag, with the
/// entering unit as causing object.
pub fn building_entered(w: &mut World, building: ObjectId, by: ObjectId) {
    let Some(&tag) = w.triggers.objects.get(&building) else {
        return;
    };
    // spec: S-0011 claim 18 (entered-by code, the entering object as causing object)
    raise(
        w,
        tag,
        Raise {
            code: Some(ENTERED_BY),
            causer: Some(by),
            carrier: Some(Carrier::Object(building)),
        },
    );
}

/// The crate-pickup raise (S-0520 claim 7, event 49): the collector's tag,
/// with the collector as causing object.
pub fn crate_picked_up(w: &mut World, collector: ObjectId) {
    let Some(&tag) = w.triggers.objects.get(&collector) else {
        return;
    };
    // spec: S-0520 claim 7 (event 49, the collector as causing object)
    raise(
        w,
        tag,
        Raise {
            code: Some(CRATE_PICKED_UP),
            causer: Some(collector),
            carrier: Some(Carrier::Object(collector)),
        },
    );
}

/// The destroyed-by-anything raise of an object that leaves play (S-0011
/// claim 19; S-0511 claim 17), with the object as causing object.
pub fn destroyed_by_anything(w: &mut World, id: ObjectId) {
    let Some(&tag) = w.triggers.objects.get(&id) else {
        return;
    };
    // spec: S-0011 claim 19 (the destroyed-by-anything code, the object as causing object)
    raise(
        w,
        tag,
        Raise {
            code: Some(DESTROYED_BY),
            causer: Some(id),
            carrier: Some(Carrier::Object(id)),
        },
    );
}

/// The destruction raises of a killing hit on a tagged object (S-0011
/// claim 19, S-0312 claim 29). The caller makes them after health reaches
/// 0 and before the death debris, and says whether the hit has a known
/// attacker and whether the vehicle's extra group applies (a vehicle, with
/// `quirks.vehicle_destruction_extra_raises` on and no hijacker exception).
pub fn destruction_raises(w: &mut World, victim: ObjectId, attacker: bool, extra_group: bool) {
    // The raise codes in order, each with whether it needs a known attacker.
    // spec: S-0312 Import mapping (extra vehicle group: 7 with a known attacker, then 48, 29)
    const EXTRA: [(i32, bool); 3] = [(7, true), (DESTROYED_BY, false), (29, false)];
    // spec: S-0312 Import mapping (common group: 6, 4, 7 with a known attacker, then 48, 29)
    const COMMON: [(i32, bool); 5] = [
        (6, true),
        (4, true),
        (7, true),
        (DESTROYED_BY, false),
        (29, false),
    ];
    let extra: &[(i32, bool)] = if extra_group { &EXTRA } else { &[] };
    // A leftover (S-0203 claim 13 (g)) is out of the world before its first
    // raise, so no preceding action removed it.
    let present = w.obj(victim).is_some();
    for (common, group) in [(false, extra), (true, &COMMON[..])] {
        for &(code, needs_attacker) in group {
            if needs_attacker && !attacker {
                continue;
            }
            // spec: S-0312 Import mapping (each raise requires the tag still attached and, for the common group, the object not removed by a preceding action)
            if common && present && w.obj(victim).is_none() {
                return;
            }
            let Some(&tag) = w.triggers.objects.get(&victim) else {
                // spec: S-0312 claim 29 (a raise that detaches the tag prevents the later raises of that tag from this object)
                return;
            };
            // spec: S-0011 claim 19 (the dying object as causing object; the raises take no draw)
            raise(
                w,
                tag,
                Raise {
                    code: Some(code),
                    causer: Some(victim),
                    carrier: Some(Carrier::Object(victim)),
                },
            );
        }
    }
}

/// The raising carrier loses the tag (S-0011 State and ordering).
fn detach(w: &mut World, tag: u32, c: Carrier) {
    let lost = match c {
        Carrier::Cell(cell) => w.triggers.cells.remove(&cell).is_some(),
        Carrier::Object(id) => w.triggers.objects.remove(&id).is_some(),
    };
    if lost && let Some(t) = w.triggers.tags.get_mut(&tag) {
        t.carriers -= 1;
    }
}

/// Evaluates one instance in a raise (S-0011 claims 8, 9, 18): whether it
/// is satisfied, whether an entered-by event was judged true, and the
/// instance with its latches and triggering house updated.
fn evaluate(w: &World, mut inst: Instance, repeat: i32, r: &Raise) -> (bool, bool, Instance) {
    let events = &w.triggers.map.triggers[inst.trigger as usize].events;
    let mut all = true;
    let mut entered = false;
    // spec: S-0011 claim 8 (every event of the trigger is judged in every evaluation)
    for (p, e) in events.iter().enumerate() {
        let judged = judge(w, &inst, e, r);
        if judged && e.code == ENTERED_BY {
            entered = true;
            // spec: S-0011 claim 18 (the owner of the causing object becomes the triggering house)
            inst.triggering = r.causer.and_then(|c| w.obj(c)).and_then(|o| o.owner);
        }
        let l = latch_pos(w, p + 1);
        // spec: S-0011 claim 9 (a latchable event judged true in an evaluation of a repeating tag's instance is latched)
        if judged && repeat == 2 && latchable(e.code) {
            inst.latches.insert(l);
        }
        let ok = judged || inst.latches.contains(&l);
        all &= ok;
    }
    // spec: S-0011 claim 8 (a trigger with an empty event list is satisfied in every raise)
    (all, entered, inst)
}

/// Whether one event is judged true in a raise (S-0011 claims 8, 13, 16 to
/// 22).
fn judge(w: &World, inst: &Instance, e: &glasnost_map::triggers::Event, r: &Raise) -> bool {
    let t = w.tick;
    if code_matched(e.code) {
        // spec: S-0011 claim 8 (a code-matched event is judged true only in a raise with its own code)
        if r.code != Some(e.code) {
            return false;
        }
        return match e.code {
            ENTERED_BY => entered_by(w, e.value, r.causer),
            // spec: S-0011 claim 8 (event 6, attacked: judged true in any raise carrying its code)
            ATTACKED => true,
            // spec: S-0011 claim 8 (event 44, attacked by house: only with a causing object, and only when the value equals the house number of the causing object's owner at that raise)
            ATTACKED_BY_HOUSE => attacked_by_house(w, e.value, r.causer),
            // spec: S-0011 claims 19 and 20 (satisfied in every such raise)
            DESTROYED_BY | BRIDGE_DESTROYED | CRATE_PICKED_UP => true,
            // Code-matched events outside the subset are never raised by this engine.
            _ => false,
        };
    }
    match e.code {
        // spec: S-0011 claim 17 (the any-event code is satisfied in every raise)
        ANY_EVENT => true,
        // spec: S-0011 claim 13 and F3 (both timer events: the timer has run out, t − s ≥ D wrapped)
        ELAPSED | RANDOM_DELAY => t.wrapping_sub(inst.start) >= inst.length,
        // spec: S-0011 claim 16 and F4 (v ≤ t quo 15)
        GAME_TIME => t >= 0 && e.value <= t / UNIT,
        // spec: S-0011 claim 22 (level conditions; a variable outside 0 … 99 counts as clear, a departure)
        VAR_SET => var(w, e.value),
        VAR_CLEAR => !var(w, e.value),
        TYPE_EXISTS | TYPE_ABSENT => type_event(w, e),
        // Condition events outside the subset (deferred): never judged true here.
        _ => false,
    }
}

/// The attacked-by-house condition (S-0011 claim 8): the raw value is the
/// house number of the causing object's owner (S-0009 Terms: the place in
/// the creation order, from 0, which is the house's index here). −1 and
/// the player-position values equal no house number, so they never match.
fn attacked_by_house(w: &World, v: i32, causer: Option<ObjectId>) -> bool {
    let Some(owner) = causer.and_then(|c| w.obj(c)).and_then(|o| o.owner) else {
        return false;
    };
    // spec: S-0011 claim 8 and Constants (event 44's value: a house number counted from 0; neither a country index nor a player-position value)
    v == i32::from(owner.0)
}

/// The entered-by condition (S-0011 claim 18).
fn entered_by(w: &World, v: i32, causer: Option<ObjectId>) -> bool {
    let Some(owner) = causer.and_then(|c| w.obj(c)).and_then(|o| o.owner) else {
        return false;
    };
    // spec: S-0011 claim 18 (−1: any house)
    if v == -1 {
        return true;
    }
    // spec: S-0011 claim 18 (a country index played by a house, the causing object's owner being the first such house; quirks.entered_by_country_only)
    u32::try_from(v)
        .ok()
        .filter(|&c| (c as usize) < w.rules().country_names.len())
        .and_then(|c| house_of_country(w, c))
        == Some(owner)
}

fn var(w: &World, v: i32) -> bool {
    usize::try_from(v)
        .ok()
        .and_then(|i| w.triggers.variables.get(i).copied())
        .unwrap_or(false)
}

/// The kinds a type event's name is looked up in, in turn (S-0011 claim 21).
const TYPE_EVENT_KINDS: [Kind; 4] = [
    Kind::Building,
    Kind::Vehicle,
    Kind::Infantry,
    Kind::Aircraft,
];

/// The type a type event's name names (S-0011 claim 21), if any.
fn named_type(r: &crate::rules::SimRules, name: &[u8], exact: bool) -> Option<(Kind, u32)> {
    // The four kinds together; the last match wins (the order is not established, claim 21; ours: buildings, vehicles, infantry, aircraft).
    if exact {
        // spec: S-0011 claim 21 (the stored name equals the event's exactly, letter case included, quirks.exact_trigger_type_names)
        let mut key = (Kind::Building, std::str::from_utf8(name).ok()?.to_owned());
        TYPE_EVENT_KINDS
            .into_iter()
            .filter_map(|kind| {
                key.0 = kind;
                r.names.get(&key).map(|&i| (kind, i))
            })
            .last()
    } else {
        // Of several names of one kind that match with letters folded, the last in name order (#400).
        let folded = name.to_ascii_lowercase();
        TYPE_EVENT_KINDS
            .into_iter()
            .filter_map(|kind| r.index_of_folded(kind, &folded).map(|i| (kind, i)))
            .last()
    }
}

/// The type events (S-0011 claim 21).
fn type_event(w: &World, e: &glasnost_map::triggers::Event) -> bool {
    let Some(name) = e.type_name.as_deref() else {
        return false;
    };
    let r = w.rules();
    // spec: S-0011 claim 21 (no type of that name: neither event is ever judged true)
    let Some((kind, ti)) = named_type(r, name, r.quirks.exact_trigger_type_names) else {
        return false;
    };
    let real = |k: Kind| {
        matches!(
            k,
            Kind::Building | Kind::Vehicle | Kind::Infantry | Kind::Aircraft
        )
    };
    if e.code == TYPE_EXISTS {
        if e.value <= 0 {
            // spec: S-0011 claim 21 (n ≤ 0: satisfied as soon as any building, vehicle, infantry or aircraft object exists)
            return w.objects.values().any(|o| real(o.kind));
        }
        // spec: S-0011 claim 21 (at least n objects of the type, of every house, including those inside transports or buildings)
        let count = w
            .objects
            .values()
            .filter(|o| o.kind == kind && o.type_index == ti)
            .count();
        count >= e.value as usize
    } else {
        // spec: S-0011 claim 21 (no object of the found type exists)
        !w.objects
            .values()
            .any(|o| o.kind == kind && o.type_index == ti)
    }
}

/// The cell of a waypoint parameter (S-0011 claim 33), or none.
pub(crate) fn waypoint_cell(w: &mut World, wp: i32) -> Option<CellXY> {
    if wp < 0 {
        // spec: S-0011 claim 33 (a waypoint parameter that decodes to −1: no effect, a departure)
        w.note("S-0011 claim 33: an action's waypoint parameter names no letter; no effect");
        return None;
    }
    let c = u16::try_from(wp)
        .ok()
        .and_then(|n| w.triggers.waypoints.get(&n).copied());
    Some(match c {
        Some((x, y)) => CellXY::new(i32::from(x), i32::from(y)),
        None => {
            // spec: S-0011 claim 33 (an undefined marker stands for cell (0, 0))
            w.note(
                "S-0011 claim 33: an action names an undefined waypoint; it stands for cell (0, 0)",
            );
            CellXY::new(0, 0)
        }
    })
}

/// Fires an instance's actions in list order (S-0011 claim 11).
fn fire_actions(w: &mut World, tag: u32, inst: &Instance) {
    w.triggers.fired += 1;
    let actions = w.triggers.map.triggers[inst.trigger as usize]
        .actions
        .clone();
    // spec: S-0011 claim 11 (every action takes effect, in list order, even when an earlier one had no effect)
    for a in &actions {
        action(w, a, tag, inst.trigger, inst.triggering);
    }
}

fn action(
    w: &mut World,
    a: &glasnost_map::triggers::Action,
    tag: u32,
    trigger: u32,
    triggering: Option<HouseId>,
) {
    let t = w.tick;
    // spec: S-0011 Terms (owner house: the first house in house order playing the trigger's owner country)
    let owner = |w: &World| owner_country(w, trigger).and_then(|c| house_of_country(w, c));
    // Mission timer lengths: 15 × v (claim 31).
    let len = UNIT.wrapping_mul(a.number);
    match a.code {
        // spec: S-0011 claim 25 (enable: every existing instance of the definition whose difficulty flag is set; its timer re-armed)
        53 => {
            let Some(tr) = a.trigger else { return };
            if !difficulty_ok(w, tr) {
                return;
            }
            let tags: Vec<u32> = w.triggers.tags.keys().copied().collect();
            for tag in tags {
                let n = w.triggers.tags[&tag].instances.len();
                for k in 0..n {
                    let mut i = w.triggers.tags[&tag].instances[k].clone();
                    if i.trigger != tr || i.removed {
                        continue;
                    }
                    i.enabled = true;
                    // spec: S-0011 claim 15 (the enable action re-arms the timer, even when it was already enabled)
                    arm(w, &mut i);
                    if let Some(tg) = w.triggers.tags.get_mut(&tag) {
                        tg.instances[k] = i;
                    }
                }
            }
        }
        // spec: S-0011 claim 25 (disable: every instance of the named definition; the timer keeps running)
        54 => {
            let Some(tr) = a.trigger else { return };
            for tg in w.triggers.tags.values_mut() {
                for i in &mut tg.instances {
                    if i.trigger == tr {
                        i.enabled = false;
                    }
                }
            }
        }
        // spec: S-0011 claim 22 (set and clear variable; a variable outside 0 … 99: no effect)
        56 | 57 => set_variable(w, a.number, a.code == 56),
        // spec: S-0011 claim 31 (timer start: a stopped timer restarts with its remaining length)
        23 => {
            let m = &mut w.triggers.mission;
            if !m.running {
                m.running = true;
                m.shown = true;
                m.since = t;
            }
        }
        // spec: S-0011 claim 31 (timer stop: stopped, keeping the remaining length)
        24 => {
            let m = &mut w.triggers.mission;
            m.length = m.remaining(t);
            m.running = false;
        }
        // spec: S-0011 claim 31 (extend and shorten: running from the current tick with the remaining length ± 15v, not below 0)
        25 | 26 => {
            let m = &mut w.triggers.mission;
            let rem = m.remaining(t);
            let l = if a.code == 25 {
                rem.wrapping_add(len)
            } else {
                rem.wrapping_sub(len)
            };
            m.length = l.max(0);
            m.since = t;
            m.running = true;
            m.shown = true;
        }
        // spec: S-0011 claim 31 (timer set: 15v ticks from the current tick, the default label)
        27 => {
            w.triggers.mission = MissionTimer {
                shown: true,
                running: true,
                since: t,
                length: len,
                label: None,
            };
        }
        // spec: S-0011 claim 31 (timer text: the label from the action's text; an empty text gives the default label)
        103 => {
            w.triggers.mission.label = (!a.text.is_empty()).then(|| a.text.clone());
        }
        // spec: S-0011 claim 26 (reveal around a waypoint: the number slot is a waypoint number)
        17 => reveal(w, a.number),
        // spec: S-0011 claim 32 (presentation-only actions: text message, play sound, play speech, radar event at a waypoint, sound at a waypoint)
        11 | 19 | 21 => present(w, a, None),
        // spec: S-0011 I4 (55: radar event at a waypoint; 99: sound at a waypoint)
        55 | 99 => {
            if let Some(c) = waypoint_cell(w, a.waypoint) {
                present(w, a, Some(c));
            }
        }
        // spec: S-0011 claim 30 (actions that hand over to other specs: recognised, their effects belong to other topics: teams (no spec yet) and computer targeting, not built yet)
        4 | 5 | 7 | 80 | 135 => {
            w.note("S-0011 claim 30: a trigger action hands over to a topic not built yet (teams, computer targeting); no effect");
        }
        // spec: S-0011 particle-system actions table and S-0616 claim 1 (88: one system of the number slot's type at the waypoint's cell centre, ground surface height; always succeeds)
        88 => {
            if let Some(c) = waypoint_cell(w, a.waypoint) {
                crate::emitters::trigger_create(w, a.number, c);
            }
        }
        // spec: S-0011 particle-system actions table and S-0616 claim 1 (89: every system in the waypoint's cell is marked; always succeeds)
        89 => {
            if let Some(c) = waypoint_cell(w, a.waypoint) {
                crate::emitters::trigger_mark(w, c);
            }
        }
        // spec: S-0524 claim 4 (108 create a crate at a waypoint: crates, S-0520, are a seam)
        108 => crate::mp_triggers::create_crate(w, a),
        // spec: S-0524 claim 5 and I1 (63 demolition blast: the number slot as waypoint number)
        63 => crate::mp_triggers::demolition_blast(w, a.number),
        // spec: S-0524 claim 6 and I1 (66 large light flash: the number slot as waypoint number)
        66 => crate::mp_triggers::light_flash(w, a),
        // spec: S-0524 claim 7 and I1 (73 set base ambient: the number slot as integer hundredths)
        73 => crate::mp_triggers::set_ambient(w, a.number),
        // spec: S-0524 claim 8 and I1 (40 resize local area: the fourth to seventh items)
        40 => crate::mp_triggers::resize_local_area(w, a.items),
        // spec: S-0524 claim 9 and I1 (130 restore starting buildings: the number slot as house value)
        130 => crate::mp_triggers::restore_starting_buildings(w, a.number, triggering),
        // spec: S-0011 I4 (33 one-time superweapon grant: number slot as superweapon type index, trigger owner house; S-0512 claim 12)
        33 => {
            if let (Some(h), Ok(i)) = (owner(w), u32::try_from(a.number)) {
                crate::superweapons::charge::one_time_grant(w, h, i);
            }
        }
        // spec: S-0011 I4 (34 permanent superweapon grant; S-0512 claim 13)
        34 => {
            if let (Some(h), Ok(i)) = (owner(w), u32::try_from(a.number)) {
                crate::superweapons::charge::permanent_grant(w, h, i);
            }
        }
        // spec: S-0011 I4 (129 set superweapon charge: the kind 11 second number slot as percentage; S-0512 claim 14)
        129 => {
            if let (Some(h), Ok(i)) = (owner(w), u32::try_from(a.number)) {
                crate::superweapons::charge::set_charge(w, h, i, a.number2);
            }
        }
        // spec: S-0011 I4 (133 reset superweapon charge; S-0512 claim 15)
        133 => {
            if let (Some(h), Ok(i)) = (owner(w), u32::try_from(a.number)) {
                crate::superweapons::charge::reset(w, h, i);
            }
        }
        // spec: S-0524 claim 1 (102 lightning storm at a waypoint: the imported duration and deferment, owned by the trigger's owner house, possibly no house)
        102 => {
            if let Some(c) = waypoint_cell(w, a.waypoint) {
                let h = owner(w);
                let (dur, def) = {
                    let s = &w.rules().superweapons.storm;
                    (s.duration, s.deferment)
                };
                crate::superweapons::storm::request(w, h, c, dur, def);
            }
        }
        // spec: S-0513 claim 20 and S-0524 Summary (44 storm start: a storm whose duration is the action's number, the game-wide countdown, a random legal cell, no owner)
        44 => {
            crate::superweapons::storm::start_action(w, a.number);
        }
        // spec: S-0513 claim 20 and S-0524 Summary (45 storm stop: clearing begins while raging)
        45 => {
            crate::superweapons::storm::stop(w);
        }
        // spec: S-0524 claim 2 (90 lightning strike at a waypoint: a cloud whether or not a storm is active)
        90 => {
            if let Some(c) = waypoint_cell(w, a.waypoint) {
                crate::mp_triggers::lightning_strike(w, a.waypoint, c);
            }
        }
        // spec: S-0524 claim 3 (95 nuclear strike at a waypoint: no firer and no owning house; the number slot is unused)
        95 => {
            if let Some(c) = waypoint_cell(w, a.waypoint) {
                crate::superweapons::nuke::waypoint(w, c);
            }
        }
        // spec: S-0513 claim 22 and Import mapping (109 invulnerability at a waypoint: the waypoint slot, no house, the imported curtain duration; the number slot is unused)
        109 => {
            if let Some(c) = waypoint_cell(w, a.waypoint) {
                crate::superweapons::protect::curtain(w, None, c, true);
            }
        }
        // spec: S-0508 Import mapping and S-0011 pointer table (111 evict the occupants of the first eligible building attached to the firing trigger: no hunting, no inside fallback)
        111 => {
            let attached: Vec<ObjectId> = w
                .triggers
                .objects
                .iter()
                .filter(|&(&id, &t)| {
                    t == tag && w.obj(id).is_some_and(|o| o.kind == Kind::Building)
                })
                .map(|(&id, _)| id)
                .collect();
            crate::structures::garrison::trigger_evict(w, &attached);
        }
        // spec: S-0508 Import mapping and S-0011 claim 23 (6 house-wide hunt: the number slot is a house value; its garrison evictions hunt, without inside fallback)
        6 => {
            if let Some(h) = crate::mp_triggers::resolve_house(w, a.number, triggering) {
                // spec: S-0217 claim 1 (map action code 6 gives the house-wide hunt order to the house its number slot resolves to; a rule of our engine for every house) and S-0508 claim 13 (r39)
                crate::computer::hunt_order(w, h);
            }
        }
        // spec: S-0011 claim 24 (14 change owner: every present object that carries the tag holding the firing trigger changes its owner to the resolved house, without announcement; a carrier inside a transport or building keeps its owner)
        14 => {
            if let Some(h) = crate::mp_triggers::resolve_house(w, a.number, triggering) {
                let carriers: Vec<ObjectId> = w
                    .triggers
                    .objects
                    .iter()
                    .filter(|&(&id, &t)| t == tag && w.obj(id).is_some_and(|o| o.on_map))
                    .map(|(&id, _)| id)
                    .collect();
                for id in carriers {
                    // spec: S-0601 claim 10 (a trigger action that changes a controller's house releases all its victims)
                    crate::mind::trigger_owner_change(w, id);
                    let Some(o) = w.obj(id) else { continue };
                    if o.kind == Kind::Building {
                        crate::structures::ownership::change(
                            w,
                            id,
                            Some(h),
                            crate::structures::ownership::Cause::Trigger,
                        );
                    } else if o.owner != Some(h) {
                        let old = o.owner;
                        if let Some(ob) = w.obj_mut(id) {
                            ob.owner = Some(h);
                        }
                        crate::econ::events::owner_changed(w, id, old);
                        // spec: S-0309 claim 31 (a trigger's change-owner action, a listed cause of claim 30: the orders reset inside the change)
                        crate::structures::ownership::reset_orders(w, id);
                    }
                }
            }
        }
        // spec: S-0011 I4 (32 destroy attached, 42 explosion, 113 cheer, 8 drop-zone animation, 41 play an animation)
        32 | 42 | 113 | 8 | 41 => {
            w.note("S-0011 claims 27, 28, 29, 36: destroy attached, explosion, cheer and the waypoint animations are not implemented yet (M6); no effect");
        }
        _ => {
            w.note("S-0011 I4: a trigger action outside the subset; no effect");
        }
    }
}

/// Sets or clears a map trigger variable (S-0011 claim 22).
fn set_variable(w: &mut World, v: i32, on: bool) {
    let Some(i) = usize::try_from(v).ok().filter(|&i| i < VARIABLES) else {
        return;
    };
    // spec: S-0011 claim 22 (a change happens only when the state differs)
    if w.triggers.variables[i] == on {
        return;
    }
    w.triggers.variables[i] = on;
    // spec: S-0011 claim 22 (every instance with a variable event for that variable has its timer re-armed)
    let tags: Vec<u32> = w.triggers.tags.keys().copied().collect();
    for tag in tags {
        let n = w.triggers.tags[&tag].instances.len();
        for k in 0..n {
            let mut inst = w.triggers.tags[&tag].instances[k].clone();
            let uses = w.triggers.map.triggers[inst.trigger as usize]
                .events
                .iter()
                .any(|e| (e.code == VAR_SET || e.code == VAR_CLEAR) && e.value == v);
            if uses && !inst.removed {
                arm(w, &mut inst);
                if let Some(t) = w.triggers.tags.get_mut(&tag) {
                    t.instances[k] = inst;
                }
            }
        }
    }
}

/// Reveal around a waypoint (S-0011 claim 26, F6), for every player house
/// (our engine's departure).
fn reveal(w: &mut World, wp: i32) {
    if !(0..=LAST_WAYPOINT).contains(&wp) {
        // spec: S-0011 claim 33 (a reveal waypoint outside 0 … 701: no effect, a departure)
        w.note("S-0011 claim 33: a reveal action's waypoint lies outside 0 … 701; no effect");
        return;
    }
    let Some(c) = waypoint_cell(w, wp) else {
        return;
    };
    let r = w.rules().triggers.reveal_radius;
    if r < 0 {
        w.note("S-0011 claim 26: a negative reveal-trigger radius is not specified; no effect");
        return;
    }
    let (h0, bridge) = w.cells.idx(c).map_or((0, false), |i| {
        let level = w.cells.grid.get(c).map_or(0, |cell| i32::from(cell.level));
        (level, w.cells.deck[i])
    });
    // spec: S-0011 F6 (h = h₀ + 4 on a high bridge; k = h quo 2; the centre shifted k cells towards the top corner, raised by h levels)
    let h = if bridge { h0 + BRIDGE_LIFT } else { h0 };
    let k = h / 2; // spec: S-0011 F6 (k = h quo 2)
    let p = Pos::new(256 * (c.x - k) + 128, 256 * (c.y - k) + 128, LEVEL * h); // spec: S-0011 F6 (256 × (x − k) + 128, 256 × (y − k) + 128, 104 × h)
    // spec: S-0011 claim 26 (our engine applies the reveal to every player house, a departure)
    let houses: Vec<HouseId> = (0..w.houses.len())
        .filter(|&i| w.houses[i].role == HouseRole::Player && !w.houses[i].removed)
        .map(|i| HouseId(i as u8))
        .collect();
    for h in houses {
        crate::econ::shroud::area_reveal(w, h, p, r);
    }
}

/// A presentation-only action (S-0011 claim 32).
fn present(w: &mut World, a: &glasnost_map::triggers::Action, cell: Option<CellXY>) {
    let pos = cell.map(|c| {
        let z = w.surface_at(256 * c.x + 128, 256 * c.y + 128); // spec: S-0011 claim 32 (the cell's centre)
        Pos::centre(c, z)
    });
    w.effects.push(Effect::Trigger {
        code: a.code,
        name: String::from_utf8_lossy(&a.name).into_owned(),
        text: String::from_utf8_lossy(&a.text).into_owned(),
        number: a.number,
        pos,
    });
}

/// The unused-house cleanup's removal of a house's triggers (S-0009 claim
/// 28): the existing instances whose definition's owner country is `c`.
pub fn remove_country(w: &mut World, c: u32) {
    let owners: Vec<Option<u32>> = (0..w.triggers.map.triggers.len() as u32)
        .map(|t| owner_country(w, t))
        .collect();
    for tag in w.triggers.tags.values_mut() {
        for i in &mut tag.instances {
            if owners[i.trigger as usize] == Some(c) {
                i.removed = true;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::SimRules;

    /// The scan of every name of each kind in turn, keeping the last match.
    fn scanned(r: &SimRules, name: &[u8], exact: bool) -> Option<(Kind, u32)> {
        let mut found = None;
        for kind in TYPE_EVENT_KINDS {
            for ((k, n), &i) in &r.names {
                let eq = if exact {
                    n.as_bytes() == name
                } else {
                    n.as_bytes().eq_ignore_ascii_case(name)
                };
                if *k == kind && eq {
                    found = Some((kind, i));
                }
            }
        }
        found
    }

    // spec: S-0011 claim 21 (the type a type event names; quirks.exact_trigger_type_names)
    #[test]
    fn named_type_matches_the_scan_of_every_name() {
        let mut set = crate::testkit::ruleset();
        let mut add = |reg: &str, names: &[&str]| {
            set.registries
                .entry(reg.to_owned())
                .or_default()
                .extend(names.iter().map(|n| (*n).to_owned()));
        };
        add("buildings", &["Zq", "ZQ", "Yx"]);
        add("vehicles", &["zq", "Wv"]);
        add("infantry", &["ZQ", "wV"]);
        add("terrain_objects", &["Ut"]);
        let mut r = SimRules::new(&set, "synthetic-type-event-names");
        let probes: [&[u8]; 9] = [
            b"zq", b"ZQ", b"Zq", b"yX", b"Yx", b"wv", b"Wv", b"ut", b"\xff",
        ];
        let check = |r: &SimRules| {
            for name in probes {
                for exact in [false, true] {
                    assert_eq!(
                        named_type(r, name, exact),
                        scanned(r, name, exact),
                        "{name:?} exact {exact}"
                    );
                }
            }
        };
        check(&r);
        // Two buildings fold to "zq": the later name in order ("Zq") wins.
        let zq = r.index_of(Kind::Building, "Zq");
        assert_eq!(r.index_of_folded(Kind::Building, b"zq"), zq);
        assert_ne!(r.index_of(Kind::Building, "ZQ"), zq);
        assert_eq!(
            named_type(&r, b"ZQ", false).map(|(k, _)| k),
            Some(Kind::Infantry)
        );
        assert_eq!(
            named_type(&r, b"ut", false),
            None,
            "terrain is not searched"
        );
        // A name added after the build is still found.
        r.names.insert((Kind::Aircraft, "zQ".to_owned()), 7);
        check(&r);
        assert_eq!(named_type(&r, b"zq", false), Some((Kind::Aircraft, 7)));
    }
}
