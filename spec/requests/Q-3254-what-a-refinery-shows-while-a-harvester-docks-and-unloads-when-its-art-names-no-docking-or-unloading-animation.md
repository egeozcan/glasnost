---
id: Q-3254
title: What a refinery shows while a harvester docks and unloads when its art names no docking or unloading animation
status: answered
related: [S-0402, S-0108]
answered-by: [S-0402]
---

## Need

S-0402 (Terms, "Docking, payment and unloading animations", and claim 11's unloading table) gives a refinery three animations while a harvester unloads at it:

- the *docking animation*, its pre-production slot (`PreProductionAnim`), started with the start row;
- the *payment animation*, its special-1 slot (`SpecialAnim`), started at each payment unless it is already playing;
- the *unloading animation*, its production slot (`ProductionAnim`), started on the empty-load row; the *finishing* harvester waits while a refinery-flag building plays it.

S-0402's Import mapping and S-0108's Import mapping read these from the building's art section. A variant that names nothing starts nothing (S-0108 Terms).

An issue report says that in a skirmish on an imported install, a refinery shows only its normal body while a harvester docks and empties its load, with no docking or unloading animation. Our trace on synthetic data finds that the simulation starts and stops the three slots as claim 11 says. A local check of an install's data (kept out of the repository) finds a case that the specs do not seem to cover:

- **If** a refinery-flag building's art names no pre-production and no production animation, and of the three keys names only the special-1 animation, then under S-0402 and S-0108 as written:
  - nothing plays when unloading starts;
  - the payment animation plays from each payment until the next payment row, or until the empty-load row stops it (S-0402 claim 11), for about 900 × *D* ticks;
  - a harvester that docks with an empty load shows nothing, since the empty-load row starts and removes the payment animation in the same processing;
  - the *finishing* harvester has no unloading animation to wait for.
- **If** that art also names the active-1 to active-4 animations, they are the refinery's fill-level display (S-0108 claim 15) and do not react to a docked harvester.

The lead expects visible docking and unloading feedback on the refinery. It is not clear whether the original has such feedback for this data, or where it would come from.

## Questions

1. For a refinery-flag building whose art names neither a pre-production nor a production animation, does the original show any animation on the refinery when a harvester starts unloading, or while it unloads? If so, where does it take that animation's name from: another art key, another art section (for example a section for a part of the building), or something the harvester's or the dock's type names?
2. If it uses another source, does that animation play in the slot and at the rows S-0402 claim 11 gives (start row, payment rows, empty-load row)? And does the *finishing* harvester wait for it, as for the unloading animation?
3. Is the payment animation drawn for its whole run from a payment to the empty-load row? Does a harvester that docks with an empty load show nothing on the refinery, as claim 11 says?
4. Does a refinery's fill level (S-0108 claim 15) change while a harvester unloads at it? Is that the only change a player sees on this data?

## Possible observation

A skirmish on an imported install with a refinery whose art is as in the first "if" above. Record the refinery's drawn animations, tick by tick, from the harvester's contact to the break of contact: once with a full load of one tiberium, once with a load of two tiberiums and once with an empty load. Also record which art keys and sections the drawn animations came from.

## Current workaround

None needed in the simulation: it follows S-0402 claim 11 and S-0108 as written. With such data only the payment animation plays, from each payment to the next payment row or the empty-load row. We change nothing until this is answered.
