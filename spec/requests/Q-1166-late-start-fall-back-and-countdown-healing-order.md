---
id: Q-1166
title: Whether the late start falls back to rest, and where self-healing falls against the berserk countdown
status: answered
related: [S-0215, S-0610, S-0314]
answered-by: [S-0215]
---

## Need
S-0215 revision 57 claim 11 gives infantry and vehicles a late start: one more ready-and-start
chance in the same update, after that update's movement. It says the late start makes a queued
behaviour current. Claim 11 also says that an infantry unit that is ready and has neither a
current nor a queued behaviour falls back to rest at its *first* per-tick start. Whether the late
start also does that fall-back "is not established and is not asserted" (claim 11 and Open
questions). The engine still has to do one or the other. The code is
`glasnost_sim::process::late_start`, called from `tick::object_turn` after the locomotor step.

S-0610 revision 8 places the berserk countdown before the update's processing, searches and shots,
and after the first per-tick start (claim 5, Effects). Its Open questions leave open whether
self-healing (S-0314 claims 9 and 10) comes before or after the countdown step in the same update.
The engine runs self-healing in the unit's own update before the behaviour part, so the countdown
now comes after self-healing (it used to come before, under Q-1180's interim rule).

## Questions
1. An infantry unit is ready to change at the end of its update, after moving in that update. It
   has neither a current nor a queued behaviour then. Does it fall back to rest in that same
   update, so that a recorder reading after the update sees a resting behaviour current or
   queued? Or does the fall-back wait for its next update?
2. Within one update of a damaged berserk unit that heals itself, does its self-healing come
   before or after its berserk countdown step? Is there any observable difference, given that
   self-healing is not a berserk hit?

## Possible observation
1. An infantry unit whose current behaviour ends during its own movement, with nothing queued,
   for example by S-0215 B46's setup with the queued slot emptied. Read its current and queued
   behaviour after the last update of the tick and before the first update of the next tick.
2. S-0610 claim 6 already notes that healing does not read the berserk state. A setup that
   observes the order, if one exists, would need a berserk unit with self-healing whose remaining
   time and health are recorded within one tick.

## Current workaround
1. The late start only makes a queued behaviour current. It does no fall-back to rest. The
   fall-back stays at the first per-tick start of the next update, as claim 11 states for it.
2. The countdown step runs in the behaviour part of the update, after the first per-tick start.
   That puts it after self-healing and the radiation hit. Healing reads no berserk state, so no
   known observable depends on this order.
