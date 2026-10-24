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
use glasnost_sim::rules::Kind;
use glasnost_sim::world::{HouseId, ObjectId, Target, World};

use crate::brain::RECALL;
use crate::drain::is_drainer;
use crate::scripted::{Role, Script, all_types, dist2};

/// The weapon slots of type `t`, ordinary and elite.
fn slots(w: &World, t: TypeRef) -> Vec<WeaponId> {
    let r = w.rules();
    let tc = match t.kind {
        Kind::Building => r.buildings.get(t.index as usize).map(|b| &b.combat),
        Kind::Terrain => None,
        _ => r.unit(t.kind, t.index).map(|u| &u.combat),
    };
    tc.map(|tc| {
        tc.slots
            .iter()
            .chain(&tc.elite_slots)
            .flatten()
            .copied()
            .collect()
    })
    .unwrap_or_default()
}

/// Whether type `t` has a weapon that can hit an object of armour class
/// `armour` in the air: an anti-air projectile (S-0306 claim 23), a
/// warhead whose versus value for the class is not zero (S-0304 claim 1)
/// and damage above zero (a healing weapon heals).
pub fn hits_air(w: &World, t: TypeRef, armour: usize) -> bool {
    let r = w.rules();
    slots(w, t).into_iter().any(|wp| {
        // spec: S-0306 claim 23 (an airborne target needs an anti-air projectile)
        let aa = r.combat.projectile_of(wp).is_some_and(|p| p.aa);
        // spec: S-0304 claim 1 (a versus value of exactly zero makes the target illegal)
        let versus = r
            .combat
            .warhead_of(wp)
            .is_some_and(|h| !h.versus(armour).eq_num(Default::default()));
        aa && versus && r.combat.weapon(wp).damage > 0
    })
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
    // spec: S-0301 claim 9 (the firing check without the range test; rearming, facing and the like pass, and rank 5's rearming would hide rank 9's range answer, so the range is tested on its own below)
    if matches!(
        check::check(w, id, t, slot, false),
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

/// Whether army unit `id` answers drainer `d`: the firing check lets it
/// attack the drainer (neither *illegal* nor *impossible*, S-0301 claim
/// 9), it is not berserk (it takes no orders, S-0610 claim 6), it stands
/// within [`RECALL`] cells of home or of the drainer, and it gets there.
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
    // spec: S-0301 claim 9 (the firing check without the range test, as the economy defence asks it)
    !matches!(
        check::check(w, id, t, slot, false),
        Answer::Illegal | Answer::Impossible
    ) && gets_there(w, id, d, wp)
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
    let house = script.house;
    let home = script.home(w);
    let hits = |t: TypeRef| threats.iter().any(|&(_, a)| hits_air(w, t, a));
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
    let mut left = Vec::new();
    for &(d, a) in threats {
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
        match answer {
            Some(x) => used.push(x),
            None => left.push((d, a)),
        }
    }
    left
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
