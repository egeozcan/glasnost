---
id: Q-1716
title: Which comes first in one advance, an animation's damage or its middle event
status: answered
related: [S-0109, S-0202, S-0203]
answered-by: [S-0109]
---

## Need
M8 step A5 plays animations in the simulation's turn pass (`crates/deterministic/sim/src/anims.rs`,
`AnimEnv::advanced`). S-0109's *State and ordering* table says that on an advance "damage due on
this advance is delivered, and the middle event occurs when the resulting stage satisfies
*a* + *k* = μ", and *Observable order within one update* says both follow the advance. It does not
say which of the two comes first when one advance has both: a type with a damage value δ > 0, μ ≠ 0
and both the scorch and crater flags (or a particle type), on the advance that reaches μ.

The order is observable. The damage is an area detonation (S-0109 claim 17) that can take draws
(S-0302's damage rules, death draws of objects it kills, S-0303 claim 17's resource reduction), and
the middle event takes the particle creation draws and the scorch-or-crater draw (claim 14,
S-0202 claim 15), so the scenario stream sees the two groups in one order or the other. A crater's
removal of 6 bales (S-0401 claim 3) and the detonation's resource reduction on the same cell also
change what the other sees.

## Questions
1. On one advance that both delivers animation damage (claim 17, F5) and produces the middle event
   (claim 14), which comes first: the damage detonation, or the middle event's particle requests
   and scorch-or-crater choice?
2. Does the same order hold on an ending advance (the advance that ends the animation, where both
   still happen) and on the advance just before a follow-up switch?

## Possible observation
An animation type with damage 1.0, a middle frame μ = 2, both the scorch and crater flags and no
particles, placed on a resource cell with the chain-reaction flag under a game-wide warhead with
the resource flag. Record the scenario stream position and the cell's bales before and after the
tick of the advance that reaches μ, with an object in the damage area whose death takes draws.
Under damage first the detonation's draws come before the ranged draw over 0 … 2147483646; under
middle event first the order is reversed, and the crater's 6 bales are gone before the detonation
reads the cell.

## Current workaround
The damage of an advance is delivered first, then the middle event, the order in which the
table's cell lists them (`AnimEnv::advanced`, interim rule of Q-1716).
