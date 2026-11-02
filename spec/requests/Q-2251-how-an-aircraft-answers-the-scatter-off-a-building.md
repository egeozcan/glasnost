---
id: Q-2251
title: How an aircraft answers the scatter request that capture, sabotage and eaten send off a building
status: answered
related: [S-0215, S-0501, S-0208, S-0207]
answered-by: [S-0501]
---

## Need
S-0215 table T1 gives aircraft a processing of their own for *capture*,
*sabotage* and *eaten* (claim 28). Claim 28's second row (no destination,
any other case) makes the actor fall back to rest (for an aircraft, S-0501
claim 18) and, "if its position is then on a cell that holds a building",
sends it a scatter request with no threat point, forced and not
move-anywhere. Claim 28 then says how an infantry unit (S-0208 claims 11
and 13) and a vehicle (S-0207 claim 16) judge that request and where they
go, but says nothing of an aircraft. S-0215 claim 19 says "What an aircraft
does with a scatter request belongs to the aircraft spec", and S-0501 has no
rule for receiving one: its claim 16 covers only the requests an aircraft's
firing pass sends to others.

An aircraft can stand on a cell that holds a building, for example landed on
a pad. So the case is reachable whenever an aircraft is in *capture*,
*sabotage* or *eaten* (a map entry with that behaviour, or an order that
gives it) with no destination.

## Questions
1. An aircraft with *capture* current, no destination and no target,
   standing on the ground on a cell of a building (a pad it is or is not in
   contact with), is processed. After its fall-back to rest, is it sent the
   scatter request at all?
2. If it is: does it accept it? Does the answer depend on whether it is on
   the ground or in flight, on its contact with the pad, or on its current or
   queued behaviour?
3. If it accepts: what destination does it get (a nearby cell, a landing
   spot, none), does it take off, and does the request change its
   behaviours?
4. Is the answer the same in *sabotage* and in *eaten*?

## Possible observation
A test map places an unarmed, landable, not pad-bound aircraft of the
human-controlled house in *capture* with no destination and no target on a
cell of a plain building of its own house, and a second one on open ground.
Record each one's behaviours, destination, height and cell at every tick for
the first two processings.

## Current workaround
The engine sends the request (claim 28) and an aircraft never accepts it:
`scatter::request` gives no effect for an aircraft, as it does for every
request an aircraft receives. The aircraft keeps what its fall-back to rest
gave it. Test: `claim_28_an_aircraft_in_capture_falls_back_with_the_standard_delay`
(`claims_s0215_capture.rs`).
