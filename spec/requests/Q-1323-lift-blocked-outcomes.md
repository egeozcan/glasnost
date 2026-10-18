---
id: Q-1323
title: Which occupancy outcomes make a drop cell blocked
status: open
related: [S-0607, S-0209]
answered-by: []
---

## Need
S-0607 claim 17 says a cell is blocked "when the occupancy test that S-0209 owns finds an obstacle for the unit". S-0209 claim 5's vehicle test gives several outcomes: clear, moving blocker, stationary friendly, friendly obstacle, enemy obstacle and blocked.

## Questions
1. Which of these outcomes count as an obstacle for claim 17? Is it every outcome but clear?
2. In particular: a moving allied unit (moving blocker), an enemy wall the unit's weapon could destroy (enemy obstacle), and a crushable occupant.

## Possible observation
Drop a unit with a low falling-damage multiplier onto cells holding each kind of occupant, and see whether it lands or takes the impact.

## Current workaround
Every outcome other than clear counts as blocked.
