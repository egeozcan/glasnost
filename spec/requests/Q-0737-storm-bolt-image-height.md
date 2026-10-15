---
id: Q-0737
title: The bolt image height H that sets the storm cloud's height
status: open
related: [S-0513, S-0109]
answered-by: []
---

## Need
S-0513 F4 places a storm cloud at *z* = *h*_level + *b*₄ + *h*_c. The term *h*_c depends on *H*,
the pixel height of the image of the first bolt animation in the bolt list. This holds even when
the cloud later uses another bolt.

The simulation's imported animation rules carry no image sizes. They give only the middle frame
μ and the creation draws (S-0109 claims 13, 14 and 25). The cloud height is simulation state,
because the cloud is the animation that strikes. So the importer must pass *H* through, or a
spec must say that the height has no simulation effect.

## Questions
1. Should the importer read *H* from the first bolt animation's image header (the shape file's
   frame height) and keep it in the simulation rules?
2. Does anything in the simulation read the cloud's height? Examples: its strike point (claim 18
   uses the cell's height level, not the cloud), the bolt's position, or the cloud's own
   animation damage. If nothing does, can *h*_c be left to presentation?
3. Which height does a missing image give: *H* = 0 (so *h*_c = −3), or no cloud?

## Possible observation
None needed for question 2 if a spec answers it. For question 1, compare cloud heights recorded
in S-0513 scenario SW8 part c across two bolt images of known heights.

## Current workaround
Interim rule (docs/development.md, *Simulation (M6b: superweapons)*):
- *H* = 0, so *h*_c = −3 leptons for every cloud.
- The F4 function itself is built and tested on the spec's examples, so a real *H* only needs to
  be passed in.
