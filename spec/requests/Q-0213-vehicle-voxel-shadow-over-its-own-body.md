---
id: Q-0213
title: Does a ground vehicle's voxel shadow darken its own body where they overlap?
status: answered
related: [S-0102, S-0105]
answered-by: [S-0105, S-0102]
---

## Need
S-0102 claim 18 says that the voxel shadow is drawn in the object's own drawing and that an
aircraft's body is shown undarkened over its own shadow. For a vehicle it leaves open whether the
shadow paints over its own body (Open questions). S-0105 revision 8 removed its own ordering
statement ("Ordering consequences": the shadow's priority relative to that object's own body is
not fixed there). The map viewer must put the shadow either before or after the body in the
vehicle's drawing (`app::mapview::build::vehicle_voxels`).

## Questions
1. Where a ground vehicle's voxel shadow image and its own body image overlap, and both pass
   their depth tests, are those body pixels shown darkened (shadow after the body) or undarkened
   (shadow before the body)?
2. Is the answer the same for a hovering unit and for a vehicle on a slope?

## Possible observation
S-0102 scenario D23 with a ground vehicle on flat ground whose shadow section is offset so that
it overlaps the lower rows of its body at one facing. Compare those body pixels with the same
facing on a light-coloured cell without the shadow (the type's no-shadow option).

## Current workaround
The shadow is put before the body, so the body covers it, as S-0102 claim 18 states for an
aircraft. This is an interim rule in `docs/development.md`.
