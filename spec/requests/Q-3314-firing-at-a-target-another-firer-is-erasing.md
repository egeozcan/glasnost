---
id: Q-3314
title: Firing at a target that another firer's temporal beam is erasing
status: answered
related: [S-0301, S-0302, S-0309, S-0514]
answered-by: [S-0301]
---

## Need

Issue #458: while a temporal firer erases an enemy, other units keep firing at that enemy. Their
shots do nothing (S-0302 claim 8, stage 1, row "The target is being erased"), so they waste their
fire instead of switching to a target they can hurt.

S-0301 claim 9 leaves the firing check's answer open for such a target. Its paragraph on targets
with a superweapon warp pending ends: "This test is not the test of a target being erased: what that
one answers for a target that a temporal beam holds outside a warp is not settled here." So no
claim gives the firing check *illegal* or *impossible* for a target being erased by another
firer's beam. S-0309 claim 29's check (every 16 ticks) drops a target only on those answers, so it
never drops it, and S-0309 claim 12's table C has no row for an object being erased. S-0514 claim
17 covers only the erased object's own inactivity.

The engine follows this. A guard test on synthetic rules (`claims_s0301_issue_458.rs`) shows an
ordinary firer ordered to attack a target that another firer's temporal beam is erasing: it keeps
the target past two of claim 29's checks, it fires, and its shots do no damage.

## Questions

1. What does the firing check of S-0301 claim 9 answer for a target that another firer's temporal
   beam is erasing, outside a superweapon warp? Where does that answer rank?
2. Does S-0309 claim 29's check, or a search, drop such a target? If so, how quickly: at the next
   16-tick check, at once, or at the next shot?
3. Do new searches skip objects that are being erased, as a row of S-0309 claim 12's table C? Does
   that row apply to every house, or only to some (claim 25)?
4. Is a target in the beam-free leaving stage of erasure (S-0302 claim 8, stage 1, "being erased"
   covers it) treated the same way as one an attached beam holds?
5. Are other temporal firers covered? They join the erasure under S-0514 claims 2 to 5, so an
   answer that drops the target would have to leave them out; does it?

## Possible observation

A target being erased by one temporal firer, with an ordinary armed unit of the same house ordered
to attack it and a second ordinary unit idle within range; record each tick each unit's target and
each shot it fires, from the beam's start to the target's removal. Repeat with the beam broken off
so that the target is in the beam-free leaving stage, and with a second temporal firer ordered to
attack it.

## Current workaround

Today's behaviour is kept: other firers keep the target and fire, and their shots do no damage.
Nothing is built.
