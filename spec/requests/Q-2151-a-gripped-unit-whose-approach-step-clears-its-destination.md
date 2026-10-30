---
id: Q-2151
title: A gripped unit whose approach step against its lifter clears its destination
status: open
related: [S-0215, S-0607]
answered-by: []
---

## Need

S-0215 revision 88 claim 37 part c removed the exception for an actor whose target is the
object that holds it with a magnetic beam. A seized unit whose target is its own lifter now
takes the approach step as for any target, and a destination the step gives it has no effect
(S-0607 claim 9).

Two parts of that step clear the actor's destination rather than give one:

- part d's drop (the keep-or-drop table says drop: the target and the destination are cleared
  at once), and
- part e's clearing (the destination lies farther from the target than the multiplier times
  r′), which is followed by part f's search; that search can also end with no destination.

S-0607 claim 9 says a rule that gives a gripped unit *no* destination is the exception to
"ignores every new destination": it ends the grip without a drop and sets the unit down where it
is. The specs do not say whether the approach step's clearing is such a rule. The engine
(`approach::step` and `lift::steer`) treats it as one, so a gripped unit can now leave its grip
through its own approach step, for example a human-owned unit of a type that cannot approach its
target, hunting its lifter (test
`claims_s0215_r88::a_seized_unit_hunting_its_lifter_drops_it_and_the_grip_ends`).

## Questions

1. A gripped unit (seized, still being pulled to its lifter) has its lifter as its target and
   takes the approach step. When part d drops the target, is the gripped unit set down where it
   is, with the grip ended, or does it keep flying to its lifter?
2. The same when part e clears the destination and part f's search then finds no cell
   (no destination at all).
3. The same when part e clears it and the search gives a nearby cell: is that cell ignored and
   the grip kept (as claim 9's "ignores every new destination" reads), or is the moment of
   clearing already the end of the grip?

## Possible observation

S-0607 scenario L9 style: a lifter grips an enemy vehicle of a human-owned house whose type is
set not to approach its target (override section), with the vehicle given *hunt* and its lifter
as its target (for instance by a trigger after the seize). Watch whether the vehicle keeps
flying to the lifter or is set down where it is. Repeat with a computer-owned vehicle of an
ordinary type (claim 37 part e, then part f).

## Current workaround

As built: any rule that leaves a gripped unit with no destination at the end of its update,
the approach step's clearing included, ends the grip by S-0607 claim 9 (set down where it is;
the lifter's target is not cleared, claim 12). A cell the step gives is ignored (the grip's
destination comes back) and the grip holds. No quirk switch.
