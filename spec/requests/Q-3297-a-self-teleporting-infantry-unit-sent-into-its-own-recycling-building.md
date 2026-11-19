---
id: Q-3297
title: A self-teleporting infantry unit sent into its own recycling building
status: open
related: [S-0603, S-0214, S-0213, S-0215, S-0207, S-0208, S-0209]
answered-by: []
---

## Need

Issue 455: the lead reports that an armed self-teleporting infantry unit (S-0214 Terms) cannot
be sent into its own recycling building, while other infantry can. The lead saw no recycle
action over the building. The engine must decide what the recycle order does for such a unit.

What the claims give, read together:

- **The action.** Over an own recycling building, *g* is select (S-0214 table G row G10). Row
  G8's attack rule fails on an own building without force-fire, and row G9 needs a target of
  another house. *g*₁ = *g*, because value *e* is undefined (the building is not a hospital).
  *v* = *g*₁, and *g*₂ = *v*, because value *x* is undefined (the building has no passengers).
  Row I26 then gives recycle (S-0603 claim 1). No row tests the teleporter flag or a temporal
  weapon. While the unit phases in after its own jump, the action is still recycle. The click
  gives no order then, because table M's recycle needs a movable actor and a phasing-in unit
  is not movable (S-0214 Terms). Once phasing in ends, the click gives table M's order:
  *eaten*, the building as destination, no target.
- **The landing point.** The order queues *eaten* without an immediate start and sets the
  destination at once (S-0213 claim 5, table T3 parts d and f; S-0215 claim 12). The
  teleporting locomotor takes the destination at once and fixes its landing point (S-0207
  claim 20 a). S-0208 claim 19 says "a ready queued behaviour can start when the landing point
  is chosen (S-0215)". But S-0215 gives only the per-tick start and the late start of claim
  11, and claim 12 says an order's behaviour has no immediate start. So when the point is
  chosen, the rest behaviour (*guard*) is still current and *eaten* is only queued.
  - Occupants are therefore not ignored (the *eaten* exception of S-0208 claim 19 does not
    hold).
  - The building is not the unit's mission target (S-0209 Terms: only while in a behaviour
    that has one), so row WP3 does not apply.
  - The own building refuses entry by row WP19.
  - The object destination's nearby-cell search (S-0208 claim 19) then picks a cell beside
    the foundation.
- **After the jump.** On the next tick *eaten* becomes current at the first per-tick start
  and is processed. The unit then jumps beside the building, and the jump removes its
  destination (S-0207 claim 20 e). With no destination, *eaten* falls back to rest (S-0603
  claim 3). The unit never arrives in a cell that holds the building, so it is never
  consumed. A second order meets the same rules, so it does not reach the foundation either.
- **Consumption, if it did land on the foundation.** S-0603 claim 5 consumes an infantry unit
  "at the arrival update of a walking step (S-0208 claim 22)". S-0208 claim 22's second
  paragraph and S-0207 claims 20 e and 22 give a teleporting unit an arrival update of its own
  kind in the jump tick. S-0207 claim 20 e says "the respective movement and combat specs own
  those effects". No claim says whether consumption at a recycling building is one of the
  effects of a teleporting infantry unit's arrival update. The same is unsettled for a
  vehicle with the teleporting locomotor: S-0603 claim 5 names "a cell-arrival update
  (S-0417 claim 7)", and S-0207 claim 22 calls the jump tick's update an arrival update of
  any object kind.

Where the code stands: `targeting::infantry_on` (row I26) gives recycle, as above.
`teleport::infantry_landing` chooses the landing point with the behaviour current at the
assignment, as above. `teleport::jump` runs no recycling-building consumption for either kind.
A walking infantry unit, by contrast, is consumed (S-0603 claims 4 and 5).

## Questions

1. Does an armed self-teleporting infantry unit get the recycle action over its own recycling
   building, whether at rest, while phasing in after its own jump, or while moving? If not,
   which state or type property takes it away, and what does it get instead?
2. After the recycle order, where does such a unit land? Beside the building, as the claims
   give above, or on a foundation cell? If on a foundation cell, what makes the *eaten*
   exception and row WP3 apply at that moment, when *eaten* is only queued?
3. A teleporting infantry unit whose current behaviour is *eaten* lands in a cell that holds
   the building it is bound for. Is it consumed in the jump tick, as at a walking step's
   arrival? If so, where does consumption come among that tick's effects (S-0207 claim 20 e:
   arrival reporting, entered-by tag, crate pickup, destination removal, the landing warp
   animation)?
4. The same question as 3 for a vehicle with the teleporting locomotor (outside S-0207 claim
   28's docking assignments) sent into an accepting recycling building.
5. If the unit lands beside the building and is not consumed, does it end in *guard* with no
   destination, as the claims give, or does it try again?

## Possible observation

An own recycling building on open ground, with an armed self-teleporting infantry unit about 5
cells away, at rest. Read the cursor over the building. Then give the recycle order and record,
each tick, the unit's cell, its current and queued behaviour, its destination and whether it
leaves the game. Repeat with a plain walking infantry unit as the control, and with the
teleporting unit standing still in its phasing-in period just after a move order. For question 4,
the same with a vehicle of a test rules file that gives it the teleporting locomotor.

## Current workaround

Today's behaviour, as the claims give it above: the action is recycle, and the click gives no
order while the unit phases in. The landing point is chosen with the rest behaviour current, so
the unit lands beside the building, falls back to *guard* and is not consumed. The jump consumes
no teleporting entrant of either kind. Guard tests:
`claims_s0603::claim_1_issue_455_a_self_teleporting_infantry_gets_recycle` and
`claims_s0603::claims_3_5_issue_455_a_self_teleporting_entrant_lands_beside_and_stays`.
