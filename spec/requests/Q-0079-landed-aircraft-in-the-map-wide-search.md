---
id: Q-0079
title: Landed aircraft in the map-wide search (both passes, and the distance unit of the score)
status: answered
related: [S-0309]
answered-by: [S-0309]
---

## Need
S-0309 claim 7 says the map-wide search considers "every aircraft on the map
when it looks for aircraft, and then every infantry unit, vehicle, aircraft and
building whose render layer is the ground layer". A landed aircraft is in the
ground layer, so it seems to be considered in both passes. F4 then gives the
distance *d* "in cells ... for aircraft in the map-wide search, but in leptons
for the ground-layer objects of the map-wide search", and a landed aircraft is
both. The engine has to decide, in `combat::acquire::search` and
`combat::acquire::score`, which unit to use, and whether one aircraft can count
as two candidates.

## Questions
1. Does a map-wide search whose searcher looks for aircraft consider a landed
   aircraft twice, once in each pass, or only once? If once, in which pass?
2. When a landed aircraft is scored in the second pass (the ground-layer pass),
   is its distance *d* of F4 in leptons, like the other ground-layer objects, or
   in cells, like the aircraft of the first pass?
3. Does a searcher that does not look for aircraft (no anti-air weapon), but
   looks for vehicles, find a landed aircraft in the map-wide search? Claim 5
   says a search for vehicles accepts aircraft.

## Possible observation
S-0309 scenario A11 with a hunting vehicle whose type has a non-zero distance
coefficient and a gun that is not anti-air, and two enemy targets: a landed
aircraft on a helipad and a vehicle, placed so that the pick differs depending
on whether the aircraft's *d* is in cells or in leptons. Then the same with an
anti-air searcher.

## Current workaround
The second pass considers landed aircraft (every object is in the ground layer
in our engine), so a searcher that looks for vehicles finds them. An aircraft
is scored with *d* in cells in either pass, and a searcher that looks for
aircraft considers a landed aircraft in both passes (the pick is the same
object either way; only a tie with another candidate could depend on it).
