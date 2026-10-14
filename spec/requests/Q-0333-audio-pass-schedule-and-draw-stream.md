---
id: Q-0333
title: Our engine's audio pass schedule and the stream of per-instance sound draws
status: open
related: [S-0701, S-0202]
answered-by: []
---

## Need

S-0701 asks the project lead which stream our engine uses for the per-instance draws of claims 13
and 14 (Questions for the project lead). It leaves our pass schedule to that decision (Edge cases,
"The original's audio passes follow real time"). `glasnost_audio` needs an answer to both now.

## Questions

1. Do per-instance draws use the unsynchronised stream, as in the original, or a separate
   presentation generator, as the spec proposes?
2. Is our audio pass schedule the original's, or something else? The original runs a pass at the
   next service once more than 33 ms of real time have passed since the last one.

## Current workaround

- Per-instance draws use a separate presentation generator, as the spec proposes. It is the
  S-0202 generator, seeded with the game's seed.
- Game-side choices (claims 17, 18, 20 and 23) use the unsynchronised stream.
- A pass runs at the first drawn frame that comes more than 33 ms of host time after the previous
  pass.
- The crowded-sound master level of F3 reaches its target at once at each pass, because its ramp
  is unmeasured.

Documented in docs/development.md, "Audio".
