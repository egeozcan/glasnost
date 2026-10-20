---
id: Q-1560
title: At which moments of a jumpjet's flight is its render layer chosen again
status: answered
related: [S-0102, S-0502]
answered-by: [S-0102]
---

## Need

S-0102 claim 4 says the layer is evaluated at placement, take-off,
landing and attachment, "possibly at further steps of flight", and every
tick while falling. Its *Edge cases* switch
`quirks.layer_change_on_events_only` (default on) keeps the layer between
those moments. The *State and ordering* table gives a unit with jumpjet
movement the ground layer below 208 leptons and the air or top layer
above.

A jumpjet's take-off (S-0502 claim 5, grounded to ascending) happens at
height 0, so it evaluates to ground. If the take-off is the only moment
until the landing, a jumpjet stays in the ground layer for its whole
flight, sorted with the ground objects by its X + Y, and the air and top
rows of the table never apply to it outside a fall. The client
(`crates/app/src/mapview/layers.rs`) does exactly that under the default.

## Questions

1. Is a jumpjet's layer chosen again during its climb, for example when
   it reaches its cruise height (S-0502's hovering state) or when it
   passes 208 leptons? If so, at which steps of its flight exactly
   (climb finished, start of cruising, start of descent, or others)?
2. Does an aircraft's layer change at any step of its flight other than
   leaving and returning to height 0 (S-0501 claim 7)?
3. A jumpjet that was put into the air or top layer and descends to land:
   is it chosen again before it touches down (for example when it starts
   descending), or only at touchdown?

## Possible observation

Two jumpjet infantry hover at cruise height beside a tall building, one
north of it and one south. If the north one is drawn behind the
building's upper part while it hovers, it is still in the ground layer;
if both are drawn over the building, they are in the top layer.

## Current workaround

`quirks.layer_change_on_events_only` on (the spec's default): a jumpjet
is evaluated when placed, when it leaves the grounded state (at r = 0,
so ground), when it enters the grounded state, and every tick while
falling. With the switch off it is evaluated every tick. See
docs/development.md, *Visual review 2026-10-19*.
