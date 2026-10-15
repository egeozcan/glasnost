---
id: Q-0635
title: Strafing wait with a zero speed field, enter processing delay, an attack run's scatter request, and a release-started crash
status: answered
related: [S-0501, S-0215, S-0506]
answered-by: [S-0501]
---

## Need

Several small gaps in `glasnost_sim::air`: Formula F6 divides by the type's speed field Vₜ,
which may be 0; claim 24 gives no processing delay for *enter*; claim 16 sends a scatter request
without saying its threat point or whether it is forced; claim 9 (Open question 9) does not say
whether a crash started by a carrier's release (S-0506 claim 12) keeps ordinary movement.

## Questions

1. What is the strafing wait of F6 when the type's speed field is 0 or below 0? Is Vₜ the raw
   field or the converted speed?
2. Which delay does the processing of *enter* return while an aircraft flies to its pad?
3. Which threat point, forced flag and move-anywhere flag does an attack run's scatter request
   carry?
4. Does a living aircraft whose crash was started by a carrier's release keep ordinary forward
   motion and altitude adjustment while it falls?

## Possible observation

S-0501 scenarios AC7 (a strafer type with speed 0 on a named test map), AC9, AC14 and AC21.

## Current workaround

F6 uses the raw speed field, at least 1; *enter* returns the standard delay; the scatter request
is unforced and not move-anywhere, with the aircraft's position as its threat point; a
release-started crash has no ordinary movement (marked Q-0635 in `glasnost_sim::air` and in
docs/development.md, M6a interim rules).
