---
id: Q-2081
title: How a rocked body moves, how its type's weight scales it, and how it returns to level
status: open
related: [S-0303, S-0614, S-0601, S-0210, S-0106, S-0504]
answered-by: []
---

## Need

S-0303 claim 20 says which detonations rock which bodies, with a strength of min(*D* × 0.01,
4.0) and a direction away from the source object or from *P*, and that rocking sets the
body-rocking rates "scaled by the type's weight". S-0614 claims 9 and 10 rock a drone's host and a
grappling parasite's host. S-0601 claim 16 and F4 set a mind-control overload's sideways rate
directly, 0.015 or 0.03 radians per tick (built: the view sets that rate as given). S-0210 claim
12 limits the forward angle (π/10 while crush-slowed, π/4 otherwise), and S-0106 claim 9 and
S-0504 F5 draw the pitch and roll. No approved spec says how a
strength becomes the two rates, how the rates move the angles over time, or how a rocked body comes
back to level (S-0504 Open questions say so). The view of our engine now draws rocking
(`crates/app/src/mapview/rocking.rs`) with interim rules of our own, listed under *Current
workaround*. Only drawing reads rocking in our engine, so the answers change pictures, not game
state (but see question 8).

## Questions

1. **Strength to rates.** What rates does a push of strength *s* set on the pitch and the roll? Is
   the strength split between them by the push's direction relative to the body's facing (and if
   so, by what rule), and in what unit is the rate (S-0601 F4 gives its rate in radians per tick;
   is a strength's rate in the same unit, or in S-0210's tilt units, and how does a tilt unit relate
   to radians)? Does a new push replace the rates the body had, or add to them (S-0601 claim 16's
   rate "takes" its value)?
2. **Return to level.** How do the angles change on each tick after a push: by how much does the
   rate change per tick, does the body swing through level and back, and when does it stop? Does
   a body at a limit stay there until something else moves it, or does it come back at once?
   Is it the same for the driving, ship, hovering and jumpjet locomotors?
3. **Weight.** "Scaled by the type's weight": is the strength multiplied or divided by the weight,
   or used another way? Which weight do kinds without a weight entry use (aircraft, and any
   voxel-drawn object that is not a vehicle)? What happens with a weight of 0 or below? Does the
   drone's strength of 1.5 (S-0614 claim 9) and the squid's push (claim 10) use the weight too?
4. **Sideways limit.** S-0210 claim 12 says the sideways angle "keeps its usual limit". What is it?
5. **Centre cell without a source.** S-0303 claim 20 pushes the centre cell's objects away from the
   source object "when there is one". Which way is a centre-cell object pushed when the detonation
   has no source object (or the source has left play), and which way is an object pushed that
   stands exactly on the point it is pushed from?
6. **The drone's side.** S-0614 F2 puts the rocking point *s* = +2 leptons along the perpendicular
   to the host's facing for a draw of 0. Which perpendicular is +: the host's right or its left?
7. **The squid's push.** S-0614 claim 10 gives the push's tilt angle sin(*k* × (±π/10)) × π/4 at
   frame *k*. Is it the pitch or the roll (S-0504 claim 15 implies the sideways angle), which way
   does a draw of 1 rock, and does it add to, replace or ignore a rocking from a detonation in the
   same ticks?
8. **What reads it.** S-0303 claim 20 is of low confidence that rocking never reaches game state,
   and S-0504 claim 16 lets the roll of a sinking open-topped transport move its passengers' fire
   point. Does a rocked body's pitch or roll move any fire point or other game rule outside
   sinking?

## Possible observation

S-0106 scenario T2's rocking part and T11's rocking part, and S-0504 scenario N16 parts b, c and h,
captured one frame per tick from the hit until the body is level again: a stock vehicle of known
weight hit from ahead, abeam and at 45 degrees by a detonation of a rocking warhead at a few damage
amounts (just above 30, mid-range, at and above 400), once crush-slowed. The same with a heavier and
a lighter type. A drone delivery and a squid's push on a vessel, recorded the same way.

## Current workaround

The view's interim rules, marked `design: Q-2081` in `crates/app/src/mapview/rocking.rs` and listed
in `docs/m8-claims/a4-rocking.md`:

- A push sets both rates, replacing what they were: rate = 0.05 radians per tick × *s* ÷ weight,
  times cos *φ* for the pitch and sin *φ* for the roll, where *φ* is the push's facing relative to
  the body's (positive clockwise, so a push to the right lowers the right side and a push ahead
  lowers the bow).
- Each tick each angle adds its rate, stops at its limit with its rate level (S-0210 claim 12),
  and then the rate is pulled towards level by a damped spring: rate −= ω² × angle + 2ζω × rate,
  with ω = 2π / 18 per tick and ζ = 0.2. The cap's push swings an unweighted body to about 0.45
  radians and settles below 0.005 radians in about 60 ticks.
- Only vehicles rock in the view; aircraft (drawn by their flight attitude), infantry and buildings
  are left out. The weight is the vehicle type's imported weight (S-0504 claim 11: 1.0 when
  absent), and 1.0 when it is not a positive number. The drone's 1.5 is scaled the same way.
- The sideways limit is π/4.
- A centre-cell object with no source object is pushed away from *P*; an object exactly on its
  push point does not rock.
- The drone's + side is the host's right; the host is pushed away from the point.
- The squid's push is the roll, a draw of 1 rocks towards +π/10 per frame, and it adds to any
  detonation's roll, held to the sideways limit.
- An overload pulse sets the roll rate to S-0601 F4's value, unscaled, and leaves the pitch rate;
  it then moves and returns to level as above.
- Nothing in the game reads rocking. A sinking transport's passengers fire from S-0313 claim 5's
  fire point, which does not take the body transform in our engine (a seam).
