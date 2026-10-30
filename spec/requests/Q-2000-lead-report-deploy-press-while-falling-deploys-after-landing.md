---
id: Q-2000
title: The lead's play report against S-0213's deploy rule and S-0522 claim 17: a deploy press on falling paratroopers deploys them after landing
status: answered
related: [S-0213, S-0214, S-0522, S-0107, S-0215]
answered-by: [S-0522]
---

## Need

The project lead reports from play (M8 step A4, round 5, note 32, 2026-10-28): "when I press D to
deploy paratrooping units, they should deploy after landing". In our game a deploy press made
while the selected paratroopers are still falling does nothing, and they stay undeployed after
they land.

The approved specs, as written, give what our game does:

- S-0213 table T2's *deploy rule* gives a selected unit a deploy order only if it "is movable or
  able to fire".
- S-0214 Terms: *movable* and *able to fire* both require that the object is not falling ("for
  example under a parachute").
- S-0522 claim 17: "Player move clicks while falling are refused because the unit is not movable
  ... They are not saved as moves to execute after landing", and "Landing does not issue, replace
  or clear an order, target or destination."

So a press during the fall gives a falling unit no deploy order, nothing is saved, and landing
issues nothing. Our engine follows these (`targeting::key_command`, `deploy_command`, `movable`
and `able_to_fire` in crates/deterministic/sim/src/targeting.rs; the test
`claim17_a_deploy_press_while_falling_gives_no_order_and_nothing_follows_at_landing` in
crates/deterministic/sim/tests/claims_s0522.rs). S-0522 claim 17's measured backing (scenario
P10) covers move clicks; nothing in the specs states a deploy press during the fall was
observed. The specs and the lead's report cannot both be right for the deploy press.

## Questions

1. A player selects infantry of an in-place deployer type (S-0214 Terms) while they fall under
   their parachutes and presses the deploy key once. Do they deploy after they land, without a
   second press? If so, when: in the tick they land, or once the queued rest behaviour of S-0522
   claim 17 becomes current, or later?
2. If they do: is the press taken as a deploy order at the press (so S-0213's deploy rule, or
   S-0214's *movable* and *able to fire*, do not exclude a falling unit for the deploy press), or
   is it saved and given at landing? Does the deploy check and deploy readiness of S-0214 apply
   at the press or at landing?
3. Does a move click or an attack click during the fall behave the same way as the deploy press
   (saved for landing, or refused), or does only the deploy press carry over? S-0522 claim 17
   says move clicks are refused; please confirm that still holds beside the deploy answer.
4. Mixed selection: falling paratroopers selected together with landed in-place deployers of the
   same type that are not deployed. One press: do the landed ones deploy at once and the falling
   ones after they land? Does the deploy rule's mixed-selection exception (deployed members get
   no order) count a falling member as deployed or not deployed?
5. A second press while still falling (or a stop order, S-0213): does it cancel the pending
   deploy?

## Possible observation

S-0522 scenario P10 with a paradrop whose payload is an in-place deployer infantry type (test data
if no stock payload type deploys in place): select the paratroopers during the fall, press the
deploy key once at a recorded tick, and record each unit's landing tick, its sequence (S-0107)
and its current and queued behaviour each tick for 120 ticks after landing. Repeat with the press
after landing as a control, with a move click during the fall, and with a mixed selection of
falling and landed deployers.

## Current workaround

The claims as written: a deploy press while falling gives the falling members no order, nothing
is saved for landing, and landing issues no order. The player presses the deploy key again once
the units have landed. No invented saved order.
