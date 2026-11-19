//! The own AI and a jumpjet that holds over its target without firing
//! (design: own AI, #445; PLAN §4.7).
//!
//! A balloon-hover vehicle whose weapon drops a vertical projectile takes
//! its target as its destination (S-0307 claim 18) and flies to the
//! target's point, for a building its reference point (S-0502 r34 claim
//! 16). Whether it then fires depends on its type and weapon, not on the
//! order: a type without the mobile-fire flag gets *moving* while the
//! target stays its destination, so it never fires at it (S-0310 claim 7);
//! with a weapon without the fire-while-moving flag it fires from over the
//! point only while its kept facing lies within the aiming tolerance of
//! map east, as after an approach flown towards map east (S-0502 claim 10,
//! Q-3257; S-0310 claim 9 gives it no re-aim there). A person sees such a
//! unit hang over its target with no bombs falling, and moves it on. The
//! brain does the same ([`Special::bombers`]):
//! - it watches each jumpjet of the house in the air in *attack* on an
//!   object; one that has fired no shot for [`STALL`] ticks from the think
//!   it was first seen within [`NEAR`] cells of its target's point, and is
//!   there still, holds over its target without firing (a shot or another
//!   target restarts the count; hovering is not asked, so one that circles
//!   about the point counts too);
//! - the first time it holds so over a target, it is sent [`ROUND`] cells
//!   west of the target's point (fewer, down to [`ROUND_MIN`], where the
//!   map ends sooner) and, once it hovers there or after [`ROUND_TIME`]
//!   ticks, at the target again: the new approach is flown towards map
//!   east;
//! - when it holds so over the same target again, the house gives up: the
//!   unit is sent home and kept out of attacks and of the waves'
//!   attack-moves for [`BENCH`] ticks ([`Watch::keeps_order`]). A unit
//!   that cannot fire at what it is sent at is of no use in a wave.
//!
//! It tells no cause from another: a unit out of ammunition, or one whose
//! target its weapon cannot reach from its hover, is handled the same way.
//! The moves are plain moves, not attack-moves, so the unit does not stop
//! on the way for the first enemy it meets.
//!
//! It reads only the house's own units (their behaviour, target,
//! destination, flight state and the tick of their last shot, which a
//! person sees as the bombs falling or not) and the targets' points, and
//! it draws no random value. The counters
//! [`crate::special::Uses::bomber_rounds`] (the move round and the attack
//! after it) and [`crate::special::Uses::bomber_retreats`] (the moves home)
//! count its orders.

use std::collections::BTreeMap;

use glasnost_map::CellXY;
use glasnost_sim::jump::{self, Phase};
use glasnost_sim::orders::Order;
use glasnost_sim::rules::Behaviour;
use glasnost_sim::targeting::Action;
use glasnost_sim::world::{ObjectId, Target, World};
use serde::{Deserialize, Serialize};

use crate::scripted::{self, Script, dist2};
use crate::special::{Special, Use, click_if};

/// Ticks a unit holds over its target without a shot before it is moved
/// on.
pub const STALL: i32 = 450; // nocite: tuning of our own opponent

/// Cells west of the target's point a stalled unit is sent for its new
/// approach.
pub const ROUND: i32 = 5; // nocite: tuning of our own opponent

/// The fewest cells west of the target's point that still make a new
/// approach.
pub const ROUND_MIN: i32 = 3; // nocite: tuning of our own opponent

/// The longest a unit sent round is kept for its move.
pub const ROUND_TIME: i32 = 450; // nocite: tuning of our own opponent

/// Cells within which a unit sent round has arrived.
const ARRIVED: i64 = 1; // nocite: tuning of our own opponent

/// Ticks a unit sent home is kept out of attacks.
pub const BENCH: i32 = 9000; // nocite: tuning of our own opponent

/// A unit seen holding over its target: the target, the think it was first
/// seen so, and the tick of its last shot then.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Hold {
    target: ObjectId,
    since: i32,
    shot: i32,
}

/// Where a stalled unit's job stands.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
enum Step {
    /// On its way to the cell of its new approach.
    Round(CellXY),
    /// Sent at its target again.
    Again,
    /// Sent home, kept out of attacks.
    Home(CellXY),
}

/// A stalled unit's job: its target, the step and the tick of its order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Job {
    target: ObjectId,
    step: Step,
    at: i32,
}

/// What the module remembers between thinks.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Watch {
    /// The units seen holding over their target.
    held: BTreeMap<ObjectId, Hold>,
    /// The stalled units' jobs.
    jobs: BTreeMap<ObjectId, Job>,
}

/// Cells from its target's point within which a unit is over its target.
pub const NEAR: i64 = 2; // nocite: tuning of our own opponent

/// The object target of unit `id` and the tick of its last shot, when it
/// is a jumpjet in the air (S-0502 Terms: not *grounded*) in *attack* on
/// an object.
fn attacking(w: &World, id: ObjectId) -> Option<(ObjectId, i32)> {
    let o = w.obj(id)?;
    if matches!(jump::phase(w, id), None | Some(Phase::Grounded))
        || o.beh.current != Some(Behaviour::Attack)
    {
        return None;
    }
    match o.target {
        Some(Target::Object(t)) if w.obj(t).is_some() => Some((t, o.last_fired)),
        _ => None,
    }
}

/// Whether unit `id` is over target `t`: within [`NEAR`] cells of its
/// point, the point it flies to (S-0502 r34 claim 16: for a building its
/// reference point). Hovering is not asked: a unit that circles about the
/// point without a shot (neighbour steps while another unit holds its
/// hover slot, S-0502 claim 12) holds there just the same for a person who
/// watches it.
fn over(w: &World, id: ObjectId, t: ObjectId) -> bool {
    // spec: S-0502 r34 claim 16 (the movement point for an object destination is the object's point; for a building its reference point)
    w.obj(id).is_some_and(|o| {
        w.obj(t).is_some() && dist2(o.pos.cell(), w.reference_point(t).cell()) <= NEAR * NEAR
    })
}

/// Whether unit `id` is on its way to `c`: in a move there, and not yet
/// hovering there (S-0502 Terms).
fn on_way(w: &World, id: ObjectId, c: CellXY) -> bool {
    w.obj(id).is_some_and(|o| {
        o.beh.current == Some(Behaviour::Move)
            && o.dest == Some(Target::Cell(c))
            && !(jump::phase(w, id) == Some(Phase::Hovering)
                && dist2(o.pos.cell(), c) <= ARRIVED * ARRIVED)
    })
}

/// An attack order of `id` on `t`.
fn attack(id: ObjectId, t: ObjectId) -> Order {
    Order::Behaviour {
        actor: id,
        behaviour: Behaviour::Attack,
        attack_move: false,
        target: Some(Target::Object(t)),
        dest: None,
    }
}

/// The cell of the new approach on target `t`: [`ROUND`] cells west of
/// its point, or fewer down to [`ROUND_MIN`], on a usable cell.
fn round_cell(w: &World, t: ObjectId) -> Option<CellXY> {
    w.obj(t)?;
    // spec: S-0502 r34 claim 16 (the movement point for a building is its reference point)
    let p = w.reference_point(t).cell();
    // spec: S-0502 claim 10 (Q-3257: over the point the target lies straight below, a direction of map east; an approach flown towards map east leaves the body on it)
    (ROUND_MIN..=ROUND)
        .rev()
        .map(|d| CellXY::new(p.x - d, p.y))
        .find(|&c| w.cells.usable(c))
}

/// A plain move of `id` to `c`.
fn move_to(id: ObjectId, c: CellXY) -> Order {
    Order::Behaviour {
        actor: id,
        behaviour: Behaviour::Move,
        attack_move: false,
        target: None,
        dest: Some(Target::Cell(c)),
    }
}

impl Watch {
    /// Forgets the units gone, the jobs on targets gone (but not a unit
    /// still kept home) and the jobs kept home longer than [`BENCH`].
    fn prune(&mut self, w: &World) {
        let t = w.tick;
        self.held.retain(|&id, _| w.obj(id).is_some());
        self.jobs.retain(|&id, j| {
            w.obj(id).is_some()
                && match j.step {
                    Step::Home(_) => t.saturating_sub(j.at) <= BENCH,
                    _ => w.obj(j.target).is_some(),
                }
        });
    }

    /// The brain's filter of its think's orders `out`: drops each order
    /// [`Watch::keeps_order`] refuses, and the repeats of an order to a
    /// unit with a job (the attack after the move round is the same order
    /// the army part gives an idle unit near an enemy: one click, not two).
    pub(crate) fn filter_orders(&self, w: &World, out: &mut Vec<Order>) {
        out.retain(|o| self.keeps_order(w, o));
        let mut seen: Vec<Order> = Vec::new();
        out.retain(|o| {
            if !o.actor().is_some_and(|a| self.jobs.contains_key(&a)) {
                return true;
            }
            if seen.contains(o) {
                return false;
            }
            seen.push(o.clone());
            true
        });
    }

    /// Whether the brain keeps order `o`: a unit on its way round takes
    /// only that move; a unit kept home takes no attack and no
    /// attack-move (a person who pulled it back does not send it out
    /// again with the next wave).
    fn keeps_order(&self, w: &World, o: &Order) -> bool {
        let Some(a) = o.actor() else { return true };
        let Some(j) = self.jobs.get(&a) else {
            return true;
        };
        let age = w.tick.saturating_sub(j.at);
        match j.step {
            Step::Round(c) if age <= scripted::THINK_PERIOD || on_way(w, a, c) => {
                *o == move_to(a, c)
            }
            Step::Home(_) if age <= BENCH => !matches!(
                o,
                Order::Behaviour {
                    behaviour: Behaviour::Attack,
                    ..
                } | Order::Behaviour {
                    attack_move: true,
                    ..
                }
            ),
            _ => true,
        }
    }
}

impl Special {
    /// The house's jumpjets that hold over their target without firing
    /// (module docs): sent round once, then home.
    pub(crate) fn bombers(
        &mut self,
        script: &Script,
        own: &[ObjectId],
        w: &World,
        out: &mut Vec<Order>,
    ) {
        let house = script.house;
        let tick = w.tick;
        self.bomber.prune(w);
        for &id in own {
            if !jump::is_jumpjet(w, id) {
                continue;
            }
            // A unit on its way round goes in again once there.
            if let Some(j) = self.bomber.jobs.get(&id).copied()
                && let Step::Round(c) = j.step
            {
                let age = tick.saturating_sub(j.at);
                if age <= scripted::THINK_PERIOD || (age <= ROUND_TIME && on_way(w, id, c)) {
                    continue;
                }
                // design: own AI (#445): the same attack the unit took before, given only while the target is seen and its cursor over it still shows attack
                if crate::in_view(w, house, j.target)
                    && click_if(w, house, id, j.target, &[Action::Attack]).is_some()
                {
                    out.push(attack(id, j.target));
                    self.offer_use(Use::BomberRound, out);
                    self.bomber.jobs.insert(
                        id,
                        Job {
                            step: Step::Again,
                            at: tick,
                            ..j
                        },
                    );
                } else {
                    self.bomber.jobs.remove(&id);
                }
                continue;
            }
            let Some((t, shot)) = attacking(w, id) else {
                self.bomber.held.remove(&id);
                continue;
            };
            // design: own AI (#445): the count starts at the first think the unit is seen over its target, and runs while it attacks that target without a shot, wherever it circles
            let h = match self.bomber.held.get(&id) {
                Some(h) if h.target == t && h.shot == shot => *h,
                _ => {
                    if over(w, id, t) {
                        self.bomber.held.insert(
                            id,
                            Hold {
                                target: t,
                                since: tick,
                                shot,
                            },
                        );
                    } else {
                        self.bomber.held.remove(&id);
                    }
                    continue;
                }
            };
            if tick.saturating_sub(h.since) < STALL || !over(w, id, t) {
                continue;
            }
            self.bomber.held.remove(&id);
            // design: own AI (#445): one new approach per target; a unit kept home that holds again goes home again
            let again = self.bomber.jobs.get(&id).is_some_and(|j| {
                (j.target == t && j.step == Step::Again) || matches!(j.step, Step::Home(_))
            });
            let round = if again { None } else { round_cell(w, t) };
            if let Some(c) = round {
                out.push(move_to(id, c));
                self.offer_use(Use::BomberRound, out);
                self.bomber.jobs.insert(
                    id,
                    Job {
                        target: t,
                        step: Step::Round(c),
                        at: tick,
                    },
                );
            } else if let Some(c) = script.home(w) {
                out.push(move_to(id, c));
                self.offer_use(Use::BomberRetreat, out);
                self.bomber.jobs.insert(
                    id,
                    Job {
                        target: t,
                        step: Step::Home(c),
                        at: tick,
                    },
                );
            }
        }
    }
}
