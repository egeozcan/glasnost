//! The vehicles' walk counters (S-0102 claim 22; S-0105 claim 16), kept
//! by the view beside the game.
//!
//! S-0102 claim 22 makes the walk counter a per-unit integer that grows by
//! 1 in a tick when the vehicle is moving and the tick number is a multiple
//! of its type's walk rate, or when it stands still, its type's idle rate is
//! not 0 and the tick number is a multiple of the idle rate. It picks the
//! walking image frame of a vehicle drawn as a sprite; S-0105 claim 16's
//! walk-animation counter picks a voxel body's motion frame, and S-0402 F8
//! reads it for the harvesting image. Only drawing reads it, so the view
//! keeps it, advanced once per game tick (never per drawn frame) from the
//! state the tick left; a save, a load or a joined network game starts the
//! counters again at 0 (design: a picture's phase, nothing else).
//!
//! S-0105 Terms leave the walk-animation counter's advance to the vehicle
//! specs; S-0102 claim 22 gives the one rule there is, for sprite vehicles.
//! Design (Q-2176): every vehicle, sprite or voxel, keeps the one counter of
//! claim 22, as S-0105 scenario MS-3 counts its frames in "walk-counter
//! steps".

use std::collections::BTreeMap;

use glasnost_sim::rules::Kind as SimKind;
use glasnost_sim::world::{ObjectId, World};

/// A vehicle type's walk rate and idle rate in ticks (S-0102 Import
/// mapping: `WalkRate`, `IdleRate`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rates {
    /// The walk rate.
    pub walk: i32,
    /// The idle rate; 0 never steps a vehicle that stands still.
    pub idle: i32,
}

impl Default for Rates {
    fn default() -> Self {
        Self {
            walk: 1, // spec: S-0102 Import mapping (WalkRate absent: 1)
            idle: 0, // spec: S-0102 Import mapping (IdleRate absent: 0)
        }
    }
}

/// What the drawing reads of a vehicle's walking: its walk counter and
/// whether it was moving in the shown tick.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Walk {
    /// The walk counter *w* (S-0102 claim 22), the walk-animation counter
    /// ν of S-0105 claim 16.
    pub counter: u32,
    /// Moving in the shown tick: S-0102 claim 22's "moving" row.
    pub moving: bool,
}

/// Whether `tick` is a multiple of `rate` (exact integers; a multiple of 0
/// is 0 alone).
fn multiple(tick: i32, rate: i32) -> bool {
    if rate == 0 {
        tick == 0
    } else {
        tick.rem_euclid(rate) == 0
    }
}

/// One tick of a counter (S-0102 claim 22): the tick number `tick` (0 while
/// the first tick runs), whether the vehicle is moving, its rates.
pub fn step(counter: u32, tick: i32, moving: bool, rates: Rates) -> u32 {
    // spec: S-0102 claim 22 (Walk counter: +1 when moving and the tick number is a multiple of
    // the walk rate, or when not moving, the idle rate is not 0 and the tick number is a multiple
    // of the idle rate)
    let grows = if moving {
        multiple(tick, rates.walk)
    } else {
        rates.idle != 0 && multiple(tick, rates.idle)
    };
    if grows {
        counter.wrapping_add(1)
    } else {
        counter
    }
}

/// Whether a vehicle is moving for its walk counter: S-0107 Terms' *is
/// moving now* (a locomotor moving it, or a stored path), as the
/// simulation answers it for S-0215's readiness.
pub fn moving(w: &World, id: ObjectId) -> bool {
    // spec: S-0102 claim 22 ("moving"), read as S-0107 Terms' is moving now
    glasnost_sim::occupancy::moving_on_tick(w, id)
}

/// The walk counters of every vehicle in play.
#[derive(Clone, Debug, Default)]
pub struct Counters {
    walks: BTreeMap<ObjectId, Walk>,
}

impl Counters {
    /// Advances every vehicle's counter by the tick the world has just
    /// completed (its tick number is `w.tick − 1`), reading each type's
    /// rates from `rates` (by vehicle type index; a missing entry takes the
    /// absent-key defaults). Vehicles that left play are forgotten.
    pub fn advance(&mut self, w: &World, rates: &[Rates]) {
        // spec: S-0102 claim 22 and glossary (the tick number: 0 while the first tick runs)
        let tick = w.tick - 1;
        let mut next = BTreeMap::new();
        for o in w.objects.values() {
            if o.kind != SimKind::Vehicle || !o.on_map {
                continue;
            }
            let r = rates
                .get(o.type_index as usize)
                .copied()
                .unwrap_or_default();
            let was = self.walks.get(&o.id).copied().unwrap_or_default();
            let moving = moving(w, o.id);
            next.insert(
                o.id,
                Walk {
                    counter: step(was.counter, tick, moving, r),
                    moving,
                },
            );
        }
        self.walks = next;
    }

    /// The walk of vehicle `id` (0 and standing for one never counted).
    pub fn of(&self, id: ObjectId) -> Walk {
        self.walks.get(&id).copied().unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // spec: S-0102 claim 22 (the walk counter steps on tick numbers divisible by the walk rate while moving)
    #[test]
    fn moving_steps_on_multiples_of_the_walk_rate() {
        let r = Rates { walk: 3, idle: 0 };
        let mut c = 0;
        let mut grew = Vec::new();
        for t in 0..10 {
            let n = step(c, t, true, r);
            if n != c {
                grew.push(t);
            }
            c = n;
        }
        assert_eq!(grew, [0, 3, 6, 9]);
        // The default walk rate steps every tick.
        assert_eq!(step(5, 7, true, Rates::default()), 6);
    }

    // spec: S-0102 claim 22 and D29 (g) (standing still: rate 0 keeps the counter; a positive idle rate steps on its multiples)
    #[test]
    fn standing_steps_only_with_an_idle_rate() {
        for t in 0..20 {
            assert_eq!(step(4, t, false, Rates { walk: 1, idle: 0 }), 4);
            let r = Rates { walk: 1, idle: 5 };
            assert_eq!(step(4, t, false, r), if t % 5 == 0 { 5 } else { 4 });
        }
    }

    // spec: S-0102 claim 22 (frame table: moving, S_w + d·F_w + (w mod F_w); standing with an idle
    // rate, the same row; standing otherwise, S_s + d·F_s when F_s > 0, else S_w + d·F_w)
    #[test]
    fn a_sprite_vehicle_walks_through_its_row_while_it_moves() {
        use super::super::build::{SpriteVehicleRows, sprite_vehicle_frame};
        let rows = SpriteVehicleRows {
            walk_start: 0,
            walk_frames: 4,
            stand_start: 40,
            standing_frames: 2,
        };
        let walk = |counter, moving| Walk { counter, moving };
        // Facing part 3, moving: 0 + 3·4 + (w mod 4), stepping with the counter.
        let frames: Vec<i32> = (0..6)
            .map(|w| sprite_vehicle_frame(rows, 3, walk(w, true), 0))
            .collect();
        assert_eq!(frames, [12, 13, 14, 15, 12, 13]);
        // Standing: the standing row, whatever the counter.
        assert_eq!(sprite_vehicle_frame(rows, 3, walk(5, false), 0), 46);
        // Standing with an idle rate: the walking row with the counter.
        assert_eq!(sprite_vehicle_frame(rows, 3, walk(5, false), 2), 13);
        // No standing frames: the walking row's first frame.
        let bare = SpriteVehicleRows {
            standing_frames: 0,
            ..rows
        };
        assert_eq!(sprite_vehicle_frame(bare, 3, walk(5, false), 0), 12);
    }

    // spec: S-0102 claim 22 (a multiple of the rate: 0 is a multiple of every rate, and only 0 is a multiple of 0)
    #[test]
    fn a_zero_walk_rate_steps_at_tick_zero_alone() {
        let r = Rates { walk: 0, idle: 0 };
        assert_eq!(step(0, 0, true, r), 1);
        assert_eq!(step(1, 1, true, r), 1);
        assert_eq!(step(1, 30, true, r), 1);
    }
}
