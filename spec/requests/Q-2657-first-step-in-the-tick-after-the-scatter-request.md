---
id: Q-2657
title: The first step one tick after a released walker's scatter request, and a walker that keeps walking to its old destination
status: open
related: [S-0417, S-0208, S-0212, S-0516]
answered-by: []
---

## Need

M-0023's baseline (S-0417 U10 part b on its map) and run 1 both give the first step 1 tick after
the turn of the factory's scatter request. Our engine gives it 2 ticks after, in the baseline and
in run 1 alike, so the relative form of S-0417 claim 17 (the first step "as soon after as the
baseline's") holds and the absolute one does not.

Our reading: S-0417 claim 17 sends the request at the end of the unit's turn
(`crates/deterministic/sim/src/scatter.rs`, `resting_turn`), after that turn's locomotion. The
destination is therefore first searched for in the next tick's locomotion, which by S-0208
claim 6's paragraph on the asking tick ("on the first cell of a walk it is the tick before the first
step") is the asking tick, so the first step comes one tick later still. Only a teleporting unit
gets a movement update at once (S-0417 claim 17, *During the walk-off*).

M-0023's controls show a second difference, which may follow from the first. In the game the
unit's first step is in tick *t* + 2, and the move order to the pen's centre is given at the start
of that tick. From then on the destination reads the pen's centre and *move* is current, yet the
unit's positions are the baseline's, tick for tick, until its walk ends on the baseline's
destination cell; only then is the destination cleared. In our engine the unit has not searched
for the baseline's destination by *t* + 2, so the ordered destination's fresh search fails at the
gate (S-0212 claim 23) and the unit waits.

## Questions

1. After the factory's scatter request gives a resting infantry unit a destination, in which tick
   is that destination first searched for: in the same tick (after the request), or in the next
   tick's movement update? Is the tick of the request also the asking tick of S-0208 claim 6, with
   the first step in the next tick?
2. Is the same true for other destinations given in a unit's turn, for example S-0208 claim 13's
   fall-back, or a destination given by behaviour processing in an ordinary turn?
3. A walking unit that has started a walk on a stored path is given an ordinary move order to a
   cell in another reach region. Does it keep taking the steps of its stored path to the old
   destination while its destination reads the new cell? When is the new destination searched for,
   and why does no gate failure clear it before the walk ends? Is the destination then cleared at
   the end of the walk because the unit stops, though it is not at the new cell?

## Possible observation

For questions 1 and 2, M-0023's baseline already gives question 1's answer for the request. For
question 3: an infantry unit walking a long straight line on open ground, given a move order to the
pen's centre in the middle of a cell step and, in a second run, as it stands on a cell between
steps. Record its position, current behaviour and destination per tick until it stops.

## Current workaround

The engine is unchanged: the request at the end of the turn and the next tick's search, as
S-0417 claim 17 and S-0208 claim 6 are written. The local-lane check
`m0023_rows_hold_in_our_engine` reports M-0023's row for the first step after *G* and its controls
row as failing and expects them to fail until this is answered.
