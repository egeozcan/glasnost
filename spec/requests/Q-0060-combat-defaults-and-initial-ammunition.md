---
id: Q-0060
title: Absent-key defaults of some combat values, and a new object's ammunition
status: open
related: [S-0301, S-0303, S-0312]
answered-by: []
---

## Need
The combat specs leave a few values without an engine default, saying that
"the rules specs", "the crate specs" or "the bridge specs" own them, and one
starting value without its rule. The simulation (`combat::data::CombatGeneral`,
`combat::fire::initial_ammo`) needs each to have a value when no rules layer
gives the key, and must not invent one.

## Questions
1. A new object's ammunition (S-0301 claim 10, Import mapping): the spec says
   a firer "starts with its type's initial ammunition" and that how a new
   object's ammunition is set from the initial-ammunition field and the
   maximum ammunition field belongs to the production specs. For a type whose
   maximum ammunition is 0 or more and whose initial-ammunition field is absent
   (−1), what ammunition does a new object of the type have: −1 (unlimited),
   the maximum, or something else? And for an initial value above the maximum?
   Does an object placed by the map start differently from one produced?
2. What does the game use when no rules layer gives these keys:
   - the crew escape chance (`CrewEscape`, S-0312 claim 13),
   - the three survivor divisors (`AlliedSurvivorDivisor`,
     `SovietSurvivorDivisor`, `ThirdSurvivorDivisor`, S-0312 F5),
   - the sinking weight (`ShipSinkingWeight`, S-0312 claim 10),
   - the bridge strength (`BridgeStrength`, S-0303 claim 18),
   - the ammunition-crate damage (`AmmoCrateDamage`, S-0303 claim 19)?

## Possible observation
For 1: a test rules file giving a vehicle type `Ammo=4` and no
`InitialAmmo`; count its shots before it answers out of ammunition, for a unit
placed by the map and for one produced in a factory. For 2: a test rules file
that removes each key from every layer, with a scenario of the owning claim.

## Current workaround
1. `combat::fire::initial_ammo` gives a new object the initial-ammunition
   field as it stands (absent: −1, unlimited), the literal reading of S-0301
   claim 10.
2. These values are held as "not given" (`Option::None`) when no layer gives
   them, and the rule that needs one does not take effect (no crew escape, no
   survivors, no sinking, no bridge destruction, no barrel detonation damage
   other than 0); the world records a note when that happens. Stock rules data
   gives every one of these keys, so imported games are not affected.
