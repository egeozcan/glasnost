---
id: Q-2526
title: A warped aircraft that cannot land takes retreat at its wait's expiry and flies off after its hold
status: answered
related: [S-0306, S-0513, S-0207, S-0501, S-0215]
answered-by: [S-0306]
---

## Need

S-0306 r75 claim 17 (*Render layer of a warp member*) says that "an aircraft member then stays at
rest at that height after its hold until an order, its own behaviour or a seize gives it a
destination, as an aircraft that the map places on the ground does". Scenario P46 run 2 expects
*A* to stay at rest at height 0 "until the trigger's move order" (in part c2 until it is seized,
in part d2 until the part ends), and is falsified if *A* "leaves height 0 before the trigger's move
order".

Built from the approved rules as they stand, our engine gives P46 run 2's *A* a destination of its
own before any order:

1. *A*'s type cannot land, so the type test of its first placement marked it visiting (S-0501
   Terms), and the mark is never cleared. The map's placement fall-back came before that test and
   saw no mark, so *A* stood in *guard*.
2. When the warp's 60-tick wait expires (tick *t*_L + 61), S-0513 claim 39 has a member with no
   target try "a target search with the conditional 4-to-8-tick draw of S-0207 claim 20 (f)".
   S-0207 claim 20 (f) adds that "when the attempt sets no target ... the object falls back to its
   idle behaviour". *A* has no target, the search finds none, and it falls back to rest while it
   still stands at its old position, on the ground.
3. That fall-back takes S-0501 table T2's row "on the ground, visiting, no passengers, not in a
   team", so it forgets target and destination and *retreat* becomes current.
4. *A* appears at its landing point at height 0 and stays there through its arrival hold (S-0513
   claims 36, 38 to 40; this part matches Q-2466's answer). The fall-backs at its appearance and at
   the hold-end search leave *retreat* unchanged (T2's second row).
5. In the tick the hold ends, *retreat* gives *A* a destination and it takes off, before any
   trigger's move order.

So claim 17's "until ... its own behaviour ... gives it a destination" is met, but P46 run 2's
Expected and Falsified lines are not.

## Questions

1. When a warp member's 60-tick wait expires, and again when its arrival hold ends, and its target
   search sets no target, does it fall back to rest, as S-0207 claim 20 (f) has an object do at
   the end of phasing in?
2. If it does: P46 run 2's *A* (its type cannot land, so it carries the visiting mark; on the
   ground, no passengers, not in a team, owned by a computer house) then takes *retreat* by S-0501
   table T2 and flies off after its hold. Does the original do this? If so, is P46 run 2's "stays
   at rest there until the trigger's move order" to be changed (for example, by putting *A* in a
   team, or by giving it a type that is selectable and can land)?
3. If it does not: which of the two searches, if either, makes the fall-back?

## Possible observation

P46 run 2 as written, with no trigger: record *A*'s current behaviour, its destination and its
height above its surface in each tick from the launch to some ticks after its arrival hold ends.

## Current workaround

None in code. The engine keeps the cited rules as they stand, so P46 run 2's *A* appears at height
0 and stays there through its hold (Q-2466's answer, tested), then takes off under *retreat* when
the hold ends. P46 run 2's rest after the hold (parts c2 and d2) is left as a seam: their tests run
in run 1 only and cite this request.
