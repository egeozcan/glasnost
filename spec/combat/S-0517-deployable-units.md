---
id: S-0517
title: Deployable units (in-place vehicle deploy cycle, deploy-fire vehicles, deployed infantry firing)
topic: combat
revision: 10
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-18
measurements: []
requests: [Q-0982]
supersedes: []
---

# S-0517: Deployable units (in-place vehicle deploy cycle, deploy-fire vehicles, deployed infantry firing)

## Summary

How a vehicle that deploys in place changes between its mobile and deployed states: which deploy
orders it obeys, how long each change lasts, what interrupts it and what it may and may not do while
deployed or changing. It also covers the vehicle that fires at its own position instead of deploying,
a vehicle that must deploy before it fires (its cell test and its toggling), why a vehicle's *unload
or deploy cycle* (S-0215) never runs, and the deployed-firing rule of deployed infantry.

Other specs own the following, and this spec only points to them:
- deploying into a building and packing up again (S-0415 claims 1 to 12),
- the infantry deploy and undeploy sequences, their delays and the infantry *unload* and *guard*
  rules (S-0215 claims 30 and 31; S-0107 claim 21 for when sequences start),
- the weapon a deployed unit chooses (S-0308 claims 8 and 9),
- the firing step's reply to a deploy-to-fire vehicle (S-0310 claim 9),
- the *must deploy* answer (S-0301 claim 9),
- the deploy order (S-0213 claim 10) and the input rules for deployed units (S-0214, Terms *deploy
  readiness* and *in-place deployer*),
- passenger unloading (S-0507),
- radiation creation and top-ups from a deployed infantry unit's own-cell shot (S-0525 claims
  1 to 4; S-0215 claim 31 owns when it fires),
- jumpjet hovering before a deploy request, landing and interrupted descent (S-0502 claims 13,
  15, 16 and 24),
- whether deployed infantry can be crushed (S-0107).

Skirmish only. Network games and campaigns are out of scope.

## Terms

- **In-place deploying vehicle.** A vehicle whose type has the in-place deploy flag (Import mapping).
  It is S-0214's *in-place deployer* when that deployer is a vehicle. It is not S-0415's *deploying
  vehicle*.
- **Mobile, deployed.** The two resting states of an in-place deploying vehicle. A new vehicle is
  mobile.
- **Deploying, undeploying (vehicle).** The transition from mobile to deployed, and back (claims 4
  and 6). A vehicle in either one is *in transition*.
- **Landing-first type.** A type of in-place deploying vehicle with the deploy-to-land flag (Import
  mapping). It must be on the ground to deploy (claim 5).
- **Landing for deploy.** The phase of a landing-first vehicle between a deploy step that finds it
  airborne and its touchdown (claim 5). It is not a transition.
- **Deploying presentation.** The animation type that a vehicle type names for its deploy (Import
  mapping). It plays at the vehicle's position during each transition. Its start field *a*, end
  field *E* are S-0109's (Terms). Its base stage delay *d* is the rate-derived delay of
  S-0109 claim 2, before random rate selection or game-speed normalisation.
- **Transition elapsed stages.** A count of elapsed stages for the vehicle's transition, starting
  at 0 when the presentation is created. It is separate from the presentation's displayed stage.
  Claim 7 owns its timing and the completion tests.
- **Deployed image type.** The type whose image draws a deployed vehicle (claim 12).
- **Deploy-fire vehicle.** A vehicle whose type has the deploy-fire flag but not the in-place deploy
  flag (claim 2).
- **Deploy-to-fire type.** A vehicle type with the deploy-to-fire flag. It must deploy before it fires
  (S-0310 claim 9).
- **Deploy-to-fire cell test.** The test of claim 14 on a vehicle's own cell. S-0301 claim 9 and
  S-0310 claim 9 call it the deploy test.
- **Deployed-firing rule.** The rule that S-0215 claims 23 and 31 run for a deployed infantry unit
  (claim 16).

## Behaviour

### Vehicles: which deploy rule applies

1. **What a vehicle's *unload* does.** S-0415 claim 1 gives the order in which a vehicle's *unload*
   processing chooses its rule. This spec owns two of those rules:
   - the deploy-fire rule, which applies to a deploy-fire vehicle (claim 2),
   - the in-place cycle (claims 3 to 8), after the applicable passenger, deploy-fire, harvesting
     and building-deploy rules of S-0415 claim 1. A higher-priority rule that handles the
     processing prevents in-place deployment.

   Bunker release is not such a stopping rule. A bunkered in-place deploying vehicle, once
   released by S-0606 claim 12, reaches claim 3's cycle in that same *unload* processing unless
   its type has the harvester or weeder flag or deploys into a building. Passenger capacity
   does not prevent this continuation, and the deploy-fire rule does not apply to a vehicle
   that began the processing bunkered. The cycle uses claim 3's delays and queued-behaviour
   effects: with claim 8's default quirk setting, its return to *guard* replaces the release's
   queued *move*. Release therefore does not guarantee that the queued *move* survives.
   This settles S-0606 claim 12's in-place continuation question; that claim and its open
   question on the released in-place vehicle need the corresponding update.
   - Backing: `measurable` (P1 a and P8 a for overlapping non-bunkered types). Bunker release
     followed by the in-place cycle is `unmeasured`: no end-to-end ordinary-input route to a
     bunkered vehicle's *unload* is established (S-0606 claim 11). P8 a's conditional bunker
     variant is diagnostic until that route is found.
     S-0606 claim 11 establishes ordinary band-box exclusion, not a universal selection
     refusal: the bunker link alone does not exclude ordinary click selection. An end-to-end
     player-input route through selection and deploy to the occupant's *unload* remains
     unestablished; band-box exclusion does not prove that route impossible.
   - Confidence: high (rule precedence and same-processing in-place continuation); low
     (end-to-end player-input reachability)

2. **Deploy-fire vehicle.** At each processing of *unload*, a deploy-fire vehicle uses its own
   cell as the target and submits the weapon in slot 0 to the firing check, including its range
   test (S-0301 claim 9). If the check answers *can fire*, it fires that weapon at its own cell.

   It always uses slot 0. The deploy weapon index (S-0308 claim 9) plays no part, because the
   vehicle never becomes deployed.

   The processing then chooses a delay of 14 + *u* ticks, where *u* is one draw over 0 … 2 from the
   scenario random stream (Formulas F3). The draw happens whatever the firing check answered.
   *unload* stays current, so the vehicle keeps firing at its own cell until it is given another
   behaviour. It stays still meanwhile, because its destination is cleared by the deploy order
   (S-0213 claim 10).
   - Backing: `measurable` (P2 for firing and delays; P2 R for the draw count, value and its
     occurrence even after refusal).
   - Confidence: high (slot 0 and the delay); medium (the draw on a refused shot)

### Vehicles: the in-place cycle

3. **Deploy order and processing.** A deploy order makes *unload* the queued behaviour (S-0213
   claim 10). S-0214 decides when the order is accepted. At each processing of *unload*, an in-place
   deploying vehicle takes one step:
   - when it is mobile, a deploy step (claims 4 and 5),
   - when it is deployed, an undeploy step (claim 6).

   So a deploy order given to a deployed vehicle packs it up.

   After the step:
   - if the vehicle is now in transition, or landing for deploy, the processing chooses 1 tick, with
     no draw. Processing therefore happens every tick until the change finishes,
   - otherwise, with claim 8's default quirk setting, *guard* replaces any queued behaviour and
     is started at once. The processing then
     chooses 450 ticks. That delay overrides the start (S-0215 claim 3c), so the first
     processing of *guard* comes 450 ticks later. The vehicle does not wait that long to find a
     target: *guard* is current, so the per-tick search of S-0215 claim 33 (S-0309 claim 1, first
     row) runs whenever its targeting timer runs out, and a target it finds is fired at through
     the ordinary firing step.
   - Backing: `measurable` (P1 b and c for behaviour and timing; P2 R for no delay draw).
   - Confidence: high (the steps and the delays); medium (the target found before the first
     *guard* processing)

4. **Deploy step on the ground.** The outcome depends on height and transition state.

   | Case | Result |
   |---|---|
   | height above the ground is 0, type names no deploying presentation | it becomes deployed in that step |
   | height 0, type names a deploying presentation, not yet deploying | the presentation starts at its position, playing forward, and the vehicle becomes deploying |
   | deploying | it becomes deployed when the completion time of F1 has passed; otherwise nothing changes |
   | height above 0 | claim 5 |

   A vehicle becomes deployed at the end of a deploy step, never part-way through a tick's
   processing. Nothing else can make it deployed.
   - Backing: `measurable` (P1 b and V)
   - Confidence: high

5. **Landing first.** A deploy step that finds the vehicle above the ground does one of two things:
   - for a type that is not landing-first, nothing. The processing then returns to *guard* (claim
     3), so the order is dropped,
   - for a landing-first type, it requests landing for deploy. Without that request, a hovering
     jumpjet of this type keeps hovering (S-0502 claim 13).

   The deploy order requests one destination clear before clearing its target and queuing
   *unload*. Destination handling follows S-0502 claims 24, 27 and 28. With a movement
   commitment, an effective clear reroutes and retains the replacement movement point and
   commitment. A suppressed empty-to-empty vehicle assignment instead retains the preceding
   point and commitment; it does not reroute. A qualifying balloon-hover rejection retains
   even a nonempty order-level destination, together with its preceding movement point,
   commitment and landing reservation. That rejection can make *attack* current before
   *unload* is queued (S-0502 claim 28). Deploy neither enables destination refresh nor
   retries the clear after target clearing: it can end with no target but a nonempty
   destination. An equal horizontal point permits descent when the landing request is
   present; a different point first causes cruise to that point. Descent and touchdown
   use the point then retained, not an absent point.

   Without a movement commitment, the clear cannot create one or wake idle hover. A landing
   request alone does not bypass S-0502 claim 3's update eligibility. Landing checks, descent
   and touchdown otherwise follow S-0502 claim 15. Each processing before touchdown is another
   deploy step that finds it still airborne. At touchdown the landing request ends. The next
   processing then takes the deploy step of claim 4.
   - Backing: `measurable` (P1 b and V; S-0502 J22 for effective versus suppressed clears and
     retained touchdown points; J24 for deploy not enabling refresh; J25 for balloon-hover
     retention). Uncommitted idle-hover deployment remains `unmeasured`: no ordinary stock
     order sequence establishing that starting state is settled, as qualified in S-0502
     claim 24. Retained refresh and a stock deployable balloon-hover fixture inherit the
     `unmeasured` limitations of S-0502 claims 27 and 28 respectively; P1 R treats them as
     diagnostics, not established stock fixtures.
   - Confidence: high (the deploy step); medium (landing motion and point retention)

6. **The undeploy step.** When it starts, the undeploy step removes a disguise the vehicle wears.

   | Case | Result |
   |---|---|
   | type names no deploying presentation | it becomes mobile in that step |
   | type names a deploying presentation, not yet undeploying | the presentation starts at its position, playing in reverse (S-0109), and the vehicle becomes undeploying |
   | undeploying | it becomes mobile when the completion time of F2 has passed; otherwise nothing changes |

   When a landing-first vehicle completes an animated undeploy, it is given a destination from
   a nearby-cell search centred on its position (S-0212 claim 25), allowing its locomotor to take
   off again. This initial search uses the animated-undeploy settings of S-0502 claim 23:
   the type's own locomotion class, with winged counted as track, and its own movement zone,
   not simplified. It is distinct from the subsequent hover-class, flying-zone destination
   adjustment of S-0502 claims 16 and 23. The destination is not guaranteed to be its current
   cell. An immediate undeploy without a presentation does not assign this destination.
   Its subsequent height is open.
   - Backing: `measurable` (P1 c and V) for the state change, destination assignment and lift-off.
     Disguise removal is `measurable` (P8 b). Search settings inherit S-0502 claim 23's
     `measurable` backing (J16); P1 establishes assignment, not those settings.
   - Confidence: high (state changes), medium (takeoff and removing the disguise)

7. **Exact completion and the two clocks.** With a vehicle update on every tick and an
   uninterrupted presentation, completion is on *T* + 1 tick in both directions, where
   *T* = *t*₀ + (*E* − 1)*d* for deploy and *T* = *t*₀ + (*E* − 2)*d* for undeploy,
   for positive *d* and positive stage thresholds (F1/F2). This is constant, not a choice
   controlled by start-tick parity, other animations, the vehicle's place among other objects,
   or a random draw. A lost vehicle update can delay completion; that is not a selection
   between two otherwise allowed completion ticks.

   Transition elapsed stages first increase at *t*₀ + *d*, never at creation for *d* > 0,
   and increase by one every *d* ticks thereafter. A processing on the tick of an increase
   still uses the preceding count: reaching the threshold on tick *T* makes completion
   possible at the next processing, *T* + 1. The count uses the base delay, not a randomly
   selected or game-speed-normalised playback delay. No draw selects this count's delay
   or completion tick. The nonpositive thresholds and zero-delay cases are the explicit
   exceptions in F1/F2 and the zero-delay quirk. Completion never occurs in the processing
   that starts the transition.

   The presentation itself follows S-0109 claims 3 to 5 and 8: forward on deploy, reversed
   on undeploy, with any random rate or normalisation affecting its playback only. It is
   created during the vehicle's turn, with no initial loop delay. With ordinary same-tick
   first update, no lost animation updates and positive playback delay *p*, its first
   displayed-stage advance is *p* ticks after creation, not on the creation tick. With no
   random rate or normalisation, *p* = *d*. Lost animation turns can delay that image change
   under S-0109 claim 5 without delaying the vehicle's separate count. Neither a backwards
   image sequence nor the first-update exclusion reverses or pauses transition elapsed stages.
   - Backing: `measurable` (P1 b, c, V and C)
   - Confidence: high

8. **Interrupting.**
   - An ordinary order cannot start a different behaviour during deploying or undeploying: the
     vehicle is not ready to change (S-0215 claim 11). With
     `quirks.deploy_transition_discards_orders` on (default original), a behaviour queued during
     the transition is replaced by *guard* at completion (claim 3), not resumed afterwards.
     This does not undo other effects of the order, such as changing a target. With the switch
     off, completion starts the pending behaviour, using *guard* only when nothing is pending.
     The completion processing still chooses 450 ticks in either setting.
   - Landing for deploy is not a transition, but this does not make the vehicle ready to change.
     The ordinary moving-vehicle readiness condition still applies (S-0215 claim 11). In
     particular, a descending jumpjet in *unload* is moving now (S-0502 claim 20) and normally
     cannot start a queued behaviour just because it is landing rather than deploying.
   - A destination given during descent, including by a move order, can nevertheless cancel
     the landing independently of starting the queued behaviour (S-0502 claim 16): the vehicle climbs back towards its flying
     height and the deploy is abandoned. If *unload* is still current, *guard* is queued with
     permission to interrupt movement and starts when the other readiness conditions permit.
   - An order without a destination is not guaranteed to replace *unload* during descent.
     If another rule does replace it, no further deploy step follows, but the rest of the descent
     is open. A stop order changes no behaviour (S-0213 claim 8); its effect on the remaining
     descent and deployment is untraced (Open questions).
   - Destruction ends any phase; nothing of the cycle outlives the vehicle.
   - Backing: `measurable` (P3 for readiness, destination cancellation and loss of pending
     behaviour; P1 Z for destruction). Later motion after a no-destination replacement or stop
     remains an open question: no outcome for it is claimed.
   - Confidence: high (readiness and replacement of pending behaviour); medium (the landing cancel)

### Vehicles: while deployed or in transition

9. **No destination.** Outside the magnetic-grip case below, a deployed vehicle or one in
   transition cannot receive a destination: any destination given to it is replaced by none.
   This covers orders, approach steps and every other rule that sets a destination. A deployed
   vehicle cannot move under its own power.

   Magnetic lift makes the vehicle *seized* (S-0607 claims 5 and 20). While it remains
   gripped, being deployed does not itself clear its destination; this also applies during
   undeploying while it is still deployed. This is not permission to assign a new destination:
   the seized-state rule still ignores new destinations, and assigning no destination has
   S-0607 claim 9's grip-ending effect.
   This corrects this spec's previously approved claim 9: being carried does not permit new
   destination assignments. S-0607 claim 9 remains unchanged; its seized-state restriction
   applies alongside the narrower deployed-state exception here.
   - Backing: `measurable` (P3 a for the ordinary restriction). The gripped/deployed exception
     and seized-state destination handling are `unmeasured`: no ordinary input route to test
     these assignments is established, as qualified in S-0607 claim 9. P4 b is conditional on
     establishing that route, not evidence that it exists.
   - Confidence: high (ordinary restriction and ignored new destinations); medium (the
     deployed-state exception during a grip)

10. **Move.** When *move* is processed for a vehicle that is deployed or in transition, *guard*
    becomes its queued behaviour and the processing chooses 1 tick. This covers the deployed
    state as well as the transitions of S-0215 claim 22. This special fallback does not apply
    to *patrol*, whose ordinary processing remains outside this spec (S-0215 claim 34).
    - Backing: `measurable` (P3 b) for the deployed state. Processing *move* during a
      transition is `unmeasured`: ordinary orders cannot start it until the transition ends,
      and named type data do not bypass the readiness gate. The patrol comparison during
      that transition is likewise unreachable by ordinary input. No additional rule for
      deployed *patrol* is claimed here; S-0215 claim 34 retains ownership.
    - Confidence: high (move fallback); medium (patrol distinction)

11. **Firing refusals.**
    - A vehicle's transition supplies an *illegal* refusal for every weapon and target at the
      rank of S-0301 claim 9's rank 4 group; higher-ranked refusals still win. A deployed vehicle
      otherwise fires normally, with the weapon of S-0308 claim 9.
    - A weapon whose warhead carries its target off with a locomotor (a magnetic beam) gets *illegal*
      against a target vehicle in either of these cases:
      - it is in transition,
      - it is an in-place deploying vehicle whose current behaviour is *unload*.

      The second case covers a vehicle landing for deploy, and one in its first processing of the
      order.
    - Backing: `measurable` (P4 for refusals; P1 b for ordinary deployed firing).
      The exact *illegal* answer is `measurable` through the healing-weapon target-drop
      reply (P4 c), and its priority against other refusals through P4 d.
    - Confidence: high (in transition); medium (the beam rule)

12. **Appearance and sound.**
    - While a vehicle is in transition its own body is not drawn: the deploying presentation stands in
      for it.
    - While deployed, the vehicle is drawn with the image of its type's deployed image type, when its
      type names one, and with its own image otherwise. The deployed image type is used only for
      drawing; the vehicle keeps its own type for every rule.
    - With `quirks.deploy_sound_repeats` on (default original), each deploy step requests the
      type's deploy sound at the vehicle's position, and each undeploy step requests its
      undeploy sound there. Landing and transitions therefore repeat the relevant request
      every processing, normally every tick, including the completion processing. Playback
      remains subject to S-0701 claims 6, 8 to 10 and 12; not every request must be audible.
      With the switch off, the deploy sound is requested once, at the first deploy step that
      starts landing, a transition or an immediate deploy, and the undeploy sound once when
      an undeploy step starts. Landing-first deployment then makes no second request at touchdown.
    - Backing: `measurable` (P1 b, c, V and S) for drawing and audible sound repetition.
      The exact per-processing sound-request cadence is `unmeasured`: P1 S records mixed
      audio, whose suppression and service timing cannot distinguish every tick's request, and
      no persistent per-request history is established for the read-only recorder. Single
      requests with the switch off are a design alternative. Non-drawing type properties
      are `measurable` (P8 c compares selection bounds, occupancy and combat properties).
    - Confidence: high (drawing); medium (when the sound plays)

### Vehicles that must deploy before they fire

13. **Deploy-to-fire.** When the firing step of S-0310 claim 9 deploys a deploy-to-fire vehicle, it
    makes *unload* its queued behaviour without starting it at once. The behaviour starts when the
    vehicle is ready to change (S-0215 claims 8 and 11). What *unload* then does follows claim 1.

    With `quirks.deploy_to_fire_repacks` on (the default), a deploy-to-fire type that also has
    the in-place deploy flag does not fire once deployed. Each
    time its firing step reaches *can fire* or *not facing* it queues *unload* again, and an
    *unload* processing of a deployed vehicle undeploys it (claim 3). Such a vehicle cycles between
    deploying and undeploying and never fires (`quirks.deploy_to_fire_repacks`).
    - Backing: `measurable` (P5, using named test data)
    - Confidence: medium (the queueing); low (the repacking cycle)

14. **Deploy-to-fire cell test.** A cell passes when all three hold:
    - its slope shape is flat (S-0406),
    - no building is among its occupants,
    - its land type has the buildable flag *B* (S-0211 claim 2 c).

    Nothing else about the cell counts: units in the cell, overlay other than what the land type
    reflects, and ownership.

    The approach of a deploy-to-fire vehicle whose own cell fails uses this same test:
    - S-0215 claim 37 f2 gives its adjusted reach,
    - in the approach search of S-0307, a primary cell is considered only when it passes this test,
      and a skipped primary never anchors the search's failure outcome.

    The rest of that search is open (S-0307, Open questions).
    - Backing: `measurable` (P5 a for the three conditions, P5 b for units, ownership and
      overlay exclusions, P5 c for the primary-cell and failure-anchor qualification).
    - Confidence: high (the three conditions); medium (that nothing else counts)

15. **The unload or deploy cycle of vehicles.** No rule ever starts a vehicle's *unload or deploy
    cycle* (S-0215, Terms), whatever its type. For vehicles that cycle is therefore always finished:
    - it never keeps a vehicle from being ready to change (S-0215 claim 11),
    - the restart in *move* and *patrol* processing (S-0215 claims 22 and 34) never has an effect.

    The deploy-time field is read for vehicle types, but it affects no vehicle. Aircraft and
    buildings are not covered by this claim.
    - Backing: `measurable` (P6, matched vehicle variants). Any observed cycle start falsifies
      the claim; finite tests do not prove its universal scope.
    - Confidence: high

### Deployed infantry

16. **Deployed-firing rule.** Whenever S-0215 claims 23 or 31 run this rule for a deployed infantry
    unit, let *w* be its weapon selected with no target supplied (S-0308 claims 3 and 8). For a deployed infantry
    type with the deploy-fire flag, this is its deploy weapon (S-0308 claim 8).

    When the unit has a target within *w*'s range under S-0307, this rule makes no targeting
    change; firing remains subject to its ordinary firing step (S-0301). Without an in-range
    target, it instead runs an S-0309 range search centred on its own position without waiting
    for its targeting timer. A found target becomes its target, replacing any out-of-range
    target. If the search finds none, an out-of-range target is dropped, while a unit that
    began without a target remains targetless. A unit left targetless falls back to rest
    under S-0215 unless its current behaviour is *guard* or its type is a jumpjet type.

    The rule never moves the unit: a deployed infantry unit takes no approach step (S-0215 claim 37
    g). The delay after the processing is S-0215's.

    The separate own-cell shot of radiation-immune deployed infantry is governed by S-0215
    claim 31, not this targeting rule. If its weapon leaves radiation, the detonation creates a
    field centred on that cell or tops up the field already centred there (S-0525 claims 1 to 4).
    - Backing: `measurable` (P7 a to e for deployed deploy-fire infantry; P7 f for the jumpjet
      exception and infantry without the deploy-fire flag). The separate radiation rule inherits
      its backing and scenarios from S-0215 claim 31 and S-0525 claims 1 to 4, not P7.
    - Confidence: high (the in-range and search outcomes); medium (that the search ignores the
      targeting timer); low (whether S-0309 claims 2 and 3 limit the search: Open questions)

## Formulas

**F1. Deploy completion.** For base stage delay *d* > 0 and end field *E* > 1, a transition
started at tick *t*₀ completes on this tick (claim 7):

> *t*₀ + (*E* − 1) × *d* + 1 tick

Here *d* is the presentation's base stage delay (S-0109 claim 2), without random rate
selection or normalisation, and *E* its end field after
absent-key defaults (S-0109 claim 25). Integer ticks and integer stage counts, exact: no rounding.
Completion observes *E* − 1 elapsed-stage increases, one per *d* ticks, with the 1-tick
observation lag of claim 7. The initial frame *a* does not
change this duration. With *E* ≤ 1, completion is at *t*₀ + 1 tick. With *d* = 0 and *E* > 1,
the default is no completion, not *t*₀ + 1 tick; the zero-delay quirk owns both settings.

**F2. Completion of an undeploy.** As F1, with *E* − 2 in place of *E* − 1.
For *d* > 0 and *E* > 2:

> *t*₀ + (*E* − 2) × *d* + 1 tick

With *E* ≤ 2, completion is at *t*₀ + 1 tick. With *d* = 0 and *E* > 2, the zero-delay quirk
applies. Only when both stage thresholds are positive is the nominal undeploy duration one stage
delay shorter; both completions include the same 1-tick lag.

**F3. Deploy-fire delay.**

> delay = 14 + *u*,  *u* uniform over 0, 1 and 2

Integer ticks. *u* is one ranged draw from the scenario random stream at each processing of a
deploy-fire vehicle's *unload*.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Transition processing delay | 1 | ticks | claim 3; no draw |
| Delay after a step that leaves no transition | 450 | ticks | claim 3; the general default of S-0215 claim 4 |
| Deploy-fire base delay | 14 | ticks | F3 |
| Deploy-fire random span | 0 … 2 | ticks | F3, both ends included |
| Deploy-fire weapon slot | 0 | slot index | claim 2 |
| Stage count step | 1 | stages per *d* ticks | F1 |
| Completion stage count, deploy | *E* − 1 | stages | F1 |
| Completion stage count, undeploy | *E* − 2 | stages | F2 |

## State and ordering

**Table V. In-place deploying vehicle.** "Processing" means a processing of *unload*. The order
events assume that S-0214 accepted the order. F1/F2 completion events occur on the exact ticks
specified by the formulas under claim 7's uninterrupted-update conditions. No completion event
occurs for a stalled zero-delay transition with its quirk on.

| State | Event | Result |
|---|---|---|
| mobile, on the ground | processing | no presentation: deployed; *guard* queued and started, 450 ticks. With presentation: deploying, presentation starts, 1 tick |
| mobile, airborne, not landing-first | processing | nothing; *guard* queued and started, 450 ticks |
| mobile, airborne, landing-first | processing | landing request set, 1 tick; flight follows the retained point and update eligibility of claim 5 |
| landing for deploy | processing, still airborne | stays landing, 1 tick |
| landing for deploy | touchdown | mobile on the ground; the next processing takes the ground row |
| landing for deploy | destination given (move, approach) | landing ends, vehicle climbs; *guard* queued if *unload* is current |
| landing for deploy | behaviour order without a destination | ordinary moving-vehicle readiness applies; no automatic interruption (claim 8) |
| landing for deploy | stop order | *unload* is not replaced; subsequent height and deployment remain open (claim 8) |
| deploying | processing before completion | stays deploying, 1 tick |
| deploying | processing that completes F1 | deployed; *guard* queued and started, 450 ticks |
| deploying or undeploying | ordinary order queues a behaviour | cannot start during the transition; default: completion replaces it with *guard* (claim 8) |
| deploying or undeploying | destination given | ordinary restriction: cleared; a seized vehicle follows claim 9 and S-0607 claim 9 |
| deploying or undeploying | *move* processing | *guard* queued, 1 tick |
| deployed | deploy order, then processing | no presentation: mobile; with presentation: undeploying, presentation starts in reverse, 1 tick |
| deployed | move order | no destination; *move* processing queues *guard* (claims 9, 10) |
| deployed | attack order, target in range | fires with S-0308 claim 9's weapon |
| undeploying | processing that completes F2 | mobile (landing-first: nearby destination assigned, permitting takeoff); *guard* queued and started, 450 ticks |
| any | destroyed | the cycle ends with the vehicle |

**Ordering consequences.**
- A deploy order reaches its first processing as S-0215 claim 3 says for any behaviour start, and
  the first step is taken there. So a vehicle with no presentation is deployed on the tick its
  *unload* is first processed.
- Completion removes the transition-based firing refusal. Later firing checks see the resulting
  state, but a magnetic beam can still be refused while *unload* remains current (claim 11).
  By default completion replaces a pending behaviour with *guard* (claims 3 and 8).
  No exact within-tick boundary between two actors is specified here; that remains an
  open timing question rather than a claim backed by P4's tick-level log.
- Every processing of a deploy-fire vehicle takes one draw for its delay (F3), so its processings
  are 14 to 16 ticks apart. A shot it fires may take draws of its own (the combat specs).

## Edge cases and quirks

- **Presentation with zero stage delay** (`quirks.deploy_stall_zero_delay`, default on).
  The vehicle uses *d* from S-0109 claim 2, including its rate sentinel −1 and fast-rate quirk.
  With this switch on and *d* = 0, no stages advance. A deploy with *E* > 1, or an undeploy
  with *E* > 2, never completes: the vehicle stays hidden and in transition, cannot fire or
  move, and orders wait rather than interrupt it. Destruction still ends the cycle.
  With the switch off, any zero-delay transition completes at the next processing,
  *t*₀ + 1 tick, as if the presentation's end had already been reached.
  Backing: `measurable` (P1 Z) for zero-delay deploy. Zero-delay undeploy is `unmeasured`:
  a positive-threshold zero-delay type cannot reach the deployed state by its deploy cycle,
  and type data cannot change the delay mid-game. Reaching that undeploy would require a
  state write, which the measurement setup forbids. Confidence: high. The off setting is a design alternative.
- **Deploy-to-fire with the in-place flag** (`quirks.deploy_to_fire_repacks`, default on; claim 13).
  On: a deployed vehicle queues *unload* instead of firing, and repacks. Off: a deployed
  deploy-to-fire vehicle fires normally, and only a mobile one queues *unload*. P5 tests the
  original setting; the off setting is a design alternative.
- **Pending behaviour lost at completion** (`quirks.deploy_transition_discards_orders`, default
  on). Claim 8 owns the rule. On: completion replaces any pending behaviour with *guard*.
  Off: completion starts the pending behaviour, or *guard* if there is none. Backing:
  `measurable` (P3 a); confidence: high. The off setting is a design alternative.
- **Forced behaviour change during a transition.** The transition advances only in the processing of
  *unload*. A behaviour that another rule forces on the vehicle mid-transition (S-0215 claim 9)
  would leave it in transition indefinitely. No stock situation that does this is known (Open
  questions). No switch is proposed until one is. Backing: `unmeasured`: scripted orders
  obey readiness, and no type-data field forces a current-behaviour replacement through that
  gate. Establishing this hypothetical state directly would require a forbidden state write.
  Confidence: low.
- **Deploy sound repeats** (`quirks.deploy_sound_repeats`, default on; claim 12). On: each deploy
  or undeploy processing requests the relevant sound, reproducing the original repetition.
  Off: request the deploy sound once at the first step that starts landing, a transition or an
  immediate deploy, and the undeploy sound once when an undeploy step starts. Sound mixing
  restrictions still apply in either setting. Backing and confidence are those of claim 12.
  The off setting is a design alternative.
- **Airborne non-landing-first type** (`quirks.airborne_deploy_order_dropped`, default on; claim 5). A deploy order to an airborne in-place deploying type without the deploy-to-land flag is silently dropped. With the switch off, the order is retained until the vehicle is on the ground, and the next processing takes the ground deploy step of claim 4.
- **Deploy-fire vehicle with no slot-0 weapon.** The firing check answers *impossible* (S-0301 claim
  9). The vehicle stays in *unload*, firing nothing and processing every 14 to 16 ticks.
- **Deployed image type that differs in size.** Only drawing changes. The footprint, selection
  box and rules stay its own type's (claim 12; P8 c).
- **Immediate landing-first undeploy.** Without a presentation, undeploy makes the vehicle mobile
  but supplies no takeoff destination (claim 6). Do not assume the animated case's takeoff.
  Later height remains an open question, not a second transition duration.
- **AI houses.** Every player house follows the human unit rules above (glossary). Original-only
  computer behaviour is not specified by this document.

## Import mapping

All keys are in the vehicle's or infantry's type section of the rules file unless stated.

| Key | Parse form | Absent default | Feeds |
|---|---|---|---|
| `IsSimpleDeployer` | boolean | no | in-place deploy flag (vehicles; Terms, claims 1, 3 to 12) |
| `DeployToLand` | boolean | no | landing-first type (claims 5, 6) |
| `DeployingAnim` | animation type name | none | deploying presentation (claims 4, 6, 12; F1, F2) |
| `UnloadingClass` | vehicle type name | none | deployed image type (claim 12) |
| `DeployFire` | boolean | no | deploy-fire flag (vehicle: claim 2; infantry: S-0215 claims 30, 31) |
| `DeployFireWeapon` | integer slot index | 1 | deployed weapon (S-0308 claims 8, 9); not used by claim 2 |
| `DeployToFire` | boolean | no | deploy-to-fire type (claims 13, 14; S-0310 claim 9) |
| `Deployer` | boolean | no | infantry in-place deployer (S-0215 claims 30, 31; S-0214) |
| `DeploySound` | sound name | none | deploy sound (claim 12) |
| `UndeploySound` | sound name | none | undeploy sound (claim 12) |
| `DeployTime` | decimal, game minutes | 0 | length of a vehicle's unload or deploy cycle, converted to ticks as S-0215 F9 converts a door time; no effect on vehicles (claim 15) |
| `UndeployDelay` | integer ticks | −1 | infantry undeploy delay (S-0215 claim 30; S-0214) |
| `DeployedCrushable` | boolean | yes | crushability of deployed infantry (S-0107) |
| art file, presentation section: `Rate` | integer frames per game minute | base delay 1 tick; −1 keeps held value | base *d* under S-0109 claim 2; vehicle completion does not apply random rate or normalisation (claim 7) |
| art file, presentation section: `Start`, `End` | integers | 0 for start; end from sprite under S-0109 claim 25 | *a*, *E* of F1 and F2 |
| art file, presentation section: `RandomRate`, `Normalized` | integer pair; boolean | (0, 0); no | presentation playback only, S-0109 claims 3 and 4; not transition elapsed stages |

## Measurement scenarios

Measurement setup (glossary): game speed index 2, 800 × 600 pixels, no engine add-on and no mods (the named test files of a scenario and the purpose-made test maps excepted), stock data
unless a named test rules file is stated. The map is a tiny purpose-made skirmish map with one human
house and two bystander computer players (glossary), one owning the named enemy targets and one
kept entirely away from the observed area. Their unnamed objects must not interfere; neither is
assumed idle. Input is scripted. The recorder (glossary) logs per tick, for
each object:
- type, owner, cell, height above the terrain, health, behaviour and target,
- its processing timer (S-0215, Terms). A tick on which the timer has just been set to a new positive
  value is a processing tick, and that value is the delay chosen (S-0215 claim 2),
- whether its body is drawn, and which image draws it,
- shots fired, with weapon slot and target,
- the local player's sounds.

The bystander and enemy house may draw from the scenario stream. P2 R therefore uses
within-tick random-state observations bracketing the tested processing; a whole-tick draw delta
alone is not evidence of attribution. Moving test actors belong to the human house; enemy targets are assigned
explicitly. The observed area comprises the test objects' cells and the cells traversed during
the run. The glossary's void rule applies. Unless stated otherwise, repeat each part five times
with different recorded seeds. Named test data are isolated overrides *deploy-cycle-test* (P1,
P3 and P4), *deploy-fire-test* (P2), *deploy-to-fire-test* (P5), *vehicle-cycle-test* (P6),
*deployed-infantry-test* (P7 f) and *deploy-overlap-test* (P8), removed after their runs.

**P1. In-place cycle timing.**
- Setup: named test data *deploy-cycle-test* gives a jumpjet vehicle the in-place and landing-first
  flags, a presentation with *d* > 0 and *E* > 2, and a deployed image distinct from its own.
  It hovers over open clear flat ground. An enemy vehicle stands inside its deployed weapon's
  range, and both its weapons can legally hit that enemy. Read *a*, *E* and *d* from the test
  presentation data. Variant S also supplies a deploy sound and an undeploy sound.
- Procedure:
  - (a) A named test rules file gives the in-place type a passenger capacity of 1. Give it the
    deploy order.
  - (b) Give the deploy order to the test type. Record the tick its *unload* is first processed
    (*t*ₒ), its touchdown tick, the tick the presentation starts (*t*₀) and the tick it is first
    drawn with its deployed image. Then record its first shot at the enemy, its processing ticks
    and its targeting timer.
  - (c) At least 500 ticks later, give the deploy order again. Record the same marks for the
    undeploy, its assigned destination, and its cell and height per tick afterwards.
  - (S) Repeat (b) and (c) with the test sounds. Keep the vehicle near the view centre on
    explored ground, mute unrelated sounds through the named test data, and use a short
    distinguishable non-looping sample without entry-level repeat suppression. Lengthen the presentation
    and descent so each lasts at least ten sample durations. Record mixed audio with tick
    timestamps; an isolated sound request is the audibility control. If that control is
    silent, or channel limits or entry suppression could prevent repetitions throughout the
    tested phase, the repetition test is inconclusive. Repeat on our engine with the switch off
    as a design-alternative check, not an oracle measurement.
  - (V) Repeat grounded with no presentation, then airborne with the landing-first flag off.
    Compare a deploy landing with an ordinary landing of the same type from the same height.
    Repeat presentation cases with invented end fields of 1, 2 and 5 stages, different initial
    frames, and with no deployed image type. Record deployed state as well as image and height.
  - (C) Exact-clock variant: grounded, with invented end field *E* = 5, no initial loop delay,
    no random rate and no normalisation. Test base delays of 1 and 2 ticks. For each delay,
    perform ten deploy/undeploy pairs at varied even and odd creation ticks, both alone and
    with other vehicles transitioning and unrelated long-lived animations already present.
    Vary the test vehicle's creation order among the other vehicles. Keep all updates
    uninterrupted and record actual presentation creation, each image-stage change, and
    deployed/mobile state, not merely order-input ticks. Expected deploy durations are 5
    and 9 ticks respectively, undeploy durations 4 and 7 ticks respectively. The first image
    advance is respectively 1 and 2 ticks after creation in either direction, with the
    undeploy image moving backwards. A creation-tick advance, completion on *T* rather than
    *T* + 1, or any dependency on parity, surrounding animations or actor order falsifies
    claim 7. In a separate named-data variant, keep base delay fixed but enable a noncollapsed
    random playback-delay range, then normalisation at two game speeds under S-0109 claims
    3 and 4. Choose a long enough sprite and playback delays that the presentation survives
    through completion. Expected: image changes follow the selected playback delay while
    completion still follows base *d*. Log random state around exclusive completion/count
    intervals as in P2 R, excluding animation creation, effects and other stream consumers:
    no draw chooses a completion tick. Ambiguous intervals are inconclusive. Request a
    read-only tick/image/state recorder and scripted input through the measurement bench.
    no state writes or installed engine changes are proposed.
  - (R) Repeat hovering deployment with the effective-clear and suppressed empty-to-empty
    starting states of S-0502 J22. Record order-level destination, movement commitment,
    retained movement point, flight state and touchdown position. Expected: an effective clear
    reroutes; a suppressed assignment preserves the preceding point and commitment. Descent
    uses the retained point; a different horizontal point causes cruise first. An erased point,
    reroute after a verified suppressed assignment, or descent at a different point falsifies
    claim 5. Also follow S-0502 J24 for refresh nonactivation and J25 for qualifying
    balloon-hover retention: record the target and current/queued behaviours around the
    deploy order's single clear. Expected, if an ordinary stock deployable balloon-hover
    fixture is established: rejection preserves the nonempty destination, point, commitment
    and reservation, can make *attack* current, then target clearing leaves that destination
    intact while *unload* is queued. No second clear or deploy-enabled refresh occurs.
    A reroute on a qualifying rejection, a retry after target clearing, or refresh newly
    enabled by deploy falsifies the respective claim. Without that stock fixture the
    balloon-hover deploy case is diagnostic only. Follow J22's diagnostic-only treatment
    of uncommitted idle hover and J24's retained-refresh limitation; do not create those
    states by writing into the game.
  - (Z) Give the presentation a zero rate and an end field of 5 stages. Deploy, wait 500 ticks,
    then destroy the vehicle. Also destroy a vehicle during ordinary deploy, undeploy and
    landing in separate runs; observe that the cycle and its presentation end.
- Runs: 5 per part, different seeds.
- Expected:
  - (a) the vehicle never lands or deploys (passenger rule first),
  - (b) processing every tick from *t*ₒ to completion at F1's tick; body hidden during the
    transition and deployed image afterwards,
  - (b) after completion the processing timer takes the value 450. The first shot at the enemy
    comes before that, once its targeting timer runs out (claim 3),
  - (c) completion at F2's tick, with a nearby destination assigned and takeoff permitted.
    Remaining over the same cell is not required,
  - (S) the oracle and our engine's default switch setting repeat the relevant sound during
    a sufficiently long landing or transition. With the switch off, our engine requests it
    once at the first step only, not again at touchdown. Audio capture alone does not establish
    every processing's request cadence (claim 12),
  - (V) grounded without a presentation: immediate deploy and undeploy at their first processings.
    Airborne without the landing-first flag: no deploy and return to *guard*. Landing descent
    matches ordinary landing once descent starts; claim 5 permits cruise to a retained point
    first and does not promise to wake uncommitted idle hover. Presentation variants follow
    F1/F2 regardless of initial frame.
    No deployed image type: the vehicle keeps its own image. A landing-first type with no
    presentation becomes mobile without the undeploy itself assigning a destination; record
    its later height without a prediction,
  - (Z) no zero-delay completion during the observation window. Destruction removes the vehicle
    and presentation and ends the cycle in all tested phases.
- Falsified by: a completion outside F1/F2, an incorrect image, a delay other than 1 in transition,
  a timer other than 450 afterwards, absent nearby destination after animated landing-first
  undeploy, a destination assigned by immediate undeploy, no shot before the first *guard* processing despite a legal
  acquired target, a different descent profile, a deploy in (a) or the airborne V variant,
  zero-delay completion in (Z), cycle survival after destruction, or no repeated oracle sound in a valid, audible S run.
  Audio suppression makes a run inconclusive, not a falsifier of the request cadence.

**P2. Deploy-fire vehicle.**
- Setup: *deploy-fire-test* gives a vehicle the deploy-fire flag without the in-place flag,
  an area-gas weapon in slot 0 and a different weapon in slot 1, with deploy weapon index 1.
  Place it on open ground, with enemy infantry in an adjacent cell.
- Procedure: deploy order. Record shots, weapon slot, target cell and processing delays.
  After 300 ticks give a move order. Repeat with slot 0 absent so the firing check refuses
  every shot; observe whether *unload* and its delays continue. Also retain a slot-0 weapon
  but set its minimum range above the own-cell distance, isolating a range refusal.
- Runs: 10, different seeds.
- Expected:
  - every shot is from slot 0, at the vehicle's own cell,
  - each processing delay is 14, 15 or 16, and all three occur over the runs,
  - the behaviour stays *unload* until the move order, including both refusal variants.
- (R) Random-state attribution: for this part only, *deploy-fire-test* replaces the gas
  weapon by a single-shot weapon with no scatter, randomized damage or randomized effects.
  Verify in an ordinary-fire control that the shot itself consumes no scenario-stream draws.
  Use the glossary's read-only random-state recorder, with observations within ticks as well
  as at tick boundaries. For 30 tested processings in each firing and refusal variant, retain
  only intervals that bracket this vehicle's processing and contain no potential stream
  consumer besides its delay draw and the verified draw-free shot: no other actor processing,
  targeting search, projectile update or unrelated creation. Vary the deploy order's tick between runs
  to obtain such intervals; an interval with an ambiguous consumer is void for this part.
  Decode the observed stream transition using S-0202 claims 1 to 5, including any rejection
  draws required by its ranged-draw rule. Expected: exactly one ranged draw over 0 … 2, with
  delay 14 plus its result, even when the shot was refused. Repeat around 30 transition
  processings and 10 completions of P1 without an enemy or weapon: expected no delay draw
  for the 1-tick or 450-tick return. A different attributed draw count, range, delay/result
  pairing, or a draw at those P1 boundaries falsifies the respective claim. Count ranged draws,
  not just underlying generator advances. If the recorder cannot resolve an exclusive interval,
  report this part inconclusive, not confirmation from a whole-tick stream delta.
- Falsified by: a shot from any slot other than 0, a different target cell, a delay outside
  14 … 16, early departure from *unload*, refusal of the later move, or a shot with slot 0 absent.
  Failure to see one allowed delay in finite runs is inconclusive, not a falsifier of uniformity.

**P3. Interrupting.**
- Setup: as P1.
- Procedure:
  - (a) Give an accepted attack order on an in-range enemy during deploying and during
    undeploying in separate runs. Confirm that *attack* becomes queued. Record
    current and queued behaviours through completion. In other runs, issue a move order to a
    deployed vehicle using force-move over an enemy vehicle: an ordinary click on a far cell
    may produce no order (S-0214 claims 8 and 9). Confirm the actual order in the log.
    Also order a deployed vehicle to attack a distant enemy, exercising an approach destination.
  - (b) Once deployed, issue that accepted force-move order; record behaviour and delay.
  - (c) While it is landing for deploy, give a move order; record its height and behaviour.
  - (d) While it is descending for deploy, issue an accepted area-guard behaviour order with no
    destination. Confirm the order and queued behaviour, and log the moving-now state and
    readiness. Record height and behaviour for 300 ticks. Separately repeat with a stop order,
    confirming that stop itself does not replace *unload*. Record subsequent height and
    deployment without a prediction.
- Expected:
  - (a) the queued *attack* does not start during either transition and is replaced by *guard* at
    completion. Neither an accepted move nor an attack approach gives a deployed vehicle a
    destination. A new move order after undeploy completion can move it,
  - (b) *guard* is queued, and the delay is 1,
  - (c) it climbs again, never deploys, and moves to the ordered cell,
  - (d) while the vehicle is moving now in *unload* and has no movement-interrupt allowance,
    the queued area guard does not start. Later height and the separate stop run's deployment
    outcome are recorded for the open questions.
- Falsified by: ordinary movement or a destination while deployed or in transition, queued
  behaviour taking over during a transition, the pending *attack* surviving completion with the
  default quirk, a delay other than 1 in (b), a deploy after (c)
  or touching down in (c) before climbing again, or area guard starting in (d) despite the
  logged moving-readiness refusal.

**P4. Firing refusals.**
- Setup: as P1, plus a named enemy magnetic-beam vehicle in range. A test-map team gives that
  enemy the attack task against the in-place vehicle; scripted player input controls only the
  human-owned deployer. Record the attack task and target to ensure refusal is tested.
- Procedure: (a) deploy order; record the ticks of the beam's shots and lifts and of the in-place
  vehicle's own shots. (b) Continue after completion; record whether the beam lifts the deployed
  vehicle and, if so, its cells and existing destination while seized.
- Expected: (a) no beam shot or new lift while *unload* is current, including landing, or
  during either transition. Its own shots are refused during transitions only: this spec does
  not prohibit firing during descent. Repeat for undeploying. A control before the deploy
  order must show the beam can hit the mobile vehicle; otherwise void the refusal test.
  (b) If a lift is established and an ordinary input route to a new destination is found,
  record the attempted order and destination before and afterwards. Expected: the new
  destination is ignored under S-0607 claim 9, not accepted merely because the vehicle is
  seized. While it is still gripped, being deployed alone does not clear the existing
  destination. Repeat during undeploying only if an accepted deploy order starts that
  transition while the vehicle is still deployed and gripped. If the input route or state
  overlap cannot be established, the respective test is inconclusive, not confirmation.
- (c) Refusal label: in *deploy-cycle-test*, give the transitioning vehicle a negative-damage
  default weapon (a healing weapon), with nonzero effectiveness against a full-health vehicle
  target. During the transition, give an accepted attack order that assigns this target
  without starting *attack* immediately; confirm the target assignment in the log. For the
  beam refusal variant, both vehicles belong to the human house: give the beam attacker the
  healing default weapon with a magnetic warhead and an accepted force-attack order on the
  full-health deployer while the deployer is in the tested state. Void a run whose input does
  not assign that target. Log target retention every tick, not just shots. The healing reply
  of S-0310 claim 11 makes *illegal* observable as target dropping. Repeat the beam test
  during landing with *unload* current, deploy and undeploy. Retain only runs whose target
  is assigned during the tested state, with no independent damage, retargeting, lift or
  other rank-4 refusal.
- (d) Priority: repeat (c) with a positive rearm countdown inherited from an earlier shot
  at a damaged control target. Use a sufficiently long test weapon rate-of-fire field to
  keep the countdown positive throughout the tested boundary. A mobile control with the same
  countdown retains its full-health target, because rearming outranks the healer's own
  lower-ranked *illegal*. In transition, or with the beam's target in the forbidden state,
  expected target dropping despite that countdown distinguishes the earlier *illegal*.
  Repeat the transitioning firer variant while it is launching spawned aircraft (S-0506
  claim 10), if those states overlap: expected no target drop until that higher-ranked busy
  state ends. A run with no verified overlap is void for this last comparison.
- Falsified by: a beam shot or new lift in a forbidden state, an own shot during a transition,
  acceptance of a new destination in a verified seized case, target retention in a
  valid (c) or rearming (d) case, or transition-induced target dropping while the higher-ranked
  busy state holds. These tests distinguish the observable rank groups, not an unstated order
  among reasons within rank 4.

**P5. Deploy-to-fire.** (a) Named test data *deploy-to-fire-test* gives a tank the deploy-to-fire flag,
no other deploy rule, and a weapon that can legally hit the enemy. Test aligned and misaligned
initial facings; record current and queued behaviours as well as facing.
- Procedure: place it, in turn, on:
  - a flat buildable cell,
  - a slope,
  - a flat cell of a land type without the buildable flag,
  - a cell holding part of a building.

  Then order it to attack an enemy in range.
- Variant: also give the type the in-place deploy flag and a presentation.
- Expected:
  - the flat buildable cell gives *unload* queued and no shot,
  - the other three give no shot, no *unload* and no change of facing,
  - the variant deploys, undeploys and deploys again repeatedly, without firing.
- Falsified by: a shot, *unload* queued on a failing cell, no queued *unload* on a passing
  cell despite an otherwise legal shot, turning on a failing cell, or no repacking in the
  variant. Repacking is an oracle/default-switch prediction; with the switch off our engine
  instead fires while deployed.
- (b) Exclusions: repeat the passing-cell case with an allied infantry unit sharing the cell,
  then an enemy infantry unit, and with and without an overlay that leaves the effective land
  type unchanged. Use named test map placements, and record actual occupants and land type at
  the firing check; void a run if the infantry leaves first. Repeat the building-failure case
  with a human-owned, allied, enemy and neutral building. Expected: units, their ownership and
  such an overlay do not change a passing result; a building of any owner still fails.
  Any changed result while the three specified conditions stay identical falsifies the exclusion.
- (c) Approach: use a second layout in *deploy-to-fire-test*, a grounded vehicle with a
  failing own cell and a distant stationary enemy. Compute S-0307 F4's ordered primary cells
  from the recorded positions and S-0215 claim 37 f2's reach. Use a flat unbuildable primary
  followed by a flat buildable primary, with all other relevant cells unchanged. Order attack
  and record the destination. Expected: the unbuildable primary is skipped; in a control
  where its land type is buildable it may be selected. Repeat with slope and building failures.
  For failure anchoring, obstruct all primary destinations and their alternatives, leaving
  nearby-cell fallback exits in two separated patches around the last qualifying primary
  and the later failing primary. Compare with a control changing only that later primary's
  deploy-test result. Record destination and target retention, inferring the failure centre
  from which separated patch supplies the fallback exit.
  Expected: a later deploy-test failure cannot replace the last qualifying failure anchor.
  If all primaries fail this test, none supplies an anchor. Selection of a failing primary
  or a failure centre anchored to one falsifies claim 14. A layout where the fallback patches
  overlap, or no failure centre can be distinguished, is inconclusive and must be redesigned.
  This tests the cell predicate and anchor exclusion only, not the unspecified rest of the search.

**P6. No unload or deploy cycle for vehicles.**
- Setup: named test data *vehicle-cycle-test* gives an empty vehicle transport a positive
  deploy-time field *D* in game minutes, chosen so 900 × *D* ticks greatly exceeds normal order
  latency. Repeat with the field absent and compare timings.
- Procedure: give it a move order across the map; while it moves, give a stop order, and later an
  attack order on an enemy. Record the tick each order is given, destination clearing, and the
  ticks of behaviour changes. Stop itself changes no behaviour (S-0213 claim 8).
- Expected: move and attack become current when ordinary readiness permits (S-0215 claim 11),
  with identical timing in the positive-field and absent-field runs. Destination clearing by stop
  and any later movement-processing fallback also match the control. There is no additional
  wait attributable to 900 × *D* ticks.
- Falsified by: an added order delay relative to the absent-field control attributable to *D*.
  Repeat with in-place, deploy-fire, deploying-into-building and harvester variants in
  *vehicle-cycle-test*, exercising their unload processing as well as move and patrol orders
  where accepted. Record whether an unload or deploy cycle ever starts and whether readiness
  waits on it. Expected: no cycle start and no delay from that field, even though the separate
  deploy transition and passenger-exit readiness gates still apply. Any recorded cycle start
  falsifies claim 15; an unrelated transition wait does not. These finite samples can falsify,
  but cannot prove, the universal absence of starts.

**P7. Deployed infantry.**
- Setup: a stock infantry type that deploys in place and fires when deployed, deployed on open
  ground, *guard* current. Enemy infantry A stands just outside the deploy weapon's range, and enemy
  infantry B inside it.
- Procedure:
  - (a) B absent, attack order on A,
  - (b) B present, attack order on A,
  - (c) B present, no order,
  - (d) attack B while it remains in range,
  - (e) no enemies and no target, first in *guard*, then in *area guard*.

  Record target, shots, behaviour and targeting timer. In (b), retain runs where processing
  searches while the timer still has positive ticks left; otherwise repeat with different timing.
- Expected:
  - (a) the target is dropped and the unit falls back to rest, without moving,
  - (b) the target becomes B,
  - (c) B is acquired,
  - (d) B is retained and fired on,
  - (e) *guard* remains current, whereas targetless *area guard* falls back to rest.
- Falsified by: movement, retaining A, failure to select the only legal in-range B, dropping
  an in-range B without another refusal, waiting for timer expiry in retained (b) runs, or
  falling back from targetless *guard*, or targetless *area guard* not falling back to rest.
- (f) Named test file *deployed-infantry-test* supplies deployable infantry variants with
  and without the deploy-fire flag and with and without the jumpjet flag. Give the non-deploy-fire
  variant two weapons with different ranges and target-based selection, so its no-target
  weapon choice differs from its choice against A. Deploy on the ground, confirm a deploy-kind
  sequence, then repeat (a), (b) and (d), using *attack* to enter S-0215 claim 23's deployed row
  rather than relying on the deploy-fire guard rule. Expected: range retention/search uses
  S-0308 claim 3's no-target choice; a found B becomes the target. For the targetless jumpjet
  case, remove all enemies while *attack* is current: it retains that behaviour rather than
  falling back to rest. A non-jumpjet control falls back. Record target, sequence, chosen
  weapon and behaviour. Wrong range basis, failure to acquire B in a valid search, or fallback
  of the jumpjet case falsifies the respective subclaim. Void a run where no deployed
  sequence or specified calling behaviour is established.

**P8. Overlaps, disguise and image-only substitution.** Use *deploy-overlap-test*, with the
same map, recorder and five-seed repeats as P1. Each variant is a separate newly loaded game.
- (a) Precedence: combine the in-place flag in turn with passenger capacity, harvester, weeder
  and deploys-into-building properties. Provide empty transports, reachable appropriate
  resources/refinery and enough clear space for the named building, as applicable. Also test
  deploy-fire without the in-place flag combined with each of those properties, and a
  passenger-capacity plus harvester plus deploys-into-building type without deploy-fire.
  Issue deploy and record behaviour, passenger events, shots, deployment and object creation.
  Expected: passenger handling wins when capacity is positive; otherwise non-in-place
  deploy-fire uses slot 0; otherwise harvesting wins over building deployment and in-place
  deployment, and building deployment wins over in-place deployment. Seeing the lower-priority
  outcome while both recorded conditions hold falsifies claim 1. Bunkered vehicles are not
  tested by these overlap variants. S-0606 claim 11 establishes ordinary band-box exclusion,
  but no bunker-link-only click-selection refusal. Its scenario K6 separately tests pointing,
  click selection and deploy to establish whether player input reaches *unload*. Failure of
  band-box selection alone does not settle that route. If that route is established, repeat
  K6 with an eligible in-place vehicle supplied by *deploy-overlap-test*, without harvester,
  weeder or building-deploy properties. Test with and without a deploying presentation and
  record release, the first deploy step, transition state, processing delays and queued
  behaviours. Expected: release and the first step occur in the same processing; the cycle
  then follows claim 3, with *guard* replacing the release's queued *move* at its ordinary
  return to *guard* under the default quirk. No first step in that processing, or a different
  cycle delay, falsifies claim 1. Without the input route this variant is diagnostic only.
  No state write may supply *unload*.
- (b) Disguise: give a grounded in-place vehicle permanent disguise capability as in S-0516
  claim 32, with no disguise-when-still reacquisition. Deploy, verify its disguise persists,
  and order undeploy without moving it or inflicting damage. Record the disguise flag at
  the first undeploy processing and completion. Expected: it is cleared at the first step,
  before animated completion, or at the immediate step without a presentation. A retained
  disguise at that step falsifies claim 6. A vehicle losing disguise before the step is void,
  not evidence. Repeat both presentation variants.
- (c) Image only: name a deployed image type with deliberately different selection dimensions,
  size/crowding properties, weapons, armour and maximum health from the actual type. Compare
  mobile and deployed instances, and a control whose deployed image has the original dimensions.
  Record image, selectable screen bounds, health, weapon slot and damage, damage received from
  identical controlled shots, and cell entry/occupancy for a second unit. Expected: the image
  changes, but all recorded non-drawing properties follow the actual vehicle type and ordinary
  deployed-state rules, never the image type. Falsified by a selection-bound, occupancy or combat
  property copied from the image type. Remove enemies between controlled shots to avoid
  additional targeting or damage effects.

## Open questions

- Whether the range search of the deployed-firing rule (claim 16) is subject to the passive
  acquisition and opportunity gate of S-0309 claims 2 and 3. P7 (b) can show only that it finds a
  target; a run in which those gates would refuse is still to be designed.
- What happens to the remaining descent if another rule replaces *unload* without assigning a
  destination during landing for deploy (claim 8). Ordinary behaviour orders may be held by
  moving readiness instead. Separately, what happens after stop, which changes no behaviour.
  P3 (d) records ordinary queueing and stop; no setup for a forced replacement is established.
- Whether any situation forces a behaviour on a vehicle in transition (S-0215 claim 9), which would
  leave it stuck (Edge cases).
- The subsequent height after immediate landing-first undeploy, when there is no presentation
  and hence no assigned destination (claim 6). Animated-undeploy search settings are specified
  by S-0502 claim 23.
- Approved S-0415 claim 1 already specifies the slot-0 weapon and the 14 + *u* tick delay, matching claim 2 here; no cross-spec change is indicated by this point.
- Approved S-0516 claim 34 already specifies disguise loss when the undeploy step starts, matching claim 6 here; no cross-spec change is indicated by this point.
- The rest of the approach search of a deploy-to-fire vehicle whose own cell fails (S-0307, Open
  questions; Q-0072).
