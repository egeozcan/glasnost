---
id: Q-1265
title: Does an absorbing building count as carrying passengers for the transport cursor rows
status: open
related: [S-0605, S-0214]
answered-by: []
---

## Need
S-0605 claim 1 is "the docking answer that S-0214's rows x3 and x4 read for an enter order on an
absorbing building". S-0214's transport verdict *x* (and the rows that test whether the clicked
object carries passengers) is worded for transports. The engine needs to know whether an
absorbing building passes the "carries passengers" test that leads to those rows
(`crates/deterministic/sim/src/targeting.rs`, `passengers`).

## Questions
1. Does an absorbing building with a positive capacity reach S-0214's rows x3 and x4 the way a
   transport does? Does one with capacity 0 or below?
2. Does any other row of S-0214 that tests "carries passengers" (for example the rows for a
   vehicle clicked on a transport) apply to an absorbing building?

## Possible observation
Select an own infantry unit and hover over an own absorbing building with capacity 3, then with
capacity 0; record the cursor in each case and the order a click gives.

## Current workaround
A building type carries passengers for S-0214's rows *x* and *q* when it is absorbing and its
capacity is above 0.
