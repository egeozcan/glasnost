---
id: M-0024
title: The scatter from damage for hunting vehicles of computer-controlled houses (S-0215 B6 part f)
topic: simulation
specs: [S-0215, S-0309]
requests: [Q-2782, Q-2988]
verdict: confirms
value-visibility: committed
measured-on: 2026-11-07
method: harness
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-11-12
---

# M-0024: The scatter from damage for hunting vehicles of computer-controlled houses

This record plays M-0021 part B's two-group fight of test vehicles, both houses computer-controlled,
with one more test section in the rules file: *hunt*'s scatter and retaliate properties. With the
scatter property on and the retaliate property off, every vehicle in *hunt* that stood with neither
a target nor a destination and survived a hit, whether the hit came from an enemy or named no
attacker, held either a destination in one of the 8 cells next to its own and no target at the end
of the hit's tick, or a target. With the scatter property off, no such vehicle gained a destination
after a hit. This is what S-0215 claim 19 case 2 gives for an actor whose current behaviour has the
scatter property and lacks the retaliate property, and whose owner is computer-controlled, and what
its test of the property before the request is sent gives with the property off.

In these runs, a free vehicle in *hunt* that was not hit gained a destination with no target only in
a tick in which another vehicle's hit destroyed one of its enemy vehicles: twice in the 8 runs, each
time a destination lying between its cell and the destroyed vehicle's cell (step 5). A vehicle in
*hunt* that held an enemy vehicle as its target and a destination at the end of a tick, was not hit
in the next tick, and whose target another vehicle's hit destroyed in that next tick, held no target
at the end of that tick in every case, and the same destination in every case but one. The runs
observe this only for a destination held before the tick of the kill. They do not show a vehicle
taking a target and a destination and losing the target within one tick, and they do not show what
cleared the one exception's destination.

## Setup

- Game build: stock Yuri's Revenge 1.001, executable hash verified; run under the oracle harness
  (PROVENANCE §7).
- Harness, map, game settings, houses, creation order, groups, test types, and what is not stock:
  as M-0021's *Setup* for part B (strength 400, fragment count 3), with one change to the test rules
  file below. Every vehicle starts in *hunt*, and no order is given during a run. Both houses are
  computer-controlled, at the hard computer level, and not allied with each other.
- Test rules files: M-0021 part B's file with a *hunt* section that sets two of the behaviour's
  properties (S-0215 Terms), its other fields left as stock:
  - *part f*: the scatter property on and the retaliate property off,
  - *part fc* (the control): the scatter property off and the retaliate property off.
- Runs:
  - part f: seeds 1000, 1001 and 1002, F on side A and on side B (M-0021's sides): 6 runs,
  - part fc: seed 1000, F on side A and on side B: 2 runs.

  In every run one side had no vehicle left before the run ended, and the tick limit of 3000 was
  not reached. The rows are read up to the run's end.

## Procedure

The recorder logs what M-0021's *Procedure* gives. It also logs, for every vehicle each tick, its
current behaviour, its target (or none) and its destination cell (or none). A vehicle's cell is its
position in leptons divided by 256, truncated, per axis, as in M-0021. The per-tick rows stamped *t*
are read after tick *t* has run, so they show the state at the end of tick *t*. By M-0021's
*Reading points*, they show a vehicle's health after the hits recorded at *t*.

1. A vehicle is *free* at the end of a tick when its row stamped with that tick shows neither a
   target nor a destination.
2. A *judged hit* is a hit at tick *t* on a vehicle that is in *hunt* and free at the end of
   *t* − 1, and on the map with health left at the end of *t*. Several hits on one vehicle in one tick count as one
   judged hit. The hit *names no attacker* when every hit on that vehicle at *t* names none.
3. For each judged hit, read the vehicle's row stamped *t*. Classify it as *target* (it holds a
   target), *neighbouring destination* (no target, and a destination at Chebyshev distance 1 from
   its cell at the end of *t* − 1), *far destination* (no target, and a destination farther away)
   or *free*. Also compare the vehicle's position at the end of *t* − 2 with its position at the
   end of *t* − 1.
4. For each vehicle that is in *hunt* and free at the end of a tick *t* − 1, is not hit at *t*, and
   is on the map with health left at the end of *t*, classify its row stamped *t* in the same way.
   For a row classed *target*, also record whether it holds a destination at Chebyshev distance 1
   from the vehicle's cell at the end of *t* − 1.
5. An enemy vehicle is *destroyed at t by another vehicle's hit* when the first hit recorded at *t*
   that leaves it with no health names as attacker a vehicle other than the vehicle being
   classified. For each row of step 4, record whether an enemy vehicle of that vehicle is destroyed
   at *t* by another vehicle's hit. For each *far destination* row of step 4 in such a tick, record
   whether the destination cell lies *between* the vehicle's cell and the destroyed vehicle's cell,
   both at the end of *t* − 1: on each axis, the destination's coordinate is at least the smaller
   and at most the larger of the two cells' coordinates.
6. *Held target destroyed:* for each vehicle that is in *hunt* at the end of *t* − 1 and holds there
   a target that is an enemy vehicle and a destination, is not hit at *t*, is on the map with health
   left at the end of *t*, and whose target is destroyed at *t* by another vehicle's hit, classify
   its row stamped *t*: whether it holds a target, and whether its destination is the same cell as
   at the end of *t* − 1, another cell, or none. Do the same for such vehicles that hold the target
   and no destination at the end of *t* − 1. For a row with neither target nor destination at *t*,
   also record whether the vehicle's cell at the end of *t* is the destination cell it held at the
   end of *t* − 1, and whether its position, target or destination changes in any later row up to
   the run's end.
7. Record the rows in which a vehicle is in a behaviour other than *hunt*. A vehicle in another
   behaviour at the end of *t* − 1 is not judged at *t*: that behaviour's properties come from stock
   data in these files.

## Result

| Quantity | Unit | n | Value | Spread | Tolerance for tests |
|---|---|---|---|---|---|
| Part f: judged hits classed *neighbouring destination* or *target* | hits | 6 runs, every judged hit (count not reported) | all | none | exact |
| Part f: judged hits classed *far destination* or *free* | hits | same | 0 | none | exact |
| Part f: judged hits classed *neighbouring destination* | hits | same | most of them (count not reported); every judged hit that names no attacker | | exact for "every hit naming no attacker" |
| Part f: judged hits naming no attacker | hits | same | at least one in each of 2 runs (counts not reported) | | not for tests |
| Part fc: judged hits classed *neighbouring destination* or *far destination* | hits | 2 runs, every judged hit (count not reported) | 0 | none | exact |
| Part fc: judged hits classed *free* or *target* | hits | same | all | none | exact |
| Parts f and fc: judged hits whose vehicle's position changed from the end of *t* − 2 to the end of *t* − 1 | hits | 8 runs, every judged hit | 0 | none | exact |
| Parts f and fc: un-hit free vehicles (step 4) classed *neighbouring destination* or *far destination*, at a tick in which no enemy vehicle of theirs is destroyed by another vehicle's hit (step 5) | vehicle-ticks | 8 runs, every such row (count not reported) | 0 | | not for tests: a count for these runs, not a rule (see *Notes*) |
| Parts f and fc: un-hit free vehicles classed *neighbouring destination*, at a tick in which an enemy vehicle of theirs is destroyed by another vehicle's hit | vehicle-ticks | 8 runs, every such row (count not reported) | 0 | | not for tests: a count for these runs, not a rule (see *Notes*) |
| Parts f and fc: un-hit free vehicles classed *far destination*, at a tick in which an enemy vehicle of theirs is destroyed by another vehicle's hit | vehicle-ticks | same | 1 in one part f run and 1 in one part fc run; in each, the destination cell lies between the vehicle's cell and the destroyed vehicle's cell (step 5) | | not for tests |
| Parts f and fc: un-hit free vehicles classed *target* whose row also holds a destination in a neighbouring cell (step 4) | vehicle-ticks | 8 runs, every such row (count not reported) | at least one in each of 7 runs (counts not reported) | | not for tests |
| Parts f and fc: held target destroyed (step 6), target and destination held at *t* − 1: rows at *t* with no target and the same destination | vehicle-ticks | 8 runs, every such row (count not reported) | all but one | | not for tests: a count for these runs, not a rule (see *Notes*) |
| Parts f and fc: the same rows at *t* with a target, or with a different destination | vehicle-ticks | same | 0 | | not for tests: a count for these runs, not a rule (see *Notes*) |
| Parts f and fc: the same rows at *t* with no target and no destination | vehicle-ticks | same | 1, in one part f run | | not for tests |
| Parts f and fc: held target destroyed (step 6), target and no destination held at *t* − 1: rows at *t* with a destination | vehicle-ticks | 8 runs, every such row (count not reported) | 0 | | not for tests: a count for these runs, not a rule (see *Notes*) |

Judged hits occur on both sides' vehicles in part f. In 5 of the 8 runs, some vehicles are in
another behaviour in some rows: *area guard* in 4 runs (G vehicles), and in one part f run one F
vehicle is in *attack* and then in *area guard*. In the 8 runs one free vehicle not in *hunt* was
hit, that F vehicle while in *attack*, and it is not judged. In part f, a judged hit by an enemy is
a hit the vehicle may not retaliate against: *hunt* lacks the retaliate property in the test file
(S-0215 claim 18). Each of the two *far destination* rows is a vehicle that was not hit gaining,
with no target, a destination several cells away, in a tick in which another vehicle's hit destroys
one of its enemy vehicles, and the destination lies between the vehicle's cell and that enemy
vehicle's cell (step 5). The rows show the state at the end of the tick only. They do not show what
gave the destination, whether the vehicle held the destroyed vehicle as its target during the tick,
or the order of turns within it.

The one held-target row with neither target nor destination at *t* (step 6) is a vehicle whose cell
at the end of *t* is not the destination cell it held. Its rows continue to the run's end, on the
map with health left. In every row from the end of *t* to the run's end its position is the same,
and it holds neither a target nor a destination. The recording does not show the order of turns
within a tick, so it does not show what cleared the destination.

## Evidence

- 8 recordings, played on 2026-11-07. The recordings stay local, and no log hash is given.
- No run is void. No run crashed or stopped before its end.

## Notes

- **What the verdict covers.** S-0215 claim 19, case 2, for driving vehicles of computer-controlled
  houses in a behaviour with the scatter property and without the retaliate property:
  - a hit by an enemy, or one naming no attacker, sends the request to a vehicle that has neither
    a target nor a destination and survives the hit,
  - the vehicle then holds a destination in a cell next to its own, at the end of the hit's tick,
  - with the scatter property off, no such destination follows a hit (the property is tested
    before the request is sent).
- **Not tested here:**
  - which of the vehicle's acceptance conditions (S-0207 claim 16) held,
  - a vehicle moving when hit: every judged vehicle stood at the same position in the two ticks
    before the hit, and the arena has no tether link,
  - which cell next to its own it takes, and why,
  - case 1,
  - hits by the vehicle's own house or by an allied house (the two houses are enemies, and no hit
    in any run comes from the victim's own house),
  - human-controlled owners and the player-scatter setting.
- **What the verdict covers for S-0309.** S-0309 claim 27 drops a destroyed object as every holder's
  target at the killing hit. Apart from a revert, it leaves the holder's destination as it is, and
  what the holder's later turns do with it is S-0215's. The rows of step 6 are consistent with this
  for vehicles in *hunt* of computer-controlled houses. Each vehicle there held an enemy vehicle as
  its target and a destination at the end of a tick, was not hit in the next tick, and had its
  target destroyed by another vehicle's hit in that tick. At the end of that tick each held no
  target, and each held the same destination, apart from one. A holder with no destination gained
  none.
  - The rows do not show what cleared the one exception's destination: the drop, a revert, or a
    later turn of the vehicle in that tick.
  - The rows of step 6 are counts for these runs, not rules. In these runs no holder had a target,
    a different destination or a newly gained destination at the end of the tick of the kill. Claim
    27 allows a revert, and a search in the holder's own turn, in that tick, and either could give
    one.
  - These rows are not a verdict on claim 27's reverts or timers, or on a destination taken in the
    same tick as the kill.
- **What the rows do not settle (Q-2988).** The record answers Q-2988 only in part. It adds
  observations of these runs, the step 4 rows split by the ticks of step 5 and the rows of step 6,
  all of them counts for these runs, and it settles none of the request's three questions.
  - The step 4 row at ticks with no kill (step 5) is a count for these runs too. Step 5 does not
    count a kill by a hit that names no attacker, or by the vehicle being classified, and S-0309
    claim 28 drops a target on other events. Any of these could leave a free vehicle with no target
    and a destination.
  - A target and a destination taken within the tick of the kill and the target then lost
    (question 1's case) is not observed. The two *far destination* rows are in such ticks, but the
    rows do not show the sequence within the tick.
  - What cleared the one exception's destination (question 2's subject) is not shown.
  - Question 3 (whether the approach search can give a cell next to the searcher's own) is not
    settled. Two rows bear on it, each only in part:
    - The row of un-hit free vehicles classed *neighbouring destination* at a tick in which an
      enemy vehicle of theirs is destroyed by another vehicle's hit counts the end state of
      Q-2988's case: no target and a destination in a neighbouring cell. That count is 0. It is a
      count for these 8 runs: they give no such row and do not show that one cannot occur.
    - The row of un-hit free vehicles classed *target* whose row also holds a destination in a
      neighbouring cell shows that a vehicle in *hunt* can gain, in one tick, a target together
      with a destination in a cell next to its own. The rows do not show what gave that
      destination, or whether it is a destination for approaching the held target. So this row
      does not show what the approach search gives, and it does not conflict with the zero above,
      which counts rows with no target.
- **Hits classed *target*.** A judged hit can leave the vehicle holding a target at the end of the
  hit's tick. The recording does not show whether a request was sent in that tick. Such hits occur
  in both files.
- **Not reported: counts and ticks.** The number of judged hits, and how many of them name no
  attacker, grow with the number of shots, which would bound the houses' rate-of-fire factor
  (M-0021, *Notes*). So they are not reported. How many ticks a vehicle took to reach the
  destination cell would bound its country's speed factor (M-0021, *Notes*), so those ticks are not
  given. Neither are the runs' outcomes and deaths: this record states only the rows above.
- **Not reported: held-target counts and distances.** The number of held-target rows (step 6)
  grows with the number of kills, and the distance from a vehicle's cell to its destination would
  bound a weapon's range, so neither is given.
- **Reproducing it.** A comparison builds M-0021's test types as M-0021's *Reproducing it* note
  gives, with this record's *hunt* section, and judges the rows of the Procedure. The rows of steps
  2 and 3 depend on the fight only through which vehicles stand free when hit. The rows of steps 4
  to 6 depend on it further:
  - in which ticks another vehicle's hit destroys an enemy vehicle,
  - which vehicles hold a target and a destination at the end of the tick before,
  - what those vehicles hold in later rows,
  - for the rows of step 6, whether a holder gets a target or a destination again in the tick of
    the kill: S-0309 claim 27 allows a revert, and a search in the holder's own turn, in that tick,
    and the recording does not show the order of turns within it.

  A comparison whose fight differs gives its own counts for those rows. Only the rows marked exact
  are for tests.
