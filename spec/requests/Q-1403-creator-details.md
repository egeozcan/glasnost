---
id: Q-1403
title: Small creator details of S-0616 claim 1
status: open
related: [S-0616, S-0303, S-0301]
answered-by: []
---

## Need
Details that S-0616 claim 1 and the specs it points to leave open, met while building the
creators.

## Questions
1. Barrel particle system (S-0303 claim 19): which owner house does it get: none, the outer
   detonation's house, or the overlay's owner?
2. A weapon with more than one of the spark-particle, fire-particle and railgun flags: which
   emitter (or emitters) does its shot make, and which live-effect kind does S-0301 claim 5's
   gate use for it?
3. Flame sweep (claim 11): "the owner's fire point for its first weapon" — with which burst
   counter (the shot's, 0, or the owner's current one)?
4. Natural smoke (claim 1): "when the building becomes yellow" — does a hit that takes the
   building from above yellow straight to red also multiply the timer?
5. A spark emitter whose system type has a spark-turn count of 0 or less never ends by its
   counter (claim 15), so the firer's spark place stays held until the firer is removed and its
   spark weapon (and the other weapon) never fires again. Is that what the original does, and
   does any other event (a move order, a new target) end such an emitter?

## Possible observation
P1's barrel and effect-weapon rows with test rules: a weapon with two flags; a spark system
type with SparkSpawnFrames=0 on a weapon; a building taken from green to red by one hit.

## Current workaround
1. The outer detonation's house. 2. The first of spark, fire, railgun, one emitter. 3. Burst
counter 0. 4. Both "now yellow" and "now red" hit results count; the factor is applied once.
5. As the spec reads: nothing else ends it.
