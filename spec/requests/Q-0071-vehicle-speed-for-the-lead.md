---
id: Q-0071
title: The target vehicle's "current speed" in the lead of a shot
status: answered
related: [S-0306, S-0207]
answered-by: [S-0306, S-0207]
---

## Need
S-0306 claim 3 and F6 lead a shot at a moving vehicle by
ℓ = trunc(D₃ / (0.9 w) × u), with *u* "the vehicle's current speed (leptons
per tick, integer)". The driving locomotor of S-0207 expresses movement as
progress units per tick (F2) along tracks, with a double speed fraction, and
no approved spec defines a speed in leptons per tick for a vehicle
(`combat::projectile::lead`).

## Questions
1. For a vehicle moved by the driving locomotor, what integer is *u* in
   S-0306 F6: the progress per tick of S-0207 F2 for the current tick, the
   type speed times the speed fraction, a distance actually covered in the
   last tick, or something else? How is it rounded?
2. What is *u* for a vehicle that is turning in place before a step (S-0206
   claim 12) or waiting, while its locomotor still counts as moving?

## Possible observation
S-0306 scenario P2 (the lead) with a target vehicle driving straight across
the firer's line at two known speed fractions; the impact points give *u*.

## Current workaround
*u* = 0, so no shot is led (the aim point is the target's reference point).
