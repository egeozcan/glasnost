---
id: Q-1770
title: Does a building's pending delayed shot outrank the water and land rows of the firing check?
status: answered
related: [S-0306, S-0301, S-0215]
answered-by: [S-0301]
---

## Need

S-0306 claim 23 (revision 33) puts the water and land rows of the firing check at rank 4 of
S-0301 claim 9 and lists the rank-4 answers that come before them. It says that where the
*rearming* of a building's pending delayed shot (S-0301 claim 14) falls against the two rows is
not stated, as S-0301 claim 9 leaves that answer's rank unstated.

Our engine asks the delayed-shot test after the rearm countdown (rank 5), so a building whose
delayed shot is pending answers *illegal* against a target that one of the two rows refuses. Its
*attack* then drops the target (S-0215 claim 32) rather than waiting for the delayed shot. The
choice matters only when a building's held target becomes one the rows refuse while a delayed
shot is pending: a target that drives onto or off a bridge, or onto or off a water or beach cell,
for a building type with land-targeting mode 1 or a naval-targeting mode that refuses it.

## Questions

1. A building type with animation-delayed shots (S-0301 claim 14) and land-targeting mode 1 holds
   an enemy vehicle on a water cell as its target and has started a delayed shot. The vehicle
   drives onto a clear land cell before the shot leaves. Is the delayed shot fired, and does the
   building keep or drop the target in the tick the vehicle's cell changes?
2. Does the answer change with naval-targeting mode 6 and a vehicle that drives from a clear land
   cell onto a water cell?
3. When the delayed shot is fired, does it fly at the vehicle as any other shot (S-0306)?

## Possible observation

A test map with one such building type (owned by a human-controlled house) beside a shore, an
enemy vehicle ordered across the shore line in its range, 10 runs per case. Record per tick the
building's target, its behaviour, its delayed-shot state and each projectile it creates.

## Current workaround

The rows win: the delayed-shot *rearming* is asked after them, at its place after the rearm
countdown (`crates/deterministic/sim/src/combat/check.rs`, cited as interim Q-1770).
