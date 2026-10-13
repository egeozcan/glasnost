---
id: Q-0110
title: Whether a house's own objects count as discovered by it (radar listing), and which object of a cell is marked
status: answered
related: [S-0409, S-0410]
answered-by: [S-0409]
---

## Need
S-0409 (Terms, *discovered by H*) says: "Objects of H count as discovered by
H." S-0410 claim 7 lists an object owned by the shown house only when "it is
discovered by H (S-0409, Terms)", and scenario R9 part A expects an own
vehicle that the player has not yet discovered (placed by a trigger where
the player has not seen) not to be shown until it is discovered. With
S-0409's sentence every own object is always discovered, so R9 part A could
never show a hidden own object. The two specs contradict each other.
`econ::shroud::discovered` and `econ::minimap::listed` need one rule.

A smaller gap in the same term: "Of several objects in one cell, only the
one nearest the cell's centre is marked." It does not say which kinds of
object take part (terrain objects such as trees stand in cells too) or which
one wins when two are equally near.

## Questions
1. Is an object of house H discovered by H from its creation (S-0409 Terms),
   or only once a reveal for H changes the cover piece over its cell, as for
   other houses' objects (S-0410 R9 part A)?
2. If the latter, does an own object that the map places, or that a factory
   produces, inside H's explored area count as discovered at once?
3. Which objects of a cell compete for the mark: every object in the cell
   (trees included), or only infantry, vehicles, aircraft and buildings?
   Which wins a tie of distance?

## Possible observation
S-0410 scenario R9 part A decides question 1. For question 3, a tree at a
cell's centre with an enemy vehicle beside it in the same cell, then the
cell explored, then the vehicle's radar pixel read.

## Current workaround
Our engine follows S-0409 Terms: own objects count as discovered, so every
own object that passes the common tests is listed on the radar map. All
objects in the cell's lists compete for the mark, nearest by squared
distance to the cell's centre, ties to the lower object number.
