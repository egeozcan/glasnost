---
id: Q-0984
title: Where temporal erasure ranks among the special detonation effects, what "idle behaviour" means for a beam, and a firer with no beam
status: answered
related: [S-0514, S-0518, S-0601, S-0519, S-0614, S-0215]
answered-by: [S-0514]
---

## Need

Three gaps in S-0514 that the simulation has to decide.

1. S-0514 claim 1 says an erasing shot is one whose warhead has the temporal flag "when temporal
   erasure is the detonation effect selected. Other special-effect flags can take precedence".
   S-0601 claim 5 ranks takeover above temporal erasure, and S-0518 claim 11 ranks mind control and
   bomb planting above electric charging. No spec ranks temporal erasure against the parasite
   attachment, air-strike designation and electric-charge effects, or against bomb planting and
   disarming. A warhead can carry the temporal flag together with one of them.
2. Claim 14 says an attached attacker lets go "entering idle behaviour", and that removing the
   target makes its attackers "enter idle behaviour". S-0215 does not define an idle behaviour:
   *stop* leaves the behaviour unchanged (S-0213 claim 8), and a unit that has lost its target goes
   to rest through S-0215's fall-back (claims 13 to 15). Scenario E10 part b ("stop") expects a
   release, while claim 16 says clearing the ordinary target alone does not detach the beam.
3. Claim 3 and the Terms give a beam to a firer whose primary weapon's warhead is temporal. A firer
   whose temporal weapon is only its secondary has no beam (S-0310 claim 7). What a delivered
   erasing shot of such a firer does (it can happen: a secondary temporal weapon, or an elite
   replacement) is not said: no lock, or a lock without a beam, and whether the target-side effects
   of claim 2 still happen. Also not said: what happens to an attached beam when its holder boards
   a transport that is not a gunner vehicle.

## Questions

1. In what order do the special detonation effects stand: takeover, bomb planting, bomb disarming,
   temporal erasure, parasite attachment, air-strike designation, electric charging, locomotor
   replacement, nuke-making? Which of them can a single warhead trigger together?
2. Which behaviour or event is the "idle behaviour" of claim 14: the fall-back to rest of S-0215
   claims 13 to 15 (guard, and the like), the behaviours *stop* and *sleep*, or something else?
   Does the stop order produce it (E10 part b)?
3. A delivered erasing shot from a firer that has no beam: does it lock, do nothing, or only have
   claim 2's side effects?
4. A holder with an attached beam that boards a non-gunner transport: does the beam let go?

## Possible observation

E10 with a firer given the *stop* order, a firer that is moved by a *sleep* team order, and a firer
whose own target dies (record whether and when the target's lock ends); a firer whose temporal
weapon is secondary only, with a warhead that also carries the parasite flag or the air-strike flag.

## Current workaround

- `projectile::wh_temporal`: the erasing effect is selected when the temporal flag is set and the
  parasite, air-strike and electric-charge flags are not; the takeover, bomb and locomotor effects
  are not built.
- `process::fall_back` lets a beam go (it is the one place a unit goes to rest), and a removed
  target puts its attackers through it; the *stop* order alone does not release.
- `temporal::deliver`: a firer with no beam makes no lock and no side effect; boarding a
  non-gunner transport does nothing to a beam.
