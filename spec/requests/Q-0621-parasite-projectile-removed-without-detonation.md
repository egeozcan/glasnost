---
id: Q-0621
title: What happens to a parasite whose projectile is removed without detonating?
status: answered
related: [S-0614, S-0306]
answered-by: [S-0614]
---

## Need

S-0614 claim 1 says the parasite travels inside its projectile and tries to latch "when the
projectile detonates"; claim 4 covers the failed latch at detonation. S-0306 table T2 removes a
projectile that leaves the legal map without a detonation. Neither spec says what becomes of a
parasite inside such a projectile. Our engine needs a rule in
`glasnost_sim::parasite::carrier_ended`, called by `glasnost_sim::combat::projectile::turn`
for a removal without detonation.

## Questions

1. When a latching shot's projectile leaves the map without detonating (for example a shot at a
   target near the map edge with an inaccurate or overshooting projectile), does the parasite
   come back at the cell it fired from (as a failed latch), come back somewhere else, or vanish?
2. Is the outcome the same for any other way a projectile ends without a detonation?

## Possible observation

A drone-type parasite at the map edge fires at a fast target that leaves its line, with a
projectile type whose flight can cross the legal map's border; record the parasite's fate.

## Current workaround

An interim rule, noted in the game's notes and in docs/development.md: the removal counts as a
failed latch (S-0614 claim 4): the parasite is placed back at the centre of the cell it last
stood in, facing north, or removed silently when it cannot be placed.
