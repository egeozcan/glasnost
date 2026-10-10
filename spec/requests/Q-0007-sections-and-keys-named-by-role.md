---
id: Q-0007
title: Names of the sections and keys that the layering rules refer to by role
status: answered
related: [S-0004, S-0006]
answered-by: [S-0006]
---

## Need
The importer applies S-0006's per-key layering, which has exceptions for
particular sections and keys. Some of them are named only by role, so the
importer cannot find them in the user's files. Code gap: the layering pass
of `glasnost_convert` (these exceptions are not applied; see Current
workaround).

## Questions
1. Which are the twelve land-type sections of S-0006 claim 5 (Import
   mapping names only `[Clear]` and `[Road]`)? The importer needs their exact
   section names to apply the land-type resets.
2. Which section is the power-up section (S-0004 Import mapping, early
   game-wide sections), and which is the combat-damage section (S-0004 claim
   4, row 4; S-0006 Terms)?
3. S-0006 claim 3 says that angle keys and animation-rate keys treat an
   explicit −1 as "not given". Which keys are they, per type kind and per
   game-wide section, and is −1 compared after the key's value form is
   applied (so that, for example, `$FFFFFFFF` also counts)?
4. Are there other keys whose explicit value counts as "not given", besides
   a superweapon's recharge time (0) and the keys of question 3?

## Possible observation
For question 1, a test map that sets only the buildable key of a candidate
section and checks whether a vehicle's speed over that terrain changes
(S-0006 scenario D2's method), once per candidate name.

## Current workaround
The land-type resets are not applied: an absent land-type key keeps the
value in force, and the import report says so. Sections whose role is not
known are layered like every other game-wide section. Only the recharge
time's "not given" value (S-0006 Import mapping) is applied.
