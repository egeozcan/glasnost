---
id: Q-0452
title: In which order does a band box add its eligible candidates to the selection?
status: open
related: [S-0214, S-0701]
answered-by: []
---

## Need

S-0214 claims 19, 20, 24 and 25 say which objects an ordinary band box can
take and which screen candidates survive overflow (buildings first, then
mobile objects in drawing order). Claim 20 says additions use claim 12's
selection order, and S-0701 claim 17 gives the selection response to the
first successful addition. Claim 12's order depends on the order of
addition (positive-primary-damage types go to the front, so they end in
reverse order of addition; the others keep their order of addition). No
claim says in which order the box visits the eligible candidates when it
adds them. `play::Play::select_box` needs that order for the resulting
selection order and for which object speaks.

## Questions

1. When a released box holds several eligible objects, in which order are
   they added: the admitted candidate order of claim 25 (buildings, then
   mobile objects in drawing order), its reverse, creation order, screen
   position, or another order?
2. Does a type-select drag (claim 19's last sentence) add in the same order?

## Possible observation

S-0214 Scenario T15 part B or T14 with a few own units of a positive-damage
type and of a non-damaging type at distinct ground sort keys and in
different render layers: read the selection order after the box (the
representative tie-break of claim 1, or the order-response object of
S-0701 claim 18) and which unit gives the selection response.

## Current workaround

The box adds eligible candidates in the admitted candidate order of claim
25: buildings in creation order, then mobile objects in their drawing order.
