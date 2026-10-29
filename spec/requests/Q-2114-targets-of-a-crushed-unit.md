---
id: Q-2114
title: What happens to the targets of objects whose target is crushed
status: open
related: [S-0210, S-0215, S-0301, S-0306, S-0309, S-0312, S-0501]
answered-by: []
---

## Need
S-0210 claim 4 removes a crushed object on the same tick, with no damage, and credits the kill
to the crusher. Other objects may have it as their target at that moment. For a killed object we
clear those targets at the kill (`combat::death::mark_killed`), and for an aircraft that loses a
removed target S-0501 claim 32 says it loses it at once (`air::targeters_lose`). No approved claim
we found says what happens to a ground unit or building whose target is removed without dying.

In M-0001's TELE against BFRT games (local lane, side A, seed 0), a fortress crushes a Magnetron
at tick 364. Two other fortresses keep that Magnetron as their target for 7 and 14 more ticks,
until a later tick of theirs drops it. One of them fires at it in that time: the shot has no
target object left and detonates at the world origin. The verdict of that pairing does not
depend on this (Q-2113), but every crush in play can do it.

## Questions
1. When an object is crushed (S-0210 claim 4), do the objects that have it as their target lose
   it on that tick, as for a killed object? Is that the same for every kind of holder (infantry,
   vehicle, building), and for a target that an override set (S-0215 claim 10)?
2. If they keep it until a later check: which check drops it (behaviour processing, the firing
   check, S-0309 table C's *exists* row), and what does a shot fired at it in between aim at?
3. Is it the same for the other removals that are not deaths (for example S-0607 claim 18's
   removal of a survivor, or an object that leaves the map)?

## Possible observation
Two stock vehicles of one house attack one enemy infantry unit; a crusher of a third house
crushes it. Record on each tick from the crush the attackers' targets and fire events.

## Current workaround
Unchanged: a crushed object's removal clears no ground holder's target; the holder drops it
on a later tick. Nothing was changed for it.
