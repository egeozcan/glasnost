---
id: Q-0277
title: Which mouse cursor shows for each action under the pointer?
status: answered
related: [S-0214, S-0406, S-0408]
answered-by: [S-0214, S-0213, S-0406, S-0410]
---

## Need

S-0214 decides what a click on the battlefield does (select, move, attack, enter, deploy and so
on), S-0406 gives placement mode and S-0408 the sell and repair modes. The pointer's look tells the
player in advance what the click will do, but no spec says which cursor file and frames show for
each case, how they animate, or where their hot spot is.

## Questions

1. For each click result of S-0214 (and for placement, sell, repair and power modes, edge
   scrolling and the radar), which cursor shows?
2. Which file holds the cursors, which frames does each use, at what frame rate, and where is
   each cursor's hot spot?
3. Is there a "no action" cursor (for example over shroud or an illegal target)?

## Possible observation

Hover the pointer over each kind of target with each kind of selection and record the pointer.

## Current workaround

Interim rule: the system pointer is shown everywhere; the battlefield shows no action cursor.
Recorded in docs/development.md, "Game client (M5b)".
