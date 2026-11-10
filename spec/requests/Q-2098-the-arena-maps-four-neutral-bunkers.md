---
id: Q-2098
title: Which objects are the four neutral bunkers of the arena records' map
status: answered
related: [S-0009, S-0606, S-0508, M-0001, M-0002, M-0003, M-0011, M-0015, M-0017]
answered-by: [S-0216]
---

## Need
The arena records as revised on 2026-10-29 list, among what is not stock in their games, "the
map's four bunkers, owned by a neutral house, are in play from tick 0" (M-0001 to M-0003 Setup,
M-0017 Setup; M-0011 and M-0015 Setup say the same). Our oracle arena
(`crates/tools/src/oracle.rs`) loads the records' map as any game on it (S-0009) and must have
those four objects in its tick-0 state.

Our tick-0 state on that map has no *tank bunker* (S-0606 Terms; the glossary's *tank bunker*):
no building of the neutral house, or of any house other than the two sides, has a type with the
bunker flag. The neutral house owns many map buildings at tick 0, garrisonable ones (S-0508
Terms) among them, and the map has terrain objects. A neutral building also stands on cells of
the side-A groups (Q-2096). We cannot tell from the records which four objects they mean, and we
do not pick them by their type IDs.

## Questions
1. What kind of object is each of the four bunkers: a tank bunker in S-0606's sense (a building
   whose type has the bunker flag), a garrisonable building in S-0508's sense, or another kind?
   Where do they stand relative to W (cells or a rough position are enough if the cells are to
   stay local)?
2. Are they map entries of the stock map that the harness keeps, or objects the harness adds or
   changes? "In play from tick 0" is listed among the changes: what would the stock game have
   done with them without the harness (for example removed them, or given them to another
   house)?
3. Are the map's other objects of the neutral house (its other buildings and its vehicle) and the
   map's terrain objects also in play from tick 0 in the records' games, as map entries?

## Possible observation
In one game of M-0001 at tick 0, list the objects that neither side owns, with their kind, owner
house and cells, and mark the four the Setup means.

## Current workaround
None needed in the code for now: the arena loads every object of the map as S-0009 gives it, and
the neutral house's buildings are in play from tick 0. No object is added, removed or changed to
make four bunkers. If the answer names objects our tick-0 state lacks, or objects the harness
changes, the arena's setup will follow it then.
