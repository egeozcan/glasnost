---
id: Q-0643
title: Which amphibious movement zones are the exception of the naval match
status: open
related: [S-0504, S-0211]
answered-by: []
---

## Need
S-0504 claim 7: a building refuses a unit that asks to enter or dock when exactly one of the
two has the naval flag, "unless the unit's movement zone is *amphibious*". S-0211 claim 12
has three zones whose names contain amphibious: *amphibious*, *amphibious destroyer* and
*amphibious crusher*. The claim names only the first. `naval::naval_match_refuses` needs to
know whether the other two are exceptions too.

## Questions
1. A non-naval vehicle of movement zone amphibious destroyer asks to dock at a stock naval
   yard. Is it refused?
2. The same with movement zone amphibious crusher.
3. A naval vehicle of either zone asks to dock at a stock land repair depot. Is it refused?

## Possible observation
Copy a stock amphibious vehicle into a naval and a non-naval variant, give each zone in turn,
and send it to a naval yard and to a land repair depot; record whether it is accepted.

## Current workaround
Interim rule: only the zone *amphibious* is the exception (the literal reading). Nothing
calls the naval match in play yet (Q-0641).
