---
id: Q-3063
title: Does a damaged building with a docking, payment or unloading animation but no damaged name play the normal animation or nothing?
status: answered
related: [S-0402, S-0108]
answered-by: [S-0402]
---

## Need

S-0402 Terms (*Docking, payment and unloading animations*) say that each of the three animations
"is played in its damaged form when the building's health ratio is at or below the yellow health
threshold (S-0003, claim C9), and only if the building's art data gives that animation; a building
without it plays nothing." S-0402's Import mapping row reads `PreProductionAnim`,
`PreProductionAnimDamaged`, `SpecialAnim`, `SpecialAnimDamaged`, `ProductionAnim` and
`ProductionAnimDamaged` as animation type names, default none.

The three animations are the building's *pre-production*, *special-1* and *production* slots of
S-0108. S-0108's Import mapping (**Slot keys**, **Names**) fills a damaged name that is empty at a
read with the normal name held at that point.

For a building whose art gives the normal name and no damaged name, the two readings differ when
the building is damaged:

- by S-0108's slot keys, the damaged name is the normal name, so the normal animation plays;
- if S-0402's "only if the building's art data gives that animation" refers to the damaged form,
  nothing plays.

For the unloading animation this also decides whether a damaged refinery-flag building delays its
harvester (S-0402 claim 11's *finishing* row).

## Questions

1. With the normal name given and the damaged name absent or empty, does a damaged building play
   the normal animation (S-0108's filled name) or nothing?
2. Is the answer the same for all three animations?
3. When nothing plays, does a damaged refinery-flag building still hold a *finishing* harvester
   (it would have no unloading animation playing)?

## Possible observation

A refinery-flag building type whose art gives `ProductionAnim` and `SpecialAnim` but no damaged
names, damaged to below the yellow threshold, with a loaded harvester docking. The recorder logs
the building's running slot animations per tick and the harvester's departure tick, against the
same building at full health.

## Current workaround

The engine reads the three names through S-0108's slot keys, so a damaged building with no damaged
name plays the normal animation, and the unloading animation delays the harvester as at full
health.
