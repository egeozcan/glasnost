---
id: Q-0716
title: Which bridge and link the repair cursor's high-bridge test looks at
status: answered
related: [S-0511, S-0214, S-0212]
answered-by: [S-0511]
---

## Need

S-0511 claim 21 offers the repair cursor over a hut when a bridge within its 5 × 5 block is
destroyed: for a low bridge when its run holds a destroyed piece, for "an ordinary, unambiguous high
bridge" when "its nearby bridge link is not enabled". It leaves the selection open. Low spans are
also bridge links, never enabled (S-0212 claim 3), so a test on any link near the hut would offer
repair at every intact low bridge. The code in `crates/deterministic/sim/src/targeting.rs` (row
I3) needs a rule.

## Questions

1. Which cells or which bridge does the high test start from: any cell of the block, the hut's cell,
   or the bridge cell of claim 16's table?
2. Which link is "its nearby bridge link": the nearest link, the first in creation order within
   some distance, or another, and at what distance?
3. When the block holds both a low bridge and a high bridge, which test decides?

## Possible observation

S-0511 scenario B13 with a hut placed between a low bridge and a high bridge, and with a hut whose
high bridge runs at distance 3 from its cell.

## Current workaround

When the block holds a low piece of the family of claim 16, the low test decides: the run of that
piece (claim 20's run) holds a destroyed piece. Otherwise claim 16's bridge-cell table from the
hut's cell finds a bridge cell, and the first bridge link in creation order within 3 cells of it
(S-0212 Terms) decides: repair is offered when it is not enabled. No bridge cell or no link: no
repair is offered. The choice is marked in the code with this request's ID.
