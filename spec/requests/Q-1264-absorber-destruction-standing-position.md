---
id: Q-1264
title: Which standing position a passenger takes when an absorbing building is destroyed
status: answered
related: [S-0605, S-0312, S-0208]
answered-by: [S-0605]
---

## Need
S-0312 claim 24 puts an infantry passenger of a destroyed building "on a free standing position"
of its foundation cell, but does not say which one nor whether a random draw is made. The sale's
release (S-0408 claim 8) names its choice (centre class, one draw). The engine needs the
destruction's choice (`crates/deterministic/sim/src/structures/absorb.rs`, `place_released`).

## Questions
1. Which free standing position does an infantry passenger take in its foundation cell when the
   building is destroyed? Is a random draw made?

## Possible observation
Destroy an absorbing building holding one infantry passenger several times from the same saved
state with different random seeds; record the passenger's position within its cell.

## Current workaround
The first free standing position of S-0208 table P, without a draw (as S-0312 claim 13's crew
placement).
