---
id: Q-3008
title: Does the cliff factor's "unit on a bridge" row cover a unit on a low bridge?
status: answered
related: [S-0102, S-0211]
answered-by: [S-0102]
---

## Need

S-0102 claim 14's cliff-factor table gives factor 0 for a "unit on a bridge", whatever the cells
toward the viewer. Scenario D9 Map C tests only a unit on the deck of a high bridge. The spec does
not say whether a unit on a low bridge counts as "on a bridge" here. A low bridge has no raised deck,
and its units stand on the ground layer of their cells.

The client (`crates/app/src/mapview/build.rs`, `Painter::unit_depth`; render issue #95 on GitHub)
now gives factor 0 to a unit on a high bridge's deck. It needs to know whether a unit on a low
bridge also gets 0.

## Questions

1. Does a vehicle or infantry unit standing on a low bridge, with the cells (*x* + 1, *y* + 1) or
   (*x* + 2, *y* + 2) more than 3 levels higher than its cell, take cliff factor 0 or the factor of
   the "no" rows?
2. Is a unit that drives under a high bridge (not on its deck) "on a bridge" for this row?

## Possible observation

As S-0102 scenario D9, with map C's parking cell on a low bridge whose bank cells toward the viewer
are more than 3 levels higher than the parking cell. Run with the cliff fudge at 0 and at 40, and
compare the hidden rows. Repeat with the parking cell under a high bridge's deck.

## Current workaround

Factor 0 only for a unit on a high bridge's deck. A unit on a low bridge, or under a high bridge,
takes the factor of the "no" rows. This is recorded in `docs/m8-claims/render-issues.md` (batch R4,
#95).
