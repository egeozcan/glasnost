---
id: Q-1158
title: S-0314 claim 23 revision 25 for a docked vehicle's enter order on its own depot, and who gives the destination when the repeated request takes a free place
status: answered
related: [S-0314, S-0215]
answered-by: [S-0314]
---

## Need

S-0314 revision 25, claim 23, first bullet: "for a later enter order on a depot that holds
*any* contact", a damaged vehicle's order queues *enter* without an immediate start, records
the depot as the object the vehicle is entering and gives it a queued entry. *Enter* then
becomes current at the vehicle's next ready start and repeats its entry request.

Two cases are not clear to us.

1. **A docked vehicle given an enter order on its own depot.** Claim 20 (unchanged) says
   this "ends the docking. A damaged vehicle takes claim 23's queued entry, and at its next
   eligible retry the permission refusal for a vehicle standing on the depot (claim 16)
   clears its contact and queued entry". The depot holds a contact (the vehicle's own), so
   the first bullet of claim 23 could apply too. If it does, *enter* becomes current in the
   vehicle's next turn and its processing runs before that turn's retry. The vehicle is still
   tethered, so that processing is the completed-entry exchange (claim 17), which queues
   *repair* on the depot and *sleep* on the vehicle. Only then does the retry's refusal clear
   contact and the queued entry.
2. **A free place taken by the repeated request.** Claim 23 says: "If a place is free when a
   processing runs, that request takes the contact; the retry below, which comes later in the
   same turn, then finds the contact already held and goes on to ask permission." Granted,
   the retry "assigns the depot as destination". Claim 18 says, for an accepted request with
   contact, that the depot directs the vehicle toward the building as its destination. If
   claim 18 applies in that processing, the vehicle can move in that turn's movement before
   the retry runs.

Our code: `depot::order_applied` (claim 20's branch and claim 23's) and `depot::process_enter`
(`from_memory`).

## Questions

1. After an enter order on its own depot, does a damaged vehicle docked at that depot get
   *enter* queued, and is it current in the next tick? Does the depot queue *repair* again,
   or does the vehicle queue *sleep* again, before the vehicle loses contact?
2. In a turn where a queued vehicle's *enter* processing takes a free place, does the vehicle
   get the depot as its destination in that processing, so that it moves in that same turn,
   or only from the retry later in the turn, so that it does not move until the next turn?

## Possible observation

H18 a, and H18 f(ii) at a two-place depot that holds one contact (or at the moment a
one-place depot's first vehicle leaves). Record, every tick, the vehicle's current and queued
behaviour, its destination and its position, and the depot's current and queued behaviour.

## Current workaround

Interim rules, cited as "Q-1158 interim" in `docs/m6-claims/depot-cycle.md` and
`docs/development.md` (Simulation, M6b: depot cycle):

1. Claim 20's branch is unchanged. A docked vehicle given an enter order on its own depot gets
   the queued entry only: no *enter* queued and no depot remembered. The retry's refusal then
   clears contact and the queued entry (`claim_20_same_depot_enter_while_docked`).
2. The processing that takes a free place gives no destination. The retry grants and assigns
   the depot, so the vehicle does not move until the next turn
   (`claim_23_free_place_taken_by_the_processing`).
