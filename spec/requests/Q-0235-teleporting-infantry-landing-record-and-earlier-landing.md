---
id: Q-0235
title: Which occupancy record teleporting infantry land in on a high bridge, and what a refused new destination does to an earlier landing
status: answered
related: [S-0208, S-0207, S-0209]
answered-by: [S-0208]
---

## Need

S-0208 claim 19 gives an infantry unit with the teleporting locomotor a standing position as
its landing point, chosen by claim 3's table "for the surface the walker is on", and says the
unit reserves that position. It gives a standing-position deck mode for the height of the
point (a high-bridge cell and more than 312 leptons above the surface), and it says the entry
check reads the deck's record on a high-bridge cell. S-0207 claim 20 (e) puts the jumping
object on the deck of a high-bridge cell and says it stays in the ground layer. Our engine
keeps separate ground and deck records and listings, and needs to know which record the
position choice reads and the reservation goes into, and on which surface the unit is
listed after landing. This is in `teleport::infantry_landing` and `teleport::jump`.

Claim 19 also says that a refused landing choice gives no jump and removes the destination.
S-0207 claim 20 (b) keeps an earlier, not yet jumped-to landing point for a non-infantry
object when a new destination finds no cell. Neither says what happens to an infantry unit's
reservation for an earlier destination when a new destination is refused before the jump.
This is in `teleport::take_infantry_destination`.

## Questions

1. On a high-bridge destination cell, does the position choice of a teleporting infantry
   unit read the deck's occupancy record or the ground's? Does it depend on the
   standing-position deck mode (the unit's height more than 312 leptons above the surface),
   or only on the cell carrying a high bridge?
2. Is the chosen standing position reserved in that same record?
3. After the jump onto a high-bridge cell, is the unit on the bridge deck for later walking,
   scatter and occupancy purposes, or on the ground surface below with a deck height?
4. When a teleporting infantry unit has reserved a landing position for one destination and,
   before it jumps, gets a new destination whose landing choice is refused: does it keep the
   earlier landing point and jump to it, or does it give it up? If it gives it up, does its
   occupancy entry go back to the position it stands on?

## Possible observation

S-0208 scenario W11 part d on a high bridge: record the deck and ground occupancy records of
the destination cell after the order, and the unit's surface after the jump, with departure
heights above and below the 312-lepton boundary. For question 4, give two move orders on the
same tick, the second to a fully occupied cell, and record the landing and the occupancy
records on the next ticks.

## Current workaround

The position choice reads, and the reservation goes into, the deck record when the cell
carries a high bridge and the ground record otherwise, the same rule as the entry check. After
the jump the unit is listed in that layer. A refused new destination gives up any earlier
landing point, and the unit's entry goes back to the position it stands on. Interim row in
`docs/development.md`.
