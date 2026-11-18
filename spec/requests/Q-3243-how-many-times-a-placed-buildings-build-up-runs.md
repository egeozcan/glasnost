---
id: Q-3243
title: How many times a placed building's build-up runs before handover
status: open
related: [S-0406, S-0108, S-0415, S-0524]
answered-by: [M-0027]
---

## Need

S-0406 claims 14, 15 and 17, F6 and F7 have a placed building with at least 2 build-up stages
and a positive stage delay run its build-up twice: once from the place order tick *T* to
*R* = *T* + (*n* − 1) × *d_b*, then again from stage 0 at the first construction processing
*C* = *R* + 1, handing over at *H* = *T* + 2(*n* − 1) × *d_b* + 1
(`quirks.placed_buildup_twice`, default on). The claims are `measurable` from scenario PL11
(parts a to d and f); no measurement record backs them yet.

The project lead watched the running game and reports that a placed building's build-up plays
**once**, not twice in a row (issue #440, an informal observation, not a measurement record).
If that holds, the default of `quirks.placed_buildup_twice` is wrong, and the handover tick
moves. That tick decides when the opening effects happen (S-0108 claim 11, the free unit of
S-0402 claim 18), when *guard* becomes current, and when an armed building may first attack
(S-0215 claim 32). It is a state-hash change.

The engine implements both positions of the switch already (`crates/deterministic/sim/src/econ/placement.rs`,
the ready mark at entry; the drawing's `Live::buildup_of` in `crates/app/src/mapview/live.rs`
follows the simulation's entry and *C* ticks). S-0406's Edge cases give the off position:
the place order sets the ready mark at entry, *C* = *T* + 2, the build-up is seen once and
ends at *T* + 2 + (*n* − 1) × *d_b*, handover by F6.

## Questions

1. After a place order, does a building with a build-up sprite of at least 2 stages and a
   positive stage delay show its build-up once or twice before its finished body? Is the answer
   the same for every such type (an ore refinery, a power plant, a defence structure, a gate
   with its gate range)?
2. If once: on which tick, counted from the place order tick *T*, does it hand over (finished
   body shown, opening effects and any free unit done, *guard* current)? Is it the Edge-case
   value *T* + 2 + (*n* − 1) × *d_b*, or another (for example *R* or *R* + 1)?
3. If once: is stage 0 shown again at any point after the build-up starts (the restart of
   claim 14), or does the stage advance from the first shown stage to the last without a
   return?
4. Does the answer change the deploy case (S-0415 claim 6, which S-0406 claim 17 already
   gives one run) or a restored replacement (S-0524 claim 15, which shares the switch)?
5. Does the answer for a filling building of a place order (S-0406 claim 11) follow the
   building the order names?

## Possible observation

S-0406 scenario PL11 parts a and f, which are built so that the two runs are easy to tell
apart: record the shown stage and the handover tick from the place order tick for a type with
several stages and a known stage delay, and the same for a deployed building.

## Current workaround

None: the engine keeps S-0406 claims 14, 15 and 17 as written, with
`quirks.placed_buildup_twice` on, until the spec is revised. A revision that changes the
default needs no new code path, only the default and the claim tests that cite PL11.
