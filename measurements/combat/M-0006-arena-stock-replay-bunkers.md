---
id: M-0006
title: Unit arena, stock replay of archived games, the map's neutral bunkers and moved units
topic: combat
specs: [S-0216]
requests: [Q-1711, Q-1713]
verdict: new-finding
value-visibility: committed
measured-on: 2026-10-28
method: harness
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-30
---

# M-0006: Unit arena, stock replay of archived games, the map's neutral bunkers and moved units

This record is a companion to M-0001, M-0002 and M-0003. It replays 32 of their 1860 archived
games and checks, for those 32 only, how far the archived games are stock behaviour. It gives the
cells of the four neutral bunkers that their Setup names, and the stated and placed cells of the
units stated on the five cliff cells of Q-1502. It does not change the records' per-game data.
M-0016 gives the stock rows that replace some of the 32.

## Setup

- Game build: stock Yuri's Revenge 1.001, executable hash verified; run under the oracle harness
  (PROVENANCE §7).
- Data: stock, no mods, no loose rules or art files.
- Map, players and game settings (M-0001's *Map* and *Players* lines), and the budget, stated cells,
  facings and the *hunt* mission (its *Groups* line): as in M-0001 (the stock map `Arena.mmx`, start
  waypoint 2, call its cell W). M-0001's *Groups* line also says how a unit that its stated cell
  does not take is placed. This record does not take that from it (*Placement*, below).
  Here W = (135, 83), so side A's unit *i* is stated at (129 − ⌊i/8⌋, 79 + (i mod 8)) and side B's
  at (141 + ⌊i/8⌋, 79 + (i mod 8)).
- Harness: the oracle's unit arena. Its house director is off for every house in both replays. The
  houses' own skirmish AI runs as in a normal game but has nothing to build with.
- **Replayed games (32).** Each is an archived game of M-0001 to M-0003, replayed from its archived
  setup and seed twice: once as archived, and once with the archive's five gameplay changes (below)
  switched off. The second replay is the stock reference.

  | Pairing (row against column) | Record | Row side | Seeds | Games |
  |---|---|---|---|---|
  | YTNK against SREF | M-0001 | A and B | 0 to 5 | 12 |
  | LTNK against ROBO | M-0001 | A and B | 0 to 5 | 12 |
  | TELE against BFRT | M-0001 | A | 0, 1 | 2 |
  | CAOS against BFRT | M-0001 | A | 0, 1 | 2 |
  | INIT against E2 | M-0002 | A | 0, 1 | 2 |
  | MGTK against TTNK | M-0003 | A | 0, 1 | 2 |

- **The five gameplay changes.** The archived games of M-0001 to M-0003 ran with the five gameplay
  changes of the harness that their Setup lists: a siege rule for long-range artillery, a change to
  Magnetron carrying, changes to the computer houses' production, extra defences, and an expansion
  and factory queue. All five were switched off together, so which of them caused a difference
  (Result) is not recorded.
- **Placement.** This record observes two things about the units' placement, both in Result: the
  units are in play in the replays' tick-0 records, and the archived logs name the cell each unit
  was finally placed on (*Units off their stated cells*). How the units were put on the map is not
  observed.
- What is not stock, in both replays:
  - no menus: the skirmish setup comes from a settings file
  - units are created on their stated cells before tick 0 (the harness's account, as M-0001 lists
    it). This record does not observe how (*Placement*)
  - the computer players' starting construction vehicles are removed
  - the two computer players are not allied with each other (a stock skirmish would ally every
    computer player when nobody is on a team)
  - the objects of the idle observer house, the human player's house, cannot be targeted or
    damaged
  - the run ends by M-0001's rule: at the first sample (every 304 ticks) at which only one side
    still owns anything, or at the first sample at or after tick 6000
  - the map's four neutral bunkers are in play from tick 0, as M-0001 lists it (observed: Result,
    *Map content*)
  - a periodic sampler and a per-tick recorder read state only
- What is not stock, in the archived replay only: the five gameplay changes named above. With the
  recorder on, the archived replay reproduces the archived games' results (the records' per-game
  rows) exactly.

## Procedure

1. Replay each of the 32 games twice, as above, recording per tick from tick 0 every unit's
   state, every damage and fire event and the scenario random stream's state. A unit's state
   includes its position, cell, health, mission and queued mission, target, destination, team and
   body and turret facing.
2. Compare the two replays game by game: the result, and the first tick at which any recorded
   quantity differs.
3. From the archived games' own logs (all 1860 games of M-0001 to M-0003), each of which names the
   cell every unit was finally placed on, list the games in which some unit was placed on a cell
   other than its stated one, with each such unit's stated and placed cell.
4. List the map's structures that are in play at tick 0 and owned by neither side, with each one's
   owning house.
5. In the per-tick records of both INIT against E2 replays, list every fire event of side A's
   units in ticks 0 to 100 with its target. In the LTNK against ROBO replays, list each
   detonation that damages a structure of step 4 (M-0005 reads these per shot). These listings
   stay local; Result gives only what they show about the bunkers.

## Result

**Archive against stock.**

| Quantity | n | Value |
|---|---|---|
| Replayed games with the same winner in both replays | 32 | 28 |
| Games identical tick for tick in both replays (every unit state of Procedure step 1, damage and fire event, random state) | 32 | 21 (the 20 games of the five other pairings, and YTNK against SREF, B5) |
| Games with different winners | 32 | 4, all YTNK against SREF |

| YTNK against SREF game | Archived winner (M-0001) | Stock winner |
|---|---|---|
| A0 (YTNK on side A, seed 0) | YTNK | SREF |
| A3 | SREF | YTNK |
| A5 | SREF | YTNK |
| B4 (YTNK on side B, seed 4) | YTNK | SREF |

- The replays' per-tick records begin at tick 0, and the units are in them, in play, from that
  tick. How they came to be there is not observed (Setup, *Placement*).
- YTNK wins 8 of 12 games in both replays, but not the same games: as side A, 3 of 6 archived and
  4 of 6 stock; as side B, 5 of 6 archived and 4 of 6 stock.
- In the other 11 YTNK against SREF games the replays first differ in an SREF's orders. In the
  archive those SREFs creep forward under a queued order. In the stock replay they stand still in
  the 6 games with YTNK on side A, and move to a destination cell in the 5 games B0 to B4. Their
  random states differ too. The ticks of the first differences stay local.
- So switching off the archive's gameplay changes changed the winners of four YTNK against
  SREF games. In the 20 replayed games of the other five pairings, the two replays are identical
  tick for tick. Every archived game with SREF or V3 on either side is replayed with stock rules
  in M-0016, which gives the stock rows that replace the changed ones, these four included.

**Map content.** Four bunker structures are in play from tick 0. They belong to the map's neutral
house, which is neither side and not the observer house:

| Cell | Relative to W | Note |
|---|---|---|
| 36,80 | (W.x − 99, W.y − 3) | far from both groups' stated cells |
| 36,85 | (W.x − 99, W.y + 2) | far from both groups' stated cells |
| 126,78 | (W.x − 9, W.y − 5) | the row north of side A's stated cells |
| 126,87 | (W.x − 9, W.y + 4) | the row south of side A's stated cells |

- Only the listed cell of each bunker is recorded.
- In the replays read by Procedure step 5, units fire at a bunker, and shells aimed at a robot
  tank damage one (M-0005).
- In this record's replays no unit was in a team at any tick. This was not checked against a game
  that has teams.

**Units off their stated cells.**

| Quantity | n | Value |
|---|---|---|
| Archived games in which a unit was not placed at all | 1860 | 0 |
| Moves on side B | 1860 | 0 |

Read from the logs, some archived games place some side-A units on a cell next to their stated
cell. This record gives no count of those games and does not say which games they are, since the
counts would bound group sizes. The moves of the units stated on the five cliff cells of Q-1502:

| Stated cell | Placed on |
|---|---|
| 125,79 | 124,78 |
| 125,80 | 126,80 |
| 124,79 | 123,78 |
| 124,80 | 123,79 |
| 125,86 | 124,85 |

- In every archived game in which a unit is stated on one of these cells, the logs place it on
  the cell this table gives. The stated and placed cells of the other moved units are in the
  archived logs and are not given here, for the same reason as the counts.
- In no archived game was a unit placed on any of the five side-A cells that Q-1502 lists
  (124,79, 124,80, 125,79, 125,80 and 125,86): each unit stated there was placed on a
  neighbouring cell, and none of the placed cells is among those five.
- Some placed cells hold more than one unit, so M-0001's "Infantry also take one cell each" does
  not hold for them. Which cells, and how many units each holds, is not given beyond what the
  cliff-cell table shows, for the same reason as the counts.
- Why each unit's placed cell differs from its stated cell is not recorded. Every move is to a
  neighbouring cell, as M-0001's Setup says, but M-0001's "free neighbouring cell" does not hold
  for every one: some end on a cell that already holds a unit.

**Tolerance for tests.** Exact for the bunker cells, the placed cells of the cliff-cell table and
the counts of the *Units off their stated cells* table. The replay comparisons (the first
differences and the changed winners) are not for tests: they describe the archive, not a
rule.

**Requests.** For Q-1711 question 5 the record shows only that the units are in play in the replays'
tick-0 records; how they were put on the map is not observed, so the record settles only that the
units are in play at tick 0, and the rest of that question is open. For Q-1713 question 1 the record
shows that no archived game placed a unit on a cliff cell of Q-1502, so the question's premise
(units placed on those cells) does not hold in the archived games; what ends those games is not
observed.

## Evidence

- 32 games, each replayed twice; per-tick records of both replays kept by the operator.
- The archived logs of all 1860 games of M-0001 to M-0003 were read for each unit's placed cell.
- Nothing went wrong in the replays. The archived replay reproduced the archived games' results
  exactly with the recorder on.
- No recording or log hash was supplied. The tick-0 cells of the 32 replays were not compared
  with the cells the archived logs name, so for a replayed game with moved units the record does
  not show that its replays placed each unit on the cell its log names. Which cells units stood
  on later in the replays is not reported.
- The per-tick records of the replays are the source of Procedure step 5's fire events.

## Notes

- **The precedent records' account of placement.** M-0001, M-0002 and M-0003 list "units are
  created on their stated cells before tick 0" and "the map's four bunkers … in play from tick 0"
  under what is not stock, and M-0011 and M-0015 say the units are created on their stated cells
  before tick 0. Those statements are the oracle's account of the harness. This record does not
  confirm them: it observes only the tick-0 records and the logs' placed cells (Result), and it
  shows that the bunkers belong to the map's neutral house and are in play from tick 0.

- **Comparing against M-0001 to M-0003.** This record does not edit them. A comparison against
  them needs the four neutral bunkers in play and each moved unit on its placed cell (Result), or
  it is not the same game: the placed cells are in the archived logs. This record's own replays
  were not checked against that: their tick-0 cells were not compared with the logged cells
  (Evidence).
- The 4 YTNK against SREF games whose winner changed are rows A0, A3, A5 and B4 of M-0001's
  YTNK against SREF pairing. M-0016 gives their stock rows.
