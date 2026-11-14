//! The update order and the turn pass (S-0203 claims 2 to 4, 10 and 11).

use serde::{Deserialize, Serialize};

use crate::world::ObjectId;

/// The sequence in which objects take their turns, and the state of a turn
/// pass in progress.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateOrder {
    /// Objects in play, in the order of entering play (S-0203 claim 3).
    pub order: Vec<ObjectId>,
    /// Index of the current object during a turn pass.
    cur: Option<usize>,
    /// The current object left play during its turn.
    cur_gone: bool,
    /// Counting removals during the current turn (S-0203 claim 10).
    skips: usize,
    /// A turn pass is running.
    in_pass: bool,
}

impl UpdateOrder {
    /// Whether an object is in play.
    pub fn contains(&self, id: ObjectId) -> bool {
        self.order.contains(&id)
    }

    /// An object enters play: it goes to the end of the order (S-0203
    /// claim 3), and takes a turn later in a running pass (claim 4).
    pub fn enter(&mut self, id: ObjectId) {
        // spec: S-0203 claim 3 (appending; an object that enters again goes to the end)
        if !self.order.contains(&id) {
            self.order.push(id);
        }
    }

    /// An object with a number never used before enters play: as
    /// [`UpdateOrder::enter`], without the search for an earlier entry.
    pub fn enter_new(&mut self, id: ObjectId) {
        debug_assert!(!self.order.contains(&id));
        // spec: S-0203 claim 3 (appending)
        self.order.push(id);
    }

    /// An object leaves play. During a turn pass, a removal of the current
    /// object or of one before it is a counting removal: with
    /// `quirks.update_order_removal_skip` it costs the next object its turn
    /// (S-0203 claim 10). When a destroyed object leaves play is S-0203
    /// claim 6: at the moment of the damage as a rule, later for the
    /// exceptions S-0312 owns (sinking, crashing, exploding or sold
    /// buildings, passengers that get out); the combat code (M4) calls this
    /// at that moment.
    pub fn leave(&mut self, id: ObjectId, removal_skip: bool) {
        // The current object leaving in its own turn is the common case
        // (an ended animation): found without a search.
        let p = match self.cur {
            Some(c) if self.order.get(c) == Some(&id) => c,
            _ => {
                let Some(p) = self.order.iter().position(|o| *o == id) else {
                    return;
                };
                p
            }
        };
        self.order.remove(p);
        if !self.in_pass {
            // spec: S-0203 claim 10 (a removal while the turn pass is not running costs nobody a turn)
            return;
        }
        if let Some(cur) = self.cur {
            // spec: S-0203 claim 10 (a removal counts when the object is the current one or stands before it; once the current one has left, index `cur` holds the object that stood after it, whose removal costs nobody a turn)
            let current = p == cur && !self.cur_gone;
            if p < cur || current {
                if removal_skip {
                    self.skips += 1;
                }
                if current {
                    self.cur_gone = true;
                } else {
                    self.cur = Some(cur - 1);
                }
            }
        }
    }

    /// Starts a turn pass.
    pub fn begin_pass(&mut self) {
        self.in_pass = true;
        self.cur = None;
        self.cur_gone = false;
        self.skips = 0;
    }

    /// The next object to take its turn, applying the skips that the last
    /// turn's counting removals caused (S-0203 claim 10). `None` ends the
    /// pass.
    pub fn next_turn(&mut self) -> Option<ObjectId> {
        let mut i = match self.cur {
            None => 0,
            Some(c) if self.cur_gone => c,
            Some(c) => c + 1,
        };
        // spec: S-0203 claim 10 (k counting removals make the k objects directly after miss their turns)
        i += self.skips;
        self.skips = 0;
        self.cur_gone = false;
        if i >= self.order.len() {
            self.cur = Some(self.order.len());
            return None;
        }
        self.cur = Some(i);
        Some(self.order[i])
    }

    /// Ends a turn pass.
    pub fn end_pass(&mut self) {
        self.in_pass = false;
        self.cur = None;
        self.cur_gone = false;
        self.skips = 0;
    }

    /// Whether a turn pass is running.
    pub fn in_pass(&self) -> bool {
        self.in_pass
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    /// A miniature world of firers, targets and projectiles that reproduces
    /// the volleys of S-0203 scenario O8: each firer appends a projectile;
    /// a projectile detonates in its first turn, hits its target and leaves
    /// play; a lethal hit removes the target too.
    #[derive(Default)]
    struct Mini {
        order: UpdateOrder,
        /// projectile → target
        flying: BTreeMap<ObjectId, ObjectId>,
        /// firer → target
        firers: BTreeMap<ObjectId, ObjectId>,
        /// targets with hit points
        hp: BTreeMap<ObjectId, i32>,
        next: u32,
        hits: Vec<(ObjectId, u32)>,
    }

    impl Mini {
        fn tick(&mut self, t: u32, fire: bool) {
            self.order.begin_pass();
            while let Some(id) = self.order.next_turn() {
                if let Some(&target) = self.firers.get(&id) {
                    if fire {
                        self.next += 1;
                        let p = ObjectId(1000 + self.next);
                        self.flying.insert(p, target);
                        self.order.enter(p);
                    }
                } else if let Some(target) = self.flying.remove(&id) {
                    self.hits.push((target, t));
                    let hp = self.hp.entry(target).or_insert(1);
                    *hp -= 1;
                    if *hp <= 0 {
                        self.order.leave(target, true);
                    }
                    self.order.leave(id, true);
                }
            }
            self.order.end_pass();
        }

        fn hit_tick(&self, target: u32) -> Option<u32> {
            self.hits
                .iter()
                .find(|h| h.0 == ObjectId(target))
                .map(|h| h.1)
        }
    }

    fn o8(lethal: bool, firers: u32) -> Mini {
        let mut m = Mini::default();
        // Targets W1 … Wn (IDs 1 …), then firers F1 … Fn (IDs 101 …).
        for i in 1..=firers {
            m.order.enter(ObjectId(i));
            m.hp.insert(ObjectId(i), if lethal && i == 1 { 1 } else { 100 });
        }
        for i in 1..=firers {
            m.order.enter(ObjectId(100 + i));
            m.firers.insert(ObjectId(100 + i), ObjectId(i));
        }
        m.tick(10, true);
        for t in 11..15 {
            m.tick(t, false);
        }
        m
    }

    // spec: S-0203 claim 10 and scenario O8 variant (a) (non-lethal: W1 n, W2 n + 1, W3 n)
    #[test]
    fn o8_non_lethal() {
        let m = o8(false, 3);
        assert_eq!(
            (m.hit_tick(1), m.hit_tick(2), m.hit_tick(3)),
            (Some(10), Some(11), Some(10))
        );
    }

    // spec: S-0203 claim 10 and scenario O8 variant (a) (lethal: W1 n, W2 n + 1, W3 n + 2); S-0203 claim 5 (revision 40: P3, fired in tick n, loses its first turn to removals in n and again in n + 1, so its first turn is in n + 2, the first tick in which no removal costs it the turn) and S-0306 State and ordering (revision 38: n + 1 or later)
    #[test]
    fn o8_lethal() {
        let m = o8(true, 3);
        assert_eq!(
            (m.hit_tick(1), m.hit_tick(2), m.hit_tick(3)),
            (Some(10), Some(11), Some(12))
        );
    }

    // spec: S-0203 claim 10 and scenario O12 (a removal counts although the removed object missed its turn)
    #[test]
    fn o12_removed_after_missed_turn() {
        // Order: W (1), U (2), F1 (101), F2 (102), F3 (103). In tick c, W
        // leaves play in its own turn, so U misses its turn; F1's projectile
        // then destroys U (standing before it): two counting removals.
        let mut o = UpdateOrder::default();
        for id in [1, 2, 101, 102, 103] {
            o.enter(ObjectId(id));
        }
        let mut hits = Vec::new();
        let mut proj: BTreeMap<ObjectId, Option<ObjectId>> = BTreeMap::new();
        for t in 0..3u32 {
            o.begin_pass();
            while let Some(id) = o.next_turn() {
                match id.0 {
                    1 if t == 0 => o.leave(ObjectId(1), true),
                    101..=103 if t == 0 => {
                        let p = ObjectId(id.0 + 100);
                        proj.insert(p, if id.0 == 101 { Some(ObjectId(2)) } else { None });
                        o.enter(p);
                    }
                    201..=203 => {
                        hits.push((id.0, t));
                        if let Some(Some(u)) = proj.get(&id) {
                            o.leave(*u, true);
                        }
                        o.leave(id, true);
                    }
                    _ => {}
                }
            }
            o.end_pass();
        }
        // F1's projectile in tick 0; F2's in tick 1; F3's in tick 2.
        assert_eq!(hits, vec![(201, 0), (202, 1), (203, 2)]);
    }

    // spec: S-0203 claim 10 (a removal of an object that stands after the current one costs nobody a turn: after the current object has left play in its own turn, the object that stood directly after it still stands after it)
    #[test]
    fn current_leaves_then_its_follower_leaves() {
        let mut o = UpdateOrder::default();
        for id in 1..=5 {
            o.enter(ObjectId(id));
        }
        o.begin_pass();
        let mut seen = Vec::new();
        while let Some(id) = o.next_turn() {
            seen.push(id.0);
            if id.0 == 1 {
                // The current object leaves play, then the object directly after it.
                o.leave(ObjectId(1), true);
                o.leave(ObjectId(2), true);
            }
        }
        o.end_pass();
        // One counting removal (object 1): object 3 misses its turn, 4 and 5 act.
        assert_eq!(seen, vec![1, 4, 5]);
    }

    // spec: S-0203 claim 10 (the same two removals in the other order: one counting removal)
    #[test]
    fn follower_leaves_then_the_current_object_leaves() {
        let mut o = UpdateOrder::default();
        for id in 1..=5 {
            o.enter(ObjectId(id));
        }
        o.begin_pass();
        let mut seen = Vec::new();
        while let Some(id) = o.next_turn() {
            seen.push(id.0);
            if id.0 == 1 {
                o.leave(ObjectId(2), true);
                o.leave(ObjectId(1), true);
            }
        }
        o.end_pass();
        assert_eq!(seen, vec![1, 4, 5]);
    }

    // spec: S-0203 claim 10 (after the current object has left play, a removal of an object that stood before it still counts)
    #[test]
    fn current_leaves_then_an_earlier_object_leaves() {
        let mut o = UpdateOrder::default();
        for id in 1..=6 {
            o.enter(ObjectId(id));
        }
        o.begin_pass();
        let mut seen = Vec::new();
        while let Some(id) = o.next_turn() {
            seen.push(id.0);
            if id.0 == 3 {
                o.leave(ObjectId(3), true);
                o.leave(ObjectId(1), true);
            }
        }
        o.end_pass();
        // Two counting removals: objects 4 and 5 miss their turns.
        assert_eq!(seen, vec![1, 2, 3, 6]);
    }

    // spec: S-0203 Edge cases (`quirks.update_order_removal_skip` off: no removal costs a turn)
    #[test]
    fn no_skip_when_off() {
        let mut o = UpdateOrder::default();
        for id in 1..=3 {
            o.enter(ObjectId(id));
        }
        o.begin_pass();
        let mut seen = Vec::new();
        while let Some(id) = o.next_turn() {
            seen.push(id.0);
            if id.0 == 1 {
                o.leave(ObjectId(1), false);
            }
        }
        assert_eq!(seen, vec![1, 2, 3]);
    }
}
