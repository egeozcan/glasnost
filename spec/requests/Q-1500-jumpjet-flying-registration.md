---
id: Q-1500
title: When a jumpjet is a tracked flying object, and the double entry of a low one
status: open
related: [S-0303, S-0502]
answered-by: []
---

## Need
S-0303 Terms make "a jumpjet between lift-off and landing" a tracked flying object, which the
above-terrain detonations of claim 6 read, and leave the registration boundaries to the movement
specs. S-0502 (Open questions, *Flying-object registration*) says a jumpjet is in the registry
"from take-off until it lands or crashes" but no claim fixes the boundaries. S-0303's Open
questions also leave open whether a jumpjet that is both cell-listed (S-0502 claim 4, below
208 leptons) and tracked can be hit twice by one detonation.

Without registration a cruising or hovering jumpjet above 208 leptons is in no cell list and in
no flying registry, so no detonation can hit it. The oracle arena showed this: in M-0001 the
pairing DISK against FV (stock: the column wins 12 of 12) gave our engine a row win in 12 of 12
games with the DISK group untouched, because every FV detonation next to a DISK found nothing.
Code: `crates/deterministic/sim/src/jump.rs` (`took_off`, `settle`, `leaving`, `move_to`).

## Questions
1. At which moment does a jumpjet start to be found by the flying-object search of S-0303
   claim 6: in the update in which it leaves the grounded state (S-0502 claim 5), at its first
   rise above the surface, or at some height?
2. At which moment does it stop: at touchdown (S-0502 claim 15), at the start of a crash
   (claim 18), at the crash's impact, or when it leaves play?
3. Does a jumpjet below 208 leptons that is both listed in its cell and found by the
   flying-object search take two hits from one detonation, or one?
4. Is a unit of a non-jumpjet type that a magnetic beam lifts with a temporary jumpjet
   locomotor (S-0607) found by the flying-object search while it is in the air?

## Possible observation
A jumpjet vehicle hovering at several heights (just above 0, below 208, above 208) next to a
detonation of a small-spread warhead above the terrain, recording its health after each; the
same during take-off, landing and a crash; and a vehicle lifted by a magnetic beam.

## Current workaround
The literal composition of S-0303 Terms and S-0502: a unit of a jumpjet type is tracked from the
update in which it leaves the grounded state (claim 5) until it settles on the ground (claim 15's
touchdown, or the set-down of S-0607) or leaves play; a crashing jumpjet stays tracked until its
impact removes it. A low jumpjet that is also cell-listed can be a candidate twice. A unit lifted
by a magnetic beam is not tracked.
