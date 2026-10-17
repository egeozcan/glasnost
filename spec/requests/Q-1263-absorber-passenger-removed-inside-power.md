---
id: Q-1263
title: Does a passenger removed inside an absorbing building make the owner's power due
status: answered
related: [S-0605, S-0405, S-0414]
answered-by: [S-0605]
---

## Need
S-0605 claim 15 lists removal of a passenger while it is inside (for example its own house's
defeat) as a change of the passenger count, and leaves open whether the building owner's power
becomes due (Open questions). The engine has to decide when the house totals drop
(`crates/deterministic/sim/src/structures/absorb.rs`, `removing`).

## Questions
1. When a passenger leaves the game inside, does the building owner's power output fall at that
   owner's next house update, or only at a later recalculation caused by something else?

## Possible observation
Scenario A11 of S-0605: an absorbing building holding a controlled enemy passenger; defeat the
passenger's house and record the building owner's power output per tick until something else
changes it.

## Current workaround
The removal makes the building owner's power due, as a release does (only when the type's extra
output is above 0).
