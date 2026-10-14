---
id: Q-0278
title: How does dragging a box on the battlefield select objects?
status: answered
related: [S-0214, S-0101]
answered-by: [S-0214]
---

## Need

S-0214 covers selecting by clicking one object, with and without the force-select modifier. A
player also selects many units by dragging a box with the left button. No spec says when a drag
starts, which objects the box takes, or how modifiers change it.

## Questions

1. How far must the pointer move with the button held before a drag replaces a click?
2. Which objects does a released box select: only the player's own, only mobile ones, only those
   whose reference point is inside, or any whose drawing meets the box? Are buildings, harvesters
   or aircraft treated differently?
3. Does a box replace the selection, add to it with the force-select modifier, or toggle?
4. Is there an upper limit on the number of objects selected?
5. Are objects under shroud taken?

## Possible observation

Drag boxes of several sizes over mixed groups of own and enemy units and buildings, with and
without each modifier, and record the selection.

## Current workaround

Interim rule: a left-button drag of more than 4 pixels in either direction is a box. On release
it selects every object of the local house that `glasnost_sim::targeting::selectable` accepts,
that is mobile (infantry, vehicle or aircraft) and whose reference point projects inside the box.
Without the force-select modifier the box replaces the selection; with it the box adds to it. An
empty box with no modifier clears the selection. Recorded in docs/development.md, "Game client
(M5b)".
