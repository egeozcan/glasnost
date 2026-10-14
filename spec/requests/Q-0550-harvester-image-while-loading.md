---
id: Q-0550
title: What does a vehicle harvester show while it loads resources on a field?
status: open
related: [S-0402, S-0109, S-0111]
answered-by: []
---

## Need

S-0402 gives the harvest cycle (field search, loading per processing, docking, payment and the
refinery's unloading animations), and S-0111 claim 14 the load pips. No approved spec says what
a vehicle harvester itself shows while it stands on a resource cell loading: whether an
animation plays at or over the harvester, whether its own image changes (a different voxel
section, frame or turret motion), and when such a display starts and stops relative to the
loading steps. The live renderer (`crates/app/src/mapview/live.rs`, `build.rs`) draws a loading
harvester exactly as an idle one. The project lead's review of a native skirmish reports that a
loading ore miner shows no mining animation.

## Questions

1. While a vehicle harvester is in its *harvest* behaviour and loading on a resource cell,
   is anything drawn at or over it that is not drawn while it drives or stands elsewhere?
   If an animation, which data names it (rules or art key, per type or game-wide), where is
   it placed relative to the harvester, and does it loop for as long as loading lasts?
2. Does the harvester's own body change while it loads (voxel section, turret or barrel
   motion, frame), and if so, by which data and on which ticks?
3. Does it differ between resource kinds (ore, gems) or between harvester types, including a
   harvester that also deploys or teleports?
4. When exactly does it start and stop: on the first loading step, on arrival in the cell,
   while the load is full, while waiting for a field?

## Possible observation

A stock harvester ordered to a field, recorded frame by frame from arrival to full load, with
the logged loading steps of S-0402 alongside.

## Current workaround

None drawn. A loading harvester looks like a standing one; the load pips of S-0111 claim 14
show the load.
