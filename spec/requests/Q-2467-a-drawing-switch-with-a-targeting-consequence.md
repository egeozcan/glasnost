---
id: Q-2467
title: Who owns quirks.layer_change_on_events_only now that its off position changes a target search
status: open
related: [S-0102, S-0306, S-0309]
answered-by: []
---

## Need

`quirks.layer_change_on_events_only` is S-0102 claim 4's switch. Until now every position of it
changed only which render list an object is in, and so only draw order. Our engine keeps it in
the client's drawing settings, outside the simulation, where each player's client may set it on
its own without any effect on the game's state.

S-0306 r72 claim 17 (*An aircraft that cannot land, in flight*) now gives its off position a
consequence in the simulation: "With S-0102's `quirks.layer_change_on_events_only` off, that
switch's sweep moves the aircraft out of the ground list in the first tick its state and height
give another layer, so S-0309 claim 7's ground-layer pass no longer considers it." A target
search is lockstep state. If the off position is to reach it, the switch must be one game-wide
setting that every player's simulation reads alike, not a per-client drawing choice.

## Questions

1. Is the switch's off position meant to reach S-0309 claim 7's ground-layer pass in our engine
   (so the switch becomes a game-wide simulation setting, with the drawing following it), or is
   that sentence of claim 17 only a description of what the drawing sweep would do, with the
   simulation's ground-layer pass always reading the on position?
2. If the switch becomes a game-wide setting, may a client still draw with its own position, or
   must the drawing follow the game's?

## Possible observation

None in the original, which has one behaviour (the on position). This is an engine-design
question.

## Current workaround

The switch stays a client drawing setting. The simulation's ground-layer pass always reads the
on position (the spec's default): an aircraft that cannot land stays in the ground list in
flight until a crash. The drawing's sweep with the switch off moves it as before. The registry
note in `crates/deterministic/rules/src/quirks.rs` records the seam.
