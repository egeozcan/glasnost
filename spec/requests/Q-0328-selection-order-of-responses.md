---
id: Q-0328
title: S-0701 claim 18's selection order against the selection order of S-0213 and S-0214
status: answered
related: [S-0701, S-0213, S-0214]
answered-by: [S-0701]
---

## Need

S-0701 claim 18 defines selection order. When an object is selected, it goes to the front of the
selection if its type's primary weapon does positive damage, and to the end otherwise. The spec's
"Changes for other specs" asks S-0213 and S-0214 to use that order wherever they order a
selection.

S-0214 (revision 18) still takes the selection in the order the objects were selected. Our client
issues a click's orders in selection order, and that order reaches the replay and game state. So
reordering the selection would change S-0213 and S-0214 behaviour; it is not only a sound change.

## Questions

1. Is S-0701's selection order also the selection order of S-0213 and S-0214, so that a click's
   orders are issued in it?
2. When the responder (the first object in that order) gets no order from the click, does any
   object respond? S-0214 claim 14 says none does.

## Current workaround

- The client keeps the selection in the order the objects were selected, and issues orders in that
  order.
- Only the sound side applies claim 18. The responder is the most recently selected object of a
  positive-primary-damage type, or the earliest selected object if there is none. It responds when
  the click gave it an order.

Documented in docs/development.md, "Audio".
