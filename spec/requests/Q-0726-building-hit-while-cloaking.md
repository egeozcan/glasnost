---
id: Q-0726
title: What a cloaking building does when an uncloak cause reaches it
status: open
related: [S-0516]
answered-by: []
---

## Need

S-0516 claim 18 and the building state table say that a building which loses eligibility while
cloaking finishes its fade first. They give no row for an uncloak cause of claim 11, such as a hit
or firing, that reaches a building still cloaking. The code in
`crates/deterministic/sim/src/cloak/machine.rs` (`uncloak`) needs the rule.

## Questions

1. A building is hit at fade level 7 while cloaking. Does it:
   - start uncloaking at once from level 7;
   - jump to 15 and fall from there; or
   - finish the fade first?
2. Does it play the cloak sound?

## Possible observation

A powered building inside its own cloak field is hit by a zero-damage weapon on a chosen tick of its
cloaking fade.

## Current workaround

The building starts uncloaking at once from its current level, with the sound, as a cloaked
building does when it falls from 15. The choice is marked in the code with this request's ID.
