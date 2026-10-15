---
id: Q-0647
title: Whether a skirmish house can build a naval yard whose tech level is above the game's tech level
status: open
related: [S-0009, S-0407, S-0412, S-0504]
answered-by: []
---

## Need

S-0009 Terms make the game's tech level a copy of the base rules' multiplayer tech level, with no
lobby control, and give every lobby house that tech level. S-0407 claim 5 (a) refuses a type whose
tech level is above the house's tech level. With a local install's data, the AI lane (counts
only) finds that two of the three naval yard types have a tech level above the game's tech level,
so our engine refuses them to every skirmish house. Only the third side's yard can be built, and
two of the three sides never get ships. Few other buildable types are affected.

That follows the specs as written, but it decides whether a naval map is playable for most
sides, so we would like it confirmed before we treat it as final. The gap, if there is one, is in
`glasnost_sim::econ::tech::house_tech_level` and `other_conditions_refuse`.

## Questions

1. In a skirmish game started from the setup screen, does the sidebar offer each side's naval
   yard once its listed prerequisites stand, on a map with water?
2. If it does, what tech level does a lobby house compare a type's tech level with: the game's
   tech level of S-0009, or another value (and where does that value come from)?
3. Does the answer differ between a human player's house and a computer player's house?

## Possible observation

Start a skirmish on a stock map with water as each side in turn. Build power, a refinery and the
other prerequisites of the side's naval yard, and note whether the yard's icon appears.

## Current workaround

None needed: the code follows S-0009 and S-0407 as approved. Our AI builds a navy only for a side
whose naval yard is not refused. The local lane's per-side games (`glasnost-headless ai-sides`)
show which sides do.
