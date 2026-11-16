---
id: Q-3182
title: Whether a vehicle that enters a vehicle bunker leaves play for the magnetic lift's grip, hits and drops
status: open
related: [S-0607, S-0606, S-0203]
answered-by: []
---

## Need
S-0607 claim 21 (*Ending*) says that leaving play "includes going inside a transport, a bunker or a
building", so a wave whose firer or target enters a bunker ends at its next update. S-0606 claim 4
(*Map presence*) says a bunkered vehicle "stays on the map, in play, and is not a passenger". Our
engine follows claim 21 for the wave only (batch S22). Three other S-0607 rules turn on leaving play
and are not given for a bunker entry: claim 12's two leaving-play rows (the lifter leaves play; the
gripped unit leaves play), claim 2's voiding of a hit whose firer has left play, and claim 14 (a
dropper that leaves play before its dropped unit lands). The engine needs to know whether a bunker
entry is one of these events (issue #285; S22 ledger seam). The gap is
`structures::bunker` (the entry has no leaving-play hook) and `lift.rs` (the grip end and the dropper
rules).

## Questions
1. A lifter that grips a unit enters a vehicle bunker (its grip began after it took the bunker as its
   destination). Does the grip end at the entry, with the lifter's target cleared, as claim 12's
   *lifter leaves play* row gives, or does the grip last while the lifter sits in the bunker?
2. A gripped unit cannot enter a bunker while it flies; but a unit dropped by a lifter that then
   enters a bunker before the unit lands: does the dropped unit lose its dropper, as claim 14 gives
   for a dropper that leaves play?
3. A lifter's seizing projectile is in flight when the lifter enters a bunker. When it detonates, is
   the hit voided as claim 2 gives for a firer that has left play?
4. Is a bunkered vehicle's exit from the bunker an entry into play for any of these rules (for
   example the wave's update order of claim 21)?

## Possible observation
A magnetic-lift vehicle (one that can enter a vehicle bunker) seizes a tank and is then ordered into
an empty friendly vehicle bunker next to it before the tank arrives. Record whether the tank keeps
flying toward the lifter, falls, or lands, and whether the lifter's target is cleared. Repeat with a
seizing projectile in flight at the entry, and with a dropped unit still falling.

## Current workaround
The wave ends at a bunker entry (S-0607 claim 21, S22). Nothing else treats a bunker entry as leaving
play: a grip lasts while the lifter is bunkered, a dropped unit keeps its dropper and a seizing hit
whose firer is bunkered is not voided. This is the S22 behaviour, unchanged.
