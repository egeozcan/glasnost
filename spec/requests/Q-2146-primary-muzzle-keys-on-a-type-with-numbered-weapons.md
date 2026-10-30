---
id: Q-2146
title: Do the primary and secondary muzzle-position keys apply to a type with numbered weapons?
status: answered
related: [S-0313, S-0308]
answered-by: [S-0313]
---

## Need
S-0313 revision 37's Import mapping gives the muzzle position of the primary and secondary slots
from `PrimaryFireFLH` and `SecondaryFireFLH` (and the elite forms), and the muzzle position of
numbered weapon *n* − 1 from `Weapon`*n*`FLH` (absent: 0, 0, 0, claim 1a). S-0308 claim 1 says a
type whose turret count is above 0 fills its slots from the numbered weapon fields and "neither
form reads the other's keys", but that sentence is about the weapon keys. For a type with numbered
weapons, slots 0 and 1 are numbered weapons 0 and 1, and both key families could give their muzzle
positions. No text says which one applies there, or what happens when only one family is given.

The code that decides is the muzzle-position table of `combat::data` (`flh`, `elite_flh`): today a
type with numbered weapons and no `Weapon`*n*`FLH` key at all takes slots 0 and 1 from
`PrimaryFireFLH` and `SecondaryFireFLH`, and its elite slots 0 and 1 from the elite pair when it
has no `EliteWeapon`*n*`FLH` key. For any type, whatever its turret count, once a `Weapon`*n*`FLH`
key is given only the numbered keys count for its non-elite slots, `PrimaryFireFLH` and
`SecondaryFireFLH` included in "not read" (question 3).

In the local lane (counts only) no type of the reference rules reaches the first case: the types
with numbered weapons there all have numbered muzzle keys. The case of question 3 was not counted.
A mod or map rules layer can reach both.

## Questions
1. For a type whose turret count is above 0, do `PrimaryFireFLH` and `SecondaryFireFLH` give the
   muzzle positions of its numbered weapons 0 and 1 in any case: when no `Weapon`*n*`FLH` key is
   given at all, when `Weapon1FLH` or `Weapon2FLH` is absent but others are given, or never?
2. The same for the elite forms: does `ElitePrimaryFireFLH` give the elite version of numbered
   weapon 0's muzzle position in any of those cases, and if so, which wins over claim 1a's
   fall-back to weapon 0's muzzle position?
3. For a type whose turret count is 0 or less, does a `Weapon1FLH` or `Weapon2FLH` key ever give
   its slot 0 or 1 a muzzle position?

## Possible observation
A test vehicle with a turret count of 2, two numbered weapons, no turret and no turret
displacement, `PrimaryFireFLH` and `SecondaryFireFLH` with distinct non-zero forward and lateral
parts, and no `Weapon`*n*`FLH` key; record the start position of each weapon's shot at body
facings 0 and 16384 (S-0313 P1's procedure). Repeat with `Weapon2FLH` given and `Weapon1FLH`
absent; then elite from the start with `ElitePrimaryFireFLH` given and no `EliteWeapon`*n*`FLH`.

## Current workaround
The code keeps its behaviour from before revision 37, described under Need, and claim 1a's
fall-backs apply on top of it. Comment `Q-2146` in `combat::data`; ledger row in
`docs/m8-claims/intake51-S-0306-S-0313.md`.
