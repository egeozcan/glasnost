---
id: Q-0976
title: What a jumpjet does with an ordered point whose nearby-cell search finds nothing
status: open
related: [S-0502]
answered-by: []
---

## Need

S-0502 claim 16 keeps the ordered point as the stored destination when the nearby-cell search finds
no cell, and leaves the moving state as it was. Its Open questions say how a later update uses that
point is not traced. The code has to say what the locomotor does with a point that is not a cell
centre and whose cell may be unusable.

## Questions

1. A jumpjet that was already moving (cruising, hovering with an order) when it got such a point:
   does it fly to the kept point and arrive there as it would at a found cell, and what does it do
   if the point is off the map?
2. A jumpjet that was not moving (grounded, or hovering without an order): does it take off, or
   stay as it is until another order?

## Possible observation

Order a grounded jumpjet and a cruising one onto a cell for which the flying-zone search finds no
cell (the map edge with every cell near it unusable).

## Current workaround

`jump::take_destination` stores the ordered point as `JumpState::dest` and changes nothing else:
a moving jumpjet flies to it by the ordinary rules, one that is not moving stays as it is.
