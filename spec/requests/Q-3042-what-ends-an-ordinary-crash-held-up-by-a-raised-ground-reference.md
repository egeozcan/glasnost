---
id: Q-3042
title: What ends an ordinary jumpjet crash while the look-ahead ground reference is raised by a listed object?
status: answered
related: [S-0502, S-0312, S-0414]
answered-by: [S-0502, S-0312]
---

## Need

S-0502 claim 18 makes a crashing jumpjet take its ordinary height step (F4) toward the target
height −5 and then drop by the crash rate *k*; the crash ends when its height above the surface
falls below 1 lepton or the drop crosses a deck. While its movement point lies in another cell,
F4's reference is claim 9's look-ahead ground reference (F3), which includes the clearance of
whatever is listed in the ground layer of its cell or the cell ahead (85 leptons for a listed unit,
104 per art-height level for a building).

When the unit is lower than that reference minus 5, F4 raises it by the climb rate *c* (or snaps
it up to the reference minus 5), and the drop takes *k* off again. With *c* = *k*, which is what
two types that leave out both keys get from the absent-key defaults, the height does not change
from tick to tick; with *c* > *k* it alternates. The crash end is then never reached while the
listed object stays: in a synthetic test (GitHub issue #189) an infantry jumpjet killed at 50
leptons over a cell where another infantry unit stood was still in play, dead and *crashing*, 3000
ticks later. Alone in its cell the same unit ends its crash 5 ticks after the kill. Two such units
killed together in one cell each hold the other up. The suspended unit also keeps its place in
S-0414 claim 4's on-map counts, since those drop only when it leaves the map.

The arithmetic is what F3, F4 and claim 18 say; nothing in the spec says what ends the crash in
this case. Claim 9 describes the rise toward a clearance only for a dropped seized unit, whose
crash rate grows by 1 each tick, so its fall always wins. S-0312 claim 16 says "a dead hovering
unit cannot remain suspended merely because its order ended", and claim 15 that a destroyed
crashable vehicle "stays in play until it hits the ground", taking for granted that it does. The
Backing note of S-0502 claim 3 says "an ordinary killing hit also clears orders and can invoke
claim 17", and scenario J15 records an "order clear/reroute" at the crash start, but no numbered
claim gives that rule; our killing hit leaves the order-level destination and movement point as
they were.

## Questions

1. For an ordinary (not seized) crashing jumpjet, which ground reference does F4 use: claim 9's
   look-ahead reference with its listed-object clearances, the direct reference of its own cell,
   or the bare surface?
2. Does F4's ordinary step raise a crashing unit at all (by *c*, or the snap to the reference
   minus 5), or does it only lower it while *crashing*?
3. Does an ordinary killing hit clear the jumpjet's order or movement point (the "order clear/
   reroute" of claim 3's Backing note and J15)? If so, at what moment, and can claim 24 then give it
   a movement point that still lies in another cell?
4. If the crash can be held up as described, is there any other event that ends it (a time limit,
   the listed object, the unit's own motion only)?

## Possible observation

A crashable infantry jumpjet type that gives neither the climb nor the crash key, standing in a
cell with another infantry unit of no order. Order it to a far cell and kill it 50 leptons up,
before it has any speed. Record its height each tick after the kill, and whether and when its
impact comes. Repeat alone in the cell as the control.

## Current workaround

None: the code follows S-0502 F3, F4 and claim 18 as written, so a unit can stay suspended in
*crashing* while something is listed under it. Nothing in the code cites this request yet.
