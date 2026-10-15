---
id: Q-0707
title: Whether a destroyed vehicle counts in its escaping passengers' entry test
status: answered
related: [S-0312, S-0507, S-0209]
answered-by: [S-0312]
---

## Need

S-0312 claim 12 lets the passengers of a destroyed vehicle get out at its position, but a
passenger dies instead "when it cannot enter the vehicle's cell (an entry outcome other than
clear or moving blocker, S-0209 claim 4)". The claim handles passengers before the vehicle leaves
play, so the vehicle is still listed in that cell. Read literally, S-0209's entry test then finds
a stationary friendly vehicle there (outcome 6), and every passenger dies on any ground, which
contradicts the claim's high-confidence escape (scenario D9).

The code in `crates/deterministic/sim/src/transport.rs` (`escape`) needs the rule.

## Questions

1. Is the destroyed vehicle itself left out of its passengers' entry test (for example because it
   has already left its cell's list, or because a destroyed object does not count)?
2. If it is counted, which outcome does the test give for it?

## Possible observation

S-0312 scenario D9 on open flat ground: the passengers get out; repeat with another stationary
friendly vehicle in the same cell (if the map allows) to see whether other occupants still count.

## Current workaround

The destroyed vehicle is taken out of its cell's list while its passengers get out, and put back
at the same place afterwards; every other occupant counts as usual. The choice is marked in the
code with this request's ID.
