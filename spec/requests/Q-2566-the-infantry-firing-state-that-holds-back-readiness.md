---
id: Q-2566
title: The infantry firing state that S-0215 claim 11's "in the middle of firing" reads, where it begins and where it ends
status: answered
related: [S-0215, S-0107, S-0513, S-0301]
answered-by: [S-0215]
---

## Need

S-0215 r104 claim 11 lists among an infantry unit's readiness conditions that "it is in the middle
of firing", and says: "For an infantry unit the 'in the middle of firing' test reads the firing
state that earlier updates left and does not see the ordinary firing check of this update." No
claim says what that firing state is, when an update sets it, or when it ends for a unit that
keeps its target.

S-0513 r115 claim 39a (*The firer*) gives one ending: "A unit with no target is no longer in the
middle of firing from its first update after the order on: the firing state that a shot left ends
at that update, after both of its per-tick starts." SW12 parts l9 and l9b, and S-0215 scenario B45
part (c), judge runs by whether the unit "is in the middle of firing just before its first update
after the deploy order", which our engine cannot tell.

Our engine has two candidates and neither is stated to be the condition:

- S-0107 r31 claim 11's attack state. It begins when the unit has a target, is not in it and its
  weapon is ready, and ends on no target, a target change, a stage with delay 0 and the other rows
  of that claim. It is kept state that earlier updates leave, so it fits claim 11's wording, but no
  spec names it as the readiness condition.
- The rearm countdown (S-0301 r71's firing state table). Clearing the target leaves it running, so
  it cannot be the state that claim 39a says ends at the first update with no target.

So the infantry readiness rule (S-0215 claim 11) has no "in the middle of firing" condition in our
engine, and the SW12 l9b and B45 (c) behaviour (an accepted *move* or *unload* staying queued
while the unit fires) is not built (M8 seam sweep 5b; intake 94's seam).

## Questions

1. Is S-0215 claim 11's "in the middle of firing" for an infantry unit the S-0107 claim 11 attack
   state? If not, what state is it?
2. Where does it begin: at the shot, at the start of the fire-standing (or fire-prone) sequence,
   or when the attack state begins before the fire frame?
3. For a unit that keeps its target, where does it end: at the end of the firing sequence, at the
   fire frame, when the rearm countdown runs out, or only at one of S-0107 claim 11's attack-state
   endings?
4. Claim 39a says the state a shot left ends "at that update, after both of its per-tick starts"
   for a unit with no target. Is that the same as S-0107 claim 11's no-target ending, applied at
   the unit's firing step, so that the early start of that update still sees the unit in the
   middle of firing?

## Possible observation

S-0215 scenario B45 part (c) as written, with the recorder also logging the S-0107 sequence frame
and the shot ticks: the first per-tick start at which the queued *move* becomes current, compared
with the end of the fire-standing sequence, the fire frame and the end of the rearm countdown. A
second run with a short sequence and a long rearm separates the sequence end from the countdown.

## Current workaround

Not built. Infantry readiness keeps its other conditions; SW12 l9b and the firing-state part of
B45 (c) stay seams (`docs/m8-claims/seams-5b.md`).
