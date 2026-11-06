---
id: Q-2616
title: When an infantry unit's change of target outside the routes S-0215 claim 11 names ends its attack state and asks for stand
status: open
related: [S-0215, S-0107, S-0309, S-0610]
answered-by: []
---

## Need

S-0215 r118 claim 11 (*In the middle of firing*, *Other endings*) and S-0107 r40 claim 11 (*End
of the attack state*) end an infantry unit's attack state at a change of its target, and make that
change ask (not forced) for *deployed*, *prone* or *stand*, in or out of the attack state. They
place the change for four routes: an order (a move, deploy or stop order, or an attack order on
another target), the target's leaving play, the unit's own processing, and claim 33's per-tick
search. The attack-move rule of S-0309 claim 23 is an exception.

Other routes change an infantry unit's target outside its own turn, and neither spec places them:

- a retaliation (S-0215 claim 18, S-0309 claim 18), made at a hit in the turn of the projectile or
  of another actor, which gives the unit *attack* as an override and the attacker as its target
  (S-0215 claim 10);
- a berserk hit's entry (S-0302 claim 10, S-0610 claim 4), which drops the target;
- any other rule that writes the target of an infantry unit from another actor's turn, for example
  a mind-control release (S-0601 claim 9) or a takeover.

Our engine must decide, for each, whether the change ends the attack state at once (so that no
per-tick start of the unit after it sees the state) and makes the request at once, or whether the
unit carries on as if its target had not changed until its own firing turn.

## Questions

1. When a hit makes an infantry unit retaliate, does that change of target end its attack state at
   once and ask for *stand* (or *prone* or *deployed*), as an order's change of target does?
2. Does a berserk hit's drop of the target do the same?
3. More generally, is every change of an infantry unit's target, whatever writes it, a change of
   target for S-0107 claim 11 at the moment it is written, with only S-0309 claim 23's rule as the
   exception?
4. Is the gaining of a target by a unit that had none (an order, a search or a retaliation that
   gives a target to a unit with no target) a change of target for S-0107 claim 11, so that it too
   asks for *deployed*, *prone* or *stand*? Or is only a change from one target, or from a target
   to none, such a change?

## Possible observation

An infantry unit of a computer-controlled house in the middle of firing (S-0215 scenario B45 c's
long *fire-standing*), with a queued behaviour that does not change its target, hit by a second
enemy so that it retaliates. Record its sequence after each update and when the queued behaviour
becomes current, as in B45 c1. Expected under "at once": the *fire-standing* gives way to *stand* at
the read after the hit's tick, and the queued behaviour becomes current at the unit's next first
per-tick start.

## Current workaround

Our engine places only the four named routes and the attack-move exception. A change by any other
route keeps the behaviour our engine had before S-0215 r118: the attack state ends at the unit's next
firing turn, when its target differs from the one it entered the state with, with no request
(`combat::behave::infantry_step`), so both per-tick starts of that update still see the state.

For question 4, our engine counts only a change from one target to another or to none. A unit
that gains a target from none makes no request and keeps its attack state, if it has one
(`infantry::watch_test`). A unit with no target is not in the middle of firing at one, so the
request would only restart a standing sequence.
