---
id: Q-3164
title: Which conditions of the engineer-friendly building apply to which of its two branches
status: answered
related: [S-0214, S-0509, S-0514]
answered-by: [S-0214]
---

## Need
S-0214 table I defines: "An *engineer-friendly building* is a repairable building, not
vehicle-like, that is allied with the actor, or that belongs to a passive neutral house, can be
garrisoned and is not being erased." The sentence has two branches joined by "or", and three
conditions whose reach is not clear from the wording:

- "repairable" and "not vehicle-like" stand before both relative clauses. They may qualify the
  building in both branches, or only the allied branch.
- "can be garrisoned and is not being erased" follow the second branch. They may belong to the
  passive-neutral branch only, or close the whole definition and also apply to the allied branch.

Our engine tests "repairable and not vehicle-like" for the allied branch only, "can be garrisoned"
for the passive-neutral branch only, and, since sim issues S23 (#358), "not being erased" for the
passive-neutral branch only. The answer decides rows I4 to I7 for an engineer over:
- a damaged building of a passive neutral house that is not repairable, or vehicle-like;
- a damaged allied building that is being erased.

## Questions
1. Do "repairable" and "not vehicle-like" apply to a building of a passive neutral house?
2. Does "is not being erased" apply to an allied building too?
3. Does "can be garrisoned" apply to an allied building too?

## Possible observation
An engineer selected over a damaged garrisonable building of a passive neutral house whose type is
not repairable: the cursor shows repair (row I6) when question 1's answer is no. An engineer over a
damaged allied building while a temporal beam erases it: the cursor shows repair when question 2's
answer is no. An engineer over a damaged allied building that cannot be garrisoned: repair when
question 3's answer is no.

## Current workaround
`engineer_friendly` in `crates/deterministic/sim/src/targeting.rs`:
`(repairable && not vehicle-like && allied) || (passive neutral && garrisonable && not being
erased)`.
