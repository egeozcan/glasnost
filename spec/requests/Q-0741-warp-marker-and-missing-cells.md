---
id: Q-0741
title: The warp source marker's draws, and warp cells that are off the map or unreachable
status: answered
related: [S-0513, S-0202, S-0212]
answered-by: [S-0513]
---

## Need
Three details of the teleport (S-0513 claims 33, 34 and 37) are not given.

1. Claim 33 places the source marker animation, which only the launching player sees and hears.
   S-0202 claim 10 gives every animation creation its draws. If a marker that only one player
   sees took scenario-stream draws, the stream would depend on which player is local. Our engine
   must decide whether the marker is simulation state at all.
2. F1 skips off-map cells of an effect block, and a warp maps source cell *i* to destination cell
   *i*. Near a map edge, source cell *i* can be on the map while destination cell *i* is not.
3. Claim 37 moves a blocked member's landing point to the cell the nearby-cell search finds. It
   does not say what happens when the search finds no cell.

## Questions
1. Does placing the source marker take creation draws from the scenario stream, on every
   machine, as for any other animation? Or is it local to the launching player and draw-free?
2. When destination cell *i* is off the map, what happens to an examined object in source cell *i*?
   Is it left in place, killed, or does it land somewhere else? Is its naval parasite still
   released and an organic object still killed?
3. When a blocked member's nearby-cell search finds no cell, does it keep its landing point and
   check again in its next update, is it killed, or does something else happen?

## Possible observation
1. Repeat S-0513 scenario SW12 part a. Pick a source in a game whose two players record the
   scenario stream, then compare the two streams.
2. Warp a block whose source lies in the map's interior and whose destination centre is on the
   map's edge row. Record the fate of the units in the source cells that map off the map.
3. Warp a vehicle onto a small island ringed by buildings, so that no cell qualifies. Record its
   fate and the ticks.

## Current workaround
Interim rule (docs/development.md, *Simulation (M6b: superweapons)*):
1. The marker is a presentation event for the launching player and takes no draw.
2. Such an object is examined as usual (parasite release, organic kill, the left-in-place tests).
   It is then left in place and does not join the group.
3. The member keeps its landing point, and the check is made again in its next update. It still
   counts as redirected, so its held delay is set.
