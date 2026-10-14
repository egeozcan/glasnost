---
id: Q-0371
title: A harvester at rest on a refinery's docking cell while another holds the dock
status: open
related: [S-0402, S-0417, S-0212, S-0215]
answered-by: []
---

## Need

In long synthetic AI-against-AI games our engine reaches a state that never
resolves, and no claim of S-0402 or S-0417 says what the game does instead.

- Harvester A has come to rest (*guard*) standing on refinery R's docking
  cell (S-0402 Terms). It is not tethered to R and does not hold R's dock.
- Harvester B, of the same house, holds R's dock (S-0402 claim 9). B is in
  *enter* and is directed to the docking cell (S-0402 claim 10, row 2). It
  never reaches that cell because A stands on it, so B moves back and forth
  between the cells next to it for thousands of ticks.
- The owner is a computer house. Its AI orders A to harvest every few
  seconds (S-0402 claim 13). Each time, A finds an ore cell within reach,
  gets that cell as its destination, and within the same tick the movement
  layer drops the destination and A is back in *guard* on the docking cell.

The gap is in `econ::harvest` (the dock and *enter* handling) and in the
movement layer's handling of a destination it cannot reach, which other
sessions own (S-0212, S-0215).

## Questions

1. Can a harvester that does not hold a refinery's dock come to rest on that
   refinery's docking cell? If it can, does anything move it off the cell
   when another harvester holds the dock and is directed there, for example a
   request to step aside or a scatter? After how long, and to which cell?
2. Does a harvester in *enter* that holds the dock but cannot reach the
   docking cell, because a friendly unit at rest stands on it, ever give up
   the dock, wait at a queueing cell, or choose another refinery? After how
   long?
3. A harvester at rest on a docking cell is given a harvest order. It finds an
   ore cell within reach whose route starts by leaving the docking cell. Does
   it drive there? If the route cannot be found, what does it do next, and
   when does it try again?

## Possible observation

A purpose-made skirmish map with one refinery and two harvesters of one human
player. Drive harvester A onto the refinery's docking cell with a move order
while harvester B is unloading, then order B to return to the refinery. Record
both harvesters' cells, behaviours and destinations for about 2000 ticks.
Repeat with A ordered to stop, and again with a computer player as the owner.
Repeat with the ore field reachable only past the refinery.

## Current workaround

None. Our engine follows S-0402 and S-0215 as written, and the state above
persists for the rest of the game. No rule has been invented to break it.
