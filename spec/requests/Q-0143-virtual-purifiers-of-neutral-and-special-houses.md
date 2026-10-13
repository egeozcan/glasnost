---
id: Q-0143
title: Which virtual-purifier list position the neutral and special houses use
status: answered
related: [S-0402, S-0412, S-0009]
answered-by: [S-0402]
---

## Need

S-0402 claim 12 covers a refinery owner that is not human-controlled. Such an owner adds virtual
purifiers to the purifier count *n*ₚ, taken from the rules' list at the position of its difficulty
row. The claim says three things about who gets them:

- in our engine the rule never holds for an AI house;
- it does hold for the neutral house and the special house;
- which list position those two houses use is not determined (S-0402 Open questions).

The engine needs this in `econ::harvest::purifier_count`.

## Questions

1. Which difficulty row (list position) do the neutral house and the special house use for the
   virtual purifiers?
2. Is that position the same in every skirmish, or does it follow a setting such as the game's AI
   difficulty?

## Possible observation

Run scenario H6 part c with the refinery owned by the neutral house, either captured and handed
over or pre-placed. Give each position of the virtual-purifier list a different value, and record
the neutral house's balance at a payment.

## Current workaround

The neutral and special houses count 0 virtual purifiers, as claim 12's departure does when the
list has no entry at the position. This is listed in the provisional table of
`docs/development.md`.
