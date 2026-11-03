---
id: Q-2377
title: The sparkle of a turn whose erasure update removes the object being erased
status: answered
related: [S-0514, S-0513, S-0109]
answered-by: [S-0514]
---

## Need

S-0514 claim 18 gives a sparkle in each turn of an object being erased whose global tick number
is divisible by 24, and claims 7, 8 and 11 give the erasure update of that same turn, which may
complete the erasure and remove the object (claims 12 and 13). S-0513 r89 claim 39a (*Sparkles
and drops in the warp's turns*) leaves out a warp member's turn whose erasure update removes it,
and its Open questions put the order of the sparkle against the erasure update within one turn
to S-0514, for every object being erased. S-0514 r14 has no answer yet. Since S-0514 r14 claim 18
makes the sparkle an ordinary animation with S-0109's effects (creation draws, damage, start and
middle effects), the order decides both whether the removing turn has a sparkle and whose draws
come first.

The engine's places are `temporal::turn_sparkle` and `temporal::erasure_update`: for a warp
member in `tick.rs` before `temporal::member_update`, for any other object in the ordinary turn
before `temporal::turn`.

## Questions

1. In a turn of an object being erased whose tick number is divisible by 24 and whose erasure
   update completes the erasure, is a sparkle created at the object?
2. Within one such turn of an object being erased, which comes first: the sparkle's creation
   (with any creation draws its animation type takes) or the erasure update?

## Possible observation

S-0514 scenario E11 part b's sparkle record, with a single attacker whose raw damage makes the
erasure complete in a tick divisible by 24: record whether a sparkle is created at the target in
that tick. For question 2, give the sparkle animation a ranged rate pair, so that its creation
takes a draw, and record the scenario random stream's draws in that tick against a completion
that takes one.

## Current workaround

The sparkle comes first, the order an ordinary turn of an object being erased has had in our
engine since S-0514 was built: a turn whose erasure update removes the object still makes its
sparkle, and the sparkle's creation draws come before the update. For a warp member, the same
(S-0513 claim 39a puts the sparkle before the warp step, and our erasure update is also before
it).
