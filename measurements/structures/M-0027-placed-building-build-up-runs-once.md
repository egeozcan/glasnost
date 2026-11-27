---
id: M-0027
title: A placed building's build-up runs once, and when its construction starts and hands over (S-0406 PL11)
topic: structures
specs: [S-0406, S-0108]
requests: [Q-3243]
verdict: contradicts
value-visibility: local
measured-on: 2026-11-18
method: harness
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-11-27
---

# M-0027: A placed building's build-up runs once, and when its construction starts and hands over (S-0406 PL11)

This record places four stock building types with the player's own place order after producing
them, among them a gate and an ore refinery. Each has a construction range with at least 2 stages
and a positive stage delay, whose (stage count − 1) × stage interval is at least 2 ticks, so the
range cannot reach its last stage on the building's first tick of logic. As a control, the record
also deploys a construction vehicle into a construction yard. In every run the build-up runs
once: in no run does the stage go back to 0 after *C* and before the handover.

Ticks are counted as S-0203 counts them. *T* is the tick in whose orders part the place order
is applied, so the placed building's first tick of logic is *T* + 1. After the logic of *T* + 1
the building has no current behaviour and *construction* queued. Its body is in the *idle* state,
at stage 0 and not advancing, and it is not construction-completed. On *T* + 2 *construction* is
current and has been processed once, the stage is 0, and its first advance comes one stage
interval later. In
every run but one the handover follows (the range's stage count − 1) × its stage interval ticks
later: *guard* is current, the body is in the *idle* state and the building is
construction-completed. In the other run, a refinery run, a stage advance after the first was
held one tick, on a tick on which the building's processing timer did not restart either, and the
handover
came one tick later (Result).

The deployed yard has *construction* current at the end of its first tick *U*, and is first
processed on *U* + 1, at stage 0. It hands over the same number of ticks after that first processing.

The refinery's free harvester appears on the handover tick. The single build-up run contradicts S-0406
revision 14 and S-0108 revision 106, which give the building a place order names two build-up runs
(Notes). The deployed control agrees with both.

The record answers Q-3243 questions 1 to 3 and the deployed half of question 4 for the recorded
stage and body state; it records nothing that is drawn (Notes). Filling buildings
and restored replacements, the rest of questions 4 and 5, are not covered.

## Setup

- Game build: stock Yuri's Revenge 1.001, executable hash verified; run under the oracle harness
  (PROVENANCE §7).
- Data: stock rules and art, no mods, no loose rules or art files. Every gameplay and AI change of
  the harness is off.
- Map: the stock multiplayer map `Arena.mmx`, as in M-0003.
- Game: a skirmish at the fastest game speed, tech level 10, with bases on, the map revealed, and
  superweapons, crates and the short game off. The human player's country is kept local. Only
  the fastest game speed was played.
- What is not stock:
  - no menus: the skirmish setup comes from a settings file.
  - the human's credits are set to a large sum at tick 0, so that production never waits for money.
  - before play the harness creates a small base for each house: the human's includes the
    construction yard that makes the placed types. It also creates the construction vehicle of
    the control.
  - production and placement are sent as the player's own events, not clicked:
    - a produce event names the building type.
    - once the item is done, a place event names the type and a top-left cell, with the content a
      click in placement mode sends.
    - the event is carried out after the logic of tick *T* and before that of *T* + 1, which is
      where S-0203 applies an order given in tick *T*.
  - a state recorder reads state only.
- Setups, 3 seeds each:
  - *A* and *B*: one of two other building types placed.
  - *gate*: a gate placed, its construction range being its gate range.
  - *refinery*: an ore refinery placed.
  - *deploy*: the construction vehicle ordered to deploy, which then becomes a construction yard.
- The types are stock types of the human player's side, kept local with that side. For each, the
  stage count of its construction range and its stage interval were read from the type in a
  rerun, so that the timings can be compared with them. The types, the counts and the intervals
  are kept in the local values file; this record gives only relations to them (Notes).

## Procedure

After each tick's logic, the recorder logs every building of the setup's type. Each row holds:
- its current and queued behaviour, the behaviour's processing status and the tick on which its
  processing timer last started.
- the body's animation state, the stage of that state's range, and whether the stage is advancing.
- whether it is construction-completed.
- its health.

The recorder also logs every unit with the tick it first appears.

1. *T* + 1 is the first tick whose row shows the placed building, which is the first tick of logic
   after its place event. In the deploy setup, *U* is the yard's first tick in play.
2. *C* is the first tick whose row shows *construction* current and already processed once.
3. *H* is the first tick whose row shows the building construction-completed with *guard* current.
4. For each row from the one after *C* up to the row before *H* (the *H* row already shows the
   *idle* body), compare its stage with that of the row before it, and count the rows whose stage is
   lower (a return towards 0).
5. Find the first tick after *C* whose row shows a higher stage than the row before, and the
   highest stage shown in any row before *H*.
6. Between *C* and *H*, count the ticks whose row shows the same processing-timer start as the row
   before (no restart). On those ticks, note any other change of the building's values.
7. In the refinery setup, find the tick on which the free harvester first appears.

## Result

All 15 runs are counted. "Interval" is the setup type's stage interval and "stages" its range's
stage count, as read from the type.

| Quantity | Unit | n | Mean / value | Spread | Tolerance for tests |
|---|---|---|---|---|---|
| Placed, after the logic of *T* + 1: current behaviour | | 12 runs | none, *construction* queued | none | exact |
| Placed, after the logic of *T* + 1: body state, stage, and whether it advances | | 12 runs | below the table | none | exact |
| Placed, after the logic of *T* + 1: construction-completed | | 12 runs | no | none | exact |
| Placed: *C* − *T* | ticks | 12 runs | below the table | none | exact |
| Placed: stage on *C*, and the tick of its first advance against *C* + interval | | 12 runs | below the table | none | exact |
| Deployed: behaviour current at the end of *U* | | 3 runs | *construction* | none | exact |
| Deployed: stage on *C* | | 3 runs | below the table | none | exact |
| Deployed: *C* − *U* | ticks | 3 runs | below the table | none | exact |
| *H* − *C* against (stages − 1) × interval | ticks | 14 runs | equal | none | exact |
| *H* − *C* against (stages − 1) × interval, refinery seed 2 | ticks | 1 run | one more | | |
| Rows with the stage lower than in the row before (step 4) | rows | 15 runs | below the table | none | exact |
| Highest stage after any tick's logic before *H*, against stages − 2 | | 15 runs | equal | none | exact |
| On *H*: *guard* current, idle body, construction-completed | | 15 runs | all three | none | exact |
| Ticks between *C* and *H* on which the processing timer did not restart: *A*, *B*, gate, deploy | ticks | 12 runs | below the table | none | exact |
| The same, refinery | ticks | 3 runs | below the table | | |
| Refinery: the free harvester's first tick against *H* | ticks | 3 runs | equal | none | exact |

Relations for the table above:

- Placed, after the logic of *T* + 1: body state, stage, and whether it advances: *idle*, 0, not advancing.
- Placed: *C* − *T*: 2.
- Placed: stage on *C*, and the tick of its first advance against *C* + interval: 0, equal.
- Deployed: stage on *C*: 0.
- Deployed: *C* − *U*: 1.
- Rows with the stage lower than in the row before (step 4): 0.
- Ticks between *C* and *H* on which the processing timer did not restart: *A*, *B*, gate, deploy: 0.
- The same, refinery: 1 or 2 per run.

The range's last stage is never seen after a tick's logic: the row after the logic of *H*
already shows the *idle* body.

On each refinery tick without a processing-timer restart, the stage did not advance either. On all
but one of those ticks no stage advance was due, so the stage would not have moved anyway; the
record has no log of the building's update itself. The only other change of the building's recorded
values on any of those ticks was a loss of health on one of them. In refinery seed 2, one of those
ticks fell on a tick when a stage advance after the first was due. That stage was held one tick
longer, the advance was made on the next
tick, and every later stage and the handover came one tick later. On the other refinery ticks without a processing-timer restart, no stage advance was due, and no later timing moved. The *H* − *C* rows count
refinery seed 2 apart for this reason; it agrees with the single build-up run, *C* − *T* and the other rows.

## Evidence

- 15 recordings, played on 2026-11-18: 3 seeds each of *A*, *B*, gate, refinery and deploy.
  Five reruns, one per setup, read the stage counts and intervals. They are not counted. The recordings stay local, and no log hash is given.
- Three refinery runs are void and not counted: the place event found no cell that the refinery
  fitted, so nothing was placed. The counted refinery runs place it on other cells.
- The three runs of one placed type hung well after their handover and were stopped there. Their
  records up to that point are counted. No other counted run crashed or stopped before its end.

## Notes

- **Verdict.** *contradicts* refers to S-0406 revision 14, claim 17 and F7, which give the
  building a place order names two build-up runs and its *C* and *H* from them, and to S-0108
  revision 106, claim 5, which says placed buildings show two runs.
- **Values left out.** The types, the human player's side, the stage counts and the intervals are
  kept in the local values file, because the counts and intervals are stock art and rules values
  and the types would identify them. This record gives relations to them only. It gives no
  handover tick counted from *T*, and no *H* − *C*, as numbers.
- **Drawing.** The recorder has no screen record, so this record says nothing about what is drawn.
  "Runs once" and "idle body" are the recorded stage and body state.
- **Not covered:**
  - a type with no build-up sprite, or with a zero-delay range.
  - a range that reaches its last stage on the first tick of logic (2 stages with a 1-tick
    interval).
  - a range of one stage or fewer.
  - an idle range with a positive delay.
  - a deployed copy of a placed type.
  - a filling building of a wall fill.
  - a replacement restored by a trigger.
  - one map and base only.
- **Why the refinery's processing timer did not restart on some ticks** is not established.
