---
id: Q-0170
title: Where in a building's turn the delayed-kill expiry hit comes
status: open
related: [S-0302, S-0312, S-0203]
answered-by: []
---

## Need

S-0302 claim 16 (revision 11) delivers the expiry hit of a delayed-kill
countdown "on the building's first turn at or after *t* + *K*". S-0312
claim 21 places a building's removal *after* the general part of its turn
(its behaviour, for example the sale's progress). For the expiry hit no
spec says where in the building's turn it comes. A building in a countdown
still takes its whole turn: it can heal, its behaviour runs, and it fires
(S-0302 claim 1: its new shots carry it). The engine has to put the
expiry check somewhere in that turn
(`sim::tick::object_turn`, `combat::damage::delay_kill_expiry_turn`).

## Questions

1. In the building turn in which the countdown has run out, does the
   expiry hit come before or after the building's behaviour, its
   self-healing and its firing in that turn? For example, can a building
   whose countdown ends in this turn still fire a shot in this turn?
2. Does a delayed shot that is pending for that turn (S-0301 claim 14)
   still go off when the expiry hit destroys the building in the same
   turn?

## Possible observation

Use S-0302 scenario D13 part e with a defensive test building that has a
target in range and is ready to fire, and arrange for its countdown to end
on a tick where it would fire. Record whether a shot leaves the building
in the tick of its expiry.

## Current workaround

The expiry check runs at the end of the building's turn, after the general
part (self-healing, behaviour processing, the delayed shot and the economy
events of the turn). This is the same place as S-0312 claim 21's removal.
