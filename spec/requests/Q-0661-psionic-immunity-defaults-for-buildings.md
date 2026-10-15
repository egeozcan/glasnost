---
id: Q-0661
title: Default of the two psychic immunities for building types (S-0610 against S-0302)
status: answered
related: [S-0610, S-0302]
answered-by: [S-0610]
---

## Need
S-0610 claim 1 says every building type has psionics immunity and psychic-weapon immunity on by
default (absent key), and off by default for infantry, vehicle and aircraft types. S-0302's
Import mapping lists the psychic-weapon immunity and psionics immunity keys with default *no* for
unit and building type sections alike. The importer (`crates/convert`, rules schema defaults)
and the psychic-damage row of S-0302 claim 8 (`combat::damage`) need one default for an absent
key on a building type.

## Questions
1. For a building type whose section omits the psychic-weapon immunity key, is a psychic-damage
   hit on it ignored (S-0610's default on) or does it apply (S-0302's default no)?
2. The same for the psionics immunity key; S-0302 claim 10 already excludes buildings from
   berserk hits, so which other consumers (control, the firing checks of S-0610 claim 3) see the
   building default?

## Current workaround
The code keeps S-0302's mapping: both keys default to no for every type, and buildings are
excluded from berserk hits by S-0302 claim 10's building row. S-0610 is wave 2 and not built.
