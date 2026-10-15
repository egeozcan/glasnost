---
id: Q-0792
title: A cloak generator's own centre during growth, and which neighbours count as idle
status: answered
related: [S-0516]
answered-by: [S-0516]
---

## Need

S-0516 claim 20 (revision 6) leaves two points open. The code in
`crates/deterministic/sim/src/cloak/field.rs` (`turn`, `restart_neighbours`) needs both.

- **Own centre during growth.** With `quirks.cloak_field_building_interrupt` on, the claim
  says the first newly covered centre cell of an own building ends a growth update. The
  generator is an own building, and its centre is in the radius-1 circle. Read literally,
  the radius-1 update would then stop at the generator's centre. Scenario CS18 part D
  expects "the full radius-1 circle" instead. For shrinking, the claim says outright that
  the generator's own centre can stop the update.
- **Idle neighbours.** The neighbour restart applies to "every other in-play, powered,
  idle generator". The claim then says that a neighbour below its full radius drops by
  one, so a growing neighbour seems to count. It does not say whether a shrinking
  neighbour (powered off) or one already restarting counts as idle.

## Questions

1. Does the generator's own centre end a growth update, the radius-1 one included, so
   that the first circle is incomplete? Or is it exempt during growth only?
2. Which generators count as idle for the restart? Growing ones? Ones at full radius only?
   A generator that is powered and in operation but has its field at radius 0?

## Possible observation

- One generator on open ground. Record the covered cells right after its first growth
  update.
- Three powered generators near a fourth that collapses: one at full radius, one growing,
  one just restarted at radius 0. Record each one's radius before and after the collapse.

## Current workaround

- The generator's own centre does not end a growth update; shrinking stops at it as the
  claim says.
- "Idle" means in operation, whatever the current radius.

These choices are marked in the code with this request's ID.
