---
id: Q-2416
title: A warp member protected or cloaked at its departure, while it waits to land
status: answered
related: [S-0513, S-0514, S-0309, S-0516]
answered-by: [S-0513, S-0301]
---

## Need

S-0513 claim 39a (*The firer*, revision 91) keeps a departed warp member as the target of a
firer whose temporal beam is attached to it, "unless an outranking answer holds": the firer's
check answers *rearming* only when no answer that S-0301 claim 9 ranks above it holds. Two of
those answers read the target itself: a protected target (S-0513 claim 27) and a target hidden
from the firer's house (S-0516 claim 24). No spec says what a member's protection or cloak
does between its departure and its appearance, when it is off the map and in no cell.

Our engine keeps the cloak state unchanged off the map (the cloak machine skips objects that
are not on the map), and protection is a window of game ticks (S-0513 F7), so it runs on and
can end off the map. So a member that was protected or cloaked
at its departure gives its firer *illegal* or *impossible*, and S-0309 claim 29's check drops
it as the firer's target. The code is `combat::check::target_exists` and the cloak update in
`cloak.rs`; the ledger row is in `docs/m8-claims/intake81.md` and `docs/m8-claims/intake84.md`.

## Questions

1. While a warp member that was cloaked at its departure waits to land, does its cloak state
   go on changing (fading, uncloaking, recloaking) as it would on the map, or does it stay as it
   was at the departure until the member appears?
2. Is such a member hidden from an enemy house while it is off the map, so that an enemy
   firer whose temporal beam holds it drops it at S-0309 claim 29's check?
3. Does a member whose protection (S-0513 claim 27) was in effect at the departure stay
   protected off the map, and does its protection timer run there? Does its firer then drop it
   at the check, as for a protected target on the map?

## Possible observation

As S-0513 SW12 part l3, with the tank given a cloak (a cloak generator in range at the launch,
or a type that cloaks by itself) or put under protection before the departure. Record, in each
tick from the departure update to the appearance update, the firer's target, and on appearance
the tank's cloak state and protection.

## Current workaround

The cloak state is kept unchanged off the map and protection runs on by its tick window (our
engine's choices, not stated for a warp member by any spec), so while either holds the firer's
check answers *illegal* or *impossible* and claim 29's check drops the member. Named as a seam;
no quirk switch.
