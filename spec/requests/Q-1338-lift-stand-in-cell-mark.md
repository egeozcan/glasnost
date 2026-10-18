---
id: Q-1338
title: Whether a unit seized with a ground stand-in holds its cell's occupancy mark
status: answered
related: [S-0607, S-0209, S-0207]
answered-by: [S-0607]
---

## Need
S-0607 claim 5 has the seized unit give up its cell's occupancy (the vehicle-present mark, its reservation and its slot) because the jumpjet stand-in lifts it. Claim 22 (r4) says a hit with the driving, ship or teleporting locomotor seizes "as in claim 5", and claim 23 then moves the unit along the ground under S-0207 or S-0504. The spec does not say whether the unit standing on the ground with such a stand-in holds the vehicle-present mark of its cell (S-0209 claim 1) from the seize onward, or stands unmarked until its stand-in first moves it. The gap is in `lift::install_stand_in` (crates/deterministic/sim/src/lift.rs).

## Questions
1. Right after a seize with the driving stand-in, while the victim has not yet moved, can another ground vehicle drive into the victim's cell?
2. Does the answer differ for the teleporting and ship stand-ins?
3. When the victim stands still for good (for example after its grip ended), is its cell marked as occupied?

## Possible observation
Scenario L13 part b with a third vehicle ordered through the victim's cell on the tick after the seize, then again after the grip has ended.

## Current workaround
Claim 5's give-up runs for every stand-in, then the installed ground stand-in starts at rest holding the vehicle-present mark of the unit's own cell (S-0209 claim 1), as any vehicle at rest does. The driving and ship stand-ins move on from there under their own rules and marks.
