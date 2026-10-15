---
id: Q-0682
title: Which exit cell a repair depot picks for a vehicle it sends away without a rally point (S-0314 claim 6)
status: answered
related: [S-0314, S-0404, S-0417, S-0504]
answered-by: [S-0314]
---

## Need
S-0314 claim 6: a vehicle sent away from a depot "moves to the depot's rally point when the depot
has one (S-0213, claim 20), and otherwise to an exit cell that the depot picks next to itself
(the structures specs own that choice)". No structures spec states that choice for a repair
depot or for a naval yard that repairs: S-0404 claim 13 and S-0417 give the exit of a produced
unit from a factory, not of a repaired one, and S-0314's Open questions leave the case without
any such cell open. The engine needs the cell in `depot::send_away`.

## Questions
1. Which cell does a depot pick for a repaired vehicle when it has no rally point: which cells
   next to its foundation are candidates, in which order are they tried, and what makes a cell
   unsuitable (occupied, impassable to the vehicle, another reach region)?
2. Is the choice the same for a naval yard sending a repaired vessel away?
3. Does the pick take a random draw?
4. When no cell is found, does the vehicle get no destination and stay on the depot?

## Possible observation
H4 with the depot's neighbouring cells blocked one at a time, recording the destination the
released vehicle receives.

## Current workaround
Interim rule, recorded in docs/development.md: the nearby-cell search of S-0212 claim 25 in its
destination-substitution use, centred on the depot's top-left cell, skipping the depot's own
foundation cells; no cell found gives no destination.
