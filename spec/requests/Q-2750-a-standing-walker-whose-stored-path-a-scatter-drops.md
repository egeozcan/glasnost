---
id: Q-2750
title: The extra locomotion of a standing walker whose stored path a scatter drops
status: answered
related: [S-0208, S-0209]
answered-by: [S-0208]
---

## Need

S-0208 r52 claim 6, *Destinations given during a unit's update*, gives the extra locomotion at a
scatter whose destination comes from the nearby-cell search to two walkers: one "standing with no
walking target and no stored path", and one "standing with no walking target that keeps a stored
path at the scatter" (*A stored path kept under enter*). It adds, as the original's, that "the
scatter starts the extra locomotion whatever the walker's state".

A third walker stands with no walking target and holds a stored path that the scatter drops: for
example a walker in *move* on the arrival tick of a standing position in the middle of its walk
(it does not choose its next walking target on its arrival tick, claim 6), or a walker under
*enter* in contact with what it enters, or any walker with `quirks.enter_keeps_stored_path` off.
After the scatter it has no walking target and no stored path, but it held one when the scatter
came. The claim does not say whether it has the extra locomotion. The code is
`crate::scatter` (where the request reads the walker's state before it gives the destination) and
`crate::walk::extra_locomotion`.

## Questions

1. Does a walker standing with no walking target whose stored path is dropped by an accepted
   scatter whose destination comes from the nearby-cell search have the extra locomotion at the
   scatter, so that it searches for the scatter's destination and chooses its walking target at
   once?
2. Does the answer differ on the walker's arrival tick (where its own locomotion would not choose
   a walking target) from a later tick on which it stands with a stored path, such as a tick on
   which it waits for a blocked cell?

## Possible observation

S-0208 scenario W17 part i's recorder, with *A* walking a long stored path in *move* and a hit
without a threat point (W7 part r) timed on the arrival tick of one of its standing positions
in the middle of the walk. Record whether *A* holds a walking target at the end of the tick of the
hit, and the draws of that tick.

## Current workaround

The state is read before the request, as S-0208 r46 gave it: such a walker held a stored path at
the scatter and does not keep it, so it has no extra locomotion. It first searches for the
scatter's destination at its next locomotion with no walking target.
