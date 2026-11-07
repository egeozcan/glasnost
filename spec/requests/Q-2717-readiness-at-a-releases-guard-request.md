---
id: Q-2717
title: When a release reads readiness for its guard request
status: answered
related: [S-0601, S-0107, S-0215]
answered-by: [S-0601]
---

## Need

S-0601 claim 9 says that at a release "the ownership change clears the target and destination and
normally requests *guard* with a readiness-dependent immediate start (S-0215 claim 7)". S-0107 r46
claim 11 (*Changes of target*) now says that every change of an infantry unit's target, the loss of
its target at a release included (S-0601 claim 9), ends the unit's attack state at the moment it is
made. S-0215 r118 claim 11 makes an infantry unit in that attack state not ready to change
behaviour.

Neither spec says whether the readiness that the guard request reads is the unit's readiness before
the release clears its target, or after it. For an infantry unit released while it is in the attack
state the two readings differ: read before the clear, the attack state holds the unit back and the
guard request gets no immediate start; read after it, the clear has ended the attack state and the
request can start at once. For any unit that holds an *attack* target while it moves, the reading
also decides whether S-0215 claim 11's "attack without a target" exception applies.

## Questions

1. At a release (S-0601 claim 9), is the readiness of the guard request read before the target and
   destination are cleared, or after?
2. If after: for an infantry unit, does the attack state that the clear ends (S-0107 claim 11) still
   count as "in the middle of firing" for that readiness?

## Possible observation

S-0601 scenario MC4, with the victim an infantry unit that has just fired at a target with a long
rearm countdown, so that it is in S-0107 claim 11's attack state, when its controller is destroyed.
Record the tick in which *guard* becomes current and the victim's first per-tick start after the
release. A guard that becomes current in the release tick reads readiness after the clear; one in
the victim's next behaviour processing reads it before.

## Current workaround

Our engine reads readiness before the clear, as it did before S-0107 r46, and then ends the attack
state at the clear (`mind::release_orders`). Cited `Q-2717` there.
