---
id: Q-3061
title: What does a revert give back when the put-aside target left the game while it was put aside?
status: answered
related: [S-0309, S-0215, S-0301, S-0307]
answered-by: [S-0309]
---

## Need

S-0215 claim 10: an override puts the holder's target (and destination) aside, and the end of the
override reverts to them. S-0309 claim 27 drops a held target when the object is destroyed, goes
off the map or is removed, and covers the case where the drop itself ends an override: "the revert
can make the same object the holder's target again ... The removal is expected to drop it again
after the revert (medium), so that no holder holds a removed object." Its DECIDED rule (Q-2006
question 3) handles a target that a revert gave back and that is off the map, or destroyed but
still in play: no firing check, and a drop with no draw and no revert at the start of the holder's
next turn. Its consequences add: "No holder asks the firing check about a removed object."

None of this covers a put-aside target that left the game **earlier**, while it was put aside.
Claim 27's drop at the removal applies to holders that hold the object as their target; a holder
that only has it put aside does not hold it then. When the override ends later (for example its
own target is destroyed), the revert gives back an object that is no longer in the game.

GitHub issue #163 reproduces this on our engine with an obstacle override (the holder's step is
refused by a non-allied object, and it attacks that object meanwhile). The holder then fires one
shot at the world origin and drives away from the fight, because nothing handles a held target
that is not in the game at all.

## Questions

1. When an object is removed from the game (or destroyed, or goes off the map) while a holder has
   it put aside under an override, is the put-aside target cleared at that moment, so that the
   later revert restores only the put-aside destination (or nothing)?
2. If it is not cleared then: at the revert, does the holder get the object back as its target?
   If so, what does it do with it: does it drop it (when: at the revert, or at the start of its
   next turn as claim 27's DECIDED rule does), with or without claim 27's timer draw, and is the
   put-aside destination kept?
3. Is the answer the same for a put-aside object that went off the map (entered a transport or a
   building) and later came back onto the map before the revert?
4. Does any of this differ between the obstacle override (a refused step, S-0215) and a retaliation
   override?

## Possible observation

A tank ordered to attack enemy X has its path blocked by an enemy unit Y, so that it overrides to
Y. X is destroyed by another unit while the tank is fighting Y. Then Y is destroyed. The recorder
logs, per tick from X's destruction on, the tank's target, destination, behaviour, shots and the
scenario random state. Repeat with X entering a transport instead of being destroyed, and with X
leaving the transport again before Y is destroyed.

## Current workaround

None in code: the engine is unchanged for #163 until this is answered. Its present result is the
dangling reference described in #163. Two candidate fixes are recorded there (count a missing object as
left under claim 27's DECIDED rule, or clear a put-aside target at the object's removal); they
differ in the destination kept and in whether the drop waits for the holder's next turn, so the
choice is left to the answer.
