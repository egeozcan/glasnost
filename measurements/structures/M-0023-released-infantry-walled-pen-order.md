---
id: M-0023
title: A released infantry unit given a move order into a walled pen before its first turn (S-0417 U10 part g run 1)
topic: structures
specs: [S-0417, S-0212]
requests: [Q-1980]
verdict: confirms
value-visibility: local
measured-on: 2026-11-05
method: harness
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-11-06
---

# M-0023: A released infantry unit given a move order into a walled pen before its first turn

This record runs S-0417 scenario U10 part g. An infantry unit is released with no rally point onto a
cell that holds its factory K. In the tick of its release, after the release, the harness gives it
a move order to the centre of a pen of clear ground closed by wall sections. The pen is in another
reach region for the unit's movement zone. The unit's first turn comes in the next tick.

In run 1 (stock data) the unit took no step in its first two turns. In its first turn the order's *move* became current
and the unit's destination was cleared. In the next turn *guard* was current, the update had the
draws of the baseline's request turn, and the unit got a destination on a cell that does not hold
K, which the record reads as part b's scatter request. Its first step came 1 tick after that
turn, as in the baseline. It then
walked to that destination and rested there in *guard*. Run 2, with a test rules file for the
path-retry delay, gave the same recording as run 1 and does not count (*Notes*). The two controls,
whose order takes effect after the unit's first turn, do not test the part.

## Setup

- Game build: stock Yuri's Revenge 1.001, executable hash verified; run under the oracle harness
  (PROVENANCE §7).
- Harness: the oracle's unit arena. Its house director is off for every house, and so are all five
  gameplay changes of the harness that M-0001's Setup lists: a siege rule for long-range artillery,
  a change to Magnetron carrying, changes to the computer houses' production, extra defences, and an
  expansion and factory queue.
- Data: stock in g1-now (run 1), g1-after and g0-none. g2-now (run 2) and g2-after add one test
  rules file, a loose rules file put in the run's game folder. It is the stock rules file with the path-retry
  delay field set to 0.1 game minutes, which gives *R* = trunc(0.1 × 900) = 90 ticks (S-0212
  claim 21). No art file is changed.
- Map: a test map made from the stock map `Arena.mmx`, with 16 wall sections added on clear ground
  as a 5 × 5 ring. The ring closes a 3 × 3 pen of clear ground. No cell of the ring or the pen held
  an overlay, a terrain object or a building in the stock map, and nothing else changes.
  - The pen's centre is the order's destination. The wall sections are of one stock wall type
    (its name kept `local`) that has the wall flag. By S-0211 claim 11's rule 2 a ring cell is
    therefore *crushable* if the type is crushable and *wall-like* otherwise, and the infantry
    movement zone may cross neither (S-0211 claim 12). So for the unit the pen is a reach region
    of its own, with no bridge to it. The record does not read the cells' categories or the reach
    regions: this follows from the set-up and those two claims.
  - The ring's nearest cell is 3 cells from K's foundation.
  - The cells of the ring and the pen, and K's place, are kept `local`.
- Game: a skirmish at the fastest game speed, seed 1000 in every run. Tech level 10; superweapons,
  crates and the short game off; bases on; the map revealed.
- Houses:
  - the human player, who is not an observer here: its objects can be targeted and damaged. It owns K, a stock infantry
    factory, and one power plant more than 3 cells from K. Before tick 0 its credits are set to 10000.
  - two computer houses, one of each side, each owning one power plant far from K and no
    construction vehicle. They have nothing to build with and take no part.
  - Nothing else stands within 3 cells of K, and K has no rally point.
- The unit: one infantry unit of a stock type that has no fraidycat flag, as U10 part b asks,
  and the infantry movement zone. Its
  type is kept `local`. The human orders it once, by scripted input, at a fixed tick before its
  release. The release tick is kept `local`.
- What is not stock, in every run of this record:
  - no menus: the skirmish setup comes from a settings file.
  - every house's starting construction vehicle is removed, as are the human's other starting
    objects and every object of a house that is not a player.
  - the two computer houses are not allied with each other.
  - the buildings are created on their stated cells before tick 0.
  - the credits and the production order are scripted input.
  - the test map.
  - **the order to the released unit (run 1, run 2 and the controls).** The harness gives the unit
    an ordinary move order to the pen's centre directly, not through a player's input. In run 1
    and run 2 the order is given in the tick of the release, after the release and outside that
    tick's update, so it takes effect before the unit's first turn. That turn is in the next tick,
    the first tick after the release (S-0215 claim 12). In the controls the same order is given
    two ticks after the release, before that tick's update, so after the unit's first turn.
  - the test rules file of g2-now and g2-after.
  - a recorder reads state only (below).
  - the run ends at a fixed tick, more than 300 ticks after the release.
- The recorder gives, until the run ends:
  - for every object of the players' houses, after each tick's update: its identifier, type,
    owning house, position in leptons, health, current and queued behaviour, and destination.
  - for each cell of a rectangle that holds K's foundation, its neighbours and the ring: its
    overlay and the objects in it.
  - every damage event.
  - every scenario-stream draw: its tick, whether it falls inside a tick's update, its kind and
    range, its result, and its source, the place in the game that takes it, which stays in the
    local recordings and is not in the values file.
  - the harness's log of the release and of its order: the tick, whether inside the update, the
    unit, its current and queued behaviour, the destination, and whether each succeeded.

| Run | Data | Order to the released unit | Role |
|---|---|---|---|
| g1-now | stock | in the release's tick, after the release | run 1: tests the part |
| g2-now | test rules file, *R* = 90 | as g1-now | run 2: tests the part if it counts |
| g1-after | stock | two ticks after the release, before that tick's update | control |
| g2-after | test rules file, *R* = 90 | as g1-after | control |
| g0-none | stock | none | baseline: part b on this map |

## Procedure

1. Play the five runs. Let *t* be the release tick in the harness's log and *U* the released unit.
2. Check each run against these void rules, where they apply to its arm:
   - all 16 ring cells hold a wall section and the pen's centre is clear.
   - one release and, in the ordered runs, one order, both successful, with the pen's centre as
     the order's destination.
   - *U* is on a cell that holds K at the end of tick *t* + 1.
   - run 1 and run 2 only: *U* does not leave K's cells before *F* (below).
   - no damage event against *U* or K, and no other object in the pen or within 3 cells of K, from
     *t* to *t* + 300.
   - the run records to *t* + 300.
   - run 2 only, as U10 part g asks: S-0212 scenario P25(a)'s passed-gate search, run with the same
     test rules file in the same game folder, reads the 90 ticks.
3. For *U*, per tick from *t* to *t* + 300, read its current and queued behaviour, destination and
   position. Its cell is position ÷ 256, truncated. Read whether that cell holds K from the cell's
   objects. Read the scenario-stream draws inside each update, apart from those that every run,
   the baseline included, takes in the same tick.
4. Derive:
   - *F*, the first tick after the order whose row shows no destination.
   - *G*, the first tick after *F* in which the draws of the baseline's request turn appear.
   - the destination at *G*.
   - *W*, the first tick whose position differs from the release position.

   Draws are matched to the baseline's by their source, which stays in the local recordings. The recording does
   not tie a draw to an object.
5. In the baseline, take its request turn, the first tick after *t* at whose end *U* holds a
   destination, with that tick's draws, and the tick of the first step. In the baseline and the
   controls, also take the arrival tick: the first tick from which *U*'s position does not change
   again within the record.

## Result

| Quantity | g1-now (run 1) | g2-now (run 2) | g1-after, g2-after (controls) | g0-none (baseline) |
|---|---|---|---|---|
| Void rules of step 2 that apply to the arm | pass | pass, its last rule not checked | pass | pass |
| Order takes effect | tick *t*, after the release | as run 1 | tick *t* + 2, before its update | no order |
| *U* on a cell that holds K at *t* + 1 | yes | yes | yes | yes |
| At *t* + 1: *guard* current, the baseline's request-turn draws, a new destination | no | no | yes, as the baseline's | yes |
| *F* − *t* | 1 | 1 | n/a | n/a |
| At the end of *F* | *move* current, *guard* queued, no destination | as run 1 | n/a | n/a |
| *G* − *F* | 1 | 1 | n/a | n/a |
| At the end of *G* | *guard* current, nothing queued, a new destination | as run 1 | n/a | n/a |
| New destination on a cell that holds K | no | no | no (the baseline's) | no |
| New destination in the pen or next to it | no | no | no | no |
| Steps before *W* | none | none | none | none |
| First step after *G* (the baseline: after its request turn) | 1 tick | 1 tick | 1 tick (the baseline's) | 1 tick |
| *U* enters the pen within 300 ticks | no | no | no | no |

- **Run 1.** *U* stayed on its release position, as the harness's log gives it, until *W*. At *F* the order's *move* was
  current, *guard* was queued, and the destination was gone. At *G* *guard* was current and *U*
  held a destination on a cell that does not hold K, outside the pen and not next to it. *W* = *G*
  + 1. That destination was kept, unchanged, until *U*'s walk ended in its cell. The destination was
  then cleared, and *U* rested there in *guard* until the end of the record.
- **Draws (run 1).** Tick *G*'s update has six scenario-stream draws besides those that every run
  takes in that tick. The last three are equal in number, kind and order to the three of the
  baseline's request turn, and have the same sources. The first three have no counterpart in the
  baseline. Tick *F*'s update has one such draw, which the baseline does not have. No tick falls
  between *G* and *W*, because *W* = *G* + 1. The draws' ranges and results are kept `local`, and
  their sources stay in the local recordings. Draws after *W* are not compared in this record.
- **Run 2** gave a recording of *U* identical to run 1's: the object rows, the draws and the
  harness's log are the same, byte for byte. It does not count: step 2's last void rule
  needs a P25(a) check with the same test rules file, which has not been run. The test rules file
  was in the game folder after the run, but nothing in any arm shows that the game read the field.
- **Controls.** In tick *t* + 1, before the order, the update has the draws of the baseline's request
  turn, and *U* gets the baseline's destination. Its positions over the 300 ticks are identical to the baseline's,
  tick for tick. From *t* + 2 its destination reads the pen's centre and *move* is current. It
  arrives on the baseline's standing position (*Baseline*, below) in the baseline's arrival tick,
  and its destination still reads the pen's centre at the end of that tick. The
  destination is cleared in the next tick, with *move* still current and *guard* queued, and
  *guard* is current in the tick after that. Tick *t* + 2's update has one draw that the baseline
  does not have. Later draws
  are not compared. Their two data sets give identical object rows. As planned, they do not test the part:
  the order takes effect after the unit's first turn, in which the update had the baseline's
  request-turn draws and the unit got the baseline's destination. Its position first changes in
  the tick at whose start the order takes effect.
- **Baseline.** Part b on this map: no destination from the release itself; in tick *t* + 1 *guard*
  current, three draws in that tick's update besides those that every run takes, and a
  destination on a cell that does not hold K (read as part b's scatter request), outside the pen
  and not next to it; the first step in tick *t* + 2. Its walk is a single step: *U*'s position
  moves from the release position to a standing position in a cell next to the release cell,
  changing in every tick with no stop between, and does not change again within the 300 ticks.
  Its destination is cleared in its arrival tick.

Every tick, cell, walk duration and type is kept `local` (`measurements/local/M-0023.json`).

## Evidence

- One run per arm, all at seed 1000, played to the same fixed end tick on 2026-11-05.
- Recordings (local). SHA-256 of the object rows, per arm:
  - g1-now and g2-now: `dcab23a7f9e8f0aee07c61422f56a98686c648d1e8c3c6bf3f366f660f6c1946` (identical).
  - g1-after and g2-after: `a510c2507fb4c9fe47b0b6687e63d31902063c5385788ae2704b666b057146b8` (identical).
  - g0-none: `065608891e3b1e645f272acebfa691eb326fb092dadf64e648364f0d3454386d`.
- SHA-256 of the draw logs:
  - g1-now and g2-now: `e30ac053de2434c4db789cf3c1eb080ed2dd392b3f28861a0845a5133bd2ed09` (identical).
  - g0-none: `0ade43ad664e0670770385c4c43b22292d7ca098501c4210a74e4f7f9a3b21c0`.
- Ticks, cells and draw details: `measurements/local/M-0023.json`.

## Notes

- **What the verdict covers.** S-0417 claim 17, *No path*, the bullet for a search that fails at
  the reachability gate, and S-0212 claim 23's row for that failure, as far as they are
  observable here. A walking unit on a cell that holds K, ordered to a cell in another reach
  region, has no destination at the end of its first turn and has taken no step. The first later
  turn in which *guard* is current, on a cell that holds K, gives it a new destination on a
  cell that does not hold K, with the draws of the baseline's request turn. The record reads that
  as the accepted scatter request; it does not record a request itself. Which search the unit made in
  its first turn is not observed.
- **What it does not test:**
  - movement in progress and the remaining retry delay: the recorder has no reading for either.
  - the length of any retry delay or a wait for one: run 2 does not count, and run 1 is not compared
    here with the stock path-retry field.
  - which of the draws is the request's own. The recording does not tie a draw to an object or
    to a rule. S-0417 claim 17 and U10 part b give the request one draw, and the baseline's
    request turn has three besides those that every run takes.
  - the other parts of U10. The baseline is part b's set-up on this map, at one seed, without
    part b's second unit or its rally-point step.
- **Also recorded, outside the verdict:** the baseline's single step, and the ticks in which the
  baseline's and the controls' destinations are cleared (Result, *Baseline* and *Controls*). They are observations for
  other specs to cite. The record makes no claim about why the controls' destination is kept
  through the arrival tick.
- **Run 2 could count** once a passed-gate search with a kept destination (S-0212 scenario
  P25 a), run with the same test rules file in the same game folder, reads the 90 ticks. That
  check has not been run.
