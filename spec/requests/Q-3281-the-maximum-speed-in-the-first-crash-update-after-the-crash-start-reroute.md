---
id: Q-3281
title: The maximum speed in the first crash update after the crash start's reroute
status: open
related: [S-0502, S-0312]
answered-by: []
---

## Need

S-0502 r30 claim 18 says that the crash start's stop reroutes a unit with a
movement commitment to the centre of the cell that the nearby search around
its current cell finds, usually its own cell (*Destination clear at crash
start*). It also says that the first crash update's ordinary physics still
uses the preceding flight state and target height, and that neither the
crash start nor *crashing* sets the maximum speed, so the maximum speed keeps
the value that its flight state last set (claims 7, 11 and 12) until the
crash end (*Path of the fall*). Scenario J12's balloon-hover subcase (a)
expects a unit killed while cruising at full speed towards a cell 20 or more
cells away to keep its maximum speed at full speed until the crash end.

The engine runs the preceding state's whole update in the first crash
update: for a unit killed while cruising, that is claim 11's cruising
outcome, with its F6 speed bands, measured against the movement point the
reroute has just set. That point is now within about one cell of the unit,
so the bands set a lower maximum speed in that update, or claim 11's arrival
(ρ < 20 leptons) sets both speeds to 0. Measured in the engine on synthetic
rules (a balloon-hover crashable jumpjet vehicle with full speed 14, killed
cruising at 14 towards a cell 18 cells away): the maximum speed is 14 at the
hit and 7 from the first crash update to the crash end, because the rerouted
point lies 96 leptons away, in F6's third band. Base (before r30) did the
same for a vehicle with a nonempty order-level destination, since its one
empty assignment already rerouted it.

The spec does not say which part of the preceding state's update the first
crash update keeps. The code is in `crates/deterministic/sim/src/jump/fly.rs`,
`tick` (the state outcome runs before `crashing_outcome`).

## Questions

1. In the update that enters *crashing*, does the unit's preceding flight
   state still make its own state decisions (for *cruising*: claim 11's
   arrival test and F6's speed bands, measured against its movement point),
   or only its height step and F1's speed step?
2. If those decisions do run, what does scenario J12 (a) show for the
   maximum speed after the hit: full speed until the crash end, or the value
   F6's band gives for the distance from the kill position to the centre of
   the cell the reroute found (and 0 speeds with no sideways step when that
   distance is under 20 leptons)?
3. Does the same hold for a unit killed while *ascending* or *descending*
   with a commitment, whose rerouted point is near?

## Possible observation

J12's balloon-hover subcase (a), recording the maximum speed in the tick of
the hit and in each of the next three ticks, killed once near the centre of
its cell (under 20 leptons from it) and once near a corner of its cell.

## Current workaround

As built: the first crash update runs the preceding state's outcome against
the rerouted point, so the maximum speed can drop in that update. No guard
test asserts the maximum speed of J12 (a) or (d). Batch S39's tests assert
(c), (b), and the parts of (a) that do not depend on this (on the crash path
throughout, the body facing equal to the heading, a sideways step only at a
current speed above 0, an open arc).
