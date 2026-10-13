---
id: Q-0061
title: Combat import details - a type's threat coefficient before any layer, the defence category word, and name and integer list items
status: answered
related: [S-0309, S-0302, S-0003, S-0303, S-0312]
answered-by: [S-0312, S-0003, S-0302, S-0303, S-0309]
---

## Need
The importer (`convert::sim`, the combat fields) reads the combat specs'
Import mappings. A few readings are not stated precisely enough to import
without a choice of our own.

## Questions
1. Threat coefficients of a type (S-0309 Import mapping and Edge cases,
   `quirks.coefficient_zero_replaced`). A coefficient key absent from a layer
   keeps the held value, except that a held 0 takes the game-wide default "as
   it stands when the type is read". What is the held value of a type's
   coefficient before any layer gives the key: 0 (so the first reading of the
   type's section without the key copies the game-wide default), or the
   game-wide default itself, or something else? And with the switch off, does
   a type whose section never gives the key keep that starting value?
2. Defence buildings (S-0302 claim 5, Import mapping: `BuildCat` "word
   (structures specs)"; S-0004 Import mapping: combat = `Combat`). How is the
   word compared: as a whole, ignoring the letter case of A to Z, after the
   text is cut to some length? What does a value that names no category give:
   the category in force, or a fixed category?
3. List items (S-0003 claim 12 and 14). In the animation-name lists of the
   combat specs (`AnimList`, `SplashList`, `DebrisAnims`, `Explosion`,
   `DestroyAnim`, `DamageFireTypes`, `MetallicDebris`) and the voxel-animation
   lists (`DebrisTypes`, `BarrelDebris`), is an item `none` or `<none>` left
   out, or does it hold a place (so that a draw over the list can pick it)?
   In the integer list `DebrisMaximums`, how is an item read (claim 3's integer
   form, or a plain decimal prefix), and does an unreadable item hold a place
   with 0?

## Possible observation
1. A test rules file with `MyEffectivenessCoefficientDefault` given and a
   vehicle type whose section never gives `MyEffectivenessCoefficient`, and a
   second run where the default is given only in a map, after the type's
   section was read in the base rules: compare which target the type picks
   between two candidates whose scores differ only in that term.
2. Building types with `BuildCat=combat` and `BuildCat=Combatx`: does the
   defence armour factor of a country apply to them?
3. A warhead whose `AnimList` is `none,X` and whose damage picks the first
   entry by S-0303 F5: is an animation shown?

## Current workaround
1. The importer starts each type's coefficient at 0 (citing this request),
   so with the switch on the first reading without the key copies the
   game-wide default, and with it off the 0 is kept.
2. `BuildCat` is read as a named value (S-0003 claim 16: the text cut to the
   common limit, compared whole ignoring case); a value that is not `Combat`
   gives "not a defence".
3. `none` and `<none>` items are left out of name lists; an integer list item
   is read by claim 3's integer form, an item that form cannot read gives 0.
