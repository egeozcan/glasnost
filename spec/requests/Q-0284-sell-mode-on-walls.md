---
id: Q-0284
title: Which cell does a sell-mode click on a wall sell, and can it be under shroud?
status: open
related: [S-0214, S-0408]
answered-by: []
---

## Need

S-0214 claim 13 gives the sell, repair and power modes' clicks on buildings. S-0408 has the wall
sell order on a cell. The client must turn a sell-mode click into either a sell order on an
object or a wall sell order on a cell, and no spec says how a click that lands on a wall cell (or
on a cell next to a wall drawing) chooses between them.

## Questions

1. In sell mode, does a click on a cell holding the player's wall overlay give the wall sell
   order for that cell? Which cell counts, the cell under the pointer by S-0406 F2 or the one of
   the wall's drawing under the pointer?
2. Is the click refused on a shrouded cell, on another house's wall, or on a cell without a wall?

## Possible observation

In sell mode, click on own and enemy walls, on shrouded walls and next to walls, and record which
walls are sold.

## Current workaround

Interim rule: sell mode gives orders only for objects (`glasnost_ui::modes::order`); a click on a
wall cell gives no order. Recorded in docs/development.md, "Game client (M5b)".
