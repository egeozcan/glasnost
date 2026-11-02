---
id: Q-2264
title: The periodic held-target check by a materialising vehicle or aircraft
status: answered
related: [S-0309, S-0301, S-0514, S-0207]
answered-by: [S-0309]
---

## Need
S-0309 claim 29 (revision 70) leaves two materialising cases open (Open questions, not rated):
a materialising vehicle or aircraft on which a temporal beam is locked, and one on which no beam
is locked when a rank-4 reason that answers *illegal* or *impossible* holds as well as the
materialising firer's *rearming*, with no order between the two given by S-0301 claim 9. Our
engine must decide what `combat::held::periodic_check` does in both.

## Questions
1. A vehicle or aircraft that is phasing in after a teleport, with a temporal beam locked on it
   and a held target: does it drop the target in a tick whose tick number is a multiple of 16, drop
   it in each of its turns as an object being erased does, or keep it?
2. A vehicle or aircraft that is phasing in, with no beam locked on it, holding a target that is
   hidden from its house (S-0516 claim 24), airborne for its projectile (S-0306 claim 23), or
   protected against its parasite warhead (S-0513 claim 27): does it keep or drop that target in
   such a tick?

## Possible observation
A teleported tank given an attack order on such a target as it lands (for question 2), and the
same with a temporal weapon firing at it while it phases in (question 1); record its target with
tick numbers through the phase-in.

## Current workaround
1. A holder on which a temporal beam is locked is being erased for our engine: it makes no check,
   and as a mobile object being erased it drops its target in each of its turns (S-0215 claim 2,
   S-0514 claim 17).
2. The firing check's order as built: the hidden-target *impossible*, the airborne-target
   *illegal* and the protected-object *illegal* are asked before the materialising firer's
   *rearming*, so the target is dropped.
