//! What the own AI remembers of the orders it gave (M6c; PLAN §4.7): a
//! person who sends a unit somewhere and sees it stand still does not click
//! the same thing again every second; they try another target, or give the
//! unit up until something changes. The brain does the same.
//!
//! Each order a special-ability module gives to a unit is noted with the
//! cell the unit stood on. At each think the notes are judged: a unit that
//! is idle again on the same cell without having fired did not take the
//! order (a path that does not exist, a target it cannot attack, a building
//! it cannot enter), so the pair of unit, goal and cell is blocked for a
//! while. A module asks [`Memory::ok`] before it picks a goal for a unit. A
//! unit that moved, died, entered something or fired leaves no memory; one
//! that is still busy on the same cell without having fired is judged again
//! at the next think (a unit that holds an attack order for a while and
//! then drops it where it stood did not take it either), for at most
//! [`BLOCKED_FOR`] ticks, and a new order for the unit replaces the note.
//!
//! It reads only what a person sees (the unit's own cell and state) and is
//! kept with the brain (saved and hashed with it).

use glasnost_map::CellXY;
use glasnost_sim::orders::Order;
use glasnost_sim::world::{ObjectId, Target, World};
use serde::{Deserialize, Serialize};

use crate::scripted;

/// How long, in ticks, a failed order stays remembered: a unit that stands
/// on the same cell tries the same goal again after this (the world may
/// have changed), not every think. About five minutes of game time.
const BLOCKED_FOR: i32 = 4500; // nocite: tuning of our own opponent

/// The most notes of either kind the memory keeps (the oldest go first).
const LIMIT: usize = 256; // nocite: tuning of our own opponent

/// An order given at some tick, to be judged at the next think.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Given {
    unit: ObjectId,
    goal: Target,
    at: CellXY,
    /// The tick of the order.
    since: i32,
    /// The unit's rearm tick when it was given the order: another value
    /// later means it fired.
    rearm: i32,
    /// The goal the module asked about when the click gave another one
    /// ([`Memory::ask`]): a failure counts for it too.
    #[serde(default)]
    asked: Option<Target>,
}

/// A failed order: the unit did not leave its cell for the goal.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Failed {
    unit: ObjectId,
    goal: Target,
    at: CellXY,
    until: i32,
}

/// The orders given and the ones that failed.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Memory {
    given: Vec<Given>,
    failed: Vec<Failed>,
    /// The behaviour orders the parts offered in the current think, noted
    /// only once the brain knows which it gives ([`Memory::note_given`]);
    /// empty between thinks (never saved).
    #[serde(skip)]
    offered: Vec<Order>,
    /// The goals asked about for offered orders that go elsewhere
    /// ([`Memory::ask`]); empty between thinks (never saved).
    #[serde(skip)]
    asked: Vec<(Order, Target)>,
}

impl Memory {
    /// Judges the orders of the last think and forgets failures that are
    /// old or whose unit is gone.
    pub fn review(&mut self, w: &World) {
        for g in std::mem::take(&mut self.given) {
            let standing = w.obj(g.unit).is_some_and(|o| {
                o.on_map && o.health > 0 && o.pos.cell() == g.at && o.combat.rearm_until == g.rearm
            });
            if !standing {
                continue;
            }
            if !scripted::idle(w, g.unit) {
                // Still busy where it stood: judged again at the next think.
                if w.tick - g.since < BLOCKED_FOR {
                    self.given.push(g);
                }
                continue;
            }
            let until = w.tick + BLOCKED_FOR;
            for goal in std::iter::once(g.goal).chain(g.asked.filter(|&a| a != g.goal)) {
                match self
                    .failed
                    .iter_mut()
                    .find(|f| f.unit == g.unit && f.goal == goal && f.at == g.at)
                {
                    Some(f) => f.until = until,
                    None => self.failed.push(Failed {
                        unit: g.unit,
                        goal,
                        at: g.at,
                        until,
                    }),
                }
            }
        }
        self.failed
            .retain(|f| f.until > w.tick && w.obj(f.unit).is_some());
        if self.failed.len() > LIMIT {
            let drop = self.failed.len() - LIMIT;
            self.failed.drain(..drop);
        }
    }

    /// Offers the behaviour orders among `orders` (the ones a module just
    /// gave) for noting: [`Memory::note_given`] notes those the brain
    /// gives in the end.
    pub fn offer(&mut self, orders: &[Order]) {
        self.offered.extend(
            orders
                .iter()
                .filter(|o| matches!(o, Order::Behaviour { .. }))
                .cloned(),
        );
    }

    /// Tells the memory that the behaviour orders among `orders` answer a
    /// click on `asked`, whatever goal they carry (a click on a cell out of
    /// the unit's reach moves it to a cell the search finds instead, S-0214
    /// claim 11): when such an order fails, `asked` is failed too, so the
    /// module that asks [`Memory::ok`] about `asked` does not click it
    /// again.
    // design: own AI (#23): the grinder's side click was judged under the cell the search found, which may differ at every think, so the same side was clicked again and again
    pub fn ask(&mut self, orders: &[Order], asked: Target) {
        self.asked.extend(
            orders
                .iter()
                .filter(|o| matches!(o, Order::Behaviour { .. }))
                .map(|o| (o.clone(), asked)),
        );
    }

    /// Notes the offered orders the brain gives: those that are their
    /// actor's last behaviour order among the think's final orders `out`
    /// (a later filter may drop one, or a later part give the actor
    /// another) and come before position `room`, the orders the house's
    /// pending list still takes (S-0213 claim 2: an order issued while the
    /// list is full is dropped). The rest were never given, and are not
    /// judged as orders the unit did not take.
    // design: own AI (#19): every offered order was noted, also those a filter dropped or the full pending list refused, and its unit was given up for BLOCKED_FOR ticks
    pub fn note_given(&mut self, w: &World, out: &[Order], room: usize) {
        let offered = std::mem::take(&mut self.offered);
        let asked = std::mem::take(&mut self.asked);
        if offered.is_empty() {
            return;
        }
        let mut last: std::collections::BTreeMap<ObjectId, usize> =
            std::collections::BTreeMap::new();
        for (i, o) in out.iter().enumerate() {
            if let Order::Behaviour { actor, .. } = o {
                last.insert(*actor, i);
            }
        }
        let given: Vec<Order> = offered
            .into_iter()
            .filter(|o| {
                let Order::Behaviour { actor, .. } = o else {
                    return false;
                };
                last.get(actor)
                    .is_some_and(|&i| i < room && out.get(i) == Some(o))
            })
            .collect();
        self.note(w, &given, &asked);
    }

    /// Notes the behaviour orders among `orders`, to be judged at the next
    /// think, each with the goal asked about for it in `asked`, if any.
    fn note(&mut self, w: &World, orders: &[Order], asked: &[(Order, Target)]) {
        for o in orders {
            let asked_for = asked.iter().find(|(x, _)| x == o).map(|&(_, t)| t);
            let Order::Behaviour {
                actor,
                target,
                dest,
                ..
            } = o
            else {
                continue;
            };
            let Some(goal) = (*target).or(*dest) else {
                continue;
            };
            let Some((at, rearm)) = w.obj(*actor).map(|u| (u.pos.cell(), u.combat.rearm_until))
            else {
                continue;
            };
            // A new order replaces what the unit was given before.
            self.given.retain(|g| g.unit != *actor);
            self.given.push(Given {
                unit: *actor,
                goal,
                at,
                since: w.tick,
                rearm,
                asked: asked_for,
            });
        }
        if self.given.len() > LIMIT {
            let drop = self.given.len() - LIMIT;
            self.given.drain(..drop);
        }
    }

    /// Whether a module may send `unit` for `goal`: it did not already fail
    /// to from the cell the unit stands on.
    pub fn ok(&self, w: &World, unit: ObjectId, goal: Target) -> bool {
        let Some(at) = w.obj(unit).map(|o| o.pos.cell()) else {
            return true;
        };
        !self
            .failed
            .iter()
            .any(|f| f.unit == unit && f.goal == goal && f.at == at && f.until > w.tick)
    }

    /// Whether `unit` was already sent for `goal` from the cell it stands
    /// on and its order is still being judged (it has not moved): a
    /// module that would click the same again waits instead.
    pub fn pending(&self, w: &World, unit: ObjectId, goal: Target) -> bool {
        let Some(at) = w.obj(unit).map(|o| o.pos.cell()) else {
            return false;
        };
        self.given
            .iter()
            .any(|g| g.unit == unit && g.goal == goal && g.at == at)
    }

    /// The goal `unit` was sent for from the cell it stands on, while that
    /// order is still being judged (it has not moved since): what a person
    /// who just clicked sees the unit still holding.
    pub fn sent_here(&self, w: &World, unit: ObjectId) -> Option<Target> {
        let at = w.obj(unit).map(|o| o.pos.cell())?;
        self.given
            .iter()
            .find(|g| g.unit == unit && g.at == at)
            .map(|g| g.goal)
    }

    /// How many (unit, goal) pairs are blocked now (diagnostics).
    pub fn blocked(&self) -> usize {
        self.failed.len()
    }
}
