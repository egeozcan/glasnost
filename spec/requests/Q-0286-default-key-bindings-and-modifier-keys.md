---
id: Q-0286
title: Which keys and modifier keys give the human order gestures by default?
status: open
related: [S-0213, S-0214]
answered-by: []
---

## Need

S-0213 table T2 lists the key commands (stop, scatter, guard, deploy, cheer, route mode) and says
the command keys are the player's own key bindings. S-0214 names the force-move, force-fire and
force-select gestures and attack-move mode. Neither says which key a fresh install binds to each
command, which modifier keys give the three force gestures, or which key opens the in-game
options menu. The game client (`app::game`) needs bindings before it has a key-binding screen.

## Questions

1. Which key is bound to each command of S-0213 table T2 in a fresh install?
2. Which held modifier keys give force-move, force-fire and force-select, and how is attack-move
   mode entered?
3. Which key opens the in-game options menu (S-0201 claim 8), and which keys scroll the view?

## Possible observation

Read the key-binding screen of a fresh install, and press each candidate key with units
selected.

## Current workaround

Interim rule: S stop, X scatter, G guard, D deploy, C cheer; Shift held is force-select,
Ctrl (or Cmd) held is force-fire, Alt held is force-move; attack-move mode is never entered by
a key; Escape opens and closes the options menu; the arrow keys scroll (Q-0279). No route mode.
Recorded in docs/development.md, "Game client (M5b)".
