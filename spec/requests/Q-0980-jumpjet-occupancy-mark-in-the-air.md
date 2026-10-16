---
id: Q-0980
title: The occupancy mark of a jumpjet that is ascending, descending or crashing
status: open
related: [S-0502, S-0209]
answered-by: []
---

## Need

S-0502 claim 4 says that in every state but hovering and cruising the mark is "lifted and laid
again at its current position in each physics step", and also that when a jumpjet leaves the ground
its mark is lifted; scenario J3 expects a ground vehicle blocked while the jumpjet is ascending
in its cell and the mark "released as it first rises off the surface". Read together, an ascending
jumpjet high above a cell would either block the cell below or not.

## Questions

1. Does an ascending jumpjet that has risen off the surface mark the cell under it (and a
   descending one before its landing reservation, and a crashing one)?
2. If it does not, is the mark laid again at touchdown only through the landing reservation?

## Possible observation

A ground vehicle ordered into the cell under an ascending jumpjet at heights of 1, 50 and 300
leptons; the same under a descending one without and with its reservation.

## Current workaround

The mark is laid only while the unit stands on the ground (height above the surface 0 or less) and
is not hovering or cruising; it is lifted at the first rise; a descending jumpjet holds only its
landing reservation until touchdown (`jump::fly::occupancy_step`).
