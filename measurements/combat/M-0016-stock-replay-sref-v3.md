---
id: M-0016
title: Unit arena, stock replay of every archived game with SREF or V3
topic: combat
specs: []
requests: []
verdict: new-finding
value-visibility: local
measured-on: 2026-10-28
method: harness
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-11-11
---

# M-0016: Unit arena, stock replay of every archived game with SREF or V3

This record replays, with stock rules, every archived game of M-0001 and M-0003 in which SREF or
V3 is on either side, and gives the stock row for each game that its comparison counts as a
changed row. Those stock rows replace the archived ones (Result). Every other row of those
pairings is confirmed as stock behaviour at its end sample. M-0002 has no game with SREF or V3.

## Setup

- Game build: stock Yuri's Revenge 1.001, executable hash verified; run under the oracle harness
  (PROVENANCE §7).
- Data: stock, no mods, no loose rules or art files.
- Games: the 276 archived games of M-0001 and M-0003 with SREF or V3 on either side, 23 pairings
  of 12 games each (the row type on side A with seeds 0 to 5, and on side B with seeds 0 to 5).
  Each is replayed from its archived setup and seed. Map, players and game settings (M-0001's
  *Map* and *Players* lines), the budget, stated cells, facings and the *hunt* mission (its
  *Groups* line) and the end rule are those of M-0001 (the stock map `Arena.mmx`, start waypoint
  2, call its cell W). M-0001's *Harness* line is not taken: this replay's harness is given below.
- **The five gameplay changes.** The archived games of M-0001 to M-0003 ran with the five
  gameplay changes of the harness that their Setup lists: a siege rule for long-range artillery,
  a change to Magnetron carrying, changes to the computer houses' production, extra defences,
  and an expansion and factory queue. This record's replay has all five off and is the stock
  reference. The house director is off for every house, in the archive
  and in the replay. The houses' own skirmish AI runs as in a normal game but has nothing to
  build with.
- Placement and map content: this record does not observe placement. M-0001's Setup gives the
  harness's account (units created on their stated cells before tick 0, and a unit that its stated
  cell does not take placed on a neighbouring cell). In the archived games' logs each such unit was
  placed on a neighbouring cell, which by M-0006 can already hold a unit. This replay's tick-0 cells
  were not compared with those logs. M-0006's Result gives the bunkers' cells and the logs' placed
  cells of some of the units that stood off their stated cells (those stated on the cliff cells of
  Q-1502). The other moves are in the archived logs and are not given.
- What is not stock, in this replay:
  - no menus: the skirmish setup comes from a settings file
  - units are created on their stated cells before tick 0 (the harness's account, as M-0001 lists
    it)
  - the map's four bunkers, owned by a neutral house, are in play from tick 0, as M-0001 lists it
  - the computer players' starting construction vehicles are removed
  - the two computer players are not allied with each other (a stock skirmish would ally every
    computer player when nobody is on a team)
  - the idle observer house's objects cannot be targeted or damaged
  - the run ends by M-0001's rule: at the first sample (every 304 ticks) at which only one side
    still owns anything, or at the first sample at or after tick 6000
  - a periodic sampler reads state only
- What is not stock, in the archived games: the same, and the five gameplay changes above.

## Procedure

1. Replay each of the 276 games once with the five changes off.
2. At every sample (every 304 ticks), record each house's unit count, the value of its units and
   whether it is defeated, and at the end the result.
3. Compare each replay with its archived game, sample by sample, on those columns and on the
   result. "Identical" and "differs" below mean those sampled columns only, not every tick.
4. Count a game as a change of its recorded row when its replay differs from its archived game
   at the end sample, in any house's unit count, units' value or defeated state, or in the result
   (winner and end frame). A game that differs only in samples before the end is not counted.
5. For each game counted in step 4, record the archived and the stock winner, end frame and each
   side's share of its starting army value left at the end, as the harness computes it.
   The shares are kept local. Each house's unit count and defeated state at the end
   are compared in step 3 but not supplied.

## Result

**Games.**

| Quantity | n | Value |
|---|---|---|
| Games identical in every sample and in the result | 276 | below the table |
| Games that differ | 276 | below the table |
| Of those, counted by the comparison as a change of the recorded row | 55 | below the table |
| Of those 48, with a change in the winner, end frame or a side's unit count | 48 | below the table |
| Of those 46, games whose winner changes | 46 | below the table |
| Of the 48, with winner, end frame and unit counts equal and a change in the shares only | 48 | below the table |
| Games that differ only in samples before the end | 55 | below the table |

- The 15 pairings in which all 12 games are identical: DISK against SREF, FV against V3, LTNK
  against SREF, LTNK against V3, MGTK against V3, MIND against V3, MTNK against V3, ROBO against V3,
  SREF against APOC, SREF against HTNK, SREF against TTNK, SREF against V3, TELE against SREF,
  TNKD against V3 and YTNK against V3.
- In the 33 games of the 46 whose winner stays, the end frame moves by one or more samples, or a
  side's unit count at the end changes, or both.
- For the 2 games whose winner, end frame and unit counts are equal, the change is in a share. In
  CAOS against SREF, B5, SREF's share is lower in stock than archived, and in TELE against V3, A2,
  V3's share is lower in stock than archived. So a side's share at the end does not follow from
  its unit count at the end.
- **What a share measures.** The oracle computes a side's share as its army value at the end over
  its army value at the start, both from the harness's samples, and reports that this reproduces the
  shares of all 1860 archived games of M-0001 to M-0003. What the harness's army value counts, and
  so why one unit count can give two shares, is not recorded.

Relations for the table above:

- *Games identical in every sample and in the result*: 221.
- *Games that differ*: 55, in 8 pairings.
- *Of those, counted by the comparison as a change of the recorded row*: 48.
- *Of those 48, with a change in the winner, end frame or a side's unit count*: 46.
- *Of those 46, games whose winner changes*: 13.
- *Of the 48, with winner, end frame and unit counts equal and a change in the shares only*: 2 (CAOS
  against SREF, B5; TELE against V3, A2).
- *Games that differ only in samples before the end*: 7 (5 BFRT against V3, 2 DISK against V3).

**Pairings with changed rows, stock** (the row's wins with the 48 stock rows in place of the
archived ones; columns as in M-0001's Result, except that its overall row win rate is not given
and the archived result is added; the 2 games in which only the shares change leave every
win as it was).

| Record | Row | Column | n | Row wins | Column wins | None | Row win rate as A / as B | Archived (as A / as B) |
|---|---|---|---|---|---|---|---|---|
| M-0001 | CAOS | V3 | 12 | 7 | 0 | 5 | 2/6 / 5/6 | 4 row, 1 column, 7 none (4/6 / 0/6) |
| M-0001 | YTNK | SREF | 12 | 8 | 4 | 0 | 4/6 / 4/6 | 8 row, 4 column (3/6 / 5/6) |
| M-0001 | TELE | V3 | 12 | 0 | 12 | 0 | 0/6 / 0/6 | unchanged |
| M-0001 | CAOS | SREF | 12 | 0 | 12 | 0 | 0/6 / 0/6 | unchanged |
| M-0001 | MIND | SREF | 12 | 12 | 0 | 0 | 6/6 / 6/6 | unchanged |
| M-0001 | DISK | V3 | 12 | 12 | 0 | 0 | 6/6 / 6/6 | unchanged |
| M-0003 | SREF | HTK | 12 | 0 | 12 | 0 | 0/6 / 0/6 | unchanged |
| M-0003 | BFRT | V3 | 12 | 12 | 0 | 0 | 6/6 / 6/6 | unchanged |

**The 48 stock rows.** Row side and seed name the game as in M-0001's per-game data; winner is
`row`, `col` or `none`. The last column gives the first of winner, end frame, a side's unit count
and the shares that changes at the end, in that order. The archived and stock shares of these
rows are local: per-game shares would give the size of a type's group and, through M-0001's
group-size rule, its stock cost.

| Record | Row | Column | Row side | Seed | Archived winner | Archived end frame | Stock winner | Stock end frame | What changes at the end |
|---|---|---|---|---|---|---|---|---|---|
| M-0003 | BFRT | V3 | A | 4 | row | 1216 | row | 912 | end frame |
| M-0003 | BFRT | V3 | B | 1 | row | 912 | row | 1216 | end frame |
| M-0003 | BFRT | V3 | B | 5 | row | 1216 | row | 912 | end frame |
| M-0001 | CAOS | SREF | A | 0 | col | 304 | col | 304 | unit count |
| M-0001 | CAOS | SREF | A | 1 | col | 304 | col | 304 | unit count |
| M-0001 | CAOS | SREF | A | 2 | col | 304 | col | 304 | unit count |
| M-0001 | CAOS | SREF | A | 3 | col | 304 | col | 304 | unit count |
| M-0001 | CAOS | SREF | A | 4 | col | 304 | col | 304 | unit count |
| M-0001 | CAOS | SREF | A | 5 | col | 304 | col | 304 | unit count |
| M-0001 | CAOS | SREF | B | 4 | col | 304 | col | 304 | unit count |
| M-0001 | CAOS | SREF | B | 5 | col | 304 | col | 304 | shares only |
| M-0001 | CAOS | V3 | A | 0 | row | 2128 | row | 2432 | end frame |
| M-0001 | CAOS | V3 | A | 1 | row | 2432 | none | 6080 | winner |
| M-0001 | CAOS | V3 | A | 2 | row | 2432 | none | 6080 | winner |
| M-0001 | CAOS | V3 | A | 3 | row | 2128 | none | 6080 | winner |
| M-0001 | CAOS | V3 | A | 4 | none | 6080 | none | 6080 | unit count |
| M-0001 | CAOS | V3 | A | 5 | none | 6080 | row | 2736 | winner |
| M-0001 | CAOS | V3 | B | 0 | none | 6080 | row | 2432 | winner |
| M-0001 | CAOS | V3 | B | 1 | none | 6080 | row | 3040 | winner |
| M-0001 | CAOS | V3 | B | 2 | col | 3648 | row | 3040 | winner |
| M-0001 | CAOS | V3 | B | 3 | none | 6080 | none | 6080 | unit count |
| M-0001 | CAOS | V3 | B | 4 | none | 6080 | row | 2736 | winner |
| M-0001 | CAOS | V3 | B | 5 | none | 6080 | row | 2736 | winner |
| M-0001 | DISK | V3 | B | 1 | row | 2128 | row | 2432 | end frame |
| M-0001 | MIND | SREF | A | 0 | row | 304 | row | 304 | unit count |
| M-0001 | MIND | SREF | A | 3 | row | 304 | row | 304 | unit count |
| M-0003 | SREF | HTK | A | 1 | col | 304 | col | 304 | unit count |
| M-0003 | SREF | HTK | B | 0 | col | 304 | col | 304 | unit count |
| M-0003 | SREF | HTK | B | 2 | col | 304 | col | 304 | unit count |
| M-0003 | SREF | HTK | B | 5 | col | 304 | col | 304 | unit count |
| M-0001 | TELE | V3 | A | 0 | col | 2128 | col | 1824 | end frame |
| M-0001 | TELE | V3 | A | 1 | col | 1824 | col | 2432 | end frame |
| M-0001 | TELE | V3 | A | 2 | col | 2128 | col | 2128 | shares only |
| M-0001 | TELE | V3 | A | 3 | col | 2128 | col | 2432 | end frame |
| M-0001 | TELE | V3 | A | 4 | col | 2128 | col | 1520 | end frame |
| M-0001 | TELE | V3 | A | 5 | col | 1520 | col | 1824 | end frame |
| M-0001 | TELE | V3 | B | 0 | col | 1216 | col | 1216 | unit count |
| M-0001 | TELE | V3 | B | 1 | col | 1520 | col | 2128 | end frame |
| M-0001 | TELE | V3 | B | 2 | col | 2736 | col | 1216 | end frame |
| M-0001 | TELE | V3 | B | 3 | col | 2128 | col | 1824 | end frame |
| M-0001 | YTNK | SREF | A | 0 | row | 304 | col | 304 | winner |
| M-0001 | YTNK | SREF | A | 1 | row | 304 | row | 304 | unit count |
| M-0001 | YTNK | SREF | A | 2 | col | 304 | col | 304 | unit count |
| M-0001 | YTNK | SREF | A | 3 | col | 304 | row | 304 | winner |
| M-0001 | YTNK | SREF | A | 4 | row | 304 | row | 304 | unit count |
| M-0001 | YTNK | SREF | A | 5 | col | 304 | row | 304 | winner |
| M-0001 | YTNK | SREF | B | 3 | row | 304 | row | 304 | unit count |
| M-0001 | YTNK | SREF | B | 4 | row | 304 | col | 304 | winner |

**What replaces what.**

- Each of the 48 rows above replaces the row of M-0001's or M-0003's per-game data with the same
  row, column, row side and seed. Its stock winner and end frame replace the archived ones, and
  its stock shares are local; for the 2 games whose shares alone change, the winner and end
  frame are the same.
- In MIND against SREF, A0 and A3, MIND's share is above 1 in both games, archived and stock. A
  share above 1 fits the harness's counting units taken over by mind control for the house that owns
  them at the sample, a reading this record does not check.
- The other 228 rows of the 23 pairings stand as stock rows. For the 221 identical games the
  replay reproduces every sample and the result; for the 7 that differ only before the end it
  reproduces their end-sample columns and result.
- **Not replayed here: the 1584 archived games without SREF or V3.** M-0017 replays them with
  the five changes off and gives their stock rows. M-0006 replays 20 of them tick by tick, and in
  those every unit state, damage event, fire event and the random state are identical with and
  without the changes.

**Tolerance for tests.** As M-0001's, per pairing (n = 12), with the 48 stock rows in place of the
archived ones. No per-game share test runs against committed data.

## Evidence

- 276 games, each replayed once; the replays stay with the operator, and no recording or log
  hash was supplied with the results.
- In all 48 changed rows the archived winner and end frame equal M-0001's or M-0003's per-game
  row. The archived shares supplied with the results, for the 48 changed rows, equal those of
  the archived games' harness output. The oracle reports that its way of computing a share
  reproduces the per-game shares of all 1860 archived games; those shares were not supplied with
  the results. In every row above, archived and stock, a side's share is 0 exactly where the
  other side is the winner.
- The oracle reports the 12 YTNK against SREF replays here identical to M-0006's 12 stock
  replays of those games, without saying which columns it compared. Their results agree: the row
  wins 8 of 12 in both.
- No replay failed.

## Notes

- **M-0001 to M-0003.** This record does not edit them. Their SREF and V3 rows are replaced by
  this record's where it gives one, and their other rows are not replayed here (M-0017 replays
  them).
- **Which change caused which row is not recorded.** All five changes were off together.
- **Size of the change.** CAOS against V3 changes most: in stock CAOS wins 7 of 12 games, against
  4 of 12 archived. 15 of the 23 pairings are unaffected.
- **M-0006 counts differently.** M-0006 compares every tick of its replayed games and counts more
  YTNK against SREF games as differing than this record, which compares samples and counts 8.
