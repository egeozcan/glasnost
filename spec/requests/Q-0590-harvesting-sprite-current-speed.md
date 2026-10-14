---
id: Q-0590
title: Which speed hides the harvesting sprite of a moving harvester?
status: answered
related: [S-0402, S-0207]
answered-by: [S-0402]
---

## Need

S-0402 claim 5a (r21) suppresses the vehicle harvesting sprite for a ground-driving harvester
with "either a turning body or an active track with positive current speed". S-0207 has two
quantities that could be meant: the speed fraction ρ (Terms), a double from 0 to 1, and the
progress a moving tick earns, *s* = trunc(*s*₂ × ρ) (claim 2), which is 0 while ρ is small
but positive, for instance in the first ticks of acceleration. Our engine needs one test in
`glasnost_sim::econ::harvest::display_shown`.

This was question 5 of Q-0580, added after the request had been taken up; S-0402 r21 and
S-0102 r16 answered questions 1 to 4 and closed Q-0580 without this one.

## Questions

1. Is "positive current speed" a speed fraction ρ above 0, or a positive progress *s* in that
   tick?
2. On a tick where the harvester is on an active track with ρ > 0 but *s* = 0, is the sprite
   shown or hidden?

## Possible observation

A loading harvester ordered to a cell a few cells away, captured every tick from the order:
the tick the sprite disappears, compared with the tick its position first changes. With an
accelerating vehicle type, any ticks of ρ > 0 and *s* = 0 lie between them.

## Current workaround

ρ > 0 on an active track hides the sprite (`display_shown`); marked open in a code comment.
