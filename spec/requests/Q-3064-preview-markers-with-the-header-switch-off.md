---
id: Q-3064
title: Where the map selection screen's preview markers go with quirks.preview_markers_from_header off
status: open
related: [S-0009]
answered-by: []
---

## Need
S-0009 claim 25 says the numbered markers over a map's preview come from the header block, and names the
switch `quirks.preview_markers_from_header` for it ("a map whose header disagrees with its start
waypoint markers shows markers in the wrong places"). With the switch on (the default) the client's map
picker draws the header's markers by the Formulas section's preview marker position. With the switch
off, no spec says where the markers go: claim 25 names the switch but not the corrected behaviour. The
client's picker (`crates/app/src/game/install.rs`, `map_info_of`) then draws no markers.

## Questions
1. With the switch off, should the markers be placed from the map's start waypoint markers (S-0009
   claim 23) instead of the header pairs? If so, how is a start waypoint's cell turned into a position
   on the drawn preview: which cell coordinates map to the preview's left and top edges, and which to
   its width and height (for example the map's size entry, its local area, or something else)?
2. With the switch off, which waypoints get a marker and how are they numbered: start waypoints 0 … 7
   present with a value other than −1 (and 0, under `quirks.lobby_counts_zero_waypoint`), labelled
   1 … N in waypoint order, or another rule?
3. Is the marker graphic's corner shift (9, 6) the same in that case?

## Possible observation
None in the original: the switch describes the original's behaviour (on). The answer is a spec decision
for our engine's corrected mode.

## Current workaround
With the switch off, the picker draws no preview markers (`map_info_of` gives no marks). The switch is
on by default, so the default behaviour is the specified one.
