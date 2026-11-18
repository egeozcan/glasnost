---
id: Q-3213
title: What a house removed by the unused-house cleanup leaves in the start position marks
status: answered
related: [S-0009, S-0414]
answered-by: [S-0009]
---

## Need
S-0009 claim 29 (r45) gives every start position 0 … 15 a mark, applied in the early phase
and again in the late phase. The unused-house cleanup of claim 28 runs between the two. S-0009's
Open questions say that what a removed house leaves in the marks is not established. The engine
builds claim 29 now (sim batch S27, `load::new_game` and `starts`), so it has to do something
with the marks at the cleanup, and in the References rule ("player at position X"), whenever a
removed house's number is a mark.

This arises only when the cleanup runs (a game with no computer player) and either the early
phase already saw the neutral or the special country as non-passive (claim 33), or the map has
header pairs beyond the eighth.

## Questions
1. A neutral or special house takes a start position in the early phase, and the cleanup then
   removes it. In the late phase, is that position free (mark −1) for a house that is assigned
   anew there? Or does it stay taken, so that the farthest-sum rule sums over it and no house can
   be given it?
2. After the cleanup removes a house, do the houses after it in house order keep their house
   numbers for the late phase's mark test? Example: the neutral house (number P) is removed and
   the special house (number P + 1) is kept. Does a mark of P + 1 still mean the special house
   holds that position?
3. A header pair puts the removed house's number on a position in the late phase. Is that
   position then taken by no player, as for a mark that names no house?
4. "Player at position X", where mark X is a removed house's number: is the reference empty, or
   does it name the removed house? If it names it, what can a trigger action still do to that
   house?

## Possible observation
This needs a skirmish with no computer player in which a neutral or special house is
non-passive and is removed. That is the no-computer-player cleanup setup that S-0009 claim 28
says no in-scope skirmish fixture yet provides. A map with N_h = 9 and pair 9 naming the neutral
house's number on position 0 would then answer questions 3 and 4 through a reveal trigger on
"player at position A", as in scenario M13b variant E.

## Current workaround
Our engine's interim rule, noted in `docs/dev/simulation.md` (*sim issues S27*):
1. The cleanup sets every mark equal to the removed house's number back to −1. The position is
   then free in the late phase, as it was before claim 29 was built, when the removed house's
   position went with the house.
2. House numbers never change. The houses after a removed one keep theirs.
3. A late-phase pair can put the removed house's number back. The position then counts as taken
   by no player.
4. The reference is empty: a removed house is no house (S-0414 claim 1, the house itself is
   removed). This is how `mp_triggers::resolve_house` already treats claim 21's reference.
