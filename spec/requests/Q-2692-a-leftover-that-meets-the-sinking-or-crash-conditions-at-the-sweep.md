---
id: Q-2692
title: The sweep's hit on a leftover vehicle that meets the sinking or crash conditions
status: open
related: [S-0203, S-0312, S-0414]
answered-by: []
---

## Need

S-0203 r50 claim 13 (c) has the defeat sweep's hit of no amount on a leftover vehicle with no
health repeat the death effects of S-0312 that the vehicle's state allows. Its S-0312 entry in
**Changes for other specs** says: "What happens to a leftover that meets claim 10's sinking
conditions or claim 15's crash conditions at this hit is not established."

Our engine can meet the case. A vehicle removed with no health keeps the position and state it had
when it left play, so a leftover can be of a naval type standing on water (claim 10's sinking
conditions) or of a type that can crash and stand above the terrain (claim 15's crash conditions).
The plainest way is a vehicle whose sinking or crash ends, so that it leaves play, in the tick of
its house's defeat or surrender.

## Questions

1. When the defeat sweep's hit reaches a leftover vehicle of a naval type that left play on water
   with no health, does it repeat the death effects of S-0312 claim 1 (debris, death weapon,
   animations, crate), start a second sinking, or do nothing?
2. The same for a leftover of a type that can crash and that left play above the terrain with no
   health: does the hit repeat the death effects, start a crash, or do nothing?

## Possible observation

1. A house whose last vehicle is a naval crate carrier on water, destroyed so that its sinking ends
   and it leaves play in a tick in which the house is defeated (or surrenders, so that the sweep
   runs in that tick). Record the crates, detonations and animations at its position in that tick.
2. A house whose last vehicle is a crate carrier of a type that can crash, destroyed above the
   terrain so that its crash ends and it leaves play in a tick in which the house is defeated.
   Record the same.

## Current workaround

No hit, cited `Q-2692` at the sweep: our engine offers the sweep's hit to such a leftover as it
does to the other leftovers whose effects claim 13 (c) leaves open (those with health left, those
of other kinds), which is none. Its place in creation order still counts for S-0414 claim 15.
