---
id: Q-0241
title: How a turreted building placed at game set-up makes its queued guard current
status: answered
related: [S-0215, S-0009, S-0108]
answered-by: [S-0215]
---

## Need

S-0215 revision 19 claim 1 says that a building placed at game set-up receives the idle body
and *guard* queued, with no current behaviour. By claim 11 a queued behaviour of a building
becomes current only at a per-tick start at which its ready mark is set. For a turreted
building, the specs name these setters of the mark:

- claim 32: a processing of an armed building in *guard*, *sticky* or *area guard*, or of
  *attack* with a target. While no behaviour is current, the building processes as *none*
  (claim 4), which does nothing;
- the construction state (S-0406 claim 17), which a set-up building never enters (S-0009
  claim 40);
- the body-range rule of claim 11, which for a turreted building applies only in
  *construction* or *selling*;
- "other structures rules (repair, selling and others)", which are not named.

None of these is known to reach a turreted building fresh from set-up. Followed literally,
its *guard* would stay queued for ever. A map-placed armed defence with a turret would then
never process *guard*, never take a target through its own processing and never fire. The
automatic search of claim 33 sees the queued *guard* (claim 1) and can set a target, but no
processing fires at it while the current slot is empty. Our engine needs a rule that lets such
a building start. This is in `econ::body::set_up` (crates/deterministic/sim/src/econ/body.rs).

## Questions

1. Which rule sets the ready mark of a turreted building placed at game set-up, or otherwise
   makes its queued *guard* current? On which tick does that happen?
2. Does the answer differ for an armed and an unarmed turreted building, or for a building
   whose turret comes from an upgrade placed at set-up?
3. Is S-0009 claim 40's "starts in the *guard* behaviour" meant as *guard* queued and seen
   (claim 1), or as *guard* current from the first tick?

## Possible observation

S-0215 scenario B33 with a named test type that has a turret and a weapon, placed by the map,
and an enemy unit placed in its range. Record its current and queued behaviours and its ready
mark (read-only recorder) each tick from entry until it first fires, and the tick of that first
shot. Repeat without the weapon, and with the turret from a set-up upgrade.

## Current workaround

At game set-up a turreted building also gets its ready mark, so its queued *guard* becomes
current at its first per-tick start. A building without a turret follows the body-range rule
of claim 11. Interim row in `docs/development.md`.
