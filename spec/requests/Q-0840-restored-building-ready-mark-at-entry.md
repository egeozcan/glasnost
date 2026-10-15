---
id: Q-0840
title: Is a building made by the restore-starting-buildings action a placed or a deployed entry for its build-up
status: open
related: [S-0524, S-0406, S-0215]
answered-by: []
---

## Need

S-0524 claim 13 says a restored building uses ordinary during-play placement, with its
construction queued and its build-up following S-0215 claim 1 and S-0406 claims 14, 15 and 17.
S-0406 claim 17 gives two different timings: a placed building (a place order) sets the ready
mark when the build-up reaches its last stage, so a build-up of two or more stages runs twice;
a deployed building has the mark set at entry and runs once. A building created by a trigger
action is neither. The engine (`mp_triggers::replace`) must pick one.

## Questions

1. For a building created by code 130, is the ready mark set at entry (as for a deploy) or only
   when the build-up reports its last stage (as for a place order)?
2. Does the answer change for a type with no build-up sprite (zero-delay range)?

## Possible observation

Sell a pre-placed building with a visible multi-stage build-up and restore it; record the
construction stage by tick and the tick it starts guarding, as in S-0406 PL11.

## Current workaround

`mp_triggers::replace` calls `begin_construction(.., deployed = false)`: the placed timing,
where only `quirks.placed_buildup_twice` off sets the mark at entry. Marked Q-0840 in
`docs/development.md`.
