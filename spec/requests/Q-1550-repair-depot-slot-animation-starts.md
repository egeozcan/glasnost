---
id: Q-1550
title: When a repair depot starts its special-1 (and production) slot animations
status: open
related: [S-0108, S-0314, S-0504]
answered-by: []
---

## Need
S-0108 claim 20 gives a repair depot's reactions once its slot animations are running: when
*special-1* ends, *special-2* starts while the depot is repairing a docked unit and *idle* starts
otherwise; out of its repair behaviour with *production* or *special-2* running, *special-3*
replaces them; *idle* follows *special-3*. Scenario B18 expects "while repairing, *special-2*
follows *special-1*". But no spec says what starts *special-1* (or *production*) at a depot in the
first place: S-0108's Open questions hand that to the repair spec, and S-0314 (claims 6 and 16 to
24) and S-0504 say nothing about slot animations beyond "a depot left without any contact plays
its idle animations". The live client (`crates/app/src/mapview/slot_play.rs`, `live.rs`) runs
claim 20's rows but never starts *special-1*, so a depot repairing a vehicle shows only its *idle*
slot animation. The lead's review of 2026-10-19 (m6b `vehicle-repaired-at-depot.png`) asked where
the repair arm is.

## Questions
1. Which event starts a land repair depot's *special-1* slot animation: a vehicle docking (the
   contact being made), the settling step, the start of the working phase of S-0314 claim 6, each
   repair step, or something else? Is it started again on every such event (replacing a running
   one) or only when absent?
2. Is the start in the normal or damaged variant only, or may it take the occupied variant?
3. Does anything start the depot's *production* slot, given that claim 20's first row stops it?
4. Does a naval depot (shipyard repairing ships, S-0504) start the same slots?
5. When the serviced vehicle leaves or dies mid-repair, does anything besides claim 20's rows stop
   *special-1* or *special-2*?

## Possible observation
Scenario B18's setup: a repair depot whose art names distinct test animations for *idle*,
*production* and *special-1* to *special-3*; drive a damaged vehicle onto it, record per tick
which slot animations are present from the order until well after the vehicle has left, and
repeat with a vehicle that is destroyed while being repaired.

## Current workaround
Interim rule (docs/development.md, *Visual review 2026-10-19*): nothing starts a depot's
*special-1* or *production* slot; the depot shows the slots of its grand opening (S-0108
claim 11), and claim 20's rows apply only if such a slot were running.
