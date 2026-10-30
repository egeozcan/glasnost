---
id: Q-2140
title: Type-select drag candidates, the select-target announcement, and the attack rule for a falling unit
status: open
related: [S-0214, S-0512, S-0522, S-0301]
answered-by: [S-0214]
---

## Need
Three small gaps met while implementing S-0214 r62 claims 32 and 33,
S-0512 claim 25 and S-0522 claim 17. Code:
`crates/app/src/game/session.rs` (`pointer_up`'s type-select drag),
`crates/app/src/game/play.rs` (superweapon arming) and
`crates/deterministic/sim/src/targeting.rs` (claim 6 (e)).

## Questions
1. A drag with the type-select role held (S-0214 claim 32: held clicks and
   drags read the reach and do not change it): which objects in the box
   count, the ordinary band box's candidates of claim 19, or every own
   object in the box that can be selected (as for the role's held clicks,
   which ask only "can be selected")? Do buildings that power a unit (the
   tap's extra candidate) count in a drag?
2. S-0512 claim 25: arming a superweapon from its icon announces the
   select-target prompt. Which string-table entry and which sound is it?
   Import mapping names none.
3. S-0214 claim 6 (e) asks that the firing rules not rule the target
   illegal; S-0301 claim 9 rank 4 rules every target illegal for a falling
   firer. S-0214's Terms and S-0522 claim 17 say an attack click during a
   fall gives an attack order at once. Is the falling refusal one of the
   "other reasons for not firing at this moment" that claim 6 (e) leaves
   out (so the attack cursor and order are as for a landed unit), or does
   the attack click during a fall give some other action?

## Possible observation
Question 1: hold the role and drag over a mixed group including a
deactivated vehicle and a unit-powering building. Question 3: scenario P10
(d) of S-0522, noting the cursor shown over the enemy during the fall.

## Current workaround
Question 1: the drag's seeds are the ordinary band box's candidates in the
box (claim 19); the expansion then adds, from the reach's pool (the screen
candidates, or every object in creation order), each own object of those
types that can be selected. Question 2: no announcement is made (seam).
Question 3: the click's claim 6 (e) asks the firing check as for a landed
firer (`check_ignoring_fall`), so the attack click during a fall gives an
attack order, as S-0522 claim 17 and its scenario P10 (d) expect.
