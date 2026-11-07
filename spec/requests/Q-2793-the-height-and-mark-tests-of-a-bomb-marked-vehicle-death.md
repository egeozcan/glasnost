---
id: Q-2793
title: The height and mark tests of a bomb-marked vehicle's death
status: open
related: [S-0312, S-0511, S-0503]
answered-by: []
---

## Need

S-0312 claim 11 gives the death effects of a vehicle that carries the bomb mark and "dies within
10 leptons above a water cell", and the table of claim 12 has a column "Bomb mark (claim 11)" whose
third row kills the passengers of a marked vehicle that cannot crash. Our engine now builds claim
11 (`combat::death::kill` and the defeat sweep's repeat, `combat::death::repeat`). Two readings
are not settled by the text, and our engine had to pick one for each:

- **Where the height is measured from.** Claim 11 measures from "a water cell". Claim 12 measures
  "above the terrain". Our engine's usual *height above the surface* is measured from the bridge
  deck for an object on the deck layer (S-0307 Constants, 416 leptons). A vehicle on a bridge deck
  over water stands about 416 leptons above the water but 0 above its own surface. The text also
  does not say whether a vehicle below the water surface (a negative height, as during a sinking
  descent, S-0504 claim 4) is "within 10 leptons above" it.
- **Which marked vehicles claim 12's third row covers.** The column is headed "Bomb mark (claim
  11)". It could mean every vehicle that carries the mark (Terms), wherever it dies, or only one
  that claim 11 covers, which dies within 10 leptons above a water cell. Before this build our
  engine already killed the passengers of every marked vehicle that cannot crash, wherever it
  died.

## Questions

1. For claim 11's test, is the height of a dying vehicle measured above the terrain surface at
   its position, or above the surface it stands on (the bridge deck for a vehicle on the deck
   layer)? So does a bomb-marked vehicle that dies on an intact bridge deck over water take claim
   11's effects or the ordinary ones? Is the cell's land type that of the ground under the deck?
2. Is a vehicle whose height above the water surface is negative "within 10 leptons above" the
   water cell, so that claim 11 applies to it when it carries the mark?
3. Does claim 12's third row (a vehicle with the bomb mark that cannot crash, 208 leptons or less
   above the terrain: all passengers die) apply to a marked vehicle that dies on land, or more than
   10 leptons above water, where claim 11 does not apply? Or do such a vehicle's passengers get out
   as in the fourth row?

## Possible observation

S-0312 scenario D8's marked-water variant already drops a loaded amphibious transport from a
collapsing bridge deck over water (S-0511 scenario B3). Repeat it with the bridge over land, so
that the transport lands on the ground with the mark and dies there by its landing hit (S-0511
claim 10): record whether each passenger dies or gets out. For question 1, give a bomb-marked
vehicle that survived an earlier landing on water (S-0511 claim 10: the mark stays) a route onto an
intact bridge over water and kill it there with an ordinary weapon: record whether it shows the
wake and the last splash entry or its explosion and destruction animations, and whether debris and
a death weapon appear. Question 2 needs a marked vessel that is sinking, for which no stock setup is
known; if none can be made, say so.

## Current workaround

Claim 11 applies when the vehicle carries the mark, the cell at its position is water (S-0312
claim 10's test) and its height above the terrain surface at its position (not the deck) is 10
leptons or less, negative heights included. Claim 12's third row applies to every marked vehicle
that cannot crash, as before this build.
