---
id: Q-0236
title: What a killed aircraft whose type has an empty explosion list shows, and whether it takes a draw
status: open
related: [S-0312]
answered-by: []
---

## Need

S-0312 claim 16 says that a killed aircraft shows one entry of its type's explosion list,
chosen by one list pick, when the list is not empty, and that what happens when the list is
empty, "including whether an animation appears or a draw is taken, is unknown". A draw changes
every later random result, so our engine needs the rule. This is the aircraft branch of
`combat::death` (the death effects by kind).

## Questions

1. When a killed aircraft's type has an empty explosion list, does an animation appear at its
   position? If so, which one: an entry of some other list, or an animation chosen another way?
2. Does that death take a draw from the simulation's random stream where the list pick would
   be? If so, what kind of draw (a list pick over what count, or a ranged draw)?
3. Does the answer depend on whether the aircraft is in the air or on the ground when killed?

## Possible observation

S-0312 scenario D11 with an aircraft type whose explosion list is empty, killed on the ground
and in the air. Record the animations created at its position on the tick of its death, and
the random stream position before and after that death (for example by a random choice of a
following event that depends on it).

## Current workaround

An empty list shows no animation and takes no draw. Interim row in `docs/development.md`.
