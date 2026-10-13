---
id: Q-0171
title: How the items of the auto-deploy delay list are read
status: open
related: [S-0003, S-0215]
answered-by: []
---

## Need

S-0215's Import mapping reads `AIAutoDeployFrameDelay` in `[General]` as a
"list of integers in ticks". S-0003 now says how one integer list is read,
the debris maxima (claim 20: decimal prefixes only, an unreadable item gives
0 and keeps its place, empty items take no place, a 511-byte limit), but no
spec says how the items of this list are read. Request Q-0061 asked only
about the debris maxima, and its answer covers only them. The importer's
general integer-list form (`convert::sim`, `Form::Ints`) has to read this
field somehow.

## Questions

1. Is each item of `AIAutoDeployFrameDelay` read with S-0003 claim 3's
   integer form (with its hexadecimal forms), or as a plain decimal prefix
   as for the debris maxima?
2. Does an item that has no readable number (for example `abc`) give 0 and
   keep its place, or does it take no place, or keep the value held for that
   position?
3. Is the text limit of the value the common 127 bytes?
4. Does an empty item (`5,,7`) take a place?

## Possible observation

Give the neutral house a unit that auto-deploys (S-0215 claim 31) and set
`AIAutoDeployFrameDelay` to `abc,40,90` and then to `$10,40,90`, `10h,40,90`
and `5,,40`. The delay used for difficulty index 1 shows which item stands
at that place.

## Current workaround

Each item is read with S-0003 claim 3's integer form within the common
127-byte limit; empty items take no place; an item the integer form cannot
read gives 0 and keeps its place.
