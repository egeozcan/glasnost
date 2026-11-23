---
id: Q-3389
title: A vehicle asleep at a depot and claim 30's install of its point
status: answered
related: [S-0207, S-0314]
answered-by: [S-0314]
---

## Need

S-0207 r103 claim 30 installs the point of a driving vehicle's order-level destination at
every locomotion update with no track, no point and no stored steps, except while it sinks,
while its body turns, or in *guard* or *unload*. Claim 31 makes a driving vehicle that holds a
point in its own cell search for that cell, fail and clear the point, and its Changes entry for
S-0314 claims 17 and 21 calls the resulting interval "our engine's rule in *enter*".

A vehicle that has made its movement arrival at a depot (S-0314 claim 17) has *sleep* queued and
then current, and keeps the depot as its order-level destination until the depot's settling step
removes it (S-0314 claim 21). Under claims 30 and 31 it now has its point present in all but one
tick of every *R*: the completion row clears it, the next update installs it again, and each later
search, about *R* ticks apart, clears it for one tick. The depot's waiting processing (S-0314
claim 6) asks the moving-destination test (S-0314 Terms, claim 21), which counts the present point
as movement, every *D* ticks. When *D* and *R* do not bring a processing onto one of the cleared
ticks, the depot never finds the vehicle still, and the vehicle is never repaired. Our synthetic
depot tests show this for a second truck that docks in place 1 with the other place busy: it
stands asleep beside the depot and is never repaired (S-0314 claim 23, its free-place case).

## Questions

1. Does a vehicle in *sleep* after its movement arrival at a depot take claim 30's install, so that
   its point comes back after the completion row clears it? Or is *sleep* (or the movement arrival)
   one more exception, like *guard*?
2. If it does take the install, what makes the depot's waiting processing accept it: should the
   moving-destination test of S-0314 ignore a point in the vehicle's own cell, or should the
   processing read the vehicle at a different moment?
3. Does the answer change for the same-cell *enter* case of S-0207 claim 31 (a vehicle already
   in the docking-point cell when the order is accepted)?

## Possible observation

Two vehicles given *enter* on a two-place depot at once, the second arriving while the first is
serviced. Record each tick the second vehicle's locomotor destination point, its behaviour, and the
depot's processing ticks and phase until the second vehicle is repaired.

## Current workaround

Batch S50 keeps today's rule for a vehicle that has made its movement arrival at a depot: claim
30's install at an update with no point is not made for it (`drive::install_again`, marked
`design: S50 hold-back pending Q-3389`). Everything else in claims 30 and 31, the approach to a
depot and the same-cell *enter* case included, follows r103.
