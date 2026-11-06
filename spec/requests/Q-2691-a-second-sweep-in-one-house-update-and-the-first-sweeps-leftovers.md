---
id: Q-2691
title: Whether a defeat sweep reaches the leftovers that an earlier sweep of the same tick made
status: open
related: [S-0203, S-0414, S-0312]
answered-by: []
---

## Need

S-0203 r50 claim 13 (a) makes every object removed in a tick, by any cause and in any part of the
tick, a leftover to the end of that tick, and claim 13 (c) has the defeat sweep of S-0414 claim 5
offer its hit to the house's leftovers at their places in creation order. Our engine offers it to
leftover vehicles with no health, which then repeat their S-0312 death effects (debris, the death
weapon of an exploding vehicle, the explosion and destruction animations, a crate they carry).

Two cases in which a sweep meets leftovers that a sweep itself made follow from these rules read
literally, but no scenario or claim names them, and both have visible effects:

- A surrendering house (S-0414 claim 11): in its house update its objects are destroyed as in
  claim 5, and then, with nothing left, it fails its defeat test in the same house update (claim 1)
  and claim 5's sweep runs again. Every vehicle the first sweep destroyed and removed with no
  health is then a leftover of that tick, so the second sweep would repeat each one's death
  effects: a second crate from each crate carrier, a second death weapon from each exploding
  vehicle, a second set of debris and animations.
- Within one sweep: the death weapon of a vehicle that the sweep destroys at its place kills a
  vehicle of the same house whose place comes later. That one is removed with no health before
  the sweep reaches its place, so the sweep finds it as a leftover and repeats its death effects
  there.

## Questions

1. When a surrendering house's objects are destroyed and the house is then defeated in the same
   house update, do its vehicles destroyed by the first destruction show their death effects a
   second time (for one, does a crate-carrying vehicle leave two crates)?
2. Within one sweep, does a vehicle of the house that a death weapon of the sweep destroys before
   its own place show its death effects a second time when the sweep comes to that place?

## Possible observation

1. A house of two vehicles, one a crate carrier and one with a death weapon whose blast does not
   reach the other, in a skirmish with crates on; the house surrenders. Record, in the tick of its
   destruction, every crate placed, every detonation and every animation started at each vehicle's
   position.
2. A house of two vehicles side by side, the first created exploding with a death weapon that
   destroys the second, the second a crate carrier; the house is defeated through the loss of a
   third object. Record the crates placed and the detonations in the defeat tick.

## Current workaround

The literal rule, cited `spec: S-0203 claim 13 (a) and (c)`. Our sweep walks the house's objects
and the tick's leftovers together in creation order, so a sweep finds any leftover at its place,
whether an earlier sweep of the same house update or a death weapon of the same sweep removed it.
In case 1 each vehicle the surrender destroyed repeats its death effects in the defeat sweep; in
case 2 the later vehicle repeats its death effects at its place.
