---
id: Q-3261
title: An owner change of a building in its factory break-off window
status: answered
related: [S-0406, S-0309, S-0215]
answered-by: [S-0406, S-0309]
---

## Need

S-0406 claim 17 (r19) gives the building a place order names a factory break-off window. It lasts
from the end of its update on its first turn *T* + 1 to its first construction processing *C*.
In the window the building is out of its construction state, so the first per-tick start of its
turn is not withheld. When its construction range sets the ready mark on *T* + 1 (a zero-delay
construction range: no build-up sprite), *construction* is current from that turn's second start,
and *C* = *T* + 2. So the building has *construction* current and not yet processed between its
turn in *T* + 1 and its turn in *T* + 2.

S-0309 claim 31 has a row for an owner change of a building whose *construction* is current and
not yet processed: "its first processing restarts the build-up, an immediate change that cancels
the idle-body request; guard current at that turn's second start". Before r19 the first start was
withheld then, so *construction* was processed before *guard* could become current.

In the window the first start is not withheld. The change's grand opening sets the ready mark and
its guard request queues *guard*. Read as they stand, the first start of the building's turn in
*T* + 2 makes *guard* current before *construction* is processed. Then there is no restart, the
idle body that the break-off gave stays, and with *construction* neither current nor queued and an
idle body there, S-0309 claim 31's end-of-turn rule ends the build-up at the end of that turn. S-0406 Open questions item 10 says
whether any rule other than those it lists treats the building differently in that window was not
established.

## Questions

1. A building a place order names has *construction* current and not yet processed in its factory
   break-off window, and changes owner (S-0309 claim 31). Does the first start of its next turn
   make *guard* current, with no construction processing and no restart? Or does *construction*
   get its first processing first, as S-0309 claim 31's row gives?
2. Does the idle body that the break-off gave stay, or does something replace it?
3. Is the same true for an owner change while *construction* is still queued in the window (S-0309
   claim 31's first row: the guard request replaces the queued *construction*)?

## Possible observation

A synthetic building type with no build-up sprite and a free unit, placed by a place order; change
its owner by a trigger in the orders part of *T* + 1 or between the building's turns in *T* + 1
and *T* + 2. Record its current and queued behaviour, its drawn body and whether a free unit
appears, each tick from *T* to *T* + 5.

## Current workaround

The engine follows S-0406 claim 17 as it reads: in the window the first start applies, so *guard*
becomes current at the first start of the next turn and *construction* is not processed. The claim
test of S-0309 claim 31's row (construction current and not yet processed) sets that state up
outside the window.
