---
id: Q-0416
title: House colours of map-defined houses outside a skirmish
status: open
related: [S-0103, S-0009, S-0412, S-0413]
answered-by: []
---

## Need

S-0103 claim 10 gives the colour of a player house in a skirmish (its lobby
colour slot) and of the neutral and special houses (light grey). Our map
viewer also draws maps outside a skirmish: every object of every owner,
including objects owned by the map's own houses (countries and houses the
map defines), which no skirmish house carries (S-0009 claim 27). The specs do
not say which colour definition such a house shows. A campaign-style map or
a scripted game would need the same answer. The code is
`crates/app/src/mapview/build.rs` (`owner_colour`) and
`crates/app/src/mapview/load.rs`.

## Questions

1. When a map's objects are owned by a house the map defines (not a skirmish
   player and not the neutral or special house), which colour definition do
   that house's units and buildings show?
2. Does a key in the map's house section or in the country's rules section
   choose it? If so, what does that key hold (a colour name, a definition
   position, or something else), and what is shown when the key is absent
   or names no definition?

## Possible observation

A single-player mission map whose houses carry different colour keys, and a
copy of it with those keys removed. Record each house's unit colours.

## Current workaround

In the viewer's all-owners and sandbox modes, the map's owners other than
the neutral house, the special house and the player placeholders get lobby
colour slots 0, 1, … 7, 0, … in the order in which they first own an object
in the map's object lists (vehicles, infantry, aircraft, buildings), mapped
to definitions by S-0103 claim 10's table. This is our convention, marked
provisional. Listed in docs/development.md (interim rendering rules).
