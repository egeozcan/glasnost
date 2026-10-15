---
id: Q-0738
title: The storm strike's debris test and where its debris appears
status: open
related: [S-0513, S-0303, S-0109]
answered-by: []
---

## Need
S-0513 claim 18 leaves R(2, 4) metallic debris pieces after a strike when:
- the unit nearest the cell centre before the damage was not infantry, and
- either something in the cell changed by the damage (its building, its nearest unit or its
  height level), or the cell held nothing and its land type is road, rock, wall or weeds.

Debris pieces deal what their animations deal (S-0109), so where they appear is simulation
state. Three details are not given.

## Questions
1. How is "the unit nearest the cell centre" measured? Which layers count (ground only, or the
   bridge deck too)? Which kinds count (infantry, vehicles, aircraft; buildings)? Which distance
   is used (planar to the centre, or S-0207 F8's top-corner distance), and how are ties broken?
2. Does "the cell held nothing" mean no object at all in the cell's lists, or no unit and no
   building?
3. At which point is each debris piece created: the strike point (the cell centre at its height
   level plus any bridge lift), the bolt point at the terrain surface, or a point raised by some
   amount as in S-0303 F6's 20 leptons?

## Possible observation
Extend S-0513 scenario SW7 part e. Strike a cell holding a lone vehicle off-centre and a second
infantry unit nearer the centre, then an empty road cell. Record the debris animations created
and their positions.

## Current workaround
Interim rule (docs/development.md, *Simulation (M6b: superweapons)*):
- The nearest unit is the infantry unit, vehicle or aircraft of the cell's ground layer with the
  smallest planar squared distance to the cell centre. Ties go to the first in listing order
  (S-0209 claim 13).
- The building is the first building in that layer's list.
- "Held nothing" means the ground layer lists no object.
- Debris pieces appear at the strike point.
