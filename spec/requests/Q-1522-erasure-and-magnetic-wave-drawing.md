---
id: Q-1522
title: What a temporal beam, a unit being erased and a magnetic wave look like
status: answered
related: [S-0514, S-0607, S-0109, S-0102]
answered-by: [S-0514, S-0607]
---

## Need
S-0514 Terms says a temporal beam "need not be visibly drawn" and S-0109 claim 21 covers only
a building's slot animations while it is being erased. Nothing says how a vehicle or infantry
being erased is drawn (translucent, faded by the erasure progress, unchanged), or whether the
attacker's temporal beam is drawn. S-0607 (end of Formulas) says the magnetic wave's drawing
"belongs to the rendering specs", and no rendering spec has it. The lead's review of 2026-10-19
asked for both ("no transparency, no beam" on the chrono capture; "no lift beam" on four
magnetic-lift captures). Our client (`crates/app/src/mapview/live.rs`) draws neither, and the
interim rule below keeps it that way rather than invent a look.

## Questions
1. Is a temporal beam drawn between the attacker and the target while it is locked on? If so:
   from which point to which, in which colours, how thick, and does it flicker or animate?
2. Is a mobile unit (vehicle, infantry, aircraft) that is being erased drawn differently from
   normal: translucent at a fixed level, translucent by its erasure progress, tinted, or not at
   all different? Does the same apply to a building body (S-0109 claim 21 covers only its slot
   animations)?
3. How is a magnetic wave drawn: its shape between the lifter and the seized unit, its colours,
   and how the values of S-0607's wave formulas (the fade-in *a*, the trailing *b* and the visual
   countdown of 100, reset to 64) map onto the picture?
4. In which order are these drawn relative to the layers and the other beam effects of S-0102
   claim 2?

## Possible observation
Attack a vehicle with a chrono weapon and capture every frame of the erasure at the highest
detail. Repeat on infantry and on a building. Seize a vehicle with a magnetic-lift unit and
capture the wave from the first shot until the drop.

## Current workaround
Nothing is drawn for the temporal beam or the magnetic wave, and a unit being erased is drawn as
normal (docs/development.md, *Visual review 2026-10-19*). The erasure's sparkle animation
(S-0514 claim 18) and the building's slot-animation translucency (S-0109 claim 21) are drawn as
their specs say.
