---
id: Q-0622
title: Where within its host's turn does an attached parasite act?
status: answered
related: [S-0614, S-0203, S-0314, S-0215]
answered-by: [S-0614]
---

## Need

S-0614 claim 18 says an attached parasite acts during its host's turn, but "its exact ordering
relative to the host's movement is not established". A host's turn in our engine runs
self-healing (S-0314 claims 9 and 10), behaviour processing (S-0215), the movement update and
the firing step, in that order. The delivery's hit, hold renewal and draws can change what the
later parts of the same turn see: for example, a delivery that sets the hostile-hit mark or
makes the host retaliate before its behaviour processing, or a hold renewed before or after the
movement update of that tick. Our engine needs one place in `glasnost_sim::tick::object_turn`.

## Questions

1. In the host's turn, does the parasite's delivery (or grappling event) come before or after
   the host's self-healing step?
2. Before or after the host's behaviour processing of that turn?
3. Before or after the host's movement update of that turn? In particular, in a delivery tick
   whose hold renewal starts a hold, does the host still move in that tick?
4. Before or after the host's own firing step?

## Possible observation

A self-healing host carrying a drone-type parasite with a 1-tick rate of fire, recorded every
tick: health after healing and delivery, the tick of its retaliation against another attacker,
and its position on the tick a hold starts after an interruption (paralysis length shorter than
the rate of fire).

## Current workaround

An interim rule, noted in docs/development.md: the parasite acts after the host's self-healing
and before its behaviour processing and movement (`glasnost_sim::parasite::host_turn`, called
from `object_turn`). A host killed by the delivery does nothing more in that turn.
