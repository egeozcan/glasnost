---
id: Q-1834
title: How a debris piece rebounds on a sloped cell and off a cliff face, and which buildings and overlays it meets
status: answered
related: [S-0317, S-0106]
answered-by: [S-0317]
---

## Need
S-0317 claim 10 gives a piece's rebound on a flat cell, v′ = e(v_x, v_y, −v_z), and says that a
move into a cell two or more height levels above is undone, but it leaves three things open (its
Open questions): the rebound on a sloped cell, the velocity after the turn off a cliff face, and
which cells count for the building contact. `glasnost-sim` `pieces::fly` must pick a velocity in
each case, so it needs these to follow the game.

## Questions
1. A piece that comes down on a sloped cell: what is its velocity just after the rebound, for each
   slope type of the cell, as a function of its velocity just before it? (Does the horizontal
   speed change, and in which direction, or is it the flat rule of claim 10?)
2. A piece whose move is undone at a cliff face (claim 10's cliff test): what is its velocity just
   after the turn, for each direction the face can face (the move crossing a cell edge along x,
   along y, or at a corner along both), as a function of its velocity before the move?
3. Claim 10's building contact (less than 150 leptons above the ground in a cell that holds a
   building): which buildings do not count (for example walls, gates, buildings still under
   construction or awaiting removal, a building of no owner), and does any overlay count (a wall
   overlay, for instance)?

## Possible observation
Scenario V7's recorder (S-0317) logs each piece's velocity per tick. Run it with pieces thrown on
each slope type, at cliff faces of each direction, and onto cells holding each kind of building
and wall overlay, and compare the velocity in the tick of the contact with the one before it.

## Current workaround
- A sloped cell rebounds as a flat one: only the vertical component is negated, then all three are
  scaled by the elasticity.
- At a cliff face, after the move is undone, each horizontal component along which the cell
  coordinate changed is negated, then all three are scaled by the elasticity.
- Every cell whose ground-layer occupancy holds a building counts; no overlay counts.

Each cites Q-1834 in `crates/deterministic/sim/src/pieces.rs`.
