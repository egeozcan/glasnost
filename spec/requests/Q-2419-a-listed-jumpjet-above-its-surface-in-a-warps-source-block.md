---
id: Q-2419
title: Whether a jumpjet listed in its cell but above its surface is "in flight" for the warp
status: answered
related: [S-0513, S-0102, S-0209, S-0502]
answered-by: [S-0513, S-0102]
---

## Need

S-0513 claim 34 (revision 91) says that the teleport warp affects the listed objects of the
source block (claim 4), and that "an object in the air (an aircraft or jumpjet in flight) is
left alone". It does not say when a unit with jumpjet movement is "in flight".

S-0102 claim 3 (revision 41) says that "the warp takes only objects that are listed in a cell
and are not in the air (S-0513 claims 4 and 34)". It adds that a unit with jumpjet movement is
listed "only where its own rows give the ground layer (S-0209 claim 13)", and that one at its
cruise height is never a member. S-0209 claim 13 lists a jumpjet in a cell when its height *r*
above the surface is below 208 leptons, or its bridge-adjusted height *r*′ is 0, at a listing
moment of S-0502 claim 4. A low own-cell ascent can keep its old ground listing while it
climbs and hovers. S-0102 claim 3 also rates as `measurable` that the hovering jumpjet is left
out of the warp.

For an aircraft, being listed and not being in flight coincide: an aircraft is listed only at no
height above its surface or deck. For a jumpjet they do not. A jumpjet can be listed in a source
cell while it is ascending or descending at 1 to 207 leptons, or while it hovers with a kept
listing. We cannot tell which of these listed jumpjets claim 34 leaves alone.

Our engine (`warp::in_air`) leaves alone any non-aircraft above its surface (or deck). So a
listed jumpjet between 1 and 207 leptons is not taken. Intake 83 noted that r41 seems to name
such a jumpjet as a possible member.

## Questions

1. A unit with jumpjet movement is listed in the ground layer of a source-block cell at the
   launch, at a height above its surface between 1 and 207 leptons, ascending or descending.
   Does it join the warp group, or is it "in flight" and left alone?
2. A hovering jumpjet that kept its ground listing from a low own-cell ascent: is it left alone
   by its flight state (*hovering*), or by its height?
3. Is "in flight" for claim 34 decided by the jumpjet's flight state of S-0502 claim 4 (anything
   but *grounded*), by its height above its surface or deck, or by its render layer (S-0102
   claim 3's ground row)?

## Possible observation

As S-0513 SW12 part b, with a jumpjet type J of the second house in a source-block cell. Order
J to a nearby cell so that its warp launch falls while it is still ascending below 208 leptons.
Run 1: order J to its own cell and let it ascend and hover, keeping its listing. Record each
J's warp state and position from the launch to t_L + 70.

## Current workaround

`warp::in_air` is unchanged: a non-aircraft above its surface or deck is in the air, whatever
its listing and flight state. A grounded jumpjet at no height above its surface joins. Tests
`claims_s0513_warp::intake84_claim34_a_listed_jumpjet_above_its_surface_is_left_alone_q2419_interim`
and `intake84_claim34_a_grounded_jumpjet_listed_in_the_block_joins_the_warp`. No quirk switch.
