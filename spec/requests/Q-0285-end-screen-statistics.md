---
id: Q-0285
title: Which statistics does the end-of-game screen show?
status: answered
related: [S-0414, S-0201]
answered-by: [S-0414]
---

## Need

S-0414 gives when a game is over and each house's verdict. The end screen after a skirmish shows
statistics per player, but no spec says which ones, how each is counted (for example whether
units built count deploys or free units, whether "harvested" counts refinery income), or how the
game time is shown.

## Questions

1. Which per-player statistics does the end screen show, and how is each counted?
2. How is the game's duration shown, and is it game time (ticks at the paced period) or wall
   time?
3. Is a score shown, and how is it computed?

## Possible observation

Play short skirmishes with known events (build counts, kills, losses, harvests) and record the
end screen.

## Current workaround

Interim rule (`glasnost_ui::end`): per house, objects ever built (the tech tree's counts), kills,
losses and score as the simulation keeps them (S-0414); the duration is the over tick times the
paced period of S-0201 claim 4 at the game's speed. No harvested-credits figure is shown (the
simulation keeps none). Recorded in docs/development.md, "Game client (M5b)".
