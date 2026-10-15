---
id: Q-0601
title: Are a building's turret-animation keys read from its rules type or its art image?
status: answered
related: [S-0313, S-0310]
answered-by: [S-0313, S-0310]
---

## Need

S-0313 revision 2 (Import mapping, claim 8) now lists `TurretAnimX`, `TurretAnimY`,
`TurretAnimIsVoxel` and `BarrelAnimIsVoxel` as keys of the **rules** building type. S-0310
revision 3 (Import mapping, claim 4) still lists `TurretAnimIsVoxel` and `TurretAnimX`,
`TurretAnimY` as keys of the **art** building image. The engine reads one value per building
type for both claims: the turret anchor's pixel shift (S-0313 claim 8) and the aiming point
shift (S-0310 claim 4).

Code: the combat importer, `building_combat` in `crates/convert/src/sim_combat.rs`, reads all
four keys from art today.

## Questions

1. For a building type, does the game read `TurretAnimX`, `TurretAnimY` and
   `TurretAnimIsVoxel` from the rules section of the type, from its art image, or from both
   (and then which wins)?
2. Is the answer the same for S-0310 claim 4 (aiming point) and S-0313 claim 8 (turret
   anchor), or do the two uses read different sources?
3. Same question for `BarrelAnimIsVoxel`.

## Possible observation

A turreted building type with `TurretAnimX` set to a large shift in its rules section only,
and a second type with the same shift in its art image only: watch where the turret is drawn
and where its shots start.

## Current workaround

Unchanged: all four keys are read from the art image, as S-0310 lists. Noted as an interim
rule in docs/development.md.
