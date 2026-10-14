---
id: Q-0196
title: What a type suppressed by a full sidebar tab is for the taken-up set, order legality and removal orders
status: open
related: [S-0004, S-0404, S-0407, S-0411]
answered-by: []
---

## Need

S-0004 claim 11 caps a sidebar tab at 76 items: "An otherwise eligible item that becomes available
while its tab is full is not shown." In our engine every player house keeps its icons per tab
(`HouseTech::icons`), and that icon set is S-0407's taken-up set, which gives order legality
(S-0411 claim 6, S-0404 claim 10) and, when a recheck removes a type, S-0411 claim 2's cancel and
cancel-all orders (S-0404 claim 9). S-0004 claim 11, S-0407 claim 1 and Open question 4, S-0404
claim 9 and S-0411 claims 2 and 13 and open question 9 all leave the consequences of suppression
open and infer no rule. `econ::tech::recheck` cannot apply the cap without choosing them, so it does
not apply the cap at all.

## Questions

1. Does a type suppressed by a full tab belong to the house's taken-up set, so that a build order
   for it (from an AI house, a script or a stale click) is legal?
2. When an item leaves a full tab, does a suppressed type gain the freed slot, and if so at which
   moment (the next recheck, the same sidebar update) and in which order when several wait?
3. When a suppressed type loses availability, are cancel and cancel-all orders given for it, and
   where do they fall in claim 13's tab and icon order, which a type without an icon has no place in?
4. Does the cap count per player house in our engine (every player house keeps its own tabs), as
   claim 13's every-player extension does for the icon order?

## Possible observation

S-0004 F4 shows suppression on screen. A follow-up on the same map: with the 80 buildings
available and the tab full, give a build order for a suppressed type by a script action or a
trigger, then sell or lose one shown building whose loss frees a slot, and record whether the
suppressed type appears, when, and whether its production can be started or is cancelled when its
prerequisite is destroyed.

## Current workaround

`econ::tech::recheck` does not apply the 76-item cap: every otherwise eligible type is shown and in
the taken-up set, however many there are. Marked as interim in docs/development.md.
