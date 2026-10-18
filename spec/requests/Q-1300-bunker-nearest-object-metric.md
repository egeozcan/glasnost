---
id: Q-1300
title: Which objects a tank bunker's clearing rows examine, and how "nearest the cell centre" is measured
status: answered
related: [S-0606, S-0215]
answered-by: [S-0606]
---

## Need
S-0606 claim 6 and table S1 send the clearing row's scatter requests to "the object nearest the centre of each foundation cell" and have the waiting row wait until that object is absent or is the entering vehicle. The spec does not say which objects take part (infantry, vehicles, aircraft on the ground, terrain objects, overlays, other buildings, a dead object still listed) or how the distance is measured (horizontal or spatial, table root or exact, and which object wins a tie).

## Questions
1. Do terrain objects, other buildings or wreck-like objects standing on a foundation cell count as "the object nearest the centre", so that they get the scatter request or hold up the waiting row?
2. Is the distance horizontal or does height count (for example a landed or low aircraft on the cell)?
3. When two objects are equally near the centre (for example two infantry at mirrored positions), which one is examined?

## Possible observation
Put a terrain object, then a landed aircraft, then two infantry at mirrored positions on one foundation cell of an empty 2 x 2 bunker and send a vehicle in; note which ones move and whether the vehicle waits.

## Current workaround
Interim rule: only live, on-map infantry, vehicles and aircraft listed in the cell's ground layer take part, the bunker excepted; the distance is the table square root of the horizontal distance to the cell centre; a tie keeps the object listed first. Off switch (`quirks.bunker_clear_nearest_only` off): every such object except the entering vehicle.
