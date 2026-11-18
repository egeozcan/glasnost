//! The own AI and its units under air attack (design: own AI, #434; PLAN
//! §4.7).
//!
//! A unit whose weapons cannot hit the air stands under an air unit that
//! shoots it: the firing check against an airborne target is *illegal*
//! without an anti-air projectile (S-0306 claim 23), so it does not
//! retaliate (S-0215 claim 18), and an AI house's unit scatters only with
//! the player-scatter setting. A person sees the shots come in and moves
//! it. The brain does the same with each army unit of the house that a
//! seen enemy in the air is shooting ([`Special::air_attack`]):
//! - toward the nearest unit or defence of the house that can hit the
//!   attacker, within [`COVER`] cells; one already within [`COVER_NEAR`]
//!   cells of it stays, under its cover;
//! - else, out in the field (beyond [`RECALL`] cells of home), back toward
//!   home;
//! - else out of the attacker's reach: to the nearest cell of the unit's
//!   reach region (S-0212 claim 4) the attacker's weapon does not reach
//!   with [`MARGIN`] cells to spare (S-0307 claim 13, the range a person
//!   reads off the weapon).
//!
//! Up to [`HELPERS`] idle army units of the house within [`HELP_REACH`]
//! cells of the unit that can hit the attacker are sent at it too; none
//! that answers a seen drainer in the air (`antiair`), which that part
//! keeps for its drainer.
//!
//! A unit sent away keeps its order ([`Watch::keeps_order`]): while it
//! moves to its goal, and after that while the attacker stays in the air
//! within [`COVER`] cells of it, for up to [`HOLD`] ticks from the order,
//! no other part of the brain orders it (in a test scene the base's rally
//! move sent it straight back under the attacker). It is not ordered again
//! until it arrives or its move ends. The module gives
//! orders only on the think ticks of the special part, and only when what
//! it sees changes: a unit already on its way, at its goal, out of the
//! attacker's reach or no longer shot is left alone.
//!
//! A unit that can hit its air attacker fights it (#443): retaliation
//! gives it the attacker as its target (S-0215 claim 18). When the
//! attacker is slow, still within [`OVERHEAD`] cells of the unit one think
//! after it was, the unit steps [`STEP`] cells away from it, at most once
//! in [`STEP_GAP`] ticks, so the attacker has to move to stay over it
//! ([`Special::air_step`]). No attack order follows the step: retaliation
//! brings the unit back to firing, as a fresh attack order would
//! (`tests/air_step.rs`). A unit that answers another seen drainer in the
//! air is left to `antiair`.
//!
//! Neither the escape nor the step takes a unit off a job another part
//! gave it: only a unit on a fighting unit's behaviour is moved
//! ([`free_of_job`]); one on its way into a bunker, or an engineer on a
//! capture at the door, keeps its job (the quick lane's trace showed a
//! capture given up for the way home).
//!
//! What "can hit" means: the weapon the unit would use against the
//! attacker (S-0308 claim 3) has an anti-air projectile and a warhead
//! whose versus value for the attacker's armour is not zero (`antiair`'s
//! test, S-0306 claim 23 and S-0304 claim 1), and the firing check's
//! lasting answer is neither *illegal* nor *impossible* (S-0301 claim 9);
//! for a defence, also its own answer of S-0405 table T1 is none.
//!
//! It reads only the house's own objects, the enemies it sees (where they
//! are, what they aim at, when they last fired: the shots a person sees)
//! and the rules a person may read. It draws no random value. The
//! counters [`crate::special::Uses::air_escapes`],
//! [`crate::special::Uses::air_helpers`] and
//! [`crate::special::Uses::air_steps`] count its orders.

use std::collections::BTreeMap;

use glasnost_map::CellXY;
use glasnost_sim::combat::check::{self, Answer};
use glasnost_sim::combat::data::{SLOTS, WeaponId};
use glasnost_sim::combat::{range, veterancy, weapon};
use glasnost_sim::econ::power;
use glasnost_sim::orders::Order;
use glasnost_sim::regions::{self, Reach};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::targeting;
use glasnost_sim::world::{ObjectId, Target, World};
use serde::{Deserialize, Serialize};

use crate::brain::RECALL;
use crate::scripted::{self, Role, Script, dist2};
use crate::special::{Special, Use};

/// Ticks since an air unit's last shot within which a person still counts
/// it as shooting.
const RECENT: i32 = 90; // nocite: tuning of our own opponent

/// Cells from the unit within which a unit or defence of the house that
/// can hit the attacker is gone to for cover.
pub const COVER: i64 = 20; // nocite: tuning of our own opponent

/// Cells from its cover within which a unit counts as under it.
pub const COVER_NEAR: i64 = 3; // nocite: tuning of our own opponent

/// Cells beyond the attacker's weapon range a unit sent out of its reach
/// goes.
pub const MARGIN: i64 = 2; // nocite: tuning of our own opponent

/// The most units of the house sent at one attacker.
pub const HELPERS: usize = 2; // nocite: tuning of our own opponent

/// Cells from the unit within which idle units are sent at its attacker.
pub const HELP_REACH: i64 = 15; // nocite: tuning of our own opponent

/// Rings of cells around a goal searched for one the unit can drive to.
const SEARCH_RINGS: i32 = 6; // nocite: tuning of our own opponent

/// Ticks after the order during which a unit sent away is kept for it
/// before it is seen moving: one think period.
const TAKE: i32 = scripted::THINK_PERIOD; // nocite: tuning of our own opponent

/// The longest a unit sent away is kept for its move.
const HOLD: i32 = 450; // nocite: tuning of our own opponent

/// Cells within which a unit has reached its goal.
const ARRIVED: i64 = 2; // nocite: tuning of our own opponent

/// The eight ways from a cell, clockwise from north.
const WAYS: [(i32, i32); 8] = [
    (0, -1),
    (1, -1),
    (1, 0),
    (1, 1),
    (0, 1),
    (-1, 1),
    (-1, 0),
    (-1, -1),
]; // nocite: the eight neighbour directions

/// A unit sent away from an air attacker.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Away {
    /// Where it was sent.
    goal: CellXY,
    /// The tick of the order.
    at: i32,
    /// The attacker it was sent away from.
    from: ObjectId,
}

/// Cells from a unit within which an air attacker is over it (#443).
pub const OVERHEAD: i64 = 2; // nocite: tuning of our own opponent

/// Cells a unit under a slow air attacker steps aside (#443).
pub const STEP: i32 = 3; // nocite: tuning of our own opponent

/// The fewest ticks between two steps of one unit (#443).
pub const STEP_GAP: i32 = 90; // nocite: tuning of our own opponent

/// What the module remembers between thinks.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Watch {
    /// The units sent away from an air attacker.
    away: BTreeMap<ObjectId, Away>,
    /// The units of the house that an air attacker shooting them was over
    /// at a think (#443): the attacker and the tick of that think.
    #[serde(default)]
    over: BTreeMap<ObjectId, (ObjectId, i32)>,
    /// The units of the house stepped aside from a slow air attacker
    /// (#443): the goal and the tick of the order.
    #[serde(default)]
    stepped: BTreeMap<ObjectId, (CellXY, i32)>,
}

impl Away {
    /// Whether unit `id` is on its way: the order is new (within [`TAKE`]
    /// ticks), or the unit moves to the goal under it and has not arrived,
    /// for at most [`HOLD`] ticks.
    fn on_way(&self, w: &World, id: ObjectId) -> bool {
        let Some(o) = w.obj(id) else { return false };
        let age = w.tick.saturating_sub(self.at);
        age <= TAKE
            || (age <= HOLD
                && o.beh.current == Some(Behaviour::Move)
                && o.dest == Some(Target::Cell(self.goal))
                && dist2(o.pos.cell(), self.goal) > ARRIVED * ARRIVED)
    }

    /// Whether unit `id` is kept back: on its way, or, within [`HOLD`]
    /// ticks of the order, while the attacker is still seen in the air
    /// within [`COVER`] cells of it (a person does not send the unit back
    /// under an air unit that hangs about).
    fn held(&self, w: &World, id: ObjectId) -> bool {
        let Some(o) = w.obj(id) else { return false };
        if self.on_way(w, id) {
            return true;
        }
        let near = o.owner.is_some_and(|h| crate::in_view(w, h, self.from))
            // spec: S-0308 Terms (airborne)
            && weapon::airborne(w, self.from)
            && w.obj(self.from)
                .is_some_and(|a| dist2(a.pos.cell(), o.pos.cell()) <= COVER * COVER);
        w.tick.saturating_sub(self.at) <= HOLD && near
    }
}

impl Watch {
    /// Whether unit `id` is on its way away from an air attacker
    /// ([`Away::on_way`]).
    pub(crate) fn sent(&self, w: &World, id: ObjectId) -> bool {
        self.away.get(&id).is_some_and(|e| e.on_way(w, id))
    }

    /// Forgets the units no longer kept back ([`Away::held`]), the attackers
    /// over a unit seen before the last think, and the steps older than
    /// [`STEP_GAP`].
    fn prune(&mut self, w: &World) {
        self.away.retain(|&id, e| e.held(w, id));
        let t = w.tick;
        self.over.retain(|&id, &mut (_, at)| {
            w.obj(id).is_some() && t.saturating_sub(at) <= scripted::THINK_PERIOD
        });
        self.stepped
            .retain(|&id, &mut (_, at)| w.obj(id).is_some() && t.saturating_sub(at) < STEP_GAP);
    }

    /// Whether the brain keeps order `o`: an order to a unit kept back
    /// from an air attacker ([`Away::held`]) stands only when it is that
    /// unit's move (a person who sent it away does not click it elsewhere
    /// meanwhile).
    pub(crate) fn keeps_order(&self, w: &World, o: &Order) -> bool {
        let Some(a) = o.actor() else { return true };
        let Some(e) = self.away.get(&a) else {
            return true;
        };
        if !e.held(w, a) {
            return true;
        }
        matches!(o, Order::Behaviour {
            behaviour: Behaviour::Move,
            dest: Some(Target::Cell(c)),
            ..
        } if *c == e.goal)
    }
}

/// Whether `id` of the house can hit enemy `a` in the air: the weapon it
/// would use against it (S-0308 claim 3) can hit the air ([`hits`]) and
/// the firing check's lasting answer allows the attack.
pub(crate) fn answers(w: &World, id: ObjectId, a: ObjectId) -> bool {
    let t = Some(Target::Object(a));
    let (slot, wp) = weapon::would_use(w, id, t);
    let Some(wp) = wp else { return false };
    let Some(armour) = veterancy::type_combat(w, a).map(|tc| tc.armour) else {
        return false;
    };
    // spec: S-0306 claim 23 and S-0304 claim 1 (an anti-air projectile and a versus value that is not zero, `antiair`)
    crate::antiair::weapon_hits_air(w, wp, armour)
        // spec: S-0301 claim 9 (the own AI's lasting test, our design, PLAN §4.7: the firing check without the range test and without the reasons of the moment)
        && !matches!(
            check::check_lasting(w, id, t, slot),
            Answer::Illegal | Answer::Impossible
        )
}

/// Whether defence `b` of the house can hit enemy `a` in the air now: its
/// own answer of the firing check is none (S-0405 claim 9 and table T1)
/// and it [`answers`] the enemy.
fn defence_answers(w: &World, b: ObjectId, a: ObjectId) -> bool {
    // spec: S-0405 claim 9 and table T1 (a building that is not in operation, or is being drained, cannot fire)
    power::firing_answer(w, b).is_none() && answers(w, b, a)
}

/// Whether unit `id` of the house is one this module moves: on the map,
/// selectable, not in the air (S-0308 Terms), not an aircraft, and free to
/// take a move order: not berserk (S-0610 claim 6), not bunkered (S-0606),
/// not deployed (S-0517 claim 9, S-0215 claim 22) and neither a lifter
/// (`lifter`) nor a drainer (`drain`), whose own parts order them.
fn movable(w: &World, id: ObjectId) -> bool {
    let Some(o) = w.obj(id) else { return false };
    // spec: S-0308 Terms (airborne), S-0610 claim 6 (a berserk unit takes no orders), S-0606 (a bunkered vehicle refuses a destination), S-0517 claim 9 and S-0215 claim 22 (a deployed unit takes no destination)
    o.on_map
        && o.kind != Kind::Aircraft
        && o.kind != Kind::Building
        && targeting::selectable(w, id)
        && !weapon::airborne(w, id)
        && !glasnost_sim::psychic::berserk(w, id)
        && !glasnost_sim::structures::bunker::bunkered(w, id)
        && !glasnost_sim::econ::inplace::blocks_destination(w, id)
        && !crate::deployers::infantry_deployed(w, id)
        && !crate::lifter::is_lifter(w, id)
        && !crate::drain::is_drainer(w, id)
}

/// Whether unit `id` is on no job another part gave it: its behaviour,
/// and the one queued after it, are none or one of a fighting unit's (guard,
/// area guard, attack, attack-move, move, hunt, patrol, stop; S-0215 Terms).
/// Any other (enter, capture, sabotage and the like) is a job this module
/// leaves to its part; a new kind of job is left alone too.
fn free_of_job(w: &World, id: ObjectId) -> bool {
    let Some(o) = w.obj(id) else { return false };
    // design: own AI (#434, #443): only a unit on a fighting unit's behaviour is moved; one on another part's job (a capture at the door, an enter click) keeps it
    let fighting = |b: Option<Behaviour>| {
        matches!(
            b,
            None | Some(
                Behaviour::Guard
                    | Behaviour::AreaGuard
                    | Behaviour::Attack
                    | Behaviour::AttackMove
                    | Behaviour::Move
                    | Behaviour::Hunt
                    | Behaviour::Patrol
                    | Behaviour::Stop
            )
        )
    };
    fighting(o.beh.current) && fighting(o.beh.queued)
}

/// The seen enemies in the air shooting a unit of the house: each with
/// its victim, in the order of the seen enemies. The enemy is airborne
/// (S-0308 Terms), aims at the unit and fired within [`RECENT`] ticks
/// (S-0206 claim 9: the tick of its last shot), the shots a person sees.
fn air_attacks(script: &Script, w: &World) -> Vec<(ObjectId, ObjectId)> {
    let house = script.house;
    script
        .seen_enemies(w)
        .into_iter()
        .filter_map(|(a, _)| {
            let o = w.obj(a)?;
            let Some(Target::Object(x)) = o.target else {
                return None;
            };
            // spec: S-0308 Terms (airborne) and S-0206 claim 9 (the tick of the last shot)
            let shooting = weapon::airborne(w, a) && w.tick.saturating_sub(o.last_fired) <= RECENT;
            let victim = w.obj(x).is_some_and(|v| v.owner == Some(house));
            (shooting && victim).then_some((a, x))
        })
        .collect()
}

/// The nearest cell to `goal`, ring by ring up to [`SEARCH_RINGS`], that
/// unit `id` can drive to (usable and in its reach region, S-0212 claim 4)
/// and that `ok` accepts; the first in cell order on equal ring.
fn drivable_near(
    w: &World,
    id: ObjectId,
    goal: CellXY,
    ok: impl Fn(CellXY) -> bool,
) -> Option<CellXY> {
    let o = w.obj(id)?;
    let zone = w.rules().unit(o.kind, o.type_index).map(|u| u.zone);
    let region = zone.map(|z| (z, regions::reach_in(w, o.pos.cell(), z)));
    // spec: S-0212 claim 4 (a unit drives only within the reach region of its movement zone)
    let drives = |c: CellXY| match region {
        Some((z, Reach::Region(r))) => regions::reach_in(w, c, z) == Reach::Region(r),
        _ => true,
    };
    let fits = |c: CellXY| w.cells.usable(c) && drives(c) && ok(c);
    if fits(goal) {
        return Some(goal);
    }
    for ring in 1..=SEARCH_RINGS {
        let mut best: Option<CellXY> = None;
        for dy in -ring..=ring {
            for dx in -ring..=ring {
                if dx.abs().max(dy.abs()) != ring {
                    continue;
                }
                let c = CellXY::new(goal.x + dx, goal.y + dy);
                if fits(c)
                    && best.is_none_or(|b| (dist2(c, goal), c.y, c.x) < (dist2(b, goal), b.y, b.x))
                {
                    best = Some(c);
                }
            }
        }
        if best.is_some() {
            return best;
        }
    }
    None
}

/// The weapon slots of enemy `a` that hold a weapon (S-0308 Terms and
/// claim 2): what a person knows the attacker's type carries.
fn armed_slots(w: &World, a: ObjectId) -> impl Iterator<Item = (usize, WeaponId)> + '_ {
    (0..SLOTS).filter_map(move |s| weapon::slot_weapon(w, a, s).map(|wp| (s, wp)))
}

/// Whether one of enemy `a`'s weapons reaches unit `x` where it stands
/// (S-0307 claims 1 to 11, the range test).
fn reaches(w: &World, a: ObjectId, x: ObjectId) -> bool {
    let t = Some(Target::Object(x));
    // spec: S-0307 claims 1 to 11 (the range test of each of the attacker's weapons)
    armed_slots(w, a).any(|(_, wp)| range::in_range(w, a, Some(wp), t, None))
}

/// The reach of enemy `a`'s weapons, in whole cells rounded up: the
/// longest range of its weapons (S-0307 claim 13) over a cell's 256
/// leptons (S-0204 claim 1). A person reads the ranges off the attacker's
/// type and does not know which weapon it fires next. `None` for an
/// unlimited range (S-0307 claim 2), which no cell escapes, and for no
/// weapon.
fn reach_cells(w: &World, a: ObjectId) -> Option<i64> {
    let mut most: Option<i64> = None;
    for (s, _) in armed_slots(w, a) {
        // spec: S-0307 claim 13 (the firer's weapon range, leptons)
        let r = range::firer_weapon_range(w, a, s);
        // spec: S-0307 claim 2 (the unlimited-range sentinel)
        if r == range::UNLIMITED {
            return None;
        }
        most = Some(most.map_or(i64::from(r), |m| m.max(i64::from(r))));
    }
    // spec: S-0204 claim 1 (a cell edge is 256 leptons)
    let lepton = 256i64;
    most.map(|r| (r.max(0) + lepton - 1) / lepton)
}

/// Where unit `x`, shot by enemy `a` in the air, goes, if anywhere: the
/// cover, home or out-of-reach goal of the module's rules
/// ([`Special::air_attack`]). `cover` is the nearest object of the house
/// that can hit the attacker within [`COVER`] cells, if any.
fn escape_goal(
    w: &World,
    home: Option<CellXY>,
    x: ObjectId,
    a: ObjectId,
    cover: Option<CellXY>,
) -> Option<CellXY> {
    let at = w.obj(x)?.pos.cell();
    let ac = w.obj(a)?.pos.cell();
    // design: own AI (#434): cover first (the house's own anti-air answers the attacker), then home from the field, then out of reach; a unit under its cover stays
    if let Some(c) = cover {
        if dist2(at, c) <= COVER_NEAR * COVER_NEAR {
            return None;
        }
        return drivable_near(w, x, c, |_| true);
    }
    if let Some(h) = home
        && dist2(at, h) > RECALL * RECALL
    {
        return drivable_near(w, x, h, |_| true);
    }
    let safe = reach_cells(w, a)? + MARGIN;
    let out = |c: CellXY| dist2(c, ac) > safe * safe;
    // The eight ways out from the attacker, at the safe distance; the one
    // nearest the unit is the shortest way out of reach.
    let s = i32::try_from(safe + 1).unwrap_or(i32::MAX / 2);
    WAYS.iter()
        .filter_map(|&(dx, dy)| {
            let g = CellXY::new(ac.x + dx * s, ac.y + dy * s);
            drivable_near(w, x, g, out)
        })
        .min_by_key(|&c| (dist2(c, at), c.y, c.x))
}

impl Special {
    /// Units under air attack (#434): each army unit of the house in `own`
    /// that a seen enemy in the air shoots, that cannot hit it and that the
    /// attacker's weapon reaches (S-0307), goes to cover, home or out of
    /// reach ([`escape_goal`]); and up to [`HELPERS`] idle army units near
    /// it that can hit the attacker are sent at it.
    pub(crate) fn air_attack(
        &mut self,
        script: &Script,
        own: &[ObjectId],
        w: &World,
        out: &mut Vec<Order>,
    ) {
        self.air.prune(w);
        let attacks = air_attacks(script, w);
        if attacks.is_empty() {
            return;
        }
        let army: std::collections::BTreeSet<ObjectId> =
            script.of_role(w, Role::Army).into_iter().collect();
        let home = script.home(w);
        let drainer_answers = crate::antiair::drainer_answers(script, w);
        let mut ordered: std::collections::BTreeSet<ObjectId> = std::collections::BTreeSet::new();
        for (a, x) in attacks {
            if ordered.contains(&x) || !own.contains(&x) || !army.contains(&x) {
                continue;
            }
            // design: own AI (#434): one on its way already is not clicked again; (#434, #443) one on another part's job keeps it
            if !movable(w, x) || !free_of_job(w, x) || self.air.sent(w, x) {
                continue;
            }
            // design: own AI (#434, #443): a unit that can hit its attacker fights, and steps aside from a slow one
            if answers(w, x, a) {
                ordered.insert(x);
                self.air_step(w, x, a, &drainer_answers, out);
                continue;
            }
            let (Some(xc), Some(ac)) = (
                w.obj(x).map(|o| o.pos.cell()),
                w.obj(a).map(|o| o.pos.cell()),
            ) else {
                continue;
            };
            // design: own AI (#434): out of the attacker's reach, the unit is safe where it stands
            if !reaches(w, a, x) {
                continue;
            }
            // The house's objects that can hit the attacker, nearest the unit first.
            let mut covers: Vec<(i64, ObjectId, CellXY)> = own
                .iter()
                .chain(army.iter())
                .copied()
                .filter(|&k| k != x && !self.air.sent(w, k))
                .filter_map(|k| {
                    let o = w.obj(k)?;
                    let c = o.pos.cell();
                    let d = dist2(c, xc);
                    if d > COVER * COVER || !o.on_map {
                        return None;
                    }
                    let can = if o.kind == Kind::Building {
                        defence_answers(w, k, a)
                    } else {
                        army.contains(&k) && answers(w, k, a)
                    };
                    can.then_some((d, k, c))
                })
                .collect();
            covers.sort_unstable();
            covers.dedup();
            let cover = covers.first().map(|&(_, _, c)| c);
            if let Some(goal) = escape_goal(w, home, x, a, cover)
                && dist2(goal, xc) > ARRIVED * ARRIVED
                && self.memory.ok(w, x, Target::Cell(goal))
                && !self.memory.pending(w, x, Target::Cell(goal))
            {
                out.push(Order::Behaviour {
                    actor: x,
                    behaviour: Behaviour::Move,
                    attack_move: false,
                    target: None,
                    dest: Some(Target::Cell(goal)),
                });
                self.offer_use(Use::AirEscape, out);
                self.air.away.insert(
                    x,
                    Away {
                        goal,
                        at: w.tick,
                        from: a,
                    },
                );
                ordered.insert(x);
            }
            self.air_helpers(w, own, &army, &drainer_answers, x, a, ac, &mut ordered, out);
        }
    }

    /// A unit that can hit its air attacker (#443): when the attacker was
    /// over it ([`OVERHEAD`] cells) at the last think and is again now (a
    /// slow attacker, which keeps over it), the unit fights it (holds it as
    /// its target and fired within [`RECENT`] ticks), and it did not step
    /// within [`STEP_GAP`] ticks, it steps [`STEP`] cells aside, to the cell
    /// of its reach region (S-0212 claim 4) farthest from the attacker. No
    /// attack order follows: retaliation (S-0215 claim 18, in guard with no
    /// target and the weapon reaching the attacker) brings it back to
    /// firing, as `tests/air_step.rs` checks against a fresh attack order.
    /// A unit that answers another seen drainer in the air (`antiair`) is
    /// left to that part.
    fn air_step(
        &mut self,
        w: &World,
        x: ObjectId,
        a: ObjectId,
        drainer_answers: &[(ObjectId, ObjectId)],
        out: &mut Vec<Order>,
    ) {
        let (Some(xo), Some(ao)) = (w.obj(x), w.obj(a)) else {
            return;
        };
        let (xc, ac) = (xo.pos.cell(), ao.pos.cell());
        let t = w.tick;
        let before = self.air.over.get(&x).copied();
        if dist2(xc, ac) > OVERHEAD * OVERHEAD {
            self.air.over.remove(&x);
            return;
        }
        self.air.over.insert(x, (a, t));
        // design: own AI (#443): slow is an attacker still over the unit one think later
        let slow = before.is_some_and(|(b, at)| b == a && at < t);
        // spec: S-0206 claim 9 (the tick of the unit's last shot)
        let fighting =
            xo.target == Some(Target::Object(a)) && t.saturating_sub(xo.last_fired) <= RECENT;
        let rested = self
            .air
            .stepped
            .get(&x)
            .is_none_or(|&(_, at)| t.saturating_sub(at) >= STEP_GAP);
        let kept = drainer_answers.iter().any(|&(d, n)| n == x && d != a);
        if !slow || !fighting || !rested || kept {
            return;
        }
        // design: own AI (#443): the step goes away from the attacker, so it has to move to stay over the unit
        let away = |c: CellXY| dist2(c, ac) > dist2(xc, ac);
        let goal = WAYS
            .iter()
            .filter_map(|&(dx, dy)| {
                let g = CellXY::new(xc.x + dx * STEP, xc.y + dy * STEP);
                drivable_near(w, x, g, away)
            })
            .filter(|&c| dist2(c, xc) > ARRIVED * ARRIVED)
            .max_by_key(|&c| (dist2(c, ac), std::cmp::Reverse((c.y, c.x))));
        let Some(goal) = goal else { return };
        if !self.memory.ok(w, x, Target::Cell(goal))
            || self.memory.pending(w, x, Target::Cell(goal))
        {
            return;
        }
        out.push(Order::Behaviour {
            actor: x,
            behaviour: Behaviour::Move,
            attack_move: false,
            target: None,
            dest: Some(Target::Cell(goal)),
        });
        self.offer_use(Use::AirStep, out);
        self.air.stepped.insert(x, (goal, t));
    }

    /// Sends up to [`HELPERS`] idle army units of the house within
    /// [`HELP_REACH`] cells of unit `x` that can hit its attacker `a` (at
    /// cell `ac`) at the attacker, counting those already on it; none that
    /// answers another seen drainer in the air (`drainer_answers`), none on
    /// its way away from an air attacker and none ordered in this think.
    #[allow(clippy::too_many_arguments)]
    fn air_helpers(
        &mut self,
        w: &World,
        own: &[ObjectId],
        army: &std::collections::BTreeSet<ObjectId>,
        drainer_answers: &[(ObjectId, ObjectId)],
        x: ObjectId,
        a: ObjectId,
        ac: CellXY,
        ordered: &mut std::collections::BTreeSet<ObjectId>,
        out: &mut Vec<Order>,
    ) {
        let Some(xc) = w.obj(x).map(|o| o.pos.cell()) else {
            return;
        };
        let goal = Target::Object(a);
        let on_it = army
            .iter()
            .filter(|&&k| w.obj(k).is_some_and(|o| o.target == Some(goal)))
            .count();
        // design: own AI (#434): a few idle units near the victim that can hit the attacker are sent at it; the drainers' answers stay with `antiair`
        let mut want = HELPERS.saturating_sub(on_it);
        if want == 0 {
            return;
        }
        let mut near: Vec<(i64, ObjectId)> = own
            .iter()
            .copied()
            .filter(|&k| {
                k != x
                    && army.contains(&k)
                    && !ordered.contains(&k)
                    && !drainer_answers.iter().any(|&(d, n)| n == k && d != a)
                    && !self.air.sent(w, k)
            })
            .filter_map(|k| {
                let c = w.obj(k)?.pos.cell();
                let d = dist2(c, xc);
                (d <= HELP_REACH * HELP_REACH).then_some((dist2(c, ac), k))
            })
            .collect();
        near.sort_unstable();
        for (_, k) in near {
            if want == 0 {
                break;
            }
            if !scripted::idle(w, k)
                || !targeting::selectable(w, k)
                || !answers(w, k, a)
                || !self.memory.ok(w, k, goal)
                || self.memory.pending(w, k, goal)
            {
                continue;
            }
            out.push(Order::Behaviour {
                actor: k,
                behaviour: Behaviour::Attack,
                attack_move: false,
                target: Some(goal),
                dest: None,
            });
            self.offer_use(Use::AirHelp, out);
            ordered.insert(k);
            want -= 1;
        }
    }
}
