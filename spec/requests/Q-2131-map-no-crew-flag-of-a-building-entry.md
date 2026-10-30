---
id: Q-2131
title: The map flag that leaves a building without crew
status: answered
related: [S-0009, S-0312, S-0408]
answered-by: [S-0312]
---

## Need
S-0312 F5 gives *C* = 0 when a building has no crew, and says a building has no crew when its
map entry says so, or when its killing damage ignores defences. Claim 24 kills the absorbed
passengers of such a building instead of letting them out. The Import mapping lists the source
as "map, building entry | building has no crew (F5) | map flag (map specs) | crew present". S-0009,
the map spec that describes building entries, gives no field for it: we found no position, key
or value in a building entry that carries a no-crew flag.

Step A7 of M8 built S-0312 claim 18 and S-0408 claim 7. The no-crew state is kept on each
building (`combat.no_crew`); killing damage that ignores defences sets it. Nothing sets it from
a map.

## Questions
1. Which field of a map's building entry gives the no-crew flag (its position in the entry, or
   its key), and which values mean "no crew"?
2. Does the flag also affect anything else: a sale's survivors (S-0408 claim 7, which uses F5's
   *C*), a vehicle's crew (claim 13), or a capture (S-0314)?
3. Does the flag stay with the building when its owner changes, or when it is repaired or
   upgraded?

## Possible observation
Two maps that differ only in that field of one crewed test building's entry; destroy the building
with damage that does not ignore defences, and record the survivors and the draw count
(S-0312 scenario D13, variant f).

## Current workaround
No map sets the no-crew state: every building placed from a map keeps its crew. Killing damage
that ignores defences still sets it (F5, claim 3's expiry hit). The seam is recorded in the
claim ledger of step A7 (`docs/m8-claims/a7-S-0312-survivors.md`).
