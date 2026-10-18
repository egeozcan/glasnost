---
id: Q-1162
title: S-0314 claim 17's rotation gate, the moment at which the completed-entry exchange starts a tethered vehicle's rotation after movement arrival
status: answered
related: [S-0314, S-0215]
answered-by: [S-0314]
---

## Need

S-0314 revision 27, claim 17, says that for a tethered vehicle *sleep* is queued from movement
arrival "before the completed-entry exchange rotates the vehicle toward 16384 facing units", and
that *sleep* does not become current while the body is still rotating, because a rotating body
counts as moving for readiness (S-0215 claim 11). *Sleep* becomes current at the first start
opportunity after the rotation ends.

In our engine the completed-entry exchange belongs to the vehicle's *enter* processing (S-0215
claim 29), which runs only on the vehicle's own processing ticks, with the standard delay between
them. Movement arrival happens inside the locomotor's update. If the rotation waits for the next
*enter* processing, the late start of the arrival tick sees a vehicle at rest that is not turning,
and *sleep* becomes current there. The *enter* processing then never runs again, so the vehicle
is never rotated. The rotation gate can only hold if the rotation has begun by the late start
that follows the arrival. The spec does not say when the exchange's rotation begins relative to
the arrival.

## Questions

1. A tethered vehicle in *enter* reaches its depot's docking-point cell during movement, with a
   body facing other than 16384. Does its body begin turning toward 16384 in the same tick as
   the arrival, before the late start? If not, when does the turn begin, and what stops *sleep*
   from becoming current at that late start?
2. Does the answer depend on the locomotor? S-0215 claim 11's turning clause covers the driving
   locomotor only, so a ship or hovering vehicle that turns would not be held by it.

## Possible observation

S-0314's depot scenario with a tethered vehicle (a pad depot, or the place-0 exchange) whose
approach ends with a body facing far from 16384, for example arriving from the east. Read the
body facing, the current and the queued behaviour after every tick from arrival until *sleep* is
current.

## Current workaround

Interim rule, cited as "Q-1162 interim" in `docs/m7-claims/step1k-m7a.md` and
`docs/development.md` (Simulation, M7a: S-0215 revision 63). Movement arrival
(`depot::movement_arrival`) starts the tethered vehicle's turn toward 16384 at its type's rate,
right after it queues *sleep*, when the body facing differs. The late start of that tick sees a
driving vehicle's body turning, so *sleep* waits for the turn to end. An untethered vehicle is
not turned. Test: `claim_17_rotation_gate_on_the_arrival_sleep`.
