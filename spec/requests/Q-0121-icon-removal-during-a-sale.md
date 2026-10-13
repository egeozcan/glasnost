---
id: Q-0121
title: Does the start of a sale of the last factory building remove its icons and cancel their items?
status: open
related: [S-0411, S-0407, S-0404]
answered-by: []
---

## Need
S-0411 claim 2 removes an icon, and gives the cancel and cancel-all orders
for its item and queued copies, at the first house update at which the type
is no longer *buildable*; a factory building that is being sold does not
count for buildability (S-0411 Terms). For a sale it says the removal comes
in tick *n* + 1 after the sell order of tick *n*, that is, near the start
of the sale.

S-0407 scenario T14 run 7 expects instead that, when the house's only
barracks is sold while an item of its type is in production, "the icon and
the item stay during the whole sale; in the first house update after the
removal the icon is gone and the item is refunded (S-0404 claim 9)". S-0407
Terms also say the taken-up set, which is the strip's icons, changes only at
a recheck, and the start of a sale is no recheck (claim 12,
`quirks.sale_start_no_recheck`).

The two cannot both hold: with S-0411 claim 2 the item is refunded at the
start of the sale; with S-0407 T14 run 7 it keeps building until the
building is gone. The engine (`sim::econ::sidebar::remove_unbuildable`)
must choose.

## Questions
1. When the house's only factory building of a kind is sold while an item
   of that kind is in production, in which tick is the item refunded and
   its icon removed: at the start of the sale (the first house update
   after the selling behaviour begins), or at the building's removal?
2. Does the item make progress steps during the sale?
3. Is the answer the same when the building is queued to be sold but its
   sale has not started yet?

## Possible observation
S-0407 T14 run 7 with the recorder logging the item's clock, the credits and
the icon list in every tick from the sell order to the removal.

## Current workaround
S-0411 claim 2 as written: buildability is judged at every house update,
and a factory building being sold does not count, so the icon goes and the
orders are given at the first house update after the sale starts. No test
asserts the sale case either way.
