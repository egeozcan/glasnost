---
id: Q-0742
title: Superweapon aim cursor images, the protection tint look and other superweapon looks
status: answered
related: [S-0512, S-0513, S-0214, S-0410, S-0104, S-0007]
answered-by: [S-0512, S-0214]
---

## Need
S-0513 claim 1 gives where each superweapon kind is offered and that a force shield shows the
no-force-shield action elsewhere. S-0410 claim 12 table C1 gives which kinds the radar offers.
Neither gives the cursor images. S-0214 claims 21 and 22 give the images for unit and sidebar
mode actions only. S-0513 claim 29 says a protected object shows the protection tint, "rendering
specs", but no rendering spec gives that tint. The client cannot draw these without guessing.

## Questions
1. For each superweapon aim action (nuclear strike, lightning storm, iron curtain, chronosphere
   source pick, chronosphere destination, force shield, paradrop, spy plane), which cursor image
   is shown on the battlefield and on the radar? Give the file, the frame range, the frame rate
   and the hot spot, as S-0214 table C does for other actions.
2. Which image does the no-force-shield action use?
3. While a warp destination is armed by a source pick (S-0513 claim 33), does the cursor differ
   from the source pick's?
4. What does the protection tint look like (S-0513 claim 29)? Give its colour or palette change,
   its phases and their lengths, and whether it applies to voxel and sprite objects alike.
5. Does a superweapon icon's charge display (S-0512 claim 16, F3 stages) use the same clock
   overlay as a production icon, and where are the "Ready", "On hold" and drain-mode state labels
   drawn?

## Possible observation
1. Arm each superweapon type from the sidebar, move the cursor over a cell, an enemy building,
   an own building and the radar, and record the cursor frames.
2. Iron-curtain a vehicle, an infantry unit and a building, and record their colours each tick
   from the start of protection to its end.
3. Record a superweapon icon at several charge stages, ready, on hold and active.

## Current workaround
- The client keeps the system pointer while a superweapon is armed. The click behaviour follows
  S-0513 claims 1 and 2.
- Protected objects are drawn without a tint.
- The icon's charge uses the production clock veil (S-0512 F3 stages). "Ready", "On hold" and
  the drain state labels are drawn where the production labels go.
