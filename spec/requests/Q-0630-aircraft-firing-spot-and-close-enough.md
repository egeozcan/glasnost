---
id: Q-0630
title: Where does a non-strafing aircraft fly to fire, and when is it close enough?
status: open
related: [S-0501, S-0301]
answered-by: []
---

## Need

S-0501 claim 12 and Open question 1 leave the firing spot of a non-strafing aircraft (fighters
and ordinary attackers) unresolved; claim 37 says one U(0, 99) draw chooses between two outcomes
when a candidate is found. Table T1 also uses "not close enough" in the *firing* and *refiring*
rows without defining it. The gap is in `glasnost_sim::air` (the *choosing* and *firing* stages
of the attack run).

## Questions

1. Where is the firing spot of a non-strafing aircraft relative to its target and to its own
   position, for each of the two outcomes of the U(0, 99) draw?
2. When does the search find no candidate, so that the run ends with no draw?
3. In T1, what makes an aircraft "not close enough" after a *not facing* or other refusal: a
   distance to the target, a distance to the firing spot, or the firing check's *out of range*
   answer?

## Possible observation

S-0501 scenarios AC8 and AC22: log the destination chosen at each *choosing* stage of a fighter
and of an ordinary attacker against a stationary target, from several approach directions, with
the random-state recorder.

## Current workaround

The *choosing* stage takes the U(0, 99) draw for a non-strafer and uses the target itself as the
firing spot (the strafer rule) whatever the draw; "not close enough" is the firing check's
*out of range* answer (marked Q-0630 in `glasnost_sim::air` and in docs/development.md, M6a
interim rules).
