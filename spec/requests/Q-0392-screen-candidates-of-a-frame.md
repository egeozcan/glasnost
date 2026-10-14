---
id: Q-0392
title: Which objects are a drawn frame's screen candidates, and in what order inside a group?
status: open
related: [S-0214]
answered-by: []
---

## Need

S-0214 claim 20 admits at most 500 screen candidates per drawn frame, mobile candidates before
buildings, and a band box takes only admitted candidates. The claim leaves the order within an
overflowing mobile group unmeasured, and claim 19 leaves the box point during transient drawing
states open. The client (`session::box_candidates`, `band::admit`) must also decide which objects
are screen candidates at all.

## Questions

1. Which objects in play are a frame's screen candidates: those whose box point lies in the
   battlefield view, those any part of whose image is in the view, or another set (for example
   with a margin around the view)?
2. When more than 500 mobile candidates are on screen, which ones are admitted: in what order are
   the mobile candidates counted (ground and air layers, houses, creation order, screen position)?
3. Does a vehicle-like building count with the mobile candidates or with the buildings?
4. Do terrain objects, overlays or animations take places among the 500?

## Possible observation

S-0214 Scenario T15 part B, with the crowded runs it names; add an object whose image only
partly enters the view and a vehicle-like building among 500 mobile units.

## Current workaround

A screen candidate is a non-terrain object in play whose box point is in the battlefield view;
mobile objects come first, each group in our object order, and a vehicle-like building is counted
as a building.
