---
id: Q-1150
title: What our engine does in the default position of quirks.removed_tag_attack_event
status: answered
related: [S-0601, S-0011]
answered-by: [S-0601]
---

## Need

S-0601 revision 9 claim 5 (*Tag removed by the first raise*) and its Edge cases name the switch
`quirks.removed_tag_attack_event`. A takeover hit raises event 6 and then event 44 on its tagged
target. When the event-6 raise takes a one-time or all-carriers tag off the target, the original
still raises event 44 at the target and faults. The switch defaults to that fault, as every quirk
switch defaults to the original. Off: event 44 is raised only if the target still carries a tag,
on that tag.

Our engine cannot fault: it has to go on deterministically in both positions. The spec gives the
off position's behaviour but not what the engine should do when the switch is on and the case
occurs. The spec's own question for the project lead asks whether the fault stays a quirk switch
at all. The code is `glasnost_sim::mind::detonation`; the switch is in
`crates/deterministic/rules/src/quirks.rs`.

## Questions

1. With the switch on and the event-6 raise having taken the tag off the target, what should our
   engine do in place of the fault: skip the event-44 raise (the off position's result), raise it
   on the tag that just left the target, stop the game with a stated outcome, or something else?
2. If the answer is that the fault becomes a departure (no switch), should the switch be removed
   from the spec, leaving only the off position's rule?

## Possible observation

None: S-0601 claim 5 says the fault must not be provoked on the oracle. This is a design decision
for our engine.

## Current workaround

Interim rule, noted in `docs/development.md` (Simulation, M7a: Mind control) and cited as
"Q-1150 interim" in the code: in both positions event 44 is raised only if the target still
carries a tag after the event-6 raise, and then on that tag. The switch therefore changes nothing
yet; both positions are tested to give that result.
