---
id: Q-1159
title: S-0605 claim 3's still-driving vehicle against S-0215 claim 11's queued-enter exception
status: answered
related: [S-0605, S-0215]
answered-by: [S-0605]
---

## Need

S-0605 revision 7, claim 3, splits a vehicle ordered in to a vehicle absorber into two cases,
according to whether it is ready to change at its first per-tick start after the order
(S-0215 claim 11). In the case where it is not ready, the claim's example is "a vehicle still
driving from an earlier order". There, *enter* is not current at that start, no approach step
is made, and the vehicle ends the update with contact, *enter* queued and the building as its
destination. Scenario A2c's vehicle variant tests this: "a step in the driving case"
falsifies it.

S-0215 claim 11 makes a moving vehicle not ready to change except in three cases, and the
first of them is "the queued behaviour is *enter*". The order has just queued *enter*. So a
vehicle that is still driving is ready by S-0215 claim 11: *enter* becomes current at that
start and the approach step is made, which is the other case of S-0605 claim 3. The two specs
disagree on the example. Our code follows S-0215 claim 11 (`process::ready`), so in our engine
the driving case makes the approach step. A vehicle that is not ready for another reason (for
example in a deploy or unload state) behaves as S-0605's second case says.

(This file first asked about a "see Flag F1" reference in S-0605 revision 6, claim 3.
Revision 7 removed that reference, so that question is withdrawn.)

## Questions

1. A vehicle is still driving from an earlier move order when an *enter* order on an empty
   vehicle absorber takes effect. In its first update after the order, does it make *enter*
   current at the first per-tick start and take a step toward a cell next to the building, or
   not?
2. If it does not, which condition makes it not ready to change, given that its queued
   behaviour is *enter*?

## Possible observation

S-0605 scenario A2c's vehicle variant, the still-driving case, as written. Record the
vehicle's current and queued behaviour at the start of its first update after the order, and
its position change in that update.

## Current workaround

Interim rule, cited as "Q-1159 interim" in `docs/m7-claims/S-0605.md` and
`docs/development.md` (Simulation, M7a: Infantry absorber): S-0215 claim 11 decides. A
driving vehicle whose queued behaviour is *enter* is ready to change, so it makes *enter*
current and takes the approach step. Only a vehicle that is not ready for another reason makes
no step.
