---
id: Q-2677
title: Guard with a target while an aircraft slides after its entry report, and where it then comes to rest
status: open
related: [S-0501]
answered-by: [S-0501]
---

## Need

S-0501 r134 claim 24, *Not pad-bound*, and Open question 30: an aircraft that is not pad-bound
touches down, untested, in a cell next to its docking position's, keeps the pad as destination and
its flight point, reports entry at its next processing and slides on (claim 7). Its *guard* without
a target changes nothing on the ground. Three things were not traced: what its *guard* does with a
target while it slides (claim 28 gives an aircraft on the pad its take-off into *attack*), where it
comes to rest once it climbs and claim 33's *move* takes it on, and whether its pad services it
there. The engine meets the case whenever a landing toward a pad whose docking position lies off
the foundation drifts into a neighbouring cell (AC31 part h (1)), and a target comes into range
during the slide.

## Questions

1. While it slides at height 0 in *guard*, in contact, with a target and its ammunition at or above
   claim 28's half-ammunition threshold, does *guard* queue *attack* and take it off at the target
   as for an aircraft on the pad? If so, does the take-off end the slide at once (claim 8)?
2. With a target and less ammunition than that threshold, does *guard* change anything?
3. After the slide's climb, with *move* given by claim 33 and the pad as destination, where does
   its flight end: a new landing toward the docking position, with claim 7's touchdown and forget
   test again, or somewhere else?
4. Where it then comes to rest, in the docking position's cell or a cell next to it, does its pad
   service it there (claim 26), with no second entry report?

## Possible observation

AC31 part h (1) with a touchdown in a cell next to the docking position's; in one run an enemy
vehicle placed within weapon range at the touchdown's tick, in another none. Record each tick to
600 ticks after the touchdown: position, height, behaviour current and queued, destination, target,
ammunition, and the pad's service steps.

## Current workaround

As the code does today. Claim 28's rule reads the height, so on the ground by its height, with a
target and enough ammunition, *guard* queues *attack* and starts it at once; *attack* gives its
destination at the target, which takes it off by claim 8 and ends the slide; with less
ammunition *guard* changes nothing. Once it climbs in *guard* with the pad as destination and
nothing queued, claim 33 gives it *move*, it flies back toward the docking position, and claim 6's
triggers start a new landing there, whose touchdown follows claim 7 again. It rests where a
touchdown's forget test forgets the destination (in the docking position's cell), with no second
report, and since it is then settled out of *enter*, the pad's service steps service it where it
rests. No `design:` citation is needed: each step is one the code already takes by the spec's own
rules; this request asks only whether the original agrees.
