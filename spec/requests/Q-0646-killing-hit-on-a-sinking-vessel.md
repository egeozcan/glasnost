---
id: Q-0646
title: What a further killing hit does to a vessel that is already sinking
status: answered
related: [S-0504, S-0312, S-0302]
answered-by: [S-0504]
---

## Need

S-0504 claim 10 says sinking is no immunity: shots, area damage and radiation still damage a
sinking vessel, and self-healing still raises its health. A sinking vessel that started from a
killing hit keeps 1 health point (S-0312 claim 10), so the next damage delivered to it, or any
damage once healing has run, can be a second killing hit. S-0504's open questions leave the
consequences of that second killing hit to an S-0312 audit. The gap is in
`glasnost_sim::combat::damage` (the killing branch) and `combat::death::kill`.

## Questions

1. When a sinking vessel takes a killing hit, is a destruction credited again (to the attacker's
   house and in the score), and are the destruction raises of S-0312 claim 29 made again?
2. Does it go through any of S-0312's death steps again: debris, the death weapon, passengers,
   crew or survivors, explosion or destruction animations, and their random draws?
3. What health does it have afterwards (0, 1, or the ordinary result), and does it keep sinking
   on its current descent, restart the descent, or leave play at once?
4. Is a sinking vessel at 0 health (if that can happen) still healed by self-healing and the
   owner's periodic unit healing, and still damaged?
5. Is the answer different for a vessel set sinking by a cull, which had no killing hit
   (S-0614 claim 11)?

## Possible observation

S-0504 scenario N10 (a) and (c): kill a heavy surface vessel on water, then let a second firer's
slow shot or an area hit reach it during the descent, with and without a self-healing vessel or a
house healing source. Record health, kill counts, raises, debris and draws, and the tick it leaves
play.

## Current workaround

Damage reaches a sinking vessel by the ordinary rules (S-0504 claim 10). A hit that would kill an
already sinking vessel ends there: no second credit, no second death and no death effects; the
vessel keeps sinking on its descent (marked Q-0646 in `combat::damage` and in docs/development.md,
M6a interim rules).
