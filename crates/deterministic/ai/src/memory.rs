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
            {
                let until = w.tick + BLOCKED_FOR;
                match self
                    .failed
                    .iter_mut()
                    .find(|f| f.unit == g.unit && f.goal == g.goal && f.at == g.at)
                {
                    Some(f) => f.until = until,
                    None => self.failed.push(Failed {
                        unit: g.unit,
                        goal: g.goal,
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

    /// Notes the behaviour orders among `orders` (the ones a module just
    /// gave), to be judged at the next think.
    pub fn note(&mut self, w: &World, orders: &[Order]) {
        for o in orders {
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

    /// How many (unit, goal) pairs are blocked now (diagnostics).
    pub fn blocked(&self) -> usize {
        self.failed.len()
    }
}
