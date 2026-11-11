---
id: M-0007
title: Repair depot timing and contact loss with the depot recorder (Q-1610)
topic: combat
specs: [S-0314, S-0108]
requests: [Q-1610]
verdict: contradicts
value-visibility: local
measured-on: 2026-10-28
method: harness
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-11-11
---

# M-0007: Repair depot timing and contact loss with the depot recorder

## Setup

- Game build: stock Yuri's Revenge 1.001, executable hash verified; run under the oracle harness
  (PROVENANCE §7).
- Data: stock, no mods, plus named test files. Every gameplay and AI change of the harness is
  off. Each test file is the stock file with the stated
  keys changed or added, put in that run's game folder. Test set "q1610" has a test rules file
  and a test art file. Test set "q1610-sp3short" has the same rules file and the short art
  variant, and test set "q1610-stockrate" has the test art file only.
- Test rules: the game-wide vehicle repair rate is 0.125 game minutes and the *repair*
  behaviour's rate is 1/64 game minute. Neither is a stock value.
- Test art: the depot's slot animations get named test animations that reuse the stock depot
  images.
  - *special-1*: not looping, longer than every run's tick limit.
  - *special-3*: not looping, running 20 ticks. A "short" variant runs 10 ticks. A *special-3*
    that starts in tick *s* runs in ticks *s* to *s* + 19 and has ended by tick *s* + 20; every
    row below uses that convention.
  - *special-2*, *idle*, *active-1* and *production*: looping.
  - The damaged variants are removed.
- Map: the arena map of M-0001, used as a purpose-made game, not a replay of an archived one.
- Houses: the human player is British, and two computer players are Americans and Russians.
  The human is a normal player: its objects can be attacked. Both computer players play at the
  hardest difficulty level.
- Objects: every starting vehicle and every object of the map's neutral house are removed. The
  human owns a repair depot at 130,82, a power plant, and an MTNK east of the depot, on a local
  cell, whose health is set before tick 0 to a local value below full, in hit points. In
  runs 4a and 4b a second MTNK stands near the first, on a local cell, from tick 0, its health
  set before tick 0 to the same value. Each computer player owns one far power plant and nothing
  else.
  The power plants are asked for at 124,82 (the human's), 60,82 (the Americans') and 100,70 (the
  Russians'). Every object, the depot and the vehicles included, is placed on its asked cell,
  or on a nearby cell if that one is taken, and the cells as placed are not recorded. The
  human's credits are set to 10000 before tick 0.
- Game settings: every other player starts with 0 credits; tech level 10; superweapons
  off; crates off; short game off; bases on; the map is revealed.
- The enter order is given at tick 120.
- Seed 1000, game speed fastest. Tick limits: 912 for the pilot and run 3; 608 for runs 1 and
  2 and their short-art repeats and runs 5, 6 and 7; 1216 for run 4. The stock repeat's and the
  hover attempt's limits are kept `local`: with the public order tick they would bound the stock
  delays and the hover's travel.
- Tick convention: "tick *f*" is the game's frame counter, and a per-object state stamped *f* is
  read after tick *f*'s logic has run.
- What is not stock:
  - no menus: the skirmish setup comes from a settings file
  - every starting vehicle and every object of the neutral house are removed, and the stated
    objects are placed on stated cells
  - the two computer players are not allied with each other
  - the named test rules and art files above
  - scripted orders for the human's vehicles (enter and move)
  - other scripted actions: lethal damage with no attacker (the "destroyed" events below), a
    credits change, and a direct health write (run 6)
  - every scripted action, the orders, the destroys, the credits change and the health write
    alike, is applied just before the logic of its stated tick. "Destroyed at 345" is lethal
    damage applied before tick 345's logic, so a reaction "at 345" is one in tick 345's logic.
    The health settings and the credits "before tick 0" are applied after the objects are placed
    and before tick 0's logic
  - the run ends at a fixed tick limit
  - a per-tick recorder reads the depot's behaviour, repair phase, contact and slot
    animations, and the vehicle's behaviour, destination, health and height, read as its height
    above the ground under it (a vehicle is *down* when that height reads 0). At each of the
    depot's repair processings it records the phase before and after, the delay to the next
    processing that the processing set, the contact and its health, the owner's credits and the
    slot animations after. It reads state only.
  - a random-state recorder logs every draw of the scenario stream, and with each draw the
    depot repair processing, if any, during which it was taken. It reads state only.
    Replaying every logged draw on the S-0202 generator model from the seed gives 0 mismatches.

## Procedure

The runs adapt S-0314 scenario H4 part a (the pilot) and part h runs 1 to 7. Part h asks for a
*special-3* shorter than the delay *D*; in runs 1 and 2 only the short art meets that. Run 7 did
not reach its intended state (Evidence). An attempt at part b's fallback is reported under
Evidence and Notes only.
Run 7's order is also the nearest case of S-0108 scenario B18 part d.

For Q-1610, run 2 settles question 1 for a destroyed vehicle once the per-tick working
processings have begun: the depot reacts in the tick of the loss. Run 1 shows a loss in the gap
after the settling step, before the working processings begin: the depot reacts only at its next
scheduled processing. Run 3 settles question 2 for a destroyed vehicle. The runs settle question
3 only for a depot back in *guard* (run 4); a depot not yet back in *guard* is not covered.

1. Pilot: order the MTNK into the depot at tick 120 and record every tick until well after the
   third repair step.
2. Runs 1 to 6: repeat the pilot and apply each run's event (Result) at its stated tick.
   Repeat runs 1 and 2 with test set "q1610-sp3short" (the short *special-3*).
3. Runs 4a and 4b: after run 2's loss, order a second damaged MTNK in at 420 or 457.
4. Run 7: order the vehicle to move to a local cell beside its start cell at tick 214, 215, 216 or
219, one order per
   repeat. The order at 214 comes before that tick's arrival, the one at 215 before the depot's
   first waiting processing, and those at 216 and 219 in settling and in working.
5. Hover attempt: the pilot with an LCRF in place of the MTNK, at the same cell, its health
   set before tick 0 to a local value below full, as the MTNK's is; no run 7 move order.
6. Repeat the pilot with test set "q1610-stockrate" (stock rules, the same test art), for the
   stock delays.

## Result

| Quantity | Unit | n | Value | Spread | Tolerance for tests |
|---|---|---|---|---|---|
| *D*, the delay that the settling step sets to the next processing, test rules | ticks | pilot; runs 1, 3 and 6 (rows below); runs 4a and 4b through *D* + *K*<sub>u</sub> − 1 | below the table |  | exact |
| *K*<sub>u</sub>, the step interval, test rules | ticks | pilot; runs 3, 4a and 4b | below the table |  | exact |
| *D* and *K*<sub>u</sub>, stock rules | ticks | 1 run | local | | exact |
| Pilot: arrival at the docking point, inferred from one reading (no position or arrival event was read): the first tick from which the MTNK has no destination | tick | 1 | below the table |  | exact for these runs' placement only (Placement, below) |
| Pilot: the MTNK's behaviour from that tick | | 1 | *guard*, not *sleep* | | exact |
| Pilot: the depot queues *repair*, inferred (Evidence) | tick | 1 | below the table |  | exact |
| Pilot: first waiting processing, accepting | tick | 1 | below the table |  | exact |
| Pilot: settling step | tick | 1 | below the table |  | exact |
| Pilot: slots at the settling step |  | 1 | below the table |  | exact |
| Pilot: working processings | ticks | 1 | below the table |  | exact |
| Pilot: second step | tick | 1 | below the table |  | exact |
| Pilot: later steps | ticks | 1 run | below the table |  | exact |
| Step cost | credits | every step | local | | exact |
| Run 1: destroyed at 223 (settling step + 5): first reaction | tick | 1 | below the table |  | exact |
| Run 1: then |  | 2 (one with each art) | below the table |  | exact |
| Run 2: destroyed at 345 (second step + 1): reaction | tick | 1 | below the table |  | exact |
| Run 2: return to *guard* | tick | 2 (one with each art) | below the table |  | exact |
| Run 3: credits set to 0 at 345: the step at 457 |  | 1 | below the table |  | exact |
| Run 3: retries | ticks | 1 | below the table |  | exact |
| Run 3: destroyed at 475: next processing | tick | 1 | below the table |  | exact |
| Runs 4a, 4b: second vehicle's settling steps | ticks | 2 | below the table |  | exact |
| Runs 4a, 4b: its later steps | ticks | 2 runs | below the table |  | exact |
| Run 5: destroyed at 216 (acceptance + 1): next processing | tick | 1 | below the table |  | exact |
| Run 6: health written to full at 216 or 217: settling step | tick | 2 | below the table |  | exact |
| Run 6: next processing | tick | 2 | below the table |  | exact |
| Run 7: move order at 214, 215, 216 or 219 | | 4 | contact released on the order's tick each time | | exact |
| Hover (LCRF): arrival, inferred from one reading (no position or arrival event was read): the first tick from which the vehicle is in *sleep* | tick | 1 | local | | exact for these runs' placement only |
| Hover: accepting waiting processing | tick | 1 | below the table |  | exact |
| Hover: settling processings while it sinks | ticks | 1 | every tick from the first settling processing until it is down, with no step (ticks, span and height local) | | exact at tick granularity: no processing in a tick before the touchdown tick takes a step (the order within the touchdown tick was not established) |
| Hover: settling step | tick | 1 | the first settling processing after it is down (tick local; height was read per tick, so whether the touchdown came before or after a processing within its tick was not established) | | exact |
| B18 part d's nearest case: move order applied at 215, before that tick's logic | | 1 | see the next three rows | | exact |
| The same: the depot's first waiting processing, at 215 |  | 1 | below the table |  | exact |
| The same: the depot's behaviour |  | 1 | below the table |  | exact |
| The same: *special-1* | | 1 | does not start | | exact |
| The same: *active-1* | | 1 | already running; whether it restarted cannot be seen | | exact |
| Scenario-stream draws in any depot repair processing | draws | every run reported under Result (not part b's unreported attempt) | below the table |  | exact |

- **Placement.** The vehicle's travel to the docking point depends on where the depot and the
  vehicle were placed, which is not recorded (Setup). So the arrival tick (itself inferred from
  the readings its rows name), and with it every
  absolute tick after it in the rows above, holds only for these runs' placement: a rerun with
  another placement can move the arrival and every later tick by the same amount, and that is
  not a falsifier. Each row's timing relative to the arrival, to the processing before it or to
  the run's event (the 1-, 3-, *D*- and *K*<sub>u</sub>-tick spacings, the reactions to a loss
  and the slot durations) is taken to carry over to such a rerun, with the table's "exact"
  tolerance. That is an inference, not an observation: these runs did not vary the placement.
  The run events are stated against those processings too (for example run 1's loss at the
  settling step + 5).
- **Relations.** *K*<sub>u</sub> = 113 is S-0314 Formulas F1's smallest integer at or above
  900 times the vehicle repair rate (112.5). *D* = 14 is the *repair* behaviour's rate delay of
  S-0215 Formulas F1, as S-0314 F1 uses it (900 times its rate is 14.0625). The stock repeat
  gives *D* and *K*<sub>u</sub> by the same formulas from the stock rates (values in the local
  file). The second step, 126 ticks after the settling step, is S-0314 claim 6's
  *D* + *K*<sub>u</sub> − 1. The run 4 settling steps are 37 ticks apart, not a multiple of 113,
  and 177 and 214 ticks after the lost vehicle's last step. In both run 4 repeats no part of the
  lost vehicle's step spacing carried over.

Relations for the table above:

- *D*, the delay that the settling step sets to the next processing, test rules: 14.
- *K*<sub>u</sub>, the step interval, test rules: 113.
- *Pilot: arrival at the docking point, inferred from one reading (no position or arrival event was
  read): the first tick from which the MTNK has no destination*: 214.
- Pilot: the depot queues *repair*, inferred (Evidence): 214.
- *Pilot: first waiting processing, accepting*: 215; it sets the next processing 3 ticks later.
- *Pilot: settling step*: 218; it sets the next processing *D* later.
- *Pilot: slots at the settling step*: *special-1* starts; *active-1* and *idle* stop.
- *Pilot: working processings*: every tick from 232.
- *Pilot: second step*: 344 (settling step + 126).
- *Pilot: later steps*: every 113.
- *Run 1: destroyed at 223 (settling step + 5): first reaction*: 232: settling; *special-3* and
  *active-1* start.
- *Run 1: then*: processed every tick while *special-3* runs; *guard* at 253 (short *special-3*:
  243), the tick after the one by which *special-3* has ended (start 232: 252, short 242).
- *Run 2: destroyed at 345 (second step + 1): reaction*: 345: settling; *special-3* and *active-1*
  start.
- Run 2: return to *guard*: 366 (short *special-3*: 356), the tick after the one by which
  *special-3*, started at 345, has ended (365, short 355).
- *Run 3: credits set to 0 at 345: the step at 457*: *cannot pay*: settling; *special-3* and
  *active-1* start.
- *Run 3: retries*: 458 and 472.
- *Run 3: destroyed at 475: next processing*: 486 (472 + 14), none in between; *guard*
  there; *special-3*, started at 457, ran to 476 and had ended by 477.
- *Runs 4a, 4b: second vehicle's settling steps*: 521 and 558.
- *Runs 4a, 4b: its later steps*: settling step + 126, then every 113.
- *Run 5: destroyed at 216 (acceptance + 1): next processing*: 218: *guard*; *special-3* does not
  start.
- *Run 6: health written to full at 216 or 217: settling step*: 218: *not needed*, no
  payment; contact released; vehicle sent to a cell away from the depot (cell local).
- *Run 6: next processing*: 232 (+14), settling with no contact; *guard* in that processing, shown
  first in tick 232's record; no *special-3*.
- *Hover: accepting waiting processing*: the arrival + 1.
- *The same: the depot's first waiting processing, at 215*: no contact; *special-3* starts; it sets
  the next processing 1 tick later.
- *The same: the depot's behaviour*: *guard* at 216, while *special-3*, started at 215, runs to 234.
- *Scenario-stream draws in any depot repair processing*: 0.

## Evidence

One run per row unless *n* says otherwise, every tick recorded. The recordings stay local. No
recording or log hash is given. Each run's draws replay with 0 mismatches against the S-0202
generator model.

- Run 5: *active-1* was already running from before the docking. A reading of the slot's name
  alone cannot show whether it restarted at the loss.
- Run 6: a direct health write stands in for the healer vehicle that the scenario names.
- Run 7's intended state, a depot in waiting that holds the vehicle's contact while the move
  order stands, was not reached: the vehicle's tether was not read, and the move order released
  the contact every time. The scenario needs S-0314 H18 b's tethered vehicle. The run 7 rows
  report what each order did.
- B18 part d requires the depot to be in waiting at the loss. Its nearest case here is a
  different case, a loss before the depot's first waiting processing, so for part d it is
  inconclusive. Its rows record what that first waiting processing does when it finds no
  contact.
- The depot's queued *repair* at 214 and the answers *cannot pay* (run 3) and *not needed* (run
  6) are not direct recorder readings. The recorded quantities behind them are the phases, the
  delays, the contact, the health, the credits and the slot animations.
- Part b's fallback was also attempted once: an MTNK at full health given an enter order. Its
  set-up beyond the enter order and its tick limit are not recorded, so it is not reported under
  Result (Notes).

## Notes

- **Inconclusive: a vehicle at full health that is ordered to enter (committed S-0314 revision
  88, claim 16, and S-0314 scenario H4 part b).** Committed claim 16 expects such a
  vehicle to enter and to be sent away at its settling step without payment. H4 part b
  expects the entry to be refused. The attempt (Evidence) saw no contact in any tick's record, no
  step and no payment. That fits a refusal, and also an enter order that was never applied, and
  its set-up is not recorded, so it neither confirms nor contradicts either expectation. Run 6
  stands in for part h's fallback.
- **Contradiction: the waiting row with no contact, for a loss before the depot's first waiting
  processing (committed S-0314 revision 88, claim 6's table and claim 22).** Claim 6's table and
  claim 22 have the depot return to *guard* at that processing. The processing, at 215, started
  *special-3* and set the next processing 1 tick later. The depot was in *guard* at 216, while
  *special-3* still ran. A settling depot with no contact waits for *special-3* to
  end (run 1); the waiting depot here did not.
- **Confirmed, with a tick added (committed S-0108 revision 81, claim 20's waiting row).** The
  row has the depot leave its repair behaviour and start *special-3*, and gives no tick for the
  exit. Both happened; the exit came 1 tick after the processing. The row's other slot changes
  are not shown: *active-1* was already running, and *production* and *special-2* are not
  reported at that processing.
- **New finding: the vehicle's state at arrival (committed S-0314 revision 88, claims 17 and
  21).** Claim 17 has the arrival queue *sleep* at the vehicle, and claim 21 has the depot remove
  the vehicle's destination at the settling step. From the arrival tick, which the Result
  infers from the readings it names, the MTNK was recorded in *guard* with no destination, before
  the settling step; the LCRF was in *sleep*. Repair proceeded in both. A current *guard*
  does not show whether a *sleep* was queued.
- **Confirmed (committed S-0314 revision 88, claims 6 and 22, and S-0108 revision 81, claim 20).**
  The working phase is processed every tick (run 2). In runs 1, 3 and 5 the loss was met at the
  processing already scheduled, and none was brought forward. A later vehicle at a depot back in
  *guard* starts its own step count (run 4: its second step comes 126 ticks after its own settling
  step, then every 113). In the pilot and in runs 1 to 4 and 6, the first vehicle's settling step
  came 3 ticks after the acceptance, which was the depot's first waiting processing after the
  arrival. A hovering vehicle is processed every tick until it is down, and its step comes then (the
  order of touchdown and processing within a tick was not established). The parts of S-0314 claim
  6's table that these runs exercised hold as written: the settling step and its delay *D* (pilot),
  the working spacing and the step interval (pilot, runs 2 and 4), *cannot pay* with its retries
  (run 3) and *not needed* (run 6).
