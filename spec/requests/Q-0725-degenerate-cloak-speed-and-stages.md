---
id: Q-0725
title: Cloaking with a stage count of 1 or less, or a negative speed
status: answered
related: [S-0516]
answered-by: [S-0516]
---

## Need

S-0516 Open questions note that the duration formulas do not cover degenerate stage counts
(N ≤ 1) or negative cloaking speeds. The code in
`crates/deterministic/sim/src/cloak/machine.rs` (`step`) needs the rule.

## Questions

1. With a cloaking speed below 0, do cloaking and uncloaking advance one stage per tick, as with a
   speed of 0 and the quirk off? Or do they stall, or do something else?
2. With a stage count of 1 or 0:
   - How long does cloaking take?
   - How long does uncloaking take?
   - Which stage does an uncloak start from?

## Possible observation

Use test files that give a cloakable vehicle a cloaking speed of −1, and the game a stage count of
1, then 0. Time the cloak and the uncloak.

## Current workaround

- A speed below 0 is treated as a speed of 0. With `quirks.cloak_zero_speed_stall`, it therefore
  stalls uncloaking.
- For N ≤ 1 the stage arithmetic is used unchanged: c = ⌈N/2⌉, and the uncloak starts from N − 1,
  floored at 0.

The choice is marked in the code with this request's ID.
