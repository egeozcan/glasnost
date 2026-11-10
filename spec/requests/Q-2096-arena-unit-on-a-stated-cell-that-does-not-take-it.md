---
id: Q-2096
title: Where the arena records' harness puts a unit that its stated cell does not take
status: answered
related: [S-0216, S-0009, S-0209, S-0212, M-0001, M-0002, M-0003, M-0011, M-0015, M-0017]
answered-by: [S-0216]
---

## Need
The revised arena records (M-0001 to M-0003 and M-0017, revised 2026-10-29) withdraw the sentence
"in every game every unit stood on its stated cell". Their Setup now says: "units are created on
their stated cells before tick 0, and a unit that its stated cell does not take is placed on a
free neighbouring cell". M-0011 and M-0015 say: "The units are not map entries. They are created on
their stated cells once the scenario has loaded, before tick 0 … When the game does not accept a
unit on its stated cell, the harness places it on a free cell nearby" and "A unit can be placed
off its stated cell. Placed cells are not given here." M-0001's Notes tie this to the cliff cells
west of W that Q-1502 lists.

Our oracle arena (`crates/tools/src/oracle.rs`, `setup`) must decide, for each unit, whether its
stated cell takes it, and if not, which cell it stands on. The records do not say which test
refuses a cell, which neighbour is taken, or in what order. This is a fact about how the records
were made, so it belongs in the records' Setup (as S-0216's Open questions, *Q-1711 question 5*,
say).

## Questions
1. Which test decides that a stated cell does not take a unit? Is it the entry test of a play-time
   placement (S-0216 claim 5: only a *clear* entry outcome accepts), or another test (for example
   only whether the cell is already occupied)? Is it applied to vehicles and infantry alike, and
   to a group of an aircraft-kind type, if a record has one?
2. Which cell does the refused unit get: only the eight neighbours of its stated cell
   ("neighbouring cell", M-0001 and M-0017), or a cell further away when none of those is free
   ("a free cell nearby", M-0011)? In what order are the candidates tried, and what makes a
   candidate *free* (no unit on it, and the same test as question 1)?
3. Is the unit placed again in the chosen cell by the same kind of placement (with its entry test
   and, for infantry, its standing-position draw), and does the retry take random draws of its
   own?
4. When the chosen cell is the stated cell of a unit placed later, is that later unit then refused
   in turn and moved on, or does the harness avoid other units' stated cells?
5. In the games of M-0001 to M-0003, which units stood off their stated cells: the infantry on the
   *rock* cliff cells of Q-1502 only, or others too (vehicles there, units on the cells of the
   map's four neutral bunkers, units on cells with trees or other map objects)? Counts per type
   and side are enough, if the cells themselves are to stay local.

## Possible observation
Replay one game of the largest side-A infantry group of M-0002 (side A, seed 0) and one of a
side-A vehicle group of M-0001 (side A, seed 0), and record, for every unit, its stated cell and the cell it stands on at tick 0, and the
scenario-stream draws taken at each placement.

## What our test refuses
With the test of the workaround below, on the records' map, only side-A units are refused. A neutral building stands on
the cells (W.x − 9, W.y − 5) to (W.x − 8, W.y − 4), entry outcome *enemy obstacle*. Further west the cliff cells Q-1502 lists and a few more cells with entry outcome *blocked* refuse
units, and so do the cells that refused units took before their own units came.

## Current workaround
Our design, until this is answered (M8 step A3 intake 47): a stated cell takes a unit when it has
no unit on it and the unit's entry outcome there, asked as S-0216 claim 5 asks it (no approach
direction, against the cell's ground occupants), is *clear*. An aircraft-kind group is not tested.
A refused unit is tried on the square rings around its stated cell, ring 1 first, each in the scan
order of S-0212 claim 25 (row cells by x then y, then column cells by y then x), up to that claim's
largest radius, and stands on the first cell that takes it by the same test. First come, first
served: a later unit whose stated cell an earlier one took is refused there in turn and moves on;
no unit's stated cell is reserved. If no cell takes the unit, the setup reports an error. In every
cell the unit stands as before at the cell's centre point, with no standing-position draw and with
*hunt* current (A1's interim).

We first tried ring 1 alone, as "neighbouring cell" reads. In a placement probe (no game played)
the first unit of the fourth column had no cell in ring 1 that
took it (its stated cell and three neighbours on the neutral building, three on cliff cells, one
taken by the moved first unit of the third column), so the setup could not be made. The rings past the first follow M-0011's and M-0015's "a free cell nearby".
The rule was fixed after that probe and before any arena comparison with it, and is not tuned to
the records.
