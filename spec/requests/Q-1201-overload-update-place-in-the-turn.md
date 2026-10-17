---
id: Q-1201
title: Where in a controller's own turn the overload update happens
status: answered
related: [S-0601, S-0610, S-0203]
answered-by: [S-0601]
---

## Need
S-0601 claim 16 says an unlimited controller's overload update happens in
its own turn, and the State and ordering section says the pulse happens in
the controller's own turn. It does not say where in the turn, relative to
the other per-object work (berserk countdown, self-healing, behaviour
processing, firing). The order decides whether the pulse damage or its
random draws come before or after that tick's shot, healing or other draws
of the same object.

## Questions
1. In the tick of a damaging pulse, does a controller that also self-heals
   show the heal before or after the pulse damage?
2. Does a controller that fires in the same tick as its pulse fire before
   or after the pulse (for example: does a lethal pulse still let that
   tick's shot leave)?
3. Does the overload update still run in the ticks where an object's turn
   is otherwise suppressed (for example while it is arriving from a
   teleport, or while it is inside a transport)?

## Possible observation
Give an unlimited controller self-healing and a low health, let it take
victims until a lethal pulse is due while it is attacking, and record per
tick its health, its shots and its victims' owners.

## Current workaround
The update runs right after the berserk countdown of S-0610 (Q-1180), after
the bookkeeping and before self-healing, sinking and behaviour processing
(`tick.rs`, `mind::overload_turn`).
