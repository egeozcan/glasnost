---
id: Q-3149
title: Waypoint reveal reaching a house with the whole-map-revealed mark through an ally
status: open
related: [S-0011, S-0409]
---

## Need
S-0011 claim 26 and S-0409 claim 17 say our engine applies the waypoint reveal (action 17) to every
player house whose whole-map-revealed mark is clear, and that the mark "is tested separately for
each house". S-0409 claim 2 says an area reveal of claim 17 counts for house *H* when the house it is
made for is *H*, or when that house is allied with *H* and ally reveal is on.

Our engine now leaves a marked house out of the houses the reveal is made for (#367). When a marked
house *H* is allied with a house *K* whose mark is clear and ally reveal is on, the reveal made for
*K* still counts for *H* through claim 2. The specs do not say whether the mark test of claim 26
also stops that: whether "tested separately for each house" names the houses the reveal is made
for, or every house the reveal would count for.

## Questions
1. When ally reveal is on, does the waypoint reveal made for an unmarked house count for an allied
   house whose whole-map-revealed mark is set?
2. If not, is a marked house the only one left out, so that an unmarked ally of a marked house still
   receives the reveal made for that ally itself?

## Possible observation
Two player houses allied with ally reveal on. Set the whole-map-revealed mark for one of them (a
spy satellite with no gap field on the map, or the defeat reveal) and let regrowth re-cover a disc
around a waypoint for it. Fire action 17 at that waypoint and record whether the disc becomes
explored and seen again for the marked house.

## Current workaround
The reveal is made for each player house whose mark is clear, and it counts for that house's allies
by S-0409 claim 2, a marked ally included. The comment at `triggers::reveal` states this interim.
