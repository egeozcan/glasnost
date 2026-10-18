---
id: Q-1425
title: Does a burning walker's damage over time reduce resource cells
status: open
related: [S-0615, S-0109, S-0303]
answered-by: []
---

## Need
S-0615 claim 11 says an expiring walker's advances deliver S-0109 claim 17's damage over time with
the flame warhead and no attacker. S-0303 claim 17 says which detonations may reduce resource
cells, but no spec says it for an animation's damage over time. Code:
`crates/deterministic/sim/src/walker.rs`, `advance_damage`.

## Questions
1. Can the damage over time of a damaging animation (a burning walker included) reduce the
   resource on the cell it hits, when its warhead could otherwise do so?
2. Is any house given for that damage (S-0302 claim 1's supplied attacking house)?

## Possible observation
A burning walker that expires on an ore cell, with a flame warhead that reduces resources in an
ordinary detonation; compare the cell's resource before and after.

## Current workaround
Resource reduction allowed; no attacker, no house.
