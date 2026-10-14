---
id: Q-0201
title: The height of a walker's locomotor destination for a cell destination with a bridge deck (S-0208 claim 16)
status: answered
related: [S-0208, S-0209, S-0212]
answered-by: [S-0208]
---

## Need
S-0208 claim 16 (revision 14) ends a walk on an arrival tick when the walker's
new position is in the same cell as its locomotor destination and the two
heights differ by less than two height levels (|Δz| < 208). For a cell
destination, the locomotor destination is "a point in that cell", but which
point, and at what height, is not given; the spec's Open questions say that
how a walker's locomotor destination is placed on a high-bridge cell was not
traced. Our destinations are cells without a layer, so the engine must decide
the height it compares with when the destination cell carries a bridge deck
and the walker arrives on the deck, or on the ground under it
(`glasnost_sim::walk`, the end-of-walk test).

## Questions
1. For a walker ordered onto a high-bridge cell, at what height is its
   locomotor destination: the ground surface, the deck (416 leptons above),
   or the height of the layer the walker will stand on?
2. Can a walker arrive in its destination cell and walk on because the height
   differs by two levels or more (for example ordered onto the ground under a
   bridge and arriving on the deck, or the reverse)? What does it do then?

## Possible observation
S-0208 scenario W10 part d with the ordered cell under a high bridge, ordered
once from the deck and once from the ground; record the walker's height,
destination and speed fraction on each arrival tick.

## Current workaround
For a cell destination the end-of-walk test takes the locomotor destination
at the height of the surface the walker stands on in that cell (the deck when
it is on the deck), so the height condition holds in its destination cell. An
object destination is compared with the object's own height. Interim row in
docs/development.md.
