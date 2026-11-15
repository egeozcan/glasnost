---
id: Q-3142
title: Whether warp-pending immobilization also leaves a teleporting infantry unit's landing point unchanged
status: open
related: [S-0207, S-0208, S-0513]
answered-by: []
---

## Need
S-0207 claim 20 (b) gives the landing point of a non-infantry object with the teleporting locomotor
and says: "Two cases leave *L* unchanged: the object has warp-pending immobilization (Terms), or the
search gives no cell." Part (c) hands the landing point of an infantry unit to S-0208 claim 19, which
has no such case: it reserves the chosen standing position when a destination reaches landing choice.

S-0207 Terms defines warp-pending immobilization as the commitment that begins when a superweapon warp
is accepted and ends when the object leaves its old position (S-0513 claims 34 and 39), including the
leaving wait.

So when a teleporting infantry unit that has joined a warp is given a destination before it departs,
no claim says whether it chooses a landing point and reserves that standing position. If it does, the
reservation stands at a cell it will never jump to (S-0513 claim 51: none of the old movement resumes
after the warp) until something releases it. (Issue #363.)

## Questions
1. Does warp-pending immobilization leave an infantry unit's landing point unchanged, as S-0207
   claim 20 (b) says for other objects (no landing choice, no reservation, and the destination
   removed when no earlier *L* is fixed)?
2. If an infantry unit does choose and reserve a standing position while warp-pending, when is that
   reservation released: at the departure (S-0513 claim 39), at the appearance, or never?

## Possible observation
S-0513 scenario SW12 with a teleporting infantry unit in the source block: order it to a far cell a
few ticks after the launch and, before it departs, move another infantry unit to that cell. Whether
the second unit can take every standing position of the cell answers question 1; repeating after the
departure answers question 2.

## Current workaround
The infantry unit chooses and reserves a standing position as S-0208 claim 19 says. The warp's
departure clears a reservation of a landing point not jumped to, and so does the launch for one fixed
before it (`teleport::warp_release`, called from `join` and `lift_off` in
`crates/deterministic/sim/src/superweapons/warp.rs`).
