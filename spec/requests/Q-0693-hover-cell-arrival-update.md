---
id: Q-0693
title: Which events are cell-arrival updates of a hovering vehicle leaving a factory
status: open
related: [S-0417, S-0503]
answered-by: []
---

## Need

S-0417 claim 7 makes a tethered vehicle report clear on a *cell-arrival update*, and the report
lets the factory close. The claim defines these updates for a vehicle moved by driving
locomotion (reaching a track target, reaching a chain point) and S-0208 claim 23 defines them for
a walker. S-0417 claim 5 sends a hovering vehicle out of a factory by its own locomotor, with the
destination (*x*₀ + 4, *y*₀ + 1). No spec says which events of a hovering vehicle's movement
(S-0503 claims 5 to 7) are cell-arrival updates. Without one, a produced hovering vehicle never
reports clear, its factory never closes and the next unit cannot leave.

The code in `crates/deterministic/sim/src/hover.rs` needs the rule.

## Questions

1. Is a hovering vehicle's arrival at its heading point (S-0503 claim 7, F5) a cell-arrival
   update, also when it chooses a further heading point on the same tick?
2. Is the end of its move (its destination cleared in the destination cell) a cell-arrival
   update?
3. Is crossing a cell edge between heading points one?

## Possible observation

S-0417 scenario U6's hovering case: record the tick of the factory's clear report (its door
starting to close) against the hover's arrival ticks at each heading point and its cell-edge
crossings.

## Current workaround

Each arrival at a heading point (F5) is a cell-arrival update, after the next heading point is
chosen, as reaching a track target is for a driving vehicle. A cell-edge crossing is not. The
choice is marked in the code with this request's ID.
