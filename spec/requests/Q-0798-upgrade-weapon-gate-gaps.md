---
id: Q-0798
title: The upgrade weapon gate's missing-secondary attempt, delayed-shot buildings and the first installed upgrade
status: answered
related: [S-0215, S-0308, S-0301, S-0406, S-0408]
---

## Need

S-0215 revision 43 claim 32 adds the upgrade weapon gate for a non-SAM building with an
installed upgrade. S-0308 revision 11 claim 12 resolves each weapon slot from the first
installed upgrade, in installation order, that has a weapon there. S-0301 revision 15 claim 9
exempts the gate's secondary attempt from the firing check. Three points are open for our
engine.

1. A passing gate makes a primary attempt and then a secondary attempt. The gate can pass
   on a primary whose burst count is above 1 with no secondary anywhere. S-0301 claim 9 and
   S-0215 B36 say the result of that missing-secondary attempt is unspecified.
2. S-0301 claim 14 says an animation-delayed building starts its delayed shot instead of
   firing on *can fire*. Claim 32's gate does not say how it combines with that rule. Does a
   failed gate also stop the delayed shot from starting? With a passing gate, is the pair of
   attempts made at once, or delayed? Or does the delayed single shot replace them?
3. Our engine records installed upgrades in the building's three upgrade positions (S-0406
   F5, position *c*′). A sale empties a position and lowers the count (S-0408 claim 5). A
   level field above 1 skips positions. The wall tower's count is uncapped. After any of
   these, position order and installation order can differ. Which upgrade is "first
   installed" then, for claim 12 and for the gate?

## Questions

1. With the gate passed and no weapon in the secondary slot, does the secondary attempt do
   anything (a shot, a burst step, a rearm, an answer), or nothing?
2. For an animation-delayed building with an installed upgrade:
   - Does a failed gate suppress the delayed shot's start?
   - Does a passing gate start the delayed shot, make the two attempts, or both?
3. After an upgrade sale, a level skip or a tower count above 3, which installed upgrade is
   the first, and in what order are the rest taken for claim 12?

## Possible observation

Use S-0215 B36's upgrade fixture, extended as follows. Record attempts, burst counter, rearm
state and shots with the recorder.

- Question 1: give the first upgrade a primary with burst count 2 and no secondary.
- Question 2: give the host the animation-delayed firing flag.
- Question 3: install three upgrades with distinct single-slot weapons, sell the first, then
  install a fourth. Record which weapon each slot resolves to.

## Current workaround

Interim rules:

- The missing-secondary attempt does nothing: no shot, no burst step and no rearm.
- An animation-delayed building with a failed gate (quirk on) starts no delayed shot. With a
  passing gate it starts its delayed single shot as before, and makes no paired attempts.
- The installed upgrades are taken in position order, 1 to 3. The first nonempty position is
  the first installed upgrade, for the gate and for claim 12.

These choices are marked in the code with this request's ID.
