---
id: Q-2176
title: How a voxel vehicle's walk-animation counter advances, and what counts as moving for it
status: answered
related: [S-0105, S-0102, S-0402, S-0107]
answered-by: [S-0102]
---

## Need
S-0105 claim 16 draws a vehicle body in HVA frame *f_b = ν mod F_body*,
with *ν* the unit's walk-animation counter. S-0105's Terms leave how *ν*
advances to the vehicle specs. The only advance rule there is S-0102
claim 22's walk counter, given for vehicles drawn as sprites: it grows by 1
in a tick when the vehicle is moving and the tick number is a multiple of
its type's walk rate, or when it is not moving, its type's idle rate is not
0 and the tick number is a multiple of the idle rate. S-0402 F8 reads the
same *ν* for the harvesting image. Scenario MS-3 and MS-3b count body frames
in "walk-counter steps", which suggests one counter for both kinds of
vehicle, but no claim says so. S-0102 claim 22 also does not say which
state counts as "moving".

M8 round 6 note 39 ("terror drone legs do not move when terror drones are in
motion") came from our client keeping no walk counter at all. The client now
keeps one per vehicle in the view (`crates/app/src/mapview/walk.rs`), and the
sprite frame table, the voxel body frame and the harvesting image read it.

## Questions
1. Does a vehicle drawn from a voxel model advance its walk-animation
   counter *ν* by S-0102 claim 22's walk-counter rule (walk rate while
   moving, idle rate while standing), on the same tick numbers? If not, by
   what rule?
2. Which state is "moving" for that rule: S-0107 Terms' *is moving now*
   (a locomotor moving the vehicle, or a stored path), any movement in
   progress (a destination waiting for its path counts too), or only ticks
   in which the vehicle's position changes? Is it read before or after the
   tick's movement?
3. Does a vehicle that is carried, in the air, turning on the spot, or
   standing part-way through a movement step advance the counter?
4. Is the counter kept when the vehicle is loaded into a transport and
   unloaded again, and is it part of what a saved game restores?

## Possible observation
Give a voxel vehicle type with a body motion file of several frames a walk
rate of 3 and an idle rate of 0. Order it to drive 8 cells and record its
body frame each tick: whether the frame steps every third tick while it
drives, on which tick numbers, whether it steps while it turns on the spot
before the first step and while it waits at a blocked cell, and whether it
holds its last frame once it arrives. Repeat with an idle rate of 5 while it
stands. Repeat one run with a sprite vehicle of the same rates to compare.

## Current workaround
Our design, marked `// design:` in `walk.rs`: every vehicle, sprite or
voxel, keeps claim 22's one counter, advanced once per game tick (never per
drawn frame) from the state the tick left, with "moving" read as S-0107's
*is moving now* (`glasnost_sim::occupancy::moving_on_tick`). The counter is
presentation state kept by the view, beside the game: a save, a load or a
joined network game starts it again at 0. Tests
`a_vehicles_walk_counter_runs_while_it_moves` (`live_tests.rs`) and the
`walk.rs` unit tests.
