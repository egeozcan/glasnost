---
id: Q-1600
title: Where a turreted voxel vehicle's shot animation sits against its drawn barrel
status: answered
related: [S-0301, S-0313, S-0105, S-0109, S-0101]
answered-by: [S-0313]
---

## Need

The 2026-10-19 screenshot review saw a gattling vehicle's shot animation drawn "low on the hull,
below and to the right of the barrel tips". We traced one shot of a turreted voxel vehicle on the
local lane (turret facing east, body facing north, flat ground, height level 6) through every step
the specs give, and found no step that departs from them:

1. The fire point is S-0313 claim 1 and F1 with the imported muzzle triple: forward along the
   turret, the lateral part negated for the odd burst counter, the height added. Both burst
   counters land symmetrically about the turret's axis at the imported height.
2. The shot animation is created at that point and attached with that displacement (S-0301 claim
   22, S-0109 claim 22); the trace shows the animation's position equal to the fire point.
3. It is drawn centred on the projection of that position with S-0101's height lift, moved down by
   the type's vertical draw shift (S-0109 claim 26, S-0102 claim 19). The drawn frame's stored
   rectangle is where its canvas offsets put it.
4. The vehicle's voxel turret is drawn by S-0105 with its imported turret displacement and the
   pixels-per-lepton scale ρ, which maps the muzzle triple's leptons to the same model units.

What remains is a few pixels between the end of the drawn barrel (which comes from the voxel art)
and the shot animation (which comes from the rules' muzzle triple): the animation sits about 5
pixels further along the turret's direction and about 2 pixels lower than the barrel's last
pixels in our picture. We cannot tell whether the original shows the same gap, because no spec
or measurement records where the animation appears against the drawn turret.

## Questions

1. For a turreted voxel vehicle on flat ground, with its turret turned to each of the eight
   8-directions in turn, at which screen pixel is its shot animation's draw point, counted from
   the vehicle's own draw point? (A screenshot pair at each facing would settle it.)
2. Is any shift applied to a shot animation's draw point, or to the voxel turret's drawing, that
   S-0109 claim 26, S-0101 claims 1 to 5, S-0313 claim 1 and S-0105 claims 8 and 9 do not
   already give? For example a height or depth correction that depends on the turret facing.

## Possible observation

A stock turreted voxel vehicle with a multi-entry shot animation list, ordered to fire at a
target placed in each of the eight directions in turn, with the game paused on the frame of the
shot. Compare the animation's centre with the vehicle's centre pixel and with the barrel's last
drawn pixel.

## Current workaround

None needed: the code follows the specs as they stand (fire point by S-0313 F1, animation at
the launch point, centred draw point with the vertical draw shift). No offset is invented. The
lane trace (`GLASNOST_TRACE_SHOTS`, `GLASNOST_TRACE_ANIMS`, `GLASNOST_TRACE_BOX` in
`crates/app/tests/m7a_lane.rs`) prints the fire points, the animations' positions and names and
the items drawn in a screen box, for a re-check once this is answered.
