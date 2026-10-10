---
id: Q-0005
title: Which rules fields create types by reference, and in which order
status: answered
related: [S-0004, S-0006, S-0007]
answered-by: [S-0004]
---

## Need
S-0004 says that weapons and projectiles exist only by reference, that every
kind except superweapons, tiberiums and countries can gain types by
reference (claim 3), that a referenced type's index and whether it reads its
section depend on what first names it and when (claims 4 and 6), and it
names a few reference fields (Import mapping: `SuperWeapon`, `DeploysInto`,
`AirburstWeapon`, `ShrapnelWeapon`, `HoldsWhat`, `CrateImg`, `WoodCrateImg`,
`WaterCrateImg`, `UndeploysInto` in S-0007). To build the registries of every
kind, the importer needs the complete list. Code gap: the registration pass
in `glasnost_convert` (types created by reference other than through the
fields above are not created, and their kinds' registries are incomplete).

## Questions
1. For each type kind (buildings, vehicles, aircraft, infantry, weapons,
   projectiles, warheads, animations, voxel animations, particles, particle
   systems, terrain objects, smudges, overlays, countries, sides), which keys
   of its own section name a type, of which kind, and does a reading of the
   key create the type if it does not exist yet (lookup with creation) or
   only find it?
2. The same for every game-wide section that names types (S-0004 claim 4,
   rows 2 and 4: `[General]`, `[AI]`, `[CrateRules]`, the combat-damage
   section, `[Radiation]`, `[ElevationModel]`, `[WallModel]`, `[AudioVisual]`,
   `[SpecialWeapons]`, the difficulty sections). Which keys, which kinds,
   create or find? Which is the combat-damage section's name?
3. For keys whose value is a list of type names (for example a list of
   animations or of weapons), does each item create its type, in item order?
4. S-0004 open question 4: the order among several types first created by
   one section (one type's section, or one game-wide section), stated as an
   observable order or as "unspecified, use this deterministic order".
5. S-0004 open question 5: where types first created during the second
   reading of warheads fall in index order, and whether they read their own
   section in that layer.
6. Which keys read a type name with a length limit other than the 31-byte
   list limit (S-0004 claim 8 says many fields allow longer names), and what
   is each limit?

## Possible observation
Overlay indices are visible through map overlay data (S-0004 scenario G); a
test map whose crate rules and a type's sections each name new overlays in
different orders would show the creation order among them.

## Current workaround
The importer creates types by reference only through the reference fields
the approved specs name (listed above), with the rank rule of S-0004 claim 6.
Every other reference stays text in the converted tree and creates nothing;
the conversion report counts such keys as unsupported. No registration index
is derived from an unspecified order.
