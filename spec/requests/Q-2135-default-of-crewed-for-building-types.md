---
id: Q-2135
title: The default of the crewed flag for building types
status: open
related: [S-0312, S-0408]
answered-by: []
---

## Need
S-0312's Import mapping reads the crewed flag from the type section's `Crewed` key (boolean) and
gives its default as "type specs". F5 gives *C* = 0 for a building whose type lacks the flag,
and S-0408 claim 7's sale survivors use the same *C*. We found no spec of building types that
gives the default when a building type has no `Crewed` key, so we cannot tell whether such a
building releases survivors when it is destroyed or sold.

## Questions
1. What is the crewed flag of a building type whose section has no `Crewed` key?
2. Does the default depend on anything else of the type (for example its being a wall, a gate,
   a type infantry can occupy, or a type the map places for the neutral house)?
3. Is the default the same for vehicle types (claim 13), or does each kind have its own?

## Possible observation
Two test building types that differ only in that one has `Crewed=yes` and the other no `Crewed`
key, both with refund and divisor giving *C* = 5 (S-0312 scenario D13); sell each and count
the survivors (S-0408 claim 7 takes no chance draw, so *C* survivors appear on open cells).

## Current workaround
Absent means no: a building type without a `Crewed` key has no crew and releases no survivors.
The importer reads the key for building types (`crewed` in the building combat table).
