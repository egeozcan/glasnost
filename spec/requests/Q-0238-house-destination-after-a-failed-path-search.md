---
id: Q-0238
title: Which cell a neutral or special house gives its unit that gave up after a failed path search
status: open
related: [S-0212, S-0215]
answered-by: []
---

## Need

S-0212 revision 22, claim 23 row 2: a unit that gives up after a failed path search queues
*area guard* when its owner is computer-controlled, and in a skirmish "is then given a new
destination chosen by its house (a house cell whose choice belongs to the AI-visible-rules
specs), provided that choice yields a cell; otherwise it gets none". Departure D5 removes this
for player houses, but the neutral and special houses keep it. No approved spec says which cell
those houses choose, so our engine cannot give the destination. This is in `path::failed`.

## Questions

1. In a skirmish, which cell does the neutral house give its unit that has just given up? Which
   cell does the special house give? Is it a fixed cell of the house (for example a base or
   start cell), a cell chosen relative to the unit, or none in practice because these houses
   have no such cell?
2. Does that choice take any draw from the scenario random stream?
3. Is the destination given in the same tick as the give-up, and is *move* or any other
   behaviour queued with it besides *area guard*?

## Possible observation

S-0212 scenario P3 or P11 with a vehicle owned by the neutral house and then by the special
house, ordered (by a map trigger or a scripted setup) to a cell it cannot reach more than one
cell away. Record its destination, its current and queued behaviours and the random stream
state on the tick of the failed search and the next ticks.

## Current workaround

The unit queues *area guard* and gets no destination from its house. Interim row in
`docs/development.md`.
