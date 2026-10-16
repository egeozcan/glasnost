---
id: Q-0979
title: A crash start on a jumpjet that takes no update
status: open
related: [S-0502, S-0312]
answered-by: []
---

## Need

S-0502 claim 3 and its Open questions: a grounded jumpjet, or a hovering one with no move order,
takes no update, so a crash start that S-0312 claim 16 gives it is not taken up until something
else wakes it, and what wakes it is not traced. A killed jumpjet hovering with no order would stay
in the air for ever.

## Questions

1. Does a crash start on such a unit wake its update (so it falls at once), or does it stay
   suspended in the air until it is given an order?
2. If it stays, what ends it: a later order, or is it removed by another rule?

## Possible observation

Shoot down a hovering jumpjet vehicle that holds no move order (a deploy-to-land vehicle that
reached its hover with no order left is the likely case).

## Current workaround

A pending crash start counts like a held move order for the update (`jump::tick`), so the fall
starts in the unit's next update.
