---
id: Q-3007
title: Does the shadow of a vehicle drawn as a sprite with a turret show while it is above the surface, and under which type flags?
status: open
related: [S-0102]
answered-by: []
---

## Need

S-0102 claim 21 draws the own shadow of a vehicle drawn as a sprite only while its height above
the surface is 0. Its type must not suppress shadows and must not be either visceroid kind. The
claim then says that a vehicle type with a turret, drawn as a sprite, is drawn differently
(claim 22).

Claim 22's turret path gives the shadow frame (the body frame plus ⌊*N*/2⌋), the bias of claim 13
and its place under the combined image. Claim 13's row for this shadow is *U* − 2, flat, with no
height condition. Neither claim says whether the height condition or the two type conditions of
claim 21 also hold on the turret path.

The client (`crates/app/src/mapview/build.rs`, `Painter::sprite_vehicle`; render issue #95 on
GitHub) now leaves out the own shadow of a non-turret sprite vehicle while *h* is not 0, as claim 21
says. It needs to know what the turret path does.

## Questions

1. Is the shadow of a vehicle drawn as a sprite whose type has a turret drawn while its height
   above the surface is not 0 (in the air, or falling)?
2. If it is, where does it lie: under the vehicle's draw point at its height, or on the ground
   below it? Is its bias still *U* − 2? (Claim 14 says sprite vehicles use *U* only at *h* = 0.)
3. Do the type's shadow-suppressing flag and the two visceroid kinds of claim 21 also suppress the
   turret path's shadow?

## Possible observation

Use S-0102 scenario D29 (c)'s turret vehicle (the turret flag set through the override section).
Lift it off the ground, for example by carrying it or dropping it, and capture its shadow during the
lift. Repeat with the type's shadow-suppressing flag set, and with each visceroid flag set.

## Current workaround

The client keeps the turret path's shadow as before, at every height, with bias *U* − 2 (claim 13's
row). The type's shadow-suppressing and visceroid flags suppress it, as on the non-turret path. The
combined image keeps *U* at every height (S-0105 claim 12, *Depth*). This is recorded in
`docs/m8-claims/render-issues.md` (batch R4, #95).
