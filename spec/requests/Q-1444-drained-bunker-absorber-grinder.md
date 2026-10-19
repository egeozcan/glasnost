---
id: Q-1444
title: A drained tank bunker, infantry absorber or grinder and its occupants
status: open
related: [S-0608, S-0606, S-0605, S-0603, S-0405]
answered-by: []
---

## Need
S-0608 lists what being drained does to a building: the house's drain blackout (S-0405 claims 5
and 7), the *illegal* firing answer (S-0405 claim 9, table T1), no sale (S-0408 claim 3), the
outage animation slots (S-0108 claim 14) and no linked-weapon support (S-0518 claim 2). S-0606
says "Drain of the bunker building: planned S-0608; no drain rule is claimed here", and S-0605
and S-0603 do not mention the drain. The M7b interaction pairs (drain with the tank bunker, the
absorber and the grinder) need to know whether being drained changes anything for these
buildings' occupants and entrants beyond the list above. We do not want to read "not listed" as
"no effect" without saying so.

## Questions
1. **Tank bunker.** While a bunker of a drainable type is drained:
   1. does a vehicle already bunkered in it keep firing (with the bunker's rate-of-fire, damage
      and range changes), or does the bunker's *illegal* firing answer reach the vehicle?
   2. may a vehicle enter it, and may the bunkered vehicle be let out (S-0606 claims 3 and 12)?
2. **Infantry absorber.** While an absorbing building of a drainable type is drained, may
   infantry still enter it (S-0605 claim 1) and be released (claim 13)? (Its output, passengers'
   extra output included, and so the drain blackout follow from S-0405 claims 1 and 5; that part
   is not asked.)
3. **Grinder.** While a recycling building of a drainable type is drained, does it still accept
   and consume entrants and pay the recycle payout to its owner (S-0603 claims 5 and 8)?
4. Does a building's being drained affect any other per-occupant rule of these three specs?

## Possible observation
Named test data: make the bunker, absorber and grinder types drainable and give a Disc-like
drainer a link with each while it holds an occupant (and, separately, while an entrant is on its
way). Record the occupant's shots and damage, entries, releases, consumptions and payouts during
the link and after it ends.

## Current workaround
Our engine applies only the listed consequences. A drained bunker, absorber or grinder takes,
holds, releases and consumes occupants as at any other time; a bunkered vehicle keeps firing.
The pair tests check only the listed consequences and the blackout; they assert nothing
about occupants.
