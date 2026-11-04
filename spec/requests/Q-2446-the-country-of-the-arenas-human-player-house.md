---
id: Q-2446
title: Which country does the human player house of the arena's game start have?
status: answered
related: [S-0216, S-0009, S-0412, M-0001, M-0017, M-0019]
answered-by: [S-0216]
---

## Need

S-0216 revision 67 claim 8 (*Game start*) gives our arena's game a human player house at start
3 (M-0001's idle house), first in S-0009 claim 19's house order. With the bases setting on, it
gets its base unit: the first type of the base-unit list whose owner list includes the house's
country (S-0009 claim 45). The unit stays on its base cell all game and takes whatever draws
its type takes in play (*Removal before the groups*). So the house's country picks the base
unit's type, and with it any objects that unit's creation makes (S-0611 claim 14) and any draws
that type takes in play.

M-0001 (Setup, *Players*), M-0017 and M-0019 name the two computer houses' countries (Americans
at start 0, Russians at start 1), but give the idle observer house at start 3 none. S-0216 claim
8 gives none either. M-0013 has a human player of the British country at start 3, but that is
another harness setup on another map, so we have not read it across.

The house's creation draw (S-0202 claim 14) and its base unit's creation draw do not depend on
the country, so M-0019's *N*₀ check does not depend on it.

## Questions

1. Which country did the records' idle human house at start 3 have in the games of M-0001 to
   M-0003, M-0016, M-0017 and M-0019?
2. If the records do not show it, which country should our arena's human player house take?

## Possible observation

PT8 (S-0216) records every object of every house at tick 0. The type of the human player
house's base unit shows its side. The house's country could be read from the harness's
settings file for those games, if that file is a record the lead may give.

## Current workaround

Our arena gives the human player house side A's country (Americans on the records' map; the
synthetic rules' first country in CI). That is our choice (design), not a reading of a record.
`oracle::HUMAN_COUNTRY_SIDE` names it.
