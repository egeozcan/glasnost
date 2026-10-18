---
id: Q-1421
title: Order of the extra virus effect and the explosion death of a parachutist
status: answered
related: [S-0615, S-0613, S-0107]
answered-by: [S-0615]
---

## Need
A parachuting infantry unit killed by a kind-8 (virus) warhead gets S-0613 claim 3's extra
virus-death animation and one cloud particle, and S-0107 claim 17 sends the same death to kind 3,
so S-0615 claim 1 also creates the explosion death animation. No spec gives the order of these two
creations. Both can take draws (the explosion type's creation draws, S-0202 claim 10, and the
virus animation's middle event when its μ is 0), and both push presentation events. Code:
`crates/deterministic/sim/src/combat/death.rs`, row 8 of the human death cases.

## Questions
1. In one such death, which is created first: the extra virus-death animation with its one
   particle, or the explosion death animation?
2. Is the order the same for a not-human infantry type killed by a kind-8 warhead whose own death
   kind dispatch also creates an animation?

## Possible observation
A parachutist killed by a virus warhead where both the virus-death animation (μ = 0, with a
particle) and the explosion death animation (a bouncing type) take draws; the stream recorder
shows which draws come first.

## Current workaround
The extra virus effect first, then the explosion death animation (the order in which S-0107
claim 17 lists them).
