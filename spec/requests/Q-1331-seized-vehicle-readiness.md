---
id: Q-1331
title: Readiness of a seized driving vehicle
status: answered
related: [S-0607, S-0215]
answered-by: [S-0607]
---

## Need
S-0215 claim 11 (revision 67) gives vehicles with the driving locomotor their own "moving" predicate for readiness: the body still turning, or the locomotor moving it with a heading point and a movement step above 0. Other locomotors (ship, hovering, jumpjet and others) use "the locomotor moves it". S-0607 claim 5 links the jumpjet locomotor to a seized unit until claim 20 gives it its own locomotor back. Neither spec says which predicate a seized vehicle whose type has the driving locomotor uses. The answer matters for claim 20: a set-down seized vehicle that is given a move order falls back to rest only when it is ready, and the jumpjet body can still be turning then.

## Questions
1. While seized, does a vehicle whose type has the driving locomotor use the jumpjet locomotor's "moving" (the locomotor moves it), and not the driving predicate (its body still turning)?
2. Does the same hold for a seized vehicle that was driving temporarily (S-0417 claim 5) before the seize?

## Current workaround
Yes to both. A seized vehicle is not "a vehicle with the driving locomotor" for S-0215 claim 11; it is moving while its jumpjet locomotor moves it (`process::driving` returns false for a seized unit). Test `claim_5_seized_turning_vehicle_is_ready`.
