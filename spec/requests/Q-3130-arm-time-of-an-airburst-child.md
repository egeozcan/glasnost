---
id: Q-3130
title: Arm time of an airburst child
status: answered
related: [S-0306]
answered-by: [S-0306]
---

## Need
S-0306 claim 9 fixes a projectile's arm time at placement: "the projectile type's arm field in
ticks, or 0 when the target is an aircraft". Claim 21 says the nine children of an airburst are
projectiles of their own, placed as claim 8 says for the projectiles a burst releases, each aimed at
a cell. It does not say whether a child takes its own projectile type's arm field as its arm time.

The arm time decides when T1's arrival and passed-by rows first apply to a ranged or steering child,
and until the first check past it the recorded value Ω keeps its placement value. A child type with
a positive arm field would therefore fall further before either row could detonate it. The engine
gives every child an arm time of 0 today, which claim 21 does not state (#315).

## Questions
1. Does an airburst child take its projectile type's arm field as its arm time (claim 9), or is its
   arm time always 0?
2. If it takes the arm field, is the arm time 0 when the parent's target is an aircraft, as claim 9
   says for a target that is an aircraft, given that every child is aimed at a cell?

## Possible observation
An airburst weapon whose projectile type is ranged, with an arm field of a few dozen ticks, burst
some hundred leptons above flat ground. Record the tick and height at which each child detonates,
against the same setup with an arm field of 0.

## Current workaround
Every airburst child has an arm time of 0. Its Ω is set at placement to the full *d*₃ from its
position at placement to its stored target point (T1), as for every other placed projectile.
