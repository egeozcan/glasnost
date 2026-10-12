---
id: Q-0035
title: Two details of a driving vehicle's refused step (the attack override and a curve's blocked second cell)
status: answered
related: [S-0207, S-0215]
answered-by: [S-0207]
---

## Need
`glasnost_sim::drive` follows S-0207 claim 15 (tables T4 and T5). Two
results that the claim names are not stated far enough for the code to
follow them without a choice of its own.

## Questions
1. **The override's destination (T4, repeated check, friendly or enemy
   obstacle).** The vehicle "takes as its target, and attacks by an override
   of its behaviour (S-0215 claim 10)" the first listed object or the wall.
   An override replaces both the target and the destination (S-0215 claim
   10). What destination does this override give the vehicle: none, its
   current move destination, or something else? (With its current
   destination kept, the vehicle would make its step checks again on the
   next ticks, and each repeated check would override again.)
2. **A curve's second cell giving an obstacle (T5, friendly or enemy
   obstacle).** The vehicle discards its stored path and "a fresh search
   follows once its path retry timer has run out (on the same tick when it
   already has)". When that search finds a path on the same tick, does a
   step check of the new path's first step follow on that tick (and, if
   so, a first check or a repeated check), or does the vehicle wait until
   the next tick?

## Possible observation
1. S-0207 scenario D17 part d, with the recorder also reading the mover's
   destination on the arrival tick and the ticks after it.
2. S-0207 scenario D17 part g run 3, with a bypass corridor open, reading on
   the arrival tick whether the mover begins a track along the new path.

## Current workaround
1. The override's destination is none (the vehicle stops; the move comes
   back if the override is reverted). Marked "Q-0035" in `drive.rs`.
2. No step check follows that search on the same tick; the vehicle checks
   its new path on the next tick on which it faces the step. Marked "Q-0035"
   in `drive.rs`.
