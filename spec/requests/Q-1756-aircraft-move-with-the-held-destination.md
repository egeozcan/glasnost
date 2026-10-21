---
id: Q-1756
title: Whether an aircraft's move with the destination it already flies to accepts a new movement point
status: open
related: [S-0501]
answered-by: []
---

## Need
S-0501 claim 6 replaces the world height of a movement point accepted while the aircraft has a
target and ammunition (terrain height there plus cruise height), and M8 step A4 fixes that
replacement when the point is accepted. Claim 40 has *move* select its destination first and then
take that destination's heading point, and claim 33 has an aircraft in *guard* in flight with a
destination queue *move*. No spec says whether a *move* that starts with the destination the
aircraft is already flying to is a new acceptance (so claim 6 is decided again, and claim 40's
selection runs again) or keeps the point already held.

It matters for every armed aircraft with no dock types, and the installed data has such types:
two armed, landable aircraft types have no dock types at all (counted in the local lane of M8 step
A4). When such an aircraft's run ends, table T2 gives it *guard*, keeping its destination (the last
firing spot), and claim 33 then gives it *move* to that destination. If the held point stays
raised, the aircraft stays above the spot at cruise height (holding height, claim 6) and never
lands, after every attack. If the move accepts a new point without a target or ammunition, it
descends and lands there. The engine needs this in `crates/deterministic/sim/src/air.rs`
(`tick_flight`, where a destination different from the one held is accepted).

## Questions
1. An armed, landable aircraft with no dock types attacks a durable ground vehicle until its
   ammunition is spent. After its last run, does it land at its last firing spot, or stay in the
   air above it? If it lands, how many ticks after the last round does it start to descend? Does
   it do anything else first?
2. The same aircraft attacks a ground vehicle that is destroyed before the aircraft fires (so it
   still has its ammunition). Does it land at the old firing spot, or stay in the air above it?
3. More generally, when *move* starts with the destination the aircraft is already flying to, does
   claim 40's destination selection run again (with its direction draws when the cell is taken)?

## Possible observation
For question 1: such an aircraft on open ground, a durable enemy vehicle 12 cells away. Order the
attack and record the aircraft's behaviour, destination, height, position and ammunition every
tick from the order until 1000 ticks after its last round. For question 2: the same, with the
vehicle destroyed by another unit while the aircraft is still approaching, recorded for 600 ticks
after the vehicle's death. For question 3: repeat question 1 with a second aircraft parked on the
last firing spot's cell, to see whether a replacement cell is sought.

## Current workaround
A *move* with the destination already held keeps the held point, so claim 6's replacement made when
the point was accepted stays: the aircraft keeps cruise height above the spot and does not land.
Claim 40's selection runs only for a destination different from the held one, as before step A4.
