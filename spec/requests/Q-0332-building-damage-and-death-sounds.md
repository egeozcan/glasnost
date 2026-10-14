---
id: Q-0332
title: When the game-wide building damage and building death sounds play
status: open
related: [S-0701, S-0302]
answered-by: []
---

## Need

S-0701 claim 23 names the game-wide building damage and building death sounds. It leaves when they
play to S-0302 claim 12 and the structures specs. S-0302 claim 12 only points at "the building's
damage sound on now yellow or now red", which the structures specs own. No structures spec gives
it yet.

## Questions

1. On which damage-state results does a building play the game-wide building damage sound? Where
   does it play: at the building as a positional sound, or as a screen sound? Is it heard for any
   owner?
2. When does the game-wide building death sound play: on the killing hit, or when the building is
   removed? Where does it play?

## Current workaround

Neither sound plays. A building's own damage sound, death sounds and responses follow claim 20, as
for any other object. Documented in docs/development.md, "Audio".
