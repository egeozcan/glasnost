---
id: Q-3361
title: How the permanent control ring is created at a permanent takeover
status: open
related: [S-0601, S-0109, S-0612]
answered-by: []
---

## Need

S-0601 r32 claim 18 says the takeover creates the controlled ring with a repeat factor of 1
(S-0109 Terms), so a ring type whose loop count is −1 plays until the ring ends. It then says:
"This claim does not give the creation of the permanent ring of a permanent takeover (claim 13)."
Claim 13 says only that the object "gets the permanent control ring (claim 18)".

The ring shown again (claim 18, *A ring shown again*) is "created and attached as at the start of
that control", so for a permanently controlled object it also depends on how the permanent ring
is created at the permanent takeover.

Our client creates every ring, the permanent one included, as an animation with no initial loop
delay, repeat factor 1 and no reverse request (`Live::follow_rings` in
`crates/app/src/mapview/live.rs`). For the permanent ring that is not given by any claim.

## Questions

1. When psychic domination makes an object permanently controlled (S-0601 claim 13, S-0612 claims
   13 and 14), with what repeat factor is its permanent ring created? Does a permanent ring type
   whose loop count is −1 play until the ring ends, or does it stop after a number of passes?
2. Is it created with an initial loop delay, or with a reverse request?
3. Is anything else different from the controlled ring's creation: its attachment, its point (the
   object's position raised by its type's ring height), its render layer or its depth adjustment?

## Possible observation

As S-0601 scenario MC13 part a's last check, for a permanently controlled vehicle: name as the
permanent-controlled-animation type a test copy of an animation type with visible frames and loop
count −1, dominate a vehicle with a psychic dominator, keep the vehicle still and in view, and
record whether its ring still shows after more than 254 passes of the type have played (S-0109
claim 7). Repeat with a loop count of 2 and count the passes before it ends.

## Current workaround

The client creates the permanent ring as it creates the controlled ring: no initial loop delay,
repeat factor 1 (the S-0109 Terms default) and no reverse request, at the object's position raised
by its type's ring height, in the ground layer. Marked `design: render (Q-3361)`. It changes no
game state and takes no scenario draw.
