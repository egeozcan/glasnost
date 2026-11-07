---
id: Q-2746
title: Whether a walker sent to garrison a building keeps its stored path as one under enter
status: answered
related: [S-0208, S-0214, S-0508]
answered-by: [S-0208]
---

## Need

S-0208 r52 claim 6, *A stored path kept under enter*: a walker given a new destination keeps its
stored path when its current or queued behaviour is *enter* and it is in contact with nothing. The
first bullet's example says: "An enter order (into a transport, or into a building to garrison)".

The specs that give the garrison order name another behaviour. S-0214 row I8 gives the order that
sends an infantry unit to garrison a building as *capture (garrison)*, and S-0508 claim 2 has the
occupier arrive "while its behaviour is *capture* and that building is its *destination*". So
going by those specs, a unit sent to garrison is under *capture*, not *enter*, and claim 6's
condition does not hold for it. Claim 6's example and its condition differ for that order.

The engine builds the rule for *enter* only, as the condition reads. The code is
`crate::walk::keeps_stored_path`.

## Questions

1. An infantry unit that walks elsewhere is ordered to garrison an empty civilian building. Does
   it keep its stored path and walk on along its old route first, as one ordered into a transport
   does (S-0208 scenario W17 part f), or does it turn towards the building after the step it is
   on?
2. An infantry unit walking to garrison a building is ordered to a free cell elsewhere. Does it
   walk on towards the building first, as one under *enter* does (scenario W17 part g)?
3. If the answer to either is yes: is it the garrison order that counts as *enter* for this rule,
   or does the unit sent to garrison carry *enter* as its behaviour, against S-0214 row I8 and
   S-0508 claim 2?

## Possible observation

S-0208 scenario W17 parts f and g, with an empty civilian garrison building of the unit's
reach in place of the transport *V*, and the garrison order in place of the enter order. Record
the current and queued behaviour each tick.

## Current workaround

The rule is built for *enter* as claim 6's condition gives (`quirks.enter_keeps_stored_path` on).
A unit sent to garrison (under *capture*) drops its stored path at a new destination, as every
walker not under *enter* does.
