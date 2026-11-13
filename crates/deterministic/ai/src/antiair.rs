//! The own AI and drainers in the air (M7b AI round 3, M8 step B6; PLAN
//! §4.7).
//!
//! A flying drainer over a building (S-0608) can be shot only by a weapon
//! whose projectile is anti-air (S-0306 claim 23) and whose warhead's
//! versus value against the drainer's armour is not zero (S-0304 claim 1).
//! In the stock lane a house drained for 9 000 ticks had nothing that could
//! hit the drainer. A person who sees enemy drainers in the air and has
//! nothing that can answer one of them adds something that can: the
//! cheapest unit or defence type he can start now whose weapons can hit
//! it, one at a time. The economy defence (`Brain::defend`) sends only
//! units the firing check lets attack the drainer. A ruleset with no such
//! type leaves the brain as it was.
//!
//! What answers a drainer (M8 step B6). Round 3 counted every object of a
//! type that could hit it, wherever it stood and whatever its power. In a
//! lane game the Allied house stood in drain blackout for some 80 000
//! ticks under one drainer, with its one anti-air defence dark and twelve
//! cells away, and built nothing more. Now:
//! - a defence answers when the firing check would let it fire at the
//!   drainer now but for the rearm, the facing and such (S-0301 claim 9;
//!   its own answer of S-0405 table T1 none: in operation and not being
//!   drained), and the drainer, or the building it drains or is after, is
//!   within its weapon's range (S-0307);
//! - a unit of the army answers when the firing check lets it attack the
//!   drainer, it stands near home or near the drainer (where the economy
//!   defence sends it from), and it flies or its movement zone reaches a
//!   cell within its weapon's range of the drainer (S-0212 claim 4);
//! - a unit item on a line answers, and so does a defence item that would
//!   be in operation when placed: none whose type the house's blackout
//!   stops (S-0405 claims 5 and 8). Such a type is not chosen either, so a
//!   house blacked out by a drained power plant trains anti-air units
//!   instead of queueing dark defences;
//! - each answer counts for one drainer only (one per drainer, as before);
//! - a finished anti-air defence is placed near the drainer it answers
//!   (`place_goal`), as a person drops it beside the drained building,
//!   not at the far side of the base.
//!
//! Every order is one a person gives with the mouse. It reads only the
//! house's own objects, the enemies it has seen and the rules a person may
//! read (its own buildings' power it sees on the bar and the base). It
//! draws no random value.

use glasnost_map::CellXY;
use glasnost_sim::combat::check::{self, Answer};
use glasnost_sim::combat::data::WeaponId;
use glasnost_sim::combat::{range, veterancy, weapon};
use glasnost_sim::econ::{TypeRef, power, production, tech};
use glasnost_sim::regions::{self, Reach};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::world::{HouseId, ObjectId, Target, World};

use crate::brain::RECALL;
use crate::drain::is_drainer;
use crate::scripted::{Role, Script, all_types, dist2};

/// The combat data of type `t`.
fn combat_of(w: &World, t: TypeRef) -> Option<&glasnost_sim::combat::data::TypeCombat> {
    let r = w.rules();
    match t.kind {
        Kind::Building => r.buildings.get(t.index as usize).map(|b| &b.combat),
        Kind::Terrain => None,
        _ => r.unit(t.kind, t.index).map(|u| &u.combat),
    }
}

/// The weapon slots of type `t`: the ordinary ones, and with `elite` the
/// elite ones too.
fn slots(w: &World, t: TypeRef, elite: bool) -> Vec<WeaponId> {
    combat_of(w, t)
        .map(|tc| {
            let none = [None; 0];
            let elite_slots: &[Option<WeaponId>] = if elite { &tc.elite_slots } else { &none };
            tc.slots
                .iter()
                .chain(elite_slots)
                .flatten()
                .copied()
                .collect()
        })
        .unwrap_or_default()
}

/// The ordinary weapons a new object of type `t` could fire at an object
/// of armour class `armour` in the air, by the weapon choice of S-0308: an
/// infantry type with the deploy-fire flag, not deploying and in no
/// transport, uses slot 0 (claim 8); a type with an empty slot 0 or 1, or
/// a never-use secondary, uses slot 0 (claim 4, table P row 3); an
/// airstrike secondary leaves slot 0 against a target that is not a
/// building (row 6); then the versus rows 15 and 16 and the air rows 19
/// and 20. A switching or gattling type (rows 1 and 5) and a draining
/// secondary (row 8) choose by the object's own state, so every ordinary
/// slot counts and the firing check decides once the object is on the
/// map. A new object rides in no transport, holds no occupants and is not
/// overpowered (rows 2, 4 and 10); the target is no building, cell or
/// water target (rows 7, 11, 13, 14 and 17, S-0308 Terms: a water target
/// is not in the air).
fn air_slots(w: &World, t: TypeRef, armour: usize) -> Vec<WeaponId> {
    let r = w.rules();
    let Some(tc) = combat_of(w, t) else {
        return Vec::new();
    };
    let (p, s) = (tc.slots[0], tc.slots[1]);
    // spec: S-0308 claim 8 (a deploy-fire infantry type not deploying and in no open-topped transport: slot 0)
    if t.kind == Kind::Infantry && tc.deploy_fire {
        return p.into_iter().collect();
    }
    // spec: S-0308 table P rows 1 and 5 (a switching or gattling type: its weapon number or stage, the object's own state)
    if tc.multi_weapon {
        return slots(w, t, false);
    }
    // spec: S-0308 claim 4 and table P row 3 (slot 0 or 1 empty, or a never-use secondary: 0)
    let (Some(pw), Some(sw)) = (p, s) else {
        return p.into_iter().collect();
    };
    if r.combat.weapon(sw).never_use {
        return vec![pw];
    }
    let warhead = |x: WeaponId| r.combat.warhead_of(x);
    // spec: S-0308 table P row 6 (an airstrike secondary: 0 against a target that is not a building)
    if warhead(sw).is_some_and(|h| h.airstrike) {
        return vec![pw];
    }
    // spec: S-0308 table P row 8 (a draining secondary: 1 while the firer drains nothing, the object's own state)
    if r.combat.weapon(sw).drain {
        return vec![pw, sw];
    }
    let zero =
        |x: WeaponId| warhead(x).is_some_and(|h| h.versus(armour).eq_num(Default::default()));
    // spec: S-0308 table P row 15 (the secondary's versus value is zero: 0)
    if zero(sw) {
        return vec![pw];
    }
    // spec: S-0308 table P row 16 (the primary's versus value is zero: 1)
    if zero(pw) {
        return vec![sw];
    }
    let aa = |x: WeaponId| r.combat.projectile_of(x).is_some_and(|p| p.aa);
    // spec: S-0308 table P row 19 (airborne and an anti-air secondary: 1; quirks.secondary_wins_against_air off: only when the primary is not anti-air)
    if aa(sw) && (r.quirks.secondary_wins_against_air || !aa(pw)) {
        return vec![sw];
    }
    // spec: S-0308 table P row 20 (otherwise 0)
    vec![pw]
}

/// Whether a new object of type `t` can hit an object of armour class
/// `armour` in the air with the weapon it would choose ([`air_slots`]):
/// an anti-air projectile (S-0306 claim 23), a warhead whose versus value
/// for the class is not zero (S-0304 claim 1) and damage above zero (a
/// healing weapon heals). A new object is not elite (S-0311 claim 11: a
/// produced unit starts a rookie or a veteran), and only an elite one
/// uses its elite weapons (S-0311 claim 6, S-0308 claim 2), so its
/// ordinary slots alone count: a type that hits the air only once elite
/// answers nothing when built, nor does one whose anti-air weapon the
/// weapon choice never takes against the air (M8 step B6).
pub fn hits_air(w: &World, t: TypeRef, armour: usize) -> bool {
    // spec: S-0311 claims 6 and 11 and S-0308 claim 2 (a produced object is not elite and uses its ordinary slots), S-0308 claims 4 and 8 and table P (the slot it chooses against the air)
    air_slots(w, t, armour)
        .into_iter()
        .any(|wp| weapon_hits_air(w, wp, armour))
}

/// Whether weapon `wp` can hit an object of armour class `armour` in the
/// air ([`hits_air`]).
fn weapon_hits_air(w: &World, wp: WeaponId, armour: usize) -> bool {
    let r = w.rules();
    // spec: S-0306 claim 23 (an airborne target needs an anti-air projectile)
    let aa = r.combat.projectile_of(wp).is_some_and(|p| p.aa);
    // spec: S-0304 claim 1 (a versus value of exactly zero makes the target illegal)
    let versus = r
        .combat
        .warhead_of(wp)
        .is_some_and(|h| !h.versus(armour).eq_num(Default::default()));
    aa && versus && r.combat.weapon(wp).damage > 0
}

/// Whether type `t` has a weapon, ordinary or (with `elite`) elite, that
/// can hit an object of armour class `armour` in the air, in any slot: the
/// first sift of the objects on the map, which the firing check then asks
/// with the weapon each would use.
fn slots_hit_air(w: &World, t: TypeRef, armour: usize, elite: bool) -> bool {
    slots(w, t, elite)
        .into_iter()
        .any(|wp| weapon_hits_air(w, wp, armour))
}

/// The seen enemy drainers in the air (S-0307 Terms: airborne by height),
/// with their armour classes.
pub fn drainers_in_air(script: &Script, w: &World) -> Vec<(ObjectId, usize)> {
    script
        .seen_enemies(w)
        .into_iter()
        .map(|(id, _)| id)
        .filter(|&id| is_drainer(w, id) && weapon::airborne_by_height(w, id))
        .filter_map(|id| veterancy::type_combat(w, id).map(|tc| (id, tc.armour)))
        .collect()
}

/// The building of house `house` that drainer `d` drains (S-0608 claim 2:
/// it stands over a cell of its foundation) or is after (its target or its
/// destination), as a person sees it hover over or fly at the building.
fn goal_building(w: &World, house: HouseId, d: ObjectId) -> Option<ObjectId> {
    let own = |b: ObjectId| {
        w.obj(b)
            .is_some_and(|o| o.kind == Kind::Building && o.owner == Some(house) && o.on_map)
    };
    let o = w.obj(d)?;
    let aimed = [o.target, o.dest]
        .into_iter()
        .flatten()
        .find_map(|t| match t {
            Target::Object(b) => Some(b),
            Target::Cell(_) => None,
        });
    glasnost_sim::drain::partner(w, d)
        .into_iter()
        .chain(aimed)
        .find(|&b| own(b))
}

/// The cell where drainer `d` works against the house: its goal
/// building's cell, or its own.
fn goal_cell(w: &World, house: HouseId, d: ObjectId) -> Option<CellXY> {
    goal_building(w, house, d)
        .or(Some(d))
        .and_then(|x| w.obj(x))
        .map(|o| o.pos.cell())
}

/// Whether defence `id` answers drainer `d`: its own answer of the firing
/// check is none (S-0405 claim 9, table T1: in operation, not being
/// drained, not building or selling), the firing check against the drainer
/// is neither *illegal* nor *impossible* (S-0301 claim 9; the anti-air and
/// versus tests of S-0306 claim 23 and S-0304 claim 1 among them), and the
/// drainer, or the building it drains or is after, is in range (S-0307).
fn defence_answers(w: &World, house: HouseId, id: ObjectId, d: ObjectId) -> bool {
    // spec: S-0405 claim 9 and table T1 (a building that is not in operation, or is being drained, cannot fire)
    if power::firing_answer(w, id).is_some() {
        return false;
    }
    let t = Some(Target::Object(d));
    let (slot, wp) = weapon::would_use(w, id, t);
    if wp.is_none() {
        return false;
    }
    // spec: S-0301 claim 9 (the own AI's lasting test, our design, PLAN §4.7: the firing check without the range test and without the reasons of the moment of ranks 5 to 8, so that rank 5's rearming hides neither rank 9's target tests nor its range answer, which is tested on its own below)
    if matches!(
        check::check_lasting(w, id, t, slot),
        Answer::Illegal | Answer::Impossible
    ) {
        return false;
    }
    // spec: S-0307 claims 1 to 11 (the range test, against the drainer or the building it drains or is after, which stands under it: S-0608 claim 2)
    range::in_range(w, id, wp, t, None)
        || goal_building(w, house, d)
            .is_some_and(|b| range::in_range(w, id, wp, Some(Target::Object(b)), None))
}

/// Whether army unit `id` can get within its weapon's reach of drainer
/// `d`: it flies (an aircraft, or a jumpjet, S-0502 Terms), or its
/// movement zone's reach region (S-0212 claim 4) holds a cell within its
/// weapon's range of the drainer's cell. A unit outside every region (a
/// factory's exit) is not ruled out.
fn gets_there(w: &World, id: ObjectId, d: ObjectId, wp: WeaponId) -> bool {
    let Some(o) = w.obj(id) else { return false };
    if o.kind == Kind::Aircraft || glasnost_sim::jump::is_jumpjet(w, id) {
        return true;
    }
    let Some(zone) = w.rules().unit(o.kind, o.type_index).map(|u| u.zone) else {
        return true;
    };
    let Reach::Region(here) = regions::reach_in(w, o.pos.cell(), zone) else {
        return true;
    };
    let reach = w.rules().combat.weapon(wp).range;
    // spec: S-0307 claim 2 (the unlimited-range sentinel passes for every target)
    if reach == range::UNLIMITED {
        return true;
    }
    let bonus = veterancy::type_combat(w, id).map_or(0, |tc| tc.air_range_bonus);
    // spec: S-0307 F1 (the range against an airborne target adds the air range bonus) and S-0204 claim 3 (a cell is 256 leptons across), in whole cells
    let r = (i64::from(reach) + i64::from(bonus)).max(0) / 256;
    let Some(dc) = w.obj(d).map(|x| x.pos.cell()) else {
        return false;
    };
    let r32 = r as i32;
    (-r32..=r32).any(|dy| {
        (-r32..=r32).any(|dx| {
            let c = CellXY::new(dc.x + dx, dc.y + dy);
            dist2(c, dc) <= r * r && regions::reach_in(w, c, zone) == Reach::Region(here)
        })
    })
}

/// Whether army unit `id` stands off drainer `d`: it holds the drainer as
/// its target, the firing check answers *out of range* (S-0301 claim 9
/// rank 9), and it is under no attack order and on its way nowhere else
/// (its destination is none or its own cell), so it neither fires nor
/// closes in. A person sees it stand there; the lane showed such a unit
/// counted as the answer for thousands of ticks while the drainer drained
/// on (M8 step B6).
fn stands_off(w: &World, id: ObjectId, d: ObjectId, slot: usize) -> bool {
    let Some(o) = w.obj(id) else { return false };
    let t = Some(Target::Object(d));
    let attacking =
        o.beh.current == Some(Behaviour::Attack) || o.beh.queued == Some(Behaviour::Attack);
    let going = match o.dest {
        None => false,
        Some(Target::Cell(c)) => c != o.pos.cell(),
        Some(Target::Object(x)) => x != id,
    };
    // nocite: own AI (what a person sees of the unit: it holds the drainer, does not fire and does not move)
    o.target == t
        && !attacking
        && !going
        // spec: S-0301 claim 9 rank 9 (the range answer of the firing check)
        && check::check(w, id, t, slot, true) == Answer::OutOfRange
}

/// Whether army unit `id` answers drainer `d`: the firing check lets it
/// attack the drainer (neither *illegal* nor *impossible*, S-0301 claim
/// 9), it is not berserk (it takes no orders, S-0610 claim 6), it stands
/// within [`RECALL`] cells of home or of the drainer, it gets there, and
/// it does not stand off the drainer ([`stands_off`]).
fn unit_answers(w: &World, home: Option<CellXY>, id: ObjectId, d: ObjectId) -> bool {
    let (Some(o), Some(dc)) = (w.obj(id), w.obj(d).map(|x| x.pos.cell())) else {
        return false;
    };
    let at = o.pos.cell();
    let near =
        home.is_some_and(|h| dist2(at, h) <= RECALL * RECALL) || dist2(at, dc) <= RECALL * RECALL;
    // spec: S-0610 claim 6 (a berserk unit takes no orders)
    if !near || glasnost_sim::psychic::berserk(w, id) {
        return false;
    }
    let t = Some(Target::Object(d));
    let (slot, wp) = weapon::would_use(w, id, t);
    let Some(wp) = wp else { return false };
    // spec: S-0301 claim 9 (the own AI's lasting test, our design, PLAN §4.7: the firing check without the range test and without the reasons of the moment, as the economy defence asks it)
    !matches!(
        check::check_lasting(w, id, t, slot),
        Answer::Illegal | Answer::Impossible
    ) && gets_there(w, id, d, wp)
        && !stands_off(w, id, d, slot)
}

/// Whether a building of type `t` placed now would be in operation as far
/// as the house's power goes (S-0405 claim 8: the powered row under a
/// blackout, the no-blackout row) and could fire at all (table T1: no
/// pulse cannon, no occupiable type, no type that needs an engineer). A
/// blackout gives the house low power whatever plants it builds (claim
/// 5); plain low power the house answers with a plant first
/// (`power::plant_first`), so it does not stop the choice. A unit type
/// always passes.
pub(crate) fn lit_when_placed(w: &World, house: HouseId, t: TypeRef) -> bool {
    if t.kind != Kind::Building {
        return true;
    }
    let Some(b) = w.rules().econ.power.buildings.get(t.index as usize) else {
        return true;
    };
    let blackout = power::blackout_running(w, house)
        || w.houses
            .get(usize::from(house.0))
            .is_some_and(|x| x.econ.power.drain_blackout);
    // spec: S-0405 claim 8 (powered: the needs-power flag and a base drain above 0 under low power; a blackout leaves the house with output 0, claim 5)
    let unpowered = b.needs_power && b.drain > 0 && blackout;
    // spec: S-0405 claim 8 (no blackout: the special-power flag and a blackout in force)
    let special = b.special_power && blackout;
    // spec: S-0405 table T1 (a pulse cannon answers impossible; an occupiable type without firing occupants illegal) and claim 8 (a needs-engineer building is not in operation until manned)
    let never = b.pulse_cannon || b.occupiable || b.needs_engineer;
    !(unpowered || special || never)
}

/// The drainers of `threats` that nothing of the house on the map
/// answers, each answer counted for one drainer only (the first in
/// threat order it answers).
fn unanswered(script: &Script, w: &World, threats: &[(ObjectId, usize)]) -> Vec<(ObjectId, usize)> {
    threats
        .iter()
        .zip(answers(script, w, threats))
        .filter(|(_, a)| a.is_none())
        .map(|(&t, _)| t)
        .collect()
}

/// Per drainer of `threats`, the object of the house on the map that
/// answers it, each answer counted for one drainer only (the first in
/// threat order it answers).
fn answers(script: &Script, w: &World, threats: &[(ObjectId, usize)]) -> Vec<Option<ObjectId>> {
    let house = script.house;
    let home = script.home(w);
    // An object on the map may be elite: its elite slots count here, and
    // the firing check below asks with the weapon it would use.
    let hits = |t: TypeRef| threats.iter().any(|&(_, a)| slots_hit_air(w, t, a, true));
    let defences: Vec<ObjectId> = script
        .own(w)
        .into_iter()
        .filter(|&id| {
            w.obj(id)
                .is_some_and(|o| o.kind == Kind::Building && hits(tech::type_of(o)))
        })
        .collect();
    let units: Vec<ObjectId> = script
        .of_role(w, Role::Army)
        .into_iter()
        .filter(|&id| w.obj(id).is_some_and(|o| hits(tech::type_of(o))))
        .collect();
    let mut used: Vec<ObjectId> = Vec::new();
    let mut out = Vec::new();
    for &(d, _) in threats {
        let answer = defences
            .iter()
            .chain(&units)
            .copied()
            .filter(|x| !used.contains(x))
            .find(|&x| {
                if defences.contains(&x) {
                    defence_answers(w, house, x, d)
                } else {
                    unit_answers(w, home, x, d)
                }
            });
        if let Some(x) = answer {
            used.push(x);
        }
        out.push(answer);
    }
    out
}

/// What the anti-air step sees now (local-lane reports only, the lane's
/// `GLASNOST_AI_AAPROBE`).
#[derive(Clone, Debug, Default)]
pub struct Probe {
    /// The seen enemy drainers in the air, each with what answers it.
    pub drainers: Vec<(ObjectId, Option<ObjectId>)>,
    /// Seen enemy drainers linked with one of the house's buildings that
    /// are not counted as in the air.
    pub draining_on_ground: usize,
    /// Answers on the house's lines ([`lined`]) for the unanswered ones.
    pub lined: usize,
    /// The type [`build_type`] would order now.
    pub build: Option<TypeRef>,
    /// The types that hit an unanswered drainer, by test passed: hits,
    /// lit when placed, startable, line free.
    pub candidates: Vec<usize>,
    /// The house's army units whose type hits the first drainer, by test
    /// passed ([`unit_answers`]): hits, near home or the drainer, not
    /// berserk, the firing check, gets there and does not stand off.
    pub units: Vec<usize>,
    /// The house's defences whose type hits the first drainer, by test
    /// passed ([`defence_answers`]): hits, in operation, a weapon, the
    /// firing check, in range.
    pub defences: Vec<usize>,
    /// The first such unit the firing check refuses: its type, the slot
    /// it would use and the answer.
    pub refused: Option<(TypeRef, usize, Answer)>,
}

/// The tests [`Probe::units`] counts, in order.
pub const UNIT_TESTS: &[&str] = &["hits", "near", "unberserk", "check", "there"]; // nocite: probe labels

/// The tests [`Probe::defences`] counts, in order.
pub const DEFENCE_TESTS: &[&str] = &["hits", "operating", "weapon", "check", "range"]; // nocite: probe labels

/// Counts one more object past test `n` of a probe's tests and moves on
/// to the next test.
fn passed(counts: &mut [usize], n: &mut usize) {
    if let Some(c) = counts.get_mut(*n) {
        *c += 1;
    }
    *n += 1;
}

/// What the anti-air step sees now ([`Probe`]).
pub fn probe(script: &Script, w: &World) -> Probe {
    let threats = drainers_in_air(script, w);
    let found = answers(script, w, &threats);
    let draining_on_ground = script
        .seen_enemies(w)
        .into_iter()
        .map(|(id, _)| id)
        .filter(|&id| is_drainer(w, id) && !threats.iter().any(|&(t, _)| t == id))
        .filter(|&id| {
            glasnost_sim::drain::partner(w, id)
                .and_then(|b| w.obj(b))
                .is_some_and(|b| b.owner == Some(script.house))
        })
        .count();
    let left: Vec<usize> = threats
        .iter()
        .zip(&found)
        .filter(|(_, a)| a.is_none())
        .map(|(&(_, a), _)| a)
        .collect();
    let types = all_types(w);
    let hits: Vec<TypeRef> = types
        .into_iter()
        .filter(|&t| left.iter().any(|&a| hits_air(w, t, a)))
        .collect();
    let lit: Vec<TypeRef> = hits
        .iter()
        .copied()
        .filter(|&t| lit_when_placed(w, script.house, t))
        .collect();
    let start: Vec<TypeRef> = lit
        .iter()
        .copied()
        .filter(|&t| crate::startable(w, script.house, t))
        .collect();
    let free = start.iter().filter(|&&t| !script.busy(w, t)).count();
    let mut units = vec![0; UNIT_TESTS.len()];
    let mut defences = vec![0; DEFENCE_TESTS.len()];
    let mut refused = None;
    if let Some(&(d, a)) = threats.first() {
        let home = script.home(w);
        let dc = w.obj(d).map(|x| x.pos.cell());
        for id in script.of_role(w, Role::Army) {
            let Some(o) = w.obj(id) else { continue };
            let mut n = 0;
            if !slots_hit_air(w, tech::type_of(o), a, true) {
                continue;
            }
            passed(&mut units, &mut n);
            let at = o.pos.cell();
            let near = home.is_some_and(|h| dist2(at, h) <= RECALL * RECALL)
                || dc.is_some_and(|c| dist2(at, c) <= RECALL * RECALL);
            if !near {
                continue;
            }
            passed(&mut units, &mut n);
            if glasnost_sim::psychic::berserk(w, id) {
                continue;
            }
            passed(&mut units, &mut n);
            let t = Some(Target::Object(d));
            let (slot, wp) = weapon::would_use(w, id, t);
            let Some(wp) = wp else { continue };
            // spec: S-0301 claim 9 (the own AI's lasting test, as unit_answers asks it)
            let answer = check::check_lasting(w, id, t, slot);
            if matches!(answer, Answer::Illegal | Answer::Impossible) {
                refused = refused.or(Some((tech::type_of(o), slot, answer)));
                continue;
            }
            passed(&mut units, &mut n);
            if gets_there(w, id, d, wp) && !stands_off(w, id, d, slot) {
                passed(&mut units, &mut n);
            }
        }
        for id in script.own(w) {
            let Some(o) = w.obj(id) else { continue };
            let mut n = 0;
            if o.kind != Kind::Building || !slots_hit_air(w, tech::type_of(o), a, true) {
                continue;
            }
            passed(&mut defences, &mut n);
            if power::firing_answer(w, id).is_some() {
                continue;
            }
            passed(&mut defences, &mut n);
            let t = Some(Target::Object(d));
            let (slot, wp) = weapon::would_use(w, id, t);
            if wp.is_none() {
                continue;
            }
            passed(&mut defences, &mut n);
            // spec: S-0301 claim 9 (the own AI's lasting test, as defence_answers asks it)
            if matches!(
                check::check_lasting(w, id, t, slot),
                Answer::Illegal | Answer::Impossible
            ) {
                continue;
            }
            passed(&mut defences, &mut n);
            if defence_answers(w, script.house, id, d) {
                passed(&mut defences, &mut n);
            }
        }
    }
    Probe {
        refused,
        units,
        defences,
        drainers: threats.iter().map(|&(d, _)| d).zip(found).collect(),
        draining_on_ground,
        lined: if left.is_empty() {
            0
        } else {
            lined(script, w, &left)
        },
        build: build_type(script, w),
        candidates: vec![hits.len(), lit.len(), start.len(), free],
    }
}

/// The items on the house's lines that will answer one of `armours` in
/// the air: units of a type that hits one, and defences of such a type that
/// would be in operation when placed ([`lit_when_placed`]).
fn lined(script: &Script, w: &World, armours: &[usize]) -> usize {
    let house = script.house;
    let hits =
        |t: TypeRef| armours.iter().any(|&a| hits_air(w, t, a)) && lit_when_placed(w, house, t);
    w.econ
        .lines
        .lines
        .iter()
        .filter(|l| l.house == house)
        .map(|l| {
            l.item.iter().filter(|i| hits(i.t)).count()
                + l.queue.iter().filter(|&&q| hits(q)).count()
        })
        .sum()
}

/// The type to build against drainers in the air: while more of the seen
/// drainers go unanswered than its lines hold answers for, the cheapest
/// unit or defence type the house can start now that hits one of them and
/// would be in operation when placed (on equal price the first in type
/// order), when its line is free.
pub(crate) fn build_type(script: &Script, w: &World) -> Option<TypeRef> {
    build_type_where(script, w, |_| true)
}

/// [`build_type`] among the types `ok` lets the house order in this think.
pub(crate) fn build_type_where(
    script: &Script,
    w: &World,
    ok: impl Fn(TypeRef) -> bool,
) -> Option<TypeRef> {
    let threats = drainers_in_air(script, w);
    if threats.is_empty() {
        return None;
    }
    let left = unanswered(script, w, &threats);
    if left.is_empty() {
        return None;
    }
    let armours: Vec<usize> = left.iter().map(|&(_, a)| a).collect();
    if lined(script, w, &armours) >= left.len() {
        return None;
    }
    all_types(w)
        .into_iter()
        .filter(|&t| armours.iter().any(|&a| hits_air(w, t, a)))
        .filter(|&t| lit_when_placed(w, script.house, t))
        .filter(|&t| crate::startable(w, script.house, t))
        .filter(|&t| !script.busy(w, t))
        .filter(|&t| ok(t))
        .min_by_key(|&t| (production::cost_field(w, t), t.kind, t.index))
}

/// Where to place a finished defence of type `t`: when it can hit a seen
/// drainer in the air that nothing answers, the cell that drainer works on
/// (the building it drains or is after, or its own cell), the first such
/// drainer in object order. `None` otherwise (the usual spiral).
pub(crate) fn place_goal(script: &Script, w: &World, t: TypeRef) -> Option<CellXY> {
    if t.kind != Kind::Building {
        return None;
    }
    let threats = drainers_in_air(script, w);
    if !threats.iter().any(|&(_, a)| hits_air(w, t, a)) {
        return None;
    }
    unanswered(script, w, &threats)
        .into_iter()
        .find(|&(_, a)| hits_air(w, t, a))
        .and_then(|(d, _)| goal_cell(w, script.house, d))
}
