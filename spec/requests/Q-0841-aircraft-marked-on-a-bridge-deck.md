---
id: Q-0841
title: When an aircraft counts as marked on a bridge deck for its shadow anchor
status: open
related: [S-0105, S-0501, S-0209]
answered-by: []
---

## Need

S-0105 claim 40 and its term *aircraft shadow height* use "the aircraft is marked on a bridge
deck" (b = 416 leptons, and the threshold a >= 0 instead of a >= 416). No spec says which
aircraft state that is: aircraft are not placed on a deck layer by the movement rules of
S-0501, and S-0209 describes deck marks for ground units. `mapview::live::shadow_anchor` must
decide.

## Questions

1. Is an aircraft that is landed on, or hovering over, a high-bridge cell ever marked on the
   deck, and by which observable state (the deck occupancy record, a layer flag, or its
   landing placement)?
2. If it is, does a flying aircraft crossing the deck keep the mark while its height is above
   the deck?

## Possible observation

Land a rotor aircraft on a deck cell (a pad on a bridge, or a map override) and read the deck
and ground occupancy records of the cell with the read-only recorder of S-0105 MS-30.

## Current workaround

An aircraft counts as marked when its layer is the bridge-deck layer, which no aircraft
movement sets, so only the unmarked threshold (a >= 416) is reachable. Marked Q-0841 in
`docs/development.md`.
