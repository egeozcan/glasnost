---
id: Q-0983
title: When the own-cell shot of a radiation-immune deployed infantry unit is made
status: open
related: [S-0215, S-0525, S-0107]
answered-by: []
---

## Need

S-0215 claim 31 (third and fourth rows of its table) says that a deployed infantry unit of a type
that deploys to fire and is immune to radiation, when no field centred on its cell has a level of at
least *R*, and the firing rules let it fire at its own cell, "fires at its own cell, drops the target,
starts its deployed-fire sequence", and that the delay is the sequence's frame count. The spec does
not say on which tick the shot itself is made: at that processing, or at the fire frame of the
sequence like an ordinary infantry shot (S-0107 claim 11). The tick matters to S-0525: it makes the
field (claim 1) and so decides when the field's level suppresses the next shot.

## Questions

1. Is the shot made in the processing that decides to fire (so the field exists from that tick), or
   on the tick the deployed-fire sequence reaches its fire frame?
2. If it is made at the fire frame, which fire frame applies (the one of the deployed-fire kind, or
   the standing or prone frame of the secondary weapon)?
3. Does the unit keep its own cell as its target until the shot, or is the target dropped at the
   processing as the table says?

## Possible observation

A radiation-immune deployed infantry unit (S-0215 scenario B19 part c) on a clean cell with a
fire frame later than the first frame; record the tick the field appears, and the tick of the
processing that started the sequence.

## Current workaround

`process::immune_deployed` makes the shot in the processing itself (`combat::fire::shot` at the unit's
own cell), drops the target and starts the deployed-fire sequence for its presentation; the delay is
the sequence's frame count.
