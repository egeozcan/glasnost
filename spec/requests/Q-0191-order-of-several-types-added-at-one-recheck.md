---
id: Q-0191
title: In which order are several types added to the sidebar when one recheck offers them together?
status: answered
related: [S-0004, S-0407, S-0411]
answered-by: [S-0407]
---

## Need
S-0004 claim 11 places each newly available item by insertion into its tab, and a full tie on
every key puts the newly added item first (`quirks.sidebar_tie_newest_first`). When one recheck
(S-0407 claims 1 and 12), including the first one at game start, adds several types to the
taken-up set at once, the final icon order of tied items depends on the order in which they
are added. S-0411 claim 13 then gives the cancellation orders in that icon order, so the order
also decides which automatic cancellation comes first (and whether an intermediate start and
its creation draw happen). No spec states the order of addition within one recheck. The
engine decides it in `econ::tech::recheck`.

## Questions
1. When one recheck makes two types of the same tab available, tied on every S-0004 claim 11
   key (same category, tech level, cost and display name), which icon is shown first?
2. Is the order of addition within one recheck by kind (infantry, vehicles, aircraft,
   buildings), by registration index within a kind, or something else?
3. Is it the same at the game-start recheck as at a recheck later in the game (for example,
   when one new building enables two types at once)?

## Possible observation
S-0004 scenario F2 with two identical copies of a type that become available at the same
moment, once at game start and once through a newly placed prerequisite building; record the
icon order.

## Current workaround
The types a recheck adds are inserted one by one in set order: kind (infantry, vehicles,
aircraft, buildings), then registration index. Marked `Q-0191` in `econ::tech::recheck`.
