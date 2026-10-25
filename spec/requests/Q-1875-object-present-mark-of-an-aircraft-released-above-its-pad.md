---
id: Q-1875
title: Where is the object-present mark of an aircraft that a factory release leaves above its pad, and when is it cleared?
status: answered
related: [S-0501, S-0209, S-0404, Q-1841]
answered-by: [S-0501]
---

## Need

S-0501 revision 52 (claim 48, answering Q-1841) states that a factory release puts an aircraft at
its docking position, height included, so that with a positive height component in the
displacement it rests above its pad: in neither cell layer, not tracked, and filed in the ground
render layer. Claim 48 does not say what happens to the aircraft's object-present mark (claim 44,
S-0209 claim 1).

The release first places the aircraft on the ground (claim 1), which marks the cell it is placed
in, at height 0. The release then moves it to its docking position above the pad. Claim 44 says
that marking and clearing use the aircraft's actual world height at the event. It also says that
an ordinary take-off clears the mark in the first movement tick that climbs away from rest.
The released aircraft makes no take-off (claim 48 (c)), and its first movement update toward a
destination descends rather than climbs. So neither rule says when, or whether, its mark is
cleared.

The mark gives no vehicle entry refusal (claim 44), but other readers of the record may test it
(S-0209 claim 1, the landing and placement record tests). So the engine has to decide where the
mark is and when it ends.

## Questions

1. After a factory release leaves an aircraft resting above its pad (claim 48 (a)), is the cell's
   object-present mark that its ground placement set still set? If it is, is it on the cell of its
   placement or on the cell of its docking position, and in which record (ground or deck)? Or does
   the release clear it?
2. If a mark stays set: when is it cleared? At the order that gives it a destination, in its first
   movement update after it accepts the destination (which descends), in the update that first
   moves it out of the cell, in its first climb, or not until some later event (a landing
   elsewhere, its destruction)?
3. For the low release (claim 48 (c), a release height of at most 20 leptons, whose first update
   reaches height 0 and lists it in its cell's ground layer): is a mark set or kept at that point,
   and is it cleared by the climb in its second update as in claim 44's take-off?

## Possible observation

AC30 part k (2) and (3) on the test pad above the terrain: record the cell's object-present flags
(ground and deck records) in the docking cell and the placement cell from the release tick on,
while it rests, at the move order and in each of its first three movement updates.

## Current workaround

Interim in `air::released_docked` and `air::tick_flight`: the released aircraft keeps the mark of its
ground placement, moved to its docking cell (the ground record, by its height above the terrain
under claim 44's 416-lepton test). The mark is cleared at the start of its first movement update
toward a destination, before that update's descent. A release with the quirk
`quirks.aircraft_released_above_pad` off settles the aircraft at height 0 and marks the cell as for
an aircraft docked by landing.
