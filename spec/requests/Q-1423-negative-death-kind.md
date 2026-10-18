---
id: Q-1423
title: What a negative infantry-death kind creates
status: answered
related: [S-0615, S-0107]
answered-by: [S-0615]
---

## Need
S-0615 claim 1's table covers kinds 0 to 10 and "11 and above". A warhead's infantry-death field is
a plain integer in the rules data, so a negative value can be given. Code:
`crates/deterministic/sim/src/combat/death.rs`, `death_kind_effect`.

## Questions
1. When the death kind is negative, is the unit removed at once with no animation (as kind 0), or
   does something else happen (for example a sequence of the unit, or an animation picked by the
   value)?

## Possible observation
A warhead with infantry-death −1 killing a plain infantry unit (S-0615 scenario DK1 with one more
row).

## Current workaround
As kind 0: no animation, the unit removed at once.
