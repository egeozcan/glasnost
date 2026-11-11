---
id: M-0013
title: Which cells receive spreading ore (S-0401 scenario E8)
topic: economy
specs: [S-0401]
requests: []
verdict: contradicts
value-visibility: local
measured-on: 2026-10-28
method: harness
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-11-11
---

# M-0013: Which cells receive spreading ore (S-0401 scenario E8)

## Setup

- Game build: stock Yuri's Revenge 1.001, executable hash verified; run under the oracle harness
  (PROVENANCE §7), with the harness's own gameplay additions off.
- Data: stock, no mods. Runs e8-s1 to e8-s5 use the stock rules. Runs e8f-s1 to e8f-s5 use a test
  rules file: the stock rules with the ore type's spread field set to 60 ticks (a test value).
- Map: a test map made from the stock multiplayer map `Kaliforn.mmx` (sha256
  `3da7ebe513ae2e6a78ac6b8343ff3d3c6d3e33bb7f518fe4e37d3e87a1e64497`, urban theatre). The test map
  file has sha256 `4d3b1357eb5d718fe8f3a7ee00727f9dc97522f17f0f83c02669e14532bd6c2a`. Its map
  options keep resource growth and resource spreading on. Changes to the map:
  - every ore and gem cell and every ore mine of the map is removed
  - every tree inside the pens' 3 × 3 blocks is removed, and the test trees are placed
  - pens are added. Each pen's centre is one full-grown ore cell (stage 11). All eight of its
    neighbours hold the wall overlay, except the cell or cells under test. A centre can therefore
    spread only into its own test cells, and the 56 wall cells are test cells too.
- Pen centres: P1 (74,74), P2 (76,74), P3 (74,77), P4 (76,77), P5 (83,75), P6 (85,75), P7
  (74,89), P8 (76,89), P9 (78,89), A (96,85), B (97,87). Each test cell below is a neighbour of
  its pen's centre.
- Test cells. Each was read in the tick-0 baseline for land type, slope, height level, tile and
  tileset, bridge bits, overlay and objects. That baseline is read before tick 0's logic: the free
  cells that gain ore in tick 0 show no overlay in it. The table gives what sets each cell apart
  from a free control cell: one condition, and two at (97,84).

| Pen | Test cell | What sets it apart from a free control cell | The rest of the cell |
|---|---|---|---|
| P1 | (74,75) | a tree (stock type `TREE18`) | rough land, flat, level 6, tileset allows ore, no overlay |
| P2 | (76,75) | a tree (stock type `TREE01`) | same |
| P4 | (76,76) | a living building (a neutral mail box, stock type `CAMISC04`) | same |
| P5 | (83,76) | an ore mine (stock type `TIBTRE01`) | same |
| A | (95,84), (96,84) | a high-bridge cell (a bridge over flat land, bridge bits set) | rough land, flat, level 2 (the centre's level), tileset allows ore |
| A | (97,84) | two conditions: a high-bridge cell, on rock land (rock alone is pen B's test) | |
| B | (98,86), (98,87), (98,88) | rock land (not buildable) | flat, level 2, the tile's tileset allows ore, no bridge bits |
| all | 56 cells | the wall overlay | |
| P7, P9 | (74,90), (78,90) | none: a neutral civilian stands in it (stock types `CIV1` in P7, `CIV2` in P9) | rough land, flat, level 6, tileset allows ore |
| P3, P6, P8 | (74,76), (85,76), (76,90) | none: free control | same |

- Infantry pens. The pens are walled on the centre's ring only. The civilian of an infantry pen can
  stand on the test cell or the centre, or step out through the pen's open side (the side away from
  the centre). The per-tick record says which cell holds it.
- Players: three houses. The human player (British, start position 3) is an idle observer whose
  objects cannot be targeted or damaged; it owns one power plant at (55,59). Two computer players,
  Americans (start position 0) and Russians (start position 1), not allied with each other and
  each at the hardest difficulty: the Americans own a power plant at (104,109) and a construction
  yard at (99,109), the Russians a power plant at (141,66) and a construction yard at (136,66).
  Every house starts with 0 credits. The
  map's neutral objects stay in play.
- Game options: tech level 10, superweapons off, crates off, short game off, bases on, the map
  revealed.
- Seeds 101 to 105 (stock rules) and 201 to 205 (test rules), 6000 ticks each, the fastest game
  speed. All ten runs reached the tick limit.
- Each run is a separate launch of the game program, so each run is the first game of its
  session.
- What is not stock:
  - no menus: the skirmish setup comes from a settings file
  - the computer players' starting construction vehicles are removed
  - the test map, and in e8f the test rules file, as stated above
  - each computer player owns a construction yard that the stock map does not give it
  - the computer players are not allied with each other (stock skirmish would ally them)
  - the observer's objects cannot be targeted or damaged
  - the computer players' credits are set to 0 before tick 0
  - the run ends only at its tick limit
  - a per-tick recorder reads state only
- Tick convention: "tick *f*" is the game's frame counter. A cell state stamped *f* is read after
  tick *f*'s logic has run. The tick-0 baseline is the exception: it is read before tick 0's logic,
  so a gain stamped 0 (a "0" in the tick table below) is a gain in tick 0's logic.

## Procedure

1. Record every cell of x 72 to 99, y 72 to 92 on tick 0, then each change after a tick's logic:
   overlay type and stage, land type, slope, level, tile, bridge bits, and the cell's objects in
   list order.
2. For each test cell, note whether it ever holds an ore overlay, and the tick and stage of each
   gain. Note which cell each civilian stands in on the ticks before and after each gain next to it.

## Result

| Quantity | Unit | n | Value | Spread | Tolerance for tests |
|---|---|---|---|---|---|
| Free control cells P3, P6, P8 receiving ore | runs | 10 | below the table | | exact |
| Infantry cells P7, P9 receiving ore | runs | 10 | below the table | | exact |
| Tree cells P1, P2 receiving ore | runs | 10 | below the table | | exact |
| Living-building cell P4 receiving ore | runs | 10 | below the table | | exact |
| Ore-mine cell P5 receiving ore | runs | 10 | below the table | | exact |
| High-bridge cells (95,84), (96,84), (97,84) receiving ore | runs | 10 | below the table | | exact |
| Rock cells (98,86) to (98,88) receiving ore | runs | 10 | below the table | | exact |
| Wall cells receiving ore | runs | 10 | below the table | | exact |
| Ticks of spread gains, stock rules | tick | 5 | below the table | per cell and run, table below | exact |
| Ticks of spread gains, test rules | tick | 5 | below the table | table below | exact |
| Stage of a new ore cell in the tick it appears (spread steps and the ore mine's) | stage | 118 gains | below the table | none | exact |
| Ore mine's new cells: ticks, stock rules | tick | 5 | below the table | | |
| Ore mine putting ore on its own cell or on a wall | runs | 10 | below the table | | exact |

Relations for the table above:

- *Free control cells P3, P6, P8 receiving ore*: 10 (all three cells).
- *Infantry cells P7, P9 receiving ore*: 10 (both cells).
- *Tree cells P1, P2 receiving ore*: 0.
- *Living-building cell P4 receiving ore*: 0.
- *Ore-mine cell P5 receiving ore*: 0.
- *High-bridge cells (95,84), (96,84), (97,84) receiving ore*: 0.
- *Rock cells (98,86) to (98,88) receiving ore*: 0.
- *Wall cells receiving ore*: 0 (all 56 cells).
- *Ticks of spread gains, stock rules*: 0, *S* or 2*S* (*S* local).
- *Ticks of spread gains, test rules*: multiples of 60 (0 to 120 for the test cells).
- *Stage of a new ore cell in the tick it appears (spread steps and the ore mine's)*: 3.
- *Ore mine's new cells: ticks, stock rules*: 77 to 2004, not on the spread schedule.
- *Ore mine putting ore on its own cell or on a wall*: 0.

Ticks of the gains in the test cells (*S* is the stock ore type's spread field, a local value):

| Run | P3 / P6 / P7 / P8 / P9 | Run | P3 / P6 / P7 / P8 / P9 |
|---|---|---|---|
| e8-s1 | *S* / 0 / 0 / 0 / 0 | e8f-s1 | 60 / 0 / 0 / 0 / 0 |
| e8-s2 | *S* / *S* / 0 / 0 / 0 | e8f-s2 | 120 / 60 / 60 / 60 / 60 |
| e8-s3 | 2*S* / *S* / 0 / 0 / 0 | e8f-s3 | 60 / 0 / 0 / 0 / 0 |
| e8-s4 | *S* / *S* / *S* / 0 / 0 | e8f-s4 | 60 / 60 / 0 / 0 / 0 |
| e8-s5 | 2*S* / *S* / *S* / 0 / 0 | e8f-s5 | 120 / 120 / 60 / 0 / 0 |

- **Relation (value in the local file).** Every recorded spread-step gain in the stock runs falls
  on a tick that is a multiple of the stock ore type's spread field, and every recorded spread-step
  gain in the test runs on a multiple of 60. The relation covers spread-step gains only: the ore
  mine's gains are in their own rows and are not on that schedule.
- **After-tick-0 ore gains in cells with civilian occupancy recorded before and after the gain.**
  A tick-0 gain cannot show whether the ore came before the civilian was placed. In each gain below,
  the per-tick record puts the civilian in the cell in the state after the tick before the gain and
  in the state after the gain's tick. That does not show occupancy at every instant between:
  - e8f-s2: (74,90) and (78,90) at tick 60; e8f-s5: (74,90) at tick 60
  - secondary spread out of a test cell into a cell holding a civilian: e8-s5 (77,91) at 2*S*,
    e8f-s2 (73,91) at 4440, e8f-s3 (73,91) at 4500, e8f-s5 (74,91) at 2280
  - P7's gains at *S* in e8-s4 and e8-s5 are after tick 0 but not in this list: whether the
    civilian stood in (74,90) before and after them is not recorded here
- **Secondary spread.** Cells that received ore went on to spread into free cells on the pens' open
  sides, also on ticks that are multiples of the spread field.
- **The ore mine** filled its three free neighbours (the pen's open side).

Against S-0401 revision 10:
- **Claim 10 is contradicted for tree cells.** Its scenario E8 expects "the infantry and tree cells
  can receive it" and is falsified if "the infantry or tree cell never does over the runs while
  free cells do". The tree cells never received ore in 10 of 10 runs while the free cells did in
  every run.
- Claim 10 is confirmed for the other cells of E8: the living building, the ore mine's cell, the
  high-bridge cells, the rock cells and the walls never received ore, and the infantry cells did.
- Claims 6, 9, 11 and 15 are confirmed as far as recorded: spread gains only on ticks that are
  multiples of the spread field; a new cell at stage 3; the mine's cells off
  the spread schedule and never on its own cell.

## Evidence

Ten runs (e8-s1 to e8-s5, e8f-s1 to e8f-s5), each to its 6000-tick limit. No recording or log hash
was supplied with the results.

## Notes

- **What is not recorded.** The scenario-stream draws of the spread steps were not recorded, so
  the record does not show which draws a spread step aimed at a tree cell takes. The order in which
  each pen's centre tried its neighbours was not recorded either.
- **Infantry cannot be held on one cell.** A first layout without walls tried a rifle infantry
  unit set to sleep, a neutral civilian set to hold its cell and a deployed rifle infantry unit as
  the infantry test. Each was moved to a neighbouring cell by the game's infantry scatter within
  about 16 to 80 ticks. That layout is superseded by the walled one. In a run of the first layout
  with a fast spread field (the number of its runs is not stated), (97,88) received ore at tick
  300 while a civilian stood in it. In the walled layout (97,88) is a wall cell of pen B.
- **Power plant sale.** In set-up trials outside the ten runs, without construction yards (their
  number is not stated), each computer player sold its power plant near tick 1030. The first
  layout's runs ended early, near tick 3040.
