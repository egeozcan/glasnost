---
id: Q-1441
title: The drain refusals among the firing check's other illegal answers, and the stranded building
status: answered
related: [S-0608, S-0301, S-0601, S-0610]
answered-by: [S-0608]
---

## Need
S-0608 claim 3 adds three *illegal* answers (rank 4) to the firing check and says their rank
among the other rank-4 reasons is left unordered by S-0301. Our check returns the first answer it
finds, so the order decides which answer a firer gets when several apply (for example a draining
weapon whose projectile cannot reach the target and that also aims at a building already being
drained); the answer feeds the weapon choice's fallbacks and the AI's order filter. Separately,
claim 3 (a) refuses "the building that the firer itself drains". With
`quirks.drain_relink_strands_previous` on, a drainer can hold a stranded first link (Edge cases)
as well as its newer one, and the spec does not say whether part (a) also refuses the stranded
first building.

## Questions
1. Among the firing check's illegal answers (target tests, projectile tests, the takeover and
   berserk refusals of S-0601 and S-0610, the drain refusals), in which order are they tested, so
   that a firer for which several apply gets which answer? Does any answer of a lower rank
   number than 4 come between them?
2. With a stranded first link, is an attack on the stranded first building by its drainer refused
   by part (a)? With which weapon?

## Possible observation
For 2: S-0608 scenario D9 part a, then order the drainer to attack the first building with its
other weapon and record whether it fires.

## Current workaround
Interim, cited as Q-1441 in `crates/deterministic/sim/src/combat/check.rs`: the drain refusals
come right after the building's own answers (S-0405 T1) and the target and projectile tests, and
before the berserk, protection, hidden-target and firer-state refusals; part (a)
reads only the drainer's newest link, so the stranded first building is not refused by it.
