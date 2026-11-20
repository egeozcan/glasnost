---
id: Q-3337
title: Where the control ring of a building stands, and what drawing it in front of the building means
status: open
related: [S-0601, S-0109, S-0102]
answered-by: []
---

## Need

S-0601 claim 18 says a controlled or permanently controlled building carries its control ring
"using a building-height-based placement" and that the ring "draws in front of the building". It
says the exact vertical conversion is not established (Open questions; scenario MC13 is to
measure it). For a mobile object the claim gives the point: the object's position raised by its
type's ring height in leptons.

What our client does for a building's ring now (render batch R19, `follow_rings` and the redraw
in `crates/app/src/mapview/live.rs`, marked `design: render (Q-3337)`):

- The simulation gives the ring's point as the building's reference point (S-0204 claim 4, the
  centre of its foundation) at the building's own height, with nothing added for the building's
  height (`glasnost_sim::mind::ring`).
- The client plays the ring type there as an attached animation in the ground render layer
  (S-0109 claim 22), ordered among the ground layer's entrants by that point's X + Y plus the
  type's sort adjustment (S-0102 claim 5).
- Its pixels neither read nor write depth, so the building's body, a depth writer (S-0102
  claim 10), cannot hide any of them. Without that, the body hides most of a ring that stands at
  the foundation's centre at ground height.
- Parts of the building that come later in the ground layer's order still draw over it. Those
  are object-pass parts and slot animations whose sort key is larger than the ring's.

## Questions

1. How high above the building does its ring stand, in leptons or in screen pixels, and from
   what does it follow? For example: the building type's height in cells, the art's height, the
   foundation's size, or a fixed value. Does the mobile objects' ring-height field
   (`MindControlRingOffset`) take part for a building?
2. At which ground point (x, y) does it stand? The centre of the foundation, its top corner, the
   building's position (the top-left foundation cell), or another point?
3. What does "in front of the building" cover? Is it only that the ring is drawn after the
   building's body and is never hidden by the body's pixels? Or is it also drawn after the
   building's other parts: its turret, its slot animations above and below it, and its bib?
4. Can other objects hide a building's ring: a unit standing in front of the building, an
   animation, or another building in front of it?

## Possible observation

S-0601 scenario MC13, with the building-sprite and ring pixels recorded in a fixed-resolution
capture, across building types of different heights and foundations and with the ring-height
field changed. For question 3, use a building type with a turret or an active slot animation
that overlaps the ring's pixels, and record which one shows where they overlap. For question 4,
park a unit in front of the building so that it overlaps the ring.

## Current workaround

The client draws a building's ring at the simulation's point, the foundation's centre at the
building's height. It is drawn as a ground-layer animation that ignores depth, so it shows over
the body, and later ground-layer entrants still draw over it. The placement is marked
`design: render (Q-3337)` and changes no game state. A mobile object's ring follows S-0601
claim 18 as written.
