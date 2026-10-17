---
id: Q-1266
title: What stops an entrant whose loading request an absorbing building refuses
status: open
related: [S-0605, S-0215]
answered-by: []
---

## Need
S-0605 claim 5 says an entrant refused at a processing "gives up the contact and falls back to
rest", and scenario A6 expects that it "stops and stands" and is falsified "if it goes in".
Under S-0215 claim 14 row 4, falling back to rest with the building still as destination queues
*move*, and under S-0215 claim 11 a walking infantry unit is not ready to change, so *move* never
starts while it walks: the unit walks on in *enter*, reaches the building and goes in under claim
6 (which does not test the switch again). The two cannot both hold. Code:
`crates/deterministic/sim/src/structures/absorb.rs`, `loading_given_up`.

## Questions
1. When an absorbing building refuses an entrant's loading request (claims 3 and 5), is the
   entrant's destination cleared?
2. Does the entrant stop walking at once, at the end of its current step, or only when the next
   behaviour starts?
3. Does the same apply when the refusal comes at the order (claim 3), before the unit has moved?

## Possible observation
Scenario A6, recording destination, behaviour and position per tick from the switch-off until
the unit stands.

## Current workaround
A refused loading request (claims 3 and 5) also clears the entrant's destination and its path
before the fall-back to rest, so that it stands where its current step ends.
