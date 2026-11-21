---
id: Q-3362
title: Does a replaced animation depth adjustment hold through a follow-up switch
status: answered
related: [S-0601, S-0102, S-0109, S-0301]
answered-by: [S-0109]
---

## Need

S-0102 claim 13 says the animation depth adjustment is "the animation's own value: its type's,
unless the creator of the animation replaced it". S-0601 r32 claim 18 has the takeover of a
building replace its ring's adjustment by −1024 depth units, "whatever the ring type's own depth
adjustment is". S-0301 claim 22 and F8 have a building's shot animation replaced in the same way.

S-0109 claim 15 lets an animation switch to its follow-up type at the end of its last pass, and
claim 22 keeps such an animation attached to the same owner. No claim says which depth adjustment
the animation has after the switch when its creator had replaced it: the creator's value, or the
follow-up type's own.

Our client keeps the creator's value on the playing animation (`LiveAnim::z_adjust` in
`crates/app/src/mapview/live.rs`), so it holds through a switch, for a building's ring as for a
building's shot animation.

## Questions

1. A building is taken over and its control ring type has a follow-up type (and a loop count that
   lets it end). After the switch, is the ring drawn with the −1024 adjustment, or with the
   follow-up type's own depth adjustment?
2. The same for a building's shot animation whose type has a follow-up type: does its replaced
   adjustment (−200 with an occupant, or S-0301 F8's value) hold after the switch?

## Possible observation

As S-0601 scenario MC13 part c (c1), with the test ring type given a loop count of 1 and, as its
follow-up type, a named test copy with visible frames, loop count −1 and a depth adjustment of 0.
At n = −1524 the body shows at the compared pixel before the switch if and only if the −1024
holds; compare which drawing shows there after the switch, at n = −524 and n = −1524.

## Current workaround

The creator's replacement holds through a follow-up switch, as our client already does for a
building's shot animation. Marked `design: render (Q-3362)` where the ring's −1024 is set. It
changes no game state.
