---
id: M-0014
title: Building slots after a power-off order next to an ownership change or a pulse's end
topic: rendering
specs: [S-0108, S-0405]
requests: []
verdict: confirms
value-visibility: local
measured-on: 2026-10-28
method: harness
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-30
---

# M-0014: Building slots after a power-off order next to an ownership change or a pulse's end

This record runs S-0108 scenario B22 parts (n) and (o) with the power-off order in each cycle that
the parts name: part (n) in the cycle of *t* − 1, part (o) in the cycles of *e* − 3, *e* and
*e* + 2. Every expectation of part (o) is met in its run, as far as the record's removal coverage
reaches (Evidence). In part (n)'s run the slots keep their ordinary schedule, with an advance at
*t* + 2, and the building is enabled after the change while switched off; no hold is shown there
(Result, b22n-t). In every run of both batches the end-of-tick object lists (buildings, vehicles and
infantry of every house) show removals only at ticks 1, 133 and 134. The lists show no removal of a
listed kind near the named ticks. Full B22 validity is not established (Evidence). Four more order
runs, in the cycles of *t* − 2, *e* − 4, *e* + 1 and *e* + 5, are reported as observations. The
record also shows when a building's light shows again after a pulse's end and after an ownership
change, for S-0405 claim 9 and scenario W15, at the recorder's level: the record reads the light
state, not W15's ground light level.

## Setup

- Game build: stock Yuri's Revenge 1.001, executable hash verified; run under the oracle harness
  (PROVENANCE §7).
- Data: stock apart from the test map, the test rules file and the test art file below, and the
  pulse. Each test file is the stock file with the stated keys changed or added, put in the run's
  game folder. No mods, and every gameplay and AI change of the harness is off.
- Map: a test map made from the stock map `Arena.mmx`, the arena map of M-0001. No hash of the
  stock map or of the test map was supplied. Every object of the neutral house is removed
  from the map file. Two test buildings of type W are added for the neutral house, switched on in
  the map's building list: W7 at cell (132, 82) and W8 at cell (132, 88). A one-time map trigger
  on W7 gives it to the bystander computer player (Americans) when its elapsed-time event fires.
  The event's value in the test map is 20, a test value. The tick of the change of owner, 300, is
  known from the recording run (Result); no second value was run, so the record does not test how
  the value maps to a tick.
- Test rules file: type W is a copy of the stock light post `GALITE`, a building whose light shows.
  The test file gives W the needs-engineer and capturable flags, a power drain of 10 (a test value)
  and no needs-power flag. The file also adds test type V, the human's extra vehicle: a copy of the
  stock medium tank `MTNK` with a test weapon. V serves only as the pulse's generator. What its test
  weapon changes against the stock tank's is not recorded. The second batch's runs record no fire
  event (Evidence), so V fires no shot there; the first batch did not log fire events.
- Test art file: W keeps the light post's art and gains four test slots. *Idle* and *active-1* have
  *freeze-unpowered* on. *Active-2* has it off and *light-unpowered* on. *Active-3* has it off and
  *effect-unpowered* on. Every slot loops 5 image frames with no loop delay and a stage delay of 3
  ticks. Each slot also has a damaged variant with another image.
- Players: the human (British) owns a power plant, two engineers and one vehicle of type V at cell
  (137, 93). The engineers start east of W7 and W8, one for each; their cells are kept `local`, in
  the record's local values file.
  There are two computer players, the Americans (the bystander) and the Russians. Each owns a power
  plant and a construction yard. Cells as placed, the same in every run (a building's cell as the
  recorder reports it): the human's power plant (121, 79), the Americans' power plant (60, 82) and
  construction yard (58, 88), the Russians' power plant (100, 70) and construction yard (101, 60).
  Both computer players play at the hardest difficulty level. Nothing is sold, and the run ends only
  at its tick limit.
- Game settings: every player starts with 0 credits; tech level 10; superweapons off; crates
  off; short game off; bases on; the map is revealed.
- Game: seed 1000, game speed 4, 608 ticks, started from a settings file without the menus.
- Inputs: the human orders the engineers to capture W7 and W8 at tick 30. Scripted orders are
  applied just before the stated tick's logic, as the harness states it. The power-off order is
  the human's switch-off order for one building.
- **Order tick and cycle (derived, not measured).** This record reports an order applied before
  tick F's logic as an order in the cycle of tick F − 1 (S-0201, *cycle*). That mapping is
  derived: S-0203 claim 7 has an order given in the cycle of tick F − 1 take effect after tick
  F − 1's turns and before any object's turn in tick F, and S-0405 claim 7 has the house work out
  its power in its update of tick F. What the runs show is consistent with it:
  - the human's drain falls in tick F (b22o-em3 at 286, b22n-tm1 at 299, b22n-t at 300),
  - the switched-off building's light is out in the state recorded for tick F (b22n-tm1,
    b22o-ep1, b22o-ep2, b22o-ep3 and b22o-ep6),
  - the state recorded for tick F − 1 does not show the order.

  That the order acts before any object's turn in tick F is not observed directly, and the runs
  do not compare an injected order with one given in the ordinary way. Every cycle label below
  (*t* − 1, *e* − 3, *e* and the rest) rests on this mapping.
- **The pulse is not a stock input.** At tick 200 a pulse is applied at W8's cell, spread 1 cell,
  length 90 ticks (test values), with the human's extra vehicle as its generator.
- **What is not stock**, in one list:
  - the test map, the test rules file and the test art file above
  - the pulse, which the harness creates (above)
  - the start from a settings file, without the menus
  - every player's starting construction vehicle, the human's included, is removed by the
    harness, and each leaves play at tick 1 (Evidence)
  - the stated objects, the human's vehicle of type V included, are put on the map by the
    harness before tick 0, at the cells given above as placed
  - computer players are not allied with each other (stock would ally them)
  - the capture and power-off orders are applied directly as the human's orders, just before the
    stated tick's logic, not through the network order queue
  - the recorders, which read state only.

## Procedure

1. Recording run b22-rec. After every tick's logic, record for W7 and W8: owner, health, switched-on
   state, manned and captured state, the disabled state (S-0108 claim 13's sense) as the recorder
   reads it, the pulse hold left, whether the light shows, the owner's power drain, and for each
   slot its image frame, variant, frozen state and stage delay. List the players' objects at the
   end of every tick. From this run take *e*, the first tick after the pulse at whose end W8's
   light shows
   again, and *t*, the tick of W7's change of owner to the bystander.
2. Order runs, each with the same seed and the same earlier inputs plus one power-off order applied
   before the logic of the tick named, recording the same fields as step 1. For the cycles that B22
   names: b22n-t (W7 at 300, the cycle of *t* − 1), b22o-em2 (W8 at 287, the cycle of *e* − 3),
   b22o-ep1 (W8 at 290, the cycle of *e*), b22o-ep3 (W8 at 292, the cycle of *e* + 2). Observations:
   b22n-tm1 (W7 at 299, the cycle of *t* − 2), b22o-em3 (W8 at 286, the cycle of *e* − 4), b22o-ep2
   (W8 at 291, the cycle of *e* + 1), b22o-ep6 (W8 at 295, the cycle of *e* + 5).
3. A second recording run in the batch of b22n-t, b22o-em2, b22o-ep1, b22o-ep3 and b22o-ep6.
4. Log every draw of the scenario random stream in every run. In the second batch, also log
   damage events and fire events.
5. In every run of both batches, list the players' objects at the end of every tick from 0 to
   607, as in step 1.

## Result

Ticks are those at whose end the state was recorded.

**Recording run.**

| Event | Tick | What the record shows |
|---|---|---|
| captures | 133 (W7), 134 (W8), as the run's timeline only, not as a measurement of engineer movement (the ticks also include the order's delay and path set-up) | owner becomes the human; manned and captured; the light shows. *Idle* and *active-1* advance in the next tick, then every 3 ticks |
| pulse | 200 | W8 disabled, light out (the pulse's light-out of S-0405 claim 9), hold 89. *Idle* and *active-1* hold the frame reached in tick 198 (frame 2); *active-2* and *active-3* go on every 3 ticks |
| hold | 201 to 289 | counts down by 1 per tick, 0 at the end of tick 289 |
| *e* | 289 | W8's light shows and W8 is enabled again, in the tick the hold reaches 0. *Idle* and *active-1* advance 2 → 3, 91 ticks after their last advance, then every 3 ticks |
| *t* | 300 | W7's owner is the bystander. Its slots do not change at the change: *idle* and *active-1* advance in 299 and 302 |

**Order runs for the cycles B22 names.** All four are in the second batch. Each "met" refers to
the recorded frame timing, subject to the removal-coverage limitations under Evidence.

| Run | Order (cycle) | Result | B22 expectation |
|---|---|---|---|
| b22n-t | W7 off at 300 (*t* − 1) | At 300 W7 is switched off and is the bystander's. After the order its light shows and it is enabled at the end of every tick. *Idle* and *active-1* advance in 299 and 302, then every 3 ticks to the end. The human's drain falls by W7's drain in 300; the bystander's recorded drain does not change when W7 joins its house. W7 stays switched off | part (n): advancing again no later than *t* + 2: met, at *t* + 2. The hold from the order is not shown: the order and the change of owner first show in one recorded tick, and the slots keep their ordinary 3-tick schedule (299, 302), so this run does not tell a hold that the change of owner ends from no hold at all |
| b22o-em2 | W8 off at 287 (*e* − 3) | At 287 W8 is switched off. No new frame at 287 or 288. At 289 the hold ends, the light shows, W8 is enabled again, and *idle* and *active-1* advance 2 → 3, then every 3 ticks to the end. W8 stays switched off | part (o): no new frame up to *e* − 2 and advancing by *e* + 1: met, at *e* |
| b22o-ep1 | W8 off at 290 (*e*) | *Idle* and *active-1* advance 2 → 3 in 289. At 290 W8 is switched off and disabled, its light is out, and they hold at frame 3 from 290 to the end (607) | part (o): no new frame after the end of *e*: met |
| b22o-ep3 | W8 off at 292 (*e* + 2) | At 292 W8 is switched off and disabled, its light is out. The advance of *idle* and *active-1* due in 292 does not come, and they hold at frame 3 to the end | part (o): no new frame after the end of *e* + 2: met |

In every order run, *active-2* and *active-3* go on every 3 ticks without a break.

**Order runs reported as observations.**

| Run | Order (cycle) | Result |
|---|---|---|
| b22n-tm1 | W7 off at 299 (*t* − 2) | At 299 W7 is switched off, its light is out, the human's drain falls by W7's drain, and *idle* and *active-1* hold at frame 0 (due at 299). At 300 W7 is the bystander's, still switched off, its light shows, it is enabled again, and *idle* and *active-1* advance 0 → 1, then every 3 ticks to the end. W7 stays switched off; the bystander's recorded drain does not change when W7 joins its house |
| b22o-em3 | W8 off at 286 (*e* − 4) | At 286 W8 is switched off and the human's drain falls by W8's drain; *idle* and *active-1* are still held by the pulse. No new frame at 287 or 288. At 289 the hold reaches 0, the light shows, W8 is enabled again, and *idle* and *active-1* advance 2 → 3, then every 3 ticks to the end. W8 stays switched off with its light showing and its drain not counted |
| b22o-ep2 | W8 off at 291 (*e* + 1) | At 291 W8 is switched off and disabled, its light is out, and *idle* and *active-1* hold at frame 3 (reached in 289). No new frame of *idle* or *active-1* from 292 to 607 |
| b22o-ep6 | W8 off at 295 (*e* + 5) | *Idle* and *active-1* advance 3 → 4 in 292. At 295 W8 is switched off and disabled, its light is out. The advance due in 295 does not come, and they hold at frame 4 to the end |

**Same game up to each order.** Each order run's record equals its batch's recording run's on
every tick before its order, and the second recording run's record equals the first's. After the
game-start seeding with seed 1000, the scenario-stream draw log is identical in every run, and
replaying it on the S-0202 generator gives 0 mismatches. The logs differ only
before the game-start seeding.

**Health.** No damaged variant shows in either batch. In the first batch W7 and W8 are at full
health at every recorded tick end, and the second batch logs no damage event.

**Light and switch.** In b22n-tm1 a switched-off building's light shows again at its change of
owner, and in b22o-em3 at its pulse's end. In both, the building stays switched off and is enabled
again, its drain is not counted, and its *freeze-unpowered* slots play on to the end. In b22o-em2
too the light shows again at the pulse's end and the slots play on. In b22n-t the switch-off and the
change of owner first show in the same recorded tick, and the record never shows the light out.

Tolerance for tests: exact ticks.

## Evidence

- Two recording runs and eight order runs, in two batches. Each order run was made once
  (*n* = 1 per order run). No recording or log hash is given.
- The first batch is b22-rec, b22n-tm1, b22o-em3 and b22o-ep2. The second batch is the second
  recording run and b22n-t, b22o-em2, b22o-ep1, b22o-ep3 and b22o-ep6. Its six runs record no
  damage event and no fire event.
- The object lists hold every building, vehicle, infantry unit and aircraft in play, of every
  house, the neutral W7 and W8 included (no aircraft is in play). They hold no animation,
  projectile, particle, terrain object or overlay, so the validity check below covers removals of
  the listed kinds only.
- In the end-of-tick object lists of every run of both batches, ticks 0 to 607, objects drop out
  only at tick 1 (the starting construction vehicles) and at ticks 133 and 134 (each engineer as
  it enters its building). So no removal those lists show falls near a tick that a part names,
  before or after its order (S-0108 B22, Validity). An object that enters and leaves play within
  one tick does not show in an end-of-tick list (S-0203), so the lists do not rule out such an
  object.
- An unlisted removal cannot account for the results, by inference from committed specs, not
  checked against the recording. A counting removal costs an object at most its turn in that tick
  (S-0203 claim 10), so it would show as a one-tick delay of an advance. It cannot make a hold
  last to the end of the record (607), as in b22o-ep1, b22o-ep2, b22o-ep3 and b22o-ep6. And each
  advance that a part's expectation checks comes in the tick the recording run gives (289 after
  the pulse's end) or on the 3-tick schedule, with no one-tick delay.

## Notes

- **The pulse's end and the light.** S-0108 B22 part (o) and S-0405 scenario W15 allow the light to
  show one tick after the pulse's end. Here it shows at the end of the tick in which the hold
  reaches 0. The record reads the light's state, not the ground light level that W15 reads.
- **Disabled state.** The record's disabled state is the building's own state as the recorder reads
  it. The record does not evaluate S-0405 claim 8's in-operation test.
- **The pulse for other scenarios.** Part (o)'s pulse is not a stock input (Setup). The scenarios
  that need a pulse, S-0108 B22 parts (e)
  and (k) and B11's pulse run, and S-0405 W15's pulse parts, are not run here.
- **Same tick as the change of owner.** In b22n-t the order is applied before the logic of the tick
  of the change of owner, while W7 is still the human's. Only b22n-tm1 (the cycle of *t* − 2) shows
  a hold from the order followed by the re-enabling at the change of owner; for the cycle of *t* − 1
  the record shows the slots on their ordinary schedule, with an advance at *t* + 2, and the
  building enabled after the change while switched off; no hold is shown.
- Not recorded: a power order applied after an ownership change in the same tick, and a type that
  needs and draws power. Every order here was given while the building was the
  human's.
