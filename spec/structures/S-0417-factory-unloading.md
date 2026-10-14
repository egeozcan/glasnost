---
id: S-0417
title: Factory unloading (doors, exit movement and rally destinations)
topic: structures
revision: 1
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-14
measurements: []
requests: []
supersedes: []
---

# S-0417: Factory unloading (doors, exit movement and rally destinations)

## Summary

Factory doors, apron clearing, vehicle exit movement and the destination chosen on release from
contact. S-0404 claim 13 owns release positions, facings, busy and failed results, and refunds.
Skirmish only: campaigns and network games are out of scope. This supplies part of Q-0120, but
leaves aircraft docking geometry and infantry initial walk-out unresolved, so does not close it.

## Terms

Contact, tether link, foundation, exit cell, exit displacement, barracks door kind, guard spot,
scatter request, threat point, forced, move-anywhere and locomotor use the glossary and S-0404's
terms. Behaviours, queueing, processing, falling back to rest and standard delay are S-0215's.
Track, drive vehicle, speed fraction ρ, target fraction ρₜ and progress unit are S-0207's.
The nearby-cell search is S-0212 claim 25. Door drawing is S-0108's, animation playback S-0109's.

- **Factory K, unit u:** the building and the unit it has just released. Its foundation's top-left
  cell is (*x*₀, *y*₀).
- **Unloading:** K's current behaviour is *unload*, specifically to release a produced unit.
  Passenger unloading, absorbed units and non-factory buildings are outside this spec.
- **Door:** closed, opening, open or closing. Open and closed mean at rest. Movement takes the
  door time of F2. This spec specifies only the factory-unloading uses of the door.
- **Exit apron cell E:** the target cell of a ground vehicle factory's exit track (claim 3).
  It need not belong to K's foundation, even when the exit list has eleven or more entries.
- **Apron neighbours:** E's eight neighbours, in step-direction order 0 to 7 (S-0204 claim 9).
- **Exit track:** table T1, distinct from the ordinary movement tracks of S-0207.
- **Clear report:** a tethered unit's notice that releases its contact and tether (claim 7).
  The name does not mean that it has physically left the foundation.
- **Run-away reply:** the reply a vehicle factory gives to that report, governing claim 8.

## Behaviour

1. **Factories that unload.** A ground vehicle factory releases u into contact with K and starts
   unloading. A naval vehicle factory starts unloading only when it had no unit in contact before
   release; it does not establish contact with the new ship. An infantry factory and a cloning
   building use contact hand-over, without entering the unloading stages. An aircraft factory
   does not enter these stages. A later release while a ground factory unloads passes to another
   factory or gives busy, exactly as S-0404 claim 13 states; busy does not retain the finished item
   for retry (S-0404 claim 7).
   - Backing: `measurable` (U1)
   - Confidence: high

2. **Ground vehicle unloading stages.** Table S1 gives the stages. Each processing handles one
   stage and assigns one standard-delay draw over 0 … 2, including an unsuccessful processing.
   The delay is F1; it is not necessarily the interval between processing ticks when it is 0 or 1
   (S-0215).
   At unloading start, the contacted unit starts *guard* without the ordinary readiness test, the door starts opening, and K stops its idle animation and starts its production animation when that slot has art (using the damaged variant as S-0108 specifies). Apron clearing proceeds without waiting for the door: a blocked apron receives claim 4's requests and remains in that stage; a free apron advances to letting out without sending u out. Letting out waits for an open, stationary door. If contact is absent, K starts closing; otherwise K queues *move* on u, applies claim 5 and waits for the tether to end. The waiting condition is tether end, not u's position. K then starts closing; closing waits for a stationary closed door, then falls back to rest, stopping the production animation and queuing *guard*. Unloading ends when *guard* starts.
   - Backing: `measurable` (U2 for stages, U3 for delay assignments)
   - Confidence: high

3. **Apron position.** For a foundation with at least eleven exit-list entries, take its eleventh
   pair (*a*, *b*) from S-0404 Constants. E is (*x*₀ + *a* − 1, *y*₀ + *b*). The track ends at
   E's centre, at ground height under the movement rules. S-0404's lists do not generally put E
   inside the foundation: for example, its 3 × 2 word gives (*x*₀, *y*₀ − 1).
   Shorter lists use the special results in Edge cases. Selection always uses the original lists
   in the default configuration. The non-default interaction with S-0404's full rings is open.
   - Backing: `measurable` (U4 for stock factory types). Other foundation words are `unmeasured`:
     no stock vehicle factory using those words is established for this scenario.
   - Confidence: high

4. **Blocked apron.** E is blocked when its ground layer lists an infantry unit, vehicle,
   aircraft or building other than K. The released u counts. Terrain objects and overlays do not.
   On a blocked processing every object in that layer receives a forced, move-anywhere scatter
   request without a threat point. Each apron neighbour that lists at least one such occupant
   other than K also sends a request to every object in its ground layer: forced, move-anywhere,
   with E's centre as threat point. Neighbours receive nothing when E is free.

   S-0209 claim 11 owns receipt by all listed objects; S-0207 claim 16 and S-0208 claims 11 to 13
   own acceptance and destinations. An accepting vehicle in E takes no scatter draw; an accepting
   vehicle in a neighbour takes one ranged draw over 0 … 2, and no acceptance draw over 1 … 4.
   Objects that cannot scatter simply ignore their request. A persistent blocker can hold the
   factory indefinitely. The stream assignments follow E, then the neighbours in direction order,
   then the processing's delay. Within a cell they follow the listing order of S-0209 claim 13.
   - Backing: `measurable` (U5, U3 for assignments)
   - Confidence: high (tests and options); medium (destinations inherited from movement specs)

5. **Sending u out.** A drive vehicle is put on T1 towards E's centre, without a path search
   from its present position. Tunnelling and teleporting vehicles temporarily receive driving
   locomotion and use the same track; restoration belongs to S-0215 claim 13. If that temporary
   replacement cannot be established, successful exit is not promised (Open question 4).
   Any other locomotor instead gets destination (*x*₀ + 4, *y*₀ + 1), independent of the
   foundation and its exit list. It moves by its own locomotor's rules. K sets u's speed fraction
   to 0.5 after this exit instruction in all these cases.
   - Backing: `measurable` (U6). Failed temporary replacement is `unmeasured`: no stock setup is
     established that makes it fail.
   - Confidence: high (driving and fixed destination); medium (temporary replacement)

6. **Exit track and speed.** T1 gives every point and facing. Passing its first point places u
   at E's centre plus (−670, −68) leptons regardless of the appear point. Between-point movement,
   progress cost and completion follow S-0207 claims 3 and 4: a point costs 7 progress units.
   A tick need not have enough progress to pass even the first point, so no universal first-tick
   jump is promised.

   The exit-track instruction sets ρₜ to 1.0, after which K sets ρ to 0.5. A type marked to
   accelerate does not receive ordinary acceleration on this special track, so retains 0.5
   absent another effect. A non-accelerating type copies ρₜ into ρ on its movement update and
   therefore uses 1.0. Speed and integer progress earned are S-0207's, not geometric distance
   divided by the unit's imported speed.
   - Backing: `measurable` (U7 and U6)
   - Confidence: high (track and speed tests); medium (first-point observable consequences)

7. **Clear report.** On a cell-arrival update while tethered, a vehicle reports clear unless it
   is in *unload*, or in *enter* with its contacted building as destination, or both these hold:
   it has a destination other than its present cell, and its cell holds a building. A destination
   object other than a cell is not treated as its own cell for this test. The vehicle-factory
   contact also suppresses a report on that factory's cell with a different destination; the
   building test already covers that case.

   With no destination, or with its own cell as destination, u can report even on K's foundation.
   Thus a normal exit track can report at E itself, without a separate move off the foundation.
   The report ends contact and the tether, enabling K's closing stage.
   - Backing: `measurable` (U2, U8 for normal exits). The *enter* and *unload* exceptions are
     `unmeasured` here: no release setup retaining either on an arrival update is established.
   - Confidence: high (tests); medium (normal arrival consequence)

8. **Run-away destination.** A vehicle factory gives the run-away reply to the clear report.
   These are precedence rules, not a required internal implementation sequence:
   - A destination other than u's current cell is kept instead of the rows below. An additional
     interaction with a destination unit or building remains open (question 7).
   - Otherwise an ore-gathering vehicle queues *harvest* with a readiness-dependent immediate
     start (S-0215's queueing terms).
   - Otherwise a human-controlled unit whose saved guard spot exists and differs from its
     destination takes that spot as destination, without queueing a new *move*. The spot was
     copied from K's rally point at release, not re-read from K at the clear report.
   - Otherwise it discards its stored path and receives a forced scatter request without a
     threat point, **not move-anywhere**. An accepting vehicle without a destination uses the
     nearby-cell search without a random draw. This row also applies when its saved guard spot
     already equals its destination, not just when no rally point was saved.

   In the original, a team member is handled by team logic; otherwise a computer-controlled unit
   leaving a vehicle factory can be sent towards its house's base cell followed by *area guard*.
   Base-cell selection is not a zero-draw rule. In our engine all player houses use the human rows
   and AI chooses its own orders (lead policy). Neutral and special houses retain the original
   non-human rule; this spec does not establish a produced-unit setup for them.

   A tethered vehicle's arrival update also requests a forced, move-anywhere scatter without a
   threat point if, after report handling, it remains on a building cell with no destination,
   no queued destinations and no stored path. This is a fallback, not a mandatory preliminary
   move before the rally point. If the report at E supplies a rally destination, u heads there
   directly. Without a rally, the run-away scatter normally already supplies a destination:
   there is no mandatory second scatter.
   - Backing: `measurable` (U8). The additional destination-object interaction, the neutral-house
     production case and the exact computer-only base-cell choice are `unmeasured`: no precise
     stock setup or complete expected base-cell selection is established here.
   - Confidence: high (human rows, options and saved rally); medium (normal exit consequences)

9. **Naval factory.** A released ship has no new contact with K and keeps the rally move from
   S-0404 claim 13. When K starts unloading, its first processing opens the door and starts the
   production animation, but advances directly to closing. Its second processing falls back to
   rest regardless of door state, stopping the production animation and queueing *guard*.
   There are exactly two unloading processings and two delay draws, no apron test and no exit
   movement commanded by those processings. The door can still be moving after unloading ends.
   - Backing: `measurable` (U9)
   - Confidence: high

10. **Infantry factory and cloning building.** Placement, facing and cloning are S-0404 claim 13
    and claim 14. The released unit saves K's rally point as guard spot. Contact is established
    and K gives clearance to unload, tethering the unit when the contact exchange succeeds.
    A tethered infantry unit reports clear on its next cell-arrival update. An ordinary infantry
    factory does not give the run-away reply: contact and tether simply end. At fall-back to rest,
    the guard-spot row of S-0215 claim 13 can supply the rally destination and queue *move*.
    K has no unloading stages or door movement and the contact exchange itself takes no draw.

    When the just-placed unit already has a destination, and its type is neither jump-jet infantry
    nor marked as a teleporting type, that destination replaces its guard spot, *move* is queued, and its
    destination becomes the exit cell. How initial placement supplies that destination has not
    been traced. This spec therefore does not promise that every infantry unit first walks to
    the exit cell, or give a no-rally walk-out rule. The original's computer-controlled release
    also queues *area guard* and can choose a base-cell guard spot. Our player houses use the
    human rule; neutral and special houses retain the original owner rule.
    - Backing: `measurable` (U10 for ordinary contact and rally outcomes). The initial-destination
      branch and no-rally walk-out are `unmeasured`: their initial movement setup is untraced.
      U10 b is exploratory, not backing for a definite rule. Non-player-house production is also
      `unmeasured`: no stock factory-release setup for it is established.
    - Confidence: medium (contact and guard spot); low (initial walk-out)

11. **Aircraft factory.** It does not enter these factory unloading stages. Contact and
    edge-entry orders belong to S-0404 claim 13. Newly released aircraft tether timing is not
    established here; the glossary's landing example is provisional, not proof of this case.
    Docking geometry and the cell near K remain open.
    - Backing: `measurable` (U1 for no unloading). Tether timing and geometry are `unmeasured`:
      the release-to-landing sequence has not been traced to an exact position or tether tick.
    - Confidence: medium (no unloading); low (unresolved docking details)

12. **Random-stream consequences.** Each vehicle-factory unloading processing assigns exactly
    one ranged delay draw over 0 … 2. A blocked processing assigns the scatter draws of claim 4
    before that delay; the receiving vehicle and its direction therefore get the earlier stream
    values. Opening and closing the door take no draw of their own. Sending out on an ordinary
    drive track, reporting clear and taking the human rally destination take no additional draw
    of their own. An accepting vehicle's no-threat scatter takes none.

    This is not an assertion that every processing tick consumes only one value. Production
    animation creation, start events and replacement effects have S-0202 claim 10 and S-0109's
    draws, assigned before that processing's delay. Failed temporary locomotion may invoke damage
    and death effects; those are not covered by the zero-draw statements. Other actors' processing
    and movement draws, initial placement, cloning creation and the original's computer-only
    base-cell choice are not unloaded-unit delay draws either.
    - Backing: `measurable` (U3). Failed replacement's effects are `unmeasured`: no stock setup
      causing that failure is established.
    - Confidence: high (explicit delay and scatter assignments); medium (downstream effects)

## Formulas

- **F1. Standard delay:** *D* = *D*r + *U*(0, 2) ticks. *D*r is S-0215 F1 in full, including
  double-precision multiplication by 900 ticks per game minute, rounding, signed-integer
  conversion and exceptional values. The rate is imported by S-0003's decimal form, not
  invariably stored as a single. S-0215 owns scheduling for delays of 0 or 1 tick.
- **F2. Door time:** use S-0215 F9's deploy-time conversion for both opening and closing with
  K's imported deploy time *t*. For ordinary nonnegative values,
  *T*d = trunc(RZ₅₃(*t* × 900)) ticks with `quirks.chop_rounding` on, RN₅₃ instead with it off.
  This is double-precision floating-point multiplication followed by truncation toward zero.
  F9 owns the signed 32-bit wrap and exceptional conversion. Negative converted durations
  behave as 0. The door becomes stationary on its timer update once the duration has elapsed.
  Duration 0 completes at the first update, not synchronously with the opening request.
  Backing: `measurable` (U2), confidence high for the shared conversion; exceptional durations
  inherit F9's backing.
- **F3. Apron:** E = (*x*₀ + *a* − 1, *y*₀ + *b*), integer cell coordinates, with the signed
  16-bit result range, −32768 … 32767, reducing each result modulo 65536 into that range. (*a*, *b*) is claim 3's eleventh pair or the
  special pair of Edge cases. E's centre is (256 Eₓ + 128, 256 Eᵧ + 128) leptons.
- **F4. Other-locomotor destination:** (*x*₀ + 4, *y*₀ + 1), integer cells with the same wrapping.

## Constants

| Name (ours) | Value | Unit | Use |
|---|---|---|---|
| Rate and door time conversion | 900 | ticks per game minute | F1, F2 |
| Delay draw | 0 … 2 inclusive | ticks | every unloading processing |
| Apron list position | 11 | entries counted from 1 | claim 3 |
| Apron displacement from that pair | (−1, 0) | cells | F3 |
| Exit speed fraction | 0.5 | fraction | claim 5 |
| Exit target fraction | 1.0 | fraction | claim 6 |
| Fixed fallback destination displacement | (4, 1) | cells | F4 |
| Exit track count | 69 | points | T1 |
| Exit track facing | 64 | 1/256 turn | 16384 facing units, map east |

**T1. Exit track.** Integer displacements in leptons from E's centre, in driven order. Every
point has facing 64. For points 1 to 68, *y* = *n* − 69 leptons, where *n* counts from 1.

| Points | x | y |
|---|---|---|
| 1 to 3 | −670, −660, −650 | −68, −67, −66 |
| 4 | −639 | −65 |
| 5 to 68 | −630 + 10 (*n* − 5) | *n* − 69 |
| 69 | 0 | 0 |

## State and ordering

**S1. Ground factory unloading.** Every row assigns F1's delay. Door updates and behaviour
processing are distinct; S-0203 owns their tick ordering.

| State | Event at processing | Effect | Next state |
|---|---|---|---|
| starting | always | contacted unit starts guard, door opening, production animation | apron clearing |
| apron clearing | blocked | scatter requests | apron clearing |
| apron clearing | free | no exit movement yet | letting out |
| letting out | door not stationary open | none | letting out |
| letting out | door open, contact absent | start closing | closing |
| letting out | door open, contact present | queue move, exit instruction, set fraction | waiting |
| waiting | tether remains | none | waiting |
| waiting | tether ended | start closing | closing |
| closing | door not stationary closed | none | closing |
| closing | door closed | stop production animation, queue guard | ends when guard starts |

The vehicle cannot be sent out before the third processing, even with a clear apron and zero
door time. The waiting stage need not observe an empty foundation: contact can end at E. A
factory processing after that report can start closing on the same tick or a later one, depending
on actor update order, rather than necessarily one tick after physical departure. Random ordering
requirements concern the resulting direction choices, delay and stream state (claim 12).

## Edge cases and quirks

- **Short exit lists: proposed `quirks.factory_apron_short_list`.** The original has definite
  special results, not arbitrary results outside the list. For the 2 × 1 and 1 × 2 words the
  pair used by F3 is (32767, 32767) cells. For the 1 × 1, 3 × 3 refinery, 6 × 4 and 0 × 0 words
  it is (0, 0) cells. The former give an outlying apron, with signed wrapping; the latter give
  (*x*₀ − 1, *y*₀). Original results are the proposed default. The fixed alternative and behaviour
  when the outlying cell is not legal are left to question 5; no invented last-entry fallback is
  prescribed. Backing: `unmeasured` (no stock ground vehicle factory with these words is known).
  Confidence high (pairs), low (invalid-cell consequences).
- **`quirks.factory_fixed_hover_exit`.** Proposed default original: non-driving locomotors other
  than the temporarily replaced two use F4, regardless of foundation. Fixed alternative: use E.
  Backing: `measurable` (U6 c for default); confidence high.
- **Refusal blocks release movement.** An occupant refusing scatter can keep the factory in
  apron clearing indefinitely. This is the ordinary refusal rule, not an extra factory quirk.
  Backing: `measurable` (U5 d); confidence high.
- **Released unit on apron.** Only K is excluded from the blocker test. If placement puts u in E,
  it receives the clearing request as well. It moves only if S-0207's acceptance gates pass:
  being in guard alone does not guarantee acceptance, for example while its body is turning.
  Backing: `unmeasured` (no stock exit placement on E established); confidence high (request),
  medium (subsequent interaction). The track still ignores its appear point if later instructed.
- **Unsuccessful track instruction.** A target-cell or unit-state refusal does not by itself
  prove u remains motionless forever: the instruction can leave partial movement state. Subsequent
  movement and contact loss have not been traced. Backing: `unmeasured` (no stock setup established).
  Confidence low. No permanent-wait rule is prescribed (question 4).
- **Contact lost before letting out.** K still clears the apron and waits for an open door before
  closing without a contacted unit. Backing: `measurable` (U2 b); confidence high.
- **Changing rally after release.** Ground vehicles retain the saved rally destination rather
  than reading a later change at their clear report. Backing: `measurable` (U8 d); confidence high.

## Import mapping

| Key | File and section | Concept and parse form | Absent default |
|---|---|---|---|
| `DeployTime` | rules, building type | decimal game minutes, door time | held value, engine default 0 (S-0215) |
| `Rate` in `[Unload]` | rules | decimal game minutes, behaviour rate | S-0215 |
| `WeaponsFactory` | rules, building type | boolean vehicle factory kind | no (S-0404) |
| `Naval` | rules, building type | boolean naval kind | no (S-0404) |
| `Foundation` | art, building type | foundation word and exit list | S-0406 |
| `ExitCoord` | rules, building type | integer triple in leptons, release point | (0, 0, 0) (S-0404) |
| `Locomotor` | rules, vehicle type | locomotor identifier, exit mode | S-0207 |
| `Accelerates` | rules, vehicle type | boolean speed-update choice | S-0207 |
| `JumpJet`, `Teleporter` | rules, infantry type | boolean exceptions to initial exit destination | S-0208 and S-0402 |

## Measurement scenarios

Use stock data, no engine add-ons, and mods off, a tiny flat clear map, one human house and one bystander computer player, fixed game speed index 2 (S-0201 supplies real-time pacing). Run each part with three recorded seeds. Use scripted input and a read-only recorder for tick numbers, positions, destinations, contact/tether, behaviours and scenario stream state. These are recorder proposals, not claims that all fields are currently exposed. Apply the bystander void rule: repeat a run if an unnamed bystander object enters the observed area or acts on a test object, or if the bystander is defeated before observation ends. Do not assume the bystander remains idle. Separate other actors' draws and animation events from unloading draws; recording only the global tick draw count is insufficient. Do not change data to manufacture the unmeasured cases above.

- **U1, factory kinds.** Produce a vehicle, ship, infantry and aircraft from their respective
  factories. Record factory behaviours and door state after release. Expect only the vehicle and
  naval factories to enter unload. Falsifier: infantry or aircraft factory entering these stages.
  Compare busy release/refund with S-0404 P12, not a retained-item retry.
- **U2, door and stages.** With a clear apron, produce a vehicle and record every processing,
  door timer update, movement start, contact loss and queued guard. Expect F2 duration, at least
  three processings before sending out, and closing enabled by tether loss rather than physical
  departure. Falsifier: exit instruction before the door is stationary open or before processing
  three. Part b: destroy the contacted vehicle while a persistent blocker occupies E, then remove
  the blocker. Expect clearing to continue until E is free, then closure after the door opens.
  Immediate closure while blocked falsifies the rule.
- **U3, draw assignments.** Repeat U2 and U5 with the random-state recorder. Label factory
  processing and production-animation events. Expect one ranged delay assignment per processing.
  A blocked processing assigns accepting neighbour vehicles' direction draws first, in direction
  and cell-list order. Starting animation may add its own S-0202 draws. Falsifier: a second
  unloading delay assignment, a missing assignment, or a reversed assignment to a scatter
  direction and the delay. Include U9's two delay assignments.
- **U4, apron.** For each stock vehicle factory type place a persistent refusing unit on candidate
  cells in separate map runs, using pre-placement where normal orders cannot reach them.
  Record processing state and movement. Only occupancy of F3's E should hold apron clearing.
  Falsifier: another candidate alone holding it, or E not holding it. A stock scenario that cannot
  put a blocker there supplies no backing for that foundation word.
- **U5, scatter.** Part a: idle vehicle in E, expect no scatter draw and S-0212's nearby-cell
  destination. Part b: also idle vehicle in a neighbour, expect one 0 … 2 draw and S-0207's
  threat-point candidate selection. Part c: neighbour only, E free, expect no clearing request.
  Part d: deployed or sticky blocker in E, expect repeated clearing and no sending out until it
  leaves. Record requests, destinations and factory state. Falsifiers: a vehicle no-threat draw,
  neighbour clearing with E free, or exit instruction while E remains blocked.
- **U6, exit modes.** Produce a stock drive, teleporting and hovering vehicle; use a stock
  tunnelling type only if one is available from a vehicle factory. Record every position and
  locomotor change. Driving and successful temporary driving use T1, with first-point position
  independent of release displacement. Hovering uses F4, not T1. Falsifier: a different track or
  a hovering vehicle following T1. Initial fractional-progress ticks are S-0207's, not necessarily
  a complete first-point jump. Unavailable stock types remain untested.
- **U7, speed.** Produce stock accelerating and non-accelerating drive types and record speed
  fractions, per-tick positions and arrival time. Calculate each prediction using its own imported
  speed, bonuses, S-0207 progress formula and T1. Expect 0.5 retained for accelerating types,
  1.0 on movement update for non-accelerating types, absent another effect. Falsifier: ordinary
  acceleration increasing the former on this track or half speed retained by the latter. Do not
  compare raw tick counts of different-speed types as though their speeds were equal.
- **U8, clear and rally.** Part a: saved rally point, expect the normal clear report at E and
  direct rally destination, without a mandatory intervening nearby-cell stop. Part b: no rally,
  expect one run-away scatter destination, not a required two-scatter sequence. Part c: ore
  gatherer, expect harvest queued at the reply. Part d: set rally A, release, change to B before
  clear report, expect A retained. Record destinations and tether tick at track end. Falsifiers:
  no report until after leaving K despite no destination at E, a mandatory intermediate stop in
  a, two obligatory scatter destinations in b, or B replacing saved A in d.
- **U9, naval.** Produce a ship with rally set. Record two unload processings, two delay
  assignments, then guard queued; the ship keeps its release move. Door animation can continue
  after unloading. Falsifier: a third unload processing or unload redirecting the ship.
- **U10, infantry.** Part a: rally set, record saved guard spot, contact, tether, initial
  destination and first cell arrival. Expect the successful contact exchange to tether and the
  subsequent clear report to end contact without a run-away reply; ordinary rally movement uses
  S-0215's guard-spot rule. Falsifiers: barracks unload stages or door movement, or that contact
  exchange taking an additional random draw. Part b: no rally, record initial destination and
  position after 300 ticks. This explores initial walk-out; there is no established expected
  first destination against which to score it.

## Open questions

1. Aircraft docking position and release-to-landing tether timing are untraced.
2. The aircraft's cell near K without a rally point remains S-0404's open question.
3. Infantry initial placement destination and walk-out without rally remain open. S-0404 claim 13's
   universal promise of a walk to the exit cell needs qualification: only the traced
   pre-existing-destination branch supplies it (claim 10 here).
4. Failed temporary driving replacement and refused exit-track instructions: subsequent damage,
   movement and contact loss remain untraced. They must not be implemented as guaranteed
   permanent waits on the evidence here.
5. The owner must decide the fixed alternative of the proposed short-list quirk and invalid-apron
   handling. The original's special pairs are definite, not an undefined last-entry fallback.
6. With S-0404's foundation exit-list quirk off, should this track retain its original E or use
   the full-ring eleventh entry? Proposed: retain the original E. Await owner decision.
7. Existing destination of a unit or building at the run-away reply: the additional interaction
   is untraced; no stock produced-unit setup is established.
8. Q-0120 remains partial. S-0404 should point here for ground and naval unloading and qualified
   post-contact rules; its infantry exit-search entry test and contact-place count remain open.
   S-0215 claim 34 and S-0108's door terms should point here. S-0207 claim 16's statement that
   only bunkers send threat-point scatter needs correction: apron-neighbour clearing is a forced,
   move-anywhere sender and may supply a measurement setup for its destination/turret-turn gate.
   Its known-senders claim also needs qualification: factory clearing does not filter moving
   occupants before sending requests; receivers still apply their own refusal tests.
9. Factory release supplies candidate tether setups for S-0213, S-0208 and S-0209 backings that
   currently rely on no known tether situation. S-0215's temporary-locomotor backing likewise
   has claim 5's candidate setup. Those specs must check whether their particular effect can be
   isolated before upgrading backing. The glossary should add exit apron cell and reference
   these concrete tether situations, while keeping aircraft tether timing provisional.
