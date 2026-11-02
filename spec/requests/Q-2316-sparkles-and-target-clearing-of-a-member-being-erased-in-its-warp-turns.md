---
id: Q-2316
title: Sparkles and target clearing of a warp member being erased, in its warp-only turns
status: open
related: [S-0513, S-0514, S-0203]
answered-by: []
---

## Need

S-0513 r86 claim 39a gives a warp member with a temporal beam attached an erasure update
(S-0514 claim 7) in each of its turns, at every stage of the warp, before the warp step. It
names that update as one of the intrinsic exceptions to claim 51's whole-member-turn contract
(S-0203 claim 4). S-0514 claim 17 also has a target being erased drop its target and
destination in each of its turns, and claim 18 has it make a sparkle in each turn whose global
tick number is divisible by 24. Neither is named as an exception to claim 51. So in the turns
whose warp update is the whole turn (pending, leaving, expiry, departure, the landing retries,
placement and arrival effects), it is not settled whether a member being erased (before its
departure, or after a later lock) also drops its target and destination and makes its sparkles.

The engine's gap is in `tick.rs` (the living turn): `temporal::member_update` makes the erasure
update before `superweapons::warp::turn`, which ends the turn in those stages, so the
`temporal::turn` that sparkles and clears comes only in the hold and after it.

## Questions

1. In a warp member's leaving-stage turns (from its first update after the launch to its
   departure update), while a temporal beam is locked on it, does it make the being-erased
   sparkle of S-0514 claim 18 in the turns whose global tick number is divisible by 24?
2. In the same turns, does it drop a target or destination given to it, as S-0514 claim 17 has
   an object being erased do? (S-0513 claim 48 already has it drop both in its first leaving
   update.)
3. After a lock made while the member is off the map waiting to land, or in its placement and
   arrival-effects updates (S-0513 claim 39a, *A lock after the departure*), do the same two
   questions have the same answers?

## Possible observation

SW12 part l's set-up, with the lock in the leaving stage and a tick divisible by 24 among the
leaving turns: count the sparkle animations at the member's position between the lock and the
departure. For question 3, part l5's later lock made while the member is off the map.

## Current workaround

No sparkle and no claim 17 clearing in the turns whose warp update is the whole turn: only the
erasure update is made there, the one exception claim 39a names. From the hold on, the member's
turns sparkle and clear as S-0514 claims 17 and 18 and S-0513 claim 39a give.
