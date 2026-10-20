---
id: Q-1540
title: Depth profile and bias of a particle's image
status: answered
related: [S-0102, S-0613, S-0616, Q-1520]
answered-by: [S-0616]
---

## Need
The game client draws each particle's sprite in the air layer (S-0102 claim 3; the interim rule
of Q-1520). S-0102 claim 10 makes everything drawn in the object pass a depth reader, and claim
11 says every object's depth bias contains −*L(z)* of its own position, but claims 12 and 13 list
no depth profile and no depth bias for particles. The client
(`crates/app/src/mapview/build.rs`, `live_particle`) needs both to decide where terrain,
overlays, trees and buildings hide a particle (a gas cloud next to a building, smoke rising
behind a cliff).

## Questions
1. Does a particle's image use the flat or the upright profile? Does it depend on the particle's
   kind (gas, smoke, fire, spark, railgun) or on anything its type sets?
2. What depth bias does it use: −*L(z)* of its own position alone, or with a further constant or
   type field (as animations add their depth adjustment and −2)?
3. Is a particle drawn without any depth test at all in some case (for example a spark or
   railgun particle drawn as pixels or lines, if Q-1520 question 5 says so)?

## Possible observation
A gas cloud released at the foot of a building and on the far side of a cliff on flat, evenly lit
ground; compare where the building or the cliff cuts the cloud's sprite. Repeat with a smoke
column rising above the building's roof.

## Current workaround
Interim rule (docs/development.md, *Visual review 2026-10-19*, *Particles (Q-1520)*): a particle's
image is a depth reader (S-0102 claim 10) with the upright profile and bias −*L(z)* of its
position (claim 11), with no further term.
