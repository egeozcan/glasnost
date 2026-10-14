---
id: Q-0266
title: Which sounds play for the sidebar clicks and refusals of S-0411 claim 12?
status: open
related: [S-0411, S-0406, S-0404]
answered-by: []
---

## Need

S-0411 claim 12 lists the clicks that play an interface sound (start or queue, resume, hold,
cancel, the busy-line refusal, a refused placement) and leaves to the sound spec which sound each
is and whether the two refusals are one sound or two. S-0404 claim 3's refusal for a full queue
has a refusal sound too, and S-0213 claim 13's refused route-mode click has "a message and a
sound". `glasnost_audio::UiCue` needs a sound name for each.

## Questions

1. For each cue of S-0411 claim 12, S-0404 claim 3's refusal and S-0213's refused route-mode
   click, which game-wide key (or fixed entry) gives the sound?
2. Are the refusals of S-0411 claims 4 and 10 (and S-0404 claim 3) one sound or two?
3. Does any cue depend on the player's side?

## Possible observation

S-0411 scenarios S3, S4 and S8 with a loose sound database copy giving each candidate entry a
distinct test sound.

## Current workaround

Every sidebar `UiCue` resolves to no sound (silent) until this is answered. Documented in
docs/development.md, "Audio (M5b)".
