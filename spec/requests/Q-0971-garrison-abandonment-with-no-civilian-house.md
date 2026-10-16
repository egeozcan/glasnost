---
id: Q-0971
title: Return of an emptied civilian garrison building when no house plays a civilian-side country
status: open
related: [S-0508, S-0414]
answered-by: []
---

## Need

S-0508 claim 8 hands an emptied civilian garrison building to the civilian house, the first house
whose country belongs to the civilian side. Its Edge cases say that with data in which no house
has such a country the original "does not handle" the case, and ask the project lead to adopt
leaving the owner unchanged and skipping the abandoned feedback (Questions for the project lead).
That decision is not yet recorded as a rule.

## Questions

1. With no civilian house, does an emptied civilian garrison building keep its owner at every
   later turn, with no abandoned sound, radar event or announcement?
2. Does the same hold for claim 9's forced evacuation (the building is emptied and stays with
   its owner)?

## Possible observation

Not observable safely in the original (S-0508 marks the outcome unsafe).

## Current workaround

Interim rule (`structures::garrison::civilian_house` finds none): the owner is left unchanged and
no abandoned feedback is given; the occupant-owner change of claim 7 is unaffected.
