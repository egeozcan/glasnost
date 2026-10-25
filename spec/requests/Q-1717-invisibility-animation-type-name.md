---
id: Q-1717
title: The built-in invisibility-animation type name that selects the C4Warhead for animation damage
status: answered
related: [S-0109]
answered-by: [S-0109]
---

## Need
M8 step A5 delivers S-0109 claim 17's damage over time in the simulation
(`crates/deterministic/sim/src/anims.rs`, `AnimEnv::advanced`). Claim 17 says the warhead is "one of
two game-wide warheads (Import mapping), chosen by the animation's type name". The Import mapping
gives `C4Warhead` "for the built-in invisibility-animation type, identified by an exact,
byte-for-byte type-name comparison" and `FlameDamage2` "for every other type", and the note after
the table repeats that the comparison is against "its built-in type name". No spec gives that
name, so we cannot tell an animation of that type from any other.

## Questions
1. What is the built-in type name of the invisibility-animation type, as the bytes the comparison
   uses (letter case included)?
2. Is the comparison against the animation's current type name, so that a follow-up switch to or
   from that type (claim 15) changes the warhead from the next advance on?
3. If the rules do not list a type of that name, is the C4Warhead never chosen?

## Possible observation
The spec's AN scenarios with a stock building in the damage area: two animation types with the
same damage value, one named as the built-in type and one not, under rules where `C4Warhead` and
`FlameDamage2` name warheads with different verses against the building's armour. The building's
health after a fixed number of advances shows which warhead each type used.

## Current workaround
Every animation type uses the `FlameDamage2` warhead; no damage is done when it is not given, as
claim 17 says for an absent warhead (interim rule of Q-1717).
