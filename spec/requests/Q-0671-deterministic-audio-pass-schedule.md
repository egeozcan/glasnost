---
id: Q-0671
title: The deterministic schedule that replaces the audio service clock (S-0701 claim 30)
status: open
related: [S-0701, S-0202]
answered-by: []
---

## Need
S-0701 revision 13, claim 30, requires our engine's presentation scheduling to be deterministic:
with identical initial unsynchronised-stream state, data, game-speed changes, presentation inputs
and cycle-indexed game events, host load and audio-device callback timing must not change the
random choices assigned or their order. It also says the exact deterministic replacement for the
original's service clock, waits and sample-completion opportunities is deferred, and adopts no
tick conversion or zero-wait substitute.

Our audio (crate `glasnost-audio`) runs its passes on host time: the client calls
`Audio::frame(now_ms, unsync)` every frame with wall-clock milliseconds, and `Mixer::service`
runs a pass when more than 33 ms of that time have passed since the last (S-0701 claim 12 and
Constants). The per-instance draws of claims 13 and 14 happen in those passes and take raw values
from the shared unsynchronised stream, so their interleaving with game-side choices depends on
host timing. This does not meet claim 30, and the replacement schedule is not specified.

## Questions
1. On what deterministic clock do our engine's audio passes run (for example a fixed number of
   passes per simulation tick at each game speed, or passes indexed by presentation frame), and
   how does it map the more-than-33-ms pass interval of claim 12?
2. What replaces the original's sample-completion opportunities (when a playing instance's next
   sample is chosen, claim 14) and delayed-mode waits?
3. At what point in that schedule are the game-side choices of a tick (claims 17 to 23) taken
   relative to the per-instance choices of the passes that fall in the same tick?

## Possible observation
None: claim 30 makes this an engine design requirement (conformance scenario H2), not an oracle
measurement.

## Current workaround
Unchanged: passes run on host time as described above. This is recorded as an interim rule in
docs/development.md (Audio). Scenario-stream state and game rules are unaffected (claim 29), so
the simulation stays deterministic; only the order of presentation draws can vary between hosts.
