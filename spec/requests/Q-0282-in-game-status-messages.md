---
id: Q-0282
title: Which text messages does the game screen show, and for how long?
status: answered
related: [S-0411, S-0406, S-0408, S-0213]
answered-by: [S-0411]
---

## Need

Several specs mention a message shown to the player (S-0213 claim 13's refused route-mode click
has "a message and a sound"; refused placements, insufficient funds and similar refusals are
reported). The game screen needs to know where messages appear, their text, how long each stays
and how many show at once.

## Questions

1. Which events show a text message on the game screen, and with which text?
2. Where do messages appear, how many are shown at once, and how long does each stay (in ticks,
   frames or time)? Does pausing stop the timer?

## Possible observation

Trigger each refusal in a skirmish and record the screen over the following seconds.

## Current workaround

Interim rule: the client shows its own short texts (for a refused build, a refused placement, a
refused sell or repair click, and the game start) in the top-left corner of the battlefield, at
most four at once, each for 90 ticks of game time. Recorded in docs/development.md, "Game client
(M5b)".
