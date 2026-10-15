---
id: Q-0641
title: How a vehicle reaches and docks at a repair depot or naval yard
status: open
related: [S-0504, S-0314, S-0214, S-0215]
answered-by: []
---

## Need
S-0504 claims 7 and 8 give the naval match of a docking answer and the depot's waiting
gate (stands still, closer than the starting distance to its docking point, settling 3
ticks later). S-0314 claim 6 gives the repair steps once settling has begun. No approved
spec gives the part in between: how a damaged vehicle comes to be *docked* at a depot at
all. Without it the engine cannot let a damaged ship dock at its yard in play. The pure
functions `naval::naval_match_refuses`, `naval::starting_distance`,
`naval::depot_distance` and `naval::waiting_gate` exist, but nothing calls them.

## Questions
1. Which player inputs send a vehicle to a depot or yard (for example a click on the
   building with the vehicle selected), and which orders do they give? Does a vehicle at
   full health get a different answer?
2. What makes a vehicle the depot's *docked vehicle*: at which moment of its approach does
   the depot start its waiting checks? Can a depot have more than one docked vehicle, and
   what does a second vehicle do while one is docked?
3. Where is the docking point at each depot and at each naval yard: which position relative
   to the building, does it depend on the approach side, and what is its height?
4. How does the vehicle move to the docking point: an ordinary move to a cell, or a
   different movement (for example ignoring the building's own cells)? Does a ship go to a
   water cell next to the yard or onto the yard's own cells?
5. What ends docking: full health, the vehicle being given another order, the depot losing
   power, being sold or destroyed, or the vehicle moving away? Where does the vehicle go
   afterwards?
6. Does the AI player send its own damaged vehicles and ships to depots and yards, and on
   which condition?

## Possible observation
Send one damaged stock ship to a stock repairing naval yard, and one damaged land vehicle
to a stock repair depot, by the normal input. Record each tick's positions, orders and
health, the moment health first rises, and the end state. Repeat with two damaged vehicles
sent together, and with a vehicle at full health.

## Current workaround
No docking is built. A damaged ship is not repaired at its yard in play; the end-to-end test
of a ship docking at its yard is not written. The naval match and the waiting gate are
tested as pure functions only (`claim_7_naval_match`, `claim_8_waiting_gate` in
`tests/claims_s0504.rs`).
