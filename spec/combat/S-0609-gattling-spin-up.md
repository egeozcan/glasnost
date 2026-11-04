---
id: S-0609
title: Gattling spin-up
topic: combat
revision: 7
sources-class: [decomp, measurement, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-11-04
measurements: [M-0021]
requests: [Q-1220, Q-1151, Q-1714]
supersedes: []
---

# S-0609: Gattling spin-up

## Summary

An object whose type has the gattling flag keeps a spin value and a gattling stage. Sustained
attacking raises them step by step, and idling or interrupted attacking lowers them again. This
spec owns that spin state: its initial and reset values, the normal and elite thresholds, when and
by how much the value rises and falls for vehicles and for buildings, how the stage follows the
value, and the spin sound requests and their random draws. Skirmish only; network games and
campaigns are out of scope.

Which weapon slot a stage selects, and the anti-air test, belong to S-0308 claims 1 to 4 and 7.
Shot timing, rate-of-fire bonuses and the berserk halving belong to S-0301 claims 1 to 8. A gattling
firer leads its target by the slot of its weapon number, not of its stage (S-0306 claim 3).
Suppression's weapon choice is S-0309 claim 14. Boarding is S-0507 claim 6, and the temporal
spin-down is S-0514 claim 4. The multi-turret drawing exclusion is S-0007 claim 16 and S-0105
claim 12. Sound playback and the report-variant rule are S-0701 claims 13, 14 and 21.

## Terms

- **Gattling-flagged type.** A vehicle, building, infantry or aircraft type whose imported gattling
  flag is set. S-0308's *gattling type* is a gattling-flagged type that is also a multi-weapon type.
  Only that subset reads the stage to choose its weapon (S-0308 claim 7), but every gattling-flagged
  type keeps the spin state below.
- **Gattling stage** (S-0308, Terms). A non-negative integer *s* per object. For a supported
  stage count *N* ≥ 2, 0 ≤ *s* ≤ *N* − 1; a type with fewer stages stays at stage 0.
- **Spin value.** An integer *v* ≥ 0 per object of a gattling-flagged type, which the updates below
  raise and lower.
- **Stage count *N*, thresholds.** Integers imported per type (Import mapping): the stage count *N*,
  the *normal thresholds* *K*₁ … *K*_N, the *elite thresholds* *E*₁ … *E*_N, the *spin-up amount* *u*
  and the *spin-down amount* *d*. The **thresholds in force** for an object are its type's elite
  thresholds while the object is of elite rank (S-0311), and its normal thresholds otherwise. Each
  update reads the rank at that moment. Below, *T*ₖ means threshold *k* of the thresholds in force.
  *T*_N is the **spin cap**.
- **Spin-up update with multiplier *m*** and **spin-down update with multiplier *m***. The two
  changes of claims 2 and 3. *m* is a whole number of ticks: 1 for vehicles, the elapsed-tick count
  for buildings (claim 7).
- **Elapsed-tick count.** An integer *c* per object. It grows by 1 every tick just before the
  object's behaviour may be processed (S-0215, *processing*), and returns to 0 whenever a behaviour
  starts (by being queued and started, or forced; an override does not reset it). It also returns
  to 0 at each building spin update that uses it and at the paused rows of claim 7. Only buildings
  read it here.
- **Spin sound, playing mark.** The spin sound is the sound that a spin-up update starts (claim 13).
  The playing mark is a per-object yes/no that records whether a spin sound has been started since
  the last event that cleared the mark.
- **Firing step** of a vehicle: S-0310 claim 9. **Firing check** and its answers: S-0301 claim 9
  (*can fire*, *rearming*, *out of ammunition*, *not facing*, *still turning*, *moving*, *cloaked*,
  *out of range*, *illegal*, *impossible*, *busy*, *must deploy*).
- **Last shot.** The tick of an object's most recent shot, as S-0206 claim 9 counts it. An object
  that has never fired counts as having last fired 100 ticks before tick number 0 (S-0206).

## Behaviour

### Spin state

1. **Initial values and persistence.** Every object of a gattling-flagged type is created with spin
   value 0 and stage 0. Spin value and stage are part of the object's state: only the updates of
   claims 2, 3 and 9 and the resets of claim 11 change them. A change of owner (capture, mind
   control, defection) leaves both unchanged. A change of rank leaves both unchanged too. It only
   switches the thresholds in force for later updates (claim 4). These persistence rules concern
   the same object, not a replacement created by deployment or undeployment.
   - Backing: `measurable` (G1 for the initial stage, G7 for owner and rank changes)
   - Confidence: high (initial values); medium (no change on owner change: no reset was found,
     but a change through an indirect path is not excluded)

2. **Spin-up update.** A spin-up update with multiplier *m* does two things, both decided from the
   spin value *before* the update (F1):
   - The value grows by *u* × *m* when it is below the spin cap *T*_N, and is unchanged otherwise.
     Nothing clamps the grown value to the cap: an increase that crosses the cap can overshoot
     it by up to *u* × *m* − 1. A later change to a smaller elite cap does not clamp the held value.
   - The stage rises by one when it is below *N* − 1 and the value before the update is at least
     *T*ₛ₊₁ (the threshold of the next stage). A stage rises by at most one per update.

   Because the rise test reads the value before the update, a stage rises one update after the
   update that first carried the value to its threshold.
   - Backing: `measured` (M-0021, for a vehicle type with two stages, thresholds 30 and 60 and
     a spin-up amount of 1: at every rise from stage 0 to stage 1 the value reads 30 in the tick
     before and 31 in the tick of the rise, and the largest value read is the cap, 60, in every
     run); `measurable` (G1, G2) for other amounts, stage counts and the overshoot
   - Confidence: high

3. **Spin-down update.** A spin-down update with multiplier *m* lowers the value by *a* = *d* × *m*,
   to no less than 0. It then lowers the stage by one when the stage is above 0 and the lowered
   value is below *T*_s, the threshold that raised it to its current stage (F2). A stage falls by at
   most one per update, even when the value falls by more than one band. When *a* = 0 (a spin-down
   amount of 0) the update sets the value to 0 (`quirks.gattling_zero_spin_down_empties`).
   - Backing: `measured` (M-0021's part B, for the same type with a spin-down amount of 1: at
     every fall from stage 1 to stage 0 the value reads 30 in the tick before and 29 in the tick
     of the fall); `measurable` (G2, G6) for falls of more than one band and the
     amount of 0
   - Confidence: high

4. **Normal and elite thresholds.** Every comparison of claims 2, 3 and 9 uses the thresholds in
   force at that update. When an object becomes elite, the stage and value it has are kept, and the
   elite thresholds govern from its next update on. A spin-up update can only raise the stage,
   and a spin-down update can only lower it, each by at most one: changing rank does not itself
   reconcile the stage with the new threshold band. Elite thresholds are their own imported values:
   an absent elite threshold is 0, not the normal one (Edge cases for what that does).
   - Backing: `measurable` (G7)
   - Confidence: high

### When vehicles spin

5. **Vehicles: one update per tick from the firing step.** In every tick in which a vehicle of a
   gattling-flagged type takes its firing step, its spin changes by exactly one update with
   multiplier 1, after the step's own reply (S-0310 claim 9):

   | Situation in the firing step | Update |
   |---|---|
   | it has a target, its slot 0 holds a weapon, and the firing check (for the weapon S-0308 chooses) answers *can fire*, *rearming*, *not facing* or *still turning* | spin-up |
   | it has a target, its slot 0 holds a weapon, and the answer is any other (*out of ammunition*, *illegal*, *impossible*, *moving*, *out of range*, *cloaked*, *busy*, *must deploy*) | spin-down |
   | it has no target, or its slot 0 is empty | spin-down |
   | a deploy-to-fire vehicle whose answer is *can fire* or *not facing*, so that it queues *unload* instead (S-0310 claim 9, S-0517 claim 13) | none in that tick |

   So a vehicle keeps spinning up while it waits between shots, turns to face its target or is
   still turning, and spins down in every tick in which it has no target or cannot attack it.
   A change of target is not itself an event: the update of each tick depends only on that tick's
   answer. Being bunkered adds no firing-step exclusion or delay (S-0606 claim 9): the same
   per-tick spin rule applies, subject to ordinary inactivity rules, not a guarantee of a shot
   every tick.

   A shot fired in this firing step uses the stage immediately before this step's spin update.
   Usually that is the stage at the end of the previous tick; another event, such as boarding or
   temporal attachment, can change it earlier in this tick. The rate of spin does not depend on the weapon's
   rate of fire, the owner's rate-of-fire bonuses, berserk or game speed: it is one update per tick
   in which the firing step runs. A berserk vehicle therefore fires faster (S-0301 claim 8) but
   spins at the same pace.
   - Backing: `measurable` (G1, G3, G12); individual refusal categories remain `unmeasured`
     if G12 cannot reach and distinguish them with accepted orders.
   - Confidence: high (table rows and shot-before-update); medium (the deploy-to-fire row)

6. **Vehicles: ticks without a firing step freeze the spin.** In a tick in which a vehicle takes no
   firing step, its spin value and stage do not change (no spin-down either). This includes a
   vehicle that is being erased by a temporal weapon between the lock-triggered spin-down updates
   of S-0514 claim 4, a
   vehicle in a chrono move while its warp restriction is active (S-0207 claim 20,
   S-0513 claim 39), and a vehicle travelling through a tunnel. A vehicle not receiving on-map
   updates, such as an ordinary passenger, likewise has no ordinary firing-step update here.
   This does not prohibit the open-topped passenger shots of S-0507. While being erased or
   warp-restricted, the vehicle also loses its target.
   - Backing: `measurable` (G8 for temporal erasure, chrono restriction, tunnel travel and ordinary
     passengers). A complete classification of every state that bypasses the firing step is
     `unmeasured`: some removal paths cannot be separated from destruction of the object.
   - Confidence: medium

### When buildings spin

7. **Buildings in *attack*.** A building of a gattling-flagged type changes its spin in *attack*
   only at the processings of S-0215 claim 32, by one update whose multiplier is its elapsed-tick
   count *c*. Each such update, and each paused row, sets *c* back to 0:

   | Firing check's answer at the processing | Update |
   |---|---|
   | *can fire* (whether or not a shot is actually launched), *not facing*, *rearming* | spin-up with *m* = *c* |
   | *out of ammunition*, *illegal*, *impossible*, *out of range* | spin-down with *m* = *c*, and the building drops its target and starts *guard* (S-0215 claim 32) |
   | *cloaked*, *busy* | spin-down with *m* = *c* |
   | *still turning*, *moving*, *must deploy* | none, and *c* returns to 0 (paused: the elapsed ticks are not counted towards any update) |
   | no target | none: the building starts *guard*, which resets *c* |

   A building with the SAM flag (S-0215 claim 32) makes no spin update in *attack* at all.
   Because S-0215 claim 32 processes a building every tick after *can fire* and every other tick
   after *not facing* or *rearming*, a building firing at a steady rate gets spin-ups with
   *m* = 1 in the ticks it fires and *m* = 2 every other tick while it waits. Its value then rises
   at the same average pace as a vehicle's, but its stage is tested less often (F4). Unlike a
   vehicle, a building pauses on *still turning*. The answer here is the final answer after any
   turret snap of S-0215 claim 32. Any shot in the processing uses the stage before its spin update.
   - Backing: `measurable` (G4, G12). Refusal categories not reached by an accepted test order
     remain `unmeasured`: no isolated setup establishes every possible building firing-check
     answer. G12 records those gaps rather than assuming coverage.
   - Confidence: high (rows and multiplier); medium (*moving* and *must deploy*)

8. **Buildings in *guard*, *sticky* and *area guard*.** Every processing of these behaviours by a
   building of a gattling-flagged type starts with a spin-down update with multiplier *c*, which
   then returns *c* to 0. This happens before anything else that processing does, armed or
   unarmed, so an armed building that finds a target and starts *attack* first spins down by the
   ticks spent in *guard* since its last processing.
   - Backing: `measurable` (G5); `unmeasured` for an unarmed building with pre-existing positive
     spin, because no reachable setup establishing that combination has been traced.
   - Confidence: high

9. **Buildings: passive spin-down outside *attack*.** In every tick in which a building of a
   gattling-flagged type is not in *attack*, and more than *G* + 5 ticks have passed since its last
   shot (*G* the game-wide guard-area targeting delay of S-0206 claim 9), its value falls by *d*, to
   no less than 0, and its stage falls by one when it is above 0 and the new value is below *T*_s.
   This is claim 3's arithmetic with multiplier 1, including the zero-amount rule of
   `quirks.gattling_zero_spin_down_empties`. It never touches the spin sound or the playing mark.
   In a tick in which a processing of claim 8 also takes place, the passive spin-down acts on the
   processing's result, so a building can lose two stages in one tick (F5 example).
   - Backing: `measurable` (G5)
   - Confidence: high

10. **Infantry and aircraft do not spin.** No rule raises the spin of an infantry or aircraft object.
    An infantry or aircraft type with the gattling flag stays at stage 0 and value 0 unless it is
    reset by claim 11 or takes the temporal spin-down of S-0514 claim 4, which leave it at 0.
    - Backing: `measurable` (G9, including aircraft)
    - Confidence: high

### Resets and other events

11. **Resets.** In the tick in which an object boards a transport (S-0507 claim 6), or enters a
    building that absorbs vehicles or infantry (S-0605 for the infantry case), its spin value, its stage and its
    elapsed-tick count become 0. The numeric reset itself has no sound effect; leaving the map
    separately ends the sound's looping and clears the playing mark (claim 13). When
    a temporal weapon successfully locks onto an object of a gattling-flagged type (S-0514
    claim 4), the object receives exactly one spin-down update with multiplier 1 (claims 3 and 13),
    not a reset. Each successful lock applies this update, including a joining attacker's lock
    during an existing erasure interval; there is no continuous temporal spin-down between locks.
    Owner changes, rank changes and new orders do not themselves reset the same object's value
    or stage. Deployment into a building or packing into a vehicle can create a replacement:
    claim 1's initial state applies to that new object, not a persistence guarantee across the
    transformation. No claim is made here that all possible removal or replacement paths preserve spin.
    - Backing: `measurable` (G7, G8, G13); the reset of positive infantry spin on absorption is
      `unmeasured`, because claim 10 supplies no reachable positive-spin infantry state.
    - Confidence: high (boarding, temporal); medium (absorption and persistence)

12. **Spin amounts have no bonus multipliers.** The amounts *u* and *d* are used as imported
    integers, without firepower, armour, rate-of-fire, house, country, difficulty or veterancy
    factors. Rank only chooses the threshold row. Game speed adds no multiplier to the spin
    arithmetic. This does not promise identical stages when another rule changes the schedule
    or firing-check answers: for example, building guard-processing delays follow S-0215 claim 32,
    and a stage can fall by only one per processing even when its value loses many elapsed ticks.
    - Backing: `measurable` (G3's modifier matrix)
    - Confidence: medium

### Spin sound

13. **Spin sound requests and their draws.** An object of a gattling-flagged type never plays the
    ordinary per-shot report of S-0701 claim 21 when it fires. Instead:
    - A spin-up update whose object's playing mark is clear starts a spin sound. The mark is checked
      after the update has raised the stage, if it did. The sound is chosen from the report list of
      the weapon in slot 2*s*, where *s* is the stage after the update, after elite replacement
      (S-0308 claim 2; this is the ground slot even when the object attacks an airborne target).
      Starting it stops any spin sound of the object that is still playing, takes one raw value of
      the unsynchronised stream, picks the list entry by that value as S-0701 claim 21 and its F5
      say, and sets the playing mark. A weapon with an empty report list starts nothing, takes no
      draw, and leaves the mark clear.
    - A spin-up update that raises the stage stops the current spin sound and clears the mark, so
      every stage rise starts a new spin sound with a new draw.
    - Every spin-down update clears the mark and tells the current spin sound to end after its
      present loop (it is not cut off). So the first spin-up update after any spin-down update
      starts a new spin sound with a new draw, even when the stage did not change.
    - A spin-up update that raises no stage while the mark is set starts nothing and takes no draw.
    - Claim 9's passive spin-down and the numeric reset of claim 11 neither stop the sound nor
      clear the mark. Leaving the map separately tells the spin sound to end looping and clears
      the mark, so boarding does not preserve that mark.
    - The spin sound is positional and follows the object as it moves, for any owner. Whether and
      how it is heard is S-0701's.

    So a vehicle that alternates between spin-up and spin-down ticks (for example one whose answer
    flips between *not facing* and *moving*) takes one draw at every spin-up tick that follows a
    spin-down tick. None of these draws reaches game state (S-0202 claim 11).
    - Backing: `measurable` (G10)
    - Confidence: high (when draws happen and which list); medium (the end-after-loop treatment of
      a spin-down, which S-0701 renders)

14. **Turret animation while spinning.** For a vehicle of a gattling-flagged type, the
    animation-frame counter of S-0105 claim 16 advances by one in every tick in which the vehicle
    completes its firing step and its spin value after that tick's update is at least 1. The
    deploy-to-fire early exit of claim 5 makes no advance. Firing adds no separate advance.
    For a building of a gattling-flagged type our engine advances the counter once per tick when
    its spin value after the tick's behaviour processing is positive, with no extra advance for
    firing or for leaving *attack*. The original has a drawing-only bug: outside *attack* it adds
    a second advance when the value remains positive after passive decay. This is corrected
    without a quirk switch. The counter has no effect on play.
    - Backing: `measurable` (G11)
    - Confidence: medium

## Formulas

For stage counts 2 through 6, all spin arithmetic uses signed 32-bit integers, not floating
point. F1 and F2 have no
fractional operations and require no rounding. Their stated non-negative results assume
non-negative amounts and thresholds. Out-of-domain settings and results are handled by the three
engine rules of Edge cases (negative settings used as 0, saturation at 2³¹ − 1, and the
stage count as the layers finally leave it). Under them F1, F2 and F4 see
non-negative inputs, and every product, sum and elapsed count they form lies in the range 0 to
2³¹ − 1. The v_max of F3 is not clamped by its formula; the stored value stops at the bound, as
Edge cases (saturation) says. *s* is the stage and *v* the spin value before the
update, *s*′ and *v*′ after it. *T*ₖ are the thresholds in force (Terms).

**F1. Spin-up update with multiplier *m*** (claim 2).

> *v*′ = *v* + *u* × *m* when *v* < *T*_N, and *v*′ = *v* otherwise.
>
> *s*′ = *s* + 1 when *s* < *N* − 1 and *v* ≥ *T*ₛ₊₁, and *s*′ = *s* otherwise.

Both conditions read *v*, the value before the update.

**F2. Spin-down update with multiplier *m*** (claims 3, 8 and 9; claim 9 uses *m* = 1).

> *a* = *d* × *m*.
>
> *v*′ = max(*v* − *a*, 0) when *a* ≠ 0, and *v*′ = 0 when *a* = 0 (`quirks.gattling_zero_spin_down_empties`; with the switch off, *v*′ = *v* when *a* = 0).
>
> *s*′ = *s* − 1 when *s* > 0 and *v*′ < *T*_s, and *s*′ = *s* otherwise.

**F3. Continuous spin-up of a vehicle** (consequence of claims 2 and 5). Let *u* > 0 and
0 < *T*₁ ≤ … ≤ *T*_N, and let the vehicle start at *v* = 0, *s* = 0 and take a spin-up update every
tick. Its update number *j* (counting from 1) sees the value (*j* − 1) × *u* before the update, while
that is below the cap. The stage reaches *k* (1 ≤ *k* ≤ *N* − 1) at update

> *j*ₖ = max(*j*ₖ₋₁ + 1, ⌈*T*ₖ / *u*⌉ + 1), with *j*₀ = 0,

where the quotient is mathematical rational division and ⌈ ⌉ rounds up to a whole number.
The first shot eligible to use stage *k* is in the tick after update *j*ₖ (claim 5); if rearming
or another refusal prevents a shot then, the first such shot is later. The value stops growing at *v*_max = *u* × ⌈*T*_N / *u*⌉, the first multiple
of *u* that is at least *T*_N.

**F4. Building elapsed-tick multiplier** (claim 7). At a processing, *c* is the number of ticks since
the last event that set it to 0, this tick included. With the delays of S-0215 claim 32, *c* = 1 at a
processing after a *can fire* processing and *c* = 2 after a *not facing* or *rearming* processing.
Away from resets or paused processings, the accumulated increase across a shot cycle is *u*
times the sum of elapsed counts at its spin-up processings, in steps of *u* or 2*u*. Its stage
test (F1) is made only at those processings; initial elapsed counts can differ from the steady cadence.

**F5. Two spin-downs of a building in one tick** (claims 8 and 9). Invented example: *T*₁ = 10,
*T*₂ = 30, *d* = 1,
*s* = 2 and *v* = 31, a *guard* processing with *c* = 25 gives *v* = 6 and *s* = 1. The passive
spin-down of the same tick then gives *v* = 5 and *s* = 0. (Applied the other way round the stage
would end at 1, so the order is observable.)

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Initial spin value and stage | 0 | — | Claim 1 |
| Vehicle spin multiplier | 1 | ticks | Claim 5: one update per firing step |
| Passive spin-down margin | 5 | ticks | Claim 9: added to *G* |
| Passive spin-down multiplier | 1 | ticks | Claim 9 |
| Largest stage count supported | 6 | stages | Import mapping and Edge cases |
| Stage rise or fall per update | at most 1 | stage | Claims 2, 3 |
| Draws per spin sound start | 1 | raw values, unsynchronised stream | Claim 13 |
| Spin arithmetic saturation bound | 2³¹ − 1 | integer: a count of spin-value units for *v* and the spin products, ticks for the elapsed-tick count *c* | Formulas (preamble) and Edge cases (saturation) |

## State and ordering

Spin of a vehicle, per tick in which it takes its firing step (claim 5):

| State | Event | Result |
|---|---|---|
| any | answer *can fire*, *rearming*, *not facing* or *still turning* | the tick's shot (if any) at the old stage, then F1 with *m* = 1 |
| any | another answer, no target, or an empty slot 0 | F2 with *m* = 1 |
| deploy-to-fire vehicle | answer *can fire* or *not facing* | *unload* queued; no update |

Spin of a building (claims 7 to 9):

| Behaviour | Event | Result |
|---|---|---|
| *attack* (no SAM flag) | processing with answer *can fire*, *not facing*, *rearming* | F1 with *m* = *c*; the elapsed-tick count then returns to 0 |
| *attack* (no SAM flag) | processing with answer *out of ammunition*, *illegal*, *impossible*, *out of range*, *cloaked*, *busy* | F2 with *m* = *c*; the elapsed-tick count then returns to 0 |
| *attack* (no SAM flag) | processing with answer *still turning*, *moving*, *must deploy* | no update; the elapsed-tick count returns to 0 |
| *attack* (no SAM flag) | processing without a target | no update; *guard* starts and the elapsed-tick count returns to 0 |
| *attack* (SAM flag) | any processing | no update |
| *guard*, *sticky*, *area guard* | processing | F2 with *m* = *c*; the elapsed-tick count then returns to 0 |
| any but *attack* | tick with more than *G* + 5 ticks since the last shot | passive F2 with *m* = 1, after the tick's processing; no sound change |

Playing mark (claim 13):

| Mark | Event | Result |
|---|---|---|
| clear | spin-up update, slot 2*s*′ weapon with a report list | stop any spin sound, one draw, start a spin sound, mark set |
| clear | spin-up update, empty report list | nothing; mark stays clear |
| set | spin-up update raising the stage | stop the sound, mark cleared, then as the first row |
| set | spin-up update not raising the stage | nothing |
| any | spin-down update (claims 3, 7, 8, S-0514 claim 4) | the sound ends after its present loop; mark cleared |
| any | passive spin-down (claim 9) or numeric reset (claim 11) | no change from that operation |
| any | leaving the map | sound ends looping; mark cleared |

Ordering consequences:
- A vehicle's firing-step shot uses the stage before that step's spin update (claim 5).
- A stage rises one update after the value reaches its threshold (claim 2), but falls in the same
  update in which the value drops below it (claim 3).
- A building's passive spin-down in a tick follows that tick's processing (claim 9, F5).

## Edge cases and quirks

- **Stage count 0 or 1 from creation.** When no earlier layer populated thresholds, they remain
  0. The value never grows and the stage stays 0. Spin-up updates still request sound from slot 0
  (claim 13). A later layer that lowers the stage count below 2 after an earlier layer populated
  thresholds is covered by the next item.
- **Stage count lowered below 2 by a later layer** (engine rule, no quirk switch; confidence:
  high that stock data never reaches it; backing: `unmeasured`, because the original's result for
  such data is not observed and the rule is a design decision for data outside the original's
  supported domain; answers Q-1151). The stage count as the rules layers finally leave it governs.
  With fewer than 2 stages the type uses no thresholds: the thresholds in force (Terms) are all 0,
  the spin cap is 0, the value never grows and the stage stays 0, as for a type that never had
  2 stages, even if an earlier layer populated them. The importer warns about the layer that
  reduced the count. The thresholds are still held as imported under S-0006 claim 3; this item
  alone states that a type with fewer than 2 stages does not use them, and S-0006 claim 3 and the
  glossary need no exception.
- **Elite rank without elite thresholds.** All elite thresholds are 0. An elite object's value then
  never grows (cap 0), but its stage rises by one at every spin-up update (the value before the
  update is always at least 0) up to *N* − 1, and never falls (no value is below 0). So such an
  object reaches its top stage after *N* − 1 spin-up updates and keeps it. This follows from F1 and
  F2 and is not a separate rule.
- **Value above the cap.** The value can exceed *T*_N (claim 2). Spin-down times from the top stage
  start from that larger value.
- **Thresholds out of order.** F1 and F2 are applied as written whatever the order of the
  thresholds. A stage can then rise with a value below a lower stage's threshold and fall again at
  the next spin-down update.
- **Empty slot 2*s*.** The original reads the report list of the weapon in slot 2*s* without checking
  that the slot holds a weapon, and fails when it does not. Our engine treats an empty slot as an
  empty report list (no sound, no draw), with an importer warning. Stock data does not reach this case.
- **More than 6 stages.** The original supports only six thresholds per rank row. Larger counts
  can give corrupted settings or fail, with no safe specified result. Our engine reads at most
  six stages: a stage count above 6 is used as 6, and the importer warns.
  Stock data does not reach this case.
- **Negative settings are used as 0** (engine rule, no quirk switch; confidence: high that stock data never reaches it; backing: `unmeasured`, because
  the original's result for such data is not observed and the rule is a design choice for data
  outside the original's supported domain). A negative stage count, spin-up amount, spin-down amount
  or threshold is used as 0, and the importer warns for each such imported value. An integer that
  S-0003 claim 4 wraps into a negative number is used as 0 in the same way; this is also a topic-spec
  override of the wrap that S-0205 claim 1 gives such an integer. The clamp applies to
  the imported value itself, so a later layer that omits the key keeps the 0, not the negative
  value.
- **Saturation** (engine rule, no quirk switch; same
  confidence and backing; the bound is in Constants). Every spin product (*u* × *m*, *d* × *m*),
  every sum (*v* + *u* × *m*) and the building elapsed-tick count *c* of F4 saturates: a result
  above 2³¹ − 1 is 2³¹ − 1. Nothing wraps: this is a topic-spec override of S-0205 claim 1, which
  makes an out-of-range integer result wrap in every spec that does not say otherwise, and whose
  overflow case for the original is `unmeasured`. No lower bound is reached because all inputs are
  non-negative. Once *v* has saturated, F1 leaves it unchanged, because it is at or above any threshold.
  Once *c* has saturated it stays there until its next reset.
- **`quirks.gattling_zero_spin_down_empties`** (default: on, the original). A spin-down update whose
  amount *d* × *m* is 0 sets the spin value to 0 at once (claims 3 and 9). With a spin-down amount of
  0 an object therefore loses its whole value at its first spin-down update; it then loses at
  most one stage per update, only while the applicable threshold is positive. Off: an amount of 0 leaves the value unchanged, so an object with a spin-down
  amount of 0 never spins down, and its stage falls only when the value is already below *T*_s.
- **Ticks lost to a pause.** A building's elapsed ticks at a *still turning*, *moving* or
  *must deploy* processing are dropped (claim 7). They count neither up nor down. This is the
  original's rule, kept without a switch.
- **Building animation double step.** Claim 14 corrects the original's presentation-only double
  advance outside *attack*, without a quirk switch.

## Import mapping

| Source | Field | Parse form | Default when absent |
|---|---|---|---|
| Rules, type sections (vehicles, buildings, infantry, aircraft), key `IsGattling` | gattling flag (gattling-flagged type) | boolean | held value; no |
| Rules, type sections, key `WeaponStages` | stage count *N* | integer | held value; 0 |
| Rules, type sections, key `RateUp` | spin-up amount *u* | integer | held value; 0 |
| Rules, type sections, key `RateDown` | spin-down amount *d* | integer | held value; 0 |
| Rules, type sections, keys `Stage1`, `Stage2`, `Stage3`, `Stage4`, `Stage5`, `Stage6` (the first *N* of them) | normal thresholds *K*₁ … *K*_N | integer each | held value; 0 |
| Rules, type sections, keys `EliteStage1`, `EliteStage2`, `EliteStage3`, `EliteStage4`, `EliteStage5`, `EliteStage6` (the first *N* of them) | elite thresholds *E*₁ … *E*_N | integer each | held value; 0 (not the normal threshold) |
| Rules, weapon sections, key `Report` | report list of claim 13 | list (S-0701 claim 21) | S-0701 |
| Rules, section `[General]`, key `GuardAreaTargetingDelay` | *G* of claim 9 | integer, ticks | as S-0206 claim 9 maps it |

- "Held value" is the layering rule of S-0006: a later layer that omits the key keeps the value of
  the earlier one.
- The numbered threshold keys are read only when the type's gattling flag is set and its stage count
  is at least 2, for *k* = 1 … *N* (at most 6 in our engine, Edge cases). A layer that does not
  meet this condition leaves the thresholds held from earlier layers unchanged (initially 0), but a
  type whose final stage count is below 2 uses no thresholds (all 0), with an importer warning
  about the reducing layer (Edge cases). Negative values of the stage count, *u*, *d* and the
  thresholds are used as 0, with an importer warning for each (Edge cases).
- The weapon slots per stage (`Weapon`*k* and `EliteWeapon`*k*) are S-0308's.

## Measurement scenarios

All scenarios use a purpose-made test map, stock data otherwise, scripted input, no engine add-on and no mods
(the named test files of a scenario excepted), skirmish, game-speed index 2, 800 × 600, and one bystander computer player kept outside the
observed test area under the glossary's void rule. Use scripted normal input and a read-only
recorder of tick, health, spin value, stage, behaviour and elapsed count. Request any needed
recorder support from the user; do not alter the running game. Repeat each deterministic variant
three times and log the seed rather than assuming it can be injected. Test types and any sound
or model test files are named test-map data, removed after the run, with made-up values chosen
for the test (not stock values). Unless stated, the test vehicle is a gattling-flagged multi-weapon vehicle
type with turret count 1, weapon count 6, stage count 3, *u* = 7, *d* = 1 and normal thresholds
(10, 30, 60). Populate all six slots at normal and elite rank; slots 1, 3 and 5 have usable
anti-air weapons so the secondary-slot gate of S-0308 claim 4 does not bypass stage selection.
Ground slots 0, 2 and 4 hold instant-hit weapons with damage 1, 2 and 3 against the target's
armour (so each hit's damage shows the stage), rate of fire as small as the original accepts
(1 tick if accepted; otherwise note the value used and compare shot ticks only). The target is a
stationary, unarmed, high-strength vehicle of another house, in range and in front of the
attacker's turret at the start. Record per run the tick and damage of every hit (health deltas).

**G1. Vehicle spin-up and the pre-update test (claims 1, 2, 5).**
- *Procedure:* order the attack at tick *t*₀ and let it run 80 ticks.
- *Expected:* the first hits do 1 damage (stage 0 at creation). With one shot per tick from the first
  update, F3 gives *j*₁ = ⌈10/7⌉ + 1 = 3 and *j*₂ = ⌈30/7⌉ + 1 = 6: the first 2-damage hit comes from
  the shot 3 ticks after the first shot, and the first 3-damage hit from the shot 6 ticks after it.
- *Falsified if:* 2 ticks and 5 ticks (a rise tested on the value after the update), or any other
  spacing, or a first hit of more than 1 damage.

**G2. Overshoot and spin-down (claims 2, 3).**
- *Procedure:* after 80 ticks of G1, remove the target (sell it or delete it with a trigger) at a
  known tick. Re-order an attack on a second, identical target after a gap of *g* ticks. Runs with
  *g* = 33, 34, 53 and 54.
- *Expected:* the value stops at *v*_max = 63 (F3). Spin-down by 1 per tick lowers stage 2 when the
  value falls below 30 (after 34 spin-downs) and stage 1 when it falls below 10 (after 54). First
  hit damage: 3 for *g* = 33, 2 for *g* = 34 and 53, 1 for *g* = 54. Allow a one-tick shift for the
  tick of the removal and the re-order, measured once in a control run with *g* = 0.
- *Falsified if:* the boundaries come 3 ticks earlier (a value capped at 60), or stages fall more than
  one per tick.

**G3. Modifier independence (claims 5, 12).**
- *Procedure:* as G1 with a rate of fire of 10 ticks, then berserk at *t*₀. Repeat with game-speed
  indices 0 and 6, and separately with named test changes to firepower, armour, country factors,
  difficulty-row factors and veteran bonuses. Keep normal and elite thresholds identical and
  targets valid and surviving. Record value and stage every tick, not only at shots.
- *Expected:* while firing-check categories match, all runs have the same values and stages per
  update, including *j*₁ = 3 and *j*₂ = 6. Shot counts and real-time durations may differ.
- *Bunker variant (claim 5):* repeat with an eligible test vehicle inside a tank bunker and an
  open-ground control of the same type (S-0606 claim 9). Compare intervals with equal recorded
  firing-check answers and no ordinary inactivity. Expect the same spin update each tick:
  bunker-only missing or delayed spin updates falsify the claim. Shot ticks may differ.
- *Falsified if:* a modifier changes value or stage at a corresponding spin update.

**G4. Building in *attack* (claim 7).**
- *Setup:* a gattling-flagged building type with the same weapons, stage values and amounts, no SAM
  flag, a voxel turret with a slow turn rate, rate of fire 4 ticks.
- *Procedure:* (a) target already in line: record hits for 80 ticks. (b) target placed behind the
  building so that it must turn first.
- *Expected:* (a) each processing follows F1 with its recorded elapsed count as multiplier, and
  any shot uses the pre-update stage. (b) a final *not facing* answer spins up, but *still turning*
  pauses spin and clears the elapsed count. Turret motion alone does not distinguish these
  answers. A snap and recheck uses the final answer, not the initial answer.
- *Falsified if:* a processing disagrees with claim 7's row and F1/F2 for its final answer and
  elapsed count. A run unable to distinguish that answer does not test that row.

**G5. Building spin-down in *guard* and the passive spin-down (claims 8, 9).**
- *Setup:* the building of G4 with a known test guard-area targeting delay *G*, *d* = 1.
- *Procedure:* spin up to the top stage, remove the target, re-target after gaps of *G* + 5,
  *G* + 6, *G* + 20 and 60 ticks.
- *Expected:* stage changes follow the sum of the *guard* processings' spin-downs (by *c* at each
  processing, cadence of S-0215 claim 32) and, from the (*G* + 6)th tick after the last shot, one
  passive spin-down per tick. With the gap *G* + 5 no passive spin-down has yet taken place.
- *Additional variants:* repeat with *sticky* and *area guard*. Record values before processing
  and at tick end. Tune the preceding attack duration and test delays to arrange F5's starting
  conditions; evaluate ordering only when the recorder confirms them. Expect final stage 0,
  not 1. An unarmed positive-spin building has no established reachable setup and remains
  unmeasured here.
- *Falsified if:* passive decay starts at *G* + 5 ticks, never occurs, a guard-family processing
  fails F2 for its recorded elapsed count, or F5 ends at stage 1.

**G6. Zero spin-down amount (claim 3, quirk).**
- *Procedure:* as G2 with *d* = 0, gaps *g* = 1, 2 and 3.
- *Expected:* first hit damage 2 for *g* = 1 and 1 for *g* ≥ 2: the value is emptied at the first
  spin-down, then one stage falls per spin-down.
- *Falsified if:* the stage never falls (value unchanged by an amount of 0).

**G7. Elite thresholds and rank change (claims 1, 4).**
- *Procedure:* (a) as G1 with the attacker pre-placed at elite rank (S-0311) and elite thresholds
  (5, 15, 30). (b) as G1 with the attacker at elite rank and no elite thresholds. (c) as G1, with the
  attacker made elite part-way through, at a known tick, using a map veterancy action. (d) as G1, with the attacker taken over by
  mind control (S-0601) part-way and the attack re-ordered at once.
- *Expected:* (a) boundaries per F3 with the elite values (*j*₁ = 2, *j*₂ = 4 updates). (b) the
  second shot does 2 damage and the third onwards 3 damage, and the stage never falls after the
  target is removed. (c) the rank change itself preserves value and stage, and subsequent
  spin-up updates can only raise the stage, by at most one per update. (d) no drop of stage at the change of owner.
- *Falsified if:* values differ from F1/F2 under the applicable threshold row, the rank change
  itself changes spin value or stage, (b) uses normal thresholds, or (d) resets spin state.
  Record intervening ordinary updates separately from the owner-change event.

**G8. Boarding and temporal spin-down (claims 6, 11).**
- *Procedure:* (a) spin a test vehicle to the top stage, let it board a transport, unload it at once
  and re-attack. (b) spin it to the top stage, then let an enemy temporal weapon start to erase it
  and break the beam (kill the attacker) after a few ticks, then re-attack.
- *Additional variants:* record ordinary passenger, chrono warp-restricted and tunnel intervals.
  Separate the boarding reset from later on-map updates. Repeat temporal attachment just above a
  stage threshold and with zero spin-down amount, to distinguish lock-triggered updates from
  continuous decay. In a separate run with positive spin-down amount and enough remaining spin
  to distinguish decrements, let a second temporal attacker join the first beam at a recorded
  tick, then break both beams before erasure completes. Record spin and the playing mark at each lock.
- *Expected:* (a) value, stage and elapsed count reset to 0 on boarding; value and stage remain
  0 while an ordinary passenger. (b) each successful lock applies F2 once, including the joining
  attacker's lock. Each also ends spin-sound looping and clears the playing mark (claim 13).
  Between locks, temporal erasure supplies no further spin update. Chrono-restricted and tunnel
  intervals preserve recorded entry value and stage.
- *Falsified if:* a reset is missed, a successful lock omits or duplicates its F2 update,
  the joining lock makes no update, or spin changes between locks during erasure without
  another specified spin-changing event. Likewise, an interval without firing steps or another
  specified event must not change spin. Compare numeric values, not merely top-stage damage,
  which could hide many decrements.

**G9. Infantry and aircraft with the gattling flag (claim 10).**
- *Procedure:* give separate infantry and aircraft test types the flag, three stages and populated
  weapon slots; attack valid surviving targets for 80 ticks, including an aircraft attack run.
- *Expected:* recorded value and stage remain 0 throughout; where stage weapon mapping applies,
  hits use stage 0.
- *Falsified if:* either type gains positive spin or stage.

**G10. Spin sound draws (claim 13).**
- *Setup:* G1's vehicle, its slot 0, 2 and 4 weapons each with a two-entry report list whose sound
  entries have no pitch shift and no volume shift (so the sound engine itself takes no draws,
  S-0701 claim 13). Read the unsynchronised stream's position with the allowed read-only recorder,
  in a scene with no other sound or voice events (no selection, no other units).
- *Procedure:* (a) attack for 80 ticks. (b) make the attacker alternate: order *move* and *attack*
  so that its answer flips between *moving* and *can fire* every few ticks, for 60 ticks.
- *Expected:* (a) exactly three raw values taken: one at the first spin-up update, one at each of the
  two stage rises. No draw per shot. (b) one raw value at each spin-up tick that follows a spin-down
  tick or stage rise, counting a tick satisfying both only once.
- *Additional variants:* an empty report list (no draw), distinct ground and anti-air report
  lists while attacking air (ground list chosen), elite replacement, and boarding/unloading
  (next spin-up requests sound again). Record audio with a named looping sound to distinguish
  ending looping from immediate cutoff. Move the attacker across the view and compare friendly
  and enemy owners for positional playback. Let sound playback finish without a spin event:
  completion alone must not clear the mark or cause a new selection draw.
- *Falsified if:* per-shot draws, two selection draws on one rise, no new draw after spin-down
  or map exit, a draw for an empty list, use of the anti-air list, or inconsistent positional or
  loop-ending playback. Separate selection draws from sound-instance draws of S-0701 claims 13
  and 14.

**G11. Turret animation (claim 14).**
- *Setup:* G4's building with a turret model of several animation frames; G1's vehicle likewise.
- *Procedure:* record video during spin-up, during *attack*, and after the target is removed until
  the value reaches 0.
- *Expected in the original:* vehicle advances once per completed firing step with positive
  post-update value. Building advances once for positive post-processing value and, outside
  *attack*, once more if value remains positive after passive decay. This measures the drawing
  bug, not claim 14's corrected engine result. A tick emptying value only at passive decay still
  has its first advance. The deploy-to-fire early exit has no vehicle advance.
- *Falsified if:* the original disagrees with those conditions or adds a separate per-shot advance.

**G12. Refusal categories and deployment (claims 5, 7).**
- *Setup/procedure:* repeat G1 and G4 with named limited-ammunition, out-of-range, cloaking,
  moving, slow-facing and deploy-to-fire variants. Record the final firing-check answer, elapsed
  count and spin immediately around each update. Use accepted ordinary orders only. Also run
  a SAM-flagged building in *attack*.
- *Expected:* each reached category has exactly claim 5's or claim 7's update. Vehicle
  deploy-to-fire early exits and SAM building *attack* make no update. A paused building
  processing discards its elapsed count instead of counting those ticks on resumption.
- *Falsified if:* a reached answer changes spin differently, or discarded ticks are later used.
  Categories that cannot be reached or distinguished remain unmeasured; do not write game state
  to manufacture coverage.

**G13. Absorption and new orders (claims 1, 11).**
- *Setup/procedure:* a named absorbing building accepts a spun-up test vehicle. Record value,
  stage and elapsed count at entry. In separate G1 runs issue stop, move and new attack orders,
  record state immediately at each order and during later ordinary updates.
- *Expected:* absorption resets all three quantities to 0. New orders have no instantaneous
  value or stage reset; later updates follow the firing-check answers. Infantry absorption from
  ordinary zero spin is only a control: positive infantry spin has no reachable setup here.
- *Falsified if:* absorption retains nonzero vehicle spin state, or an order itself resets value
  or stage before ordinary updates.

## Open questions

1. A complete classification of destruction and removal paths that bypass the vehicle firing
   step remains untraced (claim 6); no additional gameplay state is asserted from those paths.
2. Whether the end-after-loop treatment of a spin sound at a spin-down update is audibly different
   from stopping it depends on the sound entry's loop words (S-0701 claim 14).
