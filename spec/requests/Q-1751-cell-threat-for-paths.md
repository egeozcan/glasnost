---
id: Q-1751
title: Which cells count as threatened for a unit's path, and when
status: answered
related: [S-0212, S-0309]
answered-by: [S-0212]
---

## Need

S-0212 reads a house's *threat value* of a cell (straightening, claims 17 and 18, F7) and the
*coarse threat* of a sector step (the corridor, claim 13, F5, F9), weighted by the unit's threat
coefficient *k* (the type's threat-avoidance field). Its Terms say how these values arise
"belongs to the combat and AI-visible-rules specs", but no approved spec defines them. Our engine
uses 0 for every cell and every sector step (`glasnost_sim::path`, the straightening test near the
`enters_clear` closure; `glasnost_sim::corridor`, F9's *T*), so a unit with a positive *k* never
avoids anything. This may be one source of the lead's "weird pathing" play note (M8 step A4) if
stock units with a positive *k* route differently in the original.

## Questions

1. For a unit owned by a person, whose type has a positive threat-avoidance value, ordered across
   open ground past an enemy armed building or armed unit: does its route bend away from the enemy
   compared with the route of an otherwise identical type with the value 0? From how far away?
2. What makes a cell threatened for a house: enemy armed buildings, enemy armed units, both? Does it
   depend on the enemy's weapon range, damage, health or visibility (shroud, cloak) to that house?
3. When does a cell's threat change: at once when the enemy object appears, moves, dies or is
   revealed, or only at intervals? If at intervals, how many ticks apart?
4. Does the same notion of threat apply to the long-distance (sector) part of a route, and does it
   change which way round a lake or a base a unit drives on a long trip?
5. Do units owned by a person and units of a computer player use threat in the same way?

## Possible observation

Two copies of one map: a long open route that passes within weapon range of an enemy defence
building, with a detour of a few cells available. Order a unit of a type with the threat-avoidance
value set (a modified type for the test if no stock type has one) and an otherwise identical type
without it; record the cells passed each tick, 5 runs each; repeat with the defence removed, and
with the defence under shroud.

## Current workaround

Every cell threat value and every coarse threat is 0 (the comment in `path.rs` still says "before
M4"); all threat tests pass. No invented threat model.
