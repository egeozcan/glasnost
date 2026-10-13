---
id: Q-0101
title: Where the game-start recalculation pass, the unused-house cleanup and low-bridge cell animations sit among the load's draws
status: open
related: [S-0009, S-0010, S-0202, S-0412]
answered-by: []
---

## Need
`glasnost_sim::load::new_game` now runs S-0010 revision 10's game-start
pass (claim 12: a third whole-map terrain recalculation "after the skirmish
setup has placed its starting units, and before the first tick", which can
start tile animations with their creation draws, claim 19) and S-0009's
unused-house cleanup (claim 28: "once all map objects are placed, before the
late phase"). Removing a house's pre-placed objects takes them out of their
cells, which recalculates those cells (S-0010 Terms) and can start a tile
animation with its draws. Neither position is fixed against the other groups
of draws in S-0202's table "Draws during map loading" or S-0412's game-start
table.

## Questions
1. **The game-start pass.** Do the tile-animation creation draws of the
   game-start pass come after the final draw R(0, 65535) that ends the
   starting units (S-0009 claim 20), or before it? Where do they sit against
   the crate draws of S-0412 claim 10 (the "second group of draws" after the
   starting units) and the secret-lab draws of S-0407 claim 14 (S-0412 calls
   them the last draws of game start)? Does the pass run in the same place in
   a game without starting units?
2. **The cleanup against the second pass.** Does the unused-house cleanup
   run before or after the second recalculation pass of S-0010 claim 12
   (both come after the pre-placed objects and before the late phase)?
3. **Recalculation on removal.** When the cleanup removes a house's
   pre-placed objects, are their cells recalculated (as for any object taken
   out of a cell in play, S-0010 Terms), so that a tile animation can start
   there with its draws?

4. **Cell animations of low-bridge writes.** S-0009 claim 12 starts a cell
   animation "each time an overlay whose type has a cell animation is
   placed". Does that include the end-row overlays and the bridge pieces
   that a low-bridge end marker writes (S-0202 claim 10)? If so, where do
   the animation's creation draws come among each piece's raw value and the
   recalculation of its cell?

## Possible observation
S-0202's scenario R8 with the random-state recorder: (1) a map whose
game-start pass starts a tile animation with a random rate pair (S-0010
scenario G4 part C's data), with and without the crate setting; (2) and (3)
S-0009 scenario M16 variant A with a removed neutral object standing on the
attaches-to cell of a tile whose tile animation has a random rate pair, the
cell having started no animation.

## Current workaround
1. The game-start pass runs after the starting units' final draw (and after
   the sandbox's final draw in our viewer's setup), before the passability
   categories and regions are built. Crates and secret labs are not
   implemented.
2. The cleanup runs after the second pass, directly before the late phase.
3. The removed objects' cells are recalculated, as for any removal, with the
   tile-animation rule of claim 19 in force after the second pass.
4. Only overlays of the overlay type stream start cell animations; the
   end-row overlays and bridge pieces of a low bridge start none.
