---
id: Q-1763
title: The lead's play report against S-0306 claim 23 and S-0309 claim 18: no retaliation at a target a unit cannot otherwise attack
status: open
related: [S-0306, S-0309, S-0214, Q-1761]
answered-by: [S-0306]
---

## Need

This follows Q-1761 and raises its first question. The project lead reports from play of the
original game (M8 step A4, 2026-10-22):

- an anti-air-only defence building never fires at ground units, even when they attack it: it
  cannot respond to a ground attack;
- to the lead's knowledge, no unit or building that cannot attack a target normally gets an
  exception that lets it fire back at that target when attacked.

The approved specs say otherwise in one route. Retaliation (S-0309 claim 18) tests only the firing
check, and S-0306 claim 23 says the firing check "does not test the anti-ground flag for infantry,
vehicle, aircraft or building targets". That absence has only medium confidence. Our engine follows
the specs (`glasnost_sim::combat::acquire::retaliate`, `combat::check`), so in our game an
anti-air-only building fires back at a ground attacker in range. The specs and the lead's report
cannot both be right.

## Questions

1. Does the firing check, or the retaliation rule, refuse a target that the weapon's projectile
   cannot hit by its anti-ground and anti-air flags (a ground object for a projectile that is not
   anti-ground, an airborne aircraft for one that is not anti-air)? If so, does it apply to every
   target kind: infantry, vehicle, building, and a landed aircraft?
2. Is there any case at all where an object fires back at an attacker that its automatic search
   (S-0309 claim 5) and a person's attack order (S-0214 table B, row B7 for buildings) would both
   refuse? If none, say so, so that retaliation can be bounded by the same eligibility.
3. Where else does the absent anti-ground test of S-0306 claim 23 matter: a target kept from an
   earlier order, a force-fire at an object, a script or trigger attack? Does the corrected rule
   apply there too?

## Possible observation

Q-1761's scenario: a test defence building whose only weapon's projectile is anti-air and not
anti-ground; an enemy tank attacks it from inside its range; record whether it fires back over
300 ticks, 5 runs. Repeat with a unit type whose only weapon is anti-air only (if a type exists or a
test type can be made), attacked by infantry and by a vehicle.

## Current workaround

The approved claims as written: retaliation tests only the firing check, so an anti-air-only
building fires back at a ground attacker. The lead's report is recorded in
docs/m8-claims/a4-play-notes.md. No change until this is answered.
