---
id: M-0024
title: The scatter from damage for hunting vehicles of computer-controlled houses (S-0215 B6 part f)
topic: simulation
specs: [S-0215]
requests: [Q-2782]
verdict: confirms
value-visibility: local
measured-on: 2026-11-07
method: harness
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-11-07
---

# M-0024: The scatter from damage for hunting vehicles of computer-controlled houses

This record plays M-0021 part B's two-group fight of test vehicles, both houses computer-controlled,
with one more test section in the rules file: *hunt*'s scatter and retaliate properties. With the
scatter property on and the retaliate property off, every vehicle in *hunt* that stood with neither
a target nor a destination and survived a hit, whether the hit came from an enemy or named no
attacker, held either a destination in one of the 8 cells next to its own and no target at the end
of the hit's tick, or a target. No free vehicle in *hunt* that was not hit gained a destination
in a neighbouring cell. With the scatter property off, no such vehicle gained a destination after a hit. This is what
S-0215 claim 19 case 2 gives for an actor whose current behaviour has the scatter property and lacks
the retaliate property, and whose owner is computer-controlled, and what its test of the property
before the request is sent gives with the property off.

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
5. Record the rows in which a vehicle is in a behaviour other than *hunt*. A vehicle in another
   behaviour at the end of *t* − 1 is not judged at *t*: that behaviour's properties come from stock
   data in these files.

## Result

| Quantity | Unit | n | Value | Spread | Tolerance for tests |
|---|---|---|---|---|---|
| Part f: judged hits classed *neighbouring destination* or *target* | hits | 6 runs, every judged hit (count local) | all | none | exact |
| Part f: judged hits classed *far destination* or *free* | hits | same | 0 | none | exact |
| Part f: judged hits classed *neighbouring destination* | hits | same | most of them (count local); every judged hit that names no attacker | | exact for "every hit naming no attacker" |
| Part f: judged hits naming no attacker | hits | same | at least one in each of 2 runs (counts local) | | not for tests |
| Part fc: judged hits classed *neighbouring destination* or *far destination* | hits | 2 runs, every judged hit (count local) | 0 | none | exact |
| Part fc: judged hits classed *free* or *target* | hits | same | all | none | exact |
| Parts f and fc: judged hits whose vehicle's position changed from the end of *t* − 2 to the end of *t* − 1 | hits | 8 runs, every judged hit | 0 | none | exact |
| Parts f and fc: un-hit free vehicles (step 4) classed *neighbouring destination* | vehicle-ticks | 8 runs, every such row (count local) | 0 | none | exact |
| Parts f and fc: un-hit free vehicles classed *far destination* | vehicle-ticks | same | 1 in one part f run and 1 in one part fc run | | not for tests |

Judged hits occur on both sides' vehicles in part f. In 5 of the 8 runs, some vehicles
are in another behaviour in some rows: *area guard* in 4 runs (G vehicles), and in one part f run one
F vehicle is in *attack* and then in *area guard*. In the 8 runs one free vehicle not in *hunt* was
hit, that F vehicle while in *attack*, and it is not judged. In part f, a judged hit by an enemy is a hit
the vehicle may not retaliate against: *hunt* lacks the retaliate property in the test file (S-0215
claim 18). Each of the two *far destination* rows is a vehicle that was not hit gaining, with no
target, a destination several cells away. That is not a cell next to its own. The recording does
not say what gave it.

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
- **Hits classed *target*.** A judged hit can leave the vehicle holding a target at the end of the
  hit's tick. The recording does not show whether a request was sent in that tick. Such hits occur
  in both files.
- **Kept local: counts and ticks.** The number of judged hits, and how many of them name no attacker,
  grow with the number of shots, which would bound the houses' rate-of-fire factor (M-0021, *Notes*).
  So they are local. How many ticks a vehicle took to reach the destination cell would bound its
  country's speed factor (M-0021, *Notes*), so those ticks are not given. Neither are the runs'
  outcomes and deaths: this record states only the rows above.
- **Reproducing it.** A comparison builds M-0021's test types as M-0021's *Reproducing it* note
  gives, with this record's *hunt* section, and judges the rows of the Procedure. The rows above
  depend on the fight only through which vehicles stand free when hit.
