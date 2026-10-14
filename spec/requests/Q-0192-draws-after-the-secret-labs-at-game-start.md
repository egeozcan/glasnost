---
id: Q-0192
title: Are the secret-lab draws the last draws of game start, or do the game-start pass's tile-animation draws follow them?
status: answered
related: [S-0412, S-0202, S-0010, S-0407]
answered-by: [S-0412]
---

## Need
S-0412 claim 14 (revision 3) says the secret-lab draws are the last draws of game start and that
no draw follows them before tick 0. S-0202 claim 14 (the answer to Q-0101) says the game-start
recalculation of S-0010 claim 12 takes its tile-animation creation draws after the starting
units' final draw, the crate placement draws, the secret-lab choices and the end-of-loading tag
draws, and before the first tick. Both cannot hold when the game-start pass creates a tile
animation. The engine orders these steps in `load::new_game`.

## Questions
1. In a skirmish where the game-start pass starts a tile animation that takes creation draws, do
   those draws come after the secret-lab draws (as S-0202 claim 14 says) or before them (as
   S-0412 claim 14's "last draws" implies)?
2. If they come after, should S-0412 claim 14 and its state table list the game-start pass (and
   the end-of-loading tag draws) as a later group, so that the two specs agree?

## Possible observation
S-0412 scenario K9 part a with a secret lab on a map where a cell's tile animation first starts
in the game-start pass; compare the scenario random stream's state with both orders.

## Current workaround
The engine follows S-0202 claim 14: `econ::setup::start_game` (crates, secret labs) runs before
the game-start pass's `recalc_cells`, so tile-animation draws of that pass follow the secret-lab
draws. The end-of-loading tag draws are not taken (map tags are not implemented, S-0011).
