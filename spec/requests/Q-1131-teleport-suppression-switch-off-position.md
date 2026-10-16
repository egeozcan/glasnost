---
id: Q-1131
title: The proposed switch for a teleporting vehicle's suppressed assignment, and what its off position does with the suppression
status: open
related: [S-0207, S-0215, S-0514]
answered-by: []
---

## Need

S-0207 revision 36, Edge cases, names `quirks.teleport_suppression_keeps_destination` (a "proposed name; default: the original behaviour") for claim 27's second Q-1130 row: under pending point suppression a teleporting vehicle keeps the new destination, fixes no landing point and consumes the suppression. The off position is "the suppressed assignment is handled by claim 20 (a) as an ordinary one". The paragraph ends "Whether the switch exists, and its default, are for the project lead (Q1)". No question Q1 is defined in the spec. The workspace test that every `quirks.` name in an approved spec has a switch fails while the switch is missing, so the engine needs to know whether to build it and what its off position does. The code is in `glasnost_sim::teleport::take_destination`.

## Questions

1. Does the switch exist (approved), and is its default the original behaviour (on)?
2. With the switch off, is the pending suppression consumed by that assignment (as an ordinary point application would consume it), or left pending for a later assignment? For a teleporting vehicle every later nonempty assignment reaches the same test, so if it is never consumed it stays pending for good.
3. With the switch off, do S-0215 claim 2 and S-0514 claim 17 (a vehicle under pending suppression keeps its destination while erased or in the leaving stage) still apply, or does the off position remove that exception too?

## Current workaround

Interim (listed in `docs/development.md`, section *Simulation (M6c: destination refresh and the dock fallback)*): the switch is built, default on, as the spec's proposed name and default say, like the other switches a spec proposes pending the lead's sign-off. Off: the assignment consumes the suppression and is then handled by claim 20 (a) as an ordinary one (dropped while phasing in, held or being erased; otherwise a landing point is fixed). The S-0215 and S-0514 exceptions are not switched.
