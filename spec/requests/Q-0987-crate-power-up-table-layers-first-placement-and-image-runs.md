---
id: Q-0987
title: The power-up table across rules layers, the first placement test of a unit crate, the picture count of an image run, and the stored value of random contents
status: open
related: [S-0520, S-0006, S-0003, S-0112, S-0212, S-0401]
answered-by: []
---

## Need

1. S-0520 Import mapping says a key absent while the `[Powerups]` section exists gives weight 0
   and no animation, keeping the water flag and the argument, and that a layer's short list keeps
   the values it does not give. Our importer sees each key's history across layers but not which
   layers held the section, so "a key absent from a later layer whose section exists" cannot be told
   from "a key absent because that layer has no section".
2. S-0520 claim 13 says the unit is placed "at the crate cell centre at ground height" and, "if
   that fails", the nearby search runs around the crate cell. It does not say what makes the first
   placement fail (impassable land for the type, an occupant in the cell, a collector entering it,
   both).
3. S-0520 claim 24 caps a stage by "the image run's picture count" and treats a run of at most one
   picture as no change. S-0112's table gives each run as (*P*, *F*, *S*) without saying which of
   the numbers is the picture count of the flat run.
4. S-0520 Terms: contents 0 … 18 are fixed, "any other value" is random. Which value a random
   crate holds in the cell's overlay data (it can be read back by a save, a map export or a trigger)
   is not stated.

## Questions

1. Is the table built layer by layer (a layer with the section resets the keys it lacks), or only
   from the final text of each key?
2. Does the first placement fail only on the object-occupancy and land tests of the nearby search
   (so the first attempt equals the search's first candidate), or has it its own test?
3. Is the picture count of a tiberium's flat run *F*, or *F* + *S*, or another number?
4. Which value does a crate placed with random contents hold (a particular number, such as 255 or
   19, or the number given by the placing rule, such as the number slot of a trigger)?

## Possible observation

CR19 with a three-layer rules copy; CR9 with a crate cell holding a vehicle of the collector's own
house; CR18 with a one-picture run; a saved game's overlay data after CR1.

## Current workaround

- `convert::sim_econ::crate_import`: each key's history is applied in order, a missing token keeps
  the value in force; when the section is given in any layer, every key absent from all layers is
  weight 0 with no animation.
- `crates::unit`: the nearby search with the type's class and zone is the only placement test;
  the found cell (the crate cell itself when it qualifies) takes the unit.
- `resource::crate_stage`: the run's *F* is the picture count of the flat run; a stage never reaches
  *F* and never exceeds the existing top stage.
- `crates::RANDOM`: random contents are stored as 255; a trigger's value 19 … 255 stores itself.
