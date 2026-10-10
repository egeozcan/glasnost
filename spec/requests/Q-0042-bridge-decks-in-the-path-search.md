---
id: Q-0042
title: Bridge decks in the path search (the height rule's 4-level step, the reference level, bridge links)
status: open
related: [S-0212, S-0209, S-0207, S-0211]
answered-by: []
---

## Need
`glasnost_sim::path` searches (cell, search layer) pairs (S-0212 claim 7)
so that a vehicle can drive onto a bridge deck from land at deck level
(S-0207 claim 5). Three rules it needs are not stated far enough.

## Questions
1. **Claim 10's 4-level step.** "A difference of 4 is allowed only between a
   bridge deck and the ground at a bridge's end, in the way the bridge
   specs define." Are the two height levels compared those of the two cells
   (the ground level under the deck against the land), as the rest of
   claim 10 reads? Which cells are "the ground at a bridge's end": any cell
   that is not a bridge cell, or only a bridge end as S-0212 Terms define it
   (tile of the bridge or wooden-bridge role, land type not rock, along the
   bridge's axis)? Is the step allowed in both directions (onto and off the
   deck)?
2. **Steps sideways off a deck.** With the cells' height levels compared, a
   step from a deck pair into a neighbouring cell that is not a bridge cell
   and lies at the same height level as the ground under the deck has a
   difference of 0, so claim 10 allows it, and S-0207 claim 5 then takes
   the vehicle off the deck, 4 levels down. Does the original allow that
   step, or does it compare the search layer's level (the cell's plus 4 on
   a deck), which would refuse it?
3. **Reference height level in path costs.** S-0209 leaves open (its Open
   questions) which reference height level the entry outcomes of a path
   search use: the mover's own, or the level of the search pair being
   expanded. It decides which cell layer of a bridge cell the search reads.
4. **Bridge links.** Claim 3 makes the regions of a link's two end cells
   adjacent, and claim 13b adds link connections; which bridge links exist
   and when they are enabled "belongs to the bridge specs". Without links,
   the two banks of a high bridge over water are different reach regions,
   and the gate of claim 6 refuses every order across the bridge. Which
   cells are a high bridge's link end cells, and when is the link enabled?

## Possible observation
1, 2. S-0212 scenario P4 with a high bridge over dry ground of one level:
   order a vehicle on the deck to a cell beside the bridge at the ground's
   level, and one on the land at deck level onto a deck cell that is not the
   bridge's end.
3. S-0209 scenario C22 (a) and (b) with a path search past the cell.
4. S-0212 scenario P8 on a map whose only crossing is a high bridge.

## Current workaround
1. Cell levels are compared; a difference of 4 is allowed when the lower
   cell is a bridge cell taken on its deck and the higher cell is not a
   bridge cell, in both directions (`path::height_ok`).
2. Allowed, by the letter of claim 10.
3. The level of the search pair being expanded (the unit's own level for
   its first step).
4. No links: regions join only by claim 3's neighbour rule.
All four are in the provisional table of `docs/development.md`.
