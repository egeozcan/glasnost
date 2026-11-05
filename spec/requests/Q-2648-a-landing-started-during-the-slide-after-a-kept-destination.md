---
id: Q-2648
title: Whether claim 6's exact-cell trigger starts a landing while the aircraft slides on the ground
status: answered
related: [S-0501]
answered-by: [S-0501]
---

## Need

S-0501 r132 claim 7 (Q-2578): after a refusal that kept the destination, or an untested touchdown
that kept it, the aircraft is on the ground by its height, not landing, with its destination, its
flight point and the held height 0 (`quirks.aircraft_refused_kept_drop` on). It makes claim 4's
forward step with its speed fraction following claim 5 toward its flight point, and climbs only
after an update that leaves its target fraction at 1. The claim ends: "Whether claim 6's
exact-cell trigger, which applies at any height, starts a landing while it is still on the ground
is not traced."

The case is common: the flight point is usually one cell away, so claim 5's target fraction falls
to 0 in the flight point's cell before it ever reaches 1.

## Questions

1. While it slides at height 0, standing in its flight point's cell with the same tick's ordinary
   target fraction at 0 and no holding-height mark, does claim 6's exact-cell trigger start a
   landing?
2. If it does, is the next movement update's arrival at height 0 a touchdown (claim 7), with the
   touchdown's own forget test, so that it comes to rest in that cell?
3. If it does not, what does it do in that cell: stop there with fraction 0 and stay on the ground,
   or go on?

## Possible observation

AC31 part h (1) with a touchdown in a cell next to the docking position's. Record each tick from
that touchdown: position, height, speed fraction, landing state and destination, until it rests or
300 ticks have passed.

## Current workaround

Claim 6's text as written: the exact-cell trigger applies at any height, so it starts a landing in
the flight point's cell, and the next movement update makes a touchdown there, which forgets the
destination and the flight point and rests. Cited
`design: S-0501 claim 7 and Q-2648`.
