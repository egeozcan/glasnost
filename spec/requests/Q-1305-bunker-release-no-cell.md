---
id: Q-1305
title: Destination and delay of a bunker unload release when no nearby cell is found
status: open
related: [S-0606, S-0212]
answered-by: []
---

## Need
S-0606 claim 12 gives the released vehicle the nearby-cell search result as destination; the Edge cases and Open questions leave the no-result case open, and no processing delay is given for the unload processing.

## Questions
1. When the search finds nothing, what destination does the released vehicle get?
2. What delay does the vehicle's unload processing choose after the release?

## Current workaround
Interim rule: no cell found leaves no destination; the vehicle still queues move. The unload processing chooses the standard delay.
