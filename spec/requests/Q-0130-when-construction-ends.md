---
id: Q-0130
title: When a placed or deployed building's construction ends, and what it does next
status: open
related: [S-0108, S-0406, S-0408, S-0415, S-0215]
answered-by: []
---

## Need

S-0406 claim 1 and S-0415 claim 6 say that a placed or deployed building's behaviour becomes
*construction* and its construction state begins (S-0108). S-0108 claim 5 gives the tick on which
the build-up is reported finished ((*n* − 1) × *d_b* ticks after the state began) and says that the
switch to the body sprite "then follows the construction behaviour". S-0215 claim 34 names the
structures topic as the owner of the *construction* behaviour's processing but gives no delay.
No spec says how often *construction* is processed, on which tick the building counts as
constructed (the end of S-0408 claim 3 (a)'s "construction has completed"), or which behaviour
the building has afterwards. The same gap applies to the end of a sale's build-down (S-0408
claim 6: "the first processing after build-down has finished").

The engine needs this in `econ::deploy::process_construction` and `econ::sell::process_selling`.

## Questions

1. How many ticks apart are the processings of a building's *construction* behaviour, and does
   their timing take a draw from the scenario random stream?
2. On which tick does a building placed at tick *T* stop being under construction: the tick
   *T* + (*n* − 1) × *d_b*, the first processing on or after it, or another?
3. Which behaviour does the building have once its construction has ended (for example *guard*),
   and is a target it had kept?
4. For a sale, on which tick after the start of the build-down is the building removed and the
   refund paid, measured in ticks from the build-down start?
5. For a gate (S-0108 claim 5: *n* = *G* + 1) and a type with no build-up sprite, are the answers
   the same?

## Possible observation

Place a test building with a known build-up frame count and build-up time; record per tick its
behaviour, its sprite (build-up or body) and whether sell mode offers *sell* on it. Sell it and
record the tick it disappears and the tick the credits change.

## Current workaround

`econ::deploy::process_construction` runs every tick (delay 1, no draw). At the first processing
with tick − start ≥ (*n* − 1) × *d_b* it clears the construction state and forces *guard* (as a
pre-placed building's, S-0009 claim 40). The build-down of a sale ends the same way: the first
processing (every tick, S-0408 claim 16) at which (*n* − 1) × *d_b* ticks have passed since the
build-down began. Gates and theater-flag buildings use the plain ⌊*N_b* / 2⌋ rule. Listed in
`docs/development.md`'s provisional table.
