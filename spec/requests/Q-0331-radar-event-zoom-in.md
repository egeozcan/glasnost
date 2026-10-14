---
id: Q-0331
title: How long a radar event's zoom-in phase lasts for the nearby-event suppression of S-0701 F7
status: answered
related: [S-0701, S-0410]
answered-by: [S-0410]
---

## Need

S-0701 F7 suppresses an announcement while a radar event of the same kind exists near it. An
event lasts its zoom-in phase and then *t* ticks. Three things are open (S-0701 Open questions,
scenario G3 d):

- how long the zoom-in phase lasts;
- whether it advances in ticks or in drawn frames;
- whether it advances while the local player has no radar.

Our engine has no radar events yet, so the sound side keeps the events of F7 itself.

## Questions

1. How long is the zoom-in phase, and is it counted in ticks or in drawn frames?
2. Does it advance while the local player has no radar?

## Current workaround

The zoom-in phase lasts 0 ticks, so an event lasts *t* ticks from its creation. `glasnost_audio`
keeps the events for announcements only; they are not shown on the radar. Documented in
docs/development.md, "Audio".
